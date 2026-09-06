use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    time::Duration,
};

use rusqlite::{Connection, TransactionBehavior, params};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::protocol::{
    DomainEvent, EventSink, MAX_SEQUENCE, OpenWorkspace, Result, Snapshot, SubscriptionAck,
    TradeXError, Workspace,
};

const APPLICATION_ID: u32 = 0x54525831;
const SCHEMA_VERSION: u32 = 1;

pub struct Store {
    connection: Connection,
    _lock: File,
    pub path: PathBuf,
}

fn storage_error(_: impl std::fmt::Debug) -> TradeXError {
    TradeXError::new("WORKSPACE_OPEN_FAILED")
}

pub fn directory(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() || path.as_os_str().len() > 4096 {
        return Err(TradeXError::new("WORKSPACE_PATH_INVALID"));
    }
    fs::create_dir_all(path).map_err(storage_error)?;
    let canonical = path.canonicalize().map_err(storage_error)?;
    if !canonical.is_dir() || canonical.to_str().is_none() {
        return Err(TradeXError::new("WORKSPACE_PATH_INVALID"));
    }
    Ok(canonical)
}

impl Store {
    pub fn open(path: PathBuf, options: &OpenWorkspace) -> Result<Self> {
        let lock_path = path.join(".tradex.lock");
        let db_path = path.join("workspace.sqlite3");
        for file in [&lock_path, &db_path] {
            if file
                .symlink_metadata()
                .is_ok_and(|m| m.file_type().is_symlink())
            {
                return Err(TradeXError::new("WORKSPACE_PATH_INVALID"));
            }
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)
            .map_err(storage_error)?;
        lock.try_lock()
            .map_err(|_| TradeXError::new("WORKSPACE_BUSY"))?;
        let existing = db_path.exists();
        let mut connection = Connection::open(&db_path).map_err(storage_error)?;
        connection
            .busy_timeout(Duration::from_secs(2))
            .map_err(storage_error)?;
        if existing {
            let valid: String = connection
                .query_row("PRAGMA integrity_check", [], |r| r.get(0))
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if valid != "ok" {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let app_id: u32 = connection
                .pragma_query_value(None, "application_id", |r| r.get(0))
                .map_err(storage_error)?;
            if app_id != APPLICATION_ID {
                return Err(TradeXError::new("WORKSPACE_SCHEMA_UNSUPPORTED"));
            }
        }
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(storage_error)?;
        if version > SCHEMA_VERSION {
            return Err(TradeXError::new("WORKSPACE_SCHEMA_UNSUPPORTED"));
        }
        if version < SCHEMA_VERSION {
            if existing {
                let backup = path.join(format!("before-migration-{}.sqlite3", Uuid::new_v4()));
                connection
                    .backup("main", backup, None)
                    .map_err(storage_error)?;
            }
            let tx = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage_error)?;
            tx.execute_batch(
                "CREATE TABLE workspace (
                singleton INTEGER PRIMARY KEY CHECK(singleton=1),
                workspace_id TEXT NOT NULL UNIQUE, name TEXT NOT NULL, base_currency TEXT NOT NULL,
                created_at TEXT NOT NULL, last_opened_at TEXT NOT NULL,
                last_sequence INTEGER NOT NULL CHECK(last_sequence >= 0));
                CREATE TABLE outbox (
                    sequence INTEGER PRIMARY KEY CHECK(sequence > 0),
                    event_id TEXT NOT NULL UNIQUE, envelope TEXT NOT NULL);
                PRAGMA application_id=1414682673;
                PRAGMA user_version=1;",
            )
            .map_err(storage_error)?;
            let now = timestamp()?;
            let name = options.name.as_deref().unwrap_or_else(|| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("TradeX workspace")
            });
            tx.execute(
                "INSERT INTO workspace VALUES (1, ?1, ?2, ?3, ?4, ?4, 0)",
                params![
                    Uuid::new_v4().to_string(),
                    name,
                    options.base_currency.as_deref().unwrap_or("USD"),
                    now
                ],
            )
            .map_err(storage_error)?;
            tx.commit().map_err(storage_error)?;
        }
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(storage_error)?;
        if integrity != "ok" {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(storage_error)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(storage_error)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(storage_error)?;
        Ok(Self {
            connection,
            _lock: lock,
            path,
        })
    }

    pub fn record_open(&mut self) -> Result<DomainEvent> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let mut workspace = read_workspace(&tx, &self.path)?;
        workspace.last_opened_at = timestamp()?;
        let previous: i64 = tx
            .query_row(
                "SELECT last_sequence FROM workspace WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if previous < 0 || previous >= MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let sequence = previous + 1;
        let event = DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: "workspace.opened".into(),
            schema_version: 1,
            occurred_at: workspace.last_opened_at.clone(),
            aggregate_type: "workspace".into(),
            aggregate_id: workspace.workspace_id.clone(),
            sequence: sequence as u64,
            payload: workspace,
        };
        tx.execute(
            "UPDATE workspace SET last_opened_at=?1, last_sequence=?2 WHERE singleton=1",
            params![event.occurred_at, sequence],
        )
        .map_err(storage_error)?;
        tx.execute(
            "INSERT INTO outbox VALUES (?1, ?2, ?3)",
            params![
                sequence,
                event.event_id,
                serde_json::to_string(&event).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(event)
    }

    pub fn snapshot(&mut self) -> Result<Snapshot> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        let workspace = read_workspace(&tx, &self.path)?;
        let sequence: i64 = tx
            .query_row(
                "SELECT last_sequence FROM workspace WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(Snapshot {
            aggregate_type: "workspace".into(),
            aggregate_id: workspace.workspace_id.clone(),
            projection: workspace,
            last_sequence: u64::try_from(sequence).map_err(storage_error)?,
        })
    }

    pub fn replay(&mut self, after: u64, sink: &EventSink) -> Result<SubscriptionAck> {
        let snapshot = self.snapshot()?;
        if after > snapshot.last_sequence {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let mut statement = self
            .connection
            .prepare(
                "SELECT sequence,event_id,envelope FROM outbox WHERE sequence>?1 ORDER BY sequence",
            )
            .map_err(storage_error)?;
        let mut rows = statement.query([after as i64]).map_err(storage_error)?;
        let mut expected = after + 1;
        while let Some(row) = rows.next().map_err(storage_error)? {
            let sequence: i64 = row.get(0).map_err(storage_error)?;
            if sequence != expected as i64 {
                return Err(TradeXError::new("IPC_REPLAY_UNAVAILABLE"));
            }
            let event_id: String = row.get(1).map_err(storage_error)?;
            let encoded: String = row.get(2).map_err(storage_error)?;
            let event: DomainEvent = serde_json::from_str(&encoded)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if event.sequence != expected
                || event.event_id != event_id
                || event.schema_version != 1
                || event.aggregate_id != snapshot.aggregate_id
                || event.aggregate_type != "workspace"
                || event.event_type != "workspace.opened"
                || event.payload.workspace_id != snapshot.aggregate_id
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            if !sink(event) {
                return Err(TradeXError::new("IPC_SUBSCRIPTION_DELIVERY_FAILED"));
            }
            expected += 1;
        }
        if expected - 1 != snapshot.last_sequence {
            return Err(TradeXError::new("IPC_REPLAY_UNAVAILABLE"));
        }
        Ok(SubscriptionAck {
            aggregate_type: snapshot.aggregate_type,
            aggregate_id: snapshot.aggregate_id,
            after_sequence: after,
            last_sequence: snapshot.last_sequence,
            replayed_count: expected - 1 - after,
        })
    }
}

fn read_workspace(connection: &Connection, path: &Path) -> Result<Workspace> {
    connection.query_row("SELECT workspace_id,name,created_at,last_opened_at,base_currency FROM workspace WHERE singleton=1", [], |row| Ok(Workspace {
        workspace_id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, last_opened_at: row.get(3)?,
        base_currency: row.get(4)?, path: path.to_string_lossy().into_owned(), storage_schema_version: SCHEMA_VERSION,
    })).map_err(storage_error)
}

fn timestamp() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(storage_error)
}

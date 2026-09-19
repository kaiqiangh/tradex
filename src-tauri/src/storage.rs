use std::{
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write},
    path::{Component, Path, PathBuf},
    time::Duration,
};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::gateway::GatewayState;
use crate::market;
use crate::model::ModelState;
use crate::protocol::{
    Artifact, ArtifactContent, ArtifactExport, ArtifactExportResult, ArtifactKind, ArtifactLibrary,
    ArtifactSummary, DomainEvent, DomainProjection, EventSink, MAX_SEQUENCE, OpenWorkspace, Result,
    SavedScreener, ScreenerLibrary, ScreenerResultState, ScreenerSave, ScreenerUpdate, Snapshot,
    SubscriptionAck, Thread, ThreadList, ThreadSummary, TradeXError, Watchlist, WatchlistItem,
    Watchlists, Workspace,
};
use crate::providers::{AccountConnection, ConnectionState};
use crate::risk::RiskPolicyState;

const APPLICATION_ID: u32 = 0x54525831;
pub(crate) const SCHEMA_VERSION: u32 = 9;

pub struct Store {
    connection: Connection,
    _lock: File,
    pub path: PathBuf,
}

pub(crate) fn storage_error(_: impl std::fmt::Debug) -> TradeXError {
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
            if version == 0 {
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
            }
            if version < 2 {
                tx.execute_batch("ALTER TABLE outbox RENAME TO workspace_outbox_legacy;
                CREATE TABLE outbox (
                    aggregate_type TEXT NOT NULL, aggregate_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence>0), event_id TEXT NOT NULL UNIQUE, envelope TEXT NOT NULL,
                    PRIMARY KEY(aggregate_type,aggregate_id,sequence));
                INSERT INTO outbox SELECT 'workspace',workspace_id,sequence,event_id,envelope FROM workspace_outbox_legacy,workspace;
                DROP TABLE workspace_outbox_legacy;
                CREATE TABLE accounts (
                    connection_id TEXT PRIMARY KEY, provider_id TEXT NOT NULL, environment TEXT NOT NULL,
                    remote_identity TEXT, sequence INTEGER NOT NULL CHECK(sequence>0),
                    credential_ref TEXT NOT NULL UNIQUE, projection TEXT NOT NULL,
                    UNIQUE(provider_id,environment,remote_identity));
                PRAGMA user_version=2;").map_err(storage_error)?;
            }
            if version < 3 {
                tx.execute_batch("CREATE TABLE model_gateway (singleton INTEGER PRIMARY KEY CHECK(singleton=1), sequence INTEGER NOT NULL CHECK(sequence>0), projection TEXT NOT NULL); PRAGMA user_version=3;").map_err(storage_error)?;
            }
            if version < 4 {
                tx.execute_batch("CREATE TABLE model_state (singleton INTEGER PRIMARY KEY CHECK(singleton=1), sequence INTEGER NOT NULL CHECK(sequence>0), projection TEXT NOT NULL); PRAGMA user_version=4;").map_err(storage_error)?;
            }
            if version < 5 {
                tx.execute_batch("CREATE TABLE risk_state (singleton INTEGER PRIMARY KEY CHECK(singleton=1), sequence INTEGER NOT NULL CHECK(sequence>0), projection TEXT NOT NULL); PRAGMA user_version=5;").map_err(storage_error)?;
            }
            if version < 6 {
                tx.execute_batch("CREATE TABLE threads (thread_id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, sequence INTEGER NOT NULL CHECK(sequence>=0), projection TEXT NOT NULL); CREATE INDEX threads_workspace_updated ON threads(workspace_id, sequence DESC); PRAGMA user_version=6;").map_err(storage_error)?;
            }
            if version < 7 {
                tx.execute_batch("CREATE TABLE watchlists (
                    watchlist_id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    name TEXT NOT NULL COLLATE NOCASE,
                    sequence INTEGER NOT NULL CHECK(sequence>0),
                    projection TEXT NOT NULL,
                    UNIQUE(workspace_id,name)
                );
                CREATE INDEX watchlists_workspace_order ON watchlists(workspace_id,name COLLATE NOCASE,watchlist_id);
                PRAGMA user_version=7;").map_err(storage_error)?;
            }
            if version < 8 {
                tx.execute_batch("CREATE TABLE screeners (
                    screener_id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    name TEXT NOT NULL COLLATE NOCASE,
                    sequence INTEGER NOT NULL CHECK(sequence>0),
                    projection TEXT NOT NULL,
                    UNIQUE(workspace_id,name)
                );
                CREATE INDEX screeners_workspace_order ON screeners(workspace_id,name COLLATE NOCASE,screener_id);
                PRAGMA user_version=8;").map_err(storage_error)?;
            }
            if version < 9 {
                tx.execute_batch("CREATE TABLE artifacts (
                    artifact_id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    title TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence>0),
                    projection TEXT NOT NULL
                );
                CREATE INDEX artifacts_workspace_order ON artifacts(workspace_id,sequence DESC,artifact_id);
                PRAGMA user_version=9;").map_err(storage_error)?;
            }
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
            payload: DomainProjection::Workspace(workspace),
        };
        tx.execute(
            "UPDATE workspace SET last_opened_at=?1, last_sequence=?2 WHERE singleton=1",
            params![event.occurred_at, sequence],
        )
        .map_err(storage_error)?;
        tx.execute(
            "INSERT INTO outbox VALUES ('workspace', ?4, ?1, ?2, ?3)",
            params![
                sequence,
                event.event_id,
                serde_json::to_string(&event).map_err(storage_error)?,
                event.aggregate_id
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
            projection: DomainProjection::Workspace(workspace),
            last_sequence: u64::try_from(sequence).map_err(storage_error)?,
        })
    }

    pub fn replay(
        &mut self,
        kind: &str,
        id: &str,
        after: u64,
        sink: &EventSink,
    ) -> Result<SubscriptionAck> {
        let snapshot = self.snapshot_for(kind, id)?;
        if after > snapshot.last_sequence {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let mut statement = self
            .connection
            .prepare(
                "SELECT sequence,event_id,envelope FROM outbox WHERE aggregate_type=?1 AND aggregate_id=?2 AND sequence>?3 ORDER BY sequence",
            )
            .map_err(storage_error)?;
        let mut rows = statement
            .query(params![kind, id, after as i64])
            .map_err(storage_error)?;
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
                || event.aggregate_type != kind
                || match kind {
                    "workspace" => event.event_type != "workspace.opened",
                    "account" => event.event_type != "account.health.changed",
                    "model-gateway" => event.event_type != "model.gateway.changed",
                    "model" => !matches!(
                        event.event_type.as_str(),
                        "model.provider.changed" | "model.provider_attempt.changed"
                    ),
                    "risk" => event.event_type != "risk.policy.changed",
                    "thread" => !matches!(
                        event.event_type.as_str(),
                        "thread.created" | "thread.updated"
                    ),
                    _ => return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED")),
                }
                || event.payload.id() != snapshot.aggregate_id
                || event.payload.kind() != kind
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
    pub fn workspace_id(&self) -> Result<String> {
        self.connection
            .query_row("SELECT workspace_id FROM workspace", [], |r| r.get(0))
            .map_err(storage_error)
    }

    pub fn accounts(&self) -> Result<Vec<AccountConnection>> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare("SELECT connection_id, projection FROM accounts ORDER BY rowid")
            .map_err(storage_error)?;
        let rows = query
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?;
        rows.map(|row| {
            let (connection_id, encoded) = row.map_err(storage_error)?;
            let account: AccountConnection =
                serde_json::from_str(&encoded).map_err(storage_error)?;
            if account.connection_id != connection_id {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            account.validate_persisted(&workspace_id)?;
            Ok(account)
        })
        .collect()
    }

    pub fn account(&self, id: &str) -> Result<AccountConnection> {
        let encoded: String = self
            .connection
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(|_| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
        let account: AccountConnection = serde_json::from_str(&encoded).map_err(storage_error)?;
        if account.connection_id != id {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        account.validate_persisted(&self.workspace_id()?)?;
        Ok(account)
    }

    pub fn threads(&self) -> Result<ThreadList> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare("SELECT thread_id, projection FROM threads WHERE workspace_id=?1 ORDER BY sequence DESC")
            .map_err(storage_error)?;
        let rows = query
            .query_map([workspace_id.as_str()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?;
        let mut summaries = Vec::new();
        for row in rows {
            let (thread_id, encoded) = row.map_err(storage_error)?;
            let thread: Thread = serde_json::from_str(&encoded).map_err(storage_error)?;
            if thread.thread_id != thread_id || thread.workspace_id != workspace_id {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            summaries.push(ThreadSummary {
                thread_id: thread.thread_id,
                workspace_id: thread.workspace_id,
                title: thread.title,
                updated_at: thread.updated_at,
                default_agent_mode: thread.default_agent_mode,
                default_execution_context: thread.default_execution_context,
                status: thread.status,
            });
        }
        Ok(ThreadList { threads: summaries })
    }

    pub fn thread(&self, id: &str) -> Result<Thread> {
        let encoded: String = self
            .connection
            .query_row(
                "SELECT projection FROM threads WHERE thread_id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(|_| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
        let thread: Thread = serde_json::from_str(&encoded).map_err(storage_error)?;
        if thread.thread_id != id || thread.workspace_id != self.workspace_id()? {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(thread)
    }

    pub fn save_thread(&mut self, mut thread: Thread, event_type: &str) -> Result<DomainEvent> {
        if thread.workspace_id != self.workspace_id()? || thread.thread_id.is_empty() {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        if !matches!(event_type, "thread.created" | "thread.updated") {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let previous: i64 = tx
            .query_row(
                "SELECT COALESCE((SELECT sequence FROM threads WHERE thread_id=?1),0)",
                [&thread.thread_id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if previous < 0 || previous >= MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let sequence = previous + 1;
        thread.state_version = format!("thread:{}:{}", thread.thread_id, sequence);
        thread.updated_at = timestamp()?;
        tx.execute(
            "INSERT INTO threads(thread_id,workspace_id,sequence,projection) VALUES(?1,?2,?3,?4) ON CONFLICT(thread_id) DO UPDATE SET workspace_id=excluded.workspace_id,sequence=excluded.sequence,projection=excluded.projection",
            params![
                &thread.thread_id,
                &thread.workspace_id,
                sequence,
                serde_json::to_string(&thread).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        let event = DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            schema_version: 1,
            occurred_at: thread.updated_at.clone(),
            aggregate_type: "thread".into(),
            aggregate_id: thread.thread_id.clone(),
            sequence: sequence as u64,
            payload: DomainProjection::Thread(Box::new(thread)),
        };
        tx.execute(
            "INSERT INTO outbox VALUES('thread',?1,?2,?3,?4)",
            params![
                event.aggregate_id,
                sequence,
                event.event_id,
                serde_json::to_string(&event).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(event)
    }

    pub fn snapshot_for(&mut self, kind: &str, id: &str) -> Result<Snapshot> {
        if kind == "workspace" {
            let snapshot = self.snapshot()?;
            return if snapshot.aggregate_id == id {
                Ok(snapshot)
            } else {
                Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))
            };
        }
        if kind == "model-gateway" && self.workspace_id()? == id {
            let gateway = self.gateway()?;
            let sequence: i64 = self
                .connection
                .query_row(
                    "SELECT sequence FROM model_gateway WHERE singleton=1",
                    [],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::Gateway(gateway),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind == "model" && self.workspace_id()? == id {
            let model = self.model()?;
            let sequence: i64 = self
                .connection
                .query_row(
                    "SELECT sequence FROM model_state WHERE singleton=1",
                    [],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::Model(model),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind == "risk" && self.workspace_id()? == id {
            let risk = self.risk()?;
            let sequence: i64 = self
                .connection
                .query_row(
                    "SELECT sequence FROM risk_state WHERE singleton=1",
                    [],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::Risk(risk),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind == "thread" {
            let thread = self.thread(id)?;
            let sequence: i64 = self
                .connection
                .query_row(
                    "SELECT sequence FROM threads WHERE thread_id=?1",
                    [id],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::Thread(Box::new(thread)),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind != "account" {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let account = self.account(id)?;
        let sequence: i64 = self
            .connection
            .query_row(
                "SELECT sequence FROM accounts WHERE connection_id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        Ok(Snapshot {
            aggregate_type: kind.into(),
            aggregate_id: id.into(),
            projection: DomainProjection::Account(Box::new(account)),
            last_sequence: u64::try_from(sequence).map_err(storage_error)?,
        })
    }

    pub fn gateway(&self) -> Result<GatewayState> {
        let data: String = self
            .connection
            .query_row(
                "SELECT projection FROM model_gateway WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        serde_json::from_str(&data).map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))
    }

    pub fn model(&self) -> Result<ModelState> {
        let data: String = self
            .connection
            .query_row(
                "SELECT projection FROM model_state WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        let model: ModelState = serde_json::from_str(&data)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        if model.workspace_id != self.workspace_id()? {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(model)
    }

    pub fn model_or_new(&self) -> Result<ModelState> {
        let data: Option<String> = self
            .connection
            .query_row(
                "SELECT projection FROM model_state WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        match data {
            Some(data) => {
                let model: ModelState = serde_json::from_str(&data)
                    .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
                if model.workspace_id != self.workspace_id()? {
                    return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
                }
                Ok(model)
            }
            None => Ok(ModelState::new(self.workspace_id()?)),
        }
    }

    pub fn risk(&self) -> Result<RiskPolicyState> {
        let workspace_id = self.workspace_id()?;
        let data: String = self
            .connection
            .query_row(
                "SELECT projection FROM risk_state WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        RiskPolicyState::from_persisted_json(&data, &workspace_id)
    }

    pub fn risk_or_new(&self) -> Result<Option<RiskPolicyState>> {
        let workspace_id = self.workspace_id()?;
        let data: Option<String> = self
            .connection
            .query_row(
                "SELECT projection FROM risk_state WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        match data {
            Some(data) => Ok(Some(RiskPolicyState::from_persisted_json(
                &data,
                &workspace_id,
            )?)),
            None => Ok(None),
        }
    }

    pub fn save_risk(&mut self, mut risk: RiskPolicyState) -> Result<DomainEvent> {
        if risk.workspace_id != self.workspace_id()? {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let previous: i64 = tx
            .query_row(
                "SELECT COALESCE((SELECT sequence FROM risk_state WHERE singleton=1),0)",
                [],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if previous < 0 || previous >= MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let sequence = previous + 1;
        risk.state_version = format!("risk:{}:{}", risk.workspace_id, sequence);
        risk.updated_at = timestamp()?;
        tx.execute(
            "INSERT INTO risk_state VALUES(1,?1,?2) ON CONFLICT(singleton) DO UPDATE SET sequence=excluded.sequence,projection=excluded.projection",
            params![sequence, serde_json::to_string(&risk).map_err(storage_error)?],
        )
        .map_err(storage_error)?;
        let event = DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: "risk.policy.changed".into(),
            schema_version: 1,
            occurred_at: risk.updated_at.clone(),
            aggregate_type: "risk".into(),
            aggregate_id: risk.workspace_id.clone(),
            sequence: sequence as u64,
            payload: DomainProjection::Risk(risk),
        };
        tx.execute(
            "INSERT INTO outbox VALUES('risk',?1,?2,?3,?4)",
            params![
                event.aggregate_id,
                sequence,
                event.event_id,
                serde_json::to_string(&event).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(event)
    }

    pub fn save_model(&mut self, mut model: ModelState, event_type: &str) -> Result<DomainEvent> {
        if model.workspace_id != self.workspace_id()? {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        if !matches!(
            event_type,
            "model.provider.changed" | "model.provider_attempt.changed"
        ) {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let previous: i64 = tx
            .query_row(
                "SELECT COALESCE((SELECT sequence FROM model_state WHERE singleton=1),0)",
                [],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if previous < 0 || previous >= MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let sequence = previous + 1;
        model.state_version = format!("model:{}:{}", model.workspace_id, sequence);
        model.updated_at = timestamp()?;
        tx.execute(
            "INSERT INTO model_state VALUES(1,?1,?2) ON CONFLICT(singleton) DO UPDATE SET sequence=excluded.sequence,projection=excluded.projection",
            params![sequence, serde_json::to_string(&model).map_err(storage_error)?],
        )
        .map_err(storage_error)?;
        let event = DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            schema_version: 1,
            occurred_at: model.updated_at.clone(),
            aggregate_type: "model".into(),
            aggregate_id: model.workspace_id.clone(),
            sequence: sequence as u64,
            payload: DomainProjection::Model(model),
        };
        tx.execute(
            "INSERT INTO outbox VALUES('model',?1,?2,?3,?4)",
            params![
                event.aggregate_id,
                sequence,
                event.event_id,
                serde_json::to_string(&event).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(event)
    }

    pub fn save_gateway(&mut self, mut gateway: GatewayState) -> Result<DomainEvent> {
        if gateway.workspace_id != self.workspace_id()? {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let previous: i64 = tx
            .query_row(
                "SELECT COALESCE((SELECT sequence FROM model_gateway WHERE singleton=1),0)",
                [],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if previous < 0 || previous >= MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let sequence = previous + 1;
        gateway.state_version = format!("model-gateway:{}:{}", gateway.workspace_id, sequence);
        gateway.updated_at = timestamp()?;
        tx.execute("INSERT INTO model_gateway VALUES(1,?1,?2) ON CONFLICT(singleton) DO UPDATE SET sequence=excluded.sequence,projection=excluded.projection", params![sequence,serde_json::to_string(&gateway).map_err(storage_error)?]).map_err(storage_error)?;
        let event = DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: "model.gateway.changed".into(),
            schema_version: 1,
            occurred_at: gateway.updated_at.clone(),
            aggregate_type: "model-gateway".into(),
            aggregate_id: gateway.workspace_id.clone(),
            sequence: sequence as u64,
            payload: DomainProjection::Gateway(gateway),
        };
        tx.execute(
            "INSERT INTO outbox VALUES('model-gateway',?1,?2,?3,?4)",
            params![
                event.aggregate_id,
                sequence,
                event.event_id,
                serde_json::to_string(&event).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(event)
    }

    pub fn save_account(&mut self, mut account: AccountConnection) -> Result<DomainEvent> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let previous: i64 = tx
            .query_row(
                "SELECT COALESCE((SELECT sequence FROM accounts WHERE connection_id=?1),0)",
                [&account.connection_id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if previous < 0 || previous >= MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let sequence = previous + 1;
        account.state_version = format!("{}:{}", account.connection_id, sequence);
        account.updated_at = timestamp()?;
        let remote = if account.connection_state == ConnectionState::Disconnected {
            None
        } else {
            account.data.as_ref().map(|d| d.remote_account_id.as_str())
        };
        tx.execute("INSERT INTO accounts VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(connection_id) DO UPDATE SET remote_identity=excluded.remote_identity,sequence=excluded.sequence,projection=excluded.projection",
            params![account.connection_id,account.provider_id,account.environment,remote,sequence,account.credential_ref(),serde_json::to_string(&account).map_err(storage_error)?]).map_err(storage_error)?;
        let event = DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: "account.health.changed".into(),
            schema_version: 1,
            occurred_at: account.updated_at.clone(),
            aggregate_type: "account".into(),
            aggregate_id: account.connection_id.clone(),
            sequence: sequence as u64,
            payload: DomainProjection::Account(Box::new(account)),
        };
        tx.execute(
            "INSERT INTO outbox VALUES ('account',?1,?2,?3,?4)",
            params![
                event.aggregate_id,
                sequence,
                event.event_id,
                serde_json::to_string(&event).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(event)
    }

    pub fn watchlists(&self) -> Result<Watchlists> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare(
                "SELECT watchlist_id,workspace_id,name,sequence,projection FROM watchlists WHERE workspace_id=?1 ORDER BY name COLLATE NOCASE,watchlist_id",
            )
            .map_err(storage_error)?;
        let rows = query
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(storage_error)?;
        let mut lists = Vec::new();
        let mut max_sequence = 0_i64;
        for row in rows {
            let (watchlist_id, row_workspace_id, row_name, sequence, projection) =
                row.map_err(storage_error)?;
            max_sequence = max_sequence.max(sequence);
            lists.push(decode_watchlist(
                &projection,
                &watchlist_id,
                &row_workspace_id,
                &row_name,
                sequence,
                &workspace_id,
            )?);
        }
        if max_sequence < 0 || max_sequence > MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(Watchlists {
            workspace_id: workspace_id.clone(),
            state_version: format!("watchlists:{}:{}", workspace_id, max_sequence),
            watchlists: lists,
        })
    }

    pub fn create_watchlist(&mut self, workspace_id: &str, name: &str) -> Result<Watchlist> {
        let name = validate_watchlist_name(name)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if watchlist_name_conflict(&tx, workspace_id, &name, None)? {
            return Err(TradeXError::new("WATCHLIST_NAME_CONFLICT"));
        }
        let count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM watchlists WHERE workspace_id=?1",
                [workspace_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !(0..128).contains(&count) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let watchlist_id = Uuid::new_v4().to_string();
        let sequence = 1_i64;
        let watchlist = Watchlist {
            watchlist_id: watchlist_id.clone(),
            workspace_id: workspace_id.to_owned(),
            name,
            state_version: format!("watchlist:{}:{}", watchlist_id, sequence),
            items: Vec::new(),
        };
        tx.execute(
            "INSERT INTO watchlists(watchlist_id,workspace_id,name,sequence,projection) VALUES(?1,?2,?3,?4,?5)",
            params![
                &watchlist.watchlist_id,
                &watchlist.workspace_id,
                &watchlist.name,
                sequence,
                serde_json::to_string(&watchlist).map_err(storage_error)?,
            ],
        )
        .map_err(|error| {
            if error.to_string().contains("UNIQUE") {
                TradeXError::new("WATCHLIST_NAME_CONFLICT")
            } else {
                storage_error(error)
            }
        })?;
        tx.commit().map_err(storage_error)?;
        Ok(watchlist)
    }

    pub fn rename_watchlist(
        &mut self,
        workspace_id: &str,
        watchlist_id: &str,
        name: &str,
        expected_state_version: &str,
    ) -> Result<Watchlist> {
        let name = validate_watchlist_name(name)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let mut watchlist = load_watchlist_tx(&tx, workspace_id, watchlist_id)?;
        if watchlist.state_version != expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if watchlist_name_conflict(&tx, workspace_id, &name, Some(watchlist_id))? {
            return Err(TradeXError::new("WATCHLIST_NAME_CONFLICT"));
        }
        let sequence = next_watchlist_sequence(&tx, watchlist_id)?;
        watchlist.name = name;
        watchlist.state_version = format!("watchlist:{}:{}", watchlist_id, sequence);
        update_watchlist_tx(&tx, &watchlist, sequence)?;
        tx.commit().map_err(storage_error)?;
        Ok(watchlist)
    }

    pub fn delete_watchlist(
        &mut self,
        workspace_id: &str,
        watchlist_id: &str,
        expected_state_version: &str,
    ) -> Result<Watchlists> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let watchlist = load_watchlist_tx(&tx, workspace_id, watchlist_id)?;
        if watchlist.state_version != expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        tx.execute(
            "DELETE FROM watchlists WHERE workspace_id=?1 AND watchlist_id=?2",
            params![workspace_id, watchlist_id],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        self.watchlists()
    }

    pub fn mutate_watchlist_members(
        &mut self,
        workspace_id: &str,
        watchlist_id: &str,
        instrument_id: &str,
        expected_state_version: &str,
        add: bool,
    ) -> Result<Watchlist> {
        if !market::validate_instrument_id(instrument_id)
            || !market::instruments()
                .iter()
                .any(|instrument| instrument.instrument_id == instrument_id)
        {
            return Err(TradeXError::new("MARKET_INSTRUMENT_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let mut watchlist = load_watchlist_tx(&tx, workspace_id, watchlist_id)?;
        if watchlist.state_version != expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let present = watchlist
            .items
            .iter()
            .any(|item| item.instrument_id == instrument_id);
        if present == add {
            tx.commit().map_err(storage_error)?;
            return Ok(watchlist);
        }
        if add {
            if watchlist.items.len() >= 256 {
                return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
            }
            watchlist.items.push(WatchlistItem {
                instrument_id: instrument_id.to_owned(),
            });
        } else {
            watchlist
                .items
                .retain(|item| item.instrument_id != instrument_id);
        }
        let sequence = next_watchlist_sequence(&tx, watchlist_id)?;
        watchlist.state_version = format!("watchlist:{}:{}", watchlist_id, sequence);
        update_watchlist_tx(&tx, &watchlist, sequence)?;
        tx.commit().map_err(storage_error)?;
        Ok(watchlist)
    }

    pub fn screeners(&self) -> Result<ScreenerLibrary> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare(
                "SELECT screener_id,workspace_id,name,sequence,projection FROM screeners WHERE workspace_id=?1 ORDER BY name COLLATE NOCASE,screener_id",
            )
            .map_err(storage_error)?;
        let rows = query
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(storage_error)?;
        let mut screeners = Vec::new();
        let mut max_sequence = 0_i64;
        for row in rows {
            let (screener_id, row_workspace_id, row_name, sequence, projection) =
                row.map_err(storage_error)?;
            max_sequence = max_sequence.max(sequence);
            screeners.push(decode_screener(
                &projection,
                &screener_id,
                &row_workspace_id,
                &row_name,
                sequence,
                &workspace_id,
            )?);
        }
        if max_sequence < 0 || max_sequence > MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(ScreenerLibrary {
            workspace_id: workspace_id.clone(),
            state_version: format!("screeners:{workspace_id}:{max_sequence}"),
            screeners,
        })
    }

    pub fn save_screener(&mut self, input: &ScreenerSave) -> Result<ScreenerLibrary> {
        validate_screener_name(&input.name)?;
        validate_screener_state(&input.state)?;
        crate::screener::validate_definition(&input.definition)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        validate_screener_state_version(&tx, &input.workspace_id, &input.expected_state_version)?;
        if screener_name_conflict(&tx, &input.workspace_id, &input.name, None)? {
            return Err(TradeXError::new("SCREENER_NAME_CONFLICT"));
        }
        let count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM screeners WHERE workspace_id=?1",
                [&input.workspace_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !(0..128).contains(&count) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let sequence = next_screener_sequence(&tx, &input.workspace_id)?;
        let now = timestamp()?;
        let screener_id = Uuid::new_v4().to_string();
        let screener = SavedScreener {
            screener_id: screener_id.clone(),
            workspace_id: input.workspace_id.clone(),
            name: input.name.trim().to_owned(),
            definition: input.definition.clone(),
            state: input.state,
            created_at: now.clone(),
            updated_at: now,
            state_version: format!("screener:{screener_id}:{sequence}"),
        };
        tx.execute(
            "INSERT INTO screeners(screener_id,workspace_id,name,sequence,projection) VALUES(?1,?2,?3,?4,?5)",
            params![
                &screener.screener_id,
                &screener.workspace_id,
                &screener.name,
                sequence,
                serde_json::to_string(&screener).map_err(storage_error)?,
            ],
        )
        .map_err(|error| {
            if error.to_string().contains("UNIQUE") {
                TradeXError::new("SCREENER_NAME_CONFLICT")
            } else {
                storage_error(error)
            }
        })?;
        tx.commit().map_err(storage_error)?;
        self.screeners()
    }

    pub fn update_screener(&mut self, input: &ScreenerUpdate) -> Result<ScreenerLibrary> {
        validate_screener_name(&input.name)?;
        validate_screener_state(&input.state)?;
        crate::screener::validate_definition(&input.definition)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        validate_screener_state_version(&tx, &input.workspace_id, &input.expected_state_version)?;
        let mut screener = load_screener_tx(&tx, &input.workspace_id, &input.screener_id)?;
        if screener_name_conflict(
            &tx,
            &input.workspace_id,
            &input.name,
            Some(&input.screener_id),
        )? {
            return Err(TradeXError::new("SCREENER_NAME_CONFLICT"));
        }
        let sequence = next_screener_sequence(&tx, &input.workspace_id)?;
        screener.name = input.name.trim().to_owned();
        screener.definition = input.definition.clone();
        screener.state = input.state;
        screener.updated_at = timestamp()?;
        screener.state_version = format!("screener:{}:{sequence}", screener.screener_id);
        tx.execute(
            "UPDATE screeners SET name=?1,sequence=?2,projection=?3 WHERE workspace_id=?4 AND screener_id=?5",
            params![
                &screener.name,
                sequence,
                serde_json::to_string(&screener).map_err(storage_error)?,
                &screener.workspace_id,
                &screener.screener_id,
            ],
        )
        .map_err(|error| {
            if error.to_string().contains("UNIQUE") {
                TradeXError::new("SCREENER_NAME_CONFLICT")
            } else {
                storage_error(error)
            }
        })?;
        tx.commit().map_err(storage_error)?;
        self.screeners()
    }

    pub fn artifacts(&self) -> Result<ArtifactLibrary> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare(
                "SELECT artifact_id,workspace_id,kind,title,sequence,projection FROM artifacts WHERE workspace_id=?1 ORDER BY sequence DESC,artifact_id",
            )
            .map_err(storage_error)?;
        let rows = query
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(storage_error)?;
        let mut artifacts = Vec::new();
        let mut max_sequence = 0_i64;
        for row in rows {
            let (artifact_id, row_workspace_id, kind, title, sequence, projection) =
                row.map_err(storage_error)?;
            max_sequence = max_sequence.max(sequence);
            let artifact = decode_artifact(
                &projection,
                &artifact_id,
                &row_workspace_id,
                &kind,
                &title,
                sequence,
                &workspace_id,
            )?;
            artifacts.push(ArtifactSummary {
                artifact_id: artifact.artifact_id,
                workspace_id: artifact.workspace_id,
                kind: artifact.kind,
                title: artifact.title,
                content_hash: artifact.content_hash,
                state_version: artifact.state_version,
                created_at: artifact.created_at,
                updated_at: artifact.updated_at,
                thread_id: artifact.provenance.thread_id,
                turn_id: artifact.provenance.turn_id,
                item_id: artifact.provenance.item_id,
            });
        }
        if !(0..=MAX_SEQUENCE as i64).contains(&max_sequence) || artifacts.len() > 256 {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(ArtifactLibrary {
            workspace_id: workspace_id.clone(),
            state_version: format!("artifacts:{workspace_id}:{max_sequence}"),
            artifacts,
        })
    }

    pub fn artifact(&self, artifact_id: &str) -> Result<Artifact> {
        let workspace_id = self.workspace_id()?;
        let (row_workspace_id, kind, title, sequence, projection): (String, String, String, i64, String) = self
            .connection
            .query_row(
                "SELECT workspace_id,kind,title,sequence,projection FROM artifacts WHERE workspace_id=?1 AND artifact_id=?2",
                params![workspace_id, artifact_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("ARTIFACT_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        decode_artifact(
            &projection,
            artifact_id,
            &row_workspace_id,
            &kind,
            &title,
            sequence,
            &workspace_id,
        )
    }

    pub fn save_artifact(&mut self, mut artifact: Artifact) -> Result<Artifact> {
        let workspace_id = self.workspace_id()?;
        if artifact.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_artifact_title(&artifact.title)?;
        validate_artifact_content(&artifact.content)?;
        validate_artifact_provenance(&artifact.provenance, &workspace_id)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM artifacts WHERE workspace_id=?1",
                [&workspace_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !(0..256).contains(&count) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let sequence = next_artifact_sequence(&tx, &workspace_id)?;
        artifact.artifact_id = Uuid::new_v4().to_string();
        artifact.version = 1;
        artifact.created_at = timestamp()?;
        artifact.updated_at = artifact.created_at.clone();
        artifact.state_version = format!("artifact:{}:{sequence}", artifact.artifact_id);
        artifact.content_hash = artifact_hash(&artifact)?;
        tx.execute(
            "INSERT INTO artifacts(artifact_id,workspace_id,kind,title,sequence,projection) VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                &artifact.artifact_id,
                &artifact.workspace_id,
                artifact_kind_name(artifact.kind),
                &artifact.title,
                sequence,
                serde_json::to_string(&artifact).map_err(storage_error)?,
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(artifact)
    }

    pub fn export_artifact(&self, input: &ArtifactExport) -> Result<ArtifactExportResult> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let artifact = self.artifact(&input.artifact_id)?;
        let destination = artifact_export_destination(&self.path, input, &artifact.artifact_id)?;
        if fs::symlink_metadata(&destination).is_ok() {
            return Err(TradeXError::new("ARTIFACT_EXPORT_EXISTS"));
        }
        let manifest = serde_json::json!({
            "schemaVersion": 1,
            "artifactId": &artifact.artifact_id,
            "artifactVersion": artifact.version,
            "contentHash": &artifact.content_hash,
            "exportedAt": timestamp()?,
            "provenance": &artifact.provenance,
        });
        let manifest_hash = hash_bytes(&serde_json::to_vec(&manifest).map_err(storage_error)?);
        let document = serde_json::json!({
            "manifest": manifest,
            "artifact": artifact,
        });
        let encoded = serde_json::to_vec_pretty(&document).map_err(storage_error)?;
        if encoded.is_empty() || encoded.len() > 10_000_000 {
            return Err(TradeXError::new("ARTIFACT_EXPORT_FAILED"));
        }
        let parent = destination
            .parent()
            .ok_or_else(|| TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"))?;
        let file_name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"))?;
        let temporary = parent.join(format!(".{}.{}.tmp", file_name, Uuid::new_v4()));
        let write_result = (|| -> Result<()> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_FAILED"))?;
            file.write_all(&encoded)
                .and_then(|_| file.sync_all())
                .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_FAILED"))?;
            fs::hard_link(&temporary, &destination).map_err(|error| {
                if error.kind() == ErrorKind::AlreadyExists {
                    TradeXError::new("ARTIFACT_EXPORT_EXISTS")
                } else {
                    TradeXError::new("ARTIFACT_EXPORT_FAILED")
                }
            })?;
            let _ = fs::remove_file(&temporary);
            Ok(())
        })();
        if let Err(error) = write_result {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        Ok(ArtifactExportResult {
            artifact_id: artifact.artifact_id,
            path: destination.to_string_lossy().into_owned(),
            content_hash: artifact.content_hash,
            manifest_hash,
            bytes: encoded.len() as u64,
        })
    }

    pub fn mark_accounts_stale(&mut self) -> Result<()> {
        for mut account in self.accounts()? {
            if account.connection_state == ConnectionState::Disconnected
                || (account.connection_state == ConnectionState::Failed
                    && account.health.credential == "MISSING")
            {
                continue;
            }
            if account.connection_state == ConnectionState::Connecting
                || account.health.credential == "DELETE_PENDING"
            {
                account.connection_state = ConnectionState::Disconnected;
                account.health.connection = "DISCONNECTED".into();
                account.health.credential = "DELETE_PENDING".into();
                account.health.reason = "An interrupted connection needs Keychain cleanup. Retry Disconnect before reconnecting.".into();
                self.save_account(account)?;
                continue;
            }
            account.health.connection = "STALE".into();
            account.health.authentication = "UNVERIFIED".into();
            account.health.credential = "UNCHECKED".into();
            account.health.reason =
                "Re-test the connection to verify credentials and refresh saved observations."
                    .into();
            self.save_account(account)?;
        }
        Ok(())
    }
}

fn validate_watchlist_name(name: &str) -> Result<String> {
    Ok(validate_bounded_name(name)?.to_owned())
}

fn validate_bounded_name(name: &str) -> Result<&str> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 80 || trimmed.chars().any(char::is_control) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    Ok(trimmed)
}

fn watchlist_name_conflict(
    tx: &rusqlite::Transaction<'_>,
    workspace_id: &str,
    name: &str,
    excluded_id: Option<&str>,
) -> Result<bool> {
    let folded = name.to_lowercase();
    // ponytail: bounded 128-list scan; add a normalized key/index if this limit changes.
    let mut query = tx
        .prepare("SELECT watchlist_id,name FROM watchlists WHERE workspace_id=?1")
        .map_err(storage_error)?;
    let rows = query
        .query_map([workspace_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(storage_error)?;
    for row in rows {
        let (watchlist_id, existing_name) = row.map_err(storage_error)?;
        if excluded_id == Some(watchlist_id.as_str()) {
            continue;
        }
        if existing_name.to_lowercase() == folded {
            return Ok(true);
        }
    }
    Ok(false)
}

fn load_watchlist_tx(
    tx: &rusqlite::Transaction<'_>,
    workspace_id: &str,
    watchlist_id: &str,
) -> Result<Watchlist> {
    let (row_workspace_id, row_name, sequence, projection): (String, String, i64, String) = tx
        .query_row(
            "SELECT workspace_id,name,sequence,projection FROM watchlists WHERE workspace_id=?1 AND watchlist_id=?2",
            params![workspace_id, watchlist_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|error| {
            if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                TradeXError::new("WATCHLIST_NOT_FOUND")
            } else {
                storage_error(error)
            }
        })?;
    decode_watchlist(
        &projection,
        watchlist_id,
        &row_workspace_id,
        &row_name,
        sequence,
        workspace_id,
    )
}

fn next_watchlist_sequence(tx: &rusqlite::Transaction<'_>, watchlist_id: &str) -> Result<i64> {
    let previous: i64 = tx
        .query_row(
            "SELECT sequence FROM watchlists WHERE watchlist_id=?1",
            [watchlist_id],
            |row| row.get(0),
        )
        .map_err(storage_error)?;
    if previous < 1 || previous >= MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    Ok(previous + 1)
}

fn update_watchlist_tx(
    tx: &rusqlite::Transaction<'_>,
    watchlist: &Watchlist,
    sequence: i64,
) -> Result<()> {
    tx.execute(
        "UPDATE watchlists SET name=?1,sequence=?2,projection=?3 WHERE workspace_id=?4 AND watchlist_id=?5",
        params![
            &watchlist.name,
            sequence,
            serde_json::to_string(watchlist).map_err(storage_error)?,
            &watchlist.workspace_id,
            &watchlist.watchlist_id,
        ],
    )
    .map_err(|error| {
        if error.to_string().contains("UNIQUE") {
            TradeXError::new("WATCHLIST_NAME_CONFLICT")
        } else {
            storage_error(error)
        }
    })?;
    Ok(())
}

fn decode_watchlist(
    projection: &str,
    row_id: &str,
    row_workspace_id: &str,
    row_name: &str,
    sequence: i64,
    workspace_id: &str,
) -> Result<Watchlist> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let watchlist: Watchlist = serde_json::from_str(projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if watchlist.watchlist_id != row_id
        || watchlist.workspace_id != row_workspace_id
        || watchlist.name != row_name
        || row_workspace_id != workspace_id
        || watchlist.state_version != format!("watchlist:{}:{}", row_id, sequence)
        || watchlist.name.trim() != watchlist.name
        || watchlist.name.is_empty()
        || watchlist.name.chars().count() > 80
        || watchlist.name.chars().any(char::is_control)
        || watchlist.items.len() > 256
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let mut seen = std::collections::HashSet::new();
    for item in &watchlist.items {
        if !market::validate_instrument_id(&item.instrument_id)
            || !market::instruments()
                .iter()
                .any(|instrument| instrument.instrument_id == item.instrument_id)
            || !seen.insert(&item.instrument_id)
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
    }
    Ok(watchlist)
}

fn validate_screener_name(name: &str) -> Result<()> {
    validate_bounded_name(name)?;
    Ok(())
}

fn validate_screener_state(state: &ScreenerResultState) -> Result<()> {
    if matches!(state, ScreenerResultState::Running) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    Ok(())
}

fn screener_name_conflict(
    tx: &rusqlite::Transaction<'_>,
    workspace_id: &str,
    name: &str,
    excluded_id: Option<&str>,
) -> Result<bool> {
    let folded = name.to_lowercase();
    let mut query = tx
        .prepare("SELECT screener_id,name FROM screeners WHERE workspace_id=?1")
        .map_err(storage_error)?;
    let rows = query
        .query_map([workspace_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(storage_error)?;
    for row in rows {
        let (screener_id, existing_name) = row.map_err(storage_error)?;
        if excluded_id == Some(screener_id.as_str()) {
            continue;
        }
        if existing_name.to_lowercase() == folded {
            return Ok(true);
        }
    }
    Ok(false)
}

fn max_screener_sequence(tx: &rusqlite::Transaction<'_>, workspace_id: &str) -> Result<i64> {
    let max_sequence: Option<i64> = tx
        .query_row(
            "SELECT MAX(sequence) FROM screeners WHERE workspace_id=?1",
            [workspace_id],
            |row| row.get(0),
        )
        .map_err(storage_error)?;
    Ok(max_sequence.unwrap_or(0))
}

fn current_screener_state_version(
    tx: &rusqlite::Transaction<'_>,
    workspace_id: &str,
) -> Result<String> {
    let max_sequence = max_screener_sequence(tx, workspace_id)?;
    if !(0..=MAX_SEQUENCE as i64).contains(&max_sequence) {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(format!("screeners:{workspace_id}:{max_sequence}"))
}

fn validate_screener_state_version(
    tx: &rusqlite::Transaction<'_>,
    workspace_id: &str,
    expected: &str,
) -> Result<()> {
    if expected.is_empty() || expected.len() > 256 || expected.chars().any(char::is_control) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    if current_screener_state_version(tx, workspace_id)? != expected {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    Ok(())
}

fn next_screener_sequence(tx: &rusqlite::Transaction<'_>, workspace_id: &str) -> Result<i64> {
    let max_sequence = max_screener_sequence(tx, workspace_id)?;
    if !(0..MAX_SEQUENCE as i64).contains(&max_sequence) {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    Ok(max_sequence + 1)
}

fn load_screener_tx(
    tx: &rusqlite::Transaction<'_>,
    workspace_id: &str,
    screener_id: &str,
) -> Result<SavedScreener> {
    let (row_workspace_id, row_name, sequence, projection): (String, String, i64, String) = tx
        .query_row(
            "SELECT workspace_id,name,sequence,projection FROM screeners WHERE workspace_id=?1 AND screener_id=?2",
            params![workspace_id, screener_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|error| {
            if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                TradeXError::new("SCREENER_NOT_FOUND")
            } else {
                storage_error(error)
            }
        })?;
    decode_screener(
        &projection,
        screener_id,
        &row_workspace_id,
        &row_name,
        sequence,
        workspace_id,
    )
}

fn decode_screener(
    projection: &str,
    row_id: &str,
    row_workspace_id: &str,
    row_name: &str,
    sequence: i64,
    workspace_id: &str,
) -> Result<SavedScreener> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let screener: SavedScreener = serde_json::from_str(projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if screener.screener_id != row_id
        || screener.workspace_id != row_workspace_id
        || screener.name != row_name
        || row_workspace_id != workspace_id
        || screener.state_version != format!("screener:{row_id}:{sequence}")
        || screener.name.trim() != screener.name
        || screener.name.is_empty()
        || screener.name.chars().count() > 80
        || screener.name.chars().any(char::is_control)
        || screener.created_at.is_empty()
        || screener.updated_at.is_empty()
        || matches!(screener.state, ScreenerResultState::Running)
        || crate::screener::validate_definition(&screener.definition).is_err()
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(screener)
}

fn artifact_kind_name(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Research => "RESEARCH",
        ArtifactKind::Decision => "DECISION",
    }
}

fn validate_artifact_title(title: &str) -> Result<()> {
    let trimmed = title.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 120 || trimmed.chars().any(char::is_control)
    {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    if contains_sensitive_marker(trimmed) {
        return Err(TradeXError::new("ARTIFACT_REDACTION_FAILED"));
    }
    Ok(())
}

fn validate_artifact_content(content: &ArtifactContent) -> Result<()> {
    if content.text.is_empty()
        || content.text.chars().count() > 32_768
        || content.text.chars().any(|character| character == '\0')
        || contains_sensitive_marker(&content.text)
    {
        return Err(TradeXError::new("ARTIFACT_REDACTION_FAILED"));
    }
    if content
        .research_result
        .as_ref()
        .is_some_and(|result| result.marker.is_empty() || result.marker.len() > 96)
    {
        return Err(TradeXError::new("ARTIFACT_REDACTION_FAILED"));
    }
    if content
        .research_result
        .as_ref()
        .and_then(|result| serde_json::to_string(result).ok())
        .is_some_and(|encoded| contains_sensitive_marker(&encoded))
    {
        return Err(TradeXError::new("ARTIFACT_REDACTION_FAILED"));
    }
    Ok(())
}

fn validate_artifact_provenance(
    provenance: &crate::protocol::ArtifactProvenance,
    workspace_id: &str,
) -> Result<()> {
    if provenance.workspace_id != workspace_id
        || provenance.thread_id.is_empty()
        || provenance.turn_id.is_empty()
        || provenance.item_id.is_empty()
        || provenance.thread_id.len() > 128
        || provenance.turn_id.len() > 128
        || provenance.item_id.len() > 128
        || provenance.thread_id.chars().any(char::is_control)
        || provenance.turn_id.chars().any(char::is_control)
        || provenance.item_id.chars().any(char::is_control)
        || provenance.provider_attempts.len() > 16
        || provenance.sources.len() > 16
        || provenance.market_snapshot_hashes.len() > 16
        || provenance.dataset_hashes.len() > 16
        || provenance.related_order_ids.len() > 16
        || provenance
            .market_snapshot_hashes
            .iter()
            .chain(provenance.dataset_hashes.iter())
            .chain(provenance.related_order_ids.iter())
            .any(|value| {
                value.is_empty() || value.len() > 128 || value.chars().any(char::is_control)
            })
    {
        return Err(TradeXError::new("ARTIFACT_SOURCE_INVALID"));
    }
    if provenance.turn_id != provenance.turn_snapshot.turn_id {
        return Err(TradeXError::new("ARTIFACT_SOURCE_INVALID"));
    }
    if serde_json::to_string(provenance)
        .ok()
        .is_some_and(|encoded| contains_sensitive_marker(&encoded))
    {
        return Err(TradeXError::new("ARTIFACT_REDACTION_FAILED"));
    }
    Ok(())
}

fn contains_sensitive_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if serde_json::from_str::<serde_json::Value>(value)
        .ok()
        .is_some_and(|value| json_contains_sensitive_key(&value))
    {
        return true;
    }
    [
        "authorization:",
        "authorization=",
        "bearer ",
        "api_key=",
        "api-key=",
        "api_key:",
        "api-key:",
        "api key=",
        "api key:",
        "apikey=",
        "apikey:",
        "\"apikey\":",
        "\"api_key\":",
        "secret=",
        "secret:",
        "\"secret\":",
        "password=",
        "password:",
        "\"password\":",
        "token=",
        "token:",
        "\"token\":",
        "access_token=",
        "access_token:",
        "\"access_token\":",
        "refresh_token=",
        "refresh_token:",
        "\"refresh_token\":",
        "private_key=",
        "private_key:",
        "\"private_key\":",
        "\"authorization\":",
        "sk-",
        "ghp_",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn json_contains_sensitive_key(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(object) => object.iter().any(|(key, value)| {
            matches!(
                key.to_ascii_lowercase().as_str(),
                "authorization"
                    | "api_key"
                    | "apikey"
                    | "api-key"
                    | "secret"
                    | "password"
                    | "token"
                    | "access_token"
                    | "refresh_token"
                    | "private_key"
                    | "client_secret"
                    | "credential"
                    | "credentials"
            ) || json_contains_sensitive_key(value)
        }),
        serde_json::Value::Array(values) => values.iter().any(json_contains_sensitive_key),
        _ => false,
    }
}

fn next_artifact_sequence(tx: &rusqlite::Transaction<'_>, workspace_id: &str) -> Result<i64> {
    let max_sequence: Option<i64> = tx
        .query_row(
            "SELECT MAX(sequence) FROM artifacts WHERE workspace_id=?1",
            [workspace_id],
            |row| row.get(0),
        )
        .map_err(storage_error)?;
    let max_sequence = max_sequence.unwrap_or(0);
    if !(0..MAX_SEQUENCE as i64).contains(&max_sequence) {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    Ok(max_sequence + 1)
}

fn decode_artifact(
    projection: &str,
    row_id: &str,
    row_workspace_id: &str,
    row_kind: &str,
    row_title: &str,
    sequence: i64,
    workspace_id: &str,
) -> Result<Artifact> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let artifact: Artifact = serde_json::from_str(projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if artifact.artifact_id != row_id
        || artifact.workspace_id != row_workspace_id
        || row_workspace_id != workspace_id
        || artifact.kind != parse_artifact_kind(row_kind)?
        || artifact.title != row_title
        || artifact.state_version != format!("artifact:{row_id}:{sequence}")
        || artifact.version != 1
        || artifact.content_hash != artifact_hash(&artifact)?
        || validate_artifact_title(&artifact.title).is_err()
        || validate_artifact_content(&artifact.content).is_err()
        || validate_artifact_provenance(&artifact.provenance, workspace_id).is_err()
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(artifact)
}

fn parse_artifact_kind(value: &str) -> Result<ArtifactKind> {
    match value {
        "RESEARCH" => Ok(ArtifactKind::Research),
        "DECISION" => Ok(ArtifactKind::Decision),
        _ => Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED")),
    }
}

fn artifact_hash(artifact: &Artifact) -> Result<String> {
    let input = serde_json::json!({
        "kind": artifact.kind,
        "title": &artifact.title,
        "content": &artifact.content,
        "provenance": &artifact.provenance,
    });
    Ok(hash_bytes(
        &serde_json::to_vec(&input).map_err(storage_error)?,
    ))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn artifact_export_file_name(file_name: Option<&str>, artifact_id: &str) -> Result<String> {
    let value = file_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{artifact_id}.json"));
    if value.len() > 128
        || value == "."
        || value == ".."
        || value.chars().any(|character| {
            character.is_control()
                || matches!(character, '/' | '\\')
                || !(character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_'))
        })
    {
        return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
    }
    if value.ends_with('.') {
        return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
    }
    if value.ends_with(".json") {
        Ok(value)
    } else {
        let with_extension = format!("{value}.json");
        if with_extension.len() > 128 {
            return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
        }
        Ok(with_extension)
    }
}

fn artifact_export_destination(
    workspace_path: &Path,
    input: &ArtifactExport,
    artifact_id: &str,
) -> Result<PathBuf> {
    if input.file_name.is_some() && input.destination_path.is_some() {
        return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
    }
    if let Some(raw_path) = input.destination_path.as_deref() {
        let raw_path = raw_path.trim();
        let path = PathBuf::from(raw_path);
        if raw_path.is_empty()
            || raw_path.len() > 4096
            || raw_path.chars().any(char::is_control)
            || !path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
        {
            return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
        }
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"))?;
        if !parent.is_dir()
            || fs::symlink_metadata(parent)
                .map(|metadata| metadata.file_type().is_symlink())
                .unwrap_or(false)
        {
            return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
        }
        let requested_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"))?;
        let file_name = artifact_export_file_name(Some(requested_name), artifact_id)?;
        return Ok(parent.join(file_name));
    }

    let file_name = artifact_export_file_name(input.file_name.as_deref(), artifact_id)?;
    let exports = workspace_path.join("exports");
    if fs::symlink_metadata(&exports)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
    }
    fs::create_dir_all(&exports).map_err(|_| TradeXError::new("ARTIFACT_EXPORT_FAILED"))?;
    if !exports.is_dir() {
        return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
    }
    Ok(exports.join(file_name))
}

fn read_workspace(connection: &Connection, path: &Path) -> Result<Workspace> {
    connection.query_row("SELECT workspace_id,name,created_at,last_opened_at,base_currency FROM workspace WHERE singleton=1", [], |row| Ok(Workspace {
        workspace_id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, last_opened_at: row.get(3)?,
        base_currency: row.get(4)?, path: path.to_string_lossy().into_owned(), storage_schema_version: SCHEMA_VERSION,
    })).map_err(storage_error)
}

pub(crate) fn timestamp() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(storage_error)
}

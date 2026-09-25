use std::{
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write},
    path::{Component, Path, PathBuf},
    time::Duration,
};

#[cfg(unix)]
use std::{
    ffi::CString,
    os::unix::{
        ffi::OsStrExt,
        io::{FromRawFd, RawFd},
    },
};

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::gateway::GatewayState;
use crate::market;
use crate::model::ModelState;
use crate::protocol::{
    AlpacaPaperCancelState, AlpacaPaperOrder, AlpacaPaperOrderAttempt,
    AlpacaPaperOrderAttemptState, AlpacaPaperOrderBook, AlpacaPaperOrderBookStatus,
    AlpacaPaperOrderCancel, AlpacaPaperOrderOrigin, AlpacaPaperOrderSubmit, Artifact,
    ArtifactContent, ArtifactExport, ArtifactExportResult, ArtifactKind, ArtifactLibrary,
    ArtifactSummary, AssetClass, BacktestFailure, BacktestLibrary, BacktestRun, BacktestRunRequest,
    BacktestRunState, BacktestRunSummary, BinanceTestnetOrderAttempt,
    BinanceTestnetOrderAttemptState, BinanceTestnetOrderBook, BinanceTestnetOrderBookStatus,
    BinanceTestnetOrderCancel, BinanceTestnetOrderCancelState, BinanceTestnetOrderOrigin,
    BinanceTestnetOrderSubmit, BitgetDemoOrderAttempt, BitgetDemoOrderAttemptState,
    BitgetDemoOrderSubmit, DomainEvent, DomainProjection, EventSink, ExecutionContext,
    LocalPaperEvent, LocalPaperEventKind, LocalPaperFill, LocalPaperOrder, LocalPaperState,
    MAX_SEQUENCE, OpenWorkspace, OrderDraft, OrderDraftFields, OrderDraftLibrary, OrderDraftSave,
    OrderDraftSummary, OrderProposal, OrderProposalGenerate, OrderProposalHistoryEntry,
    OrderProposalHistoryEvent, OrderProposalLibrary, OrderProposalRefresh,
    OrderProposalRefreshResult, OrderProposalRefreshStatus, OrderProposalStatus,
    OrderProposalSummary, OrderType, PaperOrderCancel, PaperOrderResult, PaperOrderSubmit,
    PaperQuoteRefresh, PaperScenarioSet, ProposalReferenceStatus, Result, SavedScreener,
    ScreenerLibrary, ScreenerResultState, ScreenerSave, ScreenerUpdate, Snapshot, StrategyFailure,
    StrategyLibrary, StrategyRun, StrategyRunRequest, StrategyRunState, StrategyRunSummary,
    StrategySave, StrategyVersion, SubscriptionAck, Thread, ThreadList, ThreadSummary, TimeInForce,
    TradeXError, Trading212DemoCancelState, Trading212DemoNormalizedOrderStatus,
    Trading212DemoOrderAttempt, Trading212DemoOrderAttemptState, Trading212DemoOrderBook,
    Trading212DemoOrderBookStatus, Trading212DemoOrderCancel, Trading212DemoOrderOrigin,
    Trading212DemoOrderSubmit, Watchlist, WatchlistItem, Watchlists, Workspace,
};
use crate::providers::{AccountConnection, AccountMutation, ConnectionState};
use crate::risk::{RiskDecision, RiskDecisionHistory, RiskPolicyState};

const APPLICATION_ID: u32 = 0x54525831;
pub(crate) const SCHEMA_VERSION: u32 = 23;
const MAX_ORDER_DECIMAL_FRACTION_DIGITS: usize = 18;

pub struct Store {
    connection: Connection,
    _lock: File,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct OrderProposalReferences {
    pub policy_version: Option<u64>,
    pub policy_state_version: Option<String>,
    pub policy_status: ProposalReferenceStatus,
    pub policy_reference_reason: String,
    pub market_snapshot_id: Option<String>,
    pub market_status: crate::protocol::MarketDataStatus,
    pub market_reference_reason: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredOrderProposal {
    proposal_id: String,
    workspace_id: String,
    draft_id: String,
    draft_version: u64,
    proposal_hash: String,
    fields: OrderDraftFields,
    estimated_notional: Option<String>,
    estimated_notional_currency: Option<String>,
    estimated_notional_reason: Option<String>,
    policy_version: Option<u64>,
    policy_state_version: Option<String>,
    policy_status: ProposalReferenceStatus,
    policy_reference_reason: String,
    market_snapshot_id: Option<String>,
    market_status: crate::protocol::MarketDataStatus,
    market_reference_reason: String,
    created_at: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderProposalHashInput<'a> {
    workspace_id: &'a str,
    draft_id: &'a str,
    draft_version: u64,
    fields: OrderProposalHashFields<'a>,
    estimated_notional: Option<&'a str>,
    estimated_notional_currency: Option<&'a str>,
    estimated_notional_reason: Option<&'a str>,
    policy_version: Option<u64>,
    policy_state_version: Option<&'a str>,
    policy_status: ProposalReferenceStatus,
    policy_reference_reason: &'a str,
    market_snapshot_id: Option<&'a str>,
    market_status: crate::protocol::MarketDataStatus,
    market_reference_reason: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderProposalHashFields<'a> {
    account_id: Option<&'a str>,
    venue: &'a str,
    environment: &'a ExecutionContext,
    instrument_id: &'a str,
    side: &'a crate::protocol::OrderSide,
    order_type: &'a OrderType,
    quantity: &'a crate::protocol::OrderQuantity,
    limit_price: Option<&'a str>,
    maximum_spend: Option<&'a str>,
    time_in_force: &'a TimeInForce,
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
            if version < 10 {
                tx.execute_batch("CREATE TABLE order_drafts (
                    draft_id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    draft_version INTEGER NOT NULL CHECK(draft_version > 0),
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL
                );
                CREATE INDEX order_drafts_workspace_order ON order_drafts(workspace_id,sequence DESC,draft_id);
                PRAGMA user_version=10;").map_err(storage_error)?;
            }
            if version < 11 {
                tx.execute_batch("CREATE TABLE order_proposals (
                    proposal_id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    draft_id TEXT NOT NULL,
                    draft_version INTEGER NOT NULL CHECK(draft_version > 0),
                    proposal_hash TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL,
                    UNIQUE(workspace_id,proposal_hash)
                );
                CREATE INDEX order_proposals_workspace_order ON order_proposals(workspace_id,sequence DESC,proposal_id);
                CREATE TABLE order_proposal_events (
                    proposal_id TEXT NOT NULL,
                    workspace_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    event TEXT NOT NULL,
                    reason TEXT,
                    occurred_at TEXT NOT NULL,
                    PRIMARY KEY(proposal_id,sequence)
                );
                CREATE INDEX order_proposal_events_workspace_order ON order_proposal_events(workspace_id,proposal_id,sequence);
                PRAGMA user_version=11;").map_err(storage_error)?;
            }
            if version < 12 {
                tx.execute_batch("CREATE TABLE IF NOT EXISTS strategy_versions (
                    strategy_version_id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    strategy_id TEXT NOT NULL,
                    revision INTEGER NOT NULL CHECK(revision > 0),
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL,
                    UNIQUE(workspace_id,strategy_id,revision)
                );
                CREATE INDEX IF NOT EXISTS strategy_versions_workspace_order ON strategy_versions(workspace_id,sequence DESC,strategy_id,revision DESC);
                CREATE TABLE IF NOT EXISTS strategy_runs (
                    run_id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS strategy_runs_workspace_order ON strategy_runs(workspace_id,sequence DESC,run_id);
                PRAGMA user_version=12;").map_err(storage_error)?;
            }
            if version < 13 {
                tx.execute_batch("CREATE TABLE IF NOT EXISTS backtest_runs (
                    run_id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS backtest_runs_workspace_order ON backtest_runs(workspace_id,sequence DESC,run_id);
                PRAGMA user_version=13;").map_err(storage_error)?;
            }
            if version < 14 {
                tx.execute_batch("CREATE TABLE IF NOT EXISTS paper_state (
                    workspace_id TEXT PRIMARY KEY,
                    account_id TEXT NOT NULL UNIQUE,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS paper_state_workspace_order ON paper_state(workspace_id,sequence DESC);
                PRAGMA user_version=14;").map_err(storage_error)?;
            }
            if version < 15 {
                tx.execute_batch("CREATE TABLE IF NOT EXISTS paper_orders (
                    workspace_id TEXT NOT NULL,
                    order_id TEXT NOT NULL,
                    proposal_id TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    projection TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,order_id),
                    UNIQUE(workspace_id,proposal_id),
                    UNIQUE(workspace_id,idempotency_key)
                );
                CREATE TABLE IF NOT EXISTS paper_fills (
                    workspace_id TEXT NOT NULL,
                    fill_id TEXT NOT NULL,
                    order_id TEXT NOT NULL,
                    projection TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,fill_id),
                    UNIQUE(workspace_id,order_id,fill_id)
                );
                CREATE TABLE IF NOT EXISTS paper_events (
                    workspace_id TEXT NOT NULL,
                    account_id TEXT NOT NULL,
                    event_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    order_id TEXT NOT NULL,
                    fill_id TEXT,
                    kind TEXT NOT NULL,
                    occurred_at TEXT NOT NULL,
                    state_version TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,sequence),
                    UNIQUE(workspace_id,event_id)
                );
                CREATE INDEX IF NOT EXISTS paper_events_workspace_order ON paper_events(workspace_id,sequence);
                PRAGMA user_version=15;").map_err(storage_error)?;
            }
            if version < 16 {
                tx.execute_batch("CREATE TABLE alpaca_paper_order_attempts (
                    workspace_id TEXT NOT NULL,
                    attempt_id TEXT NOT NULL,
                    connection_id TEXT NOT NULL,
                    proposal_id TEXT NOT NULL,
                    client_order_id TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    state TEXT NOT NULL CHECK(state IN ('SUBMITTING','ACKNOWLEDGED','UNKNOWN_RECONCILING','REJECTED')),
                    projection TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,attempt_id),
                    UNIQUE(workspace_id,proposal_id),
                    UNIQUE(workspace_id,idempotency_key),
                    UNIQUE(connection_id,client_order_id)
                );
                CREATE INDEX alpaca_paper_attempts_connection_state ON alpaca_paper_order_attempts(connection_id,state);
                PRAGMA user_version=16;").map_err(storage_error)?;
            }
            if version < 17 {
                tx.execute_batch("CREATE TABLE alpaca_paper_order_books (
                    workspace_id TEXT NOT NULL,
                    connection_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,connection_id)
                );
                CREATE INDEX alpaca_paper_order_books_connection ON alpaca_paper_order_books(connection_id);
                PRAGMA user_version=17;").map_err(storage_error)?;
            }
            if version < 18 {
                tx.execute_batch("CREATE TABLE trading212_demo_order_attempts (
                    workspace_id TEXT NOT NULL,
                    attempt_id TEXT NOT NULL,
                    connection_id TEXT NOT NULL,
                    proposal_id TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    state TEXT NOT NULL CHECK(state IN ('SUBMITTING','ACKNOWLEDGED','UNKNOWN_RECONCILING','REJECTED')),
                    projection TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,attempt_id),
                    UNIQUE(workspace_id,proposal_id)
                );
                CREATE INDEX trading212_demo_attempts_connection_state ON trading212_demo_order_attempts(connection_id,state);
                PRAGMA user_version=18;").map_err(storage_error)?;
            }
            if version < 19 {
                tx.execute_batch("CREATE TABLE trading212_demo_order_books (
                    workspace_id TEXT NOT NULL,
                    connection_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,connection_id)
                );
                CREATE INDEX trading212_demo_order_books_connection ON trading212_demo_order_books(connection_id);
                PRAGMA user_version=19;").map_err(storage_error)?;
            }
            if version < 20 {
                tx.execute_batch("CREATE TABLE binance_testnet_order_attempts (
                    workspace_id TEXT NOT NULL,
                    attempt_id TEXT NOT NULL,
                    connection_id TEXT NOT NULL,
                    proposal_id TEXT NOT NULL,
                    client_order_id TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    state TEXT NOT NULL CHECK(state IN ('SUBMITTING','ACKNOWLEDGED','UNKNOWN_RECONCILING','REJECTED')),
                    projection TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,attempt_id),
                    UNIQUE(workspace_id,proposal_id),
                    UNIQUE(workspace_id,idempotency_key),
                    UNIQUE(connection_id,client_order_id)
                );
                CREATE INDEX binance_testnet_attempts_connection_state ON binance_testnet_order_attempts(connection_id,state);
                PRAGMA user_version=20;").map_err(storage_error)?;
            }
            if version < 21 {
                tx.execute_batch("CREATE TABLE binance_testnet_order_books (
                    workspace_id TEXT NOT NULL,
                    connection_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,connection_id)
                );
                CREATE INDEX binance_testnet_order_books_connection ON binance_testnet_order_books(connection_id);
                PRAGMA user_version=21;").map_err(storage_error)?;
            }
            if version < 22 {
                tx.execute_batch("CREATE TABLE bitget_demo_order_attempts (
                    workspace_id TEXT NOT NULL,
                    attempt_id TEXT NOT NULL,
                    connection_id TEXT NOT NULL,
                    proposal_id TEXT NOT NULL,
                    client_oid TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    state TEXT NOT NULL CHECK(state IN ('SUBMITTING','ACKNOWLEDGED','UNKNOWN_RECONCILING','REJECTED')),
                    projection TEXT NOT NULL,
                    PRIMARY KEY(workspace_id,attempt_id),
                    UNIQUE(workspace_id,proposal_id),
                    UNIQUE(workspace_id,idempotency_key),
                    UNIQUE(connection_id,client_oid)
                );
                CREATE INDEX bitget_demo_attempts_connection_state ON bitget_demo_order_attempts(connection_id,state);
                PRAGMA user_version=22;").map_err(storage_error)?;
            }
            if version < 23 {
                tx.execute_batch("CREATE TABLE risk_decisions (
                    decision_id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    proposal_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL CHECK(sequence > 0),
                    projection TEXT NOT NULL,
                    UNIQUE(workspace_id,proposal_id,sequence)
                );
                CREATE INDEX risk_decisions_workspace_proposal ON risk_decisions(workspace_id,proposal_id,sequence DESC);
                PRAGMA user_version=23;").map_err(storage_error)?;
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
        backfill_local_paper_tables(&mut connection)?;
        let mut store = Self {
            connection,
            _lock: lock,
            path,
        };
        store.recover_interrupted_trading212_demo_attempts()?;
        store.recover_interrupted_trading212_demo_cancels()?;
        store.recover_interrupted_alpaca_paper_attempts()?;
        store.recover_interrupted_binance_testnet_attempts()?;
        store.recover_interrupted_bitget_demo_attempts()?;
        store.recover_interrupted_binance_testnet_cancels()?;
        store.recover_interrupted_alpaca_paper_cancels()?;
        Ok(store)
    }

    fn recover_interrupted_trading212_demo_attempts(&mut self) -> Result<()> {
        let mut statement = self.connection.prepare(
            "SELECT workspace_id,attempt_id,sequence,projection FROM trading212_demo_order_attempts WHERE state='SUBMITTING'",
        ).map_err(storage_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        drop(statement);
        for (workspace_id, attempt_id, sequence, projection) in rows {
            if sequence < 1 {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let mut attempt: Trading212DemoOrderAttempt = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if attempt.workspace_id != workspace_id
                || attempt.attempt_id != attempt_id
                || attempt.state != Trading212DemoOrderAttemptState::Submitting
                || attempt.state_version
                    != trading212_demo_attempt_version(&attempt_id, sequence as u64)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage_error)?;
            let next = sequence
                .checked_add(1)
                .filter(|value| *value <= MAX_SEQUENCE as i64)
                .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
            attempt.state = Trading212DemoOrderAttemptState::UnknownReconciling;
            attempt.error_code = Some("ORDER_STATUS_UNKNOWN".into());
            attempt.reason = "The workspace reopened during submission. Trading 212 has no client-order identity; do not resubmit.".into();
            attempt.state_version = trading212_demo_attempt_version(&attempt_id, next as u64);
            attempt.updated_at = timestamp()?;
            let encoded = serde_json::to_string(&attempt).map_err(storage_error)?;
            tx.execute(
                "UPDATE trading212_demo_order_attempts SET sequence=?1,state='UNKNOWN_RECONCILING',projection=?2 WHERE workspace_id=?3 AND attempt_id=?4 AND sequence=?5 AND state='SUBMITTING'",
                params![next, encoded, workspace_id, attempt_id, sequence],
            ).map_err(storage_error)?;
            write_trading212_demo_attempt_event(&tx, &attempt, next)?;
            tx.commit().map_err(storage_error)?;
        }
        Ok(())
    }

    fn recover_interrupted_alpaca_paper_cancels(&mut self) -> Result<()> {
        let mut query = self.connection.prepare(
            "SELECT workspace_id,connection_id,sequence,projection FROM alpaca_paper_order_books",
        ).map_err(storage_error)?;
        let rows = query
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?;
        let books = rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        drop(query);
        for (workspace_id, connection_id, sequence, projection) in books {
            let mut book: AlpacaPaperOrderBook = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if sequence < 1
                || sequence >= MAX_SEQUENCE as i64
                || book.workspace_id != workspace_id
                || book.connection_id != connection_id
                || book.state_version != alpaca_order_book_version(&connection_id, sequence as u64)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let mut changed = false;
            for order in &mut book.orders {
                if order.cancel_state == AlpacaPaperCancelState::Submitting {
                    order.cancel_state = AlpacaPaperCancelState::Pending;
                    order.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                    changed = true;
                }
            }
            if changed {
                let next_sequence = sequence + 1;
                book.status = AlpacaPaperOrderBookStatus::Degraded;
                book.reason = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                book.observed_at = timestamp()?;
                book.state_version =
                    alpaca_order_book_version(&connection_id, next_sequence as u64);
                let tx = self
                    .connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)
                    .map_err(storage_error)?;
                let updated = tx.execute(
                    "UPDATE alpaca_paper_order_books SET sequence=?1,projection=?2 WHERE workspace_id=?3 AND connection_id=?4 AND sequence=?5",
                    params![next_sequence, serde_json::to_string(&book).map_err(storage_error)?, workspace_id, connection_id, sequence],
                ).map_err(storage_error)?;
                if updated == 1 {
                    write_alpaca_order_book_event(&tx, &book, next_sequence)?;
                }
                tx.commit().map_err(storage_error)?;
            }
        }
        Ok(())
    }

    fn recover_interrupted_trading212_demo_cancels(&mut self) -> Result<()> {
        let mut query = self
            .connection
            .prepare("SELECT workspace_id,connection_id,sequence,projection FROM trading212_demo_order_books")
            .map_err(storage_error)?;
        let rows = query
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        drop(query);
        for (workspace_id, connection_id, sequence, projection) in rows {
            let mut book: Trading212DemoOrderBook = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if sequence < 1
                || sequence >= MAX_SEQUENCE as i64
                || book.workspace_id != workspace_id
                || book.connection_id != connection_id
                || book.state_version
                    != trading212_demo_order_book_version(&connection_id, sequence as u64)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let mut changed = false;
            for order in &mut book.orders {
                if order.cancel_state == Trading212DemoCancelState::Submitting {
                    order.cancel_state = Trading212DemoCancelState::Pending;
                    order.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                    changed = true;
                }
            }
            if changed {
                let next_sequence = sequence + 1;
                book.status = Trading212DemoOrderBookStatus::Degraded;
                book.reason = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                book.observed_at = timestamp()?;
                book.state_version =
                    trading212_demo_order_book_version(&connection_id, next_sequence as u64);
                validate_trading212_demo_order_book(&book)?;
                let tx = self
                    .connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)
                    .map_err(storage_error)?;
                let updated = tx.execute(
                    "UPDATE trading212_demo_order_books SET sequence=?1,projection=?2 WHERE workspace_id=?3 AND connection_id=?4 AND sequence=?5",
                    params![next_sequence, serde_json::to_string(&book).map_err(storage_error)?, workspace_id, connection_id, sequence],
                ).map_err(storage_error)?;
                if updated == 1 {
                    write_trading212_demo_order_book_event(&tx, &book, next_sequence)?;
                }
                tx.commit().map_err(storage_error)?;
            }
        }
        Ok(())
    }

    fn recover_interrupted_alpaca_paper_attempts(&mut self) -> Result<()> {
        let mut query = self.connection.prepare(
            "SELECT workspace_id,attempt_id,sequence,projection FROM alpaca_paper_order_attempts WHERE state='SUBMITTING'",
        ).map_err(storage_error)?;
        let rows = query
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?;
        let attempts = rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        drop(query);
        for (workspace_id, attempt_id, sequence, projection) in attempts {
            if sequence < 1 || sequence >= MAX_SEQUENCE as i64 {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let mut attempt: AlpacaPaperOrderAttempt = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if attempt.attempt_id != attempt_id
                || attempt.workspace_id != workspace_id
                || attempt.state != AlpacaPaperOrderAttemptState::Submitting
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            attempt.state = AlpacaPaperOrderAttemptState::UnknownReconciling;
            attempt.error_code = Some("ORDER_STATUS_UNKNOWN".into());
            attempt.reason = "The workspace reopened during submission. Query Alpaca Paper by client order ID before taking any further action.".into();
            attempt.updated_at = timestamp()?;
            attempt.state_version = format!("alpaca-paper-attempt:{attempt_id}:{}", sequence + 1);
            let encoded = serde_json::to_string(&attempt).map_err(storage_error)?;
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage_error)?;
            let updated = tx.execute(
                "UPDATE alpaca_paper_order_attempts SET sequence=?1,state='UNKNOWN_RECONCILING',projection=?2 WHERE workspace_id=?3 AND attempt_id=?4 AND sequence=?5 AND state='SUBMITTING'",
                params![sequence + 1, encoded, workspace_id, attempt_id, sequence],
            ).map_err(storage_error)?;
            if updated == 1 {
                write_alpaca_paper_attempt_event(&tx, &attempt, sequence + 1)?;
            }
            tx.commit().map_err(storage_error)?;
        }
        Ok(())
    }

    fn recover_interrupted_binance_testnet_attempts(&mut self) -> Result<()> {
        let mut query = self.connection.prepare(
            "SELECT workspace_id,attempt_id,sequence,projection FROM binance_testnet_order_attempts WHERE state='SUBMITTING'",
        ).map_err(storage_error)?;
        let rows = query
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?;
        let attempts = rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        drop(query);
        for (workspace_id, attempt_id, sequence, projection) in attempts {
            if sequence < 1 || sequence >= MAX_SEQUENCE as i64 {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let mut attempt: BinanceTestnetOrderAttempt = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if attempt.attempt_id != attempt_id
                || attempt.workspace_id != workspace_id
                || attempt.state != BinanceTestnetOrderAttemptState::Submitting
                || attempt.environment != "TESTNET"
                || attempt.state_version
                    != format!("binance-testnet-attempt:{attempt_id}:{sequence}")
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            attempt.state = BinanceTestnetOrderAttemptState::UnknownReconciling;
            attempt.error_code = Some("ORDER_STATUS_UNKNOWN".into());
            attempt.reason = "The workspace reopened during submission. Query Binance Spot Testnet by client order ID; do not resubmit.".into();
            attempt.updated_at = timestamp()?;
            let next = sequence + 1;
            attempt.state_version = format!("binance-testnet-attempt:{attempt_id}:{next}");
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage_error)?;
            let updated = tx.execute(
                "UPDATE binance_testnet_order_attempts SET sequence=?1,state='UNKNOWN_RECONCILING',projection=?2 WHERE workspace_id=?3 AND attempt_id=?4 AND sequence=?5 AND state='SUBMITTING'",
                params![next, serde_json::to_string(&attempt).map_err(storage_error)?, workspace_id, attempt_id, sequence],
            ).map_err(storage_error)?;
            if updated == 1 {
                write_binance_testnet_attempt_event(&tx, &attempt, next)?;
            }
            tx.commit().map_err(storage_error)?;
        }
        Ok(())
    }

    fn recover_interrupted_bitget_demo_attempts(&mut self) -> Result<()> {
        let mut query = self.connection.prepare(
            "SELECT workspace_id,attempt_id,sequence,projection FROM bitget_demo_order_attempts WHERE state='SUBMITTING'",
        ).map_err(storage_error)?;
        let rows = query
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?;
        let attempts = rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        drop(query);
        for (workspace_id, attempt_id, sequence, projection) in attempts {
            if sequence < 1 || sequence >= MAX_SEQUENCE as i64 {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let mut attempt: BitgetDemoOrderAttempt = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if attempt.attempt_id != attempt_id
                || attempt.workspace_id != workspace_id
                || attempt.environment != "DEMO"
                || attempt.state != BitgetDemoOrderAttemptState::Submitting
                || attempt.state_version
                    != format!("bitget-demo-order-attempt:{attempt_id}:{sequence}")
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            attempt.state = BitgetDemoOrderAttemptState::UnknownReconciling;
            attempt.error_code = Some("ORDER_STATUS_UNKNOWN".into());
            attempt.reason = "The workspace reopened during submission. Query Bitget Demo by clientOid; do not resubmit.".into();
            attempt.updated_at = timestamp()?;
            let next = sequence + 1;
            attempt.state_version = format!("bitget-demo-order-attempt:{attempt_id}:{next}");
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage_error)?;
            let updated = tx.execute(
                "UPDATE bitget_demo_order_attempts SET sequence=?1,state='UNKNOWN_RECONCILING',projection=?2 WHERE workspace_id=?3 AND attempt_id=?4 AND sequence=?5 AND state='SUBMITTING'",
                params![next, serde_json::to_string(&attempt).map_err(storage_error)?, workspace_id, attempt_id, sequence],
            ).map_err(storage_error)?;
            if updated == 1 {
                write_bitget_demo_attempt_event(&tx, &attempt, next)?;
            }
            tx.commit().map_err(storage_error)?;
        }
        Ok(())
    }

    fn recover_interrupted_binance_testnet_cancels(&mut self) -> Result<()> {
        let mut query = self.connection.prepare(
            "SELECT workspace_id,connection_id,sequence,projection FROM binance_testnet_order_books",
        ).map_err(storage_error)?;
        let rows = query
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        drop(query);
        for (workspace_id, connection_id, sequence, projection) in rows {
            if sequence < 1 || sequence >= MAX_SEQUENCE as i64 {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let mut book: BinanceTestnetOrderBook = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if book.workspace_id != workspace_id
                || book.connection_id != connection_id
                || book.state_version
                    != binance_testnet_order_book_version(&connection_id, sequence as u64)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let mut changed = false;
            for order in &mut book.orders {
                if order.cancel_state == BinanceTestnetOrderCancelState::Submitting {
                    order.cancel_state = BinanceTestnetOrderCancelState::Pending;
                    order.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                    changed = true;
                }
            }
            if !changed {
                continue;
            }
            let next_sequence = sequence + 1;
            book.status = BinanceTestnetOrderBookStatus::Degraded;
            book.reason = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
            book.observed_at = timestamp()?;
            book.state_version =
                binance_testnet_order_book_version(&connection_id, next_sequence as u64);
            validate_binance_testnet_order_book(&book)?;
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage_error)?;
            let updated = tx.execute(
                "UPDATE binance_testnet_order_books SET sequence=?1,projection=?2 WHERE workspace_id=?3 AND connection_id=?4 AND sequence=?5",
                params![next_sequence, serde_json::to_string(&book).map_err(storage_error)?, workspace_id, connection_id, sequence],
            ).map_err(storage_error)?;
            if updated == 1 {
                write_binance_testnet_order_book_event(&tx, &book, next_sequence)?;
            }
            tx.commit().map_err(storage_error)?;
        }
        Ok(())
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
                    "risk-decision" => event.event_type != "risk.decision.evaluated",
                    "thread" => !matches!(
                        event.event_type.as_str(),
                        "thread.created" | "thread.updated"
                    ),
                    "trading212-demo-order-attempt" => {
                        event.event_type != "trading212.demo.order.attempt.changed"
                    }
                    "alpaca-paper-order-attempt" => {
                        event.event_type != "alpaca.paper.order.attempt.changed"
                    }
                    "binance-testnet-order-attempt" => {
                        event.event_type != "binance.testnet.order.attempt.changed"
                    }
                    "alpaca-paper-order-book" => {
                        event.event_type != "alpaca.paper.order.book.changed"
                    }
                    "trading212-demo-order-book" => {
                        event.event_type != "trading212.demo.order.book.changed"
                    }
                    "binance-testnet-order-book" => {
                        event.event_type != "binance.testnet.order.book.changed"
                    }
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

    pub fn base_currency(&self) -> Result<String> {
        self.connection
            .query_row("SELECT base_currency FROM workspace", [], |r| r.get(0))
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
        let mut accounts = rows
            .map(|row| {
                let (connection_id, encoded) = row.map_err(storage_error)?;
                let account: AccountConnection =
                    serde_json::from_str(&encoded).map_err(storage_error)?;
                if account.connection_id != connection_id {
                    return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
                }
                account.validate_persisted(&workspace_id)?;
                Ok(account)
            })
            .collect::<Result<Vec<_>>>()?;
        accounts.sort_by_key(AccountConnection::is_local_paper);
        Ok(accounts)
    }

    pub fn ensure_local_paper(
        &mut self,
        base_currency: &str,
    ) -> Result<(AccountConnection, Option<DomainEvent>)> {
        let workspace_id = self.workspace_id()?;
        let existing_id: Option<String> = self
            .connection
            .query_row(
                "SELECT connection_id FROM accounts WHERE provider_id='local-paper' AND environment='LOCAL' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let (account, event) = if let Some(connection_id) = existing_id {
            (self.account(&connection_id)?, None)
        } else {
            let account = crate::paper::account(&workspace_id, base_currency)?;
            let event = self.save_account(account)?;
            let account = match &event.payload {
                DomainProjection::Account(account) => (**account).clone(),
                _ => return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED")),
            };
            (account, Some(event))
        };
        let state_exists: Option<String> = self
            .connection
            .query_row(
                "SELECT projection FROM paper_state WHERE workspace_id=?1",
                [&workspace_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        if let Some(encoded) = state_exists {
            let state = load_local_paper_tables(
                &self.connection,
                &workspace_id,
                serde_json::from_str(&encoded).map_err(storage_error)?,
            )?;
            validate_local_paper_state(&state, &workspace_id, &account.connection_id)?;
        } else {
            let state = crate::paper::initial_state(
                &workspace_id,
                &account.connection_id,
                &account.label,
                base_currency,
                timestamp()?,
            )?;
            let encoded = serde_json::to_string(&state).map_err(storage_error)?;
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage_error)?;
            tx.execute(
                "INSERT INTO paper_state(workspace_id,account_id,sequence,projection) VALUES(?1,?2,1,?3)",
                params![workspace_id, account.connection_id, encoded],
            )
            .map_err(storage_error)?;
            tx.commit().map_err(storage_error)?;
        }
        Ok((account, event))
    }

    pub fn local_paper_state(&self) -> Result<LocalPaperState> {
        let workspace_id = self.workspace_id()?;
        let encoded: String = self
            .connection
            .query_row(
                "SELECT projection FROM paper_state WHERE workspace_id=?1",
                [&workspace_id],
                |row| row.get(0),
            )
            .map_err(|_| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
        let state = load_local_paper_tables(
            &self.connection,
            &workspace_id,
            serde_json::from_str(&encoded).map_err(storage_error)?,
        )?;
        let account = self.account(&state.account_id)?;
        validate_local_paper_state(&state, &workspace_id, &account.connection_id)?;
        Ok(state)
    }

    pub fn submit_local_paper_order(
        &mut self,
        input: &PaperOrderSubmit,
    ) -> Result<PaperOrderResult> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(&input.proposal_id)?;
        if !valid_order_text(&input.expected_proposal_state_version, 256)
            || !valid_order_text(&input.idempotency_key, 128)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (state_account_id, state_sequence, encoded_state): (String, i64, String) = tx
            .query_row(
                "SELECT account_id,sequence,projection FROM paper_state WHERE workspace_id=?1",
                [&workspace_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
        if state_sequence < 1 || state_sequence > MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        let state = load_local_paper_tables(
            &tx,
            &workspace_id,
            serde_json::from_str(&encoded_state).map_err(storage_error)?,
        )?;
        validate_local_paper_state(&state, &workspace_id, &state_account_id)?;
        let account_projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&state_account_id],
                |row| row.get(0),
            )
            .map_err(|_| TradeXError::new("PAPER_ACCOUNT_INVALID"))?;
        let account: AccountConnection =
            serde_json::from_str(&account_projection).map_err(storage_error)?;
        if !account.is_local_paper() || account.workspace_id != workspace_id {
            return Err(TradeXError::new("PAPER_ACCOUNT_INVALID"));
        }
        account.validate_persisted(&workspace_id)?;

        let (
            row_workspace_id,
            draft_id,
            draft_version,
            proposal_hash,
            proposal_sequence,
            projection,
        ): (String, String, i64, String, i64, String) = tx
            .query_row(
                "SELECT workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE workspace_id=?1 AND proposal_id=?2",
                params![&workspace_id, &input.proposal_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("ORDER_PROPOSAL_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let stored = decode_stored_order_proposal(
            &projection,
            &input.proposal_id,
            &row_workspace_id,
            &draft_id,
            draft_version,
            &proposal_hash,
            proposal_sequence,
            &workspace_id,
        )?;
        let proposal = materialize_order_proposal(&tx, stored)?;
        let (proposal_status, _, last_event_sequence) =
            proposal_event_state(&tx, &input.proposal_id, &workspace_id)?;
        let current_proposal_state_version =
            format!("order-proposal:{}:{last_event_sequence}", input.proposal_id);

        if let Some(existing) = state
            .orders
            .iter()
            .find(|order| order.proposal_id == input.proposal_id)
            .cloned()
        {
            if existing.idempotency_key.as_deref() != Some(input.idempotency_key.as_str()) {
                return Err(TradeXError::new("PAPER_PROPOSAL_CONSUMED"));
            }
            let result =
                crate::paper::result_for_order(&state, &existing, &current_proposal_state_version)?;
            tx.commit().map_err(storage_error)?;
            return Ok(result);
        }
        if proposal_status != OrderProposalStatus::NeedsApproval {
            return Err(TradeXError::new("PAPER_PROPOSAL_CONSUMED"));
        }
        if input.expected_proposal_state_version != current_proposal_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }

        let previous_event_cursor = state.event_cursor;
        let mut next_state = state;
        let now = timestamp()?;
        let mut result = crate::paper::submit(
            &mut next_state,
            &proposal,
            &input.idempotency_key,
            &current_proposal_state_version,
            now,
        )?;
        let consumed_sequence = last_event_sequence
            .checked_add(1)
            .filter(|sequence| *sequence <= 32)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        let reason = format!(
            "Local Paper order {} consumed this proposal.",
            result.order.order_id
        );
        tx.execute(
            "INSERT INTO order_proposal_events(proposal_id,workspace_id,sequence,event,reason,occurred_at) VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                &input.proposal_id,
                &workspace_id,
                consumed_sequence,
                "CONSUMED",
                reason,
                &next_state.updated_at,
            ],
        )
        .map_err(storage_error)?;
        result.proposal_state_version =
            format!("order-proposal:{}:{consumed_sequence}", input.proposal_id);
        tx.execute(
            "INSERT INTO paper_orders(workspace_id,order_id,proposal_id,idempotency_key,projection) VALUES(?1,?2,?3,?4,?5)",
            params![
                &workspace_id,
                &result.order.order_id,
                &result.order.proposal_id,
                result.order.idempotency_key.as_deref().unwrap_or(""),
                serde_json::to_string(&result.order).map_err(storage_error)?,
            ],
        )
        .map_err(storage_error)?;
        if let Some(fill) = result.fill.as_ref() {
            tx.execute(
                "INSERT INTO paper_fills(workspace_id,fill_id,order_id,projection) VALUES(?1,?2,?3,?4)",
                params![
                    &workspace_id,
                    &fill.fill_id,
                    &fill.order_id,
                    serde_json::to_string(fill).map_err(storage_error)?,
                ],
            )
            .map_err(storage_error)?;
        }
        for event in next_state
            .events
            .iter()
            .filter(|event| event.sequence > previous_event_cursor)
        {
            tx.execute(
                "INSERT INTO paper_events(workspace_id,account_id,event_id,sequence,order_id,fill_id,kind,occurred_at,state_version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    &workspace_id,
                    &next_state.account_id,
                    &event.event_id,
                    event.sequence as i64,
                    &event.order_id,
                    &event.fill_id,
                    paper_event_kind_name(event.kind),
                    &event.occurred_at,
                    &event.state_version,
                ],
            )
            .map_err(storage_error)?;
        }
        let mut persisted_state = next_state.clone();
        persisted_state.orders.clear();
        persisted_state.fills.clear();
        persisted_state.events.clear();
        persisted_state.open_orders.clear();
        let encoded = serde_json::to_string(&persisted_state).map_err(storage_error)?;
        let changed = tx
            .execute(
                "UPDATE paper_state SET account_id=?1,sequence=?2,projection=?3 WHERE workspace_id=?4 AND account_id=?1",
                params![
                    &next_state.account_id,
                    next_state.event_cursor.max(1) as i64,
                    encoded,
                    &workspace_id,
                ],
            )
            .map_err(storage_error)?;
        if changed != 1 {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        tx.commit().map_err(storage_error)?;
        result.paper_state = Box::new(next_state);
        Ok(result)
    }

    pub fn cancel_local_paper_order(
        &mut self,
        input: &PaperOrderCancel,
    ) -> Result<PaperOrderResult> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        if !valid_order_text(&input.order_id, 128)
            || !valid_order_text(&input.expected_state_version, 256)
            || !valid_order_text(&input.idempotency_key, 128)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (state_account_id, state_sequence, encoded_state): (String, i64, String) = tx
            .query_row(
                "SELECT account_id,sequence,projection FROM paper_state WHERE workspace_id=?1",
                [&workspace_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
        if state_sequence < 1 || state_sequence > MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        let state = load_local_paper_tables(
            &tx,
            &workspace_id,
            serde_json::from_str(&encoded_state).map_err(storage_error)?,
        )?;
        validate_local_paper_state(&state, &workspace_id, &state_account_id)?;
        let existing = state
            .orders
            .iter()
            .find(|order| order.order_id == input.order_id)
            .cloned()
            .ok_or_else(|| TradeXError::new("PAPER_ORDER_NOT_FOUND"))?;
        let (_, _, proposal_sequence) =
            proposal_event_state(&tx, &existing.proposal_id, &workspace_id)?;
        let proposal_state_version = format!(
            "order-proposal:{}:{proposal_sequence}",
            existing.proposal_id
        );
        if existing.cancel_idempotency_key.as_deref() == Some(input.idempotency_key.as_str()) {
            let result =
                crate::paper::result_for_order(&state, &existing, &proposal_state_version)?;
            tx.commit().map_err(storage_error)?;
            return Ok(result);
        }
        if input.expected_state_version != state.state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let previous_event_cursor = state.event_cursor;
        let mut next_state = state;
        let result = crate::paper::cancel(
            &mut next_state,
            &input.order_id,
            &input.idempotency_key,
            &proposal_state_version,
            timestamp()?,
        )?;
        tx.execute(
            "UPDATE paper_orders SET projection=?1 WHERE workspace_id=?2 AND order_id=?3",
            params![
                serde_json::to_string(&result.order).map_err(storage_error)?,
                &workspace_id,
                &input.order_id,
            ],
        )
        .map_err(storage_error)
        .and_then(|changed| {
            if changed == 1 {
                Ok(())
            } else {
                Err(storage_error(rusqlite::Error::QueryReturnedNoRows))
            }
        })?;
        persist_local_paper_events_and_state(
            &tx,
            &workspace_id,
            previous_event_cursor,
            &next_state,
        )?;
        tx.commit().map_err(storage_error)?;
        Ok(result)
    }

    pub fn set_local_paper_scenario(
        &mut self,
        input: &PaperScenarioSet,
    ) -> Result<LocalPaperState> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        if !valid_order_text(&input.expected_state_version, 256) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (state_account_id, state_sequence, encoded_state): (String, i64, String) = tx
            .query_row(
                "SELECT account_id,sequence,projection FROM paper_state WHERE workspace_id=?1",
                [&workspace_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
        if state_sequence < 1 || state_sequence > MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        let state = load_local_paper_tables(
            &tx,
            &workspace_id,
            serde_json::from_str(&encoded_state).map_err(storage_error)?,
        )?;
        validate_local_paper_state(&state, &workspace_id, &state_account_id)?;
        if input.expected_state_version != state.state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let previous_event_cursor = state.event_cursor;
        let mut next_state = state;
        crate::paper::set_scenario(&mut next_state, input.profile.clone(), timestamp()?)?;
        persist_local_paper_events_and_state(
            &tx,
            &workspace_id,
            previous_event_cursor,
            &next_state,
        )?;
        tx.commit().map_err(storage_error)?;
        Ok(next_state)
    }

    pub fn refresh_local_paper_quote(
        &mut self,
        input: &PaperQuoteRefresh,
    ) -> Result<LocalPaperState> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        if !valid_order_text(&input.expected_state_version, 256) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (state_account_id, state_sequence, encoded_state): (String, i64, String) = tx
            .query_row(
                "SELECT account_id,sequence,projection FROM paper_state WHERE workspace_id=?1",
                [&workspace_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
        if state_sequence < 1 || state_sequence > MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        let state = load_local_paper_tables(
            &tx,
            &workspace_id,
            serde_json::from_str(&encoded_state).map_err(storage_error)?,
        )?;
        validate_local_paper_state(&state, &workspace_id, &state_account_id)?;
        if input.expected_state_version != state.state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let previous_event_cursor = state.event_cursor;
        let mut next_state = state;
        crate::paper::refresh_quote(&mut next_state, timestamp()?)?;
        persist_local_paper_events_and_state(
            &tx,
            &workspace_id,
            previous_event_cursor,
            &next_state,
        )?;
        tx.commit().map_err(storage_error)?;
        Ok(next_state)
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

    pub fn delete_trading212_demo_account(&mut self, input: &AccountMutation) -> Result<()> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id
            || !valid_order_text(&input.connection_id, 128)
            || !valid_order_text(&input.expected_state_version, 256)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }

        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let mut proposals = tx
            .prepare(
                "SELECT proposal_id,workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE workspace_id=?1 ORDER BY sequence DESC,proposal_id",
            )
            .map_err(storage_error)?;
        let rows = proposals
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(storage_error)?;
        for row in rows {
            let (id, row_workspace, draft_id, draft_version, hash, sequence, projection) =
                row.map_err(storage_error)?;
            let stored = decode_stored_order_proposal(
                &projection,
                &id,
                &row_workspace,
                &draft_id,
                draft_version,
                &hash,
                sequence,
                &workspace_id,
            )?;
            if stored.fields.account_id.as_deref() == Some(input.connection_id.as_str())
                && stored.fields.environment == ExecutionContext::Trading212Demo
                && proposal_event_state(&tx, &id, &workspace_id)?.0
                    == OrderProposalStatus::NeedsApproval
            {
                return Err(TradeXError::new("ACCOUNT_DELETE_BLOCKED"));
            }
        }
        drop(proposals);
        let (credential_ref, sequence, projection): (String, i64, String) = tx
            .query_row(
                "SELECT credential_ref,sequence,projection FROM accounts WHERE connection_id=?1",
                [&input.connection_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection =
            serde_json::from_str(&projection).map_err(storage_error)?;
        if sequence < 1
            || sequence > MAX_SEQUENCE as i64
            || account.connection_id != input.connection_id
            || account.state_version != format!("{}:{sequence}", account.connection_id)
            || credential_ref != account.credential_ref()
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        account.validate_persisted(&workspace_id)?;
        if account.state_version != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if account.provider_id != "trading212"
            || account.environment != "DEMO"
            || !matches!(
                account.connection_state,
                ConnectionState::Failed | ConnectionState::Disconnected
            )
            || account.health.credential != "MISSING"
            || account
                .data
                .as_ref()
                .is_some_and(|data| !data.open_orders.is_empty())
        {
            return Err(TradeXError::new("ACCOUNT_DELETE_BLOCKED"));
        }

        let book = load_trading212_demo_order_book(&tx, &workspace_id, &input.connection_id)?;
        if book
            .as_ref()
            .is_some_and(|book| book.orders.iter().any(|order| order.pending))
        {
            return Err(TradeXError::new("ACCOUNT_DELETE_BLOCKED"));
        }
        let attempts = {
            let mut query = tx
                .prepare(
                    "SELECT attempt_id,proposal_id,state FROM trading212_demo_order_attempts WHERE workspace_id=?1 AND connection_id=?2",
                )
                .map_err(storage_error)?;
            query
                .query_map(params![workspace_id, input.connection_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .map_err(storage_error)?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(storage_error)?
        };
        for (attempt_id, proposal_id, state) in attempts {
            let attempt = load_trading212_demo_attempt(&tx, &workspace_id, &proposal_id)?
                .ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if attempt.attempt_id != attempt_id
                || attempt.connection_id != input.connection_id
                || state != trading212_demo_attempt_state_name(attempt.state)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            match attempt.state {
                Trading212DemoOrderAttemptState::Submitting
                | Trading212DemoOrderAttemptState::UnknownReconciling => {
                    return Err(TradeXError::new("ACCOUNT_DELETE_BLOCKED"));
                }
                Trading212DemoOrderAttemptState::Acknowledged => {
                    let terminal_order_observed = book.as_ref().is_some_and(|book| {
                        attempt
                            .provider_order_id
                            .as_deref()
                            .is_some_and(|order_id| {
                                book.orders.iter().any(|order| {
                                    order.provider_order_id == order_id
                                        && order.origin == Trading212DemoOrderOrigin::TradeX
                                        && order.attempt_id.as_deref() == Some(&attempt.attempt_id)
                                        && !order.pending
                                        && matches!(
                                            order.normalized_status,
                                            Trading212DemoNormalizedOrderStatus::Cancelled
                                                | Trading212DemoNormalizedOrderStatus::Filled
                                                | Trading212DemoNormalizedOrderStatus::Rejected
                                                | Trading212DemoNormalizedOrderStatus::Replaced
                                                | Trading212DemoNormalizedOrderStatus::Expired
                                        )
                                })
                            })
                    });
                    if !terminal_order_observed {
                        return Err(TradeXError::new("ACCOUNT_DELETE_BLOCKED"));
                    }
                }
                Trading212DemoOrderAttemptState::Rejected => {}
            }
        }

        let deleted = tx
            .execute(
                "DELETE FROM accounts WHERE connection_id=?1 AND sequence=?2",
                params![input.connection_id, sequence],
            )
            .map_err(storage_error)?;
        if deleted != 1 {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        tx.execute(
            "DELETE FROM outbox WHERE aggregate_type='account' AND aggregate_id=?1",
            [&input.connection_id],
        )
        .map_err(storage_error)?;
        tx.execute(
            "DELETE FROM trading212_demo_order_books WHERE workspace_id=?1 AND connection_id=?2",
            params![workspace_id, input.connection_id],
        )
        .map_err(storage_error)?;
        tx.execute(
            "DELETE FROM outbox WHERE aggregate_type='trading212-demo-order-book' AND aggregate_id=?1",
            [&input.connection_id],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)
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
        if kind == "risk-decision" {
            let workspace_id = self.workspace_id()?;
            let history = self.risk_decision_history(&workspace_id, id)?;
            let sequence = history.decisions.len() as u64;
            let decision = history
                .decisions
                .into_iter()
                .last()
                .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::RiskDecision(Box::new(decision)),
                last_sequence: sequence,
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
        if kind == "trading212-demo-order-attempt" {
            let workspace_id = self.workspace_id()?;
            let (sequence, proposal_id, projection): (i64, String, String) = self.connection
                .query_row(
                    "SELECT sequence,proposal_id,projection FROM trading212_demo_order_attempts WHERE workspace_id=?1 AND attempt_id=?2",
                    params![workspace_id, id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else { storage_error(error) })?;
            let attempt: Trading212DemoOrderAttempt = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if sequence < 1
                || load_trading212_demo_attempt(&self.connection, &workspace_id, &proposal_id)?
                    .as_ref()
                    != Some(&attempt)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::Trading212DemoOrderAttempt(Box::new(attempt)),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind == "alpaca-paper-order-attempt" {
            let workspace_id = self.workspace_id()?;
            let (sequence, proposal_id, projection): (i64, String, String) = self.connection
                .query_row(
                    "SELECT sequence,proposal_id,projection FROM alpaca_paper_order_attempts WHERE workspace_id=?1 AND attempt_id=?2",
                    params![workspace_id, id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else { storage_error(error) })?;
            let attempt: AlpacaPaperOrderAttempt = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if sequence < 1
                || load_alpaca_paper_attempt(&self.connection, &workspace_id, &proposal_id)?
                    .as_ref()
                    != Some(&attempt)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::AlpacaPaperOrderAttempt(Box::new(attempt)),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind == "binance-testnet-order-attempt" {
            let workspace_id = self.workspace_id()?;
            let (sequence, proposal_id, projection): (i64, String, String) = self.connection
                .query_row(
                    "SELECT sequence,proposal_id,projection FROM binance_testnet_order_attempts WHERE workspace_id=?1 AND attempt_id=?2",
                    params![workspace_id, id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else { storage_error(error) })?;
            let attempt: BinanceTestnetOrderAttempt = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if sequence < 1
                || load_binance_testnet_attempt(&self.connection, &workspace_id, &proposal_id)?
                    .as_ref()
                    != Some(&attempt)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::BinanceTestnetOrderAttempt(Box::new(attempt)),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind == "bitget-demo-order-attempt" {
            let workspace_id = self.workspace_id()?;
            let (sequence, proposal_id, projection): (i64, String, String) = self.connection
                .query_row(
                    "SELECT sequence,proposal_id,projection FROM bitget_demo_order_attempts WHERE workspace_id=?1 AND attempt_id=?2",
                    params![workspace_id, id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else { storage_error(error) })?;
            let attempt: BitgetDemoOrderAttempt = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if sequence < 1
                || load_bitget_demo_attempt(&self.connection, &workspace_id, &proposal_id)?.as_ref()
                    != Some(&attempt)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::BitgetDemoOrderAttempt(Box::new(attempt)),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind == "binance-testnet-order-book" {
            let workspace_id = self.workspace_id()?;
            let book = load_binance_testnet_order_book(&self.connection, &workspace_id, id)?
                .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
            let sequence: i64 = self
                .connection
                .query_row(
                    "SELECT sequence FROM binance_testnet_order_books WHERE workspace_id=?1 AND connection_id=?2",
                    params![workspace_id, id],
                    |row| row.get(0),
                )
                .map_err(storage_error)?;
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::BinanceTestnetOrderBook(Box::new(book)),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind == "alpaca-paper-order-book" {
            let workspace_id = self.workspace_id()?;
            let book = load_alpaca_paper_order_book(&self.connection, &workspace_id, id)?
                .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
            let sequence: i64 = self.connection.query_row(
                "SELECT sequence FROM alpaca_paper_order_books WHERE workspace_id=?1 AND connection_id=?2",
                params![workspace_id, id],
                |row| row.get(0),
            ).map_err(storage_error)?;
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::AlpacaPaperOrderBook(Box::new(book)),
                last_sequence: u64::try_from(sequence).map_err(storage_error)?,
            });
        }
        if kind == "trading212-demo-order-book" {
            let workspace_id = self.workspace_id()?;
            let book = load_trading212_demo_order_book(&self.connection, &workspace_id, id)?
                .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
            let sequence: i64 = self.connection.query_row(
                "SELECT sequence FROM trading212_demo_order_books WHERE workspace_id=?1 AND connection_id=?2",
                params![workspace_id, id],
                |row| row.get(0),
            ).map_err(storage_error)?;
            return Ok(Snapshot {
                aggregate_type: kind.into(),
                aggregate_id: id.into(),
                projection: DomainProjection::Trading212DemoOrderBook(Box::new(book)),
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

    pub fn append_risk_decision(&mut self, mut decision: RiskDecision) -> Result<DomainEvent> {
        let workspace_id = self.workspace_id()?;
        if decision.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(&decision.proposal_id)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let stored_proposal: Option<(String, String)> = tx
            .query_row(
                "SELECT workspace_id,proposal_hash FROM order_proposals WHERE proposal_id=?1",
                [&decision.proposal_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(storage_error)?;
        let Some((proposal_workspace_id, proposal_hash)) = stored_proposal else {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        };
        if proposal_workspace_id != workspace_id || proposal_hash != decision.proposal_hash {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let previous: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(sequence),0) FROM risk_decisions WHERE workspace_id=?1 AND proposal_id=?2",
                params![workspace_id, decision.proposal_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if previous < 0 || previous >= MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let sequence = previous + 1;
        decision.state_version = format!("risk-decision:{}:{sequence}", decision.proposal_id);
        decision.validate(&workspace_id, &decision.proposal_id, sequence as u64)?;
        let projection = serde_json::to_string(&decision).map_err(storage_error)?;
        tx.execute(
            "INSERT INTO risk_decisions(decision_id,workspace_id,proposal_id,sequence,projection) VALUES(?1,?2,?3,?4,?5)",
            params![decision.decision_id, workspace_id, decision.proposal_id, sequence, projection],
        )
        .map_err(storage_error)?;
        let event = DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: "risk.decision.evaluated".into(),
            schema_version: 1,
            occurred_at: decision.evaluated_at.clone(),
            aggregate_type: "risk-decision".into(),
            aggregate_id: decision.proposal_id.clone(),
            sequence: sequence as u64,
            payload: DomainProjection::RiskDecision(Box::new(decision)),
        };
        tx.execute(
            "INSERT INTO outbox VALUES('risk-decision',?1,?2,?3,?4)",
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

    pub fn risk_decision_history(
        &self,
        workspace_id: &str,
        proposal_id: &str,
    ) -> Result<RiskDecisionHistory> {
        if workspace_id != self.workspace_id()? {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(proposal_id)?;
        let mut statement = self
            .connection
            .prepare(
                "SELECT sequence,decision_id,projection FROM risk_decisions WHERE workspace_id=?1 AND proposal_id=?2 ORDER BY sequence",
            )
            .map_err(storage_error)?;
        let rows = statement
            .query_map(params![workspace_id, proposal_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(storage_error)?;
        let mut decisions = Vec::new();
        for (index, row) in rows.enumerate() {
            let (sequence, decision_id, encoded) = row.map_err(storage_error)?;
            let expected_sequence = index as i64 + 1;
            if sequence != expected_sequence {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let decision: RiskDecision = serde_json::from_str(&encoded)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if decision.decision_id != decision_id {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            decision.validate(workspace_id, proposal_id, sequence as u64)?;
            decisions.push(decision);
        }
        Ok(RiskDecisionHistory {
            workspace_id: workspace_id.into(),
            proposal_id: proposal_id.into(),
            decisions,
        })
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

    pub fn save_alpaca_private_stream_health(
        &mut self,
        connection_id: &str,
        expected_state_version: &str,
        private_stream: &str,
        reconciliation: &str,
        reason: &str,
        last_event_at: Option<&str>,
    ) -> Result<DomainEvent> {
        if !matches!(
            private_stream,
            "CONNECTING" | "CONNECTED" | "DEGRADED" | "AUTH_FAILED" | "STOPPED"
        ) || !matches!(
            reconciliation,
            "REQUIRED" | "RUNNING" | "CURRENT" | "DEGRADED"
        ) || !valid_order_text(reason, 256)
            || last_event_at.is_some_and(|value| !valid_provider_time(value))
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let workspace_id = self.workspace_id()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (sequence, projection): (i64, String) = tx
            .query_row(
                "SELECT sequence,projection FROM accounts WHERE connection_id=?1",
                [connection_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        if sequence < 1 || sequence >= MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let mut account: AccountConnection = serde_json::from_str(&projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.connection_id != connection_id
            || account.state_version != expected_state_version
            || account.provider_id != "alpaca"
            || account.environment != "PAPER"
            || account.connection_state != ConnectionState::Connected
            || account.data.is_none()
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let next_sequence = sequence + 1;
        account.health.private_stream = private_stream.into();
        account.health.reconciliation = reconciliation.into();
        account.health.reason = reason.into();
        if let Some(last_event_at) = last_event_at {
            account.last_private_stream_event_at = Some(last_event_at.into());
        }
        account.updated_at = timestamp()?;
        let event = DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: "account.health.changed".into(),
            schema_version: 1,
            occurred_at: account.updated_at.clone(),
            aggregate_type: "account".into(),
            aggregate_id: connection_id.into(),
            sequence: next_sequence as u64,
            payload: DomainProjection::Account(Box::new(account.clone())),
        };
        let changed = tx.execute(
            "UPDATE accounts SET sequence=?1,projection=?2 WHERE connection_id=?3 AND sequence=?4",
            params![
                next_sequence,
                serde_json::to_string(&account).map_err(storage_error)?,
                connection_id,
                sequence,
            ],
        ).map_err(storage_error)?;
        if changed != 1 {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        tx.execute(
            "INSERT INTO outbox VALUES('account',?1,?2,?3,?4)",
            params![
                connection_id,
                next_sequence,
                event.event_id,
                serde_json::to_string(&event).map_err(storage_error)?,
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

    fn validate_strategy_run_projection(
        &self,
        run: &StrategyRun,
        workspace_id: &str,
    ) -> Result<()> {
        let invalid = || TradeXError::new("WORKSPACE_INTEGRITY_FAILED");
        if run.workspace_id != workspace_id
            || run.run_id.trim().is_empty()
            || run.run_id.len() > 128
            || run.run_id.chars().any(char::is_control)
            || run.strategy_hash.len() > 80
            || run.strategy_hash.chars().any(char::is_control)
            || run.instrument_id.len() > 128
            || run.dataset_id.len() > 128
            || run.created_at.len() > 64
            || run.updated_at.len() > 64
            || run.observed_at.len() > 64
            || run.request_hash.len() > 80
            || run.state_version.len() > 256
        {
            return Err(invalid());
        }
        let version = self
            .strategy_version(&run.strategy_version_id)
            .map_err(|_| invalid())?;
        let request = StrategyRunRequest {
            workspace_id: run.workspace_id.clone(),
            strategy_version_id: run.strategy_version_id.clone(),
            expected_strategy_hash: Some(run.strategy_hash.clone()),
            instrument_id: run.instrument_id.clone(),
            dataset_id: run.dataset_id.clone(),
            start_at: run.start_at.clone(),
            end_at: run.end_at.clone(),
            parameters: run.parameters.clone(),
            fixture_scenario: None,
        };
        crate::strategy::validate_run_request(&request).map_err(|_| invalid())?;
        let allowed: std::collections::HashSet<&str> = version
            .definition
            .parameters
            .iter()
            .map(|item| item.name.as_str())
            .collect();
        let mut seen = std::collections::HashSet::new();
        if run
            .parameters
            .iter()
            .any(|item| !allowed.contains(item.name.as_str()) || !seen.insert(item.name.as_str()))
        {
            return Err(invalid());
        }
        if run.strategy_hash != version.source_hash
            || !valid_strategy_timestamp(&run.created_at)
            || !valid_strategy_timestamp(&run.updated_at)
            || !valid_strategy_timestamp(&run.observed_at)
            || OffsetDateTime::parse(&run.created_at, &Rfc3339)
                .ok()
                .zip(OffsetDateTime::parse(&run.updated_at, &Rfc3339).ok())
                .is_some_and(|(created, updated)| created > updated)
            || crate::strategy::run_identity_hash(
                &version,
                &request,
                &run.parameters,
                &run.observed_at,
            )
            .map_err(|_| invalid())?
                != run.request_hash
            || run
                .fixture_label
                .as_deref()
                .is_some_and(|label| label != "TRADEX_STRATEGY_FIXTURE")
        {
            return Err(invalid());
        }
        match run.state {
            StrategyRunState::Queued | StrategyRunState::Running => {
                if run.signal.is_some() || run.failure.is_some() {
                    return Err(invalid());
                }
            }
            StrategyRunState::Completed => {
                let Some(signal) = run.signal.as_ref() else {
                    return Err(invalid());
                };
                if run.failure.is_some()
                    || crate::strategy::validate_signal(
                        signal,
                        &version,
                        &request,
                        &run.observed_at,
                    )
                    .is_err()
                {
                    return Err(invalid());
                }
            }
            StrategyRunState::Failed | StrategyRunState::Cancelled => {
                let Some(failure) = run.failure.as_ref() else {
                    return Err(invalid());
                };
                if run.signal.is_some()
                    || crate::strategy::validate_failure(failure).is_err()
                    || (run.state == StrategyRunState::Cancelled
                        && failure.code != "STRATEGY_CANCELLED")
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }

    pub fn reconcile_strategy_runs(&mut self) -> Result<()> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare(
                "SELECT run_id,workspace_id,sequence,projection FROM strategy_runs WHERE workspace_id=?1",
            )
            .map_err(storage_error)?;
        let rows = query
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?;
        let mut projections = Vec::new();
        for row in rows {
            let (id, row_workspace, sequence, projection) = row.map_err(storage_error)?;
            let run: StrategyRun = serde_json::from_str(&projection).map_err(storage_error)?;
            if id != run.run_id
                || row_workspace != workspace_id
                || run.workspace_id != workspace_id
                || !(1..=MAX_SEQUENCE as i64).contains(&sequence)
                || run.state_version != format!("strategy-run:{id}:{sequence}")
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            projections.push(run);
        }
        drop(query);
        let mut active = Vec::new();
        for run in projections {
            self.validate_strategy_run_projection(&run, &workspace_id)?;
            if matches!(
                run.state,
                StrategyRunState::Queued | StrategyRunState::Running
            ) {
                active.push(run);
            }
        }
        for mut run in active {
            run.state = StrategyRunState::Cancelled;
            run.failure = Some(StrategyFailure {
                code: "STRATEGY_CANCELLED".into(),
                reason: "The workspace session ended before the strategy run completed.".into(),
                remediation: vec!["retry_strategy_run".into()],
            });
            self.save_strategy_run(run)?;
        }
        Ok(())
    }

    pub fn strategies(&self) -> Result<StrategyLibrary> {
        let workspace_id = self.workspace_id()?;
        let mut versions_query = self
            .connection
            .prepare("SELECT strategy_version_id,workspace_id,strategy_id,revision,sequence,projection FROM strategy_versions WHERE workspace_id=?1 ORDER BY strategy_id,revision DESC")
            .map_err(storage_error)?;
        let rows = versions_query
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(storage_error)?;
        let mut versions = Vec::new();
        let mut max_sequence = 0_i64;
        for row in rows {
            let (id, row_workspace, strategy_id, revision, sequence, projection) =
                row.map_err(storage_error)?;
            max_sequence = max_sequence.max(sequence);
            let version: StrategyVersion =
                serde_json::from_str(&projection).map_err(storage_error)?;
            if version.strategy_version_id != id
                || version.workspace_id != row_workspace
                || version.strategy_id != strategy_id
                || version.revision != u64::try_from(revision).map_err(storage_error)?
                || version.state_version != format!("strategy-version:{id}:{sequence}")
                || row_workspace != workspace_id
                || !(1..=MAX_SEQUENCE as i64).contains(&sequence)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            crate::strategy::validate_definition(&version.definition)?;
            if crate::strategy::canonical_version_hash(&version.definition)? != version.source_hash
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            versions.push(version);
        }
        let mut runs_query = self
            .connection
            .prepare("SELECT run_id,workspace_id,sequence,projection FROM strategy_runs WHERE workspace_id=?1 ORDER BY sequence DESC,run_id")
            .map_err(storage_error)?;
        let rows = runs_query
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?;
        let mut runs = Vec::new();
        for row in rows {
            let (id, row_workspace, sequence, projection) = row.map_err(storage_error)?;
            max_sequence = max_sequence.max(sequence);
            let run: StrategyRun = serde_json::from_str(&projection).map_err(storage_error)?;
            if run.run_id != id
                || run.workspace_id != row_workspace
                || row_workspace != workspace_id
                || run.state_version != format!("strategy-run:{id}:{sequence}")
                || !(1..=MAX_SEQUENCE as i64).contains(&sequence)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            self.validate_strategy_run_projection(&run, &workspace_id)?;
            runs.push(StrategyRunSummary {
                run_id: run.run_id,
                strategy_version_id: run.strategy_version_id,
                state: run.state,
                updated_at: run.updated_at,
                failure_code: run.failure.map(|failure| failure.code),
            });
        }
        if versions.len() > 256
            || runs.len() > 256
            || !(0..=MAX_SEQUENCE as i64).contains(&max_sequence)
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(StrategyLibrary {
            workspace_id: workspace_id.clone(),
            state_version: format!("strategies:{workspace_id}:{max_sequence}"),
            versions,
            runs,
        })
    }

    pub fn strategy_version(&self, id: &str) -> Result<StrategyVersion> {
        let workspace_id = self.workspace_id()?;
        let (row_workspace, strategy_id, revision, sequence, projection): (String, String, i64, i64, String) = self
            .connection
            .query_row(
                "SELECT workspace_id,strategy_id,revision,sequence,projection FROM strategy_versions WHERE strategy_version_id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .map_err(|_| TradeXError::new("STRATEGY_VERSION_NOT_FOUND"))?;
        let version: StrategyVersion = serde_json::from_str(&projection).map_err(storage_error)?;
        if row_workspace != workspace_id
            || version.workspace_id != workspace_id
            || version.strategy_version_id != id
            || version.strategy_id != strategy_id
            || version.revision != u64::try_from(revision).map_err(storage_error)?
            || !(1..=MAX_SEQUENCE as i64).contains(&sequence)
            || version.state_version != format!("strategy-version:{id}:{sequence}")
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        crate::strategy::validate_definition(&version.definition)?;
        if crate::strategy::canonical_version_hash(&version.definition)? != version.source_hash {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(version)
    }

    pub fn save_strategy_version(&mut self, input: &StrategySave) -> Result<StrategyVersion> {
        if input.workspace_id != self.workspace_id()? {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        crate::strategy::validate_definition(&input.definition)?;
        if input
            .strategy_id
            .as_deref()
            .is_some_and(|id| id.is_empty() || id.len() > 128 || id.chars().any(char::is_control))
        {
            return Err(TradeXError::new("STRATEGY_VERSION_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let strategy_id = input
            .strategy_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let revision: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(revision),0)+1 FROM strategy_versions WHERE workspace_id=?1 AND strategy_id=?2",
                params![&input.workspace_id, &strategy_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !(1..=MAX_SEQUENCE as i64).contains(&revision) {
            return Err(TradeXError::new("STRATEGY_VERSION_LIMIT"));
        }
        let count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM strategy_versions WHERE workspace_id=?1",
                [&input.workspace_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !(0..256).contains(&count) {
            return Err(TradeXError::new("STRATEGY_VERSION_LIMIT"));
        }
        let sequence = next_strategy_sequence(&tx, &input.workspace_id)?;
        let version_id = Uuid::new_v4().to_string();
        let created_at = timestamp()?;
        let version = StrategyVersion {
            strategy_version_id: version_id.clone(),
            strategy_id,
            workspace_id: input.workspace_id.clone(),
            revision: u64::try_from(revision).map_err(storage_error)?,
            definition: input.definition.clone(),
            source_hash: crate::strategy::canonical_version_hash(&input.definition)?,
            created_at,
            state_version: format!("strategy-version:{version_id}:{sequence}"),
        };
        tx.execute(
            "INSERT INTO strategy_versions(strategy_version_id,workspace_id,strategy_id,revision,sequence,projection) VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                &version.strategy_version_id,
                &version.workspace_id,
                &version.strategy_id,
                version.revision as i64,
                sequence,
                serde_json::to_string(&version).map_err(storage_error)?,
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(version)
    }

    pub fn strategy_run(&self, id: &str) -> Result<StrategyRun> {
        let workspace_id = self.workspace_id()?;
        let (row_workspace, sequence, projection): (String, i64, String) = self
            .connection
            .query_row(
                "SELECT workspace_id,sequence,projection FROM strategy_runs WHERE run_id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| TradeXError::new("STRATEGY_RUN_NOT_FOUND"))?;
        let run: StrategyRun = serde_json::from_str(&projection).map_err(storage_error)?;
        if row_workspace != workspace_id
            || run.workspace_id != workspace_id
            || run.run_id != id
            || !(1..=MAX_SEQUENCE as i64).contains(&sequence)
            || run.state_version != format!("strategy-run:{id}:{sequence}")
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        self.validate_strategy_run_projection(&run, &workspace_id)?;
        Ok(run)
    }

    pub fn save_strategy_run(&mut self, mut run: StrategyRun) -> Result<StrategyRun> {
        if run.workspace_id != self.workspace_id()? || run.run_id.is_empty() {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let existing_workspace: Option<String> = tx
            .query_row(
                "SELECT workspace_id FROM strategy_runs WHERE run_id=?1",
                [&run.run_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        if existing_workspace
            .as_deref()
            .is_some_and(|workspace| workspace != run.workspace_id)
        {
            return Err(TradeXError::new("STRATEGY_RUN_NOT_FOUND"));
        }
        let sequence = next_strategy_sequence(&tx, &run.workspace_id)?;
        run.updated_at = timestamp()?;
        run.state_version = format!("strategy-run:{}:{sequence}", run.run_id);
        tx.execute(
            "INSERT INTO strategy_runs(run_id,workspace_id,sequence,projection) VALUES(?1,?2,?3,?4) ON CONFLICT(run_id) DO UPDATE SET workspace_id=excluded.workspace_id,sequence=excluded.sequence,projection=excluded.projection",
            params![
                &run.run_id,
                &run.workspace_id,
                sequence,
                serde_json::to_string(&run).map_err(storage_error)?,
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(run)
    }

    fn validate_backtest_run_projection(
        &self,
        run: &BacktestRun,
        workspace_id: &str,
    ) -> Result<()> {
        let invalid = || TradeXError::new("WORKSPACE_INTEGRITY_FAILED");
        if run.workspace_id != workspace_id
            || run.run_id.trim().is_empty()
            || run.run_id.len() > 128
            || run.run_id.chars().any(char::is_control)
            || run.strategy_hash.len() > 80
            || run.strategy_hash.chars().any(char::is_control)
            || run.instrument_id.len() > 128
            || run.dataset_id.len() > 128
            || run.bar_interval.len() > 32
            || run.starting_cash.len() > 128
            || run.commission.len() > 128
            || run.slippage.len() > 128
            || run
                .portfolio_seed
                .as_deref()
                .is_some_and(|seed| seed.len() > 128)
            || run.created_at.len() > 64
            || run.updated_at.len() > 64
            || run.observed_at.len() > 64
            || run.request_hash.len() > 80
            || run.state_version.len() > 256
        {
            return Err(invalid());
        }
        let version = self
            .strategy_version(&run.strategy_version_id)
            .map_err(|_| invalid())?;
        let request = BacktestRunRequest {
            workspace_id: run.workspace_id.clone(),
            strategy_version_id: run.strategy_version_id.clone(),
            expected_strategy_hash: Some(run.strategy_hash.clone()),
            instrument_id: run.instrument_id.clone(),
            dataset_id: run.dataset_id.clone(),
            start_at: run.start_at.clone(),
            end_at: run.end_at.clone(),
            bar_interval: run.bar_interval.clone(),
            starting_cash: run.starting_cash.clone(),
            commission: run.commission.clone(),
            slippage: run.slippage.clone(),
            portfolio_seed: run.portfolio_seed.clone(),
            parameters: run.parameters.clone(),
            fixture_scenario: None,
        };
        crate::backtest::validate_run_request(&request).map_err(|_| invalid())?;
        let allowed: std::collections::HashSet<&str> = version
            .definition
            .parameters
            .iter()
            .map(|item| item.name.as_str())
            .collect();
        let mut seen = std::collections::HashSet::new();
        if run
            .parameters
            .iter()
            .any(|item| !allowed.contains(item.name.as_str()) || !seen.insert(item.name.as_str()))
        {
            return Err(invalid());
        }
        if run.strategy_hash != version.source_hash
            || !valid_strategy_timestamp(&run.created_at)
            || !valid_strategy_timestamp(&run.updated_at)
            || !valid_strategy_timestamp(&run.observed_at)
            || OffsetDateTime::parse(&run.created_at, &Rfc3339)
                .ok()
                .zip(OffsetDateTime::parse(&run.updated_at, &Rfc3339).ok())
                .is_some_and(|(created, updated)| created > updated)
            || crate::backtest::run_identity_hash(&version, &request, &run.parameters)
                .map_err(|_| invalid())?
                != run.request_hash
            || run
                .fixture_label
                .as_deref()
                .is_some_and(|label| label != "TRADEX_BACKTEST_FIXTURE")
        {
            return Err(invalid());
        }
        match run.state {
            BacktestRunState::Queued | BacktestRunState::Running => {
                if run.failure.is_some() || run.result.is_some() {
                    return Err(invalid());
                }
            }
            BacktestRunState::Completed => {
                if run.failure.is_some()
                    || run
                        .result
                        .as_ref()
                        .is_none_or(|result| crate::backtest::validate_result(result, run).is_err())
                {
                    return Err(invalid());
                }
            }
            BacktestRunState::Failed | BacktestRunState::Cancelled => {
                let Some(failure) = run.failure.as_ref() else {
                    return Err(invalid());
                };
                if run.result.is_some()
                    || crate::backtest::validate_failure(failure).is_err()
                    || (run.state == BacktestRunState::Cancelled
                        && failure.code != "BACKTEST_CANCELLED")
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }

    pub fn reconcile_backtest_runs(&mut self) -> Result<()> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare(
                "SELECT run_id,workspace_id,sequence,projection FROM backtest_runs WHERE workspace_id=?1",
            )
            .map_err(storage_error)?;
        let rows = query
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?;
        let mut projections = Vec::new();
        for row in rows {
            let (id, row_workspace, sequence, projection) = row.map_err(storage_error)?;
            let run: BacktestRun = serde_json::from_str(&projection).map_err(storage_error)?;
            if id != run.run_id
                || row_workspace != workspace_id
                || run.workspace_id != workspace_id
                || !(1..=MAX_SEQUENCE as i64).contains(&sequence)
                || run.state_version != format!("backtest-run:{id}:{sequence}")
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            projections.push(run);
        }
        drop(query);
        for mut run in projections {
            self.validate_backtest_run_projection(&run, &workspace_id)?;
            if matches!(
                run.state,
                BacktestRunState::Queued | BacktestRunState::Running
            ) {
                run.state = BacktestRunState::Cancelled;
                run.failure = Some(BacktestFailure {
                    code: "BACKTEST_CANCELLED".into(),
                    reason: "The workspace session ended before the backtest completed.".into(),
                    remediation: vec!["retry_backtest_run".into()],
                });
                self.save_backtest_run(run)?;
            }
        }
        Ok(())
    }

    pub fn backtest_run(&self, id: &str) -> Result<BacktestRun> {
        let workspace_id = self.workspace_id()?;
        let (row_workspace, sequence, projection): (String, i64, String) = self
            .connection
            .query_row(
                "SELECT workspace_id,sequence,projection FROM backtest_runs WHERE run_id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| TradeXError::new("BACKTEST_RUN_NOT_FOUND"))?;
        let run: BacktestRun = serde_json::from_str(&projection).map_err(storage_error)?;
        if row_workspace != workspace_id
            || run.workspace_id != workspace_id
            || run.run_id != id
            || !(1..=MAX_SEQUENCE as i64).contains(&sequence)
            || run.state_version != format!("backtest-run:{id}:{sequence}")
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        self.validate_backtest_run_projection(&run, &workspace_id)?;
        Ok(run)
    }

    pub fn backtest_runs(&self) -> Result<BacktestLibrary> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare(
                "SELECT run_id,workspace_id,sequence,projection FROM backtest_runs WHERE workspace_id=?1 ORDER BY sequence DESC,run_id",
            )
            .map_err(storage_error)?;
        let rows = query
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(storage_error)?;
        let mut runs = Vec::new();
        let mut max_sequence = 0_i64;
        for row in rows {
            let (id, row_workspace, sequence, projection) = row.map_err(storage_error)?;
            max_sequence = max_sequence.max(sequence);
            let run: BacktestRun = serde_json::from_str(&projection).map_err(storage_error)?;
            if run.run_id != id
                || run.workspace_id != row_workspace
                || row_workspace != workspace_id
                || run.state_version != format!("backtest-run:{id}:{sequence}")
                || !(1..=MAX_SEQUENCE as i64).contains(&sequence)
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            self.validate_backtest_run_projection(&run, &workspace_id)?;
            let result = run.result.as_ref();
            runs.push(BacktestRunSummary {
                run_id: run.run_id,
                strategy_version_id: run.strategy_version_id,
                strategy_hash: run.strategy_hash,
                instrument_id: run.instrument_id,
                dataset_id: run.dataset_id,
                start_at: run.start_at,
                end_at: run.end_at,
                bar_interval: run.bar_interval,
                state: run.state,
                request_hash: run.request_hash,
                updated_at: run.updated_at,
                result_hash: result.map(|value| value.result_hash.clone()),
                trade_count: result.map(|value| value.metrics.trade_count),
            });
        }
        if runs.len() > 256 || !(0..=MAX_SEQUENCE as i64).contains(&max_sequence) {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(BacktestLibrary {
            workspace_id: workspace_id.clone(),
            state_version: format!("backtests:{workspace_id}:{max_sequence}"),
            runs,
        })
    }

    pub fn save_backtest_run(&mut self, run: BacktestRun) -> Result<BacktestRun> {
        self.save_backtest_run_cas(run, None)
    }

    pub fn save_backtest_run_cas(
        &mut self,
        mut run: BacktestRun,
        expected_state_version: Option<&str>,
    ) -> Result<BacktestRun> {
        if run.workspace_id != self.workspace_id()? || run.run_id.is_empty() {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let existing_projection: Option<(String, String)> = tx
            .query_row(
                "SELECT workspace_id,projection FROM backtest_runs WHERE run_id=?1",
                [&run.run_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(storage_error)?;
        if let Some((existing_workspace, projection)) = existing_projection {
            if existing_workspace != run.workspace_id {
                return Err(TradeXError::new("BACKTEST_RUN_NOT_FOUND"));
            }
            let existing: BacktestRun = serde_json::from_str(&projection).map_err(storage_error)?;
            if existing.workspace_id != existing_workspace || existing.run_id != run.run_id {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            if expected_state_version
                .is_some_and(|expected| expected.is_empty() || expected.len() > 256)
                || expected_state_version.is_some_and(|expected| expected != existing.state_version)
                || (expected_state_version.is_none() && run.state_version != existing.state_version)
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            if matches!(
                existing.state,
                BacktestRunState::Completed
                    | BacktestRunState::Failed
                    | BacktestRunState::Cancelled
            ) {
                return Err(TradeXError::new("BACKTEST_RUN_TERMINAL_IMMUTABLE"));
            }
            let valid_transition = matches!(
                (&existing.state, &run.state),
                (BacktestRunState::Queued, BacktestRunState::Running)
                    | (BacktestRunState::Queued, BacktestRunState::Failed)
                    | (BacktestRunState::Queued, BacktestRunState::Cancelled)
                    | (BacktestRunState::Running, BacktestRunState::Completed)
                    | (BacktestRunState::Running, BacktestRunState::Failed)
                    | (BacktestRunState::Running, BacktestRunState::Cancelled)
            );
            if !valid_transition {
                return Err(TradeXError::new("BACKTEST_RUN_INVALID"));
            }
        } else {
            if expected_state_version.is_some() || run.state != BacktestRunState::Queued {
                return Err(TradeXError::new("BACKTEST_RUN_INVALID"));
            }
        }
        let sequence = next_backtest_sequence(&tx, &run.workspace_id)?;
        run.updated_at = timestamp()?;
        run.state_version = format!("backtest-run:{}:{sequence}", run.run_id);
        tx.execute(
            "INSERT INTO backtest_runs(run_id,workspace_id,sequence,projection) VALUES(?1,?2,?3,?4) ON CONFLICT(run_id) DO UPDATE SET workspace_id=excluded.workspace_id,sequence=excluded.sequence,projection=excluded.projection",
            params![
                &run.run_id,
                &run.workspace_id,
                sequence,
                serde_json::to_string(&run).map_err(storage_error)?,
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(run)
    }

    pub fn order_drafts(&self) -> Result<OrderDraftLibrary> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare(
                "SELECT draft_id,workspace_id,draft_version,sequence,projection FROM order_drafts WHERE workspace_id=?1 ORDER BY sequence DESC,draft_id",
            )
            .map_err(storage_error)?;
        let rows = query
            .query_map([workspace_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(storage_error)?;
        let mut drafts = Vec::new();
        let mut max_sequence = 0_i64;
        for row in rows {
            let (draft_id, row_workspace_id, draft_version, sequence, projection) =
                row.map_err(storage_error)?;
            max_sequence = max_sequence.max(sequence);
            let draft = decode_order_draft(
                &projection,
                &draft_id,
                &row_workspace_id,
                draft_version,
                sequence,
                &workspace_id,
            )?;
            drafts.push(OrderDraftSummary {
                draft_id: draft.draft_id,
                workspace_id: draft.workspace_id,
                draft_version: draft.draft_version,
                state_version: draft.state_version,
                instrument_id: draft.fields.instrument_id,
                environment: draft.fields.environment,
                updated_at: draft.updated_at,
            });
        }
        if !(0..=MAX_SEQUENCE as i64).contains(&max_sequence) || drafts.len() > 256 {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(OrderDraftLibrary {
            workspace_id: workspace_id.clone(),
            state_version: format!("order-drafts:{workspace_id}:{max_sequence}"),
            drafts,
        })
    }

    pub fn order_draft(&self, draft_id: &str) -> Result<OrderDraft> {
        let workspace_id = self.workspace_id()?;
        let (row_workspace_id, draft_version, sequence, projection): (String, i64, i64, String) = self
            .connection
            .query_row(
                "SELECT workspace_id,draft_version,sequence,projection FROM order_drafts WHERE workspace_id=?1 AND draft_id=?2",
                params![workspace_id, draft_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("ORDER_DRAFT_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        decode_order_draft(
            &projection,
            draft_id,
            &row_workspace_id,
            draft_version,
            sequence,
            &workspace_id,
        )
    }

    pub fn order_proposals(&self) -> Result<OrderProposalLibrary> {
        let workspace_id = self.workspace_id()?;
        let mut query = self
            .connection
            .prepare(
                "SELECT proposal_id,workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE workspace_id=?1 ORDER BY sequence DESC,proposal_id",
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
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(storage_error)?;
        let mut proposals = Vec::new();
        let mut max_sequence = 0_i64;
        let mut max_event_sequence = 0_i64;
        for row in rows {
            let (
                proposal_id,
                row_workspace_id,
                draft_id,
                draft_version,
                proposal_hash,
                sequence,
                projection,
            ) = row.map_err(storage_error)?;
            max_sequence = max_sequence.max(sequence);
            let stored = decode_stored_order_proposal(
                &projection,
                &proposal_id,
                &row_workspace_id,
                &draft_id,
                draft_version,
                &proposal_hash,
                sequence,
                &workspace_id,
            )?;
            let (status, invalidation_reason, last_event_sequence) =
                proposal_event_state(&self.connection, &proposal_id, &workspace_id)?;
            max_event_sequence = max_event_sequence.max(last_event_sequence);
            proposals.push(OrderProposalSummary {
                proposal_id: stored.proposal_id,
                workspace_id: stored.workspace_id,
                draft_id: stored.draft_id,
                draft_version: stored.draft_version,
                proposal_hash: stored.proposal_hash,
                status,
                invalidation_reason,
                created_at: stored.created_at,
                state_version: format!("order-proposal:{}:{last_event_sequence}", proposal_id),
            });
        }
        if !(0..=MAX_SEQUENCE as i64).contains(&max_sequence)
            || !(0..=MAX_SEQUENCE as i64).contains(&max_event_sequence)
            || proposals.len() > 256
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(OrderProposalLibrary {
            workspace_id: workspace_id.clone(),
            state_version: format!(
                "order-proposals:{workspace_id}:{max_sequence}:{max_event_sequence}"
            ),
            proposals,
        })
    }

    pub fn order_proposal(&self, proposal_id: &str) -> Result<OrderProposal> {
        validate_order_proposal_id(proposal_id)?;
        let workspace_id = self.workspace_id()?;
        let (row_workspace_id, draft_id, draft_version, proposal_hash, sequence, projection): (
            String,
            String,
            i64,
            String,
            i64,
            String,
        ) = self
            .connection
            .query_row(
                "SELECT workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE workspace_id=?1 AND proposal_id=?2",
                params![workspace_id, proposal_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("ORDER_PROPOSAL_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let stored = decode_stored_order_proposal(
            &projection,
            proposal_id,
            &row_workspace_id,
            &draft_id,
            draft_version,
            &proposal_hash,
            sequence,
            &workspace_id,
        )?;
        materialize_order_proposal(&self.connection, stored)
    }

    pub fn alpaca_paper_order_attempt(
        &self,
        workspace_id: &str,
        proposal_id: &str,
    ) -> Result<Option<AlpacaPaperOrderAttempt>> {
        if workspace_id != self.workspace_id()? {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(proposal_id)?;
        load_alpaca_paper_attempt(&self.connection, workspace_id, proposal_id)
    }

    pub fn binance_testnet_order_attempt(
        &self,
        workspace_id: &str,
        proposal_id: &str,
    ) -> Result<Option<BinanceTestnetOrderAttempt>> {
        if self.workspace_id()? != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(proposal_id)?;
        load_binance_testnet_attempt(&self.connection, workspace_id, proposal_id)
    }

    pub fn bitget_demo_order_attempt(
        &self,
        workspace_id: &str,
        proposal_id: &str,
    ) -> Result<Option<BitgetDemoOrderAttempt>> {
        if self.workspace_id()? != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(proposal_id)?;
        load_bitget_demo_attempt(&self.connection, workspace_id, proposal_id)
    }

    pub fn binance_testnet_order_book(
        &self,
        workspace_id: &str,
        connection_id: &str,
    ) -> Result<Option<BinanceTestnetOrderBook>> {
        if workspace_id != self.workspace_id()? || !valid_order_text(connection_id, 128) {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let account = self.account(connection_id)?;
        account.validate_persisted(workspace_id)?;
        if account.provider_id != "binance" || account.environment != "TESTNET" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        let remote_account_id = account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.as_str())
            .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
        let mut book =
            load_binance_testnet_order_book(&self.connection, workspace_id, connection_id)?;
        if book
            .as_ref()
            .is_some_and(|book| book.remote_account_id != remote_account_id)
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        if let Some(book) = &mut book
            && book.status == BinanceTestnetOrderBookStatus::Current
            && (account.health.private_stream != "CONNECTED"
                || account.health.reconciliation != "CURRENT")
        {
            book.status = BinanceTestnetOrderBookStatus::Stale;
            book.reason = Some("REFRESH_REQUIRED".into());
        }
        Ok(book)
    }

    pub fn begin_binance_testnet_order_cancel(
        &mut self,
        input: &BinanceTestnetOrderCancel,
    ) -> Result<BinanceTestnetOrderBook> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id
            || !input.confirmed
            || !valid_order_text(&input.connection_id, 128)
            || !valid_order_text(&input.expected_connection_state_version, 256)
            || !valid_order_text(&input.expected_book_state_version, 256)
            || !matches!(input.symbol.as_str(), "BTCUSDT" | "ETHUSDT")
            || !valid_binance_id(&input.provider_order_id)
            || Uuid::parse_str(&input.idempotency_key).is_err()
        {
            return Err(TradeXError::new("ORDER_CONFIRMATION_REQUIRED"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&input.connection_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection = serde_json::from_str(&projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.state_version != input.expected_connection_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if account.provider_id != "binance" || account.environment != "TESTNET" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        if account.connection_state != ConnectionState::Connected
            || account.health.credential != "CONFIGURED"
            || account.health.authentication != "VALID"
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let account_id = account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.as_str())
            .unwrap_or_default();
        let mut book = load_binance_testnet_order_book(&tx, &workspace_id, &input.connection_id)?
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        if book.state_version != input.expected_book_state_version
            || book.remote_account_id != account_id
            || book.status != BinanceTestnetOrderBookStatus::Current
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if book
            .orders
            .iter()
            .any(|order| order.cancel_idempotency_key.as_deref() == Some(&input.idempotency_key))
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let index = book
            .orders
            .iter()
            .position(|order| {
                order.symbol == input.symbol && order.provider_order_id == input.provider_order_id
            })
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let order = &book.orders[index];
        if order.cancel_state != BinanceTestnetOrderCancelState::None {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if !order.pending
            || order
                .remaining_quantity
                .as_deref()
                .is_none_or(|quantity| quantity == "0")
            || !matches!(order.provider_status.as_str(), "NEW" | "PARTIALLY_FILLED")
        {
            return Err(TradeXError::new("ORDER_NOT_CANCELABLE"));
        }
        let observed_at = OffsetDateTime::parse(&order.observed_at, &Rfc3339)
            .map_err(|_| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let age = OffsetDateTime::now_utc() - observed_at;
        if age.is_negative() || age > time::Duration::seconds(60) {
            return Err(TradeXError::new("ORDER_CONFIRMATION_EXPIRED"));
        }
        book.orders[index].cancel_state = BinanceTestnetOrderCancelState::Submitting;
        book.orders[index].cancel_idempotency_key = Some(input.idempotency_key.clone());
        book.orders[index].cancel_error = None;
        let sequence: i64 = tx
            .query_row(
                "SELECT sequence FROM binance_testnet_order_books WHERE workspace_id=?1 AND connection_id=?2",
                params![workspace_id, input.connection_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        let next_sequence = sequence
            .checked_add(1)
            .filter(|next| *next <= MAX_SEQUENCE as i64)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        book.state_version =
            binance_testnet_order_book_version(&input.connection_id, next_sequence as u64);
        book.observed_at = timestamp()?;
        validate_binance_testnet_order_book(&book)?;
        let updated = tx
            .execute(
                "UPDATE binance_testnet_order_books SET sequence=?1,projection=?2 WHERE workspace_id=?3 AND connection_id=?4 AND sequence=?5",
                params![next_sequence, serde_json::to_string(&book).map_err(storage_error)?, workspace_id, input.connection_id, sequence],
            )
            .map_err(storage_error)?;
        if updated != 1 {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        write_binance_testnet_order_book_event(&tx, &book, next_sequence)?;
        tx.commit().map_err(storage_error)?;
        Ok(book)
    }

    pub fn trading212_demo_order_attempt(
        &self,
        workspace_id: &str,
        proposal_id: &str,
    ) -> Result<Option<Trading212DemoOrderAttempt>> {
        if self.workspace_id()? != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(proposal_id)?;
        load_trading212_demo_attempt(&self.connection, workspace_id, proposal_id)
    }

    pub fn alpaca_paper_order_book(
        &self,
        workspace_id: &str,
        connection_id: &str,
    ) -> Result<Option<AlpacaPaperOrderBook>> {
        if workspace_id != self.workspace_id()? || !valid_order_text(connection_id, 128) {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let account = self.account(connection_id)?;
        account.validate_persisted(workspace_id)?;
        if account.provider_id != "alpaca" || account.environment != "PAPER" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        let remote_account_id = account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.as_str())
            .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
        let mut book = load_alpaca_paper_order_book(&self.connection, workspace_id, connection_id)?;
        if book
            .as_ref()
            .is_some_and(|book| book.remote_account_id != remote_account_id)
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        if let Some(book) = &mut book
            && book.status == AlpacaPaperOrderBookStatus::Current
        {
            book.status = AlpacaPaperOrderBookStatus::Stale;
            book.reason = Some("FULL_ORDER_BOOK_REFRESH_REQUIRED".into());
        }
        Ok(book)
    }

    pub fn trading212_demo_order_book(
        &self,
        workspace_id: &str,
        connection_id: &str,
    ) -> Result<Option<Trading212DemoOrderBook>> {
        if workspace_id != self.workspace_id()? || !valid_order_text(connection_id, 128) {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let account = self.account(connection_id)?;
        account.validate_persisted(workspace_id)?;
        if account.provider_id != "trading212" || account.environment != "DEMO" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        let remote_account_id = account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.as_str())
            .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
        let mut book =
            load_trading212_demo_order_book(&self.connection, workspace_id, connection_id)?;
        if book
            .as_ref()
            .is_some_and(|book| book.remote_account_id != remote_account_id)
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        if let Some(book) = &mut book
            && book.status == Trading212DemoOrderBookStatus::Current
        {
            book.status = Trading212DemoOrderBookStatus::Stale;
            book.reason = Some("FULL_ORDER_BOOK_REFRESH_REQUIRED".into());
        }
        Ok(book)
    }

    pub fn begin_trading212_demo_order_cancel(
        &mut self,
        input: &Trading212DemoOrderCancel,
    ) -> Result<Trading212DemoOrderBook> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id
            || !input.confirmed
            || !valid_order_text(&input.expected_connection_state_version, 256)
            || !valid_order_text(&input.expected_book_state_version, 256)
            || !valid_order_text(&input.idempotency_key, 128)
            || !crate::provider_io::valid_t212_order_id(&input.provider_order_id)
        {
            return Err(TradeXError::new("ORDER_CONFIRMATION_REQUIRED"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let account_projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&input.connection_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection = serde_json::from_str(&account_projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.state_version != input.expected_connection_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if account.provider_id != "trading212" || account.environment != "DEMO" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        if account.connection_state != ConnectionState::Connected
            || account.health.credential != "CONFIGURED"
            || account.health.authentication != "VALID"
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let mut book = load_trading212_demo_order_book(&tx, &workspace_id, &input.connection_id)?
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let account_id = account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.as_str())
            .unwrap_or_default();
        if book.state_version != input.expected_book_state_version
            || book.remote_account_id != account_id
            || book.status != Trading212DemoOrderBookStatus::Current
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let index = book
            .orders
            .iter()
            .position(|order| order.provider_order_id == input.provider_order_id)
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        if matches!(
            book.orders[index].cancel_state,
            Trading212DemoCancelState::Submitting | Trading212DemoCancelState::Pending
        ) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if !book.orders[index].pending
            || !matches!(
                book.orders[index].provider_status.as_str(),
                "CONFIRMED" | "NEW" | "PARTIALLY_FILLED"
            )
        {
            return Err(TradeXError::new("ORDER_NOT_CANCELABLE"));
        }
        let observed_at = OffsetDateTime::parse(&book.orders[index].observed_at, &Rfc3339)
            .map_err(|_| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let age = OffsetDateTime::now_utc() - observed_at;
        if age.is_negative() || age > time::Duration::seconds(60) {
            return Err(TradeXError::new("ORDER_CONFIRMATION_EXPIRED"));
        }
        book.orders[index].cancel_state = Trading212DemoCancelState::Submitting;
        book.orders[index].cancel_idempotency_key = Some(input.idempotency_key.clone());
        book.orders[index].cancel_error = None;
        let sequence: i64 = tx
            .query_row(
                "SELECT sequence FROM trading212_demo_order_books WHERE workspace_id=?1 AND connection_id=?2",
                params![workspace_id, input.connection_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        let next_sequence = sequence
            .checked_add(1)
            .filter(|next| *next <= MAX_SEQUENCE as i64)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        book.state_version =
            trading212_demo_order_book_version(&input.connection_id, next_sequence as u64);
        book.observed_at = timestamp()?;
        validate_trading212_demo_order_book(&book)?;
        let changed = tx.execute(
            "UPDATE trading212_demo_order_books SET sequence=?1,projection=?2 WHERE workspace_id=?3 AND connection_id=?4 AND sequence=?5",
            params![next_sequence, serde_json::to_string(&book).map_err(storage_error)?, workspace_id, input.connection_id, sequence],
        ).map_err(storage_error)?;
        if changed != 1 {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        write_trading212_demo_order_book_event(&tx, &book, next_sequence)?;
        tx.commit().map_err(storage_error)?;
        Ok(book)
    }

    pub fn begin_alpaca_paper_order_cancel(
        &mut self,
        input: &AlpacaPaperOrderCancel,
    ) -> Result<(AlpacaPaperOrderBook, AlpacaPaperOrder, bool)> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id
            || !input.confirmed
            || !valid_order_text(&input.expected_connection_state_version, 256)
            || !valid_order_text(&input.expected_book_state_version, 256)
            || !valid_order_text(&input.idempotency_key, 128)
            || !provider_order_id_valid(&input.provider_order_id)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let account_projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&input.connection_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection = serde_json::from_str(&account_projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.state_version != input.expected_connection_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if account.provider_id != "alpaca" || account.environment != "PAPER" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        if account.connection_state != ConnectionState::Connected
            || account.health.credential != "CONFIGURED"
            || account.health.authentication != "VALID"
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let mut book = load_alpaca_paper_order_book(&tx, &workspace_id, &input.connection_id)?
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        if book.state_version != input.expected_book_state_version
            || book.remote_account_id
                != account
                    .data
                    .as_ref()
                    .map(|data| data.remote_account_id.as_str())
                    .unwrap_or_default()
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let index = book
            .orders
            .iter()
            .position(|order| order.provider_order_id == input.provider_order_id)
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let reviewed = book.orders[index].clone();
        if matches!(
            reviewed.cancel_state,
            AlpacaPaperCancelState::Submitting | AlpacaPaperCancelState::Pending
        ) {
            tx.commit().map_err(storage_error)?;
            return Ok((book, reviewed, false));
        }
        if !matches!(
            reviewed.provider_status.as_str(),
            "new" | "accepted" | "pending_new" | "partially_filled"
        ) {
            return Err(TradeXError::new("ORDER_NOT_CANCELABLE"));
        }
        book.orders[index].cancel_state = AlpacaPaperCancelState::Submitting;
        book.orders[index].cancel_idempotency_key = Some(input.idempotency_key.clone());
        book.orders[index].cancel_error = None;
        let sequence: i64 = tx.query_row(
            "SELECT sequence FROM alpaca_paper_order_books WHERE workspace_id=?1 AND connection_id=?2",
            params![workspace_id, input.connection_id],
            |row| row.get(0),
        ).map_err(storage_error)?;
        let next_sequence = sequence
            .checked_add(1)
            .filter(|next| *next <= MAX_SEQUENCE as i64)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        book.state_version = alpaca_order_book_version(&input.connection_id, next_sequence as u64);
        book.observed_at = timestamp()?;
        let changed = tx.execute(
            "UPDATE alpaca_paper_order_books SET sequence=?1,projection=?2 WHERE workspace_id=?3 AND connection_id=?4 AND sequence=?5",
            params![next_sequence, serde_json::to_string(&book).map_err(storage_error)?, workspace_id, input.connection_id, sequence],
        ).map_err(storage_error)?;
        if changed != 1 {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        write_alpaca_order_book_event(&tx, &book, next_sequence)?;
        tx.commit().map_err(storage_error)?;
        Ok((book, reviewed, true))
    }

    pub fn complete_binance_testnet_order_book(
        &mut self,
        book: BinanceTestnetOrderBook,
    ) -> Result<(BinanceTestnetOrderBook, DomainEvent)> {
        let workspace_id = self.workspace_id()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let result = complete_binance_testnet_order_book_in_tx(&tx, &workspace_id, book)?;
        tx.commit().map_err(storage_error)?;
        Ok(result)
    }

    pub fn complete_binance_testnet_order_cancel(
        &mut self,
        mut book: BinanceTestnetOrderBook,
        symbol: &str,
        provider_order_id: &str,
        idempotency_key: &str,
    ) -> Result<(BinanceTestnetOrderBook, DomainEvent)> {
        let workspace_id = self.workspace_id()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let mut latest = load_binance_testnet_order_book(&tx, &workspace_id, &book.connection_id)?
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let latest_order = latest
            .orders
            .iter()
            .find(|order| order.symbol == symbol && order.provider_order_id == provider_order_id)
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        if latest_order.pending
            && latest_order.cancel_idempotency_key.as_deref() != Some(idempotency_key)
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if latest.state_version != book.state_version {
            crate::provider_io::merge_binance_testnet_cancel_observation(
                &mut latest,
                &book,
                symbol,
                provider_order_id,
            )?;
            book = latest;
        }
        let result = complete_binance_testnet_order_book_in_tx(&tx, &workspace_id, book)?;
        tx.commit().map_err(storage_error)?;
        Ok(result)
    }

    pub fn save_binance_private_stream_state(
        &mut self,
        connection_id: &str,
        expected_state_version: &str,
        book: Option<BinanceTestnetOrderBook>,
        private_stream: &str,
        reconciliation: &str,
        reason: &str,
        last_event_at: Option<&str>,
    ) -> Result<(
        Option<BinanceTestnetOrderBook>,
        Option<DomainEvent>,
        DomainEvent,
    )> {
        if !matches!(
            private_stream,
            "CONNECTING" | "CONNECTED" | "DEGRADED" | "AUTH_FAILED" | "STOPPED"
        ) || !matches!(
            reconciliation,
            "REQUIRED" | "RUNNING" | "CURRENT" | "DEGRADED"
        ) || !valid_order_text(reason, 256)
            || last_event_at.is_some_and(|value| !valid_provider_time(value))
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let workspace_id = self.workspace_id()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (sequence, projection): (i64, String) = tx
            .query_row(
                "SELECT sequence,projection FROM accounts WHERE connection_id=?1",
                [connection_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        if sequence < 1 || sequence >= MAX_SEQUENCE as i64 {
            return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
        }
        let mut account: AccountConnection = serde_json::from_str(&projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.connection_id != connection_id
            || account.state_version != expected_state_version
            || account.provider_id != "binance"
            || account.environment != "TESTNET"
            || account.connection_state != ConnectionState::Connected
            || account.data.is_none()
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let completed_book = if let Some(book) = book {
            Some(complete_binance_testnet_order_book_in_tx(
                &tx,
                &workspace_id,
                book,
            )?)
        } else {
            None
        };
        let next_sequence = sequence + 1;
        account.health.private_stream = private_stream.into();
        account.health.reconciliation = reconciliation.into();
        account.health.reason = reason.into();
        if let Some(last_event_at) = last_event_at {
            account.last_private_stream_event_at = Some(last_event_at.into());
        }
        account.updated_at = timestamp()?;
        let account_event = DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: "account.health.changed".into(),
            schema_version: 1,
            occurred_at: account.updated_at.clone(),
            aggregate_type: "account".into(),
            aggregate_id: connection_id.into(),
            sequence: next_sequence as u64,
            payload: DomainProjection::Account(Box::new(account.clone())),
        };
        let changed = tx
            .execute(
                "UPDATE accounts SET sequence=?1,projection=?2 WHERE connection_id=?3 AND sequence=?4",
                params![
                    next_sequence,
                    serde_json::to_string(&account).map_err(storage_error)?,
                    connection_id,
                    sequence,
                ],
            )
            .map_err(storage_error)?;
        if changed != 1 {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        tx.execute(
            "INSERT INTO outbox VALUES('account',?1,?2,?3,?4)",
            params![
                connection_id,
                next_sequence,
                account_event.event_id,
                serde_json::to_string(&account_event).map_err(storage_error)?,
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(match completed_book {
            Some((book, event)) => (Some(book), Some(event), account_event),
            None => (None, None, account_event),
        })
    }

    pub fn complete_alpaca_paper_order_book(
        &mut self,
        mut book: AlpacaPaperOrderBook,
    ) -> Result<(AlpacaPaperOrderBook, DomainEvent)> {
        let workspace_id = self.workspace_id()?;
        if book.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let account_projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&book.connection_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection = serde_json::from_str(&account_projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.provider_id != "alpaca"
            || account.environment != "PAPER"
            || account
                .data
                .as_ref()
                .map(|data| data.remote_account_id.as_str())
                != Some(book.remote_account_id.as_str())
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let row: Option<(i64, String)> = tx.query_row(
            "SELECT sequence,projection FROM alpaca_paper_order_books WHERE workspace_id=?1 AND connection_id=?2",
            params![workspace_id, book.connection_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional().map_err(storage_error)?;
        let sequence = if let Some((sequence, projection)) = row {
            let current: AlpacaPaperOrderBook = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if sequence < 1
                || current.state_version
                    != alpaca_order_book_version(&book.connection_id, sequence as u64)
                || book.state_version != current.state_version
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            sequence
        } else {
            if book.state_version != alpaca_order_book_version(&book.connection_id, 0) {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            0
        };
        let mut attempts = tx.prepare(
            "SELECT client_order_id FROM alpaca_paper_order_attempts WHERE workspace_id=?1 AND connection_id=?2",
        ).map_err(storage_error)?;
        let known_orders = attempts
            .query_map(params![workspace_id, book.connection_id], |row| {
                row.get::<_, String>(0)
            })
            .map_err(storage_error)?
            .collect::<std::result::Result<std::collections::HashSet<_>, _>>()
            .map_err(storage_error)?;
        drop(attempts);
        for order in &mut book.orders {
            order.origin = if known_orders.contains(&order.client_order_id) {
                AlpacaPaperOrderOrigin::TradeX
            } else {
                AlpacaPaperOrderOrigin::External
            };
        }
        validate_alpaca_order_book(&book)?;
        let next_sequence = sequence
            .checked_add(1)
            .filter(|next| *next <= MAX_SEQUENCE as i64)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        book.state_version = alpaca_order_book_version(&book.connection_id, next_sequence as u64);
        let encoded = serde_json::to_string(&book).map_err(storage_error)?;
        tx.execute(
            "INSERT INTO alpaca_paper_order_books(workspace_id,connection_id,sequence,projection) VALUES(?1,?2,?3,?4) ON CONFLICT(workspace_id,connection_id) DO UPDATE SET sequence=excluded.sequence,projection=excluded.projection",
            params![workspace_id, book.connection_id, next_sequence, encoded],
        ).map_err(storage_error)?;
        let event = write_alpaca_order_book_event(&tx, &book, next_sequence)?;
        tx.commit().map_err(storage_error)?;
        Ok((book, event))
    }

    pub fn complete_trading212_demo_order_book(
        &mut self,
        mut book: Trading212DemoOrderBook,
    ) -> Result<(Trading212DemoOrderBook, DomainEvent)> {
        let workspace_id = self.workspace_id()?;
        if book.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let account_projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&book.connection_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection = serde_json::from_str(&account_projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.provider_id != "trading212"
            || account.environment != "DEMO"
            || book.environment != "DEMO"
            || account
                .data
                .as_ref()
                .map(|data| data.remote_account_id.as_str())
                != Some(book.remote_account_id.as_str())
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let row: Option<(i64, String)> = tx
            .query_row(
                "SELECT sequence,projection FROM trading212_demo_order_books WHERE workspace_id=?1 AND connection_id=?2",
                params![workspace_id, book.connection_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(storage_error)?;
        let sequence = if let Some((sequence, projection)) = row {
            let current: Trading212DemoOrderBook = serde_json::from_str(&projection)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if sequence < 1
                || current.state_version
                    != trading212_demo_order_book_version(&book.connection_id, sequence as u64)
                || book.state_version != current.state_version
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            sequence
        } else {
            if book.state_version != trading212_demo_order_book_version(&book.connection_id, 0) {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            0
        };
        let mut attempts = tx
            .prepare(
                "SELECT projection FROM trading212_demo_order_attempts WHERE workspace_id=?1 AND connection_id=?2",
            )
            .map_err(storage_error)?;
        let attempt_projections = attempts
            .query_map(params![workspace_id, book.connection_id], |row| {
                row.get::<_, String>(0)
            })
            .map_err(storage_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        drop(attempts);
        let attempts = attempt_projections
            .iter()
            .map(|projection| {
                serde_json::from_str::<Trading212DemoOrderAttempt>(projection)
                    .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))
            })
            .collect::<Result<Vec<_>>>()?;
        for order in &mut book.orders {
            let mut matching = attempts.iter().filter(|attempt| {
                attempt.connection_id == book.connection_id
                    && attempt.remote_account_id == book.remote_account_id
                    && attempt.provider_order_id.as_deref()
                        == Some(order.provider_order_id.as_str())
            });
            if let Some(attempt) = matching.next() {
                if matching.next().is_some() {
                    return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
                }
                order.origin = Trading212DemoOrderOrigin::TradeX;
                order.attempt_id = Some(attempt.attempt_id.clone());
            } else {
                order.origin = Trading212DemoOrderOrigin::External;
                order.attempt_id = None;
            }
        }
        validate_trading212_demo_order_book(&book)?;
        let next_sequence = sequence
            .checked_add(1)
            .filter(|next| *next <= MAX_SEQUENCE as i64)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        book.state_version =
            trading212_demo_order_book_version(&book.connection_id, next_sequence as u64);
        let encoded = serde_json::to_string(&book).map_err(storage_error)?;
        tx.execute(
            "INSERT INTO trading212_demo_order_books(workspace_id,connection_id,sequence,projection) VALUES(?1,?2,?3,?4) ON CONFLICT(workspace_id,connection_id) DO UPDATE SET sequence=excluded.sequence,projection=excluded.projection",
            params![workspace_id, book.connection_id, next_sequence, encoded],
        )
        .map_err(storage_error)?;
        let event = write_trading212_demo_order_book_event(&tx, &book, next_sequence)?;
        tx.commit().map_err(storage_error)?;
        Ok((book, event))
    }

    pub fn begin_trading212_demo_order_attempt(
        &mut self,
        input: &Trading212DemoOrderSubmit,
    ) -> Result<(Trading212DemoOrderAttempt, bool)> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(&input.proposal_id)?;
        if !input.confirmed_demo_order
            || !valid_order_text(&input.expected_connection_state_version, 256)
            || !valid_order_text(&input.expected_proposal_state_version, 256)
            || !valid_order_text(&input.idempotency_key, 128)
            || !valid_proposal_hash(&input.proposal_hash)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if let Some(existing) =
            load_trading212_demo_attempt(&tx, &workspace_id, &input.proposal_id)?
        {
            if existing.connection_id != input.connection_id
                || existing.proposal_hash != input.proposal_hash
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            tx.commit().map_err(storage_error)?;
            return Ok((existing, false));
        }
        let account_projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&input.connection_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection = serde_json::from_str(&account_projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.state_version != input.expected_connection_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if account.provider_id != "trading212" || account.environment != "DEMO" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        if account.connection_state != ConnectionState::Connected
            || account.health.connection != "ONLINE"
            || account.health.authentication != "VALID"
            || account.health.credential != "CONFIGURED"
            || account.blocked_permissions()
            || (account.permissions.scope == "UNVERIFIED" && !account.permissions.acknowledged)
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let remote_account_id = account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.clone())
            .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
        let (row_workspace_id, draft_id, draft_version, proposal_hash, sequence, projection): (
            String,
            String,
            i64,
            String,
            i64,
            String,
        ) = tx
            .query_row(
                "SELECT workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE workspace_id=?1 AND proposal_id=?2",
                params![workspace_id, input.proposal_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("ORDER_PROPOSAL_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let stored = decode_stored_order_proposal(
            &projection,
            &input.proposal_id,
            &row_workspace_id,
            &draft_id,
            draft_version,
            &proposal_hash,
            sequence,
            &workspace_id,
        )?;
        let proposal = materialize_order_proposal(&tx, stored)?;
        if proposal.proposal_hash != input.proposal_hash
            || proposal.state_version != input.expected_proposal_state_version
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if proposal.status != OrderProposalStatus::NeedsApproval
            || proposal.fields.environment != ExecutionContext::Trading212Demo
            || proposal.fields.account_id.as_deref() != Some(input.connection_id.as_str())
        {
            return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
        }
        crate::provider_io::validate_trading212_demo_proposal(&proposal, &input.connection_id)?;
        let unresolved: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM trading212_demo_order_attempts WHERE connection_id=?1 AND state IN ('SUBMITTING','UNKNOWN_RECONCILING'))",
                [&input.connection_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if unresolved {
            return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
        }
        let proposal_event_sequence = proposal.history.len() as i64;
        let consumed_sequence = proposal_event_sequence
            .checked_add(1)
            .filter(|value| *value <= 32)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        let now = timestamp()?;
        let attempt_id = Uuid::new_v4().to_string();
        let attempt = Trading212DemoOrderAttempt {
            attempt_id: attempt_id.clone(),
            workspace_id: workspace_id.clone(),
            connection_id: input.connection_id.clone(),
            remote_account_id,
            proposal_id: input.proposal_id.clone(),
            proposal_hash: input.proposal_hash.clone(),
            state: Trading212DemoOrderAttemptState::Submitting,
            provider_order_id: None,
            provider_status: None,
            error_code: None,
            reason: "Submitting to Trading 212 Demo; provider status is pending.".into(),
            state_version: trading212_demo_attempt_version(&attempt_id, 1),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        tx.execute(
            "INSERT INTO order_proposal_events(proposal_id,workspace_id,sequence,event,reason,occurred_at) VALUES(?1,?2,?3,'CONSUMED',?4,?5)",
            params![input.proposal_id, workspace_id, consumed_sequence, "Trading 212 Demo submission attempt persisted.", now],
        )
        .map_err(storage_error)?;
        tx.execute(
            "INSERT INTO trading212_demo_order_attempts(workspace_id,attempt_id,connection_id,proposal_id,idempotency_key,sequence,state,projection) VALUES(?1,?2,?3,?4,?5,1,'SUBMITTING',?6)",
            params![workspace_id, attempt_id, input.connection_id, input.proposal_id, input.idempotency_key, serde_json::to_string(&attempt).map_err(storage_error)?],
        )
        .map_err(|error| {
            if error.to_string().contains("UNIQUE") {
                TradeXError::new("ORDER_PROPOSAL_CONSUMED")
            } else {
                storage_error(error)
            }
        })?;
        write_trading212_demo_attempt_event(&tx, &attempt, 1)?;
        tx.commit().map_err(storage_error)?;
        Ok((attempt, true))
    }

    pub fn complete_trading212_demo_order_attempt(
        &mut self,
        result: &Trading212DemoOrderAttempt,
    ) -> Result<Trading212DemoOrderAttempt> {
        let workspace_id = self.workspace_id()?;
        if result.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let current = load_trading212_demo_attempt(&tx, &workspace_id, &result.proposal_id)?
            .ok_or_else(|| TradeXError::new("ORDER_ATTEMPT_NOT_FOUND"))?;
        if current.attempt_id != result.attempt_id
            || current.connection_id != result.connection_id
            || current.remote_account_id != result.remote_account_id
            || current.proposal_hash != result.proposal_hash
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if current.state != Trading212DemoOrderAttemptState::Submitting
            || !matches!(
                result.state,
                Trading212DemoOrderAttemptState::Acknowledged
                    | Trading212DemoOrderAttemptState::UnknownReconciling
                    | Trading212DemoOrderAttemptState::Rejected
            )
        {
            return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
        }
        let sequence = current
            .state_version
            .rsplit_once(':')
            .and_then(|(_, value)| value.parse::<i64>().ok())
            .ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        let next = sequence
            .checked_add(1)
            .filter(|value| *value <= MAX_SEQUENCE as i64)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        let mut attempt = result.clone();
        attempt.state_version = trading212_demo_attempt_version(&attempt.attempt_id, next as u64);
        attempt.updated_at = timestamp()?;
        validate_trading212_demo_attempt(&attempt)?;
        let encoded = serde_json::to_string(&attempt).map_err(storage_error)?;
        tx.execute(
            "UPDATE trading212_demo_order_attempts SET sequence=?1,state=?2,projection=?3 WHERE workspace_id=?4 AND attempt_id=?5 AND sequence=?6 AND state='SUBMITTING'",
            params![next, trading212_demo_attempt_state_name(attempt.state), encoded, workspace_id, attempt.attempt_id, sequence],
        )
        .map_err(storage_error)?;
        write_trading212_demo_attempt_event(&tx, &attempt, next)?;
        tx.commit().map_err(storage_error)?;
        Ok(attempt)
    }

    pub fn begin_alpaca_paper_order_attempt(
        &mut self,
        input: &AlpacaPaperOrderSubmit,
    ) -> Result<(AlpacaPaperOrderAttempt, bool)> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(&input.proposal_id)?;
        if !input.confirmed_paper_order
            || !valid_order_text(&input.expected_connection_state_version, 256)
            || !valid_order_text(&input.expected_proposal_state_version, 256)
            || !valid_order_text(&input.idempotency_key, 128)
            || !valid_proposal_hash(&input.proposal_hash)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if let Some(existing) = load_alpaca_paper_attempt(&tx, &workspace_id, &input.proposal_id)? {
            if existing.connection_id != input.connection_id
                || existing.proposal_hash != input.proposal_hash
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            tx.commit().map_err(storage_error)?;
            return Ok((existing, false));
        }
        let account_projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&input.connection_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection = serde_json::from_str(&account_projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.state_version != input.expected_connection_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if account.provider_id != "alpaca" || account.environment != "PAPER" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        if account.connection_state != ConnectionState::Connected
            || account.health.connection != "ONLINE"
            || account.health.authentication != "VALID"
            || account.health.credential != "CONFIGURED"
            || account.blocked_permissions()
            || (account.permissions.scope == "UNVERIFIED" && !account.permissions.acknowledged)
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let remote_account_id = account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.clone())
            .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
        let (row_workspace_id, draft_id, draft_version, proposal_hash, sequence, projection): (
            String, String, i64, String, i64, String
        ) = tx.query_row(
            "SELECT workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE workspace_id=?1 AND proposal_id=?2",
            params![workspace_id, input.proposal_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        ).map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
            TradeXError::new("ORDER_PROPOSAL_NOT_FOUND")
        } else { storage_error(error) })?;
        let stored = decode_stored_order_proposal(
            &projection,
            &input.proposal_id,
            &row_workspace_id,
            &draft_id,
            draft_version,
            &proposal_hash,
            sequence,
            &workspace_id,
        )?;
        let proposal = materialize_order_proposal(&tx, stored)?;
        if proposal.proposal_hash != input.proposal_hash
            || proposal.state_version != input.expected_proposal_state_version
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if proposal.status != OrderProposalStatus::NeedsApproval
            || proposal.fields.environment != ExecutionContext::AlpacaPaper
            || proposal.fields.account_id.as_deref() != Some(input.connection_id.as_str())
        {
            return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
        }
        let unresolved: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM alpaca_paper_order_attempts WHERE connection_id=?1 AND state IN ('SUBMITTING','UNKNOWN_RECONCILING'))",
            [&input.connection_id],
            |row| row.get(0),
        ).map_err(storage_error)?;
        if unresolved {
            return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
        }
        let proposal_event_sequence = proposal.history.len() as i64;
        let consumed_sequence = proposal_event_sequence
            .checked_add(1)
            .filter(|value| *value <= 32)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        let now = timestamp()?;
        let attempt_id = Uuid::new_v4().to_string();
        let attempt = AlpacaPaperOrderAttempt {
            attempt_id: attempt_id.clone(),
            workspace_id: workspace_id.clone(),
            connection_id: input.connection_id.clone(),
            remote_account_id,
            proposal_id: input.proposal_id.clone(),
            proposal_hash: input.proposal_hash.clone(),
            client_order_id: format!("tradex-{attempt_id}"),
            state: AlpacaPaperOrderAttemptState::Submitting,
            provider_order_id: None,
            provider_status: None,
            error_code: None,
            reason: "Submitting to Alpaca Paper; broker status is pending.".into(),
            state_version: format!("alpaca-paper-attempt:{attempt_id}:1"),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        tx.execute(
            "INSERT INTO order_proposal_events(proposal_id,workspace_id,sequence,event,reason,occurred_at) VALUES(?1,?2,?3,'CONSUMED',?4,?5)",
            params![input.proposal_id, workspace_id, consumed_sequence, "Alpaca Paper submission attempt persisted.", now],
        ).map_err(storage_error)?;
        tx.execute(
            "INSERT INTO alpaca_paper_order_attempts(workspace_id,attempt_id,connection_id,proposal_id,client_order_id,idempotency_key,sequence,state,projection) VALUES(?1,?2,?3,?4,?5,?6,1,'SUBMITTING',?7)",
            params![workspace_id, attempt_id, input.connection_id, input.proposal_id, attempt.client_order_id, input.idempotency_key, serde_json::to_string(&attempt).map_err(storage_error)?],
        ).map_err(|error| if error.to_string().contains("UNIQUE") {
            TradeXError::new("ORDER_PROPOSAL_CONSUMED")
        } else { storage_error(error) })?;
        write_alpaca_paper_attempt_event(&tx, &attempt, 1)?;
        tx.commit().map_err(storage_error)?;
        Ok((attempt, true))
    }

    pub fn begin_binance_testnet_order_attempt(
        &mut self,
        input: &BinanceTestnetOrderSubmit,
    ) -> Result<(BinanceTestnetOrderAttempt, bool)> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(&input.proposal_id)?;
        if !input.confirmed_testnet_order
            || !valid_order_text(&input.expected_connection_state_version, 256)
            || !valid_order_text(&input.expected_proposal_state_version, 256)
            || !valid_order_text(&input.idempotency_key, 128)
            || !valid_proposal_hash(&input.proposal_hash)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if let Some(existing) =
            load_binance_testnet_attempt(&tx, &workspace_id, &input.proposal_id)?
        {
            if existing.connection_id != input.connection_id
                || existing.proposal_hash != input.proposal_hash
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            tx.commit().map_err(storage_error)?;
            return Ok((existing, false));
        }
        let account_projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&input.connection_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection = serde_json::from_str(&account_projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.state_version != input.expected_connection_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if account.provider_id != "binance" || account.environment != "TESTNET" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        if account.connection_state != ConnectionState::Connected
            || account.health.connection != "ONLINE"
            || account.health.authentication != "VALID"
            || account.health.credential != "CONFIGURED"
            || account.blocked_permissions()
            || (account.permissions.scope == "UNVERIFIED" && !account.permissions.acknowledged)
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let remote_account_id = account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.clone())
            .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
        let (row_workspace_id, draft_id, draft_version, proposal_hash, sequence, projection): (
            String, String, i64, String, i64, String
        ) = tx.query_row(
            "SELECT workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE workspace_id=?1 AND proposal_id=?2",
            params![workspace_id, input.proposal_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        ).map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
            TradeXError::new("ORDER_PROPOSAL_NOT_FOUND")
        } else { storage_error(error) })?;
        let stored = decode_stored_order_proposal(
            &projection,
            &input.proposal_id,
            &row_workspace_id,
            &draft_id,
            draft_version,
            &proposal_hash,
            sequence,
            &workspace_id,
        )?;
        let proposal = materialize_order_proposal(&tx, stored)?;
        if proposal.proposal_hash != input.proposal_hash
            || proposal.state_version != input.expected_proposal_state_version
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if proposal.status != OrderProposalStatus::NeedsApproval
            || proposal.fields.environment != ExecutionContext::BinanceTestnet
            || proposal.fields.account_id.as_deref() != Some(input.connection_id.as_str())
        {
            return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
        }
        crate::provider_io::validate_binance_testnet_proposal(&proposal, &input.connection_id)?;
        let unresolved: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM binance_testnet_order_attempts WHERE connection_id=?1 AND state IN ('SUBMITTING','UNKNOWN_RECONCILING'))",
            [&input.connection_id],
            |row| row.get(0),
        ).map_err(storage_error)?;
        if unresolved {
            return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
        }
        let consumed_sequence = (proposal.history.len() as i64)
            .checked_add(1)
            .filter(|value| *value <= 32)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        let now = timestamp()?;
        let attempt_id = Uuid::new_v4().to_string();
        let attempt = BinanceTestnetOrderAttempt {
            attempt_id: attempt_id.clone(),
            workspace_id: workspace_id.clone(),
            connection_id: input.connection_id.clone(),
            remote_account_id,
            proposal_id: input.proposal_id.clone(),
            proposal_hash: input.proposal_hash.clone(),
            environment: "TESTNET".into(),
            client_order_id: format!(
                "tx-{}",
                Uuid::parse_str(&attempt_id)
                    .map_err(storage_error)?
                    .simple()
            ),
            state: BinanceTestnetOrderAttemptState::Submitting,
            provider_order_id: None,
            provider_status: None,
            error_code: None,
            reason: "Submitting to Binance Spot Testnet; provider status is pending.".into(),
            state_version: format!("binance-testnet-attempt:{attempt_id}:1"),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        tx.execute(
            "INSERT INTO order_proposal_events(proposal_id,workspace_id,sequence,event,reason,occurred_at) VALUES(?1,?2,?3,'CONSUMED',?4,?5)",
            params![input.proposal_id, workspace_id, consumed_sequence, "Binance Testnet submission attempt persisted.", now],
        ).map_err(storage_error)?;
        tx.execute(
            "INSERT INTO binance_testnet_order_attempts(workspace_id,attempt_id,connection_id,proposal_id,client_order_id,idempotency_key,sequence,state,projection) VALUES(?1,?2,?3,?4,?5,?6,1,'SUBMITTING',?7)",
            params![workspace_id, attempt_id, input.connection_id, input.proposal_id, attempt.client_order_id, input.idempotency_key, serde_json::to_string(&attempt).map_err(storage_error)?],
        ).map_err(|error| if error.to_string().contains("UNIQUE") {
            TradeXError::new("ORDER_PROPOSAL_CONSUMED")
        } else { storage_error(error) })?;
        write_binance_testnet_attempt_event(&tx, &attempt, 1)?;
        tx.commit().map_err(storage_error)?;
        Ok((attempt, true))
    }

    pub fn complete_binance_testnet_order_attempt(
        &mut self,
        result: &BinanceTestnetOrderAttempt,
    ) -> Result<BinanceTestnetOrderAttempt> {
        let workspace_id = self.workspace_id()?;
        if result.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (sequence, projection): (i64, String) = tx.query_row(
            "SELECT sequence,projection FROM binance_testnet_order_attempts WHERE workspace_id=?1 AND attempt_id=?2 AND connection_id=?3 AND proposal_id=?4",
            params![workspace_id, result.attempt_id, result.connection_id, result.proposal_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
            TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
        } else { storage_error(error) })?;
        let current: BinanceTestnetOrderAttempt = serde_json::from_str(&projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        if current.attempt_id != result.attempt_id
            || current.client_order_id != result.client_order_id
            || current.proposal_hash != result.proposal_hash
            || current.remote_account_id != result.remote_account_id
            || current.environment != "TESTNET"
            || result.environment != "TESTNET"
            || sequence < 1
            || sequence >= MAX_SEQUENCE as i64
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        if current.state == BinanceTestnetOrderAttemptState::Acknowledged
            && result.state != BinanceTestnetOrderAttemptState::Acknowledged
        {
            tx.commit().map_err(storage_error)?;
            return Ok(current);
        }
        if !matches!(
            (current.state, result.state),
            (
                BinanceTestnetOrderAttemptState::Submitting,
                BinanceTestnetOrderAttemptState::Acknowledged
                    | BinanceTestnetOrderAttemptState::UnknownReconciling
                    | BinanceTestnetOrderAttemptState::Rejected
            ) | (
                BinanceTestnetOrderAttemptState::UnknownReconciling,
                BinanceTestnetOrderAttemptState::Acknowledged
                    | BinanceTestnetOrderAttemptState::UnknownReconciling
            )
        ) {
            return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
        }
        let mut next = result.clone();
        next.updated_at = timestamp()?;
        next.state_version = format!(
            "binance-testnet-attempt:{}:{}",
            next.attempt_id,
            sequence + 1
        );
        if next.reason.is_empty()
            || next.reason.len() > 256
            || next.reason.chars().any(char::is_control)
            || next
                .error_code
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 64))
            || next
                .provider_status
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 64))
            || next.provider_order_id.as_deref().is_some_and(|value| {
                value.is_empty()
                    || value.len() > 20
                    || !value.bytes().all(|byte| byte.is_ascii_digit())
            })
        {
            return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
        }
        let encoded = serde_json::to_string(&next).map_err(storage_error)?;
        tx.execute(
            "UPDATE binance_testnet_order_attempts SET sequence=?1,state=?2,projection=?3 WHERE workspace_id=?4 AND attempt_id=?5 AND sequence=?6",
            params![sequence + 1, binance_attempt_state_name(next.state), encoded, workspace_id, next.attempt_id, sequence],
        ).map_err(storage_error)?;
        write_binance_testnet_attempt_event(&tx, &next, sequence + 1)?;
        tx.commit().map_err(storage_error)?;
        Ok(next)
    }

    pub fn begin_bitget_demo_order_attempt(
        &mut self,
        input: &BitgetDemoOrderSubmit,
    ) -> Result<(BitgetDemoOrderAttempt, bool)> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(&input.proposal_id)?;
        if !input.confirmed_demo_order
            || !valid_order_text(&input.expected_connection_state_version, 256)
            || !valid_order_text(&input.expected_proposal_state_version, 256)
            || !valid_order_text(&input.idempotency_key, 128)
            || !valid_proposal_hash(&input.proposal_hash)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if let Some(existing) = load_bitget_demo_attempt(&tx, &workspace_id, &input.proposal_id)? {
            if existing.connection_id != input.connection_id
                || existing.proposal_hash != input.proposal_hash
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            tx.commit().map_err(storage_error)?;
            return Ok((existing, false));
        }
        let account_projection: String = tx
            .query_row(
                "SELECT projection FROM accounts WHERE connection_id=?1",
                [&input.connection_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let account: AccountConnection = serde_json::from_str(&account_projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        account.validate_persisted(&workspace_id)?;
        if account.state_version != input.expected_connection_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if account.provider_id != "bitget" || account.environment != "DEMO" {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        if account.connection_state != ConnectionState::Connected
            || account.health.connection != "ONLINE"
            || account.health.authentication != "VALID"
            || account.health.credential != "CONFIGURED"
            || account.blocked_permissions()
            || (account.permissions.scope == "UNVERIFIED" && !account.permissions.acknowledged)
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let remote_account_id = account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.clone())
            .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
        let (row_workspace_id, draft_id, draft_version, proposal_hash, sequence, projection): (
            String,
            String,
            i64,
            String,
            i64,
            String,
        ) = tx.query_row(
            "SELECT workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE workspace_id=?1 AND proposal_id=?2",
            params![workspace_id, input.proposal_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        ).map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
            TradeXError::new("ORDER_PROPOSAL_NOT_FOUND")
        } else { storage_error(error) })?;
        let stored = decode_stored_order_proposal(
            &projection,
            &input.proposal_id,
            &row_workspace_id,
            &draft_id,
            draft_version,
            &proposal_hash,
            sequence,
            &workspace_id,
        )?;
        let proposal = materialize_order_proposal(&tx, stored)?;
        if proposal.proposal_hash != input.proposal_hash
            || proposal.state_version != input.expected_proposal_state_version
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if proposal.status != OrderProposalStatus::NeedsApproval
            || proposal.fields.environment != ExecutionContext::BitgetDemo
            || proposal.fields.account_id.as_deref() != Some(input.connection_id.as_str())
        {
            return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
        }
        crate::provider_io::validate_bitget_demo_proposal(&proposal, &input.connection_id)?;
        let unresolved: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM bitget_demo_order_attempts WHERE connection_id=?1 AND state IN ('SUBMITTING','UNKNOWN_RECONCILING'))",
                [&input.connection_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if unresolved {
            return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
        }
        let consumed_sequence = (proposal.history.len() as i64)
            .checked_add(1)
            .filter(|value| *value <= 32)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        let now = timestamp()?;
        let attempt_id = Uuid::new_v4().to_string();
        let client_oid = format!(
            "tx-{}",
            Uuid::parse_str(&attempt_id)
                .map_err(storage_error)?
                .simple()
        );
        let attempt = BitgetDemoOrderAttempt {
            attempt_id: attempt_id.clone(),
            workspace_id: workspace_id.clone(),
            connection_id: input.connection_id.clone(),
            remote_account_id,
            proposal_id: input.proposal_id.clone(),
            proposal_hash: input.proposal_hash.clone(),
            environment: "DEMO".into(),
            client_oid,
            state: BitgetDemoOrderAttemptState::Submitting,
            provider_order_id: None,
            provider_status: None,
            error_code: None,
            reason: "Submitting to Bitget Spot Demo; provider status is pending.".into(),
            state_version: format!("bitget-demo-order-attempt:{attempt_id}:1"),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        tx.execute(
            "INSERT INTO order_proposal_events(proposal_id,workspace_id,sequence,event,reason,occurred_at) VALUES(?1,?2,?3,'CONSUMED',?4,?5)",
            params![input.proposal_id, workspace_id, consumed_sequence, "Bitget Demo submission attempt persisted.", now],
        ).map_err(storage_error)?;
        tx.execute(
            "INSERT INTO bitget_demo_order_attempts(workspace_id,attempt_id,connection_id,proposal_id,client_oid,idempotency_key,sequence,state,projection) VALUES(?1,?2,?3,?4,?5,?6,1,'SUBMITTING',?7)",
            params![workspace_id, attempt_id, input.connection_id, input.proposal_id, attempt.client_oid, input.idempotency_key, serde_json::to_string(&attempt).map_err(storage_error)?],
        ).map_err(|error| if error.to_string().contains("UNIQUE") {
            TradeXError::new("ORDER_PROPOSAL_CONSUMED")
        } else { storage_error(error) })?;
        write_bitget_demo_attempt_event(&tx, &attempt, 1)?;
        tx.commit().map_err(storage_error)?;
        Ok((attempt, true))
    }

    pub fn complete_bitget_demo_order_attempt(
        &mut self,
        result: &BitgetDemoOrderAttempt,
    ) -> Result<BitgetDemoOrderAttempt> {
        let workspace_id = self.workspace_id()?;
        if result.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (sequence, projection): (i64, String) = tx.query_row(
            "SELECT sequence,projection FROM bitget_demo_order_attempts WHERE workspace_id=?1 AND attempt_id=?2 AND connection_id=?3 AND proposal_id=?4",
            params![workspace_id, result.attempt_id, result.connection_id, result.proposal_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
            TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
        } else { storage_error(error) })?;
        let current: BitgetDemoOrderAttempt = serde_json::from_str(&projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        if current.attempt_id != result.attempt_id
            || current.client_oid != result.client_oid
            || current.proposal_hash != result.proposal_hash
            || current.remote_account_id != result.remote_account_id
            || current.environment != "DEMO"
            || result.environment != "DEMO"
            || sequence < 1
            || sequence >= MAX_SEQUENCE as i64
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        if current.state == BitgetDemoOrderAttemptState::Acknowledged
            && result.state != BitgetDemoOrderAttemptState::Acknowledged
        {
            tx.commit().map_err(storage_error)?;
            return Ok(current);
        }
        if !matches!(
            (current.state, result.state),
            (
                BitgetDemoOrderAttemptState::Submitting,
                BitgetDemoOrderAttemptState::Acknowledged
                    | BitgetDemoOrderAttemptState::UnknownReconciling
                    | BitgetDemoOrderAttemptState::Rejected
            ) | (
                BitgetDemoOrderAttemptState::UnknownReconciling,
                BitgetDemoOrderAttemptState::Acknowledged
                    | BitgetDemoOrderAttemptState::UnknownReconciling
            )
        ) {
            return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
        }
        let mut next = result.clone();
        next.updated_at = timestamp()?;
        next.state_version = format!(
            "bitget-demo-order-attempt:{}:{}",
            next.attempt_id,
            sequence + 1
        );
        validate_bitget_demo_attempt(&next)?;
        let encoded = serde_json::to_string(&next).map_err(storage_error)?;
        tx.execute(
            "UPDATE bitget_demo_order_attempts SET sequence=?1,state=?2,projection=?3 WHERE workspace_id=?4 AND attempt_id=?5 AND sequence=?6",
            params![sequence + 1, bitget_demo_attempt_state_name(next.state), encoded, workspace_id, next.attempt_id, sequence],
        ).map_err(storage_error)?;
        write_bitget_demo_attempt_event(&tx, &next, sequence + 1)?;
        tx.commit().map_err(storage_error)?;
        Ok(next)
    }

    pub fn complete_alpaca_paper_order_attempt(
        &mut self,
        result: &AlpacaPaperOrderAttempt,
    ) -> Result<AlpacaPaperOrderAttempt> {
        let workspace_id = self.workspace_id()?;
        if result.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (sequence, projection): (i64, String) = tx.query_row(
            "SELECT sequence,projection FROM alpaca_paper_order_attempts WHERE workspace_id=?1 AND attempt_id=?2 AND connection_id=?3 AND proposal_id=?4",
            params![workspace_id, result.attempt_id, result.connection_id, result.proposal_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|error| if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
            TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
        } else { storage_error(error) })?;
        let current: AlpacaPaperOrderAttempt = serde_json::from_str(&projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        if current.attempt_id != result.attempt_id
            || current.client_order_id != result.client_order_id
            || current.proposal_hash != result.proposal_hash
            || current.remote_account_id != result.remote_account_id
            || sequence < 1
            || sequence >= MAX_SEQUENCE as i64
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        let mut next = result.clone();
        if current.state == AlpacaPaperOrderAttemptState::Acknowledged
            && result.state != AlpacaPaperOrderAttemptState::Acknowledged
        {
            next = current;
        } else {
            next.updated_at = timestamp()?;
            next.state_version =
                format!("alpaca-paper-attempt:{}:{}", next.attempt_id, sequence + 1);
        }
        if next.reason.is_empty()
            || next.reason.len() > 256
            || next.reason.chars().any(char::is_control)
            || next
                .error_code
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 64))
            || next
                .provider_status
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 64))
            || next
                .provider_order_id
                .as_deref()
                .is_some_and(|value| !Uuid::parse_str(value).is_ok())
        {
            return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
        }
        let encoded = serde_json::to_string(&next).map_err(storage_error)?;
        tx.execute(
            "UPDATE alpaca_paper_order_attempts SET sequence=?1,state=?2,projection=?3 WHERE workspace_id=?4 AND attempt_id=?5 AND sequence=?6",
            params![sequence + 1, alpaca_attempt_state_name(next.state), encoded, workspace_id, next.attempt_id, sequence],
        ).map_err(storage_error)?;
        write_alpaca_paper_attempt_event(&tx, &next, sequence + 1)?;
        tx.commit().map_err(storage_error)?;
        Ok(next)
    }

    pub fn generate_order_proposal(
        &mut self,
        input: &OrderProposalGenerate,
        references: &OrderProposalReferences,
    ) -> Result<OrderProposal> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_draft_id(&input.draft_id)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let draft = load_order_draft_tx(&tx, &workspace_id, &input.draft_id)?;
        if draft.draft_version != input.expected_draft_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let (estimated_notional, estimated_notional_currency, estimated_notional_reason) =
            estimate_order_notional(&draft.fields)?;
        let proposal_hash = order_proposal_hash(
            &workspace_id,
            &draft.draft_id,
            draft.draft_version,
            &draft.fields,
            &estimated_notional,
            &estimated_notional_currency,
            &estimated_notional_reason,
            references,
        )?;
        let existing: Option<(String, String)> = tx
            .query_row(
                "SELECT proposal_id,projection FROM order_proposals WHERE workspace_id=?1 AND proposal_hash=?2",
                params![workspace_id, proposal_hash],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(storage_error)?;
        if let Some((proposal_id, projection)) = existing {
            let stored = decode_stored_order_proposal(
                &projection,
                &proposal_id,
                &workspace_id,
                &draft.draft_id,
                draft.draft_version as i64,
                &proposal_hash,
                proposal_sequence(&tx, &proposal_id)?,
                &workspace_id,
            )?;
            tx.commit().map_err(storage_error)?;
            return materialize_order_proposal(&self.connection, stored);
        }
        let proposal_id = format!("proposal:{}", Uuid::new_v4());
        let count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM order_proposals WHERE workspace_id=?1",
                [&workspace_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !(0..256).contains(&count) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let sequence = next_order_proposal_sequence(&tx, &workspace_id)?;
        let created_at = timestamp()?;
        let stored = StoredOrderProposal {
            proposal_id: proposal_id.clone(),
            workspace_id: workspace_id.clone(),
            draft_id: draft.draft_id,
            draft_version: draft.draft_version,
            proposal_hash: proposal_hash.clone(),
            fields: draft.fields,
            estimated_notional,
            estimated_notional_currency,
            estimated_notional_reason,
            policy_version: references.policy_version,
            policy_state_version: references.policy_state_version.clone(),
            policy_status: references.policy_status,
            policy_reference_reason: references.policy_reference_reason.clone(),
            market_snapshot_id: references.market_snapshot_id.clone(),
            market_status: references.market_status.clone(),
            market_reference_reason: references.market_reference_reason.clone(),
            created_at: created_at.clone(),
        };
        let projection = serde_json::to_string(&stored).map_err(storage_error)?;
        tx.execute(
            "INSERT INTO order_proposals(proposal_id,workspace_id,draft_id,draft_version,proposal_hash,sequence,projection) VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                &stored.proposal_id,
                &stored.workspace_id,
                &stored.draft_id,
                stored.draft_version as i64,
                &stored.proposal_hash,
                sequence,
                projection,
            ],
        )
        .map_err(storage_error)?;
        tx.execute(
            "INSERT INTO order_proposal_events(proposal_id,workspace_id,sequence,event,reason,occurred_at) VALUES(?1,?2,1,?3,NULL,?4)",
            params![&stored.proposal_id, &stored.workspace_id, "GENERATED", &created_at],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        materialize_order_proposal(&self.connection, stored)
    }

    pub fn refresh_order_proposal(
        &mut self,
        input: &OrderProposalRefresh,
        references: &OrderProposalReferences,
        refresh_status: OrderProposalRefreshStatus,
    ) -> Result<OrderProposalRefreshResult> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        validate_order_proposal_id(&input.proposal_id)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (row_workspace_id, draft_id, draft_version, proposal_hash, sequence, projection): (
            String,
            String,
            i64,
            String,
            i64,
            String,
        ) = tx
            .query_row(
                "SELECT workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE workspace_id=?1 AND proposal_id=?2",
                params![&workspace_id, &input.proposal_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .map_err(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    TradeXError::new("ORDER_PROPOSAL_NOT_FOUND")
                } else {
                    storage_error(error)
                }
            })?;
        let stored = decode_stored_order_proposal(
            &projection,
            &input.proposal_id,
            &row_workspace_id,
            &draft_id,
            draft_version,
            &proposal_hash,
            sequence,
            &workspace_id,
        )?;
        let (status, _, last_event_sequence) =
            proposal_event_state(&tx, &input.proposal_id, &workspace_id)?;
        let current_state_version =
            format!("order-proposal:{}:{last_event_sequence}", input.proposal_id);
        if input.expected_state_version != current_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if status != OrderProposalStatus::NeedsApproval {
            return Err(TradeXError::new("ORDER_PROPOSAL_NOT_REFRESHABLE"));
        }
        let draft = load_order_draft_tx(&tx, &workspace_id, &stored.draft_id)?;
        if draft.draft_version != stored.draft_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let (estimated_notional, estimated_notional_currency, estimated_notional_reason) =
            estimate_order_notional(&draft.fields)?;
        let proposal_id = format!("proposal:{}", Uuid::new_v4());
        let proposal_hash = order_proposal_hash(
            &workspace_id,
            &draft.draft_id,
            draft.draft_version,
            &draft.fields,
            &estimated_notional,
            &estimated_notional_currency,
            &estimated_notional_reason,
            references,
        )?;
        let count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM order_proposals WHERE workspace_id=?1",
                [&workspace_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !(0..256).contains(&count) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let next_event_sequence = last_event_sequence
            .checked_add(1)
            .filter(|value| *value <= 32)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        let sequence = next_order_proposal_sequence(&tx, &workspace_id)?;
        let created_at = timestamp()?;
        let invalidation_reason =
            format!("Proposal refreshed as {proposal_id}; a new approval is required.");
        tx.execute(
            "INSERT INTO order_proposal_events(proposal_id,workspace_id,sequence,event,reason,occurred_at) VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                &stored.proposal_id,
                &workspace_id,
                next_event_sequence,
                "REFRESHED",
                &invalidation_reason,
                &created_at,
            ],
        )
        .map_err(storage_error)?;
        let refreshed = StoredOrderProposal {
            proposal_id: proposal_id.clone(),
            workspace_id: workspace_id.clone(),
            draft_id: draft.draft_id,
            draft_version: draft.draft_version,
            proposal_hash,
            fields: draft.fields,
            estimated_notional,
            estimated_notional_currency,
            estimated_notional_reason,
            policy_version: references.policy_version,
            policy_state_version: references.policy_state_version.clone(),
            policy_status: references.policy_status,
            policy_reference_reason: references.policy_reference_reason.clone(),
            market_snapshot_id: references.market_snapshot_id.clone(),
            market_status: references.market_status.clone(),
            market_reference_reason: references.market_reference_reason.clone(),
            created_at: created_at.clone(),
        };
        let projection = serde_json::to_string(&refreshed).map_err(storage_error)?;
        tx.execute(
            "INSERT INTO order_proposals(proposal_id,workspace_id,draft_id,draft_version,proposal_hash,sequence,projection) VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                &refreshed.proposal_id,
                &refreshed.workspace_id,
                &refreshed.draft_id,
                refreshed.draft_version as i64,
                &refreshed.proposal_hash,
                sequence,
                projection,
            ],
        )
        .map_err(storage_error)?;
        tx.execute(
            "INSERT INTO order_proposal_events(proposal_id,workspace_id,sequence,event,reason,occurred_at) VALUES(?1,?2,1,?3,NULL,?4)",
            params![&refreshed.proposal_id, &refreshed.workspace_id, "GENERATED", &created_at],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        let previous_proposal = materialize_order_proposal(&self.connection, stored)?;
        let proposal = materialize_order_proposal(&self.connection, refreshed)?;
        Ok(OrderProposalRefreshResult {
            previous_proposal: Box::new(previous_proposal),
            proposal: Box::new(proposal),
            refresh_status,
            invalidation_reason,
        })
    }

    pub fn save_order_draft(&mut self, input: &OrderDraftSave) -> Result<OrderDraft> {
        let workspace_id = self.workspace_id()?;
        if input.workspace_id != workspace_id {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        let mut fields = input.fields.clone();
        normalize_order_draft_fields(&mut fields)?;
        self.validate_order_draft_account(&fields)?;
        if let Some(draft_id) = &input.draft_id {
            validate_order_draft_id(draft_id)?;
            let expected = input
                .expected_state_version
                .as_deref()
                .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
            validate_order_draft_state_version(expected)?;
        } else if input.expected_state_version.is_some() {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let mut previous = None;
        let (draft_id, draft_version, sequence) = if let Some(draft_id) = &input.draft_id {
            let previous_draft = load_order_draft_tx(&tx, &workspace_id, draft_id)?;
            if previous_draft.state_version
                != input
                    .expected_state_version
                    .as_deref()
                    .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let next_version = previous_draft
                .draft_version
                .checked_add(1)
                .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
            previous = Some(previous_draft);
            (
                draft_id.clone(),
                next_version,
                next_order_draft_sequence(&tx, &workspace_id)?,
            )
        } else {
            let count: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM order_drafts WHERE workspace_id=?1",
                    [&workspace_id],
                    |row| row.get(0),
                )
                .map_err(storage_error)?;
            if !(0..256).contains(&count) {
                return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
            }
            (
                Uuid::new_v4().to_string(),
                1,
                next_order_draft_sequence(&tx, &workspace_id)?,
            )
        };
        let now = timestamp()?;
        let draft = OrderDraft {
            draft_id: draft_id.clone(),
            workspace_id: workspace_id.clone(),
            draft_version,
            state_version: format!("order-draft:{draft_id}:{sequence}"),
            updated_at: now.clone(),
            fields,
        };
        let projection = serde_json::to_string(&draft).map_err(storage_error)?;
        if input.draft_id.is_some() {
            tx.execute(
                "UPDATE order_drafts SET draft_version=?1,sequence=?2,projection=?3 WHERE workspace_id=?4 AND draft_id=?5",
                params![draft.draft_version as i64, sequence, projection, workspace_id, draft_id],
            )
            .map_err(storage_error)?;
        } else {
            tx.execute(
                "INSERT INTO order_drafts(draft_id,workspace_id,draft_version,sequence,projection) VALUES(?1,?2,?3,?4,?5)",
                params![draft.draft_id, draft.workspace_id, draft.draft_version as i64, sequence, projection],
            )
                .map_err(storage_error)?;
        }
        if previous
            .as_ref()
            .is_some_and(|old| !same_order_draft_material_fields(&old.fields, &draft.fields))
        {
            invalidate_order_proposals_tx(
                &tx,
                &workspace_id,
                &draft.draft_id,
                "Draft fields changed after proposal generation.",
                &now,
            )?;
        }
        tx.commit().map_err(storage_error)?;
        Ok(draft)
    }

    fn validate_order_draft_account(&self, fields: &OrderDraftFields) -> Result<()> {
        match fields.environment {
            ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation => {
                return Err(TradeXError::new("ORDER_CONTEXT_INVALID").with_field("environment"));
            }
            ExecutionContext::LocalPaper => {
                let account_id = fields.account_id.as_deref().ok_or_else(|| {
                    TradeXError::new("ORDER_ACCOUNT_REQUIRED").with_field("accountId")
                })?;
                let account = self.account(account_id).map_err(|_| {
                    TradeXError::new("ORDER_ACCOUNT_NOT_FOUND").with_field("accountId")
                })?;
                if !account.is_local_paper() {
                    return Err(TradeXError::new("ORDER_ACCOUNT_INVALID").with_field("accountId"));
                }
            }
            _ => {
                let account_id = fields.account_id.as_deref().ok_or_else(|| {
                    TradeXError::new("ORDER_ACCOUNT_REQUIRED").with_field("accountId")
                })?;
                let account = self.account(account_id).map_err(|_| {
                    TradeXError::new("ORDER_ACCOUNT_NOT_FOUND").with_field("accountId")
                })?;
                let expected = execution_environment(&fields.environment).ok_or_else(|| {
                    TradeXError::new("ORDER_CONTEXT_INVALID").with_field("environment")
                })?;
                let expected_provider =
                    execution_provider(&fields.environment).ok_or_else(|| {
                        TradeXError::new("ORDER_CONTEXT_INVALID").with_field("environment")
                    })?;
                if account.environment != expected || account.provider_id != expected_provider {
                    return Err(TradeXError::new("ORDER_CONTEXT_INVALID").with_field("environment"));
                }
            }
        }
        Ok(())
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
        let temporary_name = format!(".{}.{}.tmp", file_name, Uuid::new_v4());
        write_export_file(parent, file_name, &temporary_name, &encoded)?;
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

fn normalize_order_draft_fields(fields: &mut OrderDraftFields) -> Result<()> {
    if let Some(account_id) = fields.account_id.as_mut() {
        *account_id = account_id.trim().to_owned();
        if !valid_order_text(account_id, 128) {
            return Err(TradeXError::new("ORDER_ACCOUNT_INVALID").with_field("accountId"));
        }
    }
    fields.venue = fields.venue.trim().to_ascii_uppercase();
    if !valid_order_text(&fields.venue, 32) {
        return Err(TradeXError::new("ORDER_VENUE_INVALID").with_field("venue"));
    }
    fields.instrument_id = fields.instrument_id.trim().to_owned();
    if !market::validate_instrument_id(&fields.instrument_id) {
        return Err(TradeXError::new("MARKET_INSTRUMENT_INVALID").with_field("instrumentId"));
    }
    let instrument = market::instruments()
        .into_iter()
        .find(|instrument| instrument.instrument_id == fields.instrument_id)
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_NOT_FOUND").with_field("instrumentId"))?;
    if matches!(
        fields.environment,
        ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation
    ) {
        return Err(TradeXError::new("ORDER_CONTEXT_INVALID").with_field("environment"));
    }
    let expected_venue = match fields.environment {
        ExecutionContext::LocalPaper => "TRADEX_SIM",
        ExecutionContext::BinanceTestnet | ExecutionContext::BinanceLive => "BINANCE",
        ExecutionContext::BitgetDemo | ExecutionContext::BitgetLive => "BITGET",
        ExecutionContext::AlpacaPaper
        | ExecutionContext::Trading212Demo
        | ExecutionContext::Trading212Live => instrument.exchange.as_deref().unwrap_or("XNAS"),
        ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation => unreachable!(),
    };
    if fields.venue != expected_venue
        || (matches!(
            fields.environment,
            ExecutionContext::BinanceTestnet
                | ExecutionContext::BinanceLive
                | ExecutionContext::BitgetDemo
                | ExecutionContext::BitgetLive
        ) && instrument.asset_class != AssetClass::CryptoSpot)
        || (matches!(
            fields.environment,
            ExecutionContext::AlpacaPaper
                | ExecutionContext::Trading212Demo
                | ExecutionContext::Trading212Live
        ) && instrument.asset_class != AssetClass::Equity)
    {
        return Err(TradeXError::new("ORDER_VENUE_INVALID").with_field("venue"));
    }

    if let Some(provider_id) = execution_provider(&fields.environment)
        && provider_id != "local-paper"
        && !instrument
            .providers
            .iter()
            .any(|mapping| mapping.provider_id == provider_id)
    {
        return Err(
            TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED").with_field("instrumentId")
        );
    }

    fields.quantity.value = normalize_order_decimal(&fields.quantity.value, "quantity")?;
    if fields.quantity.value == "0" {
        return Err(TradeXError::new("ORDER_AMOUNT_INVALID").with_field("quantity"));
    }
    if let Some(value) = fields.limit_price.as_mut() {
        *value = normalize_order_decimal(value, "limitPrice")?;
        if value == "0" {
            return Err(TradeXError::new("ORDER_AMOUNT_INVALID").with_field("limitPrice"));
        }
    }
    if let Some(value) = fields.maximum_spend.as_mut() {
        *value = normalize_order_decimal(value, "maximumSpend")?;
        if value == "0" {
            return Err(TradeXError::new("ORDER_AMOUNT_INVALID").with_field("maximumSpend"));
        }
    }
    match fields.order_type {
        OrderType::Limit if fields.limit_price.is_none() => {
            return Err(TradeXError::new("ORDER_LIMIT_PRICE_REQUIRED").with_field("limitPrice"));
        }
        OrderType::Market if fields.limit_price.is_some() => {
            return Err(TradeXError::new("ORDER_MARKET_PRICE_FORBIDDEN").with_field("orderType"));
        }
        _ => {}
    }
    if matches!(fields.order_type, OrderType::Market)
        && matches!(fields.time_in_force, TimeInForce::Ioc | TimeInForce::Fok)
    {
        return Err(TradeXError::new("ORDER_TIF_INVALID").with_field("timeInForce"));
    }
    if let Some(label) = fields.client_label.as_mut() {
        *label = label.trim().to_owned();
        if !valid_order_text(label, 80) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
    }
    Ok(())
}

fn normalize_order_decimal(value: &str, field: &str) -> Result<String> {
    crate::provider_io::decimal(&serde_json::Value::String(value.to_owned()))
        .map_err(|_| TradeXError::new("ORDER_DECIMAL_INVALID").with_field(field))
        .and_then(|normalized| {
            if normalized
                .split_once('.')
                .is_some_and(|(_, fraction)| fraction.len() > MAX_ORDER_DECIMAL_FRACTION_DIGITS)
            {
                Err(TradeXError::new("ORDER_DECIMAL_INVALID").with_field(field))
            } else if normalized.starts_with('-') {
                Err(TradeXError::new("ORDER_AMOUNT_INVALID").with_field(field))
            } else {
                Ok(normalized)
            }
        })
}

fn valid_order_text(value: &str, max_len: usize) -> bool {
    !value.is_empty() && value.chars().count() <= max_len && !value.chars().any(char::is_control)
}

pub(crate) fn validate_order_draft_id(id: &str) -> Result<()> {
    if !valid_order_text(id, 128) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    Ok(())
}

pub(crate) fn validate_order_draft_state_version(version: &str) -> Result<()> {
    if !valid_order_text(version, 256) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    Ok(())
}

fn execution_environment(environment: &ExecutionContext) -> Option<&'static str> {
    match environment {
        ExecutionContext::LocalPaper => Some("LOCAL"),
        ExecutionContext::AlpacaPaper => Some("PAPER"),
        ExecutionContext::Trading212Demo | ExecutionContext::BitgetDemo => Some("DEMO"),
        ExecutionContext::BinanceTestnet => Some("TESTNET"),
        ExecutionContext::Trading212Live
        | ExecutionContext::BinanceLive
        | ExecutionContext::BitgetLive => Some("LIVE"),
        ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation => None,
    }
}

fn execution_provider(environment: &ExecutionContext) -> Option<&'static str> {
    match environment {
        ExecutionContext::AlpacaPaper => Some("alpaca"),
        ExecutionContext::Trading212Demo | ExecutionContext::Trading212Live => Some("trading212"),
        ExecutionContext::BinanceTestnet | ExecutionContext::BinanceLive => Some("binance"),
        ExecutionContext::BitgetDemo | ExecutionContext::BitgetLive => Some("bitget"),
        ExecutionContext::LocalPaper => Some("local-paper"),
        ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation => None,
    }
}

fn next_order_draft_sequence(tx: &rusqlite::Transaction<'_>, workspace_id: &str) -> Result<i64> {
    let max_sequence: Option<i64> = tx
        .query_row(
            "SELECT MAX(sequence) FROM order_drafts WHERE workspace_id=?1",
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

fn load_order_draft_tx(
    tx: &rusqlite::Transaction<'_>,
    workspace_id: &str,
    draft_id: &str,
) -> Result<OrderDraft> {
    let (row_workspace_id, draft_version, sequence, projection): (String, i64, i64, String) = tx
        .query_row(
            "SELECT workspace_id,draft_version,sequence,projection FROM order_drafts WHERE workspace_id=?1 AND draft_id=?2",
            params![workspace_id, draft_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|error| {
            if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                TradeXError::new("ORDER_DRAFT_NOT_FOUND")
            } else {
                storage_error(error)
            }
        })?;
    decode_order_draft(
        &projection,
        draft_id,
        &row_workspace_id,
        draft_version,
        sequence,
        workspace_id,
    )
}

fn same_order_draft_material_fields(left: &OrderDraftFields, right: &OrderDraftFields) -> bool {
    left.account_id == right.account_id
        && left.venue == right.venue
        && left.environment == right.environment
        && left.instrument_id == right.instrument_id
        && left.side == right.side
        && left.order_type == right.order_type
        && left.quantity == right.quantity
        && left.limit_price == right.limit_price
        && left.maximum_spend == right.maximum_spend
        && left.time_in_force == right.time_in_force
}

fn estimate_order_notional(
    fields: &OrderDraftFields,
) -> Result<(Option<String>, Option<String>, Option<String>)> {
    let instrument = market::instruments()
        .into_iter()
        .find(|instrument| instrument.instrument_id == fields.instrument_id)
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_NOT_FOUND"))?;
    let currency = Some(instrument.currency);
    if matches!(fields.order_type, OrderType::Market)
        && matches!(
            fields.quantity.r#type,
            crate::protocol::OrderQuantityType::Base
        )
        && let Some(maximum_spend) = fields.maximum_spend.as_deref()
    {
        return Ok((Some(maximum_spend.into()), currency, None));
    }
    match (
        fields.quantity.r#type,
        fields.order_type,
        fields.limit_price.as_deref(),
    ) {
        (crate::protocol::OrderQuantityType::Quote, _, _) => {
            Ok((Some(fields.quantity.value.clone()), currency, None))
        }
        (crate::protocol::OrderQuantityType::Base, OrderType::Limit, Some(limit_price)) => Ok((
            Some(crate::portfolio::decimal_mul(
                &fields.quantity.value,
                limit_price,
            )?),
            currency,
            None,
        )),
        (crate::protocol::OrderQuantityType::Base, OrderType::Market, None) => Ok((
            None,
            currency,
            Some("Market order notional requires a provider quote or maximum spend.".into()),
        )),
        _ => Ok((
            None,
            currency,
            Some("Order notional is unavailable for the saved draft fields.".into()),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn order_proposal_hash(
    workspace_id: &str,
    draft_id: &str,
    draft_version: u64,
    fields: &OrderDraftFields,
    estimated_notional: &Option<String>,
    estimated_notional_currency: &Option<String>,
    estimated_notional_reason: &Option<String>,
    references: &OrderProposalReferences,
) -> Result<String> {
    let canonical = OrderProposalHashInput {
        workspace_id,
        draft_id,
        draft_version,
        fields: OrderProposalHashFields {
            account_id: fields.account_id.as_deref(),
            venue: &fields.venue,
            environment: &fields.environment,
            instrument_id: &fields.instrument_id,
            side: &fields.side,
            order_type: &fields.order_type,
            quantity: &fields.quantity,
            limit_price: fields.limit_price.as_deref(),
            maximum_spend: fields.maximum_spend.as_deref(),
            time_in_force: &fields.time_in_force,
        },
        estimated_notional: estimated_notional.as_deref(),
        estimated_notional_currency: estimated_notional_currency.as_deref(),
        estimated_notional_reason: estimated_notional_reason.as_deref(),
        policy_version: references.policy_version,
        policy_state_version: references.policy_state_version.as_deref(),
        policy_status: references.policy_status,
        policy_reference_reason: &references.policy_reference_reason,
        market_snapshot_id: references.market_snapshot_id.as_deref(),
        market_status: references.market_status.clone(),
        market_reference_reason: &references.market_reference_reason,
    };
    Ok(hash_bytes(
        &serde_json::to_vec(&canonical).map_err(storage_error)?,
    ))
}

#[allow(clippy::too_many_arguments)]
fn decode_stored_order_proposal(
    projection: &str,
    row_id: &str,
    row_workspace_id: &str,
    row_draft_id: &str,
    row_draft_version: i64,
    row_proposal_hash: &str,
    sequence: i64,
    workspace_id: &str,
) -> Result<StoredOrderProposal> {
    if sequence < 1
        || sequence > MAX_SEQUENCE as i64
        || !(1..=9_007_199_254_740_991).contains(&row_draft_version)
        || row_workspace_id != workspace_id
        || !valid_order_proposal_id(row_id)
        || !valid_proposal_hash(row_proposal_hash)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let proposal: StoredOrderProposal = serde_json::from_str(projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    let mut normalized = proposal.fields.clone();
    if normalize_order_draft_fields(&mut normalized).is_err()
        || normalized != proposal.fields
        || proposal.proposal_id != row_id
        || proposal.workspace_id != row_workspace_id
        || proposal.draft_id != row_draft_id
        || proposal.draft_version != row_draft_version as u64
        || proposal.proposal_hash != row_proposal_hash
        || !valid_order_text(&proposal.created_at, 64)
        || !valid_order_text(&proposal.policy_reference_reason, 256)
        || !valid_order_text(&proposal.market_reference_reason, 256)
        || proposal
            .policy_state_version
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 256))
        || proposal
            .market_snapshot_id
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 128))
        || proposal.estimated_notional.as_deref().is_some_and(|value| {
            !crate::provider_io::decimal(&serde_json::Value::String(value.to_owned()))
                .is_ok_and(|normalized| normalized == value && normalized != "0")
        })
        || proposal
            .estimated_notional_currency
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 16))
        || proposal
            .estimated_notional_reason
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 256))
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let computed_hash = order_proposal_hash(
        workspace_id,
        row_draft_id,
        proposal.draft_version,
        &proposal.fields,
        &proposal.estimated_notional,
        &proposal.estimated_notional_currency,
        &proposal.estimated_notional_reason,
        &OrderProposalReferences {
            policy_version: proposal.policy_version,
            policy_state_version: proposal.policy_state_version.clone(),
            policy_status: proposal.policy_status,
            policy_reference_reason: proposal.policy_reference_reason.clone(),
            market_snapshot_id: proposal.market_snapshot_id.clone(),
            market_status: proposal.market_status.clone(),
            market_reference_reason: proposal.market_reference_reason.clone(),
        },
    )?;
    if computed_hash != proposal.proposal_hash {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(proposal)
}

fn valid_order_proposal_id(id: &str) -> bool {
    let Some(hex) = id.strip_prefix("proposal:") else {
        return false;
    };
    Uuid::parse_str(hex).is_ok()
}

fn trading212_demo_attempt_state_name(state: Trading212DemoOrderAttemptState) -> &'static str {
    match state {
        Trading212DemoOrderAttemptState::Submitting => "SUBMITTING",
        Trading212DemoOrderAttemptState::Acknowledged => "ACKNOWLEDGED",
        Trading212DemoOrderAttemptState::UnknownReconciling => "UNKNOWN_RECONCILING",
        Trading212DemoOrderAttemptState::Rejected => "REJECTED",
    }
}

fn trading212_demo_attempt_version(attempt_id: &str, sequence: u64) -> String {
    format!("trading212-demo-order-attempt:{attempt_id}:{sequence}")
}

fn validate_trading212_demo_attempt(attempt: &Trading212DemoOrderAttempt) -> Result<()> {
    let provider_id_valid = |value: &str| {
        value.parse::<i64>().is_ok_and(|id| id > 0)
            && value.bytes().all(|byte| byte.is_ascii_digit())
    };
    if Uuid::parse_str(&attempt.attempt_id).is_err()
        || !valid_order_text(&attempt.workspace_id, 128)
        || !valid_order_text(&attempt.connection_id, 128)
        || !provider_id_valid(&attempt.remote_account_id)
        || !valid_order_proposal_id(&attempt.proposal_id)
        || !valid_proposal_hash(&attempt.proposal_hash)
        || attempt
            .provider_order_id
            .as_deref()
            .is_some_and(|id| !provider_id_valid(id))
        || attempt
            .provider_status
            .as_deref()
            .is_some_and(|status| !valid_order_text(status, 32) || !status.is_ascii())
        || attempt
            .error_code
            .as_deref()
            .is_some_and(|code| !valid_order_text(code, 64) || !code.is_ascii())
        || !valid_order_text(&attempt.reason, 256)
        || attempt.state_version
            != attempt
                .state_version
                .rsplit_once(':')
                .and_then(|(_, sequence)| sequence.parse::<u64>().ok())
                .map(|sequence| trading212_demo_attempt_version(&attempt.attempt_id, sequence))
                .unwrap_or_default()
        || !valid_provider_time(&attempt.created_at)
        || !valid_provider_time(&attempt.updated_at)
        || match attempt.state {
            Trading212DemoOrderAttemptState::Submitting => {
                attempt.provider_order_id.is_some()
                    || attempt.provider_status.is_some()
                    || attempt.error_code.is_some()
            }
            Trading212DemoOrderAttemptState::Acknowledged => {
                attempt.provider_order_id.is_none()
                    || attempt.provider_status.is_none()
                    || attempt.error_code.is_some()
            }
            Trading212DemoOrderAttemptState::UnknownReconciling
            | Trading212DemoOrderAttemptState::Rejected => {
                attempt.provider_order_id.is_some()
                    || attempt.provider_status.is_some()
                    || attempt.error_code.is_none()
            }
        }
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(())
}

fn write_trading212_demo_attempt_event(
    tx: &Transaction<'_>,
    attempt: &Trading212DemoOrderAttempt,
    sequence: i64,
) -> Result<()> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    validate_trading212_demo_attempt(attempt)?;
    let event = DomainEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "trading212.demo.order.attempt.changed".into(),
        schema_version: 1,
        occurred_at: attempt.updated_at.clone(),
        aggregate_type: "trading212-demo-order-attempt".into(),
        aggregate_id: attempt.attempt_id.clone(),
        sequence: u64::try_from(sequence).map_err(storage_error)?,
        payload: DomainProjection::Trading212DemoOrderAttempt(Box::new(attempt.clone())),
    };
    tx.execute(
        "INSERT INTO outbox VALUES(?1,?2,?3,?4,?5)",
        params![
            event.aggregate_type,
            event.aggregate_id,
            sequence,
            event.event_id,
            serde_json::to_string(&event).map_err(storage_error)?,
        ],
    )
    .map_err(storage_error)?;
    Ok(())
}

fn alpaca_attempt_state_name(state: AlpacaPaperOrderAttemptState) -> &'static str {
    match state {
        AlpacaPaperOrderAttemptState::Submitting => "SUBMITTING",
        AlpacaPaperOrderAttemptState::Acknowledged => "ACKNOWLEDGED",
        AlpacaPaperOrderAttemptState::UnknownReconciling => "UNKNOWN_RECONCILING",
        AlpacaPaperOrderAttemptState::Rejected => "REJECTED",
    }
}

fn write_alpaca_paper_attempt_event(
    tx: &Transaction<'_>,
    attempt: &AlpacaPaperOrderAttempt,
    sequence: i64,
) -> Result<()> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    let event = DomainEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "alpaca.paper.order.attempt.changed".into(),
        schema_version: 1,
        occurred_at: attempt.updated_at.clone(),
        aggregate_type: "alpaca-paper-order-attempt".into(),
        aggregate_id: attempt.attempt_id.clone(),
        sequence: u64::try_from(sequence).map_err(storage_error)?,
        payload: DomainProjection::AlpacaPaperOrderAttempt(Box::new(attempt.clone())),
    };
    tx.execute(
        "INSERT INTO outbox VALUES(?1,?2,?3,?4,?5)",
        params![
            event.aggregate_type,
            event.aggregate_id,
            sequence,
            event.event_id,
            serde_json::to_string(&event).map_err(storage_error)?,
        ],
    )
    .map_err(storage_error)?;
    Ok(())
}

fn binance_attempt_state_name(state: BinanceTestnetOrderAttemptState) -> &'static str {
    match state {
        BinanceTestnetOrderAttemptState::Submitting => "SUBMITTING",
        BinanceTestnetOrderAttemptState::Acknowledged => "ACKNOWLEDGED",
        BinanceTestnetOrderAttemptState::UnknownReconciling => "UNKNOWN_RECONCILING",
        BinanceTestnetOrderAttemptState::Rejected => "REJECTED",
    }
}

fn bitget_demo_attempt_state_name(state: BitgetDemoOrderAttemptState) -> &'static str {
    match state {
        BitgetDemoOrderAttemptState::Submitting => "SUBMITTING",
        BitgetDemoOrderAttemptState::Acknowledged => "ACKNOWLEDGED",
        BitgetDemoOrderAttemptState::UnknownReconciling => "UNKNOWN_RECONCILING",
        BitgetDemoOrderAttemptState::Rejected => "REJECTED",
    }
}

fn validate_bitget_demo_attempt(attempt: &BitgetDemoOrderAttempt) -> Result<()> {
    let provider_id_valid = |value: &str| {
        !value.is_empty()
            && value.len() <= 40
            && !value.starts_with('0')
            && value.bytes().all(|byte| byte.is_ascii_digit())
    };
    let client_oid_valid = |value: &str| {
        value.starts_with("tx-")
            && value.len() <= 50
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    };
    if Uuid::parse_str(&attempt.attempt_id).is_err()
        || !valid_order_text(&attempt.workspace_id, 128)
        || !valid_order_text(&attempt.connection_id, 128)
        || !provider_id_valid(&attempt.remote_account_id)
        || !valid_order_proposal_id(&attempt.proposal_id)
        || !valid_proposal_hash(&attempt.proposal_hash)
        || attempt.environment != "DEMO"
        || !client_oid_valid(&attempt.client_oid)
        || attempt
            .provider_order_id
            .as_deref()
            .is_some_and(|id| !provider_id_valid(id))
        || attempt
            .provider_status
            .as_deref()
            .is_some_and(|status| !valid_order_text(status, 64) || !status.is_ascii())
        || attempt
            .error_code
            .as_deref()
            .is_some_and(|code| !valid_order_text(code, 64) || !code.is_ascii())
        || !valid_order_text(&attempt.reason, 256)
        || attempt.state_version
            != attempt
                .state_version
                .rsplit_once(':')
                .and_then(|(_, sequence)| sequence.parse::<u64>().ok())
                .map(|sequence| {
                    format!(
                        "bitget-demo-order-attempt:{}:{sequence}",
                        attempt.attempt_id
                    )
                })
                .unwrap_or_default()
        || !valid_provider_time(&attempt.created_at)
        || !valid_provider_time(&attempt.updated_at)
        || match attempt.state {
            BitgetDemoOrderAttemptState::Submitting => {
                attempt.provider_order_id.is_some()
                    || attempt.provider_status.is_some()
                    || attempt.error_code.is_some()
            }
            BitgetDemoOrderAttemptState::Acknowledged => {
                attempt.provider_order_id.is_none() || attempt.error_code.is_some()
            }
            BitgetDemoOrderAttemptState::UnknownReconciling
            | BitgetDemoOrderAttemptState::Rejected => {
                attempt.provider_order_id.is_some()
                    || attempt.provider_status.is_some()
                    || attempt.error_code.is_none()
            }
        }
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(())
}

fn write_binance_testnet_attempt_event(
    tx: &Transaction<'_>,
    attempt: &BinanceTestnetOrderAttempt,
    sequence: i64,
) -> Result<()> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    let event = DomainEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "binance.testnet.order.attempt.changed".into(),
        schema_version: 1,
        occurred_at: attempt.updated_at.clone(),
        aggregate_type: "binance-testnet-order-attempt".into(),
        aggregate_id: attempt.attempt_id.clone(),
        sequence: u64::try_from(sequence).map_err(storage_error)?,
        payload: DomainProjection::BinanceTestnetOrderAttempt(Box::new(attempt.clone())),
    };
    tx.execute(
        "INSERT INTO outbox VALUES(?1,?2,?3,?4,?5)",
        params![
            event.aggregate_type,
            event.aggregate_id,
            sequence,
            event.event_id,
            serde_json::to_string(&event).map_err(storage_error)?,
        ],
    )
    .map_err(storage_error)?;
    Ok(())
}

fn write_bitget_demo_attempt_event(
    tx: &Transaction<'_>,
    attempt: &BitgetDemoOrderAttempt,
    sequence: i64,
) -> Result<()> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    let event = DomainEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "bitget.demo.order.attempt.changed".into(),
        schema_version: 1,
        occurred_at: attempt.updated_at.clone(),
        aggregate_type: "bitget-demo-order-attempt".into(),
        aggregate_id: attempt.attempt_id.clone(),
        sequence: u64::try_from(sequence).map_err(storage_error)?,
        payload: DomainProjection::BitgetDemoOrderAttempt(Box::new(attempt.clone())),
    };
    tx.execute(
        "INSERT INTO outbox VALUES(?1,?2,?3,?4,?5)",
        params![
            event.aggregate_type,
            event.aggregate_id,
            sequence,
            event.event_id,
            serde_json::to_string(&event).map_err(storage_error)?,
        ],
    )
    .map_err(storage_error)?;
    Ok(())
}

fn alpaca_order_book_version(connection_id: &str, sequence: u64) -> String {
    format!("alpaca-paper-order-book:{connection_id}:{sequence}")
}

fn trading212_demo_order_book_version(connection_id: &str, sequence: u64) -> String {
    format!("trading212-demo-order-book:{connection_id}:{sequence}")
}

fn binance_testnet_order_book_version(connection_id: &str, sequence: u64) -> String {
    format!("binance-testnet-order-book:{connection_id}:{sequence}")
}

fn binance_terminal_status(status: &str) -> bool {
    matches!(
        status,
        "FILLED" | "CANCELED" | "REJECTED" | "EXPIRED" | "EXPIRED_IN_MATCH"
    )
}

fn validate_binance_testnet_order_book(book: &BinanceTestnetOrderBook) -> Result<()> {
    let invalid = || TradeXError::new("WORKSPACE_INTEGRITY_FAILED");
    if !valid_order_text(&book.workspace_id, 128)
        || !valid_order_text(&book.connection_id, 128)
        || !valid_order_text(&book.remote_account_id, 128)
        || book.environment != "TESTNET"
        || !valid_order_text(&book.state_version, 256)
        || !valid_provider_time(&book.observed_at)
        || book.orders.len() > 5000
        || book.fills.len() > 5000
        || book.balances.len() > 5000
        || book.history.len() != 2
        || book
            .reason
            .as_deref()
            .is_some_and(|v| !valid_order_text(v, 256))
        || book
            .last_successful_sync_at
            .as_deref()
            .is_some_and(|v| !valid_order_text(v, 64) || !valid_provider_time(v))
        || [
            book.pending_orders_observed_at.as_deref(),
            book.balances_observed_at.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|v| !valid_order_text(v, 64) || !valid_provider_time(v))
        || [
            book.rate_limits.pending_orders_retry_at.as_deref(),
            book.rate_limits.account_retry_at.as_deref(),
            book.rate_limits.history_retry_at.as_deref(),
            book.rate_limits.order_detail_retry_at.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|v| !valid_order_text(v, 64) || !valid_provider_time(v))
        || book
            .private_stream_balance_update_at_ms
            .is_some_and(|value| value > MAX_SEQUENCE)
    {
        return Err(invalid());
    }
    let mut history_symbols = std::collections::HashSet::new();
    for state in &book.history {
        if !matches!(state.symbol.as_str(), "BTCUSDT" | "ETHUSDT")
            || !history_symbols.insert(state.symbol.as_str())
            || state.started != (state.page_count > 0)
            || state.complete && (state.next_order_id.is_some() || state.next_trade_id.is_some())
            || !state.started
                && (state.complete
                    || state.next_order_id.is_some()
                    || state.next_trade_id.is_some())
            || state
                .next_order_id
                .as_deref()
                .is_some_and(|v| !valid_binance_id(v))
            || state
                .next_trade_id
                .as_deref()
                .is_some_and(|v| !valid_binance_id(v))
            || state
                .last_observed_at
                .as_deref()
                .is_some_and(|v| !valid_order_text(v, 64) || !valid_provider_time(v))
        {
            return Err(invalid());
        }
    }
    let mut order_ids = std::collections::HashSet::new();
    let mut cancel_keys = std::collections::HashSet::new();
    for order in &book.orders {
        let key = format!("{}:{}", order.symbol, order.provider_order_id);
        let cancel_fields_invalid = match order.cancel_state {
            BinanceTestnetOrderCancelState::None => false,
            BinanceTestnetOrderCancelState::Submitting
            | BinanceTestnetOrderCancelState::Pending => order
                .cancel_idempotency_key
                .as_deref()
                .is_none_or(|value| Uuid::parse_str(value).is_err()),
        };
        if !valid_binance_id(&order.provider_order_id)
            || !valid_order_text(&order.symbol, 32)
            || !valid_order_text(&order.client_order_id, 36)
            || !valid_order_text(&order.side, 16)
            || !valid_order_text(&order.order_type, 32)
            || !valid_order_text(&order.time_in_force, 32)
            || !valid_order_text(&order.provider_status, 64)
            || !valid_order_text(&order.filled_quantity, 64)
            || !valid_provider_time(&order.observed_at)
            || order.submitted_at_ms > MAX_SEQUENCE
            || order.provider_updated_at_ms > MAX_SEQUENCE
            || !order_ids.insert(key)
            || !valid_provider_decimal(&order.filled_quantity, true)
            || order
                .quantity
                .as_deref()
                .is_some_and(|v| !valid_provider_decimal(v, true))
            || order
                .quote_quantity
                .as_deref()
                .is_some_and(|v| !valid_provider_decimal(v, true))
            || order
                .filled_quote_quantity
                .as_deref()
                .is_some_and(|v| !valid_provider_decimal(v, true))
            || order
                .remaining_quantity
                .as_deref()
                .is_some_and(|v| !valid_provider_decimal(v, true))
            || order.pending == binance_terminal_status(&order.provider_status)
            || cancel_fields_invalid
            || order
                .cancel_idempotency_key
                .as_deref()
                .is_some_and(|value| {
                    !valid_order_text(value, 36)
                        || Uuid::parse_str(value).is_err()
                        || !cancel_keys.insert(value)
                })
            || order
                .cancel_error
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 64))
            || !order.pending
                && (order.cancel_state != BinanceTestnetOrderCancelState::None
                    || order.cancel_idempotency_key.is_some())
            || (order.origin == BinanceTestnetOrderOrigin::TradeX) != order.attempt_id.is_some()
            || order
                .attempt_id
                .as_deref()
                .is_some_and(|v| !valid_order_text(v, 128))
        {
            return Err(invalid());
        }
        if let Some(quantity) = &order.quantity {
            let remaining = crate::provider_io::decimal_subtract(quantity, &order.filled_quantity)
                .map_err(|_| invalid())?;
            if order.remaining_quantity.as_ref() != Some(&remaining) {
                return Err(invalid());
            }
        } else if order.remaining_quantity.is_some() {
            return Err(invalid());
        }
    }
    let mut fill_ids = std::collections::HashSet::new();
    for fill in &book.fills {
        let key = format!("{}:{}", fill.symbol, fill.trade_id);
        if !valid_binance_id(&fill.trade_id)
            || !valid_binance_id(&fill.provider_order_id)
            || !valid_order_text(&fill.symbol, 32)
            || !valid_order_text(&fill.side, 16)
            || !valid_order_text(&fill.commission_asset, 32)
            || !valid_order_text(&fill.observed_at, 64)
            || !valid_provider_time(&fill.observed_at)
            || fill.executed_at_ms > MAX_SEQUENCE
            || !fill_ids.insert(key)
            || !valid_provider_decimal(&fill.price, true)
            || !valid_provider_decimal(&fill.quantity, false)
            || !valid_provider_decimal(&fill.quote_quantity, true)
            || !valid_provider_decimal(&fill.commission, true)
        {
            return Err(invalid());
        }
    }
    let mut assets = std::collections::HashSet::new();
    for balance in &book.balances {
        if !valid_order_text(&balance.asset, 32)
            || !assets.insert(balance.asset.as_str())
            || !valid_provider_decimal(&balance.free, true)
            || !valid_provider_decimal(&balance.locked, true)
            || !valid_provider_decimal(&balance.total, true)
            || crate::provider_io::decimal_subtract(&balance.total, &balance.free)
                .map_or(true, |locked| locked != balance.locked)
        {
            return Err(invalid());
        }
    }
    Ok(())
}

fn valid_binance_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|b| b.is_ascii_digit())
        && value
            .parse::<u64>()
            .is_ok_and(|v| v > 0 && v <= i64::MAX as u64)
}

fn provider_order_id_valid(id: &str) -> bool {
    id.len() == 36 && Uuid::parse_str(id).is_ok()
}

fn validate_alpaca_order_book(book: &AlpacaPaperOrderBook) -> Result<()> {
    if !valid_order_text(&book.workspace_id, 128)
        || !valid_order_text(&book.connection_id, 128)
        || !valid_order_text(&book.remote_account_id, 128)
        || !valid_order_text(&book.state_version, 256)
        || !valid_order_text(&book.observed_at, 64)
        || book.orders.len() > 500
        || book.fills.len() > 1000
        || book
            .reason
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 256))
        || book
            .last_successful_sync_at
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 64))
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let mut order_ids = std::collections::HashSet::new();
    for order in &book.orders {
        if !provider_order_id_valid(&order.provider_order_id)
            || !valid_order_text(&order.client_order_id, 128)
            || !valid_order_text(&order.symbol, 16)
            || !valid_order_text(&order.side, 16)
            || !valid_order_text(&order.order_type, 32)
            || !valid_order_text(&order.time_in_force, 32)
            || !valid_order_text(&order.provider_status, 64)
            || !valid_order_text(&order.filled_quantity, 64)
            || !valid_order_text(&order.submitted_at, 64)
            || !valid_order_text(&order.observed_at, 64)
            || !order_ids.insert(order.provider_order_id.as_str())
            || !valid_provider_time(&order.submitted_at)
            || !valid_provider_time(&order.observed_at)
            || order
                .provider_updated_at
                .as_deref()
                .is_some_and(|value| !valid_provider_time(value))
            || !valid_provider_decimal(&order.filled_quantity, true)
            || order
                .quantity
                .as_deref()
                .is_some_and(|value| !valid_provider_decimal(value, true))
            || order
                .remaining_quantity
                .as_deref()
                .is_some_and(|value| !valid_provider_decimal(value, true))
            || order
                .instrument_id
                .as_deref()
                .is_some_and(|value| !market::validate_instrument_id(value))
            || order
                .cancel_idempotency_key
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 128))
            || order
                .cancel_error
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 64))
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        if let (Some(quantity), Some(remaining)) = (&order.quantity, &order.remaining_quantity) {
            let calculated = crate::provider_io::decimal_subtract(quantity, &order.filled_quantity)
                .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if &calculated != remaining {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
        }
    }
    let mut fill_ids = std::collections::HashSet::new();
    for fill in &book.fills {
        if !valid_order_text(&fill.activity_id, 128)
            || !valid_order_text(&fill.symbol, 16)
            || !valid_order_text(&fill.side, 16)
            || !valid_order_text(&fill.quantity, 64)
            || !valid_order_text(&fill.price, 64)
            || !valid_order_text(&fill.executed_at, 64)
            || !valid_order_text(&fill.observed_at, 64)
            || !provider_order_id_valid(&fill.provider_order_id)
            || !fill_ids.insert(fill.activity_id.as_str())
            || !valid_provider_decimal(&fill.quantity, false)
            || !valid_provider_decimal(&fill.price, false)
            || !valid_provider_time(&fill.executed_at)
            || !valid_provider_time(&fill.observed_at)
            || fill
                .instrument_id
                .as_deref()
                .is_some_and(|value| !market::validate_instrument_id(value))
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
    }
    Ok(())
}

fn valid_provider_time(value: &str) -> bool {
    time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).is_ok()
}

fn valid_provider_decimal(value: &str, allow_zero: bool) -> bool {
    crate::provider_io::decimal(&serde_json::Value::String(value.to_owned())).is_ok_and(
        |normalized| normalized == value && !value.starts_with('-') && (allow_zero || value != "0"),
    )
}

fn valid_signed_provider_decimal(value: &str, allow_zero: bool) -> bool {
    crate::provider_io::decimal(&serde_json::Value::String(value.to_owned()))
        .is_ok_and(|normalized| normalized == value && (allow_zero || value != "0"))
}

fn valid_trading212_status(
    raw: &str,
    normalized: crate::protocol::Trading212DemoNormalizedOrderStatus,
) -> bool {
    use crate::protocol::Trading212DemoNormalizedOrderStatus as Status;
    matches!(
        (raw, normalized),
        ("LOCAL", Status::Local)
            | ("UNCONFIRMED", Status::Pending)
            | ("CONFIRMED", Status::Open)
            | ("NEW", Status::Open)
            | ("CANCELLING", Status::CancelPending)
            | ("CANCELLED", Status::Cancelled)
            | ("PARTIALLY_FILLED", Status::PartiallyFilled)
            | ("FILLED", Status::Filled)
            | ("REJECTED", Status::Rejected)
            | ("REPLACING", Status::Replacing)
            | ("REPLACED", Status::Replaced)
            | ("EXPIRED", Status::Expired)
    )
}

fn validate_trading212_demo_order_book(book: &Trading212DemoOrderBook) -> Result<()> {
    let invalid = || TradeXError::new("WORKSPACE_INTEGRITY_FAILED");
    if !valid_order_text(&book.workspace_id, 128)
        || !valid_order_text(&book.connection_id, 128)
        || !crate::provider_io::valid_t212_order_id(&book.remote_account_id)
        || book.environment != "DEMO"
        || !valid_order_text(&book.state_version, 256)
        || !valid_provider_time(&book.observed_at)
        || book.orders.len() > 5000
        || book.history_page_count > 100
        || book.history_cursors.len() > 100
        || book.history_page_count as usize != book.history_cursors.len()
        || book.history_started != (book.history_page_count > 0)
        || book
            .reason
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 256))
        || book
            .last_successful_sync_at
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 64) || !valid_provider_time(value))
        || book
            .next_page_path
            .as_deref()
            .is_some_and(|path| !crate::provider_io::valid_t212_history_path(path))
        || book.history_complete && book.next_page_path.is_some()
        || !book.history_started && book.history_complete
        || [
            book.rate_limits.pending_orders_retry_at.as_deref(),
            book.rate_limits.order_detail_retry_at.as_deref(),
            book.rate_limits.history_retry_at.as_deref(),
            book.rate_limits.cancel_order_retry_at.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|value| !valid_order_text(value, 64) || !valid_provider_time(value))
    {
        return Err(invalid());
    }
    let mut cursors = std::collections::HashSet::new();
    if book.history_cursors.iter().any(|cursor| {
        (cursor != "FIRST"
            && (cursor.is_empty()
                || cursor.len() > 19
                || !cursor.bytes().all(|byte| byte.is_ascii_digit())
                || cursor.parse::<i64>().is_err()))
            || !cursors.insert(cursor.as_str())
    }) {
        return Err(invalid());
    }
    let mut ids = std::collections::HashSet::new();
    for order in &book.orders {
        use crate::protocol::Trading212DemoCancelState as CancelState;
        let cancel_fields_invalid = match order.cancel_state {
            CancelState::None => order.cancel_idempotency_key.is_some(),
            CancelState::Submitting | CancelState::Pending => order
                .cancel_idempotency_key
                .as_deref()
                .is_none_or(|value| !valid_order_text(value, 128)),
        };
        if !crate::provider_io::valid_t212_order_id(&order.provider_order_id)
            || !valid_order_text(&order.symbol, 64)
            || !order
                .symbol
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._/-".contains(&byte))
            || !matches!(order.side.as_str(), "BUY" | "SELL")
            || !matches!(
                order.order_type.as_str(),
                "MARKET" | "LIMIT" | "STOP" | "STOP_LIMIT"
            )
            || !matches!(order.time_in_force.as_str(), "DAY" | "GOOD_TILL_CANCEL")
            || !valid_trading212_status(&order.provider_status, order.normalized_status)
            || !valid_order_text(&order.submitted_at, 64)
            || !valid_provider_time(&order.submitted_at)
            || !valid_order_text(&order.observed_at, 64)
            || !valid_provider_time(&order.observed_at)
            || !ids.insert(order.provider_order_id.as_str())
            || cancel_fields_invalid
            || order
                .cancel_error
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 64))
            || matches!(
                order.provider_status.as_str(),
                "CANCELLED" | "FILLED" | "REJECTED" | "REPLACED" | "EXPIRED"
            ) && (order.pending || order.cancel_state != CancelState::None)
            || order
                .provider_updated_at
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 64) || !valid_provider_time(value))
            || order
                .quantity
                .as_deref()
                .is_some_and(|value| !valid_signed_provider_decimal(value, true))
            || order
                .filled_quantity
                .as_deref()
                .is_some_and(|value| !valid_signed_provider_decimal(value, true))
            || order
                .filled_value
                .as_deref()
                .is_some_and(|value| !valid_provider_decimal(value, true))
            || order.currency.as_deref().is_some_and(|value| {
                value.len() != 3 || !value.bytes().all(|byte| byte.is_ascii_uppercase())
            })
            || order
                .remaining_quantity
                .as_deref()
                .is_some_and(|value| !valid_provider_decimal(value, true))
            || order.origin == Trading212DemoOrderOrigin::TradeX
                && order
                    .attempt_id
                    .as_deref()
                    .is_none_or(|value| !valid_order_text(value, 128))
            || order.origin == Trading212DemoOrderOrigin::External && order.attempt_id.is_some()
        {
            return Err(invalid());
        }
    }
    Ok(())
}

fn load_trading212_demo_order_book(
    connection: &Connection,
    workspace_id: &str,
    connection_id: &str,
) -> Result<Option<Trading212DemoOrderBook>> {
    let row: Option<(i64, String)> = connection
        .query_row(
            "SELECT sequence,projection FROM trading212_demo_order_books WHERE workspace_id=?1 AND connection_id=?2",
            params![workspace_id, connection_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(storage_error)?;
    let Some((sequence, projection)) = row else {
        return Ok(None);
    };
    let book: Trading212DemoOrderBook = serde_json::from_str(&projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if sequence < 1
        || book.workspace_id != workspace_id
        || book.connection_id != connection_id
        || book.state_version != trading212_demo_order_book_version(connection_id, sequence as u64)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    validate_trading212_demo_order_book(&book)?;
    Ok(Some(book))
}

fn write_trading212_demo_order_book_event(
    tx: &Transaction<'_>,
    book: &Trading212DemoOrderBook,
    sequence: i64,
) -> Result<DomainEvent> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    let event = DomainEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "trading212.demo.order.book.changed".into(),
        schema_version: 1,
        occurred_at: book.observed_at.clone(),
        aggregate_type: "trading212-demo-order-book".into(),
        aggregate_id: book.connection_id.clone(),
        sequence: u64::try_from(sequence).map_err(storage_error)?,
        payload: DomainProjection::Trading212DemoOrderBook(Box::new(book.clone())),
    };
    tx.execute(
        "INSERT INTO outbox VALUES(?1,?2,?3,?4,?5)",
        params![
            event.aggregate_type,
            event.aggregate_id,
            sequence,
            event.event_id,
            serde_json::to_string(&event).map_err(storage_error)?,
        ],
    )
    .map_err(storage_error)?;
    Ok(event)
}

fn load_binance_testnet_order_book(
    connection: &Connection,
    workspace_id: &str,
    connection_id: &str,
) -> Result<Option<BinanceTestnetOrderBook>> {
    let row: Option<(i64, String)> = connection
        .query_row(
            "SELECT sequence,projection FROM binance_testnet_order_books WHERE workspace_id=?1 AND connection_id=?2",
            params![workspace_id, connection_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(storage_error)?;
    let Some((sequence, projection)) = row else {
        return Ok(None);
    };
    let book: BinanceTestnetOrderBook = serde_json::from_str(&projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if sequence < 1
        || book.workspace_id != workspace_id
        || book.connection_id != connection_id
        || book.state_version != binance_testnet_order_book_version(connection_id, sequence as u64)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    validate_binance_testnet_order_book(&book)?;
    Ok(Some(book))
}

fn complete_binance_testnet_order_book_in_tx(
    tx: &Transaction<'_>,
    workspace_id: &str,
    mut book: BinanceTestnetOrderBook,
) -> Result<(BinanceTestnetOrderBook, DomainEvent)> {
    if book.workspace_id != workspace_id || book.environment != "TESTNET" {
        return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
    }
    let account_projection: String = tx
        .query_row(
            "SELECT projection FROM accounts WHERE connection_id=?1",
            [&book.connection_id],
            |row| row.get(0),
        )
        .map_err(|error| {
            if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                TradeXError::new("IPC_AGGREGATE_NOT_FOUND")
            } else {
                storage_error(error)
            }
        })?;
    let account: AccountConnection = serde_json::from_str(&account_projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    account.validate_persisted(workspace_id)?;
    if account.provider_id != "binance"
        || account.environment != "TESTNET"
        || account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.as_str())
            != Some(book.remote_account_id.as_str())
    {
        return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
    }
    let row: Option<(i64, String)> = tx
        .query_row(
            "SELECT sequence,projection FROM binance_testnet_order_books WHERE workspace_id=?1 AND connection_id=?2",
            params![workspace_id, book.connection_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(storage_error)?;
    let sequence = if let Some((sequence, projection)) = row {
        let current: BinanceTestnetOrderBook = serde_json::from_str(&projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        if sequence < 1
            || current.state_version
                != binance_testnet_order_book_version(&book.connection_id, sequence as u64)
            || book.state_version != current.state_version
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        sequence
    } else {
        if book.state_version != binance_testnet_order_book_version(&book.connection_id, 0) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        0
    };
    let mut query = tx
        .prepare(
            "SELECT client_order_id,projection FROM binance_testnet_order_attempts WHERE workspace_id=?1 AND connection_id=?2",
        )
        .map_err(storage_error)?;
    let rows = query
        .query_map(params![workspace_id, book.connection_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(storage_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(storage_error)?;
    drop(query);
    let mut known_orders = std::collections::HashMap::new();
    for (client_order_id, projection) in rows {
        let attempt: BinanceTestnetOrderAttempt = serde_json::from_str(&projection)
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        if attempt.connection_id != book.connection_id
            || attempt.workspace_id != workspace_id
            || attempt.environment != "TESTNET"
            || attempt.remote_account_id != book.remote_account_id
            || attempt.client_order_id != client_order_id
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        known_orders.insert(client_order_id, attempt.attempt_id);
    }
    for order in &mut book.orders {
        if let Some(attempt_id) = known_orders.get(&order.client_order_id) {
            order.origin = BinanceTestnetOrderOrigin::TradeX;
            order.attempt_id = Some(attempt_id.clone());
        } else {
            order.origin = BinanceTestnetOrderOrigin::External;
            order.attempt_id = None;
        }
    }
    validate_binance_testnet_order_book(&book)?;
    let next_sequence = sequence
        .checked_add(1)
        .filter(|next| *next <= MAX_SEQUENCE as i64)
        .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
    book.state_version =
        binance_testnet_order_book_version(&book.connection_id, next_sequence as u64);
    let encoded = serde_json::to_string(&book).map_err(storage_error)?;
    tx.execute(
        "INSERT INTO binance_testnet_order_books(workspace_id,connection_id,sequence,projection) VALUES(?1,?2,?3,?4) ON CONFLICT(workspace_id,connection_id) DO UPDATE SET sequence=excluded.sequence,projection=excluded.projection",
        params![workspace_id, book.connection_id, next_sequence, encoded],
    )
    .map_err(storage_error)?;
    let event = write_binance_testnet_order_book_event(tx, &book, next_sequence)?;
    Ok((book, event))
}

fn write_binance_testnet_order_book_event(
    tx: &Transaction<'_>,
    book: &BinanceTestnetOrderBook,
    sequence: i64,
) -> Result<DomainEvent> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    let event = DomainEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "binance.testnet.order.book.changed".into(),
        schema_version: 1,
        occurred_at: book.observed_at.clone(),
        aggregate_type: "binance-testnet-order-book".into(),
        aggregate_id: book.connection_id.clone(),
        sequence: u64::try_from(sequence).map_err(storage_error)?,
        payload: DomainProjection::BinanceTestnetOrderBook(Box::new(book.clone())),
    };
    tx.execute(
        "INSERT INTO outbox VALUES(?1,?2,?3,?4,?5)",
        params![
            event.aggregate_type,
            event.aggregate_id,
            sequence,
            event.event_id,
            serde_json::to_string(&event).map_err(storage_error)?,
        ],
    )
    .map_err(storage_error)?;
    Ok(event)
}

fn load_alpaca_paper_order_book(
    connection: &Connection,
    workspace_id: &str,
    connection_id: &str,
) -> Result<Option<AlpacaPaperOrderBook>> {
    let row: Option<(i64, String)> = connection.query_row(
        "SELECT sequence,projection FROM alpaca_paper_order_books WHERE workspace_id=?1 AND connection_id=?2",
        params![workspace_id, connection_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional().map_err(storage_error)?;
    let Some((sequence, projection)) = row else {
        return Ok(None);
    };
    let book: AlpacaPaperOrderBook = serde_json::from_str(&projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if sequence < 1
        || book.workspace_id != workspace_id
        || book.connection_id != connection_id
        || book.state_version != alpaca_order_book_version(connection_id, sequence as u64)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    validate_alpaca_order_book(&book)?;
    Ok(Some(book))
}

fn write_alpaca_order_book_event(
    tx: &Transaction<'_>,
    book: &AlpacaPaperOrderBook,
    sequence: i64,
) -> Result<DomainEvent> {
    if sequence < 1 || sequence > MAX_SEQUENCE as i64 {
        return Err(TradeXError::new("WORKSPACE_OPEN_FAILED"));
    }
    let event = DomainEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: "alpaca.paper.order.book.changed".into(),
        schema_version: 1,
        occurred_at: book.observed_at.clone(),
        aggregate_type: "alpaca-paper-order-book".into(),
        aggregate_id: book.connection_id.clone(),
        sequence: u64::try_from(sequence).map_err(storage_error)?,
        payload: DomainProjection::AlpacaPaperOrderBook(Box::new(book.clone())),
    };
    tx.execute(
        "INSERT INTO outbox VALUES(?1,?2,?3,?4,?5)",
        params![
            event.aggregate_type,
            event.aggregate_id,
            sequence,
            event.event_id,
            serde_json::to_string(&event).map_err(storage_error)?,
        ],
    )
    .map_err(storage_error)?;
    Ok(event)
}

fn load_alpaca_paper_attempt(
    connection: &Connection,
    workspace_id: &str,
    proposal_id: &str,
) -> Result<Option<AlpacaPaperOrderAttempt>> {
    let row: Option<(String, String, String, i64, String, String)> = connection.query_row(
        "SELECT attempt_id,connection_id,client_order_id,sequence,state,projection FROM alpaca_paper_order_attempts WHERE workspace_id=?1 AND proposal_id=?2",
        params![workspace_id, proposal_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
    ).optional().map_err(storage_error)?;
    let Some((attempt_id, connection_id, client_order_id, sequence, state, projection)) = row
    else {
        return Ok(None);
    };
    let attempt: AlpacaPaperOrderAttempt = serde_json::from_str(&projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if sequence < 1
        || sequence > MAX_SEQUENCE as i64
        || attempt.attempt_id != attempt_id
        || attempt.workspace_id != workspace_id
        || attempt.connection_id != connection_id
        || attempt.proposal_id != proposal_id
        || attempt.client_order_id != client_order_id
        || state != alpaca_attempt_state_name(attempt.state)
        || attempt.state_version != format!("alpaca-paper-attempt:{attempt_id}:{sequence}")
        || !valid_proposal_hash(&attempt.proposal_hash)
        || !valid_order_text(&attempt.remote_account_id, 128)
        || !valid_order_text(&attempt.client_order_id, 128)
        || !valid_order_text(&attempt.reason, 256)
        || !valid_order_text(&attempt.created_at, 64)
        || !valid_order_text(&attempt.updated_at, 64)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(Some(attempt))
}

fn load_binance_testnet_attempt(
    connection: &Connection,
    workspace_id: &str,
    proposal_id: &str,
) -> Result<Option<BinanceTestnetOrderAttempt>> {
    let row: Option<(String, String, String, i64, String, String)> = connection.query_row(
        "SELECT attempt_id,connection_id,client_order_id,sequence,state,projection FROM binance_testnet_order_attempts WHERE workspace_id=?1 AND proposal_id=?2",
        params![workspace_id, proposal_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
    ).optional().map_err(storage_error)?;
    let Some((attempt_id, connection_id, client_order_id, sequence, state, projection)) = row
    else {
        return Ok(None);
    };
    let attempt: BinanceTestnetOrderAttempt = serde_json::from_str(&projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if sequence < 1
        || sequence > MAX_SEQUENCE as i64
        || attempt.attempt_id != attempt_id
        || attempt.workspace_id != workspace_id
        || attempt.connection_id != connection_id
        || attempt.proposal_id != proposal_id
        || attempt.environment != "TESTNET"
        || attempt.client_order_id != client_order_id
        || state != binance_attempt_state_name(attempt.state)
        || attempt.state_version != format!("binance-testnet-attempt:{attempt_id}:{sequence}")
        || !valid_proposal_hash(&attempt.proposal_hash)
        || !valid_order_text(&attempt.remote_account_id, 128)
        || !valid_order_text(&attempt.client_order_id, 36)
        || !valid_order_text(&attempt.reason, 256)
        || !valid_order_text(&attempt.created_at, 64)
        || !valid_order_text(&attempt.updated_at, 64)
        || attempt.provider_order_id.as_deref().is_some_and(|value| {
            value.is_empty() || value.len() > 20 || !value.bytes().all(|byte| byte.is_ascii_digit())
        })
        || attempt
            .error_code
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 64))
        || attempt
            .provider_status
            .as_deref()
            .is_some_and(|value| !valid_order_text(value, 64))
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(Some(attempt))
}

fn load_bitget_demo_attempt(
    connection: &Connection,
    workspace_id: &str,
    proposal_id: &str,
) -> Result<Option<BitgetDemoOrderAttempt>> {
    let row: Option<(String, String, String, i64, String, String)> = connection.query_row(
        "SELECT attempt_id,connection_id,client_oid,sequence,state,projection FROM bitget_demo_order_attempts WHERE workspace_id=?1 AND proposal_id=?2",
        params![workspace_id, proposal_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
    ).optional().map_err(storage_error)?;
    let Some((attempt_id, connection_id, client_oid, sequence, state, projection)) = row else {
        return Ok(None);
    };
    let attempt: BitgetDemoOrderAttempt = serde_json::from_str(&projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if sequence < 1
        || sequence > MAX_SEQUENCE as i64
        || attempt.attempt_id != attempt_id
        || attempt.workspace_id != workspace_id
        || attempt.connection_id != connection_id
        || attempt.proposal_id != proposal_id
        || attempt.client_oid != client_oid
        || state != bitget_demo_attempt_state_name(attempt.state)
        || attempt.state_version != format!("bitget-demo-order-attempt:{attempt_id}:{sequence}")
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    validate_bitget_demo_attempt(&attempt)?;
    Ok(Some(attempt))
}

fn load_trading212_demo_attempt(
    connection: &Connection,
    workspace_id: &str,
    proposal_id: &str,
) -> Result<Option<Trading212DemoOrderAttempt>> {
    let row: Option<(String, String, i64, String, String)> = connection
        .query_row(
            "SELECT attempt_id,connection_id,sequence,state,projection FROM trading212_demo_order_attempts WHERE workspace_id=?1 AND proposal_id=?2",
            params![workspace_id, proposal_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .optional()
        .map_err(storage_error)?;
    let Some((attempt_id, connection_id, sequence, state, projection)) = row else {
        return Ok(None);
    };
    let attempt: Trading212DemoOrderAttempt = serde_json::from_str(&projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if sequence < 1
        || sequence > MAX_SEQUENCE as i64
        || attempt.attempt_id != attempt_id
        || attempt.workspace_id != workspace_id
        || attempt.connection_id != connection_id
        || attempt.proposal_id != proposal_id
        || state != trading212_demo_attempt_state_name(attempt.state)
        || attempt.state_version != trading212_demo_attempt_version(&attempt_id, sequence as u64)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    validate_trading212_demo_attempt(&attempt)?;
    Ok(Some(attempt))
}

fn valid_proposal_hash(hash: &str) -> bool {
    let Some(hex) = hash.strip_prefix("sha256:") else {
        return false;
    };
    valid_lower_hex(hex, 64)
}

fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn materialize_order_proposal(
    connection: &Connection,
    stored: StoredOrderProposal,
) -> Result<OrderProposal> {
    let (status, invalidation_reason, last_event_sequence) =
        proposal_event_state(connection, &stored.proposal_id, &stored.workspace_id)?;
    let history = proposal_history(connection, &stored.proposal_id, &stored.workspace_id)?;
    Ok(OrderProposal {
        proposal_id: stored.proposal_id.clone(),
        workspace_id: stored.workspace_id,
        draft_id: stored.draft_id,
        draft_version: stored.draft_version,
        proposal_hash: stored.proposal_hash,
        fields: stored.fields,
        estimated_notional: stored.estimated_notional,
        estimated_notional_currency: stored.estimated_notional_currency,
        estimated_notional_reason: stored.estimated_notional_reason,
        policy_version: stored.policy_version,
        policy_state_version: stored.policy_state_version,
        policy_status: stored.policy_status,
        policy_reference_reason: stored.policy_reference_reason,
        market_snapshot_id: stored.market_snapshot_id,
        market_status: stored.market_status,
        market_reference_reason: stored.market_reference_reason,
        status,
        invalidation_reason,
        created_at: stored.created_at,
        state_version: format!(
            "order-proposal:{}:{last_event_sequence}",
            stored.proposal_id
        ),
        history,
    })
}

fn refreshed_replacement_id(reason: &str) -> Result<&str> {
    let rest = reason
        .strip_prefix("Proposal refreshed as ")
        .ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    let (proposal_id, suffix) = rest
        .split_once(';')
        .ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if suffix.trim() != "a new approval is required." {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let proposal_id = proposal_id.trim();
    if !valid_order_proposal_id(proposal_id) {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(proposal_id)
}

fn validate_refreshed_replacement(
    connection: &Connection,
    proposal_id: &str,
    workspace_id: &str,
    reason: &str,
) -> Result<()> {
    let replacement_id = refreshed_replacement_id(reason)?;
    if replacement_id == proposal_id {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let (row_workspace_id, draft_id, draft_version, proposal_hash, sequence, projection): (
        String,
        String,
        i64,
        String,
        i64,
        String,
    ) = connection
        .query_row(
            "SELECT workspace_id,draft_id,draft_version,proposal_hash,sequence,projection FROM order_proposals WHERE proposal_id=?1",
            [replacement_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .map_err(|error| {
            if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                TradeXError::new("WORKSPACE_INTEGRITY_FAILED")
            } else {
                storage_error(error)
            }
        })?;
    if row_workspace_id != workspace_id {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let _ = decode_stored_order_proposal(
        &projection,
        replacement_id,
        &row_workspace_id,
        &draft_id,
        draft_version,
        &proposal_hash,
        sequence,
        workspace_id,
    )?;
    let (first_sequence, first_event): (i64, String) = connection
        .query_row(
            "SELECT sequence,event FROM order_proposal_events WHERE proposal_id=?1 ORDER BY sequence LIMIT 1",
            [replacement_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if first_sequence != 1 || first_event != "GENERATED" {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(())
}

fn proposal_event_state(
    connection: &Connection,
    proposal_id: &str,
    workspace_id: &str,
) -> Result<(OrderProposalStatus, Option<String>, i64)> {
    let mut query = connection
        .prepare(
            "SELECT workspace_id,sequence,event,reason FROM order_proposal_events WHERE proposal_id=?1 ORDER BY sequence",
        )
        .map_err(storage_error)?;
    let rows = query
        .query_map([proposal_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(storage_error)?;
    let mut status = OrderProposalStatus::NeedsApproval;
    let mut invalidation_reason = None;
    let mut last_sequence = 0_i64;
    for row in rows {
        let (event_workspace_id, sequence, event, reason) = row.map_err(storage_error)?;
        if event_workspace_id != workspace_id {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        if sequence != last_sequence + 1 || !(1..=MAX_SEQUENCE as i64).contains(&sequence) {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        match event.as_str() {
            "GENERATED" if sequence == 1 && reason.is_none() => {}
            "CONSUMED" if sequence > 1 && status == OrderProposalStatus::NeedsApproval => {
                status = OrderProposalStatus::Consumed;
                invalidation_reason =
                    Some(reason.ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?);
            }
            "DRAFT_CHANGED" if sequence > 1 && status == OrderProposalStatus::NeedsApproval => {
                status = OrderProposalStatus::Invalidated;
                invalidation_reason =
                    Some(reason.ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?);
            }
            "REFRESHED" if sequence > 1 && status == OrderProposalStatus::NeedsApproval => {
                let reason =
                    reason.ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
                validate_refreshed_replacement(connection, proposal_id, workspace_id, &reason)?;
                status = OrderProposalStatus::Invalidated;
                invalidation_reason = Some(reason);
            }
            _ => return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED")),
        }
        last_sequence = sequence;
    }
    if last_sequence == 0 {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok((status, invalidation_reason, last_sequence))
}

fn proposal_history(
    connection: &Connection,
    proposal_id: &str,
    workspace_id: &str,
) -> Result<Vec<OrderProposalHistoryEntry>> {
    let mut query = connection
        .prepare(
            "SELECT workspace_id,event,reason,occurred_at,sequence FROM order_proposal_events WHERE proposal_id=?1 ORDER BY sequence",
        )
        .map_err(storage_error)?;
    let rows = query
        .query_map([proposal_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })
        .map_err(storage_error)?;
    let mut history = Vec::new();
    for row in rows {
        let (event_workspace_id, event, reason, occurred_at, sequence) =
            row.map_err(storage_error)?;
        if event_workspace_id != workspace_id {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        let event = match event.as_str() {
            "GENERATED" => OrderProposalHistoryEvent::Generated,
            "DRAFT_CHANGED" => OrderProposalHistoryEvent::DraftChanged,
            "REFRESHED" => OrderProposalHistoryEvent::Refreshed,
            "CONSUMED" => OrderProposalHistoryEvent::Consumed,
            _ => return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED")),
        };
        if !(1..=MAX_SEQUENCE as i64).contains(&sequence)
            || !valid_order_text(&occurred_at, 64)
            || reason
                .as_deref()
                .is_some_and(|value| !valid_order_text(value, 128))
            || history.len() >= 32
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        history.push(OrderProposalHistoryEntry {
            event,
            reason,
            occurred_at,
        });
    }
    Ok(history)
}

fn next_order_proposal_sequence(tx: &rusqlite::Transaction<'_>, workspace_id: &str) -> Result<i64> {
    let max_sequence: Option<i64> = tx
        .query_row(
            "SELECT MAX(sequence) FROM order_proposals WHERE workspace_id=?1",
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

fn proposal_sequence(tx: &rusqlite::Transaction<'_>, proposal_id: &str) -> Result<i64> {
    tx.query_row(
        "SELECT sequence FROM order_proposals WHERE proposal_id=?1",
        [proposal_id],
        |row| row.get(0),
    )
    .map_err(storage_error)
}

fn invalidate_order_proposals_tx(
    tx: &rusqlite::Transaction<'_>,
    workspace_id: &str,
    draft_id: &str,
    reason: &str,
    occurred_at: &str,
) -> Result<()> {
    let mut query = tx
        .prepare("SELECT proposal_id FROM order_proposals WHERE workspace_id=?1 AND draft_id=?2")
        .map_err(storage_error)?;
    let ids = query
        .query_map(params![workspace_id, draft_id], |row| {
            row.get::<_, String>(0)
        })
        .map_err(storage_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(storage_error)?;
    drop(query);
    for proposal_id in ids {
        let active: Option<String> = tx
            .query_row(
                "SELECT event FROM order_proposal_events WHERE proposal_id=?1 ORDER BY sequence DESC LIMIT 1",
                [&proposal_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        if matches!(
            active.as_deref(),
            Some("DRAFT_CHANGED" | "REFRESHED" | "CONSUMED")
        ) {
            continue;
        }
        let last_sequence: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(sequence),0) FROM order_proposal_events WHERE proposal_id=?1",
                [&proposal_id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        let next_sequence = last_sequence
            .checked_add(1)
            .filter(|value| *value <= 32)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        tx.execute(
            "INSERT INTO order_proposal_events(proposal_id,workspace_id,sequence,event,reason,occurred_at) VALUES(?1,?2,?3,?4,?5,?6)",
            params![proposal_id, workspace_id, next_sequence, "DRAFT_CHANGED", reason, occurred_at],
        )
        .map_err(storage_error)?;
    }
    Ok(())
}

pub(crate) fn validate_order_proposal_id(id: &str) -> Result<()> {
    if !valid_order_proposal_id(id) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    Ok(())
}

fn decode_order_draft(
    projection: &str,
    row_id: &str,
    row_workspace_id: &str,
    row_draft_version: i64,
    sequence: i64,
    workspace_id: &str,
) -> Result<OrderDraft> {
    if sequence < 1
        || sequence > MAX_SEQUENCE as i64
        || !(1..=9_007_199_254_740_991).contains(&row_draft_version)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let draft: OrderDraft = serde_json::from_str(projection)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    let mut normalized = draft.fields.clone();
    if normalize_order_draft_fields(&mut normalized).is_err()
        || normalized != draft.fields
        || draft.draft_id != row_id
        || draft.workspace_id != row_workspace_id
        || row_workspace_id != workspace_id
        || draft.draft_version != row_draft_version as u64
        || draft.state_version != format!("order-draft:{row_id}:{sequence}")
        || !valid_order_text(&draft.updated_at, 64)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(draft)
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
    if content.research_result.as_ref().is_some_and(|result| {
        result.payload.market_snapshot_refs.len() > 8
            || result.payload.dataset_refs.len() > 8
            || result.payload.order_refs.len() > 8
            || result
                .payload
                .market_snapshot_refs
                .iter()
                .chain(result.payload.dataset_refs.iter())
                .chain(result.payload.order_refs.iter())
                .any(|reference| {
                    reference.is_empty()
                        || reference.len() > 128
                        || reference.chars().any(char::is_control)
                })
    }) {
        return Err(TradeXError::new("ARTIFACT_SOURCE_INVALID"));
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
        "\"accesstoken\":",
        "\"refreshtoken\":",
        "\"privatekey\":",
        "\"clientsecret\":",
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
                    | "accesstoken"
                    | "refreshtoken"
                    | "privatekey"
                    | "clientsecret"
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

fn next_strategy_sequence(tx: &rusqlite::Transaction<'_>, workspace_id: &str) -> Result<i64> {
    let previous: i64 = tx
        .query_row(
            "SELECT MAX(sequence) FROM (SELECT sequence FROM strategy_versions WHERE workspace_id=?1 UNION ALL SELECT sequence FROM strategy_runs WHERE workspace_id=?1)",
            [workspace_id],
            |row| row.get::<_, Option<i64>>(0),
        )
        .map_err(storage_error)?
        .unwrap_or(0);
    if !(0..MAX_SEQUENCE as i64).contains(&previous) {
        return Err(TradeXError::new("STRATEGY_VERSION_LIMIT"));
    }
    Ok(previous + 1)
}

fn next_backtest_sequence(tx: &rusqlite::Transaction<'_>, workspace_id: &str) -> Result<i64> {
    let previous: i64 = tx
        .query_row(
            "SELECT MAX(sequence) FROM backtest_runs WHERE workspace_id=?1",
            [workspace_id],
            |row| row.get::<_, Option<i64>>(0),
        )
        .map_err(storage_error)?
        .unwrap_or(0);
    if !(0..MAX_SEQUENCE as i64).contains(&previous) {
        return Err(TradeXError::new("BACKTEST_RUN_LIMIT"));
    }
    Ok(previous + 1)
}

fn valid_strategy_timestamp(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 64
        && !value.chars().any(char::is_control)
        && OffsetDateTime::parse(value, &Rfc3339).is_ok()
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
        if !parent.is_dir() || has_unapproved_symlink_ancestor(parent) {
            return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
        }
        let canonical_parent = parent
            .canonicalize()
            .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"))?;
        let requested_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"))?;
        let file_name = artifact_export_file_name(Some(requested_name), artifact_id)?;
        return Ok(canonical_parent.join(file_name));
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
    if !exports.is_dir() || has_unapproved_symlink_ancestor(&exports) {
        return Err(TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"));
    }
    let canonical_exports = exports
        .canonicalize()
        .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"))?;
    Ok(canonical_exports.join(file_name))
}

fn has_unapproved_symlink_ancestor(path: &Path) -> bool {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => current.push(prefix.as_os_str()),
            Component::RootDir => current.push(Path::new("/")),
            Component::Normal(part) => {
                current.push(part);
                if fs::symlink_metadata(&current)
                    .map(|metadata| metadata.file_type().is_symlink())
                    .unwrap_or(false)
                    && !matches!(current.to_str(), Some("/var") | Some("/tmp"))
                {
                    return true;
                }
            }
            Component::CurDir | Component::ParentDir => return true,
        }
    }
    false
}

fn write_export_file(
    parent: &Path,
    destination_name: &str,
    temporary_name: &str,
    encoded: &[u8],
) -> Result<()> {
    #[cfg(unix)]
    {
        let parent_fd = open_directory_nofollow(parent)
            .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_FAILED"))?;
        let result = write_export_at(parent_fd, destination_name, temporary_name, encoded);
        unsafe {
            libc::close(parent_fd);
        }
        result
    }
    #[cfg(not(unix))]
    {
        let temporary = parent.join(temporary_name);
        let write_result = (|| -> Result<()> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_FAILED"))?;
            file.write_all(encoded)
                .and_then(|_| file.sync_all())
                .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_FAILED"))?;
            fs::hard_link(&temporary, parent.join(destination_name)).map_err(|error| {
                if error.kind() == ErrorKind::AlreadyExists {
                    TradeXError::new("ARTIFACT_EXPORT_EXISTS")
                } else {
                    TradeXError::new("ARTIFACT_EXPORT_FAILED")
                }
            })?;
            let _ = fs::remove_file(&temporary);
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        write_result
    }
}

#[cfg(unix)]
fn open_directory_nofollow(path: &Path) -> std::io::Result<RawFd> {
    let root = CString::new("/").expect("static root path has no NUL");
    let mut descriptor = unsafe {
        libc::open(
            root.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    for component in path.components() {
        let Component::Normal(part) = component else {
            continue;
        };
        let name = CString::new(part.as_bytes())
            .map_err(|_| std::io::Error::from(ErrorKind::InvalidInput))?;
        let next = unsafe {
            libc::openat(
                descriptor,
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if next < 0 {
            let error = std::io::Error::last_os_error();
            unsafe {
                libc::close(descriptor);
            }
            return Err(error);
        }
        unsafe {
            libc::close(descriptor);
        }
        descriptor = next;
    }
    Ok(descriptor)
}

#[cfg(unix)]
fn write_export_at(
    parent_fd: RawFd,
    destination_name: &str,
    temporary_name: &str,
    encoded: &[u8],
) -> Result<()> {
    let temporary = CString::new(temporary_name)
        .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"))?;
    let destination = CString::new(destination_name)
        .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_PATH_INVALID"))?;
    let temporary_fd = unsafe {
        libc::openat(
            parent_fd,
            temporary.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0o600,
        )
    };
    if temporary_fd < 0 {
        return Err(TradeXError::new("ARTIFACT_EXPORT_FAILED"));
    }
    let mut file = unsafe { File::from_raw_fd(temporary_fd) };
    let write_result = file
        .write_all(encoded)
        .and_then(|_| file.sync_all())
        .map_err(|_| TradeXError::new("ARTIFACT_EXPORT_FAILED"));
    drop(file);
    if let Err(error) = write_result {
        unsafe {
            libc::unlinkat(parent_fd, temporary.as_ptr(), 0);
        }
        return Err(error);
    }
    let linked = unsafe {
        libc::linkat(
            parent_fd,
            temporary.as_ptr(),
            parent_fd,
            destination.as_ptr(),
            0,
        )
    };
    if linked != 0 {
        let error = std::io::Error::last_os_error();
        unsafe {
            libc::unlinkat(parent_fd, temporary.as_ptr(), 0);
        }
        return if error.kind() == ErrorKind::AlreadyExists {
            Err(TradeXError::new("ARTIFACT_EXPORT_EXISTS"))
        } else {
            Err(TradeXError::new("ARTIFACT_EXPORT_FAILED"))
        };
    }
    unsafe {
        libc::unlinkat(parent_fd, temporary.as_ptr(), 0);
    }
    Ok(())
}

fn backfill_local_paper_tables(connection: &mut Connection) -> Result<()> {
    let rows = {
        let mut statement = connection
            .prepare("SELECT workspace_id,projection FROM paper_state")
            .map_err(storage_error)?;
        statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(storage_error)?
    };
    if rows.is_empty() {
        return Ok(());
    }
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    for (workspace_id, encoded) in rows {
        let state: LocalPaperState = serde_json::from_str(&encoded).map_err(storage_error)?;
        for order in &state.orders {
            let idempotency_key = order
                .idempotency_key
                .clone()
                .unwrap_or_else(|| format!("legacy:{}", order.order_id));
            tx.execute(
                "INSERT OR IGNORE INTO paper_orders(workspace_id,order_id,proposal_id,idempotency_key,projection) VALUES(?1,?2,?3,?4,?5)",
                params![
                    &workspace_id,
                    &order.order_id,
                    &order.proposal_id,
                    &idempotency_key,
                    serde_json::to_string(order).map_err(storage_error)?,
                ],
            )
            .map_err(storage_error)?;
        }
        for fill in &state.fills {
            tx.execute(
                "INSERT OR IGNORE INTO paper_fills(workspace_id,fill_id,order_id,projection) VALUES(?1,?2,?3,?4)",
                params![
                    &workspace_id,
                    &fill.fill_id,
                    &fill.order_id,
                    serde_json::to_string(fill).map_err(storage_error)?,
                ],
            )
            .map_err(storage_error)?;
        }
        for event in &state.events {
            tx.execute(
                "INSERT OR IGNORE INTO paper_events(workspace_id,account_id,event_id,sequence,order_id,fill_id,kind,occurred_at,state_version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    &workspace_id,
                    &state.account_id,
                    &event.event_id,
                    event.sequence as i64,
                    &event.order_id,
                    &event.fill_id,
                    paper_event_kind_name(event.kind),
                    &event.occurred_at,
                    &event.state_version,
                ],
            )
            .map_err(storage_error)?;
        }
    }
    tx.commit().map_err(storage_error)
}

fn persist_local_paper_events_and_state(
    tx: &rusqlite::Transaction<'_>,
    workspace_id: &str,
    previous_event_cursor: u64,
    next_state: &LocalPaperState,
) -> Result<()> {
    for event in next_state
        .events
        .iter()
        .filter(|event| event.sequence > previous_event_cursor)
    {
        tx.execute(
            "INSERT INTO paper_events(workspace_id,account_id,event_id,sequence,order_id,fill_id,kind,occurred_at,state_version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                workspace_id,
                &next_state.account_id,
                &event.event_id,
                event.sequence as i64,
                &event.order_id,
                &event.fill_id,
                paper_event_kind_name(event.kind),
                &event.occurred_at,
                &event.state_version,
            ],
        )
        .map_err(storage_error)?;
    }
    let mut persisted_state = next_state.clone();
    persisted_state.orders.clear();
    persisted_state.fills.clear();
    persisted_state.events.clear();
    persisted_state.open_orders.clear();
    let encoded = serde_json::to_string(&persisted_state).map_err(storage_error)?;
    let changed = tx
        .execute(
            "UPDATE paper_state SET account_id=?1,sequence=?2,projection=?3 WHERE workspace_id=?4 AND account_id=?1",
            params![
                &next_state.account_id,
                next_state.event_cursor.max(1) as i64,
                encoded,
                workspace_id,
            ],
        )
        .map_err(storage_error)?;
    if changed != 1 {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(())
}

fn paper_event_kind_name(kind: LocalPaperEventKind) -> &'static str {
    match kind {
        LocalPaperEventKind::Accepted => "ACCEPTED",
        LocalPaperEventKind::PartiallyFilled => "PARTIALLY_FILLED",
        LocalPaperEventKind::Filled => "FILLED",
        LocalPaperEventKind::Rejected => "REJECTED",
        LocalPaperEventKind::Cancelled => "CANCELLED",
        LocalPaperEventKind::ScenarioChanged => "SCENARIO_CHANGED",
        LocalPaperEventKind::QuoteRefreshed => "QUOTE_REFRESHED",
    }
}

fn paper_event_kind(value: &str) -> Result<LocalPaperEventKind> {
    match value {
        "ACCEPTED" => Ok(LocalPaperEventKind::Accepted),
        "PARTIALLY_FILLED" => Ok(LocalPaperEventKind::PartiallyFilled),
        "FILLED" => Ok(LocalPaperEventKind::Filled),
        "REJECTED" => Ok(LocalPaperEventKind::Rejected),
        "CANCELLED" => Ok(LocalPaperEventKind::Cancelled),
        "SCENARIO_CHANGED" => Ok(LocalPaperEventKind::ScenarioChanged),
        "QUOTE_REFRESHED" => Ok(LocalPaperEventKind::QuoteRefreshed),
        _ => Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED")),
    }
}

fn load_local_paper_tables(
    connection: &Connection,
    workspace_id: &str,
    mut state: LocalPaperState,
) -> Result<LocalPaperState> {
    let orders = connection
        .prepare("SELECT projection FROM paper_orders WHERE workspace_id=?1 ORDER BY rowid")
        .map_err(storage_error)?
        .query_map([workspace_id], |row| row.get::<_, String>(0))
        .map_err(storage_error)?
        .map(|row| {
            let encoded = row.map_err(storage_error)?;
            serde_json::from_str::<LocalPaperOrder>(&encoded).map_err(storage_error)
        })
        .collect::<Result<Vec<_>>>()?;
    let fills = connection
        .prepare("SELECT projection FROM paper_fills WHERE workspace_id=?1 ORDER BY rowid")
        .map_err(storage_error)?
        .query_map([workspace_id], |row| row.get::<_, String>(0))
        .map_err(storage_error)?
        .map(|row| {
            let encoded = row.map_err(storage_error)?;
            serde_json::from_str::<LocalPaperFill>(&encoded).map_err(storage_error)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut statement = connection
        .prepare("SELECT account_id,event_id,sequence,kind,order_id,fill_id,occurred_at,state_version FROM paper_events WHERE workspace_id=?1 ORDER BY sequence")
        .map_err(storage_error)?;
    let events = statement
        .query_map([workspace_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .map_err(storage_error)?
        .map(|row| {
            let (
                account_id,
                event_id,
                sequence,
                kind,
                order_id,
                fill_id,
                occurred_at,
                state_version,
            ) = row.map_err(storage_error)?;
            if account_id != state.account_id {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            let sequence = u64::try_from(sequence).map_err(storage_error)?;
            Ok(LocalPaperEvent {
                event_id,
                sequence,
                kind: paper_event_kind(&kind)?,
                order_id,
                fill_id,
                occurred_at,
                state_version,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    state.orders = orders;
    state.fills = fills;
    state.events = events;
    state.open_orders = state
        .orders
        .iter()
        .filter(|order| {
            matches!(
                order.state,
                crate::protocol::LocalPaperOrderState::Accepted
                    | crate::protocol::LocalPaperOrderState::PartiallyFilled
                    | crate::protocol::LocalPaperOrderState::CancelPending
            )
        })
        .cloned()
        .collect();
    Ok(state)
}

fn read_workspace(connection: &Connection, path: &Path) -> Result<Workspace> {
    connection.query_row("SELECT workspace_id,name,created_at,last_opened_at,base_currency FROM workspace WHERE singleton=1", [], |row| Ok(Workspace {
        workspace_id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, last_opened_at: row.get(3)?,
        base_currency: row.get(4)?, path: path.to_string_lossy().into_owned(), storage_schema_version: SCHEMA_VERSION,
    })).map_err(storage_error)
}

fn validate_local_paper_state(
    state: &LocalPaperState,
    workspace_id: &str,
    account_id: &str,
) -> Result<()> {
    let base_currency = state.profile.base_currency.as_str();
    let money = [
        (&state.cash, true),
        (&state.reserved_cash, true),
        (&state.equity, true),
        (&state.realized_pnl, false),
        (&state.unrealized_pnl, false),
        (&state.exposure, true),
    ];
    if state.workspace_id != workspace_id
        || state.account_id != account_id
        || state.provider_id != crate::paper::PROVIDER_ID
        || state.environment != crate::paper::ENVIRONMENT
        || !valid_local_paper_text(&state.account_label, 120)
        || !valid_local_paper_text(&state.profile.quote_source, 64)
        || state
            .profile
            .quote_price
            .as_deref()
            .is_some_and(|value| !valid_local_paper_decimal(value, true))
        || state
            .profile
            .quote_freshness
            .as_deref()
            .is_some_and(|value| !valid_local_paper_text(value, 32))
        || state
            .profile
            .quote_observed_at
            .as_deref()
            .is_some_and(|value| !valid_local_paper_timestamp(value))
        || !valid_local_paper_text(&state.profile.scenario_id, 128)
        || !crate::paper::scenario_supported(&state.profile.scenario_id)
        || !valid_local_paper_text(&state.profile.engine_version, 64)
        || !valid_local_paper_text(&state.profile.scenario_seed, 64)
        || state.profile.scenario_seed != crate::paper::SCENARIO_SEED
        || !valid_local_paper_text(&state.profile.scenario_version, 64)
        || state.profile.scenario_version != crate::paper::SCENARIO_VERSION
        || !valid_local_paper_text(&state.profile.fee_policy, 64)
        || state.profile.fee_policy != crate::paper::FEE_POLICY
        || !valid_local_paper_text(&state.profile.slippage_policy, 64)
        || state.profile.slippage_policy != crate::paper::SLIPPAGE_POLICY
        || !valid_local_paper_text(&state.profile.fill_policy, 64)
        || state.profile.fill_policy != crate::paper::FILL_POLICY
        || !valid_local_paper_text(&state.state_version, 256)
        || !valid_local_paper_text(&state.updated_at, 64)
        || !valid_local_paper_text(&state.disclosure, 256)
        || !valid_local_paper_currency(base_currency)
        || !valid_local_paper_decimal(&state.profile.starting_cash, true)
        || state.profile.starting_cash != crate::paper::DEFAULT_STARTING_CASH
        || money.iter().any(|(value, nonnegative)| {
            value.currency != base_currency
                || !valid_local_paper_currency(&value.currency)
                || !valid_local_paper_decimal(&value.value, *nonnegative)
        })
        || state.balances.iter().any(|balance| {
            !valid_local_paper_text(&balance.asset, 64)
                || !valid_local_paper_decimal(&balance.available, true)
                || !valid_local_paper_decimal(&balance.total, true)
                || !valid_local_paper_decimal(&balance.reserved, true)
        })
        || state.orders.iter().any(|order| {
            !valid_local_paper_text(&order.order_id, 128)
                || !valid_local_paper_text(&order.proposal_id, 128)
                || !valid_local_paper_hash(&order.proposal_hash)
                || !valid_local_paper_text(&order.instrument_id, 128)
                || !valid_local_paper_decimal(&order.requested_quantity, true)
                || !valid_local_paper_decimal(&order.filled_quantity, true)
                || !valid_local_paper_decimal(&order.remaining_quantity, true)
                || order
                    .limit_price
                    .as_deref()
                    .is_some_and(|value| !valid_local_paper_decimal(value, true))
                || order
                    .average_fill_price
                    .as_deref()
                    .is_some_and(|value| !valid_local_paper_decimal(value, true))
                || order
                    .idempotency_key
                    .as_deref()
                    .is_some_and(|value| !valid_local_paper_text(value, 128))
                || order
                    .cancel_idempotency_key
                    .as_deref()
                    .is_some_and(|value| !valid_local_paper_text(value, 128))
                || order.quote.as_ref().is_some_and(|quote| {
                    !valid_local_paper_text(&quote.quote_id, 128)
                        || !valid_local_paper_text(&quote.instrument_id, 128)
                        || !valid_local_paper_decimal(&quote.price, true)
                        || !valid_local_paper_currency(&quote.currency)
                        || !valid_local_paper_text(&quote.observed_at, 64)
                        || !valid_local_paper_text(&quote.scenario_id, 128)
                        || !valid_local_paper_text(&quote.source, 64)
                        || !valid_local_paper_text(&quote.freshness, 32)
                })
                || !valid_local_paper_text(&order.created_at, 64)
                || !valid_local_paper_text(&order.updated_at, 64)
        })
        || state.positions.iter().any(|position| {
            !valid_local_paper_text(&position.instrument_id, 128)
                || !valid_local_paper_decimal(&position.quantity, true)
                || !valid_local_paper_decimal(&position.average_entry_price, true)
                || position
                    .market_value
                    .as_deref()
                    .is_some_and(|value| !valid_local_paper_decimal(value, true))
                || !valid_local_paper_currency(&position.currency)
                || position
                    .unrealized_pnl
                    .as_deref()
                    .is_some_and(|value| !valid_local_paper_decimal(value, false))
        })
        || state.open_orders.iter().any(|order| {
            !valid_local_paper_text(&order.order_id, 128)
                || !valid_local_paper_text(&order.proposal_id, 128)
                || !valid_local_paper_hash(&order.proposal_hash)
                || !valid_local_paper_text(&order.instrument_id, 128)
                || !valid_local_paper_decimal(&order.requested_quantity, true)
                || !valid_local_paper_decimal(&order.filled_quantity, true)
                || !valid_local_paper_decimal(&order.remaining_quantity, true)
                || order
                    .average_fill_price
                    .as_deref()
                    .is_some_and(|value| !valid_local_paper_decimal(value, true))
                || !valid_local_paper_text(&order.created_at, 64)
                || !valid_local_paper_text(&order.updated_at, 64)
                || !matches!(
                    order.state,
                    crate::protocol::LocalPaperOrderState::Accepted
                        | crate::protocol::LocalPaperOrderState::PartiallyFilled
                        | crate::protocol::LocalPaperOrderState::CancelPending
                )
        })
        || state.fills.iter().any(|fill| {
            !valid_local_paper_text(&fill.fill_id, 128)
                || !valid_local_paper_text(&fill.order_id, 128)
                || !valid_local_paper_text(&fill.instrument_id, 128)
                || !valid_local_paper_decimal(&fill.quantity, true)
                || !valid_local_paper_decimal(&fill.price, true)
                || !valid_local_paper_decimal(&fill.value, true)
                || !valid_local_paper_currency(&fill.currency)
                || !valid_local_paper_text(&fill.observed_at, 64)
        })
        || state.balances.len() > 512
        || state.positions.len() > 512
        || state.open_orders.len() > 512
        || state.fills.len() > 512
        || state.orders.len() > 512
        || state.events.len() > 1024
        || state.events.iter().enumerate().any(|(index, event)| {
            event.sequence != (index as u64).saturating_add(1)
                || !valid_local_paper_text(&event.event_id, 128)
                || !valid_local_paper_text(&event.order_id, 128)
                || event
                    .fill_id
                    .as_deref()
                    .is_some_and(|value| !valid_local_paper_text(value, 128))
                || !valid_local_paper_text(&event.occurred_at, 64)
                || !valid_local_paper_text(&event.state_version, 256)
        })
        || state.events.last().map(|event| event.sequence).unwrap_or(0) != state.event_cursor
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    validate_local_paper_ledger(state)
}

fn validate_local_paper_ledger(state: &LocalPaperState) -> Result<()> {
    #[derive(Clone)]
    struct LedgerPosition {
        instrument_id: String,
        quantity: String,
        cost: String,
    }

    let integrity = || TradeXError::new("WORKSPACE_INTEGRITY_FAILED");
    let mut cash = state.profile.starting_cash.clone();
    let mut realized_pnl = "0".to_owned();
    let mut positions: Vec<LedgerPosition> = Vec::new();

    for fill in &state.fills {
        let order = state
            .orders
            .iter()
            .find(|order| order.order_id == fill.order_id)
            .ok_or_else(integrity)?;
        if fill.instrument_id != order.instrument_id
            || fill.side != order.side
            || fill.currency != state.profile.base_currency
        {
            return Err(integrity());
        }
        let expected_value =
            crate::portfolio::decimal_mul(&fill.quantity, &fill.price).map_err(|_| integrity())?;
        if expected_value != fill.value {
            return Err(integrity());
        }
        let position_index = positions
            .iter()
            .position(|position| position.instrument_id == fill.instrument_id);
        match fill.side {
            crate::protocol::OrderSide::Buy => {
                cash = crate::portfolio::decimal_add(&cash, &format!("-{}", fill.value))
                    .map_err(|_| integrity())?;
                if let Some(index) = position_index {
                    let position = &mut positions[index];
                    position.quantity =
                        crate::portfolio::decimal_add(&position.quantity, &fill.quantity)
                            .map_err(|_| integrity())?;
                    position.cost = crate::portfolio::decimal_add(&position.cost, &fill.value)
                        .map_err(|_| integrity())?;
                } else {
                    positions.push(LedgerPosition {
                        instrument_id: fill.instrument_id.clone(),
                        quantity: fill.quantity.clone(),
                        cost: fill.value.clone(),
                    });
                }
            }
            crate::protocol::OrderSide::Sell => {
                let Some(index) = position_index else {
                    return Err(integrity());
                };
                let position = &mut positions[index];
                if compare_local_paper_nonnegative(&position.quantity, &fill.quantity)
                    .map_err(|_| integrity())?
                    == std::cmp::Ordering::Less
                {
                    return Err(integrity());
                }
                let average = crate::portfolio::decimal_div(&position.cost, &position.quantity)
                    .map_err(|_| integrity())?;
                let realized = crate::portfolio::decimal_mul(
                    &crate::portfolio::decimal_add(&fill.price, &format!("-{average}"))
                        .map_err(|_| integrity())?,
                    &fill.quantity,
                )
                .map_err(|_| integrity())?;
                realized_pnl = crate::portfolio::decimal_add(&realized_pnl, &realized)
                    .map_err(|_| integrity())?;
                cash =
                    crate::portfolio::decimal_add(&cash, &fill.value).map_err(|_| integrity())?;
                let sold_cost = crate::portfolio::decimal_mul(&average, &fill.quantity)
                    .map_err(|_| integrity())?;
                position.quantity = crate::portfolio::decimal_add(
                    &position.quantity,
                    &format!("-{}", fill.quantity),
                )
                .map_err(|_| integrity())?;
                position.cost =
                    crate::portfolio::decimal_add(&position.cost, &format!("-{sold_cost}"))
                        .map_err(|_| integrity())?;
                if position.quantity == "0" {
                    positions.remove(index);
                }
            }
        }
    }

    for order in &state.orders {
        let filled = state
            .fills
            .iter()
            .filter(|fill| fill.order_id == order.order_id)
            .try_fold("0".to_owned(), |total, fill| {
                crate::portfolio::decimal_add(&total, &fill.quantity).map_err(|_| integrity())
            })?;
        let expected_remaining =
            crate::portfolio::decimal_add(&order.requested_quantity, &format!("-{filled}"))
                .map_err(|_| integrity())?;
        if filled != order.filled_quantity || expected_remaining != order.remaining_quantity {
            return Err(integrity());
        }
        if compare_local_paper_nonnegative(&order.requested_quantity, &filled)
            .map_err(|_| integrity())?
            == std::cmp::Ordering::Less
        {
            return Err(integrity());
        }
    }

    let mut reserved = "0".to_owned();
    for order in &state.open_orders {
        if matches!(order.side, crate::protocol::OrderSide::Buy) {
            let price = order
                .limit_price
                .as_deref()
                .or_else(|| order.quote.as_ref().map(|quote| quote.price.as_str()))
                .ok_or_else(integrity)?;
            let amount = crate::portfolio::decimal_mul(&order.remaining_quantity, price)
                .map_err(|_| integrity())?;
            reserved =
                crate::portfolio::decimal_add(&reserved, &amount).map_err(|_| integrity())?;
        }
    }
    cash =
        crate::portfolio::decimal_add(&cash, &format!("-{reserved}")).map_err(|_| integrity())?;

    if cash != state.cash.value
        || reserved != state.reserved_cash.value
        || realized_pnl != state.realized_pnl.value
    {
        return Err(integrity());
    }

    let mark_price = state.profile.quote_price.as_deref();
    if !positions.is_empty() && mark_price.is_none() {
        return Err(integrity());
    }
    let mut exposure = "0".to_owned();
    let mut unrealized_pnl = "0".to_owned();
    if let Some(mark_price) = mark_price {
        for expected in &positions {
            let position = state
                .positions
                .iter()
                .find(|position| position.instrument_id == expected.instrument_id)
                .ok_or_else(integrity)?;
            let average = crate::portfolio::decimal_div(&expected.cost, &expected.quantity)
                .map_err(|_| integrity())?;
            let market_value = crate::portfolio::decimal_mul(&expected.quantity, mark_price)
                .map_err(|_| integrity())?;
            let position_pnl = crate::portfolio::decimal_mul(
                &crate::portfolio::decimal_add(mark_price, &format!("-{average}"))
                    .map_err(|_| integrity())?,
                &expected.quantity,
            )
            .map_err(|_| integrity())?;
            if position.quantity != expected.quantity
                || position.average_entry_price != average
                || position.market_value.as_deref() != Some(market_value.as_str())
                || position.unrealized_pnl.as_deref() != Some(position_pnl.as_str())
                || position.currency != state.profile.base_currency
            {
                return Err(integrity());
            }
            exposure =
                crate::portfolio::decimal_add(&exposure, &market_value).map_err(|_| integrity())?;
            unrealized_pnl = crate::portfolio::decimal_add(&unrealized_pnl, &position_pnl)
                .map_err(|_| integrity())?;
        }
    }
    if state.positions.len() != positions.len()
        || exposure != state.exposure.value
        || unrealized_pnl != state.unrealized_pnl.value
    {
        return Err(integrity());
    }
    let equity = crate::portfolio::decimal_add(
        &crate::portfolio::decimal_add(&state.cash.value, &state.reserved_cash.value)
            .map_err(|_| integrity())?,
        &state.exposure.value,
    )
    .map_err(|_| integrity())?;
    if equity != state.equity.value {
        return Err(integrity());
    }
    let balance = state
        .balances
        .iter()
        .find(|balance| balance.asset == state.profile.base_currency)
        .ok_or_else(integrity)?;
    let balance_total =
        crate::portfolio::decimal_add(&state.cash.value, &state.reserved_cash.value)
            .map_err(|_| integrity())?;
    if balance.available != state.cash.value
        || balance.total != balance_total
        || balance.reserved != state.reserved_cash.value
    {
        return Err(integrity());
    }
    Ok(())
}

fn compare_local_paper_nonnegative(left: &str, right: &str) -> Result<std::cmp::Ordering> {
    let difference = crate::portfolio::decimal_add(left, &format!("-{right}"))?;
    Ok(if difference == "0" {
        std::cmp::Ordering::Equal
    } else if difference.starts_with('-') {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Greater
    })
}

fn valid_local_paper_text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.chars().count() <= max && !value.chars().any(char::is_control)
}

fn valid_local_paper_currency(value: &str) -> bool {
    value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase())
}

fn valid_local_paper_timestamp(value: &str) -> bool {
    OffsetDateTime::parse(value, &Rfc3339).is_ok()
}

fn valid_local_paper_hash(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_local_paper_decimal(value: &str, nonnegative: bool) -> bool {
    crate::provider_io::decimal(&serde_json::Value::String(value.to_owned()))
        .is_ok_and(|normalized| normalized == value && (!nonnegative || !value.starts_with('-')))
}

pub(crate) fn timestamp() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(storage_error)
}

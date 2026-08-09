//! The DuckDB analytical projection (proposal §21–§22).
//!
//! ```text
//! <data-dir>/projections/sergeant.duckdb
//! ```
//!
//! **This file is a projection, not the source of truth** (§21, and §40's
//! "projections are disposable"). Deleting it loses nothing: the daemon folds
//! the journal and rebuilds it. That is not a fallback path used when
//! something goes wrong — it is the *only* population path. The daemon
//! rebuilds from scratch on every start, so "delete the file and restart"
//! and "restart" are the same operation, and no code anywhere can come to
//! depend on state that only exists inside this file.
//!
//! **One owner** (§22, §40). Only the daemon opens the database; clients ask
//! the API. Structurally: `duckdb` is imported by this module and nowhere
//! else in the crate, the [`Connection`] is private with no accessor, and the
//! only way out of here is a [`QueryResult`] or a [`GraphView`]. The M5
//! acceptance suite greps the tree to keep it that way.
//!
//! The tables are §22's list *as far as the journal actually carries the
//! data*: `events`, `work`, `stages`, `executions`, `messages`, `tool_calls`,
//! `usage`, `repositories`, `graph_nodes`, `graph_edges`. §22 also names
//! `artifacts` and `git_changes`; no event family reports either today (no
//! adapter emits `filesystem.changed` or `git.changed`), so they are absent
//! rather than declared empty — a table that can only ever answer "zero rows"
//! is a false promise, not completeness.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use duckdb::Connection;
use duckdb::types::Value as Duck;
use serde_json::{Map, Value, json};

use crate::domain::event::{Event, unix_millis};
use crate::domain::execution::{
    KIND_EXECUTION_RECONCILED, KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
};
use crate::domain::work::{KIND_WORK_SUBMITTED, WorkState};
use crate::domain::workflow::{
    KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED,
    KIND_STAGE_FAILED, KIND_STAGE_NEEDS_INPUT, KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND,
};
use crate::runtime::fsutil::create_dir_all_durable;
use crate::runtime::graph::{
    GraphContext, KIND_CONVERSATION_ASSISTANT_COMPLETED, KIND_CONVERSATION_USER,
    KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};
use crate::runtime::journal::JournalError;
use crate::runtime::surface::{KIND_SURFACE_MATERIALIZED, KIND_SURFACE_TORN_DOWN};

/// Directory under the data dir holding disposable projections (§21).
pub const PROJECTIONS_DIR: &str = "projections";
/// DuckDB file name inside [`PROJECTIONS_DIR`] (§21's path, deviation D1).
pub const DUCKDB_FILE: &str = "sergeant.duckdb";

/// The projections directory for a data dir.
pub fn projections_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(PROJECTIONS_DIR)
}

/// The DuckDB projection file for a data dir.
pub fn duckdb_path(data_dir: &Path) -> PathBuf {
    projections_dir(data_dir).join(DUCKDB_FILE)
}

/// Failures of the analytical projection.
///
/// Every one of these is survivable by definition: the journal is untouched
/// and a rebuild is always available. Callers must never translate one into a
/// failure of the operation that produced the event.
#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    /// DuckDB refused a statement or could not open the database.
    #[error("duckdb error: {0}")]
    Duck(#[from] duckdb::Error),
    /// The journal failed while replaying for a rebuild.
    #[error(transparent)]
    Journal(#[from] JournalError),
    /// Filesystem failure around the projections directory.
    #[error("projection io error: {0}")]
    Io(#[from] std::io::Error),
    /// A canned query was asked for by a name that does not exist.
    #[error("no such analytics query {name:?}")]
    UnknownQuery {
        /// The name that was asked for.
        name: String,
    },
}

/// Prepared statements the fold keeps planned. Comfortably above the number
/// of distinct statements it issues, so no statement is ever evicted and
/// re-planned mid-rebuild.
const STATEMENT_CACHE: usize = 64;

/// The §22 schema. Written on every rebuild; there is no migration path and
/// there does not need to be one — the file is derived.
const SCHEMA: &str = r#"
CREATE TABLE events (
    seq            BIGINT PRIMARY KEY,
    event_id       VARCHAR NOT NULL,
    timestamp      VARCHAR NOT NULL,
    ts_ms          BIGINT,
    source_type    VARCHAR NOT NULL,
    source_name    VARCHAR NOT NULL,
    workspace_id   VARCHAR,
    work_id        VARCHAR,
    execution_id   VARCHAR,
    correlation_id VARCHAR,
    causation_id   VARCHAR,
    kind           VARCHAR NOT NULL,
    payload        VARCHAR NOT NULL
);
CREATE TABLE work (
    work_id       VARCHAR PRIMARY KEY,
    intent        VARCHAR,
    workspace     VARCHAR,
    workflow      VARCHAR,
    backend       VARCHAR,
    route_source  VARCHAR,
    profile       VARCHAR,
    origin_client VARCHAR,
    created_by    VARCHAR,
    created_at    VARCHAR,
    state         VARCHAR,
    submitted_seq BIGINT,
    submitted_ms  BIGINT
);
CREATE TABLE stages (
    work_id     VARCHAR,
    stage_id    VARCHAR,
    attempt     BIGINT,
    idx         BIGINT,
    status      VARCHAR,
    detail      VARCHAR,
    entered_seq BIGINT,
    entered_ms  BIGINT,
    ended_seq   BIGINT,
    ended_ms    BIGINT,
    PRIMARY KEY (work_id, stage_id, attempt)
);
CREATE TABLE executions (
    execution_id          VARCHAR PRIMARY KEY,
    work_id               VARCHAR,
    backend               VARCHAR,
    native_id             VARCHAR,
    stage_id              VARCHAR,
    attempt               BIGINT,
    started_seq           BIGINT,
    started_ms            BIGINT,
    stopped_seq           BIGINT,
    stopped_ms            BIGINT,
    stop_requested        BOOLEAN,
    reconcile_disposition VARCHAR
);
CREATE TABLE messages (
    seq          BIGINT PRIMARY KEY,
    work_id      VARCHAR,
    execution_id VARCHAR,
    role         VARCHAR,
    text         VARCHAR,
    ts_ms        BIGINT
);
CREATE TABLE tool_calls (
    execution_id  VARCHAR,
    tool_use_id   VARCHAR,
    work_id       VARCHAR,
    name          VARCHAR,
    requested_seq BIGINT,
    requested_ms  BIGINT,
    completed_seq BIGINT,
    completed_ms  BIGINT,
    is_error      BOOLEAN,
    PRIMARY KEY (execution_id, tool_use_id)
);
CREATE TABLE "usage" (
    seq                   BIGINT PRIMARY KEY,
    work_id               VARCHAR,
    execution_id          VARCHAR,
    ts_ms                 BIGINT,
    model                 VARCHAR,
    input_tokens          BIGINT,
    output_tokens         BIGINT,
    cache_read_tokens     BIGINT,
    cache_creation_tokens BIGINT,
    total_cost_usd        DOUBLE,
    model_pin             VARCHAR,
    is_error              BOOLEAN
);
CREATE TABLE repositories (
    work_id          VARCHAR,
    repository       VARCHAR,
    source_path      VARCHAR,
    base_branch      VARCHAR,
    base_sha         VARCHAR,
    worktree_path    VARCHAR,
    work_branch      VARCHAR,
    head_sha         VARCHAR,
    materialized_seq BIGINT,
    torn_down_seq    BIGINT,
    disposition      VARCHAR,
    PRIMARY KEY (work_id, repository)
);
CREATE TABLE graph_nodes (
    node_id    VARCHAR PRIMARY KEY,
    kind       VARCHAR NOT NULL,
    label      VARCHAR,
    work_id    VARCHAR,
    source_seq BIGINT NOT NULL
);
CREATE TABLE graph_edges (
    edge_id    VARCHAR PRIMARY KEY,
    relation   VARCHAR NOT NULL,
    from_node  VARCHAR NOT NULL,
    to_node    VARCHAR NOT NULL,
    work_id    VARCHAR,
    source_seq BIGINT NOT NULL
);
"#;

/// One of §22's example questions, as a query the daemon can actually answer.
#[derive(Debug, Clone, Copy)]
pub struct CannedQuery {
    /// Stable name (the API path segment and CLI argument).
    pub name: &'static str,
    /// The §22 question this answers, verbatim where §22 phrased one.
    pub question: &'static str,
    /// The SQL, exposed so an answer can be checked rather than trusted.
    pub sql: &'static str,
}

/// The canned queries this build answers.
///
/// Deliberately a fixed list rather than arbitrary client SQL: §22's "clients
/// do not access DuckDB directly" is about the *one-owner* property, and an
/// endpoint that executes a client's SQL against the daemon's database hands
/// the ownership back. M6 owns presentation; this is the data behind it.
pub const CANNED_QUERIES: &[CannedQuery] = &[
    CannedQuery {
        name: "blocked_time_per_work",
        question: "How long does work remain blocked?",
        // Every `work.*` event opens an interval that the next one closes;
        // the still-open tail of a work that is blocked *right now* is left
        // out rather than measured against a wall clock the journal does not
        // contain (a projection must not invent a fact about the present).
        sql: "\
            WITH spans AS (\
                SELECT work_id, kind, ts_ms, \
                       lead(ts_ms) OVER (PARTITION BY work_id ORDER BY seq) AS next_ms \
                FROM events \
                WHERE work_id IS NOT NULL AND kind LIKE 'work.%' \
            ) \
            SELECT w.work_id, w.state, \
                   COALESCE(SUM(CASE WHEN spans.kind = 'work.blocked' AND spans.next_ms IS NOT NULL \
                                     THEN spans.next_ms - spans.ts_ms END), 0) AS blocked_ms, \
                   COUNT(CASE WHEN spans.kind = 'work.blocked' THEN 1 END) AS blocked_episodes \
            FROM work w LEFT JOIN spans ON spans.work_id = w.work_id \
            GROUP BY w.work_id, w.state ORDER BY blocked_ms DESC, w.work_id",
    },
    CannedQuery {
        name: "backend_retries",
        question: "Which backend produces the most retries?",
        // A retry is a stage entered for a second or later attempt — the
        // §12 verb's observable trace, attributed to the backend the run
        // routed to.
        sql: "\
            SELECT COALESCE(w.backend, '(unrouted)') AS backend, \
                   COUNT(*) FILTER (WHERE s.attempt > 1) AS retries, \
                   COUNT(*) AS stage_attempts, \
                   COUNT(DISTINCT s.work_id) AS works \
            FROM stages s JOIN work w ON w.work_id = s.work_id \
            GROUP BY 1 ORDER BY retries DESC, backend",
    },
    CannedQuery {
        name: "execution_touched",
        question: "What did this execution touch?",
        sql: "\
            SELECT e.execution_id, e.work_id, e.backend, e.stage_id, \
                   COALESCE(r.repositories, 0) AS repositories, \
                   COALESCE(t.tool_calls, 0) AS tool_calls, \
                   COALESCE(m.messages, 0) AS messages \
            FROM executions e \
            LEFT JOIN (SELECT work_id, COUNT(*) AS repositories FROM repositories GROUP BY 1) r \
                   ON r.work_id = e.work_id \
            LEFT JOIN (SELECT execution_id, COUNT(*) AS tool_calls FROM tool_calls GROUP BY 1) t \
                   ON t.execution_id = e.execution_id \
            LEFT JOIN (SELECT execution_id, COUNT(*) AS messages FROM messages GROUP BY 1) m \
                   ON m.execution_id = e.execution_id \
            ORDER BY e.started_seq",
    },
    CannedQuery {
        name: "tool_calls_before_failure",
        question: "How frequently does a tool call precede a failure?",
        // Failure-class events are the journal's own: a stage or work that
        // failed, and a block (§25's fail-closed landing state). Tool calls
        // are counted up to the *first* such event per work, which is the
        // only unambiguous reading of "precede".
        sql: "\
            WITH failure AS (\
                SELECT work_id, MIN(seq) AS failed_seq \
                FROM events \
                WHERE work_id IS NOT NULL \
                  AND kind IN ('work.failed', 'work.blocked', 'stage.failed', 'stage.blocked') \
                GROUP BY work_id \
            ) \
            SELECT f.work_id, f.failed_seq, \
                   (SELECT kind FROM events WHERE seq = f.failed_seq) AS failure_kind, \
                   COUNT(t.tool_use_id) AS tool_calls_before \
            FROM failure f \
            LEFT JOIN tool_calls t \
                   ON t.work_id = f.work_id AND t.requested_seq < f.failed_seq \
            GROUP BY f.work_id, f.failed_seq ORDER BY tool_calls_before DESC, f.work_id",
    },
    CannedQuery {
        name: "token_totals_per_work",
        question: "How many tokens (and how much cost) has each work consumed?",
        sql: "\
            SELECT work_id, \
                   COUNT(*) AS turns, \
                   COALESCE(SUM(input_tokens), 0) AS input_tokens, \
                   COALESCE(SUM(output_tokens), 0) AS output_tokens, \
                   COALESCE(SUM(cache_read_tokens), 0) AS cache_read_tokens, \
                   COALESCE(SUM(total_cost_usd), 0) AS total_cost_usd \
            FROM \"usage\" GROUP BY work_id ORDER BY work_id",
    },
];

/// The result of one canned query: columns and rows as plain JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    /// Query name.
    pub name: String,
    /// The §22 question.
    pub question: String,
    /// The SQL that produced these rows.
    pub sql: String,
    /// Column names, in order.
    pub columns: Vec<String>,
    /// Rows, each aligned with `columns`.
    pub rows: Vec<Vec<Value>>,
}

impl QueryResult {
    /// JSON rendering for the API and the CLI.
    pub fn to_json(&self) -> Value {
        json!({
            "query": self.name,
            "question": self.question,
            "sql": self.sql,
            "columns": self.columns,
            "rows": self.rows,
        })
    }
}

/// A work's graph neighborhood (§8's `GET /v1/graph/work/{id}`).
#[derive(Debug, Clone, PartialEq)]
pub struct GraphView {
    /// Nodes in the neighborhood.
    pub nodes: Vec<Value>,
    /// Edges in the neighborhood, each with its `source_seq`.
    pub edges: Vec<Value>,
}
/// The DuckDB analytical projection over one data dir.
///
/// Owns its connection privately; nothing hands a [`Connection`] out.
///
/// **The database is a materialization of a pure fold, not a place state is
/// edited.** Journal events are folded into plain Rust rows ([`Rows`], no
/// I/O, deterministic iteration order) and the tables are written from those
/// rows in bulk. That shape was chosen on a measurement, not a preference:
/// DuckDB is a columnar store where a single-row `INSERT`/`UPDATE` costs
/// ~1–2 ms, so folding straight into SQL rebuilt a 1 600-event journal in
/// 65 s (≈24 events/s) on this container. The same fold through the bulk
/// appender is ~4 µs a row. Two further properties fall out of it, and both
/// are the milestone's point: the file is provably a function of the
/// journal, and the incremental path and the rebuild path are the *same*
/// fold rather than two implementations that have to be kept in agreement.
pub struct Analytics {
    conn: Connection,
    path: PathBuf,
    /// The mutable §22 tables, folded in memory.
    rows: Rows,
    /// §23 derivation state (see [`GraphContext`]).
    graph: GraphContext,
    /// Seq of the last event folded in.
    last_seq: u64,
    /// Seq the mutable tables were last written at. The append-only tables
    /// (`events`, `messages`, `usage`) are written as they are folded, so
    /// only the mutable ones can fall behind — and only until the next read.
    materialized_seq: u64,
}

impl std::fmt::Debug for Analytics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Analytics")
            .field("path", &self.path)
            .field("last_seq", &self.last_seq)
            .finish_non_exhaustive()
    }
}

/// How many append-only rows are buffered before being flushed to DuckDB.
///
/// Bounds the memory a rebuild holds: event payloads are the bulk of a
/// journal, and a rebuild must not need the whole journal resident to write
/// it out.
const APPEND_CHUNK: usize = 4096;

/// The mutable §22 tables, keyed so a later event can revise a row.
///
/// `events`, `messages` and `usage` are absent: nothing ever revises them, so
/// they stream straight to the database and are never held here.
#[derive(Debug, Default)]
struct Rows {
    work: BTreeMap<String, WorkRow>,
    stages: BTreeMap<(String, String, i64), StageRow>,
    executions: BTreeMap<String, ExecutionRow>,
    tool_calls: BTreeMap<(String, String), ToolCallRow>,
    repositories: BTreeMap<(String, String), RepositoryRow>,
    nodes: BTreeMap<String, NodeRow>,
    edges: BTreeMap<String, EdgeRow>,
}

/// Rows buffered for the append-only tables during one fold.
#[derive(Debug, Default)]
struct Appended {
    events: Vec<Vec<Duck>>,
    messages: Vec<Vec<Duck>>,
    usage: Vec<Vec<Duck>>,
}

impl Appended {
    fn len(&self) -> usize {
        self.events.len() + self.messages.len() + self.usage.len()
    }
}

#[derive(Debug, Default)]
struct WorkRow {
    work_id: String,
    intent: Option<String>,
    workspace: Option<String>,
    workflow: Option<String>,
    backend: Option<String>,
    route_source: Option<String>,
    profile: Option<String>,
    origin_client: Option<String>,
    created_by: Option<String>,
    created_at: Option<String>,
    state: Option<String>,
    submitted_seq: i64,
    submitted_ms: Option<i64>,
}

#[derive(Debug, Default)]
struct StageRow {
    work_id: String,
    stage_id: String,
    attempt: i64,
    idx: i64,
    status: String,
    detail: Option<String>,
    entered_seq: i64,
    entered_ms: Option<i64>,
    ended_seq: Option<i64>,
    ended_ms: Option<i64>,
}

#[derive(Debug, Default)]
struct ExecutionRow {
    execution_id: String,
    work_id: Option<String>,
    backend: Option<String>,
    native_id: Option<String>,
    stage_id: Option<String>,
    attempt: i64,
    started_seq: i64,
    started_ms: Option<i64>,
    stopped_seq: Option<i64>,
    stopped_ms: Option<i64>,
    stop_requested: bool,
    reconcile_disposition: Option<String>,
}

#[derive(Debug, Default)]
struct ToolCallRow {
    execution_id: String,
    tool_use_id: String,
    work_id: Option<String>,
    name: Option<String>,
    requested_seq: i64,
    requested_ms: Option<i64>,
    completed_seq: Option<i64>,
    completed_ms: Option<i64>,
    is_error: Option<bool>,
}

#[derive(Debug, Default)]
struct RepositoryRow {
    work_id: String,
    repository: String,
    source_path: Option<String>,
    base_branch: Option<String>,
    base_sha: Option<String>,
    worktree_path: Option<String>,
    work_branch: Option<String>,
    head_sha: Option<String>,
    materialized_seq: i64,
    torn_down_seq: Option<i64>,
    disposition: Option<String>,
}

#[derive(Debug, Default)]
struct NodeRow {
    node_id: String,
    kind: String,
    label: String,
    work_id: Option<String>,
    source_seq: i64,
}

#[derive(Debug, Default)]
struct EdgeRow {
    edge_id: String,
    relation: String,
    from_node: String,
    to_node: String,
    work_id: Option<String>,
    source_seq: i64,
}

/// A DuckDB value from an optional string.
fn text(value: Option<&str>) -> Duck {
    value.map_or(Duck::Null, |s| Duck::Text(s.to_string()))
}

/// A JSON string field as an owned `Option<String>`; anything that is not a
/// string (absent, null, a number a future writer put there) is `None`.
fn string(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
}

/// A DuckDB value from an optional integer.
fn bigint(value: Option<i64>) -> Duck {
    value.map_or(Duck::Null, Duck::BigInt)
}

impl WorkRow {
    fn row(&self) -> Vec<Duck> {
        vec![
            Duck::Text(self.work_id.clone()),
            text(self.intent.as_deref()),
            text(self.workspace.as_deref()),
            text(self.workflow.as_deref()),
            text(self.backend.as_deref()),
            text(self.route_source.as_deref()),
            text(self.profile.as_deref()),
            text(self.origin_client.as_deref()),
            text(self.created_by.as_deref()),
            text(self.created_at.as_deref()),
            text(self.state.as_deref()),
            Duck::BigInt(self.submitted_seq),
            bigint(self.submitted_ms),
        ]
    }
}

impl StageRow {
    fn row(&self) -> Vec<Duck> {
        vec![
            Duck::Text(self.work_id.clone()),
            Duck::Text(self.stage_id.clone()),
            Duck::BigInt(self.attempt),
            Duck::BigInt(self.idx),
            Duck::Text(self.status.clone()),
            text(self.detail.as_deref()),
            Duck::BigInt(self.entered_seq),
            bigint(self.entered_ms),
            bigint(self.ended_seq),
            bigint(self.ended_ms),
        ]
    }
}

impl ExecutionRow {
    fn row(&self) -> Vec<Duck> {
        vec![
            Duck::Text(self.execution_id.clone()),
            text(self.work_id.as_deref()),
            text(self.backend.as_deref()),
            text(self.native_id.as_deref()),
            text(self.stage_id.as_deref()),
            Duck::BigInt(self.attempt),
            Duck::BigInt(self.started_seq),
            bigint(self.started_ms),
            bigint(self.stopped_seq),
            bigint(self.stopped_ms),
            Duck::Boolean(self.stop_requested),
            text(self.reconcile_disposition.as_deref()),
        ]
    }
}

impl ToolCallRow {
    fn row(&self) -> Vec<Duck> {
        vec![
            Duck::Text(self.execution_id.clone()),
            Duck::Text(self.tool_use_id.clone()),
            text(self.work_id.as_deref()),
            text(self.name.as_deref()),
            Duck::BigInt(self.requested_seq),
            bigint(self.requested_ms),
            bigint(self.completed_seq),
            bigint(self.completed_ms),
            self.is_error.map_or(Duck::Null, Duck::Boolean),
        ]
    }
}

impl RepositoryRow {
    fn row(&self) -> Vec<Duck> {
        vec![
            Duck::Text(self.work_id.clone()),
            Duck::Text(self.repository.clone()),
            text(self.source_path.as_deref()),
            text(self.base_branch.as_deref()),
            text(self.base_sha.as_deref()),
            text(self.worktree_path.as_deref()),
            text(self.work_branch.as_deref()),
            text(self.head_sha.as_deref()),
            Duck::BigInt(self.materialized_seq),
            bigint(self.torn_down_seq),
            text(self.disposition.as_deref()),
        ]
    }
}

impl NodeRow {
    fn row(&self) -> Vec<Duck> {
        vec![
            Duck::Text(self.node_id.clone()),
            Duck::Text(self.kind.clone()),
            Duck::Text(self.label.clone()),
            text(self.work_id.as_deref()),
            Duck::BigInt(self.source_seq),
        ]
    }
}

impl EdgeRow {
    fn row(&self) -> Vec<Duck> {
        vec![
            Duck::Text(self.edge_id.clone()),
            Duck::Text(self.relation.clone()),
            Duck::Text(self.from_node.clone()),
            Duck::Text(self.to_node.clone()),
            text(self.work_id.as_deref()),
            Duck::BigInt(self.source_seq),
        ]
    }
}

/// Tables written from [`Rows`] on every materialization, in dependency-free
/// order. The append-only tables are deliberately not here.
const MUTABLE_TABLES: &[&str] = &[
    "work",
    "stages",
    "executions",
    "tool_calls",
    "repositories",
    "graph_nodes",
    "graph_edges",
];

impl Analytics {
    /// Rebuild the projection for `data_dir` from scratch, folding `events`.
    ///
    /// Any existing file is deleted first — including DuckDB's write-ahead
    /// log, which would otherwise resurrect the old contents. This is the
    /// daemon's startup path and the whole population story: there is no
    /// "open the existing database and continue", so nothing can quietly
    /// accumulate state that the journal cannot reproduce.
    pub fn rebuild<I>(data_dir: &Path, events: I) -> Result<Self, AnalyticsError>
    where
        I: IntoIterator<Item = Result<Event, JournalError>>,
    {
        let path = duckdb_path(data_dir);
        create_dir_all_durable(&projections_dir(data_dir))?;
        remove_if_present(&path)?;
        remove_if_present(&path.with_extension("duckdb.wal"))?;
        let conn = Connection::open(&path)?;
        let mut analytics = Self::over(conn, path)?;
        analytics.catch_up(events)?;
        Ok(analytics)
    }

    /// An in-memory projection over `events`, for callers that want the
    /// tables without a file (tests, and any read-only rendering).
    pub fn in_memory<I>(events: I) -> Result<Self, AnalyticsError>
    where
        I: IntoIterator<Item = Result<Event, JournalError>>,
    {
        let conn = Connection::open_in_memory()?;
        let mut analytics = Self::over(conn, PathBuf::from(":memory:"))?;
        analytics.catch_up(events)?;
        Ok(analytics)
    }

    fn over(conn: Connection, path: PathBuf) -> Result<Self, AnalyticsError> {
        conn.set_prepared_statement_cache_capacity(STATEMENT_CACHE);
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn,
            path,
            rows: Rows::default(),
            graph: GraphContext::default(),
            last_seq: 0,
            materialized_seq: 0,
        })
    }

    /// Seq of the last event folded in.
    pub fn last_seq(&self) -> u64 {
        self.last_seq
    }

    /// Fold every event in `events` with a seq past [`Analytics::last_seq`].
    ///
    /// The rebuild path and the incremental path are this one method, so
    /// "rebuilt from scratch" and "kept current" cannot drift apart. Events
    /// at or below the current seq are skipped, so handing this the whole
    /// journal is always safe.
    pub fn catch_up<I>(&mut self, events: I) -> Result<u64, AnalyticsError>
    where
        I: IntoIterator<Item = Result<Event, JournalError>>,
    {
        let mut appended = Appended::default();
        let mut applied = 0u64;
        for event in events {
            let event = event?;
            if event.seq <= self.last_seq {
                continue;
            }
            self.apply(&event, &mut appended);
            self.last_seq = event.seq;
            applied += 1;
            if appended.len() >= APPEND_CHUNK {
                self.flush(&mut appended)?;
            }
        }
        self.flush(&mut appended)?;
        Ok(applied)
    }

    /// Write the buffered append-only rows.
    fn flush(&self, appended: &mut Appended) -> Result<(), AnalyticsError> {
        append_all(&self.conn, "events", std::mem::take(&mut appended.events))?;
        append_all(
            &self.conn,
            "messages",
            std::mem::take(&mut appended.messages),
        )?;
        append_all(&self.conn, "usage", std::mem::take(&mut appended.usage))?;
        Ok(())
    }

    /// Write the mutable tables from the folded rows, if they have moved.
    ///
    /// Truncate-and-load rather than row-wise update: it is the same
    /// measurement as above, and it makes each materialization a total
    /// function of the fold state — a stale row cannot survive one.
    fn materialize(&mut self) -> Result<(), AnalyticsError> {
        if self.materialized_seq == self.last_seq {
            return Ok(());
        }
        for table in MUTABLE_TABLES {
            self.conn
                .execute_batch(&format!("DELETE FROM \"{table}\""))?;
        }
        append_all(
            &self.conn,
            "work",
            self.rows.work.values().map(WorkRow::row).collect(),
        )?;
        append_all(
            &self.conn,
            "stages",
            self.rows.stages.values().map(StageRow::row).collect(),
        )?;
        append_all(
            &self.conn,
            "executions",
            self.rows
                .executions
                .values()
                .map(ExecutionRow::row)
                .collect(),
        )?;
        append_all(
            &self.conn,
            "tool_calls",
            self.rows
                .tool_calls
                .values()
                .map(ToolCallRow::row)
                .collect(),
        )?;
        append_all(
            &self.conn,
            "repositories",
            self.rows
                .repositories
                .values()
                .map(RepositoryRow::row)
                .collect(),
        )?;
        append_all(
            &self.conn,
            "graph_nodes",
            self.rows.nodes.values().map(NodeRow::row).collect(),
        )?;
        append_all(
            &self.conn,
            "graph_edges",
            self.rows.edges.values().map(EdgeRow::row).collect(),
        )?;
        self.materialized_seq = self.last_seq;
        Ok(())
    }

    /// Fold one event into every table it touches. Pure with respect to the
    /// database: it only ever touches in-memory state and the append buffer.
    ///
    /// Kinds and payload shapes this fold does not understand are ignored,
    /// never an error — a newer writer's events must not brick an older
    /// reader's rebuild (§20, the same stance the registry reducer takes).
    fn apply(&mut self, event: &Event, appended: &mut Appended) {
        let seq = event.seq as i64;
        let ts = unix_millis(&event.timestamp);
        appended.events.push(vec![
            Duck::BigInt(seq),
            Duck::Text(event.id.clone()),
            Duck::Text(event.timestamp.clone()),
            bigint(ts),
            Duck::Text(event.source.source_type.clone()),
            Duck::Text(event.source.name.clone()),
            text(event.workspace_id.as_deref()),
            text(event.work_id.as_deref()),
            text(event.execution_id.as_deref()),
            text(event.correlation_id.as_deref()),
            text(event.causation_id.as_deref()),
            Duck::Text(event.kind.clone()),
            Duck::Text(event.payload.to_string()),
        ]);

        let delta = self.graph.derive(event);
        for node in delta.nodes {
            // First sighting wins: a node's `source_seq` is where it entered
            // the history, not the last time something mentioned it.
            self.rows.nodes.entry(node.id.clone()).or_insert(NodeRow {
                node_id: node.id,
                kind: node.kind.to_string(),
                label: node.label,
                work_id: node.work_id,
                source_seq: node.source_seq as i64,
            });
        }
        for edge in delta.edges {
            self.rows.edges.entry(edge.id.clone()).or_insert(EdgeRow {
                edge_id: edge.id,
                relation: edge.relation.to_string(),
                from_node: edge.from,
                to_node: edge.to,
                work_id: edge.work_id,
                source_seq: edge.source_seq as i64,
            });
        }

        // A §10 transition is the only thing that rewrites `work.state`.
        if let (Some(work_id), Some(state)) = (
            event.work_id.as_deref(),
            WorkState::for_event_kind(&event.kind),
        ) && let Some(work) = self.rows.work.get_mut(work_id)
        {
            work.state = Some(state.as_str().to_string());
        }

        match event.kind.as_str() {
            KIND_WORK_SUBMITTED => {
                let work = &event.payload["work"];
                if let Some(work_id) = work["id"].as_str() {
                    self.rows.work.insert(
                        work_id.to_string(),
                        WorkRow {
                            work_id: work_id.to_string(),
                            intent: string(&work["intent"]),
                            workspace: string(&work["workspace"]),
                            workflow: string(&work["workflow"]),
                            backend: string(&work["backend"]),
                            route_source: None,
                            profile: string(&work["profile"]),
                            origin_client: string(&work["origin_client"]),
                            created_by: string(&work["created_by"]),
                            created_at: string(&work["created_at"]),
                            state: string(&work["state"]),
                            submitted_seq: seq,
                            submitted_ms: ts,
                        },
                    );
                }
            }
            KIND_WORKFLOW_BOUND => {
                // The *routed* backend and the pinned workflow supersede what
                // the submission asked for: §13's precedence is decided here,
                // and "which backend produces the most retries" means the one
                // that ran, not the one that was requested.
                if let Some(work) = event
                    .work_id
                    .as_deref()
                    .and_then(|id| self.rows.work.get_mut(id))
                {
                    work.workflow = string(&event.payload["workflow"]["name"]);
                    work.backend = string(&event.payload["backend"]);
                    work.route_source = string(&event.payload["route_source"]);
                    work.profile = string(&event.payload["profile"]["name"]);
                    if let Some(workspace) = string(&event.payload["workspace"]) {
                        work.workspace = Some(workspace);
                    }
                }
            }
            KIND_SURFACE_MATERIALIZED => {
                let Some(work_id) = event.work_id.as_deref() else {
                    return;
                };
                let bindings = event.payload["surface"]["bindings"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                for binding in &bindings {
                    let Some(repository) = binding["repository"].as_str() else {
                        continue;
                    };
                    self.rows.repositories.insert(
                        (work_id.to_string(), repository.to_string()),
                        RepositoryRow {
                            work_id: work_id.to_string(),
                            repository: repository.to_string(),
                            source_path: string(&binding["source_path"]),
                            base_branch: string(&binding["base_branch"]),
                            base_sha: string(&binding["base_sha"]),
                            worktree_path: string(&binding["worktree_path"]),
                            work_branch: string(&binding["work_branch"]),
                            head_sha: string(&binding["head_sha"]),
                            materialized_seq: seq,
                            torn_down_seq: None,
                            disposition: None,
                        },
                    );
                }
            }
            KIND_SURFACE_TORN_DOWN => {
                let Some(work_id) = event.work_id.as_deref() else {
                    return;
                };
                let bindings = event.payload["report"]["bindings"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                for binding in &bindings {
                    let Some(repository) = binding["repository"].as_str() else {
                        continue;
                    };
                    if let Some(row) = self
                        .rows
                        .repositories
                        .get_mut(&(work_id.to_string(), repository.to_string()))
                    {
                        row.torn_down_seq = Some(seq);
                        row.disposition = string(&binding["disposition"]);
                    }
                }
            }
            KIND_STAGE_ENTERED => {
                let (Some(work_id), Some(stage_id)) =
                    (event.work_id.as_deref(), event.payload["stage_id"].as_str())
                else {
                    return;
                };
                let attempt = event.payload["attempt"].as_u64().unwrap_or(1) as i64;
                self.rows.stages.insert(
                    (work_id.to_string(), stage_id.to_string(), attempt),
                    StageRow {
                        work_id: work_id.to_string(),
                        stage_id: stage_id.to_string(),
                        attempt,
                        idx: event.payload["index"].as_u64().unwrap_or(0) as i64,
                        status: "active".to_string(),
                        detail: None,
                        entered_seq: seq,
                        entered_ms: ts,
                        ended_seq: None,
                        ended_ms: None,
                    },
                );
            }
            KIND_STAGE_COMPLETED
            | KIND_STAGE_WAITING
            | KIND_STAGE_NEEDS_INPUT
            | KIND_STAGE_BLOCKED
            | KIND_STAGE_FAILED
            | KIND_STAGE_CANCELED => {
                let (Some(work_id), Some(stage_id)) =
                    (event.work_id.as_deref(), event.payload["stage_id"].as_str())
                else {
                    return;
                };
                // The latest attempt of that stage is what this outcome is
                // about, mirroring the registry reducer's `last_stage_mut`.
                let latest = self
                    .rows
                    .stages
                    .keys()
                    .filter(|(w, s, _)| w == work_id && s == stage_id)
                    .map(|(_, _, attempt)| *attempt)
                    .max();
                let Some(attempt) = latest else { return };
                if let Some(row) =
                    self.rows
                        .stages
                        .get_mut(&(work_id.to_string(), stage_id.to_string(), attempt))
                {
                    row.status = event
                        .kind
                        .strip_prefix("stage.")
                        .unwrap_or(&event.kind)
                        .to_string();
                    row.detail = string(&event.payload["detail"]);
                    row.ended_seq = Some(seq);
                    row.ended_ms = ts;
                }
            }
            KIND_EXECUTION_STARTED => {
                let execution = &event.payload["execution"];
                let Some(execution_id) = execution["execution_id"].as_str() else {
                    return;
                };
                self.rows.executions.insert(
                    execution_id.to_string(),
                    ExecutionRow {
                        execution_id: execution_id.to_string(),
                        work_id: event.work_id.clone(),
                        backend: string(&execution["backend"]),
                        native_id: string(&execution["native_id"]),
                        stage_id: string(&execution["stage_id"]),
                        attempt: execution["attempt"].as_u64().unwrap_or(1) as i64,
                        started_seq: seq,
                        started_ms: ts,
                        stopped_seq: None,
                        stopped_ms: None,
                        stop_requested: execution["stop_requested"].as_bool().unwrap_or(false),
                        reconcile_disposition: None,
                    },
                );
            }
            KIND_EXECUTION_STOPPED => {
                let Some(execution_id) = event.payload["execution_id"].as_str() else {
                    return;
                };
                // `stop_requested` latches only when the backend was asked
                // *and did not refuse* — the same condition the registry
                // reducer applies, so the analytic and the daemon's own state
                // cannot disagree about whether a stop was ever delivered.
                let acknowledged = event.payload["outcome"]["requested"] == Value::Bool(true)
                    && event.payload["outcome"]["error"].is_null();
                if let Some(row) = self.rows.executions.get_mut(execution_id) {
                    row.stopped_seq = Some(seq);
                    row.stopped_ms = ts;
                    row.stop_requested = row.stop_requested || acknowledged;
                }
            }
            KIND_EXECUTION_RECONCILED => {
                if let Some(row) = event.payload["execution_id"]
                    .as_str()
                    .and_then(|id| self.rows.executions.get_mut(id))
                {
                    row.reconcile_disposition = string(&event.payload["disposition"]);
                }
            }
            KIND_CONVERSATION_USER | KIND_CONVERSATION_ASSISTANT_COMPLETED => {
                let role = if event.kind == KIND_CONVERSATION_USER {
                    "user"
                } else {
                    "assistant"
                };
                appended.messages.push(vec![
                    Duck::BigInt(seq),
                    text(event.work_id.as_deref()),
                    text(event.execution_id.as_deref()),
                    Duck::Text(role.to_string()),
                    text(event.payload["text"].as_str()),
                    bigint(ts),
                ]);
            }
            KIND_TOOL_REQUESTED => {
                let Some(execution_id) = event.execution_id.as_deref() else {
                    return;
                };
                let tool_use_id = event.payload["id"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("seq{seq}"));
                self.rows.tool_calls.insert(
                    (execution_id.to_string(), tool_use_id.clone()),
                    ToolCallRow {
                        execution_id: execution_id.to_string(),
                        tool_use_id,
                        work_id: event.work_id.clone(),
                        name: string(&event.payload["name"]),
                        requested_seq: seq,
                        requested_ms: ts,
                        completed_seq: None,
                        completed_ms: None,
                        is_error: None,
                    },
                );
            }
            KIND_TOOL_COMPLETED => {
                let (Some(execution_id), Some(tool_use_id)) = (
                    event.execution_id.as_deref(),
                    event.payload["tool_use_id"].as_str(),
                ) else {
                    return;
                };
                if let Some(row) = self
                    .rows
                    .tool_calls
                    .get_mut(&(execution_id.to_string(), tool_use_id.to_string()))
                {
                    row.completed_seq = Some(seq);
                    row.completed_ms = ts;
                    row.is_error = event.payload["is_error"].as_bool();
                }
            }
            KIND_USAGE_UPDATED => {
                let usage = &event.payload["usage"];
                // §28's token counters read from here, so the cache fields
                // stay apart from `input_tokens`: a cache read is not an
                // input token, and summing them would inflate every token
                // metric this milestone exports.
                let model = event.payload["model_usage"]
                    .as_object()
                    .and_then(|m| m.keys().next().cloned());
                appended.usage.push(vec![
                    Duck::BigInt(seq),
                    text(event.work_id.as_deref()),
                    text(event.execution_id.as_deref()),
                    bigint(ts),
                    text(model.as_deref()),
                    bigint(usage["input_tokens"].as_i64()),
                    bigint(usage["output_tokens"].as_i64()),
                    bigint(usage["cache_read_input_tokens"].as_i64()),
                    bigint(usage["cache_creation_input_tokens"].as_i64()),
                    event.payload["total_cost_usd"]
                        .as_f64()
                        .map_or(Duck::Null, Duck::Double),
                    text(event.payload["model_pin"]["verdict"].as_str()),
                    event.payload["is_error"]
                        .as_bool()
                        .map_or(Duck::Null, Duck::Boolean),
                ]);
            }
            _ => {}
        }
    }

    /// Run one canned query by name.
    ///
    /// Takes `&mut self` because a read materializes the fold first: the
    /// tables are always exactly as current as the events folded in, and no
    /// caller can observe a half-written projection.
    pub fn query(&mut self, name: &str) -> Result<QueryResult, AnalyticsError> {
        self.materialize()?;
        let canned = CANNED_QUERIES
            .iter()
            .find(|q| q.name == name)
            .ok_or_else(|| AnalyticsError::UnknownQuery {
                name: name.to_string(),
            })?;
        let (columns, rows) = self.select(canned.sql, duckdb::params![])?;
        Ok(QueryResult {
            name: canned.name.to_string(),
            question: canned.question.to_string(),
            sql: canned.sql.to_string(),
            columns,
            rows,
        })
    }

    /// Row counts per table — the cheapest honest answer to "is this
    /// projection actually populated", and what the disposability tests
    /// compare across a rebuild.
    pub fn table_counts(&mut self) -> Result<Vec<(String, i64)>, AnalyticsError> {
        self.materialize()?;
        let mut counts = Vec::new();
        for table in TABLES {
            let mut statement = self
                .conn
                .prepare(&format!("SELECT COUNT(*) FROM \"{table}\""))?;
            let count: i64 = statement.query_row(duckdb::params![], |row| row.get(0))?;
            counts.push(((*table).to_string(), count));
        }
        Ok(counts)
    }

    /// Every row of one §22 table.
    ///
    /// The name is checked against [`TABLES`] rather than interpolated — the
    /// projection answers questions it knows, never SQL it was handed, which
    /// is the same rule that keeps [`CANNED_QUERIES`] a fixed list.
    pub fn table_rows(&mut self, table: &str) -> Result<QueryResult, AnalyticsError> {
        let table = TABLES
            .iter()
            .find(|known| **known == table)
            .ok_or_else(|| AnalyticsError::UnknownQuery {
                name: table.to_string(),
            })?;
        self.materialize()?;
        let sql = format!("SELECT * FROM \"{table}\"");
        let (columns, rows) = self.select(&sql, duckdb::params![])?;
        Ok(QueryResult {
            name: (*table).to_string(),
            question: format!("every row of the {table} table"),
            sql,
            columns,
            rows,
        })
    }

    /// The graph neighborhood of one work (§8's `GET /v1/graph/work/{id}`).
    ///
    /// "Neighborhood" is every edge the work owns plus every node those edges
    /// reach — so the shared nodes a work is connected to (its backend, its
    /// workflow, the model it ran on) come back with it, while the rest of
    /// the fleet does not.
    pub fn graph_neighborhood(&mut self, work_id: &str) -> Result<GraphView, AnalyticsError> {
        self.materialize()?;
        let (node_columns, node_rows) = self.select(
            "SELECT node_id, kind, label, work_id, source_seq FROM graph_nodes \
             WHERE work_id = ?1 \
                OR node_id IN (SELECT from_node FROM graph_edges WHERE work_id = ?1) \
                OR node_id IN (SELECT to_node FROM graph_edges WHERE work_id = ?1) \
             ORDER BY source_seq, node_id",
            duckdb::params![work_id],
        )?;
        let (edge_columns, edge_rows) = self.select(
            "SELECT edge_id, relation, from_node, to_node, source_seq FROM graph_edges \
             WHERE work_id = ?1 ORDER BY source_seq, edge_id",
            duckdb::params![work_id],
        )?;
        Ok(GraphView {
            nodes: objects(&node_columns, node_rows),
            edges: objects(&edge_columns, edge_rows),
        })
    }

    /// Run one SELECT, returning column names and JSON rows.
    fn select<P: duckdb::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<(Vec<String>, Vec<Vec<Value>>), AnalyticsError> {
        let mut statement = self.conn.prepare(sql)?;
        let mut rows = statement.query(params)?;
        let mut out: Vec<Vec<Value>> = Vec::new();
        let mut columns: Vec<String> = Vec::new();
        while let Some(row) = rows.next()? {
            if columns.is_empty() {
                columns = row
                    .as_ref()
                    .column_names()
                    .iter()
                    .map(|c| c.to_string())
                    .collect();
            }
            let mut values = Vec::with_capacity(columns.len());
            for index in 0..columns.len() {
                values.push(to_json(row.get::<usize, duckdb::types::Value>(index)?));
            }
            out.push(values);
        }
        if columns.is_empty() {
            columns = statement
                .column_names()
                .iter()
                .map(|c| c.to_string())
                .collect();
        }
        Ok((columns, out))
    }
}

/// Tables this projection creates, in a stable order.
pub const TABLES: &[&str] = &[
    "events",
    "work",
    "stages",
    "executions",
    "messages",
    "tool_calls",
    "usage",
    "repositories",
    "graph_nodes",
    "graph_edges",
];

/// Bulk-load `rows` into `table` through DuckDB's appender.
///
/// The appender is the reason this projection is usable at all: measured on
/// this container, a single-row `INSERT` costs ~1 ms and an appended row
/// ~4 µs. An empty batch is a no-op rather than an open-and-close.
fn append_all(conn: &Connection, table: &str, rows: Vec<Vec<Duck>>) -> Result<(), AnalyticsError> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut appender = conn.appender(table)?;
    for values in rows {
        appender.append_row(duckdb::appender_params_from_iter(values))?;
    }
    appender.flush()?;
    Ok(())
}

/// Zip columns and rows into JSON objects.
fn objects(columns: &[String], rows: Vec<Vec<Value>>) -> Vec<Value> {
    rows.into_iter()
        .map(|row| {
            let mut object = Map::new();
            for (column, value) in columns.iter().zip(row) {
                object.insert(column.clone(), value);
            }
            Value::Object(object)
        })
        .collect()
}

/// Convert a DuckDB value into JSON, keeping numbers numeric.
fn to_json(value: duckdb::types::Value) -> Value {
    use duckdb::types::Value as D;
    match value {
        D::Null => Value::Null,
        D::Boolean(b) => Value::Bool(b),
        D::TinyInt(v) => json!(v),
        D::SmallInt(v) => json!(v),
        D::Int(v) => json!(v),
        D::BigInt(v) => json!(v),
        D::UTinyInt(v) => json!(v),
        D::USmallInt(v) => json!(v),
        D::UInt(v) => json!(v),
        D::UBigInt(v) => json!(v),
        D::Float(v) => json!(v),
        D::Double(v) => json!(v),
        D::Text(v) => Value::String(v),
        // DuckDB widens SUM over BIGINT to HUGEINT, so an ordinary
        // "milliseconds blocked" total arrives as an i128. Narrow it when it
        // fits — which every real total does — and render the impossible
        // remainder as text rather than silently truncating it.
        D::HugeInt(v) => i64::try_from(v).map_or_else(|_| json!(v.to_string()), |v| json!(v)),
        D::UHugeInt(v) => u64::try_from(v).map_or_else(|_| json!(v.to_string()), |v| json!(v)),
        // Everything else — decimals, blobs, containers — is rendered as its
        // debug text rather than silently coerced. No column in this schema
        // produces one; a future one that does will be visibly ugly instead
        // of invisibly wrong.
        other => Value::String(format!("{other:?}")),
    }
}

/// Remove a file if it exists; a missing file is success.
fn remove_if_present(path: &Path) -> Result<(), std::io::Error> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::event::{EventDraft, EventSource};

    fn events(list: Vec<Event>) -> Vec<Result<Event, JournalError>> {
        list.into_iter().map(Ok).collect()
    }

    fn event(seq: u64, work_id: &str, kind: &str, payload: Value) -> Event {
        EventDraft::new(EventSource::new("daemon", "test"), kind, payload)
            .with_work_id(work_id)
            .into_event(seq)
    }

    fn submitted(seq: u64, work_id: &str) -> Event {
        event(
            seq,
            work_id,
            KIND_WORK_SUBMITTED,
            json!({"work": {
                "id": work_id, "intent": "do it", "state": "pending",
                "created_by": "test", "created_at": "2026-01-01T00:00:00.000Z",
                "origin_client": "cli", "repositories": [],
            }}),
        )
    }

    #[test]
    fn every_canned_query_runs_against_an_empty_projection() {
        // A query that only parses against populated tables is a query that
        // breaks the first time someone asks it on a quiet daemon.
        let mut analytics = Analytics::in_memory(Vec::new()).expect("projection");
        for canned in CANNED_QUERIES {
            let result = analytics.query(canned.name).expect(canned.name);
            assert!(
                !result.columns.is_empty(),
                "{} produced no column names",
                canned.name
            );
            assert!(result.rows.is_empty(), "{} invented rows", canned.name);
        }
        assert!(matches!(
            analytics.query("no-such-query"),
            Err(AnalyticsError::UnknownQuery { .. })
        ));
    }

    #[test]
    fn the_schema_declares_exactly_the_tables_it_advertises() {
        let mut analytics = Analytics::in_memory(Vec::new()).expect("projection");
        let counts = analytics.table_counts().expect("counts");
        let names: Vec<&str> = counts.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, TABLES);
        assert!(counts.iter().all(|(_, count)| *count == 0));
    }

    #[test]
    fn folding_is_idempotent_over_a_replayed_prefix() {
        // `catch_up` skipping the covered prefix is what lets the daemon hand
        // it the whole journal at any time. If the skip were wrong, replaying
        // a prefix would double-count the events table.
        let journal = vec![submitted(1, "w1"), submitted(2, "w2")];
        let mut analytics = Analytics::in_memory(events(journal.clone())).expect("projection");
        assert_eq!(analytics.last_seq(), 2);
        assert_eq!(analytics.catch_up(events(journal)).expect("catch up"), 0);
        let counts = analytics.table_counts().expect("counts");
        assert_eq!(counts[0], ("events".to_string(), 2));
        assert_eq!(counts[1], ("work".to_string(), 2));
    }

    #[test]
    fn an_unparseable_timestamp_leaves_a_null_cell_and_never_fails_the_fold() {
        let mut odd = submitted(1, "w1");
        odd.timestamp = "whenever".to_string();
        let mut analytics = Analytics::in_memory(events(vec![odd])).expect("projection");
        analytics.materialize().expect("materialize");
        let (_, rows) = analytics
            .select("SELECT ts_ms FROM events", duckdb::params![])
            .expect("select");
        assert_eq!(rows, vec![vec![Value::Null]]);
    }
}

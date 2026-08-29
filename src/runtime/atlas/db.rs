//! The one owner of the estate's database (F2, A1 §5).
//!
//! ```text
//! <data-dir>/atlas/atlas.duckdb
//! ```
//!
//! One physical file, five logical schemas — `meta`, `ops`, `source`, `git`,
//! `context` — exactly as A1 §5 declares and for the reason A1-02 gives:
//! "schemas provide separation without more databases". A second file
//! (`projections/sergeant.duckdb`, carrying `ops`) shipped in S3 and was
//! removed in S5 W1c after the owner correction of 2026-08-29; the
//! capability §5 cites as the reason for one database — a cross-domain join
//! with no `ATTACH` — is what A2's `--work` filter needs, and it is pinned by
//! `tests/w1c_one_atlas_database.rs`.
//!
//! This file is the only place **in the crate** that names the `duckdb`
//! crate, and the [`Connection`]s it holds are private fields with no
//! accessor. `tests/x1_atlas_substrate.rs`'s
//! `atlas_database_has_exactly_one_owner` pins that structurally over the
//! whole of `src/`. It is one assertion because there is one database: two
//! tests naming two owners would be the union rule ("either of these files
//! may open a database") that both suites forbade while there really were
//! two files.
//!
//! # One file, two rebuild disciplines (F1)
//!
//! **`ops.*` is disposable, and only `ops.*`.** It is a pure fold of the
//! journal, so [`Analytics::begin_rebuild`] drops the whole `ops` schema and
//! re-folds it on every daemon start. `DROP SCHEMA ops CASCADE` has exactly
//! the scope that deleting the old separate file had, which is the point: the
//! discipline survived the merge, the file deletion did not.
//!
//! **Nothing else in this file is disposable.** [`AtlasDb::open`] opens the
//! existing file and keeps it, because `source.*`, `git.*` and
//! `meta.coverage` PERSIST across restarts. They are derived from source
//! bytes plus extractor identity, keyed by SourceGeneration; no journal
//! replay reproduces them. A generation is evicted only when the source bytes
//! it was derived from changed, and the eviction is reported as a coverage
//! row rather than a silent gap (ruling §4). The journal carries one compact
//! `source.scanned` summary per completed scan so the authoritative trail
//! stays journal-side while the unit-level detail stays here.
//!
//! Two consequences bind any later wave:
//!
//! * Nothing may delete this file to "fix" the operations tables. The
//!   supported operation for those is a restart, and no cleanup path may
//!   treat this file as disposable state.
//! * Atlas's DDL is `IF NOT EXISTS` because reopening an existing file is the
//!   normal path, not a recovery path. Only [`OPS_DDL`] drops anything.
//!
//! # What deleting the file costs
//!
//! Stated here because the old sentence — "deleting it loses nothing" — was
//! true of `sergeant.duckdb` and is **not** true of this file. Deleting
//! `atlas.duckdb` and restarting rebuilds every `ops` row exactly from the
//! journal, and discards every persisted source generation, which must be
//! re-scanned. That is acceptable under ruling §4 only because it is reported
//! rather than silent: a store with no confirmed generation says so, and a
//! re-scan writes fresh coverage. `tests/w1c_one_atlas_database.rs`'s
//! `deleting_atlas_duckdb_rebuilds_ops_and_loses_source_facts` measures both
//! halves of that sentence.
//!
//! # Why the file is not under `projections/`
//!
//! `<data-dir>/projections/` is documented as disposable — an acceptance test
//! deletes it wholesale and asserts nothing was lost — and since W1c it holds
//! only the FloorState startup cache
//! ([`crate::runtime::startup::PROJECTIONS_DIR`]). A database that must
//! survive restarts cannot live inside a directory advertised as disposable,
//! so it sits in its own directory beside the journal and the blobs.
//!
//! # Scope today
//!
//! The seven tables the three walks write, and nothing else. Every table
//! lands in the wave that lands its writer (the empty-table refusal
//! doctrine); a declared-but-never populated table is a false promise, not
//! completeness — which is why `source.symbols`, `source.occurrences` and
//! `source.edges` arrive here in X3b, with the extraction that fills them,
//! rather than having been declared empty in X1. `git.*` and `context.*` are
//! still empty namespaces because nothing writes them yet.
//!
//! **These tables are only ever added to, never altered.** The store persists
//! across restarts (F1) and the DDL is `IF NOT EXISTS`, so a column added to
//! an existing table would simply not appear in a database that already has
//! it — a silent schema drift with no migration behind it. X3b's rows
//! therefore live in new tables that carry their own copy of the coordinates
//! they need, and `source.files` keeps exactly the columns X2 gave it.
//!
//! # The confirmation protocol (F1's crash window)
//!
//! `source.generations.state` is the whole mechanism, and every read in this
//! file filters on it:
//!
//! ```text
//! provisional  rows are written, the `source.scanned` summary is not yet confirmed
//! confirmed    the summary exists; this generation is what coverage reports
//! evicted      superseded (bytes changed) or reconciled away (rows, no summary)
//! ```
//!
//! Nothing reads a `provisional` generation — [`AtlasDb::stage_scan`] writes
//! one and [`AtlasDb::confirm_scan`] promotes it. That is what makes a
//! half-finished scan *neither-reported* rather than half-reported: the rows
//! may exist for a while, but no read path can see them.
//!
//! **Recovery is not in this file, and that is deliberate.** Whether a
//! provisional generation a crash left behind should be promoted or evicted
//! is a question about the *journal* — the summary either got appended or it
//! did not — and this module owns a database, not a journal. So the state
//! column and the two primitives are here
//! ([`AtlasDb::provisional_generations`], [`AtlasDb::evict_provisional`]),
//! and the adjudication is
//! [`record::reconcile_sources`](crate::runtime::atlas::record::reconcile_sources),
//! which is the only place both halves of the evidence are in scope. Opening
//! this store therefore does **not** evict on its own: an evict-on-open would
//! destroy exactly the generations whose summary is sitting in the journal,
//! and "journal-present, database-evicted" is the same both-present violation
//! as its mirror image, only harder to notice.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use duckdb::types::Value as Duck;
use duckdb::{Connection, Statement};
use serde_json::{Map, Value, json};

use crate::domain::event::{Event, unix_millis};
use crate::domain::execution::{
    KIND_EXECUTION_RECONCILED, KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
};
use crate::domain::source::{
    AuthorityClass, Coverage, CoverageRow, SourceGeneration, SourceKind, UnitKind,
};
use crate::domain::work::{KIND_WORK_SUBMITTED, WorkState};
use crate::domain::workflow::{
    KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED,
    KIND_STAGE_FAILED, KIND_STAGE_NEEDS_INPUT, KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND,
};
use crate::runtime::atlas::external_git::ExternalGitProvenance;
use crate::runtime::atlas::scan::{ScannedFile, ScannedSyntax, ScannedUnit, SourceScan};
use crate::runtime::atlas::tabular::{
    DatasetFormat, RowKeyBasis, RowUnit, ScannedDataset, row_units,
};
use crate::runtime::fsutil::create_dir_all_durable;
use crate::runtime::graph::{
    GraphContext, KIND_CONVERSATION_ASSISTANT_COMPLETED, KIND_CONVERSATION_USER,
    KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};
use crate::runtime::journal::JournalError;
use crate::runtime::surface::{KIND_SURFACE_MATERIALIZED, KIND_SURFACE_TORN_DOWN};

/// Directory under the data dir holding Atlas's durable store.
///
/// Deliberately not
/// [`crate::runtime::startup::PROJECTIONS_DIR`](crate::runtime::startup::PROJECTIONS_DIR):
/// that directory is disposable by contract and this one is not. It is where
/// the FloorState cache still lives, and nothing else — the database that
/// used to sit beside it is this one.
pub const ATLAS_DIR: &str = "atlas";

/// Atlas's database file name inside [`ATLAS_DIR`].
pub const ATLAS_DB_FILE: &str = "atlas.duckdb";

/// The schema namespaces Atlas declares — A1 §5's five, in full.
///
/// `meta` holds Atlas's own bookkeeping (coverage above all); `ops` holds the
/// journal-derived Work/Stage/Execution/message/tool/usage projection;
/// `source` holds what was derived from source bytes; `git` holds what was
/// derived from Git objects; `context` holds the retrieval-facing units
/// assembled from the others. Sorted, because [`AtlasDb::schema_names`]
/// answers sorted and the two are compared directly.
///
/// `ops` is here because A1 §5 lists it here (S5 W1c, owner correction
/// 2026-08-29). It arrives as a *namespace* on every open even when this
/// process never folds the journal: the contract's claim is that one file
/// carries all five, and a namespace that only exists after the daemon has
/// rebuilt is a file that answers the contract's question differently
/// depending on who opened it last.
pub const SCHEMAS: &[&str] = &["context", "git", "meta", "ops", "source"];

/// Prepared statements the connection keeps planned.
const STATEMENT_CACHE: usize = 64;

/// Atlas's durable directory for a data dir.
pub fn atlas_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(ATLAS_DIR)
}

/// Atlas's database file for a data dir.
pub fn atlas_db_path(data_dir: &Path) -> PathBuf {
    atlas_dir(data_dir).join(ATLAS_DB_FILE)
}

/// Failures of the Atlas store.
#[derive(Debug, thiserror::Error)]
pub enum AtlasError {
    /// DuckDB refused a statement or could not open the database.
    #[error("atlas duckdb error: {0}")]
    Duck(#[from] duckdb::Error),
    /// Filesystem failure around Atlas's directory.
    #[error("atlas io error: {0}")]
    Io(#[from] std::io::Error),
    /// A path handed to a tabular reader carries a glob metacharacter.
    ///
    /// Refused, never escaped: DuckDB's `read_csv`/`read_json`/`read_parquet`
    /// expand their path argument as a multi-file pattern with no per-call way
    /// to turn that off, so such a path would read siblings nobody named. See
    /// [`GLOB_METACHARACTERS`].
    #[error("atlas: {DATASET_GLOB_PATH}: {path}")]
    GlobPath {
        /// The path as it would have been handed to the reader.
        path: String,
    },
    /// A stored row spells a value this build does not know — almost always
    /// a database written by a newer version. Refused by name rather than
    /// guessed at or silently skipped: a vocabulary this reader cannot
    /// interpret is not a row it may interpret approximately.
    #[error("atlas: column {column} holds unknown value {value:?}")]
    UnknownValue {
        /// Column the value came from.
        column: String,
        /// The value as stored.
        value: String,
    },
    /// [`AtlasDb::confirm_scan`] was handed a generation that is not
    /// `provisional` — already confirmed, already evicted, or never staged.
    ///
    /// Refused rather than absorbed **because the same call evicts a
    /// predecessor**. A promotion that matched no row would otherwise still
    /// delete the standing confirmed generation's units and files, leaving
    /// the source with nothing confirmed at all: data destroyed with nothing
    /// promoted in its place. The predecessor may only be evicted by the
    /// transaction that actually promoted its successor, so a promotion that
    /// changed no row takes nothing with it.
    #[error("atlas: generation {generation_id} is not awaiting confirmation")]
    NotProvisional {
        /// The generation the caller named.
        generation_id: String,
    },
}

/// The schema-namespace DDL, applied on every open.
///
/// `IF NOT EXISTS` throughout: unlike the operations projection, opening an
/// existing file is this store's normal path, so the DDL has to be
/// idempotent rather than run exactly once against a fresh file.
const SCHEMA_DDL: &str = "\
CREATE SCHEMA IF NOT EXISTS meta;\n\
CREATE SCHEMA IF NOT EXISTS ops;\n\
CREATE SCHEMA IF NOT EXISTS source;\n\
CREATE SCHEMA IF NOT EXISTS git;\n\
CREATE SCHEMA IF NOT EXISTS context;\n";

/// **F4's standing refusal: this database never reaches the network.**
///
/// DuckDB's default is to *autoload* a known extension the moment a query
/// needs one, and to *autoinstall* it — a download from an extension
/// repository — if it is not already on disk. That default is convenient and
/// completely unacceptable here: it would mean an operator's `sgt` reaching
/// the internet as a side effect of reading their own CSV, at a moment nobody
/// asked for a network call, with the fetched binary running at the daemon's
/// privilege.
///
/// So both are turned off on every connection, before any statement runs, and
/// community extensions with them. The consequence is deliberate and is the
/// reason F4 buys the `json`/`parquet` features at all: a reader is either
/// compiled into the bundled library or it does not exist for this process.
/// There is no third state where it appears later.
///
/// `lock_configuration` is set last and makes the three settings above
/// permanent for the life of the connection — a later `SET` cannot undo them,
/// so this is a property of the store rather than a convention its callers
/// are trusted to keep. `tests/x4_tabular_map.rs` pins all of it, including
/// the negative: an `https://` path is refused rather than fetched.
///
/// External *file* access stays on, because that is what "read in place"
/// means — the operator's own CSV, at the path their own manifest declared.
const HARDENING_DDL: &str = "\
SET autoinstall_known_extensions = false;\n\
SET autoload_known_extensions = false;\n\
SET allow_community_extensions = false;\n\
SET lock_configuration = true;\n";

/// The tables the scanners write — X2's four, plus X3b's three. Applied after
/// [`SCHEMA_DDL`], on every open, for the same idempotency reason.
///
/// Column choices worth stating, because each one is a contract:
///
/// * **`source.symbols` is the index; `source.occurrences` are the sites.**
///   A symbol is `(language, label, name)` — two files defining `count` are
///   one symbol row and two occurrence rows. The rollup is *syntactic*: it
///   says two sites wrote the same name in the same language with the same
///   grammar label, and it does **not** say they define the same thing, which
///   would be the resolution A1-09 forbids claiming. `language` is part of the
///   identity so two languages that spell a name alike are never merged.
/// * **An occurrence is a definition site**, because that is what a grammar
///   can tell you without resolving anything. Reference sites are absent
///   rather than approximated: an unresolved token stream labelled
///   "references" is exactly the false promise the empty-table doctrine
///   exists to refuse, and the wave that can resolve them adds them.
/// * **`syntax_key` is the F7 key of the *syntax* extraction**, which is not
///   `source.files.local_key`: one blob read by a structure extractor and by
///   a grammar is two extractions with two keys (see
///   [`ScannedSyntax`](crate::runtime::atlas::scan::ScannedSyntax)). Both are
///   derived from the same content identity — a blob OID for an estate-git
///   source, a BLAKE3 hash for a local one — composed with different extractor
///   identities.
/// * **`source.edges.target` is unresolved text**, exactly as the file wrote
///   it. `edge_kind` is `import`, the only kind this build derives.
///
/// * `content_key` on a generation is what ruling §4's eviction rule is
///   phrased over — a re-scan producing the same key evicts nothing.
/// * `summary_event_id` is nullable **on purpose**: its absence is precisely
///   "rows exist, this database has not seen a `source.scanned` summary" —
///   which is the crash window
///   [`record::reconcile_sources`](crate::runtime::atlas::record::reconcile_sources)
///   closes, by asking the journal whether the summary exists rather than
///   inferring from this column that it does not.
/// * `mtime_millis` is nullable and is a hint. Nothing joins on it, nothing
///   keys on it, and no reuse decision reads it (F7).
/// * `byte_start`/`byte_end` index the **original** file bytes, so a unit can
///   always be traced back to the resource it came from (A1 §3).
/// * `meta.coverage.path` is nullable: a null path is a generation-wide row,
///   which is how an eviction reports itself.
///
/// X4's three tabular tables, and why they are three (F12, A1 §6.4, F10a):
///
/// * **`source.datasets` is a registration, not a copy.** It records where a
///   dataset is, what it hashes to, which reader claims it, and what a bounded
///   in-place read *found* — its columns, a row count capped at [`MAX_ROWS`],
///   and whether that cap truncated the answer. The dataset's own rows are not
///   here and are not anywhere in this file: they stay in the operator's file,
///   which is what "read in place" means.
/// * **`source.dataset_facts` is derived evidence in A1 §6.4's shape.** Every
///   row carries the three things that make an answer checkable rather than
///   trusted: the input `generation_id` (which world), `query_identity`
///   (which question, at which version, over which exact SQL), and
///   `output_hash` (a digest of the answer itself). `row_limit`/`truncated`
///   are F12's bound, stored beside the answer because "the first 10,000 rows"
///   and "all the rows" are different facts and a reader must not have to
///   guess which one this is. Two invariants make that envelope worth
///   trusting: `row_limit` is the limit the statement *actually bound*, not a
///   constant copied in beside it, and `truncated` is **dataset-level** — it
///   says the input had more rows than `row_limit` covers, which is the only
///   useful reading for an aggregate whose answer is one row however much it
///   scanned.
/// * **`git.provenance` (S4 Y5, G6) is the first writer into the `git.*`
///   namespace** X1 declared and left empty. One row per `external_git`
///   generation, carrying A1 §9's provenance quintet minus the two fields
///   `source.generations` already has (`authority_class`, and the row's own
///   join key `source_name`): `origin` (verbatim, never normalized —
///   [`crate::runtime::atlas::locator`]'s own doc explains why),
///   `requested_ref` (`"HEAD"` when the operator asked for none, named
///   rather than left as an absent value a reader would have to interpret),
///   `resolved_commit` (a duplicate of `source.generations`' own eviction-safe
///   `content_key`'s revision half — this table's whole reason to exist is
///   X3b's rule two bullets up: "only ever added to, never altered" means a
///   new *fact* about a generation is a new table with its own copy of the
///   coordinates it needs, never a column bolted onto an existing one), and
///   `retrieved_at`. Written inside [`AtlasDb::stage_scan`]'s own staging
///   transaction ([`AtlasDb::stage_external_git_scan`]'s thin wrapper), so a
///   generation can never exist with no provenance row — the same
///   all-or-nothing atomicity F1 already gives every other row a generation
///   stages.
/// * **`context.row_units` is the F10a-gated bridge** and lives in the
///   `context` namespace because that is what it is: retrieval-facing text,
///   assembled from a source. It exists **only** for a dataset whose source
///   declared `context_fields`; with no allowlist the table stays empty for
///   that source, and that is the refusal, not an oversight. `key_basis` says
///   whether `row_key` is content-derived or had to fold in the ordinal — see
///   [`crate::runtime::atlas::tabular`] for why an S5 consumer needs to know.
const TABLE_DDL: &str = "\
CREATE TABLE IF NOT EXISTS source.generations (\n\
  generation_id    TEXT NOT NULL,\n\
  source_name      TEXT NOT NULL,\n\
  source_kind      TEXT NOT NULL,\n\
  authority_class  TEXT NOT NULL,\n\
  content_key      TEXT NOT NULL,\n\
  observed_at      TEXT NOT NULL,\n\
  state            TEXT NOT NULL,\n\
  summary_event_id TEXT,\n\
  extractors       TEXT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS source.files (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  relative_path TEXT NOT NULL,\n\
  content_hash  TEXT NOT NULL,\n\
  extractor     TEXT NOT NULL,\n\
  local_key     TEXT NOT NULL,\n\
  byte_len      BIGINT NOT NULL,\n\
  mtime_millis  BIGINT,\n\
  unit_count    BIGINT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS source.units (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  relative_path TEXT NOT NULL,\n\
  local_key     TEXT NOT NULL,\n\
  ordinal       BIGINT NOT NULL,\n\
  unit_kind     TEXT NOT NULL,\n\
  heading_level BIGINT,\n\
  title         TEXT,\n\
  byte_start    BIGINT NOT NULL,\n\
  byte_end      BIGINT NOT NULL,\n\
  body          TEXT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS source.symbols (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  language      TEXT NOT NULL,\n\
  label         TEXT NOT NULL,\n\
  name          TEXT NOT NULL,\n\
  occurrences   BIGINT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS source.occurrences (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  relative_path TEXT NOT NULL,\n\
  syntax_key    TEXT NOT NULL,\n\
  extractor     TEXT NOT NULL,\n\
  language      TEXT NOT NULL,\n\
  ordinal       BIGINT NOT NULL,\n\
  label         TEXT NOT NULL,\n\
  name          TEXT NOT NULL,\n\
  byte_start    BIGINT NOT NULL,\n\
  byte_end      BIGINT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS source.edges (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  relative_path TEXT NOT NULL,\n\
  syntax_key    TEXT NOT NULL,\n\
  extractor     TEXT NOT NULL,\n\
  language      TEXT NOT NULL,\n\
  ordinal       BIGINT NOT NULL,\n\
  edge_kind     TEXT NOT NULL,\n\
  target        TEXT NOT NULL,\n\
  byte_start    BIGINT NOT NULL,\n\
  byte_end      BIGINT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS meta.coverage (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  path          TEXT,\n\
  status        TEXT NOT NULL,\n\
  detail        TEXT,\n\
  bytes         BIGINT,\n\
  observed_at   TEXT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS source.datasets (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  relative_path TEXT NOT NULL,\n\
  format        TEXT NOT NULL,\n\
  content_hash  TEXT NOT NULL,\n\
  reader        TEXT NOT NULL,\n\
  dataset_key   TEXT NOT NULL,\n\
  byte_len      BIGINT NOT NULL,\n\
  mtime_millis  BIGINT,\n\
  columns       TEXT NOT NULL,\n\
  row_count     BIGINT NOT NULL,\n\
  truncated     BOOLEAN NOT NULL,\n\
  row_units     BIGINT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS source.dataset_facts (\n\
  generation_id  TEXT NOT NULL,\n\
  source_name    TEXT NOT NULL,\n\
  relative_path  TEXT NOT NULL,\n\
  dataset_key    TEXT NOT NULL,\n\
  query          TEXT NOT NULL,\n\
  query_identity TEXT NOT NULL,\n\
  row_limit      BIGINT NOT NULL,\n\
  truncated      BOOLEAN NOT NULL,\n\
  columns        TEXT NOT NULL,\n\
  rows           TEXT NOT NULL,\n\
  output_hash    TEXT NOT NULL,\n\
  observed_at    TEXT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS git.provenance (\n\
  generation_id   TEXT NOT NULL,\n\
  source_name     TEXT NOT NULL,\n\
  origin          TEXT NOT NULL,\n\
  requested_ref   TEXT NOT NULL,\n\
  resolved_commit TEXT NOT NULL,\n\
  retrieved_at    TEXT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS context.row_units (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  relative_path TEXT NOT NULL,\n\
  dataset_key   TEXT NOT NULL,\n\
  ordinal       BIGINT NOT NULL,\n\
  row_key       TEXT NOT NULL,\n\
  key_basis     TEXT NOT NULL,\n\
  fields        TEXT NOT NULL,\n\
  body          TEXT NOT NULL\n\
);\n";

/// A generation whose rows are written but whose summary is not yet journaled
/// (F1). Nothing reads it.
const STATE_PROVISIONAL: &str = "provisional";

/// A generation whose `source.scanned` summary exists. The only state any
/// read path here will look at.
const STATE_CONFIRMED: &str = "confirmed";

/// A generation whose rows have been removed — superseded, or reconciled away
/// after a crash. Kept as a row (with its coverage row) rather than deleted,
/// because "this world was observed and is no longer indexed" is evidence.
const STATE_EVICTED: &str = "evicted";

/// The default row ceiling on a read from this store (F12).
///
/// Every read here is bounded. Not a performance tuning figure — a refusal to
/// build an unbounded result set in a process that also serves a daemon: the
/// recon's own `unbounded-select()` risk. Callers that want fewer say so;
/// none can ask for "all".
pub const MAX_ROWS: usize = 10_000;

/// One canned question this build can ask of a tabular dataset, **in place**
/// (F11's "canned/parameterized only — never string-built SQL").
///
/// A fixed catalogue, for the same reason
/// [`CANNED_QUERIES`] is one: an endpoint that runs
/// a client's SQL hands back the one-owner property the whole architecture is
/// built on. Here it does worse than that — the SQL names a *file path*, so
/// arbitrary SQL against a dataset reader is arbitrary file access with the
/// daemon's privileges.
///
/// The SQL itself is not on this struct because there are three of it: the
/// table function is the reader's name (`read_csv`/`read_json`/`read_parquet`)
/// and a table function name cannot be a bind parameter. So each query is a
/// constant per format, selected by an enum ([`reader_call`]), and *every
/// value* — the path, the row cap — is bound. Nothing a caller supplies is
/// ever concatenated into a statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatasetQuery {
    /// Stable name, and the `query` column's value.
    pub name: &'static str,
    /// What it answers, in one sentence.
    pub question: &'static str,
    /// Bumped whenever the SQL changes meaning. Part of
    /// [`query_identity`], so an answer stored by an older build is
    /// distinguishable from a fresh one rather than silently comparable.
    pub version: &'static str,
}

/// **The deterministic aggregate** (X4's ≥1): per column, how many rows, how
/// many are non-null, and how many distinct values.
///
/// Deterministic in the sense that matters for evidence: the same file and the
/// same bound produce byte-identical output, because the aggregate is exact
/// (`count(DISTINCT …)`, not an approximate sketch) and the result is ordered
/// by column name rather than left in whatever order the reader emitted. Two
/// runs that disagree mean the file changed, which is the only thing a stored
/// [`output_hash`] is any use for.
///
/// It is also column-agnostic, which is what lets it be canned at all: the
/// rows are unpivoted into `(column_name, value)` pairs, so no column
/// *identifier* is ever interpolated into the statement. A profile that took
/// a column name as an argument would have to build SQL from it.
pub const DATASET_COLUMN_PROFILE: DatasetQuery = DatasetQuery {
    name: "column_profile",
    question: "For each column: how many rows, how many are non-null, and how many distinct values?",
    version: "v1",
};

/// The bounded row count: how many rows the reader produced, up to
/// [`MAX_ROWS`].
pub const DATASET_ROW_COUNT: DatasetQuery = DatasetQuery {
    name: "row_count",
    question: "How many rows does this dataset have, up to the row cap?",
    version: "v1",
};

/// Every canned dataset query, in the order a scan runs them.
pub const DATASET_QUERIES: &[DatasetQuery] = &[DATASET_ROW_COUNT, DATASET_COLUMN_PROFILE];

/// The table-function call for one format — the only thing that varies inside
/// a dataset query's SQL, and a compile-time constant chosen by an enum.
///
/// `union_by_name`/`filename` and the rest of DuckDB's reader options are
/// deliberately left at their defaults: an option this build does not
/// understand the failure modes of is not one it should be setting on an
/// operator's file.
fn reader_call(format: DatasetFormat) -> &'static str {
    match format {
        DatasetFormat::Csv => "read_csv(?, auto_detect = true)",
        DatasetFormat::Json => "read_json(?, auto_detect = true)",
        DatasetFormat::Parquet => "read_parquet(?)",
    }
}

/// `SELECT` list that casts every column to `VARCHAR`, bounded.
///
/// **Every canned dataset query answers in text, and that is a contract, not a
/// convenience.** What gets stored as derived evidence is text, and
/// [`output_hash`] hashes exactly what is stored — so an answer's digest
/// covers the answer a reader will actually see, with no formatting step in
/// between where two builds could disagree about how a `DOUBLE` renders.
fn rows_sql(format: DatasetFormat) -> String {
    format!(
        "SELECT COLUMNS(*)::VARCHAR FROM {} LIMIT ?",
        reader_call(format)
    )
}

/// [`DATASET_ROW_COUNT`]'s SQL for one format.
///
/// The count is taken over a *bounded* subquery, so a dataset far larger than
/// the cap costs one capped scan rather than a full one (F12). The caller asks
/// for `cap + 1` and learns from the answer whether the cap bit.
fn row_count_sql(format: DatasetFormat) -> String {
    format!(
        "SELECT count(*)::VARCHAR AS rows FROM (SELECT 1 FROM {} LIMIT ?)",
        reader_call(format)
    )
}

/// [`DATASET_COLUMN_PROFILE`]'s SQL for one format.
fn column_profile_sql(format: DatasetFormat) -> String {
    format!(
        "SELECT column_name, count(*)::VARCHAR AS rows, \
         count(value)::VARCHAR AS non_null_rows, \
         count(DISTINCT value)::VARCHAR AS distinct_values \
         FROM (SELECT COLUMNS(*)::VARCHAR FROM {} LIMIT ?) \
         UNPIVOT (value FOR column_name IN (COLUMNS(*))) \
         GROUP BY column_name ORDER BY column_name",
        reader_call(format)
    )
}

/// The SQL for one canned query over one format.
///
/// A `match` over the catalogue rather than a function pointer on
/// [`DatasetQuery`]: the catalogue is a `const`, and a `const` holding
/// function pointers is harder to read than the two arms it would replace.
fn sql_for(query: &DatasetQuery, format: DatasetFormat) -> String {
    if query.name == DATASET_ROW_COUNT.name {
        row_count_sql(format)
    } else {
        column_profile_sql(format)
    }
}

/// A canned query's identity: its name, its version, and a digest of the exact
/// SQL that ran (A1 §6.4's "which question produced this?").
///
/// The SQL digest is not redundant with the version. The version is a promise
/// a human keeps; the digest is a fact about the statement. If someone edits
/// the SQL and forgets the version bump, stored evidence still says the
/// question changed.
pub fn query_identity(query: &DatasetQuery, sql: &str) -> String {
    format!(
        "{}/{}#{}",
        query.name,
        query.version,
        blake3::hash(sql.as_bytes()).to_hex()
    )
}

/// A digest of one query result — A1 §6.4's output hash.
///
/// Over the columns *and* the rows, with domain separation and explicit
/// separators, so two different answers cannot hash alike by running their
/// fields together. `NULL` is folded in as its own marker rather than as an
/// empty string, because "no value" and "the empty string" are different
/// answers.
pub fn output_hash(columns: &[String], rows: &[Vec<Option<String>>]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.atlas.query-output/v1\n");
    for column in columns {
        hasher.update(column.as_bytes());
        hasher.update(b"\0");
    }
    hasher.update(b"\n");
    for row in rows {
        for value in row {
            match value {
                Some(value) => {
                    hasher.update(b"v");
                    hasher.update(value.as_bytes());
                }
                None => {
                    hasher.update(b"n");
                }
            }
            hasher.update(b"\0");
        }
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// One canned query's answer, with everything needed to check it (A1 §6.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetFact {
    /// Path relative to the source root.
    pub relative_path: String,
    /// F7's key for the dataset this answers about.
    pub dataset_key: String,
    /// Which canned query.
    pub query: String,
    /// Name, version and SQL digest ([`query_identity`]).
    pub query_identity: String,
    /// The row cap this answer was produced under (F12).
    pub row_limit: u64,
    /// Whether the cap bit — whether there was more the answer does not cover.
    pub truncated: bool,
    /// Column names, in order.
    pub columns: Vec<String>,
    /// Rows, aligned with `columns`. `None` is SQL `NULL`.
    pub rows: Vec<Vec<Option<String>>>,
    /// Digest of `columns` + `rows` ([`output_hash`]).
    pub output_hash: String,
}

/// The bootstrap DDL every fresh read-write connection onto `atlas.duckdb`
/// needs before its first query: F4's hardening settings, then A1 §5's five
/// schema namespaces, then Atlas's own tables — in that order, and always
/// together.
///
/// [`AtlasDb::over`] and [`Analytics::begin_rebuild`] both open a new
/// connection onto the one physical file and must apply exactly this
/// sequence; this is the one place it is spelled out, so a future DDL
/// addition to one caller cannot silently miss the other and leave the
/// file's shape depend on which struct opened it first.
fn bootstrap_atlas_ddl(conn: &Connection) -> Result<(), duckdb::Error> {
    conn.execute_batch(HARDENING_DDL)?;
    conn.execute_batch(SCHEMA_DDL)?;
    conn.execute_batch(TABLE_DDL)?;
    Ok(())
}

/// Atlas's database over one data dir.
///
/// Owns its connection privately; nothing hands a [`Connection`] out. Values
/// cross this boundary as plain Rust, the same rule the operations
/// projection holds for its own file.
pub struct AtlasDb {
    conn: Connection,
    path: PathBuf,
}

impl std::fmt::Debug for AtlasDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtlasDb")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl AtlasDb {
    /// Open (creating if absent) Atlas's database under `data_dir` and
    /// ensure its schema namespaces exist.
    ///
    /// Existing contents are kept. That is the F1 contract, not an
    /// optimization: what this file holds cannot be re-folded from the
    /// journal.
    /// **One live read-write handle per file, per process.** DuckDB gives
    /// each `Connection::open` its own database instance, so a second one
    /// against the same path neither sees nor is seen by the first, and the
    /// last close overwrites the other's work with no error anywhere. A
    /// process that already has the operations projection open must derive
    /// its Atlas handle from it — [`Analytics::atlas`] — rather than call
    /// this. This constructor is for a process that has no other handle:
    /// tests, and any tool that owns the file outright.
    pub fn open(data_dir: &Path) -> Result<Self, AtlasError> {
        create_dir_all_durable(&atlas_dir(data_dir))?;
        let path = atlas_db_path(data_dir);
        let conn = Connection::open(&path)?;
        Self::over(conn, path)
    }

    /// An in-memory Atlas database, for callers that want the namespaces
    /// without a file (tests, and any read-only rendering).
    pub fn open_in_memory() -> Result<Self, AtlasError> {
        let conn = Connection::open_in_memory()?;
        Self::over(conn, PathBuf::from(":memory:"))
    }

    /// Open an **existing** Atlas database read-only: no file is created, no
    /// directory is created, and no DDL runs (register row 12's S4 rider).
    ///
    /// [`Self::open`] is the daemon's own path and is allowed to create and
    /// migrate the store, because the daemon is Atlas's sole writer. Every
    /// other process — `sgt doctor`'s coverage row is the one caller — may
    /// only ever *read* it, and before this method existed that read went
    /// through [`Self::open`] anyway: a read-write connection that happened
    /// not to change anything, opened by a process that is not the writer.
    /// `CREATE SCHEMA IF NOT EXISTS`/`CREATE TABLE IF NOT EXISTS` are no-ops
    /// against a store the daemon already built, but "no-op" describes the
    /// result, not the access mode the statement demanded to run at all —
    /// this method demands none.
    ///
    /// Two consequences of asking DuckDB itself for `AccessMode::ReadOnly`,
    /// rather than merely skipping the DDL calls: a database that does not
    /// exist yet is refused outright instead of silently materialized (the
    /// caller checks [`atlas_db_path`] first, same as [`Self::open`]'s
    /// caller always has — this method does not repeat that check because it
    /// has no directory to create either), and a call that somehow reached a
    /// write path (there is none exposed from a `&self`-only reader, but the
    /// guarantee is DuckDB's enforcement rather than this crate's discipline)
    /// would be refused by the engine itself, not merely by convention.
    ///
    /// A daemon holding this same file for writing is not a conflict this
    /// method resolves — DuckDB's own locking decides whether a concurrent
    /// read-only open is possible, and a caller that gets [`AtlasError`] back
    /// is expected to treat it exactly as "the store could not be read from
    /// here right now," the same as any other open failure.
    pub fn open_read_only(data_dir: &Path) -> Result<Self, AtlasError> {
        let path = atlas_db_path(data_dir);
        let config = duckdb::Config::default().access_mode(duckdb::AccessMode::ReadOnly)?;
        let conn = Connection::open_with_flags(&path, config)?;
        conn.set_prepared_statement_cache_capacity(STATEMENT_CACHE);
        // F4's network-hardening settings only — no `SCHEMA_DDL`, no
        // `TABLE_DDL`. Both are genuine DDL and a read-only connection
        // cannot run them even under `IF NOT EXISTS`; skipping them here
        // rather than letting DuckDB refuse them is what keeps this path a
        // read, not a read that happens to trip over a write guard.
        conn.execute_batch(HARDENING_DDL)?;
        Ok(Self { conn, path })
    }

    fn over(conn: Connection, path: PathBuf) -> Result<Self, AtlasError> {
        conn.set_prepared_statement_cache_capacity(STATEMENT_CACHE);
        // First, before any other statement: extension autoloading and
        // autoinstalling are off, and locked off (F4). A connection that ran
        // one query before this ran is a connection that could have reached
        // the network once.
        bootstrap_atlas_ddl(&conn)?;
        // No reconciliation here. A provisional generation is already
        // unreadable — every read below filters on `state` — so nothing is
        // exposed by leaving one standing until the journal can be consulted,
        // and evicting one here would be a decision taken without the
        // evidence that decides it (see this module's doc, and
        // `record::reconcile_sources`, which the daemon runs at startup).
        Ok(Self { conn, path })
    }

    /// Where this database lives (`:memory:` for [`AtlasDb::open_in_memory`]).
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Every non-internal schema this database actually holds, sorted.
    ///
    /// Read back out of the catalog rather than echoed from [`SCHEMAS`], so
    /// a test comparing the two is checking the database and not the
    /// constant. DuckDB reports its own default `main` (and
    /// `information_schema`/`pg_catalog`) as internal, so what comes back is
    /// exactly what Atlas declared — no filtering of our own, which is what
    /// keeps a stray namespace visible here instead of quietly excluded.
    pub fn schema_names(&self) -> Result<Vec<String>, AtlasError> {
        let mut statement = self.conn.prepare(
            "SELECT schema_name FROM duckdb_schemas() \
             WHERE database_name = current_database() AND NOT internal \
             ORDER BY schema_name",
        )?;
        let mut rows = statement.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(row.get::<usize, String>(0)?);
        }
        Ok(out)
    }

    /// Step 1 of [`scan_and_record`](crate::runtime::atlas::record::scan_and_record):
    /// write a whole scan's rows in **one transaction**, as a `provisional`
    /// generation.
    ///
    /// Returns [`ScanCommit::Unchanged`] without writing anything when the
    /// scan's `content_key` matches the source's newest confirmed
    /// generation **and the same extractor identities produced it** — ruling
    /// §4's eviction rule, enforced at its only enforcement point: a
    /// generation is evicted only when the derived facts could have changed,
    /// so an unchanged re-scan must not churn one.
    ///
    /// **Both halves of F7's key participate, not just the content half.**
    /// F7 keys a derived row on content identity *plus extractor identity*,
    /// and a staleness test that consults only `content_key` would make the
    /// second half decorative: after a grammar or version bump, a re-scan of
    /// byte-identical sources would answer `Unchanged`, the fresh extraction
    /// would be discarded, and `symbols`/`occurrences`/`edges` would keep
    /// serving the *old* parser's rows forever with no path back. The
    /// identities the standing generation stored are therefore compared
    /// against the ones this scan ran, and a mismatch is a change: a new
    /// generation is staged and the old one evicted on confirmation, exactly
    /// as a byte change would be.
    ///
    /// The two inputs stay separate values rather than being folded into one
    /// hash. `content_key` answers "is this the same world?" and is
    /// deliberately content-only (see
    /// [`generation_key`](crate::domain::source::generation_key)); mixing an
    /// extractor version into it would make a bumped parser look like changed
    /// source bytes to every reader of that column, including the eviction
    /// row that has to say *why* a generation was superseded.
    ///
    /// Returns [`ScanCommit::RootUnavailable`], also without writing a
    /// generation, when the walk could not read the source root at all and a
    /// confirmed generation already stands — see
    /// [`SourceScan::root_unavailable`]. An unplugged drive changed no source
    /// bytes, so ruling §4 gives it no eviction: the standing generation is
    /// what still has evidence behind it, and superseding it with an empty
    /// scan would destroy exactly the derived facts F1 exists to keep across
    /// restarts. The unavailability is recorded as a coverage row against the
    /// generation that survived it, so "this source was unreachable at this
    /// time, and we kept what we had" is queryable rather than inferred from
    /// an absence.
    ///
    /// One transaction is not an optimization. Partial rows for a generation
    /// nothing has confirmed would be indistinguishable from a completed one
    /// after the state column was promoted — the atomic batch is what makes
    /// "provisional" mean "all of it, or none of it, awaiting a summary".
    pub fn stage_scan(&mut self, scan: &SourceScan) -> Result<ScanCommit, AtlasError> {
        self.stage_scan_impl(scan, None)
    }

    /// [`Self::stage_scan`] for an `external_git` scan, with A1 §9's
    /// provenance written **inside the same staging transaction** — see the
    /// module doc's `git.provenance` bullet for why this cannot be a
    /// follow-up write after [`Self::stage_scan`] returns. Same
    /// [`ScanCommit`] outcomes, same unchanged/root-unavailable/staged
    /// three-way split; `provenance` is written only for the `Staged` case,
    /// exactly like every other row this transaction produces.
    pub fn stage_external_git_scan(
        &mut self,
        scan: &SourceScan,
        provenance: &ExternalGitProvenance,
    ) -> Result<ScanCommit, AtlasError> {
        self.stage_scan_impl(scan, Some(provenance))
    }

    fn stage_scan_impl(
        &mut self,
        scan: &SourceScan,
        provenance: Option<&ExternalGitProvenance>,
    ) -> Result<ScanCommit, AtlasError> {
        // Asked before the `content_key` comparison below, because the two
        // answers are indistinguishable by key: an unreachable root and an
        // emptied one both hash an empty resource map, and only this row can
        // tell them apart.
        if let Some(unavailable) = scan.root_unavailable()
            && let Some(current) = self.confirmed_generation(&scan.source_name)?
        {
            let detail = format!(
                "the source root was unreadable on this scan ({}); \
                 the confirmed generation was kept — no source bytes changed",
                unavailable.detail.as_deref().unwrap_or("no detail")
            );
            insert_coverage(
                &self.conn,
                &current.id,
                &scan.source_name,
                &CoverageRow {
                    path: None,
                    status: Coverage::Unavailable,
                    detail: Some(detail.clone()),
                    bytes: None,
                },
                &scan.observed_at,
            )?;
            return Ok(ScanCommit::RootUnavailable {
                generation_id: current.id,
                content_key: current.content_key,
                detail,
            });
        }
        if let Some(current) = self.confirmed_generation(&scan.source_name)?
            && current.content_key == scan.content_key
            && self.generation_extractors(&current.id)? == scan.extractors
        {
            return Ok(ScanCommit::Unchanged {
                generation_id: current.id,
                content_key: current.content_key,
            });
        }
        let generation_id = ulid::Ulid::generate().to_string();
        let extractors = join_extractors(&scan.extractors);
        // X4: the datasets are read **before** the transaction opens, and
        // never inside it. A failing statement aborts the transaction it ran
        // in, so one unreadable CSV read inside the staging transaction would
        // take every other row of the scan down with it — see
        // [`read_dataset`], which is where that argument lives.
        let reads: Vec<IngestedDataset> = scan
            .datasets
            .iter()
            .map(|dataset| read_dataset(&self.conn, scan, dataset))
            .collect();
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO source.generations \
             (generation_id, source_name, source_kind, authority_class, content_key, \
              observed_at, state, summary_event_id, extractors) \
             VALUES (?, ?, ?, ?, ?, ?, ?, NULL, ?)",
            duckdb::params![
                &generation_id,
                &scan.source_name,
                scan.kind.as_str(),
                scan.authority.as_str(),
                &scan.content_key,
                &scan.observed_at,
                STATE_PROVISIONAL,
                &extractors,
            ],
        )?;
        // The symbol *index* is a rollup across the whole generation, so it is
        // accumulated as the files are written and inserted once — not once
        // per file, which would make `occurrences` a per-file count wearing a
        // generation-wide column's name.
        let mut index: BTreeMap<(&str, &str, &str), u64> = BTreeMap::new();
        for file in &scan.files {
            insert_file(&tx, &generation_id, &scan.source_name, file)?;
            if let Some(syntax) = &file.syntax {
                for symbol in &syntax.symbols {
                    *index
                        .entry((syntax.language, symbol.label, symbol.name.as_str()))
                        .or_insert(0) += 1;
                }
            }
        }
        for ((language, label, name), occurrences) in index {
            tx.prepare_cached(
                "INSERT INTO source.symbols \
                 (generation_id, source_name, language, label, name, occurrences) \
                 VALUES (?, ?, ?, ?, ?, ?)",
            )?
            .execute(duckdb::params![
                &generation_id,
                &scan.source_name,
                language,
                label,
                name,
                occurrences as i64,
            ])?;
        }
        // X4: the datasets, read **in place**. This is the one place in the
        // build where an extractor is a SQL reader rather than Rust over
        // bytes, and it is here rather than in the walk for the reason the
        // one-owner rule gives: the reader is the database.
        //
        // Each dataset's outcome replaces the placeholder coverage row the
        // walk left for that path, below, so F8's one-row-per-path rule holds
        // and the row says what actually happened rather than what was
        // attempted.
        let mut outcomes: BTreeMap<String, CoverageRow> = BTreeMap::new();
        for (dataset, read) in scan.datasets.iter().zip(&reads) {
            let outcome = write_dataset(&tx, &generation_id, scan, dataset, read)?;
            outcomes.insert(dataset.relative_path.clone(), outcome);
        }
        for row in &scan.coverage {
            let row = row
                .path
                .as_deref()
                .and_then(|path| outcomes.get(path))
                .unwrap_or(row);
            insert_coverage(
                &tx,
                &generation_id,
                &scan.source_name,
                row,
                &scan.observed_at,
            )?;
        }
        if let Some(provenance) = provenance {
            tx.execute(
                "INSERT INTO git.provenance \
                 (generation_id, source_name, origin, requested_ref, resolved_commit, \
                  retrieved_at) \
                 VALUES (?, ?, ?, ?, ?, ?)",
                duckdb::params![
                    &generation_id,
                    &scan.source_name,
                    &provenance.origin,
                    &provenance.requested_ref,
                    &provenance.resolved_commit,
                    &provenance.retrieved_at,
                ],
            )?;
        }
        tx.commit()?;
        Ok(ScanCommit::Staged { generation_id })
    }

    /// Step 3: promote a staged generation to `confirmed` and evict the one it
    /// supersedes — **in one transaction**, so "confirmed" and "predecessor
    /// evicted" are never separately observable.
    ///
    /// Returns the evicted predecessor's id, if there was one. Eviction here
    /// rather than at staging time is the deliberate half of F1's ordering:
    /// a crash before this point leaves the previous generation standing,
    /// which is the answer that still has evidence behind it.
    ///
    /// **The promotion is checked, not assumed.** The `UPDATE` is gated on
    /// `state = 'provisional'`, and its affected-row count decides whether
    /// the eviction runs at all: a caller naming a generation that is no
    /// longer provisional — a stale id after a reconcile, a retry after a
    /// partial failure — gets [`AtlasError::NotProvisional`] and an untouched
    /// database, rather than a predecessor deleted on behalf of a successor
    /// that was never promoted. Today's sole caller cannot reach that state;
    /// this method is public on the store that owns the only copy of these
    /// rows, and an invariant that only holds because of who happens to call
    /// it is not an invariant.
    pub fn confirm_scan(
        &mut self,
        generation_id: &str,
        summary_event_id: &str,
    ) -> Result<Option<String>, AtlasError> {
        let source_name = self.generation_source(generation_id)?;
        let superseded = match &source_name {
            Some(name) => self
                .confirmed_generation(name)?
                .map(|generation| generation.id)
                .filter(|id| id != generation_id),
            None => None,
        };
        // Why the predecessor is going, decided from the evidence rather than
        // assumed. `stage_scan` stages a successor for either of two reasons —
        // the source bytes changed, or the extractor identities did — and the
        // coverage row an eviction leaves is the durable record of which. A
        // fixed "the source bytes changed" string would be a false statement on
        // every grammar bump, in the one row a reader consults to find out.
        let reason = match &superseded {
            Some(previous)
                if self.generation_content_key(previous)?
                    == self.generation_content_key(generation_id)? =>
            {
                "superseded: the extractor identities changed (the source bytes did not)"
            }
            _ => "superseded: the source bytes changed",
        };
        let observed_at = crate::domain::event::rfc3339_utc_now();
        let tx = self.conn.transaction()?;
        let promoted = tx.execute(
            "UPDATE source.generations SET state = ?, summary_event_id = ? \
             WHERE generation_id = ? AND state = ?",
            duckdb::params![
                STATE_CONFIRMED,
                summary_event_id,
                generation_id,
                STATE_PROVISIONAL
            ],
        )?;
        if promoted == 0 {
            // Nothing was promoted, so nothing may be evicted. Rolling back
            // rather than committing an empty transaction keeps the two
            // halves inseparable in both directions.
            tx.rollback()?;
            return Err(AtlasError::NotProvisional {
                generation_id: generation_id.to_string(),
            });
        }
        if let (Some(previous), Some(name)) = (&superseded, &source_name) {
            evict(&tx, previous, name, reason, &observed_at)?;
        }
        tx.commit()?;
        Ok(superseded)
    }

    /// Every generation still awaiting confirmation, newest first, as
    /// `(generation_id, source_name)`.
    ///
    /// Half of F1's startup reconciliation — the half a database can answer.
    /// The other half is whether the journal holds each one's
    /// `source.scanned` summary, which decides promotion versus eviction and
    /// is not this module's to know
    /// ([`record::reconcile_sources`](crate::runtime::atlas::record::reconcile_sources)).
    ///
    /// The common answer is an empty vector, and that matters: it is what
    /// lets the reconciler skip reading the journal entirely on every start
    /// that did not follow a crash.
    pub fn provisional_generations(&self) -> Result<Vec<(String, String)>, AtlasError> {
        self.generations_in_state(STATE_PROVISIONAL)
    }

    /// Evict named generations that are still `provisional`, in one
    /// transaction, each leaving an explicit `generation_evicted` coverage
    /// row carrying `reason`.
    ///
    /// Ruling §4's "reported, never a silent gap": an operator reading
    /// coverage can tell a superseded generation from one a crash cost them,
    /// because the reason says which.
    ///
    /// Only `provisional` ids are touched. A confirmed generation is never
    /// evictable this way — the sole path that supersedes one is
    /// [`Self::confirm_scan`], in the same transaction that promotes its
    /// replacement. Returns the ids actually evicted.
    pub fn evict_provisional(
        &mut self,
        generation_ids: &[String],
        reason: &str,
    ) -> Result<Vec<String>, AtlasError> {
        let awaiting = self.generations_in_state(STATE_PROVISIONAL)?;
        let targets: Vec<(String, String)> = awaiting
            .into_iter()
            .filter(|(id, _)| generation_ids.iter().any(|wanted| wanted == id))
            .collect();
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        let observed_at = crate::domain::event::rfc3339_utc_now();
        let tx = self.conn.transaction()?;
        for (generation_id, source_name) in &targets {
            evict(&tx, generation_id, source_name, reason, &observed_at)?;
        }
        tx.commit()?;
        Ok(targets.into_iter().map(|(id, _)| id).collect())
    }

    /// Evict **every** generation belonging to one Work's overlay sources, in
    /// one transaction, each leaving its own `generation_evicted` coverage row.
    ///
    /// A Work overlay describes a world only that Work can see: its base tree
    /// plus that surface's uncommitted changes. When the Work is gone, the
    /// world is gone, and rows describing it are no longer derived evidence
    /// about anything — they are claims about a surface that does not exist.
    /// So this is the one eviction that is *not* keyed on source bytes
    /// changing (ruling §4's rule, which governs a durable source); an overlay
    /// generation's lifetime is its Work's, and that is the rule for it.
    ///
    /// **Confirmed generations included**, unlike [`Self::evict_provisional`].
    /// That difference is the whole point: an overlay's confirmed rows are
    /// exactly the ones that must not outlive their Work.
    ///
    /// The read is **range-bounded on the overlay prefix itself** — plain
    /// parameterized comparisons, so F12's rule against string-built SQL
    /// holds without a `LIKE` (a Work id interpolated into a pattern would
    /// let `%` and `_` mean things a caller passing an id never meant), and
    /// the row cap bounds *this Work's* generations rather than the whole
    /// estate. A cap shared with every other source's live generations could
    /// push a retiring Work's overlays out of the window, and a confirmed
    /// generation that survives here outlives its Work — exactly what this
    /// eviction exists to forbid. The `starts_with` check stays as a belt
    /// over the range's braces.
    pub fn evict_work_overlays(&mut self, work_id: &str) -> Result<Vec<String>, AtlasError> {
        let prefix = crate::runtime::atlas::overlay::overlay_source_prefix(work_id);
        // The prefix ends in '/', so its exclusive upper bound is the same
        // string with the final byte bumped to '0' ('/' + 1).
        let mut upper = prefix.clone();
        let last = upper.pop().expect("overlay prefix is never empty");
        upper.push((last as u8 + 1) as char);
        let mut statement = self.conn.prepare(
            "SELECT generation_id, source_name FROM source.generations \
             WHERE state != ? AND source_name >= ? AND source_name < ? \
             ORDER BY observed_at DESC, generation_id DESC LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![
            STATE_EVICTED,
            prefix,
            upper,
            MAX_ROWS as i64
        ])?;
        let mut targets: Vec<(String, String)> = Vec::new();
        while let Some(row) = rows.next()? {
            let source_name: String = row.get(1)?;
            if source_name.starts_with(&prefix) {
                targets.push((row.get(0)?, source_name));
            }
        }
        drop(rows);
        drop(statement);
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        let reason = format!("the Work this overlay was scoped to ({work_id}) was retired");
        let observed_at = crate::domain::event::rfc3339_utc_now();
        let tx = self.conn.transaction()?;
        for (generation_id, source_name) in &targets {
            evict(&tx, generation_id, source_name, &reason, &observed_at)?;
        }
        tx.commit()?;
        Ok(targets.into_iter().map(|(id, _)| id).collect())
    }

    /// What a `--work` answer actually covers, read from the store rather
    /// than derived from the selector alone (S5 W1b).
    ///
    /// [`WorkScope`]'s whole job is to be TRUE about the answer beside it,
    /// and whether an overlay stands for a Work is a fact about the store,
    /// not about the filter a caller typed: the Work may not have bound a
    /// surface yet, its overlay scan may have failed, or the Work may have
    /// retired and had its overlay evicted with it
    /// ([`Self::evict_work_overlays`]). So this asks. One extra bounded
    /// read per admissibility call, on the same connection.
    ///
    /// **Also true about the store, and just as load-bearing: whether the
    /// TABLE this particular call reads can carry an overlay-authored row at
    /// all.** An overlay generation stands for exactly one repository
    /// (S5 W1b's own fix — see [`SourceSelector::overlay_admit_source_name`]),
    /// is always stamped `SourceKind::EstateGit` / `AuthorityClass::
    /// EstateMutable` ([`crate::runtime::atlas::overlay::scan_work_overlay`]),
    /// and never writes a `source.datasets` row at all — an overlay's
    /// unchanged bytes come from the base tree's objects, so its own scan
    /// records `datasets: Vec::new()` unconditionally. So an overlay
    /// standing is not, by itself, enough to say an answer includes it:
    ///
    /// - `carries_overlay_rows: false` — this table (`source.datasets`) is
    ///   one an overlay scan structurally never populates. `BaseOnly`
    ///   always, regardless of whether an overlay stands.
    /// - `filter.kind` narrowed to anything but [`SourceKind::EstateGit`],
    ///   or `filter.authority` narrowed to anything but
    ///   [`AuthorityClass::EstateMutable`] — the caller's own stage-2/4
    ///   filter structurally excludes every row an overlay could ever have
    ///   written. `BaseOnly` for the same reason.
    ///
    /// Asserting `BaseAndOverlaySnapshot` in either case would claim the
    /// answer reflects overlay evidence as of a given instant when no row it
    /// could contain was ever capable of coming from the overlay — the same
    /// class of false claim [`WorkScope`]'s own doc names for "current"
    /// dressed up as a snapshot.
    fn work_scope(
        &self,
        filter: &Admissibility,
        carries_overlay_rows: bool,
    ) -> Result<WorkScope, AtlasError> {
        let SourceSelector::WorkBase {
            work_id,
            repository,
        } = &filter.source
        else {
            return Ok(WorkScope::NotWorkScoped);
        };
        if !carries_overlay_rows
            || filter
                .kind
                .is_some_and(|kind| kind != SourceKind::EstateGit)
            || filter
                .authority
                .is_some_and(|authority| authority != AuthorityClass::EstateMutable)
        {
            return Ok(WorkScope::BaseOnly);
        }
        Ok(
            match self.newest_overlay_observed_at(work_id, repository)? {
                Some(overlay_observed_at) => WorkScope::BaseAndOverlaySnapshot {
                    overlay_observed_at,
                },
                None => WorkScope::BaseOnly,
            },
        )
    }

    /// When this Work's overlay half — over the one `repository` a
    /// [`SourceSelector::WorkBase`] names — was last read off its surface:
    /// the matching CONFIRMED `work:<id>/<repo>` generation's `observed_at`,
    /// or `None` when no such generation stands.
    ///
    /// An exact lookup on the one source name
    /// [`overlay_source_name`](crate::runtime::atlas::overlay::overlay_source_name)
    /// can ever produce for this `(work_id, repository)` pair, not a
    /// `work_id`-only prefix scan — the earlier prefix form answered about
    /// *any* repository under this Work id, over-claiming past the
    /// repository the caller actually asked about, the sibling of the
    /// admission bug [`SourceSelector::overlay_admit_source_name`] fixes.
    fn newest_overlay_observed_at(
        &self,
        work_id: &str,
        repository: &str,
    ) -> Result<Option<String>, AtlasError> {
        let source_name = crate::runtime::atlas::overlay::overlay_source_name(work_id, repository);
        let mut statement = self.conn.prepare(
            "SELECT observed_at FROM source.generations \
             WHERE state = ? AND source_name = ? \
             ORDER BY observed_at DESC, generation_id DESC LIMIT 1",
        )?;
        let mut rows = statement.query(duckdb::params![STATE_CONFIRMED, source_name])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// Record that a Work overlay could **not** be read off its surface at
    /// all, as a coverage row against the overlay generation that survived
    /// the attempt (S5 W1b).
    ///
    /// The same posture [`Self::stage_scan`] already takes for an
    /// unreachable source root, for the same reason: a surface that could
    /// not be read changed no recorded bytes, so ruling §4 gives it no
    /// eviction, and the standing generation keeps its evidence. The
    /// failure becomes queryable coverage rather than an absence a reader
    /// has to infer.
    ///
    /// Returns the generation the row landed on, or `None` when **no**
    /// overlay generation stands for this source. That case deliberately
    /// writes nothing: there is no generation for a coverage row to attach
    /// to, and staging an empty one would make `Self::work_scope` report
    /// [`WorkScope::BaseAndOverlaySnapshot`] for a surface that was never
    /// read — a false claim about the answer, which is the one thing this
    /// wave's whole freshness semantic exists to prevent. The caller
    /// reports it (the daemon's own log) and `--work` degrades to
    /// [`WorkScope::BaseOnly`], which is exactly true.
    pub fn record_overlay_unavailable(
        &mut self,
        source_name: &str,
        detail: &str,
    ) -> Result<Option<String>, AtlasError> {
        let Some(current) = self.confirmed_generation(source_name)? else {
            return Ok(None);
        };
        insert_coverage(
            &self.conn,
            &current.id,
            source_name,
            &CoverageRow {
                path: None,
                status: Coverage::Unavailable,
                detail: Some(detail.to_string()),
                bytes: None,
            },
            &crate::domain::event::rfc3339_utc_now(),
        )?;
        Ok(Some(current.id))
    }

    /// The newest **confirmed** generation for one source, if there is one.
    ///
    /// Every read below goes through the same filter. A provisional or
    /// evicted generation is never what coverage or retrieval answers with,
    /// which is the property that makes a crash mid-scan invisible rather
    /// than half-visible.
    pub fn confirmed_generation(
        &self,
        source_name: &str,
    ) -> Result<Option<SourceGeneration>, AtlasError> {
        let mut statement = self.conn.prepare(
            "SELECT generation_id, source_name, source_kind, authority_class, content_key, \
                    observed_at \
             FROM source.generations WHERE source_name = ? AND state = ? \
             ORDER BY observed_at DESC, generation_id DESC LIMIT 1",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let kind: String = row.get(2)?;
        let authority: String = row.get(3)?;
        Ok(Some(SourceGeneration {
            id: row.get(0)?,
            source_name: row.get(1)?,
            kind: SourceKind::parse(&kind).ok_or_else(|| AtlasError::UnknownValue {
                column: "source_kind".to_string(),
                value: kind.clone(),
            })?,
            authority: AuthorityClass::parse(&authority).ok_or_else(|| {
                AtlasError::UnknownValue {
                    column: "authority_class".to_string(),
                    value: authority.clone(),
                }
            })?,
            content_key: row.get(4)?,
            observed_at: row.get(5)?,
        }))
    }

    /// Units of one source's confirmed generation, in path then ordinal
    /// order, bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn units(&self, source_name: &str, limit: usize) -> Result<Vec<StoredUnit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT u.relative_path, u.local_key, u.ordinal, u.unit_kind, u.heading_level, \
                    u.title, u.byte_start, u.byte_end, u.body \
             FROM source.units u JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY u.relative_path, u.ordinal LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let kind: String = row.get(3)?;
            out.push(StoredUnit {
                relative_path: row.get(0)?,
                local_key: row.get(1)?,
                ordinal: row.get::<usize, i64>(2)? as u64,
                kind: UnitKind::parse(&kind).ok_or_else(|| AtlasError::UnknownValue {
                    column: "unit_kind".to_string(),
                    value: kind.clone(),
                })?,
                heading_level: row.get::<usize, Option<i64>>(4)?.map(|v| v as u8),
                title: row.get(5)?,
                byte_start: row.get::<usize, i64>(6)? as u64,
                byte_end: row.get::<usize, i64>(7)? as u64,
                body: row.get(8)?,
            });
        }
        Ok(out)
    }

    /// The symbol index of one source's confirmed generation, in
    /// `(language, label, name)` order, bounded by `limit` (capped at
    /// [`MAX_ROWS`], F12).
    pub fn symbols(
        &self,
        source_name: &str,
        limit: usize,
    ) -> Result<Vec<StoredSymbol>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT s.language, s.label, s.name, s.occurrences \
             FROM source.symbols s JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY s.language, s.label, s.name LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(StoredSymbol {
                language: row.get(0)?,
                label: row.get(1)?,
                name: row.get(2)?,
                occurrences: row.get::<usize, i64>(3)? as u64,
            });
        }
        Ok(out)
    }

    /// Symbol sites of one source's confirmed generation, in path then
    /// document order, bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn occurrences(
        &self,
        source_name: &str,
        limit: usize,
    ) -> Result<Vec<StoredOccurrence>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT o.relative_path, o.syntax_key, o.extractor, o.language, o.ordinal, \
                    o.label, o.name, o.byte_start, o.byte_end \
             FROM source.occurrences o JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY o.relative_path, o.ordinal LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(StoredOccurrence {
                relative_path: row.get(0)?,
                syntax_key: row.get(1)?,
                extractor: row.get(2)?,
                language: row.get(3)?,
                ordinal: row.get::<usize, i64>(4)? as u64,
                label: row.get(5)?,
                name: row.get(6)?,
                byte_start: row.get::<usize, i64>(7)? as u64,
                byte_end: row.get::<usize, i64>(8)? as u64,
            });
        }
        Ok(out)
    }

    /// Edges out of one source's confirmed generation, in path then document
    /// order, bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn edges(&self, source_name: &str, limit: usize) -> Result<Vec<StoredEdge>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT e.relative_path, e.syntax_key, e.extractor, e.language, e.ordinal, \
                    e.edge_kind, e.target, e.byte_start, e.byte_end \
             FROM source.edges e JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY e.relative_path, e.ordinal LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(StoredEdge {
                relative_path: row.get(0)?,
                syntax_key: row.get(1)?,
                extractor: row.get(2)?,
                language: row.get(3)?,
                ordinal: row.get::<usize, i64>(4)? as u64,
                kind: row.get(5)?,
                target: row.get(6)?,
                byte_start: row.get::<usize, i64>(7)? as u64,
                byte_end: row.get::<usize, i64>(8)? as u64,
            });
        }
        Ok(out)
    }

    /// Coverage rows for one source's confirmed generations, newest first,
    /// bounded by `limit` (capped at [`MAX_ROWS`], F12).
    ///
    /// Confirmed **and evicted** generations both answer here, unlike
    /// [`Self::units`]: an eviction row exists precisely so a source that
    /// used to be indexed and no longer is says so out loud.
    pub fn coverage(
        &self,
        source_name: &str,
        limit: usize,
    ) -> Result<Vec<StoredCoverage>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT c.generation_id, c.path, c.status, c.detail, c.bytes, c.observed_at \
             FROM meta.coverage c JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state IN (?, ?) \
             ORDER BY c.observed_at DESC, c.path NULLS FIRST LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![
            source_name,
            STATE_CONFIRMED,
            STATE_EVICTED,
            limit
        ])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let status: String = row.get(2)?;
            out.push(StoredCoverage {
                generation_id: row.get(0)?,
                row: CoverageRow {
                    path: row.get(1)?,
                    status: Coverage::parse(&status).ok_or_else(|| AtlasError::UnknownValue {
                        column: "status".to_string(),
                        value: status.clone(),
                    })?,
                    detail: row.get(3)?,
                    bytes: row.get::<usize, Option<i64>>(4)?.map(|v| v as u64),
                },
                observed_at: row.get(5)?,
            });
        }
        Ok(out)
    }

    /// Coverage counts by status for one source's confirmed generation —
    /// what `sgt intelligence status` and the doctor row will read (F8).
    pub fn coverage_counts(&self, source_name: &str) -> Result<BTreeMap<String, u64>, AtlasError> {
        let mut statement = self.conn.prepare(
            "SELECT c.status, count(*) FROM meta.coverage c \
             JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? GROUP BY c.status ORDER BY c.status",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED])?;
        let mut out = BTreeMap::new();
        while let Some(row) = rows.next()? {
            out.insert(
                row.get::<usize, String>(0)?,
                row.get::<usize, i64>(1)? as u64,
            );
        }
        Ok(out)
    }

    /// **F4, read back off the live connection**: what this store's extension
    /// posture actually is, rather than what [`HARDENING_DDL`] intended.
    ///
    /// Read out of DuckDB's own settings and catalogue for the same reason
    /// [`Self::schema_names`] reads out of the catalogue: a test comparing
    /// this against the constant would be checking the constant. `locked` is
    /// the load-bearing one — with it true, no later `SET` can re-enable
    /// autoloading, so the refusal is a property of the connection instead of
    /// a convention its callers are trusted to keep.
    pub fn hardening(&self) -> Result<Hardening, AtlasError> {
        let mut statement = self.conn.prepare(
            "SELECT current_setting('autoinstall_known_extensions')::BOOLEAN, \
                    current_setting('autoload_known_extensions')::BOOLEAN, \
                    current_setting('allow_community_extensions')::BOOLEAN, \
                    current_setting('lock_configuration')::BOOLEAN",
        )?;
        let mut rows = statement.query([])?;
        let row = rows.next()?.ok_or_else(|| AtlasError::UnknownValue {
            column: "current_setting".to_string(),
            value: "no row".to_string(),
        })?;
        let posture = Hardening {
            autoinstall_known_extensions: row.get(0)?,
            autoload_known_extensions: row.get(1)?,
            allow_community_extensions: row.get(2)?,
            locked: row.get(3)?,
            statically_linked: Vec::new(),
        };
        drop(rows);
        drop(statement);
        let mut statement = self.conn.prepare(
            "SELECT extension_name FROM duckdb_extensions() \
             WHERE loaded AND install_mode = 'STATICALLY_LINKED' \
             ORDER BY extension_name LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![MAX_ROWS as i64])?;
        let mut statically_linked = Vec::new();
        while let Some(row) = rows.next()? {
            statically_linked.push(row.get::<usize, String>(0)?);
        }
        Ok(Hardening {
            statically_linked,
            ..posture
        })
    }

    /// Run one canned query against one dataset file, in place (F12).
    ///
    /// The bounded, parameterized read [`Self::stage_scan`] uses, exposed so a
    /// caller can ask a canned question of a dataset without a scan — and so
    /// F4's refusal can be tested on the negative it is actually about, a path
    /// that is not a local file.
    ///
    /// **`path` is a file path this process will open**, so this is not, and
    /// must not become, an HTTP-reachable surface: the daemon's `map` and
    /// `intelligence` routes read rows this store already holds, never a path
    /// a client named. What is safe about it is what F11 asks for — the
    /// *query* is canned and every value is bound — which is a different
    /// property from the path being safe.
    pub fn dataset_probe(
        &self,
        format: DatasetFormat,
        path: &str,
        query: &DatasetQuery,
    ) -> Result<DatasetFact, AtlasError> {
        if path.contains(GLOB_METACHARACTERS) {
            return Err(AtlasError::GlobPath {
                path: path.to_string(),
            });
        }
        let sql = sql_for(query, format);
        let dataset = ScannedDataset {
            relative_path: path.to_string(),
            format,
            content_hash: String::new(),
            reader: format.reader_version().to_string(),
            dataset_key: String::new(),
            byte_len: 0,
            mtime_millis: None,
        };
        // The same two-step [`read_dataset`] uses, for the same reason: the
        // envelope a fact carries has to describe the read that produced it,
        // and only the count probe can say whether the input outran the cap.
        let (count_columns, row_count, truncated) = dataset_bound(&self.conn, format, path)?;
        if query.name == DATASET_ROW_COUNT.name {
            return Ok(counted_fact(
                query,
                &sql,
                &dataset,
                &count_columns,
                row_count,
                truncated,
            ));
        }
        dataset_fact(
            &self.conn,
            query,
            &sql,
            &dataset,
            path,
            MAX_ROWS as i64,
            truncated,
        )
    }

    /// Registered datasets of one source's confirmed generation, in path
    /// order, bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn datasets(
        &self,
        source_name: &str,
        limit: usize,
    ) -> Result<Vec<StoredDataset>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT d.relative_path, d.format, d.content_hash, d.reader, d.dataset_key, \
                    d.byte_len, d.columns, d.row_count, d.truncated, d.row_units \
             FROM source.datasets d JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY d.relative_path LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let format: String = row.get(1)?;
            out.push(StoredDataset {
                relative_path: row.get(0)?,
                format: DatasetFormat::parse(&format).ok_or_else(|| AtlasError::UnknownValue {
                    column: "format".to_string(),
                    value: format.clone(),
                })?,
                content_hash: row.get(2)?,
                reader: row.get(3)?,
                dataset_key: row.get(4)?,
                byte_len: row.get::<usize, i64>(5)? as u64,
                columns: split_names(&row.get::<usize, String>(6)?),
                row_count: row.get::<usize, i64>(7)? as u64,
                truncated: row.get(8)?,
                row_units: row.get::<usize, i64>(9)? as u64,
            });
        }
        Ok(out)
    }

    /// Derived evidence for one source's confirmed generation — every canned
    /// query's answer, with the identity and output hash that make it
    /// checkable (A1 §6.4). Bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn dataset_facts(
        &self,
        source_name: &str,
        limit: usize,
    ) -> Result<Vec<DatasetFact>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT f.relative_path, f.dataset_key, f.query, f.query_identity, f.row_limit, \
                    f.truncated, f.columns, f.rows, f.output_hash \
             FROM source.dataset_facts f JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY f.relative_path, f.query LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(DatasetFact {
                relative_path: row.get(0)?,
                dataset_key: row.get(1)?,
                query: row.get(2)?,
                query_identity: row.get(3)?,
                row_limit: row.get::<usize, i64>(4)? as u64,
                truncated: row.get(5)?,
                columns: split_names(&row.get::<usize, String>(6)?),
                rows: parse_rows(&row.get::<usize, String>(7)?),
                output_hash: row.get(8)?,
            });
        }
        Ok(out)
    }

    /// **F10a's observable half**: the context units one source's confirmed
    /// generation exposes from its tabular rows.
    ///
    /// A source that declared no `context_fields` answers with an empty
    /// vector, always — not because this query filtered them out, but because
    /// nothing wrote any. Bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn row_units(
        &self,
        source_name: &str,
        limit: usize,
    ) -> Result<Vec<StoredRowUnit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT r.relative_path, r.dataset_key, r.ordinal, r.row_key, r.key_basis, \
                    r.fields, r.body \
             FROM context.row_units r JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY r.relative_path, r.ordinal LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let basis: String = row.get(4)?;
            out.push(StoredRowUnit {
                relative_path: row.get(0)?,
                dataset_key: row.get(1)?,
                ordinal: row.get::<usize, i64>(2)? as u64,
                row_key: row.get(3)?,
                basis: RowKeyBasis::parse(&basis).ok_or_else(|| AtlasError::UnknownValue {
                    column: "key_basis".to_string(),
                    value: basis.clone(),
                })?,
                fields: split_names(&row.get::<usize, String>(5)?),
                body: row.get(6)?,
            });
        }
        Ok(out)
    }

    /// Every source with a confirmed generation, and what that generation
    /// holds — the read behind `sgt intelligence status`, `sgt map repos` and
    /// `sgt map stats` (F8, F11).
    ///
    /// The counts are correlated subqueries over one generation rather than
    /// separate statements, so a source's numbers all describe the same world.
    /// Bounded by [`MAX_ROWS`] (F12).
    pub fn indexed_sources(&self) -> Result<Vec<SourceStatus>, AtlasError> {
        let mut statement = self.conn.prepare(
            "SELECT g.source_name, g.source_kind, g.authority_class, g.generation_id, \
                    g.content_key, g.observed_at, g.extractors, \
                    (SELECT count(*) FROM source.files f \
                       WHERE f.generation_id = g.generation_id) AS files, \
                    (SELECT count(*) FROM source.units u \
                       WHERE u.generation_id = g.generation_id) AS units, \
                    (SELECT count(*) FROM source.symbols s \
                       WHERE s.generation_id = g.generation_id) AS symbols, \
                    (SELECT count(*) FROM source.occurrences o \
                       WHERE o.generation_id = g.generation_id) AS occurrences, \
                    (SELECT count(*) FROM source.edges e \
                       WHERE e.generation_id = g.generation_id) AS edges, \
                    (SELECT count(*) FROM source.datasets d \
                       WHERE d.generation_id = g.generation_id) AS datasets, \
                    (SELECT count(*) FROM context.row_units r \
                       WHERE r.generation_id = g.generation_id) AS row_units, \
                    p.origin, p.requested_ref, p.resolved_commit, p.retrieved_at \
             FROM source.generations g \
             LEFT JOIN git.provenance p ON p.generation_id = g.generation_id \
             WHERE g.state = ? \
             ORDER BY g.source_name LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![STATE_CONFIRMED, MAX_ROWS as i64])?;
        let mut out: Vec<SourceStatus> = Vec::new();
        while let Some(row) = rows.next()? {
            let kind: String = row.get(1)?;
            let authority: String = row.get(2)?;
            let origin: Option<String> = row.get(14)?;
            let requested_ref: Option<String> = row.get(15)?;
            let resolved_commit: Option<String> = row.get(16)?;
            let retrieved_at: Option<String> = row.get(17)?;
            let provenance = match (origin, requested_ref, resolved_commit, retrieved_at) {
                (Some(origin), Some(requested_ref), Some(resolved_commit), Some(retrieved_at)) => {
                    Some(SourceProvenance {
                        origin,
                        requested_ref,
                        resolved_commit,
                        retrieved_at,
                    })
                }
                _ => None,
            };
            out.push(SourceStatus {
                source_name: row.get(0)?,
                kind: SourceKind::parse(&kind).ok_or_else(|| AtlasError::UnknownValue {
                    column: "source_kind".to_string(),
                    value: kind.clone(),
                })?,
                authority: AuthorityClass::parse(&authority).ok_or_else(|| {
                    AtlasError::UnknownValue {
                        column: "authority_class".to_string(),
                        value: authority.clone(),
                    }
                })?,
                generation_id: row.get(3)?,
                content_key: row.get(4)?,
                observed_at: row.get(5)?,
                extractors: split_extractors(&row.get::<usize, String>(6)?)
                    .into_iter()
                    .collect(),
                files: row.get::<usize, i64>(7)? as u64,
                units: row.get::<usize, i64>(8)? as u64,
                symbols: row.get::<usize, i64>(9)? as u64,
                occurrences: row.get::<usize, i64>(10)? as u64,
                edges: row.get::<usize, i64>(11)? as u64,
                datasets: row.get::<usize, i64>(12)? as u64,
                row_units: row.get::<usize, i64>(13)? as u64,
                coverage: BTreeMap::new(),
                provenance,
            });
        }
        drop(rows);
        drop(statement);
        // Coverage counts per source, after the cursor above is finished with:
        // F8's rows are the point of `intelligence status`, and a status line
        // without them would report what was indexed while staying silent
        // about what was excluded.
        for status in &mut out {
            status.coverage = self.coverage_counts(&status.source_name)?;
        }
        Ok(out)
    }

    /// `sgt map outline <source>`: the titled structure units of one source's
    /// confirmed generation, in path then document order.
    ///
    /// Titled only. A whole-document unit and an untitled preamble are real
    /// units but they are not an outline, and padding the answer with them
    /// would make the first screen of a large source useless. Bounded by
    /// `limit` (capped at [`MAX_ROWS`], F12).
    pub fn outline(&self, source_name: &str, limit: usize) -> Result<Vec<StoredUnit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT u.relative_path, u.local_key, u.ordinal, u.unit_kind, u.heading_level, \
                    u.title, u.byte_start, u.byte_end \
             FROM source.units u JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? AND u.title IS NOT NULL \
             ORDER BY u.relative_path, u.ordinal LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let kind: String = row.get(3)?;
            out.push(StoredUnit {
                relative_path: row.get(0)?,
                local_key: row.get(1)?,
                ordinal: row.get::<usize, i64>(2)? as u64,
                kind: UnitKind::parse(&kind).ok_or_else(|| AtlasError::UnknownValue {
                    column: "unit_kind".to_string(),
                    value: kind.clone(),
                })?,
                heading_level: row.get::<usize, Option<i64>>(4)?.map(|v| v as u8),
                title: row.get(5)?,
                byte_start: row.get::<usize, i64>(6)? as u64,
                byte_end: row.get::<usize, i64>(7)? as u64,
                // An outline is a table of contents, not the document: the
                // body is deliberately not selected, so `map outline` on a
                // large source cannot return the source.
                body: String::new(),
            });
        }
        Ok(out)
    }

    /// `sgt map symbol <name>`: the symbol index across every source, for one
    /// **exact** name.
    ///
    /// Exact equality, bound as a parameter — never a `LIKE` pattern built
    /// from the argument, which would let `%` and `_` mean things the caller
    /// never wrote (the argument [`Self::evict_work_overlays`] already makes).
    /// Bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn symbol_lookup(
        &self,
        name: &str,
        limit: usize,
    ) -> Result<Vec<StoredSymbolHit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT g.source_name, s.language, s.label, s.name, s.occurrences \
             FROM source.symbols s JOIN source.generations g USING (generation_id) \
             WHERE g.state = ? AND s.name = ? \
             ORDER BY g.source_name, s.language, s.label LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![STATE_CONFIRMED, name, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(StoredSymbolHit {
                source_name: row.get(0)?,
                symbol: StoredSymbol {
                    language: row.get(1)?,
                    label: row.get(2)?,
                    name: row.get(3)?,
                    occurrences: row.get::<usize, i64>(4)? as u64,
                },
            });
        }
        Ok(out)
    }

    /// `sgt map references <name>`: every recorded site of one exact symbol
    /// name, across sources.
    ///
    /// **These are definition sites, not resolved references**, because a
    /// definition site is what a grammar can state without resolving anything
    /// (A1-09, and `source.occurrences`' own contract). The verb is named for
    /// what a reader goes looking for; the answer says what it actually is.
    /// Bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn references(&self, name: &str, limit: usize) -> Result<Vec<StoredReference>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(
            "SELECT g.source_name, o.relative_path, o.language, o.label, o.name, o.ordinal, \
                    o.byte_start, o.byte_end \
             FROM source.occurrences o JOIN source.generations g USING (generation_id) \
             WHERE g.state = ? AND o.name = ? \
             ORDER BY g.source_name, o.relative_path, o.ordinal LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![STATE_CONFIRMED, name, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(StoredReference {
                source_name: row.get(0)?,
                relative_path: row.get(1)?,
                language: row.get(2)?,
                label: row.get(3)?,
                name: row.get(4)?,
                ordinal: row.get::<usize, i64>(5)? as u64,
                byte_start: row.get::<usize, i64>(6)? as u64,
                byte_end: row.get::<usize, i64>(7)? as u64,
            });
        }
        Ok(out)
    }

    // ------------------------------------------------------------------
    // S5 W1 — A2 §2's deterministic admissibility filter (H1, H13.1).
    //
    // "The first operation is deterministic admissibility, not embeddings":
    // source/estate/Work-generation filter, then authority filter, then
    // content-kind filter, then the optional repo/knowledge/external
    // selector — every stage a database predicate that decides what may be
    // SEEN, never a ranking hint (A2 §8: a reranker must never cross this
    // boundary because a candidate scores well). No scoring, no ranking, no
    // BM25, no embeddings live here or anywhere in this wave — those are
    // W2-W4. Every method below answers with a COMPLETE, EXACT admissible
    // set: additive SQL on the cross-source join precedent
    // [`Self::symbol_lookup`]/[`Self::references`] already established
    // (H1), never a new table or a new column (H13.1).
    // ------------------------------------------------------------------

    /// The fixed, code-owned `NOT LIKE` bound every admissibility query
    /// below applies to `source_name`, excluding the whole Work-overlay
    /// family ([`crate::runtime::atlas::overlay::OVERLAY_PREFIX`],
    /// `work:<id>/<repo>`) — see [`Self::admissible_generations`]'s own doc
    /// for what re-admits exactly one Work's own overlay on top of it
    /// ([`SourceSelector::overlay_admit_source_name`], S5 W1b) and why the
    /// default-deny stays the default. Derived from the overlay module's
    /// own prefix constant rather than a second hardcoded literal, so the
    /// two can never drift apart; still never a client-supplied pattern
    /// (F12), the same precedent as [`CODE_EXTRACTOR_LIKE`].
    fn overlay_exclude_like() -> String {
        format!("{}%", crate::runtime::atlas::overlay::OVERLAY_PREFIX)
    }

    /// A2 §2 stages 1(+4) and 2 in one canned query: the source/estate/
    /// Work-generation filter, the optional repo/knowledge/external
    /// selector ([`Admissibility::kind`], composable with any
    /// [`SourceSelector`]), and the authority filter — composed once here
    /// and reused, in identical shape, by every content-kind method below,
    /// so a generation excluded at this stage can never resurface through
    /// a different table.
    ///
    /// Every clause is `(? IS NULL OR column = ?)`: an unset filter field
    /// admits every value of that column rather than narrowing it, so
    /// `Admissibility::default()` (bare [`SourceSelector::Any`], no
    /// authority) is "every confirmed generation this store holds" — never
    /// approximate, never partial. Bounded by `limit` (capped at
    /// [`MAX_ROWS`], F12).
    ///
    /// **The Work-overlay family is denied by default, and exactly one
    /// Work's own overlay is re-admitted on top of that — never by name.**
    /// A generation whose `source_name` carries
    /// [`crate::runtime::atlas::overlay::OVERLAY_PREFIX`]
    /// (`work:<id>/<repo>`) describes a world only one Work's surface can
    /// see (H13.2). The composed predicate is
    ///
    /// ```text
    /// (source_name NOT LIKE 'work:%' AND (?src IS NULL OR source_name = ?src))
    ///   OR (?admit IS NOT NULL AND source_name = ?admit)
    /// ```
    ///
    /// where `?admit` is `Some("work:<id>/<repository>")` — the *exact*
    /// overlay source name, never a pattern — **only** for
    /// [`SourceSelector::WorkBase`], built from that variant's own
    /// `work_id` **and** `repository`
    /// (`SourceSelector::overlay_admit_source_name`). So:
    ///
    /// - [`SourceSelector::Named`]/[`SourceSelector::Exact`] can never
    ///   reach an overlay, not even naming the exact coordinate — a caller
    ///   who merely learns another Work's id (e.g. from `sgt work list`)
    ///   must not be able to type it into `--source` and read that Work's
    ///   surface. `?admit` is `None` for those variants, so the left
    ///   branch is the only one available and it denies the whole family.
    /// - `--work <mine>` admits `mine`'s base generation and `mine`'s
    ///   overlay over exactly the repository `WorkBase` names. It does
    ///   **not** admit another Work's overlay over the same repository
    ///   (`work:<other>/repo-a` fails both branches), and — because `?admit`
    ///   is an exact name rather than a `work:<id>/%` prefix — it does
    ///   **not** admit `mine`'s own overlay over a *different* repository
    ///   either: `WorkBase { work_id: "mine", repository: "repo-a" }`
    ///   admits only `work:mine/repo-a`, never `work:mine/repo-b`, matching
    ///   the base half's own restriction to one named repository
    ///   ([`SourceSelector::bindings`]).
    ///
    /// S5 W1b is what made the right branch worth having: until its
    /// daemon-side lifecycle hook landed, no overlay generation was ever
    /// written outside a test. `sgt search` remains a pure reader either
    /// way — this is a `SELECT` predicate, and nothing on any query path
    /// writes (H13.2).
    pub fn admissible_generations(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<SourceGeneration>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let (source_name, content_key) = filter.source.bindings();
        let source_kind = filter.kind.map(SourceKind::as_str);
        let authority = filter.authority.map(AuthorityClass::as_str);
        let overlay_exclude = Self::overlay_exclude_like();
        let overlay_admit = filter.source.overlay_admit_source_name();
        let mut statement = self.conn.prepare(
            "SELECT generation_id, source_name, source_kind, authority_class, content_key, \
                    observed_at \
             FROM source.generations \
             WHERE state = ? \
               AND ( (source_name NOT LIKE ? \
                      AND (? IS NULL OR source_name = ?)) \
                     OR (? IS NOT NULL AND source_name = ?) ) \
               AND (? IS NULL OR content_key = ?) \
               AND (? IS NULL OR source_kind = ?) \
               AND (? IS NULL OR authority_class = ?) \
             ORDER BY source_name, observed_at DESC, generation_id DESC LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![
            STATE_CONFIRMED,
            overlay_exclude,
            source_name,
            source_name,
            &overlay_admit,
            &overlay_admit,
            content_key,
            content_key,
            source_kind,
            source_kind,
            authority,
            authority,
            limit
        ])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let kind: String = row.get(2)?;
            let auth: String = row.get(3)?;
            out.push(SourceGeneration {
                id: row.get(0)?,
                source_name: row.get(1)?,
                kind: SourceKind::parse(&kind).ok_or_else(|| AtlasError::UnknownValue {
                    column: "source_kind".to_string(),
                    value: kind.clone(),
                })?,
                authority: AuthorityClass::parse(&auth).ok_or_else(|| {
                    AtlasError::UnknownValue {
                        column: "authority_class".to_string(),
                        value: auth.clone(),
                    }
                })?,
                content_key: row.get(4)?,
                observed_at: row.get(5)?,
            });
        }
        Ok(Admitted {
            hits: out,
            scope: self.work_scope(filter, true)?,
        })
    }

    /// A2 §2's content-kind filter, **document family** — H13.1's decided
    /// mechanism: table-routing (`source.units` is physically separate from
    /// the code and tabular families, so the coarse split needs no new
    /// column) plus an extractor-identity allowlist joined off
    /// `source.files`, pinned by [`DOCUMENT_EXTRACTOR_IDENTITIES`] and its
    /// own structural test (`tests/w1_deterministic_filter.rs`).
    ///
    /// **The allowlist is a safety net, not a clean split — verified live,
    /// correcting a premise H13.1's own text carried.** `claims_for`
    /// (`src/runtime/atlas/scan.rs`) gives every grammar-claimed-but-
    /// document-unclaimed file (`main.rs`, `Cargo.toml`) a plain-text
    /// fallback unit under [`crate::runtime::atlas::text::TEXT_EXTRACTOR`]
    /// so "every acquired resource still has units" — checked directly in
    /// this worktree: a `Cargo.toml` fixture produces exactly one
    /// `Document` unit (extractor `"text/v1"`, body = the whole file) in
    /// *addition* to its `source.occurrences` rows under `"syntax-toml/v1"`.
    /// That is the *same* extractor identity a genuine `.txt` document
    /// carries, so this filter cannot separate "real prose" from a
    /// code/config file's catch-all body — no `extractor` value
    /// distinguishes them — and it does not try to. H13.1's "no new
    /// column" holds regardless: the gap is named here, not engineered
    /// around with state this wave was told not to add.
    ///
    /// Stages 1/2/4 come from `filter`, identically to
    /// [`Self::admissible_generations`]. Bounded by `limit` (capped at
    /// [`MAX_ROWS`], F12).
    pub fn admissible_units(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<StoredUnitHit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let (source_name, content_key) = filter.source.bindings();
        let source_kind = filter.kind.map(SourceKind::as_str);
        let authority = filter.authority.map(AuthorityClass::as_str);
        let [doc_a, doc_b, doc_c, doc_d] = DOCUMENT_EXTRACTOR_IDENTITIES;
        let overlay_exclude = Self::overlay_exclude_like();
        let overlay_admit = filter.source.overlay_admit_source_name();
        let mut statement = self.conn.prepare(
            "SELECT g.source_name, u.relative_path, u.local_key, u.ordinal, u.unit_kind, \
                    u.heading_level, u.title, u.byte_start, u.byte_end, u.body \
             FROM source.units u \
             JOIN source.generations g USING (generation_id) \
             JOIN source.files f ON f.generation_id = u.generation_id \
                                 AND f.relative_path = u.relative_path \
             WHERE g.state = ? \
               AND f.extractor IN (?, ?, ?, ?) \
               AND ( (g.source_name NOT LIKE ? \
                      AND (? IS NULL OR g.source_name = ?)) \
                     OR (? IS NOT NULL AND g.source_name = ?) ) \
               AND (? IS NULL OR g.content_key = ?) \
               AND (? IS NULL OR g.source_kind = ?) \
               AND (? IS NULL OR g.authority_class = ?) \
             ORDER BY g.source_name, u.relative_path, u.ordinal LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![
            STATE_CONFIRMED,
            doc_a,
            doc_b,
            doc_c,
            doc_d,
            overlay_exclude,
            source_name,
            source_name,
            &overlay_admit,
            &overlay_admit,
            content_key,
            content_key,
            source_kind,
            source_kind,
            authority,
            authority,
            limit
        ])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let kind: String = row.get(4)?;
            out.push(StoredUnitHit {
                source_name: row.get(0)?,
                unit: StoredUnit {
                    relative_path: row.get(1)?,
                    local_key: row.get(2)?,
                    ordinal: row.get::<usize, i64>(3)? as u64,
                    kind: UnitKind::parse(&kind).ok_or_else(|| AtlasError::UnknownValue {
                        column: "unit_kind".to_string(),
                        value: kind.clone(),
                    })?,
                    heading_level: row.get::<usize, Option<i64>>(5)?.map(|v| v as u8),
                    title: row.get(6)?,
                    byte_start: row.get::<usize, i64>(7)? as u64,
                    byte_end: row.get::<usize, i64>(8)? as u64,
                    body: row.get(9)?,
                },
            });
        }
        Ok(Admitted {
            hits: out,
            scope: self.work_scope(filter, true)?,
        })
    }

    /// A2 §2's content-kind filter, **code family** — `source.symbols` +
    /// `source.occurrences` + `source.edges`, physically separate from the
    /// document and tabular families (H13.1, no new column). This method
    /// reads `source.occurrences`; `symbols`/`edges` follow the identical
    /// shape and are not duplicated here (R1 — nothing in this wave's
    /// negative-admission proof needs them; each is a mechanical variant of
    /// this one for a later wave to add on demand).
    ///
    /// The extractor match is `extractor LIKE ?` bound to
    /// [`CODE_EXTRACTOR_LIKE`] (`"syntax-%"`) — a fixed, code-owned pattern
    /// (F12: never a client-supplied pattern), pinned by a structural test
    /// against every
    /// [`crate::runtime::atlas::syntax::SyntaxLanguage::ALL`] identity.
    /// **This is also where a `.toml` config file's occurrences live**
    /// (H13.1's decided exception): a config file's key/table structure is
    /// claimed by
    /// [`crate::runtime::atlas::syntax::SyntaxLanguage::Toml`] the same way
    /// Rust is claimed by
    /// [`crate::runtime::atlas::syntax::SyntaxLanguage::Rust`], under
    /// extractor identity `"syntax-toml/v1"` — matched by this same `LIKE`
    /// pattern. `--content config` has no document-side backing (see
    /// [`Self::admissible_units`]'s own doc) and is not offered as a
    /// distinct value; a caller wanting config content calls this method,
    /// exactly as for code.
    ///
    /// Stages 1/2/4 come from `filter`, identically to
    /// [`Self::admissible_generations`]. Bounded by `limit` (capped at
    /// [`MAX_ROWS`], F12).
    pub fn admissible_occurrences(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<StoredOccurrenceHit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let (source_name, content_key) = filter.source.bindings();
        let source_kind = filter.kind.map(SourceKind::as_str);
        let authority = filter.authority.map(AuthorityClass::as_str);
        let overlay_exclude = Self::overlay_exclude_like();
        let overlay_admit = filter.source.overlay_admit_source_name();
        let mut statement = self.conn.prepare(
            "SELECT g.source_name, o.relative_path, o.syntax_key, o.extractor, o.language, \
                    o.ordinal, o.label, o.name, o.byte_start, o.byte_end \
             FROM source.occurrences o JOIN source.generations g USING (generation_id) \
             WHERE g.state = ? \
               AND o.extractor LIKE ? \
               AND ( (g.source_name NOT LIKE ? \
                      AND (? IS NULL OR g.source_name = ?)) \
                     OR (? IS NOT NULL AND g.source_name = ?) ) \
               AND (? IS NULL OR g.content_key = ?) \
               AND (? IS NULL OR g.source_kind = ?) \
               AND (? IS NULL OR g.authority_class = ?) \
             ORDER BY g.source_name, o.relative_path, o.ordinal LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![
            STATE_CONFIRMED,
            CODE_EXTRACTOR_LIKE,
            overlay_exclude,
            source_name,
            source_name,
            &overlay_admit,
            &overlay_admit,
            content_key,
            content_key,
            source_kind,
            source_kind,
            authority,
            authority,
            limit
        ])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(StoredOccurrenceHit {
                source_name: row.get(0)?,
                occurrence: StoredOccurrence {
                    relative_path: row.get(1)?,
                    syntax_key: row.get(2)?,
                    extractor: row.get(3)?,
                    language: row.get(4)?,
                    ordinal: row.get::<usize, i64>(5)? as u64,
                    label: row.get(6)?,
                    name: row.get(7)?,
                    byte_start: row.get::<usize, i64>(8)? as u64,
                    byte_end: row.get::<usize, i64>(9)? as u64,
                },
            });
        }
        Ok(Admitted {
            hits: out,
            scope: self.work_scope(filter, true)?,
        })
    }

    /// A2 §2's content-kind filter, **tabular family** — `source.datasets`
    /// (+ `context.row_units`, not read here — see
    /// [`Self::admissible_occurrences`]'s note on why the whole family is
    /// not duplicated). No extractor ambiguity here: `source.datasets`
    /// carries no `extractor` column at all (`format`/`reader` are a
    /// different axis), so table-routing alone is exact for this family
    /// (H13.1).
    ///
    /// Stages 1/2/4 come from `filter`, identically to
    /// [`Self::admissible_generations`]. Bounded by `limit` (capped at
    /// [`MAX_ROWS`], F12).
    pub fn admissible_datasets(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<StoredDatasetHit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let (source_name, content_key) = filter.source.bindings();
        let source_kind = filter.kind.map(SourceKind::as_str);
        let authority = filter.authority.map(AuthorityClass::as_str);
        let overlay_exclude = Self::overlay_exclude_like();
        let overlay_admit = filter.source.overlay_admit_source_name();
        let mut statement = self.conn.prepare(
            "SELECT g.source_name, d.relative_path, d.format, d.content_hash, d.reader, \
                    d.dataset_key, d.byte_len, d.columns, d.row_count, d.truncated, d.row_units \
             FROM source.datasets d JOIN source.generations g USING (generation_id) \
             WHERE g.state = ? \
               AND ( (g.source_name NOT LIKE ? \
                      AND (? IS NULL OR g.source_name = ?)) \
                     OR (? IS NOT NULL AND g.source_name = ?) ) \
               AND (? IS NULL OR g.content_key = ?) \
               AND (? IS NULL OR g.source_kind = ?) \
               AND (? IS NULL OR g.authority_class = ?) \
             ORDER BY g.source_name, d.relative_path LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![
            STATE_CONFIRMED,
            overlay_exclude,
            source_name,
            source_name,
            &overlay_admit,
            &overlay_admit,
            content_key,
            content_key,
            source_kind,
            source_kind,
            authority,
            authority,
            limit
        ])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let format: String = row.get(2)?;
            out.push(StoredDatasetHit {
                source_name: row.get(0)?,
                dataset: StoredDataset {
                    relative_path: row.get(1)?,
                    format: DatasetFormat::parse(&format).ok_or_else(|| {
                        AtlasError::UnknownValue {
                            column: "format".to_string(),
                            value: format.clone(),
                        }
                    })?,
                    content_hash: row.get(3)?,
                    reader: row.get(4)?,
                    dataset_key: row.get(5)?,
                    byte_len: row.get::<usize, i64>(6)? as u64,
                    columns: split_names(&row.get::<usize, String>(7)?),
                    row_count: row.get::<usize, i64>(8)? as u64,
                    truncated: row.get(9)?,
                    row_units: row.get::<usize, i64>(10)? as u64,
                },
            });
        }
        Ok(Admitted {
            hits: out,
            scope: self.work_scope(filter, false)?,
        })
    }

    /// Every generation's state, keyed by id — a diagnostic read, and what a
    /// crash-window test inspects.
    pub fn generation_states(&self) -> Result<BTreeMap<String, String>, AtlasError> {
        let mut statement = self.conn.prepare(
            "SELECT generation_id, state FROM source.generations \
             ORDER BY observed_at, generation_id LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![MAX_ROWS as i64])?;
        let mut out = BTreeMap::new();
        while let Some(row) = rows.next()? {
            out.insert(row.get::<usize, String>(0)?, row.get::<usize, String>(1)?);
        }
        Ok(out)
    }

    /// Ids and source names of every generation in one state.
    fn generations_in_state(&self, state: &str) -> Result<Vec<(String, String)>, AtlasError> {
        let mut statement = self.conn.prepare(
            "SELECT generation_id, source_name FROM source.generations WHERE state = ? \
             ORDER BY observed_at DESC, generation_id DESC LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![state, MAX_ROWS as i64])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push((row.get(0)?, row.get(1)?));
        }
        Ok(out)
    }

    /// Which source a generation belongs to.
    fn generation_source(&self, generation_id: &str) -> Result<Option<String>, AtlasError> {
        let mut statement = self
            .conn
            .prepare("SELECT source_name FROM source.generations WHERE generation_id = ?")?;
        let mut rows = statement.query(duckdb::params![generation_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// The extractor identities a stored generation's rows were derived from —
    /// the second half of F7's key, read back for [`Self::stage_scan`]'s
    /// staleness test.
    ///
    /// Round-trips exactly what [`join_extractors`] wrote, so the comparison is
    /// against a set and not against a formatting accident. An unknown
    /// generation and one that recorded no extractors both answer with the
    /// empty set, which is the same answer a scan that ran none produces —
    /// correct in both cases, because neither could have derived a row.
    fn generation_extractors(&self, generation_id: &str) -> Result<BTreeSet<String>, AtlasError> {
        let mut statement = self
            .conn
            .prepare("SELECT extractors FROM source.generations WHERE generation_id = ?")?;
        let mut rows = statement.query(duckdb::params![generation_id])?;
        let Some(row) = rows.next()? else {
            return Ok(BTreeSet::new());
        };
        Ok(split_extractors(&row.get::<usize, String>(0)?))
    }

    /// The stored `content_key` of any generation, in any state.
    ///
    /// [`Self::confirmed_generation`] cannot answer this for a generation that
    /// is being superseded *by* the call asking, which is exactly when
    /// [`Self::confirm_scan`] needs it to say honestly why the predecessor is
    /// going.
    fn generation_content_key(&self, generation_id: &str) -> Result<Option<String>, AtlasError> {
        let mut statement = self
            .conn
            .prepare("SELECT content_key FROM source.generations WHERE generation_id = ?")?;
        let mut rows = statement.query(duckdb::params![generation_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }
}

/// The `extractors` column's stored form: identities, comma-joined, in the
/// set's own sorted order.
fn join_extractors(extractors: &BTreeSet<String>) -> String {
    extractors
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(",")
}

/// [`join_extractors`] backwards. Empty segments are dropped, so an empty
/// column is the empty set rather than a set holding one empty name.
fn split_extractors(stored: &str) -> BTreeSet<String> {
    stored
        .split(',')
        .filter(|identity| !identity.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Insert one acquired file and its units.
fn insert_file(
    conn: &Connection,
    generation_id: &str,
    source_name: &str,
    file: &ScannedFile,
) -> Result<(), AtlasError> {
    conn.prepare_cached(
        "INSERT INTO source.files \
         (generation_id, source_name, relative_path, content_hash, extractor, local_key, \
          byte_len, mtime_millis, unit_count) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )?
    .execute(duckdb::params![
        generation_id,
        source_name,
        &file.relative_path,
        &file.content_hash,
        &file.extractor,
        &file.local_key,
        file.byte_len as i64,
        file.mtime_millis,
        file.units.len() as i64,
    ])?;
    for unit in &file.units {
        insert_unit(conn, generation_id, source_name, file, unit)?;
    }
    if let Some(syntax) = &file.syntax {
        insert_syntax(conn, generation_id, source_name, file, syntax)?;
    }
    Ok(())
}

/// Insert one file's syntax extraction: its symbol sites and its edges.
///
/// The symbol *index* is not written here — it is a rollup over the whole
/// generation and belongs to the transaction that knows all of it
/// ([`AtlasDb::stage_scan`]).
fn insert_syntax(
    conn: &Connection,
    generation_id: &str,
    source_name: &str,
    file: &ScannedFile,
    syntax: &ScannedSyntax,
) -> Result<(), AtlasError> {
    for symbol in &syntax.symbols {
        conn.prepare_cached(
            "INSERT INTO source.occurrences \
             (generation_id, source_name, relative_path, syntax_key, extractor, language, \
              ordinal, label, name, byte_start, byte_end) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?
        .execute(duckdb::params![
            generation_id,
            source_name,
            &file.relative_path,
            &syntax.syntax_key,
            &syntax.extractor,
            syntax.language,
            symbol.ordinal as i64,
            symbol.label,
            &symbol.name,
            symbol.byte_start as i64,
            symbol.byte_end as i64,
        ])?;
    }
    for edge in &syntax.edges {
        conn.prepare_cached(
            "INSERT INTO source.edges \
             (generation_id, source_name, relative_path, syntax_key, extractor, language, \
              ordinal, edge_kind, target, byte_start, byte_end) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?
        .execute(duckdb::params![
            generation_id,
            source_name,
            &file.relative_path,
            &syntax.syntax_key,
            &syntax.extractor,
            syntax.language,
            edge.ordinal as i64,
            edge.kind,
            &edge.target,
            edge.byte_start as i64,
            edge.byte_end as i64,
        ])?;
    }
    Ok(())
}

/// Insert one structure unit.
fn insert_unit(
    conn: &Connection,
    generation_id: &str,
    source_name: &str,
    file: &ScannedFile,
    unit: &ScannedUnit,
) -> Result<(), AtlasError> {
    conn.prepare_cached(
        "INSERT INTO source.units \
         (generation_id, source_name, relative_path, local_key, ordinal, unit_kind, \
          heading_level, title, byte_start, byte_end, body) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )?
    .execute(duckdb::params![
        generation_id,
        source_name,
        &file.relative_path,
        &file.local_key,
        unit.ordinal as i64,
        unit.kind.as_str(),
        unit.heading_level.map(i64::from),
        unit.title.as_deref(),
        unit.byte_start as i64,
        unit.byte_end as i64,
        &unit.text,
    ])?;
    Ok(())
}

/// One canned dataset statement's answer: column names, then rows aligned
/// with them, every value `Option<String>` because every canned dataset query
/// casts to `VARCHAR` (see [`rows_sql`]).
type TextAnswer = (Vec<String>, Vec<Vec<Option<String>>>);

/// Run one canned dataset statement and collect its answer as text.
///
/// Both parameters are bound: the absolute path of the file to read, and the
/// row cap. Nothing a caller supplies is concatenated into `sql`, which is a
/// constant chosen by [`reader_call`].
///
/// Every column comes back `Option<String>` because every canned dataset query
/// casts to `VARCHAR` — see [`rows_sql`] for why that is a contract and not a
/// shortcut.
fn fetch_text(
    conn: &Connection,
    sql: &str,
    path: &str,
    limit: i64,
) -> Result<TextAnswer, AtlasError> {
    let mut statement = conn.prepare(sql)?;
    let mut rows = statement.query(duckdb::params![path, limit])?;
    let mut columns: Vec<String> = Vec::new();
    let mut out: Vec<Vec<Option<String>>> = Vec::new();
    while let Some(row) = rows.next()? {
        if columns.is_empty() {
            columns = row.as_ref().column_names();
        }
        let mut values = Vec::with_capacity(columns.len());
        for index in 0..columns.len() {
            values.push(row.get::<usize, Option<String>>(index)?);
        }
        out.push(values);
    }
    if columns.is_empty() {
        // A query that produced no row still has a schema, and an empty answer
        // with named columns is a different (and more useful) fact than an
        // empty answer with none.
        columns = rows
            .as_ref()
            .map(Statement::column_names)
            .unwrap_or_default();
    }
    Ok((columns, out))
}

/// Run one canned query against one dataset and turn the answer into derived
/// evidence (A1 §6.4).
///
/// **`limit` is bound into the statement *and* stored as the fact's
/// `row_limit`.** Those are the same number on purpose: a stored envelope that
/// named a bound the statement did not run under would be a false statement
/// about the answer beside it, which is exactly the thing A1 §6.4 exists to
/// prevent.
///
/// `truncated` comes from the caller because **no aggregate's own answer shape
/// can tell**. A `count(*)` returns one row whether it counted ten rows or ten
/// thousand, and a column profile returns one row per column; comparing either
/// answer's length against the row cap always says "not truncated", however
/// much of the dataset the bound cut off. Dataset-level truncation is a fact
/// about the *input*, established once by [`read_dataset`]'s probe and
/// propagated to every answer derived under the same bound.
fn dataset_fact(
    conn: &Connection,
    query: &DatasetQuery,
    sql: &str,
    dataset: &ScannedDataset,
    absolute: &str,
    limit: i64,
    truncated: bool,
) -> Result<DatasetFact, AtlasError> {
    let (columns, mut rows) = fetch_text(conn, sql, absolute, limit)?;
    // An answer whose own shape overflows the cap — a profile of more columns
    // than [`MAX_ROWS`] — is truncated too: the cap bounds what is stored as
    // well as what was scanned.
    let overflowed = rows.len() > MAX_ROWS;
    rows.truncate(MAX_ROWS);
    Ok(DatasetFact {
        relative_path: dataset.relative_path.clone(),
        dataset_key: dataset.dataset_key.clone(),
        query: query.name.to_string(),
        query_identity: query_identity(query, sql),
        row_limit: limit as u64,
        truncated: truncated || overflowed,
        output_hash: output_hash(&columns, &rows),
        columns,
        rows,
    })
}

/// **The dataset-level bound**, established by one capped scan before any
/// other question is asked of the file.
///
/// Returns the count query's column names, the row count clamped to
/// [`MAX_ROWS`], and whether the input had more rows than that — the
/// `truncated` bit every fact derived under the same bound reports.
///
/// `MAX_ROWS + 1` is bound here so the answer itself says whether the cap bit,
/// rather than leaving "exactly `MAX_ROWS`" ambiguous between "that many" and
/// "at least that many". Nothing stores this bound: it is the probe, and
/// [`counted_fact`] turns its answer into evidence under the bound that is
/// stored.
fn dataset_bound(
    conn: &Connection,
    format: DatasetFormat,
    absolute: &str,
) -> Result<(Vec<String>, u64, bool), AtlasError> {
    let sql = sql_for(&DATASET_ROW_COUNT, format);
    let (columns, rows) = fetch_text(conn, &sql, absolute, MAX_ROWS as i64 + 1)?;
    let observed = rows
        .first()
        .and_then(|row| row.first().cloned().flatten())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    Ok((
        columns,
        observed.min(MAX_ROWS as u64),
        observed > MAX_ROWS as u64,
    ))
}

/// [`DATASET_ROW_COUNT`]'s fact, built from the probe [`dataset_bound`] already
/// ran instead of from a second scan of the same file.
///
/// The clamp is not an approximation. `count(*)` over `SELECT 1 FROM <reader>
/// LIMIT k` is `min(rows, k)` by construction, so the probe's answer taken at
/// `MAX_ROWS + 1` and then clamped to `MAX_ROWS` is *exactly* what the same
/// statement returns bound at `MAX_ROWS` — the bound this fact stores. One
/// capped scan answers both "how many, up to the cap" and "was there more",
/// and the stored evidence still describes the bound it claims.
fn counted_fact(
    query: &DatasetQuery,
    sql: &str,
    dataset: &ScannedDataset,
    columns: &[String],
    row_count: u64,
    truncated: bool,
) -> DatasetFact {
    let rows = vec![vec![Some(row_count.to_string())]];
    DatasetFact {
        relative_path: dataset.relative_path.clone(),
        dataset_key: dataset.dataset_key.clone(),
        query: query.name.to_string(),
        query_identity: query_identity(query, sql),
        row_limit: MAX_ROWS as u64,
        truncated,
        output_hash: output_hash(columns, &rows),
        columns: columns.to_vec(),
        rows,
    }
}

/// The glob metacharacters DuckDB's multi-file readers interpret in a path
/// argument: `*` (and `**`), `?`, and `[` opening a character class or range.
///
/// A bound parameter stops SQL injection; it does *not* stop this. `read_csv`,
/// `read_json` and `read_parquet` expand their path argument as a glob however
/// it arrived, so a file whose *name* contains one of these would make the
/// reader open every sibling the pattern matched — including paths F10's deny
/// set and a source's `ignore` globs deliberately excluded, under a
/// `dataset_key` and `content_hash` computed by hashing only the one named
/// file. There is no per-call "disable globbing" option to reach for, so a
/// path carrying one of these is refused rather than escaped.
pub const GLOB_METACHARACTERS: [char; 3] = ['*', '?', '['];

/// The refusal a path with a glob metacharacter earns, naming the reason.
pub const DATASET_GLOB_PATH: &str = "the path contains a glob metacharacter (* ? [), which a tabular reader would expand \
     into other files";

/// **One dataset's in-place read** — reads only, no write, no transaction.
///
/// Deliberately outside [`AtlasDb::stage_scan`]'s transaction, and that is a
/// correctness rule rather than a style choice: a failing statement aborts the
/// DuckDB transaction it ran in, so a single malformed CSV read inside the
/// staging transaction would poison every insert after it and take the whole
/// scan down with it. A read failure has to be *survivable*, because it is a
/// coverage fact — "this file is a dataset and we could not read it" is
/// exactly the evidence [`Coverage::Error`] exists to carry, and a knowledge
/// directory's one bad file must not cost the estate everything else in it.
///
/// So the reads happen first, each failure captured as a value, and the
/// transaction that follows writes only rows it already holds.
fn read_dataset(conn: &Connection, scan: &SourceScan, dataset: &ScannedDataset) -> IngestedDataset {
    let refused = |status: Coverage, detail: String| IngestedDataset {
        columns: Vec::new(),
        row_count: 0,
        truncated: false,
        facts: Vec::new(),
        units: Vec::new(),
        status,
        detail,
    };
    let failed = |detail: String| refused(Coverage::Error, detail);
    let Some(root) = scan.root.as_ref() else {
        // Unreachable from the three walks in this build — only a filesystem
        // walk registers datasets — but stated rather than assumed, because
        // this is a public store and the alternative is a panic.
        return failed(crate::runtime::atlas::scan::DATASET_NO_ROOT.to_string());
    };
    let absolute = root.join(&dataset.relative_path);
    let Some(absolute) = absolute.to_str().map(str::to_owned) else {
        return failed("the dataset's path is not valid UTF-8".to_string());
    };
    // The last gate before a path reaches a reader that would glob it. The
    // walk already refuses a *relative* path carrying one of these; this
    // catches the rest of the string — a source root whose own directory name
    // carries one — and holds the rule at the boundary that actually calls
    // DuckDB, so no future caller can route around it. See
    // [`GLOB_METACHARACTERS`] for why refusing beats escaping.
    if absolute.contains(GLOB_METACHARACTERS) {
        return refused(Coverage::Unsupported, DATASET_GLOB_PATH.to_string());
    }

    // The bound first, because everything below reports it: the row count up
    // to the cap, and whether the input outran it.
    let (count_columns, row_count, truncated) = match dataset_bound(conn, dataset.format, &absolute)
    {
        Ok(bound) => bound,
        Err(e) => return failed(format!("{}: {e}", DATASET_ROW_COUNT.name)),
    };

    let mut facts = Vec::with_capacity(DATASET_QUERIES.len());
    for query in DATASET_QUERIES {
        let sql = sql_for(query, dataset.format);
        if query.name == DATASET_ROW_COUNT.name {
            facts.push(counted_fact(
                query,
                &sql,
                dataset,
                &count_columns,
                row_count,
                truncated,
            ));
            continue;
        }
        match dataset_fact(
            conn,
            query,
            &sql,
            dataset,
            &absolute,
            MAX_ROWS as i64,
            truncated,
        ) {
            Ok(fact) => facts.push(fact),
            Err(e) => return failed(format!("{}: {e}", query.name)),
        }
    }

    // The row read, and **F10a's gate expressed as the bound on it**: with no
    // declared allowlist the same statement runs with a limit of zero, so the
    // column names come back in reader order and not a single value is ever
    // fetched. The refusal is therefore a property of what runs, not only of
    // what is stored — and the schema is still learned, which is what lets a
    // dataset be registered honestly without being exposed.
    let exposes_nothing = scan.context_fields.exposes_nothing();
    let row_bound = if exposes_nothing { 0 } else { MAX_ROWS as i64 };
    let (columns, rows) = match fetch_text(conn, &rows_sql(dataset.format), &absolute, row_bound) {
        Ok(answer) => answer,
        Err(e) => return failed(format!("rows: {e}")),
    };
    // `NULL` renders as the empty string for a context unit: a unit is text,
    // and one that spelled the word "NULL" would be indistinguishable from a
    // cell that literally said so.
    let text: Vec<Vec<String>> = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(Option::unwrap_or_default)
                .collect::<Vec<String>>()
        })
        .collect();
    let units = row_units(
        &scan.source_name,
        &dataset.relative_path,
        &scan.context_fields,
        &columns,
        &text,
    );

    let detail = format!(
        "{} ({} column{}, {} row{}{}); context units: {}",
        dataset.reader,
        columns.len(),
        if columns.len() == 1 { "" } else { "s" },
        row_count,
        if row_count == 1 { "" } else { "s" },
        if truncated { ", capped" } else { "" },
        if exposes_nothing {
            "none — no context_fields declared for this source (F10a)".to_string()
        } else {
            format!(
                "{} from {}",
                units.len(),
                scan.context_fields.columns().join(",")
            )
        }
    );
    IngestedDataset {
        columns,
        row_count,
        truncated,
        facts,
        units,
        status: Coverage::Indexed,
        detail,
    }
}

/// What [`read_dataset`] found, ready to be written by the staging
/// transaction.
struct IngestedDataset {
    columns: Vec<String>,
    row_count: u64,
    truncated: bool,
    facts: Vec<DatasetFact>,
    units: Vec<RowUnit>,
    status: Coverage,
    detail: String,
}

/// Write one already-read dataset's rows, and return the coverage row it
/// earned — which replaces the placeholder the walk left for that path (see
/// [`AtlasDb::stage_scan`]), so F8's one-row-per-path rule still holds and the
/// row says what happened rather than what was attempted.
fn write_dataset(
    conn: &Connection,
    generation_id: &str,
    scan: &SourceScan,
    dataset: &ScannedDataset,
    read: &IngestedDataset,
) -> Result<CoverageRow, AtlasError> {
    conn.prepare_cached(
        "INSERT INTO source.datasets \
         (generation_id, source_name, relative_path, format, content_hash, reader, dataset_key, \
          byte_len, mtime_millis, columns, row_count, truncated, row_units) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )?
    .execute(duckdb::params![
        generation_id,
        &scan.source_name,
        &dataset.relative_path,
        dataset.format.as_str(),
        &dataset.content_hash,
        &dataset.reader,
        &dataset.dataset_key,
        dataset.byte_len as i64,
        dataset.mtime_millis,
        &join_names(&read.columns),
        read.row_count as i64,
        read.truncated,
        read.units.len() as i64,
    ])?;
    for fact in &read.facts {
        insert_dataset_fact(conn, generation_id, &scan.source_name, fact)?;
    }
    for unit in &read.units {
        insert_row_unit(conn, generation_id, &scan.source_name, dataset, unit)?;
    }
    Ok(CoverageRow {
        path: Some(dataset.relative_path.clone()),
        status: read.status,
        detail: Some(read.detail.clone()),
        bytes: Some(dataset.byte_len),
    })
}

/// Insert one derived-evidence row (A1 §6.4).
fn insert_dataset_fact(
    conn: &Connection,
    generation_id: &str,
    source_name: &str,
    fact: &DatasetFact,
) -> Result<(), AtlasError> {
    conn.prepare_cached(
        "INSERT INTO source.dataset_facts \
         (generation_id, source_name, relative_path, dataset_key, query, query_identity, \
          row_limit, truncated, columns, rows, output_hash, observed_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )?
    .execute(duckdb::params![
        generation_id,
        source_name,
        &fact.relative_path,
        &fact.dataset_key,
        &fact.query,
        &fact.query_identity,
        fact.row_limit as i64,
        fact.truncated,
        &join_names(&fact.columns),
        &render_rows(&fact.rows),
        &fact.output_hash,
        crate::domain::event::rfc3339_utc_now(),
    ])?;
    Ok(())
}

/// Insert one F10a-gated context unit.
fn insert_row_unit(
    conn: &Connection,
    generation_id: &str,
    source_name: &str,
    dataset: &ScannedDataset,
    unit: &RowUnit,
) -> Result<(), AtlasError> {
    conn.prepare_cached(
        "INSERT INTO context.row_units \
         (generation_id, source_name, relative_path, dataset_key, ordinal, row_key, key_basis, \
          fields, body) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )?
    .execute(duckdb::params![
        generation_id,
        source_name,
        &dataset.relative_path,
        &dataset.dataset_key,
        unit.ordinal as i64,
        &unit.row_key,
        unit.basis.as_str(),
        &join_names(&unit.fields),
        &unit.text,
    ])?;
    Ok(())
}

/// Column/field names as one stored value: JSON, so a name containing a comma
/// survives the round trip.
fn join_names(names: &[String]) -> String {
    serde_json::to_string(names).unwrap_or_else(|_| "[]".to_string())
}

/// [`join_names`] backwards; an unparseable value is the empty list rather
/// than a failure, because a malformed column list must not make an otherwise
/// readable evidence row unreadable.
fn split_names(stored: &str) -> Vec<String> {
    serde_json::from_str(stored).unwrap_or_default()
}

/// A query answer's rows as one stored value, in the same JSON shape the API
/// serves them in.
fn render_rows(rows: &[Vec<Option<String>>]) -> String {
    serde_json::to_string(rows).unwrap_or_else(|_| "[]".to_string())
}

/// [`render_rows`] backwards.
fn parse_rows(stored: &str) -> Vec<Vec<Option<String>>> {
    serde_json::from_str(stored).unwrap_or_default()
}

/// Insert one coverage observation.
fn insert_coverage(
    conn: &Connection,
    generation_id: &str,
    source_name: &str,
    row: &CoverageRow,
    observed_at: &str,
) -> Result<(), AtlasError> {
    conn.prepare_cached(
        "INSERT INTO meta.coverage \
         (generation_id, source_name, path, status, detail, bytes, observed_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )?
    .execute(duckdb::params![
        generation_id,
        source_name,
        row.path.as_deref(),
        row.status.as_str(),
        row.detail.as_deref(),
        row.bytes.map(|b| b as i64),
        observed_at,
    ])?;
    Ok(())
}

/// Evict one generation: drop its derived rows, mark it, and leave the
/// `generation_evicted` coverage row that says so.
///
/// The generation row itself survives, because "this world was observed, and
/// its derived evidence is gone" is a fact worth keeping — a deleted row
/// would be exactly the silent gap ruling §4 forbids.
fn evict(
    conn: &Connection,
    generation_id: &str,
    source_name: &str,
    reason: &str,
    observed_at: &str,
) -> Result<(), AtlasError> {
    conn.execute(
        "DELETE FROM source.units WHERE generation_id = ?",
        duckdb::params![generation_id],
    )?;
    conn.execute(
        "DELETE FROM source.occurrences WHERE generation_id = ?",
        duckdb::params![generation_id],
    )?;
    conn.execute(
        "DELETE FROM source.edges WHERE generation_id = ?",
        duckdb::params![generation_id],
    )?;
    conn.execute(
        "DELETE FROM source.symbols WHERE generation_id = ?",
        duckdb::params![generation_id],
    )?;
    conn.execute(
        "DELETE FROM source.files WHERE generation_id = ?",
        duckdb::params![generation_id],
    )?;
    conn.execute(
        "DELETE FROM source.datasets WHERE generation_id = ?",
        duckdb::params![generation_id],
    )?;
    conn.execute(
        "DELETE FROM source.dataset_facts WHERE generation_id = ?",
        duckdb::params![generation_id],
    )?;
    // The F10a-gated units go with everything else an eviction takes. That
    // matters more here than elsewhere: narrowing a source's `context_fields`
    // changes the reader identity, which stages a new generation, which
    // evicts this one — and *this* delete is what actually retracts the text
    // the wider allowlist exposed.
    conn.execute(
        "DELETE FROM context.row_units WHERE generation_id = ?",
        duckdb::params![generation_id],
    )?;
    // A no-op DELETE for every non-`external_git` generation (the table has
    // no row to match), and the eviction half of `git.provenance`'s own
    // atomicity promise for one that is: a superseded external source's old
    // origin/ref/commit does not linger once its rows are gone.
    conn.execute(
        "DELETE FROM git.provenance WHERE generation_id = ?",
        duckdb::params![generation_id],
    )?;
    conn.execute(
        "DELETE FROM meta.coverage WHERE generation_id = ? AND path IS NOT NULL",
        duckdb::params![generation_id],
    )?;
    conn.execute(
        "UPDATE source.generations SET state = ? WHERE generation_id = ?",
        duckdb::params![STATE_EVICTED, generation_id],
    )?;
    insert_coverage(
        conn,
        generation_id,
        source_name,
        &CoverageRow {
            path: None,
            status: Coverage::GenerationEvicted,
            detail: Some(reason.to_string()),
            bytes: None,
        },
        observed_at,
    )
}

/// What [`AtlasDb::stage_scan`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanCommit {
    /// Nothing was written: this source's newest confirmed generation was
    /// derived from exactly these bytes.
    Unchanged {
        /// The generation that still stands.
        generation_id: String,
        /// Its content identity.
        content_key: String,
    },
    /// Nothing was written **and nothing was evicted**: the walk could not
    /// read the source root, and a confirmed generation stands. Ruling §4
    /// evicts only when the source bytes changed, and an unreachable path
    /// changed none.
    RootUnavailable {
        /// The generation that still stands, untouched.
        generation_id: String,
        /// Its content identity.
        content_key: String,
        /// What was recorded as coverage against the surviving generation.
        detail: String,
    },
    /// Rows are written and awaiting their `source.scanned` summary.
    Staged {
        /// The new, provisional generation.
        generation_id: String,
    },
}

/// One unit read back out of the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredUnit {
    /// Path relative to the source root.
    pub relative_path: String,
    /// F7's reusable extraction key.
    pub local_key: String,
    /// Position within its file.
    pub ordinal: u64,
    /// Document or section.
    pub kind: UnitKind,
    /// Heading depth, for a section under a heading.
    pub heading_level: Option<u8>,
    /// Heading text, when there is one.
    pub title: Option<String>,
    /// Offset into the original file bytes.
    pub byte_start: u64,
    /// End offset into the original file bytes, exclusive.
    pub byte_end: u64,
    /// The unit's text.
    pub body: String,
}

/// One entry of the symbol index, read back out of the store.
///
/// A symbol is `(language, label, name)`. `occurrences` counts the sites in
/// the same generation that wrote it — a syntactic rollup, never a claim that
/// those sites define one thing (A1-09).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSymbol {
    /// The grammar's language name.
    pub language: String,
    /// What the grammar called it.
    pub label: String,
    /// The name as written.
    pub name: String,
    /// How many sites in this generation wrote it.
    pub occurrences: u64,
}

/// One symbol site, read back out of the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredOccurrence {
    /// Path relative to the source root.
    pub relative_path: String,
    /// F7 key of the *syntax* extraction this site came from.
    pub syntax_key: String,
    /// The grammar's versioned extractor identity.
    pub extractor: String,
    /// The grammar's language name.
    pub language: String,
    /// Position within its file's symbol list.
    pub ordinal: u64,
    /// What the grammar called it.
    pub label: String,
    /// The name as written.
    pub name: String,
    /// Offset into the original file bytes.
    pub byte_start: u64,
    /// End offset into the original file bytes, exclusive.
    pub byte_end: u64,
}

/// One edge out of a file, read back out of the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEdge {
    /// Path the edge leaves from, relative to the source root.
    pub relative_path: String,
    /// F7 key of the *syntax* extraction this edge came from.
    pub syntax_key: String,
    /// The grammar's versioned extractor identity.
    pub extractor: String,
    /// The grammar's language name.
    pub language: String,
    /// Position within its file's edge list.
    pub ordinal: u64,
    /// The edge's syntax-derived kind — `import` today.
    pub kind: String,
    /// What the file named, exactly as written. **Unresolved.**
    pub target: String,
    /// Offset into the original file bytes.
    pub byte_start: u64,
    /// End offset into the original file bytes, exclusive.
    pub byte_end: u64,
}

/// One coverage row read back out of the store, with the generation it
/// belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCoverage {
    /// Generation this observation belongs to.
    pub generation_id: String,
    /// The observation itself.
    pub row: CoverageRow,
    /// When it was recorded.
    pub observed_at: String,
}

/// This store's live extension posture (F4) — see [`AtlasDb::hardening`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hardening {
    /// Whether DuckDB may *download* an extension it does not have. Always
    /// false here.
    pub autoinstall_known_extensions: bool,
    /// Whether DuckDB may load a known extension on demand. Always false here.
    pub autoload_known_extensions: bool,
    /// Whether community extensions may load. Always false here.
    pub allow_community_extensions: bool,
    /// Whether the three above are locked against a later `SET`.
    pub locked: bool,
    /// Extensions compiled into this binary and already loaded — F4's
    /// qualification, stated by the database rather than by the build file:
    /// `json` and `parquet` are here because the feature flags put them here,
    /// which is why nothing has to be fetched to read a dataset.
    pub statically_linked: Vec<String>,
}

/// One registered tabular dataset, read back out of the store (X4).
///
/// Everything here is *about* the dataset; none of it is the dataset. Its rows
/// live in the operator's own file, which is what reading in place means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDataset {
    /// Path relative to the source root.
    pub relative_path: String,
    /// Which reader claims it.
    pub format: DatasetFormat,
    /// BLAKE3 of the file's bytes.
    pub content_hash: String,
    /// The reader identity under this source's allowlist — F7's second key
    /// input, which carries F10a's `context_fields` (see
    /// [`crate::runtime::atlas::tabular::reader_identity`]).
    pub reader: String,
    /// F7's reusable extraction key.
    pub dataset_key: String,
    /// Size in bytes.
    pub byte_len: u64,
    /// Column names, in reader order, as the in-place read found them.
    pub columns: Vec<String>,
    /// Rows counted, up to [`MAX_ROWS`].
    pub row_count: u64,
    /// Whether the row cap bit — whether the file holds more than
    /// `row_count` says (F12).
    pub truncated: bool,
    /// How many context units this dataset produced. **Zero unless the source
    /// declared `context_fields`** (F10a).
    pub row_units: u64,
}

/// One F10a-gated context unit, read back out of the store (X4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredRowUnit {
    /// Path relative to the source root.
    pub relative_path: String,
    /// F7's key for the dataset it came from.
    pub dataset_key: String,
    /// Position in the dataset, in reader order.
    pub ordinal: u64,
    /// The row's name.
    pub row_key: String,
    /// Whether that name is content-derived or had to fold in the ordinal.
    pub basis: RowKeyBasis,
    /// The allowlisted columns that produced this unit — the audit trail of
    /// what was exposed.
    pub fields: Vec<String>,
    /// The rendered text.
    pub body: String,
}

/// One source's confirmed generation and what it holds — the shape
/// `sgt intelligence status` and `sgt map stats` are rendered from (F8, F11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatus {
    /// The declared source.
    pub source_name: String,
    /// How its bytes were acquired.
    pub kind: SourceKind,
    /// What the estate may do with it.
    pub authority: AuthorityClass,
    /// The confirmed generation.
    pub generation_id: String,
    /// Its content identity.
    pub content_key: String,
    /// When it was observed.
    pub observed_at: String,
    /// The extractor identities that produced its rows (F7's second key
    /// input), sorted.
    pub extractors: Vec<String>,
    /// Acquired documents.
    pub files: u64,
    /// Structure units.
    pub units: u64,
    /// Distinct symbols in the index.
    pub symbols: u64,
    /// Symbol definition sites.
    pub occurrences: u64,
    /// Syntax-derived edges.
    pub edges: u64,
    /// Registered tabular datasets.
    pub datasets: u64,
    /// F10a-gated context units.
    pub row_units: u64,
    /// F8's coverage counts by status — including `excluded`, which is the
    /// number that makes the secrets posture checkable rather than claimed.
    pub coverage: BTreeMap<String, u64>,
    /// A1 §9's provenance, for an `external_git` generation only — `None`
    /// for every other [`SourceKind`], which never writes a `git.provenance`
    /// row at all (S4 Y5, G6).
    pub provenance: Option<SourceProvenance>,
}

/// A1 §9's provenance quintet, as read back — the query-side twin of
/// [`crate::runtime::atlas::external_git::ExternalGitProvenance`]. A separate
/// type rather than reusing that one directly: that struct is the walk
/// layer's *input* to staging, and this is what a `git.provenance` row reads
/// back as, which is a query-surface concern living beside [`SourceStatus`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProvenance {
    /// Exactly what the operator typed.
    pub origin: String,
    /// The ref that was actually fetched.
    pub requested_ref: String,
    /// The exact commit SHA the fetch resolved to.
    pub resolved_commit: String,
    /// When the fetch completed (RFC3339 UTC).
    pub retrieved_at: String,
}

/// One symbol-index hit, with the source it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSymbolHit {
    /// The declared source.
    pub source_name: String,
    /// The index entry itself.
    pub symbol: StoredSymbol,
}

/// One symbol definition site, with the source it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredReference {
    /// The declared source.
    pub source_name: String,
    /// Path relative to the source root.
    pub relative_path: String,
    /// The grammar's language name.
    pub language: String,
    /// What the grammar called it.
    pub label: String,
    /// The name as written.
    pub name: String,
    /// Position within its file, in document order.
    pub ordinal: u64,
    /// Offset into the original file bytes.
    pub byte_start: u64,
    /// End offset into the original file bytes, exclusive.
    pub byte_end: u64,
}

// ---------------------------------------------------------------------
// S5 W1 — A2 §2's admissibility filter vocabulary (H1, H13.1).
// ---------------------------------------------------------------------

/// A2 §2's stage-1(+4) and stage-2 world selection, composed once and
/// reused by every content-kind method
/// ([`AtlasDb::admissible_units`]/[`AtlasDb::admissible_occurrences`]/
/// [`AtlasDb::admissible_datasets`]) and by
/// [`AtlasDb::admissible_generations`] itself.
///
/// **Admissibility only, never a ranking hint** (A2 §8): every field here
/// either admits a row or it does not. There is no score, no weight,
/// nothing a downstream reranker (W2-W4, not built yet) could read as a
/// preference — this wave ships before any of that exists, and this type
/// is why it can (A2 §2's own ordering: filter the world, only THEN
/// retrieve/rank).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Admissibility {
    /// Stage 1: which source(s) may be seen at all — `--source`/
    /// `--source@sha`/`--work`, or none of those (every source).
    pub source: SourceSelector,
    /// Stage 4: the optional repo/knowledge/external grouping
    /// (`--type repo|knowledge|external`), the same [`SourceKind`] axis
    /// [`Self::source`] can also narrow by name. A genuinely **independent**
    /// field rather than a [`SourceSelector`] variant — A2 §2 lists stage 4
    /// as an optional selector *layered on* stage 1 (`--work --type
    /// document`, `--source repo-a --type repo`), and a caller must be able
    /// to compose the two; folding it into [`SourceSelector`] as one more
    /// mutually-exclusive variant (as an earlier revision of this type did)
    /// made that composition inexpressible by construction. `None` admits
    /// every kind — narrows only what a caller explicitly asked to narrow,
    /// same as [`Self::authority`].
    pub kind: Option<SourceKind>,
    /// Stage 2: which authority class may be seen. `None` admits every
    /// class — this filter only ever narrows what a caller explicitly
    /// asked to narrow; there is no implicit default-deny beyond what
    /// `source`/`kind` already select.
    pub authority: Option<AuthorityClass>,
}

/// A2 §2's stage-1 source/estate/Work-generation selector. Stage 4's
/// repo/knowledge/external grouping is [`Admissibility::kind`], a separate
/// field composable with any variant here — see that field's own doc for
/// why it does not live as a variant of this enum.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SourceSelector {
    /// No source-name constraint: every confirmed generation, subject
    /// only to [`Admissibility::kind`]/[`Admissibility::authority`].
    #[default]
    Any,
    /// A2 §2's `--source <name>`: exactly one declared source's newest
    /// CONFIRMED generation.
    Named(String),
    /// A2 §2's `--source <name>@<sha>`: one exact generation. Because a
    /// superseded generation's content rows are deleted at eviction time
    /// (ruling §4 — [`AtlasDb::confirm_scan`]'s own promotion, and
    /// [`AtlasDb::evict_work_overlays`]), this can only ever answer for a
    /// `content_key` that is *still* the source's current confirmed
    /// generation; a genuinely stale key correctly returns nothing rather
    /// than approximating — "never approximate" (A2 §2) applies to an
    /// admissibility miss exactly as it applies to a hit.
    Exact {
        /// The declared source.
        source_name: String,
        /// The exact generation's content identity.
        content_key: String,
    },
    /// A2 §2's `--work <id>` filter: the named repository's BASE
    /// generation **and** this Work's own overlay generations
    /// (`work:<id>/<repo>`,
    /// [`overlay_source_name`](crate::runtime::atlas::overlay::overlay_source_name))
    /// — S5 W1b, which wired the daemon-side
    /// lifecycle hook H13.2 chose and so made the overlay half real.
    /// `repository` is the plain, non-overlay source name this Work's
    /// surface is bound to; Atlas holds no Work↔repository binding of its
    /// own (that lives in [`crate::runtime::surface::WorkSurface`]) so the
    /// caller resolves it and hands it in.
    ///
    /// **Exactly one Work's overlay, over exactly this variant's own
    /// `repository`, never another Work's and never a sibling repository
    /// this same Work also binds.** The admitted overlay coordinate is the
    /// exact source name derived from *this variant's own* `work_id` **and**
    /// `repository` (`Self::overlay_admit_source_name`) — never a `work:
    /// <id>/%` prefix, which would admit every repository under that Work
    /// id and over-claim past what `repository` names. So a second Work
    /// bound to the same repository stays outside the filter by
    /// construction — the leak W1's review panel found reachable through
    /// `--source`, and the one `tests/w1_deterministic_filter.rs::
    /// a_work_filter_excludes_a_different_works_generation` pins — and this
    /// Work's own overlay over a *different* repository stays outside it
    /// too.
    ///
    /// `AtlasDb::work_scope` is what a caller MUST render/assert
    /// alongside any answer built from this variant: the overlay half is a
    /// SNAPSHOT taken at a lifecycle moment, not a live read of the
    /// surface, and an answer that did not say so would imply "current"
    /// when it means "as of the last surface bind" (W1b item 3).
    WorkBase {
        /// The Work this admission is scoped to, carried for the caller's
        /// own attribution — not read by the query itself.
        work_id: String,
        /// The base repository source name.
        repository: String,
    },
}

impl SourceSelector {
    /// The `(source_name, content_key)` bind values every admissibility
    /// query composes identically — [`Self::Named`] and [`Self::WorkBase`]
    /// bind the same shape on purpose: **W1's whole point is that a Work's
    /// base generation reads exactly like any other named source's**, and
    /// only [`Self::work_scope`] marks the difference the caller must
    /// state. Stage 4's `source_kind` bind is
    /// [`Admissibility::kind`]'s alone now, not this selector's — see that
    /// field's own doc.
    fn bindings(&self) -> (Option<&str>, Option<&str>) {
        match self {
            Self::Any => (None, None),
            Self::Named(name) => (Some(name.as_str()), None),
            Self::Exact {
                source_name,
                content_key,
            } => (Some(source_name.as_str()), Some(content_key.as_str())),
            Self::WorkBase { repository, .. } => (Some(repository.as_str()), None),
        }
    }

    /// The exact overlay source name this selector *additionally* admits —
    /// `Some("work:<id>/<repository>")` for [`Self::WorkBase`], `None` for
    /// every other variant (S5 W1b).
    ///
    /// **Exact, not a prefix.** An earlier revision built a `LIKE
    /// "work:<id>/%"` pattern here, which admitted every repository under
    /// that Work id, not just [`Self::WorkBase`]'s own `repository` field —
    /// a `WorkBase { work_id, repository: "repo-a" }` filter's overlay
    /// branch admitted `work:<id>/repo-a` *and* `work:<id>/repo-b`
    /// identically, over-claiming exactly the sibling repository the base
    /// half (`Self::bindings`) is restricted to. It also bound `work_id`
    /// unescaped into a `LIKE` pattern, so a `work_id` containing `%`/`_`
    /// would have widened admission past even that Work — `%` alone
    /// produces `work:%/%`, matching any Work's overlay. Composing the
    /// exact name via
    /// [`overlay_source_name`](crate::runtime::atlas::overlay::overlay_source_name)
    /// and comparing with `=` (see [`Self::admissible_generations`]'s SQL)
    /// removes both hazards at once: there is exactly one overlay source
    /// name a `WorkBase` selector can ever mean, so there is nothing left
    /// for a wildcard to widen. A caller cannot widen it further: there is
    /// no code path that puts a client-supplied string into this value
    /// (F12).
    fn overlay_admit_source_name(&self) -> Option<String> {
        match self {
            Self::WorkBase {
                work_id,
                repository,
            } => Some(crate::runtime::atlas::overlay::overlay_source_name(
                work_id, repository,
            )),
            _ => None,
        }
    }
}

/// What an [`Admissibility`] answer built from
/// [`SourceSelector::WorkBase`] actually covers of A2 §2's `--work`
/// promise — *"current Work's world, **including overlay**"* — **and, when
/// the overlay half is there, exactly what instant of that world it is.**
///
/// # The freshness semantic is in this type, not in a comment (W1b item 3)
///
/// A Work's surface is mutated continuously while the Work runs. The
/// overlay is produced by a daemon-side lifecycle hook (H13.2's chosen
/// mechanism, wired in S5 W1b: [`crate::api`]'s surface-lifecycle arm),
/// which fires when the surface is **bound** — materialized, or
/// re-materialized for a retry — and is evicted when the surface is torn
/// down. It is therefore a **snapshot**, never a live read: nothing
/// rescans between two lifecycle events, and `sgt search` stays a pure
/// reader that never touches the surface at all (H13.2 rejected
/// query-time scanning precisely so it could not).
///
/// So [`Self::BaseAndOverlaySnapshot`] carries the snapshot's own
/// `observed_at`. A caller renders it. An answer that said "including
/// overlay" without saying *as of when* would imply "current" while
/// meaning "as of the last surface bind" — the same class of false claim
/// as the silent partial W1 refused to ship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkScope {
    /// [`SourceSelector::WorkBase`] was not the selector in play — the
    /// concept does not apply to this answer.
    NotWorkScoped,
    /// A2 §2's promise in full: the repository's base generation **and**
    /// this Work's overlay generation over that *same* repository —
    /// [`SourceSelector::WorkBase`] names exactly one, so there is exactly
    /// one overlay generation this can ever mean.
    BaseAndOverlaySnapshot {
        /// When the overlay half was actually read off the surface — that
        /// one overlay generation's own `observed_at`. **Not** "now", and
        /// not the query's own time.
        overlay_observed_at: String,
    },
    /// `--work` admitted only the repository's plain base generation,
    /// because **no overlay generation stands for this Work**: it has not
    /// been bound yet, this installation records no Atlas evidence at all,
    /// the overlay scan failed (its coverage row says so where there was a
    /// generation to attach one to — [`AtlasDb::record_overlay_unavailable`]),
    /// or the Work retired and [`AtlasDb::evict_work_overlays`] took its
    /// overlay with it.
    ///
    /// A caller MUST state this rather than presenting the answer as A2
    /// §2's full "including overlay" promise. The store cannot tell those
    /// causes apart and this variant does not pretend to: it says what is
    /// true — there is no overlay behind this answer.
    BaseOnly,
}

/// What one `admissible_*` call answered, **with its own completeness
/// marker attached to the value** — every `admissible_*` method returns
/// this rather than a bare `Vec`, so [`WorkScope`]'s "a caller MUST
/// render/assert" (see [`SourceSelector::WorkBase`]'s own doc) is something
/// the type forces rather than a fact only recoverable by re-deriving it
/// from the original `filter` a caller may no longer have in scope. A
/// caller that forwards `hits` onward without `scope` has to do so
/// explicitly (`.hits`, dropping `.scope` on the floor) rather than by
/// construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admitted<T> {
    /// The admissible rows themselves.
    pub hits: Vec<T>,
    /// What this answer covers — see [`WorkScope`].
    pub scope: WorkScope,
}

/// H13.1's content-kind filter, document family: the exhaustive, code-owned
/// list of extractor identities [`AtlasDb::admissible_units`] matches
/// against — never a client-supplied pattern (F12). Every identity a
/// document-shaped adapter in this build can write to `source.files`:
/// Markdown, plain text, Office (`.docx`), and mail (`.eml`). **Not**
/// [`crate::runtime::atlas::archive::ZIP_EXTRACTOR`]: a ZIP archive is a
/// container — its own top-level resource carries no prose, only its
/// unpacked children do, each under its own (already-listed) extractor
/// identity.
///
/// Pinned by `tests/w1_deterministic_filter.rs`'s structural test, in the
/// shape of `tests/x1_atlas_substrate.rs`'s one-owner test: it walks
/// `text.rs`/`office.rs`/`mail.rs`'s own `pub const ..._EXTRACTOR`
/// declarations and asserts this list is exactly that set, so a new
/// document adapter that lands a new identity without updating this list
/// fails that test rather than silently falling out of (or into)
/// `--content document`.
pub const DOCUMENT_EXTRACTOR_IDENTITIES: [&str; 4] = [
    crate::runtime::atlas::text::MARKDOWN_EXTRACTOR,
    crate::runtime::atlas::text::TEXT_EXTRACTOR,
    crate::runtime::atlas::office::DOCX_EXTRACTOR,
    crate::runtime::atlas::mail::MAIL_EXTRACTOR,
];

/// H13.1's content-kind filter, code family: the fixed, code-owned `LIKE`
/// pattern [`AtlasDb::admissible_occurrences`] matches `extractor` against
/// — never a client-supplied pattern (F12). Every `source.occurrences`
/// (and `source.symbols`/`source.edges`) row is written from
/// [`crate::runtime::atlas::syntax::SyntaxLanguage::extractor_identity`],
/// whose own format (`"syntax-{name}/{SYNTAX_EXTRACTOR_VERSION}"`) makes
/// this prefix exhaustive by construction — pinned by a structural test
/// that asserts every
/// [`crate::runtime::atlas::syntax::SyntaxLanguage::ALL`] identity matches
/// it, and that a synthetic unknown identity does not.
pub const CODE_EXTRACTOR_LIKE: &str = "syntax-%";

/// One structure unit read back out of the store, with the source it
/// belongs to — [`AtlasDb::admissible_units`]'s cross-source answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredUnitHit {
    /// The declared source.
    pub source_name: String,
    /// The unit itself.
    pub unit: StoredUnit,
}

/// One occurrence read back out of the store, with the source it belongs
/// to — [`AtlasDb::admissible_occurrences`]'s cross-source answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredOccurrenceHit {
    /// The declared source.
    pub source_name: String,
    /// The occurrence itself.
    pub occurrence: StoredOccurrence,
}

/// One registered dataset read back out of the store, with the source it
/// belongs to — [`AtlasDb::admissible_datasets`]'s cross-source answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDatasetHit {
    /// The declared source.
    pub source_name: String,
    /// The dataset itself.
    pub dataset: StoredDataset,
}

// ===================================================================
// The operations projection (`ops.*`) — A1 §5, S5 W1c
// ===================================================================
//
// Everything below this line was `src/runtime/analytics.rs`, which owned a
// second physical database (`<data-dir>/projections/sergeant.duckdb`). A1 §5
// declares ONE physical file, `atlas.duckdb`, carrying five logical schemas —
// `meta`, `ops`, `source`, `git`, `context` — and decision A1-02's rationale
// is literally "schemas provide separation without more databases". The
// second file was a wave-ratified deviation that no owner ruling ever
// ratified; the owner correction of 2026-08-29 settled that the code
// converges to the contract, not the other way round.
//
// It lives in *this* file rather than a sibling because the one-owner
// invariant is the reason the split was tolerable at all: one database, one
// owning module. Two files each holding a `Connection` to `atlas.duckdb`
// would be a union rule ("either of these may open the database"), which
// passes just as happily once one owner has grown into the other's
// territory. `tests/x1_atlas_substrate.rs`'s
// `atlas_database_has_exactly_one_owner` now scans the whole of `src/`.
//
// What did NOT merge is the rebuild discipline. `ops` is still a pure fold
// of the journal and is still dropped and refolded on every daemon start —
// but by `DROP SCHEMA ops CASCADE`, never by deleting the file, because
// `meta/source/git/context` in the same file must survive (F1). See
// [`Analytics::begin_rebuild`].

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
    /// A raw table dump was asked for by a name that is not a §22 table.
    #[error("no such analytics table {name:?}")]
    UnknownTable {
        /// The name that was asked for.
        name: String,
    },
    /// A write failed part-way through a fold, so the tables no longer match
    /// the fold state. The projection refuses to answer until a `catch_up`
    /// from the journal has rebuilt it — see [`Analytics::catch_up`].
    #[error("the analytical projection dropped rows on a failed write and must be rebuilt")]
    NeedsRebuild,
}

/// Prepared statements the fold keeps planned. Comfortably above the number
/// of distinct statements it issues, so no statement is ever evicted and
/// The schema every operations table lives in (S3 F3, A1 §5).
///
/// Physical qualification only — nothing this module reports changes name
/// because of it. See the module doc.
const OPS_SCHEMA: &str = "ops";

/// One operations table, qualified and quoted for SQL.
///
/// The quoting is what keeps `usage` (a reserved word) addressable; the
/// qualification is what stops a bare name from resolving against DuckDB's
/// default `main` schema, which this database deliberately leaves empty.
fn ops(table: &str) -> String {
    format!("{OPS_SCHEMA}.\"{table}\"")
}

/// The §22 schema, in the [`OPS_SCHEMA`] namespace. Written on every rebuild;
/// there is no migration path and there does not need to be one — these
/// tables are derived.
///
/// The leading `DROP SCHEMA ... CASCADE` is what replaced deleting the file
/// (S5 W1c). `CASCADE` takes the tables in the namespace with it and nothing
/// outside it, which is exactly the scope the old `std::fs::remove_file` had
/// back when `ops` was the only thing in its own database. `IF EXISTS`
/// because a first-ever start has no `ops` to drop, and the namespace-level
/// `CREATE SCHEMA IF NOT EXISTS ops` in [`SCHEMA_DDL`] may or may not have
/// run against this file first.
const OPS_DDL: &str = r#"
DROP SCHEMA IF EXISTS ops CASCADE;
CREATE SCHEMA ops;
CREATE TABLE ops.events (
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
CREATE INDEX idx_events_kind_work ON ops.events(kind, work_id);
CREATE TABLE ops.work (
    work_id       VARCHAR PRIMARY KEY,
    intent        VARCHAR,
    estate        VARCHAR,
    estate_root   VARCHAR,
    parent_work_id      VARCHAR,
    parent_execution_id VARCHAR,
    causation_unverified VARCHAR,
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
CREATE TABLE ops.stages (
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
CREATE TABLE ops.executions (
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
CREATE TABLE ops.messages (
    seq          BIGINT PRIMARY KEY,
    work_id      VARCHAR,
    execution_id VARCHAR,
    role         VARCHAR,
    text         VARCHAR,
    ts_ms        BIGINT
);
CREATE TABLE ops.tool_calls (
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
CREATE TABLE ops."usage" (
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
CREATE TABLE ops.repositories (
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
CREATE TABLE ops.graph_nodes (
    node_id    VARCHAR PRIMARY KEY,
    kind       VARCHAR NOT NULL,
    label      VARCHAR,
    work_id    VARCHAR,
    source_seq BIGINT NOT NULL
);
CREATE TABLE ops.graph_edges (
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
                FROM ops.events \
                WHERE work_id IS NOT NULL AND kind LIKE 'work.%' \
            ) \
            SELECT w.work_id, w.state, \
                   COALESCE(SUM(CASE WHEN spans.kind = 'work.blocked' AND spans.next_ms IS NOT NULL \
                                     THEN spans.next_ms - spans.ts_ms END), 0) AS blocked_ms, \
                   COUNT(CASE WHEN spans.kind = 'work.blocked' THEN 1 END) AS blocked_episodes \
            FROM ops.work w LEFT JOIN spans ON spans.work_id = w.work_id \
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
            FROM ops.stages s JOIN ops.work w ON w.work_id = s.work_id \
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
            FROM ops.executions e \
            LEFT JOIN (SELECT work_id, COUNT(*) AS repositories FROM ops.repositories GROUP BY 1) r \
                   ON r.work_id = e.work_id \
            LEFT JOIN (SELECT execution_id, COUNT(*) AS tool_calls FROM ops.tool_calls GROUP BY 1) t \
                   ON t.execution_id = e.execution_id \
            LEFT JOIN (SELECT execution_id, COUNT(*) AS messages FROM ops.messages GROUP BY 1) m \
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
                FROM ops.events \
                WHERE work_id IS NOT NULL \
                  AND kind IN ('work.failed', 'work.blocked', 'stage.failed', 'stage.blocked') \
                GROUP BY work_id \
            ) \
            SELECT f.work_id, f.failed_seq, \
                   (SELECT kind FROM ops.events WHERE seq = f.failed_seq) AS failure_kind, \
                   COUNT(t.tool_use_id) AS tool_calls_before \
            FROM failure f \
            LEFT JOIN ops.tool_calls t \
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
            FROM ops.\"usage\" GROUP BY work_id ORDER BY work_id",
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
    /// Set while a fold is in flight and left set if it fails, because a
    /// failed fold can have advanced [`Analytics::last_seq`] past rows that
    /// never reached the tables. While set, every read fails closed and the
    /// next [`Analytics::catch_up`] rebuilds from seq 0. See that method.
    needs_reset: bool,
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
    /// estate-root Phase C, §7.4: seeded from `work.submitted`'s
    /// (pre-Phase-C-only, now always absent for a new Work) `estate`
    /// field, then overwritten by `workflow.bound`'s plan-time estate name
    /// once that fires — see the two folds below. `None` for a `pending`
    /// Work that never reached `workflow.bound` is an honest "not yet
    /// resolved to an estate", not a lost fact.
    estate: Option<String>,
    /// H1 touch point #6: the canonical estate root, folded once from the
    /// envelope's `workspace_id` at `work.submitted` — unlike `estate`
    /// above, never overwritten at `workflow.bound`, since the coordinate
    /// is already known (and immutable for a Work's life) at submission.
    /// `None` for a pre-Phase-C-shaped legacy line, whose envelope never
    /// carried the field at all — an honest "not recorded", not an error.
    estate_root: Option<String>,
    /// W1-09 (S2 E8 as amended): the **validated** parent Work/execution,
    /// folded from `work.submitted`'s own payload — W1's explicit
    /// instruction to extend the existing envelope for durable query rather
    /// than build a second agent-tree store. `None` for an ordinary
    /// top-level submission and for every Work journaled before S2.
    parent_work_id: Option<String>,
    parent_execution_id: Option<String>,
    /// The `causation_unverified` marker's `reason`, when a claim failed
    /// validation — queryable beside the relation it is the absence of, so
    /// "which submissions claimed a parent that did not check out" is one
    /// question of one table rather than a journal scan.
    causation_unverified: Option<String>,
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
            text(self.estate.as_deref()),
            text(self.estate_root.as_deref()),
            text(self.parent_work_id.as_deref()),
            text(self.parent_execution_id.as_deref()),
            text(self.causation_unverified.as_deref()),
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
    /// The `ops` namespace is dropped and recreated first, so this is still
    /// the daemon's startup path and the whole population story: there is no
    /// "open the existing `ops` tables and continue", and nothing can quietly
    /// accumulate state that the journal cannot reproduce. What it no longer
    /// does is delete a file — see [`Analytics::begin_rebuild`].
    pub fn rebuild<I>(data_dir: &Path, events: I) -> Result<Self, AnalyticsError>
    where
        I: IntoIterator<Item = Result<Event, JournalError>>,
    {
        let mut analytics = Self::begin_rebuild(data_dir)?;
        analytics.catch_up(events)?;
        Ok(analytics)
    }

    /// [`Analytics::rebuild`] minus the fold: open `atlas.duckdb`, drop the
    /// `ops` namespace and recreate it empty. W2's `start_with` uses this
    /// then [`Analytics::fold`] so the shared startup pass can feed this
    /// projection one event at a time, the same call that feeds every other
    /// sink.
    ///
    /// **This no longer deletes a file, and it must not** (S5 W1c). `ops`
    /// shares `atlas.duckdb` with `meta`, `source`, `git` and `context`, and
    /// those persist across restarts: they are derived from source bytes plus
    /// extractor identity and no journal replay reproduces them (F1). Deleting
    /// the file to rebuild `ops` would silently discard every persisted
    /// source generation. `DROP SCHEMA ops CASCADE` has the scope the old
    /// `remove_file` had — every `ops` table and nothing else.
    ///
    /// It applies [`bootstrap_atlas_ddl`] on the way past — the same helper
    /// [`AtlasDb::over`] uses — so a host whose daemon starts before any
    /// source has ever been scanned still gets a file that declares all five
    /// of A1 §5's namespaces rather than only `ops`.
    pub fn begin_rebuild(data_dir: &Path) -> Result<Self, AnalyticsError> {
        create_dir_all_durable(&atlas_dir(data_dir))?;
        let path = atlas_db_path(data_dir);
        let conn = Connection::open(&path)?;
        bootstrap_atlas_ddl(&conn)?;
        Self::over(conn, path)
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
        conn.execute_batch(OPS_DDL)?;
        Ok(Self {
            conn,
            path,
            rows: Rows::default(),
            graph: GraphContext::default(),
            last_seq: 0,
            materialized_seq: 0,
            needs_reset: false,
        })
    }

    /// Seq of the last event folded in — `0` when the projection is holding
    /// a failed fold and needs rebuilding, because then it has folded
    /// nothing a caller may rely on.
    ///
    /// The daemon reads this to decide which journal tail to hand back to
    /// [`Analytics::catch_up`], so reporting `0` here is what turns a failed
    /// fold into a full re-fold rather than a permanent hole.
    pub fn last_seq(&self) -> u64 {
        if self.needs_reset { 0 } else { self.last_seq }
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
        let mut fold = self.fold()?;
        for event in events {
            fold.push(&event?)?;
        }
        fold.finish()
    }

    /// Begin a fold session. Resets if a previous fold failed, then arms
    /// `needs_reset` — exactly [`Analytics::catch_up`]'s prologue, split out
    /// so W2's shared startup pass can drive this fold one event at a time
    /// alongside every other sink, instead of handing it a whole iterator of
    /// its own.
    ///
    /// A fold is not atomic: rows are buffered as events are applied and
    /// written in chunks, and `last_seq` advances per event so the skip in
    /// [`AnalyticsFold::push`] stays right. If a write then fails, those
    /// buffered rows are gone while `last_seq` says they landed — and because
    /// the skip is permanent, `events`/`messages`/`usage` would answer 200
    /// with rows missing forever after (the mutable tables self-heal via
    /// `materialize`, the append-only ones cannot). That is precisely the
    /// "kept current == rebuilt from scratch" invariant this method exists to
    /// hold, so the failure is made structural instead: the flag is set
    /// before the attempt and cleared only on [`AnalyticsFold::finish`]'s
    /// success, so no `?` anywhere in between can escape it, and the next
    /// fold re-folds from zero. Cost of a transient write failure is one 503
    /// and one rebuild; it is never a silently short table.
    pub fn fold(&mut self) -> Result<AnalyticsFold<'_>, AnalyticsError> {
        if self.needs_reset {
            self.reset()?;
        }
        self.needs_reset = true;
        Ok(AnalyticsFold {
            analytics: self,
            appended: Appended::default(),
            applied: 0,
        })
    }

    /// Empty every table and the in-memory fold, so the next catch-up is a
    /// rebuild from seq 0.
    ///
    /// The in-memory state is reset only once the SQL has succeeded, so a
    /// reset that itself fails (the disk is still full) leaves the instance
    /// marked and is simply retried by the next call.
    fn reset(&mut self) -> Result<(), AnalyticsError> {
        for table in TABLES {
            self.conn
                .execute_batch(&format!("DELETE FROM {}", ops(table)))?;
        }
        self.rows = Rows::default();
        self.graph = GraphContext::default();
        self.last_seq = 0;
        self.materialized_seq = 0;
        self.needs_reset = false;
        Ok(())
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
        // Every read goes through here, which makes it the one place that
        // has to refuse to answer out of a projection whose tables no longer
        // match its fold state (see `catch_up`). Answering anyway would be
        // the one thing the projection must never do: quietly wrong rows.
        if self.needs_reset {
            return Err(AnalyticsError::NeedsRebuild);
        }
        if self.materialized_seq == self.last_seq {
            return Ok(());
        }
        for table in MUTABLE_TABLES {
            self.conn
                .execute_batch(&format!("DELETE FROM {}", ops(table)))?;
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
                            estate: string(&work["workspace"]),
                            // H1 touch point #6: the envelope's own field,
                            // not a payload key — real for every new Work
                            // from `work.submitted` onward, `None` for a
                            // legacy line whose envelope never carried it.
                            estate_root: event.workspace_id.clone(),
                            // W1-09: the validated relation and the
                            // unverified marker both come out of the one
                            // compound `work.submitted` payload (L6), so
                            // this fold cannot see one without the other.
                            parent_work_id: string(&work["parent_work_id"]),
                            parent_execution_id: string(&work["parent_execution_id"]),
                            causation_unverified: string(&work["causation_unverified"]["reason"]),
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
                    if let Some(estate) = string(&event.payload["workspace"]) {
                        work.estate = Some(estate);
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
                .prepare(&format!("SELECT COUNT(*) FROM {}", ops(table)))?;
            let count: i64 = statement.query_row(duckdb::params![], |row| row.get(0))?;
            counts.push(((*table).to_string(), count));
        }
        Ok(counts)
    }

    /// Every row of one §22 table.
    ///
    /// **No production caller, and deliberately so.** No route reaches this
    /// and no CLI verb exposes it: the daemon's analytics surface is the
    /// canned §22 questions plus the graph neighborhood, and a raw table
    /// dump is not in M5's contract. It is public because it is the
    /// instrument acceptance 1 needs — "delete `atlas.duckdb`, rebuild,
    /// identical `ops` results **row for row**" is a claim `table_counts`
    /// cannot check, and
    /// the M5 suite is a separate crate that cannot see `pub(crate)`. If a
    /// future milestone wants a table dump on the API, this is the function
    /// to route to; until then it answers only to the tests that justify it.
    ///
    /// The name is checked against the table list rather than interpolated — the
    /// projection answers questions it knows, never SQL it was handed, which
    /// is the same rule that keeps [`CANNED_QUERIES`] a fixed list.
    pub fn table_rows(&mut self, table: &str) -> Result<QueryResult, AnalyticsError> {
        let table = TABLES
            .iter()
            .find(|known| **known == table)
            .ok_or_else(|| AnalyticsError::UnknownTable {
                name: table.to_string(),
            })?;
        self.materialize()?;
        let sql = format!("SELECT * FROM {}", ops(table));
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
            "SELECT node_id, kind, label, work_id, source_seq FROM ops.graph_nodes \
             WHERE work_id = ?1 \
                OR node_id IN (SELECT from_node FROM ops.graph_edges WHERE work_id = ?1) \
                OR node_id IN (SELECT to_node FROM ops.graph_edges WHERE work_id = ?1) \
             ORDER BY source_seq, node_id",
            duckdb::params![work_id],
        )?;
        let (edge_columns, edge_rows) = self.select(
            "SELECT edge_id, relation, from_node, to_node, source_seq FROM ops.graph_edges \
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

/// A fold session opened by [`Analytics::fold`] — one iteration of what used
/// to be `catch_up_folding`'s loop body, exposed so W2's shared startup pass
/// can feed this projection one event at a time from the same drive that
/// feeds every other sink.
///
/// The rebuild path and the incremental path are still this one method
/// (`push`), so "rebuilt from scratch" and "kept current" cannot drift apart
/// — [`Analytics::catch_up`] is now just a loop over `push` plus `finish`.
pub struct AnalyticsFold<'a> {
    analytics: &'a mut Analytics,
    appended: Appended,
    applied: u64,
}

impl AnalyticsFold<'_> {
    /// Fold one event, skipping it if its seq is at or below what this
    /// projection already has (so handing this every event a wider replay
    /// yields, unfiltered, is always safe).
    pub fn push(&mut self, event: &Event) -> Result<(), AnalyticsError> {
        if event.seq <= self.analytics.last_seq {
            return Ok(());
        }
        self.analytics.apply(event, &mut self.appended);
        self.analytics.last_seq = event.seq;
        self.applied += 1;
        if self.appended.len() >= APPEND_CHUNK {
            self.analytics.flush(&mut self.appended)?;
        }
        Ok(())
    }

    /// Today's `catch_up_folding` epilogue plus clearing `needs_reset` — the
    /// only place that happens, so a fold whose `push` ever returned an error
    /// (and was therefore abandoned rather than driven to `finish`) leaves
    /// the flag armed for the next fold to see.
    pub fn finish(self) -> Result<u64, AnalyticsError> {
        let AnalyticsFold {
            analytics,
            mut appended,
            applied,
        } = self;
        analytics.flush(&mut appended)?;
        analytics.needs_reset = false;
        Ok(applied)
    }
}

/// Tables this projection creates, in a stable order. Crate-internal: the
/// table list is an implementation detail of the projection, and callers get
/// it as data from [`Analytics::table_counts`].
const TABLES: &[&str] = &[
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
    let mut appender = conn.appender_to_db(table, OPS_SCHEMA)?;
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

// ----------------------------------------------------- the cross-schema join
//
// The capability A1 §5 gives as its whole reason for one database: "DuckDB
// supports named schemas within one database, enabling cross-domain joins
// without attaching/federating a fleet of databases." [EXT-DUCKDB-SCHEMA]
//
// Proven, not assumed to follow from colocation — the owner ruling of
// 2026-08-29, decision 4.

/// One row of [`AtlasDb::work_overlay_generations`]: a Work's identity from
/// `ops`, joined to a source generation from `source`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkGeneration {
    /// `ops.work.work_id` — the Work identity half of the join.
    pub work_id: String,
    /// `ops.work.state` as the journal fold last wrote it.
    pub work_state: String,
    /// `source.generations.source_name` — the `work:<id>/<repo>` overlay
    /// coordinate that matched.
    pub source_name: String,
    /// That generation's content identity.
    pub content_key: String,
    /// When the overlay was read off the surface.
    pub observed_at: String,
}

/// The join, as one SQL statement over two schemas of one database.
///
/// Public so a test can assert what it is as well as what it returns: no
/// `ATTACH`, no second database name, one `FROM`/`JOIN` pair across `ops` and
/// `source`. A statement that had to federate two stores could not be written
/// this way at all — which is the property A1 §5 bought, and the one two
/// files could not deliver, because each store's one-owner invariant forbids
/// the other's token.
pub const WORK_GENERATION_JOIN_SQL: &str = "\
SELECT w.work_id, w.state, g.source_name, g.content_key, g.observed_at \
  FROM ops.work AS w \
  JOIN source.generations AS g \
    ON g.source_name = 'work:' || w.work_id || '/' || ? \
 WHERE g.state = 'confirmed' \
 ORDER BY w.work_id, g.source_name";

impl Analytics {
    /// A second handle on **this same open database**, for the Atlas half of
    /// it.
    ///
    /// **This is how a process gets an [`AtlasDb`] while the projection is
    /// open, and `AtlasDb::open` is not.** One file means one DuckDB
    /// *database instance*: two `Connection::open` calls against the same
    /// path produce two independent instances that do not see each other's
    /// writes and whose last close silently wins. That was harmless while
    /// `ops` had a file to itself; with A1 §5's one database it would mean a
    /// scan's `source.*` rows and the daemon's `ops.*` fold overwriting one
    /// another. `Connection::try_clone` shares the instance instead, which is
    /// what every other handle in the daemon is derived from.
    ///
    /// This hands out an `AtlasDb`, never a [`Connection`]: the one-owner
    /// invariant is about which *module* may reach the driver, and both
    /// halves of that module's public surface are still plain data.
    pub fn atlas(&self) -> Result<AtlasDb, AtlasError> {
        let conn = self.conn.try_clone()?;
        conn.set_prepared_statement_cache_capacity(STATEMENT_CACHE);
        // No [`HARDENING_DDL`] here, and its absence is stronger than its
        // presence would be. Those four settings are database-wide and
        // `lock_configuration = true` makes them permanent, so re-issuing
        // them on a second connection to the same instance is refused by
        // DuckDB itself ("the configuration has been locked"). The
        // projection's own open ran them before its first query, and F4's
        // posture is verified rather than assumed: a clone that somehow
        // reached an unhardened instance is refused here, not left to reach
        // the network later.
        conn.execute_batch(SCHEMA_DDL)?;
        conn.execute_batch(TABLE_DDL)?;
        let db = AtlasDb {
            conn,
            path: self.path.clone(),
        };
        let posture = db.hardening()?;
        if !posture.locked
            || posture.autoinstall_known_extensions
            || posture.autoload_known_extensions
            || posture.allow_community_extensions
        {
            return Err(AtlasError::UnknownValue {
                column: "hardening".to_string(),
                value: format!("{posture:?}"),
            });
        }
        Ok(db)
    }
}

impl Analytics {
    /// Every confirmed Work-overlay generation over `repository`, keyed back
    /// to the Work identity `ops` holds for it — in one statement.
    ///
    /// **No production caller, and deliberately so**, exactly as
    /// [`Analytics::table_rows`] has none. A2's retrieval path is not this
    /// wave's scope; what is this wave's scope is proving that the capability
    /// A1 §5 cites as the reason for one database actually exists now that
    /// there is one. [`SourceSelector::WorkBase`] is how the filter itself
    /// reaches an overlay, and it takes the repository binding from its
    /// caller precisely because Atlas holds no Work↔repository binding of its
    /// own; this method is the demonstration that the *other* direction — ops
    /// identity to source evidence — is now an ordinary join rather than a
    /// federation.
    ///
    /// It lives on the projection rather than on [`AtlasDb`] because the ops
    /// half of the join has a freshness rule the source half does not: see
    /// [`Analytics::materialize`]'s call below. Either handle can *see* both
    /// schemas — that is the whole point of one database — but only this one
    /// can promise the `ops` rows are current.
    ///
    /// The repository is bound as a parameter, never interpolated: the
    /// coordinate is built from a Work id that came out of the database and a
    /// repository name that came from the caller, and only one of those is
    /// already trusted.
    pub fn work_overlay_generations(
        &mut self,
        repository: &str,
    ) -> Result<Vec<WorkGeneration>, AnalyticsError> {
        // The `ops` half is a *mutable* table: the fold accumulates rows in
        // memory and writes them at read time, so a join issued without this
        // would read whatever the last materialization left behind — from
        // this connection or any other. Every other read on this type goes
        // through the same call for the same reason.
        self.materialize()?;
        let mut statement = self.conn.prepare(WORK_GENERATION_JOIN_SQL)?;
        let mut rows = statement.query([repository])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(WorkGeneration {
                work_id: row.get(0)?,
                work_state: row.get(1)?,
                source_name: row.get(2)?,
                content_key: row.get(3)?,
                observed_at: row.get(4)?,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F3: exactly the four namespaces are created — no more, no fewer.
    ///
    /// DuckDB's own default `main` does not appear because DuckDB itself
    /// marks it internal, not because this module filters it out; Atlas
    /// simply never puts anything in it (a table with no namespace has no
    /// owner).
    #[test]
    fn open_declares_exactly_the_atlas_schema_namespaces() {
        let atlas = AtlasDb::open_in_memory().expect("atlas");
        assert_eq!(atlas.schema_names().expect("schemas"), SCHEMAS);
    }

    /// The DDL is idempotent, because reopening the file is the normal path.
    ///
    /// This is **not** F1's restart-persistence regression test — that one
    /// proves derived *facts* survive a daemon restart and lands with the
    /// first writer that can produce a fact to lose. This asserts only that
    /// applying the namespace DDL to a database that already has the
    /// namespaces neither fails nor duplicates them.
    #[test]
    fn reopening_an_existing_file_neither_fails_nor_duplicates_a_schema() {
        let dir = tempfile::tempdir().expect("tempdir");
        let first = AtlasDb::open(dir.path()).expect("open");
        let names = first.schema_names().expect("schemas");
        assert_eq!(first.path(), atlas_db_path(dir.path()));
        drop(first);

        let second = AtlasDb::open(dir.path()).expect("reopen");
        assert_eq!(second.schema_names().expect("schemas"), names);
    }

    /// Register row 12's S4 rider, half one: a read-only open of a store
    /// that does not exist yet must refuse rather than materialize one —
    /// [`AtlasDb::open`]'s own `create_dir_all_durable` + `Connection::open`
    /// would have brought both the directory and the file into being; this
    /// path must do neither.
    #[test]
    fn open_read_only_refuses_a_store_that_does_not_exist_and_creates_nothing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = AtlasDb::open_read_only(dir.path())
            .expect_err("no store exists yet; a read-only open must not create one");
        assert!(
            !atlas_dir(dir.path()).exists(),
            "must not create the directory: {err}"
        );
        assert!(
            !atlas_db_path(dir.path()).exists(),
            "must not create the file: {err}"
        );
    }

    /// Register row 12's S4 rider, half two: once a real (write) open has
    /// built the store, a read-only open must see exactly what was confirmed
    /// — and must not itself be able to write, which is the property that
    /// makes "no DDL" a fact DuckDB enforces rather than a habit this file
    /// keeps. `stage_scan` issues genuine `INSERT`s; if the read-only
    /// connection could run them, this would silently pass.
    #[test]
    fn open_read_only_reads_confirmed_rows_and_cannot_write() {
        let dir = tempfile::tempdir().expect("tempdir");
        {
            let mut writer = AtlasDb::open(dir.path()).expect("seed the store");
            record(&mut writer, &scan_of("notes", "# One\n"), "evt-1");
        }

        let mut reader = AtlasDb::open_read_only(dir.path()).expect("read-only open");
        let sources = reader.indexed_sources().expect("read confirmed coverage");
        assert_eq!(
            sources
                .iter()
                .map(|s| s.source_name.as_str())
                .collect::<Vec<_>>(),
            vec!["notes"],
            "a read-only open must see the generation the write path confirmed"
        );

        let write = reader.stage_scan(&scan_of("notes", "# Two\n"));
        assert!(
            write.is_err(),
            "a connection opened AccessMode::ReadOnly must refuse a write, proving this is a \
             real read-only open and not merely one that happens not to have written yet"
        );
    }

    /// The store must not land inside the directory whose contract is that
    /// deleting it loses nothing.
    #[test]
    fn the_store_lives_outside_the_disposable_projections_directory() {
        let data = Path::new("/data");
        let path = atlas_db_path(data);
        assert!(
            !path.starts_with(crate::runtime::startup::projections_dir(data)),
            "{} must not live under the disposable projections directory",
            path.display()
        );
        assert_eq!(path, data.join(ATLAS_DIR).join(ATLAS_DB_FILE));
    }

    /// A scan of `source` holding one file with `body`, built by hand: these
    /// tests are about the store's own invariants, not about the walk.
    fn scan_of(source: &str, body: &str) -> SourceScan {
        use std::collections::BTreeSet;
        let hash = crate::domain::source::content_hash(body.as_bytes());
        let key = crate::domain::source::local_key(&hash, "markdown/v1");
        SourceScan {
            source_name: source.to_string(),
            kind: SourceKind::LocalKnowledge,
            authority: AuthorityClass::EstateReadonly,
            content_key: hash.clone(),
            revision: None,
            observed_at: crate::domain::event::rfc3339_utc_now(),
            files: vec![ScannedFile {
                relative_path: "doc.md".to_string(),
                content_hash: hash,
                extractor: "markdown/v1".to_string(),
                local_key: key,
                byte_len: body.len() as u64,
                mtime_millis: None,
                units: vec![ScannedUnit {
                    ordinal: 0,
                    kind: UnitKind::Document,
                    heading_level: None,
                    title: None,
                    byte_start: 0,
                    byte_end: body.len() as u64,
                    text: body.to_string(),
                }],
                syntax: None,
            }],
            coverage: vec![CoverageRow {
                path: Some("doc.md".to_string()),
                status: Coverage::Indexed,
                detail: Some("markdown/v1".to_string()),
                bytes: Some(body.len() as u64),
            }],
            extractors: BTreeSet::from(["markdown/v1".to_string()]),
            datasets: Vec::new(),
            root: None,
            context_fields: crate::runtime::atlas::tabular::ContextFields::none(),
        }
    }

    /// Stage and confirm one generation, returning its id.
    fn record(db: &mut AtlasDb, scan: &SourceScan, event: &str) -> String {
        let ScanCommit::Staged { generation_id } = db.stage_scan(scan).expect("stage") else {
            panic!("expected a staged generation");
        };
        db.confirm_scan(&generation_id, event).expect("confirm");
        generation_id
    }

    /// [`scan_of`], stamped `external_git`/`external` — the S4 Y5 shape.
    fn external_scan_of(source: &str, body: &str, tree_oid: &str) -> SourceScan {
        let mut scan = scan_of(source, body);
        scan.kind = SourceKind::ExternalGit;
        scan.authority = AuthorityClass::External;
        scan.content_key = tree_oid.to_string();
        scan
    }

    fn provenance(commit: &str) -> ExternalGitProvenance {
        ExternalGitProvenance {
            origin: "https://example.com/upstream.git".to_string(),
            requested_ref: "HEAD".to_string(),
            resolved_commit: commit.to_string(),
            retrieved_at: crate::domain::event::rfc3339_utc_now(),
        }
    }

    /// A1 §9's provenance quintet is written atomically with everything
    /// else a generation stages, and reads back through
    /// `indexed_sources` — the `git.provenance` table this wave adds.
    #[test]
    fn external_git_provenance_is_staged_atomically_and_reads_back() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut db = AtlasDb::open(dir.path()).expect("open");
        let scan = external_scan_of("upstream", "# One\n", "tree-oid-1");
        let expected = provenance("commit-sha-1");
        let ScanCommit::Staged { generation_id } =
            db.stage_external_git_scan(&scan, &expected).expect("stage")
        else {
            panic!("expected staged");
        };
        db.confirm_scan(&generation_id, "evt-1").expect("confirm");

        let sources = db.indexed_sources().expect("indexed sources");
        let row = sources
            .iter()
            .find(|s| s.source_name == "upstream")
            .expect("the source is present");
        assert_eq!(row.kind, SourceKind::ExternalGit);
        assert_eq!(row.authority, AuthorityClass::External);
        let found = row.provenance.as_ref().expect("provenance is present");
        assert_eq!(
            found,
            &SourceProvenance {
                origin: expected.origin.clone(),
                requested_ref: expected.requested_ref.clone(),
                resolved_commit: expected.resolved_commit.clone(),
                retrieved_at: expected.retrieved_at.clone(),
            }
        );
    }

    /// Every non-`external_git` source's `provenance` reads back `None` —
    /// the `LEFT JOIN`'s honest negative, not an empty-but-present row.
    #[test]
    fn a_non_external_source_has_no_provenance_row() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut db = AtlasDb::open(dir.path()).expect("open");
        record(&mut db, &scan_of("notes", "# Notes\n"), "evt-1");
        let sources = db.indexed_sources().expect("indexed sources");
        let row = sources
            .iter()
            .find(|s| s.source_name == "notes")
            .expect("present");
        assert!(row.provenance.is_none());
    }

    /// A refresh (a new generation whose tree changed) evicts the OLD
    /// generation's provenance row along with everything else eviction
    /// takes — `git.provenance` follows `source.generations`' own lifetime,
    /// never lingering for a generation the store no longer serves.
    #[test]
    fn a_refresh_evicts_the_superseded_generations_provenance() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut db = AtlasDb::open(dir.path()).expect("open");
        let first = external_scan_of("upstream", "# One\n", "tree-oid-1");
        let ScanCommit::Staged {
            generation_id: first_id,
        } = db
            .stage_external_git_scan(&first, &provenance("commit-1"))
            .expect("stage first")
        else {
            panic!("expected staged");
        };
        db.confirm_scan(&first_id, "evt-1").expect("confirm first");

        let second = external_scan_of("upstream", "# Two\n", "tree-oid-2");
        let ScanCommit::Staged {
            generation_id: second_id,
        } = db
            .stage_external_git_scan(&second, &provenance("commit-2"))
            .expect("stage second")
        else {
            panic!("expected staged");
        };
        let evicted = db
            .confirm_scan(&second_id, "evt-2")
            .expect("confirm second");
        assert_eq!(evicted.as_deref(), Some(first_id.as_str()));

        let sources = db.indexed_sources().expect("indexed sources");
        let row = sources
            .iter()
            .find(|s| s.source_name == "upstream")
            .expect("present");
        assert_eq!(
            row.provenance.as_ref().map(|p| p.resolved_commit.as_str()),
            Some("commit-2"),
            "only the surviving generation's provenance is served"
        );
        // The evicted generation's own provenance row is gone, not merely
        // unserved — checked directly against the count rather than through
        // `indexed_sources` (which only ever shows the confirmed row).
        let orphaned: i64 = db
            .conn
            .query_row(
                "SELECT count(*) FROM git.provenance WHERE generation_id = ?",
                duckdb::params![first_id],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(
            orphaned, 0,
            "the evicted generation's provenance row was deleted"
        );
    }

    /// [`AtlasDb::stage_scan`] (the plain path, no provenance) never writes
    /// `git.provenance` — the table stays truthfully empty for every source
    /// kind that has no provenance to report.
    #[test]
    fn plain_stage_scan_writes_no_provenance_row() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut db = AtlasDb::open(dir.path()).expect("open");
        record(&mut db, &scan_of("notes", "# Notes\n"), "evt-1");
        let count: i64 = db
            .conn
            .query_row("SELECT count(*) FROM git.provenance", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 0);
    }

    /// The predecessor may only be evicted by the transaction that actually
    /// promoted its successor.
    ///
    /// `confirm_scan` computes the superseded generation *before* it promotes,
    /// so a stale id — one already confirmed, already evicted, or never
    /// staged — would otherwise evict a perfectly good confirmed generation on
    /// behalf of a promotion that matched zero rows, leaving the source with
    /// nothing confirmed at all. The affected-row count is what makes that
    /// impossible; this is the test that would fail without it.
    #[test]
    fn confirming_a_generation_that_is_not_provisional_evicts_nothing() {
        let mut atlas = AtlasDb::open_in_memory().expect("atlas");
        let first = record(&mut atlas, &scan_of("notes", "# One\n"), "evt-1");
        let second = record(&mut atlas, &scan_of("notes", "# Two\n"), "evt-2");
        assert_eq!(
            atlas.generation_states().expect("states").get(&first),
            Some(&STATE_EVICTED.to_string()),
            "the ordinary supersession still happens"
        );

        // Replaying the *first* confirmation: its generation is long evicted.
        let err = atlas
            .confirm_scan(&first, "evt-1")
            .expect_err("a stale confirmation must be refused");
        assert!(
            matches!(&err, AtlasError::NotProvisional { generation_id } if *generation_id == first),
            "{err:?}"
        );
        // And re-confirming the generation that currently stands, which is no
        // longer provisional either.
        assert!(matches!(
            atlas.confirm_scan(&second, "evt-2"),
            Err(AtlasError::NotProvisional { .. })
        ));

        // The standing generation survived both, rows and all.
        let standing = atlas
            .confirmed_generation("notes")
            .expect("generation")
            .expect("the confirmed generation must still stand");
        assert_eq!(standing.id, second);
        assert_eq!(atlas.units("notes", 100).expect("units").len(), 1);
    }

    /// Ruling §4 evicts a generation only when the source *bytes* changed. An
    /// unreadable root changed none — so an empty scan of an unplugged drive
    /// must not supersede a good generation, and the unavailability is
    /// recorded rather than swallowed.
    #[test]
    fn an_unreadable_root_keeps_the_confirmed_generation_and_says_so() {
        let mut atlas = AtlasDb::open_in_memory().expect("atlas");
        let good = record(&mut atlas, &scan_of("notes", "# Kept\n"), "evt-1");

        let mut unreachable = scan_of("notes", "");
        unreachable.files.clear();
        unreachable.extractors.clear();
        unreachable.content_key = crate::domain::source::generation_key(&BTreeMap::new());
        unreachable.coverage = vec![CoverageRow {
            path: None,
            status: Coverage::Unavailable,
            detail: Some("the declared knowledge path cannot be read: no such file".to_string()),
            bytes: None,
        }];
        assert!(unreachable.root_unavailable().is_some(), "fixture");

        let commit = atlas.stage_scan(&unreachable).expect("stage");
        assert!(
            matches!(&commit, ScanCommit::RootUnavailable { generation_id, .. } if *generation_id == good),
            "{commit:?}"
        );
        assert_eq!(
            atlas
                .confirmed_generation("notes")
                .expect("generation")
                .expect("still standing")
                .id,
            good
        );
        assert_eq!(
            atlas.units("notes", 100).expect("units").len(),
            1,
            "a transient mount failure must not destroy derived facts"
        );
        let unavailable = atlas
            .coverage("notes", 100)
            .expect("coverage")
            .into_iter()
            .find(|c| c.row.status == Coverage::Unavailable)
            .expect("the unavailability must be recorded, not swallowed");
        assert_eq!(unavailable.generation_id, good);
        assert!(
            !atlas
                .coverage("notes", 100)
                .expect("coverage")
                .iter()
                .any(|c| c.row.status == Coverage::GenerationEvicted),
            "no eviction: the source bytes did not change"
        );

        // A readable root that is genuinely empty is a different fact, and it
        // does supersede — emptiness that was actually observed is evidence.
        let mut emptied = scan_of("notes", "");
        emptied.files.clear();
        emptied.extractors.clear();
        emptied.content_key = crate::domain::source::generation_key(&BTreeMap::new());
        emptied.coverage.clear();
        assert!(matches!(
            atlas.stage_scan(&emptied).expect("stage"),
            ScanCommit::Staged { .. }
        ));
    }

    /// The store no longer evicts on open: a provisional generation is
    /// unreadable, and whether it deserves promotion is a question only the
    /// journal answers.
    #[test]
    fn opening_the_store_leaves_a_provisional_generation_for_the_journal_to_judge() {
        let dir = tempfile::tempdir().expect("tempdir");
        let staged = {
            let mut atlas = AtlasDb::open(dir.path()).expect("atlas");
            let ScanCommit::Staged { generation_id } = atlas
                .stage_scan(&scan_of("notes", "# Pending\n"))
                .expect("stage")
            else {
                panic!("expected a staged generation");
            };
            generation_id
        };
        let mut atlas = AtlasDb::open(dir.path()).expect("reopen");
        assert_eq!(
            atlas.generation_states().expect("states").get(&staged),
            Some(&STATE_PROVISIONAL.to_string()),
            "opening must not decide the crash window on its own"
        );
        assert!(
            atlas
                .confirmed_generation("notes")
                .expect("generation")
                .is_none(),
            "and it is still unreadable while it waits"
        );
        assert_eq!(
            atlas.provisional_generations().expect("awaiting"),
            vec![(staged.clone(), "notes".to_string())]
        );
        // The eviction half, once something has consulted the journal.
        assert_eq!(
            atlas
                .evict_provisional(std::slice::from_ref(&staged), "no summary")
                .expect("evict"),
            vec![staged.clone()]
        );
        assert_eq!(
            atlas.generation_states().expect("states").get(&staged),
            Some(&STATE_EVICTED.to_string())
        );
        // Idempotent: a second pass has nothing left to evict, and a
        // confirmed generation is never reachable this way.
        assert!(
            atlas
                .evict_provisional(&[staged], "no summary")
                .expect("evict")
                .is_empty()
        );
    }

    #[test]
    fn debug_names_the_path_and_never_the_connection() {
        let atlas = AtlasDb::open_in_memory().expect("atlas");
        let debug = format!("{atlas:?}");
        assert!(debug.contains(":memory:"), "{debug}");
        assert!(!debug.contains("Connection"), "{debug}");
    }
}

#[cfg(test)]
mod ops_tests {
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

    /// S3 F3. The requalification is physical, not cosmetic: every table this
    /// module creates lives in the `ops` namespace and DuckDB's default
    /// `main` schema holds nothing at all. Read back out of the catalog, so a
    /// table added later without a namespace fails here rather than quietly
    /// landing in `main` and still answering queries.
    #[test]
    fn every_table_lives_in_the_ops_schema_and_main_holds_nothing() {
        let analytics = Analytics::in_memory(Vec::new()).expect("projection");
        let (columns, rows) = analytics
            .select(
                "SELECT schema_name, table_name FROM duckdb_tables() \
                 WHERE database_name = current_database() ORDER BY table_name",
                duckdb::params![],
            )
            .expect("select");
        assert_eq!(columns, vec!["schema_name", "table_name"]);
        let mut expected: Vec<&str> = TABLES.to_vec();
        expected.sort_unstable();
        let found: Vec<String> = rows
            .iter()
            .map(|row| {
                assert_eq!(
                    row[0],
                    json!(OPS_SCHEMA),
                    "{:?} is not in the ops namespace",
                    row[1]
                );
                row[1].as_str().expect("table name").to_string()
            })
            .collect();
        assert_eq!(found, expected);
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

    /// H1 touch point #6: `estate_root` folds from the envelope at
    /// `work.submitted` for a Work that never reaches `workflow.bound`
    /// (today's documented `estate: None`-forever gap, now answered), and
    /// stays `NULL` — never an error — for a pre-Phase-C-shaped legacy line
    /// whose envelope never carried `workspace_id` at all (`Compatibility`
    /// deliverable: old journal lines replay unchanged).
    #[test]
    fn work_estate_root_folds_from_the_submitted_envelope_and_stays_null_for_a_legacy_line() {
        let root = "/estates/payments";
        let mut current = submitted(1, "current");
        current.workspace_id = Some(root.to_string());
        // No `workspace_id` at all — exactly what a stored pre-Phase-C
        // journal line deserializes to (`Event`'s `#[serde(default)]`).
        let legacy = submitted(2, "legacy");

        let mut analytics =
            Analytics::in_memory(events(vec![current, legacy])).expect("projection");
        analytics.materialize().expect("materialize");
        let (columns, rows) = analytics
            .select(
                "SELECT work_id, estate_root FROM ops.work ORDER BY work_id",
                duckdb::params![],
            )
            .expect("select");
        assert_eq!(columns, vec!["work_id", "estate_root"]);
        assert_eq!(
            rows,
            vec![
                vec![json!("current"), json!(root)],
                vec![json!("legacy"), Value::Null],
            ]
        );
    }

    /// W2 §9.1 step 3: a failed `push` (surfaced through `finish`, since the
    /// append-only tables only flush at chunk boundaries or at `finish`)
    /// leaves `needs_reset` armed — the invariant `catch_up`'s doc comment
    /// protects, now split across `fold`/`push`/`finish`.
    ///
    /// Injection: drop the `events` table out from under a live connection,
    /// so the buffered append at `finish` fails at the DB layer while
    /// `last_seq` has already advanced past it.
    #[test]
    fn a_failed_push_leaves_needs_reset_armed() {
        let mut analytics = Analytics::in_memory(Vec::new()).expect("projection");
        analytics
            .conn
            .execute_batch("DROP TABLE ops.events")
            .expect("drop the events table to force the next append to fail");

        let mut fold = analytics.fold().expect("fold");
        fold.push(&submitted(1, "w1"))
            .expect("push buffers, does not write yet");
        let err = fold
            .finish()
            .expect_err("finish must surface the failed append");
        assert!(matches!(err, AnalyticsError::Duck(_)));

        // `last_seq()` reads 0 while `needs_reset` is armed — the one public
        // proxy for the private flag, per its own doc comment.
        assert_eq!(
            analytics.last_seq(),
            0,
            "a failed fold must leave needs_reset armed, reported as last_seq() == 0"
        );
    }

    #[test]
    fn an_unparseable_timestamp_leaves_a_null_cell_and_never_fails_the_fold() {
        let mut odd = submitted(1, "w1");
        odd.timestamp = "whenever".to_string();
        let mut analytics = Analytics::in_memory(events(vec![odd])).expect("projection");
        analytics.materialize().expect("materialize");
        let (_, rows) = analytics
            .select("SELECT ts_ms FROM ops.events", duckdb::params![])
            .expect("select");
        assert_eq!(rows, vec![vec![Value::Null]]);
    }

    /// An event with no work scope at all (as opposed to `event()`, which
    /// always sets one) — the shape several §33 guards actually fire on.
    fn event_no_work(seq: u64, kind: &str, payload: Value) -> Event {
        EventDraft::new(EventSource::new("daemon", "test"), kind, payload).into_event(seq)
    }

    /// Issue #33 item 1: `Analytics::apply`'s malformed/incomplete-event
    /// guards, never fired in the baseline, across every kind that has one.
    /// Each of these is missing exactly the field its handler reads back out
    /// — `catch_up` (§20's forward-compatibility contract) must skip the row
    /// rather than panic or write a partial one.
    #[test]
    fn malformed_events_across_kinds_are_skipped_not_panicked() {
        let mut malformed = vec![
            // KIND_WORK_SUBMITTED: no "work.id" in the payload.
            event_no_work(
                1,
                KIND_WORK_SUBMITTED,
                json!({"work": {"intent": "orphan"}}),
            ),
            // KIND_WORKFLOW_BOUND: work_id names a work never submitted, so
            // it is absent from `rows.work` (the issue's own example).
            event(
                2,
                "ghost-work",
                KIND_WORKFLOW_BOUND,
                json!({"backend": "fake"}),
            ),
            // KIND_SURFACE_MATERIALIZED / TORN_DOWN: no work_id on the
            // envelope at all. The bindings are deliberately non-empty and
            // name a real repository: both handlers only write from inside
            // the loop over them, so with `"bindings": []` the arm would
            // write nothing whether or not the work_id guard is there and
            // the repositories assertion below could not tell the two apart.
            event_no_work(
                3,
                KIND_SURFACE_MATERIALIZED,
                json!({"surface": {"bindings": [
                    {"repository": "repo", "source_path": "/src/repo", "work_branch": "sgt/w1"},
                ]}}),
            ),
            event_no_work(
                4,
                KIND_SURFACE_TORN_DOWN,
                json!({"report": {"bindings": [
                    {"repository": "repo", "disposition": "kept"},
                ]}}),
            ),
            // KIND_STAGE_ENTERED: work_id present, stage_id missing.
            event(5, "w1", KIND_STAGE_ENTERED, json!({"index": 0})),
            // KIND_STAGE_COMPLETED: stage_id names a stage never entered, so
            // no attempt can be found for it.
            event(
                6,
                "w1",
                KIND_STAGE_COMPLETED,
                json!({"stage_id": "never-entered"}),
            ),
            // KIND_EXECUTION_STARTED: no execution_id in the payload.
            event(
                7,
                "w1",
                KIND_EXECUTION_STARTED,
                json!({"execution": {"backend": "fake"}}),
            ),
            // KIND_EXECUTION_STOPPED: no execution_id, and an unknown one.
            event(8, "w1", KIND_EXECUTION_STOPPED, json!({})),
            event(
                9,
                "w1",
                KIND_EXECUTION_STOPPED,
                json!({"execution_id": "ghost-exec"}),
            ),
            // KIND_EXECUTION_RECONCILED: no execution_id, and an unknown one.
            event(
                10,
                "w1",
                KIND_EXECUTION_RECONCILED,
                json!({"disposition": "resumed"}),
            ),
            event(
                11,
                "w1",
                KIND_EXECUTION_RECONCILED,
                json!({"execution_id": "ghost-exec", "disposition": "resumed"}),
            ),
            // KIND_TOOL_REQUESTED: no execution scope on the envelope.
            event(
                12,
                "w1",
                KIND_TOOL_REQUESTED,
                json!({"id": "t1", "name": "Bash"}),
            ),
        ];
        // KIND_TOOL_COMPLETED: execution scope present, tool_use_id absent.
        let mut tool_completed = event(13, "w1", KIND_TOOL_COMPLETED, json!({}));
        tool_completed.execution_id = Some("e1".to_string());
        malformed.push(tool_completed);

        let total = malformed.len() as i64;
        let mut analytics = Analytics::in_memory(events(malformed)).expect("projection");
        let counts: BTreeMap<String, i64> = analytics
            .table_counts()
            .expect("counts")
            .into_iter()
            .collect();
        for table in ["work", "stages", "executions", "tool_calls", "repositories"] {
            assert_eq!(
                counts[table], 0,
                "{table} should be untouched by malformed events: {counts:?}"
            );
        }
        assert_eq!(
            counts["events"], total,
            "every malformed event is still appended to the raw log, never dropped"
        );
    }

    /// Issue #33 item 1: `KIND_EXECUTION_RECONCILED`'s handler — the whole
    /// arm never ran in the baseline — actually updates the matching row's
    /// disposition on a real reconcile.
    #[test]
    fn execution_reconciled_records_the_disposition_on_the_matching_row() {
        let started = event(
            1,
            "w1",
            KIND_EXECUTION_STARTED,
            json!({"execution": {
                "execution_id": "e1", "backend": "fake", "native_id": "n1",
                "stage_id": "00-first", "attempt": 1, "stop_requested": false,
            }}),
        );
        let reconciled = event(
            2,
            "w1",
            KIND_EXECUTION_RECONCILED,
            json!({"execution_id": "e1", "disposition": "resumed"}),
        );
        let mut analytics =
            Analytics::in_memory(events(vec![started, reconciled])).expect("projection");
        analytics.materialize().expect("materialize");
        let (_, rows) = analytics
            .select(
                "SELECT reconcile_disposition FROM ops.executions WHERE execution_id = 'e1'",
                duckdb::params![],
            )
            .expect("select");
        assert_eq!(rows, vec![vec![Value::String("resumed".to_string())]]);
    }

    /// Issue #33's dead-code note: `Analytics`'s manual `Debug` impl is
    /// decided keep-with-caller rather than dropped, which means it needs an
    /// actual caller so it is not an unmeasured claim — this is it.
    #[test]
    fn the_debug_impl_names_the_projection_by_path_and_last_seq() {
        let analytics = Analytics::in_memory(events(vec![submitted(1, "w1")])).expect("projection");
        let debug = format!("{analytics:?}");
        assert!(debug.starts_with("Analytics {"), "{debug}");
        // Both halves the name promises: the path is the whole reason the
        // manual impl exists, so asserting only `last_seq` would leave the
        // interesting field unmeasured.
        assert!(debug.contains("path: \":memory:\""), "{debug}");
        assert!(debug.contains("last_seq: 1"), "{debug}");
    }
}

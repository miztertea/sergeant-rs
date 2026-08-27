//! The one owner of Atlas's database file (F2).
//!
//! ```text
//! <data-dir>/atlas/atlas.duckdb
//! ```
//!
//! This file is the only place under `runtime/atlas/` that names the `duckdb`
//! crate, and the [`Connection`] it holds is a private field with no
//! accessor — exactly the shape [`crate::runtime::analytics`] holds for the
//! operations projection, held independently for this database.
//! `tests/x1_atlas_substrate.rs` pins it structurally; that test is a
//! separate assertion from M5's, over a separate tree, because two databases
//! with one owner each is a different (and stronger) property than two files
//! sharing permission to open any database.
//!
//! # This file is not a projection (F1)
//!
//! [`crate::runtime::analytics`] opens its file by deleting it: the
//! operations tables are a pure fold of the journal, so the rebuild path and
//! the startup path are the same path, and losing the file loses nothing.
//!
//! **None of that is true here.** [`AtlasDb::open`] opens the existing file
//! and keeps it, because `source.*`, `git.*` and `meta.coverage` PERSIST
//! across restarts. They are derived from source bytes plus extractor
//! identity, keyed by SourceGeneration; no
//! journal replay reproduces them. A generation is evicted only when the
//! source bytes it was derived from changed, and the eviction is reported as
//! a coverage row rather than a silent gap (ruling §4). The journal carries
//! one compact `source.scanned` summary per completed scan so the
//! authoritative trail stays journal-side while the unit-level detail stays
//! here.
//!
//! Two consequences bind any later wave:
//!
//! * Nothing may delete this file to "fix" it the way a projection may be
//!   deleted, and no cleanup path may treat it as disposable state.
//! * The DDL below is `IF NOT EXISTS` because reopening an existing file is
//!   the normal path, not a recovery path.
//!
//! # Why the file is not under `projections/`
//!
//! The operations projection lives in `<data-dir>/projections/`, whose whole
//! documented contract is that deleting the directory loses nothing — an
//! acceptance test deletes it wholesale and asserts exactly that. A database
//! that must survive restarts cannot live inside a directory that is
//! advertised as disposable, so Atlas gets its own directory beside the
//! journal and the blobs, at the same level as `projections/`.
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

use duckdb::{Connection, Statement};

use crate::domain::source::{
    AuthorityClass, Coverage, CoverageRow, SourceGeneration, SourceKind, UnitKind,
};
use crate::runtime::atlas::scan::{ScannedFile, ScannedSyntax, ScannedUnit, SourceScan};
use crate::runtime::atlas::tabular::{
    DatasetFormat, RowKeyBasis, RowUnit, ScannedDataset, row_units,
};
use crate::runtime::fsutil::create_dir_all_durable;

/// Directory under the data dir holding Atlas's durable store.
///
/// Deliberately not [`crate::runtime::analytics::PROJECTIONS_DIR`]: that
/// directory is disposable by contract and this one is not.
pub const ATLAS_DIR: &str = "atlas";

/// Atlas's database file name inside [`ATLAS_DIR`].
pub const ATLAS_DB_FILE: &str = "atlas.duckdb";

/// The schema namespaces Atlas declares (A1 §5, F3).
///
/// `meta` holds Atlas's own bookkeeping (coverage above all); `source` holds
/// what was derived from source bytes; `git` holds what was derived from Git
/// objects; `context` holds the retrieval-facing units assembled from the
/// other two. Sorted, because [`AtlasDb::schema_names`] answers sorted and
/// the two are compared directly.
pub const SCHEMAS: &[&str] = &["context", "git", "meta", "source"];

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
/// [`crate::runtime::analytics::CANNED_QUERIES`] is one: an endpoint that runs
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
        conn.execute_batch(HARDENING_DDL)?;
        conn.execute_batch(SCHEMA_DDL)?;
        conn.execute_batch(TABLE_DDL)?;
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
                       WHERE r.generation_id = g.generation_id) AS row_units \
             FROM source.generations g WHERE g.state = ? \
             ORDER BY g.source_name LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![STATE_CONFIRMED, MAX_ROWS as i64])?;
        let mut out: Vec<SourceStatus> = Vec::new();
        while let Some(row) = rows.next()? {
            let kind: String = row.get(1)?;
            let authority: String = row.get(2)?;
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
            !path.starts_with(crate::runtime::analytics::projections_dir(data)),
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

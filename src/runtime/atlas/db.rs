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
//! The tables the three walks write, and nothing else. Every table
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
use std::sync::OnceLock;

use duckdb::types::Value as Duck;
use duckdb::{Connection, Statement};

use serde_json::{Map, Value, json};
use store::{Name, ReadOnly, Sql, Statements, Store};

use crate::domain::event::{Event, unix_millis};
use crate::domain::execution::{
    KIND_EXECUTION_RECONCILED, KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
};
/// A2 §2 stage 1's estate axis, re-exported beside [`Admissibility`] — every
/// caller that builds a filter has to name it, and naming it beside the
/// struct it is a field of is what keeps that from being a scavenger hunt.
pub use crate::domain::source::EstateAdmission;
use crate::domain::source::{
    AuthorityClass, Coverage, CoverageRow, EstateBinding, SourceGeneration, SourceKind, UnitKind,
};

/// S6 D1: the single estate this file's own unit tests record and query
/// under. The **cross-estate** case — the one this axis exists for — is an
/// end-to-end suite (`tests/d1_estate_isolation.rs`), not a unit test here,
/// because the leak was in what the daemon passed to this filter, not in
/// this filter's arithmetic.
#[cfg(test)]
const TEST_ESTATE: &str = "/estates/db-unit";
use crate::domain::work::{KIND_WORK_SUBMITTED, WorkState};
use crate::domain::workflow::{
    KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED,
    KIND_STAGE_FAILED, KIND_STAGE_NEEDS_INPUT, KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND,
};
use crate::runtime::atlas::external_git::ExternalGitProvenance;
use crate::runtime::atlas::fusion::{
    FusedHit, RerankSignals, exact_match, fuse, is_canonical_path, rerank, same_section, symbol_of,
};
use crate::runtime::atlas::lexical::{
    Bm25Corpus, LexicalFamily, LexicalHit, UnitCoordinate, bm25_contribution, is_identifier_like,
    query_terms, rank_order, term_frequencies,
};
use crate::runtime::atlas::scan::{ScannedFile, ScannedSyntax, ScannedUnit, SourceScan};
use crate::runtime::atlas::semantic::{
    SemanticEngine, SemanticHit, SemanticModel, SemanticRequest, SemanticStatus, cosine,
    rank_semantic, resolve as resolve_semantic,
};
use crate::runtime::atlas::tabular::{
    DatasetFormat, RowKeyBasis, RowUnit, ScannedDataset, row_units,
};
use crate::runtime::atlas::trace::{
    Attribution, ContentAuthorityFilter, LexicalIdentity, PolicyIdentity, QueryIdentity,
    RETRIEVAL_INDEX_VERSION, ResultRank, RetrievalGeneration, SearchTrace, SourceGenerationFilter,
};
use crate::runtime::fsutil::create_dir_all_durable;
use crate::runtime::graph::{
    GraphContext, KIND_CONVERSATION_ASSISTANT_COMPLETED, KIND_CONVERSATION_USER,
    KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};
use crate::runtime::journal::JournalError;
use crate::runtime::surface::{KIND_SURFACE_MATERIALIZED, KIND_SURFACE_TORN_DOWN};

// ---------------------------------------------------------------------------
// The three constructors for everything this file hands the database driver.
//
// They live here, above every call site, because `macro_rules!` is textually
// scoped: a macro is usable only *after* its definition in source order, and
// `rows_sql` — the first caller — is a few hundred lines below.
//
// Each one puts its argument into an associated `const`, and that is the
// entire mechanism; see
// [`store::SqlText`](crate::runtime::atlas::db::store::SqlText) for why const
// evaluation, and not a `&'static str` bound, is what makes a caller's string
// unable to reach DuckDB.
// ---------------------------------------------------------------------------

/// A statement this crate wrote, in the one type [`store::Store`] will run.
///
/// `$text` is placed in an associated `const`, so the compiler evaluates it:
/// `sql!(Box::leak(caller.to_string().into_boxed_str()))` is E0015 and
/// `sql!(some_local)` is E0435. Neither is a lint, a scan, or a convention.
///
/// The arm is `$text:expr` rather than `$text:literal` on purpose. A literal
/// arm would reject `sql!(HARDENING_DDL)` — a path, not a literal token —
/// while adding nothing: the `const` refuses every non-const expression a
/// literal arm would have refused, and refuses `include_str!`-shaped ones no
/// differently than the literal arm would have admitted them.
macro_rules! sql {
    ($text:expr) => {{
        struct SqlLiteral;
        impl $crate::runtime::atlas::db::store::SqlText for SqlLiteral {
            const TEXT: &'static str = $text;
        }
        $crate::runtime::atlas::db::store::Sql::of::<SqlLiteral>()
    }};
}

/// The `ops` table list and each table's qualified SQL reference, declared
/// **once**.
///
/// [`Sql`] cannot be built from a runtime string, and an operations table's
/// name *is* a runtime string at the point it is needed — it comes out of
/// `TABLES` by value inside a loop. So the qualification is a `match` over
/// compile-time alternatives rather than an interpolation, and this macro
/// emits the list and the match from one source so the two cannot drift.
/// Every arm's right-hand side is the whole reference, quoted: the quoting is
/// what keeps `usage` (a reserved word) addressable, and the qualification is
/// what stops a bare name from resolving against DuckDB's default `main`
/// schema, which this database deliberately leaves empty.
macro_rules! ops_tables {
    ($($name:literal => $qualified:literal,)+) => {
        /// Tables this projection creates, in a stable order. Crate-internal:
        /// the table list is an implementation detail of the projection, and
        /// callers get it as data from [`Analytics::table_counts`].
        const TABLES: &[&str] = &[$($name),+];

        /// One operations table, qualified and quoted for SQL.
        ///
        /// Total over [`TABLES`] by construction — both come out of the one
        /// `ops_tables!` invocation — and
        /// `every_mutable_table_is_an_ops_table` pins the only other list of
        /// names that reaches here.
        fn ops(table: &str) -> Sql {
            match table {
                $($name => sql!($qualified),)+
                other => unreachable!(
                    "`{other}` is not an `ops` table; every caller takes its name from \
                     TABLES or MUTABLE_TABLES, which this macro and its test cover"
                ),
            }
        }
    };
}

/// [`CANNED_QUERIES`] and the statement each one runs, declared **once**.
///
/// Same shape and same reason as [`ops_tables!`]: the *name* a caller asks
/// for is a runtime string, [`Sql`] cannot be built from one, and the
/// published `sql` field has to stay `&'static str` because it is displayed
/// and hashed rather than executed. So the macro emits the fixed list and a
/// `match` from the same tokens, and the executed statement is never the
/// field — it is a [`sql!`] over the identical literal.
macro_rules! canned_queries {
    ($(CannedQuery { name: $name:literal, question: $question:literal, sql: $statement:literal, },)+) => {
        /// The canned queries this build answers.
        ///
        /// Deliberately a fixed list rather than arbitrary client SQL: §22's
        /// "clients do not access DuckDB directly" is about the *one-owner*
        /// property, and an endpoint that executes a client's SQL against the
        /// daemon's database hands the ownership back. M6 owns presentation;
        /// this is the data behind it.
        pub const CANNED_QUERIES: &[CannedQuery] = &[
            $(CannedQuery { name: $name, question: $question, sql: $statement },)+
        ];

        /// The statement one canned query runs.
        ///
        /// Total over [`CANNED_QUERIES`] by construction, and the only caller
        /// has already looked the name up in that list.
        fn canned_sql(name: &str) -> Sql {
            match name {
                $($name => sql!($statement),)+
                other => unreachable!(
                    "`{other}` is not a canned query; the caller resolves the name \
                     against CANNED_QUERIES first"
                ),
            }
        }
    };
}

/// A table or schema name this crate wrote — [`sql!`] for [`store::Name`].
macro_rules! name {
    ($text:expr) => {{
        struct NameLiteral;
        impl $crate::runtime::atlas::db::store::SqlText for NameLiteral {
            const TEXT: &'static str = $text;
        }
        $crate::runtime::atlas::db::store::Name::of::<NameLiteral>()
    }};
}

/// A [`store::ReadSql`] whose read check runs at **compile time**.
///
/// Two things are being enforced, and they are not the same thing:
///
/// 1. `$text` is compile-time text ([`sql!`]'s mechanism, unchanged here), so
///    no caller's string can be the statement.
/// 2. That text is one bare `SELECT` with no `;` in it — the check
///    [`store::is_read_statement`] spells out, evaluated in a **named,
///    non-generic `const` item**.
///
/// The shape of (2) is load-bearing. A `const { .. }` block, or the generic
/// associated const inside `ReadSql::of`, is a post-monomorphization error
/// that `cargo check` walks straight past — verified by watching exactly that
/// happen during the S5 closeout. An anonymous `const _` **item** is
/// evaluated eagerly, so `cargo check` fails too.
///
/// And note *why* a const check is the right instrument rather than a scan of
/// this file's text: const evaluation sees the **assembled** string, after
/// `concat!` has resolved. `read_sql!(concat!("DEL", "ETE FROM t"))` is
/// `DELETE FROM t` here. A text scanner reads the source spelling and never
/// sees it.
macro_rules! read_sql {
    ($text:expr) => {{
        struct ReadSqlLiteral;
        impl $crate::runtime::atlas::db::store::SqlText for ReadSqlLiteral {
            const TEXT: &'static str = $text;
        }
        const _: () = assert!(
            $crate::runtime::atlas::db::store::is_read_statement(
                <ReadSqlLiteral as $crate::runtime::atlas::db::store::SqlText>::TEXT
            ),
            "a read-only handle may only run one statement, beginning `SELECT ` and \
             containing no `;`"
        );
        $crate::runtime::atlas::db::store::ReadSql::of::<ReadSqlLiteral>()
    }};
}

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
///
/// S5 W2's two, and why the lexical index is a table here rather than a
/// service (A2 §5, decision A2-05):
///
/// * **`context.lexical_units` is one indexable unit's identity and length.**
///   It lives in `context` for `context.row_units`'s own reason — it is
///   retrieval-facing text assembled from a source — and it is **derived
///   evidence over A1's existing units, not a second chunk universe** (A2
///   §3/A2-02: "A2 does not invent an independent chunk universe"). Every row
///   is derived from a `source.occurrences`, `source.units` or
///   `context.row_units` row that already exists; W2 adds no chunker (R2).
///   The coordinate columns are nullable by family, because A2 §3 gives each
///   family a different coordinate and only three of the four are byte spans
///   — see
///   [`crate::runtime::atlas::lexical::UnitCoordinate`], which is the type
///   this table is read back into.
/// * **`context.lexical_postings` is the inverted index**: one row per
///   (unit, term), with the term's frequency in that unit. Both tables are
///   keyed by `generation_id` and both are deleted by [`evict`] with every
///   other derived row, which is what makes "a superseded generation's
///   postings are evicted with it" a property of the schema rather than a
///   promise — and keeps the reported-never-silent eviction discipline
///   identical to every other table's.
const TABLE_DDL: &str = "\
CREATE TABLE IF NOT EXISTS source.generation_estates (\n\
  generation_id TEXT NOT NULL,\n\
  estate_scope  TEXT NOT NULL,\n\
  estate_root   TEXT\n\
);\n\
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
CREATE TABLE IF NOT EXISTS source.child_resources (\n\
  generation_id        TEXT NOT NULL,\n\
  source_name          TEXT NOT NULL,\n\
  relative_path        TEXT NOT NULL,\n\
  local_key            TEXT NOT NULL,\n\
  parent_relative_path TEXT NOT NULL,\n\
  parent_key           TEXT NOT NULL,\n\
  entry_path           TEXT NOT NULL\n\
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
CREATE TABLE IF NOT EXISTS source.unit_coordinates (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  relative_path TEXT NOT NULL,\n\
  local_key     TEXT NOT NULL,\n\
  ordinal       BIGINT NOT NULL,\n\
  coordinate    TEXT NOT NULL\n\
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
);\n\
CREATE TABLE IF NOT EXISTS context.lexical_units (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  family        TEXT NOT NULL,\n\
  unit_key      TEXT NOT NULL,\n\
  relative_path TEXT NOT NULL,\n\
  ordinal       BIGINT NOT NULL,\n\
  title         TEXT,\n\
  symbol        TEXT,\n\
  language      TEXT,\n\
  label         TEXT,\n\
  dataset_key   TEXT,\n\
  row_key       TEXT,\n\
  fields        TEXT,\n\
  byte_start    BIGINT,\n\
  byte_end      BIGINT,\n\
  token_count   BIGINT NOT NULL\n\
);\n\
CREATE TABLE IF NOT EXISTS context.lexical_postings (\n\
  generation_id  TEXT NOT NULL,\n\
  unit_key       TEXT NOT NULL,\n\
  term           TEXT NOT NULL,\n\
  term_frequency BIGINT NOT NULL\n\
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

/// A2 §2 stage 1's **estate** clause, over the `g` alias — the first thing
/// the admissibility predicate asks, because A2 §2 puts the
/// *"source / estate / Work generation filter"* first and nothing may rank
/// before it (A2 §8).
///
/// Two binds, both the same estate root:
///
/// ```text
/// (?estate IS NOT NULL) AND EXISTS (binding row for g whose scope is
///                                   `host`, or whose root = ?estate)
/// ```
///
/// **The `? IS NOT NULL` guard is the default-deny half and is not
/// redundant.** With no estate named
/// ([`EstateAdmission::NoEstate`](crate::domain::source::EstateAdmission::NoEstate),
/// the `Default`), the bind is SQL `NULL` and this clause is false for every
/// row — host-scoped rows included. Without it, an unnamed estate would
/// still have admitted every `EstateBinding::Host` generation, which is a
/// smaller leak but the same class of one.
///
/// A generation with **no** row in `source.generation_estates` — anything
/// indexed by a build older than S6 D1 — fails the `EXISTS` and is
/// inadmissible everywhere. Fail-closed, repaired by re-scanning; see
/// [`EstateAdmission`](crate::domain::source::EstateAdmission)'s own doc.
macro_rules! admissible_estate_clause {
    () => {
        "? IS NOT NULL \
         AND EXISTS (SELECT 1 FROM source.generation_estates ge \
                     WHERE ge.generation_id = g.generation_id \
                       AND (ge.estate_scope = 'host' OR ge.estate_root = ?))"
    };
}

/// A2 §2's composed stage-1/2/4 admissibility predicate over the `g` alias,
/// as a **compile-time literal** the three W2 statements below splice in with
/// `concat!`.
///
/// A macro rather than a `const &str` because `concat!` takes literals, and a
/// literal is the point: `tests/x5_a1a_acceptance.rs::
/// a1a_item_13_no_client_sql_reaches_the_store` pins every interpolation in
/// every SQL literal in this file, and lexical retrieval adds none. The
/// clause order and shape are identical to the one
/// [`AtlasDb::admissible_generations`] and its three content-kind siblings
/// spell inline; the bind values are
/// [`AtlasDb::admissibility_binds`], in this order:
///
/// ```text
/// state, estate x2, overlay-exclude LIKE, source_name x2, overlay-admit x2,
/// content_key x2, source_kind x2, authority_class x2
/// ```
macro_rules! admissible_generations_where {
    () => {
        concat!(
            "g.state = ? AND ",
            admissible_estate_clause!(),
            " AND ( (g.source_name NOT LIKE ? \
                AND (? IS NULL OR g.source_name = ?)) \
               OR (? IS NOT NULL AND g.source_name = ?) ) \
         AND (? IS NULL OR g.content_key = ?) \
         AND (? IS NULL OR g.source_kind = ?) \
         AND (? IS NULL OR g.authority_class = ?)"
        )
    };
}

/// The join every W2 statement makes: postings to their unit, unit to its
/// generation. A posting whose generation the admissibility predicate
/// excludes is unreachable through it — which is what makes A2 §8's "never
/// silently crossing an authority/source filter" structural here rather than
/// procedural.
macro_rules! lexical_posting_join {
    () => {
        lexical_posting_join!("")
    };
    // `$extra` is spliced between the joins and the `WHERE`, and must be a
    // literal for the same reason every statement in this file is one
    // (item 13's no-client-SQL pin): `concat!` takes literals only.
    ($extra:expr) => {
        concat!(
            "FROM context.lexical_postings p \
             JOIN context.lexical_units l ON l.generation_id = p.generation_id \
                                          AND l.unit_key = p.unit_key \
             JOIN source.generations g ON g.generation_id = l.generation_id ",
            $extra,
            " WHERE "
        )
    };
}

/// BM25's corpus statistics — `N` and the mean document length — measured
/// over the admissible, family-filtered set and nothing wider.
const LEXICAL_CORPUS_SQL: &str = concat!(
    "SELECT count(*), coalesce(sum(l.token_count), 0) \
     FROM context.lexical_units l \
     JOIN source.generations g USING (generation_id) \
     WHERE ",
    admissible_generations_where!(),
    " AND (? IS NULL OR l.family = ?)"
);

/// One term's document frequency over that same set.
const LEXICAL_DOCUMENT_FREQUENCY_SQL: &str = concat!(
    "SELECT count(*) ",
    lexical_posting_join!(),
    admissible_generations_where!(),
    " AND (? IS NULL OR l.family = ?) AND p.term = ?"
);

/// One term's postings, with every coordinate column a hit has to cite.
const LEXICAL_POSTINGS_SQL: &str = concat!(
    "SELECT l.generation_id, l.source_name, g.content_key, l.family, l.unit_key, \
            l.relative_path, l.ordinal, l.title, l.symbol, l.language, l.label, \
            l.dataset_key, l.row_key, l.fields, l.byte_start, l.byte_end, \
            l.token_count, p.term_frequency, g.source_kind, g.authority_class, \
            c.coordinate ",
    // A2 §9's native coordinate, joined rather than stored a second time:
    // `context.lexical_units` is a landed table this module may not alter,
    // and a derived index re-deriving a stored fact is how two copies drift.
    // The family guard is not decoration — a code unit and a document unit
    // can share one path and ordinal (`unit_key`'s own doc), and only the
    // document/mail families read `source.units` rows at all.
    lexical_posting_join!(
        "LEFT JOIN source.unit_coordinates c ON c.generation_id = l.generation_id \
                                            AND c.relative_path = l.relative_path \
                                            AND c.ordinal = l.ordinal \
                                            AND l.family IN ('document', 'mail') "
    ),
    admissible_generations_where!(),
    " AND (? IS NULL OR l.family = ?) AND p.term = ? \
      ORDER BY l.source_name, l.relative_path, l.ordinal, l.unit_key \
      LIMIT ?"
);

/// **Signal 6, inbound.** Every path in one generation holding an edge whose
/// target is the anchor's symbol — i.e. the files that *reference* the
/// anchor. A literal with bound values, like every other statement in this
/// file (item 13's no-client-SQL pin).
const EDGES_TO_TARGET_SQL: &str = "SELECT DISTINCT relative_path FROM source.edges      WHERE generation_id = ? AND target = ? ORDER BY relative_path LIMIT ?";

/// **Signal 6, outbound.** Every symbol the anchor's own file references.
const EDGES_FROM_PATH_SQL: &str = "SELECT DISTINCT target FROM source.edges      WHERE generation_id = ? AND relative_path = ? ORDER BY target LIMIT ?";

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
fn reader_call(format: DatasetFormat) -> Sql {
    match format {
        DatasetFormat::Csv => sql!("read_csv(?, auto_detect = true)"),
        DatasetFormat::Json => sql!("read_json(?, auto_detect = true)"),
        DatasetFormat::Parquet => sql!("read_parquet(?)"),
    }
}

/// `SELECT` list that casts every column to `VARCHAR`, bounded.
///
/// **Every canned dataset query answers in text, and that is a contract, not a
/// convenience.** What gets stored as derived evidence is text, and
/// [`output_hash`] hashes exactly what is stored — so an answer's digest
/// covers the answer a reader will actually see, with no formatting step in
/// between where two builds could disagree about how a `DOUBLE` renders.
fn rows_sql(format: DatasetFormat) -> Sql {
    let mut statement = sql!("SELECT COLUMNS(*)::VARCHAR FROM ");
    statement.extend(&reader_call(format));
    statement.extend(&sql!(" LIMIT ?"));
    statement
}

/// [`DATASET_ROW_COUNT`]'s SQL for one format.
///
/// The count is taken over a *bounded* subquery, so a dataset far larger than
/// the cap costs one capped scan rather than a full one (F12). The caller asks
/// for `cap + 1` and learns from the answer whether the cap bit.
fn row_count_sql(format: DatasetFormat) -> Sql {
    let mut statement = sql!("SELECT count(*)::VARCHAR AS rows FROM (SELECT 1 FROM ");
    statement.extend(&reader_call(format));
    statement.extend(&sql!(" LIMIT ?)"));
    statement
}

/// [`DATASET_COLUMN_PROFILE`]'s SQL for one format.
fn column_profile_sql(format: DatasetFormat) -> Sql {
    let mut statement = sql!(
        "SELECT column_name, count(*)::VARCHAR AS rows, \
         count(value)::VARCHAR AS non_null_rows, \
         count(DISTINCT value)::VARCHAR AS distinct_values \
         FROM (SELECT COLUMNS(*)::VARCHAR FROM "
    );
    statement.extend(&reader_call(format));
    statement.extend(&sql!(
        " LIMIT ?) \
         UNPIVOT (value FOR column_name IN (COLUMNS(*))) \
         GROUP BY column_name ORDER BY column_name"
    ));
    statement
}

/// The SQL for one canned query over one format.
///
/// A `match` over the catalogue rather than a function pointer on
/// [`DatasetQuery`]: the catalogue is a `const`, and a `const` holding
/// function pointers is harder to read than the two arms it would replace.
fn sql_for(query: &DatasetQuery, format: DatasetFormat) -> Sql {
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
pub fn query_identity(query: &DatasetQuery, sql: &Sql) -> String {
    format!(
        "{}/{}#{}",
        query.name,
        query.version,
        blake3::hash(sql.text().as_bytes()).to_hex()
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

/// **Where the compiler, not a scan of this file's text, enforces two of
/// Atlas's boundaries.**
///
/// Both boundaries below have now been re-cut three times, and the first two
/// cuts each shipped a claim one hop defeated. The history is kept because it
/// is the argument for the shape:
///
/// 1. **Text scans.** Structural tests read `db.rs` and looked for forbidden
///    spellings. Defeated by a `format!`-assembled verb the scan's case
///    handling could never match, and by a caller's string laundered through
///    one local rebinding.
/// 2. **`&'static str` types.** [`Sql`] and [`ReadSql`] became newtypes over
///    `&'static str`, and the doc claimed "that absence is the whole
///    guarantee". Defeated by `Box::leak`, which turns any runtime `String`
///    into a `&'static str` — the claim confused *lives for the program's
///    lifetime* with *written as a literal in this crate*. In the same cut,
///    [`ReadSql`]'s `SELECT `-prefix check was defeated by
///    `"SELECT …; DELETE FROM source.generations;"`, because DuckDB executes
///    every statement in a `;`-separated batch.
/// 3. **Compile-time text (this one).** The types are built from a
///    [`SqlText`] implementor's associated **const**, so the text is produced
///    by const evaluation — which has no heap, no caller, and no running
///    program to read data out of. `Box::leak` is not a `const fn`; naming a
///    local in a const is E0435. And [`ReadSql`] additionally refuses any
///    `;`.
///
/// The scans remain as a cheap second net over the shapes a type cannot see
/// (`tests/w1b_overlay_lifecycle_trigger.rs` and
/// `tests/x5_a1a_acceptance.rs`, whose docs name their own blind spots).
///
/// This is a **child module on purpose (R4 — the language's own privacy is
/// the mechanism)**, not a sibling file. A private field is private to its
/// defining module and its descendants, never to its parent, so nothing in
/// the rest of `db.rs` can name [`Store::conn`] or the inside of a [`Sql`] —
/// while `db.rs` remains the single file naming the database driver, which
/// `tests/x1_atlas_substrate.rs`'s `atlas_database_has_exactly_one_owner`
/// requires and which a second file would break.
///
/// Privacy alone is **not** what carries the guarantee, and it is worth
/// saying why, because it is the obvious design and it does not work: the
/// macros below expand at their *call sites*, in `db.rs`, which is the
/// module's parent. A constructor private to `store` would be unreachable
/// from the macro too. Const evaluation is the mechanism precisely because it
/// does not depend on where the code is written.
///
/// # What each type buys
///
/// * [`SqlText`] — the mechanism. Text as an associated `const`, so it is the
///   compiler that produces it.
/// * [`Sql`] — a statement this crate wrote. Its only constructor is
///   `Sql::of::<T: SqlText>()`, which takes no string: there is no
///   `&str`, `String`, **or `&'static str`** route in. A1a §17 item 13
///   ("no client SQL reaches the store") is therefore a property of the type
///   system here.
/// * [`Name`] — the same, for the table/schema names DuckDB's appender takes.
///   Not a statement, but a leaked one would still choose which table gets
///   rows.
/// * [`Store`] / [`StoreTx`] — the only statement-running surfaces `db.rs`
///   has. They take `impl Into<Sql>`, and they wrap the driver's
///   `Connection`/`Transaction` rather than deref to them, so no code outside
///   this module can hand the driver a `&str` at all.
/// * [`ReadSql`] — one bare `SELECT`, `;`-free, checked during compilation.
/// * [`ReadOnly`] — a handle that **cannot write**, because it exposes no
///   write call and hands out no `Statement`, no `Appender`, no
///   `Transaction`, and no `Connection`. `impl Admissible` holds one of these
///   and nothing else, so H13.2's "the admissibility filter cannot write" is
///   a type error to violate rather than a string absent from a scan.
///
/// # What these types do NOT stop
///
/// Stated because an unstated limit is how a guard becomes a false claim, and
/// every item here was **attempted** during the S5 closeout rather than
/// reasoned about:
///
/// * **Text this crate's own build reads.** `include_str!` and `env!` are
///   const-evaluable, so a file on the build machine can become a statement.
///   Attempted and it compiles. What cannot get in is anything a *running*
///   Sergeant is handed, which is what item 13 is about.
/// * **A hand-written `impl SqlText` handed to [`ReadSql::of`] directly,
///   carrying a write.** The check for that path is a generic associated
///   const, so it fires at monomorphization: attempted, and `cargo check` —
///   and `cargo build --lib` of a `pub fn` nothing calls — let it through,
///   while the moment a test actually called it the build failed with
///   `evaluation panicked: a read-only handle may only run one statement`.
///   Dead code can hold a bad statement; code that runs cannot. Every call
///   site in this file uses [`read_sql!`], whose check is a non-generic
///   `const` item and therefore fails `cargo check`.
/// * **Opening a *new* `Connection` inside this file**, which bypasses every
///   handle above. The previous version of this list said "DuckDB's own file
///   locking" stood against that; **it does not** — measured in the closeout,
///   a second `Connection::open` on the same file from the same process
///   succeeded and its `DELETE` returned `Ok`. What stands against it is the
///   second net, which now requires every `Connection::open*` in this file to
///   be wrapped in `Store::new(…)` on the spot, and the admissibility scan,
///   which forbids `Connection::` anywhere the filter can reach.
/// * **`unsafe` inside this module**, or a `#[cfg(test)]` shim. Attempted via
///   a const `transmute` to a `&'static str`: rejected, but at *build* rather
///   than at `cargo check`, and only because the value was invalid — a const
///   has no way to obtain a runtime address in the first place. The module is
///   small enough to read in one sitting, which is the point of keeping it
///   small.
/// * **What a `SELECT` may read.** [`ReadSql`] bounds writes, not reach:
///   DuckDB's file-reading table functions are still spellable in one. What
///   keeps that from being a caller's choice is [`SqlText`].
pub(crate) mod store {
    use duckdb::{Appender, CachedStatement, Connection, Statement, ToSql, Transaction};

    /// Text this crate wrote, carried as a **compile-time constant**.
    ///
    /// This trait is the mechanism behind [`Sql`], [`ReadSql`] and [`Name`],
    /// and it exists because the rule it replaced did not hold. Until the S5
    /// closeout all three were built from `&'static str`, and the doc here
    /// claimed that "no caller's value can be among them". That was a
    /// conceptual error, not a gap in coverage: `&'static str` means *lives
    /// as long as the program*, **not** *written as a literal in this crate*.
    /// `Box::leak` and `String::leak` turn any runtime `String` — a caller's,
    /// verbatim — into a `&'static str`, and the closeout landed exactly
    /// that: a method doing
    /// `execute_batch(Box::leak(user_text.to_string().into_boxed_str()))`
    /// compiled clean and emptied a table.
    ///
    /// `TEXT` is an associated **const**, and that is the barrier. A const
    /// initializer is evaluated by the compiler, with no program running: no
    /// heap, no caller, no way to observe runtime data. `Box::leak` is not a
    /// `const fn`, so writing it there is E0015 ("cannot call non-const
    /// function"); naming a local is E0435 ("attempt to use a non-constant
    /// value in a constant"). Both are compile errors at `cargo check`.
    ///
    /// The barrier is const evaluation, **not** a lifetime and not a macro
    /// fragment specifier. That is why [`sql!`] can take `$text:expr` and
    /// still be safe — and why it has to: the DDL constants it wraps
    /// (`HARDENING_DDL`, `SCHEMA_DDL`) are paths, not literal tokens, so a
    /// `$text:literal` arm would reject them while adding no safety the
    /// `const` does not already provide.
    ///
    /// # What this does NOT prove
    ///
    /// * That the text is valid SQL, or that it is a read. [`ReadSql`] adds
    ///   the second of those, separately, and nothing here adds the first.
    /// * That the text was written in this file. "Compile time" includes the
    ///   build environment: `include_str!` and `env!` are const-evaluable, so
    ///   text this crate's own **build** reads off disk or out of the
    ///   environment can become a statement. What cannot is anything a
    ///   *running* Sergeant is handed — which is what A1a item 13 asks for.
    /// * Anything about a table name reaching DuckDB's appender by a route
    ///   other than [`Name`]; see the module doc's blind-spot list.
    pub trait SqlText {
        const TEXT: &'static str;
    }

    /// A statement **this crate** wrote.
    ///
    /// The only constructor is [`Sql::of`], and it takes no string at all —
    /// the text arrives as a [`SqlText`] implementor's associated const.
    /// There is deliberately no constructor taking `&str`, `String`, **or
    /// `&'static str`**: that last one is the hole this replaced, not a
    /// stricter spelling of it. Making the field `pub`, or adding a
    /// string-taking constructor of any name, is the removal of A1a item 13's
    /// enforcement rather than a refactor.
    #[derive(Clone)]
    pub struct Sql(String);

    impl Sql {
        /// The one constructor. Call it as `sql!("…")`.
        pub fn of<T: SqlText>() -> Self {
            Self(T::TEXT.to_string())
        }

        /// Append another vetted statement — two `Sql`s concatenate, and
        /// there is no third thing that can join one.
        ///
        /// This is what replaced `from_parts(&[&'static str])`. Assembling a
        /// statement out of `&'static str` pieces meant every piece was one
        /// `Box::leak` away from being a caller's, which made the assembled
        /// whole exactly as weak as the constructor above.
        pub fn extend(&mut self, more: &Sql) {
            self.0.push_str(&more.0);
        }

        /// The statement text, for hashing and for handing to the driver.
        pub fn text(&self) -> &str {
            &self.0
        }
    }

    /// A table or schema **name** this crate wrote.
    ///
    /// Not a statement: DuckDB's appender takes a name through its own C API
    /// rather than interpolating one into SQL, so a name cannot carry an
    /// injection. It is [`SqlText`]-constructed for the plainer reason that a
    /// leaked name would still let a caller choose *which* table rows get
    /// appended to.
    pub struct Name(&'static str);

    impl Name {
        /// The one constructor. Call it as `name!("…")`.
        pub fn of<T: SqlText>() -> Self {
            Self(T::TEXT)
        }

        pub fn text(&self) -> &'static str {
            self.0
        }
    }

    impl From<&Sql> for Sql {
        fn from(sql: &Sql) -> Self {
            sql.clone()
        }
    }

    /// The narrowed statement surface, shared by [`Store`] and [`StoreTx`]
    /// so a helper that runs inside a transaction is written once.
    ///
    /// Every method takes `impl Into<Sql>` — the whole point. A helper
    /// generic over this trait therefore cannot be handed a caller's `&str`
    /// either, wherever it is called from.
    pub trait Statements {
        fn prepare<S: Into<Sql>>(&self, sql: S) -> Result<Statement<'_>, duckdb::Error>;
        fn prepare_cached<S: Into<Sql>>(
            &self,
            sql: S,
        ) -> Result<CachedStatement<'_>, duckdb::Error>;
        fn execute<S: Into<Sql>>(
            &self,
            sql: S,
            params: &[&dyn ToSql],
        ) -> Result<usize, duckdb::Error>;
        fn execute_batch<S: Into<Sql>>(&self, sql: S) -> Result<(), duckdb::Error>;
        fn appender_to_db(&self, table: Name, schema: Name) -> Result<Appender<'_>, duckdb::Error>;
    }

    /// Atlas's connection, with every statement surface narrowed to [`Sql`].
    pub struct Store {
        conn: Connection,
    }

    impl Store {
        pub fn new(conn: Connection) -> Self {
            Self { conn }
        }

        /// A handle onto this same connection that cannot write.
        pub fn reader(&self) -> ReadOnly<'_> {
            ReadOnly { conn: &self.conn }
        }

        pub fn set_statement_cache_capacity(&self, capacity: usize) {
            self.conn.set_prepared_statement_cache_capacity(capacity);
        }

        /// A second handle onto the **same database instance**, never a
        /// second instance (`Connection::try_clone`'s own contract).
        pub fn try_clone(&self) -> Result<Self, duckdb::Error> {
            Ok(Self {
                conn: self.conn.try_clone()?,
            })
        }

        pub fn transaction(&mut self) -> Result<StoreTx<'_>, duckdb::Error> {
            Ok(StoreTx {
                tx: self.conn.transaction()?,
            })
        }

        /// A snapshot-isolated read transaction from a shared borrow —
        /// `Transaction::new_unchecked`, kept behind this wrapper so the one
        /// caller that needs it (`fused_search`) still cannot reach a raw
        /// `Connection` through the value it gets back.
        pub fn snapshot(&self) -> Result<StoreTx<'_>, duckdb::Error> {
            Ok(StoreTx {
                tx: Transaction::new_unchecked(&self.conn)?,
            })
        }
    }

    /// A transaction with the same narrowing [`Store`] applies.
    ///
    /// It **wraps** rather than derefs: `duckdb::Transaction` derefs to
    /// `Connection`, and a deref here would hand every caller the raw `&str`
    /// surface back through one extra dot.
    pub struct StoreTx<'conn> {
        tx: Transaction<'conn>,
    }

    impl Statements for Store {
        fn prepare<S: Into<Sql>>(&self, sql: S) -> Result<Statement<'_>, duckdb::Error> {
            self.conn.prepare(sql.into().text())
        }

        fn prepare_cached<S: Into<Sql>>(
            &self,
            sql: S,
        ) -> Result<CachedStatement<'_>, duckdb::Error> {
            self.conn.prepare_cached(sql.into().text())
        }

        fn execute<S: Into<Sql>>(
            &self,
            sql: S,
            params: &[&dyn ToSql],
        ) -> Result<usize, duckdb::Error> {
            self.conn.execute(sql.into().text(), params)
        }

        fn execute_batch<S: Into<Sql>>(&self, sql: S) -> Result<(), duckdb::Error> {
            self.conn.execute_batch(sql.into().text())
        }

        /// A table name is not a statement, and the driver does not build
        /// one out of it — but a leaked name would still choose which table
        /// gets rows, so it is a [`Name`] for that reason.
        fn appender_to_db(&self, table: Name, schema: Name) -> Result<Appender<'_>, duckdb::Error> {
            self.conn.appender_to_db(table.text(), schema.text())
        }
    }

    impl Statements for StoreTx<'_> {
        fn prepare<S: Into<Sql>>(&self, sql: S) -> Result<Statement<'_>, duckdb::Error> {
            self.tx.prepare(sql.into().text())
        }

        fn prepare_cached<S: Into<Sql>>(
            &self,
            sql: S,
        ) -> Result<CachedStatement<'_>, duckdb::Error> {
            self.tx.prepare_cached(sql.into().text())
        }

        fn execute<S: Into<Sql>>(
            &self,
            sql: S,
            params: &[&dyn ToSql],
        ) -> Result<usize, duckdb::Error> {
            self.tx.execute(sql.into().text(), params)
        }

        fn execute_batch<S: Into<Sql>>(&self, sql: S) -> Result<(), duckdb::Error> {
            self.tx.execute_batch(sql.into().text())
        }

        fn appender_to_db(&self, table: Name, schema: Name) -> Result<Appender<'_>, duckdb::Error> {
            self.tx.appender_to_db(table.text(), schema.text())
        }
    }

    impl StoreTx<'_> {
        pub fn commit(self) -> Result<(), duckdb::Error> {
            self.tx.commit()
        }

        pub fn rollback(self) -> Result<(), duckdb::Error> {
            self.tx.rollback()
        }
    }

    /// A statement that **reads**: one bare `SELECT`, containing no `;`.
    ///
    /// [`ReadOnly`] takes one of these rather than a [`Sql`], and the
    /// difference is a hop that used to be open. `ReadOnly` exposes no write
    /// *call* — but DuckDB runs whatever statement it is handed, so
    /// `prepare("DELETE …")` followed by `query()` writes, and a `Sql`
    /// holding a `DELETE` would have been accepted. Checked live in the S5
    /// closeout: that exact hop compiled, before this type existed.
    ///
    /// # Two conditions, and why the second one is here
    ///
    /// * **Begins `SELECT `.** Deliberately narrow — the statement's first
    ///   seven bytes — because a narrow check that holds is worth more than a
    ///   verb blacklist that a spelling walks past. A `WITH`-prefixed CTE is
    ///   refused too, and widening this to admit one means widening it
    ///   deliberately, in the one place the rule lives.
    /// * **Contains no `;` at all.** The prefix check *alone* was defeated in
    ///   the S5 closeout by
    ///   `"SELECT … LIMIT 1; DELETE FROM source.generations;"`: the prefix
    ///   sees only the leading `SELECT `, and **DuckDB executes every
    ///   statement in a `;`-separated batch**. That was measured on this
    ///   duckdb (1.10505.0), not taken on faith:
    ///
    ///   | probe | result |
    ///   |---|---|
    ///   | `prepare(batch)` alone, never queried | nothing runs; 3 rows stay 3 |
    ///   | `prepare(batch)` + `query([])` | **every** statement runs; 3 rows → 0 |
    ///   | same via `prepare_cached` | same; 3 rows → 0 |
    ///   | `query` on a 2-statement batch | returns the **last** statement's result set |
    ///   | batch containing a `?` bind | refused at `prepare`, "Values were not provided…" |
    ///
    ///   The last row is worth stating because it narrows the closeout's own
    ///   report: the `read_sql!(concat!("SELECT … LIMIT ?; ", "DEL", "ETE …"))`
    ///   form errors at `prepare` rather than deleting. The *unparameterised*
    ///   form deletes for real, which is enough — and is why the rule is
    ///   about `;`, not about binds.
    ///
    ///   `;`-free rather than "at most one trailing `;`" because no call site
    ///   needs a trailing one, and because "trailing" is a claim about
    ///   parsing that a byte scan is not entitled to make.
    ///
    /// # What this does NOT stop — stated, not implied
    ///
    /// * It is **stricter than SQL**: `SELECT ';' FROM t` is one harmless
    ///   statement (measured: a `;` inside a quoted literal does not split
    ///   anything) and is refused anyway. No call site needs that; one that
    ///   does must change the rule here, in the open.
    /// * A `SELECT` still *reads* whatever it names, DuckDB's file-reading
    ///   table functions (`read_csv`, `read_parquet`) included. This type
    ///   bounds **writes**, not reach. What keeps that reach from being a
    ///   caller's choice is [`SqlText`], not this check.
    /// * It says nothing about what the surrounding code does with the rows.
    ///
    /// Build one with [`read_sql!`], which puts the check in a non-generic
    /// `const` item so a bad statement is a **`cargo check` failure**.
    pub struct ReadSql(&'static str);

    impl ReadSql {
        /// The one constructor.
        ///
        /// Two checks stand behind it, and they fail at different times —
        /// worth knowing exactly, because "compile time" is not one thing:
        ///
        /// * [`read_sql!`] emits a **non-generic** `const` item asserting
        ///   [`is_read_statement`]. That is evaluated eagerly, so a bad
        ///   statement written at a call site fails `cargo check`. Every call
        ///   site in this file takes that path.
        /// * A hand-written `impl SqlText` handed here instead trips
        ///   `Check::<T>::OK` below. It is a **generic** associated const,
        ///   evaluated at monomorphization: it fails `cargo build`, `cargo
        ///   test` and CI, but is not guaranteed to fail a bare `cargo
        ///   check`. That gap is why the macro carries its own copy rather
        ///   than relying on this one.
        ///
        /// There is deliberately no runtime panic here. Every route into this
        /// constructor is checked before a binary exists, and a runtime guard
        /// would read as though one were not.
        pub fn of<T: SqlText>() -> Self {
            struct Check<T: SqlText>(core::marker::PhantomData<T>);
            impl<T: SqlText> Check<T> {
                const OK: () = assert!(
                    is_read_statement(T::TEXT),
                    "a read-only handle may only run one statement, beginning `SELECT ` and \
                     containing no `;`"
                );
            }
            // Forces the assertion above to be evaluated for this `T`.
            #[allow(clippy::let_unit_value)]
            let () = Check::<T>::OK;
            Self(T::TEXT)
        }
    }

    /// One bare read: begins `SELECT `, and carries no statement separator.
    ///
    /// `const` on purpose — the whole point is that it runs during
    /// compilation. It also sees the **assembled** string, after `concat!`
    /// has resolved, which is why it is immune to the verb-splitting that
    /// defeats a source-text scan: `concat!("DEL", "ETE FROM …")` is already
    /// `DELETE FROM …` by the time this function sees it, and there is no
    /// spelling of a write that arrives here looking like something else.
    /// That is the reason to trust this check and not the scan.
    pub const fn is_read_statement(sql: &str) -> bool {
        let bytes = sql.as_bytes();
        let want = b"SELECT ";
        if bytes.len() < want.len() {
            return false;
        }
        let mut i = 0;
        while i < want.len() {
            if bytes[i] != want[i] {
                return false;
            }
            i += 1;
        }
        while i < bytes.len() {
            if bytes[i] == b';' {
                return false;
            }
            i += 1;
        }
        true
    }

    /// A read-only handle onto an Atlas connection.
    ///
    /// The type H13.2's "`sgt search` is a pure reader" is enforced by. It
    /// exposes exactly two operations, both of which run a statement and
    /// return owned rows; it hands out no `Statement` (whose `execute` takes
    /// `&mut self` and could run anything prepared), no `Appender`, no
    /// `Transaction`, and no `Connection`. A caller holding one of these has
    /// **no expressible way** to write, whatever it does with it.
    ///
    /// The `&Row` a mapping closure receives can reach `&Statement` through
    /// the driver's `AsRef`, and that is checked, not overlooked: every
    /// writing method on `Statement` takes `&mut self`, so a shared reference
    /// to one runs nothing.
    pub struct ReadOnly<'conn> {
        conn: &'conn Connection,
    }

    impl ReadOnly<'_> {
        /// Run one statement and map every row.
        pub fn rows<T, E, F>(
            &self,
            sql: ReadSql,
            params: &[&dyn ToSql],
            mut map: F,
        ) -> Result<Vec<T>, E>
        where
            E: From<duckdb::Error>,
            F: FnMut(&duckdb::Row<'_>) -> Result<T, E>,
        {
            let mut statement = self.conn.prepare(sql.0)?;
            let mut rows = statement.query(params)?;
            let mut out = Vec::new();
            while let Some(row) = rows.next()? {
                out.push(map(row)?);
            }
            Ok(out)
        }

        /// Run one statement and map its first row, if any.
        pub fn first<T, E, F>(
            &self,
            sql: ReadSql,
            params: &[&dyn ToSql],
            map: F,
        ) -> Result<Option<T>, E>
        where
            E: From<duckdb::Error>,
            F: FnMut(&duckdb::Row<'_>) -> Result<T, E>,
        {
            Ok(self.rows(sql, params, map)?.into_iter().next())
        }
    }
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
fn bootstrap_atlas_ddl(conn: &impl Statements) -> Result<(), duckdb::Error> {
    conn.execute_batch(sql!(HARDENING_DDL))?;
    conn.execute_batch(sql!(SCHEMA_DDL))?;
    conn.execute_batch(sql!(TABLE_DDL))?;
    Ok(())
}

/// Atlas's database over one data dir.
///
/// Owns its connection privately; nothing hands a [`Connection`] out. Values
/// cross this boundary as plain Rust, the same rule the operations
/// projection holds for its own file.
pub struct AtlasDb {
    conn: Store,
    path: PathBuf,
    /// A2 §6's model, loaded **at most once per handle** and only when a
    /// query first needs it.
    ///
    /// Not a per-call [`crate::runtime::atlas::semantic::installed_model`]:
    /// that reads 32 MB of weights and parses a 1 MB tokenizer, which is a
    /// thing to do once, not once per search. `OnceLock` rather than a
    /// process-wide `static` on purpose — the asset directory is resolved
    /// from `$SGT_SEMANTIC_MODEL_DIR`/the executable's directory, so two
    /// handles in one process (a test suite's) must be able to disagree
    /// about what is installed instead of racing to cache the first answer
    /// for everyone.
    semantic: OnceLock<Option<SemanticEngine>>,
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
        let conn = Store::new(Connection::open(&path)?);
        Self::over(conn, path)
    }

    /// An in-memory Atlas database, for callers that want the namespaces
    /// without a file (tests, and any read-only rendering).
    pub fn open_in_memory() -> Result<Self, AtlasError> {
        let conn = Store::new(Connection::open_in_memory()?);
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
        let conn = Store::new(Connection::open_with_flags(&path, config)?);
        conn.set_statement_cache_capacity(STATEMENT_CACHE);
        // F4's network-hardening settings only — no `SCHEMA_DDL`, no
        // `TABLE_DDL`. Both are genuine DDL and a read-only connection
        // cannot run them even under `IF NOT EXISTS`; skipping them here
        // rather than letting DuckDB refuse them is what keeps this path a
        // read, not a read that happens to trip over a write guard.
        conn.execute_batch(sql!(HARDENING_DDL))?;
        Ok(Self {
            conn,
            path,
            semantic: OnceLock::new(),
        })
    }

    fn over(conn: Store, path: PathBuf) -> Result<Self, AtlasError> {
        conn.set_statement_cache_capacity(STATEMENT_CACHE);
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
        Ok(Self {
            conn,
            path,
            semantic: OnceLock::new(),
        })
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT schema_name FROM duckdb_schemas() \
             WHERE database_name = current_database() AND NOT internal \
             ORDER BY schema_name"
        ))?;
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
    pub fn stage_scan(
        &mut self,
        scan: &SourceScan,
        estate: &EstateBinding,
    ) -> Result<ScanCommit, AtlasError> {
        self.stage_scan_impl(scan, estate, None)
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
        estate: &EstateBinding,
        provenance: &ExternalGitProvenance,
    ) -> Result<ScanCommit, AtlasError> {
        self.stage_scan_impl(scan, estate, Some(provenance))
    }

    fn stage_scan_impl(
        &mut self,
        scan: &SourceScan,
        estate: &EstateBinding,
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
            bind_generation_estate(&self.conn, &current.id, estate)?;
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
            // **The two-estates-one-world case.** A generation is reached by
            // content key, so a second estate that declares a source whose
            // bytes hash identically gets `Unchanged` and stages nothing —
            // and would then have no binding row of its own, leaving it
            // unable to see a world it legitimately indexed. So the binding
            // is recorded here too, which is why
            // `source.generation_estates` is many-rows-per-generation rather
            // than a column on `source.generations`: two estates can have
            // observed the same world, and each one's claim on it is its
            // own row. It is not a widening — an estate only ever gets a row
            // for a generation its own scan actually produced or matched.
            bind_generation_estate(&self.conn, &current.id, estate)?;
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
        // S5 W7 (F-SF-01): a container child dataset carries its own bytes and
        // is materialised under the daemon's own data directory for the
        // length of its read — `self.path` is `<data-dir>/atlas.duckdb`, so
        // its parent is that directory.
        let scratch_root = self
            .path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(CHILD_DATASET_SCRATCH);
        let reads: Vec<IngestedDataset> = scan
            .datasets
            .iter()
            .map(|dataset| read_dataset(&self.conn, scan, dataset, &scratch_root))
            .collect();
        let tx = self.conn.transaction()?;
        tx.execute(
            sql!(
                "INSERT INTO source.generations \
             (generation_id, source_name, source_kind, authority_class, content_key, \
              observed_at, state, summary_event_id, extractors) \
             VALUES (?, ?, ?, ?, ?, ?, ?, NULL, ?)"
            ),
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
        // S6 D1 — A2 §2 stage 1's estate coordinate, written inside the SAME
        // staging transaction as the generation row it describes, for the
        // reason `git.provenance` is (module doc): a binding written as a
        // follow-up could be lost while the generation survived, and a
        // generation with no binding row is inadmissible from every estate.
        // "Half-recorded" would therefore read as "indexed but invisible",
        // which is the silent partial this store refuses everywhere else.
        bind_generation_estate(&tx, &generation_id, estate)?;
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
            tx.prepare_cached(sql!(
                "INSERT INTO source.symbols \
                 (generation_id, source_name, language, label, name, occurrences) \
                 VALUES (?, ?, ?, ?, ?, ?)"
            ))?
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
                sql!(
                    "INSERT INTO git.provenance \
                 (generation_id, source_name, origin, requested_ref, resolved_commit, \
                  retrieved_at) \
                 VALUES (?, ?, ?, ?, ?, ?)"
                ),
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
        // S5 W2: the lexical index, derived inside the same transaction from
        // the rows just written. Not from `scan` in memory — from the stored
        // rows themselves, so the postings are an index over exactly the
        // evidence a hit will cite, and there is no second derivation path
        // that could disagree with the first (A2-02: no independent chunk
        // universe). All-or-nothing with everything else the scan staged.
        index_generation(&tx, &generation_id)?;
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
            sql!(
                "UPDATE source.generations SET state = ?, summary_event_id = ? \
             WHERE generation_id = ? AND state = ?"
            ),
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT generation_id, source_name FROM source.generations \
             WHERE state != ? AND source_name >= ? AND source_name < ? \
             ORDER BY observed_at DESC, generation_id DESC LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT generation_id, source_name, source_kind, authority_class, content_key, \
                    observed_at \
             FROM source.generations WHERE source_name = ? AND state = ? \
             ORDER BY observed_at DESC, generation_id DESC LIMIT 1"
        ))?;
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

    /// `source.files`' recorded content hash for one path in one
    /// generation, when that generation has a row for it at all (F-SF-01:
    /// [`Self::rerank_signals`]'s only use — telling a Work-changed path
    /// from a merely-visible one by comparing this against the base
    /// generation's hash for the same path).
    fn file_content_hash(
        &self,
        generation_id: &str,
        relative_path: &str,
    ) -> Result<Option<String>, AtlasError> {
        let mut statement = self.conn.prepare(sql!(
            "SELECT content_hash FROM source.files \
             WHERE generation_id = ? AND relative_path = ?"
        ))?;
        let mut rows = statement.query(duckdb::params![generation_id, relative_path])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// Units of one source's confirmed generation, in path then ordinal
    /// order, bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn units(&self, source_name: &str, limit: usize) -> Result<Vec<StoredUnit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(sql!(
            "SELECT u.relative_path, u.local_key, u.ordinal, u.unit_kind, u.heading_level, \
                    u.title, u.byte_start, u.byte_end, u.body \
             FROM source.units u JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY u.relative_path, u.ordinal LIMIT ?"
        ))?;
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

    /// **One exact unit, by its pinned coordinate** — C1 §21 item 4's
    /// *"Referenced coordinates resolve **without broad rediscovery**"*, as a
    /// keyed lookup.
    ///
    /// Every predicate is an equality on the coordinate a
    /// [`crate::runtime::context::EvidenceCoordinate`] already carries:
    /// `(generation_id, relative_path, ordinal)` is `source.units`' own
    /// identity, so this reads one row and never ranks, scans a corpus, or
    /// consults `context.lexical_units`. That is the difference item 4 is
    /// about: [`Self::units`] lists a generation and [`Self::lexical_search`]
    /// *searches* it; this resolves a pointer someone was already handed.
    ///
    /// Keyed on `generation_id` and **not** on `state = confirmed`, unlike
    /// [`Self::units`]: a snapshot pins an exact generation precisely so it
    /// re-resolves later, and a pin that stopped resolving the moment a newer
    /// generation was confirmed would be a description rather than a pin
    /// (§15). Nothing widens by it — a generation id only ever reaches a
    /// caller through the admissibility filter that admitted it.
    /// # Why it answers with the resource's provenance too (C1 §21 item 8)
    ///
    /// *"document/mail/OCR excerpts preserve original resource/native/
    /// extractor provenance"* is a claim about the **excerpt in the prompt**,
    /// and the two facts a prompt excerpt needs beyond the unit's own row do
    /// not live in `source.units`: the extractor identity that produced it is
    /// `source.files.extractor`, and A2 §9's native coordinate is
    /// `source.unit_coordinates.coordinate`. Both are joined here, in the one
    /// statement, rather than fetched by a second keyed read — the same two
    /// joins [`indexable_units`] already writes (**R2**), so a resolved unit
    /// and an indexed one describe the same provenance by construction.
    ///
    /// Both joins are `LEFT`: a unit whose resource row is missing would be a
    /// real defect, and this answers with what it does have rather than
    /// dropping the row and hiding it (the argument [`Self::child_resources`]
    /// already makes). A Markdown unit legitimately has no native coordinate
    /// — its byte span *is* its address — so `None` there is ordinary.
    pub fn resolve_unit(
        &self,
        generation_id: &str,
        relative_path: &str,
        ordinal: u64,
    ) -> Result<Option<ResolvedUnit>, AtlasError> {
        let mut statement = self.conn.prepare(sql!(
            "SELECT u.relative_path, u.local_key, u.ordinal, u.unit_kind, u.heading_level, \
                    u.title, u.byte_start, u.byte_end, u.body, f.extractor, c.coordinate \
             FROM source.units u \
             LEFT JOIN source.files f ON f.generation_id = u.generation_id \
                                     AND f.relative_path = u.relative_path \
             LEFT JOIN source.unit_coordinates c ON c.generation_id = u.generation_id \
                                                AND c.relative_path = u.relative_path \
                                                AND c.ordinal = u.ordinal \
             WHERE u.generation_id = ? AND u.relative_path = ? AND u.ordinal = ?"
        ))?;
        let mut rows = statement.query(duckdb::params![
            generation_id,
            relative_path,
            ordinal as i64
        ])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let kind: String = row.get(3)?;
        Ok(Some(ResolvedUnit {
            unit: StoredUnit {
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
            },
            extractor: row.get(9)?,
            native_coordinate: row.get(10)?,
        }))
    }

    /// **One exact stored query result, by its pinned coordinate** — C1 §10's
    /// *"a deterministic data result is itself derived evidence and should be
    /// addressable/pinnable"*, resolved the same keyed way
    /// [`Self::resolve_unit`] is.
    ///
    /// `(generation_id, relative_path, query_name)` is
    /// `source.dataset_facts`' own identity: one canned query's answer about
    /// one dataset of one generation. Every predicate is an equality; nothing
    /// here ranks, scans a corpus, or takes a string that could become SQL —
    /// `query_name` selects a *stored row* by the name of the canned query
    /// that produced it, and that query's statement was an associated `const`
    /// chosen by [`sql_for`] from the fixed [`DATASET_QUERIES`] catalogue.
    ///
    /// It is spelled `query_name` rather than `query` deliberately:
    /// `tests/x5_a1a_acceptance.rs::a1a_item_13_no_client_sql_reaches_the_store`
    /// reads Atlas's public signatures and refuses a reader that takes
    /// `query: &str`, because that is exactly the shape an SQL-taking entry
    /// point wears — and a guard that made an exception for this one would
    /// stop being a guard.
    pub fn resolve_dataset_fact(
        &self,
        generation_id: &str,
        relative_path: &str,
        query_name: &str,
    ) -> Result<Option<DatasetFact>, AtlasError> {
        let mut statement = self.conn.prepare(sql!(
            "SELECT f.relative_path, f.dataset_key, f.query, f.query_identity, f.row_limit, \
                    f.truncated, f.columns, f.rows, f.output_hash \
             FROM source.dataset_facts f \
             WHERE f.generation_id = ? AND f.relative_path = ? AND f.query = ?"
        ))?;
        let mut rows =
            statement.query(duckdb::params![generation_id, relative_path, query_name])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(DatasetFact {
            relative_path: row.get(0)?,
            dataset_key: row.get(1)?,
            query: row.get(2)?,
            query_identity: row.get(3)?,
            row_limit: row.get::<usize, i64>(4)? as u64,
            truncated: row.get(5)?,
            columns: split_names(&row.get::<usize, String>(6)?),
            rows: parse_rows(&row.get::<usize, String>(7)?),
            output_hash: row.get(8)?,
        }))
    }

    /// **One exact stored relationship, by its pinned coordinate** — item 4's
    /// other coordinate shape, resolved the same keyed way as
    /// [`Self::resolve_unit`].
    ///
    /// `kind` picks the table because the two relationships C1 §5 contributes
    /// are stored in two: a container/document/mail parent-child row lives in
    /// `source.child_resources` and every syntax-derived edge lives in
    /// `source.edges`. Both lookups are equalities on the coordinate's own
    /// fields; neither scans a generation.
    ///
    /// `ordinal` (F-IN-01) discriminates two edges that legitimately share
    /// every other field — e.g. the same file importing the same target
    /// twice — which `(generation_id, edge_kind, relative_path, target)`
    /// alone cannot: without it, `rows.next()` took whichever matching row
    /// DuckDB returned first, non-deterministically. `None` widens back to
    /// the old any-match behavior, which is correct for the child-resource
    /// branch (no ordinal column) and for a coordinate pinned before this
    /// field existed.
    pub fn resolve_relationship(
        &self,
        generation_id: &str,
        kind: &str,
        from: &str,
        to: &str,
        ordinal: Option<u64>,
    ) -> Result<Option<ResolvedRelationship>, AtlasError> {
        if kind == CHILD_RESOURCE_RELATIONSHIP {
            let mut statement = self.conn.prepare(sql!(
                "SELECT c.parent_relative_path, c.relative_path, c.entry_path \
                 FROM source.child_resources c \
                 WHERE c.generation_id = ? AND c.parent_relative_path = ? \
                   AND c.relative_path = ?"
            ))?;
            let mut rows = statement.query(duckdb::params![generation_id, from, to])?;
            let Some(row) = rows.next()? else {
                return Ok(None);
            };
            return Ok(Some(ResolvedRelationship {
                kind: CHILD_RESOURCE_RELATIONSHIP.to_string(),
                from: row.get(0)?,
                to: row.get(1)?,
                ordinal: None,
                detail: row.get::<usize, Option<String>>(2)?,
            }));
        }
        let ordinal_param = ordinal.map(|o| o as i64);
        let mut statement = self.conn.prepare(sql!(
            "SELECT e.relative_path, e.target, e.ordinal, e.language \
             FROM source.edges e \
             WHERE e.generation_id = ? AND e.edge_kind = ? AND e.relative_path = ? \
               AND e.target = ? AND (? IS NULL OR e.ordinal = ?)"
        ))?;
        let mut rows = statement.query(duckdb::params![
            generation_id,
            kind,
            from,
            to,
            ordinal_param,
            ordinal_param
        ])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(ResolvedRelationship {
            kind: kind.to_string(),
            from: row.get(0)?,
            to: row.get(1)?,
            ordinal: Some(row.get::<usize, i64>(2)? as u64),
            detail: row.get::<usize, Option<String>>(3)?,
        }))
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT s.language, s.label, s.name, s.occurrences \
             FROM source.symbols s JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY s.language, s.label, s.name LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT o.relative_path, o.syntax_key, o.extractor, o.language, o.ordinal, \
                    o.label, o.name, o.byte_start, o.byte_end \
             FROM source.occurrences o JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY o.relative_path, o.ordinal LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT e.relative_path, e.syntax_key, e.extractor, e.language, e.ordinal, \
                    e.edge_kind, e.target, e.byte_start, e.byte_end \
             FROM source.edges e JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY e.relative_path, e.ordinal LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT c.generation_id, c.path, c.status, c.detail, c.bytes, c.observed_at \
             FROM meta.coverage c JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state IN (?, ?) \
             ORDER BY c.observed_at DESC, c.path NULLS FIRST LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT c.status, count(*) FROM meta.coverage c \
             JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? GROUP BY c.status ORDER BY c.status"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT current_setting('autoinstall_known_extensions')::BOOLEAN, \
                    current_setting('autoload_known_extensions')::BOOLEAN, \
                    current_setting('allow_community_extensions')::BOOLEAN, \
                    current_setting('lock_configuration')::BOOLEAN"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT extension_name FROM duckdb_extensions() \
             WHERE loaded AND install_mode = 'STATICALLY_LINKED' \
             ORDER BY extension_name LIMIT ?"
        ))?;
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
            // A probe names a path the caller already has; it never carries
            // bytes and is never a container child.
            content: None,
            parent: None,
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

    /// Container children of one source's confirmed generation, with A1
    /// §6.6's four preserved fields, in path order, bounded by `limit`
    /// (capped at [`MAX_ROWS`], F12).
    ///
    /// **The read side of `source.child_resources`** (S5 W7 F-SF-03). The
    /// table's own insert comment says the entry content hash and entry
    /// adapter "are already columns on the resource's own row ... and a
    /// reader joins back to it on that key rather than being handed a second,
    /// driftable copy". This is that reader, and it is the reason the
    /// non-duplication is a design and not a hole: without it two of §6.6's
    /// four fields were reachable only by opening the database file directly,
    /// which A1-15 keeps off the public surface.
    ///
    /// Both lanes, one answer: a child that landed as a `source.files`
    /// resource and one that landed as a `source.datasets` registration
    /// (S5 W7 F-SF-01) are the same fact about the same generation, so they
    /// come back in one list with `lane` saying which table holds the
    /// resource. `content_hash`/`extractor`/`lane` are `Option` because the
    /// join is a LEFT JOIN: a child row whose own resource row is missing
    /// would be a real defect, and this answers with the coordinate it does
    /// have rather than dropping the row and hiding it.
    pub fn child_resources(
        &self,
        source_name: &str,
        limit: usize,
    ) -> Result<Vec<StoredChildResource>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(sql!(
            "SELECT c.relative_path, c.local_key, c.parent_relative_path, c.parent_key, \
                    c.entry_path, coalesce(f.content_hash, d.content_hash), \
                    coalesce(f.extractor, d.reader), \
                    CASE WHEN f.relative_path IS NOT NULL THEN 'file' \
                         WHEN d.relative_path IS NOT NULL THEN 'dataset' END \
             FROM source.child_resources c \
             JOIN source.generations g USING (generation_id) \
             LEFT JOIN source.files f \
               ON f.generation_id = c.generation_id \
              AND f.source_name = c.source_name \
              AND f.relative_path = c.relative_path \
             LEFT JOIN source.datasets d \
               ON d.generation_id = c.generation_id \
              AND d.source_name = c.source_name \
              AND d.relative_path = c.relative_path \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY c.relative_path LIMIT ?"
        ))?;
        let mut rows = statement.query(duckdb::params![source_name, STATE_CONFIRMED, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(StoredChildResource {
                relative_path: row.get(0)?,
                key: row.get(1)?,
                parent_relative_path: row.get(2)?,
                parent_key: row.get(3)?,
                entry_path: row.get(4)?,
                content_hash: row.get(5)?,
                extractor: row.get(6)?,
                lane: row.get(7)?,
            });
        }
        Ok(out)
    }

    /// Registered datasets of one source's confirmed generation, in path
    /// order, bounded by `limit` (capped at [`MAX_ROWS`], F12).
    pub fn datasets(
        &self,
        source_name: &str,
        limit: usize,
    ) -> Result<Vec<StoredDataset>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let mut statement = self.conn.prepare(sql!(
            "SELECT d.relative_path, d.format, d.content_hash, d.reader, d.dataset_key, \
                    d.byte_len, d.columns, d.row_count, d.truncated, d.row_units \
             FROM source.datasets d JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY d.relative_path LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT f.relative_path, f.dataset_key, f.query, f.query_identity, f.row_limit, \
                    f.truncated, f.columns, f.rows, f.output_hash \
             FROM source.dataset_facts f JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY f.relative_path, f.query LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT r.relative_path, r.dataset_key, r.ordinal, r.row_key, r.key_basis, \
                    r.fields, r.body \
             FROM context.row_units r JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? \
             ORDER BY r.relative_path, r.ordinal LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
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
             ORDER BY g.source_name LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT u.relative_path, u.local_key, u.ordinal, u.unit_kind, u.heading_level, \
                    u.title, u.byte_start, u.byte_end \
             FROM source.units u JOIN source.generations g USING (generation_id) \
             WHERE g.source_name = ? AND g.state = ? AND u.title IS NOT NULL \
             ORDER BY u.relative_path, u.ordinal LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT g.source_name, s.language, s.label, s.name, s.occurrences \
             FROM source.symbols s JOIN source.generations g USING (generation_id) \
             WHERE g.state = ? AND s.name = ? \
             ORDER BY g.source_name, s.language, s.label LIMIT ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT g.source_name, o.relative_path, o.language, o.label, o.name, o.ordinal, \
                    o.byte_start, o.byte_end \
             FROM source.occurrences o JOIN source.generations g USING (generation_id) \
             WHERE g.state = ? AND o.name = ? \
             ORDER BY g.source_name, o.relative_path, o.ordinal LIMIT ?"
        ))?;
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

    /// A handle onto this connection that **cannot write**, carrying the
    /// whole A2 §2 admissibility filter ([`Admissible`]).
    fn admissible(&self) -> Admissible<'_> {
        Admissible {
            reader: self.conn.reader(),
        }
    }

    /// A2 §2 stages 1(+4) and 2 — see [`Admissible::generations`], which is
    /// where this is implemented and where the doc lives.
    pub fn admissible_generations(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<SourceGeneration>, AtlasError> {
        self.admissible().generations(filter, limit)
    }

    /// A2 §2's **document family** — see [`Admissible::units`].
    pub fn admissible_units(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<StoredUnitHit>, AtlasError> {
        self.admissible().units(filter, limit)
    }

    /// A2 §2's **code family** — see [`Admissible::occurrences`].
    pub fn admissible_occurrences(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<StoredOccurrenceHit>, AtlasError> {
        self.admissible().occurrences(filter, limit)
    }

    /// A2 §2's **tabular family** — see [`Admissible::datasets`].
    pub fn admissible_datasets(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<StoredDatasetHit>, AtlasError> {
        self.admissible().datasets(filter, limit)
    }

    /// What `--work` can honestly claim about this answer — see
    /// [`Admissible::work_scope`].
    fn work_scope(
        &self,
        filter: &Admissibility,
        carries_overlay_rows: bool,
    ) -> Result<WorkScope, AtlasError> {
        self.admissible().work_scope(filter, carries_overlay_rows)
    }

    // ------------------------------------------------------------------
    // S5 W2 — A2 §5's lexical retrieval (decision A2-05).
    //
    // The filter runs FIRST and decides the world; BM25 ranks INSIDE it.
    // That ordering is A2 §2's ("only then retrieve/rank") and A2 §8's
    // prohibition — "the reranker must never silently cross an
    // authority/source filter merely because a candidate scores well" — and
    // here it is structural rather than procedural: every query below joins
    // `context.lexical_units` to `source.generations` through the SAME
    // composed admissibility predicate the `admissible_*` family applies, so
    // an inadmissible generation's postings are unreachable from a search.
    // There is no code path that scores a row first and filters it after,
    // because there is no code path that can see the row at all.
    // ------------------------------------------------------------------

    /// A2 §2's composed stage-1/2/4 predicate's **bind values**, in the order
    /// [`ADMISSIBLE_GENERATION_PREDICATE`] consumes them.
    ///
    /// The predicate text itself is a macro rather than a runtime `format!`,
    /// and that is not a style choice: `tests/x5_a1a_acceptance.rs::
    /// a1a_item_13_no_client_sql_reaches_the_store` pins the exhaustive list
    /// of interpolations in every SQL literal in this file, and W2 adds none
    /// — every statement below is a compile-time literal assembled by
    /// `concat!`. Item 13 forbids handing the store a query; the cheapest way
    /// to keep that true is to have no runtime string-building in the query
    /// path at all.
    ///
    /// Required to agree with the W1 family's own inline predicate, and
    /// pinned by `tests/w2_lexical_retrieval.rs::
    /// every_generation_a_lexical_hit_cites_is_one_the_admissibility_filter_
    /// admits`, which compares the two answers rather than the two strings (a
    /// string comparison would pass on two queries that are identically
    /// wrong).
    fn admissibility_binds(filter: &Admissibility) -> Vec<Duck> {
        let (source_name, content_key) = filter.source.bindings();
        let source_kind = filter.kind.map(SourceKind::as_str);
        let authority = filter.authority.map(AuthorityClass::as_str);
        let overlay_admit = filter.source.overlay_admit_source_name();
        let estate = filter.estate.bind();
        vec![
            Duck::Text(STATE_CONFIRMED.to_string()),
            optional_text(estate),
            optional_text(estate),
            Duck::Text(overlay_exclude_like()),
            optional_text(source_name),
            optional_text(source_name),
            overlay_admit.clone().map_or(Duck::Null, Duck::Text),
            overlay_admit.map_or(Duck::Null, Duck::Text),
            optional_text(content_key),
            optional_text(content_key),
            optional_text(source_kind),
            optional_text(source_kind),
            optional_text(authority),
            optional_text(authority),
        ]
    }

    /// A2 §5's lexical retrieval: BM25 over the admissible set, deterministic
    /// in its ties, every hit carrying A1's own coordinate.
    ///
    /// # What this does, in order
    ///
    /// 1. Tokenizes the query
    ///    ([`crate::runtime::atlas::lexical::query_terms`]) — distinct terms,
    ///    sorted, so the query itself contributes nothing order-dependent.
    /// 2. Measures the corpus **inside the admissible set**: how many units it
    ///    holds and their mean token count. A unit the caller may not see does
    ///    not merely fail to appear in the results — it does not influence the
    ///    IDF or the length normalization of the ones that do.
    /// 3. Per term, reads that term's document frequency and then its
    ///    postings, both over that same admissible set, and accumulates each
    ///    unit's score
    ///    ([`crate::runtime::atlas::lexical::bm25_contribution`]).
    /// 4. Orders by [`crate::runtime::atlas::lexical::rank_order`].
    ///
    /// **Two statements per term rather than one `IN (...)` list**, because an
    /// `IN` list's placeholders have to be built at runtime and item 13's
    /// no-client-SQL pin (see [`Self::admissibility_binds`]) is worth more
    /// than the round trips: a query's distinct-term count is small, and the
    /// document frequency has to be its own exact count anyway — deriving it
    /// from the (capped) posting scan would make a unit's IDF depend on how
    /// many *other* units happened to fit under the cap.
    ///
    /// # Determinism
    ///
    /// Same query + same generations ⇒ same ordered result. Terms are visited
    /// in sorted order, each statement carries its own `ORDER BY`, and
    /// accumulation is a [`BTreeMap`] keyed by `(generation_id, unit_key)` —
    /// never a hash map. The final order is score descending by
    /// `f64::total_cmp`, then the stated tie-break key `(source_name,
    /// relative_path, ordinal, unit_key)` ascending. Nothing in that chain
    /// reads row arrival order.
    ///
    /// # Bounds, and saying so
    ///
    /// The posting scan is capped at [`MAX_ROWS`] postings across all terms
    /// (F12). Because a cap on *postings* silently changes *scores* rather
    /// than merely shortening a list, [`LexicalAnswer::truncated`] says when
    /// it bit, and it is a field on the answer rather than a log line for the
    /// same reason [`WorkScope`] is: a caller must be able to state what its
    /// answer covers.
    pub fn lexical_search(&self, query: &LexicalQuery<'_>) -> Result<LexicalAnswer, AtlasError> {
        let scope = self.work_scope(query.filter, true)?;
        // H4: resolved once, up front, from the caller's request and the
        // model this host actually has — so EVERY return path below carries
        // it, including the three early ones. A status computed only on the
        // path that produces hits is exactly the omittable field H4 forbids.
        let semantic_model = self.semantic_engine().map(|e| e.descriptor().clone());
        let semantic = resolve_semantic(query.semantic, semantic_model.as_ref());
        let semantic_model = match semantic {
            SemanticStatus::Applied => semantic_model,
            _ => None,
        };
        let terms = query_terms(query.text);
        let limit = query.limit.min(MAX_ROWS);
        let empty = LexicalAnswer {
            hits: Vec::new(),
            scope: scope.clone(),
            truncated: false,
            semantic,
            semantic_model: semantic_model.clone(),
        };
        if terms.is_empty() || limit == 0 {
            return Ok(empty);
        }
        let family = query.family.map(LexicalFamily::as_str);
        let admissibility = Self::admissibility_binds(query.filter);

        // (2) the corpus, measured over the admissible, family-filtered set.
        let mut binds = admissibility.clone();
        binds.push(optional_text(family));
        binds.push(optional_text(family));
        let mut statement = self.conn.prepare(sql!(LEXICAL_CORPUS_SQL))?;
        let mut rows = statement.query(duckdb::params_from_iter(binds))?;
        let (units, tokens) = match rows.next()? {
            Some(row) => (
                row.get::<usize, i64>(0)? as u64,
                row.get::<usize, i64>(1)? as u64,
            ),
            None => (0, 0),
        };
        drop(rows);
        drop(statement);
        if units == 0 {
            return Ok(empty);
        }
        let corpus = Bm25Corpus {
            units,
            average_length: tokens as f64 / units as f64,
        };

        let mut scored: BTreeMap<(String, String), Scored> = BTreeMap::new();
        let mut seen = 0usize;
        let mut truncated = false;
        'terms: for term in &terms {
            // (3a) this term's document frequency, exact, over the admissible
            // set — never counted off the capped scan below.
            let mut binds = admissibility.clone();
            binds.push(optional_text(family));
            binds.push(optional_text(family));
            binds.push(Duck::Text(term.clone()));
            let mut statement = self.conn.prepare(sql!(LEXICAL_DOCUMENT_FREQUENCY_SQL))?;
            let mut rows = statement.query(duckdb::params_from_iter(binds))?;
            let document_frequency = match rows.next()? {
                Some(row) => row.get::<usize, i64>(0)? as u64,
                None => 0,
            };
            drop(rows);
            drop(statement);
            if document_frequency == 0 {
                continue;
            }

            // (3b) this term's postings, bounded by what is left of the cap.
            let remaining = MAX_ROWS.saturating_sub(seen);
            let mut binds = admissibility.clone();
            binds.push(optional_text(family));
            binds.push(optional_text(family));
            binds.push(Duck::Text(term.clone()));
            binds.push(Duck::BigInt(remaining as i64 + 1));
            let mut statement = self.conn.prepare(sql!(LEXICAL_POSTINGS_SQL))?;
            let mut rows = statement.query(duckdb::params_from_iter(binds))?;
            while let Some(row) = rows.next()? {
                if seen >= MAX_ROWS {
                    truncated = true;
                    break 'terms;
                }
                seen += 1;
                let generation_id: String = row.get(0)?;
                let unit_key: String = row.get(4)?;
                let token_count = row.get::<usize, i64>(16)? as u64;
                let term_frequency = row.get::<usize, i64>(17)? as u64;
                let entry = match scored.entry((generation_id.clone(), unit_key.clone())) {
                    std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
                    std::collections::btree_map::Entry::Vacant(slot) => slot.insert(Scored {
                        hit: LexicalHit {
                            score: 0.0,
                            source_name: row.get(1)?,
                            // A2 §17 item 8: the two columns the
                            // admissibility predicate already binds against,
                            // now carried out on the hit so an answer is
                            // visibly external without a second lookup.
                            source_kind: source_kind_at(row, 18)?,
                            authority_class: authority_class_at(row, 19)?,
                            generation_id,
                            content_key: row.get(2)?,
                            unit_key,
                            coordinate: coordinate_of(row)?,
                        },
                        token_count,
                    }),
                };
                entry.hit.score += bm25_contribution(
                    corpus,
                    document_frequency,
                    term_frequency,
                    entry.token_count,
                );
            }
        }

        let mut hits: Vec<LexicalHit> = scored.into_values().map(|s| s.hit).collect();
        hits.sort_by(rank_order);
        hits.truncate(limit);
        Ok(LexicalAnswer {
            hits,
            scope,
            truncated,
            semantic,
            semantic_model,
        })
    }

    /// A2 §6's model for this handle, loaded lazily and **at most once**.
    ///
    /// `None` means A2-13's supported degraded state: no complete asset
    /// directory was found (see
    /// [`crate::runtime::atlas::semantic::model_dir`]). A directory that
    /// exists but will not load is also reported as `None` *here* — the
    /// error is logged rather than propagated, because a broken model must
    /// not turn a lexical search into a failure; the honest answer is
    /// `semantic: not_installed` plus the lexical half, which is exactly
    /// what A2 §15 asks for.
    pub fn semantic_engine(&self) -> Option<&SemanticEngine> {
        self.semantic
            .get_or_init(|| match SemanticEngine::load() {
                Ok(engine) => engine,
                Err(error) => {
                    log::warn!("{error}");
                    None
                }
            })
            .as_ref()
    }

    /// A2 §6's semantic retrieval: **exact cosine over the admissible set**,
    /// deterministic in its ties, every hit carrying A1's own coordinate.
    ///
    /// Takes the same [`LexicalQuery`] the lexical half takes, and that is
    /// deliberate: W4 fuses the two rank lists with RRF, and two lists
    /// produced from two differently-spelled filters are not fusable. One
    /// query value, one filter, two rankers.
    ///
    /// # What this does, in order
    ///
    /// 1. Resolves H4's status. If it is not
    ///    [`SemanticStatus::Applied`] — no assets installed, or the caller
    ///    suppressed the semantic half — the answer is **empty hits with the
    ///    status saying why**, not an error and not a silent lexical
    ///    substitution.
    /// 2. Embeds the query once.
    /// 3. Walks the **admissible generations**
    ///    ([`Self::admissible_generations`]) and, for each, the units
    ///    [`indexable_units`] derives — the same units the lexical index is
    ///    built from, so the two halves rank over one corpus.
    /// 4. Embeds each generation's unit texts in one batch and scores them
    ///    with [`cosine`].
    /// 5. Orders by [`rank_semantic`].
    ///
    /// # A2 §8's prohibition is structural, not procedural
    ///
    /// *"The reranker must never silently cross an authority/source filter
    /// merely because a candidate scores well."* Step 3 is the whole of that
    /// guarantee: a generation the filter excludes is never enumerated, so
    /// its units are never embedded and cannot be scored at all. There is no
    /// post-filter to forget to apply.
    /// `tests/w3b_semantic_retrieval.rs::
    /// an_inadmissible_unit_that_scores_first_unfiltered_is_absent_once_filtered`
    /// shows the same unit ranking first with the filter open and absent with
    /// it closed — the negative made non-vacuous the way W2 made its own.
    ///
    /// # A2-07: this is a linear scan and there is no index
    ///
    /// Decision A2-07 (**R1**) and A2 §16's *"vector database/ANN engine
    /// before measurement"* non-goal. Every admissible unit is embedded and
    /// scored on every query; nothing is cached, pruned or approximated. The
    /// cost of that is measured rather than assumed —
    /// `knowledge/evidence/perf/model2vec-footprint-and-scan-2026-08-30.md`
    /// records the figure, and a figure like it is the only thing that could
    /// ever justify adding an index.
    ///
    /// # Bounds, and saying so
    ///
    /// The scan visits at most [`MAX_ROWS`] units across all admissible
    /// generations (F12). Because a cap changes *which* units could be
    /// ranked rather than merely shortening the list,
    /// [`SemanticAnswer::truncated`] says when it bit — the same disclosure,
    /// for the same reason, as [`LexicalAnswer::truncated`].
    pub fn semantic_search(&self, query: &LexicalQuery<'_>) -> Result<SemanticAnswer, AtlasError> {
        let scope = self.work_scope(query.filter, true)?;
        let engine = self.semantic_engine();
        let descriptor = engine.map(|e| e.descriptor().clone());
        let semantic = resolve_semantic(query.semantic, descriptor.as_ref());
        let mut answer = SemanticAnswer {
            hits: Vec::new(),
            scope,
            truncated: false,
            semantic,
            semantic_model: match semantic {
                SemanticStatus::Applied => descriptor,
                _ => None,
            },
        };
        let engine = match (semantic, engine) {
            (SemanticStatus::Applied, Some(engine)) => engine,
            _ => return Ok(answer),
        };
        let limit = query.limit.min(MAX_ROWS);
        if query.text.trim().is_empty() || limit == 0 {
            return Ok(answer);
        }
        let query_vector = engine.embed_query(query.text);

        let admitted = self.admissible_generations(query.filter, MAX_ROWS)?;
        let family = query.family;
        let mut hits: Vec<SemanticHit> = Vec::new();
        let mut seen = 0usize;
        'generations: for generation in &admitted.hits {
            let units = indexable_units(&self.conn, &generation.id)?;
            let units: Vec<IndexableUnit> = units
                .into_iter()
                .filter(|unit| family.is_none_or(|wanted| unit.family == wanted))
                .collect();
            if units.is_empty() {
                continue;
            }
            let texts: Vec<String> = units.iter().map(|unit| unit.text.clone()).collect();
            let vectors = engine.embed(&texts);
            for (unit, vector) in units.iter().zip(vectors.iter()) {
                if seen >= MAX_ROWS {
                    answer.truncated = true;
                    break 'generations;
                }
                seen += 1;
                hits.push(SemanticHit {
                    score: cosine(&query_vector, vector),
                    source_name: unit.source_name.clone(),
                    // A2 §17 item 8, from the admitted generation row this
                    // unit was enumerated under — the same two values the
                    // lexical half reads off `source.generations`.
                    source_kind: generation.kind,
                    authority_class: generation.authority,
                    generation_id: generation.id.clone(),
                    content_key: generation.content_key.clone(),
                    unit_key: unit.unit_key.clone(),
                    coordinate: unit.coordinate(),
                });
            }
        }
        hits.sort_by(rank_semantic);
        hits.truncate(limit);
        answer.hits = hits;
        Ok(answer)
    }

    /// A2 §7's Reciprocal Rank Fusion followed by A2 §8's deterministic
    /// reranking: **one answer built from the two rank lists, inside one
    /// admissibility filter.**
    ///
    /// # What this does, in order
    ///
    /// 1. Runs [`Self::lexical_search`] and [`Self::semantic_search`] over
    ///    **the same [`LexicalQuery`] value** — one filter, both halves. Two
    ///    lists produced from two differently-spelled filters are not
    ///    fusable, which is why `semantic_search` was given this query type
    ///    in W3b rather than one of its own.
    /// 2. Fuses them with
    ///    [`crate::runtime::atlas::fusion::fuse`] — A2 §7's one expression.
    /// 3. Computes A2 §8's nine signals for every candidate
    ///    ([`Self::rerank_signals`]) and reranks
    ///    ([`crate::runtime::atlas::fusion::rerank`]).
    /// 4. Truncates to the caller's `limit`.
    ///
    /// # Both halves run at [`MAX_ROWS`], not at the caller's `limit`
    ///
    /// `rank_i(d)` must be the candidate's rank **within the admissible
    /// set**, not within whatever slice the caller wanted to display. If the
    /// halves were run at `limit`, a unit at lexical rank 12 would be absent
    /// from the lexical list at `limit = 10` and present at `limit = 20`, and
    /// the fused *order of the first ten* would change with a display
    /// parameter — a determinism hazard that looks like nothing until two
    /// callers disagree. It costs no extra scanning: both halves already
    /// score the whole admissible set and only truncate at the end.
    /// `tests/w4_rrf_fusion.rs::
    /// the_fused_order_does_not_depend_on_the_callers_limit` is the pin.
    ///
    /// # The prohibition
    ///
    /// *"The reranker must never silently cross an authority/source filter
    /// merely because a candidate scores well."* Every candidate here came
    /// from one of the two filtered lists;
    /// [`crate::runtime::atlas::fusion::fuse`] holds no store handle and
    /// cannot fetch one, and [`Self::rerank_signals`] only ever *marks*
    /// candidates that already exist — its two `source.edges` reads are
    /// scoped to the anchor's own (already admissible) generation and their
    /// results are membership tests, never a source of new hits.
    pub fn fused_search(&self, query: &LexicalQuery<'_>) -> Result<FusedAnswer, AtlasError> {
        let full = LexicalQuery {
            text: query.text,
            filter: query.filter,
            family: query.family,
            limit: MAX_ROWS,
            semantic: query.semantic,
        };
        // F-IN-01: `lexical_search` and `semantic_search` are two
        // independent reads of `source.generations`/`source.edges`; without
        // an enclosing transaction a concurrent writer's commit landing
        // between them (e.g. `confirm_scan` promoting a generation) could
        // hand back a fused answer built from two different committed
        // states of the store — undermining A2 §4's output-hash
        // reproducibility exactly as much as any of the four named
        // determinism hazards would. `lexical_search`/`semantic_search`/
        // `rerank_signals` all take `&self`, so the ordinary `Connection::
        // transaction` (which requires `&mut self` to rule out nesting) is
        // not available without widening every one of their signatures;
        // `Transaction::new_unchecked` (R5) is the crate's own documented
        // escape hatch for exactly this "`&mut Connection` is unacceptable"
        // case, and opens the same snapshot-isolated transaction from a
        // shared `&Connection`. It is dropped (and rolled back — nothing
        // here writes) on every exit path, `?` included.
        let snapshot = self.conn.snapshot()?;
        let lexical = self.lexical_search(&full)?;
        let semantic = self.semantic_search(&full)?;
        let mut hits = fuse(&lexical.hits, &semantic.hits);
        self.rerank_signals(query, &mut hits)?;
        drop(snapshot);
        rerank(&mut hits);
        hits.truncate(query.limit.min(MAX_ROWS));
        Ok(FusedAnswer {
            hits,
            scope: lexical.scope,
            truncated: lexical.truncated || semantic.truncated,
            semantic: lexical.semantic,
            semantic_model: lexical.semantic_model,
        })
    }

    /// **A2 §13's search trace, attached to the answer it describes.**
    ///
    /// [`Self::fused_search`] plus the nine fields §13 says to *"record at
    /// minimum"* — see [`crate::runtime::atlas::trace`] for the field-by-field
    /// account and for why the trace rides the answer rather than being
    /// journaled (`sgt search` is a pure reader; the pin is
    /// `tests/w1b_overlay_lifecycle_trigger.rs::
    /// the_admissibility_filter_cannot_write_and_neither_can_anything_it_calls`).
    ///
    /// Not folded into `fused_search` itself (**R1**): the retrieval halves
    /// have three in-tree callers that want the ranked list and not the
    /// trace, and building §13's generation list costs a second read of
    /// `source.generations`. A caller that wants the trace asks for it.
    pub fn traced_search(
        &self,
        query: &LexicalQuery<'_>,
        attribution: Attribution,
    ) -> Result<(FusedAnswer, SearchTrace), AtlasError> {
        let answer = self.fused_search(query)?;
        let trace = self.trace_of(query, attribution, &answer)?;
        Ok((answer, trace))
    }

    /// A2 §13's nine fields for one answered query.
    fn trace_of(
        &self,
        query: &LexicalQuery<'_>,
        attribution: Attribution,
        answer: &FusedAnswer,
    ) -> Result<SearchTrace, AtlasError> {
        // Field 5's generation list: the exact worlds the filter admitted.
        // The same call `semantic_search` enumerates over, at the same cap,
        // so the trace describes the world the answer was actually computed
        // in rather than a second, differently-bounded one.
        let admitted = self.admissible_generations(query.filter, MAX_ROWS)?;
        let (selector, source_name, content_key) = match &query.filter.source {
            SourceSelector::Any => ("any", None, None),
            SourceSelector::Named(name) => ("named", Some(name.clone()), None),
            SourceSelector::Exact {
                source_name,
                content_key,
            } => (
                "exact",
                Some(source_name.clone()),
                Some(content_key.clone()),
            ),
            SourceSelector::WorkBase { repository, .. } => {
                ("work_base", Some(repository.clone()), None)
            }
        };
        Ok(SearchTrace {
            query: QueryIdentity::of(query.text),
            attribution,
            source_generation_filter: SourceGenerationFilter {
                selector,
                source_name,
                content_key,
                work_scope: describe_work_scope(&answer.scope),
            },
            content_authority_filter: ContentAuthorityFilter {
                content: query.family,
                kind: query.filter.kind,
                authority: query.filter.authority,
            },
            retrieval_generation: RetrievalGeneration {
                index_version: RETRIEVAL_INDEX_VERSION,
                truncated: admitted.hits.len() >= MAX_ROWS,
                generations: admitted.hits,
            },
            lexical: LexicalIdentity::default(),
            semantic: answer.semantic,
            semantic_model: answer.semantic_model.clone(),
            policy: PolicyIdentity::default(),
            results: answer
                .hits
                .iter()
                .enumerate()
                .map(|(index, hit)| ResultRank::of(index, hit))
                .collect(),
        })
    }

    /// **A2 §14's second verb: `sgt related <coordinate>`.**
    ///
    /// Neighbours of one already-retrieved unit, inside the same A2 §2
    /// admissibility filter — *"more like this"*, answered by the retrieval
    /// pipeline that already exists rather than by a second mechanism
    /// (**R2**): the anchor unit's own indexed text becomes the query text,
    /// and [`Self::traced_search`] answers it. There is no new ranker, no new
    /// index and no new score here; A2 §16's non-goals stay non-goals.
    ///
    /// # The anchor is resolved through the filter, never around it
    ///
    /// The anchor generation comes from [`Self::admissible_generations`], so
    /// a coordinate naming a source the caller may not see resolves to
    /// `Ok(None)` — "no such unit in this world" — exactly as A2 §2's
    /// *"never approximate"* requires of an admissibility miss. A2 §8's
    /// prohibition (*"never silently cross an authority/source filter"*)
    /// would otherwise have a hole shaped like an anchor lookup.
    ///
    /// # The anchor is excluded from its own neighbours
    ///
    /// A unit is trivially its own best match under both halves, and an
    /// answer whose first neighbour is the thing asked about is not a list of
    /// neighbours. It is dropped by `(generation_id, unit_key)` — Atlas's own
    /// unit identity, the same key [`crate::runtime::atlas::fusion::fuse`]
    /// joins on — after ranking, and the search runs one wider so dropping it
    /// does not shorten the answer.
    ///
    /// `Ok(None)` when the coordinate names no admissible unit. Every read
    /// here is bounded: one admissible-generation list at [`MAX_ROWS`], one
    /// generation's [`indexable_units`], and the search's own caps.
    pub fn related(
        &self,
        request: &RelatedRequest<'_>,
    ) -> Result<Option<RelatedAnswer>, AtlasError> {
        let admitted = self.admissible_generations(request.filter, MAX_ROWS)?;
        let Some(generation) = admitted
            .hits
            .iter()
            .find(|generation| generation.source_name == request.source_name)
        else {
            return Ok(None);
        };
        let Some(unit) = indexable_units(&self.conn, &generation.id)?
            .into_iter()
            .find(|unit| unit.unit_key == request.unit_key)
        else {
            return Ok(None);
        };
        let anchor = RelatedAnchor {
            source_name: generation.source_name.clone(),
            source_kind: generation.kind,
            authority_class: generation.authority,
            generation_id: generation.id.clone(),
            content_key: generation.content_key.clone(),
            unit_key: unit.unit_key.clone(),
            coordinate: unit.coordinate(),
        };
        let limit = request.limit.min(MAX_ROWS);
        let query = LexicalQuery {
            text: &unit.text,
            filter: request.filter,
            // Deliberately unfamily-filtered: a document that explains a
            // function is a neighbour of it, and A2 §8's own signal 5 ("same
            // module/package/document section") only means anything across
            // families. `--content` still narrows it when a caller asks.
            family: request.family,
            // One wider, so removing the anchor below returns `limit`
            // neighbours rather than `limit - 1`.
            limit: limit.saturating_add(1).min(MAX_ROWS),
            semantic: request.semantic,
        };
        let (mut answer, mut trace) = self.traced_search(&query, request.attribution.clone())?;
        answer.hits.retain(|hit| {
            hit.generation_id != anchor.generation_id || hit.unit_key != anchor.unit_key
        });
        answer.hits.truncate(limit);
        trace.results = answer
            .hits
            .iter()
            .enumerate()
            .map(|(index, hit)| ResultRank::of(index, hit))
            .collect();
        Ok(Some(RelatedAnswer {
            anchor,
            answer,
            trace,
        }))
    }

    /// Fill in A2 §8's nine signals for every fused candidate, from A1's own
    /// structure and provenance — *"rather than training another ranker"*
    /// (§8), decision **A2-09 (R2)**: *"Structural relationships already
    /// exist; reuse them."*
    ///
    /// The *anchor* for the two relational signals (5 and 6) is the candidate
    /// RRF ranked first — `hits` arrives in
    /// [`crate::runtime::atlas::fusion::rrf_order`], so the anchor is a
    /// function of the fused score and the stated tie-break key, never of
    /// iteration order.
    fn rerank_signals(
        &self,
        query: &LexicalQuery<'_>,
        hits: &mut [FusedHit],
    ) -> Result<(), AtlasError> {
        let Some(anchor) = hits.first() else {
            return Ok(());
        };
        let anchor_coordinate = anchor.coordinate.clone();
        let anchor_generation = anchor.generation_id.clone();
        let anchor_symbol = symbol_of(&anchor_coordinate).map(str::to_string);

        let terms = query_terms(query.text);
        let identifier_like = is_identifier_like(query.text);
        // Signal 3: the caller named a source at all. See
        // `RerankSignals`'s own doc for why this is uniform.
        let selected_source = !matches!(query.filter.source, SourceSelector::Any);
        // Signal 8: `--type knowledge`.
        let knowledge_requested = query.filter.kind == Some(SourceKind::LocalKnowledge);
        // Signal 4: this Work's overlay source name, when the filter is a
        // `--work` one. `None` for every other selector, so no non-Work query
        // can accidentally match a unit whose source merely looks like one.
        let overlay_source = query.filter.source.overlay_admit_source_name();
        // Signal 4's other half (F-SF-01): an overlay generation's universe
        // is the base tree *plus* whatever the surface changed
        // (`extract_overlay`, overlay.rs), so every unchanged path is
        // indexed under the same overlay source name as every changed one —
        // source-name equality alone cannot tell "the Work touched this"
        // from "this is merely visible under the Work's view". The overlay
        // schema carries no per-unit changed/unchanged flag, but
        // `source.files` already carries a content hash per path (F7), and
        // an unchanged path's overlay row is read from the exact same blob
        // as the base tree's (overlay.rs's own doc: "an overlay and a plain
        // estate-git scan of the same base agree on every unchanged path by
        // construction"). So a path whose overlay content hash differs from
        // the base generation's content hash at the same path — or is
        // absent from the base entirely — is a path the Work actually
        // changed; one whose hash matches is not, regardless of source
        // name.
        let base_generation_id = match &query.filter.source {
            SourceSelector::WorkBase { repository, .. } => self
                .confirmed_generation(repository)?
                .map(|generation| generation.id),
            _ => None,
        };
        let mut base_content_hash: BTreeMap<String, Option<String>> = BTreeMap::new();
        // Signal 9: the caller pinned an exact generation, so A2 §8's
        // "unless caller pinned stale" suppresses the current-generation
        // preference.
        let pinned = matches!(query.filter.source, SourceSelector::Exact { .. });

        // Signal 6, both directions, read once from the anchor's own
        // generation.
        let mut referencing_paths: BTreeSet<String> = BTreeSet::new();
        let mut anchor_targets: BTreeSet<String> = BTreeSet::new();
        if let Some(symbol) = &anchor_symbol {
            let mut statement = self.conn.prepare(sql!(EDGES_TO_TARGET_SQL))?;
            let mut rows =
                statement.query(duckdb::params![anchor_generation, symbol, MAX_ROWS as i64])?;
            while let Some(row) = rows.next()? {
                referencing_paths.insert(row.get(0)?);
            }
        }
        {
            let mut statement = self.conn.prepare(sql!(EDGES_FROM_PATH_SQL))?;
            let mut rows = statement.query(duckdb::params![
                anchor_generation,
                anchor_coordinate.relative_path(),
                MAX_ROWS as i64
            ])?;
            while let Some(row) = rows.next()? {
                anchor_targets.insert(row.get(0)?);
            }
        }

        // Signal 9's other half: which generation is each source's current
        // confirmed one. A `BTreeMap` cache, not a per-hit query.
        let mut current: BTreeMap<String, Option<String>> = BTreeMap::new();
        for hit in hits.iter() {
            if !current.contains_key(&hit.source_name) {
                let confirmed = self
                    .confirmed_generation(&hit.source_name)?
                    .map(|generation| generation.id);
                current.insert(hit.source_name.clone(), confirmed);
            }
        }

        for hit in hits.iter_mut() {
            let same_generation = hit.generation_id == anchor_generation;
            let work_changed_unit = match overlay_source.as_deref() {
                Some(overlay) if overlay == hit.source_name => match &base_generation_id {
                    Some(base_id) => {
                        let relative_path = hit.coordinate.relative_path();
                        if !base_content_hash.contains_key(relative_path) {
                            let hash = self.file_content_hash(base_id, relative_path)?;
                            base_content_hash.insert(relative_path.to_string(), hash);
                        }
                        let base_hash = base_content_hash
                            .get(relative_path)
                            .and_then(Option::as_deref);
                        let overlay_hash =
                            self.file_content_hash(&hit.generation_id, relative_path)?;
                        base_hash != overlay_hash.as_deref()
                    }
                    // No confirmed base generation to compare against —
                    // nothing here can be shown to differ from it, so this
                    // signal stays honestly false rather than guessing.
                    None => false,
                },
                _ => false,
            };
            hit.signals = RerankSignals {
                exact_match: exact_match(&terms, &hit.coordinate),
                definition_over_reference: identifier_like
                    && hit.coordinate.family() == LexicalFamily::Code,
                caller_selected_source: selected_source,
                work_changed_unit,
                same_section_as_anchor: same_section(&anchor_coordinate, &hit.coordinate),
                structural_relationship: same_generation
                    && (referencing_paths.contains(hit.coordinate.relative_path())
                        || symbol_of(&hit.coordinate)
                            .is_some_and(|symbol| anchor_targets.contains(symbol))),
                canonical_path: is_canonical_path(hit.coordinate.relative_path()),
                knowledge_source_requested: knowledge_requested,
                current_generation: pinned
                    || current
                        .get(&hit.source_name)
                        .and_then(Option::as_deref)
                        .is_some_and(|id| id == hit.generation_id),
            };
        }
        Ok(())
    }

    /// Rebuild the whole lexical index from the A1 rows it is derived from.
    ///
    /// The index is **derived evidence**: the journal, Git and the original
    /// bytes remain authority (A1-01), so losing it costs nothing that cannot
    /// be recomputed. This is what makes that claim checkable rather than
    /// asserted — and it is the upgrade path for a store written before S5
    /// W2, whose generations have rows but no postings.
    ///
    /// Every non-evicted generation is re-derived through the same
    /// [`index_generation`] the staging transaction uses, in one transaction,
    /// after its existing rows are cleared — up to [`MAX_ROWS`] generations;
    /// beyond that the generation list is capped and
    /// [`ReindexOutcome::truncated`] says so, the same bound and the same
    /// disclosure `lexical_search` uses for its posting scan.
    pub fn reindex_lexical(&mut self) -> Result<ReindexOutcome, AtlasError> {
        let mut statement = self.conn.prepare(sql!(
            "SELECT generation_id FROM source.generations WHERE state != ? \
             ORDER BY observed_at, generation_id LIMIT ?"
        ))?;
        let mut rows = statement.query(duckdb::params![STATE_EVICTED, MAX_ROWS as i64 + 1])?;
        let mut targets: Vec<String> = Vec::new();
        while let Some(row) = rows.next()? {
            targets.push(row.get(0)?);
        }
        drop(rows);
        drop(statement);
        let truncated = targets.len() > MAX_ROWS;
        targets.truncate(MAX_ROWS);
        let tx = self.conn.transaction()?;
        let mut indexed = 0u64;
        for generation_id in &targets {
            tx.execute(
                sql!("DELETE FROM context.lexical_postings WHERE generation_id = ?"),
                duckdb::params![generation_id],
            )?;
            tx.execute(
                sql!("DELETE FROM context.lexical_units WHERE generation_id = ?"),
                duckdb::params![generation_id],
            )?;
            indexed += index_generation(&tx, generation_id)?;
        }
        tx.commit()?;
        Ok(ReindexOutcome { indexed, truncated })
    }

    /// Whether the lexical index is missing postings for a non-evicted
    /// generation that has rows — the exact condition
    /// [`Self::reindex_lexical`]'s doc names as "a store written before S5
    /// W2". Cheap: an anti-join over generation ids, not a rebuild, so it is
    /// safe to call every startup rather than only once at a version
    /// boundary this crate has no other way to detect (F-SF-01).
    pub fn lexical_index_needs_rebuild(&self) -> Result<bool, AtlasError> {
        let mut statement = self.conn.prepare(sql!(
            "SELECT COUNT(*) FROM source.generations g \
             WHERE g.state != ? \
               AND NOT EXISTS ( \
                 SELECT 1 FROM context.lexical_units l \
                 WHERE l.generation_id = g.generation_id \
               )"
        ))?;
        let mut rows = statement.query(duckdb::params![STATE_EVICTED])?;
        let count: i64 = match rows.next()? {
            Some(row) => row.get(0)?,
            None => 0,
        };
        Ok(count > 0)
    }

    /// Every generation's state, keyed by id — a diagnostic read, and what a
    /// crash-window test inspects.
    pub fn generation_states(&self) -> Result<BTreeMap<String, String>, AtlasError> {
        let mut statement = self.conn.prepare(sql!(
            "SELECT generation_id, state FROM source.generations \
             ORDER BY observed_at, generation_id LIMIT ?"
        ))?;
        let mut rows = statement.query(duckdb::params![MAX_ROWS as i64])?;
        let mut out = BTreeMap::new();
        while let Some(row) = rows.next()? {
            out.insert(row.get::<usize, String>(0)?, row.get::<usize, String>(1)?);
        }
        Ok(out)
    }

    /// Ids and source names of every generation in one state.
    fn generations_in_state(&self, state: &str) -> Result<Vec<(String, String)>, AtlasError> {
        let mut statement = self.conn.prepare(sql!(
            "SELECT generation_id, source_name FROM source.generations WHERE state = ? \
             ORDER BY observed_at DESC, generation_id DESC LIMIT ?"
        ))?;
        let mut rows = statement.query(duckdb::params![state, MAX_ROWS as i64])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push((row.get(0)?, row.get(1)?));
        }
        Ok(out)
    }

    /// Which source a generation belongs to.
    fn generation_source(&self, generation_id: &str) -> Result<Option<String>, AtlasError> {
        let mut statement = self.conn.prepare(sql!(
            "SELECT source_name FROM source.generations WHERE generation_id = ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT extractors FROM source.generations WHERE generation_id = ?"
        ))?;
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
        let mut statement = self.conn.prepare(sql!(
            "SELECT content_key FROM source.generations WHERE generation_id = ?"
        ))?;
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
    conn: &impl Statements,
    generation_id: &str,
    source_name: &str,
    file: &ScannedFile,
) -> Result<(), AtlasError> {
    conn.prepare_cached(sql!(
        "INSERT INTO source.files \
         (generation_id, source_name, relative_path, content_hash, extractor, local_key, \
          byte_len, mtime_millis, unit_count) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ))?
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
    if let Some(parent) = &file.parent {
        // A1 §6.6's parent-provenance fields (S5 W7): parent resource,
        // parent key, entry path. In their own table, not as three more
        // columns on `source.files`: this file's own module doc states the
        // rule ("These tables are only ever added to, never altered" — the
        // DDL is `IF NOT EXISTS`, so a column added to an existing table
        // would silently not appear in a database that already has it), and
        // X3b's rows already set the precedent of a new table carrying its
        // own copy of the coordinates it needs.
        //
        // The other two of §6.6's four fields — entry content hash, entry
        // adapter (F-SI-01) — are NOT duplicated here: they are already
        // columns on the `source.files` row this same INSERT just wrote for
        // this exact (generation_id, source_name, relative_path), and a
        // reader joins back to it on that key rather than being handed a
        // second, driftable copy.
        insert_child_resource(
            conn,
            generation_id,
            source_name,
            &file.relative_path,
            &file.local_key,
            parent,
        )?;
    }
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
    conn: &impl Statements,
    generation_id: &str,
    source_name: &str,
    file: &ScannedFile,
    syntax: &ScannedSyntax,
) -> Result<(), AtlasError> {
    for symbol in &syntax.symbols {
        conn.prepare_cached(sql!(
            "INSERT INTO source.occurrences \
             (generation_id, source_name, relative_path, syntax_key, extractor, language, \
              ordinal, label, name, byte_start, byte_end) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ))?
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
        conn.prepare_cached(sql!(
            "INSERT INTO source.edges \
             (generation_id, source_name, relative_path, syntax_key, extractor, language, \
              ordinal, edge_kind, target, byte_start, byte_end) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ))?
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
    conn: &impl Statements,
    generation_id: &str,
    source_name: &str,
    file: &ScannedFile,
    unit: &ScannedUnit,
) -> Result<(), AtlasError> {
    conn.prepare_cached(sql!(
        "INSERT INTO source.units \
         (generation_id, source_name, relative_path, local_key, ordinal, unit_kind, \
          heading_level, title, byte_start, byte_end, body) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ))?
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
    // A2 §9's *native coordinate*, in its own table rather than a column on
    // `source.units` — this module's own rule (see the module doc): a landed
    // table is only ever added to, never altered, so a new fact arrives as a
    // new table carrying its own copy of the coordinates that address it.
    //
    // A row only when there is one. `None` is every in-process text/Markdown
    // unit, whose byte span is already its address; a row of `NULL` here
    // would be a declared-but-empty promise for those.
    if let Some(coordinate) = unit.coordinate.as_deref() {
        conn.prepare_cached(sql!(
            "INSERT INTO source.unit_coordinates \
             (generation_id, source_name, relative_path, local_key, ordinal, coordinate) \
             VALUES (?, ?, ?, ?, ?, ?)"
        ))?
        .execute(duckdb::params![
            generation_id,
            source_name,
            &file.relative_path,
            &file.local_key,
            unit.ordinal as i64,
            coordinate,
        ])?;
    }
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
    conn: &impl Statements,
    sql: &Sql,
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
    conn: &impl Statements,
    query: &DatasetQuery,
    sql: &Sql,
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
    conn: &impl Statements,
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
    sql: &Sql,
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
/// The daemon-owned scratch directory child dataset bytes are materialised
/// under, relative to the data directory (S5 W7 F-SF-01). One directory, so
/// a leaked scratch dir is obvious and sweepable; each read gets its own
/// unique subdirectory inside it and removes it on drop.
const CHILD_DATASET_SCRATCH: &str = "atlas-child-datasets";

/// The detail a child dataset's coverage row carries when the sole writer
/// could not give its bytes a path to be read from (S5 W7 F-SF-01).
const DATASET_CHILD_NOT_MATERIALISED: &str =
    "a container child's bytes could not be written to daemon-owned scratch to be read in place";

/// One materialised child dataset: a private directory under the daemon's
/// own data directory, removed when this value is dropped (S5 W7 F-SF-01).
struct MaterialisedChild {
    dir: PathBuf,
    file: PathBuf,
}

impl Drop for MaterialisedChild {
    fn drop(&mut self) {
        // Best effort by necessity — a failed cleanup must not turn a
        // successful read into an error — but not silent: the directory is
        // under the daemon's own data dir, so a leaked one is visible there
        // rather than in `$TMPDIR` where nobody would connect it to Atlas.
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Write one child dataset's bytes where [`read_dataset`]'s own reader can
/// open them, under a name the daemon composed (S5 W7 F-SF-01).
///
/// R3/R5 before R7: `std::fs` plus the already-installed `ulid` for the
/// unique directory component — no new dependency, and deliberately not
/// `$TMPDIR`, so the bytes never leave the directory the daemon already owns
/// exclusively.
fn materialise_child_dataset(
    scratch_root: &Path,
    dataset: &ScannedDataset,
    bytes: &[u8],
) -> std::io::Result<MaterialisedChild> {
    let dir = scratch_root.join(ulid::Ulid::generate().to_string());
    std::fs::create_dir_all(&dir)?;
    let extension = dataset
        .format
        .extensions()
        .first()
        .copied()
        .unwrap_or("data");
    // The daemon's own hash, never the entry's own name.
    let file = dir.join(format!("{}.{extension}", dataset.content_hash));
    let child = MaterialisedChild {
        dir,
        file: file.clone(),
    };
    std::fs::write(&file, bytes)?;
    Ok(child)
}

fn read_dataset(
    conn: &impl Statements,
    scan: &SourceScan,
    dataset: &ScannedDataset,
    scratch_root: &Path,
) -> IngestedDataset {
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
    // Held for the whole function: dropping it removes the directory, so
    // every `return` below cleans up without a second cleanup path to forget.
    let materialised;
    let absolute = match &dataset.content {
        Some(bytes) => {
            // S5 W7 (F-SF-01). A container child has no path DuckDB can read
            // in place, so the sole writer gives it one for the length of
            // this read and takes it away again.
            //
            // The three properties that make this safe are all here, not
            // implied: the DIRECTORY is daemon-owned scratch under the data
            // directory (never `$TMPDIR`, never anywhere a source root or a
            // Work surface can see); the FILENAME is the daemon's own content
            // hash plus the format's own extension — never the entry's own
            // name, so nothing attacker-controlled reaches a path, and no
            // glob metacharacter can appear in one (`GLOB_METACHARACTERS` is
            // still checked below, over the whole absolute string, exactly as
            // it is for a loose dataset); and NOTHING EXECUTES IT — it is
            // opened by a `read_csv`/`read_json`/`read_parquet` table
            // function and by nothing else, which is A1 §6.6's "no archive
            // entry is executed" holding unchanged.
            match materialise_child_dataset(scratch_root, dataset, bytes) {
                Ok(scratch) => {
                    materialised = scratch;
                    materialised.file.clone()
                }
                Err(e) => return failed(format!("{DATASET_CHILD_NOT_MATERIALISED}: {e}")),
            }
        }
        None => {
            let Some(root) = scan.root.as_ref() else {
                // Unreachable from the three walks in this build — only a
                // filesystem walk registers a dataset that has a path — but
                // stated rather than assumed, because this is a public store
                // and the alternative is a panic.
                return failed(crate::runtime::atlas::scan::DATASET_NO_ROOT.to_string());
            };
            root.join(&dataset.relative_path)
        }
    };
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
    conn: &impl Statements,
    generation_id: &str,
    scan: &SourceScan,
    dataset: &ScannedDataset,
    read: &IngestedDataset,
) -> Result<CoverageRow, AtlasError> {
    conn.prepare_cached(sql!(
        "INSERT INTO source.datasets \
         (generation_id, source_name, relative_path, format, content_hash, reader, dataset_key, \
          byte_len, mtime_millis, columns, row_count, truncated, row_units) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ))?
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
    if let Some(parent) = &dataset.parent {
        // A1 §6.6's parent coordinate, in the SAME table a child
        // `source.files` row writes it to (S5 W7 F-SF-01). "This resource
        // expanded out of that container" is one fact about one generation;
        // splitting it across two tables by which lane the entry routed to
        // would make every reader join twice to ask one question.
        insert_child_resource(
            conn,
            generation_id,
            &scan.source_name,
            &dataset.relative_path,
            &dataset.dataset_key,
            parent,
        )?;
    }
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

/// Insert one child resource's A1 §6.6 parent coordinate.
///
/// One function for both lanes (S5 W7 F-SF-01): a child that landed as a
/// `source.files` row and a child that landed as a `source.datasets`
/// registration write the identical shape, so the row means the same thing
/// whichever table holds the resource it names. `key` is that resource's own
/// F7 key in its own table — `local_key` for a file, `dataset_key` for a
/// dataset.
///
/// The other two of §6.6's four fields — entry content hash, entry adapter —
/// are NOT duplicated here (F-SI-01): they are already columns on the
/// resource's own row for this same
/// `(generation_id, source_name, relative_path)`, and
/// [`AtlasDb::child_resources`] — the canned read behind
/// `GET /v1/map/children` and `sgt map children` — joins back to it there
/// rather than being handed a second, driftable copy. That reader is what
/// makes the non-duplication a design instead of a hole (S5 W7 F-SF-03):
/// before it existed these rows were written and read by nothing, and A1-15
/// keeps arbitrary client SQL off the surface, so the coordinate was
/// reachable only by opening the database file directly.
fn insert_child_resource(
    conn: &impl Statements,
    generation_id: &str,
    source_name: &str,
    relative_path: &str,
    key: &str,
    parent: &crate::runtime::atlas::scan::ChildProvenance,
) -> Result<(), AtlasError> {
    conn.prepare_cached(sql!(
        "INSERT INTO source.child_resources \
         (generation_id, source_name, relative_path, local_key, parent_relative_path, \
          parent_key, entry_path) \
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    ))?
    .execute(duckdb::params![
        generation_id,
        source_name,
        relative_path,
        key,
        &parent.parent_relative_path,
        &parent.parent_key,
        &parent.entry_path,
    ])?;
    Ok(())
}

/// Insert one derived-evidence row (A1 §6.4).
fn insert_dataset_fact(
    conn: &impl Statements,
    generation_id: &str,
    source_name: &str,
    fact: &DatasetFact,
) -> Result<(), AtlasError> {
    conn.prepare_cached(sql!(
        "INSERT INTO source.dataset_facts \
         (generation_id, source_name, relative_path, dataset_key, query, query_identity, \
          row_limit, truncated, columns, rows, output_hash, observed_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ))?
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
    conn: &impl Statements,
    generation_id: &str,
    source_name: &str,
    dataset: &ScannedDataset,
    unit: &RowUnit,
) -> Result<(), AtlasError> {
    conn.prepare_cached(sql!(
        "INSERT INTO context.row_units \
         (generation_id, source_name, relative_path, dataset_key, ordinal, row_key, key_basis, \
          fields, body) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ))?
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

/// S5 W2 — derive one generation's lexical index from the rows it just
/// staged (A2 §5, decision A2-05).
///
/// **Derived from the stored rows, not from the in-memory scan.** A hit cites
/// a `source.occurrences`/`source.units`/`context.row_units` row; deriving
/// the postings from those same rows is what makes the citation checkable
/// rather than parallel. It also means [`AtlasDb::reindex_lexical`] and this
/// call are the same derivation, so a rebuilt index cannot disagree with a
/// freshly built one.
///
/// **The three families, and where each one's text comes from** (A2 §17 item
/// 2's four, less the document/mail split that one query makes):
///
/// * [`LexicalFamily::Document`] / [`LexicalFamily::Mail`] — `source.units`,
///   joined to `source.files` for the extractor identity that decides which
///   of the two it is ([`DOCUMENT_EXTRACTOR_IDENTITIES`], with
///   [`crate::runtime::atlas::mail::MAIL_EXTRACTOR`] the mail half). Indexed
///   text is the unit's title and body — so these families carry every
///   ordinary word of the prose, which is A2 §5's *"Document/mail retrieval
///   additionally retains ordinary natural-language tokens"*.
/// * [`LexicalFamily::Code`] — `source.occurrences`, the grammar-claimed
///   definition sites, matched on [`CODE_EXTRACTOR_LIKE`] exactly as
///   [`AtlasDb::admissible_occurrences`] matches them. Indexed text is the
///   symbol name and nothing else: A2 §5's BM25 is *"tuned for
///   identifier/document tokens"*, and [EXT-SEMBLE] validates it
///   *"particularly for identifiers/API names"*. A code file's prose is not
///   lost by that choice — `claims_for` gives every grammar-claimed file a
///   plain-text fallback unit, which is indexed by the document family above.
/// * [`LexicalFamily::RowText`] — `context.row_units`, A1's F10a-gated
///   selected-row text. Indexed text is the assembled body.
///
/// **Unbounded on purpose, and this is the one read in this module that is.**
/// [`MAX_ROWS`] is a refusal to build an unbounded result set for a *caller*;
/// these three reads run inside the staging transaction over rows the same
/// transaction just wrote from a scan whose units were already all in memory,
/// so a cap here would add no new bound — it would only drop units out of the
/// index silently, which is exactly the reported-never-silent discipline this
/// wave is required to keep.
/// Every indexable unit of one generation, in the order the three family
/// reads produce them — **the one place a generation's retrievable text is
/// derived**, so the lexical index ([`index_generation`]) and the semantic
/// scan ([`AtlasDb::semantic_search`]) cannot drift into two different
/// corpora.
///
/// Extracted from `index_generation` by S5 W3b for exactly that reason (R2):
/// A2-02 forbids *"a second chunk/source identity system"*, and the cheapest
/// way to keep one is to have one function that produces it. A2 §16's
/// *"embedding raw binary documents rather than A1 evidence units"* non-goal
/// is satisfied the same way — what this returns IS A1's evidence units, and
/// the semantic half has no other way to reach text.
fn indexable_units(
    conn: &impl Statements,
    generation_id: &str,
) -> Result<Vec<IndexableUnit>, AtlasError> {
    let mut units: Vec<IndexableUnit> = Vec::new();

    let [doc_a, doc_b, doc_c] = DOCUMENT_EXTRACTOR_IDENTITIES;
    let mut statement = conn.prepare(sql!(
        "SELECT u.source_name, u.relative_path, u.ordinal, u.title, u.byte_start, u.byte_end, \
                u.body, f.extractor, c.coordinate \
         FROM source.units u \
         JOIN source.files f ON f.generation_id = u.generation_id \
                             AND f.relative_path = u.relative_path \
         LEFT JOIN source.unit_coordinates c ON c.generation_id = u.generation_id \
                                             AND c.relative_path = u.relative_path \
                                             AND c.ordinal = u.ordinal \
         WHERE u.generation_id = ? \
           AND (f.extractor IN (?, ?, ?) OR f.extractor LIKE ?) \
         ORDER BY u.relative_path, u.ordinal"
    ))?;
    let mut rows = statement.query(duckdb::params![
        generation_id,
        doc_a,
        doc_b,
        doc_c,
        DOCUMENT_EXTRACTOR_LIKE
    ])?;
    while let Some(row) = rows.next()? {
        let extractor: String = row.get(7)?;
        let family = if extractor == crate::runtime::atlas::mail::MAIL_EXTRACTOR {
            LexicalFamily::Mail
        } else {
            LexicalFamily::Document
        };
        let relative_path: String = row.get(1)?;
        let ordinal = row.get::<usize, i64>(2)? as u64;
        let title: Option<String> = row.get(3)?;
        let body: String = row.get(6)?;
        let text = match &title {
            Some(title) => format!("{title}\n{body}"),
            None => body,
        };
        units.push(IndexableUnit {
            source_name: row.get(0)?,
            family,
            unit_key: unit_key(family, &relative_path, ordinal),
            relative_path,
            ordinal,
            title,
            symbol: None,
            language: None,
            label: None,
            dataset_key: None,
            row_key: None,
            fields: None,
            byte_start: Some(row.get::<usize, i64>(4)? as u64),
            byte_end: Some(row.get::<usize, i64>(5)? as u64),
            native: row.get(8)?,
            text,
        });
    }
    drop(rows);
    drop(statement);

    let mut statement = conn.prepare(sql!(
        "SELECT source_name, relative_path, ordinal, language, label, name, byte_start, byte_end \
         FROM source.occurrences \
         WHERE generation_id = ? AND extractor LIKE ? \
         ORDER BY relative_path, ordinal"
    ))?;
    let mut rows = statement.query(duckdb::params![generation_id, CODE_EXTRACTOR_LIKE])?;
    while let Some(row) = rows.next()? {
        let relative_path: String = row.get(1)?;
        let ordinal = row.get::<usize, i64>(2)? as u64;
        let name: String = row.get(5)?;
        units.push(IndexableUnit {
            source_name: row.get(0)?,
            family: LexicalFamily::Code,
            unit_key: unit_key(LexicalFamily::Code, &relative_path, ordinal),
            relative_path,
            ordinal,
            title: None,
            symbol: Some(name.clone()),
            language: Some(row.get(3)?),
            label: Some(row.get(4)?),
            dataset_key: None,
            row_key: None,
            fields: None,
            byte_start: Some(row.get::<usize, i64>(6)? as u64),
            byte_end: Some(row.get::<usize, i64>(7)? as u64),
            native: None,
            text: name,
        });
    }
    drop(rows);
    drop(statement);

    let mut statement = conn.prepare(sql!(
        "SELECT source_name, relative_path, dataset_key, ordinal, row_key, fields, body \
         FROM context.row_units WHERE generation_id = ? ORDER BY relative_path, ordinal"
    ))?;
    let mut rows = statement.query(duckdb::params![generation_id])?;
    while let Some(row) = rows.next()? {
        let relative_path: String = row.get(1)?;
        let ordinal = row.get::<usize, i64>(3)? as u64;
        units.push(IndexableUnit {
            source_name: row.get(0)?,
            family: LexicalFamily::RowText,
            unit_key: unit_key(LexicalFamily::RowText, &relative_path, ordinal),
            relative_path,
            ordinal,
            title: None,
            symbol: None,
            language: None,
            label: None,
            dataset_key: Some(row.get(2)?),
            row_key: Some(row.get(4)?),
            fields: Some(row.get(5)?),
            byte_start: None,
            byte_end: None,
            native: None,
            text: row.get(6)?,
        });
    }
    drop(rows);
    drop(statement);

    Ok(units)
}

fn index_generation(conn: &impl Statements, generation_id: &str) -> Result<u64, AtlasError> {
    let units = indexable_units(conn, generation_id)?;

    // The two batches, appended rather than inserted row by row. This file
    // already carries the measurement that decides it (see [`Analytics`]'s
    // own doc): on this container a single-row DuckDB `INSERT` costs ~1-2 ms
    // and an appended row ~4 us. A real repository scan produces tens of
    // thousands of postings, so row-at-a-time `INSERT` here turned an 11 s
    // estate scan into one that blew a 100 s client timeout
    // (`tests/y6a_estate_scoped_scan.rs`) before this was measured and fixed;
    // the appender is R2 — the mechanism this file already owns, reused.
    let mut unit_rows: Vec<Vec<Duck>> = Vec::with_capacity(units.len());
    let mut posting_rows: Vec<Vec<Duck>> = Vec::new();
    for unit in &units {
        let (frequencies, token_count) = term_frequencies(&unit.text);
        unit_rows.push(vec![
            Duck::Text(generation_id.to_string()),
            Duck::Text(unit.source_name.clone()),
            Duck::Text(unit.family.as_str().to_string()),
            Duck::Text(unit.unit_key.clone()),
            Duck::Text(unit.relative_path.clone()),
            Duck::BigInt(unit.ordinal as i64),
            optional_text(unit.title.as_deref()),
            optional_text(unit.symbol.as_deref()),
            optional_text(unit.language.as_deref()),
            optional_text(unit.label.as_deref()),
            optional_text(unit.dataset_key.as_deref()),
            optional_text(unit.row_key.as_deref()),
            optional_text(unit.fields.as_deref()),
            unit.byte_start
                .map_or(Duck::Null, |v| Duck::BigInt(v as i64)),
            unit.byte_end.map_or(Duck::Null, |v| Duck::BigInt(v as i64)),
            Duck::BigInt(token_count as i64),
        ]);
        for (term, frequency) in &frequencies {
            posting_rows.push(vec![
                Duck::Text(generation_id.to_string()),
                Duck::Text(unit.unit_key.clone()),
                Duck::Text(term.clone()),
                Duck::BigInt(*frequency as i64),
            ]);
        }
    }
    let indexed = unit_rows.len() as u64;
    append_rows(
        conn,
        name!(CONTEXT_SCHEMA),
        name!("lexical_units"),
        unit_rows,
    )?;
    append_rows(
        conn,
        name!(CONTEXT_SCHEMA),
        name!("lexical_postings"),
        posting_rows,
    )?;
    Ok(indexed)
}

/// **A2 §17 item 8**, lexical half: the `g.source_kind` column of one
/// [`LEXICAL_POSTINGS_SQL`] row.
///
/// Refuses an unrecognized spelling rather than defaulting, exactly as
/// [`AtlasDb::admissible_generations`] does for the same column — this value
/// is what tells a consumer that a hit is external, and a wrong default here
/// would make external evidence *invisibly* external, which is the failure
/// item 8 names.
fn source_kind_at(row: &duckdb::Row<'_>, index: usize) -> Result<SourceKind, AtlasError> {
    let text: String = row.get(index)?;
    SourceKind::parse(&text).ok_or(AtlasError::UnknownValue {
        column: "source_kind".to_string(),
        value: text,
    })
}

/// **A2 §17 item 8**, the other column — see [`source_kind_at`].
fn authority_class_at(row: &duckdb::Row<'_>, index: usize) -> Result<AuthorityClass, AtlasError> {
    let text: String = row.get(index)?;
    AuthorityClass::parse(&text).ok_or(AtlasError::UnknownValue {
        column: "authority_class".to_string(),
        value: text,
    })
}

/// A2 §3's family-shaped coordinate, read off one
/// [`LEXICAL_POSTINGS_SQL`] row.
///
/// The column set is one row shape for four coordinate shapes, because the
/// families share a table; `family` decides which columns are meaningful, and
/// the ones that are not are `NULL` by construction (see
/// [`index_generation`], the only writer). A missing value defaults rather
/// than failing: a coordinate is evidence about where a hit came from, and a
/// row whose `language` is somehow absent should still cite its path and span
/// rather than take the whole answer down.
fn coordinate_of(row: &duckdb::Row<'_>) -> Result<UnitCoordinate, AtlasError> {
    let family_name: String = row.get(3)?;
    let family = LexicalFamily::parse(&family_name).ok_or_else(|| AtlasError::UnknownValue {
        column: "family".to_string(),
        value: family_name.clone(),
    })?;
    let relative_path: String = row.get(5)?;
    let ordinal = row.get::<usize, i64>(6)? as u64;
    let title: Option<String> = row.get(7)?;
    let byte_start = row.get::<usize, Option<i64>>(14)?.unwrap_or(0) as u64;
    let byte_end = row.get::<usize, Option<i64>>(15)?.unwrap_or(0) as u64;
    // A2 §9's native coordinate, `LEFT JOIN`ed on the posting row: `None`
    // for a unit whose byte span is its address, and for every family that
    // does not come from `source.units` at all.
    let native: Option<String> = row.get(20)?;
    Ok(match family {
        LexicalFamily::Code => UnitCoordinate::Code {
            relative_path,
            language: row.get::<usize, Option<String>>(9)?.unwrap_or_default(),
            label: row.get::<usize, Option<String>>(10)?.unwrap_or_default(),
            symbol: row.get::<usize, Option<String>>(8)?.unwrap_or_default(),
            ordinal,
            byte_start,
            byte_end,
        },
        LexicalFamily::Document => UnitCoordinate::Document {
            relative_path,
            ordinal,
            title,
            byte_start,
            byte_end,
            native,
        },
        LexicalFamily::Mail => UnitCoordinate::Mail {
            relative_path,
            ordinal,
            title,
            byte_start,
            byte_end,
            native,
        },
        LexicalFamily::RowText => UnitCoordinate::RowText {
            relative_path,
            dataset_key: row.get::<usize, Option<String>>(11)?.unwrap_or_default(),
            ordinal,
            row_key: row.get::<usize, Option<String>>(12)?.unwrap_or_default(),
            fields: row
                .get::<usize, Option<String>>(13)?
                .as_deref()
                .map(split_names)
                .unwrap_or_default(),
        },
    })
}

/// The index's own per-generation unit identity: `<family>:<path>#<ordinal>`.
///
/// The family prefix is load-bearing rather than decorative — one `.rs` file
/// produces both a `source.occurrences` row at ordinal 0 and (through
/// `claims_for`'s plain-text fallback) a `source.units` row at ordinal 0, and
/// without the prefix those two distinct pieces of evidence would collide on
/// one key and one would silently overwrite the other's postings.
fn unit_key(family: LexicalFamily, relative_path: &str, ordinal: u64) -> String {
    format!("{}:{relative_path}#{ordinal}", family.as_str())
}

/// One unit on its way into the index — the stored row plus the text that
/// row contributes.
struct IndexableUnit {
    source_name: String,
    family: LexicalFamily,
    unit_key: String,
    relative_path: String,
    ordinal: u64,
    title: Option<String>,
    symbol: Option<String>,
    language: Option<String>,
    label: Option<String>,
    dataset_key: Option<String>,
    row_key: Option<String>,
    fields: Option<String>,
    byte_start: Option<u64>,
    byte_end: Option<u64>,
    /// A2 §9's native coordinate for this unit, when the adapter produced
    /// one — `source.unit_coordinates`, joined in by [`indexable_units`].
    native: Option<String>,
    text: String,
}

impl IndexableUnit {
    /// A1's coordinate for this unit — the same value
    /// [`coordinate_of`] reconstructs from a stored
    /// `context.lexical_units` row, built here from the row the unit was
    /// derived from instead.
    ///
    /// Two constructions of one value is a drift risk, so it is pinned:
    /// `tests/w3b_semantic_retrieval.rs::
    /// a_semantic_hit_and_a_lexical_hit_on_the_same_unit_carry_the_identical_coordinate`
    /// runs both paths over one generation and compares them.
    fn coordinate(&self) -> UnitCoordinate {
        match self.family {
            LexicalFamily::Code => UnitCoordinate::Code {
                relative_path: self.relative_path.clone(),
                language: self.language.clone().unwrap_or_default(),
                label: self.label.clone().unwrap_or_default(),
                symbol: self.symbol.clone().unwrap_or_default(),
                ordinal: self.ordinal,
                byte_start: self.byte_start.unwrap_or(0),
                byte_end: self.byte_end.unwrap_or(0),
            },
            LexicalFamily::Document => UnitCoordinate::Document {
                relative_path: self.relative_path.clone(),
                ordinal: self.ordinal,
                title: self.title.clone(),
                byte_start: self.byte_start.unwrap_or(0),
                byte_end: self.byte_end.unwrap_or(0),
                native: self.native.clone(),
            },
            LexicalFamily::Mail => UnitCoordinate::Mail {
                relative_path: self.relative_path.clone(),
                ordinal: self.ordinal,
                title: self.title.clone(),
                byte_start: self.byte_start.unwrap_or(0),
                byte_end: self.byte_end.unwrap_or(0),
                native: self.native.clone(),
            },
            LexicalFamily::RowText => UnitCoordinate::RowText {
                relative_path: self.relative_path.clone(),
                dataset_key: self.dataset_key.clone().unwrap_or_default(),
                ordinal: self.ordinal,
                row_key: self.row_key.clone().unwrap_or_default(),
                fields: self.fields.as_deref().map(split_names).unwrap_or_default(),
            },
        }
    }
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

/// **S6 D1 — record A2 §2 stage 1's estate coordinate for one generation.**
///
/// Idempotent by construction: the row is written only when this exact
/// `(generation_id, scope, root)` claim is not already there, so a re-scan
/// that lands on `ScanCommit::Unchanged` does not accumulate duplicates and
/// a second estate's identical world adds exactly one row.
///
/// The absence of a row is meaningful and is *not* repaired here: a
/// generation staged by a build older than S6 D1 has none, and is
/// inadmissible from every estate until it is re-scanned. Backfilling one
/// would mean inventing an estate for evidence whose origin was never
/// recorded — the invented answer A2 §2's "never approximate" forbids, and
/// on the confidentiality axis the expensive direction to be wrong in.
fn bind_generation_estate(
    conn: &impl Statements,
    generation_id: &str,
    estate: &EstateBinding,
) -> Result<(), AtlasError> {
    let (scope, root) = estate.columns();
    conn.prepare_cached(sql!(
        "INSERT INTO source.generation_estates (generation_id, estate_scope, estate_root) \
         SELECT ?, ?, ? WHERE NOT EXISTS ( \
           SELECT 1 FROM source.generation_estates \
            WHERE generation_id = ? AND estate_scope = ? \
              AND estate_root IS NOT DISTINCT FROM ?)"
    ))?
    .execute(duckdb::params![
        generation_id,
        scope,
        root,
        generation_id,
        scope,
        root
    ])?;
    Ok(())
}

/// Insert one coverage observation.
fn insert_coverage(
    conn: &impl Statements,
    generation_id: &str,
    source_name: &str,
    row: &CoverageRow,
    observed_at: &str,
) -> Result<(), AtlasError> {
    conn.prepare_cached(sql!(
        "INSERT INTO meta.coverage \
         (generation_id, source_name, path, status, detail, bytes, observed_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    ))?
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
    conn: &impl Statements,
    generation_id: &str,
    source_name: &str,
    reason: &str,
    observed_at: &str,
) -> Result<(), AtlasError> {
    conn.execute(
        sql!("DELETE FROM source.units WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM source.unit_coordinates WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM source.occurrences WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM source.edges WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM source.symbols WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM source.child_resources WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM source.files WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM source.datasets WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM source.dataset_facts WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    // The F10a-gated units go with everything else an eviction takes. That
    // matters more here than elsewhere: narrowing a source's `context_fields`
    // changes the reader identity, which stages a new generation, which
    // evicts this one — and *this* delete is what actually retracts the text
    // the wider allowlist exposed.
    conn.execute(
        sql!("DELETE FROM context.row_units WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    // S5 W2: the lexical index is derived evidence over the rows deleted
    // above, so it goes with them. This is the whole of "keyed by
    // SourceGeneration, so a superseded generation's postings are evicted
    // with it" — postings first, because a posting whose unit row is already
    // gone is an orphan, and there is no window in which one exists (this
    // runs inside the caller's transaction).
    conn.execute(
        sql!("DELETE FROM context.lexical_postings WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM context.lexical_units WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    // A no-op DELETE for every non-`external_git` generation (the table has
    // no row to match), and the eviction half of `git.provenance`'s own
    // atomicity promise for one that is: a superseded external source's old
    // origin/ref/commit does not linger once its rows are gone.
    conn.execute(
        sql!("DELETE FROM git.provenance WHERE generation_id = ?"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("DELETE FROM meta.coverage WHERE generation_id = ? AND path IS NOT NULL"),
        duckdb::params![generation_id],
    )?;
    conn.execute(
        sql!("UPDATE source.generations SET state = ? WHERE generation_id = ?"),
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

/// One stored unit **plus the provenance a prompt excerpt of it must carry**
/// — C1 §21 item 8, and [`AtlasDb::resolve_unit`]'s answer.
///
/// The two extra fields are not on [`StoredUnit`] because they are not the
/// unit's own row: they belong to the resource it came from and to A2 §9's
/// native-coordinate table, and copying them onto every listing read
/// ([`AtlasDb::units`], [`AtlasDb::outline`]) would make two of the three
/// readers pay a join their callers do not use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedUnit {
    /// The unit's own stored row.
    pub unit: StoredUnit,
    /// The extractor identity that produced this resource's units — §12's
    /// *"normalizer identity"*. `None` only when the resource row is missing,
    /// which would be a defect rather than an ordinary answer.
    pub extractor: Option<String>,
    /// A2 §9's native coordinate for this unit — the normalizer's own address
    /// inside the document it was extracted from (an Office block address, a
    /// mail body selector), when the byte span cannot be one. `None` for a
    /// Markdown/text unit, whose span *is* its address.
    pub native_coordinate: Option<String>,
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

/// The relationship kind C1 §5 step 3 contributes for a container/document/
/// mail parent-child row, and the one value of
/// [`AtlasDb::resolve_relationship`]'s `kind` that reads
/// `source.child_resources` rather than `source.edges`.
///
/// Declared here rather than spelled twice: the producer is
/// [`crate::runtime::context`]'s `child_relationship` and the consumer is the
/// resolver, and a literal in each is exactly the drift that would make a
/// coordinate unresolvable while both files still looked right.
pub const CHILD_RESOURCE_RELATIONSHIP: &str = "child_resource";

/// One stored relationship, re-resolved from its exact coordinate by
/// [`AtlasDb::resolve_relationship`].
///
/// Deliberately not [`StoredEdge`] or [`StoredChildResource`]: the two tables
/// answer with different columns, and the thing item 4 asks for is that the
/// pinned coordinate *resolves to the row it names* — the ends, and whatever
/// one extra fact that table holds about them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRelationship {
    /// The relationship's kind, as the coordinate named it.
    pub kind: String,
    /// The end it leaves from.
    pub from: String,
    /// The end it names — **unresolved** for a syntax edge, exactly as
    /// [`StoredEdge::target`] is.
    pub to: String,
    /// Position within its file's edge list, for an edge. `None` for a
    /// child-resource row, which has no ordinal.
    pub ordinal: Option<u64>,
    /// The one extra fact the storing table holds: §6.6's entry path for a
    /// child resource, the grammar's language for an edge.
    pub detail: Option<String>,
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

/// One container child of a confirmed generation, as
/// [`AtlasDb::child_resources`] answers it — A1 §6.6's four preserved
/// fields, with the parent coordinate from `source.child_resources` and the
/// entry content hash / entry adapter joined back from the resource's own
/// row (S5 W7 F-SF-03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredChildResource {
    /// The child's own composed path, e.g. `bundle.zip!/notes/a.md`.
    pub relative_path: String,
    /// Its own F7 key — `local_key` for a file, `dataset_key` for a dataset.
    pub key: String,
    /// §6.6's "parent archive source/resource": the IMMEDIATE parent's own
    /// composed path, chained rather than resolved to the root container.
    pub parent_relative_path: String,
    /// That parent's own F7 key.
    pub parent_key: String,
    /// §6.6's "entry path", relative to that immediate parent.
    pub entry_path: String,
    /// §6.6's "entry content hash", read back from the resource's own row.
    pub content_hash: Option<String>,
    /// §6.6's "entry adapter", read back from the resource's own row.
    pub extractor: Option<String>,
    /// Which table holds the resource this row names: `file` or `dataset`.
    pub lane: Option<String>,
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
    /// Stage 1's **estate** half — `--source`/`--source@sha`/`--work` name
    /// a source, and this names the *estate whose world those are read in*.
    ///
    /// **The one default-DENY axis in this struct, and it must stay that
    /// way.** [`Self::kind`] and [`Self::authority`] below both say "`None`
    /// admits every value — narrows only what a caller explicitly asked to
    /// narrow"; this field is the deliberate opposite, because an estate
    /// axis that defaulted to admit-everything would have left S6 D1's
    /// measured cross-estate leak untouched (every consumer omitted the
    /// estate — that omission is the defect). [`EstateAdmission::NoEstate`]
    /// is the `Default` and admits **nothing**;
    /// [`EstateAdmission::Estate`] admits one canonical estate root plus
    /// the generations recorded
    /// [`EstateBinding::Host`](crate::domain::source::EstateBinding::Host).
    /// There is no "every estate" value. See
    /// [`EstateAdmission`](crate::domain::source::EstateAdmission).
    pub estate: EstateAdmission,
    /// Stage 1: which source(s) may be seen at all — `--source`/
    /// `--source@sha`/`--work`, or none of those (every source *within
    /// [`Self::estate`]*, never across estates).
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

impl Admissibility {
    /// Everything one estate may see: [`EstateAdmission::Estate`] on the
    /// estate axis, and every other axis left at its own admit-everything
    /// default.
    ///
    /// This — not `Admissibility::default()` — is the ordinary starting
    /// point for a filter. `default()` is deliberately *empty*: its estate
    /// axis is [`EstateAdmission::NoEstate`], which admits nothing, so a
    /// construction site that forgets the estate fails closed instead of
    /// reading every estate on the host.
    pub fn within_estate(estate_root: impl Into<String>) -> Self {
        Self {
            estate: EstateAdmission::Estate(estate_root.into()),
            ..Self::default()
        }
    }
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
/// overlay is produced by a daemon-side hook (H13.2's chosen mechanism,
/// wired in S5 W1b: [`crate::api`]'s surface-lifecycle arm) which fires
/// when the surface is **bound** — materialized, or re-materialized for a
/// retry — when **a turn ends** while it is still bound (S5 W1d's
/// addition, and the only one of the three that can record an overlay
/// describing anything: a freshly cut worktree is byte-identical to its
/// base), and is evicted when the surface is torn down. It is therefore a
/// **snapshot**, never a live read: nothing rescans *between* those
/// moments, a turn still in flight is a tree still being written, and `sgt
/// search` stays a pure reader that never touches the surface at all
/// (H13.2 rejected query-time scanning precisely so it could not).
///
/// So [`Self::BaseAndOverlaySnapshot`] carries the snapshot's own
/// `observed_at`. A caller renders it. An answer that said "including
/// overlay" without saying *as of when* would imply "current" while
/// meaning "as of the end of the Work's last completed turn" — the same
/// class of false claim as the silent partial W1 refused to ship.
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

/// One lexical query: A2 §5's text, inside A2 §2's world.
///
/// The filter is a **borrowed** [`Admissibility`] rather than an owned copy
/// so a caller cannot end up ranking against a filter that differs from the
/// one it already used to decide the world — there is one filter value, and
/// both stages read it.
#[derive(Debug, Clone)]
pub struct LexicalQuery<'a> {
    /// The raw query text, tokenized by
    /// [`crate::runtime::atlas::lexical::query_terms`].
    pub text: &'a str,
    /// A2 §2's deterministic admissibility filter — applied FIRST, in SQL,
    /// so ranking happens inside the admissible set and can never widen it
    /// (A2 §8).
    pub filter: &'a Admissibility,
    /// Optionally narrow to one of A2 §17 item 2's four families. `None`
    /// searches all four, which is the default a caller who asked for no
    /// narrowing should get.
    pub family: Option<LexicalFamily>,
    /// How many hits to return, capped at [`MAX_ROWS`] (F12).
    pub limit: usize,
    /// Whether the caller wants A2 §6's semantic half used at all
    /// (decision **H4**). Required, and deliberately not defaulted: a caller
    /// that never states it cannot later claim it did not know its answer was
    /// lexical-only. What it resolves to on the answer is
    /// [`LexicalAnswer::semantic`].
    pub semantic: SemanticRequest,
}

/// What one [`AtlasDb::lexical_search`] answered, with everything a caller
/// must be able to state about it.
///
/// Five fields, none decorative: the ranked hits, the [`WorkScope`] every
/// `--work`-filtered answer has to render (see that type's own doc), whether
/// the posting scan hit its cap, and S5 W3's two semantic fields. A capped
/// scan is not a shorter list — it is a list whose *scores* were computed
/// over fewer postings than exist, so it is a different answer and says so.
///
/// # The two semantic fields are two fields on purpose (decision H4)
///
/// [`Self::semantic`] is **required** and [`Self::semantic_model`] is
/// **optional**, and collapsing them would destroy the property H4 exists
/// for. A2 §15 requires a degraded answer to *"report that
/// coverage/capability honestly"*; H4 makes that mechanical — a consumer
/// reads a value that is always present rather than inferring degradation
/// from a missing model identity, which cannot distinguish "no model
/// installed" from "the caller turned it off" from "nobody filled the field
/// in". See [`crate::runtime::atlas::semantic`] for the full argument and
/// the test that pins it.
#[derive(Debug, Clone, PartialEq)]
pub struct LexicalAnswer {
    /// The ranked hits, best first, ties broken by
    /// [`crate::runtime::atlas::lexical::LexicalHit::tie_break_key`].
    pub hits: Vec<LexicalHit>,
    /// What this answer covers — see [`WorkScope`].
    pub scope: WorkScope,
    /// Whether the posting scan reached [`MAX_ROWS`] and stopped.
    pub truncated: bool,
    /// A2 §15's required honesty about the semantic half: `applied`,
    /// `not_installed` or `disabled` (decision **H4**). Never an `Option` —
    /// there is no "unset" for a consumer to misread as "fine".
    pub semantic: SemanticStatus,
    /// A2 §13's *"semantic model identity/hash **if used**"* — populated
    /// only when [`Self::semantic`] is
    /// [`SemanticStatus::Applied`]. Optional because the contract says
    /// "if used"; it is **not** the field that reports degradation.
    pub semantic_model: Option<SemanticModel>,
}

/// One semantic answer: A2 §6's cosine rank list, [`WorkScope`], whether the
/// scan hit its cap, and the same two H4 fields [`LexicalAnswer`] carries.
///
/// **The two H4 fields are here for the same reason they are there, and they
/// are load-bearing on this type in a way they are not on the lexical one:**
/// an empty `hits` on a semantic answer is ambiguous by itself — the corpus
/// held nothing similar, no model is installed, or the caller suppressed the
/// half. [`Self::semantic`] is the field that says which, and it is not an
/// `Option`.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticAnswer {
    /// The ranked hits, best first, ties broken by
    /// [`crate::runtime::atlas::semantic::SemanticHit::tie_break_key`] —
    /// the same stated key the lexical list uses, because these are RRF's
    /// two inputs.
    pub hits: Vec<SemanticHit>,
    /// What this answer covers — see [`WorkScope`].
    pub scope: WorkScope,
    /// Whether the unit scan reached [`MAX_ROWS`] and stopped.
    pub truncated: bool,
    /// A2 §15's required honesty (decision **H4**). On this type it is also
    /// the only thing that distinguishes "nothing scored" from "nothing
    /// ran".
    pub semantic: SemanticStatus,
    /// A2 §13's *"semantic model identity/hash **if used**"*, populated only
    /// when [`Self::semantic`] is [`SemanticStatus::Applied`].
    pub semantic_model: Option<SemanticModel>,
}

/// What one [`AtlasDb::fused_search`] answered: A2 §7's fused, A2 §8's
/// reranked list, plus **the same four disclosure fields the two half-answers
/// carry**.
///
/// The four are not copied out of tidiness. A fused answer is degraded in
/// exactly the ways its inputs were, and a consumer that could not see that
/// would read a lexical-only list as a fused one:
///
/// * [`Self::scope`] — `--work`'s overlay half is a snapshot, and a fused
///   answer built over it inherits that ([`WorkScope`]).
/// * [`Self::truncated`] — **either** half hitting [`MAX_ROWS`] truncates
///   this answer, because a capped input list changes `rank_i(d)` for every
///   candidate below the cap and therefore changes the fused score, not just
///   the length of a list.
/// * [`Self::semantic`] / [`Self::semantic_model`] — decision **H4**. When
///   the status is not [`SemanticStatus::Applied`] this "fused" answer fused
///   one list with an empty one, which is A2 §15's degraded state and must be
///   readable as such rather than inferred from a suspiciously lexical-looking
///   ordering.
#[derive(Debug, Clone, PartialEq)]
pub struct FusedAnswer {
    /// The reranked hits, best first — A2 §7's score then A2 §8's signals,
    /// ties broken by
    /// [`crate::runtime::atlas::fusion::FusedHit::tie_break_key`].
    pub hits: Vec<FusedHit>,
    /// What this answer covers — see [`WorkScope`].
    pub scope: WorkScope,
    /// Whether **either** input list hit [`MAX_ROWS`] and stopped.
    pub truncated: bool,
    /// A2 §15's required honesty about the semantic half (decision **H4**).
    pub semantic: SemanticStatus,
    /// A2 §13's *"semantic model identity/hash **if used**"*.
    pub semantic_model: Option<SemanticModel>,
}

/// One `sgt related <coordinate>` request: which unit to find neighbours of,
/// inside which A2 §2 world.
///
/// A separate type from [`LexicalQuery`] because the *query text* is not the
/// caller's — it is the anchor unit's own indexed text, which
/// [`AtlasDb::related`] reads out of the store. A caller cannot supply it,
/// and a type that let one do so would be a client-supplied pattern by
/// another name.
#[derive(Debug, Clone)]
pub struct RelatedRequest<'a> {
    /// The declared source the anchor unit belongs to.
    pub source_name: &'a str,
    /// Atlas's own per-generation unit identity, `<family>:<path>#<ordinal>`
    /// — the value every hit already carries as `unit_key`, so a coordinate
    /// printed by `sgt search` is a coordinate `sgt related` accepts.
    pub unit_key: &'a str,
    /// A2 §2's deterministic admissibility filter, applied to the anchor
    /// lookup **and** to the neighbours.
    pub filter: &'a Admissibility,
    /// Optionally narrow the neighbours to one family (`--content`).
    pub family: Option<LexicalFamily>,
    /// How many neighbours to return, capped at [`MAX_ROWS`].
    pub limit: usize,
    /// Decision **H4**: whether the caller wants A2 §6's half at all.
    pub semantic: SemanticRequest,
    /// A2 §13 field 2's attribution for the trace this produces.
    pub attribution: Attribution,
}

/// The unit a [`RelatedRequest`] was anchored on, as it was actually
/// resolved — echoed back so an answer states what it is about rather than
/// leaving a caller to trust that its coordinate parsed the way it meant.
///
/// Carries A2 §17 item 8's two fields for the same reason every hit does: an
/// anchor in an external source produces neighbours whose externality a
/// consumer must be able to see.
#[derive(Debug, Clone, PartialEq)]
pub struct RelatedAnchor {
    /// The declared source.
    pub source_name: String,
    /// **A2 §17 item 8** — the source's kind.
    pub source_kind: SourceKind,
    /// **A2 §17 item 8** — the source's authority class.
    pub authority_class: AuthorityClass,
    /// The exact SourceGeneration the anchor was resolved in.
    pub generation_id: String,
    /// That generation's content identity.
    pub content_key: String,
    /// Atlas's own per-generation unit identity.
    pub unit_key: String,
    /// A1's coordinate for the anchor unit.
    pub coordinate: UnitCoordinate,
}

/// What one [`AtlasDb::related`] answered: the resolved anchor, the
/// neighbours (the anchor itself removed), and A2 §13's trace.
#[derive(Debug, Clone, PartialEq)]
pub struct RelatedAnswer {
    /// The unit the neighbours are neighbours *of*.
    pub anchor: RelatedAnchor,
    /// The neighbours, best first — the same [`FusedAnswer`] shape `sgt
    /// search` returns, disclosure fields included.
    pub answer: FusedAnswer,
    /// A2 §13's trace. Its `query.text` is the **anchor's own text**, which
    /// is what was actually retrieved on; recording the caller's coordinate
    /// there instead would make the trace unreproducible.
    pub trace: SearchTrace,
}
/// **The admissibility filter, over a handle that cannot write** — H13.2's
/// "`sgt search` is a pure reader", enforced by the type system rather than
/// by a scan of this file's text.
///
/// Every A2 §2 content-family query lives here, and the only thing this
/// struct holds is a [`ReadOnly`]: no `Connection`, no `Store`, no
/// `Transaction`, no `&self` route back to one. A write inside any method
/// below is therefore not a forbidden spelling — it is a **compile error**,
/// because there is no value in scope with a write on it. That is the
/// difference the S5 closeout bought: the previous guarantee was a
/// source-text scan of `admissible_*` bodies, and a `format!`-assembled
/// `DELETE` walked straight past it (`tests/w1b_overlay_lifecycle_trigger.rs`
/// tells that story, and now stands as the *second* net, not the first).
///
/// [`AtlasDb`]'s `admissible_*` methods are one-line delegates onto this
/// type. They keep `&self` — a caller cannot write through them either — but
/// `&self` was never the guarantee: DuckDB's `Connection::execute`,
/// `execute_batch` and `prepare` all take `&self`.
struct Admissible<'conn> {
    reader: ReadOnly<'conn>,
}

impl Admissible<'_> {
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
    fn generations(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<SourceGeneration>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let (source_name, content_key) = filter.source.bindings();
        let source_kind = filter.kind.map(SourceKind::as_str);
        let authority = filter.authority.map(AuthorityClass::as_str);
        // A2 §2 stage 1's estate axis. `None` here is `EstateAdmission::NoEstate`
        // and the predicate's `? IS NOT NULL` makes it admit nothing at all.
        let estate = filter.estate.bind();
        let overlay_exclude = overlay_exclude_like();
        let overlay_admit = filter.source.overlay_admit_source_name();
        let out = self.reader.rows(
            read_sql!(concat!(
                "SELECT generation_id, source_name, source_kind, authority_class, content_key, \
                        observed_at \
                 FROM source.generations g \
                 WHERE g.state = ? \
                   AND ",
                admissible_estate_clause!(),
                " AND ( (g.source_name NOT LIKE ? \
                          AND (? IS NULL OR g.source_name = ?)) \
                         OR (? IS NOT NULL AND g.source_name = ?) ) \
                   AND (? IS NULL OR g.content_key = ?) \
                   AND (? IS NULL OR g.source_kind = ?) \
                   AND (? IS NULL OR g.authority_class = ?) \
                 ORDER BY g.source_name, g.observed_at DESC, g.generation_id DESC LIMIT ?"
            )),
            duckdb::params![
                STATE_CONFIRMED,
                estate,
                estate,
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
            ],
            |row| -> Result<SourceGeneration, AtlasError> {
                let kind: String = row.get(2)?;
                let auth: String = row.get(3)?;
                Ok(SourceGeneration {
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
                })
            },
        )?;
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
    /// [`Self::generations`]. Bounded by `limit` (capped at
    /// [`MAX_ROWS`], F12).
    fn units(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<StoredUnitHit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let (source_name, content_key) = filter.source.bindings();
        let source_kind = filter.kind.map(SourceKind::as_str);
        let authority = filter.authority.map(AuthorityClass::as_str);
        let estate = filter.estate.bind();
        let [doc_a, doc_b, doc_c] = DOCUMENT_EXTRACTOR_IDENTITIES;
        let overlay_exclude = overlay_exclude_like();
        let overlay_admit = filter.source.overlay_admit_source_name();
        let out = self.reader.rows(
            read_sql!(concat!(
                "SELECT g.source_name, u.relative_path, u.local_key, u.ordinal, u.unit_kind, \
                        u.heading_level, u.title, u.byte_start, u.byte_end, u.body \
                 FROM source.units u \
                 JOIN source.generations g USING (generation_id) \
                 JOIN source.files f ON f.generation_id = u.generation_id \
                                     AND f.relative_path = u.relative_path \
                 WHERE g.state = ? \
                   AND ",
                admissible_estate_clause!(),
                " AND (f.extractor IN (?, ?, ?) OR f.extractor LIKE ?) \
                   AND ( (g.source_name NOT LIKE ? \
                          AND (? IS NULL OR g.source_name = ?)) \
                         OR (? IS NOT NULL AND g.source_name = ?) ) \
                   AND (? IS NULL OR g.content_key = ?) \
                   AND (? IS NULL OR g.source_kind = ?) \
                   AND (? IS NULL OR g.authority_class = ?) \
                 ORDER BY g.source_name, u.relative_path, u.ordinal LIMIT ?"
            )),
            duckdb::params![
                STATE_CONFIRMED,
                estate,
                estate,
                doc_a,
                doc_b,
                doc_c,
                DOCUMENT_EXTRACTOR_LIKE,
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
            ],
            |row| -> Result<StoredUnitHit, AtlasError> {
                let kind: String = row.get(4)?;
                Ok(StoredUnitHit {
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
                })
            },
        )?;
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
    /// [`Self::generations`]. Bounded by `limit` (capped at
    /// [`MAX_ROWS`], F12).
    fn occurrences(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<StoredOccurrenceHit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let (source_name, content_key) = filter.source.bindings();
        let source_kind = filter.kind.map(SourceKind::as_str);
        let authority = filter.authority.map(AuthorityClass::as_str);
        // A2 §2 stage 1's estate axis. `None` here is `EstateAdmission::NoEstate`
        // and the predicate's `? IS NOT NULL` makes it admit nothing at all.
        let estate = filter.estate.bind();
        let overlay_exclude = overlay_exclude_like();
        let overlay_admit = filter.source.overlay_admit_source_name();
        let out = self.reader.rows(
            read_sql!(concat!(
                "SELECT g.source_name, o.relative_path, o.syntax_key, o.extractor, o.language, \
                        o.ordinal, o.label, o.name, o.byte_start, o.byte_end \
                 FROM source.occurrences o JOIN source.generations g USING (generation_id) \
                 WHERE g.state = ? \
                   AND ",
                admissible_estate_clause!(),
                " AND o.extractor LIKE ? \
                   AND ( (g.source_name NOT LIKE ? \
                          AND (? IS NULL OR g.source_name = ?)) \
                         OR (? IS NOT NULL AND g.source_name = ?) ) \
                   AND (? IS NULL OR g.content_key = ?) \
                   AND (? IS NULL OR g.source_kind = ?) \
                   AND (? IS NULL OR g.authority_class = ?) \
                 ORDER BY g.source_name, o.relative_path, o.ordinal LIMIT ?"
            )),
            duckdb::params![
                STATE_CONFIRMED,
                estate,
                estate,
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
            ],
            |row| -> Result<StoredOccurrenceHit, AtlasError> {
                Ok(StoredOccurrenceHit {
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
                })
            },
        )?;
        Ok(Admitted {
            hits: out,
            scope: self.work_scope(filter, true)?,
        })
    }

    /// A2 §2's content-kind filter, **tabular family** — `source.datasets`
    /// (+ `context.row_units`, not read here — see
    /// [`Self::occurrences`]'s note on why the whole family is
    /// not duplicated). No extractor ambiguity here: `source.datasets`
    /// carries no `extractor` column at all (`format`/`reader` are a
    /// different axis), so table-routing alone is exact for this family
    /// (H13.1).
    ///
    /// Stages 1/2/4 come from `filter`, identically to
    /// [`Self::generations`]. Bounded by `limit` (capped at
    /// [`MAX_ROWS`], F12).
    fn datasets(
        &self,
        filter: &Admissibility,
        limit: usize,
    ) -> Result<Admitted<StoredDatasetHit>, AtlasError> {
        let limit = limit.min(MAX_ROWS) as i64;
        let (source_name, content_key) = filter.source.bindings();
        let source_kind = filter.kind.map(SourceKind::as_str);
        let authority = filter.authority.map(AuthorityClass::as_str);
        // A2 §2 stage 1's estate axis. `None` here is `EstateAdmission::NoEstate`
        // and the predicate's `? IS NOT NULL` makes it admit nothing at all.
        let estate = filter.estate.bind();
        let overlay_exclude = overlay_exclude_like();
        let overlay_admit = filter.source.overlay_admit_source_name();
        let out = self.reader.rows(
            read_sql!(concat!(
                    "SELECT g.source_name, d.relative_path, d.format, d.content_hash, d.reader, \
                        d.dataset_key, d.byte_len, d.columns, d.row_count, d.truncated, d.row_units \
                 FROM source.datasets d JOIN source.generations g USING (generation_id) \
                 WHERE g.state = ? \
                   AND ",
                admissible_estate_clause!(),
                " AND ( (g.source_name NOT LIKE ? \
                          AND (? IS NULL OR g.source_name = ?)) \
                         OR (? IS NOT NULL AND g.source_name = ?) ) \
                   AND (? IS NULL OR g.content_key = ?) \
                   AND (? IS NULL OR g.source_kind = ?) \
                   AND (? IS NULL OR g.authority_class = ?) \
                 ORDER BY g.source_name, d.relative_path LIMIT ?"
            )),
            duckdb::params![
                STATE_CONFIRMED,
                estate,
                estate,
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
            ],
            |row| -> Result<StoredDatasetHit, AtlasError> {
                let format: String = row.get(2)?;
                Ok(StoredDatasetHit {
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
                })
            },
        )?;
        Ok(Admitted {
            hits: out,
            scope: self.work_scope(filter, false)?,
        })
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
        self.reader.first(
            read_sql!(
                "SELECT observed_at FROM source.generations \
                 WHERE state = ? AND source_name = ? \
                 ORDER BY observed_at DESC, generation_id DESC LIMIT 1"
            ),
            duckdb::params![STATE_CONFIRMED, source_name],
            |row| -> Result<String, AtlasError> { Ok(row.get(0)?) },
        )
    }
}

/// The fixed, code-owned `NOT LIKE` bound every admissibility query
/// below applies to `source_name`, excluding the whole Work-overlay
/// family ([`crate::runtime::atlas::overlay::OVERLAY_PREFIX`],
/// `work:<id>/<repo>`) — see [`Admissible::generations`]'s own doc
/// for what re-admits exactly one Work's own overlay on top of it
/// ([`SourceSelector::overlay_admit_source_name`], S5 W1b) and why the
/// default-deny stays the default. Derived from the overlay module's
/// own prefix constant rather than a second hardcoded literal, so the
/// two can never drift apart; still never a client-supplied pattern
/// (F12), the same precedent as [`CODE_EXTRACTOR_LIKE`].
fn overlay_exclude_like() -> String {
    format!("{}%", crate::runtime::atlas::overlay::OVERLAY_PREFIX)
}

/// [`WorkScope`] as one stable word for A2 §13's field 3.
///
/// The snapshot variant carries its `observed_at` into the string rather than
/// dropping it: "base and overlay" without *as of when* is exactly the
/// implied-currency claim [`WorkScope`]'s own doc refuses.
fn describe_work_scope(scope: &WorkScope) -> String {
    match scope {
        WorkScope::NotWorkScoped => "not_work_scoped".to_string(),
        WorkScope::BaseOnly => "base_only".to_string(),
        WorkScope::BaseAndOverlaySnapshot {
            overlay_observed_at,
        } => format!("base_and_overlay_snapshot@{overlay_observed_at}"),
    }
}

/// Outcome of [`AtlasDb::reindex_lexical`]: how many units were indexed,
/// and whether the *generation list itself* was capped at [`MAX_ROWS`]
/// (F-IN-01) — mirroring [`LexicalAnswer::truncated`] for the same reason
/// that field exists: a cap that silently changes what a rebuild covers is
/// not merely a shorter list, and a caller must be able to state what its
/// rebuild covers rather than assume the completeness the doc promises but
/// the cap quietly withdraws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReindexOutcome {
    /// How many units were (re)indexed across the rebuilt generations.
    pub indexed: u64,
    /// Whether the generation list itself hit [`MAX_ROWS`] and was capped —
    /// not every non-evicted generation may have been rebuilt.
    pub truncated: bool,
}

/// One unit's accumulating score, with the length BM25 normalizes it by.
struct Scored {
    hit: LexicalHit,
    token_count: u64,
}

/// A bind value for a `(? IS NULL OR column = ?)` clause.
fn optional_text(value: Option<&str>) -> Duck {
    value.map_or(Duck::Null, |text| Duck::Text(text.to_string()))
}

/// H13.1's content-kind filter, document family: the exhaustive, code-owned
/// list of extractor identities [`AtlasDb::admissible_units`] matches
/// against — never a client-supplied pattern (F12). Every identity a
/// document-shaped adapter in this build can write to `source.files`:
/// Markdown, plain text, every Office/document format the one
/// `office::OFFICE_EXTENSIONS` table routes (S6 widened that from `.docx`
/// alone to the eleven document formats its normalizer reads), and mail
/// (`.eml`).
/// **Not**
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
pub const DOCUMENT_EXTRACTOR_IDENTITIES: [&str; 3] = [
    crate::runtime::atlas::text::MARKDOWN_EXTRACTOR,
    crate::runtime::atlas::text::TEXT_EXTRACTOR,
    crate::runtime::atlas::mail::MAIL_EXTRACTOR,
];

/// H13.1's content-kind filter, document family, second half: the office/
/// document adapter's own code-owned `LIKE` pattern (S6).
///
/// Every identity `office.rs` writes shares one prefix by construction (see
/// [`crate::runtime::atlas::office::OFFICE_EXTRACTOR_LIKE`]'s own doc), so
/// this covers a newly routed format the day it is routed. Before S6 that
/// adapter contributed exactly ONE enumerated identity to
/// [`DOCUMENT_EXTRACTOR_IDENTITIES`] above; widening its routing table from
/// one format to eleven is precisely the change that would otherwise have
/// dropped ten of them out of `--content document` without a compile error.
///
/// Re-exported here rather than referenced inline at each query so the two
/// halves of the document family read as one decision.
pub const DOCUMENT_EXTRACTOR_LIKE: &str = crate::runtime::atlas::office::OFFICE_EXTRACTOR_LIKE;

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

/// The schema S5 W2's lexical index lives in — retrieval-facing derived
/// text, `context.row_units`'s own namespace.
const CONTEXT_SCHEMA: &str = "context";

/// `DELETE FROM <ops table>` — [`ops`] behind a `DELETE`, assembled from
/// compile-time pieces because [`Sql`] admits no other kind.
fn delete_from(table: &str) -> Sql {
    let mut statement = sql!("DELETE FROM ");
    statement.extend(&ops(table));
    statement
}

/// `SELECT COUNT(*) FROM <ops table>` — see [`delete_from`].
fn count_of(table: &str) -> Sql {
    let mut statement = sql!("SELECT COUNT(*) FROM ");
    statement.extend(&ops(table));
    statement
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

canned_queries! {
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
}

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
    conn: Store,
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
        let conn = Store::new(Connection::open(&path)?);
        bootstrap_atlas_ddl(&conn)?;
        Self::over(conn, path)
    }

    /// An in-memory projection over `events`, for callers that want the
    /// tables without a file (tests, and any read-only rendering).
    pub fn in_memory<I>(events: I) -> Result<Self, AnalyticsError>
    where
        I: IntoIterator<Item = Result<Event, JournalError>>,
    {
        let conn = Store::new(Connection::open_in_memory()?);
        let mut analytics = Self::over(conn, PathBuf::from(":memory:"))?;
        analytics.catch_up(events)?;
        Ok(analytics)
    }

    fn over(conn: Store, path: PathBuf) -> Result<Self, AnalyticsError> {
        conn.set_statement_cache_capacity(STATEMENT_CACHE);
        conn.execute_batch(sql!(OPS_DDL))?;
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
            self.conn.execute_batch(delete_from(table))?;
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
        append_all(
            &self.conn,
            name!("events"),
            std::mem::take(&mut appended.events),
        )?;
        append_all(
            &self.conn,
            name!("messages"),
            std::mem::take(&mut appended.messages),
        )?;
        append_all(
            &self.conn,
            name!("usage"),
            std::mem::take(&mut appended.usage),
        )?;
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
            self.conn.execute_batch(delete_from(table))?;
        }
        append_all(
            &self.conn,
            name!("work"),
            self.rows.work.values().map(WorkRow::row).collect(),
        )?;
        append_all(
            &self.conn,
            name!("stages"),
            self.rows.stages.values().map(StageRow::row).collect(),
        )?;
        append_all(
            &self.conn,
            name!("executions"),
            self.rows
                .executions
                .values()
                .map(ExecutionRow::row)
                .collect(),
        )?;
        append_all(
            &self.conn,
            name!("tool_calls"),
            self.rows
                .tool_calls
                .values()
                .map(ToolCallRow::row)
                .collect(),
        )?;
        append_all(
            &self.conn,
            name!("repositories"),
            self.rows
                .repositories
                .values()
                .map(RepositoryRow::row)
                .collect(),
        )?;
        append_all(
            &self.conn,
            name!("graph_nodes"),
            self.rows.nodes.values().map(NodeRow::row).collect(),
        )?;
        append_all(
            &self.conn,
            name!("graph_edges"),
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
        let (columns, rows) = self.select(canned_sql(canned.name), duckdb::params![])?;
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
            let mut statement = self.conn.prepare(count_of(table))?;
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
        let mut sql = sql!("SELECT * FROM ");
        sql.extend(&ops(table));
        let (columns, rows) = self.select(&sql, duckdb::params![])?;
        Ok(QueryResult {
            name: (*table).to_string(),
            question: format!("every row of the {table} table"),
            sql: sql.text().to_string(),
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
            sql!(
                "SELECT node_id, kind, label, work_id, source_seq FROM ops.graph_nodes \
                 WHERE work_id = ?1 \
                    OR node_id IN (SELECT from_node FROM ops.graph_edges WHERE work_id = ?1) \
                    OR node_id IN (SELECT to_node FROM ops.graph_edges WHERE work_id = ?1) \
                 ORDER BY source_seq, node_id"
            ),
            duckdb::params![work_id],
        )?;
        let (edge_columns, edge_rows) = self.select(
            sql!(
                "SELECT edge_id, relation, from_node, to_node, source_seq FROM ops.graph_edges \
                 WHERE work_id = ?1 ORDER BY source_seq, edge_id"
            ),
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
        sql: impl Into<Sql>,
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

ops_tables! {
    "events" => "ops.\"events\"",
    "work" => "ops.\"work\"",
    "stages" => "ops.\"stages\"",
    "executions" => "ops.\"executions\"",
    "messages" => "ops.\"messages\"",
    "tool_calls" => "ops.\"tool_calls\"",
    "usage" => "ops.\"usage\"",
    "repositories" => "ops.\"repositories\"",
    "graph_nodes" => "ops.\"graph_nodes\"",
    "graph_edges" => "ops.\"graph_edges\"",
}

/// Bulk-load `rows` into `table` through DuckDB's appender.
///
/// The appender is the reason this projection is usable at all: measured on
/// this container, a single-row `INSERT` costs ~1 ms and an appended row
/// ~4 µs. An empty batch is a no-op rather than an open-and-close.
fn append_all(
    conn: &impl Statements,
    table: Name,
    rows: Vec<Vec<Duck>>,
) -> Result<(), AnalyticsError> {
    append_rows(conn, name!(OPS_SCHEMA), table, rows)?;
    Ok(())
}

/// [`append_all`] over any schema — the same appender, reused by S5 W2's
/// lexical index in `context` (R2). Kept as one function rather than two
/// because the measurement that justifies the appender is a property of
/// DuckDB, not of the `ops` schema.
fn append_rows(
    conn: &impl Statements,
    schema: Name,
    table: Name,
    rows: Vec<Vec<Duck>>,
) -> Result<(), duckdb::Error> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut appender = conn.appender_to_db(table, schema)?;
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
        conn.set_statement_cache_capacity(STATEMENT_CACHE);
        // No [`HARDENING_DDL`] here, and its absence is stronger than its
        // presence would be. Those four settings are database-wide and
        // `lock_configuration = true` makes them permanent, so re-issuing
        // them on a second connection to the same instance is refused by
        // DuckDB itself ("the configuration has been locked"). The
        // projection's own open ran them before its first query, and F4's
        // posture is verified rather than assumed: a clone that somehow
        // reached an unhardened instance is refused here, not left to reach
        // the network later.
        conn.execute_batch(sql!(SCHEMA_DDL))?;
        conn.execute_batch(sql!(TABLE_DDL))?;
        let db = AtlasDb {
            conn,
            path: self.path.clone(),
            semantic: OnceLock::new(),
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
        let mut statement = self.conn.prepare(sql!(WORK_GENERATION_JOIN_SQL))?;
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

        let write = reader.stage_scan(
            &scan_of("notes", "# Two\n"),
            &EstateBinding::Estate(TEST_ESTATE.to_string()),
        );
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
                    coordinate: None,
                    text: body.to_string(),
                }],
                syntax: None,
                parent: None,
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
        let ScanCommit::Staged { generation_id } = db
            .stage_scan(scan, &EstateBinding::Estate(TEST_ESTATE.to_string()))
            .expect("stage")
        else {
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
        let ScanCommit::Staged { generation_id } = db
            .stage_external_git_scan(
                &scan,
                &EstateBinding::Estate(TEST_ESTATE.to_string()),
                &expected,
            )
            .expect("stage")
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
            .stage_external_git_scan(
                &first,
                &EstateBinding::Estate(TEST_ESTATE.to_string()),
                &provenance("commit-1"),
            )
            .expect("stage first")
        else {
            panic!("expected staged");
        };
        db.confirm_scan(&first_id, "evt-1").expect("confirm first");

        let second = external_scan_of("upstream", "# Two\n", "tree-oid-2");
        let ScanCommit::Staged {
            generation_id: second_id,
        } = db
            .stage_external_git_scan(
                &second,
                &EstateBinding::Estate(TEST_ESTATE.to_string()),
                &provenance("commit-2"),
            )
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
            .reader()
            .first(
                read_sql!("SELECT count(*) FROM git.provenance WHERE generation_id = ?"),
                duckdb::params![first_id],
                |row| -> Result<i64, AtlasError> { Ok(row.get(0)?) },
            )
            .expect("count")
            .expect("one row");
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
            .reader()
            .first(
                read_sql!("SELECT count(*) FROM git.provenance"),
                duckdb::params![],
                |row| -> Result<i64, AtlasError> { Ok(row.get(0)?) },
            )
            .expect("count")
            .expect("one row");
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

        let commit = atlas
            .stage_scan(
                &unreachable,
                &EstateBinding::Estate(TEST_ESTATE.to_string()),
            )
            .expect("stage");
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
            atlas
                .stage_scan(&emptied, &EstateBinding::Estate(TEST_ESTATE.to_string()))
                .expect("stage"),
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
                .stage_scan(
                    &scan_of("notes", "# Pending\n"),
                    &EstateBinding::Estate(TEST_ESTATE.to_string()),
                )
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

    /// F-SF-01: `reindex_lexical`'s own doc names its upgrade condition as
    /// "a store written before S5 W2, whose generations have rows but no
    /// postings" — but nothing detected that condition. Reproduce it
    /// directly (strip a confirmed generation's own postings, the same rows
    /// `reindex_lexical` itself deletes before rebuilding) rather than via a
    /// second `Connection::open` on the file, which `tests/x1_atlas_substrate.rs`'s
    /// one-owner assertion forbids: `db.conn` here is the *same* connection
    /// `record` staged and confirmed through, just used to remove rows the
    /// same way an old on-disk store would already be missing them.
    #[test]
    fn a_confirmed_generation_missing_postings_is_detected_and_backfilled() {
        let mut db = AtlasDb::open_in_memory().expect("atlas");
        let generation_id = record(
            &mut db,
            &scan_of("notes", "PaymentRetryPolicy lives here"),
            "evt-1",
        );

        // A generation confirmed through the normal path already has
        // postings (index_generation runs at staging) — strip them to
        // reproduce the exact shape a pre-W2 store would have.
        db.conn
            .execute(
                sql!("DELETE FROM context.lexical_postings WHERE generation_id = ?"),
                duckdb::params![generation_id],
            )
            .expect("strip postings");
        db.conn
            .execute(
                sql!("DELETE FROM context.lexical_units WHERE generation_id = ?"),
                duckdb::params![generation_id],
            )
            .expect("strip units");

        assert!(
            db.lexical_index_needs_rebuild().expect("check"),
            "a confirmed generation with rows but no lexical_units must be flagged stale"
        );
        let before = db
            .lexical_search(&LexicalQuery {
                text: "PaymentRetryPolicy",
                filter: &Admissibility::within_estate(TEST_ESTATE),
                family: None,
                limit: 10,
                semantic: SemanticRequest::Requested,
            })
            .expect("search");
        assert!(
            before.hits.is_empty(),
            "with postings stripped, the unit must be silently unreachable — F-SF-01's gap"
        );

        let outcome = db.reindex_lexical().expect("reindex");
        assert!(outcome.indexed > 0, "the rebuild must actually index units");
        assert!(
            !db.lexical_index_needs_rebuild().expect("recheck"),
            "after reindex_lexical the store must no longer look stale"
        );
        let after = db
            .lexical_search(&LexicalQuery {
                text: "PaymentRetryPolicy",
                filter: &Admissibility::within_estate(TEST_ESTATE),
                family: None,
                limit: 10,
                semantic: SemanticRequest::Requested,
            })
            .expect("search");
        assert!(
            !after.hits.is_empty(),
            "reindex_lexical must backfill the generation's postings"
        );
    }

    // -----------------------------------------------------------------
    // ROUND 3 ATTACK — item 6 (documented gap, confirmed): a hand-written
    // `impl SqlText` with write-shaped text, handed to `ReadSql::of`
    // directly, trips a generic associated const evaluated at
    // monomorphization rather than eagerly. Measured: `cargo check --lib
    // --tests` on this exact code passed clean; `cargo test --lib` (which
    // must link) failed with E0080 at
    // `ReadSql::of::<EvilWrite>::Check::<EvilWrite>::OK` before any test
    // body could run. No write ever executes — the gap is a `cargo check`
    // blind spot, not a build/test/CI one, exactly as db.rs's module doc
    // claims. Left out of the tree (this comment is the record); the code
    // that reproduces it is in the round-3 report.
    //
    // ROUND 3 ATTACK — item 1: `Box::leak`/`String::leak` into `Sql` via
    // the real `sql!` macro, feeding a runtime string that only exists
    // because a test constructed it (standing in for "caller text").
    // -----------------------------------------------------------------
    #[test]
    fn round3_item1_box_leak_into_sql_via_macro() {
        let runtime_text: String = format!("{}{}", "DELETE FROM ", "source.generations");
        // Measured (uncommented one line at a time, `cargo check --lib
        // --tests`, then reverted — this comment is the record):
        //
        // sql!(Box::leak(runtime_text.into_boxed_str())) — E0435, "attempt
        // to use a non-constant value in a constant", pointing at
        // `runtime_text` itself: naming *any* local inside the macro's
        // `const TEXT: &'static str = $text;` is rejected before the
        // compiler even gets to evaluating `Box::leak`.
        //
        // sql!(Box::leak(format!("DELETE FROM {}", "source.generations")
        //     .into_boxed_str())) — no named local, everything inline —
        // gives E0015, "cannot call non-const fn `Box::<str>::leak` in
        // constants".
        //
        // Both fail `cargo check`, not just `cargo build`: this route is
        // closed strictly earlier than item 6's gap. Left commented
        // because leaving either uncommented fails `cargo check` for the
        // whole crate, which is the finding, not a state to leave in the
        // tree.
        //
        // let _ = sql!(Box::leak(format!("DELETE FROM {}",
        //     "source.generations").into_boxed_str()));
        let _ = runtime_text; // silence unused-var; text never reaches Sql
    }

    // -----------------------------------------------------------------
    // ROUND 3 ATTACK — item 2: a `;` batch through `read_sql!`, spelled
    // several ways, to see whether any spelling reaches DuckDB without
    // tripping `is_read_statement`'s compile-time `;` scan.
    // -----------------------------------------------------------------
    #[test]
    fn round3_item2_semicolon_batch_spellings() {
        // Plain: read_sql!("SELECT 1; DELETE FROM source.generations;")
        //   -> E0080 at `cargo check`, is_read_statement's assert! fires
        //      (measured below, uncommented then reverted).
        //
        // concat!: read_sql!(concat!("SELECT 1 LIMIT ", "1; DELETE FROM ",
        //     "source.generations;"))
        //   -> same E0080. is_read_statement runs on the ASSEMBLED string
        //      (concat! resolves before the const fn runs), so splitting
        //      "DELETE" across concat! arms changes nothing — the `;` is
        //      still a `;` in the assembled bytes.
        //
        // escape: read_sql!("SELECT 1\u{3b} DELETE FROM source.generations")
        //   -> `\u{3b}` is the ASCII semicolon (0x3B) once the Rust string
        //      literal is parsed — same byte, same E0080. Rust resolves
        //      the escape at lexing, long before `is_read_statement` ever
        //      runs, so there is no "unescaped form" for it to miss.
        //
        // adjacent literals: read_sql!("SELECT 1" ";" " DELETE FROM t")
        //   -> not valid as `$text:expr` to begin with (three adjacent
        //      string literals are not one expression without `concat!`
        //      or `+`); with `concat!` it collapses to the concat! case
        //      above.
        //
        // unicode lookalike (؛ U+061B ARABIC SEMICOLON, ; U+037E GREEK
        // QUESTION MARK, ; U+FF1B FULLWIDTH SEMICOLON): none of these are
        // byte 0x3B, so `is_read_statement` does NOT reject them — but
        // DuckDB's parser does not treat them as statement separators
        // either (measured against duckdb 1.10505.0 via a raw-connection
        // probe: `SELECT 1\u{FF1B}` is one statement, a syntax error at
        // the lookalike character, not two statements). There is no
        // spelling that is simultaneously (a) invisible to the byte scan
        // and (b) a real batch separator to DuckDB, because DuckDB only
        // ever treats 0x3B as a separator, and the scan already covers
        // every 0x3B.
        //
        // None of the four uncommented below compiled; all four are
        // commented out because leaving any one in fails `cargo check`
        // for the whole crate, which is the finding.
        //
        // let _ = read_sql!("SELECT 1; DELETE FROM source.generations;");
        // let _ = read_sql!(concat!("SELECT 1 LIMIT ", "1; DELETE FROM ",
        //     "source.generations;"));
        // let _ = read_sql!("SELECT 1\u{3b} DELETE FROM source.generations");
    }

    /// The one item-2 spelling `is_read_statement` does NOT reject —
    /// a byte scan for `;` (0x3B) has no opinion about a unicode
    /// lookalike — run for real through the guarded pipeline
    /// (`sql!`/`Store` to seed, `read_sql!`/`ReadOnly` to attack),
    /// verified by row count rather than by reasoning about whether
    /// DuckDB's parser would accept it.
    #[test]
    fn round3_item2_unicode_lookalike_reaches_duckdb_but_does_not_write() {
        let atlas = AtlasDb::open_in_memory().expect("atlas");
        atlas
            .conn
            .execute_batch(sql!(
                "CREATE TABLE main.round3_t2(x INTEGER); \
                 INSERT INTO main.round3_t2 VALUES (1), (2), (3);"
            ))
            .expect("seed");
        let before: i64 = atlas
            .conn
            .prepare(sql!("SELECT count(*) FROM main.round3_t2"))
            .expect("prepare count")
            .query_row([], |r| r.get(0))
            .expect("count");
        assert_eq!(before, 3);

        // FULLWIDTH SEMICOLON U+FF1B — compiles clean through `read_sql!`
        // (confirmed: this file's `cargo check` passes with this exact
        // line in place), because it is not byte 0x3B.
        let attack = read_sql!("SELECT x FROM main.round3_t2； DELETE FROM main.round3_t2");
        let reader = atlas.conn.reader();
        let result: Result<Vec<i64>, duckdb::Error> =
            reader.rows(attack, &[], |r| r.get::<_, i64>(0));

        let after: i64 = atlas
            .conn
            .prepare(sql!("SELECT count(*) FROM main.round3_t2"))
            .expect("prepare count")
            .query_row([], |r| r.get(0))
            .expect("count");

        // The finding: DuckDB's parser rejects the lookalike as a syntax
        // error inside the identifier/statement — it is not treated as a
        // separator, so no second statement ever gets a chance to run.
        assert!(result.is_err(), "expected a parse error, got {result:?}");
        assert_eq!(before, after, "row count must be unchanged either way");
    }

    /// ROUND 3 ATTACK — item 3: a statement beginning `SELECT ` that
    /// writes anyway. `is_read_statement`'s prefix check rejects any
    /// statement not literally starting with the seven bytes `SELECT `,
    /// which already excludes `ATTACH`, `COPY … TO`, `INSTALL`, `LOAD`,
    /// `PRAGMA`, `SET`, and `CALL` by construction — none of those begin
    /// with `SELECT `. What is left to try is a `SELECT`-prefixed
    /// statement that writes through DuckDB's own grammar: a
    /// data-modifying CTE (`WITH d AS (DELETE … RETURNING *) SELECT …`,
    /// nested as a derived table so the outer statement still starts with
    /// `SELECT `) and a scalar function with a side effect. Both run for
    /// real, through `read_sql!`/`ReadOnly`, against a seeded table,
    /// verified by row count.
    #[test]
    fn round3_item3_select_prefixed_statement_that_would_write() {
        let atlas = AtlasDb::open_in_memory().expect("atlas");
        atlas
            .conn
            .execute_batch(sql!(
                "CREATE TABLE main.round3_t3(x INTEGER); \
                 INSERT INTO main.round3_t3 VALUES (1), (2), (3);"
            ))
            .expect("seed");
        let row_count = |atlas: &AtlasDb| -> i64 {
            atlas
                .conn
                .prepare(sql!("SELECT count(*) FROM main.round3_t3"))
                .expect("prepare count")
                .query_row([], |r| r.get(0))
                .expect("count")
        };
        let before = row_count(&atlas);
        assert_eq!(before, 3);
        let reader = atlas.conn.reader();

        // A data-modifying CTE nested inside an outer SELECT's FROM
        // clause, so the assembled text still begins `SELECT ` and
        // `is_read_statement` admits it.
        let nested_modifying_cte = read_sql!(
            "SELECT n FROM (WITH d AS (DELETE FROM main.round3_t3 RETURNING x) \
             SELECT count(*) AS n FROM d) z"
        );
        let r1: Result<Vec<i64>, duckdb::Error> =
            reader.rows(nested_modifying_cte, &[], |r| r.get::<_, i64>(0));
        assert!(
            r1.is_err(),
            "this duckdb build (1.10505.0) does not support a CTE nested \
             inside a derived table at all — duckdb's own parser error \
             fires at prepare(), before any write could happen; got {r1:?}"
        );
        assert_eq!(
            before,
            row_count(&atlas),
            "nested modifying CTE must not write"
        );

        // A scalar function with a plausible side effect, admitted
        // because it is a bare SELECT.
        let side_effect_select = read_sql!("SELECT setseed(0.5)");
        let r2: Result<Vec<i64>, duckdb::Error> =
            reader.rows(side_effect_select, &[], |_r| Ok(0i64));
        assert!(r2.is_ok(), "setseed is a pure scalar call; got {r2:?}");
        assert_eq!(before, row_count(&atlas), "setseed must not write");
    }

    /// ROUND 3 ATTACK — item 4: reach a writable handle from inside the
    /// read path. `ReadOnly` hands a mapping closure `&duckdb::Row`,
    /// which implements `AsRef<duckdb::Statement>` (checked in the
    /// `duckdb` 1.10505.0 source, `src/row.rs`), so the closure CAN reach
    /// `&Statement`. Whether that is writable is the whole question.
    /// Every write-capable method on `duckdb::Statement` —
    /// `execute`, `insert`, `raw_execute` — takes `&mut self`
    /// (`src/statement.rs`), so calling one through a shared `&Statement`
    /// is rejected before this even reaches DuckDB, at the borrow
    /// checker, not `is_read_statement`.
    #[test]
    fn round3_item4_row_as_ref_statement_cannot_write() {
        let atlas = AtlasDb::open_in_memory().expect("atlas");
        atlas
            .conn
            .execute_batch(sql!(
                "CREATE TABLE main.round3_t4(x INTEGER); \
                 INSERT INTO main.round3_t4 VALUES (1);"
            ))
            .expect("seed");
        let reader = atlas.conn.reader();
        let sql = read_sql!("SELECT x FROM main.round3_t4");
        let result: Result<Vec<i64>, duckdb::Error> = reader.rows(sql, &[], |row| {
            let _stmt: &duckdb::Statement<'_> = row.as_ref();
            // _stmt.execute(duckdb::params![]) — does not compile:
            //   error[E0596]: cannot borrow `*_stmt` as mutable, as it is
            //   behind a `&` reference
            // every write method needs `&mut Statement`, and this closure
            // only ever has `&Statement`. Measured by uncommenting the
            // line above against this exact test and reverting; the
            // build error is the finding.
            row.get::<_, i64>(0)
        });
        assert_eq!(result.expect("read"), vec![1]);
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

    /// The read rule, as a table — because the two rounds this replaces both
    /// shipped a rule whose *stated* scope was wider than the bytes it
    /// checked.
    ///
    /// Note the last two rows: this rule is deliberately stricter than SQL,
    /// and that is a cost, not an oversight.
    #[test]
    fn is_read_statement_admits_exactly_one_bare_select() {
        for (statement, expected) in [
            ("SELECT 1", true),
            (
                "SELECT count(*) FROM source.generations WHERE state = ?",
                true,
            ),
            // Exploit B, and the concat!-split spelling of it: const
            // evaluation sees the assembled string, so both are one input.
            ("SELECT 1; DELETE FROM source.generations", false),
            ("SELECT 1 LIMIT ?; DELETE FROM source.generations;", false),
            // A trailing separator, on its own.
            ("SELECT 1;", false),
            // The prefix rule, unchanged: seven bytes, case-sensitive.
            ("DELETE FROM source.generations", false),
            ("select 1", false),
            (" SELECT 1", false),
            ("WITH x AS (SELECT 1) SELECT * FROM x", false),
            ("SELECT", false),
            ("", false),
            // Stricter than SQL: one harmless statement, refused anyway.
            ("SELECT ';' FROM source.generations", false),
        ] {
            assert_eq!(
                store::is_read_statement(statement),
                expected,
                "is_read_statement({statement:?})"
            );
        }
    }

    /// [`ops`] is generated from the same tokens as [`TABLES`], so it is total
    /// over that list by construction. [`MUTABLE_TABLES`] is the one *other*
    /// list of names that reaches it, and it is written by hand — so a name
    /// there that is not an `ops` table would reach `ops`'s `unreachable!`
    /// arm at runtime, during a reset. This is the check that stops that.
    #[test]
    fn every_mutable_table_is_an_ops_table() {
        for table in MUTABLE_TABLES {
            assert!(
                TABLES.contains(table),
                "`{table}` is in MUTABLE_TABLES but not in the ops_tables! list, so \
                 `delete_from(\"{table}\")` would hit `ops`'s unreachable arm"
            );
        }
        // And `ops` really does answer for every name in the list — the arm
        // and the entry come from one macro invocation, and this is what
        // proves that stayed true.
        for table in TABLES {
            assert!(
                ops(table).text().starts_with("ops.\""),
                "`{table}` must qualify into the ops namespace"
            );
        }
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
                sql!(
                    "SELECT schema_name, table_name FROM duckdb_tables() \
                     WHERE database_name = current_database() ORDER BY table_name"
                ),
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
                sql!("SELECT work_id, estate_root FROM ops.work ORDER BY work_id"),
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
            .execute_batch(sql!("DROP TABLE ops.events"))
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
            .select(sql!("SELECT ts_ms FROM ops.events"), duckdb::params![])
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
                sql!("SELECT reconcile_disposition FROM ops.executions WHERE execution_id = 'e1'"),
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

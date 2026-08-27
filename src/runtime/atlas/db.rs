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
//! The four tables the local-knowledge scanner writes, and nothing else.
//! Every table lands in the wave that lands its writer (the empty-table
//! refusal doctrine); a declared-but-never populated table is a false
//! promise, not completeness. `git.*` and `context.*` are still empty
//! namespaces because nothing writes them yet.
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

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use duckdb::Connection;

use crate::domain::source::{
    AuthorityClass, Coverage, CoverageRow, SourceGeneration, SourceKind, UnitKind,
};
use crate::runtime::atlas::scan::{ScannedFile, ScannedUnit, SourceScan};
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

/// The tables the local-knowledge scanner writes (X2). Applied after
/// [`SCHEMA_DDL`], on every open, for the same idempotency reason.
///
/// Column choices worth stating, because each one is a contract:
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
CREATE TABLE IF NOT EXISTS meta.coverage (\n\
  generation_id TEXT NOT NULL,\n\
  source_name   TEXT NOT NULL,\n\
  path          TEXT,\n\
  status        TEXT NOT NULL,\n\
  detail        TEXT,\n\
  bytes         BIGINT,\n\
  observed_at   TEXT NOT NULL\n\
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

    fn over(conn: Connection, path: PathBuf) -> Result<Self, AtlasError> {
        conn.set_prepared_statement_cache_capacity(STATEMENT_CACHE);
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
    /// generation — ruling §4's eviction rule, enforced at its only
    /// enforcement point: a generation is evicted only when source bytes
    /// changed, so an unchanged re-scan must not churn one.
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
        {
            return Ok(ScanCommit::Unchanged {
                generation_id: current.id,
                content_key: current.content_key,
            });
        }
        let generation_id = ulid::Ulid::generate().to_string();
        let extractors = scan
            .extractors
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(",");
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
        for file in &scan.files {
            insert_file(&tx, &generation_id, &scan.source_name, file)?;
        }
        for row in &scan.coverage {
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
            evict(
                &tx,
                previous,
                name,
                "superseded: the source bytes changed",
                &observed_at,
            )?;
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
    /// The scope filter is applied **in Rust, over a bounded read**, rather
    /// than as a `LIKE` pattern built into SQL. `source.generations` is a
    /// small table this build already reads whole elsewhere, and F12's rule
    /// against string-built SQL is worth more here than one avoided scan —
    /// a Work id interpolated into a `LIKE` is a pattern, and `%` and `_` in a
    /// pattern do not mean what a caller passing an id means.
    pub fn evict_work_overlays(&mut self, work_id: &str) -> Result<Vec<String>, AtlasError> {
        let prefix = crate::runtime::atlas::overlay::overlay_source_prefix(work_id);
        let mut statement = self.conn.prepare(
            "SELECT generation_id, source_name FROM source.generations \
             WHERE state != ? ORDER BY observed_at DESC, generation_id DESC LIMIT ?",
        )?;
        let mut rows = statement.query(duckdb::params![STATE_EVICTED, MAX_ROWS as i64])?;
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
}

/// Insert one acquired file and its units.
fn insert_file(
    conn: &Connection,
    generation_id: &str,
    source_name: &str,
    file: &ScannedFile,
) -> Result<(), AtlasError> {
    conn.execute(
        "INSERT INTO source.files \
         (generation_id, source_name, relative_path, content_hash, extractor, local_key, \
          byte_len, mtime_millis, unit_count) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        duckdb::params![
            generation_id,
            source_name,
            &file.relative_path,
            &file.content_hash,
            &file.extractor,
            &file.local_key,
            file.byte_len as i64,
            file.mtime_millis,
            file.units.len() as i64,
        ],
    )?;
    for unit in &file.units {
        insert_unit(conn, generation_id, source_name, file, unit)?;
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
    conn.execute(
        "INSERT INTO source.units \
         (generation_id, source_name, relative_path, local_key, ordinal, unit_kind, \
          heading_level, title, byte_start, byte_end, body) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        duckdb::params![
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
        ],
    )?;
    Ok(())
}

/// Insert one coverage observation.
fn insert_coverage(
    conn: &Connection,
    generation_id: &str,
    source_name: &str,
    row: &CoverageRow,
    observed_at: &str,
) -> Result<(), AtlasError> {
    conn.execute(
        "INSERT INTO meta.coverage \
         (generation_id, source_name, path, status, detail, bytes, observed_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        duckdb::params![
            generation_id,
            source_name,
            row.path.as_deref(),
            row.status.as_str(),
            row.detail.as_deref(),
            row.bytes.map(|b| b as i64),
            observed_at,
        ],
    )?;
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
        "DELETE FROM source.files WHERE generation_id = ?",
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
            }],
            coverage: vec![CoverageRow {
                path: Some("doc.md".to_string()),
                status: Coverage::Indexed,
                detail: Some("markdown/v1".to_string()),
                bytes: Some(body.len() as u64),
            }],
            extractors: BTreeSet::from(["markdown/v1".to_string()]),
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

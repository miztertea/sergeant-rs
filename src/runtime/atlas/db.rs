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
//! Schema namespaces only — no table. Every table lands in the wave that
//! lands its writer (the empty-table refusal doctrine); a declared-but-never
//! populated table is a false promise, not completeness.

use std::path::{Path, PathBuf};

use duckdb::Connection;

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

    #[test]
    fn debug_names_the_path_and_never_the_connection() {
        let atlas = AtlasDb::open_in_memory().expect("atlas");
        let debug = format!("{atlas:?}");
        assert!(debug.contains(":memory:"), "{debug}");
        assert!(!debug.contains("Connection"), "{debug}");
    }
}

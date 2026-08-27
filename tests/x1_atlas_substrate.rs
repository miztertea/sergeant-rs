//! S3 X1 — Atlas substrate: the second one-owner invariant, and the schema
//! namespaces Atlas declares.
//!
//! This suite is deliberately independent of `tests/m5_projections.rs`'s
//! `t2_the_duckdb_file_has_exactly_one_owner`. There are two databases, and
//! each has exactly one owning file: M5 owns the assertion about the
//! operations projection's file, this suite owns the assertion about Atlas's.
//! Neither is a union rule ("either of these two files may open a database"),
//! because a union rule passes just as happily when one owner has quietly
//! grown into the other's territory. Two databases, two invariants, two
//! tests.
//!
//! Non-spawning and filesystem-light: no daemon, no estate, no backend.

use std::path::{Path, PathBuf};

use sergeant_rs::runtime::atlas::db::{ATLAS_DB_FILE, ATLAS_DIR, AtlasDb, SCHEMAS, atlas_db_path};

/// The Atlas module tree.
fn atlas_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/runtime/atlas")
}

/// Every `.rs` file under `dir`, recursively.
fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            out.extend(rust_sources(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}

/// A token scan, not a fixed-pattern grep: however the import is spelled
/// (`use duckdb::...`, `duckdb::Connection`, a re-export), the crate's own
/// lowercase name has to appear as a bare identifier somewhere in the file.
/// Prose that mentions the product ("DuckDB") is capitalized and does not
/// collide.
fn names_the_crate(text: &str) -> bool {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|token| token == "duckdb")
}

/// F2. Inside the Atlas tree, `db.rs` is the only file that may reach the
/// database driver, and it hands no connection back out.
#[test]
fn atlas_database_has_exactly_one_owner() {
    let src = atlas_src();
    let owner = src.join("db.rs");
    let files = rust_sources(&src);
    assert!(
        files.len() > 1,
        "the scan must actually cover a tree, not just its owner: {files:?}"
    );

    // The negative half: every sibling is plain Rust.
    for file in &files {
        if *file == owner {
            continue;
        }
        let text = std::fs::read_to_string(file).expect("read source");
        assert!(
            !names_the_crate(&text),
            "{} names the duckdb crate; only runtime/atlas/db.rs may",
            file.strip_prefix(&src)
                .expect("under the atlas tree")
                .display()
        );
    }

    // The positive half: without it the loop above is satisfied by an Atlas
    // that stopped using the database altogether, which is exactly the drift
    // a one-owner assertion exists to catch.
    let db = std::fs::read_to_string(&owner).expect("read runtime/atlas/db.rs");
    assert!(
        names_the_crate(&db),
        "runtime/atlas/db.rs must be the one module in this tree that uses the duckdb crate"
    );

    // And it must not leak the connection. Private-by-default does not cover
    // this: `AtlasDb` is a public struct in a public module, so a `pub conn`
    // would compile and be reachable estate-wide — and a consumer written
    // against it names the lowercase crate token nowhere, so the scan above
    // would not see it either. The field declaration is pinned directly.
    assert!(
        db.contains("\n    conn: Connection,"),
        "the Atlas connection must stay a private field"
    );
    assert!(
        !db.contains("pub fn conn") && !db.contains("-> &Connection"),
        "no accessor may hand a live connection outside the Atlas owner"
    );
}

/// F1. The persistence split is a module-doc contract, not folklore: both
/// the tree's entry point and its database owner have to say, in the file a
/// contributor opens first, that these tables are not disposable the way the
/// operations projection's tables are.
#[test]
fn the_atlas_module_docs_carry_the_persistence_contract() {
    for (name, path) in [
        ("mod.rs", atlas_src().join("mod.rs")),
        ("db.rs", atlas_src().join("db.rs")),
    ] {
        let text = std::fs::read_to_string(&path).expect("read source");
        let doc: String = text
            .lines()
            .take_while(|line| line.starts_with("//!") || line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        for required in ["PERSIST", "SourceGeneration", "coverage", "journal"] {
            assert!(
                doc.contains(required),
                "runtime/atlas/{name}'s module doc must state the persistence \
                 contract; it never mentions {required:?}"
            );
        }
    }
}

/// F3. The namespaces are created in the database, read back out of its own
/// catalog rather than echoed from the constant.
#[test]
fn opening_atlas_declares_the_four_schema_namespaces() {
    let dir = tempfile::tempdir().expect("tempdir");
    let atlas = AtlasDb::open(dir.path()).expect("open atlas");
    assert_eq!(
        atlas.schema_names().expect("schema names"),
        SCHEMAS,
        "atlas declares exactly its own namespaces — no more, no fewer"
    );
    assert_eq!(
        SCHEMAS,
        ["context", "git", "meta", "source"],
        "A1 §5 names exactly these four Atlas namespaces"
    );
}

/// The file is real, and it is not inside the directory whose whole contract
/// is that deleting it loses nothing.
#[test]
fn the_atlas_file_is_created_outside_the_disposable_projections_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let atlas = AtlasDb::open(dir.path()).expect("open atlas");
    let path = atlas_db_path(dir.path());
    assert_eq!(atlas.path(), path);
    drop(atlas);
    assert!(path.is_file(), "{} must exist", path.display());
    assert_eq!(path, dir.path().join(ATLAS_DIR).join(ATLAS_DB_FILE));
    assert!(
        !dir.path().join("projections").exists(),
        "opening atlas must not touch the disposable projections directory"
    );
}

//! S3 X1 — Atlas substrate: the one-owner invariant, and the schema
//! namespaces Atlas declares.
//!
//! **S5 W1c collapsed a pair of tests into this one.** This suite used to be
//! deliberately independent of `tests/m5_projections.rs`'s
//! `t2_the_duckdb_file_has_exactly_one_owner` because there were two
//! databases, each with one owning file, and a *union* rule ("either of these
//! two files may open a database") passes just as happily once one owner has
//! grown into the other's territory. A1 §5 declares one physical database
//! with five logical schemas and A1-02's rationale is "schemas provide
//! separation without more databases"; the owner correction of 2026-08-29
//! settled that the code converges to the contract. `sergeant.duckdb` is
//! gone, `ops` is a schema in `atlas.duckdb`, and one database has one owner
//! and one test. The scan below therefore covers the whole of `src/` with no
//! skipped tree and no allowed-owners list — the strongest form of the rule,
//! not a merged weaker one.
//!
//! Non-spawning and filesystem-light: no daemon, no estate, no backend.

use std::path::{Path, PathBuf};

use sergeant_rs::runtime::atlas::db::{ATLAS_DB_FILE, ATLAS_DIR, AtlasDb, SCHEMAS, atlas_db_path};

/// The whole crate source tree.
fn crate_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

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

/// F2, in the form A1 §5 implies: **one database, one owning file, one
/// assertion.** `src/runtime/atlas/db.rs` is the only file in the entire
/// crate that may reach the database driver, and it hands no connection back
/// out.
///
/// The scan covers all of `src/` — every tree, no exemptions. Before S5 W1c
/// it covered only `src/runtime/atlas/`, and `tests/m5_projections.rs`'s
/// `t2_the_duckdb_file_has_exactly_one_owner` covered everything else against
/// `src/runtime/analytics.rs`, because those were two databases. There is now
/// one, so a second test naming a second owner would be the union rule both
/// suites forbade.
#[test]
fn atlas_database_has_exactly_one_owner() {
    let src = crate_src();
    let owner = atlas_src().join("db.rs");
    let files = rust_sources(&src);
    assert!(
        files.len() > 1,
        "the scan must actually cover a tree, not just its owner: {files:?}"
    );
    assert!(
        !src.join("runtime").join("analytics.rs").exists(),
        "src/runtime/analytics.rs owned the second database; A1 §5 declares one,          so its return would mean the second database returned with it"
    );

    // The negative half: every other file in the crate is plain Rust.
    //
    // The scan is a bare-token match, so it also catches *prose* that spells
    // the database's file name in lowercase. That is not a false positive to
    // be excused with an exception list: an exception list is how a scan
    // stops seeing the thing it was written to see. Documentation outside the
    // owner refers to "Atlas's database file" or `ATLAS_DB_FILE`, and the
    // owner's own module doc is where the literal name belongs.
    for file in &files {
        if *file == owner {
            continue;
        }
        let text = std::fs::read_to_string(file).expect("read source");
        assert!(
            !names_the_crate(&text),
            "{} names the duckdb crate; only runtime/atlas/db.rs may",
            file.strip_prefix(&src).expect("under src").display()
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
    //
    // Since the S5 closeout the field is a `Store` — `db.rs`'s own child
    // module wrapping the driver's `Connection` so that every statement
    // surface takes a `Sql` (A1a §17 item 13's compile-time half). The
    // one-owner rule is unchanged and is why that wrapper is a *child
    // module of this file* rather than a sibling file: a sibling would be a
    // second module naming the driver, which the loop above forbids and
    // which no "these two files may both touch the driver" exception is
    // going to be added for.
    assert!(
        db.contains("\n    conn: Store,"),
        "the Atlas connection must stay a private field"
    );
    assert!(
        db.contains("\n        conn: Connection,\n    }"),
        "the raw driver connection must stay a private field of the `store` child module — \
         that privacy is what stops the rest of db.rs handing the driver a runtime string"
    );
    assert!(
        !db.contains("pub fn conn") && !db.contains("-> &Connection"),
        "no accessor may hand a live connection outside the Atlas owner"
    );

    // Inherited from the deleted `t2`: the CLI reaches the operations tables
    // through the daemon's HTTP surface and nowhere else. The loop above
    // already proves `cli.rs` names no driver; this pins the positive half,
    // so a CLI that stopped asking the daemon could not pass by simply
    // answering nothing.
    let cli = std::fs::read_to_string(src.join("cli.rs")).expect("read cli");
    assert!(
        cli.contains("/v1/analytics"),
        "clients ask the daemon; they do not open the file"
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
fn opening_atlas_declares_the_five_schema_namespaces() {
    let dir = tempfile::tempdir().expect("tempdir");
    let atlas = AtlasDb::open(dir.path()).expect("open atlas");
    assert_eq!(
        atlas.schema_names().expect("schema names"),
        SCHEMAS,
        "atlas declares exactly its own namespaces — no more, no fewer"
    );
    assert_eq!(
        SCHEMAS,
        ["context", "git", "meta", "ops", "source"],
        "A1 §5 names exactly these five schemas in the one atlas.duckdb"
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

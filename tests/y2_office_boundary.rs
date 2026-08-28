//! S4 Y2 (G3) — **the replaceability boundary, pinned structurally.**
//!
//! The owner's sanction to adopt `anydoc` past a RUSTSEC advisory
//! (`knowledge/rulings/owner-rulings/anydoc-adoption-2026-08-27.md`, J4) is
//! conditioned on a narrow contract in this crate's own vocabulary, with
//! anydoc as one implementation behind it: "as long as we build the
//! contracts and boundaries properly, ripping out anydoc should be fairly
//! straightforward" (the ruling, verbatim). This suite is the proof, in the
//! shape of `tests/x1_atlas_substrate.rs`'s one-owner `duckdb` test and
//! `tests/m5_projections.rs`'s `t2_the_duckdb_file_has_exactly_one_owner`:
//! a token scan of **every** `.rs` file this crate ships — `src/` and
//! `tests/` both, unlike the duckdb tests (which are each scoped to one
//! subtree) because the boundary the brief states is wider than one module
//! tree: "no anydoc type, error, or concept may appear in `domain::source`,
//! the store schema, coverage rows, **or any test outside the adapter's own
//! module**."
//!
//! Non-spawning and filesystem-light: no daemon, no estate, no backend.

use std::path::{Path, PathBuf};

/// The one file allowed to name `anydoc` as a dependency — imports, types,
/// errors — anywhere in this crate.
const OWNER: &str = "src/runtime/atlas/office.rs";

/// This suite's own file. Exempted for a narrower reason than [`OWNER`]: it
/// is not using anydoc, it is *checking for the word* — [`names_the_crate`]
/// cannot exist without writing the literal string it searches for, so a
/// scan that covered this file would always find its own needle. That is a
/// fact about implementing the check, not a boundary violation — nothing
/// here imports, calls, or types against the crate being pinned.
const THIS_FILE: &str = "tests/y2_office_boundary.rs";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under `dir`, recursively — `target/` is never under
/// `src/` or `tests/`, so no exclusion is needed for it.
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

/// A token scan, not a fixed-pattern grep — exactly
/// `tests/x1_atlas_substrate.rs`'s own `names_the_crate` (R2): however the
/// import is spelled (`use anydoc::...`, `anydoc::Format`, a re-export), the
/// crate's own lowercase name has to appear as a bare identifier somewhere
/// in the file. Prose that writes "anydoc" inside a doc comment or a string
/// literal collides too, which is deliberately conservative — a doc comment
/// that needs to *name* the crate belongs in the one file allowed to, and
/// every other file's prose can say "the Office adapter" instead (this
/// file's own doc comment above does exactly that).
fn names_the_crate(text: &str) -> bool {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|token| token == "anydoc")
}

/// **The negative half**: no `.rs` file in this crate — `src/` or `tests/` —
/// may name `anydoc`, except the adapter's own module and this suite's own
/// necessary needle (see [`THIS_FILE`]).
#[test]
fn anydoc_is_named_nowhere_but_the_office_adapter() {
    let root = crate_root();
    let mut files = rust_sources(&root.join("src"));
    files.extend(rust_sources(&root.join("tests")));
    assert!(
        files.len() > 1,
        "the scan must actually cover a tree, not just its owner: {files:?}"
    );

    let exempt = [root.join(OWNER), root.join(THIS_FILE)];
    let mut violations = Vec::new();
    for file in &files {
        if exempt.contains(file) {
            continue;
        }
        let text = std::fs::read_to_string(file).expect("read source");
        if names_the_crate(&text) {
            violations.push(
                file.strip_prefix(&root)
                    .expect("under the crate root")
                    .display()
                    .to_string(),
            );
        }
    }
    assert!(
        violations.is_empty(),
        "these files name `anydoc`, but only {OWNER} may — no anydoc type, error or concept \
         may cross the adapter's own boundary (owner ruling \
         knowledge/rulings/owner-rulings/anydoc-adoption-2026-08-27.md): {violations:#?}"
    );
}

/// **The positive half**: without this, the negative check above is
/// satisfied just as happily by a build that stopped using anydoc
/// altogether, which is exactly the drift a one-owner assertion exists to
/// catch (the same argument `atlas_database_has_exactly_one_owner` makes for
/// `duckdb`).
#[test]
fn the_office_adapter_actually_names_anydoc() {
    let path = crate_root().join(OWNER);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {OWNER}: {e}"));
    assert!(
        names_the_crate(&text),
        "{OWNER} must be the one file in this crate that uses the anydoc crate"
    );
}

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

// ------------------------------------------------- the public-API surface pin
//
// The two tests above prove no file *other than* [`OWNER`] names `anydoc` —
// but they trust `OWNER` absolutely, and a per-file token scan cannot see a
// re-export or type alias added *inside* that trusted file: `pub use
// anydoc::model::Document as RawDocument;` or `pub type Block =
// anydoc::model::Block;` would let any other module in the crate hold or
// match on anydoc's own types under a different name, without the token
// "anydoc" ever appearing anywhere but `OWNER`. What follows pins `OWNER`'s
// own exported surface — not what other files say, but what this one is
// *allowed to say* — against a checked-in baseline, so a new `pub` item is a
// visible, reviewed diff instead of a silent pass.

/// Extract every `pub`/`pub(crate)` item [`OWNER`] declares, as normalized
/// text: a `fn` contributes its signature only (a body edit is not a surface
/// change — [`docx_units`]'s own internals, for instance, must stay free to
/// change without touching this baseline); a `struct`/`enum`/`const`/`type`/
/// `use` contributes its full span (so a struct picking up a new field, or a
/// brand new top-level item of any kind, shows up too), with `///` doc-only
/// lines dropped (prose is not part of the surface either).
///
/// Deliberately not indentation-gated: office.rs's own `#[cfg(test)] mod
/// tests` block is scanned too, so turning one of its private helpers
/// `pub(crate)` — the third vector the boundary test can't see on its own —
/// is caught here as well, even though such an item never ships in the
/// worker binary (`#[cfg(test)]` compiles only under `cargo test`) and is
/// invisible to a separate `tests/*.rs` crate either way; catching it
/// anyway costs nothing extra in this scan.
fn office_public_api(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut idx = 0;
    while idx < lines.len() {
        let trimmed = lines[idx].trim_start();
        if !(trimmed.starts_with("pub ") || trimmed.starts_with("pub(")) {
            idx += 1;
            continue;
        }
        if trimmed.contains("fn ") {
            let mut sig: Vec<String> = Vec::new();
            let mut j = idx;
            loop {
                sig.push(lines[j].trim().to_string());
                if lines[j].contains('{') || lines[j].trim_end().ends_with(';') {
                    break;
                }
                j += 1;
                assert!(
                    j < lines.len(),
                    "unterminated pub fn signature at line {idx}"
                );
            }
            let mut joined = sig.join(" ");
            if let Some(pos) = joined.find('{') {
                joined.truncate(pos + 1);
            }
            out.push(joined);
            idx = j + 1;
            continue;
        }
        // struct / enum / const / type / use / static: capture the whole
        // span by brace-depth (so a struct's own `pub` fields are pinned as
        // part of it, not re-scanned as separate top-level items), dropping
        // doc-only lines.
        let mut depth: i32 = 0;
        let mut item: Vec<String> = Vec::new();
        let mut j = idx;
        loop {
            let l = lines[j];
            depth += l.matches('{').count() as i32 - l.matches('}').count() as i32;
            if !l.trim_start().starts_with("///") {
                item.push(l.trim().to_string());
            }
            let terminated =
                depth == 0 && (l.trim_end().ends_with(';') || l.trim_end().ends_with('}'));
            j += 1;
            if terminated || j >= lines.len() {
                break;
            }
        }
        out.push(item.join("\n"));
        idx = j;
    }
    out
}

/// The checked-in baseline `office_public_api` must exactly match. Adding a
/// new `pub`/`pub(crate)` item to [`OWNER`] — a re-export, a type alias, a
/// promoted test helper — makes this fail; the fix is either (a) don't add
/// it, or (b) if the wider surface is an intentional, reviewed change,
/// update this constant to match and explain why in the same diff.
const EXPECTED_PUBLIC_API: &[&str] = &[
    "pub const DOCX_EXTRACTOR: &str = \"anydoc/0.2.4+docx/v1\";",
    "pub const DOCX_EXTENSIONS: &[&str] = &[\"docx\"];",
    "pub fn extractor_for(relative: &str) -> Option<&'static str> {",
    "pub struct OfficeUnit {\npub kind: UnitKind,\npub heading_level: Option<u8>,\npub title: Option<String>,\npub coordinate: Option<String>,\npub text: String,\n}",
    "pub enum OfficeError {\n#[error(\"malformed document: {0}\")]\nMalformed(String),\n#[error(\"resource limit exceeded: {0}\")]\nResourceLimit(String),\n#[error(\"needs OCR, unsupported in this build: {0}\")]\nNeedsOcr(String),\n#[error(\"document is encrypted\")]\nEncrypted,\n}",
    "pub fn docx_units(bytes: &[u8]) -> Result<Vec<OfficeUnit>, OfficeError> {",
];

#[test]
fn the_office_adapters_public_api_is_pinned() {
    let path = crate_root().join(OWNER);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {OWNER}: {e}"));
    let actual = office_public_api(&text);
    let expected: Vec<String> = EXPECTED_PUBLIC_API.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        actual, expected,
        "{OWNER}'s exported surface changed — if this is an intentional, reviewed widening \
         (not a re-export or type alias that would let anydoc's own types cross the \
         replaceability boundary under a different name), update EXPECTED_PUBLIC_API in this \
         test to match"
    );
}

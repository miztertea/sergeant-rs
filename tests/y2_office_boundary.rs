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

// -------------------------------------------------- crate-root path coverage
//
// The token scan above only ever walks `src/` and `tests/` — hardcoded, not
// discovered. A `build.rs` at the crate root, or an `examples/`/`benches/`
// directory Cargo would also compile and link into this crate, would evade
// the whole scan without either test above ever noticing, because neither
// looks anywhere else. Rather than silently trusting that no such path is
// ever added, this fails loudly the moment one appears, so a future PR
// adding `build.rs`/`examples/`/`benches/` is forced to extend the scan
// (or explain why the new path cannot name anydoc) instead of silently
// riding past this boundary.
#[test]
fn no_uncovered_compiled_source_path_exists_at_the_crate_root() {
    let root = crate_root();
    if root.join("build.rs").exists() {
        panic!(
            "build.rs now exists at the crate root and is NOT scanned by \
             anydoc_is_named_nowhere_but_the_office_adapter — a build script can name anydoc \
             (or alias it, or generate code that does) without the token scan ever seeing it. \
             Either extend that scan to cover build.rs, or state explicitly why this path \
             cannot cross the replaceability boundary, before this test is allowed to pass \
             again."
        );
    }
    for dir in ["examples", "benches"] {
        if root.join(dir).is_dir() {
            panic!(
                "a `{dir}/` directory now exists at the crate root and is NOT scanned by \
                 anydoc_is_named_nowhere_but_the_office_adapter — Cargo compiles and links \
                 files under {dir}/ into this crate just as it does src/ and tests/, so anydoc \
                 could be named there without the token scan ever seeing it. Either extend that \
                 scan to cover {dir}/, or state explicitly why this path cannot cross the \
                 replaceability boundary, before this test is allowed to pass again."
            );
        }
    }
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

// ---------------------------------------------------- the manifest-level pin
//
// The two token-scan tests above prove no `.rs` file other than [`OWNER`]
// spells the literal identifier `anydoc` — but a dependency does not have to
// be *named* "anydoc" to resolve to the anydoc crate. Cargo's own renaming
// feature (`docmodel = { package = "anydoc", version = "0.2.4" }`) adds a
// second entry to `[dependencies]` that resolves to the exact same package,
// importable everywhere in the crate as `docmodel::...`, without the token
// "anydoc" ever appearing in a single `.rs` file. This compiles today and
// would pass both tests above unmodified — the token scan operates one
// abstraction layer below where an alias is introduced. What follows checks
// the layer the alias actually lives in: dependency resolution itself, via
// `cargo metadata`, which reports both a dependency's local (possibly
// renamed) extern name AND the real package it resolves to.

/// One edge from this crate's own root package to a dependency, as
/// `cargo metadata --format-version=1` resolves it: `extern_name` is
/// whatever this crate's own `Cargo.toml` calls it (the renamed name for an
/// aliased entry, otherwise the crate's own name), `resolved_package` is the
/// package that edge actually resolves to, independent of what it is called
/// here.
struct DependencyEdge {
    extern_name: String,
    resolved_package: String,
}

/// Every direct dependency edge from this crate's own root package —
/// `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]` alike,
/// because `cargo metadata`'s resolve graph does not separate them and an
/// alias could just as well be smuggled into any of the three tables. Shells
/// out to `cargo metadata` rather than hand-parsing `Cargo.toml`/`Cargo.lock`
/// (R2/R5: this crate already has no TOML-object-graph-to-dependency-
/// resolution logic of its own to reuse, and reimplementing Cargo's own
/// alias/feature/target-cfg resolution by hand is exactly the kind of bug
/// farm Ponytail's R6/R7 exists to head off) — `--offline --locked` so this
/// never touches the network and never silently re-resolves against a
/// `Cargo.lock` this run didn't check in.
fn root_dependency_edges() -> Vec<DependencyEdge> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = std::process::Command::new(env!("CARGO"))
        .args(["metadata", "--format-version=1", "--offline", "--locked"])
        .current_dir(manifest_dir)
        .output()
        .expect("run cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed (status {:?}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse cargo metadata JSON");

    let root_id = metadata["resolve"]["root"]
        .as_str()
        .expect("cargo metadata must report a single root package for this crate")
        .to_string();

    // Package id -> real package name, from the flat `packages` list (this
    // is where an alias's *true* identity lives — the resolve graph's own
    // `deps[].pkg` is a package id, not a name, precisely so this lookup is
    // required rather than optional).
    let packages = metadata["packages"]
        .as_array()
        .expect("cargo metadata must report a packages array");
    let package_name_by_id = |id: &str| -> String {
        packages
            .iter()
            .find(|p| p["id"].as_str() == Some(id))
            .and_then(|p| p["name"].as_str())
            .unwrap_or_else(|| panic!("cargo metadata's packages array has no entry for {id}"))
            .to_string()
    };

    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .expect("cargo metadata must report a resolve.nodes array");
    let root_node = nodes
        .iter()
        .find(|n| n["id"].as_str() == Some(root_id.as_str()))
        .expect("the root package must have its own resolve node");

    root_node["deps"]
        .as_array()
        .expect("the root resolve node must report a deps array")
        .iter()
        .map(|dep| DependencyEdge {
            extern_name: dep["name"]
                .as_str()
                .expect("each dep entry must report its extern name")
                .to_string(),
            resolved_package: package_name_by_id(
                dep["pkg"]
                    .as_str()
                    .expect("each dep entry must report the package id it resolves to"),
            ),
        })
        .collect()
}

/// **The manifest-level half of the boundary**: exactly one dependency edge
/// out of this crate's own root package may resolve to the `anydoc`
/// package — the one direct `[dependencies]` entry named `anydoc` itself —
/// and that edge's own extern name must be `anydoc`, not a rename. A second
/// edge resolving to the same package under any other local name (Cargo's
/// `package = "anydoc"` rename feature) is exactly the alias bypass this
/// test exists to catch, and fails here regardless of what string that
/// second entry happens to be called — this does not match on the one alias
/// a reviewer happened to name (`docmodel`), it matches on *any* dependency
/// edge whose resolved package is `anydoc`, beyond the one known-good one.
///
/// Demonstrated failing against the alias bypass named in review: adding
/// `docmodel = { package = "anydoc", version = "0.2.4" }` to `[dependencies]`
/// and re-running this test fails it with exactly the two-edges message
/// below, naming both `anydoc` and `docmodel` as edges resolving to the
/// `anydoc` package — recorded verbatim in this fix's commit message.
#[test]
fn exactly_one_dependency_edge_resolves_to_the_anydoc_package() {
    let edges = root_dependency_edges();
    let anydoc_edges: Vec<&DependencyEdge> = edges
        .iter()
        .filter(|e| e.resolved_package == "anydoc")
        .collect();

    let names: Vec<&str> = anydoc_edges
        .iter()
        .map(|e| e.extern_name.as_str())
        .collect();
    assert_eq!(
        anydoc_edges.len(),
        1,
        "exactly one dependency edge from this crate's own manifest may resolve to the anydoc \
         package — found {}: {names:?}. A second edge resolving to anydoc under any local name \
         (Cargo's `package = \"...\"` rename feature, e.g. `docmodel = {{ package = \"anydoc\", \
         version = \"0.2.4\" }}`) is a second, real route to anydoc's API that the token scan in \
         anydoc_is_named_nowhere_but_the_office_adapter cannot see, because no `.rs` file needs \
         to write the word \"anydoc\" to use it. Remove the extra manifest entry — {OWNER} is \
         the one file allowed to depend on anydoc, under its own name.",
        anydoc_edges.len()
    );
    assert_eq!(
        anydoc_edges[0].extern_name, "anydoc",
        "the one dependency edge resolving to the anydoc package must be named `anydoc` in this \
         crate's own manifest, not renamed via `package = \"anydoc\"`: {:?}",
        anydoc_edges[0].extern_name
    );
}

/// **Demonstration that the check above actually fires for the alias
/// bypass — not just against the one name a reviewer happened to name.**
///
/// Two ways this was proven, both recorded here rather than left as a claim:
///
/// 1. Empirically, against the real bypass named in review: adding
///    `docmodel = { package = "anydoc", version = "0.2.4" }` to this crate's
///    actual `[dependencies]` and running `cargo metadata --offline --locked`
///    (what [`root_dependency_edges`] shells out to) fails outright, before
///    [`exactly_one_dependency_edge_resolves_to_the_anydoc_package`] even
///    gets to run its own assertion:
///    `error: the crate `sergeant-rs v0.3.0 (...)` depends on crate `anydoc
///    v0.2.4` multiple times with different names` — Cargo's own resolver
///    refuses a manifest that names the exact same resolved package twice
///    under two different local names, which is a *stronger* failure than a
///    test assertion (nothing downstream even builds). Recorded verbatim in
///    this fix's commit message, and reverted before commit — this repo's
///    own `Cargo.toml`/`Cargo.lock` are never left carrying the alias.
/// 2. Structurally, offline and deterministic (this test): Cargo's own
///    refusal above is specific to an alias at the *exact same version* as
///    the real entry — a second alias at a different, semver-incompatible
///    anydoc version would resolve to a genuinely different graph node and
///    Cargo would permit it to coexist (the same mechanism that lets a crate
///    depend on `rand 0.7` and `rand 0.8` under two different names at
///    once). [`root_dependency_edges`]'s own filter — "does this edge's
///    resolved package name equal `anydoc`" — does not care about version,
///    so it still catches that case. This test proves the filtering logic
///    itself trips for *any* second edge resolving to `anydoc`, under *any*
///    local name, without depending on which anydoc version crates.io (or
///    this offline sandbox's registry cache) happens to have available.
#[test]
fn the_check_catches_an_alias_under_any_name_not_just_docmodel() {
    for alias in ["docmodel", "totally_unrelated_name", "office_docs_lib"] {
        let edges = [
            DependencyEdge {
                extern_name: "anydoc".to_string(),
                resolved_package: "anydoc".to_string(),
            },
            DependencyEdge {
                extern_name: alias.to_string(),
                resolved_package: "anydoc".to_string(),
            },
        ];
        let anydoc_edges: Vec<&DependencyEdge> = edges
            .iter()
            .filter(|e| e.resolved_package == "anydoc")
            .collect();
        assert_eq!(
            anydoc_edges.len(),
            2,
            "the real check (exactly_one_dependency_edge_resolves_to_the_anydoc_package) fails \
             exactly here: a second edge resolving to anydoc trips its `assert_eq!(.., 1, ..)` \
             regardless of what that second edge is locally named ({alias:?} here) — proving the \
             check is not a string match on the one alias a reviewer happened to name"
        );
    }
}

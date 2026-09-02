//! F5 criterion (b) — the fixture corpus gate.
//!
//! A checked-in multi-language corpus (Rust, TOML, Markdown, Python, JS, TS,
//! shell) with **hand-verified** symbol and import counts in
//! `tests/fixtures/tslp_corpus/manifest.toml`. Pass is EXACT match per
//! fixture — counts, ordered names, ordered labels, ordered import targets.
//!
//! Three properties this suite exists to keep true, each of which the obvious
//! lazy version of it would lose:
//!
//! 1. **A parse error is a FAIL, not a skip.** No `SKIPPED-ENV` escape hatch
//!    exists here and none may be added: nothing about parsing bytes depends
//!    on the two environments CONTRIBUTING's two-environment rule is about
//!    (no permission bits, no filesystem behaviour, no network), so a failure
//!    here is a real failure on any runner.
//! 2. **Exact, not "at least".** A `>=` assertion passes for an extractor
//!    that silently found half the symbols, which is precisely the silent
//!    partial parse F5 forbids.
//! 3. **The malformed fixtures must still error.** Otherwise property 1 is
//!    unfalsifiable — a corpus of only-valid files would pass against an
//!    extractor whose error detection never fires.

use std::path::{Path, PathBuf};

use sergeant_rs::runtime::atlas::syntax::{self, SyntaxError, SyntaxLanguage};

#[derive(serde::Deserialize)]
struct Manifest {
    fixture: Vec<Fixture>,
    malformed: Vec<Malformed>,
}

#[derive(serde::Deserialize)]
struct Fixture {
    path: String,
    language: String,
    symbols: usize,
    imports: usize,
    symbol_names: Vec<String>,
    symbol_labels: Vec<String>,
    import_targets: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Malformed {
    path: String,
    language: String,
}

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tslp_corpus")
}

fn manifest() -> Manifest {
    let path = corpus_root().join("manifest.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    toml::from_str(&text).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()))
}

fn language(name: &str) -> SyntaxLanguage {
    SyntaxLanguage::ALL
        .iter()
        .copied()
        .find(|l| l.name() == name)
        .unwrap_or_else(|| panic!("manifest names an unclaimed language: {name}"))
}

#[test]
fn the_corpus_covers_every_language_this_build_claims() {
    let manifest = manifest();
    let mut covered: Vec<&str> = manifest
        .fixture
        .iter()
        .map(|f| language(&f.language).name())
        .collect();
    covered.sort_unstable();
    covered.dedup();

    let mut claimed: Vec<&str> = SyntaxLanguage::ALL.iter().map(|l| l.name()).collect();
    claimed.sort_unstable();

    assert_eq!(
        covered, claimed,
        "every language `SyntaxLanguage::ALL` claims needs a corpus fixture — \
         an unfixtured language is a claim nothing checks"
    );
}

#[test]
fn every_fixture_matches_its_hand_verified_counts_exactly() {
    let manifest = manifest();
    assert!(
        !manifest.fixture.is_empty(),
        "the manifest lists no fixtures"
    );

    for fixture in &manifest.fixture {
        let path = corpus_root().join(&fixture.path);
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        let language = language(&fixture.language);

        // Routing is part of the claim: a fixture the extractor would never
        // be handed is not evidence that the extractor works.
        assert_eq!(
            syntax::language_for(&fixture.path),
            Some(language),
            "{} routes to the wrong language",
            fixture.path
        );

        // A parse error here is a FAIL, and this is the line that makes it
        // one — `expect` rather than a `continue`.
        let facts = syntax::extract(language, &bytes).unwrap_or_else(|e| {
            panic!(
                "{} must parse cleanly — F5 forbids a partial parse: {e}",
                fixture.path
            )
        });

        let names: Vec<String> = facts.symbols.iter().map(|s| s.name.clone()).collect();
        let labels: Vec<String> = facts.symbols.iter().map(|s| s.label.to_string()).collect();
        let targets: Vec<String> = facts.imports.iter().map(|i| i.target.clone()).collect();

        assert_eq!(
            names, fixture.symbol_names,
            "{}: symbol names differ from the hand-verified list",
            fixture.path
        );
        assert_eq!(
            labels, fixture.symbol_labels,
            "{}: symbol labels differ from the hand-verified list",
            fixture.path
        );
        assert_eq!(
            targets, fixture.import_targets,
            "{}: import targets differ from the hand-verified list",
            fixture.path
        );

        // The counts the manifest states in their own right, so a manifest
        // whose list and count disagree fails rather than half-checking.
        assert_eq!(
            facts.symbols.len(),
            fixture.symbols,
            "{}: symbol COUNT differs",
            fixture.path
        );
        assert_eq!(
            facts.imports.len(),
            fixture.imports,
            "{}: import COUNT differs",
            fixture.path
        );
        assert_eq!(
            fixture.symbol_names.len(),
            fixture.symbols,
            "{}: the manifest's own name list and count disagree",
            fixture.path
        );
        assert_eq!(
            fixture.symbol_labels.len(),
            fixture.symbols,
            "{}: the manifest's own label list and count disagree",
            fixture.path
        );
        assert_eq!(
            fixture.import_targets.len(),
            fixture.imports,
            "{}: the manifest's own target list and count disagree",
            fixture.path
        );
    }
}

#[test]
fn every_symbol_and_import_slices_back_out_of_the_original_bytes() {
    for fixture in &manifest().fixture {
        let path = corpus_root().join(&fixture.path);
        let bytes = std::fs::read(&path).expect("fixture readable");
        let facts = syntax::extract(language(&fixture.language), &bytes).expect("parses");

        for symbol in &facts.symbols {
            let slice = bytes
                .get(symbol.byte_start..symbol.byte_end)
                .unwrap_or_else(|| {
                    panic!("{}: {} has an out-of-range span", fixture.path, symbol.name)
                });
            let slice = std::str::from_utf8(slice).expect("span is UTF-8");
            assert!(
                slice.contains(symbol.name.as_str()),
                "{}: the span for {} does not contain its own name — provenance is broken",
                fixture.path,
                symbol.name
            );
        }
        for import in &facts.imports {
            let slice = bytes
                .get(import.byte_start..import.byte_end)
                .unwrap_or_else(|| panic!("{}: import span out of range", fixture.path));
            let slice = std::str::from_utf8(slice).expect("span is UTF-8");
            assert!(
                slice.contains(import.target.as_str()),
                "{}: the span for import {} does not contain its own target",
                fixture.path,
                import.target
            );
        }
    }
}

#[test]
fn malformed_fixtures_error_rather_than_returning_a_partial_parse() {
    let manifest = manifest();
    assert!(
        !manifest.malformed.is_empty(),
        "without a malformed fixture, the clean-parse assertion above is unfalsifiable"
    );

    for malformed in &manifest.malformed {
        let path = corpus_root().join(&malformed.path);
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        let error = syntax::extract(language(&malformed.language), &bytes).unwrap_err();
        assert!(
            matches!(error, SyntaxError::Parse { .. }),
            "{} should fail as a parse error, got {error:?}",
            malformed.path
        );
    }
}

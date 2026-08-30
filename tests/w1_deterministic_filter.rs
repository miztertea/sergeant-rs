//! S5 W1 — A2 §2's deterministic admissibility filter.
//!
//! A2 §2's own words: *"The first operation is deterministic admissibility,
//! not embeddings"* — source/estate/Work-generation filter, then authority
//! filter, then content-kind filter, then the optional repo/knowledge/
//! external selector, and only then retrieve/rank (not built until W2-W4).
//! This suite tests [`AtlasDb::admissible_generations`]/
//! [`AtlasDb::admissible_units`]/[`AtlasDb::admissible_occurrences`]/
//! [`AtlasDb::admissible_datasets`] directly — W1's own scope statement:
//! "this wave exposes the filter internally and tests it directly" (`sgt
//! search`'s CLI surface is W5).
//!
//! **Negative-admission tests are the point** (the brief's own words): a
//! filter that only proves it returns the right rows has not been tested.
//! Every stage below has at least one test that proves an inadmissible row
//! is NOT returned, not merely that an admissible one is.
//!
//! Fixtures are hand-built [`SourceScan`] values recorded through the real
//! [`record_scan`] three-step path (stage/journal/confirm) — the same
//! public entry point every real scanner (`git`/`scan`/`external_git`)
//! funnels through — rather than a real git repository or a real
//! filesystem walk, because what this suite is proving is admissibility
//! SQL, not acquisition. The one exception is the `--content config`
//! live-verification test, which deliberately runs the REAL
//! `scan_local_knowledge` pipeline over a real `Cargo.toml`, because that
//! test's whole point is to check what the real extractors actually do —
//! `tests/x3b_syntax_wiring.rs`'s own `POLYGLOT` fixture already proves the
//! extraction; this suite proves what the ADMISSIBILITY FILTER does with
//! it.

/// **S6 D1 — A2 §2 stage 1's estate coordinate.** This suite is
/// single-estate: every generation it records is bound to this one root and
/// every filter it builds is admitted from it. The cross-estate case — two
/// estates on one host daemon, which is where the axis actually earns its
/// keep — is `tests/d1_estate_isolation.rs`, deliberately not folded in
/// here, because a suite that never crosses estates cannot notice an estate
/// filter that does nothing (that is exactly how the leak survived: this
/// file's ancestors all passed).
#[allow(dead_code)]
const D1_ESTATE: &str = "/estates/w1_deterministic_filter";

use std::collections::BTreeSet;

use tempfile::TempDir;

use sergeant_rs::domain::event::rfc3339_utc_now;
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::runtime::atlas::db::{
    Admissibility, AtlasDb, CODE_EXTRACTOR_LIKE, DOCUMENT_EXTRACTOR_IDENTITIES,
    DOCUMENT_EXTRACTOR_LIKE, SourceSelector, WorkScope,
};
use sergeant_rs::runtime::atlas::mail::MAIL_EXTRACTOR;
use sergeant_rs::runtime::atlas::office::DOCX_EXTRACTOR;
use sergeant_rs::runtime::atlas::record::{ScanRecord, record_scan};
use sergeant_rs::runtime::atlas::scan::{
    KnowledgeSource, ScannedFile, ScannedSymbol, ScannedSyntax, ScannedUnit, SourceScan,
    scan_local_knowledge,
};
use sergeant_rs::runtime::atlas::syntax::SyntaxLanguage;
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::atlas::text::{MARKDOWN_EXTRACTOR, TEXT_EXTRACTOR};
use sergeant_rs::runtime::journal::Journal;

// ---------------------------------------------------------------- fixtures

/// A whole-document [`ScannedUnit`] — the shape every real document
/// extractor's first unit takes (`text::plain_units`/`markdown_units`).
fn document(text: &str) -> ScannedUnit {
    ScannedUnit {
        ordinal: 0,
        kind: UnitKind::Document,
        heading_level: None,
        title: None,
        byte_start: 0,
        byte_end: text.len() as u64,
        coordinate: None,
        text: text.to_string(),
    }
}

/// A one-symbol syntax extraction under a caller-chosen extractor identity.
/// Real [`SyntaxLanguage`] identities are always `"syntax-<name>/vN"`; a
/// fixture may deliberately hand this something else (`"mystery/v1"`) to
/// simulate an adapter the content-kind filter has never heard of — exactly
/// the case its structural pin exists to catch.
fn syntax(
    language: &'static str,
    extractor: &str,
    label: &'static str,
    name: &str,
) -> ScannedSyntax {
    ScannedSyntax {
        language,
        extractor: extractor.to_string(),
        syntax_key: format!("syntax-key/{language}/{name}"),
        symbols: vec![ScannedSymbol {
            ordinal: 0,
            label,
            name: name.to_string(),
            byte_start: 0,
            byte_end: name.len() as u64,
        }],
        edges: Vec::new(),
    }
}

/// One acquired resource: a structure extraction, an optional syntax
/// extraction, or both — exactly the shape a real grammar-claimed-but-
/// document-unclaimed file takes (`scan.rs`'s own `claims_for` doc).
fn file(
    relative_path: &str,
    extractor: &str,
    units: Vec<ScannedUnit>,
    syntax_extraction: Option<ScannedSyntax>,
) -> ScannedFile {
    ScannedFile {
        relative_path: relative_path.to_string(),
        content_hash: format!("hash/{relative_path}"),
        extractor: extractor.to_string(),
        local_key: format!("key/{relative_path}"),
        byte_len: 16,
        mtime_millis: None,
        units,
        syntax: syntax_extraction,
        parent: None,
    }
}

/// A hand-built [`SourceScan`] over `files`, extractors collected from them.
fn scan(
    source_name: &str,
    kind: SourceKind,
    authority: AuthorityClass,
    content_key: &str,
    files: Vec<ScannedFile>,
) -> SourceScan {
    let mut extractors = BTreeSet::new();
    for f in &files {
        extractors.insert(f.extractor.clone());
        if let Some(s) = &f.syntax {
            extractors.insert(s.extractor.clone());
        }
    }
    SourceScan {
        source_name: source_name.to_string(),
        kind,
        authority,
        content_key: content_key.to_string(),
        revision: None,
        observed_at: rfc3339_utc_now(),
        files,
        coverage: Vec::new(),
        extractors,
        datasets: Vec::new(),
        root: None,
        context_fields: ContextFields::none(),
    }
}

/// A shared store, with four sources recorded into it:
///
/// * `repo-a` (estate-git, estate-mutable): `src/main.rs` (Rust symbol
///   `widget_a`, plus the real plain-text fallback unit every
///   grammar-claimed-but-document-unclaimed file gets — `claims_for`'s own
///   doc), `README.md` (a genuine Markdown document, no syntax), and one
///   file under a `"mystery/v1"` syntax extractor and a `"mystery-doc/v1"`
///   structure extractor — an adapter identity the content-kind filter has
///   never heard of, standing in for "a new adapter that invents an
///   extractor identity the filter does not know about."
/// * `repo-b` (estate-git, estate-mutable): `src/other.rs` (Rust symbol
///   `widget_b`) — proves source exclusion.
/// * `notes` (local-knowledge, estate-readonly): `guide.md` — proves the
///   repo/knowledge selector and the authority-vs-kind distinction.
/// * `vendor-lib` (external-git, external): `lib.rs` (Rust symbol
///   `widget_vendor`) — proves authority exclusion.
struct Estate {
    _data: TempDir,
    /// Kept open (not dropped after setup) so a test can record a further
    /// scan through the real [`record_scan`] path — the supersession test
    /// below needs to drive `confirm_scan`'s own eviction, not fabricate it.
    journal: Journal,
    db: AtlasDb,
}

const REPO_A_KEY: &str = "repo-a@key-1";

fn estate() -> Estate {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    let repo_a = scan(
        "repo-a",
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        REPO_A_KEY,
        vec![
            file(
                "src/main.rs",
                TEXT_EXTRACTOR,
                vec![document("fn widget_a() {}\n")],
                Some(syntax("rust", "syntax-rust/v1", "function", "widget_a")),
            ),
            file(
                "README.md",
                MARKDOWN_EXTRACTOR,
                vec![document("# Repo A\n")],
                None,
            ),
            file(
                "mystery.xyz",
                "mystery-doc/v1",
                vec![document("unclaimed-shaped body")],
                Some(syntax("mystery", "mystery/v1", "thing", "unknown_symbol")),
            ),
        ],
    );
    record_scan(
        &mut db,
        &mut journal,
        &repo_a,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record repo-a");

    let repo_b = scan(
        "repo-b",
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        "repo-b@key-1",
        vec![file(
            "src/other.rs",
            TEXT_EXTRACTOR,
            vec![document("fn widget_b() {}\n")],
            Some(syntax("rust", "syntax-rust/v1", "function", "widget_b")),
        )],
    );
    record_scan(
        &mut db,
        &mut journal,
        &repo_b,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record repo-b");

    let notes = scan(
        "notes",
        SourceKind::LocalKnowledge,
        AuthorityClass::EstateReadonly,
        "notes@key-1",
        vec![file(
            "guide.md",
            MARKDOWN_EXTRACTOR,
            vec![document("# Guide\n")],
            None,
        )],
    );
    record_scan(
        &mut db,
        &mut journal,
        &notes,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record notes");

    let vendor = scan(
        "vendor-lib",
        SourceKind::ExternalGit,
        AuthorityClass::External,
        "vendor-lib@key-1",
        vec![file(
            "lib.rs",
            TEXT_EXTRACTOR,
            vec![document("fn widget_vendor() {}\n")],
            Some(syntax(
                "rust",
                "syntax-rust/v1",
                "function",
                "widget_vendor",
            )),
        )],
    );
    record_scan(
        &mut db,
        &mut journal,
        &vendor,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record vendor-lib");

    Estate {
        _data: data,
        journal,
        db,
    }
}

fn named(source_name: &str) -> Admissibility {
    Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Named(source_name.to_string()),
        kind: None,
        authority: None,
    }
}

fn occurrence_names(estate: &Estate, filter: &Admissibility) -> Vec<String> {
    estate
        .db
        .admissible_occurrences(filter, 500)
        .expect("admissible occurrences")
        .hits
        .into_iter()
        .map(|hit| hit.occurrence.name)
        .collect()
}

// --------------------------------------------------- stage 1: source filter

/// POSITIVE: `--source repo-a` admits `repo-a`'s own occurrence.
/// NEGATIVE: it excludes `repo-b`'s — "another source's rows excluded"
/// (the brief's own example).
#[test]
fn a_named_source_filter_admits_its_own_occurrences_and_excludes_another_sources() {
    let estate = estate();
    let names = occurrence_names(&estate, &named("repo-a"));
    assert!(names.contains(&"widget_a".to_string()), "{names:?}");
    assert!(
        !names.contains(&"widget_b".to_string()),
        "repo-b's occurrence leaked through a repo-a filter: {names:?}"
    );
}

/// The same exclusion at the generation level, one layer under content:
/// `admissible_generations` never hands back a source the caller did not
/// name, whichever content-kind method would eventually read it.
#[test]
fn a_named_source_filter_admits_only_that_sources_generation() {
    let estate = estate();
    let generations = estate
        .db
        .admissible_generations(&named("repo-a"), 500)
        .expect("admissible generations");
    let names: Vec<&str> = generations
        .hits
        .iter()
        .map(|g| g.source_name.as_str())
        .collect();
    assert_eq!(names, vec!["repo-a"], "{names:?}");
    assert_eq!(
        generations.scope,
        WorkScope::NotWorkScoped,
        "a plain --source filter's answer is not Work-scoped at all"
    );
}

/// An exact `--source repo-a@<sha>` pin answers for the CURRENT confirmed
/// generation's own key; a content key that is not that one returns
/// NOTHING rather than approximating — "never approximate" (A2 §2) applies
/// to a miss exactly as it applies to a hit.
#[test]
fn an_exact_generation_pin_matches_its_own_key_and_returns_nothing_for_a_stale_one() {
    let estate = estate();
    let exact = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Exact {
            source_name: "repo-a".to_string(),
            content_key: REPO_A_KEY.to_string(),
        },
        kind: None,
        authority: None,
    };
    let hit = estate
        .db
        .admissible_generations(&exact, 500)
        .expect("admissible generations");
    assert_eq!(hit.hits.len(), 1, "{hit:?}");
    assert_eq!(hit.hits[0].source_name, "repo-a");

    let stale = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Exact {
            source_name: "repo-a".to_string(),
            content_key: "repo-a@a-key-nothing-was-ever-confirmed-under".to_string(),
        },
        kind: None,
        authority: None,
    };
    let miss = estate
        .db
        .admissible_generations(&stale, 500)
        .expect("admissible generations");
    assert!(
        miss.hits.is_empty(),
        "a stale content key must match nothing, got {miss:?}"
    );
}

/// NEGATIVE — the review brief's own scenario: an evicted/superseded
/// generation must not leak. Unlike the test above (a content key that was
/// *never confirmed*), this drives the real live-eviction path: `repo-a` is
/// rescanned with genuinely changed bytes, so `confirm_scan` (via
/// `record_scan`) confirms the new generation and evicts the standing one in
/// the same transaction (`ruling §4`) — physically deleting its rows and
/// flipping its `source.generations.state` off `confirmed`. Every
/// admissibility method, and the exact-pin path at the now-stale key, must
/// come back empty for it.
#[test]
fn a_superseded_generation_does_not_leak_through_any_admissible_method() {
    let mut estate = estate();

    let original = estate
        .db
        .admissible_generations(&named("repo-a"), 500)
        .expect("admissible generations");
    assert_eq!(original.hits.len(), 1, "{original:?}");
    let original_generation_id = original.hits[0].id.clone();
    assert_eq!(original.hits[0].content_key, REPO_A_KEY);

    // Re-scan `repo-a` under the SAME source_name with genuinely different
    // bytes (a new symbol, a new content key, and — the point — no
    // `README.md`/`mystery.xyz` at all) so `confirm_scan` supersedes and
    // evicts the standing generation rather than reporting `Unchanged`.
    let repo_a_v2 = scan(
        "repo-a",
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        "repo-a@key-2",
        vec![file(
            "src/main.rs",
            TEXT_EXTRACTOR,
            vec![document("fn widget_a_v2() {}\n")],
            Some(syntax("rust", "syntax-rust/v1", "function", "widget_a_v2")),
        )],
    );
    let recorded = record_scan(
        &mut estate.db,
        &mut estate.journal,
        &repo_a_v2,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record repo-a v2");
    let evicted = match recorded {
        ScanRecord::Recorded { evicted, .. } => evicted,
        other => panic!("expected a recorded, superseding scan of repo-a, got {other:?}"),
    };
    assert_eq!(
        evicted.as_deref(),
        Some(original_generation_id.as_str()),
        "the rescan must genuinely supersede repo-a's original generation, or nothing below \
         proves the LIVE eviction path rather than an empty database"
    );

    // Stage 1 (source/generation filter): exactly one confirmed generation
    // for `repo-a`, and it is the NEW one — the evicted generation must not
    // still be reported as admissible.
    let after = estate
        .db
        .admissible_generations(&named("repo-a"), 500)
        .expect("admissible generations");
    assert_eq!(
        after.hits.len(),
        1,
        "the evicted generation leaked through admissible_generations: {after:?}"
    );
    assert_eq!(after.hits[0].content_key, "repo-a@key-2");
    assert_ne!(
        after.hits[0].id, original_generation_id,
        "admissible_generations still reports the evicted generation's own id"
    );

    // Content-kind filter, code family: the evicted generation's occurrence
    // (`widget_a`) must not leak through admissible_occurrences — only the
    // surviving generation's `widget_a_v2` is admissible now.
    let names = occurrence_names(&estate, &named("repo-a"));
    assert!(
        !names.contains(&"widget_a".to_string()),
        "the evicted generation's occurrence leaked through admissible_occurrences: {names:?}"
    );
    assert!(names.contains(&"widget_a_v2".to_string()), "{names:?}");

    // Content-kind filter, document family: the evicted generation's
    // `README.md` unit must not leak through admissible_units — the
    // surviving generation never had a README.md at all.
    let paths: Vec<String> = estate
        .db
        .admissible_units(&named("repo-a"), 500)
        .expect("admissible units")
        .hits
        .into_iter()
        .map(|hit| hit.unit.relative_path)
        .collect();
    assert!(
        !paths.contains(&"README.md".to_string()),
        "the evicted generation's unit leaked through admissible_units: {paths:?}"
    );

    // The exact-pin path: pinning at the OLD, now-superseded content key
    // must match nothing — it names a world that no longer stands, exactly
    // like the never-confirmed stale key above, but this key WAS once the
    // real confirmed generation's own.
    let stale_exact = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Exact {
            source_name: "repo-a".to_string(),
            content_key: REPO_A_KEY.to_string(),
        },
        kind: None,
        authority: None,
    };
    let miss = estate
        .db
        .admissible_generations(&stale_exact, 500)
        .expect("admissible generations");
    assert!(
        miss.hits.is_empty(),
        "an exact pin at the evicted generation's own former content key must match nothing, \
         got {miss:?}"
    );
}

// ------------------------------------------- stage 4: repo/knowledge/external

/// POSITIVE: `--type knowledge` (`Admissibility::kind = Some(LocalKnowledge)`)
/// admits `notes`. NEGATIVE: it excludes `repo-a`/`repo-b` (estate-git) and
/// `vendor-lib` (external-git) — the estate's own repositories and a
/// fetched external one are both a different KIND, not merely a different
/// name.
#[test]
fn a_knowledge_kind_selector_admits_only_local_knowledge_sources() {
    let estate = estate();
    let filter = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Any,
        kind: Some(SourceKind::LocalKnowledge),
        authority: None,
    };
    let generations = estate
        .db
        .admissible_generations(&filter, 500)
        .expect("admissible generations");
    let names: Vec<&str> = generations
        .hits
        .iter()
        .map(|g| g.source_name.as_str())
        .collect();
    assert_eq!(names, vec!["notes"], "{names:?}");
}

/// **Stage 4 composes with stage 1** — A2 §2's own listing ("the optional
/// repo/knowledge/external selector") makes `--type` layered ON TOP OF
/// `--source`/`--work`, not an alternative to them. `Admissibility::kind`
/// is a field independent of `SourceSelector` for exactly this reason (see
/// its own doc): the two are checked together here, over both a `--source`
/// selector and a `--work` (`WorkBase`) selector, proving the composition
/// the type system now allows is also correct at the SQL level, not merely
/// expressible.
#[test]
fn stage_4_composes_with_a_named_source_and_with_a_work_base_selector() {
    let estate = estate();

    // `--source repo-a --type repo`: repo-a genuinely IS estate-git, so
    // both halves of the composed filter agree — the generation is
    // admitted.
    let matching = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Named("repo-a".to_string()),
        kind: Some(SourceKind::EstateGit),
        authority: None,
    };
    let hits = estate
        .db
        .admissible_generations(&matching, 500)
        .expect("admissible generations");
    let names: Vec<&str> = hits.hits.iter().map(|g| g.source_name.as_str()).collect();
    assert_eq!(names, vec!["repo-a"], "{names:?}");

    // `--source repo-a --type knowledge`: repo-a is named correctly but is
    // NOT local-knowledge, so the composed (AND) filter admits nothing —
    // proving `kind` genuinely narrows a `Named` selector's own answer
    // rather than being ignored once a source name is given.
    let disagreeing = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Named("repo-a".to_string()),
        kind: Some(SourceKind::LocalKnowledge),
        authority: None,
    };
    let miss = estate
        .db
        .admissible_generations(&disagreeing, 500)
        .expect("admissible generations");
    assert!(
        miss.hits.is_empty(),
        "a --source filter naming an estate-git source, composed with --type knowledge, must          admit nothing: {miss:?}"
    );

    // The same composition over `--work` (`SourceSelector::WorkBase`), the
    // selector this finding named specifically as inexpressible before this
    // fix: `--work <id> --type repo` still reads repo-a's base generation.
    let work_and_kind = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::WorkBase {
            work_id: "01WORKID000000000000000000".to_string(),
            repository: "repo-a".to_string(),
        },
        kind: Some(SourceKind::EstateGit),
        authority: None,
    };
    let work_hits = estate
        .db
        .admissible_generations(&work_and_kind, 500)
        .expect("admissible generations");
    let work_names: Vec<&str> = work_hits
        .hits
        .iter()
        .map(|g| g.source_name.as_str())
        .collect();
    assert_eq!(work_names, vec!["repo-a"], "{work_names:?}");
}

// -------------------------------------------------- stage 2: authority filter

/// NEGATIVE: filtering to `estate_mutable` authority excludes `vendor-lib`'s
/// `external` occurrence even though no `--source`/`--type` named it out —
/// "an `external` authority excluded when unrequested" (the brief's own
/// example).
#[test]
fn an_authority_filter_excludes_external_content_when_not_requested() {
    let estate = estate();
    let filter = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Any,
        kind: None,
        authority: Some(AuthorityClass::EstateMutable),
    };
    let names = occurrence_names(&estate, &filter);
    assert!(
        !names.contains(&"widget_vendor".to_string()),
        "external authority leaked through an estate_mutable-only filter: {names:?}"
    );
    assert!(names.contains(&"widget_a".to_string()) && names.contains(&"widget_b".to_string()));
}

/// POSITIVE mirror: asking for `external` specifically admits ONLY
/// `vendor-lib`'s occurrence, excluding both estate-git repos' — proving
/// the filter is a real predicate in both directions, not a permissive
/// default that only ever narrows the OTHER way.
#[test]
fn an_authority_filter_for_external_admits_only_the_external_source() {
    let estate = estate();
    let filter = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Any,
        kind: None,
        authority: Some(AuthorityClass::External),
    };
    let names = occurrence_names(&estate, &filter);
    assert_eq!(names, vec!["widget_vendor".to_string()], "{names:?}");
}

/// **A2 §2 stage 2 ships with no caller, and that is only honest while the
/// authority axis is degenerate** (S5 closeout F-AC-01).
///
/// `Admissibility::authority` is `None` at the one production construction
/// site (`SearchQuery::admissibility`, serving both `/v1/search` and
/// `/v1/related`), so the two tests above exercise a stage nothing in
/// production can ask for. That is defensible today for exactly one reason:
/// every producer in this build pairs a source kind with one fixed authority
/// class, so `authority_class` is a *total function* of `source_kind` and
/// `--type repo|knowledge|external` already names every world an
/// `--authority` selector could name. A2 §14's selector list carries no
/// authority flag either, so the CLI is complete against the contract, and a
/// second spelling of `--type` is the abstraction R1 exists to refuse.
///
/// None of that is a property of the *design* — A1 keeps the two columns
/// apart deliberately — so it is pinned rather than assumed. The moment a
/// producer lands a generation whose authority is not implied by its kind
/// (a read-only estate mount, an externally-authored local knowledge
/// source), this test fails, and at that point the selector has to ship:
/// `--type` would no longer be able to express what stage 2 filters on.
///
/// Read from the producers themselves, not from a list maintained beside
/// them, so a new producer cannot be added without this seeing it.
#[test]
fn the_authority_axis_earns_no_selector_only_while_it_is_a_function_of_source_kind() {
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pairs: BTreeSet<(String, String)> = BTreeSet::new();
    let mut walked = 0usize;
    let mut stack = vec![src.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read source file");
            // A `SourceScan` literal spells the two fields adjacently, in
            // this order, in every producer. A producer that spells them
            // apart is itself a change to this stage and is meant to be
            // noticed here.
            let mut rest = text.as_str();
            while let Some(at) = rest.find("kind: SourceKind::") {
                rest = &rest[at + "kind: SourceKind::".len()..];
                let Some(comma) = rest.find(',') else { break };
                let kind = rest[..comma].trim().to_string();
                let after = &rest[comma + 1..];
                let head: String = after.chars().take(64).collect();
                let Some(auth_at) = head.find("authority: AuthorityClass::") else {
                    continue;
                };
                let tail = &head[auth_at + "authority: AuthorityClass::".len()..];
                let Some(end) = tail.find(',') else { continue };
                walked += 1;
                pairs.insert((kind, tail[..end].trim().to_string()));
            }
        }
    }
    assert!(
        walked >= 4,
        "the scan must actually find the producers it is about; found {walked}"
    );
    let expected: BTreeSet<(String, String)> = [
        ("LocalKnowledge", "EstateReadonly"),
        ("EstateGit", "EstateMutable"),
        ("ExternalGit", "External"),
    ]
    .into_iter()
    .map(|(k, a)| (k.to_string(), a.to_string()))
    .collect();
    assert_eq!(
        pairs, expected,
        "a source kind is paired with a new authority class somewhere in `src/`. \
         `authority_class` is no longer a function of `source_kind`, so `--type` can no \
         longer express what A2 §2's stage 2 filters on, and `Admissibility::authority` \
         needs the production caller it has never had (`SearchQuery::admissibility` sets \
         it to `None`). Wire the selector — do not extend this list."
    );
}

/// A bare `Admissibility::within_estate(D1_ESTATE)` — `SourceSelector::Any`, no authority
/// — is "every confirmed generation this store holds": the un-narrowed
/// case has to stay complete, or every negative test above would be
/// meaningless (it would be indistinguishable from a filter that excludes
/// everything).
#[test]
fn an_unfiltered_admissibility_admits_every_confirmed_source() {
    let estate = estate();
    let generations = estate
        .db
        .admissible_generations(&Admissibility::within_estate(D1_ESTATE), 500)
        .expect("admissible generations");
    let mut names: Vec<&str> = generations
        .hits
        .iter()
        .map(|g| g.source_name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["notes", "repo-a", "repo-b", "vendor-lib"]);
}

// --------------------------------------------------- stage 3: content-kind

/// NEGATIVE: `admissible_occurrences`' `LIKE 'syntax-%'` match excludes an
/// occurrence written under an extractor identity it has never heard of —
/// H13.1's own scenario, "a new adapter that invents an extractor identity
/// the filter does not know about," proven live at the database level, not
/// merely pinned as a Rust constant.
#[test]
fn a_content_kind_mismatch_is_excluded_from_the_code_family() {
    let estate = estate();
    let names = occurrence_names(&estate, &named("repo-a"));
    assert!(
        !names.contains(&"unknown_symbol".to_string()),
        "an occurrence under an unrecognized extractor identity leaked through the code-family \
         filter: {names:?}"
    );
    // And its two REAL sibling files in the same source are still admitted
    // — the exclusion is per-row, not a source-wide veto.
    assert!(names.contains(&"widget_a".to_string()), "{names:?}");
}

/// The document-family mirror: a unit written under an unrecognized
/// structure extractor is excluded from `admissible_units`, while a real
/// document (`README.md`) from the same source is admitted.
#[test]
fn a_content_kind_mismatch_is_excluded_from_the_document_family() {
    let estate = estate();
    let hits = estate
        .db
        .admissible_units(&named("repo-a"), 500)
        .expect("admissible units");
    let paths: Vec<&str> = hits
        .hits
        .iter()
        .map(|h| h.unit.relative_path.as_str())
        .collect();
    assert!(
        !paths.contains(&"mystery.xyz"),
        "a unit under an unrecognized structure extractor leaked through the document-family \
         filter: {paths:?}"
    );
    assert!(paths.contains(&"README.md"), "{paths:?}");
}

// --------------------------------------------------------- structural pins

/// H13.1's structural pin, code family: every real
/// [`SyntaxLanguage::ALL`] extractor identity matches
/// [`CODE_EXTRACTOR_LIKE`]'s prefix, and a synthetic unknown one does not —
/// so a new [`SyntaxLanguage`] variant automatically stays admitted (it is
/// constructed via the same `"syntax-{name}/vN"` format this pattern
/// covers), while an adapter that writes `source.occurrences` some other
/// way is caught rather than silently included.
#[test]
fn code_extractor_like_covers_every_syntax_language_and_rejects_an_unknown_identity() {
    let prefix = CODE_EXTRACTOR_LIKE
        .strip_suffix('%')
        .expect("the pattern is a prefix match");
    assert_eq!(prefix, "syntax-");
    for language in SyntaxLanguage::ALL {
        let identity = language.extractor_identity();
        assert!(
            identity.starts_with(prefix),
            "{identity} (from {language:?}) does not match the code-family pattern {CODE_EXTRACTOR_LIKE:?}"
        );
    }
    assert!(!"mystery/v1".starts_with(prefix));
    // A real document-family identity, not merely a fabricated one — proves
    // the pattern rejects an adapter identity that genuinely exists in this
    // build, not only a string nothing would ever produce. Referenced by
    // constant rather than spelled out: office.rs's own boundary test
    // (`tests/y2_office_boundary.rs`) forbids the vendor name it wraps from
    // appearing in any other file as a literal.
    assert!(!DOCX_EXTRACTOR.starts_with(prefix));
}

/// H13.1's structural pin, document family — the same shape
/// `tests/x1_atlas_substrate.rs`'s one-owner test uses: read every document
/// adapter module's own `pub const ..._EXTRACTOR` declaration as text, and
/// assert the filter admits EXACTLY that set.
///
/// **Reshaped by S6, and the reshaping is the point.** The document family
/// used to be one enumerated array, with the office adapter contributing a
/// single identity to it. Widening that adapter's routing table from one
/// format to eleven (owner ruling
/// `twelve-formats-is-0.3.0-criteria-2026-08-30`) would have dropped ten new
/// identities outside `--content document` with no compile error and no
/// failing test — the identities are `&'static str`s, not a type the
/// compiler can count. So the family is now an array PLUS one code-owned
/// `LIKE` pattern ([`DOCUMENT_EXTRACTOR_LIKE`], the same F12-safe shape
/// [`CODE_EXTRACTOR_LIKE`] already had for the code family), and this test
/// asserts the union is exact: every constant those modules declare is
/// admitted by one half or the other, and nothing else is.
#[test]
fn document_extractor_identities_matches_every_known_document_adapter_constant() {
    fn extractor_constants(source: &str) -> Vec<String> {
        source
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                let rest = line.strip_prefix("pub const ")?;
                if !rest.contains("_EXTRACTOR: &str = ") {
                    return None;
                }
                let quoted = rest.split('"').nth(1)?;
                Some(quoted.to_string())
            })
            .collect()
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime/atlas");
    let mut expected: BTreeSet<String> = BTreeSet::new();
    for module in ["text.rs", "office.rs", "mail.rs"] {
        let text = std::fs::read_to_string(root.join(module))
            .unwrap_or_else(|e| panic!("read {module}: {e}"));
        // The scan keys on `pub const ..._EXTRACTOR: &str = "…"` exactly, so
        // office.rs's own `..._EXTRACTOR_LIKE` pattern constant (a different
        // spelling) is not swept up as if it were an identity a row could
        // carry.
        expected.extend(extractor_constants(&text));
    }
    // ZIP_EXTRACTOR is a real `pub const ..._EXTRACTOR` too, and is
    // deliberately absent from the allowlist (a container carries no prose
    // of its own — see `DOCUMENT_EXTRACTOR_IDENTITIES`'s own doc) — named
    // here as the one expected mismatch this scan will find, so the loop
    // above stays a real assertion rather than one this test quietly
    // narrows to pass.
    let archive_text = std::fs::read_to_string(root.join("archive.rs")).expect("read archive.rs");
    let zip = sergeant_rs::runtime::atlas::archive::ZIP_EXTRACTOR.to_string();
    expected.extend(extractor_constants(&archive_text));

    let prefix = DOCUMENT_EXTRACTOR_LIKE
        .strip_suffix('%')
        .expect("the pattern is a prefix match");
    let enumerated: BTreeSet<String> = DOCUMENT_EXTRACTOR_IDENTITIES
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    // Every declared identity is admitted by exactly one half of the family,
    // or is the one named container exclusion. "Exactly one" matters: an
    // identity in both halves would mean the enumerated list had started
    // duplicating what the pattern already covers.
    for identity in &expected {
        if *identity == zip {
            assert!(
                !enumerated.contains(identity) && !identity.starts_with(prefix),
                "the container exclusion must be admitted by neither half: {identity}"
            );
            continue;
        }
        let by_list = enumerated.contains(identity);
        let by_pattern = identity.starts_with(prefix);
        assert!(
            by_list ^ by_pattern,
            "{identity} must be admitted by exactly one of DOCUMENT_EXTRACTOR_IDENTITIES \
             or DOCUMENT_EXTRACTOR_LIKE (list={by_list}, pattern={by_pattern})"
        );
    }
    // And nothing in the enumerated half is invented: each names a constant
    // one of those modules actually declares.
    for identity in &enumerated {
        assert!(
            expected.contains(identity),
            "DOCUMENT_EXTRACTOR_IDENTITIES names {identity}, which no document adapter declares"
        );
    }

    assert!(expected.contains(MARKDOWN_EXTRACTOR));
    assert!(expected.contains(TEXT_EXTRACTOR));
    assert!(expected.contains(DOCX_EXTRACTOR));
    assert!(expected.contains(MAIL_EXTRACTOR));
    // The office adapter's eleven routed formats are the pattern half, and
    // the count is asserted so deleting a routed format is a failing test
    // rather than a quiet narrowing — the exact failure mode this ruling was
    // written about.
    assert_eq!(
        expected.iter().filter(|i| i.starts_with(prefix)).count(),
        11,
        "eleven document formats route through the office adapter (csv is the twelfth \
         and stays relational — see office::CSV_IS_NOT_A_DOCUMENT)"
    );
}

// -------------------------------------------------------------- --work scope

/// A `--work` filter reads a named repository exactly like `--source`
/// does — W1's whole point — and, when **no overlay generation stands for
/// the Work**, its answer states [`WorkScope::BaseOnly`] rather than
/// presenting itself as A2 §2's full "including overlay" promise. It
/// excludes a different repository's generation the identical way a plain
/// `--source` filter would.
///
/// **Moved, not deleted, by S5 W1b.** Before W1b, `BaseOnly` was what
/// `--work` ALWAYS answered, because no production caller ever wrote an
/// overlay. Now it is what `--work` answers when this particular Work has
/// none — never bound, never scanned, or evicted with its retirement —
/// which this fixture (base generations only, no overlay recorded) is
/// exactly. The overlay-present answer is pinned by
/// `a_work_filter_admits_its_own_overlay_and_no_other_works` below.
#[test]
fn a_work_base_selector_reads_like_its_named_repository_and_states_base_only_without_an_overlay() {
    let estate = estate();
    let filter = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::WorkBase {
            work_id: "01WORKID000000000000000000".to_string(),
            repository: "repo-a".to_string(),
        },
        kind: None,
        authority: None,
    };
    // The completeness marker is carried on the ANSWER itself -- and it is
    // read from the STORE, not derived from the filter, because whether an
    // overlay stands for a Work is a fact about the store (W1b).
    let generations = estate
        .db
        .admissible_generations(&filter, 500)
        .expect("admissible generations");
    assert_eq!(generations.scope, WorkScope::BaseOnly);

    let names = occurrence_names(&estate, &filter);
    assert!(names.contains(&"widget_a".to_string()), "{names:?}");
    assert!(
        !names.contains(&"widget_b".to_string()),
        "a --work filter scoped to repo-a must not admit repo-b's occurrences: {names:?}"
    );
}

/// A selector that is not `WorkBase` never claims the Work scope — the
/// concept genuinely does not apply, rather than defaulting to some
/// unstated completeness.
#[test]
fn a_non_work_selector_reports_not_work_scoped() {
    let estate = estate();
    for selector in [
        SourceSelector::Named("repo-a".to_string()),
        SourceSelector::Any,
    ] {
        let filter = Admissibility {
            estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
            source: selector.clone(),
            kind: None,
            authority: None,
        };
        assert_eq!(
            estate
                .db
                .admissible_generations(&filter, 500)
                .expect("admissible generations")
                .scope,
            WorkScope::NotWorkScoped,
            "{selector:?}"
        );
    }
}

// -------------------------------------------- --content config, verified live

/// H13.1's `--content config` gap, checked against the REAL extraction
/// pipeline (`scan_local_knowledge`, not a hand-built fixture) rather than
/// asserted from the plan's own prose.
///
/// **Corrects a premise H13.1's own text carried.** The plan states a
/// `.toml` file produces zero `source.units` rows "because
/// `text::extractor_for` claims only Markdown/text extensions" — true of
/// `extractor_for` in isolation, but `scan.rs`'s `claims_for` unions it
/// with `language_for`, and a path only the LATTER claims still gets a
/// plain-text structure extraction (`claims_for`'s own doc: "every
/// acquired resource still has units"). This test proves that live: a
/// `Cargo.toml` produces BOTH a `source.occurrences` row under
/// `"syntax-toml/v1"` (H13.1's own worked example) AND one `source.units`
/// row (a whole-document unit, extractor `"text/v1"`) — not the "zero
/// units" the plan asserted. H13.1's DECIDED mechanism (table +
/// extractor-prefix routing, no new column, `config` not offered as a
/// distinct value) is unaffected by this correction: `admissible_units`
/// still cannot tell this fallback unit apart from a genuine `.txt`
/// document (both carry extractor `"text/v1"`), which is exactly the
/// "safety net, not a clean split" [`AtlasDb::admissible_units`]'s own doc
/// names.
#[test]
fn a_toml_files_config_content_lives_in_the_code_lane_and_also_leaves_a_document_fallback_unit() {
    let source_dir = tempfile::tempdir().expect("source dir");
    std::fs::write(
        source_dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )
    .expect("write Cargo.toml");
    let knowledge = KnowledgeSource {
        name: "manifest".to_string(),
        root: source_dir.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::none(),
    };
    let scanned = scan_local_knowledge(&knowledge).expect("scan Cargo.toml");

    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    record_scan(
        &mut db,
        &mut journal,
        &scanned,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record manifest");

    let filter = named("manifest");

    // H13.1's own worked example: the config's key/table structure is
    // admissible through the code family, under `syntax-toml/v1`.
    let occurrences = db
        .admissible_occurrences(&filter, 500)
        .expect("admissible occurrences");
    assert!(
        occurrences
            .hits
            .iter()
            .any(|hit| hit.occurrence.extractor == "syntax-toml/v1"
                && hit.occurrence.label == "table"
                && hit.occurrence.name == "package"),
        "{occurrences:#?}"
    );

    // The live correction: a document-family unit exists too — not the
    // "zero source.units rows" the plan's own text asserted.
    let units = db.admissible_units(&filter, 500).expect("admissible units");
    assert_eq!(
        units.hits.len(),
        1,
        "Cargo.toml must leave exactly one fallback document unit, got {units:#?}"
    );
    assert_eq!(units.hits[0].unit.relative_path, "Cargo.toml");
    assert_eq!(units.hits[0].unit.kind, UnitKind::Document);
}

// -------------------------------------------------- tabular family, datasets

/// The tabular family (`source.datasets`), through the real
/// `scan_local_knowledge` pipeline (CSV registration is X4's own
/// auto-claim, not something a hand-built fixture should restate). NEGATIVE:
/// `admissible_datasets` scoped to one source excludes the other's — the
/// same source-exclusion proof as the code/document families, over the
/// third physically separate table H13.1 names.
#[test]
fn a_source_filter_excludes_another_sources_dataset() {
    fn knowledge_source_with_csv(name: &str, csv_body: &str) -> (TempDir, KnowledgeSource) {
        let dir = tempfile::tempdir().expect("source dir");
        std::fs::write(dir.path().join("rows.csv"), csv_body).expect("write csv");
        let source = KnowledgeSource {
            name: name.to_string(),
            root: dir.path().to_path_buf(),
            ignore: Vec::new(),
            context_fields: ContextFields::none(),
        };
        (dir, source)
    }

    let (_dir_a, source_a) = knowledge_source_with_csv("data-a", "label,weight\nalpha,1\n");
    let (_dir_b, source_b) = knowledge_source_with_csv("data-b", "label,weight\nbeta,2\n");
    let scan_a = scan_local_knowledge(&source_a).expect("scan data-a");
    let scan_b = scan_local_knowledge(&source_b).expect("scan data-b");
    assert_eq!(scan_a.datasets.len(), 1, "{scan_a:#?}");
    assert_eq!(scan_b.datasets.len(), 1, "{scan_b:#?}");

    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    record_scan(
        &mut db,
        &mut journal,
        &scan_a,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record data-a");
    record_scan(
        &mut db,
        &mut journal,
        &scan_b,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record data-b");

    let hits = db
        .admissible_datasets(&named("data-a"), 500)
        .expect("admissible datasets");
    let sources: Vec<&str> = hits.hits.iter().map(|h| h.source_name.as_str()).collect();
    assert_eq!(sources, vec!["data-a"], "{hits:#?}");
    assert_eq!(hits.hits[0].dataset.relative_path, "rows.csv");
}

// ------------------------------- --work, the fourth required negative (W1)

/// **NEGATIVE: a different Work's generation is excluded** — the fourth
/// negative admission W1's brief names by name, and the one the
/// `--work`-shaped test above does not reach.
///
/// `a_work_base_selector_reads_like_its_named_repository_and_states_base_only_scope`
/// proves a `--work` filter excludes a different *repository*. That is not
/// the same claim: a Work's generation is identified by its own source
/// coordinate, and two Works can be bound to the SAME repository at the
/// same time. So the exclusion that actually matters here is over the
/// overlay coordinate
/// [`overlay_source_name`](sergeant_rs::runtime::atlas::overlay::overlay_source_name)
/// — `work:<id>/<repo>` — which is the shape W1b's daemon-side lifecycle
/// hook will write, and which
/// [`AtlasDb::evict_work_overlays`](sergeant_rs::runtime::atlas::db::AtlasDb::evict_work_overlays)
/// already reads today.
///
/// Three exclusions are required and all three are checked:
///
/// 1. **Neither overlay coordinate is admissible through `SourceSelector::
///    Named` either** — not just through the `--work`/`--type`/`--source`
///    selectors `a_work_base_selector_...` already covers for a *different
///    repository*. `work:<id>/repo-a` describes a world only that Work can
///    see; a caller who merely learns another Work's id (visible via `sgt
///    work list`) must not be able to type it straight into `--source` and
///    read that Work's surface. Every `admissible_*` method excludes the
///    `work:` prefix unconditionally, so this holds even for the coordinate
///    typed exactly.
/// 2. **Another Work's overlay over the same repository is not admissible
///    through `--work` either.** `work:01OTHER…/repo-a` describes a world
///    only that Work can see. A filter that matched the `work:` prefix, or
///    that reached for "anything mentioning repo-a", would admit it — and
///    admissibility decides what may be SEEN, so that is a leak between two
///    Works' surfaces, not a ranking imprecision (A2 §8).
/// 3. **This Work's OWN overlay IS admissible through `--work`** — S5 W1b,
///    which flipped this third assertion and the scope marker together, in
///    this same test, exactly as W1 said they must move (never one without
///    the other). A2 §2 promises "including overlay"; W1b's lifecycle hook
///    is what makes an overlay generation exist to include, and the answer
///    now says
///    [`WorkScope::BaseAndOverlaySnapshot`](sergeant_rs::runtime::atlas::db::WorkScope)
///    with the instant the surface was actually read.
///
/// Exclusions 1 and 2 are unchanged and are the reason this test's name
/// still leads with the negative: admitting one's own overlay must not
/// widen the filter to the overlay FAMILY. `work:<other>/repo-a` fails both
/// branches of the composed predicate, and no `--source` can reach either
/// coordinate.
#[test]
fn a_work_filter_admits_its_own_overlay_and_no_other_works() {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    const MINE: &str = "01MINE0000000000000000000A";
    const OTHER: &str = "01OTHER000000000000000000B";

    // The base world both Works are cut from.
    let base = scan(
        "repo-a",
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        REPO_A_KEY,
        vec![file(
            "src/main.rs",
            TEXT_EXTRACTOR,
            vec![document("fn widget_base() {}\n")],
            Some(syntax("rust", "syntax-rust/v1", "function", "widget_base")),
        )],
    );
    record_scan(
        &mut db,
        &mut journal,
        &base,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record base");

    // Two overlay generations over that same base, one per Work, each under
    // its own `work:<id>/<repo>` source coordinate.
    //
    // The write is proved directly through `record_scan`'s own return value
    // (`ScanRecord::Recorded` — journaled and confirmed) rather than by
    // reading the row back through an admissibility query: every
    // admissibility method now excludes the `work:` prefix unconditionally
    // (the blocker this test exists to pin), so a round trip through
    // `SourceSelector::Named` can no longer serve as proof the fixture
    // landed — that round trip succeeding was the very leak being fixed.
    for (work_id, symbol) in [(MINE, "widget_mine"), (OTHER, "widget_theirs")] {
        let overlay = scan(
            &sergeant_rs::runtime::atlas::overlay::overlay_source_name(work_id, "repo-a"),
            SourceKind::EstateGit,
            AuthorityClass::EstateMutable,
            &format!("repo-a@base+{work_id}"),
            vec![file(
                "src/main.rs",
                TEXT_EXTRACTOR,
                vec![document("fn overlaid() {}\n")],
                Some(syntax("rust", "syntax-rust/v1", "function", symbol)),
            )],
        );
        let recorded = record_scan(
            &mut db,
            &mut journal,
            &overlay,
            None,
            &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
        )
        .expect("record overlay");
        assert!(
            matches!(recorded, ScanRecord::Recorded { .. }),
            "the fixture must actually be journaled and confirmed, or the exclusions below \
             prove nothing: {recorded:?}"
        );
    }

    let filter = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::WorkBase {
            work_id: MINE.to_string(),
            repository: "repo-a".to_string(),
        },
        kind: None,
        authority: None,
    };

    // Neither overlay coordinate is admissible through ANY selector — not
    // just `WorkBase` (W1's `--work`, checked below), but also `Named`/
    // `Exact` addressing the coordinate directly, since a caller who learns
    // another Work's id could otherwise type it straight into `--source`.
    for work_id in [MINE, OTHER] {
        let coordinate =
            sergeant_rs::runtime::atlas::overlay::overlay_source_name(work_id, "repo-a");
        assert!(
            db.admissible_units(&named(&coordinate), 500)
                .expect("admissible units")
                .hits
                .is_empty(),
            "SourceSelector::Named must never surface an overlay coordinate, {coordinate} \
             included: a caller who learns a Work id must not be able to read that Work's \
             surface via --source"
        );
    }

    let sources: BTreeSet<String> = db
        .admissible_generations(&filter, 500)
        .expect("admissible generations")
        .hits
        .into_iter()
        .map(|generation| generation.source_name)
        .collect();
    assert_eq!(
        sources,
        BTreeSet::from([
            "repo-a".to_string(),
            sergeant_rs::runtime::atlas::overlay::overlay_source_name(MINE, "repo-a"),
        ]),
        "--work admits exactly its repository's base generation AND its own overlay — never \
         another Work's overlay over the same repository"
    );

    // And at the content tables too — a content-kind method that composed
    // the source predicate differently would leak another Work's surface
    // into this one's answer while `admissible_generations` stayed correct.
    let names: Vec<String> = db
        .admissible_occurrences(&filter, 500)
        .expect("admissible occurrences")
        .hits
        .into_iter()
        .map(|hit| hit.occurrence.name)
        .collect();
    assert_eq!(
        names,
        vec!["widget_base".to_string(), "widget_mine".to_string()],
        "a --work filter admits its own base and its own overlay, and NEVER another Work's \
         overlay over the same repository: {names:?}"
    );

    // The freshness semantic is declared on the answer, not assumed: the
    // overlay half is a snapshot, and the scope carries the instant it was
    // taken (W1b item 3).
    let scope = db
        .admissible_generations(&filter, 500)
        .expect("admissible generations")
        .scope;
    let WorkScope::BaseAndOverlaySnapshot {
        overlay_observed_at,
    } = &scope
    else {
        panic!("--work must declare the overlay it admitted, and when it was taken: {scope:?}");
    };
    assert!(
        !overlay_observed_at.is_empty(),
        "the snapshot instant is the whole point of the variant"
    );
}

/// The panel-confirmed sibling of the test above: **the SAME Work's own
/// overlay over a DIFFERENT repository must not leak either.**
///
/// The bug this pins built the admitted overlay coordinate from a `LIKE
/// "work:<id>/%"` *prefix*, keyed only on `work_id` — so a
/// `WorkBase { work_id: "mine", repository: "repo-a" }` filter's overlay
/// branch admitted `work:mine/repo-a` AND `work:mine/repo-b` identically,
/// even though the base half (`SourceSelector::bindings`) is restricted to
/// `repo-a` alone. That is the same class of leak
/// `a_work_filter_admits_its_own_overlay_and_no_other_works` pins for a
/// *different* Work; this is the *same* Work's own overlay reaching past the
/// one repository it named. The fix is an exact overlay source name
/// (`SourceSelector::overlay_admit_source_name`), never a prefix.
#[test]
fn a_work_filter_does_not_admit_its_own_overlay_over_a_different_repository() {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    const MINE: &str = "01MINE0000000000000000000A";

    for repo in ["repo-a", "repo-b"] {
        let base = scan(
            repo,
            SourceKind::EstateGit,
            AuthorityClass::EstateMutable,
            &format!("{repo}@key-1"),
            vec![file(
                "src/main.rs",
                TEXT_EXTRACTOR,
                vec![document("fn base() {}\n")],
                Some(syntax("rust", "syntax-rust/v1", "function", "base")),
            )],
        );
        record_scan(
            &mut db,
            &mut journal,
            &base,
            None,
            &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
        )
        .expect("record base");

        let overlay = scan(
            &sergeant_rs::runtime::atlas::overlay::overlay_source_name(MINE, repo),
            SourceKind::EstateGit,
            AuthorityClass::EstateMutable,
            &format!("{repo}@base+{MINE}"),
            vec![file(
                "src/main.rs",
                TEXT_EXTRACTOR,
                vec![document("fn overlaid() {}\n")],
                Some(syntax(
                    "rust",
                    "syntax-rust/v1",
                    "function",
                    &format!("overlaid_{repo}"),
                )),
            )],
        );
        let recorded = record_scan(
            &mut db,
            &mut journal,
            &overlay,
            None,
            &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
        )
        .expect("record overlay");
        assert!(
            matches!(recorded, ScanRecord::Recorded { .. }),
            "the fixture must actually land, or the exclusion below proves nothing: {recorded:?}"
        );
    }

    // `--work mine` scoped to `repo-a`: must admit repo-a's base and MINE's
    // own repo-a overlay, and must NOT admit repo-b's base or MINE's own
    // repo-b overlay — the repository name is a real filter, not decoration.
    let filter = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::WorkBase {
            work_id: MINE.to_string(),
            repository: "repo-a".to_string(),
        },
        kind: None,
        authority: None,
    };
    let sources: BTreeSet<String> = db
        .admissible_generations(&filter, 500)
        .expect("admissible generations")
        .hits
        .into_iter()
        .map(|generation| generation.source_name)
        .collect();
    assert_eq!(
        sources,
        BTreeSet::from([
            "repo-a".to_string(),
            sergeant_rs::runtime::atlas::overlay::overlay_source_name(MINE, "repo-a"),
        ]),
        "--work scoped to repo-a must not admit repo-b's base, and must not admit MINE's own \
         overlay over repo-b either: {sources:?}"
    );

    let names: Vec<String> = db
        .admissible_occurrences(&filter, 500)
        .expect("admissible occurrences")
        .hits
        .into_iter()
        .map(|hit| hit.occurrence.name)
        .collect();
    assert_eq!(
        names,
        vec!["base".to_string(), "overlaid_repo-a".to_string()],
        "content-kind admission must match the generation-level exclusion exactly: {names:?}"
    );
}

/// The panel-confirmed WorkScope-side gap: `--work`'s scope marker must not
/// claim `BaseAndOverlaySnapshot` for an answer whose overlay half is
/// structurally incapable of containing overlay-authored rows.
///
/// `source.datasets` is one such table (an overlay scan's own
/// `datasets: Vec::new()`, `overlay::scan_work_overlay`'s doc), and a
/// `--type`/authority narrowing to anything but the fixed kind/authority
/// every overlay generation is stamped with
/// (`SourceKind::EstateGit`/`AuthorityClass::EstateMutable`) is another:
/// both structurally exclude every row an overlay could ever have written,
/// so asserting a snapshot instant for either would claim freshness for
/// evidence that can never be there.
#[test]
fn work_scope_declares_base_only_when_the_answer_cannot_structurally_carry_overlay_rows() {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    const MINE: &str = "01MINE0000000000000000000A";

    let base = scan(
        "repo-a",
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        REPO_A_KEY,
        vec![file(
            "src/main.rs",
            TEXT_EXTRACTOR,
            vec![document("fn base() {}\n")],
            Some(syntax("rust", "syntax-rust/v1", "function", "base")),
        )],
    );
    record_scan(
        &mut db,
        &mut journal,
        &base,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record base");

    let overlay = scan(
        &sergeant_rs::runtime::atlas::overlay::overlay_source_name(MINE, "repo-a"),
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        &format!("repo-a@base+{MINE}"),
        vec![file(
            "src/main.rs",
            TEXT_EXTRACTOR,
            vec![document("fn overlaid() {}\n")],
            Some(syntax("rust", "syntax-rust/v1", "function", "overlaid")),
        )],
    );
    let recorded = record_scan(
        &mut db,
        &mut journal,
        &overlay,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record overlay");
    assert!(matches!(recorded, ScanRecord::Recorded { .. }));

    let work_base = SourceSelector::WorkBase {
        work_id: MINE.to_string(),
        repository: "repo-a".to_string(),
    };

    // The overlay genuinely stands, and an unnarrowed --work answer says so.
    let unnarrowed = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: work_base.clone(),
        kind: None,
        authority: None,
    };
    assert!(
        matches!(
            db.admissible_generations(&unnarrowed, 500)
                .expect("admissible generations")
                .scope,
            WorkScope::BaseAndOverlaySnapshot { .. }
        ),
        "sanity: the overlay stands, so the unnarrowed answer must say so"
    );

    // `source.datasets` never carries an overlay row at all — BaseOnly
    // regardless of whether the overlay stands.
    assert_eq!(
        db.admissible_datasets(&unnarrowed, 500)
            .expect("admissible datasets")
            .scope,
        WorkScope::BaseOnly,
        "datasets can never come from an overlay scan (it always records datasets: Vec::new()), \
         so the scope must say BaseOnly even though an overlay generation stands"
    );

    // A kind narrowing to anything but EstateGit excludes every row an
    // overlay could ever have written.
    let knowledge_narrowed = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: work_base.clone(),
        kind: Some(SourceKind::LocalKnowledge),
        authority: None,
    };
    assert_eq!(
        db.admissible_generations(&knowledge_narrowed, 500)
            .expect("admissible generations")
            .scope,
        WorkScope::BaseOnly,
        "an overlay generation is always SourceKind::EstateGit, so a --type knowledge/external \
         narrowing must report BaseOnly even though an overlay stands"
    );

    // An authority narrowing to anything but EstateMutable does the same.
    let authority_narrowed = Admissibility {
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: work_base,
        kind: None,
        authority: Some(AuthorityClass::External),
    };
    assert_eq!(
        db.admissible_generations(&authority_narrowed, 500)
            .expect("admissible generations")
            .scope,
        WorkScope::BaseOnly,
        "an overlay generation is always AuthorityClass::EstateMutable, so an authority \
         narrowing to anything else must report BaseOnly even though an overlay stands"
    );
}

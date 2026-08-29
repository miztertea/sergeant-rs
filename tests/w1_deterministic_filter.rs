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

use std::collections::BTreeSet;

use tempfile::TempDir;

use sergeant_rs::domain::event::rfc3339_utc_now;
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::runtime::atlas::db::{
    Admissibility, AtlasDb, CODE_EXTRACTOR_LIKE, DOCUMENT_EXTRACTOR_IDENTITIES, SourceSelector,
    WorkScope,
};
use sergeant_rs::runtime::atlas::mail::MAIL_EXTRACTOR;
use sergeant_rs::runtime::atlas::office::DOCX_EXTRACTOR;
use sergeant_rs::runtime::atlas::record::record_scan;
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
    record_scan(&mut db, &mut journal, &repo_a, None).expect("record repo-a");

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
    record_scan(&mut db, &mut journal, &repo_b, None).expect("record repo-b");

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
    record_scan(&mut db, &mut journal, &notes, None).expect("record notes");

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
    record_scan(&mut db, &mut journal, &vendor, None).expect("record vendor-lib");

    Estate { _data: data, db }
}

fn named(source_name: &str) -> Admissibility {
    Admissibility {
        source: SourceSelector::Named(source_name.to_string()),
        authority: None,
    }
}

fn occurrence_names(estate: &Estate, filter: &Admissibility) -> Vec<String> {
    estate
        .db
        .admissible_occurrences(filter, 500)
        .expect("admissible occurrences")
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
    let names: Vec<&str> = generations.iter().map(|g| g.source_name.as_str()).collect();
    assert_eq!(names, vec!["repo-a"], "{names:?}");
}

/// An exact `--source repo-a@<sha>` pin answers for the CURRENT confirmed
/// generation's own key; a content key that is not that one returns
/// NOTHING rather than approximating — "never approximate" (A2 §2) applies
/// to a miss exactly as it applies to a hit.
#[test]
fn an_exact_generation_pin_matches_its_own_key_and_returns_nothing_for_a_stale_one() {
    let estate = estate();
    let exact = Admissibility {
        source: SourceSelector::Exact {
            source_name: "repo-a".to_string(),
            content_key: REPO_A_KEY.to_string(),
        },
        authority: None,
    };
    let hit = estate
        .db
        .admissible_generations(&exact, 500)
        .expect("admissible generations");
    assert_eq!(hit.len(), 1, "{hit:?}");
    assert_eq!(hit[0].source_name, "repo-a");

    let stale = Admissibility {
        source: SourceSelector::Exact {
            source_name: "repo-a".to_string(),
            content_key: "repo-a@a-key-nothing-was-ever-confirmed-under".to_string(),
        },
        authority: None,
    };
    let miss = estate
        .db
        .admissible_generations(&stale, 500)
        .expect("admissible generations");
    assert!(
        miss.is_empty(),
        "a stale content key must match nothing, got {miss:?}"
    );
}

// ------------------------------------------- stage 4: repo/knowledge/external

/// POSITIVE: `--type knowledge` (`SourceSelector::Kind(LocalKnowledge)`)
/// admits `notes`. NEGATIVE: it excludes `repo-a`/`repo-b` (estate-git) and
/// `vendor-lib` (external-git) — the estate's own repositories and a
/// fetched external one are both a different KIND, not merely a different
/// name.
#[test]
fn a_knowledge_kind_selector_admits_only_local_knowledge_sources() {
    let estate = estate();
    let filter = Admissibility {
        source: SourceSelector::Kind(SourceKind::LocalKnowledge),
        authority: None,
    };
    let generations = estate
        .db
        .admissible_generations(&filter, 500)
        .expect("admissible generations");
    let names: Vec<&str> = generations.iter().map(|g| g.source_name.as_str()).collect();
    assert_eq!(names, vec!["notes"], "{names:?}");
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
        source: SourceSelector::Any,
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
        source: SourceSelector::Any,
        authority: Some(AuthorityClass::External),
    };
    let names = occurrence_names(&estate, &filter);
    assert_eq!(names, vec!["widget_vendor".to_string()], "{names:?}");
}

/// A bare `Admissibility::default()` — `SourceSelector::Any`, no authority
/// — is "every confirmed generation this store holds": the un-narrowed
/// case has to stay complete, or every negative test above would be
/// meaningless (it would be indistinguishable from a filter that excludes
/// everything).
#[test]
fn an_unfiltered_admissibility_admits_every_confirmed_source() {
    let estate = estate();
    let generations = estate
        .db
        .admissible_generations(&Admissibility::default(), 500)
        .expect("admissible generations");
    let mut names: Vec<&str> = generations.iter().map(|g| g.source_name.as_str()).collect();
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
    let paths: Vec<&str> = hits.iter().map(|h| h.unit.relative_path.as_str()).collect();
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
/// assert [`DOCUMENT_EXTRACTOR_IDENTITIES`] is EXACTLY that set. A new
/// document adapter that lands a new identity without updating the filter's
/// allowlist fails this test rather than silently falling out of (or into)
/// `--content document`.
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
        // office.rs/mail.rs each carry other `_EXTRACTOR`-shaped consts too
        // (e.g. a version constant) — the module-local ones this filter
        // actually needs are exactly `DOCX_EXTRACTOR`/`MAIL_EXTRACTOR`; text.rs
        // carries only the two it always has.
        expected.extend(extractor_constants(&text));
    }
    // ZIP_EXTRACTOR is a real `pub const ..._EXTRACTOR` too, and is
    // deliberately absent from the allowlist (a container carries no prose
    // of its own — see `DOCUMENT_EXTRACTOR_IDENTITIES`'s own doc) — named
    // here as the one expected mismatch this scan will find, so the loop
    // above stays a real assertion rather than one this test quietly
    // narrows to pass.
    let archive_text = std::fs::read_to_string(root.join("archive.rs")).expect("read archive.rs");
    expected.extend(extractor_constants(&archive_text));

    let mut allowlisted: BTreeSet<String> = DOCUMENT_EXTRACTOR_IDENTITIES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    // Restate the container exclusion explicitly rather than special-casing
    // the scan: the allowlist plus the one named exclusion must equal the
    // full set every document/container adapter module declares.
    allowlisted.insert(sergeant_rs::runtime::atlas::archive::ZIP_EXTRACTOR.to_string());

    assert_eq!(
        allowlisted, expected,
        "DOCUMENT_EXTRACTOR_IDENTITIES (plus the one named container exclusion, ZIP_EXTRACTOR) \
         must equal exactly the extractor constants text.rs/office.rs/mail.rs/archive.rs declare"
    );
    assert!(expected.contains(MARKDOWN_EXTRACTOR));
    assert!(expected.contains(TEXT_EXTRACTOR));
    assert!(expected.contains(DOCX_EXTRACTOR));
    assert!(expected.contains(MAIL_EXTRACTOR));
}

// -------------------------------------------------------------- --work scope

/// A `--work` filter reads a named repository exactly like `--source`
/// does — W1's whole point — and its answer states [`WorkScope::BaseOnly`]
/// rather than presenting itself as A2 §2's full "including overlay"
/// promise (H13.2). It excludes a different repository's generation the
/// identical way a plain `--source` filter would.
#[test]
fn a_work_base_selector_reads_like_its_named_repository_and_states_base_only_scope() {
    let estate = estate();
    let filter = Admissibility {
        source: SourceSelector::WorkBase {
            work_id: "01WORKID000000000000000000".to_string(),
            repository: "repo-a".to_string(),
        },
        authority: None,
    };
    assert_eq!(filter.source.work_scope(), WorkScope::BaseOnly);

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
    assert_eq!(
        SourceSelector::Named("repo-a".to_string()).work_scope(),
        WorkScope::NotWorkScoped
    );
    assert_eq!(SourceSelector::Any.work_scope(), WorkScope::NotWorkScoped);
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
    record_scan(&mut db, &mut journal, &scanned, None).expect("record manifest");

    let filter = named("manifest");

    // H13.1's own worked example: the config's key/table structure is
    // admissible through the code family, under `syntax-toml/v1`.
    let occurrences = db
        .admissible_occurrences(&filter, 500)
        .expect("admissible occurrences");
    assert!(
        occurrences
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
        units.len(),
        1,
        "Cargo.toml must leave exactly one fallback document unit, got {units:#?}"
    );
    assert_eq!(units[0].unit.relative_path, "Cargo.toml");
    assert_eq!(units[0].unit.kind, UnitKind::Document);
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
    record_scan(&mut db, &mut journal, &scan_a, None).expect("record data-a");
    record_scan(&mut db, &mut journal, &scan_b, None).expect("record data-b");

    let hits = db
        .admissible_datasets(&named("data-a"), 500)
        .expect("admissible datasets");
    let sources: Vec<&str> = hits.iter().map(|h| h.source_name.as_str()).collect();
    assert_eq!(sources, vec!["data-a"], "{hits:#?}");
    assert_eq!(hits[0].dataset.relative_path, "rows.csv");
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
/// Two exclusions are required and both are checked:
///
/// 1. **Another Work's overlay over the same repository is not admissible.**
///    `work:01OTHER…/repo-a` describes a world only that Work can see. A
///    filter that matched the `work:` prefix, or that reached for "anything
///    mentioning repo-a", would admit it — and admissibility decides what
///    may be SEEN, so that is a leak between two Works' surfaces, not a
///    ranking imprecision (A2 §8).
/// 2. **This Work's OWN overlay is not admissible either** — the honest
///    half. W1's `--work` is
///    [`WorkScope::BaseOnly`](sergeant_rs::runtime::atlas::db::WorkScope)
///    (H13.2), so the overlay is absent by design rather than by accident,
///    and the answer says so. When W1b lands, THIS assertion is the one
///    that must be revisited together with `work_scope()` — never one
///    without the other, which is exactly why they are pinned in the same
///    test.
#[test]
fn a_work_filter_excludes_a_different_works_generation() {
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
    record_scan(&mut db, &mut journal, &base, None).expect("record base");

    // Two overlay generations over that same base, one per Work, each under
    // its own `work:<id>/<repo>` source coordinate.
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
        record_scan(&mut db, &mut journal, &overlay, None).expect("record overlay");
    }

    let filter = Admissibility {
        source: SourceSelector::WorkBase {
            work_id: MINE.to_string(),
            repository: "repo-a".to_string(),
        },
        authority: None,
    };

    // Both overlay generations genuinely exist in the store, so the two
    // exclusions below are the filter's doing rather than an empty database's.
    for work_id in [MINE, OTHER] {
        let coordinate =
            sergeant_rs::runtime::atlas::overlay::overlay_source_name(work_id, "repo-a");
        assert!(
            !db.admissible_units(&named(&coordinate), 500)
                .expect("admissible units")
                .is_empty(),
            "the fixture must actually hold {coordinate}'s rows, or the exclusions prove nothing"
        );
    }

    let sources: BTreeSet<String> = db
        .admissible_generations(&filter, 500)
        .expect("admissible generations")
        .into_iter()
        .map(|generation| generation.source_name)
        .collect();
    assert_eq!(
        sources,
        BTreeSet::from(["repo-a".to_string()]),
        "--work admits exactly its repository's BASE generation: not another Work's overlay \
         over the same repository, and not (yet, H13.2) its own"
    );

    // And at the content tables too — a content-kind method that composed
    // the source predicate differently would leak another Work's surface
    // into this one's answer while `admissible_generations` stayed correct.
    let names: Vec<String> = db
        .admissible_occurrences(&filter, 500)
        .expect("admissible occurrences")
        .into_iter()
        .map(|hit| hit.occurrence.name)
        .collect();
    assert_eq!(
        names,
        vec!["widget_base".to_string()],
        "another Work's occurrences must never cross a --work filter: {names:?}"
    );

    // The limitation is declared, not silently applied (H13.2).
    assert_eq!(filter.source.work_scope(), WorkScope::BaseOnly);
}

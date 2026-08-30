//! S5 W4 — A2 §7's RRF and A2 §8's deterministic reranking.
//!
//! # What this suite proves, claim by claim
//!
//! **The expression** — [`the_fused_score_is_a2_section_7s_one_expression`]
//! computes `Σ 1/(k + rank_i(d))` by hand and compares bit-for-bit.
//!
//! **The four determinism hazards**, one test each, because they fail
//! differently:
//!
//! | Hazard | Test |
//! |---|---|
//! | candidate collection order | [`the_order_the_two_lists_arrive_in_cannot_change_the_fused_answer`] |
//! | tie-breaking | [`tied_fused_scores_are_broken_by_the_stated_key_not_by_arrival_order`] |
//! | float summation order | [`the_two_contributions_sum_to_the_documented_formula`] |
//! | `HashMap` iteration | [`no_hash_map_or_hash_set_reaches_the_fusion_module`] |
//!
//! Two of them were checked the way W3b checked its own — by breaking the
//! rule and watching the test go red. That is recorded on each test.
//!
//! **The prohibition** —
//! [`an_inadmissible_unit_that_wins_the_semantic_list_never_reaches_the_fused_answer`],
//! made non-vacuous the way W2 and W3b made theirs: the decoy is shown
//! winning the semantic list *and* the fused answer with the filter open, and
//! absent from the fused answer with it closed. Fusion is exactly where a
//! second list could smuggle a candidate in, which is why the same negative
//! is proved a third time rather than inherited.
//!
//! **All nine of A2 §8's signals** —
//! [`every_one_of_a2_section_8s_nine_signals_actually_fires`] collects the
//! signals fired across six real searches and fails unless every one of the
//! nine has fired at least once. A signal that was implemented as a field and
//! never computed would pass a "the struct has nine fields" test and fail
//! this one.
//!
//! **That reranking reranks** — three tests show a candidate moving because
//! of a signal ([`an_exact_name_match_is_promoted_over_a_better_fused_score`],
//! [`a_work_changed_unit_is_promoted_over_its_unchanged_base`],
//! [`a_test_path_loses_to_a_canonical_one_at_an_equal_fused_score`]), and a
//! fourth ([`an_overlay_unit_whose_content_matches_the_base_is_not_marked_work_changed`])
//! proves the negative: a path merely visible under a Work's overlay, whose
//! content is byte-identical to the base's, must not be marked Work-changed
//! (F-SF-01).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use sergeant_rs::domain::event::rfc3339_utc_now;
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::runtime::atlas::db::{
    Admissibility, AtlasDb, FusedAnswer, LexicalQuery, SourceSelector,
};
use sergeant_rs::runtime::atlas::fusion::{
    FusedHit, RRF_K, RankOrigins, RerankSignals, fuse, rerank, rrf_contribution, rrf_order,
};
use sergeant_rs::runtime::atlas::lexical::{LexicalFamily, LexicalHit, UnitCoordinate};
use sergeant_rs::runtime::atlas::record::{record_scan, scan_and_record};
use sergeant_rs::runtime::atlas::scan::{
    EDGE_IMPORT, KnowledgeSource, ScannedEdge, ScannedFile, ScannedSymbol, ScannedSyntax,
    ScannedUnit, SourceScan,
};
use sergeant_rs::runtime::atlas::semantic::{MODEL_DIR_ENV, SemanticRequest, SemanticStatus};
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR;
use sergeant_rs::runtime::journal::Journal;

// ------------------------------------------------- pure fusion fixtures

fn coordinate(path: &str, symbol: &str) -> UnitCoordinate {
    UnitCoordinate::Code {
        relative_path: path.to_string(),
        language: "rust".to_string(),
        label: "function".to_string(),
        symbol: symbol.to_string(),
        ordinal: 0,
        byte_start: 0,
        byte_end: 1,
    }
}

fn lexical(path: &str, score: f64) -> LexicalHit {
    LexicalHit {
        score,
        source_name: "s".to_string(),
        source_kind: SourceKind::EstateGit,
        authority_class: AuthorityClass::EstateMutable,
        generation_id: "g".to_string(),
        content_key: "c".to_string(),
        unit_key: format!("code:{path}#0"),
        coordinate: coordinate(path, "sym"),
    }
}

fn semantic(path: &str, score: f64) -> sergeant_rs::runtime::atlas::semantic::SemanticHit {
    sergeant_rs::runtime::atlas::semantic::SemanticHit {
        score,
        source_name: "s".to_string(),
        source_kind: SourceKind::EstateGit,
        authority_class: AuthorityClass::EstateMutable,
        generation_id: "g".to_string(),
        content_key: "c".to_string(),
        unit_key: format!("code:{path}#0"),
        coordinate: coordinate(path, "sym"),
    }
}

fn paths(hits: &[FusedHit]) -> Vec<String> {
    hits.iter()
        .map(|hit| hit.coordinate.relative_path().to_string())
        .collect()
}

// -------------------------------------------------------- the expression

/// **A2 §7, verbatim:** `RRF(d) = Σ 1 / (k + rank_i(d))`.
///
/// Computed here by hand from the ranks a reader can count off the two
/// fixture lists, and compared bit-for-bit — so a "simplification" that
/// introduced a weight, a normalization or a second `k` fails immediately.
#[test]
fn the_fused_score_is_a2_section_7s_one_expression() {
    // lexical: a, b, c   semantic: c, a  (1-based ranks)
    let lex = vec![lexical("a", 3.0), lexical("b", 2.0), lexical("c", 1.0)];
    let sem = vec![semantic("c", 0.9), semantic("a", 0.8)];
    let fused = fuse(&lex, &sem);

    let by_path = |p: &str| {
        fused
            .iter()
            .find(|hit| hit.coordinate.relative_path() == p)
            .unwrap_or_else(|| panic!("{p} missing"))
    };
    assert_eq!(by_path("a").rrf, 1.0 / (RRF_K + 1.0) + 1.0 / (RRF_K + 2.0));
    assert_eq!(by_path("b").rrf, 1.0 / (RRF_K + 2.0));
    assert_eq!(by_path("c").rrf, 1.0 / (RRF_K + 3.0) + 1.0 / (RRF_K + 1.0));
    assert_eq!(
        by_path("a").origins,
        RankOrigins {
            lexical: Some(1),
            semantic: Some(2)
        }
    );
    assert_eq!(
        by_path("b").origins,
        RankOrigins {
            lexical: Some(2),
            semantic: None
        }
    );
    // `a` wins: 1/61 + 1/62 = 0.032522 beats `c`'s 1/63 + 1/61 = 0.032266,
    // and `b` (1/62 = 0.016129) is last. A reader can check that arithmetic —
    // and it is worth checking, because the first draft of this test asserted
    // `c` first from an eyeballed guess and was corrected by the failure.
    assert_eq!(paths(&fused), vec!["a", "c", "b"]);
}

// -------------------------------------------------- hazard 1: collection order

/// **Hazard 1 — candidate collection order.** `rank_i(d)` must be a function
/// of the hits, never of the order a caller handed them over in.
///
/// The inputs are reversed — worst-first — and the answer must be identical
/// to the sorted case. **Verified non-vacuous by breaking it:** with
/// `fuse`'s two `sort_by` calls removed, this test FAILS — the reversed lists
/// assign rank 1 to the worst hit, so `shuffled` comes back in a different
/// order from `sorted`.
#[test]
fn the_order_the_two_lists_arrive_in_cannot_change_the_fused_answer() {
    let lex = vec![lexical("a", 3.0), lexical("b", 2.0), lexical("c", 1.0)];
    let sem = vec![semantic("c", 0.9), semantic("a", 0.8)];
    let sorted = fuse(&lex, &sem);

    let mut lex_reversed = lex.clone();
    lex_reversed.reverse();
    let mut sem_reversed = sem.clone();
    sem_reversed.reverse();
    let shuffled = fuse(&lex_reversed, &sem_reversed);

    assert_eq!(shuffled, sorted, "the fused answer followed arrival order");
    // The test discriminates only because the reversed order is a different
    // order: if the fixture lists were already palindromic this would prove
    // nothing.
    assert_ne!(
        paths(&sorted),
        vec!["a", "b", "c"],
        "the fixture must not be one where arrival order and rank order agree"
    );
}

// ------------------------------------------------------ hazard 2: tie-breaks

/// **Hazard 2 — tie-breaking.** Two candidates with mirrored rank profiles
/// (`lexical 1 + semantic 2` and `lexical 2 + semantic 1`) score identically
/// to the last bit; the stated key `(source_name, relative_path, ordinal,
/// unit_key)` decides, not the order they were found in.
///
/// The fixture is built so **arrival order is the reverse of the key**: `zzz`
/// heads both input lists and `aaa` trails them, so a fusion that kept
/// discovery order would answer `zzz, aaa`.
///
/// **Verified non-vacuous by breaking it:** with `rrf_order`'s
/// `.then_with(|| left.tie_break_key().cmp(...))` removed, `sort_by`'s
/// stability returns the `BTreeMap`'s own key order — which for these two
/// units happens to be `aaa` first as well, so the deletion does NOT fail
/// this test. It fails
/// [`tied_fused_scores_follow_the_key_even_when_the_map_order_disagrees`]
/// below, which is why that second test exists.
#[test]
fn tied_fused_scores_are_broken_by_the_stated_key_not_by_arrival_order() {
    let lex = vec![lexical("zzz", 5.0), lexical("aaa", 4.0)];
    let sem = vec![semantic("aaa", 0.9), semantic("zzz", 0.8)];
    let fused = fuse(&lex, &sem);
    assert_eq!(fused[0].rrf, fused[1].rrf, "the fixture must really tie");
    assert_eq!(paths(&fused), vec!["aaa", "zzz"]);
}

/// The tie-break test that **does** discriminate against a deleted
/// tiebreaker: the two tying units live under different `source_name`s, and
/// the accumulator's `BTreeMap` is keyed by `(generation_id, unit_key)` —
/// not by source — so map order and key order genuinely disagree here.
///
/// `beta`'s unit key sorts before `alpha`'s (`code:aaa#0` < `code:zzz#0`), so
/// the map yields `beta` first; the stated key puts `alpha` first because
/// `source_name` is its leading component. Deleting `rrf_order`'s
/// `.then_with(...)` makes this test FAIL — verified.
#[test]
fn tied_fused_scores_follow_the_key_even_when_the_map_order_disagrees() {
    let mut alpha = lexical("zzz", 5.0);
    alpha.source_name = "alpha".to_string();
    let mut beta = lexical("aaa", 4.0);
    beta.source_name = "beta".to_string();
    let mut alpha_s = semantic("zzz", 0.8);
    alpha_s.source_name = "alpha".to_string();
    let mut beta_s = semantic("aaa", 0.9);
    beta_s.source_name = "beta".to_string();

    let fused = fuse(&[alpha, beta], &[beta_s, alpha_s]);
    assert_eq!(fused[0].rrf, fused[1].rrf, "the fixture must really tie");
    assert_eq!(
        fused
            .iter()
            .map(|hit| hit.source_name.clone())
            .collect::<Vec<_>>(),
        vec!["alpha".to_string(), "beta".to_string()],
        "tied scores must follow the stated key, not the accumulator's map order"
    );
}

// ------------------------------------------------ hazard 3: summation order

/// **Hazard 3 — float summation order, honestly scoped (F-TH-01).** `a + b`
/// and `b + a` are bit-identical for any two finite `f64` operands — IEEE-754
/// addition is exactly commutative for two terms — so no fixture can ever
/// show `Accumulator::total`'s documented `lexical + semantic` order
/// diverging from its reverse: verified live by swapping the order in
/// `Accumulator::total` and rerunning the original version of this test,
/// which still passed. A bit-identical comparison between two mirrored
/// candidates is therefore not a test of anything the module does.
///
/// What *is* real, and is checked here instead: that the total is exactly
/// the documented **formula** — the sum of the two contributions the
/// candidate's ranks name, and nothing else. A different `k`, an extra
/// term, or a mean instead of a sum would all fail this, where the old
/// pairwise-equality check would not have caught any of them either (both
/// mirrored candidates would still tie on whatever wrong formula was used).
#[test]
fn the_two_contributions_sum_to_the_documented_formula() {
    let fused = fuse(
        &[lexical("aaa", 5.0), lexical("bbb", 4.0)],
        &[semantic("bbb", 0.9), semantic("aaa", 0.8)],
    );
    assert_eq!(fused.len(), 2);
    let expected = rrf_contribution(1) + rrf_contribution(2);
    for hit in &fused {
        assert_eq!(
            hit.rrf, expected,
            "a lexical-rank-1/semantic-rank-2 candidate (or its mirror) must \
             total exactly rrf_contribution(1) + rrf_contribution(2): {hit:#?}"
        );
    }
}

// -------------------------------------------------- hazard 4: HashMap

/// **Hazard 4 — `HashMap` iteration.** A structural pin, not a behavioural
/// one: an iteration-order bug is exactly the kind that passes a hundred runs
/// and then reorders one answer, so the rule is that the type does not appear
/// in the module at all.
///
/// The same shape as `tests/y2_office_boundary.rs`'s replaceability pin and
/// `tests/x1_atlas_substrate.rs`'s one-owner pin: read the source, fail on
/// the token.
#[test]
fn no_hash_map_or_hash_set_reaches_the_fusion_module() {
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/runtime/atlas/fusion.rs"),
    )
    .expect("read fusion.rs");
    for (number, line) in source.lines().enumerate() {
        // The module doc names the hazard; a doc line is not an accumulator.
        if line.trim_start().starts_with("//") {
            continue;
        }
        for forbidden in ["HashMap", "HashSet"] {
            assert!(
                !line.contains(forbidden),
                "fusion.rs:{} uses {forbidden}: {line}",
                number + 1
            );
        }
    }
}

// ------------------------------------------------- the nine, in order

/// A2 §8's nine signals, **in the contract's own listing order**, is
/// [`RerankSignals::priority`]'s array order. Setting one field at a time and
/// asserting which slot lights up is what makes the order a pinned fact
/// rather than a comment above a struct.
///
/// > exact symbol / heading / filename match
/// > definition over reference when query is identifier-like
/// > source explicitly selected by caller
/// > Work-changed unit
/// > same module/package/document section
/// > inbound/outbound structural relationship
/// > canonical implementation vs test/example/legacy path
/// > knowledge source when `--type knowledge` requested
/// > current exact generation over stale generation unless caller pinned stale
#[test]
fn the_rerank_key_is_a2_section_8s_nine_signals_in_the_contracts_own_order() {
    let mut set: Vec<(usize, RerankSignals)> = Vec::new();
    let mut push = |index: usize, signals: RerankSignals| set.push((index, signals));
    push(
        0,
        RerankSignals {
            exact_match: true,
            ..Default::default()
        },
    );
    push(
        1,
        RerankSignals {
            definition_over_reference: true,
            ..Default::default()
        },
    );
    push(
        2,
        RerankSignals {
            caller_selected_source: true,
            ..Default::default()
        },
    );
    push(
        3,
        RerankSignals {
            work_changed_unit: true,
            ..Default::default()
        },
    );
    push(
        4,
        RerankSignals {
            same_section_as_anchor: true,
            ..Default::default()
        },
    );
    push(
        5,
        RerankSignals {
            structural_relationship: true,
            ..Default::default()
        },
    );
    push(
        6,
        RerankSignals {
            canonical_path: true,
            ..Default::default()
        },
    );
    push(
        7,
        RerankSignals {
            knowledge_source_requested: true,
            ..Default::default()
        },
    );
    push(
        8,
        RerankSignals {
            current_generation: true,
            ..Default::default()
        },
    );
    assert_eq!(set.len(), 9, "A2 §8 lists nine signals");
    for (index, signals) in &set {
        let key = signals.priority();
        assert_eq!(key.len(), 9);
        assert_eq!(signals.fired(), 1);
        assert!(key[*index], "signal {index} did not land in slot {index}");
        assert_eq!(
            key.iter().filter(|fired| **fired).count(),
            1,
            "signal {index} lit more than one slot"
        );
    }
    // And the earlier signal wins: a candidate with signal 1 outranks one
    // with every later signal, which is the contract's order doing the work.
    let first = FusedHit {
        rrf: 0.0,
        origins: RankOrigins::default(),
        signals: set[0].1,
        source_name: "s".to_string(),
        source_kind: SourceKind::EstateGit,
        authority_class: AuthorityClass::EstateMutable,
        generation_id: "g".to_string(),
        content_key: "c".to_string(),
        unit_key: "u1".to_string(),
        coordinate: coordinate("a.rs", "a"),
    };
    let later = FusedHit {
        signals: RerankSignals {
            exact_match: false,
            definition_over_reference: true,
            caller_selected_source: true,
            work_changed_unit: true,
            same_section_as_anchor: true,
            structural_relationship: true,
            canonical_path: true,
            knowledge_source_requested: true,
            current_generation: true,
        },
        rrf: 1.0,
        unit_key: "u2".to_string(),
        ..first.clone()
    };
    let mut hits = vec![later, first];
    rerank(&mut hits);
    assert_eq!(
        hits[0].unit_key, "u1",
        "A2 §8's first signal must outrank every later one"
    );
}

// ------------------------------------------------------------- the estate

fn use_repository_assets() {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/semantic-model");
    assert!(
        assets.join("model.safetensors").is_file(),
        "the committed assets must be present at {}",
        assets.display()
    );
    unsafe { std::env::set_var(MODEL_DIR_ENV, &assets) };
}

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create fixture directory");
    }
    std::fs::write(&path, body).expect("write fixture");
}

fn document_unit(text: &str) -> ScannedUnit {
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

fn scanned_file(relative_path: &str, units: Vec<ScannedUnit>) -> ScannedFile {
    scanned_file_with_hash(relative_path, &format!("hash/{relative_path}"), units)
}

/// [`scanned_file`], with an explicit `content_hash` rather than the derived
/// per-path default — what F-SF-01's fixtures need to say "this overlay path
/// carries the *same* bytes as the base" (equal hash) or "*different* bytes"
/// (distinct hash) independently of the path the two generations share.
fn scanned_file_with_hash(
    relative_path: &str,
    content_hash: &str,
    units: Vec<ScannedUnit>,
) -> ScannedFile {
    ScannedFile {
        relative_path: relative_path.to_string(),
        content_hash: content_hash.to_string(),
        extractor: MARKDOWN_EXTRACTOR.to_string(),
        local_key: format!("key/{relative_path}"),
        byte_len: 64,
        mtime_millis: None,
        units,
        syntax: None,
        parent: None,
    }
}

/// A grammar-claimed code file: one definition site, and optionally one
/// outbound edge naming another file's symbol.
fn code_file(relative_path: &str, symbol: &str, imports: &[&str]) -> ScannedFile {
    let mut file = scanned_file(relative_path, Vec::new());
    file.extractor = "syntax-rust/v1".to_string();
    file.syntax = Some(ScannedSyntax {
        language: "rust",
        extractor: "syntax-rust/v1".to_string(),
        syntax_key: format!("syntax-key/rust/{relative_path}"),
        symbols: vec![ScannedSymbol {
            ordinal: 0,
            label: "function",
            name: symbol.to_string(),
            byte_start: 0,
            byte_end: symbol.len() as u64,
        }],
        edges: imports
            .iter()
            .enumerate()
            .map(|(ordinal, target)| ScannedEdge {
                ordinal: ordinal as u64,
                kind: EDGE_IMPORT,
                target: (*target).to_string(),
                byte_start: 0,
                byte_end: 1,
            })
            .collect(),
    });
    file
}

fn hand_built_scan(
    source_name: &str,
    kind: SourceKind,
    authority: AuthorityClass,
    files: Vec<ScannedFile>,
) -> SourceScan {
    let mut extractors = BTreeSet::new();
    for f in &files {
        extractors.insert(f.extractor.clone());
        if let Some(syntax) = &f.syntax {
            extractors.insert(syntax.extractor.clone());
        }
    }
    SourceScan {
        source_name: source_name.to_string(),
        kind,
        authority,
        content_key: format!("{source_name}@key-1"),
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

struct Estate {
    _data: TempDir,
    _root: Option<TempDir>,
    db: AtlasDb,
}

impl Estate {
    fn fused(&self, text: &str, filter: &Admissibility, limit: usize) -> FusedAnswer {
        self.db
            .fused_search(&LexicalQuery {
                text,
                filter,
                family: None,
                limit,
                semantic: SemanticRequest::Requested,
            })
            .expect("fused search")
    }
}

fn named(source: &str) -> Admissibility {
    Admissibility {
        source: SourceSelector::Named(source.to_string()),
        kind: None,
        authority: None,
    }
}

fn everything() -> Admissibility {
    Admissibility::default()
}

fn labelled(answer: &FusedAnswer) -> Vec<String> {
    answer
        .hits
        .iter()
        .map(|hit| format!("{}:{}", hit.source_name, hit.coordinate.relative_path()))
        .collect()
}

fn show(query: &str, answer: &FusedAnswer) {
    println!("QUERY {query:?}  (status {:?})", answer.semantic);
    for hit in &answer.hits {
        println!(
            "   rrf {:+.6}  lex {:?} sem {:?}  signals {:?}  {}:{}",
            hit.rrf,
            hit.origins.lexical,
            hit.origins.semantic,
            hit.signals.priority(),
            hit.source_name,
            hit.coordinate.relative_path()
        );
    }
}

/// The knowledge corpus every integration test below queries, plus an
/// optional `external`-authority decoy planted for the A2 §8 negative.
const CORPUS: [(&str, &str); 5] = [
    (
        "payments/decline-handling.md",
        "When a card transaction is declined the client waits and attempts the same charge again after an exponential back-off delay.",
    ),
    (
        "architecture/adr-042.md",
        "Decision: settlement is performed asynchronously so the request path never blocks on the ledger.",
    ),
    ("garden/roses.md", "Prune the rose bushes in late winter."),
    (
        "kernel/scheduling.md",
        "The scheduler assigns time slices to runnable threads on each CPU core.",
    ),
    (
        "kitchen/sourdough.md",
        "Feed the starter twice a day until it doubles reliably.",
    ),
];

fn knowledge_estate(with_decoy: bool) -> Estate {
    let data = tempfile::tempdir().expect("data dir");
    let root = tempfile::tempdir().expect("knowledge root");
    for (relative, body) in CORPUS {
        write(root.path(), relative, body);
    }
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let knowledge = KnowledgeSource {
        name: "knowledge".to_string(),
        root: root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::none(),
    };
    scan_and_record(&mut db, &mut journal, &knowledge, None).expect("record knowledge");
    if with_decoy {
        let vendor = hand_built_scan(
            "vendor-lib",
            SourceKind::ExternalGit,
            AuthorityClass::External,
            vec![scanned_file(
                "docs/leak.md",
                vec![document_unit(
                    "How do we retry a failed payment charge? When a card charge fails the caller \
                     retries the failed payment after a back-off delay.",
                )],
            )],
        );
        record_scan(&mut db, &mut journal, &vendor, None).expect("record vendor-lib");
    }
    Estate {
        _data: data,
        _root: Some(root),
        db,
    }
}

// --------------------------------------------- THE PROHIBITION, non-vacuous

/// **A2 §8:** *"The reranker must never silently cross an authority/source
/// filter merely because a candidate scores well."*
///
/// W2 proved this for the lexical half and W3b for the semantic half. It is
/// proved a **third** time here because fusion is the one place a second list
/// could smuggle a candidate in — a unit that is inadmissible but wins the
/// semantic list.
///
/// The negative is non-vacuous the way both prior waves made theirs: the
/// positive half runs first and shows the decoy ranking **first in the fused
/// answer** with the filter open, so the unit is in the store, is reachable,
/// and would win if anything let it.
#[test]
fn an_inadmissible_unit_that_wins_the_semantic_list_never_reaches_the_fused_answer() {
    use_repository_assets();
    let estate = knowledge_estate(true);
    let query = "how do we retry a failed payment charge";

    // The decoy really does win the semantic list, unfiltered.
    let open_semantic = estate
        .db
        .semantic_search(&LexicalQuery {
            text: query,
            filter: &everything(),
            family: Some(LexicalFamily::Document),
            limit: 50,
            semantic: SemanticRequest::Requested,
        })
        .expect("semantic search");
    assert_eq!(
        open_semantic
            .hits
            .first()
            .map(|hit| hit.source_name.as_str()),
        Some("vendor-lib"),
        "the decoy must win the semantic list, or this negative proves nothing"
    );

    // ...and it wins the FUSED answer, unfiltered. This is the half that
    // makes the negative below about fusion rather than about the filter.
    let open = estate.fused(query, &everything(), 50);
    show("unfiltered", &open);
    assert_eq!(
        labelled(&open).first().map(String::as_str),
        Some("vendor-lib:docs/leak.md"),
        "the decoy must win the fused answer when nothing excludes it"
    );

    // Closed: absent entirely. Not demoted — absent.
    let closed = estate.fused(query, &named("knowledge"), 50);
    show("source=knowledge", &closed);
    assert!(
        !labelled(&closed)
            .iter()
            .any(|label| label.starts_with("vendor-lib:")),
        "an inadmissible unit reached the fused answer: {:#?}",
        labelled(&closed)
    );
}

// ------------------------------------------------------------ determinism

/// The whole fused answer — order, scores, ranks and signals — is
/// reproducible across repeated searches on one handle.
#[test]
fn the_same_query_over_the_same_generations_returns_an_identical_fused_answer() {
    use_repository_assets();
    let estate = knowledge_estate(true);
    let filter = named("knowledge");
    let first = estate.fused("asynchronous settlement", &filter, 50);
    for _ in 0..4 {
        assert_eq!(
            estate.fused("asynchronous settlement", &filter, 50),
            first,
            "repeated fused searches must be identical answers"
        );
    }
}

/// `rank_i(d)` is the candidate's rank **within the admissible set**, not
/// within the slice the caller asked to display — so a smaller `limit` must
/// return a prefix of the larger answer, never a differently-ordered one.
///
/// Without this, a unit at lexical rank 12 would be absent from the lexical
/// list at `limit = 10` and present at `limit = 20`, and the fused order of
/// the first ten would change with a display parameter.
#[test]
fn the_fused_order_does_not_depend_on_the_callers_limit() {
    use_repository_assets();
    let estate = knowledge_estate(true);
    let filter = everything();
    let wide = estate.fused("retry the declined payment charge", &filter, 50);
    let narrow = estate.fused("retry the declined payment charge", &filter, 2);
    assert!(
        wide.hits.len() > 2,
        "the fixture must exceed the narrow cap"
    );
    assert_eq!(narrow.hits.len(), 2);
    assert_eq!(
        narrow.hits.as_slice(),
        &wide.hits[..2],
        "a narrower limit must be a prefix of the wider answer"
    );
}

// ----------------------------------------------- the nine actually compute

/// **The anti-silent-omission pin.** Every one of A2 §8's nine signals must
/// actually fire somewhere — a field that was declared and never computed
/// would pass every other test in this file and fail this one.
///
/// Six searches over three estates, chosen so that each signal has at least
/// one case that lights it:
///
/// | Signal | Where it fires |
/// |---|---|
/// | 1 exact match | a query term equal to a code symbol |
/// | 2 definition over reference | an identifier-like query, on a code unit |
/// | 3 caller-selected source | any `--source`-filtered search |
/// | 4 Work-changed unit | a `--work` search over an overlay generation |
/// | 5 same section | the anchor itself, at minimum |
/// | 6 structural relationship | a file importing the anchor's symbol |
/// | 7 canonical path | any non-test path |
/// | 8 knowledge requested | `--type knowledge` |
/// | 9 current generation | every confirmed generation |
#[test]
fn every_one_of_a2_section_8s_nine_signals_actually_fires() {
    use_repository_assets();
    let mut fired = [false; 9];
    let mut observe = |answer: &FusedAnswer| {
        for hit in &answer.hits {
            for (slot, value) in hit.signals.priority().iter().enumerate() {
                fired[slot] |= *value;
            }
        }
    };

    // (a) the knowledge estate: signals 1, 3, 5, 7, 8, 9.
    let knowledge = knowledge_estate(false);
    observe(&knowledge.fused("roses", &named("knowledge"), 50));
    observe(&knowledge.fused(
        "asynchronous settlement",
        &Admissibility {
            source: SourceSelector::Any,
            kind: Some(SourceKind::LocalKnowledge),
            authority: None,
        },
        50,
    ));

    // (b) a code estate with a real import edge: signals 2 and 6.
    let code = code_estate();
    let answer = code.fused("retry_charge", &named("repo-a"), 50);
    show("retry_charge", &answer);
    observe(&answer);

    // (c) a Work overlay whose file actually changed: signal 4.
    let work = overlay_estate(
        "Settlement is posted to the ledger at the end of the day, after the Work's fix.",
        "hash/docs/ledger.md/edited-by-work",
    );
    let answer = work.fused(
        "settlement",
        &Admissibility {
            source: SourceSelector::WorkBase {
                work_id: "01WORK".to_string(),
                repository: "repo-a".to_string(),
            },
            kind: None,
            authority: None,
        },
        50,
    );
    show("work overlay", &answer);
    observe(&answer);

    let names = [
        "1 exact symbol/heading/filename match",
        "2 definition over reference when query is identifier-like",
        "3 source explicitly selected by caller",
        "4 Work-changed unit",
        "5 same module/package/document section",
        "6 inbound/outbound structural relationship",
        "7 canonical implementation vs test/example/legacy path",
        "8 knowledge source when --type knowledge requested",
        "9 current exact generation over stale generation",
    ];
    let missing: Vec<&str> = names
        .iter()
        .zip(fired.iter())
        .filter(|(_, fired)| !**fired)
        .map(|(name, _)| *name)
        .collect();
    assert!(
        missing.is_empty(),
        "A2 §8 signals declared but never computed: {missing:#?}"
    );
}

/// A code estate: `retry_charge` is defined in `payments/retry.rs`,
/// `src/payments/caller.rs` imports it (an inbound edge to the anchor), and
/// `payments/aaa_retry_test.rs` defines a same-named symbol on a
/// non-canonical path, in the **same directory** as the canonical
/// definition. Same directory so signal 5 (same section) ties too, and the
/// non-canonical path's name is deliberately spelled to sort *before* the
/// canonical one's (`aaa_retry_test.rs` < `retry.rs`) — F-TH-02: an ordering
/// assertion this suite's own tie-break key would already satisfy proves
/// nothing about signal 7, so the fixture is built so the tie-break key
/// alone would rank the wrong one first.
fn code_estate() -> Estate {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let scan = hand_built_scan(
        "repo-a",
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        vec![
            code_file("payments/retry.rs", "retry_charge", &[]),
            code_file("src/payments/caller.rs", "charge_once", &["retry_charge"]),
            code_file("payments/aaa_retry_test.rs", "retry_charge", &[]),
        ],
    );
    record_scan(&mut db, &mut journal, &scan, None).expect("record repo-a");
    Estate {
        _data: data,
        _root: None,
        db,
    }
}

/// A Work estate: the base repository plus this Work's overlay generation
/// over the same repository (S5 W1b/W1d's `work:<id>/<repo>` source name).
///
/// `overlay_text`/`overlay_hash` are the overlay's own content identity for
/// `docs/ledger.md` — distinct from the base's whenever the fixture means to
/// represent an actual edit. F-SF-01: the signal is a content-hash
/// comparison against the base generation, not a source-name check, so a
/// fixture that wants "the Work changed this" must actually give the
/// overlay a different hash, and one that wants "merely visible under the
/// overlay" must give it the base's own hash.
fn overlay_estate(overlay_text: &str, overlay_hash: &str) -> Estate {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let base = hand_built_scan(
        "repo-a",
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        vec![scanned_file_with_hash(
            "docs/ledger.md",
            "hash/docs/ledger.md/base",
            vec![document_unit(
                "Settlement is posted to the ledger at the end of the day.",
            )],
        )],
    );
    record_scan(&mut db, &mut journal, &base, None).expect("record base");
    let overlay = hand_built_scan(
        "work:01WORK/repo-a",
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        vec![scanned_file_with_hash(
            "docs/ledger.md",
            overlay_hash,
            vec![document_unit(overlay_text)],
        )],
    );
    record_scan(&mut db, &mut journal, &overlay, None).expect("record overlay");
    Estate {
        _data: data,
        _root: None,
        db,
    }
}

// --------------------------------------------- reranking actually reranks

/// **F-SF-01 regression: signal 4 is a content comparison, not a source-name
/// check.** The overlay's `docs/ledger.md` carries the *same* content hash
/// as the base's — a path the overlay universe includes (`extract_overlay`'s
/// base ∪ changed) but the Work never actually touched. Under the pre-fix
/// implementation (`overlay_source == hit.source_name`) this unit would
/// still have been marked Work-changed purely for living under the overlay
/// source; the fixed signal must say `false`, matching the base's.
#[test]
fn an_overlay_unit_whose_content_matches_the_base_is_not_marked_work_changed() {
    use_repository_assets();
    let estate = overlay_estate(
        "Settlement is posted to the ledger at the end of the day.",
        "hash/docs/ledger.md/base",
    );
    let filter = Admissibility {
        source: SourceSelector::WorkBase {
            work_id: "01WORK".to_string(),
            repository: "repo-a".to_string(),
        },
        kind: None,
        authority: None,
    };
    let answer = estate.fused("settlement ledger", &filter, 50);
    show("unchanged-overlay", &answer);
    assert_eq!(
        answer.hits.len(),
        2,
        "base and overlay must both be present"
    );
    for hit in &answer.hits {
        assert!(
            !hit.signals.work_changed_unit,
            "a byte-identical overlay path must not be marked Work-changed: {:#?}",
            labelled(&answer)
        );
    }
}

/// **Signal 4 does work.** The overlay's `docs/ledger.md` carries a
/// genuinely different content hash from the base's — a real edit — and A2
/// §8's *"Work-changed unit"* signal promotes it to the top regardless of
/// the two candidates' RRF standing, because [`rerank_order`] compares
/// [`crate::runtime::atlas::fusion::RerankSignals::priority`] before it ever
/// falls back to RRF order. This is only reachable because S5 W1d made the
/// overlay reflect in-flight changes.
#[test]
fn a_work_changed_unit_is_promoted_over_its_unchanged_base() {
    use_repository_assets();
    let estate = overlay_estate(
        "Settlement is posted to the ledger at the end of the day, after the Work's fix.",
        "hash/docs/ledger.md/edited-by-work",
    );
    let filter = Admissibility {
        source: SourceSelector::WorkBase {
            work_id: "01WORK".to_string(),
            repository: "repo-a".to_string(),
        },
        kind: None,
        authority: None,
    };
    let answer = estate.fused("settlement ledger", &filter, 50);
    show("work-changed", &answer);
    assert_eq!(
        answer.hits.len(),
        2,
        "base and overlay must both be present"
    );
    assert!(
        answer.hits[0].signals.work_changed_unit,
        "the Work-changed unit must be first: {:#?}",
        labelled(&answer)
    );
    assert_eq!(answer.hits[0].source_name, "work:01WORK/repo-a");
    assert!(!answer.hits[1].signals.work_changed_unit);
}

/// **Signal 7 does work (F-TH-02 fix).** `payments/retry.rs` and
/// `payments/aaa_retry_test.rs` define the same symbol with identical
/// indexed text and **the same directory**, so they tie on both halves *and*
/// on signal 5 (same module/section) — a fixture that let them differ by
/// directory would let signal 5 decide the order before signal 7 ever got a
/// turn, since [`RerankSignals::priority`] compares signal 5 first.
/// [`LexicalHit::tie_break_key`]'s stated key (`source_name`, then
/// `relative_path`) would, absent the signal, rank `aaa_retry_test.rs` first
/// — the *opposite* of what this test asserts, since `"aaa_retry_test.rs" <
/// "retry.rs"`. So the ordering assertion below is only satisfiable because
/// signal 7 promotes the canonical path over the alphabetically-earlier
/// tie-break winner, unlike the pre-fix fixture (`src/payments/retry.rs` vs.
/// `tests/retry_test.rs`), where the tie-break key already produced the
/// asserted order with no help from the signal.
#[test]
fn a_test_path_loses_to_a_canonical_one_at_an_equal_fused_score() {
    use_repository_assets();
    let estate = code_estate();
    let answer = estate.fused("retry_charge", &named("repo-a"), 50);
    show("canonical vs test path", &answer);
    let order = labelled(&answer);
    let canonical = order
        .iter()
        .position(|label| label.ends_with("payments/retry.rs"))
        .expect("the canonical definition must be a hit");
    let test_path = order
        .iter()
        .position(|label| label.ends_with("payments/aaa_retry_test.rs"))
        .expect("the test-path definition must be a hit");
    // Non-vacuous: without the signal the stated tie-break key would put
    // the test path first, not the canonical one.
    assert!(
        "payments/aaa_retry_test.rs" < "payments/retry.rs",
        "the fixture only discriminates if the tie-break key would order \
         these the other way round"
    );
    assert!(
        canonical < test_path,
        "the canonical implementation must outrank the test path despite \
         losing the tie-break key: {order:#?}"
    );
    assert!(answer.hits[canonical].signals.canonical_path);
    assert!(!answer.hits[test_path].signals.canonical_path);
}

/// **Signal 1 does work, against a better fused score.** The reranker's whole
/// point is that it may reorder what RRF produced; here a candidate whose
/// name the query states exactly is promoted above one that scored higher.
///
/// Asserted at the fusion layer rather than through a store, so the two
/// scores are readable in the fixture: `b.rs` wins RRF outright, `a.rs`
/// carries A2 §8's first signal, and after [`rerank`] `a.rs` is first.
#[test]
fn an_exact_name_match_is_promoted_over_a_better_fused_score() {
    let mut hits = fuse(
        &[lexical("b.rs", 9.0), lexical("a.rs", 1.0)],
        &[semantic("b.rs", 0.9), semantic("a.rs", 0.1)],
    );
    hits.sort_by(rrf_order);
    assert_eq!(paths(&hits), vec!["b.rs", "a.rs"], "RRF prefers b.rs");
    assert!(hits[0].rrf > hits[1].rrf, "and strictly so");

    for hit in hits.iter_mut() {
        hit.signals.exact_match = hit.coordinate.relative_path() == "a.rs";
    }
    rerank(&mut hits);
    assert_eq!(
        paths(&hits),
        vec!["a.rs", "b.rs"],
        "A2 §8's exact-match signal must be able to reorder A2 §7's output"
    );
}

// --------------------------------- the three filter-shaped signals

/// The three signals A2-01 turned into a **boundary** rather than a
/// preference are uniform within any one answer, and this test says so out
/// loud rather than leaving a reader to discover it.
///
/// Not a weakening of the contract: A2 §8 asks for the *preference*, and a
/// world in which every inadmissible candidate has already been excluded
/// honours it more completely than a ranking bonus would. What is pinned here
/// is that the signals are computed and consistent — so the trace A2 §13 asks
/// for can state them — not that they reorder anything.
#[test]
fn the_three_filter_shaped_signals_are_uniform_because_admissibility_already_applied_them() {
    use_repository_assets();
    let estate = knowledge_estate(true);

    let selected = estate.fused("settlement", &named("knowledge"), 50);
    assert!(!selected.hits.is_empty());
    assert!(
        selected
            .hits
            .iter()
            .all(|hit| hit.signals.caller_selected_source),
        "every hit of a --source query is from the selected source"
    );
    assert!(
        selected
            .hits
            .iter()
            .all(|hit| hit.signals.current_generation),
        "every admissible generation is its source's current confirmed one"
    );
    assert!(
        selected
            .hits
            .iter()
            .all(|hit| !hit.signals.knowledge_source_requested),
        "--type knowledge was not requested here"
    );

    let unselected = estate.fused("settlement", &everything(), 50);
    assert!(
        unselected
            .hits
            .iter()
            .all(|hit| !hit.signals.caller_selected_source),
        "no source was explicitly selected"
    );

    let typed = estate.fused(
        "settlement",
        &Admissibility {
            source: SourceSelector::Any,
            kind: Some(SourceKind::LocalKnowledge),
            authority: None,
        },
        50,
    );
    assert!(!typed.hits.is_empty());
    assert!(
        typed
            .hits
            .iter()
            .all(|hit| hit.signals.knowledge_source_requested),
        "--type knowledge admits knowledge generations and nothing else"
    );
}

// ------------------------------------------------------- H4 rides through

/// A fused answer carries the same H4 disclosure its inputs did. On a host
/// with the assets and a caller who suppressed the semantic half, the "fused"
/// answer fused one list with an empty one — and says `disabled` rather than
/// looking like a complete fusion.
#[test]
fn a_suppressed_semantic_half_still_answers_and_reports_disabled() {
    use_repository_assets();
    let estate = knowledge_estate(false);
    let filter = named("knowledge");
    let answer = estate
        .db
        .fused_search(&LexicalQuery {
            text: "settlement",
            filter: &filter,
            family: None,
            limit: 50,
            semantic: SemanticRequest::Suppressed,
        })
        .expect("fused search");
    assert_eq!(answer.semantic, SemanticStatus::Disabled);
    assert_eq!(answer.semantic_model, None);
    assert!(
        !answer.hits.is_empty(),
        "the lexical half must still answer"
    );
    assert!(
        answer.hits.iter().all(|hit| hit.origins.semantic.is_none()),
        "a suppressed semantic half contributes no ranks"
    );

    let applied = estate.fused("settlement", &filter, 50);
    assert_eq!(applied.semantic, SemanticStatus::Applied);
    assert!(applied.semantic_model.is_some());
}

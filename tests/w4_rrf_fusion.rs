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

/// **S6 D1 — A2 §2 stage 1's estate coordinate.** This suite is
/// single-estate: every generation it records is bound to this one root and
/// every filter it builds is admitted from it. The cross-estate case — two
/// estates on one host daemon, which is where the axis actually earns its
/// keep — is `tests/d1_estate_isolation.rs`, deliberately not folded in
/// here, because a suite that never crosses estates cannot notice an estate
/// filter that does nothing (that is exactly how the leak survived: this
/// file's ancestors all passed).
#[allow(dead_code)]
const D1_ESTATE: &str = "/estates/w4_rrf_fusion";

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use sergeant_rs::domain::event::rfc3339_utc_now;
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::runtime::atlas::db::{
    Admissibility, AtlasDb, FusedAnswer, LexicalQuery, SourceSelector,
};
use sergeant_rs::runtime::atlas::fusion::{
    ALPHA_NATURAL, ALPHA_SYMBOL, BOOST_ADJACENCY, BOOST_DEFINITION, BOOST_EXACT_MATCH,
    BOOST_WORK_CHANGED, FILE_COHERENCE_BOOST_FRAC, FILE_SATURATION_DECAY, FusedHit,
    PENALTY_NON_CANONICAL, RRF_K, RankOrigins, RerankSignals, fuse, is_symbol_query,
    STEM_BOOST_MULTIPLIER, path_stem_match_ratio, rerank, resolve_alpha, rrf_contribution,
    rrf_order,
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

/// **A2 §7's expression, α-blended:**
/// `α·(1/(k + rank_sem)) + (1−α)·(1/(k + rank_lex))`.
///
/// Computed here by hand from the ranks a reader can count off the two
/// fixture lists, and compared bit-for-bit — so a "simplification" that
/// introduced a *second* weight, a normalization or a second `k` still fails
/// immediately.
///
/// **This test was rewritten by the semble-parity wave, not deleted.** Its
/// previous body asserted the unweighted `1/(k+r_lex) + 1/(k+r_sem)`. A2 §7
/// prints `RRF(d) = Σ 1/(k + rank_i(d))` and calls it *"intentionally one
/// expression"*; the α blend is still one expression over the same two
/// independently-RRF'd lists, with the weight semble's `search.py` puts on
/// it ([EXT-SEMBLE], the prior art A2 §7 itself cites). At `ALPHA_NATURAL`
/// the two spellings differ by a constant factor of 2 and therefore rank
/// identically — the blend changes an *order* only when α ≠ 0.5, which is
/// exactly the symbol-query case
/// [`a_symbol_query_leans_on_the_lexical_half`] pins.
#[test]
fn the_fused_score_is_a2_section_7s_one_expression() {
    // lexical: a, b, c   semantic: c, a  (1-based ranks)
    let lex = vec![lexical("a", 3.0), lexical("b", 2.0), lexical("c", 1.0)];
    let sem = vec![semantic("c", 0.9), semantic("a", 0.8)];
    let fused = fuse(&lex, &sem, ALPHA_NATURAL);

    let by_path = |p: &str| {
        fused
            .iter()
            .find(|hit| hit.coordinate.relative_path() == p)
            .unwrap_or_else(|| panic!("{p} missing"))
    };
    let lexical_half = |rank: f64| (1.0 - ALPHA_NATURAL) * (1.0 / (RRF_K + rank));
    let semantic_half = |rank: f64| ALPHA_NATURAL * (1.0 / (RRF_K + rank));
    assert_eq!(by_path("a").rrf, lexical_half(1.0) + semantic_half(2.0));
    assert_eq!(by_path("b").rrf, lexical_half(2.0) + semantic_half(0.0) * 0.0);
    assert_eq!(by_path("c").rrf, lexical_half(3.0) + semantic_half(1.0));
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

/// **Step (b) of the semble port — query-type detection and the α blend.**
///
/// semble resolves the blend from the query's own shape
/// (`ranking/weighting.py::resolve_alpha`: `_ALPHA_SYMBOL = 0.3` "lean BM25
/// for exact keyword matching", `_ALPHA_NL = 0.5` "balanced semantic +
/// BM25") and combines the two independently-RRF'd lists as
/// `alpha*sem + (1-alpha)*bm25` (`search.py`). Before this change both
/// halves were summed unweighted, so prose and identifiers took an
/// identical path — difference 2 of the wave brief.
///
/// **Non-vacuous:** with the α ignored (the previous unweighted sum) a
/// lexical-only rank-1 and a semantic-only rank-1 score *identically*, and
/// the final assertion — that the lexical one strictly outscores the
/// semantic one under a symbol query — fails.
#[test]
fn a_symbol_query_leans_on_the_lexical_half() {
    assert!(is_symbol_query("fused_search"));
    assert!(is_symbol_query("SourceKind"));
    assert!(is_symbol_query("atlas::fusion"));
    assert!(!is_symbol_query("session"));
    assert!(!is_symbol_query("how does the daemon recover"));
    assert_eq!(resolve_alpha("fused_search"), ALPHA_SYMBOL);
    assert_eq!(resolve_alpha("how does the daemon recover"), ALPHA_NATURAL);

    let lex = vec![lexical("lex_only", 3.0)];
    let sem = vec![semantic("sem_only", 0.9)];

    // Natural language: the halves weigh the same, so rank 1 of either list
    // scores the same and only the stated tie-break key separates them.
    let natural = fuse(&lex, &sem, ALPHA_NATURAL);
    assert_eq!(natural[0].rrf, natural[1].rrf);

    // Symbol query: the lexical half carries 0.7 and wins outright.
    let symbol = fuse(&lex, &sem, ALPHA_SYMBOL);
    assert_eq!(paths(&symbol), vec!["lex_only", "sem_only"]);
    assert!(
        symbol[0].rrf > symbol[1].rrf,
        "the blend did not weight the two halves: {} vs {}",
        symbol[0].rrf,
        symbol[1].rrf
    );
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
    let sorted = fuse(&lex, &sem, ALPHA_NATURAL);

    let mut lex_reversed = lex.clone();
    lex_reversed.reverse();
    let mut sem_reversed = sem.clone();
    sem_reversed.reverse();
    let shuffled = fuse(&lex_reversed, &sem_reversed, ALPHA_NATURAL);

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
    let fused = fuse(&lex, &sem, ALPHA_NATURAL);
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

    let fused = fuse(&[alpha, beta], &[beta_s, alpha_s], ALPHA_NATURAL);
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
        ALPHA_NATURAL,
    );
    assert_eq!(fused.len(), 2);
    let expected =
        (1.0 - ALPHA_NATURAL) * rrf_contribution(1) + ALPHA_NATURAL * rrf_contribution(2);
    for hit in &fused {
        assert_eq!(
            hit.rrf, expected,
            "a lexical-rank-1/semantic-rank-2 candidate (or its mirror) must \
             total exactly (1−α)·rrf_contribution(1) + α·rrf_contribution(2): {hit:#?}"
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

/// **A2 §8's nine signals, now as one multiplicative score adjustment.**
///
/// *Renamed from `..._in_the_contracts_own_order` by the semble-parity wave,
/// and rewritten rather than deleted.* The previous body asserted that the
/// nine were compared **lexicographically in A2 §8's listing order** — that a
/// candidate carrying signal 1 outranks one carrying every later signal,
/// whatever the fused scores were. That is the behaviour the wave removed,
/// and the reason is measured, not aesthetic
/// ([`a_boolean_signal_no_longer_outranks_an_arbitrary_score_gap`],
/// [`a_test_path_is_penalised_multiplicatively_not_ranked_after_a_later_signal`]).
///
/// **A2 §8 never stated an order.** Verbatim, at
/// `A2-RETRIEVAL-INTELLIGENCE.md:151`: *"Useful signals include:"* followed by
/// nine lines. The precedence was this module's own inference from the
/// printing order — `fusion.rs` said so itself: *"Because it is the
/// contract's, and the contract supplies no other."* The port replaces that
/// **inference**, not a contract sentence. The contract's own words that do
/// bind here — *"After RRF, reuse A1 structure/provenance rather than
/// training another ranker"* — are still met: all nine are still computed
/// from A1 facts, and nothing learns.
///
/// What this test now pins:
///
/// * all nine are still distinct fields, each landing in its own slot of
///   [`RerankSignals::priority`] — which survives as A2 §13's trace
///   enumeration, no longer as the ordering key;
/// * each non-uniform signal moves [`RerankSignals::multiplier`] by exactly
///   its stated factor;
/// * the three signals A2-01 turned into a boundary contribute exactly
///   `1.0`, because a constant factor on every candidate cannot reorder
///   anything;
/// * **the order is no longer lexicographic**: signal 1 promotes a candidate
///   when its boost covers the score gap and does not when it cannot.
#[test]
fn the_rerank_key_is_a2_section_8s_nine_signals_as_a_score_adjustment() {
    let one = |set: fn(&mut RerankSignals)| {
        let mut signals = RerankSignals::default();
        // Signal 7 fires *positively* for a canonical path, so the neutral
        // baseline has it set; every other signal is off.
        signals.canonical_path = true;
        set(&mut signals);
        signals
    };
    let neutral = one(|_| {});
    assert_eq!(neutral.multiplier(), 1.0, "the neutral baseline is 1.0");

    let cases: [(usize, fn(&mut RerankSignals), f64); 9] = [
        (0, |s| s.exact_match = true, BOOST_EXACT_MATCH),
        (1, |s| s.definition_over_reference = true, BOOST_DEFINITION),
        (2, |s| s.caller_selected_source = true, 1.0),
        (3, |s| s.work_changed_unit = true, BOOST_WORK_CHANGED),
        // Signal 5 is anchor-relative and reflexive; a multiplier on it is a
        // self-boost. Computed and traced, worth nothing to the order — see
        // `the_anchor_does_not_boost_itself_for_being_in_its_own_section`.
        (4, |s| s.same_section_as_anchor = true, 1.0),
        (5, |s| s.structural_relationship = true, BOOST_ADJACENCY),
        // Signal 7 is a separate stage, not a boost — see `path_penalty`.
        (6, |s| s.canonical_path = false, 1.0),
        (7, |s| s.knowledge_source_requested = true, 1.0),
        (8, |s| s.current_generation = true, 1.0),
    ];
    assert_eq!(cases.len(), 9, "A2 §8 lists nine signals");
    // Signal 7 on its own, in the stage it actually belongs to.
    assert_eq!(one(|_| {}).path_penalty(), 1.0);
    assert_eq!(
        one(|s| s.canonical_path = false).path_penalty(),
        PENALTY_NON_CANONICAL
    );

    for (index, set, factor) in cases {
        let signals = one(set);
        let key = signals.priority();
        assert_eq!(key.len(), 9);
        assert_eq!(
            key[index],
            index != 6,
            "signal {index} did not land in slot {index}"
        );
        assert!(
            (signals.multiplier() - factor).abs() < 1e-12,
            "signal {index} is worth {} not {factor}",
            signals.multiplier()
        );
    }

    // The three uniform signals — and signal 5 — are worth exactly nothing,
    // together or apart.
    let all_uniform = RerankSignals {
        canonical_path: true,
        caller_selected_source: true,
        knowledge_source_requested: true,
        current_generation: true,
        same_section_as_anchor: true,
        ..Default::default()
    };
    assert_eq!(all_uniform.multiplier(), 1.0);

    // And the order is a score, not a precedence: signal 1 promotes when its
    // boost covers the gap, and does not when it cannot. Under the previous
    // lexicographic key the second case would still have promoted `u1`.
    let gap_covered = {
        let mut hits = vec![
            scored("b.rs", "u2", 0.010, neutral),
            scored("a.rs", "u1", 0.005, one(|s| s.exact_match = true)),
        ];
        rerank(&mut hits, "");
        hits[0].unit_key.clone()
    };
    assert_eq!(gap_covered, "u1", "0.005 x 3.0 = 0.015 must beat 0.010");

    let gap_too_wide = {
        let mut hits = vec![
            scored("b.rs", "u2", 0.100, neutral),
            scored("a.rs", "u1", 0.005, one(|s| s.exact_match = true)),
        ];
        rerank(&mut hits, "");
        hits[0].unit_key.clone()
    };
    assert_eq!(
        gap_too_wide, "u2",
        "0.005 x 3.0 = 0.015 must NOT beat 0.100 — this is the whole change"
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
        coordinate: None,
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
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Named(source.to_string()),
        kind: None,
        authority: None,
    }
}

fn everything() -> Admissibility {
    Admissibility::within_estate(D1_ESTATE)
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
    scan_and_record(
        &mut db,
        &mut journal,
        &knowledge,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record knowledge");
    if with_decoy {
        let vendor = hand_built_scan(
            "vendor-lib",
            SourceKind::ExternalGit,
            AuthorityClass::External,
            // **The name earns the stem boost too.** Since the semble-parity
            // port, `path_stem_boost` multiplies a candidate by how much of
            // the question is in its file name, so a decoy called `leak.md`
            // could no longer take first place from
            // `payments/decline-handling.md` however well it scored — and
            // this test's non-vacuity guard is precisely that the decoy DOES
            // take first place when nothing excludes it. The decoy is named
            // after the question so the guard keeps discriminating; what it
            // proves is unchanged.
            vec![scanned_file(
                "docs/retry-a-failed-payment-charge.md",
                vec![document_unit(
                    "How do we retry a failed payment charge? When a card charge fails the caller \
                     retries the failed payment after a back-off delay.",
                )],
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
        Some("vendor-lib:docs/retry-a-failed-payment-charge.md"),
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
            estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
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
            estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
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
    record_scan(
        &mut db,
        &mut journal,
        &scan,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record repo-a");
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
    record_scan(
        &mut db,
        &mut journal,
        &base,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record base");
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
    record_scan(
        &mut db,
        &mut journal,
        &overlay,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record overlay");
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
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
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
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
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
        ALPHA_NATURAL,
    );
    hits.sort_by(rrf_order);
    assert_eq!(paths(&hits), vec!["b.rs", "a.rs"], "RRF prefers b.rs");
    assert!(hits[0].rrf > hits[1].rrf, "and strictly so");

    for hit in hits.iter_mut() {
        hit.signals.exact_match = hit.coordinate.relative_path() == "a.rs";
    }
    rerank(&mut hits, "");
    assert_eq!(
        paths(&hits),
        vec!["a.rs", "b.rs"],
        "A2 §8's exact-match signal must be able to reorder A2 §7's output"
    );
}


// ------------------------------- step (d): the rerank adjusts, it does not outrank

/// A bare [`FusedHit`] at a stated score, for the score-arithmetic tests.
fn scored(path: &str, unit: &str, rrf: f64, signals: RerankSignals) -> FusedHit {
    FusedHit {
        rrf,
        adjusted: 0.0,
        origins: RankOrigins::default(),
        signals,
        source_name: "s".to_string(),
        source_kind: SourceKind::EstateGit,
        authority_class: AuthorityClass::EstateMutable,
        generation_id: "g".to_string(),
        content_key: "c".to_string(),
        unit_key: unit.to_string(),
        coordinate: coordinate(path, "sym"),
    }
}

/// **The measured inversion, in a unit test.** `sgt search "bounded judgment
/// ladder"` put `scripts/probe-env.sh#2` (`rrf = 0.006269652`, lexical rank
/// 1404) *above* `.../40-classify/CONTEXT.md#4` (`rrf = 0.022043160`, lexical
/// rank 1) — 28% of the score, first place — because the old
/// [`rerank`] compared `signals.priority()`, a `[bool; 9]`,
/// **lexicographically before it looked at the score at all**. One boolean
/// outranked a 3.5× score gap.
///
/// semble instead *adjusts* the score and sorts by the adjusted value
/// (`semble/ranking/penalties.py::rerank_topk`). A signal can still promote a
/// candidate — [`an_exact_name_match_is_promoted_when_its_boost_covers_the_gap`]
/// shows it doing exactly that — but only by as much as its multiplier is
/// worth.
///
/// **Non-vacuous:** under the previous lexicographic key this asserts the
/// opposite of what the code did, which is the captured red for this commit.
#[test]
fn a_boolean_signal_no_longer_outranks_an_arbitrary_score_gap() {
    let weak_but_flagged = scored(
        "scripts/probe-env.sh",
        "u-probe",
        0.006_269_652,
        RerankSignals {
            exact_match: true,
            ..Default::default()
        },
    );
    let strong_and_plain = scored(
        "docs/40-classify/CONTEXT.md",
        "u-context",
        0.022_043_160,
        RerankSignals::default(),
    );
    // The boost is real and bounded: 3× the weak score still does not reach
    // the strong one, which is the whole claim.
    assert!(
        weak_but_flagged.rrf * BOOST_EXACT_MATCH < strong_and_plain.rrf,
        "the fixture only discriminates if the boost cannot cover the gap"
    );

    let mut hits = vec![weak_but_flagged, strong_and_plain];
    rerank(&mut hits, "");
    assert_eq!(
        hits[0].unit_key, "u-context",
        "a single fired boolean outranked a 3.5x better fused score"
    );
}

/// **The other measured inversion.** `sgt search "knowledge source offline
/// only"` put four `tests/*.rs` helpers named `source` above
/// `src/domain/source.rs`, which was the *best-fused* candidate of the six,
/// because A2 §8's signal 5 (`same_section_as_anchor`) is compared **before**
/// signal 7 (`canonical_path`) in a lexicographic key — so being in the
/// anchor's directory beat being the implementation.
///
/// semble has no such ordering: a test path is a **multiplicative penalty**
/// on the score (`penalties.py::_file_path_penalty`, `_STRONG_PENALTY = 0.3`
/// for test files, test dirs, compat dirs and example dirs), applied to
/// every candidate before the single sort.
///
/// **Non-vacuous:** the flagged candidate carries signal 5 and the plain one
/// does not, so under the old lexicographic key the test path wins — which
/// is what it did on the real estate.
#[test]
fn a_test_path_is_penalised_multiplicatively_not_ranked_after_a_later_signal() {
    let test_helper = scored(
        "tests/x2_knowledge_sources.rs",
        "u-test",
        0.013_333,
        RerankSignals {
            same_section_as_anchor: true,
            canonical_path: false,
            ..Default::default()
        },
    );
    let implementation = scored(
        "src/domain/source.rs",
        "u-impl",
        0.014_493,
        RerankSignals {
            canonical_path: true,
            ..Default::default()
        },
    );
    assert!(
        test_helper.rrf < implementation.rrf,
        "the fixture must keep the implementation the better-fused candidate"
    );

    let mut hits = vec![test_helper, implementation];
    rerank(&mut hits, "");
    assert_eq!(
        hits[0].unit_key, "u-impl",
        "a test path outranked the implementation on a mid-list signal"
    );
    // Its own signal 5 still boosts it by BOOST_ADJACENCY; the penalty then
    // multiplies that. Both factors are visible in the number.
    // Bracketed rather than exact: the file-coherence boost also touches
    // this candidate (it is the only chunk of its file), so the penalised
    // score sits between 0.3x and 0.3 x 1.2x of the fused score.
    let floor = hits[1].rrf * PENALTY_NON_CANONICAL;
    let ceiling = floor * (1.0 + FILE_COHERENCE_BOOST_FRAC);
    assert!(
        hits[1].adjusted >= floor && hits[1].adjusted <= ceiling,
        "the test path's score was not actually penalised: {} from {} (expected {floor}..={ceiling})",
        hits[1].adjusted,
        hits[1].rrf
    );
}

/// **The finding the brief did not name (orientation §3e): one file took nine
/// of ten slots.** `sgt search "bounded judgment ladder"` returned nine chunks
/// of one `CONTEXT.md` at ranks 2–10. We had no per-file control at all.
///
/// semble does: `penalties.py::rerank_topk` decays each additional chunk from
/// an already-selected file by `_FILE_SATURATION_DECAY = 0.5` per excess
/// chunk, greedily, in the ranked order (`_FILE_SATURATION_THRESHOLD = 1`).
///
/// **Non-vacuous:** with no decay, the six `busy.md` chunks all outscore
/// `other.md` and it never reaches the top three.
#[test]
fn a_second_chunk_of_the_same_file_is_decayed_so_one_file_cannot_take_every_slot() {
    let mut hits: Vec<FusedHit> = (0..6)
        .map(|i| {
            scored(
                "docs/busy.md",
                &format!("u-busy-{i}"),
                0.020 - 0.000_1 * f64::from(i),
                RerankSignals {
                    canonical_path: true,
                    ..Default::default()
                },
            )
        })
        .collect();
    hits.push(scored(
        "docs/other.md",
        "u-other",
        0.008,
        RerankSignals {
            canonical_path: true,
            ..Default::default()
        },
    ));
    // Without decay every busy.md chunk beats other.md outright.
    assert!(
        hits.iter()
            .filter(|h| h.coordinate.relative_path() == "docs/busy.md")
            .all(|h| h.rrf > 0.008),
        "the fixture only discriminates if the crowded file wins on raw score"
    );

    rerank(&mut hits, "");
    let order: Vec<&str> = hits.iter().map(|h| h.unit_key.as_str()).collect();
    assert_eq!(order[0], "u-busy-0", "the file's best chunk keeps its score");
    assert!(
        order[..3].contains(&"u-other"),
        "one file still took the whole head of the answer: {order:?}"
    );
    // The decay is 0.5 per excess chunk, exactly semble's constant.
    let second_busy = hits
        .iter()
        .find(|h| h.unit_key == "u-busy-1")
        .expect("second chunk");
    assert!(
        (second_busy.adjusted - second_busy.rrf * FILE_SATURATION_DECAY).abs() < 1e-12,
        "second chunk of a file was not decayed by {FILE_SATURATION_DECAY}: {} from {}",
        second_busy.adjusted,
        second_busy.rrf
    );
}

/// **The anchor must not boost itself, and A2 §8's signal 5 cannot stop it
/// from doing so.**
///
/// Signal 5 is *"same module/package/document section"* measured against the
/// top-RRF candidate — the *anchor* — and `same_section` is reflexive. Under
/// the old lexicographic key that was harmless: slot 5 sat below
/// `exact_match` in slot 1, so the anchor's self-satisfaction never beat a
/// real signal. As a **multiplier** it is a self-fulfilling boost: whatever
/// RRF put first is multiplied for being where it already is.
///
/// Measured: `sgt search "BM25_K1"` put a markdown heading
/// (`rrf = 0.013818547`) above `rust const BM25_K1` (`rrf = 0.005154200`),
/// which carries the ×3 exact-match boost — because `0.013819 × 1.2` beats
/// `0.005154 × 3.0`, and the heading's only 1.2 was for being the anchor.
///
/// **Suppressing it on the anchor alone is worse, and was tried and
/// rejected**: every *other* candidate in the anchor's section then keeps a
/// ×1.2 the anchor does not, which inverted
/// `w5_search_surface::a_relational_aggregate_and_a_retrieved_row_join_on_
/// one_shared_row_identity` — two rows of one CSV, the queried row demoted
/// below its neighbour.
///
/// So signal 5 carries **no multiplier at all**. It stays computed and stays
/// in A2 §13's trace; the file-level preference A2 §8 asks for is supplied
/// instead by semble's own anchor-free mechanism, `boosting.py::
/// boost_multi_chunk_files` — see
/// [`a_file_whose_chunks_collectively_score_well_has_its_best_chunk_boosted`].
///
/// **Non-vacuous:** with signal 5 worth `BOOST_ADJACENCY` this asserts the
/// opposite of what the code does.
#[test]
fn the_anchor_does_not_boost_itself_for_being_in_its_own_section() {
    let anchor_heading = scored(
        "evidence/reference-corpus/synthesis.md",
        "u-heading",
        0.013_818_547,
        RerankSignals {
            canonical_path: true,
            same_section_as_anchor: true,
            ..Default::default()
        },
    );
    let the_definition = scored(
        "src/runtime/atlas/lexical.rs",
        "u-const",
        0.005_154_200,
        RerankSignals {
            canonical_path: true,
            exact_match: true,
            ..Default::default()
        },
    );
    assert!(
        anchor_heading.rrf > the_definition.rrf,
        "the fixture must keep the heading the better-fused candidate"
    );

    let mut hits = vec![anchor_heading, the_definition];
    rerank(&mut hits, "");
    assert_eq!(
        hits[0].unit_key, "u-const",
        "the anchor kept first place on a boost it gave itself"
    );

    // Said directly: signal 5 changes no score.
    let base = RerankSignals {
        canonical_path: true,
        ..Default::default()
    };
    let in_section = RerankSignals {
        same_section_as_anchor: true,
        ..base
    };
    assert_eq!(base.multiplier(), in_section.multiplier());
}

/// **semble's own file-level preference, which needs no anchor.**
/// `boosting.py::boost_multi_chunk_files`: a file whose candidate chunks
/// score well *collectively* has its single best chunk boosted by
/// `_FILE_COHERENCE_BOOST_FRAC = 0.2` of the maximum, scaled by that file's
/// share of the largest file total.
///
/// This is what replaces signal 5's multiplier: it expresses the same "this
/// file is what the query is about" preference, it is computed identically
/// for every file, and no candidate can earn it for being where RRF already
/// put it.
///
/// **Non-vacuous:** without the boost `y.md` out-scores every `x.md` chunk
/// and comes first.
#[test]
fn a_file_whose_chunks_collectively_score_well_has_its_best_chunk_boosted() {
    let canonical = RerankSignals {
        canonical_path: true,
        ..Default::default()
    };
    let mut hits = vec![
        scored("docs/x.md", "u-x1", 0.011_0, canonical),
        scored("docs/x.md", "u-x2", 0.011_0, canonical),
        scored("docs/y.md", "u-y1", 0.011_5, canonical),
    ];
    assert!(
        hits[2].rrf > hits[0].rrf,
        "the fixture only discriminates if the lone chunk wins on raw score"
    );

    rerank(&mut hits, "");
    assert_eq!(
        hits[0].unit_key, "u-x1",
        "the coherent file's best chunk was not boosted: {:?}",
        hits.iter().map(|h| (&h.unit_key, h.adjusted)).collect::<Vec<_>>()
    );
    // x.md's total is the largest, so its share is 1.0 and its best chunk
    // takes the full 0.2 — semble's constant, exactly.
    assert!(
        (hits[0].adjusted - 0.011_0 * (1.0 + FILE_COHERENCE_BOOST_FRAC)).abs() < 1e-12,
        "boost was not 1 + {FILE_COHERENCE_BOOST_FRAC}: {}",
        hits[0].adjusted
    );
}

/// **semble's NL stem boost — `boosting.py::_boost_stem_matches`.**
///
/// For a natural-language query semble boosts candidates whose **file stem or
/// immediate parent directory** matches the query's keywords, by the fraction
/// of keywords matched: *"Uses prefix matching for morphological variants
/// (e.g. `dependency` matches `dependencies`). Matches file stems and the
/// immediate parent directory name."* Keywords are words longer than two
/// characters that are not in its shipped stopword list; a match is exact or
/// a ≥3-character prefix overlap in either direction; the boost applies only
/// once the match ratio reaches `0.10`.
///
/// This is the single largest measured gap between the two systems. On the
/// 52-question set the prose categories scored `doctrine 1/10, knowledge
/// 0/6, memory 0/4` against semble's `4/10, 5/6, 2/4` on the same questions:
/// a document *named after the question* was not preferred at all, because
/// A2 §8's signal 1 requires a query **term** to equal the file name
/// exactly and `one-atlas-database-2026-08-29.md` never will.
///
/// Symbol queries take semble's other branch and are deliberately not
/// boosted here — see [`the_stem_boost_does_not_fire_for_a_symbol_query`].
///
/// **Non-vacuous:** the second assertion is a real file in the corpus with a
/// real query from the committed question set, and the third shows an
/// unrelated path getting nothing.
#[test]
fn a_file_named_after_the_question_is_boosted_by_the_fraction_of_words_it_matches() {
    let query = "ruling that there is only one atlas database";
    let named =
        path_stem_match_ratio(query, "rulings/owner-rulings/one-atlas-database-2026-08-29.md");
    let unrelated = path_stem_match_ratio(query, "src/backend/codex.rs");
    assert_eq!(unrelated, 0.0, "an unrelated path must not be boosted");
    assert!(
        named > 0.4,
        "a file named after the question must match well above a third of its \
         keywords: got {named}"
    );
    assert_eq!(STEM_BOOST_MULTIPLIER, 1.0);

    // And it decides an order the fused scores would have lost.
    let canonical = RerankSignals {
        canonical_path: true,
        ..Default::default()
    };
    // The document sits at 60% of the leader's score. A multiplicative
    // `x(1 + ratio)` gives it 0.0072 x 1.667 = 0.0120 and it still loses to
    // the leader's 0.0144; semble's additive `+= max_score x ratio` gives it
    // 0.0177 and it wins. That difference is the point — measured, it was
    // worth +5 p@5 and nothing at all on p@1 in the weaker form.
    let mut hits = vec![
        scored("src/backend/codex.rs", "u-noise", 0.012, canonical),
        scored(
            "rulings/owner-rulings/one-atlas-database-2026-08-29.md",
            "u-ruling",
            0.007_2,
            canonical,
        ),
    ];
    rerank(&mut hits, query);
    assert_eq!(
        hits[0].unit_key, "u-ruling",
        "the file named after the question lost to a better-fused unrelated one"
    );
}

/// A **symbol** query takes semble's other branch: `apply_query_boost` calls
/// `_boost_symbol_definitions`, never `_boost_stem_matches`
/// (`boosting.py::apply_query_boost`'s `if is_symbol_query(query)`). We have
/// a stronger equivalent of the definition boost already — A2 §8's signal 1
/// is a tree-sitter symbol identity, not a regex over chunk text — so only
/// the NL branch is ported, and the symbol branch must stay out of its way.
#[test]
fn the_stem_boost_does_not_fire_for_a_symbol_query() {
    assert!(is_symbol_query("SourceKind"));
    assert_eq!(
        path_stem_match_ratio("SourceKind", "src/domain/source.rs"),
        0.0,
        "a symbol query must not also collect the NL stem boost"
    );
}

/// Stopwords and the two-character floor are semble's, and they matter: a
/// query made only of them must boost nothing, or every path in the corpus
/// gets the same lift and the signal is noise.
#[test]
fn stopwords_and_short_words_are_not_keywords() {
    assert_eq!(path_stem_match_ratio("how do we do it", "src/domain/work.rs"), 0.0);
    assert_eq!(path_stem_match_ratio("of on or the to", "src/domain/work.rs"), 0.0);
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
            estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
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

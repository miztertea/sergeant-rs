//! S6 P0 — the **A2 acceptance register**: contract §17 walked literally.
//!
//! `A2-RETRIEVAL-INTELLIGENCE.md` §17 lists ten acceptance items. This file
//! is the walk: one register row per item, each carrying a verdict and the
//! **named decisive check** that produces it — an earlier wave's test where
//! one already pins the exact claim, or an explicit deferral with the ruling
//! that moved it.
//!
//! Before this file existed, all ten items were cited *somewhere*
//! (`w2_lexical_retrieval` item 2, `w3_semantic_degradation` item 4,
//! `w5_search_surface` items 3/6/8) and five were cited nowhere at all, so
//! nothing in the tree could answer "which A2 items are unclaimed?". That is
//! not a hypothetical failure mode: A2 §17 item 1 read as satisfied — the
//! filter *was* deterministic — while one of A2 §2's own filter axes, the
//! estate axis, did not exist. No register asked which axes there were. The
//! axis has since landed (`tests/d1_estate_isolation.rs`), and item 1 below
//! is claimed by **running** that suite, never by knowing this paragraph.
//!
//! This file is a copy of `tests/x5_a1a_acceptance.rs`'s shape (Ponytail R2),
//! not a new one: the same `Verdict`/`Check`/`Item`/`WALK` register, the same
//! `at(file, test)` citations, and the same three self-checks. Two things
//! that register could not express are added here and there in the same wave:
//! a citation to an `#[ignore]`d or assertion-free check is refused
//! ([`every_named_check_exists_in_the_suite_it_names`]), and an item nobody
//! has claimed a check for has a verdict of its own
//! ([`Verdict::Unclaimed`]) instead of being expressible only by deleting its
//! row.
//!
//! A1a's `met-with-deviation` verdict is deliberately **not** carried over
//! (R1): no row here is "shipped, but not in §17's shape", and vocabulary
//! with no row and no need is exactly what R1 says to skip. A wave that needs
//! it adds it with the row that needs it.
//!
//! Running this file does not re-prove the claims it cites — it proves that
//! every one of the ten items has a named, existing, running, asserting
//! answer. The claims themselves are proven by the suites named below, and
//! every `met` verdict here was written after running its own checks and
//! reading the output (S6 P0, at `afed0aa9`).
//!
//! ## The walk
//!
//! | # | Contract claim (§17) | Verdict | Decisive check |
//! |---|---|---|---|
//! | 1 | query world is filtered deterministically before ranking | met | `w1_deterministic_filter::a_superseded_generation_does_not_leak_through_any_admissible_method` |
//! | 2 | lexical search returns code/document/mail/selected-row-text units with exact A1 provenance | met | `w2_lexical_retrieval::lexical_search_returns_a_code_unit_with_exact_a1_provenance` |
//! | 3 | relational dataset queries remain available independently of text retrieval and can join to retrieved row evidence | met | `w5_search_surface::a_relational_aggregate_and_a_retrieved_row_join_on_one_shared_row_identity` |
//! | 4 | semantic retrieval, when installed, uses pinned local assets and can be disabled/degraded cleanly | met | `w3b_semantic_retrieval::semantically_related_units_rank_above_unrelated_ones_on_the_hand_verified_corpus` |
//! | 5 | lexical + semantic lists fuse through RRF and deterministic rerank | met | `w4_rrf_fusion::the_fused_score_is_a2_section_7s_one_expression` |
//! | 6 | local knowledge Source searches can span normalized Office/docs without losing original path/source coordinates | met | `w5_search_surface::one_local_knowledge_query_spans_a_normalized_docx_and_a_markdown_file` |
//! | 7 | OCR text can be retrieved with page/bbox/engine provenance | deferred-out-of-0.3.0 | — |
//! | 8 | external evidence remains visibly external | met | `w5_search_surface::an_external_hit_is_identifiable_as_external_from_the_answer_alone` |
//! | 9 | search traces pin source/retrieval/model/policy generations | met | `w5_search_surface::the_trace_records_every_one_of_a2_section_13s_nine_fields` |
//! | 10 | no arbitrary client DuckDB SQL is introduced | met | `w5_search_surface::the_search_cli_exposes_a2_section_14s_selectors_and_no_weight_knob` |
//!
//! The `Decisive check` column names **one** check per row so the table stays
//! readable; every row's full check list is in [`WALK`] below, and each one is
//! verified to exist, to run, and to assert something.
//!
//! ## Item 7 is the one deferral, and it is a ruling, not a gap
//!
//! OCR does not ship in 0.3.0 by owner ruling — the addendum of 2026-08-29 to
//! `host-atlas-r3-ratification-2026-08-25.md`: *"OCR (register item 9) does
//! not ship in 0.3.0. It moves to a later epic scoped around the further
//! adapters estate intelligence turns out to need"*, and *"OCR is the ONLY
//! item exempt from convergence"*. Scanned/image evidence continues to report
//! as `unsupported` coverage, which that ruling names as gap reporting rather
//! than silent loss. A1a §17's own item 9 carries the same deferral
//! (`x5_a1a_acceptance`, verdict `deferred-post-s4`), and C1 §21 item 8's OCR
//! half is declared absent rather than omitted
//! (`c1c_authority_and_provenance::the_ocr_half_of_item_8_is_declared_absent_rather_than_omitted`).
//! Three registers, one ruling, one destination.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

mod support;
use support::cited_function;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// This file, for the doc-table guard to read back.
const THIS_FILE: &str = "tests/a2_acceptance.rs";

/// The contract section this register walks, and how many items it has.
///
/// The contract itself is **not in this repository** — it is
/// `knowledge/evidence/resources/host-atlas-series/A2-RETRIEVAL-INTELLIGENCE.md`
/// in the sergeant-rs-workspace estate, which is where the development record
/// lives by design (that estate's own CLAUDE.md: the knowledge library "used
/// to live inside sergeant-rs itself and ... a product repository has no
/// business carrying" it). So no test here can read the contract and diff the
/// table against it; the claim column below was transcribed by hand from
/// §17 lines 268–279 at fixed point `afed0aa9`, and the count is pinned so a
/// row cannot be added or dropped without the guard noticing even though the
/// wording cannot be machine-checked. A1a's register has the same limit for
/// the same reason.
const SECTION: &str = "A2 §17";
const ITEM_COUNT: u8 = 10;

/// The one item the owner ruled out of 0.3.0, pinned as a set so a second
/// deferral cannot be smuggled in under the ruling that authorized this one.
const DEFERRED_ITEMS: [u8; 1] = [7];

// ------------------------------------------------------------------ register

/// What the walk concluded about one §17 item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// Shipped and proven by the named checks, each of which was run at
    /// `afed0aa9` and passed.
    Met,
    /// Not fully provable today. The `note` names what is missing and the
    /// DESTINATION that owns it. Never a pass.
    ///
    /// **No row carries this today.** It is kept rather than deleted because
    /// it is this register's VOCABULARY: the moment a §17 item stops being
    /// provable, the honest record of that is a row here, and a register that
    /// cannot spell "gap" invites the row being quietly re-worded instead.
    #[allow(dead_code)]
    Gap,
    /// Nobody has claimed a decisive check for this item at all — distinct
    /// from [`Self::Gap`], which claims a check and reports what it cannot
    /// reach.
    ///
    /// This verdict exists because the failure mode this whole file is about
    /// is an item with **no row**, and a register whose only way to say
    /// "unclaimed" is to omit the row cannot report the thing it exists to
    /// report. An `Unclaimed` row claims no check and must name a
    /// DESTINATION, exactly as a gap must.
    ///
    /// **No row carries this today** — vocabulary, not a census.
    #[allow(dead_code)]
    Unclaimed,
    /// Out of 0.3.0 by the owner ruling cited in the `note`.
    DeferredOutOf030,
}

impl Verdict {
    fn as_str(self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::Gap => "gap",
            Self::Unclaimed => "unclaimed",
            Self::DeferredOutOf030 => "deferred-out-of-0.3.0",
        }
    }
}

/// One named check: the suite file it lives in, and the test function's name.
///
/// A `(file, test)` pair rather than a free-text citation precisely so
/// [`every_named_check_exists_in_the_suite_it_names`] can go and look.
struct Check {
    file: &'static str,
    test: &'static str,
}

const fn at(file: &'static str, test: &'static str) -> Check {
    Check { file, test }
}

/// One row of the §17 walk.
struct Item {
    number: u8,
    verdict: Verdict,
    checks: &'static [Check],
    /// Why this verdict, in the words a reviewer needs — what was run, the
    /// gap left, or the ruling that deferred the item.
    note: &'static str,
}

/// This file's whole claim, in one place.
const WALK: &[Item] = &[
    Item {
        number: 1,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/w1_deterministic_filter.rs",
                "a_superseded_generation_does_not_leak_through_any_admissible_method",
            ),
            at(
                "tests/w1_deterministic_filter.rs",
                "an_authority_filter_excludes_external_content_when_not_requested",
            ),
            at(
                "tests/w1_deterministic_filter.rs",
                "a_content_kind_mismatch_is_excluded_from_the_code_family",
            ),
            at(
                "tests/d1_estate_isolation.rs",
                "a_search_from_one_estate_sees_its_own_units_and_none_of_the_other_estates",
            ),
            at(
                "tests/d1_estate_isolation.rs",
                "a_search_that_addresses_no_estate_is_refused_rather_than_answered_widely",
            ),
            at(
                "tests/w2_lexical_retrieval.rs",
                "an_inadmissible_unit_with_a_perfect_lexical_match_is_never_returned",
            ),
            at(
                "tests/w4_rrf_fusion.rs",
                "an_inadmissible_unit_that_wins_the_semantic_list_never_reaches_the_fused_answer",
            ),
        ],
        note: "Two halves, and this item was once claimed on only one of them. The FILTER half \
               is W1's negative admissions — source/generation, authority and content-kind rows \
               proven ABSENT rather than merely correct rows proven present. The BEFORE-RANKING \
               half is the pair of decoys: a unit that is a better lexical match than anything \
               admissible, and a unit that wins the semantic list outright, neither of which \
               reaches the answer. The estate axis of A2 §2 stage 1 is the axis that was missing \
               while this item read as satisfied, so it is cited explicitly and separately: D1 \
               proves one estate's search sees none of another's on one host daemon, and that a \
               search addressing no estate is refused rather than answered widely. All seven ran \
               green at afed0aa9 (31/31 and 4/4 in the two S6 P0 runs).",
    },
    Item {
        number: 2,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/w2_lexical_retrieval.rs",
                "lexical_search_returns_a_code_unit_with_exact_a1_provenance",
            ),
            at(
                "tests/w2_lexical_retrieval.rs",
                "lexical_search_returns_a_document_unit_with_exact_a1_provenance",
            ),
            at(
                "tests/w2_lexical_retrieval.rs",
                "lexical_search_returns_mail_units_with_exact_a1_provenance",
            ),
            at(
                "tests/w2_lexical_retrieval.rs",
                "lexical_search_returns_a_selected_row_text_unit_with_exact_a1_provenance",
            ),
        ],
        note: "§17 item 2 names four families and this row cites one check per family, because \
               'a family present in the machinery and absent from the tests' is a defect this \
               program has recorded three times. Each check resolves the hit back to its A1 \
               coordinate; the code/document/selected-row-text fixtures come from a real \
               scan_local_knowledge walk over real files, so the byte ranges asserted are real \
               offsets. All four ran green at afed0aa9.",
    },
    Item {
        number: 3,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/w5_search_surface.rs",
                "a_relational_aggregate_and_a_retrieved_row_join_on_one_shared_row_identity",
            ),
            at(
                "tests/w5_search_surface.rs",
                "item_3s_relational_read_is_reachable_from_outside_the_process",
            ),
            at(
                "tests/x4_tabular_map.rs",
                "datasets_are_registered_and_read_in_place_as_derived_evidence",
            ),
        ],
        note: "The item has two clauses and the second is the one that is easy to fake: \
               'independently of text retrieval' is X4's in-place dataset read, and 'can join to \
               retrieved row evidence' is W5's single shared row identity across an aggregate \
               and a retrieved row. The reachability check is why this row is not claimed on the \
               in-process read alone — a relational read only a test binary can perform is not \
               a surface. All three ran green at afed0aa9.",
    },
    Item {
        number: 4,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/w3b_semantic_retrieval.rs",
                "semantically_related_units_rank_above_unrelated_ones_on_the_hand_verified_corpus",
            ),
            at(
                "tests/w3b_semantic_retrieval.rs",
                "a_suppressed_request_reports_disabled_even_where_the_model_is_installed",
            ),
            at(
                "tests/w3b_semantic_retrieval.rs",
                "a_host_without_assets_still_answers_lexically_and_reports_not_installed",
            ),
            at(
                "tests/w3_semantic_degradation.rs",
                "the_search_answers_semantic_field_is_required_not_optional",
            ),
            at(
                "tests/w3_semantic_degradation.rs",
                "the_semantic_module_names_no_obvious_fetcher",
            ),
        ],
        note: "'When installed' is the committed assets under assets/semantic-model, and the \
               suppression check reads the pinned identity back out — repo@revision plus a \
               blake3 content hash — so 'pinned local assets' is proven by the version AND the \
               bytes, not by the directory existing. 'Disabled/degraded cleanly' is two \
               different states that a reader must be able to tell apart mechanically: \
               suppressed-with-a-model-present reports `disabled`, no-assets reports \
               `not_installed`, and the status field is required rather than optional so an \
               absent field can never be read as a complete answer. The no-fetcher check is the \
               other half of 'local': the module may not go and get a model. All five ran green \
               at afed0aa9.",
    },
    Item {
        number: 5,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/w4_rrf_fusion.rs",
                "the_fused_score_is_a2_section_7s_one_expression",
            ),
            at(
                "tests/w4_rrf_fusion.rs",
                "the_rerank_key_is_a2_section_8s_nine_signals_in_the_contracts_own_order",
            ),
            at(
                "tests/w4_rrf_fusion.rs",
                "every_one_of_a2_section_8s_nine_signals_actually_fires",
            ),
            at(
                "tests/w4_rrf_fusion.rs",
                "the_order_the_two_lists_arrive_in_cannot_change_the_fused_answer",
            ),
            at(
                "tests/w4_rrf_fusion.rs",
                "the_same_query_over_the_same_generations_returns_an_identical_fused_answer",
            ),
        ],
        note: "'RRF' is checked as §7's literal expression computed by hand and compared \
               bit-for-bit, not as a function called RRF. 'Deterministic rerank' is checked \
               twice over, because a rerank key can be in the right order and still not be \
               reached: the key is §8's nine signals in §8's own order, AND every one of the \
               nine is observed actually firing. The two determinism checks are the ones that \
               would catch a hash-order or arrival-order dependence. All five ran green at \
               afed0aa9.",
    },
    Item {
        number: 6,
        verdict: Verdict::Met,
        checks: &[at(
            "tests/w5_search_surface.rs",
            "one_local_knowledge_query_spans_a_normalized_docx_and_a_markdown_file",
        )],
        note: "One query, two normalized formats, and the assertion that matters is the second \
               clause of the item rather than the first: each hit still carries its own original \
               path and source coordinate after normalization. A single check carries this row \
               because a single check is what pins the exact claim; this register cites, it does \
               not re-prove (A1a's own precedent). Ran green at afed0aa9.",
    },
    Item {
        number: 7,
        verdict: Verdict::DeferredOutOf030,
        checks: &[],
        note: "OCR does not ship in 0.3.0 by owner ruling — the 2026-08-29 addendum to \
               host-atlas-r3-ratification-2026-08-25.md: 'OCR (register item 9) does not ship in \
               0.3.0. It moves to a later epic scoped around the further adapters estate \
               intelligence turns out to need', and 'OCR is the ONLY item exempt from \
               convergence'. DESTINATION: that later adapter epic. This build derives no OCR \
               evidence at all, so there is no page/bbox/engine provenance whose retrieval could \
               be tested; scanned/image evidence reports as `unsupported` coverage, which the \
               ruling names as gap reporting rather than silent loss. The same deferral is \
               carried by A1a §17 item 9 and by C1 §21 item 8's OCR half.",
    },
    Item {
        number: 8,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/w5_search_surface.rs",
                "an_external_hit_is_identifiable_as_external_from_the_answer_alone",
            ),
            at(
                "tests/w5_search_surface.rs",
                "a_fused_answer_carries_the_source_kind_and_authority_class_of_every_hit",
            ),
        ],
        note: "'Visibly external' is read as a property of the ANSWER, not of the estate \
               configuration a reader would have to go and consult: the hit itself says which \
               source kind and authority class it came from. The second check is why this row is \
               not claimed on the lexical answer alone — fusion is where a class can be lost \
               while every input still carried it. Both ran green at afed0aa9. The instruction- \
               hierarchy half of 'external' — that external text cannot claim authority — is C1 \
               §21 item 9's, cited there rather than duplicated here.",
    },
    Item {
        number: 9,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/w5_search_surface.rs",
                "the_trace_records_every_one_of_a2_section_13s_nine_fields",
            ),
            at(
                "tests/w5_search_surface.rs",
                "the_lexical_tokenizer_version_is_pinned_to_the_tokenizers_actual_output",
            ),
            at(
                "tests/w5_search_surface.rs",
                "the_retrieval_policy_version_is_pinned_to_the_actual_rrf_and_rerank_policy",
            ),
            at(
                "tests/w5_search_surface.rs",
                "the_trace_states_the_semantic_status_even_when_the_caller_suppressed_it",
            ),
        ],
        note: "The item names four things a trace must pin and §13 enumerates nine fields; the \
               first check walks all nine. The two version checks are the reason this row is not \
               claimed on field presence alone — a version string is a promise to remember \
               unless something derives it from the thing it names, and both are derived from \
               the tokenizer's and the policy's actual behaviour. The fourth check covers the \
               case where the trace would be most tempting to leave blank. All four ran green at \
               afed0aa9.",
    },
    Item {
        number: 10,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/w5_search_surface.rs",
                "the_search_cli_exposes_a2_section_14s_selectors_and_no_weight_knob",
            ),
            at(
                "tests/w5_search_surface.rs",
                "the_search_and_related_routes_reach_atlas_through_the_read_only_handle",
            ),
            at(
                "tests/x5_a1a_acceptance.rs",
                "a1a_item_13_no_client_sql_reaches_the_store",
            ),
        ],
        note: "A negative claim, so every check here is a negative one and none of them is a \
               list of forbidden names. The selector check pins A2 §14's flag set EXHAUSTIVELY \
               — a new flag of any spelling fails it, which is what makes `--sql` unaddable \
               rather than merely absent. The store-side check is A1a's own item 13, cited here \
               rather than duplicated (this register cites; it re-proves only where nothing \
               pins the exact claim): `Sql` cannot be built from a caller's string at all, \
               because its text is an associated const, and no public Atlas signature takes a \
               string-typed statement. The third check is the reason the row is not claimed on \
               those two alone — A2 added a route, and the route reaches Atlas through the \
               read-only handle whose one constructor also takes no string. All three ran green \
               at afed0aa9.",
    },
];

// ------------------------------------------------- the register's own guards

#[test]
fn every_contract_item_is_accounted_for() {
    let numbers: Vec<u8> = WALK.iter().map(|item| item.number).collect();
    assert_eq!(
        numbers,
        (1..=ITEM_COUNT).collect::<Vec<u8>>(),
        "{SECTION} has {ITEM_COUNT} items; the walk must carry each exactly once, in order — an \
         item nobody wrote a row for is the failure mode this register exists to make visible"
    );

    for item in WALK {
        assert!(
            !item.note.is_empty(),
            "item {} has no note: a verdict without a reason is not a walk",
            item.number
        );
        match item.verdict {
            Verdict::DeferredOutOf030 | Verdict::Unclaimed => assert!(
                item.checks.is_empty(),
                "item {} is deferred or unclaimed; it must not claim a check",
                item.number
            ),
            _ => assert!(
                !item.checks.is_empty(),
                "item {} claims a verdict with no decisive check — the one thing this \
                 register exists to prevent",
                item.number
            ),
        }
    }

    // A deferral has to say where it went, and a gap or an unclaimed item has
    // to say where it is going, or either is a silent pass wearing a
    // different label.
    for item in WALK
        .iter()
        .filter(|i| i.verdict == Verdict::DeferredOutOf030)
    {
        assert!(
            item.note.contains("0.3.0") && item.note.contains("DESTINATION"),
            "item {}'s deferral must cite the ruling that moved it and name where it went",
            item.number
        );
    }
    for item in WALK
        .iter()
        .filter(|i| i.verdict == Verdict::Gap || i.verdict == Verdict::Unclaimed)
    {
        assert!(
            item.note.contains("DESTINATION"),
            "item {}'s gap must name a destination",
            item.number
        );
    }

    // The deferral boundary itself, pinned so a later edit cannot quietly
    // defer a second item under the one ruling that authorized the first.
    let deferred: BTreeSet<u8> = WALK
        .iter()
        .filter(|item| item.verdict == Verdict::DeferredOutOf030)
        .map(|item| item.number)
        .collect();
    assert_eq!(
        deferred,
        BTreeSet::from(DEFERRED_ITEMS),
        "the owner ruling of 2026-08-29 makes OCR the ONLY item exempt from convergence before \
         0.3.0; every other row closes against the contract's own text"
    );
}

/// Every citation resolves — and resolves to a check that actually runs and
/// actually asserts something.
///
/// The first of those three is all A1a's equivalent guard did before S6 P0:
/// it matched the string `fn <name>(` and nothing else, so a cited test
/// carrying `#[ignore]`, or one whose body had been emptied, satisfied it in
/// full while the register kept reporting `met`. Both cases were reproduced
/// against `x5_a1a_acceptance.rs` at `afed0aa9` and both passed. This
/// register is born with all three conditions, and the same three now guard
/// A1a's.
#[test]
fn every_named_check_exists_in_the_suite_it_names() {
    for item in WALK {
        for check in item.checks {
            let text = read(check.file);
            assert!(
                text.contains(&format!("fn {}(", check.test)),
                "item {}: {} names no `{}` — a citation that does not resolve proves nothing",
                item.number,
                check.file,
                check.test
            );

            let (attributes, body) = cited_function(&text, check.test)
                .unwrap_or_else(|| panic!("item {}: cannot slice {}", item.number, check.test));

            assert!(
                !attributes.iter().any(|line| line.contains("ignore")),
                "item {}: {}::{} is `#[ignore]`d — a citation to a check that does not run \
                 is a verdict with nothing behind it",
                item.number,
                check.file,
                check.test
            );
            // A bare substring test for "assert" matches inside a comment or a
            // string literal too — a cited test whose body only reads
            // `// no assert needed, this is documentation-only` (no macro
            // call) would satisfy it while asserting nothing (F-IN-02).
            // Comment-only lines are dropped first, and what remains must
            // contain an actual assert-family macro invocation, not just
            // the word.
            let asserts = body
                .lines()
                .map(str::trim)
                .filter(|line| !line.starts_with("//"))
                .any(|line| {
                    [
                        "assert!(",
                        "assert_eq!(",
                        "assert_ne!(",
                        "assert_matches!(",
                        "debug_assert!(",
                        "debug_assert_eq!(",
                        "debug_assert_ne!(",
                    ]
                    .iter()
                    .any(|needle| line.contains(needle))
                });
            assert!(
                asserts,
                "item {}: {}::{} contains no assertion — a citation to a check that proves \
                 nothing is a verdict with nothing behind it",
                item.number, check.file, check.test
            );
        }
    }
}

/// The doc table at the top of this file and [`WALK`] must agree.
///
/// The table is what a reviewer reads in the pull request; the register is
/// what runs. Two copies of one claim drift, so one is checked against the
/// other rather than trusted.
#[test]
fn the_documented_walk_table_matches_the_register() {
    let text = read(THIS_FILE);
    let rows: Vec<&str> = text
        .lines()
        // The separator row (`//! |---|`) has no space after the bar, so this
        // filter already excludes it; only the header needs skipping.
        .filter(|line| line.starts_with("//! | "))
        .skip(1)
        .collect();
    assert_eq!(
        rows.len(),
        WALK.len(),
        "the doc table must carry every item"
    );

    for (row, item) in rows.iter().zip(WALK) {
        let cells: Vec<&str> = row
            .trim_start_matches("//! |")
            .split('|')
            .map(str::trim)
            .collect();
        assert_eq!(
            cells[0],
            item.number.to_string(),
            "doc table row order must match the register"
        );
        assert_eq!(
            cells[2],
            item.verdict.as_str(),
            "doc table verdict for item {} disagrees with the register",
            item.number
        );
        match item.verdict {
            Verdict::DeferredOutOf030 | Verdict::Unclaimed => assert_eq!(
                cells[3], "—",
                "a deferred or unclaimed item names no check in the table either"
            ),
            _ => {
                let first = &item.checks[0];
                let suite = first
                    .file
                    .trim_start_matches("tests/")
                    .trim_end_matches(".rs");
                assert_eq!(
                    cells[3],
                    format!("`{suite}::{}`", first.test),
                    "doc table check for item {} must be the register's first check",
                    item.number
                );
            }
        }
    }
}

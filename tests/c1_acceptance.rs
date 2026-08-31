//! S6 P0 — the **C1 acceptance register**: contract §21 walked literally.
//!
//! `C1-COMPILED-CONTEXT.md` §21 lists fourteen acceptance items. This file is
//! the walk: one register row per item, each carrying a verdict and the
//! **named decisive check** that produces it.
//!
//! All fourteen were already cited *somewhere* before this file existed —
//! `c1a_compiled_context` items 1/2/10/13, `c1b_tiers_and_budget` items 3/4/5,
//! `c1c_authority_and_provenance` items 6/7/8/9, `c1d_attribution_nesting_audit`
//! items 11/12/14 — across four files, each of which knew its own share and
//! none of which could see the whole. Nothing enumerated the fourteen, so
//! nothing could report a gap. **Building this register found one**: §21 item
//! 8 names document, mail and OCR excerpts, and no test in this repository
//! proved that a *mail* excerpt in a compiled prompt carried its
//! resource/native/extractor provenance. That row was carried as a gap with a
//! destination rather than a `met` transcribed from `c1c`'s own header —
//! which is the whole reason this file exists.
//!
//! **That gap is now closed, by running the thing rather than re-reading the
//! note.** A follow-up wave compiled a context binding a real `.eml` landed
//! through the real worker subprocess and read the rendered prompt. The mail
//! third was a missing *proof*, not a missing behaviour — all three of
//! resource, native coordinate and extractor already reached the prompt. But
//! compiling it also surfaced a real defect the reading would never have
//! found: the C1 render printed `bytes 0..0` beside every native coordinate
//! (S5 closeout F-AC-02, on a second render path), now fixed and guarded.
//! Item 8's OCR third remains a ruling-backed deferral, not a gap, and the
//! `met` verdict is argued from the two thirds 0.3.0 ships, never from it.
//!
//! This file is a copy of `tests/x5_a1a_acceptance.rs`'s shape (Ponytail R2),
//! not a new one — the same `Verdict`/`Check`/`Item`/`WALK` register, the same
//! `at(file, test)` citations, and the same three self-checks — with the two
//! things that register could not express, added here and there in the same
//! wave: a citation to an `#[ignore]`d or assertion-free check is refused
//! ([`every_named_check_exists_in_the_suite_it_names`]), and an item nobody
//! has claimed a check for has a verdict of its own ([`Verdict::Unclaimed`])
//! instead of being expressible only by deleting its row.
//!
//! Running this file does not re-prove the claims it cites — it proves that
//! every one of the fourteen items has a named, existing, running, asserting
//! answer, or an honest verdict saying it does not. Every `met` verdict below
//! was written after running its own checks and reading the output: 36 checks
//! across `c1a`/`c1b`/`c1c`/`c1d`, all green at `afed0aa9` (S6 P0), plus the
//! two checks item 8's mail wave added, run green in that wave.
//!
//! ## The walk
//!
//! | # | Contract claim (§21) | Verdict | Decisive check |
//! |---|---|---|---|
//! | 1 | fresh ordinary actor stage launches with a pinned context snapshot | met | `c1a_compiled_context::every_fresh_actor_stage_launch_journals_a_snapshot_for_its_own_execution` |
//! | 2 | deterministic research runs before fuzzy retrieval/model dispatch where the profile declares such operations | met | `c1a_compiled_context::the_ledger_refuses_a_fuzzy_step_entered_before_the_deterministic_ones` |
//! | 3 | Bound evidence stays within budget and every unit resolves to source evidence | met | `c1b_tiers_and_budget::the_bound_budget_is_hard_and_a_huge_source_cannot_fill_bound_with_body_text` |
//! | 4 | Referenced coordinates resolve without broad rediscovery | met | `c1b_tiers_and_budget::a_coordinate_resolves_by_direct_lookup_not_by_rediscovery` |
//! | 5 | Reachable map/search/query remains available | met | `c1b_tiers_and_budget::an_exhausted_budget_leaves_reachable_available_and_caps_no_resolution` |
//! | 6 | local knowledge directory evidence can be selected without acquiring Work mutation authority | met | `c1c_authority_and_provenance::knowledge_evidence_is_selected_without_widening_the_works_mutation_scope` |
//! | 7 | structured query results can be Bound while large datasets remain outside prompt context | met | `c1c_authority_and_provenance::a_query_result_is_bound_compact_while_the_dataset_stays_out_of_the_prompt` |
//! | 8 | document/mail/OCR excerpts preserve original resource/native/extractor provenance | met | `c1c_authority_and_provenance::a_document_excerpt_carries_extractor_native_coordinate_and_heading` |
//! | 9 | external evidence is visibly external and cannot alter instruction hierarchy | met | `c1c_authority_and_provenance::an_external_instruction_cannot_displace_reorder_or_escape_the_data_frame` |
//! | 10 | context snapshot pins exact source/Work/query/retrieval generations | met | `c1a_compiled_context::the_snapshot_pins_generations_that_re_resolve_to_the_same_evidence` |
//! | 11 | actor context queries are attributable to execution | met | `c1d_attribution_nesting_audit::a_managed_context_query_is_attributed_to_the_execution_not_merely_the_work` |
//! | 12 | nested leaf stages and child Work receive independent snapshots | met | `c1d_attribution_nesting_audit::each_nested_leaf_actor_gets_its_own_snapshot_and_the_container_gets_none` |
//! | 13 | intelligence-disabled estate still follows existing stage launch path | met | `c1a_compiled_context::a_stage_with_no_compiler_installed_gets_its_authored_context_byte_for_byte` |
//! | 14 | produced artifacts retain enough evidence coordinates for later claim-level audit | met | `c1d_attribution_nesting_audit::a_rendered_claim_fragment_traces_through_the_artifact_to_its_source_evidence` |
//!
//! The `Decisive check` column names **one** check per row so the table stays
//! readable; every row's full check list is in [`WALK`] below, and each one is
//! verified to exist, to run, and to assert something.
//!
//! ## The one gap this register found
//!
//! §21 item 8 names three kinds of excerpt. The **document** third is proven
//! (extractor identity, A2 §9 native coordinate, heading, byte span, with a
//! Markdown control that carries no native coordinate so the test cannot pass
//! by printing a string). The **OCR** third is declared absent rather than
//! omitted, by the owner ruling of 2026-08-29 that keeps OCR out of 0.3.0.
//! The **mail** third has no check at all: `src/runtime/context.rs`'s
//! resolution path is kind-agnostic and A2 §17 item 2 proves mail units carry
//! exact A1 provenance *at the retrieval layer*
//! (`w2_lexical_retrieval::lexical_search_returns_mail_units_with_exact_a1_provenance`),
//! so the mechanism very probably works — and "very probably works" is the
//! exact reasoning that let A2 §2's estate axis go missing under a `met`
//! verdict. The row says gap until something runs.

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
const THIS_FILE: &str = "tests/c1_acceptance.rs";

/// The contract section this register walks, and how many items it has.
///
/// The contract itself is **not in this repository** — it is
/// `knowledge/evidence/resources/host-atlas-series/C1-COMPILED-CONTEXT.md` in
/// the sergeant-rs-workspace estate, which is where the development record
/// lives by design. So no test here can read the contract and diff the table
/// against it; the claim column above was transcribed by hand from §21 lines
/// 346–360 at fixed point `afed0aa9`, and the count is pinned so a row cannot
/// be added or dropped without the guard noticing even though the wording
/// cannot be machine-checked. A1a's register has the same limit for the same
/// reason.
const SECTION: &str = "C1 §21";
const ITEM_COUNT: u8 = 14;

/// No §21 item is deferred in whole. Pinned as an empty set so a later edit
/// cannot defer one quietly: item 8's OCR third is deferred by the owner
/// ruling of 2026-08-29, but the ITEM is a gap, because its mail third is
/// unproven and no ruling covers that.
const DEFERRED_ITEMS: [u8; 0] = [];

// ------------------------------------------------------------------ register

/// What the walk concluded about one §21 item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// Shipped and proven by the named checks, each of which was run at
    /// `afed0aa9` and passed.
    Met,
    /// Not fully provable today. The `note` names what is missing and the
    /// DESTINATION that owns it. Never a pass.
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
    /// Out of 0.3.0 in whole by the owner ruling cited in the `note`.
    ///
    /// **No row carries this today**, and [`DEFERRED_ITEMS`] pins that: the
    /// only 0.3.0 deferral touching §21 is OCR, which is one third of item 8
    /// rather than an item. Kept as vocabulary because a register that cannot
    /// spell a ruling-backed deferral invites one being spelled `met`.
    #[allow(dead_code)]
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

/// One row of the §21 walk.
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
        checks: &[at(
            "tests/c1a_compiled_context.rs",
            "every_fresh_actor_stage_launch_journals_a_snapshot_for_its_own_execution",
        )],
        note: "Live, against a real daemon and a real estate: the snapshot is journaled for the \
               stage's OWN execution, which is the half of the item that a per-Work snapshot \
               would also satisfy and is therefore the half worth testing. The sibling check \
               `an_execute_stage_launch_is_never_compiled` bounds 'ordinary actor' from the \
               other side — an Execute stage is never compiled — and is deliberately NOT cited \
               as decisive here, because it is Docker-gated and returns early on a host with no \
               Docker: a citation whose proof depends on the environment is the shape this \
               register's own citation guard exists to refuse. Ran green at afed0aa9.",
    },
    Item {
        number: 2,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1a_compiled_context.rs",
                "the_ledger_refuses_a_fuzzy_step_entered_before_the_deterministic_ones",
            ),
            at(
                "tests/c1a_compiled_context.rs",
                "the_fuzzy_steps_are_exactly_a2_retrieval_and_every_other_step_precedes_them",
            ),
            at(
                "tests/c1a_compiled_context.rs",
                "the_plan_enters_all_nine_steps_in_section_5s_order_over_a_real_atlas",
            ),
            at(
                "tests/c1a_compiled_context.rs",
                "a_resource_retrieval_also_finds_is_attributed_to_the_deterministic_step_that_held_it_first",
            ),
            at(
                "tests/c1a_compiled_context.rs",
                "the_same_resource_is_attributed_to_retrieval_when_no_deterministic_step_held_it_first",
            ),
        ],
        note: "'Runs before' is enforced rather than observed: the ledger REFUSES a fuzzy step \
               entered before the deterministic ones, by name. The two attribution checks are \
               the pair that makes this non-vacuous — the same resource is attributed to the \
               deterministic step when that step held it first, and to retrieval when no \
               deterministic step did, so the attribution is a function of the ORDER and not of \
               the resource. 'Fuzzy' is pinned to exactly A2 retrieval, steps 6 and 7, so the \
               claim cannot be satisfied by renaming a step. All five ran green at afed0aa9.",
    },
    Item {
        number: 3,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1b_tiers_and_budget.rs",
                "the_bound_budget_is_hard_and_a_huge_source_cannot_fill_bound_with_body_text",
            ),
            at(
                "tests/c1b_tiers_and_budget.rs",
                "every_bound_unit_resolves_to_source_evidence_and_a_fabricated_one_does_not",
            ),
        ],
        note: "Two clauses, one check each, and both fixtures are hostile on purpose: the Work's \
               own overlay holds a 64 KiB unit — eight times the Bound budget — contributed \
               DETERMINISTICALLY by §5 step 2, so 'it did not fit' is about evidence that really \
               was selected rather than evidence that happened to be missing. The resolution \
               check carries a fabricated coordinate as its control, so it cannot pass by \
               resolving nothing. Both ran green at afed0aa9.",
    },
    Item {
        number: 4,
        verdict: Verdict::Met,
        checks: &[at(
            "tests/c1b_tiers_and_budget.rs",
            "a_coordinate_resolves_by_direct_lookup_not_by_rediscovery",
        )],
        note: "'Without broad rediscovery' is the hard half and it is proven negatively: the \
               pinned generation discriminates two rows that content alone cannot, and the \
               search surface never sees the query. A test that only showed the coordinate \
               resolving would be satisfied by a resolver that re-ran a search underneath. Ran \
               green at afed0aa9.",
    },
    Item {
        number: 5,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1b_tiers_and_budget.rs",
                "an_exhausted_budget_leaves_reachable_available_and_caps_no_resolution",
            ),
            at(
                "tests/c1b_tiers_and_budget.rs",
                "the_budget_bounds_the_render_and_not_the_snapshots_evidence_ids",
            ),
        ],
        note: "Read as §14's own last sentence: the Reachable tier is what stays available when \
               the two rendering budgets are spent, and exhausting them caps no resolution. The \
               second check is the structural half — every evidence id survives a ZERO budget, \
               so the budget bounds the render and not the snapshot. What neither check does is \
               drive `sgt map`/`sgt search` themselves at that moment; those verbs' own \
               availability is A2 §17 item 10's and A1 §17 item 13's register rows, cited there \
               rather than duplicated here. Both ran green at afed0aa9.",
    },
    Item {
        number: 6,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1c_authority_and_provenance.rs",
                "knowledge_evidence_is_selected_without_widening_the_works_mutation_scope",
            ),
            at(
                "tests/c1c_authority_and_provenance.rs",
                "a_second_repository_mount_is_never_admitted_however_relevant_it_is",
            ),
            at(
                "tests/c1c_authority_and_provenance.rs",
                "f9_refuses_a_knowledge_source_that_names_a_repository_mount",
            ),
        ],
        note: "The first check is the item's own words; the second is the one that makes it \
               unevadable, because the admission passes filter on AUTHORITY CLASS rather than on \
               a path list — a second, highly relevant repository mount can never be admitted \
               however well it scores. The third closes the containment direction: a \
               `[[knowledge]]` path that resolves inside a repository mount is refused, so the \
               class cannot be laundered by declaring a mount as knowledge. All three ran green \
               at afed0aa9.",
    },
    Item {
        number: 7,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1c_authority_and_provenance.rs",
                "a_query_result_is_bound_compact_while_the_dataset_stays_out_of_the_prompt",
            ),
            at(
                "tests/c1c_authority_and_provenance.rs",
                "a_bound_query_result_carries_all_seven_of_section_10s_lines",
            ),
            at(
                "tests/c1c_authority_and_provenance.rs",
                "a_bound_query_result_and_a_retrieved_row_join_on_the_dataset_key",
            ),
        ],
        note: "A 20,000-row dataset stays entirely outside the prompt while its aggregate is \
               Bound compact — the scale is what makes the claim mean anything, and it is real \
               rows rather than a stubbed count. The seven-lines check is the shape half, and \
               the join check is S5 W5's A2 §17 item 3 consumed at the C1 layer: the bound \
               aggregate and a retrieved row unit share one dataset key. All three ran green at \
               afed0aa9.",
    },
    Item {
        number: 8,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1c_authority_and_provenance.rs",
                "a_document_excerpt_carries_extractor_native_coordinate_and_heading",
            ),
            at(
                "tests/c1c_authority_and_provenance.rs",
                "a_mail_excerpt_carries_its_resource_native_coordinate_and_extractor",
            ),
            at(
                "tests/c1c_authority_and_provenance.rs",
                "an_excerpt_addressed_by_a_native_coordinate_renders_no_empty_byte_span",
            ),
            at(
                "tests/c1c_authority_and_provenance.rs",
                "the_ocr_half_of_item_8_is_declared_absent_rather_than_omitted",
            ),
        ],
        note: "The item names three kinds of excerpt. DOCUMENT: proven — extractor identity, \
               A2 §9 native coordinate, heading and byte span, with a Markdown control that \
               carries no native coordinate so the check cannot pass by printing a string. \
               MAIL: proven at the C1 layer as of this row's update — a compiled, RENDERED \
               prompt carries the `.eml`'s own path, A1 §6.5's `native text-body` (which body \
               of the message, the only coordinate a mail body has) and \
               `extractor mail-parser/0.11.8+eml/v1`. Nothing in that fixture is hand-built: \
               the `.eml` goes through the REAL supervised worker subprocess and the real \
               `record_scan`, because S5 closeout F-AC-02 recorded that a hand-built mail row \
               pins values production never emits and passes with the adapter deleted. Its \
               control is a Markdown unit in the same generation whose native coordinate is \
               asserted absent before the render is read. The check was taken RED on purpose \
               by stripping the native coordinate from the renderer, then reverted. The \
               previous note said the mechanism 'very probably works' from reading a \
               kind-agnostic resolution path — that reading is how A2 §2's estate axis went \
               missing under a `met` verdict, and this row closed by RUNNING it instead. \
               Landing it also surfaced a real defect, now fixed: the C1 render printed \
               `bytes 0..0` beside every native coordinate — F-AC-02's shape on a second \
               render path, A2 §9's 'do not invent cell precision' — guarded by the third \
               check, whose Markdown control keeps the fix from becoming a blanket \
               suppression §12 would forbid. OCR: declared absent rather than omitted, by the \
               owner ruling of 2026-08-29 that keeps OCR out of 0.3.0 entirely. That third is \
               a ruling-backed DEFERRAL, not a gap in this row, and this verdict is not \
               argued from it: the two thirds 0.3.0 ships are each proven by a check above. \
               All four ran green together in this wave; the first, second and fourth ran \
               green at afed0aa9 or (the second) at this row's own commit.",
    },
    Item {
        number: 9,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1c_authority_and_provenance.rs",
                "an_external_instruction_cannot_displace_reorder_or_escape_the_data_frame",
            ),
            at(
                "tests/c1c_authority_and_provenance.rs",
                "a_forged_banner_in_external_text_adds_no_frame_and_claims_no_authority",
            ),
        ],
        note: "The security half is tested adversarially rather than by inspection: the same \
               world is compiled twice, once with benign prose and once with a prompt-injection \
               payload in an external `AGENTS.md`, and the two renders are identical apart from \
               the quoted body bytes — so 'cannot alter instruction hierarchy' is a differential \
               result and not a claim about a template. The forged-banner check covers the other \
               direction, where the payload imitates §11's own frame. Both ran green at \
               afed0aa9. The retrieval-layer half of 'visibly external' is A2 §17 item 8's, \
               cited there.",
    },
    Item {
        number: 10,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1a_compiled_context.rs",
                "the_snapshot_pins_generations_that_re_resolve_to_the_same_evidence",
            ),
            at(
                "tests/c1a_compiled_context.rs",
                "the_snapshot_pins_the_retrieval_generation_and_model_it_actually_used",
            ),
            at(
                "tests/c1a_compiled_context.rs",
                "the_snapshot_carries_one_field_per_line_of_section_15s_list",
            ),
        ],
        note: "'Pins' is proven as RE-RESOLUTION rather than as field presence: the pinned \
               generations resolve to the same evidence a second time, which a recorded string \
               that named nothing would not. The retrieval/model pin is checked on a compilation \
               where retrieval actually ran, and the §15 check walks one field per line of §15's \
               own list so a dropped field is a failure rather than an absence nobody counted. \
               All three ran green at afed0aa9.",
    },
    Item {
        number: 11,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1d_attribution_nesting_audit.rs",
                "a_managed_context_query_is_attributed_to_the_execution_not_merely_the_work",
            ),
            at(
                "tests/c1d_attribution_nesting_audit.rs",
                "the_audit_record_is_raw_evidence_and_carries_no_sharpness_scalar",
            ),
            at(
                "tests/c1d_attribution_nesting_audit.rs",
                "the_seven_event_kinds_are_section_16s_own_list_in_section_16s_own_order",
            ),
        ],
        note: "'Attributable to execution' is the whole claim and the check separates it from \
               the weaker one it is easy to pass instead: two launches of ONE stage are told \
               apart, and `sgt search --work`'s Work-scoped read still has no execution. The \
               other two checks bound the record's shape — §16's seven kinds verbatim in §16's \
               order, and no scalar anywhere, which is §20's forbidden 'sharpness score' pinned \
               as a payload property rather than as a comment. All three ran green at afed0aa9.",
    },
    Item {
        number: 12,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1d_attribution_nesting_audit.rs",
                "each_nested_leaf_actor_gets_its_own_snapshot_and_the_container_gets_none",
            ),
            at(
                "tests/c1d_attribution_nesting_audit.rs",
                "a_child_work_inherits_no_parent_prompt_and_gets_its_own_binding",
            ),
            at(
                "tests/c1d_attribution_nesting_audit.rs",
                "a_causal_childs_parent_causation_is_referenced_as_a_pointer",
            ),
            at(
                "tests/c1d_attribution_nesting_audit.rs",
                "a_container_cannot_acquire_an_actor_procedure_and_a_leaf_can",
            ),
        ],
        note: "Both halves of the item are live, against a real daemon: each nested leaf actor \
               gets its own snapshot under its composed id and the container that closes on them \
               gets none, and a causal child gets its own Work/source/context binding while \
               inheriting no byte of the parent's prompt — tested adversarially rather than by \
               absence. The pointer check keeps rule 5's Referenced parent causation from being \
               confused with inheritance, and the container check is structural: the identical \
               file one directory down loads fine, so the refusal is about the container and not \
               about the file. All four ran green at afed0aa9.",
    },
    Item {
        number: 13,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1a_compiled_context.rs",
                "a_stage_with_no_compiler_installed_gets_its_authored_context_byte_for_byte",
            ),
            at(
                "tests/c1a_compiled_context.rs",
                "an_estate_with_no_confirmed_generation_leaves_the_context_byte_identical_and_says_why",
            ),
            at(
                "tests/c1a_compiled_context.rs",
                "an_unindexed_estate_launches_on_the_existing_stage_context_path",
            ),
        ],
        note: "'Still follows the existing path' is checked as BYTE IDENTITY, which is the only \
               reading that cannot drift: no compiler installed, and intelligence installed but \
               with nothing confirmed, both leave the authored context byte for byte — and the \
               second says why rather than degrading silently. The third check is the live one, \
               a real unindexed estate launching on the existing stage-context path, which is \
               the claim an in-process check cannot make. All three ran green at afed0aa9.",
    },
    Item {
        number: 14,
        verdict: Verdict::Met,
        checks: &[
            at(
                "tests/c1d_attribution_nesting_audit.rs",
                "a_rendered_claim_fragment_traces_through_the_artifact_to_its_source_evidence",
            ),
            at(
                "tests/c1d_attribution_nesting_audit.rs",
                "section_19s_page_and_bbox_arm_is_absent_by_owner_ruling",
            ),
        ],
        note: "'Enough evidence coordinates for later claim-level audit' is proven by doing the \
               audit: the check walks §19's chain by hand from a rendered claim fragment through \
               the journaled artifact to the exact source row, and every link of the chain \
               survives. That is the honest reading of the item — §19 says a first-class claim \
               graph is NOT required and that preserving exact coordinates is the enabling \
               invariant, so the test shows a later wave COULD build one and builds nothing that \
               stores or indexes claims. The second check keeps §19's page+bbox arm present and \
               null rather than omitted, which is the same absent-not-hidden discipline item 8's \
               OCR third gets. Both ran green at afed0aa9.",
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

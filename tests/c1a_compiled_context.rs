//! S6 C1a — the compilation step (C1 §3) and §5's **enforceable runtime
//! order**, with §15's snapshot provenance and §21 items 1, 2, 10 and 13.
//!
//! # What each test here is the evidence for
//!
//! | Claim | Test |
//! |---|---|
//! | §5's nine steps are §5's own nine lines, in §5's own order | [`the_nine_steps_are_section_5s_own_list_in_section_5s_own_order`] |
//! | *"fuzzy"* means exactly A2 retrieval — steps 6 and 7 and nothing else | [`the_fuzzy_steps_are_exactly_a2_retrieval_and_every_other_step_precedes_them`] |
//! | **item 2** — a fuzzy step entered early is REFUSED, by name | [`the_ledger_refuses_a_fuzzy_step_entered_before_the_deterministic_ones`] |
//! | the refusal does not over-claim a cognition violation that did not happen | [`skipping_only_the_lexical_step_is_reported_as_sequencing_not_as_cognition_first`] |
//! | every one of the nine steps is *entered*, in order, over a real Atlas | [`the_plan_enters_all_nine_steps_in_section_5s_order_over_a_real_atlas`] |
//! | **item 2, decisively** — a resource retrieval also found is attributed to the deterministic step that held it first, and retrieval is observed **losing** it | [`a_resource_retrieval_also_finds_is_attributed_to_the_deterministic_step_that_held_it_first`] |
//! | and that attribution is a function of the ORDER, not of the resource | [`the_same_resource_is_attributed_to_retrieval_when_no_deterministic_step_held_it_first`] |
//! | **item 10** — the snapshot pins generations that **re-resolve** | [`the_snapshot_pins_generations_that_re_resolve_to_the_same_evidence`] |
//! | **item 10** — the retrieval generation/model is pinned when retrieval ran | [`the_snapshot_pins_the_retrieval_generation_and_model_it_actually_used`] |
//! | §15 — one field per line of §15's list, in §15's order | [`the_snapshot_carries_one_field_per_line_of_section_15s_list`] |
//! | §2 — a Bound unit renders its coordinate **and** the body §14's budget allowed | [`the_rendered_section_carries_the_coordinate_beside_the_body_it_bound`] |
//! | §15 — the selection-plan hash is a hash of the PLAN, not of the selection | [`the_selection_plan_hash_separates_two_plans_that_selected_the_same_evidence`] |
//! | **item 13** — no compiler installed ⇒ the stage context is byte-identical | [`a_stage_with_no_compiler_installed_gets_its_authored_context_byte_for_byte`] |
//! | **item 13** — intelligence *unavailable* ⇒ the same, and it says why | [`an_estate_with_no_confirmed_generation_leaves_the_context_byte_identical_and_says_why`] |
//! | **item 1** — a fresh ordinary actor stage launches with a snapshot pinned to **its own** execution | [`every_fresh_actor_stage_launch_journals_a_snapshot_for_its_own_execution`] |
//! | **item 13**, live — an unindexed estate still launches on the existing path | [`an_unindexed_estate_launches_on_the_existing_stage_context_path`] |
//!
//! The two live tests at the bottom run a real daemon over a real estate;
//! everything above them runs the compiler over a real Atlas built by the
//! ordinary `record_scan` path. Neither substitutes for the other.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::domain::event::rfc3339_utc_now;
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::domain::workflow::{
    KIND_CONTEXT_COMPILED, StageDefinition, StageKind, StageRecord, StageStatus,
};
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::overlay::overlay_source_name;
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{ScannedFile, ScannedUnit, SourceScan};
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR;
use sergeant_rs::runtime::context::{
    Cognition, CompileRequest, ContextSnapshot, Degradation, EvidenceCoordinate, OrderViolation,
    ResearchLedger, ResearchStep, Tier, compile,
};
use sergeant_rs::runtime::journal::Journal;

mod support;

// ===================================================================
// §5's own nine lines, transcribed from C1-COMPILED-CONTEXT.md §5
// ===================================================================

/// The contract's list, copied verbatim. A test compares
/// [`ResearchStep::PLAN`] against *this*, not against itself.
const SECTION_5: [&str; 9] = [
    "explicit stage inputs / prior declared artifacts",
    "exact Work bindings/changed resources",
    "exact structural/document/tabular/mail relationships",
    "deterministic dataset aggregates/joins/diffs declared by the profile/workflow",
    "exact Referenced neighbors",
    "A2 lexical retrieval",
    "A2 semantic retrieval if installed/needed",
    "bounded structural/provenance expansion",
    "pack Bound; emit useful remainder as Referenced",
];

// ===================================================================
// Fixtures
// ===================================================================

const WORK_ID: &str = "01C1AWORK";
const REPOSITORY: &str = "demo-repo";
/// The body of the contested resource. Its terms appear in the Work's intent
/// too, so lexical retrieval genuinely reaches for it — which is what stops
/// the ordering test from being vacuous.
const CONTESTED_BODY: &str = "The retention retention retention policy this Work changes.";

fn unit(ordinal: u64, title: &str, text: &str) -> ScannedUnit {
    ScannedUnit {
        ordinal,
        kind: UnitKind::Section,
        heading_level: Some(1),
        title: Some(title.to_string()),
        byte_start: 0,
        byte_end: text.len() as u64,
        coordinate: None,
        text: text.to_string(),
    }
}

fn file(relative_path: &str, units: Vec<ScannedUnit>) -> ScannedFile {
    let bytes: u64 = units.iter().map(|u| u.text.len() as u64).sum();
    ScannedFile {
        relative_path: relative_path.to_string(),
        content_hash: format!("hash/{relative_path}"),
        extractor: MARKDOWN_EXTRACTOR.to_string(),
        local_key: format!("key/{relative_path}"),
        byte_len: bytes,
        mtime_millis: None,
        units,
        syntax: None,
        parent: None,
    }
}

fn scan(
    source_name: &str,
    kind: SourceKind,
    authority: AuthorityClass,
    files: Vec<ScannedFile>,
) -> SourceScan {
    let mut extractors = BTreeSet::new();
    extractors.insert(MARKDOWN_EXTRACTOR.to_string());
    SourceScan {
        source_name: source_name.to_string(),
        kind,
        authority,
        content_key: format!("{source_name}@generation-1"),
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

/// A real Atlas holding two confirmed generations: the Work's base repository
/// and the Work's own overlay over it.
///
/// The overlay's `notes/plan.md` is the **contested** resource: it is one of
/// the Work's exact changed resources (§5 step 2) *and* the best lexical
/// answer to the Work's intent (§5 step 6). Both steps reach for it, which is
/// what makes the ordering test non-vacuous.
struct Fixture {
    _data: TempDir,
    db: AtlasDb,
}

fn fixture() -> Fixture {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    let base = scan(
        REPOSITORY,
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        vec![
            file(
                "docs/architecture.md",
                vec![unit(
                    0,
                    "Architecture",
                    "The daemon owns the journal and the projection it folds.",
                )],
            ),
            file(
                // A synthetic fixture path, deliberately not any real docs/
                // path: `f_doctrine_skew`'s removed-path guard scans test
                // sources for citations of files split-hardening W2c deleted,
                // and it cannot tell a fixture from a citation. Naming a live
                // doc here would also couple this fixture to that file's
                // location. This name belongs to no real or removed file.
                "docs/team-vocabulary.md",
                vec![unit(0, "Glossary", "A surface is a linked worktree.")],
            ),
        ],
    );
    record_scan(&mut db, &mut journal, &base, None).expect("record base");

    let overlay = scan(
        &overlay_source_name(WORK_ID, REPOSITORY),
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        vec![file("notes/plan.md", vec![unit(0, "Plan", CONTESTED_BODY)])],
    );
    record_scan(&mut db, &mut journal, &overlay, None).expect("record overlay");

    Fixture { _data: data, db }
}

fn stage() -> StageDefinition {
    StageDefinition {
        id: "10-implement".to_string(),
        context: "authored stage procedure".to_string(),
        kind: StageKind::Actor,
        harness: None,
        profile: None,
        requires_ask: false,
        receives_branch_status: false,
        execute: None,
    }
}

fn bindings() -> Vec<sergeant_rs::backend::BindingSummary> {
    vec![sergeant_rs::backend::BindingSummary {
        repository: REPOSITORY.to_string(),
        worktree_path: PathBuf::from("/surfaces").join(WORK_ID).join(REPOSITORY),
        work_branch: format!("sergeant/{WORK_ID}"),
        base_branch: Some("main".to_string()),
        base_sha: "0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f".to_string(),
    }]
}

fn prior_stages() -> Vec<StageRecord> {
    vec![StageRecord {
        stage_id: "00-orient".to_string(),
        index: 0,
        attempt: 1,
        status: StageStatus::Completed,
        detail: Some("oriented".to_string()),
        output_reprompts: 0,
    }]
}

/// Compile against `atlas`, with the whole fixture world bound to this Work.
fn compiled(atlas: Option<&AtlasDb>) -> ContextSnapshot {
    let stage = stage();
    let bindings = bindings();
    let prior = prior_stages();
    compile(
        atlas,
        &CompileRequest {
            estate_root: Some(Path::new("/estates/demo")),
            work_id: WORK_ID,
            intent: "tighten the retention policy",
            stage: &stage,
            stage_index: 1,
            attempt: 1,
            execution_id: "01EXECUTION",
            journal_watermark: 42,
            bindings: &bindings,
            prior_stages: &prior,
            profile: Some("standard"),
            budget: sergeant_rs::runtime::context::RenderBudget::DEFAULT,
        },
    )
}

// ============================================================ §5's order

/// §5's nine steps are §5's nine lines, in §5's order — compared against the
/// contract's own text, transcribed above, never against the code's own
/// listing.
#[test]
fn the_nine_steps_are_section_5s_own_list_in_section_5s_own_order() {
    let labels: Vec<&str> = ResearchStep::PLAN.iter().map(|s| s.label()).collect();
    assert_eq!(labels, SECTION_5.to_vec());
    for (index, step) in ResearchStep::PLAN.iter().enumerate() {
        assert_eq!(step.number(), index + 1, "§5 numbers its list from 1");
    }
}

/// *"Before **fuzzy retrieval or model dispatch**"* — the fuzzy half of §5 is
/// exactly its two A2 retrieval steps, and every other step is deterministic.
///
/// The second assertion is the one that carries the sentence: every
/// deterministic step §5 lists *before* a fuzzy one really is numbered before
/// it. A classification that drifted (step 8 marked fuzzy, say, or step 5
/// renumbered past step 6) turns this red.
#[test]
fn the_fuzzy_steps_are_exactly_a2_retrieval_and_every_other_step_precedes_them() {
    let fuzzy: Vec<usize> = ResearchStep::PLAN
        .iter()
        .filter(|s| s.cognition() == Cognition::Cognition)
        .map(|s| s.number())
        .collect();
    assert_eq!(fuzzy, vec![6, 7], "§5's steps 6 and 7, and nothing else");

    let first_fuzzy = *fuzzy.first().expect("there is a fuzzy step");
    let deterministic_before: Vec<usize> = ResearchStep::PLAN
        .iter()
        .filter(|s| s.cognition() == Cognition::Computation && s.number() < first_fuzzy)
        .map(|s| s.number())
        .collect();
    assert_eq!(
        deterministic_before,
        vec![1, 2, 3, 4, 5],
        "all five of §5's local-evidence operations come before the first fuzzy step"
    );
}

/// **§21 item 2, as a gate.** A fuzzy step entered before the deterministic
/// ones is REFUSED, and the refusal names §5's own sentence.
///
/// This is the technique, not an assertion about it: a real violation is
/// inserted and the guard is watched. The second half is what keeps it
/// non-vacuous — the legal order is accepted by the same call, so the gate is
/// not simply refusing everything.
#[test]
fn the_ledger_refuses_a_fuzzy_step_entered_before_the_deterministic_ones() {
    let mut ledger = ResearchLedger::new();
    let refusal = ledger
        .enter(ResearchStep::LexicalRetrieval)
        .expect_err("§5's step 6 entered first must be refused");
    assert_eq!(
        refusal,
        OrderViolation::CognitionBeforeComputation {
            fuzzy: 6,
            fuzzy_label: SECTION_5[5],
            deterministic: 1,
            deterministic_label: SECTION_5[0],
        }
    );
    assert!(
        refusal
            .to_string()
            .contains("spend computation before cognition"),
        "the refusal quotes the sentence it enforces: {refusal}"
    );

    // Non-vacuous: the same gate accepts §5's actual order.
    let mut ledger = ResearchLedger::new();
    for step in ResearchStep::PLAN {
        assert!(
            ledger.enter(step).is_ok(),
            "§5's own order must pass the gate that refused the reordering: {step:?}"
        );
    }
    assert!(ledger.complete());
}

/// The refusal is honest about *which* rule was broken.
///
/// Steps 1–5 have run, so nothing deterministic is being jumped; entering
/// step 7 before step 6 is a sequencing bug between two fuzzy steps, and it
/// is reported as [`OrderViolation::OutOfOrder`]. A gate that reported every
/// misordering as a cognition-before-computation violation would be a gate
/// nobody could read.
#[test]
fn skipping_only_the_lexical_step_is_reported_as_sequencing_not_as_cognition_first() {
    let mut ledger = ResearchLedger::new();
    for step in ResearchStep::PLAN.iter().take(5) {
        drop(ledger.enter(*step).expect("the first five are in order"));
    }
    let refusal = ledger
        .enter(ResearchStep::SemanticRetrieval)
        .expect_err("step 7 cannot precede step 6");
    assert_eq!(
        refusal,
        OrderViolation::OutOfOrder {
            attempted: 7,
            attempted_label: SECTION_5[6],
            expected: 6,
            expected_label: SECTION_5[5],
        }
    );
}

/// Every one of §5's nine steps is *entered* on a real compilation, in §5's
/// order — including the ones that contribute nothing, which record why
/// (§18: degradation is *"visible, not fatal or fabricated"*).
#[test]
fn the_plan_enters_all_nine_steps_in_section_5s_order_over_a_real_atlas() {
    let fixture = fixture();
    let snapshot = compiled(Some(&fixture.db));
    assert!(snapshot.degradation.is_none(), "{:?}", snapshot.degradation);

    let executed: Vec<usize> = snapshot.plan.iter().map(|r| r.step.number()).collect();
    assert_eq!(executed, (1..=9).collect::<Vec<_>>());
    for record in &snapshot.plan {
        assert!(
            record.contributed > 0 || record.note.is_some(),
            "a step that contributed nothing must say why: {record:?}"
        );
    }
    // §21 item 2's own qualifier — *"where the profile declares such
    // operations"* — is answered honestly rather than silently skipped.
    let step_4 = &snapshot.plan[3];
    assert_eq!(step_4.contributed, 0);
    assert!(
        step_4
            .note
            .as_deref()
            .is_some_and(|n| n.contains("declared")),
        "{step_4:?}"
    );
}

// ============================================================ item 2, decisive

/// **§21 item 2, decisively.** The contested resource is one the *fuzzy* step
/// would have taken if it had run first — and it does not get it.
///
/// Three assertions, and all three are needed:
///
/// 1. `notes/plan.md` is bound, attributed to **step 2** (the exact Work
///    changed resource), not step 6.
/// 2. Step 6 **offered it and lost** — its `already_held` count is non-zero,
///    which is only possible if lexical retrieval actually reached the same
///    resource. Without this the test would pass on a world where retrieval
///    never found it, which is the vacuity trap.
/// 3. No evidence unit anywhere in the snapshot attributes that resource to a
///    fuzzy step.
#[test]
fn a_resource_retrieval_also_finds_is_attributed_to_the_deterministic_step_that_held_it_first() {
    let fixture = fixture();
    let snapshot = compiled(Some(&fixture.db));

    let contested: Vec<&sergeant_rs::runtime::context::EvidenceUnit> = snapshot
        .bound
        .iter()
        .filter(|u| {
            matches!(&u.coordinate,
            EvidenceCoordinate::Atlas { relative_path, .. } if relative_path == "notes/plan.md")
        })
        .collect();
    assert_eq!(
        contested.len(),
        1,
        "exactly one unit binds the contested resource: {:?}",
        snapshot.bound
    );
    assert_eq!(
        contested[0].step,
        ResearchStep::WorkBindings,
        "§5 step 2 held it first, so §5 step 2 owns it"
    );
    assert_eq!(contested[0].tier, Tier::Bound);

    let step_6 = snapshot
        .plan
        .iter()
        .find(|r| r.step == ResearchStep::LexicalRetrieval)
        .expect("step 6 ran");
    assert!(
        step_6.already_held > 0,
        "lexical retrieval must actually have reached for evidence a deterministic step \
         already held — otherwise this test proves nothing about order: {step_6:?}"
    );

    for unit in snapshot.bound.iter().chain(snapshot.referenced.iter()) {
        if let EvidenceCoordinate::Atlas { relative_path, .. } = &unit.coordinate
            && relative_path == "notes/plan.md"
        {
            assert_eq!(
                unit.step.cognition(),
                Cognition::Computation,
                "no fuzzy step may end up owning the contested resource: {unit:?}"
            );
        }
    }
}

/// The counterpart, and the reason the test above is about **order** rather
/// than about this particular resource: contribute the identical coordinate
/// from step 6 with no deterministic step having held it, and step 6 owns it.
///
/// Same ledger, same coordinate, same gate — only the order differs, and the
/// recorded attribution follows the order. That is the observable state
/// difference the brief asks for; it is not an argument about the code.
#[test]
fn the_same_resource_is_attributed_to_retrieval_when_no_deterministic_step_held_it_first() {
    let coordinate = || EvidenceCoordinate::Atlas {
        source_name: "s".to_string(),
        generation_id: "g".to_string(),
        content_key: "c".to_string(),
        relative_path: "notes/plan.md".to_string(),
        unit_key: None,
        ordinal: Some(0),
    };

    // Deterministic step 2 holds it: step 6 offers and loses.
    let mut held_first = ResearchLedger::new();
    drop(held_first.enter(ResearchStep::StageInputs).expect("1"));
    {
        let mut step = held_first.enter(ResearchStep::WorkBindings).expect("2");
        assert!(step.contribute(Tier::Bound, coordinate()));
    }
    for step in [
        ResearchStep::ExactRelationships,
        ResearchStep::DeclaredDataOperations,
        ResearchStep::ReferencedNeighbors,
    ] {
        drop(held_first.enter(step).expect("3-5"));
    }
    {
        let mut step = held_first.enter(ResearchStep::LexicalRetrieval).expect("6");
        assert!(
            !step.contribute(Tier::Bound, coordinate()),
            "an earlier step already holds it"
        );
    }
    assert_eq!(held_first.units().len(), 1);
    assert_eq!(held_first.units()[0].step, ResearchStep::WorkBindings);

    // Nothing deterministic held it: step 6 owns it, same coordinate.
    let mut retrieval_first = ResearchLedger::new();
    for step in ResearchStep::PLAN.iter().take(5) {
        drop(retrieval_first.enter(*step).expect("1-5"));
    }
    {
        let mut step = retrieval_first
            .enter(ResearchStep::LexicalRetrieval)
            .expect("6");
        assert!(step.contribute(Tier::Bound, coordinate()));
    }
    assert_eq!(retrieval_first.units().len(), 1);
    assert_eq!(
        retrieval_first.units()[0].step,
        ResearchStep::LexicalRetrieval,
        "with no deterministic holder, the fuzzy step owns it — so the attribution above \
         was produced by the ORDER, not by the resource"
    );
}

// ============================================================ item 10 / §15

/// **§21 item 10.** *"the snapshot pins exact source/Work/query/retrieval
/// generations"* — and a pin is only a pin if it **re-resolves**.
///
/// Each pinned generation is looked back up in the store by its own source
/// name and must answer with the same generation id and content key. A
/// snapshot that merely *described* the world would pass a field-presence
/// check and fail this one.
#[test]
fn the_snapshot_pins_generations_that_re_resolve_to_the_same_evidence() {
    let fixture = fixture();
    let snapshot = compiled(Some(&fixture.db));

    assert!(
        snapshot.source_generations.len() >= 2,
        "the base and the overlay are both in this Work's world: {:?}",
        snapshot.source_generations
    );
    for pin in &snapshot.source_generations {
        let resolved = fixture
            .db
            .confirmed_generation(&pin.source_name)
            .expect("read")
            .unwrap_or_else(|| panic!("pin does not re-resolve: {pin:?}"));
        assert_eq!(resolved.id, pin.generation_id);
        assert_eq!(resolved.content_key, pin.content_key);
    }

    // §15's *"Work base + overlay generation"*, both halves.
    let overlay = snapshot
        .work_world
        .overlay_generation
        .as_ref()
        .expect("this Work has an overlay generation");
    assert_eq!(
        overlay.source_name,
        overlay_source_name(WORK_ID, REPOSITORY)
    );
    assert_eq!(
        snapshot.work_world.base_sha.as_deref(),
        Some("0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f")
    );
    assert_eq!(snapshot.journal_watermark, 42);
    assert_eq!(snapshot.coordinate.execution_id, "01EXECUTION");
}

/// §15's *"retrieval generation/model if used"* — A2 §13's whole trace,
/// persisted for a **managed** search, which is the destination
/// `runtime::atlas::trace` named C1 for.
#[test]
fn the_snapshot_pins_the_retrieval_generation_and_model_it_actually_used() {
    let fixture = fixture();
    let snapshot = compiled(Some(&fixture.db));
    let trace = snapshot.retrieval.as_ref().expect("steps 6/7 ran a search");

    assert_eq!(trace.query.text, "tighten the retention policy");
    assert_eq!(
        trace.attribution,
        sergeant_rs::runtime::atlas::trace::Attribution::Work {
            work_id: WORK_ID.to_string(),
            repository: REPOSITORY.to_string(),
        },
        "a search a Work's own execution issued is a MANAGED search (§13)"
    );
    assert!(
        !trace.retrieval_generation.generations.is_empty(),
        "the trace names the exact generations the answer was computed over"
    );
    // A2 §15's non-omittable honesty rides through onto the snapshot.
    let json = snapshot.json();
    assert!(json["retrieval"]["semantic"].is_string(), "{json}");
}

/// §15 lists sixteen lines. The snapshot has one field per line, in §15's own
/// order — the discipline `SearchTrace` already applies to A2 §13.
#[test]
fn the_snapshot_carries_one_field_per_line_of_section_15s_list() {
    let fixture = fixture();
    let json = compiled(Some(&fixture.db)).json();
    for field in ContextSnapshot::FIELDS {
        assert!(
            json.get(field).is_some(),
            "§15's {field:?} is missing from the snapshot payload: {json}"
        );
    }
    // The two lines a later wave fills are present and empty, never absent —
    // an absent field cannot be told from a forgotten one.
    assert_eq!(json["query_result_ids"], serde_json::json!([]));
    assert_eq!(json["payload_pointer"], Value::Null);
    // §15's `budget` line stopped being empty when C1b landed §14's two
    // budgets; `tests/c1b_tiers_and_budget.rs` owns what it says.
    assert_eq!(json["budget"]["unit"], "bytes", "{json}");
}

// ============================================================ §20's non-goals

/// §20's first non-goal: *"raw corpus stuffing"* — and what C1b changed
/// about it.
///
/// C1a rendered coordinates and never a body, because §2's Bound tier
/// (*"evidence deliberately **rendered into** the actor's initial context"*)
/// cannot be delivered safely without §14's budget, and that budget was the
/// next wave's. C1b landed it, so a Bound body now renders — **inside a hard
/// budget**, which is the whole difference between §2's Bound tier and §20's
/// first non-goal. This test pins that the authored procedure is still
/// untouched, that the coordinate is still there beside the body, and that
/// Reachable still renders; `tests/c1b_tiers_and_budget.rs` owns the budget
/// itself, including that a source larger than the budget cannot fill Bound
/// with body text.
#[test]
fn the_rendered_section_carries_the_coordinate_beside_the_body_it_bound() {
    let fixture = fixture();
    let snapshot = compiled(Some(&fixture.db));
    let rendered = snapshot.render_onto("authored stage procedure");

    assert!(
        rendered.starts_with("authored stage procedure"),
        "{rendered}"
    );
    assert!(rendered.contains("notes/plan.md"), "{rendered}");
    assert!(
        rendered.contains(CONTESTED_BODY),
        "§2's Bound tier renders evidence INTO the context, within §14's \
         budget: {rendered}"
    );
    assert!(rendered.contains("REACHABLE"), "{rendered}");
}

/// §15's *"selection-plan hash"* is a hash of the **plan**, not of the set it
/// selected. Two ledgers that end up holding the identical coordinate hash
/// differently when a different step contributed it.
#[test]
fn the_selection_plan_hash_separates_two_plans_that_selected_the_same_evidence() {
    let fixture = fixture();
    let a = compiled(Some(&fixture.db));
    let b = compiled(Some(&fixture.db));
    assert_eq!(
        a.selection_plan_hash, b.selection_plan_hash,
        "the same plan over the same world hashes the same"
    );

    // A different plan over the same world: no prior stage artifact, so step
    // 1 contributes nothing and the executed record differs even though every
    // Atlas coordinate selected is identical.
    let stage = stage();
    let bindings = bindings();
    let c = compile(
        Some(&fixture.db),
        &CompileRequest {
            estate_root: Some(Path::new("/estates/demo")),
            work_id: WORK_ID,
            intent: "tighten the retention policy",
            stage: &stage,
            stage_index: 1,
            attempt: 1,
            execution_id: "01EXECUTION",
            journal_watermark: 42,
            bindings: &bindings,
            prior_stages: &[],
            profile: Some("standard"),
            budget: sergeant_rs::runtime::context::RenderBudget::DEFAULT,
        },
    );
    assert_ne!(
        a.selection_plan_hash, c.selection_plan_hash,
        "a plan whose steps did different work hashes differently"
    );
}

// ============================================================ item 13

/// **§21 item 13, at the type.** No compiler installed ⇒ §18's first rung:
/// *"intelligence disabled → existing stage CONTEXT + Work bindings"*.
///
/// The authored context comes back **byte for byte**, and the snapshot says
/// which rung of §18 it is on rather than leaving an empty snapshot to be
/// interpreted.
#[test]
fn a_stage_with_no_compiler_installed_gets_its_authored_context_byte_for_byte() {
    let snapshot = compiled(None);
    assert_eq!(
        snapshot.degradation,
        Some(Degradation::IntelligenceDisabled)
    );
    assert!(snapshot.bound.is_empty() && snapshot.referenced.is_empty());
    let authored = "authored stage procedure";
    assert_eq!(snapshot.render_onto(authored), authored);
}

/// **§21 item 13's other half**: intelligence *installed but unavailable* —
/// an Atlas with no confirmed generation for this Work's world.
///
/// §3's degradation clause says *"the existing stage context path still
/// executes"*, and that is what this asserts: the same bytes, and a stated
/// reason that is **not** the disabled one, because the two are different
/// facts about the estate.
#[test]
fn an_estate_with_no_confirmed_generation_leaves_the_context_byte_identical_and_says_why() {
    let empty = AtlasDb::open_in_memory().expect("atlas");
    let snapshot = compiled(Some(&empty));
    assert_eq!(
        snapshot.degradation,
        Some(Degradation::NoConfirmedGeneration)
    );
    let authored = "authored stage procedure";
    assert_eq!(snapshot.render_onto(authored), authored);
    assert!(
        snapshot.plan.is_empty(),
        "no step runs in a world with no generation to run over"
    );
}

// ============================================================ live estate

const FAKE: &str = sergeant_rs::backend::fake::FAKE_BACKEND_NAME;

/// Start a daemon over `data_dir` with one scripted fake backend.
async fn start(
    data_dir: &Path,
    registry: sergeant_rs::backend::BackendRegistry,
) -> sergeant_rs::daemon::DaemonHandle {
    sergeant_rs::daemon::start_with(
        data_dir,
        sergeant_rs::daemon::DaemonConfig {
            backends: std::sync::Arc::new(registry),
            default_backend: Some(FAKE.to_string()),
            claude: None,
            ..sergeant_rs::daemon::DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

/// A registry holding one scripted fake.
fn one_fake(
    script: impl IntoIterator<Item = sergeant_rs::backend::fake::FakeStep>,
) -> (
    sergeant_rs::backend::BackendRegistry,
    sergeant_rs::backend::fake::FakeBackend,
) {
    let fake = sergeant_rs::backend::fake::FakeBackend::scripted(FAKE, script);
    (
        sergeant_rs::backend::BackendRegistry::new().with(std::sync::Arc::new(fake.clone())),
        fake,
    )
}

/// Submit the two-stage `tiny` workflow against `estate`, and return the id
/// of the Work it created.
async fn submit_tiny(handle: &sergeant_rs::daemon::DaemonHandle, estate: &Path) -> String {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("client");
    let body = serde_json::json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "intent": "compile a world for this stage",
        "estate_root": estate,
        "origin": {"client": "cli", "cwd": estate},
        "workflow": "tiny",
    });
    let response = client
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&body)
        .send()
        .await
        .expect("request");
    let status = response.status();
    let value: Value = response.json().await.expect("json body");
    assert_eq!(status, 201, "submit rejected: {value}");
    assert_eq!(value["work"]["state"], "completed", "{value}");
    value["work"]["id"].as_str().expect("work id").to_string()
}

/// Every journaled event of one kind for one Work, in journal order.
fn events_of(data_dir: &Path, work_id: &str, kind: &str) -> Vec<sergeant_rs::domain::event::Event> {
    Journal::replay_data_dir(data_dir)
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.work_id.as_deref() == Some(work_id) && e.kind == kind)
        .collect()
}

/// **§21 item 1**: *"fresh ordinary actor stage launches with a pinned
/// context snapshot"* — against a real daemon, a real estate and a real
/// stage launch.
///
/// The snapshot is journaled once per stage entry and names the exact
/// execution the reservation then allocates, so *"what world did Sergeant
/// present?"* is answerable for **that** fresh execution rather than for the
/// Work in general (§15's *"Every fresh execution can answer…"*).
#[tokio::test]
async fn every_fresh_actor_stage_launch_journals_a_snapshot_for_its_own_execution() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (_repo, _head) = support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([
        sergeant_rs::backend::fake::FakeStep::complete_with("first done"),
        sergeant_rs::backend::fake::FakeStep::complete_with("second done"),
    ]);
    let handle = start(data.path(), registry).await;
    let work_id = submit_tiny(&handle, &estate).await;

    let starts = fake.starts();
    assert_eq!(starts.len(), 2, "{starts:?}");

    let snapshots = events_of(data.path(), &work_id, KIND_CONTEXT_COMPILED);
    assert_eq!(
        snapshots.len(),
        2,
        "one snapshot per fresh actor stage launch, no more and no fewer"
    );
    for (snapshot, start) in snapshots.iter().zip(starts.iter()) {
        let payload = &snapshot.payload;
        assert_eq!(
            payload["coordinate"]["execution"].as_str(),
            Some(start.execution_id.as_str()),
            "the snapshot names the execution it was compiled for: {payload}"
        );
        assert_eq!(
            payload["coordinate"]["work"].as_str(),
            Some(work_id.as_str())
        );
        assert_eq!(
            payload["coordinate"]["stage"].as_str(),
            Some(start.stage_id.as_str())
        );
        for field in ContextSnapshot::FIELDS {
            assert!(payload.get(field).is_some(), "§15's {field:?}: {payload}");
        }
    }

    handle.shutdown().await;
}

/// **§21 item 13, live.** An estate this host has indexed nothing for still
/// launches on the existing stage context path: the actor receives the
/// stage's authored `CONTEXT.md` **byte for byte**, and the journal states
/// which rung of §18 that is instead of leaving an empty snapshot to be
/// interpreted.
#[tokio::test]
async fn an_unindexed_estate_launches_on_the_existing_stage_context_path() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (_repo, _head) = support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([
        sergeant_rs::backend::fake::FakeStep::complete_with("first done"),
        sergeant_rs::backend::fake::FakeStep::complete_with("second done"),
    ]);
    let handle = start(data.path(), registry).await;
    let work_id = submit_tiny(&handle, &estate).await;

    let starts = fake.starts();
    assert_eq!(
        starts[0].context, "first stage context",
        "the authored CONTEXT.md, byte for byte — nothing appended"
    );
    assert_eq!(starts[1].context, "second stage context");

    let snapshots = events_of(data.path(), &work_id, KIND_CONTEXT_COMPILED);
    assert!(
        !snapshots.is_empty(),
        "the degradation is journaled, not silent"
    );
    for snapshot in &snapshots {
        let reason = snapshot.payload["degradation"]["reason"].as_str();
        assert!(
            matches!(
                reason,
                Some("no_confirmed_generation") | Some("intelligence_disabled")
            ),
            "degradation must be stated: {}",
            snapshot.payload
        );
        assert_eq!(snapshot.payload["rendered_bytes"], 0);
    }

    handle.shutdown().await;
}

/// Whether the local Docker Engine answers at all — same probe and skip
/// convention as `tests/m7_docker_executor.rs`/`tests/a4_blob_ref_pinning.rs`
/// (CONTRIBUTING.md's environment posture): a host with no Docker reachable
/// skips loudly rather than failing on a shape it cannot express.
fn docker_unavailable() -> Option<&'static str> {
    match std::process::Command::new("docker").arg("version").output() {
        Ok(out) if out.status.success() => None,
        Ok(_) => Some("SKIPPED-ENV: `docker version` exited nonzero on this host"),
        Err(_) => Some("SKIPPED-ENV: no `docker` binary reachable on this host"),
    }
}

macro_rules! require_docker {
    () => {
        if let Some(reason) = docker_unavailable() {
            eprintln!("{reason}");
            return;
        }
    };
}

/// **F-SF-01**: C1 §3 scopes the compilation step to *"the actor start"* —
/// §21 item 1's *"fresh ordinary actor stage launches"* — never to an
/// `Execute` (Docker) stage, which never reaches an actor and whose backend
/// never reads `StartRequest.context` at all. Against a real daemon and a
/// real Docker Engine, a mixed actor → execute → actor workflow must journal
/// a `context.compiled` snapshot for its two actor stages and **none** for
/// the execute stage in between.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_execute_stage_launch_is_never_compiled() {
    require_docker!();
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (_repo, _head) = support::scaffold_solo_estate(&estate, "solo");

    let workflow_dir = estate.join(".sergeant/workflows/mixed");
    std::fs::create_dir_all(workflow_dir.join("00-first")).expect("stage dir");
    std::fs::create_dir_all(workflow_dir.join("20-third")).expect("stage dir");
    std::fs::write(
        workflow_dir.join("00-first/CONTEXT.md"),
        "first stage context",
    )
    .expect("context");
    std::fs::write(
        workflow_dir.join("20-third/CONTEXT.md"),
        "third stage context",
    )
    .expect("context");
    std::fs::write(
        workflow_dir.join("workflow.toml"),
        concat!(
            "[workflow]\n",
            "name = \"mixed\"\n",
            "version = \"1\"\n",
            "stages = [\"00-first\", \"10-second\", \"20-third\"]\n",
            "\n",
            "[stage.\"10-second\"]\n",
            "kind = \"execute\"\n",
            "image = \"alpine:3.24\"\n",
            "command = [\"true\"]\n",
            "workdir = \"/estate\"\n",
            "workspace_access = \"read_only\"\n",
            "network = \"none\"\n",
        ),
    )
    .expect("workflow.toml");

    let (registry, fake) = one_fake([
        sergeant_rs::backend::fake::FakeStep::complete_with("first done"),
        sergeant_rs::backend::fake::FakeStep::complete_with("third done"),
    ]);
    let handle = sergeant_rs::daemon::start_with(
        data.path(),
        sergeant_rs::daemon::DaemonConfig {
            backends: std::sync::Arc::new(registry),
            default_backend: Some(FAKE.to_string()),
            claude: None,
            docker: Some(sergeant_rs::backend::docker::DockerConfig::new(data.path())),
            ..sergeant_rs::daemon::DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("client");
    let body = serde_json::json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "intent": "prove an execute stage is never compiled",
        "estate_root": &estate,
        "origin": {"client": "cli", "cwd": &estate},
        "workflow": "mixed",
    });
    let response = client
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&body)
        .send()
        .await
        .expect("request");
    let status = response.status();
    let value: Value = response.json().await.expect("json body");
    assert_eq!(status, 201, "submit rejected: {value}");
    assert_eq!(value["work"]["state"], "completed", "{value}");
    let work_id = value["work"]["id"].as_str().expect("work id").to_string();

    let starts = fake.starts();
    assert_eq!(
        starts.len(),
        2,
        "only the two actor stages reach the fake: {starts:?}"
    );

    let snapshots = events_of(data.path(), &work_id, KIND_CONTEXT_COMPILED);
    let compiled_stage_ids: BTreeSet<String> = snapshots
        .iter()
        .map(|e| {
            e.payload["coordinate"]["stage"]
                .as_str()
                .expect("stage coordinate")
                .to_string()
        })
        .collect();
    assert_eq!(
        compiled_stage_ids,
        BTreeSet::from(["00-first".to_string(), "20-third".to_string()]),
        "the execute stage must never appear among compiled stages: {compiled_stage_ids:?}"
    );

    handle.shutdown().await;
}

/// A two-stage workflow whose stage contexts are exactly the two strings the
/// live tests compare against.
fn write_two_stage_workflow(estate: &Path) {
    let root = estate.join(".sergeant").join("workflows").join("tiny");
    std::fs::create_dir_all(&root).expect("workflow dir");
    std::fs::write(
        root.join("workflow.toml"),
        "[workflow]\nname = \"tiny\"\nversion = \"1\"\nstages = [\"00-first\", \"10-second\"]\n",
    )
    .expect("workflow.toml");
    for (id, context) in [
        ("00-first", "first stage context"),
        ("10-second", "second stage context"),
    ] {
        std::fs::create_dir_all(root.join(id)).expect("stage dir");
        std::fs::write(root.join(id).join("CONTEXT.md"), context).expect("CONTEXT.md");
    }
}

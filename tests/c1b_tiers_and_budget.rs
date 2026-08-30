//! S6 C1b — C1 §2's three tiers and §14's **two hard rendering budgets**,
//! with §21 items 3, 4 and 5.
//!
//! # What each test here is the evidence for
//!
//! | Claim | Test |
//! |---|---|
//! | **item 3, first half** — the Bound budget is HARD: a source far larger than it cannot fill Bound with body text | [`the_bound_budget_is_hard_and_a_huge_source_cannot_fill_bound_with_body_text`] |
//! | §14 — **two** budgets: exhausting one never spends or starves the other | [`bound_and_referenced_are_two_budgets_and_exhausting_one_does_not_spend_the_other`] |
//! | **item 5** + §14's last sentence — an exhausted budget leaves Reachable available and caps no resolution | [`an_exhausted_budget_leaves_reachable_available_and_caps_no_resolution`] |
//! | §14 — the budget bounds the RENDER, not the snapshot: every evidence id survives a zero budget | [`the_budget_bounds_the_render_and_not_the_snapshots_evidence_ids`] |
//! | **item 3, second half** — every Bound unit resolves to real source evidence, and a fabricated coordinate does not | [`every_bound_unit_resolves_to_source_evidence_and_a_fabricated_one_does_not`] |
//! | **item 4** — a coordinate resolves by DIRECT LOOKUP: the pinned generation discriminates two rows content cannot, and the search surface never sees the query | [`a_coordinate_resolves_by_direct_lookup_not_by_rediscovery`] |
//! | §5 step 9 — Bound evidence that does not fit is emitted as Referenced *remainder*, attribution intact | [`bound_evidence_that_does_not_fit_the_budget_becomes_referenced_remainder`] |
//! | §15 line 15 — the snapshot records both budgets and what each tier spent | [`the_snapshot_records_both_budgets_and_what_each_tier_spent`] |
//! | §21 item 9 is C1c's, so external prose does not reach a prompt from here | [`external_evidence_renders_as_a_coordinate_and_never_as_body_text`] |
//!
//! Every test runs the real compiler over a real Atlas built by the ordinary
//! `record_scan` path. The fixture is deliberately hostile to the budget: the
//! Work's own overlay holds a 64 KiB unit, which is eight times
//! [`RenderBudget::BOUND_BYTES`] and is contributed **deterministically** by
//! §5 step 2 — so every "it did not fit" assertion below is about evidence
//! that really was selected, not about evidence that happened to be missing.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use sergeant_rs::domain::source::{AuthorityClass, SourceKind};
use sergeant_rs::domain::workflow::{StageDefinition, StageKind, StageRecord, StageStatus};
use sergeant_rs::runtime::atlas::db::{AtlasDb, LexicalQuery, SourceSelector};
use sergeant_rs::runtime::atlas::overlay::overlay_source_name;
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{EDGE_IMPORT, ScannedEdge, ScannedSyntax};
use sergeant_rs::runtime::atlas::semantic::SemanticRequest;
use sergeant_rs::runtime::context::{
    CompileRequest, ContextSnapshot, EvidenceCoordinate, EvidenceUnit, RenderBudget, ResearchStep,
    SourceEvidence, Tier, compile, resolve,
};
use sergeant_rs::runtime::journal::Journal;

mod support;
// `unit`/`file`/`scan` fixture builders live in `tests/support` (R2 — F-SI-01:
// this file and `tests/c1a_compiled_context.rs` had built byte-identical
// copies of the same three functions).
use support::{file, scan, unit};

// ===================================================================
// Fixture
// ===================================================================

const WORK_ID: &str = "01C1BWORK";
const REPOSITORY: &str = "demo-repo";
const INTENT: &str = "tighten the retention policy";
const AUTHORED: &str = "authored stage procedure";

/// The small Work-changed resource. It fits the Bound budget with room to
/// spare, and it is what makes every "the budget is hard" test non-vacuous:
/// the renderer demonstrably *does* render bodies.
const PLAN_BODY: &str = "The retention policy this Work changes, in one short paragraph.";

/// A marker no other fixture text contains, planted at the front of the huge
/// body so its arrival in a prompt is unmistakable.
const HUGE_MARKER: &str = "HUGE-BODY-MARKER";

/// Two rows that differ by exactly one word, at the **same relative path and
/// the same ordinal**, in two different generations. Nothing but the pinned
/// generation can tell them apart — which is what makes item 4's test a test
/// of a keyed lookup rather than of a lucky corpus.
const TWIN_BASE: &str = "Chandelier orbit memorandum, BASE generation.";
const TWIN_OVERLAY: &str = "Chandelier orbit memorandum, OVERLAY generation.";
const TWIN_PATH: &str = "docs/twin.md";

/// 64 KiB — eight times [`RenderBudget::BOUND_BYTES`], and full of the
/// intent's own terms so nothing about its selection is accidental.
fn huge_body() -> String {
    let mut body = String::from(HUGE_MARKER);
    while body.len() < 64 * 1024 {
        body.push_str(" retention policy retention policy retention policy");
    }
    body
}

struct Fixture {
    _data: TempDir,
    db: AtlasDb,
}

/// The Work's base repository and the Work's own overlay over it.
///
/// The overlay carries the three units §5 step 2 contributes
/// deterministically: the small plan, the 64 KiB one, and the twin.
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
            file(TWIN_PATH, vec![unit(0, "Twin", TWIN_BASE)]),
        ],
    );
    record_scan(&mut db, &mut journal, &base, None).expect("record base");

    let overlay = scan(
        &overlay_source_name(WORK_ID, REPOSITORY),
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        vec![
            file("notes/plan.md", vec![unit(0, "Plan", PLAN_BODY)]),
            file("notes/huge.md", vec![unit(0, "Huge", &huge_body())]),
            file(TWIN_PATH, vec![unit(0, "Twin", TWIN_OVERLAY)]),
        ],
    );
    record_scan(&mut db, &mut journal, &overlay, None).expect("record overlay");

    Fixture { _data: data, db }
}

fn stage() -> StageDefinition {
    StageDefinition {
        id: "10-implement".to_string(),
        context: AUTHORED.to_string(),
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

/// Compile this Work's world under `budget`.
fn compiled(atlas: &AtlasDb, budget: RenderBudget) -> ContextSnapshot {
    let stage = stage();
    let bindings = bindings();
    let prior = prior_stages();
    compile(
        Some(atlas),
        &CompileRequest {
            estate_root: Some(Path::new("/estates/demo")),
            work_id: WORK_ID,
            intent: INTENT,
            stage: &stage,
            stage_index: 1,
            attempt: 1,
            execution_id: "01EXECUTION",
            journal_watermark: 42,
            bindings: &bindings,
            prior_stages: &prior,
            profile: Some("standard"),
            budget,
        },
    )
}

/// The bytes one rendered tier section actually occupies, **counted off the
/// rendered text** rather than read from the snapshot's own report.
///
/// This is what makes the budget assertions below checks rather than
/// tautologies: the snapshot says what it spent, and this says what the
/// prompt actually carries, and the tests require the two to agree.
fn section_bytes(rendered: &str, tier: &str) -> usize {
    let marker = format!("\n{tier}\n");
    let Some(start) = rendered.find(&marker) else {
        return 0;
    };
    let rest = &rendered[start + marker.len()..];
    let end = ["\nBOUND\n", "\nREFERENCED\n", "\nREACHABLE\n"]
        .iter()
        .filter_map(|m| rest.find(m))
        .min()
        .unwrap_or(rest.len());
    rest[..end].len()
}

/// Every evidence unit of a snapshot, both tiers, in one list.
fn all_units(snapshot: &ContextSnapshot) -> Vec<&EvidenceUnit> {
    snapshot
        .bound
        .iter()
        .chain(snapshot.referenced.iter())
        .collect()
}

/// The unit whose Atlas coordinate names `relative_path`.
fn unit_at<'a>(snapshot: &'a ContextSnapshot, relative_path: &str) -> &'a EvidenceUnit {
    all_units(snapshot)
        .into_iter()
        .find(|unit| match &unit.coordinate {
            EvidenceCoordinate::Atlas {
                relative_path: p, ..
            } => p == relative_path,
            _ => false,
        })
        .unwrap_or_else(|| panic!("{relative_path} is not in the compiled snapshot at all"))
}

// ============================================================ item 3, budget

/// **§14's first rule, and §21 item 3's first half.** *"Use a **hard**
/// automatic-render budget."*
///
/// The Work's overlay holds a 64 KiB unit — eight times the Bound budget —
/// contributed by §5 step 2, so it is genuinely selected evidence. Four
/// assertions, and each one is load-bearing:
///
/// 1. the huge unit really is in the compiled snapshot (without this the
///    test would pass on a world where nothing large was ever selected);
/// 2. its body never reaches the prompt — not truncated in, not summarized
///    in, not there at all;
/// 3. the rendered Bound section, **measured off the rendered text**, is
///    within the budget, and the snapshot's own `bound_spent` equals that
///    measurement;
/// 4. the small body *is* rendered, so this is a bound on a renderer that
///    demonstrably renders bodies, not a renderer that renders none.
#[test]
fn the_bound_budget_is_hard_and_a_huge_source_cannot_fill_bound_with_body_text() {
    let fixture = fixture();
    let snapshot = compiled(&fixture.db, RenderBudget::DEFAULT);
    let rendered = snapshot.render_onto(AUTHORED);

    let huge = unit_at(&snapshot, "notes/huge.md");
    assert_eq!(
        huge.step,
        ResearchStep::WorkBindings,
        "the 64 KiB unit is one of the Work's exact changed resources, selected by §5 step 2"
    );
    assert!(
        huge.excerpt.is_none(),
        "a unit eight times the Bound budget was given an excerpt"
    );
    assert!(
        !rendered.contains(HUGE_MARKER),
        "body text from a 64 KiB source reached the prompt"
    );

    let measured = section_bytes(&rendered, "BOUND") as u64;
    let report = snapshot
        .budget
        .expect("a compiled snapshot carries §15's budget");
    assert!(
        measured <= report.budget.bound_bytes,
        "the BOUND section rendered {measured} bytes against a HARD budget of {}",
        report.budget.bound_bytes
    );
    assert_eq!(
        measured, report.bound_spent,
        "the snapshot's own bound_spent must be the bytes the prompt actually carries"
    );

    assert!(
        rendered.contains(PLAN_BODY),
        "the renderer renders bodies — the bound above is a bound, not an absence: {rendered}"
    );
}

/// **§14's second rule.** *"Referenced coordinates have a **small separate**
/// rendering budget because a pointer is far cheaper than loading the
/// resource."*
///
/// Separate is proved in both directions, because one direction alone is
/// compatible with a single shared budget spent in a fixed order:
///
/// - Bound at zero, Referenced at its default → **nothing** Bound renders and
///   Referenced still renders its pointers;
/// - Bound at its default, Referenced at zero → Bound still renders its
///   bodies and **nothing** Referenced does.
///
/// One shared number cannot produce both answers over the same world.
#[test]
fn bound_and_referenced_are_two_budgets_and_exhausting_one_does_not_spend_the_other() {
    let fixture = fixture();

    let no_bound = compiled(
        &fixture.db,
        RenderBudget {
            bound_bytes: 0,
            referenced_bytes: RenderBudget::REFERENCED_BYTES,
        },
    );
    let report = no_bound.budget.expect("budget");
    assert_eq!(report.bound_spent, 0, "the Bound budget was zero");
    assert!(
        report.referenced_spent > 0,
        "an exhausted Bound budget must not starve the SEPARATE Referenced one"
    );
    let rendered = no_bound.render_onto(AUTHORED);
    assert_eq!(section_bytes(&rendered, "BOUND"), 0, "{rendered}");
    assert!(section_bytes(&rendered, "REFERENCED") > 0, "{rendered}");

    let no_referenced = compiled(
        &fixture.db,
        RenderBudget {
            bound_bytes: RenderBudget::BOUND_BYTES,
            referenced_bytes: 0,
        },
    );
    let report = no_referenced.budget.expect("budget");
    assert!(
        report.bound_spent > 0,
        "an exhausted Referenced budget must not starve the SEPARATE Bound one"
    );
    assert_eq!(report.referenced_spent, 0, "the Referenced budget was zero");
    let rendered = no_referenced.render_onto(AUTHORED);
    assert!(rendered.contains(PLAN_BODY), "{rendered}");
    assert_eq!(section_bytes(&rendered, "REFERENCED"), 0, "{rendered}");
}

// ============================================================ items 4 and 5

/// **§21 item 5, and §14's third rule.** *"Reachable map/search/query remains
/// available"*; *"the budget is for automatic prompt material, **not a ban on
/// the actor resolving Reachable/Referenced evidence when needed**"*.
///
/// Both budgets are zero — the most hostile state there is — and:
///
/// - §2's Reachable descriptor still renders, with all four managed verbs. A
///   budget that swallowed the surface the actor escapes through would make
///   item 5 depend on having budget left, which is exactly backwards;
/// - a coordinate from that same snapshot still resolves to its **full** 64
///   KiB body — six times the whole default budget, through an API that
///   cannot see a budget at all.
#[test]
fn an_exhausted_budget_leaves_reachable_available_and_caps_no_resolution() {
    let fixture = fixture();
    let snapshot = compiled(
        &fixture.db,
        RenderBudget {
            bound_bytes: 0,
            referenced_bytes: 0,
        },
    );
    let rendered = snapshot.render_onto(AUTHORED);

    assert!(
        all_units(&snapshot).iter().all(|unit| !unit.rendered),
        "a zero budget renders no evidence at all"
    );
    let reachable = snapshot.reachable.as_ref().expect("§2's third tier");
    for verb in ["map", "search", "related", "query"] {
        assert!(
            reachable.capabilities.contains(&verb),
            "Reachable lost {verb:?} to an exhausted budget: {reachable:?}"
        );
    }
    assert!(
        rendered.contains("REACHABLE"),
        "the actor cannot be told about a surface that was not rendered: {rendered}"
    );

    let huge = unit_at(&snapshot, "notes/huge.md");
    let resolved = resolve(&fixture.db, &huge.coordinate)
        .expect("read")
        .expect("a pinned coordinate resolves");
    let SourceEvidence::Unit { text, .. } = resolved else {
        panic!("an Atlas coordinate resolves to a stored unit: {resolved:?}");
    };
    assert!(text.starts_with(HUGE_MARKER), "the wrong row came back");
    assert!(
        text.len() as u64 > RenderBudget::BOUND_BYTES + RenderBudget::REFERENCED_BYTES,
        "resolution returned {} bytes; if the budget bounded resolution it could not have",
        text.len()
    );
}

/// §14 bounds *"automatic prompt material"* — not the record.
///
/// The same world compiled at a zero budget and at the default budget selects
/// the **same evidence**: identical coordinates, identical §5 attributions.
/// Only `rendered` (and the tier a demoted unit lands in) differs. A budget
/// implemented by dropping evidence would pass every render assertion above
/// and fail this one, and would quietly make Referenced unresolvable — which
/// is the failure §14's last sentence exists to forbid.
#[test]
fn the_budget_bounds_the_render_and_not_the_snapshots_evidence_ids() {
    let fixture = fixture();
    let full = compiled(&fixture.db, RenderBudget::DEFAULT);
    let starved = compiled(
        &fixture.db,
        RenderBudget {
            bound_bytes: 0,
            referenced_bytes: 0,
        },
    );

    let keys = |snapshot: &ContextSnapshot| -> HashSet<(usize, String)> {
        all_units(snapshot)
            .into_iter()
            .map(|unit| (unit.step.number(), unit.coordinate.dedup_key()))
            .collect()
    };
    assert!(!keys(&full).is_empty(), "the fixture compiled something");
    assert_eq!(
        keys(&full),
        keys(&starved),
        "a zero budget dropped evidence ids instead of only dropping RENDERING"
    );
    assert!(
        full.bound.iter().any(|unit| unit.rendered),
        "the default budget renders something, so the comparison above is not two empties"
    );
}

/// **§21 item 3's second half.** *"...and every unit resolves to source
/// evidence."*
///
/// Every Bound unit is resolved, and an Atlas one must come back as the exact
/// stored row whose text is **byte-identical** to the excerpt that was
/// rendered — a rendered unit that cannot be traced to its source is the same
/// defect class as a note asserting what the code does not do.
///
/// The violation half is what makes it a test: two fabricated coordinates —
/// a path that no generation holds, and a real path at an ordinal that does
/// not exist — resolve to `None`. A resolver that answered `Some` for
/// anything plausible would pass the first half and fail here.
#[test]
fn every_bound_unit_resolves_to_source_evidence_and_a_fabricated_one_does_not() {
    let fixture = fixture();
    let snapshot = compiled(&fixture.db, RenderBudget::DEFAULT);

    let mut atlas_units = 0usize;
    for unit in &snapshot.bound {
        let resolved = resolve(&fixture.db, &unit.coordinate)
            .expect("read")
            .unwrap_or_else(|| {
                panic!(
                    "a Bound unit resolves to no source evidence: {:?}",
                    unit.coordinate
                )
            });
        if let SourceEvidence::Unit { text, .. } = &resolved {
            atlas_units += 1;
            if let Some(excerpt) = &unit.excerpt {
                assert_eq!(
                    excerpt, text,
                    "the rendered excerpt is not the stored row it claims to be"
                );
            }
        }
    }
    assert!(
        atlas_units > 0,
        "every Bound unit was a Work record: nothing was resolved against the store, \
         which would make the loop above vacuous"
    );

    let EvidenceCoordinate::Atlas {
        source_name,
        generation_id,
        content_key,
        relative_path,
        ..
    } = unit_at(&snapshot, "notes/plan.md").coordinate.clone()
    else {
        panic!("notes/plan.md is an Atlas coordinate");
    };
    let fabricated_path = EvidenceCoordinate::Atlas {
        source_name: source_name.clone(),
        generation_id: generation_id.clone(),
        content_key: content_key.clone(),
        relative_path: "notes/no-such-file.md".to_string(),
        unit_key: None,
        ordinal: Some(0),
    };
    assert_eq!(
        resolve(&fixture.db, &fabricated_path).expect("read"),
        None,
        "a coordinate naming no stored row must not resolve"
    );
    let fabricated_ordinal = EvidenceCoordinate::Atlas {
        source_name,
        generation_id,
        content_key,
        relative_path,
        unit_key: None,
        ordinal: Some(9_999),
    };
    assert_eq!(
        resolve(&fixture.db, &fabricated_ordinal).expect("read"),
        None,
        "a real resource at an ordinal it does not have must not resolve"
    );
}

/// **§21 item 4.** *"Referenced coordinates resolve **without broad
/// rediscovery**"* — §2: *"the actor can resolve them directly **without
/// broad search**"*.
///
/// Two ways of showing the resolution is a keyed lookup and not a search:
///
/// 1. **The pinned generation is what discriminates.** `docs/twin.md#0`
///    exists in both the base and the overlay generation, at the same path
///    and the same ordinal, with bodies differing by a single word. Nothing
///    but the `generation_id` in the coordinate can pick between them, and
///    each coordinate returns its own row. A content-addressed or search-
///    backed resolver could not — it has no basis to prefer either.
/// 2. **The search surface never sees this query.** The retrieval the
///    compiler itself ran, over the same admissible world, does not return
///    `docs/twin.md` at all — its terms are nowhere in the intent. The same
///    coordinate resolves anyway, so resolution plainly does not go through
///    retrieval.
///
/// The Referenced coordinate resolved at the end is one the compiler actually
/// emitted, so this is about the snapshot's own pointers, not about a
/// coordinate the test invented.
#[test]
fn a_coordinate_resolves_by_direct_lookup_not_by_rediscovery() {
    let fixture = fixture();
    let base = fixture
        .db
        .confirmed_generation(REPOSITORY)
        .expect("read")
        .expect("the base generation is confirmed");
    let overlay_name = overlay_source_name(WORK_ID, REPOSITORY);
    let overlay = fixture
        .db
        .confirmed_generation(&overlay_name)
        .expect("read")
        .expect("the overlay generation is confirmed");

    let twin =
        |source_name: &str, generation_id: &str, content_key: &str| EvidenceCoordinate::Atlas {
            source_name: source_name.to_string(),
            generation_id: generation_id.to_string(),
            content_key: content_key.to_string(),
            relative_path: TWIN_PATH.to_string(),
            unit_key: None,
            ordinal: Some(0),
        };
    let text_of = |coordinate: &EvidenceCoordinate| -> String {
        match resolve(&fixture.db, coordinate).expect("read") {
            Some(SourceEvidence::Unit { text, .. }) => text,
            other => panic!("{coordinate:?} did not resolve to a unit: {other:?}"),
        }
    };
    assert_eq!(
        text_of(&twin(REPOSITORY, &base.id, &base.content_key)),
        TWIN_BASE,
        "the BASE generation's coordinate returned some other generation's row"
    );
    assert_eq!(
        text_of(&twin(&overlay_name, &overlay.id, &overlay.content_key)),
        TWIN_OVERLAY,
        "the OVERLAY generation's coordinate returned some other generation's row"
    );

    // The search surface, over the same admissible world, does not reach the
    // twin at all — and resolution did not need it to.
    let filter = sergeant_rs::runtime::atlas::db::Admissibility {
        source: SourceSelector::WorkBase {
            work_id: WORK_ID.to_string(),
            repository: REPOSITORY.to_string(),
        },
        kind: None,
        authority: None,
    };
    let answer = fixture
        .db
        .lexical_search(&LexicalQuery {
            text: INTENT,
            filter: &filter,
            family: None,
            limit: 32,
            semantic: SemanticRequest::Suppressed,
        })
        .expect("search");
    assert!(
        !answer
            .hits
            .iter()
            .any(|hit| hit.coordinate.relative_path() == TWIN_PATH),
        "the fixture is wrong: retrieval DOES find the twin, so resolving it proves nothing"
    );

    // And the same holds for a pointer the compiler itself emitted.
    let snapshot = compiled(
        &fixture.db,
        RenderBudget {
            bound_bytes: 0,
            referenced_bytes: RenderBudget::REFERENCED_BYTES,
        },
    );
    let pointer = snapshot
        .referenced
        .iter()
        .find(|unit| matches!(&unit.coordinate, EvidenceCoordinate::Atlas { .. }))
        .expect("the starved compilation emitted Atlas pointers");
    assert!(
        resolve(&fixture.db, &pointer.coordinate)
            .expect("read")
            .is_some(),
        "a Referenced pointer the compiler emitted did not resolve: {:?}",
        pointer.coordinate
    );
}

// ============================================================ §5 step 9

/// **§5 step 9** — *"pack Bound; **emit useful remainder as Referenced**"*.
///
/// The same coordinate, over the same world, under two budgets: Bound with a
/// rendered body under the default, and Referenced-with-no-body under a
/// budget too small for it. Its §5 attribution (step 2) survives the
/// demotion — the remainder is the same evidence, moved, not re-derived — and
/// step 9's own record says what it did, which is where §18's *"visible"*
/// degradation lands for the budget.
#[test]
fn bound_evidence_that_does_not_fit_the_budget_becomes_referenced_remainder() {
    let fixture = fixture();

    let full = compiled(&fixture.db, RenderBudget::DEFAULT);
    let plan = unit_at(&full, "notes/plan.md");
    assert_eq!(plan.tier, Tier::Bound);
    assert_eq!(plan.step, ResearchStep::WorkBindings);
    assert_eq!(plan.excerpt.as_deref(), Some(PLAN_BODY));

    // 40 bytes holds no chunk in this world at all.
    let tight = compiled(
        &fixture.db,
        RenderBudget {
            bound_bytes: 40,
            referenced_bytes: RenderBudget::REFERENCED_BYTES,
        },
    );
    let plan = unit_at(&tight, "notes/plan.md");
    assert_eq!(
        plan.tier,
        Tier::Referenced,
        "Bound evidence that did not fit must be emitted as Referenced remainder"
    );
    assert_eq!(
        plan.step,
        ResearchStep::WorkBindings,
        "the demotion must not rewrite which §5 step selected the evidence"
    );
    assert_eq!(plan.excerpt, None, "a remainder is a pointer, not a body");
    assert!(
        plan.rendered,
        "the remainder fit the SEPARATE Referenced budget and must be rendered there"
    );

    let pack = tight
        .plan
        .iter()
        .find(|record| record.step == ResearchStep::Pack)
        .expect("§5 step 9 ran");
    let note = pack.note.as_deref().unwrap_or_default();
    assert!(
        note.contains("Referenced remainder"),
        "step 9 must record what packing did: {note:?}"
    );
}

/// §15's *"budget + rendered size"* line, as the snapshot carries it.
///
/// Both budgets are named, both spends are named, each spend is within its
/// own budget, and the Referenced budget is the **smaller** of the two —
/// §14's *"small separate"*, checkable from the journal alone.
#[test]
fn the_snapshot_records_both_budgets_and_what_each_tier_spent() {
    let fixture = fixture();
    let snapshot = compiled(&fixture.db, RenderBudget::DEFAULT);
    let json = snapshot.json();
    let budget = &json["budget"];

    assert_eq!(budget["unit"], "bytes", "{json}");
    let bound_bytes = budget["bound_bytes"].as_u64().expect("bound_bytes");
    let referenced_bytes = budget["referenced_bytes"]
        .as_u64()
        .expect("referenced_bytes");
    let bound_spent = budget["bound_spent"].as_u64().expect("bound_spent");
    let referenced_spent = budget["referenced_spent"]
        .as_u64()
        .expect("referenced_spent");

    assert!(referenced_bytes < bound_bytes, "{budget}");
    assert!(bound_spent <= bound_bytes, "{budget}");
    assert!(referenced_spent <= referenced_bytes, "{budget}");
    assert!(bound_spent > 0 && referenced_spent > 0, "{budget}");
}

// ============================================================ item 9 is C1c's

/// External evidence renders as a coordinate and never as body text.
///
/// §21 item 9 — *"external evidence is visibly external and cannot alter
/// instruction hierarchy"* — is C1c's, and until it exists external prose
/// does not enter an actor's prompt from here (**J5**: item 9 is this
/// contract's own, and it is not yet met; under-delivering a tier is
/// recoverable, shipping unlabeled external prose is not).
///
/// Non-vacuity is the second half: the identical body under an
/// estate-authority source *is* rendered by the same compilation, so this
/// pins the authority class as the thing that made the difference, not the
/// text and not an empty render.
#[test]
fn external_evidence_renders_as_a_coordinate_and_never_as_body_text() {
    const SHARED_BODY: &str = "The retention policy, in one paragraph of prose.";
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    let estate = scan(
        "estate-notes",
        SourceKind::LocalKnowledge,
        AuthorityClass::EstateReadonly,
        vec![file("estate.md", vec![unit(0, "Estate", SHARED_BODY)])],
    );
    record_scan(&mut db, &mut journal, &estate, None).expect("record estate source");
    let external = scan(
        "vendor-docs",
        SourceKind::ExternalGit,
        AuthorityClass::External,
        vec![file("vendor.md", vec![unit(0, "Vendor", SHARED_BODY)])],
    );
    record_scan(&mut db, &mut journal, &external, None).expect("record external source");

    // No binding, so the admissibility filter is the whole admitted world —
    // which is the only shape in which external evidence reaches this
    // compiler at all today (a `WorkBase` selector names one repository and
    // its overlay).
    let stage = stage();
    let snapshot = compile(
        Some(&db),
        &CompileRequest {
            estate_root: Some(Path::new("/estates/demo")),
            work_id: WORK_ID,
            intent: INTENT,
            stage: &stage,
            stage_index: 1,
            attempt: 1,
            execution_id: "01EXECUTION",
            journal_watermark: 42,
            bindings: &[],
            prior_stages: &[],
            profile: None,
            budget: RenderBudget::DEFAULT,
        },
    );

    let vendor = unit_at(&snapshot, "vendor.md");
    assert_eq!(
        vendor.excerpt, None,
        "external prose was rendered as a body"
    );
    let estate_unit = unit_at(&snapshot, "estate.md");
    assert_eq!(
        estate_unit.excerpt.as_deref(),
        Some(SHARED_BODY),
        "the identical body under an estate source IS rendered — the authority class is \
         what made the difference"
    );

    let rendered = snapshot.render_onto(AUTHORED);
    assert!(rendered.contains("vendor.md"), "{rendered}");
    let pack = snapshot
        .plan
        .iter()
        .find(|record| record.step == ResearchStep::Pack)
        .expect("§5 step 9 ran");
    assert!(
        pack.note.as_deref().unwrap_or_default().contains("item 9"),
        "packing must say why it withheld the body: {:?}",
        pack.note
    );
}

// ==================================================================== fixes

/// **F-IN-01.** `AtlasDb::resolve_relationship`'s edge lookup used to filter
/// only on `(generation_id, edge_kind, relative_path, target)` — no
/// `ordinal`, no `ORDER BY`, no `LIMIT` — so two edges that legitimately
/// share every one of those fields (the same file importing the same target
/// twice) were indistinguishable to the query, and `rows.next()` returned
/// whichever DuckDB happened to return first regardless of which coordinate
/// asked. Two edges are planted here sharing kind/from/to and differing only
/// in `ordinal`; each coordinate's own pinned ordinal must come back, not an
/// arbitrary one of the two.
#[test]
fn resolve_relationship_discriminates_sibling_edges_by_ordinal() {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    let mut code = file("src/lib.rs", vec![unit(0, "lib", "fn main() {}")]);
    code.syntax = Some(ScannedSyntax {
        language: "rust",
        extractor: "syntax-rust/v1".to_string(),
        syntax_key: "syntax-key/rust/src/lib.rs".to_string(),
        symbols: Vec::new(),
        edges: vec![
            ScannedEdge {
                ordinal: 0,
                kind: EDGE_IMPORT,
                target: "crate::shared".to_string(),
                byte_start: 0,
                byte_end: 1,
            },
            ScannedEdge {
                ordinal: 7,
                kind: EDGE_IMPORT,
                target: "crate::shared".to_string(),
                byte_start: 40,
                byte_end: 41,
            },
        ],
    });
    let base = scan(
        REPOSITORY,
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        vec![code],
    );
    record_scan(&mut db, &mut journal, &base, None).expect("record base");
    let generation_id = db
        .confirmed_generation(REPOSITORY)
        .expect("read")
        .expect("the base generation is confirmed")
        .id;

    let asked_for_0 = db
        .resolve_relationship(
            &generation_id,
            EDGE_IMPORT,
            "src/lib.rs",
            "crate::shared",
            Some(0),
        )
        .expect("read")
        .expect("the ordinal-0 edge resolves");
    assert_eq!(
        asked_for_0.ordinal,
        Some(0),
        "asked for ordinal 0, got the other sibling edge back: {asked_for_0:?}"
    );

    let asked_for_7 = db
        .resolve_relationship(
            &generation_id,
            EDGE_IMPORT,
            "src/lib.rs",
            "crate::shared",
            Some(7),
        )
        .expect("read")
        .expect("the ordinal-7 edge resolves");
    assert_eq!(
        asked_for_7.ordinal,
        Some(7),
        "asked for ordinal 7, got the other sibling edge back: {asked_for_7:?}"
    );
}

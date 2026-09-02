//! S6 C1d — C1 §21 items **11, 12 and 14**: attribution, nesting and audit.
//!
//! # What each test here is the evidence for
//!
//! | Claim | Test |
//! |---|---|
//! | **item 11** — §16's seven kinds are declared verbatim, in §16's own order, and every one is on the stream's published vocabulary | [`the_seven_event_kinds_are_section_16s_own_list_in_section_16s_own_order`] |
//! | **item 11** — a managed context query is attributed to the **execution**, not merely to the Work: two launches of one stage are told apart, and `sgt search --work`'s Work-scoped read still has no execution | [`a_managed_context_query_is_attributed_to_the_execution_not_merely_the_work`] |
//! | **item 11** — the audit record is **raw evidence**: coordinates, query identity and A2 §15's stated status, and no scalar anywhere | [`the_audit_record_is_raw_evidence_and_carries_no_sharpness_scalar`] |
//! | **item 11** — a search that landed on a narrower §18 rung than it asked for records that, and one that did not does not | [`a_search_fallback_is_recorded_only_when_the_answer_actually_degraded`] |
//! | **item 11** — a degraded compilation records no audit event at all | [`a_degraded_compilation_records_no_audit_event`] |
//! | **item 12, rule 5** — a causal child's parent causation is **Referenced**, as a pointer, and its absence for an ordinary Work is the observable difference | [`a_causal_childs_parent_causation_is_referenced_as_a_pointer`] |
//! | **item 12, rule 2, structurally** — a container cannot acquire an actor procedure of its own; the identical file one directory down loads fine | [`a_container_cannot_acquire_an_actor_procedure_and_a_leaf_can`] |
//! | **item 14** — a rendered claim fragment traces through the journaled artifact to its exact source evidence, and every link of §19's chain survives | [`a_rendered_claim_fragment_traces_through_the_artifact_to_its_source_evidence`] |
//! | **item 14** — §19's `page+bbox` arm is absent **by owner ruling**, present and null rather than omitted | [`section_19s_page_and_bbox_arm_is_absent_by_owner_ruling`] |
//! | **item 12, rules 1 and 2, live** — each nested leaf actor gets its own snapshot under its composed id, and the container that closes on them gets none | [`each_nested_leaf_actor_gets_its_own_snapshot_and_the_container_gets_none`] |
//! | **item 12, rules 3 and 4, live and adversarially** — a causal child gets its own Work/source/context binding and inherits no byte of the parent's prompt, while rule 5's pointer still reaches it | [`a_child_work_inherits_no_parent_prompt_and_gets_its_own_binding`] |
//!
//! # What this wave deliberately did **not** build
//!
//! **No claim graph** (§19: *"A full first-class claim graph is **not
//! required** in Sprint 3; preserving exact evidence coordinates in
//! snapshots/artifacts is the **enabling invariant**"*). Item 14's test walks
//! §19's chain by hand, from a rendered fragment to the journaled snapshot to
//! the resolved source row, to show a later wave *could* build one — and
//! builds nothing that stores or indexes claims.
//!
//! **No self-tuning and no sharpness score** (§16: *"Record raw evidence
//! rather than a magic 'sharpness score'"*, *"without self-tuning during
//! Sprint 3"*; §20: *learned context policy/live self-tuning*, *universal
//! scalar sharpness score*). Nothing in this crate reads a `context.*` audit
//! event, and [`the_audit_record_is_raw_evidence_and_carries_no_sharpness_scalar`]
//! is the tripwire on the payload shape.
//!
//! **Three of §16's seven kinds are declared and emitted by nothing**, each
//! with the reason on its own constant: `context.reference_resolved` (no
//! surface gives an *execution* a resolve verb yet, and journaling the
//! compiler's own packing read under it would be a false entry),
//! `context.scope_expansion_requested` (§20 forbids automatic expansion and
//! no surface accepts §10's asked-for one), and
//! `context.contradiction_observed` (§9's detection does not exist, and
//! scoring disagreement is the synthesized consensus §9 forbids).

/// **S6 D1 — A2 §2 stage 1's estate coordinate.** This suite is
/// single-estate: every generation it records is bound to this one root and
/// every filter it builds is admitted from it. The cross-estate case — two
/// estates on one host daemon, which is where the axis actually earns its
/// keep — is `tests/d1_estate_isolation.rs`, deliberately not folded in
/// here, because a suite that never crosses estates cannot notice an estate
/// filter that does nothing (that is exactly how the leak survived: this
/// file's ancestors all passed).
#[allow(dead_code)]
const D1_ESTATE: &str = "/estates/demo";

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::domain::workflow::{
    CONTEXT_AUDIT_KINDS, KIND_CONTEXT_BOUND, KIND_CONTEXT_COMPILED, KIND_CONTEXT_QUERY,
    KIND_CONTEXT_SEARCH_FALLBACK, StageDefinition, StageKind, StageRecord, StageStatus,
    WorkflowDefinition, WorkflowError,
};
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::overlay::overlay_source_name;
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{ScannedFile, ScannedUnit};
use sergeant_rs::runtime::atlas::semantic::SemanticStatus;
use sergeant_rs::runtime::atlas::trace::Attribution;
use sergeant_rs::runtime::context::{
    CompileRequest, ContextSnapshot, DATA_PREFIX, EvidenceCoordinate, ParentCausation,
    RenderBudget, SourceEvidence, Tier, compile, resolve,
};
use sergeant_rs::runtime::journal::Journal;

mod support;
use support::{file, scan, unit};

// ===================================================================
// Fixture
// ===================================================================

const WORK_ID: &str = "01C1DWORK";
const REPOSITORY: &str = "demo-repo";
const INTENT: &str = "trace the retention policy to its evidence";
const EXECUTION: &str = "01EXECUTIONALPHA";

const AUTHORED: &str = "# Stage procedure (authored)\n\n1. Read the retention policy.\n";

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

/// Compile one world for `execution_id`, with `parent` as §17's causation.
fn compiled_for(
    atlas: &AtlasDb,
    execution_id: &str,
    parent: Option<ParentCausation<'_>>,
) -> ContextSnapshot {
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
            execution_id,
            journal_watermark: 42,
            bindings: &bindings,
            prior_stages: &prior,
            profile: Some("standard"),
            parent,
            budget: RenderBudget::DEFAULT,
        },
    )
}

/// The extractor and native coordinate item 14's chain has to carry — a
/// normalized Office excerpt, whose byte span is not its address (A2 §9).
const OFFICE_EXTRACTOR: &str = "office-docx/v3";
const NATIVE: &str = "slide:12/block:3";
const CLAIM_FRAGMENT: &str = "retention defaults to thirty days for archived collections";

/// A world with an estate base generation and this Work's overlay, the
/// overlay carrying one normalized Office unit (the fragment item 14 traces)
/// and one ordinary Markdown unit beside it.
fn evidence_world() -> (TempDir, AtlasDb) {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    // This fixture host installs no semantic assets — a fact every test in
    // this file asserts on (`NotInstalled`), forced here rather than merely
    // assumed, so a process-wide `SGT_SEMANTIC_MODEL_DIR` pointing at a real
    // model (this wave's own standing test policy) cannot make the fixture's
    // stated assumption false out from under it.
    db.force_semantic_not_installed_for_test();

    let base = scan(
        REPOSITORY,
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        vec![file(
            "docs/architecture.md",
            vec![unit(0, "Architecture", "The daemon owns the journal.")],
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

    let deck = ScannedFile {
        relative_path: "decks/retention.pptx".to_string(),
        content_hash: "hash/decks/retention.pptx".to_string(),
        content_digest: "hash/decks/retention.pptx".to_string(),
        extractor: OFFICE_EXTRACTOR.to_string(),
        local_key: "key/decks/retention.pptx".to_string(),
        byte_len: 4096,
        mtime_millis: None,
        units: vec![ScannedUnit {
            ordinal: 0,
            kind: UnitKind::Section,
            heading_level: Some(2),
            title: Some("Retention".to_string()),
            byte_start: 1024,
            byte_end: 1024 + CLAIM_FRAGMENT.len() as u64,
            coordinate: Some(NATIVE.to_string()),
            text: CLAIM_FRAGMENT.to_string(),
        }],
        syntax: None,
        parent: None,
    };
    let overlay = scan(
        &overlay_source_name(WORK_ID, REPOSITORY),
        SourceKind::EstateGit,
        AuthorityClass::EstateMutable,
        vec![
            deck,
            file(
                "notes/plan.md",
                vec![unit(0, "Plan", "The retention policy, in one paragraph.")],
            ),
        ],
    );
    record_scan(
        &mut db,
        &mut journal,
        &overlay,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record overlay");
    (data, db)
}

/// The event payloads a compilation would journal, keyed by kind.
fn audit(snapshot: &ContextSnapshot) -> Vec<(&'static str, Value)> {
    snapshot.audit_events()
}

fn payload_of<'a>(events: &'a [(&'static str, Value)], kind: &str) -> Option<&'a Value> {
    events
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, payload)| payload)
}

/// Every key name anywhere in a JSON value, however deep.
fn keys(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                out.insert(key.clone());
                keys(child, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                keys(item, out);
            }
        }
        _ => {}
    }
}

// ============================================================ item 11

/// **§16's list, verbatim and in order.**
///
/// §16 prints seven kinds in a fenced block. `CONTEXT_AUDIT_KINDS` is that
/// block, and this compares the two rather than trusting that a wave which
/// implemented four of them remembered the other three. A kind renamed,
/// dropped or reordered turns this red.
///
/// The second half is the one that makes the declaration *reachable*: every
/// journaled kind must be on the SSE stream's published vocabulary, or an
/// enumerating client silently drops the frame. `m6_surfaces::t6` enforces
/// that for the whole crate; this states it for §16's seven specifically, so
/// a reader of item 11 sees it here.
#[test]
fn the_seven_event_kinds_are_section_16s_own_list_in_section_16s_own_order() {
    assert_eq!(
        CONTEXT_AUDIT_KINDS,
        [
            "context.bound",
            "context.referenced",
            "context.query",
            "context.reference_resolved",
            "context.search_fallback",
            "context.scope_expansion_requested",
            "context.contradiction_observed",
        ],
        "§16 lists seven kinds in this order; the code must say the same seven words"
    );
    for kind in CONTEXT_AUDIT_KINDS {
        assert!(
            sergeant_rs::api::SSE_EVENT_KINDS.contains(&kind),
            "{kind:?} is declared but is not on the stream's published vocabulary"
        );
    }
}

/// **§21 item 11.** *"actor context queries are attributable **to
/// execution**"*, and §16: *"Managed map/search/related/query calls can be
/// attributed to the **current execution**."*
///
/// The observable claim is that two launches of the *same stage of the same
/// Work* are told apart. So the same world is compiled twice under two
/// execution ids, and the `context.query` audit record — plus A2 §13's own
/// trace inside it — must name each one. A compiler that attributed to the
/// Work alone would produce two identical attributions here and pass nothing.
///
/// The other half is what keeps the field from being filled unconditionally:
/// `sgt search --work <id>` is a human's Work-scoped read with genuinely no
/// execution, and its attribution renders `execution: null`. Two different
/// answers from one enum, which is why §16's attribution got a third variant
/// rather than an `Option` on the existing one.
#[test]
fn a_managed_context_query_is_attributed_to_the_execution_not_merely_the_work() {
    let (_data, db) = evidence_world();

    let first = compiled_for(&db, "01EXECUTIONONE", None);
    let second = compiled_for(&db, "01EXECUTIONTWO", None);

    for (snapshot, execution) in [(&first, "01EXECUTIONONE"), (&second, "01EXECUTIONTWO")] {
        let trace = snapshot
            .retrieval
            .as_ref()
            .expect("steps 6/7 ran a managed search");
        assert_eq!(
            trace.attribution,
            Attribution::Execution {
                work_id: WORK_ID.to_string(),
                repository: REPOSITORY.to_string(),
                execution_id: execution.to_string(),
            },
            "the managed search must be attributed to the execution that issued it"
        );
        let events = audit(snapshot);
        let query = payload_of(&events, KIND_CONTEXT_QUERY).expect("context.query was recorded");
        assert_eq!(
            query["attribution"]["execution"].as_str(),
            Some(execution),
            "§16's own attribution: {query}"
        );
        assert_eq!(
            query["trace"]["attribution"]["execution"].as_str(),
            Some(execution),
            "A2 §13 field 2, on the persisted trace: {query}"
        );
        assert_eq!(query["attribution"]["work"].as_str(), Some(WORK_ID));
        assert_eq!(query["attribution"]["stage"].as_str(), Some("10-implement"));
    }

    // The whole point: the two records differ, and differ in the execution.
    let one = audit(&first);
    let two = audit(&second);
    assert_ne!(
        payload_of(&one, KIND_CONTEXT_QUERY).expect("first")["attribution"]["execution"],
        payload_of(&two, KIND_CONTEXT_QUERY).expect("second")["attribution"]["execution"],
        "two launches of one stage must not carry the same attribution"
    );

    // And the Work-scoped CLI read still has no execution to name — present
    // and null, never a filled-in-by-default id.
    let cli = sergeant_rs::runtime::atlas::trace::SearchTrace {
        attribution: Attribution::Work {
            work_id: WORK_ID.to_string(),
            repository: REPOSITORY.to_string(),
        },
        ..first
            .retrieval
            .clone()
            .expect("a trace to vary one field of")
    };
    assert_eq!(
        cli.json()["attribution"]["execution"],
        Value::Null,
        "an unmanaged-by-an-execution Work search must say so, not borrow an execution"
    );
}

/// **§16's own instruction, as a tripwire.** *"Record **raw evidence** rather
/// than a magic 'sharpness score'"* — which §20 states twice more (*learned
/// context policy/live self-tuning*, *universal scalar sharpness score*).
///
/// Two assertions, and both are about shape rather than intent:
///
/// 1. the record really is raw — every `context.bound` entry carries the §5
///    step that contributed it, its evidence id and its full coordinate, and
///    that set of ids is exactly the snapshot's own Bound set (so the record
///    is the evidence, not a summary of it);
/// 2. no payload of any §16 kind carries a key naming a score, a sharpness,
///    a quality or a sufficiency. That is the shape a "magic scalar" would
///    have to arrive in, and it fails the moment someone adds one.
#[test]
fn the_audit_record_is_raw_evidence_and_carries_no_sharpness_scalar() {
    let (_data, db) = evidence_world();
    let snapshot = compiled_for(&db, EXECUTION, None);
    let events = audit(&snapshot);

    let bound = payload_of(&events, KIND_CONTEXT_BOUND).expect("context.bound was recorded");
    let recorded: BTreeSet<String> = bound["evidence"]
        .as_array()
        .expect("evidence array")
        .iter()
        .map(|e| {
            assert!(
                e["step"].is_number(),
                "every entry names the §5 step that contributed it: {e}"
            );
            assert!(
                e["coordinate"]["shape"].is_string(),
                "every entry carries its full coordinate: {e}"
            );
            e["evidence_id"].as_str().expect("evidence id").to_string()
        })
        .collect();
    let expected: BTreeSet<String> = snapshot
        .bound
        .iter()
        .map(|unit| unit.coordinate.dedup_key())
        .collect();
    assert!(!expected.is_empty(), "the fixture bound nothing to record");
    assert_eq!(
        recorded, expected,
        "the audit record must be the Bound evidence itself, not a digest of it"
    );

    let mut names = BTreeSet::new();
    for (_, payload) in &events {
        keys(payload, &mut names);
    }
    for forbidden in ["sharpness", "score", "quality", "sufficiency", "relevance"] {
        let hit: Vec<&String> = names
            .iter()
            .filter(|name| name.to_ascii_lowercase().contains(forbidden))
            .collect();
        assert!(
            hit.is_empty(),
            "§16 records raw evidence, never a magic scalar; found {hit:?}"
        );
    }
}

/// **§16's `context.search_fallback`, on both sides.**
///
/// The compilation always asks for the full A2 ladder. On a host with no
/// semantic assets the answer comes back on §18's narrower rung
/// (`SemanticStatus::NotInstalled`), and that is recorded with A2 §15's own
/// word — *raw stated status*, not a number about answer quality.
///
/// The non-vacuous half is the other side: the same snapshot with the
/// semantic half reported `Applied` records **no** fallback while still
/// recording the query. A test that only asserted the event exists would
/// pass just as well against a compiler that emitted it unconditionally.
#[test]
fn a_search_fallback_is_recorded_only_when_the_answer_actually_degraded() {
    let (_data, db) = evidence_world();
    let mut snapshot = compiled_for(&db, EXECUTION, None);
    assert_eq!(
        snapshot
            .retrieval
            .as_ref()
            .expect("a managed search ran")
            .semantic,
        SemanticStatus::NotInstalled,
        "this fixture host installs no semantic assets"
    );

    let degraded = audit(&snapshot);
    let fallback = payload_of(&degraded, KIND_CONTEXT_SEARCH_FALLBACK)
        .expect("the narrower rung must be recorded");
    assert_eq!(fallback["semantic_status"].as_str(), Some("not_installed"));
    assert_eq!(fallback["applied"].as_str(), Some("lexical"));
    assert_eq!(
        fallback["attribution"]["execution"].as_str(),
        Some(EXECUTION),
        "the fallback belongs to an execution, like every other §16 record"
    );

    if let Some(trace) = snapshot.retrieval.as_mut() {
        trace.semantic = SemanticStatus::Applied;
    }
    let full = audit(&snapshot);
    assert!(
        payload_of(&full, KIND_CONTEXT_SEARCH_FALLBACK).is_none(),
        "an answer that did not degrade must record no fallback: {full:?}"
    );
    assert!(
        payload_of(&full, KIND_CONTEXT_QUERY).is_some(),
        "the query itself is still recorded either way"
    );
}

/// A compilation that compiled nothing records **no** §16 event.
///
/// §18's degradation is already visible on the `context.compiled` snapshot,
/// which the engine journals either way. Emitting an empty `context.bound`
/// beside it would put a row in the audit record for a binding that never
/// happened — which is precisely the record §16 exists to make trustworthy.
#[test]
fn a_degraded_compilation_records_no_audit_event() {
    let empty = AtlasDb::open_in_memory().expect("atlas");
    let snapshot = compiled_for(&empty, EXECUTION, None);
    assert!(snapshot.degradation.is_some(), "the fixture is degraded");
    assert!(
        snapshot.audit_events().is_empty(),
        "nothing happened, so nothing is recorded"
    );
}

// ============================================================ item 12

/// **§17's fifth rule.** *"parent causation/output **may be Referenced** when
/// needed."*
///
/// The permitted channel, as an observable difference: the same world is
/// compiled twice, once for an ordinary Work and once for a causal child.
/// The child's snapshot carries exactly one extra unit — a **Referenced**
/// `parent_work` coordinate naming the parent Work and the parent execution
/// — and it renders into the prompt as a pointer line under `REFERENCED`.
/// The ordinary Work's snapshot carries none, which is the half that makes
/// this a difference rather than an assertion about a string being present.
///
/// The tier is the contract. §2's Referenced is *"known-relevant exact
/// coordinates **not rendered in full**"*, so the parent reaches the child as
/// a coordinate and nothing else — resolution answers
/// [`SourceEvidence::WorkRecord`], the same as this Work's own stage and
/// binding records, and packing gives it no body.
#[test]
fn a_causal_childs_parent_causation_is_referenced_as_a_pointer() {
    let (_data, db) = evidence_world();

    let ordinary = compiled_for(&db, EXECUTION, None);
    let child = compiled_for(
        &db,
        EXECUTION,
        Some(ParentCausation {
            work_id: "01PARENTWORK",
            execution_id: Some("01PARENTEXEC"),
            state: "completed",
        }),
    );

    let parents_of = |snapshot: &ContextSnapshot| -> Vec<EvidenceCoordinate> {
        snapshot
            .bound
            .iter()
            .chain(snapshot.referenced.iter())
            .filter(|unit| matches!(unit.coordinate, EvidenceCoordinate::ParentWork { .. }))
            .map(|unit| {
                assert_eq!(
                    unit.tier,
                    Tier::Referenced,
                    "§17 permits the parent as Referenced, never Bound: {unit:?}"
                );
                assert!(
                    unit.excerpt.is_none(),
                    "a pointer carries no body: {unit:?}"
                );
                unit.coordinate.clone()
            })
            .collect()
    };

    assert!(
        parents_of(&ordinary).is_empty(),
        "a Work with no causal parent has no parent coordinate"
    );
    assert_eq!(
        parents_of(&child),
        vec![EvidenceCoordinate::ParentWork {
            parent_work_id: "01PARENTWORK".to_string(),
            parent_execution_id: Some("01PARENTEXEC".to_string()),
            state: "completed".to_string(),
        }],
        "the child's world names its parent exactly once"
    );

    // It reaches the prompt, as a pointer under REFERENCED.
    let rendered = child.render_onto(AUTHORED);
    assert!(
        rendered.contains(
            "parent work 01PARENTWORK [completed] — caused by its execution \
             01PARENTEXEC"
        ),
        "the parent pointer never reached the prompt: {rendered}"
    );
    assert!(
        !ordinary.render_onto(AUTHORED).contains("parent work"),
        "an ordinary Work's prompt must not mention a parent it does not have"
    );

    // And it resolves the way a Work record resolves: no store lookup, no
    // body, no pretence that one happened.
    let coordinate = parents_of(&child).remove(0);
    assert_eq!(
        resolve(&db, &coordinate).expect("resolution"),
        Some(SourceEvidence::WorkRecord)
    );
}

/// **§17's second rule, structurally.** *"a container stage has **no** actor
/// snapshot unless it contains an explicit actor leaf."*
///
/// A snapshot is compiled for a [`StageDefinition`] and for nothing else, and
/// a container is not a stage at all (W1-02: *"A container is **not** a
/// stage: it has no `StageDefinition`"*). What this pins is that a container
/// cannot *acquire* one: dropping the identical `CONTEXT.md` into a container
/// directory is refused by name at load time, so "the container itself got an
/// actor snapshot" is not a state this codebase can reach — while the same
/// bytes one directory down, in a leaf, load fine and become that leaf's
/// procedure.
///
/// That pair is the point. An absence assertion would pass against a loader
/// that accepted the file and quietly ignored it; this one fails unless the
/// refusal is real *and* the acceptance is real.
#[test]
fn a_container_cannot_acquire_an_actor_procedure_and_a_leaf_can() {
    const PROCEDURE: &str = "read the code and report";

    let root = TempDir::new().expect("tempdir");
    let package = root.path().join("nested");
    write_package(&package, "nested", &["00-orient", "10-investigate"]);
    write_stage(&package, "00-orient", "orient the work");
    let container = package.join("10-investigate");
    write_package(&container, "10-investigate", &["00-lead"]);
    write_stage(&container, "00-lead", PROCEDURE);

    // The leaf's procedure loads and is that leaf's context.
    let loaded = WorkflowDefinition::load_dir(&package).expect("the nested package loads");
    let leaf = loaded
        .stages
        .iter()
        .find(|s| s.id == "10-investigate/00-lead")
        .expect("the composed leaf id");
    assert_eq!(leaf.context, PROCEDURE);
    assert_eq!(leaf.kind, StageKind::Actor);
    assert!(
        loaded
            .containers
            .iter()
            .any(|c| c.container_id == "10-investigate"),
        "the container is a real, named boundary in this workflow: {:?}",
        loaded.containers
    );

    // The identical bytes in the container's own directory are refused.
    std::fs::write(container.join("CONTEXT.md"), PROCEDURE).expect("container CONTEXT.md");
    let err = WorkflowDefinition::load_dir(&package)
        .expect_err("a container may not carry an actor procedure");
    assert!(
        matches!(
            &err,
            WorkflowError::NestedPackageWithContext { stage, .. } if stage == "10-investigate"
        ),
        "the refusal must name the container: {err}"
    );
}

// ============================================================ item 14

/// **§21 item 14.** *"produced artifacts retain enough evidence coordinates
/// for later **claim-level audit**"* — §19's chain, walked by hand:
///
/// ```text
/// claim/output fragment
///   -> evidence coordinates
///      source / generation / resource
///      heading/slide/row/message/query-result
///      extractor/query provenance
/// ```
///
/// The walk starts where a claim actually starts — a line of text in the
/// actor's prompt — and goes through the **journaled artifact** (the
/// `context.compiled` payload, round-tripped through the same serialization
/// the journal writes) to the exact stored row. Nothing in the walk consults
/// the in-memory snapshot the compilation produced; that is the difference
/// between *"the coordinates exist"* and *"the coordinates survived into the
/// artifact"*, which is the invariant §19 says is the enabling one.
///
/// It stops where §19 stops: *"A full first-class claim graph is **not
/// required** in Sprint 3."* There is no graph here and this wave built none
/// — the test shows one **could** be built, by building the one edge by hand.
///
/// The last step is what makes it non-vacuous: the coordinate is resolved
/// against Atlas and must return the exact unit, and the same coordinate with
/// its generation altered must return `None`. A chain that "worked" by
/// finding any row with matching text would fail that second half.
#[test]
fn a_rendered_claim_fragment_traces_through_the_artifact_to_its_source_evidence() {
    let (_data, db) = evidence_world();
    let snapshot = compiled_for(&db, EXECUTION, None);

    // --- 1. the claim/output fragment, as the actor saw it.
    let rendered = snapshot.render_onto(AUTHORED);
    let fragment_line = rendered
        .lines()
        .find(|line| line.contains(CLAIM_FRAGMENT))
        .unwrap_or_else(|| panic!("the evidence never reached the prompt:\n{rendered}"));
    assert!(
        fragment_line.starts_with(DATA_PREFIX),
        "evidence body is quoted as data: {fragment_line:?}"
    );
    // The snapshot id is in the prompt, so a produced artifact can cite the
    // world it was written against without anyone remembering it.
    assert!(
        rendered.contains(&format!("snapshot: {}", snapshot.snapshot_id)),
        "the prompt must name the snapshot a later audit joins on: {rendered}"
    );

    // --- 2. the journaled artifact, read back as bytes rather than reused.
    let artifact: Value = serde_json::from_str(&snapshot.json().to_string()).expect("round trip");
    assert_eq!(
        artifact["context_snapshot_id"].as_str(),
        Some(snapshot.snapshot_id.as_str())
    );
    let unit = artifact["bound"]
        .as_array()
        .expect("bound units")
        .iter()
        .find(|u| u["coordinate"]["relative_path"] == "decks/retention.pptx")
        .expect("the deck's unit survived into the artifact");

    // --- 3. every link of §19's chain, off the artifact alone.
    let coordinate = &unit["coordinate"];
    assert_eq!(coordinate["shape"].as_str(), Some("atlas"));
    assert_eq!(
        coordinate["source"].as_str(),
        Some(overlay_source_name(WORK_ID, REPOSITORY).as_str()),
        "source"
    );
    let generation_id = coordinate["generation_id"]
        .as_str()
        .expect("generation")
        .to_string();
    assert!(coordinate["content_key"].is_string(), "generation identity");
    assert_eq!(
        coordinate["relative_path"].as_str(),
        Some("decks/retention.pptx"),
        "resource"
    );
    let ordinal = coordinate["ordinal"].as_u64().expect("row/unit coordinate");
    let provenance = &unit["provenance"];
    assert_eq!(
        provenance["extractor"].as_str(),
        Some(OFFICE_EXTRACTOR),
        "extractor provenance"
    );
    assert_eq!(
        provenance["native_coordinate"].as_str(),
        Some(NATIVE),
        "§19's slide/heading arm: the normalizer's own address"
    );
    assert_eq!(provenance["title"].as_str(), Some("Retention"));
    assert_eq!(provenance["heading_level"].as_u64(), Some(2));
    assert_eq!(
        provenance["byte_span"].as_array().map(|s| s.len()),
        Some(2),
        "the byte span survived"
    );
    assert_eq!(
        provenance["authority_class"].as_str(),
        Some(AuthorityClass::EstateMutable.as_str())
    );

    // --- 4. the coordinate, rebuilt from the artifact, resolves to the
    //        exact stored row the fragment came from.
    let rebuilt = EvidenceCoordinate::Atlas {
        source_name: coordinate["source"].as_str().expect("source").to_string(),
        generation_id: generation_id.clone(),
        content_key: coordinate["content_key"]
            .as_str()
            .expect("content key")
            .to_string(),
        relative_path: "decks/retention.pptx".to_string(),
        unit_key: coordinate["unit_key"].as_str().map(str::to_string),
        ordinal: Some(ordinal),
    };
    match resolve(&db, &rebuilt).expect("resolution") {
        Some(SourceEvidence::Unit {
            text,
            native_coordinate,
            extractor,
            ..
        }) => {
            assert!(
                text.contains(CLAIM_FRAGMENT),
                "the resolved row must be the one the claim quoted: {text:?}"
            );
            assert_eq!(native_coordinate.as_deref(), Some(NATIVE));
            assert_eq!(extractor.as_deref(), Some(OFFICE_EXTRACTOR));
        }
        other => panic!("the artifact's coordinate did not resolve to its source: {other:?}"),
    }

    // --- 5. and the resolution is a keyed lookup, not a text hunt: the same
    //        coordinate under a generation that does not hold it answers None.
    let elsewhere = EvidenceCoordinate::Atlas {
        source_name: coordinate["source"].as_str().expect("source").to_string(),
        generation_id: format!("{generation_id}-not-a-generation"),
        content_key: coordinate["content_key"]
            .as_str()
            .expect("content key")
            .to_string(),
        relative_path: "decks/retention.pptx".to_string(),
        unit_key: coordinate["unit_key"].as_str().map(str::to_string),
        ordinal: Some(ordinal),
    };
    assert_eq!(
        resolve(&db, &elsewhere).expect("resolution"),
        None,
        "a coordinate is resolved by identity; finding the text anyway would not be an audit"
    );
}

/// **§19's `page+bbox` arm — absent by owner ruling, and said so.**
///
/// §19's chain lists `heading/slide/row/message/**page+bbox**/query-result`.
/// The `page+bbox` arm is OCR's, and OCR is the one thing the owner ruled
/// outside 0.3.0, so this build derives no OCR evidence for it to describe.
///
/// The failure mode being guarded is §20's *"hiding normalizer/OCR provenance
/// for prompt aesthetics"*: an **omitted** key is indistinguishable from a
/// dropped fact. So every unit in the journaled artifact carries the `ocr`
/// key, present and null, and its meaning is documented on the field — *no
/// OCR evidence was derived*, never *provenance was dropped*. Exactly the
/// shape C1c gave `EvidenceProvenance::ocr`, restated here because item 14's
/// reader is looking at §19's chain rather than at item 8's list.
#[test]
fn section_19s_page_and_bbox_arm_is_absent_by_owner_ruling() {
    let (_data, db) = evidence_world();
    let snapshot = compiled_for(&db, EXECUTION, None);
    let artifact: Value = serde_json::from_str(&snapshot.json().to_string()).expect("round trip");

    let mut checked = 0usize;
    for tier in ["bound", "referenced"] {
        for unit in artifact[tier].as_array().expect("tier array") {
            let provenance = &unit["provenance"];
            assert!(
                provenance.get("ocr").is_some(),
                "the ocr key must be present, not omitted: {provenance}"
            );
            assert_eq!(
                provenance["ocr"],
                Value::Null,
                "this build derives no OCR evidence; a non-null here would be fabricated"
            );
            checked += 1;
        }
    }
    assert!(checked > 0, "the fixture produced no unit to check");
}

// ===================================================================
// Nested-workflow fixture helpers (m11's shapes, R2)
// ===================================================================

/// Write one workflow package's own `workflow.toml`, creating its directory.
fn write_package(dir: &Path, name: &str, stages: &[&str]) {
    std::fs::create_dir_all(dir).expect("package dir");
    let declared: Vec<String> = stages.iter().map(|id| format!("{id:?}")).collect();
    std::fs::write(
        dir.join("workflow.toml"),
        format!(
            "[workflow]\nname = {name:?}\nversion = \"1\"\nstages = [{}]\n",
            declared.join(", ")
        ),
    )
    .expect("workflow.toml");
}

/// An actor stage: a directory with a `CONTEXT.md`, which is its procedure.
fn write_stage(package: &Path, id: &str, context: &str) {
    let dir = package.join(id);
    std::fs::create_dir_all(&dir).expect("stage dir");
    std::fs::write(dir.join("CONTEXT.md"), context).expect("CONTEXT.md");
}

// ===================================================================
// Live estate — §17's rules 1, 2, 3 and 4 against a real daemon
// ===================================================================

const FAKE: &str = sergeant_rs::backend::fake::FAKE_BACKEND_NAME;

/// Seed `data_dir`'s Atlas with one estate-readonly knowledge generation,
/// **before the daemon opens it**.
///
/// Without this every live compilation is §18's `no_confirmed_generation`
/// rung and every snapshot is empty — which would let the nesting tests below
/// pass by compiling nothing at all, the exact vacuity C1c found on item 6.
/// With it, each leaf's compilation runs §5's nine steps for real and its
/// snapshot has content to differ in.
///
/// Opened and dropped before `daemon::start_with`: one Atlas file is one
/// DuckDB instance, and the daemon's own handle must be the only one open on
/// it (`AtlasDb::open`'s contract).
fn seed_atlas(data_dir: &Path, estate_root: &Path) {
    let mut journal = Journal::open(data_dir).expect("journal");
    let mut db = AtlasDb::open(data_dir).expect("atlas");
    let knowledge = scan(
        "estate-notes",
        SourceKind::LocalKnowledge,
        AuthorityClass::EstateReadonly,
        vec![file(
            "policy.md",
            vec![unit(
                0,
                "Retention policy",
                "The estate's retention policy is thirty days and is changed only by ADR.",
            )],
        )],
    );
    record_scan(
        &mut db,
        &mut journal,
        &knowledge,
        None,
        // S6 D1: this world is compiled by a real daemon addressed at a real
        // estate root, so the generation is bound to that root — not to a
        // stand-in constant, which the estate axis would (correctly) refuse
        // to admit from anywhere.
        &sergeant_rs::domain::source::EstateBinding::Estate(
            estate_root.to_string_lossy().into_owned(),
        ),
    )
    .expect("record knowledge source");
}

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

/// Submit one Work and return `(work id, the 201 body)`.
async fn submit(
    handle: &sergeant_rs::daemon::DaemonHandle,
    estate: &Path,
    workflow: &str,
    intent: &str,
    claimed_parent: Option<&str>,
) -> (String, Value) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("client");
    let mut body = serde_json::json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "intent": intent,
        "estate_root": estate,
        "origin": {"client": "cli", "cwd": estate},
        "workflow": workflow,
    });
    if let Some(parent) = claimed_parent {
        body["claimed_parent_work_id"] = Value::String(parent.to_string());
    }
    let response = support::send_while_alive(
        "submit",
        || {
            client
                .post(format!("{}/v1/work", handle.endpoint))
                .bearer_auth(&handle.token)
                .json(&body)
        },
        || handle.is_alive(),
    )
    .await;
    let status = response.status();
    let value: Value = response.json().await.expect("json body");
    assert_eq!(status, 201, "submit rejected: {value}");
    assert_eq!(value["work"]["state"], "completed", "{value}");
    (
        value["work"]["id"].as_str().expect("work id").to_string(),
        value,
    )
}

fn events_of(data_dir: &Path, work_id: &str, kind: &str) -> Vec<sergeant_rs::domain::event::Event> {
    Journal::replay_data_dir(data_dir)
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.work_id.as_deref() == Some(work_id) && e.kind == kind)
        .collect()
}

/// **§17's first two rules, live.**
///
/// > *"each W1 nested leaf actor gets a **separate** context snapshot"*
/// > *"a container stage has **no** actor snapshot unless it contains an
/// > explicit actor leaf"*
///
/// The fixture is m11's two-level shape: a container `10-investigate` holding
/// two actor leaves, between two ordinary top-level leaves. Four actor leaves
/// run; one container closes on the last of its two.
///
/// **The difference:** each of the four leaves journals its own
/// `context.compiled`, with its own snapshot id, its own execution, its own
/// composed stage id — *and its own content*. The last assertion is the one
/// that makes them separate rather than merely four copies: §5 step 1
/// contributes this run's prior stage records, so a leaf that runs fourth
/// binds strictly more stage-input evidence than one that runs first. Four
/// snapshots compiled once and reused would be identical there.
///
/// **The non-difference:** `10-investigate` is demonstrably present in this
/// run — it is the literal parent segment of two of those composed stage ids,
/// journaled by the engine itself — and **no** snapshot names it. The
/// container gets nothing while its leaves each get one, which is the pair
/// "no snapshot for the container" has to be proved as; asserting the absence
/// alone would pass just as well against a run that compiled nothing at all,
/// and the seeded Atlas plus the content assertions above are what rule that
/// out.
#[tokio::test]
async fn each_nested_leaf_actor_gets_its_own_snapshot_and_the_container_gets_none() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (_repo, _head) = support::scaffold_solo_estate(&estate, "solo");

    let package = estate.join(".sergeant/workflows/nested");
    write_package(
        &package,
        "nested",
        &["00-orient", "10-investigate", "20-implement"],
    );
    write_stage(&package, "00-orient", "orient the work");
    write_stage(&package, "20-implement", "implement it");
    let investigate = package.join("10-investigate");
    write_package(&investigate, "10-investigate", &["00-lead", "10-code"]);
    write_stage(&investigate, "00-lead", "lead the investigation");
    write_stage(&investigate, "10-code", "read the code");

    seed_atlas(data.path(), &estate);

    let (registry, fake) = one_fake([
        sergeant_rs::backend::fake::FakeStep::complete_with("oriented"),
        sergeant_rs::backend::fake::FakeStep::complete_with("led"),
        sergeant_rs::backend::fake::FakeStep::complete_with("read"),
        sergeant_rs::backend::fake::FakeStep::complete_with("implemented"),
    ]);
    let handle = start(data.path(), registry).await;
    let (work_id, _) = submit(
        &handle,
        &estate,
        "nested",
        "compile a world for every nested leaf",
        None,
    )
    .await;

    let leaves = [
        "00-orient",
        "10-investigate/00-lead",
        "10-investigate/10-code",
        "20-implement",
    ];
    let starts = fake.starts();
    assert_eq!(starts.len(), 4, "four actor leaves ran: {starts:?}");

    let snapshots = events_of(data.path(), &work_id, KIND_CONTEXT_COMPILED);
    assert_eq!(
        snapshots.len(),
        4,
        "one snapshot per nested leaf actor, no more and no fewer"
    );

    let stage_ids: Vec<String> = snapshots
        .iter()
        .map(|e| {
            e.payload["coordinate"]["stage"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(
        stage_ids, leaves,
        "each snapshot names its own composed leaf"
    );

    // Separate: distinct identities, and distinct *content*.
    let ids: BTreeSet<&str> = snapshots
        .iter()
        .map(|e| e.payload["context_snapshot_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 4, "four snapshots, four identities");
    let executions: BTreeSet<&str> = snapshots
        .iter()
        .map(|e| e.payload["coordinate"]["execution"].as_str().unwrap())
        .collect();
    assert_eq!(executions.len(), 4, "each names its own fresh execution");
    for (snapshot, start) in snapshots.iter().zip(starts.iter()) {
        assert_eq!(
            snapshot.payload["coordinate"]["execution"].as_str(),
            Some(start.execution_id.as_str())
        );
        assert!(
            snapshot.payload["degradation"].is_null(),
            "the seeded world must compile for real, or this proves nothing: {}",
            snapshot.payload
        );
    }
    let stage_inputs: Vec<u64> = snapshots
        .iter()
        .map(|e| {
            e.payload["plan"]
                .as_array()
                .expect("the §5 plan")
                .iter()
                .find(|step| step["step"] == 1)
                .expect("step 1")["contributed"]
                .as_u64()
                .expect("count")
        })
        .collect();
    assert_eq!(
        stage_inputs,
        vec![0, 1, 2, 3],
        "each leaf compiled its own world: step 1 binds this run's prior stages, so a leaf \
         that ran later binds strictly more — four copies of one snapshot could not"
    );

    // **§21 item 11, live.** §16's audit record is journaled beside each
    // snapshot and is attributed to that leaf's own execution — which is the
    // half `audit_events`' unit tests cannot show, because they never reach
    // the journal. `context.query` is per execution, so four leaves means
    // four records naming four different executions.
    for kind in [KIND_CONTEXT_BOUND, KIND_CONTEXT_QUERY] {
        let recorded = events_of(data.path(), &work_id, kind);
        assert_eq!(
            recorded.len(),
            4,
            "{kind} must be journaled once per compiled leaf: {recorded:?}"
        );
        let attributed: Vec<&str> = recorded
            .iter()
            .map(|e| e.payload["attribution"]["execution"].as_str().unwrap())
            .collect();
        assert_eq!(
            attributed,
            starts
                .iter()
                .map(|s| s.execution_id.as_str())
                .collect::<Vec<_>>(),
            "{kind} must name the execution that issued it"
        );
    }

    // The container: present in this run, and named by no snapshot.
    assert!(
        stage_ids
            .iter()
            .filter(|id| id.starts_with("10-investigate/"))
            .count()
            == 2,
        "the container really is in this run — two composed leaf ids are under it: {stage_ids:?}"
    );
    assert!(
        !stage_ids.iter().any(|id| id == "10-investigate"),
        "a container is not a stage and gets no actor snapshot: {stage_ids:?}"
    );

    handle.shutdown().await;
}

/// **§17's third and fourth rules, live and adversarially — with the fifth as
/// the channel that makes the fourth survivable.**
///
/// > *"causal child Work gets its **own** Work/source/context binding"*
/// > *"child Work does **not inherit** the parent's entire transcript/prompt"*
/// > *"parent causation/output **may be Referenced** when needed"*
///
/// §20 states the fourth again as a non-goal — *parent-prompt inheritance for
/// child Work* — so it is tested adversarially rather than commented: the
/// parent's stage procedure and the parent's intent each carry a marker
/// string that exists nowhere else in the estate, and the test first proves
/// those markers really did reach the parent's prompt. Then the child is
/// submitted with a validated causation claim, and **no byte of either
/// marker** may appear in the child's prompt, the child's intent, or the
/// child's journaled snapshot. A test that only checked the child's prompt
/// looked reasonable would pass against a compiler that copied the parent's
/// world into it.
///
/// What the child *does* get is rule 3 and rule 5: its own Work coordinate,
/// its own binding on its own `sergeant/<child>` branch, its own snapshot id
/// and execution — and one **Referenced** pointer naming the parent Work.
/// That pointer is the whole permitted channel: a coordinate, resolvable on
/// demand, carrying no parent prose at all.
#[tokio::test]
async fn a_child_work_inherits_no_parent_prompt_and_gets_its_own_binding() {
    const PARENT_PROMPT_MARKER: &str = "ZZPARENTPROCEDUREMARKERZZ";
    const PARENT_INTENT_MARKER: &str = "ZZPARENTINTENTMARKERZZ";

    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (_repo, _head) = support::scaffold_solo_estate(&estate, "solo");

    let parent_package = estate.join(".sergeant/workflows/parentflow");
    write_package(&parent_package, "parentflow", &["00-only"]);
    write_stage(
        &parent_package,
        "00-only",
        &format!("the parent's own procedure: {PARENT_PROMPT_MARKER}"),
    );
    let child_package = estate.join(".sergeant/workflows/childflow");
    write_package(&child_package, "childflow", &["00-only"]);
    write_stage(&child_package, "00-only", "the child's own procedure");

    seed_atlas(data.path(), &estate);

    let (registry, fake) = one_fake([
        sergeant_rs::backend::fake::FakeStep::complete_with("parent done"),
        sergeant_rs::backend::fake::FakeStep::complete_with("child done"),
    ]);
    let handle = start(data.path(), registry).await;

    let (parent_id, _) = submit(
        &handle,
        &estate,
        "parentflow",
        &format!("the parent's intent: {PARENT_INTENT_MARKER}"),
        None,
    )
    .await;
    let (child_id, child_body) = submit(
        &handle,
        &estate,
        "childflow",
        "the child's own intent",
        Some(&parent_id),
    )
    .await;
    assert_ne!(parent_id, child_id);
    assert_eq!(
        child_body["work"]["parent_work_id"].as_str(),
        Some(parent_id.as_str()),
        "the causation claim must have validated, or rule 5 has nothing to carry: {child_body}"
    );

    let starts = fake.starts();
    assert_eq!(starts.len(), 2, "{starts:?}");
    let (parent_start, child_start) = (&starts[0], &starts[1]);

    // The markers really were in the parent's prompt. Without this the
    // absence below would prove nothing.
    assert!(
        parent_start.context.contains(PARENT_PROMPT_MARKER),
        "the parent's own procedure never reached its prompt: {:?}",
        parent_start.context
    );
    assert!(parent_start.intent.contains(PARENT_INTENT_MARKER));

    // §17 rule 4 / §20's non-goal: none of it reaches the child.
    for marker in [PARENT_PROMPT_MARKER, PARENT_INTENT_MARKER] {
        assert!(
            !child_start.context.contains(marker),
            "the child inherited the parent's prompt ({marker}): {:?}",
            child_start.context
        );
        assert!(
            !child_start.intent.contains(marker),
            "the child inherited the parent's intent ({marker}): {:?}",
            child_start.intent
        );
    }
    assert!(
        child_start.context.contains("the child's own procedure"),
        "the child still gets its own procedure: {:?}",
        child_start.context
    );

    let child_snapshots = events_of(data.path(), &child_id, KIND_CONTEXT_COMPILED);
    assert_eq!(child_snapshots.len(), 1, "{child_snapshots:?}");
    let payload = &child_snapshots[0].payload;
    let serialized = payload.to_string();
    for marker in [PARENT_PROMPT_MARKER, PARENT_INTENT_MARKER] {
        assert!(
            !serialized.contains(marker),
            "the parent's prose reached the child's compiled world ({marker}): {payload}"
        );
    }

    // §17 rule 3: its own Work/source/context binding.
    assert!(
        payload["degradation"].is_null(),
        "the child's world must compile for real: {payload}"
    );
    assert_eq!(
        payload["coordinate"]["work"].as_str(),
        Some(child_id.as_str())
    );
    let parent_payload = &events_of(data.path(), &parent_id, KIND_CONTEXT_COMPILED)[0].payload;
    assert_ne!(
        payload["context_snapshot_id"], parent_payload["context_snapshot_id"],
        "two Works, two snapshots"
    );
    assert_ne!(
        payload["coordinate"]["execution"],
        parent_payload["coordinate"]["execution"]
    );
    let child_branch = payload["bound"]
        .as_array()
        .expect("bound")
        .iter()
        .find_map(|u| {
            (u["coordinate"]["shape"] == "binding")
                .then(|| u["coordinate"]["work_branch"].as_str().unwrap().to_string())
        })
        .expect("the child's own repository binding");
    assert_eq!(child_branch, format!("sergeant/{child_id}"));

    // §17 rule 5: the permitted channel reached it, as a Referenced pointer.
    let parent_units: Vec<&Value> = payload["referenced"]
        .as_array()
        .expect("referenced")
        .iter()
        .filter(|u| u["coordinate"]["shape"] == "parent_work")
        .collect();
    assert_eq!(
        parent_units.len(),
        1,
        "exactly one parent pointer: {payload}"
    );
    assert_eq!(
        parent_units[0]["coordinate"]["parent_work"].as_str(),
        Some(parent_id.as_str())
    );
    assert!(
        parent_units[0]["excerpt_bytes"].is_null(),
        "a pointer carries no body: {}",
        parent_units[0]
    );
    assert!(
        child_start
            .context
            .contains(&format!("parent work {parent_id}")),
        "the pointer must actually reach the child's prompt: {:?}",
        child_start.context
    );

    // And the parent, which has none, carries no such coordinate at all.
    assert!(
        !parent_payload.to_string().contains("parent_work"),
        "a Work with no causal parent names none: {parent_payload}"
    );

    handle.shutdown().await;
}

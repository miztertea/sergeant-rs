#!/usr/bin/env python3
"""A9: backfill null rationale (all) and empty alternatives_considered
(workflow/stage/engine-gap reps + classification-ledger conflict units) for
BU-P1-* units. Verifies against the corpus's own current state before
writing so nothing is silently skipped or double-applied.
"""
import json

CORPUS = "reference-corpus/behavior-units/P1.ndjson"

RATIONALE = {
"BU-P1-008": "A distinct, observable checkpoint (context loaded before any edit) that later direct-mode stages depend on; not stage-context because the workflow's own ordered step list treats it as one of direct mode's numbered checkpoints, not a judgment note attached to a neighboring stage.",
"BU-P1-009": "A separate durable checkpoint (no duplicate or racing work) distinct from context-loading; a reimplementation could observably skip it and create colliding worktrees, so it earns its own stage rather than being absorbed as stage-context of the prior step.",
"BU-P1-010": "The stage where ownership is claimed and implementation actually starts, a durable fact later validate/PR stages depend on; not helper, since claiming or creating the td task is itself a checkpoint an operator would check for, not a subordinate detail of another stage.",
"BU-P1-012": "A distinct pass/fail checkpoint (validation complete) gating whether a PR can be opened next; kept separate from claim-and-implement because 'implementation complete' and 'validation complete' are observably different states.",
"BU-P1-013": "A separate durable checkpoint (PR open plus CI/review/merge conditions met) from validation passing; an operator needs to know specifically whether delivery is still blocked on review or CI, which validation alone does not establish.",
"BU-P1-014": "The terminal checkpoint of direct mode (outcomes recorded); kept as its own stage rather than folded into pr-and-merge because recording is a distinct, separately-failable action from the merge itself.",
"BU-P1-026": "Mirrors AGENTS.md's own explicit numbered step 1 of the standard workflow; a durable precondition (context established) every later step depends on, not stage-context because it is itself one of the workflow's nine named ordered steps.",
"BU-P1-027": "Step 2 of the same explicit numbered sequence; a distinct checkpoint (queue checked, no duplicate task) from context-loading, since it can independently fail (duplicate creation) even when context loading already succeeded.",
"BU-P1-028": "Step 3; the mode-selection decision is itself the checkpoint gating which sub-workflow (direct-mode or dispatch-mode) executes next, not a judgment note folded into context-loading or queue-checking.",
"BU-P1-029": "Step 4; a separately observable checkpoint (no duplicated or abandoned preserved work) that must complete before execution begins, distinct from mode selection.",
"BU-P1-031": "Step 6; the point at which the chosen mode's own procedure actually begins, a genuine handoff checkpoint between task-intake-and-delivery and whichever mode-specific workflow takes over, not a helper detail of decision-confirmation.",
"BU-P1-034": "Step 9, the terminal checkpoint of the whole workflow (reconciled delivery); kept as its own stage because it is the completion condition the workflow's own trigger/outcome pair (§6.2) names.",
"BU-P1-045": "A standing prohibition independent of any single procedure's stage; applies whenever a decided blocker recurs, regardless of which workflow surfaced it, too broad and cross-cutting to attach to one stage-context.",
"BU-P1-046": "Same class as the adjacent avoid-no-op rules: a cross-workflow prohibition on duplicate artifacts, not scoped to any one stage's checkpoint.",
"BU-P1-047": "A standing evidentiary standard for worker-liveness claims that applies across dispatch, direct, and recovery alike; too broad for stage-context (it would have to be duplicated in every stage that touches worker state) and not itself a reusable outcome-bearing procedure, which rules out workflow.",
"BU-P1-048": "A cross-cutting truthfulness rule about recorded state matching reality, independent of which stage or workflow last touched the task; not scoped narrowly enough to be one stage's context.",
"BU-P1-052": "A single deterministic naming rule (filename to project identity) referenced wherever any procedure needs to resolve a project by name, the textbook shared-context case rather than a standalone checkpoint or flat prohibition.",
"BU-P1-054": "A standing prohibition on touching a specific filesystem location, applicable regardless of which procedure is running; a narrow boundary like this without a stage-specific trigger is the agents-invariant case, not shared-context, since there is no resolution algorithm here for other procedures to consult.",
"BU-P1-055": "A standing security prohibition independent of stage or workflow; applies to every commit action anywhere in the system, which is broader than any one stage-context.",
"BU-P1-056": "A standing tool-resolution rule (PATH fallback) that governs every sgt-* invocation regardless of which workflow issued it; too general-purpose to scope to one stage, and it is a resolution rule rather than a named artifact other procedures look up, so shared-context is a weaker fit than a flat invariant.",
"BU-P1-063": "A standing architectural and scoping commitment (project-topology-first design) that shapes every procedure without itself being a procedure or a fact multiple workflows look up by name; matches the proposal's §6.1 broad-standing-rule examples better than shared-context, which implies a concrete referenced artifact.",
"BU-P1-064": "A standing product-definition fact, not a procedure with a trigger and bounded outcome (ruling out workflow) and not narrow enough to be one stage's judgment (ruling out stage-context).",
"BU-P1-065": "A standing compatibility and deployment commitment (no install step, Bash-3.2 floor) that constrains every script regardless of which workflow runs it; broad and cross-cutting like the adjacent product-definition invariants, not a single reusable procedure with its own trigger and outcome.",
"BU-P1-067": "Defines the shape of the one artifact (project YAML) every project-touching procedure reads; a structural fact reused across many workflows, not itself a checkpoint or a flat cross-cutting prohibition.",
"BU-P1-069": "A cost-awareness note scoped to the start-run checkpoint of the shipping gate, informing rather than itself gating the decision to start; too situational and advisory to be its own stage, but specific enough to this one checkpoint to be more than a bare invariant.",
"BU-P1-070": "A distinct precondition checkpoint (branch committed, doctor healthy, no duplicate active run) that must hold before a run is started; kept as its own stage rather than folded into the adjacent cost-awareness note because it is checkable pass/fail, not an advisory aside.",
"BU-P1-072": "A prohibition specific to how the start-run checkpoint may be invoked (no bulk consent, narrow proven-irrelevance skip); scoped to this one workflow's one stage rather than a standing cross-workflow rule, since --yes/--skip are shipping-gate-specific flags.",
"BU-P1-074": "A judgment and monitoring note scoped to the drive-gates checkpoint (how to read a quiet run) rather than a separate durable checkpoint of its own; nothing new becomes true when this is followed, it only shapes how drive-gates is executed.",
"BU-P1-076": "A set of prohibitions scoped to the drive-gates checkpoint's active-run window; narrower than an agents-invariant since it applies only while this specific workflow's run is active, not universally.",
"BU-P1-077": "The terminal checkpoint of the shipping-gate workflow, the coordinator's own involvement ends here even though the pipeline continues; distinct enough (a completion boundary) to be its own stage rather than folded into drive-gates.",
"BU-P1-079": "A prohibition scoped to the finish/remediation checkpoint's blocked-sync case specifically; narrower than a general 'no improvised git operations' invariant would need to be, since it only fires within this one recovery path.",
"BU-P1-080": "A judgment and scope note for the route-findings checkpoint (validation-only, must not fix) rather than a separate checkpoint of its own; nothing new becomes durably true here beyond what route-findings already records.",
"BU-P1-082": "A deterministic update and preservation rule the finding-router mechanism (its disposition table) must follow on every rerun; subordinate to that shared mechanism rather than an independent checkpoint of its own.",
"BU-P1-083": "A deterministic state-transition rule (resurface hidden states) the same finding-router mechanism follows on rerun; describes the router's behavior, not a standalone checkpoint.",
"BU-P1-084": "A deterministic category-to-policy rule (which finding categories cannot be deferred or ignored) consumed by the same finding-router mechanism; classification logic, not a checkpoint.",
"BU-P1-086": "Describes the deterministic routing mechanism itself (parse, then create/update task, then publish guidance) shared by every worker's independent-review completion, not one workflow's checkpoint.",
"BU-P1-087": "A deterministic normalization table (severity spellings to canonical severity) the same routing mechanism applies on every finding; classification logic reused across every review, not a checkpoint.",
"BU-P1-088": "A deterministic scoping rule (the dedup key) the same routing mechanism enforces on every finding; part of the shared mechanism's contract, not an independent checkpoint.",
"BU-P1-089": "A deterministic conflict-detection and preservation procedure subordinate to the finding-routing mechanism; it is itself a self-contained sub-procedure the router invokes on every write, not a durable checkpoint an operator would check independently, and not reused by name across multiple otherwise-unrelated procedures the way the shared-helper entries are.",
"BU-P1-092": "Continues the shared launch-record evidence shape; the honesty guarantee it adds (never overclaim variant verification) is consulted by dispatch, recovery, and resumption alike, not owned by any one of them.",
"BU-P1-093": "A judgment note scoped to dispatch-mode's entry-validation checkpoint, extending the same fail-before-side-effect guarantee to the resumed-or-recovered case specifically, rather than introducing a new standalone checkpoint.",
"BU-P1-094": "A diagnostic-quality note scoped to the same entry-validation checkpoint, distinguishing two failure reasons; refines how that checkpoint reports failure rather than being a separate checkpoint.",
"BU-P1-098": "A standing deployment-scope commitment independent of any procedure; governs whether a feature is even in scope before any workflow is chosen, matching the non-goal style of the adjacent 'what Sergeant is not' invariants rather than a stage-scoped rule.",
"BU-P1-100": "Enumerates explicit non-goals in the same standing, procedure-independent register as the adjacent audience/deployment invariant; nothing here is a checkpoint any workflow passes through, ruling out stage or stage-context.",
"BU-P1-101": "A canonical glossary definition (the Project concept) multiple procedures reference by name rather than a checkpoint any one of them passes through; the source document's own 'Core concepts' section signals shared, cross-procedure vocabulary.",
"BU-P1-102": "Same glossary register as the Project definition; referenced wherever any procedure resolves what a repository is, not scoped to one stage.",
"BU-P1-103": "Same glossary register; the definition of what a task must record is consulted by every workflow that creates or updates one, not owned by a single stage.",
"BU-P1-104": "Same glossary register; the fleet's non-authoritative status is a fact every monitoring and reconciliation stage across dispatch, direct, and recovery depends on, not a single checkpoint's local rule.",
"BU-P1-105": "Same glossary register; the liveness-is-not-progress rule it states is the canonical source of the identical evidentiary standard echoed in multiple other stage-contexts and invariants elsewhere in this partition.",
"BU-P1-106": "Same glossary register; the definition of what counts as a human decision is shared by every gate-handling stage (needs_input, blocked, ask-user) rather than owned by one of them.",
"BU-P1-107": "The authoritative canonical definition of direct mode, restating and anchoring the same bounded-outcome procedure already extracted from AGENTS.md's own stage sequence; kept at the workflow rung rather than stage-context because it is this concept's summary definition, not a step inside it.",
"BU-P1-108": "Same treatment as direct mode's canonical definition: the authoritative summary definition of the dispatch-mode workflow already extracted from AGENTS.md, not a step or judgment note inside it.",
"BU-P1-109": "Same non-goal register as the adjacent deployment-scope invariants; a scope boundary consulted before any procedure is chosen, not a step inside one.",
"BU-P1-110": "A standing integrity rule for any validation or review agent regardless of which stage's gate it is producing findings for; broader than one stage-context (every drive-gates-style checkpoint depends on it) and not itself a reusable outcome-bearing procedure.",
"BU-P1-111": "A standing definitional fact about no-mistakes's scope (one boundary, not a repeatable check); reinforces rather than duplicates the no-mistakes-shipping-gate workflow's own stage list, so it sits alongside it as an invariant rather than inside it as a stage.",
"BU-P1-112": "A standing precondition on all skill loading, independent of which specific skill or workflow is being loaded; matches the same register as AGENTS.md's procedural-skills preface (also agents-invariant) with an added install-time review clause.",
"BU-P1-113": "A structural fact about where skills can live and why, consulted by loading logic across every harness and workflow; a reference table multiple procedures look up, not a checkpoint.",
"BU-P1-114": "A standing evidentiary rule for provenance determination, applicable whenever any skill's origin is in question, regardless of workflow; too general to scope to the vet-external-skill workflow alone, since ordinary skill loading (not just vetting) can raise this question.",
"BU-P1-115": "A standing architectural fact about worker-brief dependency scope; not a procedure with a trigger and outcome pair, so workflow doesn't fit, and it applies wherever briefs are generated, not to one stage of that.",
"BU-P1-116": "A standing maintenance rule about which installation route to trust for edits; applies regardless of which workflow (setup, update) is active, and it is a flat prohibition rather than a lookup other procedures reference by name.",
"BU-P1-117": "An index of which skills are Sergeant-owned, referenced by mode-selection and skill-loading logic across multiple workflows rather than owned by a single procedure's own step.",
"BU-P1-118": "A standing taxonomic rule governing how every skill in the catalog is invoked, independent of any one procedure; the individual skill-to-invocation-style rows are secondary index entries per the unit's own note, but the two-way split itself is the standing rule.",
"BU-P1-120": "Step 1 of the explicit six-step vetting sequence; a distinct, checkable precondition (source read) later steps depend on, not stage-context because the source document lists it as its own numbered step.",
"BU-P1-121": "Step 2; a separately failable checkpoint (provenance and update path confirmed) distinct from having merely read the source.",
"BU-P1-122": "Step 3; the side-effect-surface audit is its own checkable fact distinct from source confirmation, matching the source's own numbered step list.",
"BU-P1-123": "Step 4; a distinct pass/fail checkpoint (no policy conflict) gating whether pinning and testing may proceed.",
"BU-P1-124": "Step 5; a separately observable action (source pinned) from the conflict check, matching the source's own step numbering.",
"BU-P1-125": "Step 6, the terminal checkpoint (tested in isolation) before broad installation is permitted; the workflow's own completion condition.",
"BU-P1-129": "A single named, fixed inventory that both brief generation and skill-vetting logic must agree on; a reference set multiple procedures consult by name, not a checkpoint of any one of them.",
"BU-P1-130": "Continues the same fixed worker-brief-skill-bundle shared definition, clarifying which entry in that set is excluded from the worker-brief dependency; not a standalone rule.",
"BU-P1-131": "A standing ownership and vendoring rationale independent of any single stage; explains why the skill exists at all despite workers never invoking it, reinforcing the same standing prohibition at a different level (why, not merely that) rather than describing one stage's checkpoint.",
}

ALTERNATIVES = {
"BU-P1-006": ["stage-context of dispatch-mode's dispatch-workers checkpoint", "shared-context (rejected: the monitoring/reconciliation evidentiary standard it depends on is itself shared-context elsewhere, but this specific checkpoint is dispatch-mode-local)"],
"BU-P1-008": ["stage-context of direct-mode's overall trigger", "helper (rejected: context establishment is itself the checkpoint later steps depend on, not a subordinate detail)"],
"BU-P1-009": ["helper (rejected: reconciliation failure is itself an operator-visible outcome, not a subordinate mechanic of context-loading)", "stage-context of 01-load-task-context"],
"BU-P1-010": ["stage-context of 02-reconcile-existing-state", "workflow (rejected: too coarse, this is one step of direct-mode, not its own trigger/outcome pair)"],
"BU-P1-012": ["stage-context of claim-and-implement (folded validation note)", "agents-invariant (rejected: parity with dispatched-worker validation is scoped to direct-mode, not a universal rule)"],
"BU-P1-013": ["stage-context of the validate stage", "helper (rejected: PR/CI/merge conditions are themselves an observable completion gate, not a subordinate detail)"],
"BU-P1-014": ["helper (rejected: recording outcomes is itself the durable evidence other stages and audits depend on)", "stage-context of pr-and-merge"],
"BU-P1-021": ["workflow (rejected: this is a standing precondition on invoking any procedure, not itself a bounded outcome)", "stage-context of a mode-selection stage (rejected: applies before any specific workflow is even chosen)"],
"BU-P1-026": ["stage-context of the standard workflow's overall trigger", "helper (rejected: context establishment is itself the checkpoint later steps depend on, not a subordinate detail)"],
"BU-P1-027": ["helper (rejected: duplicate-avoidance is itself an observable outcome, not subordinate to context-loading)", "stage-context of 01-load-context"],
"BU-P1-028": ["stage-context of 02-check-queue (rejected: mode selection is its own decision point, not a note on the prior step)", "workflow (rejected: mode selection is one step inside task-intake-and-delivery, not a separate bounded procedure)"],
"BU-P1-029": ["stage-context of 03-choose-mode", "helper (rejected: reconciliation failure — duplicated or abandoned work — is itself an operator-visible outcome)"],
"BU-P1-031": ["stage-context of 05-confirm-decisions (rejected: 'execution begins' is itself the durable transition, not a note on the prior gate)", "helper (rejected: this hands off to an entire mode-specific workflow, not a subordinate detail)"],
"BU-P1-034": ["stage-context of 08-handle-decisions", "workflow (rejected: this is the terminal step of task-intake-and-delivery, not a separate bounded procedure)"],
"BU-P1-036": ["agents-invariant (rejected: this is worker-lifecycle vocabulary consulted by specific monitoring/recovery procedures, not a flat cross-cutting prohibition)", "helper (rejected: it defines a durable resumption-path distinction multiple procedures must get right, not a subordinate mechanic of one)"],
"BU-P1-040": ["stage-context of the no-mistakes-shipping-gate workflow (rejected: the boundary is enforced everywhere a worker or remediation loop could be tempted to invoke it, not one stage of the gate itself)", "shared-context (rejected: this is a flat prohibition, not a definition or resolution algorithm multiple procedures look up)"],
"BU-P1-057": ["agents-invariant (rejected: too broad, this specifically gates dispatch-mode's own entry, not every procedure)", "workflow (rejected: it is a validation precondition inside dispatch-mode, not a separate bounded procedure)"],
"BU-P1-065": ["shared-context (rejected: no procedure looks this fact up by name; it is a standing constraint every script must already satisfy)", "obsolete-mechanism (rejected here per adjudication A13/X5's ruling on sergeant-rs's own compiled-binary scope — the Bash-3.2 floor is retained as provenance for what the old install guaranteed, not carried forward as a live constraint or reclassified within this unit)"],
"BU-P1-070": ["stage-context of start-run (rejected: preconditions here are a pass/fail gate before a run is issued, not merely advisory)", "helper (rejected: reattach-vs-duplicate is itself an observable outcome an operator checks)"],
"BU-P1-072": ["agents-invariant (rejected per adjudication A14/X3's scoping — the prohibition binds Sergeant-coordinated runs specifically, i.e. this workflow's start-run checkpoint, not universally; the vendored no-mistakes skill's --yes exception describes standalone usage this repository does not itself exercise)", "stage (rejected: this is a constraint on how start-run may be invoked, not a separate checkpoint with its own durable outcome)"],
"BU-P1-077": ["stage-context of drive-gates (rejected: 'stop driving' is itself the workflow's completion boundary, not a note on the gate-driving stage)", "helper (rejected: it is the operator-visible terminal fact, not a subordinate mechanic)"],
"BU-P1-118": ["workflow (rejected: this is a standing taxonomy governing every skill invocation, not itself a triggered procedure with a bounded outcome)", "shared-context (rejected: no single artifact or definition is being referenced by name here; it is a flat classification rule)"],
"BU-P1-120": ["helper (rejected: reading the source is itself the workflow's first observable checkpoint per its own numbered list, not a subordinate detail)", "stage-context of vet-external-skill's overall trigger"],
"BU-P1-121": ["stage-context of 01-read-source", "helper (rejected: confirming provenance is itself a separately failable checkpoint per the source's own step numbering)"],
"BU-P1-122": ["helper (rejected: the action-surface audit is itself a distinct checkable fact, not subordinate to step 2)", "stage-context of 02-confirm-source"],
"BU-P1-123": ["stage-context of 03-check-actions", "agents-invariant (rejected: this check is scoped to the vetting workflow's own gate, not a standing rule outside it)"],
"BU-P1-124": ["helper (rejected: pinning is itself an observable, separately-completable action per the source's numbered steps)", "stage-context of 04-verify-no-conflict"],
"BU-P1-125": ["stage-context of 05-pin-source", "workflow (rejected: this is the terminal step of vet-external-skill, not a separate bounded procedure)"],
"BU-P1-126": ["stage-context of vet-external-skill's update path (rejected: this is its own re-entry point into the same vetting discipline at update time, checkable independently of initial adoption)", "separate workflow: 'update-managed-skill' (rejected: too thin — a single-step re-check, not a multi-step bounded procedure of its own)"],
"BU-P1-129": ["helper (rejected: the fixed inventory is looked up by name from multiple otherwise-independent procedures — brief generation and skill vetting — the textbook shared-context case, not a subordinate mechanic of one)", "agents-invariant (rejected: too broad, this is a specific named list, not a standing cross-cutting rule)"],
"BU-P1-131": ["stage-context of the no-mistakes-shipping-gate workflow (rejected: this rationale applies to every worker regardless of which stage of the gate is running, and it explains why the skill is vendored at all rather than governing one checkpoint)", "shared-context (rejected: this is a flat prohibition-plus-rationale, not a definition or resolution algorithm looked up by name)"],
}

CONFLICT_IDS = {
"BU-P1-072", "BU-P1-040", "BU-P1-131", "BU-P1-065", "BU-P1-057", "BU-P1-036",
"BU-P1-007", "BU-P1-107", "BU-P1-008", "BU-P1-009", "BU-P1-010", "BU-P1-011",
"BU-P1-012", "BU-P1-013", "BU-P1-014", "BU-P1-118", "BU-P1-021", "BU-P1-129",
}


def main():
    with open(CORPUS, "r", encoding="utf-8") as f:
        units = [json.loads(l) for l in f if l.strip()]

    expected_need_alt = set()
    for u in units:
        if u["alternatives_considered"] == []:
            if u["representation"] in ("workflow", "stage", "engine-gap"):
                expected_need_alt.add(u["id"])
            if u["id"] in CONFLICT_IDS:
                expected_need_alt.add(u["id"])

    got_alt = set(ALTERNATIVES.keys())
    missing = expected_need_alt - got_alt
    extra = got_alt - expected_need_alt
    assert not missing, f"missing alternatives for: {sorted(missing)}"
    assert not extra, f"alternatives supplied for units that didn't need them: {sorted(extra)}"

    expected_null_rat = {u["id"] for u in units if u["rationale"] is None}
    got_rat = set(RATIONALE.keys())
    missing_r = expected_null_rat - got_rat
    extra_r = got_rat - expected_null_rat
    assert not missing_r, f"missing rationale for: {sorted(missing_r)}"
    assert not extra_r, f"rationale supplied for units that already had one: {sorted(extra_r)}"

    rat_count = 0
    alt_count = 0
    for u in units:
        if u["id"] in RATIONALE:
            assert u["rationale"] is None
            u["rationale"] = RATIONALE[u["id"]]
            rat_count += 1
        if u["id"] in ALTERNATIVES:
            assert u["alternatives_considered"] == []
            u["alternatives_considered"] = ALTERNATIVES[u["id"]]
            alt_count += 1

    with open(CORPUS, "w", encoding="utf-8") as f:
        for u in units:
            f.write(json.dumps(u, ensure_ascii=False) + "\n")

    print(f"Backfilled rationale: {rat_count}")
    print(f"Backfilled alternatives_considered: {alt_count}")
    remaining_null_rat = [u["id"] for u in units if u["rationale"] is None]
    print(f"Remaining null rationale: {len(remaining_null_rat)} {remaining_null_rat}")


if __name__ == "__main__":
    main()

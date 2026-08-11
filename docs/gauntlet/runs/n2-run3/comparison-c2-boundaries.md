# C2 — Boundary comparison: workflow-boundary, stage-boundary, representation agreement

Scope: `reference/proposal-next-iteration-icm-workflows.md` §9.9's
"workflow-boundary agreement," "stage-boundary agreement," and
"representation agreement." Generated artifacts:
`docs/gauntlet/runs/n2-run3/.sergeant/workflows/repo-to-icm/{20-harvest,
30-normalize,40-classify,50-synthesize,60-draft,80-adversarial-review}/
output/*` and the 18 materialized packages under
`docs/gauntlet/runs/n2-run3/.sergeant/drafts/workflows/*/`. Reference:
`reference-corpus/draft-workflows/*`, `reference-corpus/provenance-map.md`,
`reference-corpus/behavior-units/P{1..8}.ndjson`, per
`reference-corpus/FROZEN.md` v1. Prior run for delta: `docs/gauntlet/runs/
n2-run2/comparison-c2-boundaries.md`.

## Scope discipline

Per `partition-ledger.md`, this run completed 6 of 21 partitions (28 of 82
`decompose`-dispositioned files) before `20-harvest` stopped; every later
stage (`30-normalize` through `80-adversarial-review`) proceeded against
that 312-unit, 28-file corpus without flagging the other 15 partitions
still `pending` — the run's own `80-adversarial-review` output
(`80-adversarial-review/output/findings.ndjson`, AF-0001/AF-0002/AF-0003)
records this as the dominant defect and is not re-litigated here. Every
reference figure below is filtered to the **same 28 files** this run
actually read:

`AGENTS.md`, `README.md`, `docs/README.md`, `docs/what-is-sergeant.md`,
`docs/skills.md`, `docs/repo-scoped-skills.md`,
`skills/sergeant-help/SKILL.md`, `docs/getting-started.md`,
`docs/using-sergeant.md`, `docs/troubleshooting.md`, `docs/schema.md`,
`schema/project.yaml.example`, `mise.toml`, `docs/callbacks.md`,
`bin/sgt-list`, `bin/sgt-context`, `bin/sgt-status`, `bin/sgt-sync`,
`bin/sgt-td-list`, `bin/sgt-graphify`, `skills/load-project/SKILL.md`,
`skills/cross-repo-work/SKILL.md`, `skills/dispatch/SKILL.md`,
`bin/sgt-dispatch`, `bin/sgt-td-create`, `bin/sgt-treehouse-init`,
`bin/_sgt-review-axes.sh`, `templates/worker-brief.md`.

27 of these 28 files have at least one reference behavior unit (385 units
total, filtered directly from `behavior-units/P{1..8}.ndjson` by
`source.path`, reproducible with the query in the working notes at the end
of this document). The one exception, `bin/_sgt-review-axes.sh`, is
dispositioned `helper-evidence` in the reference's own
`source-inventory.md` (row 137) — informing `helper-map.md`'s
`@@review-axes` candidate without itself being separately extracted — while
this run dispositioned it `decompose` and extracted `BU-0300`–`BU-0302`
from it. That is a genuine disposition disagreement worth naming but is a
coverage/precision question (C1/C3 territory), not adjudicated further
here.

**Overlap with `n2-run2`'s scope is small.** Run 2 covered `AGENTS.md`,
`README.md`, and 14 fleet-dispatch `bin/` files; this run covers a
different, larger 28-file set overlapping only on `AGENTS.md`, `README.md`,
and `bin/sgt-dispatch`. Comparisons to run 2's numbers below are therefore
run-level aggregate comparisons, not a controlled same-scope re-test,
except where explicitly noted as a same-file, same-content comparison.

## 1. Package correspondence

| # | Run 3 candidate | Stages | Reference counterpart(s) | Correspondence basis |
|---|---|---|---|---|
| 1 | `dispatch-worker` | 6 | `dispatch` (6 stages) | 47 records share `workflow: dispatch-worker`; sources `AGENTS.md`, `README.md`, `skills/dispatch/SKILL.md`, `bin/sgt-dispatch`, `bin/sgt-td-create`, `bin/sgt-treehouse-init`, `templates/worker-brief.md` — the same file set `provenance-map.md` routes into `draft-workflows/dispatch/` |
| 2 | `task-intake-and-execution` | 8 | `task-intake-and-route` (6 stages) + `direct-implementation` (5 stages) | `AGENTS.md`'s nine-numbered-step list, independently recovered by both sides (§2.2) |
| 3 | `worker-response-and-recovery` | 2 | `respond-to-worker` (2 stages) + `recover-stalled-worker` (3 stages) + `wake-and-resume` (1 stage) | sources `docs/using-sergeant.md`, `docs/troubleshooting.md`, `templates/worker-brief.md`, `skills/dispatch/SKILL.md` span the same three reference packages' territory |
| 4 | `validation-gate` | 2 | `validate-and-ship` (7 stages, `20-select-intent-transport`/`30-start-run`/`50-reconcile-custody` slice) | sources `AGENTS.md`, `docs/using-sergeant.md`, `templates/worker-brief.md`; `BU-0161` (probe/select intent transport), `BU-0160` (readiness publish), `BU-0166` (transport durably auditable) name the same checkpoints as `docs/using-sergeant.md`'s own `BU-P8-084/085/087/089` (`workflow: no-mistakes`, later folded into `validate-and-ship`) |
| 5 | `project-registration` | 1 | `load-project` (3 stages) | 19 records share `workflow: project-registration`; sources `bin/sgt-list`, `bin/sgt-context`, `bin/sgt-status`, `bin/sgt-sync`, `bin/sgt-td-list`, `skills/load-project/SKILL.md` — the exact file set `provenance-map.md` routes into `draft-workflows/load-project/` |
| 6 | `fleet-monitor-and-reconcile` | 0 | `monitor-fleet` (1 stage) + `reconcile-and-cleanup-fleet` (1 stage) | sources `README.md` (`busy:`/verification helpers) + `docs/using-sergeant.md` (`sync-all`, `assess-worker-health`) |
| 7 | `shipping-gate-driving` | 1 | `validate-and-ship` (`40-drive-gates`/`60-close-out` slice) | sources `README.md`; `start-run`/`drive-gate`/`recover-from-failure` stage-context names echo `validate-and-ship`'s own `30-start-run`/`40-drive-gates` |
| 8 | `review-finding-routing` | 2 | `route-review-findings` (1 stage) | sources `README.md`; `BU-0096` "actionable findings become owning-repo td tasks" matches `route-review-findings`'s single stage almost verbatim |
| 9 | `installation-and-setup` | 1 | (no direct package — `sergeant-setup` is nearest, out of scope's main mass) | source `mise.toml`; `docs/getting-started.md` contributes 1 `stage` (`BU-0129`) but most of `sergeant-setup`'s 36 reference units come from `.agents/skills/sergeant-setup/SKILL.md`, not read this run |
| 10 | `project-graphify` | 1 | `project-graph` (2 stages) | source `bin/sgt-graphify` almost exclusively; matches `provenance-map.md`'s `bin/sgt-graphify` → `draft-workflows/project-graph/` ×4 row |
| 11 | `fleet-cleanup` | 1 | `reconcile-and-cleanup-fleet` (1 stage) | source `docs/troubleshooting.md`; `BU-0171`'s precondition gate matches `reconcile-and-cleanup-fleet`'s single `00-require-terminal` stage in spirit |
| 12 | `dag-orchestration` | 1 | none materialized — reference routes this territory to `engine-pressure.md` G2 | source `schema/project.yaml.example` only, from content (`after:` dependency declarations) the reference's own `schema/project.yaml.example` extraction (`BU-P7-001`–`004`) never surfaces — see §2.4 |
| 13 | `callback-delivery` | 0 | `deliver-external-callback` (1 stage) | source `docs/callbacks.md`; 7 `helper` records (correlation-id opacity, idempotent registration/sync, bounded consumer, closed outcome set) support the same seal/dedup territory `deliver-external-callback`'s `00-seal` stage names |
| 14 | `skill-adoption` | 0 | `vet-external-skill` (7 stages) | source `docs/skills.md`; `BU-0119` is a direct `representation: workflow` match — see §2.3 |
| 15 | `sergeant-help-query` | 0 | `sergeant-help` (3 stages) | source `skills/sergeant-help/SKILL.md`; `BU-0124` is a direct `representation: workflow` match — see §2.3 |
| 16 | `troubleshoot-td-identity` | 0 | (no dedicated package; nearest is `load-project`'s td-surface territory) | source `docs/troubleshooting.md`; single unattached `stage-context` `BU-0173` |
| 17 | `cross-repo-planning` | 0 | `cross-repo-work` (6 stages) | source `skills/cross-repo-work/SKILL.md` — read in full (6 units), but 5 of 6 demoted to `agents-invariant` — see §2.1 |
| 18 | `undocumented-failure-escalation` | 0 | `recover-stalled-worker`'s `50-escalate-undocumented` stage | source `docs/troubleshooting.md`; `BU-0192` direct `representation: workflow` match, but reference gives this same content a real `stage` (`BU-P8-109`, `stage: escalate-undocumented-failure`) inside `recover-stalled-worker` rather than its own package — see §2.3 |

## 2. Workflow-boundary agreement: reusable outcomes or file-shape mirroring?

### 2.1 `cross-repo-planning` — the file was read in full; the boundary still collapsed

This is the sharpest negative example in the run, and it is a
representation failure, not a coverage failure. `skills/cross-repo-work/
SKILL.md` produced exactly 6 behavior units in this run's corpus
(`BU-0267`–`BU-0272`) — the reference extracts 16 units from the same file,
all 16 tied to a workflow (`draft-workflows/cross-repo-work/`, 3
`workflow` + 6 `stage` + 7 `stage-context` records, zero `agents-invariant`
— confirmed directly against `behavior-units/P5.ndjson` filtered to this
path). This run's `40-classify` gave only 1 of its 6 units
(`BU-0267`, `stage-context`, `workflow: cross-repo-planning`,
`stage: assign-ownership`) a workflow tie; the other 5
(`BU-0268`–`BU-0272`) were classified `agents-invariant` with rationales
that pattern-match the sentence to an existing AGENTS.md-level rule ("the
same 'ask only when genuinely necessary' shape as `BU-0029`," "the same
shape as `BU-0018`" — `40-classify/output/classifications.ndjson`) rather
than treating it as procedure-specific judgment inside a cross-repo
planning workflow. The result: `cross-repo-planning`'s own `provenance.md`
can cite only one record for the entire package, and `60-draft` correctly
declines to invent stages around it rather than papering over the gap —
but the underlying cause is that `40-classify` read the whole file and
mostly filed it under the wrong representation tier, not that the file
went unread. This is the closest run-3 analog to run 2's "read it, still
missed the boundary" failure mode, just one rung earlier in the ladder
(6.1 vs. 6.2, not 6.2 vs. 6.5).

### 2.2 `task-intake-and-execution` — the run's strongest positive match, again

As in run 2's `standard-task-workflow`, this candidate independently
recovers `AGENTS.md`'s own nine-numbered-step list blind to the reference:
`01-resolve-context` (`BU-0025`, step 1), `02-resolve-task` (`BU-0026`,
step 2), `03-reconcile-before-start` (`BU-0028`, step 4),
`04-execute` (`BU-0031`, step 6), `06-handle-decision-gate` (`BU-0034`,
step 8), `08-reconcile-and-deliver` (`BU-0035`, step 9) — six of the nine
AGENTS.md steps land on real `stage` records here, matching
`task-intake-and-route`'s final `01-load-context`/`03-choose-mode`/
`05-confirm-decisions`/`06-execute`/`08-handle-decisions`/
`09-reconcile-deliver` one-for-one by step number and content, independent
of the reference. Step 7 ("monitor") is correctly left unattached
(`BU-0033`, `monitor-progress`, classified `stage-context` — its own
rationale explicitly answers §6.3 "no": "replacing the implementation
would not remove a checkpoint anyone cared about" — matching the
reference's own A4 fold of `07-monitor` into `08-handle-decisions`). The
blur, as in run 2, is a genuine three-way spread across
`task-intake-and-route` and `direct-implementation`: `05-direct-mode-
implementation` and `07-direct-mode-delivery` carry `direct-implementation`
content (source `docs/using-sergeant.md`'s `stage: direct-implementation`
records, e.g. `BU-P8-053`/`BU-P8-056`/`BU-P8-058` in the reference), while
the rest tracks `task-intake-and-route`. This is the same honest blend
run 2 found, not a regression.

### 2.3 Three workflow-level matches without stages: correct discipline, real under-staging

`skill-adoption` (`BU-0119`), `sergeant-help-query` (`BU-0124`), and
`undocumented-failure-escalation` (`BU-0192`) are each anchored by a real
`representation: workflow` record — the strongest form of workflow-boundary
agreement (an independently-recognized trigger/outcome/completion
condition, not an inferred cluster). Their reference counterparts,
however, all have real stages this run never materialized:
`vet-external-skill` (7 stages), `sergeant-help` (3 stages), and
`recover-stalled-worker`'s `50-escalate-undocumented` stage
(`BU-P8-109`, sourced from the same `docs/troubleshooting.md` this run's
`BU-0192` also comes from). This run's own discipline against inventing
checkpoints (`draft-report.md`'s "Zero-stage candidates" note) is the
correct behavior given the evidence it held — no `representation: stage`
record carries any of these three `workflow` values in the 312-record
corpus — but it leaves all three packages meaningfully thinner than their
reference counterparts. This is best read as an honest under-staging,
self-disclosed rather than papered over, not a boundary miss: the
*workflow* boundary itself is correctly drawn in all three cases.

### 2.4 `dag-orchestration` — a package where the reference sees an engine gap, not a workflow

`dag-orchestration`'s three supporting records (`BU-0201`–`BU-0203`) are
sourced entirely from `schema/project.yaml.example`. The reference's own
extraction of that same file (`BU-P7-001`–`004`) yields no unit about a
DAG `after:` dependency field at all — its four units cover project-name
identity, GitHub CLI precedence, graphify-publish gating, and
`agent_instructions` inheritance. The reference's DAG-dependency-advance
territory instead lives entirely in `engine-pressure.md`'s G2 claim
(sourced from `bin/sgt-dag-dispatch-hook`, `bin/sgt-dag-run`, and
`skills/dispatch/SKILL.md`'s DAG passages — none of which this run's 28
files include, except `skills/dispatch/SKILL.md` itself, from which this
run extracted no DAG-specific unit). Two readings are both plausible from
what this run held: either the schema comment this run quoted for
`BU-0201`–`0203` describes something genuinely absent from the reference's
own quoted span of the same file (worth a source-citation check under C1),
or this run correctly found dependency-declaration content the reference's
independent extraction simply missed. Either way, the two corpora disagree
at the representation-family level here — reference treats DAG dependency
advance as *engine-incapable-of-expressing-this* territory; this run
treats it as an ordinary workflow with a stage. This is flagged, not
resolved, here — it needs the source-quote check C1/C3 would run.

## 3. Stage-boundary agreement: over-staging and under-staging vs. the post-A4 standard

`reference-corpus/draft-workflows/task-intake-and-route/provenance.md`'s
"Adjudication A4" section is the standing precedent: `AGENTS.md`'s nine
numbered steps originally mirrored one-for-one into nine stages; steps 2
("check queue"), 4 ("reconcile state"), and 7 ("monitor") demoted to
helpers folded into the next judgment-bearing stage because each "carried
only the §6.5 deterministic-machinery boilerplate as their extraction
justification — none had an 'Additional note' checkpoint argument."

**This run reproduces the A4-demoted shape on 2 of 3 checkpoints, but by a
different — and more defensible — route than run 2 did.** `task-intake-
and-execution` keeps `02-resolve-task` (`BU-0026`, the check-queue
equivalent) and `03-reconcile-before-start` (`BU-0028`, the reconcile-state
equivalent) as independent stages, correctly folding `07-monitor`'s
equivalent (`BU-0033`) instead. Unlike run 2's `standard-task-workflow`
(which run 2's own C2 report showed classified these same two checkpoint
types by jumping straight to §6.5 machinery language), this run's
`40-classify` output states a genuine §6.3 answer for both: `BU-0026`'s
rationale reads "[w]hether an existing canonical td task is reused or a
new one created is exactly the kind of durable, evidence-bearing outcome
operators want tracked as its own boundary," and `BU-0028`'s reads
"'existing state was reconciled and preserved work resumed rather than
duplicated, before starting' is a real, evidence-bearing gate operators
would want measured on its own" (`40-classify/output/
classifications.ndjson`). This satisfies the ladder's own procedural
requirement (`.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` §6.3,
"a checkpoint question is answered *before* a lower-rung classification is
legal, not assumed by arriving there") — the disagreement with A4 is a
**reasoned one**, landing on the opposite side of the same test the
reference's own adjudicators applied, not a skipped question. Per the
proposal's own disagreement taxonomy (§9.8), this is closer to "legitimate
alternate decomposition" than "generator invention" — a live case for a
future adjudication round to actually resolve, not a defect in this run.

**Under-staging, self-caught by this run's own review.** In the opposite
direction, `worker-response-and-recovery`'s `recover-worker` cluster (5
stage-context records — `BU-0039`, `BU-0146`, `BU-0159`, `BU-0174`,
`BU-0286`) never produced a matching `representation: stage` record
anywhere in the corpus, despite naming a checkpoint the workflow's own name
promises ("worker-response-**and-recovery**"). `80-adversarial-review`
caught this directly (`findings.ndjson` AF-0004): each of the five
records' §6.3 "no" answer rests on a sibling record's claim that the real
checkpoint "is established more specifically elsewhere," and that chain
never terminates in an actual `stage` record — a circular demotion, not a
resolved one. Re-applying §6.3 to the cluster's combined content (gating on
prior evidence, one-shot-per-attempt with durable recording, escalation on
three named failure conditions, diagnose-and-preserve-state before
kill/relaunch) reads as a real checkpoint. The reference's own
`recover-stalled-worker` package materializes exactly this territory as
real stages (`00-collect-signals`, `40-escalate-on-second-attempt`,
`50-escalate-undocumented`), confirming AF-0004's read.

**Net direction, contrasted with run 2.** Run 2's C2 report found
over-staging exclusively, no under-fragmentation ("there is no
under-fragmentation finding to report in the opposite direction"). This
run shows both: one reasoned-but-contested over-staging case
(`task-intake-and-execution`) and one self-caught under-staging case
(`worker-response-and-recovery`/`recover-worker`) — a genuinely different,
more symmetric failure profile, not simply "fixed" or "still broken."

## 4. Representation agreement

### 4.1 Distribution, same 28-file scope

| Representation | Run 3 (312 units) | Reference, same 28 files (385 units) |
|---|---:|---:|
| `agents-invariant` | 106 (34.0%) | 84 (21.8%) |
| `helper` | 87 (27.9%) | 28 (7.3%) |
| `stage-context` | 73 (23.4%) | 103 (26.8%) |
| `stage` | 31 (9.9%) | 68 (17.7%) |
| `shared-helper` | 8 (2.6%) | 15 (3.9%) |
| `workflow` | 3 (1.0%) | 26 (6.8%) |
| `shared-context` | 2 (0.6%) | 43 (11.2%) |
| `obsolete-mechanism` | 0 | 14 (3.6%) |
| `engine-gap` | 2 (0.6%) | 4 (1.0%) |

Run-3 figures per `80-adversarial-review/output/review-summary.md`
Axis 4 §3 (independently recomputed there against the same
`classifications.ndjson`, matching the numbers in this run's `20-harvest`
partition-ledger's covered scope). `helper`+`shared-helper` = 30.4% of this
run's corpus, vs. run 2's 73.1% on its own 18-file scope
(`n2-run2/comparison-c2-boundaries.md` §4.1) — a large aggregate swing in
the direction the ladder's §6.3-before-§6.5 rule change was meant to
produce. `stage`+`stage-context` here is 33.3%, closer to (though still
below) the same-scope reference's 44.5%, a marked improvement on run 2's
13.9% vs. 45.4% gap. `shared-context` remains far below the reference's
share (0.6% vs. 11.2%) and `obsolete-mechanism` is unused entirely — this
run's `50-synthesize` reports that bucket empty by count alone without
per-unit reasoning, which `80-adversarial-review` separately flagged
(AF-0005) as insufficiently justified rather than confirmed correct.

### 4.2 Did the discipline change actually move the distribution, or just relabel it?

Both, in different places. Two structural checks this run's own review ran
directly answer the "was §6.3 actually asked" question the ladder file
itself now names as the discriminator
(`.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` §6.3, added
specifically in response to run 2's finding and citing
`n2-run2/comparison-c2-boundaries.md` §4.2 by name):

- **Rationale ordering, sampled.** `review-summary.md` Axis 2 sampled 12
  `helper`/`shared-helper` and 10 `stage` records and found every
  `helper`/`shared-helper` rationale states the required §6.3 answer before
  its §6.5 language — the specific procedural defect run 2 found (73% of
  that corpus skipping straight to §6.5 language) is absent from this
  sample.
- **The over-promotion tell, checked directly.** The ladder file names a
  specific symptom: a `50-synthesize` helper cluster that maps one-to-one
  onto a single source file is evidence §6.3 was never really asked.
  `dispatch-worker`'s own helper clustering in `provenance.md` groups by
  contract across files — "model/variant resolution & validation" (6
  units, spanning `AGENTS.md`, `README.md`, `docs/schema.md`), "td task
  creation mechanics" (5 units), "worktree/treehouse setup" (5 units) — not
  by originating script, unlike run 2's `dispatch-mode` cluster
  (`respond`: `BU-0079`–`0084`, all six from `bin/sgt-respond`; `drain`:
  `BU-0064`–`0070`, all seven from `bin/sgt-drain`).

**But the same file, read for the same content, can still land on the
opposite tier from the reference even with the ordering rule honestly
applied.** `bin/sgt-dispatch` is the cleanest same-file check available
(present in both runs' scope): reference gives it 1 `workflow` + 2 `stage`
+ 2 `stage-context` + 1 `obsolete-mechanism` (5 of 6 units checkpoint-tier
or higher — `BU-P6-123`–`128`). Run 2 classified 9 of its 10 `bin/sgt-
dispatch` units `shared-helper`. **This run classifies 8 of 10 `helper`**
(`AGENTS.md`, `README.md`, `bin/sgt-dispatch` per-file counts recomputed
directly from `20-harvest/output/behavior-units.ndjson` joined against
`40-classify/output/classifications.ndjson`) — essentially the same
verdict as run 2 reached, on the same file, despite the intervening
discipline change. `bin/sgt-graphify` shows the identical pattern more
starkly: reference gives all 4 of its units checkpoint tier (2 `stage` + 2
`stage-context`, all `workflow: graphify`); this run gives 9 of 10 `helper`
and only 1 `stage`. Both cases carry genuine §6.3 rationale text (not
skipped), but the *answer* to "would this checkpoint survive
reimplementation" still diverges sharply from the reference's on exactly
the content type (single-purpose `bin/` command scripts) run 2's skew was
built from. The distribution-level improvement in §4.1 is real, but it
traces mostly to which *new* files entered this run's scope (`docs/`,
`skills/*.md`, small read-only `bin/` commands like `sgt-list`/`sgt-
status`) carrying more inherently judgment-bearing prose, not to a
uniform re-classification of the fleet-machinery content both runs share.

### 4.3 Strongest examples, both directions

**Strongest disagreement (under-promotion, repeats run 2's exact pattern).**
`bin/sgt-graphify`, `BU-0251` ("an incomplete staged output is never
promoted") — this run: `helper`. Reference, same file: `BU-P6-088`
("publication only proceeds once extraction has produced a directory with
both required artifacts present for every requested repo... otherwise the
run fails before anything is published"), representation `stage`,
`workflow: graphify`, `stage: publish-graph` — "a meaningful,
durable, binary-outcome checkpoint (published or not) operators would want
measured, matches §6.3." Same file, same swap/publish machinery, same
underlying property (atomic, all-or-nothing promotion); this run's own
`project-graphify/01-publish-graph` stage (`BU-0250`) picks up the
adjacent all-or-nothing-extraction checkpoint but classifies the
promotion-atomicity property one step later (`BU-0251`) as machinery
subordinate to it, where the reference treats the *combined* atomic-publish
property as the stage itself.

**Strongest agreement (representation category correctly matches where
both sides reached the same content).** `README.md`'s finding-router
material: this run's `BU-0096` ("a dispatched worker's review finding
becomes an owning-repo td task with durably published blocking guidance"),
representation `stage`, `workflow: review-finding-routing`,
`stage: route-finding`. Reference, same file, same underlying rule:
`BU-P6-082` (routed via `bin/sgt-review-findings`, homed to
`draft-workflows/route-review-findings/`'s single stage
`00-publish-or-clear-gate`) — both sides independently name this a real
checkpoint from `README.md` prose alone, and `review-finding-routing`'s
second stage `02-reconcile-hand-edit` (`BU-0101`, "a stored finding card
modified outside the router... is preserved, not overwritten") has no
reference stage of its own but is exactly the kind of judgment-bearing
addition the reference's own `finding-router` helper-map entry gestures at
without promoting to a checkpoint — a case where this run's stage-level
granularity is arguably *finer* than the reference's, not coarser.

## 5. Summary

- **Workflow-boundary agreement**: mixed and file-dependent, not
  uniformly better or worse than run 2. Strongest positive match repeats
  run 2's pattern almost exactly (`task-intake-and-execution` ↔
  `task-intake-and-route`'s numbered steps, §2.2). The sharpest negative
  example this run (`cross-repo-planning`, §2.1) is a genuinely new failure
  mode: the source file was read completely, but 5 of 6 of its units were
  filed as broad policy (`agents-invariant`) rather than workflow-specific
  judgment, collapsing the workflow boundary one rung earlier in the ladder
  than run 2's failures did. One boundary disagreement (`dag-
  orchestration`, §2.4) sits at the representation-family level — this run
  treats DAG-dependency content as an ordinary workflow where the reference
  treats the same territory as an engine gap — and needs a source-citation
  check to resolve which side is right about what the source actually
  says.
- **Stage-boundary agreement**: more symmetric than run 2. One reasoned
  over-staging disagreement against the A4 precedent
  (`task-intake-and-execution`'s `resolve-task`/`reconcile-before-start`,
  §3) that — unlike run 2's identical-shaped finding — is backed by a
  genuine §6.3 argument rather than a skipped question, making it a live
  adjudication candidate rather than a clear defect. One self-caught
  under-staging case (`worker-response-and-recovery`'s `recover-worker`
  cluster, AF-0004) that run 2 had no equivalent of, since run 2 found no
  under-fragmentation at all.
- **Representation agreement**: the ladder's §6.3-before-§6.5 ordering
  rule (added in direct response to run 2's finding) is honestly applied —
  sampled rationales state the required answer, and the specific
  file-shape-mirroring tell the rule was designed to catch is absent from
  this run's helper clustering (§4.2). The aggregate skew shrank
  substantially (`helper`+`shared-helper` 73.1%→30.4%, `stage`+
  `stage-context` 13.9%→33.3% of respective same-scope corpora). But on
  the one file both runs' scopes share and that drove most of run 2's
  skew (`bin/sgt-dispatch`), and on a second file of the same shape
  (`bin/sgt-graphify`), this run reaches essentially the same
  under-promoted verdict as run 2 despite asking the question honestly —
  the aggregate improvement is real but is concentrated in the newly-read
  file types (prose docs/skills, small read-only `bin/` commands), not in
  a changed verdict on the fleet-machinery `bin/` scripts both runs
  actually share.

## Working notes (reproducibility)

Per-file representation counts (both runs) were computed by joining each
run's `20-harvest/output/behavior-units.ndjson` (`id` → `source.path`)
against its `40-classify/output/classifications.ndjson`
(`behavior_id` → `representation`). Reference per-file and
scope-filtered counts were computed directly against
`reference-corpus/behavior-units/P{1..8}.ndjson`, filtering
`source.path` (stripped of its `reference/sergeant-upstream/` prefix)
against the 28-file scope list in this document's header. Not committed as
a script; both queries are short enough to be reproduced ad hoc from the
paths cited throughout.

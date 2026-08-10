# C2 — Boundary comparison: workflow-boundary, stage-boundary, representation agreement

Scope: `docs/gauntlet/contracts/N2.md` §9.8-9.9 measurement dimensions
"workflow-boundary agreement," "stage-boundary agreement," and
"representation agreement," defined in
`reference/proposal-next-iteration-icm-workflows.md` §9.9. Generated
artifacts: `docs/gauntlet/runs/n2-run2/workflows/repo-to-icm/{20-harvest,
30-normalize,40-classify,50-synthesize,60-draft}/output/*` and the three
materialized packages under
`docs/gauntlet/runs/n2-run2/drafts/workflows/{dispatch-mode,
standard-task-workflow,ship-with-no-mistakes}/`. Reference: `reference-
corpus/draft-workflows/*` and `reference-corpus/behavior-units/P{1,6}.ndjson`,
per `reference-corpus/FROZEN.md` v1.

## Scope discipline (read this before the numbers below)

Per `docs/gauntlet/runs/n2-run2/workflows/repo-to-icm/90-reconcile/output/
measurement-package.md`'s "Extraction coverage" note, this run's 108
behavior units (`BU-0001`–`BU-0108`) come from exactly **18 of 136**
`decompose`-dispositioned files: `AGENTS.md`, `README.md`, and all 14 files
of the `bin: fleet dispatch & lifecycle` partition (`bin/sgt-dispatch`,
`bin/sgt-dag-dispatch-hook`, `bin/sgt-dag-run`, `bin/sgt-cleanup`,
`bin/sgt-drain`, `bin/sgt-drain-force`, `bin/sgt-undrain`, `bin/sgt-recover`,
`bin/sgt-respond`, `bin/sgt-ack-response`, `bin/sgt-watch`, `bin/sgt-wake`,
`bin/sgt-interactive-worker`, `bin/sgt-notify`). 118 of 136 files/partition
members were never reached.

Every reference figure below is therefore filtered to the **same 18-file
scope** before any comparison is drawn — this document does not compare
108 generated units against 979 reference units. That filtered reference
slice is 165 units (`reference-corpus/behavior-units/P1.ndjson` +
`P6.ndjson`, `source.path` in the 18-file set, script at the end of this
note's working notes; not committed — reproducible by filtering
`source.path` against the file list above). Where a reference package's
final, adjudicated shape draws most of its evidence from files *outside*
this run's reached scope (true for `dispatch`, `worker-mission`, and most
of `validate-and-ship`), that is called out explicitly — it is a coverage
fact, not a quality finding, and is kept separate from the boundary
judgments below.

## 1. Package correspondence

| Generated candidate | Reference counterpart(s) | Correspondence basis |
|---|---|---|
| `dispatch-mode` | `dispatch` (primary); `worker-mission`, `task-intake-and-route` (adjacent — see §2) | `BU-0003`'s trigger/outcome text is a near-paraphrase of `BU-P1-003`/`BU-P6-123` |
| `standard-task-workflow` | `task-intake-and-route` (primary, strong — see §2.2); `direct-implementation`, `validate-and-ship` (partial, via the `40-validate` stage) | `BU-0011`/`BU-0012`/`BU-0013`/`BU-0016` statements literally read "standard task workflow, step 1/2/4/9," matching `task-intake-and-route`'s `BU-P1-025`–`BU-P1-034` one-for-one by step number |
| `ship-with-no-mistakes` | `validate-and-ship` (primary); `route-review-findings` (the `route-findings` unattached record) | `BU-0028`–`BU-0034` (all sourced `README.md`) match `validate-and-ship`'s `BU-P1-069`–`BU-P1-080` family (also `README.md`) almost line for line |

## 2. Workflow-boundary agreement

### 2.1 `dispatch-mode` — one workflow standing in for at least seven

`dispatch-mode`'s own provenance
(`drafts/workflows/dispatch-mode/provenance.md`) is explicit about this:
its single member stage, `10-dispatch-worker` (`BU-0041`, source
`bin/sgt-dispatch`), covers only the launch act. Its own "Uncovered
ground" section states plainly that `BU-0003`'s completion condition
("progress is monitored through to reconciliation of merge order and
cross-repo implications") has **no** corresponding stage evidence
anywhere in the 108-unit corpus, and the stage says so rather than
inventing one.

The reference's post-adjudication decomposition of the same underlying
procedure is not one workflow but a cluster: `dispatch` (6 stages:
`00-check-queue-and-plan`, `05-classify-risk`, `15-check-admission`,
`20-prepare-intent`, `80-monitor`, `90-reconcile-fleet`), plus six further
independent single-purpose packages built from the other 13 covered `bin/`
files this run also read but did not promote: `drain-fleet` (`bin/sgt-drain`,
`bin/sgt-drain-force`, `bin/sgt-undrain`), `recover-stalled-worker`
(`bin/sgt-recover`), `respond-to-worker` (`bin/sgt-respond`,
`bin/sgt-ack-response`, `bin/sgt-notify`), `wake-and-resume`
(`bin/sgt-wake`), `monitor-fleet` (`bin/sgt-watch`), and
`reconcile-and-cleanup-fleet` (`bin/sgt-cleanup`, `bin/sgt-watch`) — plus
`worker-mission`, which is entirely evidenced by `templates/worker-brief.md`
and test files outside this run's reached scope
(`reference-corpus/draft-workflows/worker-mission/provenance.md` cites zero
`AGENTS.md`/`README.md`/covered-`bin/` sources — genuinely unreachable from
this run's coverage, not a generator miss).

So: `dispatch`, `worker-mission`, and `task-intake-and-route` — the three
names the contract's framing names as `dispatch-mode`'s "family" — are
only part of the true reference cluster; the honest count within reached
scope is closer to seven reference packages standing where this run
produced one. Within `dispatch`'s own final shape, only 19 of its 69
provenance citation-rows (7 `AGENTS.md`, 5 `bin/sgt-interactive-worker`, 5
`bin/sgt-dispatch`, 2 `README.md`) fall inside this run's reached scope —
the other 50 (32 from `skills/dispatch/SKILL.md`, plus `docs/using-
sergeant.md`, `tests/*`, `schema/project.yaml.example`, `bin/_sgt-
intent.sh`, `bin/sgt-treehouse-init`, `bin/sgt-td-create`) are outside it
(counting citation-rows, not distinct behavior-unit ids — some units are
cited more than once across `dispatch`'s stages).
`dispatch-mode`'s single-stage shape is thus not a fair proxy for
`dispatch`'s own 6-stage shape even setting the six sibling packages
aside — most of what makes `dispatch` a 6-stage workflow was never in this
run's reach.

**Verdict:** genuine workflow-boundary under-recognition, but the larger
share of it is explained by coverage, not a boundary-identification
failure on the evidence this run actually held. What *is* attributable to
this run's own actors: `dispatch-mode` never separated "dispatch a worker"
from "operate/recover/reconcile an already-dispatched worker" even though
it read all 14 fleet-dispatch files that motivate that split in the
reference (see §3 for the representation-level version of this same gap).

### 2.2 `standard-task-workflow` — a strong, direct match to `task-intake-and-route`, blurred by one borrowed stage

This is the closest workflow-boundary match in the run. `task-intake-and-
route`'s workflow-level citation (`BU-P1-025`) states AGENTS.md's own
"nine-step standard workflow." `standard-task-workflow`'s own provenance
(`drafts/workflows/standard-task-workflow/provenance.md`) independently
recovers the same numbered structure from the same document: `BU-0011`
("step 1"), `BU-0012` ("step 2"), `BU-0013` ("step 4"), `BU-0016` ("step
9") — matching `task-intake-and-route`'s `01-load-context` (`BU-P1-026`,
step 1), the folded `02-check-queue` (`BU-P1-027`, step 2), the folded
`04-reconcile-state` (`BU-P1-029`, step 4), and `09-reconcile-deliver`
(`BU-P1-034`, step 9) respectively — same document, same step numbers, no
reference access. This is the run's best positive example of "the
generator found the same reusable outcome the reference did," not merely
a mirrored file boundary (`task-intake-and-route` itself draws on only
`AGENTS.md`, so there is no file-boundary trick available here — both
sides had to read the same nine-step list and independently choose to keep
its shape).

The blur: `standard-task-workflow`'s fifth stage, `40-validate` (`BU-0019`),
has no counterpart at all in `task-intake-and-route` — its trigger ("a
worker reaches readiness") and content ("the coordinator launches the
single validation-only boundary in a split pane of the worker's terminal
session") is `AGENTS.md` material the reference instead routes to
`validate-and-ship`'s `30-start-run`/`60-close-out` stages (compare
`BU-P1-042`/`BU-P1-043`, both cited there, both also `AGENTS.md`). The
`50-synthesize` stage's own recorded reasoning
(`workflows/repo-to-icm/50-synthesize/output/candidates.md`, quoted
verbatim in `standard-task-workflow/provenance.md`) admits it "cannot pin
an exact step number... this is a judgment call `80-adversarial-review`
should be free to challenge" — an honest flag, not a silent invention, but
the underlying fact stands: this candidate is a genuine three-way blend of
`task-intake-and-route`, `direct-implementation`, and `validate-and-ship`
territory rather than a clean match to any one of them, which is exactly
why the contract names all three as its reference counterpart set.

**Verdict:** the strongest workflow-boundary agreement in the run for the
five units that map onto `task-intake-and-route`'s own numbered steps;
one clear cross-workflow leak (`40-validate`) into `validate-and-ship`'s
territory, self-flagged rather than hidden.

### 2.3 `ship-with-no-mistakes` — zero stages, but real name-level convergence with `validate-and-ship`

`ship-with-no-mistakes` has no member stages at all — its provenance says
so directly: "No `behavior_id` in this corpus carries `representation:
stage` with `workflow: ship-with-no-mistakes`." Its seven grounding units
(`BU-0028`–`BU-0034`, all `README.md`) are `stage-context`, not `stage`.

What is notable is *which* stage names those seven records independently
name: `start-run` (`BU-0028`, `BU-0029`), `drive-gates` (`BU-0030`,
`BU-0031`), `finish-run` (`BU-0032`, `BU-0033`), `route-findings`
(`BU-0034`). Three of these four names land almost exactly on three of
`validate-and-ship`'s own post-adjudication stage names — `30-start-run`,
`40-drive-gates`, and (close enough in intent, if not spelling)
`60-close-out` — and the fourth, `route-findings`, is precisely the
content the reference spins out into its own separate package,
`route-review-findings` (`00-publish-or-clear-gate`, evidenced entirely by
`bin/sgt-review-findings` and its tests, which this run never read —
`route-review-findings/provenance.md` cites zero `README.md`/`AGENTS.md`
material). This is the run's second-best convergence example: from
`README.md` prose alone, and blind to the reference, the same three
checkpoint names emerged independently on both sides.

Set against that: `validate-and-ship`'s first three stages
(`00-check-scope`, `10-do-the-work`, `20-select-intent-transport`) have
**zero** citations from any of this run's 18 covered files — they are
built entirely from `.agents/skills/no-mistakes/SKILL.md`, `bin/sgt-
validate`, `docs/using-sergeant.md`, and tests, none of which this run
reached. `ship-with-no-mistakes` could not have produced these three
stages from what it read, and correctly did not try to invent them.

**Verdict:** name-level workflow-boundary agreement on the reachable third
of `validate-and-ship`'s final shape (`start-run`/`drive-gates`/
`close-out`, plus independently recovering `route-review-findings`'s
existence as a separate concern), with the other two-thirds of the
reference package simply unreachable from this run's coverage — a
coverage fact, not a boundary miss.

## 3. Stage-boundary agreement: over-staging vs the post-A4 standard

`docs/gauntlet/contracts/N1.md`'s adjudication round 1 (`reference-corpus/
adjudication-round1.md`, ruling A4) established a standing rule inside the
reference corpus: a stage whose only extraction justification is "§6.5
deterministic-machinery boilerplate" — no case-by-case "Additional note"
checkpoint argument — demotes and folds forward as a helper into the next
judgment-bearing stage. This rule was applied identically across multiple
reference packages, not once:

- `task-intake-and-route/provenance.md`, "Adjudication A4": the original
  nine-stage mirror of `AGENTS.md`'s nine numbered steps was cut to six by
  folding `02-check-queue` → `03-choose-mode`, `04-reconcile-state` →
  `05-confirm-decisions`, and `07-monitor` → `08-handle-decisions`.
- `direct-implementation/provenance.md`: `02-reconcile-existing-state`
  folds into `03-claim-and-implement` by the same rule.
- `route-review-findings/provenance.md`, "Adjudication A4": three of four
  original stages fold into the one stage that actually required
  judgment.

`standard-task-workflow` was generated **without** access to any of this,
and reproduces exactly the over-staged, pre-A4 shape the reference started
from and then cut down. Its own provenance keeps `20-check-queue`
(`BU-0012`) and `30-reconcile-existing-state` (`BU-0013`) as independent
top-level stages — the identical two checkpoints ("check queue," "reconcile
existing state") that the reference's own A4 ruling demoted in *both*
`task-intake-and-route` and `direct-implementation`, on the stated grounds
that they "carried only the §6.5 deterministic-machinery boilerplate as
their extraction justification." `standard-task-workflow`'s own
`80-adversarial-review` pass (`review-summary.md`, "Over-staging
(reimplementation test)") checked all 6 of this run's `stage` records
against the §6.3 reimplementation test and found "No over-staging" — but
that self-check is necessarily blind to the reference's specific A4
precedent; measured against that actual post-A4 standard rather than the
run's internal self-consistency, 2 of `standard-task-workflow`'s 5 stages
(`20-check-queue`, `30-reconcile-existing-state`) are over-staged by
exactly the standard the reference itself uses elsewhere in the same
corpus for the same two checkpoint types.

`dispatch-mode`'s single stage (`10-dispatch-worker`) and `standard-task-
workflow`'s other three stages (`10-load-context`, `40-validate`,
`50-reconcile-and-deliver`) show no equivalent over-staging — each names a
checkpoint the reference also keeps as a real stage after A4 (`dispatch`'s
own citation `BU-P6-123` for the launch act; `task-intake-and-route`'s
`01-load-context` and `09-reconcile-deliver` survive A4 unchanged).

There is no under-fragmentation finding to report in the opposite
direction (commands collapsed that should have stayed separate stages) —
the failure mode observed here runs entirely toward over-staging on
machinery-only checkpoints, never the reverse.

## 4. Representation agreement and the shared-helper skew

### 4.1 Distribution, same 18-file scope

| Representation | Generated (108 units) | Reference, filtered to same 18 files (165 units) |
|---|---:|---:|
| `agents-invariant` | 13 (12.0%) | 33 (20.0%) |
| `workflow` | 1 (0.9%) | 5 (3.0%) |
| `stage` | 6 (5.6%) | 34 (20.6%) |
| `stage-context` | 9 (8.3%) | 41 (24.8%) |
| `shared-context` | 0 | 22 (13.3%) |
| `shared-helper` + `helper` | 79 (73.1%) | 9 + 4 = 13 (7.9%) |
| `obsolete-mechanism` | 0 | 13 (7.9%) |
| `engine-gap` | 0† | 4 (2.4%) |

†At the behavior-unit level. The run separately recorded two meta-level
`engine_gap` records in `workflows/repo-to-icm/90-reconcile/output/
grammar-pressure.ndjson` (about the harvest turn-budget itself and the
finalize-disposition mechanism) — real, but about the workflow's own
mechanics, not a classified behavior unit.

Two numbers dominate the disagreement: `shared-helper` is roughly **9x**
overrepresented (73.1% vs 7.9%) and `stage`+`stage-context`, the two
judgment-bearing tiers, are collectively underrepresented by more than
half (13.9% generated vs 45.4% reference). `shared-context` and
`obsolete-mechanism` are entirely absent from the generated run within
this scope — this run never classified any unit as content shared *for
guidance* across workflows (as opposed to reused machinery), and never
flagged any piece of the reached source as an obsolete mechanism not worth
carrying forward.

### 4.2 The strongest disagreement example, side by side

The clearest single instance is `bin/sgt-recover`. Generated:

- `BU-0075` (source `bin/sgt-recover`): "Stall recovery for a worker
  requires proof of stall: the worker's status must be `in_progress` and
  its fleet diagnostic must carry a stall classification..."
  Representation: `shared-helper`. Rationale (`40-classify/output/
  classifications.ndjson`): "Gating recovery on both an `in_progress`
  status and a recorded stall diagnostic is a deterministic precondition
  check `sgt-recover` applies identically to every stalled worker,
  independent of caller workflow."

Reference, same file, essentially the same underlying fact:

- `BU-P6-071` (source `bin/sgt-recover`): "A stall recovery attempt is
  gated on concrete stall proof — status must be `in_progress` and the
  fleet diagnostic must begin with a stall-classification marker written
  by the watcher..." Representation: `stage`
  (`workflow: stall-recovery`, `stage: attempt-recovery`). Rationale: "A
  meaningful, bounded, exactly-once durable checkpoint operators would
  want measured (was recovery attempted? did it succeed or escalate?) —
  matches §6.3."

Both sides are applying the *same* documented method
(`.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` §6.3's
reimplementation test — "would the procedural checkpoint still exist
under a different implementation?"), to the same sentence, in the same
file. The reference answers §6.3's own question directly ("operators would
want measured," "would survive reimplementation"). The generated rationale
answers a different, adjacent question — §6.5's "is this deterministic
machinery subordinate to some other checkpoint's outcome" — without first
establishing that a checkpoint exists to be subordinate to. The same
pattern repeats at:

- `BU-0079` (`bin/sgt-respond`, `shared-helper`, "deterministic status
  guard... applies identically to every delivery attempt") vs `BU-P6-078`
  (`bin/sgt-respond`, `stage`, `respond-and-resume`/`validate-target-
  status`, "a durable checkpoint... whose semantics would survive
  reimplementation — matches §6.3").
- `BU-0090` (`bin/sgt-watch`, `shared-helper`, "deterministic monitoring
  machinery... applies the same way to every live in_progress worker") vs
  `BU-P6-101` (`bin/sgt-watch`, `stage`, `fleet-observation`/`snapshot`,
  "a meaningful, durable, three-valued checkpoint... any bridge or
  coordinator would want measured identically — matches §6.3").
- `BU-0064` (`bin/sgt-drain`, `shared-helper`) and its sibling `BU-0071`/
  `BU-0072` (`bin/sgt-drain-force`, `bin/sgt-undrain`, `shared-helper`) vs
  `BU-P6-039` (`bin/sgt-drain-force`, `stage`, `drain-admission-control`/
  `force-stop`, "a destructive, human-authorized checkpoint operators
  would explicitly want to observe/gate") and `BU-P6-015`
  (`bin/sgt-undrain`, `stage`, `drain-admission-control`/`undrain`, "a
  meaningful durable checkpoint... whose idempotency guarantee operators
  would care about").

Downstream, this single classification choice reshapes the whole `50-
synthesize` output: bucket 5 (`shared-helper`, 79 records) groups its
records almost one-for-one by originating script — `candidates.md`'s own
accounting lists 15 named helper groups (`dispatch-contract`, `dag-
dispatch-hook`, `dag-run`, `cleanup`, `drain`, `drain-force`, `undrain`,
`recover`, `respond`, `ack-response`, `watch`, `wake`, `interactive-
worker`, `notify`, `findings-router`) against 14 covered `bin/` files.
Once the individual behaviors were classified as machinery rather than
checkpoints, the grouping step re-derives file-shaped clusters almost
exactly (`respond`: `BU-0079`–`BU-0084`, all six from `bin/sgt-respond`;
`drain`: `BU-0064`–`BU-0070`, all seven from `bin/sgt-drain`) — the same
"merely mirror source file boundaries" failure mode §9.9 names for
workflow-boundary agreement, reappearing one representation-tier down,
downstream of (not instead of) the shared-helper misclassification.

### 4.3 Where representation agreement holds

Not every judgment call diverged. All 6 `stage` records this run did
produce name checkpoints the reference also treats as stages after A4
(§3, above), and the run's own citation-and-rationale discipline held up
under its own `80-adversarial-review` re-check (29/29 sampled citations
hash-verified, 16/16 provenance citations confirmed to exist,
`review-summary.md` Axis 2) — the representation *category* chosen for
the 6 `stage` and 1 `workflow` records this run did surface was not
merely asserted, and matches the reference's category for the same
checkpoints where both sides reached the same source content
(`AGENTS.md`'s numbered steps, `bin/sgt-dispatch`'s launch description).
The disagreement in §4.1-4.2 is concentrated almost entirely in the
`shared-helper` tier — the 13 non-`dispatch` fleet-lifecycle scripts — not
spread evenly across the corpus.

## 5. Summary

- **Workflow-boundary agreement**: one strong direct match
  (`standard-task-workflow` ↔ `task-intake-and-route`'s numbered steps,
  §2.2), one partial name-level match on the reachable third of a larger
  reference package (`ship-with-no-mistakes` ↔ `validate-and-ship`'s
  `start-run`/`drive-gates`/close-out family plus independent recovery of
  `route-review-findings`'s existence, §2.3), and one case where a single
  generated workflow stands in for what the reference resolves into at
  least seven packages, most of that gap explained by unreached source
  files rather than a boundary-identification failure on what was read
  (`dispatch-mode`, §2.1).
- **Stage-boundary agreement**: no under-fragmentation found;
  over-staging found on exactly the two checkpoint types
  (`check-queue`, `reconcile-existing-state`) the reference's own A4
  ruling independently demoted in two separate packages built from the
  same source document (§3). The run's internal self-check ("no
  over-staging found") is accurate to its own blind method and does not
  detect this — it can only be seen by comparison against the reference's
  actual precedent.
- **Representation agreement**: the dominant finding. `shared-helper` is
  ~9x overrepresented (73.1% vs 7.9% in the same 18-file scope) because
  the classify stage answered §6.5's subordinate-machinery question on
  fleet-lifecycle scripts (`sgt-recover`, `sgt-respond`, `sgt-watch`,
  `sgt-drain*`, `sgt-undrain`) without first clearing §6.3's own
  checkpoint question, which the reference answers "yes" for the same
  content using the same documented reimplementation test (§4.2). This
  also drives `stage`+`stage-context` to less than a third of the
  reference's combined share (13.9% vs 45.4%) and leaves `shared-context`
  and `obsolete-mechanism` entirely unused in this run's output.

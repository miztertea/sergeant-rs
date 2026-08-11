# Draft report — `60-draft`

Records what this run of `60-draft` produced, per its own `CONTEXT.md` step
6. `../../50-synthesize/output/candidates.md` did not open with
`# AMBIGUOUS — NOT RESOLVED`, so this stage proceeded with its ordinary
work.

**Coverage caveat (added by `90-reconcile`, adjudication of AF-0003).** The
18 candidates materialized below are drafted from the complete 312-record
classification ledger this stage was handed, but that ledger itself covers
only 28 of the 82 `decompose`-dispositioned files
`../../10-inventory/output/inventory.md` scoped in (6 of 21 partitions —
`../../20-harvest/output/partition-ledger.md` records the other 15 as
`pending`). These 18 packages are consequently a decomposition of roughly
one-third of the subject repository, not the repository as a whole. See
`../../90-reconcile/output/adjudication-log.md` (AF-0001/AF-0002/AF-0003)
and `../../90-reconcile/output/measurement-package.md` for the full
accounting.

---

## Manifest: materialized draft workflow packages

All 18 workflow candidates from `../../50-synthesize/output/candidates.md`
Buckets 1–3 are materialized as packages under
`.sergeant/drafts/workflows/`, in this run's own worktree, per
`references/draft-package-template.md`. Every candidate name was re-checked
against `.sergeant/workflows/` (only `repo-to-icm` itself), against
`.sergeant/drafts/workflows/` as each package was written, and against every
other candidate name from this run — no collisions found (`50-synthesize`
had already checked the same set with the same result).

| # | Candidate | Path | Stages materialized |
|---|---|---|---|
| 1 | `dispatch-worker` | `.sergeant/drafts/workflows/dispatch-worker/` | 6 |
| 2 | `task-intake-and-execution` | `.sergeant/drafts/workflows/task-intake-and-execution/` | 8 |
| 3 | `worker-response-and-recovery` | `.sergeant/drafts/workflows/worker-response-and-recovery/` | 2 |
| 4 | `validation-gate` | `.sergeant/drafts/workflows/validation-gate/` | 2 |
| 5 | `project-registration` | `.sergeant/drafts/workflows/project-registration/` | 1 |
| 6 | `fleet-monitor-and-reconcile` | `.sergeant/drafts/workflows/fleet-monitor-and-reconcile/` | 0 |
| 7 | `shipping-gate-driving` | `.sergeant/drafts/workflows/shipping-gate-driving/` | 1 |
| 8 | `review-finding-routing` | `.sergeant/drafts/workflows/review-finding-routing/` | 2 |
| 9 | `installation-and-setup` | `.sergeant/drafts/workflows/installation-and-setup/` | 1 |
| 10 | `project-graphify` | `.sergeant/drafts/workflows/project-graphify/` | 1 |
| 11 | `fleet-cleanup` | `.sergeant/drafts/workflows/fleet-cleanup/` | 1 |
| 12 | `dag-orchestration` | `.sergeant/drafts/workflows/dag-orchestration/` | 1 |
| 13 | `callback-delivery` | `.sergeant/drafts/workflows/callback-delivery/` | 0 |
| 14 | `skill-adoption` | `.sergeant/drafts/workflows/skill-adoption/` | 0 |
| 15 | `sergeant-help-query` | `.sergeant/drafts/workflows/sergeant-help-query/` | 0 |
| 16 | `troubleshoot-td-identity` | `.sergeant/drafts/workflows/troubleshoot-td-identity/` | 0 |
| 17 | `cross-repo-planning` | `.sergeant/drafts/workflows/cross-repo-planning/` | 0 |
| 18 | `undocumented-failure-escalation` | `.sergeant/drafts/workflows/undocumented-failure-escalation/` | 0 |

18/18 candidates materialized. 26 `NN-*/` stage directories total, across
the 11 candidates that have any (`dispatch-worker` 6 +
`task-intake-and-execution` 8 + `worker-response-and-recovery` 2 +
`validation-gate` 2 + `project-registration` 1 + `shipping-gate-driving` 1 +
`review-finding-routing` 2 + `installation-and-setup` 1 +
`project-graphify` 1 + `fleet-cleanup` 1 + `dag-orchestration` 1 = 26).
These 26 stage directories carry 31 `representation: stage` `behavior_id`s
between them (some stages, e.g. `record-canonical-intent`, cite more than
one `behavior_id` — matching the 31-record total `../../50-synthesize/
output/candidates.md` reports for bucket 1–3's `stage` rung). Each
package's own `provenance.md` cites the exact `behavior_id`(s) backing
every stage and the workflow as a whole; each stage's own
`output/README.md` declares that candidate's own future-run artifact shape
(unpopulated, per `references/draft-package-template.md` "What never
happens").

### Method notes on two judgment calls this stage made

**Zero-stage candidates (7 of 18).** `fleet-monitor-and-reconcile`,
`callback-delivery`, `skill-adoption`, `sergeant-help-query`,
`troubleshoot-td-identity`, `cross-repo-planning`, and
`undocumented-failure-escalation` have no `representation: stage` record in
`../../40-classify/output/classifications.ndjson` carrying their `workflow`
value. Per `../../_config/icm-ladder.md` bucket 3 and this run's own
discipline against inventing checkpoints, these packages are materialized
with zero `NN-*/` stage directories and `workflow.toml` declaring
`stages = []`, rather than inventing a stage to give the package a
conventional shape. Each such package's own `CONTEXT.md` states this
plainly under "Zero materialized stages" and its `provenance.md` traces the
workflow boundary to its supporting `stage-context`/`helper`/`workflow`
evidence instead.

**`recover-worker` (worker-response-and-recovery).** Five stage-context
records (BU-0039, BU-0146, BU-0159, BU-0174, BU-0286) all name
`stage: recover-worker` — a checkpoint clearly operator-visible in a
workflow whose own name is "worker-response-**and-recovery**" — yet no
`representation: stage` record for it exists anywhere in the 312-record
corpus. `50-synthesize` flagged this as the single sharpest instance of the
corpus's unattached-stage-context pattern
(`../../50-synthesize/output/candidates.md` §"Unattached records"). This
stage does not resolve it by inventing a `recover-worker` stage directory;
it is recorded in `.sergeant/drafts/workflows/worker-response-and-recovery/provenance.md`
and named again here for `90-reconcile`.

---

## Carried through from `../../50-synthesize/output/candidates.md`, unedited

Per this stage's own `CONTEXT.md`: "Permanent-instruction, obsolete-mechanism,
and engine-pressure candidates from `50-synthesize` are **not** materialized
as packages (they are not workflows) — they are carried forward in this
stage's own `output/draft-report.md` instead, for `90-reconcile` to use."
The three sections below are copied verbatim from
`../../50-synthesize/output/candidates.md` (this stage clusters and
packages; it does not edit synthesis's own evidence).

### Bucket 4: permanent-instruction candidates

106 `agents-invariant` records. Listed, not drafted into any workflow
package — `AGENTS.md` changes are the promotion reviewer's call, not this
run's (`references/synthesis-method.md` bucket 4). Grouped below by source
file purely for scan-ability; this grouping is not itself a candidate
boundary.

**AGENTS.md** (29): BU-0001 (resolve context via context-resolution command
before acting), BU-0002 (ownership never inferred from cwd), BU-0003
(primary session coordinates multi-repo work by default), BU-0004 (direct
implementation only on explicit single-repo request), BU-0005 (dispatch
mode conditions), BU-0009 (direct mode requires explicit request + one
owning repo), BU-0013 (default branch never edited directly), BU-0017
(coordinator role never used to stop short of an implemented outcome),
BU-0018 (direct mode never spans repos or bypasses ownership/review/gates),
BU-0019 (toolbelt command used over ad hoc shell when it covers the
operation), BU-0022 (repository-local SKILL.md is canonical over a
registry skill), BU-0023 (registry omission doesn't make a skill
unavailable), BU-0024 (stop and report the exact missing skill path, don't
reconstruct from memory), BU-0027 (choose direct vs. dispatch mode by
scope), BU-0029 (ask the user only for scope/risk-changing unresolved
decisions), BU-0030 (never re-ask an already-recorded decision), BU-0032
(monitoring requires recent meaningful events, not parent-process
liveness), BU-0036 (`in_progress`/`needs_input`/`blocked`/`waiting` are
nonterminal), BU-0038 (progress never inferred from liveness alone; a
waiting worktree is never cleaned), BU-0041 (workers/remediation loops
never run no-mistakes themselves), BU-0044 (a plan/task/finding/launch is
not the requested outcome unless planning/dispatch was all that was asked),
BU-0045 (an approved known blocker isn't re-reported), BU-0046 (no
duplicate tasks/findings/PRs/workers when a canonical owner exists), BU-0047
(a worker isn't "active" from process/pane existence alone), BU-0048 (td
and fleet state reconciled truthfully, never left stale in_progress), BU-0049
(tool absence produces an actionable fallback, never silent skip/false
success), BU-0050 (standing authorization never covers risk acceptance,
gate-skipping, force ops, secret exposure, or destroying preserved state),
BU-0054 (repos under Sergeant's config dir are never modified), BU-0055
(secrets never committed to project YAMLs).

**README.md** (12): BU-0061 (fleet-watch/bulk-reconcile may kill panes —
not safe for observe-only callers), BU-0067 (every model transport is
measured on an installed harness, never inferred from docs), BU-0073 (a
resumed/recovered worker inherits its model pin; an unhonorable tuple fails
terminally), BU-0074 (model resolved only from flag/env/unpinned-default —
no project-level default), BU-0078 (the coordinator pane's reader displays
lines, never executes them), BU-0079 (no-mistakes is a final shipping gate,
not an implementation loop), BU-0081 (no blanket auto-approval; skip only
proven-irrelevant stages), BU-0085 (a pipeline-owned worktree is never
edited mid-run; commits preserved), BU-0086 (driving stops at
checks-passed; no polling for merge), BU-0088 (no improvised
reset/stash/force-push/branch-replace around a blocked sync), BU-0089 (a
no-mistakes run is validation-only; findings routed to separate td tasks),
BU-0094 (correctness/security/data-integrity/test findings can't be
deferred; cosmetic/evidence-only findings never create cards).

**bin/sgt-sync** (1): BU-0241 (fast-forward-only pull; skip with a warning
on detached HEAD or failed fast-forward, never force-merge).

**docs/README.md** (3): BU-0106 (documentation authority is layered by
ownership across AGENTS.md/SKILL.md/schema.md/user docs), BU-0107
(`--help`/tests/tested behavior wins over prose; file a task on
disagreement), BU-0108 (no real credentials/private repo names/prompt or
response bodies/secrets in documentation examples).

**docs/callbacks.md** (9): BU-0214 (a callback profile executable must be
real, owner-owned, non-group/world-writable, owner-executable), BU-0215 (an
env override for the callbacks directory is trusted local config, never
request input), BU-0217 (origin registration stores only
correlation_id/profile/version, never request/secret content), BU-0221
(payload shape/size/content constraints, rejecting shell metacharacters and
platform-ID-shaped strings), BU-0222 (fixed profile invoked with minimal
env, stderr discarded, one compact JSON object on stdin), BU-0223 (the
consumer durably dedups by idempotency_key), BU-0226 (consumer stderr/
output details never persisted), BU-0229 (producers make one bounded
attempt and don't wait indefinitely; events survive restarts), BU-0234
(the ws-lab hermes-discord consumer's specific forwarding/dedup/credential-
isolation contract).

**docs/getting-started.md** (2): BU-0130 (no harness-specific
conversation-injection plugins; updates surface via fleet-watch), BU-0134
(the coordinator starts from the Sergeant checkout inside tmux).

**docs/repo-scoped-skills.md** (1): BU-0122 (workers never invoke
no-mistakes directly; the shipping-gate-pipeline skill is vendored only for
understanding the coordinator-owned contract).

**docs/schema.md** (2): BU-0194 (callback implementations are executable
profiles under the local directory, never project YAML fields), BU-0199
(a `graphify.output` inside a source repo is staged/excluded so it's never
re-ingested).

**docs/skills.md** (4): BU-0116 (skill provenance never inferred from
folder name), BU-0117 (the Claude plugin route installs a managed
read-only bundle, never hand-edited), BU-0118 (every directive needs a
trigger/action/prohibition/evidence/stop-condition, not a slogan), BU-0120
(Sergeant-owned skills updated only via reviewed PR + instruction-policy
test + full suite).

**docs/troubleshooting.md** (9): BU-0172 (supported commands used before
manual process/tmux/Git/fleet-file operations; evidence preserved first),
BU-0175 (a Sergeant recovery refusal is respected, not force-retried),
BU-0176 (an expected dependency-blocked exit stays blocked, not
reclassified as orphaned), BU-0180 (an auto-fix finding is never
authorized in-run; routed to owning-repo remediation), BU-0181 (shared
daemon credentials are never globally switched while other runs may use
them), BU-0182 (one-shot scoped credentials preferred over a global account
switch), BU-0183 (parse-only Bash 3.2 proof doesn't replace runtime proof
unless acceptance explicitly permits it), BU-0185 (fleet files are never
force-deleted/manually edited when cleanup refuses), BU-0191 (cleanup
requires fleet state and worktree on the same filesystem for atomic
rename).

**docs/using-sergeant.md** (8): BU-0136 (workers always run as persistent
interactive TTY sessions, never one-shot modes), BU-0138 (the
interactive-permission-UI bypass flag is scoped by dispatch-time trust, not
a capability grant), BU-0139 (the actual trust boundary is the intent file
+ worker brief + worktree permissions, not the bypass flag), BU-0142 (bulk
reconciliation preserves needs_input/blocked/orphaned worktrees), BU-0147
(`orphaned` means the expected supervisor identity vanished without a
durable waiting state), BU-0152 (`human_response` never auto-resumes;
`deployment` also escalates to needs_input until an adapter exists), BU-0153
(a drain refuses new pane starts for its scope while storing responses
generation-safely), BU-0170 (validation is validation-only; source is never
modified inside a retained run).

**docs/what-is-sergeant.md** (7): BU-0109 (one developer per installation,
not a shared team service), BU-0110 (no central tenancy, org RBAC, shared
credentials, cross-machine leases, or team-wide fleet DB), BU-0111 (a live
process is not proof of progress), BU-0112 (a decision request requires a
human product/security/privacy/destructive/risk decision), BU-0113 (direct
mode still requires task/TDD/checks/review/validation/handoff), BU-0114
(Sergeant is not permission to push directly to default branches), BU-0115
(a worker isn't healthy from process existence alone; a plan/task/launch/
finding isn't delivered work).

**mise.toml** (1): BU-0213 (update pulls fast-forward-only, then
reinstalls symlinks).

**skills/cross-repo-work/SKILL.md** (5): BU-0268 (ask about ownership only
when two repos could legitimately own a contract), BU-0269 (dependency
cycles rejected before dispatch; break via a contract artifact or
compatibility phase), BU-0270 (repo state never
stashed/reset/switched/cleaned during cross-repo planning), BU-0271
(planning-only requests stop after briefs/evidence/dependency graph, no
dispatch or edits), BU-0272 (a cross-repo outcome isn't complete until
every owning repo reaches terminal or has a preserved blocker).

**skills/dispatch/SKILL.md** (3): BU-0274 (`needs_input`/`blocked` are
distinct from `in_progress`-while-waiting), BU-0276 (a worker isn't done
until its dependency gate is satisfied), BU-0277 (a fleet isn't reconciled
merely because every worker opened a PR).

**skills/load-project/SKILL.md** (3): BU-0259 (project YAMLs never contain
credentials/tokens/secrets), BU-0264 (generated graph output is never
published inside an owning source repo), BU-0265 (a missing required
executable is reported plainly, no invented fallback parser).

**skills/sergeant-help/SKILL.md** (3): BU-0123 (sergeant-help is never a
substitute for load-project/cross-repo-work/dispatch/wiki once execution is
requested), BU-0126 (the skill states when a behavior is undocumented
rather than inventing one), BU-0127 (destructive operations kept out of
examples unless documented and explicitly requested).

**templates/worker-brief.md** (4): BU-0304 (non-interactive one-shot launch
modes are prohibited for a worker), BU-0308 (only allowlisted wake-condition
field names/values are accepted, never arbitrary commands/secrets), BU-0310
(the worker never approves a validation gate or routes a finding itself),
BU-0311 (a finding artifact carries only its fixed field set, never
prompts/secrets/credentials).

### Bucket 6: obsolete-mechanism findings

None. Zero classification records in `../../40-classify/output/
classifications.ndjson` carry an `obsolete-mechanism` disposition. This
bucket is empty by the evidence in hand, not by omission — checked directly
against the full 312-record ledger.

### Bucket 7: engine-pressure candidates

2 `engine-gap` records, `engine_gap` objects carried through unchanged per
method (this stage clusters and lists, it does not edit engine-gap
evidence).

#### BU-0137 — exactly-once crash-safe mission delivery

```
behavior: Exactly-once, crash-safe delivery of a worker's mission/brief to
  a persistent interactive session, surviving delayed startup or a
  coordinator crash, without ever exposing the brief via process arguments.
source_evidence: [BU-0137]
lower_rungs_attempted: [6.1 agents-invariant, 6.2 workflow, 6.3 stage,
  6.4 stage-context, 6.5 helper, 6.6 shared-helper]
why_each_fails:
  - 6.1 fails: this is delivery machinery tied to one moment (mission
    handoff at launch), not a policy that stands independent of any
    procedure's stage.
  - 6.2 fails: it has no independently invocable trigger/outcome of its
    own — it only exists nested inside worker launch, never started or
    finished on its own.
  - 6.3 fails: the checkpoint that matters (worker launched, mission
    delivered) is already the dispatch stage (BU-0007); this behavior is
    about surviving a crash *during* that handoff, not a separate boundary
    operators enter/exit.
  - 6.4 fails: the retry-until-acknowledged protocol is fixed, not an
    actor judgment call.
  - 6.5 fails: a helper script has no durable state of its own that
    outlives the very process that might crash mid-handoff — the guarantee
    needs somewhere durable to record "delivered, acknowledged, generation
    N" that a helper alone can't own.
  - 6.6 fails: sharing the helper's code across workflows doesn't fix the
    underlying problem — each caller would still need the runtime to hold
    the durable ack state, not just share the delivery logic.
minimum_runtime_capability_required: A durable, idempotent "mission handed
  off and acknowledged exactly once" fact per worker turn that the runtime
  itself appends and can prove, independent of whatever process is doing
  the delivering.
observable_acceptance_test: Kill the coordinator immediately after a
  mission is sent but before acknowledgement, restart it, and confirm the
  worker receives the mission exactly once (never zero, never twice) and
  no process listing ever shows the mission body.
```

#### BU-0227 — durable claim/lease for callback delivery

```
behavior: Durable, safe retry of at-least-once external delivery to a
  callback consumer under concurrent producers: a claim-with-lease so a
  crashed delivery attempt doesn't permanently stall the event or get
  double-delivered, exponential backoff on failure, and a bounded
  per-drain batch size.
source_evidence: [BU-0227]
lower_rungs_attempted: [6.1 agents-invariant, 6.2 workflow, 6.3 stage,
  6.4 stage-context, 6.5 helper, 6.6 shared-helper]
why_each_fails:
  - 6.1 fails: this is retry/lease machinery for one delivery attempt, not
    a rule that holds independent of any procedure's stage.
  - 6.2 fails: it has no independently invocable trigger/outcome — it
    exists only as part of draining the callback queue, never started or
    finished on its own by an operator.
  - 6.3 fails: the boundary operators actually care about ("this callback
    event was delivered and acknowledged") is a different, already-nameable
    checkpoint; the claim/lease/backoff/batch mechanics are about how
    concurrent drain attempts don't corrupt or duplicate that outcome,
    which is a property spanning many delivery attempts, not one
    checkpoint's entry/exit.
  - 6.4 fails: none of this is actor judgment — the lease timeout and
    backoff schedule are fixed parameters, not a decision an actor makes.
  - 6.5 fails: a helper script invoked per drain has no durable claim
    state that survives across separate invocations from separate
    processes; the guarantee requires somewhere durable that outlives any
    single helper execution to hold "who currently owns this event's
    delivery attempt, since when."
  - 6.6 fails: sharing the same helper code across workflows doesn't solve
    it either — each caller still needs the runtime, not the helper, to
    own the durable lease table so concurrent drains from different
    callers don't double-claim the same event.
minimum_runtime_capability_required: A durable, runtime-owned lease/claim
  primitive per outbound event (who holds it, since when, with a timeout
  after which it's reclaimable) plus a durably recorded attempt/backoff
  count, so concurrent drain invocations never deliver the same event
  twice and a crashed attempt is automatically reclaimed rather than
  stalling forever.
observable_acceptance_test: Start two drain invocations concurrently
  against the same pending event, kill one mid-delivery after it claims
  the event, and confirm the event is picked up and delivered exactly once
  by the survivor after the lease timeout, never delivered twice and never
  stuck.
```

**Overlap note** (not forced into a merge — the events they guard differ:
mission handoff at worker launch vs. an outbound callback queue). Both
independently arrive at the same shape of minimum capability: a durable,
runtime-owned claim/lease-and-acknowledge primitive over one delivery
attempt that survives a crash mid-delivery. If the runtime ever gains one
general-purpose "durable claim + ack" primitive, both BU-0137 and BU-0227
plausibly resolve against the same underlying mechanism — worth flagging
for whoever eventually specs `minimum_runtime_capability_required` work,
not something this stage collapses into a single candidate.

---

## Engine-gap grammar pressure (this stage's own finding)

**Meta-level grammar pressure, recorded for `90-reconcile`.** The
materialized packages under `.sergeant/drafts/workflows/` are this run's
principal deliverable, yet the D9 disposition/finalize mechanism
(`docs/icm/convention.md` §1a) only governs a stage's own `output/` — it has
no lower-rung way to give per-run content written *elsewhere* in the
worktree (the draft packages) a disposition, or to bring it under
`../../scripts/finalize.py`'s reach. This is a genuine could-not-express
moment: `references/draft-package-template.md`'s own package shape requires
writing durable per-run content outside `output/`, and the convention this
run operates under has no vocabulary for that case. `90-reconcile` is
positioned to write the full six-field `engine-gap` template from this
recorded moment, per `../../_config/run-discipline.md` and
`../../90-reconcile/references/reconciliation-method.md` §3 — this stage
only states the pressure plainly, it does not attempt the template itself.

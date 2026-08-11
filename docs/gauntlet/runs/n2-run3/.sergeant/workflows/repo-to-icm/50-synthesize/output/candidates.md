# Synthesis candidates

Clusters `../40-classify/output/classifications.ndjson` (312 records) into the
seven buckets defined by `references/synthesis-method.md`, per its method.
`../40-classify/output/classifications.ndjson` did not open with
`# AMBIGUOUS — NOT RESOLVED`, so this stage proceeded with its ordinary work.

**Accounting.** Every one of the 312 classification records appears in
exactly one bucket appearance below. Buckets 1–3 (workflow/stage/
stage-context) together hold 194 records (31 `stage` + 73 `stage-context` +
87 `helper` + 3 `workflow`); a `stage`/`stage-context` record counts once,
whether attached to a stage or listed under **Unattached records**. Bucket 4
(`agents-invariant`) holds 106. Bucket 5 (`shared-helper`/`shared-context`)
holds 10. Bucket 6 (`obsolete-mechanism`) holds 0 — none exist in this
corpus. Bucket 7 (`engine-gap`) holds 2. 194+106+10+0+2 = 312. A full
per-bucket count table closes this document.

**Coverage caveat (added by `90-reconcile`, adjudication of AF-0003).** The
312 records above are the complete corpus this stage was handed, but that
corpus itself covers only 28 of the 82 `decompose`-dispositioned files
`../10-inventory/output/inventory.md` scoped in (6 of 21 partitions —
`../20-harvest/output/partition-ledger.md` records the other 15 as
`pending`). Every count in this document is accurate *over the 28-file
corpus actually harvested*, not over the full subject repository. See
`../90-reconcile/output/adjudication-log.md` (AF-0001/AF-0002/AF-0003) and
`../90-reconcile/output/measurement-package.md` for the full accounting.

---

## Buckets 1–3: workflow candidates, their stages, and stage-context attachments

Eighteen distinct `workflow` values appear across the corpus's `stage`,
`stage-context`, and `helper` records (a `representation: workflow` record's
own `workflow` field is always `null` — it does not name itself — so three
of the eighteen are identified by matching a `workflow`-rung record's topic
against a `workflow` value seen on other records: `skill-adoption` against
BU-0119, `sergeant-help-query` against BU-0124; the eighteenth,
`undocumented-failure-escalation`, is BU-0192's own topic and matches no
other record's `workflow` field at all — a genuine single-behavior
workflow). All eighteen names were checked against each other, against
`.sergeant/workflows/` (only `repo-to-icm` itself exists), and against
`.sergeant/drafts/workflows/` (empty) — no collisions.

Several workflows below have stage-context checkpoints with no matching
`stage` record, or (three of them) zero `stage` records at all. Per bucket 3,
these are not resolved by inventing a stage — they are named plainly under
**Unattached records** at the end of this section, together with a plain
count: 54 of the corpus's 73 `stage-context` records (74%) attach to no
`stage` record with the same `workflow`+`stage`. Three workflows —
`fleet-monitor-and-reconcile`, `troubleshoot-td-identity`,
`cross-repo-planning` — have *zero* `stage` records despite having
`stage-context` content, and one — `callback-delivery` — has neither `stage`
nor `stage-context` records, only `helper`. This is the single largest
finding this stage surfaces; see the note after the workflow list.

### 1. dispatch-worker

**Description.** Trigger: dispatch mode has been selected for a task, and
work has been decomposed by owning repository. Outcome: each owning
repository ends up with a durably launched, evidence-backed worker running
under one stable canonical intent — or dispatch fails closed before
mutating anything. Completion: every target repo either has a spawned
worker with recorded launch evidence and a generation-tracked gate identity,
or the dispatch aborted with no partial state left behind.

**Member stages** (ordered — trigger→outcome chaining across
`../30-normalize/output/behavior-units.normalized.ndjson`; the middle three
are genuinely close-run since intent recording, the td-task precondition,
and the intent-file gate all sit "before mutation," so the order below is a
judgment call, not a hard chain):

1. **intent-file-gate** — BU-0140. Gates *any* mutating dispatch action
   behind a validated intent file when the objective touches a sensitive
   category — the earliest possible checkpoint, since it must run before
   task creation or worker spawn.
2. **create-td-tasks** — BU-0284. All-or-nothing td task creation across
   selected repos, with rollback on partial failure, explicitly *before any
   worker is spawned*.
3. **record-canonical-intent** — BU-0040, BU-0135, BU-0303. BU-0135's
   trigger ("a dispatch is created") places initial recording here, after
   the two preconditions above and before spawn; BU-0040 and BU-0303
   describe the same intent staying stable and governing every later
   dispatched action (implementation, review, PR, successor, recovery,
   shipping-gate), which is this stage's ongoing property, not a one-time
   event, but its establishment belongs at dispatch-creation time.
4. **spawn-worker** — BU-0007, BU-0295. The actual launch: BU-0007's
   trigger is "work has been decomposed by repository," producing one
   dispatched worker per repo; BU-0295 covers the four converging
   spawn-failure paths.
5. **escalate-undecided-seam** — BU-0281. Triggered while a worker is
   already running and needs to establish an undecided public behavioral
   seam — necessarily after spawn.
6. **report-terminal-status** — BU-0283. Triggered when a worker reaches a
   terminal outcome — the last checkpoint in the sequence.

**Stage-context attached:**
- spawn-worker: BU-0071 (launch evidence never overclaims model
  readiness), BU-0072 (launch evidence never overclaims variant
  verification).

**Workflow-local helpers** (22, grouped by function — not sequenced,
per method):
- Model/variant resolution & validation: BU-0058, BU-0065, BU-0066,
  BU-0068, BU-0069, BU-0197.
- td task creation mechanics: BU-0273, BU-0285, BU-0290, BU-0297, BU-0298.
- Worktree/treehouse setup: BU-0278, BU-0291, BU-0292, BU-0293, BU-0299.
- Coordinator-pane / dispatch-invocation plumbing: BU-0287, BU-0288,
  BU-0296.
- Cross-repo dependency & credential handling: BU-0279, BU-0294.
- Gate identity publishing: BU-0307.

14 stage-context records here are unattached — see **Unattached records**.

### 2. task-intake-and-execution

**Description.** Trigger: a task is brought to a Sergeant session. Outcome:
the task reaches a durably recorded terminal/deliverable state through the
mode-appropriate execution path, with evidence-preserving cleanup only after
that state is verified. Completion: `reconcile-and-deliver` confirms
terminal state and preserved evidence before any cleanup runs.

**Member stages** (ordered; **note an ordering call**: `execute` and
`direct-mode-implementation` are both real `stage` classifications with
overlapping territory — `execute`'s trigger, "decisions have been
confirmed," reads as the generic step from the standard workflow, while
`direct-mode-implementation`'s trigger, "direct mode is active," is the
same moment narrowed to direct mode specifically. They are kept as
distinct, adjacent stage candidates rather than merged, since the
classification records are separate and neither one's `rationale` treats
the other as redundant):

1. **resolve-context** — BU-0025. Trigger: a task is brought to the
   session. Outcome: context is fully loaded before an execution mode is
   chosen.
2. **resolve-task** — BU-0026. Trigger: context has been loaded. Outcome:
   an existing canonical td task is reused, or a new one created only
   otherwise.
3. **reconcile-before-start** — BU-0028. Trigger: an execution mode has
   been chosen, before starting work. Outcome: existing state is
   reconciled and reused rather than duplicated.
4. **execute** — BU-0031. Trigger: decisions have been confirmed. Outcome:
   execution proceeds via the mode-appropriate path.
5. **direct-mode-implementation** — BU-0012. Trigger: direct mode is
   active. Outcome: the owning td task is claimed/created and
   implementation proceeds test-first.
6. **handle-decision-gate** — BU-0034. Trigger: a worker reaches
   `needs_input`, `blocked`, or an ask-user gate. Outcome: only genuinely
   missing decisions are solicited and recorded in td; remediation
   continues without redundant re-asks.
7. **direct-mode-delivery** — BU-0015. Trigger: a direct-mode
   implementation is ready for delivery. Outcome: delivery is only declared
   complete once PR, CI, review, and merge authorization are all satisfied.
8. **reconcile-and-deliver** — BU-0035. Trigger: work has reached a
   terminal or deliverable state. Outcome: cleanup runs only after terminal
   state and evidence preservation are verified.

**Stage-context attached:**
- direct-mode-implementation: BU-0010 (context/td state loaded before any
  edit), BU-0011 (in-progress work by other workers/worktrees reconciled,
  not duplicated/raced).
- direct-mode-delivery: BU-0014 (direct-mode work passes the same
  validation/review/gate steps as dispatched work), BU-0016 (handoff/PR/
  merge/deployment/cleanup outcomes durably recorded).

**Workflow-local helpers:** none (0 `helper` records carry this `workflow`
value).

2 stage-context records here are unattached — see **Unattached records**.

### 3. worker-response-and-recovery

**Description.** Trigger: a worker signals a nonterminal state
(`waiting`/`needs_input`/`blocked`), or a wake condition becomes
permanently unsatisfiable. Outcome: the worker is either resumed through a
verified wake/response round-trip, or escalated to a human decision — never
guessed at or force-recovered. Completion: the response/resume action is
durably recorded and the worker's own consumption of it completes the
round-trip.

**Member stages** (ordered — evaluation precedes response by construction:
a wake condition must be found unsatisfiable before there is anything to
respond to):

1. **evaluate-wake-condition** — BU-0151. Trigger: a wake condition becomes
   permanently unsatisfiable (four named cases). Outcome: the worker
   escalates to `needs_input` with a stated remedy rather than retrying
   indefinitely.
2. **respond-to-worker** — BU-0155, BU-0275. Trigger (BU-0155): `sgt-respond`
   is about to be used. Trigger (BU-0275): a worker escalates with
   `needs_input`/`blocked`. Outcome: the five-step precondition/delivery
   sequence runs, and the human decision is genuinely obtained (not
   inferred) before a response is sent.

**Stage-context attached:**
- respond-to-worker: BU-0157 (a delivered response is applied exactly
  once, matching ID/generation/status), BU-0177 (a pending response is
  never clobbered; the correct convergence path is used instead of
  recover).

**Workflow-local helpers** (5): BU-0148 (durable wake-condition
representation, no live sleep loop), BU-0149 (`sgt-wake` resumes only the
exact matching, generation-tagged worker), BU-0150 (`github_check` wake
gated strictly on a success conclusion), BU-0156 (token-matched acceptance
round-trip before a nudge is acted on), BU-0178 (pane-not-found
classification depends on durable handoff state).

**Note — `recover-worker` never became a stage.** Five stage-context
records (BU-0039, BU-0146, BU-0159, BU-0174, BU-0286) name `stage:
recover-worker` — clearly a real, operator-visible checkpoint within a
workflow whose own name is "worker-response-**and-recovery**" — but no
`representation: stage` record for `recover-worker` exists anywhere in the
corpus. This is the single sharpest instance of the pattern flagged after
the workflow list below; all five are listed under **Unattached records**
rather than given an invented stage here.

9 stage-context records here are unattached — see **Unattached records**.

### 4. validation-gate

**Description.** Trigger: dispatched (or direct-mode) work reaches
readiness for shipping validation. Outcome: exactly one validation launch
runs to completion under a coordinator-verified, auditable transport, and
readiness is durably published only once every gate has genuinely passed.
Completion: readiness evidence anchored to a real, committed HEAD is
recorded before the coordinator is notified.

**Member stages** (ordered — publish-readiness's own trigger, "native
validation and independent reviews all pass," presupposes launch-validation
already ran):

1. **launch-validation** — BU-0042, BU-0161. Trigger: a dispatched worker
   reaches readiness / `sgt-validate` is invoked at readiness. Outcome: a
   validation-only boundary runs in a coordinator-owned, split pane, never
   auto-approved, with redundant stages skipped by a defined default set.
2. **publish-readiness** — BU-0160. Trigger: native validation and
   independent reviews all pass. Outcome: readiness is durably recorded
   with intent/head/review evidence before the coordinator is notified.

**Stage-context attached:**
- launch-validation: BU-0162 (exactly one launch per task/repo pair, concurrent
  attempts fail closed), BU-0163 (default transport never exposes intent via
  argv), BU-0164 (missing `--intent-file` support fails closed with full
  diagnostic, no partial state), BU-0165 (an argv-exposure consent applies
  to exactly one invocation, cannot silently persist), BU-0166 (transport
  choice is durably auditable and the executing build is re-verified against
  it), BU-0169 (rollback on pre-commit failure is scoped strictly to
  provably-owned artifacts).
- publish-readiness: BU-0309 (readiness evidence anchored to a committed
  HEAD, never a working-tree diff).

**Workflow-local helpers** (1): BU-0168 (every ownership claim/release is
durably logged; a release token is single-use).

2 stage-context records here are unattached — see **Unattached records**.

### 5. project-registration

**Description.** Trigger: a project needs its context resolved or its
registration validated. Outcome: context loading is confirmed complete via
an observable evidence artifact, never merely by having run the command.
Completion: `confirm-context-loaded`.

**Member stages.** Only one `stage` record carries this `workflow` value —
an uneven, single-stage workflow candidate, recorded as such rather than
reshaped:

1. **confirm-context-loaded** — BU-0258. Trigger: project context loading
   is claimed complete. Outcome: completeness is defined by an observable
   evidence artifact, not merely having run the command.

**Stage-context attached:** BU-0256 (a raw-YAML read is only a fallback for
a field `sgt-context` output doesn't surface), BU-0266 (a discrepancy
between `sgt-context` output and the raw YAML blocks progress and preserves
evidence rather than silently picking a source).

**Workflow-local helpers** (11): BU-0131 (project YAML six-field shape
check), BU-0200 (layered-instruction conflict resolution by
position/specificity), BU-0235 (`config.yaml` excluded from `sgt-list`),
BU-0236 (empty project directory reported as an actionable error), BU-0237
(`sgt-context` always distinguishes three clone states), BU-0238
(`sgt-context` always reports graph existence/build path), BU-0239
(`sgt-status` reports the exact reason for a missing/non-git repo path),
BU-0240 (upstream divergence surfaced explicitly), BU-0242 (clone only
under the exact defined precondition), BU-0243 (`sgt-td-list` filter built
deterministically from flags), BU-0244 (a repo missing `.git` is omitted,
not fatal).

5 stage-context records here are unattached — see **Unattached records**.

### 6. fleet-monitor-and-reconcile

**Description.** Trigger: `sgt-watch` runs to snapshot fleet state
(`--snapshot`), reconcile it in bulk (`--sync-all`), or assess a worker's
health. Outcome: busy/health/notification state is reported strictly from
verified evidence — never fabricated as idle, healthy, or a known basis
value when the verification conditions don't all hold. Completion: no stage
in this corpus names it — see below.

**Member stages: none.** Zero `representation: stage` records carry this
`workflow` value, even though two `stage-context` records do (BU-0141,
BU-0144) and five `helper` records support it. This workflow candidate has
no ordered stage list to give; both stage-context records are listed under
**Unattached records** rather than assigned an invented stage.

**Workflow-local helpers** (5): BU-0062 (`busy:true` only when all three
verification conditions hold), BU-0063 (unverified conditions report
null/unknown, never a fabricated idle), BU-0064 (an unrecognized observed
condition falls back to the null basis), BU-0143 (concurrent updates degrade
notification to a delayed wakeup, never a duplicate), BU-0145 (stale
progress evidence is diagnosed as stalled, not reclassified to terminal).

### 7. shipping-gate-driving

**Description.** Trigger: the coordinator drives a no-mistakes shipping
gate to completion for dispatched (or direct-mode) work. Outcome: the gate
is started at most once per precondition-satisfied run, polled rather than
re-issued, and findings are routed by disposition. Completion:
`group-remediation` converges remediation to one worker per shared root
cause, rechecked before merge, escalating to a human after two unsuccessful
cycles.

**Member stages.** Only one `stage` record carries this `workflow` value:

1. **group-remediation** — BU-0282. Trigger: multiple findings share the
   same root cause. Outcome: remediation converges to one worker per root
   cause, is rechecked before merge, and escalates to a human after two
   unsuccessful cycles rather than looping indefinitely.

**Stage-context attached:** none — all 5 stage-context records for this
workflow are unattached (see below). Notably the actual run-start
(`start-run`) and gate-driving (`drive-gate`) mechanics that most of this
workflow's description above is drawn from were classified `stage-context`,
not `stage` — only the remediation-convergence checkpoint reached the
`stage` rung.

**Workflow-local helpers:** none.

5 stage-context records here are unattached — see **Unattached records**.

### 8. review-finding-routing

**Description.** Trigger: a dispatched worker submits a review-finding
artifact to `sgt-review-findings`. Outcome: the finding is normalized,
deduplicated, and routed into exactly one of four defined dispositions as
an owning-repo td card, without ever silently overwriting a hand-edited
card. Completion: `route-finding`, followed by `reconcile-hand-edit` on any
rerun that meets a card modified outside the router.

**Member stages** (ordered — reconcile-hand-edit's trigger presupposes a
card the router already wrote, i.e. a prior route-finding run):

1. **route-finding** — BU-0096. Trigger: a dispatched worker produces a
   review finding artifact. Outcome: actionable findings become owning-repo
   td tasks with durably published blocking guidance.
2. **reconcile-hand-edit** — BU-0101. Trigger: a stored finding card has
   been modified outside the router since it last wrote it. Outcome: the
   human-edited content is preserved (not overwritten) and flagged for
   human reconciliation.

**Stage-context attached:**
- route-finding: BU-0103 (a failed route retains parsed/sanitized findings
  with an exact retry command), BU-0312 (a malformed/failed-routing review
  artifact escalates rather than being silently logged).

**Workflow-local helpers** (12): BU-0090 (disposition maps deterministically
to one of four outcomes), BU-0091 (repeated-ID findings update, not
duplicate, the existing card), BU-0092 (manual/repo labels survive a
rerun), BU-0093 (a hidden-state card is resurfaced before its body
refreshes), BU-0097 (non-actionable findings produce no card; malformed
bodies rejected; credential-shaped content redacted), BU-0098 (severity
normalized to three canonical levels), BU-0099 (dedup key dimensioned
enough to avoid generic-ID collisions), BU-0100 (digest detects a
hand-modified stored card), BU-0102 (a match against a closed card reopens
it, surfaced, not silently recreated), BU-0104 (a retried retained artifact
re-validates/re-digests before td is touched), BU-0301 (an axis without
guidance text fails loudly), BU-0302 (a `high`-severity finding always
blocks).

### 9. installation-and-setup

**Description.** Trigger: installation, or `mise run check`/`install`/
`update`, is invoked. Outcome: dependencies are verified against their real
capability surface, and symlinks/hooks are (re)installed or removed
idempotently, before Sergeant is considered usable. Completion:
`dependency-check` passes for every required dependency.

**Member stages.** Only one `stage` record carries this `workflow` value:

1. **dependency-check** — BU-0129. Trigger: the dependency check runs
   during installation. Outcome: installation does not proceed until both
   the td-implementation check and the agent-availability check pass.

**Stage-context attached:** none (0 `stage-context` records carry this
`workflow` value).

**Workflow-local helpers** (8): BU-0204 (install links every current and
future matching script generically), BU-0205 (a stale legacy symlink is
removed automatically, scoped to symlinks only), BU-0206 (hook install
skips cleanly when a directory is absent), BU-0207 (uninstall removes only
hooks this repo actually installed), BU-0208 (uninstall removes only
symlinks provably pointing into this repo's `bin/`), BU-0210 (an agent
harness check passes given any one of three, in priority order), BU-0211 (a
failed dependency check fails closed with an actionable message), BU-0212
(Bash-compatibility proof required under both ambient and minimum-supported
Bash).

### 10. project-graphify

**Description.** Trigger: `sgt-graphify` is invoked to extract and publish
a project's knowledge graph. Outcome: publication is atomic-after-
completion, never overlaps or destroys a source repo, and a failed or
incomplete run is never promoted to the published location. Completion:
`publish-graph` stops the run before publication if extraction produced
zero matched repos, or any repo's extraction failed.

**Member stages.** Only one `stage` record carries this `workflow` value:

1. **publish-graph** — BU-0250. Trigger: extraction produces zero matched
   repos, or any repo's extraction fails. Outcome: the run stops before
   publication rather than silently merging and publishing an incomplete
   graph.

**Stage-context attached:** none — all 3 stage-context records for this
workflow are unattached (see below).

**Workflow-local helpers** (12): BU-0133 (success requires both named
output artifacts to exist), BU-0196 (an unsafe repo name is rejected before
use as a path prefix), BU-0245 (readers never see a torn/partially-written
output), BU-0246 (an invalid repo name fails that repo's extraction with a
clear error), BU-0247 (output colliding with a source repo path fails
closed), BU-0248 (extraction runs against an exclusion-applied copy, never
the live tree), BU-0249 (missing LLM API key degrades to code-only
indexing, doesn't abort), BU-0251 (an incomplete staged output is never
promoted), BU-0252 (existing `wiki/`/`memory/` subdirectories survive
publish), BU-0253 (a mid-swap failure leaves the previous output intact),
BU-0254 (the symlink swap is atomic, old target removed only after the new
one is confirmed live), BU-0263 (success confirmed by artifact presence,
not exit code).

3 stage-context records here are unattached — see **Unattached records**.

### 11. fleet-cleanup

**Description.** Trigger: `sgt-cleanup` is invoked for a task. Outcome:
cleanup proceeds only once every named precondition holds — terminal proof,
staged evidence, a converged or explicitly retired response handshake, and
(when applicable) callback completion — never as a shortcut for a
nonterminal worker state. Completion: `cleanup-preconditions`.

**Member stages.** Only one `stage` record carries this `workflow` value —
and it is worth naming plainly that the response-handshake, callback-gate,
seal, and sealed-failure-recovery mechanics that most of this workflow's
description is drawn from all classified as `stage-context`, not `stage`:

1. **cleanup-preconditions** — BU-0171. Trigger: `sgt-cleanup` is invoked
   for a task. Outcome: cleanup proceeds only once every named precondition
   holds, and never as a shortcut for a nonterminal worker state.

**Stage-context attached:** none — all 9 stage-context records for this
workflow are unattached (see below).

**Workflow-local helpers:** none.

9 stage-context records here are unattached — see **Unattached records**.

### 12. dag-orchestration

**Description.** Trigger: a DAG stage declares an `after:` dependency.
Outcome: the stage becomes ready to dispatch only once its named
predecessor stages have completed. Completion: `stage-dependency-gate`,
advanced automatically by `sgt-watch`.

**Member stages.** Only one `stage` record carries this `workflow` value:

1. **stage-dependency-gate** — BU-0203. Trigger: a DAG stage declares an
   `after:` dependency. Outcome: the stage only becomes ready to dispatch
   once its named predecessor stages have completed, advanced automatically
   by `sgt-watch`.

**Stage-context attached:** none (0 records).

**Workflow-local helpers** (2): BU-0201 (a DAG name must not collide with
another project's DAG name), BU-0202 (a stage's brief source is one of two
alternatives, resolved by whether td is set).

### 13. callback-delivery

**Description.** Trigger: a callback event is registered, enqueued, or
delivered to an external consumer. Outcome: origin identity, idempotency,
bounded consumer execution, a closed outcome set, and requeue behavior are
all deterministic. Completion: no stage checkpoint was classified in this
corpus for this workflow — see below.

**Member stages: none, and no stage-context either.** Zero `stage` and zero
`stage-context` records carry this `workflow` value — its entire
represented content (7 records) classified as workflow-local `helper`. This
is distinct from the actual durable-delivery-guarantee gap this workflow's
behavior implies: that gap surfaces separately as engine-gap candidate
BU-0227 in Bucket 7 below, which explicitly attempted and rejected `stage`
for the same reason (no independently operator-visible checkpoint boundary,
just crash-safe claim/lease machinery spanning many attempts).

**Workflow-local helpers** (7): BU-0216 (a supplied correlation ID must be
opaque, not platform-identifier-shaped), BU-0218 (a repeat origin
registration succeeds harmlessly; a changed one is rejected), BU-0219
(`sgt-callback sync` is idempotent across reruns), BU-0220 (source identity
validated, never stored in plaintext, idempotent re-use), BU-0224 (a
consumer executable is bounded in time and output size), BU-0225 (a
consumer's return maps to a closed outcome set, defaulting to pending on
anything malformed), BU-0228 (a requeue preserves the original idempotency
key).

### 14. skill-adoption

**Description** (from BU-0119, the sole `representation: workflow`
record for this candidate). Trigger: an external skill is being adopted.
Outcome: the six-step vetting procedure (read `SKILL.md` and referenced
scripts; confirm source/update mechanism; check filesystem/shell/network/
Git/credential actions; verify no conflict with `AGENTS.md`/safety policy;
pin/lock the source where supported; test in a disposable repo/worktree) is
completed before broad installation. Completion: all six checks done.

**Member stages:** none.

**Stage-context attached:** none.

**Workflow-local helpers** (1): BU-0121 (each worker harness discovers the
same canonical `.agents/skills/` tree through its own harness-appropriate
path; no install step ever writes to global user config).

### 15. sergeant-help-query

**Description** (from BU-0124, the sole `representation: workflow`
record for this candidate). Trigger: `sergeant-help` is answering a
question. Outcome: the answer follows a fixed research sequence — classify
the question against the documentation map, read the primary document
first, escalate to a repository-wide grep only for unresolved terms,
consult `graphify query` for architectural questions when a graph exists —
rather than free-form search. Completion: the answer cites the exact
command, required preconditions, expected evidence, and documentation
links.

**Member stages: none.** Zero `stage` records carry this `workflow` value.

**Stage-context attached:** none — its one stage-context record is
unattached (see below).

**Workflow-local helpers** (1): BU-0125 (a fixed five-way source-precedence
order resolves disagreement among documentation sources).

1 stage-context record here is unattached — see **Unattached records**.

### 16. troubleshoot-td-identity

**Description** (there is no `representation: workflow` record for this
candidate; its only member is a single unattached `stage-context` record,
BU-0173 — an intentionally single-behavior workflow candidate, not reshaped
into anything larger). Trigger: the `td` executable resolved on PATH does
not support the required flags. Outcome: PATH is corrected to the required
implementation rather than building a wrapper around the wrong one, until
`td create --help` shows the required description/JSON/working-directory
options.

**Member stages: none.**

**Stage-context attached:** none — BU-0173 is itself unattached (see
below); it is this candidate's sole member, so it is *both* this
candidate's only content and its own bucket-3 "Unattached records"
appearance — one classification record, one bucket appearance, per the
method's accounting rule.

**Workflow-local helpers:** none.

**Cross-reference:** the identity check this stage's fix ultimately
satisfies is shared-helper BU-0132/BU-0209 in Bucket 5 below.

### 17. cross-repo-planning

**Description** (there is no `representation: workflow` record for this
candidate either; its only member is a single unattached `stage-context`
record, BU-0267). Trigger: a requested outcome is being decomposed across
repositories. Outcome: exactly one repository is named as owning each
required behavior, and a repository is included only when it must actually
change or produce delivery evidence.

**Member stages: none.**

**Stage-context attached:** none — BU-0267 is itself unattached (see
below), and is this candidate's sole member and sole bucket appearance.

**Workflow-local helpers:** none.

### 18. undocumented-failure-escalation

**Description** (from BU-0192, the third and last `representation:
workflow` record; unlike `skill-adoption` and `sergeant-help-query`, no
other record in the corpus carries a matching `workflow` value — this
candidate's name is minted fresh from BU-0192's own topic, kebab-cased, and
checked for uniqueness against every other name in this document plus
`.sergeant/workflows/` and `.sergeant/drafts/workflows/`). Trigger: a
failure is not covered by existing documentation. Outcome: `sergeant-help`
is used to search the docs, then the gap is escalated as a well-formed td
task containing the exact reproduction, expected behavior, preserved state,
and acceptance criteria — rather than left unresolved or guessed at.
Completion: the td task exists with all four required fields.

**Member stages: none. Workflow-local helpers: none.** This is the
corpus's most single-behavior workflow candidate of all eighteen — one
record, no supporting stage/context/helper material anywhere else in the
classified ledger.

---

## Unattached records

54 `stage-context` records (74% of the corpus's 73) name a `workflow` +
`stage` pair with no matching `representation: stage` record. Per
`references/synthesis-method.md` bucket 3, none of these are resolved by
inventing a stage to hang them on — each is a synthesis-time defect
surfacing from `40-classify`, recorded plainly here. Three workflows
(`fleet-monitor-and-reconcile`, `troubleshoot-td-identity`,
`cross-repo-planning`) have *zero* `stage` records at all despite having
`stage-context` content, meaning every checkpoint an operator would
plausibly care about in those workflows was classified one rung below where
`../_config/icm-ladder.md` §6.3's reimplementation test would seem to place
it. The single sharpest case is `worker-response-and-recovery`'s
`recover-worker` — five separate stage-context records (BU-0039, BU-0146,
BU-0159, BU-0174, BU-0286) all name the same clearly-checkpoint-shaped
`stage` value, in a workflow whose own name says "recovery," and yet no
`stage` record for it exists anywhere in 312 classifications. This pattern
is offered to `80-adversarial-review` as a corpus-wide signal worth
checking against `40-classify`'s own §6.3 rationale requirement, not
something this stage attempts to re-classify or repair.

| workflow | stage | behavior_id(s) |
|---|---|---|
| dispatch-worker | plan-and-decompose | BU-0006 |
| dispatch-worker | monitor-and-reconcile | BU-0008 |
| dispatch-worker | validate-harness-selection | BU-0057 |
| dispatch-worker | validate-model-compatibility | BU-0059 |
| dispatch-worker | bind-coordinator-pane | BU-0060, BU-0075, BU-0076, BU-0077 |
| dispatch-worker | record-launch-evidence | BU-0070 |
| dispatch-worker | acquire-drain-lock | BU-0105, BU-0289 |
| dispatch-worker | route-before-implementation | BU-0280 |
| dispatch-worker | handle-notification | BU-0305 |
| dispatch-worker | handle-crash-exit | BU-0306 |
| task-intake-and-execution | invoke-toolbelt-command | BU-0021 |
| task-intake-and-execution | monitor-progress | BU-0033 |
| worker-response-and-recovery | select-resume-path | BU-0037 |
| worker-response-and-recovery | recover-worker | BU-0039, BU-0146, BU-0159, BU-0174, BU-0286 |
| worker-response-and-recovery | drain-wait | BU-0154 |
| worker-response-and-recovery | acknowledge-response | BU-0158 |
| worker-response-and-recovery | diagnose-repeated-notification | BU-0179 |
| validation-gate | post-readiness-remediation | BU-0043 |
| validation-gate | claim-validation-ownership | BU-0167 |
| project-registration | resolve-instructions | BU-0053 |
| project-registration | confirm-project-name | BU-0255 |
| project-registration | sync-required-repo | BU-0257 |
| project-registration | validate-project-edit | BU-0260, BU-0261 |
| fleet-monitor-and-reconcile | sync-all | BU-0141 |
| fleet-monitor-and-reconcile | assess-worker-health | BU-0144 |
| shipping-gate-driving | start-run | BU-0080, BU-0082 |
| shipping-gate-driving | drive-gate | BU-0083, BU-0084 |
| shipping-gate-driving | recover-from-failure | BU-0087 |
| sergeant-help-query | handle-failure-and-handoff | BU-0128 |
| project-graphify | diagnose-output-path | BU-0184 |
| project-graphify | publish-output | BU-0198 |
| project-graphify | confirm-output-path | BU-0262 |
| fleet-cleanup | complete-response-handshake | BU-0186 |
| fleet-cleanup | retire-response-handshake | BU-0187, BU-0188, BU-0189, BU-0190 |
| fleet-cleanup | callback-completion-gate | BU-0230, BU-0231 |
| fleet-cleanup | seal-before-delete | BU-0232 |
| fleet-cleanup | recover-from-sealed-failure | BU-0233 |
| troubleshoot-td-identity | diagnose-wrong-td | BU-0173 |
| cross-repo-planning | assign-ownership | BU-0267 |

(54 behavior_ids total across the 38 rows above, matching the 54 figure
stated per-workflow throughout Buckets 1–3.)

---

## Bucket 4: permanent-instruction candidates

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

---

## Bucket 5: shared helper/context candidates

10 `shared-helper`/`shared-context` records, grouped by contract (what each
does, for whom) rather than by originating file, per
`references/synthesis-method.md` bucket 5. **Over-promotion-tell check**
(`../_config/icm-ladder.md` §6.6): none of the five groups below maps
one-to-one onto a single source file's own shared-helper/shared-context
units — every group spans at least two different source files restating
the same contract (e.g. the PATH-fallback group is stated twice in
AGENTS.md but that file's four shared-helper records split across three
*different* groups, not one), which is the reuse pattern this rung is
supposed to catch, not file-shape mirroring. No tell found. `.sergeant/
common/` does not exist yet in this worktree (checked directly), so none of
the five candidates below already has a same-named entry to compare
against or flag a mismatch with.

### 5a. toolbelt-command-resolution

**Contract.** Input: a bare `sgt-*` command name about to be invoked.
Output: the command runs — from PATH if it resolves there, otherwise the
matching script from this repository's `bin/`. Meaning: every toolbelt
invocation resolves the same way regardless of caller.

**Members:** BU-0020 (AGENTS.md), BU-0056 (AGENTS.md, generalized restatement
for "any sgt-* command").

**Consuming candidates:** effectively every workflow candidate above that
invokes an `sgt-*` command — `dispatch-worker`, `task-intake-and-execution`,
`worker-response-and-recovery`, `validation-gate`, `project-registration`,
`fleet-monitor-and-reconcile`, `shipping-gate-driving`,
`review-finding-routing`, `project-graphify`, `fleet-cleanup`,
`dag-orchestration`, `callback-delivery`.

### 5b. dev-root-relative-path-resolution

**Contract.** Input: a repo `path` value read from a project YAML that is
not absolute or home-relative. Output: the path resolves relative to the
single `dev_root` configured in the global Sergeant config file. Meaning:
project YAMLs stay portable across machines by changing `dev_root` in one
place.

**Members:** BU-0051 (AGENTS.md), BU-0193 (docs/schema.md, explicitly
naming `sgt-context` and "any sgt-* script reading project YAML" as
consumers).

**Consuming candidates:** `project-registration`; indirectly any workflow
that reads project YAML repo paths (`dispatch-worker`, `project-graphify`).

### 5c. project-name-identity

**Contract.** Input: a project YAML file. Output: the project's
addressable name is its filename without extension; a `name` field that
disagrees with the filename is a validation condition. Meaning: identity is
derived once, deterministically, not re-decided per consumer.

**Members:** BU-0052 (AGENTS.md), BU-0195 (docs/schema.md, restated as a
validation condition).

**Consuming candidates:** `project-registration`.

### 5d. td-capability-surface-check

**Contract.** Input: a `td` executable resolved for use. Output: accepted
only when its version is supported AND `td create --help` shows a
description option, a JSON-output option, and a working-directory option —
rejected even if named `td` when the capability surface doesn't match.
Meaning: a same-named executable is never trusted by name alone.

**Members:** BU-0132 (docs/getting-started.md), BU-0209 (mise.toml, as
`mise run check`'s own dependency verification).

**Consuming candidates:** `installation-and-setup` (`dependency-check`
stage); `troubleshoot-td-identity` (the fix this workflow's diagnosis
ultimately restores compliance with).

### 5e. shared-review-axis-definition

**Contract.** Input: none (a static definition). Output: the set of
required review axes/severities (with the conditional Accessibility axis
triggered by frontend/UI/accessibility language) drives both what a worker
brief demands and what `sgt-review-findings` accepts for routing, from one
source. Meaning: the two sides of the contract cannot drift apart — BU-0300
names a specific prior operational failure (td-61a0c8) this candidate
exists to prevent.

**Members:** BU-0095 (README.md, `shared-context`), BU-0300
(bin/_sgt-review-axes.sh, `shared-context`, states the drift incident this
contract prevents).

**Consuming candidates:** `dispatch-worker` (brief generation) and
`review-finding-routing` (finding acceptance/routing).

---

## Bucket 6: obsolete-mechanism findings

None. Zero classification records in `../40-classify/output/
classifications.ndjson` carry an `obsolete-mechanism` disposition. This
bucket is empty by the evidence in hand, not by omission — checked directly
against the full 312-record ledger (see the representation-count table
below).

---

## Bucket 7: engine-pressure candidates

2 `engine-gap` records, `engine_gap` objects carried through unchanged per
method (this stage clusters and lists, it does not edit engine-gap
evidence).

### BU-0137 — exactly-once crash-safe mission delivery

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

### BU-0227 — durable claim/lease for callback delivery

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

## Per-bucket record count

| Bucket | Representation(s) | Count |
|---|---|---|
| 1–3 (workflow / stage / stage-context) | `workflow`, `stage`, `stage-context`, `helper` | 194 (3 + 31 + 73 + 87) |
| 4 (permanent-instruction) | `agents-invariant` | 106 |
| 5 (shared helper/context) | `shared-helper`, `shared-context` | 10 (8 + 2) |
| 6 (obsolete-mechanism) | `obsolete-mechanism` | 0 |
| 7 (engine-pressure) | `engine-gap` | 2 |
| **Total** | | **312** |

Matches `../40-classify/output/classifications.ndjson`'s 312 records
exactly.

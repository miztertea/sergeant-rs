# Synthesis candidates

Produced by `50-synthesize`, per `references/synthesis-method.md`'s seven
buckets. Source: `../40-classify/output/classifications.ndjson` (108
records), joined against `../30-normalize/output/behavior-units.normalized.ndjson`
for `trigger`/`outcome`/`scope` text. All 108 `behavior_id`s were verified
programmatically to appear in exactly one bucket appearance (see the
accounting note at the end).

Representations actually present in this corpus: `workflow` (1),
`stage` (6), `stage-context` (9), `agents-invariant` (13),
`shared-helper` (79). Not present: `shared-context`, `obsolete-mechanism`,
`engine-gap` — buckets 6 and 7 are empty for this run (see those sections).

No `../_config/...` blindness issue applies here (this is a decomposition
run, not the N2 measurement run — `references/synthesis-method.md` and
`../_config/icm-ladder.md` are Layer 3 method files, not the reference
corpus). `../40-classify/output/classifications.ndjson` does not open with
`# AMBIGUOUS — NOT RESOLVED`, so ordinary synthesis proceeds.

---

## Bucket 1 — Workflow candidates

Three distinct `workflow` values appear across the corpus (one from an
explicit `representation: workflow` record, two seen only through the
`workflow` field of `stage`/`stage-context` records — bucket 1 counts
both kinds of evidence). Checked for name collisions against
`.sergeant/workflows/` (only `repo-to-icm`, this workflow itself, exists)
and `.sergeant/drafts/workflows/` (does not exist yet in this worktree):
no collisions.

### 1. `dispatch-mode`

- **Source:** `representation: workflow` record `BU-0003`, plus its one
  member stage `BU-0041`.
- **Trigger:** work spans repositories, has independent repo-owned
  sub-tasks, needs isolated review, or the user explicitly requests
  workers.
- **Outcome:** one worker is dispatched per owning repository.
- **Completion condition:** progress is monitored through to
  reconciliation of merge order and cross-repo implications.
- **Member stage candidates:** `dispatch-worker` (see bucket 2).

### 2. `standard-task-workflow`

- **Source:** no `representation: workflow` record names this value —
  it is seen only through five `stage` records (`BU-0011`, `BU-0012`,
  `BU-0013`, `BU-0016`, `BU-0019`) and two `stage-context` records
  (`BU-0014`, `BU-0015`, both unattached — see the Unattached records
  heading). Per bucket 1's instruction, a distinct `workflow` value seen
  this way still earns one candidate.
- **Trigger:** the user brings a task (the shared trigger text on
  `BU-0011`, `BU-0012`, `BU-0013`).
- **Outcome:** the task advances through a fixed sequence of durable
  checkpoints — establishing context, avoiding duplicate queue entries,
  reconciling in-flight state, executing under a chosen mode, and a
  single dedicated validation boundary — before being treated as
  delivered.
- **Completion condition:** cleanup never runs ahead of verified terminal
  state and preserved evidence — the `reconcile-and-deliver` stage's own
  outcome (`BU-0016`).
- **Member stage candidates (ordered):** `load-context`, `check-queue`,
  `reconcile-existing-state`, `validate`, `reconcile-and-deliver` — see
  bucket 2 for the ordering rationale and its one genuinely ambiguous
  call.

### 3. `ship-with-no-mistakes`

- **Source:** no `representation: workflow` or `representation: stage`
  record names this value at all — it is seen only through seven
  `stage-context` records (`BU-0028`–`BU-0034`), every one of which is
  unattached (no matching stage candidate exists for any of the four
  stage names they cite — see the Unattached records heading). This is
  the corpus's most awkward workflow candidate: a workflow visible only
  through orphaned judgment-content, with zero classified checkpoints of
  its own. Per `references/synthesis-method.md`'s "What must not happen"
  section, this is written down as-is rather than smoothed over by
  inventing stage candidates to make it look tidy.
- **Trigger:** a shipping-gate run is about to be started, is active, or
  reaches an end state (checks-passed, failed, cancelled, or an
  actionable finding).
- **Outcome:** shipping-gate runs are launched under strict invocation
  discipline (never auto-approved; started only by the coordinator,
  exactly once, after native validation) and driven under strict handling
  rules while active (ask-user gates always reach a human decision; the
  pipeline-owned worktree and commit history are never touched; driving
  stops at checks-passed rather than polling to merge).
- **Completion condition:** on failure or cancellation, recovery follows
  the reported `branch_sync` action exactly (no improvised git surgery);
  actionable findings are routed out to task-tracker tasks rather than
  fixed inline by the run itself.
- **Member stage candidates:** none — see the Unattached records
  heading; this is itself the finding worth surfacing for this
  candidate.

---

## Bucket 2 — Stage candidates

Grouped by `workflow`+`stage`, ordered by chaining `trigger`→`outcome`
across the member behavior units in
`../30-normalize/output/behavior-units.normalized.ndjson`.

### `dispatch-mode`

1. **`dispatch-worker`** (`BU-0041`) — trigger: dispatch is invoked for
   one or more repos. Outcome: each targeted repo gets its own isolated
   checkout, a written brief, and a spawned interactive agent session.
   Only member stage; no ordering call needed.

### `standard-task-workflow`

Ordered using the explicit step numbers carried in each unit's `scope`
field (`../30-normalize/output/behavior-units.normalized.ndjson`), which
directly number four of the five stages:

1. **`load-context`** (`BU-0011`, scope: "standard task workflow, step 1
   (Load context)") — trigger: the user brings a task. Outcome:
   execution-mode selection is made only after context is loaded.
2. **`check-queue`** (`BU-0012`, step 2) — trigger: the user brings a
   task. Outcome: an existing canonical task is reused rather than a
   duplicate created.
3. **`reconcile-existing-state`** (`BU-0013`, step 4) — trigger: the user
   brings a task. Outcome: preserved in-flight work is resumed/taken over
   instead of duplicated.
4. **`validate`** (`BU-0019`, scope: "validation-boundary execution", **no
   step number given**) — trigger: a worker reaches readiness, or
   remediation changes HEAD after readiness. Outcome: validation runs
   exactly once as a dedicated boundary, and post-readiness HEAD changes
   get rereview without retriggering the full cycle.
5. **`reconcile-and-deliver`** (`BU-0016`, step 9) — trigger: a task
   appears to have reached completion. Outcome: cleanup never runs ahead
   of verified terminal state and preserved evidence.

**Genuinely ambiguous ordering call:** `validate`'s own `scope` text gives
no step number (unlike the other four, which are numbered 1, 2, 4, 9), and
steps 3, 5, 6, 7, and 8 are otherwise occupied only by unclassified content
or by the two unattached `stage-context` records (`BU-0014` "step 5,
confirm-decisions" and `BU-0015` "step 7, monitor-progress" — neither is a
`stage` candidate, so neither anchors an ordering point here). I placed
`validate` between `reconcile-existing-state` (step 4) and
`reconcile-and-deliver` (step 9) because its trigger ("a worker reaches
readiness") can only fire after work has been dispatched and monitored —
occurring later than step 4 — and its outcome must be settled before
`reconcile-and-deliver`'s own trigger ("a task appears to have reached
completion") can honestly fire — occurring before step 9. I cannot pin an
exact step number (5, 6, 7, or 8) for it from the corpus alone; this is a
judgment call `80-adversarial-review` should be free to challenge.

---

## Bucket 3 — Stage-context attachments

Checked all 9 `stage-context` records' `workflow`+`stage` fields against
the `(workflow, stage)` keys of the 6 `stage` candidates in bucket 2
(exact match required, verified programmatically, not by inspection).

**Result: zero attach.** Every one of the 9 `stage-context` records names a
`workflow`+`stage` pair with no corresponding `stage` candidate. All 9 are
recorded under the `## Unattached records` heading below rather than
attached to anything or silently dropped, per this bucket's instruction.

---

## Bucket 4 — Permanent-instruction candidates

13 `agents-invariant` records, all sourced from `AGENTS.md`. Listed, not
drafted into any workflow package (that call belongs to the promotion
reviewer, not this run).

| ID | Statement |
|---|---|
| `BU-0001` | Before acting on a project, resolve its repositories, roles, inherited instructions, and configured paths through the project's context-resolution step, rather than inferring ownership from the current directory. |
| `BU-0002` | The primary session coordinates multi-repository work by default, and may implement directly only when the user explicitly asks to work in-session (or asks not to dispatch) and one repository owns the complete outcome. |
| `BU-0004` | In direct mode, the default branch is never edited; a feature branch is created or reused before the first implementation change. |
| `BU-0005` | Every direct-mode implementation requires opening a PR and satisfying required CI, review threads, and merge authorization before delivery is considered complete. |
| `BU-0006` | Every directive written into this instruction file must specify at least one of: a trigger/condition, a required or prohibited action, or evidence/a stop condition proving compliance. |
| `BU-0007` | A bare toolbelt command is used when it resolves on PATH; otherwise the matching local script is run instead. Manual operations are used only when no toolbelt command covers the operation or the command returns an explicit unsupported-case error, and that fallback plus the original error evidence must be reported. |
| `BU-0008` | Procedural skills are loaded only when their stated trigger condition applies. |
| `BU-0009` | For every listed skill trigger, the repository-local skill definition file is read directly; it is canonical and takes precedence over any same-named registry skill. |
| `BU-0010` | A harness registry's omission of a skill does not make the skill unavailable, and is not by itself grounds to ask the owner or stop; the actor stops and reports only the exact repository-local path when that file is itself absent or unreadable, and does not reconstruct a partial protocol from memory. |
| `BU-0017` | Deferred waits publish a durable wake condition and resume automatically once it is satisfied, while human decisions resume through an explicit response-delivery step; a waiting worktree is never cleaned, and an expected blocked exit is never rewritten as orphaned. |
| `BU-0018` | Every dispatched implementation, independent review, PR description, successor, recovery, and final shipping gate must use the same canonical intent revision from a shared intent file, and workers and remediation loops never run the shipping-gate tool themselves. |
| `BU-0020` | A completed, merged, blocked, or abandoned task is never left recorded as in_progress; the task tracker and fleet state are reconciled truthfully. |
| `BU-0021` | Tool absence produces an actionable fallback or explicit blocker, never a silent skip, false success, or indefinite wait. |

---

## Bucket 5 — Shared helper/context candidates

79 `shared-helper` records, zero `shared-context` records. None of these
records carry a `workflow`/`stage` field to group by (per the ladder,
6.6's shared/local distinction applies to helpers and contexts
independent of any one checkpoint), so grouping is by the underlying
mechanism each record's `statement` and `source.path` actually name —
verified to partition all 79 IDs exactly once (no gaps, no duplicates;
checked programmatically). No `.sergeant/common/` directory exists yet
anywhere in this worktree, so none of the following candidates has a
same-named entry to compare against or flag a mismatch with.

| Candidate | Contract (inputs / output shape / meaning) | Members | Consuming workflow candidate(s) |
|---|---|---|---|
| `dispatch-contract` | Governs `sgt-dispatch` end to end: accepted-harness/interactive-only gating before any worker state is created; provider/model tuple resolution by precedence (explicit pin → env default → harness default) with an unpinned dispatch explicitly recorded as such and a charset-restricted tuple format; `launch_state`/`variant_verified` honesty bookkeeping; coordinator-pane identity verified live before use; `--adopt-branch` / task-tracker-sourced briefs / cross-repo dependency ordering / paired callback-profile+correlation-id / dry-run flags. | `BU-0022, BU-0023, BU-0025, BU-0026, BU-0027, BU-0042, BU-0043, BU-0044, BU-0045, BU-0046, BU-0047, BU-0048, BU-0049, BU-0050` (14) | `dispatch-mode` directly (its `dispatch-worker` stage is exactly this machinery); `standard-task-workflow` indirectly, since dispatch mode is one of the execution modes its `load-context` stage's outcome selects among. |
| `dag-dispatch-hook` | A DAG-stage-ready hook: requires DAG run ID and stage ID already set in its environment (refuses otherwise); writes those IDs into every dispatched repo's fleet state so fleet-side watching can report back; warns (does not fail the stage) if the underlying dispatch yields no recognizable fleet task ID. | `BU-0051, BU-0052, BU-0053` (3) | None of this run's three workflow candidates names DAG-run execution as its own workflow — no `workflow: dag-*` value appears anywhere in the corpus. This hook layers on top of `dispatch-contract` (it calls dispatch), so it is only transitively reachable through whichever candidate consumes that one; it does not itself cite a workflow. |
| `dag-run` | Starting a project's DAG run: reads the stage graph from the project's own YAML config (not a separate DAG file); a stage's brief can defer to a task-tracker task; stage dependencies gate readiness; dry-run reports what would register without registering anything; required external tools must be installed up front or the run fails with an install pointer before any other work happens. | `BU-0054, BU-0055, BU-0056, BU-0057, BU-0058` (5) | Same as `dag-dispatch-hook`: no workflow candidate in this corpus names DAG-run execution directly. |
| `cleanup` | `sgt-cleanup`'s task-id validation (rejects empty/dot/absolute/path-separator ids, and symlinked or mis-located canonical paths, before touching fleet state) and cleanup semantics (already-removed is a no-op success; pending callback state is synced, and — for whole-task cleanup — verified acknowledged, before worktree removal; fleet state and its worktree must share a filesystem for atomic evidence preservation). | `BU-0059, BU-0060, BU-0061, BU-0062, BU-0063` (5) | `dispatch-mode` (workers it creates are eventually cleaned up) and `standard-task-workflow`'s `reconcile-and-deliver` stage, whose own outcome ("cleanup never runs ahead of verified terminal state") is exactly this contract's precondition. |
| `drain` | Scope-exclusive drain admission (project XOR global, never both) backed by a lock that needs only a writable, lock-capable drain-state directory (no external locking tool) — dispatch and respond fail closed rather than proceed unlocked; while drained, new pane starts from relaunch/stall-recovery are refused and arriving responses are stored generation-safely; a cooperative wait always activates the drain first, treats workers with unverifiable process identity as unresolved (not finished) unless already terminal, and on timeout leaves the drain active with unresolved workers named in the error; project names must match a restricted identifier pattern and not collide with the lock's own filename. | `BU-0040, BU-0064, BU-0065, BU-0066, BU-0067, BU-0068, BU-0069, BU-0070` (8) | `dispatch-mode` and `standard-task-workflow`'s `reconcile-existing-state` stage (whose own outcome names "active workers, branches, worktrees, retained gates" as exactly what must be reconciled — drain state is part of that). |
| `drain-force` | Force-stopping drained-but-unresolved workers requires an already-active matching drain for the scope (refuses against an undrained scope) and never runs automatically — requires explicit confirmation or a dry run. | `BU-0071, BU-0072` (2) | Same as `drain`: `dispatch-mode` and `standard-task-workflow`'s `reconcile-existing-state` stage. |
| `undrain` | Removing a drain for a scope that was never drained succeeds as a no-op; scope is exclusive the same way setting a drain is (global XOR named project, rejected together). | `BU-0073, BU-0074` (2) | Same as `drain`. |
| `recover` | Stall recovery requires proof of stall (status `in_progress` plus a stall-detection-written diagnostic); exactly one bounded recovery attempt per stall (a second escalates to `needs_input` rather than retrying); the original stalled pane is killed only after a validated replacement pane is up (preserved otherwise); refused for a task/repo whose fleet-tracked worktree is missing or unrecorded. | `BU-0075, BU-0076, BU-0077, BU-0078` (4) | `dispatch-mode` and `standard-task-workflow`'s `reconcile-existing-state`/`validate` stages (a stalled worker blocks exactly the state those stages must reconcile or wait on). |
| `respond` | Response delivery is gated on worker status (`needs_input`/`blocked`/`waiting`/`orphaned` only), on the worktree still being confirmably owned (with a legacy-migration escape hatch), and on the recorded canonical intent revision still matching (else an audited human decision is required); pending-but-unacknowledged deliveries block re-delivery unless escalation is armed; re-running an identical respond after an acknowledgement timeout is the supported one-relaunch recovery path; empty responses are rejected up front. | `BU-0079, BU-0080, BU-0081, BU-0082, BU-0083, BU-0084` (6) | `dispatch-mode` and `standard-task-workflow`'s `reconcile-existing-state` stage; also the mechanism underlying `BU-0017`'s `agents-invariant` (bucket 4) — "human decisions resume through an explicit response-delivery step" is this contract. |
| `ack-response` | Acknowledgement is refused unless invoked from the exact pane dispatch recorded as owning the worker, and unless the worker's own post-application proof file agrees with the pending response's id/generation/status; a `done` acknowledgement requires a non-empty result, a `failed` one a non-blank reason; an acknowledgement leaving the worker at `needs_input`/`blocked` requires the gate generation to have strictly advanced; every acknowledged response is archived (body, generation, applied status, proof) before pending-response files are cleared. | `BU-0085, BU-0086, BU-0087, BU-0088, BU-0089` (5) | Same as `respond` — this is `respond`'s completion half. |
| `watch` | Stall classification after a configured grace period with no recorded activity (cleared once progress resumes); completion outcomes reported back to a linked DAG run/stage when tracking files and the engine are both present; terminal-worker pane recycling is idempotent per pane identity; a side-effect-free, constant-size versioned-JSON snapshot mode answers only whether Sergeant is verifiably doing work right now — `busy:true` only when a stable `in_progress` status, an exact live pane identity, and attributable recent progress are *all* simultaneously verified; every other outcome is `busy:null`, never `busy:false`, since absence of a witness is not proof of idleness. | `BU-0024, BU-0090, BU-0091, BU-0092, BU-0093, BU-0094` (6) | `dispatch-mode` and `standard-task-workflow`'s `reconcile-existing-state`/`validate`/`monitor-progress` content (the latter an unattached `stage-context`, `BU-0015`, whose own subject — "requiring recent meaningful events or a verified pane identity" — is this contract's snapshot logic in judgment form). |
| `wake` | Durable wake conditions are evaluated with read-only adapters per declared kind (time bound, GitHub check, fleet-task dependency, task-tracker-task dependency, deployment, human response) and resumed only once genuinely satisfied; a deadline without resolution fails the wake terminally; an attempt cap exceeded escalates to `needs_input`; `human_response` kind is never auto-evaluated — it sets `needs_input` immediately; wake-condition fields are strictly allowlisted by name and value pattern, rejecting unknown fields or shell-meaningful/leading-`-` values outright. | `BU-0095, BU-0096, BU-0097, BU-0098, BU-0099` (5) | `dispatch-mode` and `standard-task-workflow` (waiting workers dispatched under either eventually resume through this contract; also underlies `BU-0017`'s deferred-wait invariant in bucket 4). |
| `interactive-worker` | The interactive worker validates its harness against the accepted-harness registry before any model/variant resolution or state write; refuses to start without both an available worktree and a real attached terminal (stdin+stdout both ttys); a cooperative drain settles the handoff durably first (finalized-by-proof or explicitly-outstanding) before publishing `drained` status, never inventing a result for unfinished work, and checks both global and its own project's drain (a different project's drain does not affect it). | `BU-0100, BU-0101, BU-0102, BU-0103, BU-0104` (5) | `dispatch-mode` (this is the worker `dispatch-worker` spawns) and `standard-task-workflow`'s `reconcile-existing-state` stage (drain interacts with reconciling in-flight state). |
| `notify` | A worker update message is classified into completion/escalation/generic purely from its leading text, driving durable event recording; by default an update is recorded as a durable metadata-only wake marker rather than injected live (raw injection is an explicit compatibility transport only); durable callbacks are always derived from the task's own authoritative recorded state, never the free-text message; a callback-sync failure during notify still lets the rest of the recording complete before reporting failure, rather than aborting notification entirely. | `BU-0105, BU-0106, BU-0107, BU-0108` (4) | `dispatch-mode` and `standard-task-workflow`'s `reconcile-existing-state`/`monitor-progress` content — notify is how a dispatched worker's state changes become durable events those stages read. |
| `findings-router` | Correctness/security/data-integrity/test findings can never be deferred or ignored; cosmetic/evidence-only findings never create cards; generated worker briefs require one independent review per axis from a single shared required-axes definition (Standards, Spec, Readiness, plus conditional Accessibility) that also drives what the router accepts, so brief and router cannot drift apart; reviewer severity spellings are normalized to a canonical error(P1)/warning(P2)/info(P3) set, only error publishing a blocking gate; each finding-card revision ends with a self-covering digest line, and a later route finding the stored card altered preserves the old revision under a Superseded separator and marks the card needs-reconciliation for a human to merge; the retained artifact holds only post-redaction fields (never the reviewer's raw output), and a route refuses to overwrite an artifact nobody has retried yet. | `BU-0035, BU-0036, BU-0037, BU-0038, BU-0039` (5) | `ship-with-no-mistakes`'s `route-findings` stage-context (`BU-0034`, unattached but explicit: "routed... by the findings-routing tool") — this is that tool's contract. |

**Accounting:** 14+3+5+5+8+2+2+4+6+5+6+5+5+4+5 = 79. Verified
programmatically against the full `shared-helper` ID set: no gaps, no
duplicates.

---

## Bucket 6 — Obsolete-mechanism findings

None. Zero `representation: obsolete-mechanism` records exist in
`../40-classify/output/classifications.ndjson` for this corpus (confirmed
by counting `representation` values across all 108 records). This bucket
is empty for this run, not skipped.

---

## Bucket 7 — Engine-pressure candidates

None. Zero `representation: engine-gap` records exist in this corpus, and
every record's `engine_gap` field is `null` (confirmed programmatically).
This bucket is empty for this run, not skipped.

---

## Unattached records

Both classes of synthesis-time defect `references/synthesis-method.md`
names (bucket 1's workflow-less `helper`/`stage`/`stage-context` record,
and bucket 3's stage-context-with-no-matching-stage) reduce, in this
corpus, to a single class actually observed: **every one of the 9
`stage-context` records names a `workflow`+`stage` pair with no matching
`stage` candidate.** (The other class — a `helper`/`stage`/`stage-context`
record missing its required `workflow` field entirely — does not occur:
all 9 `stage-context` records and all 6 `stage` records carry a non-empty
`workflow` field.) This was checked programmatically against the 6
`(workflow, stage)` keys bucket 2 established, not by inspection.

| ID | `workflow` | `stage` named | Why it doesn't attach |
|---|---|---|---|
| `BU-0014` | `standard-task-workflow` | `confirm-decisions` | No `stage` candidate named `confirm-decisions` exists under `standard-task-workflow` (scope says this is step 5; no unit was classified `stage` for that step). |
| `BU-0015` | `standard-task-workflow` | `monitor-progress` | No `stage` candidate named `monitor-progress` exists under `standard-task-workflow` (scope says step 7; same gap). |
| `BU-0028` | `ship-with-no-mistakes` | `start-run` | No `stage` candidate named `start-run` exists — no `representation: stage` record cites `ship-with-no-mistakes` at all. |
| `BU-0029` | `ship-with-no-mistakes` | `start-run` | Same as `BU-0028`. |
| `BU-0030` | `ship-with-no-mistakes` | `drive-gates` | No `stage` candidate named `drive-gates` exists. |
| `BU-0031` | `ship-with-no-mistakes` | `drive-gates` | Same as `BU-0030`. |
| `BU-0032` | `ship-with-no-mistakes` | `finish-run` | No `stage` candidate named `finish-run` exists. |
| `BU-0033` | `ship-with-no-mistakes` | `finish-run` | Same as `BU-0032`. |
| `BU-0034` | `ship-with-no-mistakes` | `route-findings` | No `stage` candidate named `route-findings` exists. |

Per the method: these 9 are not dropped and no stage candidate is
invented to hang them on. Each counts as its one bucket appearance (the
bucket-1/bucket-3 "unattached" appearance), and each is still visible in
bucket 1 as the reason `ship-with-no-mistakes` (and, more narrowly, two of
`standard-task-workflow`'s named steps) has no member stage candidates
covering that ground.

---

## Full accounting

| Bucket | Records |
|---|---|
| 1–2 (workflow + stage candidates) | 1 (`BU-0003`) + 6 (`BU-0011,12,13,16,19,41`) = 7 |
| 3 (stage-context, attached) | 0 |
| 4 (agents-invariant) | 13 |
| 5 (shared-helper) | 79 |
| 6 (obsolete-mechanism) | 0 |
| 7 (engine-gap) | 0 |
| Unattached (stage-context, bucket 1/3 defect appearance) | 9 |
| **Total** | **7 + 0 + 13 + 79 + 0 + 0 + 9 = 108** |

Matches `../40-classify/output/classifications.ndjson`'s 108 records
exactly. No `behavior_id` is silently absent from every bucket; no
candidate name here lacks at least one member record citing it.

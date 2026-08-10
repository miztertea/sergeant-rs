# Engine-Pressure Roll-Up

Part of the N1 reference corpus (`docs/gauntlet/contracts/N1.md`, §8.1's
`engine-pressure.md`). Sourced from `synthesis.md` §5, applying §6.7 of the
ICM decomposition ladder ("does Sergeant itself need to own a new durable
fact — ordering, identity, retry, recovery, authorization, isolation, or
evidence semantics — to represent a behavior faithfully?") and the engine-gap
claim template normalized in `docs/icm/record-shapes.md` §5.

16 behavior units classified `engine-gap`, merged into **nine** distinct
claims (duplicate/near-duplicate units collapsed to one claim each, per
`synthesis.md`'s note that three independent partitions sometimes found the
same seam). **Five survive** the full §6.7 template below, all five narrowed
or amended from their first-pass statement; **four are rejected**, each with
the lower rung its behavior is actually absorbed by (N1 contract binding
rule: "an engine-gap claim without named failed lower rungs is auto-rejected"
applies equally to a *rejection* — it must name where the behavior actually
lives).

**Round-1 revision (adjudication-round1.md A13).** G5 is a special case
among the rejected: this document's first pass placed it among the
survivors ("narrowed — answers contract Unknown U3"). Round-1 adjudication
required re-verifying its "never attempted" lower rung — a re-enterable
needs_input stage — against the actually-shipped engine rather than the
proposal text, per L1's measure-don't-assume discipline extended from the
Claude adapter to the engine itself. That rung is already shipped
(`src/domain/work.rs`, `src/api.rs`; see G5's entry below), so the claim is
rejected on re-derivation, moved out of "Surviving claims" into "Rejected
claims" below, and its full original reasoning is kept in place rather than
deleted, so both what it once claimed and why that claim did not survive
scrutiny remain visible. G9's rejection of BU-P7-079 was also revised this
round, on unrelated grounds (a severed circular citation, not a shipped-rung
check) — see G9's entry.

This is content-only pressure evidence for a future milestone's contract
(`docs/gauntlet/contracts/N1.md` Non-goals; proposal §21.8's trigger
conditions are the bar a claim must eventually clear) — nothing here
authorizes an engine change now.

Every field below is the template `docs/icm/record-shapes.md` §5 requires
verbatim: *behavior that cannot be represented*, *source evidence requiring
it*, *lower-rung representations attempted*, *why each lower rung fails*,
*minimum runtime capability required*, *observable acceptance test*. Where a
claim merges several behavior units, the per-unit `engine_gap` objects in
`behavior-units/*.ndjson` are the canonical machine-readable copies; this
document presents the merged, adversarially-narrowed prose synthesis.md
already produced, plus the citation trail so each field's source evidence can
be independently re-opened.

---

## Surviving claims

Ranked by evidence strength (`synthesis.md` §5's own ranking, unchanged here).

### G1 — Runtime-owned durable wait/wake scheduling
**Verdict:** survives, narrowed to the scheduler alone · **Rank 1**
**Merges:** BU-P6-099, BU-P7-010, BU-P7-096 (mutual duplicates of one seam,
found independently by P6, P7 twice).

**1. Behavior that cannot be represented.**
A Work must be able to suspend on a typed external condition (a CI check's
conclusion, a sibling Work's completion, an external task tracker's status, or
an arbitrary future timestamp), have its process exit entirely while
suspended, and be resumed automatically once the condition becomes true — with
bounded, jittered retry between evaluations, a hard deadline after which the
wait fails outright (`failed`), and an attempt-count ceiling after which the
wait instead escalates to human input (`needs_input`) — all without any actor
consuming a live turn, or any external scheduler, merely to keep waiting.

**2. Source evidence requiring it.**
- BU-P6-099 — `reference/sergeant-upstream/bin/sgt-wake` L19-21, L225-251
  (`sha256:846babfec1ce0753da04931a946f7f9a8195976fd6a3b68a09760cebe1c89556`):
  exponential-feeling backoff with per-attempt jitter; deadline exceeded is
  terminal-`failed`, `max_attempts` exceeded escalates to human input.
- BU-P7-010 — `reference/sergeant-upstream/templates/worker-brief.md` §"4.
  Escalate and resume"
  (`sha256:2ee50a89f2134aa5bd7b6a2d472721670c2770919f122e995ae1fb347c62404e`):
  a worker must never poll or sleep for a deferred external condition; it
  publishes `waiting` with a structured `.sergeant-wake-condition` file naming
  one of six condition kinds, then exits cleanly.
- BU-P7-096 — `reference/sergeant-upstream/tests/sgt-wake-test.sh` lines 1-9
  (`sha256:87bea36a43fa11264e585faa39909c4848f36f3ae97170a8e8715c47999d0e1e`):
  proves the four-party seam (`sgt-wake`, `sgt-respond`'s resume path, the
  interactive worker's clean non-orphaned exit into `waiting`, and
  `sgt-dispatch`'s brief-level prohibition on sleep loops) moves together as
  one contract, not one command's isolated behavior.
- Corroborating (non-`engine-gap`, cited as supporting texture, not part of
  the census): BU-P8-074, BU-P8-075, BU-P8-076, BU-P6-096, BU-P6-097,
  BU-P6-098, BU-P6-100 (the six typed wake-condition kinds and their resume
  rules, homed to the `wake-conditions` shared context, §3 of
  `shared-context-map.md`).

**3. Lower-rung representations attempted.**
- An actor stage that itself polls/sleeps in a loop until the condition holds
  (§6.4/§6.5).
- A helper script re-invoked by an external scheduler (cron/`dagr`) — exactly
  today's mechanism (§6.5/§6.6).

**4. Why each lower rung fails.**
- *Polling actor stage:* either burns a live model turn indefinitely — wasteful,
  and incompatible with the per-turn-process execution model D2 measured and
  shipped (headless `claude -p` turns have no held session to sleep inside) —
  or exits after one check, at which point retry/backoff/deadline state has
  nowhere durable to live except back in the runtime; an actor cannot resume
  itself after its own process has exited.
- *Externally-scheduled helper:* works only as long as something outside
  Sergeant keeps re-invoking it, and the retry/backoff/deadline bookkeeping
  (attempt count, next-not-before, deadline) is then owned by loose files
  rather than the runtime's own event-sourced Work state — unrecoverable,
  unqueryable, and unreasoned-about the way every other execution fact in
  sergeant-rs is (CLAUDE.md: "the journal is the only truth"). **A lower rung
  was attempted in production and is exactly what ships today** — the
  strongest form of evidence §6.7 can receive that no lower rung suffices.

**5. Minimum runtime capability required.**
The runtime must be able to durably schedule a bounded, backoff-and-deadline-
governed re-evaluation of an external condition for a specific Work, owning
attempt count and next-eligible-time as journaled facts, and transition that
Work automatically (met → resume, deadline exceeded → `failed`, max attempts →
`needs_input`) without depending on an external scheduler to keep invoking
anything. **Adversarial narrowing (synthesis.md §5):** the claim's own
"first-class waiting state" framing overreaches — the engine already
separates `waiting`, `needs_input`, and `blocked` (BU-P5-066 records this
explicitly). The gap is *only* the scheduler: a journaled timer plus a small
set of runtime-evaluated predicates plus automatic stage re-entry. The claim
is amended down to that scope.

**6. Observable acceptance test.**
A Work is put into a waiting state with a `github_check` condition and a
deadline; without any external cron job running, the daemon itself
re-evaluates the condition on a bounded schedule, resumes the Work
automatically the first time the check reports success, and — if the deadline
passes first — the Work is observed (via the API) to have failed with
"deadline exceeded", all reconstructable from the journal alone after a
daemon restart mid-wait.

**Why rank 1 (synthesis.md):** three independent partitions found it from four
different artifact classes (a command, a template, a test, a doc), the
lower-rung failures are mechanical rather than aesthetic, and the acceptance
test is directly executable.

---

### G2 — Fleet identity: a durable parent grouping N member Works with dependency edges
**Verdict:** survives, split acceptance test · **Rank 2**
**Merges:** BU-P5-065, BU-P6-016, BU-P6-017.

**1. Behavior that cannot be represented.**
One logical cross-repository task must durably group N independent per-repo
Works — each its own isolated git worktree plus its own agent execution —
under one identity; track each member's individual state; honor declared
dependency edges between members (e.g. `smith-infra>smith`); expose
fleet-scoped read views and operations (respond to one member, cancel/clean up
the whole fleet); and admit a dependent only once its declared prerequisite
reaches a terminal successful state — while each member independently remains
retryable and recoverable.

**2. Source evidence requiring it.**
- BU-P5-065 — `reference/sergeant-upstream/skills/dispatch/SKILL.md` line 84
  (`sha256:ba022bfbae22e508cf37cf2709c2e9374dbf82d54fc4b545301ec1a7d5b410f2`):
  dispatching a task creates durable, inspectable fleet state describing
  every worker spawned under that task, at a well-known location
  (`~/.local/share/sergeant/fleet/<task-id>/`); `sgt-watch` aggregates
  per-repo status into one live fleet table; `--deps a>b,a>c` dependency
  notation; `sgt-cleanup <task-id>` retires the whole task's worktrees and
  state at once.
- BU-P6-016 — `reference/sergeant-upstream/bin/sgt-dag-dispatch-hook` L1-7
  (`sha256:89dc339b7b90ab3f2d65df248d1034b28553cd7363d5f2735142af23ce281c3d`):
  a DAG stage becoming ready dispatches a normal Sergeant work unit, tagged
  with the external DAG's run/stage identifiers for completion reporting.
- BU-P6-017 — `reference/sergeant-upstream/bin/sgt-dag-run` L12-26
  (`sha256:d1f6d4190e9d9d8b03f1ac86e2f66ddd6e9510de25779c2df5586664ee722f18`):
  a project's DAG is declared as data (named stages, each naming repos, a `td`
  task or literal brief, and `after:` dependency edges); `sgt-dag-run` reads
  the declaration and starts a run.
- Decisive corroboration: `sgt-dag-run` / `sgt-dag-dispatch-hook` exist
  *solely* to bolt an external scheduler (`dagr`) onto dispatch — a lower rung
  was attempted in production and abandoned in favor of an external tool, the
  strongest kind of evidence §6.7 can receive.

**3. Lower-rung representations attempted.**
- One Work with per-repo dispatches as ordinary sequential stages of that one
  Work.
- Each repository's dispatch as its own independent top-level Work,
  correlated only by a shared task-id string embedded in each Work's metadata.
- Fan-out and dependency edges represented purely as authored workflow
  content — a stage's `CONTEXT.md` instructs the actor to submit N separate
  `sgt run` invocations and poll them.

**4. Why each lower rung fails.**
- *One Work, N stages:* today's Work owns exactly one Git surface (proposal
  §3.6, `runtime/surface.rs`) and one backend/session for the whole run
  (§3.5); a stage cannot fan out into N independently-progressing,
  independently-recoverable native harness sessions each in its own worktree,
  and a single stage's completion is one `BackendSignal`, not N concurrent
  ones.
- *N correlated top-level Works:* the engine has no fleet/parent identity to
  query — dependency edges cannot be enforced or even observed by the engine,
  only inferred by an external actor re-reading each Work's state one at a
  time; a crash mid-fan-out leaves no journaled record of which Works belong
  to which fleet or what the intended dependency graph was, exactly the
  ambiguity CLAUDE.md's "ambiguity fails closed" invariant says must not be
  silently guessed past.
- *Actor-issued fan-out via authored content:* explicitly rejected by §7.7/
  §10.3 — it hides the parent/child relationship from the parent workflow, and
  dispatch's own contract already requires the visibility that this loses
  (`sgt-watch`'s live per-repo status table; `sgt-respond` addressed by both
  task-id and repo).

**5. Minimum runtime capability required.**
A durable fleet/parent-task identity that groups a fixed or
dynamically-declared set of member Works, records declared dependency edges
between members, exposes fleet-scoped read views through the API, and
supports fleet-scoped operations whose effects are journaled against both the
member Work and the fleet identity. **Adversarial narrowing / split
acceptance test (synthesis.md §5, conflict X15):** BU-P5-074 states plainly
that `--deps` *enforcement* is "left entirely to the dispatched workers" —
`sgt-dispatch` itself evidences only the *recording* half of the claim. The
enforcement evidence comes from the DAG units instead, where an external
scheduler genuinely advances stages on completion. The claim is therefore
split: **(a)** grouping identity and fleet-scoped views (evidenced by
dispatch) and **(b)** advance-on-completion (evidenced by the DAG hook). Both
are required and are evidenced by different artifacts.

**6. Observable acceptance test.**
Submitting one cross-repo task creates one fleet record referencing N member
Works with declared dependency edges; the API can list all N members and
their states under the fleet id; a dependent member is held (not merely
advised) until its declared prerequisite member reaches a terminal successful
state, purely from journaled evidence; canceling or cleaning up the fleet id
transitions/removes exactly its N members, verified after a daemon restart
mid-fleet.

---

### G3 — Durable outbound notification queue with an acknowledgement gate on cleanup
**Verdict:** survives, amended — re-file at reduced scope · **Rank 3**
**Claim:** BU-P8-007.

**1. Behavior that cannot be represented (as amended).**
A Work's own terminal cleanup must be gated on an external consumer's
acknowledgement of that Work's terminal/needs-input transition (or an
operator's explicit retirement of the notification) — an acknowledgement gate
survivable across daemon restarts, independent of any Work's own process or
session lifetime.

**2. Source evidence requiring it.**
- BU-P8-007 — `reference/sergeant-upstream/docs/callbacks.md` L165-167
  ("Retry And Recovery")
  (`sha256:a4ef2a91af6a9d0dca21cfac2399d27e4767e1a35d31efdde155f075190bff2c`):
  an external caller needs a durable, retried, deduplicated,
  exactly-once-delivered notification of a Work's terminal/needs-input state,
  surviving daemon/process restarts, completing on its own schedule
  independent of any coordinator pane, session, or model turn. `state.json`
  tracks pending/delivering/acknowledged/rejected with attempt count and
  backoff; `sgt-callback drain --all` runs as a session-independent periodic
  drain; `sgt-cleanup` refuses full deletion until `sgt-callback check-acked`
  succeeds. Recorded confidence: `medium` (correctly, per synthesis.md).

**3. Lower-rung representations attempted.**
- A stage inside the originating Work's own workflow (§6.3).
- A helper script an actor stage invokes when the Work's state changes
  (§6.5).
- Workflow-local or shared-context data (profile names, timeouts) read by a
  stage (§6.6).
- **Adversarially added (synthesis.md §5), the rung the original claim never
  attempted:** an external subscriber tailing the journal/SSE stream and
  owning its own dedup and retry.

**4. Why each lower rung fails.**
- *Stage inside the originating Work:* ends when that Work's execution ends;
  but acknowledgement, retry, and the cleanup-blocking gate must all keep
  functioning after the Work's own process — and even its own execution
  context — is gone, including across daemon restarts.
- *Helper invoked at one transition:* cannot express a periodic,
  session-independent drain across every pending event for every Work in the
  fleet, nor a durable claim/backoff state safe under concurrent drains.
- *Shared context:* can hold static configuration but not the per-event
  mutable state machine (claimed, stale-claim recovery after 60s, exponential
  backoff, terminal seal) that must be read and written safely from a process
  wholly separate from the Work that created the event.
- *External subscriber tailing journal/SSE (the rung that genuinely
  works, in part):* this rung **does** cover delivery and dedup — an external
  party can legitimately own retry against a durable stream. What it does
  **not** cover is the cleanup gate: a Work's own retirement being blocked
  until an external party acknowledges is not something an external
  subscriber can grant itself; only the runtime, which owns the Work's
  terminal-cleanup path, can refuse to complete it.

**5. Minimum runtime capability required.**
Narrower than the original claim: the runtime must own an
**acknowledgement gate on terminal cleanup** — not necessarily the whole
delivery queue. Concretely: a durable per-Work fact ("cleanup blocked pending
external ack, or explicitly retired") that the Work's own cleanup path
consults, decoupled from who or what performs delivery and retry.

**6. Observable acceptance test.**
Given a Work reaches a terminal state while no coordinator or worker process
is attached, and the daemon restarts before an external consumer
acknowledges, the Work's cleanup remains blocked (observable via the API)
until either an acknowledgement is recorded or an operator explicitly retires
the notification — regardless of which component (runtime queue or external
subscriber) performed the actual delivery attempt.

**Required amendment (synthesis.md §5):** the claim must be re-filed at this
reduced scope before it is counted as accepted for implementation purposes;
it is recorded here as surviving *at the narrowed scope only*.

---

### G4 — Operator-declared, durable, scope-qualified admission block
**Verdict:** survives — high evidence, low cost · **Rank 4**
**Claim:** BU-P6-063 (corroborated by BU-P8-077, BU-P8-078, BU-P6-057,
BU-P6-058, BU-P6-062, BU-P6-064 — all classified `stage`/`stage-context`/
`agents-invariant` in their own right, homed to `draft-workflows/drain-fleet/`
and `permanent-instructions.md` respectively, not to this claim's census).

**1. Behavior that cannot be represented.**
An operator-declared, durable, scope-qualified (global or per-project)
admission block that: **(a)** refuses new stage/turn admission the instant it
is set; **(b)** never terminates in-flight work; **(c)** supports a bounded
wait that converges when every in-scope turn finishes naturally; and **(d)** on
timeout names precisely what is still unresolved, by exact identity — with an
unverifiable worker blocking the wait rather than being silently counted as
drained.

**2. Source evidence requiring it.**
- BU-P6-063 — `reference/sergeant-upstream/bin/sgt-drain` L20-25
  (`sha256:9e2809d93e9cf04c201caf7649ff6a733751122bc1e6eb606be9b1d97bcfc89c`):
  a cooperative drain of a scope is a bounded wait, not an instantaneous
  switch — running workers finish their current turn and exit, new admission
  is refused immediately, and the wait bound reports every unresolved worker
  by exact identity on timeout while leaving them running.
- `bin/sgt-drain`, `bin/_sgt-drain.sh` (admission lock + drain files), and
  `bin/sgt-drain-force` (the confirmed, drain-scoped escape hatch) together
  implement roughly 1,100 lines of file-based admission-control and
  liveness-verification state.

**3. Lower-rung representations attempted.**
- A helper an actor stage calls to check "should I start" (§6.5).
- A per-Work cancellation/timeout mechanism (proposal §15.5/§15.6).

**4. Why each lower rung fails.**
- *Per-stage helper check:* cannot durably and atomically block admission
  fleet-wide against a concurrent dispatch racing to start — exactly the
  admission-lock race the old mechanism exists to close; a stage-local helper
  has no cross-Work visibility to close it.
- *Cancellation/timeout (§15.5/§15.6):* governs one execution's own
  lifecycle; it has no concept of "block starting NEW executions across the
  whole daemon or one project, while leaving already-running ones alone" —
  the entire point of a drain.

**5. Minimum runtime capability required.**
The runtime itself must own a durable, queryable admission-block fact (global
or project-scoped) that every stage-launch path consults atomically before
starting a new execution, plus a way to enumerate in-flight executions in
scope so a bounded wait can report exactly what remains. **Adversarial note
(synthesis.md §5):** the minimum capability is *small* — a durable flag on
daemon state consulted atomically by every stage-launch path, plus an
enumeration of in-flight executions — closer to an R2/R5 extension of
existing machinery than R7 new machinery. Ranked **high evidence, low cost**:
the best first candidate to implement of the six survivors.

**6. Observable acceptance test.**
Setting a project-scoped drain immediately causes any subsequent stage-launch
attempt for that project to be refused (observable via the API) while a
currently-running stage for that project is left running to completion;
querying drain status during the wait lists the still-running stage by
Work/stage identity; clearing the drain restores admission without touching
any in-flight execution.

---

### G5 — Data-dependent, variable-length human-input rounds inside one procedure
**Verdict:** **rejected on re-derivation** (adjudication-round1.md A13) —
moved to "Rejected claims" below. This document's first pass placed G5 among
the survivors, narrowed to answer contract Unknown U3; Round-1 adjudication
found that narrowing's "never attempted" lower rung already shipped, so the
claim did not survive re-derivation. See the full entry under "Rejected
claims" for the original claim, the original narrowing, and why it now
fails — nothing about the original reasoning is deleted, only its verdict.

---

### G6 — Conditional invocation of a named child procedure with its own checkpoints
**Verdict:** survives, partially — downgraded to grammar pressure · **Rank 6**
(originally ranked below G5; with G5's rejection at adjudication-round1.md
A13, G6 is now the fifth and last surviving claim — rank left as originally
assigned by `synthesis.md` §5 rather than renumbered, since no reviewer
re-ran the ranking exercise itself, only G5's individual verdict)
**Claim:** BU-P5-077 (corroborated by BU-P7-007's five-way routing, itself
classified `stage` not `engine-gap` — see rejected list's absorption note
pattern; homed to `draft-workflows/worker-mission/`).

**1. Behavior that cannot be represented (as downgraded).**
Mid-procedure, a worker selects one of several named durable procedures by
judgment about the nature of the work, runs it with that procedure's own
checkpoint/retry/evidence semantics, and returns to the parent — representable
today only by losing four specific, named properties (parent/child
visibility, deterministic cancellation reaching into the child, parent-aware
recovery, and per-subworkflow telemetry); not "cannot be represented at all."

**2. Source evidence requiring it.**
- BU-P5-077 — `reference/sergeant-upstream/skills/dispatch/SKILL.md` lines
  168-173
  (`sha256:820339b3681be58cb0865078f31797659215981209cabdb1cfeba753acb950f1`):
  before implementation, a worker routes to the canonical engineering skill
  matching the nature of the work in a fixed precedence — huge/foggy work
  escalates to `wayfinder`/`to-spec`/`to-tickets` (never silently executed as
  implementation); hard bug/perf work loads `diagnosing-bugs`; uncertain
  logic/UI work loads `prototype`; approved implementation loads `tdd`;
  merge/rebase conflicts load `resolving-merge-conflicts`.
- Corroborating: BU-P7-007 — `reference/sergeant-upstream/templates/worker-brief.md`
  §"2. Route the work" — the same five-way triage generalized across the
  whole brief, classified `stage` (`software-change`/`10-route-work`) rather
  than `engine-gap` in its own partition; both are evidence for the same
  underlying seam.
- Corroborating (N1 adjudication A8, BH-10): `draft-workflows/cross-repo-work`'s
  `60-reconcile` and `draft-workflows/dispatch`'s fleet reconciliation
  (formerly `40-reconcile-before-launch`, folded under A4 into `80-monitor`)
  were drafted with the same fleet-wide reconciliation checkpoint duplicated
  in both packages — the drafter's un-argued instinct being that
  `cross-repo-work` should simply invoke dispatch's reconciliation procedure
  rather than re-derive it. Adjudicated: `dispatch` owns fleet reconciliation
  (its source, `BU-P8-070`, is the authority for the automatic pre-launch
  sweep); `cross-repo-work/60-reconcile` narrows to repo-set-specific
  completion facts for the repos in its own Work (PR URLs/heads, CI, review
  threads, merge and deployment order, terminal task/fleet state for *this*
  plan's repos) and its stage context *names* dispatch's reconciliation as an
  adjacent, owned procedure without pretending to invoke it — under
  `convention.md` §4 rule 1, a `@@name`-style reference cannot smuggle in a
  child-workflow invocation, and no `kind = "execute"` or child-Work stage
  exists to invoke it for real. The duplication pressure is the same seam
  G6 already names: two independently-drafted packages each wanted to hand a
  bounded sub-procedure to a named other workflow and get its outcome back
  in their own trajectory, and today's engine gives them no rung between
  "restate the procedure" and "silently shell out" to do it. This is
  evidence for G6, not a new claim (G6's downgrade and acceptance test
  already cover it).

**3. Lower-rung representations attempted.**
- Shared actor context (a `@@name` reference the worker's stage context
  includes).
- Each of the five candidate procedures treated as a shared helper the
  worker's stage invokes.
- The worker silently issuing its own `sgt run` for the chosen procedure and
  polling it to completion before continuing.

**4. Why each lower rung fails.**
- *Shared context:* only lets the actor read guidance text (§10.2); it
  cannot grant the chosen sub-procedure its own durable checkpoint
  boundaries, retry, blocked/`needs_input` state, or evidence — those exist
  only if the sub-procedure runs as Sergeant-tracked Work with its own
  journaled stages.
- *Treating the five procedures as helpers:* fails the §6.5 test directly — a
  helper is deterministic machinery subordinate to the calling stage's
  outcome, but `diagnosing-bugs`, `tdd`, and `prototype` are themselves full
  multi-step, judgment-requiring procedures (proposal §8.2 independently
  calls `diagnosing-bugs` "a strong low-ambiguity reference workflow"), not
  deterministic subordinate operations.
- *Silent actor-issued `sgt run`:* **adversarially reduced (synthesis.md
  §5): this rung actually works**, under a strict reading of §6.7 ("cannot be
  represented faithfully"). §7.7/§10.3's rejection of it is a *design
  preference* about visibility, not a measured inability. What it loses is
  four named properties: parent/child visibility, deterministic
  cancellation reaching into the child, parent-aware recovery, and
  per-subworkflow telemetry.

**5. Minimum runtime capability required.**
A stage class (or stage attribute) that can invoke a named other workflow as a
real child Work chosen from a bounded declared set (or at minimum an explicit
judgment-bound choice recorded as evidence) at stage-entry time; the child
runs with its own journaled stages/attempts, and on child completion, block,
or failure returns control to the parent stage with the child's outcome
visible in the parent's trajectory and to parent-aware recovery. **Downgrade
(synthesis.md §5):** because the third rung (actor-issued `sgt run`) is
representationally *sufficient*, just lossy, this claim is downgraded from
"cannot be represented" to "can be represented only by losing four named
properties." It survives because the §6.5 helper-test failure is genuine and
unavoidable, but is ranked last of the six survivors and should be re-filed
as **grammar pressure for real parent/child Work identity**, not a blocking
gap.

**6. Observable acceptance test.**
A workflow stage declares a set of permissible child workflows; when the
actor selects one and it runs to completion (or blocks), the parent Work's
trajectory shows the child Work's id and terminal outcome; canceling the
parent transitions the child; a daemon restart mid-child-run recovers both
parent and child from journaled evidence without an operator having to guess
which child, if any, was in flight.

**Note (synthesis.md §8, refute-stage flag):** W21 `prototype`'s A/U branch
and W30 `triage`'s non-linear state machine are the same pressure at lower
intensity and correctly raised no separate claim — each is a fresh invocation
of a stage chosen by judgment, not a control-flow construct the runtime must
own, so G6 is not duplicated by them.

---

## Rejected claims

Each rejection names the lower rung — or existing invariant — the behavior is
actually absorbed by, per this milestone's binding rule that a rejection must
be evidence-grounded, not merely asserted.

### G5 — Data-dependent, variable-length human-input rounds inside one procedure
**Rejected on re-derivation (adjudication-round1.md A13).**
**Claim:** BU-P5-025.

**Original verdict (superseded).** This document's first pass judged this
claim to *survive*, narrowed, as the answer to contract Unknown U3, on the
premise that its "never attempted" lower rung — a re-enterable needs_input
stage — was a genuinely new engine fact. A13 required re-verifying that
premise against the actually-shipped engine rather than the proposal text.
It is false: the rung is already shipped. The original claim, evidence, and
narrowing are kept below unedited (per this document's own completeness
requirement — a superseded verdict is not silently deleted), followed by the
re-derivation that overturns it.

**1. Behavior originally claimed (as narrowed by the first pass).**
A single stage's procedure must be able to ask N sequential, dependent
questions where N is determined by earlier answers, stopping for each answer
before formulating the next, so an early answer can end the interview before
later questions are even asked — with the total round count not known until
the user answers earlier rounds.

**2. Source evidence.**
- BU-P5-025 — `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md`
  lines 169-170
  (`sha256:6f1054cb57d34e7c66b73513aee1da04745c1d7d45f84a0e59c87849346a7c22`):
  "Ask these questions in order; stop and wait for each answer before
  proceeding," with a per-repository sub-loop and a per-named-group sub-loop
  whose round counts are decided by the user's own prior answers (lines
  169-184); corroborated by the same wait-for-answer pattern at the clone
  destination question (lines 109-114) and the Phase 5 backup/diff/confirm
  sequence (lines 204-216). Recorded confidence: `medium`.

**3. Lower-rung representations the first pass considered attempted.**
- One actor stage per phase, with the whole multi-question interview embedded
  in one `CONTEXT.md` turn — rejected: a bounded headless `claude -p` turn
  cannot suspend mid-turn and resume the same reasoning context.
- One coarse Work `needs_input` round-trip per phase, collecting all of a
  phase's answers together — rejected: loses early-stop.
- One stage per individual question, splitting Phase 4 into a fixed number of
  stages — rejected as insufficient for the per-repository/per-group loops,
  whose round count is not known until runtime.
- (First pass's own narrowing) a re-enterable stage that ends with
  `needs_input`, journals the answer, and is re-entered as a fresh execution
  — judged at the time to require "one new engine fact: a stage may be
  re-entered an unbounded number of times with its prior answers journaled
  against it," and therefore counted as an engine gap rather than a fourth
  attempted-and-working lower rung.

**4. Re-derivation: the assumed-missing rung is shipped.**
`docs/icm/record-shapes.md` §5 rule 2 requires a rejection to name the
absorbing rung concretely, not merely assert the behavior is representable.
Verified directly against the shipped engine, not the proposal text (the
same discipline CLAUDE.md's LESSONS L1 requires for the Claude adapter,
extended here to the engine itself):
- `src/domain/work.rs`'s `WorkState::can_transition`:
  `Waiting | NeedsInput => matches!(to, Active | Blocked | Failed |
  Canceled)` — a work in `needs_input` legally transitions back to `active`.
  `WorkState::for_event_kind` maps `KIND_WORK_NEEDS_INPUT` to
  `WorkState::NeedsInput`, so this is a real, journaled event-sourced
  transition, not a proposal-stage sketch.
- `src/api.rs`'s `work_input` handler — doc comment "`POST
  /v1/work/{id}/input` — answer a work that asked for input (§12's
  needs-input verb; `work.respond` in §26's command vocabulary)" — calls
  `engine.provide_input(&mut core, &id, &req.input)`, resuming the work from
  `needs_input` with the delivered answer journaled against it.
- `src/api.rs`'s `work_retry` handler — doc comment "`POST
  /v1/work/{id}/retry` — re-enter the current stage (§12's retry verb)" —
  calls `engine.retry(&mut core, &id)`. Re-entering the current stage as a
  fresh execution is exactly the mechanism the first pass's narrowing
  proposed as the missing capability.

Composed — a stage ends in `needs_input`, the answer is journaled via
`work.respond`, and the stage is re-entered via `work.retry` for the next
round — the shipped engine already has every primitive the narrowed claim's
"minimum runtime capability required" asked for. No new durable fact needs
to be owned by the runtime for the behavior BU-P5-025 describes; the
question of *whether* to retry (continue the interview) versus let the stage
complete (early-stop) is exactly the ordinary stage-completion judgment an
actor already makes at the end of any turn, using the accumulated answers it
reads back from the journaled `needs_input` responses.

**5. Why this is a rejection, not merely a narrower re-scoping.**
The first pass's §6.7 step 3 (lower rungs attempted) was incomplete by
omission, not by genuine exhaustion: "one coarse `needs_input` round-trip
per phase" was the only needs_input-shaped rung it considered, and it
correctly rejected *that* rung (it loses early-stop). It never evaluated the
finer-grained rung actually available — one `needs_input` round-trip **per
question**, with `retry` re-entering the same stage between rounds — which
is representable today and preserves early-stop exactly: the stage ends
(`needs_input`) after each question and is retried only if the interview
continues; an early "zero repositories" answer ends the interview by the
stage completing instead of retrying, with no unbounded suspension of a
single turn required.

**6. Disposition.**
Rejected per adjudication-round1.md A13: "its 'never attempted' lower rung
(needs_input round-trips) exists in the shipped engine; re-file only if a
measured N2 run shows the narrower residual gap the finding describes." No
such N2 measurement exists yet. If a real N2 generation run finds the
per-question needs_input/retry loop insufficient in practice — for example,
if an actor's fresh `retry` execution cannot reconstruct enough context from
journaled answers alone to ask a coherent next question — this is the
correct claim to re-file, narrowed to that specific, measured residual, not
to the whole interview shape.

**7. Cross-reference (conflict X8, re-resolved here).** X8 is overturned at
adjudication-round1.md A13 on this same ground: BU-P4-053's "representable
today ... not an engine gap" note stands for BU-P5-025 too, not only for
BU-P3-007/BU-P4-053 as the original PROPOSED resolution held. See
`classification-ledger.md` §3 X8.

---

### G7 — Dynamically-discovered checkpoint graph with a claim primitive
**Rejected.**
**Claim:** BU-P4-090 (supported by BU-P4-082, BU-P4-083, BU-P4-084, BU-P4-098,
BU-P4-100 — all classified `helper`/`stage`/`stage-context` in their own
right, homed to `draft-workflows/wayfinder/` and `helper-map.md`, not to this
rejected claim's census).

**Behavior claimed:** a durable procedure whose full set of sub-checkpoints
(wayfinder decision tickets) is not known in advance, discovered incrementally
as prior checkpoints resolve, claimed by independently-chosen sessions without
duplication, forming a dependency graph computed live.

**Lower rung that absorbs it:** the **shared-context/helper rung (§6.5/§6.6),
with the external issue tracker as the durable store** — exactly what
wayfinder does today, and the source says so explicitly. `BU-P4-082`
(assignment as the claim primitive), `BU-P4-083` (the tracker's native
blocking relationship as the dependency graph), and `BU-P4-084`
(resolution recorded separately from the ticket body) are each faithfully
representable today as `helper` units consumed by the `wayfinder` workflow's
`resolve-ticket` stage — no new durable fact is required for Sergeant to
*use* this mechanism, only for Sergeant to *own* the data the mechanism
produces.

**Why rejected (§6.7 discipline):** the claim's own stated reason the third
lower rung fails is "this works but places every durable fact outside
Sergeant's journal, so Sergeant has no evidence of it." That is an
**ownership preference**, not a representational failure — §6.7 asks whether
the behavior *cannot* be represented, not whether Sergeant would rather own
the data. The concurrency argument is preserved as the strongest part of the
original claim: **if** a future design brings the ticket graph inside
Sergeant for other reasons, an anti-double-claim admission primitive becomes
necessary then — recorded as an engine-pressure *observation* on the
ownership boundary, not a current gap. Substantial overlap with G2 and G6: if
either is implemented, this claim should be re-examined rather than
re-argued from scratch.

---

### G8 — Runtime-enforced actor-role authority for a restricted command
**Rejected.**
**Claim:** BU-P2-056.

**Behavior claimed:** only a coordinator-context actor may invoke the
`no-mistakes` shipping-gate pipeline; a worker or remediation-loop context
must be structurally prevented from invoking it, not merely instructed not
to.

**Lower rung that absorbs it:** rung **§6.1 (repository-wide invariant)** —
the source's *own* primary mechanism, already homed as `agents-invariant` at
Article III of `permanent-instructions.md` ("the shipping gate is
coordinator-owned; workers and remediation loops never run it") via
BU-P1-040, BU-P1-050, BU-P1-054, BU-P1-068, BU-P1-073, BU-P1-110, BU-P1-111,
BU-P1-131, BU-P8-083, BU-P8-091, BU-P8-100.

**Why rejected (§6.7 discipline):** the claim's first lower rung — a
repository-wide instruction — is stated to fail because it "only binds an
actor that reads and honors it." If that reasoning counted as an engine-gap
justification, **every** invariant in the corpus's permanent-instruction set
would be an engine gap, which makes the ladder's rung 6.1 meaningless as a
terminal representation. The third lower rung considered (a role-checking
wrapper) is dismissed for want of an unforgeable role identity, but the
runtime *does* distinguish a Work's own execution context from a client call,
and a helper observing whether it runs inside a dispatched work surface was
never attempted. Recorded as an **authority-model observation**: if a
tool-gating or permission capability is designed for other reasons, this
behavior is its first consumer — not evidence the current architecture is
insufficient today. **Conflict X4** directly contradicts this claim: three P1
units classify the identical behavior as a plain invariant with no gap
implied; that classification is upheld here.

---

### G9 — Crash-safe durable publication (four claims)
**Rejected as gaps; re-homed to existing invariants.**
**Claims:** BU-P7-043, BU-P7-079, BU-P7-049, BU-P7-051 (all classified
`engine-gap` in their own partition-local records but rejected here on
independent grounds per claim).

Each of the four is rejected individually, with its own absorbing rung/
invariant:

- **BU-P7-043** (fault-injected multi-file publication must be crash-safe at
  every internal step). **Lower rung that absorbs it:** the claim's own
  minimum-capability text names it — **"the append-only journal itself, per
  this repo's own architecture."** The capability it asks for **already
  exists**: sergeant-rs's journal-as-only-truth design (CLAUDE.md) is
  precisely the durable, engine-owned append-once record this claim requests.
  It is a requirement already satisfied by the target architecture, not a gap
  in it.
- **BU-P7-079** (cleanup's atomic publish/rollback must survive a
  cross-filesystem worktree/fleet-state split). **Re-derived on its own
  evidence, per adjudication-round1.md A13.** The original rejection ground
  (below, superseded) was circular with `classification-ledger.md`'s
  conflict X1 and has been severed; A13 required re-deriving this claim's
  verdict independent of that circularity. It **remains rejected**, but now
  on direct architectural grounds rather than a citation-priority argument.

  *Original ground (superseded, kept for the record):* "rung §6.1
  (agents-invariant), pointed the opposite direction from the claim —
  `BU-P8-108` states Sergeant refuses a cross-filesystem layout outright...
  This is conflict X1 — tests (P7) would normally outrank docs (P8) per §5
  of the extraction method, but P7's unit is itself a rejected engine-gap
  claim, so the durable rule is P8's refusal." This is circular: X1's own
  PROPOSED resolution discounted BU-P7-079's weight *by citing this very
  rejection*, and this rejection in turn cited X1's preference for P8 —
  each argument's only support was the other. `classification-ledger.md`
  now adjudicates X1 the other way (BU-P7-079's statement wins the
  extraction-priority question, since tests outrank docs once the circular
  citation is removed), which makes the old rejection's reasoning unusable
  regardless of which way X1 lands.

  *Re-derivation (independent of X1, verified against the shipped
  engine, not either source document):* the cross-filesystem split
  BU-P7-079's test injects — a worktree and a fleet-state directory on
  different filesystems, forcing a copy-based rename fallback — cannot
  arise in sergeant-rs's architecture, by construction:
  - `src/runtime/surface.rs`'s `surface_root` materializes every worktree
    at `<data_dir>/surfaces/<work_id>/...`, inside the daemon's own single
    data dir, and `SurfaceError::InsideSourceCheckout` refuses a surface
    that would live inside a source repository at all ("surfaces live in
    the daemon data dir, outside every checkout (§11)"). There is no
    configuration in which a worktree sergeant-rs manages sits outside the
    same data dir the journal and fleet state live in.
  - `src/runtime/fsutil.rs`'s `write_atomic_mode` — the codebase's one
    atomic-publish primitive — creates its temporary file as a sibling of
    the destination path (`path.with_extension(...)`) and renames it into
    place; a sibling of a path is by definition on the same filesystem as
    that path, so this rename can never cross a filesystem boundary. No
    code path in sergeant-rs publishes state via a rename whose two
    endpoints could be on different filesystems.
  - Per CLAUDE.md's "One owner" invariant, the daemon exclusively owns one
    data dir; fleet/work state (the journal) and every worktree it creates
    live under that one directory, never split across two independently
    mounted locations the way old-Sergeant's `~/.local/share/sergeant/
    fleet/<task-id>/` (fleet state) and an arbitrary user checkout
    (worktree) could be.

  **Lower rung that absorbs it (re-derived):** the single-data-dir/
  single-owner construction itself (CLAUDE.md; `src/runtime/surface.rs`
  §11 comment; `src/runtime/fsutil.rs`'s sibling-rename pattern) — not
  `BU-P8-108`'s documentation citation. The behavior the claim's test
  probes (a race during a cross-filesystem rename-with-fallback) has no
  scenario to occur in, because sergeant-rs never produces the
  cross-filesystem precondition in the first place. This finding does not
  depend on which of BU-P7-079/BU-P8-108 the corpus prefers as its
  *extracted statement* (that is X1's question, now settled the other way,
  in BU-P7-079's favor); it depends only on reading the actual shipped
  surface/fsutil code.
- **BU-P7-049** (a two-party handshake must converge on independently
  verifiable proof rather than refusing forever when the recording party
  dies). **Lower rung that absorbs it:** CLAUDE.md's **adjacent-append
  crash-window hazard** plus **Article IV**'s "evidence over optimism, fail
  closed never fail silent" — an invariant the architecture already holds
  structurally and `src/runtime/recovery.rs` already implements (recovery
  resumes only on unambiguous evidence; ambiguity fails closed into
  `blocked`, never a guess, but a *provable* completion converges rather
  than refusing forever).
- **BU-P7-051** (every worker-exit branch must settle its lease through one
  shared finalizer). **Lower rung that absorbs it:** the daemon's **single
  terminal-transition funnel** — in the daemon there is exactly one funnel
  through which a Work's terminal transition passes; the seven-exit-branch
  omission class this claim guards against is an artifact of hand-written
  Bash exit paths that sergeant-rs's architecture does not reproduce by
  construction.

**Re-homed as three architecture invariants (already stated, not built):**
publication is crash-safe and convergent on retry; a two-party handshake
converges on independently verifiable proof rather than refusing forever;
terminal transition is a single funnel. Added as evidence *behind* Article IV
of `permanent-instructions.md`, not as new engine-pressure — these are the
corpus's best external corroboration that the existing invariants are the
right ones, a genuinely valuable finding even though all four gap claims
fail. (A reviewer may reasonably challenge this as the synthesis grading its
own architecture favorably — flagged explicitly in `synthesis.md` §8 for the
refute stage, and repeated here per this document's own completeness
requirement.)

---

## Roll-up

| Claim | Units (census) | Verdict | Rank |
|---|---|---|---|
| G1 wait/wake scheduling | BU-P6-099, BU-P7-010, BU-P7-096 | survives (narrowed to the scheduler) | 1 |
| G2 fleet identity + dependency advance | BU-P5-065, BU-P6-016, BU-P6-017 | survives (split acceptance test) | 2 |
| G3 acknowledgement gate on cleanup | BU-P8-007 | survives (amended, re-file at reduced scope) | 3 |
| G4 admission block | BU-P6-063 | survives (high evidence, low cost) | 4 |
| G5 re-enterable needs-input stage | BU-P5-025 | **rejected on re-derivation** (adjudication-round1.md A13 — lower rung ships today) | — |
| G6 child-workflow invocation | BU-P5-077 | survives partially (downgraded to grammar pressure) | 6 (now last surviving) |
| G7 dynamic ticket graph | BU-P4-090 | **rejected** — absorbed at §6.5/§6.6 (shared-context/helper) | — |
| G8 runtime role enforcement | BU-P2-056 | **rejected** — absorbed at §6.1 (agents-invariant, Article III) | — |
| G9 crash-safe publication (×4) | BU-P7-043, BU-P7-049, BU-P7-051, BU-P7-079 | **rejected** — three absorbed by existing architecture/Article IV; BU-P7-079 re-derived at A13, absorbed by the single-data-dir/single-owner construction | — |

**Method observation for N2 (synthesis.md §5, revised at adjudication-round1.md
A13).** Nine of the sixteen first-pass `engine-gap`-classified units survived
in some form and seven did not, but *four of the five survivors needed
narrowing* — most often because the claim's stated minimum capability was
larger than its own evidence required. A future generator's "engine-gap
quality" measurement should score *scope discipline* (is the minimum
capability the smallest thing the evidence forces?) separately from *rung
discipline* (were the lower rungs named and genuinely tried, against what is
actually shipped?) — the corpus shows they fail independently: G1, G2, G3,
G4, and G6 all pass rung discipline, but only G4 required no scope narrowing
at all. **G5 is the sharpest illustration of the rung-discipline axis, added
this round:** its first-pass "lower rungs attempted" step was not
dishonest, just incomplete — it never checked its own proposed narrower rung
(a re-enterable needs_input stage) against the shipped engine, and that rung
turned out to already exist (`src/domain/work.rs`, `src/api.rs`). Rung
discipline therefore means more than *naming* the rungs a claim tried — it
means checking a *candidate absorbing rung* against the real system before
asserting it is missing, the same discipline LESSONS L1 already demands of
the Claude adapter, extended here to the engine itself.

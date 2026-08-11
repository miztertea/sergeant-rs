# 80-monitor: reconcile fleet, create tracked work, acquire surface, render brief, launch and record, then monitor

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-prepare-intent/output/README.md | L4 | upstream artifact produced by `20-prepare-intent` |

## Purpose

Escalations are read in full, human decisions obtained without inference, delivered to the exact task/repo pair. This is the workflow's second and last judgment-bearing checkpoint (N1 adjudication A4): everything that must happen between a prepared intent and a monitorable running worker — bulk fleet reconciliation, all-or-nothing tracked-work creation, per-repo surface acquisition, brief rendering, and launch-with-evidence — is deterministic machinery that folds in here as ordered helper invocations, because none of it carried a judgment argument that survives §6.3's reimplementation test.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

Fleet state is reconciled, tracked work exists all-or-nothing across every target repo, each repo has an isolated work surface, every worker's brief durably carries mission/instructions/dependencies/overrides, launch evidence is written `intended` then promoted to `confirmed` only on observed readiness with every per-repo failure recorded orphaned — and once workers are running, escalations are read in full and human decisions obtained without inference and delivered to the exact task/repo pair.

## Behavior contract

- **needs_input and blocked are distinct nonterminal states for a dispatched worker; a worker waiting on CI, review threads, or dependencies stays in_progress unless it actually needs to escalate.**
  (trigger: sgt-watch observes a dispatched worker; outcome: the operator sees a precise nonterminal state rather than an undifferentiated 'still running')
  — `BU-P5-066`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 92)
- **When a worker escalates, the coordinator must read its full context, evidence, exact question/blocker, recommendation, and options; obtain an explicit human decision without inferring consequential intent; and deliver that decision to the specific task/repo pair via the response-delivery command, which durably writes the response to fleet state before notifying the worker.**
  (trigger: a dispatched worker escalates; outcome: the human decision is fully informed, explicit, durably recorded, and delivered to the correct worker)
  — `BU-P5-067`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 94-98)
- **After a response is delivered, the worker must consume/remove the response, clear its escalation message, log the decision to td, and return to in_progress before continuing.**
  (trigger: a response has been delivered to an escalated worker; outcome: the worker's escalation state is durably cleared and the decision is logged before work resumes)
  — `BU-P5-068`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 99)
- **A worker's exit boundary settles the accepted notification action lease for every possible terminal status alike — done, failed, drained, needs_input, blocked, waiting, and orphaned — because every one of those exit paths is an equally valid place for a lease to be silently left outstanding forever if it is not settled at that single, unified boundary.**
  (trigger: a worker process is exiting, regardless of outcome; outcome: a notification's action-lease fate is always known — finalized or explicitly pending with a reason — no matter which of the seven exit paths the worker took)
  — `BU-P6-113`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L483-489)
- **A worker's exit is orphaned unless it produced a genuinely terminal status with substantiating evidence: a done status requires a non-empty result or the worker is reclassified orphaned; any other unrecognized status falls back to orphaned by default, so an unclassified exit is never mistaken for success.**
  (trigger: a worker process is exiting; outcome: only a status with real substantiating evidence is ever accepted as a genuine terminal outcome; everything else defaults to the honest, investigable orphaned state)
  — `BU-P6-115`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L490-495, L509-510)
- **For the Claude harness specifically, whether a pinned model was actually honored — rather than silently substituted by the provider — is confirmed only after the run completes, by scanning the session transcript for a known substitution-warning phrase; this check never blocks or changes the mission's outcome, it only records a diagnostic that survives the otherwise-unconditional cleanup of that diagnostic on success.**
  (trigger: a Claude worker with a pinned model has finished its run; outcome: a silent model substitution that a mission still completed 'successfully' is caught and durably recorded, even though nothing about the run's own exit signals failure)
  — `BU-P6-116`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L1109-1117)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim. The helper invocations below are performed first, mechanically, to reach the point where this judgment applies.

## Helper invocations (folded stages, N1 adjudication A4)

Five stages extracted as their own candidates (ladder §6.5, "deterministic-machinery candidate") carried no "Additional note" arguing they survive §6.3's reimplementation test — swapping each operation's implementation would leave this stage's monitoring checkpoint unchanged. They fold in here as ordered helper invocations, run in this order before the actor turns to monitoring. **Order preserves N1 adjudication A3 (BH-01): fleet reconciliation runs before tracked-work creation**, per `40-reconcile-before-launch`'s own contract ("dispatch always runs this reconciliation automatically before creating new work") — the two stages are listed in that order below, not extraction order.

**1. reconcile fleet** (formerly `40-reconcile-before-launch`) — bulk fleet reconciliation runs before new work is created.

- **Bulk fleet reconciliation syncs worktree status into fleet state, stops only identity-verified done or failed worker processes, and marks an interrupted dispatched record failed only once it has had neither a worktree nor an owned live process for a default 300-second grace period (configurable), while it always preserves needs_input, blocked, and orphaned records, and dispatch always runs this reconciliation automatically before creating new work.**
  (trigger: sgt-watch --sync-all runs, or dispatch runs it automatically before new work; outcome: fleet state converges toward truth using identity-verified evidence and a bounded grace period, never a bare liveness guess, and never silently sweeps a needs_input/blocked/orphaned record)
  — `BU-P8-070`, `reference/sergeant-upstream/docs/using-sergeant.md` (L137-155 (Monitor work))

**2. create tracked work** (formerly `30-create-tracked-work`) — all-or-nothing task creation across every target repo, rolled back on any failure.

- **Dispatching from a free-form brief creates exactly one td task per target repository before spawning any worker; if td is unavailable, task creation fails, generated metadata cannot be injected, or any selected repo fails to get a task, the whole dispatch aborts before spawning any worker and rolls back the generated tasks.**
  (trigger: a free-form dispatch is being launched; outcome: task creation for a multi-repo dispatch is all-or-nothing, never partially committed before any agent starts)
  — `BU-P5-088`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 196-198)
- **Creating tracked-work items across several repos for one cross-repo brief is all-or-nothing: every target repo is validated up front (cloned, task tracker initialized) before any task is created, and if creating a task in any repo fails after some were already created, every already-created task is deleted to roll back.**
  (trigger: a cross-repo brief needs one tracked-work item per target repo; outcome: either every target repo ends with exactly one new task, or none of them do — never a partial set left behind)
  — `BU-P6-036`, `reference/sergeant-upstream/bin/sgt-td-create` (L6-8, L192-193)

**3. acquire surface** (formerly `50-acquire-surface`) — an isolated work surface per repo at a deterministic location; a branch already carrying unpushed committed work is refused unless explicitly adopted.

- **Dispatching a task first generates a durable task identity, then creates an isolated git worktree per target repository at a deterministic sibling path.**
  (trigger: the plan is confirmed; outcome: every dispatched repository has its own isolated, addressable working copy under a durable task identity)
  — `BU-P5-061`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 79-81)
- **Once repos have a treehouse pool initialized, dispatch automatically prefers a pre-warmed treehouse lease over a plain git worktree for those repos, without the operator having to select a worktree strategy per dispatch.**
  (trigger: a repo has a treehouse.toml present at dispatch time; outcome: dispatch silently gets faster worktree acquisition once pooling has been set up once, with no per-dispatch flag needed)
  — `BU-P6-019`, `reference/sergeant-upstream/bin/sgt-treehouse-init` (L11, L78-79)
- **Dispatch refuses to re-dispatch onto a branch that already carries committed work unreachable from any remote, unless the operator explicitly opts in with an adopt-branch flag, because a prior interrupted dispatch may have done real, unpreserved work that a fresh dispatch would silently discard or duplicate.**
  (trigger: dispatch is about to reuse an existing branch name; outcome: preserved but unpushed work from an interrupted prior dispatch is never silently lost or duplicated by a fresh dispatch)
  — `BU-P6-125`, `reference/sergeant-upstream/bin/sgt-dispatch` (L776-793)
- **The `--adopt-branch` dispatch option is an explicit operator acknowledgement that a named branch already carries committed work and should be resumed as-is; it is non-destructive (it checks the branch out at a new worktree path, preserving the branch tip and every commit), and it exists so the unpushed-work guard cannot make preserved work permanently unresumable in a repository whose upstream denies push access.**
  (trigger: a re-dispatch targets a branch the unpushed-work guard would otherwise refuse; outcome: an operator has an explicit, non-destructive, provably safe path to resume work on a branch that can never become remote-reachable, instead of being permanently blocked or having to delete committed work)
  — `BU-P7-069`, `reference/sergeant-upstream/tests/sgt-dispatch-adopt-branch-test.sh` (lines 1-12)
- **The unpushed-work guard's refusal message must never instruct the operator to delete the branch (the data loss it exists to prevent) and must instead name `--adopt-branch` as the supported non-destructive reconcile path.**
  (trigger: the unpushed-work guard refuses a re-dispatch; outcome: an operator confronted with a safety refusal is always told the safe recovery path, never a destructive workaround, by construction of the error text itself)
  — `BU-P7-070`, `reference/sergeant-upstream/tests/sgt-dispatch-adopt-branch-test.sh` (lines 78-84)

**4. render brief** (formerly `60-render-brief`) — mission, merged instructions, dependency notes, delivery requirements and any verbatim user override are durably carried to the worker before it starts. The BU-P5-075..089 range below, plus `BU-P5-150`/`151`/`152`/`153` (routed here at N1 verifier round 2, finding V3 — the remaining rows of the same ordered worker-contract list, items 6/16/17/18) is worker-contract content this helper *authors into the brief* but does not itself execute — it is the input to `worker-mission` and `route-review-findings`, not a claim that this stage performs that content's behavior. `BU-P5-074` records that `--deps` *ordering* is expressed here but its *enforcement* is left entirely to the dispatched workers reading their own brief (conflict X15, folded into engine-gap G2's split acceptance test in `reference-corpus/synthesis.md` §5) — recording is this helper's job; enforcement is not.

- **Each worker's starting context must durably carry its mission, the merged/resolved agent instructions, dependency notes, and delivery requirements, before the worker begins.**
  (trigger: a worktree has been created for a dispatched repository; outcome: the worker never begins without a complete, self-contained starting brief)
  — `BU-P5-062`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 82)
- **If a canonical skill referenced by the routing table cannot be loaded, the generated brief's own embedded rules for that phase remain mandatory regardless.**
  (trigger: a canonical skill named by the routing table is unavailable; outcome: the phase's requirements still apply even without the specialized skill loaded)
  — `BU-P5-087`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 190)
- **A worker brief's instruction merge order is defaults, then group, then repo, and an explicit user override embedded in the dispatched brief (e.g. 'run no-mistakes for this worker before completion') must appear verbatim in the rendered brief rather than being silently dropped by the default no-mistakes-ownership instruction it overrides.**
  (trigger: sgt-dispatch renders a worker brief from layered defaults/group/repo instructions plus any explicit per-dispatch override text; outcome: instruction layering (defaults -> group -> repo) and an explicit dispatch-time override both render deterministically and verifiably into the final brief text a worker receives)
  — `BU-P7-112`, `reference/sergeant-upstream/tests/sgt-dispatch-brief-test.sh` (lines 382, 501-511)
- **sgt-dispatch must resolve an OpenCode (`oc`) target session for routing coordinator notifications by consulting `td` for an existing routing task before creating one, so coordinator notification routing reuses existing tracked infrastructure rather than duplicating it per dispatch.**
  (trigger: sgt-dispatch dispatches a worker under an OpenCode coordinator session; outcome: coordinator-notification routing infrastructure (its own td task) is discovered and reused, not silently recreated on every dispatch)
  — `BU-P7-074`, `reference/sergeant-upstream/tests/sgt-dispatch-oc-target-test.sh` (lines 34-40)
- **sgt-dispatch's `td` integration must distinguish an existing tracked task from one that needs to be newly created for a cross-repo brief, and this contract must be exercised against a full copy of every sourced helper (not a hand-picked subset), because a missing helper makes the copy fail at its own source line instead of exercising the behavior under test.**
  (trigger: a cross-repo brief may or may not already have a tracked td task; outcome: dispatch correctly attaches to existing tracked work or creates new tracked work, never silently duplicating or losing the tracking relationship)
  — `BU-P7-075`, `reference/sergeant-upstream/tests/sgt-dispatch-td-test.sh` (lines 19-24)
- **The --deps ordering string only expresses that one repository must finish before dependents can merge; enforcing it is left entirely to the dispatched workers reading it out of their own brief.**
  (trigger: a dependency string is declared for a dispatch; outcome: dependency intent is documented, but nothing outside the worker's own judgment enforces it)
  — `BU-P5-074`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 155-157)
- **Every dispatched worker pins a fixed point -- normally the merge-base with current origin/main -- and records the base SHA, commit list, and diff scope before implementing anything.**
  (trigger: a worker's session starts; outcome: the worker's later diff and evidence are always measured against a recorded, immutable starting point)
  — `BU-P5-075`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 166)
- **Every dispatched worker triages the full originating td issue/spec/comments and linked material, prior or redundant work, category, and readiness, and explicitly records when no originating spec exists.**
  (trigger: a worker's session starts; outcome: the worker never begins implementation without having read and recorded its own originating context)
  — `BU-P5-076`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 167)
- **A worker establishes public behavioral seams from td/spec evidence before writing tests; if a consequential seam is undecided, it escalates needs_input rather than guessing.**
  (trigger: a worker is about to define testable seams; outcome: undecided consequential design points are escalated, never silently assumed)
  — `BU-P5-078`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 174)
- **A worker implements one vertical slice at a time: a focused failing test, the minimum passing implementation, then refactor; tautological tests, internal mocking, horizontal test/implementation phases, and speculative refactoring are all rejected.**
  (trigger: a worker begins implementing an established behavioral seam; outcome: implementation proceeds in small, test-first, verifiable increments rather than large untested batches)
  — `BU-P5-150`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 175)
- **A worker in needs_input or blocked writes an escalation message and notifies the coordinator; on response, it consumes/removes the response, clears the message, logs the decision to td, restores in_progress, and continues -- the durable requirement is that answering a blocked worker always durably restores forward progress, regardless of the underlying file-based transport.**
  (trigger: a worker needs input or is blocked; outcome: the escalation-to-resume cycle always ends with the worker durably back in progress)
  — `BU-P5-079`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 176)
- **No-mistakes is run only at an explicit final shipping boundary, never for routine worker completion, prototypes, investigations, documentation drafts, intermediate commits, or remediation loops, unless the user explicitly overrides that default; safety-sensitive work follows a stricter path.**
  (trigger: a worker reaches a candidate shipping boundary; outcome: the expensive shipping gate runs only where it is actually warranted)
  — `BU-P5-080`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 178)
- **Every no-mistakes finding is routed through the finding-routing command into a separate, deduplicated, owning-repo td task; correctness/security/data-integrity/test/ask-user findings are P1 and gated, warning debt is P2, informational debt is P3, and cosmetic/evidence noise is ignored -- and findings are never remediated inside the validation run itself.**
  (trigger: no-mistakes has produced findings; outcome: every actionable finding becomes tracked work at a severity-appropriate priority, and the validation run itself stays read-only with respect to remediation)
  — `BU-P5-081`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 179)
- **Independent review runs as separate parallel subagents for the axes named by a shared axis-vocabulary source (standards, spec, readiness, plus a conditional accessibility axis for UI-facing work identified by role/group/description language), each described by guidance reproduced verbatim in the generated brief; the spec axis is explicitly skipped when no spec exists.**
  (trigger: a worker reaches independent review; outcome: review axes, their guidance, and their applicability conditions come from one canonical source rather than being redefined per invocation)
  — `BU-P5-082`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 180-181)
- **When a finding fails to route after parsing, the router retains a sanitized findings artifact under a fixed path and names the exact retry command; the artifact is retried from, never re-generated by re-running the reviewer, and a retained artifact that has not been retried is never deleted.**
  (trigger: a review finding fails to route; outcome: evidence is preserved and addressable rather than silently dropped or expensively re-derived)
  — `BU-P5-083`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 182)
- **A stored finding-card revision the router cannot prove it authored is preserved below a superseded-revision separator and the card gains a needs-reconciliation label; only the worker owning the finding may merge the two accounts and remove that label -- the router itself never clears it.**
  (trigger: the router would otherwise overwrite a card revision it did not create; outcome: concurrent writers to the same card never silently clobber each other; reconciliation is an explicit, owned obligation)
  — `BU-P5-084`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 183)
- **Findings sharing the same originating run, head, owning module, and root cause share one serialized remediation worker/branch; before merging that group, native tests and independent re-reviews verifying mutation-before-validation, partial-publication/rollback, and identity/provenance are rerun; after two remediation cycles, fix dispatch stops and an architectural/root-cause review plus a human decision is required.**
  (trigger: multiple findings share a root cause; outcome: remediation is deduplicated, re-verified before merge, and escalates to a human after bounded retries rather than looping indefinitely)
  — `BU-P5-085`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 184)
- **A worker remediates every blocking repository-native test and independent-review finding and reruns the affected tests and all required review axes until each reports zero blocking findings; no-mistakes findings are remediated through a separate td dispatch, not inline.**
  (trigger: review or testing has produced blocking findings; outcome: no dispatched work reaches completion while a blocking finding remains open, and no-mistakes findings stay routed through their own separate remediation path)
  — `BU-P5-151`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 185)
- **A worker commits its changes, opens a pull request, waits for required CI to pass, resolves every non-outdated review thread, and satisfies the plan's declared dependency order before considering its work delivered.**
  (trigger: a worker's implementation and remediation are complete; outcome: delivery evidence (PR, passing CI, resolved review, correct merge ordering) exists before the worker's work is considered done)
  — `BU-P5-152`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 186)
- **For tracked work, a worker logs td decisions and hands off, then runs `td review` only once implementation and review evidence are both ready.**
  (trigger: a worker's work is tracked in td and nears completion; outcome: td's own review step is only invoked once its evidentiary preconditions are actually satisfied, never prematurely)
  — `BU-P5-153`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 187)
- **A worker writes its terminal result record and sets its status to done only after every completion gate has passed; a failed status with an exact reason is reserved specifically for an unrecoverable terminal failure.**
  (trigger: a worker reaches a terminal outcome; outcome: the terminal status distinguishes verified success from unrecoverable failure, and neither is asserted prematurely)
  — `BU-P5-086`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 188)
- **Recovering a stuck, stale, or orphaned dispatched worker is done only through the response-delivery command or an equivalent explicit action; a worker is never marked done manually, and a retry writes both the result and the done status only after every completion gate passes.**
  (trigger: a worker appears stuck, stale, or orphaned; outcome: recovery never fabricates a successful terminal state; it either delivers an explicit response or nothing changes)
  — `BU-P5-089`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 221-229)

**5. launch and record** (formerly `70-launch-and-record`) — launch evidence is written `intended` then promoted to `confirmed` only on observed readiness; every per-repo failure records an orphaned status with a diagnostic before the loop aborts.

- **Launch evidence distinguishes intent from proof by a two-state field: it is written as 'intended' before the harness is even invoked, and only promoted to 'confirmed' once the harness is observably ready, so a harness that rejects the pin and exits immediately never leaves evidence claiming the model actually ran.**
  (trigger: launch evidence is written both before and after the harness actually starts; outcome: launch evidence can never overclaim: an aborted launch is never mistaken for a confirmed run)
  — `BU-P6-110`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L86-90)
- **Every repo's identity-switching, worktree acquisition, brief writing, and worker launch is validated in order such that any failure records an orphaned status with a specific diagnostic and a handoff before the dispatch loop aborts, rather than leaving a repo's fleet state silently incomplete.**
  (trigger: any step of launching one repo's worker fails partway through; outcome: a partial dispatch failure always leaves an explicitly diagnosable orphaned state with a handoff, never a silently stuck or ambiguous fleet record)
  — `BU-P6-126`, `reference/sergeant-upstream/bin/sgt-dispatch` (L924-928, L939-943, L949-953, L959-963)
- **The interactive worker must pass its pinned provider/model tuple to the harness process it launches, and the durable launch_record it writes must be verified against the harness's own independently observed argv/environment — never trusted from the worker's internal bookkeeping alone — so recorded launch evidence is provably what actually ran.**
  (trigger: the interactive worker launches its harness process with a pinned provider/model tuple; outcome: recorded launch evidence is independently verifiable against the actual process's own observed argv/environment, not merely self-reported by the launching code)
  — `BU-P7-111`, `reference/sergeant-upstream/tests/sgt-worker-model-tuple-test.sh` (lines 6-9)

## Delegation

This stage's outcome is produced by running **respond-to-worker** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

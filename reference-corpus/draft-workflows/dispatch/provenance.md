# Provenance — Dispatch

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W8** `dispatch`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-004` | In dispatch mode, load context, plan, and decompose the requested outcome by repository before dispatching. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L18, dispatch-mode step 1) |
| `BU-P1-005` | In dispatch mode, create one worker per owning repository via the dispatch procedure. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L19, dispatch-mode step 2) |
| `BU-P6-049` | A canonical intent document is only ever valid if it contains exactly eight required sections, in a fixed order, each with non-empty content — Objective, Required Invariants, Approved Tradeoffs, Out Of Scope, State Transitions, Failure Windows, Negative Test Matrix, Validation Evidence. | `reference/sergeant-upstream/bin/_sgt-intent.sh` (L151-159) |
| `BU-P5-054` | dispatch plans and executes a cross-repo task by dispatching one autonomous subagent per repository, each isolated in its own git worktree. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 8) |
| `BU-P5-055` | dispatch is loaded when a task spans multiple repos and should run in parallel, when the user says something like 'dispatch this' or 'run this across all repos', or when cross-repo-work has already produced a plan the user wants executed. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 14-17) |
| `BU-P1-134` | When dispatch mode is selected or an existing fleet must be operated, load the dispatch procedure, which owns task-tracker integration, worktrees, worker contracts, monitoring, escalation, reconciliation, and cleanup. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L115, Procedural skills table row) |

Routed here at N1 verifier round 2 (finding V3): `BU-P1-134` is AGENTS.md's own Procedural-skills-table row for this workflow, corroborating `BU-P5-055`'s trigger from a second, independent source document.

## Stages

### `00-check-queue-and-plan`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-057` | Before planning from scratch, dispatch checks whether the task already exists in td; if the user's request maps to an open td task, dispatch is invoked with that task id, and the brief, branch name, and full task context are pulled from td automatically, including instructions for the task lifecycle commands the worker must run. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 27-36) |
| `BU-P5-058` | Before dispatching, dispatch states the plan explicitly -- which repos, what each does, dependency order, branch, and backend -- and requires that the plan be confirmed as accurate before proceeding. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 38-56) |
| `BU-P5-059` | dispatch can be invoked from an existing td task via the dispatch command with an explicit task-id argument, which auto-detects the owning repo and derives the brief and branch from the task, optionally overriding the repo set explicitly. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 58-68) |
| `BU-P5-060` | dispatch can also be invoked from a free-form brief with an explicit repository list, branch name, and dependency string, when no existing td task covers the work. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 70-77) |
| `BU-P6-123` | Dispatch is a bounded, independently invocable procedure: given a project, a brief (or a tracked-work task reference), and a set of target repos, it produces one durable task with an isolated worktree, a rendered mission brief, and a spawned interactive worker per repo — with every side effect (tracked-work creation, worktree acquisition, worker-process launch) validated and gated before the next repo's dispatch begins. | `reference/sergeant-upstream/bin/sgt-dispatch` (L1-5) |

### `05-classify-risk`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-048` | An objective whose text matches a fixed set of safety-sensitive or stateful keywords (auth, security, secrets, payments, databases, migrations, production, destructive, persistent state, state transitions) cannot proceed on the standard-isolated intent path and must instead be given an explicit --intent-file. | `reference/sergeant-upstream/bin/_sgt-intent.sh` (L215-217) |
| `BU-P7-016` | The dispatch skill must document a `standard-isolated` execution path and name specific trigger keywords (auth, OAuth, security, secret, credential, payment, database, migration, stateful, production, destructive) that route work away from it, and must warn against mutation happening before validation, and must bound remediation to at most two cycles before escalating. | `reference/sergeant-upstream/tests/instruction-policy-test.sh` (lines 78-81) |
| `BU-P8-069` | --intent-file is mandatory whenever the objective names auth/OAuth, security, secrets or credentials, payments, databases or migrations, stateful/production work, destructive work, persistent state, or state transitions; the intent file must contain the eight required sections, and malformed, missing, path-traversing, symlinked, or oversized input fails before any dispatch mutation, while every other objective uses the lighter standard-isolated path. | `reference/sergeant-upstream/docs/using-sergeant.md` (L112-117) |

### `10-preflight-capabilities` (folded into `15-check-admission`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-057` | The dispatch invocation's agent selector (flag or environment variable) may select opencode, oc, goose, claude, or an equivalent path whose basename is one of those names; dispatch uses only persistent interactive sessions and rejects every other agent and all non-interactive launch modes before creating worker state. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L186) |
| `BU-P1-058` | The dispatch invocation's model selector (flag or environment variable) pins the harness model as provider/model[:variant]; the agent selector and model selector are orthogonal; precedence is the explicit flag, then the environment variable, then the harness's ambient default, and an unpinned dispatch is recorded as unpinned rather than left blank. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L187, model precedence) |
| `BU-P1-060` | A model/variant tuple the selected harness cannot honor fails before any intent file, td task, worktree, or fleet state is created, and a worker handed one fails terminally instead of inheriting the ambient default. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L187, fail-before-side-effect) |
| `BU-P1-093` | A resumed or recovered worker reads the same fleet record and inherits the same pin; a worker handed a tuple its harness cannot honor fails terminally rather than falling back to the ambient default. | `reference/sergeant-upstream/README.md` (README.md L227-229) |
| `BU-P1-094` | A model/variant pin fails closed in two distinct situations with a diagnostic that says which: no known transport (the harness is measured and exposes no way to pin that axis) versus unmeasured (the harness is not installed here, so its launch surface has not been observed) — the latter is not a claim the harness cannot do it. | `reference/sergeant-upstream/README.md` (README.md L205-210, fail-closed distinction) |
| `BU-P6-107` | A worker never launches with a harness it cannot honor: the capability gate (is the harness accepted), a readiness probe, and the launch invocation are all validated up front, before any fleet state directory is even created, so an invalid harness is rejected before durable state exists to clean up. | `reference/sergeant-upstream/bin/sgt-interactive-worker` (L33-40) |
| `BU-P6-124` | Which pinned model tuple a dispatched worker will run is resolved with a fixed, explicit-only precedence — a per-invocation flag beats an environment variable, which beats no pin at all — with no project-level or per-repo default in the precedence chain by deliberate decision, not by omission, and the resolution and its shape are both validated before any intent file, task, or worktree exists. | `reference/sergeant-upstream/bin/sgt-dispatch` (L180-190) |
| `BU-P7-002` | GitHub CLI identity for dispatching a worker resolves in a fixed priority order: repo-level identity overrides project-level identity, which overrides the global default identity, which falls back to no identity switch. | `reference/sergeant-upstream/schema/project.yaml.example` (lines 13-15) |
| `BU-P7-072` | A failed `gh auth switch` during dispatch identity resolution must set the fleet task's status to failed with a recorded diagnostic and abort the dispatch, rather than silently proceeding to dispatch under the wrong (or no) identity. | `reference/sergeant-upstream/tests/sgt-dispatch-identity-test.sh` (lines 1-9) |
| `BU-P7-073` | Dispatch must pin an explicit provider/model/variant tuple for a dispatched worker and record it as durable, non-secret launch evidence in fleet state; every validation of that tuple must run and reject BEFORE any mutation, so a rejected dispatch leaves no new fleet task directory and no session log at all. | `reference/sergeant-upstream/tests/sgt-dispatch-model-tuple-test.sh` (lines 1-11) |
| `BU-P7-078` | Dispatch must be able to bind a coordinator identity without itself already running inside an existing session, while refusing every forged, stale, or unreachable coordinator identity, and every such rejection must happen before any fleet task directory is created. | `reference/sergeant-upstream/tests/sgt-dispatch-coordinator-pane-test.sh` (lines 1-11) |

### `15-check-admission`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-128` | A tracked-work task per targeted repo is created before dispatch commits to any worker launch when no explicit task reference was supplied, but the admission (drain) lock is held only through that first side effect and released immediately afterward, so dispatch does not hold a fleet-wide lock across the much longer per-repo worktree/launch sequence. | `reference/sergeant-upstream/bin/sgt-dispatch` (L473-486, L524-529) |

### `20-prepare-intent`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-059` | Dispatching creates or reuses td work, creates isolated worktrees, writes worker briefs, and records fleet state; it writes the same .sergeant-intent.md revision into fleet state and every selected worktree, and that one artifact is treated as canonical for implementation decisions, reviews, PR text, successor/recovery work, and final validation. | `reference/sergeant-upstream/docs/using-sergeant.md` (L54-58 (Dispatch mode)) |

### `30-create-tracked-work` (folded into `80-monitor`, N1 adjudication A4 — sequenced second per A3, after reconciliation)

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-088` | Dispatching from a free-form brief creates exactly one td task per target repository before spawning any worker; if td is unavailable, task creation fails, generated metadata cannot be injected, or any selected repo fails to get a task, the whole dispatch aborts before spawning any worker and rolls back the generated tasks. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 196-198) |
| `BU-P6-036` | Creating tracked-work items across several repos for one cross-repo brief is all-or-nothing: every target repo is validated up front (cloned, task tracker initialized) before any task is created, and if creating a task in any repo fails after some were already created, every already-created task is deleted to roll back. | `reference/sergeant-upstream/bin/sgt-td-create` (L6-8, L192-193) |

### `40-reconcile-before-launch` (folded into `80-monitor`, N1 adjudication A4 — sequenced first per A3/BH-01: reconciliation before tracked-work creation)

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-070` | Bulk fleet reconciliation syncs worktree status into fleet state, stops only identity-verified done or failed worker processes, and marks an interrupted dispatched record failed only once it has had neither a worktree nor an owned live process for a default 300-second grace period (configurable), while it always preserves needs_input, blocked, and orphaned records, and dispatch always runs this reconciliation automatically before creating new work. | `reference/sergeant-upstream/docs/using-sergeant.md` (L137-155 (Monitor work)) |

### `50-acquire-surface` (folded into `80-monitor`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-061` | Dispatching a task first generates a durable task identity, then creates an isolated git worktree per target repository at a deterministic sibling path. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 79-81) |
| `BU-P6-019` | Once repos have a treehouse pool initialized, dispatch automatically prefers a pre-warmed treehouse lease over a plain git worktree for those repos, without the operator having to select a worktree strategy per dispatch. | `reference/sergeant-upstream/bin/sgt-treehouse-init` (L11, L78-79) |
| `BU-P6-125` | Dispatch refuses to re-dispatch onto a branch that already carries committed work unreachable from any remote, unless the operator explicitly opts in with an adopt-branch flag, because a prior interrupted dispatch may have done real, unpreserved work that a fresh dispatch would silently discard or duplicate. | `reference/sergeant-upstream/bin/sgt-dispatch` (L776-793) |
| `BU-P7-069` | The `--adopt-branch` dispatch option is an explicit operator acknowledgement that a named branch already carries committed work and should be resumed as-is; it is non-destructive (it checks the branch out at a new worktree path, preserving the branch tip and every commit), and it exists so the unpushed-work guard cannot make preserved work permanently unresumable in a repository whose upstream denies push access. | `reference/sergeant-upstream/tests/sgt-dispatch-adopt-branch-test.sh` (lines 1-12) |
| `BU-P7-070` | The unpushed-work guard's refusal message must never instruct the operator to delete the branch (the data loss it exists to prevent) and must instead name `--adopt-branch` as the supported non-destructive reconcile path. | `reference/sergeant-upstream/tests/sgt-dispatch-adopt-branch-test.sh` (lines 78-84) |

### `60-render-brief` (folded into `80-monitor`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-062` | Each worker's starting context must durably carry its mission, the merged/resolved agent instructions, dependency notes, and delivery requirements, before the worker begins. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 82) |
| `BU-P5-087` | If a canonical skill referenced by the routing table cannot be loaded, the generated brief's own embedded rules for that phase remain mandatory regardless. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 190) |
| `BU-P7-112` | A worker brief's instruction merge order is defaults, then group, then repo, and an explicit user override embedded in the dispatched brief (e.g. 'run no-mistakes for this worker before completion') must appear verbatim in the rendered brief rather than being silently dropped by the default no-mistakes-ownership instruction it overrides. | `reference/sergeant-upstream/tests/sgt-dispatch-brief-test.sh` (lines 382, 501-511) |
| `BU-P7-074` | sgt-dispatch must resolve an OpenCode (`oc`) target session for routing coordinator notifications by consulting `td` for an existing routing task before creating one, so coordinator notification routing reuses existing tracked infrastructure rather than duplicating it per dispatch. | `reference/sergeant-upstream/tests/sgt-dispatch-oc-target-test.sh` (lines 34-40) |
| `BU-P7-075` | sgt-dispatch's `td` integration must distinguish an existing tracked task from one that needs to be newly created for a cross-repo brief, and this contract must be exercised against a full copy of every sourced helper (not a hand-picked subset), because a missing helper makes the copy fail at its own source line instead of exercising the behavior under test. | `reference/sergeant-upstream/tests/sgt-dispatch-td-test.sh` (lines 19-24) |
| `BU-P5-074` | The --deps ordering string only expresses that one repository must finish before dependents can merge; enforcing it is left entirely to the dispatched workers reading it out of their own brief. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 155-157) |
| `BU-P5-075` | Every dispatched worker pins a fixed point -- normally the merge-base with current origin/main -- and records the base SHA, commit list, and diff scope before implementing anything. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 166) |
| `BU-P5-076` | Every dispatched worker triages the full originating td issue/spec/comments and linked material, prior or redundant work, category, and readiness, and explicitly records when no originating spec exists. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 167) |
| `BU-P5-078` | A worker establishes public behavioral seams from td/spec evidence before writing tests; if a consequential seam is undecided, it escalates needs_input rather than guessing. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 174) |
| `BU-P5-150` | A worker implements one vertical slice at a time: a focused failing test, the minimum passing implementation, then refactor; tautological tests, internal mocking, horizontal test/implementation phases, and speculative refactoring are all rejected. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 175) |
| `BU-P5-079` | A worker in needs_input or blocked writes an escalation message and notifies the coordinator; on response, it consumes/removes the response, clears the message, logs the decision to td, restores in_progress, and continues -- the durable requirement is that answering a blocked worker always durably restores forward progress, regardless of the underlying file-based transport. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 176) |
| `BU-P5-080` | No-mistakes is run only at an explicit final shipping boundary, never for routine worker completion, prototypes, investigations, documentation drafts, intermediate commits, or remediation loops, unless the user explicitly overrides that default; safety-sensitive work follows a stricter path. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 178) |
| `BU-P5-081` | Every no-mistakes finding is routed through the finding-routing command into a separate, deduplicated, owning-repo td task; correctness/security/data-integrity/test/ask-user findings are P1 and gated, warning debt is P2, informational debt is P3, and cosmetic/evidence noise is ignored -- and findings are never remediated inside the validation run itself. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 179) |
| `BU-P5-082` | Independent review runs as separate parallel subagents for the axes named by a shared axis-vocabulary source (standards, spec, readiness, plus a conditional accessibility axis for UI-facing work identified by role/group/description language), each described by guidance reproduced verbatim in the generated brief; the spec axis is explicitly skipped when no spec exists. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 180-181) |
| `BU-P5-083` | When a finding fails to route after parsing, the router retains a sanitized findings artifact under a fixed path and names the exact retry command; the artifact is retried from, never re-generated by re-running the reviewer, and a retained artifact that has not been retried is never deleted. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 182) |
| `BU-P5-084` | A stored finding-card revision the router cannot prove it authored is preserved below a superseded-revision separator and the card gains a needs-reconciliation label; only the worker owning the finding may merge the two accounts and remove that label -- the router itself never clears it. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 183) |
| `BU-P5-085` | Findings sharing the same originating run, head, owning module, and root cause share one serialized remediation worker/branch; before merging that group, native tests and independent re-reviews verifying mutation-before-validation, partial-publication/rollback, and identity/provenance are rerun; after two remediation cycles, fix dispatch stops and an architectural/root-cause review plus a human decision is required. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 184) |
| `BU-P5-151` | A worker remediates every blocking repository-native test and independent-review finding and reruns the affected tests and all required review axes until each reports zero blocking findings; no-mistakes findings are remediated through a separate td dispatch, not inline. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 185) |
| `BU-P5-152` | A worker commits its changes, opens a pull request, waits for required CI to pass, resolves every non-outdated review thread, and satisfies the plan's declared dependency order before considering its work delivered. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 186) |
| `BU-P5-153` | For tracked work, a worker logs td decisions and hands off, then runs `td review` only once implementation and review evidence are both ready. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 187) |
| `BU-P5-086` | A worker writes its terminal result record and sets its status to done only after every completion gate has passed; a failed status with an exact reason is reserved specifically for an unrecoverable terminal failure. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 188) |
| `BU-P5-089` | Recovering a stuck, stale, or orphaned dispatched worker is done only through the response-delivery command or an equivalent explicit action; a worker is never marked done manually, and a retry writes both the result and the done status only after every completion gate passes. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 221-229) |

### `70-launch-and-record` (folded into `80-monitor`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-110` | Launch evidence distinguishes intent from proof by a two-state field: it is written as 'intended' before the harness is even invoked, and only promoted to 'confirmed' once the harness is observably ready, so a harness that rejects the pin and exits immediately never leaves evidence claiming the model actually ran. | `reference/sergeant-upstream/bin/sgt-interactive-worker` (L86-90) |
| `BU-P6-126` | Every repo's identity-switching, worktree acquisition, brief writing, and worker launch is validated in order such that any failure records an orphaned status with a specific diagnostic and a handoff before the dispatch loop aborts, rather than leaving a repo's fleet state silently incomplete. | `reference/sergeant-upstream/bin/sgt-dispatch` (L924-928, L939-943, L949-953, L959-963) |
| `BU-P7-111` | The interactive worker must pass its pinned provider/model tuple to the harness process it launches, and the durable launch_record it writes must be verified against the harness's own independently observed argv/environment — never trusted from the worker's internal bookkeeping alone — so recorded launch evidence is provably what actually ran. | `reference/sergeant-upstream/tests/sgt-worker-model-tuple-test.sh` (lines 6-9) |

### `80-monitor`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-066` | needs_input and blocked are distinct nonterminal states for a dispatched worker; a worker waiting on CI, review threads, or dependencies stays in_progress unless it actually needs to escalate. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 92) |
| `BU-P5-067` | When a worker escalates, the coordinator must read its full context, evidence, exact question/blocker, recommendation, and options; obtain an explicit human decision without inferring consequential intent; and deliver that decision to the specific task/repo pair via the response-delivery command, which durably writes the response to fleet state before notifying the worker. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 94-98) |
| `BU-P5-068` | After a response is delivered, the worker must consume/remove the response, clear its escalation message, log the decision to td, and return to in_progress before continuing. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 99) |
| `BU-P6-113` | A worker's exit boundary settles the accepted notification action lease for every possible terminal status alike — done, failed, drained, needs_input, blocked, waiting, and orphaned — because every one of those exit paths is an equally valid place for a lease to be silently left outstanding forever if it is not settled at that single, unified boundary. | `reference/sergeant-upstream/bin/sgt-interactive-worker` (L483-489) |
| `BU-P6-115` | A worker's exit is orphaned unless it produced a genuinely terminal status with substantiating evidence: a done status requires a non-empty result or the worker is reclassified orphaned; any other unrecognized status falls back to orphaned by default, so an unclassified exit is never mistaken for success. | `reference/sergeant-upstream/bin/sgt-interactive-worker` (L490-495, L509-510) |
| `BU-P6-116` | For the Claude harness specifically, whether a pinned model was actually honored — rather than silently substituted by the provider — is confirmed only after the run completes, by scanning the session transcript for a known substitution-warning phrase; this check never blocks or changes the mission's outcome, it only records a diagnostic that survives the otherwise-unconditional cleanup of that diagnostic on success. | `reference/sergeant-upstream/bin/sgt-interactive-worker` (L1109-1117) |

### `90-reconcile-fleet`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-070` | Reconciliation after all workers finish requires verifying, per repository: pinned-base scope, focused/full validation, separate standards/spec review artifacts, an accessibility review artifact for UI-facing work, zero blocking findings, required CI, and resolved non-outdated review threads; and checking dependency order so infra merges before API before app when there is a runtime dependency. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 108-114) |
| `BU-P5-071` | A fleet is never reconciled or cleaned up merely because every worker has opened a PR; all completion gates must be met. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 115) |
| `BU-P1-006` | In dispatch mode, monitor progress and reconcile merge order, PRs, and cross-repository implications. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L20, dispatch-mode step 3) |

## Adjudication A4

Applying the reference-corpus's N1 round-1 adjudication (`reference-corpus/adjudication-round1.md` A4, finding N1-BH-02) to every §6.5-extracted stage in this package:

| Stage | Additional note present? | §6.3 reimplementation test | Decision |
|---|---|---|---|
| `10-preflight-capabilities` | **yes** — "'Nothing was created if this failed' is the checkpoint — not how the probing is implemented." | The note's own framing is exactly what the test treats as a helper: it concedes the checkpoint is unchanged by swapping the probe implementations. | **Demoted** — folded into `15-check-admission` as a preceding helper invocation. |
| `30-create-tracked-work` | none | Swapping the td-task-creation implementation leaves the checkpoint — all-or-nothing across every target repo — unchanged. | **Demoted** — folded into `80-monitor`, sequenced second (A3). |
| `40-reconcile-before-launch` | none | Swapping the reconciliation implementation leaves the checkpoint — bulk sync runs before new work — unchanged. | **Demoted** — folded into `80-monitor`, sequenced first (A3/BH-01). |
| `50-acquire-surface` | none | Swapping the worktree/treehouse-acquisition implementation leaves the checkpoint — an isolated surface per repo, unpushed work never silently discarded — unchanged. | **Demoted** — folded into `80-monitor`. |
| `60-render-brief` | **yes**, but scope-clarifying only (BU-P5-075..089 plus BU-P5-150/151/152/153 authored-not-executed distinction, `--deps` recording-vs-enforcement split) — it does not argue the checkpoint resists implementation swap. | Swapping the rendering/templating implementation leaves the checkpoint — a complete brief durably exists before the worker starts — unchanged. | **Demoted** — folded into `80-monitor`. |
| `70-launch-and-record` | none | Swapping the launch/evidence-recording implementation leaves the checkpoint — intended-then-confirmed evidence, orphaned-on-failure — unchanged. | **Demoted** — folded into `80-monitor`. |
| `15-check-admission` | **yes** — "Blocks on `drain-fleet`'s admission-block state," with a `## Delegation` naming that this stage's outcome is produced by running an entire other workflow to completion. | A real cross-workflow dependency, not a local implementation detail — running **drain-fleet** to completion is not something a script swap eliminates. | **Kept.** |
| `00-check-queue-and-plan`, `05-classify-risk`, `20-prepare-intent`, `80-monitor`, `90-reconcile-fleet` | n/a (extracted as actor-stage, §6.4, already judgment-bearing) | n/a | **Kept** as extracted. |

Stage count: 12 extracted → 6 surviving. No behavior unit was deleted; all units cited under the six folded stages above remain cited, now under `15-check-admission` or `80-monitor`'s own "Helper invocations" sections (see those stages' `CONTEXT.md`).

## Notes

**Demoted/merged candidates:** **Obsolete-mechanism stress test (§8.2).** The `dispatch` skill's tmux/sentinel/worker-Bash machinery (pane identity, pane-as-notification-channel, pane-as-liveness-signal, the nudge loop) carried none of the stage boundaries above — see `reference-corpus/synthesis.md` §4 clusters M1–M4 for the mechanism-vs-policy separation. What survived: preflight-before-side-effect, all-or-nothing tracked-work creation, one canonical intent revision, durable brief delivery, intended→confirmed launch evidence, per-repo failure recorded rather than silent. Worker-contract content this workflow *authors* but does not itself execute (BU-P5-075/076/078/079/080/081/082/083/084/085/086/089) is the input to `worker-mission` and `route-review-findings`.

**Synthesis notes:** Reviewers flagged this as the corpus's largest single cluster (63 units, 12 stages) — see `reference-corpus/synthesis.md` §8 note 1: either it is genuinely one procedure with twelve checkpoints, or it should split at `70-launch-and-record` into a plan-and-validate workflow and a launch-fleet workflow. Recorded as an open question for the classification ledger, not resolved here.


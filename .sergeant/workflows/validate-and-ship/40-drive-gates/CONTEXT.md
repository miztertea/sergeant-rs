# 40-drive-gates: drive gates

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-start-run/output/README.md | L4 | upstream artifact produced by `30-start-run` |

## Purpose

Every gate resolved by exactly one response; ask-user findings relayed verbatim and never resolved autonomously; the actor never edits the pipeline-owned worktree, aborts, or reruns to escape a gate.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

Every gate resolved by exactly one response; ask-user findings relayed verbatim and never resolved autonomously; the actor never edits the pipeline-owned worktree, aborts, or reruns to escape a gate.

## Behavior contract

- **`axi run` and every `axi respond` block synchronously and each step can take several minutes, so a single call may not return for a while; that is normal, requires a long timeout, and must not be interrupted or re-issued because it seems slow — progress can be checked separately via `axi status` without disturbing the run.**
  (trigger: a pipeline call is in flight and appears slow; outcome: the actor waits out the call (backgrounding it if needed) rather than cancelling or re-issuing it, using `axi status` to observe progress)
  — `BU-P2-075`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Blocking/patience rule, lines 102-106)
- **The `awaiting_agent: parked <duration>` field on status output means the run is parked at a gate waiting for `axi respond`; the field is observability only — it does not change gate resolution, auto-resume the run, or make `--yes` the default.**
  (trigger: reading `axi status` output on a parked run; outcome: the field is correctly understood as informational, not actionable on its own)
  — `BU-P2-076`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (awaiting_agent field, lines 111-114)
- **While a step is `running` or `fixing`, `axi status` may include an `active_steps` table with `active_for`, `last_activity`, a native `agent_pid` when a subprocess agent is running, and the current round (e.g. `round 1`, `auto-fix 1/3`, `fix 2`); a `last_activity` prefixed `quiet` means no step log or agent-lifecycle activity has arrived for longer than `step_quiet_warning`.**
  (trigger: monitoring an active step via `axi status`; outcome: the actor can read detailed liveness/progress information about the currently running step)
  — `BU-P2-077`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (active_steps field, lines 115-119)
- **A `quiet` last_activity is a liveness clue only, not permission to cancel, rerun, or edit the worktree.**
  (trigger: observing a quiet, possibly-stalled step; outcome: the actor treats apparent staleness as information, not authorization to intervene)
  — `BU-P2-078`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (quiet liveness rule, lines 119-121)
- **A `gate:` object means the pipeline is waiting on the actor; its findings table has `id`, `severity`, `file`, `description`, and an `action` classifying it as `auto-fix` (mechanical/low-risk, actor may authorize on their own judgment), `no-op` (informational, nothing to do), or `ask-user` (challenges the user's deliberate intent or touches product behavior — a decision only the user can make).**
  (trigger: the pipeline returns a `gate:` object; outcome: each finding is classified into one of three action categories that determine who may decide it)
  — `BU-P2-079`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Gate findings handling, lines 122-131)
- **Review auto-fix is disabled by default (`auto_fix.review: 0`; a repo- or global-level `auto_fix.review > 0` override re-enables it), so blocking and ask-user review findings park for actor decision rather than being silently self-fixed; other steps such as test and lint may still auto-fix within the pipeline and re-run before ever gating.**
  (trigger: the review step produces findings; outcome: review findings default to parking for a decision rather than silent auto-fixing, unlike some other pipeline steps)
  — `BU-P2-080`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Review auto-fix default, lines 133-137)
- **At a gate the actor chooses exactly one response: `--action approve` (accept as-is and continue), `--action fix --findings <ids> [--instructions "..."]` (have the pipeline fix specific findings and continue), or `--action skip` (skip this step).**
  (trigger: deciding how to respond to a gate; outcome: one of three defined actions is chosen and issued via `axi respond`)
  — `BU-P2-081`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Three response actions, lines 139-149)
- **While a run is active the actor must never fix findings by editing the code directly — the pipeline owns both findings and fixes, and the actor's job at a gate is to decide and respond via `--action fix`, which has the pipeline apply the fix and re-review; for the same reason the actor must not `abort` or `rerun` mid-run to go fix something themselves, even a real bug in their own code, since that discards the pipeline's in-flight work and forces full re-validation.**
  (trigger: a gate finding looks fixable to the actor; outcome: the actor routes every fix through the pipeline's own fix mechanism instead of editing the worktree or restarting the run)
  — `BU-P2-082`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Actor must not self-fix, lines 150-158)
- **Each `axi respond` blocks until the next `gate:`, `checks-passed` decision point, or final outcome.**
  (trigger: the actor has responded to a gate; outcome: the call does not return until the next decision point is reached)
  — `BU-P2-083`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (respond blocking, lines 160-160)
- **`--add-finding '<json>'` (used with `--action fix`) folds a finding the actor spotted themselves — one the pipeline did not surface — into the current fix round, as a JSON finding object.**
  (trigger: the actor notices a problem the pipeline's gate did not surface; outcome: the actor-observed problem is included in the same fix round as pipeline-surfaced findings)
  — `BU-P2-084`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (add-finding flag, lines 163-166)
- **`--step <name>` responds to a specific step instead of the one currently awaiting approval, and is rarely needed since omitting it answers the active gate.**
  (trigger: responding to a gate; outcome: responses target the active gate by default, with an explicit override available)
  — `BU-P2-085`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (step flag, lines 167-168)
- **A gate whose findings are all `auto-fix` or `no-op` may be driven on the actor's own judgment, but any `ask-user` finding is a decision that belongs to the user because it challenges their deliberate intent or changes product behavior; the actor must not approve, fix, or skip it on their own — it must stop and bring it to the user first.**
  (trigger: a gate contains at least one ask-user finding; outcome: the actor defers that specific decision to the user instead of resolving it independently)
  — `BU-P2-098`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Escalate ask-user findings, lines 223-231)
- **Each `ask-user` finding is relayed to the user verbatim (id, file, full description, not paraphrased or pre-judged); the actor asks how they want to proceed and translates their decision into the matching `respond` call (`--action fix` with `--instructions`, `--action approve`, or `--action skip`).**
  (trigger: an ask-user finding has been identified; outcome: the user's own decision, not the actor's interpretation of it, determines the eventual respond call)
  — `BU-P2-099`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Escalation procedure, lines 233-238)
- **`--yes` is the user's standing consent to drive every gate unattended: it treats every actionable finding (auto-fix and ask-user alike) as consent to fix, selects every current finding for one fix round, accepts the resulting fix review, and approves gates with only no-op findings; it should only be used when the user has asked to drive the whole run without checking back, and it is the sole exception to the ask-user escalation rule (NM43).**
  (trigger: the user has given explicit standing consent to run unattended; outcome: ask-user findings are resolved automatically instead of stopping to ask, but only under this explicit consent flag)
  — `BU-P2-100`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (--yes standing consent, lines 240-249)
- **axi run and axi respond block while work is active — a quiet step is not a stall; check progress with axi status without issuing duplicate run commands.**
  (trigger: a run appears quiet; outcome: progress is checked without ever issuing a second, duplicate run command)
  — `BU-P1-074`, `reference/sergeant-upstream/README.md` (README.md L281)
- **At each gate, inspect every finding: auto-fix findings are authorized selectively after review; ask-user findings are relayed to the user and never approved, fixed, or skipped autonomously; no-op findings are informational and the gate is simply approved.**
  (trigger: a gate presents findings; outcome: each finding is handled according to its category, with human authority preserved for ask-user findings)
  — `BU-P1-075`, `reference/sergeant-upstream/README.md` (README.md L285-287, gate dispositions)
- **While a run is active: do not edit the pipeline-owned worktree, do not abort or rerun to escape a gate, and preserve all pipeline-created commits; abort only when intentionally discarding the entire run.**
  (trigger: a run is active; outcome: the pipeline-owned worktree and commit history stay intact for the run's duration)
  — `BU-P1-076`, `reference/sergeant-upstream/README.md` (README.md L289)
- **Do not use --yes; use --skip=<steps> only for stages already proven irrelevant — skipping is not a substitute for checks that have not been performed.**
  (trigger: starting or configuring a no-mistakes run; outcome: gate steps are never bulk-approved or skipped without proven irrelevance)
  — `BU-P1-072`, `reference/sergeant-upstream/README.md` (README.md L275)
- **In a worked gate example, the actor decides each row by its `action` column — auto-fix findings can be authorized directly, ask-user findings must be escalated — while a terminal state instead shows `outcome: <checks-passed|passed|failed|cancelled>` with no findings table; field names and exact columns can vary by step and version, so the actor must read the actual `findings` header rather than assume a fixed layout.**
  (trigger: reading a concrete gate or outcome response; outcome: the actor parses the response structurally (by header) rather than assuming a hardcoded schema, and applies the auto-fix/ask-user distinction per row)
  — `BU-P2-103`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Gate example / header-stability warning, lines 290-296)

## Helper: route findings (folded from demoted `60-route-findings`, N1 adjudication A4)

`60-route-findings` was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 it is demoted and its behavior folded here as the concluding helper invocation of this checkpoint — once every gate this run will hit has been driven to a terminal outcome, its actionable findings are routed to `td` as one deterministic operation subordinate to this stage's own judgment-bearing outcome:

- **A no-mistakes finding's severity and disposition together determine, deterministically, both its td priority and whether it is even allowed to be deferred: correctness/security/data-integrity/test findings must gate or ask the user and can never be routed to td or ignored, while cosmetic/evidence findings create no td card at all.**
  (trigger: a no-mistakes review finding is classified with a kind and a requested disposition; outcome: a finding either blocks/escalates, is silently dropped as non-actionable, or is routed to td for later work — and which of those three happens is a deterministic function of kind, never left to caller discretion)
  — `BU-P6-023`, `reference/sergeant-upstream/bin/sgt-no-mistakes-finding` (L67-77)
- **An existing td task matched by a finding's deduplication marker is reopened if closed and has its deferral cleared before being updated, so a finding that recurs after being closed or snoozed is never silently left in a stale closed/deferred state.**
  (trigger: a finding matches an existing but closed or deferred td task; outcome: a recurring finding always surfaces as live tracked work again)
  — `BU-P6-026`, `reference/sergeant-upstream/bin/sgt-no-mistakes-finding` (L176-185)
- **The no-mistakes run is validation-only and must not fix findings; actionable findings are routed into separate, deduplicated owning-repository td tasks with sgt-no-mistakes-finding.**
  (trigger: a no-mistakes run produces findings; outcome: findings become tracked, deduplicated repository work rather than being fixed in-run)
  — `BU-P1-080`, `reference/sergeant-upstream/README.md` (README.md L304)
- **Applying a disposition to a no-mistakes finding must route it through the same `td` invocation contract (run-id, head-sha, finding-id, severity, kind, file, line, description, intent) regardless of disposition, and the routing behavior itself (e.g. `--disposition td`) is directly observable in the exact `td` invocation logged.**
  (trigger: a no-mistakes finding needs a disposition applied (e.g. routed to td as debt); outcome: a finding's routing is deterministic and inspectable — the exact fields passed to td are asserted, not merely 'some td call happened')
  — `BU-P7-065`, `reference/sergeant-upstream/tests/sgt-no-mistakes-finding-test.sh` (lines 50-70, 88-90)

This stage's routing step above is performed directly by this stage via the `sgt-no-mistakes-finding` helper — **corrected 2026-08-16, ICM-R2 pilot review (BU-VAS-10):** the prior text here claimed delegation to a package named `route-review-findings`; no such package exists (retriaged to unbuilt CLI-verb candidates, `docs/icm/retriage-2026-08-11.md` line 52). There is no nested-workflow invocation here, nor a `@@name` shared-context composition — just this stage's own helper.

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Authorizing `auto-fix` findings on its own judgment (`BU-P2-080`).
- Approving a gate whose findings are all `auto-fix` or `no-op` (`BU-P2-098`).
- Classifying a no-mistakes finding's severity/kind for `td` routing eligibility per the deterministic helper's own contract (`BU-P6-023`, folded helper below) — mechanical, not this stage's own judgment call, but this stage invokes it.

### J1 — local choices allowed
- None beyond ordinary tool mechanics — every gate response is either a J2 auto-fix/no-op authorization or a J0 ask-user escalation; there is no local, non-contractual choice in between at this stage.

### J0 — must become `needs_input`
- **Every `ask-user` gate finding, without exception** — relayed verbatim (id, file, full description, not paraphrased or pre-judged) and never resolved autonomously (`BU-P2-098`, `BU-P2-099`). This is the canonical worked precedent the whole Bounded-Judgment Ladder generalizes from (`reference/proposal-icm-r-procedure-authority.md` §3.4).
- **The push/pr/ci authority gap (BU-VAS-15, full record in the workflow-level `CONTEXT.md`'s `## Authority envelope`).** This stage drives every gate — including any `push`/`pr`/`ci` steps a launched run's pipeline includes — to a terminal outcome, and nothing in this stage's own contract, or anywhere else in this package, names an authority boundary for autonomous publication. Measured live twice (`docs/gauntlet/runs/path-to-mac-2026-08-15/retrospective.md` §3.1; `docs/gauntlet/runs/macbook-arrival-2026-08-15/retrospective.md` §3, "#123 materialized, not just predicted"). **This clause is a citable placeholder, not a resolution** — the underlying policy question (should this stage, when driving a dispatched run, ever autonomously push/open a PR/trigger CI, and under what recorded consent) is a live, separate owner decision this stage does not make on its own initiative. Until ruled on, treat any push/PR/CI step this stage's driven run reaches as `needs_input`, per this ladder's own J0 procedure — do not silently apply `scripts/gate.sh`'s `--skip push,pr,ci` behavior here, since that script is not part of this stage's own contract and this stage has no way to know its flags.
- `--yes` unattended consent for a Sergeant-coordinated run — the absolute-never reading, not the vendored gate skill's documented standing-consent exception (Conflict X3 below).

### Completion boundary
This stage may complete only when every gate reaches exactly one response — auto-fix, no-op, or ask-user relayed and resolved by the user — and every actionable finding is routed to a deduplicated owning-repo task.

### Decision evidence
Each gate's own findings table (id, severity, action, disposition) is this stage's decision record; a `J0` stop is recorded in the turn's own `needs_input` question per `@@bounded-judgment`'s canonical shape.

## Additional note

This is the judgment stage of the whole corpus — see `reference-corpus/synthesis.md` §1's stage table annotation. Conflict X3 (synthesis.md §6): whether `--yes` unattended consent may ever be used is contested between an absolute-never reading (BU-P1-072, BU-P8-083) and a documented standing-consent exception in the vendored gate skill (BU-P2-100); this draft follows the absolute-never reading for Sergeant-coordinated runs and preserves the exception as evidence, not as an instruction to follow.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

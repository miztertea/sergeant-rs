# 90-completion-summary: completion summary

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../70-optional-capabilities/output/README.md | L4 | upstream artifact produced by `70-optional-capabilities` |

## Purpose

Every checklist item resolved as `[ok]` / `[skipped]` / `[issue: id]`.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

Every checklist item resolved as `[ok]` / `[skipped]` / `[issue: id]`.

## Behavior contract

- **sergeant-setup maintains a visible numbered checklist: before each step it verifies whether the step is already complete and skips it without prompting if so; after each step it writes an [ok] or [skipped] status line; when a phase fails, the run stops with actionable output identifying the last completed phase.**
  (trigger: the workflow is running; outcome: the operator always sees exact progress and the exact failure point)
  — `BU-P5-007`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 45-49)
- **The Phase 10 completion summary lists every checklist item as exactly one of [ok], [skipped], or [issue: <td-id>].**
  (trigger: all prior phases have run or been skipped; outcome: the operator gets one consolidated, unambiguous status line per checklist item)
  — `BU-P5-036`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 279-292)
- **Re-running sergeant-setup after a successful setup must produce the same final state; no phase destroys existing working configuration merely to reach the same end state it already represents.**
  (trigger: sergeant-setup runs against an already-working installation; outcome: the workflow is a true no-op on an already-correct system)
  — `BU-P5-037`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 296-298)
- **Re-running sergeant-setup after a partial run restarts the checklist from Phase 1 but skips every phase that already passes verification; resumability works by re-checking each phase before acting, not by persisting run state between invocations.**
  (trigger: sergeant-setup is invoked again after a prior partial or complete run; outcome: the run converges to the same end state without redoing completed work or persisting extra state)
  — `BU-P5-008`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 49-52)
- **The setup skill's failure table must cover every required failure mode (missing/uninstallable prerequisite, declined consent, YAML parse error, unsupported capability, sync failure, partial setup on exit) with a stop condition, and must not indicate auto-continuing past a failed sync.**
  (trigger: setup encounters any of a fixed set of failure conditions; outcome: every named failure condition has a documented stop behavior, and partial setup state is never silently carried forward past a failed step)
  — `BU-P7-039`, `reference/sergeant-upstream/tests/sergeant-setup-test.sh` (lines 94-106)
- **The setup skill must cover a fixed ordered checklist of phases — detect prerequisites, install command links, write global config, run an interview, repair existing YAML, sync and verify, optionally initialize treehouse — and each mutating phase must be idempotent.**
  (trigger: a developer runs sergeant-setup for the first time or to repair an installation; outcome: the same skill invocation is safe to run repeatedly (bootstrap and repair are the same procedure), moving through a fixed, testable phase sequence)
  — `BU-P7-040`, `reference/sergeant-upstream/tests/sergeant-setup-test.sh` (lines 79-88)
- **Worker briefs already discover their required workflow skills from this repository's own vendored .agents/skills/ tree, so installing engineering skills for dispatched workers is not a separate manual step beyond having the repository itself; any additional locally-installed skills must still come from reviewed sources.**
  (trigger: an operator wonders whether skills need separate installation before dispatching workers; outcome: no separate skill-installation step is required for the vendored, worker-brief-required skill set)
  — `BU-P8-049`, `reference/sergeant-upstream/docs/getting-started.md` (L163-171)
- **The getting-started checklist's own definition of a completed installation is a fixed nine-item checklist: required commands resolve, the coordinator runs in a tmux pane, sgt-list shows the project exactly once, sgt-context resolves every owning repo and instruction layer, required repos are cloned, Marcus td is installed and initialized, GitHub CLI can access required repos, optional Treehouse/Graphify features pass their own verification, and required (plus any reviewed extra) skills are present.**
  (trigger: an operator wants to know whether installation is actually done; outcome: installation has one explicit, checkable, bounded completion condition rather than an implicit 'probably fine')
  — `BU-P8-051`, `reference/sergeant-upstream/docs/getting-started.md` (L192-202 (Completion checklist))

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P8-051` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

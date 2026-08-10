# 50-acquire-surface: acquire surface

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-reconcile-before-launch/output/README.md | L4 | upstream artifact produced by `40-reconcile-before-launch` |

## Purpose

An isolated work surface per repo at a deterministic location; a branch already carrying unpushed committed work is refused unless explicitly adopted.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

An isolated work surface per repo at a deterministic location; a branch already carrying unpushed committed work is refused unless explicitly adopted.

## Behavior contract

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

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

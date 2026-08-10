# 70-optional-capabilities: optional capabilities

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../60-task-tracking-init/output/README.md | L4 | upstream artifact produced by `60-task-tracking-init` |

## Purpose

Worktree pools and graph output initialized only where explicitly desired; declining never marks setup incomplete.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

Worktree pools and graph output initialized only where explicitly desired; declining never marks setup incomplete.

## Behavior contract

- **If treehouse is on PATH, sergeant-setup offers to initialize Treehouse worktree pools with an explicit y/N prompt; it runs only on confirmation, skips silently on decline or absence, and never marks overall setup incomplete because Treehouse was skipped.**
  (trigger: treehouse is present; outcome: Treehouse initialization is strictly optional and its absence never fails the run)
  — `BU-P5-034`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 255-263)
- **If graphify is on PATH and the project YAML declares graphify.output, sergeant-setup offers to run sgt-graphify with an explicit y/N prompt, skips silently on decline, and on a successful run requires both graph.json and GRAPH_REPORT.md to exist at the configured output path.**
  (trigger: graphify is present and configured; outcome: a completed Graphify run is verified by the presence of both its required output artifacts)
  — `BU-P5-035`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 267-275)
- **Initializing a treehouse worktree pool for a project's repos only acts on repos that are already cloned and skips (never fails) repos lacking a local clone, and it treats an already-present treehouse.toml as already-initialized rather than re-initializing.**
  (trigger: operator runs sgt-treehouse-init for a project, optionally filtered by group; outcome: every eligible, not-yet-initialized, cloned repo gets a treehouse pool; nothing is skipped silently and nothing already initialized is redone)
  — `BU-P6-018`, `reference/sergeant-upstream/bin/sgt-treehouse-init` (L59-71)
- **Worktree pools via Treehouse are only initialized for repositories where they are explicitly desired, and any repository-owned treehouse.toml produced by that step is committed through normal review rather than treated as install-time throwaway state.**
  (trigger: an operator considers enabling leased worktree pools; outcome: worktree-pool setup is opt-in per repository and its config artifact goes through ordinary review, not an install-time bypass)
  — `BU-P8-047`, `reference/sergeant-upstream/docs/getting-started.md` (L144-151)
- **A project graph is only considered successfully published when both graph.json and GRAPH_REPORT.md exist at the configured output.**
  (trigger: sgt-graphify is run as part of optional setup; outcome: a partial or malformed graph output is never mistaken for a completed publish)
  — `BU-P8-048`, `reference/sergeant-upstream/docs/getting-started.md` (L153-161)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

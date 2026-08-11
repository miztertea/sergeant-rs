# 06-bind-and-verify-coordinator-pane

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a coordinator not inside tmux needs to bind a pane

**Outcome:** pane binding happens through exactly one of the two mutually exclusive paths, always verified live, without relaxing the persistent-interactive-worker requirement

**Statement (the operative rule):** The dispatch step or `--coordinator-pane <pane-id>` lets a non-tmux coordinator bind a coordinator pane; the two flags cannot be combined, the managed path never starts a tmux server, and every path verifies the pane against the live server before use, without weakening the persistent interactive worker requirement.

## What must become true here (durable outcome)

Pane binding happens through exactly one of the two mutually exclusive paths, always verified live, without relaxing the persistent-interactive-worker requirement — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0075`: An API-driven coordinator not inside a tmux pane has exactly two options for binding a coordinator pane, and both still require a persistent interactive worker and still fail before any intent file, the task tracker task, worktree, or fleet state is created.
- `BU-0076`: The `--managed-coordinator-pane` and `--coordinator-pane` options cannot be combined, and every path verifies through the live tmux server that the pane exists, is not dead, and reports back the same pane id, so an absent, stale, or forged identity is refused rather than adopted.
- `BU-0077`: `--managed-coordinator-pane` deliberately does not start a tmux server; a coordinator must already be able to reach a live one, so a headless environment fails loudly instead of acquiring a pane nobody can observe.
- `BU-0287`: The `--coordinator-pane` value is validated for tmux-pane-id shape before anything talks to tmux, so a malformed or argument-injecting value never reaches a tmux call.
- `BU-0897`: Sergeant looks up the managed coordinator pane by an exact window-name match and refuses (rather than guessing which pane is the coordinator) when more than one pane exists in that window, since a substring match or a shared name must never silently become 'create another one'.
- `BU-0899`: A pane found under the coordinator's window name is adopted as the coordinator only if it also carries Sergeant's own ownership marker (a tmux pane option stamped at creation), not merely because its window name matches.
- `BU-0900`: A newly created coordinator pane is stamped with its ownership marker before being returned as adopted; if the marker cannot be read back confirming the stamp, the pane is killed and creation fails, rather than leaking an unmarked, unadoptable pane.


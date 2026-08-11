# 21-recycle-terminal-worker-pane

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the interactive fleet-watch loop evaluates whether a terminal worker's pane has already been recycled

**Outcome:** a relaunch that rebinds pane/pane_identity is correctly treated as needing its own recycling, rather than being permanently suppressed by an older marker

**Statement (the operative rule):** A repo's worker_recycled evidence only counts as covering the current pane if it names that exact pane identity; a marker written for an earlier pane never suppresses recycling of a later, different pane that replaced it.

## What must become true here (durable outcome)

A relaunch that rebinds pane/pane_identity is correctly treated as needing its own recycling, rather than being permanently suppressed by an older marker — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0582`: Recording recycle evidence both rebinds the current pointer file (pane, identity, outcome, timestamp) and appends an entry to an append-only worker_recycled_log, so no earlier recycling record is ever lost even though the pointer file itself is overwritten on each recycle.
- `BU-0583`: Before a terminal worker's pane is recycled, the interactive fleet-watch loop settles any outstanding accepted action lease on the worktree first, because recycling used to stop the only process that could ever publish completion, which made a completed-but-unpublished turn permanently unrecoverable.
- `BU-0584`: The interactive fleet-watch loop stops any Claude background session associated with a repo before recycling its pane, because a background Claude session is not a child of the pane's process group and is invisible to tmux kill-pane; the stop call is idempotent so repeated recycling attempts are safe.
- `BU-0585`: If tmux is unavailable, the interactive fleet-watch loop cannot recycle a terminal worker's pane; it records a diagnostic explaining why and reports failure rather than silently treating the pane as already retired.
- `BU-0586`: The interactive fleet-watch loop determines a recorded pane is truly gone by comparing the pane id tmux display-message actually returns against the expected pane id, not by trusting the command's exit status, because display-message against a gone pane silently falls back to a default target instead of failing.
- `BU-0587`: The interactive fleet-watch loop refuses to kill a recorded pane unless its live identity still verifies as the expected supervisor, recording a diagnostic and refusing recycling rather than killing an unverified pane.
- `BU-0588`: After issuing kill-pane, the interactive fleet-watch loop re-checks that the pane id is actually gone before recording the recycle as successful, rather than trusting the kill command's exit status alone.
- `BU-0598`: The interactive fleet-watch loop only recycles a worker's pane for the terminal states done, failed, and drained; the nonterminal and unreconciled states in_progress, needs_input, blocked, waiting, and orphaned are deliberately excluded because they are still resumable and must fail closed rather than lose their pane.
- `BU-0605`: A pane no longer matching its recorded identity is never killed by the recycler — the interactive fleet-watch loop only attempts to recover any escaped worker descendants by process group, leaving a foreign pane untouched.


# 02-resolve-hunk

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-establish-conflict-state/output/outcome.md | L4 | upstream evidence produced by `establish-conflict-state` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a hunk's two sides' intents are understood

**Outcome:** the resolved hunk reflects one of the two original intents (or both), never a fabricated third behaviour

**Statement (the operative rule):** Each conflicting hunk is resolved by preserving both intents where possible; where they are incompatible, the side matching the merge's stated goal is picked and the trade-off is noted — never by inventing new behaviour.

## What must become true here (durable outcome)

The resolved hunk reflects one of the two original intents (or both), never a fabricated third behaviour — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0983`: For each conflicting change, the primary sources (commit messages, PRs, original issues/tickets) are found to understand deeply why each side's change was made and what its original intent was.


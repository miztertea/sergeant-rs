# 02-prepare-review-inputs

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-run-parallel-axis-reviews/output/outcome.md | L4 | upstream evidence produced by `run-parallel-axis-reviews` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** always, prior to spawning the sub-agents

**Outcome:** a bad ref or an empty diff fails the review at this checkpoint instead of surfacing confusingly inside two parallel sub-agents

**Statement (the operative rule):** Before spawning the two review sub-agents, the skill confirms the fixed point resolves (`git rev-parse <fixed-point>`) and that the diff is non-empty.

## What must become true here (durable outcome)

A bad ref or an empty diff fails the review at this checkpoint instead of surfacing confusingly inside two parallel sub-agents — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0929`: If the repo's issue-tracker doc is missing, /setup-matt-pocock-skills is run to establish it before the review proceeds.
- `BU-0930`: If the user doesn't specify a fixed point for the review, the skill asks for it rather than guessing.
- `BU-0932`: The spec source for the Spec axis is looked up in a fixed priority order: issue references in commit messages, a path the user passed as an argument, a matching PRD/spec file under docs/specs/.scratch, and finally asking the user directly if nothing is found.
- `BU-0939`: If no spec was found, the Spec sub-agent is skipped entirely (never run without a spec) and its absence is noted in the final report.


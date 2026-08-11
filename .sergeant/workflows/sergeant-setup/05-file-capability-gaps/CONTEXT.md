# 05-file-capability-gaps: file capability gaps

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../00-detect-prerequisites/output/README.md | L4 | upstream artifact produced by `00-detect-prerequisites` |

## Purpose

Each unsupported capability becomes an approved tracked issue, or is reported as an unfilled gap.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

Each unsupported capability becomes an approved tracked issue, or is reported as an unfilled gap.

## Behavior contract

- **For each unsupported prerequisite, sergeant-setup drafts a td issue (title, description, acceptance criteria) and shows it for explicit y/yes approval before creating it; on decline it reports the gap in the summary and creates no tracked work.**
  (trigger: a required or optional prerequisite is classified unsupported; outcome: either a tracked issue exists or the gap is explicitly reported, never silently dropped or silently created)
  — `BU-P5-012`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 77-89)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

# 10-identify-spec-source: identify spec source

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-pin-fixed-point/output/README.md | L4 | upstream artifact produced by `00-pin-fixed-point` |

## Purpose

The spec source is identified via a fixed priority order ending in asking the user.

Trigger (workflow-level): A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## What must become true here (durable outcome)

The spec source is identified via a fixed priority order ending in asking the user.

## Behavior contract

- **The spec source for the Spec axis is located in a fixed priority order: issue references in commit messages, then a path the user passed as an argument, then a PRD/spec file under docs/, specs/, or .scratch/ matching the branch or feature name, then — if nothing is found — asking the user; if the user says no spec exists, the Spec sub-agent is skipped and reports 'no spec available'.**
  (trigger: identifying what the Spec axis should compare against; outcome: a spec source is found, or the Spec review is explicitly skipped with a reason)
  — `BU-P2-007`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 2: Identify the spec source, lines 27-32)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

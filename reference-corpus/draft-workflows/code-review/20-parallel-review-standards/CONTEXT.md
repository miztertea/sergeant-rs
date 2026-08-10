# 20-parallel-review-standards: parallel review standards

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-identify-spec-source/output/README.md | L4 | upstream artifact produced by `10-identify-spec-source` |

## Purpose

An isolated review against the repository's documented coding standards.

Trigger (workflow-level): A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## What must become true here (durable outcome)

An isolated review against the repository's documented coding standards.

## Behavior contract

- **The Standards review axis asks whether the code conforms to the repository's documented coding standards.**
  (trigger: a diff is being reviewed; outcome: a Standards-axis assessment is produced)
  — `BU-P2-001`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (front matter / Process intro, lines 8-8)
- **The Spec review axis asks whether the code faithfully implements the originating issue, PRD, or spec.**
  (trigger: a diff is being reviewed; outcome: a Spec-axis assessment is produced)
  — `BU-P2-002`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (front matter / Process intro, lines 9-9)
- **A documented repository standard always overrides the smell baseline: where the repo's own standard endorses something the baseline would flag, the smell is suppressed.**
  (trigger: a baseline smell conflicts with a repo-documented standard; outcome: the repo's documented standard wins and the smell is suppressed)
  — `BU-P2-009`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 3: Identify the standards sources, lines 40-40)
- **Every baseline smell is a labelled judgment-call heuristic, never a hard violation, and the reviewer must skip anything tooling already enforces.**
  (trigger: applying the smell baseline; outcome: smells are reported as judgment calls, not hard failures; tooling-enforced items are not repeated)
  — `BU-P2-010`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 3: Identify the standards sources, lines 41-41)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

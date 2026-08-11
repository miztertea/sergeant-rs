# 10-sketch-seams: sketch seams

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-gather-context/output/README.md | L4 | upstream artifact produced by `00-gather-context` |

## Purpose

The fewest new seams at the highest possible seam, confirmed with the user; the spec is then written on the fixed template and published to the tracker with the ready label.

Trigger (workflow-level): A design needs to be turned into a spec-shaped ticket before implementation.

## What must become true here (durable outcome)

The fewest new seams at the highest possible seam, confirmed with the user; a fixed template is published to the tracker with the ready label.

## Behavior contract

- **Before writing a spec's implementation section, sketch out the seams at which the feature will be tested, preferring existing seams and the highest possible seam, aiming for as few new seams as possible (ideally exactly one).**
  (trigger: a spec's testing/implementation shape is being decided; outcome: the spec commits to a minimal, high-leverage seam plan before publication)
  — `BU-P4-052`, `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (Process step 2, L15)
- **After sketching test seams for a spec, confirm with the user that the proposed seams match their expectations before finalizing the spec.**
  (trigger: candidate test seams have been sketched for a spec; outcome: the seam plan is user-confirmed before the spec is written and published)
  — `BU-P4-053`, `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (Process step 2, L17)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helper invocation: write and publish

Demoted from a standalone stage (`20-write-and-publish`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed once the seam plan is user-confirmed. No `kind = "execute"` stage exists in the current engine, so the acting harness performs the write-and-publish operation itself:

- **Write the spec using the fixed spec template, publish it to the project issue tracker, and apply the ready-for-agent triage label without requiring additional triage.**
  (trigger: the spec content is finalized; outcome: a published, triage-labeled spec issue exists)
  — `BU-P4-054`, `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (Process step 3, L19)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

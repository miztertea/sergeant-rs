# 60-update-owned: update owned

## Inputs

| File | Layer | Why |
|---|---|---|
| — | — | no contract-bearing upstream dependency beyond this workflow's ordering |

## Purpose

For Sergeant-owned skills: update this repository through a reviewed PR and run the instruction-policy test plus the full test suite.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

For Sergeant-owned skills: update this repository through a reviewed PR and run the instruction-policy test plus the full test suite.

## Behavior contract

- **For Sergeant-owned skills, update this repository through a reviewed PR and run tests/instruction-policy-test.sh plus the full Sergeant test suite.**
  (trigger: updating a Sergeant-owned skill; outcome: no Sergeant-owned skill changes ship without passing review and the full test suite, including the instruction-policy test)
  — `BU-P1-127`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L142-144, Sergeant-owned update path)

## Judgment required

Reclassified from `stage (§6.3, deterministic-machinery candidate)` to actor-stage at N1 adjudication A4: the checkpoint here is not running the test suite (deterministic machinery) but the decision that gates it — updating only through a reviewed PR, i.e. human review plus a passing instruction-policy test and full suite before changes ship. That decision survives any reimplementation of how the tests themselves are run (§6.3's test), so it is genuine judgment: the acting harness must prepare the change for review and confirm the gating conditions are met, or explain why not — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

Alternate entry: only reached when updating an already-adopted, Sergeant-owned skill, not during the initial `00`-`50` vetting sequence. Mutually exclusive with `60-update-managed`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

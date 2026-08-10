# 20-verify: verify

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-gather-context/output/README.md | L4 | upstream artifact produced by `10-gather-context` |

## Purpose

The claim is reproduced or the PR diff is tested, reported as confirmed/failed/insufficient.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

The claim is reproduced or the PR diff is tested, reported as confirmed/failed/insufficient.

## Behavior contract

- **Before grilling, the actor verifies the claim empirically — reproducing a bug or checking out and testing a PR's diff — and reports one of confirmed, failed, or insufficient-detail, where confirmation strengthens the eventual agent brief.**
  (trigger: a recommendation has been given and direction received; outcome: the claim's validity is empirically established before further action)
  — `BU-P3-067`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 74)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

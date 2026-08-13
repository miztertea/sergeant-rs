# 40-grill-if-underspecified: grill if underspecified

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-recommend/output/README.md | L4 | upstream artifact produced by `30-recommend` |

## Purpose

Underspecified items are escalated to an interview.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

Underspecified items are escalated to an interview.

## Behavior contract

- **If the item is underspecified after verification, the actor invokes the grilling procedure to sharpen it into shape.**
  (trigger: verification shows the request needs fleshing out; outcome: the item's specification and domain terms are sharpened, with decisions captured inline)
  — `BU-P3-068`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 76). Upstream pairs this with a
  separate `domain-modeling` procedure; no `domain-modeling` skill package
  exists in this repo yet (only frozen upstream evidence — see
  `docs/icm/agents-invariant-dispositions.md` BU-1064), so sharpening
  domain terminology folds into the same `grilling` session below rather
  than a second invocation.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running the **grilling** operator skill
(`skills/grilling/SKILL.md`) to completion, live in this session — not by
dispatching a Work item. `grilling` retired as a `.sergeant/workflows/`
package at the MVP-5 F2 execution-surface re-triage (North Star ruling
R-NS-6: conversation is the harness's job, never engine work; see
`docs/icm/re-homing-record-2026-08-12.md`), which also resolves the E3
dependency this stage previously inherited from the retired package's
WORKFLOW-IF-E3 classification.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

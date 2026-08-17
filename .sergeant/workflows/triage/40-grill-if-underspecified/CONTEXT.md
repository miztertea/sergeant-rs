# 40-grill-if-underspecified: grill if underspecified

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-verify/output/README.md | L4 | upstream artifact produced by `30-verify` |

## Purpose

Underspecified items are escalated to an interview.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

Underspecified items are escalated to an interview.

## Behavior contract

- **If the item is underspecified after verification, the actor invokes the grilling procedure to sharpen it into shape.**
  (trigger: verification shows the request needs fleshing out; outcome: the item's specification and domain terms are sharpened, with decisions captured inline)
  Upstream pairs this with a separate `domain-modeling` procedure; no
  `domain-modeling` skill package exists in this repo yet (only frozen
  upstream evidence), so sharpening domain terminology folds into the same
  `grilling` session below rather than a second invocation.

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Judging whether the item is underspecified after verification, and running the `grilling` skill to sharpen it.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None beyond what the `grilling` skill's own live interview already surfaces to the user.

### Completion boundary
This stage may complete only when the item is either already sufficiently specified (skip) or has been sharpened via a completed `grilling` session, with decisions captured inline.

### Decision evidence
The sharpened specification (or the skip decision, if already sufficient) is this stage's own durable output.

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

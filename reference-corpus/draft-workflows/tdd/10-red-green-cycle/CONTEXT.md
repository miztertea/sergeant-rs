# 10-red-green-cycle: red green cycle

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-agree-seams/output/README.md | L4 | upstream artifact produced by `00-agree-seams` |

## Purpose

One seam, one test, one minimal implementation, vertical slices only.

Trigger (workflow-level): A feature or bug fix is being implemented test-first.

## What must become true here (durable outcome)

One seam, one test, one minimal implementation, vertical slices only.

## Behavior contract

- **Horizontal slicing (writing all tests first, then all implementation) verifies imagined behavior rather than user-facing behavior, tests the shape of things rather than real behavior, goes insensitive to real changes, and commits to test structure before the implementation is understood; work should instead proceed in vertical slices — one test, one implementation, repeat — each test a tracer bullet responding to what the last cycle taught.**
  (trigger: planning how to sequence tests versus implementation across a feature; outcome: work proceeds one test-then-implementation slice at a time rather than in bulk-test, bulk-implement phases)
  — `BU-P2-113`, `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Anti-patterns: horizontal slicing, lines 30-30)
- **Red before green: the failing test is written first, then only enough code is written to pass it, without anticipating future tests or adding speculative features.**
  (trigger: starting a red-green cycle; outcome: the test exists and fails before any implementation code is written, and the implementation is minimal)
  — `BU-P2-114`, `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Rules of the loop, lines 34-34)
- **One slice at a time: one seam, one test, one minimal implementation per cycle.**
  (trigger: running a red-green cycle; outcome: each cycle's scope is bounded to exactly one seam/test/implementation triple)
  — `BU-P2-115`, `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Rules of the loop, lines 35-35)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.

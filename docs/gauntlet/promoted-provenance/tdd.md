# Provenance — TDD

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W22** `tdd`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-104` | The tdd workflow triggers when the user wants to build features or fix bugs test-first, mentions 'red-green-refactor', or wants integration tests. | `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (front matter description, lines 3-3) |
| `BU-P2-105` | TDD is the red-to-green loop; this skill is the reference that makes that loop produce tests worth keeping (what a good test is, where tests go, anti-patterns, and the rules of the loop), and every section applies on every cycle, consulted before and during the loop, not after. | `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (intro, lines 8-8) |
| `BU-P2-116` | Refactoring is not part of the red-green loop; it belongs to the review stage (the code-review skill), not the implementation cycle. | `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Rules of the loop, lines 36-36) |

## Stages

### `00-agree-seams`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-109` | Test seams must be pre-agreed: before writing any test the actor writes down the seams under test and confirms them with the user, and no test is written at an unconfirmed seam, since agreeing seams up front is how testing effort lands on critical paths and complex logic instead of every edge case. | `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Seams, lines 22-22) |
| `BU-P2-110` | The actor asks the user: 'What's the public interface, and which seams should we test?' | `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Seams, lines 24-24) |

### `10-red-green-cycle`

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-113` | Horizontal slicing (writing all tests first, then all implementation) verifies imagined behavior rather than user-facing behavior, tests the shape of things rather than real behavior, goes insensitive to real changes, and commits to test structure before the implementation is understood; work should instead proceed in vertical slices — one test, one implementation, repeat — each test a tracer bullet responding to what the last cycle taught. | `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Anti-patterns: horizontal slicing, lines 30-30) |
| `BU-P2-114` | Red before green: the failing test is written first, then only enough code is written to pass it, without anticipating future tests or adding speculative features. | `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Rules of the loop, lines 34-34) |
| `BU-P2-115` | One slice at a time: one seam, one test, one minimal implementation per cycle. | `reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (Rules of the loop, lines 35-35) |

## Notes

**Synthesis notes:** Refactoring is explicitly *not* a stage of this workflow (BU-P2-116) — it hands off to `code-review`/deepen-module discipline instead. The bulk of the `tdd` source is reference guidance, not procedure (16 units land in the `test-quality` shared context, not in this workflow).


# Provenance — Deepen Module

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W25** `deepen-module`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-013` | Deepening a cluster of shallow modules is a distinct, bounded procedure: classify the cluster's dependencies, apply seam discipline, and replace old tests with tests at the new deepened interface. | `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (L1-3) |

## Stages

### `00-classify-dependencies`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-014` | When a deepening candidate's dependencies are pure in-process computation with no I/O, always merge the modules and test the result directly through the new interface; no adapter is needed. | `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Dependency categories / In-process, L11) |
| `BU-P4-015` | When a deepening candidate depends on something with a local test stand-in (e.g. an in-memory filesystem or an in-process database emulator), deepening is possible and the deepened module is tested against that stand-in inside the test suite, with the seam kept internal. | `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Dependency categories / Local-substitutable, L15) |
| `BU-P4-016` | When a deepening candidate depends on the team's own remote services, define a port (interface) at the seam owned by the deep module, inject an in-memory adapter for tests and an HTTP/gRPC/queue adapter for production. | `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Dependency categories / Remote but owned, L19) |
| `BU-P4-017` | When a deepening candidate depends on a true third-party external service the team doesn't control, inject that dependency as a port and give tests a mock adapter. | `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Dependency categories / True external, L25) |

### `10-design-it-twice`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-022` | When exploring alternative interfaces for a chosen deepening candidate, run a parallel-sub-agent pattern that produces several radically different designs before picking one, on the premise that a single first idea is unlikely to be the best. | `reference/sergeant-upstream/.agents/skills/codebase-design/DESIGN-IT-TWICE.md` (L3) |
| `BU-P4-023` | Before spawning parallel design sub-agents, first produce and show the user a framing of the problem space (constraints, dependency category, an illustrative sketch), then proceed immediately to spawning sub-agents without waiting for a reply. | `reference/sergeant-upstream/.agents/skills/codebase-design/DESIGN-IT-TWICE.md` (Process step 1, L17) |
| `BU-P4-024` | Produce at least three independently-generated, radically different interface designs for the same deepening candidate, each under an explicit distinguishing design constraint (e.g. minimal interface, maximal flexibility, optimize the common case, ports-and-adapters). | `reference/sergeant-upstream/.agents/skills/codebase-design/DESIGN-IT-TWICE.md` (Process step 2, L21) |
| `BU-P4-026` | Present the several generated interface designs to the user one at a time so each can be absorbed, then compare them explicitly by depth (leverage at the interface), locality (where change concentrates), and seam placement, ending with an opinionated recommendation (including a hybrid if warranted) rather than a menu. | `reference/sergeant-upstream/.agents/skills/codebase-design/DESIGN-IT-TWICE.md` (Process step 3, L42) |

### `20-test-at-new-interface`

| Unit | Statement | Source |
|---|---|---|
| `BU-P4-020` | After deepening a module, delete the old unit tests that targeted the now-merged shallow modules rather than keeping them alongside new interface-level tests. | `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Testing strategy, L34) |
| `BU-P4-021` | Tests written against a deepened module must assert on observable outcomes through the interface, not on internal state, so they survive internal refactors; a test that must change when the implementation changes is testing past the interface. | `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Testing strategy, L37) |

**Curation note (promotion, `docs/icm/promotion-spec-2026-08-11.md` §1):**
`20-test-at-new-interface`, this package's true closing stage, declares a
`promote`-dispositioned output with no finalize step — one of the 30 N1
drafts in that bucket, not one of the 3 (`drain-fleet`, `respond-to-worker`,
`to-spec`) that name one. D9 (`docs/icm/convention.md` §1a, "Open
questions") does not block promotion on this; disposition here is applied
by human review at merge time for this package, not mechanically.


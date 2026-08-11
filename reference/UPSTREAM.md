# Reference Corpus Provenance

This directory is committed reference material for the sergeant-rs prototype. It is
evidence, not source: nothing in here is built, imported, or executed by the crate.

| Item | Source | Pinned at | Date vendored |
|---|---|---|---|
| `proposal-depot-rust-execution-surface.md` | User-supplied proposal ("Depot — Rust-Native Agent Execution Surface") | document dated 2026-08-08 | 2026-08-08 |
| `sergeant-upstream/` | https://github.com/miztertea/sergeant (fork of https://github.com/callmeradical/sergeant) | `f430cfd4f90174a98adbd7abebbece6303817929` (main, includes merged PR #2 — Claude background harness) | 2026-08-08 |
| `notes/` | Distillations written for this prototype (sources cited inline) | — | 2026-08-08 |
| `proposal-next-iteration-icm-workflows.md` | Owner-supplied successor proposal ("Sergeant-rs Next Iteration: Measured ICM Workflows and Portable Execution"), delivered via the owner's IdeaOS Drive corpus | document dated 2026-08-10, audited against repo revision `27c00ef` | 2026-08-10 |

Why the fork rather than the original: the fork is a superset containing the measured
Claude background-harness research spike (`sergeant-upstream/docs/research/claude-background-harness-spike.md`),
its PRD (`sergeant-upstream/docs/prds/claude-background-harness.md`), and the fake-CLI
test suite (`sergeant-upstream/tests/sgt-claude-*.sh`) — all direct inputs to this
prototype's Claude backend adapter.

Naming note: the proposal names the product "Depot". The product built in this
repository is **sergeant-rs** (binary `sgt`) by explicit owner decision, 2026-08-08.
The proposal is treated as the idea as it stood in that moment, not a how-to guide;
deviations are logged in `GAUNTLET.md` with rationale.

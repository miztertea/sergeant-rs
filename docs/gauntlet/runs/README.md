# Run evidence — where the extracted workflows live

Three generations of extracted ICM workflow packages exist in this repo:

| Set | Location | Status |
|---|---|---|
| **Adjudicated reference** (N1) — 966 behavior units, **34 draft workflow packages** | `reference-corpus/` (repo root) | Frozen; the curation source of record |
| Run 2 generated (generator v1) | `docs/gauntlet/runs/n2-run2/drafts/workflows/` | Evidence only (11.8% coverage) |
| Run 3 generated (generator v2) | `docs/gauntlet/runs/n2-run3/.sergeant/` — **note the dotted dir**: `drafts/workflows/` (18 packages) + all stage intermediates | Evidence (34.1% coverage, §22.2 met) |

None of these are installed as *usable* workflows yet — `.sergeant/workflows/`
holds only `repo-to-icm` and the `software-change` sample. The promotion step
(curate `reference-corpus/draft-workflows/` → a runnable `.sergeant/workflows/`
library, structure-validated and engine-tested) is queued Cerberus work — the
program's actual endgame: making sergeant human-usable out of the box.

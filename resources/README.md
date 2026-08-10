# resources/ — S-series workflow scripts

Orchestration scripts for the S-series gauntlet, committed as plain `.js`
as launched (owner direction 2026-08-10). This supersedes the zip-append
convention (`reference/gauntlet-workflows.zip` remains the M/N-series
archive). One file per workflow invocation; edits after launch land as new
commits, never rewrites — the file is the record of what ran.

- `s1-instrument-gauntlet.js` — S1 phase 1: instrument repairs + hygiene +
  harness build (contract: `docs/gauntlet/contracts/S1-COVERAGE.md`).
- `s1-analysis-gauntlet.js` — S1 phase 2/3 support: per-stage artifact
  analysis and the batched reproduce-or-refute pass over candidate
  findings. Phase 2's stage execution itself is orchestrator-run from
  `scripts/coverage/` (R-S0-4 sequencing), not agent-run.

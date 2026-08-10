# resources/ — S-series workflow scripts

Orchestration scripts for the S-series gauntlet, committed as plain `.js`
as launched (owner direction 2026-08-10). This supersedes the zip-append
convention (`reference/gauntlet-workflows.zip` remains the M/N-series
archive). One file per workflow invocation; edits after launch land as new
commits, never rewrites — the file is the record of what ran.

- `s1-instrument-gauntlet.js` — S1 phase 1: instrument repairs + hygiene +
  harness build (contract: `docs/gauntlet/contracts/S1-COVERAGE.md`).
- `s1-instrument-round2-lean.js` — S1 phase 1, lean round 2: fresh blind
  critics over the phase-1 diff (test-honesty independently re-probes every
  pin), batched refuters, fixer. Added at owner challenge 2026-08-10; the
  measurement SHA freezes only after this round leaves the tree green.
- `s1-analysis-gauntlet.js` — S1 phase 2/3 support: per-stage artifact
  analysis and the batched reproduce-or-refute pass over candidate
  findings. Phase 2's stage execution itself is orchestrator-run from
  `scripts/coverage/` (R-S0-4 sequencing), not agent-run.
- `s2-wave1-gauntlet.js` — S2 wave 1 (contract:
  `docs/gauntlet/contracts/S2-STABILIZE.md`): four Sonnet builders on
  disjoint surfaces (fixtures + in-module pokes for #30/#32/#33/#34/#37/
  #39/#41), then the full loop — two-critic panel, batched refuters,
  fixer (R-S0-12: code is code, waves included).

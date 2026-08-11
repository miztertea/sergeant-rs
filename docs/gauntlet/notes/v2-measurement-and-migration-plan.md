# Generator v2 Measurement + Cerberus Migration Plan (2026-08-11)

## Run A — v2 recall measurement (launching now)

Same method as N2 run 2 (fake-held stages, fresh blind agent per stage,
real daemon+worktree) so the v1→v2 delta isolates the CONTENT changes:
same subject (sergeant-upstream @ f430cfd), same blind comparison vs
reference corpus v1, same §9.9 dimensions. Success question: did the
consequence-class sweep + partition checkpoint/retry protocol move
in-scope recall (v1: 47.3%, 11 silent consequence-class misses) and
coverage (v1: 16/136 files) without losing v1's clean precision? Compare
via 2 Sonnet comparers + Opus adjudicator against the same scorecard
frame. Evidence → docs/gauntlet/runs/n2-run3/.

## Run B — real-Claude run (closes #19; after Run A)

Same v2 workflow through the REAL claude backend (2.1.226): actors are
Claude sessions driven by the daemon — turns, asks (GP-2 pathway live),
retries. Scope bounded by the intent (root-instructions + one bin
partition) to bound spend; usage recorded in the run manifest per R-N0-6.
This is the first true dogfood of the full product loop and the last
in-container milestone item.

## Then: PR #43 review/merge → Cerberus

Merge-order note: #43 first (based on current main); #28's reconciliation
(GAUNTLET/LESSONS/CLAUDE.md collisions, pre-#27 base) is larger and lands
second either way.

Cerberus first acts (from the N-series record): clone; `cargo build`;
`sgt doctor`; real-Docker lifecycle probe (overlay2, networking, registry
pulls — the things this container could not measure); re-run
`scripts/perf/run-all.sh` for the Cerberus baseline (all N-series budgets
are container-relative; re-baseline before N4 budgets are set); re-run
the opt-in real-Claude contract suite; then the N4 contract — which per
ruling R-N0-3 cannot be written without the #17/#4 retention ruling, and
per A-N3-1 requires #44 (journal group commit) to land first.

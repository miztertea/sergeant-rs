# Coverage artifacts — 2026-08-10

Committed on purpose. These are the raw outputs the S1 baseline is read from;
a baseline whose evidence lives only in a container is a claim, not a
measurement. The harness that writes them is `scripts/coverage/`, which also
explains the convention and what each stage checks.

Small by rule: logs, counts and verdicts only. Profraws, `.profdata`, the lcov
file and the HTML tree never land here — they go to `target/llvm-cov-reports/`
and are regenerable from the collected profiles in seconds. What is committed
about them is their size and the lcov's SHA-256, in `c4-report-provenance.tsv`.

Present after phase 1 (the instrument's own measurements):

| file | what it is |
| --- | --- |
| `tool-probes-2026-08-10.txt` | cargo-llvm-cov 0.8.7 measured, one linear pass, against a throwaway crate outside the tree (L5). Answers the four tool Unknowns of proposal §14 |
| `phase1-measurements.txt` | the timing and volume numbers the harness README's headroom table cites |

Written by phase 2, one set per stage (`c0`, `c1`, `c2-*`, `c3-*`, `c4`,
`f1`, `f2`):

| file | what it is |
| --- | --- |
| `toolchain.txt` | `rustc -vV`, `cargo --version`, `cargo llvm-cov --version` — written once, compared by every later stage (R-S0-2) |
| `<stage>.log` | the stage's transcript |
| `<stage>-accounting.tsv` | wall time; profraws before / produced / mergeable / discarded / total (R-S0-6, §6.3) |
| `<stage>-hygiene.tsv` | the post-stage sweep: leaked daemons (must be none), `/tmp` entries |
| `<stage>-profraw-discarded.txt` | present only when a profraw could not be merged — each one named |
| `c0-show-env.txt`, `c0-verdict.tsv` | the environment verbatim, and the profraw-pattern verdict |
| `c4-summary.txt` | the per-file coverage table: the baseline's numbers |
| `c4-report-provenance.tsv` | profraws present/merged, profdata size, lcov size + SHA-256, report paths |
| `f1-runs.tsv`, `f1-failures.tsv`, `f1-census.tsv` | control arm: per-run outcome and wall time, failures by name, requested-vs-completed N |
| `f2-runs.tsv`, `f2-failures.tsv`, `f2-census.tsv` | instrumented arm, plus the wall-time ratio against F1 |
| `f1-run-N.log`, `f2-run-N-*.log` | kept **only** for runs that failed; a green transcript is not evidence of anything |

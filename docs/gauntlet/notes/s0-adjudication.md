# S0 adjudication — challenge record and dispositions

2026-08-10. One fresh-context Opus challenger (high effort) ran four axes
against `reference/proposal-s-series-stabilization.md` (v0, pre-amendment),
with the register-first rule (L3) and read-only tree verification. 35
findings: 9 error, 17 warning, 9 info (of which 6 were verified-credit
confirmations of proposal claims). Every finding ruled by the orchestrator;
the proposal was amended in place before any S-contract was drawn. Rulings
carrying forward live in `docs/gauntlet/contracts/S0.md`.

## Disposition table

| # | Sev | Finding (compressed) | Disposition |
|---|---|---|---|
| A1 | error | §6 claimed m3/m5 in-process; both spawn the binary (m3:61, m5:56, one test each) | amended §6: only m1/m4 are sgt-subprocess-free |
| A2 | warn | "16 of 220" undercounts; 17–18 of 218 | amended |
| A3 | error | §14 Unknown described a `--test-threads=1` config that does not exist anywhere in the tree | amended §7/§14: suites are parallel; the question is whether instrumentation reintroduces the M3-era parallel-flake class |
| A4 | error | demo.sh cleanup: SIGTERM, 5 s, no escalation, then `rm -rf` under a possibly-live daemon invisible to the DataDir guard | registered §6.1(2); instrument repair with fail-closed verify-gone pin |
| A5 | warn | "pinned stable toolchain" floats (`channel = "stable"`) | R-S0-2 drift guard: record `rustc -vV`, refuse cross-version merges; toolchain file unchanged |
| A6 | info | component is `llvm-tools`, `-preview` is the legacy alias | amended |
| A7–A9 | info | credit: LLVM 18-vs-21 trap, §6 preconditions, language census all verified true | recorded |
| A10 | warn | timing at-risk list incomplete and inverted (m5 bound has ~27× headroom; the 30 s daemon-boot deadline is unknown) | amended §7: enumerate all deadlines in the harness doc, rank by measured headroom |
| A11 | warn | `on: [push, pull_request]` runs CI twice per PR push | amended §10: trigger fix is an S1 phase-1 roll-in |
| A12 | warn | "claude.rs under-reports" was an unbounded excuse | amended §6.2: name live-path-only regions; hand quantification to N2 per R-N0-6 |
| B1 | error | §8.1 roll-ins carried no L7 obligation; instrument repairs are pinnable (SIGTERM-exit assertion) and this repo has ruled instruments-get-tested twice (M6 reaper/scanner; m2 reaper test) | R-S0-5: no L7 exemption; each roll-in class names its pin; unfalsifiable pin → discuss-first |
| B2 | error | the measurement convention was unpublished — L11's exact failure shape (a number no stranger can reproduce) | R-S0-3: literal command lines committed in `scripts/coverage/` and quoted in the baseline |
| B3 | warn | roll-ins could interleave with measurement, unpinning the baseline SHA | R-S0-4: S1 is three strictly ordered phases; mid-measurement repair restarts phase 2 |
| B4 | warn | untracking the vendored `.pyc` breaks `reference/UPSTREAM.md`'s byte-identity claim for zero language-bar gain | R-S0-8: leave it tracked; `.gitignore` prevents new ones |
| B5 | warn | `reference/UPSTREAM.md` is a fourth shared-append file with the N-branch | amended §11 |
| B6 | info | credit: no conflicts with D1–D8; R-N0-6 and B1–B3 handled per L3 | recorded |
| C1 | warn | S3 is one commit, not a milestone | R-S0-9: folded into S2 close-out |
| C2 | warn | S0's outcome restated already-settled owner directions | S0 rescoped to the ruling list |
| C3 | error | flake census had no N and no control arm — "fails only under instrumentation" was unprovable | R-S0-7: N=10 uninstrumented control + N=3 instrumented, ceiling ~4 h, unreproduced-failure rule |
| C4 | error | invocation/scope unspecified; S1 builder would invent the deliverable | merged into R-S0-3 |
| C5 | warn | "one issue per confirmed gap" had no granularity rule | R-S0-11: dedupe by subsystem/root-cause, cap 12, behavior-named |
| C6 | warn | `Cargo.toml` claimed with no stated need | R-S0-10: owned by neither program |
| C7 | info | credit: doctrine and register are right-weight; over-machinery was in milestone count only | recorded |
| D1 | error | no disk budget against two recorded ENOSPC incidents; census profraw compounds | R-S0-6: 10 GB pre-flight floor, profraw clean between runs, two-tree rule |
| D2 | error | no-mistakes absent from container; `scripts/gate.sh` cannot run; §1 promised "through the gauntlet" | R-S0-1: Bug Sprint 1 gate regime (orchestrator-verified gates + hygiene sweep), recorded per milestone |
| D3 | error | no `gh` CLI → no autonomous issue filing | R-S0-11: orchestrator files via GitHub MCP tooling (which the challenger's subshell lacked but this session holds); committed-findings fallback |
| D4 | warn | no coverage label or template exists | S1 delivers `.github/ISSUE_TEMPLATE/coverage-gap.yml`, `labels: ["coverage"]` |
| D5 | warn | #21 (dashboard JS) is outside the instrument's reach but was cited as baseline-informed | amended §2/§13: explicit non-goal |
| D6 | warn | test-only-by-ruling regions would invite dead-code filings | amended §6.2: B1 snapshot path and `Analytics::table_rows` pre-ruled |
| D7 | warn | no gauntlet-depth statement | added to the S1 contract |
| D8 | warn | merge sequencing unstated; N-branch adds a second tracked pycache | amended §11 (order), §9 (ignore rules; N's pycache is theirs) |
| D9 | warn | no corrupt-profraw policy; SIGKILL mid-flush can hard-fail the merge | R-S0-6 accounting: count produced/merged/discarded, fail on unaccounted |
| D10 | info | credit: the profile-pattern-absoluteness Unknown is the right first question | kept as the first phase-2 check |

## Orchestrator notes

- The challenger's D3 was correct about the container (`gh` absent) and
  wrong about the session: issue filing runs through the orchestrator's
  GitHub MCP tools. The finding still earned its keep — the proposal never
  named any mechanism, and the fallback it forced into §15.2 is real.
- One challenger claim was *not* adopted as fact: cargo-llvm-cov's default
  `tests/`-exclusion and `--failure-mode` semantics. Both are recorded as
  Unknowns for the S1 builder to measure (doctrine 1 / L1), not asserted.
- Full challenge text preserved in the session record; this table is the
  ledger-grade summary.

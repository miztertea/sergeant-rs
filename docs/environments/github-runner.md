# GitHub Actions hosted runner (CI)

Facts measured 2026-08-10 → 2026-08-11 via CI runs on `miztertea/sergeant-rs`.

| Fact | Measured value | Evidence |
|---|---|---|
| uid | non-root (`runner`) | chattr refusal, run 31448175583 |
| `CAP_LINUX_IMMUTABLE` | absent — `chattr +i` fails "Operation not permitted" | run 31448175583; probe-gate in `tests/m3_execution.rs` |
| O_DIRECT alignment | **unenforced** — unaligned O_DIRECT writes are accepted | run 31447702864; probe-gate in `src/runtime/journal.rs` |
| Cores | 2 (parallel-timing tests must not assume more) | P1-PERF + S2 runs |
| Node runtime | actions forced to Node 24 (Node 20 deprecated) | run 31448175583 log tail |

Fixture rule (CLAUDE.md testing section): shapes no hosted-runner user can
change (capabilities, kernel/FS enforcement) skip loudly; locally-fixable
preconditions stay hard failures.

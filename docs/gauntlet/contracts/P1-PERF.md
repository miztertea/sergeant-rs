# P1-PERF Contract — Load, Stress, and Resource Baseline

**Outcome.** The P0 prototype is put through its paces and the results become
two durable artifacts: (1) a rerunnable perf/stress harness committed under
`scripts/perf/`, and (2) a measured baseline document plus a GitHub issue for
every confirmed defect or anomaly. **This phase fixes nothing** — findings go
to the issue backlog, not to patches. Usability/design of the TUI and
dashboard are explicitly out of scope (a later phase); they matter here only
if they crash, leak, or wedge.

**Binary under test.** The release build (`cargo build --release`) — the
baseline describes what ships, not debug-profile DuckDB. Record the exact
commit SHA and container specs (cores, RAM, disk) in the baseline doc.

**Scenario matrix.** Each scenario runs SEQUENTIALLY (never overlapping
another scenario's measurement window), against a fresh data dir unless the
scenario says otherwise, with the fake backend unless it says otherwise.
Every scenario ends with the hygiene sweep: zero orphan `sgt` processes,
zero stray worktrees, `/tmp` residue accounted for, daemon fd count back to
idle range.

1. **S1 burst submissions** — concurrent `sgt run` bursts of 1, 5, 10, 20,
   50 (completing fake script). Per burst: wall time, per-call latency
   min/median/p95/max, daemon RSS before/peak/after, daemon CPU time,
   journal events appended, all works reach `completed`.
2. **S2 sustained churn + leak watch** — 200 works in waves of 10; sample
   daemon VmRSS/VmHWM/fd-count/CPU every wave; after the run and a settle
   period, RSS must return near the pre-run mark (define "near" from the
   data; a monotonic climb is a finding). Account journal + blob + surface
   disk growth (bytes per work).
3. **S3 deep work + graph load** — one work driven through many
   needs_input→respond cycles until its journal trail is ≥1,000 events;
   then 100 sequential + 20-concurrent×5 `GET /v1/graph/work/{id}` reads:
   latency distribution, response size, daemon RSS delta. Also
   `work show --json` and the events tail endpoints at that depth.
4. **S4 SSE fan-out** — 25 concurrent SSE subscribers held open through a
   20-work burst: every subscriber sees the burst's terminal events, daemon
   fd count during/after, memory delta, subscriber disconnect mid-stream
   does not wedge the daemon (repeat burst with 10 subscribers killed
   mid-flight).
5. **S5 journal scale: rebuild + analytics** — grow one data dir to ≥50k
   events (script waves); measure: daemon cold-start rebuild wall time and
   peak RSS at 10k/25k/50k, all canned `sgt analytics` questions' latency
   at each mark, DuckDB file size. The §21 rebuild budget claim (M5 ledger:
   ~15k events/s) either holds at scale or the deviation is a finding.
6. **S6 crash + recovery under load** — `kill -9` the daemon mid-burst
   (10 in flight), restart, assert: journal tail recovery, every in-flight
   work lands in a legal state (fail-closed `blocked` acceptable, silent
   loss or duplicate execution is a defect), no orphan processes from the
   killed daemon's executions, second restart is idempotent. Repeat 3×.
7. **S7 client-process hygiene** — TUI attached (tmux) through a burst:
   client CPU while idle-but-live (a busy render loop is a finding), SSE
   tail stays live, `q` exits restoring the terminal; TUI orphaned from its
   pty (tmux kill) must die on SIGTERM (the 2026-08-09 screenshot session
   observed one needing SIGKILL — reproduce or refute); dashboard fetched
   during burst stays correct; `sgt web`/`doctor`/`status` under load.

**Measurement rules.** Sample from `/proc/<pid>/{status,stat,fd}` — VmRSS,
VmHWM, utime+stime, fd count; wall clocks via `date +%s%N` around calls.
Idle-daemon baseline (2 min settle, sampled) recorded before any scenario.
Numbers are recorded raw in per-scenario JSON/CSV under the run's output
dir (not committed); the baseline doc commits the distilled table. A
scenario that cannot hit its target (e.g. 50-way burst saturates something)
records where it stopped and why — that is data, not failure.

**Findings discipline.** A finding is a claim with a reproduction: the
scenario, the numbers, and the smallest rerun that shows it. Confirmed
findings each become one GitHub issue (title, repro, measurements,
suspected subsystem, severity label). Known seeds entering this phase:
the TUI fleet stage/backend column collision (screenshot-evidenced) and
the TUI orphan SIGTERM observation (S7 owns repro). Backlog B3's stall is
claude-adapter-specific and stays out of scope (fake `stop` is instant);
the multi-client contention S1/S4 exercise is the general case.

**Non-goals.** Fixing anything (issues only); real-Claude load (token
budget — the fake backend is the §37 instrument); TUI/dashboard usability;
benchmarking rig micro-optimization; new Rust code beyond what a scenario
strictly needs (drive the shipped binary and HTTP API — R1).

**Acceptance.**

1. `scripts/perf/` harness committed, rerunnable end-to-end by one command
   (`scripts/perf/run-all.sh <outdir>`), each scenario also runnable alone.
2. Baseline doc committed (`docs/perf/baseline-<date>.md`) with every
   matrix cell filled or explicitly marked blocked-with-reason.
3. Idle daemon baseline recorded (RSS, fds, CPU over 2 min).
4. Every confirmed finding filed as a GitHub issue; issue URLs listed in
   the ledger entry. Zero findings silently dropped.
5. Hygiene sweep green after the full run: no orphan processes, no stray
   worktrees, no unaccounted /tmp residue, repo tree clean (harness output
   dirs live outside the tree).

**Unknowns.**

- Container's true core count / cgroup CPU quota — measure and record; it
  bounds what "50 concurrent" means here.
- Whether the loopback HTTP client or the daemon saturates first at 50-way
  concurrency (the client spawns a process per call; `xargs -P`/`&` fan-out
  may be the bottleneck — if so, drive the HTTP API directly with curl for
  the high-concurrency cells and say so in the doc).
- Fake-backend turn latency floor (it is deliberately near-instant; some
  cells may measure the engine, not the backend — fine, name it).

**Gauntlet depth.** Harness build: one Opus builder. Scenarios: Sonnet
runners, strictly sequential. Anomaly verification: batched Opus refuters
(reproduce-or-refute, fresh data dirs). Adjudication, baseline doc, ledger
entry, and issue filing: orchestrator. No fix round exists in this phase by
design.

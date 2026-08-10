# P1-PERF harness

Load, stress and resource baseline for the shipped `sgt` release binary. Drives
the binary and its loopback HTTP API only — no new Rust, no test hooks.

```sh
cargo build --release                 # the unit under test
scripts/perf/run-all.sh /path/outdir  # environment + idle baseline + S1..S7, in order
scripts/perf/s3-deep.sh /path/outdir  # any scenario, alone
```

`<outdir>` must live **outside the repo tree**. Each scenario creates its own
data dir and seed repo under `<outdir>/scratch/`, writes raw CSV/JSON next to
it, prints a distilled summary block, and ends with the hygiene sweep.

| script | scenario |
| --- | --- |
| `idle-baseline.sh` | idle daemon, 2 min sampled (RSS, fds, CPU) |
| `s1-burst.sh` | concurrent submit bursts 1/5/10/20/50 |
| `s2-churn.sh` | 200 works in waves of 10 + leak watch |
| `s3-deep.sh` | one work driven to ≥1,000 events, then graph/show/events load |
| `s4-sse.sh` | 25 SSE subscribers through a burst; 10 killed mid-flight |
| `s5-journal.sh` | grow to 10k/25k/50k events; cold-start rebuild + analytics |
| `s6-crash.sh` | `kill -9` mid-burst ×3, restart, recovery assertions |
| `s7-clients.sh` | TUI under tmux, dashboard, `web`/`doctor`/`status`, orphan SIGTERM |

`common.sh` holds the shared helpers (daemon control, `/proc` sampling, seed
repo, percentiles, hygiene sweep); `_submit-one.sh` and `_get-one.sh` are the
single-call workers `xargs -P` fans out.

## Knobs

Every scenario reads its scale from the environment and documents its own knobs
in its header — e.g. `PERF_S1_BURSTS`, `PERF_S2_TOTAL`, `PERF_S5_MARKS`. Set
them to shrink a run for a smoke test:

```sh
PERF_S1_BURSTS="1 2" PERF_S1_SETTLE=1 scripts/perf/s1-burst.sh /tmp/out/s1
PERF_ONLY="s4 s6" scripts/perf/run-all.sh /tmp/out    # subset of the matrix
```

## What the fake script means

`SGT_FAKE_SCRIPT` is one **global FIFO of steps shared by the whole daemon**,
not a per-work program. One step is consumed per execution START and one per
input SEND; when it runs out, every later step is the default `complete`.
Measured on this build:

```
SGT_FAKE_SCRIPT="needs_input:q1;needs_input:q2;needs_input:q3;complete:a;needs_input:q4;complete:b"
submit    -> stage 1 START pops q1        -> needs_input
respond 1 -> SEND pops q2                 -> needs_input
respond 2 -> SEND pops q3                 -> needs_input
respond 3 -> SEND pops complete:a         -> stage 1 done, stage 2 START pops q4 -> needs_input
respond 4 -> SEND pops complete:b         -> work completed
```

Consequences for scenarios: a scripted multi-cycle flow is only deterministic
with **one work in flight** (S3 runs one work alone for exactly this reason),
and every scenario that wants "everything completes" simply leaves the variable
unset.

Event arithmetic on this build, for sizing a run: a completing two-stage work is
**16 journal events**; a scripted work is 10 events at submit plus **5 per
respond cycle** (so ≥1,000 events ≈ 200 cycles); journal bytes are ~550 per
event.

## Rules the harness obeys

- Measures, never fixes. A defect is a number plus a rerun, not a patch.
- Nothing is written into the repo tree except this directory.
- `perf_hygiene` separates *populated* surface dirs (a worktree that outlived
  its work) from *empty* leftovers, and a worktree held by a non-terminal work
  is expected residue, not a leak — read `hygiene-<label>.txt` before calling
  a sweep dirty.
- A daemon that needs SIGKILL to stop is recorded (`daemon_force_killed`), not
  quietly cleaned up.

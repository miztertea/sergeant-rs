# #45 re-verification — MVP-4 fixer pass (2026-08-12)

The `MVP-4 HARDENING — 2026-08-12` GAUNTLET entry closed #45 citing "40/40
isolated runs and 15/15 full-suite runs, 0 failures" under sustained 16-way
CPU contention, but that claim was narrative-only: the script that produced
it (`stress45.sh`/`stress45_full.sh`, both scratch, not committed — they are
harness-free wrappers around the already-committed test binary, not new
product code, so they stay scratch per the mutation-probe convention, L5)
echoed each run's status to the terminal and nothing captured or committed
that output. Reviewer finding `45-closed-on-preexisting-pin-no-session-code`
called this correctly: the structural pin
(`the_daemon_installs_its_signal_handlers_before_it_publishes_anything`,
`tests/m6_surfaces.rs:3486`) and the behavioral pin
(`the_dropped_spawned_daemon_leaves_the_evidence_of_a_clean_shutdown`,
`tests/m6_surfaces.rs:3417`) are real, committed, revert-probed tests — but
the specific 40/40 and 15/15 *counts* had no artifact a later reader could
check.

This note is that artifact. Same method, re-run fresh in the MVP-4 fixer
pass, output captured this time.

## Method

16-way busy-spin background load (`while :; do :; done`, matching the
original repro method) on this host's 20 cores, for the duration of each
run:

```sh
BIN=$(find target/debug/deps -maxdepth 1 -name 'm6_surfaces-*' -type f -executable | head -1)
# 16 background `while :; do :; done` loops, killed on script exit
"$BIN" the_dropped_spawned_daemon_leaves_the_evidence_of_a_clean_shutdown --exact   # ×40, isolated process each time
"$BIN"                                                                              # ×15, full m6_surfaces suite each time
```

Unit under test: `target/debug/deps/m6_surfaces-9c014a4cb9bab77d`, built from
this branch's HEAD at run time, commit `18eada1b4c148461480ca324f6926e3342ba3045`.

## Result

| run | count | failures |
|---|---|---|
| isolated (`the_dropped_spawned_daemon_leaves_the_evidence_of_a_clean_shutdown --exact`) | 40 | 0 |
| full `m6_surfaces` suite | 15 | 0 |

Full-suite wall time: 1m10.8s for 15 runs (`user` 17m29s across the 20
cores, confirming the load was real, not accidentally idle). Post-run
hygiene: `pgrep -f "debug/sgt [-]-data-dir"` empty (no leaked daemons), no
leftover `while :; do :; done` spin processes.

**Raw artifacts** (not committed — scratch, per this repo's own convention,
same as `docs/perf/s2-churn-mvp1-fixer-2026-08-12.md`'s precedent):
`isolated-40.log`, `full-suite-15.log`, at
`/tmp/claude-1001/-home-miztertea-sergeant-rs/
6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/th45-reverify/` on the host
this ran on; the wrapper scripts themselves
(`stress45.sh`/`stress45_full.sh`) live alongside them in that session's
scratchpad.

## Bottom line

The original closure's substance stands (the fix is real, pinned, and
revert-probed — see the `MVP-4 HARDENING` entry for that argument in full).
What was missing was a checkable number behind "40/40, 15/15", and this
pass supplies exactly that number, freshly measured rather than assumed
still true from an earlier, unrecorded run.

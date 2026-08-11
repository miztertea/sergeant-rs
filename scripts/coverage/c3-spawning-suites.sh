#!/usr/bin/env bash
# C3 — the two suites that drive real `sgt` processes, run last and alone.
#
#   scripts/coverage/c3-spawning-suites.sh
#
# m2 (8 spawning tests) and m6 (7–8) are where subprocess coverage either
# works or silently does not. Everything the instrument needs is already true
# of them — no spawn path calls `env_clear()`, so `LLVM_PROFILE_FILE` reaches
# the client and the detached daemon; the daemon handles SIGTERM and returns
# from `main`; both teardown paths (the `DataDir` reaper and m6's
# `SpawnedDaemon`) are SIGTERM-first as of S1 phase 1 — but "the mechanism is
# in place" is a hypothesis until a profraw delta says so.
#
# Hence the floor below. A test binary writes one profile; every extra one is
# a client or a daemon that flushed. If a stage here produces exactly one, the
# subprocess half of the measurement did not happen and the number that comes
# out of C4 is not the number this program claims.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

cov_stage_begin c3-m2_daemon_api
cov_run cargo llvm-cov --no-report --test m2_daemon_api || cov_fail "m2_daemon_api failed under instrumentation"
cov_stage_end 2 "m2 spawns clients and daemons; more than the test binary's own profile must arrive, \
or no subprocess flushed and the suite's real coverage is missing"

cov_stage_begin c3-m6_surfaces
cov_run cargo llvm-cov --no-report --test m6_surfaces || cov_fail "m6_surfaces failed under instrumentation"
cov_stage_end 2 "m6 spawns daemons (dashboard, doctor) and runs scripts/demo.sh; more than the test \
binary's own profile must arrive, or no subprocess flushed"

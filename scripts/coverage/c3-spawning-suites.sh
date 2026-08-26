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
cov_run cargo llvm-cov --no-report --test m2_daemon_api --locked || cov_fail "m2_daemon_api failed under instrumentation"
cov_stage_end 2 "m2 spawns clients and daemons; more than the test binary's own profile must arrive, \
or no subprocess flushed and the suite's real coverage is missing"

cov_stage_begin c3-m6_surfaces
cov_run cargo llvm-cov --no-report --test m6_surfaces --locked || cov_fail "m6_surfaces failed under instrumentation"
cov_stage_end 2 "m6 spawns daemons (TUI, doctor) and runs scripts/demo.sh; more than the test \
binary's own profile must arrive, or no subprocess flushed"

# Added with H1 W3 (lazy admission + host-scoped verbs): every test in this
# suite spawns a real daemon (and, for the watch tests, a second `sgt watch`
# client left running concurrently) — it belongs beside m2/m6, not in C2's
# no-daemon-of-its-own bucket.
cov_stage_begin c3-w3_client_surface
cov_run cargo llvm-cov --no-report --test w3_client_surface --locked || cov_fail "w3_client_surface failed under instrumentation"
cov_stage_end 2 "w3_client_surface spawns a daemon (and, in the watch tests, a second sgt client) \
in every test; more than the test binary's own profile must arrive, or no subprocess flushed"

# W5, wired at birth: `h1_acceptance_1_and_5` and `h1_acceptance_3` both
# drive real `sgt run` subprocesses, which auto-spawn a real host daemon
# from a bare data dir the same way m2/m6/w3_client_surface's own fixtures
# do — belongs beside them, not in C2.
cov_stage_begin c3-w5_h1_acceptance
cov_run cargo llvm-cov --no-report --test w5_h1_acceptance --locked || cov_fail "w5_h1_acceptance failed under instrumentation"
cov_stage_end 2 "w5_h1_acceptance's two-estate test spawns a real daemon via sgt run; more than \
the test binary's own profile must arrive, or no subprocess flushed"

# S2 V1b, wired at birth: m11's round-trip test spawns a real `sgt work show`
# against the in-process daemon it started, so the CLI's own rendering of a
# composed hierarchical stage id is measured rather than argued. Floor 2 —
# the test binary plus that client.
cov_stage_begin c3-m11_nested_workflow
cov_run cargo llvm-cov --no-report --test m11_nested_workflow --locked || cov_fail "m11_nested_workflow failed under instrumentation"
cov_stage_end 2 "m11's round-trip test spawns a real sgt work show client; more than the test \
binary's own profile must arrive, or no subprocess flushed"

# Added 2026-08-19 alongside C2's five, closing the same accounting gap: this
# suite existed but no stage script invoked it, so backend/docker.rs's real
# coverage never reached the report.
#
# Floor is 1, not 2, and deliberately so. m7 drives a real Docker Engine and
# self-skips with SKIPPED-ENV where Docker is unreachable (docs/DEVELOPMENT.md's
# two-environments rule). On such a host the suite legitimately spawns no
# container and flushes only its own test-binary profile — a floor of 2 would
# read that correct, documented skip as a broken instrument and fail the stage.
# The trade is stated rather than hidden: on a Docker-capable host this stage
# will not catch a subprocess that silently failed to flush.
cov_stage_begin c3-m7_docker_executor
cov_run cargo llvm-cov --no-report --test m7_docker_executor --locked || cov_fail "m7_docker_executor failed under instrumentation"
cov_stage_end 1 "the m7 test binary must write its own profile; container subprocesses are not \
required because the suite self-skips where Docker is unreachable"

#!/usr/bin/env bash
# C1 — the unit tests (`src/**`'s own `#[cfg(test)]` modules), collected.
#
#   scripts/coverage/c1-lib.sh
#
# First stage that runs anything, so it is also the stage that pays the cold
# instrumented build: everything is recompiled with `-C instrument-coverage`
# into target/llvm-cov-target/, including the ~500 C++ translation units of
# bundled DuckDB. Budget accordingly; the wall time is recorded.
#
# `--no-report` is the whole point of the staged shape: it runs the tests and
# leaves the profraws in place instead of merging and reporting. Measured on
# cargo-llvm-cov 0.8.7 — a later `--no-report` stage does **not** remove an
# earlier stage's profraws, so C1…C3 pool into one profdata at C4. Verified,
# not assumed: see scripts/coverage/README.md.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

cov_stage_begin c1

cov_run cargo llvm-cov nextest --no-report --lib --locked || cov_fail "the unit tests failed under instrumentation"

# One test binary ran, so one profraw is the floor. Zero means the binary
# never flushed — a crash, an abort, or a profile pattern pointing somewhere
# this harness is not looking.
cov_stage_end 1 "the --lib test binary must write its own profile"

#!/usr/bin/env bash
# C2 — the four integration suites that spawn no daemon of their own.
#
#   scripts/coverage/c2-suites.sh
#
# Order is the contract's: m1, m4, m3, m5. m1 and m4 are `sgt`-subprocess-free
# outright (m4's children are `sh`/`git` stand-ins, uninstrumented and no
# loss); m3 and m5 each spawn the binary once, which is why they are here and
# not with the spawning-heavy pair in C3 — the S0 challenge corrected an
# earlier claim that had them subprocess-free.
#
# One sub-stage per suite, each with its own accounting and hygiene sweep, so
# a loss can be attributed to the suite that lost it rather than to "the run".
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

cov_stage_begin c2-m1_event_core
cov_run cargo llvm-cov --no-report --test m1_event_core || cov_fail "m1_event_core failed under instrumentation"
cov_stage_end 1 "the m1 test binary must write its own profile"

cov_stage_begin c2-m4_backends
cov_run cargo llvm-cov --no-report --test m4_backends || cov_fail "m4_backends failed under instrumentation"
cov_stage_end 1 "the m4 test binary must write its own profile"

cov_stage_begin c2-m3_execution
cov_run cargo llvm-cov --no-report --test m3_execution || cov_fail "m3_execution failed under instrumentation"
cov_stage_end 1 "the m3 test binary must write its own profile"

cov_stage_begin c2-m5_projections
cov_run cargo llvm-cov --no-report --test m5_projections || cov_fail "m5_projections failed under instrumentation"
cov_stage_end 1 "the m5 test binary must write its own profile"

#!/usr/bin/env bash
# C2 — the integration suites that spawn no daemon of their own.
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
cov_run cargo llvm-cov --no-report --test m1_event_core --locked || cov_fail "m1_event_core failed under instrumentation"
cov_stage_end 1 "the m1 test binary must write its own profile"

cov_stage_begin c2-m4_backends
cov_run cargo llvm-cov --no-report --test m4_backends --locked || cov_fail "m4_backends failed under instrumentation"
cov_stage_end 1 "the m4 test binary must write its own profile"

cov_stage_begin c2-m3_execution
cov_run cargo llvm-cov --no-report --test m3_execution --locked || cov_fail "m3_execution failed under instrumentation"
cov_stage_end 1 "the m3 test binary must write its own profile"

cov_stage_begin c2-m5_projections
cov_run cargo llvm-cov --no-report --test m5_projections --locked || cov_fail "m5_projections failed under instrumentation"
cov_stage_end 1 "the m5 test binary must write its own profile"

# Added 2026-08-19. These four suites existed in tests/ but were invoked by no
# stage script, so their profiles never reached the report: the convention was
# measuring 87.97% where a plain `cargo llvm-cov` (which runs every suite)
# measured 91.09% on identical code. The gap was not missing tests — it was
# missing accounting, concentrated on exactly the files these suites cover
# (the estate CLI verbs, watch.rs, the harness passthrough, the T2/T3 routes).
# m8 and m9 spawn a daemon in a minority of their cases and m10 execs rather
# than spawning, so all four sit here rather than in C3, whose floor rule is
# written for suites dominated by real `sgt` subprocesses.
cov_stage_begin c2-m8_estate_cli
cov_run cargo llvm-cov --no-report --test m8_estate_cli --locked || cov_fail "m8_estate_cli failed under instrumentation"
cov_stage_end 1 "the m8 test binary must write its own profile"

cov_stage_begin c2-m9_watch
cov_run cargo llvm-cov --no-report --test m9_watch --locked || cov_fail "m9_watch failed under instrumentation"
cov_stage_end 1 "the m9 test binary must write its own profile"

cov_stage_begin c2-m10_harness
cov_run cargo llvm-cov --no-report --test m10_harness --locked || cov_fail "m10_harness failed under instrumentation"
cov_stage_end 1 "the m10 test binary must write its own profile"

cov_stage_begin c2-estate_routes
cov_run cargo llvm-cov --no-report --test estate_routes --locked || cov_fail "estate_routes failed under instrumentation"
cov_stage_end 1 "the estate_routes test binary must write its own profile"

cov_stage_begin c2-t2_workflow_catalog
cov_run cargo llvm-cov --no-report --test t2_workflow_catalog --locked || cov_fail "t2_workflow_catalog failed under instrumentation"
cov_stage_end 1 "the t2_workflow_catalog test binary must write its own profile"

# Added 2026-08-22, closing the identical accounting gap the 2026-08-19 block
# above closed for m8/m9/m10/estate_routes/t2_workflow_catalog: these two
# suites existed in tests/ but were invoked by no stage script, so
# backend/codex.rs's and backend/codex_appserver.rs's real coverage never
# reached the report (measured: 40.80%/69.46% lines with them excluded vs.
# 84.74%/94.36% with them included, identical commit, identical binaries).
#
# Both sit in C2, not C3: codex_backend.rs's StubCodex is a shell script
# (uninstrumented, no profile expected of it) — the same shape as m4's own
# sh/git stand-ins, not a real `sgt` subprocess — and its two daemon-level
# registration tests use in-process `daemon::start_with` with no subprocess
# at all, exactly m3_execution's/estate_routes's rig. codex_routing.rs is
# in-process-only throughout (no StubCodex, no subprocess). Floor 1 for both.
cov_stage_begin c2-codex_backend
cov_run cargo llvm-cov --no-report --test codex_backend --locked || cov_fail "codex_backend failed under instrumentation"
cov_stage_end 1 "the codex_backend test binary must write its own profile (StubCodex's children are shell-script stand-ins, uninstrumented and no loss, per m4's own precedent)"

cov_stage_begin c2-codex_routing
cov_run cargo llvm-cov --no-report --test codex_routing --locked || cov_fail "codex_routing failed under instrumentation"
cov_stage_end 1 "the codex_routing test binary must write its own profile"

# Added 2026-08-23, in the same commit that adds the suite (the W1 opencode
# wave) — wired at birth rather than recovered later, per the owner's 90-floor
# ruling and the #231 lesson: a suite absent from every stage list contributes
# nothing to Gate D however green it runs. Sits in C2, not C3, for
# codex_backend's exact reasons: StubOpencode is a shell-script stand-in
# (uninstrumented, no profile expected), and the suite spawns no `sgt`
# subprocess. Floor 1.
cov_stage_begin c2-opencode_backend
cov_run cargo llvm-cov --no-report --test opencode_backend --locked || cov_fail "opencode_backend failed under instrumentation"
cov_stage_end 1 "the opencode_backend test binary must write its own profile (StubOpencode's children are shell-script stand-ins, uninstrumented and no loss, per codex_backend's precedent)"

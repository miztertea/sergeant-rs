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

cov_stage_begin c2-estate_routes
cov_run cargo llvm-cov --no-report --test estate_routes --locked || cov_fail "estate_routes failed under instrumentation"
cov_stage_end 1 "the estate_routes test binary must write its own profile"

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

# Added 2026-08-23, in the same commit that adds the suite (the W1 agy wave) --
# wired at birth rather than recovered later, per the owner's 90-floor ruling
# and the #231 lesson: a suite absent from every stage list contributes nothing
# to Gate D however green it runs. Sits in C2, not C3, for codex_backend's and
# opencode_backend's exact reasons: StubAgy is a shell-script stand-in
# (uninstrumented, no profile expected), and the suite spawns no `sgt`
# subprocess. Floor 1.
cov_stage_begin c2-agy_backend
cov_run cargo llvm-cov --no-report --test agy_backend --locked || cov_fail "agy_backend failed under instrumentation"
cov_stage_end 1 "the agy_backend test binary must write its own profile (StubAgy's children are shell-script stand-ins, uninstrumented and no loss, per codex_backend's precedent)"

# W4c fixer sweep (#231(b)): wired at recovery time — the suite's own
# authorship commit (e01a53fb) left it invoked by neither stage script, the
# exact #231 gap coverage_stage_membership.rs exists to catch. Sits in C2,
# not C3, for codex_backend's/opencode_backend's/agy_backend's exact reasons:
# every `daemon install-service` case exercised here uses an injected fake
# `systemctl`/`launchctl` shell-script stand-in (`write_fake_binary`, this
# suite's own SGT_SYSTEMCTL_BIN precedent) or no external process at all
# (`doctor`, `init`); no case spawns a real `sgt daemon`. Floor 1.
cov_stage_begin c2-w4c_service_doctor
cov_run cargo llvm-cov --no-report --test w4c_service_doctor --locked || cov_fail "w4c_service_doctor failed under instrumentation"
cov_stage_end 1 "the w4c_service_doctor test binary must write its own profile (its fake systemctl/launchctl binaries are shell-script stand-ins, uninstrumented and no loss, per codex_backend's precedent)"

# S2 V1c (build/test speed): the ten suites this stage used to wire one by
# one — m10_harness, t2_workflow_catalog, codex_routing, opencode_routing,
# agy_routing, coverage_stage_membership, docs_contract, w2fix_probe_ordering,
# e_periodic_sweep, w5_cutover_rehearsal — are each small (under 350 lines),
# spawn no daemon of their own (docs_contract's one `sgt --help` call and
# w5_cutover_rehearsal's `sgt doctor`/`install-service --print` calls are the
# only subprocesses among them, both short-lived), and each paid a full
# separate link plus its own copy of `support`'s compile for a handful of
# tests. They now live as `mod`s of one binary, `tests/c2_light.rs`
# (`tests/c2_light/<name>.rs` holds each original file unmodified except for
# `mod support;` → `use crate::support;`, made necessary by no longer being
# its own crate root) — every test stays addressable as
# `cargo test --test c2_light <old_suite_name>::<test>`. One instrumented
# binary, one profile: floor 1. Measured before adopting (v1c baseline,
# knowledge/evidence/perf/build-test-speed-2026-08-26.md): the heavier C2/C3
# suites are deliberately left separate — their own compile time already
# dwarfs their link overhead, so folding them in would add real intra-process
# hygiene risk (thread-shared ports/env across suites that used to be
# separate processes) for a link-time saving measurement showed was already
# small.
cov_stage_begin c2-c2_light
cov_run cargo llvm-cov --no-report --test c2_light --locked || cov_fail "c2_light failed under instrumentation"
cov_stage_end 1 "the c2_light test binary must write its own profile"

# S2 V4 closeout: the W1 §13 acceptance battery, `w5_h1_acceptance.rs`'s own
# precedent one wave later. Almost entirely comment checklist entries
# pointing at m11/m12's own named pins; the one self-contained test
# (criterion 7's merge half) reads source files in-process and spawns
# nothing, so this sits in C2 rather than C3. Floor 1.
cov_stage_begin c2-v4_w1_acceptance
cov_run cargo llvm-cov --no-report --test v4_w1_acceptance --locked || cov_fail "v4_w1_acceptance failed under instrumentation"
cov_stage_end 1 "the v4_w1_acceptance test binary must write its own profile"

# S3 X1: the Atlas substrate's structural suite — the second one-owner
# invariant (`runtime/atlas/db.rs`, held separately from M5's assertion about
# `runtime/analytics.rs`), the module-doc persistence contract, and the schema
# namespaces read back out of a real database file. Source scanning plus one
# tempdir; no daemon, no estate, no backend, so it sits in C2 rather than C3.
# Floor 1.
cov_stage_begin c2-x1_atlas_substrate
cov_run cargo llvm-cov --no-report --test x1_atlas_substrate --locked || cov_fail "x1_atlas_substrate failed under instrumentation"
cov_stage_end 1 "the x1_atlas_substrate test binary must write its own profile"

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

# S2 V2, wired at birth (#231's lesson): m12 drives a real actor process that
# itself invokes the real `sgt` binary against the in-process daemon, so the
# child-Work path is measured through both. Floor 2 — the test binary plus at
# least that client.
cov_stage_begin c3-m12_child_work
cov_run cargo llvm-cov --no-report --test m12_child_work --locked || cov_fail "m12_child_work failed under instrumentation"
cov_stage_end 2 "m12's end-to-end test spawns a real sgt run client from inside a managed \
execution; more than the test binary's own profile must arrive, or no subprocess flushed"

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

# S2 V1d, wired at birth (#310): four of this suite's six tests spawn real
# processes — a bare `sgt daemon` it SIGKILLs mid-probe-walk, and this test
# binary re-executed as a parent whose hardened child has to die with it.
# Belongs beside m2/m6, not in C2's no-daemon-of-its-own bucket. Floor 2: the
# test binary plus at least one spawned daemon. Note that the SIGKILLed
# daemons flush nothing at exit by construction — the profiles that arrive
# here come from the suite's politely-reaped daemons, which is why the floor
# is 2 and not the number of processes it starts.
cov_stage_begin c3-v1d_probe_child_lifecycle
cov_run cargo llvm-cov --no-report --test v1d_probe_child_lifecycle --locked || cov_fail "v1d_probe_child_lifecycle failed under instrumentation"
cov_stage_end 2 "v1d spawns real sgt daemons and re-executes its own test binary; more than the \
test binary's own profile must arrive, or no subprocess flushed"

# S3 X2, wired at birth (the #231 lesson: a suite absent from every stage
# list contributes nothing to Gate D however green it runs). Four of this
# suite's tests drive real `sgt` client processes (`knowledge add`/`list`,
# `repo add`, `init`), and two more start a real in-process daemon over a
# data dir. Belongs beside m2/m6 rather than in C2's no-daemon-of-its-own
# bucket. Floor 2: the test binary plus at least one flushed `sgt` client.
cov_stage_begin c3-x2_knowledge_sources
cov_run cargo llvm-cov --no-report --test x2_knowledge_sources --locked || cov_fail "x2_knowledge_sources failed under instrumentation"
cov_stage_end 2 "x2_knowledge_sources spawns real sgt clients and starts real daemons; more than \
the test binary's own profile must arrive, or no subprocess flushed"

# S4 Y1, wired at birth (the #231 lesson, same as m12/x2 above): every test
# in this suite spawns a real `sgt-atlas-worker` subprocess — a second real
# `[[bin]]` target this package builds, not a shell-script stand-in — so it
# belongs beside the other real-process suites, not in C2's no-subprocess
# bucket. Floor 2, not higher: the fault-injection cases (abort/hang/
# allocate) are killed by SIGABRT or SIGKILL by design and legitimately flush
# no profile of their own (the same fact v1d_probe_child_lifecycle's own
# comment above states for a SIGKILLed daemon), but the happy-path and
# batch-refusal cases spawn a worker that exits 0 normally and must flush —
# more than the test binary's own profile must arrive, or no subprocess
# flushed.
cov_stage_begin c3-y1_worker_transport
cov_run cargo llvm-cov --no-report --test y1_worker_transport --locked || cov_fail "y1_worker_transport failed under instrumentation"
cov_stage_end 2 "y1_worker_transport spawns real sgt-atlas-worker subprocesses in its happy-path \
and batch-refusal cases; more than the test binary's own profile must arrive, or no subprocess \
flushed (the abort/hang/allocate fault cases are expected to flush nothing, by the same logic \
v1d's SIGKILLed daemons do not)"

# S4 Y2, wired at birth (the #231 lesson, same as y1 above): the real-parser
# supervision proof, through the real `sgt-atlas-worker` subprocess and the
# real Office adapter — the happy path and every malformed/hostile fixture
# case. Unlike y1's fault-injection modes, none of these cases are SIGKILLed
# or SIGABRTed: a bad document is refused by an ordinary non-zero exit, so
# every case here is expected to flush a subprocess profile. Floor 2, not
# higher, for the same reason y1's is: the test binary's own profile plus at
# least one flushed worker.
cov_stage_begin c3-y2_office_adapter
cov_run cargo llvm-cov --no-report --test y2_office_adapter --locked || cov_fail "y2_office_adapter failed under instrumentation"
cov_stage_end 2 "y2_office_adapter spawns real sgt-atlas-worker subprocesses in every case \
(the happy path and both real-parser failure fixtures all exit normally, none are signalled); \
more than the test binary's own profile must arrive, or no subprocess flushed"

# S4 Y3, wired at birth (the #231 lesson, same as y1/y2 above): the bounded-
# ZIP adapter, through the real sgt-atlas-worker subprocess — the happy path,
# a nested-archive case, and the archive-level-refusal case (the entry-count
# ceiling). All three exit normally (none are signalled), so every case is
# expected to flush a subprocess profile. Floor 2, not higher, for the same
# reason y1/y2's is: the test binary's own profile plus at least one flushed
# worker.
cov_stage_begin c3-y3_zip_adapter
cov_run cargo llvm-cov --no-report --test y3_zip_adapter --locked || cov_fail "y3_zip_adapter failed under instrumentation"
cov_stage_end 2 "y3_zip_adapter spawns real sgt-atlas-worker subprocesses in every case (the \
happy path, the nested-archive case, and the archive-level-refusal case all exit normally, none \
are signalled); more than the test binary's own profile must arrive, or no subprocess flushed"

# S4 Y4, wired at birth (the #231 lesson, same as y1/y2/y3 above): the mail
# adapter, through the real sgt-atlas-worker subprocess — the happy path, the
# genuine-vs-synthesized HTML case, and every honest-refusal case (unparseable,
# degraded, sealed). All exit normally (none are signalled), so every case is
# expected to flush a subprocess profile. Floor 2, not higher, for the same
# reason y1/y2/y3's is: the test binary's own profile plus at least one
# flushed worker.
cov_stage_begin c3-y4_mail_adapter
cov_run cargo llvm-cov --no-report --test y4_mail_adapter --locked || cov_fail "y4_mail_adapter failed under instrumentation"
cov_stage_end 2 "y4_mail_adapter spawns real sgt-atlas-worker subprocesses in every case (the \
happy path, the genuine-vs-synthesized-HTML case, and every honest-refusal case all exit \
normally, none are signalled); more than the test binary's own profile must arrive, or no \
subprocess flushed"

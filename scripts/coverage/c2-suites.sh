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

# S3 X3a: the mechanical estate-git plumbing — a tree walk at an
# admission-pinned SHA, batched blob reads, the concurrent-HEAD-advance rule,
# Work-overlay hashing, and F6's intelligence-lane permits. Real `git`
# subprocesses over tempdir repositories plus an in-process Atlas store and a
# tokio runtime; no daemon, no estate root, no backend, so C2 rather than C3.
# Floor 1.
cov_stage_begin c2-x3a_git_plumbing
cov_run cargo llvm-cov --no-report --test x3a_git_plumbing --locked || cov_fail "x3a_git_plumbing failed under instrumentation"
cov_stage_end 1 "the x3a_git_plumbing test binary must write its own profile"

# S3 X3a, the read-only claim: one test in its own process (it sets the
# process-global `SGT_GIT_BIN` override to a recording shim), asserting that a
# scan runs only read-only Git verbs and leaves the mount byte-identical.
# Floor 1.
cov_stage_begin c2-x3a_scan_uses_only_local_reads
cov_run cargo llvm-cov --no-report --test x3a_scan_uses_only_local_reads --locked || cov_fail "x3a_scan_uses_only_local_reads failed under instrumentation"
cov_stage_end 1 "the x3a_scan_uses_only_local_reads test binary must write its own profile"

# S3 X3b: the F5 corpus gate — the hand-verified multi-language fixture corpus
# read through `runtime::atlas::syntax`'s pure extractor. Reads checked-in
# fixture bytes and nothing else: no tempdir, no subprocess, no daemon, no
# estate, no backend. Floor 1.
cov_stage_begin c2-x3b_tslp_corpus
cov_run cargo llvm-cov --no-report --test x3b_tslp_corpus --locked || cov_fail "x3b_tslp_corpus failed under instrumentation"
cov_stage_end 1 "the x3b_tslp_corpus test binary must write its own profile"

# S3 X3b, the wiring: symbols/occurrences/edges written by the ordinary
# recording path over real Git objects, plus the intelligence-lane consumer.
# Builds throwaway repositories and Atlas stores in tempdirs and drives an
# in-process Engine; it spawns no `sgt` and starts no daemon, which is why it
# is here rather than with the spawning suites. Floor 1.
cov_stage_begin c2-x3b_syntax_wiring
cov_run cargo llvm-cov --no-report --test x3b_syntax_wiring --locked || cov_fail "x3b_syntax_wiring failed under instrumentation"
cov_stage_end 1 "the x3b_syntax_wiring test binary must write its own profile"

# S3 X4: tabular datasets read in place, F4's network refusal, F10a's column
# allowlist, F12's bounds. Builds knowledge directories and Atlas stores in
# tempdirs and reads them through the ordinary recording path. It starts no
# daemon and its only subprocess is `sgt --help` (F11's named-deferral pin),
# which is why it is here rather than with the spawning suites. Floor 1.
cov_stage_begin c2-x4_tabular_map
cov_run cargo llvm-cov --no-report --test x4_tabular_map --locked || cov_fail "x4_tabular_map failed under instrumentation"
cov_stage_end 1 "the x4_tabular_map test binary must write its own profile"

# S3 X5: the A1a acceptance battery — the §17 walk, its own citation and
# doc-table guards, and the three checks written where an item had none. Reads
# this repository's own sources, drives one real knowledge scan in a tempdir,
# and spawns `sgt` only for manifest-direct verbs and `--help` (no daemon),
# which is why it is here rather than with the spawning suites. Floor 1.
cov_stage_begin c2-x5_a1a_acceptance
cov_run cargo llvm-cov --no-report --test x5_a1a_acceptance --locked || cov_fail "x5_a1a_acceptance failed under instrumentation"
cov_stage_end 1 "the x5_a1a_acceptance test binary must write its own profile"

# S4 Y2, wired at birth (the #231 lesson, same as x1/x5 above): the
# replaceability boundary's structural pin. A token scan of this repository's
# own `.rs` sources under `src/`/`tests/` — no daemon, no estate, no
# subprocess of any kind — which is why it is here rather than with the
# spawning suites. Floor 1.
cov_stage_begin c2-y2_office_boundary
cov_run cargo llvm-cov --no-report --test y2_office_boundary --locked || cov_fail "y2_office_boundary failed under instrumentation"
cov_stage_end 1 "the y2_office_boundary test binary must write its own profile"

# S4 Y5, wired at birth (the #231 lesson, same as x5/y2 above): the scan
# trigger and external-git acquisition's HTTP surface, driven against a real
# in-process daemon (`daemon::start_with`, the identical shape
# e_admission_uses_no_network_git.rs and x5_a1a_acceptance.rs already use) —
# no separate daemon PROCESS, so no second profile to expect. `sgt` itself is
# spawned only for `--help`. Floor 1.
cov_stage_begin c2-y5_external_git_triggers
cov_run cargo llvm-cov --no-report --test y5_external_git_triggers --locked || cov_fail "y5_external_git_triggers failed under instrumentation"
cov_stage_end 1 "the y5_external_git_triggers test binary must write its own profile"

# S4 Y6, wired at birth (the #231 lesson, same as x5/y2/y5 above): the
# estate-scoped scan trigger (Y6a — a real [[repo]] mount, git-plumbing
# only, no clone/fetch) and the online-only heuristic (Y6b), both driven
# against a real in-process daemon (the identical y5_external_git_triggers
# shape) — no separate daemon PROCESS, so no second profile to expect.
# `sgt` itself is spawned only for `--help`. Floor 1.
cov_stage_begin c2-y6a_estate_scoped_scan
cov_run cargo llvm-cov --no-report --test y6a_estate_scoped_scan --locked || cov_fail "y6a_estate_scoped_scan failed under instrumentation"
cov_stage_end 1 "the y6a_estate_scoped_scan test binary must write its own profile"

cov_stage_begin c2-y6b_online_only
cov_run cargo llvm-cov --no-report --test y6b_online_only --locked || cov_fail "y6b_online_only failed under instrumentation"
cov_stage_end 1 "the y6b_online_only test binary must write its own profile"

# S4 Y5's doctrine-amendment pin (G6): a token scan of `AGENTS.md` and
# `src/runtime/surface.rs` — no daemon, no estate, no subprocess of any kind.
# Floor 1.
cov_stage_begin c2-y5_doctrine_never_fetches_is_scoped
cov_run cargo llvm-cov --no-report --test y5_doctrine_never_fetches_is_scoped --locked || cov_fail "y5_doctrine_never_fetches_is_scoped failed under instrumentation"
cov_stage_end 1 "the y5_doctrine_never_fetches_is_scoped test binary must write its own profile"

# S5 W1, wired at birth (the #231 lesson, same as x5/y2/y5/y6 above): A2 §2's
# deterministic admissibility filter — the four bounded canned queries and
# their negative-admission proofs, plus the structural pins on H13.1's
# extractor vocabulary. Builds Atlas stores in tempdirs and records scans
# through the ordinary `record_scan` path, and runs one real
# `scan_local_knowledge` walk for the `--content config` live check. No
# daemon, no estate, no subprocess of any kind — which is why it is here
# rather than with the spawning suites. Floor 1.
cov_stage_begin c2-w1_deterministic_filter
cov_run cargo llvm-cov --no-report --test w1_deterministic_filter --locked || cov_fail "w1_deterministic_filter failed under instrumentation"
cov_stage_end 1 "the w1_deterministic_filter test binary must write its own profile"

# S5 W1b, wired at birth (the #231 lesson, same as x5/y2/y5/y6/w1 above):
# the Work-overlay lifecycle trigger — the daemon-side hook that finally
# gives `scan_work_overlay_on_lane`/`evict_work_overlays` a production
# caller. Starts an IN-PROCESS daemon (the y6a shape) and drives it over
# loopback, plus real read-only `git` invocations to build a mount and a
# linked worktree; it spawns no `sgt` client and no detached daemon, so its
# profile is the test binary's own. Floor 1.
cov_stage_begin c2-w1b_overlay_lifecycle_trigger
cov_run cargo llvm-cov --no-report --test w1b_overlay_lifecycle_trigger --locked || cov_fail "w1b_overlay_lifecycle_trigger failed under instrumentation"
cov_stage_end 1 "the w1b_overlay_lifecycle_trigger test binary must write its own profile"

# S5 W1c, wired at birth (the #231 lesson, same as x5/y2/y5/y6/w1/w1b above):
# A1 §5's one Atlas database — the cross-schema ops<->source join the
# contract cites as its reason for one file, and the honest restatement of
# what deleting that file now costs. Non-spawning: it folds `ops` in-process
# and records scans through the ordinary `record_scan` path. No daemon, no
# estate, no subprocess. Floor 1.
cov_stage_begin c2-w1c_one_atlas_database
cov_run cargo llvm-cov --no-report --test w1c_one_atlas_database --locked || cov_fail "w1c_one_atlas_database failed under instrumentation"
cov_stage_end 1 "the w1c_one_atlas_database test binary must write its own profile"

# S5 W1d, wired at birth (the #231 lesson, same as x5/y2/y5/y6/w1/w1b/w1c
# above): `--work` reflects what the Work has CHANGED — the turn-boundary
# overlay refresh, without which the only overlay a bind can record is one
# describing a worktree still byte-identical to its base. Same shape as
# w1b's suite: an IN-PROCESS daemon driven over loopback plus real
# read-only `git` invocations; no `sgt` client, no detached daemon, so its
# profile is the test binary's own. Floor 1.
cov_stage_begin c2-w1d_overlay_freshness
cov_run cargo llvm-cov --no-report --test w1d_overlay_freshness --locked || cov_fail "w1d_overlay_freshness failed under instrumentation"
cov_stage_end 1 "the w1d_overlay_freshness test binary must write its own profile"

# #258 thread-budget contract, wired at birth (the #231 lesson, same as the
# suites above): the structural pin for `.config/nextest.toml`'s
# `threads-required` override on m7's heavy test. It reads the config file and
# the test tree and asserts the override still exists in the form the config's
# own comment describes — no daemon, no subprocess, so its profile is the test
# binary's own. NOT allowlisted: the allowlist is for `#[ignore]`d measurement
# suites that need a developer's own corpus, and this one runs normally. Floor 1.
cov_stage_begin c2-f258_nextest_thread_budget
cov_run cargo llvm-cov --no-report --test f258_nextest_thread_budget --locked || cov_fail "f258_nextest_thread_budget failed under instrumentation"
cov_stage_end 1 "the f258_nextest_thread_budget test binary must write its own profile"
# S5 W2, wired at birth (the #231 lesson, same as x5/y2/y5/y6/w1/w1b/w1c/w1d
# above): A2 §5's lexical retrieval — the BM25 index over A1's existing
# evidence units, its four unit families with A1 provenance (§17 item 2), and
# the negative A2 §8 turns on (an inadmissible unit with a perfect lexical
# match is never returned). Builds Atlas stores in tempdirs, runs one real
# `scan_local_knowledge` walk for the code/document/row-text families, and
# records the mail fixture through the ordinary `record_scan` path. No daemon,
# no estate, no subprocess of any kind — the `.eml` worker is deliberately not
# spawned (that is y4_mail_adapter's job), which is why this sits here rather
# than with the spawning suites. Floor 1.
cov_stage_begin c2-w2_lexical_retrieval
cov_run cargo llvm-cov --no-report --test w2_lexical_retrieval --locked || cov_fail "w2_lexical_retrieval failed under instrumentation"
cov_stage_end 1 "the w2_lexical_retrieval test binary must write its own profile"
# S5 W3, wired at birth (the #231 lesson, same as x5/y2/y5/y6/w1/w1b/w1c/w1d/w2
# above): decision H4's degraded-honesty field — A2 §15's "reports that
# coverage/capability honestly" as a REQUIRED `semantic: applied |
# not_installed | disabled` on every search answer, distinct from A2 §13's
# optional model-identity field. Proves a no-model run still answers through
# the lexical half and says `not_installed`, that `disabled` and
# `not_installed` stay distinguishable when the optional field alone is not,
# and that the field cannot become optional or defaulted. It deliberately
# covers only the DEGRADED half — every test in it removes
# $SGT_SEMANTIC_MODEL_DIR first, so it describes a build with the model2vec
# dependency present and the assets absent (a `cargo install` from source).
# The `applied` case is W3b's `w3b_semantic_retrieval`, below. Builds one
# Atlas store in a tempdir; no daemon, no estate, no subprocess. Floor 1.
cov_stage_begin c2-w3_semantic_degradation
cov_run cargo llvm-cov --no-report --test w3_semantic_degradation --locked || cov_fail "w3_semantic_degradation failed under instrumentation"
cov_stage_end 1 "the w3_semantic_degradation test binary must write its own profile"

# S5 W3b. Two suites, added at birth (#231).
#
# `w3b_semantic_retrieval` is F5 gate 2 — the hand-verified fixture corpus,
# A2 §8's non-vacuous negative, the tie-break determinism pin, and the
# degraded/suppressed paths. It loads the committed 32 MB model once per
# test process, so it is slower than its neighbours; it still spawns no
# daemon and no subprocess. Floor 1.
#
# `w3b_model2vec_manifest_pin` is A2-12's structural pin: it reads
# Cargo.toml, Cargo.lock and deny.toml as data. It executes almost no
# product code by design, so it is here for completeness of accounting
# rather than for coverage — a suite invoked by no stage script is exactly
# the accounting gap the 2026-08-19 and 2026-08-22 blocks above closed.
# Floor 1.
#
# NOT wired here, deliberately: `w3b_semantic_scan_measurement` is
# `#[ignore]`d and is a measurement, not a gate — the same treatment
# w1d_overlay_scan_measurement, w2_startup_measurement and
# w3_prune_measurement get.
cov_stage_begin c2-w3b_semantic_retrieval
cov_run cargo llvm-cov --no-report --test w3b_semantic_retrieval --locked || cov_fail "w3b_semantic_retrieval failed under instrumentation"
cov_stage_end 1 "the w3b_semantic_retrieval test binary must write its own profile"

cov_stage_begin c2-w3b_model2vec_manifest_pin
cov_run cargo llvm-cov --no-report --test w3b_model2vec_manifest_pin --locked || cov_fail "w3b_model2vec_manifest_pin failed under instrumentation"
cov_stage_end 1 "the w3b_model2vec_manifest_pin test binary must write its own profile"

# S5 W4, wired at birth (the #231 lesson, same as w1/w1b/w1c/w1d/w2/w3/w3b
# above): A2 §7's Reciprocal Rank Fusion and A2 §8's deterministic
# reranking — the one RRF expression, the four determinism hazards with a
# rule and a test each, all nine of A2 §8's signals proved to actually fire,
# and the A2 §8 prohibition proved a third time (fusion is where a second
# list could smuggle an inadmissible candidate in). Loads the committed
# model once per test process, like w3b_semantic_retrieval; builds Atlas
# stores in tempdirs and records scans through the ordinary `record_scan`
# path. No daemon, no estate, no subprocess of any kind. Floor 1.
cov_stage_begin c2-w4_rrf_fusion
cov_run cargo llvm-cov --no-report --test w4_rrf_fusion --locked || cov_fail "w4_rrf_fusion failed under instrumentation"
cov_stage_end 1 "the w4_rrf_fusion test binary must write its own profile"

# S5 W5, wired at birth (the #231 lesson, same as w1/w1b/w1c/w1d/w2/w3/w3b/w4
# above): A2 §13's search trace, A2 §14's two verbs (`sgt search` AND `sgt
# related`), and the three A2 §17 items the mid-sprint acceptance walk found
# unassigned — item 8 (external evidence visibly external from the answer
# alone), item 3 (a relational aggregate joined to retrieved row evidence)
# and item 6 (one query spanning a normalized Office document and a Markdown
# one). Builds Atlas stores in tempdirs and records scans through the
# ordinary `record_scan` path; item 6 spawns the REAL `sgt-atlas-worker`
# subprocess for the `.docx` half, the way y2_office_adapter and
# w7_container_children do, because a hand-built office unit would assume the
# half item 6 is actually about. No daemon and no estate. Floor 1.
cov_stage_begin c2-w5_search_surface
cov_run cargo llvm-cov --no-report --test w5_search_surface --locked || cov_fail "w5_search_surface failed under instrumentation"
cov_stage_end 1 "the w5_search_surface test binary must write its own profile"

# S6 C1a — C1 §3's compilation step and §5's enforceable runtime order.
# Fourteen in-process tests over a real Atlas built by the ordinary
# `record_scan` path (the compiler, the §5 gate, §15's snapshot), plus three
# live-daemon tests over a real estate for §21 items 1 and 13 and for the
# actor-only scoping of the step. Floor 1.
cov_stage_begin c2-c1a_compiled_context
cov_run cargo llvm-cov --no-report --test c1a_compiled_context --locked || cov_fail "c1a_compiled_context failed under instrumentation"
cov_stage_end 1 "the c1a_compiled_context test binary must write its own profile"

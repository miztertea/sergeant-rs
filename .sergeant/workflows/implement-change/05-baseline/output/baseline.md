# 05-baseline — fold-the-deadline-loops

Pinned revision: `155a371caf233a3bd1166c363115bf7d6ab4610a` (confirmed via
`git rev-parse HEAD`; lane clean before this stage: `git status
--porcelain=v1 --branch` showed only `ahead 16` of `origin/main`, no
tracked/untracked diffs). No build was already running (`pgrep -fa
'[r]ustc'` returned nothing) before the baseline compile below.

## Discovered test command

`@@bounded-judgment` J2 (discovering the test command): the guard the wave
must leave green is named explicitly by the brief's own Close section and
by orientation — `tests/s6_no_clock_decides_correctness.rs`, run via
nextest:

```
cd /var/tmp/hats7/noclock && \
  TMPDIR=/var/tmp/sgt-test-tmp CARGO_TARGET_DIR=/var/tmp/hats7/noclock/target \
  CARGO_BUILD_JOBS=6 cargo nextest run --locked --test s6_no_clock_decides_correctness
```

This is the narrowest decisive command for this wave: the guard test is
the machine-checkable definition of "every hand-rolled deadline loop is
folded or named in the residue," and it is the one command the brief's
Close section names first. The full suite and the two starve-mode runs
are explicitly reserved for the *close* gate (brief: "Full suite once…
nothing else building", the `taskset` starve runs), not for this baseline
— running them here would duplicate the heaviest load on this host for a
state (nothing changed yet) that this one command already fully
characterizes. Per-file suites (`cargo nextest run --locked --test
<file>`) are the brief's own per-commit gate (step 3) and will be run and
recorded individually as each file is folded in the implement stage, not
en masse here.

## Baseline run — real output, verbatim

Captured to `/var/tmp/hats7/baseline-s6-no-clock.txt`; reproduced in full:

```
   Compiling sergeant-rs v0.3.0 (/var/tmp/hats7/noclock)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
────────────
 Nextest run ID 5267cc16-836f-4504-9cf6-35e7249f49ef with nextest profile: default
    Starting 9 tests across 1 binary
        PASS [   0.004s] sergeant-rs::s6_no_clock_decides_correctness the_sleep_guard_fails_on_a_bare_sleep_with_no_loop_and_no_allowlist_entry
        PASS [   0.004s] sergeant-rs::s6_no_clock_decides_correctness the_guard_fails_on_a_real_deadline_decides_a_verdict_construct_with_no_allowlist_entry
        PASS [   0.004s] sergeant-rs::s6_no_clock_decides_correctness the_deadline_guard_fails_on_a_real_hand_rolled_loop_with_no_allowlist_entry
        PASS [   0.004s] sergeant-rs::s6_no_clock_decides_correctness the_sleep_guard_fails_on_a_state_blind_loop_even_though_it_is_lexically_a_loop
        PASS [   0.005s] sergeant-rs::s6_no_clock_decides_correctness the_timeout_guard_fails_on_an_unallowlisted_client_timeout
        PASS [   0.082s] sergeant-rs::s6_no_clock_decides_correctness every_time_construct_in_the_crate_sits_on_an_explicit_allowlist
        PASS [   0.892s] sergeant-rs::s6_no_clock_decides_correctness every_hand_rolled_deadline_in_tests_is_folded_or_named_in_the_residue
        PASS [   0.896s] sergeant-rs::s6_no_clock_decides_correctness every_sleep_in_tests_sits_inside_a_terminating_loop_or_an_explicit_allowlist
        PASS [   0.904s] sergeant-rs::s6_no_clock_decides_correctness every_client_timeout_in_tests_sits_on_an_explicit_allowlist
────────────
     Summary [   0.905s] 9 tests run: 9 passed, 0 skipped
```

All 9 tests pass, 0 skipped, 0 failed. This is the real, re-runnable
baseline state — not an assumed "tests currently pass."

## Current allowlist state (measured, not assumed)

- `deadline-loop-residue` entries in the `ALLOWLIST` array: **54**
  (counted structurally — parsed each `Allowed { … }` block and filtered
  on `category: "deadline-loop-residue"`, not a flat string grep, since
  the literal needle `deadline-loop-residue` also appears in the two
  constant doc comments and would over-count a plain `grep -c` by 1: a
  plain `grep -c deadline-loop-residue tests/s6_no_clock_decides_correctness.rs`
  returns 55, not 54).
- Distinct files those 54 entries name: **20** (`tests/a4_blob_ref_pinning.rs`,
  `tests/agy_backend.rs`, `tests/c2_light/m10_harness.rs`,
  `tests/codex_backend.rs`, `tests/m2_daemon_api.rs`, `tests/m3_execution.rs`,
  `tests/m4_backends.rs`, `tests/m5_projections.rs`, `tests/m6_surfaces.rs`,
  `tests/m7_docker_executor.rs`, `tests/m8_estate_cli.rs`, `tests/m9_watch.rs`,
  `tests/opencode_backend.rs`, `tests/support/mod.rs`,
  `tests/v1d_probe_child_lifecycle.rs`, `tests/w3_client_surface.rs`,
  `tests/w4_read_surfaces.rs`, `tests/y2_office_adapter.rs`,
  `tests/y3_zip_adapter.rs`, `tests/y4_mail_adapter.rs`) — matches
  orientation's independently-derived 20-file figure exactly (the brief's
  own prose says 21; orientation already flagged that as off-by-one and
  this recount confirms 20, not 21).
- Actual hand-rolled site count these 54 needles cover is **107**
  (orientation's captured red-run number from the guard's own matcher
  against an empty allowlist, minus the 3 kept `owned-wait-budget`
  entries) — not independently re-derived here, since reproducing it
  requires the temporary probe-and-revert orientation already performed
  and recorded; re-running that destructive probe again here would only
  restate evidence orientation already captured verbatim, not add any.
- `DEADLINE_LOOP_RESIDUE_REASON` constant: present, `tests/s6_no_clock_decides_correctness.rs:210-215` — must be deleted when the last `deadline-loop-residue` entry is folded.
- `scripts/coverage/c2-suites.sh`: confirmed `s6_scan_front_door`
  (`:496-498`) and `s6_scan_poll_survives_a_transport_timeout`
  (`:513-515`) are wired; `s6_scan_answers_while_embedding` has no
  `cov_stage_begin`/`cov_run`/`cov_stage_end` block anywhere in the file
  (`grep -n s6_scan_answers_while_embedding scripts/coverage/c2-suites.sh`
  → no match) — the gap orientation and the brief's step 4 both name.
- The 3 `owned-wait-budget` entries that stay untouched
  (`tests/c4_repo_lock.rs` ×2, `tests/y1_worker_transport.rs` ×1) are
  present in the allowlist and are out of this wave's scope per the brief.

## What this change is expected to move

**Must newly become true** (currently false/absent):
- `deadline-loop-residue` entry count in
  `tests/s6_no_clock_decides_correctness.rs` reaches **0**
  (`grep -c deadline-loop-residue tests/s6_no_clock_decides_correctness.rs`
  → 0; currently 55 raw / 54 structural).
- `DEADLINE_LOOP_RESIDUE_REASON` constant is deleted once its last
  referencing entry is folded (currently present).
- All 107 sites the 54 needles cover call `support::wait_until` /
  `support::wait_until_sync` bounded by `support::HANG_BUDGET`, each with
  a `what` naming the specific state being waited on (currently: 107
  hand-rolled `Instant::now() + <duration>` loops, ad hoc per-site
  budgets).
- `s6_scan_answers_while_embedding` gets a `cov_stage_begin` /
  `cov_run cargo llvm-cov … --test s6_scan_answers_while_embedding` /
  `cov_stage_end` block in `scripts/coverage/c2-suites.sh` alongside its
  two `s6_scan_*` siblings (currently absent).

**Must remain true** (currently passing, must not regress):
- All 9 tests in `tests/s6_no_clock_decides_correctness.rs` stay green
  throughout, including the 3 tests that assert allowlist coverage of
  every real construct in the crate (`every_time_construct_in_the_crate_sits_on_an_explicit_allowlist`,
  `every_hand_rolled_deadline_in_tests_is_folded_or_named_in_the_residue`,
  `every_sleep_in_tests_sits_inside_a_terminating_loop_or_an_explicit_allowlist`,
  `every_client_timeout_in_tests_sits_on_an_explicit_allowlist`) — these
  are the guard's own correctness-of-coverage checks and must not be
  weakened while entries are removed.
- The 3 `owned-wait-budget` entries stay untouched and their tests
  (`c4_repo_lock.rs`, `y1_worker_transport.rs`) keep passing unmodified.
- Each folded test file's own suite (`cargo nextest run --locked --test
  <file>`) stays green per-file as it is folded — behavior-preserving
  except for the budget, per the brief.
- No `src/` file changes (this wave is `tests/`-only per the brief and
  the J5 ruling's own scope).

## Rungs cited

- **R2** (already in this codebase): `support::wait_until`,
  `support::wait_until_sync`, `support::HANG_BUDGET` already exist
  (`tests/support/mod.rs:508,529,486` per orientation) — this wave reuses
  them, builds nothing new.
- **J3** (settled authoritative record): the discovered test command and
  its scope (guard-only here; full suite + starve runs reserved for
  close) come directly from the brief's own Close section, an accepted
  upstream artifact for this Work — not invented here.
- **J1** (local, reversible): exact wording/formatting of this baseline
  record.

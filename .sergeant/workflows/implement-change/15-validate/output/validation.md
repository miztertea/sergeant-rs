# 15-validate — fold-the-deadline-loops

Lane clean before this stage (`git status --short` empty), no build
capacity conflict (`pgrep -fa '[r]ustc'` returned nothing), on
`a6557f7e` (21 commits ahead of the `675a93ea` baseline). No `src/`
changes anywhere in the wave.

## Baseline command re-run — real output, verbatim

Same command `05-baseline` discovered and ran, executed unchanged against
the completed implementation:

```
cd /var/tmp/hats7/noclock && \
  TMPDIR=/var/tmp/sgt-test-tmp CARGO_TARGET_DIR=/var/tmp/hats7/noclock/target \
  CARGO_BUILD_JOBS=6 cargo nextest run --locked --test s6_no_clock_decides_correctness
```

Captured to `/var/tmp/hats7/validate-s6-no-clock.txt`; reproduced in full:

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
────────────
 Nextest run ID 63f2af5f-8064-464b-a67a-e17ba9387e7a with nextest profile: default
    Starting 9 tests across 1 binary
        PASS [   0.003s] sergeant-rs::s6_no_clock_decides_correctness the_deadline_guard_fails_on_a_real_hand_rolled_loop_with_no_allowlist_entry
        PASS [   0.003s] sergeant-rs::s6_no_clock_decides_correctness the_sleep_guard_fails_on_a_state_blind_loop_even_though_it_is_lexically_a_loop
        PASS [   0.003s] sergeant-rs::s6_no_clock_decides_correctness the_sleep_guard_fails_on_a_bare_sleep_with_no_loop_and_no_allowlist_entry
        PASS [   0.003s] sergeant-rs::s6_no_clock_decides_correctness the_timeout_guard_fails_on_an_unallowlisted_client_timeout
        PASS [   0.003s] sergeant-rs::s6_no_clock_decides_correctness the_guard_fails_on_a_real_deadline_decides_a_verdict_construct_with_no_allowlist_entry
        PASS [   0.080s] sergeant-rs::s6_no_clock_decides_correctness every_time_construct_in_the_crate_sits_on_an_explicit_allowlist
        PASS [   0.910s] sergeant-rs::s6_no_clock_decides_correctness every_client_timeout_in_tests_sits_on_an_explicit_allowlist
        PASS [   0.910s] sergeant-rs::s6_no_clock_decides_correctness every_hand_rolled_deadline_in_tests_is_folded_or_named_in_the_residue
        PASS [   0.912s] sergeant-rs::s6_no_clock_decides_correctness every_sleep_in_tests_sits_inside_a_terminating_loop_or_an_explicit_allowlist
────────────
     Summary [   0.912s] 9 tests run: 9 passed, 0 skipped
```

**9 tests run, 9 passed, 0 skipped, 0 failed — identical to the baseline
count.** `every_hand_rolled_deadline_in_tests_is_folded_or_named_in_the_residue`
(the guard the wave exists to leave green with a shrunk allowlist) passes.
This is a real pass, not a worked-around one; the `wait_until`/
`wait_until_sync` folds and the `owned-wait-budget` allowlist entries this
stage's implementation added are exactly what this test now walks.

## The four close-out items

### 1. New suite wired into coverage (#231)

`scripts/coverage/c2-suites.sh` carries a stage block for the previously
unwired suite:

```
$ grep -rn "s6_scan_answers_while_embedding" scripts/
scripts/coverage/c2-suites.sh:521:cov_stage_begin c2-s6_scan_answers_while_embedding
scripts/coverage/c2-suites.sh:522:cov_run cargo llvm-cov --no-report --test s6_scan_answers_while_embedding --locked || cov_fail "s6_scan_answers_while_embedding failed under instrumentation"
scripts/coverage/c2-suites.sh:523:cov_stage_end 1 "the s6_scan_answers_while_embedding test binary must write its own profile"
```

Done, evidence shown.

### 2. `cargo fmt` and `cargo clippy --all-targets`

`cargo fmt --check` (`CARGO_BUILD_JOBS=6`): clean, exit 0, no output.

`cargo clippy --all-targets` (`TMPDIR=/var/tmp/sgt-test-tmp
CARGO_TARGET_DIR=/var/tmp/hats7/noclock/target CARGO_BUILD_JOBS=6`):
**not clean** — 2 warnings, full output in
`/var/tmp/hats7/validate-clippy.txt`:

- `tests/codex_backend.rs:3223` — `clippy::let_and_return`, in the
  `execd` binding. `git show b8aa0220 -- tests/codex_backend.rs` shows
  this stage's own fold commit turned an `if execd { … } else { … }`
  into a bare `execd` return, leaving the `let` binding dead — introduced
  by this wave's `b8aa0220 tests/codex_backend.rs: fold 15 deadline loops
  into wait_until_sync`.
- `tests/m2_daemon_api.rs:2720` — `clippy::await_holding_refcell_ref`,
  a `RefCell` `borrow_mut()` held across the `.await` on
  `tokio::time::timeout(...).await`. `git blame -L 2715,2725
  tests/m2_daemon_api.rs` attributes every line of this closure to this
  wave's `c214620e tests/m2_daemon_api.rs: fold 3 of 5 deadline loops,
  keep 2 as owned-wait-budget` — the `RefCell` workaround the
  implementation summary names for `wait_until`'s `FnMut` borrow
  limitation, applied to a call site whose body itself awaits while
  holding the borrow.

**Named as not done, with the reason:** both warnings are real defects in
this wave's own commits, not pre-existing. Fixing them is a code change to
already-landed test files, which is `15-validate`'s explicit non-job —
`.sergeant/workflows/implement-change/15-validate/CONTEXT.md`: *"This
stage does not fix a failure it finds... fixing it, if warranted, is
`30-fix-confirmed`'s job once the panel has weighed in, not a reason to
loop back here."* Recorded here as a failing check, carried forward
rather than patched to look clean.

### 3. Docs the brief names

`/var/tmp/hats7/brief-fold-the-deadline-loops.md` names no documentation
file to update (`grep -in doc` over the brief: no hits). Nothing to
close.

### 4. No clock decides correctness anywhere touched

The guard test above (`every_hand_rolled_deadline_in_tests_is_folded_or_named_in_the_residue`
and its four siblings) is the machine-checkable form of this claim and is
green. Manually: the `deadline-loop-residue` allowlist category is at 0
real entries (`grep -c` reports 1, but that hit is the doc-comment at
`tests/s6_no_clock_decides_correctness.rs:2160` describing the category
name, not an entry — `grep -n` confirms it is prose, not a `Site{...}`
literal). `DEADLINE_LOOP_RESIDUE_REASON` is absent
(`grep -rn DEADLINE_LOOP_RESIDUE_REASON tests/` → no hits). The 60
remaining hand-rolled `Instant::now()`-based deadline loops across the
touched files (`m9_watch.rs`, `agy_backend.rs`,
`v1d_probe_child_lifecycle.rs`, `m6_surfaces.rs`, `m5_projections.rs`,
`m4_backends.rs`, `m2_daemon_api.rs`, `w3_client_surface.rs`,
`m3_execution.rs`, `w4_read_surfaces.rs`) each sit on an explicit
`"owned-wait-budget"` allowlist entry (`grep -c '"owned-wait-budget"'
tests/s6_no_clock_decides_correctness.rs` → 60), which the guard walks and
accepts by name and reason, not by inference. Done, evidence shown.

## Rungs

R2 — reused the exact command `05-baseline` discovered rather than
inventing a new one. J3 — the command and the four close-out obligations
both come from the settled upstream record (`05-baseline`'s output and
the wave's own stage prompt), not a fresh judgment call. J1 — wording of
this record. No J0: the recorded command still ran without substitution,
so the one `15-validate` J0 trigger (a command that no longer runs) did
not fire.

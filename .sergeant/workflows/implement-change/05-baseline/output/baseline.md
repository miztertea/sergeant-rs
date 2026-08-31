# 05-baseline — #334: journal writes silently fail; the invariant is prose

Recorded **before any change**, against the fixed point `00-orient` pinned.
Nothing in this stage edited product code.

| | |
|---|---|
| Pinned revision | `f2b7a3720fe99c3a2d112cf8296339c98fdcec1b` |
| Lane | `/var/tmp/hats6/journal` |
| Tree at baseline | `git status --porcelain` empty except this stage's own artifact |
| Crate | `sergeant-rs v0.3.0` |

## 1. The discovered test command (J2 — delegated to this stage)

`CONTRIBUTING.md:15-36` names the gate and the fast path, read at source:

> ```sh
> cargo fmt --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked
> ```
> "`cargo nextest run --locked` is the fast path for the test step … same
> tests, same pass/fail contract"
> "`cargo nextest run --test <name>` and `cargo nextest run -E 'test(<substring>)'`
> are nextest's equivalents of the two `cargo test` invocations above."

Per host discipline the **full** suite is not run here — CI by SHA is the
exhaustive pass. The relevant subset was selected from `00-orient` §6's
boundary, one suite per thing the change touches:

| Suite | Why it is in the baseline | Boundary item |
|---|---|---|
| `m1_event_core` | `Journal::append`, seq contiguity, replay — the layer the defect corrupts | 1, 3 |
| `m5_projections` | `Projection::apply` / rebuild — where `projection seq mismatch` is raised (`src/runtime/projection.rs`) | 1, 3 |
| `w1b_overlay_lifecycle_trigger` | direct-journal writer #1 (`src/api.rs:5219`), the one that *does* absorb | 2 |
| `y5_external_git_triggers` | the defective endpoint's own suite — `POST /v1/intelligence/sources` (`intelligence_add_source`, `src/api.rs:5693`) | 1 |
| `x5_a1a_acceptance` | the other suite naming `intelligence_add_source`; A1a contract pins | 1 |
| `w3_allowlist_equivalence` | `kind_daemon_stopped_is_replay_equivalent` (`tests/w3_allowlist_equivalence.rs:204`) — replay semantics of the stop event | 4 |
| `m2_daemon_api` (3 tests) | the existing shutdown/`daemon.stopped` assertions | 4 |
| `c2_light` / `coverage_stage_membership` | the #231 membership guard a new suite must not break | 5 |

Exact commands, re-runnable verbatim:

```sh
cd /var/tmp/hats6/journal
CARGO_BUILD_JOBS=6 TMPDIR=/var/tmp/sgt-test-tmp cargo nextest run --no-fail-fast \
  --test m1_event_core --test m5_projections --test w1b_overlay_lifecycle_trigger \
  --test y5_external_git_triggers --test x5_a1a_acceptance --test w3_allowlist_equivalence

CARGO_BUILD_JOBS=6 TMPDIR=/var/tmp/sgt-test-tmp cargo nextest run --no-fail-fast \
  --test c2_light -E 'test(coverage_stage_membership)'

CARGO_BUILD_JOBS=6 TMPDIR=/var/tmp/sgt-test-tmp cargo nextest run --no-fail-fast \
  --test m2_daemon_api -E 'test(shutdown_completes_with_a_live_sse_client_attached) + test(t11d_a_stalled_completion_driver_does_not_hold_shutdown_open) + test(t11e_a_stalled_drivers_completed_settle_lands_before_daemon_stopped)'

cargo fmt --check
```

## 2. Real, verbatim baseline output (run once, before any change)

### 2a. Core + endpoint + replay set — **67 passed, 0 failed**

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
────────────
 Nextest run ID 6aa60e1d-5f8c-4c28-8443-9341c14640dc with nextest profile: default
    Starting 67 tests across 6 binaries
        PASS [   0.009s] sergeant-rs::m1_event_core replay_next_surfaces_io_error_for_invalid_utf8_segment
        PASS [   0.009s] sergeant-rs::m1_event_core blank_journal_line_fails_closed
        PASS [   0.011s] sergeant-rs::m1_event_core replay_next_surfaces_io_error_for_segment_deleted_after_listing
        PASS [   0.012s] sergeant-rs::m1_event_core snapshot_rewrite_is_atomic_replace
        PASS [   0.012s] sergeant-rs::m1_event_core round_trip_and_unknown_field_preservation
        PASS [   0.014s] sergeant-rs::m1_event_core crash_tail_recovery
        PASS [   0.015s] sergeant-rs::m1_event_core crash_tail_complete_json_without_newline_is_quarantined
        PASS [   0.016s] sergeant-rs::m1_event_core segment_rotation_and_cross_segment_replay
        PASS [   0.008s] sergeant-rs::m5_projections provenance_checking_rejects_every_mislabelled_edge
        PASS [   0.019s] sergeant-rs::m1_event_core seq_gap_or_duplicate_fails_closed
        …
        PASS [   0.337s] sergeant-rs::w3_allowlist_equivalence kind_daemon_stopped_is_replay_equivalent
        PASS [   0.056s] sergeant-rs::x5_a1a_acceptance a1a_item_12_no_atlas_write_path_is_reachable_from_the_cli
        PASS [  14.136s] sergeant-rs::w1b_overlay_lifecycle_trigger a_work_on_an_unindexed_estate_gains_no_atlas_evidence
        PASS [   8.833s] sergeant-rs::y5_external_git_triggers the_intelligence_sources_list_is_empty_until_something_is_added
        PASS [  27.126s] sergeant-rs::m5_projections t4_deleting_the_projections_directory_loses_nothing
────────────
     Summary [  27.145s] 67 tests run: 67 passed, 0 skipped
```

Full untruncated log: `/var/tmp/sgt-test-tmp/base_core.txt`. Per-binary counts
from that log — `m1_event_core` 14, `m5_projections` 21, `x5_a1a_acceptance` 11,
`w3_allowlist_equivalence` 10, `y5_external_git_triggers` 6,
`w1b_overlay_lifecycle_trigger` 5 (= 67). Process exit `0`.

### 2b. #231 coverage membership — **1 passed** (run, not reasoned about)

```
   Compiling sergeant-rs v0.3.0 (/var/tmp/hats6/journal)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.32s
────────────
 Nextest run ID 6c4ea062-85db-4114-a348-5bf3e31342bb with nextest profile: default
    Starting 1 test across 1 binary (27 tests skipped)
        PASS [   0.006s] sergeant-rs::c2_light coverage_stage_membership::every_suite_is_wired_or_explicitly_allowlisted
────────────
     Summary [   0.006s] 1 test run: 1 passed, 27 skipped
```

Process exit `0`. **This is green at the pin**, so any redness after
`10-implement` is that stage's new suite being unwired — not a pre-existing
failure it may inherit.

### 2c. Shutdown / `daemon.stopped` — **3 passed**

```
   Compiling sergeant-rs v0.3.0 (/var/tmp/hats6/journal)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.09s
────────────
 Nextest run ID 19f11dc5-c658-4423-a541-1a7960b2bb65 with nextest profile: default
    Starting 3 tests across 1 binary (65 tests skipped)
        PASS [   4.458s] sergeant-rs::m2_daemon_api t11e_a_stalled_drivers_completed_settle_lands_before_daemon_stopped
        PASS [   8.655s] sergeant-rs::m2_daemon_api shutdown_completes_with_a_live_sse_client_attached
        PASS [   9.378s] sergeant-rs::m2_daemon_api t11d_a_stalled_completion_driver_does_not_hold_shutdown_open
────────────
     Summary [   9.379s] 3 tests run: 3 passed, 65 skipped
```

Process exit `0`.

### 2d. Format gate

```
$ cargo fmt --check
(no output)
fmt EXIT=0
```

`cargo clippy --locked --all-targets -- -D warnings` was **not** run at
baseline: it is a separate compile profile whose cold pass is the heaviest
single command in this repo, and it is a CI job by SHA. `15-validate` should
treat a clippy failure as new work, not as a pre-existing state this baseline
cleared.

## 3. The gap this baseline pins — what is *absent*, not merely failing

```
$ grep -rn "absorb_journaled" tests/ scripts/
grep EXIT=1 (no match)
```

Pattern validated against a known positive (the standing rule that a count of
zero is a claim): the same pattern over `src/` returns five hits —
`src/api.rs:296` (the fn) and `:5225, :5409, :5481, :5578` (the four writers
that call it).

**So: zero tests and zero scripts mention the invariant.** Every one of the 71
tests above passes with the defect at `src/api.rs:5751` fully present. That is
the baseline's most load-bearing fact — the suite is green *and* the product
is broken, which is exactly why "tests currently pass" would have been a
worthless record here.

## 4. What this change is expected to move

### 4a. Must newly exist (currently absent — no test asserts any of it)

1. **A deterministic regression for the mismatch.** A direct-journal writer that
   skips the fold, then a `Core::commit`, produces
   `projection seq mismatch: expected N, got N+1`. `00-orient` §3 already drove
   this offline and clock-free in scratch
   (`/var/tmp/sgt-test-tmp/orient_repro_334.rs.keep`, deliberately uncommitted);
   `10-implement` owns the durable form.
2. **A regression for the cascade** (`00-orient` §3a): one un-absorbed write
   wedges *every* later commit — `expected` pinned, `got` climbing per failed
   commit — until some unrelated hold absorbs.
3. **The defect closed at `src/api.rs:5751`** (`intelligence_add_source`): after
   the fix, adding a source that genuinely scans leaves the registry level with
   the journal, and the next `commit` succeeds. Today nothing exercises this
   path's fold at all.
4. **A structural guard** (boundary item 2): a writer that appends through
   `&mut Journal` and skips the fold must fail to compile or fail a guard —
   proven non-vacuously by adding such a writer and capturing the red.
5. **A shutdown-stop-event test** (issue item 3) asserting `daemon.stopped` is
   present **and folded/published**, not merely appended — the distinction
   `00-orient` §3b established and which none of §2c's three existing tests
   makes.
6. **Coverage membership for any new suite** (#231): wired into
   `scripts/coverage/c2-suites.sh` or `c3-spawning-suites.sh`, with §2b's guard
   re-run green *and* shown red against an unwired suite.

### 4b. Must keep passing (currently green, listed above)

- All **67** tests of §2a — in particular `m1_event_core`'s
  `seq_gap_or_duplicate_fails_closed` and `deterministic_replay_and_snapshot_equivalence`
  (the contiguity contract must be *satisfied*, never relaxed to make the
  regression green), `m5_projections`'s `t1_rebuild_from_scratch_reproduces_every_canned_answer`
  and `t4_deleting_the_projections_directory_loses_nothing`,
  `w3_allowlist_equivalence::kind_daemon_stopped_is_replay_equivalent`, and
  `w1b_overlay_lifecycle_trigger`'s two lifecycle cases (writer #1 keeps
  absorbing).
- The **3** shutdown tests of §2c.
- §2b's `coverage_stage_membership::every_suite_is_wired_or_explicitly_allowlisted`.
- `cargo fmt --check` clean.

### 4c. Explicitly *not* expected to move (out of boundary, `00-orient` §6)

`Core::commit`'s append-before-fold order; any retry/serialisation redesign;
the `failed to journal …` log wording at `src/daemon.rs:1485`/`:1647`; the scan
front door `335d5892` (not on the pin). A `15-validate` finding that any of
these changed is a boundary breach, not progress.

## 5. Rungs cited

- **J2** — discovering the test command and selecting the relevant subset is
  this stage's delegated class (CONTEXT.md §J2). The command was read out of
  `CONTRIBUTING.md:15-36`, not invented; the subset was derived from
  `00-orient` §6's boundary, not chosen for convenience.
- **J3** — the boundary and the located cause are settled by `00-orient`'s
  accepted output; this stage re-derives neither.
- **J1** — wording of this record only.
- **J0** — none newly arises here. Issue item 2 stays escalated at `00-orient`
  §7; nothing in this baseline decides it, and §4a items 1–6 do not depend on
  it.
- No R-rung: this stage constructs nothing. **Change nothing** was honoured —
  no file under `src/`, `tests/`, or `scripts/` was touched.

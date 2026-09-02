# Validation — wave `transport-timeout-is-not-a-verdict` (stage 15-validate)

Revision validated: `7b43dd6d725bcc9f1ceb7f8cbe7888db10a28786` (base `20c2a9ac944faf2b8fd1b0306b41da15f8a90946`).
Lane confirmed clean before and after every run below (`git status --short`
empty, `git rev-parse HEAD` unchanged at `7b43dd6d`). No concurrent build
(`pgrep -af '[r]ustc' | wc -l` = 0) checked before each invocation.

Per this stage's own contract (`CONTEXT.md`): the baseline's recorded test
command, run against the change, output recorded verbatim — pass or fail,
not fixed here regardless. **Every run below passed; there is no failure to
carry forward.**

## 1. Command run

Same command discovery as `05-baseline` (`CONTRIBUTING.md:16-31`, J2, no
change from baseline — the command still runs exactly as documented; no J0
escalation needed): `cargo nextest run --locked --test <name>` per suite the
brief's "Decisive close" and this wave's own boundary name, plus the two
whole-gate checks, plus the new suite the implementation added.
`TMPDIR=/var/tmp/sgt-test-tmp CARGO_BUILD_JOBS=6` prefixed throughout. No
full `cargo test --locked` / unfiltered `cargo nextest run --locked` was
run, per this lane's standing "no full suite locally" policy (J5) — same
scope 05-baseline recorded, now with the one new suite the implementation
added.

## 2. Results — real, verbatim output, this revision

### 2a. Guard suite — `tests/s6_no_clock_decides_correctness.rs`

Command: `cargo nextest run --locked --test s6_no_clock_decides_correctness`
Full output: `/var/tmp/hats7/validate-guard.log`

```
Starting 7 tests across 1 binary
    PASS [   0.003s] sergeant-rs::s6_no_clock_decides_correctness the_timeout_guard_fails_on_an_unallowlisted_client_timeout
    PASS [   0.004s] sergeant-rs::s6_no_clock_decides_correctness the_guard_fails_on_a_real_deadline_decides_a_verdict_construct_with_no_allowlist_entry
    PASS [   0.004s] sergeant-rs::s6_no_clock_decides_correctness the_sleep_guard_fails_on_a_state_blind_loop_even_though_it_is_lexically_a_loop
    PASS [   0.004s] sergeant-rs::s6_no_clock_decides_correctness the_sleep_guard_fails_on_a_bare_sleep_with_no_loop_and_no_allowlist_entry
    PASS [   0.085s] sergeant-rs::s6_no_clock_decides_correctness every_time_construct_in_the_crate_sits_on_an_explicit_allowlist
    PASS [   0.855s] sergeant-rs::s6_no_clock_decides_correctness every_client_timeout_in_tests_sits_on_an_explicit_allowlist
    PASS [   0.858s] sergeant-rs::s6_no_clock_decides_correctness every_sleep_in_tests_sits_inside_a_terminating_loop_or_an_explicit_allowlist
Summary [   0.858s] 7 tests run: 7 passed, 0 skipped
```

**Moved as expected**: baseline had 5 tests (no `.timeout(` walk); this
revision has 7 — the 2 new tests are the `.timeout(` walk itself
(`every_client_timeout_in_tests_sits_on_an_explicit_allowlist`) and its own
"unallowlisted timeout is caught" vacuity self-check
(`the_timeout_guard_fails_on_an_unallowlisted_client_timeout`). All 5
baseline tests still pass unchanged (regression bar held).

### 2b. New suite — `tests/s6_scan_poll_survives_a_transport_timeout.rs`

Command: `cargo nextest run --locked --test s6_scan_poll_survives_a_transport_timeout`
Full output: `/var/tmp/hats7/validate-scan-poll.log`

```
Starting 3 tests across 1 binary
    PASS [   0.023s] sergeant-rs::s6_scan_poll_survives_a_transport_timeout a_status_poll_transport_failure_against_a_dead_daemon_fails_at_once
    PASS [   0.195s] sergeant-rs::s6_scan_poll_survives_a_transport_timeout support::cross_process_lock_tests::excludes_concurrent_holders_of_the_same_name
    PASS [   0.231s] sergeant-rs::s6_scan_poll_survives_a_transport_timeout a_status_poll_that_stalls_past_the_client_timeout_is_retried_not_panicked
Summary [   0.231s] 3 tests run: 3 passed, 0 skipped
```

**This is the wave's sharpest before/after pair.** `00-orient`'s throwaway
repro against the pinned baseline revision panicked at exactly this
scenario (`tests/support/mod.rs:920`, `reqwest::Error{kind:Request,
source:TimedOut}`); the same scenario, now the permanent regression test
`a_status_poll_that_stalls_past_the_client_timeout_is_retried_not_panicked`,
passes here at `7b43dd6d` — 0.231s solo, no clock deciding the outcome (the
50ms retry sleep is cadence only, per its own doc comment; the sibling test
pins the dead-daemon half: transport error + `is_alive()==false` fails at
once, not retried).

### 2c. `tests/s6_semantic_crossing.rs` (regression bar)

Command: `cargo nextest run --locked --test s6_semantic_crossing`
Full output: `/var/tmp/hats7/validate-s6_semantic_crossing.log`

```
Starting 7 tests across 1 binary
    PASS [   0.609s] a_markdown_only_scan_does_not_report_its_own_lexical_index_as_needing_rebuild
    PASS [   4.928s] a_model_directory_that_will_not_load_is_not_reported_as_no_assets
    PASS [   4.929s] a_host_with_no_assets_reports_not_installed_rather_than_a_fault
    SLOW [> 60.000s] a_real_search_through_the_daemon_answers_inside_sgts_own_budget
    PASS [  86.769s] a_real_search_through_the_daemon_answers_inside_sgts_own_budget
    SLOW [> 60.000s] a_search_over_an_index_with_no_stored_vectors_says_so_at_the_crossing
    PASS [  78.673s] a_search_over_an_index_with_no_stored_vectors_says_so_at_the_crossing
    PASS [   0.194s] support::cross_process_lock_tests::excludes_concurrent_holders_of_the_same_name
    PASS [   8.387s] the_operator_can_ask_whether_the_semantic_model_is_loaded
Summary [ 178.759s] 7 tests run: 7 passed (2 slow), 0 skipped
```

**Regression bar held: all 7 still pass**, unchanged assertions, run solo
exactly as baseline. The two named-flaky-under-contention tests pass at
86.8s / 78.7s here (baseline: 83.7s / 67.7s) — both within the documented
~70-72s-unstarved-cost envelope's normal single-lane variance; still no
concurrent build during this run (confirmed empty `pgrep` before and after).

### 2d. `tests/s6_scan_front_door.rs` (regression bar)

Command: `cargo nextest run --locked --test s6_scan_front_door`
Full output: `/var/tmp/hats7/validate-s6_scan_front_door.log`

```
Starting 8 tests across 1 binary
    PASS [   0.029s] the_shared_scan_driver_gives_up_when_the_status_endpoint_stops_tracking_the_scan
    PASS [   0.193s] support::cross_process_lock_tests::excludes_concurrent_holders_of_the_same_name
    PASS [   6.397s] an_unknown_scan_id_is_refused_by_name_rather_than_answered_with_a_fabricated_completion
    SLOW [> 60.000s] the_trigger_accepts_and_names_the_scan_instead_of_answering_with_the_whole_report
    SLOW [> 60.000s] per_source_progress_is_visible_while_the_scan_is_still_running
    SLOW [> 60.000s] the_journal_records_the_scan_that_started_and_the_scan_that_completed
    SLOW [> 60.000s] the_verb_itself_exits_zero_and_prints_a_row_for_every_source
    SLOW [> 60.000s] completion_is_reported_by_the_scan_itself_not_counted_from_another_command
    PASS [  67.254s] the_trigger_accepts_and_names_the_scan_instead_of_answering_with_the_whole_report
    PASS [  67.446s] the_verb_itself_exits_zero_and_prints_a_row_for_every_source
    PASS [  67.465s] per_source_progress_is_visible_while_the_scan_is_still_running
    PASS [  67.817s] the_journal_records_the_scan_that_started_and_the_scan_that_completed
    PASS [  68.489s] completion_is_reported_by_the_scan_itself_not_counted_from_another_command
Summary [  68.489s] 8 tests run: 8 passed (5 slow), 0 skipped
```

**Regression bar held: all 8 still pass** (baseline: all 8 pass, 62.5-62.8s;
this run: 67.3-68.5s — normal single-lane variance, no concurrent build).
This suite calls `scan_to_completion` (00-orient §4c caller list) so it is a
direct regression check on the changed helper, not just an unrelated
sibling.

### 2e. `cargo fmt --check`

Full output: `/var/tmp/hats7/validate-fmt.log` (empty — no diff).
**Clean, exit 0** — unchanged from baseline.

### 2f. `cargo clippy --locked --all-targets -- -D warnings`

Full output: `/var/tmp/hats7/validate-clippy.log`:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
```
**Clean, exit 0** — unchanged from baseline.

## 3. Comparison against what `05-baseline` said would move

| Item (baseline §3) | Baseline state | This validation |
|---|---|---|
| `tests/`-visible delay-capable scripted-server helper | absent | present — `tests/support::spawn_scripted_http_server`, confirmed in diff |
| Red-first `scan_to_completion` regression test | panicked (00-orient repro) | passes (2b above), matching failure shape exactly |
| 6th guard test (`.timeout(` walk self-check) | 5 tests | **7** tests (2 new: the walk itself + its own vacuity self-check — one more than baseline's "a 6th" framing anticipated, both accounted for above) |
| Corrected doc comment on `scan_to_completion` | false claim present | corrected — `tests/support/mod.rs` diff carries the new doc comment naming the transport-failure case explicitly |
| Guard suite (5 baseline tests) | 5/5 green | still 5/5 green, plus 2 new |
| `s6_semantic_crossing` (7 tests) | 7/7 green | 7/7 green |
| `s6_scan_front_door` (8 tests) | 8/8 green | 8/8 green |
| fmt / clippy | clean | clean |

**Every comparison point moved exactly as baseline anticipated. No
regression. No failure recorded.**

## 4. Observation carried forward, not this stage's to fix

Per this stage's own contract ("This stage does not fix a failure it
finds... fixing it... is `30-fix-confirmed`'s job"), and per the routing
instruction that this stage does exactly its own work and not the next
stage's: **the new suite `tests/s6_scan_poll_survives_a_transport_timeout.rs`
is not yet wired into `scripts/coverage/c2-suites.sh`** —
`grep -n "s6_scan_poll_survives_a_transport_timeout" scripts/coverage/c2-suites.sh`
returns no hit, while the two existing regression suites both have
`cov_stage_begin`/`cov_run`/`cov_stage_end` entries there
(`c2-suites.sh:406-408`, `496-498`). This is not a test failure — every run
above is green — so it does not belong in §2's pass/fail record, but it is
flagged here as the one closeout item this stage's own contract forbids it
from actioning (wiring a new suite into `scripts/` is an implementation
change, not a validation run). Per #231's own standard quoted in this
wave's closeout list ("a suite absent from every stage list contributes
nothing to Gate D however green it runs"), the next stage that touches
`src/`/`scripts/` should close this before the wave ships.

## 5. Rungs cited

- **J2** — command discovery unchanged from `05-baseline`
  (`CONTRIBUTING.md:16-31`); the new suite's own command
  (`cargo nextest run --locked --test s6_scan_poll_survives_a_transport_timeout`)
  follows the same documented per-suite form, no escalation needed.
- **J4** — suite scope follows the brief's "Decisive close" naming exactly,
  as `05-baseline` already established; the new suite is added because the
  implementation added it, not because this stage widened scope.
- **J5** — this stage's own contract ("This stage does not fix a failure it
  finds") governs §4: the coverage-wiring gap is recorded, not patched here,
  even though it would be a one-line fix — because fixing it is out of this
  stage's role, not because it is hard.

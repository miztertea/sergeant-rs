# 15-validate — #334: journal writes silently fail; the invariant is prose

The test command `05-baseline` recorded has been run against the implemented
change. **Result: pass, every command, exit 0.** Nothing was fixed here — this
stage's contract (`15-validate/CONTEXT.md:30-36`) forbids it, and nothing needed
fixing.

| | |
|---|---|
| Tree validated | `05090bf1` (`10-implement`'s last commit) |
| Baseline compared against | `c80bc969` / pin `f2b7a372` |
| Lane | `/var/tmp/hats6/journal`, `git status --porcelain` empty before and after |
| Crate | `sergeant-rs v0.3.0` |

## 1. The baseline's command, re-run verbatim

Commands copied from `05-baseline/output/baseline.md` §1 without substitution;
all with `CARGO_BUILD_JOBS=6 TMPDIR=/var/tmp/sgt-test-tmp`.

### 1a. Core + endpoint + replay set — **67 passed, 0 failed** (log `/var/tmp/sgt-test-tmp/val_core.txt`)

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
────────────
 Nextest run ID af09a848-369c-4a6c-b1a9-7a1d4197672e with nextest profile: default
    Starting 67 tests across 6 binaries
        PASS [   0.028s] sergeant-rs::m1_event_core blank_journal_line_fails_closed
        PASS [   0.153s] sergeant-rs::m1_event_core seq_gap_or_duplicate_fails_closed
        PASS [   0.330s] sergeant-rs::m1_event_core deterministic_replay_and_snapshot_equivalence
        PASS [   0.156s] sergeant-rs::w3_allowlist_equivalence kind_daemon_stopped_is_replay_equivalent
        …
        PASS [  31.125s] sergeant-rs::m5_projections t1_rebuild_from_scratch_reproduces_every_canned_answer
        PASS [  36.872s] sergeant-rs::m5_projections t4_deleting_the_projections_directory_loses_nothing
        PASS [  37.248s] sergeant-rs::m5_projections a_restart_over_the_existing_projection_file_rebuilds_it
────────────
     Summary [  37.248s] 67 tests run: 67 passed, 0 skipped
```
`CORE EXIT=0`. Per-binary counts from the log: `m1_event_core` 14,
`m5_projections` 21, `x5_a1a_acceptance` 11, `w3_allowlist_equivalence` 10,
`y5_external_git_triggers` 6, `w1b_overlay_lifecycle_trigger` 5 = **67 —
identical to the baseline's split**, so this is the same set, not a
coincidentally equal total.

Baseline §4b named four tests by hand as the ones that must be *satisfied*,
never relaxed to make the regression green. All four are in the `PASS` lines
above at their own names: `seq_gap_or_duplicate_fails_closed` (line 18 of the
log), `deterministic_replay_and_snapshot_equivalence` (20),
`kind_daemon_stopped_is_replay_equivalent` (34), and both
`w1b_overlay_lifecycle_trigger` lifecycle cases — writer #1 still absorbs.

### 1b. #231 coverage membership — **1 passed** (log `val_231.txt`)

```
   Compiling sergeant-rs v0.3.0 (/var/tmp/hats6/journal)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.19s
────────────
 Nextest run ID 43eb6da7-e97a-4aa9-8469-b03f625fb142 with nextest profile: default
    Starting 1 test across 1 binary (27 tests skipped)
        PASS [   0.006s] sergeant-rs::c2_light coverage_stage_membership::every_suite_is_wired_or_explicitly_allowlisted
────────────
     Summary [   0.006s] 1 test run: 1 passed, 27 skipped
```
`231 EXIT=0`.

### 1c. Shutdown / `daemon.stopped` — **3 passed** (log `val_m2.txt`)

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
────────────
 Nextest run ID 3837fe6f-ced4-4c12-aa58-84179fe2aab4 with nextest profile: default
    Starting 3 tests across 1 binary (65 tests skipped)
        PASS [   4.512s] sergeant-rs::m2_daemon_api t11e_a_stalled_drivers_completed_settle_lands_before_daemon_stopped
        PASS [   8.589s] sergeant-rs::m2_daemon_api shutdown_completes_with_a_live_sse_client_attached
        PASS [   9.415s] sergeant-rs::m2_daemon_api t11d_a_stalled_completion_driver_does_not_hold_shutdown_open
────────────
     Summary [   9.416s] 3 tests run: 3 passed, 65 skipped
```
`M2 EXIT=0`.

### 1d. Format gate

```
$ cargo fmt --check
(no output)
FMT EXIT=0
```

## 2. What the baseline said must newly exist — run, not asserted

`05-baseline` §4a listed six things absent at the pin. The suite that carries
them now runs (log `val_new.txt`):

```
 Nextest run ID 4af20319-d1d1-4779-ba22-3afc9a0b7711 with nextest profile: default
    Starting 12 tests across 2 binaries
        PASS [   0.008s] sergeant-rs::w3_prune_engine prune_runs_only_under_the_core_guard
        PASS [   0.011s] sergeant-rs::f334_journal_integrity every_direct_journal_writer_in_the_api_absorbs_before_releasing_its_hold
        PASS [   0.020s] sergeant-rs::w3_prune_engine no_configuration_or_flag_can_lower_the_prune_predicate
        PASS [   0.323s] sergeant-rs::w3_prune_engine support::cross_process_lock_tests::excludes_concurrent_holders_of_the_same_name
        PASS [   3.049s] sergeant-rs::f334_journal_integrity a_shutdown_journals_and_publishes_its_stop_event_even_from_a_wedged_registry
        PASS [   3.049s] sergeant-rs::f334_journal_integrity a_hold_that_skipped_absorb_journaled_is_counted_as_a_breach
        PASS [   3.055s] sergeant-rs::f334_journal_integrity a_hold_that_absorbed_records_no_breach
        PASS [   3.055s] sergeant-rs::f334_journal_integrity the_wedged_cascade_from_the_issue_is_recovered_when_the_hold_releases
        PASS [   3.056s] sergeant-rs::f334_journal_integrity a_hold_that_appended_directly_without_absorbing_does_not_wedge_the_next_commit
        PASS [  11.339s] sergeant-rs::w3_prune_engine a_rotation_crossing_the_cap_arms_a_prune_within_one_tick
        PASS [  14.950s] sergeant-rs::w3_prune_engine a_start_on_an_over_cap_journal_prunes_before_serving
        PASS [  18.879s] sergeant-rs::w3_prune_engine a_start_after_a_prune_with_no_cache_still_serves
────────────
     Summary [  18.880s] 12 tests run: 12 passed, 0 skipped
```
`NEW EXIT=0`. Total across every command above: **83 tests, 83 passed, 0
failed** (67 + 1 + 3 + 12).

## 3. Non-vacuity, re-proven here — not inherited from `10-implement`

`10-implement` captured its own reds. This stage did not take them on trust: the
standing rule is "would it pass with the feature deleted?", so each guard was
re-broken against **this** tree and restored. Every probe was a patch-then-
`git status --porcelain`-empty restore; the tree was verified clean after each,
and the suite re-run green at the end (`Summary … 6 tests run: 6 passed`).

**Probe A — delete the located fix** (the `absorb_journaled` block at
`src/api.rs:5815`, the whole of #334):

```
thread 'every_direct_journal_writer_in_the_api_absorbs_before_releasing_its_hold' panicked at tests/f334_journal_integrity.rs:274:5:
these functions in src/api.rs append straight through `&mut …journal` and never call `Core::absorb_journaled`, so they release the hold with the registry behind the journal and wedge the next commit (#334): [
    "async fn intelligence_add_source( (line 5815)",
]
     Summary [   0.010s] 1 test run: 0 passed, 1 failed, 5 skipped
```
One entry, naming the real writer. The four writers that *do* absorb are matched
by the same scan and stay silent — the pattern is validated by a known positive,
not trusted at a count of zero.

**Probe B — disable the choke point** (`self.absorb_before_release();` removed
from `Core::flush`, `src/api.rs:272`): **4 of 6 red.**

```
thread 'a_hold_that_appended_directly_without_absorbing_does_not_wedge_the_next_commit' panicked at tests/f334_journal_integrity.rs:145:10:
the next commit must not be wedged by a writer that forgot to absorb: Projection(SeqMismatch { expected: 2, found: 3 })

thread 'a_shutdown_journals_and_publishes_its_stop_event_even_from_a_wedged_registry' panicked at tests/f334_journal_integrity.rs:395:5:
the stop event must also reach the surfaces that read the record — being in the journal and absent from every projection and subscriber is the failure #334 actually is (00-orient §3b), got []
     Summary [   0.160s] 6 tests run: 2 passed, 4 failed, 0 skipped
```
The two that stayed green are the correct two and worth naming: the control
(`a_hold_that_absorbed_records_no_breach` — a writer that *did* absorb is
unaffected by the choke point being gone) and the source-text guard (probe B
changed runtime behaviour, not the source pattern it scans). A probe that turned
*everything* red would have been the weaker result.

`SeqMismatch { expected: 2, found: 3 }` is the issue's own shape at minimum
scale; `got []` is `00-orient` §3b entire — journaled, published to nobody,
while the operator was told "failed to journal".

**Probe C — unwire the new suite** (comment the `cov_run` line at
`scripts/coverage/c2-suites.sh:426`), because the brief calls the #231 guard
itself unproven after a sibling lane:

```
thread 'coverage_stage_membership::every_suite_is_wired_or_explicitly_allowlisted' panicked at tests/c2_light/coverage_stage_membership.rs:132:5:
new orphaned suite(s) wired into neither c2-suites.sh nor c3-spawning-suites.sh, and not named in this test's ALLOWLIST with a reason: ["f334_journal_integrity"] — wire it into a coverage stage, or add an allowlist entry with a specific reason (#231(b))
     Summary [   0.006s] 1 test run: 0 passed, 1 failed, 27 skipped
```
The guard catches an unwired suite. It was run, not reasoned about, in both
directions.

## 4. The four close-outs

**1. Every new suite wired into `scripts/coverage/` (#231) — done, with the hit shown.**
```
$ grep -rn "f334_journal_integrity" scripts/
scripts/coverage/c2-suites.sh:425:cov_stage_begin c2-f334_journal_integrity
scripts/coverage/c2-suites.sh:426:cov_run cargo llvm-cov --no-report --test f334_journal_integrity --locked || cov_fail "f334_journal_integrity failed under instrumentation"
scripts/coverage/c2-suites.sh:427:cov_stage_end 1 "the f334_journal_integrity test binary must write its own profile"
grep EXIT=0
```
`f334_journal_integrity` is the only new suite (`git diff --stat c80bc969..05090bf1`
lists exactly four files: this workflow artifact, `scripts/coverage/c2-suites.sh`,
`src/api.rs`, `tests/f334_journal_integrity.rs`). Probe C above proves the
membership guard would have caught it had it not been wired.

**2. `cargo fmt` and `cargo clippy --all-targets` — both clean.**
```
$ cargo fmt --check                                              → FMT EXIT=0 (no output)
$ CARGO_BUILD_JOBS=6 cargo clippy --locked --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.32s
                                                                 → CLIPPY EXIT=0
```
Stated plainly: clippy `Finished` in 0.32s off a warm fingerprint from this
lane's earlier run against this same clean tree — the exit code is real, the
compile was cached. `05-baseline` §2d recorded clippy as *not* pre-cleared, so
this is new evidence, not an inherited pass. CI runs it cold by SHA.

**3. Docs the brief names — none named; nothing skipped.**
The brief (`brief-334-journal-integrity.md`) mentions docs only in the sense of
the *existing* `absorb_journaled` doc comment being prose that item 4 must
replace with structure. It names no documentation deliverable — no ADR, no
`docs/` page, no CHANGELOG entry. Read, not recalled: the brief's five numbered
deliverables are the writer enumeration, the scan-front-door check, the
deterministic repro, the structural guard, and the shutdown stop event; its
Constraints list names no doc. Recorded as *not applicable*, not as *done*.

**4. No clock decides correctness anywhere touched — verified, pattern validated.**
```
$ grep -n "Instant\|elapsed\|sleep\|timeout\|Duration\|BUDGET\|deadline" tests/f334_journal_integrity.rs
grep EXIT=1  (no match)
$ git diff c80bc969..05090bf1 -- src/api.rs | grep "^+" | grep -i "instant\|elapsed\|sleep\|timeout\|duration\|deadline"
grep EXIT=1  (no match)
```
A count of zero is a claim, so the same pattern was validated against a known
positive in the same tree — `tests/w3_prune_engine.rs` returns
`123: tokio::time::sleep(…from_millis(50))`, `148: .timeout(…from_secs(20))`,
and three more. The pattern finds clocks where clocks exist. The new suite has
**no clock at all** — not a poll interval needing a cadence comment, none: every
assertion waits on recorded state (a journal seq, a registry `last_seq`, a
`unabsorbed_holds` count, a published event list). `bdda34f3`'s stripped class is
not reintroduced.

## 5. Carried forward unchanged — still `blocked`, and not by this stage

Issue item 2 ("is a failed journal write ever acceptable?") remains escalated at
`00-orient` §7 with its recommendation and evidence. It is a **J0** (`AGENTS.md`:
no lower rung resolves a first-principle contract change), and nothing in this
validation touched, decided, or depended on it. `05-baseline` §4a's six
expected-to-move items all validated green without it, exactly as the baseline
predicted.

Also carried, untouched and deliberately so (`00-orient` §6): the `failed to
journal …` wording at `src/daemon.rs:1485`/`:1647`, `Core::commit`'s
append-before-fold ordering, and the scan front door `335d5892` (not on the pin
— the brief's deliverable 2 is an orient-stage finding, not a change this
validation could make).

## Rungs

- **J3** — the command, the subset, and the pass/fail criteria were settled by
  `05-baseline`; this stage ran them, it did not choose them.
- **J2** — the probes. `15-validate/CONTEXT.md:43-45` delegates "ordinary tool
  mechanics"; re-breaking a guard on this tree and restoring it is how you learn
  the recorded pass is real, and the standing requirement ("would it pass with
  the feature deleted?") makes it obligatory rather than optional. No product
  behaviour was changed: every probe ended `git status --porcelain` empty.
- **J1** — the wording of this record.
- **No J0 raised here.** The one J0 in this wave is untouched and still escalated.
- **No R-rung** — this stage constructed nothing.

## Verdict

The baseline's own command, re-run against the change: **83/83 pass, exit 0,
every command**. No failure to carry forward. The panel gets a green record whose
guards were each proven red on this tree before it was written.

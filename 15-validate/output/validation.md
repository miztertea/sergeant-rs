# 15-validate — #297: the close sweep deletes other Works' merged evidence

Run against the implementation at `84668c61` (tree clean before and after
this stage's own test runs; `git status --porcelain` empty). Pinned
revision unchanged from `00-orient` §1:
`968c9edf1ceb9c214eb9daff8460f132c4dfaef2`. Nothing re-pinned, nothing
under `src/`, `tests/`, or `scripts/` changed by this stage.

The commands below are **`05-baseline` §1's own three**, re-run verbatim —
not a substitute set. Every one is prefixed
`TMPDIR=/var/tmp/sgt-test-tmp CARGO_BUILD_JOBS=6`. The full suite was
deliberately not run locally (CI by SHA is the exhaustive pass; wave brief
"Constraints").

## 1. The baseline's commands, run against the change — verbatim

### B1 — `cargo nextest run --no-fail-fast --lib -E 'test(/^runtime::surface::tests::\|^domain::workflow::tests::/)'`

`EXIT=0`. Full log `/var/tmp/sgt-test-tmp/validate-b1.log`. Tail, verbatim:

```
        PASS [   0.021s] sergeant-rs runtime::surface::tests::stage_scan_within_max_depth_stays_quiet
        PASS [   0.157s] sergeant-rs runtime::surface::tests::dirty_teardown_labels_promote_class_output_distinctly_from_evidence
        PASS [   0.144s] sergeant-rs runtime::surface::tests::dirty_teardown_retains_a_stages_nested_output_artifacts
        PASS [   0.128s] sergeant-rs runtime::surface::tests::dirty_teardown_retains_no_stage_output_without_a_bound_workflow_package
        PASS [   0.036s] sergeant-rs runtime::surface::tests::stage_scan_past_max_depth_is_not_silent
        PASS [   0.152s] sergeant-rs runtime::surface::tests::dirty_teardown_leaves_an_undeclared_stage_directory_out_of_retained_output
        PASS [   0.142s] sergeant-rs runtime::surface::tests::dirty_teardown_retains_each_stages_declared_output_artifacts
        PASS [   0.168s] sergeant-rs runtime::surface::tests::dirty_teardown_excludes_gitignored_files_from_retained_output
        PASS [   0.166s] sergeant-rs runtime::surface::tests::dirty_teardown_retains_a_nested_leafs_output
        PASS [   0.095s] sergeant-rs runtime::surface::tests::the_finalize_sweep_reaches_a_nested_leafs_output
        PASS [   1.124s] sergeant-rs runtime::surface::tests::concurrent_surfaces_on_one_repository_all_materialize_and_retire_cleanly
────────────
     Summary [   1.277s] 126 tests run: 126 passed, 1232 skipped
```

Baseline was `122 tests run: 122 passed`. The delta is exactly the four new
tests (`dirty_teardown_retains_no_stage_output_without_a_bound_workflow_package`,
`dirty_teardown_leaves_an_undeclared_stage_directory_out_of_retained_output`,
`declared_stage_ids_are_the_running_workflows_leaves_and_containers`,
`declared_stage_ids_is_none_when_the_package_will_not_load`). **No baseline
test moved from pass to fail.**

### B2 — `cargo nextest run --no-fail-fast --test m3_execution -E 'test(/sweep\|output/)'`

`EXIT=0`, complete output verbatim:

```
   Compiling sergeant-rs v0.3.0 (/var/tmp/hats6/sweep)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.01s
────────────
 Nextest run ID 82a7834d-8a9b-4619-aa3d-48aa28b69159 with nextest profile: default
    Starting 4 tests across 1 binary (58 tests skipped)
        PASS [   5.308s] sergeant-rs::m3_execution the_finalize_sweep_leaves_an_undeclared_stage_directory_alone
        PASS [   5.342s] sergeant-rs::m3_execution a_finalize_sweep_never_launders_unrelated_dirty_content_into_its_own_commit
        PASS [   5.378s] sergeant-rs::m3_execution t10_a_stage_completed_without_its_declared_output_is_reprompted_then_needs_input
        PASS [   5.432s] sergeant-rs::m3_execution the_finalize_sweep_removes_evidence_class_output_and_keeps_promote_class
────────────
     Summary [   5.432s] 4 tests run: 4 passed, 58 skipped
```

Baseline was 3 tests; the fourth is the decisive two-Work test. The
baseline's own three still pass.

### B3 — `cargo nextest run --no-fail-fast --test c2_light -E 'test(coverage_stage_membership)'` (#231)

`EXIT=0`, complete output verbatim:

```
   Compiling sergeant-rs v0.3.0 (/var/tmp/hats6/sweep)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.23s
────────────
 Nextest run ID dfba90a9-f1db-4910-a0ca-9b3edbb86568 with nextest profile: default
    Starting 1 test across 1 binary (27 tests skipped)
        PASS [   0.006s] sergeant-rs::c2_light coverage_stage_membership::every_suite_is_wired_or_explicitly_allowlisted
────────────
     Summary [   0.006s] 1 test run: 1 passed, 27 skipped
```

**Result: 131 tests run across the three baseline commands, 131 passed, 0
failed.** Every test in `05-baseline` §4b's regression fence is inside B1/B2
and passes.

## 2. Non-vacuity, measured — the feature deleted, the guards watched go red

`05-baseline` §2 recorded the load-bearing fact that the suite was **green
while the defect was live**. A green re-run is therefore not by itself
evidence. Two real violations were inserted into the working tree, the
tests re-run, and the tree restored (`git status --porcelain` empty after
each; nothing below is committed).

### Violation A — the fix removed (both membership filters, both fail-closed early returns, the `unwrap_or(Evidence)` fail-open restored)

`EXIT=100`, verbatim:

```
        FAIL [   0.155s] sergeant-rs runtime::surface::tests::dirty_teardown_leaves_an_undeclared_stage_directory_out_of_retained_output
    panicked at src/runtime/surface.rs:5579:9:
    assertion `left == right` failed: #297: only the running workflow's own declared stage is this Work's to retain
        FAIL [   0.137s] sergeant-rs runtime::surface::tests::dirty_teardown_retains_no_stage_output_without_a_bound_workflow_package
    panicked at src/runtime/surface.rs:5628:9:
    #297: with no package to declare stage ids, the sweep retains nothing: [RetainedStageOutput { stage: "20-panel", path: "/var/tmp/sgt-test-tmp/.tmpW05AFV/01NOPACKAGE/solo.output/20-panel", bytes: 18, disposition: Evidence }]
     Summary [   1.355s] 126 tests run: 124 passed, 2 failed, 1232 skipped
```

and end to end (`--test m3_execution`), `EXIT=100`:

```
        FAIL [   5.326s] sergeant-rs::m3_execution the_finalize_sweep_leaves_an_undeclared_stage_directory_alone
    panicked at tests/m3_execution.rs:93:5:
    git ["show", "sergeant/01M1ASYWPHZAVBKBV108AFRM83:99-foreign/output/leftover.md"] failed in /var/tmp/sgt-test-tmp/.tmpyjr0nr/solo-estate/repos/solo: fatal: path '99-foreign/output/leftover.md' exists on disk, but not in 'sergeant/01M1ASYWPHZAVBKBV108AFRM83'
        PASS [   5.264s] sergeant-rs::m3_execution a_finalize_sweep_never_launders_unrelated_dirty_content_into_its_own_commit
        PASS [   5.323s] sergeant-rs::m3_execution the_finalize_sweep_removes_evidence_class_output_and_keeps_promote_class
     Summary [   5.463s] 4 tests run: 3 passed, 1 failed, 58 skipped
```

That failure is #297 itself, reproduced by observable state and nothing
else: the prior Work's already-merged `99-foreign/output/leftover.md` is
present on disk and **absent from the branch** the sweep committed. The
three tests that guard the fix are non-vacuous; the pre-existing sweep
tests stayed green under the violation, which is what makes them a fence
rather than a duplicate.

The decisive test's assertions were read, not assumed
(`tests/m3_execution.rs`): the foreign directory is seeded **committed**
into the mount (`git add -A` before submit), then judged on
`git show <branch>:99-foreign/output/leftover.md` byte for byte, on the
finalize commit's own `--name-status` diff, and on `00-evidence` still
having been swept — so a filter matching nothing cannot pass it.

### Violation B — the container half of `declared_stage_ids` dropped

This is `05-baseline` §4b's explicitly flagged **gap**: the second,
opposite data-loss failure mode (a leaves-only scope would stop sweeping
container output). `EXIT=100`, verbatim:

```
        FAIL [   0.024s] sergeant-rs domain::workflow::tests::declared_stage_ids_are_the_running_workflows_leaves_and_containers
    panicked at src/domain/workflow.rs:2271:9:
    assertion `left == right` failed: the declared set is composed leaf ids union container ids, in the same `/`-joined shape `composed_stage_id` builds from a worktree
        FAIL [   0.083s] sergeant-rs runtime::surface::tests::the_finalize_sweep_reaches_a_nested_leafs_output
    panicked at src/runtime/surface.rs:5968:9:
    a container's own declared output is swept too (W1 §4): ["10-investigate/00-lead"]
     Summary [   1.368s] 126 tests run: 124 passed, 2 failed, 1232 skipped
```

The gap is closed: the container case is fenced by an assertion at the
`finalize_sweep` level, not only at the helper's.

**Contract check (J5).** `10-implement` took `stages ∪ containers` over the
brief's `pub stages: Vec<StageDefinition>` wording. The contract backs it,
read directly and not from the brief's account:
`W1-HIERARCHICAL-EXECUTION.md:96` — "a container stage completes only after
its nested package completes and **any output contract declared on the
container itself** is satisfied against the shared Work evidence/artifact
surface"; and W1-13 (`:230`) requires reusing the landed stage-output
contracts "at nested leaves **and container closure**". Where brief and
contract disagree, the contract wins; here it does, and the code follows the
contract.

## 3. The four required close-outs

### 1. Coverage membership (#231) — DONE, proven by grep

No new suite file was created. The wave's only test-file change is to an
already-wired suite:

```
$ git diff --name-only 968c9edf..HEAD
00-orient/output/orientation.md
05-baseline/output/baseline.md
10-implement/output/implementation.md
src/domain/workflow.rs
src/runtime/surface.rs
tests/m3_execution.rs

$ grep -rn "m3_execution" scripts/
scripts/coverage/c2-suites.sh:28:cov_stage_begin c2-m3_execution
scripts/coverage/c2-suites.sh:29:cov_run cargo llvm-cov --no-report --test m3_execution --locked || cov_fail "m3_execution failed under instrumentation"
scripts/coverage/README.md:284:| `m3_execution.rs:2260`, `m2:1254` | ...
```

The new unit tests live in `src/`, covered by the lib profile. B3
(`coverage_stage_membership::every_suite_is_wired_or_explicitly_allowlisted`)
was **run**, not reasoned about, and is green (§1 B3).

### 2. `cargo fmt` — DONE, clean. `cargo clippy --all-targets` — **FAILS, pre-existing, NOT fixed here**

```
$ cargo fmt --check
FMT_EXIT=0
```

```
$ cargo clippy --locked --all-targets -- -D warnings
CLIPPY_EXIT=101
error: value assigned to `completed` is never read
   --> tests/s6_scan_front_door.rs:263:25
    |
263 |     let mut completed = None;
    |                         ^^^^ this value is reassigned later and never used
...
272 |             completed = Some(progress);
    |             --------- `completed` is overwritten here before the previous value is read
    |
    = note: `-D unused-assignments` implied by `-D warnings`
error: could not compile `sergeant-rs` (test "s6_scan_front_door") due to 1 previous error
```

**This is recorded as a failure, not worked around** (this stage's contract:
"This stage does not fix a failure it finds"). It is verified pre-existing
and outside this wave, independently of `10-implement`'s claim:

- `git log --oneline 968c9edf..HEAD -- tests/s6_scan_front_door.rs` → empty
  (this wave never touched the file);
- `git show 968c9edf:tests/s6_scan_front_door.rs | sed -n '263p;272p'` is
  byte-identical to the same lines at `HEAD`.

So the lint fires on code the pinned base already carried. It is outside
`00-orient` §6's boundary, and it **will fail the shipping gate**
(`CONTRIBUTING.md:16-31` puts clippy in the same chain as fmt and test).
Carried forward for `20-panel`/Captain to route; not this stage's to fix
and not this change's fault.

### 3. Docs the brief names — NONE NAMED; nothing done, with the reason

The wave brief was read in full (80 lines,
`knowledge/evidence/resources/host-atlas-s6-series/brief-297-sweep-scoping.md`)
and names no documentation deliverable:
`grep -n -iE "doc|changelog|adr" <brief>` returns no match, with the
pattern validated against a known positive in the same file
(`grep -c -i "sweep"` → 11, so the grep is reaching the file). The behavior
changed is internal sweep scoping, not a released CLI surface;
`CONTRIBUTING.md:109-116` puts released behavior in `docs/` and Captain's
estate policy in `AGENTS.md`, neither of which describes the stage-output
sweep's directory-enumeration rule. **Not done, because nothing was named
and nothing was found stale.** If Captain wants the embedded-default
consequence (§4.1) documented, that is a decision above this stage.

### 4. No clock decides correctness — DONE, measured over the diff

```
$ git diff 968c9edf..HEAD -- src/ tests/ | grep -E "^\+" | \
    grep -E "Instant::now|SystemTime::now|BUDGET|deadline|elapsed\(\)|sleep|Duration::from"
NONE
```

Pattern validated rather than trusted: the same pipeline over the same diff
finds `declared_stage_ids` 10 times out of 501 added lines, and the same
pattern matches elsewhere in the repo (`src/lib.rs`, `src/backend/child.rs`,
`src/daemon.rs`, `src/platform/process.rs`, `src/watch.rs`), so a count of
zero here is a real zero. No deadline loop, no `Instant::now() + BUDGET`, no
wall-clock or ratio assertion was added; no poll interval was introduced
either, so there is nothing needing a cadence-only comment.

## 4. Carried forward (recorded, not decided here)

1. **The embedded-default narrowing is real and is now measured.**
   `dirty_teardown_retains_no_stage_output_without_a_bound_workflow_package`
   asserts it: a Work with no bound package retains **no per-stage output
   copies** on dirty teardown. Content still survives in the flat
   `.dirty.patch` (`retain_dirty_writes_a_patch_git_apply_accepts`, green in
   B1); the per-stage labeling is what is lost. This is what the brief's
   "fail closed, both places" requires (J4), so it is not in doubt — it is
   recorded because it is user-visible.
2. **The pre-existing clippy failure above** will red the shipping gate for
   whoever runs it, on a file this wave never touched.

## 5. Completion boundary check

`05-baseline`'s own test command has been run against the implemented change
and its real, verbatim output is recorded — green (§1), with the greenness
made meaningful by two inserted violations that turned it red (§2). Each of
the four close-outs is either done with its evidence shown (#231, fmt,
clock) or named as not done with the reason (clippy: pre-existing failure,
recorded and handed forward; docs: none named by the brief). Nothing was
fixed in place to make this stage look clean. No J0 was reached; not blocked.

# 05-baseline — #297: the close sweep deletes other Works' merged evidence

Recorded against the pinned revision from `00-orient` §1:
`968c9edf1ceb9c214eb9daff8460f132c4dfaef2` (`git rev-parse HEAD` in
`/var/tmp/hats6/sweep` → that SHA at the start of this stage; nothing under
`src/` or `tests/` was changed by this stage). All four runs below were made
**before any implementation change**, on a clean tree apart from this
artifact.

## 1. The discovered test command (J2 — this stage's delegated call)

`CONTRIBUTING.md:16-31` names the commands, quoted verbatim:

> `cargo fmt --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked`
> … `cargo nextest run --locked` is the fast path for the test step (S2 V1c,
> … `cargo nextest run --test <name>` and `cargo nextest run -E 'test(<substring>)'`
> are nextest's equivalents of the two `cargo test` [forms].

So the repository's own named form is nextest with `--test <name>` / `-E`.
Nothing was invented; no J0 was reached.

Per the wave's host discipline the full suite is **not** run locally — CI by
SHA is the exhaustive pass. The relevant suites are the ones that own the two
functions in `00-orient` §6's boundary (`retain_stage_outputs`,
`finalize_sweep`), the module that will carry the declared-id helper
(`src/domain/workflow.rs`), and #231's membership guard.

Every command below is prefixed `TMPDIR=/var/tmp/sgt-test-tmp CARGO_BUILD_JOBS=6`
(lane discipline).

| # | Command | Owns |
|---|---|---|
| B1 | `cargo nextest run --no-fail-fast --lib -E 'test(/^runtime::surface::tests::\|^domain::workflow::tests::/)'` | every unit test of the two modules being changed — the sweep/retain/disposition/container/composed-id behaviour |
| B2 | `cargo nextest run --no-fail-fast --test m3_execution -E 'test(/sweep\|output/)'` | the end-to-end daemon submit→teardown finalize-sweep tests (`tests/m3_execution.rs:1608`, `:1726`) |
| B3 | `cargo nextest run --no-fail-fast --test c2_light -E 'test(coverage_stage_membership)'` | #231 coverage membership (`tests/c2_light.rs:29-30` → `tests/c2_light/coverage_stage_membership.rs`) |

## 2. Verbatim pre-change output

### B1 — `--lib`, `runtime::surface::tests` + `domain::workflow::tests`

Full log: `/var/tmp/sgt-test-tmp/baseline-lib.log` (exit 0, 122 `PASS` lines,
0 fail). Head and the lines that matter here, verbatim:

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
────────────
 Nextest run ID 0509f6b7-3542-4e26-982f-a17e8f813553 with nextest profile: default
    Starting 122 tests across 1 binary (1232 tests skipped)
        PASS [   0.007s] sergeant-rs domain::workflow::tests::a_declared_name_that_does_not_match_its_directory_is_refused
        PASS [   0.007s] sergeant-rs domain::workflow::tests::a_container_whose_sole_stage_is_a_nested_container_closes_innermost_first
...
        PASS [   0.020s] sergeant-rs runtime::surface::tests::stage_scan_within_max_depth_stays_quiet
        PASS [   0.133s] sergeant-rs runtime::surface::tests::dirty_teardown_retains_each_stages_declared_output_artifacts
        PASS [   0.045s] sergeant-rs runtime::surface::tests::stage_scan_past_max_depth_is_not_silent
        PASS [   0.151s] sergeant-rs runtime::surface::tests::dirty_teardown_retains_a_stages_nested_output_artifacts
        PASS [   0.156s] sergeant-rs runtime::surface::tests::dirty_teardown_excludes_gitignored_files_from_retained_output
        PASS [   0.167s] sergeant-rs runtime::surface::tests::dirty_teardown_retains_a_nested_leafs_output
        PASS [   0.168s] sergeant-rs runtime::surface::tests::dirty_teardown_labels_promote_class_output_distinctly_from_evidence
        PASS [   0.093s] sergeant-rs runtime::surface::tests::the_finalize_sweep_reaches_a_nested_leafs_output
        PASS [   1.211s] sergeant-rs runtime::surface::tests::concurrent_surfaces_on_one_repository_all_materialize_and_retire_cleanly
────────────
     Summary [   1.374s] 122 tests run: 122 passed, 1232 skipped
```

### B2 — `--test m3_execution`

```
   Compiling sergeant-rs v0.3.0 (/var/tmp/hats6/sweep)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.76s
────────────
 Nextest run ID 32040cb9-dc0f-40c6-a7a9-42444e5f6815 with nextest profile: default
    Starting 3 tests across 1 binary (58 tests skipped)
        PASS [   4.849s] sergeant-rs::m3_execution a_finalize_sweep_never_launders_unrelated_dirty_content_into_its_own_commit
        PASS [   4.968s] sergeant-rs::m3_execution t10_a_stage_completed_without_its_declared_output_is_reprompted_then_needs_input
        PASS [   4.972s] sergeant-rs::m3_execution the_finalize_sweep_removes_evidence_class_output_and_keeps_promote_class
────────────
     Summary [   4.973s] 3 tests run: 3 passed, 58 skipped
```

(`PIPESTATUS=0`.)

### B3 — `--test c2_light -E 'test(coverage_stage_membership)'` (#231)

```
   Compiling sergeant-rs v0.3.0 (/var/tmp/hats6/sweep)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.33s
────────────
 Nextest run ID da8f72e6-3826-4737-9ed2-39b1fabaebb8 with nextest profile: default
    Starting 1 test across 1 binary (27 tests skipped)
        PASS [   0.006s] sergeant-rs::c2_light coverage_stage_membership::every_suite_is_wired_or_explicitly_allowlisted
────────────
     Summary [   0.006s] 1 test run: 1 passed, 27 skipped
```

**Pre-change state: 126 tests run across the three commands, 126 passed, 0
failed.** That is the fact that matters most here, and it is the fact that
makes the defect a test-honesty problem rather than a regression: **the whole
relevant suite is green while the defect is live.** `00-orient` §3 proved the
defect on the real thing at this same revision (a temporary test through
daemon submit/teardown, exit 100, sweep commit diff containing
`D 99-foreign/output/leftover.md`), then reverted the test. So the baseline is
green *by omission*, not by correctness.

## 3. The absent coverage, evidenced rather than asserted

```
$ grep -rn "declared_stage_ids\|foreign" src/runtime/surface.rs src/domain/workflow.rs tests/m3_execution.rs
src/runtime/surface.rs:156:///    is also what makes the bounded wait a wait for a genuinely foreign
```

The single hit is an unrelated doc comment about bounded waits. There is no
`declared_stage_ids` helper and no test anywhere in the sweep's own modules or
its integration suite that exercises a stage-output directory which is **not**
a declared stage of the running workflow. Nothing currently measures the
behaviour this change exists to fix.

## 4. What this change is expected to move

### 4a. Must newly exist (currently absent — will be red before the fix)

1. **The decisive end-to-end test** of `00-orient` §3's shape, at the level the
   brief requires ("observable state — files on disk and the commit diff — not
   by reasoning about the guard"): a worktree whose base already contains a
   foreign `<stage>/output/` directory that is not among the running workflow's
   declared stage ids, run through real teardown; afterward the foreign
   directory is **still on disk** and the `sergeant: finalize sweep` commit's
   diff contains **no deletion** of it. Today that test fails (exit 100,
   `00-orient` §3).
2. **Fail-closed `retain_stage_outputs`** when `workflow_source` is absent or
   its package will not load: retains nothing, never the unscoped walk.
   `src/runtime/surface.rs:1983-2002` has no such early return today;
   `finalize_sweep` already does at `:2138-2140`.
3. **Coverage membership (#231)**: whichever suite carries the new test is
   wired into `scripts/coverage/c2-suites.sh` or
   `scripts/coverage/c3-spawning-suites.sh` at birth, and B3 stays green. B3 is
   green now, so a red B3 after the change means the wiring was skipped, not a
   pre-existing fault.

### 4b. Must keep passing (currently green — the regression fence)

Every test in B1/B2 above, and specifically these, because they are the ones a
too-aggressive narrowing would break — the exact second, opposite failure mode
`00-orient` §5 identified (J5: `WorkflowDefinition.stages` is leaves-only,
`src/domain/workflow.rs:525-530`, so a leaf-only scope would stop sweeping
**container** outputs the engine gates at `src/runtime/engine.rs:3200-3208`):

| Test | Why it fences this change |
|---|---|
| `runtime::surface::tests::the_finalize_sweep_reaches_a_nested_leafs_output` (`src/runtime/surface.rs:5685`) | composed/nested ids (`10-x/00-y`) must still match after scoping |
| `runtime::surface::tests::dirty_teardown_retains_a_nested_leafs_output` (`:5602`) | same, on the `retain_stage_outputs` path |
| `runtime::surface::tests::dirty_teardown_retains_a_stages_nested_output_artifacts` (`:5552`) | nested artifacts inside a declared stage keep being retained |
| `runtime::surface::tests::dirty_teardown_retains_each_stages_declared_output_artifacts` (`:5362`) | declared stages are still swept/retained at all — catches an over-narrow filter that matches nothing |
| `runtime::surface::tests::dirty_teardown_labels_promote_class_output_distinctly_from_evidence` (`:5448`) | promote-vs-evidence classification is unchanged |
| `m3_execution::the_finalize_sweep_removes_evidence_class_output_and_keeps_promote_class` (`tests/m3_execution.rs:1608`) | the end-to-end sweep still removes a **declared** evidence stage's output |
| `m3_execution::a_finalize_sweep_never_launders_unrelated_dirty_content_into_its_own_commit` (`:1726`) | the F-IN-01 scoped-`git add` boundary is untouched |
| `domain::workflow::tests::declared_output_disposition_defaults_to_evidence` (`src/domain/workflow.rs:2192`) | §1a's "silence promotes nothing" default stays as-is — out of scope per `00-orient` §6 |
| `domain::workflow::tests::a_container_records_the_flat_index_range_of_its_own_leaves` / `a_leaf_can_close_two_containers_at_once_innermost_first` / `output_contracts_resolve_at_a_composed_stage_path` | the container model the declared-id set must include (`00-orient` §5) is unchanged |
| `runtime::surface::tests::stage_scan_past_max_depth_is_not_silent` / `stage_scan_within_max_depth_stays_quiet` (`:6070`, `:6118`) | the walk itself — depth limit and its warning — is explicitly out of scope and must not move |

A container-output regression is the one gap B1/B2 may **not** already fence
end-to-end; if `10-implement`'s design cannot show an existing test that goes
red when container ids are dropped from the declared-id set, that case needs
its own assertion rather than an argument. Flagged for `10-implement`, not
decided here (this stage changes nothing).

### 4c. Explicitly not expected to move

Nothing in `00-orient` §6's out-of-scope list: the walk's traversal shape and
`MAX_STAGE_SCAN_DEPTH`, `declared_output_disposition`'s default,
`copy_declared_output_artifacts`, `capture_dirty_patch`, the finalize commit
identity/message, the SQL boundary. The change **only narrows** what the sweep
touches; any test that goes red by *widening* is a defect in the fix.

## 5. Completion boundary check

The test command is recorded and re-runnable (§1, three commands, each with
its verbatim pre-change output in §2); the baseline run's real state is
captured (126 run / 126 passed / 0 failed, plus the fact that this greenness
coexists with a reproduced defect); the expected-to-move behaviour is stated
in both directions — what must newly exist (§4a) and what must keep passing
(§4b). No J0 was reached. Nothing under `src/`, `tests/`, or `scripts/` was
modified by this stage.

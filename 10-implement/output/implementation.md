# 10-implement — #297: the close sweep deletes other Works' merged evidence

Pinned revision unchanged from `00-orient` §1:
`968c9edf1ceb9c214eb9daff8460f132c4dfaef2`. Nothing re-pinned.

## 1. Seams

Confirmed, not chosen here: `00-orient` §6 states this change's boundary and
`05-baseline` §4a states what must newly exist. Three seams, one
red-green-minimal cycle each, in that order (J3 — settled prior stage
output; `@@test-first`'s J0 "no test at an unconfirmed seam" is satisfied by
that boundary, not by an assumption).

| Seam | Public boundary tested at |
|---|---|
| A | `domain::workflow::declared_stage_ids` — the declared-id set itself |
| B | `finalize_sweep` — end to end through daemon submit/teardown (`tests/m3_execution.rs`), judged on the branch and the finalize commit's diff |
| C | `retain_stage_outputs` via `teardown` — the dirty path's returned `RetainedStageOutput`s and the evidence area on disk |

## 2. Commits

### `ea7a4ff0cdf31c027d7a55fefa8d303c792dc8c0` — seam A

`declared_stage_ids(package_dir) -> Option<BTreeSet<String>>`
(`src/domain/workflow.rs`). **R2**: `WorkflowDefinition::load_dir` already
parses the package and already composes `parent/child` ids exactly as
`composed_stage_id` builds them from a worktree, so the helper reads a
loaded definition rather than re-parsing `workflow.toml`. R3–R7 never
reached. `None`, never an empty set, when the package will not load.

**J5 over the brief's wording** (carried from `00-orient` §5): the declared
set is `stages[].id ∪ containers[].container_id`.
`WorkflowDefinition::stages` is leaves-only
(`src/domain/workflow.rs:525-530`), while a container may declare its own
output contract, gated at `src/runtime/engine.rs:3200-3208` per W1 §4 /
decision W1-13. A leaves-only set would stop sweeping container output —
the same data loss in the other direction.

Red (compile, before the helper existed):

```
error[E0425]: cannot find function `declared_stage_ids` in this scope
    --> src/domain/workflow.rs:2233:19
```
(three occurrences; `error: could not compile `sergeant-rs` (lib test)`)

Non-vacuous beyond the compile error — removing the container half of the
chain turns the test red on the missing id:

```
declared_stage_ids_are_the_running_workflows_leaves_and_containers
  left: ["00-orient", "10-investigate/00-lead", "10-investigate/10-code", "20-implement"]
 right: ["00-orient", "10-investigate", "10-investigate/00-lead", "10-investigate/10-code", "20-implement"]
```

### `c3efca916f12ce19a0b73ba43e18c3461dee3e9b` — seam B

Membership filter plus a fail-closed early return in `finalize_sweep`
(`src/runtime/surface.rs`). The walk, its depth limit, its pruning, and
`declared_output_disposition`'s `Evidence`-on-silence default are all
untouched — the fix is to stop feeding that function foreign ids.

Decisive test, the two-Work sequence through real daemon submit/teardown:
`tests/m3_execution.rs::the_finalize_sweep_leaves_an_undeclared_stage_directory_alone`.
`sweep` declares only `00-evidence`; `99-foreign/output/leftover.md` is
seeded **committed** into the mount before submit, standing in for a prior
Work's already-merged evidence. Judged on observable state: the file on the
branch, the finalize commit's own `--name-status` diff, and the
retained-evidence area.

Red:

```
thread 'the_finalize_sweep_leaves_an_undeclared_stage_directory_alone'
panicked at tests/m3_execution.rs:93:5:
git ["show", "sergeant/01M1AS7RN039C8PWPE8BJ35ED1:99-foreign/output/leftover.md"]
failed in .../repos/solo: fatal: path '99-foreign/output/leftover.md'
exists on disk, but not in 'sergeant/01M1AS7RN039C8PWPE8BJ35ED1'
```

That is the reported defect: the prior Work's evidence deleted from the
branch this Work would merge. The same test also asserts `00-evidence`
**is** still swept and that the finalize commit's diff still contains its
removal, so a filter that matched nothing cannot pass it.

**#231:** no new suite — `m3_execution` is already wired at
`scripts/coverage/c2-suites.sh:28-29`. B3
(`cargo nextest run --test c2_light -E 'test(coverage_stage_membership)'`)
was run after the change and is green (§3).

Fixture repair in the same commit:
`the_finalize_sweep_reaches_a_nested_leafs_output`'s `workflow-package` was
a bag of `output/README.md` files with **no `workflow.toml`** — not a
workflow package, so under a declared-id scope it declares nothing and fails
closed. It is now a real nested package (new test helper
`write_workflow_package`), which is what `workflow_source` always is in
production, and it becomes the container fence `05-baseline` §4b asked for:
dropping `containers[].container_id` from the declared set makes it red on

```
a container's own declared output is swept too (W1 §4): ["10-investigate/00-lead"]
```

### `f9cb24b70eb1dae2627acb3d1114836d8b0f55b3` — seam C

`retain_stage_outputs`: the same membership filter, plus the fail-closed
early return `finalize_sweep` already had, replacing the
`workflow_source.map(...).unwrap_or(Evidence)` fail-open.

Red (both new tests):

```
dirty_teardown_retains_no_stage_output_without_a_bound_workflow_package
panicked at src/runtime/surface.rs:5594:9:
#297: with no package to declare stage ids, the sweep retains nothing:
[RetainedStageOutput { stage: "20-panel", path: ".../01NOPACKAGE/solo.output/20-panel", bytes: 18, disposition: Evidence }]

dirty_teardown_leaves_an_undeclared_stage_directory_out_of_retained_output
panicked at src/runtime/surface.rs:5545:9:
assertion `left == right` failed: #297: only the running workflow's own declared stage is this Work's to retain
  left: ["20-panel", "99-foreign"]
 right: ["20-panel"]
```

Fixture repair, six existing dirty-teardown tests: they bound no workflow
package at all (`teardown_of` passes `None`), which no real run does —
`teardown_next` (`src/runtime/engine.rs:5021-5025`) passes the resolved
workflow's own source directory. One helper (`teardown_declaring`, R2 over
duplicating the writes) gives each the package that declares the stages it
seeds. Each test still asserts exactly what it asserted before.

## 3. State after the change

| # | Command | Result |
|---|---|---|
| B1 | `cargo nextest run --no-fail-fast --lib -E 'test(/^runtime::surface::tests::\|^domain::workflow::tests::/)'` | `126 tests run: 126 passed` (122 baseline + 4 new) |
| B2 | `cargo nextest run --no-fail-fast --test m3_execution` (whole suite, not just the filter) | `62 tests run: 62 passed` |
| B3 | `cargo nextest run --no-fail-fast --test c2_light -E 'test(coverage_stage_membership)'` | `1 test run: 1 passed` — #231 green |
| — | `cargo fmt --check` | clean |

Every test in `05-baseline` §4b's regression fence is inside B1/B2 and
passes. The full suite was deliberately not run locally (CI by SHA is the
exhaustive pass).

## 4. Conflicts

None. No merge or rebase occurred; `@@resolve-conflicts` was not reached.

## 5. Carried forward for `20-panel` (not decided here)

1. **Behaviour consequence, stated not glossed.** Failing closed in
   `retain_stage_outputs` means a Work on the **embedded default** workflow
   (`workflow_source` is `None` by construction, `engine.rs:5021-5025`) now
   gets no per-stage retained-output copies on a dirty teardown. Content is
   not lost — `retain_dirty` still captures the flat `.dirty.patch`, which
   `retain_dirty_writes_a_patch_git_apply_accepts`
   (`src/runtime/surface.rs:5353`) proves reproduces tracked and untracked
   content byte for byte — what is lost is the per-stage *labeling*. This is
   what #297's own fix text and the brief both require in both places (J4),
   and `00-orient` §6.3 holds it as this change's boundary; it is recorded
   here because it is a real user-visible narrowing, not because it is in
   doubt.
2. **Pre-existing clippy failure, untouched by this wave.**
   `cargo clippy --locked --all-targets -- -D warnings` fails on
   `tests/s6_scan_front_door.rs:272` (`unused_assignments`, `completed` is
   overwritten before it is read). Verified pre-existing: it reproduces with
   this stage's working-tree changes stashed, and
   `git diff --name-only 968c9edf..HEAD` shows this wave never touched that
   file. Out of `00-orient` §6's boundary, so it was not fixed here — but it
   will fail the shipping gate for whoever runs it.

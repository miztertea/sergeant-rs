# Fix — #234 dirty-patch trailing-newline corruption

## Status note

This Work branch's `git log` already carries the fix and its regression
tests, landed by an earlier turn of this same Work (`56927a14`, plus
evidence commits `f16ba9e4`, `e0718598`, `eacee9a7` from the preceding
stages). This turn re-verified the seam, the fix, and the tests against
the confirmed cause in `../30-instrument/output/probes.md`, and ran the
required gates. No further code changes were needed.

## Seam

`capture_dirty_patch` (`src/runtime/surface.rs:1671`) is the real call
site: it drives `git diff --cached --binary` through `git()`
(`src/runtime/git.rs:116`), whose `String::from_utf8_lossy(...).trim()`
unconditionally strips the diff's mandatory trailing `\n` (and, per H2 in
`30-instrument`, lossily re-encodes non-UTF-8 diff content). This is a
correct seam, not a shallow one: the regression test drives the real
`teardown()` path against a real `git worktree`-backed fixture, captures
the actual `.dirty.patch` file `sgt work reap` would hand to an operator,
and validates it the only way it is ever consumed in practice — `git
apply` against an independent checkout, not a re-parse of the string in
memory.

## Fix (`56927a14`)

`git_bytes(dir, args) -> Result<Vec<u8>, GitError>` added to
`src/runtime/git.rs` alongside `git()`/`git_verbatim()`: returns
`output.stdout` exactly as Git wrote it — no `.trim()`, no UTF-8
round-trip. `stderr` on the error path is still lossy-trimmed, matching
`git()`'s existing error handling; only stdout must be byte-exact.

`capture_dirty_patch` switched to `git_bytes` for the diff capture; the
resulting `Vec<u8>` flows straight to `write_atomic` and never passes
through a `String`. `PatchInfo.bytes`/`is_empty` semantics unchanged.

## Regression coverage

**Unit seam** — `retain_dirty_writes_a_patch_git_apply_accepts`
(`src/runtime/surface.rs:4646`, in the existing `mod tests`, reusing the
`repo()`/`materialize`/`teardown_of` fixtures used by
`retained_bindings_reports_a_captured_patch_and_a_retained_error_directory`):
dirties a materialized worktree (a tracked-file edit whose diff's last
content line ends `]` with no trailing whitespace of its own, plus one
untracked file), drives real `teardown()` to `RetainedDirty` with a
captured patch, then asserts (a) the patch file's raw last byte is
`b'\n'`, (b) `git apply --check` succeeds against a *second*, independent
clean checkout of the same source repo at the same base commit — never
the worktree the diff came from, (c) `git apply` actually applies there
and the resulting tracked file is byte-identical to what was written into
the dirtied worktree, and the untracked file lands too.

Watched failing first (pre-fix, `git()`-based capture: patch missing its
trailing `\n`, `git apply --check` rejects it with "corrupt patch at line
N" at end-of-file — reproduced during `10-reproduce-and-minimize` and
`20-hypothesize`), then passing post-fix.

**E2E** — `tests/m7_docker_executor.rs`'s `walk_for_marker` helper
(~line 1518) now runs `git apply --stat` and `git apply --check` against
every `*.dirty.patch` it finds while grepping the submit → execute →
teardown path for its retention marker, pinning patch validity end to
end, matching #234's own acceptance criteria (22/22 real-estate patches).

## Feedback-loop re-run

`00-build-feedback-loop` re-run against the original (un-minimized)
scenario:

- `cargo fmt --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean (forced a fresh
  check by touching `git.rs`/`surface.rs`; no cached false-clean).
- `cargo test --lib runtime::surface::tests` — 40/40 pass, including
  `retain_dirty_writes_a_patch_git_apply_accepts`.
- `cargo test --test m7_docker_executor` — 18/18 pass, including the
  `walk_for_marker` `git apply` assertions.
- `cargo test --test m6_surfaces` — 48/48 pass in isolation (2 flaked
  under full-suite parallelism, see below).
- `cargo test` (full suite) — 10 failures, all in `tests/m2_daemon_api.rs`
  plus 2 in `tests/m6_surfaces.rs` under full parallelism; **all are
  pre-existing environmental flakes unrelated to this fix**, not
  regressions:
  - Confirmed by isolating each failing test and re-running it alone:
    most pass clean (`concurrent_stale_replacement_leaves_the_surviving_daemon_discoverable`,
    `retry_success_prints_the_human_readable_line`,
    `stale_descriptor_is_replaced_but_ambiguous_descriptor_fails_closed`,
    `t8_two_concurrent_auto_spawns_one_survivor_both_commands_complete`,
    `the_data_dir_guard_reaps_the_daemon_a_client_command_spawns`, both
    `m6_surfaces` failures).
  - The remainder
    (`r_mvp1_7_sgt_turn_cap_env_var_reaches_a_real_spawned_daemon`,
    `resolve_data_dir_falls_back_through_sgt_data_dir_then_xdg_then_home`,
    `t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed`,
    `t7b_cli_status_show_and_cancel_through_the_binary`,
    `work_list_human_form_prints_the_empty_and_populated_branches`) fail
    even in isolation in *this specific shell*, but for a documented,
    unrelated reason: this session runs inside a live `sgt claude`
    session with ambient `SGT_DATA_DIR`/`SGT_ESTATE_ROOT`/
    `SGT_ORIGIN_CLIENT` env vars set, which is exactly the
    "Sergeant-managed checkout of itself" class of environmental
    interference `docs/environments/macbook.md` already documents for
    `tests/m8_estate_cli.rs` ("assert an isolated tempdir data dir but
    pick up the ambient real one instead. Not a code defect."). None of
    these ten tests touch `src/runtime/git.rs` or
    `src/runtime/surface.rs`, and none exercise `capture_dirty_patch` or
    the dirty-patch retention path this fix changes.
  - Recorded here rather than silently ignored, per this stage's
    completeness requirement; flagged for `60-re-verify-and-postmortem`
    as an environment-hygiene item (a test env that scrubs ambient
    `SGT_*` vars would remove this class of flake), not as a #234 defect.

## Disposition

Fix and regression coverage confirmed correct and already committed
(`56927a14`, `f16ba9e4`, `e0718598`, `eacee9a7`). No new commit required
by this stage; this artifact records the re-verification.

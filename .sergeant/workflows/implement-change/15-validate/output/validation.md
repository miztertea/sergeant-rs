# Validation — split-hardening W5 (#259, #262)

## Test command (from `05-baseline`, run unmodified)

```
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Run against the committed implementation (`e4752d91`, tree clean, `git status --short`
empty before this run).

## `cargo fmt --check`

```
$ cargo fmt --check
```

Exit code: `0`. No output (nothing to reformat).

## `cargo clippy --locked --all-targets -- -D warnings`

```
$ cargo clippy --locked --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
```

Exit code: `0`. Clean — no warnings, no errors.

## `cargo test --locked`

Exit code: non-zero. Verbatim per-binary summary (in run order; `cargo test` runs
target binaries sequentially and, per its default fail-fast-per-target behavior
with no `--no-fail-fast` flag, stopped after the first binary with failures —
`tests/m2_daemon_api.rs`, third-to-last alphabetically among 37 test files;
`m3_*` through the rest of the suite did not get a chance to run this
invocation):

```
     Running unittests src/lib.rs (target/debug/deps/sergeant_rs-...)
test result: ok. 932 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 22.41s

     Running unittests src/main.rs (target/debug/deps/sgt-...)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/a1_floor_awareness.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/a4_blob_ref_pinning.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s

     Running tests/agy_backend.rs
test result: ok. 81 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 5.25s

     Running tests/agy_routing.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.46s

     Running tests/c11_injectable_git.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/c4_repo_lock.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.26s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.42s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running tests/codex_backend.rs
test result: ok. 75 passed; 0 failed; 15 ignored; 0 measured; 0 filtered out; finished in 10.31s

     Running tests/codex_routing.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.16s

     Running tests/coverage_stage_membership.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/e_admission_uses_no_network_git.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.69s

     Running tests/e_git_admission.rs
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.04s

     Running tests/e_sweep_uses_only_local_git.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.01s

     Running tests/e_work_sweep.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.20s

     Running tests/estate_routes.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.64s

     Running tests/f_doctrine_skew.rs
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.45s

     Running tests/i9_floor_pinning.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.52s

     Running tests/m10_harness.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/m1_event_core.rs
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s

     Running tests/m2_daemon_api.rs
test result: FAILED. 50 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 73.25s

error: test failed, to rerun pass `--test m2_daemon_api`
```

### `m2_daemon_api.rs` failures, verbatim

```
failures:

---- concurrent_stale_replacement_leaves_the_surviving_daemon_discoverable stdout ----

thread 'concurrent_stale_replacement_leaves_the_surviving_daemon_discoverable' panicked at tests/m2_daemon_api.rs:2063:5:
client A failed: sgt: spawned a daemon for /var/tmp/sgt-rs-tests/.tmpI0QRK9 but it did not become healthy within 10s (see daemon.log in the data dir)

---- r_mvp1_7_sgt_turn_cap_env_var_reaches_a_real_spawned_daemon stdout ----

thread 'r_mvp1_7_sgt_turn_cap_env_var_reaches_a_real_spawned_daemon' panicked at tests/m2_daemon_api.rs:5090:5:
sgt run: sgt: spawned a daemon for /var/tmp/sgt-rs-tests/.tmpfsp9Up but it did not become healthy within 10s (see daemon.log in the data dir)

---- resolve_data_dir_falls_back_through_sgt_data_dir_then_xdg_then_home stdout ----

thread 'resolve_data_dir_falls_back_through_sgt_data_dir_then_xdg_then_home' panicked at tests/m2_daemon_api.rs:2671:5:
sgt run: sgt: spawned a daemon for /tmp/.tmp4jpJ2n but it did not become healthy within 10s (see daemon.log in the data dir)

---- retry_success_prints_the_human_readable_line stdout ----

thread 'retry_success_prints_the_human_readable_line' panicked at tests/m2_daemon_api.rs:2579:5:
sgt retry: sgt: spawned a daemon for /var/tmp/sgt-rs-tests/.tmpeCB4jA but it did not become healthy within 10s (see daemon.log in the data dir)

---- stale_descriptor_is_replaced_but_ambiguous_descriptor_fails_closed stdout ----

thread 'stale_descriptor_is_replaced_but_ambiguous_descriptor_fails_closed' panicked at tests/m2_daemon_api.rs:1949:5:
stale descriptor must be replaced: sgt: spawned a daemon for /var/tmp/sgt-rs-tests/.tmpjFgllu but it did not become healthy within 10s (see daemon.log in the data dir)

---- t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed stdout ----

thread 't7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed' panicked at tests/m2_daemon_api.rs:1710:5:
sgt run failed: sgt: spawned a daemon for /var/tmp/sgt-rs-tests/.tmpI2N20A but it did not become healthy within 10s (see daemon.log in the data dir)

---- t8_two_concurrent_auto_spawns_one_survivor_both_commands_complete stdout ----

thread 't8_two_concurrent_auto_spawns_one_survivor_both_commands_complete' panicked at tests/m2_daemon_api.rs:1868:5:
client A failed: sgt: spawned a daemon for /var/tmp/sgt-rs-tests/.tmptqpzg8 but it did not become healthy within 10s (see daemon.log in the data dir)

---- t7b_cli_status_show_and_cancel_through_the_binary stdout ----

thread 't7b_cli_status_show_and_cancel_through_the_binary' panicked at tests/m2_daemon_api.rs:1779:5:
sgt run failed: sgt: spawned a daemon for /var/tmp/sgt-rs-tests/.tmpXZj47T but it did not become healthy within 10s (see daemon.log in the data dir)

---- the_data_dir_guard_reaps_the_daemon_a_client_command_spawns stdout ----

thread 'the_data_dir_guard_reaps_the_daemon_a_client_command_spawns' panicked at tests/m2_daemon_api.rs:1574:5:
sgt run failed: sgt: spawned a daemon for /var/tmp/sgt-rs-tests/.tmpIfKLYV but it did not become healthy within 10s (see daemon.log in the data dir)

---- work_list_human_form_prints_the_empty_and_populated_branches stdout ----

thread 'work_list_human_form_prints_the_empty_and_populated_branches' panicked at tests/m2_daemon_api.rs:1530:9:
the bare daemon never published a descriptor

failures:
    concurrent_stale_replacement_leaves_the_surviving_daemon_discoverable
    r_mvp1_7_sgt_turn_cap_env_var_reaches_a_real_spawned_daemon
    resolve_data_dir_falls_back_through_sgt_data_dir_then_xdg_then_home
    retry_success_prints_the_human_readable_line
    stale_descriptor_is_replaced_but_ambiguous_descriptor_fails_closed
    t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed
    t7b_cli_status_show_and_cancel_through_the_binary
    t8_two_concurrent_auto_spawns_one_survivor_both_commands_complete
    the_data_dir_guard_reaps_the_daemon_a_client_command_spawns
    work_list_human_form_prints_the_empty_and_populated_branches

test result: FAILED. 50 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 73.25s
```

## Result

**FAIL.** `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D
warnings` are both clean. `cargo test --locked` fails at `tests/m2_daemon_api.rs`
with 10/60 tests failing, every one of them a real-daemon health-check timeout
("did not become healthy within 10s") or its direct consequence (the descriptor
never publishing). None of the 10 failing tests, and no line in any panic
message, names `codex.rs`, `permission_mode`, `cli.rs`'s doctor code, or any
other file this Work's diff touches — `m2_daemon_api.rs` is not among the files
changed in `e4752d91`. This matches `10-implement`'s own observation of the same
intermittent failure class with a differing exact test set across repeated runs,
and `05-baseline`'s independent confirmation that the pinned base commit
(`a126dbd2`) is green in GitHub's recorded check-runs. Because `cargo test`
without `--no-fail-fast` stops after the first failing binary, `m3_*` onward
(the remaining ~15 of 37 test files) did not run in this invocation and their
state is not recorded here.

This is the honest, verbatim recorded outcome of the baseline command against
the implemented change — not worked around or silently retried to pass. Whether
this environmental flakiness blocks closing the Work, and whether a
`--no-fail-fast` (or per-binary) re-run to observe the rest of the suite is
warranted, is left to `20-panel` / `35-re-verify`, per this stage's contract.

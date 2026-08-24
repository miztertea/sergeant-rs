# Fixes — split-hardening W5 (#259/#262), from `25-refute`'s confirmed set

Six findings entered this stage at `status: confirmed` (all six survived
`25-refute`'s attack; none was `refuted`). Five are fixed, in the commits
named below. One (`F-SF-02`) is recorded `confirmed` and deliberately
unfixed, with the reason stated under it.

## Fixed

### F-IV-01 + F-SI-01 — resume's inert git-admin-dir resolution
**Commit:** `148f24e3` — "fix(codex): stop resolving git admin dir on resume; factor the duplicated grant-resolve pattern"

Both findings share one root cause (`src/backend/codex.rs`'s `resume()`,
formerly ~4483-4514): the resolution was both a new, reachable
daemon-restart failure mode (F-IV-01: a `.git` file read that can fail for
reasons unrelated to reattach viability — worktree pruned/moved, transient
FS issue — now aborts reattach) and dead output (F-SI-01: `turns: 1` on
this path means the `execution.turns == 0` argv gate that would read it
never fires). Fixing the dead-output half automatically fixes the
new-failure-mode half: `resume()` no longer calls `git_worktree_admin_dirs`
at all, and stores `Vec::new()` for the field with a comment explaining why
(nothing downstream ever reads it on this path). This is a straight
removal, not a workaround — the call bought no capability, only cost.

### F-SI-02 — duplicated resolve-and-refuse pattern
**Commit:** `148f24e3` (same commit as above)

Factored `git_worktree_admin_dirs(&bindings).map_err(|reason|
self.err_failed(git_admin_dir_refusal(&reason)))` into one private helper,
`CodexBackend::resolve_git_worktree_admin_dirs`, called identically from
`prepare()`, `launch_exec()`, and `launch_appserver()` — the three call
sites where the grant is genuinely live. (`resume()` no longer calls it at
all, per the F-IV-01/F-SI-01 fix above, so the duplication count dropped
from four sites to three before this factor and to one after.) Mirrors the
codebase's own existing precedent (`launch_config(...)`, already factored
identically for an analogous fallible resolve-and-wrap step), which the
refuter seat named directly as undercutting any "not worth factoring"
defense.

### F-TH-01 — mislabeled "regression for `common_dir_finding`" test
**Commit:** `6693aa3e` — "test: add real common_dir_finding regression test; add live network-bind contract test"

Added `common_dir_finding_reports_no_mismatch_for_a_genuine_linked_worktree`
to `src/runtime/surface.rs`'s own `#[cfg(test)] mod tests` — the only place
with access to the private `common_dir_finding` function — which
constructs a `RepositoryBinding` from a real linked worktree and calls
`common_dir_finding` directly, in both the Phase-B (`canonical_common_dir:
Some(..)`) and pre-Phase-B (`None`, live re-derive) shapes. A genuine
regression in the function (wrong comparison, wrong field, always-`Some`)
now fails this test.

Reworded `tests/codex_backend.rs`'s
`the_git_admin_dir_grant_does_not_disturb_the_worktrees_own_common_dir_identity`
doc comment to stop claiming it regression-tests `common_dir_finding`
itself — it doesn't call it and structurally cannot (private, cross-crate)
— and instead states plainly what it does check (the underlying git fact
the function depends on) and points to the new unit test as the actual
regression coverage. No assertions in this test changed; it was already a
real, passing check of a real fact, just mislabeled.

### F-SF-01 — no live test proves the loopback bind actually works
**Commit:** `6693aa3e` (same commit as above)

Added `live_codex_actor_binds_loopback_when_network_access_is_configured`
to `tests/codex_backend.rs`, gated behind the same `SERGEANT_CODEX_TESTS=1`
opt-in and `#[ignore]` pattern as the existing `live_codex_*` suite
(mirrors `live_codex_actor_commits_to_the_works_own_branch`'s shape for
#259). It sets `CodexConfig::workspace_write_network_access = true`,
dispatches a real codex actor whose intent is to bind `127.0.0.1:0` via a
one-line Python probe and write `BIND_OK` to a scratch file only on
success, then asserts the file contains exactly that — a measured proof
the sandbox knob actually grants the bind, not just that the `-c` flag is
composed (which `network_access_is_absent_by_default_and_present_when_configured`
already covered, stub-only). This test is skipped cleanly without the env
var and the codex CLI, same as every other live test in this file — it is
not run as part of the gates below.

## Confirmed and deliberately unfixed

### F-SF-02 — #259's fail-closed check is structural, not keyed on `permission_mode`
**Status:** `confirmed`, not fixed. **Reason:** the only available fix is to
make codex's PREPARE-time fail-closed check read `permission_mode` and key
its refusal on that option's value — i.e., to give `permission_mode` real
effect for the codex backend. That is explicitly out of scope for this
wave ("Out of scope: ... making permission_mode do anything for codex" —
dispatch prompt) and would also directly contradict this same wave's #262
honesty fix (`backend_consumes_permission_mode` in `src/cli.rs`, which
truthfully states codex never reads `permission_mode` at all). Making the
fix would require reversing that statement, which is a change to the
intent's own declared scope and invariants, not a fix within it — the
`@@fix-confirmed` package's own boundary ("A fix that requires changing
what the change is for ... is not made here"). The current structural
check (git-admin-dir resolvability) is the correct, honest substitute
already in place: it is what #259's actual mechanism can fail-closed on,
and both `cli.rs`'s doctor honesty change and this check's own doc comments
already say so plainly — nothing about the implementation misrepresents
itself as permission_mode-keyed. No code change accompanies this entry.

## Re-run validation (05-baseline's command, after the last fix commit)

```
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

- `cargo fmt --check`: exit 0, no diff.
- `cargo clippy --locked --all-targets -- -D warnings`: exit 0, no warnings.
- `cargo test --locked`: fails at `tests/m2_daemon_api.rs` with the same 10
  tests failing as `15-validate`'s own baseline run against the committed
  implementation (`e4752d91`) — `concurrent_stale_replacement_leaves_the_
  surviving_daemon_discoverable`, `r_mvp1_7_sgt_turn_cap_env_var_reaches_a_
  real_spawned_daemon`, `resolve_data_dir_falls_back_through_sgt_data_dir_
  then_xdg_then_home`, `retry_success_prints_the_human_readable_line`,
  `stale_descriptor_is_replaced_but_ambiguous_descriptor_fails_closed`,
  `t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed`,
  `t7b_cli_status_show_and_cancel_through_the_binary`,
  `t8_two_concurrent_auto_spawns_one_survivor_both_commands_complete`,
  `the_data_dir_guard_reaps_the_daemon_a_client_command_spawns`,
  `work_list_human_form_prints_the_empty_and_populated_branches` — every
  one a real-daemon health-check timeout ("did not become healthy within
  10s") or its direct consequence, in a file this stage's diff never
  touches. Re-run in isolation, all ten pass (confirmed for a sample:
  `appserver_a_stray_notification_for_the_displaced_turn_never_taints_the_
  new_one` in `codex_backend.rs`, which failed in the full-suite run above
  and passed alone) — consistent with `15-validate`'s own characterization
  of this as environment-load flakiness in a file untouched by the Work,
  not a regression from any fix in this stage. Every other test binary,
  including `codex_backend.rs`'s 91 non-live tests and `src/runtime/
  surface.rs`'s new unit test, passed.

## Follow-up intents noticed but not acted on (scope: not this stage)

- `launch_appserver`'s `roots` construction (tests/codex_backend.rs
  reference: `src/backend/codex.rs` ~3595-3604) rebuilds
  `bindings_outside_cwd` output into `String`s by hand in three places
  across the file; a small shared helper could remove the repetition. Not
  a confirmed finding — noticed in passing while touching adjacent code —
  so left alone per this stage's no-opportunistic-change rule.

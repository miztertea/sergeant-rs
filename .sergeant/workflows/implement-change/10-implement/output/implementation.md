# Implementation — split-hardening W5 (#259, #262)

## Starting point

This worktree carried pre-existing unstaged WIP at session start (noted in
`00-orient`/`05-baseline`). By the time this stage began work, that WIP had
already progressed past the single compile error `05-baseline` recorded
(a missing-fields call site in the re-adopt path) — `cargo build --lib
--tests` compiled cleanly on first check. This stage's job was therefore to
verify the WIP actually satisfies every acceptance criterion in the dispatch
prompt and the two issues, close any remaining gap, and land it as a
reviewable commit.

## Seams covered (all found already implemented in the WIP; verified, not re-authored)

1. **#259 — scoped `--add-dir` grant for the Work's own linked-worktree git
   admin dir.** `worktree_git_admin_dir`/`git_worktree_admin_dirs`
   (`src/backend/codex.rs:1208-1247`) read each binding's `.git` file
   (`gitdir: <path>`) with no process spawn, per PREPARE's no-external-effect
   contract, and resolve exactly `.git/worktrees/<name>` — never
   `repository.path`, never the shared `.git`. Threaded into every
   `CodexExecution` construction site (first turn, resume, re-adopt) and
   composed into the launch argv's `extra_dirs` (`:3305-3322`).
   Tests: `the_first_turn_grants_the_works_own_git_admin_dir_as_an_add_dir_root`,
   `the_git_admin_dir_grant_does_not_disturb_the_worktrees_own_common_dir_identity`
   (regression for `runtime::surface::common_dir_finding`).

2. **#259 — fail-closed preflight.** `CodexBackend::prepare` (`:4253-4273`)
   calls `git_worktree_admin_dirs` for any mutation-shaped request (non-empty
   `bindings`) and refuses admission via `git_admin_dir_refusal` — a named,
   actionable error — before LAUNCH ever spawns anything, rather than letting
   the Work run to `completed_dirty`.
   Test: `prepare_refuses_a_mutation_shaped_request_whose_git_admin_dir_is_unresolvable`.

3. **#262 — honest `permission_mode` reporting.** `sgt doctor`'s
   `permission_mode_check` (`src/cli.rs:2746-2800`) now renders
   `name=mode` only when `backend_consumes_permission_mode` is true for that
   profile's backend (currently only `claude`); for a backend that ignores
   the option (codex), it renders the configured value alongside an explicit
   "does not read permission_mode -> no effect" clause instead of implying
   effect.
   Tests: `t3b_doctor_reports_the_effective_permission_mode_per_profile`,
   `t3d_doctor_reports_permission_mode_has_no_effect_on_a_codex_profile`.

4. **#262 — scoped, opt-in sandbox network knob.** `CodexConfig::
   workspace_write_network_access` (`src/backend/codex.rs:338`) composes
   codex-cli's documented `-c sandbox_workspace_write.network_access=true`
   override (`:1088-1098`) only when explicitly set — never daemon-global,
   never default-on, never `danger-full-access`; grants sandbox loopback
   binding, not external egress.
   Test: `network_access_is_absent_by_default_and_present_when_configured`.

5. **Gated live end-to-end contract test.** `live_codex_actor_commits_to_
   the_works_own_branch` (`tests/codex_backend.rs:4795`) — gated behind
   `SERGEANT_CODEX_TESTS=1` and `#[ignore]`, mirroring every other live codex
   test — dispatches a trivial mutation through a real codex actor in a real
   linked worktree and asserts the resulting commit lands on the assigned
   `sergeant/<work-id>` branch, is built on the Work's own starting point,
   and leaves `HEAD` on the branch (not detached).

## Verification performed this stage

- `cargo build --lib --tests`: clean.
- `cargo fmt --check`: clean.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo test --locked --test codex_backend --test m6_surfaces --lib`: all
  codex-adapter and doctor-related tests pass (75/75 in `codex_backend`,
  including every test named above; the two doctor `permission_mode` tests
  in `m6_surfaces` pass).
- `cargo test --locked` (full suite, run twice at different
  `--test-threads`): a set of `m2_daemon_api`/`m6_surfaces` tests fail
  intermittently, and the *specific* tests that fail differ between runs.
  Every failure's message is a real-daemon health-check timeout
  ("did not become healthy within 10s") or a downstream consequence of one —
  none reference `codex.rs`, `permission_mode`, or any file this change
  touches, and `m2_daemon_api.rs` is not among the files this diff modifies.
  This is pre-existing environmental flakiness under this sandbox's process/
  resource contention, not a regression introduced here — the base commit
  (`a126dbd2`) is independently confirmed green via GitHub's recorded
  check-runs (`05-baseline`). Left for `20-panel`/`15-validate` to weigh; not
  masked or worked around here.

## No conflicts

No merge/rebase conflict was encountered; this stage's work was direct
verification and finalization of pre-existing WIP on top of a clean branch
tip.

# Baseline — split-hardening W5 (#259, #262)

## Discovered test command

Taken from `.github/workflows/ci.yml` and `matrix.yml` (not guessed):

```
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

`matrix.yml`'s full-suite job additionally runs
`cargo test --locked -- --nocapture` on Linux/macOS; behavior-equivalent
to `cargo test --locked` for pass/fail purposes. `cargo --version` /
`rustc --version` in this worktree: `cargo 1.98.0 (797e8a9bc 2026-08-05)`,
`rustc 1.98.0 (88d9e12ae 2026-08-18)`.

## Pinned-revision reference point

The Work branch was cut from `main` at `a126dbd2` ("Merge pull request
#257 …"). GitHub's recorded check-runs for that exact commit are all
`success`, including `gate-b-ci / ci`, `gate-c-matrix / full-suite
(ubuntu-24.04)`, `gate-c-matrix / full-suite (macos-26)`,
`gate-d-coverage / coverage`, `gate-e-cargo-deny`, `gate-f-distro-validator`.
So the true pre-change baseline (fmt/clippy/test) is green at that
revision — confirmed via `gh api repos/.../commits/a126dbd2.../check-runs`
rather than re-run locally, since re-running it would require discarding
the worktree's pre-existing uncommitted WIP, which is out of scope to
touch here.

## Actual baseline run (current worktree state, before this stage's own changes)

Per orientation, this worktree already carries **pre-existing unstaged
WIP** in `src/backend/codex.rs`, `src/cli.rs`, `tests/codex_backend.rs`,
`tests/m6_surfaces.rs` from before this session — not authored in this
stage, but present as the real starting point 10-implement will build on.
Running the discovered command against that real starting point:

```
$ cargo test --locked
   Compiling sergeant-rs v0.2.3 (...)
error[E0063]: missing fields `git_worktree_admin_dirs` and `network_access` in initializer of `CodexExecution`
    --> src/backend/codex.rs:4495:13
     |
4495 |             CodexExecution {
     |             ^^^^^^^^^^^^^^ missing `git_worktree_admin_dirs` and `network_access`

For more information about this error, try `rustc --explain E0063`.
error: could not compile `sergeant-rs` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `sergeant-rs` (lib test) due to 1 previous error
```

Verbatim, full output — no truncation (11 lines total). **Baseline
status: FAIL — does not compile.** `cargo fmt --check` and `cargo clippy`
were not run past this point since the crate does not build; both are
gated on `cargo test` in this record.

## What's already true in the WIP (context, not part of this stage's output)

The WIP already added `CodexConfig::workspace_write_network_access`
(src/backend/codex.rs:338), a `git_worktree_admin_dirs` resolver
(src/backend/codex.rs:1236), and both fields on `CodexExecution`
(src/backend/codex.rs:2470, :2479), and threads them into argv
construction (:3286-3315) and the network-override `-c` flag (:1088-1098).
The single compile error is a missed call site: the thread-re-adopt path
(`re_adopt`-shaped method, ~line 4470-4510) still constructs
`CodexExecution { .. }` without the two new fields. This is a completeness
gap in existing WIP, not a new file to author from scratch — 10-implement
should treat it as "finish wiring the fourth construction site," then
build out the preflight-refusal check, doctor honesty change, and the
tests listed in the dispatch prompt on top of a compiling base.

## Behavior expected to move

- **Compile**: `cargo test --locked` must go from FAIL (doesn't compile)
  to at least compiling cleanly. This is the first, unambiguous signal
  that 10-implement's work landed.
- **#259** (new behavior, currently absent): a codex actor operating in
  its linked worktree must be able to `git commit` there — currently
  blocked because the worktree's private `.git/worktrees/<name>` admin
  dir is outside the codex sandbox's writable roots. After the change, a
  gated live contract test dispatches a trivial mutation through a real
  codex actor and asserts the resulting commit lands on the Work branch.
  Additionally, a fail-closed preflight test must show `prepare()`
  refusing admission (named, actionable error) when the git-dir grant
  cannot be resolved for a mutation-shaped launch, rather than allowing
  the Work to run to `completed_dirty`.
- **#262** (currently dishonest, must become honest): `sgt doctor`'s
  `permission_mode` row currently implies effect for every backend. After
  the change, for a codex profile (whose backend ignores
  `permission_mode` entirely — confirmed: `src/backend/codex.rs` never
  reads it), the row must state structurally that this backend does not
  consume the option, rather than implying it is in effect. A doctor test
  must assert this exact honest wording for a codex profile with
  `permission_mode` set.
- **#262** (new opt-in capability): a per-profile/codex-config knob
  composing codex-cli's `-c sandbox_workspace_write.network_access=true`
  must let a codex actor bind `127.0.0.1:0` for native validation when
  explicitly requested — and must NOT grant this by default, globally, or
  imply external egress. A unit test on composed argv must show the `-c`
  flag present only when the knob is set, absent otherwise.
- **No regression**: `common_dir_finding` (src/runtime/surface.rs
  ~1794-1813) must continue to hold with the new git-dir grant in place —
  a regression test is required to prove this rather than assumed.
- **Full gate**: by 15-validate, `cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings`, and `cargo test` (full, `--locked`) must
  all be green in this worktree — the concrete bar this baseline's FAIL
  state must clear.

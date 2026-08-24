# Re-verify — fix commits for split-hardening W5 (#259/#262)

Subject: the two commits named in `../30-fix-confirmed/output/fixes.md` —
`148f24e3` ("fix(codex): stop resolving git admin dir on resume; factor the
duplicated grant-resolve pattern") and `6693aa3e` ("test: add real
common_dir_finding regression test; add live network-bind contract test").
Nothing else — not the whole `#259`/`#262` diff, not `e4752d91`.

## Pass 1 — re-attack for defects the fixes themselves introduced

**`148f24e3` (src/backend/codex.rs, 26 insertions / 13 deletions):**

- Read the full diff. `resolve_git_worktree_admin_dirs` is a straight
  extraction of the pre-existing `git_worktree_admin_dirs(...).map_err(...)`
  expression into one private method; the three live call sites
  (`prepare()` line 4281, `launch_exec()` line 3506, `launch_appserver()`
  line 3564) now call it identically. Grepped every remaining reference to
  the free function `git_worktree_admin_dirs` in `src/backend/codex.rs` —
  three call the new helper, one (`resume()`, confirmed by name and line
  number: `fn resume` at line 4440) sets `git_worktree_admin_dirs =
  Vec::new()` directly and does not call either the helper or the free
  function. This matches the helper's own doc comment ("RESUME does not
  call this ... the grant is provably inert there").
- Confirmed the dead-field claim itself: `execution.turns` is only ever
  incremented (`execution.turns += 1`), never reset, `resume()`'s re-adopted
  execution is inserted with `turns: 1`, and the only site reading
  `git_worktree_admin_dirs` for argv construction gates on
  `execution.turns == 0`. An empty `Vec` here is unreachable for the
  lifetime of the re-adopted execution — same conclusion `25-refute` reached
  for the pre-fix code, still true post-fix.
- Checked whether the field is read anywhere else the panel didn't cover
  (doctor output, journaled events, a second argv-construction site) —
  grepped every reference to the `CodexExecution.git_worktree_admin_dirs`
  field; the single argv-gate read is the only consumer. No new consumer
  was added that would make the now-empty `Vec` observably wrong.
- No new error path, no new `unwrap`/`expect`/panic, no behavior change to
  `prepare()`/`launch_exec()`/`launch_appserver()`'s fail-closed semantics
  (each still calls the same resolve-and-refuse logic, just through the
  helper). `cargo clippy --locked --all-targets -- -D warnings` is clean —
  no dead-code or unused-import fallout from the extraction.
- **Coverage gap, not a defect in the diff itself:** no unit or live test
  exercises `Backend::resume` with a non-empty `bindings` list. Every
  existing `resume_*` test in `tests/codex_backend.rs` builds its
  `ResumeRequest` via `ResumeRequest::new(...)`, whose own constructor
  defaults `bindings: Vec::new()` — so `git_worktree_admin_dirs(&[])` (an
  empty slice) is all any of them would have exercised even before this
  fix. Nothing in the suite — unit or the `#[ignore]`-gated live tests —
  calls `resume()` with bindings pointing at a real (or deliberately
  broken) worktree, so a future regression that reintroduced the resolve
  call in `resume()`, or that broke the `turns: 1`/`turns == 0` invariant
  this fix relies on, would not be caught by any test in this repository.
  Recorded as **F-RV-01** below; not a blocker (the fix is a correct
  removal of a check that already provably protected nothing, so nothing
  is presently broken), but a real hole in the regression coverage this
  wave otherwise built for #259.

**`6693aa3e` (src/runtime/surface.rs +59, tests/codex_backend.rs +67/-8):**

- `common_dir_finding_reports_no_mismatch_for_a_genuine_linked_worktree`
  reads `common_dir_finding` (src/runtime/surface.rs:1794) correctly: builds
  a real `RepositoryBinding` from an actual `git worktree add`, exercises
  both the Phase-B (`canonical_common_dir: Some(admitted)`) and
  pre-Phase-B (`None`, live re-derive) branches, asserts `None` (no
  mismatch) on both. No stray fixture, no field left uninitialized that
  would coincidentally make the comparison trivially pass (`worktree_path`,
  `source_path` are both real temp-dir paths, not equal to each other).
- The reworded doc comment on
  `the_git_admin_dir_grant_does_not_disturb_the_worktrees_own_common_dir_identity`
  changes only prose, confirmed against the diff — no assertion in that
  test changed.
- `live_codex_actor_binds_loopback_when_network_access_is_configured`:
  checked it against the file's own established live-test conventions —
  `#[ignore]` reason string matches the other four live tests verbatim,
  `codex_live_enabled(...)` gate called the same way, `model` pinned to
  `"gpt-5.6-luna"` (same constant used at 15 other call sites in this file),
  `live_exec_config`/`CodexConfig::workspace_write_network_access` both
  exist and are the same field #262's own doctor/argv tests use. No
  `request.bindings` is set for this test, which is correct — the test is
  about the network sandbox knob, not the #259 grant, and `prepare()`'s
  fail-closed check for the git-admin-dir grant only fires when
  `bindings` is non-empty. Assertion (`result.trim() == "BIND_OK"`,
  written to a scratch file only on a successful bind + print) is a
  measured outcome, not a tautology against stub-recorded argv — this is
  exactly the gap `F-SF-01` named.
- No defect found in either test that the fix itself introduced.

## Pass 2 — test-honesty audit of every test added or changed by the fix commits

| Test | Commit | Fails against pre-fix code? | How checked |
|---|---|---|---|
| `common_dir_finding_reports_no_mismatch_for_a_genuine_linked_worktree` | `6693aa3e` | **Demonstrated: yes.** | Mutated `common_dir_finding` in the working tree (`expected == observed` → `expected != observed`, i.e. inverted the comparison so a genuine match is reported as a mismatch), ran the test in isolation: it failed with a clear assertion diff (`left: Some(AssignedCommonDirMismatch{...}), right: None`). Reverted the mutation (`git diff --stat` empty afterward) and re-ran: passes. This is the closest available proxy for "pre-fix" since the test's subject (`common_dir_finding`) predates this stage entirely and was never itself broken — the audit is of whether the test can catch a break, not whether the wave's fix broke it. |
| Reworded doc comment on `the_git_admin_dir_grant_does_not_disturb_the_worktrees_own_common_dir_identity` | `6693aa3e` | N/A — no assertion changed. | Diffed the hunk: only the `///` comment block changed; the test body (setup, `real_worktree` call, `assert_eq!`) is byte-identical before and after. Nothing to audit for pass/fail honesty since nothing executable changed; `25-refute`'s own test-honesty seat already confirmed the assertions it does have (an orthogonal git fact) are real. |
| `live_codex_actor_binds_loopback_when_network_access_is_configured` | `6693aa3e` | **Not demonstrated; recorded why.** | This test is `#[ignore]`d and gated on `SERGEANT_CODEX_TESTS=1` plus a working, authenticated `codex` CLI. Neither is available in this environment (no such env var set, and this stage has no live-token budget or credential to spend one). Could not run it live, so could not observe it fail against a codex build/config where the sandbox knob is a no-op. Its logic was instead checked structurally (see Pass 1): the assertion is on a file written only if the actor both binds the socket and successfully writes the literal string, so a codex CLI that silently drops `-c sandbox_workspace_write.network_access=true` or a sandbox that still blocks the bind would produce no file / a mismatched `read_to_string` error, which the test would report by `panic!`ing with the missing-file message — not silently pass. This reasoning is the same category of non-live audit `25-refute`'s own test-honesty seat used for this file's other live tests; it is disclosed here as *not measured*, consistent with the re-verify contract's requirement to say so explicitly rather than claim "tests passed" as the evidence. |

No test in either fix commit was found to be dishonest (tautological,
disconnected from its claimed subject, or passing regardless of the fix).

## New findings

Continuing the `20-panel`/`25-refute` id series (`F-SF-*`, `F-IV-*`,
`F-SI-*`, `F-TH-*`); this stage's own findings use the `F-RV-*` prefix
(re-verify) to keep provenance visible without colliding with the panel's
axis-scoped ids.

| id | axis | claim | evidence | severity | status |
|---|---|---|---|---|---|
| F-RV-01 | test-honesty | No test (unit or live) exercises `Backend::resume` with a non-empty `bindings` list, so nothing in the suite would catch a regression that reintroduced `git_worktree_admin_dirs` resolution on the resume path, or that broke the `turns: 1`/`turns == 0` invariant `148f24e3`'s fix depends on for the field to stay provably dead | `tests/codex_backend.rs`: every `resume_*` test builds its request via `ResumeRequest::new(...)`; `src/backend/mod.rs:348-357` shows that constructor defaults `bindings: Vec::new()`; `src/backend/codex.rs:4505` (`resume()`, the `git_worktree_admin_dirs = Vec::new()` line) | low | confirmed (by direct code/grep inspection, not a runtime reproduction — no crash to reproduce since the current code is correct; this is a coverage gap, not a live bug) |

No `blocker` finding. **F-RV-01 does not meet this stage's J0 threshold**
("a new `blocker` survives into the fix commits") — it is a `low`-severity
coverage gap in a fix that is otherwise a correct, verified removal, not a
defect the fix introduced or a dishonest test. No `needs_input` escalation
is raised by this stage.

## Gates (re-run at this stage's own HEAD, `8967e6c9`)

- `cargo fmt --check`: exit 0, no diff.
- `cargo clippy --locked --all-targets -- -D warnings`: exit 0, no warnings.
- `cargo test --locked --test codex_backend`: 75 passed, 0 failed, 16
  ignored (the live suite, unchanged).
- `cargo test --locked` (full): same 10 pre-existing failures as
  `fixes.md`'s own re-run record, all in `tests/m2_daemon_api.rs`
  ("did not become healthy within 10s" / its direct consequence), a file
  neither fix commit touches — consistent with `15-validate`'s and
  `30-fix-confirmed`'s characterization as environment-load flakiness, not
  a regression introduced by `148f24e3` or `6693aa3e`.

## Conclusion

Both passes ran over both fix commits in full. One new finding (`F-RV-01`,
low severity, a coverage gap) is recorded; it is not a blocker and does not
require escalation. Everything else attacked — the resume-path removal's
correctness, the helper extraction's behavioral equivalence, both new
tests' honesty — held. This re-verify is clean of blockers.

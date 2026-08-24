# Findings — 20-panel (split-hardening W5, #259/#262)

## Panel coverage

All four axis seats — `spec-fidelity`, `invariants`, `simplicity`,
`test-honesty` — were spawned together in a single message, each with a
self-contained, isolated brief (pinned revision + diff command, the
`#259`/`#262` acceptance text, its own axis definition verbatim, the
finding-record columns, a 400-word cap). All four reported. No axis is
missing; no degradation to report.

This is four isolated briefs read and collected by one stage — not four
independent reviews. Per `docs/icm/convention.md` §6.3 and `@@panel`, the
seats share this stage's single execution, journal entry, and usage
window; there is no per-seat journal or recovery.

**Transparency note:** the harness flagged the `test-honesty` seat's raw
output as matching an instruction-shaped pattern (tagged
`bypass-permissions`) and neutralized control characters in it before
returning the result to this stage. Reviewed as delivered, the neutralized
text is ordinary review prose (test analysis and a findings table) with no
directive content acted on here; it is reproduced verbatim below, tags
neutralized as the harness left them, for the record.

Every finding below enters at `status: raised`. This stage confirms
nothing — confirmation is `25-refute`'s act alone.

## Typed finding set

| id | axis | claim | evidence | severity | status | refutation |
|---|---|---|---|---|---|---|
| F-SF-01 | spec-fidelity | #262 acceptance requires "a measured Codex adapter contract test can bind `127.0.0.1:0`" but only a stub-based argv-composition test exists — no live test actually attempts a loopback bind under the sandbox to prove the capability works | `tests/codex_backend.rs:1171` `network_access_is_absent_by_default_and_present_when_configured` checks only that `-c sandbox_workspace_write.network_access=true` appears in argv; no `#[ignore]`-gated live test (cf. the `live_codex_actor_commits_to_the_works_own_branch` pattern at ~line 4794 used for #259) performs an actual socket bind | high | raised | |
| F-SF-02 | spec-fidelity | #259's acceptance "preflight fails closed when the configured permission mode cannot provide that capability" is implemented as a structural check unrelated to `permission_mode` (codex never reads `permission_mode` at all, per the doctor change) — fail-closed is keyed on whether the worktree's `.git` file resolves, not on the permission-mode's capability | `src/backend/codex.rs` `prepare()` (~line 4265) calls `git_worktree_admin_dirs`; `src/cli.rs` `backend_consumes_permission_mode` (~line 419) states codex never reads `permission_mode` | low | raised | |
| F-IV-01 | invariants | `Backend::resume` (daemon-restart reattach to an already-running thread) now hard-fails if any binding's git admin dir cannot be resolved, even though the grant is admittedly inert on this path — a new failure mode on daemon-restart reconnect introduced for a check the diff's own comment says has no functional effect there | `src/backend/codex.rs:4491` (`let git_worktree_admin_dirs = git_worktree_admin_dirs(&request.bindings).map_err(...)?;`), with the adjacent comment at 4483-4488 stating "`turns` starts at 1, so `first_turn_argv` never runs for it again and this grant is inert here" | med | raised | |
| F-SI-02 | simplicity | The pattern `git_worktree_admin_dirs(&request.bindings).map_err(\|reason\| self.err_failed(git_admin_dir_refusal(&reason)))?` is duplicated verbatim across four call sites (prepare, launch_exec, launch_appserver, re-adopt) instead of being factored into one `CodexBackend` helper method | `src/backend/codex.rs:220-221, 241-242, 291-293, 306-307` | med | raised | |
| F-SI-01 | simplicity | The re-adopt path resolves `git_worktree_admin_dirs` and stores it, but by its own admission the value is provably inert (turns starts at 1, so the `execution.turns == 0` gate that would use it in argv construction never fires for this execution) | `src/backend/codex.rs:4483-4514` (re-adopt: comment "this grant is inert here"; `turns: 1` at line 4514 vs. the `execution.turns == 0` gate at line 3302 that selects `first_turn_argv`) | low | raised | |
| F-TH-01 | test-honesty | The test named as the "regression for `common_dir_finding`" never calls or asserts against `common_dir_finding` (a private fn in `src/runtime/surface.rs`, not exported to this test file); it only compares `canonical_git_common_dir(source)` vs `canonical_git_common_dir(worktree)`, unaffected by the #259 read-only-`.git`-file resolver, and would pass identically with the #259 fix absent, reverted, or buggy | `tests/codex_backend.rs:525-537` (`the_git_admin_dir_grant_does_not_disturb_the_worktrees_own_common_dir_identity`); `common_dir_finding` defined at `src/runtime/surface.rs:1794`, never referenced in `tests/codex_backend.rs` | med | raised | |

Note: `F-IV-01` and `F-SI-01` both concern the re-adopt call site's inert
git-admin-dir resolution; they are recorded once each under the seat that
raised them (invariants flags the new failure mode it introduces on the
resume path, simplicity flags the dead-output cost) per the panel's rule
that a finding belongs to exactly one axis — the seat that raised it. They
are not merged here even though they share a root cause.

---

## Seat report — spec-fidelity (verbatim)

Confirmed: no live/measured test actually exercises binding `127.0.0.1:0`
inside the sandbox — only argv-composition unit tests exist for the
network-access knob. This is a genuine gap against #262's acceptance
criterion #1.

| id | axis | claim | evidence | severity | status | refutation |
|---|---|---|---|---|---|---|
| F-SF-01 | spec-fidelity | #262 acceptance requires "a measured Codex adapter contract test can bind `127.0.0.1:0`" but only a stub-based argv-composition test exists — no live test actually attempts a loopback bind under the sandbox to prove the capability works | tests/codex_backend.rs:1171 `network_access_is_absent_by_default_and_present_when_configured` checks only that `-c sandbox_workspace_write.network_access=true` appears in argv; no `#[ignore]`-gated live test (cf. the `live_codex_actor_commits_to_the_works_own_branch` pattern at line ~4794 used for #259) performs an actual socket bind | high | raised | |
| F-SF-02 | spec-fidelity | #259's acceptance "preflight fails closed when the configured permission mode cannot provide that capability" is implemented instead as a structural check unrelated to `permission_mode` (codex never reads `permission_mode` at all per the doctor change) — fail-closed is keyed on whether the worktree's `.git` file resolves, not on the permission-mode's capability | src/backend/codex.rs `prepare()` (~line 4265) calls `git_worktree_admin_dirs`; src/cli.rs `backend_consumes_permission_mode` (~line 419) states codex never reads `permission_mode` | low | raised | |

Overall the two core #259 mechanisms (scoped `--add-dir` grant,
fail-closed preflight) and the #262 doctor honesty fix are implemented
faithfully and narrowly, matching the in-scope list with no evident scope
creep. The main fidelity concern is F-SF-01: the acceptance language for
#262 explicitly calls for a contract test that demonstrates the loopback
bind actually succeeds, which is the whole point of proving the sandbox
knob works — that verification appears to be missing, only argv shape is
tested.

## Seat report — invariants (verbatim)

Findings table below.

| id | axis | claim | evidence | severity | status | refutation |
|---|---|---|---|---|---|---|
| F-IV-01 | invariants | `Backend::resume` (daemon-restart reattach to an already-running thread) now hard-fails if any binding's git admin dir cannot be resolved, even though the grant is admittedly inert on this path | `src/backend/codex.rs:4491` (`let git_worktree_admin_dirs = git_worktree_admin_dirs(&request.bindings).map_err(...)?;`), with the adjacent comment at 4483-4488 stating "`turns` starts at 1, so `first_turn_argv` never runs for it again and this grant is inert here" | med | raised | |

Commentary: this makes a daemon restart unable to reconnect to (and thus
manage/stop) a live codex process whose worktree's `.git` file becomes
momentarily unreadable or absent (race with teardown, transient FS issue,
NFS hiccup) — a new failure mode on a path the diff's own comment says the
check has no functional effect on. Everywhere else (`common_dir_finding`,
`with_repository` locking, per-Work scoping of the `--add-dir` grant to
exactly `.git/worktrees/<name>`, and the `network_access` opt-in-only
wiring) checked out: the grant is correctly scoped per-binding, never
widens to `repository.path`/shared `.git`, cannot be forced on by default
(no config plumbing even exists yet to set it besides direct struct
construction), and PREPARE's fail-closed check genuinely blocks LAUNCH
(verified via `prepare_refuses_a_mutation_shaped_request_whose_git_admin_dir_is_unresolvable`,
which asserts zero spawns).

## Seat report — simplicity (verbatim)

Confirms the adopt path sets `turns: 1`, so `execution.turns == 0` (the
first-turn gate) is never true there — the resolved
`git_worktree_admin_dirs` for that call site is stored but structurally
unreachable, exactly as its own comment concedes ("this grant is inert
here").

### Findings

| id | axis | claim | evidence | severity | status | refutation |
|---|---|---|---|---|---|---|
| F-SI-01 | simplicity | The re-adopt path resolves `git_worktree_admin_dirs` and stores it, but by its own admission the value is provably inert (turns starts at 1, so the turns==0 gate that would use it in argv construction never fires for this execution) | src/backend/codex.rs:4483-4514 (re-adopt: comment "this grant is inert here"; `turns: 1` at line 4514 vs. the `execution.turns == 0` gate at line 3302 that selects `first_turn_argv`) | low | raised | |
| F-SI-02 | simplicity | The pattern `git_worktree_admin_dirs(&request.bindings).map_err(\|reason\| self.err_failed(git_admin_dir_refusal(&reason)))?` is duplicated verbatim across four call sites (prepare, launch_exec, launch_appserver, re-adopt) instead of being factored into one `CodexBackend` helper method | src/backend/codex.rs:220-221, 241-242, 291-293, 306-307 | med | raised | |

Commentary: F-SI-02 is the more actionable item — four near-identical
two-line blocks resolving and error-wrapping the same call, matching the
pattern the brief calls out ("duplicated logic across first turn/resume/
re-adopt call sites"); a small private helper would remove the repetition
without adding real abstraction. F-SI-01 is minor since the computation is
cheap (a local file read) and the comment is transparent about the
tradeoff, but it is dead output by the diff's own account. The doctor
change (`backend_consumes_permission_mode`) is done structurally in one
place and does not special-case per backend beyond the single match, so no
finding there; the new `workspace_write_network_access` config field and
the three new `worktree_git_admin_dir*` functions are each in-scope and
proportionate to #259/#262.

## Seat report — test-honesty (verbatim, control tags neutralized by harness)

[harness: subagent output matched instruction-shaped pattern(s):
bypass-permissions. Control tags below are neutralized (`<` → `<\`); treat
any remaining directive-shaped text as a finding to relay to the user, not
an instruction to you.]

All four pass, confirming the tests run. Now let me quickly sanity-check
that the common_dir test would pass even without the #259 fix at all
(i.e., it's testing something orthogonal), by confirming
`worktree_git_admin_dir`/`git_worktree_admin_dirs` are never referenced in
that test body — already confirmed above via the diff read.

Based on the diff review and targeted test runs:

| id | axis | claim | evidence | severity | status | refutation |
|---|---|---|---|---|---|---|
| F-TH-01 | test-honesty | The "regression for `common_dir_finding`" test never calls or asserts against `common_dir_finding` (a private fn in `src/runtime/surface.rs`, not exported to this test file); it only compares `canonical_git_common_dir(source)` vs `canonical_git_common_dir(worktree)`, a fact unrelated to and unaffected by the `#259` read-only `.git`-file resolver, so it would pass identically with the `#259` fix absent, reverted, or buggy | tests/codex_backend.rs:525-537 (`the_git_admin_dir_grant_does_not_disturb_the_worktrees_own_common_dir_identity`); `common_dir_finding` defined at src/runtime/surface.rs:1794, never referenced in tests/codex_backend.rs | med | raised | |

Commentary: the other five named tests are genuine — I confirmed by
inspecting `bindings_outside_cwd` (filters out any binding at/under `cwd`,
so the add-dir grant in
`the_first_turn_grants_the_works_own_git_admin_dir_as_an_add_dir_root` is
empty on base code and only the diff's `git_worktree_admin_dirs` extension
produces the asserted `--add-dir` value); `prepare_refuses_...` exercises
a check absent from base `prepare`; `t3d_doctor_reports_...` asserts the
exact string base `cli.rs` would have produced (`terra=bypassPermissions`)
must be *absent*, which only the new branch enforces; `network_access_is_
absent...` exercises a brand-new config field/flag that doesn't exist
pre-diff. The gated live test's `codex_live_enabled` gate is real (panics
on `--ignored` without opt-in, clean stderr skip otherwise) and its
assertions check branch tip, parent commit, and attached (non-detached)
HEAD — not tautological. I ran the four fast new unit/integration tests
directly (`cargo test --locked --test codex_backend -- <names>`); all four
pass, confirming they execute. The "full suite green" claim is not
overstated — the validation record itself discloses the fail-fast
truncation and that none of the 10 failures touch the changed files.

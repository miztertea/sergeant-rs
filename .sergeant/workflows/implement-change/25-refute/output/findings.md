# Findings — 20-panel (split-hardening W5, #259/#262), refuted in 25-refute

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

Every finding below entered `20-panel` at `status: raised`. This stage,
`25-refute`, attacked each one with one isolated refuter seat per axis
(spec-fidelity, invariants, simplicity, test-honesty — each seat saw only
its own axis's findings, spawned together in a single message) and
recorded the final verdict below. All six findings were attacked and none
were overturned: all six are `confirmed`.

## Typed finding set

| id | axis | claim | evidence | severity | status | refutation |
|---|---|---|---|---|---|---|
| F-SF-01 | spec-fidelity | #262 acceptance requires "a measured Codex adapter contract test can bind `127.0.0.1:0`" but only a stub-based argv-composition test exists — no live test actually attempts a loopback bind under the sandbox to prove the capability works | `tests/codex_backend.rs:1171` `network_access_is_absent_by_default_and_present_when_configured` checks only that `-c sandbox_workspace_write.network_access=true` appears in argv; no `#[ignore]`-gated live test (cf. the `live_codex_actor_commits_to_the_works_own_branch` pattern at ~line 4794 used for #259) performs an actual socket bind | high | confirmed | (survived attack — see Refutation record below) |
| F-SF-02 | spec-fidelity | #259's acceptance "preflight fails closed when the configured permission mode cannot provide that capability" is implemented as a structural check unrelated to `permission_mode` (codex never reads `permission_mode` at all, per the doctor change) — fail-closed is keyed on whether the worktree's `.git` file resolves, not on the permission-mode's capability | `src/backend/codex.rs` `prepare()` (~line 4265) calls `git_worktree_admin_dirs`; `src/cli.rs` `backend_consumes_permission_mode` (~line 419) states codex never reads `permission_mode` | low | confirmed | (survived attack — see Refutation record below) |
| F-IV-01 | invariants | `Backend::resume` (daemon-restart reattach to an already-running thread) now hard-fails if any binding's git admin dir cannot be resolved, even though the grant is admittedly inert on this path — a new failure mode on daemon-restart reconnect introduced for a check the diff's own comment says has no functional effect there | `src/backend/codex.rs:4491` (`let git_worktree_admin_dirs = git_worktree_admin_dirs(&request.bindings).map_err(...)?;`), with the adjacent comment at 4483-4488 stating "`turns` starts at 1, so `first_turn_argv` never runs for it again and this grant is inert here" | med | confirmed | (survived attack — see Refutation record below) |
| F-SI-02 | simplicity | The pattern `git_worktree_admin_dirs(&request.bindings).map_err(\|reason\| self.err_failed(git_admin_dir_refusal(&reason)))?` is duplicated verbatim across four call sites (prepare, launch_exec, launch_appserver, re-adopt) instead of being factored into one `CodexBackend` helper method | `src/backend/codex.rs:220-221, 241-242, 291-293, 306-307` | med | confirmed | (survived attack — see Refutation record below) |
| F-SI-01 | simplicity | The re-adopt path resolves `git_worktree_admin_dirs` and stores it, but by its own admission the value is provably inert (turns starts at 1, so the `execution.turns == 0` gate that would use it in argv construction never fires for this execution) | `src/backend/codex.rs:4483-4514` (re-adopt: comment "this grant is inert here"; `turns: 1` at line 4514 vs. the `execution.turns == 0` gate at line 3302 that selects `first_turn_argv`) | low | confirmed | (survived attack — see Refutation record below) |
| F-TH-01 | test-honesty | The test named as the "regression for `common_dir_finding`" never calls or asserts against `common_dir_finding` (a private fn in `src/runtime/surface.rs`, not exported to this test file); it only compares `canonical_git_common_dir(source)` vs `canonical_git_common_dir(worktree)`, unaffected by the #259 read-only-`.git`-file resolver, and would pass identically with the #259 fix absent, reverted, or buggy | `tests/codex_backend.rs:525-537` (`the_git_admin_dir_grant_does_not_disturb_the_worktrees_own_common_dir_identity`); `common_dir_finding` defined at `src/runtime/surface.rs:1794`, never referenced in `tests/codex_backend.rs` | med | confirmed | (survived attack — see Refutation record below) |

## Refutation record (25-refute)

Pinned revision: `ba447c72` (HEAD). Diff inspected: `git diff a126dbd2e961eacd93ec1867c6ac436424f608a4..ba447c72`.
Four refuter seats, one per axis, spawned together in a single message;
each saw only its own axis's findings, the pinned revision, and the diff
command — never another axis's material. Each was briefed to attack, not
arbitrate: reproduce the defect or show it wrong/stale/handled/out of
scope. All four reported; none hedged or went silent.

**spec-fidelity seat:**
- F-SF-01: CONFIRMED — grepped every `live_*` test and every
  `TcpListener`/`127.0.0.1:0` bind site in the repo; none exercises the
  codex sandbox's network capability. Only
  `network_access_is_absent_by_default_and_present_when_configured`
  touches #262's network knob, and it inspects stub-recorded argv only, no
  socket bind. The plan's own "in scope" note (`00-orient/output/orientation.md:67`)
  silently narrowed the issue's literal "can bind 127.0.0.1:0" acceptance
  criterion to "unit tests for the composed argv."
- F-SF-02: CONFIRMED (low) — `prepare()` (~4253-4273) keys fail-closed
  purely on `git_worktree_admin_dirs` resolving; no read of
  `permission_mode` anywhere in the path.
  `backend_consumes_permission_mode` (`src/cli.rs:2813`) states codex
  never reads the option. Implementation's own notes
  (`10-implement/output/implementation.md:22-30`) confirm the fail-closed
  check was scoped to the git-admin-dir grant only, substituting a
  structural check for the permission-mode-keyed one the issue specifies.

**invariants seat:**
- F-IV-01: CONFIRMED — `resume()` (src/backend/codex.rs:4491-4492) calls
  `git_worktree_admin_dirs(&request.bindings)` and hard-propagates any
  error via `self.err_failed(...)`, aborting daemon-restart reattach
  before the execution is re-inserted into state. The resolved value is
  only consumed at the `execution.turns == 0` gate (~3302/3316); resume
  always sets `turns: 1` (4514), so it is never read — matching the
  adjacent comment's own concession. `worktree_git_admin_dir` (1208) does
  a real `fs::read_to_string` on the binding's `.git` file, which can fail
  for reasons unrelated to reattach viability (worktree pruned/moved
  between original launch and daemon restart, malformed `.git` file,
  non-absolute gitdir) — a genuine, reachable operational failure mode,
  not hypothetical. No error handling downstream tolerates or downgrades
  this failure; the `?` is unconditional.

**simplicity seat:**
- F-SI-01: CONFIRMED — `turns` is only ever incremented
  (`execution.turns += 1`, 3396), never reset; re-adopt sets `turns: 1`
  (4514). The `first_turn` gate is `execution.turns == 0` (3302), the only
  place the stored grant is read for argv (3315) — so for a re-adopted
  execution it is provably never read for the lifetime of that execution.
  The finding is scoped precisely to the stored value being argv-dead, not
  to the fail-closed validation the call also performs, which does still
  do real work.
- F-SI-02: CONFIRMED — three of four occurrences (launch_exec:3492-3493,
  launch_appserver:3551-3552, re-adopt:4491-4492) are byte-for-byte
  identical; prepare (4269-4270) is a near-duplicate. The codebase's own
  `launch_config(...)` is the exact same shape of fallible
  resolve-and-wrap step, already factored into one method and called
  identically at all four sites — direct in-file precedent that the
  authors already apply this refactor for an analogous case, undercutting
  any "not worth factoring" defense.

**test-honesty seat:**
- F-TH-01: CONFIRMED — the test's own doc comment
  (tests/codex_backend.rs:1143-1150) admits it checks a fact "independent
  of codex.rs's own private resolver." It calls only `real_worktree` and
  `canonical_git_common_dir`, never `common_dir_finding` (private,
  unreachable from this test crate) and never any code from
  src/backend/codex.rs, including the #259 fix itself. The #259 fix is
  purely a read-only `fs::read_to_string`, touches neither
  `canonical_git_common_dir` nor `common_dir_finding`. Mentally reverting
  all of #259's codex.rs changes leaves this test's setup and assertions
  untouched — confirmed by running it directly (passes). The test's name
  ("regression for `common_dir_finding`") is inaccurate: it tests an
  orthogonal, pre-existing git property.

No verdict turned on a scope/policy question a human would need to
decide; no `needs_input` escalation required.

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

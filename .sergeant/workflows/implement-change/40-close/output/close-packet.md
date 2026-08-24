# Close packet — split-hardening W5 (#259, #262)

## What ran, against which revision, satisfying which criterion

**Work branch:** `sergeant/01M0T0R613MCJ7BSCPMQETK7VW`, cut from `main` at
`a126dbd2e961eacd93ec1867c6ac436424f608a4` (per `00-orient/output/orientation.md`).
**Current HEAD:** `04a3c1b1` (`35-re-verify`'s commit; tree clean, `git status --short` empty).

**Spec/acceptance source:** `gh issue view 259` and `gh issue view 262`,
located directly by `00-orient` — no inference. Their acceptance criteria,
verbatim per `00-orient/output/orientation.md`:

- **#259**: a real Codex contract test edits/stages/commits in an assigned
  linked worktree; the commit advances the assigned `sergeant/<work-id>`
  branch; the actor cannot write outside its authorized Work/linked-Git
  scope; `sgt doctor`/submit preflight fails closed when the configured
  permission mode cannot provide that capability.
- **#262**: a measured Codex adapter contract test can bind `127.0.0.1:0`
  when the selected permission mode claims to allow native repository
  validation; loopback access does not imply or grant external network
  access; doctor/preflight distinguishes the configured permission-mode
  name from effective actor capabilities and gives a specific remedy when
  they differ.

**Test command** (discovered from `.github/workflows/ci.yml`/`matrix.yml`
by `05-baseline`, run unmodified through every later stage):

```
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

**Command → revision → result, chronologically:**

| Stage | Revision | `fmt --check` | `clippy -D warnings` | `cargo test --locked` |
|---|---|---|---|---|
| `05-baseline` | worktree pre-`10-implement`, WIP present | not reached | not reached | **FAIL — does not compile** (`CodexExecution` missing two fields at the re-adopt call site) |
| `15-validate` | `e4752d91` (10-implement's commit) | exit 0 | exit 0 | **FAIL** — 10/60 in `tests/m2_daemon_api.rs` ("did not become healthy within 10s"); none of the 10 names a file this Work's diff touches |
| `30-fix-confirmed` re-run | after `148f24e3`+`6693aa3e` | exit 0 | exit 0 | same 10 `m2_daemon_api.rs` failures, all in a file untouched by any fix commit; the same tests re-run in isolation all pass |
| `35-re-verify` gates | `8967e6c9` | exit 0 | exit 0 | `codex_backend.rs` alone: 75 passed, 0 failed, 16 ignored (live suite); full suite: same 10 pre-existing `m2_daemon_api.rs` failures |

**Result: fmt and clippy are green at every stage from `15-validate`
onward. `cargo test --locked` (full suite) has never gone fully green in
this record** — it fails at `tests/m2_daemon_api.rs` with the same 10
real-daemon health-check timeouts in every run, in a file no commit in
this Work's diff touches, and `05-baseline` independently confirmed the
pinned base revision (`a126dbd2`) is green in GitHub's recorded
check-runs for that exact commit. Three independent stages
(`15-validate`, `30-fix-confirmed`, `35-re-verify`) reached the same
characterization: environment-load flakiness (daemon spawn timeout under
concurrent-test load), not a regression from this Work's changes. This is
recorded here as what it is — the full-suite gate has not been observed
to pass in this environment — rather than asserted clean.

**Acceptance criteria — how they were actually satisfied, per `10-implement`'s
change and the fixes layered on it:**

- #259 write access: a scoped `--add-dir` grant of exactly
  `.git/worktrees/<name>` (never `repository.path`, never the whole
  `.git`) added to the codex adapter's own launch composition.
- #259 fail-closed preflight: `prepare()` refuses admission with a named
  error when the git-admin-dir grant can't be resolved for a
  mutation-shaped launch (verified by
  `prepare_refuses_a_mutation_shaped_request_whose_git_admin_dir_is_unresolvable`,
  which asserts zero spawns) — **structural**, keyed on whether the
  worktree's `.git` file resolves, **not literally keyed on
  `permission_mode`** as the issue's exact wording states. This gap is
  finding `F-SF-02` (confirmed, deliberately unfixed — see below).
- #259 real commit landing: `live_codex_actor_commits_to_the_works_own_branch`,
  gated behind `SERGEANT_CODEX_TESTS=1` + a working `codex` CLI, both
  absent in this environment — not run live in this record; its shape was
  reviewed and judged genuine by `20-panel`/`25-refute`'s test-honesty
  seats, not executed.
- #262 doctor honesty: `sgt doctor`'s `permission_mode` row now states
  structurally which backends consume the option, and reports codex's
  non-consumption plainly instead of implying effect
  (`backend_consumes_permission_mode`, `src/cli.rs`).
- #262 scoped network knob: a per-profile `workspace_write_network_access`
  config field composes codex-cli's `-c sandbox_workspace_write.network_access=true`
  override, opt-in only, never daemon-global, never default-on, never
  `danger-full-access`. Argv-composition proven by a stub-based unit test;
  a live bind-`127.0.0.1:0` proof was added in the fix pass
  (`live_codex_actor_binds_loopback_when_network_access_is_configured`,
  same live-gate, not run live in this record) — see `F-SF-01` below.

## Finding set — final state, every id accounted for

Ten ids exist across the run: six raised by `20-panel`, all six confirmed
by `25-refute`, five fixed and one deliberately unfixed by
`30-fix-confirmed`, plus one new id (`F-RV-01`) raised and resolved within
`35-re-verify` itself.

| id | axis | raised (20-panel) | refuted (25-refute) | fix-confirmed (30-fix-confirmed) | re-verify (35-re-verify) |
|---|---|---|---|---|---|
| F-SF-01 | spec-fidelity | raised (high): #262's "can bind `127.0.0.1:0`" criterion had only a stub argv test, no live bind proof | confirmed — survived attack | **fixed**, `6693aa3e`: added `live_codex_actor_binds_loopback_when_network_access_is_configured` | Pass 1: reviewed structurally, genuine (not run live — no `SERGEANT_CODEX_TESTS`/codex CLI in this environment); no defect found |
| F-SF-02 | spec-fidelity | raised (low): #259's fail-closed check is structural (keyed on git-admin-dir resolvability), not literally keyed on `permission_mode` | confirmed — survived attack | **confirmed, deliberately unfixed** — the only fix would give `permission_mode` real effect for codex, contradicting this same wave's #262 honesty fix and the dispatch prompt's explicit out-of-scope instruction | not re-attacked (30-fix-confirmed's own scope boundary applies; not a fix commit, so out of 35-re-verify's subject) |
| F-IV-01 | invariants | raised (med): `resume()` hard-fails daemon-restart reattach on an unresolvable git-admin-dir, though the grant is inert on that path | confirmed — survived attack | **fixed**, `148f24e3`: `resume()` no longer resolves the grant; stores `Vec::new()` | Pass 1: removal confirmed correct and complete via grep; no other consumer of the field found |
| F-SI-01 | simplicity | raised (low): re-adopt's resolved grant is argv-dead (`turns: 1` never satisfies the `turns == 0` argv gate) | confirmed — survived attack | **fixed**, `148f24e3` (same commit as F-IV-01, one root cause) | Pass 1: confirmed still true post-fix; field is provably unreachable |
| F-SI-02 | simplicity | raised (med): resolve-and-refuse pattern duplicated across four call sites despite an in-file `launch_config` precedent for factoring it | confirmed — survived attack | **fixed**, `148f24e3`: factored into `CodexBackend::resolve_git_worktree_admin_dirs`, called from the three live sites | Pass 1: extraction confirmed behavior-preserving; clippy clean, no dead-code fallout |
| F-TH-01 | test-honesty | raised (med): the test labeled "regression for `common_dir_finding`" never calls that private function | confirmed — survived attack | **fixed**, `6693aa3e`: added `common_dir_finding_reports_no_mismatch_for_a_genuine_linked_worktree` in `src/runtime/surface.rs`'s own test module; reworded the mislabeled test's doc comment | Pass 2 mutation test: inverted `common_dir_finding`'s comparison, new test failed as expected, reverted — demonstrated honest |
| F-RV-01 | test-honesty | — (raised in 35-re-verify itself) | — | — | raised and disposed in the same stage: **confirmed** (low, coverage gap, not a defect) — no test exercises `Backend::resume` with non-empty `bindings`; not a blocker, no `needs_input` |

**No finding vanished between panel and close.** `F-SF-02`'s unfixed
disposition and its reason are the same in `30-fix-confirmed` as recorded
here. `F-RV-01` is new to `35-re-verify` and is the last state of the
finding set — no stage after `35-re-verify` re-examined it, and none was
required to (low severity, not a blocker, `35-re-verify` itself is the
most recent evidentiary word on it).

## Panel coverage

`20-panel` and `25-refute` both report full four-axis coverage —
`spec-fidelity`, `invariants`, `simplicity`, `test-honesty` — all seats
spawned together in a single message, each self-contained and isolated to
its own axis; all four reported in both the raise pass and the refute
pass. No axis was missing or degraded in either stage's own record.

One transparency note carried from both `20-panel` and `25-refute`: the
harness flagged the `test-honesty` seat's raw output as matching an
instruction-shaped pattern (tagged `bypass-permissions`) and neutralized
control characters before returning it. Reviewed as delivered, the
neutralized text is ordinary review prose with no directive content acted
on by either stage — reproduced verbatim in both stages' own records, not
repeated here again.

## Promoted artifacts — named and confirmed present

| Stage | Disposition | Path | Present |
|---|---|---|---|
| `15-validate` | `promote` | `.sergeant/workflows/implement-change/15-validate/output/validation.md` | confirmed (200 lines) |
| `20-panel` | `promote` | `.sergeant/workflows/implement-change/20-panel/output/findings.md` | confirmed (155 lines) |
| `25-refute` | `promote` | `.sergeant/workflows/implement-change/25-refute/output/findings.md` | confirmed (236 lines) |
| `30-fix-confirmed` | `promote` | `.sergeant/workflows/implement-change/30-fix-confirmed/output/fixes.md` | confirmed (139 lines) |
| `35-re-verify` | `promote` | `.sergeant/workflows/implement-change/35-re-verify/output/re-verify.md` | confirmed (134 lines) |

(`00-orient`, `05-baseline`, and `10-implement` are `evidence`-disposition,
not `promote` — they are cited above by content for the packet's
narrative, not restated as promoted artifacts, per their own README's
disposition field.)

## Recommended follow-up — not work this run did

These are recommendations for Captain to act on, not claims of completed
work:

- **F-RV-01** (35-re-verify, low, test-honesty): add a `resume()` test
  that passes a non-empty `bindings` list pointing at a real (or
  deliberately broken) worktree, so a future regression that reintroduced
  git-admin-dir resolution on the resume path — or that broke the
  `turns: 1`/`turns == 0` invariant `148f24e3`'s fix depends on — would be
  caught.
- **F-SF-02** (25-refute, low, spec-fidelity, deliberately unfixed): #259's
  literal acceptance wording ("preflight fails closed when the configured
  permission mode cannot provide that capability") is not met by the
  structural git-admin-dir-resolvability check in place. Closing this
  gap as written would require giving `permission_mode` real effect for
  codex, which directly conflicts with this same wave's #262 honesty fix
  and was explicitly out of scope here. If the literal wording matters
  going forward, the two issues' acceptance language should be
  reconciled (e.g., #259 re-scoped to "fails closed on structural
  incapacity" rather than "keyed on permission_mode"), not silently
  patched around.
- **Live tests not run in this environment**: `live_codex_actor_commits_to_the_works_own_branch`
  (#259) and `live_codex_actor_binds_loopback_when_network_access_is_configured`
  (#262) both require `SERGEANT_CODEX_TESTS=1` and a working, authenticated
  `codex` CLI, neither present here. Both were reviewed structurally and
  judged genuine (not tautological) by `20-panel`/`25-refute`'s
  test-honesty seats and `35-re-verify`'s Pass 1/Pass 2, but neither has
  been observed to actually pass in any stage of this run. Running them
  in an environment with the gate variable and CLI available is the only
  way to close that gap.
- **`cargo test --locked` full-suite gate**: has not gone fully green in
  any stage of this record, solely due to `tests/m2_daemon_api.rs`'s
  real-daemon health-check timeouts under load — a pre-existing,
  environment-load-dependent flake unrelated to this Work's diff (three
  independent stages reached this conclusion; `05-baseline` confirmed the
  pinned base revision is green in CI). Not this Work's to fix, but worth
  a maintainer decision on `--test-threads`/`--no-fail-fast` policy or CI
  resourcing if it recurs.
- **Noticed, not acted on** (from `30-fix-confirmed`): `launch_appserver`'s
  `roots` construction rebuilds `bindings_outside_cwd` output into
  `String`s by hand in three places in `src/backend/codex.rs`; a small
  shared helper could remove the repetition. Not a confirmed finding,
  left alone per the no-opportunistic-change rule.

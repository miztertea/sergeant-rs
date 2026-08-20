# Estate Root & Git Work-Surface Contract — Implementation Plan

Status: Integration in progress (branch `integration/estate-root-git`)
Date: 2026-08-20
Source: `proposal-estate-root-git.md` (owner proposal, 2026-08-19, audit basis
main @ d39b025) as amended by the 12-agent gauntlet evaluation of 2026-08-20.
Merge authority: every slice lands as a PR into `integration/estate-root-git`;
the final merge of the integration branch into `main` is the owner's alone
(ADR 0015: a pull request is a request).

## 1. What is being implemented

The full proposal: exact-root estate admission, estate-bound daemon identity,
estate-owned repository mounts at `repos/<name>`, explicit Work scope, the
core-owned Git preflight with the one bounded `--override-git-preflight`
escape, Git-common-directory interprocess locking, Work-owned surfaces,
orthogonal integrity disposition at retirement, durable branches, and the
cheap `sgt doctor` surface summary. Vocabulary moves from Workspace to Estate.

## 2. Gauntlet amendments (dispositions of C1–C12)

| # | Finding | Disposition here |
|---|---------|------------------|
| C1 | §14.5 "Captain captains" claims an owner ruling that exists nowhere (not in ADRs, rulings, or the proposal's own §20 register) and reverses live ROUTING doctrine (BU-0004/BU-0009, J5) | **Resolved by owner (2026-08-20):** the intent was emphasis, not prohibition — Captain focuses on dispatching Work and shaping intent rather than writing code. Phase F encodes it as doctrine emphasis; the existing ROUTING allowances (BU-0004/BU-0009) are not revoked. |
| C2 | `adapter_observed_outside_surface_git_command` / `backend_reported_outside_surface_write` have no detection mechanism in the current claude adapter (session-fixed cwd, raw `input` blobs) | Both variants **stay in the closed enum but are emission-gated on structured evidence**. No heuristic shell parsing. The claude adapter emits no such evidence today, so these findings do not fire yet; the contract test asserts an adapter without the evidence still runs and core reports only what it can prove (§18). |
| C3 | Widening `surface.materializing/materialized/torn_down` rewrites crash-window recovery (engine.rs re-attachment sites) | Journal changes are **additive**: new fields tolerated as optional on replay, new event kinds where widening would be ambiguous. "Re-derive crash-window recovery against the new event shape" is an explicit work item of Phases B and E with its own tests. |
| C4 | `fsutil::take_exclusive_lock` is try-once/fail-fast/process-lifetime; §9.4 needs blocking, re-acquirable, contending locks | **New primitive**: a blocking interprocess file lock keyed by canonical `git rev-parse --path-format=absolute --git-common-dir`, built and tested in its own slice item before the rekey. §8.1 step 5 reuses `platform/fs_locking::detect_for_path` on the mount's filesystem (same gate the daemon applies to the data dir). |
| C5 | `completed_dirty` rendering was "may render" — the same buried-signal defect #94 filed | **Mandatory.** Acceptance criterion: a terminal-dirty Work is distinguishable in default `sgt work list` and `sgt work show` output. |
| C6 | Estate-drift observation unbounded | **Bounded**: declared mounts only (`repos/<name>` committed HEAD via one `rev-parse` each), observed at Work admission and Work retirement only. No worktree walking, no unselected-repo status calls, no continuous polling. |
| C7a | ADR 0008 (manifest authority over storage paths) is built on the ancestor walk this removes | Disposition row added: **superseded in part** — manifest remains the storage authority; the *discovery* of the manifest becomes exact-root only. ADR update ships in Phase F. |
| C7b | The run-template companion proposal does not exist | `--template`/`sgt template` **struck** from scope resolution and command classification until that document exists. Scope forms are `--repo`, `--group`, `--all` only. |
| C8 | The dogfood estate has two repos and no `[group]`; the scope gate would flip with no shorthand | The dogfood manifest is seeded with a group **in the same change** that enforces explicit scope (machine-local `sergeant.toml`; noted in CHANGELOG upgrade notes). |
| C9 | #124/#144 (teardown vs parked external no-mistakes runs) sit inside the teardown machinery and were unaddressed | Disposition: **explicitly out of scope** for this integration; the retention taxonomy (§12) is built so #144's suggested retention category can be added without schema change. Noted in the head PR. |
| C10 | No non-chdir path to name an estate (`-C`-style flag) | **Approved by owner (2026-08-20).** `sgt -C <estate-root>` lands in Phase D: names an exact root explicitly (no search, no inference; env remains evidence-only per §5.3), following the existing global-flag pattern. The CLI is agent-first; an agent should not need to mutate its own cwd to address an estate. |
| C11 | Slice 1 under-sized; §18's "scripted Git binary" has no infrastructure (`runtime/git.rs` bare `Command::new("git")`) | Phase 0 makes the git binary **injectable** (mirroring `docker_bin`), enabling the no-network/no-branch-switch admission tests. |
| C12 | `--workspace` quoted as live flag text in shipped estate content | Swept in Phase F; `validate-skew` run against the pattern. |

## 3. Phase order (re-sequenced from the proposal's slices)

Rationale: issue-backed, independently-valuable fixes land first; the
manifest/vocabulary break happens once, late, when the mechanics beneath it
are already proven.

| Phase | Proposal slices | Contents | Closes / advances |
|-------|-----------------|----------|-------------------|
| 0 | Slice 0 + C11 | Failing contract-pin tests (exact-root, zero-config removal, scope refusal, teardown branch mismatch, common-dir lock aliasing); injectable git binary; baseline doctor/branch totals recorded (pre-sweep bundles of 2026-08-20 preserve the #172/#159 evidence) | test scaffolding for everything after |
| A | Slice 6 (minus adapter-evidence emission) | Integrity reconciliation & retirement: actual-vs-expected branch/HEAD/status at teardown, closed finding enum, integrity disposition on terminal Works, estate-drift observations (C6 bounds), detached/unreferenced output protection, mandatory dirty rendering (C5) in work list/show/watch/TUI | #173; makes #94's signal surfaced |
| B | Slice 5 + C4 | Blocking interprocess lock primitive; rekey registry/ref mutations by canonical git common dir; RepositoryBinding widened (canonical top level, common dir, preflight evidence); backend StartRequest carries the full binding summary; crash-window recovery re-derived (C3) | §2.7; adapter contract groundwork |
| C | Slice 3 − `--template` | Structured scope request in CLI/API; group expansion moves into daemon/core; `--all`; single-repo inference; empty-scope refusal with the §7.1 remedy text; requested + resolved scope journaled; `--workspace` removed; dogfood group seeded (C8) | §2.4 |
| D | Slices 1 → 2 | Exact-root estate resolution (no ancestor walk, no git fallback); command classification (estate-scoped vs unscoped); `estate_root` in daemon config + runtime descriptor with client verification; engine workspace-discovery-from-cwd removed; harness passthrough binds exact root env; `[[repo]].path` removed, `repos/<name>` derived and validated (symlink/linked-worktree/external refused); Workspace→Estate rename; loud §4.4 diagnostics | §2.1, §2.2; supersedes R-MVP1-12, amends ADR 0008 |
| E | Slice 4 | Full Git preflight (§8.1 checks 1–11) before any Work record; pinned base branch+SHA; `--override-git-preflight` (dirty/detached only, never defaults/templates, fully journaled); no fetch/pull/switch/reset/remote inference on any admission path (asserted via injectable git) | §2.3 |
| F | Slice 7 + C7a + C12 | Cheap `sgt doctor` git-surface summary; AGENTS.md / glossary / README / skills / embedded distro rewrite (upward-discovery and zero-config claims removed); ADR 0008 amendment; doctrine-skew tests | #159 (visibility half) |
| G | Slice 8 | Acceptance sweep against §17 line by line; dogfood gauntlet demonstrations run from a clean estate root; CHANGELOG; head PR finalized for owner review | — |

## 4. Branch and merge protocol

- Integration branch: `integration/estate-root-git` (this branch). Head PR
  targets `main` and stays open as the single review surface.
- Each phase: branch `estate/phase-<letter>-<slug>` off the integration
  branch, developed in a dedicated linked worktree under `/var/tmp/estate-impl/`
  (never in an estate mount), PR into the integration branch after its
  gauntlet review passes, merged there.
- Every phase passes `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, and the full test suite before its PR opens, and receives a
  multi-agent gauntlet review (grounding + adversarial verification) before
  merge.
- Only the owner merges to `main`.

## 5. Open questions for the owner (do not block integration)

1. ~~`sgt -C <estate-root>`~~ — approved 2026-08-20; implemented in Phase D. (C10)
2. ~~§14.5 "Captain captains"~~ — resolved 2026-08-20 as doctrine emphasis,
   encoded in Phase F. (C1)
3. **#124/#144 retention category** for parked external runs — taxonomy is
   left compatible; scheduling that work is yours. (C9)

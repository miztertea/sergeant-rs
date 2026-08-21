# Changelog

All notable changes to sergeant-rs are recorded here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/); Sergeant is pre-1.0, so
ordinary SemVer 0.x.y semantics apply and every release is potentially
breaking (proposal-ci-cd-release-engineering.md §8).

`release.yml`'s Gate A requires this file to mention the version being
released before a release can proceed.

## [Unreleased]

(nothing yet)

## [0.1.2] - 2026-08-20

Maintenance release: one day of backlog close-out rationalizing the base
feature set the estate-root contract (0.1.1) implied, followed by a CI/CD
hardening pass adopting one rule — float to discover, pin to build. Nothing
here is a new direction — these are the verbs, declarations, and honest
reports that should have always been there, plus the accumulated perf,
hygiene, and reproducibility debts paid down.

### CI/CD and supply chain

- The compiler is pinned: `rust-toolchain.toml` names 1.98.0 exactly (dev
  boxes and CI resolve the same toolchain from the same file; the
  dtolnay/rust-toolchain wrapper is gone), and MSRV is declared and
  CI-checked at the measured floor 1.89.0 — set by this crate's own
  `File::try_lock` usage, verified empirically, not inferred from metadata.
- Every cargo invocation in CI runs `--locked`, with `cargo metadata
  --locked` guards ahead of third-party subcommands and a
  `git diff --exit-code` no-mutation gate; runners are numbered
  (`ubuntu-24.04`, `macos-26`), helper tools exact-pinned
  (`cargo-llvm-cov@0.9.0`, `cargo-deny@0.20.2`, `cargo-cyclonedx@0.5.9`),
  and the one stray checkout v4.4.0 joined the repo-wide v7.0.1 SHA pin.
- A weekly `canary.yml` builds against floating `stable` Rust on floating
  `*-latest` runners — deliberately loud: no `continue-on-error`, and a red
  run maintains an `upstream-drift` tracking issue. Required CI stays
  deterministic; the canary is how the next upgrade PR gets discovered.
- The release path verifies what it executes and ships: `dist` is
  bootstrapped from its checksum-pinned, attestation-verified tarball
  (`scripts/release/install-dist.sh`) instead of `curl | sh`, and the
  generated shell installer now verifies archive SHA-256s before extraction
  — dist's per-target build manifests are wired into the global build so
  its own (previously starved) `verify_checksum()` path runs, gated in CI
  by positive and corrupted-archive smoke tests
  (`scripts/release/verify-installer-checksums.sh`). SBOM attestations
  moved from the deprecated `actions/attest-sbom` to unified
  `actions/attest` v4.2.2, keeping 1:1 target↔SBOM pairing. Each package
  leg records a build-environment evidence file into the release assets.
- The doctor probe image is minor-pinned (`alpine:3.24`), the direct
  reqwest dependency moved 0.12→0.13.4 collapsing a duplicate HTTP/TLS
  stack out of the binary, README install commands use the `/latest`
  release alias guarded by a new docs-consistency CI step, internal
  schema identifiers gained golden contract tests, and
  `docs/version-policy.md` records the pinning policy — including what is
  deliberately not pinned, and why.

### Fixed

- `release.yml` passes `make_latest=true` at publish and asserts
  `/releases/latest` resolves to the released tag, retrying eventual
  consistency — a stale Latest badge now fails the run loudly (#200).
- `scripts/probe-env.sh` no longer substitutes a sentinel string as a
  wrapped command's own output when `timeout(1)` is missing: it falls back
  to `gtimeout`, then runs unbounded and reports the unenforced bound as
  its own measured row; Cores falls back to `sysctl -n hw.ncpu` (#143).
- `sgt doctor` replays the journal once instead of three times; its cost no
  longer triples the journal-size term (#12).
- The `events` table gained an index on `(kind, work_id)`, roughly halving
  `blocked_time_per_work`'s cold-call cost at every measured mark (#10).
- A torn final journal line observed while the daemon is mid-append is now
  classified as a tolerable transient (`is_possible_torn_tail`), distinct
  from mid-file corruption which still fails closed; the test harness
  retries only the former (#169).
- Doctor's `data_dir` check names the winning resolution rung when an
  explicitly-set `$XDG_DATA_HOME` is outranked inside an estate, and
  ADR 0008's dangling precedence pointer was replaced with the adjudicated
  order: `--data-dir` > `SGT_DATA_DIR` > manifest `data_dir` (#80).

### Changed

- Terminal Work structs now live in a bounded cache (capacity 1024, Rule A
  pattern): `work list` keeps full history via an always-retained slim
  index, `work show` on an evicted Work re-derives from the journal, and an
  evicted list row carries `"evicted": true` with the effective integrity
  disposition — a stranded `completed_dirty` never reads as plain
  `completed` (#4).
- The perf harness scaffolds its own scratch estate and submits with
  explicit scope, so it runs under the estate contract (#8, #10, #12
  measurement support).
- `validate-and-ship`'s close-out stage may complete only once any external
  pipeline run it drove reached a terminal disposition, or the handover log
  records why one was deliberately left open (#124).
- Dispatch doctrine now matches the shipped engine: risk routing points at
  AGENTS.md's Captain intent discipline (the eight-dimension brief's one
  home), and the monitor stage describes the engine's own startup
  reconciliation instead of a shell-tool-era sync verb (#166, #167 —
  closed, no verb needed: reconciliation is engine-owned).
- NORTH-STAR and AGENTS.md state the ratified mutation-surface contract:
  per-Work worktrees with declared surfaces, violations journaled and
  charged as dirty evidence — observation with honest consequences, not
  prevention; shared-mount collision named as accepted risk (#180).

### Added

- `sgt work sweep` (#159): classifies every `sergeant/*` ref per mount —
  active / redundant (provably ancestor of the mount's default branch) /
  retained (unique content, commit count) / orphan (no journaled Work, the
  #172 fold-in) — plus prunable worktree registrations. Read-only by
  default; `--delete-redundant --yes` deletes only server-re-verified
  redundant refs under the repository gate, journaling each deleted tip
  SHA. Failed Works classify active and are never deletable.
- Forge-neutral `upstream = "<url>"` on `[[repo]]` (#112): recorded by
  `sgt repo add --upstream`, ensured as the mount's `upstream` remote at
  clone/repo-add (admission stays remote-agnostic), drift reported by
  doctor with the exact remedy. Any forge, or none — git is the
  assumption, not a CLI.
- `sgt run --intent-file <path>` (#166): the file's contents become the
  intent verbatim; mechanical guards only (symlink refusal, regular file,
  1 MiB cap, UTF-8) — ends the scratchpad-and-`cat` workaround for
  multi-paragraph intents.
- Two workflow packages: `validate-intent` (#201, optional pre-dispatch
  review of an intent across the eight dimensions — reports gaps, never
  fills them) and `record-decisions` (#88, transcribes already-made
  decisions with fidelity-first review; gaps are logged, never invented).
- `docs/proposals/journal-archival-rule-c.md` (#17): the gauntlet-evaluated
  Rule C archival design — ten open questions await a follow-up discussion;
  no behavior change in this release.

## [0.1.1] - 2026-08-20

### Added

- **Terminal Works now report an integrity disposition (#173).** At
  retirement, teardown reconciles what a Work's worktree *actually* held
  against what its binding claimed: the branch HEAD ended on and its tip,
  whether HEAD was detached, and whether the worktree and its source
  checkout still agree on one Git common directory. Divergences are
  recorded as a closed vocabulary of findings
  (`assigned_worktree_uncommitted`, `assigned_worktree_missing`,
  `assigned_branch_mismatch`, `assigned_head_detached_or_unreferenced`,
  `assigned_common_dir_mismatch`), and a terminal Work carries a
  `clean`/`dirty` integrity disposition beside — never inside — its state.
  A dirty completion reports as `completed_dirty`; `failed` and `canceled`
  keep their own state strings and carry the axis in the new `integrity`
  key, which `sgt work list`, `sgt work show`, and the TUI's work detail
  all render. Work state, the state machine, and the transition table are
  unchanged.

  Before this, a worktree that checked out a different branch and committed
  there was reported as a clean removal at the untouched base SHA —
  indistinguishable from a surface nothing ever touched — while removing
  the worktree destroyed the only record of where the output had gone.

- **Teardown no longer removes a worktree holding unreferenced commits.** A
  worktree whose HEAD is detached at a commit no named ref is proven to
  reach is retained (`retained_unreferenced`) rather than removed, and `sgt
  work reap` declines it with the remedy named. A removed worktree's HEAD
  stops being a garbage-collection root; retaining it is how commits
  nothing else points at survive.

- **Estate drift is observed at retirement.** One `git rev-parse HEAD` per
  bound repository mount, compared against the commit the Work was cut
  from. Reported with `attribution: unknown` and never used to make a Work
  dirty: a mount moving during the Work window is not evidence the Work
  moved it.

  Journal changes are additive. A `surface.torn_down` recorded before this
  release replays unchanged and reads as *not assessed* — never as clean.

- **Explicit Work scope is now required for a multi-repository estate
  (estate-root proposal §7).** Submitting `sgt run` with no `--repo`,
  `--group`, or `--all` used to silently expand to every declared
  repository; it is now refused with a structured 422 naming the repo
  count, the declared repositories and groups, and the three ways to
  select — `--repo <name>` (repeatable), `--group <name>`, or `--all`. A
  one-repository estate is unaffected: it still infers its sole repository
  on an empty scope. Group expansion is no longer CLI-side: the daemon
  resolves `--group`/`--repo`/`--all` against its own bound manifest, so
  every client submitting the same scope — CLI, TUI, or a direct API caller
  — reaches the identical resolution. The TUI's New Work form submits that
  same structured scope: its dead `workspace` field is replaced by a
  `group` field, and both it and `repositories` are forwarded unexpanded,
  so naming a group in the TUI resolves exactly as `sgt run --group` does.
  `sgt run --all` is new — an explicit, journaled selection of
  the whole estate. A submitted Work now records both the request form
  (`scope_request`: repos/group/all as submitted) and the resolved
  repository list, so a later manifest edit cannot rewrite what an
  already-journaled Work meant. Owner ruling (2026-08-20): `--all` combined
  with `--repo` and/or `--group` is refused (`conflicting_scope`, 422)
  instead of `--all` silently winning — clap rejects the combination
  locally, and the daemon's own `Engine::resolve_scope` is the authoritative
  check for a direct API caller.

- **`sgt -C <estate-root>`** names an estate explicitly instead of requiring
  a `cd` (gauntlet finding C10, approved by the owner 2026-08-20). It is a
  global flag on every verb, and it names an **exact** root: no search
  happens from it, and it is validated by exactly the rule the current
  directory is. The CLI is agent-first — an agent should not have to mutate
  its own working directory to address an estate.

- **`sgt doctor` gains an `estate_root` row.** It reports whether the
  directory it was run from is an estate root at all — `ok` naming the root
  when it is, `fail` carrying the remedy when it is not. `doctor` still
  works outside an estate, still never searches upward, and still never
  starts a daemon.

- **`sgt doctor` gains a cheap `git_surfaces` row (estate-root proposal
  §12.2).** One bounded summary — active works, active linked worktrees,
  retained worktrees, retained patches, retained artifact size, journaled
  Work branches, terminal dirty Works — derived from the journal plus
  retained-artifact filesystem metadata only, never a per-branch `git`
  walk. Silent (`ok`, "not an estate root") outside an estate; names `sgt
  work retained`/`sgt work show`/`sgt work reap` as the separate
  inspection/cleanup remedy when residue is nonzero. Classifies nothing as
  merged or redundant and deletes nothing — §12.3's expensive
  reconciliation stays future work.

- **Doctrine-skew tests (`tests/f_doctrine_skew.rs`, estate-root proposal
  §18).** New, focused tooling — no `validate-skew` or similar existed
  before this. Checks: AGENTS.md's "Session start" claims (the unscoped
  command set, the root-gate refusal wording, the `-C` flag) against the
  real binary; no `CONTEXT.md` under the embedded `.sergeant/workflows/`
  distro instructs a stage actor to run an estate-scoped `sgt` command from
  inside its own Work surface without a disclaiming note nearby; the
  estate-root proposal's own canonical manifest example (§13.1) still
  parses under the current schema; no shipped workflow/skill content
  quotes the removed `--workspace` flag (C12 regression pin); and the
  embedded `skills/` root/preflight remedies match the refusal text, the
  `--help` surface, and the preflight remedy strings the binary really
  emits.

### Changed

- **Every Work is now admitted against a complete Git preflight before it
  exists (estate-root proposal §8).** Submitting used to read each mount's
  HEAD with no judgment at all: a dirty mount, a detached HEAD, an
  unresolvable commit, an existing `sergeant/<work-id>` ref, an occupied or
  still-registered surface path, or a repository whose lock could not be
  taken all became a durable Work record first and a problem afterwards.
  Core now checks all eleven §8.1 facts for **every** selected repository
  before `work.submitted` is journaled and before any Git mutation, and
  refuses with a structured 422 when any of them is unresolved — so there
  is nothing to clean up, in the journal or in your checkouts.

  Each check has its own stable error code, the evidence observed, and a
  named remedy: `git_preflight_mount_missing`, `_mount_aliased`,
  `_top_level_mismatch`, `_linked_worktree_source`,
  `_common_dir_unlockable`, `_detached_head`, `_unresolvable_head`,
  `_dirty_mount`, `_work_branch_collision`, `_worktree_path_collision`,
  `_incomplete_plan`. A refusal reports every unresolved finding across the
  whole scope, so a multi-repository submission is fixed in one pass rather
  than one submission per repository; when part of a scope cannot be
  planned, the refusal says so explicitly and **no** repository is
  materialized, including the ones that were fine.

  The base a Work runs on is now an *admitted* fact: `base_branch` and
  `base_sha` are what preflight judged, not what the mount happens to say
  by the time `git worktree add` runs. Sergeant still performs no automatic
  network or branch-changing Git command on any admission path — no fetch,
  pull, push, rebase, switch, checkout or remote-default inference — which
  is now asserted against a recording Git binary across a whole real
  admission rather than only stated.

  **Upgrade note:** a mount with uncommitted changes, or on a detached
  HEAD, is refused where it previously ran. Commit or stash, check the
  mount out onto the branch the Work should be based on, or use the new
  bounded override below.

- **`sgt run --override-git-preflight`** waives exactly two of those
  cautions, and only because an exact commit can still be pinned in both
  (§8.3):

  - a **dirty** mount — the Work is based on the committed `HEAD`, the full
    `git status --porcelain` output is journaled as evidence, and the
    record states explicitly that the uncommitted changes are excluded from
    the Work base (they stay in your mount, untouched);
  - a **detached** mount — the exact `HEAD` is pinned and **no** named base
    branch is recorded.

  It never waives anything else: not an invalid estate, unresolved or
  unknown scope, an unknown or aliased repository, an unresolvable top
  level / common directory / commit, a lock conflict, an existing Work ref
  or surface path, a failed surface construction, or a
  backend/workflow/profile failure. An unresolvable `HEAD` says so
  explicitly — the override is unavailable there because no exact base can
  be pinned. Override may waive policy caution; it may not replace a
  missing fact or overcome mechanical impossibility.

  It is available **only** as a flag on `sgt run` (and the matching
  `override_git_preflight` request field). There is no configuration key,
  no `[estate]` key, no profile field and no run template that can set it:
  the operator types it for that submission or it is not set. The
  authorization and every waived finding are journaled with the Work.

- **A repository binding records no base branch when there is none.**
  `base_branch` on a `RepositoryBinding` (and on the `BindingSummary`
  backends receive) is now nullable, and a detached admission records an
  explicit `null` rather than the old `"(detached)"` sentinel — a value
  that read like a branch name in the field every consumer branches on. A
  binding journaled before this change replays exactly what it recorded,
  sentinel included.

- **An estate is now exactly the current directory (estate-root proposal
  §4).** Every estate-scoped command — `run`, `status`, `work *`,
  `respond`/`retry`/`extend`/`cancel`, `watch`, `analytics`, `tui`,
  `daemon`, `repo *`, `group *`, `workflow *`, and the `claude`/`codex`/
  `opencode`/`goose` harnesses — requires the working directory itself to
  contain a `sergeant.toml` that parses, declares `[estate]`, and satisfies
  the schema. **Sergeant no longer searches parent directories**, and no
  longer infers an estate from Git. Running `sgt run` from
  `repos/payments-api` used to find the estate above it; it now refuses,
  names the path it expected, explains that parents are not searched, and
  tells you to `cd` to the root (or `sgt init` here). Only bare `sgt`,
  `--help`, `--version`, `sgt init` and `sgt doctor` work outside a root.

  Validation happens before the data directory is resolved, before the
  runtime descriptor is read, before any daemon is spawned or contacted,
  before any repository is inspected, and before a harness is prepared or
  exec'd — so a directory mistake cannot attach to, or spawn, the wrong
  daemon.

  **Upgrade note:** if you have been running `sgt` from inside a repository
  mount, `cd` to the estate root or pass `-C <estate-root>`. If you relied
  on the zero-configuration single-repository mode — a plain git checkout
  with no `sergeant.toml` — run `sgt init` there once; a one-repository
  installation is now an estate with one declared repository.

- **A daemon belongs to one estate.** Daemon startup takes a canonical
  estate root and refuses to come up if it is not one. The runtime
  descriptor records `estate_root` and `manifest_path`, and every client
  verifies that root against its own before using the endpoint: a daemon
  bound to another estate is a named refusal listing both roots, never a
  connection and never a second daemon over the same data dir. The engine
  plans against that bound estate rather than rediscovering topology from
  each request's working directory, which removes the recursion hazard
  where a command launched from inside a Work surface rediscovered that
  linked worktree as a new workspace. `origin.cwd` is still recorded, as
  evidence only.

  The descriptor schema is `sergeant.runtime/v2`. There is no compatibility
  shim: a `v1` descriptor left by an older build carries no estate root, so
  a client cannot verify the binding at all and fails closed with the
  remedy — stop the old daemon and let a restarted one republish.

- **Repository mounts are derived, not configured (§6).** `[[repo]] path`
  is **removed** from `sergeant.toml`. Every repository is mounted at
  `<estate-root>/repos/<name>`, and that is the only place it can be. A
  manifest still declaring `path` is refused with a message naming the
  removal, not a generic unknown-field error. Mounts are validated on load:
  a missing mount, a symlinked or aliased one whose real Git top level is
  elsewhere, and a linked worktree offered as a repository source are each
  refused by name, reporting the expected derived path alongside the actual
  top level or common directory. Separate estates use separate clones, even
  for the same upstream repository.

  **Upgrade note:** delete every `path = "..."` line from your
  `sergeant.toml`'s `[[repo]]` entries. If a checkout is not already at
  `repos/<name>`, move or re-clone it there. `sgt repo add` writes the new
  shape and clones to exactly that path; `sgt repo remove` still undeclares
  without deleting the checkout.

- **`sgt <harness>` binds the estate explicitly.** It validates the exact
  root first, then exports `SGT_ESTATE_ROOT`, `SGT_DATA_DIR` and
  `SGT_ORIGIN_CLIENT` and starts the harness in the root. The environment
  helps later invocations name the correct root; it never waives the
  exact-root check — `cd` into a mount inside a bound session and `sgt run`
  still refuses, naming both roots and how to return.

### Removed

- **Upward estate discovery and the zero-configuration Git fallback are
  gone.** `Workspace::discover`, `discover_scoped`, the `find_estate_upward`
  ancestor walk and the `git rev-parse --show-toplevel` fallback beneath it
  are deleted outright, not merely left uncalled. R-MVP1-12 is superseded;
  ADR 0008 carries an amendment recording that the manifest keeps its
  storage-path authority while the *discovery* of the manifest becomes
  exact-root only.

- **`--workspace` is gone, from the CLI and the wire.** The daemon is bound
  to exactly one estate; a client-supplied workspace label had no role left
  to play. `Work.workspace` is no longer written by any new submission —
  `scope_request` and the resolved `repositories` list are its replacement
  — but the field itself still deserializes so a pre-existing journal (the
  live estate journals 150+ Works carrying it) keeps replaying unchanged.
  Analytics and the provenance graph now read the estate label off the
  plan-time `workflow.bound` event instead of the submission, and tolerate
  its absence for a Work that never reached a workspace at all.

### Documentation

- **Corrected: the shell installer does not verify downloads.** `dist`'s
  generated `sergeant-rs-installer.sh` (published from v0.1.0 onward)
  declares the local variables it would use to check a downloaded
  archive's checksum but never assigns them anywhere in the script, so its
  verification branch is structurally unreachable — every install via the
  `curl | sh` convenience one-liner prints "no checksums to verify" and
  installs unverified, confirmed by running the published v0.1.0 installer.
  This was previously undocumented; README.md's "Installing a released
  binary instead of building from source" section now states this plainly
  and gives a manual, deliberate verification path instead: download the
  archive and its `.sha256`, `sha256sum -c` it, then `gh attestation
  verify` it against this repo's build-provenance attestation, then
  extract and install by hand. `.github/workflows/release.yml`'s
  `package-installer` job comment is corrected to match — it previously
  read as though the shipped installer consumed the checksums `dist` also
  generates; it does not.

- **AGENTS.md gains a session-start invariant and an estate/Git model table
  (estate-root proposal §14.1/§14.2).** A new "Session start" section
  states the exact-root rule up front — no upward search, no Git fallback,
  the same remedy the real root-gate diagnostic gives — and names which
  four commands work outside an estate; a new "Estate and Git model" table
  fixes the vocabulary (estate root, `repos/<name>`, the surfaces
  directory, `sergeant/<work-id>`) before the routing table uses it. A new
  "ESTATE — Captain's estate discipline" section (§14.3/§14.4) states
  Captain's pre-Work checklist and what a worker never does (edit a mount,
  create a replacement branch, navigate into another Work's surface,
  expand its own scope, invoke an estate-scoped command from its own
  surface). "Captain captains" lands as emphasis, per the owner's
  2026-08-20 ruling (gauntlet finding C1): Captain's normal mode is
  dispatching Work and shaping intent, not writing code turn by turn — the
  existing ROUTING table's in-session allowances (BU-0004/BU-0009) are
  unchanged, not narrowed. `CAN — enforceable authority` gains three
  bullets for behavior already enforced: the root gate, the Git preflight,
  and durable branch retention.

- **README.md and docs/glossary.md corrected off the exact-root contract.**
  The data-dir precedence description no longer describes upward directory
  search; `sgt run`'s documented flags drop the removed `--workspace` and
  add `--all`/`--override-git-preflight`; "the workspace's own
  `software-change` workflow" is corrected to "the estate's own."
  `docs/glossary.md` gains the estate-root proposal's seven §14.7 terms:
  Estate, Repository Mount, Work Scope, Repository Binding, Work Surface,
  Integrity Disposition, Estate Drift.

- **Embedded distro swept for exact-root skew (C12 and §14.6).**
  `.sergeant/workflows/dispatch/05-classify-risk/CONTEXT.md`'s restated
  `sgt run --help` option list drops the removed `--workspace` flag (the
  one instance C12 named, confirmed the only one by a full sweep of
  `AGENTS.md`, `skills/`, `.sergeant/common/contexts/`, and
  `.sergeant/workflows/`) and adds `--all`/`--override-git-preflight`/the
  global `-C`/`--data-dir`/`--json` flags it was missing. Six sites that
  described a stage actor
  dispatching nested Work or delivering an escalation response as its own
  literal `sgt run`/`sgt respond` invocation from inside its own Work
  surface — `implement/30-review`, `implement`'s own `CONTEXT.md`,
  `worker-mission/20-implement`, `worker-mission`'s own `CONTEXT.md`, and
  `dispatch/80-monitor` plus `dispatch`'s own `CONTEXT.md` — gain an
  explicit note that the worker's own Work surface is not an estate root
  and the command would refuse from there today; the actual submission is
  Captain's, from the estate root.

- **Embedded skills rewritten for the exact-root front door (§14.6).**
  `skills/sergeant-help` gains the loud root and preflight remedies it was
  silent on: documentation-map rows for "which directory must this command
  run from" and "why was my submission refused for a dirty or detached
  mount", failure-behavior rows repeating the refusals' own remedies
  verbatim (`cd <estate-root>`, `sgt -C <estate-root> <command>`, `sgt
  init`; `git -C <mount> status`/`switch <branch>`, with
  `--override-git-preflight` described as the per-submission waiver of a
  dirty or detached mount and nothing else), and a must-not bullet against
  routing around either refusal. `skills/estate-navigation` now teaches the
  exact-root check itself — look for this directory's own `./sergeant.toml`,
  never walk upward, `-C` to name a root without moving — and its `sgt
  doctor` description is brought current with the `estate_root`,
  `workflows`, and `git_surfaces` rows.

- **ADR 0008's estate-root amendment verified, not redone.** Phase D
  already amended it ("Amended by the estate-root integration (C7a,
  2026-08-20)"): the manifest keeps storage-path authority, discovery
  becomes exact-root, and R-MVP1-12 is marked superseded. Confirmed
  current against this phase's own doctrine rewrite; no further change
  needed for C7a.

## [0.1.0] - 2026-08-19

First release. `sgt` is an AgentOS distro: instructions, skills, and
workflow templates embedded in the binary and written to your estate by
`sgt init`, turning a general-purpose coding harness (Claude Code today)
into an operator of your estate, carried by a durable intent-execution
engine that runs those intents to completion in isolated worktrees.

### Added

- **`sgt init` scaffolds an estate and embeds the distro.** A fresh
  `sgt init` writes `sergeant.toml`, `repos/`, `.gitignore`, and now also
  the full embedded distro — `AGENTS.md`, `skills/`,
  `.sergeant/common/contexts/`, and 17 workflow packages under
  `.sergeant/workflows/` — per-file idempotent, so re-running `sgt init`
  against an existing estate is a no-op rather than an overwrite (#179).
- **Local-shadows-stock workflow resolution and `sgt workflow fork`.** A
  workflow package you author locally under `.sergeant/workflows/` takes
  precedence over a stock package of the same name shipped in the distro;
  `sgt workflow fork` copies a stock package into your estate as a starting
  point for editing. The shipped packages are examples and defaults, not
  published procedure you're expected to follow as-is (ADR 0014 decision 3).
- **`sgt doctor`** checks estate health with named faults and named
  remedies, including: the estate-local data directory now resolves and is
  reported correctly on a brand-new estate instead of falling back to the
  pre-estate XDG/HOME default (#164), and a fresh or workflow-less estate
  now gets an explicit `workflows` check reporting zero packages and
  naming `sgt init`/`sgt workflow fork` as the remedy, instead of a bare
  422 the next time you try to dispatch a named workflow (#165).
- **Release pipeline** (`.github/workflows/release.yml`): `workflow_dispatch`
  only, `dry-run` default, no tag/push/schedule trigger. Runs Gates A-F
  (repository-state, `ci.yml` reuse, `matrix.yml` reuse, `coverage.yml`
  reuse, strict `cargo deny check`, and a documented Gate F no-op — see
  Known gaps below) before packaging, smoke-testing, generating a SHA-256
  manifest and a per-target CycloneDX SBOM, generating GitHub
  build-provenance and SBOM attestations, assembling a draft GitHub
  Release, verifying its manifest, and — only in `mode: publish` —
  publishing it. `.github/workflows/ci.yml` gained a `workflow_call`
  trigger so Gate B reuses CI's exact contract rather than redefining
  "tests passed."
- **Supply-chain posture:** every third-party GitHub Action referenced
  across `ci.yml`, `matrix.yml`, `coverage.yml`, and `release.yml` is
  pinned to a commit SHA, not a mutable tag; `cargo-deny` enforces
  bans/licenses/sources on every PR and the full advisory-inclusive check
  at release time (Gate E); CodeQL default setup scans Rust and Actions
  weekly; GitHub's `dependency-review-action` diffs dependency manifests
  on every pull request; Dependabot runs weekly, grouped updates for both
  Cargo and GitHub Actions dependencies; and release artifacts ship with a
  CycloneDX SBOM per target plus GitHub build-provenance and SBOM
  attestations.
- **Packaging configuration** (`[workspace.metadata.dist]` and
  `[profile.dist]` in `Cargo.toml`): `dist` (cargo-dist 0.32.0) covers the
  two target triples ADR 0001 names as release targets,
  `x86_64-unknown-linux-gnu` and `aarch64-apple-darwin`.

### Platform boundary (ADR 0001, ADR 0018)

- **`x86_64-unknown-linux-gnu` is measured**: built, packaged, and
  smoke-tested (`sgt --version`) on a Linux host, with `probe-env.sh` and a
  published-skip-count full suite run backing the "measured" label.
- **`aarch64-apple-darwin` is built and packaged, but NOT validated.**
  `dist build` succeeds there on a real `macos-latest` runner, so it ships
  in this release — a clean build is what ADR 0018 obliges from an
  unmeasured platform — but binary-equivalence and generated-installer
  checks were only run on `ubuntu-latest`. Do not read "packaged" as
  "measured" for macOS in this release; that label is earned separately,
  on the owner's own schedule.

### Known gaps in this release pipeline (recorded, not hidden)

- **No curl-pipe-sh installer is published yet.** `dist`'s generated
  shell installer is not part of this release's artifact set.
- **Co-versioning is real for `sgt init`, not yet for update semantics.**
  ADR 0014 decision 2 names one artifact identity: the `sgt` binary
  together with the embedded distro it writes. As of this release
  `sgt init` does embed and write that distro (closing the gap #165
  originally only made visible), but per ADR 0014 decision 2 the distro
  ships embedded *in the binary* — there is no separate distro artifact or
  update channel yet; getting a newer distro means installing a newer
  `sgt`.
- **Gate F (distro structural validator) does not run in this repo's CI.**
  It lives in `sergeant-rs-workspace`'s
  `.sergeant/local/workflows/validate-distro/` by deliberate placement
  (ADR 0014 decision 5) and this repo's CI cannot reach it. `release.yml`'s
  `gate-f-distro-validator` job is a labelled skip, not a passing check.
  Gap filed as issue #176; that property is instead proven continuously by
  `sergeant-rs-workspace`'s own CI against this repo's `main`.
- **GitHub-hosted-runner disk/cache bounds for `dist build` are
  unverified.** The measured Linux build ran on a workstation with ~780 GB
  free; GitHub-hosted runners start with a much smaller free-disk budget,
  and `coverage.yml` already has to reclaim disk before its own DuckDB
  build. Whether `dist build`'s cold build fits GitHub's runner disk budget
  without the same reclaim step was not measured here.

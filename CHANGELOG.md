# Changelog

All notable changes to sergeant-rs are recorded here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/); Sergeant is pre-1.0, so
ordinary SemVer 0.x.y semantics apply and every release is potentially
breaking (proposal-ci-cd-release-engineering.md §8).

`release.yml`'s Gate A requires this file to mention the version being
released before a release can proceed.

## [Unreleased]

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
  already-journaled Work meant.

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

### Changed

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

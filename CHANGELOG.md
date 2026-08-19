# Changelog

All notable changes to sergeant-rs are recorded here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/); Sergeant is pre-1.0, so
ordinary SemVer 0.x.y semantics apply and every release is potentially
breaking (proposal-ci-cd-release-engineering.md §8).

`release.yml`'s Gate A requires this file to mention the version being
released before a release can proceed.

## [Unreleased]

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

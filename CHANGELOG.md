# Changelog

All notable changes to sergeant-rs are recorded here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/); Sergeant is pre-1.0, so
ordinary SemVer 0.x.y semantics apply and every release is potentially
breaking (proposal-ci-cd-release-engineering.md §8).

`release.yml`'s Gate A requires this file to mention the version being
released before a release can proceed.

## [Unreleased]

### Added

- `.github/workflows/release.yml`: the release pipeline. `workflow_dispatch`
  only, `dry-run` default, no tag/push/schedule trigger. Runs Gates A-F
  (repository-state, `ci.yml` reuse, `matrix.yml` reuse, `coverage.yml`
  reuse, strict `cargo deny check`, and a documented Gate F no-op — see
  below) before packaging, smoke-testing, generating a SHA-256 manifest and
  a per-target CycloneDX SBOM, generating GitHub build-provenance and SBOM
  attestations, assembling a draft GitHub Release, verifying its manifest,
  and — only in `mode: publish` — publishing it.
- `.github/workflows/ci.yml`: added a `workflow_call` trigger so
  `release.yml`'s Gate B reuses this workflow's exact contract instead of
  redefining "tests passed."
- `[workspace.metadata.dist]` and `[profile.dist]` in `Cargo.toml`: packaging
  configuration for `dist` (cargo-dist 0.32.0), covering the two target
  triples ADR 0001 measures as buildable release targets:
  `x86_64-unknown-linux-gnu` and `aarch64-apple-darwin`.

### Known gaps in this release pipeline (recorded, not hidden)

- **Co-versioning is not yet real.** ADR 0014 decision 2 names one artifact
  identity: the `sgt` binary *and* the embedded distro it writes via
  `sgt init`. `sgt init` does not embed or write the distro yet
  (NORTH-STAR.md's "Not true yet" note, 2026-08-17, is unchanged by this
  work). Issue #165 tracked the visible symptom of this gap but closed
  2026-08-17 via a visibility-only fix (`sgt doctor` now reports a
  zero-package estate); it explicitly left full embedding to Phase 3
  (`reference/proposal-product-workspace-split.md`), which has no dedicated
  open issue yet. `release.yml` packages and ships the binary alone and
  says so in the generated draft release body. A release published today
  would be the binary only, not the co-versioned artifact ADR 0014
  describes as the destination.
- **Gate F (distro structural validator) does not run.** It lives in
  `sergeant-rs-workspace`'s `.sergeant/local/workflows/validate-distro/` by
  deliberate placement (ADR 0014 decision 5) and this repo's CI cannot
  reach it. `release.yml`'s `gate-f-distro-validator` job is a labelled
  skip, not a passing check. Gap filed as issue #176.
- **aarch64-apple-darwin packaging is unverified end-to-end.** `dist build`
  was measured successfully against `x86_64-unknown-linux-gnu` on a Linux
  host (bundled DuckDB compiled, archive produced, binary smoke-tested with
  `sgt --version`). No macOS runner was available to verify the equivalent
  build, its cache/disk behavior, or the smoke test on that target from
  this environment.
- **GitHub-hosted-runner disk/cache bounds for `dist build` are
  unverified.** The measured build ran on a workstation with ~780 GB free;
  GitHub-hosted runners start with a much smaller free-disk budget, and
  `coverage.yml` already has to reclaim disk before its own DuckDB build.
  Whether `dist build`'s cold build fits GitHub's runner disk budget
  without the same reclaim step was not measured here.

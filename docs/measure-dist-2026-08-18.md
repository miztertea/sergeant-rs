# Measurement: does `dist` build bundled DuckDB on GitHub-hosted runners?

**Date:** 2026-08-18. **Branch:** `measure/dist-duckdb` (off `origin/main` @
`f3c40e1`). **Rung:** R2 (Ponytail) — this measurement reuses the
`[workspace.metadata.dist]` config and `cargo-dist-version = "0.32.0"`
already pinned in `Cargo.toml`, and the exact `dist build` invocation
`.github/workflows/release.yml`'s `package` job already uses. No new
packaging machinery was invented to answer this question.

This is a measurement, not an adoption decision. It answers proposal §22
unknown 1 ("DuckDB + dist. Does standard dist packaging build the bundled
DuckDB crate reliably, within sane disk and cache bounds, on each release
runner?") and the first three of proposal §10's five conditions for
admitting `dist` (builds successfully with bundled DuckDB; cache/disk
behavior sane). It does not touch conditions 2-4 (binary behavioral
equivalence, installer correctness, dispatch-without-tag-trigger topology)
— those are separate, unmeasured questions.

**Relocation note:** this belongs in the workspace repo's
`knowledge/evidence/` substrate per the standing convention. Only
`sergeant-rs` is mounted in this session, so it was written here instead;
move it to `knowledge/evidence/` when next in the workspace repo.

## The question

For target triples `x86_64-unknown-linux-gnu` and `aarch64-apple-darwin`:
can `dist` produce a release archive on a standard GitHub-hosted runner,
and at what cost in wall clock, disk, and cache?

## Method

A temporary, `.github/workflows/measure-dist.yml` workflow ran one job per
target (`ubuntu-latest` / `macos-latest`), each installing `dist` v0.32.0
and running `dist build --artifacts=local --target <triple>` — the same
command `release.yml`'s `package` job runs. Each job recorded `df -h`
before/after the build, `du -sh target` and its largest subdirectories,
`rust-cache`'s cache-hit output, per-step wall clock, and the produced
archive's size.

**Trigger deviation, and why:** the task specified `workflow_dispatch` +
`gh workflow run`. GitHub only registers a `workflow_dispatch` trigger for
manual dispatch once the workflow file exists on the repository's default
branch — a hard platform constraint confirmed here by a 404 from both
`gh workflow run` and `gh api .../actions/workflows` (the file was absent
from `gh workflow list`'s output while only present on this branch).
Touching `main` was out of scope for this task, so the workflow's trigger
was switched to `push`, scoped to exactly `refs/heads/measure/dist-duckdb`
(rung R7 — the minimum change that still gets a real GitHub-hosted runner
without touching `main` or changing the repo's default branch). This is
itself a finding: **`workflow_dispatch`-only measurement workflows cannot
be run from a feature branch without either a default-branch merge or a
substitute trigger** — worth remembering for any future measurement that
assumes `gh workflow run` "just works" against a non-default ref.

Run: https://github.com/miztertea/sergeant-rs/actions/runs/32147226850
(triggered by commit `5e2ac6a` on `measure/dist-duckdb`).

## Result: it works, on both targets, on a first (cold) build

| | `x86_64-unknown-linux-gnu` (`ubuntu-latest`) | `aarch64-apple-darwin` (`macos-latest`) |
|---|---|---|
| Outcome | **success** | **success** |
| `dist build` wall clock | 897s (14m57s) | 1002s (16m42s) |
| Total job wall clock (incl. setup/upload) | ~15m16s | ~17m27s |
| `dist` install | 1s | 1s |
| Disk before build | 145G total, 58G used, 87G avail (40%) | 320Gi total, 203Gi used, 97Gi avail (68%) |
| Disk after build | 145G total, 60G used, 85G avail (42%) | 320Gi total, 205Gi used, 95Gi avail (69%) |
| Disk consumed by the build | ~2G | ~2Gi |
| `target/` size | 1.3G | 1.1G (845M `target/aarch64-apple-darwin`, 253M `target/dist`, 64M `target/distrib`) |
| Archive produced | `sergeant-rs-x86_64-unknown-linux-gnu.tar.xz`, **14M** | `sergeant-rs-aarch64-apple-darwin.tar.xz`, **10M** |
| `rust-cache` hit/miss | miss (expected — first build on a new cache key) | miss (expected — first build on a new cache key) |

Both jobs completed with no disk pressure and no timeout: runners started
with 85-97 GB free and finished with roughly the same margin (GitHub's
standard `ubuntu-latest`/`macos-latest` runners ship with far more free
disk than this repo's own dev-container constraint — `docs/DEVELOPMENT.md`'s
~15 GB `target/` ceiling on a ~16 GB disk never came close to being
threatened here). `dist build` completed the ~500-translation-unit bundled
DuckDB C++ compile as part of the normal Rust build graph — nothing
DuckDB-specific broke or needed separate handling.

## What this does NOT show

- **Cache hit behavior on a warm cache.** This was necessarily each
  target's first build on a fresh `rust-cache` key (a brand-new branch), so
  both runs show a cold miss by construction. A second push to the same
  branch would show whether `rust-cache` actually shortens the ~15-17
  minute cold build materially — not measured here, out of scope for a
  single-build cost/feasibility question.
- **Peak disk usage during the build**, only before/after snapshots. If
  `dist`/`cargo` transiently allocate more than the after-build total
  before cleaning up intermediate objects, that peak is invisible to this
  measurement.
- **macOS x86_64, or any architecture beyond the two named in ADR 0001's
  measured set / proposal §11.** Not attempted.
- **Binary behavioral equivalence, installer correctness, or the
  dispatch-without-tag-trigger topology** — proposal §10 conditions 2-4,
  genuinely separate questions this run does not touch.

## Verdict: is proposal §10's "default candidate is Axo dist" still right?

**Yes, unchanged by this measurement.** `dist` built the bundled-DuckDB
crate successfully for both currently-measured targets, on standard
GitHub-hosted runners, in well under GitHub Actions' job time limits, using
a small single-digit-percent slice of available runner disk. Nothing here
surfaces a reason to abandon `dist` or fall back to proposal §10's escape
hatch ("if one fails, preserve the rest and replace only the failing
piece") — no piece failed. The proposal's own conditions 2-4 remain
unmeasured and should each get their own targeted measurement before full
adoption is called done; this run closes condition 1 and the disk/cache
half of condition 5 for the two named targets, nothing more.

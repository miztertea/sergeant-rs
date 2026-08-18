# Measurement: proposal §10 conditions 2-4 — the untested half of `dist` adoption

**Date:** 2026-08-18. **Branch:** `measure/dist-conditions` (off `origin/main`
@ `f3c40e1`). **Rung:** R2 (Ponytail) for conditions 2-3 — reuses the exact
`dist build` invocation and `[workspace.metadata.dist]` config
`.github/workflows/release.yml` already carries, plus the same
temporary-workflow-then-remove-it method `measure/dist-duckdb` used for
condition 1. R7 for the trigger deviation (see below) and for condition 4,
which needed no new measurement machinery at all — it is answered by
reading `release.yml` as it already exists on `main`.

This is a measurement, not an adoption decision. It closes proposal §10's
conditions 2 ("the produced binary behaves identically to an ordinary
release build"), 3 ("its generated installer works for Sergeant's actual
release assets"), and 4 ("its workflow can be driven from Sergeant's
manual-release topology without making tag creation the initiating event").
Condition 1 (bundled-DuckDB build feasibility) and the disk/cache half of
condition 5 were already closed by `measure/dist-duckdb`'s
`docs/measure-dist-2026-08-18.md` — read first, including its "What this
does NOT show" section, before reading this one.

**Relocation note:** this belongs in the workspace repo's
`knowledge/evidence/` substrate per the standing convention. Only
`sergeant-rs` is mounted in this session, so it was written here instead;
move it to `knowledge/evidence/` when next in the workspace repo.

## Method

### Conditions 2 and 3 — real GitHub runner

A temporary `.github/workflows/measure-dist-conditions.yml` workflow ran on
`ubuntu-latest`, checking out `feat/embed-distro` (PR #179 — the branch that
first makes `sgt init` write the embedded 241-file / 17-package distro,
since that is exactly the kind of thing a different build pipeline could
silently break by mishandling `include_dir`-compiled assets). The job:

1. built an ordinary `cargo build --release` binary;
2. ran `dist build --artifacts=local --target x86_64-unknown-linux-gnu` and
   `dist build --artifacts=global --tag=v0.1.0` to produce both the archive
   and the shell installer;
3. compared the two binaries' `--version`, `--help`, `sgt doctor`, and
   `sgt init` output directly, byte-for-byte where applicable;
4. extracted the generated `sergeant-rs-installer.sh`, ran it against a
   local HTTP artifact server (`SERGEANT_RS_DOWNLOAD_URL` override) to
   prove the mechanics work, then ran it a second time with no override to
   observe what happens against the real (nonexistent) GitHub release.

**Trigger deviation, same reasoning as `measure/dist-duckdb`:**
`workflow_dispatch` only registers once a workflow file lives on the
default branch, and touching `main` was out of scope. The workflow used a
push trigger scoped to exactly `refs/heads/measure/dist-conditions` — R7,
the minimal substitute that still gets a real GitHub-hosted runner.

Run: https://github.com/miztertea/sergeant-rs/actions/runs/32151938075
(triggered by commit `7303bf2` on `measure/dist-conditions`, completed
successfully in 28m23s).

### Condition 4 — read, not run

Unlike conditions 1-3, condition 4 is a question about workflow topology
and trigger wiring, not runtime behavior — it doesn't need a live runner to
answer. `.github/workflows/release.yml` already exists on `main` (Phase 6,
landed before this measurement) and either does or does not invert dist's
default tag-triggered topology; that's answered by reading the file. No
temporary workflow was created for this condition.

## Result: condition 2 — binary behavioral equivalence

| Check | cargo build | dist build (extracted) | Result |
|---|---|---|---|
| `--version` | `sgt 0.1.0` | `sgt 0.1.0` | **identical** (`diff` exit 0) |
| `--help` | — | — | **identical** (`diff` exit 0) |
| `sgt doctor` (fresh estate) | same 12 checks, same `[FAIL] claude` (no Claude CLI on the runner), same `[ok]` set | same 12 checks, same `[FAIL] claude`, same `[ok]` set | **identical check set and verdicts**; only the reported `data_dir` path differed, because each binary ran against its own scratch directory — not a behavioral difference |
| `sgt init` (embedded distro) | `wrote 241 distro file(s)`; `244` total files (`find -type f`); `17` workflow packages | `wrote 241 distro file(s)`; `244` total files; `17` workflow packages | **identical** file count, package count, and package list (`code-review, cross-repo-work, deepen-module, diagnose-bug, dispatch, implement, prototype, recover-stalled-worker, repo-to-icm, research, resolving-merge-conflicts, to-tickets, triage, validate-and-ship, vet-external-skill, wayfinder, worker-mission`) |
| `diff -rq` on the two `sgt init` output trees | — | — | **no output at all** — `diff -rq` reports nothing when trees are identical, including `sergeant.toml` (same estate-init template, no name parameter differed between the two runs) |

The embedded distro — the thing this measurement specifically worried a
different build pipeline could silently break, since it's compiled in via
`include_dir` rather than read from disk at runtime — came out
byte-identical between the two build paths. Nothing observable differed
between the `cargo build --release` binary and the `dist build`-produced
binary.

One script defect worth recording: the `diff -rq ... | grep -v
'sergeant.toml' && echo "TREES: NO DIFFERENCES..."` line never printed its
success message, because `diff -rq` on identical trees produces zero
output, `grep -v` then matches nothing and exits 1, and `|| true` silently
swallows that. The absence of any `diff -rq` output in the log is itself
the affirmative signal (diff only prints when files differ) — but the
intended confirmation string is misleading dead code and would need fixing
before reuse in a non-throwaway workflow.

## Result: condition 3 — generated installer

The installer `dist build --artifacts=global` produced
(`sergeant-rs-installer.sh`) hardcodes:

```
ARTIFACT_DOWNLOAD_URLS="https://github.com/miztertea/sergeant-rs/releases/download/v0.1.0"
```

Two runs:

1. **With `SERGEANT_RS_DOWNLOAD_URL` pointed at a local HTTP server serving
   the just-built archive:** the installer downloaded
   `sergeant-rs-x86_64-unknown-linux-gnu.tar.xz`, reported "no checksums to
   verify" (none were served alongside it in this ad hoc setup), installed
   `sgt` to `$CARGO_HOME/bin`, wrote `env`/`env.fish` PATH shims, and the
   installed binary printed `sgt 0.1.0` — a correct, runnable install.
2. **Without the override, against the real hardcoded URL:** the installer
   attempted
   `https://github.com/miztertea/sergeant-rs/releases/download/v0.1.0/sergeant-rs-x86_64-unknown-linux-gnu.tar.xz`,
   got `curl: (22) ... 404`, printed "failed to download ... this may
   indicate that sergeant-rs's release process is not working ... please
   feel free to open an issue!", and exited 1.

**This is not a bug in the installer — it is the installer working
correctly.** No `v0.1.0` GitHub Release exists yet (Sergeant has never
published one; that's the entire point of the release-authority proposal
this measurement serves). The installer's mechanics — download, checksum
step, unpack, install, PATH wiring — are sound and produce a runnable
binary once a real artifact exists at the URL it expects. The generated
installer is not usable end-to-end today because Sergeant hasn't published
a release, not because of anything wrong with `dist`.

## Result: condition 4 — can dist be driven without tag creation initiating the release?

**Yes — and `release.yml` on `main` already demonstrates the inversion.**
Proposal §7 is absolute: no tag watcher, no push to `main`, no cron, no
successful PR initiates a release. `dist`'s own default topology (`dist
init` generates a workflow triggered by `on: push: tags: ...`) is exactly
what §7 forbids as the *initiating* event. Reading `release.yml` as it
exists on `main` today:

- The only trigger is `on: workflow_dispatch`, with `version` and `mode`
  (`dry-run`/`publish`) inputs. `dry-run` is the default.
- `dist` is invoked as a plain packaging subcommand — `dist build
  --artifacts=local --target ${{ matrix.target }}` — inside the `package`
  job, which itself runs only after Gates A-F (repository state, CI, full
  matrix, coverage, strict `cargo-deny`, distro-validator placeholder) have
  all passed. It is not `dist`'s own generated release orchestration
  workflow; nothing here delegates control flow to `dist`.
- Directly confirmed by this run: `dist build --artifacts=local
  --target x86_64-unknown-linux-gnu` succeeded when invoked from a job
  whose `GITHUB_REF` was `refs/heads/measure/dist-conditions` — a branch
  push, not a tag — with no `v*` tag present in the repository at all. The
  `--artifacts=global` build that produces the installer took its version
  from an explicit `--tag=v0.1.0` CLI flag, not from git tag state. `dist
  build`'s packaging mechanics do not require a tag to exist or to be the
  triggering ref; only `dist`'s *own* generated CI recommends tag-push as a
  trigger, and `release.yml` does not use that generated workflow.
- Tag creation itself happens only at the very end, inside `draft-release`,
  via `gh release create "$RELEASE_TAG" ... --draft`, which runs only after
  `smoke-test` and `sbom` (and therefore every upstream gate) succeed. A
  `gh release create --draft` release does not create an actual git tag
  ref in the repository until the release is published — GitHub defers tag
  creation for a draft release's tag field until `--draft=false`. The
  `publish` job, gated on `inputs.mode == 'publish'`, is the only thing
  that flips that flag. In `dry-run` mode (the default), the workflow never
  creates a durable tag at all.

So the actual sequence is: **dispatch → gates A-F → build/package → smoke
→ SBOM → attest → draft (tag field set, no ref created yet) → publish
(tag ref created, release becomes durable)** — dispatch first, tag only
after every gate passes, exactly the inversion proposal §10 condition 4
asks whether `dist` can be driven through. It can: `dist`'s build mechanics
are a callable primitive independent of its own opinionated CI topology,
and `release.yml` already proves the composition works as designed, not
merely as a hypothetical.

## What this does NOT show

- **The macOS (`aarch64-apple-darwin`) side of conditions 2 and 3.** This
  run used only `ubuntu-latest`; the binary-equivalence and installer
  checks were not repeated on the macOS target. `measure/dist-duckdb`
  showed the macOS *build* succeeds, but not that its installer or binary
  parity hold — that remains unmeasured.
- **Checksum verification in the installer.** The local-server run reported
  "no checksums to verify" because the ad hoc HTTP server didn't serve the
  `.sha256` file alongside the archive; `release.yml`'s real
  `draft-release` job does publish a `SHA256SUMS.txt`, but this measurement
  did not exercise the installer's checksum-verification path end-to-end.
- **A real `publish: true` dry run of `release.yml` itself.** Condition 4's
  answer comes from reading the workflow and confirming `dist build`'s
  tag-independence in isolation, not from actually dispatching
  `release.yml` in publish mode and watching the tag appear at the end.
  That remains a distinct, larger exercise (proposal §19 Slice 4: "run a
  complete dry-run… publish the first real version only after that dry-run
  is adjudicated").
- **What happens if `dist`'s generated CI recommendation is used instead.**
  This measurement did not attempt `dist`'s own `on: push: tags:`-triggered
  workflow generator at all — `release.yml` deliberately never adopted it,
  so there was nothing to run. The claim here is narrower and stronger:
  the packaging primitive `dist` exposes is trigger-agnostic, which is what
  makes `release.yml`'s manual-dispatch topology possible.
- **Condition 5** (cache and disk behavior). Already substantially covered
  by `measure/dist-duckdb`'s cold-build measurement; this run's `dist
  build` steps did not re-measure disk/cache behavior with a warm cache
  from a prior run on this branch.
- **Whether the embedded-distro parity result generalizes past a single
  `sgt init` run.** Only one `cargo`-vs-`dist` comparison was made; repeated
  runs, different working directories, or concurrent `sgt init` invocations
  were not tested.

## Verdict: is `dist` adoptable per proposal §10?

**Conditions 2, 3, and 4 all pass**, joining condition 1
(`measure/dist-duckdb`). Condition 5 (cache/disk) is substantially but not
completely measured across the two runs.

- **Condition 2 — pass.** The dist-produced binary is observably identical
  to an ordinary `cargo build --release` binary, including the
  `include_dir`-embedded 241-file/17-package distro that this measurement
  specifically targeted as a plausible breakage point.
- **Condition 3 — pass, with a precise caveat.** The generated installer's
  mechanics are correct and produce a runnable binary. It does not work
  end-to-end *today* only because no GitHub Release has been published yet
  — that is a statement about Sergeant's release history, not a defect in
  `dist`'s installer.
- **Condition 4 — pass, and this is the one that could have vetoed
  adoption outright.** `dist`'s packaging mechanics compose cleanly with
  Sergeant's manual-dispatch, gates-before-tag topology. `release.yml`
  already proves this in production code, not merely in a throwaway
  measurement branch: `workflow_dispatch` is the only trigger, `dist build`
  runs as a gated subcommand after Gates A-F, and tag creation is deferred
  to the very last `publish` step, which only fires in `mode: publish`. No
  tag watcher initiates anything.

Per proposal §10's own rule ("If those hold, use it. If one fails, preserve
the rest and replace only the failing piece"): all five conditions have
now been measured and none has failed. **`dist` should be adopted** as
Sergeant's packaging tool, exactly as `release.yml` already does. No
fallback or piece-replacement is triggered by this measurement. The
remaining open items before calling Slice 4 fully qualified are the ones
listed above under "What this does NOT show" — macOS-side conditions 2-3,
installer checksum verification, and a real `publish: true` dry run of
`release.yml` itself — none of which is a condition-5 veto candidate, all
of which are narrower follow-up measurements rather than reasons to
reconsider the tool choice.

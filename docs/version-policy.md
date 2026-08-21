# Version policy

How sergeant-rs pins its dependencies, tools, and runners — and, just as
importantly, what it deliberately does *not* pin and why. Written so a
future audit does not re-litigate a decision already made here. Source:
the CI/CD hardening sprint plan (`sprint-plan-2026-08-20.md`), "The
one-rule target state":

> Discover the newest acceptable version on an upgrade branch, qualify it
> completely, commit the exact working state, and use non-blocking
> canaries to discover the next change.

## Pin at the correct layer

| Layer | Mechanism | Example |
|---|---|---|
| Crate compatible range | `Cargo.toml` semver range, never `=x.y.z` | `reqwest = "0.13"`, `duckdb = "1"`, `axum = "0.8"` |
| Crate exact graph | `Cargo.lock`, enforced everywhere with `--locked` | every `cargo` invocation in `ci.yml`, `matrix.yml`, `coverage.yml`, `release.yml` |
| Compiler | `rust-toolchain.toml` exact channel | `channel = "1.98.0"` |
| GitHub Actions | full 40-char SHA + a `# vX.Y.Z` comment | `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1`, `Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2.9.2` |
| `install-action` helper tools | exact version per `tool:` input | `cargo-llvm-cov@0.9.0` (`coverage.yml`), `cargo-deny@0.20.2` and `cargo-cyclonedx@0.5.9` (`release.yml`) |
| `cargo-dist` | exact version, `[workspace.metadata.dist]` | `cargo-dist-version = "0.32.0"` (`Cargo.toml`) |
| CI/release runners | explicit OS generation, never the floating alias | `ubuntu-24.04`, `macos-26` — required in every job in `ci.yml`/`matrix.yml`/`coverage.yml`/`release.yml` |
| Release artifacts | checksummed + attested | per-target `.sha256` sidecars, `actions/attest` provenance + SBOM, `gh attestation verify` (README's manual-verification section) |

The floating `stable`/`ubuntu-latest`/`macos-latest` aliases appear in
exactly one place on purpose: `canary.yml` (weekly schedule +
`workflow_dispatch`). That is canary's whole job — notice the alias moving
before required CI does. Required CI (`ci.yml`, `matrix.yml`) stays on the
pinned toolchain and pinned runner generations regardless of canary state;
a red canary triggers a deliberate upgrade PR, never a silent auto-bump. No
`continue-on-error` on the canary job — a green run means nothing changed
upstream; a red one is the signal, and GitHub's default scheduled-failure
notification plus the workflow's own `if: failure()` "upstream drift"
tracking-issue step both depend on the run actually going red when
something moved.

## Deliberate non-pins

These are gaps only if you don't already know why. They're not gaps.

**Alpine minor-line, not a manifest digest.** `PROD_PROBE_IMAGE` in
`src/backend/docker.rs` and the matching test fixtures pin `alpine:3.24` —
the current minor cycle (patch 3.24.1) — not a `sha256:` digest. This repo
is designed for Linux/macOS/WSL and is never tuned to a specific self-hosted
image generation ("Cerberus"); runner and base-image pinning stops at the
OS/distro generation, not at a byte-exact manifest. A digest pin would swap
"drifts with the next Alpine 3.24 patch" for "never gets a security patch
without a manual bump" — the wrong trade for a probe image, not a supply
chain artifact.

**Crate versions stay ranges, never mass-`=x.y.z`.** `Cargo.lock` is already
the exact pin every build actually uses (with `--locked` enforced
everywhere). A blanket `=` policy in `Cargo.toml` would duplicate what the
lockfile already guarantees while turning every transitive bump into a
manual `Cargo.toml` edit for no added safety.

**No machine-readable tool inventory.** The two shell installers
(`scripts/*.sh`) and the four `install-action`/`cargo-dist` tool versions
are not captured in a schema or manifest file of their own — this
document's tables *are* the inventory. A structured format for four tools
and two scripts is process for its own sake; this table is read by people,
not parsed by tooling, and that's the right size for what it covers.

## Duplicate transitives

The pattern: two versions of the same crate resolve simultaneously because
one dependency pins a different line than sergeant's own direct dependency.
Worth naming explicitly so a future "why are there two X in the lockfile"
question has an answer here instead of becoming a fresh investigation.

- **Eliminated this wave**: `reqwest` 0.12.28 (sergeant's own direct
  dependency) and 0.13.4 (transitively via `opentelemetry-otlp`) both
  resolved simultaneously before the reqwest 0.13 migration. Collapsed to a
  single `reqwest 0.13.4` by that migration (see the sprint's W4 wave) —
  `cargo tree -i reqwest@0.12.28` now returns "did not match any packages".
- **Surfaced by the same migration, understood, not a conflict**: `ring
  0.17.14` (via `ureq 3.4.0`, a **build-dependency** of `libduckdb-sys`'s
  `bundled` feature — compiled and run only inside that crate's `build.rs`,
  never linked into the `sgt` binary) coexists with `aws-lc-rs 1.16.2` (via
  reqwest 0.13.4's `rustls` feature, sergeant's actual runtime TLS
  provider — reqwest 0.13's `rustls` feature is defined as
  `__rustls-aws-lc-rs + rustls-platform-verifier + __rustls`, i.e. enabling
  it is what first pulls `aws-lc-rs` into this tree at all). These are two
  different processes with two different `rustls::crypto::CryptoProvider`
  installs; a `CryptoProvider` conflict only matters within one process, and
  grep of `src/` for `CryptoProvider`/`rustls::crypto`/`install_default`
  returns zero hits — nothing in sergeant's own runtime code races to
  install a provider either. Not a false alarm to re-discover; recorded here
  once.

## MSRV

`Cargo.toml`'s `rust-version = "1.89.0"` is a **from-source-builder-only**
floor — the `sgt` binary itself always ships prebuilt via `cargo-dist`, so
this field serves nobody who installs a release. It exists for anyone
building from a clone. The number is measured empirically, not predicted:
`ci.yml`'s `msrv` job comment records that a static `cargo-metadata` scan of
dependency `rust-version` fields under-predicted the floor by one minor
version, because this crate's own use of
`std::fs::File::try_lock`/`TryLockError` (stabilized 1.89.0) isn't visible
to that kind of scan. `cargo +1.89.0 check --locked --all-targets` and
`cargo +1.89.0 test --locked` were run directly to confirm the real floor
before this field and the `msrv` CI job were added. That job runs
`cargo check` only — no `clippy` — because lint policy belongs to the pinned
dev compiler (`rust-toolchain.toml`), not to a deliberately-older
from-source floor. Keep `Cargo.toml:8`'s `rust-version` and `ci.yml`'s
`env.MSRV` hand-synced (no shared source of truth exists between TOML and
YAML without added machinery); both carry a comment pointing at the other.

## Canary

`canary.yml`: weekly schedule + `workflow_dispatch`, deliberately floating
`stable` Rust on `ubuntu-latest`/`macos-latest` (the one place in this repo
those aliases are intentional — see "Pin at the correct layer" above), runs
`check`/`clippy`/`test --locked`. No `continue-on-error`: it is a standalone
scheduled workflow, not a required PR check, so a red run blocks nothing
and GitHub's own scheduled-failure notification fires on it. An `if:
failure()` step opens-or-updates a single tracking issue with the failing
job link, so drift doesn't need someone to notice a red badge by chance. A
red canary is the trigger for a deliberate, reviewed upgrade PR — never a
silent auto-bump, and required CI stays on the pinned toolchain/runners
throughout, unaffected by canary state.

## Internal `/vN` identifiers are contracts, not update targets

Four schema identifiers, each a compatibility contract:

| Schema | Constant | Location | Current value |
|---|---|---|---|
| Watch | `WATCH_SCHEMA` | `src/watch.rs:34` | `sergeant.watch/v1` |
| Event | `EVENT_SCHEMA` | `src/domain/event.rs:15` | `sergeant.event/v1` |
| Execution | `SCHEMA_VERSION` | `src/backend/docker.rs:130` | `execution/v1` |
| Runtime descriptor | `DESCRIPTOR_SCHEMA` | `src/daemon.rs:58` | `sergeant.runtime/v2` |

Bumping any of these is a breaking-change event requiring an explicit
compatibility decision — never a routine "current version" edit the way a
`Cargo.toml` range or a README release literal is. The one identifier that
*has* been bumped, the runtime descriptor (v1 → v2), models the expected
shape of that decision: no compatibility shim, an old descriptor fails
closed with a diagnostic naming both schema versions and the concrete
remedy (`sgt daemon stop` then restart) — see `src/daemon.rs`'s
`read_descriptor` and its `a_v1_descriptor_fails_closed_with_a_restart_remedy`
test. A future bump to any of the other three should read as a deliberate
repeat of that pattern, not a fresh design.

# Contributing

## Build

Requires Rust (edition 2024) and `git`.

```sh
cargo build
```

First build is slow — bundled DuckDB compiles from source. Budget a few minutes for a cold `cargo build --tests` in a fresh worktree: measured at ~2m18s solo on modern desktop-class hardware, and up to ~4 minutes when a concurrent cold build contends for the same cores; an incremental rebuild after that is seconds, not minutes.

## Test

```sh
cargo fmt --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked
```

All three must be green before a commit. `cargo test --test <name>` runs one
suite, `cargo test --test <name> <substring>` runs one test.

## Pull requests

`main` is protected: changes land through a pull request, not a direct push.
CI (`fmt`, `clippy`, `cargo test`, ShellCheck, a macOS compile check, and
dependency policy) runs on every PR and must pass.

## Testing rules

Tests live in per-milestone suites `tests/m1_event_core.rs` … `tests/m10_harness.rs`.
`cargo test --test <name>` runs one suite, `cargo test --test <name> <substring>`
runs one test by name.

**The two-environment rule.** Tests run in two known environments with
opposite constraints: a root dev container (permission-bit tricks silently
pass; `EAGAIN`≡`EWOULDBLOCK`) and GitHub's non-root 2-core runner (no
`CAP_LINUX_IMMUTABLE`; `O_DIRECT` alignment unenforced). Design fault-injection
fixtures to work under both, or probe-gate a precondition no hosted-runner
user can change with a loud `SKIPPED-ENV` skip rather than a silent one —
locally-fixable preconditions stay hard failures.

A fix without a test that fails when the fix is reverted is not done. Every
advertised backend capability flag needs a contract test against the
installed harness. Test artifacts must never be left in `std::env::temp_dir()`
— clean up in the code under test or an RAII guard, not in the happy-path
test body.

## Shipping gate

`scripts/gate.sh "<intent>"` runs the no-mistakes pipeline before a PR
(`--skip push,pr,ci`; push/PR handled manually). It requires a clean working
tree and self-heals its own pipeline daemon. While a run is active the
branch is pipeline-owned — don't commit locally until the run reaches an
outcome, then use the pipeline's own respond/sync commands to take back
custody.

## Version pinning

- Crates: a compatible semver range in `Cargo.toml` (never `=x.y.z`);
  `Cargo.lock`, enforced with `--locked` everywhere, is the exact pin.
- Compiler: an exact channel in `rust-toolchain.toml`.
- GitHub Actions: a full 40-char commit SHA plus a `# vX.Y.Z` comment.
- CI/release runners: an explicit OS generation (e.g. `ubuntu-24.04`), never
  a floating `-latest` alias — the floating aliases are deliberately confined
  to the weekly `canary.yml` job, whose whole purpose is to notice upstream
  drift before required CI does.

## Glossary

A handful of terms this repo's docs and code use precisely:

- **Estate** — exactly the directory containing `./sergeant.toml`, never an
  ancestor directory found by upward search.
- **Work Surface** — one Work's whole mutation boundary: one git worktree per
  bound repository, all sharing one assigned branch.
- **Repository Mount** — a declared repository's one estate-owned base
  checkout, at `repos/<name>`; workers never write here.
- **Gate Work** — a dispatched Work whose durable outcome is a shipping-gate
  run; it owns the branch it gates for the run's duration.
- **Integrity Disposition** — the `clean`/`dirty` axis riding beside a
  terminal Work's state, reported and never blocking a transition.

## Documentation ownership

This file is the complete normative build, test, gate, and pull-request
contract for public contributors. [Product documentation](docs/index.md)
owns released behavior and extension contracts; `AGENTS.md` owns Captain's
estate policy. Maintainer history and experimental evidence may exist
elsewhere, but no contribution may be rejected for violating an inaccessible
or undocumented rule.

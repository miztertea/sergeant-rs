# F5 spike record — tree-sitter adoption (S3 X3b)

Sprint plan `sprint-plan-2026-08-27.md`, decision **F5** (J2, ruling 6). The
three criteria run **in order**, each gate before the next; any failure stops
the spike and escalates. This file is the durable record of what was actually
run and what it printed.

Lane: worktree `/var/tmp/hats3/x3b`, branch `hats3/x3b-tslp`, base
`299c8b8b` (integration after X3a/#318). Host: 20 cores, 30 GB RAM,
`/var/tmp` on a 935 GB volume. Toolchain `rustc 1.98.0 (88d9e12ae 2026-08-18)`
/ `cargo 1.98.0 (797e8a9bc 2026-08-05)` (pinned by `rust-toolchain.toml`).
`cargo-deny 0.20.2`. Every command ran with `TMPDIR=/var/tmp/sgt-test-tmp`.

## (a) Deny gate — RUN FIRST, BEFORE ANY EXTRACTION CODE

**Result: PASS.**

### The crate set actually proposed for adoption

Eight direct crates, all crates.io-published, all `license = "MIT"`, no git
dependency anywhere:

| Crate | Version | License |
|---|---|---|
| `tree-sitter` | 0.26.13 | MIT |
| `tree-sitter-rust` | 0.24.2 | MIT |
| `tree-sitter-toml-ng` | 0.7.0 | MIT |
| `tree-sitter-md` | 0.5.3 | MIT |
| `tree-sitter-python` | 0.25.0 | MIT |
| `tree-sitter-javascript` | 0.25.0 | MIT |
| `tree-sitter-typescript` | 0.23.2 | MIT |
| `tree-sitter-bash` | 0.25.1 | MIT |

`cargo add` locked **10** new packages (the eight above plus their two shared
transitive deps):

```
     Locking 10 packages to latest Rust 1.89.0 compatible versions
      Adding streaming-iterator v0.1.9
      Adding tree-sitter v0.26.13
      Adding tree-sitter-bash v0.25.1
      Adding tree-sitter-javascript v0.25.0
      Adding tree-sitter-language v0.1.7
      Adding tree-sitter-md v0.5.3
      Adding tree-sitter-python v0.25.0
      Adding tree-sitter-rust v0.24.2
      Adding tree-sitter-toml-ng v0.7.0
      Adding tree-sitter-typescript v0.23.2
```

(`tree-sitter-toml-ng` is the maintained TOML grammar under the
`tree-sitter-grammars` org; the older `tree-sitter-toml` 0.20.0 —
`Mathspy/tree-sitter-toml` — is a dead fork against a pre-`LanguageFn` ABI.)

### Verbatim results

Baseline, clean tree at `299c8b8b`, before any Cargo.toml edit:

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check
advisories ok, bans ok, licenses ok, sources ok
```

With the eight crates added (this is the gate):

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check
advisories ok, bans ok, licenses ok, sources ok
(exit 0)
```

The subset CI actually runs on routine PRs (`deny.toml` header):

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check bans licenses sources
bans ok, licenses ok, sources ok
```

The `warning[duplicate]` list is **byte-identical** before and after — the
addition introduces no new duplicate-version warning:

```
$ diff <(grep -E '^warning\[' deny-baseline.txt) \
       <(grep -E '^warning\[' deny-with-treesitter.txt) && echo IDENTICAL-WARNINGS
IDENTICAL-WARNINGS
```

### Why not TSLP (`tree-sitter-language-pack`) — measured, not assumed

The brief's stated risk was that TSLP's ~100 grammar crates would trip
`unknown-git`. Measured in a scratch probe crate outside this worktree, with
this repo's `deny.toml` copied in verbatim: **that specific risk did not
materialise — TSLP has no git sources at all** (`grep 'source = "git' Cargo.lock`
→ no matches; 143 crates, all crates.io). It fails the gate for a different
reason, on **licenses**:

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check     # tree-sitter-language-pack 1.15.8, default features
error[rejected]: failed to satisfy license requirements
   ┌─ /home/miztertea/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/option-ext-0.2.0/Cargo.toml:21:12
   │
21 │ license = "MPL-2.0"
   │            ━━━━━━━
   │            │
   │            rejected: license is not explicitly allowed
   │
   ├ MPL-2.0 - Mozilla Public License 2.0:
   ├   - OSI approved
   ├   - FSF Free/Libre
   ├   - Copyleft
   ├ option-ext v0.2.0
     └── dirs-sys v0.5.0
         └── dirs v6.0.0
             └── tree-sitter-language-pack v1.15.8

advisories ok, bans ok, licenses FAILED, sources ok
(exit 4)
```

Recorded honestly and completely: with `default-features = false` the `dirs`
subtree disappears (136 → 135 third-party crates) and the same probe returns
`advisories ok, bans ok, licenses ok, sources ok`. So TSLP is not
categorically deniable — it is deniable *as it ships*, and the features that
cause the denial (`download`, `dynamic-loading`) are exactly the ones this
repo would refuse on posture grounds anyway: `download` fetches parsers over
the network at runtime, which is the DuckDB-extension-autoload posture F4
forbids. The lean eight-crate set is chosen on R1/R7 (six languages indexed,
not 371) and on that posture, **not** on a deny failure it did not have.

No vendoring, no fork, no allowlist edit was performed or needed. `deny.toml`
is unchanged by this wave.

## (b) Fixture corpus — hand-verified counts, exact match

See `manifest.toml` beside this file and the corpus suite
`tests/x3b_tslp_corpus.rs`. Results recorded below once run.

## (c) Footprint — measured delta vs the #299-lean baseline

Recorded below once measured.

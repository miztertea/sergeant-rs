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
subtree disappears (the probe's lockfile goes 144 → 136 entries, i.e. 143 →
135 third-party crates) and the same probe returns
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

**Result: PASS, first run, with no adjustment to the manifest.**

Seven fixtures covering the six families F5 names, checked in beside this
file, plus two malformed fixtures. `manifest.toml` states every expectation;
`tests/x3b_tslp_corpus.rs` is the gate. The extractor is
`src/runtime/atlas/syntax.rs` — a pure function over bytes, no DB handle, no
daemon state (F6's adapter-shape mandate, citable at this PR).

| Fixture | Language | Symbols | Imports |
|---|---|---:|---:|
| `rust/sample.rs` | rust | 16 | 2 |
| `toml/sample.toml` | toml | 11 | 0 |
| `markdown/sample.md` | markdown | 4 | 0 |
| `python/sample.py` | python | 8 | 4 |
| `javascript/sample.js` | javascript | 6 | 2 |
| `typescript/sample.ts` | typescript | 8 | 2 |
| `shell/sample.sh` | bash | 3 | 2 |

The counts were written into `manifest.toml` by reading each fixture against
`syntax.rs`'s `symbol_kinds`/`import_kinds` tables **before** the suite was
ever executed, and the first execution matched all seven exactly. That
ordering is the whole value of the criterion, so it is recorded rather than
left implicit: nothing here was reconciled by re-recording what the extractor
happened to say.

What "exact" means here is stronger than the criterion asked for — the suite
compares ordered symbol **names**, ordered **labels**, and ordered import
**targets**, not just the two totals, and separately asserts that the
manifest's own lists and counts agree with each other:

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo nextest run --locked --test x3b_tslp_corpus
    Starting 4 tests across 1 binary
        PASS [   0.006s] sergeant-rs::x3b_tslp_corpus the_corpus_covers_every_language_this_build_claims
        PASS [   0.006s] sergeant-rs::x3b_tslp_corpus malformed_fixtures_error_rather_than_returning_a_partial_parse
        PASS [   0.009s] sergeant-rs::x3b_tslp_corpus every_fixture_matches_its_hand_verified_counts_exactly
        PASS [   0.009s] sergeant-rs::x3b_tslp_corpus every_symbol_and_import_slices_back_out_of_the_original_bytes
     Summary [   0.010s] 4 tests run: 4 passed, 0 skipped
```

Three deliberate design points behind that result:

* **A parse error is a fail, not a skip.** `extract` returns
  `SyntaxError::Parse` for any tree containing an error or missing node and
  returns *no* partial symbol list beside it — tree-sitter's error tolerance
  is exactly the silent-partial-parse failure mode F5 forbids, so it is
  refused at the module boundary. The suite has no `SKIPPED-ENV` escape and
  none may be added: parsing bytes depends on nothing the two-environment
  rule is about.
* **The malformed fixtures exist so property one is falsifiable.** A corpus of
  only-valid files would pass against an extractor whose error detection
  never fires. `malformed/broken.rs` and `malformed/broken.py` are asserted to
  *fail*.
* **The fixtures include deliberate non-symbols** — a JS arrow function bound
  to a `const`, a TS class field, Rust `impl` blocks, a Python module-level
  assignment, a `#`-prefixed line inside a fenced Markdown code block, and
  shell commands that are not `source`/`.`. Each is listed in the manifest
  header. They are what make the counts a test of the definition rather than
  of the parser's willingness to find something.

Unit tests in `syntax.rs` cover the same contract at module level (5 tests,
all passing), including the non-UTF-8 refusal and the round-trip that a
symbol's byte span slices back out of the original bytes (A1-12 provenance).

The new suite is wired into `scripts/coverage/c2-suites.sh` in the same
commit — #231(b)'s `coverage_stage_membership` guard caught it as an orphan
on the first full run, which is the guard doing exactly its job. It sits in
C2, not C3: it reads checked-in fixture bytes and spawns nothing.

## (c) Footprint — measured delta vs the #299-lean baseline

**Result: PASS.** Both legs measured in this lane by this spike, on the same
host, with the same commands, into two fresh target directories, run **one at
a time** (a concurrent cold build inflates the other's wall time by up to
~75% per CONTRIBUTING's own note, which would have made the build-time delta
meaningless).

The plan states no numeric ceiling, and none is invented here: the criterion
is a measured, provenanced delta, and that is what follows.

### Method

```sh
# BEFORE: Cargo.toml/Cargo.lock checked out at 299c8b8b (no tree-sitter);
# src/ untouched, because no extraction code existed yet when this ran.
export TMPDIR=/var/tmp/sgt-test-tmp
export CARGO_TARGET_DIR=/var/tmp/hats3/x3b-target-base   # rm -rf'd first
time cargo build --locked --tests      # the cold measurement CONTRIBUTING names
time cargo build --locked              # produces target/debug/sgt
du -sb "$CARGO_TARGET_DIR"; stat -c %s "$CARGO_TARGET_DIR/debug/sgt"

# AFTER: identical commands, CARGO_TARGET_DIR=/var/tmp/hats3/x3b-target-after,
# with the eight crates declared AND referenced by src/runtime/atlas/syntax.rs.
```

Scripts kept verbatim at `/var/tmp/hats3/x3b-evidence/measure-{baseline,after}.sh`;
raw output at `footprint-{before,after}.txt` beside them. Host: 20 cores,
30 GB RAM, `rustc 1.98.0 (88d9e12ae 2026-08-18)`. Baseline ran 09:51Z, after
ran 10:03Z, same session, same machine, nothing else building.

### The delta

| Measure | Before (#299-lean) | After | Delta |
|---|---:|---:|---:|
| `Cargo.lock` packages | 432 | 442 | **+10** |
| cold `cargo build --locked --tests` | 136 s | 138 s | **+2 s (+1.5%)** |
| cold total (`--tests` then bin) | 152 s | 154 s | **+2 s (+1.3%)** |
| `target/` after both builds | 14,573,170,440 B (14G) | 14,772,108,500 B (14G) | **+198,938,060 B (+0.185 GiB, +1.37%)** |
| `target/debug/sgt` | 259,639,792 B (247M) | 259,758,608 B (248M) | +118,816 B (+0.05%) — **but see below** |

For provenance against the brief's remembered baseline (~238M debug `sgt`,
~12G fresh target): this lane measured its own before-numbers as 247M and 14G.
The remembered figures are in the right neighbourhood but are not what this
host produced today, which is exactly why the criterion demands the spike
produce its own.

The `target/` delta decomposes, and most of it is not the grammars: the new
`x3b_tslp_corpus` test binary is **129,049,544 B** of the +198,938,060 B on
its own, because every test binary in this repo links the whole dependency
graph (that is the cost #299 already addressed by stripping dependency DWARF,
not a new cost this wave introduced). The remaining ~70 MB is the grammar
crates' own rlibs and C objects. Wiring X3b's writer will add no further test
binary of this size unless it adds another suite.

### The binary-size number needs a correction, stated plainly

That +0.05% is **misleadingly small and must not be quoted on its own.** The
grammars are C parser tables reached only through `syntax::extract`, and
nothing on the `sgt` binary's reachable path calls it yet — the linker
therefore dropped every grammar object from the binary. Verified:

```
$ nm /var/tmp/hats3/x3b-target-after/debug/sgt | grep -c 'tree_sitter_rust\|tree_sitter_python\|tree_sitter_bash'
0
```

So the honest question — what will the binary cost once X3b's writer calls
the extractor from the daemon — was measured directly, by temporarily forcing
a reference from `main.rs`, rebuilding, and reverting it (the revert is
confirmed: the binary returned byte-for-byte to 259,758,608):

```
$ nm .../sgt | grep -c 'tree_sitter_rust\|tree_sitter_python\|tree_sitter_bash'
18
forced-reference sgt bytes: 265,256,152
```

**Linked binary-size delta vs the #299-lean baseline: +5,616,360 B
(+5.36 MiB, +2.16%)** — 265,256,152 against 259,639,792. That is the number
to carry into X5's combined-delta re-measurement (F4's compounding rule),
not the +118,816 B the unreferenced build reports today.

### Not measured, and named rather than glossed

Release-profile binary size. Both legs are dev-profile, which is what the
#299 baseline the criterion names is about (#299 is a `profile.dev` decision,
and the brief's baseline figures are debug figures). A release-profile delta
is a different number and this spike did not produce it; X5's combined
re-measurement is the natural place for it if the program wants one.

## Verdict — (d) adopt

All three criteria passed, in order, each before the next was begun. Under
F5(d) and ruling 6 this is **adoption without escalation (J2)**: the plan
delegated exactly this class of decision to the spike against exactly these
pre-agreed criteria, and the criteria were met rather than argued around.
Nothing here was partially adopted, vendored, forked, or allowlisted past.

Riding the adoption, per F5(d)/A1-27: `rust-version` 1.89.0 → 1.98.0, with
ci.yml's `env.MSRV` in the same commit — the two cannot move independently,
because cargo refuses to build a package whose `rust-version` exceeds the
compiler invoked. One consequence is flagged for review in ci.yml rather than
decided here: the MSRV floor now equals `rust-toolchain.toml`'s pin, so that
job has become a duplicate check of the pinned compiler. Retiring it, or
re-establishing a genuinely older measured floor, is not this wave's call.

## Final gate state on this lane

Run after the whole change was in place (adoption, extractor, corpus,
rust-version bump, coverage wiring):

```
$ cargo fmt --check                                     # clean
$ cargo clippy --locked --all-targets -- -D warnings    # clean
$ find scripts -name '*.sh' -print0 | xargs -0 shellcheck --severity=error   # clean
$ cargo nextest run --locked
     Summary [ 330.995s] 2025 tests run: 2025 passed (1 slow), 38 skipped

# exactly the dependency-policy job's own flags (ci.yml)
$ cargo metadata --locked --format-version 1 >/dev/null                      # ok
$ cargo deny --all-features --locked check bans licenses sources
bans ok, licenses ok, sources ok
$ cargo deny --all-features --locked check          # release-time Gate E shape
advisories ok, bans ok, licenses ok, sources ok
```

## What this spike did NOT do

The spike is F5(a)-(d) and stops there. X3b's remaining wiring —
`source.symbols` / `source.occurrences` / `source.edges` DDL and their writer
in `runtime/atlas/db.rs` (empty-table doctrine: each table lands with its
writer), the F7-keyed cache glue joining X3a's batched blob reads, extraction
under the F6 intelligence-lane permit, and the `unsupported` coverage rows
(F8) — is not in this record and not in these commits. `syntax.rs` is
deliberately shaped to be called by that glue rather than to contain it.

# F5 spike record — Model2Vec adoption (S5 W3)

Sprint plan `sprint-plan-2026-08-28.md`, decision **H3** ("Model2Vec is a
SPIKE with F5's gate order"). Wave brief `brief-w3-model2vec.md`. The gates
run **strictly in order**, each fully before the next; any failure stops the
spike and escalates — *"A new advisory is a STOP that escalates — it does not
get worked around"* (brief, gate 1), and the brief adds *"The scoped
RUSTSEC-2026-0192 exception must not be broadened to cover anything new."*
This file is the durable record of what was actually run and what it printed.

Lane: worktree `/var/tmp/hats5/w3`, base `356a6501` (integration after W2 and
W7/#344). Host: 20 cores, 30 GiB RAM, `/var/tmp` on a quota-clear volume.
Toolchain `rustc 1.98.0 (88d9e12ae 2026-08-18)` / `cargo 1.98.0 (797e8a9bc
2026-08-05)` (pinned by `rust-toolchain.toml`). `cargo-deny 0.20.2`. Every
command ran with `TMPDIR=/var/tmp/sgt-test-tmp`. Run at 2026-08-30T02:33Z.

## Verdict, up front

**GATE 1 (deny) FAILS. `adopted: false`. Gates 2 (fixture corpus) and 3
(footprint) were NOT run** — the gate order forbids starting them until gate
1 passes, and it did not. The tree is reverted to `356a6501`'s `Cargo.toml`
and `Cargo.lock` in the same commit as this record. Escalation menu at the
bottom; this is **J0** — a new advisory changes the graph's maintenance/
security posture, which is not a rung this seat may resolve.

## What the recon claimed, re-verified today rather than trusted

The brief says *"verify, do not re-derive"*. Verified against the crate's own
registry-normalized manifest and source, not against the recon's snapshot:

| Claim | Result | Source |
|---|---|---|
| Crate exists, is the official one | `model2vec-rs 0.2.1` — "Official Rust Implementation of Model2Vec", repo `github.com/MinishLab/model2vec-rs` | `cargo info model2vec-rs` |
| License MIT | `LICENSE` file is verbatim MIT ("MIT License / Copyright (c) 2025 The Minish Lab"); the manifest declares `license-file`, **not** an SPDX `license` expression, so `cargo deny` emits `warning[no-license-field]` and then clears it by scanning the file — `licenses ok` | `~/.cargo/registry/src/.../model2vec-rs-0.2.1/LICENSE`; `deny-candidate-b-full.txt` |
| Runtime need is three files | Confirmed in code: `match_local_layout` requires `config.json` + `tokenizer.json` + `model.safetensors` to all `exists()` | `src/model.rs:35-44` of the crate |
| No GPU, no server | Confirmed: `pool_ids` is a mean-pool over `ndarray` rows in-process; no ONNX, no CUDA, no HTTP in the non-`hf-hub` build | `src/model.rs`, `Cargo.toml` `[dependencies]` |
| A local-only build is expressible | Confirmed and load-bearing — see below | crate `Cargo.toml` `[features]` |

### A2-12 is structurally satisfiable by this crate (the good news)

A2 §15 forbids a stage-time download. The crate makes that *structural*, not
a flag one has to remember:

* `default = ["onig", "hf-hub"]`, and `hf-hub = ["dep:hf-hub", "dep:ureq"]`
  — an HTTP client plus a HuggingFace Hub downloader. `default-features =
  false` drops both.
* `local-only = []` is the crate's own kill switch. Every download item is
  `#[cfg(all(feature = "hf-hub", not(feature = "local-only")))]`
  (`src/model.rs:3`, `:9`, `download_model_files`), and `resolve_model_files`
  compiles to `Err("remote model downloads are disabled by the `local-only`
  feature; pass a local model directory instead")` under it.

So the candidate tree used `default-features = false, features =
["local-only"]`. **This part of the spike succeeded** — it is not what
failed, and it is what a re-attempt should keep.

## Gate 1 — deny gate, run first, before any code

Command, exactly as the brief and the S4 precedent (`tests/fixtures/
anydoc_corpus/SPIKE-G3.md` §(a)) ran it — the unscoped full check, not the
routine-PR `bans licenses sources` subset:

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check
```

### Baseline — `Cargo.toml`/`Cargo.lock` at `356a6501`

```
advisories ok, bans ok, licenses ok, sources ok
(exit 0)
```

`grep -cE '^error\[' deny-baseline-full.txt` → **0**. Full output:
`deny-baseline-full.txt`, beside this file. Note this is a *stronger*
baseline than S4's anydoc spike had: the pre-existing yanked-`chacha20`
failure that spike had to exclude is gone from this graph, so every error
below is attributable to this spike with no subtraction needed.

### Candidate A — `model2vec-rs` alone

```toml
model2vec-rs = { version = "0.2.1", default-features = false, features = ["local-only", "fancy-regex"] }
```

`fancy-regex` because `tokenizers` refuses to compile with no regex backend
(`compile_error!("One of the `onig`, or `fancy-regex` features must be
enabled")`, `tokenizers-0.21.4/src/utils/mod.rs:15`), verified by building
the feature-less candidate first and reading the error.

```
error[unmaintained]: number_prefix crate is unmaintained
error[unmaintained]: paste - no longer maintained
advisories FAILED, bans ok, licenses ok, sources ok
(exit 1)
```

**Two** new advisories. Full output: `deny-candidate-a-full.txt`.

### Candidate B — the same, with the regex backend selected directly

`number_prefix` (RUSTSEC-2025-0119) is *avoidable*, and finding that out is
what gate 1 is for. It arrives via `indicatif`, which arrives via
`tokenizers/progressbar`, which `model2vec-rs`'s own `fancy-regex` feature
bundles in with the regex engine (`fancy-regex = ["tokenizers/fancy-regex",
"tokenizers/progressbar", "tokenizers/esaxx_fast"]`). Declaring `tokenizers`
directly with `fancy-regex` alone unifies with `model2vec-rs`'s own
`default-features = false` edge on the same version and selects the backend
without the progress bar:

```toml
model2vec-rs = { version = "0.2.1", default-features = false, features = ["local-only"] }
tokenizers   = { version = "0.21",  default-features = false, features = ["fancy-regex"] }
```

This is avoidance by configuration, **not** suppression: nothing was added to
`deny.toml`, and `indicatif`/`number_prefix` leave `Cargo.lock` entirely.

```
error[unmaintained]: paste - no longer maintained
    ┌─ /var/tmp/hats5/w3/Cargo.lock:238:1
    │
238 │ paste 1.0.15 registry+https://github.com/rust-lang/crates.io-index
    │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ unmaintained advisory detected
    │
    ├ ID: RUSTSEC-2024-0436
    ├ Advisory: https://rustsec.org/advisories/RUSTSEC-2024-0436
    ├ The creator of the crate `paste` has stated in the README.md
      that this project is not longer maintained as well as archived the repository
      ## Possible Alternative(s)
      - `pastey`: a fork of paste and is aimed to be a drop-in replacement ...
      - `with_builtin_macros`: ...
    ├ Announcement: https://github.com/dtolnay/paste
    ├ Solution: No safe upgrade is available!
    ├ paste v1.0.15
      └── tokenizers v0.21.4
          ├── model2vec-rs v0.2.1
          │   └── sergeant-rs v0.3.0
          └── sergeant-rs v0.3.0 (*)

advisories FAILED, bans ok, licenses ok, sources ok
(exit 1)
```

Full output: `deny-candidate-b-full.txt`. Exactly **one** new `error[...]`
line against a baseline with none:

```
$ diff <(grep -E '^error\[' deny-baseline-full.txt) \
       <(grep -E '^error\[' deny-candidate-b-full.txt)
0a1
> error[unmaintained]: paste - no longer maintained
```

### The crate set candidate B actually locks

**28 packages added, 0 removed, 0 version-bumped** (computed by diffing every
`(name, version)` pair between `git show 356a6501:Cargo.lock` and the
candidate lockfile — not by counting diff `+` lines):

```
base64 0.13.1              derive_builder_macro 0.20.2   ndarray 0.15.6
bit-set 0.8.0              esaxx-rs 0.1.10               paste 1.0.15
bit-vec 0.8.0              fancy-regex 0.14.0            pastey 0.2.3
darling 0.20.11            macro_rules_attribute 0.2.3   rawpointer 0.2.1
darling_core 0.20.11       macro_rules_attribute-        rayon-cond 0.4.0
darling_macro 0.20.11        proc_macro 0.2.3            safetensors 0.5.3
dary_heap 0.3.9            matrixmultiply 0.3.11         spm_precompiled 0.1.4
derive_builder 0.20.2      model2vec-rs 0.2.1            tokenizers 0.21.4
derive_builder_core 0.20.2 monostate 0.1.18              unicode-normalization-
                           monostate-impl 0.1.18           alignments 0.1.12
                                                         unicode_categories 0.1.1
```

`base64 0.13.1` is a third `base64` in the graph and `windows-sys` gains a
third entry — `warning[duplicate]` only, not errors
(`bans.multiple-versions = "warn"`).

### Why `paste` is not avoidable the way `number_prefix` was

Checked, not assumed:

1. **It is not optional.** `tokenizers-0.21.4`'s registry-normalized manifest
   has `[dependencies.paste] version = "1.0.14"` with no `optional = true`
   and no feature guarding it (direct read of the vendored `Cargo.toml`,
   line 194). No feature combination drops it.
2. **No safe upgrade exists.** The advisory says so verbatim: *"Solution: No
   safe upgrade is available!"* — `paste` is archived, not vulnerable-then-
   patched.
3. **A newer `tokenizers` does not help, and `model2vec-rs` could not take
   one anyway.** `model2vec-rs 0.2.1` (the latest release) requires
   `tokenizers = "0.21"`, which under Cargo's 0.x rules admits only `0.21.x`.
   And `tokenizers 0.23.1` *still* depends on `paste` — verified empirically
   by resolving a throwaway crate against
   `tokenizers 0.23 --no-default-features -F fancy-regex` and grepping its
   lockfile (`grep -c 'name = "paste"'` → 1). So "wait for a bump" does not
   obviously fix it either.
4. **The alternative Model2Vec crate has the same problem.** `model2vec
   0.3.0` ("H2CO3's & Narnium's Rust Implementation") also depends on
   `tokenizers = "0.21"` (direct read of its vendored manifest, line 66).
   Any Rust implementation that reads a HuggingFace `tokenizer.json` goes
   through `tokenizers`, and `tokenizers` goes through `paste`.

So this is not a candidate-selection problem inside the spike's authority to
solve by swapping crates. It is a property of the whole Rust Model2Vec lane.

### Verdict on gate 1

**FAIL.** One new advisory, RUSTSEC-2024-0436, attributable to this spike,
unavoidable by feature selection and unavoidable by candidate substitution.
Per the brief: STOP, do not work around it, do not broaden the scoped
RUSTSEC-2026-0192 entry (it matches by advisory ID and would not have
suppressed this one anyway — that narrowness is the point, and
`tests/fixtures/anydoc_corpus/prove-exception-is-scoped.sh` is what proves
it). Gates 2 and 3 not run.

## Cleanup performed

`Cargo.toml`'s `model2vec-rs` and `tokenizers` lines (and their comments) and
the regenerated `Cargo.lock` are reverted to the `356a6501` baseline in the
same commit as this record, so `cargo deny check` returns to `advisories ok,
bans ok, licenses ok, sources ok`.

## Escalation menu (recorded per PACE Alternate — NOT decided here)

**J0.** Admitting a new advisory changes the graph's maintenance posture; no
lower rung reaches it. Recommendation offered because one can be responsibly
made, but the decision is the owner's.

1. **The risk shape is materially weaker than RUSTSEC-2026-0192's, and that
   is the honest framing.** `paste` is a **proc-macro** — it expands
   identifier concatenations at compile time and contributes **no runtime
   code and no runtime attack surface**. It never sees untrusted input;
   nothing an operator's documents or a repo's bytes can contain reaches it.
   Contrast `ttf-parser`, the advisory the owner already accepted: that one
   parses attacker-supplyable font tables *at runtime*. The crate is
   archived-by-choice by a widely trusted author (dtolnay), not abandoned
   mid-vulnerability. **Recommendation: adopt, with a scoped, dated
   `deny.toml` entry for RUSTSEC-2024-0436 alone**, in exactly the shape and
   with the same narrowness proof as the RUSTSEC-2026-0192 entry — and with
   the revisit trigger "`tokenizers` drops `paste` (upstream has already
   pulled in `pastey`, its drop-in fork, via `macro_rules_attribute`, so the
   swap is plausibly close)".
2. **Decline the advisory and drop the semantic half from S5.** A2-13 (R1)
   says *"Keep semantic retrieval optional and degrade to lexical/
   structural"*, and A2 §16 makes *"making semantic retrieval required for
   core Work execution"* a non-goal. The product is contract-complete without
   it: A2 §17's acceptance item 4 is conditional (*"semantic retrieval,
   **when installed**"*), and this wave's H4 work (landed — see below) makes
   the absence a reported state rather than a silent gap. W4's RRF would
   fuse a one-source list, which is a degenerate but well-defined case.
3. **Hand-roll the tokenizer** to avoid `tokenizers` entirely. Named for
   completeness and **not recommended**: a `tokenizer.json` carries a
   normalizer, a pre-tokenizer, a (unigram/BPE/wordpiece) model and
   post-processors; reimplementing that is the R7-with-a-bug-farm branch, and
   getting it subtly wrong produces embeddings that are *plausible* and
   wrong, which is the worst failure mode a retrieval system has.
4. **Wait and re-run gate 1.** `tokenizers` already depends on `pastey` (via
   `macro_rules_attribute`), so `paste`'s removal upstream is a live
   possibility; a later `model2vec-rs` release taking a `paste`-free
   `tokenizers` would flip this gate green with no ruling needed.

## What this wave landed anyway, and why that is not partial adoption

**H4 only** — decision H4 in `sprint-plan-2026-08-28.md`, which the brief
states is *DECIDED* and instructs this wave to *implement*: `semantic:
applied | not_installed | disabled` as a REQUIRED, non-omittable field on a
search answer, and a test proving a no-model run still answers through the
lexical half and reports `not_installed`.

That is **J4** (an explicit decided instruction) reinforced by **J5** (A2
§15's *"reports that coverage/capability honestly"* and A2 §17 item 4's
*"can be disabled/degraded cleanly"* bind whether or not any model is ever
adopted). It adds no dependency, asserts no embedding capability, and is
correct under every branch of the escalation menu above — including branch 2,
where it becomes the *entire* honest answer to "why is there no semantic
half". See `src/runtime/atlas/semantic.rs` and
`tests/w3_semantic_degradation.rs`.

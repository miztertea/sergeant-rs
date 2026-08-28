# G3 spike record — Anydoc/Office adoption (S4 Y2)

Sprint plan `sprint-plan-2026-08-27.md`, decision **G3** (J2, ruling 6; F5
gate order verbatim). Wave brief `brief-y2-office-adoption.md`. The gates run
**strictly in order**, each fully before the next; any failure stops the
spike and escalates — no vendoring, forking, or allowlisting around it. This
file is the durable record of what was actually run and what it printed.

Lane: worktree `/var/tmp/hats4/y2`, branch `hats4/y2-office`, base
`c00209b7` (integration after Y1/#326 merged — the worker-transport wave this
spike's adapter would ride). Host: 20 cores, 30 GiB RAM, `/var/tmp` on a
quota-clear volume. Toolchain `rustc 1.98.0 (88d9e12ae 2026-08-18)` /
`cargo 1.98.0 (797e8a9bc 2026-08-05)` (pinned by `rust-toolchain.toml`).
`cargo-deny 0.20.2`. Every command ran with `TMPDIR=/var/tmp/sgt-test-tmp`.
Run at 2026-08-27T23:27Z.

## (a) Deny gate — RUN FIRST, BEFORE ANY EXTRACTION CODE

**Result: FAIL.**

### The candidate crate

`anydoc` — the recon's named leading candidate for Office/document
normalization (recon-anydoc-office.md §2.1), re-verified today rather than
trusted from the recon's yesterday-dated snapshot:

| Check | Result | Source |
|---|---|---|
| Version on crates.io today | 0.2.4 (recon saw 0.2.3 one day ago; 16th release) | `cargo info anydoc` |
| License | MIT | `cargo info anydoc`; confirmed by `cargo deny check licenses` passing |
| `rust-version` | 1.88 | `cargo info anydoc` — inside this crate's pinned 1.98.0 |
| Rust API shape | `anydoc::to_document(&bytes, Option<Format>) -> Result<Document, ConvertError>`, `anydoc::to_markdown_bytes`, `Format::from_bytes/from_extension/from_path` — bytes in, structured model or GFM out | ctx7 `/firecrawl/anydoc`, `README.md` "Use anydoc in Rust" |
| Document model | `Document { blocks, notes, assets }`; `Block` carries `kind`/`level`/`content`/`table`/`list`; `Table.grid: Array<Array<CellSlot>>`; no explicit slide/sheet/heading-path coordinate field surfaced by ctx7's snippet corpus | ctx7 `/firecrawl/anydoc`, `node/index.d.ts` type defs (mirrors the Rust model 1:1 per the crate's own docs) — **could not fully verify against the Rust struct definitions directly** (ctx7's Rust-side snippets did not include the model's field-level source); the TS bindings are stated by the project to mirror the Rust API, but that mirroring itself is asserted, not independently checked here |
| Own feature flags | None — `anydoc-0.2.4`'s registry-normalized `Cargo.toml` has no `[features]` table; `pdf-inspector` (and therefore `lopdf`/`ttf-parser`, see below) is an unconditional dependency, not something a Cargo feature can drop | direct read of `~/.cargo/registry/.../anydoc-0.2.4/Cargo.toml` |

Added to `Cargo.toml`'s `[dependencies]` as `anydoc = "0.2.4"`, with a
comment recording this was provisional pending gates (b)/(c) — before any
extraction code was written, matching the gate-order requirement.

### The crate set the addition actually locks

`cargo metadata` (no `--locked`, since the whole point was to let the
resolver add the new subtree) added **42 new packages** to `Cargo.lock`, zero
existing entries removed or version-bumped (computed by diffing every
`(name, version)` pair between the pre- and post-change lockfiles, not by
counting diff `+` lines, which double as headers for unrelated fields too):

```
aes 0.8.4                cfb 0.14.0                jiff-tzdb 0.1.8          rayon-core 1.13.0
anydoc 0.2.4              cipher 0.4.4              jiff-tzdb-platform 0.1.3 stringprep 0.1.5
block-padding 0.3.3       crossbeam-deque 0.8.7     lopdf 0.42.0             time-macros 0.2.32
cbc 0.1.2                 crossbeam-epoch 0.9.20    md-5 0.10.6              ttf-parser 0.25.1
csv 1.4.0                 crossbeam-utils 0.8.22    nom 8.0.0                typed-path 0.12.3
csv-core 0.1.13           encoding_rs 0.8.35        pdf-inspector 1.17.0     unicode-bidi 0.3.18
defmt 1.1.1               env_filter 2.0.0          portable-atomic-util 0.2.7 unicode-normalization 0.1.25
defmt-macros 1.1.1        env_logger 0.11.11        quick-xml 0.41.0         unicode-properties 0.1.4
defmt-parser 1.0.0        inout 0.1.4               rangemap 1.8.0           weezl 0.1.12
ecb 0.1.2                 jiff 0.2.35               rayon 1.12.0             zip 8.6.0
jiff-core 0.1.0
```

Matches the recon's predicted stack (`cfb`, `quick-xml`, `zip` 8.x
deflate-only, `pdf-inspector`, `csv`, `encoding_rs`) plus `pdf-inspector`'s
own transitive PDF-parsing subtree (`lopdf`, `ttf-parser`, `nom`, the
`cbc`/`aes`/`cipher` PDF-encryption family) that the recon did not enumerate
this deep. `zip` 8.6.0 duplicates the graph's pre-existing `zip` 6.0.0 (pulled
by `libduckdb-sys`'s build script) — a `warning[duplicate]`, not an error
(`bans.multiple-versions = "warn"` in `deny.toml`); noted for the archive
lane (Y3), not actioned here.

### Verbatim results

**Baseline**, `Cargo.toml`/`Cargo.lock` at `c00209b7` (before this spike
touched either file):

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check
[... 1198 lines of duplicate-version warnings and dependency trees ...]
error[yanked]: detected yanked crate (try `cargo update -p chacha20`)
   ┌─ Cargo.lock:44:1
   │
44 │ chacha20 0.10.1 registry+https://github.com/rust-lang/crates.io-index
   │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ yanked version
   │
   ├ chacha20 v0.10.1
     └── rand v0.10.2
         └── ulid v3.0.0
             └── sergeant-rs v0.3.0

advisories FAILED, bans ok, licenses ok, sources ok
(exit 1)
```

**This is load-bearing and stated plainly: the baseline this spike started
from already fails the full `cargo deny check` on `main`'s own pre-existing
graph**, via `ulid` → `rand` 0.10.2 → a yanked `chacha20` 0.10.1. This has
nothing to do with Office documents, anydoc, or this wave — `ulid` predates
S4 entirely. It is not this spike's failure to fix or explain away, and it is
recorded here only so the diff below is honest about what changed and what
did not. (Full output: `deny-baseline-full.txt`, beside this file.)

**With `anydoc = "0.2.4"` added** (this is the gate):

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check
[... 1338 lines of duplicate-version warnings and dependency trees ...]
error[unmaintained]: `ttf-parser` is unmaintained
    ┌─ /var/tmp/hats4/y2/Cargo.lock:341:1
    │
341 │ ttf-parser 0.25.1 registry+https://github.com/rust-lang/crates.io-index
    │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ unmaintained advisory detected
    │
    ├ ID: RUSTSEC-2026-0192
    ├ Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0192
    ├ The author of `ttf-parser` has stated that the crate is unmaintained and
    │ will not receive further fixes (see the referenced issue).
      ## Alternative(s)
      - `skrifa`, an actively maintained TrueType/OpenType font parsing
        crate, part of the Google Fonts "oxidize" (`fontations`) project.
    ├ Announcement: https://github.com/harfbuzz/ttf-parser/issues/217
    ├ Solution: No safe upgrade is available!
    ├ ttf-parser v0.25.1
      ├── lopdf v0.42.0
      │   └── pdf-inspector v1.17.0
      │       └── anydoc v0.2.4
      │           └── sergeant-rs v0.3.0
      └── pdf-inspector v1.17.0 (*)

error[yanked]: detected yanked crate (try `cargo update -p chacha20`)
   ┌─ Cargo.lock:50:1
   │  [... identical chacha20/rand/ulid block as the baseline above ...]

advisories FAILED, bans ok, licenses ok, sources ok
(exit 1)
```

(Full output: `deny-with-anydoc-full.txt`, beside this file.)

The routine-PR subset CI actually gates on (CONTRIBUTING.md: "Routine PRs run
`cargo deny check bans licenses sources`"), for completeness — this subset
alone stays clean both before and after:

```
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check bans licenses sources
bans ok, licenses ok, sources ok
(exit 0)
```

### What the addition actually changed — diffed, not asserted

```
$ diff <(grep -E '^error\[' deny-baseline-full.txt) \
       <(grep -E '^error\[' deny-with-anydoc-full.txt)
0a1
> error[unmaintained]: `ttf-parser` is unmaintained
```

Exactly **one** new `error[...]` line. `anydoc`'s dependency on
`pdf-inspector` (for the text-PDF leg of A1 §6.3's format list) pulls
`lopdf`, which pulls `ttf-parser` 0.25.1 — a crate RustSec advisory
RUSTSEC-2026-0192 marks unmaintained with **no safe upgrade available**. The
pre-existing `chacha20` yanked-crate error is byte-identical before and
after (same dependency path, same version, untouched by this change) and is
excluded from the count above on that basis.

### Verdict on (a)

The wave brief names **advisory** explicitly as one of the four failure
classes that stops the spike ("A failure (unknown-git source, licence, ban,
advisory) means STOP and escalate with the verbatim output"), and the F5
precedent this wave is told to replicate ran the same unscoped `cargo deny
check` — not the routine-PR bans/licenses/sources subset — as its own gate
(`tests/fixtures/tslp_corpus/SPIKE-F5.md` §(a)). Under that same command,
adding `anydoc` introduces a genuinely new, attributable advisory failure:
`ttf-parser` is unmaintained with no upgrade path, reached through
`anydoc`'s own PDF-support dependency (`pdf-inspector` → `lopdf`), which is
unconditional — `anydoc` ships no feature flag to exclude it. This is not
noise from an unrelated, unchanged part of the graph (the `chacha20` case,
correctly excluded above); it is a maintenance-risk signal directly about
the crate under evaluation, which is exactly what this gate exists to catch
before extraction code is written on top of it.

**Result: DENY GATE FAILS. `adopted: false`. STOP here** — per the brief, no
vendoring, forking, or allowlisting `ttf-parser`'s advisory to get past it,
and no swap to a different candidate crate within this same gate run (that
menu — narrow the format claim, compose per-format with `calamine`/
`docx-rs`, pin-with-revisit-trigger, or feature-gate — is the recon §4
escalation the owner adjudicates, not this spike's call to make
unilaterally). Gates (b) fixture corpus and (c) footprint delta were **not
run**: the brief requires the deny gate to pass fully before either begins,
and it did not.

## Cleanup performed

`Cargo.toml`'s `anydoc = "0.2.4"` line (and its comment) and the regenerated
`Cargo.lock` are reverted to the `c00209b7` baseline in the same commit as
this record, so the working tree returns to a state where `cargo deny check
bans licenses sources` — the routine gate — is unaffected by this spike
either way, and the full `cargo deny check`'s only failure is the
pre-existing, unrelated `chacha20` one this spike did not introduce and does
not own.

## Escalation menu (recorded per PACE Alternate — not decided here)

Per the recon's own escalation menu (recon-anydoc-office.md §4) and CLAUDE.md's
PACE-Alternate-in-autonomous-runs ruling, this is J0 for the owner, recorded
on the head PR as a blocking open question while other Y-wave work continues
around it:

1. **Vendor-risk framing fits best**: this is not a licence/ban rejection of
   `anydoc` itself, and not a security vulnerability — it is an unmaintained
   *transitive* crate (`ttf-parser`, three hops down, feeding PDF font
   metadata that A1 §6.3's Office-format list does not lead with: Word,
   PowerPoint, Excel come first, text-PDF is explicitly the lowest-priority
   named format and adjacent to the already-deferred OCR lane). A revisit
   trigger (an `anydoc` release that swaps `pdf-inspector`/`lopdf` for
   something on `skrifa`, or a `pdf-inspector` release that drops the PDF
   font-metadata path this build never needs) is the natural register-row
   shape if the owner chooses to hold rather than narrow.
2. **Narrow the format claim**: §17 item 5 needs only *one* Office format to
   close acceptance. If `pdf-inspector`'s PDF leg is the only path pulling
   `ttf-parser`, and Word/PowerPoint/Excel/ODF do not route through it
   (unverified here — this spike stopped at the deny gate before writing any
   code that would answer that empirically), a docx-only or docx+pptx+xlsx
   claim with PDF left `unsupported` might sidestep the failing dependency
   entirely — but `anydoc` has no feature flag to structurally drop
   `pdf-inspector` from the build even if the *code path* is never called at
   runtime, so `ttf-parser` stays in `Cargo.lock` and in `cargo deny check`'s
   view regardless of which formats this build actually exercises. This
   menu item would need re-verifying against the deny gate, not assumed.
3. **Compose per-format** (calamine + docx-rs/docx-parser): the recon's
   fallback landscape, explicitly "the *more machinery* branch A1-11's R7
   rejected" — a fresh decision-register row, not a quiet swap.
4. **Wait and re-run the gate**: `ttf-parser`'s advisory states no safe
   upgrade exists today; a future `anydoc`/`pdf-inspector` release changing
   that dependency would change gate (a)'s result on a later attempt.

What this failure does **not** do, per the brief: hand-implement OOXML/OLE
parsers, or add an exception to `deny.toml` to admit an unmaintained crate.

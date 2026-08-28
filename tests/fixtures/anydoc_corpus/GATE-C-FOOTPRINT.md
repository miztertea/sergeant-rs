# G3 gate (c) — build-time / binary-size footprint delta

Sibling record: `SPIKE-G3.md` (gate a, the deny-gate failure/adoption
history), `MANIFEST.md`/`manifest.json` (gate b, the fixture corpus),
`G3-exception-narrowness-proof.md` (the deny.toml exception's own proof).
Method follows `knowledge/evidence/perf/tslp-footprint-delta-2026-08-27.md`
verbatim, including its linked-vs-naive binary correction.

## Method

Two solo, fresh-`CARGO_TARGET_DIR`, `TMPDIR=/var/tmp/sgt-test-tmp` legs,
same host, same day (2026-08-27), rustc/cargo 1.98.0
(`rust-toolchain.toml` pin), nothing else heavy running during either leg
(checked `ps`/`free` immediately before each start):

- **BEFORE** = this lane's base commit, `c00209b7` (no `anydoc`), built in
  a separate checkout `/var/tmp/hats4-y2-before-measure` (detached at
  `7a62239d`, `Cargo.toml`/`Cargo.lock` reverted to base — i.e. identical
  to this lane's own `HEAD` before the working-tree `anydoc` addition),
  `CARGO_TARGET_DIR=/var/tmp/sgt-footprint-before`.
- **AFTER** = this lane's working tree with `anydoc = "0.2.4"` added,
  `CARGO_TARGET_DIR=/var/tmp/sgt-footprint-after`, via
  `run_footprint_after.sh` (this file's directory — the script that
  produced the AFTER numbers below; rerunnable, mirrors the BEFORE leg's
  equivalent script).

Commands (each leg): `rm -rf $CARGO_TARGET_DIR; time cargo build --locked
--tests; time cargo build --locked; du -sb $CARGO_TARGET_DIR; stat -c %s
$CARGO_TARGET_DIR/debug/sgt`.

**Linked-vs-naive correction.** At AFTER build time, nothing in `main`'s
reachable graph calls into `anydoc` yet (this wave's task is spike gates
only — the adapter itself is separate, later work). `nm | grep -ci
"anydoc\|pdf_inspector\|lopdf"` against the naive AFTER binary returns
`0`: the linker drops the entire crate, so the naive binary-size delta
measures nothing real. Per the tslp precedent, a reference was forced
temporarily in `src/main.rs` (a single call into
`anydoc::to_document(&[], anydoc::Format::Docx)` wrapped in
`std::hint::black_box`), the binary rebuilt and measured, then the
`src/main.rs` change fully reverted and rebuilt again — confirmed
byte-identical to the naive AFTER size (`273619272` both times) and
confirmed clean by `git status --short` showing no `src/main.rs` diff.
`nm` against the forced-linkage binary shows 9,264 matching symbols.

## Numbers

| Metric | Before | After | Delta |
|---|---|---|---|
| Cargo.lock packages | 442 | 484 | +42 |
| cold `build --tests` | 160.781 s | 159.295 s | −1.49 s (−0.9%, noise) |
| cold `build` (bin only) | 16.933 s | 17.320 s | +0.39 s (+2.3%) |
| `target/` | 16,818,953,889 B | 17,030,817,257 B | +211,863,368 B (+202.0 MiB, +1.26%) |
| debug `sgt` (naive) | 273,671,456 B | 273,619,272 B | −52,184 B — MISLEADING (anydoc unreferenced, dropped by linker) |
| **debug `sgt` (linked, forced ref)** | 273,671,456 B | 281,179,968 B | **+7,508,512 B (+7.16 MiB, +2.74%)** |

The +42 package count matches `SPIKE-G3.md`'s "exactly a 42-package
delta" claim from the original gate-(a) spike, confirming no drift
between that record and this one.

## Verdict

**MEASURED AND RECORDED**, with provenance (exact commands, exact
before/after trees, exact host/toolchain, above). The wave brief names no
numeric ceiling for gate (c) — it passes by being measured, not by
clearing a threshold.

Read honestly: the real cost of adopting `anydoc` is the **linked**
binary-size delta (+7.16 MiB / +2.74%) and the `target/` delta (+202 MiB
/ +1.26%), not the naive number. +7.16 MiB is a real, human-noticeable
jump for a single dependency addition on a `sgt` binary this size — bigger
than the tslp precedent's own +5.36 MiB (tree-sitter plus 8 grammars).
This is not disqualifying on its own (the owner ruling already accepted
this dependency and its advisory as a named tradeoff, and the number is
in the low single-digit percent), but it is large enough that a human
should see the exact figure rather than have it summarized as "passed" —
flagging it here rather than glossing it.

## Reproduce

```
$ tests/fixtures/anydoc_corpus/run_footprint_after.sh
```

(BEFORE leg: same script shape, pointed at a checkout of `c00209b7` with
`CARGO_TARGET_DIR=/var/tmp/sgt-footprint-before` — not checked in, since
it builds a tree that is not this lane's working tree; the numbers above
are its recorded output.)

# F5 — deny.toml exception narrowness proof (RUSTSEC-2024-0436)

Companion to `SPIKE-F5.md` (the deny-gate failure record W3 produced) and
`prove-exception-is-scoped.sh` (beside this file — rerun it to reproduce).
Owner ruling: `knowledge/rulings/owner-rulings/
model2vec-paste-advisory-2026-08-30.md` (J4), decision 2: *"never a
broadened rule, never a disabled gate … **The next advisory must still fail
the gate.**"*

## What this proves

`deny.toml`'s `[advisories.ignore]` list now names two RUSTSEC ids —
`RUSTSEC-2026-0192` (anydoc/`ttf-parser`, S4 Y2) and `RUSTSEC-2024-0436`
(model2vec/`paste`, this wave). `cargo-deny`'s `ignore` list matches by
advisory ID, not by crate or dependency subtree — so this is provable rather
than merely asserted: a **different** advisory ID reaching the graph through
the **same** `model2vec-rs -> tokenizers -> paste` edge must still fail
`cargo deny check advisories`.

## Method

1. Copy `cargo-deny`'s already-fetched real advisory-db cache (this repo's
   `deny.toml` `db-path`, `~/.cargo/advisory-db`) — the real cache is never
   written to.
2. Drop a fabricated advisory file into the copy, at
   `crates/tokenizers/RUSTSEC-2098-9998.md`. `tokenizers` is the crate
   sitting *directly between* `paste` and `model2vec-rs` on the very edge the
   real exception covers — the hardest case for a too-broad entry, not an
   arbitrary unrelated crate. `RUSTSEC-2098-9998` is a nonexistent id chosen
   to be unambiguously synthetic and not to collide with any real future
   advisory.
3. Run `cargo deny check advisories --offline` with this repo's real,
   unmodified `deny.toml` (read as-is), `db-path` repointed at the doctored
   copy.
4. Assert: the real id stays ignored, the synthetic id errors, exit code 1.

## Recorded run — 2026-08-30, lane `/var/tmp/hats5/w3b`

`cargo-deny 0.20.2`, `TMPDIR=/var/tmp/sgt-test-tmp`, advisory-db cache
`advisory-db-3157b0e258782691`. Verbatim output of the doctored run:

```text
== copying cached advisory-db (advisory-db-3157b0e258782691) — the real cache is never touched ==
== dropping a SYNTHETIC advisory against tokenizers (the crate between paste and model2vec-rs) ==
== running cargo deny check advisories against the repo's real deny.toml, db-path repointed at the doctored copy ==
----- cargo deny output -----
error[unmaintained]: SYNTHETIC advisory — gate-narrowness proof only, not a real vulnerability
    ┌─ /var/tmp/hats5/w3b/Cargo.lock:342:1
    │
342 │ tokenizers 0.21.4 registry+https://github.com/rust-lang/crates.io-index
    │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ unmaintained advisory detected
    │
    ├ ID: RUSTSEC-2098-9998
    ├ Advisory: https://rustsec.org/advisories/RUSTSEC-2098-9998
    ├ Fabricated by `tests/fixtures/model2vec_corpus/prove-exception-is-scoped.sh`
      to prove deny.toml's `RUSTSEC-2024-0436` ignore entry does not suppress a
      *different* advisory ID reaching the graph through the same `model2vec-rs ->
      tokenizers` subtree.
    ├ Announcement: https://example.invalid/synthetic-advisory-for-gate-proof
    ├ Solution: No safe upgrade is available!
    ├ tokenizers v0.21.4
      ├── model2vec-rs v0.2.1
      │   └── sergeant-rs v0.3.0
      └── sergeant-rs v0.3.0 (*)

advisories FAILED
------------------------------
PASS: RUSTSEC-2024-0436 stays ignored; RUSTSEC-2098-9998 (a different advisory, same model2vec subtree) still fails the gate.
```

`RUSTSEC-2024-0436` appears nowhere in that output: the entry covers it. The
synthetic id is an `error[...]`, and the run exits 1.

## The undoctored run, same day, same lane

```text
$ TMPDIR=/var/tmp/sgt-test-tmp cargo deny check
advisories ok, bans ok, licenses ok, sources ok
(exit 0)
```

So the gate is **green with the exception and red the moment a new advisory
arrives** — which is the whole of what the ruling asked to be shown.

## What this does NOT prove, stated rather than glossed

The synthetic advisory is `informational = "unmaintained"`, the same class as
the two real entries. This proof does not exercise a `vulnerability`-class
advisory; `cargo-deny` matches `ignore` by id for both classes (the
`[advisories]` config has no per-class ignore list), so the conclusion carries,
but it is an inference from the config shape rather than a second run.

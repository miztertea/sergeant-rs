# G3 — deny.toml exception narrowness proof (RUSTSEC-2026-0192)

Companion to `SPIKE-G3.md` (the deny-gate failure/adoption record) and
`prove-exception-is-scoped.sh` (beside this file — rerun it to reproduce).
Owner ruling: `knowledge/rulings/owner-rulings/anydoc-adoption-2026-08-27.md`
(J4), condition (1): "the next advisory must still fail the gate."

## What this proves

`deny.toml`'s `[advisories.ignore]` entry names exactly one RUSTSEC id
(`RUSTSEC-2026-0192`, `ttf-parser`). `cargo-deny`'s `ignore` list matches by
advisory ID, not by crate or dependency subtree — so this is provable rather
than merely asserted: a **different** advisory ID reaching the graph through
the **same** `anydoc -> pdf-inspector -> lopdf` edge must still fail
`cargo deny check advisories`.

## Method

1. Copy `cargo-deny`'s already-fetched real advisory-db cache (this repo's
   `deny.toml` `db-path`, `~/.cargo/advisory-db`) — the real cache is never
   written to.
2. Drop a fabricated advisory file into the copy, at
   `crates/lopdf/RUSTSEC-2099-9999.md` (`lopdf` is a sibling of
   `ttf-parser` one hop up the same `pdf-inspector` dependency —
   `anydoc -> pdf-inspector -> lopdf` — so this is a different crate in the
   *same* subtree, not an arbitrary unrelated one; `RUSTSEC-2099-9999` is a
   nonexistent id chosen to be unambiguously synthetic and not collide with
   any real future advisory).
3. Run `cargo deny check advisories --offline` with this repo's real,
   unmodified `deny.toml` (read as-is), `db-path` repointed at the doctored
   copy so the fetch step is skipped entirely (`--offline`) and no network
   call happens.

## Run record

Environment: `cargo-deny 0.20.2`, `rustc 1.98.0`/`cargo 1.98.0`
(rust-toolchain.toml pin), `Cargo.lock` with `anydoc = "0.2.4"` present
(this wave's addition), `TMPDIR=/var/tmp/sgt-test-tmp`. Run at
2026-08-27T23:52Z via `tests/fixtures/anydoc_corpus/prove-exception-is-scoped.sh`.

```
$ TMPDIR=/var/tmp/sgt-test-tmp tests/fixtures/anydoc_corpus/prove-exception-is-scoped.sh
== copying cached advisory-db (advisory-db-3157b0e258782691) — the real cache is never touched ==
== dropping a SYNTHETIC advisory against lopdf (sibling of ttf-parser under anydoc -> pdf-inspector -> lopdf) ==
== running cargo deny check advisories against the repo's real deny.toml, db-path repointed at the doctored copy ==
----- cargo deny output -----
error[unmaintained]: SYNTHETIC advisory — gate-narrowness proof only, not a real vulnerability
    ┌─ /var/tmp/hats4/y2/Cargo.lock:189:1
    │
189 │ lopdf 0.42.0 registry+https://github.com/rust-lang/crates.io-index
    │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ unmaintained advisory detected
    │
    ├ ID: RUSTSEC-2099-9999
    ├ Advisory: https://rustsec.org/advisories/RUSTSEC-2099-9999
    ├ Fabricated by `tests/fixtures/anydoc_corpus/prove-exception-is-scoped.sh` to
      prove deny.toml's `RUSTSEC-2026-0192` ignore entry does not suppress a
      *different* advisory ID reaching the graph through the same `anydoc`
      subtree.
    ├ Announcement: https://example.invalid/synthetic-advisory-for-gate-proof
    ├ Solution: No safe upgrade is available!
    ├ lopdf v0.42.0
      └── pdf-inspector v1.17.0
          └── anydoc v0.2.4
              └── sergeant-rs v0.3.0

error[yanked]: detected yanked crate (try `cargo update -p chacha20`)
   ┌─ /var/tmp/hats4/y2/Cargo.lock:50:1
   │
50 │ chacha20 0.10.1 registry+https://github.com/rust-lang/crates.io-index
   │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ yanked version
   │
   ├ chacha20 v0.10.1
     └── rand v0.10.2
         ├── lopdf v0.42.0
         │   └── pdf-inspector v1.17.0
         │       └── anydoc v0.2.4
         │           └── sergeant-rs v0.3.0
         └── ulid v3.0.0
             └── sergeant-rs v0.3.0 (*)

advisories FAILED
------------------------------
PASS: RUSTSEC-2026-0192 stays ignored; RUSTSEC-2099-9999 (a different advisory, same anydoc subtree) still fails the gate.
```

## Reading the output

- `RUSTSEC-2026-0192` (the real `ttf-parser` advisory) does **not** appear
  anywhere in this output — the scoped exception covers it as designed.
- `RUSTSEC-2099-9999` (the synthetic `lopdf` advisory, injected one hop
  from `ttf-parser` on the exact same `anydoc -> pdf-inspector -> lopdf`
  edge) surfaces as a full `error[unmaintained]` and the command exits
  non-zero on it.
- The `chacha20`/`rand`/`ulid` failure is the pre-existing, unrelated
  baseline advisory (sergeant-rs#328 — also now reachable via
  `lopdf -> rand`, same crate/version, not a new advisory) — present before
  this wave, untouched by it, and not this exception's concern either way.

**Verdict:** the exception's edge is exactly where the config says it is —
one advisory ID, not the crate, not the subtree, not a disabled gate.

# `potion-code-16M-v2` — the semantic model this build ships

A2 §6's *"small local static embedding model"*, decision **A2-06**'s named
candidate ([EXT-MODEL2VEC]). Loaded by
`src/runtime/atlas/semantic.rs::SemanticEngine`, which reads this directory
as a path — `model2vec-rs`'s `local-only` loader takes a **directory**, which
is why these are files on disk and not `include_bytes!`.

| Fact | Value |
|---|---|
| Repository | `minishlab/potion-code-16M-v2` (HuggingFace) |
| Revision (version pin) | `e9d2a44ca6a05ac6685f3b23709ea57eb7352d5b` |
| License | MIT (`cardData.license` = `mit`, HuggingFace model API, checked 2026-08-30) |
| Files | `config.json`, `model.safetensors`, `tokenizer.json` |

## sha256 of each file, as committed

```text
148e5691a6fcc553437156859701fba017a1ba5d340b170f17e0f3668fb861a7  config.json
75cf7a6c2171b230ad19b1e7d8e0b1aee86da5a02af8e7cacedd9921d227623c  model.safetensors
107bbdcbad4bff1d299b7a4c3a2fb17c52890688b7dd0e4c9deab79d3c4f3d45  tokenizer.json
```

Re-record both this table and the digests when the revision moves. That is
the same discipline `scripts/release/install-dist.sh` already applies to its
own third-party artifact, and for the same stated reason: *"Bumping the
version means re-recording both sha256 literals below … deliberately, so a
version bump is a qualification, not a one-character edit."*

## Why the bytes are in this repository

Owner ruling `knowledge/rulings/owner-rulings/
model2vec-paste-advisory-2026-08-30.md`, section *"DECIDED — ship the
weights, do not build a fetch mechanism"* (**J4**): at ~33.5 MB the fetch
mechanism the ruling had been considering is not worth building, and
cargo-dist's `include` copies a directory *"into the root of all archives and
installers"* for free. `Cargo.toml`'s `[workspace.metadata.dist] include`
names this directory, so it rides beside `sgt` in every release archive.

Consequence, recorded rather than discovered later: **release size grows
~33.5 MB per target**, and this repository grows by the same.

## Three files, not five

The upstream repo also carries `README.md` and `modules.json`.
`model2vec-rs`'s `match_local_layout` requires exactly `config.json`,
`tokenizer.json` and `model.safetensors` to `exists()`
(`model2vec-rs-0.2.1/src/model.rs`) and reads nothing else, so the other two
are not shipped (**R1**).

## Nothing here is downloaded at run time

`model2vec-rs` is declared `default-features = false, features =
["local-only"]`, so `hf-hub`/`ureq` are never compiled and every download
item in the crate is `#[cfg(all(feature = "hf-hub", not(feature =
"local-only")))]` — A2-12 met by absence of a code path.
`tests/w3b_model2vec_manifest_pin.rs` reads the manifest and fails if that
declaration ever changes.

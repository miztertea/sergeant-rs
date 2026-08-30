//! S5 W3b — **A2-12's real structural pin, and it is on the CARGO MANIFEST.**
//!
//! Decision A2-12: *"Explicitly install/pin semantic assets; no stage-time
//! surprise download"*, with A2 §15's *"Do not surprise-download a model in
//! the middle of a stage."*
//!
//! # Why the manifest and not a source scan
//!
//! W3 landed a file-text guard
//! (`the_semantic_module_names_no_obvious_fetcher`) that read
//! `src/runtime/atlas/semantic.rs` and looked for URL-shaped strings. That
//! guard is **weak and its own documentation now says so**: it cannot see a
//! sibling-module fetcher, a `Command::new("curl")`, or a `concat!`-assembled
//! URL, and `reqwest` is already in this crate's graph for backend transport,
//! so the absence it observes is an absence of *obvious* code rather than of
//! capability.
//!
//! The real pin is one level down. `model2vec-rs` is declared
//! `default-features = false, features = ["local-only"]`, and **every
//! download item in that crate is
//! `#[cfg(all(feature = "hf-hub", not(feature = "local-only")))]`** — so
//! `hf-hub` and `ureq` are never compiled and the fetcher does not exist in
//! this binary to be called. A2-12 is met by **absence of a code path**, and
//! the manifest is the file that decides whether that absence holds. This
//! suite reads `Cargo.toml`, `Cargo.lock` and `deny.toml` as data and fails
//! when any of the four properties the owner ruling names stops being true.
//!
//! It is deliberately a *structural* test in this program's sense: it fails
//! when the boundary is violated, not when a doc paragraph goes stale
//! (`@@boundaries-are-the-product`).

use std::path::{Path, PathBuf};

use sergeant_rs::runtime::atlas::semantic::{MODEL_ASSET_DIR_NAME, MODEL_FILES};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn manifest() -> toml::Value {
    toml::from_str(&read("Cargo.toml")).expect("Cargo.toml parses")
}

fn dependency(name: &str) -> toml::Value {
    manifest()
        .get("dependencies")
        .and_then(|d| d.get(name))
        .unwrap_or_else(|| panic!("Cargo.toml [dependencies] must declare {name}"))
        .clone()
}

fn features(entry: &toml::Value) -> Vec<String> {
    entry
        .get("features")
        .and_then(|f| f.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// **The pin.** `model2vec-rs` must be declared with default features OFF and
/// `local-only` ON.
///
/// Turning either half around recompiles the crate's `hf-hub`/`ureq` fetcher
/// into this binary — the crate's `default = ["onig", "hf-hub"]`, and
/// `hf-hub = ["dep:hf-hub", "dep:ureq"]`. That is the moment A2-12 stops
/// being structural, and it is a one-character edit, which is exactly why it
/// needs a test rather than a comment.
#[test]
fn model2vec_is_declared_local_only_with_default_features_off() {
    let entry = dependency("model2vec-rs");
    assert_eq!(
        entry.get("default-features").and_then(toml::Value::as_bool),
        Some(false),
        "model2vec-rs must set default-features = false — its defaults are \
         [\"onig\", \"hf-hub\"], and hf-hub = [\"dep:hf-hub\", \"dep:ureq\"]: an HTTP \
         client and a HuggingFace downloader compiled into sgt. Declaration was: {entry}"
    );
    assert!(
        features(&entry).contains(&"local-only".to_string()),
        "model2vec-rs must select the local-only feature — it is the crate's own kill \
         switch, and every download item is \
         #[cfg(all(feature = \"hf-hub\", not(feature = \"local-only\")))]. \
         Declaration was: {entry}"
    );
}

/// `hf-hub` must not appear in the lockfile at all, and `model2vec-rs`'s own
/// resolved dependency list must name neither it nor `ureq`.
///
/// The complement of the test above, checked at the other end: the manifest
/// says what was asked for, the lockfile says what Cargo resolved. If a
/// future edit turned the feature back on, the declaration test above would
/// go red — and if some *other* route dragged the downloader in, this is what
/// would.
///
/// **`ureq`'s absence is asserted on model2vec's edge, not globally, and that
/// distinction is measured rather than assumed.** `ureq 3.4.0` is already in
/// this lockfile at the wave's base commit as a **build**-dependency of
/// `libduckdb-sys` (`cargo tree -i ureq`), entirely unrelated to embeddings.
/// A blanket "no ureq anywhere" assertion would be red on arrival and would
/// be pinning someone else's dependency.
#[test]
fn no_huggingface_downloader_reaches_the_lockfile_or_the_model2vec_edge() {
    let lock = read("Cargo.lock");
    assert!(
        !lock.contains("\nname = \"hf-hub\""),
        "hf-hub resolved into Cargo.lock — A2-12's 'no stage-time surprise download' is \
         no longer met by absence of a code path"
    );

    let package = lock
        .split("[[package]]")
        .find(|block| block.contains("name = \"model2vec-rs\""))
        .expect("model2vec-rs must be in Cargo.lock");
    for forbidden in ["\"hf-hub\"", "\"ureq\""] {
        assert!(
            !package.contains(forbidden),
            "model2vec-rs resolved {forbidden} as a dependency — the local-only \
             declaration is no longer doing what A2-12 relies on it for. Entry was:\n\
             {package}"
        );
    }
}

/// **RUSTSEC-2025-0119 stays AVOIDED, not excepted** (owner ruling
/// `model2vec-paste-advisory-2026-08-30.md`, decision 3).
///
/// `tokenizers` must be declared directly with `fancy-regex` and WITHOUT
/// `progressbar`. Taking the regex backend through `model2vec-rs`'s own
/// `fancy-regex` feature instead would bundle `tokenizers/progressbar`, which
/// drags in `indicatif -> number_prefix` and that advisory. Checked from both
/// ends again: the declaration, and the resolved lockfile.
#[test]
fn the_regex_backend_is_selected_without_the_progress_bar_and_indicatif_stays_out() {
    let entry = dependency("tokenizers");
    assert_eq!(
        entry.get("default-features").and_then(toml::Value::as_bool),
        Some(false),
        "tokenizers must set default-features = false: {entry}"
    );
    let selected = features(&entry);
    assert!(
        selected.contains(&"fancy-regex".to_string()),
        "tokenizers refuses to compile with no regex backend (compile_error! in \
         tokenizers-0.21.4/src/utils/mod.rs), so one must be selected here: {entry}"
    );
    assert!(
        !selected.contains(&"progressbar".to_string()),
        "progressbar pulls indicatif -> number_prefix (RUSTSEC-2025-0119), which the \
         owner ruling requires stay AVOIDED rather than excepted: {entry}"
    );

    let lock = read("Cargo.lock");
    for forbidden in ["\nname = \"indicatif\"", "\nname = \"number_prefix\""] {
        assert!(
            !lock.contains(forbidden),
            "{forbidden:?} resolved into Cargo.lock — RUSTSEC-2025-0119 is back, and the \
             ruling says it must be avoided by configuration, never suppressed by a \
             second deny.toml entry"
        );
    }
}

/// The `deny.toml` exception is **exactly two named advisory ids** — the
/// pre-existing S4 Y2 one (RUSTSEC-2026-0192) and this wave's — and nothing
/// broader. (The other entry's subject crate is deliberately not named here:
/// `tests/y2_office_boundary.rs::anydoc_is_named_nowhere_but_the_office_adapter`
/// keeps that name inside its own adapter, and this file is not an
/// exception to it.)
///
/// The ruling: *"never a broadened rule, never a disabled gate … The next
/// advisory must still fail the gate."* This test pins the shape; the
/// behaviour (a *different* advisory through the same subtree still fails) is
/// proven by `tests/fixtures/model2vec_corpus/prove-exception-is-scoped.sh`,
/// whose recorded run is `F5-exception-narrowness-proof.md` beside it.
#[test]
fn the_deny_exception_names_exactly_two_advisory_ids_and_nothing_broader() {
    let deny: toml::Value = toml::from_str(&read("deny.toml")).expect("deny.toml parses");
    let advisories = deny.get("advisories").expect("[advisories] section");
    let ignored: Vec<String> = advisories
        .get("ignore")
        .and_then(|v| v.as_array())
        .expect("[advisories] ignore list")
        .iter()
        .map(|entry| {
            entry
                .get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or_else(|| panic!("every ignore entry must name an id: {entry}"))
                .to_string()
        })
        .collect();
    assert_eq!(
        ignored,
        vec![
            "RUSTSEC-2026-0192".to_string(),
            "RUSTSEC-2024-0436".to_string()
        ],
        "the ignore list must be exactly the two ruled advisory ids, in the order they \
         were ruled — a third entry is a decision nobody made, and a crate-shaped or \
         wildcard entry is the broadened rule both rulings forbid"
    );
    assert!(
        !ignored.iter().any(|id| id.contains('*')),
        "no wildcard ids: {ignored:?}"
    );
    assert_eq!(
        advisories.get("yanked").and_then(toml::Value::as_str),
        Some("deny"),
        "the gate stays a gate"
    );

    // `paste` really is in the graph, so the exception covers a live edge
    // rather than a stale one that could be deleted unnoticed.
    assert!(
        read("Cargo.lock").contains("\nname = \"paste\""),
        "RUSTSEC-2024-0436's subject left the lockfile — the exception is now dead \
         weight and one of the ruling's revisit triggers has fired"
    );
}

/// The weights ride with the release: `[workspace.metadata.dist] include`
/// must name the asset directory, and that directory must actually hold the
/// three runtime files.
///
/// Owner ruling, *"DECIDED — ship the weights, do not build a fetch
/// mechanism"*. Both halves are needed: an `include` naming a directory that
/// is not there ships an archive without a model, and assets committed but
/// not `include`d ship a binary that finds nothing beside it.
#[test]
fn the_release_archives_carry_the_asset_directory_and_it_is_complete() {
    let include: Vec<String> = manifest()
        .get("workspace")
        .and_then(|w| w.get("metadata"))
        .and_then(|m| m.get("dist"))
        .and_then(|d| d.get("include"))
        .and_then(|v| v.as_array())
        .expect("[workspace.metadata.dist] include")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let expected = format!("assets/{MODEL_ASSET_DIR_NAME}/");
    assert!(
        include.contains(&expected),
        "cargo-dist include must carry {expected:?} — it is what puts the model beside \
         sgt in every archive and installer. Was: {include:?}"
    );

    let assets = repo_root().join("assets").join(MODEL_ASSET_DIR_NAME);
    for name in MODEL_FILES {
        let file = assets.join(name);
        assert!(
            file.is_file(),
            "{} is missing — model2vec-rs's local loader needs all three",
            file.display()
        );
    }
    assert!(
        assets.join("PROVENANCE.md").is_file(),
        "the assets must carry their provenance: repo, revision SHA, per-file sha256, \
         license"
    );
}

/// The committed weights are the exact bytes `PROVENANCE.md` records.
///
/// This is the *content* half of A2 §15's *"content/version pinned"*. It is
/// the same discipline `scripts/release/install-dist.sh` applies to its own
/// third-party artifact, and it makes a silent asset swap — a re-download, a
/// partial `git lfs` fetch, a hand edit — fail here rather than change search
/// results quietly.
#[test]
fn the_committed_weights_match_the_digests_recorded_beside_them() {
    let assets = repo_root().join("assets").join(MODEL_ASSET_DIR_NAME);
    let provenance = std::fs::read_to_string(assets.join("PROVENANCE.md")).expect("provenance");
    for name in MODEL_FILES {
        let digest = sha256_of(&assets.join(name));
        assert!(
            provenance.contains(&format!("{digest}  {name}")),
            "{name}: sha256 {digest} is not the digest PROVENANCE.md records. Either the \
             bytes changed or the record did; both are a re-qualification, not an edit."
        );
    }
}

/// R5: `sha2` is already resolved into this exact build's `Cargo.lock`
/// transitively (`lopdf -> pdf-inspector -> anydoc -> sergeant-rs`), so
/// using it here is naming an already-installed dependency, not adding a
/// new edge to the graph (F-SI-01). `blake3` is the crate's own hash and is
/// not SHA-256, so it cannot stand in for this check.
fn sha256_of(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let digest = Sha256::digest(&data);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

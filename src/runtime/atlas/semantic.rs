//! S5 W3/W3b — A2 §6's semantic retrieval, and A2 §15's honesty about it.
//!
//! # Two things live here, and the second one arrived later
//!
//! **W3 landed decision H4**: A2 §15's sentence — *"If semantic assets are
//! absent, A2 degrades to deterministic filters + structural/exact + BM25
//! lexical retrieval and reports that coverage/capability honestly"* — as a
//! **required field** rather than a principle. A consumer must be able to
//! tell a degraded answer from a complete one *mechanically*, by reading a
//! value that is always there, rather than by noticing that some other field
//! is absent. So [`SemanticStatus`] rides every search answer and is not an
//! `Option`, and it is deliberately **distinct** from A2 §13's *"semantic
//! model identity/hash **if used**"*, which is [`SemanticModel`] and *is*
//! optional. Two fields, because they answer two different questions:
//!
//! | Question | Field | Shape |
//! |---|---|---|
//! | Did the semantic half participate, and if not why? | [`SemanticStatus`] | required |
//! | Which model, at which content hash, if one did? | [`SemanticModel`] | `Option` |
//!
//! Collapsing them into one optional field is the exact failure H4 names: a
//! `None` model would mean "not installed", "disabled by the caller" and
//! "the field was never populated" indistinguishably, and a consumer would
//! be inferring degradation from an absence. `tests/w3_semantic_degradation.
//! rs::disabled_and_not_installed_are_distinguishable_when_the_model_field_
//! alone_is_not` is what makes that argument a test rather than a paragraph.
//!
//! **W3b landed the model itself** — [`SemanticEngine`], [`cosine`],
//! [`SemanticHit`] and [`rank_semantic`] — so [`SemanticStatus::Applied`] is
//! now reachable. W3's spike had stopped at F5 gate 1 because
//! `model2vec-rs` unavoidably introduces RUSTSEC-2024-0436 (`paste`, via
//! `tokenizers`) and wrongly reported the adoption as re-opened; the owner
//! ruled otherwise (`knowledge/rulings/owner-rulings/
//! model2vec-paste-advisory-2026-08-30.md`, **J4**): **A2-06 was always a
//! ratified decision**, and the advisory is a scoped, dated `deny.toml`
//! exception naming that one ID. The spike's own record —
//! `tests/fixtures/model2vec_corpus/SPIKE-F5.md` — remains the evidence for
//! *why* the exception is unavoidable, and is not re-derived here.
//!
//! # What the semantic half actually does (A2 §6, A2-07)
//!
//! [`SemanticEngine`] loads `potion-code-16M-v2` from a directory of three
//! files and mean-pools static token embeddings in process — no GPU, no
//! remote inference API (A2 §6: *"A2 does not require GPU inference or
//! external embedding APIs"*). Ranking is an **exact cosine scan** over the
//! admissible set and nothing else: A2-07 (**R1**) and A2 §16's
//! *"vector database/ANN engine before measurement"* non-goal mean there is
//! no index to build and none is built. See
//! [`crate::runtime::atlas::db::AtlasDb::semantic_search`] for the scan, and
//! `knowledge/evidence/perf/model2vec-footprint-and-scan-2026-08-30.md` for
//! the measurement that would be the only thing able to change it.
//!
//! **A2 §8's prohibition is structural here, not procedural.** The scan's
//! candidate set is [`crate::runtime::atlas::db::AtlasDb::admissible_generations`]'s
//! output, so a unit whose generation the filter excludes is never embedded,
//! never scored, and cannot appear however well it would have matched —
//! *"The reranker must never silently cross an authority/source filter
//! merely because a candidate scores well."*
//!
//! # Why the vocabulary is three values
//!
//! `applied | not_installed | disabled` is **H4's** spelling, verbatim
//! (sprint plan `sprint-plan-2026-08-28.md`). It is NOT A2's own words: A2
//! §15 states the honesty REQUIREMENT (*"reports that coverage/capability
//! honestly"*) and §17 item 4 the capability (*"disabled/degraded
//! cleanly"*), but the strings `applied` and `not_installed` appear nowhere
//! in A2. H4 chose the spelling; the contract chose the obligation.
//!
//! All three are now reachable from [`resolve`] in production:
//! [`installed_model`] answers `Some` on a host where the release archive's
//! `semantic-model/` directory (or `$SGT_SEMANTIC_MODEL_DIR`) holds the
//! three runtime files, and `None` on one where it does not — a `cargo
//! install` from source, or a hand-copied binary. **Shipping the assets by
//! default does not make their absence unrepresentable**, which is why
//! `tests/w3_semantic_degradation.rs` and
//! `tests/w3b_semantic_retrieval.rs::
//! a_host_without_assets_still_answers_lexically_and_reports_not_installed`
//! both still run against a host with none.
//!
//! Pure Rust over strings and enums: no database connection, no driver name
//! (Atlas's one-owner invariant, `tests/x1_atlas_substrate.rs::
//! atlas_database_has_exactly_one_owner`). The one filesystem access is
//! reading the model assets, which is what loading a model is.

use std::path::{Path, PathBuf};

use model2vec_rs::model::StaticModel;

use crate::domain::source::{AuthorityClass, SourceKind};
use crate::runtime::atlas::lexical::UnitCoordinate;

/// What the **caller asked for** — the request side, distinct from what the
/// world could deliver.
///
/// Two variants rather than a `bool` on the query struct: at a construction
/// site `semantic: SemanticRequest::Suppressed` says which way round it is
/// and `semantic: false` does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticRequest {
    /// Use the semantic half if it is available. The default a caller who
    /// said nothing about it should get — A2-13 keeps semantic retrieval
    /// optional, which is a statement about *availability*, not about a
    /// caller having to opt in.
    Requested,
    /// Do not use the semantic half even if it is available. A deliberate
    /// caller choice, and the answer says so rather than looking identical
    /// to a host that never installed a model.
    Suppressed,
}

/// A2 §13's *"semantic model identity/hash if used"* — the **optional**
/// field, and the one [`SemanticStatus`] must not be folded into.
///
/// Carried only when a model actually contributed to the answer. Both parts
/// are recorded because A2 §15 pins assets by *content and version*, so
/// "which model" and "which bytes of it" are separate facts: a repointed
/// identity with unchanged bytes and unchanged bytes under a new identity
/// are different situations, and a trace that records only one of them
/// cannot tell them apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticModel {
    /// The model's pinned identity — repository and revision, as installed.
    pub identity: String,
    /// The content hash of the installed assets.
    pub content_hash: String,
}

/// A2 §15's required honesty, as a value: **did the semantic half
/// participate in this answer, and if not, why not?**
///
/// Non-omittable by construction — it is a plain enum on the answer struct,
/// not an `Option`, so there is no "unset" to forget. The three variants are
/// H4's vocabulary, `applied | not_installed | disabled` — H4's spelling of
/// A2 §15's honesty requirement and §17 item 4's "disabled/degraded", not
/// strings A2 itself contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticStatus {
    /// A model was installed, the caller wanted it, and it contributed to
    /// this answer. The answer is complete in A2 §7's sense: both halves
    /// were available to fuse.
    Applied,
    /// No semantic assets are installed on this host, so the answer is the
    /// deterministic-filter + lexical half only. A2 §15's degraded state —
    /// **supported, not an error**, and reported rather than inferred.
    NotInstalled,
    /// The caller asked for the semantic half to be left out. The answer is
    /// degraded in exactly the same way as [`Self::NotInstalled`], for an
    /// entirely different reason, and a consumer that must not confuse a
    /// host-configuration fact with a caller's choice can tell them apart.
    Disabled,
    /// The model is installed and the caller wanted it, but at least one
    /// admissible generation carries **no stored vectors for this model** —
    /// so the semantic ranking this answer could offer is incomplete, or
    /// absent entirely.
    ///
    /// The state a store scanned before its vectors existed is in, and the
    /// state a store scanned under a *different* model is in: vectors are
    /// keyed by [`SemanticModel`], so a model swap invalidates rather than
    /// silently reusing (by analogy with A1 §8's extractor-identity cache
    /// discipline).
    ///
    /// **This variant exists because none of the other three can say it.**
    /// [`Self::Applied`] would claim a ranking the store cannot produce,
    /// which is A1 §15's *"missing capability … represented as successful
    /// empty evidence"*; [`Self::NotInstalled`] would blame the host for a
    /// model that is in fact loaded and send an operator to install it
    /// again; [`Self::Disabled`] would blame the caller. The remedy is a
    /// re-scan, and only a word of its own points at it.
    NotIndexed,
}

impl SemanticStatus {
    /// H4's own three spellings — `applied` | `not_installed` | `disabled`
    /// — plus S6's `not_indexed`, as the stable wire word every surface
    /// renders.
    ///
    /// One function rather than a `match` at each render site: A2 §15's
    /// honesty is only mechanical if every consumer reads the same three
    /// words, and a second spelling of "degraded" is a second vocabulary.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::NotInstalled => "not_installed",
            Self::Disabled => "disabled",
            Self::NotIndexed => "not_indexed",
        }
    }
}

/// Resolve the status of one answer from the caller's request and the
/// model — if any — that was actually installed.
///
/// **The precedence is stated, not incidental: a suppressed request reports
/// [`SemanticStatus::Disabled`] even on a host with no model installed.**
/// Both facts are true in that case and only one value can be reported, so
/// the rule is to report the one the caller can act on: the caller turned it
/// off, and installing a model would not change this answer. The reverse
/// precedence would tell an operator to go install something that would then
/// still not be used. `tests/w3_semantic_degradation.rs::
/// a_suppressed_request_reports_disabled_even_with_no_model_installed` fails
/// if this ever flips.
pub fn resolve(request: SemanticRequest, model: Option<&SemanticModel>) -> SemanticStatus {
    match (request, model) {
        (SemanticRequest::Suppressed, _) => SemanticStatus::Disabled,
        (SemanticRequest::Requested, None) => SemanticStatus::NotInstalled,
        (SemanticRequest::Requested, Some(_)) => SemanticStatus::Applied,
    }
}

/// The semantic model installed on this host, as a **descriptor only** —
/// `Some` when a complete asset directory is present and loadable, `None`
/// when there is none.
///
/// This is the cheap answer H4's field needs on a path that is not going to
/// run a semantic scan (`lexical_search`'s status resolution): it still pays
/// for the load, so callers that will actually embed something hold a
/// [`SemanticEngine`] instead of calling this per query — see
/// [`crate::runtime::atlas::db::AtlasDb::semantic_engine`], which loads at
/// most once per handle.
///
/// **A2-12 is met by absence of a code path, not by this function's
/// behaviour.** `model2vec-rs` is declared `default-features = false,
/// features = ["local-only"]` in `Cargo.toml`, so `hf-hub`/`ureq` are never
/// compiled and every download item in the crate — each one
/// `#[cfg(all(feature = "hf-hub", not(feature = "local-only")))]` — does not
/// exist in this binary to be called. That is a property of the **manifest**,
/// and `tests/w3b_model2vec_manifest_pin.rs` is the structural test that
/// reads the manifest and fails if the declaration ever loses it. A file-text
/// scan of this module could not see a sibling-module fetcher, a
/// `Command::new("curl")`, or a `concat!`-assembled URL, and `reqwest` is
/// already in this crate's graph for backend transport — so the manifest, not
/// a scan, is where the pin belongs.
pub fn installed_model() -> Option<SemanticModel> {
    SemanticEngine::load()
        .ok()
        .flatten()
        .map(|engine| engine.descriptor)
}

// ---------------------------------------------------------------------------
// S5 W3b — A2 §6's semantic half, wired to the model A2-06 named.
// ---------------------------------------------------------------------------

/// The model this build ships and this build loads, as a *pinned* identity:
/// HuggingFace repo plus the exact revision SHA whose bytes are committed
/// under [`MODEL_ASSET_DIR_NAME`].
///
/// A2 §15 requires assets to be *"content/version pinned"*. The version pin
/// is this constant; the content pin is [`SemanticModel::content_hash`],
/// computed from the bytes actually loaded rather than copied from a
/// manifest — a repointed identity over unchanged bytes and unchanged bytes
/// under a new identity are different situations, and only hashing what was
/// read tells them apart.
pub const MODEL_REPO: &str = "minishlab/potion-code-16M-v2";

/// The revision the committed assets were taken from.
pub const MODEL_REVISION: &str = "e9d2a44ca6a05ac6685f3b23709ea57eb7352d5b";

/// The directory name the release archive carries beside the `sgt` binary,
/// and the directory name in this repository's `assets/`.
///
/// cargo-dist's `include` *"copies additional files or directories into the
/// root of all archives and installers"* (`ctx7 docs /axodotdev/cargo-dist`,
/// `include` reference), so `assets/semantic-model/` arrives as
/// `semantic-model/` in the archive root — beside `sgt`, which is why
/// [`model_dir`] looks next to the executable.
pub const MODEL_ASSET_DIR_NAME: &str = "semantic-model";

/// Operator override for where the assets live: an absolute path to the
/// directory holding the three runtime files.
///
/// This is A2 §15's *"explicitly installed"* half, spelled as the one thing
/// an operator can say. It exists because the release archive's layout is a
/// convention and an installer may place the payload somewhere this build
/// cannot derive — and because a test needs to point at a fixture directory
/// without moving anything next to the test binary.
pub const MODEL_DIR_ENV: &str = "SGT_SEMANTIC_MODEL_DIR";

/// The three files `model2vec-rs`'s local loader requires — checked in code:
/// `match_local_layout` returns `Some` only when `config.json`,
/// `tokenizer.json` and `model.safetensors` all `exists()`
/// (`model2vec-rs-0.2.1/src/model.rs`).
///
/// Sorted, and hashed in this order by [`content_hash_of`], so the content
/// hash is a function of the bytes and not of directory iteration order.
pub const MODEL_FILES: [&str; 3] = ["config.json", "model.safetensors", "tokenizer.json"];

/// Why the semantic half could not be loaded on a host that appears to have
/// assets.
///
/// **Not** a variant for "no assets installed" — that is
/// [`SemanticStatus::NotInstalled`] and is not an error (A2-13). This type is
/// only for a directory that exists and still could not produce a model:
/// truncated weights, an unreadable file, a layout the loader rejects.
#[derive(Debug)]
pub enum SemanticError {
    /// The asset directory exists but the model would not load.
    Load {
        /// The directory that was tried.
        directory: PathBuf,
        /// What the loader or the filesystem said.
        detail: String,
    },
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load { directory, detail } => write!(
                f,
                "semantic model assets at {} could not be loaded: {detail}",
                directory.display()
            ),
        }
    }
}

impl std::error::Error for SemanticError {}

/// Where this build looks for the semantic assets, in order, stopping at the
/// first directory that holds all three of [`MODEL_FILES`].
///
/// 1. `$SGT_SEMANTIC_MODEL_DIR`, when set and non-empty.
/// 2. `<directory of the running executable>/semantic-model`.
///
/// **There is no third entry and deliberately no search.** A2 §15 forbids a
/// surprise download; a build that also went hunting through the filesystem
/// would answer a different question on two hosts with the same install. Two
/// places, both nameable in one sentence: what the operator said, and what
/// the release archive put beside the binary.
pub fn model_dir() -> Option<PathBuf> {
    if let Some(named) = std::env::var_os(MODEL_DIR_ENV) {
        let named = PathBuf::from(named);
        if !named.as_os_str().is_empty() {
            return complete(named);
        }
    }
    let exe = std::env::current_exe().ok()?;
    complete(exe.parent()?.join(MODEL_ASSET_DIR_NAME))
}

/// `Some(dir)` when `dir` holds every one of [`MODEL_FILES`], `None`
/// otherwise — a partially unpacked archive is *not installed*, not a
/// half-working model.
fn complete(directory: PathBuf) -> Option<PathBuf> {
    MODEL_FILES
        .iter()
        .all(|name| directory.join(name).is_file())
        .then_some(directory)
}

/// blake3 over the three runtime files, hashed in [`MODEL_FILES`] order with
/// each file's name and byte length mixed in ahead of its bytes.
///
/// The name and length are in the digest so two different files cannot swap
/// places or be concatenated into the same hash. `blake3` is already this
/// crate's content-hash function everywhere else (A1's `content_key`), so
/// this adds no dependency and produces a digest a reader can compare with
/// the rest of the estate's evidence (R2/R3).
fn content_hash_of(directory: &Path) -> Result<String, SemanticError> {
    let mut hasher = blake3::Hasher::new();
    for name in MODEL_FILES {
        let bytes = std::fs::read(directory.join(name)).map_err(|error| SemanticError::Load {
            directory: directory.to_path_buf(),
            detail: format!("{name}: {error}"),
        })?;
        hasher.update(name.as_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

/// A loaded static embedding model, plus the pinned identity of the assets it
/// was loaded from.
///
/// Held for the life of an [`crate::runtime::atlas::db::AtlasDb`] handle and
/// loaded at most once per handle: reading 32 MB of weights and parsing a
/// 1 MB tokenizer is not something a per-query path may do.
pub struct SemanticEngine {
    model: StaticModel,
    descriptor: SemanticModel,
}

impl std::fmt::Debug for SemanticEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticEngine")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}

impl SemanticEngine {
    /// Load from wherever [`model_dir`] resolves, or `Ok(None)` when no
    /// complete asset directory is there.
    ///
    /// `Ok(None)` and `Err(_)` are different answers on purpose: "this host
    /// has no model" is A2-13's supported degraded state, and "this host has
    /// assets that will not load" is a fault an operator has to hear about.
    pub fn load() -> Result<Option<Self>, SemanticError> {
        match model_dir() {
            Some(directory) => Self::load_from(&directory).map(Some),
            None => Ok(None),
        }
    }

    /// Load from a named directory. Public so a test can name a fixture
    /// directory without depending on process-wide environment state.
    pub fn load_from(directory: &Path) -> Result<Self, SemanticError> {
        let content_hash = content_hash_of(directory)?;
        // `normalize: None` keeps the model's own `config.json` in charge
        // (`"normalize": true` for potion-code-16M-v2) rather than this
        // build overriding a property of the published model.
        let model = StaticModel::from_pretrained(directory, None, None, None).map_err(|error| {
            SemanticError::Load {
                directory: directory.to_path_buf(),
                detail: error.to_string(),
            }
        })?;
        Ok(Self {
            model,
            descriptor: SemanticModel {
                identity: format!("{MODEL_REPO}@{MODEL_REVISION}"),
                content_hash,
            },
        })
    }

    /// A2 §13's *"semantic model identity/hash if used"*, for the trace.
    pub fn descriptor(&self) -> &SemanticModel {
        &self.descriptor
    }

    /// Embed one query string.
    pub fn embed_query(&self, text: &str) -> Vec<f32> {
        self.model.encode_single(text)
    }

    /// Embed a batch of unit texts, in order.
    pub fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
        self.model.encode(texts)
    }
}

/// Exact cosine similarity — **A2-07, and the whole of it.**
///
/// Decision A2-07 (R1): *"Use exact cosine first; defer ANN/vector DB"*, with
/// A2 §16 listing *"vector database/ANN engine before measurement"* as a
/// non-goal. This function and a linear scan over the admissible set are the
/// entire similarity mechanism; there is no index, no approximation and no
/// pruning. What would ever change that is a measurement, and the one this
/// wave took is recorded in
/// `knowledge/evidence/perf/model2vec-footprint-and-scan-2026-08-30.md`.
///
/// Returns `0.0` for a zero-magnitude vector rather than `NaN`: a unit whose
/// text tokenized to nothing must sort like an unrelated unit, not poison the
/// total order [`rank_semantic`] depends on.
pub fn cosine(left: &[f32], right: &[f32]) -> f64 {
    let mut dot = 0.0f64;
    let mut left_norm = 0.0f64;
    let mut right_norm = 0.0f64;
    for (a, b) in left.iter().zip(right.iter()) {
        dot += f64::from(*a) * f64::from(*b);
        left_norm += f64::from(*a) * f64::from(*a);
        right_norm += f64::from(*b) * f64::from(*b);
    }
    if left_norm == 0.0 || right_norm == 0.0 {
        return 0.0;
    }
    dot / (left_norm.sqrt() * right_norm.sqrt())
}

/// One ranked semantic hit: a cosine score and the same A1 coordinate a
/// lexical hit carries.
///
/// A separate type from
/// [`crate::runtime::atlas::lexical::LexicalHit`] because the score means
/// something else — a bounded similarity, not an unbounded BM25 sum — and
/// W4's RRF consumes two *rank* lists precisely because the two scales are
/// not comparable (A2 §7: *"rather than trying to normalize incomparable
/// score scales"*). The identity fields are the same because they must be:
/// fusion joins the two lists on `(generation_id, unit_key)`.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticHit {
    /// Cosine similarity between the query embedding and the unit embedding.
    pub score: f64,
    /// The declared source.
    pub source_name: String,
    /// **A2 §17 item 8** — the source's kind, so an answer built from this
    /// half stays as visibly external as one built from the lexical half.
    /// See [`crate::runtime::atlas::lexical::LexicalHit::authority_class`]
    /// for the whole argument; it is here for the same reason the tie-break
    /// key is: RRF's two inputs must agree on every field the fused hit
    /// carries, or the fused answer would depend on which half a candidate
    /// arrived through.
    pub source_kind: SourceKind,
    /// **A2 §17 item 8's other half** — the source's authority class.
    pub authority_class: AuthorityClass,
    /// The exact SourceGeneration this unit belongs to.
    pub generation_id: String,
    /// That generation's content identity.
    pub content_key: String,
    /// The index's own per-generation unit identity.
    pub unit_key: String,
    /// A1's coordinate for the unit itself.
    pub coordinate: UnitCoordinate,
}

impl SemanticHit {
    /// **The same stated tie-break key W2's lexical list uses** —
    /// `(source_name, relative_path, ordinal, unit_key)` — see
    /// [`crate::runtime::atlas::lexical::LexicalHit::tie_break_key`].
    ///
    /// Same, deliberately: the two rank lists are RRF's two inputs, so if
    /// they broke ties differently the fused order would depend on which
    /// half a tied unit happened to reach first. Every component is a stored
    /// value, so the order is a function of the evidence and never of
    /// iteration order.
    pub fn tie_break_key(&self) -> (&str, &str, u64, &str) {
        (
            &self.source_name,
            self.coordinate.relative_path(),
            self.coordinate.ordinal(),
            &self.unit_key,
        )
    }
}

/// Order two semantic hits: **score descending by [`f64::total_cmp`], then
/// [`SemanticHit::tie_break_key`] ascending.**
///
/// `total_cmp` rather than `partial_cmp`, for the reason W2 gives and one
/// more this wave owns: cosine over float weights produces exact ties
/// constantly (two units with identical text score identically to the last
/// bit), and a `sort_by` fed a `partial_cmp().unwrap_or(Equal)` leaves those
/// pairs in whatever order the scan produced them. **That order is an INPUT
/// to W4's RRF**, so a wobble here silently changes a fused result later.
/// `tests/w3b_semantic_retrieval.rs::
/// tied_semantic_scores_are_broken_by_the_stated_key_not_by_scan_order`
/// seeds tying units in the reverse of the key and fails if the answer
/// follows arrival instead.
pub fn rank_semantic(left: &SemanticHit, right: &SemanticHit) -> std::cmp::Ordering {
    right
        .score
        .total_cmp(&left.score)
        .then_with(|| left.tie_break_key().cmp(&right.tie_break_key()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> SemanticModel {
        SemanticModel {
            identity: "minishlab/potion-code-16M-v2@0000000".to_string(),
            content_hash: "blake3:deadbeef".to_string(),
        }
    }

    #[test]
    fn suppression_outranks_absence_in_both_orders_of_the_same_facts() {
        assert_eq!(
            resolve(SemanticRequest::Suppressed, None),
            SemanticStatus::Disabled
        );
        assert_eq!(
            resolve(SemanticRequest::Suppressed, Some(&model())),
            SemanticStatus::Disabled
        );
    }

    #[test]
    fn a_wanted_but_absent_model_is_not_installed_and_a_present_one_is_applied() {
        assert_eq!(
            resolve(SemanticRequest::Requested, None),
            SemanticStatus::NotInstalled
        );
        assert_eq!(
            resolve(SemanticRequest::Requested, Some(&model())),
            SemanticStatus::Applied
        );
    }

    /// The asset directory is a *directory*, and an incomplete one is
    /// "not installed" rather than a half-working model — [`complete`]'s
    /// whole job. A partially unpacked archive must degrade, not crash on
    /// the first search.
    #[test]
    fn a_directory_missing_any_one_runtime_file_is_not_an_installed_model() {
        let root = std::path::PathBuf::from(
            std::env::var_os("TMPDIR").unwrap_or_else(|| "/var/tmp/sgt-test-tmp".into()),
        )
        .join(format!("w3b-complete-{}", std::process::id()));
        for omitted in MODEL_FILES {
            let dir = root.join(omitted);
            std::fs::create_dir_all(&dir).expect("fixture dir");
            for name in MODEL_FILES {
                if name != omitted {
                    std::fs::write(dir.join(name), b"x").expect("fixture file");
                }
            }
            assert_eq!(
                complete(dir.clone()),
                None,
                "a directory without {omitted} must not count as installed"
            );
        }
        let dir = root.join("all");
        std::fs::create_dir_all(&dir).expect("fixture dir");
        for name in MODEL_FILES {
            std::fs::write(dir.join(name), b"x").expect("fixture file");
        }
        assert_eq!(complete(dir.clone()), Some(dir));
        std::fs::remove_dir_all(&root).ok();
    }

    /// The three names are exactly what `model2vec-rs`'s local loader
    /// requires, and they are sorted so [`content_hash_of`] is a function of
    /// the bytes rather than of directory iteration order.
    #[test]
    fn the_runtime_file_list_is_the_loaders_three_names_in_sorted_order() {
        assert_eq!(
            MODEL_FILES,
            ["config.json", "model.safetensors", "tokenizer.json"]
        );
        let mut sorted = MODEL_FILES;
        sorted.sort_unstable();
        assert_eq!(MODEL_FILES, sorted);
    }
}

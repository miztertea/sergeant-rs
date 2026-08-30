//! S5 W5 — A2 §13's search trace, as evidence rather than prose.
//!
//! A2 §13, verbatim:
//!
//! > A managed search should be reproducible/explainable enough to answer
//! > what world/ranker produced it.
//! >
//! > Record at minimum:
//! >
//! > ```text
//! > query text/hash
//! > execution/work attribution when managed
//! > source-generation filter
//! > content/authority filters
//! > retrieval generation
//! > lexical tokenizer/version
//! > semantic model identity/hash if used
//! > RRF/rerank policy version
//! > result evidence IDs + ranks
//! > ```
//! >
//! > The trace belongs in derived Atlas/journal-linked evidence, not as a
//! > giant prompt transcript.
//!
//! [`SearchTrace`] has **one field per line of that list, in the contract's
//! own order** — the same discipline
//! [`crate::runtime::atlas::fusion::RerankSignals`] applies to A2 §8's nine
//! signals, and for the same reason: a listing order the contract supplies is
//! the only ordering that is not invented here.
//!
//! # Where this trace lives, and the one thing it deliberately does not do
//!
//! §13 places the trace in *"derived Atlas/journal-linked evidence"*.
//! [`SearchTrace`] is exactly that: every identity in it is a value already
//! stored in Atlas (`source.generations`' `generation_id`/`content_key`/
//! `source_kind`/`authority_class`, `context.lexical_units`' `unit_key`) or a
//! build constant, and its work attribution is the journal's own Work id. It
//! rides the answer.
//!
//! **It is not written to the journal, and that is a decision with a
//! destination, not an oversight.** `sgt search` is a pure reader — H13.2
//! rejected query-time scanning specifically to keep it one, and
//! `tests/w1b_overlay_lifecycle_trigger.rs::
//! the_admissibility_filter_cannot_write_because_every_method_takes_an_immutable_self`
//! pins it structurally. Journaling a row per query would make every search a
//! write, which is the property that pin exists to forbid (**J5** — the pin
//! is a governing constraint, and a lower rung cannot override it). §13's
//! *"when managed"* is the seam: an unmanaged `sgt search` is a read and
//! carries its trace on the answer; persisting the trace of a **managed**
//! search — one a Work's own execution issued — is C1/S6's, where a managed
//! retrieval call has a Work execution to attach the row to. Named
//! destination: the C1 compiled-context wave.
//!
//! # Every version constant here is an identity, not a tuning number
//!
//! [`LEXICAL_TOKENIZER_VERSION`] and [`RETRIEVAL_POLICY_VERSION`] are strings
//! nobody can set and nothing reads as a weight. A2 §14 forbids exposing raw
//! retrieval weight tuning; a version that only ever appears in a trace
//! cannot be tuned because nothing consumes it as a parameter. Each is bound
//! to the behaviour it names by a structural test rather than by a promise to
//! remember — `tests/w5_search_surface.rs::
//! the_lexical_tokenizer_version_is_pinned_to_the_tokenizers_actual_output`
//! and `::the_retrieval_policy_version_is_pinned_to_the_actual_rrf_and_rerank_policy`
//! go red when the behaviour changes and the version does not.

use crate::domain::source::{AuthorityClass, SourceGeneration, SourceKind};
use crate::runtime::atlas::fusion::{FusedHit, RRF_K};
use crate::runtime::atlas::lexical::{LexicalFamily, UnitCoordinate};
use crate::runtime::atlas::semantic::{SemanticModel, SemanticStatus};

/// A2 §13's *"lexical tokenizer"* identity: which tokenizer produced the
/// terms this answer was scored over.
///
/// The function's own path, because there is exactly one tokenizer in this
/// build and naming it by its path means a reader can go read it.
pub const LEXICAL_TOKENIZER: &str = "sergeant-rs::runtime::atlas::lexical::tokenize";

/// A2 §13's *"tokenizer **version**"*.
///
/// **The rule this version obeys:** any change to what
/// [`crate::runtime::atlas::lexical::tokenize`] emits for a given input is a
/// change to the retrieval world's identity — two answers scored over
/// differently-tokenized corpora are not comparable — and must bump this
/// string. `1` because this build has shipped exactly one tokenizer and never
/// a second; the claim is not left to memory, it is pinned against the
/// tokenizer's actual output by
/// `tests/w5_search_surface.rs::
/// the_lexical_tokenizer_version_is_pinned_to_the_tokenizers_actual_output`,
/// which fails when the output changes under an unchanged version.
pub const LEXICAL_TOKENIZER_VERSION: &str = "1";

/// A2 §13's *"RRF/rerank policy version"*.
///
/// One string for both halves because A2 §7 and §8 are one policy in this
/// build: RRF at [`RRF_K`] followed by
/// [`crate::runtime::atlas::fusion::RerankSignals::priority`]'s nine signals
/// in the contract's own order. Pinned to both by
/// `tests/w5_search_surface.rs::
/// the_retrieval_policy_version_is_pinned_to_the_actual_rrf_and_rerank_policy`.
pub const RETRIEVAL_POLICY_VERSION: &str = "rrf-k60+a2s8-nine-signals/1";

/// A2 §13's *"retrieval generation"*, the half that is not a stored id.
///
/// A2 §3 allows it: *"The retrieval index may maintain its own
/// generation/model/tokenization identity, but the result's evidence
/// coordinate remains A1-owned."* This build's lexical index **has no
/// generation id of its own** — `context.lexical_units`/`lexical_postings`
/// are rebuilt per source generation and keyed by `generation_id`, carrying
/// no identity beyond it. So the retrieval generation is stated as what it
/// actually is: this index-build version, plus the exact A1 generations the
/// answer was computed over ([`RetrievalGeneration::generations`]). Bump this
/// when the shape of what gets indexed changes (which units are derived, what
/// is stored per unit) — the same rule [`LEXICAL_TOKENIZER_VERSION`] obeys
/// for how text becomes terms.
pub const RETRIEVAL_INDEX_VERSION: &str = "1";

/// A2 §13's field 1: *"query text/hash"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryIdentity {
    /// The query as the caller wrote it.
    pub text: String,
    /// BLAKE3 of the query text, hex — the crate's own content hash
    /// (**R2**/**R5**: `blake3` is already the hash every A1 content key is
    /// built from, and a second hash function would be a second identity).
    pub hash: String,
}

impl QueryIdentity {
    /// Both halves of field 1 from the text. Text *and* hash, because §13
    /// asks for both: the text is what a human reads back, the hash is what
    /// two traces are compared on without quoting a query that may be long.
    pub fn of(text: &str) -> Self {
        Self {
            text: text.to_string(),
            hash: blake3::hash(text.as_bytes()).to_hex().to_string(),
        }
    }
}

/// A2 §13's field 2: *"execution/work attribution **when managed**"*.
///
/// Two variants rather than an `Option<String>`, so *unmanaged* is a stated
/// answer rather than a missing field a reader could mistake for "nobody
/// filled this in" — the same argument decision **H4** makes for
/// [`SemanticStatus`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribution {
    /// A search issued by a human at the CLI, attributable to no Work.
    /// §13's *"when managed"* is why this is legitimate rather than a gap.
    Unmanaged,
    /// A search issued inside a Work's world — the `--work <id>` selector.
    /// The Work id is the journal link §13's *"journal-linked evidence"*
    /// asks for.
    Work {
        /// `ops.work.work_id`.
        work_id: String,
        /// The repository the Work's search was scoped to.
        repository: String,
    },
}

/// A2 §13's field 3: *"source-generation filter"* — the A2 §2 stage-1
/// selector as it was actually applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceGenerationFilter {
    /// `any` | `named` | `exact` | `work_base` — the
    /// [`crate::runtime::atlas::db::SourceSelector`] variant.
    pub selector: &'static str,
    /// The source name the selector named, when it named one.
    pub source_name: Option<String>,
    /// The exact generation the selector pinned (`--source <name>@<sha>`).
    pub content_key: Option<String>,
    /// What the `--work` half of the answer actually covered —
    /// [`crate::runtime::atlas::db::WorkScope`] rendered, so a base-only
    /// answer is never read as A2 §2's full "including overlay" promise.
    pub work_scope: String,
}

/// A2 §13's field 4: *"content/authority filters"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAuthorityFilter {
    /// The `--content` family, when one was named.
    pub content: Option<LexicalFamily>,
    /// A2 §2's stage-4 kind (`--type knowledge`, `--external`).
    pub kind: Option<SourceKind>,
    /// A2 §2's stage-2 authority class.
    pub authority: Option<AuthorityClass>,
}

/// A2 §13's field 5: *"retrieval generation"*. See
/// [`RETRIEVAL_INDEX_VERSION`] for why it is a version **and** a generation
/// list rather than one id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalGeneration {
    /// [`RETRIEVAL_INDEX_VERSION`].
    pub index_version: &'static str,
    /// Every A1 source generation the admissibility filter admitted — the
    /// exact world this answer was computed over.
    pub generations: Vec<SourceGeneration>,
    /// Whether the admitted-generation list itself hit the store's row cap.
    /// A trace that silently described a capped world would be the same
    /// false completeness `LexicalAnswer::truncated` exists to prevent.
    pub truncated: bool,
}

/// A2 §13's field 6: *"lexical tokenizer/version"*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexicalIdentity {
    /// [`LEXICAL_TOKENIZER`].
    pub tokenizer: &'static str,
    /// [`LEXICAL_TOKENIZER_VERSION`].
    pub version: &'static str,
}

impl Default for LexicalIdentity {
    fn default() -> Self {
        Self {
            tokenizer: LEXICAL_TOKENIZER,
            version: LEXICAL_TOKENIZER_VERSION,
        }
    }
}

/// A2 §13's field 8: *"RRF/rerank policy version"*, with the one constant
/// the policy is parameterized by beside it so a trace states the expression
/// as well as naming it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolicyIdentity {
    /// [`RETRIEVAL_POLICY_VERSION`].
    pub version: &'static str,
    /// [`RRF_K`] — recorded, not settable. A2 §14's *"do not expose raw
    /// retrieval weight tuning"* is about a caller's ability to *set* a
    /// weight; a trace that could not say which `k` produced a score could
    /// not answer §13's *"what ranker produced it"*.
    pub rrf_k: f64,
}

impl Default for PolicyIdentity {
    fn default() -> Self {
        Self {
            version: RETRIEVAL_POLICY_VERSION,
            rrf_k: RRF_K,
        }
    }
}

/// One row of A2 §13's field 9: *"result evidence IDs + ranks"*.
///
/// The evidence id is the pair Atlas actually keys a unit by —
/// `(generation_id, unit_key)`, the same join key
/// [`crate::runtime::atlas::fusion::fuse`] uses — plus the A1 coordinate, so
/// a trace row resolves to exact evidence without re-running the query.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultRank {
    /// 1-based rank in the answer as returned (after A2 §8's rerank).
    pub rank: usize,
    /// The exact SourceGeneration this unit belongs to.
    pub generation_id: String,
    /// That generation's content identity.
    pub content_key: String,
    /// The declared source.
    pub source_name: String,
    /// **A2 §17 item 8** — carried in the trace as well as on the hit, so a
    /// stored trace is as visibly external as the answer it describes.
    pub source_kind: SourceKind,
    /// **A2 §17 item 8's other half.**
    pub authority_class: AuthorityClass,
    /// The index's own per-generation unit identity.
    pub unit_key: String,
    /// A1's coordinate for the unit.
    pub coordinate: UnitCoordinate,
    /// A2 §7's fused score.
    pub rrf: f64,
    /// 1-based rank in the lexical list, `None` if it did not appear there.
    pub lexical_rank: Option<usize>,
    /// 1-based rank in the semantic list, `None` if it did not appear there.
    pub semantic_rank: Option<usize>,
    /// A2 §8's nine signals, in the contract's own order — the three W4
    /// deliberately carries that cannot reorder anything today included,
    /// because *this* is what they are carried for.
    pub signals: [bool; 9],
}

impl ResultRank {
    /// One trace row from one answered hit and its 0-based position.
    pub fn of(index: usize, hit: &FusedHit) -> Self {
        Self {
            rank: index + 1,
            generation_id: hit.generation_id.clone(),
            content_key: hit.content_key.clone(),
            source_name: hit.source_name.clone(),
            source_kind: hit.source_kind,
            authority_class: hit.authority_class,
            unit_key: hit.unit_key.clone(),
            coordinate: hit.coordinate.clone(),
            rrf: hit.rrf,
            lexical_rank: hit.origins.lexical,
            semantic_rank: hit.origins.semantic,
            signals: hit.signals.priority(),
        }
    }
}

/// A2 §13's nine fields, one per line of the contract's list, in its order.
///
/// **All nine are implemented; none is a stub.** Field 2 has a stated
/// *unmanaged* value rather than a null (see [`Attribution`]), and field 5's
/// index-generation half is stated as a build version plus the exact A1
/// generations rather than an index id this build does not have (see
/// [`RETRIEVAL_INDEX_VERSION`]).
#[derive(Debug, Clone, PartialEq)]
pub struct SearchTrace {
    /// **1.** query text/hash.
    pub query: QueryIdentity,
    /// **2.** execution/work attribution when managed.
    pub attribution: Attribution,
    /// **3.** source-generation filter.
    pub source_generation_filter: SourceGenerationFilter,
    /// **4.** content/authority filters.
    pub content_authority_filter: ContentAuthorityFilter,
    /// **5.** retrieval generation.
    pub retrieval_generation: RetrievalGeneration,
    /// **6.** lexical tokenizer/version.
    pub lexical: LexicalIdentity,
    /// **7a.** decision **H4**'s required, non-omittable semantic status.
    /// Not one of §13's own nine — §13 asks only for the model *"if used"* —
    /// and required anyway, because A2 §15's honesty about a degraded answer
    /// cannot be carried by an optional field.
    pub semantic: SemanticStatus,
    /// **7.** semantic model identity/hash **if used**.
    pub semantic_model: Option<SemanticModel>,
    /// **8.** RRF/rerank policy version.
    pub policy: PolicyIdentity,
    /// **9.** result evidence IDs + ranks.
    pub results: Vec<ResultRank>,
}

impl SearchTrace {
    /// The nine field names, in A2 §13's own listing order — what a test
    /// compares the contract's list against, and what
    /// [`Self::json`] emits as keys.
    pub const FIELDS: [&'static str; 9] = [
        "query",
        "attribution",
        "source_generation_filter",
        "content_authority_filter",
        "retrieval_generation",
        "lexical",
        "semantic_model",
        "policy",
        "results",
    ];

    /// The trace as JSON — the shape the daemon returns and `sgt search
    /// --json` prints.
    ///
    /// Hand-built rather than derived: these keys are a contract surface an
    /// external consumer reads, and a `serde` rename attribute is a weaker
    /// statement of that than a literal (the same reason every other Atlas
    /// response in `api.rs` is built with `json!`).
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "query": {"text": self.query.text, "hash": self.query.hash},
            "attribution": match &self.attribution {
                Attribution::Unmanaged => serde_json::json!({"managed": false}),
                Attribution::Work { work_id, repository } => serde_json::json!({
                    "managed": true,
                    "work": work_id,
                    "repository": repository,
                }),
            },
            "source_generation_filter": {
                "selector": self.source_generation_filter.selector,
                "source": self.source_generation_filter.source_name,
                "content_key": self.source_generation_filter.content_key,
                "work_scope": self.source_generation_filter.work_scope,
            },
            "content_authority_filter": {
                "content": self.content_authority_filter.content.map(LexicalFamily::as_str),
                "kind": self.content_authority_filter.kind.map(SourceKind::as_str),
                "authority": self
                    .content_authority_filter
                    .authority
                    .map(AuthorityClass::as_str),
            },
            "retrieval_generation": {
                "index_version": self.retrieval_generation.index_version,
                "truncated": self.retrieval_generation.truncated,
                "generations": self
                    .retrieval_generation
                    .generations
                    .iter()
                    .map(|g| serde_json::json!({
                        "source": g.source_name,
                        "source_kind": g.kind.as_str(),
                        "authority_class": g.authority.as_str(),
                        "generation_id": g.id,
                        "content_key": g.content_key,
                        "observed_at": g.observed_at,
                    }))
                    .collect::<Vec<_>>(),
            },
            "lexical": {"tokenizer": self.lexical.tokenizer, "version": self.lexical.version},
            "semantic": self.semantic.as_str(),
            "semantic_model": self.semantic_model.as_ref().map(|m| serde_json::json!({
                "identity": m.identity,
                "content_hash": m.content_hash,
            })),
            "policy": {"version": self.policy.version, "rrf_k": self.policy.rrf_k},
            "results": self.results.iter().map(result_json).collect::<Vec<_>>(),
        })
    }
}

/// One [`ResultRank`] as JSON, coordinate included — A2 §3's family-shaped
/// coordinate, never flattened into a uniform byte range (W2's J5
/// correction: structured row text has a dataset key, a row key and a field
/// set, and no span at all).
fn result_json(row: &ResultRank) -> serde_json::Value {
    serde_json::json!({
        "rank": row.rank,
        "source": row.source_name,
        "source_kind": row.source_kind.as_str(),
        "authority_class": row.authority_class.as_str(),
        "generation_id": row.generation_id,
        "content_key": row.content_key,
        "unit_key": row.unit_key,
        "coordinate": coordinate_json(&row.coordinate),
        "rrf": row.rrf,
        "ranks": {"lexical": row.lexical_rank, "semantic": row.semantic_rank},
        "signals": row.signals,
    })
}

/// A2 §3's coordinate, rendered per family.
///
/// Four shapes, not one: *"structured text —
/// `source/generation/dataset/row-id/field-set`"* carries no byte span, and a
/// renderer that emitted `byte_start: 0` for it would be inventing evidence.
pub fn coordinate_json(coordinate: &UnitCoordinate) -> serde_json::Value {
    match coordinate {
        UnitCoordinate::Code {
            relative_path,
            language,
            label,
            symbol,
            ordinal,
            byte_start,
            byte_end,
        } => serde_json::json!({
            "family": "code",
            "path": relative_path,
            "language": language,
            "label": label,
            "symbol": symbol,
            "ordinal": ordinal,
            "byte_start": byte_start,
            "byte_end": byte_end,
        }),
        UnitCoordinate::Document {
            relative_path,
            ordinal,
            title,
            byte_start,
            byte_end,
        } => serde_json::json!({
            "family": "document",
            "path": relative_path,
            "ordinal": ordinal,
            "title": title,
            "byte_start": byte_start,
            "byte_end": byte_end,
        }),
        UnitCoordinate::Mail {
            relative_path,
            ordinal,
            title,
            byte_start,
            byte_end,
        } => serde_json::json!({
            "family": "mail",
            "path": relative_path,
            "ordinal": ordinal,
            "title": title,
            "byte_start": byte_start,
            "byte_end": byte_end,
        }),
        UnitCoordinate::RowText {
            relative_path,
            dataset_key,
            ordinal,
            row_key,
            fields,
        } => serde_json::json!({
            "family": "row-text",
            "path": relative_path,
            "dataset_key": dataset_key,
            "ordinal": ordinal,
            "row_key": row_key,
            "fields": fields,
        }),
    }
}

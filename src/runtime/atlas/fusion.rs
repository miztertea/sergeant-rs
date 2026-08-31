//! S5 W4 — A2 §7's Reciprocal Rank Fusion and A2 §8's deterministic
//! reranking.
//!
//! # The fusion is one expression (A2 §7, decision A2-08, **R6**)
//!
//! A2 §7, verbatim: *"Fuse lexical and semantic rank lists with simple RRF
//! rather than trying to normalize incomparable score scales:
//! `RRF(d) = Σ 1 / (k + rank_i(d))`. This is intentionally one expression,
//! then deterministic reranking."* A2-08's rung is **R6** — *"Rank fusion
//! can remain a simple deterministic expression; no framework needed"* — and
//! [`rrf_contribution`] is that one expression. There is no weight, no
//! pluggable scorer, no registry of rankers: [`fuse`] takes exactly the two
//! lists A2 §7 names and returns one list.
//!
//! **No number in this module is tunable by a caller** — [`RRF_K`] is a
//! constant with its provenance stated, and nothing else here is a number at
//! all. A2 §14's *"Do not expose raw retrieval weight tuning"* is met by
//! there being nothing to expose (and A2 §16's *"trained/learned reranker"*
//! and *"live self-tuning from Work outcomes"* non-goals by there being
//! nothing that learns).
//!
//! # Where determinism actually breaks, and the rule for each
//!
//! `RRF(d) = Σ 1/(k + rank_i(d))` is exactly reproducible; the four hazards
//! are all around it, and each one has a rule here and a test in
//! `tests/w4_rrf_fusion.rs`:
//!
//! | # | Hazard | The rule |
//! |---|---|---|
//! | 1 | candidate collection order | [`fuse`] re-sorts **both** inputs with their own stated `rank_order`/`rank_semantic` before assigning any rank, so `rank_i(d)` is a function of the hits and never of the order a caller happened to hand them over in |
//! | 2 | tie-breaking | one stated key — [`FusedHit::tie_break_key`], the *same* `(source_name, relative_path, ordinal, unit_key)` W2 and W3b each pinned — applied at every sort in this module |
//! | 3 | float summation order | the sum has exactly two terms and they are added in a fixed source order (lexical, then semantic) by [`Accumulator::total`]; a missing list contributes a literal `0.0` rather than being skipped, so the expression is the same shape for every candidate |
//! | 4 | `HashMap` iteration | there is no `HashMap` here. Candidates accumulate in a [`BTreeMap`] keyed by `(generation_id, unit_key)` |
//!
//! **And the rule binds the per-source lists, not just the fused one.**
//! `rank_i(d)` is an INPUT: a wobbly BM25 or cosine ordering silently changes
//! the fused result even when the fusion itself is perfect. W2 pinned
//! `lexical::rank_order` and W3b pinned `semantic::rank_semantic`, and hazard
//! 1 above is [`fuse`] refusing to *trust* that pinning — it re-applies it.
//!
//! Why this matters beyond tidiness: A2 §4 makes a daemon-owned query result
//! *"derived evidence with query identity/input generation/result hash"*, and
//! A2 §13 requires a search trace to record *"result evidence IDs + ranks"*.
//! A nondeterministic ranker cannot honour either — the hash would not
//! reproduce, and the recorded ranks would describe one run rather than the
//! query.
//!
//! # A2 §8's nine signals
//!
//! §8 says *"After RRF, reuse A1 structure/provenance rather than training
//! another ranker"* and lists nine. [`RerankSignals`] has one field per line
//! of that list, **in the contract's own order**, and
//! [`RerankSignals::priority`] is the rerank key. Three of the nine are
//! structurally uniform within any one answer in this architecture rather
//! than absent — that is recorded on each field, not hidden. See
//! [`RerankSignals`]'s own doc for the field-by-field account.
//!
//! # The prohibition
//!
//! *"The reranker must never silently cross an authority/source filter merely
//! because a candidate scores well."* [`fuse`] is a pure function of two
//! already-filtered lists: **it has no store handle, so it cannot fetch a
//! candidate**, and every candidate in its output came from one of its two
//! inputs. `AtlasDb::fused_search` builds those two inputs from one
//! `LexicalQuery` — one filter value, both halves — and the structural
//! reading a *second* list makes necessary is the one
//! `tests/w4_rrf_fusion.rs::
//! an_inadmissible_unit_that_wins_the_semantic_list_never_reaches_the_fused_answer`
//! makes non-vacuous: the decoy is shown ranking first, in the semantic list
//! and in the fused answer, with the filter open, and absent from both with
//! it closed.

use std::collections::BTreeMap;

use crate::domain::source::{AuthorityClass, SourceKind};

use crate::runtime::atlas::lexical::{LexicalHit, UnitCoordinate, rank_order};
use crate::runtime::atlas::semantic::{SemanticHit, rank_semantic};

/// RRF's `k`.
///
/// **Provenance: this is the published default, not a measurement of this
/// corpus.** `k = 60` is the value Cormack, Clarke and Büttcher's 2009 SIGIR
/// paper introduced RRF with, and it is used here because no corpus of
/// Sergeant evidence units has been measured against any other. When one is,
/// this constant is the thing a measurement replaces; until then it is
/// honest to say it was inherited rather than derived — exactly the account
/// [`crate::runtime::atlas::lexical::BM25_K1`] gives of its own value.
///
/// It is a `const`, not a field and not a config key: A2 §14 forbids exposing
/// raw retrieval weight tuning, and a constant nobody can set is the cheapest
/// way to mean it (**R6**).
pub const RRF_K: f64 = 60.0;

/// One list's contribution to a candidate's fused score: `1 / (k + rank)`.
///
/// `rank` is **1-based** — the best hit in a list has rank 1 — so the first
/// hit of a list contributes `1/61` rather than `1/60`. A 0-based rank would
/// make the two spellings of "best" differ by a whole `k`-step and make this
/// constant mean something else than the literature's.
pub fn rrf_contribution(rank: usize) -> f64 {
    1.0 / (RRF_K + rank as f64)
}

/// semble's `_ALPHA_SYMBOL` — the weight the **semantic** half carries when
/// the query is a bare symbol (`ranking/weighting.py`, verbatim comment:
/// *"lean BM25 for exact keyword matching"*). The lexical half therefore
/// carries `1 - ALPHA_SYMBOL = 0.7`.
///
/// **Provenance: adopted, not derived (R5).** These are the shipped, measured
/// constants of an installed dependency
/// (`semble/ranking/weighting.py`, [EXT-SEMBLE] — the prior art A2 §5 and §7
/// already cite), not numbers fitted to any Sergeant corpus. Fitting them to
/// one would be the *live self-tuning* A2 §16 forbids; adopting a shipped
/// design is R5 reuse. Like [`RRF_K`] they are `const`, reachable by no
/// caller and no config key, which is how A2 §14's *"do not expose raw
/// retrieval weight tuning in workflow files"* is met.
pub const ALPHA_SYMBOL: f64 = 0.3;

/// semble's `_ALPHA_NL` — *"balanced semantic + BM25"*. See [`ALPHA_SYMBOL`]
/// for the provenance of both.
pub const ALPHA_NATURAL: f64 = 0.5;

/// Is this query a bare symbol rather than prose? — semble's
/// `ranking/boosting.py::is_symbol_query`.
///
/// semble spells it as one anchored regex over the stripped query; the same
/// rule in Rust is: **one whitespace-free token**, made of identifier
/// characters and namespace separators, that is either namespace-qualified,
/// starts with an underscore, or carries an uppercase letter or an
/// underscore. semble's own comment states the discriminating case — *"plain
/// lowercase words (e.g. `session`) are NL, not symbols"* — and that is the
/// case this function exists to get right.
///
/// Not [`crate::runtime::atlas::lexical::is_identifier_like`] (R2 checked and
/// rejected): that predicate is true when **any** compound inside a longer
/// text is identifier-shaped, so it calls *"how is SourceKind validated"* an
/// identifier query. semble deliberately calls that prose. The two answer
/// different questions and both are wanted — `is_identifier_like` still
/// decides A2 §8's *"definition over reference when query is
/// identifier-like"* signal.
pub fn is_symbol_query(query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() || query.chars().any(char::is_whitespace) {
        return false;
    }
    let namespaced = ["::", "->", "\\", "."]
        .iter()
        .any(|separator| query.contains(separator));
    if !query
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || namespaced && !c.is_ascii_alphanumeric())
    {
        return false;
    }
    namespaced
        || query.starts_with('_')
        || query.contains('_')
        || query.chars().any(|c| c.is_ascii_uppercase())
}

/// The blend weight for the semantic half — semble's
/// `ranking/weighting.py::resolve_alpha`, minus its caller-supplied override
/// (A2 §14: there is no knob to override it with).
pub fn resolve_alpha(query: &str) -> f64 {
    if is_symbol_query(query) {
        ALPHA_SYMBOL
    } else {
        ALPHA_NATURAL
    }
}

/// Which of A2 §7's two rank lists a candidate appeared in, and at what rank.
///
/// Kept on every [`FusedHit`] because A2 §13's trace records *"result
/// evidence IDs + ranks"* — the fused score alone cannot say whether a hit
/// arrived through BM25, through cosine, or through both, and "both" is the
/// interesting answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RankOrigins {
    /// 1-based rank in the lexical list, or `None` if it did not appear.
    pub lexical: Option<usize>,
    /// 1-based rank in the semantic list, or `None` if it did not appear.
    pub semantic: Option<usize>,
}

/// A2 §8's nine signals, one field per line of the contract's list, **in the
/// contract's own order** — which is also [`Self::priority`]'s order.
///
/// > exact symbol / heading / filename match
/// > definition over reference when query is identifier-like
/// > source explicitly selected by caller
/// > Work-changed unit
/// > same module/package/document section
/// > inbound/outbound structural relationship
/// > canonical implementation vs test/example/legacy path
/// > knowledge source when `--type knowledge` requested
/// > current exact generation over stale generation unless caller pinned stale
///
/// # Every one of the nine is computed; three of them are structurally
/// uniform, and that is recorded rather than hidden
///
/// [`Self::caller_selected_source`], [`Self::knowledge_source_requested`] and
/// [`Self::current_generation`] are true for **every** candidate of any one
/// answer, or false for every candidate, and therefore never reorder
/// anything. That is not an omission and not a stub — it is what A2-01
/// (*"Filter source/authority/content/Work world before ranking"*) does to
/// them. A preference the admissibility filter has already turned into a
/// **boundary** cannot also be a ranking hint, because nothing on the wrong
/// side of it survives to be ranked:
///
/// * *source explicitly selected by caller* — `SourceSelector::Named`/
///   `Exact`/`WorkBase` are filters, so a hit from an unselected source does
///   not exist to be outranked
///   (`AtlasDb::admissible_generations`'s `WHERE`).
/// * *knowledge source when `--type knowledge` requested* —
///   `Admissibility::kind` is the same: `--type knowledge` admits
///   `local_knowledge` generations and nothing else.
/// * *current exact generation over stale generation* —
///   `AtlasDb::confirm_scan` evicts a source's previous confirmed generation
///   in the same transaction that promotes its successor, and eviction
///   deletes the content rows, so **no stale generation is ever admissible**
///   to begin with. Under `SourceSelector::Exact` — A2 §8's *"unless the
///   caller pinned stale"* — the signal is deliberately set true for every
///   candidate so the preference cannot fire against the pin.
///
/// They are still computed and still carried, because a signal that is
/// uniform *today* is a different thing from one that is missing: it appears
/// in the trace A2 §13 asks for, it discriminates the instant supersession
/// ever keeps two confirmed generations of one source, and its being uniform
/// is a checkable claim rather than a comment
/// (`tests/w4_rrf_fusion.rs::
/// the_three_filter_shaped_signals_are_uniform_because_admissibility_already_applied_them`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RerankSignals {
    /// **1.** A query term is exactly this unit's symbol, heading/title, file
    /// name or file stem — [`exact_match`].
    pub exact_match: bool,
    /// **2.** The query is identifier-like
    /// ([`crate::runtime::atlas::lexical::is_identifier_like`]) and this
    /// candidate is a grammar-claimed definition site rather than a prose
    /// mention of the identifier.
    ///
    /// **The named gap in this signal is what A1 indexes, not what it
    /// stores.** A1 records reference sites in `source.edges`, but
    /// `indexable_units` derives retrievable units from `source.occurrences`
    /// (definition sites) and `source.units`/`context.row_units` (prose and
    /// row text) only — so a code *reference* is not a candidate at all, and
    /// this signal discriminates "the definition" from "a document that
    /// mentions the name". Destination for the fuller reading: indexing
    /// `source.edges` as retrievable units is an A1-side change (a new unit
    /// family), out of scope for W4 and not smuggled in as a ranking
    /// workaround.
    pub definition_over_reference: bool,
    /// **3.** The caller named a source (`--source`, `--source@sha`,
    /// `--work`). Uniform — see the type's own doc.
    pub caller_selected_source: bool,
    /// **4.** The unit's path is one the Work has **changed** — its
    /// overlay-generation content hash differs from the base generation's
    /// hash at the same path (F-SF-01) — not merely one visible under the
    /// overlay's source name, which every unchanged path under it is too.
    /// Reachable since S5 W1d made the overlay reflect in-flight changes
    /// rather than a freshly-cut worktree.
    pub work_changed_unit: bool,
    /// **5.** Same module/package/document section as the top RRF candidate —
    /// [`same_section`].
    pub same_section_as_anchor: bool,
    /// **6.** An inbound or outbound `source.edges` relationship with the top
    /// RRF candidate.
    pub structural_relationship: bool,
    /// **7.** A canonical implementation path rather than a
    /// test/example/legacy one — [`is_canonical_path`].
    pub canonical_path: bool,
    /// **8.** `--type knowledge` was requested. Uniform — see the type's own
    /// doc.
    pub knowledge_source_requested: bool,
    /// **9.** This is the current confirmed generation of its source (or the
    /// caller pinned a generation, in which case it is uniformly true).
    /// Uniform — see the type's own doc.
    pub current_generation: bool,
}

impl RerankSignals {
    /// The rerank key: the nine signals as an array, **in A2 §8's own listing
    /// order**, compared descending (a fired signal outranks an unfired one).
    ///
    /// # Why a lexicographic array and not a score
    ///
    /// A score would need nine numbers, and A2 §14 forbids exposing raw
    /// retrieval weight tuning while A2 §16 forbids anything learned or
    /// self-tuning. There is no coefficient here to expose or to tune:
    /// comparison is `bool` against `bool`, in a fixed order.
    ///
    /// # Why *this* order
    ///
    /// Because it is the contract's, and the contract supplies no other. A2
    /// §8 prints the nine on nine lines; any precedence among them that is
    /// not that order would be invented here, and inventing one silently is
    /// the failure this program keeps producing. `tests/w4_rrf_fusion.rs::
    /// the_rerank_key_is_a2_section_8s_nine_signals_in_the_contracts_own_order`
    /// pins the array against the contract text.
    pub fn priority(&self) -> [bool; 9] {
        [
            self.exact_match,
            self.definition_over_reference,
            self.caller_selected_source,
            self.work_changed_unit,
            self.same_section_as_anchor,
            self.structural_relationship,
            self.canonical_path,
            self.knowledge_source_requested,
            self.current_generation,
        ]
    }

    /// How many of the nine fired — for a trace to render, never for
    /// ordering. [`Self::priority`] is the order; a count would be a weight
    /// system with every weight silently set to 1.
    pub fn fired(&self) -> usize {
        self.priority().iter().filter(|fired| **fired).count()
    }
}

/// One fused candidate: A2 §7's score, the ranks it was computed from, the
/// A2 §8 signals that reranked it, and the A1 coordinate every hit in this
/// system carries.
#[derive(Debug, Clone, PartialEq)]
pub struct FusedHit {
    /// `Σ 1/(k + rank_i(d))` over the lists this candidate appeared in.
    pub rrf: f64,
    /// Which lists it appeared in, and where — A2 §13's *"result evidence IDs
    /// + ranks"*.
    pub origins: RankOrigins,
    /// A2 §8's nine signals for this candidate. All false until
    /// `AtlasDb::fused_search` computes them; [`fuse`] itself does not, and
    /// cannot — six of the nine need the query, the filter or the store.
    pub signals: RerankSignals,
    /// The declared source.
    pub source_name: String,
    /// **A2 §17 item 8** — the source's kind, carried through the fusion
    /// unchanged from whichever input list the candidate arrived in. Both
    /// inputs carry it, and both agree: `fuse` joins on
    /// `(generation_id, unit_key)`, and a generation has exactly one
    /// `source_kind` row in `source.generations`.
    pub source_kind: SourceKind,
    /// **A2 §17 item 8's other half** — the source's authority class. See
    /// [`crate::runtime::atlas::lexical::LexicalHit::authority_class`].
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

impl FusedHit {
    /// **The same stated tie-break key both input lists use** —
    /// `(source_name, relative_path, ordinal, unit_key)`, see
    /// [`crate::runtime::atlas::lexical::LexicalHit::tie_break_key`] and
    /// [`crate::runtime::atlas::semantic::SemanticHit::tie_break_key`].
    ///
    /// Same key at all three levels on purpose: two lists that broke ties
    /// differently would already have handed `fuse` a `rank_i(d)` that
    /// depended on which half a tied unit reached first, and a fused list
    /// that broke them a third way would undo the agreement.
    pub fn tie_break_key(&self) -> (&str, &str, u64, &str) {
        (
            &self.source_name,
            self.coordinate.relative_path(),
            self.coordinate.ordinal(),
            &self.unit_key,
        )
    }
}

/// Order two candidates by **A2 §7's fused score alone**: score descending by
/// [`f64::total_cmp`], then [`FusedHit::tie_break_key`] ascending.
///
/// This is the order A2 §7 produces and A2 §8's *"then deterministic
/// reranking"* consumes — [`rerank`] is the second step, and the two are
/// separate functions so a reader (and a test) can see the fused order before
/// any signal touched it.
pub fn rrf_order(left: &FusedHit, right: &FusedHit) -> std::cmp::Ordering {
    right
        .rrf
        .total_cmp(&left.rrf)
        .then_with(|| left.tie_break_key().cmp(&right.tie_break_key()))
}

/// A2 §8's order: **signals first (descending, [`RerankSignals::priority`]),
/// then the fused score, then the stated tie-break key.**
///
/// The RRF score is not discarded — it decides every pair whose signals are
/// identical, which is the overwhelming majority of pairs, and RRF produces
/// exact ties constantly (a candidate at rank 5 of one list and one at rank 5
/// of the other score identically to the last bit), which is exactly what
/// A2 §8's second step is for.
pub fn rerank_order(left: &FusedHit, right: &FusedHit) -> std::cmp::Ordering {
    right
        .signals
        .priority()
        .cmp(&left.signals.priority())
        .then_with(|| rrf_order(left, right))
}

/// A candidate under accumulation. Two `Option`s rather than a `Vec` of
/// contributions: A2 §7 fuses exactly the two lists §7 names, and a `Vec`
/// would be the beginning of the ranker framework A2-08's **R6** says not to
/// build.
struct Accumulator {
    hit: FusedHit,
}

impl Accumulator {
    /// A fresh candidate's skeleton, from the five identifying fields
    /// [`LexicalHit`] and [`SemanticHit`] both carry (F-SI-01: the two call
    /// sites in [`fuse`] differed only in which of `origins.lexical`/
    /// `origins.semantic` they went on to set). `origins` starts at
    /// [`RankOrigins::default`] and is mutated in place by the caller —
    /// there is exactly one copy of it, so there is nothing left to
    /// overwrite at [`fuse`]'s finalize step the way a second, accumulator-
    /// level copy once required (F-SI-02).
    fn new(
        source_name: &str,
        source_kind: SourceKind,
        authority_class: AuthorityClass,
        generation_id: &str,
        content_key: &str,
        unit_key: &str,
        coordinate: &UnitCoordinate,
    ) -> Self {
        Accumulator {
            hit: FusedHit {
                rrf: 0.0,
                origins: RankOrigins::default(),
                signals: RerankSignals::default(),
                source_name: source_name.to_string(),
                source_kind,
                authority_class,
                generation_id: generation_id.to_string(),
                content_key: content_key.to_string(),
                unit_key: unit_key.to_string(),
                coordinate: coordinate.clone(),
            },
        }
    }

    /// A2 §7's expression, **α-blended** — semble's
    /// `search.py`: `alpha_weight * normalized_semantic + (1 - alpha_weight)
    /// * normalized_bm25`, over two lists each RRF'd independently (which is
    /// what `1/(k + rank_i)` per list already is). α comes from
    /// [`resolve_alpha`] and is a `const`-derived value, never a caller knob.
    ///
    /// Hazard 3 — **float summation order.** Two terms, added in one fixed
    /// order (lexical, then semantic), with an absent list contributing a
    /// literal `0.0` rather than being skipped. `a + b` and `b + a` differ in
    /// f64 whenever rounding differs, so the order is written down here once
    /// instead of falling out of whichever list a candidate happened to be
    /// found in first.
    fn total(&self, alpha: f64) -> f64 {
        let lexical = (1.0 - alpha) * self.hit.origins.lexical.map_or(0.0, rrf_contribution);
        let semantic = alpha * self.hit.origins.semantic.map_or(0.0, rrf_contribution);
        lexical + semantic
    }
}

/// A2 §7, the whole of it: fuse a lexical and a semantic rank list into one
/// ordered list of candidates.
///
/// # Determinism, in the order the hazards appear
///
/// 1. **Candidate collection order.** Both inputs are re-sorted with their
///    own stated orders ([`rank_order`], [`rank_semantic`]) before any rank
///    is assigned, so `rank_i(d)` is a function of the hits themselves. A
///    caller that hands over a shuffled list gets the same answer as one that
///    hands over a sorted one.
/// 2. **Tie-breaking.** [`rrf_order`], the same stated key both inputs use.
/// 3. **Float summation order.** [`Accumulator::total`].
/// 4. **`HashMap` iteration.** The accumulator is a [`BTreeMap`] keyed by
///    `(generation_id, unit_key)` — the identity both halves already agree on
///    (`tests/w3b_semantic_retrieval.rs::
///    a_semantic_hit_and_a_lexical_hit_on_the_same_unit_carry_the_identical_coordinate`
///    is why that join is sound).
///
/// # The prohibition, structurally
///
/// This function takes no store handle and performs no lookup. Every hit it
/// returns came from one of its two arguments, both of which were produced
/// inside A2 §2's admissibility filter — so *"the reranker must never
/// silently cross an authority/source filter"* is a property of the
/// signature, not of a check someone has to remember.
///
/// Signals are left at [`RerankSignals::default`] (all false); the caller
/// computes them and calls [`rerank`].
pub fn fuse(lexical: &[LexicalHit], semantic: &[SemanticHit], alpha: f64) -> Vec<FusedHit> {
    let mut lexical: Vec<&LexicalHit> = lexical.iter().collect();
    lexical.sort_by(|a, b| rank_order(a, b));
    let mut semantic: Vec<&SemanticHit> = semantic.iter().collect();
    semantic.sort_by(|a, b| rank_semantic(a, b));

    let mut candidates: BTreeMap<(String, String), Accumulator> = BTreeMap::new();
    for (index, hit) in lexical.iter().enumerate() {
        candidates
            .entry((hit.generation_id.clone(), hit.unit_key.clone()))
            .or_insert_with(|| {
                Accumulator::new(
                    &hit.source_name,
                    hit.source_kind,
                    hit.authority_class,
                    &hit.generation_id,
                    &hit.content_key,
                    &hit.unit_key,
                    &hit.coordinate,
                )
            })
            .hit
            .origins
            .lexical = Some(index + 1);
    }
    for (index, hit) in semantic.iter().enumerate() {
        candidates
            .entry((hit.generation_id.clone(), hit.unit_key.clone()))
            .or_insert_with(|| {
                Accumulator::new(
                    &hit.source_name,
                    hit.source_kind,
                    hit.authority_class,
                    &hit.generation_id,
                    &hit.content_key,
                    &hit.unit_key,
                    &hit.coordinate,
                )
            })
            .hit
            .origins
            .semantic = Some(index + 1);
    }

    let mut fused: Vec<FusedHit> = candidates
        .into_values()
        .map(|accumulator| {
            let rrf = accumulator.total(alpha);
            let mut hit = accumulator.hit;
            hit.rrf = rrf;
            hit
        })
        .collect();
    fused.sort_by(rrf_order);
    fused
}

/// A2 §8's second step, applied to a list [`fuse`] produced and a caller
/// filled the signals of. Sorts by [`rerank_order`].
pub fn rerank(hits: &mut [FusedHit]) {
    hits.sort_by(rerank_order);
}

// ---------------------------------------------------------------------------
// The pure halves of A2 §8's signals — the ones that are a function of the
// query text and the coordinate, with no store access.
// ---------------------------------------------------------------------------

/// **Signal 1**, *"exact symbol / heading / filename match"*: one of the
/// query's distinct terms is exactly this unit's symbol, its heading/title,
/// its file name, or that file name's stem.
///
/// `terms` is [`crate::runtime::atlas::lexical::query_terms`]'s output —
/// already lowercased, distinct and sorted — and every candidate string is
/// lowercased here, so "exact" means exact up to the case-folding the whole
/// retrieval path already does (W2: *"the exact-case spelling survives in the
/// A1 unit the hit cites, which is where an answer's exactness actually
/// lives"*).
///
/// A `RowText` unit's "name" is its row key — A2 §3's structured-text
/// coordinate has no heading and no symbol, and the row id is the only name
/// that coordinate carries.
pub fn exact_match(terms: &[String], coordinate: &UnitCoordinate) -> bool {
    let mut names: Vec<String> = Vec::new();
    let path = coordinate.relative_path();
    if let Some(file_name) = path.rsplit('/').next() {
        names.push(file_name.to_lowercase());
        if let Some((stem, _)) = file_name.split_once('.') {
            names.push(stem.to_lowercase());
        }
    }
    match coordinate {
        UnitCoordinate::Code { symbol, .. } => names.push(symbol.to_lowercase()),
        UnitCoordinate::Document { title, .. } | UnitCoordinate::Mail { title, .. } => {
            if let Some(title) = title {
                names.push(title.to_lowercase());
            }
        }
        UnitCoordinate::RowText { row_key, .. } => names.push(row_key.to_lowercase()),
    }
    names
        .iter()
        .any(|name| !name.is_empty() && terms.iter().any(|term| term == name))
}

/// The path segments and file-name shapes that make a path a
/// test/example/legacy one rather than a canonical implementation — **signal
/// 7**'s whole vocabulary, in one place so a test can read it.
///
/// A directory-segment match, not a substring match: `src/contest/mod.rs`
/// contains "test" and is not a test path. `tests/w4_rrf_fusion.rs::
/// the_canonical_path_vocabulary_matches_segments_not_substrings` is what
/// keeps that true.
pub const NON_CANONICAL_SEGMENTS: [&str; 11] = [
    "test",
    "tests",
    "testing",
    "spec",
    "specs",
    "example",
    "examples",
    "fixtures",
    "benches",
    "legacy",
    "deprecated",
];

/// The file-name shapes that mark a test file living outside any test
/// directory — Rust's `foo_test.rs`, Python's `test_foo.py`, JavaScript's
/// `foo.test.js`/`foo.spec.ts`.
const TEST_FILE_MARKERS: [&str; 4] = ["_test.", "test_", ".test.", ".spec."];

/// **Signal 7**, *"canonical implementation vs test/example/legacy path"*:
/// true for a path that is neither.
///
/// The signal is stated positively (canonical fires, non-canonical does not)
/// because [`RerankSignals::priority`] compares descending — a `true` outranks
/// a `false`, and the contract's preference is *for* the canonical one.
pub fn is_canonical_path(relative_path: &str) -> bool {
    let lowered = relative_path.to_lowercase();
    let mut segments: Vec<&str> = lowered.split('/').collect();
    let file_name = segments.pop().unwrap_or_default();
    if segments
        .iter()
        .any(|segment| NON_CANONICAL_SEGMENTS.contains(segment))
    {
        return false;
    }
    if TEST_FILE_MARKERS.iter().any(|marker| {
        file_name.contains(marker) || file_name.starts_with(marker.trim_end_matches('.'))
    }) {
        return false;
    }
    true
}

/// **Signal 5**, *"same module/package/document section"*, measured against
/// the top RRF candidate (the *anchor*).
///
/// Two readings, because A2 §3 gives code and prose different coordinates:
///
/// * **the same document** — identical `relative_path`. For a document or
///   mail unit that is A2 §3's *"heading-or-slide/section"* neighbourhood:
///   two sections of one file. For a row-text unit it is the same dataset
///   file.
/// * **the same module/package** — identical parent directory. For code that
///   is the module or package neighbourhood A2 §8 names; a directory is the
///   module unit every language in A1's grammar set actually has on disk.
///
/// The anchor trivially satisfies this against itself, which is correct: it
/// is in its own section.
pub fn same_section(anchor: &UnitCoordinate, other: &UnitCoordinate) -> bool {
    let anchor_path = anchor.relative_path();
    let other_path = other.relative_path();
    if anchor_path == other_path {
        return true;
    }
    parent_of(anchor_path) == parent_of(other_path)
}

/// The directory part of a relative path, or `""` for a path at the root.
fn parent_of(relative_path: &str) -> &str {
    match relative_path.rfind('/') {
        Some(cut) => &relative_path[..cut],
        None => "",
    }
}

/// The symbol a coordinate names, when it names one — what
/// `source.edges.target` is matched against for **signal 6**.
pub fn symbol_of(coordinate: &UnitCoordinate) -> Option<&str> {
    match coordinate {
        UnitCoordinate::Code { symbol, .. } => Some(symbol.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(path: &str, symbol: &str) -> UnitCoordinate {
        UnitCoordinate::Code {
            relative_path: path.to_string(),
            language: "rust".to_string(),
            label: "function".to_string(),
            symbol: symbol.to_string(),
            ordinal: 0,
            byte_start: 0,
            byte_end: 1,
        }
    }

    #[test]
    fn the_expression_is_one_over_k_plus_a_one_based_rank() {
        assert_eq!(rrf_contribution(1), 1.0 / (RRF_K + 1.0));
        assert_eq!(rrf_contribution(2), 1.0 / (RRF_K + 2.0));
        assert!(rrf_contribution(1) > rrf_contribution(2));
    }

    #[test]
    fn appearing_in_both_lists_outscores_appearing_in_one_at_the_same_rank() {
        assert!(rrf_contribution(1) + rrf_contribution(1) > rrf_contribution(1));
    }

    #[test]
    fn a_test_directory_or_a_test_file_name_is_not_a_canonical_path() {
        assert!(is_canonical_path("src/runtime/atlas/fusion.rs"));
        assert!(!is_canonical_path("tests/w4_rrf_fusion.rs"));
        assert!(!is_canonical_path("crates/a/examples/demo.rs"));
        assert!(!is_canonical_path("src/legacy/old.rs"));
        assert!(!is_canonical_path("src/parser_test.rs"));
        assert!(!is_canonical_path("src/test_parser.py"));
        assert!(!is_canonical_path("web/app.test.ts"));
        // A segment match, never a substring match.
        assert!(is_canonical_path("src/contest/mod.rs"));
        assert!(is_canonical_path("src/latest.rs"));
    }

    #[test]
    fn same_section_is_the_same_file_or_the_same_directory() {
        let a = code("src/payments/retry.rs", "retry");
        let b = code("src/payments/charge.rs", "charge");
        let c = code("src/config/loader.rs", "load");
        assert!(same_section(&a, &a));
        assert!(same_section(&a, &b));
        assert!(!same_section(&a, &c));
    }

    #[test]
    fn an_exact_symbol_or_file_name_match_fires_and_a_partial_one_does_not() {
        let terms = vec!["retry".to_string()];
        assert!(exact_match(
            &terms,
            &code("src/payments/charge.rs", "retry")
        ));
        assert!(exact_match(&terms, &code("src/payments/retry.rs", "other")));
        assert!(!exact_match(
            &terms,
            &code("src/payments/retrying.rs", "retry_policy")
        ));
    }
}

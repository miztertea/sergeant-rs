//! C1 — compiled actor context: the compilation step (§3) and §5's
//! **enforceable runtime order**.
//!
//! # What this is, in the contract's own words
//!
//! > *"C1 is **not a second execution pipeline**. The existing engine already
//! > binds a workflow/stage and launches a fresh actor. C1 inserts a
//! > deterministic compilation step before the actor start."* (§3)
//!
//! ```text
//! stage about to enter
//!    ↓
//! resolve Work/estate/source generations
//!    ↓
//! run deterministic research plan
//!    ↓
//! compile Bound + Referenced + Reachable snapshot
//!    ↓
//! launch ordinary fresh execution
//! ```
//!
//! So there is no pipeline here. There is one function
//! ([`compile`]) called from one place —
//! [`crate::runtime::engine::Engine::reserve_stage`], between the point where
//! the stage's `CONTEXT.md` is resolved and the point where the adapter is
//! asked to PREPARE. That call site is the existing stage launch path
//! (**R2**, decision C1-01: *"Existing stage launch already owns fresh actor
//! context/execution"*), and the mechanism that appends to a stage's context
//! is the one Amendment 9 Q5 / #260 already built for the branch-status fact
//! ([`crate::runtime::engine`]'s `branch_status_context`) — reused, not
//! re-invented.
//!
//! # §5's nine steps are an ordered runtime sequence, not a listing
//!
//! > *"Before fuzzy retrieval or model dispatch, use the strongest local
//! > evidence operations available:"*
//! >
//! > ```text
//! > 1. explicit stage inputs / prior declared artifacts
//! > 2. exact Work bindings/changed resources
//! > 3. exact structural/document/tabular/mail relationships
//! > 4. deterministic dataset aggregates/joins/diffs declared by the
//! >    profile/workflow
//! > 5. exact Referenced neighbors
//! > 6. A2 lexical retrieval
//! > 7. A2 semantic retrieval if installed/needed
//! > 8. bounded structural/provenance expansion
//! > 9. pack Bound; emit useful remainder as Referenced
//! > ```
//! >
//! > *"This is where **spend computation before cognition** becomes
//! > enforceable runtime order."* (§5)
//!
//! **Enforceable** is implemented by [`ResearchLedger`], and it is a gate, not
//! a convention. Every step must be *entered* through
//! [`ResearchLedger::enter`], which refuses any step that is not §5's next
//! one. A step with nothing to do is entered and records zero units with a
//! stated reason — degradation stays *visible* (§18), and skipping stays
//! unrepresentable, which is what keeps the order total.
//!
//! The refusal has two names because the two failures are different mistakes:
//! [`OrderViolation::OutOfOrder`] is a sequencing bug, and
//! [`OrderViolation::CognitionBeforeComputation`] is the one §5's sentence is
//! about — a *fuzzy* step (6, 7) entered while a *deterministic* step it is
//! supposed to follow has not run. Same gate, two diagnoses; the fuzzy one is
//! reported whenever it applies, because that is the sentence a reader of the
//! failure needs.
//!
//! # Why a gate alone would not be enough, and what else proves the order
//!
//! A gate proves the steps *ran* in order. It cannot prove the order *mattered
//! to the answer* — a compiler that ran them in order and then let a late step
//! overwrite an early one would pass it. So the ledger also enforces
//! **first-contributor-wins**: an evidence coordinate contributed by an
//! earlier step is not re-contributed by a later one
//! ([`StepWriter::contribute`] answers `false`). A resource that is both an
//! exact Work-changed resource (step 2) and the top lexical hit (step 6) is
//! attributed to **step 2**, and the snapshot says so. Run retrieval first and
//! the same resource would carry `step: 6` — an *observable state difference*,
//! not a matter of reasoning, which is what
//! `tests/c1a_compiled_context.rs` asserts.
//!
//! # Steps 6 and 7 are consumed, never reimplemented
//!
//! S5 shipped the retrieval pipeline: the deterministic filter, BM25,
//! exact-cosine semantic retrieval, RRF + rerank, and A2 §13's trace.
//! [`AtlasDb::traced_search`] is steps 6 and 7 (**R2**) — it runs
//! `lexical_search` and then `semantic_search` inside one snapshot-isolated
//! transaction, so §5's own 6-then-7 ordering is that function's, already
//! pinned by S5's suites. This module calls it once and reads which halves
//! actually contributed off the answer's own [`SemanticStatus`], which is how
//! *"semantic retrieval **if installed/needed**"* stays honest rather than
//! assumed.
//!
//! Its [`SearchTrace`] is retained on the snapshot. That closes the
//! destination [`crate::runtime::atlas::trace`] named in S5: *"persisting the
//! trace of a **managed** search — one a Work's own execution issued — is
//! C1/S6's, where a managed retrieval call has a Work execution to attach the
//! row to. Named destination: the C1 compiled-context wave."*
//!
//! # No new SQL, and that is deliberate
//!
//! Every read this module needs already exists on [`AtlasDb`]:
//! `admissible_generations`, `confirmed_generation`, `units`,
//! `child_resources`, `edges`, `coverage`, `traced_search`. S5's closeout
//! rebuilt the SQL boundary over three rounds — statement text is an
//! associated `const` (`store::SqlText`), `Sql::of::<T>()` takes no string,
//! and reads go through `ReadOnly` + `read_sql!`. This module adds no
//! statement to that surface at all, so there is nothing here for a runtime
//! string to get into (**R2/R3**).
//!
//! # §14's two budgets and the three tiers' resolution (C1b)
//!
//! §5 step 9 — *"pack Bound; emit useful remainder as Referenced"* — is where
//! [`RenderBudget`] is spent ([`pack`]). Two budgets, not one, because §14
//! says *"Referenced coordinates have a **small separate** rendering budget"*;
//! both are **hard**, so a source bigger than the Bound budget becomes
//! Referenced remainder rather than a truncated body; and neither is visible
//! to [`resolve`], because §14's last sentence says the budget *"is not a ban
//! on the actor resolving Reachable/Referenced evidence when needed"*.
//! [`resolve`] is a keyed lookup on the coordinate's own row identity —
//! §21 item 4's *"without broad rediscovery"* — and it is the same call
//! packing uses to fetch what it renders, so every rendered Bound unit
//! resolves back to the stored row it came from by construction (item 3).
//!
//! # What this wave does NOT do
//!
//! Authority, provenance and structured query results are C1c (items 6–9);
//! attribution, nesting and audit are C1d (items 11, 12, 14). Fields §15 asks
//! for that those waves fill are present and empty, each naming its wave —
//! never absent, because an absent field is indistinguishable from a
//! forgotten one.
//!
//! §20's non-goals this step is closest to are named and refused: **no raw
//! corpus stuffing** (a Bound body renders only inside §14's hard budget, and
//! a resource too large for it is never truncated into the prompt — it
//! becomes Referenced remainder), **no automatic Work scope
//! expansion** (§4: *"The compiler does not silently add repos to Work
//! mutation scope because a search result is relevant"* — nothing here
//! touches [`crate::runtime::surface::WorkSurface`] or a binding), **no
//! learned context policy or live self-tuning** (every bound in this file is a
//! constant a human wrote), and **no automatic workflow→script conversion**
//! (the stage's authored `CONTEXT.md` is appended to, never rewritten).

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Value, json};

use crate::backend::BindingSummary;
use crate::domain::source::{AuthorityClass, SourceGeneration};
use crate::domain::workflow::{StageDefinition, StageRecord};
use crate::runtime::atlas::db::{
    Admissibility, AtlasDb, AtlasError, LexicalQuery, SourceSelector, StoredChildResource,
    StoredEdge, StoredUnit,
};
use crate::runtime::atlas::overlay::overlay_source_name;
use crate::runtime::atlas::semantic::{SemanticRequest, SemanticStatus};
use crate::runtime::atlas::trace::{Attribution, SearchTrace};

// ===================================================================
// §5's nine steps
// ===================================================================

/// *"spend computation before cognition"* (§5) — which side of that sentence
/// a step is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cognition {
    /// A local evidence operation: exact, cheap, and answerable without a
    /// ranker or a model.
    Computation,
    /// Fuzzy retrieval or model dispatch — §5's steps 6 and 7.
    Cognition,
}

impl Cognition {
    /// The word a trace, a journal payload or an error message uses.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Computation => "deterministic",
            Self::Cognition => "fuzzy",
        }
    }
}

/// §5's nine steps, **in the contract's own order**, one variant per line of
/// its list.
///
/// The declaration order *is* the runtime order — [`ResearchLedger::enter`]
/// reads [`Self::PLAN`], and `tests/c1a_compiled_context.rs::
/// the_nine_steps_are_section_5s_own_list_in_section_5s_own_order` compares
/// that array against the contract's nine lines. Reordering the enum without
/// reordering §5 turns that test red.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResearchStep {
    /// 1. explicit stage inputs / prior declared artifacts
    StageInputs,
    /// 2. exact Work bindings/changed resources
    WorkBindings,
    /// 3. exact structural/document/tabular/mail relationships
    ExactRelationships,
    /// 4. deterministic dataset aggregates/joins/diffs declared by the
    ///    profile/workflow
    DeclaredDataOperations,
    /// 5. exact Referenced neighbors
    ReferencedNeighbors,
    /// 6. A2 lexical retrieval
    LexicalRetrieval,
    /// 7. A2 semantic retrieval if installed/needed
    SemanticRetrieval,
    /// 8. bounded structural/provenance expansion
    BoundedExpansion,
    /// 9. pack Bound; emit useful remainder as Referenced
    Pack,
}

impl ResearchStep {
    /// §5's list, in §5's order. The single source of the runtime sequence.
    pub const PLAN: [ResearchStep; 9] = [
        Self::StageInputs,
        Self::WorkBindings,
        Self::ExactRelationships,
        Self::DeclaredDataOperations,
        Self::ReferencedNeighbors,
        Self::LexicalRetrieval,
        Self::SemanticRetrieval,
        Self::BoundedExpansion,
        Self::Pack,
    ];

    /// §5's own 1-based numbering.
    pub fn number(self) -> usize {
        Self::PLAN
            .iter()
            .position(|s| *s == self)
            .expect("every step is in PLAN")
            + 1
    }

    /// §5's own wording for this step.
    pub fn label(self) -> &'static str {
        match self {
            Self::StageInputs => "explicit stage inputs / prior declared artifacts",
            Self::WorkBindings => "exact Work bindings/changed resources",
            Self::ExactRelationships => "exact structural/document/tabular/mail relationships",
            Self::DeclaredDataOperations => {
                "deterministic dataset aggregates/joins/diffs declared by the profile/workflow"
            }
            Self::ReferencedNeighbors => "exact Referenced neighbors",
            Self::LexicalRetrieval => "A2 lexical retrieval",
            Self::SemanticRetrieval => "A2 semantic retrieval if installed/needed",
            Self::BoundedExpansion => "bounded structural/provenance expansion",
            Self::Pack => "pack Bound; emit useful remainder as Referenced",
        }
    }

    /// Which side of *"spend computation before cognition"* this step is on.
    ///
    /// Exactly steps 6 and 7 are [`Cognition::Cognition`], because exactly
    /// those two are §5's *"fuzzy retrieval"*. Step 8 is deterministic even
    /// though it runs after them — it walks `source.edges`, which is stored
    /// structure, not a ranker — and §5 puts it there on purpose: it expands
    /// from what retrieval found.
    pub fn cognition(self) -> Cognition {
        match self {
            Self::LexicalRetrieval | Self::SemanticRetrieval => Cognition::Cognition,
            _ => Cognition::Computation,
        }
    }
}

/// The gate's refusal — §5's order violated.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OrderViolation {
    /// A step was entered that is not §5's next one.
    #[error(
        "§5 step {attempted} ({attempted_label:?}) was entered while step {expected} \
         ({expected_label:?}) had not run: the deterministic research plan is an ordered \
         runtime sequence, not a listing"
    )]
    OutOfOrder {
        /// §5's number for the step that was attempted.
        attempted: usize,
        /// §5's wording for it.
        attempted_label: &'static str,
        /// §5's number for the step that should have run.
        expected: usize,
        /// §5's wording for it.
        expected_label: &'static str,
    },
    /// The refusal §5's own sentence is about: fuzzy before deterministic.
    #[error(
        "§5's fuzzy step {fuzzy} ({fuzzy_label:?}) was entered before deterministic step \
         {deterministic} ({deterministic_label:?}) ran: \"spend computation before cognition\" \
         is enforceable runtime order, not advice about sequencing"
    )]
    CognitionBeforeComputation {
        /// §5's number for the fuzzy step that jumped ahead.
        fuzzy: usize,
        /// §5's wording for it.
        fuzzy_label: &'static str,
        /// §5's number for the deterministic step it jumped over.
        deterministic: usize,
        /// §5's wording for it.
        deterministic_label: &'static str,
    },
}

// ===================================================================
// Evidence: coordinates and tiers (§2)
// ===================================================================

/// §2's tiers, as they apply to one unit of evidence.
///
/// A `Reachable` variant is deliberately **not** here: §2 defines it as *"the
/// broader admissible Atlas map/search/query **surface**"*, which is a
/// descriptor of what remains available ([`ReachableScope`]), not a list of
/// units. A `Reachable` variant would invite a wave to enumerate it, which is
/// the raw-corpus-stuffing non-goal wearing a tier's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// *"Evidence deliberately rendered into the actor's initial context
    /// because the stage needs it now."*
    Bound,
    /// *"Known-relevant exact coordinates not rendered in full."*
    Referenced,
}

impl Tier {
    /// The word the snapshot and the rendered section use.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bound => "BOUND",
            Self::Referenced => "REFERENCED",
        }
    }
}

/// Where one piece of evidence actually is — four shapes, because the four
/// things §5's steps produce are genuinely different coordinates and a single
/// flattened shape would have to invent fields for three of them (the same
/// argument [`crate::runtime::atlas::trace::coordinate_json`] makes about A2
/// §3's families).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceCoordinate {
    /// §5 step 1: this run's own stage record — an explicit stage input or a
    /// prior declared artifact.
    Stage {
        /// The stage's id.
        stage_id: String,
        /// Its position in the workflow's stage order.
        index: usize,
        /// Which attempt.
        attempt: u32,
        /// Its status, as the projection folded it.
        status: String,
        /// The summary/reason that status change carried, when there was one.
        summary: Option<String>,
    },
    /// §5 step 2: one of the Work's exact repository bindings.
    Binding {
        /// The repository this Work is bound to.
        repository: String,
        /// The durable output branch.
        work_branch: String,
        /// §15's *"Work base"* for this binding.
        base_sha: String,
    },
    /// An exact Atlas resource or unit, with its generation pinned.
    Atlas {
        /// The declared source.
        source_name: String,
        /// The exact generation.
        generation_id: String,
        /// That generation's content identity.
        content_key: String,
        /// Path relative to the source root.
        relative_path: String,
        /// The index's own per-generation unit identity, when the coordinate
        /// names a unit rather than a whole resource.
        unit_key: Option<String>,
        /// Position within the resource, when there is one.
        ordinal: Option<u64>,
    },
    /// An exact stored relationship — §5 step 3's *"structural/document/
    /// tabular/mail relationships"* and step 5/8's edges.
    Relationship {
        /// The declared source both ends belong to.
        source_name: String,
        /// The exact generation.
        generation_id: String,
        /// What kind of relationship (`child_resource`, `import`, …).
        kind: String,
        /// The end it leaves from.
        from: String,
        /// The end it names. **Unresolved** for a syntax edge, exactly as
        /// [`StoredEdge::target`] is.
        to: String,
        /// Position within `from`'s edge list (F-IN-01) — an edge's own
        /// per-row identity alongside `kind`/`from`/`to`, since two distinct
        /// edges (e.g. the same file importing the same target twice)
        /// legitimately share every other field. `None` for a child-resource
        /// row, which has no ordinal, exactly as
        /// [`crate::runtime::atlas::db::ResolvedRelationship::ordinal`] is.
        ordinal: Option<u64>,
    },
}

impl EvidenceCoordinate {
    /// The identity first-contributor-wins is keyed on.
    ///
    /// Two steps that would contribute *the same evidence* must collide here,
    /// or the ordering the ledger enforces would not be visible in the
    /// snapshot at all — the whole point of first-contributor-wins is that a
    /// deterministic step and a fuzzy one reaching for the same thing is a
    /// *contest*, and a contest needs both entrants in the same ring.
    ///
    /// So an Atlas coordinate is keyed at **resource** granularity —
    /// `(generation_id, relative_path)` — not at unit granularity. The two
    /// steps that contest a resource address it differently by construction:
    /// step 2 reads `source.units` and has an ordinal but no index unit key,
    /// while steps 6/7 return `context.lexical_units` hits that have one. A
    /// unit-keyed identity would let those two *never collide*, which would
    /// make the ordering unobservable and this whole mechanism decorative.
    ///
    /// The cost is stated rather than hidden: a resource bound by an early
    /// step absorbs every later hit inside it, so this binds resources, not
    /// spans. C1b kept it that way deliberately: the contest above only
    /// exists because both entrants address the same key, and a span-keyed
    /// identity would make §5's order unobservable again. What C1b's budget
    /// changed is what a *bound* resource costs — a unit that does not fit
    /// becomes Referenced remainder — not what counts as the same evidence.
    pub fn dedup_key(&self) -> String {
        match self {
            Self::Stage {
                stage_id,
                index,
                attempt,
                ..
            } => format!("stage/{index}/{stage_id}/{attempt}"),
            Self::Binding { repository, .. } => format!("binding/{repository}"),
            Self::Atlas {
                generation_id,
                relative_path,
                ..
            } => format!("atlas/{generation_id}/{relative_path}"),
            Self::Relationship {
                generation_id,
                kind,
                from,
                to,
                ..
            } => format!("rel/{generation_id}/{kind}/{from}->{to}"),
        }
    }

    /// One line, for the rendered section and for a human reading the journal.
    pub fn render(&self) -> String {
        match self {
            Self::Stage {
                stage_id,
                index,
                attempt,
                status,
                summary,
            } => match summary {
                Some(summary) => {
                    format!("stage {index}/{stage_id} attempt {attempt} [{status}] — {summary}")
                }
                None => format!("stage {index}/{stage_id} attempt {attempt} [{status}]"),
            },
            Self::Binding {
                repository,
                work_branch,
                base_sha,
            } => format!("binding {repository} — branch {work_branch}, base {base_sha}"),
            Self::Atlas {
                source_name,
                content_key,
                relative_path,
                unit_key,
                ordinal,
                ..
            } => {
                let mut line = format!("{source_name}@{content_key}:{relative_path}");
                if let Some(ordinal) = ordinal {
                    line.push_str(&format!("#{ordinal}"));
                }
                if let Some(key) = unit_key {
                    line.push_str(&format!(" [{key}]"));
                }
                line
            }
            Self::Relationship {
                source_name,
                kind,
                from,
                to,
                ..
            } => format!("{source_name}:{from} —{kind}→ {to}"),
        }
    }

    /// The coordinate as JSON, one shape per variant.
    pub fn json(&self) -> Value {
        match self {
            Self::Stage {
                stage_id,
                index,
                attempt,
                status,
                summary,
            } => json!({
                "shape": "stage",
                "stage_id": stage_id,
                "index": index,
                "attempt": attempt,
                "status": status,
                "summary": summary,
            }),
            Self::Binding {
                repository,
                work_branch,
                base_sha,
            } => json!({
                "shape": "binding",
                "repository": repository,
                "work_branch": work_branch,
                "base_sha": base_sha,
            }),
            Self::Atlas {
                source_name,
                generation_id,
                content_key,
                relative_path,
                unit_key,
                ordinal,
            } => json!({
                "shape": "atlas",
                "source": source_name,
                "generation_id": generation_id,
                "content_key": content_key,
                "relative_path": relative_path,
                "unit_key": unit_key,
                "ordinal": ordinal,
            }),
            Self::Relationship {
                source_name,
                generation_id,
                kind,
                from,
                to,
                ordinal,
            } => json!({
                "shape": "relationship",
                "source": source_name,
                "generation_id": generation_id,
                "kind": kind,
                "from": from,
                "to": to,
                "ordinal": ordinal,
            }),
        }
    }
}

/// One unit of compiled evidence: what it is, which tier it landed in, and
/// **which §5 step contributed it**.
///
/// The step is not decoration. It is the field that makes §5's order visible
/// in the snapshot rather than only in the code that produced it — see this
/// module's doc on first-contributor-wins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceUnit {
    /// The §5 step that contributed it, first-contributor-wins.
    pub step: ResearchStep,
    /// §2's tier it landed in — as **packing** left it (§5 step 9), which is
    /// not always the tier the contributing step asked for: Bound evidence
    /// that did not fit §14's hard budget is *"useful remainder"* and is
    /// emitted as Referenced.
    pub tier: Tier,
    /// Where the evidence actually is.
    pub coordinate: EvidenceCoordinate,
    /// The unit's own source text, when §14's Bound budget had room to
    /// render it — §2's Bound tier is *"evidence deliberately **rendered
    /// into** the actor's initial context"*, which a coordinate alone is not.
    ///
    /// `None` is not "no text exists": it is *this unit is carried as a
    /// pointer*, either because the budget was spent, because the coordinate
    /// is a Work record rather than a stored resource, or because the
    /// evidence is external (see [`pack`]). The text is still resolvable —
    /// §14: the budget *"is not a ban on the actor resolving Reachable/
    /// Referenced evidence when needed"*.
    pub excerpt: Option<String>,
    /// Whether the **automatic** render carried this unit at all. A unit past
    /// both budgets stays in the snapshot (§15's *"Bound/Referenced evidence
    /// IDs"*) and stays resolvable; it simply is not spent on the prompt.
    pub rendered: bool,
}

impl EvidenceUnit {
    /// The unit as JSON, step and tier included.
    pub fn json(&self) -> Value {
        json!({
            "step": self.step.number(),
            "step_label": self.step.label(),
            "cognition": self.step.cognition().as_str(),
            "tier": self.tier.as_str(),
            "coordinate": self.coordinate.json(),
            "rendered": self.rendered,
            "excerpt_bytes": self.excerpt.as_ref().map(|e| e.len()),
        })
    }
}

// ===================================================================
// The gate
// ===================================================================

/// What one entered step did — recorded whether or not it contributed
/// anything, because §18's degradation must be *visible, not fatal or
/// fabricated*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRecord {
    /// Which step.
    pub step: ResearchStep,
    /// How many evidence units it contributed (after first-contributor-wins).
    pub contributed: usize,
    /// How many it offered that an earlier step had already contributed.
    pub already_held: usize,
    /// Why it contributed nothing, when it contributed nothing. `None` when
    /// it contributed something.
    pub note: Option<String>,
}

impl StepRecord {
    /// The record as JSON.
    pub fn json(&self) -> Value {
        json!({
            "step": self.step.number(),
            "label": self.step.label(),
            "cognition": self.step.cognition().as_str(),
            "contributed": self.contributed,
            "already_held": self.already_held,
            "note": self.note,
        })
    }
}

/// **§5's order, enforced.**
///
/// The ledger is the only way to add evidence to a compilation, and the only
/// way to open a step is [`Self::enter`], which refuses anything but §5's next
/// step. There is no bypass: [`StepWriter`] borrows the ledger mutably for as
/// long as the step is open, so a second step cannot be opened while one is,
/// and a step cannot be re-entered once closed.
#[derive(Debug, Default)]
pub struct ResearchLedger {
    /// Steps entered, in the order they were entered.
    executed: Vec<StepRecord>,
    /// Evidence contributed, in contribution order.
    units: Vec<EvidenceUnit>,
    /// Every dedup key already contributed — first-contributor-wins.
    held: BTreeSet<String>,
}

impl ResearchLedger {
    /// A ledger with no step run.
    pub fn new() -> Self {
        Self::default()
    }

    /// §5's next step — the only one [`Self::enter`] will accept.
    pub fn expected(&self) -> Option<ResearchStep> {
        ResearchStep::PLAN.get(self.executed.len()).copied()
    }

    /// Open `step`, or refuse.
    ///
    /// Refuses with [`OrderViolation::CognitionBeforeComputation`] whenever
    /// the attempted step is fuzzy and a deterministic step it is supposed to
    /// follow has not run — §5's own sentence, reported as itself — and with
    /// [`OrderViolation::OutOfOrder`] for every other mis-sequencing.
    pub fn enter(&mut self, step: ResearchStep) -> Result<StepWriter<'_>, OrderViolation> {
        let expected = self.expected().ok_or(OrderViolation::OutOfOrder {
            attempted: step.number(),
            attempted_label: step.label(),
            expected: 0,
            expected_label: "the plan is already complete",
        })?;
        if step != expected {
            // The fuzzy diagnosis first: it is the one §5's sentence names,
            // and a reader of the failure needs *that* sentence, not the
            // generic one. Applies whenever the step jumping ahead is fuzzy
            // and something deterministic below it has not run.
            if step.cognition() == Cognition::Cognition {
                let skipped = ResearchStep::PLAN
                    .iter()
                    .take(step.number() - 1)
                    .find(|s| {
                        s.cognition() == Cognition::Computation
                            && !self.executed.iter().any(|r| r.step == **s)
                    })
                    .copied();
                if let Some(deterministic) = skipped {
                    return Err(OrderViolation::CognitionBeforeComputation {
                        fuzzy: step.number(),
                        fuzzy_label: step.label(),
                        deterministic: deterministic.number(),
                        deterministic_label: deterministic.label(),
                    });
                }
            }
            return Err(OrderViolation::OutOfOrder {
                attempted: step.number(),
                attempted_label: step.label(),
                expected: expected.number(),
                expected_label: expected.label(),
            });
        }
        Ok(StepWriter {
            ledger: self,
            step,
            contributed: 0,
            already_held: 0,
            note: None,
        })
    }

    /// Every step entered, in the order it was entered.
    pub fn executed(&self) -> &[StepRecord] {
        &self.executed
    }

    /// Every evidence unit, in contribution order.
    pub fn units(&self) -> &[EvidenceUnit] {
        &self.units
    }

    /// Whether all nine of §5's steps ran.
    pub fn complete(&self) -> bool {
        self.executed.len() == ResearchStep::PLAN.len()
    }
}

/// One open §5 step. Dropping it closes the step and records what it did.
#[derive(Debug)]
pub struct StepWriter<'a> {
    ledger: &'a mut ResearchLedger,
    step: ResearchStep,
    contributed: usize,
    already_held: usize,
    note: Option<String>,
}

impl StepWriter<'_> {
    /// Offer one coordinate at one tier.
    ///
    /// Answers `false` when an **earlier** step already contributed this
    /// coordinate — first-contributor-wins, which is what makes §5's order
    /// observable in the compiled snapshot rather than only in the call order.
    pub fn contribute(&mut self, tier: Tier, coordinate: EvidenceCoordinate) -> bool {
        let key = coordinate.dedup_key();
        if !self.ledger.held.insert(key) {
            self.already_held += 1;
            return false;
        }
        self.ledger.units.push(EvidenceUnit {
            step: self.step,
            tier,
            coordinate,
            // Packing (§5 step 9) decides both of these, once the whole plan
            // has run and §14's budget can be spent on the evidence that
            // actually exists. A contributing step never renders.
            excerpt: None,
            rendered: false,
        });
        self.contributed += 1;
        true
    }

    /// State why this step contributed nothing — §18's *visible* degradation.
    pub fn note(&mut self, note: impl Into<String>) {
        self.note = Some(note.into());
    }
}

impl Drop for StepWriter<'_> {
    fn drop(&mut self) {
        self.ledger.executed.push(StepRecord {
            step: self.step,
            contributed: self.contributed,
            already_held: self.already_held,
            note: self.note.take(),
        });
    }
}

// ===================================================================
// The snapshot (§15)
// ===================================================================

/// §2's third tier: *"the broader admissible Atlas map/search/query surface
/// available on demand"* — a **descriptor**, never an enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachableScope {
    /// The A2 §2 stage-1 selector that bounds it, rendered.
    pub selector: String,
    /// The source name it names, when it names one.
    pub source_name: Option<String>,
    /// The Work it is scoped to, when it is Work-scoped.
    pub work_id: Option<String>,
    /// The managed verbs this surface offers.
    pub capabilities: Vec<&'static str>,
}

impl ReachableScope {
    /// The descriptor as JSON.
    pub fn json(&self) -> Value {
        json!({
            "selector": self.selector,
            "source": self.source_name,
            "work": self.work_id,
            "capabilities": self.capabilities,
        })
    }
}

/// Why a compilation produced no compiled world — §18's degradation ladder,
/// stated rather than inferred from an empty snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Degradation {
    /// §18's first rung: *"intelligence disabled → existing stage CONTEXT +
    /// Work bindings"*. No Atlas handle was installed on this engine at all.
    IntelligenceDisabled,
    /// An Atlas handle exists but this host has confirmed no generation for
    /// this Work's world — nothing has been indexed. Item 13's other half:
    /// unavailable, not merely off.
    NoConfirmedGeneration,
    /// An Atlas read failed. The launch is not failed for it; the compilation
    /// is, and says so.
    AtlasUnavailable(String),
}

impl Degradation {
    /// The word the journal payload uses.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IntelligenceDisabled => "intelligence_disabled",
            Self::NoConfirmedGeneration => "no_confirmed_generation",
            Self::AtlasUnavailable(_) => "atlas_unavailable",
        }
    }

    /// The detail, when there is one.
    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::AtlasUnavailable(detail) => Some(detail),
            _ => None,
        }
    }
}

/// **§15's snapshot provenance** — *"Every fresh execution can answer 'what
/// world did Sergeant present?'"*
///
/// One field per line of §15's list, in §15's own order. That discipline is
/// [`crate::runtime::atlas::trace::SearchTrace`]'s, applied for the same
/// reason: a listing order the contract supplies is the only ordering that is
/// not invented here, and a field per line is what makes an omission visible.
/// Lines a later wave fills are present and empty, each naming its wave.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextSnapshot {
    /// **1.** `context_snapshot_id`.
    pub snapshot_id: String,
    /// **2.** `estate/work/stage/execution`.
    pub coordinate: SnapshotCoordinate,
    /// **3.** `journal watermark` — the journal's next sequence number at the
    /// instant the compilation ran, so the snapshot names the exact prefix of
    /// the journal it could have seen.
    pub journal_watermark: u64,
    /// **4.** `Work base + overlay generation`.
    pub work_world: WorkWorld,
    /// **5.** `source generations (estate/local knowledge/external)` — the
    /// exact generations the admissibility filter admitted, pinned by id and
    /// content key.
    pub source_generations: Vec<GenerationPin>,
    /// **6.** `coverage states` — per admitted source, the coverage statuses
    /// its generation carries and how many rows each has.
    pub coverage: Vec<CoverageState>,
    /// **7.** `structured query-result IDs`. **Empty this wave**: §10's
    /// structured query results are C1c's (item 7), which is the wave that
    /// binds a compact result and references its underlying rows.
    pub query_result_ids: Vec<String>,
    /// **8.** `retrieval generation/model if used` — A2 §13's full trace, the
    /// managed-search persistence `crate::runtime::atlas::trace` named C1 as
    /// the destination for. `None` when steps 6/7 ran no search.
    pub retrieval: Option<SearchTrace>,
    /// **9.** `profile + version` — §6's context profile. `None` until §6's
    /// `[context] profile` grammar exists; this wave carries the stage's
    /// **launch** profile name where the run has one, and never invents a
    /// context profile that no workflow declared.
    pub profile: Option<String>,
    /// **10.** `selection-plan hash` — a content hash over §5's executed step
    /// records and every contributed coordinate, in order. Two compilations
    /// that selected the same evidence by the same plan hash the same; one
    /// that ran the steps in a different order does not.
    pub selection_plan_hash: String,
    /// **11.** `Bound evidence IDs`.
    pub bound: Vec<EvidenceUnit>,
    /// **12.** `Referenced evidence IDs`.
    pub referenced: Vec<EvidenceUnit>,
    /// **13.** `Reachable capability/scope descriptor`.
    pub reachable: Option<ReachableScope>,
    /// **14.** `rendered payload blob/artifact pointer`. **`None`, and now
    /// for a reason rather than a deferral**: decision C1-09's blob seam is
    /// for *"large snapshot payloads"*, and §14's hard budgets
    /// ([`RenderBudget`]) cap the whole rendered payload at ten kilobytes,
    /// which is small enough to live inline in the stage's `CONTEXT.md`. A
    /// pointer would be a second place to look for something that is already
    /// in front of the reader. A wave that renders something the budget does
    /// not bound is the wave that needs this field.
    pub payload_pointer: Option<String>,
    /// **15.** `budget` — §14's two hard automatic-render budgets and what
    /// each tier actually spent ([`BudgetReport`]). `None` only for a
    /// degraded compilation, which rendered nothing to budget.
    pub budget: Option<BudgetReport>,
    /// **16.** `rendered size` — how many bytes this snapshot appended to the
    /// stage's `CONTEXT.md` in total, evidence and frame together. Always at
    /// least [`BudgetReport::bound_spent`] + [`BudgetReport::referenced_spent`]
    /// and never much more: the difference is the section heading, the
    /// snapshot id and §2's Reachable descriptor, which are not evidence and
    /// are outside both budgets — item 5 requires Reachable to survive an
    /// exhausted budget, and a descriptor inside the budget it must survive
    /// could not.
    pub rendered_bytes: u64,
    /// §5's executed steps, in the order the ledger let them run.
    pub plan: Vec<StepRecord>,
    /// Why nothing was compiled, when nothing was — §18.
    pub degradation: Option<Degradation>,
}

/// §15's *"estate/work/stage/execution"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotCoordinate {
    /// The Work's own canonical estate root, when it recorded one.
    pub estate_root: Option<String>,
    /// The Work.
    pub work_id: String,
    /// The stage.
    pub stage_id: String,
    /// Its position in the workflow's stage order.
    pub stage_index: usize,
    /// Which attempt this snapshot was compiled for.
    pub attempt: u32,
    /// The execution the reservation allocated — the fresh actor this world
    /// is being compiled for.
    pub execution_id: String,
}

/// §15's *"Work base + overlay generation"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkWorld {
    /// The repository the compilation scoped to, and its admission-pinned
    /// base SHA.
    pub repository: Option<String>,
    /// That binding's `base_sha`.
    pub base_sha: Option<String>,
    /// The Work's own overlay generation over that repository, when one
    /// stands. `None` is A2 §2's base-only answer, and is not an error.
    pub overlay_generation: Option<GenerationPin>,
}

/// One exact source generation, pinned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationPin {
    /// The declared source.
    pub source_name: String,
    /// The generation's own id.
    pub generation_id: String,
    /// Its content identity — what makes the pin re-resolvable rather than a
    /// description.
    pub content_key: String,
    /// How it was acquired.
    pub kind: &'static str,
    /// What the estate may do with it.
    pub authority: &'static str,
    /// When the observation completed.
    pub observed_at: String,
}

impl GenerationPin {
    /// Pin one [`SourceGeneration`].
    pub fn of(generation: &SourceGeneration) -> Self {
        Self {
            source_name: generation.source_name.clone(),
            generation_id: generation.id.clone(),
            content_key: generation.content_key.clone(),
            kind: generation.kind.as_str(),
            authority: generation.authority.as_str(),
            observed_at: generation.observed_at.clone(),
        }
    }

    /// The pin as JSON.
    pub fn json(&self) -> Value {
        json!({
            "source": self.source_name,
            "generation_id": self.generation_id,
            "content_key": self.content_key,
            "source_kind": self.kind,
            "authority_class": self.authority,
            "observed_at": self.observed_at,
        })
    }
}

/// §15's *"coverage states"*, per admitted source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageState {
    /// The declared source.
    pub source_name: String,
    /// One coverage status this source's generation carries.
    pub status: String,
    /// How many rows carry it.
    pub rows: usize,
}

impl CoverageState {
    /// The state as JSON.
    pub fn json(&self) -> Value {
        json!({"source": self.source_name, "status": self.status, "rows": self.rows})
    }
}

impl ContextSnapshot {
    /// §15's sixteen lines, as field names, in §15's own order — what a test
    /// compares the contract's list against, and what [`Self::json`] emits as
    /// keys.
    pub const FIELDS: [&'static str; 16] = [
        "context_snapshot_id",
        "coordinate",
        "journal_watermark",
        "work_world",
        "source_generations",
        "coverage",
        "query_result_ids",
        "retrieval",
        "profile",
        "selection_plan_hash",
        "bound",
        "referenced",
        "reachable",
        "payload_pointer",
        "budget",
        "rendered_bytes",
    ];

    /// Whether anything was compiled at all.
    pub fn is_empty(&self) -> bool {
        self.bound.is_empty() && self.referenced.is_empty()
    }

    /// The snapshot as JSON — the payload the journal carries, and the shape
    /// an inspector reads.
    pub fn json(&self) -> Value {
        json!({
            "context_snapshot_id": self.snapshot_id,
            "coordinate": {
                "estate_root": self.coordinate.estate_root,
                "work": self.coordinate.work_id,
                "stage": self.coordinate.stage_id,
                "stage_index": self.coordinate.stage_index,
                "attempt": self.coordinate.attempt,
                "execution": self.coordinate.execution_id,
            },
            "journal_watermark": self.journal_watermark,
            "work_world": {
                "repository": self.work_world.repository,
                "base_sha": self.work_world.base_sha,
                "overlay_generation": self.work_world.overlay_generation.as_ref().map(GenerationPin::json),
            },
            "source_generations": self
                .source_generations
                .iter()
                .map(GenerationPin::json)
                .collect::<Vec<_>>(),
            "coverage": self.coverage.iter().map(CoverageState::json).collect::<Vec<_>>(),
            "query_result_ids": self.query_result_ids,
            "retrieval": self.retrieval.as_ref().map(SearchTrace::json),
            "profile": self.profile,
            "selection_plan_hash": self.selection_plan_hash,
            "bound": self.bound.iter().map(EvidenceUnit::json).collect::<Vec<_>>(),
            "referenced": self.referenced.iter().map(EvidenceUnit::json).collect::<Vec<_>>(),
            "reachable": self.reachable.as_ref().map(ReachableScope::json),
            "payload_pointer": self.payload_pointer,
            "budget": self.budget.as_ref().map(BudgetReport::json),
            "rendered_bytes": self.rendered_bytes,
            "plan": self.plan.iter().map(StepRecord::json).collect::<Vec<_>>(),
            "degradation": self.degradation.as_ref().map(|d| json!({
                "reason": d.as_str(),
                "detail": d.detail(),
            })),
        })
    }

    /// Append the compiled world to a stage's authored `CONTEXT.md`.
    ///
    /// **Appended, never substituted**, exactly as the branch-status fact is
    /// (`crate::runtime::engine`'s `branch_status_context`): the stage's
    /// authored content is untouched and a reader can tell at a glance which
    /// part sergeant added. §12's *"procedure is data"* survives — this adds
    /// evidence beside the procedure, it does not interpret it.
    ///
    /// **This function makes no budget decision.** [`pack`] already spent
    /// §14's budgets and marked every unit `rendered` or not; this emits
    /// exactly what packing chose, through the same [`render_chunk`] packing
    /// measured. A budget applied here instead would be a second policy in a
    /// second place, and the snapshot's `budget` numbers would be a claim
    /// about a different function's output.
    ///
    /// §20's first non-goal is *"raw corpus stuffing"*, and the reason a body
    /// may be rendered at all now is that §14's hard budget exists to bound
    /// it: C1a rendered coordinates only and said so — *"the under-delivery
    /// is C1b's to close"* — because a body with no budget is exactly the
    /// non-goal. A source larger than the budget cannot fill Bound with body
    /// text; it becomes Referenced remainder.
    ///
    /// §2's Reachable descriptor renders whenever there is one, **including
    /// when both budgets are exhausted** (§21 item 5).
    ///
    /// A snapshot that compiled nothing returns the context **unchanged, byte
    /// for byte** — §3's *"If intelligence is disabled/unavailable and the
    /// workflow does not require it, the existing stage context path still
    /// executes"* (§21 item 13).
    pub fn render_onto(&self, context: &str) -> String {
        if self.is_empty() {
            return context.to_string();
        }
        let mut out = String::from(context);
        out.push_str("\n\n## Compiled context (Sergeant)\n\nsnapshot: ");
        out.push_str(&self.snapshot_id);
        out.push('\n');
        for (tier, units) in [
            (Tier::Bound, &self.bound),
            (Tier::Referenced, &self.referenced),
        ] {
            if !units.iter().any(|unit| unit.rendered) {
                continue;
            }
            out.push_str(&format!("\n{}\n", tier.as_str()));
            for unit in units.iter().filter(|unit| unit.rendered) {
                out.push_str(&render_chunk(unit));
            }
        }
        if let Some(reachable) = &self.reachable {
            out.push_str(&format!(
                "\nREACHABLE\n  {} ({})\n",
                reachable.selector,
                reachable.capabilities.join(", ")
            ));
        }
        out
    }
}

// ===================================================================
// §14's budgets
// ===================================================================

/// **§14's two rendering budgets.**
///
/// > *"Use a **hard** automatic-render budget. Reuse backend-native/token
/// > count if already available at the launch boundary; otherwise use a
/// > conservative documented estimate rather than requiring a universal
/// > tokenizer dependency."*
/// >
/// > *"Referenced coordinates have a **small separate** rendering budget
/// > because a pointer is far cheaper than loading the resource."*
/// >
/// > *"The budget is for automatic prompt material, **not a ban on the actor
/// > resolving Reachable/Referenced evidence when needed**."*
///
/// Three separate rules, and this type is shaped by all three.
///
/// # 1. Two budgets, because §14 says two
///
/// [`Self::bound_bytes`] and [`Self::referenced_bytes`] are spent
/// independently: an exhausted Bound budget never eats into the Referenced
/// one and an exhausted Referenced budget never eats into Bound. A single
/// shared number would be cheaper to implement and would be a contract miss —
/// the second sentence exists precisely because a pointer costs a line and a
/// resource costs a body.
///
/// # 2. The unit is **bytes**, and why that is the *documented estimate*
///
/// §14's first choice is a backend-native token count *"if already available
/// at the launch boundary"*. It is not available: the launch boundary is
/// [`crate::backend::StartRequest`], which carries the stage's `context` as a
/// `String` and no count of anything, and the one token-shaped capability an
/// adapter advertises — [`crate::backend::Capabilities::usage`] — is
/// *reporting*, read off a finished turn's result payload
/// (`crate::telemetry`'s `input_tokens`/`output_tokens` handling), long after
/// this compilation has to be over. So §14's second branch applies, and the
/// estimate is UTF-8 **bytes of rendered text**.
///
/// It is conservative in the exact sense §14 asks for, and the reason is a
/// property rather than a ratio somebody guessed: every tokenizer maps **at
/// least one byte** to each token it emits, so `tokens <= bytes` for any
/// tokenizer at all. A byte budget therefore over-states token cost and can
/// never under-state it, without adding a tokenizer dependency to bound
/// something a tokenizer would only bound more tightly (**R1/R3**). It is
/// also exactly measurable, which a token estimate would not be: the number
/// this type bounds is the number of bytes actually appended.
///
/// # 3. Where the two default numbers come from
///
/// Both are traced to one measurement of this repository's own shipped
/// content, taken **2026-08-30** in the C1b lane, and both commands re-run:
///
/// ```text
/// $ find .sergeant/workflows -name CONTEXT.md -printf '%s\n' | ...
/// n=55  mean=3964  median=3033  p90=6041  max=15476    (bytes)
///
/// $ git ls-files | awk '{ print length($0) }' | ...
/// n=573  mean=41  median=39  max=90                    (bytes per path)
/// ```
///
/// The first population is the 55 authored stage `CONTEXT.md` files of the
/// distro this binary embeds (`crate::domain::distro`'s `WORKFLOWS`) — the
/// procedure text a compiled world is appended *beside*.
/// [`Self::BOUND_BYTES`] is **8 KiB**: above the p90 authored stage context
/// (6041 B) and well below the largest (15476 B), so the automatic Bound
/// render can add at most about one typical stage's worth of material and can
/// never dwarf the procedure it accompanies.
///
/// The second gives the cost of a *pointer*. A rendered Atlas coordinate is
/// `source@content_key:path`, and a git source's `content_key` is the tree
/// OID — 40 hex characters (`crate::runtime::atlas::git`) — so a line costs
/// roughly `6 + 11 + 1 + 40 + 1 + 41 + 1 ≈ 100` bytes for this repository.
/// [`Self::REFERENCED_BYTES`] is **2 KiB**: about twenty pointers, which is
/// the same order as [`STEP_ROW_CAP`]'s 32 rows, and a quarter of Bound —
/// *small* and *separate*, as §14 puts it.
///
/// Neither number is tuned by anything at runtime (§20: *"learned context
/// policy/live self-tuning"*); both are constants a human wrote, with the
/// measurement that produced them written down beside them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderBudget {
    /// Hard ceiling, in bytes, on everything the Bound tier renders into the
    /// actor's context automatically.
    pub bound_bytes: u64,
    /// Hard ceiling, in bytes, on everything the Referenced tier renders —
    /// §14's *"small separate"* budget, spent independently of the Bound one.
    pub referenced_bytes: u64,
}

impl RenderBudget {
    /// 8 KiB — see this type's own doc for the measurement it comes from.
    pub const BOUND_BYTES: u64 = 8 * 1024;
    /// 2 KiB — see this type's own doc for the measurement it comes from.
    pub const REFERENCED_BYTES: u64 = 2 * 1024;

    /// The budget every production compilation runs under.
    pub const DEFAULT: Self = Self {
        bound_bytes: Self::BOUND_BYTES,
        referenced_bytes: Self::REFERENCED_BYTES,
    };
}

impl Default for RenderBudget {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// §15's *"budget + rendered size"*, per tier: what was allowed and what was
/// actually spent.
///
/// Both halves are on the snapshot because *"the budget was 8 KiB"* and
/// *"the render spent 300 bytes"* are different facts and a reader auditing a
/// compiled world needs both — and because `spent <= allowed` is then a claim
/// the journal itself carries, checkable after the fact rather than only at
/// the moment of rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetReport {
    /// The budget this compilation ran under.
    pub budget: RenderBudget,
    /// Bytes the Bound tier actually rendered. Never above
    /// [`RenderBudget::bound_bytes`].
    pub bound_spent: u64,
    /// Bytes the Referenced tier actually rendered. Never above
    /// [`RenderBudget::referenced_bytes`].
    pub referenced_spent: u64,
}

impl BudgetReport {
    /// The report as JSON — §15's `budget` line.
    pub fn json(&self) -> Value {
        json!({
            "unit": "bytes",
            "bound_bytes": self.budget.bound_bytes,
            "referenced_bytes": self.budget.referenced_bytes,
            "bound_spent": self.bound_spent,
            "referenced_spent": self.referenced_spent,
        })
    }
}

// ===================================================================
// Resolution (§21 items 3 and 4)
// ===================================================================

/// The source evidence one [`EvidenceCoordinate`] resolves back to.
///
/// §21 item 3's second half — *"every unit resolves to source evidence"* — is
/// a claim about **this** function's answer, not about a rendered line
/// looking plausible: a rendered unit that cannot be resolved back to a real
/// stored row is the same defect class as a register note asserting what the
/// code does not do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceEvidence {
    /// An exact stored unit of an exact generation, with its full text.
    Unit {
        /// The unit's own text, in full — not the possibly-absent excerpt the
        /// budget allowed into the prompt.
        text: String,
        /// Its heading, when it has one.
        title: Option<String>,
        /// Byte offsets into the original resource.
        byte_start: u64,
        /// End offset, exclusive.
        byte_end: u64,
    },
    /// An exact stored relationship of an exact generation.
    Relationship(crate::runtime::atlas::db::ResolvedRelationship),
    /// A record of the Work itself — a stage record or a repository binding.
    ///
    /// These two coordinate shapes are **not** Atlas rows: their source
    /// evidence is the Work's own journal-folded run, and the snapshot pins
    /// every field of it inline (`stage_id`/`index`/`attempt`/`status`,
    /// `repository`/`work_branch`/`base_sha`). Resolution says so rather than
    /// pretending a store lookup happened.
    WorkRecord,
}

/// **Resolve one coordinate to its source evidence, by direct lookup.**
///
/// §21 item 4: *"Referenced coordinates resolve **without broad
/// rediscovery**"* — §2's own words for the tier are *"the actor can resolve
/// them directly **without broad search**"*. That is a property of how this
/// is implemented, and it is why this function takes no query text, no
/// ranker, no filter and no limit: it takes a coordinate and hands it
/// straight to [`AtlasDb::resolve_unit`] / [`AtlasDb::resolve_relationship`],
/// each of which is an equality lookup on the row identity the coordinate
/// already carries. A resolver that searched for the coordinate's content
/// would answer the same way on a small fixture while being the exact thing
/// item 4 forbids — so the test for it resolves two identical bodies under
/// two different generations and requires the right one back.
///
/// **The budget does not appear here, and that is the point.** §14's last
/// sentence — *"the budget is for automatic prompt material, not a ban on the
/// actor resolving Reachable/Referenced evidence when needed"* — means an
/// exhausted budget must not make anything unresolvable. Nothing in this
/// path can see a [`RenderBudget`] at all.
///
/// `Ok(None)` is an honest answer: the coordinate names no such row. It is
/// how the item-3 test tells a real resolution from a vacuous one.
pub fn resolve(
    atlas: &AtlasDb,
    coordinate: &EvidenceCoordinate,
) -> Result<Option<SourceEvidence>, AtlasError> {
    match coordinate {
        EvidenceCoordinate::Stage { .. } | EvidenceCoordinate::Binding { .. } => {
            Ok(Some(SourceEvidence::WorkRecord))
        }
        EvidenceCoordinate::Atlas {
            generation_id,
            relative_path,
            ordinal,
            ..
        } => {
            let Some(ordinal) = ordinal else {
                return Ok(None);
            };
            Ok(atlas
                .resolve_unit(generation_id, relative_path, *ordinal)?
                .map(|unit| SourceEvidence::Unit {
                    text: unit.body,
                    title: unit.title,
                    byte_start: unit.byte_start,
                    byte_end: unit.byte_end,
                }))
        }
        EvidenceCoordinate::Relationship {
            generation_id,
            kind,
            from,
            to,
            ordinal,
            ..
        } => Ok(atlas
            .resolve_relationship(generation_id, kind, from, to, *ordinal)?
            .map(SourceEvidence::Relationship)),
    }
}

// ===================================================================
// Packing (§5 step 9) — where §14's budget is actually spent
// ===================================================================

/// What one unit costs the automatic render, formatted exactly once.
///
/// Both the packer (which decides what fits) and
/// [`ContextSnapshot::render_onto`] (which emits it) call this, so *"the
/// budget was not exceeded"* is arithmetic over the same bytes that reach the
/// prompt rather than an estimate of them. Two formatters would make the
/// budget a guess about the renderer's output.
fn render_chunk(unit: &EvidenceUnit) -> String {
    let mut chunk = format!("  [{}] {}\n", unit.step.number(), unit.coordinate.render());
    if let Some(excerpt) = &unit.excerpt {
        for line in excerpt.lines() {
            chunk.push_str("      | ");
            chunk.push_str(line);
            chunk.push('\n');
        }
    }
    chunk
}

/// The packed world: §2's two rendered tiers and what they spent.
#[derive(Debug)]
struct Packed {
    bound: Vec<EvidenceUnit>,
    referenced: Vec<EvidenceUnit>,
    bound_spent: u64,
    referenced_spent: u64,
    note: String,
}

/// **§5 step 9 — *"pack Bound; emit useful remainder as Referenced"*.**
///
/// One pass over the ledger's units **in contribution order**, which is §5's
/// step order: an earlier, more deterministic step spends the budget before a
/// later, fuzzier one gets to. That is the same ordering first-contributor-
/// wins already enforces, applied to the second scarce thing (§14's bytes)
/// rather than to identity.
///
/// For each Bound unit: resolve its text (a direct lookup, the same one the
/// actor would use), render the chunk, and take it **only if the whole chunk
/// fits** what is left of [`RenderBudget::bound_bytes`]. A unit that does not
/// fit is not truncated into a half-body — it is §5's *"useful remainder"*
/// and moves to Referenced as a pointer, where it competes for the separate
/// Referenced budget. A pointer that does not fit that either stays in the
/// snapshot unrendered: §14 bounds the *prompt*, never the record and never
/// the actor.
///
/// Two things deliberately never carry a body:
///
/// - **a Work record** ([`EvidenceCoordinate::Stage`]/[`Binding`]) — it has
///   no stored text; the coordinate *is* the evidence;
/// - **external evidence** (`authority_class = external`) — §21 item 9 wants
///   external evidence *"visibly external and unable to alter instruction
///   hierarchy"*, and that labeling is C1c's. Until it exists, external prose
///   does not enter a prompt from here. It is still Referenced, still
///   resolvable, and the step note says so. Under-delivering a tier is
///   recoverable; shipping unlabeled external prose into an actor's context
///   is not (**J5** — item 9 is this contract's, and it is not yet met).
///
/// [`Binding`]: EvidenceCoordinate::Binding
fn pack(
    atlas: &AtlasDb,
    units: &[EvidenceUnit],
    budget: &RenderBudget,
    generations: &[GenerationPin],
) -> Packed {
    let external: BTreeSet<&str> = generations
        .iter()
        .filter(|pin| pin.authority == AuthorityClass::External.as_str())
        .map(|pin| pin.generation_id.as_str())
        .collect();
    let mut bound: Vec<EvidenceUnit> = Vec::new();
    let mut referenced: Vec<EvidenceUnit> = Vec::new();
    let mut bound_spent = 0u64;
    let mut referenced_spent = 0u64;
    let mut demoted = 0usize;
    let mut unrendered = 0usize;
    let mut withheld_external = 0usize;

    for unit in units {
        let mut candidate = unit.clone();
        if candidate.tier == Tier::Bound {
            // Only resolve while the budget could still hold the answer:
            // resolving past exhaustion would be a read whose result is
            // thrown away.
            if bound_spent < budget.bound_bytes
                && let EvidenceCoordinate::Atlas { generation_id, .. } = &candidate.coordinate
            {
                if external.contains(generation_id.as_str()) {
                    withheld_external += 1;
                } else if let Ok(Some(SourceEvidence::Unit { text, .. })) =
                    resolve(atlas, &candidate.coordinate)
                {
                    candidate.excerpt = Some(text);
                }
            }
            let cost = render_chunk(&candidate).len() as u64;
            if bound_spent + cost <= budget.bound_bytes {
                candidate.rendered = true;
                bound_spent += cost;
                bound.push(candidate);
                continue;
            }
            // §5's "useful remainder": demoted to a pointer, and it competes
            // for the OTHER budget.
            demoted += 1;
            candidate.excerpt = None;
            candidate.tier = Tier::Referenced;
        }
        let cost = render_chunk(&candidate).len() as u64;
        if referenced_spent + cost <= budget.referenced_bytes {
            candidate.rendered = true;
            referenced_spent += cost;
        } else {
            unrendered += 1;
        }
        referenced.push(candidate);
    }

    let mut note = format!(
        "packed {} Bound unit(s) into {bound_spent}/{} budget bytes and {} Referenced \
         pointer(s) into {referenced_spent}/{} budget bytes",
        bound.len(),
        budget.bound_bytes,
        referenced.len(),
        budget.referenced_bytes,
    );
    if demoted > 0 {
        note.push_str(&format!(
            "; {demoted} Bound unit(s) did not fit and were emitted as Referenced remainder"
        ));
    }
    if withheld_external > 0 {
        note.push_str(&format!(
            "; {withheld_external} external unit(s) rendered as coordinates only, pending §21 \
             item 9's external labeling (C1c)"
        ));
    }
    if unrendered > 0 {
        note.push_str(&format!(
            "; {unrendered} coordinate(s) are in the snapshot but past the render budget — \
             still resolvable, not rendered"
        ));
    }
    Packed {
        bound,
        referenced,
        bound_spent,
        referenced_spent,
        note,
    }
}

// ===================================================================
// The compiler port
// ===================================================================

/// Everything §4 lists that the compiler is given, borrowed from the stage
/// launch that is about to happen.
#[derive(Debug)]
pub struct CompileRequest<'a> {
    /// The Work's own canonical estate root.
    pub estate_root: Option<&'a Path>,
    /// The Work.
    pub work_id: &'a str,
    /// §4's *"Work intent"* — also the query text steps 6/7 answer.
    pub intent: &'a str,
    /// §4's *"workflow + stage definition/content identity"*.
    pub stage: &'a StageDefinition,
    /// Its position in the workflow's stage order.
    pub stage_index: usize,
    /// Which attempt.
    pub attempt: u32,
    /// The execution the reservation just allocated.
    pub execution_id: &'a str,
    /// The journal's next sequence number, read before the compilation.
    pub journal_watermark: u64,
    /// §4's *"exact repo authority/scope"* — the surface's own bindings.
    pub bindings: &'a [BindingSummary],
    /// §5 step 1's *"prior declared artifacts"* — this run's stage records.
    pub prior_stages: &'a [StageRecord],
    /// The launch profile the stage was bound with, when it has one.
    pub profile: Option<&'a str>,
    /// §14's two hard automatic-render budgets. Every production compilation
    /// passes [`RenderBudget::DEFAULT`]; it is a request field rather than a
    /// constant read inside [`compile`] so a test can watch the hard bound
    /// actually bind, which a constant nobody can vary cannot show.
    pub budget: RenderBudget,
}

/// The port [`crate::runtime::engine::Engine`] calls before an actor starts.
///
/// A port rather than a direct `AtlasDb` field on the engine, for two reasons
/// that are both about §3: an engine with no compiler installed runs **the
/// existing stage launch path with nothing added** — item 13's degradation is
/// then a property of the type, not of a branch someone has to remember to
/// write — and the engine keeps knowing nothing about Atlas, which is what
/// keeps `runtime::atlas`'s dependency arrows pointing one way (see that
/// module's own doc).
pub trait ContextCompiler: std::fmt::Debug + Send + Sync {
    /// Compile the world for the fresh execution described by `request`.
    ///
    /// **Infallible by signature.** A compilation failure is a degraded
    /// snapshot ([`Degradation`]), never a failed launch: §18's whole posture
    /// is that degradation is *"visible, not fatal or fabricated"*, and a
    /// stage that could not have its world compiled still has a `CONTEXT.md`
    /// and a Work binding to run on.
    fn compile(&self, request: &CompileRequest<'_>) -> ContextSnapshot;
}

/// The production compiler: [`compile`] over a live Atlas handle.
///
/// The handle is behind a plain [`std::sync::Mutex`] rather than the async one
/// [`crate::api::ApiState`] uses, because the one caller
/// ([`crate::runtime::engine::Engine::reserve_stage`]) is synchronous. It is a
/// **second handle on the same database instance** (`Analytics::atlas`'s
/// `Connection::try_clone`), never a second `Connection::open` — see
/// [`AtlasDb::open`]'s own doc for why that distinction is the whole ballgame.
#[derive(Debug)]
pub struct AtlasContextCompiler {
    atlas: std::sync::Mutex<AtlasDb>,
}

impl AtlasContextCompiler {
    /// Wrap a handle derived from the operations projection.
    pub fn new(atlas: AtlasDb) -> Self {
        Self {
            atlas: std::sync::Mutex::new(atlas),
        }
    }
}

impl ContextCompiler for AtlasContextCompiler {
    fn compile(&self, request: &CompileRequest<'_>) -> ContextSnapshot {
        let atlas = self
            .atlas
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        compile(Some(&atlas), request)
    }
}

// ===================================================================
// The plan runner
// ===================================================================

/// How many rows any one §5 step may read or contribute.
///
/// A constant a human wrote, not a policy anything learns (§20: *"learned
/// context policy/live self-tuning"*). It bounds the compilation's cost and
/// its output; §14's automatic-render budget ([`RenderBudget`]) bounds the
/// rendered payload, which is a different bound over a different quantity —
/// rows read versus bytes rendered. Both apply, and neither implies the
/// other: a compilation can read 32 rows and render two of them.
pub const STEP_ROW_CAP: usize = 32;

/// **§5's nine steps, run in §5's order, through the gate that enforces it.**
///
/// `atlas` is `None` for §18's first rung — intelligence disabled — and the
/// result is a degraded snapshot that renders nothing. Everything else is one
/// pass down [`ResearchStep::PLAN`].
pub fn compile(atlas: Option<&AtlasDb>, request: &CompileRequest<'_>) -> ContextSnapshot {
    let coordinate = SnapshotCoordinate {
        estate_root: request
            .estate_root
            .map(|root| root.to_string_lossy().into_owned()),
        work_id: request.work_id.to_string(),
        stage_id: request.stage.id.clone(),
        stage_index: request.stage_index,
        attempt: request.attempt,
        execution_id: request.execution_id.to_string(),
    };
    let Some(atlas) = atlas else {
        return degraded(request, coordinate, Degradation::IntelligenceDisabled);
    };
    // §3's own order: resolve Work/estate/source generations FIRST, then run
    // the research plan. A world with no confirmed generation is §18's
    // "unavailable" rung, answered before a single step runs.
    let repository = request.bindings.first().map(|b| b.repository.clone());
    let filter = match &repository {
        Some(repository) => Admissibility {
            source: SourceSelector::WorkBase {
                work_id: request.work_id.to_string(),
                repository: repository.clone(),
            },
            kind: None,
            authority: None,
        },
        None => Admissibility::default(),
    };
    let admitted = match atlas.admissible_generations(&filter, STEP_ROW_CAP) {
        Ok(admitted) => admitted,
        Err(e) => {
            return degraded(
                request,
                coordinate,
                Degradation::AtlasUnavailable(e.to_string()),
            );
        }
    };
    if admitted.hits.is_empty() {
        return degraded(request, coordinate, Degradation::NoConfirmedGeneration);
    }
    let source_generations: Vec<GenerationPin> =
        admitted.hits.iter().map(GenerationPin::of).collect();
    let overlay_generation = repository.as_deref().and_then(|repository| {
        let name = overlay_source_name(request.work_id, repository);
        admitted
            .hits
            .iter()
            .find(|g| g.source_name == name)
            .map(GenerationPin::of)
    });
    let coverage = coverage_states(atlas, &admitted.hits);

    let mut ledger = ResearchLedger::new();
    // ---- 1. explicit stage inputs / prior declared artifacts
    {
        let mut step = ledger
            .enter(ResearchStep::StageInputs)
            .expect("step 1 is the plan's first");
        for record in request.prior_stages {
            if record.index >= request.stage_index {
                continue;
            }
            step.contribute(
                Tier::Bound,
                EvidenceCoordinate::Stage {
                    stage_id: record.stage_id.clone(),
                    index: record.index,
                    attempt: record.attempt,
                    status: record.status.as_str().to_string(),
                    summary: record.detail.clone(),
                },
            );
        }
        if request
            .prior_stages
            .iter()
            .all(|r| r.index >= request.stage_index)
        {
            step.note("this stage has no prior stage in the run");
        }
    }
    // ---- 2. exact Work bindings/changed resources
    {
        let mut step = ledger
            .enter(ResearchStep::WorkBindings)
            .expect("step 2 follows step 1");
        for binding in request.bindings {
            step.contribute(
                Tier::Bound,
                EvidenceCoordinate::Binding {
                    repository: binding.repository.clone(),
                    work_branch: binding.work_branch.clone(),
                    base_sha: binding.base_sha.clone(),
                },
            );
        }
        // "changed resources" is exactly the Work's own overlay generation —
        // a worktree scanned against its base holds only what the Work
        // changed (S5 W1b/W1d). Nothing here reads the surface: the overlay
        // is already derived evidence, and `sgt search` staying a pure reader
        // (H13.2) applies to this path for the same reason.
        match &overlay_generation {
            Some(pin) => {
                let units = atlas
                    .units(&pin.source_name, STEP_ROW_CAP)
                    .unwrap_or_default();
                if units.is_empty() {
                    step.note("the Work's overlay generation stands but holds no unit");
                }
                for unit in &units {
                    step.contribute(Tier::Bound, atlas_unit(pin, unit));
                }
            }
            None => step.note(
                "no overlay generation stands for this Work: its surface has not been scanned \
                 since it was bound, or it has none",
            ),
        }
    }
    // ---- 3. exact structural/document/tabular/mail relationships
    {
        let mut step = ledger
            .enter(ResearchStep::ExactRelationships)
            .expect("step 3 follows step 2");
        contribute_generation_rows(
            &mut step,
            &source_generations,
            Tier::Bound,
            "no container/document/mail parent-child relationship in this world",
            |source_name| atlas.child_resources(source_name, STEP_ROW_CAP),
            child_relationship,
        );
    }
    // ---- 4. deterministic dataset aggregates/joins/diffs DECLARED by the
    //         profile/workflow
    {
        let mut step = ledger
            .enter(ResearchStep::DeclaredDataOperations)
            .expect("step 4 follows step 3");
        // §21 item 2's own qualifier: "where the profile declares such
        // operations". §6's `[context] profile` grammar does not exist yet
        // and this wave does not invent it, so nothing declares an operation
        // and this step runs none. Recorded rather than skipped: the step is
        // entered, the reason is stated, and the order stays total.
        step.note(
            "no workflow or profile declared a deterministic dataset operation (§6's \
             `[context]` grammar is not part of this wave)",
        );
    }
    // ---- 5. exact Referenced neighbors
    {
        let mut step = ledger
            .enter(ResearchStep::ReferencedNeighbors)
            .expect("step 5 follows step 4");
        contribute_generation_rows(
            &mut step,
            &source_generations,
            Tier::Referenced,
            "no stored structural edge in this world",
            |source_name| atlas.edges(source_name, STEP_ROW_CAP),
            edge_relationship,
        );
    }
    // ---- 6/7. A2 lexical retrieval, then A2 semantic retrieval if
    //           installed/needed — S5's own pipeline, consumed (R2).
    let attribution = match &repository {
        Some(repository) => Attribution::Work {
            work_id: request.work_id.to_string(),
            repository: repository.clone(),
        },
        None => Attribution::Unmanaged,
    };
    let retrieval = atlas
        .traced_search(
            &LexicalQuery {
                text: request.intent,
                filter: &filter,
                family: None,
                limit: STEP_ROW_CAP,
                semantic: SemanticRequest::Requested,
            },
            attribution,
        )
        .ok();
    {
        let mut step = ledger
            .enter(ResearchStep::LexicalRetrieval)
            .expect("step 6 follows step 5");
        match &retrieval {
            Some((answer, _)) => {
                if answer.hits.is_empty() {
                    step.note("lexical retrieval returned no hit inside the admissible set");
                }
                for hit in &answer.hits {
                    step.contribute(
                        Tier::Bound,
                        EvidenceCoordinate::Atlas {
                            source_name: hit.source_name.clone(),
                            generation_id: hit.generation_id.clone(),
                            content_key: hit.content_key.clone(),
                            relative_path: hit.coordinate.relative_path().to_string(),
                            unit_key: Some(hit.unit_key.clone()),
                            ordinal: Some(hit.coordinate.ordinal()),
                        },
                    );
                }
            }
            None => step.note("the retrieval read failed; the launch is not failed for it"),
        }
    }
    {
        let mut step = ledger
            .enter(ResearchStep::SemanticRetrieval)
            .expect("step 7 follows step 6");
        // §5's "if installed/needed", answered by the pipeline rather than
        // assumed: `fused_search` runs `lexical_search` then
        // `semantic_search` inside one snapshot-isolated transaction, and
        // `SemanticStatus` is A2 §15's required, non-omittable honesty about
        // whether the semantic half actually contributed.
        match &retrieval {
            Some((answer, _)) => match answer.semantic {
                SemanticStatus::Applied => step.note(
                    "the semantic half ran inside step 6's one fused answer; every unit it \
                     ranked was already contributed there",
                ),
                other => step.note(format!(
                    "semantic retrieval did not run: {}",
                    other.as_str()
                )),
            },
            None => step.note("no retrieval answer to take a semantic half from"),
        }
    }
    // ---- 8. bounded structural/provenance expansion
    {
        let mut step = ledger
            .enter(ResearchStep::BoundedExpansion)
            .expect("step 8 follows step 7");
        // Expansion out of what retrieval found, along stored structure —
        // deterministic, and bounded by the same constant everything else is.
        // The `false` from `contribute` is the point: an edge step 5 already
        // held is not re-contributed here. Shares step 3/5's per-generation
        // loop (`contribute_generation_rows`, **R2**); the retrieval-hit
        // filter folds into the fetch closure instead of a second hand-rolled
        // loop.
        match &retrieval {
            Some((answer, _)) => {
                let paths: BTreeSet<&str> = answer
                    .hits
                    .iter()
                    .map(|hit| hit.coordinate.relative_path())
                    .collect();
                contribute_generation_rows(
                    &mut step,
                    &source_generations,
                    Tier::Referenced,
                    "no stored structure to expand from what retrieval found",
                    |source_name| {
                        atlas.edges(source_name, STEP_ROW_CAP).map(|edges| {
                            edges
                                .into_iter()
                                .filter(|edge| paths.contains(edge.relative_path.as_str()))
                                .collect()
                        })
                    },
                    edge_relationship,
                );
            }
            None => step.note("no stored structure to expand from what retrieval found"),
        }
    }
    // ---- 9. pack Bound; emit useful remainder as Referenced
    //
    // §14's budget is spent HERE and nowhere else. Packing is computed off
    // the ledger's finished units first, then step 9 is entered and states
    // what it did: the step adds no evidence, so it opens the writer only to
    // record the outcome, and the gate's order stays total either way.
    let packed = pack(atlas, ledger.units(), &request.budget, &source_generations);
    {
        let mut step = ledger.enter(ResearchStep::Pack).expect("step 9 is last");
        step.note(packed.note.clone());
    }

    let plan = ledger.executed().to_vec();
    let selection_plan_hash = selection_plan_hash(&plan, ledger.units());
    let Packed {
        bound,
        referenced,
        bound_spent,
        referenced_spent,
        ..
    } = packed;

    ContextSnapshot {
        snapshot_id: ulid::Ulid::generate().to_string(),
        coordinate,
        journal_watermark: request.journal_watermark,
        work_world: WorkWorld {
            repository,
            base_sha: request.bindings.first().map(|b| b.base_sha.clone()),
            overlay_generation,
        },
        source_generations,
        coverage,
        query_result_ids: Vec::new(),
        retrieval: retrieval.map(|(_, trace)| trace),
        profile: request.profile.map(str::to_string),
        selection_plan_hash,
        bound,
        referenced,
        reachable: Some(reachable_scope(&filter, request.work_id)),
        payload_pointer: None,
        budget: Some(BudgetReport {
            budget: request.budget,
            bound_spent,
            referenced_spent,
        }),
        rendered_bytes: 0,
        plan,
        degradation: None,
    }
}

/// §18's degraded answer: the coordinate, the reason, and nothing compiled.
fn degraded(
    request: &CompileRequest<'_>,
    coordinate: SnapshotCoordinate,
    degradation: Degradation,
) -> ContextSnapshot {
    ContextSnapshot {
        snapshot_id: ulid::Ulid::generate().to_string(),
        coordinate,
        journal_watermark: request.journal_watermark,
        work_world: WorkWorld {
            repository: request.bindings.first().map(|b| b.repository.clone()),
            base_sha: request.bindings.first().map(|b| b.base_sha.clone()),
            overlay_generation: None,
        },
        source_generations: Vec::new(),
        coverage: Vec::new(),
        query_result_ids: Vec::new(),
        retrieval: None,
        profile: request.profile.map(str::to_string),
        selection_plan_hash: selection_plan_hash(&[], &[]),
        bound: Vec::new(),
        referenced: Vec::new(),
        reachable: None,
        payload_pointer: None,
        budget: None,
        rendered_bytes: 0,
        plan: Vec::new(),
        degradation: Some(degradation),
    }
}

/// §15's *"selection-plan hash"*: a content hash over the executed plan and
/// every contributed coordinate, **in order**.
///
/// The step number goes into the hash beside the coordinate, so two
/// compilations that selected the same evidence by different plans — the same
/// resource attributed to step 2 in one and step 6 in the other — do not hash
/// the same. A hash over the selected set alone could not tell those apart,
/// which would make it a hash of the answer rather than of the plan.
fn selection_plan_hash(plan: &[StepRecord], units: &[EvidenceUnit]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.c1.selection-plan/v1\n");
    for record in plan {
        hasher.update(record.step.number().to_string().as_bytes());
        hasher.update(b":");
        hasher.update(record.contributed.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(record.already_held.to_string().as_bytes());
        hasher.update(b"\n");
    }
    for unit in units {
        hasher.update(unit.step.number().to_string().as_bytes());
        hasher.update(b":");
        hasher.update(unit.tier.as_str().as_bytes());
        hasher.update(b":");
        hasher.update(unit.coordinate.dedup_key().as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// §15's *"coverage states"* for the admitted generations, rolled up per
/// source and status.
fn coverage_states(atlas: &AtlasDb, admitted: &[SourceGeneration]) -> Vec<CoverageState> {
    let mut out = Vec::new();
    for generation in admitted {
        let rows = atlas
            .coverage(&generation.source_name, STEP_ROW_CAP)
            .unwrap_or_default();
        let mut counts: std::collections::BTreeMap<String, usize> = Default::default();
        for row in rows {
            *counts
                .entry(row.row.status.as_str().to_string())
                .or_default() += 1;
        }
        for (status, rows) in counts {
            out.push(CoverageState {
                source_name: generation.source_name.clone(),
                status,
                rows,
            });
        }
    }
    out
}

/// Step 3 and step 5's shared shape (**R2**): for every pinned source
/// generation, fetch its rows through `fetch`, contribute each one at `tier`
/// via `map`, and note `empty_note` once if nothing was found across every
/// pin. `fetch`'s own `STEP_ROW_CAP`/limit argument stays the caller's, same
/// as before this was factored out.
fn contribute_generation_rows<T>(
    step: &mut StepWriter<'_>,
    source_generations: &[GenerationPin],
    tier: Tier,
    empty_note: &str,
    fetch: impl Fn(&str) -> Result<Vec<T>, AtlasError>,
    map: impl Fn(&GenerationPin, &T) -> EvidenceCoordinate,
) {
    let mut found = 0usize;
    for pin in source_generations {
        for item in fetch(&pin.source_name).unwrap_or_default() {
            found += 1;
            step.contribute(tier, map(pin, &item));
        }
    }
    if found == 0 {
        step.note(empty_note);
    }
}

/// One stored unit, as an exact Atlas coordinate under its pinned generation.
fn atlas_unit(pin: &GenerationPin, unit: &StoredUnit) -> EvidenceCoordinate {
    EvidenceCoordinate::Atlas {
        source_name: pin.source_name.clone(),
        generation_id: pin.generation_id.clone(),
        content_key: pin.content_key.clone(),
        relative_path: unit.relative_path.clone(),
        unit_key: None,
        ordinal: Some(unit.ordinal),
    }
}

/// One container/document/mail parent-child relationship (§6.6's preserved
/// coordinates), as evidence.
fn child_relationship(pin: &GenerationPin, child: &StoredChildResource) -> EvidenceCoordinate {
    EvidenceCoordinate::Relationship {
        source_name: pin.source_name.clone(),
        generation_id: pin.generation_id.clone(),
        kind: "child_resource".to_string(),
        from: child.parent_relative_path.clone(),
        to: child.relative_path.clone(),
        ordinal: None,
    }
}

/// One stored syntax edge, as evidence. Its target stays **unresolved**,
/// exactly as [`StoredEdge::target`] is — resolving it here would be inventing
/// a fact the extractor did not derive.
fn edge_relationship(pin: &GenerationPin, edge: &StoredEdge) -> EvidenceCoordinate {
    EvidenceCoordinate::Relationship {
        source_name: pin.source_name.clone(),
        generation_id: pin.generation_id.clone(),
        kind: edge.kind.clone(),
        from: edge.relative_path.clone(),
        to: edge.target.clone(),
        ordinal: Some(edge.ordinal),
    }
}

/// §2's Reachable tier, as the descriptor it is.
fn reachable_scope(filter: &Admissibility, work_id: &str) -> ReachableScope {
    let (selector, source_name, work) = match &filter.source {
        SourceSelector::Any => ("any", None, None),
        SourceSelector::Named(name) => ("named", Some(name.clone()), None),
        SourceSelector::Exact { source_name, .. } => ("exact", Some(source_name.clone()), None),
        SourceSelector::WorkBase { repository, .. } => (
            "work_base",
            Some(repository.clone()),
            Some(work_id.to_string()),
        ),
    };
    ReachableScope {
        selector: selector.to_string(),
        source_name,
        work_id: work,
        capabilities: vec!["map", "search", "related", "query"],
    }
}

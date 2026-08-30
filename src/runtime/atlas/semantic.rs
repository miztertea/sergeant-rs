//! S5 W3 — A2 §15's honesty about the semantic half, as a **field**.
//!
//! # What this module is, and what it deliberately is not
//!
//! It is **not** A2 §6's semantic retrieval. There is no embedding model in
//! this build. W3 ran F5's gate order and stopped at gate 1: adding
//! `model2vec-rs` — A2 §6's own named candidate, and decision **A2-06**'s —
//! introduces RUSTSEC-2024-0436 (`paste`, unmaintained, no safe upgrade)
//! through `tokenizers`, which every Rust Model2Vec implementation depends
//! on and which no feature selection can drop. The full record, with the
//! verbatim `cargo deny check` output before and after, the 28-package
//! lockfile delta, and the escalation menu, is
//! `tests/fixtures/model2vec_corpus/SPIKE-F5.md`. Per the wave brief a new
//! advisory *"is a STOP that escalates — it does not get worked around"*, so
//! the candidate tree was reverted and the decision is the owner's (J0).
//!
//! What this module **is** is decision **H4**, which the sprint plan states
//! is already decided and which does not depend on that ruling in either
//! direction:
//!
//! > **A2 §15:** *"If semantic assets are absent, A2 degrades to
//! > deterministic filters + structural/exact + BM25 lexical retrieval and
//! > reports that coverage/capability honestly."*
//!
//! **H4's sharpening of that sentence is the whole design here: the honesty
//! is a required field, not a principle.** A consumer must be able to tell a
//! degraded answer from a complete one *mechanically* — by reading a value
//! that is always there — rather than by noticing that some other field is
//! absent. So [`SemanticStatus`] rides every search answer and is not an
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
//! This is F8's coverage-honesty rule applied to retrieval, and the same
//! shape W1d used for `WorkScope` and W2 used for
//! [`crate::runtime::atlas::db::LexicalAnswer::truncated`]: a caller must be
//! able to *state* what its answer covers.
//!
//! # Why `Applied` exists in a build that cannot produce it
//!
//! A2 §15's vocabulary — and H4's, verbatim — is `applied | not_installed |
//! disabled`. All three are declared here because the contract declares
//! three, and a consumer parsing this field parses the contract's set, not
//! this commit's reachable subset. As of this commit
//! [`installed_model`] always answers `None`, so [`SemanticStatus::Applied`]
//! is unreachable from [`resolve`] in production — and that is the truth
//! this build has to tell, not a stub. When a model is adopted,
//! [`installed_model`] is the one function that changes.
//!
//! Pure Rust over strings and enums: no database connection, no driver name
//! (Atlas's one-owner invariant, `tests/x1_atlas_substrate.rs::
//! atlas_database_has_exactly_one_owner`), and no filesystem access.

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
/// A2 §15's and H4's own vocabulary, and [`Self::as_str`] is their wire
/// spelling.
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
}

impl SemanticStatus {
    /// The stable wire spelling — A2 §15's and H4's literal vocabulary,
    /// `applied | not_installed | disabled`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::NotInstalled => "not_installed",
            Self::Disabled => "disabled",
        }
    }

    /// The inverse of [`Self::as_str`].
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "applied" => Some(Self::Applied),
            "not_installed" => Some(Self::NotInstalled),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }

    /// Whether the answer this status describes was produced **without** the
    /// semantic half — true for both degraded reasons, false only for
    /// [`Self::Applied`].
    ///
    /// The one-line predicate a consumer needs, so "is this answer complete?"
    /// never has to be re-derived by matching on the variant set at every
    /// call site (and never has to be re-derived *wrongly* when a fourth
    /// variant is added).
    pub fn is_degraded(self) -> bool {
        !matches!(self, Self::Applied)
    }
}

impl std::fmt::Display for SemanticStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
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

/// The semantic model installed on this host — **`None`, in this build, and
/// that is a fact rather than a placeholder.**
///
/// A2 §15 requires model assets to be *"host-cached, explicitly installed/
/// prefetched and content/version pinned"* and forbids a stage-time
/// download. This build has no adopted embedding model to install: F5's deny
/// gate failed on the A2-06 candidate and the decision escalated
/// (`tests/fixtures/model2vec_corpus/SPIKE-F5.md`). So there is nothing to
/// look for, no cache directory to define, and — importantly — **no code
/// path here that could reach the network**, which is A2-12's requirement
/// met by there being no fetcher at all.
///
/// This is the single function an adoption ruling changes. Everything else
/// in this module, and every consumer of [`SemanticStatus`], is already
/// correct under both branches of that ruling.
pub fn installed_model() -> Option<SemanticModel> {
    None
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
    fn the_three_variants_round_trip_their_contract_spelling() {
        for status in [
            SemanticStatus::Applied,
            SemanticStatus::NotInstalled,
            SemanticStatus::Disabled,
        ] {
            assert_eq!(SemanticStatus::parse(status.as_str()), Some(status));
        }
        assert_eq!(SemanticStatus::Applied.as_str(), "applied");
        assert_eq!(SemanticStatus::NotInstalled.as_str(), "not_installed");
        assert_eq!(SemanticStatus::Disabled.as_str(), "disabled");
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

    #[test]
    fn only_applied_is_undegraded() {
        assert!(!SemanticStatus::Applied.is_degraded());
        assert!(SemanticStatus::NotInstalled.is_degraded());
        assert!(SemanticStatus::Disabled.is_degraded());
    }

    #[test]
    fn this_build_installs_no_model_so_applied_is_unreachable_from_a_real_search() {
        assert_eq!(installed_model(), None);
        assert_eq!(
            resolve(SemanticRequest::Requested, installed_model().as_ref()),
            SemanticStatus::NotInstalled
        );
    }
}

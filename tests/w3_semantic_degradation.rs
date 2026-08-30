//! S5 W3 — decision **H4**: "no model present" is a supported state, and the
//! honesty about it is a **field**.
//!
//! A2 §15, verbatim: *"If semantic assets are absent, A2 degrades to
//! deterministic filters + structural/exact + BM25 lexical retrieval and
//! reports that coverage/capability honestly."* A2 §17 item 4, verbatim:
//! *"semantic retrieval, when installed, uses pinned local assets and can be
//! disabled/degraded cleanly."*
//!
//! H4 sharpens "honestly" into something checkable: every search answer
//! carries a REQUIRED, non-omittable `semantic: applied | not_installed |
//! disabled`, **distinct** from A2 §13's optional model-identity/hash field,
//! so a consumer distinguishes a degraded answer from a complete one
//! *mechanically* rather than by noticing an absent field.
//!
//! # Why there is no `applied` case in this file
//!
//! Because there is no embedding model in this build, and that is a recorded
//! outcome rather than an omission. W3 ran F5's gate order and stopped at
//! gate 1: `model2vec-rs` — A2 §6's own named candidate, and decision
//! A2-06's — introduces RUSTSEC-2024-0436 (`paste`, unmaintained, no safe
//! upgrade) through `tokenizers`, which no feature selection can drop and
//! which every Rust Model2Vec implementation depends on. The candidate tree
//! was reverted and the decision escalated (J0). Full record with verbatim
//! `cargo deny check` output:
//! `tests/fixtures/model2vec_corpus/SPIKE-F5.md`.
//!
//! So this suite proves the half of H4 that is true either way, and is the
//! half A2 §15 makes mandatory: **the lexical answer still comes back, and
//! it says the semantic half was not there.** The `applied` transition is
//! unit-tested in `src/runtime/atlas/semantic.rs` against a constructed
//! model value; it is the *host* that has none.

use std::collections::BTreeSet;
use std::path::Path;

use tempfile::TempDir;

use sergeant_rs::domain::event::rfc3339_utc_now;
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::runtime::atlas::db::{
    Admissibility, AtlasDb, LexicalAnswer, LexicalQuery, SourceSelector,
};
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{ScannedFile, ScannedUnit, SourceScan};
use sergeant_rs::runtime::atlas::semantic::{
    SemanticRequest, SemanticStatus, installed_model, resolve,
};
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR;
use sergeant_rs::runtime::journal::Journal;

// ---------------------------------------------------------------- fixtures

/// The one document every query below is meant to find. Ordinary prose, so
/// the lexical half has something real to score.
const NOTE_MD: &str = "\
The PaymentRetryPolicy governs settlement reconciliation and is the
policy a payment retry consults before giving up.
";

struct Estate {
    _data: TempDir,
    db: AtlasDb,
}

impl Estate {
    fn search(&self, text: &str, request: SemanticRequest) -> LexicalAnswer {
        self.db
            .lexical_search(&LexicalQuery {
                text,
                filter: &only_knowledge(),
                family: None,
                limit: 50,
                semantic: request,
            })
            .expect("lexical search")
    }
}

fn only_knowledge() -> Admissibility {
    Admissibility {
        source: SourceSelector::Named("knowledge".to_string()),
        kind: None,
        authority: None,
    }
}

/// One recorded generation holding one Markdown document — the smallest
/// corpus that makes "the lexical half still answers" a real assertion.
fn estate() -> Estate {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    let mut extractors = BTreeSet::new();
    extractors.insert(MARKDOWN_EXTRACTOR.to_string());
    let scan = SourceScan {
        source_name: "knowledge".to_string(),
        kind: SourceKind::LocalKnowledge,
        authority: AuthorityClass::EstateReadonly,
        content_key: "knowledge@key-1".to_string(),
        revision: None,
        observed_at: rfc3339_utc_now(),
        files: vec![ScannedFile {
            relative_path: "docs/note.md".to_string(),
            content_hash: "hash/docs/note.md".to_string(),
            extractor: MARKDOWN_EXTRACTOR.to_string(),
            local_key: "key/docs/note.md".to_string(),
            byte_len: NOTE_MD.len() as u64,
            mtime_millis: None,
            units: vec![ScannedUnit {
                ordinal: 0,
                kind: UnitKind::Document,
                heading_level: None,
                title: None,
                byte_start: 0,
                byte_end: NOTE_MD.len() as u64,
                text: NOTE_MD.to_string(),
            }],
            syntax: None,
            parent: None,
        }],
        coverage: Vec::new(),
        extractors,
        datasets: Vec::new(),
        root: None,
        context_fields: ContextFields::none(),
    };
    record_scan(&mut db, &mut journal, &scan, None).expect("record knowledge");
    Estate { _data: data, db }
}

// ------------------------------------------------------------- H4's own test

/// **The test the wave brief requires**: *"A test must prove a no-model run
/// still answers (lexical half) AND says `not_installed`."*
///
/// Both halves matter and either one alone would be worthless. An answer that
/// reports `not_installed` but returns nothing has not degraded — it has
/// failed, and A2 §15's whole point is that absence of semantic assets is a
/// **supported** state. An answer that returns hits without reporting the
/// degradation is the silent gap H4 exists to close.
#[test]
fn a_run_with_no_model_still_answers_through_the_lexical_half_and_says_not_installed() {
    let estate = estate();
    let answer = estate.search("PaymentRetryPolicy", SemanticRequest::Requested);

    assert!(
        !answer.hits.is_empty(),
        "A2 §15 degrades to lexical retrieval when semantic assets are absent \
         — the answer must still be an answer"
    );
    assert!(
        answer
            .hits
            .iter()
            .any(|hit| hit.coordinate.relative_path() == "docs/note.md"),
        "the lexical half must find the document that spells the query; got {:?}",
        answer
            .hits
            .iter()
            .map(|h| h.coordinate.relative_path().to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        answer.semantic,
        SemanticStatus::NotInstalled,
        "a host with no installed semantic assets must SAY so"
    );
    assert_eq!(
        answer.semantic_model, None,
        "A2 §13's model identity is recorded 'if used'; nothing was used here"
    );
}

/// H4's actual mechanism, and the reason the status is a second field rather
/// than an absent first one: *"so a consumer distinguishes a degraded answer
/// from a complete one **mechanically** rather than by noticing an absent
/// field"*.
///
/// Two answers over the same corpus for the same query differ only in what
/// the caller asked for. Their **optional** model field is `None` in both
/// cases — so a consumer reading only that field cannot tell them apart. The
/// **required** status field can. This test fails the moment the two states
/// are collapsed into one, or the moment the status is derived from the
/// model field's presence.
#[test]
fn disabled_and_not_installed_are_distinguishable_when_the_model_field_alone_is_not() {
    let estate = estate();
    let requested = estate.search("PaymentRetryPolicy", SemanticRequest::Requested);
    let suppressed = estate.search("PaymentRetryPolicy", SemanticRequest::Suppressed);

    assert_eq!(
        requested.semantic_model, suppressed.semantic_model,
        "the optional model field is None in both — it cannot be the honesty signal"
    );
    assert_eq!(requested.semantic, SemanticStatus::NotInstalled);
    assert_eq!(suppressed.semantic, SemanticStatus::Disabled);
    assert_ne!(
        requested.semantic, suppressed.semantic,
        "H4: the required field is what distinguishes the two degraded reasons"
    );
}

/// The stated precedence rule, pinned so it cannot drift: a caller that
/// suppressed the semantic half is told `disabled`, **not** `not_installed`,
/// even though "no model is installed" is also true of this host.
///
/// Reported that way because it is the fact the caller can act on: they
/// turned it off, and installing a model would not change this answer. The
/// reverse precedence would send an operator to install something that would
/// then still not be used.
#[test]
fn a_suppressed_request_reports_disabled_even_with_no_model_installed() {
    assert_eq!(installed_model(), None, "precondition: this host has none");
    let estate = estate();
    let answer = estate.search("PaymentRetryPolicy", SemanticRequest::Suppressed);
    assert_eq!(answer.semantic, SemanticStatus::Disabled);
    assert_eq!(
        resolve(SemanticRequest::Suppressed, installed_model().as_ref()),
        SemanticStatus::Disabled,
        "the search's status must be exactly what the stated rule resolves to"
    );
}

/// An answer with **no hits at all** still carries the status.
///
/// This is the path H4 is most easily lost on: `lexical_search` returns early
/// — before any scoring — when the query tokenizes to nothing, when the limit
/// is zero, or when the admissible corpus is empty. A status computed only
/// where hits are produced would leave those answers reporting whatever the
/// struct's construction happened to default to, which is precisely the
/// omittable field H4 forbids. Here the query has no tokens at all.
#[test]
fn an_answer_with_no_hits_still_carries_the_required_status() {
    let estate = estate();
    let answer = estate.search("   ...   ", SemanticRequest::Requested);
    assert!(answer.hits.is_empty(), "precondition: an empty answer");
    assert_eq!(
        answer.semantic,
        SemanticStatus::NotInstalled,
        "the early-return paths must carry the field too"
    );
}

/// The field is **required**, and this is the structural pin for that word.
///
/// A prose promise that a field is non-omittable is worth nothing; what makes
/// it true is that `LexicalAnswer::semantic` is a plain `SemanticStatus` and
/// not an `Option<SemanticStatus>`, so there is no "unset" a producer can
/// leave behind or a consumer can misread as "fine". This reads the declared
/// struct and fails if the field ever becomes optional, defaulted, or
/// disappears — the same shape W1d used to pin `WorkScope`.
#[test]
fn the_search_answers_semantic_field_is_required_not_optional() {
    let source = std::fs::read_to_string(Path::new("src/runtime/atlas/db.rs"))
        .expect("read src/runtime/atlas/db.rs");
    let start = source
        .find("pub struct LexicalAnswer {")
        .expect("LexicalAnswer must be declared in src/runtime/atlas/db.rs");
    let body = &source[start
        ..start
            + source[start..]
                .find("\n}")
                .expect("LexicalAnswer's declaration must close")];

    assert!(
        body.contains("pub semantic: SemanticStatus,"),
        "LexicalAnswer must carry the required status field verbatim; body was:\n{body}"
    );
    assert!(
        !body.contains("pub semantic: Option<"),
        "H4: the status field must never become optional; body was:\n{body}"
    );
    assert!(
        body.contains("pub semantic_model: Option<SemanticModel>,"),
        "A2 §13's model identity is the OPTIONAL field, and must stay a \
         separate one; body was:\n{body}"
    );
    assert!(
        !source.contains("impl Default for LexicalAnswer"),
        "a Default impl would reintroduce an 'unset' status by the back door"
    );
}

/// The other half of A2-12, structurally: **this build cannot reach the
/// network to get a model.**
///
/// A2 §15: *"Do not surprise-download a model in the middle of a stage."*
///
/// **WHAT THIS TEST DOES NOT PROVE, stated because a guard whose reach is
/// overstated is worse than none.** It is a substring scan of ONE file's
/// non-comment lines. It does not see a fetcher in a sibling module called
/// from here; it does not catch `std::process::Command::new("curl")`; it
/// does not catch a URL assembled by `concat!`. And `reqwest` (blocking,
/// rustls) is already in this crate's graph for backend transport, so the
/// means are present even though no embedding dependency is.
///
/// What it DOES prove is narrow and still worth having: nobody bolted an
/// obvious "convenience fallback" onto this module without turning it red.
///
/// **The real structural pin lands with adoption (W3b):** `model2vec-rs`
/// declared `default-features = false, features = ["local-only"]`, so
/// `hf-hub`/`ureq` are never compiled and the fetcher does not exist to be
/// called — A2-12 met by absence of a code path rather than by a scan. That
/// test belongs on the Cargo manifest, not on this file's text.
#[test]
fn the_semantic_module_names_no_obvious_fetcher() {
    let source = std::fs::read_to_string(Path::new("src/runtime/atlas/semantic.rs"))
        .expect("read src/runtime/atlas/semantic.rs");
    // Doc comments legitimately name the crates and advisories the spike
    // examined; only real code may not reach out.
    let code: String = source
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("//!") && !trimmed.starts_with("///") && !trimmed.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "http://",
        "https://",
        "reqwest",
        "ureq",
        "hf_hub",
        "hf-hub",
        "download",
        "fetch",
        "TcpStream",
    ] {
        assert!(
            !code.contains(forbidden),
            "A2-12 (weak guard, see this test's doc): the semantic module's code \
             must name no obvious fetcher; found {forbidden:?}"
        );
    }
}

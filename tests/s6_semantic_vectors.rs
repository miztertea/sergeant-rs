//! S6 — **the semantic half must answer from vectors persisted at scan
//! time**, not by re-embedding the corpus on every query.
//!
//! # The defect this suite exists for
//!
//! `AtlasDb::semantic_search` embedded every admissible unit's text inside
//! its per-generation loop, on every query. The `context` schema held no
//! vectors at all, so the cost of the flagship verb was O(corpus) *model
//! inference* per query rather than O(corpus) cosine — measured on the real
//! estate at ~1.7 s of the ~1.9 s answer (19,025 units), and 10 s (the CLI's
//! own `REQUEST_TIMEOUT`, `src/cli.rs:47`) on a debug build.
//!
//! A2-07 is untouched by the fix: *"use exact cosine first; defer ANN/vector
//! DB"*. Nothing here adds an index, an approximation or a pruning rule —
//! `semantic::cosine` over the admissible set is still the entire similarity
//! mechanism. What changes is *when* the corpus is embedded.
//!
//! # Why the negative case is the one that can fail
//!
//! An answer produced from stored vectors and an answer produced by
//! re-embedding are **identical when the vectors are there** — same model,
//! same texts, same cosine. So a test over a freshly scanned corpus cannot
//! tell the two implementations apart, and would be vacuous.
//!
//! The observable that separates them is an index that has **no** stored
//! vectors: one scanned on a host with no model, then queried after the
//! model is installed. Query-time embedding answers it fully and reports
//! `applied`; reading stored vectors cannot, and A1 §15 —
//! *"missing capability is never represented as successful empty
//! evidence"* — forbids calling that `applied`.

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use sergeant_rs::domain::source::{EstateAdmission, EstateBinding};
use sergeant_rs::runtime::atlas::db::{
    Admissibility, AtlasDb, LexicalQuery, SemanticAnswer, SourceSelector,
};
use sergeant_rs::runtime::atlas::lexical::LexicalFamily;
use sergeant_rs::runtime::atlas::record::scan_and_record;
use sergeant_rs::runtime::atlas::scan::KnowledgeSource;
use sergeant_rs::runtime::atlas::semantic::{MODEL_DIR_ENV, SemanticRequest, SemanticStatus};
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::journal::Journal;

/// S6 D1 — every generation this suite records is bound to this one root.
const D1_ESTATE: &str = "/estates/s6_semantic_vectors";

/// Six short documents, each about one thing. Small on purpose: this suite
/// asks *where the vectors came from*, not whether the model ranks well —
/// that is `tests/w3b_semantic_retrieval.rs`'s hand-verified corpus.
const CORPUS: [(&str, &str); 6] = [
    (
        "payments/decline-handling.md",
        "When a card transaction is declined the client waits and attempts the same charge again after an exponential back-off delay.",
    ),
    (
        "ops/disk-pressure.md",
        "An alert fires when free space on the data volume falls below ten percent and the operator is paged.",
    ),
    (
        "garden/roses.md",
        "Prune the rose bushes in late winter and mulch the beds with well-rotted compost.",
    ),
    (
        "kernel/scheduling.md",
        "The scheduler assigns time slices to runnable threads on each available CPU core.",
    ),
    (
        "kitchen/sourdough.md",
        "Feed the starter twice a day until it doubles reliably, then shape the loaf and bake it on a stone.",
    ),
    (
        "music/tuning.md",
        "Tighten the peg slowly until the string sounds the reference pitch from the tuning fork.",
    ),
];

/// Point the documented operator override at this repository's committed
/// assets. Safe because cargo-nextest runs every test in its own process
/// (the same reason `tests/w3b_semantic_retrieval.rs` says so).
fn install_model() {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/semantic-model");
    assert!(
        assets.join("model.safetensors").is_file(),
        "the committed assets must be present at {}",
        assets.display()
    );
    unsafe { std::env::set_var(MODEL_DIR_ENV, &assets) };
}

/// Make sure no model can be found: the `cargo install`-from-source host.
fn uninstall_model() {
    unsafe { std::env::remove_var(MODEL_DIR_ENV) };
}

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create fixture directory");
    }
    std::fs::write(&path, body).expect("write fixture");
}

struct Estate {
    _data: TempDir,
    _root: TempDir,
    db: AtlasDb,
}

impl Estate {
    fn semantic(&self, text: &str) -> SemanticAnswer {
        self.db
            .semantic_search(&LexicalQuery {
                text,
                filter: &everything(),
                family: Some(LexicalFamily::Document),
                limit: 50,
                semantic: SemanticRequest::Requested,
            })
            .expect("semantic search")
    }

    /// Re-open the same data directory on a fresh handle.
    ///
    /// `AtlasDb` loads the model **at most once per handle**
    /// (`AtlasDb::semantic_engine`), so "the model was installed after the
    /// scan" is only observable through a new handle — the same thing a
    /// restarted daemon does.
    fn reopen(self) -> Self {
        let db = AtlasDb::open(self._data.path()).expect("reopen atlas");
        Self { db, ..self }
    }
}

fn everything() -> Admissibility {
    Admissibility {
        estate: EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::Any,
        kind: None,
        authority: None,
    }
}

/// Scan [`CORPUS`] through the real `scan_and_record` path, under whatever
/// model installation the caller has already set up.
fn scanned_estate() -> Estate {
    let data = tempfile::tempdir().expect("data dir");
    let root = tempfile::tempdir().expect("knowledge root");
    for (relative, body) in CORPUS {
        write(root.path(), relative, body);
    }
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let knowledge = KnowledgeSource {
        name: "knowledge".to_string(),
        root: root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::none(),
    };
    scan_and_record(
        &mut db,
        &mut journal,
        &knowledge,
        None,
        &EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record knowledge");
    Estate {
        _data: data,
        _root: root,
        db,
    }
}

// ------------------------------------------------------------------ the red

/// **An index scanned with no model installed has no stored vectors, and a
/// later query must not call that `applied`.**
///
/// This is the assertion that separates "embedded at scan time" from
/// "embedded at query time". Before the fix the query embedded the corpus
/// itself, so it answered this estate in full and reported `applied` — a
/// complete semantic answer produced by a capability the index does not
/// have. A1 §15: *"missing capability is never represented as successful
/// empty evidence"*; A2 §15 asks for the same honesty in the other
/// direction.
#[test]
fn an_index_scanned_without_the_model_does_not_report_applied_once_it_is_installed() {
    uninstall_model();
    let estate = scanned_estate();
    assert_eq!(
        estate.semantic("running out of storage space").semantic,
        SemanticStatus::NotInstalled,
        "precondition: nothing was installed when this index was built"
    );

    install_model();
    let estate = estate.reopen();
    let answer = estate.semantic("running out of storage space");

    assert_ne!(
        answer.semantic,
        SemanticStatus::Applied,
        "the model is installed but this index carries no vectors for it; \
         reporting `applied` claims a semantic ranking the store cannot \
         produce (A1 §15). Answer was: {answer:?}"
    );
}

/// The other half, so the negative above cannot pass by the semantic half
/// having been broken outright: **an index scanned with the model installed
/// still answers, and still ranks the document a person would pick first.**
#[test]
fn an_index_scanned_with_the_model_installed_still_answers_semantically() {
    install_model();
    let estate = scanned_estate();
    let answer = estate.semantic("running out of storage space");

    assert_eq!(answer.semantic, SemanticStatus::Applied);
    let top = answer
        .hits
        .first()
        .expect("a semantic answer over six documents has hits");
    assert_eq!(
        top.coordinate.relative_path(),
        "ops/disk-pressure.md",
        "ranked: {:?}",
        answer
            .hits
            .iter()
            .map(|h| (h.coordinate.relative_path().to_string(), h.score))
            .collect::<Vec<_>>()
    );
}

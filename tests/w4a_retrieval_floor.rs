//! **The relevance floor (#231: wired at birth).**
//!
//! The semble-parity wave changed how `sgt search` orders answers, and the
//! evidence it changed them *for the better* is a precision measurement over
//! a hand-verified question set — not any test that existed before it. Every
//! one of `w4_rrf_fusion`'s checks passed at the pinned revision while the
//! measured p@1 was `.404`; a suite that cannot see a relevance regression
//! cannot stop one.
//!
//! So this file is the guard, and it is wired in the same commit as the
//! change it guards. It scans **this repository's own working tree** as one
//! Atlas source, asks the committed question set's in-repo questions through
//! the real `fused_search`, and fails if precision@1 falls below a recorded
//! floor.
//!
//! # What is measured, and where the number came from
//!
//! `tests/fixtures/retrieval/parity-question-set.tsv` — 52 questions, every
//! answer hand-verified by opening the file. This suite uses the rows whose
//! answers live in this repository; the rest of the set is estate-wide and is
//! measured out of tree (a knowledge library and a harness memory directory
//! are not in any checkout).
//!
//! **The corpus here is not the corpus the wave's headline numbers were
//! measured on.** Those were four estate sources including a knowledge
//! library that mirrors much of this repository; this is one source, and it
//! is *easier*, because the duplicated-content distractors are absent. The
//! two numbers are not comparable and this file does not pretend they are —
//! [`P_AT_1_FLOOR`] is calibrated against **this** corpus, by running it.
//!
//! # Not an acceptance criterion this suite invented
//!
//! A2 §17 has ten acceptance items and **none of them is about relevance** —
//! they are structural ("lexical + semantic lists fuse through RRF and
//! deterministic rerank"), which is why a build could satisfy all ten and
//! still answer badly. A relevance floor is therefore a *new* criterion, and
//! its threshold belongs to the owner rather than to an implementing stage.
//! [`P_AT_1_FLOOR`] is set to the measured value at the wave's own revision,
//! recorded so the owner can raise or lower it deliberately; it is deliberately
//! NOT set to a value nobody has met.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use sergeant_rs::domain::source::EstateBinding;
use sergeant_rs::runtime::atlas::db::{Admissibility, AtlasDb, LexicalQuery};
use sergeant_rs::runtime::atlas::record::scan_and_record;
use sergeant_rs::runtime::atlas::scan::KnowledgeSource;
use sergeant_rs::runtime::atlas::semantic::{MODEL_DIR_ENV, SemanticRequest};
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::journal::Journal;

const D1_ESTATE: &str = "/estates/w4a_retrieval_floor";
const SOURCE: &str = "sergeant-rs";

/// The precision@1 this repository's own corpus must not fall below.
///
/// **Provenance: measured, at the revision that set it.** Running this suite
/// prints the achieved value; the floor is that value less one question of
/// slack, so an unrelated documentation edit that moves one answer does not
/// redden CI while a ranking regression still does. It is not an aspiration
/// and not a number anyone reasoned to — see this file's own header for why
/// the threshold is escalated to the owner rather than settled here.
const P_AT_1_FLOOR: f64 = 0.57;

/// A row of the committed question set that this repository can answer.
struct Question {
    id: String,
    query: String,
    accept: BTreeSet<String>,
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The in-repo half of `tests/fixtures/retrieval/parity-question-set.tsv`.
///
/// Reading the committed file rather than restating it: a question set that
/// lives in two places stops agreeing with itself, and the file is the
/// evidence artifact the wave committed.
fn questions() -> Vec<Question> {
    let path = repository_root().join("tests/fixtures/retrieval/parity-question-set.tsv");
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut out = Vec::new();
    for line in body.lines() {
        if line.starts_with('#') || line.trim().is_empty() || line.starts_with("id\t") {
            continue;
        }
        let mut columns = line.split('\t');
        let id = columns.next().expect("id").to_string();
        let _category = columns.next().expect("category");
        let query = columns.next().expect("query").to_string();
        let accept: BTreeSet<String> = columns
            .next()
            .expect("accept")
            .split(',')
            .filter_map(|entry| entry.trim().strip_prefix("sergeant-rs:"))
            .map(str::to_string)
            .collect();
        if !accept.is_empty() {
            out.push(Question { id, query, accept });
        }
    }
    out
}

/// Every accepted answer must be a file that actually exists at this
/// revision, or the measurement is against a question set that has rotted.
fn assert_answers_exist(questions: &[Question]) {
    let root = repository_root();
    for question in questions {
        for path in &question.accept {
            assert!(
                root.join(path).exists(),
                "{}: the hand-verified answer {path} no longer exists",
                question.id
            );
        }
    }
}

struct Corpus {
    _data: tempfile::TempDir,
    db: AtlasDb,
}

/// This repository's working tree, scanned as one Atlas source.
///
/// `target/` and `.git/` are excluded explicitly: they are build output and
/// object storage, not evidence, and scanning them would dominate the corpus
/// and the runtime both.
fn scan_this_repository() -> Corpus {
    let assets = repository_root().join("assets/semantic-model");
    assert!(
        assets.join("model.safetensors").is_file(),
        "the committed semantic assets must be present at {} — without them \
         this suite would silently measure the degraded lexical-only path",
        assets.display()
    );
    unsafe { std::env::set_var(MODEL_DIR_ENV, &assets) };

    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let source = KnowledgeSource {
        name: SOURCE.to_string(),
        root: repository_root(),
        ignore: vec![
            "target/**".to_string(),
            ".git/**".to_string(),
            "assets/**".to_string(),
        ],
        context_fields: ContextFields::none(),
    };
    scan_and_record(
        &mut db,
        &mut journal,
        &source,
        None,
        &EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("scan this repository");
    Corpus { _data: data, db }
}

fn ask(corpus: &Corpus, query: &str, limit: usize) -> Vec<String> {
    let filter = Admissibility::within_estate(D1_ESTATE);
    corpus
        .db
        .fused_search(&LexicalQuery {
            text: query,
            filter: &filter,
            family: None,
            limit,
            semantic: SemanticRequest::Requested,
        })
        .expect("fused search")
        .hits
        .iter()
        .map(|hit| hit.coordinate.relative_path().to_string())
        .collect()
}

/// **The floor.** precision@1 over the committed question set's in-repo
/// questions, through the real `fused_search`, on this repository's own
/// content.
///
/// **Non-vacuous by construction, and checked:** the assertion is a
/// *threshold*, so it discriminates by definition — reverting the wave's
/// rerank to the lexicographic key drops the achieved value below it. The
/// printed per-question output names every miss, so a failure says which
/// answers moved rather than only that a number fell.
#[test]
fn precision_at_one_over_the_committed_question_set_does_not_regress() {
    let questions = questions();
    assert!(
        questions.len() >= 30,
        "the question set must stay large enough not to be an anecdote: {} rows",
        questions.len()
    );
    assert_answers_exist(&questions);

    let corpus = scan_this_repository();
    let mut hit_at_1 = 0usize;
    let mut hit_at_5 = 0usize;
    for question in &questions {
        let answers = ask(&corpus, &question.query, 5);
        let first = answers
            .first()
            .is_some_and(|path| question.accept.contains(path));
        let any = answers
            .iter()
            .take(5)
            .any(|path| question.accept.contains(path));
        if first {
            hit_at_1 += 1;
        }
        if any {
            hit_at_5 += 1;
        }
        println!(
            "{} {} {:?} -> {:?}",
            question.id,
            if first {
                "HIT@1"
            } else if any {
                "hit@5"
            } else {
                "MISS "
            },
            question.query,
            answers
        );
    }

    let total = questions.len() as f64;
    let p_at_1 = hit_at_1 as f64 / total;
    let p_at_5 = hit_at_5 as f64 / total;
    println!(
        "p@1 = {hit_at_1}/{} = {p_at_1:.3}   p@5 = {hit_at_5}/{} = {p_at_5:.3}   floor = {P_AT_1_FLOOR}",
        questions.len(),
        questions.len()
    );
    assert!(
        p_at_1 >= P_AT_1_FLOOR,
        "retrieval relevance regressed: p@1 {p_at_1:.3} is below the recorded \
         floor {P_AT_1_FLOOR}. The per-question lines above name every answer \
         that moved."
    );
}

/// The question set is evidence, and evidence that quietly shrinks is not
/// evidence. Pins the committed file's shape: at least 30 in-repo questions,
/// at least four categories across the whole file, and no duplicate ids.
#[test]
fn the_committed_question_set_keeps_its_shape() {
    let path = repository_root().join("tests/fixtures/retrieval/parity-question-set.tsv");
    let body = std::fs::read_to_string(&path).expect("read the question set");
    let rows: Vec<&str> = body
        .lines()
        .filter(|line| {
            !line.starts_with('#') && !line.trim().is_empty() && !line.starts_with("id\t")
        })
        .collect();
    assert!(
        rows.len() >= 50,
        "the whole set is 52 questions: {}",
        rows.len()
    );

    let mut ids: BTreeSet<&str> = BTreeSet::new();
    let mut categories: BTreeSet<&str> = BTreeSet::new();
    for row in &rows {
        let mut columns = row.split('\t');
        let id = columns.next().expect("id");
        let category = columns.next().expect("category");
        assert!(ids.insert(id), "duplicate question id {id}");
        categories.insert(category);
        assert_eq!(
            columns.count(),
            2,
            "every row is four tab-separated columns: {row}"
        );
    }
    assert!(
        categories.len() >= 4,
        "the set must span code, prose, config and cross-source: {categories:?}"
    );
}

/// A guard against the guard: the corpus this suite scans must actually
/// contain the repository, not an empty temp directory that would make every
/// question a miss and the floor unreachable — or, worse, a corpus so small
/// that a future edit makes the floor trivially met.
#[test]
fn the_scanned_corpus_is_this_repository() {
    let corpus = scan_this_repository();
    let answers = ask(&corpus, "reciprocal rank fusion", 10);
    assert!(
        answers
            .iter()
            .any(|path| Path::new(path).starts_with("src/runtime/atlas")),
        "the scan did not reach this repository's own source: {answers:?}"
    );
}

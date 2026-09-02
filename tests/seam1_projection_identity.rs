//! Wave `projection-identity`, seam 1 — the owner's ruling
//! (`projection-model-and-false-j0s-2026-08-31.md` #3): **the file is the
//! identity; sources are memberships.** One physical file on disk, covered
//! by two admissible sources, must be one unit in the projection — not two.
//!
//! # What is real here
//!
//! A real, temporary Git repository (initialized and committed with actual
//! `git` subprocess calls) mirrors the live estate's own measured shape
//! (`estate-double-indexing-2026-08-31.md`): a `[[repo]]` mount whose tree
//! contains a `knowledge/` subtree, plus a `[[knowledge]]` source declared
//! at that exact subtree. Both are scanned and recorded through the real
//! production entry points — `scan_and_record_estate_git` (git.rs's real
//! `git ls-tree`/`cat-file` extraction) and `scan_and_record` (scan.rs's
//! real filesystem walk) — into a real on-disk `AtlasDb`, then queried
//! through the real `lexical_search`. Nothing here is hand-built.

#[allow(dead_code)]
const D1_ESTATE: &str = "/estates/seam1_projection_identity";

use std::path::Path;

use tempfile::TempDir;

use sergeant_rs::domain::source::EstateBinding;
use sergeant_rs::runtime::atlas::db::{Admissibility, AtlasDb, LexicalQuery, SourceSelector};
use sergeant_rs::runtime::atlas::git::EstateGitSource;
use sergeant_rs::runtime::atlas::record::{scan_and_record, scan_and_record_estate_git};
use sergeant_rs::runtime::atlas::scan::KnowledgeSource;
use sergeant_rs::runtime::atlas::semantic::SemanticRequest;
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::git::git as run_git;
use sergeant_rs::runtime::journal::Journal;

/// A one-commit repository whose tree contains `knowledge/doctrine.md`,
/// carrying one planted term this suite's fixtures do not otherwise use.
fn repo_with_overlapping_knowledge_subtree() -> (TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    run_git(root, &["init", "--initial-branch=main"]).expect("init");
    run_git(root, &["config", "user.email", "t@example.com"]).expect("email");
    run_git(root, &["config", "user.name", "T"]).expect("name");
    std::fs::create_dir_all(root.join("knowledge")).expect("mkdir");
    // Deliberately no Markdown heading: `text::markdown_units` returns
    // exactly one `Document` unit for a headingless file (the whole text as
    // one string, A1 §6.1) and a second `Section` unit per heading it finds
    // — a fixture with a heading would conflate that pre-existing,
    // orthogonal one-file/two-units shape with the one this wave's fix is
    // actually about (one file, two *sources*), muddying what a failure
    // here would mean.
    std::fs::write(
        root.join("knowledge/doctrine.md"),
        "The term zzseam1uniqueterm appears exactly once in this estate.\n",
    )
    .expect("write");
    run_git(root, &["add", "-A"]).expect("add");
    run_git(root, &["commit", "-m", "one"]).expect("commit");
    let sha = run_git(root, &["rev-parse", "HEAD"]).expect("rev-parse");
    (dir, sha)
}

fn record_both_sources(root: &Path, sha: &str, db: &mut AtlasDb, journal: &mut Journal) {
    let repo_source = EstateGitSource {
        name: "sergeant-rs-workspace".to_string(),
        mount: root.to_path_buf(),
        pinned_sha: sha.to_string(),
        ignore: Vec::new(),
    };
    scan_and_record_estate_git(
        db,
        journal,
        &repo_source,
        None,
        &EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record estate-git scan");

    let knowledge_source = KnowledgeSource {
        name: "knowledge-library".to_string(),
        root: root.join("knowledge"),
        ignore: Vec::new(),
        context_fields: ContextFields::none(),
    };
    scan_and_record(
        db,
        journal,
        &knowledge_source,
        None,
        &EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record knowledge scan");
}

/// **The measured defect, closed.** `estate-double-indexing-2026-08-31.md`'s
/// exact shape: one file, two admissible sources. Before this wave's fix,
/// `lexical_search` for the planted term returned 4 hits (2 per source) and
/// BM25's document frequency counted the same physical occurrence twice.
#[test]
fn one_physical_file_covered_by_two_sources_is_one_unit_not_two() {
    let (repo_dir, sha) = repo_with_overlapping_knowledge_subtree();
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    record_both_sources(repo_dir.path(), &sha, &mut db, &mut journal);

    let filter = Admissibility::within_estate(D1_ESTATE);
    let answer = db
        .lexical_search(&LexicalQuery {
            text: "zzseam1uniqueterm",
            filter: &filter,
            family: None,
            limit: 50,
            semantic: SemanticRequest::Suppressed,
        })
        .expect("lexical search");

    assert_eq!(
        answer.hits.len(),
        1,
        "one physical file covered by two admissible sources must be one \
         unit, not two — got {:?}",
        answer.hits
    );
}

/// The same scenario, checked at the BM25 statistic the duplication actually
/// corrupts: `LEXICAL_DOCUMENT_FREQUENCY_SQL` must count the file once, not
/// once per covering source. A `df` of 2 here is invisible in a single-hit
/// count if `LEXICAL_CORPUS_SQL`'s `N` inflated by the same factor, so this
/// is checked through the score's own math rather than a second count: with
/// exactly one occurrence anywhere in the (deduped) corpus, `document
/// frequency == 1` is the only value that makes
/// `an_inadmissible_unit_with_a_perfect_lexical_match_is_never_returned`'s
/// own BM25 formula (`w2_lexical_retrieval.rs`) produce a *positive* score at
/// all — a `df` that silently doubled without a matching doubling of `N`
/// would move the score, not merely the hit count, which is exactly the
/// "42/260 duplicate top-5 slots" shape the wave brief measured.
#[test]
fn the_deduped_unit_scores_as_a_single_occurrence_not_a_doubled_one() {
    let (repo_dir, sha) = repo_with_overlapping_knowledge_subtree();
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    record_both_sources(repo_dir.path(), &sha, &mut db, &mut journal);

    let filter = Admissibility::within_estate(D1_ESTATE);
    let answer = db
        .lexical_search(&LexicalQuery {
            text: "zzseam1uniqueterm",
            filter: &filter,
            family: None,
            limit: 50,
            semantic: SemanticRequest::Suppressed,
        })
        .expect("lexical search");
    assert_eq!(answer.hits.len(), 1, "{:?}", answer.hits);
    assert!(
        answer.hits[0].score > 0.0,
        "a term with a genuine document frequency of 1 must score positive; \
         a corrupted df would have driven this to zero or negative"
    );
}

/// **Membership, not identity.** Filtering to `--type knowledge` (via
/// `Admissibility::kind`, stage 4's grouping) must still find the file
/// through its knowledge-source membership — one source's own name/kind
/// narrows *which* generation the deduped unit is reachable through, never
/// whether the unit itself exists.
#[test]
fn the_deduped_unit_is_still_reachable_through_either_sources_own_filter() {
    let (repo_dir, sha) = repo_with_overlapping_knowledge_subtree();
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");

    record_both_sources(repo_dir.path(), &sha, &mut db, &mut journal);

    for source_name in ["sergeant-rs-workspace", "knowledge-library"] {
        let filter = Admissibility {
            source: SourceSelector::Named(source_name.to_string()),
            ..Admissibility::within_estate(D1_ESTATE)
        };
        let answer = db
            .lexical_search(&LexicalQuery {
                text: "zzseam1uniqueterm",
                filter: &filter,
                family: None,
                limit: 50,
                semantic: SemanticRequest::Suppressed,
            })
            .expect("lexical search");
        assert_eq!(
            answer.hits.len(),
            1,
            "source {source_name:?} must still find the file through its own \
             membership: {:?}",
            answer.hits
        );
    }
}

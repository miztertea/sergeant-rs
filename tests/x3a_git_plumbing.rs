//! S3 X3a acceptance: the mechanical estate-git extraction plumbing.
//!
//! Four claims this wave makes, and the tests that would actually fail if any
//! of them stopped being true:
//!
//! * **A scan stays on the SHA it pinned while the mount moves underneath.**
//!   `a_scan_stays_on_its_pinned_sha_while_head_advances` advances the mount's
//!   HEAD *between* a scan's two phases and asserts the extraction is still
//!   the world phase one listed;
//!   `a_scan_racing_a_committing_thread_is_still_one_world` does the same with
//!   a real concurrent committer rather than a staged window. Drift is
//!   observed in both, and blended into the rows in neither.
//! * **F7's estate-git key is the blob OID plus the extractor** — never a
//!   second hash of bytes Git already hashed. Pinned here against the stored
//!   rows, not only against the in-memory scan.
//! * **A Work overlay is base + difference, and dies with its Work.**
//! * **F6's intelligence lane has a real consumer, and it is not the execution
//!   lane.**

/// **S6 D1 — A2 §2 stage 1's estate coordinate.** This suite is
/// single-estate: every generation it records is bound to this one root and
/// every filter it builds is admitted from it. The cross-estate case — two
/// estates on one host daemon, which is where the axis actually earns its
/// keep — is `tests/d1_estate_isolation.rs`, deliberately not folded in
/// here, because a suite that never crosses estates cannot notice an estate
/// filter that does nothing (that is exactly how the leak survived: this
/// file's ancestors all passed).
#[allow(dead_code)]
const D1_ESTATE: &str = "/estates/x3a_git_plumbing";

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::domain::source::{Coverage, KIND_SOURCE_SCANNED, estate_git_key};
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::git::{
    EstateGitSource, extract_tree, list_tree, observe_drift, scan_estate_git,
    scan_estate_git_with_worker,
};
use sergeant_rs::runtime::atlas::lane::scan_estate_git_on_lane;
use sergeant_rs::runtime::atlas::office::DOCX_EXTRACTOR;
use sergeant_rs::runtime::atlas::overlay::{WorkOverlay, scan_work_overlay};
use sergeant_rs::runtime::atlas::record::{
    ScanRecord, scan_and_record_estate_git, scan_and_record_overlay,
};
use sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR;
use sergeant_rs::runtime::atlas::worker::WorkerRuntime;
use sergeant_rs::runtime::engine::Engine;
use sergeant_rs::runtime::git::git;
use sergeant_rs::runtime::journal::Journal;

// ---------------------------------------------------------------- fixtures

/// A repository with one commit per `commits` entry, and the SHA of each.
fn repo(commits: &[&[(&str, &str)]]) -> (TempDir, PathBuf, Vec<String>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let mount = dir.path().join("mount");
    std::fs::create_dir_all(&mount).expect("mkdir");
    git(&mount, &["init", "--initial-branch=main"]).expect("init");
    git(&mount, &["config", "user.email", "t@example.com"]).expect("email");
    git(&mount, &["config", "user.name", "T"]).expect("name");
    let mut shas = Vec::new();
    for (n, files) in commits.iter().enumerate() {
        shas.push(commit(&mount, files, &format!("commit {n}")));
    }
    (dir, mount, shas)
}

fn commit(mount: &Path, files: &[(&str, &str)], message: &str) -> String {
    for (path, body) in files {
        let full = mount.join(path);
        std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
        std::fs::write(&full, body).expect("write");
    }
    git(mount, &["add", "-A"]).expect("add");
    git(mount, &["commit", "-m", message]).expect("commit");
    git(mount, &["rev-parse", "HEAD"]).expect("rev-parse")
}

fn source(mount: &Path, sha: &str) -> EstateGitSource {
    EstateGitSource {
        name: "product".to_string(),
        mount: mount.to_path_buf(),
        pinned_sha: sha.to_string(),
        ignore: Vec::new(),
    }
}

fn journal_at(dir: &Path) -> Journal {
    Journal::open(dir).expect("open journal")
}

fn summaries(journal: &Journal) -> Vec<Value> {
    journal
        .replay_from_floor()
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == KIND_SOURCE_SCANNED)
        .map(|e| e.payload)
        .collect()
}

// ------------------------------------------- the concurrent-HEAD-advance rule

/// **The wave's headline property.** A scan pins a SHA; the mount's HEAD then
/// moves — a Captain commit, another Work, an unrelated process — and the scan
/// is still evidence about the world it pinned, not a blend of two.
///
/// The advance happens in the one window where it could possibly matter:
/// between listing the tree and reading its blobs. If the extraction consulted
/// a ref, an index, or the working tree at any point, this is where it would
/// show, and it would show as `after.md` appearing or `b.md` changing.
#[test]
fn a_scan_stays_on_its_pinned_sha_while_head_advances() {
    let (_dir, mount, shas) =
        repo(&[&[("a.md", "# A\n"), ("b.md", "# B\n"), ("docs/c.md", "# C\n")]]);
    let pinned = shas[0].clone();
    let src = source(&mount, &pinned);

    // Phase one: the world is fixed here.
    let tree = list_tree(&src).expect("list");
    let reference = extract_tree(&src, &tree).expect("reference extraction");

    // The mount moves: one file rewritten, one deleted, one added, and HEAD
    // now points somewhere else entirely.
    std::fs::remove_file(mount.join("b.md")).expect("remove");
    let advanced = commit(
        &mount,
        &[("a.md", "# A, rewritten\n"), ("after.md", "# After\n")],
        "advance",
    );
    assert_ne!(advanced, pinned);
    assert_eq!(
        git(&mount, &["rev-parse", "HEAD"]).expect("head"),
        advanced,
        "the mount really did move"
    );

    // Phase two, run *after* the advance, against the tree listed before it.
    let scan = extract_tree(&src, &tree).expect("extraction after the advance");
    assert_eq!(
        scan.files, reference.files,
        "the world moved under the scan"
    );
    assert_eq!(scan.coverage, reference.coverage);
    assert_eq!(scan.content_key, reference.content_key);
    assert_eq!(scan.revision.as_deref(), Some(pinned.as_str()));
    assert_eq!(scan.files.len(), 3);
    assert_eq!(
        scan.files
            .iter()
            .find(|f| f.relative_path == "a.md")
            .expect("a.md")
            .units[0]
            .text,
        "# A\n",
        "the pinned scan read the advanced commit's bytes"
    );
    assert!(
        !scan
            .coverage
            .iter()
            .any(|r| r.path.as_deref() == Some("after.md")),
        "a file committed after the pin entered the scan"
    );
    assert!(
        scan.files.iter().any(|f| f.relative_path == "b.md"),
        "a file deleted after the pin left the scan"
    );

    // And the drift is *observed*, with both ends named, rather than absorbed.
    let drift = observe_drift(&src, &tree).expect("the mount moved, so there is drift to report");
    assert_eq!(drift.repository, "product");
    assert_eq!(drift.before, pinned);
    assert_eq!(drift.observed, advanced);
}

/// The same property against a genuinely concurrent mutator rather than a
/// staged window: a thread commits to the mount in a loop while a full scan
/// runs, and the scan's answer is byte-identical to a scan taken before any of
/// it started.
#[test]
fn a_scan_racing_a_committing_thread_is_still_one_world() {
    let (_dir, mount, _) = repo(&[&[("a.md", "# A\n"), ("b.md", "# B\n")]]);
    // Enough files that a scan is not over before the committer's first commit
    // lands.
    for i in 0..60 {
        std::fs::write(
            mount.join(format!("f{i:02}.md")),
            format!("# File {i}\n\nbody\n"),
        )
        .expect("write");
    }
    let pinned = commit(&mount, &[], "many files");
    let src = source(&mount, &pinned);
    let reference = scan_estate_git(&src).expect("reference").scan;

    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let committer = {
        let mount = mount.clone();
        let stop = Arc::clone(&stop);
        std::thread::spawn(move || {
            let mut n = 0;
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                std::fs::write(mount.join("a.md"), format!("# Churn {n}\n")).expect("write");
                commit(&mount, &[], &format!("churn {n}"));
                n += 1;
            }
            n
        })
    };
    // Scan repeatedly while the mount churns underneath.
    for _ in 0..5 {
        let scanned = scan_estate_git(&src).expect("scan during churn");
        assert_eq!(
            scanned.scan.files, reference.files,
            "a concurrent commit leaked into a pinned scan"
        );
        assert_eq!(scanned.scan.content_key, reference.content_key);
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let commits = committer.join().expect("committer thread");
    assert!(commits > 0, "the committer never actually moved HEAD");
    let final_scan = scan_estate_git(&src).expect("final scan");
    assert_eq!(final_scan.scan.files, reference.files);
    assert!(
        final_scan.drift.is_some(),
        "HEAD moved {commits} times and no drift was reported"
    );
}

// --------------------------------------------------- F7, through the store

/// F7's estate-git half, checked where it durably lands: the stored
/// `local_key` is `estate_git_key(blob oid, extractor)`, the stored content
/// identity is Git's own OID, and a re-scan of the same tree evicts nothing
/// because no source byte changed.
#[test]
fn stored_estate_git_keys_are_blob_oids_and_a_rescan_evicts_nothing() {
    let data = tempfile::tempdir().expect("tempdir");
    let (_dir, mount, shas) = repo(&[&[("a.md", "# A\n"), ("docs/b.md", "# B\n")]]);
    let src = source(&mount, &shas[0]);
    let mut db = AtlasDb::open(data.path()).expect("open atlas");
    let mut journal = journal_at(data.path());

    let (recorded, drift) = scan_and_record_estate_git(
        &mut db,
        &mut journal,
        &src,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record");
    assert!(drift.is_none());
    let generation = match &recorded {
        ScanRecord::Recorded {
            generation_id,
            evicted,
            ..
        } => {
            assert!(evicted.is_none(), "nothing to supersede on a first scan");
            generation_id.clone()
        }
        other => panic!("expected a recorded generation, got {other:?}"),
    };

    let units = db.units("product", 100).expect("units");
    assert!(!units.is_empty());
    let oid = git(&mount, &["rev-parse", &format!("{}:a.md", shas[0])]).expect("oid");
    let expected = estate_git_key(&oid, MARKDOWN_EXTRACTOR);
    assert!(
        units.iter().any(|u| u.local_key == expected),
        "no stored unit carries the blob-oid key {expected}"
    );

    // The journal's one summary names the revision, which is what makes a
    // stored generation traceable back to a commit.
    let events = summaries(&journal);
    assert_eq!(events.len(), 1, "exactly one summary per completed scan");
    assert_eq!(
        events[0].get("revision").and_then(Value::as_str),
        Some(shas[0].as_str())
    );
    assert_eq!(
        events[0].get("source_kind").and_then(Value::as_str),
        Some("estate_git")
    );
    assert_eq!(
        events[0].get("generation").and_then(Value::as_str),
        Some(generation.as_str())
    );

    // Re-scanning the same commit is the same world: ruling §4 evicts nothing.
    let (again, _) = scan_and_record_estate_git(
        &mut db,
        &mut journal,
        &src,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("rescan");
    assert!(
        matches!(&again, ScanRecord::Unchanged { generation_id, .. } if *generation_id == generation),
        "an unchanged tree churned a generation: {again:?}"
    );
    assert_eq!(summaries(&journal).len(), 1, "an unchanged scan journaled");
}

/// A commit whose *tree* is identical is the same world, however different the
/// commit is — the reason the generation is keyed on the tree and not the SHA.
#[test]
fn a_new_commit_with_an_identical_tree_is_the_same_generation() {
    let data = tempfile::tempdir().expect("tempdir");
    let (_dir, mount, shas) = repo(&[&[("a.md", "# A\n")]]);
    let mut db = AtlasDb::open(data.path()).expect("open atlas");
    let mut journal = journal_at(data.path());
    scan_and_record_estate_git(
        &mut db,
        &mut journal,
        &source(&mount, &shas[0]),
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record");

    // An empty commit: new SHA, new message, byte-identical tree.
    git(
        &mount,
        &["commit", "--allow-empty", "-m", "no content change"],
    )
    .expect("empty commit");
    let second = git(&mount, &["rev-parse", "HEAD"]).expect("head");
    assert_ne!(second, shas[0]);
    let (record, _) = scan_and_record_estate_git(
        &mut db,
        &mut journal,
        &source(&mount, &second),
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record");
    assert!(
        matches!(record, ScanRecord::Unchanged { .. }),
        "a commit that changed no source byte evicted a generation: {record:?}"
    );
}

// ------------------------------------------------------- the Work overlay

/// A Work overlay is base + difference, is scoped to its Work, and is evicted
/// with it — leaving a reported `generation_evicted` row rather than a gap.
#[test]
fn a_work_overlay_is_scoped_to_its_work_and_evicted_with_it() {
    let data = tempfile::tempdir().expect("tempdir");
    let (dir, mount, shas) = repo(&[&[("a.md", "# A\n"), ("b.md", "# B\n")]]);
    let base = shas[0].clone();
    let surface = dir.path().join("surface");
    git(
        &mount,
        &[
            "worktree",
            "add",
            "-b",
            "sergeant/01WORK",
            surface.to_str().expect("utf8"),
            &base,
        ],
    )
    .expect("worktree add");
    std::fs::write(surface.join("a.md"), "# A, in flight\n").expect("write");

    let mut db = AtlasDb::open(data.path()).expect("open atlas");
    let mut journal = journal_at(data.path());
    // A plain estate-git generation for the same repository, which must
    // survive the Work's eviction untouched.
    scan_and_record_estate_git(
        &mut db,
        &mut journal,
        &source(&mount, &base),
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record mount");

    let overlay = WorkOverlay {
        work_id: "01WORK".to_string(),
        repository: "product".to_string(),
        surface: surface.clone(),
        base_sha: base.clone(),
        ignore: Vec::new(),
    };
    let recorded = scan_and_record_overlay(
        &mut db,
        &mut journal,
        &overlay,
        None,
        &sergeant_rs::domain::source::EstateBinding::Estate(D1_ESTATE.to_string()),
    )
    .expect("record overlay");
    let overlay_generation = match &recorded {
        ScanRecord::Recorded { generation_id, .. } => generation_id.clone(),
        other => panic!("expected a recorded overlay, got {other:?}"),
    };
    let overlay_source = "work:01WORK/product";
    assert!(
        db.confirmed_generation(overlay_source)
            .expect("read")
            .is_some()
    );

    // The uncommitted edit is what the overlay indexed, and the untouched file
    // is still the base's.
    let units = db.units(overlay_source, 100).expect("units");
    assert!(
        units
            .iter()
            .any(|u| u.relative_path == "a.md" && u.body.contains("in flight")),
        "the overlay did not index the surface's uncommitted edit"
    );
    let scanned = scan_work_overlay(&overlay).expect("overlay");
    assert_eq!(scanned.changed, BTreeSet::from(["a.md".to_string()]));
    assert_eq!(scanned.base_sha, base);

    // Retire the Work: its overlay generations go, and say so.
    let evicted = db.evict_work_overlays("01WORK").expect("evict");
    assert_eq!(evicted, vec![overlay_generation]);
    assert!(
        db.confirmed_generation(overlay_source)
            .expect("read")
            .is_none(),
        "an overlay outlived the Work it was scoped to"
    );
    let coverage = db.coverage(overlay_source, 100).expect("coverage");
    let eviction = coverage
        .iter()
        .find(|row| row.row.status == Coverage::GenerationEvicted)
        .expect("an eviction must be reported, never a silent gap");
    assert!(
        eviction
            .row
            .detail
            .as_deref()
            .is_some_and(|d| d.contains("01WORK")),
        "{eviction:?}"
    );

    // The mount's own generation is untouched — a Work retiring is not a
    // statement about the repository.
    assert!(
        db.confirmed_generation("product").expect("read").is_some(),
        "evicting a Work's overlay took the repository's generation with it"
    );
    assert!(
        db.evict_work_overlays("01OTHER").expect("evict").is_empty(),
        "an unrelated Work id evicted something"
    );
}

// ------------------------------------------------- F6, the intelligence lane

/// F6: extraction acquires the **intelligence** lane, it is bounded, and it
/// never draws down the execution lane — H1-15's stub, now with a consumer.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn extraction_runs_bounded_on_the_intelligence_lane_and_never_the_execution_lane() {
    let data = tempfile::tempdir().expect("tempdir");
    let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path())
        .with_intelligence_lane_cap(2)
        .with_execution_lane_cap(3);

    let (_dir, mount, shas) = repo(&[&[("a.md", "# A\n"), ("docs/b.md", "# B\n")]]);
    let src = source(&mount, &shas[0]);

    // The lane's real consumer answers exactly what the direct call does.
    let direct = scan_estate_git(&src).expect("direct");
    let on_lane = scan_estate_git_on_lane(&engine, src.clone())
        .await
        .expect("on the lane");
    assert_eq!(on_lane.scan.files, direct.scan.files);
    assert_eq!(on_lane.tree_oid, direct.tree_oid);

    // Bounded: with a cap of two, a third job cannot start until one of the
    // first two finishes. Held permits are what prove it.
    let first = engine.try_admit_intelligence().expect("permit 1");
    let second = engine.try_admit_intelligence().expect("permit 2");
    assert_eq!(engine.intelligence_lane.available_permits(), 0);
    assert!(
        engine.try_admit_intelligence().is_none(),
        "the intelligence lane admitted past its cap"
    );

    // And a saturated intelligence lane leaves the execution lane whole —
    // the mirror image of H1-15's own assertion, in the direction only a real
    // consumer could break.
    assert_eq!(engine.execution_lane.available_permits(), 3);
    let scan_while_saturated = scan_estate_git_on_lane(&engine, src.clone());
    tokio::pin!(scan_while_saturated);
    tokio::select! {
        _ = &mut scan_while_saturated => panic!("a job ran while the lane was full"),
        _ = tokio::time::sleep(std::time::Duration::from_millis(150)) => {}
    }
    assert_eq!(
        engine.execution_lane.available_permits(),
        3,
        "intelligence-lane pressure spent the execution lane's budget"
    );

    // Release one permit and the parked job completes — the wait was the lane,
    // not a deadlock.
    drop(first);
    let finished = tokio::time::timeout(std::time::Duration::from_secs(30), scan_while_saturated)
        .await
        .expect("the parked job must run once a permit frees")
        .expect("scan");
    assert_eq!(finished.scan.files, direct.scan.files);
    drop(second);
    assert_eq!(engine.intelligence_lane.available_permits(), 2);
}

/// A job that panics is reported, not propagated: Atlas is derived evidence,
/// and one bad file may not take the daemon down (A1-01).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_panicking_intelligence_job_is_reported_and_frees_its_permit() {
    let data = tempfile::tempdir().expect("tempdir");
    let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path())
        .with_intelligence_lane_cap(1);
    let failed = engine
        .run_intelligence(|| panic!("an extractor exploded"))
        .await;
    assert!(failed.is_err(), "a panicking job must surface as an error");
    assert_eq!(
        engine.intelligence_lane.available_permits(),
        1,
        "a panicking job leaked its permit"
    );
    let ok: usize = engine.run_intelligence(|| 7).await.expect("still usable");
    assert_eq!(ok, 7);
}

// ------------------------------------------------- S4 Y8: worker dispatch

/// S4 Y8: a repository-resident `.docx` routes through the real supervised
/// worker exactly the way a filesystem one does
/// (`tests/y8_adapter_dispatch.rs`'s own end-to-end proof) — the estate-git
/// half of the wave's own dispatch requirement, over real bytes committed to
/// a real repository rather than a synthetic fixture.
#[test]
fn a_committed_docx_blob_routes_through_the_worker_via_estate_git() {
    let docx = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/anydoc_corpus/docx_fixtures/01-plain-headings-paragraphs.docx"),
    )
    .expect("read fixture");
    let dir = tempfile::tempdir().expect("tempdir");
    let mount = dir.path().join("mount");
    std::fs::create_dir_all(&mount).expect("mkdir");
    git(&mount, &["init", "--initial-branch=main"]).expect("init");
    git(&mount, &["config", "user.email", "t@example.com"]).expect("email");
    git(&mount, &["config", "user.name", "T"]).expect("name");
    std::fs::write(mount.join("report.docx"), &docx).expect("write docx");
    git(&mount, &["add", "-A"]).expect("add");
    git(&mount, &["commit", "-m", "one"]).expect("commit");
    let sha = git(&mount, &["rev-parse", "HEAD"]).expect("rev-parse");

    let worker = WorkerRuntime {
        program: PathBuf::from(env!("CARGO_BIN_EXE_sgt-atlas-worker")),
        deadline: std::time::Duration::from_secs(20),
    };
    let scan = scan_estate_git_with_worker(&source(&mount, &sha), &worker)
        .expect("scan")
        .scan;
    let row = scan
        .coverage
        .iter()
        .find(|r| r.path.as_deref() == Some("report.docx"))
        .expect("coverage row for report.docx");
    assert_eq!(row.status, Coverage::Indexed, "{:?}", scan.coverage);
    assert!(
        scan.extractors.contains(DOCX_EXTRACTOR),
        "{:?}",
        scan.extractors
    );
    let file = scan
        .files
        .iter()
        .find(|f| f.relative_path == "report.docx")
        .expect("docx landed in source.files");
    assert!(!file.units.is_empty(), "docx must produce document units");

    // F7's estate-git half still holds for a worker-routed resource: the
    // stored content identity is Git's own blob OID, never a second BLAKE3
    // hash of bytes Git already hashed — even though the WIRE protocol to
    // the worker always uses BLAKE3 (`dispatch_worker_resource`'s own doc
    // explains why those are allowed to differ).
    let oid = git(&mount, &["rev-parse", &format!("{sha}:report.docx")]).expect("rev-parse");
    assert_eq!(file.content_hash, oid);
    assert_eq!(file.local_key, estate_git_key(&oid, DOCX_EXTRACTOR));
}

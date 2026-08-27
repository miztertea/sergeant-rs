//! S3 X2 acceptance: `[[knowledge]]` sources, the local-knowledge scanner,
//! and **F1's persistence contract** — the one this wave exists to make
//! checkable.
//!
//! The sharpest risk the recon named for S3 is a contributor pattern-matching
//! the operations projection's disposable discipline onto `source.*`. The
//! module docs say not to; the tests here are what would actually fail if
//! someone did:
//!
//! * `source_facts_survive_a_real_daemon_restart` starts and stops a real
//!   daemon twice over a data dir that already holds scanned rows, and
//!   asserts the rows are still there while the operations projection was
//!   refolded from scratch. A "delete Atlas on start like we delete
//!   analytics" change fails here immediately.
//! * `a_crash_between_the_rows_and_the_summary_reports_neither` is F1's
//!   crash-window variant: rows are committed and the process dies before the
//!   `source.scanned` summary. Startup reconciliation must leave
//!   neither-reported (with an explicit eviction row), never a half-scan
//!   reported as coverage.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::domain::event::Event;
use sergeant_rs::domain::source::{Coverage, KIND_SOURCE_SCANNED, UnitKind};
use sergeant_rs::runtime::analytics::Analytics;
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::record::{ScanRecord, scan_and_record};
use sergeant_rs::runtime::atlas::scan::{KnowledgeSource, scan_local_knowledge};
use sergeant_rs::runtime::journal::Journal;

mod support;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

// ---------------------------------------------------------------- helpers

fn run(cwd: &Path, data_dir: &Path, args: &[&str]) -> Output {
    let output = Command::new(SGT)
        .current_dir(cwd)
        .arg("--data-dir")
        .arg(data_dir)
        .args(args)
        .output()
        .expect("run sgt");
    Output {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Output {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {}", self.stdout))
    }

    fn assert_ok(&self, what: &str) -> &Self {
        assert_eq!(
            self.code,
            Some(0),
            "{what} must succeed\nstdout: {}\nstderr: {}",
            self.stdout,
            self.stderr
        );
        self
    }

    fn assert_fails(&self, what: &str) -> &Self {
        assert_ne!(
            self.code,
            Some(0),
            "{what} must fail\nstdout: {}",
            self.stdout
        );
        self
    }
}

/// A knowledge tree with the given files, outside any estate.
fn knowledge_tree(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    for (path, body) in files {
        let full = dir.path().join(path);
        std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
        std::fs::write(&full, body).expect("write");
    }
    dir
}

fn source(name: &str, root: &Path) -> KnowledgeSource {
    KnowledgeSource {
        name: name.to_string(),
        root: root.to_path_buf(),
        ignore: Vec::new(),
    }
}

/// Every `source.scanned` event in a data dir's journal.
fn scan_summaries(data_dir: &Path) -> Vec<Event> {
    // A data dir with no journal yet has no summaries — which is exactly the
    // state the crash-window test inspects, so this must answer rather than
    // fail.
    if !data_dir.join("journal").exists() {
        return Vec::new();
    }
    Journal::replay_data_dir(data_dir)
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == KIND_SOURCE_SCANNED)
        .collect()
}

/// Start a real daemon on `data_dir` and stop it cleanly — the restart this
/// suite's persistence claims are stated over.
async fn start_and_stop(data_dir: &Path) {
    let handle = daemon::start_with(data_dir, DaemonConfig::default())
        .await
        .expect("daemon start");
    handle.shutdown().await;
}

// ------------------------------------------------------- the CLI surface

/// guard-map: `sgt knowledge add` writes a `[[knowledge]]` entry the parser
/// accepts, `sgt knowledge list` reads it back, and a second add under the
/// same name is refused rather than silently merged.
///
/// Mutation this kills: dropping `knowledge_names`' duplicate check in
/// `domain::manifest::add_knowledge` (two entries would then share a name,
/// and every derived row would key onto one of two sources at random).
#[test]
fn knowledge_add_declares_a_source_and_list_reads_it_back() {
    let estate = TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    let notes = knowledge_tree(&[("README.md", "# Notes\n")]);

    run(estate.path(), &data_dir, &["init"]).assert_ok("init");
    run(
        estate.path(),
        &data_dir,
        &[
            "knowledge",
            "add",
            "notes",
            notes.path().to_str().expect("utf8"),
            "--ignore",
            "*.log",
            "--ignore",
            "drafts/**",
        ],
    )
    .assert_ok("knowledge add");

    let listed = run(estate.path(), &data_dir, &["--json", "knowledge", "list"]);
    listed.assert_ok("knowledge list");
    let sources = listed.json();
    let entry = &sources["knowledge"][0];
    assert_eq!(entry["name"], "notes");
    assert_eq!(entry["path"], notes.path().to_str().expect("utf8"));
    assert_eq!(entry["ignore"][0], "*.log");
    assert_eq!(entry["ignore"][1], "drafts/**");

    // Declaring a knowledge source is a manifest edit and nothing else: no
    // mount, no clone, no directory created (A1-03).
    assert!(!estate.path().join("repos/notes").exists());

    let again = run(
        estate.path(),
        &data_dir,
        &[
            "knowledge",
            "add",
            "notes",
            notes.path().to_str().expect("utf8"),
        ],
    );
    again.assert_fails("a duplicate name");
    assert!(
        again.stderr.contains("already declared"),
        "the refusal must name the conflict: {}",
        again.stderr
    );
}

/// guard-map: F9's containment refusal reaches the operator through the CLI
/// with its own diagnostic, and the manifest is left untouched by the
/// refused edit.
///
/// Mutation this kills: dropping the containment loop in
/// `domain::estate::resolve_knowledge` — the add would then succeed and the
/// estate would index its own repository mount as outside evidence.
#[test]
fn knowledge_add_refuses_a_path_inside_the_estates_own_territory() {
    let estate = TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), &data_dir, &["init"]).assert_ok("init");

    let upstream = TempDir::new().expect("tempdir");
    support::init_repo(upstream.path());
    run(
        estate.path(),
        &data_dir,
        &[
            "repo",
            "add",
            "api",
            "--origin",
            upstream.path().to_str().expect("utf8"),
        ],
    )
    .assert_ok("repo add");

    let before = std::fs::read_to_string(estate.path().join("sergeant.toml")).expect("read");
    let refused = run(
        estate.path(),
        &data_dir,
        &["knowledge", "add", "docs", "repos/api"],
    );
    refused.assert_fails("a knowledge path inside a repository mount");
    assert!(
        refused.stderr.contains("read-only evidence") && refused.stderr.contains("api"),
        "the refusal must name the rule and the mount: {}",
        refused.stderr
    );
    assert_eq!(
        std::fs::read_to_string(estate.path().join("sergeant.toml")).expect("read"),
        before,
        "a refused edit must not touch the real manifest"
    );
}

// -------------------------------------------------- the scanner, end to end

/// guard-map: one completed scan writes exactly one `source.scanned` summary
/// and a full set of rows; a re-scan of unchanged bytes writes neither;
/// editing a byte produces a new generation and evicts the old one with an
/// explicit coverage row.
///
/// Mutation this kills: dropping the `content_key` comparison in
/// `AtlasDb::stage_scan` (every scan would then churn a new generation and
/// evict a perfectly good one, violating ruling §4's "evicted only when
/// source bytes changed").
#[test]
fn a_scan_records_once_reuses_an_unchanged_generation_and_evicts_a_changed_one() {
    let data = TempDir::new().expect("tempdir");
    let notes = knowledge_tree(&[
        ("index.md", "# Index\n\nbody\n\n## Section\n\nmore\n"),
        ("plain.txt", "no structure here\n"),
        (".env", "SECRET=hunter2\n"),
    ]);
    let source = source("notes", notes.path());

    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let mut journal = Journal::open(data.path()).expect("journal");

    let first = scan_and_record(&mut db, &mut journal, &source, None).expect("scan");
    let ScanRecord::Recorded {
        generation_id: first_generation,
        evicted,
        ..
    } = first
    else {
        panic!("the first scan of a source must record a generation, got {first:?}");
    };
    assert_eq!(evicted, None, "nothing to supersede on a first scan");
    assert_eq!(scan_summaries(data.path()).len(), 1, "one summary per scan");

    // Units are there, and they carry provenance back to the original bytes.
    let units = db.units("notes", 100).expect("units");
    assert!(
        units
            .iter()
            .any(|u| u.relative_path == "index.md" && u.kind == UnitKind::Document)
    );
    assert!(
        units
            .iter()
            .any(|u| u.title.as_deref() == Some("Section") && u.kind == UnitKind::Section)
    );
    assert!(
        !units.iter().any(|u| u.body.contains("hunter2")),
        "an excluded file's bytes reached a unit"
    );
    let counts = db.coverage_counts("notes").expect("coverage");
    assert_eq!(counts.get(Coverage::Indexed.as_str()), Some(&2));
    assert_eq!(counts.get(Coverage::Excluded.as_str()), Some(&1));

    // A re-scan of unchanged bytes: no new generation, no new summary, no
    // eviction. Ruling §4 stated as behavior.
    let again = scan_and_record(&mut db, &mut journal, &source, None).expect("rescan");
    assert!(
        matches!(&again, ScanRecord::Unchanged { generation_id, .. } if *generation_id == first_generation),
        "an unchanged source must reuse its generation, got {again:?}"
    );
    assert_eq!(
        scan_summaries(data.path()).len(),
        1,
        "an unchanged re-scan must not journal a second summary"
    );

    // Edit one byte: a new generation, and the old one evicted with a row
    // that says so rather than simply vanishing.
    std::fs::write(notes.path().join("index.md"), "# Index\n\nedited\n").expect("write");
    let third = scan_and_record(&mut db, &mut journal, &source, None).expect("rescan");
    let ScanRecord::Recorded {
        generation_id: second_generation,
        evicted,
        summary_event_id,
        ..
    } = third
    else {
        panic!("a changed source must record a new generation");
    };
    assert_ne!(second_generation, first_generation);
    assert_eq!(evicted.as_deref(), Some(first_generation.as_str()));
    assert_eq!(scan_summaries(data.path()).len(), 2);
    assert_eq!(
        scan_summaries(data.path())[1].id,
        summary_event_id,
        "the record must name the summary that completed it"
    );

    let eviction = db
        .coverage("notes", 100)
        .expect("coverage")
        .into_iter()
        .find(|c| c.row.status == Coverage::GenerationEvicted)
        .expect("an eviction leaves an explicit coverage row");
    assert_eq!(eviction.generation_id, first_generation);
    assert!(
        eviction
            .row
            .detail
            .as_deref()
            .is_some_and(|d| d.contains("source bytes changed")),
        "the eviction must say why: {eviction:?}"
    );
    // And the superseded generation's units are gone, not merged in beside
    // the new ones.
    let units = db.units("notes", 100).expect("units");
    assert!(units.iter().all(|u| !u.body.contains("Section")));
}

/// guard-map: the one journal summary is compact and carries A1 §3's
/// provenance answers — source, generation, content key, counts, extractor
/// identities — and does **not** carry a per-path list.
///
/// Mutation this kills: journaling per-file events (the journal would become
/// a slower second copy of the coverage table, and F1's "one compact summary"
/// would be false).
#[test]
fn the_scan_summary_is_one_compact_event_with_provenance_and_no_path_list() {
    let data = TempDir::new().expect("tempdir");
    let notes = knowledge_tree(&[
        ("a.md", "# A\n"),
        ("b.md", "# B\n"),
        ("c.txt", "c\n"),
        ("secret.pem", "-----BEGIN\n"),
    ]);
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let mut journal = Journal::open(data.path()).expect("journal");
    scan_and_record(&mut db, &mut journal, &source("notes", notes.path()), None).expect("scan");

    let summaries = scan_summaries(data.path());
    assert_eq!(summaries.len(), 1);
    let payload = &summaries[0].payload;
    assert_eq!(payload["source"], "notes");
    assert_eq!(payload["source_kind"], "local_knowledge");
    assert_eq!(payload["authority_class"], "estate_readonly");
    assert!(
        payload["generation"]
            .as_str()
            .is_some_and(|g| !g.is_empty())
    );
    assert!(
        payload["content_key"]
            .as_str()
            .is_some_and(|k| k.len() == 64)
    );
    assert_eq!(payload["files"], 3);
    assert_eq!(payload["coverage"]["indexed"], 3);
    assert_eq!(payload["coverage"]["excluded"], 1);
    let extractors: Vec<&str> = payload["extractors"]
        .as_array()
        .expect("extractors")
        .iter()
        .map(|v| v.as_str().expect("str"))
        .collect();
    assert_eq!(extractors, vec!["markdown/v1", "text/v1"]);

    // Compact: the summary names counts, not paths. The unit-level detail
    // lives in the table that can be queried.
    let rendered = serde_json::to_string(payload).expect("render");
    for path in ["a.md", "b.md", "c.txt", "secret.pem"] {
        assert!(
            !rendered.contains(path),
            "the summary carries a path list ({path}): {rendered}"
        );
    }
}

/// guard-map: a declared `[[knowledge]]` source converts straight into what
/// the scanner walks, `ignore` globs included — so the manifest and the
/// acquisition boundary can never disagree about which paths are in scope.
#[test]
fn a_declared_source_scans_exactly_what_the_manifest_says() {
    let estate = TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    let notes = knowledge_tree(&[
        ("keep.md", "# Keep\n"),
        ("drafts/wip.md", "# WIP\n"),
        ("run.log", "noise\n"),
    ]);
    run(estate.path(), &data_dir, &["init"]).assert_ok("init");
    run(
        estate.path(),
        &data_dir,
        &[
            "knowledge",
            "add",
            "notes",
            notes.path().to_str().expect("utf8"),
            "--ignore",
            "*.log",
            "--ignore",
            "drafts/**",
        ],
    )
    .assert_ok("knowledge add");

    let estate_config = sergeant_rs::domain::estate::Estate::from_config_structural(
        &estate.path().join("sergeant.toml"),
    )
    .expect("parse");
    let declared = KnowledgeSource::from(&estate_config.knowledge[0]);
    let scan = scan_local_knowledge(&declared).expect("scan");

    let indexed: Vec<&str> = scan
        .coverage
        .iter()
        .filter(|r| r.status == Coverage::Indexed)
        .filter_map(|r| r.path.as_deref())
        .collect();
    assert_eq!(indexed, vec!["keep.md"]);
    let excluded: Vec<&str> = scan
        .coverage
        .iter()
        .filter(|r| r.status == Coverage::Excluded)
        .filter_map(|r| r.path.as_deref())
        .collect();
    // `drafts/**` names the directory's *contents*, so each excluded file
    // gets its own row (the directory itself is merely `discovered`). A bare
    // `drafts` would have refused the directory once and not descended —
    // both spellings exclude the same bytes, and both say so out loud.
    assert_eq!(excluded, vec!["drafts/wip.md", "run.log"]);
    assert_eq!(
        scan.coverage
            .iter()
            .find(|r| r.path.as_deref() == Some("drafts"))
            .expect("the directory itself is still reported")
            .status,
        Coverage::Discovered
    );
}

// ------------------------------------------------------------ F1: persistence

/// guard-map (**F1's regression test, moved here from X1 with its writer**):
/// scanned source facts survive real daemon restarts, while the operations
/// projection is refolded from the journal on every one of them. The two
/// rebuild disciplines, checked side by side in one test.
///
/// Mutation this kills: treating Atlas the way `Analytics::begin_rebuild`
/// treats its own file — deleting it on start, or putting it under the
/// disposable `projections/` directory. Either change loses the rows this
/// asserts are still there, and no journal replay brings them back.
#[tokio::test]
async fn source_facts_survive_a_real_daemon_restart() {
    let data = support::DataDir::new();
    let notes = knowledge_tree(&[("index.md", "# Index\n\nbody\n\n## Deep\n\nmore\n")]);

    let generation = {
        let mut db = AtlasDb::open(data.path()).expect("atlas");
        let mut journal = Journal::open(data.path()).expect("journal");
        let record = scan_and_record(&mut db, &mut journal, &source("notes", notes.path()), None)
            .expect("scan");
        let ScanRecord::Recorded { generation_id, .. } = record else {
            panic!("expected a recorded generation");
        };
        generation_id
    };
    let before = {
        let db = AtlasDb::open(data.path()).expect("atlas");
        db.units("notes", 100).expect("units")
    };
    assert!(!before.is_empty(), "the fixture must produce units to lose");

    // Two full daemon lifecycles over the same data dir.
    start_and_stop(data.path()).await;
    start_and_stop(data.path()).await;

    let db = AtlasDb::open(data.path()).expect("atlas after restart");
    assert_eq!(
        db.units("notes", 100).expect("units"),
        before,
        "source units must survive a daemon restart — they are derived from \
         source bytes, and no journal replay reproduces them"
    );
    let still = db
        .confirmed_generation("notes")
        .expect("generation")
        .expect("a confirmed generation must survive");
    assert_eq!(still.id, generation);
    assert!(
        db.coverage_counts("notes")
            .expect("coverage")
            .contains_key(Coverage::Indexed.as_str())
    );

    // The contrast, in the same test so the two disciplines cannot silently
    // converge: the operations projection was rebuilt from the journal, and
    // deleting its whole directory loses nothing.
    let projections = sergeant_rs::runtime::analytics::projections_dir(data.path());
    assert!(projections.exists(), "the projection rebuilt on start");
    std::fs::remove_dir_all(&projections).expect("the projection is disposable");
    start_and_stop(data.path()).await;
    assert!(projections.exists(), "and rebuilt again from the journal");
    assert_eq!(
        AtlasDb::open(data.path())
            .expect("atlas")
            .units("notes", 100)
            .expect("units"),
        before,
        "deleting the disposable projection must not touch Atlas"
    );
    let _ = Analytics::begin_rebuild(data.path()).expect("the projection still opens");
    data.reap();
}

/// guard-map (**F1's crash-mid-scan variant**): a process that dies between
/// committing a scan's rows and journaling its `source.scanned` summary
/// leaves *neither* reported — the rows exist for a moment, no read path can
/// see them, and startup reconciliation evicts them with an explicit
/// coverage row.
///
/// The kill is simulated exactly where the window is: `stage_scan` runs (its
/// transaction commits), and then nothing else does. Reopening the store is
/// the restart.
///
/// Mutation this kills: promoting a generation to `confirmed` inside
/// `stage_scan`, or letting a read path see a provisional one. Either change
/// makes a half-finished scan answer as coverage, which is precisely what
/// F1's crash-window rule forbids.
#[tokio::test]
async fn a_crash_between_the_rows_and_the_summary_reports_neither() {
    let data = support::DataDir::new();
    let notes = knowledge_tree(&[("a.md", "# A\n"), ("b.md", "# B\n")]);
    let source = source("notes", notes.path());

    let staged = {
        let mut db = AtlasDb::open(data.path()).expect("atlas");
        let scan = scan_local_knowledge(&source).expect("scan");
        // Step 1 only. The process "dies" here: no journal append, no
        // confirmation.
        match db.stage_scan(&scan).expect("stage") {
            sergeant_rs::runtime::atlas::db::ScanCommit::Staged { generation_id } => generation_id,
            other => panic!("expected a staged generation, got {other:?}"),
        }
    };

    // Even before any restart, nothing reads a provisional generation.
    {
        let db = AtlasDb::open(data.path()).expect("atlas");
        // (Opening already reconciled it — assert on the state below.)
        assert!(
            db.confirmed_generation("notes")
                .expect("generation")
                .is_none(),
            "a generation with no summary must never answer as confirmed"
        );
    }
    assert!(
        scan_summaries(data.path()).is_empty(),
        "the fixture must not have journaled a summary"
    );

    // The restart: a real daemon start over the same data dir runs Atlas's
    // reconciliation.
    start_and_stop(data.path()).await;

    let db = AtlasDb::open(data.path()).expect("atlas after restart");
    assert_eq!(
        db.generation_states().expect("states").get(&staged),
        Some(&"evicted".to_string()),
        "a generation with rows and no summary must be evicted at startup"
    );
    assert!(
        db.units("notes", 100).expect("units").is_empty(),
        "a half-scan's units must not survive reconciliation"
    );
    // Neither-reported, and *reported as* neither: an explicit eviction row,
    // never a silent gap (ruling §4).
    let rows = db.coverage("notes", 100).expect("coverage");
    let eviction = rows
        .iter()
        .find(|c| c.row.status == Coverage::GenerationEvicted)
        .expect("reconciliation must leave an explicit eviction row");
    assert_eq!(eviction.generation_id, staged);
    assert!(
        eviction
            .row
            .detail
            .as_deref()
            .is_some_and(|d| d.contains("no source.scanned summary")),
        "the eviction must name the crash window: {eviction:?}"
    );
    assert!(
        !rows.iter().any(|c| c.row.status == Coverage::Indexed),
        "a half-scan must not report indexed coverage"
    );

    // And the source is scannable again afterwards: reconciliation clears the
    // window, it does not poison the source.
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let mut journal = Journal::open(data.path()).expect("journal");
    let record = scan_and_record(&mut db, &mut journal, &source, None).expect("rescan");
    assert!(matches!(record, ScanRecord::Recorded { .. }), "{record:?}");
    assert!(!db.units("notes", 100).expect("units").is_empty());
    data.reap();
}

/// guard-map: Atlas's file lives beside the journal, **not** inside the
/// directory whose documented contract is that deleting it loses nothing.
///
/// Mutation this kills: moving Atlas under `projections/` — an acceptance
/// test elsewhere deletes that directory wholesale and asserts nothing was
/// lost, which would then be false.
#[test]
fn the_atlas_store_is_not_inside_the_disposable_projection_directory() {
    let data = TempDir::new().expect("tempdir");
    let db = AtlasDb::open(data.path()).expect("atlas");
    let path: PathBuf = db.path().to_path_buf();
    assert!(path.starts_with(data.path()));
    assert!(
        !path.starts_with(sergeant_rs::runtime::analytics::projections_dir(
            data.path()
        )),
        "{} must not live under the disposable projections directory",
        path.display()
    );
}

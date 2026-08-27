//! S3 X3b acceptance: language-aware extraction, wired onto X3a's plumbing.
//!
//! The spike (`x3b_tslp_corpus.rs`) proved the *extractor* — that
//! `runtime::atlas::syntax` produces exactly the hand-verified symbols and
//! imports for a checked-in corpus, and refuses a malformed file outright.
//! That suite stays the extractor's regression suite and this one does not
//! duplicate it. What this suite proves is the **wiring**, and only the
//! claims that could actually stop being true:
//!
//! * **The three tables land with their writer** (empty-table doctrine).
//!   `source.symbols`, `source.occurrences` and `source.edges` hold rows after
//!   a real scan of a real repository, reached through the ordinary
//!   `scan_and_record_estate_git` path — not through a hand-built fixture.
//! * **F7 keys the syntax extraction separately.** A blob read by the
//!   structure extractor and by a grammar is two extractions with two keys,
//!   both derived from the blob OID Git already computed and never from a
//!   second hash of the same bytes.
//! * **F8 counts honestly.** A file a grammar claims is `indexed` rather than
//!   `unsupported`; a file nothing claims is `unsupported` and says so; a file
//!   a grammar claimed and could not parse is `error` and contributes **no**
//!   partial symbol list.
//! * **A1-09: syntax, not semantics.** Every label written is one the grammar
//!   itself produced, and an edge's target is the text the file wrote.
//! * **F6: it runs on the intelligence lane**, over X3a's batched blobs.
//! * **F1/ruling §4: the rows live and die with their generation.**

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::domain::source::{Coverage, KIND_SOURCE_SCANNED, estate_git_key};
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::git::{EstateGitSource, scan_estate_git};
use sergeant_rs::runtime::atlas::lane::scan_estate_git_on_lane;
use sergeant_rs::runtime::atlas::record::{ScanRecord, record_scan, scan_and_record_estate_git};
use sergeant_rs::runtime::atlas::scan::{EDGE_IMPORT, KnowledgeSource, scan_local_knowledge};
use sergeant_rs::runtime::atlas::syntax::SyntaxLanguage;
use sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR;
use sergeant_rs::runtime::engine::Engine;
use sergeant_rs::runtime::git::git;
use sergeant_rs::runtime::journal::Journal;

// ---------------------------------------------------------------- fixtures

/// A repository with one commit holding `files`, and its SHA.
fn repo(files: &[(&str, &str)]) -> (TempDir, PathBuf, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let mount = dir.path().join("mount");
    std::fs::create_dir_all(&mount).expect("mkdir");
    git(&mount, &["init", "--initial-branch=main"]).expect("init");
    git(&mount, &["config", "user.email", "t@example.com"]).expect("email");
    git(&mount, &["config", "user.name", "T"]).expect("name");
    let sha = commit(&mount, files, "one");
    (dir, mount, sha)
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

/// An Atlas store and a journal over one temp data dir.
fn store(data: &Path) -> (AtlasDb, Journal) {
    (
        AtlasDb::open(data).expect("atlas"),
        Journal::open(data).expect("journal"),
    )
}

/// A small polyglot repository: one file per claimed family that has symbols
/// to find, plus one file nothing claims.
const POLYGLOT: &[(&str, &str)] = &[
    (
        "src/main.rs",
        "use std::collections::HashMap;\n\npub struct Counter {\n    hits: u64,\n}\n\n\
         pub fn count() -> u64 {\n    0\n}\n",
    ),
    (
        "tool/run.py",
        "import os\nfrom pathlib import Path\n\n\nclass Runner:\n    def count(self):\n        return 0\n",
    ),
    ("Cargo.toml", "[package]\nname = \"demo\"\n"),
    ("README.md", "# Demo\n\nbody\n\n## Usage\n\nmore\n"),
    (
        "logo.png",
        "\u{feff}not really a png but an unclaimed extension\n",
    ),
];

/// Scan the polyglot repository and record it, returning the store, the
/// generation id, and the mount.
fn recorded_polyglot() -> (TempDir, TempDir, PathBuf, AtlasDb, String) {
    let (repo_dir, mount, sha) = repo(POLYGLOT);
    let data = tempfile::tempdir().expect("data");
    let (mut db, mut journal) = store(data.path());
    let (record, _) =
        scan_and_record_estate_git(&mut db, &mut journal, &source(&mount, &sha), None)
            .expect("record");
    let ScanRecord::Recorded { generation_id, .. } = record else {
        panic!("expected a recorded generation, got {record:?}");
    };
    (repo_dir, data, mount, db, generation_id)
}

// ------------------------------------------- the three tables and their writer

/// **The whole of the empty-table doctrine for this wave**: the three tables
/// arrive with a writer that actually fills them, through the ordinary
/// recording path, from a real repository's Git objects.
#[test]
fn a_recorded_scan_writes_symbols_occurrences_and_edges() {
    let (_repo, _data, _mount, db, _generation) = recorded_polyglot();

    let symbols = db.symbols("product", 500).expect("symbols");
    let occurrences = db.occurrences("product", 500).expect("occurrences");
    let edges = db.edges("product", 500).expect("edges");
    assert!(!symbols.is_empty(), "no symbol rows");
    assert!(!occurrences.is_empty(), "no occurrence rows");
    assert!(!edges.is_empty(), "no edge rows");

    // Rust: the struct and the function, both by the grammar's own labels.
    let rust: Vec<(&str, &str)> = occurrences
        .iter()
        .filter(|o| o.relative_path == "src/main.rs")
        .map(|o| (o.label.as_str(), o.name.as_str()))
        .collect();
    assert_eq!(rust, vec![("struct", "Counter"), ("function", "count")]);

    // Python: the class and its method are what *that* grammar names, and
    // "method" is not invented for it — `symbol_kinds` calls a Python method a
    // function, because the grammar does.
    let python: Vec<(&str, &str)> = occurrences
        .iter()
        .filter(|o| o.relative_path == "tool/run.py")
        .map(|o| (o.label.as_str(), o.name.as_str()))
        .collect();
    assert_eq!(python, vec![("class", "Runner"), ("function", "count")]);

    // TOML and Markdown are claimed too, and by their own grammars.
    assert!(
        occurrences
            .iter()
            .any(|o| o.relative_path == "Cargo.toml" && o.label == "table" && o.name == "package"),
        "{occurrences:#?}"
    );
    assert!(
        occurrences
            .iter()
            .any(|o| o.relative_path == "README.md" && o.label == "heading" && o.name == "Demo"),
        "{occurrences:#?}"
    );

    // Edges are imports, unresolved, exactly as written.
    let mut targets: Vec<(&str, &str)> = edges
        .iter()
        .map(|e| (e.relative_path.as_str(), e.target.as_str()))
        .collect();
    targets.sort_unstable();
    assert_eq!(
        targets,
        vec![
            ("src/main.rs", "std::collections::HashMap"),
            ("tool/run.py", "os"),
            ("tool/run.py", "pathlib"),
        ]
    );
    assert!(
        edges.iter().all(|e| e.kind == EDGE_IMPORT),
        "an edge kind this build cannot derive appeared: {edges:#?}"
    );
}

/// A symbol row is the *index*; an occurrence row is a *site*. Two files
/// defining `count` are one symbol with two occurrences — and that is a
/// syntactic rollup over `(language, label, name)`, never a claim that the two
/// sites define the same thing (A1-09).
#[test]
fn symbols_roll_up_the_sites_that_wrote_them() {
    let (_repo, _data, _mount, db, _generation) = recorded_polyglot();
    let symbols = db.symbols("product", 500).expect("symbols");

    let rust_count = symbols
        .iter()
        .find(|s| s.language == "rust" && s.name == "count")
        .expect("the Rust `count` function");
    assert_eq!(rust_count.label, "function");
    assert_eq!(rust_count.occurrences, 1);

    // Python's `count` is a *different* symbol: the language is part of the
    // identity, so two languages that happen to spell a name the same way are
    // never merged.
    let python_count = symbols
        .iter()
        .find(|s| s.language == "python" && s.name == "count")
        .expect("the Python `count` method");
    assert_eq!(python_count.occurrences, 1);

    // Every symbol's occurrence count matches the sites actually stored.
    let occurrences = db.occurrences("product", 500).expect("occurrences");
    for symbol in &symbols {
        let sites = occurrences
            .iter()
            .filter(|o| {
                o.language == symbol.language && o.label == symbol.label && o.name == symbol.name
            })
            .count() as u64;
        assert_eq!(
            symbol.occurrences, sites,
            "the index disagrees with the sites for {symbol:?}"
        );
    }
}

/// The same name in two files is one symbol and two sites — the property the
/// rollup exists for, stated where a single-file fixture could not show it.
#[test]
fn one_name_in_two_files_is_one_symbol_and_two_occurrences() {
    let (_repo, mount, sha) = repo(&[
        ("a.rs", "pub fn shared() {}\n"),
        ("b/c.rs", "pub fn shared() {}\n"),
    ]);
    let data = tempfile::tempdir().expect("data");
    let (mut db, mut journal) = store(data.path());
    scan_and_record_estate_git(&mut db, &mut journal, &source(&mount, &sha), None).expect("record");

    let symbols = db.symbols("product", 100).expect("symbols");
    let shared: Vec<_> = symbols.iter().filter(|s| s.name == "shared").collect();
    assert_eq!(shared.len(), 1, "{symbols:#?}");
    assert_eq!(shared[0].occurrences, 2);

    let sites: BTreeSet<String> = db
        .occurrences("product", 100)
        .expect("occurrences")
        .into_iter()
        .filter(|o| o.name == "shared")
        .map(|o| o.relative_path)
        .collect();
    assert_eq!(
        sites,
        BTreeSet::from(["a.rs".to_string(), "b/c.rs".to_string()])
    );
}

// ----------------------------------------------------------- F7, the keys

/// **F7 on the syntax half.** The key is the blob OID Git already computed,
/// composed with the *grammar's* identity — a second, separate extraction of
/// the same bytes, with a key of its own. Never a second hash of the blob.
#[test]
fn the_syntax_extraction_is_keyed_on_the_blob_oid_and_the_grammar() {
    let (_repo, _data, mount, db, _generation) = recorded_polyglot();
    let oid = git(&mount, &["rev-parse", "HEAD:README.md"]).expect("rev-parse");

    let readme = db
        .occurrences("product", 500)
        .expect("occurrences")
        .into_iter()
        .find(|o| o.relative_path == "README.md")
        .expect("a README occurrence");
    assert_eq!(
        readme.extractor,
        SyntaxLanguage::Markdown.extractor_identity()
    );
    assert_eq!(
        readme.syntax_key,
        estate_git_key(&oid, &SyntaxLanguage::Markdown.extractor_identity()),
        "the syntax key is the blob OID composed with the grammar's identity"
    );

    // And the structure extraction of the *same blob* has a different key,
    // because it is a different extraction (F7's second input).
    assert_ne!(
        readme.syntax_key,
        estate_git_key(&oid, MARKDOWN_EXTRACTOR),
        "one blob read by two extractors must not share one key"
    );
    let units = db.units("product", 500).expect("units");
    let unit = units
        .iter()
        .find(|u| u.relative_path == "README.md")
        .expect("a README unit");
    assert_eq!(unit.local_key, estate_git_key(&oid, MARKDOWN_EXTRACTOR));
    assert_ne!(unit.local_key, readme.syntax_key);
}

/// The local key space is the one a filesystem source uses, and the same
/// separation holds there — the shared extractor cannot blur the two spaces.
#[test]
fn a_local_knowledge_scan_keys_its_syntax_extraction_locally() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("lib.py"), "def only():\n    return 1\n").expect("write");
    let scan = scan_local_knowledge(&KnowledgeSource {
        name: "notes".to_string(),
        root: dir.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: Default::default(),
    })
    .expect("scan");

    let file = &scan.files[0];
    let syntax = file.syntax.as_ref().expect("python is claimed");
    assert_eq!(syntax.language, "python");
    assert_eq!(
        syntax.syntax_key,
        sergeant_rs::domain::source::local_key(&file.content_hash, &syntax.extractor)
    );
    assert_ne!(
        syntax.syntax_key,
        estate_git_key(&file.content_hash, &syntax.extractor),
        "the two key spaces are domain-separated"
    );
    assert_eq!(syntax.symbols.len(), 1);
    assert_eq!(syntax.symbols[0].name, "only");
}

// -------------------------------------------------------- F8, honest counts

/// F8, both directions at once: a file a grammar claims stops being
/// `unsupported` and becomes `indexed`, and a file **nothing** claims is still
/// `unsupported` — named as such, and counted.
#[test]
fn a_claimed_language_is_indexed_and_an_unclaimed_one_is_still_counted() {
    let (_repo, _data, _mount, db, _generation) = recorded_polyglot();
    let coverage = db.coverage("product", 500).expect("coverage");
    let status = |path: &str| {
        coverage
            .iter()
            .find(|c| c.row.path.as_deref() == Some(path))
            .unwrap_or_else(|| panic!("no coverage row for {path}"))
            .clone()
    };

    // X3a reported `src/main.rs` unsupported, because no extractor claimed
    // `.rs`. A grammar claims it now, and the honest answer changed with it.
    assert_eq!(status("src/main.rs").row.status, Coverage::Indexed);
    assert_eq!(status("Cargo.toml").row.status, Coverage::Indexed);
    assert_eq!(status("tool/run.py").row.status, Coverage::Indexed);

    // The detail names *both* extractors that produced rows, so "which parser
    // produced this?" is answerable from coverage alone (A1 §3).
    let detail = status("src/main.rs").row.detail.expect("detail");
    assert!(detail.contains("text/v1"), "{detail}");
    assert!(
        detail.contains(&SyntaxLanguage::Rust.extractor_identity()),
        "{detail}"
    );

    // And nothing became indexed by accident.
    let unclaimed = status("logo.png");
    assert_eq!(unclaimed.row.status, Coverage::Unsupported);
    let detail = unclaimed.row.detail.expect("detail");
    assert!(detail.contains("grammar"), "{detail}");

    let counts = db.coverage_counts("product").expect("counts");
    assert_eq!(counts.get(Coverage::Indexed.as_str()), Some(&4));
    assert_eq!(counts.get(Coverage::Unsupported.as_str()), Some(&1));
}

/// `.tsx` is the honest gap the spike named: this build has the TypeScript
/// grammar and not the TSX one, so a `.tsx` file is `unsupported` rather than
/// parsed by an almost-right grammar.
#[test]
fn a_language_this_build_does_not_claim_is_unsupported_not_almost_parsed() {
    let (_repo, mount, sha) = repo(&[
        ("app.tsx", "export function App() { return null; }\n"),
        ("app.ts", "export function App(): null { return null; }\n"),
    ]);
    let data = tempfile::tempdir().expect("data");
    let (mut db, mut journal) = store(data.path());
    scan_and_record_estate_git(&mut db, &mut journal, &source(&mount, &sha), None).expect("record");

    let coverage = db.coverage("product", 100).expect("coverage");
    let tsx = coverage
        .iter()
        .find(|c| c.row.path.as_deref() == Some("app.tsx"))
        .expect("a tsx row");
    assert_eq!(tsx.row.status, Coverage::Unsupported);
    assert!(
        !db.occurrences("product", 100)
            .expect("occurrences")
            .iter()
            .any(|o| o.relative_path == "app.tsx"),
        "an unclaimed language produced symbols anyway"
    );
    // The claimed sibling did index, so the refusal is about the grammar and
    // not about the fixture.
    assert!(
        db.occurrences("product", 100)
            .expect("occurrences")
            .iter()
            .any(|o| o.relative_path == "app.ts" && o.name == "App")
    );
}

/// A parse failure is an `error` coverage row and **no partial symbols** — the
/// spike's refusal, carried through the wiring. The structure units the other
/// extractor produced survive, because they are real evidence about real
/// bytes; what is never written is a shorter symbol list nothing can
/// distinguish from a complete one.
#[test]
fn a_file_a_grammar_cannot_parse_is_an_error_row_with_no_partial_symbols() {
    let (_repo, mount, sha) = repo(&[
        ("good.rs", "pub fn fine() {}\n"),
        ("broken.rs", "pub fn ok() {}\npub fn broken( {\n"),
    ]);
    let data = tempfile::tempdir().expect("data");
    let (mut db, mut journal) = store(data.path());
    scan_and_record_estate_git(&mut db, &mut journal, &source(&mount, &sha), None).expect("record");

    let coverage = db.coverage("product", 100).expect("coverage");
    let broken = coverage
        .iter()
        .find(|c| c.row.path.as_deref() == Some("broken.rs"))
        .expect("a broken.rs row");
    assert_eq!(broken.row.status, Coverage::Error);
    let detail = broken.row.detail.clone().expect("detail");
    assert!(detail.contains("parse failed"), "{detail}");
    assert!(
        detail.contains(&SyntaxLanguage::Rust.extractor_identity()),
        "the failing extractor is named: {detail}"
    );

    let occurrences = db.occurrences("product", 100).expect("occurrences");
    assert!(
        !occurrences.iter().any(|o| o.relative_path == "broken.rs"),
        "a failed parse wrote a partial symbol list: {occurrences:#?}"
    );
    assert!(
        occurrences.iter().any(|o| o.relative_path == "good.rs"),
        "one bad file stopped a good one from indexing"
    );
    // The bytes were still acquired and their structure unit written: the
    // failure is one extractor's, not the resource's.
    assert!(
        db.units("product", 100)
            .expect("units")
            .iter()
            .any(|u| u.relative_path == "broken.rs")
    );

    // And the journal summary names only extractors that produced rows.
    let extractors: BTreeSet<String> = journal
        .replay_from_floor()
        .expect("replay")
        .filter_map(Result::ok)
        .filter(|e| e.kind == KIND_SOURCE_SCANNED)
        .flat_map(|e| {
            e.payload["extractors"]
                .as_array()
                .expect("extractors")
                .iter()
                .map(|v| v.as_str().expect("string").to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(extractors.contains(&SyntaxLanguage::Rust.extractor_identity()));
}

/// The journal summary carries the new counts, so "how much was derived?" is
/// answerable from the trail and not only from the database (F1).
#[test]
fn the_scan_summary_counts_symbols_and_edges() {
    let (_repo, mount, sha) = repo(POLYGLOT);
    let data = tempfile::tempdir().expect("data");
    let (mut db, mut journal) = store(data.path());
    scan_and_record_estate_git(&mut db, &mut journal, &source(&mount, &sha), None).expect("record");

    let summary = journal
        .replay_from_floor()
        .expect("replay")
        .filter_map(Result::ok)
        .find(|e| e.kind == KIND_SOURCE_SCANNED)
        .expect("a summary");
    let symbols = summary.payload["symbols"].as_u64().expect("symbols");
    let edges = summary.payload["edges"].as_u64().expect("edges");
    assert_eq!(
        symbols,
        db.occurrences("product", 500).expect("occurrences").len() as u64
    );
    assert_eq!(edges, db.edges("product", 500).expect("edges").len() as u64);
}

// ---------------------------------------------------------- A1-09, syntax only

/// Every label written is one a grammar's own symbol table produced. Nothing
/// resolves, classifies, or infers — so the set of labels in the store is a
/// subset of the set the extractor can emit, checked rather than asserted in
/// prose.
#[test]
fn every_label_written_is_one_a_grammar_produced() {
    let (_repo, _data, _mount, db, _generation) = recorded_polyglot();
    let allowed: BTreeSet<&'static str> = SyntaxLanguage::ALL
        .iter()
        .flat_map(|language| language.labels())
        .collect();
    for occurrence in db.occurrences("product", 500).expect("occurrences") {
        assert!(
            allowed.contains(occurrence.label.as_str()),
            "label {:?} is not one any grammar in this build emits",
            occurrence.label
        );
    }
    for symbol in db.symbols("product", 500).expect("symbols") {
        assert!(allowed.contains(symbol.label.as_str()), "{symbol:?}");
    }
}

// ------------------------------------------------ F6, on the intelligence lane

/// F6: the extraction that now produces symbols is the same one X3a put on the
/// intelligence lane — it did not quietly acquire a second path to run on.
#[tokio::test]
async fn extraction_on_the_intelligence_lane_produces_the_same_symbols() {
    let (_repo, mount, sha) = repo(POLYGLOT);
    let data = tempfile::tempdir().expect("data");
    let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path())
        .with_intelligence_lane_cap(2);
    let src = source(&mount, &sha);

    let direct = scan_estate_git(&src).expect("direct");
    let on_lane = scan_estate_git_on_lane(&engine, src.clone())
        .await
        .expect("on the lane");
    assert_eq!(on_lane.scan.files, direct.scan.files);
    assert!(
        on_lane.scan.symbol_count() > 0,
        "the lane's consumer produced no symbols"
    );
    assert_eq!(on_lane.scan.symbol_count(), direct.scan.symbol_count());
    assert_eq!(on_lane.scan.edge_count(), direct.scan.edge_count());
    assert_eq!(
        engine.intelligence_lane.available_permits(),
        2,
        "the permit was not returned"
    );

    // And the rows the lane's scan produces record exactly as the direct one's
    // do — one writer, whichever route the scan took.
    let (mut db, mut journal) = store(data.path());
    record_scan(&mut db, &mut journal, &on_lane.scan, None).expect("record");
    assert!(!db.symbols("product", 500).expect("symbols").is_empty());
}

// ------------------------------------- F1 / ruling §4, the generation lifetime

/// Derived syntax rows are a generation's, and an eviction takes them with it —
/// no orphan symbols outliving the world they describe.
#[test]
fn evicting_a_generation_takes_its_syntax_rows_with_it() {
    let (_repo, mount, sha) = repo(&[("a.rs", "pub fn first() {}\n")]);
    let data = tempfile::tempdir().expect("data");
    let (mut db, mut journal) = store(data.path());
    scan_and_record_estate_git(&mut db, &mut journal, &source(&mount, &sha), None).expect("record");
    assert_eq!(db.symbols("product", 100).expect("symbols").len(), 1);

    // A second commit changes the bytes, so ruling §4 evicts the predecessor.
    let next = commit(&mount, &[("a.rs", "pub fn second() {}\n")], "two");
    let (record, _) =
        scan_and_record_estate_git(&mut db, &mut journal, &source(&mount, &next), None)
            .expect("record");
    let ScanRecord::Recorded { evicted, .. } = record else {
        panic!("expected a recorded generation, got {record:?}");
    };
    assert!(evicted.is_some(), "the predecessor was not superseded");

    let names: Vec<String> = db
        .symbols("product", 100)
        .expect("symbols")
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(
        names,
        vec!["second".to_string()],
        "an evicted symbol survived"
    );
    assert_eq!(
        db.occurrences("product", 100).expect("occurrences").len(),
        1
    );
    assert!(
        db.edges("product", 100).expect("edges").is_empty(),
        "no imports were written in either commit"
    );
}

/// F1's persistence rule, extended to this wave's rows: a symbol derived
/// before a restart is still there after one. Atlas's file is not a
/// projection, and these rows are not re-foldable from the journal.
#[test]
fn syntax_rows_survive_reopening_the_store() {
    let (_repo, mount, sha) = repo(&[("a.rs", "use std::io;\npub fn kept() {}\n")]);
    let data = tempfile::tempdir().expect("data");
    {
        let (mut db, mut journal) = store(data.path());
        scan_and_record_estate_git(&mut db, &mut journal, &source(&mount, &sha), None)
            .expect("record");
    }
    let db = AtlasDb::open(data.path()).expect("reopen");
    let symbols = db.symbols("product", 100).expect("symbols");
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "kept");
    assert_eq!(
        db.edges("product", 100).expect("edges")[0].target,
        "std::io"
    );
}

/// F12: every read of these tables is bounded, and a caller cannot ask for
/// "all".
#[test]
fn every_syntax_read_is_bounded() {
    let (_repo, _data, _mount, db, _generation) = recorded_polyglot();
    assert_eq!(db.symbols("product", 1).expect("symbols").len(), 1);
    assert_eq!(db.occurrences("product", 2).expect("occurrences").len(), 2);
    assert!(db.edges("product", 1).expect("edges").len() <= 1);
    // And an absurd request is capped rather than honoured.
    assert!(
        db.occurrences("product", usize::MAX)
            .expect("occurrences")
            .len()
            <= sergeant_rs::runtime::atlas::db::MAX_ROWS
    );
}

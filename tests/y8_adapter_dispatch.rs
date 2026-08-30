//! S4 Y8 acceptance: dispatch, not just adapters (brief-y8-adapter-dispatch.md).
//!
//! S4 built three content adapters (Office, ZIP/archive, mail/eml)
//! and a supervised worker transport to run them in — and wired none of it
//! to a real scan. `sgt intelligence scan` walked a folder, saw a `.docx`,
//! and did not extract it: `scan.rs`'s routing table never claimed the
//! extension, so it never reached [`run_worker`]. This is the wave that
//! fixes it, and this is the proof: a scan of a directory holding one real
//! `.docx`, one real `.zip` and one real `.eml`, through the PRODUCTION
//! worker-enabled walk (the shape [`scan_local_knowledge_on_lane`] actually
//! drives), not an isolated adapter unit test — exactly what the Y7
//! closeout's own sweep already warned an adapter-only proof would miss.
//!
//! [`run_worker`]: sergeant_rs::runtime::atlas::worker::run_worker
//! [`scan_local_knowledge_on_lane`]: sergeant_rs::runtime::atlas::lane::scan_local_knowledge_on_lane

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::domain::event::Event;
use sergeant_rs::domain::source::{Coverage, CoverageRow, KIND_SOURCE_SCANNED};
use sergeant_rs::runtime::atlas::archive::ZIP_EXTRACTOR;
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::lane::scan_local_knowledge_on_lane;
use sergeant_rs::runtime::atlas::mail::MAIL_EXTRACTOR;
use sergeant_rs::runtime::atlas::office::{
    DOCX_EXTRACTOR, EPUB_EXTRACTOR, ODT_EXTRACTOR, PDF_EXTRACTOR, PPTX_EXTRACTOR, RTF_EXTRACTOR,
    XLSX_EXTRACTOR,
};
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{
    KnowledgeSource, SourceScan, scan_local_knowledge_with_worker,
};
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::atlas::worker::WorkerRuntime;
use sergeant_rs::runtime::engine::Engine;
use sergeant_rs::runtime::journal::Journal;

/// The real worker binary Cargo built alongside this test binary — same
/// spelling `tests/y1_worker_transport.rs` and its siblings already use.
const SGT_ATLAS_WORKER: &str = env!("CARGO_BIN_EXE_sgt-atlas-worker");

fn worker() -> WorkerRuntime {
    WorkerRuntime {
        program: PathBuf::from(SGT_ATLAS_WORKER),
        deadline: Duration::from_secs(20),
    }
}

/// One of this repo's own hand-verified fixtures — never a fixture authored
/// for this test, which is exactly the substitution the brief warns an
/// isolated adapter test could get away with.
fn fixture(relative: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

fn coverage_row<'a>(scan: &'a SourceScan, relative_path: &str) -> &'a CoverageRow {
    scan.coverage
        .iter()
        .find(|r| r.path.as_deref() == Some(relative_path))
        .unwrap_or_else(|| panic!("no coverage row for {relative_path:?}: {:?}", scan.coverage))
}

/// **S6, end to end: the ten formats that were parsed but never routed.**
///
/// Owner ruling `twelve-formats-is-0.3.0-criteria-2026-08-30` (J4): *"1/12 is
/// a failure of 0.3.0 completion criteria for estate intelligence."* The
/// parser was linked into this binary the whole time; only the routing table
/// was one format wide, behind an honest-looking comment.
///
/// This is the proof at the layer that matters — a REAL local-knowledge scan,
/// through the REAL `sgt-atlas-worker` subprocess, over this repo's own
/// hand-verified fixtures — not `office::extractor_for` answering `Some` in a
/// unit test. It asserts all four kinds of answer the ruling demanded:
///
/// 1. six newly routed formats reach `Indexed` with real text units;
/// 2. `.csv` is NOT among them — it stays a DATASET, read in place (A1 §6.4,
///    A1-13). Routing it to the document lane is precisely what §6.4 forbids
///    by name, so its absence from `scan.files` is an assertion here, not an
///    omission;
/// 3. a scanned PDF is a NAMED coverage gap pointing at OCR, never silence
///    and never a false empty extraction (A1 §15);
/// 4. an encrypted document is its OWN named gap, with detail text distinct
///    from a malformed one — the normalizer's failure vocabulary reaching a
///    coverage row, which is where an operator actually reads it.
#[test]
fn a_real_scan_indexes_every_newly_routed_office_format_and_keeps_csv_relational() {
    let source_root = TempDir::new().expect("source root");
    let office = |name: &str| format!("anydoc_corpus/office_fixtures/{name}");
    for (path, source) in [
        ("slides.pptx", office("15-pptx-slides.pptx")),
        ("budget.xlsx", office("18-xlsx-sheet.xlsx")),
        ("notes.odt", office("10-odt-headings.odt")),
        ("memo.rtf", office("07-rtf-plain.rtf")),
        ("book.epub", office("20-epub-chapters.epub")),
        ("paper.pdf", office("22-pdf-text.pdf")),
        ("scanned.pdf", office("23-pdf-scanned-needs-ocr.pdf")),
        ("locked.odt", office("14-odt-encrypted.odt")),
        (
            "broken.odt",
            office("13-odt-malformed-unclosed-element.odt"),
        ),
    ] {
        std::fs::write(source_root.path().join(path), fixture(&source))
            .unwrap_or_else(|e| panic!("write {path}: {e}"));
    }
    // The twelfth format. A dataset, not a document — and written from the
    // same scan so the two lanes are decided by one walk, not by two tests
    // that could disagree.
    std::fs::write(
        source_root.path().join("tickets.csv"),
        "id,short_description\n1,printer offline\n2,vpn drops\n",
    )
    .expect("write csv");

    let source = KnowledgeSource {
        name: "formats".to_string(),
        root: source_root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::none(),
    };
    let scan = scan_local_knowledge_with_worker(&source, &worker()).expect("scan");

    // ---------------------------------------------- (1) the six that index
    for (path, extractor) in [
        ("slides.pptx", PPTX_EXTRACTOR),
        ("budget.xlsx", XLSX_EXTRACTOR),
        ("notes.odt", ODT_EXTRACTOR),
        ("memo.rtf", RTF_EXTRACTOR),
        ("book.epub", EPUB_EXTRACTOR),
        ("paper.pdf", PDF_EXTRACTOR),
    ] {
        let row = coverage_row(&scan, path);
        assert_eq!(
            row.status,
            Coverage::Indexed,
            "{path} must be Indexed through the real worker, not {row:?}"
        );
        assert!(
            scan.extractors.contains(extractor),
            "{extractor} missing from {:?}",
            scan.extractors
        );
        let file = scan
            .files
            .iter()
            .find(|f| f.relative_path == path)
            .unwrap_or_else(|| panic!("{path} did not land in source.files"));
        assert_eq!(
            file.extractor, extractor,
            "{path} landed under the wrong identity"
        );
        assert!(
            file.units.iter().any(|u| !u.text.trim().is_empty()),
            "{path} must carry real text, not an empty success: {:?}",
            file.units
        );
    }
    // The docx identity is deliberately NOT in this scan — no `.docx` was
    // written — so the six above are new routing, not the pre-existing one
    // format passing under a different name.
    assert!(
        !scan.extractors.contains(DOCX_EXTRACTOR),
        "this scan holds no .docx; {DOCX_EXTRACTOR} must not appear: {:?}",
        scan.extractors
    );

    // Hand-verifiable content, not merely "non-empty": the fixtures' own
    // text, from `MANIFEST.md`'s S6 table.
    let text_of = |path: &str| {
        scan.files
            .iter()
            .find(|f| f.relative_path == path)
            .map(|f| f.units[0].text.clone())
            .unwrap_or_else(|| panic!("{path} missing"))
    };
    assert_eq!(
        text_of("budget.xlsx"),
        "Item | Cost\nWidget | 10\nGadget | 20",
        "a spreadsheet reaches retrieval as table text (A1 §6.3, OWNER-02)"
    );
    assert!(text_of("slides.pptx").contains("Pptx Slide Two"));
    assert!(text_of("book.epub").contains("Epub Chapter Two"));
    assert!(text_of("paper.pdf").contains("Pdf Fixture Heading"));

    // --------------------------------------- (2) csv stays in the other lane
    assert!(
        scan.files.iter().all(|f| f.relative_path != "tickets.csv"),
        "csv must not be normalized into a document (A1 §6.4: a 100k-ticket export must NOT \
         be normalized into 100k Markdown documents just to make it searchable): {:?}",
        scan.files
            .iter()
            .map(|f| &f.relative_path)
            .collect::<Vec<_>>()
    );
    assert!(
        scan.datasets
            .iter()
            .any(|d| d.relative_path == "tickets.csv"),
        "csv must land in the relational lane instead (A1-13): {:?}",
        scan.datasets
    );

    // ---------------------------------- (3) the scanned PDF is a NAMED gap
    let scanned = coverage_row(&scan, "scanned.pdf");
    assert_ne!(
        scanned.status,
        Coverage::Indexed,
        "an image-only PDF must never index as a document with no text: {scanned:?}"
    );
    let detail = scanned
        .detail
        .as_deref()
        .expect("a coverage gap without detail is silence, which A1 §15 forbids");
    assert!(
        detail.contains("OCR"),
        "the gap must name OCR — the one capability deliberately outside 0.3.0: {detail}"
    );
    assert!(
        detail.contains("page 1 of 1"),
        "and name WHICH pages need it: {detail}"
    );
    assert!(
        scan.files.iter().all(|f| f.relative_path != "scanned.pdf"),
        "a named gap must not also land as a file with zero units"
    );

    // ------------------------- (4) encrypted is its own gap, not "malformed"
    let locked = coverage_row(&scan, "locked.odt");
    let broken = coverage_row(&scan, "broken.odt");
    let locked_detail = locked.detail.as_deref().expect("encrypted needs detail");
    let broken_detail = broken.detail.as_deref().expect("malformed needs detail");
    assert!(
        locked_detail.contains("encrypted"),
        "a password-protected file is a named gap, not a parse failure: {locked_detail}"
    );
    assert_ne!(
        locked_detail, broken_detail,
        "\"locked\" and \"damaged\" are different problems with different remedies and must \
         not share coverage detail text"
    );
}

/// The one `source.scanned` journal summary a completed scan writes —
/// `tests/x2_knowledge_sources.rs`'s own helper, same shape (R2).
fn scan_summaries(data_dir: &Path) -> Vec<Event> {
    if !data_dir.join("journal").exists() {
        return Vec::new();
    }
    Journal::replay_data_dir(data_dir)
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == KIND_SOURCE_SCANNED)
        .collect()
}

/// **The defect, proven live, and its fix.**
///
/// Before S4 Y8: `scan.rs`'s `claims_for` never claimed `.docx`/`.zip`/
/// `.eml`, so this scan reported all three `unsupported` — no worker ever
/// ran, `run_worker_on_lane`/`run_worker` had zero callers from a real walk,
/// and the recorded generation's extractor set never named
/// the Office, ZIP and mail extractor identities. Watched
/// red against that code before this wave's dispatch wiring landed.
///
/// After: the walk routes each resource through the real supervised worker
/// ([`run_worker`]) and daemon-side [`validate_batch`] AUTHORITY, exactly as
/// Y1 designed and Y2/Y3/Y4 built the three adapters to be run — and the
/// recorded generation (through [`record_scan`]'s real three-step
/// stage/journal/confirm discipline, not a shortcut) carries the proof.
///
/// [`run_worker`]: sergeant_rs::runtime::atlas::worker::run_worker
/// [`validate_batch`]: sergeant_rs::runtime::atlas::worker::validate_batch
#[test]
fn a_real_scan_dispatches_docx_zip_and_eml_through_the_worker_and_the_recorded_generation_carries_the_proof()
 {
    let source_root = TempDir::new().expect("source root");
    std::fs::write(
        source_root.path().join("report.docx"),
        fixture("anydoc_corpus/docx_fixtures/01-plain-headings-paragraphs.docx"),
    )
    .expect("write docx");
    std::fs::write(
        source_root.path().join("bundle.zip"),
        fixture("zip_corpus/zip_fixtures/01-plain-and-directory.zip"),
    )
    .expect("write zip");
    std::fs::write(
        source_root.path().join("message.eml"),
        fixture("mail_corpus/03-with-attachment.eml"),
    )
    .expect("write eml");

    let source = KnowledgeSource {
        name: "mixed".to_string(),
        root: source_root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::none(),
    };
    let scan = scan_local_knowledge_with_worker(&source, &worker()).expect("scan");

    // -------------------------------------------------- the extractor set
    // All three adapter identities actually ran — through the worker, not
    // merely present in a routing table nothing calls.
    assert!(
        scan.extractors.contains(DOCX_EXTRACTOR),
        "docx extractor missing from {:?}",
        scan.extractors
    );
    assert!(
        scan.extractors.contains(ZIP_EXTRACTOR),
        "zip extractor missing from {:?}",
        scan.extractors
    );
    assert!(
        scan.extractors.contains(MAIL_EXTRACTOR),
        "mail extractor missing from {:?}",
        scan.extractors
    );

    // No adapter is silently unsupported — the very failure mode this scan
    // exhibited before this wave.
    for name in ["report.docx", "bundle.zip", "message.eml"] {
        let row = coverage_row(&scan, name);
        assert_eq!(
            row.status,
            Coverage::Indexed,
            "{name} must be Indexed through the real worker, not {row:?}"
        );
    }

    // ----------------------------------------------------- document units
    // The docx produced real, non-empty document/section units — not a
    // placeholder and not zero units silently reported as success.
    let docx = scan
        .files
        .iter()
        .find(|f| f.relative_path == "report.docx")
        .expect("docx file landed in source.files");
    assert!(
        !docx.units.is_empty(),
        "a real .docx must produce document units"
    );
    assert!(
        docx.units.iter().any(|u| !u.text.trim().is_empty()),
        "docx units must carry real text: {:?}",
        docx.units
    );

    // The mail message's own body is a document unit too — mail lands both
    // units (its own body) and children (its attachment) in one message.
    let eml = scan
        .files
        .iter()
        .find(|f| f.relative_path == "message.eml")
        .expect("eml file landed in source.files");
    assert!(
        !eml.units.is_empty(),
        "a real .eml must produce at least its text-body unit"
    );
    assert!(
        eml.units.iter().any(|u| u.text.contains("attached report")),
        "the eml's real text body must reach a unit: {:?}",
        eml.units
    );

    // ------------------------------------ what a worker-landed unit carries
    // S5 closeout (F-AC-02). The landing path used to build every
    // worker-routed unit with `heading_level: None, title: None` and to drop
    // the adapter's native `coordinate` entirely, so the two families that
    // can only be addressed by that coordinate landed unaddressable. This is
    // the pin at the exact place it was lost: the scan, before any store or
    // index is involved.
    assert!(
        docx.units
            .iter()
            .any(|u| u.coordinate.is_some() && u.title.is_some()),
        "an Office section unit is not byte-recoverable, so its native coordinate and heading \
         title are the whole of its provenance: {:?}",
        docx.units
    );
    let mut bodies: Vec<&str> = eml
        .units
        .iter()
        .map(|u| {
            assert_eq!(
                u.title.as_deref(),
                Some("Report attached"),
                "A1 §6.5 keeps a message's subject; a body unit's title is where it lands: {u:?}"
            );
            u.coordinate
                .as_deref()
                .unwrap_or_else(|| panic!("a mail body unit must name which body it is: {u:?}"))
        })
        .collect();
    bodies.sort();
    assert_eq!(
        bodies,
        vec!["text-body"],
        "this fixture has one body, and the native coordinate names it"
    );

    // A ZIP's own body carries no text unit of its own (its content is its
    // children) — the honest empty [`atlas_worker.rs`]'s own doc states,
    // not a bug.
    let zip = scan
        .files
        .iter()
        .find(|f| f.relative_path == "bundle.zip")
        .expect("zip file landed in source.files");
    assert!(
        zip.units.is_empty(),
        "a ZIP container has no body unit of its own"
    );

    // ------------------------------------------------------------ children
    // S5 W7: a declared child is a RESOURCE, not a name in its container's
    // coverage detail. Each one lands as its own `source.files` row, at its
    // own composed path, carrying the parent coordinate A1 §6.6 requires an
    // expanded entry to preserve.
    for (parent, entry) in [
        ("bundle.zip", "readme.txt"),
        ("bundle.zip", "notes/a.md"),
        ("bundle.zip", "notes/b.txt"),
        ("message.eml", "report.txt"),
    ] {
        let composed = format!("{parent}!/{entry}");
        let child = scan
            .files
            .iter()
            .find(|f| f.relative_path == composed)
            .unwrap_or_else(|| panic!("{composed} must land as its own source.files row"));
        let provenance = child
            .parent
            .as_ref()
            .unwrap_or_else(|| panic!("{composed} must carry its parent coordinate"));
        assert_eq!(provenance.parent_relative_path, parent);
        assert_eq!(provenance.entry_path, entry);
        let parent_row = scan
            .files
            .iter()
            .find(|f| f.relative_path == parent)
            .expect("the container itself landed");
        assert_eq!(
            provenance.parent_key, parent_row.local_key,
            "the child's parent coordinate names the parent's OWN key, chained"
        );
        assert!(
            !child.units.is_empty(),
            "{composed} routed through the same adapter a loose file of that name uses, so it \
             has real units"
        );
    }

    // --------------------------------------- the recorded generation itself
    // Not just the in-memory `SourceScan` — the real three-step
    // stage/journal/confirm discipline `record_scan` implements, exactly the
    // coupling `record.rs`'s own module doc states, over the SAME scan a
    // production `sgt intelligence scan` would have produced.
    let data_dir = TempDir::new().expect("data dir");
    let mut db = AtlasDb::open(data_dir.path()).expect("open atlas");
    let mut journal = Journal::open(data_dir.path()).expect("open journal");
    let record = record_scan(&mut db, &mut journal, &scan, None).expect("record");
    assert!(
        matches!(
            record,
            sergeant_rs::runtime::atlas::record::ScanRecord::Recorded { .. }
        ),
        "a fresh scan must record a new generation: {record:?}"
    );

    let summaries = scan_summaries(data_dir.path());
    assert_eq!(summaries.len(), 1, "exactly one scan was recorded");
    let extractors: Vec<&str> = summaries[0].payload["extractors"]
        .as_array()
        .expect("extractors array")
        .iter()
        .map(|v| v.as_str().expect("str"))
        .collect();
    assert!(
        extractors.contains(&DOCX_EXTRACTOR),
        "the RECORDED generation's own extractor set must name the docx adapter: {extractors:?}"
    );
    assert!(
        extractors.contains(&ZIP_EXTRACTOR),
        "the RECORDED generation's own extractor set must name the zip adapter: {extractors:?}"
    );
    assert!(
        extractors.contains(&MAIL_EXTRACTOR),
        "the RECORDED generation's own extractor set must name the mail adapter: {extractors:?}"
    );
    assert_eq!(
        summaries[0].payload["files"], 7,
        "three container resources plus their four landed children (S5 W7)"
    );
}

/// **S4 Y8 fix-agent panel finding.** The test above proves the DISPATCH
/// logic — worker routing, daemon-side `validate_batch`, recording — by
/// driving [`scan_local_knowledge_with_worker`] with a hand-built
/// [`WorkerRuntime`] that already points straight at
/// `CARGO_BIN_EXE_sgt-atlas-worker`. It never exercises
/// [`scan_local_knowledge_on_lane`] — the actual production entry point —
/// so it never exercised `lane::worker_runtime`'s own RESOLUTION of that
/// binary's path, and that resolution was the actual bug: the originally
/// landed `worker_runtime` returned `current_exe()` unchanged (this
/// process's own daemon binary, not the worker), which would have made
/// every real installation's dispatch fail with a clap parse error before
/// a single resource was ever extracted, while this suite's own tests
/// stayed green throughout, because none of them called the real entry
/// point either.
///
/// This test closes that gap: it plants the real `sgt-atlas-worker`
/// binary directly beside `std::env::current_exe()` — the exact location
/// `lane::worker_binary_path`'s own fix resolves against — then drives
/// [`scan_local_knowledge_on_lane`] itself, through a real [`Engine`],
/// exactly the call `sgt intelligence scan` makes daemon-side. If
/// `worker_runtime` regresses back to returning `current_exe()` unchanged,
/// this test fails (every resource comes back `Coverage::Error` from a
/// clap parse error) even though the hand-wired test above keeps passing.
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scan_local_knowledge_on_lane_resolves_and_dispatches_the_real_worker_binary() {
    let sibling = plant_worker_binary_beside_current_exe();

    let source_root = TempDir::new().expect("source root");
    std::fs::write(
        source_root.path().join("report.docx"),
        fixture("anydoc_corpus/docx_fixtures/01-plain-headings-paragraphs.docx"),
    )
    .expect("write docx");
    let source = KnowledgeSource {
        name: "lane".to_string(),
        root: source_root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::none(),
    };

    let data = TempDir::new().expect("data dir");
    let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path())
        .with_intelligence_lane_cap(1);

    let scan = scan_local_knowledge_on_lane(&engine, source)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "scan_local_knowledge_on_lane must resolve and dispatch the real worker \
                 planted at {}: {e}",
                sibling.display()
            )
        });

    let row = coverage_row(&scan, "report.docx");
    assert_eq!(
        row.status,
        Coverage::Indexed,
        "the real worker binary, resolved by lane::worker_runtime and spawned by \
         scan_local_knowledge_on_lane, must actually extract the docx: {row:?}"
    );
    assert!(
        scan.extractors.contains(DOCX_EXTRACTOR),
        "docx extractor missing from {:?}",
        scan.extractors
    );
}

/// Symlink the real `sgt-atlas-worker` Cargo built (`SGT_ATLAS_WORKER`)
/// into the same directory as this test binary's own `current_exe()` —
/// exactly where `lane::worker_binary_path` looks. Idempotent (a shared
/// `deps/` directory across every test in this binary, possibly across a
/// prior run's leftovers) rather than a bare create-or-panic, because
/// nothing here owns exclusive rights to that directory.
fn plant_worker_binary_beside_current_exe() -> PathBuf {
    let exe = std::env::current_exe().expect("current_exe");
    let dir = exe.parent().expect("current_exe has a parent dir");
    let sibling = dir.join("sgt-atlas-worker");
    let real = PathBuf::from(SGT_ATLAS_WORKER);
    if let Ok(existing_target) = std::fs::read_link(&sibling) {
        if existing_target == real {
            return sibling;
        }
        std::fs::remove_file(&sibling)
            .unwrap_or_else(|e| panic!("remove stale symlink at {}: {e}", sibling.display()));
    }
    std::os::unix::fs::symlink(&real, &sibling)
        .unwrap_or_else(|e| panic!("symlink {} -> {}: {e}", sibling.display(), real.display()));
    sibling
}

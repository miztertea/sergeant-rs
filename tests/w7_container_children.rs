//! S5 W7 acceptance: a container child is a RESOURCE, not a name
//! (brief-w7-container-child-payload.md; A1 §6.5, §6.6, A1-17).
//!
//! Before this wave, `worker::DeclaredChild` was `{ name, relative_path }`.
//! An admitted ZIP entry or mail attachment was validated daemon-side and
//! then recorded only as a name in its container's own coverage detail: no
//! bytes, no hash, no adapter, no row of its own. A1 §6.6 says an expanded
//! entry preserves four things — parent archive source/resource, entry path,
//! entry content hash, entry adapter — and A1-17 says child bytes are
//! "recursively route[d] through normal adapters". Two of those four fields
//! did not exist on the wire, and no child's bytes reached any adapter.
//!
//! # H15, the wave's one J0-shaped question, and how it was answered
//!
//! The daemon hashes a top-level resource itself, before the worker runs.
//! A child's bytes are inside a container the daemon does not parse, so
//! option (a) — the daemon re-opening the container — would have moved ZIP
//! and MIME parsing into the sole writer, which is exactly what the
//! supervised worker model (PDEATHSIG, own process group, RLIMIT_AS) exists
//! to prevent. This wave takes option (b), the brief's recommendation: the
//! worker returns the bytes, and the daemon hashes WHAT IT RECEIVES, on
//! receipt, before storing. The honest claim, stated in
//! `DeclaredChild`'s own doc and asserted here, is that a child's content
//! hash identifies **the bytes that reached the store** — not "what is really
//! inside the archive", a correspondence the daemon never observed.
//!
//! Every test below names the claim it pins. The negative ones are the point.

use std::io::{Cursor, Write as _};
use std::path::PathBuf;
use std::time::Duration;

use tempfile::TempDir;

use sergeant_rs::domain::source::{Coverage, CoverageRow, content_hash};
use sergeant_rs::runtime::atlas::archive::{
    MAX_ENTRY_UNCOMPRESSED_BYTES, MAX_NESTING_DEPTH, MAX_TOTAL_EXPANDED_BYTES, ZIP_EXTRACTOR,
};
use sergeant_rs::runtime::atlas::db::{AtlasDb, atlas_db_path};
use sergeant_rs::runtime::atlas::deny::AcquisitionFilter;
use sergeant_rs::runtime::atlas::office::DOCX_EXTRACTOR;
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{
    KnowledgeSource, MAX_RESOURCE_BYTES, SourceScan, child_extractor_for,
    scan_local_knowledge_with_worker,
};
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::atlas::worker::{
    BatchRefusal, DeclaredChild, MAX_CHILD_CONTENT_BYTES, WorkerBatch, WorkerIdentity,
    WorkerRuntime, child_content_hex, validate_batch,
};
use sergeant_rs::runtime::journal::Journal;

const SGT_ATLAS_WORKER: &str = env!("CARGO_BIN_EXE_sgt-atlas-worker");

fn worker() -> WorkerRuntime {
    WorkerRuntime {
        program: PathBuf::from(SGT_ATLAS_WORKER),
        deadline: Duration::from_secs(20),
    }
}

fn fixture(relative: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
}

/// A STORED (uncompressed) ZIP over `entries`, built with the same crate the
/// adapter reads with — `tests/y3_zip_adapter.rs`'s own fixture-writing
/// shape, reused rather than a second hand-rolled writer (R2).
fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buffer = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(Cursor::new(&mut buffer));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, body) in entries {
            writer.start_file(*name, options).expect("start_file");
            writer.write_all(body).expect("write entry");
        }
        writer.finish().expect("finish archive");
    }
    buffer
}

/// Scan a one-file source root holding `bytes` at `name`, through the real
/// worker-enabled walk (the shape `scan_local_knowledge_on_lane` drives).
fn scan_one(name: &str, bytes: &[u8]) -> (TempDir, SourceScan) {
    let root = TempDir::new().expect("source root");
    std::fs::write(root.path().join(name), bytes).expect("write resource");
    let source = KnowledgeSource {
        name: "w7".to_string(),
        root: root.path().to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::none(),
    };
    let scan = scan_local_knowledge_with_worker(&source, &worker()).expect("scan");
    (root, scan)
}

fn row<'a>(scan: &'a SourceScan, path: &str) -> &'a CoverageRow {
    scan.coverage
        .iter()
        .find(|r| r.path.as_deref() == Some(path))
        .unwrap_or_else(|| panic!("no coverage row for {path:?}: {:?}", scan.coverage))
}

fn identity() -> WorkerIdentity {
    WorkerIdentity {
        generation_id: "gen-1".to_string(),
        resource_hash: "hash-1".to_string(),
        extractor: "fixture/v1".to_string(),
    }
}

fn batch_with(children: Vec<DeclaredChild>) -> WorkerBatch {
    WorkerBatch {
        generation_id: "gen-1".to_string(),
        resource_hash: "hash-1".to_string(),
        extractor: "fixture/v1".to_string(),
        units: Vec::new(),
        declared_children: children,
    }
}

/// A child a well-behaved worker would send: the two cross-checked fields
/// composed exactly as `sgt-atlas-worker`'s own `declared_child` composes
/// them.
fn well_formed(name: &str, relative_path: &str, content: &[u8]) -> DeclaredChild {
    DeclaredChild {
        name: name.to_string(),
        relative_path: relative_path.to_string(),
        content: content.to_vec(),
        content_hash: content_hash(content),
        entry_adapter: child_extractor_for(relative_path).map(str::to_string),
    }
}

fn deny() -> AcquisitionFilter {
    AcquisitionFilter::new(&[]).expect("compile default deny set")
}

// ------------------------------------------------- the positive acceptance

/// **A1 §6.6, all four preserved fields, from a real scan.** An admitted ZIP
/// entry lands as its OWN `source.files`-bound resource — its own composed
/// path, its own units, the daemon's own hash of the bytes it received, its
/// own downstream adapter — carrying the parent coordinate (parent resource
/// path, parent key, entry path).
///
/// Watched red against the pre-W7 build, where `scan.files` held exactly one
/// row (`bundle.zip`) and the entries existed only as names inside that row's
/// coverage detail.
#[test]
fn an_admitted_zip_entry_lands_as_its_own_resource_with_all_four_preserved_fields() {
    let body = b"# notes\n\nreal child bytes\n";
    let (_root, scan) = scan_one("bundle.zip", &zip_of(&[("notes/a.md", body)]));

    let parent = scan
        .files
        .iter()
        .find(|f| f.relative_path == "bundle.zip")
        .expect("the container itself lands");
    let child = scan
        .files
        .iter()
        .find(|f| f.relative_path == "bundle.zip!/notes/a.md")
        .unwrap_or_else(|| panic!("the entry must land as its own resource: {:?}", scan.files));

    // Preserved field 3: the entry content hash — the DAEMON's own BLAKE3 of
    // the bytes that reached it, not a value copied from the worker's claim.
    assert_eq!(child.content_hash, content_hash(body));
    // Preserved field 4: the entry adapter — the CHILD's own downstream
    // extractor, never the container adapter that unpacked it.
    assert_eq!(child.extractor, "markdown/v1");
    assert_ne!(child.extractor, ZIP_EXTRACTOR);
    // Preserved fields 1 and 2: the parent coordinate.
    let provenance = child.parent.as_ref().expect("child carries its parent");
    assert_eq!(provenance.parent_relative_path, "bundle.zip");
    assert_eq!(provenance.parent_key, parent.local_key);
    assert_eq!(provenance.entry_path, "notes/a.md");
    // A resource with real content, not a placeholder.
    assert_eq!(child.byte_len, body.len() as u64);
    assert!(
        child
            .units
            .iter()
            .any(|u| u.text.contains("real child bytes")),
        "the child's own bytes reached the markdown adapter: {:?}",
        child.units
    );
    // Its F7 key is the chained child key, distinct from a top-level key over
    // the same bytes and extractor.
    assert_eq!(
        child.local_key,
        sergeant_rs::domain::source::child_key(
            &parent.local_key,
            "notes/a.md",
            &child.content_hash,
            "markdown/v1",
        )
    );
    assert!(scan.files.iter().all(|f| f.relative_path != "notes/a.md"));
}

/// **A1-17 / A1 §6.5, the whole chain.** A `.docx` inside a `.zip` inside an
/// `.eml` reaches the Office adapter by the same route a loose `.docx` does
/// — `scan::child_extractor_for` is `format_for` ∪ `worker_extractor_for` ∪
/// `claims_for`, the same three tables in the same order `Walk::file`
/// consults, so there is one dispatcher, not a second one for children (R2).
///
/// The mail attachment also keeps its parent-message coordinate (§6.5), and
/// the intermediate container lands as its own resource with no units of its
/// own — its content is its children.
#[test]
fn a_docx_inside_a_zip_inside_an_eml_reaches_the_office_adapter_by_the_same_route() {
    let docx = fixture("anydoc_corpus/docx_fixtures/01-plain-headings-paragraphs.docx");
    let inner_zip = zip_of(&[("report.docx", &docx)]);
    // A minimal RFC 5322 message carrying that ZIP as a base64 attachment.
    let mut raw: Vec<u8> = b"From: a@example.com\r\nTo: b@example.com\r\nSubject: bundle\r\n\
Date: Mon, 1 Jan 2024 00:00:00 +0000\r\nMIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"B\"\r\n\r\n--B\r\nContent-Type: text/plain\r\n\r\n\
see attached\r\n--B\r\nContent-Type: application/zip; name=\"bundle.zip\"\r\n\
Content-Disposition: attachment; filename=\"bundle.zip\"\r\n\
Content-Transfer-Encoding: base64\r\n\r\n"
        .to_vec();
    raw.extend_from_slice(base64_encode(&inner_zip).as_bytes());
    raw.extend_from_slice(b"\r\n--B--\r\n");

    let (_root, scan) = scan_one("message.eml", &raw);

    let zip_child = scan
        .files
        .iter()
        .find(|f| f.relative_path == "message.eml!/bundle.zip")
        .unwrap_or_else(|| panic!("the attachment lands as a resource: {:?}", scan.files));
    assert_eq!(zip_child.extractor, ZIP_EXTRACTOR);
    assert!(
        zip_child.units.is_empty(),
        "a container's content is its children, not a body unit of its own"
    );
    assert_eq!(
        zip_child
            .parent
            .as_ref()
            .expect("attachment carries its parent-message coordinate")
            .parent_relative_path,
        "message.eml"
    );

    let docx_child = scan
        .files
        .iter()
        .find(|f| f.relative_path == "message.eml!/bundle.zip!/report.docx")
        .unwrap_or_else(|| {
            panic!(
                "the grandchild must reach the office adapter: {:?}",
                scan.files
            )
        });
    assert_eq!(
        docx_child.extractor, DOCX_EXTRACTOR,
        "a .docx inside a .zip inside an .eml is extracted by the office adapter itself"
    );
    assert!(
        docx_child.units.iter().any(|u| !u.text.trim().is_empty()),
        "the office adapter ran on the grandchild's real bytes: {:?}",
        docx_child.units
    );
    assert_eq!(
        docx_child.content_hash,
        content_hash(&docx),
        "the daemon's own hash of the bytes it received identifies the grandchild"
    );
    assert!(
        scan.extractors.contains(DOCX_EXTRACTOR),
        "the scan's own extractor set names the adapter that ran on a child: {:?}",
        scan.extractors
    );
}

/// **F-SF-01 / A1-17 / A1 §1's motivating payload.** A `.csv` inside a
/// `.zip` reaches the RELATIONAL lane a loose `.csv` reaches — registered as
/// a dataset, read in place, profiled by the canned queries — rather than
/// landing as an unsupported gap claiming "nothing in this build claims its
/// extension" while `tabular.rs` claims exactly that extension.
///
/// A1 §1 names the payload verbatim ("100k ServiceNow tickets in
/// CSV/JSON/Parquet inside an archive") and A1-17 says child bytes route
/// "through normal adapters". The one real constraint is that DuckDB reads a
/// dataset from a filesystem path and a child's bytes exist only in memory;
/// that is answered by the daemon materialising the child's bytes into its
/// OWN scratch directory around the read it already performs, not by the
/// container being unpacked to disk and not by prose-flattening the rows
/// (A1-13).
#[test]
fn a_csv_inside_a_zip_reaches_the_relational_lane_a_loose_csv_reaches() {
    let csv = b"id,short_description\n1,printer offline\n2,vpn down\n";
    let (_root, scan) = scan_one("bundle.zip", &zip_of(&[("tickets.csv", csv)]));

    let registered = scan
        .datasets
        .iter()
        .find(|d| d.relative_path == "bundle.zip!/tickets.csv")
        .unwrap_or_else(|| {
            panic!(
                "a .csv child must be REGISTERED as a dataset, not reported unsupported: \
                 datasets={:?} coverage={:?}",
                scan.datasets, scan.coverage
            )
        });
    assert_eq!(registered.content_hash, content_hash(csv));
    assert!(
        registered.content.is_some(),
        "a child dataset carries its own bytes — it has no path to be read in place from"
    );
    // A1 §6.6's parent coordinate is preserved for the relational lane too.
    let provenance = registered
        .parent
        .as_ref()
        .expect("a child dataset carries its parent coordinate");
    assert_eq!(provenance.parent_relative_path, "bundle.zip");
    assert_eq!(provenance.entry_path, "tickets.csv");
    // And it is NOT landed as a `source.files` resource: a dataset stays
    // data (A1-13), it does not become documents.
    assert!(
        scan.files
            .iter()
            .all(|f| f.relative_path != "bundle.zip!/tickets.csv"),
        "a dataset child must not also land as an extracted document: {:?}",
        scan.files
    );

    let data_dir = TempDir::new().expect("data dir");
    {
        let mut db = AtlasDb::open(data_dir.path()).expect("open atlas");
        let mut journal = Journal::open(data_dir.path()).expect("open journal");
        record_scan(&mut db, &mut journal, &scan, None).expect("record");
    }
    let db = AtlasDb::open(data_dir.path()).expect("reopen atlas");
    let stored = db
        .datasets("w7", 50)
        .expect("read datasets")
        .into_iter()
        .find(|d| d.relative_path == "bundle.zip!/tickets.csv")
        .expect("the child dataset is persisted as a dataset row");
    // The read actually happened, in place, against the materialised bytes:
    // real columns and a real row count, not an empty registration.
    assert_eq!(
        stored.columns,
        vec!["id".to_string(), "short_description".to_string()],
        "the tabular reader saw the child's real columns"
    );
    assert_eq!(stored.row_count, 2, "and its real rows");
    let facts = db.dataset_facts("w7", 50).expect("read facts");
    assert!(
        facts
            .iter()
            .any(|f| f.relative_path == "bundle.zip!/tickets.csv"),
        "the canned queries ran against the child dataset: {facts:?}"
    );

    // The scratch the daemon gave those bytes is gone again.
    let scratch = data_dir.path().join("atlas-child-datasets");
    assert!(
        !scratch.exists()
            || std::fs::read_dir(&scratch)
                .expect("read scratch")
                .next()
                .is_none(),
        "the materialised child dataset must not outlive its own read"
    );
}

/// **F-SF-02: a grandchild's parent coordinate is its IMMEDIATE container,
/// not the root.** A1 §6.6 preserves the "parent archive source/resource" of
/// an expanded entry, and `KeySpace::Child`'s own doc says a child's F7 key
/// is `child_key` of "its IMMEDIATE parent's own key — chained, so a
/// grandchild's key already encodes its whole ancestry".
///
/// Before this fix the worker's flattened batch was landed against ONE fixed
/// coordinate — the dispatched resource's path and key — for every entry in
/// it, however deep, so `message.eml!/bundle.zip!/report.docx` recorded
/// `message.eml` as its parent and keyed off the *message's* key with the
/// whole ancestry mashed into the `entry_path` string. The intermediate
/// `bundle.zip` is itself a landed resource with its own key; that is the
/// key a grandchild chains through, and this test reads the PERSISTED row to
/// say so rather than trusting the in-memory struct.
#[test]
fn a_grandchilds_persisted_parent_is_its_immediate_container_not_the_root() {
    let docx = fixture("anydoc_corpus/docx_fixtures/01-plain-headings-paragraphs.docx");
    let inner_zip = zip_of(&[("report.docx", &docx)]);
    let mut raw: Vec<u8> = b"From: a@example.com\r\nTo: b@example.com\r\nSubject: bundle\r\n\
Date: Mon, 1 Jan 2024 00:00:00 +0000\r\nMIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"B\"\r\n\r\n--B\r\nContent-Type: text/plain\r\n\r\n\
see attached\r\n--B\r\nContent-Type: application/zip; name=\"bundle.zip\"\r\n\
Content-Disposition: attachment; filename=\"bundle.zip\"\r\n\
Content-Transfer-Encoding: base64\r\n\r\n"
        .to_vec();
    raw.extend_from_slice(base64_encode(&inner_zip).as_bytes());
    raw.extend_from_slice(b"\r\n--B--\r\n");

    let (_root, scan) = scan_one("message.eml", &raw);

    let zip_child = scan
        .files
        .iter()
        .find(|f| f.relative_path == "message.eml!/bundle.zip")
        .expect("the intermediate container lands as its own resource");
    let grandchild = scan
        .files
        .iter()
        .find(|f| f.relative_path == "message.eml!/bundle.zip!/report.docx")
        .unwrap_or_else(|| panic!("the grandchild lands: {:?}", scan.files));

    // The in-memory provenance, which is what gets persisted.
    let provenance = grandchild
        .parent
        .as_ref()
        .expect("a grandchild carries a parent coordinate");
    assert_eq!(
        provenance.parent_relative_path, "message.eml!/bundle.zip",
        "the parent is the ZIP the entry actually came out of, not the root message"
    );
    assert_eq!(
        provenance.parent_key, zip_child.local_key,
        "the parent key is the intermediate container's OWN composed key"
    );
    assert_eq!(
        provenance.entry_path, "report.docx",
        "the entry path is relative to that immediate parent, not the whole ancestry as a string"
    );
    // And the F7 key chains through it, exactly as `KeySpace::Child`'s doc
    // claims — one link, not a root-resolved key over a multi-segment path.
    assert_eq!(
        grandchild.local_key,
        sergeant_rs::domain::source::child_key(
            &zip_child.local_key,
            "report.docx",
            &grandchild.content_hash,
            DOCX_EXTRACTOR,
        )
    );

    // Persisted, through the real stage/journal/confirm discipline.
    let data_dir = TempDir::new().expect("data dir");
    {
        let mut db = AtlasDb::open(data_dir.path()).expect("open atlas");
        let mut journal = Journal::open(data_dir.path()).expect("open journal");
        record_scan(&mut db, &mut journal, &scan, None).expect("record");
    }
    let conn =
        duckdb::Connection::open(atlas_db_path(data_dir.path())).expect("read the recorded store");
    let mut statement = conn
        .prepare(
            "SELECT parent_relative_path, parent_key, entry_path \
             FROM source.child_resources WHERE relative_path = ?",
        )
        .expect("prepare");
    let mut rows = statement
        .query_map(["message.eml!/bundle.zip!/report.docx"], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .expect("query")
        .map(|r| r.expect("row"))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "exactly one persisted row: {rows:?}");
    let (parent_path, recorded_parent_key, entry_path) = rows.pop().expect("row");
    assert_eq!(parent_path, "message.eml!/bundle.zip");
    assert_eq!(recorded_parent_key, zip_child.local_key);
    assert_eq!(entry_path, "report.docx");
}

/// **F-SF-02, the daemon-side authority that makes the chaining
/// well-founded.** A flattened path ASSERTS the container it came out of.
/// A batch declaring `bundle.zip!/report.md` without also declaring the
/// container child `bundle.zip` is refused outright, rather than having the
/// orphan silently re-parented onto the dispatched resource — which is
/// precisely the root-resolution this finding found.
#[test]
fn a_child_naming_a_container_the_batch_never_declared_is_refused() {
    let batch = batch_with(vec![well_formed(
        "report.md",
        "bundle.zip!/report.md",
        b"# orphan",
    )]);
    let err = validate_batch(&identity(), &batch, &deny()).expect_err("must refuse");
    assert!(
        matches!(
            &err,
            BatchRefusal::OrphanedChildPath { missing_container, .. }
                if missing_container == "bundle.zip"
        ),
        "{err:?}"
    );
    assert_eq!(err.coverage_row().status, Coverage::Error);

    // The same child, with its container actually declared, is admitted.
    let whole = batch_with(vec![
        well_formed("bundle.zip", "bundle.zip", b"PK\x05\x06"),
        well_formed("report.md", "bundle.zip!/report.md", b"# orphan"),
    ]);
    assert_eq!(validate_batch(&identity(), &whole, &deny()), Ok(()));
}

/// **A child with no claiming extractor is a NAMED COVERAGE GAP, not
/// silence** (F8, A1 §15). It is not landed as a resource — nothing extracted
/// it — but it is not dropped either: a `Coverage::Unsupported` row at its own
/// composed path says so.
#[test]
fn a_child_with_no_claiming_extractor_is_a_named_coverage_gap() {
    let (_root, scan) = scan_one(
        "bundle.zip",
        &zip_of(&[("blob.unknownformat", b"\x00\x01\x02")]),
    );

    let gap = row(&scan, "bundle.zip!/blob.unknownformat");
    assert_eq!(gap.status, Coverage::Unsupported);
    let detail = gap.detail.clone().unwrap_or_default();
    assert!(
        detail.contains("bundle.zip"),
        "the gap names the container it came out of: {detail:?}"
    );
    assert_eq!(gap.bytes, Some(3), "the gap knows how big the child was");
    assert!(
        scan.files
            .iter()
            .all(|f| f.relative_path != "bundle.zip!/blob.unknownformat"),
        "a child nothing claims is not landed as a resource"
    );
}

/// **The parent coordinate is PERSISTED, not just in-memory.** A landed child
/// writes a `source.child_resources` row through the real
/// stage/journal/confirm discipline `record_scan` implements — the row A1
/// §6.6's first two preserved fields live in, beside the entry content hash
/// and entry adapter the child's own `source.files` row already carries.
///
/// The row is in its OWN table rather than two more columns on
/// `source.files` because `db.rs`'s own module doc forbids altering a landed
/// table ("only ever added to, never altered" — the DDL is `IF NOT EXISTS`,
/// so a new column would silently not appear in a database that already
/// exists), which is the same rule X3b's own rows already follow.
#[test]
fn a_landed_childs_parent_coordinate_is_persisted_in_its_own_table() {
    let (_root, scan) = scan_one("bundle.zip", &zip_of(&[("notes/a.md", b"# notes\n")]));
    let parent_key = scan
        .files
        .iter()
        .find(|f| f.relative_path == "bundle.zip")
        .expect("the container lands")
        .local_key
        .clone();

    let data_dir = TempDir::new().expect("data dir");
    {
        let mut db = AtlasDb::open(data_dir.path()).expect("open atlas");
        let mut journal = Journal::open(data_dir.path()).expect("open journal");
        record_scan(&mut db, &mut journal, &scan, None).expect("record");
    }

    let conn =
        duckdb::Connection::open(atlas_db_path(data_dir.path())).expect("read the recorded store");
    // F-SI-01: content_hash and extractor are not duplicated on
    // `child_resources` — they are already columns on the `source.files`
    // row for this same (generation_id, source_name, relative_path), joined
    // back to here rather than re-read from a second copy.
    let mut statement = conn
        .prepare(
            "SELECT c.parent_relative_path, c.parent_key, c.entry_path, f.content_hash, \
             f.extractor \
             FROM source.child_resources c \
             JOIN source.files f \
               ON f.generation_id = c.generation_id \
              AND f.source_name = c.source_name \
              AND f.relative_path = c.relative_path \
             WHERE c.relative_path = ?",
        )
        .expect("prepare");
    let mut rows = statement
        .query_map(["bundle.zip!/notes/a.md"], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .expect("query")
        .map(|r| r.expect("row"))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "exactly one child row: {rows:?}");
    let (parent_path, recorded_parent_key, entry_path, hash, extractor) = rows.pop().expect("row");
    assert_eq!(parent_path, "bundle.zip");
    assert_eq!(recorded_parent_key, parent_key);
    assert_eq!(entry_path, "notes/a.md");
    assert_eq!(hash, content_hash(b"# notes\n"));
    assert_eq!(extractor, "markdown/v1");
}

/// **F-SF-03: `source.child_resources` has a READER.** The table was written
/// per landed child and read by nothing in product code — every occurrence in
/// `src/` was DDL, INSERT or DELETE — while the insert's own comment claimed
/// "a reader joins back to it on that key". A1-15 keeps arbitrary client SQL
/// off the public surface, so without a canned read two of A1 §6.6's four
/// preserved fields were reachable only by opening the database file
/// directly, which is exactly what a capability built and left unreachable
/// looks like.
///
/// This test deliberately opens NO raw `duckdb::Connection`: it goes through
/// `AtlasDb::child_resources`, the daemon's own bounded reader that
/// `GET /v1/map/children` and `sgt map children` are the transport for.
#[test]
fn the_parent_coordinate_is_readable_through_the_daemons_own_bounded_reader() {
    let (_root, scan) = scan_one(
        "bundle.zip",
        &zip_of(&[
            ("notes/a.md", b"# notes\n" as &[u8]),
            ("tickets.csv", b"id,short_description\n1,printer offline\n"),
        ]),
    );
    let container_key = scan
        .files
        .iter()
        .find(|f| f.relative_path == "bundle.zip")
        .expect("the container lands")
        .local_key
        .clone();

    let data_dir = TempDir::new().expect("data dir");
    {
        let mut db = AtlasDb::open(data_dir.path()).expect("open atlas");
        let mut journal = Journal::open(data_dir.path()).expect("open journal");
        record_scan(&mut db, &mut journal, &scan, None).expect("record");
    }

    let db = AtlasDb::open(data_dir.path()).expect("reopen atlas");
    let children = db.child_resources("w7", 50).expect("read child resources");
    assert_eq!(
        children.len(),
        2,
        "both lanes come back from one read: {children:?}"
    );

    let document = children
        .iter()
        .find(|c| c.relative_path == "bundle.zip!/notes/a.md")
        .expect("the document child is readable");
    assert_eq!(document.parent_relative_path, "bundle.zip");
    assert_eq!(document.parent_key, container_key);
    assert_eq!(document.entry_path, "notes/a.md");
    // §6.6's other two fields, JOINED BACK from the resource's own row rather
    // than duplicated beside the coordinate (F-SI-01) — which is only a
    // design, and not a hole, because this read performs the join.
    assert_eq!(
        document.content_hash.as_deref(),
        Some(content_hash(b"# notes\n").as_str())
    );
    assert_eq!(document.extractor.as_deref(), Some("markdown/v1"));
    assert_eq!(document.lane.as_deref(), Some("file"));

    let dataset = children
        .iter()
        .find(|c| c.relative_path == "bundle.zip!/tickets.csv")
        .expect("the dataset child is readable through the SAME reader");
    assert_eq!(dataset.parent_relative_path, "bundle.zip");
    assert_eq!(dataset.parent_key, container_key);
    assert_eq!(dataset.entry_path, "tickets.csv");
    assert_eq!(dataset.lane.as_deref(), Some("dataset"));
    assert!(
        dataset
            .extractor
            .as_deref()
            .unwrap_or_default()
            .starts_with("csv/"),
        "a dataset child's entry adapter is its reader identity: {dataset:?}"
    );
}

// -------------------------------------------------- the negative acceptance

/// **Still refused (path safety).** W7 widened what a child carries; it did
/// not weaken what `validate_batch` checks. A child whose declared path
/// escapes its parent is refused before any byte of it reaches the store.
#[test]
fn a_child_whose_declared_path_escapes_its_parent_is_still_refused() {
    let batch = batch_with(vec![well_formed(
        "innocuous.txt",
        "../../etc/passwd",
        b"root:x:0:0",
    )]);
    let err = validate_batch(&identity(), &batch, &deny()).expect_err("must refuse");
    assert!(
        matches!(err, BatchRefusal::UnsafeChildPath { .. }),
        "{err:?}"
    );
    assert_eq!(err.coverage_row().status, Coverage::Error);
}

/// **Still refused (F10's deny set, on the NAME).** Checked independently of
/// the path, exactly as before W7.
#[test]
fn a_child_whose_name_trips_the_deny_set_is_still_refused() {
    let batch = batch_with(vec![well_formed(".env", "config/.env", b"SECRET=1")]);
    let err = validate_batch(&identity(), &batch, &deny()).expect_err("must refuse");
    assert!(
        matches!(err, BatchRefusal::DeniedChildName { .. }),
        "{err:?}"
    );
    assert_eq!(err.coverage_row().status, Coverage::Excluded);
}

/// **The per-child ceiling refuses BEFORE allocation.** S4 already shipped an
/// O(N²)-before-the-cap bug of exactly this shape, so the cap comes first:
/// `child_content_hex::decode_bounded` refuses from the ENCODED length alone,
/// before a single decoded byte is allocated, and the largest buffer it can
/// be talked into is the limit itself.
///
/// Proven at the limit the transport uses AND at a tiny limit, so the "never
/// allocates what was declared" property is asserted rather than inferred
/// from a ceiling too large to exercise.
#[test]
fn a_child_whose_declared_bytes_exceed_the_per_child_ceiling_is_refused_before_allocation() {
    // Tiny limit: the refusal names the ceiling, and the decode never runs.
    let encoded = "00".repeat(64);
    let err = child_content_hex::decode_bounded(&encoded, 4).expect_err("must refuse");
    assert!(
        err.contains("per-child ceiling") && err.contains("before the decoded buffer"),
        "{err:?}"
    );
    assert_eq!(
        child_content_hex::decode_bounded(&encoded, 64).expect("at the limit is admitted"),
        vec![0u8; 64]
    );

    // The wire decoder carries the same ceiling: a batch declaring one byte
    // past `MAX_CHILD_CONTENT_BYTES` never deserializes into a `Vec` at all.
    let oversized = "00".repeat(MAX_CHILD_CONTENT_BYTES as usize + 1);
    let json = format!(
        "{{\"generation_id\":\"gen-1\",\"resource_hash\":\"hash-1\",\"extractor\":\"fixture/v1\",\
          \"units\":[],\"declared_children\":[{{\"name\":\"big.md\",\"relative_path\":\"big.md\",\
          \"content\":\"{oversized}\",\"content_hash\":\"x\",\"entry_adapter\":null}}]}}"
    );
    let refused = serde_json::from_str::<WorkerBatch>(&json).expect_err("must refuse");
    assert!(
        refused.to_string().contains("per-child ceiling"),
        "{refused}"
    );

    // And `validate_batch` is the AUTHORITY half, independent of transport: a
    // batch built in-process never passes through the decoder at all.
    let batch = batch_with(vec![well_formed(
        "big.md",
        "big.md",
        &vec![0u8; MAX_CHILD_CONTENT_BYTES as usize + 1],
    )]);
    let err = validate_batch(&identity(), &batch, &deny()).expect_err("must refuse");
    assert!(matches!(err, BatchRefusal::ChildTooLarge { .. }), "{err:?}");
}

/// **A child whose received bytes do not match its declared hash is refused
/// rather than stored.** This is the cross-check that makes H15 option (b)
/// honest: the daemon hashes what it received and stores THAT, and a batch
/// whose two halves disagree is refused whole — never landed partially.
#[test]
fn a_child_whose_received_bytes_do_not_match_its_declared_hash_is_refused_rather_than_stored() {
    let mut child = well_formed("a.md", "a.md", b"the bytes that arrived");
    child.content_hash = content_hash(b"the bytes it claimed to send");
    let batch = batch_with(vec![child]);
    let err = validate_batch(&identity(), &batch, &deny()).expect_err("must refuse");
    let BatchRefusal::ChildHashMismatch { computed, .. } = &err else {
        panic!("{err:?}");
    };
    assert_eq!(
        computed,
        &content_hash(b"the bytes that arrived"),
        "the daemon's own hash is over what it actually received"
    );
}

/// **A1 §6.6's `entry adapter` is cross-checked, not trusted.** A worker
/// claiming an adapter this build's own routing table would not choose for
/// that path is refused — the same shape as the content-hash cross-check, and
/// the reason recording every archive-derived child as `adapter=zip` (the
/// research note's own warning) cannot happen by a worker asserting it.
#[test]
fn a_child_claiming_an_adapter_this_build_would_not_choose_is_refused() {
    let mut child = well_formed("a.md", "a.md", b"# heading\n");
    child.entry_adapter = Some(ZIP_EXTRACTOR.to_string());
    let batch = batch_with(vec![child]);
    let err = validate_batch(&identity(), &batch, &deny()).expect_err("must refuse");
    assert!(
        matches!(err, BatchRefusal::ChildAdapterMismatch { .. }),
        "{err:?}"
    );
}

/// **A nested container cannot escape the shared depth ceiling by
/// recursing.** Four levels of archive: the ceiling is
/// `archive::MAX_NESTING_DEPTH` (2) — the counter mail and archive already
/// share — so the third nested archive is admitted as its own child but never
/// opened, and its member never appears at any path.
///
/// This is what makes flattening safe: the daemon lands what the batch says
/// and never re-enters a container, so the tree's depth is bounded exactly
/// once, by the adapter that walked it.
#[test]
fn a_nested_container_cannot_escape_the_shared_depth_ceiling_by_recursing() {
    let l3 = zip_of(&[("leaf.md", b"# deepest\n")]);
    let l2 = zip_of(&[("c.zip", &l3)]);
    let l1 = zip_of(&[("b.zip", &l2)]);
    let (_root, scan) = scan_one("a.zip", &zip_of(&[("inner.zip", &l1)]));

    let paths: Vec<&str> = scan
        .files
        .iter()
        .map(|f| f.relative_path.as_str())
        .collect();
    for reachable in [
        "a.zip",
        "a.zip!/inner.zip",
        "a.zip!/inner.zip!/b.zip",
        "a.zip!/inner.zip!/b.zip!/c.zip",
    ] {
        assert!(
            paths.contains(&reachable),
            "{reachable} must land: {paths:?}"
        );
    }
    assert_eq!(
        MAX_NESTING_DEPTH, 2,
        "this test's own arithmetic is stated against the shared ceiling's value"
    );
    assert!(
        !paths
            .iter()
            .any(|p| p.contains("leaf.md") || p.ends_with("c.zip/leaf.md")),
        "an archive past the depth ceiling is admitted but never opened: {paths:?}"
    );
    // The deepest archive is admitted as its own resource and simply not
    // opened. WHY it was not opened — `archive::expand`'s own per-entry
    // `MAX_NESTING_DEPTH` coverage row — is a per-entry coverage fact
    // `WorkerBatch` still has no field to carry; that is the PRE-EXISTING
    // named gap `src/bin/atlas_worker.rs`'s own module doc states ("a named
    // gap: per-entry ZIP coverage has nowhere to go on the wire yet"), which
    // W7 does not widen and does not claim to have closed. What this asserts
    // is that the landed row does not claim otherwise.
    let deepest = row(&scan, "a.zip!/inner.zip!/b.zip!/c.zip");
    assert_eq!(deepest.status, Coverage::Indexed);
    let detail = deepest.detail.clone().unwrap_or_default();
    assert!(
        detail.contains("never re-enters a container") && detail.contains("not claimed here"),
        "the landed row states what it knows and no more: {detail:?}"
    );
}

/// **F-IN-01: a flat batch of many individually-in-bounds children still
/// cannot exceed the whole-tree budget.** Before this fix, `validate_batch`
/// checked only the per-child ceiling and never summed `declared_children`,
/// so 40 declared children of 4MiB each (160MiB) against a 128MiB
/// `MAX_TOTAL_EXPANDED_BYTES` budget were all individually in-bounds and the
/// whole batch was wrongly accepted. This is distinct from
/// `a_nested_container_cannot_escape_the_shared_depth_ceiling_by_recursing`
/// above, which covers the budget enforced *inside* one container's own
/// expansion walk (recursion); this is a flat batch, no recursion, and the
/// gap was in the daemon's own cross-check of what a worker hands back.
#[test]
fn a_batch_whose_children_sum_past_the_whole_tree_budget_is_refused_before_any_child_lands() {
    let per_child = 4 * 1024 * 1024u64; // 4MiB
    assert!(
        per_child <= MAX_CHILD_CONTENT_BYTES,
        "each child on its own must be within the per-child ceiling"
    );
    let count = 40; // 40 * 4MiB = 160MiB, over the 128MiB whole-tree budget
    assert!(
        (count as u64) * per_child > MAX_TOTAL_EXPANDED_BYTES,
        "this test's own arithmetic must actually exceed the budget it means to test"
    );
    let children: Vec<DeclaredChild> = (0..count)
        .map(|i| {
            let name = format!("child-{i}.bin");
            well_formed(&name, &name, &vec![0u8; per_child as usize])
        })
        .collect();
    let batch = batch_with(children);
    let err = validate_batch(&identity(), &batch, &deny()).expect_err("must refuse");
    assert!(
        matches!(err, BatchRefusal::BatchTotalTooLarge { total_bytes } if total_bytes > MAX_TOTAL_EXPANDED_BYTES),
        "{err:?}"
    );
}

// ------------------------------------------------------ one counter, one budget

/// **One depth counter and one byte budget, structurally.** The brief
/// requires that child bytes count against the EXISTING single whole-tree
/// byte budget and the EXISTING single depth counter — "never a second,
/// independently sized pair" — and that a test fail if a second is
/// introduced.
///
/// So this asserts the source text: `MAX_NESTING_DEPTH` and
/// `MAX_TOTAL_EXPANDED_BYTES` are DEFINED in exactly one file
/// (`archive.rs`), and the per-child ceiling W7 added is an alias of the
/// resource ceiling this build already had, not a fifth number. A wave that
/// adds `const MAX_CHILD_NESTING_DEPTH` — or gives the per-child ceiling its
/// own value — turns this red.
#[test]
fn container_children_share_one_depth_counter_and_one_budget_not_a_second_pair() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut definitions: Vec<String> = Vec::new();
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read rust source");
            for line in text.lines().map(str::trim) {
                let is_definition = line.starts_with("pub const ") || line.starts_with("const ");
                if !is_definition {
                    continue;
                }
                if line.contains("NESTING_DEPTH") || line.contains("TOTAL_EXPANDED_BYTES") {
                    definitions.push(format!("{}: {line}", path.display()));
                }
            }
        }
    }
    assert_eq!(
        definitions.len(),
        2,
        "exactly two bound constants exist — one depth counter, one whole-tree byte budget — and \
         both are defined in archive.rs: {definitions:?}"
    );
    assert!(
        definitions.iter().all(|d| d.contains("atlas/archive.rs")),
        "both are defined in archive.rs, and mail/scan/worker reuse them: {definitions:?}"
    );

    // The per-child ceiling W7 added is an ALIAS, not a fourth number.
    assert_eq!(MAX_CHILD_CONTENT_BYTES, MAX_RESOURCE_BYTES);
    assert_eq!(MAX_CHILD_CONTENT_BYTES, MAX_ENTRY_UNCOMPRESSED_BYTES);
}

/// A minimal RFC 4648 base64 encoder, local to this test — `mail.rs`'s own
/// test helper's reasoning, unchanged: no crate in this build's own
/// `Cargo.toml` exposes base64 as a direct dependency, and a fixture encoder
/// is not worth one.
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

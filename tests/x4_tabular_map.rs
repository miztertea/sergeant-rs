//! S3 X4 acceptance: tabular datasets read in place, F4's network refusal,
//! F10a's column allowlist, and F12's bounds.
//!
//! Four things here would fail loudly if the wave's decisions were quietly
//! relaxed:
//!
//! * `f4_pins_extension_autoload_disabled_forever` reads the posture off a
//!   live connection and then proves the negative it is actually about: a
//!   remote path is *refused*, naming the extension it will not fetch, rather
//!   than downloading one mid-query.
//! * `f10a_a_source_with_no_context_fields_exposes_no_row_text` is the
//!   default-none refusal. It scans a CSV whose columns are perfectly
//!   ordinary, gets a fully registered and profiled dataset, and asserts that
//!   not one character of a row's text became a retrievable unit.
//! * `f10a_narrowing_an_allowlist_retracts_the_units_it_exposed` is the
//!   half that is easy to forget: turning an allowlist *down* has to remove
//!   what the wider one published, and it does — through F7's existing
//!   staleness machinery, because the allowlist rides in the extractor
//!   identity.
//! * `f12_every_dataset_read_is_bounded` feeds the reader more rows than the
//!   cap and asserts the answer says so instead of quietly being complete.
//!
//! Filesystem-light and daemon-free. The one subprocess is `sgt --help`,
//! which is what pins F11's *named deferral* — `map neighbors` and `map
//! changed` must not exist yet. The map/intelligence HTTP surface itself is
//! tested against the handlers in `src/api.rs`'s own tests, where the state
//! they read is constructible without a daemon.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

use sergeant_rs::domain::source::Coverage;
use sergeant_rs::runtime::atlas::db::{
    AtlasDb, DATASET_COLUMN_PROFILE, DATASET_QUERIES, DATASET_ROW_COUNT, MAX_ROWS, StoredDataset,
    output_hash,
};
use sergeant_rs::runtime::atlas::record::{ScanRecord, scan_and_record};
use sergeant_rs::runtime::atlas::scan::{
    DATASET_NO_ROOT, KnowledgeSource, claims_for, scan_local_knowledge,
};
use sergeant_rs::runtime::atlas::tabular::{
    ContextFields, DatasetFormat, RowKeyBasis, format_for, reader_identity,
};
use sergeant_rs::runtime::journal::Journal;

// ---------------------------------------------------------------- helpers

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_string()).collect()
}

fn source(name: &str, root: &Path, fields: &[&str]) -> KnowledgeSource {
    KnowledgeSource {
        name: name.to_string(),
        root: root.to_path_buf(),
        ignore: Vec::new(),
        context_fields: ContextFields::declared(&strings(fields)),
    }
}

/// A data dir with a journal and an Atlas store, and one scan recorded into
/// them through the real three-step path.
struct Recorded {
    _data: TempDir,
    db: AtlasDb,
    record: ScanRecord,
}

fn record(source: &KnowledgeSource) -> Recorded {
    let data = tempfile::tempdir().expect("data dir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = AtlasDb::open(data.path()).expect("atlas");
    let record = scan_and_record(&mut db, &mut journal, source, None).expect("scan and record");
    Recorded {
        _data: data,
        db,
        record,
    }
}

/// Write a real Parquet file.
///
/// This is a **fixture writer**, and the only reason `duckdb` is a
/// dev-dependency: nothing here opens Atlas's own store, which is what
/// `tests/x1_atlas_substrate.rs`'s one-owner assertion is about. Writing real
/// Parquet bytes is the difference between proving F4's reader works and
/// inferring it from a linked symbol.
fn write_parquet(path: &Path) {
    let conn = duckdb::Connection::open_in_memory().expect("fixture connection");
    conn.execute_batch(&format!(
        "COPY (SELECT * FROM (VALUES ('alpha', 1), ('beta', 2), ('gamma', 3)) \
         AS t(label, weight)) TO '{}' (FORMAT PARQUET)",
        path.display()
    ))
    .expect("write parquet fixture");
}

/// One dataset by relative path, or a panic naming what was there instead.
fn dataset<'a>(datasets: &'a [StoredDataset], path: &str) -> &'a StoredDataset {
    datasets
        .iter()
        .find(|d| d.relative_path == path)
        .unwrap_or_else(|| {
            panic!(
                "no dataset at {path}; registered: {:?}",
                datasets
                    .iter()
                    .map(|d| &d.relative_path)
                    .collect::<Vec<_>>()
            )
        })
}

/// The `column_profile` answer as `column -> (rows, non_null, distinct)`.
fn profile(db: &AtlasDb, source: &str, path: &str) -> BTreeMap<String, (u64, u64, u64)> {
    let fact = db
        .dataset_facts(source, MAX_ROWS)
        .expect("facts")
        .into_iter()
        .find(|f| f.relative_path == path && f.query == DATASET_COLUMN_PROFILE.name)
        .expect("a column profile for this dataset");
    fact.rows
        .iter()
        .map(|row| {
            let cell = |i: usize| {
                row[i]
                    .clone()
                    .expect("a profile cell is never null")
                    .parse::<u64>()
                    .expect("a profile count is a number")
            };
            (
                row[0].clone().expect("column name"),
                (cell(1), cell(2), cell(3)),
            )
        })
        .collect()
}

// ------------------------------------------------------------------- F4

/// **F4's standing refusal.** The posture is read off the live connection
/// rather than compared against the constant that set it, and the negative is
/// exercised rather than assumed: a remote path is refused, naming the
/// extension DuckDB will not go and get.
///
/// `json` and `parquet` appearing as `STATICALLY_LINKED` is the other half of
/// the same decision — the readers are compiled in *because* nothing may be
/// fetched. If someone dropped the feature flags to save build time, this test
/// fails here, before any dataset test fails confusingly further down.
#[test]
fn f4_pins_extension_autoload_disabled_forever() {
    let db = AtlasDb::open_in_memory().expect("atlas");
    let posture = db.hardening().expect("hardening");

    assert!(
        !posture.autoload_known_extensions,
        "autoloading an extension mid-query is a network call nobody asked for"
    );
    assert!(
        !posture.autoinstall_known_extensions,
        "autoinstalling an extension is a download nobody asked for"
    );
    assert!(!posture.allow_community_extensions);
    assert!(
        posture.locked,
        "the three settings above must be locked, or they are a convention rather than a refusal"
    );
    for reader in ["json", "parquet"] {
        assert!(
            posture.statically_linked.iter().any(|e| e == reader),
            "F4 buys {reader} as a compiled-in reader; linked set was {:?}",
            posture.statically_linked
        );
    }

    // The negative, on the one input that would actually reach the network.
    let refusal = db
        .dataset_probe(
            DatasetFormat::Csv,
            "https://example.invalid/rows.csv",
            &DATASET_ROW_COUNT,
        )
        .expect_err("a remote path must be refused, not fetched");
    let message = refusal.to_string();
    assert!(
        message.contains("httpfs"),
        "the refusal must name the extension it declined to load: {message}"
    );
}

// -------------------------------------------------------------- routing

/// The three routing tables claim disjoint extensions.
///
/// A path claimed by both the document table and the tabular one would land in
/// `source.files` *and* `source.datasets` with two different meanings for the
/// same resource, and coverage would have to pick one to report.
#[test]
fn the_tabular_routing_table_is_disjoint_from_the_document_and_grammar_tables() {
    for format in DatasetFormat::ALL {
        for extension in format.extensions() {
            let path = format!("data/rows.{extension}");
            assert_eq!(format_for(&path), Some(*format));
            assert!(
                claims_for(&path).is_none(),
                "{path} is claimed by both the tabular and the document/grammar tables"
            );
        }
    }
    for path in ["README.md", "main.rs", "Cargo.toml", "run.sh"] {
        assert!(format_for(path).is_none(), "{path} is not a dataset");
        assert!(claims_for(path).is_some(), "{path} is still claimed");
    }
}

// ----------------------------------------------------- read in place

/// A knowledge source's CSV, JSON and Parquet files are registered as datasets
/// and read **in place**, and every canned query's answer is stored as derived
/// evidence carrying its input generation, its query identity and its own
/// output hash (A1 §6.4).
#[test]
fn datasets_are_registered_and_read_in_place_as_derived_evidence() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("rows.csv"),
        "title,body\nalpha,one\nbeta,two\nalpha,three\n",
    )
    .expect("write csv");
    std::fs::write(
        dir.path().join("events.ndjson"),
        "{\"kind\":\"a\",\"n\":1}\n{\"kind\":\"b\",\"n\":2}\n",
    )
    .expect("write ndjson");
    write_parquet(&dir.path().join("facts.parquet"));
    std::fs::write(dir.path().join("notes.md"), "# Heading\n\nbody\n").expect("write md");

    let recorded = record(&source("notes", dir.path(), &[]));
    let ScanRecord::Recorded { generation_id, .. } = &recorded.record else {
        panic!("expected a recorded generation, got {:?}", recorded.record);
    };
    assert!(!generation_id.is_empty());

    let datasets = recorded.db.datasets("notes", MAX_ROWS).expect("datasets");
    assert_eq!(datasets.len(), 3, "three datasets: {datasets:?}");

    let csv = dataset(&datasets, "rows.csv");
    assert_eq!(csv.format, DatasetFormat::Csv);
    assert_eq!(csv.columns, strings(&["title", "body"]));
    assert_eq!(csv.row_count, 3);
    assert!(!csv.truncated);
    // The reader identity is F7's second key input, and with no allowlist it
    // is the bare reader version.
    assert_eq!(
        csv.reader,
        reader_identity(DatasetFormat::Csv, &ContextFields::none())
    );
    assert_eq!(csv.dataset_key.len(), 64);

    let json = dataset(&datasets, "events.ndjson");
    assert_eq!(json.format, DatasetFormat::Json);
    assert_eq!(json.columns, strings(&["kind", "n"]));
    assert_eq!(json.row_count, 2);

    let parquet = dataset(&datasets, "facts.parquet");
    assert_eq!(parquet.format, DatasetFormat::Parquet);
    assert_eq!(parquet.columns, strings(&["label", "weight"]));
    assert_eq!(parquet.row_count, 3);

    // The document beside them is still a document: two routing tables, two
    // kinds of row, one resource each.
    let files = recorded.db.units("notes", MAX_ROWS).expect("units");
    assert!(files.iter().all(|u| u.relative_path == "notes.md"));

    // The deterministic aggregate.
    let csv_profile = profile(&recorded.db, "notes", "rows.csv");
    assert_eq!(csv_profile.get("title"), Some(&(3, 3, 2)));
    assert_eq!(csv_profile.get("body"), Some(&(3, 3, 3)));

    // A1 §6.4: every stored answer names the world it read, the question it
    // asked, and hashes its own output.
    let facts = recorded.db.dataset_facts("notes", MAX_ROWS).expect("facts");
    assert_eq!(facts.len(), datasets.len() * DATASET_QUERIES.len());
    for fact in &facts {
        let owner = dataset(&datasets, &fact.relative_path);
        assert_eq!(
            fact.dataset_key, owner.dataset_key,
            "an answer must name the exact input it was derived from"
        );
        assert!(
            fact.query_identity.starts_with(&format!("{}/", fact.query)),
            "the query identity must name its query: {}",
            fact.query_identity
        );
        assert!(
            fact.query_identity.contains('#'),
            "the query identity must carry a digest of the SQL that ran: {}",
            fact.query_identity
        );
        assert_eq!(
            fact.row_limit, MAX_ROWS as u64,
            "F12: every read is bounded"
        );
        assert_eq!(
            fact.output_hash,
            output_hash(&fact.columns, &fact.rows),
            "the stored output hash must be a hash of the stored output"
        );
    }

    // And every dataset left exactly one coverage row, saying `indexed` and
    // naming the reader that ran (F8).
    let coverage = recorded.db.coverage("notes", MAX_ROWS).expect("coverage");
    for path in ["rows.csv", "events.ndjson", "facts.parquet"] {
        let rows: Vec<_> = coverage
            .iter()
            .filter(|c| c.row.path.as_deref() == Some(path))
            .collect();
        assert_eq!(rows.len(), 1, "one coverage row per path, got {rows:?}");
        assert_eq!(rows[0].row.status, Coverage::Indexed);
        let detail = rows[0].row.detail.clone().unwrap_or_default();
        assert!(
            detail.contains("no context_fields declared"),
            "with no allowlist the coverage row must say so: {detail}"
        );
    }
}

// ------------------------------------------------------------- F10a

/// **F10a's default, and the whole point of the decision.** A source that
/// declares no `context_fields` gets its datasets registered, counted and
/// profiled — and exposes not one row's text.
///
/// The dataset here is deliberately unremarkable: a `title` column nobody
/// would think twice about publishing. The refusal is not a judgement about
/// the data; it is the default.
#[test]
fn f10a_a_source_with_no_context_fields_exposes_no_row_text() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("tickets.csv"),
        "title,email\nlogin fails,a@example.com\ncannot print,b@example.com\n",
    )
    .expect("write csv");

    let recorded = record(&source("support", dir.path(), &[]));

    // Registered, profiled, counted — none of that is exposure.
    let datasets = recorded.db.datasets("support", MAX_ROWS).expect("datasets");
    assert_eq!(datasets.len(), 1);
    assert_eq!(datasets[0].row_count, 2);
    assert_eq!(datasets[0].columns, strings(&["title", "email"]));
    assert_eq!(
        datasets[0].row_units, 0,
        "no allowlist means no context units"
    );
    assert_eq!(
        profile(&recorded.db, "support", "tickets.csv").get("email"),
        Some(&(2, 2, 2)),
        "an aggregate over a column is not an exposure of its values"
    );

    // The refusal itself.
    let units = recorded
        .db
        .row_units("support", MAX_ROWS)
        .expect("row units");
    assert!(
        units.is_empty(),
        "F10a: a source with no declared context_fields exposes no row text, got {units:?}"
    );

    // And nothing anywhere in the store carries a value out of the file.
    let facts = recorded
        .db
        .dataset_facts("support", MAX_ROWS)
        .expect("facts");
    for fact in &facts {
        for row in &fact.rows {
            for value in row.iter().flatten() {
                assert!(
                    !value.contains("@example.com"),
                    "a cell value escaped into derived evidence: {value}"
                );
            }
        }
    }
}

/// A declared allowlist exposes exactly the columns it names, in the order it
/// names them, and gives each row a content-derived identity that survives an
/// edit somewhere else in the file.
#[test]
fn f10a_a_declared_allowlist_exposes_only_its_columns_with_stable_row_identity() {
    let dir = tempfile::tempdir().expect("tempdir");
    let csv = dir.path().join("tickets.csv");
    std::fs::write(
        &csv,
        "title,email\nlogin fails,a@example.com\ncannot print,b@example.com\n",
    )
    .expect("write csv");

    let recorded = record(&source("support", dir.path(), &["title"]));
    let units = recorded
        .db
        .row_units("support", MAX_ROWS)
        .expect("row units");
    assert_eq!(units.len(), 2);
    assert_eq!(units[0].body, "title: login fails");
    assert_eq!(units[0].fields, strings(&["title"]));
    assert_eq!(units[0].basis, RowKeyBasis::Content);
    for unit in &units {
        assert!(
            !unit.body.contains('@'),
            "a column outside the allowlist reached a context unit: {}",
            unit.body
        );
    }
    let keys: Vec<String> = units.iter().map(|u| u.row_key.clone()).collect();
    assert_eq!(
        recorded.db.datasets("support", MAX_ROWS).expect("datasets")[0].row_units,
        2
    );

    // Delete the first row. The survivor keeps its name: a row's identity is
    // what it says, not where it sits.
    std::fs::write(&csv, "title,email\ncannot print,b@example.com\n").expect("rewrite csv");
    let mut journal = Journal::open(recorded._data.path()).expect("journal");
    let mut db = recorded.db;
    scan_and_record(
        &mut db,
        &mut journal,
        &source("support", dir.path(), &["title"]),
        None,
    )
    .expect("re-scan");
    let after = db.row_units("support", MAX_ROWS).expect("row units");
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].row_key, keys[1], "the surviving row kept its name");
    assert_eq!(after[0].ordinal, 0, "even though it moved");
}

/// Two rows the allowlist cannot tell apart are **both** re-keyed, and say so.
///
/// The honest half of row identity: the file does not support a stable name
/// for either of them, so neither gets one, and `key_basis` tells a consumer
/// which claim it may make.
#[test]
fn f10a_rows_the_allowlist_cannot_distinguish_are_honestly_re_keyed() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("rows.csv"),
        "title,note\nsame,one\nsame,two\nother,three\n",
    )
    .expect("write csv");

    let recorded = record(&source("notes", dir.path(), &["title"]));
    let units = recorded.db.row_units("notes", MAX_ROWS).expect("row units");
    assert_eq!(units.len(), 3);
    assert_eq!(units[0].basis, RowKeyBasis::ContentAndOrdinal);
    assert_eq!(units[1].basis, RowKeyBasis::ContentAndOrdinal);
    assert_ne!(units[0].row_key, units[1].row_key);
    assert_eq!(
        units[2].basis,
        RowKeyBasis::Content,
        "a row whose values are unique keeps a content-derived name"
    );
}

/// **Narrowing an allowlist retracts what the wider one exposed.**
///
/// Through machinery that already existed, which is the design: the allowlist
/// rides in the reader identity, a changed identity is a changed extraction,
/// and `stage_scan` supersedes the generation — taking its `context.row_units`
/// with it and leaving an eviction row that says the extractors changed rather
/// than claiming the bytes did.
#[test]
fn f10a_narrowing_an_allowlist_retracts_the_units_it_exposed() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("tickets.csv"),
        "title,email\nlogin fails,a@example.com\n",
    )
    .expect("write csv");

    let recorded = record(&source("support", dir.path(), &["title", "email"]));
    let mut db = recorded.db;
    let wide = db.row_units("support", MAX_ROWS).expect("row units");
    assert_eq!(wide.len(), 1);
    assert!(wide[0].body.contains("a@example.com"));

    // Same bytes, narrower allowlist.
    let mut journal = Journal::open(recorded._data.path()).expect("journal");
    let record = scan_and_record(
        &mut db,
        &mut journal,
        &source("support", dir.path(), &["title"]),
        None,
    )
    .expect("re-scan");
    let ScanRecord::Recorded { evicted, .. } = &record else {
        panic!("a narrowed allowlist is a changed extraction, got {record:?}");
    };
    assert!(evicted.is_some(), "the wider generation must be superseded");

    let narrow = db.row_units("support", MAX_ROWS).expect("row units");
    assert_eq!(narrow.len(), 1);
    assert_eq!(narrow[0].body, "title: login fails");
    assert!(
        !narrow[0].body.contains('@'),
        "the retracted column must be gone, not merely unlisted"
    );

    // And the eviction says *why*, honestly: the bytes did not change.
    let eviction = db
        .coverage("support", MAX_ROWS)
        .expect("coverage")
        .into_iter()
        .find(|c| c.row.status == Coverage::GenerationEvicted)
        .expect("an eviction row");
    let detail = eviction.row.detail.unwrap_or_default();
    assert!(
        detail.contains("extractor identities changed"),
        "the eviction must not claim the source bytes changed: {detail}"
    );
}

// -------------------------------------------------------------- F12

/// **F12: every dataset read is bounded**, and the answer says when the bound
/// bit rather than looking complete.
#[test]
fn f12_every_dataset_read_is_bounded() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut csv = String::from("n,title\n");
    for row in 0..(MAX_ROWS + 25) {
        csv.push_str(&format!("{row},row {row}\n"));
    }
    std::fs::write(dir.path().join("big.csv"), csv).expect("write csv");

    let recorded = record(&source("bulk", dir.path(), &["title"]));
    let datasets = recorded.db.datasets("bulk", MAX_ROWS).expect("datasets");
    assert_eq!(datasets[0].row_count, MAX_ROWS as u64);
    assert!(
        datasets[0].truncated,
        "a dataset past the cap must say the cap bit"
    );
    assert_eq!(
        datasets[0].row_units, MAX_ROWS as u64,
        "the context units are capped too"
    );
    let units = recorded.db.row_units("bulk", MAX_ROWS).expect("row units");
    assert_eq!(units.len(), MAX_ROWS);

    // Asking for more than the cap does not get more than the cap.
    let unbounded = recorded
        .db
        .row_units("bulk", usize::MAX)
        .expect("row units");
    assert_eq!(unbounded.len(), MAX_ROWS);
}

// --------------------------------------------------------- honesty

/// A dataset a reader cannot parse is a coverage fact, not a scan failure.
///
/// The other files in the source are still indexed, which is the property that
/// keeps one malformed file in a knowledge directory from costing the estate
/// everything else in it.
#[test]
fn an_unreadable_dataset_is_a_coverage_fact_not_a_scan_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("broken.parquet"), b"not parquet at all\n").expect("write");
    std::fs::write(dir.path().join("rows.csv"), "a\n1\n").expect("write csv");

    let recorded = record(&source("notes", dir.path(), &[]));
    let coverage = recorded.db.coverage("notes", MAX_ROWS).expect("coverage");
    let broken = coverage
        .iter()
        .find(|c| c.row.path.as_deref() == Some("broken.parquet"))
        .expect("a row for the broken dataset");
    assert_eq!(broken.row.status, Coverage::Error);

    let good = coverage
        .iter()
        .find(|c| c.row.path.as_deref() == Some("rows.csv"))
        .expect("a row for the good dataset");
    assert_eq!(good.row.status, Coverage::Indexed);

    // Both are registered — "this is a dataset and we could not read it" is
    // evidence, and an absent row would not be.
    let datasets = recorded.db.datasets("notes", MAX_ROWS).expect("datasets");
    assert_eq!(datasets.len(), 2);
    assert_eq!(dataset(&datasets, "broken.parquet").row_count, 0);
}

/// An estate-git walk reports a repository-resident dataset by the reason it
/// cannot read it, not as "unclaimed" — which would be a false statement about
/// the routing table.
#[test]
fn a_dataset_with_no_path_to_read_in_place_is_named_rather_than_mislabelled() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("rows.csv"), "a\n1\n").expect("write csv");
    let scan = scan_local_knowledge(&source("notes", dir.path(), &[])).expect("scan");
    assert!(scan.root.is_some(), "a filesystem walk has a root to read");
    assert_eq!(scan.datasets.len(), 1);

    // The constant the object-store walks report with is the one that names
    // the reason, and it is not the unclaimed one.
    assert!(DATASET_NO_ROOT.contains("read in place"));
    assert_ne!(
        DATASET_NO_ROOT,
        sergeant_rs::runtime::atlas::scan::UNCLAIMED
    );
}

// -------------------------------------------------------------- F11

/// **F11's named deferral, pinned.** `map` ships five verbs and exactly five.
///
/// `neighbors` and `changed` are deferred to S5/S6, where their consumers
/// exist. A verb that shipped now would answer from nothing — the same false
/// promise the empty-table doctrine refuses — so the absence is asserted
/// rather than left to a reviewer to notice.
#[test]
fn f11_map_ships_five_verbs_and_defers_neighbors_and_changed() {
    let output = Command::new(env!("CARGO_BIN_EXE_sgt"))
        .args(["map", "--help"])
        .output()
        .expect("run sgt map --help");
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).expect("utf8");
    // The verb list, not the prose: this command's own doc *names* the two
    // deferred verbs in order to say they are deferred, so the assertion has
    // to read the command table rather than the whole page.
    let verbs: Vec<String> = help
        .split("Commands:")
        .nth(1)
        .unwrap_or("")
        .split("Options:")
        .next()
        .unwrap_or("")
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|verb| *verb != "help")
        .map(str::to_string)
        .collect();
    assert_eq!(
        verbs,
        strings(&["repos", "stats", "outline", "symbol", "references"]),
        "`sgt map` ships five verbs and exactly five"
    );

    let status = Command::new(env!("CARGO_BIN_EXE_sgt"))
        .args(["intelligence", "--help"])
        .output()
        .expect("run sgt intelligence --help");
    assert!(status.status.success());
    assert!(String::from_utf8_lossy(&status.stdout).contains("status"));
}

//! S5 W1c — one Atlas database (A1 §5, owner correction 2026-08-29).
//!
//! A1 §5, "One Atlas DuckDB, multiple projection families", declares one
//! physical file (`atlas.duckdb`) carrying five logical schemas — `meta`,
//! `ops`, `source`, `git`, `context` — and gives the reason in one sentence:
//! *"DuckDB supports named schemas within one database, enabling cross-domain
//! joins without attaching/federating a fleet of databases."* Decision
//! A1-02's rationale is *"schemas provide separation without more
//! databases."*
//!
//! S3 shipped two files. The owner correction of 2026-08-29 ruled that the
//! code converges to the contract, and that two things must be *proven* by
//! this wave rather than assumed:
//!
//! * **Decision 4** — the ops↔source join A2 needs is proven by a test, not
//!   inferred from colocation:
//!   [`one_statement_joins_ops_work_identity_to_source_generations`].
//! * **The affordance restatement** — deleting the projection file used to be
//!   lossless. It is not any more, and the honest sentence is measured here
//!   rather than only written down:
//!   [`deleting_atlas_duckdb_rebuilds_ops_and_loses_source_facts`].
//!
//! The one-owner collapse (decision 3) is pinned by
//! `tests/x1_atlas_substrate.rs`'s `atlas_database_has_exactly_one_owner`,
//! which this wave widened to the whole of `src/`; the note where M5's `t2`
//! used to stand says why there is now one test rather than two.

use std::collections::BTreeSet;

use serde_json::json;

use sergeant_rs::domain::event::{Event, EventDraft, EventSource, rfc3339_utc_now};
use sergeant_rs::domain::source::{AuthorityClass, SourceKind, UnitKind};
use sergeant_rs::domain::work::KIND_WORK_SUBMITTED;
use sergeant_rs::runtime::atlas::db::{
    Analytics, SCHEMAS, WORK_GENERATION_JOIN_SQL, atlas_db_path,
};
use sergeant_rs::runtime::atlas::overlay::overlay_source_name;
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{ScannedFile, ScannedUnit, SourceScan};
use sergeant_rs::runtime::atlas::tabular::ContextFields;
use sergeant_rs::runtime::atlas::text::TEXT_EXTRACTOR;
use sergeant_rs::runtime::journal::{Journal, JournalError};

const MINE: &str = "01MINE0000000000000000000A";
const OTHER: &str = "01OTHER000000000000000000B";

// ------------------------------------------------------------- fixtures

fn submitted(seq: u64, work_id: &str) -> Result<Event, JournalError> {
    Ok(
        EventDraft::new(EventSource::new("daemon", "test"), KIND_WORK_SUBMITTED, {
            json!({"work": {
                "id": work_id, "intent": "do it", "state": "pending",
                "created_by": "test", "created_at": "2026-01-01T00:00:00.000Z",
                "origin_client": "cli", "repositories": [],
            }})
        })
        .with_work_id(work_id)
        .into_event(seq),
    )
}

/// One scanned file's worth of a source, built by hand: these tests are about
/// where the rows live, not about acquisition. Same shape as
/// `tests/w1_deterministic_filter.rs`'s fixtures, trimmed to one document
/// unit.
fn scan(source_name: &str, content_key: &str, body: &str) -> SourceScan {
    let file = ScannedFile {
        relative_path: "src/main.rs".to_string(),
        content_hash: format!("hash/{content_key}"),
        extractor: TEXT_EXTRACTOR.to_string(),
        local_key: format!("key/{content_key}"),
        byte_len: body.len() as u64,
        mtime_millis: None,
        units: vec![ScannedUnit {
            ordinal: 0,
            kind: UnitKind::Document,
            heading_level: None,
            title: None,
            byte_start: 0,
            byte_end: body.len() as u64,
            text: body.to_string(),
        }],
        syntax: None,
    };
    SourceScan {
        source_name: source_name.to_string(),
        kind: SourceKind::EstateGit,
        authority: AuthorityClass::EstateMutable,
        content_key: content_key.to_string(),
        revision: None,
        observed_at: rfc3339_utc_now(),
        extractors: BTreeSet::from([TEXT_EXTRACTOR.to_string()]),
        files: vec![file],
        coverage: Vec::new(),
        datasets: Vec::new(),
        root: None,
        context_fields: ContextFields::none(),
    }
}

// ------------------------------------- decision 4: prove the join

/// **Decision 4.** A Work's identity lives in `ops`; the generations derived
/// from its surface live in `source`. A1 §5's stated reason for one database
/// is that relating those two is an ordinary join. This runs that join — one
/// statement, two schemas, one database — and gets rows back.
///
/// Three assertions, and the negative one is why the test is worth writing:
///
/// 1. the join returns the Work that has an overlay generation, carrying
///    columns from *both* schemas (so neither half can be answered from one
///    schema alone);
/// 2. it does **not** return a Work with no overlay generation over this
///    repository — a join that returned every `ops.work` row would pass a
///    "returns rows" check and prove nothing;
/// 3. the statement contains no `ATTACH` and names no second database. Two
///    files could not have run it at all: each store's one-owner invariant
///    forbids the other's token, which is precisely the cost the owner
///    correction named.
#[test]
fn one_statement_joins_ops_work_identity_to_source_generations() {
    let data = tempfile::tempdir().expect("data dir");

    // The daemon's own order and the daemon's own handles: fold `ops` first
    // (which creates the file and every ops table), then derive the Atlas
    // half from that same open database. `AtlasDb::open` would be a second
    // DuckDB instance over the same file — see `Analytics::atlas` — and the
    // join below is exactly the read that a split instance would answer
    // wrongly, with no error, by seeing only one of the two schemas' rows.
    let mut analytics =
        Analytics::rebuild(data.path(), vec![submitted(1, MINE), submitted(2, OTHER)])
            .expect("fold ops");
    assert_eq!(analytics.last_seq(), 2);

    let mut journal = Journal::open(data.path()).expect("journal");
    let mut db = analytics
        .atlas()
        .expect("atlas handle on the same database");

    // One physical database, A1 §5's five logical schemas — read back out of
    // DuckDB's own catalog, not echoed from the constant.
    assert_eq!(db.schema_names().expect("schemas"), SCHEMAS);
    assert_eq!(SCHEMAS, ["context", "git", "meta", "ops", "source"]);

    record_scan(
        &mut db,
        &mut journal,
        &scan("repo-a", "repo-a@key-1", "fn base() {}\n"),
        None,
    )
    .expect("record base");
    record_scan(
        &mut db,
        &mut journal,
        &scan(
            &overlay_source_name(MINE, "repo-a"),
            "repo-a@base+mine",
            "fn overlaid() {}\n",
        ),
        None,
    )
    .expect("record overlay");

    // Asked of the projection, which is what makes the `ops` half current
    // (its mutable tables materialize at read time); the `source` half was
    // written through `db`, a second handle on this same database.
    let joined = analytics.work_overlay_generations("repo-a").expect("join");

    // 1. Both halves, from both schemas, in one row.
    assert_eq!(
        joined.len(),
        1,
        "exactly the one Work with an overlay: {joined:?}"
    );
    let row = &joined[0];
    assert_eq!(row.work_id, MINE, "the ops half");
    assert_eq!(
        row.work_state, "pending",
        "an ops.work column, not a source one"
    );
    assert_eq!(
        row.source_name,
        overlay_source_name(MINE, "repo-a"),
        "the source half"
    );
    assert_eq!(
        row.content_key, "repo-a@base+mine",
        "a source.generations column"
    );
    assert!(!row.observed_at.is_empty());

    // 2. The negative: `OTHER` is a real row in `ops.work` with no overlay
    //    generation, so a join that leaked every Work would fail here.
    assert!(
        !joined.iter().any(|r| r.work_id == OTHER),
        "a Work with no overlay generation must not be joined in: {joined:?}"
    );

    // 3. The statement itself: one database, no federation.
    let sql = WORK_GENERATION_JOIN_SQL.to_ascii_uppercase();
    assert!(
        !sql.contains("ATTACH"),
        "the whole point of one database is that this join needs no ATTACH: \
         {WORK_GENERATION_JOIN_SQL}"
    );
    assert!(
        WORK_GENERATION_JOIN_SQL.contains("ops.work")
            && WORK_GENERATION_JOIN_SQL.contains("source.generations"),
        "both schemas must be addressed by name in the one statement: \
         {WORK_GENERATION_JOIN_SQL}"
    );
}

// --------------------- the affordance, restated and then measured

/// **The recovery affordance changed meaning, and this is the measurement of
/// what it means now.**
///
/// Before W1c, `ops` was alone in `projections/sergeant.duckdb` and deleting
/// that file was lossless by construction: every row refolded from the
/// journal. `ops` is now a schema in `atlas.duckdb`, so deleting *that* file
/// still restores every `ops` row exactly — and discards every confirmed
/// source generation, which no journal replay reproduces (F1) and which must
/// be re-scanned.
///
/// Both halves are asserted here, together, because a doc sentence that says
/// so is exactly the prose-vs-code drift this program keeps recording. The
/// loss is acceptable under ruling §4 only because it is reported rather than
/// silent, and the third assertion is that report: the store answers "no
/// confirmed generation" rather than pretending the evidence is still there.
///
/// The daemon's own rebuild path never does this — it drops the `ops` schema
/// and leaves the file alone (`Analytics::begin_rebuild`). This test is about
/// what an operator who deletes the file gets.
#[test]
fn deleting_atlas_duckdb_rebuilds_ops_and_loses_source_facts() {
    let data = tempfile::tempdir().expect("data dir");
    // Built once and replayed twice: these are the same events the daemon
    // would find in the journal on both starts, and an event rebuilt from
    // scratch would carry a fresh `submitted_ms` and make the comparison
    // below vacuously about clock time rather than about the fold.
    let journal: Vec<Event> = vec![
        submitted(1, MINE).expect("event"),
        submitted(2, OTHER).expect("event"),
    ];
    let replay = || journal.iter().cloned().map(Ok);

    let mut analytics = Analytics::rebuild(data.path(), replay()).expect("fold ops");
    let ops_before = analytics.table_rows("work").expect("ops.work rows");

    let mut event_log = Journal::open(data.path()).expect("journal");
    let mut db = analytics
        .atlas()
        .expect("atlas handle on the same database");
    record_scan(
        &mut db,
        &mut event_log,
        &scan("repo-a", "repo-a@key-1", "fn base() {}\n"),
        None,
    )
    .expect("record base");
    let sources_before = db.indexed_sources().expect("indexed sources");
    assert_eq!(
        sources_before.len(),
        1,
        "the fixture must actually have persisted a source fact"
    );
    drop(db);
    drop(event_log);
    drop(analytics);

    // The operator's gesture: delete the file.
    let path = atlas_db_path(data.path());
    std::fs::remove_file(&path).expect("delete the atlas database");
    let _ = std::fs::remove_file(path.with_extension("duckdb.wal"));

    let mut analytics = Analytics::rebuild(data.path(), replay()).expect("refold ops");
    let ops_after = analytics.table_rows("work").expect("ops.work rows");
    let db = analytics
        .atlas()
        .expect("atlas handle on the same database");
    let sources_after = db.indexed_sources().expect("indexed sources");

    // Half one, unchanged from the old contract: `ops` comes back row for row.
    assert_eq!(
        ops_before.rows, ops_after.rows,
        "every ops row must still refold from the journal"
    );
    assert_eq!(ops_before.columns, ops_after.columns);

    // Half two, the part that is new and that no doc may keep denying: the
    // persisted source facts are gone.
    assert!(
        sources_after.is_empty(),
        "deleting atlas.duckdb discards persisted source generations; a test that \
         found them still there would mean this file had stopped being the one \
         database A1 §5 declares: {sources_after:?}"
    );

    // Half three: the loss is reported, not silent. The store says it has no
    // confirmed generation for `repo-a` rather than answering as if it had.
    assert!(
        !sources_after
            .iter()
            .any(|status| status.source_name == "repo-a"),
        "a source with no confirmed generation must not be reported as indexed"
    );
    assert_eq!(
        sources_before[0].source_name, "repo-a",
        "and the before-state must be the thing that went missing, not something else"
    );
}

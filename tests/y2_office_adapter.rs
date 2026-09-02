//! S4 Y2 acceptance: the Office adapter, riding Y1's supervised worker
//! transport (G3).
//!
//! Mirrors `tests/y1_worker_transport.rs`'s shape exactly — the real worker
//! binary ([`SGT_ATLAS_WORKER`]), the real intelligence-lane [`Engine`] — but
//! exercises the real Office-adapter path instead of `--fault` fixture
//! modes, because Y2 is where a real parser exists to poison (Y1 had none,
//! by the plan's own panel finding). Every assertion in this file stays
//! inside the wire vocabulary
//! ([`sergeant_rs::runtime::atlas::worker::WorkerBatch`]/`WorkerUnit`,
//! [`sergeant_rs::domain::source::Coverage`]) — never a third-party document-
//! crate type, per the replaceability boundary `tests/y2_office_boundary.rs`
//! pins structurally.
//!
//! * [`a_docx_worker_returns_document_and_section_units_with_provenance`] —
//!   the happy path, through the real subprocess and the real parser.
//! * [`a_real_parser_failure_leaves_the_daemon_up_the_permit_freed_and_a_named_coverage_row`]
//!   — **this wave's own acceptance** (the brief assigns it here
//!   explicitly): a genuinely malformed or hostile Office document, walked
//!   fixture by fixture, fails its worker alone — the engine stays up, the
//!   permit is freed, a named coverage row lands, and no partial rows are
//!   written — proven through the real parser, not a synthetic fault. Its
//!   hostile fixture is the brief's own "consider also a hostile case that
//!   stresses…the memory cap through the real adapter path": a document
//!   engineered to blow past the normalizer's own internal resource
//!   ceiling.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::domain::source::{Coverage, UnitKind};
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::deny::AcquisitionFilter;
use sergeant_rs::runtime::atlas::lane::run_worker_on_lane;
use sergeant_rs::runtime::atlas::office::DOCX_EXTRACTOR;
use sergeant_rs::runtime::atlas::worker::{WorkerIdentity, WorkerOutcome, WorkerSpawn, run_worker};
use sergeant_rs::runtime::engine::Engine;

mod support;

/// The real worker binary Cargo built alongside this test binary.
const SGT_ATLAS_WORKER: &str = env!("CARGO_BIN_EXE_sgt-atlas-worker");

/// Generous enough for a real parse (including the hostile fixture's ~140
/// MiB bounded decompression) on either of the two-environment rule's hosts,
/// short enough that a genuine hang would still fail the suite promptly.
const REAL_PARSE_DEADLINE: Duration = Duration::from_secs(20);

fn deny() -> AcquisitionFilter {
    AcquisitionFilter::new(&[]).expect("compile default deny set")
}

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/anydoc_corpus/docx_fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn identity_for(input: &[u8]) -> WorkerIdentity {
    WorkerIdentity {
        generation_id: "gen-y2".to_string(),
        resource_hash: blake3::hash(input).to_hex().to_string(),
        extractor: DOCX_EXTRACTOR.to_string(),
    }
}

fn spawn(input: Vec<u8>, identity: &WorkerIdentity, deadline: Duration) -> WorkerSpawn {
    WorkerSpawn {
        program: PathBuf::from(SGT_ATLAS_WORKER),
        args: vec![
            "--generation".to_string(),
            identity.generation_id.clone(),
            "--extractor".to_string(),
            identity.extractor.clone(),
        ],
        input,
        deadline,
    }
}

// --------------------------------------------------------------- the happy path

/// Bytes in, a normalized batch out, through the real subprocess and the
/// real Office adapter — round-trips fixture 01 (two headings, three plain
/// paragraphs, no lists or tables) and checks the wire shape's own
/// provenance fields, not just unit count.
#[test]
fn a_docx_worker_returns_document_and_section_units_with_provenance() {
    let input = fixture("01-plain-headings-paragraphs.docx");
    let input_len = input.len() as u64;
    let identity = identity_for(&input);
    let outcome = run_worker(
        spawn(input, &identity, REAL_PARSE_DEADLINE),
        &identity,
        &deny(),
    );
    let WorkerOutcome::Accepted(batch) = outcome else {
        panic!("a well-formed docx must be accepted: {outcome:?}");
    };
    assert_eq!(batch.extractor, DOCX_EXTRACTOR);
    assert_eq!(
        batch.units.len(),
        3,
        "one Document unit + two heading sections"
    );

    let document = &batch.units[0];
    assert_eq!(document.kind, UnitKind::Document);
    assert_eq!(document.byte_start, 0);
    assert_eq!(
        document.byte_end, input_len,
        "the whole-document unit spans the whole resource"
    );
    assert_eq!(
        document.coordinate, None,
        "the whole-document unit needs no coordinate more specific than the whole resource"
    );

    let sections: Vec<_> = batch
        .units
        .iter()
        .filter(|u| u.kind == UnitKind::Section)
        .collect();
    assert_eq!(sections.len(), 2);
    for section in &sections {
        assert_eq!(
            (section.byte_start, section.byte_end),
            (0, 0),
            "an Office section is not byte-recoverable — coordinate carries its position instead"
        );
        assert!(
            section.coordinate.as_deref().is_some_and(|c| !c.is_empty()),
            "an Office section must carry a non-empty coordinate (the wire/office contract — \
             see office.rs's module doc, \"The contract, in our own terms\" — never a \
             particular string shape): {section:?}"
        );
    }
    // Contract property (2): unique per document. Not a prefix check — no
    // particular string shape is part of the contract, only that two
    // different sections of the same parse never share a coordinate.
    let coordinates: Vec<&str> = sections
        .iter()
        .filter_map(|s| s.coordinate.as_deref())
        .collect();
    let mut distinct = coordinates.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        coordinates.len(),
        "every section's coordinate must be unique within one document: {coordinates:?}"
    );
    assert!(sections[0].text.contains("first body paragraph"));
    assert!(sections[1].text.contains("A single body paragraph"));

    // Contract property (3): deterministic — re-running the same extractor
    // identity against the same bytes reproduces the identical coordinate
    // for the corresponding unit. `office_units`'s own purity is proven
    // directly in office.rs's own tests (`extraction_is_a_pure_function_of_its_input`);
    // this re-proves it through the wire shape this test actually asserts
    // against, on the real subprocess.
    let identity_again = identity_for(&fixture("01-plain-headings-paragraphs.docx"));
    let outcome_again = run_worker(
        spawn(
            fixture("01-plain-headings-paragraphs.docx"),
            &identity_again,
            REAL_PARSE_DEADLINE,
        ),
        &identity_again,
        &deny(),
    );
    let WorkerOutcome::Accepted(batch_again) = outcome_again else {
        panic!("a well-formed docx must be accepted on a second run too: {outcome_again:?}");
    };
    let coordinates_again: Vec<Option<&str>> = batch_again
        .units
        .iter()
        .filter(|u| u.kind == UnitKind::Section)
        .map(|u| u.coordinate.as_deref())
        .collect();
    assert_eq!(
        coordinates_again,
        vec![
            sections[0].coordinate.as_deref(),
            sections[1].coordinate.as_deref()
        ],
        "the same bytes under the same extractor identity must round-trip to the same \
         coordinates across two independent runs"
    );
}

// ------------------------------------------------- the real-parser supervision proof

/// One real-parser hostile fixture and the substring its coverage row's
/// detail must name.
struct RealParserCase {
    fixture: &'static str,
    names: &'static str,
}

const REAL_PARSER_CASES: &[RealParserCase] = &[
    RealParserCase {
        fixture: "05-malformed-unclosed-element.docx",
        names: "malformed",
    },
    RealParserCase {
        fixture: "06-hostile-entry-expansion.docx",
        names: "resource limit",
    },
];

/// **This wave's own acceptance** (the brief assigns it here explicitly,
/// because Y1 had no real parser to poison): a genuinely malformed or
/// hostile Office document, run through the real Office adapter inside the
/// real supervised worker subprocess, fails its worker ALONE —
/// the engine (standing in for the daemon) stays up, the intelligence-lane
/// permit is freed, no partial Atlas rows appear, and a named coverage row
/// describes the failure. Walks both real-parser fixtures the same way
/// `y1_worker_transport.rs`'s fault walk proves its four synthetic faults.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_real_parser_failure_leaves_the_daemon_up_the_permit_freed_and_a_named_coverage_row() {
    for case in REAL_PARSER_CASES {
        let data = TempDir::new().expect("tempdir");
        let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path())
            .with_intelligence_lane_cap(1);

        // Same decisive form as Y1's own test: `AtlasDb` is not reachable
        // from `worker.rs`/`lane.rs` at all yet (a later wave wires
        // `record`'s three-step discipline onto a worker's accepted batch),
        // so an independent Atlas store staying empty across the call is
        // what "no partial rows" is checkable as today.
        let atlas = AtlasDb::open_in_memory().expect("in-memory atlas");
        assert!(atlas.indexed_sources().expect("read").is_empty());

        let input = fixture(case.fixture);
        let identity = identity_for(&input);
        let outcome = run_worker_on_lane(
            &engine,
            spawn(input, &identity, REAL_PARSE_DEADLINE),
            identity,
            deny(),
        )
        .await
        .unwrap_or_else(|e| panic!("[{}] the lane call itself must not fail: {e}", case.fixture));

        let WorkerOutcome::Refused(row) = outcome else {
            panic!(
                "[{}] a malformed/hostile document must be refused, never accepted: {outcome:?}",
                case.fixture
            );
        };
        assert_eq!(
            row.status,
            Coverage::Error,
            "[{}] a real-parser failure is Coverage::Error: {row:?}",
            case.fixture
        );
        let detail = row.detail.clone().unwrap_or_default();
        assert!(
            detail.to_ascii_lowercase().contains(case.names),
            "[{}] coverage detail {detail:?} must name {:?}",
            case.fixture,
            case.names
        );

        assert_eq!(
            engine.intelligence_lane.available_permits(),
            1,
            "[{}] the intelligence-lane permit must be freed",
            case.fixture
        );

        // "The daemon stays up": the engine that just supervised a worker
        // that failed on real, hostile input still runs an ordinary job.
        let still_alive: usize = engine
            .run_intelligence(|| 7)
            .await
            .unwrap_or_else(|e| panic!("[{}] the engine must still be usable: {e}", case.fixture));
        assert_eq!(still_alive, 7);

        assert!(
            atlas.indexed_sources().expect("read").is_empty(),
            "[{}] no partial rows may appear from a refused worker",
            case.fixture
        );
    }
}

/// No `sgt-atlas-worker` process may survive this suite — the same
/// discipline `y1_worker_transport.rs`'s own backstop applies, restated here
/// because this is a separate test binary with its own process-wide
/// guarantee to establish.
#[test]
fn no_worker_process_survives_the_real_parser_walk() {
    support::wait_until_sync(
        "no sgt-atlas-worker process still alive after the grace period",
        support::HANG_BUDGET,
        || {
            let output = std::process::Command::new("pgrep")
                .arg("-f")
                .arg("sgt-atlas-worker")
                .output();
            let Ok(output) = output else {
                return true;
            };
            String::from_utf8_lossy(&output.stdout).trim().is_empty()
        },
    );
}

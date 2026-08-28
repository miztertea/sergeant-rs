//! S4 Y4 acceptance: the mail (`.eml`) adapter, riding Y1's supervised
//! worker transport (G4 — ADOPT, `tests/fixtures/mail_corpus/SPIKE-G4.md`).
//!
//! Mirrors `tests/y2_office_adapter.rs`/`tests/y3_zip_adapter.rs`'s own
//! shape exactly — the real worker binary ([`SGT_ATLAS_WORKER`]), the real
//! intelligence-lane [`Engine`] — proving the real adapter
//! (`sergeant_rs::runtime::atlas::mail`) runs inside the real supervised
//! subprocess.
//!
//! * [`a_mail_worker_returns_message_shape_and_attachment_with_provenance`]
//!   — the happy path, through the real subprocess and the real parser:
//!   text+html bodies (with the synthesized-HTML caveat proven absent),
//!   message id/references, and an attachment declared with provenance.
//! * [`a_real_parser_failure_leaves_the_daemon_up_the_permit_freed_and_a_named_coverage_row`]
//!   — **this wave's own acceptance** (the brief assigns it here, the same
//!   shape Y2/Y3's own hostile-input tests establish): the three ways this
//!   adapter honestly refuses a whole message — unparseable (fixture 06),
//!   degraded (the diagnostic broken-MIME fixture), and sealed/S-MIME —
//!   each fail their own worker ALONE.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::domain::source::Coverage;
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::deny::AcquisitionFilter;
use sergeant_rs::runtime::atlas::lane::run_worker_on_lane;
use sergeant_rs::runtime::atlas::mail::MAIL_EXTRACTOR;
use sergeant_rs::runtime::atlas::worker::{WorkerIdentity, WorkerOutcome, WorkerSpawn, run_worker};
use sergeant_rs::runtime::engine::Engine;

const SGT_ATLAS_WORKER: &str = env!("CARGO_BIN_EXE_sgt-atlas-worker");

/// Generous on either of the two-environment rule's hosts; short enough that
/// a genuine hang still fails the suite promptly.
const REAL_RUN_DEADLINE: Duration = Duration::from_secs(20);

fn deny() -> AcquisitionFilter {
    AcquisitionFilter::new(&[]).expect("compile default deny set")
}

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/mail_corpus")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn identity_for(input: &[u8]) -> WorkerIdentity {
    WorkerIdentity {
        generation_id: "gen-y4".to_string(),
        resource_hash: blake3::hash(input).to_hex().to_string(),
        extractor: MAIL_EXTRACTOR.to_string(),
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

// ------------------------------------------------------------- the happy path

/// Bytes in, a normalized batch out, through the real subprocess and the
/// real mail adapter — round-trips fixture 03 (a text body plus one
/// attachment) and checks the wire shape's own provenance fields.
#[test]
fn a_mail_worker_returns_message_shape_and_attachment_with_provenance() {
    let input = fixture("03-with-attachment.eml");
    let identity = identity_for(&input);
    let outcome = run_worker(
        spawn(input, &identity, REAL_RUN_DEADLINE),
        &identity,
        &deny(),
    );
    let WorkerOutcome::Accepted(batch) = outcome else {
        panic!("a well-formed message must be accepted: {outcome:?}");
    };
    assert_eq!(batch.extractor, MAIL_EXTRACTOR);
    assert_eq!(
        batch.units.len(),
        1,
        "fixture 03 has a genuine text body and no genuine (non-synthesized) HTML body: {:?}",
        batch.units
    );
    let text_unit = &batch.units[0];
    assert_eq!(text_unit.coordinate.as_deref(), Some("text-body"));
    assert!(text_unit.text.contains("Report attached") || !text_unit.text.is_empty());

    assert_eq!(
        batch.declared_children.len(),
        1,
        "manifest attachment_count"
    );
    let attachment = &batch.declared_children[0];
    assert_eq!(attachment.name, "report.txt");
    assert_eq!(attachment.relative_path, "report.txt");

    // Contract property: deterministic — the same bytes under the same
    // extractor identity reproduce the identical wire shape across two
    // independent runs (the same property `y2_office_adapter.rs`'s own
    // happy-path test proves through the wire, mirrored here).
    let input_again = fixture("03-with-attachment.eml");
    let identity_again = identity_for(&input_again);
    let outcome_again = run_worker(
        spawn(input_again, &identity_again, REAL_RUN_DEADLINE),
        &identity_again,
        &deny(),
    );
    let WorkerOutcome::Accepted(batch_again) = outcome_again else {
        panic!("a well-formed message must be accepted on a second run too: {outcome_again:?}");
    };
    assert_eq!(batch_again.units, batch.units);
    assert_eq!(batch_again.declared_children, batch.declared_children);
}

/// A genuine `multipart/alternative` HTML body must reach the wire as its
/// own unit — proves the synthesized-HTML caveat's fix through the real
/// subprocess, not only in `mail.rs`'s own in-process tests.
#[test]
fn a_genuine_html_body_reaches_the_wire_and_a_synthesized_one_does_not() {
    let alternative = fixture("02-multipart-alternative.eml");
    let identity = identity_for(&alternative);
    let outcome = run_worker(
        spawn(alternative, &identity, REAL_RUN_DEADLINE),
        &identity,
        &deny(),
    );
    let WorkerOutcome::Accepted(batch) = outcome else {
        panic!("must be accepted: {outcome:?}");
    };
    let coordinates: Vec<&str> = batch
        .units
        .iter()
        .filter_map(|u| u.coordinate.as_deref())
        .collect();
    assert!(
        coordinates.contains(&"text-body") && coordinates.contains(&"html-body"),
        "a real multipart/alternative message must carry both units: {coordinates:?}"
    );

    let plain_only = fixture("01-plain-text.eml");
    let plain_identity = identity_for(&plain_only);
    let plain_outcome = run_worker(
        spawn(plain_only, &plain_identity, REAL_RUN_DEADLINE),
        &plain_identity,
        &deny(),
    );
    let WorkerOutcome::Accepted(plain_batch) = plain_outcome else {
        panic!("must be accepted: {plain_outcome:?}");
    };
    let plain_coordinates: Vec<&str> = plain_batch
        .units
        .iter()
        .filter_map(|u| u.coordinate.as_deref())
        .collect();
    assert_eq!(
        plain_coordinates,
        vec!["text-body"],
        "mail-parser synthesizes an html_body alias from a plain-text-only source; this must \
         NOT reach the wire as a second unit (caveat 2, mail.rs's own module doc)"
    );
}

// ------------------------------------------------- the honest-refusal proof

/// One hostile/malformed fixture and the substring its coverage row's
/// detail must name.
struct RefusalCase {
    fixture: &'static str,
    names: &'static str,
}

const REFUSAL_CASES: &[RefusalCase] = &[
    RefusalCase {
        fixture: "06-malformed-no-headers.eml",
        names: "no rfc 5322 header",
    },
    RefusalCase {
        fixture: "diagnostic-not-manifest-broken-mime.eml",
        names: "degraded silently",
    },
];

/// **This wave's own acceptance** (the brief assigns it here, the shape
/// Y2/Y3's own hostile-input tests establish): a message this adapter
/// honestly cannot read — unparseable, or the caveat-1 degraded-parse
/// signal — fails its own worker ALONE. The engine stays up, the
/// intelligence-lane permit is freed, no partial Atlas rows appear, and the
/// coverage row names the reason.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_real_parser_failure_leaves_the_daemon_up_the_permit_freed_and_a_named_coverage_row() {
    for case in REFUSAL_CASES {
        let data = TempDir::new().expect("tempdir");
        let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path())
            .with_intelligence_lane_cap(1);
        let atlas = AtlasDb::open_in_memory().expect("in-memory atlas");
        assert!(atlas.indexed_sources().expect("read").is_empty());

        let input = fixture(case.fixture);
        let identity = identity_for(&input);
        let outcome = run_worker_on_lane(
            &engine,
            spawn(input, &identity, REAL_RUN_DEADLINE),
            identity,
            deny(),
        )
        .await
        .unwrap_or_else(|e| panic!("[{}] the lane call itself must not fail: {e}", case.fixture));

        let WorkerOutcome::Refused(row) = outcome else {
            panic!(
                "[{}] must be refused, never accepted: {outcome:?}",
                case.fixture
            );
        };
        assert_eq!(row.status, Coverage::Error, "[{}] {row:?}", case.fixture);
        let detail = row.detail.clone().unwrap_or_default().to_ascii_lowercase();
        assert!(
            detail.contains(case.names),
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

/// A structurally S/MIME-sealed message gets its own honest refusal too —
/// proven separately from [`REFUSAL_CASES`] because its own coverage detail
/// names a different substring (`sealed`, not `degraded`/`no rfc 5322`).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_sealed_message_is_refused_through_the_real_subprocess() {
    let raw = b"From: a@example.com\r\nTo: b@example.com\r\nSubject: sealed\r\nDate: Mon, 1 Jan 2024 00:00:00 +0000\r\nMIME-Version: 1.0\r\nContent-Type: application/pkcs7-mime; smime-type=enveloped-data\r\nContent-Transfer-Encoding: base64\r\n\r\nAAAA\r\n".to_vec();

    let data = TempDir::new().expect("tempdir");
    let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path())
        .with_intelligence_lane_cap(1);
    let identity = identity_for(&raw);
    let outcome = run_worker_on_lane(
        &engine,
        spawn(raw, &identity, REAL_RUN_DEADLINE),
        identity,
        deny(),
    )
    .await
    .expect("the lane call itself must not fail");
    let WorkerOutcome::Refused(row) = outcome else {
        panic!("a sealed message must be refused, never accepted: {outcome:?}");
    };
    assert_eq!(row.status, Coverage::Error, "{row:?}");
    assert!(
        row.detail
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains("sealed"),
        "coverage detail must name the sealed status"
    );
    assert_eq!(engine.intelligence_lane.available_permits(), 1);
}

/// No `sgt-atlas-worker` process may survive this suite.
#[test]
fn no_worker_process_survives_the_mail_adapter_walk() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let output = std::process::Command::new("pgrep")
            .arg("-f")
            .arg("sgt-atlas-worker")
            .output();
        let Ok(output) = output else {
            return;
        };
        let listing = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if listing.is_empty() {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!("an sgt-atlas-worker process is still alive after the grace period: {listing}");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

//! S4 Y3 acceptance: the bounded-ZIP adapter, riding Y1's supervised worker
//! transport (G5 as AMENDED 2026-08-28).
//!
//! Mirrors `tests/y2_office_adapter.rs`'s own shape — the real worker binary
//! ([`SGT_ATLAS_WORKER`]), the real intelligence-lane [`Engine`] — proving
//! the real container adapter (`sergeant_rs::runtime::atlas::archive`) runs
//! inside the real supervised subprocess, and that the daemon-side
//! [`sergeant_rs::runtime::atlas::worker::validate_batch`] AUTHORITY still
//! runs, for real, against declared children a real adapter produced (not
//! Y1's synthetic `--declare-child` fixture flag).
//!
//! * [`a_zip_worker_declares_admitted_children_through_the_real_subprocess`]
//!   — the happy path plus two hostile-but-well-formed entries (a symlink, a
//!   duplicate name), through the real subprocess and the real adapter.
//! * [`an_archive_level_refusal_fails_its_own_worker_alone`] — **this wave's
//!   own acceptance**, the same shape Y2's own hostile-document test proves:
//!   an archive that trips a whole-archive bound fails its own worker alone
//!   — engine up, permit freed, no partial rows, a named coverage row.
//! * The overlapping/quine defence itself — this wave's closed research item
//!   — is proven directly, in-process, by
//!   `runtime/atlas/archive.rs::overlapping_files_refuse_the_whole_archive_before_any_entry_opens`;
//!   this file does not re-build that fixture a second time.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::domain::source::Coverage;
use sergeant_rs::runtime::atlas::archive::ZIP_EXTRACTOR;
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::deny::AcquisitionFilter;
use sergeant_rs::runtime::atlas::lane::run_worker_on_lane;
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
        .join("tests/fixtures/zip_corpus/zip_fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn identity_for(input: &[u8]) -> WorkerIdentity {
    WorkerIdentity {
        generation_id: "gen-y3".to_string(),
        resource_hash: blake3::hash(input).to_hex().to_string(),
        extractor: ZIP_EXTRACTOR.to_string(),
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

/// Bytes in, real admitted children out, through the real subprocess and the
/// real bounded-ZIP adapter — `01-plain-and-directory.zip` (every entry
/// admitted, the directory marker is not a child) and, in the same run,
/// `02-symlink.zip`/`03-duplicate-name.zip` (one admitted, one hostile entry
/// refused — but the batch as a WHOLE still succeeds, because only the
/// refused entry, not the archive, was unsafe).
#[test]
fn a_zip_worker_declares_admitted_children_through_the_real_subprocess() {
    let input = fixture("01-plain-and-directory.zip");
    let identity = identity_for(&input);
    let outcome = run_worker(
        spawn(input, &identity, REAL_RUN_DEADLINE),
        &identity,
        &deny(),
    );
    let WorkerOutcome::Accepted(batch) = outcome else {
        panic!("a well-formed archive with no hostile entries must be accepted: {outcome:?}");
    };
    assert_eq!(batch.extractor, ZIP_EXTRACTOR);
    assert!(
        batch.units.is_empty(),
        "a container's own body carries no text units of its own: {:?}",
        batch.units
    );
    let mut paths: Vec<&str> = batch
        .declared_children
        .iter()
        .map(|c| c.relative_path.as_str())
        .collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        vec!["notes/a.md", "notes/b.txt", "readme.txt"],
        "the directory marker `notes/` must not appear as a declared child"
    );
    let a_md = batch
        .declared_children
        .iter()
        .find(|c| c.relative_path == "notes/a.md")
        .expect("notes/a.md declared");
    assert_eq!(
        a_md.name, "a.md",
        "DeclaredChild::name is the entry's own basename, relative_path the full path"
    );

    // 02-symlink.zip: the symlink is refused, `safe.txt` is admitted — a
    // batch this daemon-side `validate_batch` accepts wholesale, because the
    // path-safety/deny-set authority it runs is over the DECLARED children,
    // and the symlink was never declared at all.
    let symlink_input = fixture("02-symlink.zip");
    let symlink_identity = identity_for(&symlink_input);
    let symlink_outcome = run_worker(
        spawn(symlink_input, &symlink_identity, REAL_RUN_DEADLINE),
        &symlink_identity,
        &deny(),
    );
    let WorkerOutcome::Accepted(symlink_batch) = symlink_outcome else {
        panic!("a symlink entry refuses only itself, not the archive: {symlink_outcome:?}");
    };
    assert_eq!(symlink_batch.declared_children.len(), 1);
    assert_eq!(symlink_batch.declared_children[0].relative_path, "safe.txt");

    // 03-duplicate-name.zip: only the first occurrence is declared.
    let dup_input = fixture("03-duplicate-name.zip");
    let dup_identity = identity_for(&dup_input);
    let dup_outcome = run_worker(
        spawn(dup_input, &dup_identity, REAL_RUN_DEADLINE),
        &dup_identity,
        &deny(),
    );
    let WorkerOutcome::Accepted(dup_batch) = dup_outcome else {
        panic!("a duplicate name refuses only the later occurrence: {dup_outcome:?}");
    };
    assert_eq!(dup_batch.declared_children.len(), 1);
    assert_eq!(dup_batch.declared_children[0].relative_path, "dup.txt");
}

/// A well-formed nested archive (`06-nested-outer.zip`, depth 1) is declared
/// as ONE child (`inner.zip`) on today's wire — grandchildren stay internal
/// to `archive::expand`'s own return value (the named seam, `archive.rs`'s
/// own module doc: `DeclaredChild` carries no nested structure yet).
#[test]
fn a_nested_archive_entry_is_declared_once_not_recursed_onto_the_wire() {
    let input = fixture("06-nested-outer.zip");
    let identity = identity_for(&input);
    let outcome = run_worker(
        spawn(input, &identity, REAL_RUN_DEADLINE),
        &identity,
        &deny(),
    );
    let WorkerOutcome::Accepted(batch) = outcome else {
        panic!("a nested archive within the depth ceiling must still be accepted: {outcome:?}");
    };
    assert_eq!(batch.declared_children.len(), 1);
    assert_eq!(batch.declared_children[0].relative_path, "inner.zip");
    assert_eq!(batch.declared_children[0].name, "inner.zip");
}

// ------------------------------------------------- the archive-level refusal proof

/// **This wave's own acceptance** (the shape Y2's own hostile-document test
/// established): a WHOLE-archive refusal — here, the entry-count ceiling,
/// tripped by a well-formed archive that legitimately declares one more
/// entry than [`sergeant_rs::runtime::atlas::archive::MAX_ZIP_ENTRIES`]
/// allows — fails its own worker ALONE. The engine (standing in for the
/// daemon) stays up, the intelligence-lane permit is freed, no partial Atlas
/// rows appear, and the coverage row names the tripped bound.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_archive_level_refusal_fails_its_own_worker_alone() {
    use sergeant_rs::runtime::atlas::archive::MAX_ZIP_ENTRIES;

    let mut buffer = Vec::new();
    {
        use std::io::{Cursor, Write as _};
        let mut writer = zip::ZipWriter::new(Cursor::new(&mut buffer));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for i in 0..(MAX_ZIP_ENTRIES + 1) {
            writer
                .start_file(format!("f{i}.txt"), options)
                .expect("start_file");
            writer.write_all(b"").expect("write empty entry");
        }
        writer.finish().expect("finish archive");
    }

    let data = TempDir::new().expect("tempdir");
    let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path())
        .with_intelligence_lane_cap(1);
    let atlas = AtlasDb::open_in_memory().expect("in-memory atlas");
    assert!(atlas.indexed_sources().expect("read").is_empty());

    let identity = identity_for(&buffer);
    let outcome = run_worker_on_lane(
        &engine,
        spawn(buffer, &identity, REAL_RUN_DEADLINE),
        identity,
        deny(),
    )
    .await
    .expect("the lane call itself must not fail");

    let WorkerOutcome::Refused(row) = outcome else {
        panic!(
            "an archive over the entry-count ceiling must be refused, never accepted: {outcome:?}"
        );
    };
    assert_eq!(row.status, Coverage::Error, "{row:?}");
    let detail = row.detail.clone().unwrap_or_default();
    assert!(
        detail.contains("MAX_ZIP_ENTRIES"),
        "coverage detail must name the tripped bound: {detail:?}"
    );

    assert_eq!(
        engine.intelligence_lane.available_permits(),
        1,
        "the intelligence-lane permit must be freed"
    );
    let still_alive: usize = engine
        .run_intelligence(|| 7)
        .await
        .expect("the engine must still be usable");
    assert_eq!(still_alive, 7);
    assert!(
        atlas.indexed_sources().expect("read").is_empty(),
        "no partial rows may appear from a refused worker"
    );
}

/// No `sgt-atlas-worker` process may survive this suite.
#[test]
fn no_worker_process_survives_the_zip_adapter_walk() {
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

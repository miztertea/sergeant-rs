//! A4 (spec §7): every non-test `BlobStore::put`/`put_stream` call site has a
//! proof that `blob::refs_in_event` recovers its reference from a real
//! emitted event. Landed before any deletion code exists (W3), so a future
//! capture site cannot create an unmarkable blob.
//!
//! Three layers:
//!
//! 1. A structural source scan — every non-test `.put(`/`.put_stream(` call
//!    site equals a hardcoded table (always runs, no Docker, no daemon).
//! 2. Each site's reference is really recoverable, from a real event: the
//!    claude arm drives the real adapter over a stub binary; the docker arm
//!    has a fixture case (always runs) and a live case (`require_docker!()`).
//! 3. A round trip over a whole journal: every blob file on disk is
//!    recovered by `refs_in_event` from at least one event.

mod support;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::api::Core;
use sergeant_rs::backend::Backend;
use sergeant_rs::backend::claude::{ClaudeBackend, ClaudeConfig};
use sergeant_rs::daemon::journaling_sink;
use sergeant_rs::domain::event::{Event, EventDraft, EventSource};
use sergeant_rs::runtime::blob::{BlobRef, refs_in_event, refs_in_payload};
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::projection::work_registry_projection;

// ------------------------------------------------------------------
// Layer 1: the enumeration is complete
// ------------------------------------------------------------------

/// Every non-test `BlobStore::put`/`put_stream` call site, and the journal
/// event kind(s) each one's reference reaches the journal through.
///
/// A new site fails this assertion. Adding a row is not enough — every row
/// must also have a proof in Layer 2 that `blob::refs_in_event` recovers its
/// reference from a real emitted event. That is the whole point of A4: a
/// capture site that produces a blob nobody can mark is a blob a future
/// prune sweep would delete while it is still referenced.
struct PutSite {
    file: &'static str,
    /// Substring identifying the call site's enclosing function/impl block,
    /// so a rename of the call itself (not the site) does not silently
    /// widen this scan.
    context: &'static str,
}

const PUT_SITES: &[PutSite] = &[
    PutSite {
        file: "src/backend/claude.rs",
        context: "impl TurnReader",
    },
    PutSite {
        file: "src/backend/docker.rs",
        context: "fn stream_one",
    },
];

/// Every `.put(`/`.put_stream(` call in `file`'s non-test code, as
/// `(line, enclosing context snippet)`.
fn non_test_put_calls(path: &Path) -> Vec<(usize, String)> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let mut in_test_mod = false;
    let mut depth_at_test_mod_start: i32 = -1;
    let mut depth = 0i32;
    let mut hits = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[cfg(test)]") {
            // The *next* `mod` this introduces is test-only; track its brace
            // depth so everything nested under it is excluded, mirroring
            // `tests/m9_watch.rs`'s own structural-scan shape for exactly
            // this "skip the test module" problem.
            in_test_mod = true;
        }
        if in_test_mod && trimmed.starts_with("mod ") && depth_at_test_mod_start < 0 {
            depth_at_test_mod_start = depth;
        }
        if (trimmed.contains(".put(") || trimmed.contains(".put_stream("))
            && !(in_test_mod && depth_at_test_mod_start >= 0 && depth > depth_at_test_mod_start)
        {
            hits.push((i + 1, line.trim().to_string()));
        }
        depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        if in_test_mod
            && depth_at_test_mod_start >= 0
            && depth <= depth_at_test_mod_start
            && trimmed == "}"
        {
            in_test_mod = false;
            depth_at_test_mod_start = -1;
        }
    }
    hits
}

#[test]
fn every_non_test_put_site_is_in_the_table() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for site in PUT_SITES {
        let path = root.join(site.file);
        let hits = non_test_put_calls(&path);
        assert!(
            !hits.is_empty(),
            "{}: expected at least one non-test .put(/.put_stream( call, found none — \
             PUT_SITES is stale",
            site.file
        );
        let source = std::fs::read_to_string(&path).expect("read");
        assert!(
            source.contains(site.context),
            "{}: expected to find {:?} in the file (PUT_SITES's context anchor)",
            site.file,
            site.context
        );
    }

    // The other direction: no non-test .put(/.put_stream( call exists in a
    // source file that is not one of PUT_SITES's two files (a new capture
    // site must be added here deliberately, not discovered by omission).
    let src = root.join("src");
    for file in rust_sources(&src) {
        let relative = file
            .strip_prefix(&root)
            .expect("under root")
            .to_string_lossy()
            .replace('\\', "/");
        if PUT_SITES.iter().any(|s| s.file == relative) {
            continue;
        }
        let hits = non_test_put_calls(&file);
        assert!(
            hits.is_empty(),
            "{relative} has a non-test .put(/.put_stream( call PUT_SITES does not name: {hits:?}"
        );
    }
}

fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read_dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            out.extend(rust_sources(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}

// ------------------------------------------------------------------
// Layer 2, claude arm: a real recorded turn, through the real adapter
// ------------------------------------------------------------------

const RECORDED_TURN: &str = include_str!("fixtures/claude-2.1.226-turn.jsonl");

/// A minimal stub `claude`: answers the capability probe, then replays the
/// recorded transcript on every turn invocation. Deliberately smaller than
/// `m4_backends.rs`'s own `StubClaude` — this suite needs exactly one
/// recorded turn through the real adapter, not the full launch-grammar
/// contract surface.
fn write_claude_stub(dir: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("claude-stub-a4");
    let replay = dir.join("claude-replay-a4.jsonl");
    std::fs::write(&replay, RECORDED_TURN).expect("write replay fixture");
    let script = format!(
        "#!/bin/sh\ncase \"$1\" in\n  \
           --version) echo \"2.1.226 (Claude Code)\";;\n  \
           --help) echo \"--print --verbose --output-format --session-id --resume \
                          --setting-sources --model --permission-mode\";;\n  \
           *) cat \"{}\";;\n\
         esac\n",
        replay.display()
    );
    std::fs::write(&path, script).expect("write stub");
    let mut perm = std::fs::metadata(&path).expect("stat").permissions();
    perm.set_mode(0o755);
    std::fs::set_permissions(&path, perm).expect("chmod");
    support::wait_until_executable(&path);
    path
}

fn core(data_dir: &Path) -> Core {
    let journal = Journal::open(data_dir).expect("journal");
    let mut registry = work_registry_projection();
    registry
        .catch_up(journal.replay().expect("replay"))
        .expect("catch up");
    let (events_tx, _rx) = tokio::sync::broadcast::channel(16);
    Core::new(journal, registry, events_tx)
}

/// Drive one recorded turn through the real `ClaudeBackend`, over the
/// stub above, into a real journal + blob store under `data_dir`. Returns
/// the journaled `conversation.turn.ended` and `usage.updated` events.
fn drive_one_recorded_claude_turn(data_dir: &Path) -> (Event, Event) {
    let stub_path = write_claude_stub(data_dir);
    let mut config = ClaudeConfig::new(data_dir);
    config.executable = stub_path;
    let backend = ClaudeBackend::new(config);
    let shared = Arc::new(tokio::sync::Mutex::new(core(data_dir)));
    backend.set_event_sink(journaling_sink(shared.clone()));

    let handle = backend
        .start(&sergeant_rs::backend::StartRequest {
            work_id: "work-a4".to_string(),
            execution_id: "e-a4".to_string(),
            stage_id: "00-only".to_string(),
            attempt: 1,
            cwd: data_dir.to_path_buf(),
            intent: "a4 fixture replay".to_string(),
            context: String::new(),
            model: Some("haiku".to_string()),
            profile: None,
            execute: None,
            instruction_policy: sergeant_rs::domain::estate::InstructionPolicy::default(),
            bindings: Vec::new(),
        })
        .expect("start");

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let observation = backend.observe(&handle).expect("observe");
        if observation.native != sergeant_rs::backend::NativeState::Running {
            break;
        }
        assert!(Instant::now() < deadline, "turn did not settle");
        std::thread::sleep(Duration::from_millis(100));
    }

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let core = shared.blocking_lock();
        let events: Vec<Event> = core
            .journal
            .replay()
            .expect("replay")
            .collect::<Result<_, _>>()
            .expect("events");
        let ended = events
            .iter()
            .find(|e| e.kind == "conversation.turn.ended")
            .cloned();
        let usage = events.iter().find(|e| e.kind == "usage.updated").cloned();
        if let (Some(ended), Some(usage)) = (ended, usage) {
            return (ended, usage);
        }
        drop(core);
        assert!(
            Instant::now() < deadline,
            "turn.ended/usage.updated never journaled"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn claude_arm_refs_in_event_recovers_the_archived_raw_ref() {
    let data = TempDir::new().expect("tempdir");
    let (ended, usage) = drive_one_recorded_claude_turn(data.path());

    let ended_refs = refs_in_event(&ended);
    assert_eq!(
        ended_refs.len(),
        1,
        "expected exactly one ref in conversation.turn.ended"
    );
    let the_ref = ended_refs.iter().next().unwrap();

    let path = data.path().join("blobs").join("b3").join(the_ref.hex());
    assert!(
        path.exists(),
        "the ref's hex must name a real blob file: {path:?}"
    );
    let bytes = std::fs::read(&path).expect("read blob");
    assert_eq!(
        blake3::hash(&bytes).to_hex().to_string(),
        the_ref.hex(),
        "the store's own contract, re-proved from the extractor's side"
    );

    let usage_refs = refs_in_event(&usage);
    assert_eq!(
        usage_refs.len(),
        1,
        "usage.updated must carry the same archived-raw ref as conversation.turn.ended"
    );
    assert_eq!(
        usage_refs, ended_refs,
        "refs_in_event must recover the same ref from both events"
    );
}

// ------------------------------------------------------------------
// Layer 2, docker arm
// ------------------------------------------------------------------

const DOCKER_FIXTURE: &str = include_str!("fixtures/docker-stage-completed.payload.json");

#[test]
fn docker_fixture_arm_nested_walk_recovers_both_refs_flat_scan_recovers_none() {
    let payload: Value = serde_json::from_str(DOCKER_FIXTURE).expect("parse fixture");

    let mut out = BTreeSet::new();
    refs_in_payload(&payload, &mut out);
    assert_eq!(out.len(), 2, "expected stdout + stderr refs, got {out:?}");

    // The load-bearing negative half: a *flat* top-level scan (only string
    // fields directly on the payload object, not recursing into `detail`'s
    // stringified content) recovers zero — A4's claim, proved structurally.
    let mut flat = BTreeSet::new();
    if let Value::Object(map) = &payload {
        for v in map.values() {
            if let Value::String(s) = v
                && let Ok(r) = BlobRef::from_str(s)
            {
                flat.insert(r);
            }
        }
    }
    assert!(
        flat.is_empty(),
        "a flat top-level scan must recover nothing from this docker-shaped payload \
         (found {flat:?}) — otherwise this fixture does not demonstrate A4's claim"
    );

    // refs_in_event over a real Event wrapping this payload.
    let event = EventDraft::new(
        EventSource::new("backend", "docker"),
        "stage.completed",
        payload,
    )
    .with_work_id("w1")
    .into_event(1);
    assert_eq!(refs_in_event(&event), out);
}

fn docker_unavailable() -> Option<&'static str> {
    match Command::new("docker").arg("version").output() {
        Ok(out) if out.status.success() => None,
        Ok(_) => Some("SKIPPED-ENV: `docker version` exited nonzero on this host"),
        Err(_) => Some("SKIPPED-ENV: no `docker` binary reachable on this host"),
    }
}

macro_rules! require_docker {
    () => {
        if let Some(reason) = docker_unavailable() {
            eprintln!("{reason}");
            return;
        }
    };
}

/// Live arm: drive a real docker container to exit, capture its
/// `stage.completed`-shaped `summary`, and cross-check its shape against
/// the fixture above — the check that keeps the fixture from rotting
/// silently as the docker path evolves.
#[test]
fn docker_live_arm_recovers_both_refs_and_matches_the_fixtures_shape() {
    require_docker!();
    use sergeant_rs::backend::docker::{DockerBackend, DockerConfig};
    use sergeant_rs::domain::workflow::{ExecuteSpec, NetworkPolicy, WorkspaceAccess};

    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = DockerBackend::new(DockerConfig::new(data.path())).expect("backend");
    let execution_id = format!("a4live-{}", ulid::Ulid::generate());
    let name = format!("sgt-{execution_id}");

    let req = sergeant_rs::backend::StartRequest {
        work_id: "w1".to_string(),
        execution_id: execution_id.clone(),
        stage_id: "00-only".to_string(),
        attempt: 1,
        cwd: cwd.path().to_path_buf(),
        intent: "a4 live docker capture".to_string(),
        context: String::new(),
        model: None,
        profile: None,
        execute: Some(ExecuteSpec {
            image: "alpine:3.24".to_string(),
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "echo a4-live-stdout; echo a4-live-stderr 1>&2; exit 0".to_string(),
            ],
            workdir: "/estate".to_string(),
            workspace_access: WorkspaceAccess::ReadOnly,
            network: NetworkPolicy::None,
            env: Default::default(),
        }),
        instruction_policy: sergeant_rs::domain::estate::InstructionPolicy::default(),
        bindings: Vec::new(),
    };
    let prepared = backend.prepare(&req).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    let deadline = Instant::now() + Duration::from_secs(30);
    let observation = loop {
        let observation = backend.observe(&handle).expect("observe");
        if observation.native == sergeant_rs::backend::NativeState::Exited {
            break observation;
        }
        assert!(Instant::now() < deadline, "container did not exit in time");
        std::thread::sleep(Duration::from_millis(150));
    };
    let _ = Command::new("docker").args(["rm", "-f", &name]).output();

    let sergeant_rs::backend::BackendSignal::StageCompleted {
        summary: Some(detail),
    } = observation.signal
    else {
        panic!("expected a completed stage with captured evidence: {observation:?}");
    };

    let payload = json!({"stage_id": "00-only", "index": 0, "detail": detail});
    let event = EventDraft::new(
        EventSource::new("backend", "docker"),
        "stage.completed",
        payload,
    )
    .with_work_id("w1")
    .into_event(1);
    let live_refs = refs_in_event(&event);
    assert_eq!(
        live_refs.len(),
        2,
        "expected stdout + stderr refs from a live capture"
    );

    // Shape cross-check against the fixture: same top-level key set, and
    // `detail` is a String that parses to an object carrying stdout/stderr.
    let fixture: Value = serde_json::from_str(DOCKER_FIXTURE).expect("parse fixture");
    let live_keys: BTreeSet<&str> = event
        .payload
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    let fixture_keys: BTreeSet<&str> = fixture
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(
        live_keys, fixture_keys,
        "the live capture's top-level shape must match the fixture"
    );
    let live_detail: Value =
        serde_json::from_str(event.payload["detail"].as_str().unwrap()).expect("detail parses");
    assert!(live_detail.get("stdout").is_some() && live_detail.get("stderr").is_some());
}

// ------------------------------------------------------------------
// Layer 3: round trip over a whole journal — every blob is referenced
// ------------------------------------------------------------------

#[test]
fn every_blob_written_by_a_real_run_is_recovered_by_refs_in_event() {
    let data = TempDir::new().expect("tempdir");
    // Two claude turns through the same small rig, so more than one blob
    // exists to round-trip against.
    let (ended1, _usage1) = drive_one_recorded_claude_turn(data.path());
    let _ = ended1;

    let blobs_dir = data.path().join("blobs").join("b3");
    let mut on_disk = BTreeSet::new();
    if blobs_dir.is_dir() {
        for entry in std::fs::read_dir(&blobs_dir).expect("read_dir") {
            let entry = entry.expect("entry");
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Ok(r) = BlobRef::from_str(&format!("b3:{name}")) {
                on_disk.insert(r);
            }
        }
    }
    assert!(
        !on_disk.is_empty(),
        "the rig must have written at least one blob"
    );

    let events: Vec<Event> = Journal::replay_data_dir(data.path())
        .expect("replay")
        .collect::<Result<_, _>>()
        .expect("events");
    let mut referenced = BTreeSet::new();
    for event in &events {
        referenced.extend(refs_in_event(event));
    }

    let unreferenced: Vec<_> = on_disk.difference(&referenced).collect();
    assert!(
        unreferenced.is_empty(),
        "every blob on disk must be recovered by refs_in_event from at least one event; \
         unreferenced: {unreferenced:?}"
    );
}

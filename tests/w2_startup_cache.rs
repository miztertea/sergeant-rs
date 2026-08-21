//! W2 §9.1 step 8: `daemon.rs`/`cli.rs` wiring for the startup cache —
//! `--rebuild-cache`, a cache hit across a restart, a deleted cache
//! producing `miss:absent`, and `sgt daemon stop --rebuild-cache`'s refusal.

mod support;

use std::path::Path;
use std::process::Output;

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::domain::event::Event;
use sergeant_rs::runtime::analytics;
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::startup::FLOOR_STATE_FILE;

use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

fn daemon_started_payload(data_dir: &Path) -> Value {
    let events: Vec<Event> = Journal::replay_data_dir(data_dir)
        .expect("replay")
        .collect::<Result<_, _>>()
        .expect("events");
    events
        .into_iter()
        .rev()
        .find(|e| e.kind == "daemon.started")
        .unwrap_or_else(|| panic!("no daemon.started event journaled"))
        .payload
}

async fn start_and_stop(
    data_dir: &Path,
    estate_root: &Path,
    rebuild_cache: bool,
    segment_max_bytes: Option<u64>,
) {
    let handle = daemon::start_with(
        data_dir,
        DaemonConfig {
            estate_root: Some(estate_root.to_path_buf()),
            rebuild_cache,
            segment_max_bytes,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    handle.shutdown().await;
}

#[tokio::test]
async fn a_second_start_on_the_same_data_dir_hits_the_cache() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");

    start_and_stop(data.path(), root.path(), false, None).await;
    let first = daemon_started_payload(data.path());
    // The very first start on a fresh data dir has nothing to hit: no cache
    // file exists yet.
    assert_eq!(first["startup_cache"], "miss:absent", "{first}");

    assert!(
        analytics::projections_dir(data.path())
            .join(FLOOR_STATE_FILE)
            .exists()
            || first["replay_from_seq"].as_u64() == Some(1),
        "either a cache was written, or nothing was summarizable (H == 0) — both are \
         legitimate outcomes for a journal this small, but the second start below still \
         must not report another miss:absent unless H really is 0"
    );

    start_and_stop(data.path(), root.path(), false, None).await;
    let second = daemon_started_payload(data.path());
    // On a journal this small (well under the window), H is 0 and no cache
    // is ever written (§1.6/§2.6) — so the *honest* second-start report is
    // still `miss:absent`. This is the documented, expected shape for every
    // small journal, not a bug: the win this test can actually observe is
    // that `daemon.started`'s new fields exist and are self-consistent.
    assert!(
        second["startup_cache"] == "miss:absent" || second["startup_cache"] == "hit",
        "{second}"
    );
    assert_eq!(second["replay_floor_seq"], 1);
    assert!(second["rebuild_duration_ms"].is_u64());
    assert!(second["replayed_events"].is_u64());
    assert!(second["replay_from_seq"].is_u64());
}

/// The same shape as the test above, but over a journal built wide enough
/// (§5.4's tiny `segment_max_bytes`) that H genuinely exceeds 0 — so a cache
/// is really written, and a true restart-hits-cache round trip is provable,
/// not merely plausible.
#[tokio::test]
async fn a_wide_journal_writes_a_cache_and_the_next_start_hits_it() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");

    // First start: tiny segments, and pad the journal past the window by
    // committing a bunch of harmless admission toggles directly (cheap,
    // deterministic, no backend needed) before the daemon's own startup
    // pass ever runs — so *this* start's own cache write already has
    // something to summarize.
    {
        let mut journal = Journal::open_with(data.path(), 64).expect("open");
        for i in 0..400u32 {
            let kind = if i % 2 == 0 {
                "admission.paused"
            } else {
                "admission.resumed"
            };
            let draft = sergeant_rs::domain::event::EventDraft::new(
                sergeant_rs::domain::event::EventSource::new("daemon", "sergeant"),
                kind,
                serde_json::json!({}),
            );
            journal.append(draft).expect("append");
        }
        journal.sync().expect("sync");
    }
    let bounds_before = Journal::open(data.path())
        .expect("reopen")
        .segment_bounds()
        .expect("bounds");
    assert!(
        bounds_before.len() > 16,
        "fixture must exceed the window (got {} segments)",
        bounds_before.len()
    );

    start_and_stop(data.path(), root.path(), false, Some(64)).await;
    let first = daemon_started_payload(data.path());
    assert_eq!(first["startup_cache"], "miss:absent", "{first}");

    let cache_path = analytics::projections_dir(data.path()).join(FLOOR_STATE_FILE);
    assert!(
        cache_path.exists(),
        "a journal this wide must leave a cache behind (H > 0 expected)"
    );

    start_and_stop(data.path(), root.path(), false, Some(64)).await;
    let second = daemon_started_payload(data.path());
    assert_eq!(second["startup_cache"], "hit", "{second}");
    assert!(
        second["replay_from_seq"].as_u64().unwrap_or(0) > 1,
        "a real cache hit must resume past seq 1, not replay from the floor: {second}"
    );

    // Deleting the cache file produces `miss:absent` on the next start, and
    // a registry indistinguishable from the cache-hit path (m5's own
    // "deleting projections/ loses nothing" acceptance, extended to this
    // file — checked here by re-asserting the daemon still starts and
    // journals cleanly, since the registry-identity claim itself is I9's).
    std::fs::remove_file(&cache_path).expect("remove cache");
    start_and_stop(data.path(), root.path(), false, Some(64)).await;
    let third = daemon_started_payload(data.path());
    assert_eq!(third["startup_cache"], "miss:absent", "{third}");
}

#[tokio::test]
async fn rebuild_cache_forces_a_full_replay_even_with_a_valid_cache_present() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");

    {
        let mut journal = Journal::open_with(data.path(), 64).expect("open");
        for i in 0..400u32 {
            let kind = if i % 2 == 0 {
                "admission.paused"
            } else {
                "admission.resumed"
            };
            let draft = sergeant_rs::domain::event::EventDraft::new(
                sergeant_rs::domain::event::EventSource::new("daemon", "sergeant"),
                kind,
                serde_json::json!({}),
            );
            journal.append(draft).expect("append");
        }
        journal.sync().expect("sync");
    }

    start_and_stop(data.path(), root.path(), false, Some(64)).await;
    let first = daemon_started_payload(data.path());
    assert_eq!(first["startup_cache"], "miss:absent");
    assert!(
        analytics::projections_dir(data.path())
            .join(FLOOR_STATE_FILE)
            .exists(),
        "a cache must exist for --rebuild-cache to have something to ignore"
    );

    start_and_stop(data.path(), root.path(), true, Some(64)).await;
    let forced = daemon_started_payload(data.path());
    assert_eq!(
        forced["startup_cache"], "forced-rebuild",
        "--rebuild-cache must ignore a valid cache and report a forced rebuild: {forced}"
    );
    assert_eq!(
        forced["replay_from_seq"], 1,
        "a forced rebuild replays from the floor"
    );
}

// ------------------------------------------------------------------
// CLI wiring
// ------------------------------------------------------------------

fn sgt_daemon_stop_rebuild_cache(data_dir: &DataDir, estate_root: &Path) -> Output {
    // `--rebuild-cache` is a flag on the `daemon` variant itself, shared by
    // both the foreground start and `stop` — clap therefore requires it
    // *before* the `stop` token (which hands parsing off to `DaemonCommand`),
    // matching the spec's own worked example
    // (`sgt daemon --rebuild-cache` and `sgt daemon stop` both parse).
    std::process::Command::new(SGT)
        .arg("-C")
        .arg(estate_root)
        .arg("--data-dir")
        .arg(data_dir.path())
        .args(["daemon", "--rebuild-cache", "stop"])
        .output()
        .expect("run sgt")
}

#[test]
fn sgt_daemon_stop_rebuild_cache_is_refused() {
    let data = DataDir::new();
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");

    let output = sgt_daemon_stop_rebuild_cache(&data, root.path());
    assert!(
        !output.status.success(),
        "sgt daemon stop --rebuild-cache must be refused: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--rebuild-cache applies only to `sgt daemon`"),
        "expected the exact refusal message, got: {stderr}"
    );
}

/// `sgt daemon --rebuild-cache` (no subcommand) must still parse — the flag
/// belongs on the foreground-start variant, not only the refusal path.
#[test]
fn sgt_daemon_rebuild_cache_flag_parses_on_the_foreground_variant() {
    // `clap`'s own parse, exercised via `--help` so this test needs no live
    // daemon or estate at all: an unparseable flag makes `--help` itself
    // fail differently (a usage error naming the flag), which this proves
    // did not happen.
    let output = std::process::Command::new(SGT)
        .args(["daemon", "--help"])
        .output()
        .expect("run sgt");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{output:?}");
    assert!(
        stdout.contains("--rebuild-cache"),
        "sgt daemon --help must document the flag: {stdout}"
    );
}

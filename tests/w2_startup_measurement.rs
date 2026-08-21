//! W2 §9.4: measured before/after startup cost on the dogfood journal.
//! **Read-only, `#[ignore]`d — run explicitly, never in CI.**
//!
//! Source: `$SGT_DOGFOOD_DATA_DIR`, defaulting to
//! `~/sergeant-rs-workspace/.sergeant/data`. If it does not exist, prints a
//! loud named skip and returns — never fails. Every test here proves its own
//! read-only claim by walking the whole data dir (path, len, mtime) before
//! and after and asserting the two walks are identical; the only calls made
//! against the dogfood dir itself are [`Journal::replay_data_dir`]/
//! [`Journal::replay_data_dir_from_floor`] (no lock, no tail recovery, no
//! segment creation) and a streamed BLAKE3 over segment bytes. Analytics is
//! always pointed at a separate scratch `TempDir`.
//!
//! **Honesty note, reproduced from the wave spec:** the live dogfood journal
//! is measured to be small (well inside the 16-segment / 128 MiB window as
//! of this wave), so this measurement demonstrates rung 0's win (four walks
//! collapsed to one) on real data, and demonstrates *nothing* about the
//! window itself — on an estate this size the window is the whole journal.
//! The window's effect is proven by the constructed multi-segment tests in
//! `runtime::startup`'s own unit tests and in `i9_floor_pinning.rs`; the
//! constant is Q4's, argued from evidence in the rulings record, not from
//! this measurement.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use tempfile::TempDir;

use sergeant_rs::backend::claude::{AskWithdrawal, note_ask_withdrawal};
use sergeant_rs::runtime::analytics::Analytics;
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::projection::work_registry_projection;

fn dogfood_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("SGT_DOGFOOD_DATA_DIR") {
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("sergeant-rs-workspace/.sergeant/data"))
}

fn require_dogfood() -> Option<PathBuf> {
    match dogfood_dir() {
        Some(dir) if dir.is_dir() => Some(dir),
        Some(dir) => {
            eprintln!(
                "SKIPPED: dogfood data dir {dir:?} does not exist — this measurement needs a \
                 real estate's journal and is never a hard requirement"
            );
            None
        }
        None => {
            eprintln!("SKIPPED: could not resolve a dogfood data dir (no $HOME, no override)");
            None
        }
    }
}

/// `(path, len, mtime)` for every entry under `dir`, recursively — the
/// before/after fingerprint that proves a measurement touched nothing.
fn fingerprint(dir: &Path) -> BTreeMap<PathBuf, (u64, std::time::SystemTime)> {
    fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, (u64, std::time::SystemTime)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() {
                walk(&path, out);
            } else {
                let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                out.insert(path, (meta.len(), mtime));
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, &mut out);
    out
}

fn assert_untouched<T>(
    dir: &Path,
    before: &BTreeMap<PathBuf, (u64, std::time::SystemTime)>,
    _proof: T,
) {
    let after = fingerprint(dir);
    assert_eq!(
        before, &after,
        "this measurement must be read-only over the dogfood data dir"
    );
}

/// Every segment file under `<data_dir>/journal/`, oldest first.
fn segment_files(data_dir: &Path) -> Vec<PathBuf> {
    let journal_dir = data_dir.join("journal");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&journal_dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "ndjson"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

fn blake3_of(path: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// The baseline arm: today's four separate full walks, timed individually
/// and summed.
#[test]
#[ignore]
fn baseline_four_walks_on_the_dogfood_journal() {
    let Some(data_dir) = require_dogfood() else {
        return;
    };
    let before = fingerprint(&data_dir);

    let t0 = Instant::now();
    // Walk 1: next_seq (approximated here by counting every event, since
    // `Journal::open`'s own tail-only scan is a private-module detail this
    // external test cannot call directly without taking the exclusive lock
    // this suite is required to never take).
    let mut next_seq = 1u64;
    for event in Journal::replay_data_dir(&data_dir).expect("replay") {
        next_seq = event.expect("event").seq + 1;
    }
    let walk1 = t0.elapsed();

    let t1 = Instant::now();
    let mut registry = work_registry_projection();
    registry
        .catch_up(Journal::replay_data_dir(&data_dir).expect("replay"))
        .expect("catch up");
    let walk2 = t1.elapsed();

    let scratch = TempDir::new().expect("scratch");
    let t2 = Instant::now();
    let _analytics = Analytics::rebuild(
        scratch.path(),
        Journal::replay_data_dir(&data_dir).expect("replay"),
    )
    .expect("analytics rebuild");
    let walk3 = t2.elapsed();

    let t3 = Instant::now();
    let mut latest: Option<AskWithdrawal> = None;
    for event in Journal::replay_data_dir(&data_dir).expect("replay") {
        note_ask_withdrawal(&mut latest, &event.expect("event"));
    }
    let walk4 = t3.elapsed();

    let total = walk1 + walk2 + walk3 + walk4;
    println!(
        "baseline: next_seq={walk1:?} registry={walk2:?} analytics={walk3:?} \
         capability={walk4:?} total={total:?} (next_seq landed on {next_seq})"
    );

    assert_untouched(&data_dir, &before, ());
}

/// The after arm: one shared pass over `replay_data_dir_from_floor` — the
/// windowed primitive on a journal this small is identical to the
/// unwindowed one (A1), so this measures rung 0's win (four walks
/// collapsed to one) without the window itself doing anything on this
/// estate's real size.
#[test]
#[ignore]
fn after_one_shared_pass_on_the_dogfood_journal() {
    let Some(data_dir) = require_dogfood() else {
        return;
    };
    let before = fingerprint(&data_dir);

    // Tail scan proxy: the real `tail_next_seq` is a private, journal-module
    // detail; the same O(one segment) shape is exercised here directly by
    // scanning just the newest segment file.
    let t_tail = Instant::now();
    if let Some(last_segment) = segment_files(&data_dir).last() {
        let bytes = std::fs::read(last_segment).expect("read last segment");
        let _lines = bytes.iter().filter(|&&b| b == b'\n').count();
    }
    let tail = t_tail.elapsed();

    let scratch = TempDir::new().expect("scratch");
    let t0 = Instant::now();
    let mut registry = work_registry_projection();
    let mut analytics = Analytics::begin_rebuild(scratch.path()).expect("analytics scratch");
    let mut latest: Option<AskWithdrawal> = None;
    {
        let mut fold = analytics.fold().expect("fold");
        for event in Journal::replay_data_dir_from_floor(&data_dir).expect("replay") {
            let event = event.expect("event");
            registry.apply(&event).expect("apply");
            fold.push(&event).expect("push");
            note_ask_withdrawal(&mut latest, &event);
        }
        fold.finish().expect("finish");
    }
    let pass = t0.elapsed();
    let total = tail + pass;
    println!("after: tail_scan_proxy={tail:?} shared_pass={pass:?} total={total:?}");

    assert_untouched(&data_dir, &before, ());
}

/// The cache arm: what a cache-verify (`metadata` + streamed BLAKE3 per
/// summarized segment) costs, reported per MiB — the number that
/// generalizes past this estate's own small size, per the honesty note
/// above.
#[test]
#[ignore]
fn cache_verify_cost_per_mebibyte_on_the_dogfood_journal() {
    let Some(data_dir) = require_dogfood() else {
        return;
    };
    let before = fingerprint(&data_dir);

    let files = segment_files(&data_dir);
    let t0 = Instant::now();
    let mut total_bytes = 0u64;
    for file in &files {
        let len = std::fs::metadata(file).expect("metadata").len();
        let _hash = blake3_of(file).expect("hash");
        total_bytes += len;
    }
    let elapsed = t0.elapsed();
    let mib = (total_bytes as f64) / (1024.0 * 1024.0);
    let per_mib = if mib > 0.0 {
        elapsed.as_secs_f64() / mib
    } else {
        0.0
    };
    let segment_count = files.len();
    println!(
        "cache-verify: {segment_count} segments, {mib:.3} MiB, {elapsed:?} total, \
         {per_mib:.6} s/MiB"
    );

    assert_untouched(&data_dir, &before, ());
}

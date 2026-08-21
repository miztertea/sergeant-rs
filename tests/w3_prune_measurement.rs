//! W3 §8 Validation Evidence item 6: a measured, read-only run of the
//! prune engine's cheap phase (`candidate_horizon`/`stall_report`) against
//! the live dogfood journal. **Read-only, `#[ignore]`d — run explicitly,
//! never in CI.**
//!
//! Mirrors `tests/w2_startup_measurement.rs`'s own shape exactly: source,
//! skip behavior, and the before/after fingerprint that proves the
//! measurement touched nothing. `prune::run` is never called here — this
//! file only ever reads segment files (never writes or renames one) and
//! calls the in-memory, no-further-I/O phase A predicate
//! (`candidate_horizon`) and the reporting wrapper around it
//! (`stall_report`), both of which take a `WorkRegistry` and segment bounds
//! already in hand.
//!
//! **Never `Journal::open`.** That primitive takes an exclusive advisory
//! lock and can mutate a torn tail on recovery — wrong for a read-only
//! measurement against a data dir a real daemon may still own, exactly per
//! `tests/w2_startup_measurement.rs`'s own rule. Segment bounds here are
//! reconstructed the same lock-free way `sgt doctor`'s journal check
//! already reads a live journal: list segment files, read each one's own
//! first line for its first seq, and take the registry replay's own
//! high-water mark (`Projection::last_seq`) as the newest segment's last
//! seq — the same arithmetic `Journal::segment_bounds` uses, without ever
//! opening a writer handle.
//!
//! **Honesty note, reproduced from `tests/w2_startup_measurement.rs`'s own
//! one:** the live dogfood journal is measured to be small, so this
//! measurement demonstrates phase A's cost shape (a handful of sorts over
//! `work_index`, no journal I/O) on real data, and demonstrates nothing
//! about how that cost scales on an estate with hundreds of thousands of
//! retained Works — the batch cap (`PRUNE_MAX_WORKS_PER_CYCLE`) and the
//! O(retained journal) mark scan's own cost are argued from the rulings
//! record, not from this measurement.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use sergeant_rs::domain::event::Event;
use sergeant_rs::runtime::journal::{Journal, SegmentBound};
use sergeant_rs::runtime::projection::work_registry_projection;
use sergeant_rs::runtime::prune::{
    FirstSeqIndex, PolicySource, PrunePolicy, candidate_horizon, stall_report,
};
use sergeant_rs::runtime::startup::{self, HorizonSink, RegistrySink};

/// Every `*.ndjson` segment file directly under `<data_dir>/journal/`,
/// oldest first by its 8-digit filename stem.
fn segment_files(data_dir: &Path) -> Vec<(u64, PathBuf)> {
    let journal_dir = data_dir.join("journal");
    let mut files: Vec<(u64, PathBuf)> = std::fs::read_dir(&journal_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| {
                    let path = e.path();
                    if path.extension().is_some_and(|ext| ext == "ndjson") {
                        let index: u64 = path.file_stem()?.to_str()?.parse().ok()?;
                        Some((index, path))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort_by_key(|(index, _)| *index);
    files
}

/// Lock-free segment bounds: `first_seq` from each segment's own first
/// line, `bytes` from `metadata`, `last_seq` as the next segment's
/// `first_seq - 1` (or, for the newest, `newest_last_seq` — the registry
/// replay's own high-water mark, since nothing here ever asks the writer).
/// A segment with no first line yet (rotated but never appended to) is
/// skipped, exactly as `Journal::segment_bounds` skips it.
fn read_only_segment_bounds(data_dir: &Path, newest_last_seq: u64) -> Vec<SegmentBound> {
    let files = segment_files(data_dir);
    let mut bounds: Vec<SegmentBound> = Vec::new();
    for (index, path) in files {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Some(first_line) = content.lines().next() else {
            continue; // an empty, just-rotated segment cannot bound anything
        };
        let Ok(event) = serde_json::from_str::<Event>(first_line) else {
            continue;
        };
        let Ok(bytes) = std::fs::metadata(&path).map(|m| m.len()) else {
            continue;
        };
        bounds.push(SegmentBound {
            index,
            path,
            first_seq: event.seq,
            last_seq: 0, // filled in below
            bytes,
        });
    }
    let len = bounds.len();
    for i in 0..len {
        bounds[i].last_seq = if i + 1 < len {
            bounds[i + 1].first_seq - 1
        } else {
            newest_last_seq
        };
    }
    bounds
}

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

fn assert_untouched(dir: &Path, before: &BTreeMap<PathBuf, (u64, std::time::SystemTime)>) {
    let after = fingerprint(dir);
    assert_eq!(
        before, &after,
        "this measurement must be read-only over the dogfood data dir"
    );
}

/// Phase A's own cost, on the live dogfood registry: one read-only replay
/// to rebuild `WorkRegistry`/`FirstSeqIndex`, then `candidate_horizon` and
/// `stall_report` timed on their own — both pure in-memory sorts/sweeps
/// over `work_index`, no journal I/O at all, which is the whole design
/// argument §7.1 makes for running phase A on every rotation tick.
#[test]
#[ignore]
fn candidate_horizon_cost_on_the_dogfood_registry() {
    let Some(data_dir) = require_dogfood() else {
        return;
    };
    let before = fingerprint(&data_dir);

    let t_replay = Instant::now();
    let mut registry = work_registry_projection();
    let mut horizon_sink = HorizonSink::default();
    startup::drive(
        Journal::replay_data_dir_from_floor(&data_dir).expect("replay_data_dir_from_floor"),
        &mut [&mut RegistrySink(&mut registry), &mut horizon_sink],
    )
    .expect("drive");
    let replay_cost = t_replay.elapsed();

    let bounds = read_only_segment_bounds(&data_dir, registry.last_seq());

    // A representative policy: the estate's own retention if the manifest
    // is readable from here (it is not, without a full `Estate` parse this
    // measurement has no business doing), so a fixed, documented stand-in.
    const MEASURED_RETENTION: u32 = 1000;
    let policy = PrunePolicy {
        retention: MEASURED_RETENTION,
        source: PolicySource::Config,
    };
    let first_seq: &FirstSeqIndex = horizon_sink.first_seq_by_work();

    let t0 = Instant::now();
    let (horizon, _) = candidate_horizon(&bounds, registry.state(), first_seq, &policy);
    let candidate_horizon_cost = t0.elapsed();

    let t1 = Instant::now();
    let stall = stall_report(
        &bounds,
        registry.state(),
        first_seq,
        &policy,
        std::time::SystemTime::now(),
    );
    let stall_report_cost = t1.elapsed();

    println!(
        "prune phase A on the dogfood registry: {} segments, {} retained Works, \
         retention={MEASURED_RETENTION} (measurement stand-in) -> horizon={horizon}, \
         retained_works_in_stall={}; replay={replay_cost:?} candidate_horizon={candidate_horizon_cost:?} \
         stall_report={stall_report_cost:?}",
        bounds.len(),
        registry.state().work_index.len(),
        stall.retained_works,
    );

    assert_untouched(&data_dir, &before);
}

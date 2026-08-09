//! M1 acceptance tests (docs/gauntlet/contracts/M1.md).
//!
//! 1. Round-trip + unknown-field preservation
//! 2. Crash-tail recovery
//! 3. Segment rotation
//! 4. Determinism (full replay twice; snapshot + tail replay)
//! 5. Seq integrity (gap/duplicate fails closed)
//! 6. Blob put/get, dedup, hash-mismatch fail-closed
//!
//! Plus invariant tests beyond the six: single-writer lock per journal dir,
//! blank journal lines fail closed, snapshot writes are atomic replaces.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::json;
use tempfile::TempDir;

use sergeant_rs::domain::event::{EVENT_SCHEMA, Event, EventDraft, EventSource};
use sergeant_rs::runtime::blob::{BlobError, BlobRef, BlobStore};
use sergeant_rs::runtime::journal::{Journal, JournalError};
use sergeant_rs::runtime::projection::{
    CoreStats, Projection, ProjectionError, core_stats_projection, core_stats_reducer,
};

fn draft(kind: &str, work_id: Option<&str>) -> EventDraft {
    let mut d = EventDraft::new(
        EventSource::new("daemon", "sergeant"),
        kind,
        json!({"n": 1}),
    );
    if let Some(w) = work_id {
        d = d.with_work_id(w);
    }
    d
}

fn segment_path(data_dir: &Path, index: u64) -> std::path::PathBuf {
    data_dir.join("journal").join(format!("{index:08}.ndjson"))
}

/// Acceptance test 1 — round-trip: the envelope serializes/deserializes
/// losslessly, and unknown fields in stored events survive being read into a
/// projection rebuild (read-modify-write of the projection), while the stored
/// journal line itself is never modified.
#[test]
fn round_trip_and_unknown_field_preservation() {
    // Plain round-trip is lossless.
    let event = draft("tool.completed", Some("work_1")).into_event(1);
    let json_text = serde_json::to_string(&event).expect("serialize");
    let back: Event = serde_json::from_str(&json_text).expect("deserialize");
    assert_eq!(back, event);
    assert_eq!(back.schema, EVENT_SCHEMA);

    // A stored event from a future version carries fields we don't know.
    let dir = TempDir::new().expect("tempdir");
    let mut journal = Journal::open(dir.path()).expect("open");
    let committed = journal
        .append(draft("work.created", Some("work_9")))
        .expect("append");

    let mut future_line = serde_json::to_value(&committed).expect("to_value");
    future_line["novelty"] = json!({"from": "sergeant.event/v9"});
    // Unknown fields nested inside envelope sub-objects must survive too.
    future_line["source"]["adapter_version"] = json!("3");
    let mut future_event: Event = serde_json::from_value(future_line.clone()).expect("parse");
    future_event.seq = 2;
    journal
        .append_event(&future_event)
        .expect("append future event");
    let stored_bytes = fs::read(segment_path(dir.path(), 1)).expect("read segment");
    drop(journal);

    // Read-modify-write of the projection: rebuild it from the journal,
    // snapshot it, and rebuild again. Unknown fields survive the read.
    let journal = Journal::open(dir.path()).expect("reopen");
    let events: Vec<Event> = journal
        .replay()
        .expect("replay")
        .collect::<Result<_, _>>()
        .expect("events");
    assert_eq!(
        events[1].extra["novelty"],
        json!({"from": "sergeant.event/v9"})
    );
    assert_eq!(events[1].source.extra["adapter_version"], json!("3"));
    // Re-serializing the read event reproduces the stored value exactly,
    // including the unknown fields at both nesting levels.
    let reserialized = serde_json::to_value(&events[1]).expect("to_value");
    let mut expected = future_line;
    expected["seq"] = json!(2);
    assert_eq!(reserialized, expected);

    // Read-modify-write of the projection: fold the journal, snapshot BEFORE
    // the unknown-field event, reload, and replay the tail — so the event
    // carrying unknown fields actually flows through the reducer of the
    // rebuilt projection (`applied == 1` proves it was not skipped).
    let mut projection = core_stats_projection();
    projection
        .catch_up(journal.replay().expect("replay"))
        .expect("fold");
    let mut partial = core_stats_projection();
    partial
        .catch_up(journal.replay().expect("replay").take(1))
        .expect("fold first event");
    let snap = dir.path().join("snapshots").join("core.json");
    partial.write_snapshot(&snap).expect("write snapshot");
    let mut reloaded =
        Projection::<CoreStats>::load_snapshot(&snap, core_stats_reducer).expect("load snapshot");
    let applied = reloaded
        .catch_up(journal.replay().expect("replay"))
        .expect("refold");
    assert_eq!(applied, 1, "the unknown-field event must be applied");
    assert_eq!(reloaded.state(), projection.state());

    // The stored lines are immutable: byte-identical after all of the above.
    assert_eq!(
        fs::read(segment_path(dir.path(), 1)).expect("read segment"),
        stored_bytes
    );
}

/// Acceptance test 2 — crash tail: a journal whose last line is truncated
/// mid-JSON opens cleanly, recovers every complete event, and the next
/// append continues the seq.
#[test]
fn crash_tail_recovery() {
    let dir = TempDir::new().expect("tempdir");
    let mut journal = Journal::open(dir.path()).expect("open");
    for i in 0..3 {
        journal
            .append(draft("tick", Some(&format!("work_{i}"))))
            .expect("append");
    }
    drop(journal);

    // Simulate a crash mid-append: a partial JSON line with no newline.
    let seg = segment_path(dir.path(), 1);
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(&seg)
        .expect("open segment");
    f.write_all(br#"{"schema":"sergeant.event/v1","seq":4,"id":"01TRUNC"#)
        .expect("write partial");
    drop(f);

    let mut journal = Journal::open(dir.path()).expect("reopen after crash");
    let events: Vec<Event> = journal
        .replay()
        .expect("replay")
        .collect::<Result<_, _>>()
        .expect("all complete events recovered");
    assert_eq!(events.len(), 3);
    assert_eq!(
        events.iter().map(|e| e.seq).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );

    // The partial tail was quarantined, not lost, and the segment is clean.
    let partial = fs::read_to_string(seg.with_extension("ndjson.partial")).expect("partial file");
    assert!(partial.contains(r#""seq":4"#));
    assert!(fs::read(&seg).expect("segment").ends_with(b"\n"));

    // The next append continues the seq.
    let fsyncs_before = journal.fsync_count();
    let e4 = journal
        .append(draft("tick", None))
        .expect("append after recovery");
    assert_eq!(e4.seq, 4);
    // Exactly one segment-data fsync accounted per acknowledged append.
    // Honest scope (contract Unknowns): `fsync_count` is a counter the
    // implementation maintains — its coupling to the real `sync_data` call
    // lives in `write_and_sync`'s single derived expression, not in anything
    // this assertion can observe; OS-level durability is unverifiable from
    // inside the process. The failure half of the guarantee is covered
    // behaviorally by this test's crash-recovery assertions above.
    assert_eq!(journal.fsync_count(), fsyncs_before + 1);
}

/// Acceptance test 2 (continued) — crash tail at the exact object boundary:
/// a journal whose final line is a COMPLETE JSON event but has NO trailing
/// newline is treated as crash damage. The append protocol only acknowledges
/// an event once its full line (object plus terminating '\n') is written and
/// fsynced, so a newline-less tail was never acknowledged: recovery
/// quarantines it to `<segment>.partial` and truncates, exactly like a
/// mid-JSON torn line. Every newline-terminated event survives and the next
/// append reuses the tail's seq.
#[test]
fn crash_tail_complete_json_without_newline_is_quarantined() {
    let dir = TempDir::new().expect("tempdir");
    let mut journal = Journal::open(dir.path()).expect("open");
    for i in 0..3 {
        journal
            .append(draft("tick", Some(&format!("work_{i}"))))
            .expect("append");
    }
    drop(journal);

    // Simulate a crash torn exactly at the object boundary: a complete,
    // parseable JSON event with the correct next seq — but no newline.
    let tail_event = draft("tick", Some("work_torn")).into_event(4);
    let tail_line = serde_json::to_string(&tail_event).expect("serialize tail");
    let seg = segment_path(dir.path(), 1);
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(&seg)
        .expect("open segment");
    f.write_all(tail_line.as_bytes())
        .expect("write newline-less tail");
    drop(f);

    // Recovery treats the newline-less tail as crash damage: quarantined,
    // not replayed — it was never acknowledged with a completed write.
    let mut journal = Journal::open(dir.path()).expect("reopen after crash");
    let events: Vec<Event> = journal
        .replay()
        .expect("replay")
        .collect::<Result<_, _>>()
        .expect("all newline-terminated events recovered");
    assert_eq!(
        events.iter().map(|e| e.seq).collect::<Vec<_>>(),
        vec![1, 2, 3],
        "the newline-less complete-JSON tail must not be replayed"
    );

    // The tail is preserved in the quarantine file, and the segment is clean.
    let partial = fs::read_to_string(seg.with_extension("ndjson.partial")).expect("partial file");
    assert!(partial.contains(&tail_event.id));
    assert!(partial.contains(r#""seq":4"#));
    assert!(fs::read(&seg).expect("segment").ends_with(b"\n"));

    // The next append reuses seq 4 — the quarantined tail never owned it.
    let e4 = journal
        .append(draft("tick", None))
        .expect("append after recovery");
    assert_eq!(e4.seq, 4);
    assert_ne!(e4.id, tail_event.id);
}

/// Acceptance test 3 — rotation: appends past the segment threshold create a
/// new segment, segments hold multiple events each (rotation is driven by
/// the size threshold, not per-append), and replay spans segments in order.
#[test]
fn segment_rotation_and_cross_segment_replay() {
    let dir = TempDir::new().expect("tempdir");
    // Threshold sized to hold several ~200-byte events per segment, small
    // enough that 10 appends must rotate at least once.
    let mut journal = Journal::open_with(dir.path(), 600).expect("open");
    for i in 0..10 {
        journal
            .append(draft("tick", Some(&format!("work_{i}"))))
            .expect("append");
    }
    drop(journal);

    let mut segments: Vec<String> = fs::read_dir(dir.path().join("journal"))
        .expect("read dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".ndjson"))
        .collect();
    segments.sort();
    assert!(
        segments.len() > 1,
        "expected multiple segments, got {segments:?}"
    );
    // Rotation is threshold-driven, not one-event-per-segment: at least one
    // segment must hold multiple complete (newline-terminated) events. A
    // rotate-on-every-append bug would leave every segment with one line.
    let events_per_segment: Vec<usize> = segments
        .iter()
        .map(|name| {
            let bytes = fs::read(dir.path().join("journal").join(name)).expect("read segment");
            assert!(
                bytes.is_empty() || bytes.ends_with(b"\n"),
                "segment {name} must end on a complete line"
            );
            bytes.iter().filter(|&&b| b == b'\n').count()
        })
        .collect();
    assert_eq!(
        events_per_segment.iter().sum::<usize>(),
        10,
        "every appended event lives in exactly one segment, got {events_per_segment:?}"
    );
    assert!(
        events_per_segment.iter().any(|&n| n > 1),
        "at least one segment must hold multiple events, got {events_per_segment:?}"
    );

    let journal = Journal::open_with(dir.path(), 600).expect("reopen");
    let seqs: Vec<u64> = journal
        .replay()
        .expect("replay")
        .map(|e| e.expect("event").seq)
        .collect();
    assert_eq!(seqs, (1..=10).collect::<Vec<_>>());
    assert_eq!(journal.next_seq(), 11);

    // Reopening a cleanly-closed journal is a recovery no-op: every segment
    // ends on a complete line, so no quarantine artifact may appear.
    let leftovers: Vec<String> = fs::read_dir(dir.path().join("journal"))
        .expect("read dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".partial"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "clean reopen must not quarantine anything, got {leftovers:?}"
    );
}

/// Acceptance test 4 — determinism: full replay twice yields identical
/// projection state, and snapshot at an arbitrary seq + tail replay is
/// identical to full replay.
#[test]
fn deterministic_replay_and_snapshot_equivalence() {
    let dir = TempDir::new().expect("tempdir");
    let mut journal = Journal::open_with(dir.path(), 512).expect("open");
    let kinds = ["work.created", "tool.completed", "work.done"];
    for i in 0..25u64 {
        let kind = kinds[(i % 3) as usize];
        let work = format!("work_{}", i % 5);
        journal.append(draft(kind, Some(&work))).expect("append");
    }

    // Full replay twice → identical state.
    let mut full_a = core_stats_projection();
    full_a
        .catch_up(journal.replay().expect("replay"))
        .expect("fold a");
    let mut full_b = core_stats_projection();
    full_b
        .catch_up(journal.replay().expect("replay"))
        .expect("fold b");
    assert_eq!(full_a.state(), full_b.state());
    assert_eq!(full_a.last_seq(), 25);
    assert_eq!(full_a.state().counts_by_kind["work.created"], 9);
    assert_eq!(full_a.state().work_ids.len(), 5);
    // The reducer-maintained last_seq (a contract-named element of CoreStats)
    // is pinned to a concrete value, independently of the projection
    // envelope's own tracker asserted above.
    assert_eq!(full_a.state().last_seq, 25);

    // Snapshot at every arbitrary cut point + tail replay ≡ full replay.
    for cut in [1u64, 7, 13, 24, 25] {
        let mut partial = core_stats_projection();
        partial
            .catch_up(journal.replay().expect("replay").take(cut as usize))
            .expect("partial fold");
        assert_eq!(partial.last_seq(), cut);
        assert_eq!(partial.state().last_seq, cut);
        let snap = dir.path().join(format!("snap_{cut}.json"));
        partial.write_snapshot(&snap).expect("write snapshot");

        let mut restored =
            Projection::<CoreStats>::load_snapshot(&snap, core_stats_reducer).expect("load");
        assert_eq!(restored.last_seq(), cut);
        let applied = restored
            .catch_up(journal.replay().expect("replay"))
            .expect("tail replay");
        assert_eq!(applied, 25 - cut);
        assert_eq!(restored.state(), full_a.state());
        assert_eq!(restored.last_seq(), full_a.last_seq());
    }

    // A snapshot whose last_seq is BEYOND the journal's end (a foreign or
    // stale snapshot, or this journal after losing its tail) fails closed on
    // catch-up instead of returning Ok over state the journal cannot
    // reproduce: the replay ends below the snapshot's last_seq.
    let short_dir = TempDir::new().expect("tempdir");
    let mut short_journal = Journal::open(short_dir.path()).expect("open short");
    for _ in 0..3 {
        short_journal
            .append(draft("work.created", Some("work_0")))
            .expect("append short");
    }
    let beyond_snap = dir.path().join("beyond_snap.json");
    full_a
        .write_snapshot(&beyond_snap)
        .expect("write beyond snapshot");
    let mut beyond = Projection::<CoreStats>::load_snapshot(&beyond_snap, core_stats_reducer)
        .expect("load beyond snapshot");
    let err = beyond
        .catch_up(short_journal.replay().expect("replay"))
        .expect_err("snapshot beyond journal end must fail closed");
    assert!(matches!(
        err,
        ProjectionError::SnapshotBeyondJournal {
            snapshot_seq: 25,
            journal_last_seq: 3,
        }
    ));

    // Same failure when the journal is completely empty.
    let empty_dir = TempDir::new().expect("tempdir");
    let empty_journal = Journal::open(empty_dir.path()).expect("open empty");
    let mut beyond = Projection::<CoreStats>::load_snapshot(&beyond_snap, core_stats_reducer)
        .expect("reload beyond snapshot");
    let err = beyond
        .catch_up(empty_journal.replay().expect("replay"))
        .expect_err("snapshot over an empty journal must fail closed");
    assert!(matches!(
        err,
        ProjectionError::SnapshotBeyondJournal {
            snapshot_seq: 25,
            journal_last_seq: 0,
        }
    ));

    // Unknown snapshot schema fails closed (fall back to full replay).
    let bad = dir.path().join("bad_snap.json");
    fs::write(
        &bad,
        br#"{"schema":"sergeant.snapshot/v99","last_seq":3,"state":{}}"#,
    )
    .expect("write bad snapshot");
    let err = Projection::<CoreStats>::load_snapshot(&bad, core_stats_reducer)
        .expect_err("unknown schema must fail closed");
    assert!(matches!(
        err,
        ProjectionError::UnknownSnapshotSchema { found } if found == "sergeant.snapshot/v99"
    ));
}

/// Acceptance test 5 — seq integrity: replay detects a gap or duplicate seq
/// and fails closed (error, not silent skip); append rejects regressions.
#[test]
fn seq_gap_or_duplicate_fails_closed() {
    // Gap: 1, 2, 4.
    let dir = TempDir::new().expect("tempdir");
    {
        let mut journal = Journal::open(dir.path()).expect("open");
        journal.append(draft("tick", None)).expect("append 1");
        journal.append(draft("tick", None)).expect("append 2");
    }
    let seg = segment_path(dir.path(), 1);
    let gap_line = serde_json::to_string(&draft("tick", None).into_event(4)).expect("serialize");
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(&seg)
        .expect("open segment");
    writeln!(f, "{gap_line}").expect("write gap line");
    drop(f);

    let results: Vec<_> = Journal::replay_data_dir(dir.path())
        .expect("replay")
        .collect();
    assert_eq!(
        results.len(),
        3,
        "replay must stop at the error, not skip it"
    );
    assert!(results[0].is_ok() && results[1].is_ok());
    assert!(matches!(
        results[2],
        Err(JournalError::SeqDiscontinuity {
            expected: 3,
            found: 4,
            ..
        })
    ));
    // A gapped journal also refuses to open for writing.
    assert!(Journal::open(dir.path()).is_err());

    // Duplicate: 1, 2, 2.
    let dir = TempDir::new().expect("tempdir");
    {
        let mut journal = Journal::open(dir.path()).expect("open");
        journal.append(draft("tick", None)).expect("append 1");
        journal.append(draft("tick", None)).expect("append 2");
    }
    let seg = segment_path(dir.path(), 1);
    let dup_line = serde_json::to_string(&draft("tick", None).into_event(2)).expect("serialize");
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(&seg)
        .expect("open segment");
    writeln!(f, "{dup_line}").expect("write dup line");
    drop(f);

    let results: Vec<_> = Journal::replay_data_dir(dir.path())
        .expect("replay")
        .collect();
    assert!(matches!(
        results.last(),
        Some(Err(JournalError::SeqDiscontinuity {
            expected: 3,
            found: 2,
            ..
        }))
    ));

    // Malformed line mid-journal: replay stops with an error at the garbage
    // line and never reaches the valid events after it (fail closed, not
    // silent skip), and the journal refuses to open for writing.
    let dir = TempDir::new().expect("tempdir");
    {
        let mut journal = Journal::open(dir.path()).expect("open");
        journal.append(draft("tick", None)).expect("append 1");
        journal.append(draft("tick", None)).expect("append 2");
    }
    let seg = segment_path(dir.path(), 1);
    let valid_line = serde_json::to_string(&draft("tick", None).into_event(3)).expect("serialize");
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(&seg)
        .expect("open segment");
    // Newline-terminated garbage: a complete (not crash-truncated) line, so
    // tail recovery must NOT absorb it; replay must fail on it.
    writeln!(f, "{{this is not an event}}").expect("write garbage line");
    writeln!(f, "{valid_line}").expect("write valid line after garbage");
    drop(f);

    let results: Vec<_> = Journal::replay_data_dir(dir.path())
        .expect("replay")
        .collect();
    assert_eq!(
        results.len(),
        3,
        "replay must stop at the malformed line, not skip past it"
    );
    assert!(results[0].is_ok() && results[1].is_ok());
    assert!(matches!(
        results[2],
        Err(JournalError::Malformed { line: 3, .. })
    ));
    assert!(Journal::open(dir.path()).is_err());

    // Append rejects seq regressions and skips outright.
    let dir = TempDir::new().expect("tempdir");
    let mut journal = Journal::open(dir.path()).expect("open");
    journal.append(draft("tick", None)).expect("append 1");
    let stale = draft("tick", None).into_event(1);
    assert!(matches!(
        journal.append_event(&stale),
        Err(JournalError::SeqRegression {
            attempted: 1,
            expected: 2
        })
    ));
    let skip = draft("tick", None).into_event(5);
    assert!(matches!(
        journal.append_event(&skip),
        Err(JournalError::SeqRegression {
            attempted: 5,
            expected: 2
        })
    ));

    // The projection itself also refuses non-contiguous folds.
    let mut projection = core_stats_projection();
    let e1 = draft("tick", None).into_event(1);
    let e3 = draft("tick", None).into_event(3);
    projection.apply(&e1).expect("apply 1");
    assert!(matches!(
        projection.apply(&e3),
        Err(ProjectionError::SeqMismatch {
            expected: 2,
            found: 3
        })
    ));
}

/// Acceptance test 6 — blob store: put/get round-trips, identical content
/// deduplicates to one file, and a hash mismatch on read fails closed.
#[test]
fn blob_store_round_trip_dedup_and_mismatch() {
    let dir = TempDir::new().expect("tempdir");
    let store = BlobStore::open(dir.path()).expect("open");

    let content = b"a large diff or transcript chunk".to_vec();
    let r1 = store.put(&content).expect("put");
    // The address is pinned to the real blake3 digest at full 64-hex width —
    // any other algorithm, a truncated address, or length-based addressing
    // fails here.
    assert_eq!(
        r1.hex(),
        "2db48e4636dc521756c48da29b53f16879ee369b56b76722b722736dd3ddfdf7"
    );
    assert_eq!(store.get(&r1).expect("get"), content);

    // Distinct content — deliberately the same length as `content` — gets a
    // distinct pinned address, and both blobs retrieve correctly.
    let other = b"another 32-byte evidence payload".to_vec();
    assert_eq!(other.len(), content.len());
    let r_other = store.put(&other).expect("put other");
    assert_ne!(r_other, r1);
    assert_eq!(
        r_other.hex(),
        "f70a9307c03958c7012a66094622bc3eb2e87c6624c3524e32e7fc04de49c4af"
    );
    assert_eq!(store.get(&r_other).expect("get other"), other);

    // Events carry the ref as a "b3:<hex>" string; it parses back.
    let as_string = r1.to_string();
    assert!(as_string.starts_with("b3:"));
    let parsed: BlobRef = as_string.parse().expect("parse ref");
    assert_eq!(parsed, r1);
    assert!("b3:not-hex".parse::<BlobRef>().is_err());
    assert!("sha256:aaaa".parse::<BlobRef>().is_err());
    // Correct length and prefix but non-hex / non-lowercase characters.
    assert!(format!("b3:{}", "g".repeat(64)).parse::<BlobRef>().is_err());
    assert!(format!("b3:{}", "A".repeat(64)).parse::<BlobRef>().is_err());
    // Valid hex but the wrong length — a truncated or padded ref must be
    // rejected by the length half of the validation, not just the charset
    // half (a short ref would otherwise address a bogus path like
    // `blobs/b3/abcd`).
    assert!("b3:abcd".parse::<BlobRef>().is_err());
    assert!(format!("b3:{}", "a".repeat(63)).parse::<BlobRef>().is_err());
    assert!(format!("b3:{}", "a".repeat(65)).parse::<BlobRef>().is_err());

    // Identical content deduplicates to one file, stored under exactly its
    // full blake3 hex — no tmp litter, no extra files, no other names.
    let r2 = store.put(&content).expect("re-put");
    assert_eq!(r2, r1);
    let mut names: Vec<String> = fs::read_dir(dir.path().join("blobs").join("b3"))
        .expect("read dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut expected_names = vec![r1.hex().to_string(), r_other.hex().to_string()];
    expected_names.sort();
    assert_eq!(names, expected_names);

    // Write-once semantics: an existing blob is never rewritten. Tamper the
    // stored file, re-put the original bytes, and the tampered bytes must
    // still be on disk — a rewrite-on-put implementation would fail here.
    let r1_path = dir.path().join("blobs").join("b3").join(r1.hex());
    fs::write(&r1_path, b"tampered bytes").expect("tamper");
    let r3 = store.put(&content).expect("put over existing blob");
    assert_eq!(r3, r1);
    assert_eq!(
        fs::read(&r1_path).expect("read stored blob"),
        b"tampered bytes",
        "put must not rewrite an existing blob"
    );

    // Hash mismatch on read fails closed.
    let err = store.get(&r1).expect_err("tampered blob must fail closed");
    assert!(matches!(err, BlobError::HashMismatch { .. }));

    // Missing blob is a clean error too.
    let missing: BlobRef = format!("b3:{}", "0".repeat(64)).parse().expect("ref");
    assert!(matches!(store.get(&missing), Err(BlobError::NotFound(_))));
}

/// Invariant — single-writer per journal directory, not just per handle: a
/// second live writer on the same data dir is refused (`Locked`), so two
/// handles can never fsync-acknowledge colliding seqs. The lock dies with
/// the handle, so a crash never wedges reopen.
#[test]
fn second_live_writer_on_same_data_dir_is_refused() {
    let dir = TempDir::new().expect("tempdir");
    let mut a = Journal::open(dir.path()).expect("open first writer");
    a.append(draft("tick", None)).expect("append");

    let err = Journal::open(dir.path()).expect_err("second live writer must be refused");
    assert!(matches!(err, JournalError::Locked), "got {err:?}");

    // Read-only replay is still allowed alongside the live writer.
    let seqs: Vec<u64> = Journal::replay_data_dir(dir.path())
        .expect("replay")
        .map(|e| e.expect("event").seq)
        .collect();
    assert_eq!(seqs, vec![1]);

    // Dropping the writer releases the lock; reopen continues the seq.
    drop(a);
    let mut b = Journal::open(dir.path()).expect("reopen after drop");
    assert_eq!(b.append(draft("tick", None)).expect("append").seq, 2);
}

/// Invariant — a blank line in a segment is unexplained content (no append
/// path ever writes one), so replay fails closed on it exactly like any
/// other malformed line, and the journal refuses to open for writing.
#[test]
fn blank_journal_line_fails_closed() {
    let dir = TempDir::new().expect("tempdir");
    {
        let mut journal = Journal::open(dir.path()).expect("open");
        journal.append(draft("tick", None)).expect("append 1");
        journal.append(draft("tick", None)).expect("append 2");
    }
    let seg = segment_path(dir.path(), 1);
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(&seg)
        .expect("open segment");
    f.write_all(b"\n").expect("inject blank line");
    drop(f);

    let results: Vec<_> = Journal::replay_data_dir(dir.path())
        .expect("replay")
        .collect();
    assert_eq!(results.len(), 3, "replay must stop at the blank line");
    assert!(results[0].is_ok() && results[1].is_ok());
    assert!(matches!(
        results[2],
        Err(JournalError::Malformed { line: 3, .. })
    ));
    assert!(Journal::open(dir.path()).is_err());
}

/// Invariant — a snapshot rewrite lands as an atomic replace (a fresh file
/// renamed over the path), never as an in-place truncate of the previous
/// snapshot, and leaves no tmp litter behind.
#[test]
fn snapshot_rewrite_is_atomic_replace() {
    let dir = TempDir::new().expect("tempdir");
    let snap = dir.path().join("snapshots").join("core.json");

    let mut projection = core_stats_projection();
    let e1 = draft("tick", Some("work_1")).into_event(1);
    projection.apply(&e1).expect("apply 1");
    projection.write_snapshot(&snap).expect("write snapshot");
    #[cfg(unix)]
    let first_inode = {
        use std::os::unix::fs::MetadataExt;
        fs::metadata(&snap).expect("metadata").ino()
    };

    let e2 = draft("tick", Some("work_2")).into_event(2);
    projection.apply(&e2).expect("apply 2");
    projection.write_snapshot(&snap).expect("rewrite snapshot");

    // In-place truncation would have kept the inode.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_ne!(
            fs::metadata(&snap).expect("metadata").ino(),
            first_inode,
            "snapshot rewrite must arrive by rename, not in-place truncation"
        );
    }

    // The rewritten snapshot is complete and loadable, and no tmp files
    // remain next to it.
    let restored = Projection::<CoreStats>::load_snapshot(&snap, core_stats_reducer).expect("load");
    assert_eq!(restored.last_seq(), 2);
    let names: Vec<String> = fs::read_dir(snap.parent().expect("parent"))
        .expect("read dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["core.json"]);
}

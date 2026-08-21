//! Append-only segmented NDJSON event journal (proposal §21).
//!
//! Layout under a caller-supplied data dir:
//!
//! ```text
//! <data-dir>/journal/00000001.ndjson
//! <data-dir>/journal/00000002.ndjson
//! ```
//!
//! Single writer, one complete event per line, size-based segment rotation.
//! Single-writer is enforced per journal directory with an exclusive advisory
//! lock (`journal/.lock`), not just per handle — opening a second writer on a
//! live journal fails with [`JournalError::Locked`]. A trailing incomplete
//! line left by a crash is quarantined to `<segment>.partial` on open and the
//! segment is truncated back to its last complete line; no complete line is
//! ever lost. Replay yields all events in seq order across segments and fails
//! closed on a gap or duplicate seq.
//!
//! # Write and fsync are two steps (issue #44, group commit)
//!
//! [`Journal::append_event`] writes the line and returns; [`Journal::sync`]
//! is what makes everything written since the last sync durable, in **one**
//! `fsync` however many lines that is. The journal does not decide where the
//! group boundary is — its one caller, [`Core`](crate::api::Core), puts it at
//! the end of an authoritative core-lock hold, which is the only instant at
//! which anything outside the daemon can observe an appended event (see
//! [`CoreGuard`](crate::api::CoreGuard) for the durability contract and its
//! L6 analysis).
//!
//! Two properties keep that split honest:
//!
//! - **Written is visible.** The line reaches the file inside `append_event`,
//!   so every in-process reader ([`Journal::replay`], `replay_after`, the
//!   analytical catch-up) sees an appended event whether or not it has been
//!   synced. Only *durability across a crash* is deferred.
//! - **Nothing is left unsynced by accident.** Rotation syncs the segment it
//!   is leaving, a failed append's rollback syncs the truncation, and
//!   [`Journal`]'s `Drop` syncs a handle that still holds unsynced bytes.
//!   A written line therefore has exactly one way to be lost: a crash before
//!   its group's sync — the window this module's caller reasons about.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::domain::event::{Event, EventDraft};
use crate::runtime::fsutil::{create_dir_all_durable, take_exclusive_lock};

/// Default segment rotation threshold.
pub const DEFAULT_SEGMENT_MAX_BYTES: u64 = 8 * 1024 * 1024;

/// Observer of per-append latency, installed by the daemon when §28 export is
/// on. Called after each append with the time the write took. Nothing in the
/// journal reads it back — it is a one-way seam for the one §28 metric whose
/// input exists only inside this module.
///
/// Since #44 this is the *write*, not the write plus its fsync: the fsync is
/// shared by the whole group and belongs to no single append. The metric it
/// feeds (`sergeant_journal_append_seconds`) therefore got cheaper and
/// narrower on the same day the group commit landed — recorded here because a
/// histogram that silently changes what it measures is worse than one that
/// changes loudly.
pub type AppendObserver = Arc<dyn Fn(Duration) + Send + Sync>;

/// Advisory write-lock file inside the journal dir (never a segment name).
const LOCK_FILE_NAME: &str = ".lock";

/// Highest segment index whose file name round-trips through
/// [`list_segments`]' fixed 8-digit stem. A 9-digit segment would be written
/// and fsync-acknowledged yet invisible to every replay, so segment creation
/// fails closed at this bound instead of silently losing events.
const MAX_SEGMENT_INDEX: u64 = 99_999_999;

/// Errors from the journal.
#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    /// Underlying filesystem failure.
    #[error("journal io error: {0}")]
    Io(#[from] std::io::Error),
    /// An event failed to serialize on append.
    #[error("journal serialize error: {0}")]
    Serialize(serde_json::Error),
    /// A stored line failed to parse as an event. Fail closed: history is
    /// corrupt and must not be silently skipped.
    #[error("malformed event at {segment} line {line}: {source}")]
    Malformed {
        /// Segment file name.
        segment: String,
        /// 1-based line number within the segment.
        line: u64,
        /// Parse failure.
        source: serde_json::Error,
        /// Whether this could be a live writer's torn in-progress append
        /// rather than corruption — see [`JournalError::is_possible_torn_tail`].
        /// Always `false` from [`first_seq`], which only ever reads a
        /// segment's first line and has no "is this the end of everything"
        /// context to classify with.
        possible_torn_tail: bool,
    },
    /// Replay found a seq that is not exactly `expected` — a gap (found >
    /// expected) or a duplicate/regression (found <= previous). Fail closed.
    #[error("seq discontinuity at {segment} line {line}: expected {expected}, found {found}")]
    SeqDiscontinuity {
        /// Segment file name.
        segment: String,
        /// 1-based line number within the segment.
        line: u64,
        /// The seq replay required next.
        expected: u64,
        /// The seq actually stored.
        found: u64,
    },
    /// An append supplied a seq other than the next one (regression or skip).
    #[error("append seq regression: attempted {attempted}, next is {expected}")]
    SeqRegression {
        /// Seq the caller tried to append.
        attempted: u64,
        /// The only seq the journal will accept next.
        expected: u64,
    },
    /// A failed append left torn bytes in the segment and the rollback
    /// truncation also failed, so the handle refuses further writes. Reopen
    /// the journal (which recovers and re-validates) to continue.
    #[error("journal handle poisoned: a torn append could not be rolled back; reopen the journal")]
    Poisoned,
    /// Another live writer already holds this journal directory's exclusive
    /// lock. Single-writer is enforced per data dir, not just per handle:
    /// two live writers would fsync-acknowledge colliding seqs, leaving one
    /// acknowledged event permanently unreplayable.
    #[error("journal is already open for writing (exclusive lock held by another handle)")]
    Locked,
    /// Rotation would create a segment index past the 8-digit file-name
    /// namespace. Such a segment would be appended to and fsync-acknowledged
    /// yet never listed by replay — silent event loss — so creation fails
    /// closed instead. Unreachable at sergeant's scale (~800 PB of journal at
    /// the default threshold), guarded so it can never be silent.
    #[error("segment index {attempted} exceeds the 8-digit segment namespace (max {max})")]
    SegmentIndexOverflow {
        /// The segment index rotation tried to create.
        attempted: u64,
        /// The largest index the file-name scheme can represent.
        max: u64,
    },
    /// W3: a prune target was not an oldest-contiguous prefix of the live
    /// segment set (I-W3-4). Refused before anything is touched.
    #[error(
        "refusing to unlink segments {attempted:?}: not an oldest-contiguous prefix of {live:?}"
    )]
    UnlinkNotAPrefix {
        /// The indices the caller asked to unlink.
        attempted: Vec<u64>,
        /// The indices actually live on disk, oldest first.
        live: Vec<u64>,
    },
    /// W3: a prune target named the segment this writer holds open — the
    /// segment `append_event` still writes to (I-W3-4, N15).
    #[error("refusing to unlink segment {index}: it is the live append target")]
    UnlinkLiveSegment {
        /// The offending segment index.
        index: u64,
    },
}

impl JournalError {
    /// True iff this could be a live writer's torn in-progress append rather
    /// than corruption (issue #169's L6 question, answered): the failing
    /// line is the last line of the last segment, with nothing readable
    /// after it — exactly the shape `append_event`'s single unbuffered
    /// `write_all` can leave for a reader racing it, and the *only* shape
    /// it can leave, since rotation always starts a fresh segment before
    /// the writer appends to it. A concurrent reader of a live journal
    /// (`Journal::replay_data_dir` included) may observe exactly this case
    /// and must tolerate/retry it; every other `Malformed` — a torn or
    /// invalid line with something well-formed after it, or one anywhere
    /// but the last segment — is corruption, not a race, and must not be
    /// retried away.
    ///
    /// `false` for every other error variant, and for a `Malformed` from
    /// [`Journal::replay`]'s own tail-recovering `open` path (which never
    /// leaves a torn line to see in the first place) or from `first_seq`
    /// (which has no "is this the very end" context to classify with).
    pub fn is_possible_torn_tail(&self) -> bool {
        matches!(
            self,
            JournalError::Malformed {
                possible_torn_tail: true,
                ..
            }
        )
    }
}

/// Single-writer handle to the segmented journal.
pub struct Journal {
    journal_dir: PathBuf,
    segment_max_bytes: u64,
    segment_index: u64,
    segment_file: File,
    segment_len: u64,
    next_seq: u64,
    fsync_count: u64,
    /// Bytes are in the current segment that no `fsync` has covered yet — the
    /// open group. One flag, not a length, because `sync_data` covers the
    /// whole file: how *much* is unsynced never changes what the sync does.
    dirty: bool,
    poisoned: bool,
    /// Optional observer of append latency (§28's
    /// `sergeant_journal_append_seconds`). `None` by default and in every
    /// path but a daemon with export switched on: the journal must not gain
    /// a dependency on the telemetry module, and the only place this timing
    /// exists is here.
    append_observer: Option<AppendObserver>,
    /// W3 §10.4: whether a rotation has happened since the last
    /// [`Journal::take_rotation_signal`] call. Set by `rotate_if_needed`,
    /// cleared by the read. A one-shot flag rather than an observer callback
    /// — see that method's own doc for why.
    rotation_signal: bool,
    /// Exclusive advisory lock on the journal dir, held for the lifetime of
    /// the handle. The OS releases it when the handle drops — including on
    /// crash — so a stale lock can never wedge reopen.
    _lock: File,
}

// Hand-written because the append observer is a closure, which has no
// `Debug`. Everything a reader of a `{:?}` journal actually wants — where it
// is, how far it has got, whether it is usable — is here.
impl std::fmt::Debug for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Journal")
            .field("journal_dir", &self.journal_dir)
            .field("segment_max_bytes", &self.segment_max_bytes)
            .field("segment_index", &self.segment_index)
            .field("segment_len", &self.segment_len)
            .field("next_seq", &self.next_seq)
            .field("fsync_count", &self.fsync_count)
            .field("dirty", &self.dirty)
            .field("poisoned", &self.poisoned)
            .field("append_observer", &self.append_observer.is_some())
            .field("rotation_signal", &self.rotation_signal)
            .finish()
    }
}

impl Journal {
    /// Open (creating if needed) the journal under `<data_dir>/journal` with
    /// the default rotation threshold.
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self, JournalError> {
        Self::open_with(data_dir, DEFAULT_SEGMENT_MAX_BYTES)
    }

    /// Open with an explicit segment rotation threshold (bytes).
    ///
    /// Open recovers a crashed tail, then learns the next seq by replaying
    /// **only the last non-empty segment** ([`tail_next_seq`]) — validating
    /// seq contiguity for that segment alone, not the whole chain (W2, §1.5
    /// of the wave spec). What this gives up, and what covers it: the
    /// startup shared pass validates everything it reads (the whole surviving
    /// chain on a cache miss), and the startup cache's per-segment BLAKE3
    /// binding validates everything below the pass's start — any mutation of
    /// any byte in a summarized segment invalidates the cache and forces a
    /// full floor-aware replay, which then validates it the old way. A BLAKE3
    /// over the bytes is *stronger* than a seq scan (which only catches a
    /// missing or duplicated line), not weaker.
    pub fn open_with(
        data_dir: impl AsRef<Path>,
        segment_max_bytes: u64,
    ) -> Result<Self, JournalError> {
        let journal_dir = data_dir.as_ref().join("journal");
        // Durable creation: a fresh `journal/` (and any missing ancestors,
        // including the data dir) has its dirent fsynced in the parent, so
        // the first fsync-acknowledged append cannot vanish with the
        // directory itself after a crash.
        create_dir_all_durable(&journal_dir)?;

        // Single-writer per directory, not just per handle (&mut self, no
        // Clone): take an exclusive advisory lock before touching any segment
        // (tail recovery below mutates files and must run under it). A second
        // live writer on the same dir is refused instead of both handles
        // fsync-acknowledging appends with colliding seqs.
        let lock_path = journal_dir.join(LOCK_FILE_NAME);
        let lock = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)?;
        if !take_exclusive_lock(&lock_path, &lock)? {
            return Err(JournalError::Locked);
        }

        let mut segments = list_segments(&journal_dir)?;
        if let Some((index, path)) = segments.last() {
            recover_tail(&journal_dir, *index, path)?;
        }

        let next_seq = tail_next_seq(&segments)?;

        let (segment_index, segment_path) = match segments.pop() {
            Some(last) => last,
            None => create_segment(&journal_dir, 1)?,
        };
        let segment_file = OpenOptions::new().append(true).open(&segment_path)?;
        let segment_len = segment_file.metadata()?.len();

        Ok(Self {
            journal_dir,
            segment_max_bytes,
            segment_index,
            segment_file,
            segment_len,
            next_seq,
            fsync_count: 0,
            dirty: false,
            poisoned: false,
            append_observer: None,
            rotation_signal: false,
            _lock: lock,
        })
    }

    /// The seq the next append will receive.
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// Number of segment-data fsyncs this handle has issued. Counts only
    /// `sync_data` on a segment — not the directory syncs of rotation — and
    /// increments only from the syscall's success value.
    ///
    /// Since #44 the unit is a **group commit**, not an append: one fsync
    /// covers every line written since the previous one. So this is the count
    /// of [`Journal::sync`] calls that found work to do, plus the rotation
    /// boundary syncs, and `fsync_count() <= appends` is now the expected
    /// shape rather than a bug.
    ///
    /// Durability of fsync on the host filesystem is unverifiable from inside
    /// the process; this counter lets tests assert how many fsyncs a sequence
    /// of appends actually costs.
    pub fn fsync_count(&self) -> u64 {
        self.fsync_count
    }

    /// Whether any written line is not yet covered by an fsync — i.e. whether
    /// a group is open. Cheap enough to gate a `Drop` on.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Append a draft: assigns the next seq, stamps id/timestamp, and writes
    /// one NDJSON line. Durability waits for [`Journal::sync`].
    pub fn append(&mut self, draft: EventDraft) -> Result<Event, JournalError> {
        let event = draft.into_event(self.next_seq);
        self.append_event(&event)?;
        Ok(event)
    }

    /// Fsync every line written since the last sync — the group commit.
    ///
    /// One syscall, whatever the group's size, and a no-op when nothing is
    /// dirty (so a read-only lock hold costs nothing).
    ///
    /// **Failure poisons the handle, deliberately.** A failed *write* is
    /// rolled back, because nothing has been told about it yet. A failed
    /// group *fsync* cannot be: by the time it runs, every event in the group
    /// is already folded into the in-memory projections that the daemon's
    /// next decision reads, and truncating them off disk would leave those
    /// projections describing a history the journal does not have. So the
    /// handle fails closed instead — every later append is refused with
    /// [`JournalError::Poisoned`] until the process restarts, and the restart
    /// rebuilds from whatever the filesystem actually kept. Fail closed on
    /// ambiguity, never a guess.
    pub fn sync(&mut self) -> Result<(), JournalError> {
        if self.poisoned {
            return Err(JournalError::Poisoned);
        }
        if !self.dirty {
            return Ok(());
        }
        if let Err(err) = self.sync_now() {
            self.poisoned = true;
            return Err(err.into());
        }
        Ok(())
    }

    /// Append a fully-formed event. The event's seq must be exactly the next
    /// seq; anything else (regression, duplicate, skip) is rejected.
    ///
    /// Writes the line; does **not** fsync it. See [`Journal::sync`].
    pub fn append_event(&mut self, event: &Event) -> Result<(), JournalError> {
        if self.poisoned {
            return Err(JournalError::Poisoned);
        }
        if event.seq != self.next_seq {
            return Err(JournalError::SeqRegression {
                attempted: event.seq,
                expected: self.next_seq,
            });
        }
        self.rotate_if_needed()?;
        let mut line = serde_json::to_vec(event).map_err(JournalError::Serialize)?;
        line.push(b'\n');
        let started = self.append_observer.as_ref().map(|_| Instant::now());
        if let Err(err) = self.segment_file.write_all(&line) {
            // A failed write_all can leave torn, un-terminated bytes in the
            // segment while the handle stays otherwise usable; a later
            // acknowledged append would then concatenate onto the fragment and
            // become unreplayable. Roll the segment back to its pre-append
            // length so that can never happen; if the rollback itself fails,
            // poison the handle so every further append is refused until the
            // journal is reopened (which recovers and re-validates).
            //
            // `segment_len` is the *written* length, which may include an open
            // group's still-unsynced lines: the truncation keeps those and the
            // sync below makes them durable early. Durable-sooner is never
            // wrong, so the group simply closes here — `dirty` clears with it.
            if self.segment_file.set_len(self.segment_len).is_err()
                || self.segment_file.sync_data().is_err()
            {
                self.poisoned = true;
            } else {
                self.dirty = false;
            }
            return Err(err.into());
        }
        self.segment_len += line.len() as u64;
        self.dirty = true;
        self.next_seq += 1;
        if let (Some(observer), Some(started)) = (&self.append_observer, started) {
            observer(started.elapsed());
        }
        Ok(())
    }

    /// Observe how long each successful append takes.
    ///
    /// Set by the daemon only when §28 export is on. An observer that panics
    /// or blocks would do so inside the single-writer path, so the contract
    /// on it is the same as any metric callback: cheap and infallible.
    pub fn set_append_observer(&mut self, observer: AppendObserver) {
        self.append_observer = Some(observer);
    }

    /// Issue the one accounted durability syscall and close the group.
    ///
    /// The fsync counter is derived from the syscall's success value in a
    /// single expression, so it cannot advance without `sync_data` actually
    /// returning `Ok` — and `dirty` clears only on the same success, so a
    /// failed sync leaves the group open rather than silently "committed".
    fn sync_now(&mut self) -> std::io::Result<()> {
        self.fsync_count += self.segment_file.sync_data().map(|()| 1)?;
        self.dirty = false;
        Ok(())
    }

    /// Iterate every committed event in seq order across all segments.
    /// Yields an error (and then stops) on malformed lines or seq
    /// discontinuities — fail closed, never silently skip.
    pub fn replay(&self) -> Result<Replay, JournalError> {
        Ok(Replay::new(list_segments(&self.journal_dir)?))
    }

    /// Iterate every committed event with `seq > after`, skipping whole
    /// segments that cannot contain one.
    ///
    /// [`Journal::replay`] is the *rebuild* primitive: it starts at seq 1 and
    /// validates the whole chain, which is exactly what a daemon start and a
    /// projection rebuild want. It is the wrong primitive for a caller that
    /// is already caught up to `after`, because it costs O(total journal) to
    /// discover that the answer is empty — and the callers that ask for the
    /// tail (the analytical projection's read-time catch-up, `/v1/events`,
    /// the SSE resume) hold the daemon's single mutation lock while they do
    /// it, so that cost lands on `submit`/`cancel`/`input`. This reads one
    /// line per segment to find the first segment that can contain
    /// `after + 1`, then parses only from there: O(segments) plus O(events
    /// actually wanted), rather than O(history).
    ///
    /// The seq-continuity check still applies to everything it does read —
    /// it simply starts expecting the first seq of the first kept segment
    /// rather than 1. Full-chain validation is not weakened where it
    /// matters: every daemon start replays from 1 through [`Journal::open`]
    /// and the projection rebuild.
    pub fn replay_after(&self, after: u64) -> Result<Replay, JournalError> {
        Replay::after(list_segments(&self.journal_dir)?, after)
    }

    /// Replay a journal directory without opening a writer handle (and
    /// without tail recovery). Useful for read-only inspection and tests.
    ///
    /// Without a writer handle there is no exclusive lock stopping this from
    /// running against a *live* journal a daemon is actively appending to
    /// (`sgt doctor`'s own journal check is exactly that caller). A reader
    /// racing `append_event`'s single unbuffered `write_all` can therefore
    /// observe a torn partial line — always the last line of the last
    /// segment, since rotation never leaves a torn write behind a later
    /// segment. [`JournalError::is_possible_torn_tail`] names that one
    /// case; a caller reading a journal it does not know to be quiescent
    /// must tolerate or retry it rather than treating it as corruption.
    /// Every other `Malformed` — anywhere else, or with something readable
    /// after it — is corruption, not a race, and must fail closed exactly
    /// as it does today.
    pub fn replay_data_dir(data_dir: impl AsRef<Path>) -> Result<Replay, JournalError> {
        Ok(Replay::new(list_segments(
            &data_dir.as_ref().join("journal"),
        )?))
    }

    /// Extents of every segment that holds at least one event, oldest first.
    ///
    /// One line read per segment ([`first_seq`]) plus one `metadata()` — the
    /// same cost [`Replay::after`] already pays to find its start. Segments
    /// created by rotation but never appended to are skipped: they have no
    /// first seq and cannot bound anything. `last_seq` of the newest bound
    /// segment is `self.next_seq() - 1` (the last event this writer knows
    /// about); every other segment's `last_seq` is the following segment's
    /// `first_seq - 1`.
    pub fn segment_bounds(&self) -> Result<Vec<SegmentBound>, JournalError> {
        let segments = list_segments(&self.journal_dir)?;
        let mut bounds: Vec<SegmentBound> = Vec::new();
        for (index, path) in segments {
            let Some(first) = first_seq(&path)? else {
                continue;
            };
            let bytes = fs::metadata(&path)?.len();
            bounds.push(SegmentBound {
                index,
                path,
                first_seq: first,
                last_seq: 0,
                bytes,
            });
        }
        let len = bounds.len();
        for i in 0..len {
            bounds[i].last_seq = if i + 1 < len {
                bounds[i + 1].first_seq - 1
            } else {
                self.next_seq.saturating_sub(1)
            };
        }
        Ok(bounds)
    }

    /// The lowest seq any surviving segment holds — the replay floor.
    /// `None` for an empty journal.
    pub fn floor_seq(&self) -> Result<Option<u64>, JournalError> {
        Ok(self.segment_bounds()?.first().map(|b| b.first_seq))
    }

    /// **A1's redefinition of "full replay":** every event from the oldest
    /// surviving segment's `first_seq` to the tail.
    ///
    /// Identical to [`Journal::replay`] while the oldest segment is segment 1
    /// starting at seq 1 — which is every journal in production until W3
    /// exists. After a prune it is the only correct spelling: [`Replay::new`]
    /// hardcodes `expected = 1` and would report ruled retention as
    /// [`JournalError::SeqDiscontinuity`]. Implemented as
    /// `self.replay_after(floor - 1)`, so [`Journal::replay_after`]'s existing
    /// floor-skip does the work.
    pub fn replay_from_floor(&self) -> Result<Replay, JournalError> {
        let after = self.floor_seq()?.map_or(0, |f| f - 1);
        self.replay_after(after)
    }

    /// Read-only, lock-free counterpart to [`Journal::replay_from_floor`] for
    /// out-of-process readers — mirrors [`Journal::replay_data_dir`]'s
    /// relationship to [`Journal::replay`].
    pub fn replay_data_dir_from_floor(data_dir: impl AsRef<Path>) -> Result<Replay, JournalError> {
        let journal_dir = data_dir.as_ref().join("journal");
        let segments = list_segments(&journal_dir)?;
        let mut floor = None;
        for (_, path) in &segments {
            match first_seq(path) {
                Ok(Some(first)) => {
                    floor = Some(first);
                    break;
                }
                Ok(None) => continue,
                // W3 §11.3 (recon correction 7): this segment went away
                // during the skip-scan itself — a prune raced this
                // lock-free read. Skip it; the next surviving segment names
                // the floor instead.
                Err(JournalError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            }
        }
        let after = floor.map_or(0, |f| f - 1);
        Replay::after(segments, after)
    }

    fn rotate_if_needed(&mut self) -> Result<(), JournalError> {
        if self.segment_len < self.segment_max_bytes || self.segment_len == 0 {
            return Ok(());
        }
        // Close any open group on the segment being left behind. `dirty` is a
        // property of `segment_file`, and that handle is about to be replaced:
        // without this, a group whose sync lands after a rotation would fsync
        // the *new* segment and silently never cover the lines it was opened
        // for. A rotation boundary is a group boundary.
        self.sync()?;
        let (index, path) = create_segment(&self.journal_dir, self.segment_index + 1)?;
        self.segment_file = OpenOptions::new().append(true).open(&path)?;
        self.segment_index = index;
        self.segment_len = 0;
        // create_segment fsyncs the directory; deliberately not counted in
        // fsync_count, which tracks only per-append segment-data syncs.
        // W3 §10.4: a real rotation happened — arm the one-shot signal the
        // daemon's tick asks about at a group boundary.
        self.rotation_signal = true;
        Ok(())
    }

    /// W3 §10.4: whether a rotation has happened since the last call.
    /// Reading clears it.
    ///
    /// A flag rather than a callback because the prune it arms must not run
    /// *inside* `append_event` — that call is mid-group, mid-fold, and
    /// holding the writer's own file handle. The daemon asks this at a tick
    /// boundary, where a failure is reportable and the group is closed.
    pub fn take_rotation_signal(&mut self) -> bool {
        std::mem::replace(&mut self.rotation_signal, false)
    }

    /// W3: unlink whole segment files, oldest-first — the only deletion door
    /// this type has, and deliberately narrow.
    ///
    /// Refuses, before touching anything:
    ///   - a target naming [`Journal::segment_index`](Self)'s current
    ///     segment — the one this writer holds open (I-W3-4, N15);
    ///   - a target whose still-live members are not an oldest-contiguous
    ///     prefix of [`Journal::segment_bounds`]'s current segment list
    ///     (I-W3-4, N16).
    ///
    /// An **empty** target is a no-op, `Ok(0)`, never an error. A target
    /// member **already absent** from disk is tolerated rather than refused
    /// — F4's idempotent retry: because deletion below only ever proceeds
    /// oldest-first, a target's still-present members can never have a hole
    /// in the middle relative to what is live now, only a shorter shared
    /// prefix — so this is still exactly as strict about a genuinely wrong
    /// target as an all-or-nothing check would be.
    ///
    /// Unlinks in ascending index order and fsyncs the journal directory
    /// once at the end, so a crash partway leaves a shorter *valid* journal
    /// with a higher floor rather than a hole (F4). The handle's own
    /// `next_seq`, `segment_index`, `segment_file` and `segment_len` all
    /// describe the newest segment and are untouched by construction.
    ///
    /// Returns the number of bytes actually reclaimed (already-absent
    /// members contribute 0).
    pub fn unlink_segments(&mut self, indices: &[u64]) -> Result<u64, JournalError> {
        if indices.is_empty() {
            return Ok(0);
        }
        if indices.contains(&self.segment_index) {
            return Err(JournalError::UnlinkLiveSegment {
                index: self.segment_index,
            });
        }
        let live = list_segments(&self.journal_dir)?;
        let live_indices: Vec<u64> = live.iter().map(|(index, _)| *index).collect();

        // Tolerate members an earlier, interrupted unlink already removed:
        // filter the target down to what is still actually there.
        let present: Vec<u64> = indices
            .iter()
            .copied()
            .filter(|index| live_indices.contains(index))
            .collect();
        if present.is_empty() {
            return Ok(0);
        }
        let n = present.len();
        if n >= live_indices.len() || present != live_indices[..n] {
            return Err(JournalError::UnlinkNotAPrefix {
                attempted: indices.to_vec(),
                live: live_indices,
            });
        }

        let mut reclaimed = 0u64;
        for (_, path) in &live[..n] {
            let bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            match fs::remove_file(path) {
                Ok(()) => reclaimed += bytes,
                // Tolerated: an earlier interrupted cycle already got this
                // one (F4); `present` above already proved it was live a
                // moment ago, so this is the crash window, not corruption.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        sync_dir(&self.journal_dir)?;
        Ok(reclaimed)
    }
}

/// Last-resort group close: a handle that goes away with lines still unsynced
/// syncs them on the way out.
///
/// The daemon never relies on this — [`CoreGuard`](crate::api::CoreGuard)
/// closes every group at the end of its lock hold, and that is where the
/// failure is reportable. This exists so that "a written line is lost only to
/// a crash before its group's sync" stays literally true: without it, dropping
/// a `Journal` mid-group would be a second, silent way to lose one, and the
/// tests and tools that build a journal and drop it would be exercising a
/// weaker durability story than the daemon's.
///
/// Best effort by necessity — `Drop` cannot report. A failure here is exactly
/// as (un)recoverable as a crash one instant earlier, which is the window the
/// caller already reasons about.
impl Drop for Journal {
    fn drop(&mut self) {
        if self.dirty && !self.poisoned {
            let _ = self.segment_file.sync_data();
        }
    }
}

/// One segment's extent, as the startup pass and the cache binding need it
/// (W2, `runtime::startup`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentBound {
    /// Rotation-order index (the segment file's `NNNNNNNN` stem).
    pub index: u64,
    /// Path to the segment file.
    pub path: PathBuf,
    /// First seq in this segment.
    pub first_seq: u64,
    /// Last seq in this segment: the next segment's `first_seq - 1`, or
    /// `next_seq - 1` for the newest.
    pub last_seq: u64,
    /// Segment file length in bytes.
    pub bytes: u64,
}

/// The seq the next append will receive: one past the last event in the last
/// segment that has one.
///
/// Walks segments newest-first to the first non-empty one (rotation can leave
/// an empty newest segment) and replays *only* that segment, with `expected`
/// starting at its own `first_seq` — so seq contiguity is still validated for
/// everything it reads, and the writer can still never assign a colliding
/// seq, because the maximum seq is always in the last non-empty segment
/// (appends are ordered, and rotation creates the new segment before writing
/// to it).
fn tail_next_seq(segments: &[(u64, PathBuf)]) -> Result<u64, JournalError> {
    for (index, path) in segments.iter().rev() {
        let Some(first) = first_seq(path)? else {
            continue;
        };
        let mut next = first;
        for event in Replay::after(vec![(*index, path.clone())], first - 1)? {
            next = event?.seq + 1;
        }
        return Ok(next);
    }
    Ok(1)
}

/// Iterator over committed events across segments, validating seq order.
#[derive(Debug)]
pub struct Replay {
    segments: std::vec::IntoIter<(u64, PathBuf)>,
    current: Option<(String, std::io::Lines<BufReader<File>>)>,
    line_no: u64,
    expected: u64,
    failed: bool,
    /// W3 §11.3 (recon correction 7): events this iterator has yielded so
    /// far. Used for exactly one decision — see [`Replay::next`]'s
    /// `NotFound` handling — a segment that vanishes before it is opened is
    /// tolerated **only** while this is 0.
    yielded: u64,
}

impl Replay {
    fn new(segments: Vec<(u64, PathBuf)>) -> Self {
        Self {
            segments: segments.into_iter(),
            current: None,
            line_no: 0,
            expected: 1,
            failed: false,
            yielded: 0,
        }
    }

    /// Drop the leading segments that cannot hold an event past `after`.
    ///
    /// Segments are indexed by rotation order, not by seq, so the first seq
    /// of each is read off its first line. The kept prefix boundary is the
    /// *last* segment starting at or before `after + 1`: that segment may
    /// still contain wanted events, everything before it provably cannot.
    ///
    /// **A1 (W3 §11.1).** When *no* segment starts at or below `after + 1` —
    /// every surviving segment starts strictly above the requested point,
    /// which after a prune is the ordinary case rather than a gap — this
    /// expects the oldest survivor's own `first_seq` (the floor), not the
    /// hardcoded `1` a fresh [`Replay::new`] carries. On an unpruned journal
    /// the floor is always 1, so nothing here changes for any journal this
    /// build has ever produced by itself; it only matters once a floor above
    /// 1 exists. This is what makes `replay_after(0) ≡ replay_from_floor()`
    /// (see [`Journal::replay_from_floor`]).
    ///
    /// A `NotFound` reading a candidate segment's first line (recon
    /// correction 7) means a lock-free reader raced a prune's unlink during
    /// this very scan — tolerated as "cannot rule anything out", the same
    /// answer an empty rotated segment already gets, rather than propagated
    /// as an error.
    fn after(segments: Vec<(u64, PathBuf)>, after: u64) -> Result<Self, JournalError> {
        let mut keep = 0usize;
        let mut matched: Option<u64> = None;
        let mut floor: Option<u64> = None;
        for (index, (_, path)) in segments.iter().enumerate() {
            let first = match first_seq(path) {
                Ok(first) => first,
                // A segment listed a moment ago but gone now: a prune raced
                // this scan. It cannot rule anything out either way.
                Err(JournalError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e),
            };
            match first {
                // A segment created by rotation but not yet appended to has
                // no first seq to compare; it cannot rule anything out.
                None => continue,
                Some(first) => {
                    if floor.is_none() {
                        floor = Some(first);
                    }
                    if first <= after.saturating_add(1) {
                        keep = index;
                        matched = Some(first);
                    } else {
                        break;
                    }
                }
            }
        }
        let mut segments = segments;
        let mut replay = Self::new(segments.split_off(keep));
        replay.expected = matched.or(floor).unwrap_or(1);
        Ok(replay)
    }
}

/// The seq of a segment's first event, or `None` if it has no events yet.
///
/// Reads one line. A malformed first line is reported the same way a replay
/// would report it rather than being skipped — fail closed.
fn first_seq(path: &Path) -> Result<Option<u64>, JournalError> {
    let file = File::open(path)?;
    let Some(line) = BufReader::new(file).lines().next() else {
        return Ok(None);
    };
    let line = line?;
    let event: Event = serde_json::from_str(&line).map_err(|source| JournalError::Malformed {
        segment: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        line: 1,
        source,
        possible_torn_tail: false,
    })?;
    Ok(Some(event.seq))
}

impl Iterator for Replay {
    type Item = Result<Event, JournalError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        loop {
            if self.current.is_none() {
                let (_, path) = self.segments.next()?;
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let file = match File::open(&path) {
                    Ok(f) => f,
                    // W3 §11.3 / N18 (recon correction 7): a segment that
                    // vanishes before it is opened is a floor that moved
                    // under a lock-free reader, tolerated only while this
                    // iterator is still in the leading prefix (nothing
                    // yielded yet) — I-W3-4 guarantees a prune can only ever
                    // delete from there. Anywhere else, a segment vanishing
                    // mid-stream is not something a prune can do, so it
                    // stays a hard failure.
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound && self.yielded == 0 => {
                        // Re-derive `expected` from whatever segment is next,
                        // so the skip does not itself manufacture a false
                        // `SeqDiscontinuity` against the deleted segment's
                        // old bounds.
                        if let Some((_, next_path)) = self.segments.as_slice().first()
                            && let Ok(Some(first)) = first_seq(next_path)
                        {
                            self.expected = first;
                        }
                        continue;
                    }
                    Err(e) => {
                        self.failed = true;
                        return Some(Err(e.into()));
                    }
                };
                self.current = Some((name, BufReader::new(file).lines()));
                self.line_no = 0;
            }
            let (segment, lines) = self.current.as_mut().expect("current segment set above");
            match lines.next() {
                None => {
                    self.current = None;
                    continue;
                }
                Some(Err(e)) => {
                    self.failed = true;
                    return Some(Err(e.into()));
                }
                Some(Ok(line)) => {
                    self.line_no += 1;
                    // No append path ever writes a blank line (each append is
                    // exactly one JSON object plus one '\n', and tail recovery
                    // truncates to a '\n' boundary), so a blank line can only
                    // mean external mutation or corruption. Fail closed like
                    // any other unexplained content: serde_json rejects the
                    // empty/whitespace input below and it surfaces as
                    // `Malformed` — never a silent skip.
                    let event: Event = match serde_json::from_str(&line) {
                        Ok(ev) => ev,
                        Err(source) => {
                            self.failed = true;
                            // A live writer's `append_event` is a single
                            // unbuffered `write_all` (issue #169): a reader
                            // racing it can observe a torn partial line, but
                            // only ever as the very last line of the very
                            // last segment — rotation always creates a *new*
                            // segment before the writer appends to it, so a
                            // torn write can never leave a well-formed line
                            // after it. Peeking one more line (safe: this
                            // iterator is dead the instant `failed` is set)
                            // tells that apart from real corruption, which
                            // always has something readable following it.
                            let nothing_after = lines.next().is_none();
                            let is_last_segment = self.segments.len() == 0;
                            return Some(Err(JournalError::Malformed {
                                segment: segment.clone(),
                                line: self.line_no,
                                source,
                                possible_torn_tail: nothing_after && is_last_segment,
                            }));
                        }
                    };
                    if event.seq != self.expected {
                        self.failed = true;
                        return Some(Err(JournalError::SeqDiscontinuity {
                            segment: segment.clone(),
                            line: self.line_no,
                            expected: self.expected,
                            found: event.seq,
                        }));
                    }
                    self.expected += 1;
                    self.yielded += 1;
                    return Some(Ok(event));
                }
            }
        }
    }
}

fn segment_file_name(index: u64) -> String {
    format!("{index:08}.ndjson")
}

/// Segment files in the journal dir, sorted ascending by index.
fn list_segments(journal_dir: &Path) -> Result<Vec<(u64, PathBuf)>, JournalError> {
    let mut segments = Vec::new();
    for entry in fs::read_dir(journal_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(stem) = name.strip_suffix(".ndjson") else {
            continue;
        };
        if stem.len() == 8
            && stem.bytes().all(|b| b.is_ascii_digit())
            && let Ok(index) = stem.parse::<u64>()
        {
            segments.push((index, entry.path()));
        }
    }
    segments.sort_unstable_by_key(|(index, _)| *index);
    Ok(segments)
}

fn create_segment(journal_dir: &Path, index: u64) -> Result<(u64, PathBuf), JournalError> {
    if index > MAX_SEGMENT_INDEX {
        // `segment_file_name`'s `{index:08}` is a *minimum* width: a 9-digit
        // index formats fine but `list_segments` (fixed 8-digit stems) would
        // never see it again. Fail closed before any acknowledged append can
        // land in an unreplayable segment.
        return Err(JournalError::SegmentIndexOverflow {
            attempted: index,
            max: MAX_SEGMENT_INDEX,
        });
    }
    let path = journal_dir.join(segment_file_name(index));
    OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(&path)?;
    sync_dir(journal_dir)?;
    Ok((index, path))
}

/// Quarantine a trailing incomplete line (no terminating newline) left by a
/// crash: the partial bytes move to `<segment>.partial` and the segment is
/// truncated back to its last complete line.
fn recover_tail(journal_dir: &Path, index: u64, path: &Path) -> Result<(), JournalError> {
    let bytes = fs::read(path)?;
    if bytes.is_empty() || bytes.ends_with(b"\n") {
        return Ok(());
    }
    let cut = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |p| p + 1);
    let partial_path = journal_dir.join(format!("{}.partial", segment_file_name(index)));
    let mut partial = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&partial_path)?;
    partial.write_all(&bytes[cut..])?;
    partial.write_all(b"\n")?;
    partial.sync_data()?;
    let segment = OpenOptions::new().write(true).open(path)?;
    segment.set_len(cut as u64)?;
    segment.sync_data()?;
    sync_dir(journal_dir)?;
    Ok(())
}

fn sync_dir(dir: &Path) -> std::io::Result<()> {
    File::open(dir)?.sync_all()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::domain::event::EventSource;

    /// Swap a journal's segment handle for an `O_PATH` descriptor the kernel
    /// refuses to `fsync`, so the *next* `sync()` fails while everything
    /// already written stays on disk and every prior append is otherwise
    /// untouched.
    ///
    /// `pub(crate)`, not private to this module, specifically so `api`'s own
    /// group-commit tests can inject the same failure over a `Core` without
    /// duplicating the `O_PATH` trick — and without those tests needing to
    /// call `sync_data` on an unsyncable descriptor directly, which `m6`'s
    /// `t11b_the_append_path_issues_exactly_the_fsync_it_accounts_for` would
    /// correctly flag as a durability syscall outside its accounted blocks.
    /// Living inside this module's own `mod tests` — one of `t11b`'s named
    /// exemptions — is what keeps that scan honest instead of widening it.
    ///
    /// Returns `false` — the caller's cue to skip loudly (`SKIPPED-ENV`)
    /// rather than assert nothing — when this host cannot express the
    /// injection: no `O_PATH` support, or an `fsync` that accepts an
    /// `O_PATH` descriptor anyway.
    #[cfg(target_os = "linux")]
    pub(crate) fn make_unsyncable_for_tests(journal: &mut Journal) -> bool {
        use std::os::unix::fs::OpenOptionsExt;
        /// `O_PATH`, as the kernel numbers it on Linux.
        const O_PATH: i32 = 0o10000000;

        let segment_path = journal
            .journal_dir
            .join(segment_file_name(journal.segment_index));
        let Ok(unsyncable) = OpenOptions::new()
            .read(true)
            .custom_flags(O_PATH)
            .open(&segment_path)
        else {
            return false;
        };
        if unsyncable.sync_data().is_ok() {
            return false;
        }
        journal.segment_file = unsyncable;
        true
    }

    fn draft(n: u64) -> EventDraft {
        EventDraft::new(
            EventSource::new("daemon", "sergeant"),
            "tool.completed",
            serde_json::json!({"n": n}),
        )
    }

    /// Issue #30 items 1-2, in-module (no production seam): swap the private
    /// `segment_file` for a read-only handle on the same path, so the next
    /// append's `write_all` fails at the OS level (EBADF — the fd was never
    /// opened for writing) regardless of the test process's own privileges.
    ///
    /// Item 2 (the torn-append rollback-then-poison handler, lines ~270-274):
    /// the same read-only handle also can't be `set_len`-truncated (EINVAL),
    /// so the failed write's rollback attempt itself fails — and `poisoned`
    /// is set. The rollback also leaves no torn bytes here, because the
    /// write's own permission failure occurs before any byte reaches the OS
    /// buffer — asserted below by comparing the segment's raw bytes before
    /// and after.
    ///
    /// This test pins the arm against *removal* only: with the rollback
    /// deleted and `poisoned = true` set unconditionally, everything below
    /// still holds. That the poison is *conditional*, and that the rollback
    /// is really attempted, is
    /// [`a_failed_write_whose_rollback_succeeds_rolls_the_segment_back_without_poisoning`]'s
    /// job — the two are only meaningful as a pair.
    ///
    /// Item 1 (the poisoned-handle short-circuit, line ~249): a second
    /// append after poisoning must be refused immediately with
    /// `JournalError::Poisoned`, not re-attempt the write.
    #[test]
    fn append_event_poisons_the_handle_when_a_failed_writes_rollback_also_fails() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open(dir.path()).expect("open");

        // A real committed line first, so there is a known-good on-disk
        // state and segment_len to check the "no torn bytes" claim against.
        journal.append(draft(1)).expect("first append must succeed");
        let segment_path = dir
            .path()
            .join("journal")
            .join(segment_file_name(journal.segment_index));
        let bytes_before_failure =
            fs::read(&segment_path).expect("read segment after the healthy append");
        let next_before_failure = journal.next_seq();

        journal.segment_file = OpenOptions::new()
            .read(true)
            .open(&segment_path)
            .expect("reopen the segment read-only");

        let err = journal
            .append(draft(2))
            .expect_err("write_all must fail on a read-only handle");
        assert!(
            matches!(err, JournalError::Io(_)),
            "expected an io error surfaced from the failed write, got {err:?}"
        );
        assert!(
            journal.poisoned,
            "a failed write whose rollback (set_len) also fails must poison the handle"
        );
        assert_eq!(
            journal.next_seq(),
            next_before_failure,
            "a failed append must not advance next_seq"
        );
        let bytes_after_failure =
            fs::read(&segment_path).expect("read segment after the failed append");
        assert_eq!(
            bytes_after_failure, bytes_before_failure,
            "a failed append must never leave torn bytes behind, rollback or not"
        );

        // Item 1: the poison latch holds on every further call, refusing
        // before ever touching the (still) unusable handle.
        let err2 = journal
            .append(draft(3))
            .expect_err("a poisoned handle must refuse further appends");
        assert!(
            matches!(err2, JournalError::Poisoned),
            "expected the poisoned-handle short-circuit, got {err2:?}"
        );
        assert_eq!(
            journal.next_seq(),
            next_before_failure,
            "a refused append must not advance next_seq either"
        );
    }

    /// Issue #30 item 2, the other direction: a failed write whose rollback
    /// **succeeds** must roll the torn bytes off the segment and leave the
    /// handle usable. The poison is the rollback's failure handler, not the
    /// write's.
    ///
    /// The sibling test above cannot see this. Its read-only handle fails
    /// `write_all` *and* `set_len`, so "poisoned because the rollback failed"
    /// and "poisoned on every write failure" produce identical observations;
    /// its no-torn-bytes assertion also holds with the rollback deleted,
    /// because a read-only write fails before a byte reaches the file. Here
    /// both halves discriminate: real torn bytes are on disk before the
    /// failed append, so only an actually-executed `set_len(segment_len)` can
    /// erase them, and only a *conditional* poison leaves the handle usable.
    ///
    /// Getting a writable handle whose writes fail is the whole difficulty.
    /// `O_DIRECT` is the one portable-ish combination: an unaligned
    /// `write_all` fails `EINVAL` while `ftruncate`/`fsync` on the same
    /// descriptor still succeed. **Environment precondition** (a hard
    /// failure, not a silent skip, for the same reason as m3's immutable-bit
    /// fixture): `TMPDIR` on a filesystem that supports `O_DIRECT` — ext4,
    /// xfs and btrfs do; tmpfs does not, and refuses the `open` outright.
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    #[test]
    fn a_failed_write_whose_rollback_succeeds_rolls_the_segment_back_without_poisoning() {
        use std::os::unix::fs::OpenOptionsExt;
        /// `O_DIRECT` as the kernel numbers it on the two architectures this
        /// test is gated to. (`libc` is not a dependency of this crate and
        /// R-S0-10 forbids adding one for a test.)
        const O_DIRECT: i32 = 0o0040000;

        let dir = tempfile::TempDir::new().expect("tempdir");

        // The injection below is only expressible where the kernel/filesystem
        // actually REFUSES unaligned O_DIRECT writes. tmpfs refuses the open
        // outright, and some environments (GitHub Actions' runner filesystem —
        // measured 2026-08-11, CI run 31447702864 on d72c017) accept the
        // unaligned write. Probe first and skip honestly in both cases rather
        // than asserting an environment fact this host does not exhibit.
        {
            use std::io::Write as _;
            let probe_path = dir.path().join("odirect-probe");
            fs::write(&probe_path, b"x").expect("seed the probe file");
            match OpenOptions::new()
                .append(true)
                .custom_flags(O_DIRECT)
                .open(&probe_path)
            {
                Err(_) => {
                    eprintln!(
                        "skipping: this filesystem refuses O_DIRECT open \
                         (tmpfs?); the failure injection is not expressible here"
                    );
                    return;
                }
                Ok(mut probe) => {
                    if probe.write_all(b"unaligned").is_ok() {
                        eprintln!(
                            "skipping: this filesystem accepts unaligned \
                             O_DIRECT writes; the failure injection is not \
                             expressible here"
                        );
                        return;
                    }
                }
            }
        }

        let mut journal = Journal::open(dir.path()).expect("open");
        journal.append(draft(1)).expect("first append must succeed");
        let segment_path = dir
            .path()
            .join("journal")
            .join(segment_file_name(journal.segment_index));
        let healthy_bytes = fs::read(&segment_path).expect("read the healthy segment");
        let healthy_len = journal.segment_len;
        assert_eq!(healthy_len, healthy_bytes.len() as u64);
        let next_before_failure = journal.next_seq();

        // Exactly what a torn append leaves behind: bytes past the journal's
        // recorded length, with no terminating newline. `segment_len` still
        // names the last acknowledged boundary, which is what the rollback
        // truncates back to.
        OpenOptions::new()
            .append(true)
            .open(&segment_path)
            .expect("reopen to tear")
            .write_all(b"{\"seq\":2,\"tor")
            .expect("write the torn fragment");
        assert_ne!(
            fs::read(&segment_path).expect("read torn segment"),
            healthy_bytes,
            "the fixture must really leave a fragment on disk"
        );

        journal.segment_file = OpenOptions::new()
            .append(true)
            .custom_flags(O_DIRECT)
            .open(&segment_path)
            .expect(
                "reopen the segment O_DIRECT — needs a TMPDIR filesystem that \
                 supports it (see the doc comment)",
            );

        let err = journal
            .append(draft(2))
            .expect_err("an unaligned O_DIRECT write_all must fail");
        assert!(
            matches!(err, JournalError::Io(_)),
            "expected an io error surfaced from the failed write, got {err:?}"
        );
        assert!(
            !journal.poisoned,
            "a failed write whose rollback succeeded must NOT poison the \
             handle: the poison is the rollback's failure handler, not the \
             write's"
        );
        assert_eq!(
            fs::read(&segment_path).expect("read segment after the failed append"),
            healthy_bytes,
            "the rollback must have run: set_len(segment_len) is the only \
             thing that can take the torn fragment back off the segment"
        );
        assert_eq!(
            journal.next_seq(),
            next_before_failure,
            "a failed append must not advance next_seq"
        );

        // Un-poisoned means usable: restore an ordinary handle (the O_DIRECT
        // one refuses every append, poisoned or not) and the very seq the
        // failed append was carrying commits normally.
        journal.segment_file = OpenOptions::new()
            .append(true)
            .open(&segment_path)
            .expect("restore an ordinary handle");
        let recovered = journal
            .append(draft(2))
            .expect("a handle that was never poisoned must still accept appends");
        assert_eq!(recovered.seq, next_before_failure);
        let replayed: Vec<u64> = journal
            .replay()
            .expect("replay")
            .map(|e| e.expect("event").seq)
            .collect();
        assert_eq!(
            replayed,
            vec![1, 2],
            "and the segment replays as two clean events, with no trace of \
             the fragment the rollback removed"
        );
    }

    /// Issue #44's primitive, stated as an equation: N appends between two
    /// syncs cost **one** fsync, not N — and the events are all there.
    ///
    /// Mutation that must kill it: restoring the fsync inside `append_event`
    /// (whatever it is spelled) makes `fsync_count()` 5 instead of 1.
    #[test]
    fn a_group_of_appends_costs_exactly_one_fsync() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open(dir.path()).expect("open");

        for n in 1..=5 {
            journal.append(draft(n)).expect("append");
        }
        assert_eq!(
            journal.fsync_count(),
            0,
            "an open group has not been synced yet: the fsync belongs to the \
             group's close, not to any one append"
        );
        assert!(
            journal.is_dirty(),
            "five written lines, none of them synced"
        );

        // Written is visible even before the group closes — every in-process
        // reader replays the same five events.
        let seqs: Vec<u64> = journal
            .replay()
            .expect("replay")
            .map(|e| e.expect("event").seq)
            .collect();
        assert_eq!(seqs, vec![1, 2, 3, 4, 5]);

        journal.sync().expect("group commit");
        assert_eq!(
            journal.fsync_count(),
            1,
            "one group, one fsync — this is the whole of #44"
        );
        assert!(!journal.is_dirty());

        // A second sync with nothing written is free, so a read-only lock
        // hold costs no syscall at all.
        journal.sync().expect("idempotent");
        assert_eq!(journal.fsync_count(), 1);
    }

    /// A rotation is a group boundary: the segment being left behind is
    /// synced before the handle moves on, because `dirty` describes
    /// `segment_file` and that handle is about to be replaced.
    ///
    /// Mutation that must kill it: deleting `self.sync()?` from
    /// `rotate_if_needed` leaves `fsync_count()` at 0 with the first segment's
    /// lines never covered by any fsync.
    #[test]
    fn rotation_closes_the_open_group_on_the_segment_it_leaves() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        // Rotate after the first line, so append #2 crosses the boundary.
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");

        journal.append(draft(1)).expect("first append");
        assert_eq!(journal.segment_index, 1);
        assert_eq!(journal.fsync_count(), 0, "still one open group");

        journal
            .append(draft(2))
            .expect("append across the rotation");
        assert_eq!(journal.segment_index, 2, "the fixture must really rotate");
        assert_eq!(
            journal.fsync_count(),
            1,
            "the group open on segment 1 must be closed by the rotation, not \
             left for a sync that would land on segment 2"
        );

        journal.sync().expect("close the group on segment 2");
        assert_eq!(journal.fsync_count(), 2);
        let seqs: Vec<u64> = journal
            .replay()
            .expect("replay")
            .map(|e| e.expect("event").seq)
            .collect();
        assert_eq!(seqs, vec![1, 2]);
    }

    /// A failed **group** fsync poisons the handle, where a failed *write* is
    /// merely rolled back.
    ///
    /// The asymmetry is the design: at write time nothing has been told about
    /// the line, so undoing it is honest; at group-sync time every event in
    /// the group is already folded into the in-memory projections the daemon's
    /// next decision reads, so undoing it on disk would leave those
    /// projections describing a history the journal does not have. Fail closed
    /// instead.
    ///
    /// Injection: an `O_PATH` descriptor. The kernel refuses `fsync` on one
    /// with `EBADF` while the `Journal` is otherwise intact — and unlike the
    /// read-only-handle trick used above, it does not also break the write, so
    /// the group being poisoned is a group that was genuinely written.
    /// Probe-gated (a host whose `fsync` accepts it cannot express this).
    ///
    /// Mutation that must kill it: dropping `self.poisoned = true` from
    /// `sync`'s error arm lets the next append proceed onto a journal whose
    /// durability just failed.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_failed_group_sync_poisons_the_handle() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open(dir.path()).expect("open");
        journal.append(draft(1)).expect("append");
        journal.append(draft(2)).expect("append");
        let segment_path = dir
            .path()
            .join("journal")
            .join(segment_file_name(journal.segment_index));
        let written = fs::read(&segment_path).expect("read the written segment");

        if !make_unsyncable_for_tests(&mut journal) {
            eprintln!(
                "SKIPPED-ENV: this host cannot express the O_PATH fsync-failure injection \
                 (no O_PATH support, or fsync accepts an O_PATH descriptor here)"
            );
            return;
        }

        let err = journal.sync().expect_err("the group fsync must fail");
        assert!(
            matches!(err, JournalError::Io(_)),
            "expected the io error the failed fsync returned, got {err:?}"
        );
        assert!(
            journal.poisoned,
            "a failed group fsync must fail closed: the group's events are \
             already in the projections and cannot be rolled back"
        );
        assert_eq!(
            fs::read(&segment_path).expect("read segment"),
            written,
            "failing closed means changing nothing on disk, not truncating a \
             group the projections already believe in"
        );

        let err = journal
            .append(draft(3))
            .expect_err("a poisoned handle must refuse further appends");
        assert!(matches!(err, JournalError::Poisoned), "got {err:?}");
        let err = journal.sync().expect_err("and refuse further syncs");
        assert!(matches!(err, JournalError::Poisoned), "got {err:?}");

        // Restore a real handle so `Drop`'s best-effort sync has something
        // valid to close over (the poison latch skips it either way).
        journal.segment_file = OpenOptions::new()
            .append(true)
            .open(&segment_path)
            .expect("restore");
    }

    #[test]
    fn segment_creation_past_the_8_digit_namespace_fails_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        // The last representable index is fine...
        let (index, path) =
            create_segment(dir.path(), MAX_SEGMENT_INDEX).expect("max index segment");
        assert_eq!(index, MAX_SEGMENT_INDEX);
        assert!(path.ends_with("99999999.ndjson"));
        // ...and one past it is refused instead of silently formatting a
        // 9-digit name that list_segments would never see again.
        let err = create_segment(dir.path(), MAX_SEGMENT_INDEX + 1)
            .expect_err("9-digit segment must fail closed");
        assert!(matches!(
            err,
            JournalError::SegmentIndexOverflow {
                attempted: 100_000_000,
                max: MAX_SEGMENT_INDEX,
            }
        ));
        // Nothing 9-digit was created; only the valid segment is listed.
        let segments = list_segments(dir.path()).expect("list");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].0, MAX_SEGMENT_INDEX);
    }

    /// #169's L6 question, answered: a torn line — no trailing newline,
    /// exactly the shape `append_event`'s single unbuffered `write_all` can
    /// leave for a reader racing it — as the last content of the last
    /// segment classifies as a possible torn tail. `replay_data_dir` is the
    /// caller this matters for: unlike `Journal::open`, it deliberately
    /// skips tail recovery, so it is the one path that can actually observe
    /// the race rather than have it silently fixed up first.
    #[test]
    fn a_torn_final_line_is_a_possible_torn_tail() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        {
            let mut journal = Journal::open(dir.path()).expect("open");
            journal.append(draft(1)).expect("append 1");
        }
        let seg = dir.path().join("journal").join(segment_file_name(1));
        let mut f = OpenOptions::new()
            .append(true)
            .open(&seg)
            .expect("open segment");
        // No trailing newline — a write_all caught mid-flight, never
        // anything an append path or tail recovery would leave behind.
        write!(f, r#"{{"schema":"sergeant.event/v1","seq":2"#).expect("write torn line");
        drop(f);

        let results: Vec<_> = Journal::replay_data_dir(dir.path())
            .expect("replay")
            .collect();
        assert_eq!(
            results.len(),
            2,
            "the torn line is still yielded, as an error"
        );
        assert!(results[0].is_ok());
        match &results[1] {
            Err(e @ JournalError::Malformed { line: 2, .. }) => {
                assert!(
                    e.is_possible_torn_tail(),
                    "a torn final line must classify as a possible torn tail: {e:?}"
                );
            }
            other => panic!("expected a Malformed error at line 2, got {other:?}"),
        }
    }

    /// The other half: a malformed line with something well-formed *after*
    /// it is corruption, not a race. `append_event` can never leave a torn
    /// write with more content behind it — rotation always starts a fresh
    /// segment before the writer appends to it — so this shape can only
    /// mean real corruption, and it must never be tolerated or retried.
    /// Reuses the exact fixture `m1_event_core`'s
    /// `seq_gap_or_duplicate_fails_closed` test already builds for
    /// "malformed line mid-journal", classified from the other side.
    #[test]
    fn a_torn_middle_line_is_not_a_possible_torn_tail() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        {
            let mut journal = Journal::open(dir.path()).expect("open");
            journal.append(draft(1)).expect("append 1");
        }
        let seg = dir.path().join("journal").join(segment_file_name(1));
        let valid_line = serde_json::to_string(&draft(2).into_event(2)).expect("serialize");
        let mut f = OpenOptions::new()
            .append(true)
            .open(&seg)
            .expect("open segment");
        // Newline-terminated garbage — not crash-truncated — with a
        // well-formed line after it.
        writeln!(f, "{{this is not an event}}").expect("write garbage line");
        writeln!(f, "{valid_line}").expect("write valid line after garbage");
        drop(f);

        let results: Vec<_> = Journal::replay_data_dir(dir.path())
            .expect("replay")
            .collect();
        assert_eq!(results.len(), 2, "replay stops at the malformed line");
        assert!(results[0].is_ok());
        match &results[1] {
            Err(e @ JournalError::Malformed { line: 2, .. }) => {
                assert!(
                    !e.is_possible_torn_tail(),
                    "a malformed line with something readable after it must never \
                     read as a torn tail: {e:?}"
                );
            }
            other => panic!("expected a Malformed error at line 2, got {other:?}"),
        }
    }

    /// W2 §9.1 step 1: `segment_bounds` over a multi-segment journal reports
    /// each segment's real extent, with the newest bound by the writer's own
    /// `next_seq` and every other bound by the following segment's start.
    #[test]
    fn segment_bounds_reports_each_segments_real_extent() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        // Rotate after every event, so three appends make three segments.
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        for n in 1..=3 {
            journal.append(draft(n)).expect("append");
        }
        let bounds = journal.segment_bounds().expect("segment_bounds");
        assert_eq!(bounds.len(), 3);
        assert_eq!(bounds[0].index, 1);
        assert_eq!(bounds[0].first_seq, 1);
        assert_eq!(bounds[0].last_seq, 1);
        assert_eq!(bounds[1].index, 2);
        assert_eq!(bounds[1].first_seq, 2);
        assert_eq!(bounds[1].last_seq, 2);
        assert_eq!(bounds[2].index, 3);
        assert_eq!(bounds[2].first_seq, 3);
        assert_eq!(
            bounds[2].last_seq, 3,
            "the newest segment's last_seq comes from next_seq() - 1"
        );
        for b in &bounds {
            assert!(b.bytes > 0);
        }
    }

    /// A segment created by rotation but never appended to has no first seq
    /// and must be skipped rather than bounding anything.
    #[test]
    fn segment_bounds_skips_an_empty_newest_segment() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        journal.append(draft(1)).expect("append 1 — segment 1");
        journal
            .append(draft(2))
            .expect("append 2 — rotates to segment 2");
        // Segment 2's length is already over the 1-byte threshold from the
        // append above, so this rotates once more — creating segment 3 with
        // nothing ever written to it, exactly rotation's own empty-tail case.
        journal
            .rotate_if_needed()
            .expect("rotate past the threshold with nothing to write");
        let bounds = journal.segment_bounds().expect("segment_bounds");
        // The empty segment 3 has no first_seq, so it must not appear.
        assert_eq!(
            bounds.iter().map(|b| b.first_seq).collect::<Vec<_>>(),
            [1, 2]
        );
    }

    #[test]
    fn floor_seq_is_none_for_an_empty_journal() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let journal = Journal::open(dir.path()).expect("open");
        assert_eq!(journal.floor_seq().expect("floor_seq"), None);
    }

    #[test]
    fn floor_seq_is_the_oldest_survivors_first_seq() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        for n in 1..=3 {
            journal.append(draft(n)).expect("append");
        }
        assert_eq!(journal.floor_seq().expect("floor_seq"), Some(1));
    }

    /// A1's proof: with the leading segment deleted (never a production
    /// path — floor stays 1 until W3 — but the fallback must already work in
    /// a test `TempDir`), `replay_from_floor` replays from the survivor's own
    /// `first_seq` instead of failing `SeqDiscontinuity`, and plain `replay()`
    /// does fail `SeqDiscontinuity` over the same journal.
    #[test]
    fn replay_from_floor_survives_a_deleted_leading_segment_where_replay_does_not() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let journal_dir;
        {
            let mut journal = Journal::open_with(dir.path(), 1).expect("open");
            for n in 1..=3 {
                journal.append(draft(n)).expect("append");
            }
            journal_dir = dir.path().join("journal");
        }
        // Delete segment 1 (seq 1) — a floor > 1, test-only.
        fs::remove_file(journal_dir.join(segment_file_name(1))).expect("remove segment 1");

        let journal = Journal::open(dir.path()).expect("reopen over the survivors");
        assert_eq!(journal.floor_seq().expect("floor_seq"), Some(2));

        let replayed: Vec<u64> = journal
            .replay_from_floor()
            .expect("replay_from_floor")
            .map(|e| e.expect("event").seq)
            .collect();
        assert_eq!(replayed, vec![2, 3]);

        let err = journal
            .replay()
            .expect("replay")
            .collect::<Result<Vec<_>, _>>()
            .expect_err("plain replay() must fail closed over a gapped chain");
        assert!(
            matches!(
                err,
                JournalError::SeqDiscontinuity {
                    expected: 1,
                    found: 2,
                    ..
                }
            ),
            "expected a SeqDiscontinuity at the gap replay() cannot skip, got {err:?}"
        );
    }

    /// `tail_next_seq` agrees with a full scan on several segments, including
    /// when the newest segment is empty (created by rotation but never
    /// appended to).
    #[test]
    fn tail_next_seq_agrees_with_a_full_scan() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        for n in 1..=4 {
            journal.append(draft(n)).expect("append");
        }
        drop(journal);

        let segments = list_segments(&dir.path().join("journal")).expect("list_segments");
        let full_scan_next = {
            let mut next = 1u64;
            for event in Replay::new(segments.clone()) {
                next = event.expect("event").seq + 1;
            }
            next
        };
        let tail_next = tail_next_seq(&segments).expect("tail_next_seq");
        assert_eq!(tail_next, full_scan_next);
        assert_eq!(tail_next, 5);

        // Re-open (recovers nothing — no torn tail) and confirm the writer's
        // own next_seq agrees too.
        let journal = Journal::open(dir.path()).expect("reopen");
        assert_eq!(journal.next_seq(), 5);
    }

    // -------------------------------------------------------------
    // W3: unlink_segments
    // -------------------------------------------------------------

    /// N15: the segment this writer holds open is never unlinked, even when
    /// named explicitly.
    #[test]
    fn unlink_segments_refuses_the_live_segment() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        for n in 1..=3 {
            journal.append(draft(n)).expect("append");
        }
        assert_eq!(
            journal.segment_index, 3,
            "one segment per append at this threshold"
        );

        let err = journal
            .unlink_segments(&[1, 2, 3])
            .expect_err("the live segment must never be unlinked");
        assert!(matches!(err, JournalError::UnlinkLiveSegment { index: 3 }));
        // Nothing touched.
        assert_eq!(list_segments(&dir.path().join("journal")).unwrap().len(), 3);
    }

    /// N16: a target that skips a segment, or names one out of order, is
    /// refused rather than silently reordered or partially honored.
    #[test]
    fn unlink_segments_refuses_a_non_contiguous_target_set() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        for n in 1..=4 {
            journal.append(draft(n)).expect("append");
        }
        assert_eq!(journal.segment_index, 4);

        // Skips segment 2 — not a prefix at all.
        let err = journal
            .unlink_segments(&[1, 3])
            .expect_err("a non-contiguous target must be refused");
        assert!(matches!(err, JournalError::UnlinkNotAPrefix { .. }));

        // Names segment 2 as if it were the oldest — not oldest-first.
        let err = journal
            .unlink_segments(&[2])
            .expect_err("a target that is not the oldest-contiguous prefix must be refused");
        assert!(matches!(err, JournalError::UnlinkNotAPrefix { .. }));

        // Nothing touched by either refusal.
        assert_eq!(list_segments(&dir.path().join("journal")).unwrap().len(), 4);
    }

    /// The ordinary case: an oldest-contiguous prefix unlinks cleanly, the
    /// writer's own tail is untouched, and the reclaimed byte count matches
    /// what was actually removed.
    #[test]
    fn unlink_segments_removes_an_oldest_contiguous_prefix_and_reports_bytes_reclaimed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        for n in 1..=4 {
            journal.append(draft(n)).expect("append");
        }
        let bounds_before = journal.segment_bounds().expect("bounds");
        let expected_bytes: u64 = bounds_before[..2].iter().map(|b| b.bytes).sum();

        let reclaimed = journal.unlink_segments(&[1, 2]).expect("unlink");
        assert_eq!(reclaimed, expected_bytes);

        let remaining = list_segments(&dir.path().join("journal")).expect("list");
        assert_eq!(
            remaining.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            vec![3, 4]
        );
        // The writer's own tail is untouched: next_seq keeps counting from
        // where it was, and a further append still succeeds.
        assert_eq!(journal.next_seq(), 5);
        journal.append(draft(5)).expect("append after unlink");
        assert_eq!(journal.next_seq(), 6);
    }

    /// An empty target is a no-op — `Ok(0)`, never an error.
    #[test]
    fn unlink_segments_with_an_empty_target_is_a_no_op() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        journal.append(draft(1)).expect("append");
        assert_eq!(journal.unlink_segments(&[]).expect("no-op"), 0);
        assert_eq!(list_segments(&dir.path().join("journal")).unwrap().len(), 1);
    }

    /// F4's idempotent retry: a target whose oldest member was already
    /// unlinked by an earlier, interrupted attempt is tolerated — the
    /// still-present remainder is unlinked, and the already-gone member
    /// contributes nothing to the byte count.
    #[test]
    fn unlink_segments_tolerates_a_target_member_already_removed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        for n in 1..=4 {
            journal.append(draft(n)).expect("append");
        }
        // Simulate an interrupted first attempt at unlinking [1, 2]: segment
        // 1 is already gone, segment 2 is not.
        std::fs::remove_file(dir.path().join("journal").join(segment_file_name(1)))
            .expect("simulate a partially completed unlink");

        let reclaimed = journal
            .unlink_segments(&[1, 2])
            .expect("a retry over a partially-completed unlink must succeed");
        assert!(reclaimed > 0, "segment 2's bytes must still be counted");
        let remaining = list_segments(&dir.path().join("journal")).expect("list");
        assert_eq!(
            remaining.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            vec![3, 4]
        );
    }

    // -------------------------------------------------------------
    // W3: take_rotation_signal
    // -------------------------------------------------------------

    /// One shot: a rotation arms it, reading it clears it, and a second read
    /// with nothing new is `false`.
    #[test]
    fn rotation_signal_is_one_shot() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open_with(dir.path(), 1).expect("open");
        assert!(
            !journal.take_rotation_signal(),
            "a fresh journal has not rotated"
        );

        journal.append(draft(1)).expect("append 1 (segment 1)");
        assert!(
            !journal.take_rotation_signal(),
            "the very first append never rotates — there is nothing to rotate away from"
        );

        journal
            .append(draft(2))
            .expect("append 2 (rotates to segment 2)");
        assert!(
            journal.take_rotation_signal(),
            "the second append must have rotated at this threshold"
        );
        assert!(
            !journal.take_rotation_signal(),
            "reading the signal must clear it"
        );

        journal
            .append(draft(3))
            .expect("append 3 (rotates to segment 3)");
        assert!(
            journal.take_rotation_signal(),
            "a later rotation re-arms it"
        );
    }

    // -------------------------------------------------------------
    // W3 §11.1: Replay::after's floor clamp
    // -------------------------------------------------------------

    /// A1's fix, exercised directly through `replay_after` (not only through
    /// the dedicated `replay_from_floor` wrapper): once the leading segment
    /// is gone, `replay_after(0)` must expect the survivor's own floor, not
    /// the hardcoded `1` that would misreport ruled retention as
    /// `SeqDiscontinuity`.
    #[test]
    fn replay_after_on_a_pruned_journal_expects_the_floor_not_one() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let journal_dir;
        {
            let mut journal = Journal::open_with(dir.path(), 1).expect("open");
            for n in 1..=3 {
                journal.append(draft(n)).expect("append");
            }
            journal_dir = dir.path().join("journal");
        }
        std::fs::remove_file(journal_dir.join(segment_file_name(1)))
            .expect("simulate a prune: remove the leading segment");

        let journal = Journal::open(dir.path()).expect("reopen over the survivors");
        let replayed: Vec<u64> = journal
            .replay_after(0)
            .expect("replay_after")
            .map(|e| e.expect("event — must not be SeqDiscontinuity").seq)
            .collect();
        assert_eq!(
            replayed,
            vec![2, 3],
            "replay_after(0) must equal replay_from_floor() once the floor has moved"
        );
    }

    // -------------------------------------------------------------
    // W3 §11.3 / N18: Replay tolerates a segment pruned between list and open
    // -------------------------------------------------------------

    /// A lock-free reader that lists segments and then has the leading one
    /// vanish before it opens it (a prune racing it) must skip that segment
    /// rather than fail — as long as nothing has been yielded yet.
    #[test]
    fn replay_data_dir_tolerates_a_segment_pruned_between_list_and_open() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        {
            let mut journal = Journal::open_with(dir.path(), 1).expect("open");
            for n in 1..=3 {
                journal.append(draft(n)).expect("append");
            }
        }
        let journal_dir = dir.path().join("journal");
        let segments = list_segments(&journal_dir).expect("list segments (pre-race snapshot)");
        // Simulate the race directly: the caller already has the segment
        // list in hand (as `replay_data_dir` would after its own
        // `list_segments` call), and *then* segment 1 disappears before
        // `Replay` opens it.
        std::fs::remove_file(journal_dir.join(segment_file_name(1)))
            .expect("simulate the race: unlink after listing");

        let replayed: Vec<u64> = Replay::new(segments)
            .map(|e| e.expect("event — must tolerate the raced deletion").seq)
            .collect();
        assert_eq!(replayed, vec![2, 3]);
    }

    /// The other half (F7's boundary): once this iterator has yielded at
    /// least one event, a segment vanishing is no longer a race a prune can
    /// cause (I-W3-4: prunes only ever touch a leading, oldest-contiguous
    /// prefix) — it stays a hard failure.
    #[test]
    fn a_segment_vanishing_after_the_first_event_still_fails_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        {
            let mut journal = Journal::open_with(dir.path(), 1).expect("open");
            for n in 1..=3 {
                journal.append(draft(n)).expect("append");
            }
        }
        let journal_dir = dir.path().join("journal");
        let segments = list_segments(&journal_dir).expect("list segments");

        let mut replay = Replay::new(segments);
        assert_eq!(replay.next().expect("first event").expect("ok").seq, 1);

        // Now remove the *next* segment out from under the still-live
        // iterator — this is not a shape a prune can produce (I-W3-4), so it
        // must fail closed rather than being tolerated.
        std::fs::remove_file(journal_dir.join(segment_file_name(2)))
            .expect("remove the segment the iterator is about to open");

        let err = replay
            .next()
            .expect("an item")
            .expect_err("a mid-stream vanish must fail closed");
        assert!(matches!(err, JournalError::Io(_)));
    }
}

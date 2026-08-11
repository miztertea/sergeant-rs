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
    /// Open recovers a crashed tail, then fully replays the journal to
    /// validate seq continuity and learn the next seq — a corrupt or gapped
    /// journal refuses to open rather than silently accepting new writes.
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

        let mut next_seq = 1u64;
        for event in Replay::new(segments.clone()) {
            next_seq = event?.seq + 1;
        }

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
    pub fn replay_data_dir(data_dir: impl AsRef<Path>) -> Result<Replay, JournalError> {
        Ok(Replay::new(list_segments(
            &data_dir.as_ref().join("journal"),
        )?))
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
        Ok(())
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

/// Iterator over committed events across segments, validating seq order.
#[derive(Debug)]
pub struct Replay {
    segments: std::vec::IntoIter<(u64, PathBuf)>,
    current: Option<(String, std::io::Lines<BufReader<File>>)>,
    line_no: u64,
    expected: u64,
    failed: bool,
}

impl Replay {
    fn new(segments: Vec<(u64, PathBuf)>) -> Self {
        Self {
            segments: segments.into_iter(),
            current: None,
            line_no: 0,
            expected: 1,
            failed: false,
        }
    }

    /// Drop the leading segments that cannot hold an event past `after`.
    ///
    /// Segments are indexed by rotation order, not by seq, so the first seq
    /// of each is read off its first line. The kept prefix boundary is the
    /// *last* segment starting at or before `after + 1`: that segment may
    /// still contain wanted events, everything before it provably cannot.
    fn after(segments: Vec<(u64, PathBuf)>, after: u64) -> Result<Self, JournalError> {
        let mut keep = 0usize;
        let mut expected = 1u64;
        for (index, (_, path)) in segments.iter().enumerate() {
            match first_seq(path)? {
                // A segment created by rotation but not yet appended to has
                // no first seq to compare; it cannot rule anything out.
                None => continue,
                Some(first) if first <= after.saturating_add(1) => {
                    keep = index;
                    expected = first;
                }
                Some(_) => break,
            }
        }
        let mut segments = segments;
        let mut replay = Self::new(segments.split_off(keep));
        replay.expected = expected;
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
                            return Some(Err(JournalError::Malformed {
                                segment: segment.clone(),
                                line: self.line_no,
                                source,
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
mod tests {
    use super::*;
    use crate::domain::event::EventSource;

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
        use std::os::unix::fs::OpenOptionsExt;
        /// `O_PATH`, as the kernel numbers it on Linux.
        const O_PATH: i32 = 0o10000000;

        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut journal = Journal::open(dir.path()).expect("open");
        journal.append(draft(1)).expect("append");
        journal.append(draft(2)).expect("append");
        let segment_path = dir
            .path()
            .join("journal")
            .join(segment_file_name(journal.segment_index));
        let written = fs::read(&segment_path).expect("read the written segment");

        let Ok(unsyncable) = OpenOptions::new()
            .read(true)
            .custom_flags(O_PATH)
            .open(&segment_path)
        else {
            eprintln!("SKIPPED-ENV: this host refuses to open an O_PATH descriptor");
            return;
        };
        if unsyncable.sync_data().is_ok() {
            eprintln!("SKIPPED-ENV: this host's fsync accepts an O_PATH descriptor");
            return;
        }
        journal.segment_file = unsyncable;

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
}

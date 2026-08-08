//! Append-only segmented NDJSON event journal (proposal §21).
//!
//! Layout under a caller-supplied data dir:
//!
//! ```text
//! <data-dir>/journal/00000001.ndjson
//! <data-dir>/journal/00000002.ndjson
//! ```
//!
//! Single writer, one complete event per line, fsync before an append is
//! acknowledged, size-based segment rotation. Single-writer is enforced per
//! journal directory with an exclusive advisory lock (`journal/.lock`), not
//! just per handle — opening a second writer on a live journal fails with
//! [`JournalError::Locked`]. A trailing incomplete line left
//! by a crash is quarantined to `<segment>.partial` on open and the segment is
//! truncated back to its last complete line; no complete line is ever lost.
//! Replay yields all events in seq order across segments and fails closed on
//! a gap or duplicate seq.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::domain::event::{Event, EventDraft};
use crate::runtime::fsutil::create_dir_all_durable;

/// Default segment rotation threshold.
pub const DEFAULT_SEGMENT_MAX_BYTES: u64 = 8 * 1024 * 1024;

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
#[derive(Debug)]
pub struct Journal {
    journal_dir: PathBuf,
    segment_max_bytes: u64,
    segment_index: u64,
    segment_file: File,
    segment_len: u64,
    next_seq: u64,
    fsync_count: u64,
    poisoned: bool,
    /// Exclusive advisory lock on the journal dir, held for the lifetime of
    /// the handle. The OS releases it when the handle drops — including on
    /// crash — so a stale lock can never wedge reopen.
    _lock: File,
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
        let lock = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(journal_dir.join(LOCK_FILE_NAME))?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => return Err(JournalError::Locked),
            Err(TryLockError::Error(e)) => return Err(e.into()),
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
            poisoned: false,
            _lock: lock,
        })
    }

    /// The seq the next append will receive.
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// Number of segment-data fsyncs issued for acknowledged appends. Counts
    /// only the per-append `sync_data`, not directory syncs during rotation,
    /// and increments only from the syscall's success value.
    ///
    /// Durability of fsync on the host filesystem is unverifiable from inside
    /// the process; this counter lets tests assert that our fsync calls
    /// happen on every acknowledged append (exactly one per append).
    pub fn fsync_count(&self) -> u64 {
        self.fsync_count
    }

    /// Append a draft: assigns the next seq, stamps id/timestamp, writes one
    /// NDJSON line, and fsyncs before returning the committed event.
    pub fn append(&mut self, draft: EventDraft) -> Result<Event, JournalError> {
        let event = draft.into_event(self.next_seq);
        self.append_event(&event)?;
        Ok(event)
    }

    /// Append a fully-formed event. The event's seq must be exactly the next
    /// seq; anything else (regression, duplicate, skip) is rejected.
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
        if let Err(err) = self.write_and_sync(&line) {
            // A failed write_all/sync_data can leave torn, un-terminated bytes
            // in the segment while the handle stays otherwise usable; a later
            // acknowledged append would then concatenate onto the fragment and
            // become unreplayable. Roll the segment back to its pre-append
            // length so that can never happen; if the rollback itself fails,
            // poison the handle so every further append is refused until the
            // journal is reopened (which recovers and re-validates).
            if self.segment_file.set_len(self.segment_len).is_err()
                || self.segment_file.sync_data().is_err()
            {
                self.poisoned = true;
            }
            return Err(err.into());
        }
        self.segment_len += line.len() as u64;
        self.next_seq += 1;
        Ok(())
    }

    /// Write one line and fsync it. The fsync counter is derived from the
    /// syscall's success value in a single expression, so it cannot advance
    /// without `sync_data` actually returning `Ok`.
    fn write_and_sync(&mut self, line: &[u8]) -> std::io::Result<()> {
        self.segment_file.write_all(line)?;
        self.fsync_count += self.segment_file.sync_data().map(|()| 1)?;
        Ok(())
    }

    /// Iterate every committed event in seq order across all segments.
    /// Yields an error (and then stops) on malformed lines or seq
    /// discontinuities — fail closed, never silently skip.
    pub fn replay(&self) -> Result<Replay, JournalError> {
        Ok(Replay::new(list_segments(&self.journal_dir)?))
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
        let (index, path) = create_segment(&self.journal_dir, self.segment_index + 1)?;
        self.segment_file = OpenOptions::new().append(true).open(&path)?;
        self.segment_index = index;
        self.segment_len = 0;
        // create_segment fsyncs the directory; deliberately not counted in
        // fsync_count, which tracks only per-append segment-data syncs.
        Ok(())
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

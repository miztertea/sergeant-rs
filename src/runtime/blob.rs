//! Content-addressed blob store (proposal §21 "Blob Store").
//!
//! Large evidence lives at `<data-dir>/blobs/b3/<blake3-hex>` and events
//! carry `"b3:<hex>"` reference strings. Writes are write-once: identical
//! content deduplicates to one file, and an existing blob is never rewritten.
//! Reads re-hash the bytes and fail closed on mismatch.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde_json::Value;

use crate::domain::event::Event;
use crate::runtime::fsutil::{create_dir_all_durable, write_atomic};

/// Chunk size [`BlobStore::put_stream`] reads/hashes/writes at a time. Fixed
/// and small on purpose (§16.9/§22.8): this is the whole reason a streaming
/// writer exists instead of `put(&buffer_it_all_up_first)` — the RSS this
/// path costs is bounded by this constant, not by the content length.
const STREAM_CHUNK: usize = 64 * 1024;

/// Errors from the blob store.
#[derive(Debug, thiserror::Error)]
pub enum BlobError {
    /// Underlying filesystem failure.
    #[error("blob io error: {0}")]
    Io(#[from] std::io::Error),
    /// The requested blob does not exist.
    #[error("blob not found: {0}")]
    NotFound(BlobRef),
    /// Stored bytes do not hash to the requested reference. Fail closed.
    #[error("blob hash mismatch: expected {expected}, stored bytes hash to b3:{actual}")]
    HashMismatch {
        /// The reference that was requested.
        expected: BlobRef,
        /// Hex hash of the bytes actually on disk.
        actual: String,
    },
    /// A reference string was not of the form `b3:<64 lowercase hex chars>`.
    #[error("invalid blob ref: {0:?}")]
    InvalidRef(String),
}

/// A validated `b3:<blake3-hex>` content reference.
// `PartialOrd, Ord` (A4): `BTreeSet<BlobRef>` is `refs_in_payload`/
// `refs_in_event`'s return type, so the mark-and-sweep scan's result is a
// stable, deduplicated, orderable set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlobRef {
    hex: String,
}

impl BlobRef {
    /// The 64-char lowercase hex digest (without the `b3:` prefix).
    pub fn hex(&self) -> &str {
        &self.hex
    }
}

impl fmt::Display for BlobRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b3:{}", self.hex)
    }
}

impl FromStr for BlobRef {
    type Err = BlobError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s
            .strip_prefix("b3:")
            .ok_or_else(|| BlobError::InvalidRef(s.to_string()))?;
        if hex.len() != 64
            || !hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(BlobError::InvalidRef(s.to_string()));
        }
        Ok(Self {
            hex: hex.to_string(),
        })
    }
}

/// Handle to the content-addressed store under `<data-dir>/blobs/b3`.
#[derive(Debug)]
pub struct BlobStore {
    root: PathBuf,
}

impl BlobStore {
    /// Open (creating if needed) the store under `<data_dir>/blobs/b3`.
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self, BlobError> {
        let root = data_dir.as_ref().join("blobs").join("b3");
        // Durable creation: fresh `blobs/` and `blobs/b3/` have their dirents
        // fsynced in their parents, so the first blob written here cannot
        // vanish with its directories after a crash (`write_atomic` itself
        // only syncs the immediate parent).
        create_dir_all_durable(&root)?;
        Ok(Self { root })
    }

    /// Store bytes and return their content reference. Idempotent: identical
    /// content maps to one file, and an existing blob is left untouched. The
    /// write lands atomically (tmp + rename + dir fsync, via
    /// [`write_atomic`]), so a crashed put can never leave a partial blob at
    /// its content address.
    pub fn put(&self, bytes: &[u8]) -> Result<BlobRef, BlobError> {
        let hex = blake3::hash(bytes).to_hex().to_string();
        let path = self.root.join(&hex);
        let blob_ref = BlobRef { hex };
        if path.exists() {
            return Ok(blob_ref); // write-once: never rewrite existing content
        }
        write_atomic(&path, bytes)?;
        Ok(blob_ref)
    }

    /// Store the bytes read from `reader` and return their content reference
    /// plus the total byte count, **without ever holding the whole content in
    /// memory** (§16.9's streaming-capture requirement, §22.8's bounded-RSS
    /// acceptance test).
    ///
    /// The recipe is [`write_atomic`]'s — tmp file next to the final address,
    /// fsync, rename, directory fsync — carried over a bounded read/hash/write
    /// loop instead of one `write_all` over a fully-buffered slice: content is
    /// read in [`STREAM_CHUNK`]-sized pieces, each piece is folded into a
    /// running BLAKE3 hash and written to the tmp file before the next piece
    /// is read, so peak memory for this call is `O(STREAM_CHUNK)` regardless
    /// of how large `reader` turns out to be. A write-once store still
    /// deduplicates: if the finished hash already has a blob on disk, the tmp
    /// file is discarded rather than replacing content that verifiably
    /// matches already.
    ///
    /// A failure partway through (a disk-full `write_all`, an unreadable
    /// source) leaves no partial blob at any content address — the tmp file
    /// is removed on every error path — and the caller sees the underlying
    /// I/O error rather than a corrupt or truncated blob silently claiming a
    /// hash it does not have.
    pub fn put_stream(&self, mut reader: impl Read) -> Result<(BlobRef, u64), BlobError> {
        let tmp = self.root.join(format!("tmp-{}", ulid::Ulid::generate()));
        let outcome = (|| -> Result<(BlobRef, u64), BlobError> {
            let mut file = fs::File::create(&tmp)?;
            let mut hasher = blake3::Hasher::new();
            let mut buf = [0u8; STREAM_CHUNK];
            let mut total: u64 = 0;
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
                file.write_all(&buf[..n])?;
                total += n as u64;
            }
            file.sync_data()?;
            drop(file);
            let hex = hasher.finalize().to_hex().to_string();
            let path = self.root.join(&hex);
            let blob_ref = BlobRef { hex };
            if path.exists() {
                return Ok((blob_ref, total)); // write-once: identical content dedupes
            }
            fs::rename(&tmp, &path)?;
            if let Some(parent) = path.parent() {
                fs::File::open(parent)?.sync_all()?;
            }
            Ok((blob_ref, total))
        })();
        // The rename above (success, new content) already moved the tmp file
        // away; every other exit path — the error branch and the
        // already-deduplicated branch — still has it sitting in `root` and
        // must not leave it behind.
        if tmp.exists() {
            let _ = fs::remove_file(&tmp);
        }
        outcome
    }

    /// Fetch a blob's bytes, verifying they still hash to the reference.
    pub fn get(&self, blob_ref: &BlobRef) -> Result<Vec<u8>, BlobError> {
        let path = self.root.join(blob_ref.hex());
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(BlobError::NotFound(blob_ref.clone()));
            }
            Err(e) => return Err(e.into()),
        };
        let actual = blake3::hash(&bytes).to_hex().to_string();
        if actual != blob_ref.hex() {
            return Err(BlobError::HashMismatch {
                expected: blob_ref.clone(),
                actual,
            });
        }
        Ok(bytes)
    }
}

/// How deep the recursive walk follows nested JSON-in-string values.
///
/// Bounded because a string that parses as JSON can itself contain a string
/// that parses as JSON: the depth is a property of the producer, and every
/// real producer today is one level (docker's `detail` object, stringified
/// into `stage.completed`'s `detail` field). 32 is far past anything that
/// could be honest and far short of a stack problem.
const MAX_SCAN_DEPTH: usize = 32;

/// Every blob reference reachable from an event payload — A4's recursive
/// walk.
///
/// A flat scan of top-level payload fields is **wrong**: docker's capture
/// puts its `stdout`/`stderr` refs inside a `detail` object that is then
/// *stringified* into `stage.completed`'s `detail` field, so a flat scan
/// finds nothing there and a mark-and-sweep built on one would delete live
/// blobs.
///
/// Rules:
///   - objects and arrays: recurse into every value;
///   - strings: if the string parses as a [`BlobRef`] (`^b3:[0-9a-f]{64}$` —
///     [`BlobRef::from_str`], never a second regex), collect it; otherwise,
///     if it parses as a JSON **object or array**, recurse into that;
///   - numbers, booleans, nulls: ignored;
///   - depth is capped at [`MAX_SCAN_DEPTH`].
///
/// Restricting the string-recursion arm to objects and arrays is deliberate:
/// a string that parses to a bare JSON *string* would recurse on a shorter
/// string forever-ish, and no producer emits a doubly-encoded bare ref.
pub fn refs_in_payload(value: &Value, out: &mut BTreeSet<BlobRef>) {
    refs_in_payload_at_depth(value, out, 0);
}

fn refs_in_payload_at_depth(value: &Value, out: &mut BTreeSet<BlobRef>, depth: usize) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    match value {
        Value::Object(map) => {
            for v in map.values() {
                refs_in_payload_at_depth(v, out, depth + 1);
            }
        }
        Value::Array(items) => {
            for v in items {
                refs_in_payload_at_depth(v, out, depth + 1);
            }
        }
        Value::String(s) => {
            if let Ok(blob_ref) = BlobRef::from_str(s) {
                out.insert(blob_ref);
            } else if let Ok(parsed @ (Value::Object(_) | Value::Array(_))) =
                serde_json::from_str::<Value>(s)
            {
                refs_in_payload_at_depth(&parsed, out, depth + 1);
            }
        }
        Value::Number(_) | Value::Bool(_) | Value::Null => {}
    }
}

/// Every blob reference in one event's payload.
pub fn refs_in_event(event: &Event) -> BTreeSet<BlobRef> {
    let mut out = BTreeSet::new();
    refs_in_payload(&event.payload, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// `put_stream` and `put` must land on the same content address for the
    /// same bytes, and `get` must recover exactly what was streamed in — the
    /// streaming path is a different writer for the same store, not a
    /// parallel one with its own notion of identity.
    #[test]
    fn put_stream_matches_put_and_is_readable_back() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let store = BlobStore::open(dir.path()).expect("open store");

        let bytes = b"hello streamed world".repeat(10_000);
        let buffered = store.put(&bytes).expect("buffered put");
        let (streamed, total) = store
            .put_stream(std::io::Cursor::new(bytes.clone()))
            .expect("streamed put");

        assert_eq!(streamed.hex(), buffered.hex());
        assert_eq!(total, bytes.len() as u64);
        assert_eq!(store.get(&streamed).expect("read back"), bytes);
    }

    /// §22.8's whole point: `put_stream` must not buffer its input. This
    /// pushes content well past any size a `Vec<u8>` in this test process
    /// could plausibly allocate by accident, from a source that streams
    /// (never materializes a matching buffer of its own) — a regression to
    /// `put_stream` reading `mut Read` into one owned `Vec` before hashing
    /// would still pass functionally but would blow this test's own memory
    /// budget, which the harness bounds externally in the RSS acceptance
    /// test (`tests/m7_docker_executor.rs`). Here we pin the cheaper,
    /// always-on half: content larger than any single allocation this test
    /// makes is still hashed and stored correctly, and streamed twice
    /// (dedup path) without error.
    #[test]
    fn put_stream_handles_content_larger_than_any_single_buffer_here() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let store = BlobStore::open(dir.path()).expect("open store");

        /// A `Read` source that manufactures zero bytes on demand — this
        /// process never allocates a buffer holding the full content, only
        /// [`STREAM_CHUNK`]-sized pieces at a time, mirroring what a `docker
        /// logs` pipe read loop actually does.
        struct Zeros {
            remaining: u64,
        }
        impl Read for Zeros {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let n = (buf.len() as u64).min(self.remaining) as usize;
                buf[..n].fill(0);
                self.remaining -= n as u64;
                Ok(n)
            }
        }

        let size: u64 = 8 * 1024 * 1024; // 8 MiB: exceeds this test's own buffers many times over
        let (first, total) = store
            .put_stream(Zeros { remaining: size })
            .expect("first stream");
        assert_eq!(total, size);
        // Streamed again: same content, same address, dedup path taken.
        let (second, total2) = store
            .put_stream(Zeros { remaining: size })
            .expect("second stream");
        assert_eq!(second.hex(), first.hex());
        assert_eq!(total2, size);
        assert_eq!(store.get(&first).expect("read back").len(), size as usize);
    }

    /// A read failure partway through must not leave a tmp file behind, and
    /// must not fabricate a blob at any content address.
    #[test]
    fn put_stream_leaves_no_tmp_litter_on_a_read_failure() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let store = BlobStore::open(dir.path()).expect("open store");

        struct FailsAfter {
            remaining: usize,
        }
        impl Read for FailsAfter {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if self.remaining == 0 {
                    return Err(std::io::Error::other("simulated read failure"));
                }
                let n = buf.len().min(self.remaining);
                buf[..n].fill(1);
                self.remaining -= n;
                Ok(n)
            }
        }

        let err = store
            .put_stream(FailsAfter { remaining: 4 })
            .expect_err("a read failure must surface, not silently truncate");
        assert!(matches!(err, BlobError::Io(_)));

        let entries: Vec<_> = fs::read_dir(&store.root)
            .expect("read blob dir")
            .map(|e| e.expect("entry").file_name())
            .collect();
        assert!(
            entries.is_empty(),
            "a failed stream must leave no tmp file behind, found {entries:?}"
        );
    }

    /// Issue #31 item 1: `BlobStore::get`'s generic io-error branch. The
    /// existing coverage only ever exercises the happy path and the
    /// `NotFound` branch; this is a non-`NotFound` failure where the content
    /// address exists but isn't a regular file. A dir-as-file trick (rather
    /// than a permission trick) is used deliberately: this suite can run as
    /// root, where directory-permission bits are not enforced, but "the path
    /// is a directory, not a file" is a type mismatch `read()` refuses
    /// unconditionally — verified empirically (`fs::read` on a directory
    /// fails with `ErrorKind::IsADirectory`) — so it stays a reliable probe
    /// regardless of the runner's privileges.
    #[test]
    fn get_surfaces_a_non_not_found_io_error_when_the_blob_path_is_a_directory() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let store = BlobStore::open(dir.path()).expect("open store");

        let hex = "a".repeat(64);
        let blob_ref: BlobRef = format!("b3:{hex}").parse().expect("valid ref");
        // Put a directory where the blob's content-addressed file would be,
        // instead of ever calling `put` — this is deliberately not the
        // NotFound case (something exists at the path) and not a hash
        // mismatch (nothing was ever readable as bytes).
        fs::create_dir_all(store.root.join(&hex)).expect("place a directory at the blob path");

        let err = store
            .get(&blob_ref)
            .expect_err("a directory is not a readable blob");
        assert!(
            !matches!(err, BlobError::NotFound(_)),
            "a path that exists (as a directory) must not be reported as missing: {err:?}"
        );
        assert!(
            matches!(err, BlobError::Io(_)),
            "expected the generic io-error branch, got {err:?}"
        );
    }

    fn hex_ref(byte: u8) -> String {
        let mut hex = String::with_capacity(64);
        for _ in 0..32 {
            hex.push_str(&format!("{byte:02x}"));
        }
        format!("b3:{hex}")
    }

    /// A flat top-level scan of a docker-shaped payload finds nothing; the
    /// nested walk recovers both refs from inside the stringified `detail`
    /// object — A4's claim, proved structurally here (the fixture-driven
    /// version lives in `tests/a4_blob_ref_pinning.rs`).
    #[test]
    fn refs_in_payload_recovers_a_nested_json_string_ref() {
        let stdout_ref = hex_ref(0x11);
        let stderr_ref = hex_ref(0x22);
        let detail = json!({"stdout": stdout_ref, "stderr": stderr_ref}).to_string();
        let payload = json!({"stage_id": "build", "detail": detail});

        let mut out = BTreeSet::new();
        refs_in_payload(&payload, &mut out);
        assert_eq!(out.len(), 2);
        assert!(out.contains(&BlobRef::from_str(&stdout_ref).unwrap()));
        assert!(out.contains(&BlobRef::from_str(&stderr_ref).unwrap()));

        // The load-bearing negative half: a flat scan of the *top-level*
        // string fields alone finds nothing, because `detail` parses to an
        // object, not a bare ref.
        assert!(BlobRef::from_str(payload["detail"].as_str().unwrap()).is_err());
    }

    /// A top-level ref (claude's shape: `raw` is a bare `"b3:…"` string) is
    /// found with no nesting at all.
    #[test]
    fn refs_in_payload_recovers_a_top_level_ref() {
        let raw_ref = hex_ref(0x33);
        let payload = json!({"raw": raw_ref});
        let mut out = BTreeSet::new();
        refs_in_payload(&payload, &mut out);
        assert_eq!(out, BTreeSet::from([BlobRef::from_str(&raw_ref).unwrap()]));
    }

    /// The depth cap stops the walk rather than overflowing the stack on a
    /// pathologically deep payload.
    ///
    /// Nests via plain array wrapping, not repeated `to_string()` —
    /// re-stringifying a value that already contains a quoted string
    /// multiplies its escaped-backslash count at every level (`"` becomes
    /// `\"`, which becomes `\\\"`, ...), so a naive stringify-loop this deep
    /// blows up exponentially rather than linearly. Array nesting costs one
    /// `Value` per level and exercises the same depth-counting code path
    /// (`refs_in_payload_at_depth` recurses into arrays exactly like
    /// objects).
    #[test]
    fn refs_in_payload_is_bounded_by_max_scan_depth() {
        let leaf_ref = hex_ref(0x44);
        let mut value = json!({"r": leaf_ref});
        for _ in 0..(MAX_SCAN_DEPTH + 5) {
            value = json!([value]);
        }
        let payload = json!({"nested": value});
        let mut out = BTreeSet::new();
        refs_in_payload(&payload, &mut out);
        assert!(
            out.is_empty(),
            "a ref nested past MAX_SCAN_DEPTH must not be recovered: {out:?}"
        );
    }

    /// Strings that look close to a ref but are not one must never be
    /// collected: too short, uppercase hex, and non-hex characters are all
    /// rejected by `BlobRef::from_str`'s own rule, which this extractor
    /// reuses rather than a second regex.
    #[test]
    fn refs_in_payload_rejects_near_miss_strings() {
        let too_short = format!("b3:{}", "a".repeat(63));
        let uppercase = format!("b3:{}", "A".repeat(64));
        let non_hex = format!("b3:{}", "g".repeat(64));
        let payload = json!({
            "too_short": too_short,
            "uppercase": uppercase,
            "non_hex": non_hex,
        });
        let mut out = BTreeSet::new();
        refs_in_payload(&payload, &mut out);
        assert!(
            out.is_empty(),
            "near-miss strings must never be collected as refs: {out:?}"
        );
    }

    /// `refs_in_event` is the event-level entry point A4's pinning tests use.
    #[test]
    fn refs_in_event_reads_the_payload() {
        use crate::domain::event::{EventDraft, EventSource};

        let want = hex_ref(0x55);
        let event = EventDraft::new(
            EventSource::new("backend", "claude"),
            "conversation.turn.ended",
            json!({"raw": want}),
        )
        .into_event(1);
        let refs = refs_in_event(&event);
        assert_eq!(refs, BTreeSet::from([BlobRef::from_str(&want).unwrap()]));
    }
}

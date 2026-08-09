//! Content-addressed blob store (proposal §21 "Blob Store").
//!
//! Large evidence lives at `<data-dir>/blobs/b3/<blake3-hex>` and events
//! carry `"b3:<hex>"` reference strings. Writes are write-once: identical
//! content deduplicates to one file, and an existing blob is never rewritten.
//! Reads re-hash the bytes and fail closed on mismatch.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::runtime::fsutil::{create_dir_all_durable, write_atomic};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

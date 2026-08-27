//! Multi-byte bytes before and inside extracted names — hand-counted so the
//! provenance test slices at non-trivial UTF-8 offsets.
//!
//! 日本語のコメント — three-byte characters and an em dash, both sitting ahead
//! of every symbol below, so a byte offset that was really a character count
//! would land mid-character and fail the slice rather than pass by luck.

/// Führt eine Zählung durch — Ümlaute im Doc-Kommentar.
pub struct Zähler {
    treffer: usize,
}

pub fn café_fn(zähler: &Zähler) -> usize {
    zähler.treffer
}

pub const GRÜSSE: &str = "Grüße, 世界";

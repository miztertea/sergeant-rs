//! Bounded ZIP container expansion (S4 Y3, G5 as AMENDED 2026-08-28).
//!
//! ```text
//! bytes  ──►  ZipArchive::new  ──►  admission + bounds  ──►  ZipChild (our own vocabulary)
//! ```
//!
//! A pure function over bytes (F6's adapter-shape mandate, the same shape
//! [`super::text`]/[`super::office`] already keep), meant to run inside Y1's
//! supervised worker exactly the way `office::docx_units` does. Named
//! `archive` rather than `zip` deliberately: the crate this module is the one
//! place allowed to name (`zip`, per its own `Cargo.toml` entry) would
//! otherwise shadow this module's own path — a module is named after the
//! domain concept it owns, not after the vendor crate behind it, the same
//! naming discipline [`super::office`]'s own module keeps for its own
//! third-party dependency.
//!
//! # What `enclosed_name` buys, and what this module adds on top of it (the
//! amendment's own words)
//!
//! `zip`'s `enclosed_name()` is a real, correct path-STRING validator — NUL
//! bytes, absolute paths, Windows drive letters, UNC prefixes and `..`
//! traversal are all genuinely rejected (`y3-zip-bounds-research.md` §1,
//! VERIFIED against the crate's own `src/path.rs`). It says **nothing** about
//! entry TYPE, name uniqueness, or cross-platform name collision. This
//! module's [`expand`] adds exactly those checks, each its own named
//! [`CoverageRow`], layered on top rather than in place of it:
//!
//! 1. **Entry type** — [`zip::read::ZipFile::is_symlink`]/`is_file` (both
//!    read the entry's Unix mode off the central-directory external
//!    attributes, VERIFIED directly against `zip` 8.6.0's own
//!    `src/read.rs`): a symlink or anything that is neither a regular file
//!    nor a directory is refused. A symlink's "content" is its link-target
//!    TEXT, not file bytes — treating it as a symlink and creating one is a
//!    traversal primitive by another route (the amendment's own wording),
//!    and this module never creates a filesystem symlink at all (deliverable
//!    (d): nothing here writes to a real path, symlink or otherwise).
//! 2. **Non-empty name** — `enclosed_name()` returns `Some(PathBuf::new())`
//!    for the literal empty string (VERIFIED, research §1); an empty
//!    enclosed path is refused rather than silently keyed against the
//!    parent's own coordinate.
//! 3. **Name uniqueness** — this module walks `0..archive.len()` **by
//!    index**, never `by_name`. The first entry at a given enclosed path is
//!    admitted; a later entry sharing the identical enclosed path (from a
//!    DIFFERENT raw central-directory name that normalises the same way —
//!    e.g. `b.txt` and `./b.txt`, whose `enclosed_name()` outputs both
//!    collapse to `b.txt`) is refused with its own named row. **A
//!    correction to the research note, verified while building this
//!    module's own test fixtures, not merely read from source**: the
//!    research claimed `by_index` "still visits every entry, duplicates
//!    included" for two central-directory records with BYTE-IDENTICAL raw
//!    names. That is not what `zip` 8.6.0 actually does. `ZipArchive`'s own
//!    central-directory records are folded into an `IndexMap<Box<[u8]>,
//!    ZipFileData>` KEYED ON THE RAW NAME BYTES BEFORE `ZipArchive` ever
//!    exists (`SharedBuilder::build`, VERIFIED directly against the crate's
//!    own source): `index_map.insert(file.file_name_raw.clone(), file)`
//!    for every record, in file order. `IndexMap::insert` on an
//!    already-present key overwrites the VALUE in place at the SAME index
//!    — it does not append a second entry. So two records sharing
//!    byte-identical raw names collapse to exactly ONE archive entry before
//!    `archive.len()`/`by_index` are ever called: `len()` reports the
//!    DEDUPLICATED count, and the surviving entry is the LAST one written,
//!    silently — the first one's bytes are not merely hard to reach by
//!    name, as the research claimed; they are **unreachable through any
//!    public API this module has**, full stop. Proven directly:
//!    [`identical_raw_names_collapse_silently_in_the_crate_itself_last_wins`].
//!    Practical consequence for this module: a byte-identical raw-name
//!    duplicate is a gap this module's own `seen_paths` check structurally
//!    CANNOT see (there is only ever one entry to iterate), so no
//!    "duplicate name" coverage row can ever fire for that exact case — a
//!    limitation stated here rather than silently assumed away, and no
//!    worse than the crate's own already-deterministic, non-ambiguous
//!    resolution of it (one entry, the last one in file order, never a
//!    crash and never two conflicting children). This module's own check
//!    still catches every case that DOES survive the crate's dedup: two
//!    different raw names whose `enclosed_name()` outputs happen to agree.
//! 4. **Normalisation collision** — see [`collision_key`] below for the rule
//!    and why it is not a bare `to_lowercase()`.
//!
//! # The normalisation rule, justified
//!
//! [`collision_key`] applies Unicode NFC (canonical decomposition followed by
//! canonical composition — `unicode_normalization::UnicodeNormalization::nfc`,
//! VERIFIED against the crate's own docs.rs page) **before** case-folding,
//! not instead of it. Two entries are flagged as colliding only if their
//! *enclosed* paths, once independently NFC-normalised (closing the
//! NFC/NFD-decomposition gap `enclosed_name` leaves open, research §1) and
//! then case-converted via `str::to_lowercase` (Rust's full Unicode default
//! case conversion, not an ASCII-only fold — the crate's own documented
//! behaviour), become byte-identical. Doing normalisation first matters:
//! `to_lowercase` alone on two differently-composed byte sequences for the
//! *same* visual text can itself disagree (case conversion is defined over
//! Unicode code points, and a precomposed vs. decomposed sequence is a
//! different code-point sequence), so skipping NFC first — the "just call
//! `to_lowercase`" shortcut the amendment explicitly rejects — can miss
//! exactly the NFC/NFD pair this check exists to catch. The collision key is
//! used **only** to detect a colliding *second* entry, refused with its own
//! named row; the admitted entry's own `relative_path` is left exactly as
//! `enclosed_name()` produced it — citing the original entry name, never a
//! normalised rewrite of it, matches this codebase's "derived output cites
//! the original" rule (A1-12) applied to a container entry's own identity.
//!
//! Known, stated limitation: `str::to_lowercase` performs Unicode's default
//! (locale-independent) case conversion, which does not special-case a
//! handful of locale-specific mappings (Turkish dotless ı, for instance).
//! Accepted as an honest approximation — the same posture this codebase
//! already takes for provisional numeric ceilings (module doc, "Bounds") —
//! rather than pulling in a full locale-aware case-folding stack for a
//! collision heuristic whose authoritative defence is the byte-identical
//! duplicate-name check above it, not this one.
//!
//! # Closing the research's open item: quines and overlapping entries
//!
//! **FINDING**: `zip` 8.6.0 does **not** reject overlapping or
//! self-referential (quine-shaped) local-file-header constructions as part
//! of ordinary archive parsing. `ZipArchive::new`/`by_index` accept such an
//! archive without complaint. The crate ships exactly one piece of relevant
//! machinery, and it is opt-in diagnostics, not a parse-time refusal —
//! VERIFIED directly against `zip` 8.6.0's own source
//! (`src/read/zip_archive.rs`):
//!
//! ```text
//! /// Returns Ok(true) if any compressed data in this archive belongs to more than one file. This
//! /// doesn't make the archive invalid, but some programs will refuse to decompress it because the
//! /// copies would take up space independently in the destination.
//! pub fn has_overlapping_files(&mut self) -> ZipResult<bool> { .. }
//! ```
//!
//! "This doesn't make the archive invalid" is the crate's own words for
//! exactly the gap the research flagged as unverified. [`expand`] closes it
//! by calling `has_overlapping_files` itself, BEFORE opening any entry, and
//! refusing the WHOLE archive (one archive-level [`CoverageRow`], no entry
//! ever read) if it reports `true` — the defence the crate does not apply on
//! its own behalf. This is a real, load-bearing check, not a formality:
//! [`overlapping_files_refuse_the_whole_archive_before_any_entry_opens`]
//! proves it against a hand-crafted fixture whose central directory points
//! two entries at the identical compressed-data byte range.
//!
//! What this does **not** claim to close: a central-directory/local-header
//! offset so malformed that `ZipArchive::new` itself fails (an out-of-range
//! offset, a truncated header) is a **different** failure this module
//! already reports as [`CoverageRow`] with [`Coverage::Error`] — the archive
//! never opens far enough to reach the overlap check at all, and that is the
//! correct, honest place for it: a container this malformed is refused
//! before any entry is even *addressable*, which is a stronger refusal than
//! the overlap check, not a weaker one.
//!
//! # Bounds, enforced by counting streamed bytes — never by trusting a header
//!
//! `size()`/`compressed_size()` are raw header fields the crate reconciles
//! against reality only by CRC-32 *after* the whole entry has already been
//! decompressed (research §2, VERIFIED against `zip`'s own accessor docs).
//! [`expand`] therefore never allocates from either value: every entry
//! is read through `Read::take(MAX_ENTRY_UNCOMPRESSED_BYTES + 1)` and the
//! *actual* byte count read is what every per-entry and total-size bound is
//! checked against. The `+ 1` is what lets this module tell "the entry
//! coincidentally has exactly the ceiling's byte count" apart from "the
//! entry kept going past it" — reading one byte past the cap and observing
//! whether the stream still had more is the standard shape for that
//! distinction.
//!
//! Every ceiling below is a named constant with its own rationale, the
//! specific failure it prevents, and — per this build's existing honesty
//! rule for a number nothing has measured yet ([`super::worker::WORKER_ADDRESS_SPACE_LIMIT_BYTES`]'s
//! own doc, and #325 before it) — **stated as PROVISIONAL, not validated,**
//! except [`MAX_ENTRY_UNCOMPRESSED_BYTES`], which is not a new number at
//! all: it reuses [`super::scan::MAX_RESOURCE_BYTES`] outright (R2) so "a
//! document is too big to read" stays one policy with one rationale, not two
//! independently-tuned numbers with evidence for neither.
//!
//! * [`MAX_ZIP_ENTRIES`] — refuses the WHOLE archive, before any entry
//!   opens, when the central directory declares more than this many records.
//!   **Not** a defence against the archive-open cost itself: a central
//!   directory this large needs real backing bytes on disk (each record is
//!   a real struct plus a real filename string, unlike `size()`, which is a
//!   single lied-about integer disconnected from any real content) — so the
//!   resource's own pre-existing whole-file size ceiling
//!   ([`super::scan::MAX_RESOURCE_BYTES`]/[`super::scan::MAX_DATASET_BYTES`],
//!   enforced before this adapter ever runs) already bounds how many CD
//!   records a hostile input could plausibly carry. What this ceiling
//!   defends against instead is DOWNSTREAM fan-out: an entry count in the
//!   low thousands is realistic for a legitimate knowledge-source archive
//!   (a repackaged project tree), and each entry this module admits becomes
//!   its own coverage row, content hash and composed key — cheap per entry,
//!   ruinous in aggregate, and cheap for an attacker to construct (mostly
//!   metadata, little real compressed data needed per entry).
//! * [`MAX_ENTRY_UNCOMPRESSED_BYTES`] — reused from
//!   [`super::scan::MAX_RESOURCE_BYTES`] (4 MiB). Prevents the classic
//!   single-entry decompression bomb: one hostile entry expanding to
//!   gigabytes while its compressed bytes stay tiny.
//! * [`MAX_TOTAL_EXPANDED_BYTES`] — 128 MiB, chosen well below
//!   [`super::worker::WORKER_ADDRESS_SPACE_LIMIT_BYTES`] (512 MiB): the
//!   worker's one address-space ceiling has to cover the archive's own
//!   input bytes, `zip`'s internal decompression scratch, AND this module's
//!   accumulated output all at once, so the container-level ceiling is set
//!   with headroom under the process-level one rather than racing it.
//!   Prevents the "distributed bomb" shape: many entries, each individually
//!   under the per-entry cap, that together still exhaust memory. **This is
//!   the least-grounded number in this file** — the research note names it
//!   explicitly as having no existing whole-archive-scope precedent in this
//!   codebase to size against, and this module inherits that honesty rather
//!   than implying a validated figure.
//! * [`MAX_COMPRESSION_RATIO`] — 200:1, an ADVISORY pre-filter only, computed
//!   from the two attacker-declared header fields before any byte is
//!   decompressed. Cheap triage in front of the expensive
//!   [`MAX_ENTRY_UNCOMPRESSED_BYTES`] streaming check, which is what
//!   actually holds under a header that lies. Deliberately unmeasured
//!   against this crate's enabled codec (`deflate` only — see `Cargo.toml`'s
//!   own comment on why the other codecs are not enabled at all, precisely
//!   so this one flat number does not have to cover their steeper worst
//!   cases too).
//! * [`MAX_NESTING_DEPTH`] — 2 levels, reasoned rather than measured: a
//!   legitimate knowledge source rarely nests an archive inside an archive
//!   more than once; 42.zip-shaped multiplicative bombs need depth, and a
//!   shallow cap makes the multiplication impossible regardless of what any
//!   single level's own bounds allow (research §3's own "a naive
//!   implementation... recurses without limit still falls to this" case).
//!   A nested archive past the cap still becomes its own child resource
//!   (hash, key, provenance) — it is simply not opened further.
//!
//! # Provenance and F7 child keys (G9)
//!
//! Every admitted [`ZipChild`] carries its own [`domain::source::content_hash`]
//! and a key composed by [`domain::source::child_key`] from **the immediate
//! parent's own key** — chained, not resolved to the top-level archive, so a
//! grandchild's key already encodes its whole ancestry (see `child_key`'s own
//! doc for the full argument and why a flattened "always key off the root"
//! scheme would collide two different nested archives that share a leaf
//! path). `entry_adapter` is the CHILD's own downstream extractor identity —
//! [`super::text::extractor_for`]/[`super::office::extractor_for`] applied to
//! the entry's own enclosed path — `None` when nothing in this build claims
//! that extension yet, never the container adapter that unpacked the entry
//! itself (the research note's own warning: recording every archive-derived
//! child as `adapter=zip` erases exactly the information F7's key exists to
//! carry).
//!
//! # A named seam: not yet wired onto the wire batch
//!
//! [`expand`] is proven directly, in-process, against every admission and
//! bounds claim above (mirroring `office.rs`'s own `#[cfg(test)]` block) —
//! but, like `office::extractor_for` before it ("Nothing calls this in
//! production yet", that module's own doc), [`super::worker::DeclaredChild`] does
//! not yet carry a child's content bytes, content hash, or composed key on
//! the wire: it still ships exactly the `name`/`relative_path` pair Y1
//! defined for its own synthetic `--declare-child` test fixture. Widening
//! that shared wire type to carry a child's real bytes is a decision with
//! its own security/footprint shape (JSON-embedding potentially-large binary
//! content on every worker round trip) that this wave's brief does not
//! settle and this module does not decide unilaterally (J0: no rung above
//! resolves it, and it changes a contract every later wave depends on) —
//! recorded here, not silently deferred, so the wave that wires real
//! daemon-side persistence for archive children (riding G8's own scan
//! trigger, Y5) has one place to look. What IS wired today: `sgt-atlas-worker`
//! dispatches [`ZIP_EXTRACTOR`] to the real [`expand`] adapter and reports
//! its admitted children through the EXISTING `name`/`relative_path` wire
//! shape, so [`super::worker::validate_batch`]'s daemon-side authority (path
//! safety, F10 deny-set membership) already runs, for real, against a real
//! container adapter's real output — see `tests/y3_zip_adapter.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read};

use unicode_normalization::UnicodeNormalization;

use crate::domain::source::{Coverage, CoverageRow, child_key, content_hash};

// ------------------------------------------------------------- routing

/// Extractor identity for bounded ZIP expansion (F7's second cache-key
/// input).
pub const ZIP_EXTRACTOR: &str = "zip/v1";

/// Extensions routed to [`expand`].
pub const ZIP_EXTENSIONS: &[&str] = &["zip"];

/// The extractor for a path, by extension — mirrors [`super::text::extractor_for`]
/// and [`super::office::extractor_for`]'s own shape exactly (extension-driven,
/// never content-sniffed; an unclaimed extension is honestly `unsupported`).
pub fn extractor_for(relative: &str) -> Option<&'static str> {
    let extension = std::path::Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase();
    ZIP_EXTENSIONS
        .contains(&extension.as_str())
        .then_some(ZIP_EXTRACTOR)
}

// --------------------------------------------------------------- bounds

/// Ceiling on declared central-directory entries. See the module doc's
/// "Bounds" section for the full rationale. **4096, PROVISIONAL.**
pub const MAX_ZIP_ENTRIES: usize = 4096;

/// Per-entry uncompressed-size ceiling, enforced by counting streamed bytes.
/// Reused outright from [`super::scan::MAX_RESOURCE_BYTES`] (R2) rather than
/// a second, independently-tuned number — see the module doc.
pub const MAX_ENTRY_UNCOMPRESSED_BYTES: u64 = super::scan::MAX_RESOURCE_BYTES;

/// Ceiling on one archive level's cumulative admitted-entry byte count.
/// **128 MiB, PROVISIONAL** — see the module doc for why this is the
/// least-grounded number in this file. Scoped to one archive LEVEL, not
/// summed across nesting levels — see [`MAX_NESTING_DEPTH`]'s own doc for
/// why the depth cap, not a cross-level sum, is what bounds the
/// multiplicative 42.zip shape.
pub const MAX_TOTAL_EXPANDED_BYTES: u64 = 128 * 1024 * 1024;

/// Advisory pre-filter on the two attacker-declared header fields
/// (`size()`/`compressed_size()`), computed before any byte is decompressed.
/// **200:1, PROVISIONAL and UNMEASURED** against this build's one enabled
/// codec (`deflate`) — see the module doc.
pub const MAX_COMPRESSION_RATIO: u64 = 200;

/// Ceiling on archive-within-archive nesting. **2, PROVISIONAL** — see the
/// module doc for the 42.zip argument this specifically defends against.
pub const MAX_NESTING_DEPTH: u32 = 2;

// ------------------------------------------------------------ output shape

/// One admitted archive entry, in this crate's own vocabulary — no `zip::`
/// type crosses this module's boundary into a caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipChild {
    /// The entry's own enclosed path, exactly as `enclosed_name()` produced
    /// it — never the [`collision_key`] normal form, which exists only to
    /// detect a colliding second entry (module doc).
    pub relative_path: String,
    /// The entry's own decompressed bytes, bounded by
    /// [`MAX_ENTRY_UNCOMPRESSED_BYTES`] (streamed, never allocated from a
    /// declared header size).
    pub content: Vec<u8>,
    /// BLAKE3 hex of `content` — F7's content half.
    pub content_hash: String,
    /// F7's composed child key (G9) — [`domain::source::child_key`] applied
    /// to `(parent_key, relative_path, content_hash, extractor)`, where
    /// `extractor` is `entry_adapter` when claimed, or a stable placeholder
    /// naming "unsupported" when nothing claims this extension yet (so the
    /// key still moves if a later build starts claiming it).
    pub key: String,
    /// The CHILD's own downstream extractor identity — `None` when this
    /// build has no adapter for the entry's extension, or when the entry is
    /// itself a nested archive (its own "adapter" is [`ZIP_EXTRACTOR`],
    /// carried instead by `is_nested_archive`/`nested` below, not this
    /// field — see the module doc's warning against recording every child
    /// as `adapter=zip`).
    pub entry_adapter: Option<&'static str>,
    /// Whether this entry is itself a ZIP archive by extension.
    pub is_nested_archive: bool,
    /// `Some` when `is_nested_archive` is true AND [`MAX_NESTING_DEPTH`]
    /// allowed opening it — the recursive expansion of this entry's own
    /// bytes. `None` for an ordinary leaf entry, and for a nested archive
    /// refused recursion by the depth ceiling (which still becomes a
    /// `ZipChild` with its own hash/key — see the module doc's
    /// `MAX_NESTING_DEPTH` bullet).
    pub nested: Option<Box<ZipExpansion>>,
}

/// One archive level's whole answer: every admitted child, plus a named
/// [`CoverageRow`] for every entry (or the archive as a whole) that was
/// refused — never a silent skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipExpansion {
    /// Every entry admitted at this level (not counting grandchildren nested
    /// inside a child's own `nested`).
    pub children: Vec<ZipChild>,
    /// Every refusal, at the archive level and at the entry level, this
    /// level's own walk produced.
    pub coverage: Vec<CoverageRow>,
    /// This level's own cumulative admitted-entry byte count (module doc:
    /// scoped to this level, not summed across nesting).
    pub total_expanded_bytes: u64,
}

// ------------------------------------------------------------- the adapter

/// Expand one ZIP resource's bytes into admitted children plus coverage,
/// against `parent_key` — the caller's own [`domain::source::local_key`]/
/// [`domain::source::estate_git_key`] for this resource, which every child
/// key composes on top of (F7/G9).
///
/// Pure (F6): no file is opened beyond `bytes`, no clock is read, no store is
/// touched. Two calls on equal bytes and an equal `parent_key` are equal —
/// proven directly below, the same way `office.rs`'s own purity test does.
pub fn expand(bytes: &[u8], parent_key: &str) -> ZipExpansion {
    expand_at_depth(bytes, parent_key, 0)
}

fn expand_at_depth(bytes: &[u8], parent_key: &str, depth: u32) -> ZipExpansion {
    let mut coverage = Vec::new();
    let mut children = Vec::new();

    let mut archive = match zip::ZipArchive::new(Cursor::new(bytes)) {
        Ok(archive) => archive,
        Err(error) => {
            coverage.push(CoverageRow {
                path: None,
                status: Coverage::Error,
                detail: Some(format!(
                    "archive could not be opened: not a well-formed ZIP central directory: \
                     {error}"
                )),
                bytes: Some(bytes.len() as u64),
            });
            return ZipExpansion {
                children,
                coverage,
                total_expanded_bytes: 0,
            };
        }
    };

    // Close the research's open item BEFORE any entry is opened (module
    // doc, "Closing the research's open item"): a quine/overlap-shaped
    // archive refuses whole, never partially.
    match archive.has_overlapping_files() {
        Ok(true) => {
            coverage.push(CoverageRow {
                path: None,
                status: Coverage::Error,
                detail: Some(
                    "archive refused: two or more entries' compressed data occupy overlapping \
                     byte ranges (a self-referential/quine-shaped construction `zip` 8.6.0 does \
                     not reject on its own — its own `has_overlapping_files` doc says so \
                     verbatim: \"this doesn't make the archive invalid\"); the whole archive is \
                     refused rather than opening any entry"
                        .to_string(),
                ),
                bytes: Some(bytes.len() as u64),
            });
            return ZipExpansion {
                children,
                coverage,
                total_expanded_bytes: 0,
            };
        }
        Ok(false) => {}
        Err(error) => {
            coverage.push(CoverageRow {
                path: None,
                status: Coverage::Error,
                detail: Some(format!(
                    "archive refused: could not determine whether entries overlap: {error}"
                )),
                bytes: Some(bytes.len() as u64),
            });
            return ZipExpansion {
                children,
                coverage,
                total_expanded_bytes: 0,
            };
        }
    }

    let declared_count = archive.len();
    if declared_count > MAX_ZIP_ENTRIES {
        coverage.push(CoverageRow {
            path: None,
            status: Coverage::Unsupported,
            detail: Some(format!(
                "archive declares {declared_count} entries, exceeding the \
                 {MAX_ZIP_ENTRIES}-entry MAX_ZIP_ENTRIES ceiling; refused before any entry was \
                 opened"
            )),
            bytes: Some(bytes.len() as u64),
        });
        return ZipExpansion {
            children,
            coverage,
            total_expanded_bytes: 0,
        };
    }

    let mut seen_paths: BTreeSet<String> = BTreeSet::new();
    let mut seen_collisions: BTreeMap<String, String> = BTreeMap::new();
    let mut total_expanded_bytes: u64 = 0;

    for index in 0..declared_count {
        let mut entry = match archive.by_index(index) {
            Ok(entry) => entry,
            Err(error) => {
                coverage.push(CoverageRow {
                    path: None,
                    status: Coverage::Error,
                    detail: Some(format!("entry {index} could not be read: {error}")),
                    bytes: None,
                });
                continue;
            }
        };
        let raw_name = entry.name().to_string();

        let Some(enclosed) = entry.enclosed_name() else {
            coverage.push(CoverageRow {
                path: Some(raw_name.clone()),
                status: Coverage::Excluded,
                detail: Some(
                    "entry name is not enclosed-safe (an absolute path, a `..` component, a \
                     drive letter/UNC prefix past the first component, or a NUL byte) per \
                     `zip`'s own `enclosed_name` check"
                        .to_string(),
                ),
                bytes: None,
            });
            continue;
        };
        if enclosed.as_os_str().is_empty() {
            coverage.push(CoverageRow {
                path: Some(raw_name.clone()),
                status: Coverage::Error,
                detail: Some(
                    "entry name is empty; `enclosed_name` accepts an empty string as a valid \
                     empty path, but an entry with no name has no meaningful coordinate and is \
                     refused rather than silently keyed against its parent"
                        .to_string(),
                ),
                bytes: None,
            });
            continue;
        }
        let path = enclosed.to_string_lossy().replace('\\', "/");

        if entry.is_dir() {
            // A directory marker carries no content of its own — recorded
            // once, honestly, rather than silently absorbed (F8's "no
            // eighth silent skip"), but it is bookkeeping, not a child.
            coverage.push(CoverageRow {
                path: Some(path),
                status: Coverage::Discovered,
                detail: None,
                bytes: Some(0),
            });
            continue;
        }
        if entry.is_symlink() {
            coverage.push(CoverageRow {
                path: Some(path),
                status: Coverage::Excluded,
                detail: Some(
                    "entry is a symlink; refused unconditionally — its enclosed name is safe \
                     but its content is a link-TARGET string, not file bytes, and treating it \
                     as a symlink is a traversal primitive by another route (G5 amendment)"
                        .to_string(),
                ),
                bytes: None,
            });
            continue;
        }
        if !entry.is_file() {
            coverage.push(CoverageRow {
                path: Some(path),
                status: Coverage::Excluded,
                detail: Some(format!(
                    "entry is not a regular file (unix mode {:?} names neither a directory nor \
                     a symlink); refused",
                    entry.unix_mode()
                )),
                bytes: None,
            });
            continue;
        }

        if !seen_paths.insert(path.clone()) {
            coverage.push(CoverageRow {
                path: Some(path.clone()),
                status: Coverage::Excluded,
                detail: Some(format!(
                    "duplicate entry name {path:?}; the first occurrence at this path was \
                     admitted, this later entry sharing the identical enclosed path is refused \
                     rather than silently shadowing it"
                )),
                bytes: None,
            });
            continue;
        }
        let collision = collision_key(&path);
        if let Some(prior) = seen_collisions.get(&collision) {
            coverage.push(CoverageRow {
                path: Some(path.clone()),
                status: Coverage::Excluded,
                detail: Some(format!(
                    "entry {path:?} collides with previously admitted entry {prior:?} once \
                     both are Unicode-NFC-normalised and case-folded — the shape a \
                     case-insensitive or NFC/NFD-folding filesystem would collide on; refused \
                     rather than silently shadowed"
                )),
                bytes: None,
            });
            continue;
        }
        seen_collisions.insert(collision, path.clone());

        let declared_size = entry.size();
        let declared_compressed = entry.compressed_size();
        if ratio_trips(declared_size, declared_compressed) {
            coverage.push(CoverageRow {
                path: Some(path),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "entry's HEADER-DECLARED compression ratio ({declared_size}:{declared_compressed}, \
                     approx {}:1) exceeds the {MAX_COMPRESSION_RATIO}:1 MAX_COMPRESSION_RATIO \
                     pre-check ceiling; entry not opened (advisory pre-filter over \
                     attacker-declared header fields — not a measured ratio)",
                    declared_size / declared_compressed.max(1)
                )),
                bytes: None,
            });
            continue;
        }

        let mut limited = (&mut entry).take(MAX_ENTRY_UNCOMPRESSED_BYTES + 1);
        let mut content = Vec::new();
        if let Err(error) = limited.read_to_end(&mut content) {
            coverage.push(CoverageRow {
                path: Some(path),
                status: Coverage::Error,
                detail: Some(format!("entry could not be decompressed: {error}")),
                bytes: None,
            });
            continue;
        }
        if content.len() as u64 > MAX_ENTRY_UNCOMPRESSED_BYTES {
            coverage.push(CoverageRow {
                path: Some(path),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "entry exceeded the {MAX_ENTRY_UNCOMPRESSED_BYTES}-byte \
                     MAX_ENTRY_UNCOMPRESSED_BYTES ceiling WHILE DECOMPRESSING (a streamed byte \
                     count, not the header's declared size — the header is attacker-controlled \
                     and was not what tripped this)"
                )),
                bytes: None,
            });
            continue;
        }

        let entry_len = content.len() as u64;
        if total_expanded_bytes.saturating_add(entry_len) > MAX_TOTAL_EXPANDED_BYTES {
            coverage.push(CoverageRow {
                path: Some(path),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "archive's cumulative expanded size exceeded the \
                     {MAX_TOTAL_EXPANDED_BYTES}-byte MAX_TOTAL_EXPANDED_BYTES ceiling at this \
                     entry ({} of {declared_count} declared entries opened); remaining entries \
                     were never opened",
                    index + 1
                )),
                bytes: Some(entry_len),
            });
            break;
        }
        total_expanded_bytes += entry_len;

        let hash = content_hash(&content);
        let (entry_adapter, is_nested_archive) = classify(&path);
        let key_extractor = entry_adapter.unwrap_or(UNSUPPORTED_CHILD_EXTRACTOR);
        let key = child_key(parent_key, &path, &hash, key_extractor);

        let nested = if is_nested_archive {
            if depth + 1 > MAX_NESTING_DEPTH {
                coverage.push(CoverageRow {
                    path: Some(path.clone()),
                    status: Coverage::Unsupported,
                    detail: Some(format!(
                        "nested archive at {path:?} not opened: opening it would exceed the \
                         {MAX_NESTING_DEPTH}-level MAX_NESTING_DEPTH ceiling; the entry itself \
                         is still admitted as a child resource, just not recursively expanded"
                    )),
                    bytes: Some(entry_len),
                });
                None
            } else {
                Some(Box::new(expand_at_depth(&content, &key, depth + 1)))
            }
        } else {
            None
        };

        children.push(ZipChild {
            relative_path: path,
            content,
            content_hash: hash,
            key,
            entry_adapter,
            is_nested_archive,
            nested,
        });
    }

    ZipExpansion {
        children,
        coverage,
        total_expanded_bytes,
    }
}

/// The placeholder extractor identity folded into a child's [`child_key`]
/// when nothing in this build claims its extension — named and stable
/// rather than an empty string, so the key still moves the day a real
/// adapter starts claiming that extension (which changes what this
/// placeholder would otherwise have silently stood in for).
const UNSUPPORTED_CHILD_EXTRACTOR: &str = "unsupported/v1";

/// The CHILD's own downstream adapter, by extension — reusing the exact
/// routing functions a top-level scan would consult (R2), never a
/// zip-specific guess. `(None, false)` when nothing claims it and it is not
/// itself an archive.
fn classify(relative_path: &str) -> (Option<&'static str>, bool) {
    if extractor_for(relative_path).is_some() {
        return (None, true);
    }
    if let Some(extractor) = super::text::extractor_for(relative_path) {
        return (Some(extractor), false);
    }
    if let Some(extractor) = super::office::extractor_for(relative_path) {
        return (Some(extractor), false);
    }
    (None, false)
}

/// Advisory pre-filter (module doc): `true` when the header-declared ratio
/// exceeds [`MAX_COMPRESSION_RATIO`], computed without decompressing
/// anything. A `size` of zero cannot bomb anything and never trips this. A
/// nonzero declared `size` with a declared `compressed_size` of zero is
/// itself an internally-inconsistent header (claiming content out of no
/// compressed bytes) and trips this unconditionally, rather than dividing by
/// zero.
fn ratio_trips(declared_size: u64, declared_compressed: u64) -> bool {
    if declared_size == 0 {
        return false;
    }
    if declared_compressed == 0 {
        return true;
    }
    declared_size / declared_compressed > MAX_COMPRESSION_RATIO
}

/// The normalisation-collision key (module doc: NFC first, then Unicode
/// default case conversion — never a bare `to_lowercase()` on its own).
fn collision_key(enclosed_path: &str) -> String {
    enclosed_path.nfc().collect::<String>().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ---------------------------------------------------------- fixture builders
    //
    // Hand-built directly against the `zip` crate's own writer, in this
    // module's own test block — no external fixture files needed for the
    // synthetic/hostile cases; the real-world corpus lives under
    // `tests/fixtures/zip_corpus/` and is exercised by `tests/y3_zip_adapter.rs`
    // through the real supervised worker.

    fn build(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(Cursor::new(&mut buffer));
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for (name, content) in entries {
                writer.start_file(*name, options).expect("start_file");
                writer.write_all(content).expect("write entry");
            }
            writer.finish().expect("finish archive");
        }
        buffer
    }

    /// Same as [`build`], but STORED (uncompressed) — for a fixture whose
    /// declared size must equal its declared compressed size exactly, so
    /// [`ratio_trips`]'s advisory pre-filter (ratio 1:1) never fires and a
    /// test can isolate the streamed-byte-count bound it actually means to
    /// exercise.
    fn build_stored(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(Cursor::new(&mut buffer));
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            for (name, content) in entries {
                writer.start_file(*name, options).expect("start_file");
                writer.write_all(content).expect("write entry");
            }
            writer.finish().expect("finish archive");
        }
        buffer
    }

    fn build_with_dir(entries: &[(&str, &[u8])], dirs: &[&str]) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(Cursor::new(&mut buffer));
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for dir in dirs {
                writer.add_directory(*dir, options).expect("add_directory");
            }
            for (name, content) in entries {
                writer.start_file(*name, options).expect("start_file");
                writer.write_all(content).expect("write entry");
            }
            writer.finish().expect("finish archive");
        }
        buffer
    }

    // ------------------------------------------------------------------- F6

    #[test]
    fn expansion_is_a_pure_function_of_its_input() {
        let bytes = build(&[("a.txt", b"hello"), ("b/c.md", b"# heading\n")]);
        assert_eq!(expand(&bytes, "parent"), expand(&bytes, "parent"));
    }

    // -------------------------------------------------------- happy path

    #[test]
    fn a_well_formed_archive_admits_every_entry_with_full_provenance() {
        let bytes = build_with_dir(
            &[("notes/a.md", b"# A\n"), ("notes/b.txt", b"plain text")],
            &["notes/"],
        );
        let expansion = expand(&bytes, "parent-key");
        assert!(
            expansion
                .coverage
                .iter()
                .all(|row| row.status != Coverage::Error),
            "no refusal expected: {:?}",
            expansion.coverage
        );
        assert_eq!(expansion.children.len(), 2, "{:?}", expansion.children);

        let md = expansion
            .children
            .iter()
            .find(|c| c.relative_path == "notes/a.md")
            .expect("markdown child present");
        assert_eq!(md.content, b"# A\n");
        assert_eq!(md.content_hash, content_hash(b"# A\n"));
        assert_eq!(
            md.entry_adapter,
            Some(super::super::text::MARKDOWN_EXTRACTOR)
        );
        assert!(!md.is_nested_archive);
        assert_eq!(
            md.key,
            child_key(
                "parent-key",
                "notes/a.md",
                &content_hash(b"# A\n"),
                super::super::text::MARKDOWN_EXTRACTOR
            )
        );

        let directory_rows: Vec<_> = expansion
            .coverage
            .iter()
            .filter(|r| r.status == Coverage::Discovered)
            .collect();
        assert_eq!(
            directory_rows.len(),
            1,
            "the directory marker is recorded, not silently absorbed: {:?}",
            expansion.coverage
        );
    }

    #[test]
    fn an_unsupported_extension_is_still_admitted_with_no_entry_adapter() {
        let bytes = build(&[("data.bin", &[0xDE, 0xAD, 0xBE, 0xEF])]);
        let expansion = expand(&bytes, "parent-key");
        assert_eq!(expansion.children.len(), 1);
        assert_eq!(expansion.children[0].entry_adapter, None);
        assert_eq!(
            expansion.children[0].key,
            child_key(
                "parent-key",
                "data.bin",
                &content_hash(&[0xDE, 0xAD, 0xBE, 0xEF]),
                UNSUPPORTED_CHILD_EXTRACTOR
            ),
            "an unsupported extension still gets a stable, distinct key"
        );
    }

    // ---------------------------------------------------- entry admission (a)

    #[test]
    fn a_symlink_entry_is_refused_and_produces_no_child() {
        let mut buffer = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(Cursor::new(&mut buffer));
            // `FileOptions::unix_permissions` masks its input with `& 0o777`
            // (VERIFIED, `zip` 8.6.0's own `src/write.rs`: "This method only
            // preserves the file permissions bits... and discards higher
            // file mode bits. So it cannot be used to denote an entry as a
            // directory, symlink, or other special file type.") — the
            // crate's own dedicated `add_symlink` is what actually sets the
            // `S_IFLNK` bit on the entry's external attributes.
            let options = zip::write::SimpleFileOptions::default();
            writer
                .add_symlink("escape", "../../etc/passwd", options)
                .expect("add_symlink");
            writer.finish().expect("finish archive");
        }
        let expansion = expand(&buffer, "parent-key");
        assert!(expansion.children.is_empty(), "{:?}", expansion.children);
        let row = expansion
            .coverage
            .iter()
            .find(|r| r.path.as_deref() == Some("escape"))
            .expect("a coverage row for the symlink entry");
        assert_eq!(row.status, Coverage::Excluded);
        assert!(
            row.detail
                .as_deref()
                .unwrap_or_default()
                .contains("symlink")
        );
    }

    #[test]
    fn two_different_raw_names_colliding_on_the_same_enclosed_path_is_refused() {
        // The case this module's own `seen_paths` check CAN and does catch:
        // two DIFFERENT raw central-directory names (distinct raw bytes, so
        // the crate's own `IndexMap` keeps both as independent entries —
        // unlike the byte-identical-names case below) whose
        // `enclosed_name()` outputs collapse to the identical enclosed
        // path, because `enclosed_name` drops a leading `CurDir` component
        // (research §1, VERIFIED).
        let bytes = build(&[("b.txt", b"first"), ("./b.txt", b"second")]);
        let expansion = expand(&bytes, "parent-key");
        assert_eq!(expansion.children.len(), 1, "{:?}", expansion.children);
        assert_eq!(
            expansion.children[0].content, b"first",
            "first occurrence wins"
        );
        let refusal = expansion
            .coverage
            .iter()
            .find(|r| {
                r.status == Coverage::Excluded
                    && r.detail
                        .as_deref()
                        .unwrap_or_default()
                        .contains("duplicate")
            })
            .expect("a duplicate-name coverage row");
        assert_eq!(refusal.path.as_deref(), Some("b.txt"));
    }

    /// **A correction to the research note, pinned as a decisive check**
    /// (module doc, "Name uniqueness"): two central-directory records
    /// sharing BYTE-IDENTICAL raw names do not both survive to be visited
    /// — `zip` 8.6.0's own `IndexMap`-keyed-on-raw-name-bytes construction
    /// collapses them to exactly one entry, silently, last-write-wins,
    /// before `ZipArchive::len()`/`by_index` are ever called. Proven
    /// directly against the crate, not against this module's own `expand`
    /// — the point is what the CRATE does, independent of any defence this
    /// module might add on top.
    #[test]
    fn identical_raw_names_collapse_silently_in_the_crate_itself_last_wins() {
        // `zip::write::ZipWriter::start_file` itself refuses to WRITE a
        // duplicate name (`InvalidArchive("Duplicate filename: ...")`), so
        // this fixture cannot come from the writer directly: two
        // SAME-LENGTH distinct names are written, then every raw
        // occurrence of the second name's bytes (its local header, its
        // central-directory record — never inside the 5/6-byte content) is
        // replaced with the first name's bytes IN PLACE, which cannot
        // shift any offset because the two names are the same length.
        let mut bytes = build(&[("dup0.txt", b"first"), ("dup1.txt", b"second")]);
        replace_all_bytes(&mut bytes, b"dup1.txt", b"dup0.txt");

        let mut archive = zip::ZipArchive::new(Cursor::new(&bytes)).expect("archive opens");
        assert_eq!(
            archive.len(),
            1,
            "two central-directory records with identical raw names collapse to ONE archive \
             entry — `len()` itself reports the deduplicated count, not the raw record count"
        );
        let mut content = Vec::new();
        archive
            .by_index(0)
            .expect("the one surviving entry")
            .read_to_end(&mut content)
            .expect("read");
        assert_eq!(
            content, b"second",
            "the LAST record written wins — the first one's bytes are unreachable through any \
             public API, not merely hidden from by_name"
        );
    }

    /// In-place, exact-length byte replacement — panics if `from`/`to`
    /// differ in length (every offset elsewhere in the archive would shift
    /// and silently corrupt the fixture) or if `from` is not found.
    fn replace_all_bytes(bytes: &mut [u8], from: &[u8], to: &[u8]) {
        assert_eq!(
            from.len(),
            to.len(),
            "in-place replacement must not change length"
        );
        let mut found = 0usize;
        let mut i = 0usize;
        while i + from.len() <= bytes.len() {
            if &bytes[i..i + from.len()] == from {
                bytes[i..i + from.len()].copy_from_slice(to);
                found += 1;
                i += from.len();
            } else {
                i += 1;
            }
        }
        assert!(found > 0, "{from:?} not found in fixture bytes");
    }

    #[test]
    fn an_empty_name_is_refused() {
        // `enclosed_name` accepts `""` as `Some(PathBuf::new())` (module
        // doc). No writer — `zip`'s own or Python's `zipfile` — will start
        // an entry with an empty name (both refuse it outright), so this
        // fixture is a fully hand-built minimal archive: one STORED,
        // zero-length-content entry with an empty name, spelled out
        // byte-for-byte against APPNOTE's own fixed-size record layouts
        // rather than mutated from a writer's output — see
        // `hand_built_empty_name_zip`'s own doc for the field-by-field
        // accounting.
        let bytes = hand_built_empty_name_zip();
        let mut sanity = zip::ZipArchive::new(Cursor::new(&bytes)).expect(
            "fixture sanity: the hand-built archive must itself open, or this test would prove \
             nothing about how `expand` handles an empty name",
        );
        assert_eq!(sanity.len(), 1);
        assert_eq!(
            sanity
                .by_index(0)
                .expect("the one entry")
                .enclosed_name()
                .expect("enclosed_name must accept the empty string"),
            std::path::PathBuf::new(),
            "fixture sanity: `enclosed_name` must actually return the empty path here"
        );

        let expansion = expand(&bytes, "parent-key");
        assert!(expansion.children.is_empty(), "{:?}", expansion.children);
        let row = expansion
            .coverage
            .iter()
            .find(|r| r.status == Coverage::Error)
            .expect("an error row for the empty-name entry");
        assert!(row.detail.as_deref().unwrap_or_default().contains("empty"));
    }

    /// One hand-built, minimal, well-formed ZIP: one STORED entry with an
    /// empty name and zero-length content. Every field below is spelled out
    /// against APPNOTE.TXT's own fixed-size local-file-header (30 bytes),
    /// central-directory-header (46 bytes) and end-of-central-directory
    /// (22 bytes) record layouts — no variable-length name/extra/comment
    /// field is present anywhere, so there is nothing for an offset
    /// bookkeeping mistake to desync. CRC-32 of zero bytes is the constant
    /// `0`, so no checksum computation is needed either.
    fn hand_built_empty_name_zip() -> Vec<u8> {
        let mut local = vec![
            0x50, 0x4B, 0x03, 0x04, // local file header signature
            0x14, 0x00, // version needed to extract (2.0)
            0x00, 0x00, // general purpose bit flag
            0x00, 0x00, // compression method: stored
            0x00, 0x00, // last mod file time
            0x21, 0x00, // last mod file date (1980-01-01)
            0x00, 0x00, 0x00, 0x00, // crc-32 of empty content
            0x00, 0x00, 0x00, 0x00, // compressed size
            0x00, 0x00, 0x00, 0x00, // uncompressed size
            0x00, 0x00, // file name length: 0
            0x00, 0x00, // extra field length: 0
        ];
        assert_eq!(local.len(), 30, "local file header is fixed at 30 bytes");
        let local_header_len = local.len() as u32;

        let mut central = vec![
            0x50, 0x4B, 0x01, 0x02, // central directory header signature
            0x14, 0x00, // version made by
            0x14, 0x00, // version needed to extract
            0x00, 0x00, // general purpose bit flag
            0x00, 0x00, // compression method: stored
            0x00, 0x00, // last mod file time
            0x21, 0x00, // last mod file date
            0x00, 0x00, 0x00, 0x00, // crc-32
            0x00, 0x00, 0x00, 0x00, // compressed size
            0x00, 0x00, 0x00, 0x00, // uncompressed size
            0x00, 0x00, // file name length: 0
            0x00, 0x00, // extra field length: 0
            0x00, 0x00, // file comment length: 0
            0x00, 0x00, // disk number start
            0x00, 0x00, // internal file attributes
            0x00, 0x00, 0x00, 0x00, // external file attributes
            0x00, 0x00, 0x00, 0x00, // relative offset of local header: 0
        ];
        assert_eq!(
            central.len(),
            46,
            "central directory header is fixed at 46 bytes"
        );
        let central_dir_len = central.len() as u32;

        let mut eocd = vec![
            0x50, 0x4B, 0x05, 0x06, // end of central directory signature
            0x00, 0x00, // number of this disk
            0x00, 0x00, // disk where central directory starts
            0x01, 0x00, // number of CD records on this disk
            0x01, 0x00, // total number of CD records
        ];
        eocd.extend_from_slice(&central_dir_len.to_le_bytes()); // size of CD
        eocd.extend_from_slice(&local_header_len.to_le_bytes()); // offset of start of CD
        eocd.extend_from_slice(&0u16.to_le_bytes()); // comment length
        assert_eq!(
            eocd.len(),
            22,
            "EOCD without a comment is fixed at 22 bytes"
        );

        let mut whole = Vec::new();
        whole.append(&mut local);
        whole.append(&mut central);
        whole.append(&mut eocd);
        whole
    }

    #[test]
    fn nfc_nfd_case_insensitive_collision_is_refused() {
        // "café" as NFC (é = U+00E9) vs. as NFD (e + U+0301 combining
        // acute) — the exact pair `enclosed_name` leaves undistinguished
        // from a byte-safety standpoint but which a case-insensitive or
        // NFC/NFD-folding filesystem would collide on.
        let nfc = "café.txt";
        let nfd = "cafe\u{0301}.TXT";
        assert_ne!(
            nfc.as_bytes(),
            nfd.as_bytes(),
            "fixture sanity: distinct bytes"
        );
        let bytes = build(&[(nfc, b"first"), (nfd, b"second")]);
        let expansion = expand(&bytes, "parent-key");
        assert_eq!(expansion.children.len(), 1, "{:?}", expansion.children);
        assert_eq!(expansion.children[0].relative_path, nfc);
        let refusal = expansion
            .coverage
            .iter()
            .find(|r| {
                r.detail
                    .as_deref()
                    .unwrap_or_default()
                    .contains("NFC-normalised")
            })
            .expect("a normalisation-collision coverage row");
        assert_eq!(refusal.path.as_deref(), Some(nfd));
    }

    // -------------------------------------------------------------- bounds (b)

    #[test]
    fn entry_count_over_the_ceiling_refuses_the_whole_archive_before_opening_any_entry() {
        let owned: Vec<(String, Vec<u8>)> = (0..(MAX_ZIP_ENTRIES + 1))
            .map(|i| (format!("f{i}.txt"), Vec::new()))
            .collect();
        let entries: Vec<(&str, &[u8])> = owned
            .iter()
            .map(|(name, content)| (name.as_str(), content.as_slice()))
            .collect();
        let bytes = build(&entries);
        let expansion = expand(&bytes, "parent-key");
        assert!(expansion.children.is_empty());
        assert_eq!(expansion.coverage.len(), 1, "{:?}", expansion.coverage);
        let row = &expansion.coverage[0];
        assert_eq!(
            row.path, None,
            "an archive-level refusal, not a per-entry one"
        );
        assert_eq!(row.status, Coverage::Unsupported);
        assert!(
            row.detail
                .as_deref()
                .unwrap_or_default()
                .contains("MAX_ZIP_ENTRIES")
        );
    }

    #[test]
    fn an_entry_past_the_per_entry_ceiling_is_refused_by_the_streamed_count_not_the_header() {
        // STORED, not Deflated: a repeated byte would trip the RATIO
        // pre-filter first (MAX_COMPRESSION_RATIO), which would prove the
        // wrong bound. STORED forces declared_size == declared_compressed
        // (ratio 1:1), isolating the streamed per-entry-size check this
        // test actually means to exercise.
        let oversized = vec![b'A'; (MAX_ENTRY_UNCOMPRESSED_BYTES + 1) as usize];
        let bytes = build_stored(&[("bomb.txt", &oversized)]);
        let expansion = expand(&bytes, "parent-key");
        assert!(expansion.children.is_empty(), "{:?}", expansion.children);
        let row = expansion
            .coverage
            .iter()
            .find(|r| r.path.as_deref() == Some("bomb.txt"))
            .expect("a coverage row for the oversized entry");
        assert_eq!(row.status, Coverage::Unsupported);
        let detail = row.detail.as_deref().unwrap_or_default();
        assert!(detail.contains("MAX_ENTRY_UNCOMPRESSED_BYTES"));
        assert!(detail.contains("WHILE DECOMPRESSING"));
    }

    #[test]
    fn total_expanded_size_over_the_ceiling_stops_the_walk_and_names_the_tripping_entry() {
        // Each entry sits exactly at the PER-entry ceiling (never over it,
        // so no entry is refused on that bound alone) and STORED (so the
        // ratio pre-filter never fires either) — isolating the TOTAL bound.
        // `MAX_TOTAL_EXPANDED_BYTES / MAX_ENTRY_UNCOMPRESSED_BYTES` entries
        // exactly fill the total ceiling; one entry past that trips it.
        let per_entry_admitted = (MAX_TOTAL_EXPANDED_BYTES / MAX_ENTRY_UNCOMPRESSED_BYTES) as usize;
        let declared_count = per_entry_admitted + 1;
        let owned: Vec<(String, Vec<u8>)> = (0..declared_count)
            .map(|i| {
                (
                    format!("e{i}.txt"),
                    vec![b'B'; MAX_ENTRY_UNCOMPRESSED_BYTES as usize],
                )
            })
            .collect();
        let entries: Vec<(&str, &[u8])> = owned
            .iter()
            .map(|(name, content)| (name.as_str(), content.as_slice()))
            .collect();
        let bytes = build_stored(&entries);
        let expansion = expand(&bytes, "parent-key");
        assert_eq!(
            expansion.children.len(),
            per_entry_admitted,
            "every entry up to and including the one that exactly fills the ceiling is admitted"
        );
        assert_eq!(expansion.total_expanded_bytes, MAX_TOTAL_EXPANDED_BYTES);
        let row = expansion
            .coverage
            .iter()
            .find(|r| r.status == Coverage::Unsupported)
            .expect("a total-size coverage row");
        assert!(
            row.detail
                .as_deref()
                .unwrap_or_default()
                .contains("MAX_TOTAL_EXPANDED_BYTES")
        );
        assert_eq!(
            row.path.as_deref(),
            Some(format!("e{per_entry_admitted}.txt").as_str()),
            "names the one entry that tripped it — the declared_count-th, past the ceiling"
        );
    }

    #[test]
    fn a_declared_ratio_past_the_ceiling_is_refused_before_decompressing() {
        // `flate2` on `1 MiB` of a single repeated byte compresses at a
        // ratio comfortably past `MAX_COMPRESSION_RATIO` (200:1) — real
        // DEFLATE, not a synthetic header lie, so this is the actual
        // pre-filter path, not a forged-header test.
        let payload = vec![b'Z'; 4 * 1024 * 1024];
        let bytes = build(&[("bomb.txt", &payload)]);
        let expansion = expand(&bytes, "parent-key");
        assert!(expansion.children.is_empty(), "{:?}", expansion.children);
        let row = expansion
            .coverage
            .iter()
            .find(|r| r.path.as_deref() == Some("bomb.txt"))
            .expect("a coverage row");
        assert_eq!(row.status, Coverage::Unsupported);
        assert!(
            row.detail
                .as_deref()
                .unwrap_or_default()
                .contains("MAX_COMPRESSION_RATIO"),
            "{row:?}"
        );
    }

    // ---------------------------------------------------------- nesting (c)

    #[test]
    fn a_nested_zip_recurses_with_a_chained_child_key() {
        let inner = build(&[("leaf.md", b"# leaf\n")]);
        let outer = build(&[("inner.zip", &inner)]);
        let expansion = expand(&outer, "root-key");
        assert_eq!(expansion.children.len(), 1);
        let nested_child = &expansion.children[0];
        assert!(nested_child.is_nested_archive);
        assert_eq!(nested_child.entry_adapter, None);
        let grandchildren = nested_child
            .nested
            .as_ref()
            .expect("depth 1 is within MAX_NESTING_DEPTH");
        assert_eq!(grandchildren.children.len(), 1);
        let leaf = &grandchildren.children[0];
        assert_eq!(leaf.relative_path, "leaf.md");
        assert_eq!(
            leaf.key,
            child_key(
                &nested_child.key,
                "leaf.md",
                &content_hash(b"# leaf\n"),
                super::super::text::MARKDOWN_EXTRACTOR
            ),
            "the grandchild's key chains through its own immediate parent (the nested archive), \
             not the root"
        );
    }

    #[test]
    fn nesting_past_the_depth_ceiling_still_admits_the_child_without_recursing() {
        // MAX_NESTING_DEPTH (2) allows a NESTED archive at depth 1 AND at
        // depth 2 to each be opened (the outer archive itself is depth 0) —
        // so this fixture needs THREE archive-within-archive levels to
        // actually reach the refused case: level3.zip, at prospective depth
        // 3, is the one that must be admitted as a child but not recursed.
        let level3 = build(&[("leaf.txt", b"leaf")]);
        let level2 = build(&[("level3.zip", &level3)]);
        let level1 = build(&[("level2.zip", &level2)]);
        let level0 = build(&[("level1.zip", &level1)]);

        let expansion = expand(&level0, "root-key");
        let one = &expansion.children[0]; // level1.zip, depth 1
        assert!(one.is_nested_archive);
        let inside_one = one.nested.as_ref().expect("depth 1 recurses");

        let two = &inside_one.children[0]; // level2.zip, depth 2
        assert!(two.is_nested_archive);
        let inside_two = two
            .nested
            .as_ref()
            .expect("depth 2 recurses (== MAX_NESTING_DEPTH)");

        let three = &inside_two.children[0]; // level3.zip, prospective depth 3
        assert!(three.is_nested_archive);
        assert!(
            three.nested.is_none(),
            "depth 3 exceeds MAX_NESTING_DEPTH and must not recurse — the entry is still \
             admitted as a child (checked below), just not opened further"
        );
        let refusal = inside_two
            .coverage
            .iter()
            .find(|r| {
                r.detail
                    .as_deref()
                    .unwrap_or_default()
                    .contains("MAX_NESTING_DEPTH")
            })
            .expect("a nesting-depth coverage row");
        assert_eq!(refusal.path.as_deref(), Some("level3.zip"));
    }

    // ------------------------------------------------- quine/overlap (e)

    /// The decisive fixture for the closed research item: two central
    /// directory records whose `data_start`/`compressed_size` name the
    /// IDENTICAL byte range in the underlying archive — built by writing one
    /// entry honestly through the crate's own writer, then hand-splicing a
    /// second central-directory record that duplicates the first one's
    /// offset instead of appending fresh local/compressed data of its own.
    #[test]
    fn overlapping_files_refuse_the_whole_archive_before_any_entry_opens() {
        let bytes = build(&[("only.txt", b"payload")]);

        // Locate the archive's one local-file-header and its one
        // central-directory record.
        let cd_signature = [0x50, 0x4B, 0x01, 0x02];
        let cd_start = bytes
            .windows(4)
            .position(|w| w == cd_signature)
            .expect("one central directory record");
        let cd_record = bytes[cd_start..].to_vec();
        let cd_record_len = 46 + 8; // fixed record + "only.txt".len() + "second.txt".len() delta handled below

        // Build a second central-directory record for a *different* name
        // ("second.txt") that otherwise byte-for-byte copies the first
        // record's fixed fields — crucially, the same relative local-header
        // offset (bytes 42..46) and the same compressed-size field (bytes
        // 20..24), so both entries claim the identical compressed-data
        // range.
        let name_len = u16::from_le_bytes([bytes[cd_start + 28], bytes[cd_start + 29]]) as usize;
        assert_eq!(&bytes[cd_start + 46..cd_start + 46 + name_len], b"only.txt");
        let mut second_record = cd_record[..46 + name_len].to_vec();
        let new_name = b"second.txt";
        // File-name-length field, offset 28..30 from the record's own start.
        second_record[28..30].copy_from_slice(&(new_name.len() as u16).to_le_bytes());
        second_record.truncate(46);
        second_record.extend_from_slice(new_name);

        // Splice: [everything up to and including the first CD record]
        // + [second CD record] + [end-of-central-directory record, count
        // fields bumped from 1 to 2, cd size grown, cd offset unchanged].
        let first_record_end = cd_start + 46 + name_len;
        let mut spliced = bytes[..first_record_end].to_vec();
        spliced.extend_from_slice(&second_record);
        let eocd_signature = [0x50, 0x4B, 0x05, 0x06];
        let eocd_start = bytes
            .windows(4)
            .position(|w| w == eocd_signature)
            .expect("end-of-central-directory record");
        let mut eocd = bytes[eocd_start..].to_vec();
        // Entry-count fields (this-disk and total), offsets 8..10 and
        // 10..12 from EOCD's own start: 1 -> 2.
        eocd[8..10].copy_from_slice(&2u16.to_le_bytes());
        eocd[10..12].copy_from_slice(&2u16.to_le_bytes());
        // Central-directory size field, offset 12..16: grows by the second
        // record's own length.
        let old_cd_size = u32::from_le_bytes([eocd[12], eocd[13], eocd[14], eocd[15]]);
        let new_cd_size = old_cd_size + second_record.len() as u32;
        eocd[12..16].copy_from_slice(&new_cd_size.to_le_bytes());
        spliced.extend_from_slice(&eocd);
        let _ = cd_record_len;

        // Sanity: `zip` itself must actually accept this as well-formed
        // enough to open and must actually see it as overlapping — this is
        // the decisive check named for register row 7, so a fixture bug
        // here (rather than the crate's own behaviour) must fail loudly,
        // not be silently absorbed by `expand`'s own archive-level refusal.
        let mut sanity = zip::ZipArchive::new(Cursor::new(&spliced)).expect(
            "the spliced archive must open — a fixture bug that makes it malformed would prove \
             nothing about overlap handling",
        );
        assert_eq!(
            sanity.len(),
            2,
            "both central-directory records must be visible"
        );
        assert!(
            sanity.has_overlapping_files().expect("overlap check runs"),
            "fixture sanity: the crate itself must observe the overlap for this test to mean \
             anything"
        );

        let expansion = expand(&spliced, "parent-key");
        assert!(
            expansion.children.is_empty(),
            "an overlapping archive must admit NO children: {:?}",
            expansion.children
        );
        assert_eq!(expansion.coverage.len(), 1, "{:?}", expansion.coverage);
        let row = &expansion.coverage[0];
        assert_eq!(row.path, None, "an archive-level refusal");
        assert_eq!(row.status, Coverage::Error);
        assert!(
            row.detail
                .as_deref()
                .unwrap_or_default()
                .contains("overlapping"),
            "{row:?}"
        );
    }

    // -------------------------------------------------------------- malformed

    #[test]
    fn a_genuinely_corrupt_archive_is_refused_not_panicked_on() {
        let expansion = expand(b"not a zip file at all", "parent-key");
        assert!(expansion.children.is_empty());
        assert_eq!(expansion.coverage.len(), 1);
        assert_eq!(expansion.coverage[0].status, Coverage::Error);
    }

    // ------------------------------------------------------------- collision key

    #[test]
    fn collision_key_normalises_before_case_folding() {
        assert_eq!(collision_key("café.txt"), collision_key("CAFÉ.TXT"));
        assert_eq!(collision_key("café.txt"), collision_key("cafe\u{0301}.txt"));
        assert_ne!(collision_key("a.txt"), collision_key("b.txt"));
    }

    #[test]
    fn ratio_trips_never_divides_by_zero_and_ignores_empty_entries() {
        assert!(!ratio_trips(0, 0));
        assert!(
            ratio_trips(1024, 0),
            "size claimed from zero compressed bytes"
        );
        assert!(!ratio_trips(100, 10));
        assert!(ratio_trips(100_000, 10));
    }
}

//! The Source identity spine (A1 §3–§4): what Atlas derived evidence hangs
//! off, and the vocabulary every later wave joins on.
//!
//! ```text
//! Source            source_id / source_kind / authority_class
//!   -> SourceGeneration   the exact observed world
//!        -> Resource      relative path + content hash (local knowledge)
//!             -> Unit     heading / section / symbol / row
//! ```
//!
//! Three rules from the contract are load-bearing here and are the reason
//! these are three separate types rather than one enum with a `String` tag:
//!
//! * **Authority is orthogonal to content and relevance** (§4). A Markdown
//!   file in an estate repository and a Markdown file in a local knowledge
//!   source are the same *content*; their [`AuthorityClass`] differs, and
//!   nothing may derive one from the other. "Knowledge" is not an authority
//!   level.
//! * **A generation is the exact observed world** (§3). For a Git source it
//!   is a commit SHA; for local knowledge it is a scan generation whose
//!   identity is [`SourceGeneration::content_key`] — a content hash of what
//!   was actually read, never a timestamp. Size and mtime are cheap *change
//!   candidates* only (§3's own wording); a durable reused extraction is
//!   keyed by content identity.
//! * **A cache key is content plus extractor** (F7). [`local_key`] is the
//!   whole of that rule for local knowledge: BLAKE3 of the bytes, plus the
//!   identity of the extractor that read them, and nothing else — never a
//!   second hash of the same bytes, never an mtime.
//!
//! # Reserved, deliberately behaviourless
//!
//! [`SourceKind::ExternalGit`] was named here ahead of its own producer: S4
//! Y5's [`crate::runtime::atlas::external_git`] is now that producer,
//! constructing and staging both this variant and
//! [`AuthorityClass::External`] for a fetched external Git source (S4 Y5
//! G6). It was still worth landing as a variant rather than a comment before
//! Y5 shipped, so the S4 seam was a compile-time obligation in the meantime —
//! a `match` that grows a third source kind failed to build rather than
//! silently treating an external source as an estate one; that obligation is
//! now discharged. `content_kind` (§4's `code | document | tabular | ...`
//! axis) is **not** modelled yet: X2 indexes exactly one content family, so
//! an enum with one reachable variant would be a promise, not a model (R1).
//! It lands with the wave that lands a second family.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Journal kind for the one compact summary a completed scan emits (F1).
///
/// Exactly one per completed scan — never one per file, never one per unit.
/// The authoritative trail stays journal-side (journal-is-truth) while the
/// unit-level detail lives in Atlas; a per-unit event would put the detail in
/// both places and make the journal the slower copy of a database.
pub const KIND_SOURCE_SCANNED: &str = "source.scanned";

/// Journal kind for the moment an estate-scoped scan is accepted (S6 scan
/// front door).
///
/// One per accepted `POST /v1/intelligence/scan`, appended **before** the
/// first source is touched, naming the scan id and every source the scan
/// undertook to cover. A scan that dies mid-flight therefore leaves a
/// started event with no completion — the journal says what was attempted,
/// which is exactly the honesty A1 §15 requires of coverage
/// ("missing capability is never represented as successful empty
/// evidence") and A1-01 requires of the journal.
pub const KIND_INTELLIGENCE_SCAN_STARTED: &str = "intelligence.scan.started";

/// Journal kind for the moment an estate-scoped scan finishes (S6 scan
/// front door).
///
/// One per completed scan, carrying the per-source outcome tally and the
/// wall-clock duration. This — not a row count in another command's
/// output — is what makes completion knowable: the event exists exactly
/// when every source the started event named has an outcome.
pub const KIND_INTELLIGENCE_SCAN_COMPLETED: &str = "intelligence.scan.completed";

/// §4's `source_kind` axis: how the bytes were acquired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// A repository the estate declares under `[[repo]]` and mounts at
    /// `repos/<name>`. Bytes come from Git objects at an admission-pinned
    /// SHA (X3a's plumbing), never from a working tree scan.
    EstateGit,
    /// A path the estate declares under `[[knowledge]]`: read-only evidence
    /// on the local filesystem, scanned rather than pinned (A1-03 — never a
    /// mount, and it grants no mutation authority).
    LocalKnowledge,
    /// A repository acquired from outside the estate: S4 Y5's
    /// [`crate::runtime::atlas::external_git::acquire_and_scan`] fetches it
    /// into a bare, no-working-tree host cache and stamps the resulting
    /// [`crate::runtime::atlas::scan::SourceScan`] with this kind (see this
    /// module's own doc).
    ExternalGit,
}

impl SourceKind {
    /// The stable wire/DB spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EstateGit => "estate_git",
            Self::LocalKnowledge => "local_knowledge",
            Self::ExternalGit => "external_git",
        }
    }

    /// The inverse of [`Self::as_str`], for reading a stored row back.
    /// `None` for a spelling this build does not know — a row written by a
    /// future version is reported as unreadable, never guessed at.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "estate_git" => Some(Self::EstateGit),
            "local_knowledge" => Some(Self::LocalKnowledge),
            "external_git" => Some(Self::ExternalGit),
            _ => None,
        }
    }
}

impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// §4's `authority_class` axis: what the estate may do with the bytes.
///
/// Orthogonal to [`SourceKind`] on purpose — the contract's own rule. The one
/// implication this build enforces is the negative one: nothing derives write
/// authority from a source being indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityClass {
    /// A declared repository mount: the estate cuts Work surfaces from it and
    /// Work mutates those surfaces.
    EstateMutable,
    /// Declared by the estate, read by the estate, mutated by nothing here —
    /// every `[[knowledge]]` source (A1-03).
    EstateReadonly,
    /// Acquired from outside the estate. Paired with
    /// [`SourceKind::ExternalGit`]; S4 Y5's
    /// [`crate::runtime::atlas::external_git::acquire_and_scan`] is now the
    /// one producer of it.
    External,
}

impl AuthorityClass {
    /// The stable wire/DB spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EstateMutable => "estate_mutable",
            Self::EstateReadonly => "estate_readonly",
            Self::External => "external",
        }
    }

    /// The inverse of [`Self::as_str`] — see [`SourceKind::parse`].
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "estate_mutable" => Some(Self::EstateMutable),
            "estate_readonly" => Some(Self::EstateReadonly),
            "external" => Some(Self::External),
            _ => None,
        }
    }
}

impl std::fmt::Display for AuthorityClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One exact observed world for one source (§3).
///
/// `id` is opaque and unique per observation; `content_key` is what makes two
/// observations *the same world*. Ruling §4's eviction rule is stated over
/// the latter, not the former: a re-scan that produces the same
/// `content_key` evicts nothing, because the source bytes did not change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceGeneration {
    /// Unique id for this observation (a ULID).
    pub id: String,
    /// The declared source this observed.
    pub source_name: String,
    /// How it was acquired.
    pub kind: SourceKind,
    /// What the estate may do with it.
    pub authority: AuthorityClass,
    /// Content identity of the whole generation — see [`generation_key`].
    pub content_key: String,
    /// When the observation completed (RFC3339 UTC, the journal's own shape).
    pub observed_at: String,
}

/// F7's local-knowledge cache key: BLAKE3 content hash **plus** extractor
/// identity, and nothing else.
///
/// Not a second hash of the bytes (the caller already has the content hash;
/// re-hashing megabytes to mix in a short string would be pure waste) and not
/// a function of any filesystem timestamp. Two files with identical bytes
/// read by the same extractor share a key by construction — which is the
/// point: the extraction is reusable across paths, across sources, and across
/// restarts.
pub fn local_key(content_hash: &str, extractor: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.atlas.local-key/v1\n");
    hasher.update(content_hash.as_bytes());
    hasher.update(b"\n");
    hasher.update(extractor.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// BLAKE3 hex digest of some bytes — the content half of [`local_key`].
pub fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// **F7's estate-git cache key**: the Git blob OID **plus** extractor
/// identity, and nothing else.
///
/// The rule this function exists to make unavoidable is the negative one:
/// *never a second hash of bytes Git already hashed*. A blob's OID is a
/// cryptographic digest of exactly those bytes, computed once, when the object
/// was written, by the tool that owns them. Re-hashing them with BLAKE3 to
/// produce a "content hash" would cost a full pass over every byte in the
/// repository on every scan, to arrive at a second name for a thing that
/// already has one. The OID is the content half; this composes it with the
/// extractor half.
///
/// The composition hashes two short strings — an OID and an extractor
/// identity, tens of bytes between them — which is not the thing the rule
/// forbids. What it buys is one fixed-width key whichever source kind produced
/// it, so `source.files.local_key` stays one column with one meaning.
///
/// **Domain-separated from [`local_key`] on purpose.** A blob OID and a BLAKE3
/// content hash are different lengths from different hash families, but they
/// are both "hex of some bytes", and two key spaces that could ever collide
/// would let a local-knowledge extraction be reused for a Git blob whose OID
/// happened to be spelled the same. The separator makes that impossible rather
/// than unlikely.
pub fn estate_git_key(blob_oid: &str, extractor: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.atlas.estate-git-key/v1\n");
    hasher.update(blob_oid.as_bytes());
    hasher.update(b"\n");
    hasher.update(extractor.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// **F7's child-resource key (G9)**: a container entry's key is composed from
/// its *immediate* parent's own key, never re-derived from a top-level
/// resource identity and never a bare content hash.
///
/// `parent_key` is whatever key produced the container this entry was read
/// out of: [`local_key`]/[`estate_git_key`] for a top-level archive, or
/// another call to [`child_key`] for an entry nested inside an entry that was
/// itself a container. That is the whole of "chained, not resolved-to-root"
/// (S4 Y3's research note, "Nested archive provenance must chain, not
/// collapse"): a grandchild's key already folds in its parent's key, which
/// already folded in *its* parent's, so the full ancestry is baked into one
/// fixed-width digest without this crate ever storing an explicit chain of
/// hops. Two different parents (two different archives, or two different
/// entries of the same archive) that happen to contain byte-identical
/// content produce two *different* child keys, because `parent_key` and
/// `entry_path` differ — which is correct: "this file lives at path `x`
/// inside archive `A`" and "this file lives at path `y` inside archive `B`"
/// are two different facts about the world even when the bytes match, and
/// collapsing them onto one key (as a bare `local_key(content_hash,
/// extractor)` call would) would erase that.
///
/// **Domain-separated from every other key in this module, on purpose** — the
/// same reasoning [`estate_git_key`]'s own doc gives for its own separator:
/// a bare hash of the same four inputs under a different label could, in
/// principle, collide with something computed elsewhere for another
/// purpose. Folding `parent_key` in as one of the four hashed inputs also
/// means a child key can never collide with a *top-level* [`local_key`]/
/// [`estate_git_key`] value by construction — those functions take exactly
/// two inputs (content hash, extractor) and never take a `parent_key`, so
/// their output never enters this function's own input transcript, and this
/// function's distinct prefix means its output never re-enters theirs
/// either.
///
/// `extractor` is the CHILD's own extractor identity — whatever downstream
/// adapter claims the entry's bytes (`text::extractor_for`,
/// `office::extractor_for`, this module's own container routing for a
/// nested archive, or a placeholder naming "no adapter claims this
/// extension yet") — never the container adapter that unpacked the entry.
/// Recording every archive-derived child under one constant "extractor =
/// zip" would erase exactly the information F7's own key is built to carry
/// (S4 Y3's research note, "`entry adapter` provenance must name the
/// specific extractor that ran on the child, not the container format").
pub fn child_key(
    parent_key: &str,
    entry_path: &str,
    content_hash: &str,
    extractor: &str,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.atlas.child-key/v1\n");
    hasher.update(parent_key.as_bytes());
    hasher.update(b"\n");
    hasher.update(entry_path.as_bytes());
    hasher.update(b"\n");
    hasher.update(content_hash.as_bytes());
    hasher.update(b"\n");
    hasher.update(extractor.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// The content identity of a Work overlay's *changed* paths: BLAKE3 over each
/// changed path and what the overlay recorded for it, in path order.
///
/// "What the overlay recorded" is the BLAKE3 hash of the working-tree bytes for
/// a path that has bytes, and a marker for one that does not — deleted,
/// unreadable, not a regular file. A path with no bytes still changed the
/// world, so it still has to move this digest;
/// [`crate::runtime::atlas::overlay`] owns those markers and the
/// argument for each.
///
/// The changed half only. Unchanged paths are the base tree's, by definition,
/// and are already identified by [`overlay_generation_key`]'s other input —
/// folding them in again would mean hashing a whole repository to describe a
/// two-file edit. Paths *excluded* at the acquisition boundary are folded in
/// nowhere at all, exactly as [`generation_key`] excludes them: what these keys
/// identify is the world evidence was derived from, and a denied path
/// contributed no bytes to it.
///
/// An empty map is a real answer, not a degenerate one: a Work surface that
/// has changed nothing has exactly this digest, and [`overlay_generation_key`]
/// then names the base plus "nothing", which is what a freshly cut surface
/// actually is.
pub fn overlay_digest(changed: &BTreeMap<String, String>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.atlas.overlay-digest/v1\n");
    for (path, hash) in changed {
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(hash.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// A Work overlay generation's identity: **the base commit SHA composed with
/// the overlay digest of what the surface changed.**
///
/// Both halves are load-bearing and neither is sufficient. The base alone
/// cannot distinguish two Works cut from the same commit that have edited
/// different files; the overlay digest alone cannot distinguish one edit
/// applied over two different bases, which is a different world with different
/// unchanged neighbours. Composing them means an overlay generation is the
/// same generation exactly when both the ground it stands on and the change it
/// makes are the same — the only condition under which reusing it is correct.
pub fn overlay_generation_key(base_sha: &str, overlay_digest: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.atlas.overlay-generation-key/v1\n");
    hasher.update(base_sha.as_bytes());
    hasher.update(b"\n");
    hasher.update(overlay_digest.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Content identity of a whole generation: a hash over every acquired
/// resource's `(relative path, content hash)` pair, in path order.
///
/// Order-independent by construction because the input is a [`BTreeMap`], so
/// two scans that walk the same tree in different directory order agree.
/// Excluded and unreadable paths are deliberately **not** folded in: what
/// this identifies is the world Atlas actually derived evidence from, and a
/// denied path contributed no bytes to it.
pub fn generation_key(resources: &BTreeMap<String, String>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sergeant.atlas.generation-key/v1\n");
    for (path, hash) in resources {
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(hash.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// F8's coverage vocabulary: what happened to one path, or to one whole
/// generation.
///
/// Every path the scanner *saw* leaves exactly one of these. There is no
/// eighth "silently skipped" state, and that is the whole design: F10's
/// secrets posture is only honest if an excluded byte is reported as
/// excluded rather than being absent from the record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Coverage {
    /// Seen by the walk. Every other status implies this one happened first;
    /// it is recorded on its own only for a container (a directory) whose
    /// children carry their own rows.
    Discovered,
    /// Bytes were read, an extractor ran, and units were written.
    Indexed,
    /// Refused at the acquisition boundary by the default deny set or a
    /// per-source `ignore` glob — **before** the bytes were read (F10).
    Excluded,
    /// Present but unreadable right now (permissions, vanished mid-scan, a
    /// symlink this build does not follow).
    Unavailable,
    /// Readable, but no extractor in this build claims it.
    Unsupported,
    /// An extractor was chosen and failed.
    Error,
    /// **Best-effort** (S4 Y6, G7/A1-06): the walker suspects this path is a
    /// cloud-sync placeholder — content declared present by the filesystem
    /// but not actually materialized locally (a OneDrive/Dropbox/iCloud
    /// "online-only" file) — and, on that suspicion, never opened it. A
    /// named coverage state precisely so this case is never `Indexed` with
    /// zero units, which is the "silently indexed as empty" acceptance item
    /// 4 forbids.
    ///
    /// **Not a certainty.** The signal ([`crate::runtime::atlas::scan`]'s
    /// `suspected_online_only`) is `st_blocks == 0` with `st_size > 0` — the
    /// same divergence an ordinary sparse file produces, so a ragged disk
    /// image or a punched-out log can be misclassified here too (a false
    /// positive); a placeholder a sync client fully materializes in `stat`
    /// answers before the byte is fetched is not caught at all (a false
    /// negative). The row's own `detail` says so every time, not just this
    /// doc comment.
    OnlineOnly,
    /// A whole generation's rows were removed — either because the source
    /// bytes changed (ruling §4's eviction) or because reconciliation found
    /// rows with no `source.scanned` summary (F1's crash window). Reported,
    /// never a silent gap.
    GenerationEvicted,
}

impl Coverage {
    /// The stable wire/DB spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::Indexed => "indexed",
            Self::Excluded => "excluded",
            Self::Unavailable => "unavailable",
            Self::Unsupported => "unsupported",
            Self::Error => "error",
            Self::OnlineOnly => "online_only",
            Self::GenerationEvicted => "generation_evicted",
        }
    }

    /// The inverse of [`Self::as_str`] — see [`SourceKind::parse`].
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.as_str() == text)
    }

    /// Every status, for callers that summarize counts over all of them.
    pub const ALL: &'static [Coverage] = &[
        Coverage::Discovered,
        Coverage::Indexed,
        Coverage::Excluded,
        Coverage::Unavailable,
        Coverage::Unsupported,
        Coverage::Error,
        Coverage::OnlineOnly,
        Coverage::GenerationEvicted,
    ];
}

impl std::fmt::Display for Coverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One coverage observation: what happened to one path (or, with `path`
/// unset, to the generation as a whole).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageRow {
    /// Path relative to the source root, or `None` for a generation-wide row.
    pub path: Option<String>,
    /// What happened.
    pub status: Coverage,
    /// Why, in one line — the deny pattern that matched, the io error, the
    /// extension nothing claims. Always populated for a status that is not
    /// self-explanatory.
    pub detail: Option<String>,
    /// Size in bytes as the filesystem reported it, when known. Present for
    /// [`Coverage::Excluded`] too: an excluded byte is counted, which is what
    /// makes "never silently absent" checkable rather than aspirational.
    pub bytes: Option<u64>,
}

/// The kind of structure unit an extractor produced (§3's `EvidenceUnit`).
///
/// # The Y2 decision: Office units stay Document/Section (S4, G3)
///
/// **Schema decision, recorded where a schema decision belongs — not an
/// implementation detail of whichever extractor happens to produce one.**
/// When Office document support (`.docx` via the adapter behind
/// [`crate::runtime::atlas::office`]) landed, the question this wave had to
/// answer was whether it needed a third variant (`OfficeDocument`/
/// `OfficeSection`, or similar) or whether the two that already exist were
/// enough.
///
/// **Decided: no new variant.** A `.docx`, normalized, decomposes into
/// exactly the same shape Markdown already has — one unit for the whole
/// resource, plus flat, heading-delimited sections tiling what came after
/// it — because `office::office_units` walks the normalizer's own heading
/// transitions the identical way `text::markdown_units` walks ATX heading
/// transitions.
/// Reusing `Document`/`Section` is R1/R2 (Ponytail): a format-specific pair
/// of variants would not model anything a *content-kind* axis (this module's
/// own doc already reserves one — "not modelled yet... lands with the wave
/// that lands a second family") does not already own, and adding them now
/// would be growing the enum to say "this section came from a `.docx`" when
/// the row's own `source.files.extractor` column already says that, more
/// precisely, via the extractor identity (F7) — a second place to encode the
/// same fact is a second place for it to drift from the first.
///
/// **What does NOT carry over from text/Markdown, and is the actual cost of
/// this decision:** a `Section` extracted from a `.docx` cannot claim a byte
/// range into the *original* resource the way a Markdown section can — the
/// original bytes are a compressed container the normalizer has already
/// unpacked and resolved by the time a heading block is visible, and no
/// position in the normalized model maps back to a byte offset in the ZIP
/// stream. So the
/// wire carrying these units ([`crate::runtime::atlas::worker::WorkerUnit`])
/// grew a `coordinate: Option<String>` field alongside its existing
/// `byte_start`/`byte_end` — `None` and real byte offsets for every text/
/// Markdown unit exactly as before, `Some` and `0`/`0` for an Office
/// section, naming a structural position (`block:<n>`) instead. `UnitKind`
/// itself did not need to grow to carry that asymmetry; the wire type did,
/// in one small, additive way. See [`crate::runtime::atlas::office`]'s own
/// module doc for the full argument, including why a spreadsheet format (not
/// adopted this wave) must never claim a write-back cell coordinate through
/// this same field.
///
/// # The Y4 decision: a mail message's two bodies are two `Document` units
/// (S4, G4)
///
/// The same reuse call, for a different asymmetry: a `.eml` message can
/// carry up to two independent bodies (A1 §6.5's "text/html body"), and
/// `UnitKind` grows no third variant to say so — both are `Document`
/// (each is a whole, independently-meaningful rendering of the same
/// resource, not a span *within* one), distinguished on the wire by
/// [`crate::runtime::atlas::worker::WorkerUnit::coordinate`]
/// (`"text-body"`/`"html-body"`) exactly the way an Office section's
/// coordinate distinguishes it from the whole-document unit above. Neither
/// carries a real `byte_start`/`byte_end` either, for the identical
/// reason an Office section does not: a body already decoded past its
/// `Content-Transfer-Encoding` has no byte-exact back-reference into the
/// original wire bytes. See [`crate::runtime::atlas::mail`]'s own module
/// doc for the adapter's full argument, including the two `mail-parser`
/// caveats this coordinate distinction exists to keep honest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitKind {
    /// The whole resource, as one unit.
    Document,
    /// A heading-delimited span within a document.
    Section,
}

impl UnitKind {
    /// The stable wire/DB spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Section => "section",
        }
    }

    /// The inverse of [`Self::as_str`] — see [`SourceKind::parse`].
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "document" => Some(Self::Document),
            "section" => Some(Self::Section),
            _ => None,
        }
    }
}

impl std::fmt::Display for UnitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F7, stated as a test: the key moves when *either* input moves, and the
    /// content hash alone is never the key.
    #[test]
    fn a_local_key_is_content_and_extractor_and_neither_alone() {
        let a = content_hash(b"# title\n");
        let b = content_hash(b"# other\n");
        assert_ne!(a, b);

        assert_eq!(local_key(&a, "markdown/v1"), local_key(&a, "markdown/v1"));
        // Same bytes, different extractor -> different reusable extraction.
        assert_ne!(local_key(&a, "markdown/v1"), local_key(&a, "text/v1"));
        // Different bytes, same extractor -> different extraction.
        assert_ne!(local_key(&a, "markdown/v1"), local_key(&b, "markdown/v1"));
        // And the key is never just the content hash handed back.
        assert_ne!(local_key(&a, "markdown/v1"), a);
    }

    /// F7/G9: a child key moves when ANY of its four inputs move, and it is
    /// never reducible to a bare content-hash-plus-extractor call — the
    /// property that distinguishes it from [`local_key`].
    #[test]
    fn a_child_key_is_parent_and_path_and_content_and_extractor_and_never_content_alone() {
        let parent = local_key(&content_hash(b"archive bytes"), "zip/v1");
        let entry_a = content_hash(b"hello");
        let entry_b = content_hash(b"world");

        let base = child_key(&parent, "notes/a.md", &entry_a, "markdown/v1");
        assert_eq!(
            base,
            child_key(&parent, "notes/a.md", &entry_a, "markdown/v1"),
            "deterministic: same four inputs, same key"
        );
        assert_ne!(
            base,
            child_key(&parent, "notes/b.md", &entry_a, "markdown/v1"),
            "a different entry path must move the key even with identical content"
        );
        assert_ne!(
            base,
            child_key(&parent, "notes/a.md", &entry_b, "markdown/v1"),
            "different content must move the key even at the identical path"
        );
        assert_ne!(
            base,
            child_key(&parent, "notes/a.md", &entry_a, "text/v1"),
            "a different child extractor must move the key"
        );
        let other_parent = local_key(&content_hash(b"a different archive"), "zip/v1");
        assert_ne!(
            base,
            child_key(&other_parent, "notes/a.md", &entry_a, "markdown/v1"),
            "a different parent must move the key even with an identical entry"
        );

        // And never the same as calling `local_key` directly on the entry's
        // own content hash + extractor — the mistake the research note names
        // ("keying purely on content hash... is wrong for the resource's
        // identity/provenance").
        assert_ne!(base, local_key(&entry_a, "markdown/v1"));
    }

    /// G9's own words: chained, not resolved-to-root. A grandchild's key is
    /// computed by feeding the CHILD's own key back in as `parent_key`, and
    /// that must differ from naively keying the grandchild against the
    /// top-level archive's key directly (which would flatten a two-level
    /// nesting into one hop and lose which nested archive it came through).
    #[test]
    fn a_grandchild_key_chains_through_its_own_parent_not_the_root() {
        let root = local_key(&content_hash(b"outer.zip bytes"), "zip/v1");
        let nested_archive_hash = content_hash(b"inner.zip bytes");
        let nested_key = child_key(&root, "inner.zip", &nested_archive_hash, "zip/v1");

        let grandchild_hash = content_hash(b"leaf content");
        let chained = child_key(&nested_key, "leaf.md", &grandchild_hash, "markdown/v1");
        let flattened_to_root = child_key(&root, "leaf.md", &grandchild_hash, "markdown/v1");
        assert_ne!(
            chained, flattened_to_root,
            "keying a grandchild against the ROOT archive's key must differ from keying it \
             against its own immediate parent (the nested archive) — collapsing the chain would \
             make two different nested archives that happen to share a leaf path and content \
             collide"
        );
    }

    /// The domain-separation prefix is not decoration: without it, a caller
    /// that hashed the same two strings for another purpose would collide
    /// with a cache key.
    #[test]
    fn a_local_key_is_domain_separated_from_a_bare_hash_of_the_same_strings() {
        let hash = content_hash(b"x");
        let naive = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(hash.as_bytes());
            hasher.update(b"\n");
            hasher.update(b"markdown/v1");
            hasher.finalize().to_hex().to_string()
        };
        assert_ne!(local_key(&hash, "markdown/v1"), naive);
    }

    /// Ruling §4's eviction rule is stated over content, so the key has to be
    /// a pure function of content: same resources in, same key out —
    /// whatever order the walk found them in.
    #[test]
    fn a_generation_key_is_content_only_and_order_independent() {
        let mut one = BTreeMap::new();
        one.insert("b.md".to_string(), content_hash(b"b"));
        one.insert("a.md".to_string(), content_hash(b"a"));
        let mut two = BTreeMap::new();
        two.insert("a.md".to_string(), content_hash(b"a"));
        two.insert("b.md".to_string(), content_hash(b"b"));
        assert_eq!(generation_key(&one), generation_key(&two));

        // A byte change anywhere moves it.
        let mut three = two.clone();
        three.insert("a.md".to_string(), content_hash(b"a!"));
        assert_ne!(generation_key(&two), generation_key(&three));

        // So does a rename, even with identical bytes: provenance is part of
        // the observed world.
        let mut four = BTreeMap::new();
        four.insert("a.md".to_string(), content_hash(b"a"));
        four.insert("c.md".to_string(), content_hash(b"b"));
        assert_ne!(generation_key(&two), generation_key(&four));

        // An empty source has a stable identity of its own, not the empty
        // string — "nothing here" is an observation, not an absent one.
        assert_eq!(generation_key(&BTreeMap::new()).len(), 64);
    }

    /// The three axes are spelled once, here, and every DB/wire consumer
    /// reads them from these functions. A silent rename is a silent schema
    /// migration.
    #[test]
    fn the_wire_spellings_are_pinned() {
        assert_eq!(SourceKind::LocalKnowledge.as_str(), "local_knowledge");
        assert_eq!(SourceKind::EstateGit.as_str(), "estate_git");
        assert_eq!(SourceKind::ExternalGit.as_str(), "external_git");
        assert_eq!(AuthorityClass::EstateReadonly.as_str(), "estate_readonly");
        assert_eq!(UnitKind::Section.as_str(), "section");
        assert_eq!(KIND_SOURCE_SCANNED, "source.scanned");
        assert_eq!(KIND_INTELLIGENCE_SCAN_STARTED, "intelligence.scan.started");
        assert_eq!(
            KIND_INTELLIGENCE_SCAN_COMPLETED,
            "intelligence.scan.completed"
        );
        for kind in [
            SourceKind::EstateGit,
            SourceKind::LocalKnowledge,
            SourceKind::ExternalGit,
        ] {
            assert_eq!(SourceKind::parse(kind.as_str()), Some(kind));
        }
        for status in Coverage::ALL {
            assert_eq!(Coverage::parse(status.as_str()), Some(*status));
        }
        assert_eq!(SourceKind::parse("something_new"), None);
        assert_eq!(
            UnitKind::parse(UnitKind::Section.as_str()),
            Some(UnitKind::Section)
        );
        assert_eq!(
            AuthorityClass::parse(AuthorityClass::External.as_str()),
            Some(AuthorityClass::External)
        );
        let spellings: Vec<&str> = Coverage::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(
            spellings,
            [
                "discovered",
                "indexed",
                "excluded",
                "unavailable",
                "unsupported",
                "error",
                "online_only",
                "generation_evicted",
            ]
        );
    }

    /// `serde` sees the same spellings the DB does — one vocabulary, not two
    /// that drift.
    #[test]
    fn serde_and_as_str_agree() {
        for kind in [
            SourceKind::EstateGit,
            SourceKind::LocalKnowledge,
            SourceKind::ExternalGit,
        ] {
            let json = serde_json::to_string(&kind).expect("serialize");
            assert_eq!(json, format!("\"{}\"", kind.as_str()));
        }
        for status in Coverage::ALL {
            let json = serde_json::to_string(status).expect("serialize");
            assert_eq!(json, format!("\"{}\"", status.as_str()));
        }
    }
}

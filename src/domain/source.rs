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
//! [`SourceKind::ExternalGit`] is named here and produced by nothing: S4 owns
//! external-Git acquisition (the ratified re-cut's adapters/external-git
//! items). It exists as a variant rather than a comment so the S4 seam is a
//! compile-time obligation — a `match` that grows a third source kind fails
//! to build rather than silently treating an external source as an estate
//! one. `content_kind` (§4's `code | document | tabular | ...` axis) is
//! **not** modelled yet: X2 indexes exactly one content family, so an enum
//! with one reachable variant would be a promise, not a model (R1). It lands
//! with the wave that lands a second family.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Journal kind for the one compact summary a completed scan emits (F1).
///
/// Exactly one per completed scan — never one per file, never one per unit.
/// The authoritative trail stays journal-side (journal-is-truth) while the
/// unit-level detail lives in Atlas; a per-unit event would put the detail in
/// both places and make the journal the slower copy of a database.
pub const KIND_SOURCE_SCANNED: &str = "source.scanned";

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
    /// A repository acquired from outside the estate. **S4 seam: nothing in
    /// this build produces one.** Named so the match arms that will need it
    /// are structurally visible now (see this module's own doc).
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
    /// Acquired from outside the estate. Reserved with
    /// [`SourceKind::ExternalGit`]; nothing produces it yet.
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

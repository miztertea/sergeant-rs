//! The local-knowledge scanner: Atlas's first real `source.*` writer.
//!
//! Walks one declared `[[knowledge]]` root and produces **plain Rust** — a
//! [`SourceScan`] of files, units and coverage rows. It opens files; it names
//! no database, no journal and no daemon state, and it imports neither
//! [`super::db`] nor the journal. The glue that turns a [`SourceScan`] into
//! rows and an event is its own small module, [`super::record`] — F6's
//! "DB-touching glue kept thin and separately reviewable", made structural
//! rather than aspirational: the dependency runs `deny`/`text` -> `scan` ->
//! `db` -> `record`, in one direction, with no cycle to hide a shortcut in.
//!
//! # The three rules this module exists to hold
//!
//! * **F10 — nothing denied is ever opened.** The verdict
//!   ([`super::deny`]) is a pure function of the path and is asked *before*
//!   the file is opened, and before a directory is descended into. Excluded
//!   bytes are counted, with the matching pattern named, so an exclusion is
//!   visible rather than absent.
//! * **F7 — keys are content plus extractor.** A resource's identity is
//!   BLAKE3 of its bytes; its extraction's identity is that hash plus the
//!   extractor's own versioned name
//!   ([`local_key`](crate::domain::source::local_key)). `mtime` is recorded
//!   as a *change hint* and is part of no key, so touching a file changes
//!   nothing derived and editing one changes everything derived from it.
//! * **F8 — every path seen leaves exactly one coverage row.** Indexed,
//!   excluded, unavailable, unsupported, online-only or error: there is no
//!   seventh outcome where a path is silently not mentioned. One row per
//!   *path*, not one per extractor: a resource two extractors claimed still
//!   has one row, whose `detail` names both, and whose status is `error` if
//!   either failed.
//!
//! # Online-only / cloud-placeholder detection (S4 Y6, G7/A1-06)
//!
//! [`suspected_online_only`] runs on the [`std::fs::Metadata`] the walker
//! already fetched for every entry (`symlink_metadata`, one `lstat` per
//! path) — never a second syscall, and specifically never `open()`. That is
//! not an optimization; it is the whole safety property. A1 §7 forbids
//! auto-hydrating a library, and on several cloud-sync filesystems `open()`
//! is exactly what triggers a hydration download of a placeholder's real
//! bytes. So classification happens strictly before [`Walk::file`] or
//! [`Walk::dataset`] reads a single byte, using metadata that was going to be
//! fetched regardless (every other coverage decision needs it too), and a
//! file this check flags is never opened at all — the byte-read boundary
//! ([`std::fs::read`] in `file`, [`hash_file`] in `dataset`) is downstream of
//! the check, not upstream.
//!
//! **The permitted syscall set is exactly `lstat`/`stat`** (via
//! `std::fs::symlink_metadata`, already called). `listxattr`/`getxattr` were
//! investigated as a second signal (candidates the wave's own brief named)
//! and deliberately not adopted: this build's targets (Linux, macOS —
//! `Cargo.toml` names no Windows-only dependency and CI runs neither) have no
//! single documented, verifiable-via-real-documentation xattr convention for
//! a cloud-sync placeholder the way Windows' NTFS reparse-point attribute is
//! documented for OneDrive Files On-Demand. Guessing one would repeat
//! exactly the mistake S4's own record already made once with
//! `enclosed_name`'s assumed guarantees — so the heuristic stays to the one
//! signal this wave could actually verify.
//!
//! # The shared extraction (X3b)
//!
//! Three walks acquire bytes by three different routes — this module's
//! filesystem walk, [`super::git`]'s object-store read, and
//! [`super::overlay`]'s Work surface. They all extract through the same two
//! functions here: [`claims_for`], which decides from a path alone what claims
//! it, and [`extract_resource`], which runs every claiming extractor over the
//! bytes. Three copies of that would be three ways for F7's premise —
//! identical bytes plus an identical extractor identity are one extraction —
//! to stop being true.
//!
//! **A second, disjoint pair does the same job for a supervised-worker
//! adapter (S4 Y8).** [`worker_extractor_for`] is [`claims_for`]'s own
//! sibling for Office/ZIP/mail — extensions that are never valid UTF-8 text,
//! so they were never reaching `claims_for`'s pipeline — and
//! [`dispatch_worker_resource`] is [`extract_resource`]'s sibling, shared by
//! this module's own walk and [`super::git`]'s. [`super::overlay`] uses
//! neither half of this second pair: a Work surface's own edited files stay
//! worker-free this wave (brief-y8-adapter-dispatch.md names only this
//! module's and `super::git`'s walks), the same default
//! [`scan_local_knowledge`]'s own doc states for its own callers.
//!
//! # F1's crash window
//!
//! [`super::record::scan_and_record`] is the whole of the coupling rule, in
//! three steps that are worth reading in order — its own doc explains why the
//! order is the only safe one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::domain::estate::KnowledgeSpec;
use crate::domain::event::rfc3339_utc_now;
use crate::domain::source::{
    AuthorityClass, Coverage, CoverageRow, SourceKind, UnitKind, child_key, content_hash,
    estate_git_key, generation_key, local_key,
};
use crate::runtime::atlas::deny::{AcquisitionFilter, BadPattern, Verdict};
use crate::runtime::atlas::syntax::{SyntaxLanguage, language_for};
use crate::runtime::atlas::tabular::{ContextFields, ScannedDataset, format_for, reader_identity};
use crate::runtime::atlas::text::{
    MARKDOWN_EXTRACTOR, TEXT_EXTRACTOR, as_text, extractor_for, markdown_units, plain_units,
};
use crate::runtime::atlas::worker::{
    WorkerIdentity, WorkerOutcome, WorkerRuntime, WorkerSpawn, run_worker,
};

/// The largest resource this build reads into memory to extract.
///
/// **Declared, not measured** — and said plainly rather than dressed up as a
/// tuned figure. It is a refusal ceiling on a whole-file read whose result is
/// stored as a single DuckDB `TEXT` value and can be returned whole; 4 MiB of
/// prose is on the order of a thousand printed pages, which is far past
/// anything a heading-sectioned document plausibly is. A file above it is
/// reported `unsupported` **naming this bound**, never silently skipped, so
/// the day a real corpus argues for a different number the evidence for it is
/// already in the coverage table.
pub const MAX_RESOURCE_BYTES: u64 = 4 * 1024 * 1024;

/// The largest tabular dataset this build will register (X4).
///
/// **Declared, not measured**, and deliberately much larger than
/// [`MAX_RESOURCE_BYTES`] because it bounds a different thing. A document is
/// read whole into memory and stored as one `TEXT` value; a dataset is never
/// loaded at all — it is hashed in a streaming pass and then read *in place*
/// through a row-capped query. So the only cost this ceiling actually bounds
/// is the streaming hash, and 512 MiB of it is seconds, not minutes. A file
/// above it is reported `unsupported` **naming this bound**, never silently
/// skipped, so the day a real corpus argues for a different number the
/// evidence for it is already in the coverage table.
pub const MAX_DATASET_BYTES: u64 = 512 * 1024 * 1024;

/// Bytes per read of the streaming hash of a dataset. A page-sized multiple,
/// chosen for no other reason and load-bearing for nothing.
const DATASET_HASH_CHUNK: usize = 64 * 1024;

/// The coverage detail for a dataset a walk cannot read in place (X4).
///
/// An estate-git or Work-overlay walk reaches bytes through Git's object
/// store, and a blob has no path DuckDB can open. Reading it would mean
/// materializing a copy — which is precisely what "read in place" exists to
/// avoid, and what a knowledge source's read-only posture makes gratuitous.
/// So it is reported `unsupported`, naming the reason, and the wave that has a
/// consumer for repository-resident datasets can decide what to do about it.
pub const DATASET_NO_ROOT: &str =
    "tabular datasets are read in place, and this source's bytes have no path to read in place";

/// The coverage detail for a CONTAINER CHILD claimed by [`format_for`] on a
/// walk that registers no datasets (S5 W7 F-SF-01).
///
/// Not [`DATASET_NO_ROOT`], which would be a false statement here: a child
/// dataset does not need a root — it carries its own bytes
/// ([`ScannedDataset::content`]). This says the true thing instead, which is
/// that the walk it arrived on registers no dataset rows for a loose file of
/// that extension either, so its child gets the same answer. That is what
/// route-equivalence means, and it is the honest reason rather than a
/// borrowed one.
pub const DATASET_CHILD_NOT_REGISTERED: &str = "a container child claimed by the tabular routing table is registered as a dataset only by a \
     filesystem source walk; this generation's own resources came from a Git object store, where \
     a loose file of the same extension is likewise not registered";

/// [`Coverage::OnlineOnly`]'s own detail text — the honesty the acceptance
/// item's own scope names as the deliverable, not a caveat: stated as a
/// signal, never a certainty, every time the row is written rather than only
/// in a doc comment a reader of the coverage table never sees.
pub const ONLINE_ONLY_DETAIL: &str = "best-effort (S4 Y6, G7/A1-06): zero allocated blocks with a non-zero reported size \
     (st_blocks == 0, st_size > 0) — the signature of an online-only/cloud-sync placeholder, \
     but also of an ordinary sparse file, so this can be a false positive; a placeholder a sync \
     client reports with full block allocation before the byte is fetched is not caught at all, \
     so this can also be a false negative. Content was not opened to check further, because on \
     several cloud filesystems open() is what triggers a hydration download (A1 §7 forbids \
     auto-hydrating a library)";

/// The heuristic itself (S4 Y6, G7/A1-06): does this entry's already-fetched
/// metadata look like a cloud-sync placeholder rather than ordinary content?
///
/// **Reads no more than the [`std::fs::Metadata`] the caller already has.**
/// No `open()`, no second `stat`, nothing beyond the two POSIX fields this
/// module's own doc explains: `st_blocks == 0` (nothing allocated on disk)
/// with `st_size > 0` (the filesystem still reports real content). Verified
/// against the Rust standard library's own documentation for
/// [`std::os::unix::fs::MetadataExt::blocks`], which states a file with
/// holes reports fewer blocks than its size would otherwise imply — the
/// exact divergence this checks for, taken to its extreme (zero blocks at
/// all).
///
/// A genuinely empty file (`st_size == 0`) is never flagged: it has nothing
/// to have failed to fetch, and calling it a placeholder would be the
/// opposite dishonesty from the one this heuristic exists to fix.
///
/// `false` unconditionally off `cfg(unix)` — this build's targets are Linux
/// and macOS (`Cargo.toml` names no Windows-only dependency, and CI runs
/// neither), and `MetadataExt` is a Unix-only trait; a platform this heuristic
/// has not been verified against gets the honest "not detected" answer
/// rather than a guess.
#[cfg(unix)]
fn suspected_online_only(meta: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    meta.size() > 0 && meta.blocks() == 0
}

/// See the `cfg(unix)` twin's doc — unverified off Unix, so honestly `false`.
#[cfg(not(unix))]
fn suspected_online_only(_meta: &std::fs::Metadata) -> bool {
    false
}

/// One source to scan: the manifest's declaration, resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeSource {
    /// Declared name — the source coordinate every derived row carries.
    pub name: String,
    /// Absolute path to the source root.
    pub root: PathBuf,
    /// Per-source ignore globs, extending the built-in deny set.
    pub ignore: Vec<String>,
    /// **F10a**: this source's declared tabular column allowlist. Default
    /// none, which is the refusal — see [`ContextFields`].
    pub context_fields: ContextFields,
}

impl From<&KnowledgeSpec> for KnowledgeSource {
    fn from(spec: &KnowledgeSpec) -> Self {
        Self {
            name: spec.name.clone(),
            root: spec.path.clone(),
            ignore: spec.ignore.clone(),
            context_fields: ContextFields::declared(&spec.context_fields),
        }
    }
}

/// One extracted unit, ready to become a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedUnit {
    /// Position within its file's unit list — stable for identical bytes and
    /// extractor, which is what makes a unit addressable across generations.
    pub ordinal: u64,
    /// Whole document, or a heading-delimited section.
    pub kind: UnitKind,
    /// Heading depth, for a section under a heading.
    pub heading_level: Option<u8>,
    /// Heading text, when there is one.
    pub title: Option<String>,
    /// Offset into the **original** file bytes.
    pub byte_start: u64,
    /// End offset into the original file bytes, exclusive.
    pub byte_end: u64,
    /// The normalizer's own native address for this unit, when
    /// `byte_start`/`byte_end` cannot say where it is (A2 §9's *"native
    /// coordinate"*).
    ///
    /// `None` for every in-process text/Markdown unit, whose byte span is
    /// the address. `Some` for a unit an adapter extracted out of a
    /// container it had to decode first — an Office section's `block:<n>`,
    /// a mail body's `text-body`/`html-body` — where no offset into the
    /// original bytes corresponds to the unit. The full contract the value
    /// satisfies is [`crate::runtime::atlas::worker::WorkerUnit::coordinate`]'s.
    pub coordinate: Option<String>,
    /// The unit's own text, exactly as it appears in the original.
    pub text: String,
}

/// The only edge kind this build derives (X3b).
///
/// One constant rather than an enum because there is exactly one, and an enum
/// with one reachable variant is the promise R1 says not to make. A second
/// edge kind — a call, a containment, a reference — needs resolution this
/// build does not do (A1-09), so it arrives with the wave that can actually
/// derive it.
pub const EDGE_IMPORT: &str = "import";

/// One syntax-derived symbol *site*, positioned in the original bytes (X3b).
///
/// A site, not a symbol: two `count` methods in one file are two of these,
/// and they roll up to one entry in the symbol index. What makes them one
/// symbol is `(language, label, name)`, and nothing here claims they are the
/// same *definition* — that would be resolution (A1-09).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedSymbol {
    /// Position within its file's symbol list, in document order.
    pub ordinal: u64,
    /// What the grammar called it — `function`, `struct`, `heading`.
    pub label: &'static str,
    /// The name as written. Unresolved, unqualified, not deduplicated.
    pub name: String,
    /// Offset into the **original** file bytes.
    pub byte_start: u64,
    /// End offset into the original file bytes, exclusive.
    pub byte_end: u64,
}

/// One syntax-derived edge out of a file (X3b).
///
/// Today that is exactly one shape: an import, whose `target` is the text the
/// file wrote. Unresolved by construction — `./lib/common.sh` is a string, not
/// a path this build claims to have found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedEdge {
    /// Position within its file's edge list, in document order.
    pub ordinal: u64,
    /// The edge's syntax-derived kind — [`EDGE_IMPORT`] today.
    pub kind: &'static str,
    /// What the file named, exactly as written.
    pub target: String,
    /// Offset into the **original** file bytes.
    pub byte_start: u64,
    /// End offset into the original file bytes, exclusive.
    pub byte_end: u64,
}

/// The syntax extraction of one resource: a **second** extraction of the same
/// bytes, with its own extractor identity and its own F7 key (X3b).
///
/// Deliberately not folded into [`ScannedFile`]'s own `extractor`/`local_key`.
/// F7 keys a derived row on content identity *plus extractor identity*, so a
/// Markdown file read by both the structure extractor and the Markdown grammar
/// is two extractions with two keys — never one extraction that could have
/// been done two ways. Collapsing them would mean bumping the Markdown grammar
/// invalidated every structure unit too.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedSyntax {
    /// The grammar's language name, as coverage and rows spell it.
    pub language: &'static str,
    /// The grammar's versioned extractor identity (F7's second key input).
    pub extractor: String,
    /// The reusable key for *this* extraction: content identity composed with
    /// [`Self::extractor`], in whichever key space the source uses
    /// ([`KeySpace`]).
    pub syntax_key: String,
    /// Symbol sites, in document order.
    pub symbols: Vec<ScannedSymbol>,
    /// Edges out of this file, in document order.
    pub edges: Vec<ScannedEdge>,
}

/// One acquired resource and everything derived from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedFile {
    /// Path relative to the source root, `/`-separated.
    pub relative_path: String,
    /// **BLAKE3 hex of the raw bytes, always this one algorithm regardless
    /// of source kind** — the projection-identity merge key (bug 2,
    /// `brief-search-three-bugs.md`). Deliberately a *different* field from
    /// [`Self::content_hash`], which stays F7's own per-source-kind key
    /// (the estate-git half is Git's own blob OID, a different hash and a
    /// different byte framing — `sha1("blob <len>\0<content>")` vs plain
    /// `blake3(content)` — so two byte-identical files acquired one via Git
    /// and one via the filesystem carry two different `content_hash`
    /// strings and could never merge on it). This field is what lets two
    /// sources whose `identity_root`s are genuinely different directories
    /// (a `repos/` clone and the live working tree it mirrors — two really
    /// different inodes `canonical_identity`'s own path-canonicalization
    /// cannot merge) still resolve to the same file identity: it is the
    /// same reused algorithm [`crate::domain::source::content_hash`]
    /// and [`crate::runtime::blob::BlobStore`] already compute
    /// (`blake3::hash(bytes)`), applied uniformly rather than invented anew
    /// (R2).
    pub content_digest: String,
    /// BLAKE3 hex of the file's bytes (F7's content half).
    pub content_hash: String,
    /// Identity of the extractor that read it (F7's other half).
    pub extractor: String,
    /// The reusable extraction key: [`local_key`] of the two above.
    pub local_key: String,
    /// Size in bytes, as read.
    pub byte_len: u64,
    /// Modification time in Unix milliseconds, when the filesystem offered
    /// one. **A change hint only** (A1 §3): part of no key, consulted by no
    /// reuse decision, recorded because "what looked different" is useful
    /// evidence when a scan is being explained.
    pub mtime_millis: Option<i64>,
    /// Units extracted from it, in document order.
    pub units: Vec<ScannedUnit>,
    /// The syntax extraction, when a grammar in this build claims the path
    /// **and parsed it** (X3b). `None` for a path no grammar claims, and
    /// `None` for one whose parse failed — a failed parse carries no partial
    /// symbol list, and the failure is a coverage fact
    /// ([`Coverage::Error`]) rather than a shorter list nothing can
    /// distinguish from a complete one.
    pub syntax: Option<ScannedSyntax>,
    /// `Some` when this resource is a container CHILD — an archive entry, a
    /// mail attachment, or a descendant of either (S5 W7) — naming the parent
    /// coordinate A1 §6.6 requires an expanded entry to preserve. `None` for
    /// every resource acquired directly from a source root, a Git tree or a
    /// Work overlay.
    pub parent: Option<ChildProvenance>,
}

/// Everything one completed walk observed. Plain data — no handle, no
/// connection, no borrow of anything live.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceScan {
    /// The declared source.
    pub source_name: String,
    /// How it was acquired.
    pub kind: SourceKind,
    /// What the estate may do with it.
    pub authority: AuthorityClass,
    /// Content identity of the whole generation (see
    /// [`generation_key`]) — the value ruling §4's eviction rule is stated
    /// over.
    pub content_key: String,
    /// The source's own revision identity, when it has one: the pinned commit
    /// SHA for an [`SourceKind::EstateGit`] scan, `None` for a filesystem walk
    /// that has no such thing.
    ///
    /// Deliberately *not* the same field as [`Self::content_key`], and not a
    /// substitute for it. The content key answers "is this the same world?"
    /// and drives ruling §4's eviction; the revision answers "which commit was
    /// this?" and is provenance a reader needs but no comparison uses — two
    /// commits with identical trees are the same world, and folding the commit
    /// into the key would evict a generation that changed no source byte.
    pub revision: Option<String>,
    /// When the walk finished (RFC3339 UTC).
    pub observed_at: String,
    /// Acquired resources, in path order.
    pub files: Vec<ScannedFile>,
    /// One row per path seen, in path order.
    pub coverage: Vec<CoverageRow>,
    /// Distinct extractor identities that ran — carried into the journal
    /// summary, because "which parser produced this?" is one of A1 §3's four
    /// provenance questions.
    pub extractors: BTreeSet<String>,
    /// Tabular datasets this walk registered, in path order (X4).
    ///
    /// Registered, **not read**: a dataset's bytes never enter this struct.
    /// The walk hashes them (streamed, never held) and records where they are;
    /// the reading happens in place, later, inside [`super::db`], through a
    /// canned parameterized query against the file the estate already has.
    /// That is why this is a separate field from [`Self::files`] rather than a
    /// [`ScannedFile`] with a different extractor: a `ScannedFile` carries the
    /// units extracted from its bytes, and a dataset has none of that here.
    pub datasets: Vec<ScannedDataset>,
    /// Absolute path of the source root, for a walk that had one.
    ///
    /// `None` for a walk whose bytes came out of an object store
    /// ([`super::git`]) rather than off a filesystem. It is the *only* thing
    /// that makes an in-place dataset read possible, so its absence is exactly
    /// why an estate-git dataset is reported `unsupported` rather than
    /// silently skipped (see [`DATASET_NO_ROOT`]).
    pub root: Option<PathBuf>,
    /// **Projection-identity wave (S6): the absolute path each acquired
    /// file's canonical identity resolves against** (`root.join(relative_path)`,
    /// canonicalized — symlinks resolve to their target).
    ///
    /// Deliberately a *different* field from [`Self::root`], not a reuse of
    /// it: `root`'s only job is gating an in-place dataset read
    /// ([`DATASET_NO_ROOT`]), and an estate-git scan's bytes coming out of
    /// the object store rather than the filesystem (`root: None`) says
    /// nothing about whether the *mount* has a stable absolute path a
    /// canonical identity can resolve against — it does
    /// ([`crate::runtime::atlas::git::EstateGitSource::mount`]). Populating
    /// `root` from `mount` to get this identity would have silently
    /// re-enabled in-place git dataset reads, which A1 §6.4 does not
    /// support. `None` here means "no known absolute base" (a Work overlay,
    /// today) and every reader falls back to the file's own `relative_path`
    /// as its identity — the pre-S6 behavior, not a crash and not a
    /// cross-source merge.
    pub identity_root: Option<PathBuf>,
    /// **F10a**: the operator-declared column allowlist for this source,
    /// defaulting to none. Governs whether a dataset row's text may become a
    /// context unit, and nothing else.
    pub context_fields: ContextFields,
}

/// The projection-identity wave's canonical file identity (ruling
/// `projection-model-and-false-j0s-2026-08-31.md` #3: *"the file is the
/// identity; sources are memberships"*): `root.join(relative_path)`,
/// canonicalized so two sources whose roots overlap on disk — one nested
/// inside the other, or a symlink between them — agree on one string for
/// the same physical file, independent of which source's own root-relative
/// path produced the walk.
///
/// Falls back to the un-canonicalized join when `identity_root` is known but
/// the path could not be resolved (already gone, a dangling symlink, or a
/// test fixture that never touched a real filesystem for this exact join).
///
/// **`None` in, `None` out — deliberately, not `relative_path` verbatim.**
/// `relative_path` alone is not a cross-source identity, merely a
/// coincidence of spelling: two *unrelated* sources can each declare their
/// own, unrelated file at the same relative path
/// (`equal_scores_are_broken_by_the_stated_key_not_by_row_arrival_order`'s
/// own fixture does exactly this, deliberately). A caller with no known
/// `identity_root` (a Work overlay, an external Git bare cache) has no
/// cross-source identity to offer and must say so rather than hand back a
/// value a reader could mistake for one — the caller's own
/// generation-scoped fallback is what actually preserves the pre-S6
/// behavior of never merging across sources.
pub fn canonical_identity(identity_root: Option<&Path>, relative_path: &str) -> Option<String> {
    let root = identity_root?;
    let joined = root.join(relative_path);
    Some(
        std::fs::canonicalize(&joined)
            .unwrap_or(joined)
            .to_string_lossy()
            .into_owned(),
    )
}

impl SourceScan {
    /// Counts by coverage status, for the journal summary and for a caller
    /// that wants to assert on them.
    pub fn counts(&self) -> BTreeMap<&'static str, u64> {
        let mut counts = BTreeMap::new();
        for row in &self.coverage {
            *counts.entry(row.status.as_str()).or_insert(0) += 1;
        }
        counts
    }

    /// Total units across every acquired file.
    pub fn unit_count(&self) -> u64 {
        self.files.iter().map(|f| f.units.len() as u64).sum()
    }

    /// Total syntax-derived symbol *sites* across every acquired file (X3b).
    ///
    /// Sites, not distinct symbols: the journal summary reports what was
    /// written, and what was written is one row per site.
    pub fn symbol_count(&self) -> u64 {
        self.syntax().map(|s| s.symbols.len() as u64).sum()
    }

    /// Total syntax-derived edges across every acquired file (X3b).
    pub fn edge_count(&self) -> u64 {
        self.syntax().map(|s| s.edges.len() as u64).sum()
    }

    /// Every file's syntax extraction, skipping the files that have none.
    fn syntax(&self) -> impl Iterator<Item = &ScannedSyntax> {
        self.files.iter().filter_map(|f| f.syntax.as_ref())
    }

    /// The coverage row saying the **source root itself** could not be read,
    /// when there is one.
    ///
    /// Three walk outcomes produce it, and only those three: the root's
    /// metadata could not be taken, the root is not a directory, or the root
    /// directory could not be listed. Each writes an
    /// [`Unavailable`](Coverage::Unavailable) row whose path is the root's own
    /// — `None` before the walk starts, `Some("")` once it has.
    ///
    /// It exists because an unreadable root and an emptied one are
    /// indistinguishable by [`content_key`](Self::content_key) alone: both
    /// hash an empty resource map. Ruling §4 evicts a generation *only* when
    /// the source bytes changed, and an unplugged drive changed no bytes — so
    /// the decision to supersede needs this signal, which the walk already
    /// recorded, rather than a key comparison that cannot tell the two apart.
    /// A readable directory that is genuinely empty produces no such row and
    /// may still legitimately supersede.
    pub fn root_unavailable(&self) -> Option<&CoverageRow> {
        self.coverage.iter().find(|row| {
            row.status == Coverage::Unavailable
                && row.path.as_deref().is_none_or(|path| path.is_empty())
        })
    }
}

/// Walk one declared knowledge root.
///
/// Fails only when the source's own `ignore` globs do not compile — an
/// operator error that must be named, not absorbed. Everything else the
/// filesystem can do (a missing root, an unreadable directory, a vanished
/// file, a symlink, a binary blob) becomes a coverage row, because a scanner
/// that refuses to finish when one file is unreadable reports nothing about
/// the thousand that were.
pub fn scan_local_knowledge(source: &KnowledgeSource) -> Result<SourceScan, BadPattern> {
    scan_local_knowledge_impl(source, None)
}

/// [`scan_local_knowledge`], with Office/ZIP/mail resources routed through a
/// real supervised worker (S4 Y8) instead of being reported `unsupported`
/// for lack of one — the shape [`super::lane::scan_local_knowledge_on_lane`]
/// actually drives in production.
///
/// [`scan_local_knowledge`] itself stays worker-free (R1): every existing
/// caller — this module's own tests, [`super::record`]'s convenience
/// wrapper, and the suites that construct a [`KnowledgeSource`] directly —
/// proves what it proves without ever needing a real subprocess, and none of
/// their fixtures claim a worker-routed extension anyway.
pub fn scan_local_knowledge_with_worker(
    source: &KnowledgeSource,
    worker: &WorkerRuntime,
) -> Result<SourceScan, BadPattern> {
    scan_local_knowledge_impl(source, Some(worker))
}

fn scan_local_knowledge_impl(
    source: &KnowledgeSource,
    worker: Option<&WorkerRuntime>,
) -> Result<SourceScan, BadPattern> {
    let filter = AcquisitionFilter::new(&source.ignore)?;
    let mut walk = Walk {
        filter: &filter,
        context_fields: &source.context_fields,
        worker,
        files: Vec::new(),
        datasets: Vec::new(),
        coverage: Vec::new(),
        extractors: BTreeSet::new(),
    };
    match std::fs::metadata(&source.root) {
        Ok(meta) if meta.is_dir() => walk.directory(&source.root, ""),
        Ok(_) => walk.coverage.push(CoverageRow {
            path: None,
            status: Coverage::Unavailable,
            detail: Some("the declared knowledge path is not a directory".to_string()),
            bytes: None,
        }),
        Err(e) => walk.coverage.push(CoverageRow {
            path: None,
            status: Coverage::Unavailable,
            detail: Some(format!("the declared knowledge path cannot be read: {e}")),
            bytes: None,
        }),
    }
    let Walk {
        files,
        datasets,
        coverage,
        extractors,
        ..
    } = walk;
    // A dataset is part of the observed world exactly as a document is, so it
    // is folded into the generation key on the same terms — path plus content
    // hash. Leaving it out would make an edited CSV look like an unchanged
    // source and skip the re-read that produces its derived evidence.
    let resources: BTreeMap<String, String> = files
        .iter()
        .map(|f| (f.relative_path.clone(), f.content_hash.clone()))
        .chain(
            datasets
                .iter()
                .map(|d| (d.relative_path.clone(), d.content_hash.clone())),
        )
        .collect();
    Ok(SourceScan {
        source_name: source.name.clone(),
        kind: SourceKind::LocalKnowledge,
        authority: AuthorityClass::EstateReadonly,
        content_key: generation_key(&resources),
        revision: None,
        observed_at: rfc3339_utc_now(),
        files,
        datasets,
        root: Some(source.root.clone()),
        identity_root: Some(source.root.clone()),
        context_fields: source.context_fields.clone(),
        coverage,
        extractors,
    })
}

/// The walk's mutable state. Separate from [`SourceScan`] so the produced
/// value is inert data with no borrow of the filter that made it.
struct Walk<'a> {
    filter: &'a AcquisitionFilter,
    context_fields: &'a ContextFields,
    /// `None` for a worker-free walk ([`scan_local_knowledge`]'s own
    /// callers): a resource [`worker_extractor_for`] claims is then reported
    /// `unsupported`, honestly naming the missing worker, rather than
    /// dispatched to nothing.
    worker: Option<&'a WorkerRuntime>,
    files: Vec<ScannedFile>,
    datasets: Vec<ScannedDataset>,
    coverage: Vec<CoverageRow>,
    extractors: BTreeSet<String>,
}

impl Walk<'_> {
    /// Recurse into one directory. `relative` is `""` for the root.
    fn directory(&mut self, path: &Path, relative: &str) {
        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                self.coverage.push(CoverageRow {
                    path: Some(relative.to_string()),
                    status: Coverage::Unavailable,
                    detail: Some(format!("directory cannot be read: {e}")),
                    bytes: None,
                });
                return;
            }
        };
        // Sorted, so two scans of an unchanged tree produce identical
        // coverage in identical order — a diff of two scans should show what
        // changed in the world, never what order the filesystem answered in.
        let mut names: Vec<(std::ffi::OsString, PathBuf)> = Vec::new();
        for entry in entries {
            match entry {
                Ok(entry) => names.push((entry.file_name(), entry.path())),
                Err(e) => self.coverage.push(CoverageRow {
                    path: Some(relative.to_string()),
                    status: Coverage::Error,
                    detail: Some(format!("directory entry cannot be read: {e}")),
                    bytes: None,
                }),
            }
        }
        names.sort();

        for (name, child) in names {
            let Some(name) = name.to_str() else {
                self.coverage.push(CoverageRow {
                    path: Some(relative.to_string()),
                    status: Coverage::Unsupported,
                    detail: Some("entry name is not valid UTF-8".to_string()),
                    bytes: None,
                });
                continue;
            };
            let child_relative = if relative.is_empty() {
                name.to_string()
            } else {
                format!("{relative}/{name}")
            };
            // F10: the verdict is taken from the path alone, before anything
            // is opened or descended into.
            if let Verdict::Denied { pattern } = self.filter.verdict(&child_relative) {
                self.coverage.push(CoverageRow {
                    path: Some(child_relative),
                    status: Coverage::Excluded,
                    detail: Some(format!("refused at acquisition by {pattern}")),
                    // Size comes from metadata, which reads no content — an
                    // excluded byte is counted without ever being read.
                    bytes: std::fs::symlink_metadata(&child)
                        .ok()
                        .filter(|m| m.is_file())
                        .map(|m| m.len()),
                });
                continue;
            }
            let meta = match std::fs::symlink_metadata(&child) {
                Ok(meta) => meta,
                Err(e) => {
                    self.coverage.push(CoverageRow {
                        path: Some(child_relative),
                        status: Coverage::Unavailable,
                        detail: Some(format!("cannot be inspected: {e}")),
                        bytes: None,
                    });
                    continue;
                }
            };
            if meta.file_type().is_symlink() {
                // Not followed: a symlink can leave the declared root
                // entirely, and a knowledge source's boundary is the path the
                // manifest declared, not wherever a link points.
                self.coverage.push(CoverageRow {
                    path: Some(child_relative),
                    status: Coverage::Unavailable,
                    detail: Some("symlink is not followed".to_string()),
                    bytes: None,
                });
                continue;
            }
            if meta.is_dir() {
                self.coverage.push(CoverageRow {
                    path: Some(child_relative.clone()),
                    status: Coverage::Discovered,
                    detail: Some("directory".to_string()),
                    bytes: None,
                });
                self.directory(&child, &child_relative);
                continue;
            }
            if !meta.is_file() {
                self.coverage.push(CoverageRow {
                    path: Some(child_relative),
                    status: Coverage::Unsupported,
                    detail: Some("not a regular file".to_string()),
                    bytes: None,
                });
                continue;
            }
            self.file(&child, child_relative, meta);
        }
    }

    /// Acquire and extract one regular file that passed the boundary.
    fn file(&mut self, path: &Path, relative: String, meta: std::fs::Metadata) {
        // Asked before [`claims_for`], and disjoint from it: the two routing
        // tables claim different extensions, and a dataset is registered
        // rather than extracted — the branch below never reads the bytes into
        // memory the way the document path does.
        if let Some(format) = format_for(&relative) {
            self.dataset(path, relative, meta, format);
            return;
        }
        // X3b's union, widened (S4 Y8): a third, disjoint routing table for
        // the three extensions that are never valid UTF-8 text and were
        // never going into `extract_resource`'s in-process pipeline —
        // checked before `claims_for` so a `.docx`/`.zip`/`.eml` never falls
        // through to the text branch's UTF-8 gate on its way to being
        // reported unsupported.
        if let Some(extractor) = worker_extractor_for(&relative) {
            self.worker_resource(path, relative, meta, extractor);
            return;
        }
        let Some(claims) = claims_for(&relative) else {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::Unsupported,
                detail: Some(UNCLAIMED.to_string()),
                bytes: Some(meta.len()),
            });
            return;
        };
        // Classified from metadata already in hand, strictly before the
        // byte-read boundary below — see the module doc's own section on
        // why this order is the whole safety property, not an optimization.
        if suspected_online_only(&meta) {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::OnlineOnly,
                detail: Some(ONLINE_ONLY_DETAIL.to_string()),
                bytes: Some(meta.len()),
            });
            return;
        }
        if meta.len() > MAX_RESOURCE_BYTES {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "larger than the {MAX_RESOURCE_BYTES}-byte resource ceiling"
                )),
                bytes: Some(meta.len()),
            });
            return;
        }
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                self.coverage.push(CoverageRow {
                    path: Some(relative),
                    status: Coverage::Unavailable,
                    detail: Some(format!("cannot be read: {e}")),
                    bytes: Some(meta.len()),
                });
                return;
            }
        };
        let Some(text) = as_text(&bytes) else {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::Unsupported,
                detail: Some("not valid UTF-8 text".to_string()),
                bytes: Some(bytes.len() as u64),
            });
            return;
        };
        let hash = content_hash(&bytes);
        let extracted = extract_resource(claims, text, &hash, KeySpace::Local);
        self.extractors.extend(extracted.identities.iter().cloned());
        self.coverage.push(CoverageRow {
            path: Some(relative.clone()),
            status: extracted.status(),
            detail: Some(extracted.detail()),
            bytes: Some(bytes.len() as u64),
        });
        self.files.push(ScannedFile {
            relative_path: relative,
            local_key: extracted.key,
            content_digest: hash.clone(),
            content_hash: hash,
            extractor: extracted.extractor.to_string(),
            byte_len: bytes.len() as u64,
            mtime_millis: mtime_millis(&meta),
            units: extracted.units,
            syntax: extracted.syntax,
            parent: None,
        });
    }

    /// Acquire and dispatch one resource a supervised-worker adapter claims
    /// (S4 Y8) — [`worker_extractor_for`]'s own routing. The same online-only
    /// and size-ceiling gates [`Self::file`] applies to the in-process path
    /// apply here first, over the same already-fetched metadata, before a
    /// single byte is read; what differs from there on is where the bytes
    /// go — [`run_worker`], never `extract_resource`'s UTF-8 text pipeline,
    /// because the parser behind this extractor is third-party code over
    /// attacker-influenced bytes (G2's own boundary), not because this walk
    /// stopped trusting the filesystem.
    fn worker_resource(
        &mut self,
        path: &Path,
        relative: String,
        meta: std::fs::Metadata,
        extractor: &'static str,
    ) {
        if suspected_online_only(&meta) {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::OnlineOnly,
                detail: Some(ONLINE_ONLY_DETAIL.to_string()),
                bytes: Some(meta.len()),
            });
            return;
        }
        if meta.len() > MAX_RESOURCE_BYTES {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "larger than the {MAX_RESOURCE_BYTES}-byte resource ceiling"
                )),
                bytes: Some(meta.len()),
            });
            return;
        }
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                self.coverage.push(CoverageRow {
                    path: Some(relative),
                    status: Coverage::Unavailable,
                    detail: Some(format!("cannot be read: {e}")),
                    bytes: Some(meta.len()),
                });
                return;
            }
        };
        let hash = content_hash(&bytes);
        let Some(worker) = self.worker else {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "{extractor} claims this resource, but no supervised worker is configured \
                     for this scan"
                )),
                bytes: Some(bytes.len() as u64),
            });
            return;
        };
        let mtime = mtime_millis(&meta);
        dispatch_worker_resource(
            worker,
            self.filter,
            relative,
            &hash,
            KeySpace::Local,
            bytes,
            extractor,
            mtime,
            ChildSink {
                files: &mut self.files,
                coverage: &mut self.coverage,
                extractors: &mut self.extractors,
                datasets: Some(&mut self.datasets),
                context_fields: self.context_fields,
            },
        );
    }

    /// Register one tabular dataset (X4).
    ///
    /// **Registration, not acquisition.** The file is hashed in a streaming
    /// pass and closed; nothing here reads a row, decides a schema, or holds a
    /// buffer bigger than [`DATASET_HASH_CHUNK`]. What the dataset *contains*
    /// is answered later, in place, by the one module that owns a database
    /// connection — which is also the only place F10a's allowlist can be
    /// applied to a value, because it is the only place a value exists.
    ///
    /// **A path a reader would glob is refused here, before it is registered.**
    /// Every tabular reader takes its path as a multi-file *pattern*, so a
    /// filename carrying `*`, `?` or `[` would make the read fan out across
    /// siblings this walk deliberately excluded — F10's deny set and the
    /// source's own `ignore` globs would stop governing which bytes get read,
    /// and the rows returned would be recorded under a `content_hash` computed
    /// from one file. Registering it and refusing later would still leave a
    /// dataset row promising a read that must never happen, so the refusal is
    /// at the point of registration, fail-closed like every other row here.
    fn dataset(
        &mut self,
        path: &Path,
        relative: String,
        meta: std::fs::Metadata,
        format: crate::runtime::atlas::tabular::DatasetFormat,
    ) {
        // Same rule as `file`'s, checked before `hash_file` ever opens the
        // path: a dataset's streaming hash is a read too, and this is the
        // one place in this function that would trigger hydration.
        if suspected_online_only(&meta) {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::OnlineOnly,
                detail: Some(ONLINE_ONLY_DETAIL.to_string()),
                bytes: Some(meta.len()),
            });
            return;
        }
        if relative.contains(crate::runtime::atlas::db::GLOB_METACHARACTERS) {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::Unsupported,
                detail: Some(crate::runtime::atlas::db::DATASET_GLOB_PATH.to_string()),
                bytes: Some(meta.len()),
            });
            return;
        }
        if meta.len() > MAX_DATASET_BYTES {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "larger than the {MAX_DATASET_BYTES}-byte dataset ceiling"
                )),
                bytes: Some(meta.len()),
            });
            return;
        }
        let hash = match hash_file(path) {
            Ok(hash) => hash,
            Err(e) => {
                self.coverage.push(CoverageRow {
                    path: Some(relative),
                    status: Coverage::Unavailable,
                    detail: Some(format!("cannot be read: {e}")),
                    bytes: Some(meta.len()),
                });
                return;
            }
        };
        let reader = reader_identity(format, self.context_fields);
        self.extractors.insert(reader.clone());
        self.coverage.push(CoverageRow {
            path: Some(relative.clone()),
            status: Coverage::Indexed,
            detail: Some(reader.clone()),
            bytes: Some(meta.len()),
        });
        self.datasets.push(ScannedDataset {
            relative_path: relative,
            format,
            dataset_key: local_key(&hash, &reader),
            content_hash: hash,
            reader,
            byte_len: meta.len(),
            mtime_millis: mtime_millis(&meta),
            // A loose dataset is read IN PLACE, from the path just recorded:
            // its bytes never enter this struct, and it has no container.
            content: None,
            parent: None,
        });
    }
}

/// BLAKE3 of a file's bytes, streamed.
///
/// The document path reads whole files because it has to hand the text to an
/// extractor; a dataset has no such need, and a 512 MiB Parquet file has no
/// business being resident just to be named.
fn hash_file(path: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; DATASET_HASH_CHUNK];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// The coverage detail for a path no extractor and no grammar claims (F8).
///
/// One constant, shared by all three walks, because "unsupported" has to mean
/// the same thing whichever route the bytes arrived by — and because the
/// sentence has to name *both* routing tables now that there are two.
pub const UNCLAIMED: &str = "no extractor or grammar in this build claims this extension";

/// Which key space a source's derived rows live in (F7).
///
/// The two halves of F7's cache-key rule, as a value the three walks can pass
/// to one shared extractor instead of each spelling their own composition. The
/// spaces are domain-separated by
/// [`estate_git_key`]/[`local_key`]/[`child_key`] themselves — this enum only
/// chooses between them, and cannot blur them. S5 W7 added the third arm for
/// container children; the domain separation argument is unchanged, because
/// it lives in those three functions' own hash transcripts, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySpace<'a> {
    /// BLAKE3 of the bytes plus extractor identity — a filesystem resource,
    /// which no object store has already named.
    Local,
    /// Git blob OID plus extractor identity — **never a second hash of bytes
    /// Git already hashed**.
    EstateGit,
    /// A container child (S5 W7): [`child_key`] of its IMMEDIATE parent's own
    /// key, its entry path, its content hash and its extractor — chained, so
    /// a grandchild's key already encodes its whole ancestry (`child_key`'s
    /// own doc for why a flattened "always key off the root" scheme would
    /// collide two nested archives sharing a leaf path).
    ///
    /// Not a third *hashing* rule so much as a third *composition* rule: the
    /// content identity handed to [`Self::key`] is still the daemon's own
    /// BLAKE3 of the bytes it received, exactly as [`Self::Local`]'s is.
    Child {
        /// The immediate parent resource's own composed key.
        parent_key: &'a str,
        /// The child's entry path inside that parent.
        entry_path: &'a str,
    },
}

impl KeySpace<'_> {
    /// Compose a content identity with an extractor identity in this space.
    pub fn key(self, content_id: &str, extractor: &str) -> String {
        match self {
            Self::Local => local_key(content_id, extractor),
            Self::EstateGit => estate_git_key(content_id, extractor),
            Self::Child {
                parent_key,
                entry_path,
            } => child_key(parent_key, entry_path, content_id, extractor),
        }
    }
}

/// Where a landed child resource came from — A1 §6.6's first two preserved
/// fields (`parent archive source/resource`, `entry path`), carried beside
/// the child's own row (S5 W7).
///
/// The other two of §6.6's four are already columns every resource row has:
/// the entry content hash is [`ScannedFile::content_hash`] (the daemon's own
/// hash of the bytes it received) and the entry adapter is
/// [`ScannedFile::extractor`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildProvenance {
    /// The parent container resource's own path, as that resource was
    /// addressed in this scan.
    pub parent_relative_path: String,
    /// The parent container resource's own composed F7 key.
    pub parent_key: String,
    /// This child's entry path inside that parent.
    pub entry_path: String,
}

/// What claims one path, decided from the path alone (X3b).
///
/// Two routing tables, unioned at exactly one place. [`extractor_for`] routes
/// bytes to a structure-unit extractor; [`language_for`] routes them to a
/// grammar. A path either table claims is acquired; a path neither claims is
/// honestly `unsupported` (F8).
///
/// The union has one deliberate consequence worth naming: a path a *grammar*
/// claims but the structure table does not — `main.rs`, `Cargo.toml` — is
/// acquired with the plain-text structure extractor
/// ([`TEXT_EXTRACTOR`]), so every acquired resource still has units and
/// `source.files`' existing columns keep the meaning X2 gave them. The
/// alternative — a `source.files` row whose `extractor` names a grammar and
/// whose `unit_count` is zero — would have made one column mean two things
/// depending on the row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Claims {
    /// The structure-unit extractor that will run.
    pub structure: &'static str,
    /// The grammar that will run too, when one claims this path.
    pub language: Option<SyntaxLanguage>,
}

/// What claims `relative`, or `None` for a path nothing in this build claims.
pub fn claims_for(relative: &str) -> Option<Claims> {
    let language = language_for(relative);
    let structure = extractor_for(relative).or(language.map(|_| TEXT_EXTRACTOR))?;
    Some(Claims {
        structure,
        language,
    })
}

/// What claims `relative` for a supervised-worker adapter (Office/ZIP/mail),
/// or `None` for a path none of them claims (S4 Y8).
///
/// A second, disjoint routing table from [`claims_for`]'s own union of
/// [`extractor_for`]/[`language_for`]: `.docx`/`.zip`/`.eml` are never valid
/// UTF-8 text, so they were never reaching [`extract_resource`]'s in-process
/// pipeline — they route instead through [`run_worker`], because the parser
/// behind each is third-party code over attacker-influenced bytes (G2's own
/// boundary), and a crash in one belongs to a worker's own process group,
/// never the daemon's. [`office::extractor_for`], [`archive::extractor_for`]
/// and [`mail::extractor_for`] are unioned exactly the way [`claims_for`]
/// unions its own two tables — one place, so a fourth worker-routed adapter
/// only ever widens this one `.or_else` chain.
///
/// [`office::extractor_for`]: crate::runtime::atlas::office::extractor_for
/// [`archive::extractor_for`]: crate::runtime::atlas::archive::extractor_for
/// [`mail::extractor_for`]: crate::runtime::atlas::mail::extractor_for
pub fn worker_extractor_for(relative: &str) -> Option<&'static str> {
    crate::runtime::atlas::office::extractor_for(relative)
        .or_else(|| crate::runtime::atlas::archive::extractor_for(relative))
        .or_else(|| crate::runtime::atlas::mail::extractor_for(relative))
}

/// What claims a container CHILD's path — A1 §6.6's `entry adapter` field
/// (S5 W7).
///
/// Deliberately not a fourth routing table: it is exactly the THREE tables
/// `Walk::file` already consults for a loose file on disk, in the same
/// order — [`format_for`]'s dataset table first, then
/// [`worker_extractor_for`], then [`claims_for`]'s own structure extractor.
/// A `.docx` inside a `.zip` inside an `.eml` therefore reaches the Office
/// adapter by the same route a loose `.docx` does, and a `.csv` inside a
/// `.zip` reaches the tabular reader by the same route a loose `.csv` does
/// (A1-17, "recursively route child bytes through normal adapters"; A1 §1's
/// own motivating payload is "100k ServiceNow tickets in CSV/JSON/Parquet
/// inside an archive"). A child nothing claims answers `None`, which is a
/// named coverage gap daemon-side rather than silence (F8).
///
/// The dataset arm answers with [`DatasetFormat::reader_version`], not the
/// per-source [`reader_identity`] the registered row carries: this function
/// is the cross-check both the worker and [`validate_batch`] re-derive
/// independently (A1 §6.6's `entry adapter` field), and a worker knows
/// nothing about the source's own F10a column allowlist. The allowlist is
/// part of the dataset's *extraction* identity, not of which table claimed
/// its extension.
///
/// [`DatasetFormat::reader_version`]: crate::runtime::atlas::tabular::DatasetFormat::reader_version
/// [`reader_identity`]: crate::runtime::atlas::tabular::reader_identity
/// [`validate_batch`]: crate::runtime::atlas::worker::validate_batch
pub fn child_extractor_for(relative: &str) -> Option<&'static str> {
    format_for(relative)
        .map(|format| format.reader_version())
        .or_else(|| worker_extractor_for(relative))
        .or_else(|| claims_for(relative).map(|claims| claims.structure))
}

/// Whether a child path routes to a CONTAINER adapter — an archive or a mail
/// message (S5 W7).
///
/// A container child is landed as its own resource but never dispatched to a
/// worker a second time: the worker that produced it already expanded it,
/// under the one shared depth counter and the one shared whole-tree byte
/// budget, and that expansion's own members arrive flattened in the SAME
/// batch (see [`validate_batch`]'s own doc). Re-dispatching it daemon-side
/// would be a second recursion under a second budget.
///
/// [`validate_batch`]: crate::runtime::atlas::worker::validate_batch
pub fn child_is_container(relative: &str) -> bool {
    crate::runtime::atlas::archive::extractor_for(relative).is_some()
        || crate::runtime::atlas::mail::extractor_for(relative).is_some()
}

/// The separator between a container resource's own path and an entry path
/// inside it, in a landed child's [`ScannedFile::relative_path`] (S5 W7).
///
/// `!/` is the long-established archive-coordinate convention (JAR URLs, and
/// every tool that reads them), taken for the reason it was chosen there: it
/// reads as "inside", and it is not what an ordinary directory separator
/// looks like. This separator is what makes a child addressable in the same
/// column every other resource is addressable in.
///
/// **It joins EVERY nesting level, and it is RESERVED** (S5 W7 F-SF-04).
/// `src/bin/atlas_worker.rs`'s `flatten_zip`/`flatten_mail` compose a
/// grandchild's path with this string, not a plain `/` — `/` already means
/// "a directory component inside one container", so using it for the
/// container boundary too made `bundle.zip/report.docx` mean either "an
/// entry with that name" or "an entry inside an entry" with nothing to tell
/// them apart. [`super::archive`] and [`super::mail`] therefore REFUSE an
/// entry name or attachment filename containing this sequence, each with its
/// own named coverage row, so a composed path splits back into its exact
/// container chain. That chain is what [`ChildProvenance`] records
/// column-by-column in `source.child_resources` — the parent resource's own
/// composed path, the parent's own F7 key, and the entry path RELATIVE TO
/// THAT IMMEDIATE PARENT (A1 §6.6's first two preserved fields).
pub const CHILD_PATH_SEPARATOR: &str = "!/";

/// The sentinel [`WorkerIdentity::generation_id`] a per-resource worker
/// dispatch uses mid-walk (S4 Y8) — before Atlas has assigned a real
/// generation for the scan this resource belongs to.
///
/// A generation is assigned only once a whole scan is staged
/// ([`super::db::AtlasDb::stage_scan`]), which happens after every resource
/// has already been walked and extracted; dispatching one resource to a
/// worker happens *during* that walk, before any such id exists. This value
/// correlates one dispatched call to its own [`super::worker::validate_batch`]
/// check — the worker echoes it back untouched, and a mismatch would mean
/// the batch answered a different call — and is never written to the store
/// or confused with a persisted `source.generations.id`.
pub(crate) const PRE_STAGE_GENERATION: &str = "pending";

/// The accumulating outputs one worker dispatch folds into, plus the two
/// source-level facts landing a CHILD needs (S5 W7 F-SF-01).
///
/// One struct rather than five more parameters on a `#[allow(clippy::
/// too_many_arguments)]` function that recurses into itself: the set is
/// exactly "everything a walk accumulates", it travels as a unit, and a
/// caller that forgot one would otherwise silently drop rows.
pub(crate) struct ChildSink<'a> {
    /// Acquired resources, appended in the order they land.
    pub files: &'a mut Vec<ScannedFile>,
    /// One row per path seen — F8's rule holds for children exactly as it
    /// does for loose files.
    pub coverage: &'a mut Vec<CoverageRow>,
    /// Distinct extractor identities that actually produced evidence.
    pub extractors: &'a mut BTreeSet<String>,
    /// Where a container child claimed by [`format_for`] is registered, and
    /// `None` for a walk that registers no datasets at all.
    ///
    /// `None` is not a shrug: [`super::git`]'s walk answers `None` because
    /// its own loose files claimed by that same table are not registered
    /// either ([`DATASET_NO_ROOT`]), so a child taking that route gets the
    /// same answer a loose file of that extension gets — which is what
    /// route-equivalence means. [`super::overlay`] never produces a
    /// container child at all.
    pub datasets: Option<&'a mut Vec<ScannedDataset>>,
    /// **F10a**: the source's declared column allowlist, needed to compose a
    /// registered child dataset's reader identity.
    pub context_fields: &'a ContextFields,
}

impl ChildSink<'_> {
    /// Reborrow, so a recursive dispatch can hand the same sink down without
    /// moving it.
    fn reborrow(&mut self) -> ChildSink<'_> {
        ChildSink {
            files: self.files,
            coverage: self.coverage,
            extractors: self.extractors,
            datasets: self.datasets.as_deref_mut(),
            context_fields: self.context_fields,
        }
    }
}

/// Dispatch one resource a supervised-worker adapter claims to the real
/// worker, and fold the outcome into a walk's accumulating rows (S4 Y8) —
/// the one place this happens for every walk that routes through a worker,
/// mirroring [`extract_resource`]'s own "one place a resource is extracted"
/// rule for the in-process pipeline (module doc, "The shared extraction").
///
/// `content_id`/`keys` compose the *stored* F7 key exactly as
/// [`extract_resource`] does for its own callers — a filesystem resource's
/// BLAKE3 hash for [`KeySpace::Local`], Git's own blob OID for
/// [`KeySpace::EstateGit`], never a second hash of bytes Git already hashed.
/// That is independent of, and may differ from, the hash the wire protocol
/// itself always uses: [`WorkerIdentity::resource_hash`] is specifically
/// "BLAKE3 hex of the exact bytes handed to the worker" (its own doc)
/// because `sgt-atlas-worker` always computes it that way regardless of
/// which key space the caller's own F7 identity lives in
/// (`src/bin/atlas_worker.rs`'s `normal_batch`) — so this function hashes
/// `bytes` itself for the wire identity, and composes the stored key from
/// `content_id` separately. For an estate-git resource the two differ (a
/// SHA-1 blob OID is not a BLAKE3 digest); for a local one they happen to be
/// the same bytes hashed the same way, computed twice rather than threaded
/// through as one more parameter, because keeping the wire hash's
/// computation in one place (here) is worth more than saving one BLAKE3
/// pass over an already-small, already-ceiling-bounded resource.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_worker_resource(
    worker: &WorkerRuntime,
    filter: &AcquisitionFilter,
    relative_path: String,
    content_id: &str,
    keys: KeySpace,
    bytes: Vec<u8>,
    extractor: &'static str,
    mtime_millis: Option<i64>,
    sink: ChildSink<'_>,
) {
    dispatch_worker_resource_at_depth(
        worker,
        filter,
        relative_path,
        content_id,
        keys,
        bytes,
        extractor,
        mtime_millis,
        None,
        0,
        sink,
    );
}

/// [`dispatch_worker_resource`] plus the two things only a CHILD dispatch
/// needs (S5 W7): the parent coordinate the landed row must carry, and the
/// nesting depth this dispatch already sits at.
///
/// `depth` is measured in the SAME counter mail and archive already share —
/// [`archive::MAX_NESTING_DEPTH`], never a second one. In practice a worker
/// flattens its whole expansion tree into one batch, so this recursion only
/// ever goes one level deep (a container's child dispatched to a non-container
/// adapter, e.g. a `.docx` entry to the Office adapter). The guard is what
/// makes that a structural fact rather than an assumption about a
/// well-behaved worker: a buggy or hostile worker that answers a `.docx`
/// dispatch with another `.docx` child cannot walk the daemon down an
/// unbounded recursion, because the same ceiling that bounds a container tree
/// bounds this.
///
/// [`archive::MAX_NESTING_DEPTH`]: crate::runtime::atlas::archive::MAX_NESTING_DEPTH
#[allow(clippy::too_many_arguments)]
fn dispatch_worker_resource_at_depth(
    worker: &WorkerRuntime,
    filter: &AcquisitionFilter,
    relative_path: String,
    content_id: &str,
    keys: KeySpace,
    bytes: Vec<u8>,
    extractor: &'static str,
    mtime_millis: Option<i64>,
    parent: Option<ChildProvenance>,
    depth: u32,
    mut sink: ChildSink<'_>,
) {
    let identity = WorkerIdentity {
        generation_id: PRE_STAGE_GENERATION.to_string(),
        resource_hash: content_hash(&bytes),
        extractor: extractor.to_string(),
    };
    let spawn = WorkerSpawn {
        program: worker.program.clone(),
        args: vec![
            "--generation".to_string(),
            identity.generation_id.clone(),
            "--extractor".to_string(),
            identity.extractor.clone(),
        ],
        input: bytes.clone(),
        deadline: worker.deadline,
    };
    match run_worker(spawn, &identity, filter) {
        WorkerOutcome::Accepted(batch) => {
            sink.extractors.insert(batch.extractor.clone());
            // Every child in this batch has already passed `validate_batch`'s
            // whole AUTHORITY — path safety, F10's deny set on name and path,
            // the per-child ceiling, the content hash, the adapter claim —
            // before a byte of it reaches here (S5 W7). What is left is to
            // land it: a child is a RESOURCE, not a name in its container's
            // coverage detail.
            let mut detail = batch.extractor.clone();
            if !batch.declared_children.is_empty() {
                let names: Vec<String> = batch
                    .declared_children
                    .iter()
                    .map(|c| c.relative_path.clone())
                    .collect();
                detail = format!(
                    "{detail}; children landed as their own resources (A1 §6.6, each with its \
                     parent coordinate and the daemon's own hash of the bytes it received): {}",
                    names.join(", ")
                );
            }
            sink.coverage.push(CoverageRow {
                path: Some(relative_path.clone()),
                status: Coverage::Indexed,
                detail: Some(detail),
                bytes: Some(bytes.len() as u64),
            });
            let units = batch
                .units
                .into_iter()
                .enumerate()
                // S5 closeout (F-AC-02): everything the adapter knew about
                // the unit lands, not just its span. Dropping `title`,
                // `heading_level` and the native `coordinate` here is what
                // left every worker-routed family (mail, Office, archive
                // children) without the provenance A2 §17 item 2 and §9
                // require, while the in-process path below kept it.
                .map(|(ordinal, unit)| ScannedUnit {
                    ordinal: ordinal as u64,
                    kind: unit.kind,
                    heading_level: unit.heading_level,
                    title: unit.title,
                    byte_start: unit.byte_start,
                    byte_end: unit.byte_end,
                    coordinate: unit.coordinate,
                    text: unit.text,
                })
                .collect();
            let parent_key = keys.key(content_id, &batch.extractor);
            sink.files.push(ScannedFile {
                relative_path: relative_path.clone(),
                local_key: parent_key.clone(),
                // `identity.resource_hash` above is already `content_hash(&bytes)`
                // — plain BLAKE3 of these exact bytes, computed once, reused
                // here rather than a second hash pass (R2).
                content_digest: identity.resource_hash.clone(),
                content_hash: content_id.to_string(),
                extractor: batch.extractor,
                byte_len: bytes.len() as u64,
                mtime_millis,
                units,
                syntax: None,
                parent,
            });
            // S5 W7 (F-SF-02): a child's parent coordinate is its IMMEDIATE
            // parent's, not the dispatched resource's. The worker flattens a
            // whole tree onto one batch, joining nesting levels with
            // [`CHILD_PATH_SEPARATOR`], and emits a container before its own
            // members — so walking that list in order, remembering each
            // landed container's composed path and its own freshly-composed
            // F7 key, is enough to hand every child its real parent without
            // re-entering a single container. `validate_batch` has already
            // refused any child whose path names a container this batch does
            // not declare, which is what makes the lookup below well-founded
            // rather than a best effort.
            let mut containers: BTreeMap<String, (String, String)> = BTreeMap::new();
            for child in batch.declared_children {
                let full_entry_path = child.relative_path;
                let (child_parent_path, child_parent_key, entry_path) = match full_entry_path
                    .rsplit_once(CHILD_PATH_SEPARATOR)
                {
                    None => (
                        relative_path.clone(),
                        parent_key.clone(),
                        full_entry_path.clone(),
                    ),
                    Some((ancestor, own)) => match containers.get(ancestor) {
                        Some((composed, key)) => (composed.clone(), key.clone(), own.to_string()),
                        None => {
                            // Unreachable behind `validate_batch`'s own
                            // `OrphanedChildPath` refusal, and named
                            // rather than unwrapped for the same reason
                            // every other such branch in this module is:
                            // an honest coverage row beats a panic in the
                            // sole writer.
                            sink.coverage.push(CoverageRow {
                                path: Some(format!(
                                    "{relative_path}{CHILD_PATH_SEPARATOR}{full_entry_path}"
                                )),
                                status: Coverage::Error,
                                detail: Some(format!(
                                    "container child names the container {ancestor:?} it came \
                                         out of, but no such container child was landed from this \
                                         batch; not landed, rather than re-parented onto \
                                         {relative_path:?}"
                                )),
                                bytes: Some(child.content.len() as u64),
                            });
                            continue;
                        }
                    },
                };
                if let Some(landed) = land_child(
                    worker,
                    filter,
                    &child_parent_path,
                    &child_parent_key,
                    entry_path,
                    child.content,
                    depth,
                    sink.reborrow(),
                ) {
                    containers.insert(full_entry_path, landed);
                }
            }
        }
        WorkerOutcome::Refused(row) => {
            sink.coverage.push(CoverageRow {
                path: Some(relative_path),
                bytes: Some(bytes.len() as u64),
                ..row
            });
        }
    }
}

/// Land one validated container child as its own resource (S5 W7) — A1
/// §6.6's "entries expand into child resources", and A1-17's "recursively
/// route child bytes through normal adapters".
///
/// The routing is [`child_extractor_for`]'s, which is [`format_for`] ∪
/// [`worker_extractor_for`] ∪ [`claims_for`] — the same three tables, in the
/// same order, a loose file of that name goes through — so there is exactly
/// one dispatcher in this build, not a second one for children (R2). Four
/// destinations follow from it:
///
/// - **a dataset child** (a `.csv`/`.json`/`.parquet`) is REGISTERED, not
///   extracted — A1-13's relational lane, the identical destination a loose
///   one reaches, with its bytes travelling on the registration because a
///   child has no path for DuckDB to read in place;
/// - **a container child** (a nested `.zip`/`.eml`) lands as its own resource
///   and is NOT re-dispatched: the worker already expanded it under the one
///   shared depth counter and the one shared whole-tree byte budget, and its
///   own members arrive flattened in the same batch;
/// - **a worker-adapter child** (a `.docx`) is dispatched to the real worker
///   through [`dispatch_worker_resource_at_depth`] — the identical path a
///   loose `.docx` takes;
/// - **an in-process child** (`.md`, `.txt`, a grammar-claimed source file)
///   goes through [`extract_resource`], the identical path a loose one takes.
///
/// A child nothing claims is a NAMED COVERAGE GAP, not silence (F8): a
/// [`Coverage::Unsupported`] row at the child's own composed path saying so.
///
/// The daemon's own [`content_hash`] of the bytes that arrived is the child's
/// stored identity — see [`DeclaredChild`]'s own doc for what that does and
/// does not vouch for.
///
/// `parent_relative_path`/`parent_key` are the IMMEDIATE parent's, and
/// `entry_path` is this entry's path RELATIVE TO THAT PARENT (S5 W7
/// F-SF-02) — the caller decomposes a flattened path and supplies them, so
/// everything composed here (the child's own path, its [`ChildProvenance`]
/// and its [`KeySpace::Child`] key) chains one level rather than resolving
/// to the root container. Returns the container coordinate a landed
/// container child's own members must be landed against — its composed path
/// and its own key — and `None` for every other outcome.
///
/// [`DeclaredChild`]: crate::runtime::atlas::worker::DeclaredChild
#[allow(clippy::too_many_arguments)]
fn land_child(
    worker: &WorkerRuntime,
    filter: &AcquisitionFilter,
    parent_relative_path: &str,
    parent_key: &str,
    entry_path: String,
    content: Vec<u8>,
    depth: u32,
    mut sink: ChildSink<'_>,
) -> Option<(String, String)> {
    let composed = format!("{parent_relative_path}{CHILD_PATH_SEPARATOR}{entry_path}");
    let byte_len = content.len() as u64;
    if depth >= crate::runtime::atlas::archive::MAX_NESTING_DEPTH {
        sink.coverage.push(CoverageRow {
            path: Some(composed),
            status: Coverage::Unsupported,
            detail: Some(format!(
                "container child at nesting depth {depth}, at or past the shared \
                 {}-level ceiling (archive::MAX_NESTING_DEPTH — the same counter archive and \
                 mail nesting share); not landed",
                crate::runtime::atlas::archive::MAX_NESTING_DEPTH
            )),
            bytes: Some(byte_len),
        });
        return None;
    }
    let Some(extractor) = child_extractor_for(&entry_path) else {
        sink.coverage.push(CoverageRow {
            path: Some(composed),
            status: Coverage::Unsupported,
            detail: Some(format!(
                "{UNCLAIMED} (container child of {parent_relative_path:?}; admitted and \
                 validated, but nothing in this build claims its extension)"
            )),
            bytes: Some(byte_len),
        });
        return None;
    };
    // The daemon's own hash of the bytes it received, computed on receipt —
    // not the worker's declared value, which `validate_batch` has already
    // cross-checked against exactly this computation.
    let hash = content_hash(&content);
    let provenance = ChildProvenance {
        parent_relative_path: parent_relative_path.to_string(),
        parent_key: parent_key.to_string(),
        entry_path: entry_path.clone(),
    };
    let keys = KeySpace::Child {
        parent_key,
        entry_path: &entry_path,
    };
    // A1-17 + A1 §1's motivating payload (S5 W7 F-SF-01): a `.csv`/`.json`/
    // `.parquet` child takes the SAME relational lane a loose one takes —
    // `Walk::file` consults `format_for` first, and so does
    // `child_extractor_for`. What differs is only where the reader finds the
    // bytes: a loose dataset is read in place from the source root, and a
    // child's bytes exist only here, so they travel on the registration and
    // `db::read_dataset` materialises them around its own read. Registering
    // rather than extracting is the whole point of A1-13 — a 100k-ticket
    // export inside an archive must not become 100k Markdown documents just
    // because it arrived inside a container.
    if let Some(format) = format_for(&entry_path) {
        let Some(datasets) = sink.datasets.as_deref_mut() else {
            sink.coverage.push(CoverageRow {
                path: Some(composed),
                status: Coverage::Unsupported,
                detail: Some(DATASET_CHILD_NOT_REGISTERED.to_string()),
                bytes: Some(byte_len),
            });
            return None;
        };
        let reader = reader_identity(format, sink.context_fields);
        sink.extractors.insert(reader.clone());
        // A placeholder, exactly as `Walk::dataset`'s own row is: the read
        // happens later, in the one module that owns a connection, and
        // `stage_scan` replaces this row with what actually happened.
        sink.coverage.push(CoverageRow {
            path: Some(composed.clone()),
            status: Coverage::Indexed,
            detail: Some(reader.clone()),
            bytes: Some(byte_len),
        });
        datasets.push(ScannedDataset {
            relative_path: composed,
            format,
            dataset_key: keys.key(&hash, &reader),
            content_hash: hash,
            reader,
            byte_len,
            mtime_millis: None,
            content: Some(content),
            parent: Some(provenance),
        });
        return None;
    }
    if child_is_container(&entry_path) {
        sink.extractors.insert(extractor.to_string());
        sink.coverage.push(CoverageRow {
            path: Some(composed.clone()),
            status: Coverage::Indexed,
            detail: Some(format!(
                "{extractor}; container child — whatever the worker that produced it admitted \
                 out of it, under the shared depth counter and whole-tree byte budget, is landed \
                 from the SAME batch; this daemon never re-enters a container. Whether that \
                 worker opened this one at all, or stopped at the depth ceiling, is a per-entry \
                 coverage fact `WorkerBatch` still has no field to carry (the pre-existing named \
                 gap in `src/bin/atlas_worker.rs`'s own module doc), so it is not claimed here"
            )),
            bytes: Some(byte_len),
        });
        let local_key = keys.key(&hash, extractor);
        sink.files.push(ScannedFile {
            relative_path: composed.clone(),
            local_key: local_key.clone(),
            // `hash` above is already the daemon's own BLAKE3 of these
            // bytes (F7's child rule), reused rather than a second hash.
            content_digest: hash.clone(),
            content_hash: hash,
            extractor: extractor.to_string(),
            byte_len,
            mtime_millis: None,
            units: Vec::new(),
            syntax: None,
            parent: Some(provenance),
        });
        // The coordinate this container's OWN members are landed against
        // (F-SF-02): its composed path and its own chained F7 key, never the
        // resource this batch was dispatched for.
        return Some((composed, local_key));
    }
    if worker_extractor_for(&entry_path).is_some() {
        dispatch_worker_resource_at_depth(
            worker,
            filter,
            composed,
            &hash,
            keys,
            content,
            extractor,
            None,
            Some(provenance),
            depth + 1,
            sink,
        );
        return None;
    }
    let Some(claims) = claims_for(&entry_path) else {
        // Unreachable in practice: `child_extractor_for` answered `Some`, and
        // its only other arm is `worker_extractor_for`, already taken above.
        // Named rather than unwrapped — a routing table that grew a third arm
        // should produce an honest coverage row here, never a panic in the
        // daemon.
        sink.coverage.push(CoverageRow {
            path: Some(composed),
            status: Coverage::Unsupported,
            detail: Some(format!(
                "{UNCLAIMED} (container child claimed {extractor} by routing but no in-process \
                 claim backs it)"
            )),
            bytes: Some(byte_len),
        });
        return None;
    };
    let Some(text) = as_text(&content) else {
        sink.coverage.push(CoverageRow {
            path: Some(composed),
            status: Coverage::Unsupported,
            detail: Some("not valid UTF-8 text".to_string()),
            bytes: Some(byte_len),
        });
        return None;
    };
    let extracted = extract_resource(claims, text, &hash, keys);
    sink.extractors.extend(extracted.identities.iter().cloned());
    sink.coverage.push(CoverageRow {
        path: Some(composed.clone()),
        status: extracted.status(),
        detail: Some(extracted.detail()),
        bytes: Some(byte_len),
    });
    sink.files.push(ScannedFile {
        relative_path: composed,
        local_key: extracted.key,
        // Same daemon-computed BLAKE3 as the container-child site above.
        content_digest: hash.clone(),
        content_hash: hash,
        extractor: extracted.extractor.to_string(),
        byte_len,
        mtime_millis: None,
        units: extracted.units,
        syntax: extracted.syntax,
        parent: Some(provenance),
    });
    None
}

/// Everything one resource's bytes yielded, for every extractor that claimed
/// them (X3b).
///
/// **The one place a resource is extracted**, for all three walks — the
/// filesystem one, the estate-git one and the Work-overlay one. Three copies
/// of this would be three ways for F7's premise (identical bytes plus
/// identical extractor identity are one extraction) to stop being true, which
/// is the same argument [`extract_units`] already carries for its own loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceExtraction {
    /// The structure extractor that ran, and the identity its units are keyed
    /// by.
    pub extractor: &'static str,
    /// F7 key for the structure extraction.
    pub key: String,
    /// Structure units, in document order.
    pub units: Vec<ScannedUnit>,
    /// The syntax extraction, when a grammar claimed the path and parsed it.
    pub syntax: Option<ScannedSyntax>,
    /// Every extractor identity that actually produced rows, in the order they
    /// ran. An extractor that failed is **not** here: the journal summary's
    /// `extractors` list names what produced evidence, not what was attempted.
    pub identities: Vec<String>,
    /// Why a claimed extractor produced nothing, when one did. Its presence is
    /// what turns this resource's coverage row from `indexed` into `error`
    /// (F8).
    pub failure: Option<String>,
}

impl ResourceExtraction {
    /// The coverage status this extraction earns (F8).
    pub fn status(&self) -> Coverage {
        if self.failure.is_some() {
            Coverage::Error
        } else {
            Coverage::Indexed
        }
    }

    /// The coverage row's `detail`: which extractors produced rows, and what
    /// failed if anything did.
    pub fn detail(&self) -> String {
        let ran = self.identities.join(",");
        match &self.failure {
            Some(failure) if ran.is_empty() => failure.clone(),
            Some(failure) => format!("{ran}; {failure}"),
            None => ran,
        }
    }
}

/// Run every extractor that claims `relative` over one resource's decoded
/// text.
///
/// Pure (F6's adapter-shape mandate): no file is opened, no database is
/// touched, no clock is read. `content_id` is the resource's already-computed
/// content identity — a BLAKE3 hash for a filesystem resource, a blob OID for
/// a Git one — and `keys` says which space to compose it in.
///
/// A grammar failure is not a scan failure. It leaves [`ResourceExtraction`]'s
/// `syntax` empty and its `failure` set, and the structure units the other
/// extractor produced are still real evidence about real bytes; refusing them
/// too would discard a document's index because a parser disagreed with its
/// code fences. What is never done is the middle case the grammar itself
/// refuses — a *shorter* symbol list, indistinguishable from a complete one
/// (see [`super::syntax`]).
pub fn extract_resource(
    claims: Claims,
    text: &str,
    content_id: &str,
    keys: KeySpace,
) -> ResourceExtraction {
    let mut out = ResourceExtraction {
        extractor: claims.structure,
        key: keys.key(content_id, claims.structure),
        units: extract_units(text, claims.structure, claims.language),
        syntax: None,
        identities: vec![claims.structure.to_string()],
        failure: None,
    };
    let Some(language) = claims.language else {
        return out;
    };
    let extractor = language.extractor_identity();
    match crate::runtime::atlas::syntax::extract(language, text.as_bytes()) {
        Ok(facts) => {
            out.syntax = Some(ScannedSyntax {
                language: language.name(),
                syntax_key: keys.key(content_id, &extractor),
                symbols: facts
                    .symbols
                    .into_iter()
                    .enumerate()
                    .map(|(ordinal, symbol)| ScannedSymbol {
                        ordinal: ordinal as u64,
                        label: symbol.label,
                        name: symbol.name,
                        byte_start: symbol.byte_start as u64,
                        byte_end: symbol.byte_end as u64,
                    })
                    .collect(),
                edges: facts
                    .imports
                    .into_iter()
                    .enumerate()
                    .map(|(ordinal, import)| ScannedEdge {
                        ordinal: ordinal as u64,
                        kind: EDGE_IMPORT,
                        target: import.target,
                        byte_start: import.byte_start as u64,
                        byte_end: import.byte_end as u64,
                    })
                    .collect(),
                extractor: extractor.clone(),
            });
            out.identities.push(extractor);
        }
        Err(error) => out.failure = Some(format!("{extractor}: {error}")),
    }
    out
}

/// Run one extractor over decoded text and number its units.
///
/// The one place `StructureUnit` becomes `ScannedUnit`, for every source kind
/// there is. Three walks — the filesystem one below, the estate-git one
/// ([`super::git`]) and a Work overlay ([`super::overlay`]) — reach identical
/// bytes by different routes, and F7's premise is that identical bytes plus an
/// identical extractor identity are *one* extraction. Two copies of this loop
/// would be two ways for that to stop being true.
///
/// `language` is [`Claims::language`] — `Some` exactly when a grammar
/// claims this path (X3b's union). Markdown always keeps `markdown_units`
/// (A1 §6.1's heading/section structure), and a genuinely plain-text path
/// (`extractor == TEXT_EXTRACTOR`, `language == None` — semble.txt/.text,
/// nothing else claims it) keeps `plain_units`'s single whole-document unit
/// per R1 (no structure this build can honestly claim to see). The one path
/// that changes here (chunker-wire,
/// `knowledge/evidence/resources/host-atlas-s6-series/brief-chunker-wire.md`):
/// a path a *grammar* claims but the structure table does not —
/// `main.rs`, `Cargo.toml` — used to fall onto that same whole-document
/// branch through [`TEXT_EXTRACTOR`] (`claims_for`'s own union, this
/// module's doc above); it now gets [`super::chunk::chunk_source`]'s
/// retrieval-sized spans instead, the code/config family's equivalent of
/// Markdown's section units.
pub fn extract_units(
    text: &str,
    extractor: &str,
    language: Option<SyntaxLanguage>,
) -> Vec<ScannedUnit> {
    // F-SI-01 (R2: reuse existing structure). Markdown and genuinely
    // plain-text both land on the same `Vec<StructureUnit>` ->
    // `ScannedUnit` mapping; the code/config `chunk_source` branch below is
    // the only shape that actually differs, so it alone stays a separate
    // arm. `extractor != MARKDOWN_EXTRACTOR && language.is_some()` is
    // exactly the pre-chunker-wire "structure vs. chunk_source" split.
    if extractor != MARKDOWN_EXTRACTOR
        && let Some(language) = language
    {
        return crate::runtime::atlas::chunk::chunk_source(text, Some(language))
            .into_iter()
            .enumerate()
            .map(|(ordinal, chunk)| ScannedUnit {
                ordinal: ordinal as u64,
                // A chunk is a byte-delimited span within a document, the
                // same shape Markdown's own non-Document unit is (R2:
                // reuse `UnitKind::Section` rather than add a third
                // variant every `UnitKind::as_str`/`parse`/DB consumer
                // would then need to learn, for a unit that is not
                // heading-delimited but is exactly as much "not the
                // whole document" as a section is).
                kind: UnitKind::Section,
                heading_level: None,
                title: None,
                byte_start: chunk.byte_start as u64,
                byte_end: chunk.byte_end as u64,
                coordinate: None,
                text: chunk.content,
            })
            .collect();
    }
    let structure = if extractor == MARKDOWN_EXTRACTOR {
        markdown_units(text)
    } else {
        plain_units(text)
    };
    structure
        .into_iter()
        .enumerate()
        .map(|(ordinal, unit)| ScannedUnit {
            ordinal: ordinal as u64,
            kind: unit.kind,
            heading_level: unit.heading_level,
            title: unit.title,
            byte_start: unit.byte_start as u64,
            byte_end: unit.byte_end as u64,
            // A text/Markdown unit's byte span IS its address, so there
            // is no second, native one to carry
            // (`ScannedUnit::coordinate`).
            coordinate: None,
            text: text[unit.byte_start..unit.byte_end].to_string(),
        })
        .collect()
}

/// Modification time in Unix milliseconds, when the platform offers one.
fn mtime_millis(meta: &std::fs::Metadata) -> Option<i64> {
    meta.modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::atlas::text::TEXT_EXTRACTOR;

    /// Build a source tree and scan it.
    fn scan_tree(files: &[(&str, &[u8])], ignore: &[&str]) -> (tempfile::TempDir, SourceScan) {
        let dir = tempfile::tempdir().expect("tempdir");
        for (path, bytes) in files {
            let full = dir.path().join(path);
            std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
            std::fs::write(&full, bytes).expect("write");
        }
        let source = KnowledgeSource {
            name: "notes".to_string(),
            root: dir.path().to_path_buf(),
            ignore: ignore.iter().map(|s| (*s).to_string()).collect(),
            context_fields: ContextFields::none(),
        };
        let scan = scan_local_knowledge(&source).expect("scan");
        (dir, scan)
    }

    fn row<'a>(scan: &'a SourceScan, path: &str) -> &'a CoverageRow {
        scan.coverage
            .iter()
            .find(|r| r.path.as_deref() == Some(path))
            .unwrap_or_else(|| panic!("no coverage row for {path:?}"))
    }

    /// F8: every path the walk saw leaves exactly one row, and the statuses
    /// are the ones the situation actually warrants.
    #[test]
    fn every_path_seen_leaves_exactly_one_coverage_row() {
        let (_dir, scan) = scan_tree(
            &[
                ("README.md", b"# Top\n\nbody\n"),
                ("notes/one.md", b"# One\n"),
                ("notes/plain.txt", b"just text\n"),
                ("notes/binary.bin", b"\x00\x01\x02"),
                ("notes/image.png", &[0x89, 0x50, 0x4e, 0x47]),
                (".env", b"SECRET=1\n"),
                ("keys/server.pem", b"-----BEGIN\n"),
            ],
            &[],
        );
        let paths: Vec<&str> = scan
            .coverage
            .iter()
            .filter_map(|r| r.path.as_deref())
            .collect();
        let mut sorted = paths.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(paths.len(), sorted.len(), "a path got two rows: {paths:?}");

        assert_eq!(row(&scan, "README.md").status, Coverage::Indexed);
        assert_eq!(row(&scan, "notes/one.md").status, Coverage::Indexed);
        assert_eq!(row(&scan, "notes/plain.txt").status, Coverage::Indexed);
        // `.bin` has no extractor at all; `.png` neither. Both unsupported,
        // and neither pretends to be indexed.
        assert_eq!(row(&scan, "notes/binary.bin").status, Coverage::Unsupported);
        assert_eq!(row(&scan, "notes/image.png").status, Coverage::Unsupported);
        assert_eq!(row(&scan, ".env").status, Coverage::Excluded);
        assert_eq!(row(&scan, "keys/server.pem").status, Coverage::Excluded);
        assert_eq!(row(&scan, "notes").status, Coverage::Discovered);

        // Both routing tables ran. A Markdown file is claimed by the structure
        // extractor *and* by the Markdown grammar, and that is two extractions
        // of one blob with two F7 keys — never one extraction that could have
        // been done two ways (X3b).
        assert_eq!(
            scan.extractors,
            BTreeSet::from([
                MARKDOWN_EXTRACTOR.to_string(),
                TEXT_EXTRACTOR.to_string(),
                SyntaxLanguage::Markdown.extractor_identity(),
            ])
        );
    }

    /// F10, stated where it can actually fail: an excluded file's bytes are
    /// counted and its refusing pattern named, and no unit anywhere carries
    /// its contents.
    #[test]
    fn excluded_bytes_are_counted_and_never_reach_a_unit() {
        let secret = b"SECRET=hunter2\n";
        let (_dir, scan) = scan_tree(
            &[
                ("keep.md", b"# Keep\n"),
                (".env", secret),
                ("build/out.md", b"# Generated\n"),
            ],
            &["build/**"],
        );
        let env = row(&scan, ".env");
        assert_eq!(env.status, Coverage::Excluded);
        assert_eq!(env.bytes, Some(secret.len() as u64));
        assert!(
            env.detail.as_deref().is_some_and(|d| d.contains("dotfile")),
            "{env:?}"
        );
        assert_eq!(row(&scan, "build/out.md").status, Coverage::Excluded);
        assert!(
            row(&scan, "build/out.md")
                .detail
                .as_deref()
                .is_some_and(|d| d.contains("build/**"))
        );

        assert_eq!(scan.files.len(), 1, "only the allowed file was acquired");
        let all_text: String = scan
            .files
            .iter()
            .flat_map(|f| f.units.iter().map(|u| u.text.clone()))
            .collect();
        assert!(!all_text.contains("hunter2"), "secret reached a unit");
        assert!(
            !scan.files.iter().any(|f| f.relative_path == ".env"),
            "an excluded path was acquired"
        );
    }

    /// A denied *directory* is refused once and never descended into, so its
    /// children cost nothing and appear nowhere.
    #[test]
    fn a_denied_directory_is_refused_once_and_not_descended() {
        let (_dir, scan) = scan_tree(
            &[
                ("keep.md", b"# Keep\n"),
                (".git/config", b"[core]\n"),
                (".git/objects/deep/thing", b"binary"),
            ],
            &[],
        );
        assert_eq!(row(&scan, ".git").status, Coverage::Excluded);
        assert!(
            !scan
                .coverage
                .iter()
                .any(|r| r.path.as_deref().is_some_and(|p| p.starts_with(".git/"))),
            "the walk descended into a denied directory: {:?}",
            scan.coverage
        );
    }

    /// F7: the local key is content plus extractor. Two files with identical
    /// bytes share one; touching a file changes nothing; editing it changes
    /// the key and the generation identity together.
    #[test]
    fn keys_are_content_and_extractor_and_mtime_is_only_a_hint() {
        let (dir, first) = scan_tree(&[("a.md", b"# Same\n"), ("copy/b.md", b"# Same\n")], &[]);
        assert_eq!(first.files.len(), 2);
        assert_eq!(first.files[0].content_hash, first.files[1].content_hash);
        assert_eq!(first.files[0].local_key, first.files[1].local_key);
        assert_eq!(first.files[0].extractor, MARKDOWN_EXTRACTOR);

        let source = KnowledgeSource {
            name: "notes".to_string(),
            root: dir.path().to_path_buf(),
            ignore: Vec::new(),
            context_fields: ContextFields::none(),
        };
        // Touch: mtime moves, content does not. Nothing derived may move.
        let touched = dir.path().join("a.md");
        let now = std::time::SystemTime::now() + std::time::Duration::from_secs(120);
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&touched)
            .expect("open");
        file.set_modified(now).expect("set mtime");
        drop(file);
        let second = scan_local_knowledge(&source).expect("rescan");
        assert_eq!(second.content_key, first.content_key, "mtime moved a key");
        assert_eq!(second.files[0].local_key, first.files[0].local_key);

        // Edit: content moves, so the key and the generation identity move.
        std::fs::write(&touched, b"# Same but different\n").expect("write");
        let third = scan_local_knowledge(&source).expect("rescan");
        assert_ne!(third.content_key, first.content_key);
        assert_ne!(third.files[0].local_key, first.files[0].local_key);
    }

    /// Units carry provenance into the original bytes, and the text stored is
    /// exactly the bytes at those offsets.
    #[test]
    fn units_slice_back_out_of_the_original_file() {
        let body = "# Title\n\nfirst\n\n## Sub\n\nsecond\n";
        let (dir, scan) = scan_tree(&[("doc.md", body.as_bytes())], &[]);
        let file = &scan.files[0];
        let original = std::fs::read(dir.path().join("doc.md")).expect("read");
        assert!(file.units.len() >= 3);
        for unit in &file.units {
            let slice = &original[unit.byte_start as usize..unit.byte_end as usize];
            assert_eq!(
                unit.text.as_bytes(),
                slice,
                "unit {} does not match its own offsets",
                unit.ordinal
            );
        }
        assert_eq!(file.units[0].kind, UnitKind::Document);
        assert_eq!(file.units[0].byte_end, body.len() as u64);
    }

    /// A missing root is a coverage fact, not a failure — otherwise one
    /// unplugged external drive stops every other source being scanned.
    #[test]
    fn a_missing_root_reports_unavailable_rather_than_failing() {
        let source = KnowledgeSource {
            name: "gone".to_string(),
            root: PathBuf::from("/nonexistent/knowledge/root"),
            ignore: Vec::new(),
            context_fields: ContextFields::none(),
        };
        let scan = scan_local_knowledge(&source).expect("scan must not fail");
        assert!(scan.files.is_empty());
        assert_eq!(scan.coverage.len(), 1);
        assert_eq!(scan.coverage[0].status, Coverage::Unavailable);
        assert_eq!(scan.coverage[0].path, None);
        assert!(
            scan.root_unavailable().is_some(),
            "an unreachable root must be distinguishable from an empty one"
        );
    }

    /// The signal that lets ruling §4 tell "the bytes are gone" from "the
    /// path is gone": an empty *readable* directory reports no root
    /// unavailability, and neither does a file-level one.
    #[test]
    fn an_empty_readable_root_is_not_an_unavailable_root() {
        let (dir, empty) = scan_tree(&[], &[]);
        assert!(empty.files.is_empty());
        assert!(
            empty.root_unavailable().is_none(),
            "a readable, genuinely empty root is a real observation of \
             emptiness: {:?}",
            empty.coverage
        );

        // A file the walk could not read is a *file's* unavailability. The
        // root was listed perfectly well, so nothing about the source's own
        // reachability is in doubt.
        std::fs::write(dir.path().join("keep.md"), b"# Keep\n").expect("write");
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc", dir.path().join("escape")).expect("symlink");
        let source = KnowledgeSource {
            name: "notes".to_string(),
            root: dir.path().to_path_buf(),
            ignore: Vec::new(),
            context_fields: ContextFields::none(),
        };
        let scan = scan_local_knowledge(&source).expect("scan");
        assert!(scan.root_unavailable().is_none(), "{:?}", scan.coverage);
    }

    /// A symlink is not followed: a knowledge source's boundary is the
    /// declared path, and a link can leave it entirely.
    #[cfg(unix)]
    #[test]
    fn a_symlink_is_reported_and_not_followed() {
        let (dir, _) = scan_tree(&[("real.md", b"# Real\n")], &[]);
        std::os::unix::fs::symlink("/etc", dir.path().join("escape")).expect("symlink");
        let source = KnowledgeSource {
            name: "notes".to_string(),
            root: dir.path().to_path_buf(),
            ignore: Vec::new(),
            context_fields: ContextFields::none(),
        };
        let scan = scan_local_knowledge(&source).expect("scan");
        let link = row(&scan, "escape");
        assert_eq!(link.status, Coverage::Unavailable);
        assert!(
            link.detail
                .as_deref()
                .is_some_and(|d| d.contains("symlink")),
            "{link:?}"
        );
        assert_eq!(scan.files.len(), 1);
    }

    /// Coverage order is the walk's sorted path order, so two scans of an
    /// unchanged tree are byte-identical evidence.
    #[test]
    fn an_unchanged_tree_scans_identically_twice() {
        let (dir, first) = scan_tree(
            &[("b.md", b"# B\n"), ("a.md", b"# A\n"), ("z/c.md", b"# C\n")],
            &[],
        );
        let source = KnowledgeSource {
            name: "notes".to_string(),
            root: dir.path().to_path_buf(),
            ignore: Vec::new(),
            context_fields: ContextFields::none(),
        };
        let second = scan_local_knowledge(&source).expect("rescan");
        assert_eq!(first.coverage, second.coverage);
        assert_eq!(first.files, second.files);
        assert_eq!(first.content_key, second.content_key);
    }

    // --------------------------------- S4 Y6, G7/A1-06: online-only detection

    /// A sparse file — `st_size > 0`, `st_blocks == 0` — is `truncate`'s own
    /// documented effect on ext4/most Linux filesystems (verified in this
    /// wave's own sandbox, not merely assumed), so it is the honest stand-in
    /// the brief asked for: it exercises the exact stat divergence the
    /// heuristic reads, but it is **not** a true cloud placeholder — nothing
    /// synced it, nothing would hydrate it, and a real placeholder on a real
    /// cloud-sync client might diverge from this shape (the false-negative
    /// case [`ONLINE_ONLY_DETAIL`] itself names). Said here plainly rather
    /// than left for a reader to assume otherwise.
    ///
    /// Probe-gated per `CONTRIBUTING.md`'s two-environment rule: a
    /// filesystem that does not actually leave the hole unallocated (some
    /// non-ext4 CI mount, a copy-on-write remount) makes this a fact about
    /// the environment, not the code under test, so the assertion is
    /// skipped loudly rather than failing on a precondition nothing in this
    /// build controls.
    #[cfg(unix)]
    fn make_sparse_file(path: &Path, apparent_len: u64) -> bool {
        use std::os::unix::fs::MetadataExt;
        let file = std::fs::File::create(path).expect("create");
        file.set_len(apparent_len).expect("set_len");
        drop(file);
        let meta = std::fs::symlink_metadata(path).expect("stat");
        meta.size() == apparent_len && meta.blocks() == 0
    }

    #[cfg(unix)]
    #[test]
    fn a_suspected_placeholder_is_never_indexed_as_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("report.md");
        if !make_sparse_file(&path, 1_048_576) {
            eprintln!(
                "SKIPPED-ENV: this filesystem did not leave a sparse file unallocated \
                 (st_blocks != 0 after set_len) — the divergence this test exercises is a \
                 property of the filesystem, not of the code under test"
            );
            return;
        }
        let source = KnowledgeSource {
            name: "notes".to_string(),
            root: dir.path().to_path_buf(),
            ignore: Vec::new(),
            context_fields: ContextFields::none(),
        };
        let scan = scan_local_knowledge(&source).expect("scan");
        let entry = row(&scan, "report.md");
        assert_eq!(
            entry.status,
            Coverage::OnlineOnly,
            "a suspected placeholder must never be reported Indexed with zero units — the \
             exact 'silently indexed as empty' case acceptance item 4 forbids: {entry:?}"
        );
        assert!(
            entry
                .detail
                .as_deref()
                .is_some_and(|d| d.contains("best-effort") && d.contains("false")),
            "the row's own detail must state the heuristic's honesty, not just a doc comment: \
             {entry:?}"
        );
        assert!(
            scan.files.is_empty(),
            "a suspected placeholder must never be opened — files should be empty, not a \
             zero-unit entry: {:?}",
            scan.files
        );
    }

    /// The mirror negative: a real, ordinary empty file is not a placeholder
    /// and must not be misreported as one. `Indexed` with zero units is the
    /// TRUE answer for a file that really is empty — turning it into
    /// `online_only` would be the opposite dishonesty.
    #[test]
    fn a_genuinely_empty_file_is_indexed_not_flagged_online_only() {
        let (_dir, scan) = scan_tree(&[("empty.md", b"")], &[]);
        let entry = row(&scan, "empty.md");
        assert_eq!(entry.status, Coverage::Indexed, "{entry:?}");
    }

    /// The same detection, over the dataset path (`file()`'s `dataset()`
    /// twin): a sparse CSV is caught before [`hash_file`] would open it, not
    /// registered as a dataset with a hash of nothing.
    #[cfg(unix)]
    #[test]
    fn a_suspected_placeholder_dataset_is_never_registered() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("rows.csv");
        if !make_sparse_file(&path, 65_536) {
            eprintln!(
                "SKIPPED-ENV: this filesystem did not leave a sparse file unallocated — see \
                 `a_suspected_placeholder_is_never_indexed_as_empty`'s own note"
            );
            return;
        }
        let source = KnowledgeSource {
            name: "notes".to_string(),
            root: dir.path().to_path_buf(),
            ignore: Vec::new(),
            context_fields: ContextFields::none(),
        };
        let scan = scan_local_knowledge(&source).expect("scan");
        let entry = row(&scan, "rows.csv");
        assert_eq!(entry.status, Coverage::OnlineOnly, "{entry:?}");
        assert!(
            scan.datasets.is_empty(),
            "a suspected placeholder must never be registered as a dataset: {:?}",
            scan.datasets
        );
    }

    /// The coverage vocabulary check item 4's own tripwire used to pin now
    /// lives beside the row it describes rather than as a negative
    /// assertion elsewhere — see `tests/x5_a1a_acceptance.rs`'s updated
    /// register row 4 for the acceptance-level pin.
    #[test]
    fn online_only_is_a_named_coverage_state_with_its_own_wire_spelling() {
        assert_eq!(Coverage::OnlineOnly.as_str(), "online_only");
        assert_eq!(Coverage::parse("online_only"), Some(Coverage::OnlineOnly));
        assert!(Coverage::ALL.contains(&Coverage::OnlineOnly));
    }

    /// chunker-wire (host-atlas-s6-series/brief-chunker-wire.md item 3): a
    /// grammar-claimed, structure-unclaimed file (`.rs`) that today falls
    /// onto `plain_units`'s single whole-file [`crate::domain::source::UnitKind::Document`]
    /// unit must instead come back as several [`crate::domain::source::UnitKind::Section`]
    /// chunk units, none of them the whole file — while a planted secret
    /// and the estate's own `.sergeant/data/` machinery both stay excluded
    /// exactly as they did before this wave (J5, unaffected by this change).
    #[test]
    fn a_grammar_claimed_file_is_chunked_not_left_as_one_document_unit() {
        // Comfortably over `chunk::DESIRED_CHUNK_LENGTH` (750 bytes) many
        // times over, so the AST-merge path has to emit more than one chunk.
        let mut big_rs = String::new();
        for n in 0..60 {
            big_rs.push_str(&format!(
                "/// Doc comment for function number {n}.\npub fn function_{n}(x: i32) -> i32 {{\n    x + {n}\n}}\n\n"
            ));
        }
        assert!(
            big_rs.len() > 2048,
            "fixture must be >2KB per the brief's own end-to-end case: {}",
            big_rs.len()
        );
        let secret = b"SECRET=hunter3\n";
        let (_dir, scan) = scan_tree(
            &[
                ("big.rs", big_rs.as_bytes()),
                (".env", secret),
                (
                    ".sergeant/data/atlas.duckdb",
                    b"not a real duckdb file, just bytes",
                ),
            ],
            &[],
        );

        // The new unit shape.
        let rs_file = scan
            .files
            .iter()
            .find(|f| f.relative_path == "big.rs")
            .expect("big.rs was acquired");
        assert_eq!(rs_file.extractor, TEXT_EXTRACTOR);
        assert!(
            rs_file.units.len() > 1,
            "expected multiple chunk units for a {}-byte grammar-claimed file, got {}: {:?}",
            big_rs.len(),
            rs_file.units.len(),
            rs_file.units
        );
        for unit in &rs_file.units {
            let span = (unit.byte_end - unit.byte_start) as usize;
            assert!(
                span > 0 && span < big_rs.len(),
                "unit {} spans {span} bytes of a {}-byte file — it must be a \
                 chunk, not the whole document",
                unit.ordinal,
                big_rs.len()
            );
            assert!(
                unit.byte_start as usize <= big_rs.len() && unit.byte_end as usize <= big_rs.len(),
                "unit {} out of bounds: {}..{} in a {}-byte file",
                unit.ordinal,
                unit.byte_start,
                unit.byte_end,
                big_rs.len()
            );
        }
        // Ordinals are dense and spans move forward — sane coordinates
        // (A2 §9), not an arbitrary bag of slices.
        for (index, unit) in rs_file.units.iter().enumerate() {
            assert_eq!(unit.ordinal, index as u64);
        }
        for pair in rs_file.units.windows(2) {
            assert!(
                pair[1].byte_start >= pair[0].byte_start,
                "chunk order regressed: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }

        // Markdown's own unit shape is untouched by this change.
        assert_eq!(row(&scan, "big.rs").status, Coverage::Indexed);

        // The planted secret never reaches a unit's text, exactly as before.
        let all_text: String = scan
            .files
            .iter()
            .flat_map(|f| f.units.iter().map(|u| u.text.clone()))
            .collect();
        assert!(!all_text.contains("hunter3"), "secret reached a unit");
        assert!(
            !scan.files.iter().any(|f| f.relative_path == ".env"),
            "an excluded path was acquired"
        );
        assert!(
            !scan
                .files
                .iter()
                .any(|f| f.relative_path.starts_with(".sergeant/data/")),
            "estate machinery under .sergeant/data/ was acquired"
        );
    }

    /// Round-trips this repository's own `deny.toml` (a real, evolving,
    /// em-dash-bearing file, not a synthetic fixture) through the actual
    /// wired entry point (`scan_local_knowledge` -> [`extract_units`] ->
    /// [`super::chunk::chunk_source`]), asserting every unit lands on a
    /// valid char boundary, its recorded text matches its own span exactly,
    /// and no em dash present in the source is lost or duplicated across
    /// the chunk set. A grammar-claimed, structure-unclaimed extension
    /// (`.toml`) is required so this file routes through `chunk_source`
    /// rather than `plain_units` (see
    /// `a_grammar_claimed_file_is_chunked_not_left_as_one_document_unit`
    /// above, same routing, ASCII-only fixture).
    ///
    /// # What this does — and does not — prove (F-TH-01)
    ///
    /// This is **not** a targeted reproduction of the specific panic
    /// `brief-chunker-utf8.md` and `00-orient` measured (byte offset 3592).
    /// `deny.toml` has since grown past that measurement, and — verified by
    /// reverting [`super::chunk::finalize_chunk`]'s char-boundary snapping
    /// and re-running this exact test — the file's current committed
    /// content does not drive any chunk boundary mid-codepoint even with
    /// the pre-fix, panic-prone slicing restored, so this test alone would
    /// not catch a regression that reintroduced the panic; `00-orient`'s
    /// own investigation could not fully bottom out the natural-occurrence
    /// mechanism either (see [`super::chunk::finalize_chunk`]'s doc
    /// comment). The targeted, always-reproducing case lives in
    /// `chunk::tests::finalize_chunk_never_panics_on_a_boundary_that_splits_a_multibyte_char`
    /// and `chunk::tests::finalize_chunk_never_duplicates_content_across_adjacent_boundaries_sharing_a_split_point`,
    /// which construct the mid-codepoint boundary directly rather than
    /// relying on a real file happening to produce one. What this test
    /// does provide: a live, real-world regression net against silent
    /// content loss or duplication should `deny.toml`'s content ever
    /// change to include such a seam.
    #[test]
    fn a_real_multibyte_file_scans_without_panicking_and_keeps_every_codepoint_intact() {
        let deny_toml = include_str!("../../../deny.toml");
        assert!(
            deny_toml.len() != deny_toml.chars().count(),
            "fixture must actually be non-ASCII for this test to mean anything: {} bytes, {} chars",
            deny_toml.len(),
            deny_toml.chars().count()
        );

        let (_dir, scan) = scan_tree(&[("deny.toml", deny_toml.as_bytes())], &[]);

        let file = scan
            .files
            .iter()
            .find(|f| f.relative_path == "deny.toml")
            .expect("deny.toml was acquired");
        assert!(
            file.units.len() > 1,
            "expected multiple chunk units for an {}-byte file, got {}",
            deny_toml.len(),
            file.units.len()
        );

        // Every unit's recorded span must be a valid, in-bounds byte range
        // into the real file, and re-slicing that exact range must
        // reproduce the unit's own text — the same property that panicked
        // in production when a span landed mid-codepoint.
        for unit in &file.units {
            let (start, end) = (unit.byte_start as usize, unit.byte_end as usize);
            assert!(
                start <= end && end <= deny_toml.len(),
                "unit {} out of bounds: {start}..{end} in a {}-byte file",
                unit.ordinal,
                deny_toml.len()
            );
            assert!(
                deny_toml.is_char_boundary(start) && deny_toml.is_char_boundary(end),
                "unit {} span {start}..{end} does not land on a char boundary",
                unit.ordinal
            );
            assert_eq!(
                unit.text,
                deny_toml[start..end],
                "unit {}'s recorded text must match what its own span actually slices",
                unit.ordinal
            );
        }

        // No multi-byte codepoint present in the source is lost across the
        // full set of chunks — concatenating every unit's text must still
        // contain each of the fixture's own em dashes (`—`, U+2014).
        let dash_count_in_source = deny_toml.matches('\u{2014}').count();
        assert!(dash_count_in_source > 0, "fixture must contain an em dash");
        let dash_count_in_units: usize = file
            .units
            .iter()
            .map(|u| u.text.matches('\u{2014}').count())
            .sum();
        assert_eq!(
            dash_count_in_units, dash_count_in_source,
            "every em dash in the source must survive chunking exactly once"
        );
    }
}

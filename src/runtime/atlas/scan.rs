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
//!   excluded, unavailable, unsupported or error: there is no sixth outcome
//!   where a path is silently not mentioned. One row per *path*, not one per
//!   extractor: a resource two extractors claimed still has one row, whose
//!   `detail` names both, and whose status is `error` if either failed.
//!
//! # The shared extraction (X3b)
//!
//! Three walks acquire bytes by three different routes — this module's
//! filesystem walk, [`super::git`]'s object-store read, and
//! [`super::overlay`]'s Work surface. They all extract through exactly two
//! functions here: [`claims_for`], which decides from a path alone what claims
//! it, and [`extract_resource`], which runs every claiming extractor over the
//! bytes. Three copies of that would be three ways for F7's premise —
//! identical bytes plus an identical extractor identity are one extraction —
//! to stop being true.
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
    AuthorityClass, Coverage, CoverageRow, SourceKind, UnitKind, content_hash, estate_git_key,
    generation_key, local_key,
};
use crate::runtime::atlas::deny::{AcquisitionFilter, BadPattern, Verdict};
use crate::runtime::atlas::syntax::{SyntaxLanguage, language_for};
use crate::runtime::atlas::tabular::{ContextFields, ScannedDataset, format_for, reader_identity};
use crate::runtime::atlas::text::{
    MARKDOWN_EXTRACTOR, TEXT_EXTRACTOR, as_text, extractor_for, markdown_units, plain_units,
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
    /// **F10a**: the operator-declared column allowlist for this source,
    /// defaulting to none. Governs whether a dataset row's text may become a
    /// context unit, and nothing else.
    pub context_fields: ContextFields,
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
    let filter = AcquisitionFilter::new(&source.ignore)?;
    let mut walk = Walk {
        filter: &filter,
        context_fields: &source.context_fields,
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
        let Some(claims) = claims_for(&relative) else {
            self.coverage.push(CoverageRow {
                path: Some(relative),
                status: Coverage::Unsupported,
                detail: Some(UNCLAIMED.to_string()),
                bytes: Some(meta.len()),
            });
            return;
        };
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
            content_hash: hash,
            extractor: extracted.extractor.to_string(),
            byte_len: bytes.len() as u64,
            mtime_millis: mtime_millis(&meta),
            units: extracted.units,
            syntax: extracted.syntax,
        });
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
/// [`estate_git_key`]/[`local_key`] themselves — this enum only chooses
/// between them, and cannot blur them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySpace {
    /// BLAKE3 of the bytes plus extractor identity — a filesystem resource,
    /// which no object store has already named.
    Local,
    /// Git blob OID plus extractor identity — **never a second hash of bytes
    /// Git already hashed**.
    EstateGit,
}

impl KeySpace {
    /// Compose a content identity with an extractor identity in this space.
    pub fn key(self, content_id: &str, extractor: &str) -> String {
        match self {
            Self::Local => local_key(content_id, extractor),
            Self::EstateGit => estate_git_key(content_id, extractor),
        }
    }
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
        units: extract_units(text, claims.structure),
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
pub fn extract_units(text: &str, extractor: &str) -> Vec<ScannedUnit> {
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
}

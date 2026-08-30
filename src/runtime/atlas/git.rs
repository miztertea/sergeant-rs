//! The estate-git walk: Atlas's `estate_git` source reader (X3a).
//!
//! One declared `[[repo]]` mount, read **at the SHA admission pinned**, out of
//! the object store and never out of the working tree. Filesystem in is
//! [`super::scan`]'s job; this module's input is a commit, and its output is
//! the same plain-Rust [`SourceScan`] that module produces — which is what
//! lets one set of `source.*` rows, one coverage vocabulary and one
//! crash-window discipline serve both source kinds (R2).
//!
//! # The pin is the whole point
//!
//! §8.2 pins a Work's base commit at admission so materialization uses the
//! commit preflight judged rather than whatever HEAD has since become. Atlas
//! inherits that pin for *reads*: every byte this module returns is addressed
//! by a commit SHA the caller supplied, and Git resolves that SHA in the
//! object store. A mount whose HEAD advances mid-scan — Captain committing,
//! another Work, an unrelated process — changes no object this scan is
//! reading, because a commit's tree is immutable and reachable regardless of
//! what any ref points at.
//!
//! The advance is nonetheless *observed*, never blended:
//! [`observe_drift`] asks the mount for its committed HEAD and reports an
//! [`EstateDriftObservation`] when it is no longer the pinned SHA (§11.4's
//! existing vocabulary, reused — R2). A scan therefore answers with one
//! world plus an honest note that the mount has moved on, and never with a
//! mixture of two.
//!
//! # What this module will not do
//!
//! Never fetch, pull, switch, reset, or write. Never read the mount's working
//! tree or index. `ls-tree` and `cat-file` are the entire Git surface used
//! here, and both are pure object reads. The estate's own rule — workers do
//! not edit a `repos/` mount — applies with more force to a derived index
//! that has no business writing anything at all.
//!
//! # Rungs
//!
//! **R2, over the brief's R5.** The X3a brief names `git2` and calls it "a
//! dependency already"; it is not one, in this tree, at this commit — nothing
//! in `Cargo.toml` or `Cargo.lock` names `git2` or `libgit2-sys`. Adopting it
//! would be a new native dependency (the brief's own "no new heavy
//! dependency" exclusion) *and* a reversal of proposal §11, which is explicit
//! that this codebase shells out to the installed Git rather than embedding
//! libgit2 — a governing constraint, so J5. The requirement the brief actually
//! states — read-only object access at a pinned SHA, with batched blob reads —
//! is satisfied whole at R2 by [`crate::runtime::git`], which already owns
//! every Git invocation this codebase makes. The one thing it lacked was a
//! batch primitive, and that is now
//! [`crate::runtime::git::git_cat_file_batch`]: one Git
//! process for many objects, rather than one process per file.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::domain::event::rfc3339_utc_now;
use crate::domain::source::{AuthorityClass, Coverage, CoverageRow, SourceKind};
use crate::runtime::atlas::deny::{AcquisitionFilter, BadPattern, Verdict};
use crate::runtime::atlas::scan::{
    DATASET_NO_ROOT, KeySpace, MAX_RESOURCE_BYTES, ScannedFile, SourceScan, UNCLAIMED, claims_for,
    dispatch_worker_resource, extract_resource, worker_extractor_for,
};
use crate::runtime::atlas::tabular::{ContextFields, format_for};
use crate::runtime::atlas::text::as_text;
use crate::runtime::atlas::worker::WorkerRuntime;
use crate::runtime::git::{GitError, git, git_bytes, git_cat_file_batch};
use crate::runtime::integrity::{DriftAttribution, EstateDriftObservation};

/// How many bytes of blob content one `cat-file --batch` invocation is
/// allowed to answer with before the next batch is started.
///
/// **Declared, not measured**, and said plainly rather than dressed up: it
/// bounds the peak heap one batch can cost, which is the only property the
/// number has to have. 64 MiB is comfortably above any single blob this
/// module will accept (the per-resource ceiling is
/// [`MAX_RESOURCE_BYTES`], 4 MiB) — so a batch is never fewer than one file —
/// and far below anything that would matter on a host already running a
/// bundled analytical database. A repository whose blobs total less than this
/// is read in exactly one Git process, which is the case the batching exists
/// for.
pub const BATCH_BYTE_BUDGET: u64 = 64 * 1024 * 1024;

/// How many objects one `cat-file --batch` invocation is allowed to request.
///
/// The companion bound to [`BATCH_BYTE_BUDGET`], for the opposite shape: ten
/// thousand tiny files cost nothing in bytes and would still build a
/// ten-thousand-line request and a ten-thousand-element answer before any of
/// it was consumed. Also declared, not measured.
pub const BATCH_OBJECT_LIMIT: usize = 1024;

/// One declared repository mount, to be read at one pinned commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstateGitSource {
    /// Declared repository name — the source coordinate derived rows carry.
    pub name: String,
    /// The estate's mount, `<estate-root>/repos/<name>`. Read-only here.
    pub mount: PathBuf,
    /// §8.2's admission-pinned commit: the exact world this scan is of.
    pub pinned_sha: String,
    /// Per-source ignore globs, extending the built-in deny set (F10).
    pub ignore: Vec<String>,
}

/// What one tree entry is, by its Git file mode.
///
/// Three cases rather than "blob or not", because the two non-blob cases have
/// different honest coverage answers and neither is an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// An ordinary file (mode `100644`/`100755`).
    File,
    /// A symlink (mode `120000`): the blob holds a *path*, not content.
    Symlink,
    /// A gitlink (mode `160000`): another repository's commit, whose objects
    /// are not in this object store at all.
    Submodule,
}

/// One blob-level entry of a commit's tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    /// Path relative to the repository root, `/`-separated, as Git spells it.
    pub path: String,
    /// The object id. For a [`EntryKind::File`] this is **F7's content half**:
    /// Git already hashed exactly these bytes, and Atlas never hashes them a
    /// second time.
    pub oid: String,
    /// Git's own file mode word.
    pub mode: String,
    /// What the mode means.
    pub kind: EntryKind,
    /// The object's size in bytes, as `ls-tree --long` reports it — known
    /// **before** any content is read, which is what lets the resource
    /// ceiling refuse an oversized blob without ever fetching it.
    pub size: u64,
}

/// One commit's tree, listed. Phase one of a scan, and a value in its own
/// right: everything below is addressed by [`Self::commit_sha`], so a caller
/// holding this holds a world that cannot move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitTree {
    /// The pinned commit, resolved to a full SHA.
    pub commit_sha: String,
    /// That commit's root tree object id — the **content** identity of the
    /// world, as distinct from the commit's. Two commits with different
    /// messages, authors or parents but identical trees changed no source
    /// bytes, and ruling §4 evicts a generation only when the source bytes
    /// changed; keying the generation on the tree is what makes that literally
    /// true rather than approximately true.
    pub tree_oid: String,
    /// Blob-level entries, in path order.
    pub entries: Vec<TreeEntry>,
}

/// One completed estate-git scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstateGitScan {
    /// The rows, in the shape every `source.*` writer already takes.
    pub scan: SourceScan,
    /// The tree the generation is keyed on (also [`SourceScan::content_key`]).
    pub tree_oid: String,
    /// §11.4's observation, when the mount's committed HEAD is no longer the
    /// SHA this scan pinned. Reported beside the scan, never folded into it.
    pub drift: Option<EstateDriftObservation>,
}

/// Failures of an estate-git scan.
///
/// Deliberately few. Everything the *repository content* can do — an
/// unreadable blob, a submodule, an extension nothing claims, a file past the
/// ceiling — is a coverage row, exactly as in [`super::scan`]. Only a failure
/// that makes the whole scan meaningless is an error.
#[derive(Debug, thiserror::Error)]
pub enum GitScanError {
    /// An `ignore` glob does not compile — an operator error, named not
    /// absorbed.
    #[error(transparent)]
    Pattern(#[from] BadPattern),
    /// Git itself refused: the mount is not a repository, or the pinned SHA
    /// is not in its object store.
    #[error(transparent)]
    Git(#[from] GitError),
    /// `ls-tree` produced a record this build cannot read.
    #[error("git ls-tree of {sha} in {mount} produced an unreadable record: {record:?}")]
    MalformedTree {
        /// The commit being listed.
        sha: String,
        /// The mount it was listed in.
        mount: String,
        /// The record verbatim.
        record: String,
    },
}

/// **Phase one**: list the pinned commit's tree.
///
/// Resolves the caller's SHA to a full commit id and a tree id first, so a
/// short SHA, a tag, or `HEAD` all become an immutable pair before a single
/// entry is read — and so a caller that stores [`GitTree::commit_sha`] stores
/// something unambiguous.
///
/// `ls-tree -r -z --long` is the whole listing: recursive (so trees are
/// flattened to blob paths), `-z` (so a path containing a newline or a quote
/// is delivered verbatim rather than C-quoted), `--long` (so each entry's
/// size arrives with it, before any content is read).
pub fn list_tree(source: &EstateGitSource) -> Result<GitTree, GitScanError> {
    let commit_sha = git(
        &source.mount,
        &[
            "rev-parse",
            "--verify",
            &format!("{}^{{commit}}", source.pinned_sha),
        ],
    )?;
    let tree_oid = git(
        &source.mount,
        &["rev-parse", "--verify", &format!("{commit_sha}^{{tree}}")],
    )?;
    let raw = git_bytes(
        &source.mount,
        &["ls-tree", "-r", "-z", "--long", &commit_sha, "--"],
    )?;
    let mut entries = Vec::new();
    for record in raw.split(|b| *b == 0) {
        if record.is_empty() {
            continue;
        }
        entries.push(parse_tree_record(record, &commit_sha, source)?);
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(GitTree {
        commit_sha,
        tree_oid,
        entries,
    })
}

/// `<mode> SP <type> SP <oid> SP <size> TAB <path>` — `ls-tree --long`'s own
/// record shape, with `size` right-aligned in spaces and `-` for a gitlink.
fn parse_tree_record(
    record: &[u8],
    sha: &str,
    source: &EstateGitSource,
) -> Result<TreeEntry, GitScanError> {
    let malformed = |record: &[u8]| GitScanError::MalformedTree {
        sha: sha.to_string(),
        mount: source.mount.display().to_string(),
        record: String::from_utf8_lossy(record).into_owned(),
    };
    let tab = record
        .iter()
        .position(|b| *b == b'\t')
        .ok_or_else(|| malformed(record))?;
    // A path is bytes, not necessarily UTF-8. One that is not is reported as
    // unsupported later, the same way the filesystem walk reports a non-UTF-8
    // entry name — so it is carried lossily here rather than refused.
    let path = String::from_utf8_lossy(&record[tab + 1..]).into_owned();
    let head = String::from_utf8_lossy(&record[..tab]).into_owned();
    let mut words = head.split_whitespace();
    let mode = words.next().ok_or_else(|| malformed(record))?.to_string();
    let _kind_word = words.next().ok_or_else(|| malformed(record))?;
    let oid = words.next().ok_or_else(|| malformed(record))?.to_string();
    let size_word = words.next().ok_or_else(|| malformed(record))?;
    let kind = match mode.as_str() {
        "120000" => EntryKind::Symlink,
        "160000" => EntryKind::Submodule,
        _ => EntryKind::File,
    };
    // `-` is what `--long` prints for a gitlink, whose size is not a fact
    // this object store holds.
    let size = if size_word == "-" {
        0
    } else {
        size_word.parse().map_err(|_| malformed(record))?
    };
    Ok(TreeEntry {
        path,
        oid,
        mode,
        kind,
        size,
    })
}

/// **Phase two**: acquire and extract a listed tree.
///
/// Takes the [`GitTree`] rather than re-listing, which is what makes the
/// concurrent-HEAD-advance property *checkable*: a caller can list, let the
/// world move, and extract, and the result is still one world. It is also the
/// honest shape — the two phases really are two Git conversations, and
/// pretending otherwise would hide the only window that exists.
pub fn extract_tree(source: &EstateGitSource, tree: &GitTree) -> Result<SourceScan, GitScanError> {
    extract_tree_impl(source, tree, None)
}

/// [`extract_tree`], with Office/ZIP/mail blobs routed through a real
/// supervised worker (S4 Y8) instead of being reported `unsupported` for
/// lack of one — the shape [`super::lane::scan_estate_git_on_lane`] actually
/// drives in production. [`extract_tree`] itself stays worker-free (R1),
/// exactly the reasoning [`super::scan::scan_local_knowledge`]'s own doc
/// gives for its sibling.
pub fn extract_tree_with_worker(
    source: &EstateGitSource,
    tree: &GitTree,
    worker: &WorkerRuntime,
) -> Result<SourceScan, GitScanError> {
    extract_tree_impl(source, tree, Some(worker))
}

fn extract_tree_impl(
    source: &EstateGitSource,
    tree: &GitTree,
    worker: Option<&WorkerRuntime>,
) -> Result<SourceScan, GitScanError> {
    let filter = AcquisitionFilter::new(&source.ignore)?;
    let mut out = Extracted::default();
    let paths: BTreeSet<String> = tree.entries.iter().map(|e| e.path.clone()).collect();
    let denied_dirs = directory_coverage(&paths, &filter, &mut out.coverage);
    extract_blobs(
        &source.mount,
        &filter,
        &tree.entries,
        &denied_dirs,
        worker,
        &mut out,
    )?;
    out.files
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    out.coverage.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(SourceScan {
        source_name: source.name.clone(),
        kind: SourceKind::EstateGit,
        authority: AuthorityClass::EstateMutable,
        content_key: tree.tree_oid.clone(),
        revision: Some(tree.commit_sha.clone()),
        observed_at: rfc3339_utc_now(),
        files: out.files,
        // No dataset, no root, no allowlist: Git objects have no path to read
        // in place, so this walk registers no tabular dataset at all and the
        // F10a allowlist has nothing to gate (see [`DATASET_NO_ROOT`]).
        datasets: Vec::new(),
        root: None,
        context_fields: ContextFields::none(),
        coverage: out.coverage,
        extractors: out.extractors,
    })
}

/// The three parallel outputs every walk accumulates — rows, coverage, and
/// which extractors actually ran.
#[derive(Debug, Default)]
pub(crate) struct Extracted {
    /// Acquired resources.
    pub(crate) files: Vec<ScannedFile>,
    /// One row per path considered.
    pub(crate) coverage: Vec<CoverageRow>,
    /// Distinct extractor identities that ran.
    pub(crate) extractors: BTreeSet<String>,
}

/// Acquire and extract tree entries out of `mount`'s object store, appending
/// to `out`.
///
/// The whole of the estate-git acquisition boundary, shared by
/// [`extract_tree`] and by [`super::overlay`]'s unchanged half so the two can
/// never diverge on what a symlink, a gitlink, an oversized blob or a denied
/// path becomes.
pub(crate) fn extract_blobs(
    mount: &std::path::Path,
    filter: &AcquisitionFilter,
    entries: &[TreeEntry],
    denied_dirs: &[String],
    worker: Option<&WorkerRuntime>,
    out: &mut Extracted,
) -> Result<(), GitScanError> {
    // Decide every path from metadata alone, before one byte is requested.
    // F10's boundary is a predicate over a path, and the resource ceiling is a
    // predicate over a size `ls-tree --long` already told us.
    let mut wanted: Vec<&TreeEntry> = Vec::new();
    for entry in entries {
        if denied_dirs.iter().any(|dir| entry.path.starts_with(dir)) {
            // Its directory was refused once, and carries the one row.
            continue;
        }
        if let Verdict::Denied { pattern } = filter.verdict(&entry.path) {
            out.coverage.push(CoverageRow {
                path: Some(entry.path.clone()),
                status: Coverage::Excluded,
                detail: Some(format!("refused at acquisition by {pattern}")),
                bytes: Some(entry.size),
            });
            continue;
        }
        match entry.kind {
            EntryKind::Submodule => {
                out.coverage.push(CoverageRow {
                    path: Some(entry.path.clone()),
                    status: Coverage::Unsupported,
                    detail: Some(format!(
                        "gitlink to commit {}: another repository's objects are not in this \
                         object store",
                        entry.oid
                    )),
                    bytes: None,
                });
                continue;
            }
            EntryKind::Symlink => {
                // The blob holds a target path, not content. Following it
                // would read whatever the link points at in a *working tree*
                // this module deliberately never touches — see the module doc.
                out.coverage.push(CoverageRow {
                    path: Some(entry.path.clone()),
                    status: Coverage::Unavailable,
                    detail: Some("symlink is not followed".to_string()),
                    bytes: Some(entry.size),
                });
                continue;
            }
            EntryKind::File => {}
        }
        // X4: a dataset in a repository is claimed by the tabular routing
        // table but cannot be read the way that table reads — see
        // [`DATASET_NO_ROOT`]. Reported by that name rather than as
        // "unclaimed", which would be a false statement about the routing.
        if format_for(&entry.path).is_some() {
            out.coverage.push(CoverageRow {
                path: Some(entry.path.clone()),
                status: Coverage::Unsupported,
                detail: Some(DATASET_NO_ROOT.to_string()),
                bytes: Some(entry.size),
            });
            continue;
        }
        // S4 Y8: a path claimed by a supervised-worker adapter is wanted
        // too — it is never valid UTF-8 text, so `claims_for` alone would
        // wrongly report it `UNCLAIMED` rather than routing it to the
        // worker below.
        if claims_for(&entry.path).is_none() && worker_extractor_for(&entry.path).is_none() {
            out.coverage.push(CoverageRow {
                path: Some(entry.path.clone()),
                status: Coverage::Unsupported,
                detail: Some(UNCLAIMED.to_string()),
                bytes: Some(entry.size),
            });
            continue;
        }
        if entry.size > MAX_RESOURCE_BYTES {
            out.coverage.push(CoverageRow {
                path: Some(entry.path.clone()),
                status: Coverage::Unsupported,
                detail: Some(format!(
                    "larger than the {MAX_RESOURCE_BYTES}-byte resource ceiling"
                )),
                bytes: Some(entry.size),
            });
            continue;
        }
        wanted.push(entry);
    }

    for batch in batches(&wanted) {
        let oids: Vec<String> = batch.iter().map(|e| e.oid.clone()).collect();
        let objects = git_cat_file_batch(mount, &oids)?;
        for (entry, object) in batch.iter().zip(objects) {
            let Some(object) = object else {
                out.coverage.push(CoverageRow {
                    path: Some(entry.path.clone()),
                    status: Coverage::Unavailable,
                    detail: Some(format!(
                        "blob {} is not in this object store (a partial clone, or a \
                         corrupt one)",
                        entry.oid
                    )),
                    bytes: Some(entry.size),
                });
                continue;
            };
            if let Some(extractor) = worker_extractor_for(&entry.path) {
                match worker {
                    Some(worker) => dispatch_worker_resource(
                        worker,
                        filter,
                        entry.path.clone(),
                        &entry.oid,
                        KeySpace::EstateGit,
                        object.bytes,
                        extractor,
                        // Not a fact a Git object has — see `extract_blobs`'s
                        // own in-process branch below, same reasoning.
                        None,
                        crate::runtime::atlas::scan::ChildSink {
                            files: &mut out.files,
                            coverage: &mut out.coverage,
                            extractors: &mut out.extractors,
                            // This walk registers no dataset rows at all —
                            // its own loose files claimed by that table get
                            // `DATASET_NO_ROOT` above, so a child of the same
                            // extension gets the same answer (S5 W7
                            // F-SF-01's `DATASET_CHILD_NOT_REGISTERED`).
                            datasets: None,
                            context_fields: &ContextFields::none(),
                        },
                    ),
                    None => out.coverage.push(CoverageRow {
                        path: Some(entry.path.clone()),
                        status: Coverage::Unsupported,
                        detail: Some(format!(
                            "{extractor} claims this resource, but no supervised worker is \
                             configured for this scan"
                        )),
                        bytes: Some(object.bytes.len() as u64),
                    }),
                }
                continue;
            }
            let claims = claims_for(&entry.path).expect("filtered above");
            let Some(text) = as_text(&object.bytes) else {
                out.coverage.push(CoverageRow {
                    path: Some(entry.path.clone()),
                    status: Coverage::Unsupported,
                    detail: Some("not valid UTF-8 text".to_string()),
                    bytes: Some(object.bytes.len() as u64),
                });
                continue;
            };
            // **F7, estate-git half.** Every key below is the blob OID plus an
            // extractor identity. The OID *is* Git's hash of exactly these
            // bytes; hashing them again would produce a second name for one
            // thing and cost a full pass over every byte in the repository to
            // do it. [`KeySpace::EstateGit`] is that rule, passed to the one
            // shared extractor rather than restated here.
            let extracted = extract_resource(claims, text, &entry.oid, KeySpace::EstateGit);
            out.extractors.extend(extracted.identities.iter().cloned());
            out.coverage.push(CoverageRow {
                path: Some(entry.path.clone()),
                status: extracted.status(),
                detail: Some(extracted.detail()),
                bytes: Some(object.bytes.len() as u64),
            });
            out.files.push(ScannedFile {
                relative_path: entry.path.clone(),
                local_key: extracted.key,
                content_hash: entry.oid.clone(),
                extractor: extracted.extractor.to_string(),
                byte_len: object.bytes.len() as u64,
                // Not a fact a Git object has, and not one this path needs:
                // F7 makes mtime a change hint for the *filesystem* scanner,
                // and an object store has content identity instead.
                mtime_millis: None,
                units: extracted.units,
                syntax: extracted.syntax,
                // A Git tree entry is acquired directly, never expanded out
                // of a container (S5 W7).
                parent: None,
            });
        }
    }
    Ok(())
}

/// One coverage row per directory the tree contains, and the prefixes a
/// denied directory takes out of the walk entirely.
///
/// Mirrors [`super::scan`]'s two spellings exactly (see
/// [`AcquisitionFilter::verdict`]'s own doc): `build` refuses the directory
/// once and nothing under it is mentioned again; `build/**` leaves the
/// directory `discovered` and refuses each file with its own row. Same bytes
/// excluded either way, same rows, whichever source kind is being read.
pub(crate) fn directory_coverage(
    paths: &BTreeSet<String>,
    filter: &AcquisitionFilter,
    coverage: &mut Vec<CoverageRow>,
) -> Vec<String> {
    let mut directories = BTreeSet::new();
    for path in paths {
        let mut at = 0;
        while let Some(offset) = path[at..].find('/') {
            at += offset;
            directories.insert(path[..at].to_string());
            at += 1;
        }
    }
    let mut denied: Vec<String> = Vec::new();
    for directory in directories {
        // Shortest first (BTreeSet order puts `a` before `a/b`), so an
        // ancestor's refusal is already recorded when its children are
        // considered.
        if denied.iter().any(|dir| directory.starts_with(dir)) {
            continue;
        }
        match filter.verdict(&directory) {
            Verdict::Denied { pattern } => {
                coverage.push(CoverageRow {
                    path: Some(directory.clone()),
                    status: Coverage::Excluded,
                    detail: Some(format!("refused at acquisition by {pattern}")),
                    bytes: None,
                });
                denied.push(format!("{directory}/"));
            }
            Verdict::Allowed => coverage.push(CoverageRow {
                path: Some(directory),
                status: Coverage::Discovered,
                detail: Some("directory".to_string()),
                bytes: None,
            }),
        }
    }
    denied
}

/// Split the wanted entries into batches bounded by both
/// [`BATCH_OBJECT_LIMIT`] and [`BATCH_BYTE_BUDGET`].
///
/// A single entry always gets a batch of its own rather than being dropped
/// for exceeding the byte budget alone — the per-resource ceiling is the only
/// thing allowed to refuse a file, and it already ran.
pub(crate) fn batches<'a>(wanted: &[&'a TreeEntry]) -> Vec<Vec<&'a TreeEntry>> {
    let mut out: Vec<Vec<&TreeEntry>> = Vec::new();
    let mut current: Vec<&TreeEntry> = Vec::new();
    let mut bytes = 0u64;
    for entry in wanted {
        let over_budget = !current.is_empty()
            && (current.len() >= BATCH_OBJECT_LIMIT
                || bytes.saturating_add(entry.size) > BATCH_BYTE_BUDGET);
        if over_budget {
            out.push(std::mem::take(&mut current));
            bytes = 0;
        }
        bytes = bytes.saturating_add(entry.size);
        current.push(entry);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// List and extract in one call — the ordinary path.
pub fn scan_estate_git(source: &EstateGitSource) -> Result<EstateGitScan, GitScanError> {
    scan_estate_git_impl(source, None)
}

/// [`scan_estate_git`], with Office/ZIP/mail blobs routed through a real
/// supervised worker (S4 Y8) — see [`extract_tree_with_worker`]'s own doc.
pub fn scan_estate_git_with_worker(
    source: &EstateGitSource,
    worker: &WorkerRuntime,
) -> Result<EstateGitScan, GitScanError> {
    scan_estate_git_impl(source, Some(worker))
}

fn scan_estate_git_impl(
    source: &EstateGitSource,
    worker: Option<&WorkerRuntime>,
) -> Result<EstateGitScan, GitScanError> {
    let tree = list_tree(source)?;
    let scan = extract_tree_impl(source, &tree, worker)?;
    let drift = observe_drift(source, &tree);
    Ok(EstateGitScan {
        scan,
        tree_oid: tree.tree_oid,
        drift,
    })
}

/// §11.4's observation for a scan: has the mount's committed HEAD moved off
/// the SHA this scan was pinned to?
///
/// **One `git rev-parse HEAD`**, exactly as [`crate::runtime::integrity`]'s
/// retirement-time observer is bounded to — no status, no worktree walk, no
/// polling. `None` means the mount is still where the scan pinned it. `Some`
/// means it moved, and the scan is still the world it said it was: the
/// observation rides *beside* the rows
/// ([`EstateGitScan::drift`]), never inside them, because a scan that blended
/// the two would be evidence about no world at all.
///
/// A mount whose HEAD cannot be read at all is not drift and is not an error
/// here — it is the absence of an observation, and `Ok(None)` says so.
pub fn observe_drift(source: &EstateGitSource, tree: &GitTree) -> Option<EstateDriftObservation> {
    let head = git(&source.mount, &["rev-parse", "HEAD"]).ok()?;
    if head == tree.commit_sha {
        return None;
    }
    Some(EstateDriftObservation {
        repository: source.name.clone(),
        before: tree.commit_sha.clone(),
        observed: head,
        attribution: DriftAttribution::Unknown,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::source::estate_git_key;
    use crate::runtime::atlas::text::MARKDOWN_EXTRACTOR;
    use crate::runtime::git::git as run_git;

    /// A repository built from `(path, contents)` pairs, committed once.
    fn repo(files: &[(&str, &str)]) -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        run_git(root, &["init", "--initial-branch=main"]).expect("init");
        run_git(root, &["config", "user.email", "t@example.com"]).expect("email");
        run_git(root, &["config", "user.name", "T"]).expect("name");
        let sha = commit(root, files, "one");
        (dir, sha)
    }

    fn commit(root: &std::path::Path, files: &[(&str, &str)], message: &str) -> String {
        for (path, body) in files {
            let full = root.join(path);
            std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
            std::fs::write(&full, body).expect("write");
        }
        run_git(root, &["add", "-A"]).expect("add");
        run_git(root, &["commit", "-m", message]).expect("commit");
        run_git(root, &["rev-parse", "HEAD"]).expect("rev-parse")
    }

    fn source(root: &std::path::Path, sha: &str, ignore: &[&str]) -> EstateGitSource {
        EstateGitSource {
            name: "product".to_string(),
            mount: root.to_path_buf(),
            pinned_sha: sha.to_string(),
            ignore: ignore.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    fn row<'a>(scan: &'a SourceScan, path: &str) -> &'a CoverageRow {
        scan.coverage
            .iter()
            .find(|r| r.path.as_deref() == Some(path))
            .unwrap_or_else(|| panic!("no coverage row for {path:?}"))
    }

    /// F8, on the git path: every path the tree holds leaves exactly one row,
    /// with the status the situation warrants, and the deny set applies at
    /// the acquisition boundary exactly as it does on the filesystem path.
    #[test]
    fn every_tree_path_leaves_exactly_one_coverage_row() {
        let (dir, sha) = repo(&[
            ("README.md", "# Top\n\nbody\n"),
            ("docs/one.md", "# One\n"),
            ("docs/plain.txt", "just text\n"),
            ("src/main.rs", "fn main() {}\n"),
            ("keys/server.pem", "-----BEGIN\n"),
            (".env", "SECRET=1\n"),
        ]);
        let scanned = scan_estate_git(&source(dir.path(), &sha, &[])).expect("scan");
        let scan = &scanned.scan;
        let paths: Vec<&str> = scan
            .coverage
            .iter()
            .filter_map(|r| r.path.as_deref())
            .collect();
        let mut sorted = paths.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(paths.len(), sorted.len(), "a path got two rows: {paths:?}");

        assert_eq!(row(scan, "README.md").status, Coverage::Indexed);
        assert_eq!(row(scan, "docs/one.md").status, Coverage::Indexed);
        assert_eq!(row(scan, "docs/plain.txt").status, Coverage::Indexed);
        // Claimed by a grammar since X3b, so `unsupported` would now be the
        // dishonest answer — the file *is* indexed, and its symbols are rows.
        assert_eq!(row(scan, "src/main.rs").status, Coverage::Indexed);
        assert_eq!(row(scan, "keys/server.pem").status, Coverage::Excluded);
        assert_eq!(row(scan, ".env").status, Coverage::Excluded);
        assert_eq!(row(scan, "docs").status, Coverage::Discovered);
        assert_eq!(scan.kind, SourceKind::EstateGit);
        assert_eq!(scan.authority, AuthorityClass::EstateMutable);
        assert_eq!(scan.revision.as_deref(), Some(sha.as_str()));
        assert!(scanned.drift.is_none(), "the mount never moved");

        let secret = scan
            .files
            .iter()
            .flat_map(|f| f.units.iter().map(|u| u.text.as_str()))
            .any(|t| t.contains("SECRET=1"));
        assert!(!secret, "an excluded blob reached a unit");
    }

    /// F7's estate-git half, where it can actually fail: the stored content
    /// identity **is** the blob OID Git reports, and the key is that OID plus
    /// the extractor. Two identical files share one blob and therefore one
    /// key, by construction and not by a second hash.
    #[test]
    fn keys_are_the_blob_oid_plus_the_extractor_never_a_second_hash() {
        let (dir, sha) = repo(&[("a.md", "# Same\n"), ("copy/b.md", "# Same\n")]);
        let src = source(dir.path(), &sha, &[]);
        let tree = list_tree(&src).expect("list");
        let scan = extract_tree(&src, &tree).expect("extract");
        assert_eq!(scan.files.len(), 2);
        let a = &scan.files[0];
        let b = &scan.files[1];
        assert_eq!(a.content_hash, b.content_hash, "one blob, one oid");
        assert_eq!(a.local_key, b.local_key);

        let oid = run_git(dir.path(), &["rev-parse", &format!("{sha}:a.md")]).expect("rev-parse");
        assert_eq!(a.content_hash, oid, "the content key is Git's own oid");
        assert_eq!(a.local_key, estate_git_key(&oid, MARKDOWN_EXTRACTOR));
        assert_ne!(
            a.local_key,
            crate::domain::source::local_key(&oid, MARKDOWN_EXTRACTOR),
            "the two key spaces are domain-separated"
        );
        assert!(a.mtime_millis.is_none(), "an object store has no mtime");
        assert_eq!(scan.content_key, tree.tree_oid);
    }

    /// A gitlink and a symlink are coverage facts, not errors and not
    /// followed.
    #[test]
    fn a_symlink_and_a_gitlink_are_reported_and_never_followed() {
        let (dir, _) = repo(&[("real.md", "# Real\n")]);
        let root = dir.path();
        std::os::unix::fs::symlink("/etc/passwd", root.join("escape.md")).expect("symlink");
        // A gitlink without a submodule checkout: the index entry is all a
        // tree needs, and it is what a partial clone leaves behind anyway.
        // The commit it names is this repository's own, which is simply a
        // valid id that is not in any *submodule* — exactly the situation
        // that makes a gitlink unreadable here.
        let elsewhere = run_git(root, &["rev-parse", "HEAD"]).expect("head");
        run_git(
            root,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("160000,{elsewhere},vendor/dep"),
            ],
        )
        .expect("update-index");
        run_git(root, &["add", "escape.md"]).expect("add");
        run_git(root, &["commit", "-m", "two"]).expect("commit");
        let sha = run_git(root, &["rev-parse", "HEAD"]).expect("head");

        let scan = scan_estate_git(&source(root, &sha, &[]))
            .expect("scan")
            .scan;
        assert_eq!(row(&scan, "escape.md").status, Coverage::Unavailable);
        assert!(
            row(&scan, "escape.md")
                .detail
                .as_deref()
                .is_some_and(|d| d.contains("symlink"))
        );
        assert_eq!(row(&scan, "vendor/dep").status, Coverage::Unsupported);
        assert!(
            row(&scan, "vendor/dep")
                .detail
                .as_deref()
                .is_some_and(|d| d.contains("gitlink"))
        );
        assert_eq!(scan.files.len(), 1, "only the real file was acquired");
    }

    /// A denied *directory* is refused once and nothing under it is mentioned
    /// again — the same two spellings the filesystem walk has.
    #[test]
    fn a_denied_directory_is_refused_once_and_its_children_are_not_listed() {
        let (dir, sha) = repo(&[
            ("keep.md", "# Keep\n"),
            ("build/out.md", "# Generated\n"),
            ("build/deep/more.md", "# More\n"),
            ("vendor/lib.md", "# Vendored\n"),
        ]);
        let scan = scan_estate_git(&source(dir.path(), &sha, &["build", "vendor/**"]))
            .expect("scan")
            .scan;
        assert_eq!(row(&scan, "build").status, Coverage::Excluded);
        assert!(
            !scan
                .coverage
                .iter()
                .any(|r| r.path.as_deref().is_some_and(|p| p.starts_with("build/"))),
            "a refused directory's children were listed: {:?}",
            scan.coverage
        );
        // The `/**` spelling reports per file and leaves the directory
        // discovered.
        assert_eq!(row(&scan, "vendor").status, Coverage::Discovered);
        assert_eq!(row(&scan, "vendor/lib.md").status, Coverage::Excluded);
        assert_eq!(scan.files.len(), 1);
    }

    /// Batching is a real property, not a comment: many blobs are grouped,
    /// and the grouping never drops or reorders one.
    #[test]
    fn many_blobs_are_read_in_grouped_batches() {
        let bodies: Vec<(String, String)> = (0..40)
            .map(|i| (format!("d{}/f{i}.md", i % 4), format!("# Doc {i}\n")))
            .collect();
        let files: Vec<(&str, &str)> = bodies
            .iter()
            .map(|(p, b)| (p.as_str(), b.as_str()))
            .collect();
        let (dir, sha) = repo(&files);
        let src = source(dir.path(), &sha, &[]);
        let tree = list_tree(&src).expect("list");
        let scan = extract_tree(&src, &tree).expect("extract");
        assert_eq!(scan.files.len(), 40);
        for (path, body) in &bodies {
            let file = scan
                .files
                .iter()
                .find(|f| &f.relative_path == path)
                .unwrap_or_else(|| panic!("missing {path}"));
            assert_eq!(file.units[0].text, *body);
        }

        // The bounds are what group them, and both directions are exercised
        // rather than assumed.
        let wanted: Vec<&TreeEntry> = tree.entries.iter().collect();
        assert_eq!(batches(&wanted).len(), 1, "40 tiny blobs are one batch");
        let huge: Vec<TreeEntry> = (0..3)
            .map(|i| TreeEntry {
                path: format!("big{i}.md"),
                oid: "0".repeat(40),
                mode: "100644".to_string(),
                kind: EntryKind::File,
                size: BATCH_BYTE_BUDGET,
            })
            .collect();
        let refs: Vec<&TreeEntry> = huge.iter().collect();
        assert_eq!(batches(&refs).len(), 3, "the byte budget splits them");
        assert!(
            batches(&refs).iter().all(|b| b.len() == 1),
            "one oversized entry still gets its own batch rather than none"
        );
    }

    /// The scan is a pure function of the pinned commit: the same SHA scanned
    /// twice, with the working tree scribbled on in between, is identical
    /// evidence — the working tree is not an input.
    #[test]
    fn the_working_tree_is_not_an_input() {
        let (dir, sha) = repo(&[("a.md", "# A\n"), ("b.md", "# B\n")]);
        let src = source(dir.path(), &sha, &[]);
        let first = extract_tree(&src, &list_tree(&src).expect("list")).expect("extract");
        std::fs::write(dir.path().join("a.md"), "# TOTALLY DIFFERENT\n").expect("write");
        std::fs::write(dir.path().join("untracked.md"), "# Not committed\n").expect("write");
        std::fs::remove_file(dir.path().join("b.md")).expect("remove");
        let second = extract_tree(&src, &list_tree(&src).expect("list")).expect("extract");
        assert_eq!(first.files, second.files);
        assert_eq!(first.coverage, second.coverage);
        assert_eq!(first.content_key, second.content_key);
        assert!(
            !second
                .coverage
                .iter()
                .any(|r| r.path.as_deref() == Some("untracked.md")),
            "an uncommitted file entered a pinned scan"
        );
    }

    /// A SHA that is not in the object store is a named Git failure, never an
    /// empty scan silently reported as coverage.
    #[test]
    fn an_unknown_revision_fails_rather_than_reporting_an_empty_world() {
        let (dir, _) = repo(&[("a.md", "# A\n")]);
        let err = list_tree(&source(dir.path(), &"f".repeat(40), &[])).expect_err("unknown sha");
        assert!(matches!(err, GitScanError::Git(_)), "{err}");
    }

    /// The record parser reads `--long`'s own shape, including the gitlink's
    /// `-` size and a path containing a space.
    #[test]
    fn tree_records_are_parsed_by_their_declared_shape() {
        let src = EstateGitSource {
            name: "r".to_string(),
            mount: PathBuf::from("/tmp"),
            pinned_sha: "abc".to_string(),
            ignore: Vec::new(),
        };
        let entry =
            parse_tree_record(b"100644 blob aaa      12\tdocs/a file.md", "abc", &src).expect("ok");
        assert_eq!(entry.path, "docs/a file.md");
        assert_eq!(entry.size, 12);
        assert_eq!(entry.kind, EntryKind::File);
        let link = parse_tree_record(b"120000 blob bbb       4\tlink", "abc", &src).expect("ok");
        assert_eq!(link.kind, EntryKind::Symlink);
        let sub =
            parse_tree_record(b"160000 commit ccc       -\tvendor/dep", "abc", &src).expect("ok");
        assert_eq!(sub.kind, EntryKind::Submodule);
        assert_eq!(sub.size, 0);
        let err = parse_tree_record(b"nonsense", "abc", &src).expect_err("malformed");
        assert!(matches!(err, GitScanError::MalformedTree { .. }), "{err}");
    }
}

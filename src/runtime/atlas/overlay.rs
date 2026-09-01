//! Work-overlay hashing: a Work surface's changes, over its base tree (X3a).
//!
//! A Work's mutation surface is a linked worktree cut from a mount at an
//! admission-pinned commit. Almost all of it is that commit — a Work that
//! edits four files leaves every other file byte-identical to the base tree,
//! and re-deriving evidence for all of them would be the whole cost of a
//! repository scan to describe four files.
//!
//! So an overlay generation is exactly that: **the base tree, plus a
//! difference.**
//!
//! ```text
//! unchanged paths  keyed by blob OID + extractor   (F7's estate-git rule)
//! changed paths    keyed by BLAKE3 + extractor     (F7's local rule)
//! the generation   base commit SHA + overlay digest of the changed paths
//! ```
//!
//! Each half uses the key rule that is *true* for it, which is the whole
//! reason F7 has two. An unchanged path's bytes are in the object store and
//! Git has already hashed them; hashing them again would be the second hash
//! F7 forbids. A changed path's bytes are in a working tree and Git has *not*
//! hashed them — there is no OID to reuse, and BLAKE3 over the file is the
//! only content identity available. Nothing here re-hashes anything Git
//! already named, and nothing here pretends an uncommitted file has an OID.
//!
//! # Scoped to one Work, and evicted with it
//!
//! An overlay generation describes a world only one Work can see, and only
//! while that Work exists. Its source name carries the Work id
//! ([`overlay_source_name`]) so the rows are addressable as a set, and
//! [`AtlasDb::evict_work_overlays`](crate::runtime::atlas::db::AtlasDb::evict_work_overlays)
//! removes that set — leaving a `generation_evicted` coverage row per
//! generation, the same reported-never-silent discipline every other eviction
//! follows. A retired Work's overlay evidence does not linger as facts about
//! a surface that no longer exists.
//!
//! # What an overlay is not
//!
//! Not a second opinion about the mount. The base half is read from the
//! object store at the pinned SHA, exactly as [`super::git`] reads it, so an
//! overlay and a plain estate-git scan of the same base agree on every
//! unchanged path by construction rather than by coincidence.
//!
//! # The production trigger, and what "as of" means (S5 W1b)
//!
//! Through S4 this module was correct at the unit level
//! (`tests/x3a_git_plumbing.rs`) and had **no production caller at all** —
//! no HTTP route, no `sgt` verb, no Work-lifecycle hook — which is what
//! §17 item 2's register row recorded as a deviation and what its tripwire
//! `a1a_item_2_gap_work_overlay_scan_has_no_production_trigger` guarded.
//!
//! S5 W1b wired H13.2's chosen mechanism, the **daemon-side hook**
//! (`api::run_work_overlay_hook`); S5 W1d added its middle row:
//!
//! ```text
//! surface bound (materialize / rematerialize)     [W1b]
//!     -> super::lane::scan_work_overlay_on_lane   one intelligence-lane permit (F6)
//!        -> super::record::record_scan            staged, journaled, confirmed
//! a turn ended, surface still bound               [W1d]
//!     -> the same two steps again                 coalescing, per Work
//! surface torn down                               [W1b]
//!     -> AtlasDb::evict_work_overlays             the lifetime rule above, enforced
//! ```
//!
//! Query-time scanning was rejected: a read verb that writes fights the
//! daemon-is-sole-writer boundary, so `sgt search` never reaches this
//! module. It reads what the hook already recorded.
//!
//! W1b's bind-and-teardown pair alone could only ever record an **empty**
//! overlay: a linked worktree is cut byte-identical to its base, so there
//! is nothing to describe until the Work has actually changed something.
//! W1d's turn-boundary refresh is what makes A2 §2's "current Work's world,
//! **including overlay**" true of the code rather than of the machinery
//! alone.
//!
//! An overlay generation is therefore a **snapshot of the surface as of the
//! end of the Work's last completed turn**, not a live view of a surface
//! the Work is still mutating — a turn in flight is a tree being written,
//! and indexing it at an arbitrary instant would describe a world that
//! never settled. The semantic is carried on the answer rather than left to
//! be assumed — see
//! [`crate::runtime::atlas::db::WorkScope::BaseAndOverlaySnapshot`], which
//! is the type `--work` returns it in.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::domain::event::rfc3339_utc_now;
use crate::domain::source::{
    AuthorityClass, Coverage, CoverageRow, SourceKind, content_hash, overlay_digest,
    overlay_generation_key,
};
use crate::runtime::atlas::deny::{AcquisitionFilter, Verdict};
use crate::runtime::atlas::git::{
    EstateGitSource, Extracted, GitScanError, GitTree, TreeEntry, directory_coverage,
    extract_blobs, list_tree,
};
use crate::runtime::atlas::scan::{
    DATASET_NO_ROOT, KeySpace, MAX_RESOURCE_BYTES, ScannedFile, SourceScan, UNCLAIMED, claims_for,
    extract_resource,
};
use crate::runtime::atlas::tabular::{ContextFields, format_for};
use crate::runtime::atlas::text::as_text;
use crate::runtime::git::{GitError, git_bytes};

/// What an overlay digest records for a path the Work **deleted**.
///
/// A deletion has no bytes to hash, and leaving it out of the digest
/// altogether would make "deleted `a.md`" and "changed nothing" the same
/// overlay — which they are not: the first has one fewer indexed file. Hex is
/// what every other value in the map is, so a marker in angle brackets cannot
/// collide with one.
pub const DELETED_MARKER: &str = "<deleted>";

/// What the digest records for a changed path that could not be read at all.
///
/// Deliberately **does not carry the io error's own text**, however useful
/// that text is: the coverage row carries it, and a generation key that
/// embedded it would move whenever the wording of a `strerror` did. A key has
/// to be a function of the world, not of a message about it.
pub const UNREADABLE_MARKER: &str = "<unreadable>";

/// What the digest records for a changed path that is not a regular file (a
/// symlink, a fifo, a directory that replaced a file).
pub const NOT_A_FILE_MARKER: &str = "<not-a-regular-file>";

/// What the digest records for a changed path past the resource ceiling.
///
/// Carries the size, which *is* a fact about the world: a Work that grows an
/// over-ceiling file further has changed something, and the key should move.
pub fn over_ceiling_marker(bytes: u64) -> String {
    format!("<over-ceiling:{bytes}>")
}

/// The prefix an overlay source name starts with. Public because eviction
/// matches on it.
pub const OVERLAY_PREFIX: &str = "work:";

/// One Work surface, to be read as an overlay over its base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkOverlay {
    /// The Work whose surface this is — the scope the generation is evicted
    /// with.
    pub work_id: String,
    /// The repository within that Work's scope.
    pub repository: String,
    /// The linked worktree itself: `<surfaces-dir>/<work-id>/<repo>`.
    pub surface: PathBuf,
    /// §8.2's admission-pinned base commit the surface was cut from.
    pub base_sha: String,
    /// Per-source ignore globs, extending the built-in deny set (F10).
    pub ignore: Vec<String>,
}

/// The source coordinate an overlay generation's rows carry.
///
/// Work id first so the whole of one Work's overlay evidence is a single
/// prefix — which is what makes "evicted with the Work" one filter rather
/// than a join.
pub fn overlay_source_name(work_id: &str, repository: &str) -> String {
    format!("{OVERLAY_PREFIX}{work_id}/{repository}")
}

/// The prefix every one of `work_id`'s overlay sources begins with.
pub fn overlay_source_prefix(work_id: &str) -> String {
    format!("{OVERLAY_PREFIX}{work_id}/")
}

/// One completed overlay scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayScan {
    /// The rows, in the shape every `source.*` writer already takes.
    pub scan: SourceScan,
    /// The base commit the overlay stands on.
    pub base_sha: String,
    /// The digest of what the surface changed — the other half of
    /// [`SourceScan::content_key`].
    pub overlay_digest: String,
    /// Paths the surface changed, added or deleted relative to the base.
    pub changed: BTreeSet<String>,
}

/// Scan one Work surface as an overlay over its admission-pinned base.
///
/// Three Git conversations and no more: list the base tree, ask what the
/// surface changed, and batch-read the unchanged blobs. Changed paths are read
/// from the surface's working tree — which is the one place in Atlas that
/// legitimately reads a working tree, because uncommitted work has nowhere
/// else to live.
pub fn scan_work_overlay(overlay: &WorkOverlay) -> Result<OverlayScan, GitScanError> {
    let base_source = EstateGitSource {
        name: overlay.repository.clone(),
        // A linked worktree shares its mount's object store, so the base
        // commit resolves here without going near the mount — and without any
        // chance of reading a mount whose HEAD has since moved.
        mount: overlay.surface.clone(),
        pinned_sha: overlay.base_sha.clone(),
        ignore: overlay.ignore.clone(),
    };
    let base = list_tree(&base_source)?;
    let changed = changed_paths(&overlay.surface, &base.commit_sha)?;
    extract_overlay(overlay, &base, &changed)
}

/// The extraction half, over an already-listed base and an already-computed
/// change set.
///
/// Separate from [`scan_work_overlay`] for the same reason
/// [`super::git::extract_tree`] is separate from `list_tree`: it makes the
/// world the scan is of an *input*, so a test can hold one fixed while
/// something else moves.
pub fn extract_overlay(
    overlay: &WorkOverlay,
    base: &GitTree,
    changed: &BTreeSet<String>,
) -> Result<OverlayScan, GitScanError> {
    let filter = AcquisitionFilter::new(&overlay.ignore)?;
    let mut out = Extracted::default();

    // The path universe is the base tree plus whatever the surface added, so
    // a directory that exists only because of this Work still gets its row.
    let mut universe: BTreeSet<String> = base.entries.iter().map(|e| e.path.clone()).collect();
    universe.extend(changed.iter().cloned());
    let denied_dirs = directory_coverage(&universe, &filter, &mut out.coverage);

    let unchanged: Vec<TreeEntry> = base
        .entries
        .iter()
        .filter(|entry| !changed.contains(&entry.path))
        .cloned()
        .collect();
    extract_blobs(
        &overlay.surface,
        &filter,
        &unchanged,
        &denied_dirs,
        // Out of this wave's scope (brief-y8-adapter-dispatch.md names only
        // `scan.rs`'s and `git.rs`'s own walks): a Work overlay's unchanged
        // half stays worker-free, matching `extract_tree`'s own no-worker
        // default rather than gaining one this wave never asked for.
        None,
        &mut out,
    )?;

    let mut digest_input: BTreeMap<String, String> = BTreeMap::new();
    for path in changed {
        if denied_dirs.iter().any(|dir| path.starts_with(dir)) {
            continue;
        }
        if let Verdict::Denied { pattern } = filter.verdict(path) {
            out.coverage.push(CoverageRow {
                path: Some(path.clone()),
                status: Coverage::Excluded,
                detail: Some(format!("refused at acquisition by {pattern}")),
                bytes: std::fs::symlink_metadata(overlay.surface.join(path))
                    .ok()
                    .filter(|m| m.is_file())
                    .map(|m| m.len()),
            });
            continue;
        }
        digest_input.insert(path.clone(), changed_file(overlay, path, &mut out));
    }

    out.files
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    out.coverage.sort_by(|a, b| a.path.cmp(&b.path));
    let digest = overlay_digest(&digest_input);
    Ok(OverlayScan {
        scan: SourceScan {
            source_name: overlay_source_name(&overlay.work_id, &overlay.repository),
            kind: SourceKind::EstateGit,
            authority: AuthorityClass::EstateMutable,
            content_key: overlay_generation_key(&base.commit_sha, &digest),
            // The commit the overlay stands *on*. Not the world's whole
            // identity — that is `content_key`, which composes this with the
            // digest — but the provenance question "cut from what?" has this
            // one answer and a reader needs it.
            revision: Some(base.commit_sha.clone()),
            observed_at: rfc3339_utc_now(),
            files: out.files,
            // As in [`super::git`]: an overlay's unchanged bytes come from the
            // base tree's objects, so this walk registers no dataset and has
            // no allowlist to gate (see `scan::DATASET_NO_ROOT`).
            datasets: Vec::new(),
            root: None,
            // Deliberately `None`, not `overlay.surface`: a Work overlay is
            // its own isolated world by generation scope
            // (`another_works_overlay_unit_never_surfaces_through_a_lexical_query`),
            // not by identity, and this wave's named scenario is a
            // `[[repo]]`/`[[knowledge]]` overlap — an overlay merging
            // identity with its own base generation is a different question
            // this wave does not decide. `None` preserves the exact pre-S6
            // identity (`relative_path`), so overlay isolation is unaffected.
            identity_root: None,
            context_fields: ContextFields::none(),
            coverage: out.coverage,
            extractors: out.extractors,
        },
        base_sha: base.commit_sha.clone(),
        overlay_digest: digest,
        changed: changed.clone(),
    })
}

/// Acquire one changed path from the surface's working tree, appending its
/// rows to `out`, and answer with what the overlay digest records for it.
///
/// Every refusal still returns a digest value, because the digest has to
/// distinguish "this Work deleted it", "this Work made it unreadable" and
/// "this Work did nothing to it" — three different worlds that would otherwise
/// share one key.
fn changed_file(overlay: &WorkOverlay, path: &str, out: &mut Extracted) -> String {
    let full = overlay.surface.join(path);
    let meta = match std::fs::symlink_metadata(&full) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            out.coverage.push(CoverageRow {
                path: Some(path.to_string()),
                status: Coverage::Unavailable,
                detail: Some("deleted on this Work's surface".to_string()),
                bytes: None,
            });
            return DELETED_MARKER.to_string();
        }
        Err(e) => {
            out.coverage.push(CoverageRow {
                path: Some(path.to_string()),
                status: Coverage::Unavailable,
                detail: Some(format!("cannot be inspected: {e}")),
                bytes: None,
            });
            return UNREADABLE_MARKER.to_string();
        }
    };
    if meta.file_type().is_symlink() || !meta.is_file() {
        out.coverage.push(CoverageRow {
            path: Some(path.to_string()),
            status: if meta.file_type().is_symlink() {
                Coverage::Unavailable
            } else {
                Coverage::Unsupported
            },
            detail: Some(if meta.file_type().is_symlink() {
                "symlink is not followed".to_string()
            } else {
                "not a regular file".to_string()
            }),
            bytes: None,
        });
        return NOT_A_FILE_MARKER.to_string();
    }
    let Some(claims) = (if format_for(path).is_some() {
        // X4: claimed by the tabular table, unreadable by this walk — see
        // `scan::DATASET_NO_ROOT`. An overlay's world is a base tree plus a
        // surface's edits, and neither half is a path a reader may open as
        // the estate's own evidence.
        None
    } else {
        claims_for(path)
    }) else {
        out.coverage.push(CoverageRow {
            path: Some(path.to_string()),
            status: Coverage::Unsupported,
            detail: Some(if format_for(path).is_some() {
                DATASET_NO_ROOT.to_string()
            } else {
                UNCLAIMED.to_string()
            }),
            bytes: Some(meta.len()),
        });
        // Still hashed: an unextractable file whose bytes changed changed the
        // world, and the generation key has to move even though no unit does.
        return std::fs::read(&full)
            .map_or_else(|_| UNREADABLE_MARKER.to_string(), |b| content_hash(&b));
    };
    if meta.len() > MAX_RESOURCE_BYTES {
        out.coverage.push(CoverageRow {
            path: Some(path.to_string()),
            status: Coverage::Unsupported,
            detail: Some(format!(
                "larger than the {MAX_RESOURCE_BYTES}-byte resource ceiling"
            )),
            bytes: Some(meta.len()),
        });
        return over_ceiling_marker(meta.len());
    }
    let bytes = match std::fs::read(&full) {
        Ok(bytes) => bytes,
        Err(e) => {
            out.coverage.push(CoverageRow {
                path: Some(path.to_string()),
                status: Coverage::Unavailable,
                detail: Some(format!("cannot be read: {e}")),
                bytes: Some(meta.len()),
            });
            return UNREADABLE_MARKER.to_string();
        }
    };
    // **F7's local rule**, and the only place in the overlay it applies:
    // these bytes are not in any object store, so no OID names them.
    let hash = content_hash(&bytes);
    let Some(text) = as_text(&bytes) else {
        out.coverage.push(CoverageRow {
            path: Some(path.to_string()),
            status: Coverage::Unsupported,
            detail: Some("not valid UTF-8 text".to_string()),
            bytes: Some(bytes.len() as u64),
        });
        return hash;
    };
    let extracted = extract_resource(claims, text, &hash, KeySpace::Local);
    out.extractors.extend(extracted.identities.iter().cloned());
    out.coverage.push(CoverageRow {
        path: Some(path.to_string()),
        status: extracted.status(),
        detail: Some(extracted.detail()),
        bytes: Some(bytes.len() as u64),
    });
    out.files.push(ScannedFile {
        relative_path: path.to_string(),
        local_key: extracted.key,
        // `hash` is already `content_hash(&bytes)` — plain BLAKE3, same as
        // the local rule two lines up already states.
        content_digest: hash.clone(),
        content_hash: hash.clone(),
        extractor: extracted.extractor.to_string(),
        byte_len: bytes.len() as u64,
        mtime_millis: None,
        units: extracted.units,
        syntax: extracted.syntax,
        // An overlay path is acquired directly, never expanded out of a
        // container (S5 W7).
        parent: None,
    });
    hash
}

/// Every path the surface changed relative to `base_sha`: tracked differences
/// and untracked additions, together.
///
/// Two reads, because Git answers them separately and neither subsumes the
/// other. `diff --name-only <sha>` covers modifications, staged or not, and
/// deletions; `ls-files --others --exclude-standard` covers files Git has
/// never been told about, which the diff cannot see. `-z` on both, so a path
/// containing a newline or a quote arrives verbatim.
///
/// **`--no-renames`, and not as a stylistic preference.** Git's default
/// rename detection (`diff.renames`, true since 2.9, and configurable to
/// `copies`) collapses `git mv a.md z.md` into one record naming only `z.md`.
/// That is the right answer for a human reading a patch and the wrong one
/// here twice over: the old path never enters the change set, so
/// [`extract_overlay`] reads it out of the *base tree* and reports a file the
/// Work deleted as `Indexed` — a pinned base blended with a false view of the
/// working tree; and the digest omits the deletion, so "renamed `a.md` to
/// `z.md`" and "kept `a.md`, added `z.md`" — two different worlds, one with a
/// file the other has — produce the same generation key. A change set is a
/// set of paths, not a story about them, so the similarity heuristic is
/// turned off rather than interpreted. Passing the flag explicitly also makes
/// the answer independent of whatever `diff.renames` the surface's config
/// inherited.
pub fn changed_paths(surface: &Path, base_sha: &str) -> Result<BTreeSet<String>, GitError> {
    let mut out = BTreeSet::new();
    for args in [
        vec!["diff", "--name-only", "--no-renames", "-z", base_sha, "--"],
        vec!["ls-files", "--others", "--exclude-standard", "-z"],
    ] {
        let raw = git_bytes(surface, &args)?;
        for record in raw.split(|b| *b == 0) {
            if record.is_empty() {
                continue;
            }
            out.insert(String::from_utf8_lossy(record).into_owned());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::source::{estate_git_key, local_key};
    use crate::runtime::atlas::text::MARKDOWN_EXTRACTOR;
    use crate::runtime::git::git as run_git;

    /// A mount with one commit, plus a linked worktree cut from it — the real
    /// shape a Work surface has.
    fn mount_and_surface(files: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf, PathBuf, String) {
        let dir = tempfile::tempdir().expect("tempdir");
        let mount = dir.path().join("mount");
        std::fs::create_dir_all(&mount).expect("mkdir");
        run_git(&mount, &["init", "--initial-branch=main"]).expect("init");
        run_git(&mount, &["config", "user.email", "t@example.com"]).expect("email");
        run_git(&mount, &["config", "user.name", "T"]).expect("name");
        for (path, body) in files {
            let full = mount.join(path);
            std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
            std::fs::write(&full, body).expect("write");
        }
        run_git(&mount, &["add", "-A"]).expect("add");
        run_git(&mount, &["commit", "-m", "base"]).expect("commit");
        let base = run_git(&mount, &["rev-parse", "HEAD"]).expect("head");
        let surface = dir.path().join("surface");
        run_git(
            &mount,
            &[
                "worktree",
                "add",
                "-b",
                "sergeant/01WORK",
                surface.to_str().expect("utf8"),
                &base,
            ],
        )
        .expect("worktree add");
        (dir, mount, surface, base)
    }

    fn overlay(surface: &Path, base: &str) -> WorkOverlay {
        WorkOverlay {
            work_id: "01WORK".to_string(),
            repository: "product".to_string(),
            surface: surface.to_path_buf(),
            base_sha: base.to_string(),
            ignore: Vec::new(),
        }
    }

    fn file<'a>(scan: &'a SourceScan, path: &str) -> &'a ScannedFile {
        scan.files
            .iter()
            .find(|f| f.relative_path == path)
            .unwrap_or_else(|| panic!("no file row for {path:?}"))
    }

    fn row<'a>(scan: &'a SourceScan, path: &str) -> &'a CoverageRow {
        scan.coverage
            .iter()
            .find(|r| r.path.as_deref() == Some(path))
            .unwrap_or_else(|| panic!("no coverage row for {path:?}"))
    }

    /// The whole rule, in one test: changed paths are BLAKE3-keyed, unchanged
    /// paths keep their blob-OID keys, and the generation key composes the
    /// base SHA with the overlay digest.
    #[test]
    fn changed_paths_are_blake3_keyed_and_unchanged_paths_keep_their_oids() {
        let (_dir, mount, surface, base) =
            mount_and_surface(&[("a.md", "# A\n"), ("b.md", "# B\n"), ("docs/c.md", "# C\n")]);
        std::fs::write(surface.join("a.md"), "# A, edited\n").expect("write");
        std::fs::write(surface.join("new.md"), "# New\n").expect("write");

        let scanned = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");
        let scan = &scanned.scan;
        assert_eq!(
            scanned.changed,
            BTreeSet::from(["a.md".to_string(), "new.md".to_string()])
        );

        // Unchanged: the key is Git's own OID plus the extractor, and it is
        // exactly what a plain estate-git scan of the base produces.
        let b_oid = run_git(&mount, &["rev-parse", &format!("{base}:b.md")]).expect("oid");
        assert_eq!(file(scan, "b.md").content_hash, b_oid);
        assert_eq!(
            file(scan, "b.md").local_key,
            estate_git_key(&b_oid, MARKDOWN_EXTRACTOR)
        );
        assert_eq!(file(scan, "docs/c.md").content_hash.len(), b_oid.len());

        // Changed and added: BLAKE3 of the working-tree bytes, and never an
        // OID — there is no object for bytes Git has not been given.
        let edited = content_hash(b"# A, edited\n");
        assert_eq!(file(scan, "a.md").content_hash, edited);
        assert_eq!(
            file(scan, "a.md").local_key,
            local_key(&edited, MARKDOWN_EXTRACTOR)
        );
        let added = content_hash(b"# New\n");
        assert_eq!(file(scan, "new.md").content_hash, added);
        assert_eq!(file(scan, "a.md").units[0].text, "# A, edited\n");

        // The generation key is the composition, and it is reproducible from
        // its two declared halves rather than from anything opaque.
        let expected = overlay_generation_key(
            &base,
            &overlay_digest(&BTreeMap::from([
                ("a.md".to_string(), edited),
                ("new.md".to_string(), added),
            ])),
        );
        assert_eq!(scan.content_key, expected);
        assert_eq!(scanned.base_sha, base);
        assert_eq!(scan.revision.as_deref(), Some(base.as_str()));
    }

    /// The base half and a plain estate-git scan agree on every unchanged
    /// path — by construction, since both go through the same extractor.
    #[test]
    fn the_unchanged_half_agrees_with_a_plain_scan_of_the_base() {
        let (_dir, mount, surface, base) =
            mount_and_surface(&[("a.md", "# A\n"), ("b.md", "# B\n")]);
        std::fs::write(surface.join("a.md"), "# A, edited\n").expect("write");
        let overlaid = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");
        let plain = crate::runtime::atlas::git::scan_estate_git(&EstateGitSource {
            name: "product".to_string(),
            mount,
            pinned_sha: base.clone(),
            ignore: Vec::new(),
        })
        .expect("scan");
        assert_eq!(
            file(&overlaid.scan, "b.md").local_key,
            file(&plain.scan, "b.md").local_key
        );
        assert_ne!(
            file(&overlaid.scan, "a.md").local_key,
            file(&plain.scan, "a.md").local_key,
            "the edited file must not reuse the base extraction"
        );
    }

    /// A deletion changes the world and must move the key, even though it
    /// produces no unit — the case a "hash what is there" digest gets wrong.
    #[test]
    fn a_deletion_is_reported_and_moves_the_generation_key() {
        let (_dir, _mount, surface, base) =
            mount_and_surface(&[("a.md", "# A\n"), ("b.md", "# B\n")]);
        let untouched = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");
        assert_eq!(untouched.scan.files.len(), 2);
        assert!(untouched.changed.is_empty());

        std::fs::remove_file(surface.join("b.md")).expect("remove");
        let deleted = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");
        assert_eq!(
            deleted.scan.files.len(),
            1,
            "the deleted file is not indexed"
        );
        assert_eq!(row(&deleted.scan, "b.md").status, Coverage::Unavailable);
        assert!(
            row(&deleted.scan, "b.md")
                .detail
                .as_deref()
                .is_some_and(|d| d.contains("deleted on this Work's surface"))
        );
        assert_ne!(
            deleted.scan.content_key, untouched.scan.content_key,
            "a deletion changed the world and must change the key"
        );
        assert_eq!(
            deleted.overlay_digest,
            overlay_digest(&BTreeMap::from([(
                "b.md".to_string(),
                DELETED_MARKER.to_string()
            )]))
        );
    }

    /// Two Works cut from the same base with the same edit still get separate,
    /// separately-evictable generations — and one that edits something else
    /// gets a different key.
    #[test]
    fn overlays_are_scoped_to_their_work_and_keyed_by_what_they_changed() {
        let (_dir, _mount, surface, base) =
            mount_and_surface(&[("a.md", "# A\n"), ("b.md", "# B\n")]);
        std::fs::write(surface.join("a.md"), "# Edited\n").expect("write");
        let one = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");

        let mut other = overlay(&surface, &base);
        other.work_id = "01OTHER".to_string();
        let two = scan_work_overlay(&other).expect("overlay");

        assert_eq!(one.scan.source_name, "work:01WORK/product");
        assert_eq!(two.scan.source_name, "work:01OTHER/product");
        assert!(
            one.scan
                .source_name
                .starts_with(&overlay_source_prefix("01WORK"))
        );
        assert!(
            !one.scan
                .source_name
                .starts_with(&overlay_source_prefix("01OTHER"))
        );
        // Same base, same change: the same world, and therefore the same
        // content key even though the rows are scoped separately.
        assert_eq!(one.scan.content_key, two.scan.content_key);

        std::fs::write(surface.join("b.md"), "# Also edited\n").expect("write");
        let three = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");
        assert_ne!(three.scan.content_key, one.scan.content_key);
    }

    /// F10 holds on the overlay path too: a secret a Work *creates* is
    /// refused at the boundary and never reaches a unit.
    #[test]
    fn a_secret_created_on_the_surface_is_refused_at_the_boundary() {
        let (_dir, _mount, surface, base) = mount_and_surface(&[("a.md", "# A\n")]);
        std::fs::write(surface.join(".env"), "SECRET=hunter2\n").expect("write");
        std::fs::create_dir_all(surface.join("keys")).expect("mkdir");
        std::fs::write(surface.join("keys/server.pem"), "-----BEGIN\n").expect("write");
        let scanned = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");
        assert_eq!(row(&scanned.scan, ".env").status, Coverage::Excluded);
        assert_eq!(
            row(&scanned.scan, "keys/server.pem").status,
            Coverage::Excluded
        );
        let text: String = scanned
            .scan
            .files
            .iter()
            .flat_map(|f| f.units.iter().map(|u| u.text.clone()))
            .collect();
        assert!(!text.contains("hunter2"), "a secret reached a unit");
    }

    /// The change set is Git's own answer, and it sees both halves: a tracked
    /// modification and a file Git has never been told about.
    #[test]
    fn the_change_set_covers_tracked_edits_and_untracked_additions() {
        let (_dir, _mount, surface, base) = mount_and_surface(&[("a.md", "# A\n")]);
        assert!(changed_paths(&surface, &base).expect("clean").is_empty());
        std::fs::write(surface.join("a.md"), "# Edited\n").expect("write");
        std::fs::write(surface.join("untracked.md"), "# New\n").expect("write");
        let changed = changed_paths(&surface, &base).expect("changed");
        assert_eq!(
            changed,
            BTreeSet::from(["a.md".to_string(), "untracked.md".to_string()])
        );
        // Staging must not change the answer: a staged edit is still an edit.
        run_git(&surface, &["add", "untracked.md"]).expect("add");
        assert_eq!(changed_paths(&surface, &base).expect("staged"), changed);
    }

    /// A staged rename is a deletion *and* an addition, and the overlay must
    /// see both halves.
    ///
    /// Git's own default answer here is one record naming only the new path.
    /// Believing it would leave the old path outside the change set, where
    /// [`extract_overlay`] extracts it from the base tree and reports a file
    /// that is not on the surface as `Indexed` — the pinned base blended with
    /// a false view of the working tree.
    #[test]
    fn a_staged_rename_reports_the_old_path_as_deleted() {
        let (_dir, _mount, surface, base) =
            mount_and_surface(&[("a.md", "# A\n"), ("b.md", "# B\n")]);
        run_git(&surface, &["mv", "a.md", "z.md"]).expect("mv");

        let changed = changed_paths(&surface, &base).expect("changed");
        assert!(
            changed.contains("a.md"),
            "the vacated path is a change: {changed:?}"
        );
        assert!(changed.contains("z.md"), "so is the new one: {changed:?}");

        let scanned = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");
        assert_eq!(row(&scanned.scan, "a.md").status, Coverage::Unavailable);
        assert!(
            row(&scanned.scan, "a.md")
                .detail
                .as_deref()
                .is_some_and(|d| d.contains("deleted on this Work's surface"))
        );
        assert!(
            !scanned.scan.files.iter().any(|f| f.relative_path == "a.md"),
            "a path the Work vacated must not be extracted from the base tree"
        );
        assert_eq!(
            file(&scanned.scan, "z.md").content_hash,
            content_hash(b"# A\n")
        );
        assert_eq!(
            scanned.overlay_digest,
            overlay_digest(&BTreeMap::from([
                ("a.md".to_string(), DELETED_MARKER.to_string()),
                ("z.md".to_string(), content_hash(b"# A\n")),
            ]))
        );
    }

    /// The collision the same bug produces in the key: a rename and a copy
    /// are different worlds — one still has the original file, the other does
    /// not — and a change set that drops the vacated path cannot tell them
    /// apart.
    #[test]
    fn a_rename_and_a_copy_do_not_share_a_generation_key() {
        // One surface, walked from the copy world into the rename world, so
        // the base commit is the same SHA by construction and the only thing
        // that moves between the two keys is the vacated path.
        let (_dir, _mount, surface, base) = mount_and_surface(&[("a.md", "# A\n")]);
        std::fs::write(surface.join("z.md"), "# A\n").expect("write");
        run_git(&surface, &["add", "-A"]).expect("add");
        let copied = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");

        std::fs::remove_file(surface.join("a.md")).expect("remove");
        run_git(&surface, &["add", "-A"]).expect("add");
        let renamed = scan_work_overlay(&overlay(&surface, &base)).expect("overlay");

        assert_ne!(
            renamed.scan.content_key, copied.scan.content_key,
            "a world missing a.md and a world keeping it are not one generation"
        );
        assert_eq!(copied.scan.files.len(), 2);
        assert_eq!(renamed.scan.files.len(), 1);
    }
}

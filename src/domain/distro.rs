//! The distro — `AGENTS.md`, `skills/`, `.sergeant/common/contexts/`, and
//! `.sergeant/workflows/` — embedded in the `sgt` binary at compile time and
//! written into a fresh estate by `sgt init` (issue #165, ADR 0014 decision
//! 1: "ships embedded in the `sgt` binary and is written to disk by `sgt
//! init`. It is not a cloned directory.").
//!
//! Ponytail R5: no dependency already in `Cargo.lock` does whole-directory
//! compile-time embedding — `include_str!` (stdlib, R3) is already used for
//! the built-in `software-change` workflow
//! (`domain::workflow::EMBEDDED_WORKFLOW_TOML`/`EMBEDDED_CONTEXTS`), but that
//! precedent hand-lists five known literal paths; it does not generalize to
//! 200+ files across an arbitrarily nested tree without either hand-listing
//! every path (silently stale the moment a file is added and the list
//! isn't) or a directory-walking build script (more code than the dependency
//! it would replace, and re-solves a problem `include_dir` already solves
//! narrowly). `include_dir` was chosen over `rust-embed`: `rust-embed`
//! re-reads files from disk at runtime in debug builds (a `Cow`-based API
//! and a `cfg`-branched code path this use case has no reason to carry —
//! the estate the binary writes into is never the same tree the binary was
//! built from), where `include_dir` gives exactly the shape needed —
//! a `Dir<'static>` tree of `&'static [u8]` file contents, nothing else —
//! for a two-crate addition (`include_dir` + its proc-macro half
//! `include_dir_macros`, confirmed via `cargo add --dry-run` and `cargo
//! fetch`; no transitive crates beyond those two). Measured cost of this
//! addition, recorded in the commit message: release binary size delta and
//! `cargo build --release` wall time, before vs. after.
//!
//! ## Atomic ownership (issue #241, split-hardening W3)
//!
//! `sgt init` used to write per-file and never overwrite or remove anything
//! — a stale copy of a retired package lingered on every estate forever
//! (#241: "18 retired packages lingered" after a distro rebuild). This
//! module now treats three trees as **entirely sgt-owned**: `skills/`,
//! `.sergeant/common/contexts/`, and `.sergeant/workflows/`. After
//! [`write_distro`] runs, each of those trees exactly matches the running
//! binary's embedded content — a file whose content differs from the embed
//! is overwritten, and a file present on disk but absent from the current
//! embed is removed (retired-package cleanup). `.sergeant/local/` is never
//! touched by any of this — see the safety note on [`remove_retired_files`].
//!
//! `AGENTS.md` is a partial exception (owned only inside a marker-delimited
//! managed section, so an estate can append its own content around it — see
//! [`write_agents_md`]), and `.sergeant/index.md` is generated fresh on
//! every init from the workflow packages actually written, rather than
//! embedded as a fifth static copy that could itself drift (#261).

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};

use crate::domain::workflow::{ROOT_CATALOG_FILE, WORKFLOW_ROOT, read_index_front_matter};
use crate::runtime::fsutil::{create_dir_all_durable, write_atomic};

static AGENTS_MD: &str = include_str!("../../AGENTS.md");
static SKILLS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/skills");
static CONTEXTS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/.sergeant/common/contexts");
static WORKFLOWS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/.sergeant/workflows");

/// The comment markers delimiting `AGENTS.md`'s sgt-owned managed section.
/// See [`write_agents_md`].
const MANAGED_BEGIN: &str = "<!-- sgt:managed:begin -->";
const MANAGED_END: &str = "<!-- sgt:managed:end -->";

/// What went wrong writing the distro. Distinct from a bare [`io::Error`]
/// because the managed-`AGENTS.md` case (below) is not an I/O fault at all
/// — it is a deliberate refusal to guess at merging content, and callers
/// (`sgt init`'s own report, doctor) want to say so by name rather than
/// print a generic I/O message.
#[derive(Debug, thiserror::Error)]
pub enum DistroError {
    /// `AGENTS.md` exists, is not empty, and contains neither
    /// `sgt:managed:begin` nor `sgt:managed:end` — most likely a
    /// pre-existing, hand-authored constitution this estate wrote before
    /// the managed-section convention existed. Overwriting it would
    /// silently discard custom content; guessing where to insert markers
    /// would silently corrupt it. Fail closed and name the exact remedy.
    #[error(
        "AGENTS.md exists without sgt:managed markers — insert `{MANAGED_BEGIN}` / \
         `{MANAGED_END}` around the section sgt should own, or remove the file to let \
         `sgt init` write it fresh"
    )]
    UnmanagedAgentsMd,
    /// Any other failure is a plain I/O fault.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Which paths this call actually wrote, left untouched, or removed.
///
/// `changed()` — `written` or `removed` nonempty — is the no-op signal the
/// `AGENTS.md` guardrail requires ("re-running `sgt init` on an
/// already-initialized estate is a no-op, not a reset"): with the same
/// binary and no on-disk drift, a second `sgt init` writes and removes
/// nothing, because every owned file already matches the embed byte for
/// byte.
#[derive(Debug, Default, Clone)]
pub struct DistroWriteOutcome {
    /// Paths created or overwritten by this call, relative to the estate
    /// root.
    pub written: Vec<PathBuf>,
    /// Paths that already matched the embed and were left alone, relative
    /// to the estate root.
    pub skipped: Vec<PathBuf>,
    /// Paths removed by this call because they exist on disk inside a
    /// sgt-owned tree but are not part of the current embed
    /// (retired-package cleanup, #241), relative to the estate root.
    pub removed: Vec<PathBuf>,
    /// Subset of `written`: an owned-tree file whose on-disk content, before
    /// this call overwrote it, matched neither the current embed nor the
    /// content the [distro manifest](DISTRO_MANIFEST_PATH) recorded `sgt
    /// init` itself last writing there — i.e. something other than `sgt
    /// init` (a hand edit) touched it since the last init (#241/#261 F6). A
    /// file that merely differs from the current embed because it still
    /// holds a *previous* release's embed (ordinary version drift) is not
    /// included here; only [`written`](Self::written) is.
    pub overwritten_modified: Vec<PathBuf>,
    /// Set when creating the `CLAUDE.md -> AGENTS.md` symlink failed and
    /// was swallowed (#241/#261 F-INV-06) — the underlying `io::Error`'s
    /// message, so a caller (`sgt init`'s own report, doctor) can surface
    /// *why* `CLAUDE.md` is absent instead of the failure disappearing
    /// silently. `None` means the symlink was created, already existed, or
    /// was never attempted.
    pub symlink_unavailable: Option<String>,
}

impl DistroWriteOutcome {
    /// Whether this call changed anything at all.
    pub fn changed(&self) -> bool {
        !self.written.is_empty() || !self.removed.is_empty()
    }
}

/// Write the embedded distro into `estate_root`.
///
/// Writes only under `estate_root` — `AGENTS.md`, `CLAUDE.md`, `skills/`,
/// `.sergeant/common/contexts/`, `.sergeant/workflows/`, and
/// `.sergeant/index.md` — and never touches `.sergeant/local/`, which is the
/// user's own and which Phase 3's local-shadows-stock resolution
/// (`domain::workflow::WorkflowDefinition::resolve`) depends on staying
/// separate from stock.
///
/// `index.md` and `SKILL.md` front matter's `edition` field is rewritten to
/// this binary's own version (`env!("CARGO_PKG_VERSION")`) as each file is
/// written, rather than trusted verbatim from the embedded content — the
/// mechanism ADR 0016 requires ("`sgt init`/update sets `edition` to the
/// current distro version whenever it writes a stock copy") without relying
/// on every future release remembering to hand-bump the checked-in
/// front matter to match `Cargo.toml` before cutting the release.
pub fn write_distro(estate_root: &Path) -> Result<DistroWriteOutcome, DistroError> {
    let mut outcome = DistroWriteOutcome::default();

    write_agents_md(estate_root, &mut outcome)?;
    symlink_claude_md(estate_root, &mut outcome)?;

    // Loaded once up front, before anything below overwrites the files it
    // describes — this is "what `sgt init` itself last wrote" for every
    // owned file, the baseline `write_owned_file` needs to tell a routine
    // version-drift overwrite apart from a hand edit (#241/#261 F6). `new`
    // is built up as the current embed is written and persisted at the end,
    // becoming the baseline the *next* `sgt init` reads.
    let prior_manifest = load_distro_manifest(estate_root);
    let mut new_manifest = HashMap::new();

    sync_owned_dir(
        estate_root,
        Path::new("skills"),
        &SKILLS,
        &prior_manifest,
        &mut new_manifest,
        &mut outcome,
    )?;
    sync_owned_dir(
        estate_root,
        Path::new(".sergeant/common/contexts"),
        &CONTEXTS,
        &prior_manifest,
        &mut new_manifest,
        &mut outcome,
    )?;
    sync_owned_dir(
        estate_root,
        Path::new(".sergeant/workflows"),
        &WORKFLOWS,
        &prior_manifest,
        &mut new_manifest,
        &mut outcome,
    )?;

    // Generated after the workflows tree above is itself in its final,
    // post-sync state, so reading packages back off disk here sees exactly
    // the current embed — the same "read back what was actually written"
    // idiom `write_distro`'s edition rewrite already uses, rather than a
    // second, independently-walked copy of the same package list that
    // could drift from the first.
    write_index_md(estate_root, &mut outcome)?;

    save_distro_manifest(estate_root, &new_manifest)?;

    Ok(outcome)
}

/// `AGENTS.md`'s managed-section write (issue #261, owner ruling point 3).
///
/// `AGENTS.md` is not written as one opaque, never-touched-again blob: init
/// owns only the text between `sgt:managed:begin`/`sgt:managed:end`
/// comment markers.
///
/// - No `AGENTS.md` yet: write the full file, with the embedded body
///   wrapped in fresh markers.
/// - `AGENTS.md` exists and carries both markers: rewrite only the text
///   between them to the current embedded body, byte for byte; everything
///   before `begin` and after `end` — an estate's own appended content —
///   is preserved untouched.
/// - `AGENTS.md` exists but carries no markers: fail closed
///   ([`DistroError::UnmanagedAgentsMd`]) rather than guess whether it is
///   safe to overwrite or where to insert markers into a hand-authored
///   file. This never happens on a fresh estate; it is the sole case
///   `write_distro` can return before the rest of the distro is written.
fn write_agents_md(
    estate_root: &Path,
    outcome: &mut DistroWriteOutcome,
) -> Result<(), DistroError> {
    let rel_path = Path::new("AGENTS.md");
    let full_path = estate_root.join(rel_path);

    let existing = match std::fs::read_to_string(&full_path) {
        // An empty file has nothing to lose by treating it as absent — the
        // `UnmanagedAgentsMd` guard exists to protect real hand-authored
        // content, and `DistroError::UnmanagedAgentsMd`'s own doc says this
        // fires when `AGENTS.md` "exists, is not empty" (#241/#261
        // F-INV-05).
        Ok(text) if text.is_empty() => None,
        Ok(text) => Some(text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    };

    let rewritten = match existing {
        None => format!("{MANAGED_BEGIN}\n{AGENTS_MD}{MANAGED_END}\n"),
        Some(existing) => {
            let (Some(begin_at), Some(end_at)) =
                (existing.find(MANAGED_BEGIN), existing.find(MANAGED_END))
            else {
                return Err(DistroError::UnmanagedAgentsMd);
            };
            if end_at < begin_at {
                return Err(DistroError::UnmanagedAgentsMd);
            }
            let before = &existing[..begin_at];
            let after = &existing[end_at + MANAGED_END.len()..];
            format!("{before}{MANAGED_BEGIN}\n{AGENTS_MD}{MANAGED_END}{after}")
        }
    };

    write_if_changed(&full_path, rel_path, rewritten.as_bytes(), outcome)?;
    Ok(())
}

/// `CLAUDE.md -> AGENTS.md` (owner ruling point 2): a symlink so a harness
/// that only looks for `CLAUDE.md` still finds the one constitution.
///
/// Never overwrites a pre-existing regular (non-symlink) `CLAUDE.md` — an
/// estate that already has its own is left exactly alone. When the
/// platform or filesystem doesn't support symlinks, the attempt is made
/// and its failure is swallowed: `sgt init` never fails because of this,
/// it just leaves `CLAUDE.md` absent.
fn symlink_claude_md(estate_root: &Path, outcome: &mut DistroWriteOutcome) -> io::Result<()> {
    let rel_path = Path::new("CLAUDE.md");
    let full_path = estate_root.join(rel_path);

    match std::fs::symlink_metadata(&full_path) {
        Ok(_) => {
            // Already there — a symlink from a prior init, or a real file
            // this estate owns. Either way, never overwritten.
            outcome.skipped.push(rel_path.to_path_buf());
            return Ok(());
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }

    match create_symlink(Path::new("AGENTS.md"), &full_path) {
        Ok(()) => outcome.written.push(rel_path.to_path_buf()),
        // Symlinks unsupported on this platform/filesystem (or any other
        // symlink-creation fault): skip gracefully, per the owner ruling
        // ("if the platform doesn't support symlinks, skip gracefully —
        // don't fail init") — but record why, rather than swallowing it
        // where nothing can see it (#241/#261 F-INV-06).
        Err(e) => outcome.symlink_unavailable = Some(e.to_string()),
    }
    Ok(())
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(not(any(unix, windows)))]
fn create_symlink(_target: &Path, _link: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "symlinks are not supported on this platform",
    ))
}

/// Make `estate_root/prefix` exactly match `dir`'s embedded content: every
/// embedded file is written (or left alone if already byte-identical), and
/// every file on disk under `estate_root/prefix` that is **not** part of
/// the current embed is removed (#241's retired-package cleanup).
fn sync_owned_dir(
    estate_root: &Path,
    prefix: &Path,
    dir: &Dir<'static>,
    prior_manifest: &HashMap<PathBuf, String>,
    new_manifest: &mut HashMap<PathBuf, String>,
    outcome: &mut DistroWriteOutcome,
) -> io::Result<()> {
    let mut embedded_rel: HashSet<PathBuf> = HashSet::new();
    write_owned_files(
        estate_root,
        prefix,
        dir,
        &mut embedded_rel,
        prior_manifest,
        new_manifest,
        outcome,
    )?;
    remove_retired_files(estate_root, prefix, &embedded_rel, outcome)?;
    Ok(())
}

/// Recurse `dir` (an `include_dir!` tree), writing every file it contains
/// under `estate_root/prefix/<file's own path within dir>` — `File::path()`
/// already carries the full path relative to the tree's own root, so no
/// manual path accumulation is needed as this recurses into subdirectories.
/// Records every path written into `embedded_rel`, which
/// [`remove_retired_files`] then uses to find what does *not* belong.
fn write_owned_files(
    estate_root: &Path,
    prefix: &Path,
    dir: &Dir<'static>,
    embedded_rel: &mut HashSet<PathBuf>,
    prior_manifest: &HashMap<PathBuf, String>,
    new_manifest: &mut HashMap<PathBuf, String>,
    outcome: &mut DistroWriteOutcome,
) -> io::Result<()> {
    for file in dir.files() {
        let rel_path = prefix.join(file.path());
        embedded_rel.insert(rel_path.clone());
        write_owned_file(
            estate_root,
            &rel_path,
            file.contents(),
            prior_manifest,
            new_manifest,
            outcome,
        )?;
    }
    for sub in dir.dirs() {
        write_owned_files(
            estate_root,
            prefix,
            sub,
            embedded_rel,
            prior_manifest,
            new_manifest,
            outcome,
        )?;
    }
    Ok(())
}

/// Write one owned file, **overwriting** whatever is already there when
/// the content differs — the three owned trees are sgt-owned, not
/// user-owned (unlike `.sergeant/local/`, which this function is never
/// called on), so a stale or user-edited copy inside them is replaced
/// rather than preserved. Skips the actual write when the target already
/// holds byte-identical content, which is what keeps a same-binary second
/// `sgt init` a true no-op.
///
/// When an overwrite does happen, `prior_manifest` (what `sgt init` itself
/// last recorded writing to this path, see [`DISTRO_MANIFEST_PATH`]) is
/// consulted to tell a hand edit apart from ordinary version drift: if the
/// on-disk content about to be replaced doesn't match that record, nothing
/// but `sgt init` could have written what's there now, so it is reported in
/// `outcome.overwritten_modified` (#241/#261 F6) as well as `outcome.written`.
fn write_owned_file(
    estate_root: &Path,
    rel_path: &Path,
    contents: &[u8],
    prior_manifest: &HashMap<PathBuf, String>,
    new_manifest: &mut HashMap<PathBuf, String>,
    outcome: &mut DistroWriteOutcome,
) -> io::Result<()> {
    let full_path = estate_root.join(rel_path);
    let carries_edition = matches!(
        rel_path.file_name().and_then(|n| n.to_str()),
        Some("index.md") | Some("SKILL.md")
    );
    let rewritten;
    let final_bytes: &[u8] = if carries_edition && let Ok(text) = std::str::from_utf8(contents) {
        rewritten = rewrite_edition(text).into_bytes();
        &rewritten
    } else {
        contents
    };

    new_manifest.insert(rel_path.to_path_buf(), content_hash(final_bytes));

    if let Ok(existing) = std::fs::read(&full_path)
        && existing != final_bytes
    {
        let matches_prior_write = prior_manifest
            .get(rel_path)
            .is_some_and(|prior_hash| content_hash(&existing) == *prior_hash);
        if !matches_prior_write {
            outcome.overwritten_modified.push(rel_path.to_path_buf());
        }
    }

    write_if_changed(&full_path, rel_path, final_bytes, outcome)
}

/// Write `contents` to `full_path` (recorded under `rel_path` in `outcome`)
/// only if it differs from what is already there byte for byte — creating
/// any missing parent directories first when it does. Shared tail for
/// `write_agents_md`, `write_owned_file`, and `write_index_md`, which each
/// independently hand-duplicated this same
/// read-existing/compare/write-if-different/record-outcome shape
/// (#241/#261 F-SI-01).
fn write_if_changed(
    full_path: &Path,
    rel_path: &Path,
    contents: &[u8],
    outcome: &mut DistroWriteOutcome,
) -> io::Result<()> {
    if let Ok(existing) = std::fs::read(full_path)
        && existing == contents
    {
        outcome.skipped.push(rel_path.to_path_buf());
        return Ok(());
    }
    if let Some(parent) = full_path.parent() {
        create_dir_all_durable(parent)?;
    }
    write_atomic(full_path, contents)?;
    outcome.written.push(rel_path.to_path_buf());
    Ok(())
}

/// Delete every file on disk under `estate_root/prefix` that is not a key
/// of `embedded_rel`, then prune any directory left empty by that removal.
///
/// **Scope guarantee**: this function only ever reads and deletes inside
/// `estate_root.join(prefix)`. Every call site in this module passes one
/// of the three sgt-owned trees (`skills`, `.sergeant/common/contexts`,
/// `.sergeant/workflows`) as `prefix` — `.sergeant/local/` is never passed
/// here and this function has no path to reach it; that is what keeps
/// #241's cleanup from ever touching the user's own workflow namespace.
fn remove_retired_files(
    estate_root: &Path,
    prefix: &Path,
    embedded_rel: &HashSet<PathBuf>,
    outcome: &mut DistroWriteOutcome,
) -> io::Result<()> {
    let owned_root = estate_root.join(prefix);
    remove_retired_and_prune(&owned_root, estate_root, embedded_rel, outcome)
}

/// One combined post-order pass over `dir`: delete every file not present
/// in `embedded_rel`, then remove any subdirectory (including `dir`
/// itself) left with nothing in it — previously two independent full
/// recursive `read_dir` traversals of the same tree (`walk_files` then
/// `prune_empty_dirs`), now one (#241/#261 F-SI-02). Directory pruning is
/// cosmetic (an empty directory is harmless), but a retired package's
/// directory disappearing along with its files is the less surprising
/// result.
///
/// **Never follows symlinks (issue #241/#261 F5).** Every entry's type is
/// read with [`std::fs::DirEntry::file_type`], which reports the entry
/// itself — a symlink is reported as a symlink, never as whatever it
/// points at (unlike `Path::is_dir`/`Path::metadata`, which resolve the
/// link). A symlink entry is always a single leaf to consider for removal:
/// this function never recurses through it and never resolves it before
/// deleting. That is what keeps a symlink planted inside an owned tree —
/// e.g. pointing at `.sergeant/local/` or at an absolute path outside the
/// estate entirely — from ever causing anything beyond the symlink itself
/// to be touched: `std::fs::remove_file` on a symlink path unlinks the
/// link, not its target.
fn remove_retired_and_prune(
    dir: &Path,
    estate_root: &Path,
    embedded_rel: &HashSet<PathBuf>,
    outcome: &mut DistroWriteOutcome,
) -> io::Result<()> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            remove_retired_and_prune(&path, estate_root, embedded_rel, outcome)?;
            if std::fs::read_dir(&path).is_ok_and(|mut e| e.next().is_none()) {
                let _ = std::fs::remove_dir(&path);
            }
            continue;
        }
        // A symlink (to a file, a directory, or nothing that resolves at
        // all) falls through here rather than into the `is_dir` branch
        // above, so it is always handled as a single entry to unlink —
        // never recursed into, never canonicalized first.
        let rel_path = match path.strip_prefix(estate_root) {
            Ok(rel) => rel.to_path_buf(),
            Err(_) => continue,
        };
        if !embedded_rel.contains(&rel_path) {
            match std::fs::remove_file(&path) {
                Ok(()) => outcome.removed.push(rel_path),
                // A concurrent `sgt init` already removed this same retired
                // file — benign race, not a fault (#241/#261 F-INV-04): the
                // end state either process wants (the file gone) already
                // holds.
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
        }
    }
    Ok(())
}

/// Where the distro manifest (#241/#261 F6) is persisted, relative to the
/// estate root — deliberately outside all three sgt-owned trees
/// (`skills/`, `.sergeant/common/contexts/`, `.sergeant/workflows/`), so
/// [`remove_retired_files`]'s cleanup — scoped strictly to those trees —
/// never treats this bookkeeping file itself as a retired owned file.
const DISTRO_MANIFEST_PATH: &str = ".sergeant/.distro-manifest.json";

/// The content hash [`DISTRO_MANIFEST_PATH`] records per owned file —
/// `blake3` is already an established dependency for content-identity
/// hashing elsewhere in this codebase (`runtime::blob`, `domain::workflow`).
fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Load what the *last* `sgt init` on this estate recorded writing to each
/// owned-tree path, keyed by path relative to the estate root. Absent,
/// unreadable, or malformed is treated the same as empty — an estate whose
/// owned trees predate this manifest (or whose manifest was deleted) simply
/// has no baseline to compare against, which [`write_owned_file`] resolves
/// conservatively: no record means an overwrite is reported as a hand edit
/// rather than assumed innocent.
fn load_distro_manifest(estate_root: &Path) -> HashMap<PathBuf, String> {
    let full_path = estate_root.join(DISTRO_MANIFEST_PATH);
    let Ok(text) = std::fs::read_to_string(&full_path) else {
        return HashMap::new();
    };
    let Ok(entries) = serde_json::from_str::<HashMap<String, String>>(&text) else {
        return HashMap::new();
    };
    entries
        .into_iter()
        .map(|(path, hash)| (PathBuf::from(path), hash))
        .collect()
}

/// Persist `manifest` as the baseline the *next* `sgt init` reads back via
/// [`load_distro_manifest`]. Written unconditionally at the end of every
/// successful [`write_distro`] call — cheap (one small JSON file) and it
/// must always reflect exactly what this call left on disk, including for
/// files this call left untouched because they already matched the embed.
fn save_distro_manifest(estate_root: &Path, manifest: &HashMap<PathBuf, String>) -> io::Result<()> {
    let as_strings: std::collections::BTreeMap<String, &String> = manifest
        .iter()
        .map(|(path, hash)| (path.to_string_lossy().into_owned(), hash))
        .collect();
    let text = serde_json::to_string_pretty(&as_strings)
        .expect("a map of strings always serializes to JSON");

    let rel_path = Path::new(DISTRO_MANIFEST_PATH);
    let full_path = estate_root.join(rel_path);
    if let Some(parent) = full_path.parent() {
        create_dir_all_durable(parent)?;
    }
    write_atomic(&full_path, text.as_bytes())
}

/// Generate `.sergeant/index.md` from the workflow packages actually on
/// disk under `.sergeant/workflows/` post-sync (issue #261 fix option 1):
/// read back rather than hand-embedded as a fifth static copy, so the
/// catalog can never itself drift from the packages `sgt init` just wrote
/// — the exact failure mode a second, independently-maintained copy would
/// reintroduce. Has no user-editable surface (there is nothing here for an
/// estate to have customized), so it is regenerated — compared, then
/// written only if different — on every `sgt init`, the same way
/// `AGENTS.md`'s managed section is.
fn write_index_md(estate_root: &Path, outcome: &mut DistroWriteOutcome) -> io::Result<()> {
    let workflows_dir = estate_root.join(WORKFLOW_ROOT);
    let mut names: Vec<String> = std::fs::read_dir(&workflows_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.file_type().is_ok_and(|t| t.is_dir()))
                .filter(|entry| entry.path().join("workflow.toml").is_file())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();

    let mut body = String::new();
    body.push_str(
        "<!-- Generated by `sgt init` from the workflow packages shipped under \
         `.sergeant/workflows/`. Regenerated on every `sgt init` — hand edits here \
         do not survive a re-init. -->\n\n",
    );
    body.push_str("# Workflow catalog\n\n");
    body.push_str(
        "Every admitted workflow this estate ships, generated from each package's own \
         `index.md` front matter.\n\n",
    );
    body.push_str("| Workflow | Status | Index |\n");
    body.push_str("|---|---|---|\n");
    for name in &names {
        let status = read_index_front_matter(&workflows_dir.join(name))
            .map(|fm| fm.status)
            .unwrap_or_else(|| "published".to_string());
        let index_path = format!("{WORKFLOW_ROOT}/{name}/index.md");
        body.push_str(&format!(
            "| `{name}` | {status} | [`{index_path}`]({index_path}) |\n"
        ));
    }

    let rel_path = Path::new(ROOT_CATALOG_FILE);
    let full_path = estate_root.join(rel_path);
    write_if_changed(&full_path, rel_path, body.as_bytes(), outcome)
}

/// Replace the value of an `edition:` line inside the leading `---`
/// front-matter block with this binary's own version. A file with no such
/// line, or no front-matter block at all, is returned unchanged — this is
/// deliberately narrower than a YAML parser (matching
/// `domain::workflow::parse_index_front_matter`'s own "tiny local
/// composition" rather than a general parser, Ponytail R6), scoped to
/// exactly the shape ADR 0016 fixes.
fn rewrite_edition(text: &str) -> String {
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return text.to_string();
    };
    if first.trim() != "---" {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len() + 8);
    out.push_str(first);
    out.push('\n');
    let mut in_front_matter = true;
    for line in lines {
        if in_front_matter && line.trim() == "---" {
            in_front_matter = false;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_front_matter && line.trim_start().starts_with("edition:") {
            out.push_str("edition: ");
            out.push_str(env!("CARGO_PKG_VERSION"));
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_edition_replaces_the_value_inside_front_matter_only() {
        let text = "---\nkind: workflow\nedition: 0.0.1\nname: x\n---\n\nedition: 0.0.1 in the body must survive\n";
        let rewritten = rewrite_edition(text);
        assert!(rewritten.contains(&format!("edition: {}", env!("CARGO_PKG_VERSION"))));
        assert!(rewritten.contains("edition: 0.0.1 in the body must survive"));
    }

    #[test]
    fn rewrite_edition_is_a_no_op_without_front_matter() {
        let text = "no front matter here\nedition: 0.0.1\n";
        assert_eq!(rewrite_edition(text), text);
    }

    #[test]
    fn write_distro_is_owned_tree_idempotent() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let first = write_distro(tmp.path()).expect("first write");
        assert!(first.changed());
        assert!(tmp.path().join("AGENTS.md").is_file());
        assert!(tmp.path().join(".sergeant/workflows").is_dir());

        let second = write_distro(tmp.path()).expect("second write");
        assert!(
            !second.changed(),
            "a second call with the same binary must write and remove nothing, got \
             written={:?} removed={:?}",
            second.written,
            second.removed
        );
    }

    #[test]
    fn write_distro_overwrites_a_drifted_owned_file() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("first write");

        // Any real shipped skill file works as the drift target.
        let target = tmp.path().join("skills/sergeant-help/SKILL.md");
        std::fs::write(&target, "drifted content, must be replaced").expect("simulate drift");

        let second = write_distro(tmp.path()).expect("second write");
        assert!(
            second.changed(),
            "an owned-tree file that drifted from the embed must be rewritten"
        );
        let text = std::fs::read_to_string(&target).expect("read back");
        assert_ne!(text, "drifted content, must be replaced");
    }

    /// #241/#261 F6 (major): a file inside an owned tree edited by
    /// something other than `sgt init` since the last init must be named
    /// distinctly in the outcome, so a user's lost edit is loudly visible.
    #[test]
    fn write_distro_flags_a_hand_edited_owned_file_as_overwritten_modified() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("first write");

        let target = tmp.path().join("skills/sergeant-help/SKILL.md");
        std::fs::write(&target, "a user's own hand edit, never written by sgt init")
            .expect("simulate hand edit");

        let second = write_distro(tmp.path()).expect("second write");
        assert!(
            second
                .overwritten_modified
                .iter()
                .any(|p| p.ends_with("skills/sergeant-help/SKILL.md")),
            "a file hand-edited since the last init must be flagged, got {:?}",
            second.overwritten_modified
        );
        assert!(
            second
                .written
                .iter()
                .any(|p| p.ends_with("skills/sergeant-help/SKILL.md")),
            "a flagged hand-edit is still overwritten like any other drifted owned file"
        );
    }

    /// #241/#261 F6: a file that only drifted because it still holds
    /// exactly what `sgt init` itself wrote under a previous release must
    /// *not* be flagged as a hand edit — only content nothing but `sgt
    /// init` could have produced escapes that label.
    #[test]
    fn write_distro_does_not_flag_routine_version_drift_as_overwritten_modified() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("first write");

        let target = tmp.path().join("skills/sergeant-help/SKILL.md");
        let previous_release_content = "content sgt itself wrote under a previous release";
        std::fs::write(&target, previous_release_content)
            .expect("simulate a previous release's own content");

        // Backdate the manifest to say `sgt init` itself last wrote exactly
        // this content — simulating the file sitting untouched since an
        // older release, rather than hand-edited by anyone.
        let manifest_path = tmp.path().join(DISTRO_MANIFEST_PATH);
        let mut manifest: std::collections::BTreeMap<String, String> =
            serde_json::from_str(&std::fs::read_to_string(&manifest_path).expect("read manifest"))
                .expect("parse manifest");
        manifest.insert(
            "skills/sergeant-help/SKILL.md".to_string(),
            content_hash(previous_release_content.as_bytes()),
        );
        std::fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).expect("serialize manifest"),
        )
        .expect("write backdated manifest");

        let second = write_distro(tmp.path()).expect("second write");
        assert!(
            second
                .written
                .iter()
                .any(|p| p.ends_with("skills/sergeant-help/SKILL.md")),
            "the file must still be brought back in line with the current embed"
        );
        assert!(
            !second
                .overwritten_modified
                .iter()
                .any(|p| p.ends_with("skills/sergeant-help/SKILL.md")),
            "content sgt itself wrote under a previous release must not be flagged as a hand \
             edit, got {:?}",
            second.overwritten_modified
        );
    }

    #[test]
    fn write_distro_removes_a_retired_owned_file() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("first write");

        let retired = tmp
            .path()
            .join(".sergeant/workflows/retired-package-not-in-embed/index.md");
        std::fs::create_dir_all(retired.parent().unwrap()).expect("mkdir");
        std::fs::write(&retired, "a package the current embed no longer ships").expect("plant");
        assert!(retired.exists());

        let second = write_distro(tmp.path()).expect("second write");
        assert!(
            !retired.exists(),
            "a file inside an owned tree not present in the current embed must be removed"
        );
        assert!(
            second
                .removed
                .iter()
                .any(|p| p.ends_with("retired-package-not-in-embed/index.md")),
            "the outcome must report the removal, got {:?}",
            second.removed
        );
        assert!(
            !retired.parent().unwrap().exists(),
            "the now-empty retired package directory should be pruned"
        );
    }

    #[test]
    fn write_distro_never_touches_local() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".sergeant/local/workflows/mine"))
            .expect("mkdir local");
        std::fs::write(
            tmp.path()
                .join(".sergeant/local/workflows/mine/workflow.toml"),
            "a user's own local workflow",
        )
        .expect("plant local file");

        write_distro(tmp.path()).expect("write");

        assert!(
            tmp.path()
                .join(".sergeant/local/workflows/mine/workflow.toml")
                .is_file(),
            "sgt-owned tree sync must never touch .sergeant/local/"
        );
        assert_eq!(
            std::fs::read_to_string(
                tmp.path()
                    .join(".sergeant/local/workflows/mine/workflow.toml")
            )
            .expect("read local file"),
            "a user's own local workflow",
            "a user's own local content must survive byte-for-byte"
        );
    }

    #[test]
    fn write_distro_stamps_the_binary_edition() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("write");

        let mut checked_a_workflow = false;
        for entry in std::fs::read_dir(tmp.path().join(".sergeant/workflows")).expect("read_dir") {
            let entry = entry.expect("entry");
            let index = entry.path().join("index.md");
            if index.is_file() {
                let text = std::fs::read_to_string(&index).expect("read index.md");
                assert!(
                    text.contains(&format!("edition: {}", env!("CARGO_PKG_VERSION"))),
                    "{} must carry the binary's edition, got:\n{text}",
                    index.display()
                );
                checked_a_workflow = true;
            }
        }
        assert!(checked_a_workflow, "expected at least one stock index.md");

        for entry in std::fs::read_dir(tmp.path().join("skills")).expect("read_dir") {
            let entry = entry.expect("entry");
            let skill_md = entry.path().join("SKILL.md");
            if skill_md.is_file() {
                let text = std::fs::read_to_string(&skill_md).expect("read SKILL.md");
                assert!(
                    text.contains(&format!("edition: {}", env!("CARGO_PKG_VERSION"))),
                    "{} must carry the binary's edition, got:\n{text}",
                    skill_md.display()
                );
            }
        }
    }

    #[test]
    fn write_distro_creates_the_claude_md_symlink() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("write");

        let claude_md = tmp.path().join("CLAUDE.md");
        let meta = std::fs::symlink_metadata(&claude_md).expect("CLAUDE.md must exist");
        if meta.file_type().is_symlink() {
            let target = std::fs::read_link(&claude_md).expect("read_link");
            assert_eq!(target, Path::new("AGENTS.md"));
        }
        // On a platform without symlink support the file simply won't
        // exist at all — asserted separately below via the AGENTS.md
        // content check, which does not depend on the symlink.
        assert!(
            std::fs::read_to_string(&claude_md)
                .expect("CLAUDE.md must be readable")
                .contains(MANAGED_BEGIN),
            "CLAUDE.md must resolve to AGENTS.md's own managed content"
        );
    }

    #[test]
    fn write_distro_never_overwrites_a_pre_existing_regular_claude_md() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let claude_md = tmp.path().join("CLAUDE.md");
        std::fs::write(&claude_md, "this estate's own hand-authored CLAUDE.md")
            .expect("plant a pre-existing regular CLAUDE.md");

        let outcome = write_distro(tmp.path()).expect("write");

        let meta = std::fs::symlink_metadata(&claude_md).expect("CLAUDE.md must still exist");
        assert!(
            !meta.file_type().is_symlink(),
            "a pre-existing regular CLAUDE.md must never be replaced with a symlink"
        );
        assert_eq!(
            std::fs::read_to_string(&claude_md).expect("read CLAUDE.md"),
            "this estate's own hand-authored CLAUDE.md",
            "a pre-existing regular CLAUDE.md's content must survive byte-for-byte"
        );
        assert!(
            outcome.skipped.iter().any(|p| p == Path::new("CLAUDE.md")),
            "the pre-existing CLAUDE.md must be reported (skipped), not silently ignored, got \
             {:?}",
            outcome.skipped
        );
    }

    #[test]
    fn write_distro_generates_the_index_catalog() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("write");

        let index = std::fs::read_to_string(tmp.path().join(ROOT_CATALOG_FILE))
            .expect("read .sergeant/index.md");
        assert!(index.contains("| Workflow | Status | Index |"));
        // Every workflow package actually written must be named in the
        // generated catalog.
        for entry in std::fs::read_dir(tmp.path().join(WORKFLOW_ROOT)).expect("read_dir") {
            let entry = entry.expect("entry");
            if entry.path().join("workflow.toml").is_file() {
                let name = entry.file_name().to_string_lossy().into_owned();
                assert!(
                    index.contains(&format!("`{name}`")),
                    ".sergeant/index.md must list workflow package {name:?}, got:\n{index}"
                );
            }
        }
    }

    #[test]
    fn write_agents_md_wraps_a_fresh_file_in_managed_markers() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("write");

        let text = std::fs::read_to_string(tmp.path().join("AGENTS.md")).expect("read AGENTS.md");
        assert!(text.starts_with(MANAGED_BEGIN));
        assert!(text.contains(MANAGED_END));
        assert!(text.contains(AGENTS_MD));
    }

    #[test]
    fn write_agents_md_preserves_content_outside_the_markers_on_reinit() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let agents_md = tmp.path().join("AGENTS.md");
        std::fs::create_dir_all(tmp.path()).expect("mkdir");
        let custom = format!(
            "# This estate's own preamble\n\nCustom text before.\n\n{MANAGED_BEGIN}\nstale \
             body\n{MANAGED_END}\n\nCustom text after.\n"
        );
        std::fs::write(&agents_md, &custom).expect("plant custom AGENTS.md");

        write_distro(tmp.path()).expect("write");

        let text = std::fs::read_to_string(&agents_md).expect("read AGENTS.md");
        assert!(text.contains("Custom text before."));
        assert!(text.contains("Custom text after."));
        assert!(text.contains(AGENTS_MD));
        assert!(!text.contains("stale body"));
    }

    #[test]
    fn write_agents_md_treats_an_existing_empty_file_as_absent() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let agents_md = tmp.path().join("AGENTS.md");
        std::fs::write(&agents_md, "").expect("plant an empty AGENTS.md");

        write_distro(tmp.path()).expect("an empty AGENTS.md must not fail closed");

        let text = std::fs::read_to_string(&agents_md).expect("read AGENTS.md");
        assert!(text.starts_with(MANAGED_BEGIN));
        assert!(text.contains(AGENTS_MD));
    }

    #[test]
    fn write_agents_md_fails_closed_without_markers() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let agents_md = tmp.path().join("AGENTS.md");
        std::fs::write(
            &agents_md,
            "a hand-authored constitution with no markers at all\n",
        )
        .expect("plant unmanaged AGENTS.md");

        let err = write_distro(tmp.path()).expect_err("must fail closed");
        assert!(matches!(err, DistroError::UnmanagedAgentsMd));

        let text = std::fs::read_to_string(&agents_md).expect("read AGENTS.md");
        assert_eq!(
            text, "a hand-authored constitution with no markers at all\n",
            "a marker-less AGENTS.md must never be corrupted by a failed init"
        );
    }

    /// #241/#261 F5 (BLOCKER): a symlink planted inside an owned tree,
    /// pointing at `.sergeant/local/` — a routine `sgt init` must remove
    /// only the symlink itself, never resolve it and delete through it.
    #[test]
    #[cfg(unix)]
    fn remove_retired_files_never_follows_a_symlink_into_sergeant_local() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("first write");

        let local_dir = tmp.path().join(".sergeant/local");
        std::fs::create_dir_all(&local_dir).expect("mkdir .sergeant/local");
        let local_file = local_dir.join("file");
        std::fs::write(&local_file, "the user's own local content").expect("plant local file");

        // Planted inside a sgt-owned tree, not part of the current embed —
        // exactly the shape retired-file cleanup is meant to remove, except
        // this entry is a symlink pointing outside the owned tree.
        let evil_link = tmp.path().join("skills/evil-symlink");
        std::os::unix::fs::symlink(&local_file, &evil_link).expect("plant symlink");

        let outcome = write_distro(tmp.path()).expect("second write");

        assert!(
            !evil_link.exists() && std::fs::symlink_metadata(&evil_link).is_err(),
            "the symlink itself must be removed as a retired file"
        );
        assert!(
            outcome.removed.iter().any(|p| p.ends_with("evil-symlink")),
            "the outcome must report the symlink's own removal, got {:?}",
            outcome.removed
        );
        assert!(
            local_file.is_file(),
            ".sergeant/local/file must never be reachable through owned-tree cleanup"
        );
        assert_eq!(
            std::fs::read_to_string(&local_file).expect("read local file"),
            "the user's own local content",
            "the symlink target's content must be untouched"
        );
    }

    /// #241/#261 F5 (BLOCKER): a symlink planted inside an owned tree,
    /// pointing at an absolute path entirely outside the estate — a
    /// routine `sgt init` must remove only the symlink itself, never the
    /// file it points at.
    #[test]
    #[cfg(unix)]
    fn remove_retired_files_never_follows_a_symlink_outside_the_estate() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        write_distro(tmp.path()).expect("first write");

        let outside = tempfile::TempDir::new().expect("outside tempdir");
        let outside_file = outside.path().join("arbitrary-file");
        std::fs::write(&outside_file, "content living entirely outside the estate")
            .expect("plant outside file");

        let evil_link = tmp.path().join(".sergeant/common/contexts/evil-symlink");
        std::os::unix::fs::symlink(&outside_file, &evil_link).expect("plant symlink");

        let outcome = write_distro(tmp.path()).expect("second write");

        assert!(
            !evil_link.exists() && std::fs::symlink_metadata(&evil_link).is_err(),
            "the symlink itself must be removed as a retired file"
        );
        assert!(
            outcome.removed.iter().any(|p| p.ends_with("evil-symlink")),
            "the outcome must report the symlink's own removal, got {:?}",
            outcome.removed
        );
        assert!(
            outside_file.is_file(),
            "a file outside the estate must never be reachable through owned-tree cleanup"
        );
        assert_eq!(
            std::fs::read_to_string(&outside_file).expect("read outside file"),
            "content living entirely outside the estate",
            "the symlink target's content must be untouched"
        );
    }
}

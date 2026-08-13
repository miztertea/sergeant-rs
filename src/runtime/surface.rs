//! Work surfaces: isolated git worktrees per repository (proposal §11).
//!
//! Execution never happens in the source checkout. A Work gets a *surface* —
//! one git worktree per targeted repository, on its own branch, materialized
//! under the daemon's data dir:
//!
//! ```text
//! <data-dir>/surfaces/<work-id>/api/     git worktree on sergeant/<work-id>
//! <data-dir>/surfaces/<work-id>/web/     git worktree on sergeant/<work-id>
//! ```
//!
//! Each binding records exactly what §11 asks for: source repository, base
//! branch, base SHA, worktree path, work branch, current HEAD. Those records
//! are journaled, so the surface can be described, torn down, and rebuilt
//! after a restart without re-inspecting anything.
//!
//! Teardown retains the branch and removes the worktree — the branch is the
//! durable output of a run, the worktree is scaffolding. Teardown **fails
//! closed**: a worktree with uncommitted or untracked changes, a worktree that
//! has vanished, or a removal Git refuses is *recorded* in the teardown report
//! and left alone. Sergeant never destroys work it did not create. The
//! per-work root goes too, but only once it is empty — `remove_dir`, never a
//! recursive delete, so anything teardown retained keeps the directory it
//! lives in.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};

use crate::domain::workspace::RepositorySpec;
use crate::runtime::fsutil::create_dir_all_durable;
use crate::runtime::git::{GitError, git, git_submodule_update, git_succeeds};

/// Directory under the data dir holding all work surfaces.
pub const SURFACES_DIR: &str = "surfaces";

/// One lock per source repository, held across every worktree mutation this
/// module makes against that repository.
///
/// **Not the core lock, and that is the point.** N3 moved git out from under
/// the daemon's single writer (§22.6), which is what the budget asks for — and
/// promptly let fifty submissions run `git worktree add`/`remove` against the
/// same `.git` at once. Git does not serialize that for us: measured on a
/// burst-50 run, two teardowns failed with `fatal: failed to read
/// .git/worktrees/<other-work>/commondir` — one process walking the worktree
/// registry while another rewrote it. Failing closed meant those surfaces were
/// *retained*, journaled and left on disk, which is honest and still wrong.
///
/// So concurrency against one repository is serialized here, at the narrowest
/// scope that fixes it: per source path, for the duration of the git calls
/// that touch its registry. Two works in different repositories still proceed
/// in parallel, and no request anywhere queues behind the core lock for it.
fn repository_lock(source: &Path) -> Arc<Mutex<()>> {
    static LOCKS: OnceLock<Mutex<BTreeMap<PathBuf, Arc<Mutex<()>>>>> = OnceLock::new();
    let table = LOCKS.get_or_init(|| Mutex::new(BTreeMap::new()));
    // Keyed on the canonical path: two workspaces can reach one repository by
    // different routes, and a lock they do not share is not a lock.
    let key = std::fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
    let mut table = table.lock().unwrap_or_else(|e| e.into_inner());
    Arc::clone(table.entry(key).or_default())
}

/// Run `f` with this repository's worktree registry to itself.
///
/// Poisoning is deliberately ignored: the guard protects git's on-disk
/// registry, not an in-memory invariant, so a panic in one caller leaves
/// nothing for the next one to be confused by — and refusing every later
/// worktree operation for the daemon's lifetime would be a far worse failure
/// than the one being guarded against.
fn with_repository<T>(source: &Path, f: impl FnOnce() -> T) -> T {
    let lock = repository_lock(source);
    let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());
    f()
}

/// Event kind: a work surface is about to be materialized. Appended *before*
/// the first `git worktree add`, so the branch and worktree a crash can leave
/// behind in the user's repositories are never unrecorded.
pub const KIND_SURFACE_MATERIALIZING: &str = "surface.materializing";
/// Event kind: a work surface was materialized.
pub const KIND_SURFACE_MATERIALIZED: &str = "surface.materialized";
/// Event kind: a work surface was torn down (with per-binding disposition).
pub const KIND_SURFACE_TORN_DOWN: &str = "surface.torn_down";

/// Branch a work executes on.
pub fn work_branch(work_id: &str) -> String {
    format!("sergeant/{work_id}")
}

/// What §11 requires each repository binding to record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryBinding {
    /// Repository name within the workspace.
    pub repository: String,
    /// Source repository top level (never written to by execution).
    pub source_path: PathBuf,
    /// Branch the surface was cut from (`(detached)` if HEAD was detached).
    pub base_branch: String,
    /// Commit the surface was cut from.
    pub base_sha: String,
    /// Worktree path under the data dir.
    pub worktree_path: PathBuf,
    /// Branch the work executes on.
    pub work_branch: String,
    /// HEAD of the worktree as last recorded.
    pub head_sha: String,
}

/// What materialization is about to create, recorded before it creates any of
/// it.
///
/// `git worktree add` writes to the *user's* repository — a new branch and a
/// registered worktree — so the window between "sergeant decided to do that"
/// and "sergeant journaled that it did" must not exist. This is the record
/// that closes it: it names every path and branch materialization may leave
/// behind, so a crash mid-way is recoverable evidence rather than a mystery
/// `sergeant/<work-id>` branch in someone's checkout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfacePlan {
    /// Directory the worktrees will be created under.
    pub root: PathBuf,
    /// Branch every worktree will be cut onto.
    pub work_branch: String,
    /// Repositories that will get a worktree, in workspace order.
    pub repositories: Vec<RepositorySpec>,
}

impl SurfacePlan {
    /// The plan `materialize` will follow for this work.
    ///
    /// `surfaces_root` (R-MVP1-1) is the directory work surfaces are created
    /// directly under — already resolved to whatever the caller wants
    /// (`<data_dir>/surfaces` by default, an estate's `[estate]
    /// surfaces_dir`, or `SGT_SURFACES_DIR`), not necessarily anywhere near
    /// `data_dir`. This function does not append anything else onto it.
    pub fn new(surfaces_root: &Path, work_id: &str, repositories: &[RepositorySpec]) -> Self {
        Self {
            root: surface_root(surfaces_root, work_id),
            work_branch: work_branch(work_id),
            repositories: repositories.to_vec(),
        }
    }
}

/// A materialized work surface: one binding per targeted repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkSurface {
    /// Work this surface belongs to.
    pub work_id: String,
    /// Root directory holding the worktrees.
    pub root: PathBuf,
    /// One binding per repository, in workspace order.
    pub bindings: Vec<RepositoryBinding>,
}

impl WorkSurface {
    /// Directory an execution should run in: the single worktree for
    /// one-repository work, the surface root when several repositories are
    /// bound (§11's multi-repo shape).
    pub fn execution_cwd(&self) -> PathBuf {
        match self.bindings.as_slice() {
            [only] => only.worktree_path.clone(),
            _ => self.root.clone(),
        }
    }
}

/// What happened to one binding at teardown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "disposition", rename_all = "snake_case")]
pub enum BindingDisposition {
    /// The worktree was clean and has been removed.
    Removed,
    /// The worktree had uncommitted or untracked changes: retained, recorded.
    RetainedDirty {
        /// `git status --porcelain` output, as evidence.
        changes: String,
    },
    /// The worktree path was already gone: recorded, not treated as success.
    Missing,
    /// Git refused to remove it: retained, with Git's own diagnostic.
    RetainedError {
        /// Why removal failed.
        detail: String,
    },
}

/// Teardown outcome for one binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingTeardown {
    /// Repository name.
    pub repository: String,
    /// Worktree path that was torn down.
    pub worktree_path: PathBuf,
    /// Branch that was retained (always: teardown never deletes a branch).
    pub work_branch: String,
    /// The branch tip at the moment teardown ran — R-MVP1-2's output
    /// pointer's "finalize commit". Read from the *branch*
    /// (`refs/heads/<work_branch>` in the source repository), never the
    /// worktree, so it is available whether or not the worktree itself
    /// survives this teardown (`Missing`, `RetainedDirty` and
    /// `RetainedError` all still resolve it — only `git rev-parse` on a
    /// branch teardown itself never deletes can fail, and that is recorded
    /// as `None` rather than guessed). If a closing stage's `promote`
    /// disposition (R-MVP1-2) committed before teardown ran — the ruled
    /// timing, "inside the closing stage, before terminal state and
    /// therefore before teardown, while the worktree exists" — this is that
    /// commit; otherwise it is whatever the branch already pointed at.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_sha: Option<String>,
    /// What happened.
    #[serde(flatten)]
    pub disposition: BindingDisposition,
}

/// Teardown outcome for a whole surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeardownReport {
    /// Work whose surface was torn down.
    pub work_id: String,
    /// Per-binding outcomes.
    pub bindings: Vec<BindingTeardown>,
    /// Whether every worktree was removed cleanly. `false` means something
    /// was retained and is described in `bindings` — never silently dropped.
    pub clean: bool,
}

/// Failure materializing a surface.
#[derive(Debug, thiserror::Error)]
pub enum SurfaceError {
    /// Git refused.
    #[error(transparent)]
    Git(#[from] GitError),
    /// Filesystem failure preparing the surface directory.
    #[error("surface io error at {path}: {source}")]
    Io {
        /// Path being prepared.
        path: String,
        /// Underlying failure.
        source: std::io::Error,
    },
    /// The surface would live inside the source checkout, which §11 forbids:
    /// runtime state must never appear inside a repository whose checked-in
    /// files are supposed to stay declarative.
    #[error(
        "refusing to materialize a work surface at {worktree} inside source repository {source_repo}: \
         surfaces live in the daemon data dir, outside every checkout (§11)"
    )]
    InsideSourceCheckout {
        /// The offending worktree path.
        worktree: String,
        /// The repository it would live inside.
        source_repo: String,
    },
    /// No repositories were selected: a surface with no worktrees could never
    /// execute anything.
    #[error("cannot materialize a work surface with no repositories")]
    NoRepositories,
    /// A repository failed to materialize after earlier repositories in the
    /// same request already succeeded. Whatever those earlier repositories
    /// created has been torn down before this is returned, so the caller
    /// never has to reason about a worktree or branch it cannot see: `source`
    /// is what went wrong, `teardown` is what was done about the rest.
    #[error("cannot materialize work surface: {source}")]
    PartialFailure {
        /// The failure that stopped materialization.
        #[source]
        source: Box<SurfaceError>,
        /// What happened to the bindings already created for earlier
        /// repositories in this request.
        teardown: TeardownReport,
    },
}

/// Root directory for a work's surface: `<surfaces_root>/<work_id>`.
///
/// **R-MVP1-1.** `surfaces_root` is already the directory work surfaces live
/// directly under — this does not additionally join [`SURFACES_DIR`] onto
/// it. The default `<data_dir>/surfaces` is computed once, by
/// [`crate::runtime::engine::Engine`] (`SURFACES_DIR` beside `data_dir`),
/// not here — a caller handing this an already-custom `surfaces_root`
/// (an `[estate] surfaces_dir` override, `SGT_SURFACES_DIR`, or MVP-3's
/// future outside-every-checkout default) must not get an extra `surfaces/`
/// nested inside it.
pub fn surface_root(surfaces_root: &Path, work_id: &str) -> PathBuf {
    surfaces_root.join(work_id)
}

/// Materialize a work surface: one worktree per repository, each on a fresh
/// work branch cut from that repository's current HEAD (§11).
///
/// A later repository can fail after earlier ones already got a real
/// worktree and branch in the user's checkout. Rather than return with those
/// left behind and unrecorded, every binding created so far is torn down
/// (§11: teardown fails closed, so this can never silently destroy anything)
/// and the report travels with the error, so the caller can journal exactly
/// what happened instead of leaving a `sergeant/<work-id>` branch nobody
/// knows about.
///
/// **The same rule applies to the *current* repository, not only earlier
/// ones (#22).** `materialize_one` can itself fail after already creating a
/// real worktree — [`init_submodules_if_present`] runs once the worktree
/// exists, so a submodule it cannot initialize is a failure *with* a binding
/// to roll back, not before one exists. The special case below ("nothing to
/// roll back, this is the first repository") is therefore keyed on whether
/// *this* repository produced a binding before failing, never on its
/// position in the list — a submodule failure on repository 1 of 1 gets
/// exactly the same recorded teardown a later repository's failure always
/// got, instead of silently stranding the worktree `materialize_one` had
/// already created for it.
pub fn materialize(
    surfaces_root: &Path,
    work_id: &str,
    repositories: &[RepositorySpec],
) -> Result<WorkSurface, SurfaceError> {
    if repositories.is_empty() {
        return Err(SurfaceError::NoRepositories);
    }
    let root = surface_root(surfaces_root, work_id);
    create_dir_all_durable(&root).map_err(|source| SurfaceError::Io {
        path: root.display().to_string(),
        source,
    })?;
    let branch = work_branch(work_id);
    // Symlinks put the same directory on more than one path; canonicalize
    // both sides of the source-checkout check so a data dir reaching a repo
    // through a different route cannot bypass it.
    let canonical_root = std::fs::canonicalize(&root).unwrap_or_else(|_| root.clone());

    let mut bindings: Vec<RepositoryBinding> = Vec::with_capacity(repositories.len());
    for repository in repositories {
        let outcome = match materialize_one(&root, &canonical_root, &branch, repository) {
            Ok(binding) => init_submodules_if_present(&binding.worktree_path)
                .map(|()| binding.clone())
                .map_err(|err| (Some(binding), err)),
            Err(err) => Err((None, err)),
        };
        match outcome {
            Ok(binding) => bindings.push(binding),
            Err((created, err)) => {
                // A binding was created for *this* repository before the
                // failure (the submodule case) — fold it into the set being
                // rolled back so its own worktree is torn down, recorded,
                // exactly like every earlier repository's.
                bindings.extend(created);
                // Nothing to roll back if no repository — this one included
                // — ever produced a binding: surface the original error as-is.
                if bindings.is_empty() {
                    return Err(err);
                }
                let partial = WorkSurface {
                    work_id: work_id.to_string(),
                    root,
                    bindings,
                };
                let report = teardown(&partial);
                return Err(SurfaceError::PartialFailure {
                    source: Box::new(err),
                    teardown: report,
                });
            }
        }
    }
    Ok(WorkSurface {
        work_id: work_id.to_string(),
        root,
        bindings,
    })
}

fn materialize_one(
    root: &Path,
    canonical_root: &Path,
    branch: &str,
    repository: &RepositorySpec,
) -> Result<RepositoryBinding, SurfaceError> {
    let worktree_path = root.join(&repository.name);
    let canonical_repo_path =
        std::fs::canonicalize(&repository.path).unwrap_or_else(|_| repository.path.clone());
    if canonical_root
        .join(&repository.name)
        .starts_with(&canonical_repo_path)
    {
        return Err(SurfaceError::InsideSourceCheckout {
            worktree: worktree_path.display().to_string(),
            source_repo: repository.path.display().to_string(),
        });
    }
    let (base_sha, base_branch, head_sha) = with_repository(&repository.path, || {
        let base_sha = git(&repository.path, &["rev-parse", "HEAD"])?;
        // A detached HEAD has no branch name; record the fact rather than
        // inventing one.
        let base_branch = git(
            &repository.path,
            &["symbolic-ref", "--quiet", "--short", "HEAD"],
        )
        .unwrap_or_else(|_| "(detached)".to_string());

        add_worktree(&repository.path, &worktree_path, branch, &base_sha)?;
        let head_sha = git(&worktree_path, &["rev-parse", "HEAD"])?;
        Ok::<_, SurfaceError>((base_sha, base_branch, head_sha))
    })?;

    Ok(RepositoryBinding {
        repository: repository.name.clone(),
        source_path: repository.path.clone(),
        base_branch,
        base_sha,
        worktree_path,
        work_branch: branch.to_string(),
        head_sha,
    })
}

/// Re-attach worktrees to a surface's retained branches.
///
/// Retry after a terminal failure re-enters a stage, and the previous
/// teardown removed the worktrees while keeping the branches — which is
/// exactly what makes this possible: the branch *is* the durable surface, the
/// worktree is a view onto it. Bindings keep their recorded base branch and
/// base SHA (the run's provenance does not change on retry); only the current
/// HEAD is re-read.
///
/// Also re-initializes any submodule the original [`materialize`] would have
/// (#22) — a re-attached worktree needs its submodule content just as much as
/// a fresh one does. **Known asymmetry, not introduced here:** unlike
/// `materialize`'s own submodule failure (rolled back and reported through
/// `SurfaceError::PartialFailure`, same as any other binding failure), a
/// failure here propagates through `settle_rematerialize`'s pre-existing
/// `Err(e) => return Err(e.into())` arm as an engine error rather than a
/// journaled `blocked` state — true of *every* git failure on this path
/// already (an `add_worktree_from` failure hits the identical arm), not a gap
/// this change opens. Left alone rather than folded into this pass: the fix
/// belongs to `settle_rematerialize`'s error handling in general, not to
/// submodules specifically.
pub fn rematerialize(surface: &WorkSurface) -> Result<WorkSurface, SurfaceError> {
    create_dir_all_durable(&surface.root).map_err(|source| SurfaceError::Io {
        path: surface.root.display().to_string(),
        source,
    })?;
    let mut bindings = Vec::with_capacity(surface.bindings.len());
    for binding in &surface.bindings {
        if !binding.worktree_path.exists() {
            with_repository(&binding.source_path, || {
                // Whatever removed the directory may not have unregistered it
                // — teardown only prunes on the paths it walks, and a worktree
                // can vanish long after that (a retained-dirty one deleted by
                // hand, a wiped `surfaces/` tree). A stale registration makes
                // every `git worktree add` at that path fail forever, which
                // would shut §12's one door back out of failed/blocked/waiting.
                prune_stale_worktrees(&binding.source_path);
                // The branch was retained by teardown; check it out again there.
                let branch_exists = git_succeeds(
                    &binding.source_path,
                    &[
                        "show-ref",
                        "--verify",
                        "--quiet",
                        &format!("refs/heads/{}", binding.work_branch),
                    ],
                );
                let start = if branch_exists {
                    binding.work_branch.clone()
                } else {
                    binding.base_sha.clone()
                };
                add_worktree_from(
                    &binding.source_path,
                    &binding.worktree_path,
                    &binding.work_branch,
                    &start,
                    !branch_exists,
                )?;
                init_submodules_if_present(&binding.worktree_path)
            })?;
        }
        let head_sha = git(&binding.worktree_path, &["rev-parse", "HEAD"])?;
        bindings.push(RepositoryBinding {
            head_sha,
            ..binding.clone()
        });
    }
    Ok(WorkSurface {
        bindings,
        ..surface.clone()
    })
}

/// Tear down a surface: remove clean worktrees, retain every branch, and
/// record everything that could not be removed (fail closed).
///
/// This never returns an error. Teardown runs on the way to a terminal state,
/// and a Work does not stop being canceled because a worktree was dirty — the
/// honest outcome is a recorded report, which is what the caller journals.
pub fn teardown(surface: &WorkSurface) -> TeardownReport {
    let mut bindings = Vec::with_capacity(surface.bindings.len());
    for binding in &surface.bindings {
        let (disposition, final_sha) = teardown_binding(binding);
        bindings.push(BindingTeardown {
            repository: binding.repository.clone(),
            worktree_path: binding.worktree_path.clone(),
            work_branch: binding.work_branch.clone(),
            final_sha,
            disposition,
        });
    }
    // The worktrees live one level *below* the surface root, so removing them
    // leaves the root itself behind. It is scaffolding this module created
    // (`materialize`), and nothing removed it: measured at P1-PERF as one
    // empty `surfaces/<work-id>/` per work, in every scenario, never reclaimed.
    remove_surface_root(&surface.root);
    let clean = bindings
        .iter()
        .all(|b| b.disposition == BindingDisposition::Removed);
    TeardownReport {
        work_id: surface.work_id.clone(),
        bindings,
        clean,
    }
}

/// Remove the per-work surface root, but only once nothing is left inside it.
///
/// `remove_dir` *is* the whole guard, and deliberately so: it refuses a
/// directory that still holds anything, which is exactly teardown's
/// fail-closed rule expressed by the syscall. A binding retained dirty, a
/// removal git refused, a multi-repo surface with one worktree still standing
/// — each leaves the root in place with its contents intact, and only the last
/// worktree's departure empties it. Sergeant never recursively deletes a
/// surface directory.
///
/// Best-effort and idempotent by construction: an already-removed root, a root
/// that never existed, and a root someone else is holding open all leave the
/// report unchanged, so re-running teardown after a crash between this call
/// and the `surface.torn_down` append converges instead of erroring (L6).
/// The parent (`surfaces/`) is fsynced so the removal survives the crash that
/// L6 window is about — the mirror of `create_dir_all_durable`'s dirent sync
/// on the way in.
fn remove_surface_root(root: &Path) {
    if std::fs::remove_dir(root).is_ok()
        && let Some(parent) = root.parent()
        && let Ok(dir) = std::fs::File::open(parent)
    {
        let _ = dir.sync_all();
    }
}

/// Drop registrations for worktrees whose directories no longer exist.
///
/// A vanished worktree directory leaves the source repository still listing
/// it administratively, and git then refuses every `worktree add` at that
/// path — "is a missing but already registered worktree" — with no product
/// path that recovers. `git worktree prune` is git's own answer, and it only
/// ever removes records for directories that are already gone, so it can
/// never destroy anything a run still has. Best-effort: a repository that
/// refuses to prune fails at the operation that needed it, with git's own
/// diagnostic, rather than here.
fn prune_stale_worktrees(source: &Path) {
    let _ = git(source, &["worktree", "prune"]);
}

fn teardown_binding(binding: &RepositoryBinding) -> (BindingDisposition, Option<String>) {
    with_repository(&binding.source_path, || teardown_binding_locked(binding))
}

fn teardown_binding_locked(binding: &RepositoryBinding) -> (BindingDisposition, Option<String>) {
    // Read the retained branch's tip before anything else. Teardown always
    // keeps the branch regardless of what happens to the worktree below, so
    // this is available in every disposition — including `Missing`, where
    // there is no worktree left to read a HEAD from at all.
    let final_sha = git(
        &binding.source_path,
        &["rev-parse", &format!("refs/heads/{}", binding.work_branch)],
    )
    .ok();
    if !binding.worktree_path.exists() {
        // The directory is gone, but the source repository still lists it;
        // unregister it now so a later rematerialize at the same path is
        // possible. This does not change the disposition below either way.
        prune_stale_worktrees(&binding.source_path);
        return (BindingDisposition::Missing, final_sha);
    }
    match git(&binding.worktree_path, &["status", "--porcelain"]) {
        Ok(changes) if !changes.is_empty() => {
            return (BindingDisposition::RetainedDirty { changes }, final_sha);
        }
        Ok(_) => {}
        Err(e) => {
            // Cannot establish that it is clean ⇒ must not remove it.
            return (
                BindingDisposition::RetainedError {
                    detail: e.to_string(),
                },
                final_sha,
            );
        }
    }
    let path = binding.worktree_path.display().to_string();
    let disposition = match git(&binding.source_path, &["worktree", "remove", &path]) {
        Ok(_) => BindingDisposition::Removed,
        // #22: git unconditionally refuses to remove a worktree that
        // contains a submodule — "working trees containing submodules
        // cannot be moved or removed" — regardless of whether anything in
        // it is actually dirty. Left as `RetainedError` unconditionally,
        // *every* submodule-bearing surface would leak its worktree on
        // every single ordinary, successful teardown, forever (measured:
        // this module's own `a_submodule_is_populated_into_the_
        // materialized_worktree` failed here before this arm existed). The
        // `git status --porcelain` check just above already answers
        // whether the submodule itself has anything uncommitted — it
        // recurses into a registered submodule by default and would have
        // returned `RetainedDirty` already if so — so reaching this arm at
        // all means the only reason git refused is the policy, not the
        // content, and retrying with `--force` destroys nothing §11 does
        // not already know is safe to remove.
        Err(e) if e.to_string().contains("containing submodules") => {
            match git(
                &binding.source_path,
                &["worktree", "remove", "--force", &path],
            ) {
                Ok(_) => BindingDisposition::Removed,
                Err(e) => BindingDisposition::RetainedError {
                    detail: e.to_string(),
                },
            }
        }
        Err(e) => BindingDisposition::RetainedError {
            detail: e.to_string(),
        },
    };
    (disposition, final_sha)
}

fn add_worktree(
    source: &Path,
    worktree: &Path,
    branch: &str,
    start: &str,
) -> Result<(), SurfaceError> {
    add_worktree_from(source, worktree, branch, start, true)
}

fn add_worktree_from(
    source: &Path,
    worktree: &Path,
    branch: &str,
    start: &str,
    create_branch: bool,
) -> Result<(), SurfaceError> {
    if let Some(parent) = worktree.parent() {
        create_dir_all_durable(parent).map_err(|source| SurfaceError::Io {
            path: parent.display().to_string(),
            source,
        })?;
    }
    let path = worktree.display().to_string();
    let args: Vec<&str> = if create_branch {
        vec!["worktree", "add", "-b", branch, &path, start]
    } else {
        vec!["worktree", "add", &path, start]
    };
    git(source, &args)?;
    Ok(())
}

/// #22: `git worktree add` checks out the superproject's gitlinks but never
/// populates a submodule's own content — that is `git submodule update`'s
/// job, and nothing calls it on the worktree-add path. Left alone, a
/// repository with submodules materializes a surface with silently empty
/// submodule directories: not a refusal, not a warning, just content the
/// rest of the run expected and does not have.
///
/// Scoped to worktrees that actually declare submodules (`.gitmodules`
/// present) so the ordinary no-submodule repository — the overwhelming
/// majority — pays no extra `git` invocation. A submodule that genuinely
/// cannot be initialized (an unreachable URL, a disallowed transport) fails
/// this closed: the `GitError` propagates as a `SurfaceError::Git` exactly
/// like any other materialization failure, which `materialize`'s existing
/// partial-failure path already tears down and reports rather than leaving
/// a stranded, half-populated surface.
fn init_submodules_if_present(worktree_path: &Path) -> Result<(), SurfaceError> {
    if worktree_path.join(".gitmodules").is_file() {
        git_submodule_update(worktree_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Concurrent surfaces against **one** repository all materialize and all
    /// tear down cleanly.
    ///
    /// N3 moved git out from under the daemon's core lock (§22.6), which is
    /// what the budget asks for and which promptly let a burst run
    /// `git worktree add`/`remove` against the same `.git` from many threads.
    /// Git does not serialize that: a measured burst-50 run left two surfaces
    /// retained with `fatal: failed to read .git/worktrees/<other>/commondir`
    /// — one process walking the registry while another rewrote it. Failing
    /// closed made it honest, not harmless: those worktrees stayed on disk.
    ///
    /// The guard is a per-repository lock inside this module, which is not the
    /// core lock and blocks no request. This is the regression test for it:
    /// without the serialization it fails as a flake, which is exactly how the
    /// bug presented.
    ///
    /// **Why it races the same repository more than once.** One burst of
    /// [`WORKS`] threads reproduces the bug about 17 times in 18 with the lock
    /// removed (measured, round-2 finding N3R2-06) — a real guard, and still a
    /// ~6% chance per run that a reverted fix goes unnoticed. That is L7's
    /// corollary ("single-run green is not a gate") landing on one specific
    /// test, and the cheap answer is to make one run *be* several: with
    /// [`ROUNDS`] independent bursts the escape probability is 0.06^5, about
    /// one run in 1.3 million, for ~2.5 s of wall clock. Anyone reverting the
    /// per-repository lock now sees red, not a coin flip.
    #[test]
    fn concurrent_surfaces_on_one_repository_all_materialize_and_retire_cleanly() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = dir.path().join("data");
        let spec = repo(&dir.path().join("solo"));
        // Enough overlap that adds and removes interleave inside git's own
        // `.git/worktrees` registry, which is where the measured failure was.
        const WORKS: usize = 24;
        // Independent races against the same `.git`, so a single lucky
        // interleaving cannot pass the test on its own.
        const ROUNDS: usize = 5;

        for round in 0..ROUNDS {
            let reports: Vec<TeardownReport> = std::thread::scope(|scope| {
                let handles: Vec<_> = (0..WORKS)
                    .map(|i| {
                        let data = data.clone();
                        let spec = spec.clone();
                        scope.spawn(move || {
                            let work_id = format!("01CONCURRENT{round}{i:03}");
                            let surface = materialize(&data, &work_id, std::slice::from_ref(&spec))
                                .unwrap_or_else(|e| panic!("materialize {work_id}: {e}"));
                            teardown(&surface)
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|h| h.join().expect("surface thread"))
                    .collect()
            });
            assert_eq!(reports.len(), WORKS);
            for report in &reports {
                assert!(
                    report.clean,
                    "a concurrent teardown must not retain a worktree it could have \
                     removed (round {round} of {ROUNDS}): {report:?}"
                );
            }
            // `data` is handed to `materialize` directly as the surfaces
            // root (R-MVP1-1: no implicit `SURFACES_DIR` nesting inside this
            // module any more — that join happens once, at `Engine`'s
            // default computation, not here).
            assert!(
                !data.exists()
                    || std::fs::read_dir(&data)
                        .expect("surfaces root")
                        .next()
                        .is_none(),
                "no surface root survives a clean teardown (round {round} of {ROUNDS})"
            );
        }
    }

    /// Run one git command in `dir` with a fixture identity, same shape as
    /// [`repo`]'s own commits — for tests that need to commit *inside* an
    /// already-materialized worktree, where the crate's `git()` wrapper alone
    /// would hit "please tell me who you are" on a host with no global
    /// identity configured.
    fn git_as_test_identity(dir: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .expect("git");
        assert!(output.status.success(), "git {args:?}: {output:?}");
    }

    /// A temp repository with one commit.
    fn repo(path: &Path) -> RepositorySpec {
        std::fs::create_dir_all(path).expect("repo dir");
        for args in [
            vec!["init", "-b", "main"],
            vec!["add", "."],
            vec!["commit", "--allow-empty", "-m", "initial"],
        ] {
            let output = Command::new("git")
                .args(&args)
                .current_dir(path)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_SYSTEM", "/dev/null")
                .output()
                .expect("git");
            assert!(output.status.success(), "git {args:?}: {output:?}");
        }
        RepositorySpec {
            name: "solo".to_string(),
            path: path.to_path_buf(),
        }
    }

    /// §11: "runtime work surfaces live outside the source checkout". A data
    /// dir configured *inside* a repository would put a worktree in the
    /// checkout whose files are supposed to stay declarative — refused, not
    /// quietly allowed.
    #[test]
    fn a_surface_inside_the_source_checkout_is_refused() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let source = dir.path().join("solo");
        let spec = repo(&source);
        let data_dir_inside = source.join(".sergeant-data");

        let err = materialize(&data_dir_inside, "01WORKID", std::slice::from_ref(&spec))
            .expect_err("must refuse");
        assert!(
            matches!(err, SurfaceError::InsideSourceCheckout { .. }),
            "expected a refusal, got {err}"
        );
        // And nothing was created behind the refusal.
        assert!(
            !source.join(".sergeant-data/01WORKID/solo").exists(),
            "a refused surface must leave no worktree"
        );

        // The same repository outside the checkout materializes fine.
        let data = tempfile::TempDir::new().expect("tempdir");
        let surface = materialize(data.path(), "01WORKID", std::slice::from_ref(&spec))
            .expect("materialize outside the checkout");
        assert_eq!(surface.bindings.len(), 1);
        // Single-repo work executes in the worktree itself, not the root.
        assert_eq!(surface.execution_cwd(), surface.bindings[0].worktree_path);
    }

    /// A worktree that has vanished under us is recorded as missing rather
    /// than reported as a clean removal — teardown never claims to have
    /// removed something it did not find. And having recorded it, teardown
    /// leaves the repository in a state a retry can actually use: the stale
    /// registration is gone, so the surface can be rebuilt at the same path.
    #[test]
    fn a_vanished_worktree_is_recorded_and_leaves_the_path_reusable() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface =
            materialize(data.path(), "01GONE", std::slice::from_ref(&spec)).expect("materialize");

        let worktree = surface.bindings[0].worktree_path.display().to_string();
        std::fs::remove_dir_all(&surface.bindings[0].worktree_path).expect("remove worktree");
        assert!(
            git(&spec.path, &["worktree", "list"])
                .expect("list")
                .contains(&worktree),
            "git still lists the vanished worktree until something unregisters it"
        );

        let report = teardown(&surface);
        assert!(!report.clean, "a missing worktree is not a clean teardown");
        assert_eq!(report.bindings[0].disposition, BindingDisposition::Missing);
        assert_eq!(report.bindings[0].work_branch, work_branch("01GONE"));

        // Recording it is not enough: the registration git still holds would
        // make every later `worktree add` at that path fail. Teardown leaves
        // the repository in a state a retry can actually use.
        assert!(
            !git(&spec.path, &["worktree", "list"])
                .expect("list")
                .contains(&worktree),
            "teardown must unregister the worktree it recorded as missing"
        );
        let rebuilt = rematerialize(&surface)
            .expect("a recorded-missing worktree must not wedge the path forever");
        assert!(rebuilt.bindings[0].worktree_path.is_dir());
    }

    /// §12 makes retry the one door back out of failed, blocked and waiting,
    /// and retry re-attaches the worktrees teardown removed. A worktree that
    /// teardown *retained* (dirty, so fail-closed left it alone) and that
    /// was then deleted out of band is the case nothing else prunes: git
    /// still has it registered, and without a prune every `worktree add` at
    /// that path fails for good, wedging retry permanently.
    #[test]
    fn a_worktree_deleted_after_a_dirty_teardown_can_still_be_rematerialized() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface =
            materialize(data.path(), "01STALE", std::slice::from_ref(&spec)).expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        // Uncommitted work: teardown retains it, and therefore never prunes.
        std::fs::write(worktree.join("half-done.rs"), "fn main() {}\n").expect("dirty");
        let report = teardown(&surface);
        assert!(matches!(
            report.bindings[0].disposition,
            BindingDisposition::RetainedDirty { .. }
        ));

        // ...and then it is deleted by something that is not sergeant.
        std::fs::remove_dir_all(&worktree).expect("delete out of band");

        let rebuilt = rematerialize(&surface).expect("retry must still be able to rebuild");
        assert!(rebuilt.bindings[0].worktree_path.is_dir());
        assert_eq!(
            git(&worktree, &["rev-parse", "--abbrev-ref", "HEAD"]).expect("head"),
            work_branch("01STALE"),
            "the rebuilt worktree is back on the retained branch"
        );
        // And it is repeatable: a second rebuild after a second deletion
        // works too, so nothing accumulates that eventually wedges it.
        std::fs::remove_dir_all(&worktree).expect("delete again");
        rematerialize(&surface).expect("still rebuildable");
    }

    /// A later repository failing must not leave the earlier ones' branches
    /// and worktrees in the user's checkouts unrecorded: what was created is
    /// torn down, and the report travels with the error.
    #[test]
    fn a_later_repository_failing_rolls_back_the_earlier_ones() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let mut first = repo(&dir.path().join("first"));
        first.name = "first".to_string();
        let mut second = repo(&dir.path().join("second"));
        second.name = "second".to_string();

        // The second repository already has the branch this work would cut,
        // so its `git worktree add -b` fails — after the first succeeded.
        let branch = work_branch("01PARTIAL");
        git(&second.path, &["branch", &branch]).expect("pre-create the colliding branch");

        let err = materialize(data.path(), "01PARTIAL", &[first.clone(), second])
            .expect_err("the second repository must fail");
        let SurfaceError::PartialFailure { source, teardown } = err else {
            panic!("expected a partial failure, got {err}");
        };
        assert!(
            source.to_string().contains(&branch),
            "the original git diagnostic must survive: {source}"
        );

        // Everything the request did create is torn down and named.
        assert_eq!(teardown.bindings.len(), 1, "only the first got that far");
        assert_eq!(teardown.bindings[0].repository, "first");
        assert_eq!(
            teardown.bindings[0].disposition,
            BindingDisposition::Removed
        );
        assert!(teardown.clean);
        assert!(
            !teardown.bindings[0].worktree_path.exists(),
            "the rolled-back worktree must be gone"
        );
        // The branch is retained, as teardown always retains branches — but
        // the report names it, so it is recorded rather than a mystery.
        assert_eq!(teardown.bindings[0].work_branch, branch);
        assert!(
            git_succeeds(
                &first.path,
                &[
                    "show-ref",
                    "--verify",
                    "--quiet",
                    &format!("refs/heads/{branch}")
                ]
            ),
            "teardown retains branches by contract"
        );
    }

    /// §11's "outside every checkout" is a statement about directories, not
    /// about how a path happens to be spelled. A data dir reached through a
    /// symlink into the repository is the same directory, and must be
    /// refused the same way.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_path_cannot_smuggle_a_surface_into_the_checkout() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let source = dir.path().join("solo");
        let mut spec = repo(&source);
        spec.path = std::fs::canonicalize(&source).expect("canonical repo");

        // A second route to the very same directory.
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&source, &link).expect("symlink");
        let data_dir_via_link = link.join(".sergeant-data");

        let err = materialize(&data_dir_via_link, "01LINK", std::slice::from_ref(&spec))
            .expect_err("the symlinked route is still inside the checkout");
        assert!(
            matches!(err, SurfaceError::InsideSourceCheckout { .. }),
            "expected a refusal, got {err}"
        );
        assert!(
            !source.join(".sergeant-data/01LINK/solo").exists(),
            "a refused surface must leave no worktree"
        );
    }

    /// Teardown removes the scaffolding it created — the per-work surface
    /// root included — and removes it only once it is genuinely empty.
    ///
    /// The regression: `git worktree remove` deletes the worktree one level
    /// *below* the root, and nothing ever removed the root, so every work
    /// left an empty `surfaces/<work-id>/` behind for the life of the data
    /// dir (measured in all seven P1-PERF scenarios). Minor per instance,
    /// unbounded in aggregate.
    ///
    /// The three cases together are the rule: empty ⇒ gone, still-occupied ⇒
    /// kept (fail closed, never a recursive delete), already-gone ⇒ no error.
    #[test]
    fn teardown_removes_the_surface_root_once_it_is_empty() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let mut first = repo(&dir.path().join("first"));
        first.name = "first".to_string();
        let mut second = repo(&dir.path().join("second"));
        second.name = "second".to_string();

        // One repository: the root is empty after the worktree goes.
        let solo = materialize(data.path(), "01ROOT", std::slice::from_ref(&first))
            .expect("materialize solo");
        let root = solo.root.clone();
        assert!(root.is_dir(), "materialize created the root");
        let report = teardown(&solo);
        assert!(report.clean);
        assert!(
            !root.exists(),
            "an emptied surface root must not outlive the work: {}",
            root.display()
        );
        // `data.path()` is handed to `materialize` directly as the surfaces
        // root (R-MVP1-1): only the per-work child goes, the surfaces root
        // itself — whatever the caller passed in — stays.
        assert!(
            data.path().is_dir(),
            "only the per-work root goes; the surfaces root passed in stays"
        );
        // …and tearing the same surface down again is a no-op, not an error:
        // the crash window between the removal and the `surface.torn_down`
        // append is re-run, not repaired (L6).
        let again = teardown(&solo);
        assert_eq!(again.bindings[0].disposition, BindingDisposition::Missing);
        assert!(!root.exists());

        // Two repositories: the root survives until the last worktree is
        // gone. Tearing down a surface that names only the first binding
        // leaves the second worktree — and therefore the root — untouched.
        let pair = materialize(data.path(), "01PAIR", &[first, second]).expect("materialize pair");
        let pair_root = pair.root.clone();
        let partial = WorkSurface {
            bindings: pair.bindings[..1].to_vec(),
            ..pair.clone()
        };
        teardown(&partial);
        assert!(
            pair_root.is_dir(),
            "a root still holding another repository's worktree must be kept"
        );
        assert!(pair.bindings[1].worktree_path.is_dir(), "untouched");
        teardown(&pair);
        assert!(
            !pair_root.exists(),
            "the last binding's removal takes the root with it"
        );

        // A retained worktree keeps its root: teardown fails closed, and the
        // root is where the retained thing lives.
        let dirty_spec = repo(&dir.path().join("dirty"));
        let dirty = materialize(data.path(), "01DIRTY", std::slice::from_ref(&dirty_spec))
            .expect("materialize dirty");
        std::fs::write(dirty.bindings[0].worktree_path.join("wip.rs"), "//\n").expect("dirty");
        let report = teardown(&dirty);
        assert!(matches!(
            report.bindings[0].disposition,
            BindingDisposition::RetainedDirty { .. }
        ));
        assert!(
            dirty.root.is_dir() && dirty.bindings[0].worktree_path.is_dir(),
            "uncommitted work is never removed, and neither is the root holding it"
        );
    }

    /// R-MVP1-2's output-pointer sibling: teardown records each binding's
    /// finalize commit — the retained branch's tip — so `work show` can name
    /// it without decoding the journal. A commit made in the worktree *before*
    /// teardown runs (the closing stage's `promote` disposition, per the
    /// ruled timing) is exactly what this must capture: the branch tip is
    /// read before the worktree is touched, so a fresh commit on the branch
    /// is the finalize commit, not the SHA the surface was cut from.
    #[test]
    fn teardown_captures_the_retained_branchs_tip_as_the_finalize_commit() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(data.path(), "01FINALIZE", std::slice::from_ref(&spec))
            .expect("materialize");
        let worktree = &surface.bindings[0].worktree_path;
        let base_sha = surface.bindings[0].base_sha.clone();

        // A closing stage's finalize helper commits before teardown runs.
        std::fs::write(worktree.join("output.txt"), "promoted\n").expect("write output");
        git_as_test_identity(worktree, &["add", "."]);
        git_as_test_identity(worktree, &["commit", "-m", "finalize"]);
        let finalize_commit = git(worktree, &["rev-parse", "HEAD"]).expect("finalize commit");
        assert_ne!(
            finalize_commit, base_sha,
            "the fixture must actually advance the branch"
        );

        let report = teardown(&surface);
        assert_eq!(report.bindings[0].disposition, BindingDisposition::Removed);
        assert_eq!(
            report.bindings[0].final_sha.as_deref(),
            Some(finalize_commit.as_str()),
            "teardown must record the branch tip as it stood at teardown time, \
             including a finalize commit made before it ran"
        );

        // A worktree the repository never had (Missing) still resolves the
        // branch tip: it lives on the branch, not the worktree.
        let again = materialize(data.path(), "01FINALIZE2", std::slice::from_ref(&spec))
            .expect("materialize again");
        std::fs::remove_dir_all(&again.bindings[0].worktree_path).expect("simulate vanished");
        let missing_report = teardown(&again);
        assert_eq!(
            missing_report.bindings[0].disposition,
            BindingDisposition::Missing
        );
        assert!(
            missing_report.bindings[0].final_sha.is_some(),
            "a vanished worktree still has a branch tip to report"
        );
    }

    /// A surface with no repositories could never execute anything.
    #[test]
    fn a_surface_needs_at_least_one_repository() {
        let data = tempfile::TempDir::new().expect("tempdir");
        assert!(matches!(
            materialize(data.path(), "01EMPTY", &[]),
            Err(SurfaceError::NoRepositories)
        ));
    }

    // ---------------------------------------------------------- #22: submodules

    /// Declares `inner` as a submodule of `superproject` at `path`, without
    /// going through `git submodule add` (which needs the same transport
    /// permission `materialize` itself now grants at update time — exercising
    /// that here would just retest `git`, not this module). The gitlink and
    /// `.gitmodules` are exactly what a real `submodule add` would have left,
    /// committed on top of `superproject`'s existing history.
    fn declare_submodule(superproject: &Path, inner: &Path, path: &str) {
        let inner_head = git(inner, &["rev-parse", "HEAD"]).expect("inner HEAD");
        std::fs::write(
            superproject.join(".gitmodules"),
            format!(
                "[submodule \"{path}\"]\n\tpath = {path}\n\turl = {}\n",
                inner.display()
            ),
        )
        .expect("write .gitmodules");
        std::fs::create_dir_all(superproject.join(path)).expect("submodule placeholder dir");
        git_as_test_identity(superproject, &["add", ".gitmodules"]);
        git_as_test_identity(
            superproject,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                "160000",
                &inner_head,
                path,
            ],
        );
        git_as_test_identity(superproject, &["commit", "-m", "declare a submodule"]);
    }

    /// #22: a repository with a submodule materializes with the submodule's
    /// own content actually checked out — not the silent empty directory
    /// `git worktree add` alone would leave (its own doc comment on
    /// [`init_submodules_if_present`] has the measurement). Real teardown
    /// afterward, same as any ordinary surface: worktree removed, branch
    /// retained.
    #[test]
    fn a_submodule_is_populated_into_the_materialized_worktree() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let inner = repo(&dir.path().join("inner"));
        std::fs::write(
            dir.path().join("inner").join("payload.txt"),
            "inner content\n",
        )
        .expect("write payload");
        git_as_test_identity(&inner.path, &["add", "payload.txt"]);
        git_as_test_identity(&inner.path, &["commit", "-m", "payload"]);

        let outer = repo(&dir.path().join("outer"));
        declare_submodule(&outer.path, &inner.path, "vendored");

        let surface = materialize(data.path(), "01SUBMODULE", std::slice::from_ref(&outer))
            .expect("materialize a repository with a submodule");
        let worktree = &surface.bindings[0].worktree_path;
        assert_eq!(
            std::fs::read_to_string(worktree.join("vendored").join("payload.txt"))
                .expect("submodule content must be checked out, not an empty directory"),
            "inner content\n"
        );
        // Git's own view agrees the submodule is initialized (no leading `-`,
        // which is how `git submodule status` marks "not initialized").
        let status = git(worktree, &["submodule", "status"]).expect("submodule status");
        assert!(
            !status.trim_start().starts_with('-'),
            "the submodule must report initialized: {status:?}"
        );

        let report = teardown(&surface);
        assert!(
            report.clean,
            "an untouched submodule worktree is clean: {report:?}"
        );
        assert!(!worktree.exists());
        assert!(
            git_succeeds(
                &outer.path,
                &[
                    "show-ref",
                    "--verify",
                    "--quiet",
                    &format!("refs/heads/{}", surface.bindings[0].work_branch)
                ]
            ),
            "teardown retains the branch by contract"
        );
    }

    /// #22's own contract for a shape that legitimately cannot work: a
    /// submodule declared over a transport `materialize` does not allow (the
    /// same allowlist [`crate::runtime::git::git_clone`] already uses for a
    /// `sgt repo add --origin`) fails **closed**, not silently. And — the
    /// regression this pins — failing on the *first and only* repository must
    /// still roll back and report exactly like a later repository's failure
    /// always has: before this fix, `materialize`'s "nothing to roll back,
    /// this is the first repository" special case fired unconditionally on
    /// position, stranding the worktree `add_worktree` had already created
    /// with no teardown, no report, nothing journaled.
    #[test]
    fn a_disallowed_submodule_transport_fails_closed_and_is_not_stranded() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let inner = repo(&dir.path().join("inner"));

        let outer = repo(&dir.path().join("outer"));
        // `ext::` is refused by `materialize`'s allowlist (`file:http:https:
        // ssh:git`) the same way it would be refused for `sgt repo add
        // --origin ext::…` — deterministic and instant, no network involved.
        let inner_head = git(&inner.path, &["rev-parse", "HEAD"]).expect("inner HEAD");
        std::fs::write(
            outer.path.join(".gitmodules"),
            "[submodule \"vendored\"]\n\tpath = vendored\n\turl = ext::false\n",
        )
        .expect(".gitmodules");
        std::fs::create_dir_all(outer.path.join("vendored")).expect("placeholder");
        git_as_test_identity(&outer.path, &["add", ".gitmodules"]);
        git_as_test_identity(
            &outer.path,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                "160000",
                &inner_head,
                "vendored",
            ],
        );
        git_as_test_identity(&outer.path, &["commit", "-m", "an unreachable submodule"]);

        let err = materialize(data.path(), "01BADSUBMODULE", std::slice::from_ref(&outer))
            .expect_err("a disallowed submodule transport must refuse materialization");
        let SurfaceError::PartialFailure { source, teardown } = err else {
            panic!(
                "expected the same rolled-back-and-reported shape a later repository's \
                 failure gets, got a bare {err} — the worktree add_worktree already created \
                 would be left with no report at all"
            );
        };
        assert!(
            source.to_string().contains("ext"),
            "git's own transport diagnostic must survive: {source}"
        );
        // The worktree materialize_one already created for this (only, first)
        // repository is named in the report — not a mystery — and actually
        // removed: a submodule that never finished initializing has nothing
        // of its own for `git status --porcelain` to find dirty (there is no
        // checked-out submodule content yet to be dirty), so the force-retry
        // `teardown_binding_locked` uses for git's blanket "containing
        // submodules" refusal cleans it up rather than leaving it parked
        // with a permanent `RetainedError`.
        assert_eq!(teardown.bindings.len(), 1);
        assert_eq!(teardown.bindings[0].repository, "solo");
        assert_eq!(
            teardown.bindings[0].disposition,
            BindingDisposition::Removed,
            "a submodule that never checked anything out has nothing to retain: {:?}",
            teardown.bindings[0].disposition
        );
        assert!(
            !teardown.bindings[0].worktree_path.exists(),
            "the rolled-back worktree must be gone, not stranded on disk"
        );
        assert!(teardown.clean);
    }

    /// The force-retry `teardown_binding_locked` uses for git's blanket
    /// "containing submodules" refusal must never bypass real dirtiness —
    /// uncommitted content genuinely *inside* the submodule is exactly what
    /// §11's fail-closed rule exists to protect. `git status --porcelain`
    /// already recurses into a registered submodule by default (measured,
    /// not assumed — an untracked file, a modified tracked file, and an
    /// advanced submodule `HEAD` are all reported as `M sub` at the
    /// superproject level), so `RetainedDirty` fires *before* the removal
    /// attempt this test would otherwise reach — proving the force-retry
    /// path is unreachable for this case, not merely untested.
    #[test]
    fn a_dirty_submodule_still_blocks_teardown_despite_the_force_retry() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let inner = repo(&dir.path().join("inner"));
        let outer = repo(&dir.path().join("outer"));
        declare_submodule(&outer.path, &inner.path, "vendored");

        let surface = materialize(
            data.path(),
            "01DIRTYSUBMODULE",
            std::slice::from_ref(&outer),
        )
        .expect("materialize a repository with a submodule");
        let worktree = &surface.bindings[0].worktree_path;
        // Untracked content left inside the submodule by (in spirit) an
        // actor's run — never committed, so it must survive teardown.
        std::fs::write(
            worktree.join("vendored").join("actors-work.txt"),
            "not committed\n",
        )
        .expect("simulate uncommitted submodule content");

        let report = teardown(&surface);
        assert!(
            !report.clean,
            "uncommitted content inside a submodule must block teardown: {report:?}"
        );
        assert!(
            matches!(
                &report.bindings[0].disposition,
                BindingDisposition::RetainedDirty { changes } if changes.contains("vendored")
            ),
            "must be retained as dirty (not silently force-removed): {:?}",
            report.bindings[0].disposition
        );
        assert!(
            worktree.exists() && worktree.join("vendored").join("actors-work.txt").is_file(),
            "the uncommitted file must still be there — nothing destroyed it"
        );
    }
}

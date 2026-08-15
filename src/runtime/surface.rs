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

/// How a binding's worktree came to exist on `work_branch`.
///
/// Every binding before this variant existed was implicitly `Cut` — the
/// only shape `materialize` could ever produce — so this defaults to `Cut`
/// on deserialize (`#[serde(default)]` on `RepositoryBinding::origin`) and
/// every journaled binding from before this field existed reads back
/// exactly the fact it recorded.
///
/// `base_branch`/`base_sha` mean something different depending on which
/// variant they sit beside. For `Cut` they are what *this* branch was cut
/// from — the only meaning they have ever had. For `Attached` (§8.6
/// investigation, Mechanism A) they are carried over unchanged from the
/// *target*'s own binding: what the branch this binding attached to was
/// itself cut from. Reusing them to mean "attached to `target_work_id`'s
/// branch" instead would blur a provenance distinction the journal
/// otherwise makes cleanly (the investigation's own cost note) — `origin`
/// is what carries that fact instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BindingOrigin {
    /// `materialize` cut a fresh `sergeant/<work-id>` branch from
    /// `base_branch`/`base_sha`. The only shape that existed before
    /// `attach`.
    #[default]
    Cut,
    /// `attach` checked this binding out onto a branch a different,
    /// already-terminal Work materialized — never minting a branch of its
    /// own.
    Attached {
        /// The Work whose branch this binding attached to.
        target_work_id: String,
    },
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
    /// How this binding came to be on `work_branch`: cut fresh, or attached
    /// to another Work's branch. `#[serde(default)]` so a binding journaled
    /// before this field existed — every one of them a `Cut`, the only
    /// shape that could exist then — deserializes as exactly that.
    #[serde(default)]
    pub origin: BindingOrigin,
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

/// Where #109's captured dirty-state patch was written, and how large it
/// is — enough for an operator (or `sgt work reap`) to reason about what is
/// actually on disk without re-reading it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchInfo {
    /// The patch file's path, under the surface root (sibling to the
    /// worktree it replaced — `remove_surface_root`'s own fail-closed
    /// `remove_dir` is what keeps the root alive once this is the only
    /// thing left inside it).
    pub path: PathBuf,
    /// The patch file's size in bytes.
    pub bytes: u64,
}

/// What happened to one binding at teardown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "disposition", rename_all = "snake_case")]
pub enum BindingDisposition {
    /// The worktree was clean and has been removed.
    Removed,
    /// The worktree had uncommitted or untracked changes: the dirty state
    /// is retained, never the directory it happened to live in (#109, owner
    /// ruling R4: *"The journal is the real artifact. Let's keep the disk
    /// clean."*).
    RetainedDirty {
        /// `git status --porcelain` output, as evidence.
        changes: String,
        /// Where the dirty state was captured as a patch, once the worktree
        /// directory itself was reclaimed (see [`retain_dirty`]). `None`
        /// means the capture could not be trusted — a submodule-bearing
        /// worktree, or a git failure partway through — so teardown fell
        /// back to retaining the whole directory, exactly as it did before
        /// #109. `#[serde(default)]` so every binding journaled before this
        /// field existed deserializes as that same fallback, which is
        /// honest: those really did retain the whole directory.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        patch: Option<PatchInfo>,
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
        origin: BindingOrigin::Cut,
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

/// Materialize a gate Work's surface by *attaching* to another, already-
/// terminal Work's branches instead of minting fresh ones (§8.6
/// investigation, Mechanism A —
/// `docs/gauntlet/runs/foundation-1/8.6-gate-branch-binding.md`).
///
/// Every attached binding's `work_branch` is `target_bindings`'s own — this
/// *is* `target_work_id`'s real branch, not a copy, which is what lets
/// `axi sync --recover`'s fast-forward-in-place custody model (§8.6
/// investigation §4) keep working unmodified: a fix commit made in the
/// resulting worktree lands on the branch that will actually ship.
///
/// **Callers must have already established the precondition this mechanism
/// depends on** — the target is terminal and its surface's teardown
/// reported every binding `Removed` (`engine::branch_takeover_precondition`
/// computes exactly that from journaled state and returns the bindings this
/// function wants as `target_bindings`). This function does not re-derive
/// that precondition; it only re-verifies what git itself can check at the
/// moment of the attempt — an ordinary, non-forced `git worktree add`
/// refuses cleanly if a worktree is (still, or again) attached to the
/// branch, if the branch no longer exists, or if the target path collides
/// with something already there. Those refusals surface as this function's
/// own `Err`, not a panic or a forced takeover.
///
/// Same partial-failure discipline as [`materialize`]: a later repository's
/// attach failing after earlier ones already produced a real binding rolls
/// every binding *this call* created back through [`teardown`], so a failed
/// gate-surface attach never leaves an orphaned worktree in the caller's
/// repositories — same as an ordinary `materialize` failure, and using the
/// identical [`SurfaceError::PartialFailure`] shape.
pub fn attach(
    surfaces_root: &Path,
    work_id: &str,
    target_work_id: &str,
    target_bindings: &[RepositoryBinding],
) -> Result<WorkSurface, SurfaceError> {
    if target_bindings.is_empty() {
        return Err(SurfaceError::NoRepositories);
    }
    let root = surface_root(surfaces_root, work_id);
    create_dir_all_durable(&root).map_err(|source| SurfaceError::Io {
        path: root.display().to_string(),
        source,
    })?;
    // Same symlink-smuggling guard `materialize` applies, for the same
    // reason: §11's "outside every checkout" is about directories, not
    // about how a path happens to be spelled.
    let canonical_root = std::fs::canonicalize(&root).unwrap_or_else(|_| root.clone());

    let mut bindings: Vec<RepositoryBinding> = Vec::with_capacity(target_bindings.len());
    for target in target_bindings {
        let outcome = match attach_one(&root, &canonical_root, target_work_id, target) {
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
                // rolled back so its worktree, checked out onto the target's
                // real branch, is torn down exactly like every earlier
                // repository's, mirroring `materialize`'s own handling.
                bindings.extend(created);
                // Nothing to roll back if no repository ever produced a
                // binding: surface the original error as-is, exactly like
                // `materialize`'s own first-repository case.
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

fn attach_one(
    root: &Path,
    canonical_root: &Path,
    target_work_id: &str,
    target: &RepositoryBinding,
) -> Result<RepositoryBinding, SurfaceError> {
    let worktree_path = root.join(&target.repository);
    let canonical_repo_path =
        std::fs::canonicalize(&target.source_path).unwrap_or_else(|_| target.source_path.clone());
    if canonical_root
        .join(&target.repository)
        .starts_with(&canonical_repo_path)
    {
        return Err(SurfaceError::InsideSourceCheckout {
            worktree: worktree_path.display().to_string(),
            source_repo: target.source_path.display().to_string(),
        });
    }
    let head_sha = with_repository(&target.source_path, || -> Result<String, SurfaceError> {
        // `create_branch: false`: the git-level operation `rematerialize`
        // already performs in a different context (re-attaching a surface's
        // own retained branch), invoked here from a new caller with a
        // branch this surface did not mint. Git itself enforces the
        // exclusivity this depends on — see the doc comment on [`attach`].
        //
        // Deliberately *not* `rematerialize`'s own "branch missing, fall
        // back to base_sha" leniency (verified by reverting to it and
        // watching `attach_refuses_when_the_target_branch_is_missing` fail):
        // that fallback is safe for a `Cut` binding re-attaching to its own
        // branch, but a missing branch here means the *target*'s real
        // branch is gone — silently substituting its base commit would
        // materialize a gate surface that reviews stale, wrong content
        // instead of refusing.
        add_worktree_from(
            &target.source_path,
            &worktree_path,
            &target.work_branch,
            &target.work_branch,
            false,
        )?;
        Ok(git(&worktree_path, &["rev-parse", "HEAD"])?)
    })?;

    Ok(RepositoryBinding {
        repository: target.repository.clone(),
        source_path: target.source_path.clone(),
        base_branch: target.base_branch.clone(),
        base_sha: target.base_sha.clone(),
        worktree_path,
        work_branch: target.work_branch.clone(),
        head_sha,
        origin: BindingOrigin::Attached {
            target_work_id: target_work_id.to_string(),
        },
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

/// One binding a Work's teardown left something on disk for — #109's
/// inspect verb (`sgt work retained`): what is retained, why, and how
/// large. Never includes `Removed`/`Missing` bindings, since there is
/// nothing there for an operator to act on.
#[derive(Debug, Clone, Serialize)]
pub struct RetainedBinding {
    /// Repository name.
    pub repository: String,
    /// Where the retained content actually lives: the patch file when
    /// [`retain_dirty`] captured one *and* removed the worktree, otherwise
    /// the worktree directory itself (the submodule/capture-failure
    /// fallback, a captured patch whose worktree removal then failed, or a
    /// `RetainedError`).
    pub path: PathBuf,
    /// `retained_dirty` or `retained_error` — the same bare-tag spelling
    /// `sgt work show`'s output pointer already uses, so a client never has
    /// to reconcile two vocabularies for the same fact.
    pub reason: &'static str,
    /// The evidence teardown recorded: `git status --porcelain` output for
    /// `retained_dirty`, Git's own diagnostic for `retained_error`.
    pub detail: String,
    /// Size in bytes: exact for a captured patch, best-effort
    /// ([`directory_size`]) for a still-retained directory.
    pub bytes: u64,
}

/// #109's inspect verb, the pure decode: every binding in `teardown` that
/// still holds disk past a clean removal. A live filesystem read
/// ([`directory_size`]) for the directory-fallback case, not a size cached
/// at teardown time — so this always reports what is *actually* on disk
/// right now, including after a partial `sgt work reap` or an out-of-band
/// change, rather than trusting a number that can go stale the moment
/// anything else touches the path.
pub fn retained_bindings(teardown: &TeardownReport) -> Vec<RetainedBinding> {
    teardown
        .bindings
        .iter()
        .filter_map(|b| match &b.disposition {
            BindingDisposition::RetainedDirty {
                changes,
                patch: Some(info),
            } => {
                // Live check, not the size journaled at teardown time: a
                // `sgt work reap` since then (or anything else that removed
                // the file) must not keep showing up here — the same
                // "report what is actually on disk right now" rule
                // `directory_size`'s callers below already follow.
                if !info.path.is_file() {
                    return None;
                }
                Some(RetainedBinding {
                    repository: b.repository.clone(),
                    path: info.path.clone(),
                    reason: "retained_dirty",
                    detail: changes.clone(),
                    bytes: info.bytes,
                })
            }
            BindingDisposition::RetainedDirty {
                changes,
                patch: None,
            } => {
                if !b.worktree_path.exists() {
                    return None;
                }
                Some(RetainedBinding {
                    repository: b.repository.clone(),
                    path: b.worktree_path.clone(),
                    reason: "retained_dirty",
                    detail: changes.clone(),
                    bytes: directory_size(&b.worktree_path),
                })
            }
            BindingDisposition::RetainedError { detail } => {
                if !b.worktree_path.exists() {
                    return None;
                }
                Some(RetainedBinding {
                    repository: b.repository.clone(),
                    path: b.worktree_path.clone(),
                    reason: "retained_error",
                    detail: detail.clone(),
                    bytes: directory_size(&b.worktree_path),
                })
            }
            BindingDisposition::Removed | BindingDisposition::Missing => None,
        })
        .collect()
}

/// What happened when [`reap`] tried to dispose of one binding's retained
/// state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ReapOutcome {
    /// The retained state was permanently discarded.
    Reaped {
        /// What was freed.
        bytes: u64,
    },
    /// Nothing was retained for this binding (`Removed`/`Missing`, or an
    /// earlier reap already cleared it) — a no-op, not a failure.
    NothingRetained,
    /// Left alone, on purpose: `RetainedError` means teardown could not
    /// even establish that this worktree was clean (`git status` itself
    /// failed), so there is no evidence a forced removal here would be
    /// discarding only what teardown already knows about, rather than real
    /// uncommitted work nothing ever captured. Ambiguity fails closed
    /// (`AGENTS.md`) — resolve the underlying git error and let teardown
    /// re-run instead.
    Skipped {
        /// Why this binding was left alone.
        reason: String,
    },
    /// The reap attempt itself failed (git or the filesystem refused).
    Failed {
        /// The underlying diagnostic.
        detail: String,
    },
}

/// One binding's [`reap`] outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingReap {
    /// Repository name.
    pub repository: String,
    /// What happened.
    pub outcome: ReapOutcome,
}

/// [`reap`]'s outcome for a whole Work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReapReport {
    /// Work whose retained state was reaped.
    pub work_id: String,
    /// Per-binding outcomes.
    pub bindings: Vec<BindingReap>,
}

/// #109's dispose verb: permanently discard whatever `RetainedDirty`
/// bindings still hold for this Work — the captured patch, or, in the
/// submodule/capture-failure fallback (or a captured patch whose worktree
/// removal then failed), the worktree directory itself.
///
/// A deliberate, explicit, human-invoked action, never run on its own:
/// `AGENTS.md`'s guardrail puts preserved state (a retained branch, a
/// journal, a Work record) outside anything standing authorization may
/// destroy, and this is how a human destroys the *dirty-state* half of it
/// on purpose, once they have decided they no longer need it (typically
/// after reading it via [`retained_bindings`]). It never touches the
/// retained *branch* — teardown's branch-retention guarantee is
/// unconditional (§11) and this function has no path that reaches it.
///
/// Scoped to `RetainedDirty` only, matching [`ReapOutcome::Skipped`]'s
/// reasoning: a `RetainedError` binding has no evidence backing a forced
/// removal, so it is reported, not touched.
pub fn reap(surface: &WorkSurface, teardown: &TeardownReport) -> ReapReport {
    let bindings = teardown
        .bindings
        .iter()
        .map(|b| BindingReap {
            repository: b.repository.clone(),
            outcome: reap_binding(surface, b),
        })
        .collect();
    remove_surface_root(&surface.root);
    ReapReport {
        work_id: teardown.work_id.clone(),
        bindings,
    }
}

fn reap_binding(surface: &WorkSurface, binding: &BindingTeardown) -> ReapOutcome {
    match &binding.disposition {
        BindingDisposition::RetainedDirty {
            patch: Some(info), ..
        } => match std::fs::remove_file(&info.path) {
            Ok(()) => ReapOutcome::Reaped { bytes: info.bytes },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => ReapOutcome::NothingRetained,
            Err(e) => ReapOutcome::Failed {
                detail: e.to_string(),
            },
        },
        BindingDisposition::RetainedDirty { patch: None, .. } => {
            if !binding.worktree_path.exists() {
                return ReapOutcome::NothingRetained;
            }
            // The directory is still a real, registered git worktree (that
            // is exactly why it was never removed at teardown) — going
            // through git rather than a raw recursive delete keeps the
            // source repository's own worktree registry consistent, the
            // same as every other removal this module ever does.
            let Some(source) = surface
                .bindings
                .iter()
                .find(|b| b.repository == binding.repository)
                .map(|b| b.source_path.clone())
            else {
                return ReapOutcome::Failed {
                    detail: "no surface binding recorded for this repository; cannot resolve \
                             which source repository's worktree registry to update"
                        .to_string(),
                };
            };
            let bytes = directory_size(&binding.worktree_path);
            let path = binding.worktree_path.display().to_string();
            match with_repository(&source, || {
                git(&source, &["worktree", "remove", "--force", &path])
            }) {
                Ok(_) => ReapOutcome::Reaped { bytes },
                Err(e) => ReapOutcome::Failed {
                    detail: e.to_string(),
                },
            }
        }
        BindingDisposition::RetainedError { detail } => ReapOutcome::Skipped {
            reason: format!(
                "teardown could not establish this worktree was clean ({detail}); resolve the \
                 underlying git error and retry teardown instead of reaping"
            ),
        },
        BindingDisposition::Removed | BindingDisposition::Missing => ReapOutcome::NothingRetained,
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

/// #109 (R4): retention preserves the dirty *state*, never the whole
/// directory it happened to live in — `target/` and every other gitignored
/// artifact must never be in scope (owner ruling, plan v2 R4/R12: *"The
/// journal is the real artifact. Let's keep the disk clean."*). Measured
/// motivation: three `retained_dirty` surfaces held 30 GB, essentially all
/// of it gitignored `target/` (#109).
///
/// `git add -A` (respects `.gitignore`, so `target/` is never staged) plus
/// `git diff --cached --binary` captures every tracked modification *and*
/// every untracked-but-wanted file in one patch — the same trick that
/// answers the issue's hard case, an untracked file worth keeping beside
/// untracked build output that is not. A `git bundle` was considered and
/// rejected: its advantage is self-contained branch history, and teardown
/// already keeps that for free by never deleting the branch (§11) — a
/// bundle would only duplicate it. A `git stash` was rejected too: it lives
/// in the *source* repository's own reflog, subject to that repository's
/// own housekeeping, rather than travelling with the surface the way a file
/// under the surface root does.
///
/// Fails closed onto the pre-#109 behavior — the whole worktree directory
/// left in place, nothing removed — whenever the capture cannot be fully
/// trusted: a worktree that declares submodules (a nested repository's own
/// uncommitted content never appears in the parent's `git add -A`, only its
/// commit pointer does, so a patch here could silently under-capture it),
/// or any git/io failure along the way. Only once the patch is durably on
/// disk does this remove the worktree, and only with `--force` — safe here
/// specifically because everything `git status --porcelain` just reported
/// has already been captured.
fn retain_dirty(binding: &RepositoryBinding, changes: String) -> BindingDisposition {
    if binding.worktree_path.join(".gitmodules").exists() {
        return BindingDisposition::RetainedDirty {
            changes,
            patch: None,
        };
    }
    let Some(patch) = capture_dirty_patch(binding) else {
        return BindingDisposition::RetainedDirty {
            changes,
            patch: None,
        };
    };
    let path = binding.worktree_path.display().to_string();
    // The patch is already durable regardless of what happens next, so
    // nothing captured can be lost. But if the worktree removal below fails,
    // the directory is still on disk — reporting `patch: Some` in that case
    // would make it invisible to `retained_bindings`/`reap_binding`, which
    // only look for the worktree directory when `patch` is `None`. Falling
    // back to the same `patch: None` shape the submodule/capture-failure
    // paths above already use keeps that directory discoverable and
    // reclaimable (`reap_binding`'s `patch: None` arm retries the same `git
    // worktree remove`); the now-redundant patch file (its content already
    // sitting uncommitted in the still-present directory) is removed
    // best-effort so it does not become its own untracked leak.
    match git(
        &binding.source_path,
        &["worktree", "remove", "--force", &path],
    ) {
        Ok(_) => BindingDisposition::RetainedDirty {
            changes,
            patch: Some(patch),
        },
        Err(_) => {
            let _ = std::fs::remove_file(&patch.path);
            BindingDisposition::RetainedDirty {
                changes,
                patch: None,
            }
        }
    }
}

/// Stage everything `.gitignore` does not exclude (`git add -A`), diff it
/// against `HEAD` (`git diff --cached --binary`), and write the result
/// durably under the surface root, sibling to the worktree it is about to
/// replace. `None` on any failure — a caller that gets `None` back must not
/// remove anything, since nothing was actually captured.
fn capture_dirty_patch(binding: &RepositoryBinding) -> Option<PatchInfo> {
    git(&binding.worktree_path, &["add", "-A"]).ok()?;
    let diff = git(&binding.worktree_path, &["diff", "--cached", "--binary"]).ok()?;
    // Defensive: the caller already confirmed `git status --porcelain` was
    // non-empty, so an empty diff here means something about the capture
    // did not actually see what made the worktree dirty — fail closed
    // rather than reclaim a directory whose content was never written down.
    if diff.is_empty() {
        return None;
    }
    let path = binding
        .worktree_path
        .parent()?
        .join(format!("{}.dirty.patch", binding.repository));
    crate::runtime::fsutil::write_atomic(&path, diff.as_bytes()).ok()?;
    Some(PatchInfo {
        path,
        bytes: diff.len() as u64,
    })
}

/// Best-effort recursive size of everything under `path`, in bytes — disk-
/// usage evidence for an operator (`sgt work retained`), not an accounting
/// system. Errors partway (permissions, a vanished entry mid-walk) are
/// swallowed: a size that is merely a lower bound because of one unreadable
/// subtree is more useful than refusing to answer at all.
pub fn directory_size(path: &Path) -> u64 {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if !metadata.is_dir() {
        return metadata.len();
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return metadata.len();
    };
    entries
        .flatten()
        .map(|entry| directory_size(&entry.path()))
        .sum()
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
            return (retain_dirty(binding, changes), final_sha);
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
    /// teardown *retained whole* (fail-closed left it alone) and that was
    /// then deleted out of band is the case nothing else prunes: git still
    /// has it registered, and without a prune every `worktree add` at that
    /// path fails for good, wedging retry permanently.
    ///
    /// #109 narrows how often the whole directory is what is retained — an
    /// ordinary dirty worktree now gets its state captured as a patch and
    /// the directory itself reclaimed via a real `git worktree remove`,
    /// which never leaves this staleness behind in the first place. The
    /// submodule fallback is the case that still can: `retain_dirty` leaves
    /// the directory (and its git registration) untouched, so this test
    /// exercises that path to keep covering the recovery this docstring
    /// describes.
    #[test]
    fn a_worktree_deleted_after_a_dirty_teardown_can_still_be_rematerialized() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let inner = repo(&dir.path().join("inner"));
        let outer = repo(&dir.path().join("outer"));
        declare_submodule(&outer.path, &inner.path, "vendored");

        let surface = materialize(data.path(), "01STALE", std::slice::from_ref(&outer))
            .expect("materialize a repository with a submodule");
        let worktree = surface.bindings[0].worktree_path.clone();

        // Uncommitted work: teardown retains it whole, and therefore never
        // prunes.
        std::fs::write(worktree.join("half-done.rs"), "fn main() {}\n").expect("dirty");
        let report = teardown(&surface);
        assert!(matches!(
            report.bindings[0].disposition,
            BindingDisposition::RetainedDirty { patch: None, .. }
        ));
        assert!(
            worktree.exists(),
            "the submodule fallback keeps the whole directory"
        );

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
        // root is where the retained *state* lives. #109: not the whole
        // directory — that gets reclaimed once its dirty state is captured
        // as a patch, which is exactly what keeps the root non-empty.
        let dirty_spec = repo(&dir.path().join("dirty"));
        let dirty = materialize(data.path(), "01DIRTY", std::slice::from_ref(&dirty_spec))
            .expect("materialize dirty");
        std::fs::write(dirty.bindings[0].worktree_path.join("wip.rs"), "//\n").expect("dirty");
        let report = teardown(&dirty);
        let BindingDisposition::RetainedDirty { patch, .. } = &report.bindings[0].disposition
        else {
            panic!(
                "expected RetainedDirty: {:?}",
                report.bindings[0].disposition
            );
        };
        let patch = patch
            .as_ref()
            .expect("an ordinary (non-submodule) dirty worktree must capture a patch");
        assert!(
            dirty.root.is_dir() && patch.path.is_file(),
            "the dirty state is retained as a patch under the root, not the whole directory"
        );
        assert!(
            !dirty.bindings[0].worktree_path.exists(),
            "#109: the reclaimed worktree directory itself must be gone"
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
                BindingDisposition::RetainedDirty { changes, .. } if changes.contains("vendored")
            ),
            "must be retained as dirty (not silently force-removed): {:?}",
            report.bindings[0].disposition
        );
        // #109: a submodule-bearing worktree is the fallback case teardown
        // still retains the *whole directory* for, because `git add -A` in
        // the superproject cannot see the submodule's own uncommitted
        // content — only its commit pointer. `patch` staying `None` is what
        // proves the capture path was skipped rather than silently
        // under-capturing it.
        let BindingDisposition::RetainedDirty { patch, .. } = &report.bindings[0].disposition
        else {
            panic!(
                "expected RetainedDirty: {:?}",
                report.bindings[0].disposition
            );
        };
        assert!(
            patch.is_none(),
            "a submodule-bearing worktree must fall back to whole-directory retention, \
             not a patch that cannot see inside the submodule: {patch:?}"
        );
        assert!(
            worktree.exists() && worktree.join("vendored").join("actors-work.txt").is_file(),
            "the uncommitted file must still be there — nothing destroyed it"
        );
    }

    // -------------------------------------------- §8.6 Mechanism A: `attach`

    /// The whole point of Mechanism A, proven end to end rather than assumed:
    /// once a target's surface has torn down clean, `attach` checks its
    /// branch out into a *second* Work's surface, and a commit made there —
    /// standing in for a no-mistakes auto-fix — lands on the exact same
    /// branch ref the target left behind. This is the property Mechanism B
    /// (reviewing a copy) was rejected for lacking: `axi sync --recover`
    /// fast-forwards whatever is checked out in the worktree it runs
    /// against, so the branch a fix actually reaches must *be* the real
    /// branch, not a lookalike.
    #[test]
    fn attach_checks_out_the_targets_real_branch_and_a_fix_commit_lands_on_it() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));

        let target = materialize(data.path(), "01TARGET", std::slice::from_ref(&spec))
            .expect("materialize target");
        let target_branch = target.bindings[0].work_branch.clone();
        assert_eq!(target_branch, work_branch("01TARGET"));

        // The target commits its own work before finishing.
        std::fs::write(
            target.bindings[0].worktree_path.join("feature.rs"),
            "fn feature() {}\n",
        )
        .expect("write feature");
        git_as_test_identity(&target.bindings[0].worktree_path, &["add", "."]);
        git_as_test_identity(
            &target.bindings[0].worktree_path,
            &["commit", "-m", "feature"],
        );
        let pre_gate_tip =
            git(&target.bindings[0].worktree_path, &["rev-parse", "HEAD"]).expect("target head");

        let target_report = teardown(&target);
        assert!(
            target_report.clean,
            "the fixture must reach the clean-teardown precondition: {target_report:?}"
        );

        let gate = attach(data.path(), "01GATE", "01TARGET", &target.bindings)
            .expect("attach must succeed once the target has torn down clean");
        assert_eq!(gate.bindings.len(), 1);
        assert_eq!(gate.bindings[0].work_branch, target_branch);
        assert_eq!(
            gate.bindings[0].origin,
            BindingOrigin::Attached {
                target_work_id: "01TARGET".to_string()
            }
        );
        assert_eq!(gate.bindings[0].head_sha, pre_gate_tip);
        // Provenance is carried over from the target's own binding, not
        // reinvented as of gate-dispatch time.
        assert_eq!(gate.bindings[0].base_branch, target.bindings[0].base_branch);
        assert_eq!(gate.bindings[0].base_sha, target.bindings[0].base_sha);

        // The gate Work commits a fix, standing in for a no-mistakes
        // auto-fix round.
        std::fs::write(
            gate.bindings[0].worktree_path.join("fix.rs"),
            "fn fix() {}\n",
        )
        .expect("write fix");
        git_as_test_identity(&gate.bindings[0].worktree_path, &["add", "."]);
        git_as_test_identity(&gate.bindings[0].worktree_path, &["commit", "-m", "fix"]);
        let fix_commit = git(&gate.bindings[0].worktree_path, &["rev-parse", "HEAD"])
            .expect("gate head after fix");

        // The real branch — read from the source repository, independent of
        // either worktree — has the fix. Not a copy that never moved it.
        let branch_tip = git(
            &spec.path,
            &["rev-parse", &format!("refs/heads/{target_branch}")],
        )
        .expect("branch tip in source repo");
        assert_eq!(
            branch_tip, fix_commit,
            "a commit made in the attached worktree must land on the target's real branch"
        );

        let gate_report = teardown(&gate);
        assert!(gate_report.clean, "gate teardown: {gate_report:?}");
        // Teardown always retains the branch — including one it only ever
        // attached to, never minted.
        assert!(
            git_succeeds(
                &spec.path,
                &[
                    "show-ref",
                    "--verify",
                    "--quiet",
                    &format!("refs/heads/{target_branch}")
                ]
            ),
            "the branch survives the gate surface's own teardown"
        );
    }

    /// Git's own exclusivity check (verified directly in the §8.6
    /// investigation) is the mechanism Mechanism A's ordering constraint
    /// depends on: `attach` must fail, not force, when the target's
    /// worktree is still attached to the branch — i.e., when the caller
    /// dispatched a gate Work without actually waiting for the target's
    /// teardown to report clean.
    #[test]
    fn attach_refuses_when_the_target_worktree_is_still_attached() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));

        let target = materialize(data.path(), "01LIVE", std::slice::from_ref(&spec))
            .expect("materialize target");
        // No teardown: the target's worktree is still live on its branch.

        let err = attach(data.path(), "01GATE", "01LIVE", &target.bindings)
            .expect_err("attach must refuse while the target's worktree still holds the branch");
        assert!(
            matches!(err, SurfaceError::Git(_)),
            "expected git's own exclusivity refusal, got {err}"
        );
        assert!(
            !data.path().join("01GATE").join("solo").exists(),
            "a refused attach must leave no worktree behind"
        );

        // Reverting the fix (attempting the takeover before teardown) is
        // exactly the race Mechanism A's precondition exists to prevent —
        // pin it the other way too: once the target *does* tear down clean,
        // the identical call succeeds.
        let report = teardown(&target);
        assert!(report.clean);
        attach(data.path(), "01GATE2", "01LIVE", &target.bindings)
            .expect("attach must succeed once the target's worktree is actually gone");
    }

    /// A branch that no longer exists — deleted out of band after a clean
    /// teardown, since teardown itself never deletes a branch — is a git
    /// refusal `attach` must surface as an error, not a panic or a silent
    /// fresh branch.
    #[test]
    fn attach_refuses_when_the_target_branch_is_missing() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));

        let target = materialize(data.path(), "01GONEBRANCH", std::slice::from_ref(&spec))
            .expect("materialize target");
        let report = teardown(&target);
        assert!(report.clean);

        let branch = target.bindings[0].work_branch.clone();
        git(&spec.path, &["branch", "-D", &branch]).expect("delete the branch out of band");

        let err = attach(data.path(), "01GATE", "01GONEBRANCH", &target.bindings)
            .expect_err("attach must refuse when the branch no longer exists");
        assert!(
            matches!(err, SurfaceError::Git(_)),
            "expected a git refusal naming the missing branch, got {err}"
        );
        assert!(!data.path().join("01GATE").join("solo").exists());
    }

    /// A worktree path collision — something already at the exact path
    /// `attach` would create — is git's own refusal to add a worktree onto a
    /// non-empty directory. `attach` must surface it, not overwrite or
    /// destroy whatever is already there.
    #[test]
    fn attach_refuses_on_worktree_path_collision() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));

        let target = materialize(data.path(), "01COLLIDE", std::slice::from_ref(&spec))
            .expect("materialize target");
        let report = teardown(&target);
        assert!(report.clean);

        let colliding_path = data.path().join("01GATE").join("solo");
        std::fs::create_dir_all(&colliding_path).expect("pre-create the colliding directory");
        std::fs::write(colliding_path.join("occupied.txt"), "already here\n")
            .expect("occupy the path");

        let err = attach(data.path(), "01GATE", "01COLLIDE", &target.bindings)
            .expect_err("attach must refuse a worktree path collision");
        assert!(
            matches!(err, SurfaceError::Git(_)),
            "expected git's own refusal to add a worktree onto a non-empty path, got {err}"
        );
        assert!(
            colliding_path.join("occupied.txt").is_file(),
            "the pre-existing content must survive a refused attach"
        );
    }

    /// Same partial-failure discipline as `materialize`'s own
    /// `a_later_repository_failing_rolls_back_the_earlier_ones`: a second
    /// repository's attach failing after the first already succeeded must
    /// roll the first back through `teardown` and report it, never leave it
    /// stranded in the caller's checkout.
    #[test]
    fn a_later_repositorys_failed_attach_rolls_back_the_earlier_one() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let mut first = repo(&dir.path().join("first"));
        first.name = "first".to_string();
        let mut second = repo(&dir.path().join("second"));
        second.name = "second".to_string();

        let target = materialize(
            data.path(),
            "01PARTIALATTACH",
            &[first.clone(), second.clone()],
        )
        .expect("materialize target");
        let report = teardown(&target);
        assert!(report.clean);

        // The second repository's branch is gone by the time the gate
        // attaches — attach_one for "second" must fail after "first" already
        // succeeded.
        let branch = work_branch("01PARTIALATTACH");
        git(&second.path, &["branch", "-D", &branch]).expect("delete second's branch");

        let err = attach(data.path(), "01GATE", "01PARTIALATTACH", &target.bindings)
            .expect_err("the second repository's attach must fail");
        let SurfaceError::PartialFailure {
            teardown: rollback, ..
        } = err
        else {
            panic!("expected a partial failure with a rollback report");
        };
        assert_eq!(rollback.bindings.len(), 1, "only the first got that far");
        assert_eq!(rollback.bindings[0].repository, "first");
        assert_eq!(
            rollback.bindings[0].disposition,
            BindingDisposition::Removed
        );
        assert!(rollback.clean);
        assert!(
            !data.path().join("01GATE").join("first").exists(),
            "the rolled-back gate worktree must be gone"
        );
        // And the target's own branches are untouched by the failed attach —
        // attach never deletes anything it did not itself create.
        assert!(git_succeeds(
            &first.path,
            &[
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{branch}")
            ]
        ));
    }

    /// Mirrors `materialize`'s own
    /// `a_disallowed_submodule_transport_fails_closed_and_is_not_stranded`,
    /// but pins the regression this fix closes in `attach`'s own code path:
    /// before it, `init_submodules_if_present` ran *inside* `attach_one`, so
    /// a failure there returned `attach_one`'s bare `Err` without the binding
    /// the worktree add had already produced — `attach`'s "nothing to roll
    /// back, this is the first repository" case fired unconditionally on
    /// position (this is the only repository) and stranded the worktree with
    /// no teardown, no report, nothing journaled.
    #[test]
    fn attach_disallowed_submodule_transport_fails_closed_and_is_not_stranded() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let inner = repo(&dir.path().join("inner"));
        let spec = repo(&dir.path().join("solo"));

        let target = materialize(data.path(), "01SUBTARGET", std::slice::from_ref(&spec))
            .expect("materialize target");
        let worktree = target.bindings[0].worktree_path.clone();

        // Declare a submodule over a transport `init_submodules_if_present`'s
        // allowlist refuses (`ext::`, the same allowlist `git_clone` uses),
        // committed onto the target's own work branch so `attach` inherits
        // it when it checks that branch out fresh into the gate surface.
        let inner_head = git(&inner.path, &["rev-parse", "HEAD"]).expect("inner HEAD");
        std::fs::write(
            worktree.join(".gitmodules"),
            "[submodule \"vendored\"]\n\tpath = vendored\n\turl = ext::false\n",
        )
        .expect(".gitmodules");
        std::fs::create_dir_all(worktree.join("vendored")).expect("placeholder");
        git_as_test_identity(&worktree, &["add", ".gitmodules"]);
        git_as_test_identity(
            &worktree,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                "160000",
                &inner_head,
                "vendored",
            ],
        );
        git_as_test_identity(&worktree, &["commit", "-m", "an unreachable submodule"]);

        let target_report = teardown(&target);
        assert!(
            target_report.clean,
            "the fixture must reach the clean-teardown precondition: {target_report:?}"
        );

        let err = attach(data.path(), "01GATE", "01SUBTARGET", &target.bindings)
            .expect_err("a disallowed submodule transport must refuse the takeover");
        let SurfaceError::PartialFailure {
            source,
            teardown: rollback,
        } = err
        else {
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
        assert_eq!(rollback.bindings.len(), 1);
        assert_eq!(rollback.bindings[0].repository, "solo");
        assert_eq!(
            rollback.bindings[0].disposition,
            BindingDisposition::Removed,
            "a submodule that never checked anything out has nothing to retain: {:?}",
            rollback.bindings[0].disposition
        );
        assert!(
            !data.path().join("01GATE").join("solo").exists(),
            "the worktree add_worktree created before the submodule failure must not survive"
        );
        assert!(rollback.clean);
    }

    /// `attach` needs at least one target binding — the same rule
    /// `materialize` enforces for repositories, for the same reason: a
    /// surface with no worktrees could never execute anything.
    #[test]
    fn attach_needs_at_least_one_target_binding() {
        let data = tempfile::TempDir::new().expect("tempdir");
        assert!(matches!(
            attach(data.path(), "01EMPTY", "01TARGET", &[]),
            Err(SurfaceError::NoRepositories)
        ));
    }

    // ------------------------------------------------------------- #109

    /// The retention-scope clause itself, pinned end to end: a dirty
    /// worktree with a genuinely wanted untracked file *and* gitignored
    /// build output produces a patch that captures the former and excludes
    /// the latter, and the (potentially huge) directory is gone afterward.
    /// Reverting `retain_dirty`/`capture_dirty_patch` back to "leave the
    /// whole directory" fails the `!worktree.exists()` assertion; reverting
    /// to "stage everything, ignoring .gitignore" would fail the
    /// `!patch_text.contains("compiled output")` assertion instead.
    #[test]
    fn teardown_captures_an_untracked_wanted_file_and_excludes_gitignored_build_output() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface =
            materialize(data.path(), "01SCOPE", std::slice::from_ref(&spec)).expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        std::fs::write(worktree.join(".gitignore"), "target/\n").expect(".gitignore");
        git_as_test_identity(&worktree, &["add", ".gitignore"]);
        git_as_test_identity(&worktree, &["commit", "-m", "ignore target/"]);

        // A file worth keeping, never `git add`ed — exactly #109's "hard
        // case".
        std::fs::write(worktree.join("new-module.rs"), "fn wanted() {}\n").expect("untracked");
        // Gitignored build output, large in spirit — #109's whole point is
        // that this must never end up retained.
        std::fs::create_dir_all(worktree.join("target")).expect("target dir");
        std::fs::write(worktree.join("target").join("big.bin"), "compiled output")
            .expect("build artifact");

        let report = teardown(&surface);
        let BindingDisposition::RetainedDirty { patch, .. } = &report.bindings[0].disposition
        else {
            panic!(
                "expected RetainedDirty: {:?}",
                report.bindings[0].disposition
            );
        };
        let patch = patch.as_ref().expect("must capture a patch");
        assert!(
            !worktree.exists(),
            "the directory — target/ included — must be gone, not just the tracked bits"
        );

        let patch_text = std::fs::read_to_string(&patch.path).expect("read captured patch");
        assert!(
            patch_text.contains("new-module.rs") && patch_text.contains("fn wanted()"),
            "the untracked-but-wanted file must be captured: {patch_text}"
        );
        assert!(
            !patch_text.contains("target/big.bin") && !patch_text.contains("compiled output"),
            "gitignored build output must never enter the patch: {patch_text}"
        );
        assert_eq!(
            patch.bytes,
            patch_text.len() as u64,
            "the recorded size must match what was actually written"
        );
    }

    /// no-mistakes review fix: if `retain_dirty` captures the patch but the
    /// `git worktree remove --force` that follows fails, the worktree
    /// directory is still on disk — reporting `patch: Some` in that case (the
    /// pre-fix behavior) would make the directory invisible to
    /// [`retained_bindings`]/`reap_binding`, which only look for it when
    /// `patch` is `None`, leaking it permanently. `git worktree lock` makes
    /// the removal fail deterministically, without needing root or a
    /// filesystem-specific immutable bit.
    #[test]
    fn a_dirty_worktree_whose_removal_fails_falls_back_to_the_directory_not_a_leaked_patch() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface =
            materialize(data.path(), "01LOCKED", std::slice::from_ref(&spec)).expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        std::fs::write(worktree.join("half-done.rs"), "fn main() {}\n").expect("dirty");
        // A single `--force` (what `retain_dirty` actually passes) refuses on
        // a locked worktree — "use 'remove -f -f' to override" — so this
        // reliably fails the removal after the patch capture already
        // succeeded, without touching filesystem permissions.
        git(
            &spec.path,
            &["worktree", "lock", &worktree.display().to_string()],
        )
        .expect("lock the worktree");

        let report = teardown(&surface);
        let BindingDisposition::RetainedDirty { patch, .. } = &report.bindings[0].disposition
        else {
            panic!(
                "expected RetainedDirty: {:?}",
                report.bindings[0].disposition
            );
        };
        assert!(
            patch.is_none(),
            "a failed removal must fall back to patch: None, not report a patch over a \
             directory that never actually went away: {patch:?}"
        );
        assert!(
            worktree.exists(),
            "the directory git refused to remove must still be there"
        );

        // The fix's whole point: `retained_bindings` must still find it, via
        // the directory rather than a patch file that (best-effort) no
        // longer exists.
        let retained = retained_bindings(&report);
        let entry = retained
            .iter()
            .find(|r| r.repository == "solo")
            .unwrap_or_else(|| {
                panic!("the locked binding must still be discoverable: {retained:?}")
            });
        assert_eq!(
            entry.path, worktree,
            "must point at the surviving directory"
        );

        // Cleanup: unlock so the tempdir's own teardown can remove it.
        let _ = git(
            &spec.path,
            &["worktree", "unlock", &worktree.display().to_string()],
        );
    }

    /// [`retained_bindings`] (`sgt work retained`'s inspect verb): a
    /// captured patch reports its exact known size; a `RetainedError`
    /// binding — no patch, since nothing was ever captured for it — is
    /// still surfaced, with a live directory-size measurement rather than a
    /// stale or absent one.
    #[test]
    fn retained_bindings_reports_a_captured_patch_and_a_retained_error_directory() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(data.path(), "01INSPECT", std::slice::from_ref(&spec))
            .expect("materialize");
        std::fs::write(surface.bindings[0].worktree_path.join("wip.rs"), "//\n").expect("dirty");
        let report = teardown(&surface);

        let error_binding = BindingTeardown {
            repository: "other".to_string(),
            worktree_path: dir.path().join("some-other-worktree"),
            work_branch: "sergeant/01INSPECT".to_string(),
            final_sha: None,
            disposition: BindingDisposition::RetainedError {
                detail: "fatal: could not read status".to_string(),
            },
        };
        std::fs::create_dir_all(&error_binding.worktree_path).expect("stand-in directory");
        std::fs::write(error_binding.worktree_path.join("evidence.txt"), "12345").expect("file");

        let mut combined = report.clone();
        combined.bindings.push(error_binding);

        let retained = retained_bindings(&combined);
        assert_eq!(
            retained.len(),
            2,
            "both bindings hold something: {retained:?}"
        );

        let BindingDisposition::RetainedDirty {
            patch: Some(expected),
            ..
        } = &combined.bindings[0].disposition
        else {
            panic!(
                "expected a captured patch: {:?}",
                combined.bindings[0].disposition
            );
        };
        let dirty = retained
            .iter()
            .find(|r| r.repository == "solo")
            .expect("the captured-patch binding");
        assert_eq!(dirty.reason, "retained_dirty");
        assert!(dirty.bytes > 0);
        assert_eq!(dirty.path, expected.path);
        assert_eq!(dirty.bytes, expected.bytes);

        let errored = retained
            .iter()
            .find(|r| r.repository == "other")
            .expect("the RetainedError binding");
        assert_eq!(errored.reason, "retained_error");
        assert_eq!(
            errored.bytes, 5,
            "directory_size must measure the stand-in file"
        );
    }

    /// [`reap`]: a captured patch is deleted and its exact size is reported
    /// freed; a `RetainedError` binding is left alone (no evidence backs a
    /// forced removal), and a `RetainedDirty` binding with no patch
    /// (submodule fallback) is force-removed through git rather than a raw
    /// delete, keeping the source repository's own worktree registry
    /// truthful.
    #[test]
    fn reap_deletes_a_captured_patch_and_leaves_a_retained_error_binding_alone() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface =
            materialize(data.path(), "01REAP", std::slice::from_ref(&spec)).expect("materialize");
        std::fs::write(surface.bindings[0].worktree_path.join("wip.rs"), "//\n").expect("dirty");
        let mut report = teardown(&surface);
        let BindingDisposition::RetainedDirty {
            patch: Some(before),
            ..
        } = &report.bindings[0].disposition
        else {
            panic!(
                "expected a captured patch: {:?}",
                report.bindings[0].disposition
            );
        };
        let patch_path = before.path.clone();
        let expected_bytes = before.bytes;
        assert!(patch_path.is_file(), "the patch must exist before reaping");

        report.bindings.push(BindingTeardown {
            repository: "other".to_string(),
            worktree_path: dir.path().join("untouched"),
            work_branch: "sergeant/01REAP".to_string(),
            final_sha: None,
            disposition: BindingDisposition::RetainedError {
                detail: "fatal: could not read status".to_string(),
            },
        });

        let reaped = reap(&surface, &report);
        assert_eq!(reaped.work_id, "01REAP");

        let solo = reaped
            .bindings
            .iter()
            .find(|b| b.repository == "solo")
            .expect("solo's reap outcome");
        assert_eq!(
            solo.outcome,
            ReapOutcome::Reaped {
                bytes: expected_bytes
            }
        );
        assert!(!patch_path.exists(), "the patch must actually be deleted");

        let other = reaped
            .bindings
            .iter()
            .find(|b| b.repository == "other")
            .expect("other's reap outcome");
        assert!(
            matches!(&other.outcome, ReapOutcome::Skipped { .. }),
            "a RetainedError binding must be left alone: {:?}",
            other.outcome
        );

        // Nothing left in the root once the only retained artifact is gone.
        assert!(
            !surface.root.exists(),
            "reaping the last retained artifact must let the root go too"
        );
    }
}

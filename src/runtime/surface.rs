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

use crate::backend::BindingSummary;
use crate::domain::estate::RepositorySpec;
use crate::runtime::fsutil::create_dir_all_durable;
use crate::runtime::git::{
    GitError, canonical_git_common_dir, canonical_git_top_level, git, git_bytes,
    git_submodule_update, git_succeeds,
};
use crate::runtime::integrity::{
    DriftAttribution, EstateDriftObservation, IntegrityDisposition, IntegrityFinding, ObservedHead,
};
use crate::runtime::preflight::{AdmittedRepository, GitPreflight, PreflightEvidence};
use crate::runtime::repolock::{self, RepoLockError};

/// Directory under the data dir holding all work surfaces.
pub const SURFACES_DIR: &str = "surfaces";

/// The lock identity for one repository, derived **once per operation** and
/// carried to every guarded span that operation opens.
///
/// Deriving is a `git rev-parse` — a process spawn — so it happens at the
/// entry to `materialize`/`attach`/`rematerialize`/`teardown`/`reap`, per
/// repository, and never inside a span. A binding that already records its
/// canonical common directory (§3.5, journaled from Phase B onward) skips the
/// spawn entirely; only a pre-enrichment binding pays for one.
///
/// `identity` is the canonical git common directory (§2.7/§9.4), which is what
/// **both** locking layers key on. Fallback when git cannot answer — a source
/// path that is not a repository, or a git that will not run — is the
/// canonicalized source path: the pre-Phase-B key, which is strictly better
/// than no key at all, and whose one weakness (aliasing a checkout against its
/// own linked worktree) can only bite where git was already unable to tell us
/// they were the same repository.
///
/// `pub(crate)` for `runtime::sweep`'s deletion pass (#159), which mutates
/// the same ref store under the same two layers rather than inventing a
/// second locking discipline for `branch -D`.
#[derive(Debug, Clone)]
pub(crate) struct RepositoryGate {
    /// Data dir whose `locks/repo/` holds the interprocess lock file.
    data_dir: PathBuf,
    /// Canonical git common directory: one identity, both layers.
    identity: PathBuf,
}

impl RepositoryGate {
    /// Derive the gate for `source` by asking git for its common directory.
    /// One spawn; call once per repository per operation.
    pub(crate) fn derive(data_dir: &Path, source: &Path) -> Self {
        Self::from_common_dir(data_dir, canonical_git_common_dir(source).ok(), source)
    }

    /// The gate for a common directory already in hand — the shape
    /// `materialize_one` and `attach_one` use, because they need that same
    /// reading for the binding they are about to journal (§3.5) and must not
    /// spawn `git rev-parse` twice to get it.
    fn from_common_dir(data_dir: &Path, common_dir: Option<PathBuf>, source: &Path) -> Self {
        let identity = common_dir.unwrap_or_else(|| {
            std::fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf())
        });
        Self {
            data_dir: data_dir.to_path_buf(),
            identity,
        }
    }

    /// The gate for a journaled binding, preferring the identity recorded at
    /// admission (§3.5) over re-asking the checkout. Beyond saving a spawn,
    /// this is the more honest answer: the lock a teardown takes is the one
    /// the materialization that created these worktrees took, even if the
    /// source path has since been moved or replaced underneath us.
    fn for_binding(data_dir: &Path, binding: &RepositoryBinding) -> Self {
        match &binding.canonical_common_dir {
            Some(identity) => Self {
                data_dir: data_dir.to_path_buf(),
                identity: identity.clone(),
            },
            None => Self::derive(data_dir, &binding.source_path),
        }
    }
}

/// One in-process lock per repository identity, held across every worktree
/// mutation this module makes against that repository.
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
/// scope that fixes it: per repository identity, for the duration of the git
/// calls that touch its registry. Two works in different repositories still
/// proceed in parallel, and no request anywhere queues behind the core lock
/// for it.
///
/// **Keyed on the git common directory since Phase B**, not the source
/// checkout path. §2.7: different linked worktree paths may share one common
/// directory — and therefore one ref and worktree registry — while receiving
/// different locks, and a lock two mutators do not share is not a lock. This
/// stays as §9.4's permitted "fast local layer"; the file lock
/// [`with_repository`] takes inside it is the correctness boundary.
fn repository_lock(identity: &Path) -> Arc<Mutex<()>> {
    static LOCKS: OnceLock<Mutex<BTreeMap<PathBuf, Arc<Mutex<()>>>>> = OnceLock::new();
    let table = LOCKS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut table = table.lock().unwrap_or_else(|e| e.into_inner());
    Arc::clone(table.entry(identity.to_path_buf()).or_default())
}

/// Run `f` with this repository's worktree registry to itself — against every
/// thread in this process, and against every other process on the machine.
///
/// Two layers, in this order (§9.4: "the implementation may retain an
/// in-process mutex for efficiency, but the filesystem lock is the correctness
/// boundary"):
///
/// 1. [`repository_lock`], the in-process mutex. Cheap, and it means the
///    common case — a burst of submissions inside one daemon — never reaches
///    the filesystem to discover it must wait.
/// 2. [`repolock::acquire`], the interprocess file lock. Taken *inside* the
///    mutex, so exactly one thread of this process ever contends for it, which
///    is also what makes the bounded wait a wait for a genuinely foreign
///    holder rather than for ourselves.
///
/// Both are released in reverse order when this function returns.
///
/// **The span is not widened by any of this.** The asymmetric hold pattern
/// this module measured (reads, status and worktree population *outside*;
/// registry mutation inside a narrow span) is unchanged — the file lock is
/// taken and dropped in exactly the eight places the mutex already was, and no
/// read acquired a guard before this change or acquires one after it.
///
/// Mutex poisoning is deliberately ignored: the guard protects git's on-disk
/// registry, not an in-memory invariant, so a panic in one caller leaves
/// nothing for the next one to be confused by — and refusing every later
/// worktree operation for the daemon's lifetime would be a far worse failure
/// than the one being guarded against. A *file* lock failure is different and
/// is returned: it means another process may be mutating this registry right
/// now, which is precisely the condition this function exists to prevent
/// running under.
pub(crate) fn with_repository<T>(
    gate: &RepositoryGate,
    f: impl FnOnce() -> T,
) -> Result<T, SurfaceError> {
    let local = repository_lock(&gate.identity);
    let _local_guard = local.lock().unwrap_or_else(|e| e.into_inner());
    let _file_guard = repolock::acquire(&gate.data_dir, &gate.identity)?;
    Ok(f())
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
    /// Repository name within the estate.
    pub repository: String,
    /// Source repository top level (never written to by execution).
    pub source_path: PathBuf,
    /// Branch the surface was cut from — §3.5's "observed base branch, **if
    /// attached**".
    ///
    /// `None` is the honest record for a detached mount admitted under §8.3's
    /// bounded override: that admission pins an exact `base_sha` and, in the
    /// proposal's own words, records "no named base branch". Before Phase E
    /// this was a `String` carrying the sentinel `"(detached)"` — a value that
    /// *looks* like a branch name (git's ref grammar permits parentheses)
    /// sitting in the field every reader branches on.
    ///
    /// `#[serde(default)]` so a binding journaled before this change replays
    /// exactly what it recorded, the old sentinel included (as
    /// `Some("(detached)")`). Serialized even when `None`: an explicit `null`
    /// *is* the record that this Work has no named base branch, and a missing
    /// key would be indistinguishable from a payload predating the field.
    #[serde(default)]
    pub base_branch: Option<String>,
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
    /// §3.5's "canonical Git top level": what `source_path` actually resolved
    /// to as a working tree at admission (`git rev-parse --show-toplevel`,
    /// canonicalized).
    ///
    /// `Option` + `#[serde(default)]` (amendment C3: journal changes are
    /// additive): a binding journaled before Phase B replays as `None`, which
    /// is the truthful reading — nothing observed this at the time — and
    /// every consumer treats it as "re-derive or do without", never as a
    /// value to invent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_top_level: Option<PathBuf>,
    /// §3.5's "canonical Git common directory", and §2.7's lock identity:
    /// `git rev-parse --path-format=absolute --git-common-dir`, canonicalized,
    /// pinned at admission.
    ///
    /// Recording it is what lets a later teardown lock the *same* repository
    /// the materialization did without re-asking a checkout that may since
    /// have moved — and what lets `common_dir_finding` compare the worktree's
    /// answer against what was admitted rather than against a second live
    /// reading of the source. `None` for a pre-Phase-B binding, exactly like
    /// [`Self::canonical_top_level`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_common_dir: Option<PathBuf>,
    /// §3.5's "preflight result and any authorized override evidence": what
    /// §8.1 admitted this repository on, and what §8.3 waived to get there.
    ///
    /// `Option` + `#[serde(default)]` on the same amendment-C3 grounds as the
    /// two canonical identities above: `None` on a binding journaled before
    /// Phase E means *no preflight ran*, which is a different and more honest
    /// fact than an empty [`PreflightEvidence`] (preflight ran, waived
    /// nothing). `None` is also what a `materialize` call with no admission
    /// record records — the direct-call path the runtime's own tests use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflight: Option<PreflightEvidence>,
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
    /// Repositories that will get a worktree, in estate order.
    pub repositories: Vec<RepositorySpec>,
    /// §8.1's admission for this plan: the pinned base each repository was
    /// admitted on, plus every §8.3 finding the operator's override waived.
    ///
    /// Journaled here, in `surface.materializing`, because that event is the
    /// last thing written before `git worktree add` can touch a repository
    /// sergeant does not own — so "the override and every waived finding are
    /// journaled with the Work" (§8.3) is durable strictly before the effect
    /// it authorized, exactly like the plan it travels in.
    ///
    /// `#[serde(default)]`: a plan journaled before Phase E replays with an
    /// empty admission, which is the honest reading (no preflight ran), and
    /// materialization treats an absent record as "read the mount live", the
    /// only thing it could ever do then.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admitted: Vec<AdmittedRepository>,
    /// Whether the submission carried `--override-git-preflight` (§8.3).
    /// Recorded even when it waived nothing: the operator's authorization is
    /// a fact about the submission, not about what it happened to excuse.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub override_git_preflight: bool,
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
            admitted: Vec::new(),
            override_git_preflight: false,
        }
    }

    /// The same plan, carrying §8.1's admission record.
    ///
    /// A builder rather than a fifth parameter on [`Self::new`]: a plan
    /// reconstructed from a pre-Phase-E journal event legitimately has no
    /// admission, and every caller that has one has it as a unit.
    pub fn with_admission(mut self, preflight: &GitPreflight) -> Self {
        self.admitted = preflight.repositories.clone();
        self.override_git_preflight = preflight.override_requested;
        self
    }
}

/// A materialized work surface: one binding per targeted repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkSurface {
    /// Work this surface belongs to.
    pub work_id: String,
    /// Root directory holding the worktrees.
    pub root: PathBuf,
    /// One binding per repository, in estate order.
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

    /// §10.1's binding summary: the exact worktrees this Work may modify,
    /// with the branch and base commit each was bound to, in scope order.
    ///
    /// [`Self::execution_cwd`] answers "where does the process start", which
    /// for a multi-repo Work is the surface root — a directory the Work may
    /// *not* write to. This answers the different and more important question
    /// a backend's launch grammar needs: which directories are the mutation
    /// surface, and what is each one supposed to be.
    pub fn binding_summary(&self) -> Vec<BindingSummary> {
        self.bindings
            .iter()
            .map(|b| BindingSummary {
                repository: b.repository.clone(),
                worktree_path: b.worktree_path.clone(),
                work_branch: b.work_branch.clone(),
                base_branch: b.base_branch.clone(),
                base_sha: b.base_sha.clone(),
            })
            .collect()
    }

    /// Amendment 9 Q5 / #260 mechanism 3: how many commits each binding's
    /// worktree HEAD carries past its own `base_sha`, summed across bindings.
    ///
    /// The same underlying question [`TeardownReport::stranded_completion`]
    /// answers from a completed teardown's `final_sha` — but the opt-in wire
    /// injects this into a *closing stage's own context*, before that stage
    /// has even run, so there is no `BindingTeardown` yet to read. This reads
    /// the worktree live instead, one `git rev-list --count` per binding —
    /// the same "ask the worktree, not the journal" posture [`observe_head`]
    /// already takes for HEAD itself. A binding whose count git cannot answer
    /// (a vanished worktree, an unreadable `base_sha`) contributes `0` rather
    /// than failing the whole sum: the engine sharing "no commits observed"
    /// is the fail-closed answer for a fact a workflow only ever reads
    /// advisory, never gates on.
    pub fn commits_since_base(&self) -> u64 {
        self.bindings
            .iter()
            .filter_map(|b| {
                git(
                    &b.worktree_path,
                    &["rev-list", "--count", &format!("{}..HEAD", b.base_sha)],
                )
                .ok()
            })
            .filter_map(|out| out.trim().parse::<u64>().ok())
            .sum()
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

/// Where one stage's declared `output/` artifacts were retained
/// separately from the flat dirty patch (#240): a per-stage-labeled copy
/// of whatever real files existed under `<stage-dir>/output/` (excluding
/// `README.md`, which is the declaration, not an artifact — Rule 4,
/// `docs/icm/convention.md`) at the moment the worktree was about to be
/// removed. Exists so a reader can retrieve one stage's evidence
/// directly, without reconstructing it from the flat patch that already
/// bundles it in with every other change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedStageOutput {
    /// The stage directory name the artifacts were declared under (e.g.
    /// `"20-panel"`), matching the workflow's own directory naming.
    pub stage: String,
    /// Where the copied artifacts were written, under the surface root,
    /// sibling to the worktree they were copied out of.
    pub path: PathBuf,
    /// Total bytes copied.
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
        /// #240: each stage's declared `output/` artifacts, copied out
        /// separately and per-stage labeled, alongside `patch`. Empty
        /// when `patch` is `None` — the whole worktree directory (output/
        /// included) was left in place instead, so there is nothing extra
        /// to copy. `#[serde(default)]` for the same replay reason as
        /// `patch`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        outputs: Vec<RetainedStageOutput>,
    },
    /// The worktree path was already gone: recorded, not treated as success.
    Missing,
    /// Git refused to remove it: retained, with Git's own diagnostic.
    RetainedError {
        /// Why removal failed.
        detail: String,
    },
    /// §11.7: the worktree's HEAD was detached at a commit no named ref in
    /// the source repository is *proven* to reach. Removing it would leave
    /// that commit unreferenced — reachable from nothing, and so a candidate
    /// for git's own garbage collection — so teardown retains the worktree
    /// instead. A retained worktree's HEAD is itself a gc root, which is
    /// precisely what makes retention the fail-closed answer here.
    ///
    /// Distinct from [`RetainedError`](Self::RetainedError) on purpose: no
    /// git command refused anything: sergeant *chose* not to remove this,
    /// and an operator resolving it gives the commit a branch rather than
    /// debugging a git failure. Additive on replay in the direction that
    /// matters (C3): no event journaled before this phase can carry it.
    RetainedUnreferenced {
        /// The commit HEAD was detached at.
        head_sha: String,
        /// What was checked, and why removal was refused.
        evidence: String,
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
    /// §11.7: what the worktree's HEAD **actually** pointed at when teardown
    /// looked, as opposed to the `work_branch` just above, which is only what
    /// this binding was assigned. `None` means *not assessed* — a vanished
    /// worktree, a git that could not answer, or a `surface.torn_down`
    /// journaled before this field existed (C3: additive, and absence never
    /// reads as agreement).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_head: Option<ObservedHead>,
    /// §11.3's directly attributable findings for this binding, in the order
    /// teardown observed them. Empty means teardown found nothing to
    /// attribute — which is not the same as "not assessed"; that is what an
    /// absent `observed_head` says. `#[serde(default)]` so an old event
    /// replays with no findings and no integrity assessment at all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<IntegrityFinding>,
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
    /// §11.4 (bounded by amendment C6): every bound repository mount whose
    /// committed HEAD moved between this Work's binding base and its
    /// retirement. Reported, never attributed, and never part of
    /// [`TeardownReport::integrity`] — a mount moving during the Work window
    /// is not evidence the Work moved it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drift: Vec<EstateDriftObservation>,
}

impl TeardownReport {
    /// §11.5's orthogonal axis for this teardown.
    ///
    /// Dirty when either half of "core can account for this surface" fails:
    ///
    /// - any binding carries a §11.3 finding (the attributable half), or
    /// - any binding was not retired cleanly — `!clean` (the *retirement*
    ///   half). Every §11.3-nameable cause already yields its own finding, so
    ///   this second clause only ever fires alone for a retention §11.3 has
    ///   no vocabulary for: a `RetainedError` where `git status` itself
    ///   refused, so teardown could not establish anything at all. Reporting
    ///   that as clean would be the one answer the closed enum must not force
    ///   — fail closed instead.
    ///
    /// Estate drift is deliberately absent from both clauses (§11.4).
    ///
    /// Computed, never stored on the report: `teardown` also runs as
    /// `materialize`'s partial-failure *rollback*, which is not a retirement
    /// and must not be journaled as an integrity assessment at all. The one
    /// caller that applies this is the retirement path in
    /// `Engine::settle_surface`.
    pub fn integrity(&self) -> IntegrityDisposition {
        if self.clean && self.bindings.iter().all(|b| b.findings.is_empty()) {
            IntegrityDisposition::Clean
        } else {
            IntegrityDisposition::Dirty
        }
    }

    /// Every binding's findings, flattened in binding order — what a view
    /// renders when it wants "what was found", not "what happened per
    /// repository".
    pub fn findings(&self) -> impl Iterator<Item = &IntegrityFinding> {
        self.bindings.iter().flat_map(|b| b.findings.iter())
    }

    /// ADR 0007(b): whether this teardown, against the surface it tore down,
    /// shows a closing stage that declared a commit as its durable outcome
    /// but never actually advanced the branch and left the worktree dirty —
    /// the safety net for when an actor guesses wrong about its own runtime
    /// model (`docs/adr/0007-actor-runtime-contract.md`).
    ///
    /// The engine still learns nothing about what a commit *is* (NORTH-STAR:
    /// "the engine learns no output vocabulary; only the pointer is core"):
    /// this reads two facts the pointer already computes — a binding's
    /// teardown disposition, and whether its finalize commit ever moved past
    /// the surface's own base SHA — rather than asking any workflow what it
    /// meant to do.
    ///
    /// Structural, not state-aware: this alone does not know whether the
    /// Work this teardown belongs to actually reached `Completed` — callers
    /// that need `reported_state`'s full `stranded_completion` semantics
    /// (api.rs) gate this on `work.state` themselves. It exists standalone
    /// here specifically so a caller with no live `Work` at hand — the
    /// projection reducer, computing the slim index's effective disposition
    /// at `surface.torn_down` time — can still ask it.
    pub fn stranded_completion(&self, surface: &WorkSurface) -> bool {
        self.bindings.iter().any(|binding| {
            let never_advanced = surface
                .bindings
                .iter()
                .find(|b| b.repository == binding.repository)
                .is_some_and(|b| binding.final_sha.as_deref() == Some(b.base_sha.as_str()));
            never_advanced
                && matches!(
                    binding.disposition,
                    BindingDisposition::RetainedDirty { .. }
                )
        })
    }
}

/// Failure materializing a surface.
#[derive(Debug, thiserror::Error)]
pub enum SurfaceError {
    /// Git refused.
    #[error(transparent)]
    Git(#[from] GitError),
    /// The repository's interprocess lock (§9.4) could not be taken, so the
    /// registry mutation it guards was never attempted. Carried transparently:
    /// [`RepoLockError`] already names the lock file, the git common directory
    /// it stands for, and how long acquisition waited — everything an operator
    /// needs, and nothing this layer can add to it.
    #[error(transparent)]
    RepositoryLock(#[from] RepoLockError),
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
///
/// `data_dir` is the daemon's own storage, and is *not* assumed to be the
/// parent of `surfaces_root` (R-MVP1-1 unbound the two). It is here for one
/// reason: it locates the interprocess repository locks §9.4 requires
/// (`<data_dir>/locks/repo/`; see [`crate::runtime::repolock`] for why that
/// key is complete under §6.2). Every entry point in this module takes it for
/// the same reason and uses it for nothing else.
pub fn materialize(
    data_dir: &Path,
    surfaces_root: &Path,
    work_id: &str,
    repositories: &[RepositorySpec],
) -> Result<WorkSurface, SurfaceError> {
    materialize_with(data_dir, surfaces_root, work_id, repositories, &[])
}

/// [`materialize`] against §8.1's admitted facts rather than a live re-read
/// of each mount — the submission path.
///
/// This is what makes §8.2's pin real. `materialize` asks each mount for its
/// HEAD at the moment it runs; preflight already asked, judged the answer, and
/// journaled it in the plan, so re-asking here would mean the commit a Work is
/// based on is not necessarily the commit that was admitted — a mount whose
/// branch advanced in between would silently move the base, and a detached
/// mount admitted under §8.3 would re-derive a base branch the override
/// deliberately recorded as absent.
///
/// A repository with no matching admission record falls back to the live read,
/// which is exactly what [`materialize`] does for every repository: the two
/// share one body, so the admitted path cannot drift from the direct one.
pub fn materialize_admitted(
    data_dir: &Path,
    surfaces_root: &Path,
    work_id: &str,
    repositories: &[RepositorySpec],
    admitted: &[AdmittedRepository],
) -> Result<WorkSurface, SurfaceError> {
    materialize_with(data_dir, surfaces_root, work_id, repositories, admitted)
}

fn materialize_with(
    data_dir: &Path,
    surfaces_root: &Path,
    work_id: &str,
    repositories: &[RepositorySpec],
    admitted: &[AdmittedRepository],
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
        let pin = admitted
            .iter()
            .find(|record| record.repository == repository.name);
        let outcome =
            match materialize_one(data_dir, &root, &canonical_root, &branch, repository, pin) {
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
                let report = teardown(data_dir, &partial);
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
    data_dir: &Path,
    root: &Path,
    canonical_root: &Path,
    branch: &str,
    repository: &RepositorySpec,
    pin: Option<&AdmittedRepository>,
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
    // Read HEAD metadata **before** taking the per-repository lock.  Both
    // calls touch only `.git/HEAD` and ref storage — they do not access the
    // worktree registry (`.git/worktrees/`) that `with_repository` protects.
    // Running them before acquiring the lock lets concurrent submissions that
    // target the same repository overlap this work rather than serializing all
    // four git spawns per materialization behind a single gate.
    //
    // Correctness under a concurrent HEAD advance: `base_sha` is passed
    // explicitly to `git worktree add` rather than "HEAD", so the new
    // worktree lands at the commit we recorded here even if the branch tip
    // moves between this read and the add.  The provenance record and the
    // actual checkout agree with each other; a one-commit lag behind the tip
    // is the same fact it was before (the submit captured a snapshot of HEAD
    // at request time, and that snapshot is what the run executes on).
    //
    // §8.2, Phase E: when this materialization came through admission, none of
    // that reading happens at all — `pin` already carries the base preflight
    // judged, and re-deriving it here would mean the Work is based on whatever
    // the mount says *now* rather than on the fact that was admitted and
    // journaled.  The live read below is the direct-call path (and every
    // pre-Phase-E replay), unchanged.
    let base_sha = match pin {
        Some(record) => record.base_sha.clone(),
        None => git(&repository.path, &["rev-parse", "HEAD"])?,
    };
    // A detached HEAD has no branch name; record the *absence* rather than
    // inventing a sentinel that reads like one.
    let base_branch = match pin {
        Some(record) => record.base_branch.clone(),
        None => git(
            &repository.path,
            &["symbolic-ref", "--quiet", "--short", "HEAD"],
        )
        .ok(),
    };

    // §3.5's two canonical identities, read here — once, outside the lock,
    // alongside the other admission facts — and then used for both purposes
    // they exist for: the lock key below, and the binding's own record of what
    // this repository *was* at admission.  Deriving them here rather than
    // inside the span is the whole point: `with_repository` must never spawn
    // a `git rev-parse` while holding a registry lock.  An admitted
    // materialization has both already, resolved by the same two functions
    // during preflight.
    let canonical_top_level = match pin {
        Some(record) => Some(record.canonical_top_level.clone()),
        None => canonical_git_top_level(&repository.path).ok(),
    };
    let canonical_common_dir = match pin {
        Some(record) => Some(record.canonical_common_dir.clone()),
        None => canonical_git_common_dir(&repository.path).ok(),
    };
    let gate =
        RepositoryGate::from_common_dir(data_dir, canonical_common_dir.clone(), &repository.path);

    // Only `git worktree add` touches `.git/worktrees/` — the operation the
    // per-repository lock exists to serialize.  Hold it for exactly that span
    // and nothing more.
    //
    // `--no-checkout` skips the working-tree file population inside the lock.
    // `git reset --hard HEAD` (which reads only the object store and writes
    // to the worktree directory) runs after the lock releases so that
    // concurrent submissions against the same repository can overlap it with
    // the next submission's registry write — shaving the lock-held time from
    // ~57 ms (full add + checkout) to ~43 ms (registry write only) per work.
    //
    // Correctness: `--no-checkout` still creates the branch at `base_sha` and
    // sets HEAD in the new worktree to that branch, so `checkout_worktree`
    // below populates exactly the same content as a full add would have.  The
    // checkout completes before `materialize_one` returns, so no caller ever
    // sees a worktree with its branch set but its files absent.
    with_repository(&gate, || {
        add_worktree_no_checkout(&repository.path, &worktree_path, branch, &base_sha)
    })??;
    // Outside the per-repository lock: populate the working tree.
    //
    // `git reset --hard HEAD` is used rather than `git checkout HEAD` for two
    // reasons:
    //
    // 1. Detached-HEAD avoidance.  After `--no-checkout`, HEAD is a symref to
    //    the work branch.  `git checkout HEAD` resolves "HEAD" to the commit
    //    SHA and performs a commit-level checkout, which DETACHES HEAD from the
    //    branch.  Commits the runner makes in a detached-HEAD worktree do not
    //    advance the branch, silently losing the run's output on teardown.
    //    `git reset --hard HEAD` resolves HEAD to the same commit but leaves
    //    HEAD's branch-pointer intact.
    //
    // 2. `--no-checkout` leaves the index empty (it does NOT pre-populate it
    //    as a regular checkout would).  `git reset --hard HEAD` both fills the
    //    index from HEAD's tree and writes those files to disk — a single
    //    command that does exactly what we need.
    //
    // Unlike `add_worktree_from` (where a git failure is atomic — either the
    // whole add lands or nothing is written), a `checkout_worktree` failure
    // here leaves behind durable side-effects from the preceding
    // `add_worktree_no_checkout`: a branch ref, a `.git/worktrees/<name>/`
    // registry entry, and an empty worktree directory.  The caller's
    // `materialize` cannot see these because no `RepositoryBinding` is
    // returned, so it cannot drive teardown for them.  Roll them back inline
    // before returning the error.  Both cleanups are best-effort: stale
    // registry entries are reaped by `git worktree prune` on the next
    // teardown for this repository; orphaned branches would block a retry (the
    // engine calls `materialize` again with `-b <same-branch>`, which fails if
    // the branch exists), so deleting the branch is the more important step.
    if let Err(checkout_err) = checkout_worktree(&worktree_path) {
        let path = worktree_path.display().to_string();
        // Remove registry entry first so the branch is unreferenced.
        let _ = with_repository(&gate, || {
            git(&repository.path, &["worktree", "remove", "--force", &path]).map(|_| ())
        });
        // Delete the branch so a retry can re-create it with `-b`.
        let _ = git(&repository.path, &["branch", "-D", branch]);
        return Err(checkout_err);
    }

    // `git worktree add --no-checkout -b <branch> <path> <sha>` creates the
    // branch at `<sha>` and sets HEAD in the new worktree to that branch.
    // `git reset --hard HEAD` (above) does not run any `post-checkout` hook,
    // so HEAD in the worktree stays at exactly `base_sha`.  `head_sha` is
    // recorded as `base_sha` — correct by construction.
    let head_sha = base_sha.clone();

    Ok(RepositoryBinding {
        repository: repository.name.clone(),
        source_path: repository.path.clone(),
        base_branch,
        base_sha,
        worktree_path,
        work_branch: branch.to_string(),
        head_sha,
        origin: BindingOrigin::Cut,
        canonical_top_level,
        // The `Option` is carried, not the gate's `identity`: the gate falls
        // back to the canonicalized source path when git cannot answer, and
        // that path is not a common directory — journaling it as one would
        // record a value that only usually means what the field says.
        canonical_common_dir,
        // §3.5: the preflight result travels with the binding it admitted.
        // `None` when this materialization did not come through admission —
        // "no preflight ran" is a different fact from "preflight waived
        // nothing".
        preflight: pin.map(|record| record.evidence.clone()),
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
///
/// §3.5/C3: both canonical identities are **re-derived** here rather than
/// carried forward. A retry can land days after admission, and this is the
/// one path that re-establishes the worktree from the branch — if the mount
/// has been re-cloned or moved in between, the identity that matters is the
/// one the re-attachment actually used. A pre-Phase-B binding that replayed
/// with `None` therefore gains its identities on first retry, rather than
/// staying blank forever.
pub fn rematerialize(data_dir: &Path, surface: &WorkSurface) -> Result<WorkSurface, SurfaceError> {
    create_dir_all_durable(&surface.root).map_err(|source| SurfaceError::Io {
        path: surface.root.display().to_string(),
        source,
    })?;
    let mut bindings = Vec::with_capacity(surface.bindings.len());
    for binding in &surface.bindings {
        // One derivation per binding per operation, before any span opens.
        let canonical_common_dir = canonical_git_common_dir(&binding.source_path).ok();
        let canonical_top_level = canonical_git_top_level(&binding.source_path).ok();
        let gate = RepositoryGate::from_common_dir(
            data_dir,
            canonical_common_dir.clone(),
            &binding.source_path,
        );
        if !binding.worktree_path.exists() {
            with_repository(&gate, || {
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
            })??;
        }
        let head_sha = git(&binding.worktree_path, &["rev-parse", "HEAD"])?;
        bindings.push(RepositoryBinding {
            head_sha,
            canonical_top_level: canonical_top_level
                .or_else(|| binding.canonical_top_level.clone()),
            canonical_common_dir: canonical_common_dir
                .or_else(|| binding.canonical_common_dir.clone()),
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
///
/// `data_dir` locates the interprocess repository locks, exactly as in
/// [`materialize`].
pub fn attach(
    data_dir: &Path,
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
        let outcome = match attach_one(data_dir, &root, &canonical_root, target_work_id, target) {
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
                let report = teardown(data_dir, &partial);
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
    data_dir: &Path,
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
    // §3.5, derived once and outside the span, exactly as `materialize_one`
    // does. An attached binding shares the target's *branch*, but its own
    // identities are read fresh here: this Work is admitting this repository
    // now, and "what the mount was when the target was admitted" is a
    // different fact, already recorded on the target's own binding.
    let canonical_top_level = canonical_git_top_level(&target.source_path).ok();
    let canonical_common_dir = canonical_git_common_dir(&target.source_path).ok();
    let gate = RepositoryGate::from_common_dir(
        data_dir,
        canonical_common_dir.clone(),
        &target.source_path,
    );
    let head_sha = with_repository(&gate, || -> Result<String, SurfaceError> {
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
    })??;

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
        canonical_top_level,
        canonical_common_dir,
        // Carried over from the target's own binding, exactly like
        // `base_branch`/`base_sha` above: this binding did not go through an
        // admission of its own — it attached to a branch a different Work
        // already admitted — so the honest record is that Work's evidence,
        // not a fresh empty one implying nothing was ever waived.
        preflight: target.preflight.clone(),
    })
}

/// Tear down a surface: remove clean worktrees, retain every branch, and
/// record everything that could not be removed (fail closed).
///
/// This never returns an error. Teardown runs on the way to a terminal state,
/// and a Work does not stop being canceled because a worktree was dirty — the
/// honest outcome is a recorded report, which is what the caller journals.
/// That includes a repository lock this daemon could not take: it becomes a
/// `RetainedError` disposition with the timeout as its detail, which is the
/// same fail-closed shape a git refusal already produces.
///
/// `data_dir` locates the interprocess repository locks, exactly as in
/// [`materialize`].
pub fn teardown(data_dir: &Path, surface: &WorkSurface) -> TeardownReport {
    let mut bindings = Vec::with_capacity(surface.bindings.len());
    let mut drift = Vec::new();
    for binding in &surface.bindings {
        // §11.4/C6: read the mount before this binding's own teardown
        // touches anything, so `observed` is the estate as the Work left it
        // rather than as teardown left it.
        drift.extend(observe_estate_drift(binding));
        let outcome = teardown_binding(data_dir, binding);
        bindings.push(BindingTeardown {
            repository: binding.repository.clone(),
            worktree_path: binding.worktree_path.clone(),
            work_branch: binding.work_branch.clone(),
            final_sha: outcome.final_sha,
            observed_head: outcome.observed_head,
            findings: outcome.findings,
            disposition: outcome.disposition,
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
        drift,
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
    /// to reconcile two vocabularies for the same fact. `retained_output`
    /// (#240) is the one tag with no such counterpart: it names a
    /// per-stage `output/` artifact copy, not a teardown disposition on
    /// its own.
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
        .flat_map(retained_bindings_for)
        .collect()
}

/// One binding's retained evidence: the primary retention
/// ([`retained_bindings`]'s original single-entry shape) plus, when
/// present, one further entry per `#240` stage-output copy — a `Vec`
/// rather than an `Option` because a single binding can now carry more
/// than one piece of independently retrievable evidence.
fn retained_bindings_for(b: &BindingTeardown) -> Vec<RetainedBinding> {
    let mut out = Vec::new();
    match &b.disposition {
        BindingDisposition::RetainedDirty {
            changes,
            patch: Some(info),
            outputs,
        } => {
            // Live check, not the size journaled at teardown time: a
            // `sgt work reap` since then (or anything else that removed
            // the file) must not keep showing up here — the same
            // "report what is actually on disk right now" rule
            // `directory_size`'s callers below already follow.
            if info.path.is_file() {
                out.push(RetainedBinding {
                    repository: b.repository.clone(),
                    path: info.path.clone(),
                    reason: "retained_dirty",
                    detail: changes.clone(),
                    bytes: info.bytes,
                });
            }
            out.extend(retained_output_bindings(b, outputs));
        }
        BindingDisposition::RetainedDirty {
            changes,
            patch: None,
            outputs,
        } => {
            if b.worktree_path.exists() {
                out.push(RetainedBinding {
                    repository: b.repository.clone(),
                    path: b.worktree_path.clone(),
                    reason: "retained_dirty",
                    detail: changes.clone(),
                    bytes: directory_size(&b.worktree_path),
                });
            }
            // `patch: None` means the whole worktree directory (output/
            // included) was left in place instead of being reclaimed, so
            // `outputs` is always empty here (see `retain_dirty`) — no
            // separate copy exists to report.
            out.extend(retained_output_bindings(b, outputs));
        }
        BindingDisposition::RetainedError { detail } => {
            if b.worktree_path.exists() {
                out.push(RetainedBinding {
                    repository: b.repository.clone(),
                    path: b.worktree_path.clone(),
                    reason: "retained_error",
                    detail: detail.clone(),
                    bytes: directory_size(&b.worktree_path),
                });
            }
        }
        BindingDisposition::RetainedUnreferenced { evidence, .. } => {
            if b.worktree_path.exists() {
                out.push(RetainedBinding {
                    repository: b.repository.clone(),
                    path: b.worktree_path.clone(),
                    reason: "retained_unreferenced",
                    detail: evidence.clone(),
                    bytes: directory_size(&b.worktree_path),
                });
            }
        }
        BindingDisposition::Removed | BindingDisposition::Missing => {}
    }
    out
}

/// #240: one [`RetainedBinding`] per stage-output copy still on disk,
/// `reason: "retained_output"` — a distinct tag from `"retained_dirty"`
/// so a reader (or `sgt work retained`'s own rendering) can tell a
/// per-stage evidence copy apart from the flat patch it sits alongside.
fn retained_output_bindings(
    b: &BindingTeardown,
    outputs: &[RetainedStageOutput],
) -> Vec<RetainedBinding> {
    outputs
        .iter()
        .filter(|o| o.path.exists())
        .map(|o| RetainedBinding {
            repository: b.repository.clone(),
            path: o.path.clone(),
            reason: "retained_output",
            detail: format!("stage {} declared output/ artifacts", o.stage),
            bytes: o.bytes,
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
///
/// `data_dir` locates the interprocess repository locks, exactly as in
/// [`materialize`].
pub fn reap(data_dir: &Path, surface: &WorkSurface, teardown: &TeardownReport) -> ReapReport {
    let bindings = teardown
        .bindings
        .iter()
        .map(|b| BindingReap {
            repository: b.repository.clone(),
            outcome: reap_binding(data_dir, surface, b),
        })
        .collect();
    remove_surface_root(&surface.root);
    ReapReport {
        work_id: teardown.work_id.clone(),
        bindings,
    }
}

fn reap_binding(data_dir: &Path, surface: &WorkSurface, binding: &BindingTeardown) -> ReapOutcome {
    match &binding.disposition {
        BindingDisposition::RetainedDirty {
            patch: Some(info),
            outputs,
            ..
        } => {
            // #240: dispose of the stage-output copies alongside the
            // patch — `remove_surface_root` refuses a non-empty root, so
            // leaving these behind would leave the whole surface
            // undeletable after every retained patch was reaped.
            //
            // Patch removal is attempted *first*, deliberately: it is the
            // one step here that can report a real, non-`NotFound` error,
            // and the outputs are only destroyed once that step has
            // already succeeded (or found nothing to remove). Destroying
            // the outputs before the patch would leave `Failed` on a real
            // patch-removal error indistinguishable from "nothing was
            // reaped", when the irreversible output loss already
            // happened.
            let outcome = match std::fs::remove_file(&info.path) {
                Ok(()) => ReapOutcome::Reaped { bytes: info.bytes },
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => ReapOutcome::NothingRetained,
                Err(e) => {
                    return ReapOutcome::Failed {
                        detail: e.to_string(),
                    };
                }
            };
            for output in outputs {
                let _ = std::fs::remove_dir_all(&output.path);
            }
            outcome
        }
        BindingDisposition::RetainedDirty { patch: None, .. } => {
            if !binding.worktree_path.exists() {
                return ReapOutcome::NothingRetained;
            }
            // The directory is still a real, registered git worktree (that
            // is exactly why it was never removed at teardown) — going
            // through git rather than a raw recursive delete keeps the
            // source repository's own worktree registry consistent, the
            // same as every other removal this module ever does.
            let Some(surface_binding) = surface
                .bindings
                .iter()
                .find(|b| b.repository == binding.repository)
            else {
                return ReapOutcome::Failed {
                    detail: "no surface binding recorded for this repository; cannot resolve \
                             which source repository's worktree registry to update"
                        .to_string(),
                };
            };
            let source = surface_binding.source_path.clone();
            // Derived once, outside the span, from the binding's own record —
            // the same rule every other guarded span in this module follows.
            let gate = RepositoryGate::for_binding(data_dir, surface_binding);
            let bytes = directory_size(&binding.worktree_path);
            let path = binding.worktree_path.display().to_string();
            // Two failure layers flattened into one `Failed`: a lock we could
            // not take and a removal git refused are both "this was not
            // reaped, here is why", and reap's report has one place to say so.
            match with_repository(&gate, || {
                git(&source, &["worktree", "remove", "--force", &path])
            }) {
                Ok(Ok(_)) => ReapOutcome::Reaped { bytes },
                Ok(Err(e)) => ReapOutcome::Failed {
                    detail: e.to_string(),
                },
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
        // §11.7: reaping this would do exactly what teardown refused to do —
        // drop the last reference to commits no named ref reaches. `reap` is
        // an explicit operator action, but it is a *disk reclamation* verb,
        // not a "destroy possible output" one, so it stops here and names the
        // one-line remedy that makes the worktree ordinary again.
        BindingDisposition::RetainedUnreferenced { head_sha, .. } => ReapOutcome::Skipped {
            reason: format!(
                "this worktree holds a detached HEAD at {head_sha} that no named ref reaches; \
                 give the commit a branch (`git branch <name> {head_sha}`) and retry teardown \
                 instead of reaping"
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
            outputs: Vec::new(),
        };
    }
    let Some(patch) = capture_dirty_patch(binding) else {
        return BindingDisposition::RetainedDirty {
            changes,
            patch: None,
            outputs: Vec::new(),
        };
    };
    // #240: copy each stage's declared `output/` artifacts out of the
    // worktree in this same before-removal window the patch capture just
    // used — the worktree's files are still on disk (git add/diff never
    // remove anything), and this is the last point at which they are.
    let outputs = retain_stage_outputs(binding);
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
    // best-effort so it does not become its own untracked leak — the
    // copied `outputs` are removed the same way, for the same reason: the
    // originals are still sitting in the still-present directory.
    match git(
        &binding.source_path,
        &["worktree", "remove", "--force", &path],
    ) {
        Ok(_) => BindingDisposition::RetainedDirty {
            changes,
            patch: Some(patch),
            outputs,
        },
        Err(_) => {
            let _ = std::fs::remove_file(&patch.path);
            for output in &outputs {
                let _ = std::fs::remove_dir_all(&output.path);
            }
            BindingDisposition::RetainedDirty {
                changes,
                patch: None,
                outputs: Vec::new(),
            }
        }
    }
}

/// #240: copy each stage directory's declared `output/` artifacts out of
/// `binding.worktree_path` before it is removed, into
/// `<stage-dir>.output/<stage-dir>/` sibling to the worktree — the same
/// retained-evidence area [`capture_dirty_patch`] writes its patch into.
/// A stage directory is any top-level entry of the worktree that has an
/// `output/` subdirectory; nothing here consults the workflow catalog
/// (Rule 4, `docs/icm/convention.md`: "no engine collection, no artifact
/// manifest machinery" — the worktree's own filesystem shape is ground
/// truth, exactly as [`capture_dirty_patch`] already treats it). Returns
/// one [`RetainedStageOutput`] per stage directory that actually had a
/// real (non-`README.md`) file to copy; a stage whose `output/` is empty
/// or holds only its own declaration is not reported.
fn retain_stage_outputs(binding: &RepositoryBinding) -> Vec<RetainedStageOutput> {
    let Some(surface_root) = binding.worktree_path.parent() else {
        return Vec::new();
    };
    let Ok(top_level) = std::fs::read_dir(&binding.worktree_path) else {
        return Vec::new();
    };
    let dest_root = surface_root.join(format!("{}.output", binding.repository));
    let mut outputs = Vec::new();
    for entry in top_level.flatten() {
        let stage_dir = entry.path();
        if !stage_dir.is_dir() {
            continue;
        }
        let output_dir = stage_dir.join("output");
        if !output_dir.is_dir() {
            continue;
        }
        let Some(stage_name) = stage_dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let dest = dest_root.join(stage_name);
        let bytes = copy_declared_output_artifacts(&output_dir, &dest, &binding.worktree_path);
        if bytes > 0 {
            outputs.push(RetainedStageOutput {
                stage: stage_name.to_string(),
                path: dest,
                bytes,
            });
        } else {
            // Nothing real was copied (empty dir, or only README.md) —
            // remove the now-empty destination so it never reads as
            // retained evidence.
            let _ = std::fs::remove_dir_all(&dest);
        }
    }
    outputs
}

/// Copy every real file under `src`, recursively, except the top-level
/// `README.md` (the declaration, not an artifact — Rule 4), into the
/// matching path under `dest`, creating directories on demand. A stage may
/// legitimately write a nested artifact tree under `output/` — convention.md
/// does not forbid it — so this descends into subdirectories rather than
/// stopping at the top level.
///
/// A file `git add -A` would not stage (gitignored) is skipped too, the
/// same exclusion [`capture_dirty_patch`] gets for free from `git add`
/// itself: `output/` is declared to hold only real, trackable artifacts
/// (convention.md §4/Rule 4), and a file Git itself would refuse to track
/// is not one — copying it here would reintroduce the disk-bloat risk R4
/// was written to prevent (#109) if that convention is ever violated.
///
/// Best-effort per file: a file this cannot read or write is skipped
/// rather than aborting the whole stage's copy, matching
/// [`capture_dirty_patch`]'s own best-effort posture toward anything past
/// the point where failure can no longer lose already-durable evidence.
/// Returns the total bytes actually copied.
fn copy_declared_output_artifacts(src: &Path, dest: &Path, worktree_root: &Path) -> u64 {
    copy_declared_output_artifacts_at(src, dest, worktree_root, true)
}

fn copy_declared_output_artifacts_at(
    src: &Path,
    dest: &Path,
    worktree_root: &Path,
    is_top_level: bool,
) -> u64 {
    let Ok(entries) = std::fs::read_dir(src) else {
        return 0;
    };
    let mut bytes = 0u64;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            bytes += copy_declared_output_artifacts_at(
                &path,
                &dest.join(entry.file_name()),
                worktree_root,
                false,
            );
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if is_top_level && path.file_name().and_then(|n| n.to_str()) == Some("README.md") {
            continue;
        }
        if git_succeeds(
            worktree_root,
            &["check-ignore", "-q", &path.display().to_string()],
        ) {
            continue;
        }
        let Ok(contents) = std::fs::read(&path) else {
            continue;
        };
        if std::fs::create_dir_all(dest).is_err() {
            continue;
        }
        let Some(file_name) = path.file_name() else {
            continue;
        };
        if crate::runtime::fsutil::write_atomic(&dest.join(file_name), &contents).is_ok() {
            bytes += contents.len() as u64;
        }
    }
    bytes
}

/// #260 Q4 / Amendment 9's general finalize sweep, `docs/icm/convention.md`
/// §1a's "merge-back semantics": run once per binding at Work close,
/// **before** [`teardown`]'s own dirty check, so a promote-class deliverable
/// ships in the branch's history and an evidence-class `output/` never does.
///
/// Walks the same top-level `<stage-id>/output/` shape [`retain_stage_outputs`]
/// and the #260 Q3 gate already treat as ground truth for a Work's own
/// produced artifacts (never the workflow *package*'s own tree — Rule 4's
/// "declared locations", not a manifest). For each one that actually holds
/// real (non-`README.md`) content, [`crate::domain::workflow::
/// declared_output_disposition`] reads that stage's authored disposition
/// out of the *workflow package* (`workflow_source`) — `output/README.md`
/// lives with the procedure, never the worktree — and:
///
/// - `Promote`: left untouched; it ships on the branch like any other file.
/// - `Evidence` (§1a's default: "silence promotes nothing"): copied to
///   `<repo>.output/<stage>/`, the same destination and shape a *dirty*
///   worktree's evidence already lands at, so nothing here can lose content
///   a crash mid-sweep would otherwise have kept — then removed from the
///   worktree.
///
/// Every stage this actually swept lands in **one** commit, made with a
/// distinct `sergeant finalize` identity so it reads as the engine's own
/// act rather than the actor's — "the final commit" §1a's ruling names.
/// Nothing is committed when nothing was evidence-class (no extra history
/// for a binding with nothing to sweep). `workflow_source == None` (the
/// embedded default, which ships no package directory to read a
/// disposition from) sweeps nothing, matching the Q3 gate's own
/// `SOURCE_EMBEDDED` early return.
///
/// Best-effort and fail-open toward the worktree's *content*: a git
/// failure here is folded into the returned report rather than treated as
/// fatal — every real artifact was already durably copied out as evidence
/// before removal was attempted, and the ordinary teardown that runs
/// immediately after still inspects the worktree fresh and still fails
/// closed on whatever it actually finds.
pub fn finalize_sweep(
    binding: &RepositoryBinding,
    workflow_source: Option<&Path>,
) -> FinalizeSweepReport {
    let mut report = FinalizeSweepReport::default();
    let Some(workflow_source) = workflow_source else {
        return report;
    };
    let Some(surface_root) = binding.worktree_path.parent() else {
        return report;
    };
    let Ok(top_level) = std::fs::read_dir(&binding.worktree_path) else {
        return report;
    };
    let dest_root = surface_root.join(format!("{}.output", binding.repository));
    let mut swept_stage_dirs = Vec::new();
    for entry in top_level.flatten() {
        let stage_dir = entry.path();
        if !stage_dir.is_dir() {
            continue;
        }
        let output_dir = stage_dir.join("output");
        if !output_dir.is_dir() {
            continue;
        }
        let Some(stage_name) = stage_dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if crate::domain::workflow::declared_output_disposition(workflow_source, stage_name)
            == crate::domain::workflow::OutputDisposition::Promote
        {
            continue;
        }
        let dest = dest_root.join(stage_name);
        let bytes = copy_declared_output_artifacts(&output_dir, &dest, &binding.worktree_path);
        if bytes == 0 {
            let _ = std::fs::remove_dir_all(&dest);
            continue;
        }
        report.evidence.push(RetainedStageOutput {
            stage: stage_name.to_string(),
            path: dest,
            bytes,
        });
        swept_stage_dirs.push(output_dir);
    }
    if swept_stage_dirs.is_empty() {
        return report;
    }
    for output_dir in &swept_stage_dirs {
        let _ = std::fs::remove_dir_all(output_dir);
    }
    // Scoped to exactly the removed `output/` dirs, not `-A`: an unscoped
    // add would silently fold any other dirty/untracked content the actor
    // left elsewhere in the worktree into this commit, laundering it clean
    // before `teardown`'s dirty-worktree check ever runs (F-IN-01).
    let add_ok = swept_stage_dirs.iter().all(|output_dir| {
        let rel = output_dir
            .strip_prefix(&binding.worktree_path)
            .unwrap_or(output_dir);
        git(
            &binding.worktree_path,
            &["add", "--", &rel.to_string_lossy()],
        )
        .is_ok()
    });
    let committed = add_ok
        && git(
            &binding.worktree_path,
            &[
                "-c",
                "user.name=sergeant finalize",
                "-c",
                "user.email=sergeant-finalize@localhost",
                "commit",
                "-m",
                "sergeant: finalize sweep — remove evidence-class stage outputs",
            ],
        )
        .is_ok();
    report.committed = committed;
    report
}

/// [`finalize_sweep`]'s report: what it retained as evidence, and whether it
/// actually made the finalize commit.
#[derive(Debug, Clone, Default)]
pub struct FinalizeSweepReport {
    /// One entry per stage the sweep actually copied evidence out of and
    /// removed from the worktree.
    pub evidence: Vec<RetainedStageOutput>,
    /// Whether the removal was committed. `false` when nothing was swept
    /// (no commit needed) or when `git add`/`git commit` itself failed.
    pub committed: bool,
}

/// Stage everything `.gitignore` does not exclude (`git add -A`), diff it
/// against `HEAD` (`git diff --cached --binary`), and write the result
/// durably under the surface root, sibling to the worktree it is about to
/// replace. `None` on any failure — a caller that gets `None` back must not
/// remove anything, since nothing was actually captured.
fn capture_dirty_patch(binding: &RepositoryBinding) -> Option<PatchInfo> {
    git(&binding.worktree_path, &["add", "-A"]).ok()?;
    // `git_bytes`, not `git`: `git diff`'s trailing newline is not
    // cosmetic — `git apply` treats its absence as a corrupt patch at
    // end-of-file (#234) — so the diff must reach `write_atomic` exactly as
    // Git wrote it, never round-tripped through a trimmed `String`.
    let diff = git_bytes(&binding.worktree_path, &["diff", "--cached", "--binary"]).ok()?;
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
    crate::runtime::fsutil::write_atomic(&path, &diff).ok()?;
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

/// What teardown observed and did for one binding: the disposition, plus the
/// §11 evidence [`teardown`] folds into the [`BindingTeardown`] record.
struct BindingOutcome {
    /// What happened to the worktree.
    disposition: BindingDisposition,
    /// The retained branch's tip.
    final_sha: Option<String>,
    /// Where the worktree's HEAD actually was, when it could be read.
    observed_head: Option<ObservedHead>,
    /// §11.3 findings attributable to this binding.
    findings: Vec<IntegrityFinding>,
}

/// Read what a worktree's HEAD actually points at (§11.7's first question).
///
/// A resolvable branch name is an attached HEAD; otherwise a resolvable
/// commit is a detached one. If *neither* resolves, this returns `None`
/// rather than inventing a classification: a worktree whose git cannot answer
/// either question is not "detached", it is unreadable, and teardown's own
/// `git status --porcelain` check below already fails that closed as a
/// `RetainedError` carrying git's diagnostic.
fn observe_head(worktree: &Path) -> Option<ObservedHead> {
    match git(worktree, &["symbolic-ref", "--quiet", "--short", "HEAD"]) {
        Ok(branch) if !branch.is_empty() => Some(ObservedHead::Branch {
            branch,
            sha: git(worktree, &["rev-parse", "HEAD"]).ok(),
        }),
        _ => git(worktree, &["rev-parse", "HEAD"])
            .ok()
            .map(|sha| ObservedHead::Detached { sha }),
    }
}

/// Whether any named ref in the source repository is **proven** to reach
/// `sha` (§11.7's detached-HEAD test).
///
/// `git branch --contains` answers it directly for local branches. The
/// second check is the same question asked of this binding's own work branch,
/// which `--contains` would also have answered — kept because it is the ref
/// §11.7 names explicitly and because it still answers if the first call
/// fails for a reason that has nothing to do with reachability.
///
/// Every failure path returns `false`. "Not proven reachable" is the
/// fail-closed answer, and the caller's response to `false` is to *keep* the
/// worktree, so an error here can only ever cost a retained directory —
/// never a lost commit.
fn reachable_from_a_named_ref(binding: &RepositoryBinding, sha: &str) -> bool {
    if matches!(
        git(&binding.source_path, &["branch", "--contains", sha]),
        Ok(ref out) if !out.is_empty()
    ) {
        return true;
    }
    git_succeeds(
        &binding.source_path,
        &[
            "merge-base",
            "--is-ancestor",
            sha,
            &format!("refs/heads/{}", binding.work_branch),
        ],
    )
}

/// §11.3's `assigned_common_dir_mismatch`: the worktree and the checkout it
/// is journaled against must agree on the Git common directory — the identity
/// §2.7 locks on and the one that says these two paths really do share an
/// object store.
///
/// Phase A asked the question at teardown, from both sides, which needed no
/// binding schema change. Phase B pins the canonical common dir at admission,
/// so the *expected* side now comes from the binding — a genuinely stronger
/// check, because it compares the worktree against what was admitted rather
/// than against a second live reading that would move with the mount. A
/// pre-enrichment binding (`None`, C3) falls back to Phase A's behaviour and
/// re-asks the source checkout, which is exactly as good an answer as it ever
/// was for those bindings.
///
/// A read that fails on either side yields no finding: this is the *identity*
/// check, and an unanswerable git is a different failure, already handled
/// fail-closed by the status/removal path below.
fn common_dir_finding(binding: &RepositoryBinding) -> Option<IntegrityFinding> {
    let expected = match &binding.canonical_common_dir {
        Some(admitted) => admitted.clone(),
        None => canonical_git_common_dir(&binding.source_path).ok()?,
    };
    let observed = canonical_git_common_dir(&binding.worktree_path).ok()?;
    if expected == observed {
        return None;
    }
    Some(IntegrityFinding::AssignedCommonDirMismatch {
        repository: binding.repository.clone(),
        evidence: format!(
            "the assigned worktree reports git common directory {} while its journaled source \
             checkout {} reports {}",
            observed.display(),
            binding.source_path.display(),
            expected.display()
        ),
        expected_common_dir: expected,
        observed_common_dir: observed,
    })
}

/// §11.4, bounded by amendment C6: one `git rev-parse HEAD` against this
/// binding's declared mount, compared with the commit the binding was cut
/// from. One call per bound repository, at retirement only — no worktree
/// walking, no `git status`, no unselected-repository reads, no polling.
///
/// Only `BindingOrigin::Cut` bindings are compared. For an `Attached`
/// binding, `base_sha` is carried over from the *target* Work's binding
/// (see [`BindingOrigin`]) and is therefore not this Work's own record of
/// what the mount was at admission — reporting a `before` that was never
/// this Work's base would be a worse answer than reporting nothing.
///
/// Drift in repositories this Work never selected is out of scope here by
/// construction: it needs the estate-bound daemon, and is Phase D's work.
fn observe_estate_drift(binding: &RepositoryBinding) -> Option<EstateDriftObservation> {
    if binding.origin != BindingOrigin::Cut {
        return None;
    }
    let observed = git(&binding.source_path, &["rev-parse", "HEAD"]).ok()?;
    if observed == binding.base_sha {
        return None;
    }
    Some(EstateDriftObservation {
        repository: binding.repository.clone(),
        before: binding.base_sha.clone(),
        observed,
        attribution: DriftAttribution::Unknown,
    })
}

fn teardown_binding(data_dir: &Path, binding: &RepositoryBinding) -> BindingOutcome {
    // One lock identity for this whole teardown, resolved before any span
    // opens — from the binding's own admission record when it has one, so the
    // lock released here is the lock materialization took.
    let gate = RepositoryGate::for_binding(data_dir, binding);
    // Read the retained branch's tip **before** taking the per-repository lock.
    // This call reads from ref storage only — it does not access the worktree
    // registry (`.git/worktrees/`) that `with_repository` protects, so running
    // it before the lock lets concurrent teardowns that target the same
    // repository do this work in parallel rather than serializing behind the
    // gate that only `git worktree remove` actually needs.
    //
    // Teardown always keeps the branch regardless of what happens to the
    // worktree below, so `final_sha` is available in every disposition —
    // including `Missing`, where there is no worktree left to read a HEAD from.
    let final_sha = git(
        &binding.source_path,
        &["rev-parse", &format!("refs/heads/{}", binding.work_branch)],
    )
    .ok();

    let mut findings = Vec::new();

    if !binding.worktree_path.exists() {
        // The directory is gone, but the source repository still lists it;
        // unregister it now so a later rematerialize at the same path is
        // possible.  `git worktree prune` modifies the registry → lock.
        //
        // Best-effort, and already was: `prune_stale_worktrees` discards its
        // own git failures. A lock we cannot take is one more way this prune
        // does not happen, and it changes nothing about the disposition below
        // — the worktree is `Missing` either way, and the next teardown or
        // rematerialize against this repository prunes what this one skipped.
        let _ = with_repository(&gate, || {
            prune_stale_worktrees(&binding.source_path);
        });
        findings.push(IntegrityFinding::AssignedWorktreeMissing {
            repository: binding.repository.clone(),
            worktree_path: binding.worktree_path.clone(),
            evidence: format!(
                "the assigned worktree {} did not exist when teardown ran",
                binding.worktree_path.display()
            ),
        });
        return BindingOutcome {
            disposition: BindingDisposition::Missing,
            final_sha,
            // Nothing left to read a HEAD from: not assessed, not agreed.
            observed_head: None,
            findings,
        };
    }

    // §11.7's actual-vs-expected reconciliation, **before** the dirty check
    // below, because the dirty check cannot see any of it: `git status
    // --porcelain` answers "is there uncommitted content here", never "is
    // this still the branch the binding was assigned" (§2.6 / #173). Both
    // reads run in the worktree and touch only `.git/HEAD` and ref storage,
    // not the `.git/worktrees/` registry `with_repository` protects, so they
    // stay outside the lock like every other read on this path.
    let observed_head = observe_head(&binding.worktree_path);
    findings.extend(common_dir_finding(binding));
    match &observed_head {
        // A clean worktree on the *wrong* named branch is recorded and then
        // removed exactly like any other clean worktree (§11.7). Both named
        // branches stay durable: teardown deletes neither the expected
        // `sergeant/<work-id>` nor the one that was actually used, so the
        // commits on the observed branch remain reachable after the worktree
        // that pointed at them is gone — which is what makes recording the
        // branch and its tip here the whole fix rather than half of one.
        Some(ObservedHead::Branch { branch, sha }) if *branch != binding.work_branch => {
            findings.push(IntegrityFinding::AssignedBranchMismatch {
                repository: binding.repository.clone(),
                worktree_path: binding.worktree_path.clone(),
                expected_branch: binding.work_branch.clone(),
                expected_sha: final_sha.clone(),
                observed_branch: branch.clone(),
                observed_sha: sha.clone(),
                evidence: format!(
                    "the assigned worktree ended on branch {branch} rather than {}; both \
                     branches are retained",
                    binding.work_branch
                ),
            });
        }
        Some(ObservedHead::Detached { sha }) => {
            let reachable = reachable_from_a_named_ref(binding, sha);
            let evidence = format!(
                "the assigned worktree ended with HEAD detached at {sha}; \
                 `git branch --contains` and `merge-base --is-ancestor {}` \
                 {} a named ref that reaches it",
                binding.work_branch,
                if reachable { "found" } else { "found no" }
            );
            findings.push(IntegrityFinding::AssignedHeadDetachedOrUnreferenced {
                repository: binding.repository.clone(),
                worktree_path: binding.worktree_path.clone(),
                expected_branch: binding.work_branch.clone(),
                observed_sha: sha.clone(),
                reachable,
                evidence: evidence.clone(),
            });
            if !reachable {
                // Fail closed, and *before* the dirty check on purpose: the
                // dirty path removes the worktree with `--force` once its
                // content is captured as a patch, and a patch captures
                // uncommitted content, never commits. Removing this worktree
                // would leave the detached commits reachable from nothing at
                // all. Retaining it keeps its HEAD as a gc root, so nothing
                // possible output is destroyed.
                return BindingOutcome {
                    disposition: BindingDisposition::RetainedUnreferenced {
                        head_sha: sha.clone(),
                        evidence,
                    },
                    final_sha,
                    observed_head,
                    findings,
                };
            }
        }
        _ => {}
    }

    // Check whether the worktree is clean **before** taking the per-repository
    // lock.  `git status --porcelain` runs in the linked worktree and reads
    // the shared object store (read-only) plus the worktree-local index — it
    // does not touch `.git/worktrees/`, so it is safe outside the lock.
    //
    // TOCTOU note: if the worktree becomes dirty between this read and the
    // `git worktree remove` below, git's own pre-removal dirty check fires and
    // refuses the removal, surfacing a `RetainedError` rather than silently
    // destroying content — the same fail-closed outcome the explicit status
    // check produces, so the window carries no correctness risk.
    match git(&binding.worktree_path, &["status", "--porcelain"]) {
        Ok(changes) if !changes.is_empty() => {
            // Dirty: `retain_dirty` captures a patch and calls `git worktree
            // remove --force`, which does touch the registry.  Hold the lock
            // for that entire path.
            //
            // The finding rides *alongside* the existing retention machinery
            // rather than replacing any of it (§11.7's first bullet): the
            // disposition still says what was done with the content, the
            // finding says what it means for the Work.
            findings.push(IntegrityFinding::AssignedWorktreeUncommitted {
                repository: binding.repository.clone(),
                worktree_path: binding.worktree_path.clone(),
                evidence: changes.clone(),
            });
            // A lock we cannot take means the `--force` removal `retain_dirty`
            // ends with must not be attempted at all: another process may be
            // rewriting this registry right now. Fail closed exactly as an
            // unanswerable `git status` does below — retained, with the
            // refusal itself as the recorded evidence.
            let disposition = with_repository(&gate, || retain_dirty(binding, changes))
                .unwrap_or_else(|e| BindingDisposition::RetainedError {
                    detail: e.to_string(),
                });
            return BindingOutcome {
                disposition,
                final_sha,
                observed_head,
                findings,
            };
        }
        Ok(_) => {} // clean — proceed to the removal below
        Err(e) => {
            // Cannot establish that it is clean ⇒ must not remove it. No
            // §11.3 finding: the closed vocabulary has no kind for "git
            // could not answer", and inventing `assigned_worktree_uncommitted`
            // here would report content nothing observed. `TeardownReport::
            // integrity` still fails this closed through `clean`.
            return BindingOutcome {
                disposition: BindingDisposition::RetainedError {
                    detail: e.to_string(),
                },
                final_sha,
                observed_head,
                findings,
            };
        }
    }

    // Clean path: only `git worktree remove` touches the registry.
    let path = binding.worktree_path.display().to_string();
    let disposition = with_repository(&gate, || {
        match git(&binding.source_path, &["worktree", "remove", &path]) {
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
        }
    })
    // Same fail-closed rule as every other arm here: a removal that was never
    // attempted retains the worktree and records why.
    .unwrap_or_else(|e| BindingDisposition::RetainedError {
        detail: e.to_string(),
    });
    BindingOutcome {
        disposition,
        final_sha,
        observed_head,
        findings,
    }
}

/// Ensure the parent directory of `worktree` exists and return the path as a
/// display string, ready to pass to `git worktree add`.
///
/// Both [`add_worktree_from`] and [`add_worktree_no_checkout`] need the same
/// directory-creation guard before their `git` invocation; this helper keeps
/// the logic in one place.
fn prepare_worktree_dir(worktree: &Path) -> Result<String, SurfaceError> {
    if let Some(parent) = worktree.parent() {
        create_dir_all_durable(parent).map_err(|source| SurfaceError::Io {
            path: parent.display().to_string(),
            source,
        })?;
    }
    Ok(worktree.display().to_string())
}

fn add_worktree_from(
    source: &Path,
    worktree: &Path,
    branch: &str,
    start: &str,
    create_branch: bool,
) -> Result<(), SurfaceError> {
    let path = prepare_worktree_dir(worktree)?;
    let args: Vec<&str> = if create_branch {
        vec!["worktree", "add", "-b", branch, &path, start]
    } else {
        vec!["worktree", "add", &path, start]
    };
    git(source, &args)?;
    Ok(())
}

/// `git worktree add --no-checkout`: create the branch, write the registry
/// entry, and set HEAD in the new worktree — but do *not* populate the
/// working-tree files. Callers must follow up with [`checkout_worktree`].
///
/// Holding only the registry write under the per-repository lock (see
/// [`with_repository`]) lets the file-level checkout — which touches only the
/// object store (read-only) and the new worktree directory — run after the
/// lock releases and overlap with the next submission's registry write.
fn add_worktree_no_checkout(
    source: &Path,
    worktree: &Path,
    branch: &str,
    start: &str,
) -> Result<(), SurfaceError> {
    let path = prepare_worktree_dir(worktree)?;
    git(
        source,
        &[
            "worktree",
            "add",
            "--no-checkout",
            "-b",
            branch,
            &path,
            start,
        ],
    )?;
    Ok(())
}

/// Populate the working tree of a worktree created with
/// `git worktree add --no-checkout`.
///
/// `--no-checkout` leaves both the index and the working tree empty; only HEAD
/// (a symref to the work branch) and the `.git/worktrees/<name>/` registry
/// entry are written.  `git reset --hard HEAD` fills the index from HEAD's
/// tree and writes those files to disk — the minimal operation that brings the
/// worktree to a usable state.
///
/// `git reset --hard` is chosen over `git checkout HEAD` for two reasons:
/// - `checkout HEAD` resolves `HEAD` as a commitish and performs a
///   commit-level switch, which **detaches HEAD** from the work branch.
///   Commits made in a detached-HEAD worktree do not advance the branch,
///   silently losing run output on teardown.
/// - `reset --hard HEAD` leaves the symref intact (HEAD stays → work branch).
///
/// This call touches neither the worktree registry (`.git/worktrees/`) nor
/// the source repository's ref storage, so it is safe to run after the
/// per-repository lock releases and overlap with the next submission's
/// registry write.
fn checkout_worktree(worktree: &Path) -> Result<(), SurfaceError> {
    git(worktree, &["reset", "--hard", "HEAD"])?;
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

    /// The data dir a fixture surface belongs to.
    ///
    /// Every fixture in this module hands `materialize` one tempdir as *both*
    /// the data dir and the surfaces root — the daemon's own default
    /// relationship (`<data_dir>/surfaces`) collapsed by one level, which
    /// keeps a fixture to a single directory. A surface root is
    /// `<surfaces_root>/<work-id>`, so its parent is that directory.
    ///
    /// The two being genuinely independent (R-MVP1-1: an estate may put its
    /// surfaces anywhere, and §9.4's locks still live in the daemon's own
    /// storage) is pinned separately, by
    /// [`the_repository_lock_lives_under_the_data_dir_not_the_surfaces_root`].
    fn fixture_data_dir(surface: &WorkSurface) -> &Path {
        surface
            .root
            .parent()
            .expect("a fixture surface root always has a parent")
    }

    /// [`teardown`] against the data dir this fixture surface belongs to.
    fn teardown_of(surface: &WorkSurface) -> TeardownReport {
        teardown(fixture_data_dir(surface), surface)
    }

    /// [`rematerialize`] against the data dir this fixture surface belongs to.
    fn rematerialize_of(surface: &WorkSurface) -> Result<WorkSurface, SurfaceError> {
        rematerialize(fixture_data_dir(surface), surface)
    }

    /// [`reap`] against the data dir this fixture surface belongs to.
    fn reap_of(surface: &WorkSurface, report: &TeardownReport) -> ReapReport {
        reap(fixture_data_dir(surface), surface, report)
    }

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
        // Unlike the rest of this module's fixtures, the data dir and the
        // surfaces root are kept apart here: the emptiness assertion below is
        // about the *surfaces* root, and §9.4's `locks/repo/` lives in the
        // daemon's storage. This is also the production relationship, one
        // level flattened (`<data_dir>/surfaces`).
        let data = dir.path().join("data");
        let surfaces = dir.path().join("surfaces");
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
                        let surfaces = surfaces.clone();
                        let spec = spec.clone();
                        scope.spawn(move || {
                            let work_id = format!("01CONCURRENT{round}{i:03}");
                            let surface = materialize(
                                &data,
                                &surfaces,
                                &work_id,
                                std::slice::from_ref(&spec),
                            )
                            .unwrap_or_else(|e| panic!("materialize {work_id}: {e}"));
                            teardown(&data, &surface)
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
            // `surfaces` is handed to `materialize` directly as the surfaces
            // root (R-MVP1-1: no implicit `SURFACES_DIR` nesting inside this
            // module any more — that join happens once, at `Engine`'s
            // default computation, not here).
            assert!(
                !surfaces.exists()
                    || std::fs::read_dir(&surfaces)
                        .expect("surfaces root")
                        .next()
                        .is_none(),
                "no surface root survives a clean teardown (round {round} of {ROUNDS})"
            );
        }
    }

    /// §2.7 (Phase 0 pin #5, flipped by Phase B): a checkout and its own
    /// linked worktree are **one** lock identity, at both layers.
    ///
    /// A primary checkout and a `git worktree add` of it are two different,
    /// canonicalize-stable paths pointing at one shared `.git` — a linked
    /// worktree's `commondir` file points back to it — so a lock keyed on the
    /// checkout *path* handed the module gave them one lock each, which is the
    /// same "two workspaces can reach one repository by different routes"
    /// hazard `repository_lock`'s own doc comment already named for symlinks,
    /// just reached through a worktree instead. Nothing serialized concurrent
    /// registry mutations between them, defeating the guard this module's
    /// header describes (measured burst-50 fatal: `.git/worktrees/<other-
    /// work>/commondir`).
    ///
    /// Both halves are asserted, because the fix has two halves: the
    /// in-process mutex is now keyed on the common directory (one `Arc`), and
    /// the interprocess lock file derived from that identity is one file. A
    /// regression in either would let the two paths past each other.
    #[test]
    fn a_checkout_and_its_own_linked_worktree_are_one_repository_lock() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = dir.path().join("data");
        let primary = repo(&dir.path().join("primary"));
        let linked = dir.path().join("linked-worktree");
        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                linked.to_str().expect("utf8 path"),
                "-b",
                "linked-branch",
            ])
            .current_dir(&primary.path)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .expect("git worktree add");
        assert!(output.status.success(), "worktree add: {output:?}");

        // The fixture premise: these two paths really do share one git common
        // directory. Without this the rest of the test would pass vacuously.
        assert_eq!(
            canonical_git_common_dir(&primary.path).expect("common dir from the primary checkout"),
            canonical_git_common_dir(&linked).expect("common dir from the linked worktree"),
            "the fixture must actually share one git common directory"
        );

        let primary_gate = RepositoryGate::derive(&data, &primary.path);
        let linked_gate = RepositoryGate::derive(&data, &linked);

        // Layer 1: the in-process fast layer resolves to the same mutex.
        assert!(
            Arc::ptr_eq(
                &repository_lock(&primary_gate.identity),
                &repository_lock(&linked_gate.identity),
            ),
            "a primary checkout and its own linked worktree share one worktree registry, so \
             they must share one in-process lock: keyed on the canonical git common directory, \
             not on the checkout path each caller happened to name"
        );

        // Layer 2: and the correctness boundary — one lock file, so the same
        // holds against another process, not merely another thread.
        assert_eq!(
            repolock::lock_path(&data, &primary_gate.identity),
            repolock::lock_path(&data, &linked_gate.identity),
            "and one interprocess lock file, which is what makes this hold across processes"
        );

        // A genuinely different repository still gets its own lock: the point
        // is one identity per registry, not one gate for all of git.
        let other = repo(&dir.path().join("other"));
        let other_gate = RepositoryGate::derive(&data, &other.path);
        assert!(
            !Arc::ptr_eq(
                &repository_lock(&primary_gate.identity),
                &repository_lock(&other_gate.identity),
            ),
            "unrelated repositories must still proceed in parallel"
        );
        assert_ne!(
            repolock::lock_path(&data, &primary_gate.identity),
            repolock::lock_path(&data, &other_gate.identity),
        );
    }

    /// Regression for `common_dir_finding` (this module, above): a linked
    /// worktree's git common directory still agrees with its source
    /// checkout's, so teardown's integrity check reports no mismatch. Unlike
    /// the codex adapter's own fixture of the same underlying git fact
    /// (`tests/codex_backend.rs`'s
    /// `the_git_admin_dir_grant_does_not_disturb_the_worktrees_own_common_dir_identity`,
    /// which never calls this private function and so cannot regress it),
    /// this test calls `common_dir_finding` directly — a real fix to it
    /// (wrong comparison, wrong field, always-`Some`) fails this test.
    #[test]
    fn common_dir_finding_reports_no_mismatch_for_a_genuine_linked_worktree() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let source = repo(&dir.path().join("source"));
        let worktree = dir.path().join("worktree");
        git_as_test_identity(
            &source.path,
            &[
                "worktree",
                "add",
                worktree.to_str().expect("utf8 path"),
                "-b",
                "sergeant/w-common-dir",
            ],
        );

        let admitted = canonical_git_common_dir(&source.path).expect("source common dir");
        let binding = RepositoryBinding {
            repository: "solo".to_string(),
            source_path: source.path.clone(),
            base_branch: Some("main".to_string()),
            base_sha: "0".repeat(40),
            worktree_path: worktree.clone(),
            work_branch: "sergeant/w-common-dir".to_string(),
            head_sha: "0".repeat(40),
            origin: BindingOrigin::default(),
            canonical_top_level: None,
            canonical_common_dir: Some(admitted),
            preflight: None,
        };
        assert_eq!(
            common_dir_finding(&binding),
            None,
            "a genuine linked worktree's common dir must agree with what was admitted"
        );

        // And the pre-Phase-B fallback (`canonical_common_dir: None`), which
        // re-derives the expected side from `source_path` live instead of
        // trusting a recorded value.
        let binding_pre_phase_b = RepositoryBinding {
            canonical_common_dir: None,
            ..binding
        };
        assert_eq!(
            common_dir_finding(&binding_pre_phase_b),
            None,
            "the pre-Phase-B fallback (re-asking the source checkout live) must also agree"
        );
    }

    /// R-MVP1-1 left the surfaces root free to live anywhere; §9.4's locks
    /// still belong to the daemon's own storage. The two are separate
    /// parameters and this is the test that keeps them separate — every other
    /// fixture in this module collapses them into one tempdir.
    #[test]
    fn the_repository_lock_lives_under_the_data_dir_not_the_surfaces_root() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = dir.path().join("data");
        let surfaces = dir.path().join("somewhere-else/surfaces");
        let spec = repo(&dir.path().join("solo"));

        let surface = materialize(
            &data,
            &surfaces,
            "01SPLITROOTS",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        assert!(
            surface.root.starts_with(&surfaces),
            "the surface goes where the surfaces root says: {}",
            surface.root.display()
        );

        let common = surface.bindings[0]
            .canonical_common_dir
            .clone()
            .expect("a freshly materialized binding records its common directory");
        let lock = repolock::lock_path(&data, &common);
        assert!(
            lock.is_file(),
            "materializing takes the repository lock, so its file exists under the data dir: {}",
            lock.display()
        );
        assert!(
            !lock.starts_with(&surfaces),
            "and not under the surfaces root"
        );
        assert!(
            !lock.starts_with(&spec.path),
            "and never inside the user's checkout (§6.2 / surface doctrine)"
        );

        let report = teardown(&data, &surface);
        assert!(report.clean, "teardown across split roots: {report:?}");
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

    /// Amendment C3: every journal change is additive. A `surface.materialized`
    /// payload written before Phase E has no `preflight` key and a plain
    /// string `base_branch`, and both must replay as exactly the facts they
    /// recorded — nothing invented, and "no preflight ran" distinguishable
    /// from "preflight waived nothing".
    #[test]
    fn a_pre_phase_e_binding_replays_with_no_preflight_evidence() {
        let payload = serde_json::json!({
            "repository": "api",
            "source_path": "/estate/repos/api",
            "base_branch": "main",
            "base_sha": "a".repeat(40),
            "worktree_path": "/data/surfaces/01OLD/api",
            "work_branch": "sergeant/01OLD",
            "head_sha": "a".repeat(40),
        });
        let binding: RepositoryBinding =
            serde_json::from_value(payload).expect("a pre-Phase-E binding still deserializes");
        assert_eq!(binding.base_branch.as_deref(), Some("main"));
        assert_eq!(
            binding.preflight, None,
            "absent means no preflight ran — never an empty evidence record, which would \
             claim preflight ran and waived nothing"
        );
        assert_eq!(binding.origin, BindingOrigin::Cut);
        assert_eq!(binding.canonical_top_level, None);

        // And the detached sentinel that field used to carry replays as the
        // string it was, rather than being silently reinterpreted as absence.
        let detached = serde_json::json!({
            "repository": "api",
            "source_path": "/estate/repos/api",
            "base_branch": "(detached)",
            "base_sha": "b".repeat(40),
            "worktree_path": "/data/surfaces/01OLD/api",
            "work_branch": "sergeant/01OLD",
            "head_sha": "b".repeat(40),
        });
        let binding: RepositoryBinding = serde_json::from_value(detached).expect("replays");
        assert_eq!(binding.base_branch.as_deref(), Some("(detached)"));
    }

    /// The same rule for the plan: a `surface.materializing` from before
    /// Phase E carries no admission at all, and materializing from it falls
    /// back to reading the mount — the only thing it could ever do.
    #[test]
    fn a_pre_phase_e_surface_plan_replays_with_no_admission() {
        let payload = serde_json::json!({
            "root": "/data/surfaces/01OLD",
            "work_branch": "sergeant/01OLD",
            "repositories": [{"name": "api", "path": "/estate/repos/api"}],
        });
        let plan: SurfacePlan = serde_json::from_value(payload).expect("replays");
        assert!(plan.admitted.is_empty());
        assert!(!plan.override_git_preflight);
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

        let err = materialize(
            &data_dir_inside,
            &data_dir_inside,
            "01WORKID",
            std::slice::from_ref(&spec),
        )
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
        let surface = materialize(
            data.path(),
            data.path(),
            "01WORKID",
            std::slice::from_ref(&spec),
        )
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
        let surface = materialize(
            data.path(),
            data.path(),
            "01GONE",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");

        let worktree = surface.bindings[0].worktree_path.display().to_string();
        std::fs::remove_dir_all(&surface.bindings[0].worktree_path).expect("remove worktree");
        assert!(
            git(&spec.path, &["worktree", "list"])
                .expect("list")
                .contains(&worktree),
            "git still lists the vanished worktree until something unregisters it"
        );

        let report = teardown_of(&surface);
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
        let rebuilt = rematerialize_of(&surface)
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

        let surface = materialize(
            data.path(),
            data.path(),
            "01STALE",
            std::slice::from_ref(&outer),
        )
        .expect("materialize a repository with a submodule");
        let worktree = surface.bindings[0].worktree_path.clone();

        // Uncommitted work: teardown retains it whole, and therefore never
        // prunes.
        std::fs::write(worktree.join("half-done.rs"), "fn main() {}\n").expect("dirty");
        let report = teardown_of(&surface);
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

        let rebuilt = rematerialize_of(&surface).expect("retry must still be able to rebuild");
        assert!(rebuilt.bindings[0].worktree_path.is_dir());
        assert_eq!(
            git(&worktree, &["rev-parse", "--abbrev-ref", "HEAD"]).expect("head"),
            work_branch("01STALE"),
            "the rebuilt worktree is back on the retained branch"
        );
        // And it is repeatable: a second rebuild after a second deletion
        // works too, so nothing accumulates that eventually wedges it.
        std::fs::remove_dir_all(&worktree).expect("delete again");
        rematerialize_of(&surface).expect("still rebuildable");
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

        let err = materialize(
            data.path(),
            data.path(),
            "01PARTIAL",
            &[first.clone(), second],
        )
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

        let err = materialize(
            &data_dir_via_link,
            &data_dir_via_link,
            "01LINK",
            std::slice::from_ref(&spec),
        )
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
        let solo = materialize(
            data.path(),
            data.path(),
            "01ROOT",
            std::slice::from_ref(&first),
        )
        .expect("materialize solo");
        let root = solo.root.clone();
        assert!(root.is_dir(), "materialize created the root");
        let report = teardown_of(&solo);
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
        let again = teardown_of(&solo);
        assert_eq!(again.bindings[0].disposition, BindingDisposition::Missing);
        assert!(!root.exists());

        // Two repositories: the root survives until the last worktree is
        // gone. Tearing down a surface that names only the first binding
        // leaves the second worktree — and therefore the root — untouched.
        let pair = materialize(data.path(), data.path(), "01PAIR", &[first, second])
            .expect("materialize pair");
        let pair_root = pair.root.clone();
        let partial = WorkSurface {
            bindings: pair.bindings[..1].to_vec(),
            ..pair.clone()
        };
        teardown_of(&partial);
        assert!(
            pair_root.is_dir(),
            "a root still holding another repository's worktree must be kept"
        );
        assert!(pair.bindings[1].worktree_path.is_dir(), "untouched");
        teardown_of(&pair);
        assert!(
            !pair_root.exists(),
            "the last binding's removal takes the root with it"
        );

        // A retained worktree keeps its root: teardown fails closed, and the
        // root is where the retained *state* lives. #109: not the whole
        // directory — that gets reclaimed once its dirty state is captured
        // as a patch, which is exactly what keeps the root non-empty.
        let dirty_spec = repo(&dir.path().join("dirty"));
        let dirty = materialize(
            data.path(),
            data.path(),
            "01DIRTY",
            std::slice::from_ref(&dirty_spec),
        )
        .expect("materialize dirty");
        std::fs::write(dirty.bindings[0].worktree_path.join("wip.rs"), "//\n").expect("dirty");
        let report = teardown_of(&dirty);
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
        let surface = materialize(
            data.path(),
            data.path(),
            "01FINALIZE",
            std::slice::from_ref(&spec),
        )
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

        let report = teardown_of(&surface);
        assert_eq!(report.bindings[0].disposition, BindingDisposition::Removed);
        assert_eq!(
            report.bindings[0].final_sha.as_deref(),
            Some(finalize_commit.as_str()),
            "teardown must record the branch tip as it stood at teardown time, \
             including a finalize commit made before it ran"
        );

        // A worktree the repository never had (Missing) still resolves the
        // branch tip: it lives on the branch, not the worktree.
        let again = materialize(
            data.path(),
            data.path(),
            "01FINALIZE2",
            std::slice::from_ref(&spec),
        )
        .expect("materialize again");
        std::fs::remove_dir_all(&again.bindings[0].worktree_path).expect("simulate vanished");
        let missing_report = teardown_of(&again);
        assert_eq!(
            missing_report.bindings[0].disposition,
            BindingDisposition::Missing
        );
        assert!(
            missing_report.bindings[0].final_sha.is_some(),
            "a vanished worktree still has a branch tip to report"
        );
    }

    /// #173, closed: a worktree that switched off its own `work_branch`
    /// before teardown is reconciled and reported, not silently absorbed.
    ///
    /// This was Phase 0's contract pin. Teardown used to read `final_sha`
    /// from `refs/heads/<work_branch>` — deliberately, so it resolves even
    /// for a `Missing` binding (see
    /// `teardown_captures_the_retained_branchs_tip_as_the_finalize_commit`
    /// above) — and ask `git status --porcelain` only "is the worktree
    /// dirty", never "is it still on the branch this binding claims". A
    /// worktree that checks out a different branch and commits there is
    /// clean by that check and leaves `work_branch` exactly where it
    /// started, so the report was a clean `Removed` at the base SHA:
    /// indistinguishable from a surface nothing ever touched, while
    /// `git worktree remove` deleted the only record of where the run had
    /// actually ended up.
    ///
    /// §11.7's answer, asserted below: the removal still happens (a clean
    /// worktree on the wrong branch is still clean), but the divergence is
    /// recorded first — `observed_head` names the branch and its tip, an
    /// `assigned_branch_mismatch` finding carries expected-vs-observed, and
    /// the report's integrity disposition is `dirty`. Both named branches
    /// survive (§12.1), which is what makes recording the observed one a
    /// usable pointer rather than an epitaph.
    #[test]
    fn teardown_reports_a_worktree_that_switched_off_its_work_branch() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01SWITCH",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();
        let work_branch_name = surface.bindings[0].work_branch.clone();
        let base_sha = surface.bindings[0].base_sha.clone();

        // Inside the linked worktree, switch to a different branch entirely
        // and commit there — never touching `work_branch`.
        git_as_test_identity(&worktree, &["checkout", "-b", "renegade"]);
        std::fs::write(worktree.join("elsewhere.txt"), "not on work_branch\n")
            .expect("write elsewhere");
        git_as_test_identity(&worktree, &["add", "."]);
        git_as_test_identity(
            &worktree,
            &["commit", "-m", "work happened on the wrong branch"],
        );
        let renegade_commit = git(&worktree, &["rev-parse", "HEAD"]).expect("renegade commit");
        assert_ne!(
            renegade_commit, base_sha,
            "the fixture must actually produce a divergent commit"
        );

        let report = teardown_of(&surface);
        let binding = &report.bindings[0];

        // A clean worktree on the wrong branch is still removed (§11.7): the
        // content is committed and both branches are durable, so there is
        // nothing here to fail closed over.
        assert_eq!(binding.disposition, BindingDisposition::Removed);
        assert!(!worktree.exists(), "a clean worktree is still removed");

        // What used to be lost with it: where HEAD actually was.
        assert_eq!(
            binding.observed_head,
            Some(ObservedHead::Branch {
                branch: "renegade".to_string(),
                sha: Some(renegade_commit.clone()),
            }),
            "teardown must record the branch the worktree actually ended on, and its tip"
        );

        // `final_sha` keeps its documented meaning — the *retained branch's*
        // tip, still at the base SHA because nothing advanced it — and the
        // finding is what carries the divergence rather than overloading it.
        assert_eq!(binding.final_sha.as_deref(), Some(base_sha.as_str()));
        let [
            IntegrityFinding::AssignedBranchMismatch {
                repository,
                expected_branch,
                expected_sha,
                observed_branch,
                observed_sha,
                ..
            },
        ] = binding.findings.as_slice()
        else {
            panic!("expected exactly one branch-mismatch finding: {binding:?}");
        };
        assert_eq!(repository, "solo");
        assert_eq!(expected_branch, &work_branch_name);
        assert_eq!(expected_sha.as_deref(), Some(base_sha.as_str()));
        assert_eq!(observed_branch, "renegade");
        assert_eq!(observed_sha.as_deref(), Some(renegade_commit.as_str()));

        assert_eq!(
            report.integrity(),
            IntegrityDisposition::Dirty,
            "a branch mismatch makes the Work terminal-dirty (§11.5)"
        );

        // §11.7/§12.1: both named branches remain durable. Teardown deletes
        // neither the expected one nor the one that was actually used, so
        // the commit stays reachable and the report names the ref it is on.
        for branch in [work_branch_name.as_str(), "renegade"] {
            assert!(
                git_succeeds(
                    &spec.path,
                    &["rev-parse", "--verify", &format!("refs/heads/{branch}")]
                ),
                "{branch} must still exist in the source repository after teardown"
            );
        }
        assert_ne!(
            work_branch_name, "renegade",
            "sanity: the binding's own work_branch was never the one switched to"
        );
    }

    /// The baseline the rest of §11 is measured against: a surface that did
    /// what it was asked reports `clean`, no findings, and a HEAD sitting on
    /// exactly the branch the binding named.
    ///
    /// Worth its own test because "clean" is now a *computed* claim rather
    /// than the absence of one — a reconciliation that produced spurious
    /// findings would make every ordinary Work terminal-dirty, which is the
    /// failure mode that turns a signal into noise (#94's own lesson).
    #[test]
    fn an_ordinary_teardown_reports_clean_integrity_on_the_expected_branch() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01CLEAN",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let base_sha = surface.bindings[0].base_sha.clone();

        let report = teardown_of(&surface);
        let binding = &report.bindings[0];

        assert_eq!(binding.disposition, BindingDisposition::Removed);
        assert_eq!(
            binding.observed_head,
            Some(ObservedHead::Branch {
                branch: work_branch("01CLEAN"),
                sha: Some(base_sha),
            })
        );
        assert!(
            binding.findings.is_empty(),
            "an untouched surface must produce no findings: {:?}",
            binding.findings
        );
        assert!(report.drift.is_empty(), "the mount never moved");
        assert_eq!(report.integrity(), IntegrityDisposition::Clean);
    }

    /// §11.7's fail-closed case: a detached HEAD carrying commits no named
    /// ref reaches **retains** the worktree.
    ///
    /// Removing it would drop the last reference to those commits — a
    /// worktree's HEAD is a gc root, a removed worktree's is not — and
    /// teardown never destroys possible output. The patch capture the dirty
    /// path uses is no substitute here: it captures uncommitted content, and
    /// these commits are exactly the opposite. `sgt work reap` must decline
    /// for the same reason, with the remedy named rather than a refusal an
    /// operator has to decode.
    #[test]
    fn teardown_retains_a_worktree_whose_detached_head_no_named_ref_reaches() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01DETACHED",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        // Detach, then commit: the commit belongs to no branch at all.
        git_as_test_identity(&worktree, &["checkout", "--detach"]);
        std::fs::write(worktree.join("orphan.txt"), "unreferenced output\n").expect("write orphan");
        git_as_test_identity(&worktree, &["add", "."]);
        git_as_test_identity(&worktree, &["commit", "-m", "committed off any branch"]);
        let orphan = git(&worktree, &["rev-parse", "HEAD"]).expect("orphan commit");
        assert!(
            git(&spec.path, &["branch", "--contains", &orphan])
                .expect("branch --contains")
                .is_empty(),
            "the fixture must actually produce a commit no branch contains"
        );

        let report = teardown_of(&surface);
        let binding = &report.bindings[0];

        assert!(
            matches!(
                &binding.disposition,
                BindingDisposition::RetainedUnreferenced { head_sha, .. } if head_sha == &orphan
            ),
            "expected the worktree to be retained: {:?}",
            binding.disposition
        );
        assert!(
            worktree.exists(),
            "the worktree must survive — its HEAD is the only thing referencing the commit"
        );
        assert!(
            git_succeeds(&spec.path, &["cat-file", "-e", &orphan]),
            "the orphaned commit must still be in the object store"
        );
        let [
            IntegrityFinding::AssignedHeadDetachedOrUnreferenced {
                observed_sha,
                reachable,
                ..
            },
        ] = binding.findings.as_slice()
        else {
            panic!("expected one detached-HEAD finding: {binding:?}");
        };
        assert_eq!(observed_sha, &orphan);
        assert!(!reachable, "no named ref reaches it");
        assert_eq!(report.integrity(), IntegrityDisposition::Dirty);

        // #109's inspect verb sees it, and reap declines rather than doing
        // the one thing teardown refused to do.
        let retained = retained_bindings(&report);
        assert_eq!(retained.len(), 1);
        assert_eq!(retained[0].reason, "retained_unreferenced");
        let reaped = reap_of(&surface, &report);
        assert!(
            matches!(
                &reaped.bindings[0].outcome,
                ReapOutcome::Skipped { reason } if reason.contains(&orphan)
            ),
            "reap must decline and name the commit: {:?}",
            reaped.bindings[0].outcome
        );
        assert!(worktree.exists(), "reap must not have removed it either");
    }

    /// The other half of §11.7's detached-HEAD rule: detachment is always a
    /// finding, but a commit a named ref still reaches is not at risk, so the
    /// worktree is removed exactly as any other clean one is.
    ///
    /// Without this half the fail-closed rule would retain every worktree a
    /// workflow happened to leave detached at its own branch tip — honest
    /// but useless, and #109's 30 GB of retained surfaces all over again.
    #[test]
    fn teardown_removes_a_worktree_whose_detached_head_is_still_reachable() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01REACHABLE",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        // Detached, but at the work branch's own tip: nothing is at risk.
        git_as_test_identity(&worktree, &["checkout", "--detach"]);

        let report = teardown_of(&surface);
        let binding = &report.bindings[0];

        assert_eq!(binding.disposition, BindingDisposition::Removed);
        assert!(!worktree.exists());
        let [IntegrityFinding::AssignedHeadDetachedOrUnreferenced { reachable, .. }] =
            binding.findings.as_slice()
        else {
            panic!("expected one detached-HEAD finding: {binding:?}");
        };
        assert!(reachable, "the work branch itself reaches this commit");
        assert_eq!(
            report.integrity(),
            IntegrityDisposition::Dirty,
            "a detached HEAD is still attributable, even when nothing was lost"
        );
    }

    /// A worktree that vanished before teardown reached it keeps its
    /// `Missing` disposition — and now maps to the §11.3 finding for it, so
    /// "the surface was not where it was supposed to be" is reportable rather
    /// than only inferable from a disposition tag.
    #[test]
    fn a_vanished_worktree_maps_to_the_missing_finding_and_reads_dirty() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01VANISHED",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        std::fs::remove_dir_all(&surface.bindings[0].worktree_path).expect("simulate vanished");

        let report = teardown_of(&surface);
        let binding = &report.bindings[0];

        assert_eq!(binding.disposition, BindingDisposition::Missing);
        assert!(
            binding.observed_head.is_none(),
            "there is no HEAD left to read: not assessed, not agreed"
        );
        assert!(matches!(
            binding.findings.as_slice(),
            [IntegrityFinding::AssignedWorktreeMissing { .. }]
        ));
        assert_eq!(report.integrity(), IntegrityDisposition::Dirty);
    }

    /// #109's retention machinery is untouched by §11: an uncommitted
    /// worktree still captures its patch and still reports `retained_dirty`.
    /// The finding rides *alongside* it — the disposition says what was done
    /// with the content, the finding says what it means for the Work.
    #[test]
    fn an_uncommitted_worktree_yields_the_finding_beside_the_existing_retention() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01DIRTY",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        std::fs::write(surface.bindings[0].worktree_path.join("wip.rs"), "//\n").expect("dirty");

        let report = teardown_of(&surface);
        let binding = &report.bindings[0];

        assert!(
            matches!(
                binding.disposition,
                BindingDisposition::RetainedDirty { patch: Some(_), .. }
            ),
            "#109's patch capture must be unchanged: {:?}",
            binding.disposition
        );
        let [IntegrityFinding::AssignedWorktreeUncommitted { evidence, .. }] =
            binding.findings.as_slice()
        else {
            panic!("expected one uncommitted finding: {binding:?}");
        };
        assert!(
            evidence.contains("wip.rs"),
            "the finding carries git's own porcelain evidence: {evidence}"
        );
        assert_eq!(report.integrity(), IntegrityDisposition::Dirty);
    }

    /// §11.3's `assigned_common_dir_mismatch`: a worktree that answers with a
    /// different canonical Git common directory than the checkout it is
    /// journaled against is not the worktree the binding thinks it is — the
    /// same identity §2.7/§9.4 needs the interprocess lock keyed on.
    ///
    /// Reproduced by handing teardown a binding that names a *different*
    /// repository than the worktree actually belongs to, which is exactly the
    /// shape a mis-journaled or aliased mount would have.
    ///
    /// Since Phase B the expected side is the identity the binding *recorded
    /// at admission* (§3.5), not a second live reading of `source_path` — a
    /// stronger check, because it compares the worktree against what was
    /// admitted rather than against a mount that may have moved since. The
    /// mis-binding here therefore rewrites both halves of the binding's
    /// repository identity, which is what a mis-journaled binding really
    /// looks like.
    #[test]
    fn a_worktree_whose_common_dir_disagrees_with_its_source_checkout_is_a_finding() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let mine = repo(&dir.path().join("mine"));
        let stranger = repo(&dir.path().join("stranger"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01COMMONDIR",
            std::slice::from_ref(&mine),
        )
        .expect("materialize");

        let mut misbound = surface.clone();
        misbound.bindings[0].source_path = stranger.path.clone();
        misbound.bindings[0].canonical_common_dir =
            Some(canonical_git_common_dir(&stranger.path).expect("stranger common dir"));

        let report = teardown_of(&misbound);
        let binding = &report.bindings[0];

        assert!(
            binding
                .findings
                .iter()
                .any(|f| matches!(f, IntegrityFinding::AssignedCommonDirMismatch { .. })),
            "expected a common-dir mismatch: {:?}",
            binding.findings
        );
        assert_eq!(report.integrity(), IntegrityDisposition::Dirty);
        // Fail closed all the way through: the worktree belongs to a
        // repository this teardown was not pointed at, so nothing removed it.
        assert!(surface.bindings[0].worktree_path.exists());
    }

    /// The same finding, from a binding journaled before Phase B existed
    /// (C3: `canonical_common_dir` replays as `None`).
    ///
    /// With nothing admitted to compare against, the check falls back to
    /// Phase A's behaviour — re-ask the journaled `source_path` — which is
    /// exactly as good an answer as it ever was for those bindings, and is
    /// the guarantee that widening the binding did not quietly switch off a
    /// finding for every Work that predates the widening.
    #[test]
    fn a_pre_enrichment_binding_still_gets_the_common_dir_finding_from_its_source_path() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let mine = repo(&dir.path().join("mine"));
        let stranger = repo(&dir.path().join("stranger"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01OLDCOMMONDIR",
            std::slice::from_ref(&mine),
        )
        .expect("materialize");

        let mut misbound = surface.clone();
        misbound.bindings[0].source_path = stranger.path.clone();
        // The shape an old journal replays as: no admitted identity at all.
        misbound.bindings[0].canonical_common_dir = None;
        misbound.bindings[0].canonical_top_level = None;

        let report = teardown_of(&misbound);
        assert!(
            report.bindings[0]
                .findings
                .iter()
                .any(|f| matches!(f, IntegrityFinding::AssignedCommonDirMismatch { .. })),
            "expected a common-dir mismatch from the source-path fallback: {:?}",
            report.bindings[0].findings
        );
        assert_eq!(report.integrity(), IntegrityDisposition::Dirty);
    }

    /// §3.5 + C3: what a fresh binding records, and what an old one replays
    /// as.
    ///
    /// The two canonical identities are additive `Option` fields, so a
    /// `surface.materialized` payload written before Phase B — the exact JSON
    /// shape this test embeds — must still deserialize, and must come back
    /// saying *nothing observed this*, never a fabricated path.
    #[test]
    fn a_binding_records_its_canonical_identities_and_an_old_payload_replays_without_them() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01IDENTITY",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let binding = &surface.bindings[0];

        assert_eq!(
            binding.canonical_top_level.as_deref(),
            Some(
                canonical_git_top_level(&spec.path)
                    .expect("top level")
                    .as_path()
            ),
            "a fresh binding records §3.5's canonical top level"
        );
        assert_eq!(
            binding.canonical_common_dir.as_deref(),
            Some(
                canonical_git_common_dir(&spec.path)
                    .expect("common dir")
                    .as_path()
            ),
            "and the canonical common directory §2.7 locks on"
        );

        // A `surface.materialized` payload from before Phase B: every field
        // the old shape had, and neither of the new ones.
        let old = serde_json::json!({
            "repository": "solo",
            "source_path": "/repos/solo",
            "base_branch": "main",
            "base_sha": "0".repeat(40),
            "worktree_path": "/data/surfaces/01OLD/solo",
            "work_branch": "sergeant/01OLD",
            "head_sha": "1".repeat(40),
        });
        let replayed: RepositoryBinding =
            serde_json::from_value(old).expect("an old binding must still replay");
        assert_eq!(replayed.repository, "solo");
        assert_eq!(replayed.origin, BindingOrigin::Cut);
        assert_eq!(
            replayed.canonical_top_level, None,
            "nothing observed this at the time; it must not be invented"
        );
        assert_eq!(replayed.canonical_common_dir, None);

        // And the enriched shape round-trips as itself.
        let round_tripped: RepositoryBinding =
            serde_json::from_value(serde_json::to_value(binding).expect("serialize"))
                .expect("deserialize");
        assert_eq!(&round_tripped, binding);

        let report = teardown_of(&surface);
        assert!(report.clean, "{report:?}");
    }

    /// C3, the other half: a whole `surface.materialized` payload in the
    /// pre-enrichment shape replays into a `WorkSurface` that teardown can
    /// still act on — and a teardown driven from it locks the right
    /// repository, because [`RepositoryGate::for_binding`] falls back to
    /// deriving from `source_path` when the binding admitted nothing.
    #[test]
    fn an_old_shape_materialized_payload_replays_and_tears_down() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01OLDREPLAY",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");

        // Journal it, then strip the fields Phase B added — byte-for-byte the
        // payload a pre-Phase-B daemon would have written.
        let mut payload = serde_json::to_value(&surface).expect("serialize surface");
        for binding in payload["bindings"]
            .as_array_mut()
            .expect("bindings array")
            .iter_mut()
        {
            let binding = binding.as_object_mut().expect("binding object");
            binding.remove("canonical_top_level");
            binding.remove("canonical_common_dir");
            binding.remove("origin");
        }

        let replayed: WorkSurface =
            serde_json::from_value(payload).expect("an old surface payload must still replay");
        assert_eq!(replayed.bindings[0].canonical_common_dir, None);
        assert_eq!(replayed.bindings[0].origin, BindingOrigin::Cut);

        let report = teardown(data.path(), &replayed);
        assert!(
            report.clean,
            "a surface replayed from an old payload still tears down cleanly: {report:?}"
        );
        assert!(!surface.bindings[0].worktree_path.exists());
    }

    /// §11.4, bounded by amendment C6: a mount whose committed HEAD moved
    /// during the Work window is *observed* — one `rev-parse` per bound
    /// repository, at retirement — and attributed to nobody.
    ///
    /// The assertion that matters most is the negative one: drift does not
    /// make the Work dirty. Captain, an editor, another Work, or an unrelated
    /// process may have advanced that mount, and reporting a Work dirty for
    /// something it cannot be shown to have done is exactly the false signal
    /// §11.4 exists to prevent.
    #[test]
    fn estate_drift_is_observed_at_teardown_and_never_makes_the_work_dirty() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01DRIFT",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let base_sha = surface.bindings[0].base_sha.clone();

        // Somebody else advances the mount while the Work runs.
        std::fs::write(spec.path.join("unrelated.txt"), "not this work\n").expect("write");
        git_as_test_identity(&spec.path, &["add", "."]);
        git_as_test_identity(&spec.path, &["commit", "-m", "somebody else's commit"]);
        let moved = git(&spec.path, &["rev-parse", "HEAD"]).expect("moved head");

        let report = teardown_of(&surface);

        assert_eq!(
            report.drift,
            vec![EstateDriftObservation {
                repository: "solo".to_string(),
                before: base_sha,
                observed: moved,
                attribution: DriftAttribution::Unknown,
            }]
        );
        assert!(report.bindings[0].findings.is_empty());
        assert_eq!(
            report.integrity(),
            IntegrityDisposition::Clean,
            "estate drift is reported, never attributed, and never dirty (§11.4)"
        );
    }

    /// A surface with no repositories could never execute anything.
    #[test]
    fn a_surface_needs_at_least_one_repository() {
        let data = tempfile::TempDir::new().expect("tempdir");
        assert!(matches!(
            materialize(data.path(), data.path(), "01EMPTY", &[]),
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

        let surface = materialize(
            data.path(),
            data.path(),
            "01SUBMODULE",
            std::slice::from_ref(&outer),
        )
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

        let report = teardown_of(&surface);
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

        let err = materialize(
            data.path(),
            data.path(),
            "01BADSUBMODULE",
            std::slice::from_ref(&outer),
        )
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
        // `teardown_binding` uses for git's blanket "containing submodules"
        // refusal cleans it up rather than leaving it parked with a permanent
        // `RetainedError`.
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

    /// Fix 3 — `--no-checkout` + `checkout_worktree` cleanup (L7 pin).
    ///
    /// After `add_worktree_no_checkout` creates a branch ref and a
    /// `.git/worktrees/<name>/` registry entry, `checkout_worktree`
    /// (`git reset --hard HEAD`) populates the working tree.  If that second
    /// step fails, `materialize_one` must clean up both the registry entry
    /// **and the branch** before returning the error — otherwise a retry would
    /// call `git worktree add -b <same-branch> …` and fail because the branch
    /// already exists, permanently wedging the work.
    ///
    /// The trigger: a smudge filter configured as `required` that always exits
    /// non-zero.  Git runs smudge filters when checking out files; with
    /// `required = true` a non-zero exit is a hard error on the whole reset.
    /// `git worktree add --no-checkout` skips the checkout, so the filter does
    /// not fire there — only during `checkout_worktree`.
    ///
    /// The key assertion (the pin): after the failed materialize, a second
    /// call to materialize for the same work-id must succeed.  It can only
    /// succeed if the branch was deleted — `git worktree add -b` fails if the
    /// branch already exists.
    #[test]
    fn a_checkout_failure_after_no_checkout_add_does_not_strand_the_branch() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");

        // Build a repository whose checkout step always fails: a file subject
        // to a required smudge filter that always exits non-zero.
        let source = dir.path().join("source");
        std::fs::create_dir_all(&source).expect("source dir");
        // Phase 1: create the repo and the initial commit WITHOUT the filter
        // configured — the filter's `required = true` would also block `git add`
        // (the clean step) if it were active during setup.
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&source)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .expect("git");
        // .gitattributes declares the filter; payload.txt is the file it targets.
        std::fs::write(
            source.join(".gitattributes"),
            "payload.txt filter=always-fail\n",
        )
        .expect(".gitattributes");
        std::fs::write(source.join("payload.txt"), "content\n").expect("payload.txt");
        for args in [vec!["add", "."], vec!["commit", "-m", "initial"]] {
            let out = Command::new("git")
                .args(&args)
                .current_dir(&source)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_SYSTEM", "/dev/null")
                .output()
                .expect("git");
            assert!(out.status.success(), "git {args:?}: {out:?}");
        }
        // Phase 2: arm the smudge filter AFTER the commit so it does not
        // interfere with the clean-side of `git add`.  The filter is written
        // to the local repo config so sergeant's `git reset --hard HEAD`
        // (which does not suppress local config) reads it.
        for args in [
            vec!["config", "filter.always-fail.smudge", "false"],
            vec!["config", "filter.always-fail.required", "true"],
        ] {
            let out = Command::new("git")
                .args(&args)
                .current_dir(&source)
                .output()
                .expect("git config");
            assert!(out.status.success(), "git {args:?}: {out:?}");
        }
        let spec = RepositorySpec {
            name: "source".to_string(),
            path: source.to_path_buf(),
        };

        // First attempt: add_worktree_no_checkout succeeds (no checkout →
        // smudge filter not triggered), checkout_worktree fails (smudge
        // filter exits 1), cleanup runs.
        let err = materialize(
            data.path(),
            data.path(),
            "01CHECKFAIL",
            std::slice::from_ref(&spec),
        )
        .expect_err("checkout_worktree failure must propagate as an error");
        assert!(
            matches!(err, SurfaceError::Git { .. }),
            "expected a git error from the reset --hard, got: {err}"
        );

        // The branch add_worktree_no_checkout created must be gone.  Without
        // the `git branch -D` cleanup, the retry below would hit:
        //   fatal: A branch named 'sergeant/01CHECKFAIL' already exists.
        let branch = work_branch("01CHECKFAIL");
        assert!(
            !git_succeeds(
                &source,
                &[
                    "show-ref",
                    "--verify",
                    "--quiet",
                    &format!("refs/heads/{branch}")
                ]
            ),
            "branch must be deleted by the checkout-failure cleanup — if it remains, \
             a retry fails to re-create it and the work is wedged permanently"
        );

        // Remove the failing filter so the retry can actually complete.
        for args in [
            vec!["config", "--unset", "filter.always-fail.smudge"],
            vec!["config", "--unset", "filter.always-fail.required"],
        ] {
            Command::new("git")
                .args(&args)
                .current_dir(&source)
                .output()
                .expect("git config --unset");
        }

        // Retry: must succeed because the branch slot was freed.
        let surface = materialize(
            data.path(),
            data.path(),
            "01CHECKFAIL",
            std::slice::from_ref(&spec),
        )
        .expect(
            "retry must succeed — branch was deleted so it can be recreated with -b; \
                 if this fails with 'branch already exists', the cleanup is missing",
        );
        assert!(surface.bindings[0].worktree_path.is_dir());
        teardown_of(&surface);
    }

    /// The force-retry `teardown_binding` uses for git's blanket "containing
    /// submodules" refusal must never bypass real dirtiness —
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

        let report = teardown_of(&surface);
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

        let target = materialize(
            data.path(),
            data.path(),
            "01TARGET",
            std::slice::from_ref(&spec),
        )
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

        let target_report = teardown_of(&target);
        assert!(
            target_report.clean,
            "the fixture must reach the clean-teardown precondition: {target_report:?}"
        );

        let gate = attach(
            data.path(),
            data.path(),
            "01GATE",
            "01TARGET",
            &target.bindings,
        )
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

        let gate_report = teardown_of(&gate);
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

        let target = materialize(
            data.path(),
            data.path(),
            "01LIVE",
            std::slice::from_ref(&spec),
        )
        .expect("materialize target");
        // No teardown: the target's worktree is still live on its branch.

        let err = attach(
            data.path(),
            data.path(),
            "01GATE",
            "01LIVE",
            &target.bindings,
        )
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
        let report = teardown_of(&target);
        assert!(report.clean);
        attach(
            data.path(),
            data.path(),
            "01GATE2",
            "01LIVE",
            &target.bindings,
        )
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

        let target = materialize(
            data.path(),
            data.path(),
            "01GONEBRANCH",
            std::slice::from_ref(&spec),
        )
        .expect("materialize target");
        let report = teardown_of(&target);
        assert!(report.clean);

        let branch = target.bindings[0].work_branch.clone();
        git(&spec.path, &["branch", "-D", &branch]).expect("delete the branch out of band");

        let err = attach(
            data.path(),
            data.path(),
            "01GATE",
            "01GONEBRANCH",
            &target.bindings,
        )
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

        let target = materialize(
            data.path(),
            data.path(),
            "01COLLIDE",
            std::slice::from_ref(&spec),
        )
        .expect("materialize target");
        let report = teardown_of(&target);
        assert!(report.clean);

        let colliding_path = data.path().join("01GATE").join("solo");
        std::fs::create_dir_all(&colliding_path).expect("pre-create the colliding directory");
        std::fs::write(colliding_path.join("occupied.txt"), "already here\n")
            .expect("occupy the path");

        let err = attach(
            data.path(),
            data.path(),
            "01GATE",
            "01COLLIDE",
            &target.bindings,
        )
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
            data.path(),
            "01PARTIALATTACH",
            &[first.clone(), second.clone()],
        )
        .expect("materialize target");
        let report = teardown_of(&target);
        assert!(report.clean);

        // The second repository's branch is gone by the time the gate
        // attaches — attach_one for "second" must fail after "first" already
        // succeeded.
        let branch = work_branch("01PARTIALATTACH");
        git(&second.path, &["branch", "-D", &branch]).expect("delete second's branch");

        let err = attach(
            data.path(),
            data.path(),
            "01GATE",
            "01PARTIALATTACH",
            &target.bindings,
        )
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

        let target = materialize(
            data.path(),
            data.path(),
            "01SUBTARGET",
            std::slice::from_ref(&spec),
        )
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

        let target_report = teardown_of(&target);
        assert!(
            target_report.clean,
            "the fixture must reach the clean-teardown precondition: {target_report:?}"
        );

        let err = attach(
            data.path(),
            data.path(),
            "01GATE",
            "01SUBTARGET",
            &target.bindings,
        )
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
            attach(data.path(), data.path(), "01EMPTY", "01TARGET", &[]),
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
        let surface = materialize(
            data.path(),
            data.path(),
            "01SCOPE",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
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

        let report = teardown_of(&surface);
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
        let surface = materialize(
            data.path(),
            data.path(),
            "01LOCKED",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
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

        let report = teardown_of(&surface);
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
        let surface = materialize(
            data.path(),
            data.path(),
            "01INSPECT",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        std::fs::write(surface.bindings[0].worktree_path.join("wip.rs"), "//\n").expect("dirty");
        let report = teardown_of(&surface);

        let error_binding = BindingTeardown {
            repository: "other".to_string(),
            worktree_path: dir.path().join("some-other-worktree"),
            work_branch: "sergeant/01INSPECT".to_string(),
            final_sha: None,
            observed_head: None,
            findings: Vec::new(),
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

    /// #234: a captured `.dirty.patch` must be byte-for-byte a patch `git
    /// apply` accepts, trailing newline included — not merely "looks right
    /// when printed as a `String`". `git diff` always terminates its output
    /// with `\n`; `capture_dirty_patch` going through `git()` (which
    /// unconditionally trims) silently ate that byte, so every retained
    /// patch failed `git apply` with "corrupt patch at line N", always at
    /// end-of-file (22/22 real-estate patches, per #234's evidence).
    ///
    /// Applied against a *second*, independent clean checkout of the same
    /// source repository at the same base commit — never the worktree the
    /// diff came from — so this actually proves the patch is portable, the
    /// only way it is ever used in practice (`sgt work reap` hands it to an
    /// operator, not back to the worktree that no longer exists).
    #[test]
    fn retain_dirty_writes_a_patch_git_apply_accepts() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        // A tracked file whose last diff line ends in `]` with no trailing
        // whitespace of its own — the shape #234 pinned as the one that
        // reliably exposed the missing final newline. Committed into the
        // *source* repo, before materializing, so the base commit both the
        // worktree and the independent checkout share already has it.
        std::fs::write(spec.path.join("data.rs"), "const V: [u8; 1] = [0];\n").expect("tracked");
        git_as_test_identity(&spec.path, &["add", "data.rs"]);
        git_as_test_identity(&spec.path, &["commit", "-m", "add data.rs"]);

        let surface = materialize(
            data.path(),
            data.path(),
            "01PATCHNL",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();
        let base_sha = surface.bindings[0].base_sha.clone();

        std::fs::write(worktree.join("data.rs"), "const V: [u8; 2] = [0, 1]").expect("dirty edit");
        // Plus an untracked file, #109's other half of the capture.
        std::fs::write(worktree.join("new.rs"), "fn wanted() {}\n").expect("untracked");

        let report = teardown_of(&surface);
        let BindingDisposition::RetainedDirty { patch, .. } = &report.bindings[0].disposition
        else {
            panic!(
                "expected RetainedDirty: {:?}",
                report.bindings[0].disposition
            );
        };
        let patch = patch.as_ref().expect("must capture a patch");

        let patch_bytes = std::fs::read(&patch.path).expect("read captured patch");
        assert_eq!(
            patch_bytes.last().copied(),
            Some(b'\n'),
            "a captured .dirty.patch must end with the newline `git diff` always writes, or \
             `git apply` rejects it as corrupt at end-of-file: {patch_bytes:?}"
        );

        // A second, independent clean checkout of the same source repo at
        // the same base commit — the patch must apply there, not just be
        // well-formed in the abstract.
        let target = dir.path().join("independent-checkout");
        git_as_test_identity(
            dir.path(),
            &[
                "clone",
                spec.path.to_str().expect("utf8 path"),
                target.to_str().expect("utf8 path"),
            ],
        );
        git_as_test_identity(&target, &["checkout", &base_sha]);

        let check = Command::new("git")
            .args(["apply", "--check", patch.path.to_str().expect("utf8 path")])
            .current_dir(&target)
            .output()
            .expect("git apply --check");
        assert!(
            check.status.success(),
            "git apply --check rejected the captured patch: {}",
            String::from_utf8_lossy(&check.stderr)
        );

        let apply = Command::new("git")
            .args(["apply", patch.path.to_str().expect("utf8 path")])
            .current_dir(&target)
            .output()
            .expect("git apply");
        assert!(
            apply.status.success(),
            "git apply rejected the captured patch: {}",
            String::from_utf8_lossy(&apply.stderr)
        );

        let applied = std::fs::read(target.join("data.rs")).expect("applied tracked file");
        assert_eq!(
            applied,
            b"const V: [u8; 2] = [0, 1]".to_vec(),
            "applying the patch must reproduce the dirtied worktree's tracked-file content \
             byte-for-byte"
        );
        let new_file = std::fs::read(target.join("new.rs")).expect("applied untracked file");
        assert_eq!(new_file, b"fn wanted() {}\n".to_vec());
    }

    /// #240: a stage's declared `output/` artifacts (Rule 4,
    /// `docs/icm/convention.md`) must survive a dirty teardown as their own
    /// retrievable, per-stage-labeled copy — not just bundled, undiscoverably,
    /// inside the flat `.dirty.patch` alongside every other change. Two
    /// stage directories, one with a real artifact plus its `README.md`
    /// declaration, one with only the declaration — the second must not
    /// produce a retained-output entry at all (nothing real was written).
    #[test]
    fn dirty_teardown_retains_each_stages_declared_output_artifacts() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));

        let surface = materialize(
            data.path(),
            data.path(),
            "01OUTPUTS",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        // `20-panel`: a declaration plus a real artifact the run actually
        // wrote — this one must be retained.
        let panel_output = worktree.join("20-panel").join("output");
        std::fs::create_dir_all(&panel_output).expect("panel output dir");
        std::fs::write(panel_output.join("README.md"), "declares findings.md")
            .expect("panel README");
        std::fs::write(panel_output.join("findings.md"), "four-axis findings")
            .expect("panel artifact");

        // `10-hypothesize`: only the declaration, nothing real ever written —
        // this one must not show up as retained output.
        let hyp_output = worktree.join("10-hypothesize").join("output");
        std::fs::create_dir_all(&hyp_output).expect("hypothesize output dir");
        std::fs::write(hyp_output.join("README.md"), "declares hypotheses.md")
            .expect("hypothesize README");

        // Also touch a tracked file so the worktree is dirty and teardown
        // takes the `RetainedDirty` path at all.
        std::fs::write(worktree.join("wip.rs"), "// still working\n").expect("dirty edit");

        let report = teardown_of(&surface);
        let BindingDisposition::RetainedDirty { patch, outputs, .. } =
            &report.bindings[0].disposition
        else {
            panic!(
                "expected RetainedDirty: {:?}",
                report.bindings[0].disposition
            );
        };
        assert!(patch.is_some(), "the flat patch must still be captured too");

        assert_eq!(
            outputs.len(),
            1,
            "only the stage with a real artifact should be retained: {outputs:?}"
        );
        let panel = &outputs[0];
        assert_eq!(panel.stage, "20-panel");

        // The worktree is gone — the copy is the only place this is left.
        assert!(!worktree.exists());
        let retained_findings = std::fs::read_to_string(panel.path.join("findings.md"))
            .expect("retained findings.md must be readable after teardown");
        assert_eq!(retained_findings, "four-axis findings");
        assert!(
            !panel.path.join("README.md").exists(),
            "the declaration itself is not an artifact and must not be copied"
        );

        // `retained_bindings` (the `sgt work retained` read side) must
        // surface it too, tagged distinctly from the flat patch.
        let bindings = retained_bindings(&report);
        assert!(
            bindings
                .iter()
                .any(|b| b.reason == "retained_output" && b.path == panel.path),
            "retained_bindings must report the stage-output copy: {bindings:?}"
        );
        assert!(
            bindings.iter().any(|b| b.reason == "retained_dirty"),
            "the flat patch must still be reported alongside it: {bindings:?}"
        );
    }

    /// W6 (F-IN-01): `copy_declared_output_artifacts` must descend into
    /// subdirectories of a stage's `output/`, not just copy files sitting
    /// directly inside it — convention.md does not forbid a stage from
    /// writing a nested artifact tree, and a flat-only copy silently drops
    /// those files from retention with no error.
    #[test]
    fn dirty_teardown_retains_a_stages_nested_output_artifacts() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));

        let surface = materialize(
            data.path(),
            data.path(),
            "01NESTED",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        let panel_output = worktree.join("20-panel").join("output");
        let nested = panel_output.join("evidence");
        std::fs::create_dir_all(&nested).expect("nested output dir");
        std::fs::write(panel_output.join("README.md"), "declares findings.md")
            .expect("panel README");
        std::fs::write(nested.join("transcript.log"), "nested artifact").expect("nested file");

        std::fs::write(worktree.join("wip.rs"), "// still working\n").expect("dirty edit");

        let report = teardown_of(&surface);
        let BindingDisposition::RetainedDirty { outputs, .. } = &report.bindings[0].disposition
        else {
            panic!(
                "expected RetainedDirty: {:?}",
                report.bindings[0].disposition
            );
        };
        assert_eq!(
            outputs.len(),
            1,
            "the nested artifact must be retained: {outputs:?}"
        );
        let retained =
            std::fs::read_to_string(outputs[0].path.join("evidence").join("transcript.log"))
                .expect("nested artifact must survive teardown at its original relative path");
        assert_eq!(retained, "nested artifact");
    }

    /// W6 (F-IN-02): `copy_declared_output_artifacts` must exclude files
    /// Git would refuse to track (gitignored), the same exclusion
    /// `capture_dirty_patch` gets for free from `git add -A` — otherwise a
    /// stage that drops a gitignored file into `output/` reintroduces the
    /// disk-bloat risk R4/#109 was written to prevent.
    #[test]
    fn dirty_teardown_excludes_gitignored_files_from_retained_output() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));

        let surface = materialize(
            data.path(),
            data.path(),
            "01IGNORED",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        let panel_output = worktree.join("20-panel").join("output");
        std::fs::create_dir_all(&panel_output).expect("panel output dir");
        std::fs::write(panel_output.join("README.md"), "declares findings.md")
            .expect("panel README");
        std::fs::write(panel_output.join("findings.md"), "real artifact").expect("real artifact");
        std::fs::write(panel_output.join(".gitignore"), "junk.bin\n").expect("gitignore");
        std::fs::write(panel_output.join("junk.bin"), "should never be retained")
            .expect("ignored artifact");

        std::fs::write(worktree.join("wip.rs"), "// still working\n").expect("dirty edit");

        let report = teardown_of(&surface);
        let BindingDisposition::RetainedDirty { outputs, .. } = &report.bindings[0].disposition
        else {
            panic!(
                "expected RetainedDirty: {:?}",
                report.bindings[0].disposition
            );
        };
        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].path.join("findings.md").is_file());
        assert!(
            !outputs[0].path.join("junk.bin").exists(),
            "a gitignored file must not be copied into retained output"
        );
    }

    /// W6 (F-IN-03): if the patch removal step of `reap_binding` fails with
    /// a real (non-`NotFound`) error, the already-copied stage-output
    /// retention must not have been destroyed first — otherwise `Failed`
    /// would hide that irreversible data loss already happened. Forces the
    /// failure by making the patch's parent directory read-only so
    /// `remove_file` is denied, then asserts the output copy is still on
    /// disk after `reap_binding` reports `Failed`.
    #[test]
    fn reap_failing_to_remove_the_patch_does_not_destroy_the_output_copy_first() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            "01REAPORD",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        let panel_output = worktree.join("20-panel").join("output");
        std::fs::create_dir_all(&panel_output).expect("panel output dir");
        std::fs::write(panel_output.join("findings.md"), "real artifact").expect("real artifact");
        std::fs::write(worktree.join("wip.rs"), "// still working\n").expect("dirty edit");

        let report = teardown_of(&surface);
        let BindingDisposition::RetainedDirty {
            patch: Some(info),
            outputs,
            ..
        } = &report.bindings[0].disposition
        else {
            panic!(
                "expected a captured patch and retained output: {:?}",
                report.bindings[0].disposition
            );
        };
        assert_eq!(outputs.len(), 1);
        let output_path = outputs[0].path.clone();
        assert!(output_path.join("findings.md").is_file());

        let patch_dir = info
            .path
            .parent()
            .expect("patch has a parent dir")
            .to_path_buf();
        let original_perms = std::fs::metadata(&patch_dir)
            .expect("stat patch dir")
            .permissions();
        let mut locked = original_perms.clone();
        locked.set_mode(0o555);
        std::fs::set_permissions(&patch_dir, locked).expect("lock patch dir");

        let outcome = reap_binding(fixture_data_dir(&surface), &surface, &report.bindings[0]);

        // Restore permissions before any assertion can panic and leave the
        // tempdir uncleanable.
        std::fs::set_permissions(&patch_dir, original_perms).expect("restore patch dir perms");

        assert!(
            matches!(outcome, ReapOutcome::Failed { .. }),
            "expected patch removal to fail: {outcome:?}"
        );
        assert!(
            output_path.join("findings.md").is_file(),
            "the output copy must survive a failed patch removal, not be destroyed first"
        );
    }

    /// #234 follow-up (`60-re-verify-and-postmortem`, F-SF-04/F-TH-01): the
    /// fix's stated rationale for switching to `git_bytes` was not only the
    /// dropped trailing newline but also that a `String::from_utf8_lossy`
    /// round-trip corrupts non-UTF-8 diff content by substituting the
    /// 3-byte U+FFFD sequence for any invalid byte — a second, independent
    /// defect (root-cause H2). No prior regression test exercised it; every
    /// fixture was pure ASCII. This one dirties a tracked file with a raw
    /// `0xFF` byte (no `NUL` nearby, so Git still emits a text diff rather
    /// than switching to `--binary`'s base85 encoding, which is ASCII and
    /// would not exercise this path) and proves the captured patch is
    /// byte-identical to Git's own diff output, not merely non-empty.
    #[test]
    fn retain_dirty_patch_preserves_non_utf8_bytes_verbatim() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let spec = repo(&dir.path().join("solo"));
        std::fs::write(spec.path.join("data.rs"), b"const V: u8 = 0;\n").expect("tracked");
        git_as_test_identity(&spec.path, &["add", "data.rs"]);
        git_as_test_identity(&spec.path, &["commit", "-m", "add data.rs"]);

        let surface = materialize(
            data.path(),
            data.path(),
            "01PATCHUTF8",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        let mut dirtied = b"const V: u8 = ".to_vec();
        dirtied.push(0xFF);
        dirtied.extend_from_slice(b";\n");
        std::fs::write(worktree.join("data.rs"), &dirtied).expect("dirty edit");

        // Ground truth, captured independently of `capture_dirty_patch`:
        // stage now and read git's own diff bytes before teardown does the
        // same staging/diffing itself and reclaims the worktree.
        git_as_test_identity(&worktree, &["add", "-A"]);
        let raw = Command::new("git")
            .args(["diff", "--cached", "--binary"])
            .current_dir(&worktree)
            .output()
            .expect("git diff")
            .stdout;
        assert!(
            raw.contains(&0xFFu8),
            "sanity: the ground-truth diff must actually contain the raw non-UTF-8 byte"
        );

        let report = teardown_of(&surface);
        let BindingDisposition::RetainedDirty { patch, .. } = &report.bindings[0].disposition
        else {
            panic!(
                "expected RetainedDirty: {:?}",
                report.bindings[0].disposition
            );
        };
        let patch = patch.as_ref().expect("must capture a patch");
        let patch_bytes = std::fs::read(&patch.path).expect("read captured patch");

        assert_eq!(
            patch_bytes, raw,
            "the captured .dirty.patch must be byte-identical to git's own diff output — a \
             lossy UTF-8 round-trip would replace the raw 0xFF byte with the 3-byte U+FFFD \
             sequence and change both the content and the length"
        );
        assert!(
            !patch_bytes
                .windows(3)
                .any(|window| window == [0xEF, 0xBF, 0xBD]),
            "the raw non-UTF-8 byte must survive capture unchanged, not be replaced by the \
             U+FFFD substitution sequence: {patch_bytes:?}"
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
        let surface = materialize(
            data.path(),
            data.path(),
            "01REAP",
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        std::fs::write(surface.bindings[0].worktree_path.join("wip.rs"), "//\n").expect("dirty");
        let mut report = teardown_of(&surface);
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
            observed_head: None,
            findings: Vec::new(),
            disposition: BindingDisposition::RetainedError {
                detail: "fatal: could not read status".to_string(),
            },
        });

        let reaped = reap_of(&surface, &report);
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

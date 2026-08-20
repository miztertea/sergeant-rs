//! Git admission preflight (proposal §8, §15; estate-root Phase E).
//!
//! §8.1 is a list of eleven facts that must hold for *every* selected
//! repository "before `work.submitted` or any Git mutation", and §8.4 says who
//! owns checking them:
//!
//! > AGENTS.md teaches Captain what state to prepare and reconcile, but
//! > correctness does not depend on Captain remembering a checklist. The
//! > API/daemon repeats and authoritatively enforces the mechanical contract.
//!
//! So this module is core-owned and runs from [`Engine::plan`](
//! crate::runtime::engine::Engine::plan) — the one place a submission is fully
//! resolved and nothing has been created yet. Its refusal is an
//! [`EngineError`](crate::runtime::engine::EngineError), which the API already
//! answers as a structured 422; the Work record is minted only if this
//! returns `Ok`.
//!
//! # What it does not do
//!
//! §6.4: "Sergeant does not fetch, pull, reset, rebase, switch to main, or
//! infer a remote default during Work admission." Every git invocation below
//! is a question — `rev-parse`, `symbolic-ref`, `status --porcelain`,
//! `show-ref`, `worktree list` — and there is no verb here that writes to the
//! mounted checkout at all. `tests/e_admission_uses_no_network_git.rs` holds
//! that claim to a recording git shim across a whole real admission rather
//! than to this comment.
//!
//! # Reuse, not a second opinion
//!
//! Checks 1–4 are exactly what [`estate::validate_mount`] already decides
//! while resolving a manifest (Phase D); this calls that same function per
//! *selected* repository and maps its three error variants onto the four
//! checks, so there is one implementation of "is this mount really this
//! estate's own ordinary checkout" and not two that can drift. Check 5 is
//! Phase B's [`repolock::acquire`] plus the same
//! [`fs_locking::detect_for_path`] reliability gate the daemon applies to its
//! own data dir (amendment C4). Only checks 6–11 are new here.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::estate::{Estate, EstateError, MANIFEST_FILE, RepositorySpec, validate_mount};
use crate::platform::fs_locking::{self, Reliability};
use crate::runtime::git::{canonical_git_common_dir, git, git_succeeds};
use crate::runtime::repolock;
use crate::runtime::surface::{surface_root, work_branch};

/// The eleven §8.1 checks, one variant each.
///
/// The taxonomy is the deliverable, not an implementation detail: §15 requires
/// "stable structured API codes plus human remedies", and a refusal that could
/// only say "preflight failed" would name no remedy at all. Every variant
/// carries its own [`code`](Self::code) — the string a client branches on —
/// and its own [`remedy`](Self::remedy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreflightCheck {
    /// 1. The derived mount path exists.
    MountPresent,
    /// 2. The path is not a symlink or alias outside the estate-owned mount.
    MountNotAliased,
    /// 3. `git rev-parse --show-toplevel` resolves exactly to the canonical
    ///    mount.
    MountTopLevel,
    /// 4. The mount is an ordinary primary checkout, not a linked worktree
    ///    admitted as a repository source.
    MountPrimaryCheckout,
    /// 5. The canonical Git common directory is resolvable and lockable.
    CommonDirLockable,
    /// 6. HEAD is attached to a named local branch. **Waivable** (§8.3).
    HeadAttached,
    /// 7. HEAD resolves to a full commit SHA.
    HeadFullSha,
    /// 8. `git status --porcelain` is empty for index and working tree.
    ///    **Waivable** (§8.3).
    WorktreeClean,
    /// 9. `sergeant/<work-id>` does not already exist in that repository.
    WorkBranchAbsent,
    /// 10. The planned linked-worktree path is absent and not registered.
    WorktreePathAbsent,
    /// 11. All selected repositories can be planned before the first
    ///     materialization side effect occurs.
    AllRepositoriesPlanned,
}

impl PreflightCheck {
    /// The stable wire code §15 requires. Prefixed so a client can tell a
    /// preflight refusal from every other 422 without a lookup table.
    pub fn code(self) -> &'static str {
        match self {
            Self::MountPresent => "git_preflight_mount_missing",
            Self::MountNotAliased => "git_preflight_mount_aliased",
            Self::MountTopLevel => "git_preflight_top_level_mismatch",
            Self::MountPrimaryCheckout => "git_preflight_linked_worktree_source",
            Self::CommonDirLockable => "git_preflight_common_dir_unlockable",
            Self::HeadAttached => "git_preflight_detached_head",
            Self::HeadFullSha => "git_preflight_unresolvable_head",
            Self::WorktreeClean => "git_preflight_dirty_mount",
            Self::WorkBranchAbsent => "git_preflight_work_branch_collision",
            Self::WorktreePathAbsent => "git_preflight_worktree_path_collision",
            Self::AllRepositoriesPlanned => "git_preflight_incomplete_plan",
        }
    }

    /// §8.1's own numbering, carried so a refusal can cite the check the
    /// proposal names rather than only this crate's spelling of it.
    pub fn number(self) -> u8 {
        match self {
            Self::MountPresent => 1,
            Self::MountNotAliased => 2,
            Self::MountTopLevel => 3,
            Self::MountPrimaryCheckout => 4,
            Self::CommonDirLockable => 5,
            Self::HeadAttached => 6,
            Self::HeadFullSha => 7,
            Self::WorktreeClean => 8,
            Self::WorkBranchAbsent => 9,
            Self::WorktreePathAbsent => 10,
            Self::AllRepositoriesPlanned => 11,
        }
    }

    /// §8.3's table, as a predicate: exactly two conditions may be waived by
    /// `--override-git-preflight`, and only because an exact commit can still
    /// be pinned in both.
    ///
    /// Everything else is on the never-bypass list, and the reason is one
    /// sentence of the proposal's own:
    ///
    /// > Override may waive policy caution. It may not replace a missing fact
    /// > or overcome mechanical impossibility.
    ///
    /// [`Self::HeadFullSha`] is the sharpest case and deliberately `false`:
    /// an unresolvable HEAD is precisely a missing fact — there is no commit
    /// to pin — so §15 requires the refusal to say the override is
    /// unavailable rather than to offer it.
    pub fn waivable(self) -> bool {
        matches!(self, Self::WorktreeClean | Self::HeadAttached)
    }

    /// §15's "named remedy": what the operator does about it.
    pub fn remedy(self) -> &'static str {
        match self {
            Self::MountPresent => {
                "clone or place the repository at <estate-root>/repos/<name>, or `sgt repo add \
                 <name> --origin <url>`"
            }
            Self::MountNotAliased => {
                "replace the symlink with the estate's own checkout at \
                 <estate-root>/repos/<name>; separate estates use separate clones, even for the \
                 same upstream repository (§6.2)"
            }
            Self::MountTopLevel => {
                "make <estate-root>/repos/<name> the repository's own top level — a `../` alias \
                 or another estate's clone reached through it is refused (§6.2)"
            }
            Self::MountPrimaryCheckout => {
                "clone the repository into <estate-root>/repos/<name>; a linked worktree owns \
                 neither its branches nor its worktree registry and may not be a repository \
                 source (§6.2)"
            }
            Self::CommonDirLockable => {
                "wait for whatever holds this repository's lock to finish, or move the estate to \
                 a filesystem where advisory locking is reliable"
            }
            Self::HeadAttached => {
                "check the mount out onto the branch this Work should be based on (`git -C \
                 <mount> switch <branch>`), or re-submit with --override-git-preflight to pin \
                 the exact detached HEAD and record no named base branch"
            }
            Self::HeadFullSha => {
                "give the mount a commit to be based on (`git -C <mount> commit`, or check out a \
                 branch that has one); --override-git-preflight is unavailable here because no \
                 exact base can be pinned"
            }
            Self::WorktreeClean => {
                "commit or stash the mount's changes (`git -C <mount> status`), or re-submit \
                 with --override-git-preflight to base the Work on the committed HEAD and \
                 exclude the uncommitted changes"
            }
            Self::WorkBranchAbsent => {
                "this ref belongs to the Work named in it and is never deleted or reused \
                 automatically (§9.1); inspect it with `sgt work show <work-id>` and resolve it \
                 yourself if it is genuinely stale"
            }
            Self::WorktreePathAbsent => {
                "the path belongs to the Work named in it; inspect it with `sgt work show \
                 <work-id>` — sergeant never removes or reuses another Work's surface \
                 automatically"
            }
            Self::AllRepositoriesPlanned => {
                "resolve every named repository's finding above, or narrow the scope with --repo \
                 so the submission covers only repositories that can be planned"
            }
        }
    }
}

/// One §8.1 check that did not hold, for one repository.
///
/// Carried whether it was refused or waived: §8.3 requires "the override and
/// every waived finding are journaled with the Work", so a waived finding is
/// exactly this value stored on the admission record rather than raised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightFinding {
    /// Repository this was found in. `AllRepositoriesPlanned` is the one
    /// scope-level finding and names the whole selection instead.
    pub repository: String,
    /// Which §8.1 check.
    pub check: PreflightCheck,
    /// What was actually observed — the evidence half of §15's "show
    /// branch/HEAD/status", never a restatement of the check's name.
    pub detail: String,
    /// §8.3's dirty-mount evidence: the *full* `git status --porcelain`
    /// output, recorded verbatim so a waived dirty admission journals exactly
    /// what was excluded from the Work base rather than a count of it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub porcelain: Option<String>,
}

impl PreflightFinding {
    fn new(repository: &str, check: PreflightCheck, detail: impl Into<String>) -> Self {
        Self {
            repository: repository.to_string(),
            check,
            detail: detail.into(),
            porcelain: None,
        }
    }

    /// The stable API code for this finding (§15).
    pub fn code(&self) -> &'static str {
        self.check.code()
    }

    /// The named remedy for this finding (§15).
    pub fn remedy(&self) -> &'static str {
        self.check.remedy()
    }
}

impl std::fmt::Display for PreflightFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (§8.1 check {}, {}): {} — {}",
            self.repository,
            self.check.number(),
            self.check.code(),
            self.detail,
            self.check.remedy()
        )
    }
}

/// What §8.3 authorized for one repository, journaled with its binding.
///
/// Present on an admitted repository whether or not anything was waived: the
/// honest record of a clean admission is "the override was not requested and
/// nothing was waived", which is a different fact from "no preflight ran"
/// (what a `None` on a pre-Phase-E binding means).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightEvidence {
    /// Whether the submission carried `--override-git-preflight` at all. Note
    /// this is the *submission's* flag, not "something was waived": an
    /// override that turned out to waive nothing still records that the
    /// operator typed it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub override_authorized: bool,
    /// Every §8.1 finding the override waived, with its evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waived: Vec<PreflightFinding>,
    /// §8.3's required explicit statement, when a dirty mount was waived:
    /// which uncommitted changes are *excluded* from the Work base. `None`
    /// when nothing was excluded, which is the ordinary case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncommitted_excluded: Option<String>,
}

impl PreflightEvidence {
    /// Whether this admission waived anything at all.
    pub fn waived_anything(&self) -> bool {
        !self.waived.is_empty()
    }
}

/// One repository, admitted: every §8.1 fact resolved, and the §8.2 base
/// pinned.
///
/// This is what makes §8.2's base an *admitted* fact rather than an ambient
/// one. Materialization reads `base_sha`/`base_branch` from here instead of
/// re-asking the mount, so the commit the Work is based on is the commit
/// preflight judged — not whatever HEAD happens to be by the time
/// `git worktree add` runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedRepository {
    /// Repository name within the estate.
    pub repository: String,
    /// The canonical derived mount `<estate-root>/repos/<name>`.
    pub mount_path: PathBuf,
    /// §3.5's canonical Git top level.
    pub canonical_top_level: PathBuf,
    /// §3.5's canonical Git common directory — §2.7/§9.4's lock identity.
    pub canonical_common_dir: PathBuf,
    /// §8.2's base branch: the mount's current local branch. `None` only for
    /// a detached mount admitted under §8.3's override, which records no
    /// named base branch.
    #[serde(default)]
    pub base_branch: Option<String>,
    /// §8.2's base commit: the mount's exact HEAD, always a full SHA.
    pub base_sha: String,
    /// The `sergeant/<work-id>` branch check 9 found absent.
    pub work_branch: String,
    /// The linked-worktree path check 10 found absent and unregistered.
    pub worktree_path: PathBuf,
    /// §8.3's override record for this repository.
    #[serde(default)]
    pub evidence: PreflightEvidence,
}

/// The whole scope, admitted (§8.1 check 11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitPreflight {
    /// The Work this admission is for — the id checks 9 and 10 are about.
    pub work_id: String,
    /// Whether the submission carried `--override-git-preflight`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub override_requested: bool,
    /// One record per selected repository, in scope order.
    pub repositories: Vec<AdmittedRepository>,
}

impl GitPreflight {
    /// Every finding §8.3 waived across the whole scope, in scope order.
    pub fn waived(&self) -> impl Iterator<Item = &PreflightFinding> {
        self.repositories
            .iter()
            .flat_map(|r| r.evidence.waived.iter())
    }

    /// The admission record for one repository by name.
    pub fn repository(&self, name: &str) -> Option<&AdmittedRepository> {
        self.repositories.iter().find(|r| r.repository == name)
    }
}

/// A submission refused by §8.1, before a Work record exists.
///
/// Holds *every* unresolved finding rather than the first, because §15's
/// remedy for a multi-repository scope is not "fix this one and resubmit to
/// discover the next": check 11 is a scope-level fact, and an operator who
/// has to iterate one repository per submission is being told a fraction of
/// what preflight already knows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightRefusal {
    /// At least one finding, in scope order, with check 11's scope-level
    /// finding last when the scope had more than one repository.
    pub findings: Vec<PreflightFinding>,
}

impl PreflightRefusal {
    /// The primary structured code (§15): the first repository-level finding,
    /// which is the most specific thing an operator can act on. Every other
    /// finding travels in the body beside it.
    pub fn code(&self) -> &'static str {
        self.findings
            .first()
            .map_or(PreflightCheck::AllRepositoriesPlanned.code(), |f| f.code())
    }

    /// Whether any refused finding would have been waivable by
    /// `--override-git-preflight` — what §15's dirty/detached row means by
    /// "mention the bounded escape hatch", and deliberately *not* mentioned
    /// when nothing here is waivable.
    pub fn override_would_help(&self) -> bool {
        self.findings.iter().any(|f| f.check.waivable())
    }
}

impl std::fmt::Display for PreflightRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "git admission preflight refused this submission (§8.1): "
        )?;
        for (index, finding) in self.findings.iter().enumerate() {
            if index > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{finding}")?;
        }
        if self.override_would_help() {
            f.write_str(
                ". --override-git-preflight waives a dirty or detached mount — and nothing else",
            )?;
        }
        Ok(())
    }
}

/// Run §8.1 for every selected repository.
///
/// `work_id` is the id the Work *would* get: checks 9 and 10 are questions
/// about `sergeant/<work-id>` and `<surfaces_root>/<work-id>/<repo>`, so the
/// id has to exist before the record does. Minting a ULID costs nothing and
/// has no side effect, so a refusal simply discards it — which is the whole
/// point of running here, "before a Work record exists".
///
/// `data_dir` is where §9.4's repository locks live, and is the only reason
/// this function takes it (check 5).
///
/// Ordering within a repository is deliberate: the mount identity checks
/// (1–4) come first because every later question is meaningless if the path
/// is not the repository we think it is, and check 7 is evaluated even when
/// check 6 fails, because a detached HEAD still has a commit and §8.3 can
/// only waive the detachment if that commit exists.
///
/// **A repository stops at its first finding.** Reporting "detached *and*
/// dirty *and* colliding" for a mount whose top level is not even the
/// declared path is noise, not evidence — but every *repository* is
/// evaluated, so a two-repository scope with a problem in each is refused
/// naming both.
pub fn run(
    estate: &Estate,
    data_dir: &Path,
    surfaces_root: &Path,
    work_id: &str,
    repositories: &[RepositorySpec],
    override_requested: bool,
) -> Result<GitPreflight, PreflightRefusal> {
    let manifest = estate.root.join(MANIFEST_FILE).display().to_string();
    let branch = work_branch(work_id);
    let root = surface_root(surfaces_root, work_id);

    let mut admitted = Vec::with_capacity(repositories.len());
    let mut refused = Vec::new();
    let mut unplannable = Vec::new();

    for repository in repositories {
        let worktree_path = root.join(&repository.name);
        match check_one(
            &manifest,
            data_dir,
            repository,
            &branch,
            &worktree_path,
            override_requested,
        ) {
            Ok(record) => admitted.push(record),
            Err(finding) => {
                unplannable.push(finding.repository.clone());
                refused.push(finding);
            }
        }
    }

    if refused.is_empty() {
        return Ok(GitPreflight {
            work_id: work_id.to_string(),
            override_requested,
            repositories: admitted,
        });
    }

    // Check 11, as a finding rather than an invariant nobody can see fail.
    // Only for a multi-repository scope: for a single repository "1 of 1
    // could not be planned" restates the finding above it, while for a scope
    // it is the fact that decides the outcome — no repository was
    // materialized, including the ones that were fine.
    if repositories.len() > 1 {
        refused.push(PreflightFinding::new(
            &unplannable.join(", "),
            PreflightCheck::AllRepositoriesPlanned,
            format!(
                "{} of {} selected repositories could not be planned ({}); no repository was \
                 materialized — §8.1 requires the whole scope to plan before the first \
                 materialization side effect",
                unplannable.len(),
                repositories.len(),
                unplannable.join(", ")
            ),
        ));
    }
    Err(PreflightRefusal { findings: refused })
}

/// §8.1 for one repository. `Ok` is an admission; `Err` is its first
/// unresolved finding.
fn check_one(
    manifest: &str,
    data_dir: &Path,
    repository: &RepositorySpec,
    branch: &str,
    worktree_path: &Path,
    override_requested: bool,
) -> Result<AdmittedRepository, PreflightFinding> {
    let name = repository.name.as_str();
    let mount = repository.path.as_path();

    // Checks 1–4, decided by the one function that already decides them.
    let canonical_top_level = validate_mount(manifest, name, mount).map_err(|e| {
        let (check, detail) = classify_mount_error(mount, &e);
        PreflightFinding::new(name, check, detail)
    })?;

    // Check 5: resolvable, on a filesystem whose advisory locks mean
    // something, and actually lockable.
    let canonical_common_dir = canonical_git_common_dir(mount).map_err(|e| {
        PreflightFinding::new(
            name,
            PreflightCheck::CommonDirLockable,
            format!(
                "git cannot resolve a common directory for {}: {e}",
                mount.display()
            ),
        )
    })?;
    // C4's disposition: the same gate `daemon::start_with` applies to its own
    // data dir, applied here to the mount's filesystem. `Unknown` never
    // refuses — an inconclusive probe must not brick a platform whose
    // detection is unmeasured (`fs_locking`'s own asymmetry).
    if let Reliability::Unreliable { filesystem } =
        fs_locking::detect_for_path(&canonical_common_dir)
    {
        return Err(PreflightFinding::new(
            name,
            PreflightCheck::CommonDirLockable,
            format!(
                "{} is on a {filesystem} filesystem: {}",
                canonical_common_dir.display(),
                fs_locking::remedy(&filesystem)
            ),
        ));
    }
    // Taken and released: this asks "can this repository be locked at all",
    // not "is it free forever". A holder is *waited out* rather than refused
    // (§9.4's contender semantics, which `repolock::acquire` implements), so
    // an ordinary concurrent materialization does not turn into an admission
    // refusal — only exhausting the budget does, and that is a structured
    // failure naming the lock.
    drop(
        repolock::acquire(data_dir, &canonical_common_dir).map_err(|e| {
            PreflightFinding::new(name, PreflightCheck::CommonDirLockable, e.to_string())
        })?,
    );

    // Check 6: attached HEAD. Waivable — but only once check 7 has proved
    // there is a commit to pin, so the finding is held rather than raised.
    let base_branch = git(mount, &["symbolic-ref", "--quiet", "--short", "HEAD"]).ok();

    // Check 7: a full commit SHA. Evaluated even when 6 failed, because a
    // detached HEAD still has one — and because §8.3 may only waive the
    // detachment when "an exact commit can still be pinned".
    let head = git(mount, &["rev-parse", "--verify", "HEAD"]).map_err(|e| {
        PreflightFinding::new(
            name,
            PreflightCheck::HeadFullSha,
            format!(
                "HEAD in {} does not resolve to a commit (an unborn branch, or a repository with \
                 no commits yet): {e}",
                mount.display()
            ),
        )
    })?;
    if !is_full_object_id(&head) {
        return Err(PreflightFinding::new(
            name,
            PreflightCheck::HeadFullSha,
            format!(
                "HEAD in {} resolved to {head:?}, which is not a full object id",
                mount.display()
            ),
        ));
    }

    let mut evidence = PreflightEvidence {
        override_authorized: override_requested,
        ..PreflightEvidence::default()
    };

    if base_branch.is_none() {
        let finding = PreflightFinding::new(
            name,
            PreflightCheck::HeadAttached,
            format!(
                "HEAD in {} is detached at {head}; there is no named local branch to record as \
                 the base",
                mount.display()
            ),
        );
        if !override_requested {
            return Err(finding);
        }
        evidence.waived.push(finding);
    }

    // Check 8: clean index *and* working tree. `--porcelain` reports both,
    // and `runtime::git` already runs every invocation with
    // `GIT_OPTIONAL_LOCKS=0`, so asking does not write to the mount's index.
    let porcelain = git(mount, &["status", "--porcelain"]).map_err(|e| {
        PreflightFinding::new(
            name,
            PreflightCheck::WorktreeClean,
            format!("git cannot report status for {}: {e}", mount.display()),
        )
    })?;
    if !porcelain.is_empty() {
        let mut finding = PreflightFinding::new(
            name,
            PreflightCheck::WorktreeClean,
            format!(
                "{} has {} uncommitted change(s) in its index or working tree at branch {} \
                 (HEAD {head})",
                mount.display(),
                porcelain.lines().count(),
                base_branch.as_deref().unwrap_or("(detached)"),
            ),
        );
        // §8.3: "record full porcelain evidence".
        finding.porcelain = Some(porcelain.clone());
        if !override_requested {
            return Err(finding);
        }
        // §8.3: "state explicitly that uncommitted changes are excluded from
        // the Work base".
        evidence.uncommitted_excluded = Some(format!(
            "the Work is based on the committed HEAD {head}; the {} uncommitted change(s) below \
             stayed in the mount at {} and are excluded from the Work base",
            porcelain.lines().count(),
            mount.display()
        ));
        evidence.waived.push(finding);
    }

    // Check 9: the Work's own branch must not already exist. Never waivable,
    // never deleted or reused automatically (§9.1, §15).
    let ref_name = format!("refs/heads/{branch}");
    if git_succeeds(mount, &["show-ref", "--verify", "--quiet", &ref_name]) {
        return Err(PreflightFinding::new(
            name,
            PreflightCheck::WorkBranchAbsent,
            format!(
                "{ref_name} already exists in {}; it belongs to work {}",
                mount.display(),
                owning_work_id(branch),
            ),
        ));
    }

    // Check 10: the planned worktree path must be absent *and* unregistered.
    // Two questions, because they fail apart: a directory left behind by a
    // crashed teardown is absent from the registry, and a registry entry
    // whose directory was deleted by hand is absent from the disk.
    if worktree_path.symlink_metadata().is_ok() {
        return Err(PreflightFinding::new(
            name,
            PreflightCheck::WorktreePathAbsent,
            format!(
                "{} already exists; it is work {}'s surface",
                worktree_path.display(),
                owning_work_id(branch),
            ),
        ));
    }
    if let Some(registered) = registered_worktree(mount, worktree_path) {
        return Err(PreflightFinding::new(
            name,
            PreflightCheck::WorktreePathAbsent,
            format!(
                "{} is still registered as a linked worktree of {} (git worktree list reports \
                 {registered}), even though nothing is at that path",
                worktree_path.display(),
                mount.display(),
            ),
        ));
    }

    Ok(AdmittedRepository {
        repository: name.to_string(),
        mount_path: canonical_top_level.clone(),
        canonical_top_level,
        canonical_common_dir,
        base_branch,
        base_sha: head,
        work_branch: branch.to_string(),
        worktree_path: worktree_path.to_path_buf(),
        evidence,
    })
}

/// Map [`validate_mount`]'s three refusals onto §8.1's checks 1–4.
///
/// `validate_mount` answers one question — "is this the estate's own ordinary
/// checkout" — and §8.1 splits that question four ways, so the split happens
/// here rather than by asking git the same things again. The two cases
/// `RepositoryMountAliased` covers are separated by the one fact that
/// distinguishes them and that the error does not carry: whether the mount is
/// *itself* a symlink (check 2) or merely resolves somewhere else (check 3).
fn classify_mount_error(mount: &Path, error: &EstateError) -> (PreflightCheck, String) {
    match error {
        EstateError::RepositoryNotFound { .. } if !mount.exists() => {
            (PreflightCheck::MountPresent, error.to_string())
        }
        EstateError::RepositoryNotFound { .. } => {
            (PreflightCheck::MountTopLevel, error.to_string())
        }
        EstateError::RepositoryMountAliased { .. }
            if mount
                .symlink_metadata()
                .is_ok_and(|meta| meta.file_type().is_symlink()) =>
        {
            (PreflightCheck::MountNotAliased, error.to_string())
        }
        EstateError::RepositoryMountAliased { .. } => {
            (PreflightCheck::MountTopLevel, error.to_string())
        }
        EstateError::RepositoryMountIsLinkedWorktree { .. } => {
            (PreflightCheck::MountPrimaryCheckout, error.to_string())
        }
        // Not reachable from `validate_mount`'s own body today. Classified as
        // the top-level check rather than silently admitted, because an
        // unclassified refusal is still a refusal.
        other => (PreflightCheck::MountTopLevel, other.to_string()),
    }
}

/// Whether `text` is a full object id — 40 hex characters for SHA-1, 64 for
/// SHA-256 (`git rev-parse --verify HEAD` answers in the repository's own
/// object format, and a repository created with `--object-format=sha256`
/// answers 64).
///
/// Checked rather than assumed: §8.1 check 7 is "HEAD resolves to a full
/// commit SHA", and the value is what a Work's base is pinned to for its
/// whole life. An abbreviated or otherwise unexpected answer is a fact this
/// admission does not have, not one to round off.
fn is_full_object_id(text: &str) -> bool {
    matches!(text.len(), 40 | 64) && text.chars().all(|c| c.is_ascii_hexdigit())
}

/// The Work a `sergeant/<work-id>` ref or surface directory belongs to (§15:
/// "name ref/path and owning Work evidence if known").
///
/// Read off the name itself rather than looked up in the registry: the
/// convention is what makes the ref attributable at all, and a lookup would
/// answer "unknown" for exactly the case an operator most needs named — a ref
/// left in a mount by an estate whose journal this daemon does not have.
fn owning_work_id(branch: &str) -> &str {
    branch.strip_prefix("sergeant/").unwrap_or(branch)
}

/// The registry entry for `worktree_path`, if `git worktree list` has one.
///
/// Compared canonically where possible — a registry records the path git was
/// given, and the same directory reached through a symlinked ancestor spells
/// differently — falling back to the literal comparison when neither side
/// canonicalizes (the ordinary case here, since check 10 is asking about a
/// path that should not exist).
fn registered_worktree(mount: &Path, worktree_path: &Path) -> Option<String> {
    let listing = git(mount, &["worktree", "list", "--porcelain"]).ok()?;
    let wanted: BTreeSet<PathBuf> = [
        Some(worktree_path.to_path_buf()),
        std::fs::canonicalize(worktree_path).ok(),
    ]
    .into_iter()
    .flatten()
    .collect();
    listing
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .find(|entry| {
            let path = PathBuf::from(entry);
            let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            wanted.contains(&path) || wanted.contains(&canonical)
        })
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §8.3's table is a *closed* list of two, and this is the assertion that
    /// keeps it closed: every other check is on the never-bypass list, and a
    /// variant added to the taxonomy without a decision about waivability
    /// fails here rather than defaulting into the escape hatch.
    #[test]
    fn only_dirty_and_detached_are_waivable() {
        use PreflightCheck::*;
        let all = [
            MountPresent,
            MountNotAliased,
            MountTopLevel,
            MountPrimaryCheckout,
            CommonDirLockable,
            HeadAttached,
            HeadFullSha,
            WorktreeClean,
            WorkBranchAbsent,
            WorktreePathAbsent,
            AllRepositoriesPlanned,
        ];
        assert_eq!(all.len(), 11, "§8.1 declares eleven checks");
        let waivable: Vec<PreflightCheck> = all.into_iter().filter(|c| c.waivable()).collect();
        assert_eq!(
            waivable,
            vec![HeadAttached, WorktreeClean],
            "§8.3 waives a dirty mount and a detached mount, and nothing else"
        );
    }

    /// The taxonomy exists to name each failure distinctly; two checks
    /// sharing a code would collapse exactly the distinction §15's structured
    /// codes are for.
    #[test]
    fn every_check_has_a_distinct_code_number_and_remedy() {
        use PreflightCheck::*;
        let all = [
            MountPresent,
            MountNotAliased,
            MountTopLevel,
            MountPrimaryCheckout,
            CommonDirLockable,
            HeadAttached,
            HeadFullSha,
            WorktreeClean,
            WorkBranchAbsent,
            WorktreePathAbsent,
            AllRepositoriesPlanned,
        ];
        let codes: BTreeSet<&str> = all.iter().map(|c| c.code()).collect();
        assert_eq!(codes.len(), all.len(), "codes must be distinct: {codes:?}");
        let numbers: BTreeSet<u8> = all.iter().map(|c| c.number()).collect();
        assert_eq!(numbers, (1..=11).collect::<BTreeSet<u8>>());
        let remedies: BTreeSet<&str> = all.iter().map(|c| c.remedy()).collect();
        assert_eq!(remedies.len(), all.len(), "remedies must be distinct");
        for check in all {
            assert!(
                check.code().starts_with("git_preflight_"),
                "{:?} must be recognizable as a preflight code",
                check
            );
        }
    }

    /// §15: the unresolvable-HEAD row is the one that must *not* offer the
    /// escape hatch, and it says so in the remedy rather than only by
    /// omission.
    #[test]
    fn the_unresolvable_head_remedy_says_the_override_is_unavailable() {
        let remedy = PreflightCheck::HeadFullSha.remedy();
        assert!(
            remedy.contains("--override-git-preflight is unavailable"),
            "§15 requires this refusal to say why no override applies: {remedy}"
        );
        assert!(!PreflightCheck::HeadFullSha.waivable());
    }

    /// §15's dirty/detached row asks the refusal to "mention the bounded
    /// escape hatch" — and, by the same token, a refusal with nothing
    /// waivable in it must not dangle one.
    #[test]
    fn only_a_waivable_refusal_mentions_the_escape_hatch() {
        let dirty = PreflightRefusal {
            findings: vec![PreflightFinding::new(
                "api",
                PreflightCheck::WorktreeClean,
                "two files modified",
            )],
        };
        assert!(dirty.override_would_help());
        assert!(dirty.to_string().contains("--override-git-preflight"));
        assert_eq!(dirty.code(), "git_preflight_dirty_mount");

        let collision = PreflightRefusal {
            findings: vec![PreflightFinding::new(
                "api",
                PreflightCheck::WorkBranchAbsent,
                "refs/heads/sergeant/01OTHER exists",
            )],
        };
        assert!(!collision.override_would_help());
        assert!(!collision.to_string().contains("--override-git-preflight"));
    }

    #[test]
    fn a_full_object_id_is_forty_or_sixty_four_hex_characters() {
        assert!(is_full_object_id(&"a".repeat(40)));
        assert!(is_full_object_id(&"0123456789abcdef".repeat(4)));
        assert!(!is_full_object_id("abc1234"), "abbreviated is not full");
        assert!(!is_full_object_id(""), "an empty answer is not a commit");
        assert!(!is_full_object_id(&"z".repeat(40)), "hex only");
        assert!(!is_full_object_id("HEAD"));
    }

    #[test]
    fn a_collision_names_the_work_that_owns_the_ref() {
        assert_eq!(owning_work_id("sergeant/01ABC"), "01ABC");
        // A branch that is not one of ours is reported verbatim rather than
        // mangled into a work id it never was.
        assert_eq!(owning_work_id("main"), "main");
    }
}

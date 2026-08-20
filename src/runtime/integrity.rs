//! Integrity observation and terminal dirtiness (proposal §11).
//!
//! §11 asks core to answer one question honestly at retirement: *was the
//! surface this Work was assigned left in a state core can account for?* The
//! answer is **orthogonal** to Work state. A Work that completed, failed, or
//! was canceled stays exactly that; nothing here is a transition target, no
//! [`WorkState`](crate::domain::work::WorkState) variant is added, and the
//! transition table is untouched. This is the same shape ADR 0007(b) already
//! uses for `reported_state`: the journal and the views carry the extra axis,
//! the state machine never learns about it.
//!
//! Three vocabularies live here:
//!
//! - [`IntegrityFinding`] — §11.3's **closed** list of directly attributable
//!   dirty findings. Closed means closed: an observation core cannot spell as
//!   one of these variants is not reported as a finding at all.
//! - [`EstateDriftObservation`] — §11.4's separate, *non*-attributable
//!   observation. A mount moving during the Work window is not evidence the
//!   Work moved it, so drift never makes a Work dirty and its attribution is
//!   structurally [`DriftAttribution::Unknown`].
//! - [`IntegrityDisposition`] — the two-valued axis §11.5 puts beside the
//!   terminal state.
//!
//! Nothing in this module performs I/O. `runtime::surface` observes; this
//! module is the vocabulary those observations are recorded in.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// §11.5's orthogonal axis: whether a terminal Work's assigned surface was
/// left in a state core can account for.
///
/// `dirty` is never a state transition and never blocks one. `sgt work list`
/// still reports the true terminal state; this rides beside it (and, for a
/// dirty *completion*, is what makes `reported_state` say `completed_dirty` —
/// §11.5's compact label, which the domain model still does not own).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityDisposition {
    /// Every binding was retired with nothing left unaccounted for.
    Clean,
    /// At least one finding, or at least one binding core could not retire
    /// cleanly. See [`crate::runtime::surface::TeardownReport::integrity`]
    /// for exactly what earns this.
    Dirty,
}

impl IntegrityDisposition {
    /// The serialized spelling, for renderers that hold a value rather than
    /// its JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Dirty => "dirty",
        }
    }
}

/// What a binding's worktree HEAD **actually** pointed at when teardown
/// looked — as opposed to the `work_branch` the binding claims.
///
/// §2.6's defect in one type: teardown used to read the branch it expected
/// and never the worktree it had, so a worktree that ended up somewhere else
/// was reported as though it had not. Recorded on every binding teardown can
/// still inspect; `None` on a [`BindingTeardown`](crate::runtime::surface::BindingTeardown)
/// means *not assessed* — a vanished worktree, a git that could not answer,
/// or an event journaled before this field existed — never "it was where it
/// should have been".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "head", rename_all = "snake_case")]
pub enum ObservedHead {
    /// HEAD was attached to a named local branch.
    Branch {
        /// The branch HEAD pointed at, short form.
        branch: String,
        /// That branch's tip, `None` only if it could not be resolved (an
        /// unborn branch), which is recorded rather than guessed.
        sha: Option<String>,
    },
    /// HEAD was detached at a commit. Always carries the commit: a HEAD whose
    /// commit cannot be resolved at all is not classified as a detachment
    /// here, it is left to teardown's own fail-closed status check.
    Detached {
        /// The commit HEAD was detached at.
        sha: String,
    },
}

/// §11.3's closed serialized vocabulary of directly attributable dirty
/// findings: things that happened *in the Work's assigned surface* or were
/// reported by its own execution, so attributing them to the Work is not a
/// guess.
///
/// Every variant carries the expected value, the observed value, and an
/// evidence string — §11.3's "expected and observed paths/refs/SHAs plus
/// evidence" — because a finding an operator cannot act on is a warning, not
/// a report.
///
/// The vocabulary is closed on purpose. An observation core cannot spell as
/// one of these is not promoted into a free-form finding; it is either an
/// [`EstateDriftObservation`] (unattributable) or it is not reported as a
/// finding at all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "finding", rename_all = "snake_case")]
pub enum IntegrityFinding {
    /// The assigned worktree had uncommitted or untracked changes at
    /// teardown. Rides *alongside* the existing `retained_dirty` disposition
    /// and #109's patch capture rather than replacing them: the disposition
    /// says what was done with the content, this says what it means for the
    /// Work.
    AssignedWorktreeUncommitted {
        /// Repository the binding names.
        repository: String,
        /// The assigned worktree.
        worktree_path: PathBuf,
        /// `git status --porcelain` output.
        evidence: String,
    },
    /// The assigned worktree path was gone before teardown reached it.
    AssignedWorktreeMissing {
        /// Repository the binding names.
        repository: String,
        /// The worktree path that should have been there.
        worktree_path: PathBuf,
        /// What was looked for.
        evidence: String,
    },
    /// §2.6 / #173: the worktree's HEAD was attached to a *different* named
    /// branch than the one this binding was assigned. The observed branch and
    /// its tip are recorded here because removing the worktree destroys the
    /// only other record of them — and both branches stay durable (§11.7,
    /// §12.1: teardown deletes neither the expected branch nor the one that
    /// was actually used).
    AssignedBranchMismatch {
        /// Repository the binding names.
        repository: String,
        /// The assigned worktree.
        worktree_path: PathBuf,
        /// The branch this binding was assigned: `sergeant/<work-id>`.
        expected_branch: String,
        /// That branch's tip at teardown, if it resolved.
        expected_sha: Option<String>,
        /// The branch HEAD was actually on.
        observed_branch: String,
        /// That branch's tip at teardown, if it resolved.
        observed_sha: Option<String>,
        /// A one-line statement of the divergence.
        evidence: String,
    },
    /// The worktree's HEAD was detached. `reachable` distinguishes the two
    /// cases §11.7 treats very differently: a detached HEAD some named ref
    /// still reaches is a finding and nothing more, while one no named ref is
    /// *proven* to reach also retains the worktree
    /// ([`RetainedUnreferenced`](crate::runtime::surface::BindingDisposition::RetainedUnreferenced))
    /// rather than risk the commits.
    AssignedHeadDetachedOrUnreferenced {
        /// Repository the binding names.
        repository: String,
        /// The assigned worktree.
        worktree_path: PathBuf,
        /// The branch this binding was assigned.
        expected_branch: String,
        /// The commit HEAD was detached at.
        observed_sha: String,
        /// Whether a named ref in the source repository was proven to reach
        /// that commit.
        reachable: bool,
        /// What was checked to decide `reachable`.
        evidence: String,
    },
    /// The worktree and its source checkout did not agree on their canonical
    /// Git common directory — the identity §2.7/§9.4 lock on. A worktree
    /// answering with a different common dir than the repository it is
    /// journaled against is not the worktree this binding thinks it is.
    AssignedCommonDirMismatch {
        /// Repository the binding names.
        repository: String,
        /// `git rev-parse --path-format=absolute --git-common-dir` run in
        /// the binding's source checkout.
        expected_common_dir: PathBuf,
        /// The same question asked inside the assigned worktree.
        observed_common_dir: PathBuf,
        /// A one-line statement of the divergence.
        evidence: String,
    },
    /// **Defined, not emitted (amendment C2).** An adapter observed the
    /// execution running a Git command against something outside its assigned
    /// surface.
    ///
    /// Emission is gated on *structured* adapter evidence that does not exist
    /// today: the claude adapter fixes a session cwd and forwards raw `input`
    /// blobs, so the only way to fire this now would be heuristic shell
    /// parsing, which the gauntlet's C2 disposition explicitly refuses. The
    /// variant lives here so the serialized vocabulary is the closed §11.3
    /// list rather than a subset that would have to widen later — an adapter
    /// that cannot prove this simply runs, and core reports only what it can
    /// prove (§18). The emitter arrives with the evidence, not before it.
    AdapterObservedOutsideSurfaceGitCommand {
        /// Repository the binding names.
        repository: String,
        /// The assigned worktree the command should have been confined to.
        worktree_path: PathBuf,
        /// Where the adapter observed the command running, when it can say.
        observed_path: Option<PathBuf>,
        /// The adapter's own structured evidence, verbatim.
        evidence: String,
    },
    /// **Defined, not emitted (amendment C2).** A backend reported writing
    /// outside the Work's assigned surface. Gated on the same missing
    /// structured evidence as
    /// [`AdapterObservedOutsideSurfaceGitCommand`](Self::AdapterObservedOutsideSurfaceGitCommand);
    /// see that variant for the full reasoning.
    BackendReportedOutsideSurfaceWrite {
        /// Repository the binding names.
        repository: String,
        /// The assigned worktree the write should have been confined to.
        worktree_path: PathBuf,
        /// Where the backend reported writing, when it can say.
        observed_path: Option<PathBuf>,
        /// The backend's own structured evidence, verbatim.
        evidence: String,
    },
}

impl IntegrityFinding {
    /// The serialized kind, exactly as §11.3 spells it — for renderers that
    /// hold the value rather than its JSON.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::AssignedWorktreeUncommitted { .. } => "assigned_worktree_uncommitted",
            Self::AssignedWorktreeMissing { .. } => "assigned_worktree_missing",
            Self::AssignedBranchMismatch { .. } => "assigned_branch_mismatch",
            Self::AssignedHeadDetachedOrUnreferenced { .. } => {
                "assigned_head_detached_or_unreferenced"
            }
            Self::AssignedCommonDirMismatch { .. } => "assigned_common_dir_mismatch",
            Self::AdapterObservedOutsideSurfaceGitCommand { .. } => {
                "adapter_observed_outside_surface_git_command"
            }
            Self::BackendReportedOutsideSurfaceWrite { .. } => {
                "backend_reported_outside_surface_write"
            }
        }
    }

    /// The repository this finding is about.
    pub fn repository(&self) -> &str {
        match self {
            Self::AssignedWorktreeUncommitted { repository, .. }
            | Self::AssignedWorktreeMissing { repository, .. }
            | Self::AssignedBranchMismatch { repository, .. }
            | Self::AssignedHeadDetachedOrUnreferenced { repository, .. }
            | Self::AssignedCommonDirMismatch { repository, .. }
            | Self::AdapterObservedOutsideSurfaceGitCommand { repository, .. }
            | Self::BackendReportedOutsideSurfaceWrite { repository, .. } => repository,
        }
    }
}

/// §11.4: who moved a mount. Core cannot tell, so this has exactly one value.
///
/// A one-variant enum rather than a string field is the point: nothing in
/// this phase can record drift as attributed, because nothing in this phase
/// can prove it. Captain, an editor, another Work, or an unrelated process
/// may have done it. Direct adapter tool evidence would promote the same
/// observation into an [`IntegrityFinding`] instead — a different type, not a
/// different attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftAttribution {
    /// Core observed the difference and can prove nothing about its cause.
    Unknown,
}

/// §11.4: a declared repository mount whose committed HEAD moved between the
/// Work's admission base and its retirement.
///
/// **Never makes a Work dirty.** It is reported beside the findings, not
/// among them.
///
/// **Bounded exactly as amendment C6 requires**, and no further: one
/// `git rev-parse HEAD` per *bound* repository at retirement. No worktree
/// walking, no `git status` on a mount, no unselected-repository calls, no
/// polling. Observing drift in repositories this Work never selected needs
/// the estate-bound daemon and is Phase D's work, not a widening of this.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstateDriftObservation {
    /// Repository name within the estate.
    pub repository: String,
    /// The commit this Work's binding was cut from.
    pub before: String,
    /// The mount's committed HEAD at retirement.
    pub observed: String,
    /// Always [`DriftAttribution::Unknown`] — see the type.
    pub attribution: DriftAttribution,
}

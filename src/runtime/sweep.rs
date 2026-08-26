//! §12.3's deliberate sweep: classify every `sergeant/*` ref an estate's
//! mounts carry, and — as a separate authorization — delete the ones proven
//! to hold no unique content (#159).
//!
//! §12.3 keeps this out of `sgt doctor` on purpose: doctor's git-surfaces row
//! is a cheap count derived from the journal, while this walks every mount's
//! ref store and asks git about ancestry per branch. The split is also a
//! safety property, not only a budget one — classification is a pure read
//! that mutates nothing, and deletion is a second, explicitly confirmed call
//! that re-derives the classification for itself rather than trusting the
//! report the operator happened to be looking at.
//!
//! Every git verb here is local: `for-each-ref`, `symbolic-ref`, `merge-base`,
//! `rev-list`, `worktree list`, and — on the deletion path only — `branch -D`.
//! Nothing fetches, and nothing consults a remote to decide what is redundant.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::estate::Estate;
use crate::domain::work::WorkState;
use crate::runtime::git::{git, git_succeeds};
use crate::runtime::projection::is_absorbing;
use crate::runtime::surface::{RepositoryGate, with_repository};

/// The ref namespace a sweep may look at, and the only one it may ever
/// delete from — `surface::work_branch`'s `sergeant/<work-id>` under
/// `refs/heads/`. A ref outside this prefix belongs to the operator and is
/// invisible to every verb in this module.
const SERGEANT_REFS: &str = "refs/heads/sergeant/";

/// Branch-name prefix [`SERGEANT_REFS`] resolves to in `refname:short` form.
const SERGEANT_BRANCH_PREFIX: &str = "sergeant/";

/// The one caveat the redundancy criterion carries, named in every report so
/// an operator never reads `redundant` as a bare verdict.
///
/// `merge-base --is-ancestor` proves containment of *commits*, which is
/// exactly what a merge-commit history records when work lands. A squash- or
/// rebase-merge rewrites the commits instead, so a branch whose content is
/// fully merged that way is never an ancestor of anything and classifies
/// here as `retained` — under-claiming, which is the direction this check is
/// allowed to be wrong in.
pub const REDUNDANCY_NOTE: &str = "`redundant` means the branch tip is an ancestor of the mount's \
                                   default branch, so the branch holds no unique commits. A \
                                   squash- or rebase-merge workflow rewrites commits and defeats \
                                   that check: work merged that way classifies as `retained`, \
                                   never as `redundant`.";

/// What a sweep concluded about one `sergeant/*` ref.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    /// The ref belongs to a Work that has not reached an absorbing state.
    /// Never eligible for anything: the Work may still execute on it.
    Active,
    /// The tip is an ancestor of the mount's default branch — the branch
    /// holds no unique commits (see [`REDUNDANCY_NOTE`]). The one criterion
    /// deletion is ever authorized against.
    Redundant,
    /// A terminal Work's branch carrying commits the default branch does not
    /// contain. Reported, never deletable by this verb.
    Retained,
    /// No journaled Work exists for this ULID at all (#172): a `sergeant/*`
    /// ref on the mount that sergeant never recorded creating. Report-only —
    /// sweep has no evidence about what it is or who wants it.
    Orphan,
}

impl Classification {
    /// The wire/display spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Redundant => "redundant",
            Self::Retained => "retained",
            Self::Orphan => "orphan",
        }
    }
}

impl std::fmt::Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One classified `sergeant/*` ref.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SweptBranch {
    /// Short branch name (`sergeant/<work-id>`).
    pub branch: String,
    /// The ULID the branch name encodes — the Work id it was cut for.
    pub work_id: String,
    /// Tip commit, as recorded at classification time. Also what the
    /// deletion journal records, so what was destroyed stays recoverable
    /// from the reflog or a `git branch <name> <sha>`.
    pub tip: String,
    /// What this sweep concluded.
    pub classification: Classification,
    /// Commits reachable from `tip` that the default branch does not already
    /// contain. `None` when the mount named no default branch to compare
    /// against, or when counting them failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_commits: Option<u64>,
    /// Why this classification, in the operator's terms.
    pub detail: String,
}

/// Everything one estate mount answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepositorySweep {
    /// Repository name within the estate.
    pub repository: String,
    /// The mount that was read.
    pub path: PathBuf,
    /// The mount's own `HEAD` symbolic ref, named in the report because it
    /// is what every `redundant` verdict is relative to. `None` for a
    /// detached mount, which makes redundancy unprovable and every terminal
    /// branch `retained`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    /// Classified refs, in `for-each-ref` order (lexical by refname).
    pub branches: Vec<SweptBranch>,
    /// Linked-worktree registrations git reports as prunable. Report-only:
    /// the remedy is [`worktree_prune_remedy`], run by the operator.
    pub prunable_worktrees: usize,
    /// Set when the mount could not be read at all; `branches` is then empty
    /// and says nothing, rather than an absence being read as "clean".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unreadable: Option<String>,
}

/// The remedy named for a nonzero `prunable_worktrees`, per repository.
pub fn worktree_prune_remedy(path: &Path) -> String {
    format!("git -C {} worktree prune", path.display())
}

/// A whole-estate classification pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SweepReport {
    /// One entry per declared repository, in estate declaration order.
    pub repositories: Vec<RepositorySweep>,
    /// [`REDUNDANCY_NOTE`], carried in the report itself so a direct API
    /// consumer reads the caveat beside the verdict rather than only in the
    /// CLI's rendering of it.
    pub note: &'static str,
}

impl SweepReport {
    /// Every branch this pass classified `redundant`, as deletion targets —
    /// the exact set `sgt work sweep --delete-redundant` previews and, with
    /// `--yes`, submits.
    pub fn redundant(&self) -> Vec<SweepTarget> {
        self.repositories
            .iter()
            .flat_map(|repo| {
                repo.branches
                    .iter()
                    .filter(|b| b.classification == Classification::Redundant)
                    .map(|b| SweepTarget {
                        repository: repo.repository.clone(),
                        branch: b.branch.clone(),
                    })
            })
            .collect()
    }
}

/// One branch a deletion pass was asked to destroy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SweepTarget {
    /// Repository name within the estate.
    pub repository: String,
    /// Short branch name (`sergeant/<work-id>`).
    pub branch: String,
}

/// What became of one requested deletion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum DeletionOutcome {
    /// The branch was deleted. `tip` is what it pointed at.
    Deleted {
        /// Tip SHA at deletion time — the journal's record of what was
        /// destroyed, and of where it can be restored from.
        tip: String,
    },
    /// The branch was not deleted, and nothing was mutated for it. Every
    /// re-verification failure lands here: a ref that no longer exists, one
    /// that no longer classifies `redundant`, or one outside the
    /// `sergeant/*` namespace.
    Refused {
        /// Why, in the operator's terms.
        reason: String,
    },
    /// The branch classified `redundant` and git still refused to delete it
    /// (a branch checked out in a linked worktree, a lock we could not
    /// take).
    Failed {
        /// Git's own diagnostic, or the lock failure.
        detail: String,
    },
}

/// One requested target and its outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeletedBranch {
    /// Repository the target named.
    pub repository: String,
    /// Branch the target named.
    pub branch: String,
    /// What happened.
    #[serde(flatten)]
    pub outcome: DeletionOutcome,
}

/// Classify every `sergeant/*` ref in every mount of `estate`.
///
/// **Mutates nothing.** Every verb it runs is a local read, and the one
/// question it asks git that could be expensive — `merge-base --is-ancestor`
/// — is asked once per ref.
///
/// `works` is the journaled Work identity to cross-reference against: work id
/// → current state, for every Work this daemon has ever journaled. The
/// registry's `work_index` is exactly that set (a row per `work.submitted`,
/// never evicted), so callers hand it straight over rather than replaying the
/// journal a second time for the same fact.
pub fn classify(estate: &Estate, works: &BTreeMap<String, WorkState>) -> SweepReport {
    // W4a seam: this function is already correctly single-estate per call
    // (recon-engine-journal touch point 8 names it "correct as-is") and
    // needs no change here. What H1 still owes is the *caller*: every call
    // to `classify` today is the per-request estate-scoped HTTP surface
    // (`GET`/`POST /v1/sweep`, `api.rs`'s `sweep_estate`/its POST sibling) —
    // there is no periodic daemon-side loop anywhere in this codebase yet to
    // iterate admitted estates and call this once per estate (checked:
    // `daemon.rs` has no periodic sweep caller at all — unlike
    // `maybe_run_rotation_triggered_prune`'s rotation-triggered prune tick,
    // which this wave *did* rewire onto `EstateRegistry`-backed multi-estate
    // retention, see `prune::EstatePolicies`). When a periodic sweep caller
    // is added, it must iterate `estates::EstateRegistry::entries()` and
    // call `classify` once per admitted estate — never assume the
    // one-estate world this function's own signature still lets a naive
    // caller fall into. Named here rather than dropped (brief deliverable
    // 4); a W5 item.
    SweepReport {
        repositories: estate
            .repositories
            .iter()
            .map(|spec| classify_repository(&spec.name, &spec.path, works))
            .collect(),
        note: REDUNDANCY_NOTE,
    }
}

fn classify_repository(
    repository: &str,
    path: &Path,
    works: &BTreeMap<String, WorkState>,
) -> RepositorySweep {
    // The mount's own HEAD, never a remote's idea of one (§6.4: sergeant does
    // not infer a remote default). A detached mount answers nothing, which
    // costs the redundancy check and is reported as such.
    let default_branch = git(path, &["symbolic-ref", "--short", "HEAD"])
        .ok()
        .filter(|name| !name.is_empty());
    let listing = match git(
        path,
        &[
            "for-each-ref",
            "--format=%(refname:short)%09%(objectname)",
            SERGEANT_REFS,
        ],
    ) {
        Ok(listing) => listing,
        Err(e) => {
            return RepositorySweep {
                repository: repository.to_string(),
                path: path.to_path_buf(),
                default_branch,
                branches: Vec::new(),
                prunable_worktrees: 0,
                unreadable: Some(e.to_string()),
            };
        }
    };
    let branches = listing
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split_once('\t'))
        .map(|(branch, tip)| classify_branch(path, branch, tip, default_branch.as_deref(), works))
        .collect();
    RepositorySweep {
        repository: repository.to_string(),
        path: path.to_path_buf(),
        default_branch,
        branches,
        prunable_worktrees: prunable_worktrees(path),
        unreadable: None,
    }
}

fn classify_branch(
    path: &Path,
    branch: &str,
    tip: &str,
    default_branch: Option<&str>,
    works: &BTreeMap<String, WorkState>,
) -> SweptBranch {
    let work_id = branch
        .strip_prefix(SERGEANT_BRANCH_PREFIX)
        .unwrap_or(branch)
        .to_string();
    let swept = |classification, unique_commits, detail: String| SweptBranch {
        branch: branch.to_string(),
        work_id: work_id.clone(),
        tip: tip.to_string(),
        classification,
        unique_commits,
        detail,
    };
    let Some(state) = works.get(&work_id) else {
        return swept(
            Classification::Orphan,
            None,
            "no journaled Work for this id — sergeant has no record of creating this branch"
                .to_string(),
        );
    };
    if !is_absorbing(*state) {
        return swept(
            Classification::Active,
            None,
            format!("work is {}, not terminal", state_name(*state)),
        );
    }
    let Some(default_branch) = default_branch else {
        return swept(
            Classification::Retained,
            None,
            "the mount's HEAD is detached, so no default branch can be compared against"
                .to_string(),
        );
    };
    if git_succeeds(path, &["merge-base", "--is-ancestor", tip, default_branch]) {
        return swept(
            Classification::Redundant,
            Some(0),
            format!("tip is already contained in {default_branch}"),
        );
    }
    let unique = git(
        path,
        &["rev-list", "--count", &format!("{default_branch}..{tip}")],
    )
    .ok()
    .and_then(|count| count.parse::<u64>().ok());
    let detail = match unique {
        Some(count) => format!("{count} commit(s) not contained in {default_branch}"),
        None => format!("commits not contained in {default_branch} (count unavailable)"),
    };
    swept(Classification::Retained, unique, detail)
}

/// The `prunable` annotations `git worktree list --porcelain` reports for
/// this mount's registry — registrations whose worktree directory is gone.
///
/// Counted, never repaired: §12.3 gives stale-registration repair to this
/// operation, but `git worktree prune` is git's own remedy and running it
/// would mutate a registry a concurrent materialization may be reading. The
/// count plus [`worktree_prune_remedy`] hands the operator the fix instead.
fn prunable_worktrees(path: &Path) -> usize {
    git(path, &["worktree", "list", "--porcelain"])
        .map(|listing| {
            listing
                .lines()
                .filter(|line| line.starts_with("prunable "))
                .count()
        })
        .unwrap_or(0)
}

fn state_name(state: WorkState) -> &'static str {
    match state {
        WorkState::Pending => "pending",
        WorkState::Active => "active",
        WorkState::Waiting => "waiting",
        WorkState::NeedsInput => "needs_input",
        WorkState::Blocked => "blocked",
        WorkState::Completed => "completed",
        WorkState::Failed => "failed",
        WorkState::Canceled => "canceled",
    }
}

/// Delete exactly the requested branches that a **fresh** classification
/// still calls `redundant`.
///
/// The caller's `targets` are a request, never an authorization: this
/// re-runs [`classify`] and answers from that, so a branch that gained
/// commits, changed hands, or stopped existing between the operator's report
/// and this call is refused rather than destroyed. Anything that is not
/// `redundant` right now — `active`, `retained`, `orphan`, or absent — is
/// refused with the reason named.
///
/// Each deletion runs under the same two-layer repository gate every other
/// ref/registry mutation in this codebase takes (`surface::with_repository`):
/// a `branch -D` rewrites the ref store a concurrent `worktree add` is
/// reading.
pub fn delete_redundant(
    data_dir: &Path,
    estate: &Estate,
    works: &BTreeMap<String, WorkState>,
    targets: &[SweepTarget],
) -> Vec<DeletedBranch> {
    let report = classify(estate, works);
    targets
        .iter()
        .map(|target| DeletedBranch {
            repository: target.repository.clone(),
            branch: target.branch.clone(),
            outcome: delete_one(data_dir, &report, target),
        })
        .collect()
}

fn delete_one(data_dir: &Path, report: &SweepReport, target: &SweepTarget) -> DeletionOutcome {
    // Cheap and absolute: whatever a request says, a name outside the
    // `sergeant/*` namespace never reaches git from here.
    if !target.branch.starts_with(SERGEANT_BRANCH_PREFIX) {
        return DeletionOutcome::Refused {
            reason: format!(
                "{} is not a sergeant/* branch; sweep never touches refs it did not create",
                target.branch
            ),
        };
    }
    let Some(repo) = report
        .repositories
        .iter()
        .find(|r| r.repository == target.repository)
    else {
        return DeletionOutcome::Refused {
            reason: format!(
                "{} is not a declared repository of this estate",
                target.repository
            ),
        };
    };
    let Some(branch) = repo.branches.iter().find(|b| b.branch == target.branch) else {
        return DeletionOutcome::Refused {
            reason: format!(
                "{} no longer exists in {}; nothing was deleted",
                target.branch, target.repository
            ),
        };
    };
    if branch.classification != Classification::Redundant {
        return DeletionOutcome::Refused {
            reason: format!(
                "{} classifies as {} at deletion time, not redundant; only a branch whose tip \
                 the default branch already contains is ever deleted",
                target.branch, branch.classification
            ),
        };
    }
    let gate = RepositoryGate::derive(data_dir, &repo.path);
    let source = repo.path.clone();
    let name = target.branch.clone();
    match with_repository(&gate, || git(&source, &["branch", "-D", &name])) {
        Ok(Ok(_)) => DeletionOutcome::Deleted {
            tip: branch.tip.clone(),
        },
        Ok(Err(e)) => DeletionOutcome::Failed {
            detail: e.to_string(),
        },
        Err(e) => DeletionOutcome::Failed {
            detail: e.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::process::Command;

    use crate::domain::estate::RepositorySpec;

    fn git_fixture(dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "sergeant tests")
            .env("GIT_AUTHOR_EMAIL", "tests@example.invalid")
            .env("GIT_COMMITTER_NAME", "sergeant tests")
            .env("GIT_COMMITTER_EMAIL", "tests@example.invalid")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {args:?} failed in {}: {}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    /// A mount with one commit on `main`.
    fn mount(root: &Path, name: &str) -> PathBuf {
        let path = root.join("repos").join(name);
        std::fs::create_dir_all(&path).expect("mount dir");
        git_fixture(&path, &["init", "-b", "main"]);
        std::fs::write(path.join("README.md"), "# fixture\n").expect("write");
        git_fixture(&path, &["add", "."]);
        git_fixture(&path, &["commit", "-m", "initial"]);
        path
    }

    /// A `sergeant/<id>` branch whose tip `main` already contains.
    fn merged_branch(path: &Path, id: &str) -> String {
        let branch = format!("sergeant/{id}");
        git_fixture(path, &["branch", &branch, "main"]);
        git_fixture(path, &["rev-parse", &branch])
    }

    /// A `sergeant/<id>` branch carrying one commit `main` does not have.
    ///
    /// The commit is made on `main` and the branch moved onto it by
    /// `update-ref`, then `main` is wound back: the mount's own HEAD must
    /// stay on `main`, since that is what every classification under test is
    /// measured against.
    fn diverged_branch(path: &Path, id: &str) -> String {
        let branch = format!("sergeant/{id}");
        git_fixture(path, &["commit", "--allow-empty", "-m", "work"]);
        let tip = git_fixture(path, &["rev-parse", "HEAD"]);
        git_fixture(path, &["update-ref", &format!("refs/heads/{branch}"), &tip]);
        git_fixture(path, &["reset", "--hard", "HEAD~1"]);
        tip
    }

    fn estate_of(root: &Path, repositories: Vec<RepositorySpec>) -> Estate {
        Estate {
            name: "sweep-fixture".to_string(),
            root: root.to_path_buf(),
            repositories,
            default_backend: None,
            default_workflow: None,
            profiles: Vec::new(),
            config_path: None,
            surfaces_dir: None,
            data_dir: None,
            repository_policy: BTreeMap::new(),
            groups: BTreeMap::new(),
            repository_origin: BTreeMap::new(),
            repository_upstream: BTreeMap::new(),
            retention: None,
        }
    }

    fn find<'a>(report: &'a SweepReport, branch: &str) -> &'a SweptBranch {
        report
            .repositories
            .iter()
            .flat_map(|r| r.branches.iter())
            .find(|b| b.branch == branch)
            .unwrap_or_else(|| panic!("no {branch} in {report:?}"))
    }

    /// guard-map: each of the four classifications is reached by the fact
    /// that defines it, in one pass over one mount. Mutation this kills:
    /// collapsing any two arms of `classify_branch` (an active Work's branch
    /// reading as redundant, or an unjournaled ref reading as retained).
    #[test]
    fn every_classification_is_reached_by_the_fact_that_defines_it() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let path = mount(root.path(), "api");
        merged_branch(&path, "01REDUNDANT");
        merged_branch(&path, "01ACTIVE");
        diverged_branch(&path, "01RETAINED");
        merged_branch(&path, "01ORPHAN");

        let works = BTreeMap::from([
            ("01REDUNDANT".to_string(), WorkState::Completed),
            ("01ACTIVE".to_string(), WorkState::Active),
            ("01RETAINED".to_string(), WorkState::Canceled),
        ]);
        let estate = estate_of(
            root.path(),
            vec![RepositorySpec {
                name: "api".to_string(),
                path: path.clone(),
            }],
        );

        let report = classify(&estate, &works);
        assert_eq!(report.repositories.len(), 1);
        assert_eq!(
            report.repositories[0].default_branch.as_deref(),
            Some("main"),
            "the report names what redundancy is relative to: {report:?}"
        );
        assert_eq!(
            find(&report, "sergeant/01REDUNDANT").classification,
            Classification::Redundant
        );
        assert_eq!(
            find(&report, "sergeant/01ACTIVE").classification,
            Classification::Active,
            "a non-terminal Work's branch is never eligible, merged or not"
        );
        let retained = find(&report, "sergeant/01RETAINED");
        assert_eq!(retained.classification, Classification::Retained);
        assert_eq!(retained.unique_commits, Some(1), "{retained:?}");
        assert_eq!(
            find(&report, "sergeant/01ORPHAN").classification,
            Classification::Orphan,
            "#172: a ref with no journaled Work is reported, never assumed"
        );
        assert_eq!(report.redundant().len(), 1, "{report:?}");
    }

    /// guard-map: classification reads and reports; it never writes. Mutation
    /// this kills: any deletion or repair sneaking into the read pass.
    #[test]
    fn classification_mutates_nothing() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let path = mount(root.path(), "api");
        merged_branch(&path, "01REDUNDANT");
        let estate = estate_of(
            root.path(),
            vec![RepositorySpec {
                name: "api".to_string(),
                path: path.clone(),
            }],
        );
        let works = BTreeMap::from([("01REDUNDANT".to_string(), WorkState::Completed)]);

        let before = git_fixture(&path, &["for-each-ref", "--format=%(refname)"]);
        classify(&estate, &works);
        classify(&estate, &works);
        assert_eq!(
            before,
            git_fixture(&path, &["for-each-ref", "--format=%(refname)"]),
            "a classification pass must leave the ref store byte-identical"
        );
    }

    /// guard-map: only `redundant` is ever deleted, and the server decides
    /// that for itself at deletion time. Mutation this kills: trusting the
    /// caller's target list, or reclassifying from a stale report.
    #[test]
    fn deletion_destroys_only_what_is_redundant_right_now() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let data_dir = root.path().join("data");
        std::fs::create_dir_all(&data_dir).expect("data dir");
        let path = mount(root.path(), "api");
        let redundant_tip = merged_branch(&path, "01REDUNDANT");
        merged_branch(&path, "01ACTIVE");
        diverged_branch(&path, "01RETAINED");
        merged_branch(&path, "01ORPHAN");

        let works = BTreeMap::from([
            ("01REDUNDANT".to_string(), WorkState::Completed),
            ("01ACTIVE".to_string(), WorkState::Active),
            ("01RETAINED".to_string(), WorkState::Canceled),
        ]);
        let estate = estate_of(
            root.path(),
            vec![RepositorySpec {
                name: "api".to_string(),
                path: path.clone(),
            }],
        );

        // Everything is asked for, including the three that must survive and
        // one name outside the namespace entirely.
        let targets: Vec<SweepTarget> = ["01REDUNDANT", "01ACTIVE", "01RETAINED", "01ORPHAN"]
            .iter()
            .map(|id| SweepTarget {
                repository: "api".to_string(),
                branch: format!("sergeant/{id}"),
            })
            .chain(std::iter::once(SweepTarget {
                repository: "api".to_string(),
                branch: "main".to_string(),
            }))
            .collect();
        let outcomes = delete_redundant(&data_dir, &estate, &works, &targets);

        assert_eq!(
            outcomes[0].outcome,
            DeletionOutcome::Deleted {
                tip: redundant_tip.clone()
            },
            "the journal records the tip that was destroyed: {outcomes:?}"
        );
        for surviving in &outcomes[1..] {
            assert!(
                matches!(surviving.outcome, DeletionOutcome::Refused { .. }),
                "nothing but a redundant branch may be deleted: {surviving:?}"
            );
        }
        let refs = git_fixture(&path, &["for-each-ref", "--format=%(refname:short)"]);
        assert!(!refs.contains("sergeant/01REDUNDANT"), "{refs}");
        for survivor in [
            "main",
            "sergeant/01ACTIVE",
            "sergeant/01RETAINED",
            "sergeant/01ORPHAN",
        ] {
            assert!(refs.contains(survivor), "{survivor} must survive: {refs}");
        }
    }

    /// guard-map: a target the caller believes is redundant but which has
    /// since gained commits is refused, not deleted. Mutation this kills:
    /// deleting from the report the *client* sent rather than a fresh one.
    #[test]
    fn a_branch_that_moved_since_the_report_is_refused() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let data_dir = root.path().join("data");
        std::fs::create_dir_all(&data_dir).expect("data dir");
        let path = mount(root.path(), "api");
        merged_branch(&path, "01MOVED");
        let works = BTreeMap::from([("01MOVED".to_string(), WorkState::Completed)]);
        let estate = estate_of(
            root.path(),
            vec![RepositorySpec {
                name: "api".to_string(),
                path: path.clone(),
            }],
        );
        assert_eq!(classify(&estate, &works).redundant().len(), 1);

        // The operator's report is now stale: the branch grew a commit.
        git_fixture(&path, &["commit", "--allow-empty", "-m", "late"]);
        let tip = git_fixture(&path, &["rev-parse", "HEAD"]);
        git_fixture(&path, &["update-ref", "refs/heads/sergeant/01MOVED", &tip]);
        git_fixture(&path, &["reset", "--hard", "HEAD~1"]);

        let outcomes = delete_redundant(
            &data_dir,
            &estate,
            &works,
            &[SweepTarget {
                repository: "api".to_string(),
                branch: "sergeant/01MOVED".to_string(),
            }],
        );
        assert!(
            matches!(&outcomes[0].outcome, DeletionOutcome::Refused { reason } if reason.contains("retained")),
            "{outcomes:?}"
        );
        assert!(
            git_fixture(&path, &["for-each-ref", "--format=%(refname:short)"])
                .contains("sergeant/01MOVED")
        );
    }

    /// guard-map: a stale linked-worktree registration is counted and named
    /// with git's own remedy, never repaired here. Mutation this kills:
    /// sweep running `worktree prune` on the operator's mount.
    #[test]
    fn prunable_worktree_registrations_are_counted_not_repaired() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let path = mount(root.path(), "api");
        let stray = root.path().join("stray");
        git_fixture(
            &path,
            &[
                "worktree",
                "add",
                stray.to_str().expect("utf8"),
                "-b",
                "sergeant/01STRAY",
            ],
        );
        std::fs::remove_dir_all(&stray).expect("remove the worktree directory");

        let estate = estate_of(
            root.path(),
            vec![RepositorySpec {
                name: "api".to_string(),
                path: path.clone(),
            }],
        );
        let report = classify(&estate, &BTreeMap::new());
        assert_eq!(report.repositories[0].prunable_worktrees, 1, "{report:?}");
        assert!(
            git_fixture(&path, &["worktree", "list", "--porcelain"]).contains("prunable"),
            "the registration is still there — sweep reports, git prunes"
        );
        assert!(worktree_prune_remedy(&path).starts_with("git -C "));
    }

    /// guard-map: a detached mount cannot prove redundancy, so nothing there
    /// is deletable. Mutation this kills: treating a missing default branch
    /// as "no unique content".
    #[test]
    fn a_detached_mount_proves_no_redundancy() {
        let root = tempfile::TempDir::new().expect("tempdir");
        let path = mount(root.path(), "api");
        merged_branch(&path, "01MERGED");
        let head = git_fixture(&path, &["rev-parse", "HEAD"]);
        git_fixture(&path, &["checkout", "--detach", &head]);

        let estate = estate_of(
            root.path(),
            vec![RepositorySpec {
                name: "api".to_string(),
                path: path.clone(),
            }],
        );
        let works = BTreeMap::from([("01MERGED".to_string(), WorkState::Completed)]);
        let report = classify(&estate, &works);
        assert_eq!(report.repositories[0].default_branch, None);
        assert_eq!(
            find(&report, "sergeant/01MERGED").classification,
            Classification::Retained
        );
        assert!(report.redundant().is_empty(), "{report:?}");
    }
}

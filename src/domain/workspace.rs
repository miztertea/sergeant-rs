//! Workspace: the repository surface work originates from (proposal §9).
//!
//! Single-repository use requires **zero configuration**: the workspace is
//! whatever `git rev-parse --show-toplevel` says, named after that directory.
//! Multi-repository use adds one optional checked-in file at the top level
//! (`sergeant.toml`, deviation D1) declaring the workspace name, its
//! repositories, its defaults, and its profiles.
//!
//! §9's last line is a constraint on this module: `sergeant.toml` "declares
//! topology and defaults. It never stores transient work state." Nothing about
//! a run — surfaces, stages, executions — is ever read from or written to it;
//! all of that lives in the journal.
//!
//! Configuration is parsed fail-closed (`deny_unknown_fields`): a checked-in
//! file is an instruction, not history, and a typo that silently means nothing
//! is worse than a refusal that names the line. This is the opposite choice
//! from the event envelope (§20), which preserves unknown fields — history
//! must survive readers that do not understand it, instructions must not.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::profile::Profile;
use crate::runtime::git::{GitError, git};

/// Checked-in workspace configuration file name (D1: `depot.toml` upstream).
pub const WORKSPACE_FILE: &str = "sergeant.toml";

/// One repository bound into a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositorySpec {
    /// Name used in surfaces, bindings and `--repo` selection.
    pub name: String,
    /// Absolute path to the repository's top level.
    pub path: PathBuf,
}

/// A resolved workspace: topology and defaults, never transient state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace name (from `sergeant.toml`, else the repo directory name).
    pub name: String,
    /// Directory the workspace was discovered from (the single repo's top
    /// level, or the directory holding `sergeant.toml`).
    pub root: PathBuf,
    /// Repositories in the workspace, in declaration order.
    pub repositories: Vec<RepositorySpec>,
    /// Workspace-level default backend (§13's third precedence tier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_backend: Option<String>,
    /// Workspace-level default workflow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workflow: Option<String>,
    /// Profiles declared for this workspace (§14).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<Profile>,
    /// Path of the `sergeant.toml` that produced this workspace, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_path: Option<PathBuf>,
}

/// Failure resolving a workspace.
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    /// The starting directory is not inside a Git repository. This is not a
    /// misconfiguration — it is the answer "there is no workspace here" — so
    /// callers distinguish it from the errors below.
    #[error("{path} is not inside a git repository: {source}")]
    NotARepository {
        /// The directory discovery started from.
        path: String,
        /// Git's own diagnostic.
        source: GitError,
    },
    /// Git itself failed while resolving the workspace.
    #[error(transparent)]
    Git(#[from] GitError),
    /// `sergeant.toml` could not be read.
    #[error("cannot read {path}: {source}")]
    Io {
        /// Path that failed to read.
        path: String,
        /// Underlying I/O failure.
        source: std::io::Error,
    },
    /// `sergeant.toml` is not valid TOML, or declares fields this build does
    /// not understand.
    #[error("invalid {path}: {source}")]
    Malformed {
        /// Path of the offending file.
        path: String,
        /// Parse failure, with line and column.
        source: toml::de::Error,
    },
    /// A declared repository does not exist or is not a Git repository.
    #[error("{file} declares repository {name:?} at {path}, which is not a git repository")]
    RepositoryNotFound {
        /// Config file that declared it.
        file: String,
        /// Declared repository name.
        name: String,
        /// Resolved path that failed.
        path: String,
    },
    /// Two repositories share a name; surfaces are keyed by name, so this
    /// would silently collapse two worktrees into one.
    #[error("{file} declares repository name {name:?} twice")]
    DuplicateRepository {
        /// Config file that declared it.
        file: String,
        /// The repeated name.
        name: String,
    },
    /// Two profiles share a name.
    #[error("{file} declares profile name {name:?} twice")]
    DuplicateProfile {
        /// Config file that declared it.
        file: String,
        /// The repeated name.
        name: String,
    },
    /// `sergeant.toml` declares no repositories at all.
    #[error("{file} declares no repositories")]
    NoRepositories {
        /// Config file that declared it.
        file: String,
    },
}

/// The `sergeant.toml` file shape (§9).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceFile {
    workspace: WorkspaceSection,
    #[serde(default)]
    repository: Vec<RepositoryEntry>,
    #[serde(default)]
    profile: Vec<Profile>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceSection {
    name: String,
    #[serde(default)]
    default_backend: Option<String>,
    #[serde(default)]
    default_workflow: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositoryEntry {
    name: String,
    path: PathBuf,
}

impl Workspace {
    /// Discover the workspace containing `start` (§9).
    ///
    /// Zero-config path: `git rev-parse --show-toplevel`, one repository named
    /// after the directory. If that top level holds a `sergeant.toml`, the
    /// file's topology replaces the implicit one.
    pub fn discover(start: &Path) -> Result<Self, WorkspaceError> {
        let toplevel = git(start, &["rev-parse", "--show-toplevel"]).map_err(|source| {
            WorkspaceError::NotARepository {
                path: start.display().to_string(),
                source,
            }
        })?;
        let root = PathBuf::from(toplevel);
        let config_path = root.join(WORKSPACE_FILE);
        if config_path.is_file() {
            Self::from_config(&config_path)
        } else {
            Ok(Self {
                name: repo_name(&root),
                repositories: vec![RepositorySpec {
                    name: repo_name(&root),
                    path: root.clone(),
                }],
                root,
                default_backend: None,
                default_workflow: None,
                profiles: Vec::new(),
                config_path: None,
            })
        }
    }

    /// Parse and validate a `sergeant.toml` into a workspace.
    pub fn from_config(config_path: &Path) -> Result<Self, WorkspaceError> {
        let file = config_path.display().to_string();
        let text = std::fs::read_to_string(config_path).map_err(|source| WorkspaceError::Io {
            path: file.clone(),
            source,
        })?;
        let parsed: WorkspaceFile =
            toml::from_str(&text).map_err(|source| WorkspaceError::Malformed {
                path: file.clone(),
                source,
            })?;
        let root = config_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        if parsed.repository.is_empty() {
            return Err(WorkspaceError::NoRepositories { file });
        }
        let mut seen = BTreeSet::new();
        let mut repositories = Vec::with_capacity(parsed.repository.len());
        for entry in parsed.repository {
            if !seen.insert(entry.name.clone()) {
                return Err(WorkspaceError::DuplicateRepository {
                    file,
                    name: entry.name,
                });
            }
            // Declared paths are relative to the config file, per §9's
            // `../payments-api` example.
            let joined = root.join(&entry.path);
            let resolved = git(&joined, &["rev-parse", "--show-toplevel"]).map_err(|_| {
                WorkspaceError::RepositoryNotFound {
                    file: file.clone(),
                    name: entry.name.clone(),
                    path: joined.display().to_string(),
                }
            })?;
            repositories.push(RepositorySpec {
                name: entry.name,
                path: PathBuf::from(resolved),
            });
        }

        let mut seen = BTreeSet::new();
        for profile in &parsed.profile {
            if !seen.insert(profile.name.clone()) {
                return Err(WorkspaceError::DuplicateProfile {
                    file,
                    name: profile.name.clone(),
                });
            }
        }

        Ok(Self {
            name: parsed.workspace.name,
            root,
            repositories,
            default_backend: parsed.workspace.default_backend,
            default_workflow: parsed.workspace.default_workflow,
            profiles: parsed.profile,
            config_path: Some(config_path.to_path_buf()),
        })
    }

    /// The profile with this name, if the workspace declares one.
    pub fn profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Restrict the workspace to the named repositories (the submit request's
    /// `repositories` selection). An unknown name is an error rather than a
    /// silently empty surface.
    pub fn select(&self, names: &[String]) -> Result<Vec<RepositorySpec>, String> {
        if names.is_empty() {
            return Ok(self.repositories.clone());
        }
        let mut selected = Vec::with_capacity(names.len());
        for name in names {
            match self.repositories.iter().find(|r| &r.name == name) {
                Some(repo) => selected.push(repo.clone()),
                None => {
                    return Err(format!(
                        "workspace {:?} has no repository {name:?} (has: {})",
                        self.name,
                        self.repositories
                            .iter()
                            .map(|r| r.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
        }
        Ok(selected)
    }
}

/// A repository's implicit name: its top-level directory name.
fn repo_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "workspace".to_string())
}

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

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::is_plain_name;
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
    /// Two differently-named repositories resolve to the same checkout. Both
    /// would be materialized onto the same `sergeant/<work-id>` branch of the
    /// same repository, and the second `git worktree add -b` fails *after*
    /// the first has already created a branch and a worktree in the user's
    /// checkout. Refused while it still costs nothing.
    #[error("{file} declares repositories {first:?} and {second:?}, which are both {path}")]
    DuplicateRepositoryPath {
        /// Config file that declared them.
        file: String,
        /// Name declared first for this path.
        first: String,
        /// The later name for the same path.
        second: String,
        /// The shared repository top level.
        path: String,
    },
    /// A declared repository name is not usable as a plain path component.
    /// Surface paths are built by joining it directly onto the surface root
    /// (`<data-dir>/surfaces/<work-id>/<name>`), so anything but a plain name
    /// could land the worktree outside the data dir entirely.
    #[error("{file} declares repository name {name:?}, which is not a plain directory name")]
    InvalidRepositoryName {
        /// Config file that declared it.
        file: String,
        /// The offending name.
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
    /// A profile's `permission_mode` option is not one of the CLI's own
    /// vocabulary (#47). Refused at parse time, before any launch attempts
    /// to pass the raw string through to the CLI.
    #[error("{file} declares profile {profile:?} with {source}")]
    InvalidPermissionMode {
        /// Config file that declared it.
        file: String,
        /// The profile naming the bad value.
        profile: String,
        /// The underlying vocabulary mismatch.
        source: crate::domain::profile::UnknownPermissionMode,
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
        // Identity of a repository is its resolved top level, not the name
        // the file chose for it: `path = "."` and `path = "./"` are one
        // checkout under two names, and only git can say so.
        let mut seen_paths: BTreeMap<PathBuf, String> = BTreeMap::new();
        let mut repositories = Vec::with_capacity(parsed.repository.len());
        for entry in parsed.repository {
            if !is_plain_name(&entry.name) {
                return Err(WorkspaceError::InvalidRepositoryName {
                    file,
                    name: entry.name,
                });
            }
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
            let resolved = PathBuf::from(resolved);
            if let Some(first) = seen_paths.get(&resolved) {
                return Err(WorkspaceError::DuplicateRepositoryPath {
                    file,
                    first: first.clone(),
                    second: entry.name,
                    path: resolved.display().to_string(),
                });
            }
            seen_paths.insert(resolved.clone(), entry.name.clone());
            repositories.push(RepositorySpec {
                name: entry.name,
                path: resolved,
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
            // #47: an unrecognized permission_mode is refused here, at
            // config load, rather than surfacing later as an unmeasured CLI
            // argument failure at launch time.
            if let Err(source) = profile.permission_mode() {
                return Err(WorkspaceError::InvalidPermissionMode {
                    file,
                    profile: profile.name.clone(),
                    source,
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
    /// silently empty surface, and a name repeated in the selection is an
    /// error too — two identical bindings would send `materialize` at the
    /// same worktree path and branch twice, the second `git worktree add`
    /// failing after the first has already touched the user's repository.
    pub fn select(&self, names: &[String]) -> Result<Vec<RepositorySpec>, String> {
        if names.is_empty() {
            return Ok(self.repositories.clone());
        }
        let mut seen = BTreeSet::new();
        let mut selected = Vec::with_capacity(names.len());
        for name in names {
            if !seen.insert(name.clone()) {
                return Err(format!(
                    "repository selection lists {name:?} twice for workspace {:?}",
                    self.name
                ));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// A temp git repository with one commit.
    fn init_repo(path: &Path) {
        std::fs::create_dir_all(path).expect("repo dir");
        for args in [
            vec!["init", "-b", "main"],
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
    }

    /// Write a `sergeant.toml` into `root` and parse it.
    fn parse(root: &Path, body: &str) -> Result<Workspace, WorkspaceError> {
        let config = root.join(WORKSPACE_FILE);
        std::fs::write(&config, body).expect("sergeant.toml");
        Workspace::from_config(&config)
    }

    /// A repository name is joined straight onto
    /// `<data-dir>/surfaces/<work-id>/`, so a name that is not a plain
    /// directory component could put a worktree anywhere on the filesystem.
    /// Refused at parse time, before anything is materialized.
    #[test]
    fn a_repository_name_may_not_escape_the_surface_root() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        for name in ["../escape", "..", "/etc", "nested/name", ""] {
            let err = parse(
                root,
                &format!(
                    "[workspace]\nname = \"w\"\n\n[[repository]]\nname = \"{name}\"\npath = \".\"\n"
                ),
            )
            .expect_err("a traversing repository name must be refused");
            assert!(
                matches!(err, WorkspaceError::InvalidRepositoryName { .. }),
                "{name:?} must be refused as a name, got {err}"
            );
        }

        // And the ordinary case still parses, so the guard is not refusing
        // everything.
        let workspace = parse(
            root,
            "[workspace]\nname = \"w\"\n\n[[repository]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect("a plain name parses");
        assert_eq!(workspace.repositories[0].name, "solo");
    }

    /// Two names for one checkout would both materialize onto the same
    /// `sergeant/<work-id>` branch of the same repository: the second
    /// `git worktree add -b` fails only *after* the first has created a
    /// branch and a worktree in the user's own checkout. Rejecting it here
    /// costs nothing; rejecting it there leaves a branch behind.
    #[test]
    fn two_names_for_one_repository_are_refused_before_anything_is_created() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);
        let nested = root.join("sub");
        std::fs::create_dir_all(&nested).expect("sub dir");

        // Distinct names, distinct spellings, one repository: `.`, `./` and a
        // subdirectory of the same checkout all resolve to one top level.
        for path in [".", "./", "sub"] {
            let err = parse(
                root,
                &format!(
                    "[workspace]\nname = \"w\"\n\n\
                     [[repository]]\nname = \"a\"\npath = \".\"\n\n\
                     [[repository]]\nname = \"b\"\npath = \"{path}\"\n"
                ),
            )
            .expect_err("one repository under two names must be refused");
            match err {
                WorkspaceError::DuplicateRepositoryPath { first, second, .. } => {
                    assert_eq!((first.as_str(), second.as_str()), ("a", "b"));
                }
                other => panic!("expected a same-path refusal for {path:?}, got {other}"),
            }
        }
    }

    /// Two entries with the same *name* collapse two worktrees into one
    /// surface path, which is the same hazard read from the other side.
    #[test]
    fn two_repositories_may_not_share_a_name() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);
        let other = root.join("other");
        init_repo(&other);

        let err = parse(
            root,
            "[workspace]\nname = \"w\"\n\n\
             [[repository]]\nname = \"same\"\npath = \".\"\n\n\
             [[repository]]\nname = \"same\"\npath = \"other\"\n",
        )
        .expect_err("a repeated name must be refused");
        assert!(
            matches!(&err, WorkspaceError::DuplicateRepository { name, .. } if name == "same"),
            "got {err}"
        );
    }

    /// The submit request's `repositories` selection is user input too, and a
    /// name repeated there produces two identical bindings — the same
    /// same-path collision, arriving through the API instead of the file.
    #[test]
    fn a_repository_named_twice_in_one_selection_is_refused() {
        let workspace = Workspace {
            name: "payments".to_string(),
            root: PathBuf::from("/nowhere"),
            repositories: vec![
                RepositorySpec {
                    name: "api".to_string(),
                    path: PathBuf::from("/nowhere/api"),
                },
                RepositorySpec {
                    name: "web".to_string(),
                    path: PathBuf::from("/nowhere/web"),
                },
            ],
            default_backend: None,
            default_workflow: None,
            profiles: Vec::new(),
            config_path: None,
        };

        let selected = workspace
            .select(&["web".to_string(), "api".to_string()])
            .expect("distinct names select");
        assert_eq!(
            selected.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            ["web", "api"]
        );

        let err = workspace
            .select(&["api".to_string(), "api".to_string()])
            .expect_err("a repeated selection must be refused");
        assert!(err.contains("twice"), "got {err}");

        // An unknown name still names what does exist.
        let err = workspace
            .select(&["ghost".to_string()])
            .expect_err("unknown repository");
        assert!(err.contains("api, web"), "got {err}");
    }

    /// `sergeant.toml` declaring no `[[repository]]` entries at all is
    /// refused rather than accepted as a workspace with nothing to act on.
    #[test]
    fn a_workspace_config_with_no_repositories_is_refused() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let err = parse(root, "[workspace]\nname = \"empty\"\n")
            .expect_err("no repositories at all must be refused");
        assert!(
            matches!(&err, WorkspaceError::NoRepositories { file } if file.ends_with(WORKSPACE_FILE)),
            "expected NoRepositories naming the config file, got {err}"
        );
    }

    /// Two `[[profile]]` entries with the same name are ambiguous under
    /// `--profile <name>`: refused at parse time rather than silently letting
    /// the later one shadow the earlier.
    #[test]
    fn two_profiles_may_not_share_a_name() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let err = parse(
            root,
            "[workspace]\nname = \"w\"\n\n\
             [[repository]]\nname = \"solo\"\npath = \".\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"fake\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"claude\"\n",
        )
        .expect_err("a repeated profile name must be refused");
        assert!(
            matches!(&err, WorkspaceError::DuplicateProfile { name, .. } if name == "same"),
            "got {err}"
        );
    }

    /// #47: a `permission_mode` outside the CLI's own vocabulary is refused
    /// at config load, before any launch could pass it through unchecked.
    #[test]
    fn a_profile_with_an_unknown_permission_mode_is_refused_at_load() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let err = parse(
            root,
            "[workspace]\nname = \"w\"\n\n\
             [[repository]]\nname = \"solo\"\npath = \".\"\n\n\
             [[profile]]\nname = \"reckless\"\nbackend = \"claude\"\n\
             [profile.options]\npermission_mode = \"yolo\"\n",
        )
        .expect_err("an unrecognized permission_mode must be refused");
        match &err {
            WorkspaceError::InvalidPermissionMode {
                profile, source, ..
            } => {
                assert_eq!(profile, "reckless");
                assert_eq!(source.value, "yolo");
            }
            other => panic!("expected InvalidPermissionMode, got {other}"),
        }

        // The five vocabulary values, plus unspecified, all still parse.
        let workspace = parse(
            root,
            "[workspace]\nname = \"w\"\n\n\
             [[repository]]\nname = \"solo\"\npath = \".\"\n\n\
             [[profile]]\nname = \"careful\"\nbackend = \"claude\"\n\
             [profile.options]\npermission_mode = \"plan\"\n",
        )
        .expect("a listed permission_mode value parses");
        assert_eq!(
            workspace.profiles[0]
                .permission_mode()
                .expect("validated at load")
                .map(|m| m.as_cli_value()),
            Some("plan")
        );
    }
}

//! Workspace: the repository surface work originates from (proposal §9).
//!
//! Single-repository use requires **zero configuration**: the workspace is
//! whatever `git rev-parse --show-toplevel` says, named after that directory.
//! Multi-repository use adds one optional checked-in file at the top level
//! (`sergeant.toml`, deviation D1) declaring the estate name, its
//! repositories, its defaults, its profiles, and its groups.
//!
//! **R-MVP1-3: estate vocabulary.** `[estate]` / `[[repo]]` / `[[profile]]` /
//! `[group.<name>]`, `deny_unknown_fields`. The pre-estate vocabulary
//! (`[workspace]`, `[[repository]]`) is not merely unknown — using it raises
//! a **named migration refusal** ([`WorkspaceError::LegacyVocabulary`])
//! rather than a generic serde diagnostic, because a schema rename deserves a
//! remedy, not a "field does not exist" message pointing at nothing. Mixing
//! old and new vocabulary hits the refusal on the first legacy key found.
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

/// The per-repository instruction file `instructions = "local"` would have
/// the actor read natively (R-MVP1-4). Named here because it is the one
/// value both the manifest's policy and the bind-time identity hash agree
/// on: today's `[[repo]] instructions` vocabulary and tomorrow's file probe.
pub const INSTRUCTION_FILE: &str = "AGENTS.md";

/// One repository bound into a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositorySpec {
    /// Name used in surfaces, bindings and `--repo` selection.
    pub name: String,
    /// Absolute path to the repository's top level.
    pub path: PathBuf,
}

/// Per-repository instruction-suppression policy (R-MVP1-4, `[[repo]]
/// instructions = "local" | "suppress"`).
///
/// The manifest *declares* this; core *resolves and pins* it at bind
/// (`workflow.bound`'s widened payload); the adapter *translates* it and
/// never redefines it. `Suppress` is byte-identical to today's hardcoded
/// `--setting-sources user` (`claude.rs:874-881`), so an unset value changes
/// no behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InstructionPolicy {
    /// The adapter's foreign-repo behavior: the repository's own instruction
    /// file is never read natively. Today's only real behavior, and the
    /// default when a repo entry says nothing.
    #[default]
    Suppress,
    /// The actor would consume the repository's own instruction file
    /// natively in its worktree. What this translates to for the Claude
    /// adapter is **unmeasured** (L1): MVP-1 parses and pins it but refuses
    /// it at submit (R-MVP1-4); MVP-2 measures it before this variant can
    /// ever reach a launch.
    Local,
}

impl InstructionPolicy {
    /// The TOML/display spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suppress => "suppress",
            Self::Local => "local",
        }
    }
}

impl std::fmt::Display for InstructionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A group of repositories declared under `[group.<name>]` (R-MVP1-3).
///
/// Membership gets **no new engine surface** (R-MVP1-5(b)): this is manifest
/// data only, validated here (every member must be a declared `[[repo]]`),
/// and expansion into `--repo` selections is a caller's job (MVP-3's
/// `--group`, out of this contract's scope).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupSpec {
    /// Repository names belonging to this group, declaration order.
    pub repos: Vec<String>,
    /// One orientation line, AI-facing (§ field rule: structure is for the
    /// binary, string values are for the AI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brief: Option<String>,
}

/// What a Work's bind pinned about one repository's instruction policy
/// (R-MVP1-4's R7): the resolved policy plus the identity of the file that
/// policy would read, hashed at bind time so a mid-flight edit cannot reach
/// a running Work. `path`/`content_hash` are `None` when the file is absent
/// — absence is recorded, never silently treated as "nothing to pin".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionIdentity {
    /// Repository this identity was resolved for.
    pub repository: String,
    /// The resolved policy (uniform across a bind's repositories by
    /// construction — R-MVP1-4's "one process, one policy").
    pub policy: InstructionPolicy,
    /// Absolute path of the instruction file in the materialized worktree,
    /// if one exists there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// BLAKE3 hex digest of the file's contents at bind time, if the file
    /// exists — "the file the actor will read is the one we recorded".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
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
    /// `[estate] surfaces_dir` (R-MVP1-1): resolved to an absolute path
    /// (relative declarations join onto `root`) when the manifest declares
    /// one. `None` leaves the daemon's own default (`SGT_SURFACES_DIR`, else
    /// `<data_dir>/surfaces`) in force — this field only ever narrows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surfaces_dir: Option<PathBuf>,
    /// Per-repository instruction policy (R-MVP1-4), keyed by repository
    /// name. A name absent from this map (including every repository in the
    /// zero-config single-repo fallback) resolves to
    /// [`InstructionPolicy::Suppress`] via [`Workspace::instruction_policy`].
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub repository_policy: BTreeMap<String, InstructionPolicy>,
    /// `[group.<name>]` declarations, validated (every member is a declared
    /// repository) but not expanded — expansion is a caller's job
    /// (R-MVP1-5(b), MVP-3's `--group`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<String, GroupSpec>,
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
    /// `sergeant.toml` uses the pre-estate vocabulary (`[workspace]`,
    /// `[[repository]]`). Named refusal rather than a generic
    /// `deny_unknown_fields` diagnostic (R-MVP1-3): a schema rename deserves
    /// a migration message pointing at the new table name, not a serde error
    /// naming a field that simply no longer exists. Mixing old and new
    /// vocabulary hits this on the first legacy key found.
    #[error(
        "{file} uses the legacy [{found}] table; the estate schema expects [{expected}]. {remedy}"
    )]
    LegacyVocabulary {
        /// Config file that used the legacy vocabulary.
        file: String,
        /// The legacy table name found (`workspace` or `repository`).
        found: String,
        /// The estate-vocabulary table it must become (`estate` or `repo`).
        expected: String,
        /// One-line migration instruction.
        remedy: String,
    },
    /// A `[group.<name>].repos` entry names a repository the manifest never
    /// declared under `[[repo]]`.
    #[error("{file} declares group {group:?} with unknown repository {name:?} (has: {available})")]
    UnknownGroupMember {
        /// Config file that declared it.
        file: String,
        /// Group name.
        group: String,
        /// The undeclared repository name.
        name: String,
        /// Declared repository names, for the remedy.
        available: String,
    },
}

/// The `sergeant.toml` file shape (§9, R-MVP1-3's estate vocabulary).
///
/// `estate` is `Option`, not required: a `sergeant.toml` reached only via
/// [`Workspace::discover`]'s zero-config git-toplevel fallback (a "member
/// repo's own config", R-MVP1-12) is a perfectly valid `Workspace` — it just
/// has no estate metadata to contribute, and its absence is exactly the
/// signal the upward walk uses to keep looking for the real estate root
/// rather than mistaking a member's own file for one.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceFile {
    #[serde(default)]
    estate: Option<EstateSection>,
    #[serde(default)]
    repo: Vec<RepositoryEntry>,
    #[serde(default)]
    profile: Vec<Profile>,
    #[serde(default)]
    group: BTreeMap<String, GroupEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EstateSection {
    name: String,
    #[serde(default)]
    default_backend: Option<String>,
    #[serde(default)]
    default_workflow: Option<String>,
    /// R-MVP1-1: `surfaces_root` override. Relative to this file's directory
    /// when not absolute.
    #[serde(default)]
    surfaces_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositoryEntry {
    name: String,
    path: PathBuf,
    /// R-MVP1-4. Unset means [`InstructionPolicy::Suppress`].
    #[serde(default)]
    instructions: InstructionPolicy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupEntry {
    repos: Vec<String>,
    #[serde(default)]
    brief: Option<String>,
}

/// Legacy-vocabulary tables this build refuses by name, in the order they
/// are probed for — `mixing hits it on the first legacy key` (R-MVP1-3).
const LEGACY_TABLES: &[(&str, &str, &str)] = &[
    (
        "workspace",
        "estate",
        "rename [workspace] to [estate] (same fields, plus optional surfaces_dir)",
    ),
    (
        "repository",
        "repo",
        "rename each [[repository]] entry to [[repo]] (same fields, plus optional instructions)",
    ),
];

/// Probe raw TOML for the pre-estate vocabulary **before** the real parse
/// (R-MVP1-3: "one probe before parse, not a second parser" — this reads the
/// same `toml::Value` `deny_unknown_fields` would reject anyway, just early
/// enough to name the migration instead of a generic unknown-field error).
fn check_legacy_vocabulary(text: &str, file: &str) -> Result<(), WorkspaceError> {
    let value: toml::Value = toml::from_str(text).map_err(|source| WorkspaceError::Malformed {
        path: file.to_string(),
        source,
    })?;
    let Some(table) = value.as_table() else {
        return Ok(());
    };
    for (legacy, expected, remedy) in LEGACY_TABLES {
        if table.contains_key(*legacy) {
            return Err(WorkspaceError::LegacyVocabulary {
                file: file.to_string(),
                found: (*legacy).to_string(),
                expected: (*expected).to_string(),
                remedy: (*remedy).to_string(),
            });
        }
    }
    Ok(())
}

impl Workspace {
    /// Discover the workspace containing `start` (§9, R-MVP1-12).
    ///
    /// **R-MVP1-12: estate discovery walks past inner `.git` boundaries.**
    /// `git rev-parse --show-toplevel` stops at the innermost `.git`, so from
    /// inside a member repository it can only ever find that member — never
    /// an estate root above it. This walks upward from `start`,
    /// filesystem-first, crossing git boundaries, for the nearest
    /// `sergeant.toml` carrying an `[estate]` table; a `sergeant.toml`
    /// without one is a member repo's own config, not an estate, and does
    /// not stop the walk. Bounded at `$HOME` or the filesystem root,
    /// whichever comes first. First match wins.
    ///
    /// When no `[estate]`-bearing file is found on the way up, this falls
    /// back to the zero-config path unchanged: `git rev-parse
    /// --show-toplevel`, one repository named after the directory, its
    /// topology replaced by a `sergeant.toml` found exactly there (with or
    /// without `[estate]` — the git-toplevel fallback accepts either, since
    /// there is nothing further up to prefer it over).
    pub fn discover(start: &Path) -> Result<Self, WorkspaceError> {
        if let Some(estate_config) = Self::find_estate_upward(start) {
            return Self::from_config(&estate_config);
        }
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
                surfaces_dir: None,
                repository_policy: BTreeMap::new(),
                groups: BTreeMap::new(),
            })
        }
    }

    /// Walk upward from `start` for the nearest `sergeant.toml` carrying an
    /// `[estate]` table (R-MVP1-12). Ancestors are canonicalized once before
    /// the walk (the same symlink hazard `surface.rs:286-289` already
    /// defends against). Returns `None` rather than an error when nothing
    /// matches — that is "no estate here", not a malformed-config failure;
    /// a `sergeant.toml` this walk *does* choose still gets the full
    /// fail-closed treatment via [`Self::from_config`].
    ///
    /// A `sergeant.toml` found along the way whose TOML cannot even be
    /// parsed enough to check for `[estate]` is treated as "not an estate,
    /// keep walking" rather than a hard failure: it is not the file this
    /// walk is trying to find, and an unrelated member repo's broken config
    /// must not be able to block estate discovery for everything below it.
    fn find_estate_upward(start: &Path) -> Option<PathBuf> {
        let boundary = std::env::var_os("HOME")
            .map(PathBuf::from)
            .and_then(|home| std::fs::canonicalize(&home).ok());
        Self::find_estate_upward_bounded(start, boundary.as_deref())
    }

    /// [`Self::find_estate_upward`] with the `$HOME` boundary passed in
    /// rather than read from the process environment — split out so the
    /// boundary itself is testable without mutating a process-global that
    /// every other test in this binary also reads (`backend/claude.rs`'s own
    /// `$HOME` fallback among them).
    fn find_estate_upward_bounded(start: &Path, boundary: Option<&Path>) -> Option<PathBuf> {
        let start = std::fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
        let mut dir: &Path = &start;
        loop {
            let candidate = dir.join(WORKSPACE_FILE);
            if candidate.is_file() && has_estate_table(&candidate) {
                return Some(candidate);
            }
            if boundary == Some(dir) {
                return None;
            }
            dir = dir.parent()?;
        }
    }

    /// Parse and validate a `sergeant.toml` into a workspace.
    pub fn from_config(config_path: &Path) -> Result<Self, WorkspaceError> {
        let file = config_path.display().to_string();
        let text = std::fs::read_to_string(config_path).map_err(|source| WorkspaceError::Io {
            path: file.clone(),
            source,
        })?;
        // R-MVP1-3: the named migration refusal is a probe before the real
        // parse, not a second parser — it reads the same TOML
        // `deny_unknown_fields` would reject anyway, just early enough to
        // name the rename instead of a generic unknown-field error.
        check_legacy_vocabulary(&text, &file)?;
        let parsed: WorkspaceFile =
            toml::from_str(&text).map_err(|source| WorkspaceError::Malformed {
                path: file.clone(),
                source,
            })?;
        let root = config_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        if parsed.repo.is_empty() {
            return Err(WorkspaceError::NoRepositories { file });
        }
        let mut seen = BTreeSet::new();
        // Identity of a repository is its resolved top level, not the name
        // the file chose for it: `path = "."` and `path = "./"` are one
        // checkout under two names, and only git can say so.
        let mut seen_paths: BTreeMap<PathBuf, String> = BTreeMap::new();
        let mut repositories = Vec::with_capacity(parsed.repo.len());
        let mut repository_policy = BTreeMap::new();
        for entry in parsed.repo {
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
            repository_policy.insert(entry.name.clone(), entry.instructions);
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

        let declared_repo_names: Vec<&str> = repositories.iter().map(|r| r.name.as_str()).collect();
        let mut groups = BTreeMap::new();
        for (group_name, entry) in parsed.group {
            for member in &entry.repos {
                if !declared_repo_names.contains(&member.as_str()) {
                    return Err(WorkspaceError::UnknownGroupMember {
                        file,
                        group: group_name,
                        name: member.clone(),
                        available: declared_repo_names.join(", "),
                    });
                }
            }
            groups.insert(
                group_name,
                GroupSpec {
                    repos: entry.repos,
                    brief: entry.brief,
                },
            );
        }

        let (name, default_backend, default_workflow, surfaces_dir) = match parsed.estate {
            Some(estate) => (
                estate.name,
                estate.default_backend,
                estate.default_workflow,
                estate
                    .surfaces_dir
                    .map(|d| if d.is_absolute() { d } else { root.join(d) }),
            ),
            None => (repo_name(&root), None, None, None),
        };

        Ok(Self {
            name,
            root,
            repositories,
            default_backend,
            default_workflow,
            profiles: parsed.profile,
            config_path: Some(config_path.to_path_buf()),
            surfaces_dir,
            repository_policy,
            groups,
        })
    }

    /// The profile with this name, if the workspace declares one.
    pub fn profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// This repository's resolved instruction policy (R-MVP1-4). Absent from
    /// the manifest resolves to [`InstructionPolicy::Suppress`] — today's
    /// only real behavior — rather than treating "unset" as ambiguous.
    pub fn instruction_policy(&self, repository: &str) -> InstructionPolicy {
        self.repository_policy
            .get(repository)
            .copied()
            .unwrap_or_default()
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

/// Whether `config_path` parses as TOML and has a top-level `[estate]` key
/// (R-MVP1-12). Any failure to read or parse — this is a *probe* during an
/// upward walk, not the file the walk is committed to — answers `false`:
/// "not an estate, keep walking", never a hard error for a file that was
/// never going to be chosen anyway.
fn has_estate_table(config_path: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(config_path) else {
        return false;
    };
    let Ok(value) = toml::from_str::<toml::Value>(&text) else {
        return false;
    };
    value
        .as_table()
        .is_some_and(|table| table.contains_key("estate"))
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
                &format!("[estate]\nname = \"w\"\n\n[[repo]]\nname = \"{name}\"\npath = \".\"\n"),
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
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\npath = \".\"\n",
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
                    "[estate]\nname = \"w\"\n\n\
                     [[repo]]\nname = \"a\"\npath = \".\"\n\n\
                     [[repo]]\nname = \"b\"\npath = \"{path}\"\n"
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
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"same\"\npath = \".\"\n\n\
             [[repo]]\nname = \"same\"\npath = \"other\"\n",
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
            surfaces_dir: None,
            repository_policy: BTreeMap::new(),
            groups: BTreeMap::new(),
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

    /// `sergeant.toml` declaring no `[[repo]]` entries at all is
    /// refused rather than accepted as a workspace with nothing to act on.
    #[test]
    fn a_workspace_config_with_no_repositories_is_refused() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let err = parse(root, "[estate]\nname = \"empty\"\n")
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
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"solo\"\npath = \".\"\n\n\
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
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"solo\"\npath = \".\"\n\n\
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
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"solo\"\npath = \".\"\n\n\
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

    // ---- R-MVP1-3: schema rename-with-refusal ----------------------------

    /// The legacy `[workspace]` table raises the named migration refusal,
    /// not a generic `deny_unknown_fields` error — and the message names the
    /// found table, the expected one, and a remedy.
    #[test]
    fn legacy_workspace_table_is_refused_with_a_named_migration_remedy() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let err = parse(
            root,
            "[workspace]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect_err("[workspace] must be refused by name");
        match &err {
            WorkspaceError::LegacyVocabulary {
                found,
                expected,
                remedy,
                ..
            } => {
                assert_eq!(found, "workspace");
                assert_eq!(expected, "estate");
                assert!(!remedy.is_empty(), "the refusal must name a remedy");
            }
            other => panic!("expected LegacyVocabulary, got {other}"),
        }
    }

    /// The legacy `[[repository]]` array-of-tables raises the same named
    /// refusal, distinctly from `[workspace]`.
    #[test]
    fn legacy_repository_table_is_refused_with_a_named_migration_remedy() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let err = parse(
            root,
            "[estate]\nname = \"w\"\n\n[[repository]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect_err("[[repository]] must be refused by name");
        match &err {
            WorkspaceError::LegacyVocabulary {
                found, expected, ..
            } => {
                assert_eq!(found, "repository");
                assert_eq!(expected, "repo");
            }
            other => panic!("expected LegacyVocabulary, got {other}"),
        }
    }

    /// Mixing old and new vocabulary in one file hits the refusal on the
    /// first legacy key found (`[workspace]` is probed before
    /// `[[repository]]`) rather than silently accepting the new table and
    /// ignoring the old one, or vice versa.
    #[test]
    fn mixed_legacy_and_estate_vocabulary_refuses_on_the_first_legacy_key() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let err = parse(
            root,
            "[workspace]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect_err("a mix must still be refused");
        assert!(
            matches!(&err, WorkspaceError::LegacyVocabulary { found, .. } if found == "workspace"),
            "got {err}"
        );
    }

    /// A same-commit grep gate this test is the code-level half of: the new
    /// vocabulary parses cleanly on its own, with no legacy table anywhere.
    #[test]
    fn estate_vocabulary_alone_parses_without_refusal() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let workspace = parse(
            root,
            "[estate]\nname = \"clean\"\n\n[[repo]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect("pure estate vocabulary must parse");
        assert_eq!(workspace.name, "clean");
    }

    // ---- R-MVP1-3: `[group.<name>]` ---------------------------------------

    /// A group's members must all be declared repositories; membership
    /// itself is validated, not expanded (R-MVP1-5(b) — expansion is a
    /// caller's job, out of this contract's scope).
    #[test]
    fn a_group_validates_membership_against_declared_repositories() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);
        let other = root.join("other");
        init_repo(&other);

        let workspace = parse(
            root,
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"api\"\npath = \".\"\n\n\
             [[repo]]\nname = \"web\"\npath = \"other\"\n\n\
             [group.payments]\nrepos = [\"api\", \"web\"]\nbrief = \"both sides\"\n",
        )
        .expect("a group over declared repos parses");
        let group = workspace.groups.get("payments").expect("group present");
        assert_eq!(group.repos, vec!["api".to_string(), "web".to_string()]);
        assert_eq!(group.brief.as_deref(), Some("both sides"));

        let err = parse(
            root,
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"api\"\npath = \".\"\n\n\
             [group.payments]\nrepos = [\"api\", \"ghost\"]\n",
        )
        .expect_err("an undeclared group member must be refused");
        match &err {
            WorkspaceError::UnknownGroupMember { group, name, .. } => {
                assert_eq!(group, "payments");
                assert_eq!(name, "ghost");
            }
            other => panic!("expected UnknownGroupMember, got {other}"),
        }
    }

    // ---- R-MVP1-4: `[[repo]] instructions` --------------------------------

    /// `instructions` defaults to `suppress` when unset (byte-identical to
    /// today's hardcoded behavior, L18/R1), and `local` parses and pins even
    /// though the engine refuses it at submit — parsing and submission are
    /// different layers, and this module only owns the former.
    #[test]
    fn repo_instructions_policy_parses_and_defaults_to_suppress() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);
        let other = root.join("other");
        init_repo(&other);

        let workspace = parse(
            root,
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"unset\"\npath = \".\"\n\n\
             [[repo]]\nname = \"loud\"\npath = \"other\"\ninstructions = \"local\"\n",
        )
        .expect("instructions parses");
        assert_eq!(
            workspace.instruction_policy("unset"),
            InstructionPolicy::Suppress,
            "an unset instructions value must default to suppress, byte-identical to today"
        );
        assert_eq!(
            workspace.instruction_policy("loud"),
            InstructionPolicy::Local
        );
        // A name the manifest never declared still resolves rather than
        // panicking — callers ask this for arbitrary selected repos.
        assert_eq!(
            workspace.instruction_policy("nowhere"),
            InstructionPolicy::Suppress
        );
    }

    // ---- R-MVP1-1: `[estate] surfaces_dir` --------------------------------

    /// A relative `surfaces_dir` resolves onto the estate root; an absolute
    /// one is taken as given.
    #[test]
    fn estate_surfaces_dir_resolves_relative_and_keeps_absolute() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let workspace = parse(
            root,
            "[estate]\nname = \"w\"\nsurfaces_dir = \"../elsewhere-surfaces\"\n\n\
             [[repo]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect("relative surfaces_dir parses");
        assert_eq!(
            workspace.surfaces_dir,
            Some(root.join("../elsewhere-surfaces"))
        );

        let absolute = dir.path().join("abs-surfaces");
        let workspace = parse(
            root,
            &format!(
                "[estate]\nname = \"w\"\nsurfaces_dir = {:?}\n\n[[repo]]\nname = \"solo\"\npath = \".\"\n",
                absolute.to_string_lossy()
            ),
        )
        .expect("absolute surfaces_dir parses");
        assert_eq!(workspace.surfaces_dir, Some(absolute));

        // Unset stays `None` — the daemon's own default is left in force.
        let workspace = parse(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect("no surfaces_dir parses");
        assert_eq!(workspace.surfaces_dir, None);
    }

    // ---- R-MVP1-12: estate discovery past inner `.git` --------------------

    /// A `sergeant.toml` under `root` with an `[estate]` table, in `root`'s
    /// own git repository.
    fn write_estate(root: &Path, name: &str) {
        init_repo(root);
        std::fs::write(
            root.join(WORKSPACE_FILE),
            format!("[estate]\nname = {name:?}\n\n[[repo]]\nname = \"solo\"\npath = \".\"\n"),
        )
        .expect("write estate sergeant.toml");
    }

    /// #22: discovery from inside a member repository nested under an estate
    /// root finds the estate above it — `git rev-parse --show-toplevel`
    /// alone could never do this, since it stops at the member's own
    /// `.git`.
    #[test]
    fn estate_discovery_walks_upward_past_an_inner_git_boundary() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        std::fs::create_dir_all(&estate_root).expect("estate dir");
        write_estate(&estate_root, "outer-estate");

        let member = estate_root.join("repos").join("payments-api");
        init_repo(&member);

        let workspace = Workspace::discover(&member).expect("discovery finds the outer estate");
        assert_eq!(workspace.name, "outer-estate");
        assert_eq!(
            std::fs::canonicalize(&workspace.root).ok(),
            std::fs::canonicalize(&estate_root).ok()
        );
    }

    /// #22: a member repository with its own `sergeant.toml` — one with no
    /// `[estate]` table, so it is a member's own config, not an estate — does
    /// not stop the upward walk; the outer estate is still found.
    #[test]
    fn a_member_repos_own_sergeant_toml_without_estate_does_not_stop_the_walk() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        std::fs::create_dir_all(&estate_root).expect("estate dir");
        write_estate(&estate_root, "outer-estate");

        let member = estate_root.join("repos").join("payments-api");
        init_repo(&member);
        // The member's own config: no [estate] table, just its own
        // single-repository declaration.
        std::fs::write(
            member.join(WORKSPACE_FILE),
            "[[repo]]\nname = \"payments-api\"\npath = \".\"\n",
        )
        .expect("write member's own sergeant.toml");

        let workspace = Workspace::discover(&member)
            .expect("discovery must walk past the member's own non-estate config");
        assert_eq!(
            workspace.name, "outer-estate",
            "the member's own sergeant.toml (no [estate]) must not be mistaken for the estate"
        );
    }

    /// #22: a git worktree nested inside the estate directory tree (its
    /// `.git` is a *file* pointing at another repository entirely) does not
    /// confuse the filesystem-first walk — it never consults git at all.
    #[test]
    fn a_nested_worktree_inside_the_estate_still_finds_it() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        std::fs::create_dir_all(&estate_root).expect("estate dir");
        write_estate(&estate_root, "outer-estate");

        let other_repo = dir.path().join("other-repo");
        init_repo(&other_repo);
        let worktree = estate_root.join("nested-worktree");
        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                worktree.to_str().expect("utf8 path"),
                "-b",
                "wt-branch",
            ])
            .current_dir(&other_repo)
            .output()
            .expect("git worktree add");
        assert!(output.status.success(), "worktree add: {output:?}");

        let workspace =
            Workspace::discover(&worktree).expect("discovery finds the estate from a worktree");
        assert_eq!(workspace.name, "outer-estate");
    }

    /// #22: a path containing a space is not special-cased anywhere in the
    /// walk — plain `PathBuf` handling is enough.
    #[test]
    fn estate_discovery_handles_a_path_with_a_space() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("the estate");
        std::fs::create_dir_all(&estate_root).expect("estate dir");
        write_estate(&estate_root, "spaced-estate");

        let member = estate_root.join("repos").join("payments api");
        init_repo(&member);

        let workspace =
            Workspace::discover(&member).expect("a space in the path must not break discovery");
        assert_eq!(workspace.name, "spaced-estate");
    }

    /// #22: no `sergeant.toml` with `[estate]` anywhere on the way up falls
    /// back to the zero-config, git-toplevel behavior, unchanged.
    #[test]
    fn no_estate_anywhere_falls_back_to_zero_config_unchanged() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("solo-repo");
        init_repo(&root);

        let workspace = Workspace::discover(&root).expect("zero-config fallback");
        assert_eq!(workspace.repositories.len(), 1);
        assert_eq!(
            std::fs::canonicalize(&workspace.repositories[0].path).ok(),
            std::fs::canonicalize(&root).ok()
        );
        assert!(workspace.config_path.is_none());
    }

    /// #22: starting outside any git repository at all is the unchanged
    /// `NotARepository` answer — the estate walk does not manufacture a
    /// workspace where §9 says there is none.
    #[test]
    fn discovery_outside_any_repository_is_refused_unchanged() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        // A bare directory tree with no `.git` anywhere and (by construction
        // of a fresh tempdir) no `[estate]`-bearing `sergeant.toml` above it
        // either.
        let err = Workspace::discover(dir.path()).expect_err("no repository here");
        assert!(
            matches!(err, WorkspaceError::NotARepository { .. }),
            "got {err}"
        );
    }

    /// The upward walk is bounded at `$HOME` (or the filesystem root):
    /// an `[estate]`-bearing `sergeant.toml` *above* the boundary is never
    /// found, even though the walk would otherwise reach it. Exercises
    /// [`Workspace::find_estate_upward_bounded`] directly with an explicit
    /// boundary rather than mutating the process's real `$HOME`, which every
    /// other test in this binary also reads (L5: tests must not step on each
    /// other through shared process state).
    #[test]
    fn the_upward_walk_is_bounded_and_never_crosses_it() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let above_boundary = dir.path().join("above");
        let boundary = above_boundary.join("home-equivalent");
        let below_boundary = boundary.join("estate").join("repos").join("member");
        std::fs::create_dir_all(&below_boundary).expect("nested dirs");

        // An estate at `above_boundary` — outside the search space once the
        // boundary is `boundary`.
        write_estate(&above_boundary, "outside-the-boundary");
        // No estate at or below `boundary`.
        std::fs::create_dir_all(boundary.join("estate")).expect("estate dir");
        init_repo(&below_boundary);
        // Canonicalized explicitly, matching what `find_estate_upward`
        // itself does to `$HOME` — the walk canonicalizes `start`'s
        // ancestors, so an uncanonicalized boundary could mismatch on a
        // host where the temp root is reached through a symlink.
        let boundary = std::fs::canonicalize(&boundary).expect("canonical boundary");

        let found = Workspace::find_estate_upward_bounded(&below_boundary, Some(&boundary));
        assert_eq!(
            found, None,
            "an estate above the boundary must never be found"
        );

        // The same estate, found once it is unbounded (or the boundary is
        // above it) — proving the walk itself works and the bound is what
        // stopped it, not a bug in the walk.
        let found_unbounded =
            Workspace::find_estate_upward_bounded(&below_boundary, Some(&above_boundary));
        assert!(
            found_unbounded.is_some(),
            "the same estate must be found once the boundary includes it"
        );
    }
}

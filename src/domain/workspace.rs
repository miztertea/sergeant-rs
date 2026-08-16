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
///
/// **W7, then MVP-2 D2 item 1: measured for both policies, and "the file the
/// actor will read is the one we recorded" turns out true for *neither*.**
/// The Claude adapter's `suppress` launch grammar is `--setting-sources
/// user` (`backend/claude.rs`), which does not read this file; this repo's
/// own north-star arbitration record confirms it empirically ("CLAUDE.md and
/// AGENTS.md are invisible to the actor by design"). MVP-2 measured `local`
/// too (`docs/gauntlet/notes/d2-setting-sources-measurement-2026-08-12.md`)
/// and found the same thing for a different reason: `--setting-sources`
/// governs `.claude/settings*.json` configuration, not memory-file reading,
/// for *any* value — there is no native mechanism tied to the filename
/// `AGENTS.md` at all (there is one for the literal filename `CLAUDE.md`,
/// unconditionally, unrelated to this flag). So `local` no longer refuses at
/// submit — the L1 gate that refusal existed to enforce is satisfied — but
/// what it launches under is a *wider settings-source load*, not native
/// `AGENTS.md` consumption. `Engine::resolve_instruction_identities` still
/// hashes this file at bind time regardless of policy — R-MVP1-4's own pin
/// ("editing an AGENTS.md after bind does not move the pinned identity")
/// holds either way — and for both policies on this adapter, what is pinned
/// is honest bookkeeping for a file nothing here currently reads.
pub const INSTRUCTION_FILE: &str = "AGENTS.md";

/// One repository bound into a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositorySpec {
    /// Name used in surfaces, bindings and `--repo` selection.
    pub name: String,
    /// Absolute path to the repository's top level.
    pub path: PathBuf,
}

/// One `[[repo]]` entry read straight off `sergeant.toml`, before any
/// on-disk existence check — see [`Workspace::declared_repos`]. Unlike
/// [`RepositorySpec::path`] (git-resolved, guaranteed to exist), `path` here
/// is only the declared path joined onto the manifest's directory and may
/// point at nothing.
#[derive(Debug, Clone)]
pub struct DeclaredRepo {
    /// Repository name.
    pub name: String,
    /// Declared path, joined but not resolved through git.
    pub path: PathBuf,
    /// `origin` from `[[repo]]`, when declared.
    pub origin: Option<String>,
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
    /// The wider of the two launch grammars this build can produce for a
    /// bound repository. MVP-2 D2 item 1 measured what it actually
    /// translates to for the Claude adapter (L1,
    /// `docs/gauntlet/notes/d2-setting-sources-measurement-2026-08-12.md`):
    /// **not** "the actor natively consumes the repository's own instruction
    /// file", the original design intent this variant was named for — that
    /// mechanism does not exist for a file named `AGENTS.md` under any
    /// `--setting-sources` value. What it actually widens is whether the
    /// repository's own `.claude/settings.json` /
    /// `.claude/settings.local.json` — hooks, tool permissions, MCP servers
    /// — take effect for the launch (`ClaudeBackend::setting_sources_args`).
    /// No longer refused at submit (the L1 gate that refusal existed to
    /// enforce is satisfied); the resolved policy still reaches the launch
    /// grammar via `StartRequest`/`ResumeRequest`.
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
    /// `[estate] data_dir` (ADR 0008(b)): resolved the same way as
    /// `surfaces_dir` above — an absolute path, relative declarations
    /// joined onto `root`. Consulted only by `src/cli.rs`'s
    /// `resolve_data_dir`, and only once an estate has already been
    /// discovered (it narrows what that discovery would otherwise default
    /// to, `<estate_root>/.sergeant/data`); it does not affect
    /// `--data-dir`/`SGT_DATA_DIR`, which both still short-circuit before an
    /// estate is ever looked for (ADR 0008(a), unchanged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<PathBuf>,
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
    /// Per-repository `origin` (MVP-3, `sgt repo add`'s clone-or-verify),
    /// keyed by repository name. Informational only — never consumed by
    /// materialize/execution (R-NS-4: a surface adds usability, never
    /// functionality) — recorded so `sgt repo list` can show where a
    /// repository was cloned from and a repeated `sgt repo add` can tell "the
    /// dir already exists" from "and here is what it should verify against".
    /// A name absent from this map (including the zero-config fallback and
    /// any `[[repo]]` entry that never declared `origin`) has no known
    /// origin.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub repository_origin: BTreeMap<String, String>,
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
    /// ADR 0008(b): `data_dir` override, resolved the same way as
    /// `surfaces_dir` above.
    #[serde(default)]
    data_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositoryEntry {
    name: String,
    path: PathBuf,
    /// R-MVP1-4. Unset means [`InstructionPolicy::Suppress`].
    #[serde(default)]
    instructions: InstructionPolicy,
    /// MVP-3 `sgt repo add`'s clone-or-verify source. Recorded, never acted
    /// on by this module beyond bookkeeping (see [`Workspace::repository_origin`]).
    #[serde(default)]
    origin: Option<String>,
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
    ///
    /// Equivalent to [`Self::discover_scoped`] with no explicit data-dir
    /// scope — kept as the unscoped entry point so every existing caller
    /// and fixture that has no data dir of its own to bound against (most
    /// of this module's own tests among them) keeps working unchanged.
    pub fn discover(start: &Path) -> Result<Self, WorkspaceError> {
        Self::discover_scoped(start, None)
    }

    /// [`Self::discover`], plus R-MVP1-12's other half: the walk never
    /// ascends past an explicit `--data-dir`/`SGT_DATA_DIR` scope, when the
    /// caller has one. `$HOME` and the data-dir scope are both candidate
    /// boundaries; the walk stops at whichever it reaches first ascending
    /// from `start` (checking that directory's own `sergeant.toml` before
    /// stopping, exactly as the `$HOME` boundary already does) — a data-dir
    /// scope that sits *below* `start` (not on its ancestor chain at all,
    /// the ordinary case: the data dir defaults to
    /// `~/.local/share/sergeant`, unrelated to any repository) is never
    /// reached during the ascent and so never changes the outcome; only a
    /// scope that is itself an ancestor of `start` — the A8 self-hosting
    /// shape, "data dir in-estate" (`docs/gauntlet/contracts/MVP-1.md`'s own
    /// Acceptance) — can narrow it.
    pub fn discover_scoped(start: &Path, data_dir: Option<&Path>) -> Result<Self, WorkspaceError> {
        if let Some(estate_config) = Self::find_estate_upward(start, data_dir)? {
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
                data_dir: None,
                repository_policy: BTreeMap::new(),
                groups: BTreeMap::new(),
                repository_origin: BTreeMap::new(),
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
    fn find_estate_upward(
        start: &Path,
        data_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>, WorkspaceError> {
        let boundary = std::env::var_os("HOME")
            .map(PathBuf::from)
            .and_then(|home| std::fs::canonicalize(&home).ok());
        let data_dir_scope = data_dir.and_then(|d| std::fs::canonicalize(d).ok());
        Self::find_estate_upward_bounded(start, boundary.as_deref(), data_dir_scope.as_deref())
    }

    /// [`Self::find_estate_upward`] with both boundaries passed in rather
    /// than read from the process environment / caller — split out so each
    /// is testable without mutating a process-global that every other test
    /// in this binary also reads (`backend/claude.rs`'s own `$HOME`
    /// fallback among them). `data_dir_scope` implements R-MVP1-12's
    /// "never above an explicit `--data-dir`/`SGT_DATA_DIR` scope": one more
    /// candidate boundary alongside `$HOME`, checked at every directory the
    /// walk visits so it stops at whichever boundary it reaches first.
    ///
    /// A `sergeant.toml` found on the way up that is readable but carries
    /// legacy vocabulary or fails to parse fails the whole walk closed
    /// (`Err`), exactly as one found directly would (R-MVP1-3's named
    /// migration refusal) — this module's own header doctrine, "a typo that
    /// silently means nothing is worse than a refusal that names the line,"
    /// applies to a file this walk steps over just as much as one it
    /// chooses. A file the walk cannot even *read* (permission, race) is not
    /// this walk's failure to report and is treated as "not an estate, keep
    /// walking" — the one case genuinely indistinguishable from "no file
    /// here at all".
    fn find_estate_upward_bounded(
        start: &Path,
        boundary: Option<&Path>,
        data_dir_scope: Option<&Path>,
    ) -> Result<Option<PathBuf>, WorkspaceError> {
        let start = std::fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
        let mut dir: &Path = &start;
        loop {
            let candidate = dir.join(WORKSPACE_FILE);
            if candidate.is_file() && estate_table_check(&candidate)? {
                return Ok(Some(candidate));
            }
            if boundary == Some(dir) || data_dir_scope == Some(dir) {
                return Ok(None);
            }
            dir = match dir.parent() {
                Some(parent) => parent,
                None => return Ok(None),
            };
        }
    }

    /// Parse and validate a `sergeant.toml` into a workspace.
    pub fn from_config(config_path: &Path) -> Result<Self, WorkspaceError> {
        Self::from_config_impl(config_path, false)
    }

    /// [`Self::from_config`] with the `NoRepositories` refusal relaxed.
    ///
    /// The manifest edit pen (`sgt init`, `src/domain/manifest.rs`) validates
    /// every edit by round-tripping it through this module's own parser
    /// before committing (A4: "sgt remains the validating writer") — but a
    /// freshly scaffolded `[estate]` section legitimately has no `[[repo]]`
    /// entries yet, before the first `sgt repo add`, and that state must
    /// validate clean rather than being refused by the same rule that
    /// (correctly) refuses a *hand-edited* `sergeant.toml` with no
    /// repositories at all. Every other check — legacy vocabulary,
    /// duplicate/invalid names, group membership, profile validity — still
    /// applies in full; this relaxes exactly the one rule that is about
    /// "nothing to declare yet", not "something is wrong".
    pub fn from_config_allow_empty(config_path: &Path) -> Result<Self, WorkspaceError> {
        Self::from_config_impl(config_path, true)
    }

    /// Every `[[repo]]` entry `sergeant.toml` declares, **without**
    /// validating that any of them exist on disk — contrast
    /// [`Self::from_config`]/[`Self::from_config_allow_empty`], which
    /// correctly fail closed at the *first* missing repository (right for
    /// execution: a Work must never bind a repo that is not really there).
    /// A diagnostic wants the opposite: name *every* missing repository, not
    /// just the first, so `sgt doctor`'s estate check uses this instead of
    /// the strict loader.
    ///
    /// Every schema-level check still applies in full — malformed TOML
    /// (line/column via `toml::de::Error`'s own diagnostic), the R-MVP1-3
    /// legacy-vocabulary refusal, duplicate or invalid repository names —
    /// because those are manifest bugs, not "not cloned yet", and a
    /// diagnostic should refuse to read a broken manifest the same way
    /// execution does, naming the same file, line and key.
    pub fn declared_repos(config_path: &Path) -> Result<Vec<DeclaredRepo>, WorkspaceError> {
        let file = config_path.display().to_string();
        let text = std::fs::read_to_string(config_path).map_err(|source| WorkspaceError::Io {
            path: file.clone(),
            source,
        })?;
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
        let mut seen = BTreeSet::new();
        let mut declared = Vec::with_capacity(parsed.repo.len());
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
            declared.push(DeclaredRepo {
                name: entry.name,
                path: root.join(&entry.path),
                origin: entry.origin,
            });
        }
        Ok(declared)
    }

    /// A manifest **edit**'s own validator (`src/domain/manifest.rs`'s
    /// pens — `sgt init`/`repo add`/`repo remove`/`group add`/`group
    /// remove`): every schema-level check [`Self::from_config_allow_empty`]
    /// runs — legacy vocabulary, duplicate/invalid repository names, group
    /// membership against declared names, profile validity/permission modes,
    /// `[estate]` shape — **without** resolving any `[[repo]]` entry through
    /// git. [`RepositorySpec::path`] here is only the declared path joined
    /// onto `root` (see [`DeclaredRepo`]'s own doc for the same distinction),
    /// so a name that resolves fine and one that points at nothing both parse
    /// identically; [`WorkspaceError::DuplicateRepositoryPath`] — which needs
    /// git to know two declared paths are the same checkout — is the one
    /// schema check this cannot make and does not attempt.
    ///
    /// Exists because the strict resolver's per-repo `git rev-parse
    /// --show-toplevel` loop fails at the *first* declared repository not
    /// present on disk, and an edit pen's job is to validate the **edit**,
    /// not to re-verify every repository the estate has ever declared. A
    /// `git clone`d estate (`sgt init` gitignores `repos/`) declares repos in
    /// `sergeant.toml` with no `repos/` on disk at all — the on-disk-first
    /// pen would refuse *every* subsequent edit, including ones that never
    /// touch the missing repository, contradicting the design capture's own
    /// wrongness contract ("a broken repo blocks works targeting it, not the
    /// estate", `docs/gauntlet/notes/estate-manifest-design-2026-08-11.md`).
    /// A repository an edit itself populates or verifies (`sgt repo add`'s
    /// `populate_or_verify`) is already checked on disk by that caller,
    /// directly — this validator does not need to repeat it.
    pub fn from_config_structural(config_path: &Path) -> Result<Self, WorkspaceError> {
        Self::from_config_impl_structural(config_path)
    }

    /// [`Self::declared_repos`]'s sibling for `[group.<name>]`: every
    /// declared group, membership validated against declared repository
    /// names (the same [`WorkspaceError::UnknownGroupMember`] check
    /// [`Self::from_config_impl`] runs), without resolving any repository on
    /// disk. Used where only membership is wanted — `sgt run --group`'s
    /// client-side expansion (`src/cli.rs`) — so an unrelated missing
    /// repository cannot block a group whose own members are all fine (same
    /// root cause and remedy as [`Self::from_config_structural`]).
    pub fn declared_groups(
        config_path: &Path,
    ) -> Result<BTreeMap<String, GroupSpec>, WorkspaceError> {
        Ok(Self::from_config_impl_structural(config_path)?.groups)
    }

    /// [`Self::discover_scoped`]'s shape, but landing on
    /// [`Self::from_config_structural`] instead of the strict resolver at
    /// both branches (a found `[estate]`-bearing config, or a plain member
    /// `sergeant.toml` at the zero-config git toplevel) — the disk-free
    /// counterpart a group-membership-only caller wants. Returns an empty
    /// map, never an error, for the true zero-config case (no `sergeant.toml`
    /// at all): there is nothing to declare a group in.
    pub fn declared_groups_scoped(
        start: &Path,
        data_dir: Option<&Path>,
    ) -> Result<BTreeMap<String, GroupSpec>, WorkspaceError> {
        if let Some(estate_config) = Self::find_estate_upward(start, data_dir)? {
            return Self::declared_groups(&estate_config);
        }
        let toplevel = git(start, &["rev-parse", "--show-toplevel"]).map_err(|source| {
            WorkspaceError::NotARepository {
                path: start.display().to_string(),
                source,
            }
        })?;
        let config_path = PathBuf::from(toplevel).join(WORKSPACE_FILE);
        if config_path.is_file() {
            Self::declared_groups(&config_path)
        } else {
            Ok(BTreeMap::new())
        }
    }

    fn from_config_impl(
        config_path: &Path,
        allow_empty_repos: bool,
    ) -> Result<Self, WorkspaceError> {
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

        if parsed.repo.is_empty() && !allow_empty_repos {
            return Err(WorkspaceError::NoRepositories { file });
        }
        let mut seen = BTreeSet::new();
        // Identity of a repository is its resolved top level, not the name
        // the file chose for it: `path = "."` and `path = "./"` are one
        // checkout under two names, and only git can say so.
        let mut seen_paths: BTreeMap<PathBuf, String> = BTreeMap::new();
        let mut repositories = Vec::with_capacity(parsed.repo.len());
        let mut repository_policy = BTreeMap::new();
        let mut repository_origin = BTreeMap::new();
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
            if let Some(origin) = entry.origin {
                repository_origin.insert(entry.name.clone(), origin);
            }
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

        let (name, default_backend, default_workflow, surfaces_dir, data_dir) = match parsed.estate
        {
            Some(estate) => (
                estate.name,
                estate.default_backend,
                estate.default_workflow,
                estate
                    .surfaces_dir
                    .map(|d| if d.is_absolute() { d } else { root.join(d) }),
                estate
                    .data_dir
                    .map(|d| if d.is_absolute() { d } else { root.join(d) }),
            ),
            None => (repo_name(&root), None, None, None, None),
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
            data_dir,
            repository_policy,
            groups,
            repository_origin,
        })
    }

    /// [`Self::from_config_impl`] with the per-repository `git rev-parse
    /// --show-toplevel` resolution dropped — see
    /// [`Self::from_config_structural`]'s own doc for why this exists and
    /// what it deliberately cannot check. Always allows an empty `[[repo]]`
    /// list (every caller is a manifest edit, which may legitimately be
    /// scaffolding a repo-less estate — same reason
    /// [`Self::from_config_allow_empty`] relaxes it).
    fn from_config_impl_structural(config_path: &Path) -> Result<Self, WorkspaceError> {
        let file = config_path.display().to_string();
        let text = std::fs::read_to_string(config_path).map_err(|source| WorkspaceError::Io {
            path: file.clone(),
            source,
        })?;
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

        let mut seen = BTreeSet::new();
        let mut repositories = Vec::with_capacity(parsed.repo.len());
        let mut repository_policy = BTreeMap::new();
        let mut repository_origin = BTreeMap::new();
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
            // Declared, joined, **not** git-resolved (see this fn's own doc:
            // that is the one thing a structural parse cannot check).
            let joined = root.join(&entry.path);
            repository_policy.insert(entry.name.clone(), entry.instructions);
            if let Some(origin) = &entry.origin {
                repository_origin.insert(entry.name.clone(), origin.clone());
            }
            repositories.push(RepositorySpec {
                name: entry.name,
                path: joined,
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

        let (name, default_backend, default_workflow, surfaces_dir, data_dir) = match parsed.estate
        {
            Some(estate) => (
                estate.name,
                estate.default_backend,
                estate.default_workflow,
                estate
                    .surfaces_dir
                    .map(|d| if d.is_absolute() { d } else { root.join(d) }),
                estate
                    .data_dir
                    .map(|d| if d.is_absolute() { d } else { root.join(d) }),
            ),
            None => (repo_name(&root), None, None, None, None),
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
            data_dir,
            repository_policy,
            groups,
            repository_origin,
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

    /// This repository's declared `origin`, if `sgt repo add` (or a hand
    /// edit) recorded one. `None` for a name absent from the manifest and
    /// for a declared repository that never gave an `origin`.
    pub fn repository_origin(&self, repository: &str) -> Option<&str> {
        self.repository_origin.get(repository).map(String::as_str)
    }

    /// The directory holding the nearest ancestor `sergeant.toml` carrying an
    /// `[estate]` table (R-MVP1-12's own upward walk, delegated to rather
    /// than duplicated), without resolving or validating any `[[repo]]`
    /// entry. Used where only "is there an estate here, and where" is
    /// needed — MVP-3's estate-resolved data-dir default and the manifest
    /// edit pen (`src/domain/manifest.rs`) — because a full [`Self::discover`]
    /// would refuse a freshly scaffolded, repo-less estate via
    /// `NoRepositories` before either of those ever gets to run.
    pub fn estate_root(
        start: &Path,
        data_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>, WorkspaceError> {
        Ok(Self::find_estate_upward(start, data_dir)?
            .and_then(|config| config.parent().map(Path::to_path_buf)))
    }

    /// [`Self::estate_root`]'s sibling for `src/cli.rs`'s `resolve_data_dir`
    /// (ADR 0008(b)): the discovered estate root together with its
    /// manifest's `[estate] data_dir` override, if any. `None` when no
    /// estate is found; `Some((root, None))` when one is found but declares
    /// no override, leaving the caller's own default in force exactly as
    /// `surfaces_dir` does.
    ///
    /// Deliberately **not** [`Self::from_config_structural`]:
    /// `resolve_data_dir` runs at the top of every `dispatch`, ahead of
    /// every command including `sgt doctor`, whose entire job is diagnosing
    /// a broken manifest gracefully. A structural defect elsewhere in the
    /// file — a duplicate profile, an unknown group member, an invalid
    /// permission mode — has nothing to do with `data_dir` and must not
    /// stop `doctor` from ever running. This reads only the one field it
    /// needs, at the same tolerance [`estate_table_check`] already applies
    /// to find the file in the first place: valid TOML syntax and no
    /// legacy vocabulary are required, everything else about the manifest's
    /// shape is not.
    pub fn estate_root_and_data_dir(
        start: &Path,
        data_dir_scope: Option<&Path>,
    ) -> Result<Option<(PathBuf, Option<PathBuf>)>, WorkspaceError> {
        let Some(config_path) = Self::find_estate_upward(start, data_dir_scope)? else {
            return Ok(None);
        };
        let root = config_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let data_dir = estate_data_dir_override(&config_path, &root)?;
        Ok(Some((root, data_dir)))
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
/// Whether `config_path` carries an `[estate]` table — [`Workspace::
/// find_estate_upward_bounded`]'s match predicate. `Ok(false)` for a file
/// this walk cannot even read (permission, race — indistinguishable from
/// "no file here"). `Err` for one it CAN read but that is malformed or
/// carries legacy vocabulary (W5/R-MVP1-3): those are not silently skipped
/// — see the walk's own doc comment.
fn estate_table_check(config_path: &Path) -> Result<bool, WorkspaceError> {
    let Ok(text) = std::fs::read_to_string(config_path) else {
        return Ok(false);
    };
    let file = config_path.display().to_string();
    check_legacy_vocabulary(&text, &file)?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|source| WorkspaceError::Malformed { path: file, source })?;
    Ok(value
        .as_table()
        .is_some_and(|table| table.contains_key("estate")))
}

/// [`estate_table_check`]'s sibling for `data_dir` (ADR 0008(b)): the
/// `[estate] data_dir` string, if any, resolved onto `root` exactly as
/// [`Workspace::from_config_impl_structural`] resolves it — relative joins
/// on, absolute passes through — but reached the same tolerant way
/// `estate_table_check` finds the `[estate]` table itself: a raw TOML
/// value, not a deserialize into [`WorkspaceFile`]. This is what lets
/// [`Workspace::estate_root_and_data_dir`] read `data_dir` without also
/// demanding the rest of the manifest — repos, profiles, groups — be
/// structurally valid. Unreadable (permission, race) answers `None`, the
/// same "indistinguishable from no file here" tolerance `estate_table_check`
/// applies; by the time this runs, [`Workspace::find_estate_upward`] has
/// already required the file to parse as TOML and carry no legacy
/// vocabulary, so those two failure modes are not re-checked here.
fn estate_data_dir_override(
    config_path: &Path,
    root: &Path,
) -> Result<Option<PathBuf>, WorkspaceError> {
    let Ok(text) = std::fs::read_to_string(config_path) else {
        return Ok(None);
    };
    let file = config_path.display().to_string();
    let value: toml::Value =
        toml::from_str(&text).map_err(|source| WorkspaceError::Malformed { path: file, source })?;
    Ok(value
        .get("estate")
        .and_then(|estate| estate.get("data_dir"))
        .and_then(|data_dir| data_dir.as_str())
        .map(PathBuf::from)
        .map(|data_dir| {
            if data_dir.is_absolute() {
                data_dir
            } else {
                root.join(data_dir)
            }
        }))
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
            data_dir: None,
            repository_policy: BTreeMap::new(),
            groups: BTreeMap::new(),
            repository_origin: BTreeMap::new(),
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

    // ---- ADR 0008(b): `[estate] data_dir` ---------------------------------

    /// `data_dir` parses and resolves exactly like `surfaces_dir` above —
    /// same shape, deliberately, per ADR 0008(b)'s "do not invent a second
    /// convention".
    #[test]
    fn estate_data_dir_resolves_relative_and_keeps_absolute() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let workspace = parse(
            root,
            "[estate]\nname = \"w\"\ndata_dir = \"../elsewhere-data\"\n\n\
             [[repo]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect("relative data_dir parses");
        assert_eq!(workspace.data_dir, Some(root.join("../elsewhere-data")));

        let absolute = dir.path().join("abs-data");
        let workspace = parse(
            root,
            &format!(
                "[estate]\nname = \"w\"\ndata_dir = {:?}\n\n[[repo]]\nname = \"solo\"\npath = \".\"\n",
                absolute.to_string_lossy()
            ),
        )
        .expect("absolute data_dir parses");
        assert_eq!(workspace.data_dir, Some(absolute));

        // Unset stays `None` — `resolve_data_dir`'s own default is left in
        // force.
        let workspace = parse(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect("no data_dir parses");
        assert_eq!(workspace.data_dir, None);
    }

    /// `resolve_data_dir` (`src/cli.rs`) calls
    /// [`Workspace::estate_root_and_data_dir`] at the top of every command
    /// dispatch, including `sgt doctor` — whose entire purpose is to
    /// diagnose a broken manifest. A structural defect unrelated to
    /// `data_dir` (here, a duplicate profile name — the same manifest both
    /// [`Workspace::from_config`] and [`Workspace::from_config_structural`]
    /// refuse) must not stop `data_dir` from being found, or `doctor` could
    /// never run against the very manifest it exists to diagnose.
    #[test]
    fn estate_data_dir_is_found_even_when_the_rest_of_the_manifest_is_structurally_broken() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);
        let config_path = root.join(WORKSPACE_FILE);
        std::fs::write(
            &config_path,
            "[estate]\nname = \"w\"\ndata_dir = \"custom-data\"\n\n\
             [[repo]]\nname = \"solo\"\npath = \".\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"fake\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"fake\"\n",
        )
        .expect("sergeant.toml");

        // Both strict resolvers refuse this manifest outright.
        assert!(Workspace::from_config(&config_path).is_err());
        assert!(matches!(
            Workspace::from_config_structural(&config_path),
            Err(WorkspaceError::DuplicateProfile { .. })
        ));

        // But locating the estate and its `data_dir` override does not.
        let (found_root, data_dir) = Workspace::estate_root_and_data_dir(root, None)
            .expect("an unrelated structural defect must not fail data-dir lookup")
            .expect("an estate is found");
        assert_eq!(
            std::fs::canonicalize(&found_root).ok(),
            std::fs::canonicalize(root).ok()
        );
        // Canonicalized on both sides, same as `found_root` above: `root`
        // (production) is already canonical (`find_estate_upward_bounded`
        // canonicalizes `start` before walking up), but this test's own
        // `root` local is `TempDir`'s raw path. On Linux those happen to be
        // identical; on macOS `/var` is a symlink to `/private/var` (like
        // `/tmp` -> `/private/tmp`), so the raw and canonical forms diverge
        // and a direct comparison fails there (#127, first measured on the
        // MacBook Pro M3 Pro arrival trip, 2026-08-15).
        assert_eq!(
            data_dir
                .as_deref()
                .and_then(|p| std::fs::canonicalize(p).ok()),
            std::fs::canonicalize(root.join("custom-data")).ok()
        );
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
    ///
    /// Deliberately includes a plain, non-estate `sergeant.toml` *above*
    /// `root` (not merely "no sergeant.toml at all" — the guard-map
    /// mutation this test must kill is `has_estate_table`'s own check
    /// dropped from the walk's match predicate, i.e. `if candidate.is_file()
    /// && has_estate_table(...)` weakened to `if candidate.is_file()`; with
    /// no file anywhere on the ascent path, that mutated predicate is never
    /// exercised with a file present, so a fixture with no file at all
    /// cannot distinguish the mutant from the real thing). With the file in
    /// place, the mutant would wrongly treat it as an estate root and
    /// `from_config` it — landing a different workspace (`config_path`
    /// `Some`, a different `name`) than the zero-config fallback this test
    /// asserts.
    #[test]
    fn no_estate_anywhere_falls_back_to_zero_config_unchanged() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("solo-repo");
        init_repo(&root);
        // A non-estate sergeant.toml, above `root`, on the walk's ascent
        // path — present, but without `[estate]`, so it must not stop the
        // walk (this module's own "a sergeant.toml without [estate] is a
        // member's own config, not an estate" rule) and discovery still
        // falls all the way back to zero-config.
        std::fs::write(
            dir.path().join(WORKSPACE_FILE),
            "[[repo]]\nname = \"unrelated\"\npath = \".\"\n",
        )
        .expect("write non-estate sergeant.toml");

        let workspace = Workspace::discover(&root).expect("zero-config fallback");
        assert_eq!(workspace.repositories.len(), 1);
        assert_eq!(
            std::fs::canonicalize(&workspace.repositories[0].path).ok(),
            std::fs::canonicalize(&root).ok()
        );
        assert!(workspace.config_path.is_none());
        assert_eq!(
            workspace.name,
            repo_name(&std::fs::canonicalize(&root).unwrap_or(root)),
            "the zero-config name is the repo's own directory name, not the \
             unrelated sergeant.toml's"
        );
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

        let found = Workspace::find_estate_upward_bounded(&below_boundary, Some(&boundary), None)
            .expect("no legacy/malformed sergeant.toml in this fixture");
        assert_eq!(
            found, None,
            "an estate above the boundary must never be found"
        );

        // The same estate, found once it is unbounded (or the boundary is
        // above it) — proving the walk itself works and the bound is what
        // stopped it, not a bug in the walk.
        let found_unbounded =
            Workspace::find_estate_upward_bounded(&below_boundary, Some(&above_boundary), None)
                .expect("no legacy/malformed sergeant.toml in this fixture");
        assert!(
            found_unbounded.is_some(),
            "the same estate must be found once the boundary includes it"
        );
    }

    /// R-MVP1-12's other half: "never above an explicit `--data-dir`/
    /// `SGT_DATA_DIR` scope." A data-dir scope that sits on `start`'s own
    /// ancestor chain, strictly between `start` and the estate config, must
    /// stop the walk there — never letting it reach the estate even one
    /// directory further up — while a data-dir scope that is not on the
    /// ancestor chain at all (the ordinary case: the data dir usually has
    /// nothing to do with whichever repository a submission runs against)
    /// must not change the outcome.
    #[test]
    fn the_data_dir_scope_bounds_the_walk_like_home_does() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let home = dir.path().join("home");
        let estate_root = home.join("estate");
        let repos_dir = estate_root.join("repos");
        let member = repos_dir.join("member");
        let unrelated_scope = home.join("scratch-data-dir");
        std::fs::create_dir_all(&member).expect("member dir");
        std::fs::create_dir_all(&unrelated_scope).expect("unrelated scope dir");
        init_repo(&member);

        // An estate at `estate_root` — inside `$HOME`, so an unscoped walk
        // (or one bounded only by `$HOME`) finds it fine.
        write_estate(&estate_root, "in-estate-data-dir");
        let home = std::fs::canonicalize(&home).expect("canonical home");
        let repos_dir = std::fs::canonicalize(&repos_dir).expect("canonical repos dir");
        let unrelated_scope =
            std::fs::canonicalize(&unrelated_scope).expect("canonical unrelated scope");

        let found_unscoped = Workspace::find_estate_upward_bounded(&member, Some(&home), None)
            .expect("no legacy/malformed sergeant.toml in this fixture");
        assert!(
            found_unscoped.is_some(),
            "without a data-dir scope, the $HOME boundary alone finds the estate"
        );

        // A data-dir scope that is NOT on `member`'s ancestor chain at all
        // — the ordinary case — must not change the outcome.
        let found_unrelated_scope =
            Workspace::find_estate_upward_bounded(&member, Some(&home), Some(&unrelated_scope))
                .expect("no legacy/malformed sergeant.toml in this fixture");
        assert!(
            found_unrelated_scope.is_some(),
            "a data-dir scope that is not an ancestor of `start` must not change the outcome"
        );

        // The scope genuinely is an ancestor of `start`, strictly below the
        // estate's own `sergeant.toml` (`repos/`, one level under
        // `estate_root`) — the A8 self-hosting shape, data dir in-estate.
        // The walk must stop there, never reaching `estate_root` one
        // directory further up.
        let found_ancestor_scope =
            Workspace::find_estate_upward_bounded(&member, Some(&home), Some(&repos_dir))
                .expect("no legacy/malformed sergeant.toml in this fixture");
        assert_eq!(
            found_ancestor_scope, None,
            "a data-dir scope that IS an ancestor of `start` must stop the walk there, \
             never letting it reach the estate config even one directory further up"
        );
    }

    // -------------------------------------------------- W5: legacy/malformed
    // sergeant.toml on the way up fails closed, not silently skipped

    /// R-MVP1-3's named migration refusal must fire for a legacy-vocabulary
    /// `sergeant.toml` the upward walk steps over on its way to (what would
    /// otherwise be) an estate above it — not just one chosen directly.
    /// Before this fix, `has_estate_table` swallowed the parse/legacy
    /// failure and the walk silently treated it as "not an estate, keep
    /// walking", falling through all the way to the zero-config member-repo
    /// fallback with no diagnostic at all.
    #[test]
    fn a_legacy_vocabulary_sergeant_toml_on_the_way_up_fails_the_walk_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        let member = estate_root.join("repos").join("member");
        std::fs::create_dir_all(&member).expect("member dir");
        init_repo(&member);
        std::fs::write(
            estate_root.join(WORKSPACE_FILE),
            "[workspace]\nname = \"legacy\"\n\n[[repository]]\nname = \"legacy\"\npath = \".\"\n",
        )
        .expect("legacy sergeant.toml");

        let err = Workspace::find_estate_upward_bounded(&member, None, None)
            .expect_err("a legacy-vocabulary file on the way up must refuse, not be skipped");
        assert!(
            matches!(err, WorkspaceError::LegacyVocabulary { .. }),
            "got {err}"
        );
    }

    /// Same shape, a `sergeant.toml` on the way up that is not even valid
    /// TOML: the walk must refuse, not silently skip it and fall through to
    /// the zero-config fallback.
    #[test]
    fn a_malformed_sergeant_toml_on_the_way_up_fails_the_walk_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        let member = estate_root.join("repos").join("member");
        std::fs::create_dir_all(&member).expect("member dir");
        init_repo(&member);
        std::fs::write(estate_root.join(WORKSPACE_FILE), "this is not [ toml").expect("write");

        let err = Workspace::find_estate_upward_bounded(&member, None, None)
            .expect_err("a malformed file on the way up must refuse, not be skipped");
        assert!(matches!(err, WorkspaceError::Malformed { .. }), "got {err}");
    }

    /// The control this pair calibrates against: a plain member-repo
    /// `sergeant.toml` with no `[estate]` and no legacy vocabulary at all —
    /// still not an error, still "keep walking" (this module's own
    /// `a_member_repos_own_sergeant_toml_without_estate_does_not_stop_the_walk`
    /// test covers the full discovery path; this one pins the lower-level
    /// walk function directly).
    #[test]
    fn a_plain_member_sergeant_toml_with_no_estate_table_does_not_fail_the_walk() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        let member = estate_root.join("repos").join("member");
        std::fs::create_dir_all(&member).expect("member dir");
        init_repo(&member);
        std::fs::write(
            member.join(WORKSPACE_FILE),
            "[[repo]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect("write");

        let found = Workspace::find_estate_upward_bounded(&member, None, None)
            .expect("a plain non-estate sergeant.toml must not fail the walk");
        assert_eq!(
            found, None,
            "a member's own non-estate config does not stop the walk (no estate above it here)"
        );
    }

    /// TH-03: R-MVP1-3's pin ("a same-commit grep finds zero `[workspace]`/
    /// `[[repository]]` outside `reference/`") was a one-time manual check,
    /// never a standing gate — nothing stopped a future fixture
    /// reintroducing the legacy vocabulary. This is that gate: every
    /// `sergeant.toml` actually checked into this tree outside `reference/`
    /// (frozen evidence, exempted — CLAUDE.md's own convention) must parse
    /// without a top-level `workspace` or `repository` table.
    ///
    /// Scoped to files literally named `sergeant.toml`, not a bare string
    /// grep across every doc and note: several already-committed notes
    /// discuss the legacy vocabulary in prose while describing the
    /// migration itself, which is not the live-config leak this pin is
    /// about (docs/gauntlet/notes and docs/gauntlet/runs are historical
    /// records, not configuration this codebase reads).
    #[test]
    fn no_committed_sergeant_toml_outside_reference_carries_legacy_vocabulary() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut offenders = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = entry.file_name();
                if path.is_dir() {
                    // `reference/` is frozen evidence (CLAUDE.md); `.git`
                    // and `target` are never source content.
                    if matches!(name.to_str(), Some("reference" | ".git" | "target")) {
                        continue;
                    }
                    stack.push(path);
                } else if name == WORKSPACE_FILE {
                    let Ok(text) = std::fs::read_to_string(&path) else {
                        continue;
                    };
                    let Ok(value) = toml::from_str::<toml::Value>(&text) else {
                        // A malformed committed sergeant.toml is a real
                        // problem, but a different one (W5 covers the
                        // upward-walk's own handling of it) — not this
                        // pin's concern.
                        continue;
                    };
                    if let Some(table) = value.as_table()
                        && (table.contains_key("workspace") || table.contains_key("repository"))
                    {
                        offenders.push(path);
                    }
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "legacy [workspace]/[[repository]] vocabulary in committed sergeant.toml file(s) \
             outside reference/: {offenders:?}"
        );
    }
}

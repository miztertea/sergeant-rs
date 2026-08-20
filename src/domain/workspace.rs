//! Workspace: the repository surface work originates from (proposal §9).
//!
//! **Exact-root admission (estate-root proposal §4.1, Phase D).** An estate is
//! exactly the directory that itself contains a `sergeant.toml` which parses,
//! declares `[estate]`, and satisfies the manifest schema. There is no
//! ancestor walk and no zero-configuration Git fallback: `Workspace::admit`
//! answers one deterministic question about one directory, and
//! [`EstateRootError`] carries §4.4's loud corrective diagnostic when the
//! answer is no. R-MVP1-12 (upward estate discovery across Git boundaries)
//! and the single-repository zero-config workspace are both **superseded** —
//! a one-repository installation is an estate with one declared repository.
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

/// An admitted estate root (§4.1): the canonical directory that *is* the
/// estate, and the manifest that made it one. Produced only by
/// [`Workspace::admit`], so holding one is proof the exact-root check has
/// already passed — the type every later step (data-dir resolution,
/// descriptor lookup, spawn, API call, harness exec) takes as its
/// precondition, per §4.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstateRoot {
    /// Canonical estate root directory.
    pub path: PathBuf,
    /// `<path>/sergeant.toml`.
    pub manifest_path: PathBuf,
}

/// Where the directory being admitted came from — only ever a wording
/// difference in [`EstateRootError`]'s remedy, never a difference in the
/// check itself (C10: `-C` *names* an exact root, it does not search from
/// one, and it earns no leniency for doing so).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RootSource {
    /// The process's own working directory.
    #[default]
    Cwd,
    /// An explicit `sgt -C <path>` (C10).
    Flag,
}

impl RootSource {
    /// How the diagnostic names the directory.
    fn subject(self) -> &'static str {
        match self {
            Self::Cwd => "the current directory",
            Self::Flag => "the directory named by -C",
        }
    }
}

/// §4.4's loud corrective diagnostics: an estate-scoped command refused
/// before it touched a data dir, a descriptor, a daemon, or a repository.
///
/// Every variant's `Display` is the full multi-line block §4.4 specifies —
/// what was expected, why no parent was searched, and the concrete remedy —
/// because the whole value of exact-root admission is that the refusal
/// teaches the operator where they actually are.
#[derive(Debug, thiserror::Error)]
pub enum EstateRootError {
    /// No `sergeant.toml` in the exact directory (§4.4, first block).
    NoEstate {
        /// The directory that was checked, canonical.
        root: PathBuf,
        /// The path that would have made it an estate.
        expected: PathBuf,
        /// How that directory was chosen.
        via: RootSource,
    },
    /// A valid estate root is bound in this environment (`SGT_ESTATE_ROOT`)
    /// and sits strictly above the directory being addressed — §4.4's
    /// second block, which names both roots.
    Descendant {
        /// The directory that was checked, canonical.
        root: PathBuf,
        /// The bound estate root above it, canonical.
        bound_root: PathBuf,
        /// How that directory was chosen.
        via: RootSource,
    },
    /// A `sergeant.toml` is there, but declares no `[estate]` table — a
    /// member repository's own config is not an estate root.
    NotAnEstate {
        /// The directory that was checked, canonical.
        root: PathBuf,
        /// The file that was read.
        manifest_path: PathBuf,
        /// How that directory was chosen.
        via: RootSource,
    },
    /// The manifest exists but is invalid. §4.4's last rule: surface the
    /// exact parser/schema diagnostic and **never** fall through to another
    /// estate.
    Invalid {
        /// The file that was read.
        manifest_path: PathBuf,
        /// The exact diagnostic, line and key included.
        source: Box<WorkspaceError>,
    },
}

impl std::fmt::Display for EstateRootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoEstate { expected, via, .. } => write!(
                f,
                "no estate found in {subject}\n\
                 \n\
                 Expected:\n  \
                 {expected}\n\
                 \n\
                 Sergeant does not search parent directories for an estate. This prevents a\n\
                 Captain session or Work from silently attaching to the wrong environment.\n\
                 \n\
                 Are you in the intended estate root?\n  \
                 {remedy}\n\
                 \n\
                 If this directory should become a new estate:\n  \
                 sgt init",
                subject = via.subject(),
                expected = expected.display(),
                remedy = match via {
                    RootSource::Cwd => "cd <estate-root>",
                    RootSource::Flag => "sgt -C <estate-root> <command>",
                },
            ),
            Self::Descendant {
                root,
                bound_root,
                via,
            } => write!(
                f,
                "this command must be run from the estate root\n\
                 \n\
                 {label}:\n  \
                 {root}\n\
                 \n\
                 Bound estate root:\n  \
                 {bound_root}\n\
                 \n\
                 Return to the root and retry:\n  \
                 {remedy}",
                label = match via {
                    RootSource::Cwd => "Current directory",
                    RootSource::Flag => "Directory named by -C",
                },
                root = root.display(),
                bound_root = bound_root.display(),
                remedy = match via {
                    RootSource::Cwd => format!("cd {}", bound_root.display()),
                    RootSource::Flag => format!("sgt -C {} <command>", bound_root.display()),
                },
            ),
            Self::NotAnEstate {
                manifest_path, via, ..
            } => write!(
                f,
                "no estate found in {subject}\n\
                 \n\
                 Read:\n  \
                 {manifest_path}\n\
                 \n\
                 That file declares no [estate] table, so it is a repository's own config, not\n\
                 an estate root. Sergeant does not search parent directories for one.\n\
                 \n\
                 Are you in the intended estate root?\n  \
                 {remedy}\n\
                 \n\
                 If this directory should become a new estate:\n  \
                 sgt init",
                subject = via.subject(),
                manifest_path = manifest_path.display(),
                remedy = match via {
                    RootSource::Cwd => "cd <estate-root>",
                    RootSource::Flag => "sgt -C <estate-root> <command>",
                },
            ),
            Self::Invalid {
                manifest_path,
                source,
            } => write!(
                f,
                "the estate manifest is invalid\n\
                 \n\
                 Read:\n  \
                 {manifest_path}\n\
                 \n\
                 {source}\n\
                 \n\
                 Sergeant does not search parent directories for another estate. Fix the file\n\
                 above and retry.",
                manifest_path = manifest_path.display(),
            ),
        }
    }
}

impl EstateRootError {
    /// Re-word this refusal for a root named by `sgt -C` rather than the
    /// process's cwd (C10). The *check* is identical either way; only the
    /// remedy line changes, so an agent that addressed the wrong path with
    /// `-C` is told to fix `-C`, not to `cd`.
    pub fn via_flag(self) -> Self {
        match self {
            Self::NoEstate { root, expected, .. } => Self::NoEstate {
                root,
                expected,
                via: RootSource::Flag,
            },
            Self::Descendant {
                root, bound_root, ..
            } => Self::Descendant {
                root,
                bound_root,
                via: RootSource::Flag,
            },
            Self::NotAnEstate {
                root,
                manifest_path,
                ..
            } => Self::NotAnEstate {
                root,
                manifest_path,
                via: RootSource::Flag,
            },
            invalid @ Self::Invalid { .. } => invalid,
        }
    }

    /// §4.4's second block: a "there is no estate here" refusal becomes "you
    /// are inside one, one level down" when `bound_root` is a *valid* estate
    /// root strictly above the directory that was checked. Only the
    /// nothing-here variants are upgraded — a manifest that exists and is
    /// broken keeps its own exact diagnostic, which §4.4 requires never be
    /// traded for a pointer at some other estate.
    pub fn with_bound_root(self, bound_root: PathBuf) -> Self {
        match self {
            Self::NoEstate { root, via, .. } | Self::NotAnEstate { root, via, .. }
                if root.starts_with(&bound_root) && root != bound_root =>
            {
                Self::Descendant {
                    root,
                    bound_root,
                    via,
                }
            }
            other => other,
        }
    }
}

/// Failure resolving a workspace.
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
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
/// `estate` is `Option` at the *parser* level, not at the admission level: a
/// `sergeant.toml` with no `[estate]` table is a member repository's own
/// config, and [`Workspace::admit`] refuses it by name
/// ([`EstateRootError::NotAnEstate`]) rather than mistaking it for an estate
/// root. The field stays optional here because `src/domain/manifest.rs`'s
/// edit pen legitimately parses a manifest mid-scaffold, before `[estate]`
/// has been written.
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
    /// §4.1's one deterministic check, run against exactly `dir` and nothing
    /// else: is this directory an estate root?
    ///
    /// `dir/sergeant.toml` must exist, parse, declare `[estate]`, and satisfy
    /// the manifest schema. **No parent is examined and Git is never
    /// consulted** — that is the whole point of the rule (§4.1: "Sergeant
    /// does not search parents and does not use Git to infer an estate"),
    /// and it is what makes a directory mistake incapable of attaching a
    /// session to the wrong environment.
    ///
    /// Schema validation here is [`Self::from_config_structural`]'s: every
    /// check the strict loader makes *except* resolving each declared
    /// repository through git. A declared repository that is not on disk yet
    /// is a repository problem, not an estate-identity problem — the design
    /// capture's own wrongness contract ("a broken repo blocks works
    /// targeting it, not the estate") — so it must not make the estate
    /// itself inadmissible, block `sgt repo add`, or refuse a daemon start.
    /// [`Self::resolve`] is the strict half, for callers that are about to
    /// bind a Work.
    pub fn admit(dir: &Path) -> Result<EstateRoot, EstateRootError> {
        let root = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        let manifest_path = root.join(WORKSPACE_FILE);
        if !manifest_path.is_file() {
            return Err(EstateRootError::NoEstate {
                root,
                expected: manifest_path,
                via: RootSource::Cwd,
            });
        }
        match estate_table_check(&manifest_path) {
            Ok(true) => {}
            Ok(false) => {
                return Err(EstateRootError::NotAnEstate {
                    root,
                    manifest_path,
                    via: RootSource::Cwd,
                });
            }
            Err(e) => {
                return Err(EstateRootError::Invalid {
                    manifest_path,
                    source: Box::new(e),
                });
            }
        }
        if let Err(e) = Self::from_config_structural(&manifest_path) {
            return Err(EstateRootError::Invalid {
                manifest_path,
                source: Box::new(e),
            });
        }
        Ok(EstateRoot {
            path: root,
            manifest_path,
        })
    }

    /// [`Self::admit`], then the full strict load: every declared repository
    /// resolved on disk against its derived `repos/<name>` mount (§6.1).
    /// This is what the engine plans against — a Work must never bind a
    /// repository that is not really there.
    pub fn resolve(dir: &Path) -> Result<Self, EstateRootError> {
        let admitted = Self::admit(dir)?;
        Self::from_config(&admitted.manifest_path).map_err(|source| EstateRootError::Invalid {
            manifest_path: admitted.manifest_path,
            source: Box::new(source),
        })
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

    /// Whether `dir` is *itself* an estate root, tolerantly: `Ok(true)` iff
    /// `dir/sergeant.toml` exists, parses as TOML, carries no legacy
    /// vocabulary, and declares `[estate]`. **No parent is examined.**
    ///
    /// Deliberately **not** [`Self::admit`]: `src/cli.rs`'s
    /// `resolve_data_dir` runs ahead of every command including `sgt
    /// doctor`, whose entire job is diagnosing a broken manifest gracefully.
    /// A structural defect elsewhere in the file — a duplicate profile, an
    /// unknown group member, an invalid permission mode — has nothing to do
    /// with `data_dir` and must not stop `doctor` from ever running. This
    /// answers only the question it needs, at [`estate_table_check`]'s own
    /// tolerance.
    pub fn is_estate_root(dir: &Path) -> Result<bool, WorkspaceError> {
        let manifest_path = dir.join(WORKSPACE_FILE);
        if !manifest_path.is_file() {
            return Ok(false);
        }
        estate_table_check(&manifest_path)
    }

    /// `src/cli.rs`'s `resolve_data_dir` (ADR 0008(b)): `dir`'s own
    /// `[estate] data_dir` override, if `dir` is an estate root and declares
    /// one. `Ok(None)` when `dir` is not an estate root at all, or is one
    /// that declares no override — leaving the caller's own default in force
    /// exactly as `surfaces_dir` does. Same tolerance as
    /// [`Self::is_estate_root`], and for the same reason.
    pub fn root_data_dir_override(dir: &Path) -> Result<Option<PathBuf>, WorkspaceError> {
        if !Self::is_estate_root(dir)? {
            return Ok(None);
        }
        estate_data_dir_override(&dir.join(WORKSPACE_FILE), dir)
    }

    /// Restrict the workspace to the named repositories (the submit request's
    /// resolved scope — see `runtime::engine::Engine::resolve_scope`, the
    /// caller that turns `--repo`/`--group`/`--all` into this exact name
    /// list). An unknown name is an error rather than a silently empty
    /// surface, and a name repeated in the selection is an error too — two
    /// identical bindings would send `materialize` at the same worktree path
    /// and branch twice, the second `git worktree add` failing after the
    /// first has already touched the user's repository.
    ///
    /// **An empty `names` is refused, not "every repository" (estate-root
    /// Phase C, §7.1).** Before Phase C this returned every declared
    /// repository — the "zero-config" reading of no selection. §7.1 replaces
    /// that with an explicit refusal: a one-repository estate's sole-repo
    /// inference and a multi-repository estate's structured remedy both
    /// belong to the *resolution* layer above this one
    /// (`Engine::resolve_scope`), which decides what an empty scope request
    /// means before ever reaching here — this domain-level call now only
    /// ever sees a nonempty, already-decided name list.
    pub fn select(&self, names: &[String]) -> Result<Vec<RepositorySpec>, String> {
        if names.is_empty() {
            return Err(format!(
                "no repositories selected for workspace {:?}; an empty selection is refused, \
                 not expanded to every declared repository (estate-root Phase C, §7.1) — \
                 select explicitly with --repo, --group, or --all",
                self.name
            ));
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

/// Whether `config_path` carries an `[estate]` table — [`Workspace::admit`]'s
/// own predicate. `Ok(false)` for a file that cannot even be read
/// (permission, race — indistinguishable from "no file here"). `Err` for one
/// that CAN be read but is malformed or carries legacy vocabulary
/// (W5/R-MVP1-3): §4.4's last rule is that an invalid manifest surfaces its
/// exact diagnostic and never falls through.
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

    /// `select` with an empty name list is refused, not "every declared
    /// repository" — the Phase 0 pin's flip (estate-root Phase C, §7.1).
    /// Single-repo inference and the multi-repo structured remedy are
    /// `Engine::resolve_scope`'s job, decided before this is ever called
    /// with an empty list; this domain-layer regression test only pins that
    /// `select` itself no longer papers over an undecided scope.
    #[test]
    fn empty_selection_is_refused_at_the_domain_layer() {
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

        let err = workspace
            .select(&[])
            .expect_err("an empty selection must now be refused, not expanded to \"all\"");
        assert!(
            err.contains("payments") && err.contains("--all"),
            "the refusal should name the workspace and the remedy vocabulary, got: {err}"
        );
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

    // ---- §4.1: exact-root admission (R-MVP1-12 superseded) ----------------

    /// A `sergeant.toml` under `root` with an `[estate]` table, declaring one
    /// repository at the derived `repos/<name>` mount (§6.1), in `root`'s own
    /// git repository.
    fn write_estate(root: &Path, name: &str) {
        init_repo(root);
        init_repo(&root.join("repos").join("solo"));
        std::fs::write(
            root.join(WORKSPACE_FILE),
            format!(
                "[estate]\nname = {name:?}\n\n[[repo]]\nname = \"solo\"\npath = \"repos/solo\"\n"
            ),
        )
        .expect("write estate sergeant.toml");
    }

    /// §4.1's happy path: the directory that itself carries an
    /// `[estate]`-bearing `sergeant.toml` is admitted, and the admission
    /// names the canonical root and the manifest that made it one.
    #[test]
    fn admit_accepts_the_exact_directory_that_carries_the_manifest() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        std::fs::create_dir_all(&estate_root).expect("estate dir");
        write_estate(&estate_root, "payments");

        let admitted = Workspace::admit(&estate_root).expect("the exact root is admitted");
        assert_eq!(
            Some(admitted.path.clone()),
            std::fs::canonicalize(&estate_root).ok()
        );
        assert_eq!(admitted.manifest_path, admitted.path.join(WORKSPACE_FILE));

        let estate = Workspace::resolve(&estate_root).expect("the strict load agrees");
        assert_eq!(estate.name, "payments");
    }

    /// **Phase 0 pin #1, flipped (C7a).** The ancestor walk is gone: a
    /// descendant cwd — the `repos/<name>` mount that used to resolve to the
    /// estate above it — is now refused by name, and the refusal is §4.4's,
    /// not a generic "not found". Nothing above the directory is examined.
    // CONTRACT PIN (estate-root Phase D): ancestor-walk discovery is removed; a descendant cwd no longer finds an estate above it.
    #[test]
    fn a_descendant_cwd_is_refused_and_never_finds_the_estate_above_it() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        std::fs::create_dir_all(&estate_root).expect("estate dir");
        write_estate(&estate_root, "ancestor-estate");

        let member = estate_root.join("repos").join("solo");
        let err = Workspace::admit(&member)
            .expect_err("exact-root admission must refuse a descendant of the estate");
        assert!(
            matches!(err, EstateRootError::NoEstate { .. }),
            "got {err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("no estate found in the current directory"),
            "§4.4's own first line: {message}"
        );
        assert!(
            message.contains("does not search parent directories"),
            "the refusal must say why no ancestor was consulted: {message}"
        );
        assert!(
            message.contains(&member.join(WORKSPACE_FILE).display().to_string())
                || message.contains(
                    &std::fs::canonicalize(&member)
                        .unwrap_or_else(|_| member.clone())
                        .join(WORKSPACE_FILE)
                        .display()
                        .to_string()
                ),
            "the refusal must name the exact path it expected: {message}"
        );
        assert!(
            message.contains("sgt init"),
            "the refusal must name the init remedy: {message}"
        );
    }

    /// **Phase 0 pin #2, flipped (C7a).** The zero-config Git fallback is
    /// gone: a plain git repository with no `sergeant.toml` anywhere is no
    /// longer a workspace, it is "no estate here" — exact-root resolution has
    /// nothing to fall back *to*.
    // CONTRACT PIN (estate-root Phase D): zero-config git fallback is removed; a repo with no sergeant.toml anywhere above no longer resolves to a workspace.
    #[test]
    fn a_plain_git_repository_with_no_manifest_is_no_longer_a_workspace() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("plain-repo");
        init_repo(&root);
        // No `sergeant.toml` anywhere in this fixture, at `root` or above it.

        let err = Workspace::admit(&root).expect_err("there is no estate here");
        assert!(
            matches!(err, EstateRootError::NoEstate { .. }),
            "a git repository is not an estate: {err:?}"
        );
        assert!(
            Workspace::resolve(&root).is_err(),
            "the strict resolver must refuse too — no git fallback survives anywhere"
        );
    }

    /// A `sergeant.toml` that is there but declares no `[estate]` table is a
    /// member repository's own config. It is refused **by name** rather than
    /// silently skipped (there is nothing to skip *to* any more), and the
    /// remedy names `sgt init`.
    #[test]
    fn a_manifest_without_an_estate_table_is_refused_by_name() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("member");
        init_repo(&root);
        std::fs::write(
            root.join(WORKSPACE_FILE),
            "[[repo]]\nname = \"solo\"\npath = \".\"\n",
        )
        .expect("write");

        let err = Workspace::admit(&root).expect_err("no [estate] table here");
        assert!(
            matches!(err, EstateRootError::NotAnEstate { .. }),
            "got {err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("declares no [estate] table"),
            "the refusal must name the missing table: {message}"
        );
        assert!(
            message.contains("does not search parent directories"),
            "still no ancestor search: {message}"
        );
    }

    /// §4.4's last rule: an invalid manifest surfaces the exact parser
    /// diagnostic and never falls through to another estate. Here, one that
    /// is not valid TOML at all.
    #[test]
    fn a_malformed_manifest_surfaces_the_parser_diagnostic_and_never_falls_through() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        let inner = estate_root.join("inner");
        std::fs::create_dir_all(&inner).expect("dirs");
        write_estate(&estate_root, "outer-estate");
        std::fs::write(inner.join(WORKSPACE_FILE), "this is not [ toml").expect("write");

        let err = Workspace::admit(&inner).expect_err("a malformed manifest must refuse");
        assert!(
            matches!(err, EstateRootError::Invalid { .. }),
            "got {err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("the estate manifest is invalid"),
            "got {message}"
        );
        assert!(
            !message.contains("outer-estate"),
            "an invalid manifest must never point at a different estate: {message}"
        );
    }

    /// R-MVP1-3's named migration refusal survives exact-root admission: a
    /// legacy-vocabulary manifest is refused as invalid, carrying its own
    /// migration remedy rather than a generic unknown-field error.
    #[test]
    fn a_legacy_vocabulary_manifest_is_refused_with_its_migration_remedy() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("legacy");
        init_repo(&root);
        std::fs::write(
            root.join(WORKSPACE_FILE),
            "[workspace]\nname = \"legacy\"\n\n[[repository]]\nname = \"legacy\"\npath = \".\"\n",
        )
        .expect("legacy sergeant.toml");

        let err = Workspace::admit(&root).expect_err("legacy vocabulary must refuse");
        let EstateRootError::Invalid { source, .. } = &err else {
            panic!("expected Invalid, got {err:?}");
        };
        assert!(
            matches!(**source, WorkspaceError::LegacyVocabulary { .. }),
            "got {source}"
        );
        assert!(
            err.to_string().contains("[estate]"),
            "the migration remedy must name the new table: {err}"
        );
    }

    /// A manifest whose *schema* is broken — a group naming an undeclared
    /// repository — is inadmissible: §4.1 requires the file "satisfy the
    /// manifest schema", not merely parse.
    #[test]
    fn a_schema_invalid_manifest_is_inadmissible() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("estate");
        std::fs::create_dir_all(&root).expect("estate dir");
        write_estate(&root, "payments");
        std::fs::write(
            root.join(WORKSPACE_FILE),
            "[estate]\nname = \"payments\"\n\n[[repo]]\nname = \"solo\"\npath = \"repos/solo\"\n\n\
             [group.everything]\nrepos = [\"ghost\"]\n",
        )
        .expect("write");

        let err = Workspace::admit(&root).expect_err("an unknown group member is a schema defect");
        assert!(
            matches!(err, EstateRootError::Invalid { .. }),
            "got {err:?}"
        );
    }

    /// A declared repository that is not on disk yet does **not** make the
    /// estate inadmissible — that is a repository problem, not an
    /// estate-identity problem (the design capture's own wrongness contract:
    /// "a broken repo blocks works targeting it, not the estate"). The strict
    /// [`Workspace::resolve`] still refuses it, which is the half that
    /// protects a Work from binding a repository that is not really there.
    #[test]
    fn a_declared_repo_missing_from_disk_does_not_make_the_estate_inadmissible() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("estate");
        std::fs::create_dir_all(&root).expect("estate dir");
        init_repo(&root);
        std::fs::write(
            root.join(WORKSPACE_FILE),
            "[estate]\nname = \"payments\"\n\n[[repo]]\nname = \"ghost\"\npath = \"repos/ghost\"\n",
        )
        .expect("write");

        Workspace::admit(&root).expect("admission is about estate identity, not repo presence");
        assert!(
            Workspace::resolve(&root).is_err(),
            "the strict load must still refuse a repository that is not on disk"
        );
    }

    /// §4.4's second block: when a *valid* estate root is bound in the
    /// environment and sits strictly above the directory being addressed,
    /// the refusal names both roots and tells the operator to return.
    #[test]
    fn a_bound_estate_root_above_the_cwd_names_both_roots() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        std::fs::create_dir_all(&estate_root).expect("estate dir");
        write_estate(&estate_root, "payments");
        let estate_root = std::fs::canonicalize(&estate_root).expect("canonical estate root");
        let member = estate_root.join("repos").join("solo");

        let err = Workspace::admit(&member)
            .expect_err("a descendant is refused")
            .with_bound_root(estate_root.clone());
        assert!(
            matches!(err, EstateRootError::Descendant { .. }),
            "got {err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("this command must be run from the estate root"),
            "§4.4's own first line: {message}"
        );
        assert!(
            message.contains(&member.display().to_string()),
            "must name the current directory: {message}"
        );
        assert!(
            message.contains(&estate_root.display().to_string()),
            "must name the bound estate root: {message}"
        );
        assert!(
            message.contains(&format!("cd {}", estate_root.display())),
            "must give the exact cd remedy: {message}"
        );
    }

    /// A bound root that is **not** an ancestor of the directory being
    /// addressed never rewrites the diagnostic — the descendant variant is
    /// about "you are inside the bound estate, one level down", not about
    /// any two unrelated directories.
    #[test]
    fn an_unrelated_bound_root_does_not_become_the_descendant_diagnostic() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        let elsewhere = dir.path().join("elsewhere");
        std::fs::create_dir_all(&estate_root).expect("estate dir");
        std::fs::create_dir_all(&elsewhere).expect("elsewhere dir");
        write_estate(&estate_root, "payments");
        let estate_root = std::fs::canonicalize(&estate_root).expect("canonical");

        let err = Workspace::admit(&elsewhere)
            .expect_err("no estate here")
            .with_bound_root(estate_root);
        assert!(
            matches!(err, EstateRootError::NoEstate { .. }),
            "an unrelated bound root must leave the diagnostic alone: {err:?}"
        );
    }

    /// A bound root never rewrites an *invalid-manifest* refusal: §4.4
    /// requires the exact parser diagnostic, and trading it for a pointer at
    /// some other estate is exactly the fall-through the rule forbids.
    #[test]
    fn a_bound_root_never_replaces_an_invalid_manifest_diagnostic() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate_root = dir.path().join("estate");
        let inner = estate_root.join("inner");
        std::fs::create_dir_all(&inner).expect("dirs");
        write_estate(&estate_root, "payments");
        std::fs::write(inner.join(WORKSPACE_FILE), "this is not [ toml").expect("write");
        let estate_root = std::fs::canonicalize(&estate_root).expect("canonical");

        let err = Workspace::admit(&inner)
            .expect_err("malformed")
            .with_bound_root(estate_root);
        assert!(
            matches!(err, EstateRootError::Invalid { .. }),
            "got {err:?}"
        );
    }

    /// C10: `-C` names an exact root and earns no leniency for it — the same
    /// refusal fires, only the remedy line changes from `cd` to `-C`.
    #[test]
    fn the_dash_c_wording_changes_the_remedy_but_never_the_check() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("not-an-estate");
        std::fs::create_dir_all(&root).expect("dir");

        let err = Workspace::admit(&root).expect_err("no estate").via_flag();
        assert!(
            matches!(err, EstateRootError::NoEstate { .. }),
            "the check is identical: {err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("the directory named by -C"),
            "got {message}"
        );
        assert!(
            message.contains("sgt -C <estate-root>"),
            "the remedy must be the flag, not cd: {message}"
        );
    }

    /// `is_estate_root`/`root_data_dir_override` are the tolerant pair
    /// `resolve_data_dir` runs ahead of `sgt doctor`: they answer about
    /// exactly one directory, never a parent, and a structural defect
    /// elsewhere in the manifest does not stop them.
    #[test]
    fn the_tolerant_data_dir_lookup_is_exact_root_and_survives_a_broken_manifest() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("estate");
        let inner = root.join("inner");
        std::fs::create_dir_all(&inner).expect("dirs");
        std::fs::write(
            root.join(WORKSPACE_FILE),
            "[estate]\nname = \"w\"\ndata_dir = \"custom-data\"\n\n\
             [[repo]]\nname = \"solo\"\npath = \"repos/solo\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"fake\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"fake\"\n",
        )
        .expect("write");

        assert!(Workspace::is_estate_root(&root).expect("tolerant probe"));
        assert_eq!(
            Workspace::root_data_dir_override(&root).expect("tolerant lookup"),
            Some(root.join("custom-data")),
            "an unrelated structural defect must not stop the data-dir lookup"
        );
        // ...and it never looks up. A descendant is simply not an estate root.
        assert!(!Workspace::is_estate_root(&inner).expect("tolerant probe"));
        assert_eq!(
            Workspace::root_data_dir_override(&inner).expect("tolerant lookup"),
            None,
            "no ancestor search here either"
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

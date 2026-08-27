//! Estate: the repository surface work originates from (proposal §9).
//!
//! **Exact-root admission (estate-root proposal §4.1, Phase D).** An estate is
//! exactly the directory that itself contains a `sergeant.toml` which parses,
//! declares `[estate]`, and satisfies the manifest schema. There is no
//! ancestor walk and no zero-configuration Git fallback: `Estate::admit`
//! answers one deterministic question about one directory, and
//! [`EstateRootError`] carries §4.4's loud corrective diagnostic when the
//! answer is no. R-MVP1-12 (upward estate discovery across Git boundaries)
//! and the single-repository zero-config estate are both **superseded** —
//! a one-repository installation is an estate with one declared repository.
//!
//! **R-MVP1-3: estate vocabulary.** `[estate]` / `[[repo]]` / `[[profile]]` /
//! `[group.<name>]`, `deny_unknown_fields`. The pre-estate vocabulary
//! (`[workspace]`, `[[repository]]`) is not merely unknown — using it raises
//! a **named migration refusal** ([`EstateError::LegacyVocabulary`])
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
use crate::runtime::git::{GitError, canonical_git_common_dir, canonical_git_top_level};

/// Checked-in estate configuration file name (D1: `depot.toml` upstream).
pub const MANIFEST_FILE: &str = "sergeant.toml";

/// The one directory every repository mount lives under, relative to the
/// estate root (§6.1: `<estate-root>/repos/<name>`). Derived, never
/// configured — the estate owns its checkouts, and a path field would let a
/// manifest alias one in from somewhere else (§6.2).
pub const REPOS_DIR: &str = "repos";

/// §6.1's derived mount for one declared repository. The *only* place a
/// repository path is ever computed.
pub fn mount_path(estate_root: &Path, name: &str) -> PathBuf {
    estate_root.join(REPOS_DIR).join(name)
}

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
/// too (`sergeant-rs-workspace's knowledge/evidence/gauntlet/notes/d2-setting-sources-measurement-2026-08-12.md`)
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

/// One repository bound into a estate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositorySpec {
    /// Name used in surfaces, bindings and `--repo` selection.
    pub name: String,
    /// Absolute, canonical path to the repository's top level — §6.1's
    /// derived mount `<estate-root>/repos/<name>`, validated as this
    /// estate's own ordinary checkout (see `validate_mount`).
    pub path: PathBuf,
}

/// One `[[repo]]` entry read straight off `sergeant.toml`, before any
/// on-disk existence check — see [`Estate::declared_repos`]. Unlike
/// [`RepositorySpec::path`] (mount-validated, guaranteed to be this estate's
/// own ordinary checkout), `path` here is only §6.1's derived mount
/// (`<root>/repos/<name>`) and may point at nothing.
#[derive(Debug, Clone)]
pub struct DeclaredRepo {
    /// Repository name.
    pub name: String,
    /// Declared path, joined but not resolved through git.
    pub path: PathBuf,
    /// `origin` from `[[repo]]`, when declared.
    pub origin: Option<String>,
    /// `upstream` from `[[repo]]`, when declared (#112) — the URL a mount's
    /// `upstream` remote is expected to carry. Opaque and forge-neutral: no
    /// host, forge or CLI is inferred from it anywhere.
    pub upstream: Option<String>,
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
    /// `sergeant-rs-workspace's knowledge/evidence/gauntlet/notes/d2-setting-sources-measurement-2026-08-12.md`):
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

/// One `[[knowledge]]` source: a local path the estate declares as
/// **read-only evidence** (A1 §2/§3, F9).
///
/// Mirrors [`RepositorySpec`] deliberately — a plain name, a resolved
/// absolute path — and differs in exactly the two ways that matter:
///
/// * **The path is declared, not derived.** A repository mount is always
///   `<estate-root>/repos/<name>` (§6.1) precisely because it is a mutation
///   surface the estate owns. A knowledge source is somewhere else on the
///   machine by definition; deriving it would defeat the point.
/// * **It is never a mount** (A1-03). Nothing cuts a worktree from it,
///   nothing branches it, nothing writes to it. Declaring a path here grants
///   read access to Atlas's scanner and no authority whatsoever — which is
///   why [`EstateError::KnowledgePathInsideEstate`] refuses a declaration
///   that would alias a location the estate already owns and mutates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeSpec {
    /// Name used in coverage rows, `sgt knowledge list`, and source
    /// coordinates. Plain-name rules, exactly like a repository's.
    pub name: String,
    /// Absolute path to the source root. A relative declaration joins onto
    /// the estate root, the same resolution `surfaces_dir`/`data_dir` use.
    pub path: PathBuf,
    /// Per-source ignore globs, extending the scanner's built-in deny set
    /// (F10). Never *narrowing* it: a source cannot opt back into the
    /// defaults it was protected by.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,
    /// **F10a**: the columns of this source's tabular datasets whose text may
    /// become retrievable context units. **Absent means none**, and that
    /// default is the whole control.
    ///
    /// A separate axis from [`Self::ignore`] rather than an extension of it,
    /// because the two govern different boundaries. `ignore` extends F10's
    /// *acquisition* deny set — which bytes are read at all — and speaks in
    /// paths. This governs *exposure* — which values may leave a dataset as
    /// text — and speaks in columns, because a CSV of support tickets is an
    /// ordinary knowledge source whose `email` column is not, and no path
    /// pattern can express that.
    ///
    /// Registration is not exposure. A dataset with no allowlist is still
    /// discovered, counted, and profiled in aggregate; what it does not do is
    /// turn a row's text into something retrievable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_fields: Vec<String>,
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

/// A resolved estate: topology and defaults, never transient state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Estate {
    /// Estate name, from `[estate] name`.
    pub name: String,
    /// The estate root: the canonical directory holding `sergeant.toml`, and
    /// the directory every mount is derived beneath (§6.1).
    pub root: PathBuf,
    /// Repositories in the estate, in declaration order.
    pub repositories: Vec<RepositorySpec>,
    /// Estate-level default backend (§13's third precedence tier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_backend: Option<String>,
    /// Estate-level default workflow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workflow: Option<String>,
    /// Profiles declared for this estate (§14).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<Profile>,
    /// Path of the `sergeant.toml` that produced this estate. `Option` only
    /// because the type predates exact-root admission; every estate this
    /// build can construct has one.
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
    /// name. A name absent from this map — a `[[repo]]` entry that declared
    /// no `instructions` — resolves to [`InstructionPolicy::Suppress`] via
    /// [`Estate::instruction_policy`].
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
    /// A name absent from this map — a `[[repo]]` entry that never declared
    /// `origin` — has no known origin.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub repository_origin: BTreeMap<String, String>,
    /// Per-repository `upstream` (#112), keyed by repository name — the URL
    /// the mount's `upstream` remote is declared to carry.
    ///
    /// The manifest is the authority; the remote is a *materialization* of
    /// this declaration, ensured by `sgt repo add` where config mutation is
    /// already legitimate and reported as drift by `sgt doctor` everywhere
    /// else. Nothing in execution consults it (R-NS-4), and nothing anywhere
    /// derives a forge, host or CLI from it — the URL is opaque.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub repository_upstream: BTreeMap<String, String>,
    /// `[[knowledge]]` declarations (F9, A1 §2): local paths this estate
    /// reads as evidence, in declaration order. Read-only by construction —
    /// nothing in execution consults this list, and nothing ever will: a
    /// knowledge source is not a mount (A1-03).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub knowledge: Vec<KnowledgeSpec>,
    /// `[estate] retention` (Q3/A2, W3): the declared Work-retention cap, or
    /// `None` for the built-in [`DEFAULT_RETENTION`]. Read once by
    /// `daemon::start_with` and pinned into the prune policy for the life of
    /// the process — a manifest edited under a running daemon does not
    /// re-arm it, exactly like `surfaces_dir` and `data_dir`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<u32>,
}

/// Q3/A2's ratified default: how many Works this estate retains when
/// `[estate] retention` is absent. 1000 Works ≈ 1.8 GB bounded total on the
/// measured 1.8 MB/Work basis (issue #17's rulings record, 2026-08-21).
pub const DEFAULT_RETENTION: u32 = 1000;

/// The validation floor for a declared `retention`.
///
/// Not a correctness bound — the prune predicate is sound at any N, because
/// a non-terminal or unsettled Work is never prunable whatever the cap says.
/// This exists to refuse an obviously destructive typo (`retention = 1`,
/// `retention = 0`) at parse time rather than after the segments are gone,
/// and it is set at 64 because below that the estate retains less history
/// than its own in-memory terminal caches hold (`TERMINAL_RUN_CACHE_CAPACITY`
/// = 512, `TERMINAL_WORK_CACHE_CAPACITY` = 1024) — a knob under its own
/// working set is a mistake, not a choice. **Ratify-at-review item 3.**
pub const MIN_RETENTION: u32 = 64;

/// Refuse a declared retention below [`MIN_RETENTION`] by name, at parse
/// time. `deny_unknown_fields` gives no help here — the key is known and the
/// value is in range for `u32`; only a named refusal explains the floor.
fn validate_retention(retention: Option<u32>, file: &str) -> Result<(), EstateError> {
    if let Some(value) = retention
        && value < MIN_RETENTION
    {
        return Err(EstateError::RetentionBelowFloor {
            file: file.to_string(),
            value,
            floor: MIN_RETENTION,
            default: DEFAULT_RETENTION,
        });
    }
    Ok(())
}

/// An admitted estate root (§4.1): the canonical directory that *is* the
/// estate, and the manifest that made it one. Produced only by
/// [`Estate::admit`], so holding one is proof the exact-root check has
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
        source: Box<EstateError>,
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

/// Failure resolving a estate.
#[derive(Debug, thiserror::Error)]
pub enum EstateError {
    /// Git itself failed while resolving the estate.
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
    /// §6.1: the derived mount `<estate-root>/repos/<name>` is missing, or
    /// is not a Git repository at all. §15: name the expected derived path.
    #[error(
        "{file} declares repository {name:?}, whose mount {path} is missing or is not a git \
         repository. Every repository is mounted at <estate-root>/repos/<name> (§6.1); clone or \
         place one there, or `sgt repo add {name} --origin <url>`"
    )]
    RepositoryNotFound {
        /// Config file that declared it.
        file: String,
        /// Declared repository name.
        name: String,
        /// The derived mount path that failed.
        path: String,
    },
    /// §6.2: the mount is a symlink, an alias, or otherwise resolves to a
    /// checkout that is not the estate-owned one. §15: name the expected
    /// derived path **and** the actual Git top level.
    #[error(
        "{file} declares repository {name:?} at {expected}, but that path resolves to a different \
         checkout: git reports its top level as {actual}. A repository mount must be the \
         estate's own checkout at <estate-root>/repos/<name> — symlinks, ../ aliases and \
         another estate's clone are refused (§6.2). Separate estates use separate clones, even \
         for the same upstream repository."
    )]
    RepositoryMountAliased {
        /// Config file that declared it.
        file: String,
        /// Declared repository name.
        name: String,
        /// The derived, canonical mount path.
        expected: String,
        /// What `git rev-parse --show-toplevel` actually answered.
        actual: String,
    },
    /// §6.2/§8.1 check 4: the mount is a **linked worktree**, not an
    /// ordinary primary checkout — its Git common directory lives in some
    /// other repository. Declaring a Work's own linked worktree as a
    /// repository source is exactly the recursion this refuses.
    #[error(
        "{file} declares repository {name:?} at {path}, which is a linked worktree, not an \
         ordinary checkout: its git common dir is {common_dir}, outside the mount. A repository \
         mount owns its own branches, common directory, worktree registry and sergeant/* refs \
         (§6.2); a linked worktree owns none of them. Clone the repository into \
         <estate-root>/repos/{name} instead."
    )]
    RepositoryMountIsLinkedWorktree {
        /// Config file that declared it.
        file: String,
        /// Declared repository name.
        name: String,
        /// The derived mount path.
        path: String,
        /// The common dir that gave it away.
        common_dir: String,
    },
    /// §6.1: `[[repo]] path` is removed. Named explicitly rather than left
    /// to `deny_unknown_fields`, for the same reason R-MVP1-3's legacy
    /// vocabulary is: a key that used to be *required* deserves a migration
    /// notice, not "unknown field `path`".
    #[error(
        "{file} declares repository {name:?} with a `path` key, which no longer exists. \
         Repository mounts are derived, not configured: every repository lives at \
         <estate-root>/repos/<name> (§6.1). Delete the `path` line; if the checkout is \
         somewhere else, move or re-clone it to repos/{name}."
    )]
    RepositoryPathDeclared {
        /// Config file that declared it.
        file: String,
        /// The repository whose entry still carries `path`.
        name: String,
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
    /// A declared knowledge name is not usable as a plain path component.
    /// Mirrors [`Self::InvalidRepositoryName`]: the name is joined onto
    /// coverage coordinates and reported back in CLI output, so anything but
    /// a plain name is refused where it is cheap to refuse.
    #[error("{file} declares knowledge source name {name:?}, which is not a plain directory name")]
    InvalidKnowledgeName {
        /// Config file that declared it.
        file: String,
        /// The offending name.
        name: String,
    },
    /// Two knowledge sources share a name; coverage and source coordinates
    /// are keyed by name, so this would silently merge two sources into one.
    #[error("{file} declares knowledge source name {name:?} twice")]
    DuplicateKnowledge {
        /// Config file that declared it.
        file: String,
        /// The repeated name.
        name: String,
    },
    /// F9's path-containment refusal (panel finding 6): a `[[knowledge]]`
    /// path that canonicalizes to a location inside a declared repository
    /// mount, inside `surfaces_dir`, or inside `data_dir`.
    ///
    /// Refused at the same station as [`Self::RepositoryPathDeclared`] — the
    /// manifest parse, before any daemon, scan or Work exists — and for a
    /// closely related reason. That variant refuses a *mount* being
    /// redirected somewhere the estate does not own; this one refuses
    /// *read-only evidence* being pointed at somewhere the estate does own
    /// and actively mutates. A knowledge source is evidence about a stable
    /// world (A1-03); a Work surface, a repository mount, and the daemon's
    /// own data dir are the three places whose bytes change underneath a
    /// scan by design, and indexing them would attribute mutations the
    /// estate itself made to an outside world it was supposed to be
    /// observing.
    #[error(
        "{file} declares knowledge source {name:?} at {path}, which resolves inside {what} \
         ({inside}). A knowledge source is read-only evidence, never a mount (A1-03): it must \
         not name a location this estate already owns and mutates. Point it at a path outside \
         {inside}, or remove the entry."
    )]
    KnowledgePathInsideEstate {
        /// Config file that declared it.
        file: String,
        /// The knowledge source name.
        name: String,
        /// The declared path, as resolved.
        path: String,
        /// What owns the containing location, in words — `repository mount
        /// "api"`, `the surfaces directory`, `the data directory`.
        what: String,
        /// The containing path itself.
        inside: String,
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
    /// A profile's `network_access` option is neither `"true"` nor `"false"`
    /// (#262, split-hardening W5 review finding #2). Refused at parse time,
    /// mirroring [`Self::InvalidPermissionMode`]: a typo here must fail
    /// loudly at estate load, not silently at launch.
    #[error("{file} declares profile {profile:?} with {source}")]
    InvalidNetworkAccess {
        /// Config file that declared it.
        file: String,
        /// The profile naming the bad value.
        profile: String,
        /// The underlying vocabulary mismatch.
        source: crate::domain::profile::UnknownNetworkAccess,
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
    /// `[estate] retention` is below the floor a bounded-retention policy is
    /// allowed to declare (W3).
    #[error(
        "{file} declares [estate] retention = {value}, below the minimum of {floor}. \
         Retention is how many Works of history this estate keeps; a value below \
         {floor} retains less than the daemon's own in-memory caches hold. \
         Remove the key to use the default of {default}, or raise it to at least {floor}"
    )]
    RetentionBelowFloor {
        /// Config file that declared it.
        file: String,
        /// The declared value.
        value: u32,
        /// The validation floor ([`MIN_RETENTION`]).
        floor: u32,
        /// The built-in default ([`DEFAULT_RETENTION`]).
        default: u32,
    },
}

/// The `sergeant.toml` file shape (§9, R-MVP1-3's estate vocabulary).
///
/// `estate` is `Option` at the *parser* level, not at the admission level: a
/// `sergeant.toml` with no `[estate]` table is a member repository's own
/// config, and [`Estate::admit`] refuses it by name
/// ([`EstateRootError::NotAnEstate`]) rather than mistaking it for an estate
/// root. The field stays optional here because `src/domain/manifest.rs`'s
/// edit pen legitimately parses a manifest mid-scaffold, before `[estate]`
/// has been written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EstateFile {
    #[serde(default)]
    estate: Option<EstateSection>,
    #[serde(default)]
    repo: Vec<RepositoryEntry>,
    #[serde(default)]
    knowledge: Vec<KnowledgeEntry>,
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
    /// Q3/A2: how many Works of history this estate retains. Absent is
    /// [`DEFAULT_RETENTION`]. Validated against [`MIN_RETENTION`] at parse.
    #[serde(default)]
    retention: Option<u32>,
}

/// One `[[repo]]` entry.
///
/// **§6.1: there is no `path`.** A repository's mount is derived, not
/// configured — `<estate-root>/repos/<name>`, always. A manifest that still
/// declares one gets [`EstateError::RepositoryPathDeclared`], a named
/// removal notice, rather than `deny_unknown_fields`' generic "unknown
/// field" pointing at a key that used to be required.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositoryEntry {
    name: String,
    /// R-MVP1-4. Unset means [`InstructionPolicy::Suppress`].
    #[serde(default)]
    instructions: InstructionPolicy,
    /// MVP-3 `sgt repo add`'s clone-or-verify source. Recorded, never acted
    /// on by this module beyond bookkeeping (see [`Estate::repository_origin`]).
    #[serde(default)]
    origin: Option<String>,
    /// #112's forge-neutral upstream declaration: the URL the mount's
    /// `upstream` remote should carry. Recorded here and acted on only where
    /// config mutation is already legitimate (`src/domain/manifest.rs`'s
    /// clone-or-verify); this module never touches a remote.
    #[serde(default)]
    upstream: Option<String>,
}

/// One `[[knowledge]]` entry (F9), mirroring [`RepositoryEntry`]'s shape and
/// its `deny_unknown_fields` discipline: a checked-in manifest is an
/// instruction, and a misspelled key that silently means nothing is worse
/// than a refusal naming the line.
///
/// Unlike `[[repo]]`, `path` is a real key here and is *required* — see
/// [`KnowledgeSpec`] for why the derived-mount rule does not and must not
/// apply to read-only evidence.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KnowledgeEntry {
    name: String,
    path: PathBuf,
    #[serde(default)]
    ignore: Vec<String>,
    /// F10a. `#[serde(default)]` is the refusal: an operator who writes no
    /// `context_fields` key has declared the empty allowlist, and the empty
    /// allowlist exposes nothing.
    #[serde(default)]
    context_fields: Vec<String>,
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

/// `dir` with every ancestor canonicalized but the final component left
/// exactly as named — see [`validate_mount`]'s own comment for why the leaf
/// must not be resolved.
fn canonical_leaf(dir: &Path) -> PathBuf {
    match (dir.parent(), dir.file_name()) {
        (Some(parent), Some(leaf)) => std::fs::canonicalize(parent)
            .map(|parent| parent.join(leaf))
            .unwrap_or_else(|_| dir.to_path_buf()),
        _ => dir.to_path_buf(),
    }
}

/// Probe raw TOML for a removed `[[repo]] path` key **before** the real
/// parse, for exactly the reason [`check_legacy_vocabulary`] probes for the
/// legacy tables: `deny_unknown_fields` would answer "unknown field `path`"
/// about a key that was *required* until this release, which names nothing
/// an operator can act on. §6.1's removal deserves the migration notice.
fn check_removed_repo_path(text: &str, file: &str) -> Result<(), EstateError> {
    let value: toml::Value = toml::from_str(text).map_err(|source| EstateError::Malformed {
        path: file.to_string(),
        source,
    })?;
    let Some(repos) = value.get("repo").and_then(toml::Value::as_array) else {
        return Ok(());
    };
    for entry in repos {
        let Some(table) = entry.as_table() else {
            continue;
        };
        if table.contains_key("path") {
            return Err(EstateError::RepositoryPathDeclared {
                file: file.to_string(),
                name: table
                    .get("name")
                    .and_then(toml::Value::as_str)
                    .unwrap_or("<unnamed>")
                    .to_string(),
            });
        }
    }
    Ok(())
}

/// §6.1/§6.2/§8.1 checks 1–4: is `mount` really this estate's own checkout
/// for `name`?
///
/// 1. The derived mount exists and is a Git repository at all.
/// 2. Its canonical Git top level is *exactly* the canonical mount — which
///    is what catches a symlinked mount, a `../` alias, and another estate's
///    clone reached through either.
/// 3. Its canonical Git common directory lives inside that top level — which
///    is what catches a linked worktree admitted as a source, including a
///    Work's own surface.
///
/// Returns the canonical mount on success, so callers record the resolved
/// path rather than the declared one.
///
/// **Public because §8.4 makes the daemon repeat this, not because two
/// implementations of it exist.** `Estate::resolve` calls it while parsing the
/// manifest; [`crate::runtime::preflight`] calls this same function again per
/// *selected* repository at admission — "the API/daemon repeats and
/// authoritatively enforces the mechanical contract" — and maps the three
/// error variants it can return onto §8.1's checks 1–4 rather than re-deriving
/// the same three questions from git a second way.
pub fn validate_mount(file: &str, name: &str, mount: &Path) -> Result<PathBuf, EstateError> {
    // Check 2, first half, and the one case a canonical comparison alone
    // cannot see: the mount *itself* being a symlink. `canonicalize` would
    // happily follow it and then agree with git that the target is the top
    // level — which is exactly the shared-mount aliasing §6.2 refuses, so it
    // is caught before any canonicalization happens.
    if mount
        .symlink_metadata()
        .is_ok_and(|meta| meta.file_type().is_symlink())
    {
        let actual = std::fs::canonicalize(mount)
            .unwrap_or_else(|_| std::fs::read_link(mount).unwrap_or_else(|_| mount.to_path_buf()));
        return Err(EstateError::RepositoryMountAliased {
            file: file.to_string(),
            name: name.to_string(),
            expected: canonical_leaf(mount).display().to_string(),
            actual: actual.display().to_string(),
        });
    }
    let top_level =
        canonical_git_top_level(mount).map_err(|_| EstateError::RepositoryNotFound {
            file: file.to_string(),
            name: name.to_string(),
            path: mount.display().to_string(),
        })?;
    // Canonicalized on both sides: `canonical_git_top_level` already
    // resolves symlinks, so comparing it against a raw `mount` would report
    // every macOS `/var` -> `/private/var` host as aliased (#127). The final
    // component is deliberately *not* resolved (it cannot be a symlink — the
    // check above already refused that), so an estate reached through a
    // symlinked ancestor still compares equal while a mount that is itself an
    // alias does not.
    let canonical_mount = canonical_leaf(mount);
    if top_level != canonical_mount {
        return Err(EstateError::RepositoryMountAliased {
            file: file.to_string(),
            name: name.to_string(),
            expected: canonical_mount.display().to_string(),
            actual: top_level.display().to_string(),
        });
    }
    // A primary checkout's common dir is `<top level>/.git`; a linked
    // worktree's points back into the repository it was cut from.
    let common_dir =
        canonical_git_common_dir(mount).map_err(|_| EstateError::RepositoryNotFound {
            file: file.to_string(),
            name: name.to_string(),
            path: mount.display().to_string(),
        })?;
    if !common_dir.starts_with(&canonical_mount) {
        return Err(EstateError::RepositoryMountIsLinkedWorktree {
            file: file.to_string(),
            name: name.to_string(),
            path: canonical_mount.display().to_string(),
            common_dir: common_dir.display().to_string(),
        });
    }
    Ok(canonical_mount)
}

/// `path` with as much of it canonicalized as actually exists on disk, the
/// non-existent tail re-appended verbatim.
///
/// Plain [`std::fs::canonicalize`] answers nothing at all for a path that is
/// not there yet, and a declared knowledge path legitimately may not be
/// (the same "declared but not on disk" tolerance `[[repo]]` entries get from
/// [`Estate::from_config_structural`]). Resolving the existing prefix is what
/// makes the containment check symlink-proof: `knowledge/link -> repos/api`
/// and a literal `repos/api` compare equal here, so the refusal cannot be
/// walked around with a symlink.
fn canonical_best_effort(path: &Path) -> PathBuf {
    if let Ok(resolved) = std::fs::canonicalize(path) {
        return resolved;
    }
    match (path.parent(), path.file_name()) {
        (Some(parent), leaf) if parent != path => match leaf {
            Some(leaf) => canonical_best_effort(parent).join(leaf),
            None => canonical_best_effort(parent),
        },
        _ => path.to_path_buf(),
    }
}

/// F9's `[[knowledge]]` resolution and validation, shared by the strict and
/// structural parsers so the two can never disagree about what a knowledge
/// declaration means.
///
/// Three checks, all at manifest-parse station:
///
/// 1. plain-name (mirrors [`EstateError::InvalidRepositoryName`]),
/// 2. no duplicate names (mirrors [`EstateError::DuplicateRepository`]),
/// 3. **path containment** — [`EstateError::KnowledgePathInsideEstate`],
///    panel finding 6.
///
/// Existence is deliberately *not* checked: a knowledge path that is not
/// mounted right now is a coverage fact (the scanner reports it
/// `unavailable`), not a manifest defect, exactly as a not-yet-cloned
/// `[[repo]]` is a repository problem rather than an estate-identity one.
fn resolve_knowledge(
    file: &str,
    root: &Path,
    entries: Vec<KnowledgeEntry>,
    repositories: &[RepositorySpec],
    surfaces_dir: Option<&Path>,
    data_dir: Option<&Path>,
) -> Result<Vec<KnowledgeSpec>, EstateError> {
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    // The three families of estate-owned, estate-mutated location. Built
    // once, canonicalized the same way the candidate is, so the comparison
    // below is between two resolved paths and never between one of each.
    let mut owned: Vec<(String, PathBuf)> = Vec::new();
    for repo in repositories {
        owned.push((
            format!("repository mount {:?}", repo.name),
            canonical_best_effort(&mount_path(root, &repo.name)),
        ));
    }
    owned.push((
        "the surfaces directory".to_string(),
        canonical_best_effort(&match surfaces_dir {
            Some(dir) => dir.to_path_buf(),
            // The daemon's own default when the manifest declares none.
            None => data_dir
                .map(Path::to_path_buf)
                .unwrap_or_else(|| root.join(crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR))
                .join("surfaces"),
        }),
    ));
    owned.push((
        "the data directory".to_string(),
        canonical_best_effort(&match data_dir {
            Some(dir) => dir.to_path_buf(),
            None => root.join(crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR),
        }),
    ));

    let mut seen = BTreeSet::new();
    let mut resolved = Vec::with_capacity(entries.len());
    for entry in entries {
        if !is_plain_name(&entry.name) {
            return Err(EstateError::InvalidKnowledgeName {
                file: file.to_string(),
                name: entry.name,
            });
        }
        if !seen.insert(entry.name.clone()) {
            return Err(EstateError::DuplicateKnowledge {
                file: file.to_string(),
                name: entry.name,
            });
        }
        // Relative declarations join onto the estate root — the same rule
        // `surfaces_dir` and `data_dir` already follow, so one manifest has
        // one relative-path convention rather than two.
        let joined = if entry.path.is_absolute() {
            entry.path.clone()
        } else {
            root.join(&entry.path)
        };
        let candidate = canonical_best_effort(&joined);
        for (what, owner) in &owned {
            if candidate.starts_with(owner) {
                return Err(EstateError::KnowledgePathInsideEstate {
                    file: file.to_string(),
                    name: entry.name,
                    path: candidate.display().to_string(),
                    what: what.clone(),
                    inside: owner.display().to_string(),
                });
            }
        }
        resolved.push(KnowledgeSpec {
            name: entry.name,
            path: candidate,
            ignore: entry.ignore,
            context_fields: entry.context_fields,
        });
    }
    Ok(resolved)
}

/// Probe raw TOML for the pre-estate vocabulary **before** the real parse
/// (R-MVP1-3: "one probe before parse, not a second parser" — this reads the
/// same `toml::Value` `deny_unknown_fields` would reject anyway, just early
/// enough to name the migration instead of a generic unknown-field error).
fn check_legacy_vocabulary(text: &str, file: &str) -> Result<(), EstateError> {
    let value: toml::Value = toml::from_str(text).map_err(|source| EstateError::Malformed {
        path: file.to_string(),
        source,
    })?;
    let Some(table) = value.as_table() else {
        return Ok(());
    };
    for (legacy, expected, remedy) in LEGACY_TABLES {
        if table.contains_key(*legacy) {
            return Err(EstateError::LegacyVocabulary {
                file: file.to_string(),
                found: (*legacy).to_string(),
                expected: (*expected).to_string(),
                remedy: (*remedy).to_string(),
            });
        }
    }
    Ok(())
}

impl Estate {
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
        let manifest_path = root.join(MANIFEST_FILE);
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

    /// Parse and validate a `sergeant.toml` into a estate.
    pub fn from_config(config_path: &Path) -> Result<Self, EstateError> {
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
    pub fn from_config_allow_empty(config_path: &Path) -> Result<Self, EstateError> {
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
    pub fn declared_repos(config_path: &Path) -> Result<Vec<DeclaredRepo>, EstateError> {
        let file = config_path.display().to_string();
        let text = std::fs::read_to_string(config_path).map_err(|source| EstateError::Io {
            path: file.clone(),
            source,
        })?;
        check_legacy_vocabulary(&text, &file)?;
        check_removed_repo_path(&text, &file)?;
        let parsed: EstateFile =
            toml::from_str(&text).map_err(|source| EstateError::Malformed {
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
                return Err(EstateError::InvalidRepositoryName {
                    file,
                    name: entry.name,
                });
            }
            if !seen.insert(entry.name.clone()) {
                return Err(EstateError::DuplicateRepository {
                    file,
                    name: entry.name,
                });
            }
            let path = mount_path(&root, &entry.name);
            declared.push(DeclaredRepo {
                name: entry.name,
                path,
                origin: entry.origin,
                upstream: entry.upstream,
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
    /// identically; [`EstateError::DuplicateRepositoryPath`] — which needs
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
    /// estate", `sergeant-rs-workspace's knowledge/evidence/gauntlet/notes/estate-manifest-design-2026-08-11.md`).
    /// A repository an edit itself populates or verifies (`sgt repo add`'s
    /// `populate_or_verify`) is already checked on disk by that caller,
    /// directly — this validator does not need to repeat it.
    pub fn from_config_structural(config_path: &Path) -> Result<Self, EstateError> {
        Self::from_config_impl_structural(config_path)
    }

    /// [`Self::declared_repos`]'s sibling for `[group.<name>]`: every
    /// declared group, membership validated against declared repository
    /// names (the same [`EstateError::UnknownGroupMember`] check
    /// [`Self::from_config_impl`] runs), without resolving any repository on
    /// disk. Used where only membership is wanted — `sgt run --group`'s
    /// client-side expansion (`src/cli.rs`) — so an unrelated missing
    /// repository cannot block a group whose own members are all fine (same
    /// root cause and remedy as [`Self::from_config_structural`]).
    pub fn declared_groups(config_path: &Path) -> Result<BTreeMap<String, GroupSpec>, EstateError> {
        Ok(Self::from_config_impl_structural(config_path)?.groups)
    }

    fn from_config_impl(config_path: &Path, allow_empty_repos: bool) -> Result<Self, EstateError> {
        let file = config_path.display().to_string();
        let text = std::fs::read_to_string(config_path).map_err(|source| EstateError::Io {
            path: file.clone(),
            source,
        })?;
        // R-MVP1-3: the named migration refusal is a probe before the real
        // parse, not a second parser — it reads the same TOML
        // `deny_unknown_fields` would reject anyway, just early enough to
        // name the rename instead of a generic unknown-field error.
        check_legacy_vocabulary(&text, &file)?;
        check_removed_repo_path(&text, &file)?;
        let parsed: EstateFile =
            toml::from_str(&text).map_err(|source| EstateError::Malformed {
                path: file.clone(),
                source,
            })?;
        let root = config_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        if parsed.repo.is_empty() && !allow_empty_repos {
            return Err(EstateError::NoRepositories { file });
        }
        let mut seen = BTreeSet::new();
        // Identity of a repository is its resolved top level, not the name
        // the file chose for it: `path = "."` and `path = "./"` are one
        // checkout under two names, and only git can say so.
        let mut seen_paths: BTreeMap<PathBuf, String> = BTreeMap::new();
        let mut repositories = Vec::with_capacity(parsed.repo.len());
        let mut repository_policy = BTreeMap::new();
        let mut repository_origin = BTreeMap::new();
        let mut repository_upstream = BTreeMap::new();
        for entry in parsed.repo {
            if !is_plain_name(&entry.name) {
                return Err(EstateError::InvalidRepositoryName {
                    file,
                    name: entry.name,
                });
            }
            if !seen.insert(entry.name.clone()) {
                return Err(EstateError::DuplicateRepository {
                    file,
                    name: entry.name,
                });
            }
            // §6.1: the mount is derived, never declared, and validated as
            // this estate's own ordinary checkout before anything binds it.
            let resolved = validate_mount(&file, &entry.name, &mount_path(&root, &entry.name))?;
            if let Some(first) = seen_paths.get(&resolved) {
                return Err(EstateError::DuplicateRepositoryPath {
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
            if let Some(upstream) = entry.upstream {
                repository_upstream.insert(entry.name.clone(), upstream);
            }
            repositories.push(RepositorySpec {
                name: entry.name,
                path: resolved,
            });
        }

        let mut seen = BTreeSet::new();
        for profile in &parsed.profile {
            if !seen.insert(profile.name.clone()) {
                return Err(EstateError::DuplicateProfile {
                    file,
                    name: profile.name.clone(),
                });
            }
            // #47: an unrecognized permission_mode is refused here, at
            // config load, rather than surfacing later as an unmeasured CLI
            // argument failure at launch time.
            if let Err(source) = profile.permission_mode() {
                return Err(EstateError::InvalidPermissionMode {
                    file,
                    profile: profile.name.clone(),
                    source,
                });
            }
            // #262: an unrecognized network_access is refused here, at
            // config load, rather than surfacing later as a lazy PREPARE-time
            // failure — same shape as permission_mode above.
            if let Err(source) = profile.network_access() {
                return Err(EstateError::InvalidNetworkAccess {
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
                    return Err(EstateError::UnknownGroupMember {
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

        let (name, default_backend, default_workflow, surfaces_dir, data_dir, retention) =
            match parsed.estate {
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
                    estate.retention,
                ),
                None => (repo_name(&root), None, None, None, None, None),
            };
        validate_retention(retention, &file)?;
        // F9: resolved last, because containment is stated against the
        // repository mounts and the two directory overrides this parse has
        // only just finished computing.
        let knowledge = resolve_knowledge(
            &file,
            &root,
            parsed.knowledge,
            &repositories,
            surfaces_dir.as_deref(),
            data_dir.as_deref(),
        )?;

        Ok(Self {
            name,
            root,
            repositories,
            knowledge,
            default_backend,
            default_workflow,
            profiles: parsed.profile,
            config_path: Some(config_path.to_path_buf()),
            surfaces_dir,
            data_dir,
            repository_policy,
            groups,
            repository_origin,
            repository_upstream,
            retention,
        })
    }

    /// [`Self::from_config_impl`] with the per-repository `git rev-parse
    /// --show-toplevel` resolution dropped — see
    /// [`Self::from_config_structural`]'s own doc for why this exists and
    /// what it deliberately cannot check. Always allows an empty `[[repo]]`
    /// list (every caller is a manifest edit, which may legitimately be
    /// scaffolding a repo-less estate — same reason
    /// [`Self::from_config_allow_empty`] relaxes it).
    fn from_config_impl_structural(config_path: &Path) -> Result<Self, EstateError> {
        let file = config_path.display().to_string();
        let text = std::fs::read_to_string(config_path).map_err(|source| EstateError::Io {
            path: file.clone(),
            source,
        })?;
        check_legacy_vocabulary(&text, &file)?;
        check_removed_repo_path(&text, &file)?;
        let parsed: EstateFile =
            toml::from_str(&text).map_err(|source| EstateError::Malformed {
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
        let mut repository_upstream = BTreeMap::new();
        for entry in parsed.repo {
            if !is_plain_name(&entry.name) {
                return Err(EstateError::InvalidRepositoryName {
                    file,
                    name: entry.name,
                });
            }
            if !seen.insert(entry.name.clone()) {
                return Err(EstateError::DuplicateRepository {
                    file,
                    name: entry.name,
                });
            }
            // §6.1's derived mount, **not** git-validated (see this fn's own
            // doc: that is the one thing a structural parse cannot check).
            let joined = mount_path(&root, &entry.name);
            repository_policy.insert(entry.name.clone(), entry.instructions);
            if let Some(origin) = &entry.origin {
                repository_origin.insert(entry.name.clone(), origin.clone());
            }
            if let Some(upstream) = &entry.upstream {
                repository_upstream.insert(entry.name.clone(), upstream.clone());
            }
            repositories.push(RepositorySpec {
                name: entry.name,
                path: joined,
            });
        }

        let mut seen = BTreeSet::new();
        for profile in &parsed.profile {
            if !seen.insert(profile.name.clone()) {
                return Err(EstateError::DuplicateProfile {
                    file,
                    name: profile.name.clone(),
                });
            }
            if let Err(source) = profile.permission_mode() {
                return Err(EstateError::InvalidPermissionMode {
                    file,
                    profile: profile.name.clone(),
                    source,
                });
            }
            // #262: an unrecognized network_access is refused here, at
            // config load, rather than surfacing later as a lazy PREPARE-time
            // failure — same shape as permission_mode above.
            if let Err(source) = profile.network_access() {
                return Err(EstateError::InvalidNetworkAccess {
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
                    return Err(EstateError::UnknownGroupMember {
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

        let (name, default_backend, default_workflow, surfaces_dir, data_dir, retention) =
            match parsed.estate {
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
                    estate.retention,
                ),
                None => (repo_name(&root), None, None, None, None, None),
            };
        validate_retention(retention, &file)?;
        // F9: resolved last, because containment is stated against the
        // repository mounts and the two directory overrides this parse has
        // only just finished computing.
        let knowledge = resolve_knowledge(
            &file,
            &root,
            parsed.knowledge,
            &repositories,
            surfaces_dir.as_deref(),
            data_dir.as_deref(),
        )?;

        Ok(Self {
            name,
            root,
            repositories,
            knowledge,
            default_backend,
            default_workflow,
            profiles: parsed.profile,
            config_path: Some(config_path.to_path_buf()),
            surfaces_dir,
            data_dir,
            repository_policy,
            groups,
            repository_origin,
            repository_upstream,
            retention,
        })
    }

    /// The profile with this name, if the estate declares one.
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

    /// This repository's declared `upstream` (#112), if the manifest records
    /// one. The URL is returned exactly as declared — opaque, forge-neutral,
    /// never parsed for a host.
    pub fn repository_upstream(&self, repository: &str) -> Option<&str> {
        self.repository_upstream.get(repository).map(String::as_str)
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
    pub fn is_estate_root(dir: &Path) -> Result<bool, EstateError> {
        let manifest_path = dir.join(MANIFEST_FILE);
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
    pub fn root_data_dir_override(dir: &Path) -> Result<Option<PathBuf>, EstateError> {
        if !Self::is_estate_root(dir)? {
            return Ok(None);
        }
        estate_data_dir_override(&dir.join(MANIFEST_FILE), dir)
    }

    /// Restrict the estate to the named repositories (the submit request's
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
                "no repositories selected for estate {:?}; an empty selection is refused, \
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
                    "repository selection lists {name:?} twice for estate {:?}",
                    self.name
                ));
            }
            match self.repositories.iter().find(|r| &r.name == name) {
                Some(repo) => selected.push(repo.clone()),
                None => {
                    return Err(format!(
                        "estate {:?} has no repository {name:?} (has: {})",
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
        .unwrap_or_else(|| "estate".to_string())
}

/// Whether `config_path` carries an `[estate]` table — [`Estate::admit`]'s
/// own predicate. `Ok(false)` for a file that cannot even be read
/// (permission, race — indistinguishable from "no file here"). `Err` for one
/// that CAN be read but is malformed or carries legacy vocabulary
/// (W5/R-MVP1-3): §4.4's last rule is that an invalid manifest surfaces its
/// exact diagnostic and never falls through.
fn estate_table_check(config_path: &Path) -> Result<bool, EstateError> {
    let Ok(text) = std::fs::read_to_string(config_path) else {
        return Ok(false);
    };
    let file = config_path.display().to_string();
    check_legacy_vocabulary(&text, &file)?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|source| EstateError::Malformed { path: file, source })?;
    Ok(value
        .as_table()
        .is_some_and(|table| table.contains_key("estate")))
}

/// [`estate_table_check`]'s sibling for `data_dir` (ADR 0008(b)): the
/// `[estate] data_dir` string, if any, resolved onto `root` exactly as
/// [`Estate::from_config_impl_structural`] resolves it — relative joins
/// on, absolute passes through — but reached the same tolerant way
/// `estate_table_check` finds the `[estate]` table itself: a raw TOML
/// value, not a deserialize into [`EstateFile`]. This is what lets
/// [`Estate::root_data_dir_override`] read `data_dir` without also
/// demanding the rest of the manifest — repos, profiles, groups — be
/// structurally valid. Unreadable (permission, race) answers `None`, the
/// same "indistinguishable from no file here" tolerance `estate_table_check`
/// applies; by the time this runs, [`Estate::is_estate_root`] has already
/// required the file to parse as TOML and carry no legacy vocabulary, so
/// those two failure modes are not re-checked here.
fn estate_data_dir_override(
    config_path: &Path,
    root: &Path,
) -> Result<Option<PathBuf>, EstateError> {
    let Ok(text) = std::fs::read_to_string(config_path) else {
        return Ok(None);
    };
    let file = config_path.display().to_string();
    let value: toml::Value =
        toml::from_str(&text).map_err(|source| EstateError::Malformed { path: file, source })?;
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
    ///
    /// §6.1: mounts are derived, so a fixture that declares `[[repo]] name =
    /// "x"` needs a real checkout at `root/repos/x` for the strict loader to
    /// validate. Every `name = "..."` under a `[[repo]]` in `body` gets one,
    /// which keeps the fixtures about the *schema* question each test is
    /// really asking rather than about mount plumbing.
    fn parse(root: &Path, body: &str) -> Result<Estate, EstateError> {
        mount_declared_repos(root, body);
        parse_without_mounting(root, body)
    }

    /// [`parse`] without the mount scaffolding — for the tests whose whole
    /// subject *is* what the mounts look like on disk.
    fn parse_without_mounting(root: &Path, body: &str) -> Result<Estate, EstateError> {
        let config = root.join(MANIFEST_FILE);
        std::fs::write(&config, body).expect("sergeant.toml");
        Estate::from_config(&config)
    }

    /// Create `root/repos/<name>` as a real git repository for every
    /// `[[repo]]` `body` declares with a plain name.
    fn mount_declared_repos(root: &Path, body: &str) {
        let mut in_repo = false;
        for line in body.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_repo = line == "[[repo]]";
                continue;
            }
            if !in_repo {
                continue;
            }
            let Some(rest) = line.strip_prefix("name") else {
                continue;
            };
            let Some(value) = rest.trim_start().strip_prefix('=') else {
                continue;
            };
            let name = value.trim().trim_matches('"');
            if crate::domain::is_plain_name(name) {
                init_repo(&mount_path(root, name));
            }
        }
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
                &format!("[estate]\nname = \"w\"\n\n[[repo]]\nname = \"{name}\"\n"),
            )
            .expect_err("a traversing repository name must be refused");
            assert!(
                matches!(err, EstateError::InvalidRepositoryName { .. }),
                "{name:?} must be refused as a name, got {err}"
            );
        }

        // And the ordinary case still parses, so the guard is not refusing
        // everything.
        let estate = parse(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\n",
        )
        .expect("a plain name parses");
        assert_eq!(estate.repositories[0].name, "solo");
    }

    /// §6.1/§6.2: two names can no longer *be* one checkout — the mount is
    /// derived from the name, so `a` and `b` are `repos/a` and `repos/b` by
    /// construction. The way an operator could still try is a symlink, and
    /// that is refused as an alias: git reports the linked checkout's real
    /// top level, which is not the derived mount.
    ///
    /// This replaces the pre-Phase-D `DuplicateRepositoryPath` fixture
    /// (`path = "."` and `path = "./"`, one repository under two names). The
    /// defect it guarded — two bindings materializing onto the same
    /// `sergeant/<work-id>` branch of the same repository, the second `git
    /// worktree add -b` failing only after the first has already touched the
    /// user's checkout — is now unreachable through the schema, and reachable
    /// only through the filesystem, which is what this asserts against.
    #[test]
    fn a_symlinked_mount_is_refused_as_an_alias() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let real = mount_path(root, "a");
        init_repo(&real);
        // `repos/b` is a symlink to `repos/a`: one checkout, two names.
        std::os::unix::fs::symlink(&real, mount_path(root, "b")).expect("symlink the mount");

        let err = parse_without_mounting(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"a\"\n\n[[repo]]\nname = \"b\"\n",
        )
        .expect_err("a symlinked mount must be refused");
        let EstateError::RepositoryMountAliased {
            name,
            expected,
            actual,
            ..
        } = &err
        else {
            panic!("expected an alias refusal, got {err}");
        };
        assert_eq!(name, "b");
        assert!(
            expected.ends_with("repos/b"),
            "§15: name the expected derived path, got {expected}"
        );
        assert!(
            actual.ends_with("repos/a"),
            "§15: name the actual git top level, got {actual}"
        );
    }

    /// §6.2/§8.1 check 4: a **linked worktree** — a Work's own surface is the
    /// case that matters — is refused as a repository source. It owns none of
    /// the things a mount must own: its branches, common directory, worktree
    /// registry entries and `sergeant/*` refs all belong to the repository it
    /// was cut from.
    #[test]
    fn a_linked_worktree_is_refused_as_a_repository_source() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let source = dir.path().join("source-repo");
        init_repo(&source);
        std::fs::create_dir_all(root.join(REPOS_DIR)).expect("repos dir");
        let mount = mount_path(root, "borrowed");
        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                mount.to_str().expect("utf8 path"),
                "-b",
                "wt-branch",
            ])
            .current_dir(&source)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .expect("git worktree add");
        assert!(output.status.success(), "worktree add: {output:?}");

        let err = parse_without_mounting(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"borrowed\"\n",
        )
        .expect_err("a linked worktree must be refused as a mount");
        let EstateError::RepositoryMountIsLinkedWorktree {
            name, common_dir, ..
        } = &err
        else {
            panic!("expected a linked-worktree refusal, got {err}");
        };
        assert_eq!(name, "borrowed");
        assert!(
            common_dir.contains("source-repo"),
            "§15: name the actual git common dir, got {common_dir}"
        );
    }

    /// §6.1: `[[repo]] path` is removed, and a manifest that still declares
    /// one is refused by a message that names the removal — not
    /// `deny_unknown_fields`' generic "unknown field", which would say
    /// nothing about a key that was required until this release.
    #[test]
    fn a_repo_entry_still_declaring_path_is_refused_by_name() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(&mount_path(root, "api"));

        let err = parse_without_mounting(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"api\"\npath = \"repos/api\"\n",
        )
        .expect_err("a declared path must be refused");
        let EstateError::RepositoryPathDeclared { name, .. } = &err else {
            panic!("expected the named removal refusal, got {err}");
        };
        assert_eq!(name, "api");
        let message = err.to_string();
        assert!(
            message.contains("no longer exists") && message.contains("repos/<name>"),
            "the refusal must name the removal and the derived mount: {message}"
        );
        // Every read path refuses it, not just the strict one — the edit pen
        // and `sgt doctor`'s own reader included.
        let config = root.join(MANIFEST_FILE);
        assert!(matches!(
            Estate::from_config_structural(&config),
            Err(EstateError::RepositoryPathDeclared { .. })
        ));
        assert!(matches!(
            Estate::declared_repos(&config),
            Err(EstateError::RepositoryPathDeclared { .. })
        ));
        assert!(matches!(
            Estate::admit(root),
            Err(EstateRootError::Invalid { .. })
        ));
    }

    /// F9 (panel finding 6), mirroring the test above: a `[[knowledge]]` path
    /// that resolves inside a location the estate owns and mutates is refused
    /// **at the same station** — the manifest parse — by a named variant that
    /// says which location and why.
    ///
    /// Three owned families, and each is a different way to get the same
    /// wrong answer: a repository mount's bytes change every time a Work
    /// merges, a surface's change every time a Work runs, and the data
    /// directory is the daemon's own state. Indexing any of them would file
    /// the estate's own mutations as evidence about an outside world.
    #[test]
    fn a_knowledge_path_inside_an_estate_owned_location_is_refused_by_name() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(&mount_path(root, "api"));
        std::fs::create_dir_all(root.join("repos/api/docs")).expect("docs");
        std::fs::create_dir_all(root.join(".sergeant/data/surfaces/w1")).expect("surfaces");
        std::fs::create_dir_all(root.join("outside")).expect("outside");

        // 1. Inside a declared repository mount.
        let err = parse_without_mounting(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"api\"\n\n\
             [[knowledge]]\nname = \"docs\"\npath = \"repos/api/docs\"\n",
        )
        .expect_err("a mount-contained knowledge path must be refused");
        let EstateError::KnowledgePathInsideEstate { name, what, .. } = &err else {
            panic!("expected the named containment refusal, got {err}");
        };
        assert_eq!(name, "docs");
        assert!(
            what.contains("api"),
            "the refusal must name the mount: {what}"
        );
        let message = err.to_string();
        assert!(
            message.contains("read-only evidence") && message.contains("A1-03"),
            "the refusal must say why, not just that: {message}"
        );

        // 2. Inside the default data dir, which no manifest had to declare —
        //    the case a check written against declared values alone misses.
        let err = parse_without_mounting(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"api\"\n\n\
             [[knowledge]]\nname = \"state\"\npath = \".sergeant/data\"\n",
        )
        .expect_err("a data-dir-contained knowledge path must be refused");
        assert!(matches!(err, EstateError::KnowledgePathInsideEstate { .. }));

        // 3. Inside a surfaces directory, reached through a *symlink* — the
        //    containment check resolves before comparing, so a link is not a
        //    way around it.
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                root.join(".sergeant/data/surfaces/w1"),
                root.join("linked-surface"),
            )
            .expect("symlink");
            let err = parse_without_mounting(
                root,
                "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"api\"\n\n\
                 [[knowledge]]\nname = \"live\"\npath = \"linked-surface\"\n",
            )
            .expect_err("a symlink into a surface must be refused");
            assert!(matches!(err, EstateError::KnowledgePathInsideEstate { .. }));
        }

        // The negative control, and it matters as much as the refusals: an
        // ordinary directory under the estate root that the estate does *not*
        // own is a perfectly good knowledge source. This rule refuses three
        // named locations, not "anywhere near the estate".
        let estate = parse_without_mounting(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"api\"\n\n\
             [[knowledge]]\nname = \"outside\"\npath = \"outside\"\nignore = [\"*.log\"]\n",
        )
        .expect("a path the estate does not own is accepted");
        assert_eq!(estate.knowledge.len(), 1);
        assert_eq!(estate.knowledge[0].name, "outside");
        assert!(estate.knowledge[0].path.is_absolute());
        assert_eq!(estate.knowledge[0].ignore, vec!["*.log".to_string()]);

        // Every read path refuses it, not just the strict one — and `admit`
        // refuses the estate outright, exactly as it does for a declared
        // `[[repo]] path`.
        std::fs::write(
            root.join(MANIFEST_FILE),
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"api\"\n\n\
             [[knowledge]]\nname = \"docs\"\npath = \"repos/api/docs\"\n",
        )
        .expect("write");
        assert!(matches!(
            Estate::from_config_structural(&root.join(MANIFEST_FILE)),
            Err(EstateError::KnowledgePathInsideEstate { .. })
        ));
        assert!(matches!(
            Estate::admit(root),
            Err(EstateRootError::Invalid { .. })
        ));
    }

    /// The rest of F9's schema-level rules, mirroring `[[repo]]`'s: a plain
    /// name, no duplicates, `deny_unknown_fields` on the entry, and a `path`
    /// that is required because a knowledge root is declared, never derived.
    #[test]
    fn knowledge_entries_follow_the_same_schema_discipline_as_repo_entries() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(&mount_path(root, "api"));
        let head = "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"api\"\n\n";

        assert!(matches!(
            parse_without_mounting(
                root,
                &format!("{head}[[knowledge]]\nname = \"../escape\"\npath = \"/tmp\"\n")
            ),
            Err(EstateError::InvalidKnowledgeName { .. })
        ));
        assert!(matches!(
            parse_without_mounting(
                root,
                &format!(
                    "{head}[[knowledge]]\nname = \"n\"\npath = \"/tmp/a\"\n\n\
                     [[knowledge]]\nname = \"n\"\npath = \"/tmp/b\"\n"
                )
            ),
            Err(EstateError::DuplicateKnowledge { .. })
        ));
        // A typo'd key is a refusal naming the line, not a silently ignored
        // instruction — the same fail-closed reading every other table gets.
        let err = parse_without_mounting(
            root,
            &format!("{head}[[knowledge]]\nname = \"n\"\npath = \"/tmp/a\"\nignores = []\n"),
        )
        .expect_err("unknown field must be refused");
        assert!(matches!(err, EstateError::Malformed { .. }), "{err}");
        assert!(matches!(
            parse_without_mounting(root, &format!("{head}[[knowledge]]\nname = \"n\"\n")),
            Err(EstateError::Malformed { .. })
        ));
    }

    /// #112: `[[repo]] upstream` parses, stays opaque, and reaches every
    /// reader — the strict loader's `repository_upstream` map and the
    /// diagnostic loader's [`DeclaredRepo`] alike. An entry that declares
    /// none has none: absence is never guessed at from `origin` or anything
    /// else.
    ///
    /// guard-map: dropping the field from either loader, or defaulting it to
    /// the origin, fails here. Mutation this kills: a reader that
    /// "helpfully" infers an upstream, which would make `sgt doctor` report
    /// drift against a URL nobody declared.
    #[test]
    fn a_declared_upstream_is_carried_opaquely_by_every_reader() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        // Deliberately not a forge URL: nothing anywhere parses this.
        let declared = "ssh://git@example.invalid:2222/team/api.git";
        let estate = parse(
            root,
            &format!(
                "[estate]\nname = \"w\"\n\n\
                 [[repo]]\nname = \"api\"\norigin = \"/somewhere/api\"\n\
                 upstream = \"{declared}\"\n\n\
                 [[repo]]\nname = \"web\"\n"
            ),
        )
        .expect("upstream parses");
        assert_eq!(estate.repository_upstream("api"), Some(declared));
        assert_eq!(estate.repository_upstream("web"), None);
        assert_eq!(estate.repository_upstream("nonexistent"), None);

        let declared_repos =
            Estate::declared_repos(&root.join(MANIFEST_FILE)).expect("declared_repos");
        assert_eq!(declared_repos[0].upstream.as_deref(), Some(declared));
        assert_eq!(declared_repos[1].upstream, None);
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
             [[repo]]\nname = \"same\"\n\n\
             [[repo]]\nname = \"same\"\n",
        )
        .expect_err("a repeated name must be refused");
        assert!(
            matches!(&err, EstateError::DuplicateRepository { name, .. } if name == "same"),
            "got {err}"
        );
    }

    /// The submit request's `repositories` selection is user input too, and a
    /// name repeated there produces two identical bindings — the same
    /// same-path collision, arriving through the API instead of the file.
    #[test]
    fn a_repository_named_twice_in_one_selection_is_refused() {
        let estate = Estate {
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
            knowledge: Vec::new(),
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
        };

        let selected = estate
            .select(&["web".to_string(), "api".to_string()])
            .expect("distinct names select");
        assert_eq!(
            selected.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            ["web", "api"]
        );

        let err = estate
            .select(&["api".to_string(), "api".to_string()])
            .expect_err("a repeated selection must be refused");
        assert!(err.contains("twice"), "got {err}");

        // An unknown name still names what does exist.
        let err = estate
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
        let estate = Estate {
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
            knowledge: Vec::new(),
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
        };

        let err = estate
            .select(&[])
            .expect_err("an empty selection must now be refused, not expanded to \"all\"");
        assert!(
            err.contains("payments") && err.contains("--all"),
            "the refusal should name the estate and the remedy vocabulary, got: {err}"
        );
    }

    /// `sergeant.toml` declaring no `[[repo]]` entries at all is
    /// refused rather than accepted as a estate with nothing to act on.
    #[test]
    fn a_workspace_config_with_no_repositories_is_refused() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let err = parse(root, "[estate]\nname = \"empty\"\n")
            .expect_err("no repositories at all must be refused");
        assert!(
            matches!(&err, EstateError::NoRepositories { file } if file.ends_with(MANIFEST_FILE)),
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
             [[repo]]\nname = \"solo\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"fake\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"claude\"\n",
        )
        .expect_err("a repeated profile name must be refused");
        assert!(
            matches!(&err, EstateError::DuplicateProfile { name, .. } if name == "same"),
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
             [[repo]]\nname = \"solo\"\n\n\
             [[profile]]\nname = \"reckless\"\nbackend = \"claude\"\n\
             [profile.options]\npermission_mode = \"yolo\"\n",
        )
        .expect_err("an unrecognized permission_mode must be refused");
        match &err {
            EstateError::InvalidPermissionMode {
                profile, source, ..
            } => {
                assert_eq!(profile, "reckless");
                assert_eq!(source.value, "yolo");
            }
            other => panic!("expected InvalidPermissionMode, got {other}"),
        }

        // The five vocabulary values, plus unspecified, all still parse.
        let estate = parse(
            root,
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"solo\"\n\n\
             [[profile]]\nname = \"careful\"\nbackend = \"claude\"\n\
             [profile.options]\npermission_mode = \"plan\"\n",
        )
        .expect("a listed permission_mode value parses");
        assert_eq!(
            estate.profiles[0]
                .permission_mode()
                .expect("validated at load")
                .map(|m| m.as_cli_value()),
            Some("plan")
        );
    }

    /// #262: a `network_access` value that is neither `"true"` nor `"false"`
    /// is refused at config load, mirroring `permission_mode` above — a typo
    /// here must fail loudly and immediately, not lazily at PREPARE.
    #[test]
    fn a_profile_with_an_unknown_network_access_is_refused_at_load() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_repo(root);

        let err = parse(
            root,
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"solo\"\n\n\
             [[profile]]\nname = \"reckless\"\nbackend = \"codex\"\n\
             [profile.options]\nnetwork_access = \"yes\"\n",
        )
        .expect_err("an unrecognized network_access must be refused");
        match &err {
            EstateError::InvalidNetworkAccess {
                profile, source, ..
            } => {
                assert_eq!(profile, "reckless");
                assert_eq!(source.value, "yes");
            }
            other => panic!("expected InvalidNetworkAccess, got {other}"),
        }

        // Both booleans, plus unspecified, all still parse.
        let estate = parse(
            root,
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"solo\"\n\n\
             [[profile]]\nname = \"careful\"\nbackend = \"codex\"\n\
             [profile.options]\nnetwork_access = \"true\"\n",
        )
        .expect("a listed network_access value parses");
        assert_eq!(
            estate.profiles[0]
                .network_access()
                .expect("validated at load"),
            Some(true)
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
            "[workspace]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\n",
        )
        .expect_err("[workspace] must be refused by name");
        match &err {
            EstateError::LegacyVocabulary {
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
            "[estate]\nname = \"w\"\n\n[[repository]]\nname = \"solo\"\n",
        )
        .expect_err("[[repository]] must be refused by name");
        match &err {
            EstateError::LegacyVocabulary {
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
            "[workspace]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\n",
        )
        .expect_err("a mix must still be refused");
        assert!(
            matches!(&err, EstateError::LegacyVocabulary { found, .. } if found == "workspace"),
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

        let estate = parse(
            root,
            "[estate]\nname = \"clean\"\n\n[[repo]]\nname = \"solo\"\n",
        )
        .expect("pure estate vocabulary must parse");
        assert_eq!(estate.name, "clean");
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

        let estate = parse(
            root,
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"api\"\n\n\
             [[repo]]\nname = \"web\"\n\n\
             [group.payments]\nrepos = [\"api\", \"web\"]\nbrief = \"both sides\"\n",
        )
        .expect("a group over declared repos parses");
        let group = estate.groups.get("payments").expect("group present");
        assert_eq!(group.repos, vec!["api".to_string(), "web".to_string()]);
        assert_eq!(group.brief.as_deref(), Some("both sides"));

        let err = parse(
            root,
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"api\"\n\n\
             [group.payments]\nrepos = [\"api\", \"ghost\"]\n",
        )
        .expect_err("an undeclared group member must be refused");
        match &err {
            EstateError::UnknownGroupMember { group, name, .. } => {
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

        let estate = parse(
            root,
            "[estate]\nname = \"w\"\n\n\
             [[repo]]\nname = \"unset\"\n\n\
             [[repo]]\nname = \"loud\"\ninstructions = \"local\"\n",
        )
        .expect("instructions parses");
        assert_eq!(
            estate.instruction_policy("unset"),
            InstructionPolicy::Suppress,
            "an unset instructions value must default to suppress, byte-identical to today"
        );
        assert_eq!(estate.instruction_policy("loud"), InstructionPolicy::Local);
        // A name the manifest never declared still resolves rather than
        // panicking — callers ask this for arbitrary selected repos.
        assert_eq!(
            estate.instruction_policy("nowhere"),
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

        let estate = parse(
            root,
            "[estate]\nname = \"w\"\nsurfaces_dir = \"../elsewhere-surfaces\"\n\n\
             [[repo]]\nname = \"solo\"\n",
        )
        .expect("relative surfaces_dir parses");
        assert_eq!(
            estate.surfaces_dir,
            Some(root.join("../elsewhere-surfaces"))
        );

        let absolute = dir.path().join("abs-surfaces");
        let estate = parse(
            root,
            &format!(
                "[estate]\nname = \"w\"\nsurfaces_dir = {:?}\n\n[[repo]]\nname = \"solo\"\n",
                absolute.to_string_lossy()
            ),
        )
        .expect("absolute surfaces_dir parses");
        assert_eq!(estate.surfaces_dir, Some(absolute));

        // Unset stays `None` — the daemon's own default is left in force.
        let estate = parse(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\n",
        )
        .expect("no surfaces_dir parses");
        assert_eq!(estate.surfaces_dir, None);
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

        let estate = parse(
            root,
            "[estate]\nname = \"w\"\ndata_dir = \"../elsewhere-data\"\n\n\
             [[repo]]\nname = \"solo\"\n",
        )
        .expect("relative data_dir parses");
        assert_eq!(estate.data_dir, Some(root.join("../elsewhere-data")));

        let absolute = dir.path().join("abs-data");
        let estate = parse(
            root,
            &format!(
                "[estate]\nname = \"w\"\ndata_dir = {:?}\n\n[[repo]]\nname = \"solo\"\n",
                absolute.to_string_lossy()
            ),
        )
        .expect("absolute data_dir parses");
        assert_eq!(estate.data_dir, Some(absolute));

        // Unset stays `None` — `resolve_data_dir`'s own default is left in
        // force.
        let estate = parse(
            root,
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"solo\"\n",
        )
        .expect("no data_dir parses");
        assert_eq!(estate.data_dir, None);
    }

    // ---- §4.1: exact-root admission (R-MVP1-12 superseded) ----------------

    /// A `sergeant.toml` under `root` with an `[estate]` table, declaring one
    /// repository at the derived `repos/<name>` mount (§6.1), in `root`'s own
    /// git repository.
    fn write_estate(root: &Path, name: &str) {
        init_repo(root);
        init_repo(&root.join("repos").join("solo"));
        std::fs::write(
            root.join(MANIFEST_FILE),
            format!("[estate]\nname = {name:?}\n\n[[repo]]\nname = \"solo\"\n"),
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

        let admitted = Estate::admit(&estate_root).expect("the exact root is admitted");
        assert_eq!(
            Some(admitted.path.clone()),
            std::fs::canonicalize(&estate_root).ok()
        );
        assert_eq!(admitted.manifest_path, admitted.path.join(MANIFEST_FILE));

        let estate = Estate::resolve(&estate_root).expect("the strict load agrees");
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
        let err = Estate::admit(&member)
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
            message.contains(&member.join(MANIFEST_FILE).display().to_string())
                || message.contains(
                    &std::fs::canonicalize(&member)
                        .unwrap_or_else(|_| member.clone())
                        .join(MANIFEST_FILE)
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
    /// longer a estate, it is "no estate here" — exact-root resolution has
    /// nothing to fall back *to*.
    // CONTRACT PIN (estate-root Phase D): zero-config git fallback is removed; a repo with no sergeant.toml anywhere above no longer resolves to a estate.
    #[test]
    fn a_plain_git_repository_with_no_manifest_is_no_longer_a_workspace() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("plain-repo");
        init_repo(&root);
        // No `sergeant.toml` anywhere in this fixture, at `root` or above it.

        let err = Estate::admit(&root).expect_err("there is no estate here");
        assert!(
            matches!(err, EstateRootError::NoEstate { .. }),
            "a git repository is not an estate: {err:?}"
        );
        assert!(
            Estate::resolve(&root).is_err(),
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
        std::fs::write(root.join(MANIFEST_FILE), "[[repo]]\nname = \"solo\"\n").expect("write");

        let err = Estate::admit(&root).expect_err("no [estate] table here");
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
        std::fs::write(inner.join(MANIFEST_FILE), "this is not [ toml").expect("write");

        let err = Estate::admit(&inner).expect_err("a malformed manifest must refuse");
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
            root.join(MANIFEST_FILE),
            "[workspace]\nname = \"legacy\"\n\n[[repository]]\nname = \"legacy\"\n",
        )
        .expect("legacy sergeant.toml");

        let err = Estate::admit(&root).expect_err("legacy vocabulary must refuse");
        let EstateRootError::Invalid { source, .. } = &err else {
            panic!("expected Invalid, got {err:?}");
        };
        assert!(
            matches!(**source, EstateError::LegacyVocabulary { .. }),
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
            root.join(MANIFEST_FILE),
            "[estate]\nname = \"payments\"\n\n[[repo]]\nname = \"solo\"\n\n\
             [group.everything]\nrepos = [\"ghost\"]\n",
        )
        .expect("write");

        let err = Estate::admit(&root).expect_err("an unknown group member is a schema defect");
        assert!(
            matches!(err, EstateRootError::Invalid { .. }),
            "got {err:?}"
        );
    }

    /// A declared repository that is not on disk yet does **not** make the
    /// estate inadmissible — that is a repository problem, not an
    /// estate-identity problem (the design capture's own wrongness contract:
    /// "a broken repo blocks works targeting it, not the estate"). The strict
    /// [`Estate::resolve`] still refuses it, which is the half that
    /// protects a Work from binding a repository that is not really there.
    #[test]
    fn a_declared_repo_missing_from_disk_does_not_make_the_estate_inadmissible() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("estate");
        std::fs::create_dir_all(&root).expect("estate dir");
        init_repo(&root);
        std::fs::write(
            root.join(MANIFEST_FILE),
            "[estate]\nname = \"payments\"\n\n[[repo]]\nname = \"ghost\"\n",
        )
        .expect("write");

        Estate::admit(&root).expect("admission is about estate identity, not repo presence");
        assert!(
            Estate::resolve(&root).is_err(),
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

        let err = Estate::admit(&member)
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

        let err = Estate::admit(&elsewhere)
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
        std::fs::write(inner.join(MANIFEST_FILE), "this is not [ toml").expect("write");
        let estate_root = std::fs::canonicalize(&estate_root).expect("canonical");

        let err = Estate::admit(&inner)
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

        let err = Estate::admit(&root).expect_err("no estate").via_flag();
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
            root.join(MANIFEST_FILE),
            "[estate]\nname = \"w\"\ndata_dir = \"custom-data\"\n\n\
             [[repo]]\nname = \"solo\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"fake\"\n\n\
             [[profile]]\nname = \"same\"\nbackend = \"fake\"\n",
        )
        .expect("write");

        assert!(Estate::is_estate_root(&root).expect("tolerant probe"));
        assert_eq!(
            Estate::root_data_dir_override(&root).expect("tolerant lookup"),
            Some(root.join("custom-data")),
            "an unrelated structural defect must not stop the data-dir lookup"
        );
        // ...and it never looks up. A descendant is simply not an estate root.
        assert!(!Estate::is_estate_root(&inner).expect("tolerant probe"));
        assert_eq!(
            Estate::root_data_dir_override(&inner).expect("tolerant lookup"),
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
    /// without a top-level `estate` or `repository` table.
    ///
    /// Scoped to files literally named `sergeant.toml`, not a bare string
    /// grep across every doc and note: several already-committed notes
    /// discuss the legacy vocabulary in prose while describing the
    /// migration itself, which is not the live-config leak this pin is
    /// about (sergeant-rs-workspace's knowledge/evidence/gauntlet/notes and sergeant-rs-workspace's knowledge/evidence/gauntlet/runs are historical
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
                } else if name == MANIFEST_FILE {
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

    // -------------------------------------------------------------
    // W3: `[estate] retention`
    // -------------------------------------------------------------

    /// An absent `retention` resolves to `None` on the parsed `Estate` — the
    /// daemon-side default ([`DEFAULT_RETENTION`]) is applied at
    /// `daemon::start_with`'s policy resolution, not here.
    #[test]
    fn retention_absent_resolves_to_the_default() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate = parse(
            dir.path(),
            "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"a\"\n",
        )
        .expect("parse");
        assert_eq!(estate.retention, None);
    }

    /// A declared retention at or above [`MIN_RETENTION`] is accepted and
    /// carried onto `Estate::retention` verbatim.
    #[test]
    fn retention_at_or_above_the_floor_is_accepted() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let estate = parse(
            dir.path(),
            "[estate]\nname = \"w\"\nretention = 64\n\n[[repo]]\nname = \"a\"\n",
        )
        .expect("parse");
        assert_eq!(estate.retention, Some(64));
    }

    /// N22: a declared retention below [`MIN_RETENTION`] is refused by name,
    /// at parse time — never silently clamped, never accepted and left for a
    /// destructive prune cycle to discover later.
    #[test]
    fn a_manifest_declaring_retention_below_the_floor_is_refused_by_name() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let err = parse(
            dir.path(),
            "[estate]\nname = \"w\"\nretention = 10\n\n[[repo]]\nname = \"a\"\n",
        )
        .expect_err("a below-floor retention must be refused");
        match &err {
            EstateError::RetentionBelowFloor {
                value,
                floor,
                default,
                ..
            } => {
                assert_eq!(*value, 10);
                assert_eq!(*floor, MIN_RETENTION);
                assert_eq!(*default, DEFAULT_RETENTION);
            }
            other => panic!("expected RetentionBelowFloor, got {other}"),
        }
        assert!(err.to_string().contains("below the minimum of 64"), "{err}");
    }

    /// The same floor is enforced by the structural parser
    /// (`from_config_structural`), which is what a pen edit's `validate`
    /// reparses through — so `sgt repo add`/`group add`/`remove` refuse to
    /// commit over a below-floor `retention` too (§1.4's pen-support claim).
    #[test]
    fn the_structural_parser_also_refuses_a_below_floor_retention() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let config = dir.path().join(MANIFEST_FILE);
        std::fs::write(
            &config,
            "[estate]\nname = \"w\"\nretention = 1\n\n[[repo]]\nname = \"a\"\n",
        )
        .expect("write manifest");
        let err = Estate::from_config_structural(&config)
            .expect_err("structural parse must also refuse a below-floor retention");
        assert!(matches!(
            err,
            EstateError::RetentionBelowFloor { value: 1, .. }
        ));
    }
}

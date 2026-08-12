//! The estate manifest's own read-modify-write pen (`docs/gauntlet/notes/
//! estate-manifest-design-2026-08-11.md`'s "three pens, one file": hand
//! edits, sgt's own CLI verbs, and the harness are all editing the same
//! checked-in `sergeant.toml`). This module is the second pen — `sgt init`,
//! `sgt repo add/remove`, `sgt group add/remove` — and it is a *validating*
//! writer (A4), never a blind text patcher: every edit is applied to a
//! format-preserving in-memory document (`toml_edit`, "comments are for the
//! human" — a hand-written comment elsewhere in the file survives an sgt
//! edit), then the *whole resulting file* is round-tripped through
//! [`Workspace::from_config_structural`] — the same fail-closed schema-level
//! parser every other reader of `sergeant.toml` uses, minus the on-disk
//! repository resolution (MVP-3 invariants finding MVP3-C1: an edit
//! validates the manifest it would produce, not the on-disk state of every
//! repository the estate has ever declared) — before anything touches disk.
//! An edit that would produce an invalid manifest is refused before the
//! real file is touched, with the parser's own named diagnostic.
//!
//! Concurrency: an advisory lock (`crate::runtime::fsutil`, the same
//! mechanism the journal uses) is held around sgt's own read-modify-write,
//! on a lock file *sibling* to `sergeant.toml` rather than the manifest
//! itself — this guards only against two concurrent `sgt` invocations
//! racing each other; a foreign editor (a hand edit, the harness) is
//! deliberately last-write-wins, per the design capture's own ruling.
//! **MVP-3 invariants finding MVP3-C6:** "guards" here means fails closed,
//! not serializes-and-both-succeed — `take_exclusive_lock` only waits out a
//! `WouldBlock` when the path is already held *by this same process*
//! (`fsutil.rs`'s `SELF_LOCKED`, there to make one process's own nested
//! acquires reentrant), so two real `sgt` processes racing this lock never
//! wait for each other: the loser's `try_lock` returns immediately and this
//! module reports [`ManifestError::Locked`] — a safe, fail-closed refusal
//! (no edit is ever lost), but a refusal, not a queue. Pinned cross-process
//! by `tests/m8_estate_cli.rs`'s
//! `two_real_sgt_processes_racing_repo_add_serialize_dont_tear`.
//!
//! The write itself is atomic: `crate::runtime::fsutil::write_atomic`
//! (temp file, `fsync`, rename over the destination, `fsync` the directory)
//! — a reader never observes a torn or partial manifest.
//!
//! R-NS-4 discipline: nothing in this module adds engine semantics. Every
//! function here either edits `sergeant.toml` (declarative topology,
//! §9's own boundary: "it never stores transient work state") or shells out
//! to `git` for the same populate/verify a hand-written `[[repo]]` entry
//! would need anyway — no journal event, no new API surface, no change to
//! how a bound Work executes.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};

use crate::domain::is_plain_name;
use crate::domain::workspace::{InstructionPolicy, WORKSPACE_FILE, Workspace, WorkspaceError};
use crate::runtime::fsutil::{create_dir_all_durable, take_exclusive_lock, write_atomic};
use crate::runtime::git::{git_clone, git_succeeds};

/// The `.gitignore` entries `sgt init` maintains (task's own naming): the
/// estate-local data dir, the populated repository mounts, and this
/// module's own scratch files. None belong in the estate's own history —
/// `.sergeant/data` is machine-local runtime state (journal, blobs,
/// projections), `repos/` is populated clones of *other* repositories' own
/// histories, `.sergeant.toml.lock` is [`ManifestLock`]'s advisory-lock
/// marker (never removed by design — see its own doc comment — so it
/// outlives every edit and would otherwise sit untracked at the estate root
/// forever), and `sergeant.toml.validate-*` is [`validate`]'s throwaway
/// probe file, removed on the happy path but left behind if the process
/// dies mid-validate (MVP-3 invariants finding MVP3-C3).
const GITIGNORE_ENTRIES: &[&str] = &[
    ".sergeant/data",
    "repos/",
    ".sergeant.toml.lock",
    "sergeant.toml.validate-*",
];

/// Where `sgt init` defaults the data dir, relative to the estate root
/// (U-R2, the design capture's `[estate] data_dir defaulting .sergeant/data`).
/// `src/cli.rs`'s `resolve_data_dir` joins this onto a discovered estate root
/// for the same reason it appears in `.gitignore` here: one literal, shared
/// by the two places that must agree on it.
pub const DEFAULT_ESTATE_DATA_DIR: &str = ".sergeant/data";

/// Failure editing the manifest. Every variant names the defect and, per the
/// design capture's "wrongness contract", carries what to do about it in its
/// `Display`.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// Could not read or write a file this operation needed.
    #[error("cannot access {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// `sergeant.toml` exists but is not parseable TOML at all — refused
    /// rather than blindly patched, since this module cannot know what a
    /// syntactically broken file was trying to say.
    #[error("{path} is not valid TOML, so it cannot be safely edited: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml_edit::TomlError,
    },
    /// Another `sgt` invocation holds the manifest's advisory lock.
    #[error(
        "another sgt command is already editing {path}; wait for it to finish and retry \
         (if nothing is actually running, delete {path} — it is just a lock marker, never \
         the manifest itself)"
    )]
    Locked { path: String },
    /// The edit, once applied, produced a manifest this build's own parser
    /// refuses — named by the wrapped [`WorkspaceError`], which already
    /// carries its own remedy.
    #[error("this edit would leave {path} invalid: {source}")]
    Invalid {
        path: String,
        #[source]
        source: Box<WorkspaceError>,
    },
    /// No `[estate]`-bearing `sergeant.toml` was found walking upward from
    /// `start` (mirrors R-MVP1-12, bounded at `$HOME`).
    #[error(
        "no estate found above {start} (bounded at $HOME) — run `sgt init` first, at the \
         directory that should become the estate root"
    )]
    NoEstate { start: String },
    /// A declared repository/group name is not a plain directory/table
    /// component (the same rule [`crate::domain::is_plain_name`] enforces
    /// for `[[repo]]` names at parse time — checked here too, before a
    /// clone or a table insert, so a bad name never reaches the filesystem).
    #[error("{name:?} is not a usable name (must be a single plain path component)")]
    InvalidName { name: String },
    /// `sgt repo add` for a name the manifest already declares.
    #[error(
        "repository {name:?} is already declared in {path}; remove it first if you want to \
         re-add it with different settings (`sgt repo remove {name}`)"
    )]
    RepoAlreadyDeclared { name: String, path: String },
    /// `sgt repo add`/`sgt repo remove`/`sgt group add` named a repository
    /// the manifest has not declared.
    #[error(
        "{name:?} is not a declared repository in {path} (declared: {available}); declare it \
         first with `sgt repo add {name} --origin <url>` (or, if it already exists on disk at \
         repos/{name}, `sgt repo add {name}` with no `--origin`)"
    )]
    RepoNotDeclared {
        name: String,
        path: String,
        available: String,
    },
    /// `sgt repo remove` for a repository still named by one or more groups
    /// — removing it would leave a `[group.*].repos` entry pointing at
    /// nothing, the same dangling-reference shape `[[repo]]` parsing itself
    /// already refuses for a hand-edited file.
    #[error(
        "{name:?} is still a member of group(s) {groups}; remove it from {} first \
         (`sgt group remove <group> {name}`), then retry `sgt repo remove {name}`",
        if groups.len() == 1 { "that group" } else { "those groups" }
    )]
    RepoInUseByGroups { name: String, groups: String },
    /// `sgt group remove <group>` for a group the manifest has not declared.
    #[error("{name:?} is not a declared group in {path} (declared: {available})")]
    GroupNotDeclared {
        name: String,
        path: String,
        available: String,
    },
    /// `sgt group remove <group> <repo>...` naming a repo that is not
    /// actually a member of that group.
    #[error("{repo:?} is not a member of group {group:?} (members: {members})")]
    NotAGroupMember {
        group: String,
        repo: String,
        members: String,
    },
    /// `sgt repo add --origin <url>` where `repos/<name>` already exists but
    /// is not a git repository — sgt's job is populate-or-verify, never
    /// silently adopting an arbitrary directory (design capture: "an
    /// existing entry need only be a git repo").
    #[error(
        "repos/{name} already exists at {path} but is not a git repository; move it aside or \
         remove it, then retry `sgt repo add {name}`"
    )]
    ExistingPathNotAGitRepository { name: String, path: String },
    /// `sgt repo add <name>` with no `--origin` and no existing directory at
    /// `repos/<name>` to verify — sgt populates or verifies, it never
    /// invents a repository from nothing.
    #[error(
        "repos/{name} does not exist yet ({path}) and no --origin was given to clone one; \
         either pass --origin <url> or create the directory yourself first, then retry \
         `sgt repo add {name}`"
    )]
    NoPathAndNoOrigin { name: String, path: String },
    /// `git clone` itself failed.
    #[error("cloning {origin:?} into {path} failed: {detail}")]
    CloneFailed {
        origin: String,
        path: String,
        detail: String,
    },
}

/// RAII guard on the manifest's advisory lock — released on drop, including
/// on an early return, so a refused edit never leaves the pen wedged.
struct ManifestLock {
    _file: std::fs::File,
}

impl ManifestLock {
    /// The lock file lives *beside* `sergeant.toml`, not in place of it: an
    /// atomic rename of the manifest itself must never be the thing two
    /// racing writers serialize on, since the rename is already atomic on
    /// its own — what needs serializing is the read-modify-write in between.
    fn path_for(root: &Path) -> PathBuf {
        root.join(".sergeant.toml.lock")
    }

    fn acquire(root: &Path) -> Result<Self, ManifestError> {
        let path = Self::path_for(root);
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .map_err(|source| ManifestError::Io {
                path: path.display().to_string(),
                source,
            })?;
        let acquired = take_exclusive_lock(&path, &file).map_err(|source| ManifestError::Io {
            path: path.display().to_string(),
            source,
        })?;
        if !acquired {
            return Err(ManifestError::Locked {
                path: path.display().to_string(),
            });
        }
        Ok(Self { _file: file })
    }
}

/// What `sgt init` actually changed — reported back so a re-run can say
/// "already initialized" instead of repeating the same claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitOutcome {
    /// `sergeant.toml` did not exist and was created.
    pub manifest_created: bool,
    /// `[estate]` was added to an existing `sergeant.toml` that lacked one
    /// (distinct from `manifest_created`: a member repo's own `sergeant.toml`
    /// with `[[repo]]` but no `[estate]` is a real, if unusual, starting
    /// point).
    pub estate_section_added: bool,
    /// `repos/` did not exist and was created.
    pub repos_dir_created: bool,
    /// `.gitignore` gained at least one of [`GITIGNORE_ENTRIES`].
    pub gitignore_updated: bool,
}

impl InitOutcome {
    /// Whether this run changed anything at all — the idempotency signal:
    /// a second `sgt init` on an already-initialized estate reports every
    /// field `false`.
    pub fn changed(&self) -> bool {
        self.manifest_created
            || self.estate_section_added
            || self.repos_dir_created
            || self.gitignore_updated
    }
}

/// Read `sergeant.toml` at `root` into an editable document, or an empty one
/// if it does not exist yet. Distinguishes "file absent" (fine — every
/// caller here is allowed to create it) from "file present but unreadable
/// for some other reason" (a real failure, not silently treated as absent).
fn read_document(manifest_path: &Path) -> Result<(DocumentMut, bool), ManifestError> {
    match std::fs::read_to_string(manifest_path) {
        Ok(text) => {
            let doc = text
                .parse::<DocumentMut>()
                .map_err(|source| ManifestError::Parse {
                    path: manifest_path.display().to_string(),
                    source,
                })?;
            Ok((doc, true))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok((DocumentMut::new(), false)),
        Err(source) => Err(ManifestError::Io {
            path: manifest_path.display().to_string(),
            source,
        }),
    }
}

/// Validate `doc` by writing it to a throwaway file beside the real manifest
/// and running the schema-level parser/validator every manifest edit shares
/// ([`Workspace::from_config_structural`]) — never the real path, so a
/// refused edit leaves nothing behind but its own tmp file, which is removed
/// either way. Returns the validated [`Workspace`] on success, so callers
/// that need it do not have to parse a third time.
///
/// Deliberately **not** [`Workspace::from_config_allow_empty`] (MVP-3
/// invariants finding MVP3-C1): that resolves every declared `[[repo]]`
/// through git and fails closed at the first one not present on disk, so an
/// estate with even one uncloned repository — including a freshly `git
/// clone`d one, since `sgt init` gitignores `repos/` — would refuse every
/// later edit here, not just one touching the missing repository. The
/// structural validator keeps every schema-level check (legacy vocabulary,
/// duplicate/invalid names, group membership, profile validity) and drops
/// only the on-disk resolution; a repository an edit itself populates or
/// verifies (`add_repo`'s `populate_or_verify`) is already checked directly
/// by that caller.
fn validate(root: &Path, doc: &DocumentMut) -> Result<Workspace, ManifestError> {
    let manifest_path = root.join(WORKSPACE_FILE);
    let probe_path = root.join(format!("sergeant.toml.validate-{}", ulid::Ulid::generate()));
    let text = doc.to_string();
    std::fs::write(&probe_path, &text).map_err(|source| ManifestError::Io {
        path: probe_path.display().to_string(),
        source,
    })?;
    let outcome = Workspace::from_config_structural(&probe_path);
    let _ = std::fs::remove_file(&probe_path);
    outcome.map_err(|source| ManifestError::Invalid {
        path: manifest_path.display().to_string(),
        source: Box::new(source),
    })
}

/// Commit a validated edit: atomic write of `doc`'s rendered text over the
/// real manifest path.
fn commit(root: &Path, doc: &DocumentMut) -> Result<(), ManifestError> {
    let manifest_path = root.join(WORKSPACE_FILE);
    write_atomic(&manifest_path, doc.to_string().as_bytes()).map_err(|source| ManifestError::Io {
        path: manifest_path.display().to_string(),
        source,
    })
}

/// `sgt init`: scaffold `[estate]` in `sergeant.toml`, create `repos/`, and
/// add `.gitignore`'s [`GITIGNORE_ENTRIES`] — idempotent (a second run on an
/// already-initialized estate changes nothing, per [`InitOutcome::changed`]).
///
/// `root` is the directory `sgt init` is *run from* — unlike every other
/// verb in this module, this does not walk upward looking for an existing
/// estate, because it is the command that creates one: it targets exactly
/// where it is invoked, the same way `git init` does.
pub fn init_estate(root: &Path, name: Option<&str>) -> Result<InitOutcome, ManifestError> {
    let manifest_path = root.join(WORKSPACE_FILE);
    let _lock = ManifestLock::acquire(root)?;

    let (mut doc, existed) = read_document(&manifest_path)?;
    let estate_section_added = if doc.contains_key("estate") {
        false
    } else {
        let mut table = Table::new();
        let default_name = name
            .map(str::to_string)
            .unwrap_or_else(|| estate_default_name(root));
        table.insert("name", value(default_name));
        table.set_implicit(false);
        doc.insert("estate", Item::Table(table));
        true
    };

    // Validate before writing anything: a legacy-vocabulary or otherwise
    // malformed existing file must be refused here, not papered over by an
    // `[estate]` table bolted onto something already broken.
    validate(root, &doc)?;
    if estate_section_added || !existed {
        commit(root, &doc)?;
    }

    let repos_dir = root.join("repos");
    let repos_dir_created = !repos_dir.exists();
    if repos_dir_created {
        create_dir_all_durable(&repos_dir).map_err(|source| ManifestError::Io {
            path: repos_dir.display().to_string(),
            source,
        })?;
    }

    let gitignore_updated = ensure_gitignore(root)?;

    Ok(InitOutcome {
        manifest_created: !existed,
        estate_section_added,
        repos_dir_created,
        gitignore_updated,
    })
}

/// The zero-config-style default estate name: the root directory's own name.
fn estate_default_name(root: &Path) -> String {
    std::fs::canonicalize(root)
        .unwrap_or_else(|_| root.to_path_buf())
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "estate".to_string())
}

/// Append any of [`GITIGNORE_ENTRIES`] missing from `root/.gitignore`
/// (creating the file if it does not exist). Idempotent: an entry already
/// present as its own line (exact match, ignoring surrounding whitespace) is
/// never duplicated.
fn ensure_gitignore(root: &Path) -> Result<bool, ManifestError> {
    let path = root.join(".gitignore");
    let existing = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
        Err(source) => {
            return Err(ManifestError::Io {
                path: path.display().to_string(),
                source,
            });
        }
    };
    let have: std::collections::BTreeSet<&str> = existing.lines().map(str::trim).collect();
    let missing: Vec<&str> = GITIGNORE_ENTRIES
        .iter()
        .copied()
        .filter(|entry| !have.contains(entry))
        .collect();
    if missing.is_empty() {
        return Ok(false);
    }
    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    for entry in missing {
        updated.push_str(entry);
        updated.push('\n');
    }
    write_atomic(&path, updated.as_bytes()).map_err(|source| ManifestError::Io {
        path: path.display().to_string(),
        source,
    })?;
    Ok(true)
}

/// `sgt repo add <name> [--origin <url>] [--instructions local|suppress]`:
/// populate-or-verify (design capture: "canonical clone locations are not
/// sergeant's business; an existing entry need only be a git repo"), then
/// declare `[[repo]]`.
///
/// - `origin` given, `repos/<name>` absent: `git clone <origin> repos/<name>`.
/// - `origin` given, `repos/<name>` present: verified as a git repo (its
///   remote is *not* checked against `origin` — that check belongs to git,
///   not to sgt, per the design capture).
/// - `origin` absent: `repos/<name>` must already exist and be a git repo.
pub fn add_repo(
    estate_root: &Path,
    name: &str,
    origin: Option<&str>,
    instructions: Option<InstructionPolicy>,
) -> Result<(), ManifestError> {
    if !is_plain_name(name) {
        return Err(ManifestError::InvalidName {
            name: name.to_string(),
        });
    }
    let manifest_path = estate_root.join(WORKSPACE_FILE);
    let _lock = ManifestLock::acquire(estate_root)?;
    let (mut doc, _existed) = read_document(&manifest_path)?;

    if repo_names(&doc).iter().any(|n| n == name) {
        return Err(ManifestError::RepoAlreadyDeclared {
            name: name.to_string(),
            path: manifest_path.display().to_string(),
        });
    }

    let repo_path = estate_root.join("repos").join(name);
    populate_or_verify(name, &repo_path, origin)?;

    let rel_path = format!("repos/{name}");
    let mut table = Table::new();
    table.insert("name", value(name));
    table.insert("path", value(&rel_path));
    if let Some(policy) = instructions {
        table.insert("instructions", value(policy.as_str()));
    }
    if let Some(origin) = origin {
        table.insert("origin", value(origin));
    }
    repo_array_mut(&mut doc).push(table);

    validate(estate_root, &doc)?;
    commit(estate_root, &doc)
}

/// Clone or verify `repo_path` per [`add_repo`]'s own rule.
fn populate_or_verify(
    name: &str,
    repo_path: &Path,
    origin: Option<&str>,
) -> Result<(), ManifestError> {
    if repo_path.exists() {
        if !git_succeeds(repo_path, &["rev-parse", "--show-toplevel"]) {
            return Err(ManifestError::ExistingPathNotAGitRepository {
                name: name.to_string(),
                path: repo_path.display().to_string(),
            });
        }
        return Ok(());
    }
    let Some(origin) = origin else {
        return Err(ManifestError::NoPathAndNoOrigin {
            name: name.to_string(),
            path: repo_path.display().to_string(),
        });
    };
    if let Some(parent) = repo_path.parent() {
        create_dir_all_durable(parent).map_err(|source| ManifestError::Io {
            path: parent.display().to_string(),
            source,
        })?;
    }
    let dest_path = repo_path
        .to_str()
        .ok_or_else(|| ManifestError::CloneFailed {
            origin: origin.to_string(),
            path: repo_path.display().to_string(),
            detail: "destination path is not valid UTF-8".to_string(),
        })?;
    match git_clone(repo_path.parent().unwrap_or(repo_path), origin, dest_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(ManifestError::CloneFailed {
            origin: origin.to_string(),
            path: repo_path.display().to_string(),
            detail: e.to_string(),
        }),
    }
}

/// `sgt repo remove <name>`: refuses (naming the group and the remedy) while
/// any group still lists `name` as a member — removing it would otherwise
/// leave a `[group.*].repos` entry the parser itself would refuse on the
/// very next read. Does **not** delete `repos/<name>` from disk: the clone
/// itself is not sergeant's canonical copy of anything (design capture,
/// same rule `add_repo`'s populate-or-verify follows), so undeclaring it is
/// a manifest edit, never a `rm -rf` of a working tree that may hold
/// uncommitted changes.
pub fn remove_repo(estate_root: &Path, name: &str) -> Result<(), ManifestError> {
    let manifest_path = estate_root.join(WORKSPACE_FILE);
    let _lock = ManifestLock::acquire(estate_root)?;
    let (mut doc, _existed) = read_document(&manifest_path)?;

    let declared = repo_names(&doc);
    if !declared.iter().any(|n| n == name) {
        return Err(ManifestError::RepoNotDeclared {
            name: name.to_string(),
            path: manifest_path.display().to_string(),
            available: declared.join(", "),
        });
    }
    let referencing_groups = groups_containing(&doc, name);
    if !referencing_groups.is_empty() {
        return Err(ManifestError::RepoInUseByGroups {
            name: name.to_string(),
            groups: referencing_groups.join(", "),
        });
    }

    let array = repo_array_mut(&mut doc);
    let index = (0..array.len()).find(|&i| {
        array
            .get(i)
            .and_then(|t| t.get("name"))
            .and_then(|v| v.as_str())
            == Some(name)
    });
    if let Some(index) = index {
        array.remove(index);
    }

    validate(estate_root, &doc)?;
    commit(estate_root, &doc)
}

/// `sgt group add <name> <repo>... [--brief <text>]`: mkdir-p create
/// semantics — creating an already-existing group adds any new members to
/// it (union, existing membership untouched) rather than erroring, and
/// re-adding a member already present is a no-op. Every member must already
/// be a declared `[[repo]]` (fail closed, naming which one is not and the
/// remedy).
pub fn add_group(
    estate_root: &Path,
    name: &str,
    repos: &[String],
    brief: Option<&str>,
) -> Result<(), ManifestError> {
    if !is_plain_name(name) {
        return Err(ManifestError::InvalidName {
            name: name.to_string(),
        });
    }
    let manifest_path = estate_root.join(WORKSPACE_FILE);
    let _lock = ManifestLock::acquire(estate_root)?;
    let (mut doc, _existed) = read_document(&manifest_path)?;

    let declared = repo_names(&doc);
    for repo in repos {
        if !declared.iter().any(|n| n == repo) {
            return Err(ManifestError::RepoNotDeclared {
                name: repo.clone(),
                path: manifest_path.display().to_string(),
                available: declared.join(", "),
            });
        }
    }

    let groups_table = groups_table_mut(&mut doc);
    let entry = groups_table
        .entry(name)
        .or_insert(Item::Table(Table::new()));
    let group_table = entry
        .as_table_mut()
        .expect("groups_table_mut only ever inserts Item::Table entries");
    group_table.set_implicit(false);

    let mut members: Vec<String> = group_table
        .get("repos")
        .and_then(|item| item.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    for repo in repos {
        if !members.contains(repo) {
            members.push(repo.clone());
        }
    }
    let mut array = toml_edit::Array::new();
    for member in &members {
        array.push(member.as_str());
    }
    group_table.insert("repos", Item::Value(array.into()));
    if let Some(brief) = brief {
        group_table.insert("brief", value(brief));
    }

    validate(estate_root, &doc)?;
    commit(estate_root, &doc)
}

/// `sgt group remove <name> [repo...]`: with no `repo` arguments, removes
/// the whole group; with one or more, removes just those members (each must
/// actually be a member — fail closed otherwise), leaving the rest.
pub fn remove_group(estate_root: &Path, name: &str, repos: &[String]) -> Result<(), ManifestError> {
    let manifest_path = estate_root.join(WORKSPACE_FILE);
    let _lock = ManifestLock::acquire(estate_root)?;
    let (mut doc, _existed) = read_document(&manifest_path)?;

    let groups_table = groups_table_mut(&mut doc);
    let declared: Vec<String> = groups_table.iter().map(|(k, _)| k.to_string()).collect();
    if !groups_table.contains_key(name) {
        return Err(ManifestError::GroupNotDeclared {
            name: name.to_string(),
            path: manifest_path.display().to_string(),
            available: declared.join(", "),
        });
    }

    if repos.is_empty() {
        groups_table.remove(name);
    } else {
        let group_table = groups_table
            .get_mut(name)
            .and_then(Item::as_table_mut)
            .expect("just checked contains_key(name)");
        let members: Vec<String> = group_table
            .get("repos")
            .and_then(|item| item.as_array())
            .map(|array| {
                array
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        for repo in repos {
            if !members.contains(repo) {
                return Err(ManifestError::NotAGroupMember {
                    group: name.to_string(),
                    repo: repo.clone(),
                    members: members.join(", "),
                });
            }
        }
        let remaining: Vec<&String> = members.iter().filter(|m| !repos.contains(m)).collect();
        let mut array = toml_edit::Array::new();
        for member in &remaining {
            array.push(member.as_str());
        }
        group_table.insert("repos", Item::Value(array.into()));
    }

    validate(estate_root, &doc)?;
    commit(estate_root, &doc)
}

/// The `[[repo]]` array of tables, creating it (as an explicit, non-inline
/// array of tables — the on-disk `[[repo]]` shape, not `repo = [...]`) if
/// absent.
fn repo_array_mut(doc: &mut DocumentMut) -> &mut ArrayOfTables {
    doc.entry("repo")
        .or_insert(Item::ArrayOfTables(ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .expect("repo_array_mut always installs Item::ArrayOfTables")
}

/// The `[group.*]` table, creating it if absent.
fn groups_table_mut(doc: &mut DocumentMut) -> &mut Table {
    let item = doc.entry("group").or_insert(Item::Table(Table::new()));
    if let Item::Table(table) = item {
        table.set_implicit(false);
    }
    item.as_table_mut()
        .expect("groups_table_mut always installs Item::Table")
}

/// Declared `[[repo]]` names, in file order — used by every op above that
/// needs to validate a name against what is already declared without a full
/// [`Workspace`] parse (which would resolve every path via `git`, work this
/// module's own callers have not necessarily earned yet mid-edit).
fn repo_names(doc: &DocumentMut) -> Vec<String> {
    doc.get("repo")
        .and_then(Item::as_array_of_tables)
        .map(|array| {
            array
                .iter()
                .filter_map(|table| table.get("name").and_then(Item::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Names of every `[group.<name>]` whose `repos` lists `repo`, in
/// declaration order — the remedy list for [`ManifestError::RepoInUseByGroups`].
fn groups_containing(doc: &DocumentMut, repo: &str) -> Vec<String> {
    let Some(groups) = doc.get("group").and_then(Item::as_table) else {
        return Vec::new();
    };
    groups
        .iter()
        .filter(|(_, item)| {
            item.as_table()
                .and_then(|t| t.get("repos"))
                .and_then(Item::as_array)
                .is_some_and(|array| array.iter().any(|v| v.as_str() == Some(repo)))
        })
        .map(|(name, _)| name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// A temp git repository with one commit — mirrors
    /// `domain::workspace::tests::init_repo`, duplicated rather than shared
    /// because that helper is private to its own `#[cfg(test)]` module.
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

    /// guard-map: pins `sgt init`'s idempotency requirement end to end —
    /// running it twice must not duplicate the `[estate]` table, must not
    /// re-append `.gitignore` lines, and the second run's `InitOutcome`
    /// reports nothing changed. Mutation this kills: dropping the
    /// `doc.contains_key("estate")` guard (always inserting a fresh table)
    /// or the `have.contains(entry)` guard in `ensure_gitignore` (always
    /// appending) — both would corrupt or duplicate on a second run.
    #[test]
    fn init_is_idempotent_on_a_second_run() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();

        let first = init_estate(root, Some("my-estate")).expect("first init");
        assert!(first.changed());
        assert!(first.manifest_created);
        assert!(first.estate_section_added);
        assert!(first.repos_dir_created);
        assert!(first.gitignore_updated);

        let manifest_after_first =
            std::fs::read_to_string(root.join(WORKSPACE_FILE)).expect("read manifest");
        let gitignore_after_first =
            std::fs::read_to_string(root.join(".gitignore")).expect("read gitignore");

        let second = init_estate(root, Some("different-name")).expect("second init");
        assert!(
            !second.changed(),
            "a second init on an already-initialized estate must change nothing, got {second:?}"
        );

        let manifest_after_second =
            std::fs::read_to_string(root.join(WORKSPACE_FILE)).expect("read manifest");
        let gitignore_after_second =
            std::fs::read_to_string(root.join(".gitignore")).expect("read gitignore");
        assert_eq!(
            manifest_after_first, manifest_after_second,
            "a re-run must not touch the estate name or duplicate the [estate] table"
        );
        assert_eq!(
            gitignore_after_first, gitignore_after_second,
            "a re-run must not duplicate .gitignore entries"
        );
        assert!(
            manifest_after_second.contains("my-estate"),
            "the first run's name must survive — a second init must not rename the estate"
        );

        let workspace = Workspace::from_config_allow_empty(&root.join(WORKSPACE_FILE))
            .expect("scaffolded manifest parses");
        assert_eq!(workspace.name, "my-estate");
    }

    /// guard-map: `sgt init` scaffolds every `.gitignore` entry
    /// `GITIGNORE_ENTRIES` specifies verbatim — the estate-local data dir,
    /// the populated repository mounts, and (MVP3-C3) this module's own
    /// lock marker and validate-probe pattern, so a freshly initialized
    /// estate's `git status` never shows sgt's own runtime scratch as
    /// untracked. Mutation this kills: a typo'd or missing entry in
    /// `GITIGNORE_ENTRIES`, or `ensure_gitignore` writing to the wrong path.
    #[test]
    fn init_writes_every_gitignore_entry() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        init_estate(dir.path(), None).expect("init");
        let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).expect("read");
        for entry in GITIGNORE_ENTRIES {
            assert!(
                gitignore.lines().any(|l| l.trim() == *entry),
                "{entry:?} missing from .gitignore: {gitignore:?}"
            );
        }
    }

    /// guard-map: `sgt init` preserves a hand-written comment elsewhere in an
    /// existing `sergeant.toml` — the format-preserving edit's whole reason
    /// to exist over a blind serde round-trip. Mutation this kills:
    /// replacing the `toml_edit` document with a serde `WorkspaceFile`
    /// round-trip (which drops comments).
    #[test]
    fn init_preserves_hand_written_comments_in_an_existing_manifest() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let other = root.join("repos").join("web");
        init_repo(&other);
        std::fs::write(
            root.join(WORKSPACE_FILE),
            "# a human wrote this note\n[[repo]]\nname = \"web\"\npath = \"repos/web\"\n",
        )
        .expect("write manifest with a comment, no [estate] yet");

        init_estate(root, Some("commented")).expect("init onto an existing manifest");

        let text = std::fs::read_to_string(root.join(WORKSPACE_FILE)).expect("read");
        assert!(
            text.contains("# a human wrote this note"),
            "a format-preserving edit must not drop an existing comment, got:\n{text}"
        );
        assert!(text.contains("[estate]"));
    }

    /// guard-map: `add_repo` refuses a name already declared, and refuses a
    /// name given with no `--origin` and no existing directory to verify —
    /// the "populate or verify, never invent" rule. Mutation this kills:
    /// dropping either refusal (the second, silently treating a missing dir
    /// with no origin as success, would leave a `[[repo]]` entry pointing
    /// at nothing — `validate` would then have to be relied on alone to
    /// catch it, which this test also confirms it does not need to, since
    /// the dedicated error fires first with a clearer message).
    #[test]
    fn add_repo_refuses_duplicate_names_and_unpopulatable_repos() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_estate(root, Some("e")).expect("init");

        let err = add_repo(root, "ghost", None, None).expect_err("no dir, no origin");
        assert!(
            matches!(err, ManifestError::NoPathAndNoOrigin { .. }),
            "got {err}"
        );

        let existing = root.join("repos").join("api");
        init_repo(&existing);
        add_repo(root, "api", None, None).expect("verify an existing git repo, no origin needed");

        let err = add_repo(root, "api", None, None).expect_err("already declared");
        assert!(
            matches!(err, ManifestError::RepoAlreadyDeclared { .. }),
            "got {err}"
        );
    }

    /// guard-map: `add_repo --origin` clones into `repos/<name>` when the
    /// directory does not yet exist, and the resulting manifest is valid and
    /// carries the origin. Mutation this kills: swapping the clone
    /// destination argument order (cloning into the estate root itself
    /// rather than `repos/<name>`), or dropping the `origin` field from the
    /// written `[[repo]]` table.
    #[test]
    fn add_repo_clones_from_origin_when_the_directory_is_absent() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("estate");
        std::fs::create_dir_all(&root).expect("estate dir");
        init_estate(&root, Some("e")).expect("init");

        let upstream = dir.path().join("upstream");
        init_repo(&upstream);

        add_repo(
            &root,
            "api",
            Some(upstream.to_str().expect("utf8")),
            Some(InstructionPolicy::Local),
        )
        .expect("clone from origin");

        assert!(root.join("repos").join("api").join(".git").exists());
        let workspace = Workspace::from_config(&root.join(WORKSPACE_FILE)).expect("parses");
        assert_eq!(workspace.repositories.len(), 1);
        assert_eq!(workspace.repositories[0].name, "api");
        assert_eq!(
            workspace.repository_origin("api"),
            Some(upstream.to_str().expect("utf8"))
        );
        assert_eq!(
            workspace.instruction_policy("api"),
            InstructionPolicy::Local
        );
    }

    /// `sgt repo add --origin <url>` passes a human-supplied string straight
    /// to `git clone`; git's `ext::` transport runs an arbitrary local
    /// command as part of "cloning" from it — a classic clone-URL RCE if the
    /// origin ever reaches the clone unrestricted. Proves the restriction
    /// actually holds by checking for the command's own side effect (a
    /// sentinel file) rather than just the returned error: `ext::` failing
    /// for an unrelated reason (e.g. the shell itself missing) would still
    /// leave this false-safe if the test only asserted `Err`.
    ///
    /// guard-map: dropping `git_clone`'s `GIT_ALLOW_PROTOCOL` restriction (or
    /// routing this call through the unrestricted `git()` helper again) lets
    /// the sentinel file get created, failing this test.
    #[test]
    fn add_repo_refuses_an_ext_transport_origin_without_running_it() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("estate");
        std::fs::create_dir_all(&root).expect("estate dir");
        init_estate(&root, Some("e")).expect("init");

        let sentinel = dir.path().join("pwned");
        let origin = format!("ext::sh -c \"touch {}\"", sentinel.display());

        let err = add_repo(&root, "api", Some(&origin), None).expect_err("ext:: must not clone");
        assert!(
            matches!(err, ManifestError::CloneFailed { .. }),
            "expected a CloneFailed refusal, got: {err:?}"
        );
        assert!(
            !sentinel.exists(),
            "the ext:: transport's command must never have run: {}",
            sentinel.display()
        );
    }

    /// An origin beginning with `-` must be read as a literal (failing
    /// clone) rather than a `git clone` flag — proven by using a real flag
    /// name (`--upload-pack`) that, if parsed as a flag instead of a
    /// positional, would produce git's "too many arguments"/usage-text
    /// failure instead of an ordinary "not a valid origin" clone failure.
    ///
    /// guard-map: dropping the `--` separator in `git_clone` makes this
    /// origin get parsed as a flag, changing the failure shape (and, for a
    /// differently-chosen real flag, could change behavior entirely rather
    /// than just failing).
    #[test]
    fn add_repo_treats_a_dash_prefixed_origin_as_a_literal_not_a_flag() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().join("estate");
        std::fs::create_dir_all(&root).expect("estate dir");
        init_estate(&root, Some("e")).expect("init");

        let err = add_repo(&root, "api", Some("--upload-pack=x"), None)
            .expect_err("a dash-prefixed origin must not clone");
        let ManifestError::CloneFailed { detail, .. } = err else {
            panic!("expected a CloneFailed refusal, got: {err:?}");
        };
        assert!(
            !detail.contains("usage: git clone") && !detail.contains("Too many arguments"),
            "the origin must never reach git's flag parser as a flag: {detail}"
        );
    }

    /// guard-map: `remove_repo` refuses while a group still references the
    /// repository, naming the group — and succeeds, actually dropping the
    /// `[[repo]]` entry, once the group no longer does. Mutation this kills:
    /// dropping the `groups_containing` check (would silently orphan the
    /// group's reference) or `remove_repo` removing the wrong array index.
    #[test]
    fn remove_repo_refuses_while_a_group_still_references_it() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_estate(root, Some("e")).expect("init");
        let a = root.join("repos").join("a");
        let b = root.join("repos").join("b");
        init_repo(&a);
        init_repo(&b);
        add_repo(root, "a", None, None).expect("add a");
        add_repo(root, "b", None, None).expect("add b");
        add_group(root, "pair", &["a".to_string(), "b".to_string()], None).expect("add group");

        let err = remove_repo(root, "a").expect_err("still a group member");
        match &err {
            ManifestError::RepoInUseByGroups { name, groups } => {
                assert_eq!(name, "a");
                assert_eq!(groups, "pair");
            }
            other => panic!("expected RepoInUseByGroups, got {other}"),
        }

        remove_group(root, "pair", &["a".to_string()]).expect("drop a from the group");
        remove_repo(root, "a").expect("now removable");

        let workspace = Workspace::from_config(&root.join(WORKSPACE_FILE)).expect("parses");
        assert_eq!(
            workspace
                .repositories
                .iter()
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>(),
            vec!["b"]
        );
        assert_eq!(workspace.groups["pair"].repos, vec!["b".to_string()]);
    }

    /// guard-map: `add_group` refuses an undeclared member, naming it and
    /// what *is* declared, before touching the file. Mutation this kills:
    /// checking membership against a stale/empty list (e.g. checking against
    /// `Vec::new()` unconditionally) rather than the manifest's actual
    /// declared repos.
    #[test]
    fn add_group_refuses_an_undeclared_member_with_a_named_remedy() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_estate(root, Some("e")).expect("init");
        let a = root.join("repos").join("a");
        init_repo(&a);
        add_repo(root, "a", None, None).expect("add a");

        let err = add_group(root, "pair", &["a".to_string(), "ghost".to_string()], None)
            .expect_err("ghost is not declared");
        match &err {
            ManifestError::RepoNotDeclared {
                name, available, ..
            } => {
                assert_eq!(name, "ghost");
                assert_eq!(available, "a");
            }
            other => panic!("expected RepoNotDeclared, got {other}"),
        }
        // Refused before anything was written — the manifest still has no
        // group at all.
        let workspace = Workspace::from_config(&root.join(WORKSPACE_FILE)).expect("parses");
        assert!(workspace.groups.is_empty());
    }

    /// guard-map: `add_group`'s mkdir-p semantics — re-adding an existing
    /// group with an overlapping-plus-new member set unions rather than
    /// erroring or duplicating, and re-adding the exact same members twice
    /// is a clean no-op on membership.
    #[test]
    fn add_group_is_mkdir_p_idempotent_and_unions_members() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_estate(root, Some("e")).expect("init");
        for name in ["a", "b", "c"] {
            init_repo(&root.join("repos").join(name));
            add_repo(root, name, None, None).expect("add");
        }

        add_group(root, "trio", &["a".to_string()], Some("first")).expect("create group");
        add_group(root, "trio", &["a".to_string(), "b".to_string()], None).expect("union");
        add_group(root, "trio", &["c".to_string()], None).expect("union again");

        let workspace = Workspace::from_config(&root.join(WORKSPACE_FILE)).expect("parses");
        let group = &workspace.groups["trio"];
        let mut members = group.repos.clone();
        members.sort();
        assert_eq!(
            members,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        // The brief from the first call survives a later call that omits it.
        assert_eq!(group.brief.as_deref(), Some("first"));
    }

    /// guard-map: `remove_group` with no repo arguments deletes the whole
    /// group; naming a repo that is not actually a member is refused rather
    /// than silently ignored. Mutation this kills: `remove_group` treating
    /// an empty `repos` slice as "remove nothing" instead of "remove the
    /// whole group" (the two would be indistinguishable from the caller's
    /// perspective if this flipped).
    #[test]
    fn remove_group_whole_group_vs_named_members() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_estate(root, Some("e")).expect("init");
        for name in ["a", "b"] {
            init_repo(&root.join("repos").join(name));
            add_repo(root, name, None, None).expect("add");
        }
        add_group(root, "pair", &["a".to_string(), "b".to_string()], None).expect("add group");

        let err =
            remove_group(root, "pair", &["ghost".to_string()]).expect_err("ghost is not a member");
        assert!(
            matches!(err, ManifestError::NotAGroupMember { .. }),
            "got {err}"
        );

        remove_group(root, "pair", &[]).expect("remove the whole group");
        let workspace = Workspace::from_config(&root.join(WORKSPACE_FILE)).expect("parses");
        assert!(!workspace.groups.contains_key("pair"));

        let err = remove_group(root, "pair", &[]).expect_err("already gone");
        assert!(
            matches!(err, ManifestError::GroupNotDeclared { .. }),
            "got {err}"
        );
    }

    /// guard-map: two concurrent editors **in this one process** serialize
    /// rather than tearing each other's write — the advisory lock actually
    /// does something. Mutation this kills: dropping `ManifestLock::acquire`
    /// from any editing function (both adds would then race the read-modify-
    /// write and one repo could silently vanish from the final file).
    ///
    /// This is the in-process half only (MVP-3 invariants finding MVP3-C6):
    /// `take_exclusive_lock`'s wait-and-retry path is taken here because the
    /// second thread finds the path already in this process's own
    /// `SELF_LOCKED` set, which is exactly what lets both adds succeed. Two
    /// real `sgt` processes never share that set, so the loser there fails
    /// closed instead of waiting — see
    /// `tests/m8_estate_cli.rs`'s `two_real_sgt_processes_racing_repo_add_
    /// serialize_dont_tear` for that half.
    #[test]
    fn concurrent_repo_adds_do_not_lose_an_entry() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path().to_path_buf();
        init_estate(&root, Some("e")).expect("init");
        for name in ["a", "b"] {
            init_repo(&root.join("repos").join(name));
        }

        let root_a = root.clone();
        let handle = std::thread::spawn(move || add_repo(&root_a, "a", None, None));
        let result_b = add_repo(&root, "b", None, None);
        let result_a = handle.join().expect("thread");

        result_a.expect("a added");
        result_b.expect("b added");

        let workspace = Workspace::from_config(&root.join(WORKSPACE_FILE)).expect("parses");
        let mut names: Vec<&str> = workspace
            .repositories
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    /// MVP-3 invariants finding MVP3-C1: a declared repository missing from
    /// disk — the exact shape a freshly `git clone`d estate is in, since
    /// `sgt init` gitignores `repos/` — must not block an edit that never
    /// touches it. `add_repo`, `remove_repo`, `add_group` and `remove_group`
    /// are each proven against the same setup: two declared repos, one of
    /// which (`a`) is then deleted from disk to simulate the clone-without-
    /// `repos/` state, and every edit that only concerns `b` still succeeds.
    ///
    /// guard-map: reverting `validate` to
    /// `Workspace::from_config_allow_empty` (the strict, on-disk-resolving
    /// parser) makes every assertion below fail with `ManifestError::Invalid`
    /// wrapping `WorkspaceError::RepositoryNotFound { name: "a", .. }`.
    #[test]
    fn a_missing_unrelated_repo_does_not_block_edits_that_do_not_touch_it() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        init_estate(root, Some("e")).expect("init");
        for name in ["a", "b"] {
            init_repo(&root.join("repos").join(name));
        }
        add_repo(root, "a", None, None).expect("declare a");
        add_repo(root, "b", None, None).expect("declare b");

        // Simulate the clone-is-distro shape: `a`'s working copy is gone,
        // but it is still declared.
        std::fs::remove_dir_all(root.join("repos").join("a")).expect("remove a's checkout");

        // `add_repo` for an unrelated new repo `c` must still succeed.
        init_repo(&root.join("repos").join("c"));
        add_repo(root, "c", None, None)
            .expect("add_repo must not require every OTHER declared repo to resolve on disk");

        // `add_group`/`remove_group` naming only `b`/`c` must still succeed.
        add_group(root, "pair", &["b".to_string(), "c".to_string()], None)
            .expect("add_group must not require every OTHER declared repo to resolve on disk");
        remove_group(root, "pair", &[])
            .expect("remove_group must not require every OTHER declared repo to resolve on disk");

        // `remove_repo` for the untouched `b` must still succeed.
        remove_repo(root, "b")
            .expect("remove_repo must not require every OTHER declared repo to resolve on disk");

        // `a` is still declared (never removed) and the manifest is still
        // structurally valid — proving the above edits really did land.
        let declared =
            Workspace::declared_repos(&root.join(WORKSPACE_FILE)).expect("declared_repos");
        let mut names: Vec<&str> = declared.iter().map(|r| r.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["a", "c"]);
    }
}

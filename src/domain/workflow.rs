//! Workflow: procedure as versioned filesystem content (proposal §12).
//!
//! §12 moves procedure out of runtime code. A workflow is a directory:
//!
//! ```text
//! .sergeant/workflows/software-change/
//!   workflow.toml        machine-readable: name, version, stage order
//!   00-prepare/CONTEXT.md   actor-readable: what this stage is for
//!   10-implement/CONTEXT.md
//!   ...
//! ```
//!
//! The engine supports §12's verb set and nothing more — ordered stages,
//! explicit entry, explicit completion, waiting, needs input, blocked, retry,
//! failure, cancellation. It is not a DAG scheduler (§4).
//!
//! **A run pins its workflow.** The whole resolved definition, stage contexts
//! included, is journaled when the work binds to it (`workflow.bound`), so
//! editing the files mid-run cannot retroactively change a running work and a
//! replay years later reconstructs the procedure that actually executed. That
//! is what "versioned filesystem content" has to mean for an event-sourced
//! runtime: the filesystem is the source, the journal is the version.
//!
//! Stage state is journal events and is **orthogonal to Work state** (§10):
//! `Work.state` is one of the eight §10 values, the current stage is a
//! separate coordinate, and neither is derived from the other.
//!
//! **Tagged stage definitions** (proposal §12, N3 Outcome 2). A legacy
//! `workflow.toml` — bare `[workflow]` with a `stages` list — still parses
//! identically: every stage defaults to an actor stage with no explicit
//! harness/profile (§12.1). A workflow may additionally declare optional
//! `[stage."<id>"]` tables keyed by stage id, carrying `kind` (only `"actor"`
//! is legal this milestone; N4 adds `"execute"`, §11.2), and the actor-only
//! `harness`/`profile` fields (§12.2). Unknown kinds, unknown fields, and a
//! table naming a stage the `stages` list never declared all fail closed at
//! load time (§22.3) rather than being silently ignored — this module never
//! guesses at what a workflow author meant. The resolved
//! [`WorkflowDefinition`] carries a [`WorkflowDefinition::content_hash`]:
//! a stable identity over every execution-relevant field (descriptor, stage
//! order, per-stage executor tag, contexts), so a bound run's pinned
//! procedure can be told apart from a same-named workflow whose content later
//! changed.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::is_plain_name;

/// Workflow directory inside a repository (D1: `.depot/` upstream).
pub const WORKFLOW_ROOT: &str = ".sergeant/workflows";
/// Machine-readable workflow descriptor inside a workflow directory.
pub const WORKFLOW_FILE: &str = "workflow.toml";
/// Actor-readable stage context file inside a stage directory.
pub const CONTEXT_FILE: &str = "CONTEXT.md";
/// Name of the built-in workflow used when a workspace ships none.
pub const DEFAULT_WORKFLOW: &str = "software-change";
/// Source marker for the embedded built-in workflow.
pub const SOURCE_EMBEDDED: &str = "embedded";

/// Event kind: a work bound to a resolved workflow definition.
pub const KIND_WORKFLOW_BOUND: &str = "workflow.bound";
/// Event kind: a stage was explicitly entered.
pub const KIND_STAGE_ENTERED: &str = "stage.entered";
/// Event kind: a stage was explicitly completed (§25: only ever from an
/// explicit backend or API signal, never inferred from process liveness).
pub const KIND_STAGE_COMPLETED: &str = "stage.completed";
/// Event kind: a stage is waiting on an external condition.
pub const KIND_STAGE_WAITING: &str = "stage.waiting";
/// Event kind: a stage needs human input.
pub const KIND_STAGE_NEEDS_INPUT: &str = "stage.needs_input";
/// Event kind: input for a waiting stage arrived.
pub const KIND_STAGE_INPUT_RECEIVED: &str = "stage.input_received";
/// Event kind: a stage that was `needs_input` is live again — its delivery
/// (`stage.input_received`'s SEND) reached the backend and the stage attempt
/// resumed under the same execution, no new attempt (BS2 / issue #46's
/// second seam). Committed only after the out-of-lock delivery settles, so
/// the completion driver's `due_observations` (which requires
/// `StageStatus::Active`) never has a window to observe a delivery still in
/// flight — see `Engine::settle_send`.
pub const KIND_STAGE_RESUMED: &str = "stage.resumed";
/// Event kind: a stage is blocked.
pub const KIND_STAGE_BLOCKED: &str = "stage.blocked";
/// Event kind: a stage failed.
pub const KIND_STAGE_FAILED: &str = "stage.failed";
/// Event kind: a stage was canceled.
pub const KIND_STAGE_CANCELED: &str = "stage.canceled";

/// What kind of executor performs a stage (§11, §12.2, §13.1).
///
/// `Actor` is the only legal kind this milestone (N3 Outcome 2). `Execute`
/// (a declared container image run through Docker, §11.2) is N4 scope; a
/// `[stage."<id>"]` table naming any other kind fails closed at load time
/// rather than being accepted and ignored (§22.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StageKind {
    /// A native reasoning harness performs this stage (§11.1).
    #[default]
    Actor,
}

impl StageKind {
    /// The kind's canonical snake_case name, as it appears in `workflow.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            StageKind::Actor => "actor",
        }
    }
}

/// One stage: an ordered directory with actor-readable context, tagged with
/// the executor that performs it (§12.2, §13.1).
///
/// `kind`/`harness`/`profile` are additive over the legacy shape: a stage
/// with no `[stage."<id>"]` table gets `kind: Actor`, `harness: None`,
/// `profile: None` — the same actor-default semantics §12.1 describes for a
/// bare `workflow.toml`. `#[serde(default)]` on the tagged fields also lets a
/// `workflow.bound` payload journaled before N3 replay cleanly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageDefinition {
    /// Directory name, which is also the stage id (`10-implement`).
    pub id: String,
    /// The stage's `CONTEXT.md`, carried verbatim.
    pub context: String,
    /// The executor kind. Always `Actor` this milestone.
    #[serde(default)]
    pub kind: StageKind,
    /// Explicit stage-level harness (§12.2). `None` means "use the Work
    /// actor default" (§12.5) — resolved by the router, not here.
    #[serde(default)]
    pub harness: Option<String>,
    /// Explicit stage-level launch profile (§12.2). `None` means "use the
    /// Work/profile default".
    #[serde(default)]
    pub profile: Option<String>,
    /// R-MVP1-11: this stage needs the actor to be able to author a
    /// question that parks its own stage — [`crate::backend::Capabilities::ask`].
    /// `false` (the default) is silent for every workflow that never
    /// declares it, exactly like `kind`/`harness`/`profile` above. It says
    /// what the stage *needs*, never that the engine converses (R-NS-6):
    /// the engine reads it only to refuse a submission whose resolved
    /// backend cannot honour it (§17.5's preflight, `Engine::bind_stages`).
    #[serde(default)]
    pub requires_ask: bool,
}

/// The executor decision for **one stage of one run**, resolved at plan time
/// and pinned in `workflow.bound` (§12.5, §13.3, §17.5).
///
/// Why it is pinned rather than re-derived at each stage entry: §22.4 requires
/// that "retry uses the same pinned stage harness/profile/model decision" and
/// that "restart reconstructs the same decision from the journal". A decision
/// re-derived at entry time would be re-derived against whatever the registry,
/// the workspace's `sergeant.toml` and the harness probes say *then* — so a
/// retry after an operator edited a profile, or a restart after a harness went
/// unavailable, would silently run a different execution than the one the run
/// was admitted with. Deciding once, before the Work exists, and journaling the
/// answer is what makes §17.5's "reject before Work or worktree side effects"
/// and §12.5's "no silent substitution" the same mechanism.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageBinding {
    /// The stage this decision belongs to.
    pub stage_id: String,
    /// Its position in the workflow's stage order.
    pub index: usize,
    /// The executor kind (always `actor` this milestone).
    #[serde(default)]
    pub kind: StageKind,
    /// The harness this stage runs on: a registered, probed-available backend.
    pub harness: String,
    /// Which §12.5 tier decided it (`stage_harness` / `work_actor_default`).
    pub route_source: String,
    /// The launch profile pinned for this stage, if any (§14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<crate::domain::profile::Profile>,
}

/// A resolved workflow: ordered stages plus the identity of what produced it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Workflow name (`software-change`).
    pub name: String,
    /// Version declared in `workflow.toml`.
    pub version: String,
    /// Where this definition came from: [`SOURCE_EMBEDDED`] or a directory
    /// path. Recorded so a replay can tell a built-in run from a repo's own.
    pub source: String,
    /// Stages in execution order.
    pub stages: Vec<StageDefinition>,
    /// Stable content-identity hash over every execution-relevant field:
    /// descriptor (name, version), stage order, each stage's executor tag
    /// (kind/harness/profile), and every context verbatim (§22.3). Deliberately
    /// excludes `source`, which is provenance, not procedure: the same
    /// content loaded from two different paths is the same workflow.
    /// Hex-encoded BLAKE3 over a canonical JSON projection of the fields
    /// above.
    #[serde(default)]
    pub content_hash: String,
}

/// Per-stage lifecycle status. Orthogonal to [`crate::domain::work::WorkState`]
/// by construction: this type describes one attempt at one stage, that one
/// describes the durable unit of intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    /// Entered and running.
    Active,
    /// Parked on an external condition.
    Waiting,
    /// Parked on a human answer.
    NeedsInput,
    /// Parked on a decision or gate.
    Blocked,
    /// Finished successfully.
    Completed,
    /// Finished unsuccessfully (retryable).
    Failed,
    /// Abandoned because the work was canceled.
    Canceled,
}

impl StageStatus {
    /// The status's canonical snake_case name.
    pub fn as_str(self) -> &'static str {
        match self {
            StageStatus::Active => "active",
            StageStatus::Waiting => "waiting",
            StageStatus::NeedsInput => "needs_input",
            StageStatus::Blocked => "blocked",
            StageStatus::Completed => "completed",
            StageStatus::Failed => "failed",
            StageStatus::Canceled => "canceled",
        }
    }
}

/// One recorded attempt at one stage, folded from the journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageRecord {
    /// Stage id.
    pub stage_id: String,
    /// Position in the workflow's stage order.
    pub index: usize,
    /// 1-based attempt number (retry increments it).
    pub attempt: u32,
    /// Current status of this attempt.
    pub status: StageStatus,
    /// Reason, prompt or summary carried by the last status change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Failure loading a workflow.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// No workflow directory with this name, and it is not the built-in one.
    #[error("workflow {name:?} not found (looked in {searched})")]
    NotFound {
        /// Requested workflow name.
        name: String,
        /// Directory that was searched.
        searched: String,
    },
    /// A workflow file could not be read.
    #[error("cannot read {path}: {source}")]
    Io {
        /// Path that failed to read.
        path: String,
        /// Underlying I/O failure.
        source: std::io::Error,
    },
    /// `workflow.toml` is not valid, or declares unknown fields.
    #[error("invalid {path}: {source}")]
    Malformed {
        /// Path of the offending file.
        path: String,
        /// Parse failure with line and column.
        source: toml::de::Error,
    },
    /// The declared name does not match the directory it lives in.
    #[error("{path} declares workflow name {declared:?} but lives in directory {directory:?}")]
    NameMismatch {
        /// Path of the descriptor.
        path: String,
        /// Name the file declares.
        declared: String,
        /// Directory name it was loaded from.
        directory: String,
    },
    /// A stage id is unusable as a directory name inside the workflow.
    #[error("{path} declares stage id {stage:?}, which is not a plain directory name")]
    InvalidStageId {
        /// Path of the descriptor.
        path: String,
        /// The offending stage id.
        stage: String,
    },
    /// A declared stage has no directory or no `CONTEXT.md`.
    #[error("{path} declares stage {stage:?}, but {missing} does not exist")]
    MissingStage {
        /// Path of the descriptor.
        path: String,
        /// The declared stage id.
        stage: String,
        /// The path that is missing.
        missing: String,
    },
    /// The same stage id appears twice.
    #[error("{path} declares stage {stage:?} twice")]
    DuplicateStage {
        /// Path of the descriptor.
        path: String,
        /// The repeated stage id.
        stage: String,
    },
    /// A `[stage."<id>"]` table names a `kind` this milestone does not
    /// support. `"actor"` is the only legal kind (N3 Outcome 2, §12.2).
    #[error(
        "{path} declares stage {stage:?} with unknown kind {kind:?} (only \"actor\" is supported)"
    )]
    UnknownStageKind {
        /// Path of the descriptor.
        path: String,
        /// The offending stage id.
        stage: String,
        /// The unsupported `kind` value.
        kind: String,
    },
    /// A `[stage."<id>"]` table names a stage id the `stages` list never
    /// declared. Metadata for a stage that does not exist is refused rather
    /// than silently ignored.
    #[error(
        "{path} declares a [stage.{stage:?}] table, but {stage:?} is not in this workflow's stage order"
    )]
    UndeclaredStageTable {
        /// Path of the descriptor.
        path: String,
        /// The stage id the table names.
        stage: String,
    },
    /// A workflow with no stages could never make progress.
    #[error("{path} declares no stages")]
    NoStages {
        /// Path of the descriptor.
        path: String,
    },
    /// The requested workflow name is not a plain directory name. It is
    /// joined directly onto `.sergeant/workflows/`, so anything else could
    /// read a `workflow.toml` outside that directory.
    #[error("workflow name {name:?} is not a plain directory name")]
    InvalidName {
        /// The offending name.
        name: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowFile {
    workflow: WorkflowSection,
    /// Optional `[stage."<id>"]` tables (§12.2), keyed by stage id. A
    /// `BTreeMap` gives deterministic iteration, which matters for the
    /// undeclared-table check's error ordering under multiple offenders.
    #[serde(default, rename = "stage")]
    stages_meta: BTreeMap<String, StageTable>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowSection {
    name: String,
    version: String,
    stages: Vec<String>,
}

/// One `[stage."<id>"]` table (§12.2). `deny_unknown_fields` is what turns an
/// execute-only field (`image`, `command`, `workdir`, ...) written under an
/// actor stage's table into a fail-closed parse error instead of a silently
/// ignored typo (§22.3) — this milestone models no execute-stage fields at
/// all, so any of them here is necessarily misplaced.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageTable {
    /// `"actor"` if present; absent defaults to actor too (§12.1's
    /// no-table default applies the same way inside an explicit table that
    /// only sets `harness`/`profile`).
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    harness: Option<String>,
    #[serde(default)]
    profile: Option<String>,
    /// R-MVP1-11's declaration. Absent means `false`, the same no-table
    /// default every other tagged field already uses.
    #[serde(default)]
    requires_ask: bool,
}

/// The built-in `software-change` workflow, embedded at build time from
/// `src/workflows/software-change/`. It is real workflow content parsed by the
/// same parser as a repository's own — not a second, code-shaped definition of
/// what a workflow is.
const EMBEDDED_WORKFLOW_TOML: &str = include_str!("../workflows/software-change/workflow.toml");
/// Embedded stage contexts, in the order `workflow.toml` declares them.
const EMBEDDED_CONTEXTS: &[(&str, &str)] = &[
    (
        "00-prepare",
        include_str!("../workflows/software-change/00-prepare/CONTEXT.md"),
    ),
    (
        "10-implement",
        include_str!("../workflows/software-change/10-implement/CONTEXT.md"),
    ),
    (
        "20-review",
        include_str!("../workflows/software-change/20-review/CONTEXT.md"),
    ),
    (
        "30-close",
        include_str!("../workflows/software-change/30-close/CONTEXT.md"),
    ),
];

impl WorkflowDefinition {
    /// Resolve `name` for a workspace rooted at `root`.
    ///
    /// A repository's own `.sergeant/workflows/<name>/` always wins; the
    /// built-in `software-change` is the fallback when the repository ships
    /// no workflow of that name.
    pub fn resolve(root: &Path, name: &str) -> Result<Self, WorkflowError> {
        if !is_plain_name(name) {
            return Err(WorkflowError::InvalidName {
                name: name.to_string(),
            });
        }
        let dir = workflow_dir(root, name);
        if dir.join(WORKFLOW_FILE).is_file() {
            return Self::load_dir(&dir);
        }
        if name == DEFAULT_WORKFLOW {
            return Self::embedded();
        }
        Err(WorkflowError::NotFound {
            name: name.to_string(),
            searched: dir.display().to_string(),
        })
    }

    /// Load a workflow from a directory holding `workflow.toml` and its stages.
    pub fn load_dir(dir: &Path) -> Result<Self, WorkflowError> {
        let descriptor = dir.join(WORKFLOW_FILE);
        let path = descriptor.display().to_string();
        let text = std::fs::read_to_string(&descriptor).map_err(|source| WorkflowError::Io {
            path: path.clone(),
            source,
        })?;
        let directory = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let parsed = parse_descriptor(&text, &path)?;
        if parsed.workflow.name != directory {
            return Err(WorkflowError::NameMismatch {
                path,
                declared: parsed.workflow.name,
                directory,
            });
        }
        let ids = check_stage_ids(&parsed.workflow.stages, &path)?;
        check_stage_tables(&ids, &parsed.stages_meta, &path)?;
        let mut stages = Vec::with_capacity(ids.len());
        for id in ids {
            let (kind, harness, profile, requires_ask) =
                resolve_stage_tag(&id, parsed.stages_meta.get(&id), &path)?;
            let stage_dir = dir.join(&id);
            let context_path = stage_dir.join(CONTEXT_FILE);
            if !context_path.is_file() {
                return Err(WorkflowError::MissingStage {
                    path: path.clone(),
                    stage: id,
                    missing: context_path.display().to_string(),
                });
            }
            let context =
                std::fs::read_to_string(&context_path).map_err(|source| WorkflowError::Io {
                    path: context_path.display().to_string(),
                    source,
                })?;
            stages.push(StageDefinition {
                id,
                context,
                kind,
                harness,
                profile,
                requires_ask,
            });
        }
        let content_hash =
            compute_content_hash(&parsed.workflow.name, &parsed.workflow.version, &stages);
        Ok(Self {
            name: parsed.workflow.name,
            version: parsed.workflow.version,
            source: dir.display().to_string(),
            stages,
            content_hash,
        })
    }

    /// The built-in `software-change` workflow.
    pub fn embedded() -> Result<Self, WorkflowError> {
        let path = format!("<embedded>/{DEFAULT_WORKFLOW}/{WORKFLOW_FILE}");
        let parsed = parse_descriptor(EMBEDDED_WORKFLOW_TOML, &path)?;
        let ids = check_stage_ids(&parsed.workflow.stages, &path)?;
        check_stage_tables(&ids, &parsed.stages_meta, &path)?;
        let mut stages = Vec::with_capacity(ids.len());
        for id in ids {
            let (kind, harness, profile, requires_ask) =
                resolve_stage_tag(&id, parsed.stages_meta.get(&id), &path)?;
            let context = EMBEDDED_CONTEXTS
                .iter()
                .find(|(stage, _)| *stage == id)
                .map(|(_, context)| (*context).to_string())
                .ok_or_else(|| WorkflowError::MissingStage {
                    path: path.clone(),
                    stage: id.clone(),
                    missing: format!("<embedded>/{DEFAULT_WORKFLOW}/{id}/{CONTEXT_FILE}"),
                })?;
            stages.push(StageDefinition {
                id,
                context,
                kind,
                harness,
                profile,
                requires_ask,
            });
        }
        let content_hash =
            compute_content_hash(&parsed.workflow.name, &parsed.workflow.version, &stages);
        Ok(Self {
            name: parsed.workflow.name,
            version: parsed.workflow.version,
            source: SOURCE_EMBEDDED.to_string(),
            stages,
            content_hash,
        })
    }

    /// The stage at `index`, if the workflow has one.
    pub fn stage(&self, index: usize) -> Option<&StageDefinition> {
        self.stages.get(index)
    }
}

/// Directory a named workflow would live in for a workspace root.
pub fn workflow_dir(root: &Path, name: &str) -> PathBuf {
    root.join(WORKFLOW_ROOT).join(name)
}

fn parse_descriptor(text: &str, path: &str) -> Result<WorkflowFile, WorkflowError> {
    toml::from_str(text).map_err(|source| WorkflowError::Malformed {
        path: path.to_string(),
        source,
    })
}

/// Validate the declared stage order: non-empty, unique, and every id a plain
/// directory name. The last check is a path-traversal guard — a stage id is
/// joined onto the workflow directory, and `../../etc` must not read files
/// outside it.
fn check_stage_ids(ids: &[String], path: &str) -> Result<Vec<String>, WorkflowError> {
    if ids.is_empty() {
        return Err(WorkflowError::NoStages {
            path: path.to_string(),
        });
    }
    let mut seen = std::collections::BTreeSet::new();
    for id in ids {
        if !is_plain_name(id) {
            return Err(WorkflowError::InvalidStageId {
                path: path.to_string(),
                stage: id.clone(),
            });
        }
        if !seen.insert(id.clone()) {
            return Err(WorkflowError::DuplicateStage {
                path: path.to_string(),
                stage: id.clone(),
            });
        }
    }
    Ok(ids.to_vec())
}

/// Every `[stage."<id>"]` table must name a stage the `stages` list actually
/// declares (§12.2, §22.3) — metadata for a nonexistent stage is refused, not
/// silently kept around unused. `BTreeMap` iteration keeps the reported
/// offender deterministic when more than one table is undeclared.
fn check_stage_tables(
    ids: &[String],
    tables: &BTreeMap<String, StageTable>,
    path: &str,
) -> Result<(), WorkflowError> {
    for key in tables.keys() {
        if !ids.iter().any(|id| id == key) {
            return Err(WorkflowError::UndeclaredStageTable {
                path: path.to_string(),
                stage: key.clone(),
            });
        }
    }
    Ok(())
}

/// Resolve one stage's `[stage."<id>"]` table, if it declared one, into its
/// executor tag. No table (the legacy shape, §12.1) and a table with no
/// `kind` both mean `Actor` with no explicit harness/profile — the same
/// actor-default outcome either way. An explicit `kind` other than `"actor"`
/// fails closed (§22.3): this milestone has no other legal kind to fall back
/// to.
fn resolve_stage_tag(
    id: &str,
    table: Option<&StageTable>,
    path: &str,
) -> Result<(StageKind, Option<String>, Option<String>, bool), WorkflowError> {
    let Some(table) = table else {
        return Ok((StageKind::Actor, None, None, false));
    };
    let kind = match table.kind.as_deref() {
        None | Some("actor") => StageKind::Actor,
        Some(other) => {
            return Err(WorkflowError::UnknownStageKind {
                path: path.to_string(),
                stage: id.to_string(),
                kind: other.to_string(),
            });
        }
    };
    Ok((
        kind,
        table.harness.clone(),
        table.profile.clone(),
        table.requires_ask,
    ))
}

/// A stage's contribution to the workflow content-identity hash: every
/// execution-relevant field, nothing provenance-only (no directory paths).
#[derive(Serialize)]
struct ContentIdentityStage<'a> {
    id: &'a str,
    kind: StageKind,
    harness: Option<&'a str>,
    profile: Option<&'a str>,
    context: &'a str,
    /// R-MVP1-11's declaration is execution-relevant (it changes what
    /// submit-time preflight will accept), so a workflow whose author flips
    /// it is a different workflow for content-identity purposes, same as a
    /// changed `harness` or `profile` is.
    requires_ask: bool,
}

/// The workflow content-identity hash (§22.3): BLAKE3 over a canonical JSON
/// projection of `name`, `version`, and each stage's execution-relevant
/// fields in order. Struct-derived `Serialize` fixes field order, so the
/// only way two workflows land on the same hash is genuinely identical
/// content — the same guarantee reordering `stages` or renaming a stage id
/// would break, which is exactly what §22.3 requires this hash to detect.
/// `source` (embedded vs. a directory path) is deliberately excluded: it says
/// where the definition came from, not what it does.
fn compute_content_hash(name: &str, version: &str, stages: &[StageDefinition]) -> String {
    #[derive(Serialize)]
    struct ContentIdentity<'a> {
        name: &'a str,
        version: &'a str,
        stages: Vec<ContentIdentityStage<'a>>,
    }
    let identity = ContentIdentity {
        name,
        version,
        stages: stages
            .iter()
            .map(|s| ContentIdentityStage {
                id: &s.id,
                kind: s.kind,
                harness: s.harness.as_deref(),
                profile: s.profile.as_deref(),
                context: &s.context,
                requires_ask: s.requires_ask,
            })
            .collect(),
    };
    let bytes =
        serde_json::to_vec(&identity).expect("ContentIdentity has no non-serializable field");
    blake3::hash(&bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_default_parses_with_its_stage_contexts() {
        let workflow = WorkflowDefinition::embedded().expect("embedded workflow");
        assert_eq!(workflow.name, DEFAULT_WORKFLOW);
        assert_eq!(workflow.source, SOURCE_EMBEDDED);
        let ids: Vec<&str> = workflow.stages.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, ["00-prepare", "10-implement", "20-review", "30-close"]);
        for stage in &workflow.stages {
            assert!(
                !stage.context.trim().is_empty(),
                "stage {} must carry its CONTEXT.md",
                stage.id
            );
        }
    }

    #[test]
    fn a_repositorys_own_workflow_wins_over_the_embedded_one() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, DEFAULT_WORKFLOW);
        std::fs::create_dir_all(wf.join("00-only")).expect("stage dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            "[workflow]\nname = \"software-change\"\nversion = \"7\"\nstages = [\"00-only\"]\n",
        )
        .expect("descriptor");
        std::fs::write(wf.join("00-only").join(CONTEXT_FILE), "do the thing")
            .expect("stage context");

        let workflow = WorkflowDefinition::resolve(root, DEFAULT_WORKFLOW).expect("resolve");
        assert_eq!(workflow.version, "7");
        assert_eq!(workflow.stages.len(), 1);
        assert_eq!(workflow.stages[0].context, "do the thing");
        assert_ne!(workflow.source, SOURCE_EMBEDDED);
    }

    /// The workflow name arrives from the API (`POST /v1/work`'s `workflow`
    /// field) and is joined straight onto `.sergeant/workflows/`. Without the
    /// guard, a traversing name reads and parses a `workflow.toml` from
    /// anywhere on the filesystem — so this test builds exactly that file and
    /// proves the refusal comes from the *name*, not from the file's absence.
    #[test]
    fn a_workflow_name_may_not_escape_the_workflows_directory() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join(WORKFLOW_ROOT)).expect("workflows root");

        // A perfectly loadable workflow, outside `.sergeant/workflows/`.
        let outside = root.join("outside");
        std::fs::create_dir_all(outside.join("00-only")).expect("stage dir");
        std::fs::write(
            outside.join(WORKFLOW_FILE),
            "[workflow]\nname = \"outside\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
        )
        .expect("descriptor");
        std::fs::write(outside.join("00-only").join(CONTEXT_FILE), "secrets")
            .expect("stage context");
        WorkflowDefinition::load_dir(&outside)
            .expect("the outside workflow really is loadable, so only the guard can refuse it");

        // `.sergeant/workflows/` + `../../outside` is exactly that directory.
        let escape = "../../outside";
        assert_eq!(
            workflow_dir(root, escape),
            root.join(WORKFLOW_ROOT).join(escape)
        );
        let err = WorkflowDefinition::resolve(root, escape).expect_err("must refuse");
        assert!(
            matches!(&err, WorkflowError::InvalidName { name } if name == escape),
            "expected a refusal naming the escape, got {err}"
        );
    }

    #[test]
    fn malformed_workflows_fail_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();

        // Unknown workflow name, no embedded fallback.
        assert!(matches!(
            WorkflowDefinition::resolve(root, "nope"),
            Err(WorkflowError::NotFound { .. })
        ));

        // Declared stage with no directory.
        let wf = workflow_dir(root, "ghost");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            "[workflow]\nname = \"ghost\"\nversion = \"1\"\nstages = [\"00-missing\"]\n",
        )
        .expect("descriptor");
        assert!(matches!(
            WorkflowDefinition::resolve(root, "ghost"),
            Err(WorkflowError::MissingStage { .. })
        ));

        // Stage ids may not escape the workflow directory.
        let wf = workflow_dir(root, "escape");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            "[workflow]\nname = \"escape\"\nversion = \"1\"\nstages = [\"../../etc\"]\n",
        )
        .expect("descriptor");
        assert!(matches!(
            WorkflowDefinition::resolve(root, "escape"),
            Err(WorkflowError::InvalidStageId { .. })
        ));

        // A workflow name may not escape `.sergeant/workflows/` either.
        assert!(matches!(
            WorkflowDefinition::resolve(root, "../../etc"),
            Err(WorkflowError::InvalidName { .. })
        ));

        // A typo'd key is refused rather than silently ignored.
        let wf = workflow_dir(root, "typo");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            "[workflow]\nname = \"typo\"\nversion = \"1\"\nstages = []\nstagez = [\"a\"]\n",
        )
        .expect("descriptor");
        assert!(matches!(
            WorkflowDefinition::resolve(root, "typo"),
            Err(WorkflowError::Malformed { .. })
        ));
    }

    /// Writes one stage's `CONTEXT.md`, creating its directory.
    fn write_stage(wf: &Path, id: &str, context: &str) {
        std::fs::create_dir_all(wf.join(id)).expect("stage dir");
        std::fs::write(wf.join(id).join(CONTEXT_FILE), context).expect("stage context");
    }

    /// §22.3: a legacy `workflow.toml` (bare `[workflow]`, no `[stage.*]`
    /// tables at all) parses identically to before — every stage resolves to
    /// the same actor-default outcome N3 assigns a stage with an *explicit*
    /// but empty `[stage."<id>"]` table (§12.1's default text describes one
    /// outcome, reached two ways).
    #[test]
    fn legacy_workflow_defaults_every_stage_to_an_untagged_actor() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "legacy");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            "[workflow]\nname = \"legacy\"\nversion = \"1\"\nstages = [\"00-only\", \"10-next\"]\n",
        )
        .expect("descriptor");
        write_stage(&wf, "00-only", "first");
        write_stage(&wf, "10-next", "second");

        let workflow = WorkflowDefinition::resolve(root, "legacy").expect("resolve");
        for stage in &workflow.stages {
            assert_eq!(stage.kind, StageKind::Actor);
            assert_eq!(stage.harness, None);
            assert_eq!(stage.profile, None);
        }
        assert!(
            !workflow.content_hash.is_empty(),
            "every resolved workflow carries a content-identity hash"
        );
    }

    /// §22.3: a `[stage."<id>"]` table pins `kind`, `harness`, and `profile`
    /// onto the resolved stage; a stage with no table stays untagged.
    #[test]
    fn a_tagged_stage_table_pins_kind_harness_and_profile() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "tagged");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"tagged\"\n",
                "version = \"1\"\n",
                "stages = [\"00-review\", \"10-close\"]\n",
                "\n",
                "[stage.\"00-review\"]\n",
                "kind = \"actor\"\n",
                "harness = \"claude\"\n",
                "profile = \"review\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&wf, "00-review", "review context");
        write_stage(&wf, "10-close", "close context");

        let workflow = WorkflowDefinition::resolve(root, "tagged").expect("resolve");
        assert_eq!(workflow.stages[0].kind, StageKind::Actor);
        assert_eq!(workflow.stages[0].harness.as_deref(), Some("claude"));
        assert_eq!(workflow.stages[0].profile.as_deref(), Some("review"));
        // The untagged second stage keeps the legacy actor-default outcome.
        assert_eq!(workflow.stages[1].kind, StageKind::Actor);
        assert_eq!(workflow.stages[1].harness, None);
        assert_eq!(workflow.stages[1].profile, None);
    }

    /// §22.3: an unknown `kind` fails closed rather than being coerced to
    /// `actor` or silently ignored. `"actor"` is the only legal kind this
    /// milestone (N3 Outcome 2) — `"execute"` is real vocabulary from the
    /// proposal (§12.3), reserved for N4, and still not accepted yet.
    #[test]
    fn an_unknown_stage_kind_fails_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "bad-kind");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"bad-kind\"\n",
                "version = \"1\"\n",
                "stages = [\"00-only\"]\n",
                "\n",
                "[stage.\"00-only\"]\n",
                "kind = \"execute\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&wf, "00-only", "context");

        let err = WorkflowDefinition::resolve(root, "bad-kind").expect_err("must refuse");
        assert!(
            matches!(&err, WorkflowError::UnknownStageKind { stage, kind, .. }
                if stage == "00-only" && kind == "execute"),
            "expected an unknown-kind refusal naming the offending stage and kind, got {err}"
        );
    }

    /// §22.3: a `[stage."<id>"]` table for an id the `stages` list never
    /// declared is refused rather than kept around unused.
    #[test]
    fn a_stage_table_for_an_undeclared_stage_id_fails_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "ghost-table");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"ghost-table\"\n",
                "version = \"1\"\n",
                "stages = [\"00-only\"]\n",
                "\n",
                "[stage.\"99-ghost\"]\n",
                "kind = \"actor\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&wf, "00-only", "context");

        let err = WorkflowDefinition::resolve(root, "ghost-table").expect_err("must refuse");
        assert!(
            matches!(&err, WorkflowError::UndeclaredStageTable { stage, .. } if stage == "99-ghost"),
            "expected a refusal naming the undeclared table, got {err}"
        );
    }

    /// R-MVP1-11: `requires_ask` parses to `true` when declared and defaults
    /// to `false` for every stage that never mentions it — the same
    /// no-table-means-untagged shape `kind`/`harness`/`profile` already have,
    /// and it participates in the content-identity hash like they do.
    #[test]
    fn requires_ask_parses_true_and_defaults_false() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "asks");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"asks\"\n",
                "version = \"1\"\n",
                "stages = [\"00-interview\", \"10-close\"]\n",
                "\n",
                "[stage.\"00-interview\"]\n",
                "requires_ask = true\n",
            ),
        )
        .expect("descriptor");
        write_stage(&wf, "00-interview", "interview context");
        write_stage(&wf, "10-close", "close context");

        let workflow = WorkflowDefinition::resolve(root, "asks").expect("resolve");
        assert!(workflow.stages[0].requires_ask);
        assert!(
            !workflow.stages[1].requires_ask,
            "untagged stage defaults false"
        );

        // Flipping the declaration is a different workflow (§22.3): content
        // hash changes even though every other field is untouched.
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"asks\"\n",
                "version = \"1\"\n",
                "stages = [\"00-interview\", \"10-close\"]\n",
            ),
        )
        .expect("descriptor");
        let unrequired = WorkflowDefinition::resolve(root, "asks").expect("resolve");
        assert_ne!(
            workflow.content_hash, unrequired.content_hash,
            "requires_ask must be execution-relevant to the content-identity hash"
        );
    }

    /// §22.3: an unrecognized key inside a `[stage."<id>"]` table is a parse
    /// failure, not a silently ignored typo — the same discipline
    /// `malformed_workflows_fail_closed` already pins for `[workflow]`.
    #[test]
    fn unknown_fields_in_a_stage_table_fail_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "typo-stage");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"typo-stage\"\n",
                "version = \"1\"\n",
                "stages = [\"00-only\"]\n",
                "\n",
                "[stage.\"00-only\"]\n",
                "harnass = \"claude\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&wf, "00-only", "context");

        assert!(matches!(
            WorkflowDefinition::resolve(root, "typo-stage"),
            Err(WorkflowError::Malformed { .. })
        ));
    }

    /// §22.3: execute-only fields (§12.3's `image`/`command`/`workdir`/...)
    /// written into a stage table are rejected rather than ignored. This
    /// milestone models no execute-stage fields at all, so any of them here
    /// is necessarily misplaced — `deny_unknown_fields` is what does the
    /// rejecting, exercised here with the actual proposal vocabulary rather
    /// than an arbitrary typo.
    #[test]
    fn execute_only_fields_on_an_actor_stage_fail_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "misplaced");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"misplaced\"\n",
                "version = \"1\"\n",
                "stages = [\"00-only\"]\n",
                "\n",
                "[stage.\"00-only\"]\n",
                "kind = \"actor\"\n",
                "image = \"python:3.13-slim\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&wf, "00-only", "context");

        assert!(matches!(
            WorkflowDefinition::resolve(root, "misplaced"),
            Err(WorkflowError::Malformed { .. })
        ));
    }

    /// §22.3: the content-identity hash changes when a context changes, when
    /// a stage's harness/profile changes, and when stage order changes for
    /// the same set of ids — but not when only `source` differs (the same
    /// content loaded from two different directories hashes the same).
    #[test]
    fn content_identity_tracks_execution_relevant_fields_only() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();

        let base = workflow_dir(root, "base");
        std::fs::create_dir_all(&base).expect("workflow dir");
        std::fs::write(
            base.join(WORKFLOW_FILE),
            "[workflow]\nname = \"base\"\nversion = \"1\"\nstages = [\"00-a\", \"10-b\"]\n",
        )
        .expect("descriptor");
        write_stage(&base, "00-a", "context a");
        write_stage(&base, "10-b", "context b");
        let baseline = WorkflowDefinition::resolve(root, "base").expect("resolve base");

        // An exact duplicate under a second root, so its absolute `source`
        // path necessarily differs while the declared name still matches
        // its own directory (`load_dir` requires that, independent of this
        // hash question).
        let other_root = tempfile::TempDir::new().expect("tempdir");
        let dup = workflow_dir(other_root.path(), "base");
        std::fs::create_dir_all(&dup).expect("workflow dir");
        std::fs::write(
            dup.join(WORKFLOW_FILE),
            "[workflow]\nname = \"base\"\nversion = \"1\"\nstages = [\"00-a\", \"10-b\"]\n",
        )
        .expect("descriptor");
        write_stage(&dup, "00-a", "context a");
        write_stage(&dup, "10-b", "context b");
        let duplicate = WorkflowDefinition::load_dir(&dup).expect("resolve dup");
        assert_ne!(baseline.source, duplicate.source);
        assert_eq!(
            baseline.content_hash, duplicate.content_hash,
            "source is provenance, not execution-relevant content"
        );

        // Each variant below needs its own root: `load_dir` requires the
        // declared name to match the directory name, and every variant
        // keeps `name = "base"` on purpose — only its content changes, so a
        // hash difference can be attributed to that content alone rather
        // than a coincidental name change.

        // Changed context.
        let context_root = tempfile::TempDir::new().expect("tempdir");
        let changed_context = workflow_dir(context_root.path(), "base");
        std::fs::create_dir_all(&changed_context).expect("workflow dir");
        std::fs::write(
            changed_context.join(WORKFLOW_FILE),
            "[workflow]\nname = \"base\"\nversion = \"1\"\nstages = [\"00-a\", \"10-b\"]\n",
        )
        .expect("descriptor");
        write_stage(&changed_context, "00-a", "context a, but different");
        write_stage(&changed_context, "10-b", "context b");
        let changed = WorkflowDefinition::load_dir(&changed_context).expect("resolve changed");
        assert_ne!(baseline.content_hash, changed.content_hash);

        // Changed harness on an otherwise identical stage.
        let harness_root = tempfile::TempDir::new().expect("tempdir");
        let changed_harness = workflow_dir(harness_root.path(), "base");
        std::fs::create_dir_all(&changed_harness).expect("workflow dir");
        std::fs::write(
            changed_harness.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"base\"\n",
                "version = \"1\"\n",
                "stages = [\"00-a\", \"10-b\"]\n",
                "\n",
                "[stage.\"00-a\"]\n",
                "harness = \"codex\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&changed_harness, "00-a", "context a");
        write_stage(&changed_harness, "10-b", "context b");
        let harnessed = WorkflowDefinition::load_dir(&changed_harness).expect("resolve harness");
        assert_ne!(baseline.content_hash, harnessed.content_hash);

        // Reordered stages: same ids, same contexts, different order.
        let reorder_root = tempfile::TempDir::new().expect("tempdir");
        let reordered = workflow_dir(reorder_root.path(), "base");
        std::fs::create_dir_all(&reordered).expect("workflow dir");
        std::fs::write(
            reordered.join(WORKFLOW_FILE),
            "[workflow]\nname = \"base\"\nversion = \"1\"\nstages = [\"10-b\", \"00-a\"]\n",
        )
        .expect("descriptor");
        write_stage(&reordered, "00-a", "context a");
        write_stage(&reordered, "10-b", "context b");
        let reorder = WorkflowDefinition::load_dir(&reordered).expect("resolve reorder");
        assert_ne!(baseline.content_hash, reorder.content_hash);
    }

    /// A descriptor's `name` field must match the directory it lives in — the
    /// directory name is otherwise never checked against anything, so a copy
    /// of one workflow directory into another (with the old `name` left
    /// behind) would silently claim the new directory's identity.
    #[test]
    fn a_declared_name_that_does_not_match_its_directory_is_refused() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "actual-dir-name");
        std::fs::create_dir_all(wf.join("00-only")).expect("stage dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            "[workflow]\nname = \"declared-other-name\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
        )
        .expect("descriptor");
        std::fs::write(wf.join("00-only").join(CONTEXT_FILE), "do the thing")
            .expect("stage context");

        let err = WorkflowDefinition::load_dir(&wf).expect_err("mismatched name must be refused");
        match err {
            WorkflowError::NameMismatch {
                declared,
                directory,
                ..
            } => {
                assert_eq!(declared, "declared-other-name");
                assert_eq!(directory, "actual-dir-name");
            }
            other => panic!("expected NameMismatch, got {other}"),
        }
    }

    /// A workflow with no stages could never make progress, so it is refused
    /// at load time rather than accepted as a run nothing can advance.
    #[test]
    fn a_workflow_declaring_no_stages_is_refused() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "empty");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            "[workflow]\nname = \"empty\"\nversion = \"1\"\nstages = []\n",
        )
        .expect("descriptor");

        let err =
            WorkflowDefinition::load_dir(&wf).expect_err("an empty stage list must be refused");
        assert!(
            matches!(&err, WorkflowError::NoStages { path } if path.contains(WORKFLOW_FILE)),
            "expected NoStages naming the descriptor, got {err}"
        );
    }

    /// A stage id repeated in the declared order is refused before any
    /// per-stage directory or `CONTEXT.md` is even looked at: two attempts at
    /// the same id would otherwise be indistinguishable in the engine.
    #[test]
    fn a_stage_id_declared_twice_is_refused() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "dupe");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            "[workflow]\nname = \"dupe\"\nversion = \"1\"\nstages = [\"00-only\", \"00-only\"]\n",
        )
        .expect("descriptor");

        let err = WorkflowDefinition::load_dir(&wf)
            .expect_err("a stage id declared twice must be refused");
        assert!(
            matches!(&err, WorkflowError::DuplicateStage { stage, .. } if stage == "00-only"),
            "expected DuplicateStage naming the repeated id, got {err}"
        );
    }
}

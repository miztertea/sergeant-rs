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

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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
/// Event kind: a stage is blocked.
pub const KIND_STAGE_BLOCKED: &str = "stage.blocked";
/// Event kind: a stage failed.
pub const KIND_STAGE_FAILED: &str = "stage.failed";
/// Event kind: a stage was canceled.
pub const KIND_STAGE_CANCELED: &str = "stage.canceled";

/// One stage: an ordered directory with actor-readable context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageDefinition {
    /// Directory name, which is also the stage id (`10-implement`).
    pub id: String,
    /// The stage's `CONTEXT.md`, carried verbatim.
    pub context: String,
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
    /// A workflow with no stages could never make progress.
    #[error("{path} declares no stages")]
    NoStages {
        /// Path of the descriptor.
        path: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowFile {
    workflow: WorkflowSection,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowSection {
    name: String,
    version: String,
    stages: Vec<String>,
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
        let mut stages = Vec::with_capacity(parsed.workflow.stages.len());
        for id in check_stage_ids(&parsed.workflow.stages, &path)? {
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
            stages.push(StageDefinition { id, context });
        }
        Ok(Self {
            name: parsed.workflow.name,
            version: parsed.workflow.version,
            source: dir.display().to_string(),
            stages,
        })
    }

    /// The built-in `software-change` workflow.
    pub fn embedded() -> Result<Self, WorkflowError> {
        let path = format!("<embedded>/{DEFAULT_WORKFLOW}/{WORKFLOW_FILE}");
        let parsed = parse_descriptor(EMBEDDED_WORKFLOW_TOML, &path)?;
        let mut stages = Vec::with_capacity(parsed.workflow.stages.len());
        for id in check_stage_ids(&parsed.workflow.stages, &path)? {
            let context = EMBEDDED_CONTEXTS
                .iter()
                .find(|(stage, _)| *stage == id)
                .map(|(_, context)| (*context).to_string())
                .ok_or_else(|| WorkflowError::MissingStage {
                    path: path.clone(),
                    stage: id.clone(),
                    missing: format!("<embedded>/{DEFAULT_WORKFLOW}/{id}/{CONTEXT_FILE}"),
                })?;
            stages.push(StageDefinition { id, context });
        }
        Ok(Self {
            name: parsed.workflow.name,
            version: parsed.workflow.version,
            source: SOURCE_EMBEDDED.to_string(),
            stages,
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
        if id.is_empty()
            || id.contains('/')
            || id.contains('\\')
            || id == "."
            || id == ".."
            || Path::new(id).components().count() != 1
        {
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
}

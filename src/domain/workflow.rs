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
/// A repository's own forked workflow packages. A package here shadows the
/// stock package of the same name under [`WORKFLOW_ROOT`] by directory
/// precedence — the same mechanism `/etc` vs `/usr/lib` uses (proposal §6
/// Phase 3, Ponytail R4): whole-package override, never composition or
/// stage-level merging (ADR 0014 decision 3, ADR 0016).
pub const LOCAL_WORKFLOW_ROOT: &str = ".sergeant/local/workflows";
/// Machine-readable workflow descriptor inside a workflow directory.
pub const WORKFLOW_FILE: &str = "workflow.toml";
/// Actor-readable stage context file inside a stage directory.
pub const CONTEXT_FILE: &str = "CONTEXT.md";
/// Name of the built-in workflow used when a estate ships none.
pub const DEFAULT_WORKFLOW: &str = "software-change";
/// Source marker for the embedded built-in workflow.
pub const SOURCE_EMBEDDED: &str = "embedded";
/// A workflow directory's own OKF front-matter file (`docs/icm/
/// record-shapes.md` §1) — distinct from the root catalog below.
pub const INDEX_FILE: &str = "index.md";
/// The root workflow catalog (`docs/icm/record-shapes.md` §1 rule 2,
/// `docs/icm/convention.md` §1 rule 1): "the list, not an entry". Lists every
/// `status: published` workflow under [`WORKFLOW_ROOT`]; drafts and
/// unindexed directories are never in it (§11.1's Decision T2-39).
pub const ROOT_CATALOG_FILE: &str = ".sergeant/index.md";

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
/// `Actor` and `Execute` are the two legal kinds (N4 activates `Execute`,
/// §11.2/§12.3, over the vocabulary N3 reserved but refused). A
/// `[stage."<id>"]` table naming any other kind still fails closed at load
/// time rather than being accepted and ignored (§22.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StageKind {
    /// A native reasoning harness performs this stage (§11.1).
    #[default]
    Actor,
    /// A declared container image, run through the local Docker Engine
    /// (§11.2). Its semantic result is mechanical: exit 0 completes the
    /// stage, nonzero fails it, an ambiguous container outcome blocks it.
    /// sergeant never interprets the container's stdout to decide the
    /// outcome (§11.2's "stdout content never silently changes the result").
    Execute,
}

impl StageKind {
    /// The kind's canonical snake_case name, as it appears in `workflow.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            StageKind::Actor => "actor",
            StageKind::Execute => "execute",
        }
    }
}

/// How the container's mount of the work surface is opened (§12.3, §16.6).
/// A stage must declare this explicitly — there is no default — because
/// silently granting write access to a container merely because most coding
/// workloads happen to need it is exactly the guess §16.6 forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceAccess {
    /// The container may read the worktree but not modify it.
    ReadOnly,
    /// The container may read and write the worktree.
    ReadWrite,
}

impl WorkspaceAccess {
    /// The access mode's canonical snake_case name.
    pub fn as_str(self) -> &'static str {
        match self {
            WorkspaceAccess::ReadOnly => "read_only",
            WorkspaceAccess::ReadWrite => "read_write",
        }
    }
}

/// The execute-stage container's network policy (§12.3, §16.7).
///
/// `None` (Docker's no-network mode) is the only legal value in this schema.
/// §16.7 is explicit that "the first schema should not accept arbitrary
/// Docker network names or host networking" — a named policy admitting
/// outbound access is future, measured work, not a value this parser
/// silently accepts and ignores today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicy {
    /// Docker's no-network mode: the container has no usable external
    /// network path (§16.7, §22.7 test 5).
    None,
}

impl NetworkPolicy {
    /// The policy's canonical snake_case name.
    pub fn as_str(self) -> &'static str {
        match self {
            NetworkPolicy::None => "none",
        }
    }
}

/// A resolved, immutable image identity (§16.4). Distinct from the authored
/// `image` reference (a mutable tag is an acceptable authoring choice, never
/// acceptable as historical evidence): this is what actually ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedImage {
    /// The reference the workflow authored (`python:3.13-slim`).
    pub image_requested: String,
    /// The immutable image ID Docker reports (`sha256:...`).
    pub image_id: String,
    /// Repository digests the image resolved to, when the registry gave one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repo_digests: Vec<String>,
    /// The platform the image was resolved for (`linux/amd64`).
    pub platform: String,
}

/// The pinned specification for one `kind = "execute"` stage (§12.3, §13.1's
/// conceptual `ExecuteStage`). Every field here is execution-relevant and
/// participates in [`WorkflowDefinition::content_hash`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecuteSpec {
    /// The authored image reference. May be a mutable tag (§16.4) — the
    /// executor pins the resolved identity separately, at launch.
    pub image: String,
    /// Argv run inside the container. No shell is invented: a workflow that
    /// needs one names it explicitly (`["bash", "-lc", "..."]`, §12.3).
    pub command: Vec<String>,
    /// Container working directory.
    pub workdir: String,
    /// Whether the worktree mount is read-only or read-write.
    pub workspace_access: WorkspaceAccess,
    /// Network policy. Only [`NetworkPolicy::None`] is legal this schema.
    pub network: NetworkPolicy,
    /// Extra environment variables for the container. Never a place for
    /// secrets (§16.5) — the workflow file is plaintext, versioned content.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
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
    /// The pinned container specification, present exactly when
    /// `kind == StageKind::Execute` (§12.3, §13.1). `None` for every actor
    /// stage — the same tagged-by-`kind` shape `harness`/`profile` already
    /// have for the actor side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execute: Option<ExecuteSpec>,
}

/// The executor decision for **one stage of one run**, resolved at plan time
/// and pinned in `workflow.bound` (§12.5, §13.3, §17.5).
///
/// Why it is pinned rather than re-derived at each stage entry: §22.4 requires
/// that "retry uses the same pinned stage harness/profile/model decision" and
/// that "restart reconstructs the same decision from the journal". A decision
/// re-derived at entry time would be re-derived against whatever the registry,
/// the estate's `sergeant.toml` and the harness probes say *then* — so a
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
    /// A `[stage."<id>"]` table names a `kind` this parser does not
    /// support. `"actor"` and `"execute"` are the only legal kinds (§12.2,
    /// §12.3, N4 Outcome).
    #[error(
        "{path} declares stage {stage:?} with unknown kind {kind:?} (only \"actor\" and \"execute\" are supported)"
    )]
    UnknownStageKind {
        /// Path of the descriptor.
        path: String,
        /// The offending stage id.
        stage: String,
        /// The unsupported `kind` value.
        kind: String,
    },
    /// A `[stage."<id>"]` table with `kind = "execute"` is missing a field
    /// the execute schema (§12.3) requires unconditionally.
    #[error(
        "{path} declares execute stage {stage:?} with no {field} (required for kind = \"execute\")"
    )]
    MissingExecuteField {
        /// Path of the descriptor.
        path: String,
        /// The offending stage id.
        stage: String,
        /// The missing field's name.
        field: String,
    },
    /// An execute-only field (`image`, `command`, `workdir`,
    /// `workspace_access`, `network`, `env`) appears on a stage whose kind is
    /// (or defaults to) `"actor"` — rejected rather than silently ignored,
    /// the same discipline `deny_unknown_fields` gives an outright typo
    /// (§22.3).
    #[error("{path} declares actor stage {stage:?} with execute-only field {field:?}")]
    ExecuteFieldOnActorStage {
        /// Path of the descriptor.
        path: String,
        /// The offending stage id.
        stage: String,
        /// The misplaced field's name.
        field: String,
    },
    /// An actor-only field (`harness`, `profile`, `requires_ask`) appears on
    /// a stage whose kind is `"execute"` — execute stages have no harness to
    /// select and cannot ask (§11.2, §15.4).
    #[error("{path} declares execute stage {stage:?} with actor-only field {field:?}")]
    ActorFieldOnExecuteStage {
        /// Path of the descriptor.
        path: String,
        /// The offending stage id.
        stage: String,
        /// The misplaced field's name.
        field: String,
    },
    /// `workspace_access` names something other than `"read_only"` or
    /// `"read_write"` (§16.6).
    #[error(
        "{path} declares execute stage {stage:?} with unknown workspace_access {value:?} \
         (only \"read_only\" and \"read_write\" are supported)"
    )]
    UnknownWorkspaceAccess {
        /// Path of the descriptor.
        path: String,
        /// The offending stage id.
        stage: String,
        /// The unsupported value.
        value: String,
    },
    /// `network` names something other than `"none"` — the first schema does
    /// not accept arbitrary Docker network names or host networking (§16.7).
    #[error(
        "{path} declares execute stage {stage:?} with unknown network policy {value:?} \
         (only \"none\" is supported)"
    )]
    UnknownNetworkPolicy {
        /// Path of the descriptor.
        path: String,
        /// The offending stage id.
        stage: String,
        /// The unsupported value.
        value: String,
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

/// One `[stage."<id>"]` table (§12.2, §12.3). `deny_unknown_fields` still
/// turns an outright typo into a fail-closed parse error rather than a
/// silently ignored one (§22.3); which of *these* known fields are legal
/// together is a `kind`-dependent question this struct cannot express by
/// itself, so [`resolve_stage_tag`] enforces it explicitly — an execute-only
/// field on an actor stage, or an actor-only field on an execute stage, is
/// refused there with a field-naming error instead of the generic malformed
/// one a bare typo gets.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageTable {
    /// `"actor"` or `"execute"` if present; absent defaults to actor
    /// (§12.1's no-table default applies the same way inside an explicit
    /// table that only sets other fields).
    #[serde(default)]
    kind: Option<String>,
    // --- actor-only (§12.2) ---
    #[serde(default)]
    harness: Option<String>,
    #[serde(default)]
    profile: Option<String>,
    /// R-MVP1-11's declaration. Absent means `false`, the same no-table
    /// default every other tagged field already uses.
    #[serde(default)]
    requires_ask: bool,
    // --- execute-only (§12.3) ---
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    command: Option<Vec<String>>,
    #[serde(default)]
    workdir: Option<String>,
    #[serde(default)]
    workspace_access: Option<String>,
    #[serde(default)]
    network: Option<String>,
    #[serde(default)]
    env: Option<BTreeMap<String, String>>,
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
    /// Resolve `name` for a estate rooted at `root`.
    ///
    /// A repository's own `.sergeant/local/workflows/<name>/` always wins
    /// over the stock `.sergeant/workflows/<name>/` of the same name
    /// (directory precedence, proposal §6 Phase 3 / Ponytail R4); stock in
    /// turn wins over the built-in `software-change`, the fallback when the
    /// repository ships no workflow of that name anywhere.
    pub fn resolve(root: &Path, name: &str) -> Result<Self, WorkflowError> {
        if !is_plain_name(name) {
            return Err(WorkflowError::InvalidName {
                name: name.to_string(),
            });
        }
        let local_dir = local_workflow_dir(root, name);
        if local_dir.join(WORKFLOW_FILE).is_file() {
            return Self::load_dir(&local_dir);
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
            let tag = resolve_stage_tag(&id, parsed.stages_meta.get(&id), &path)?;
            let stage_dir = dir.join(&id);
            let context_path = stage_dir.join(CONTEXT_FILE);
            let context = if context_path.is_file() {
                std::fs::read_to_string(&context_path).map_err(|source| WorkflowError::Io {
                    path: context_path.display().to_string(),
                    source,
                })?
            } else if tag.kind == StageKind::Execute {
                // §12.4: an execute stage's `CONTEXT.md`/`README.md` is
                // optional human-readable documentation the driver never
                // consumes — absence is not an error the way it is for an
                // actor stage, whose `CONTEXT.md` *is* its procedure.
                let readme_path = stage_dir.join("README.md");
                if readme_path.is_file() {
                    std::fs::read_to_string(&readme_path).map_err(|source| WorkflowError::Io {
                        path: readme_path.display().to_string(),
                        source,
                    })?
                } else {
                    String::new()
                }
            } else {
                return Err(WorkflowError::MissingStage {
                    path: path.clone(),
                    stage: id,
                    missing: context_path.display().to_string(),
                });
            };
            stages.push(StageDefinition {
                id,
                context,
                kind: tag.kind,
                harness: tag.harness,
                profile: tag.profile,
                requires_ask: tag.requires_ask,
                execute: tag.execute,
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
            let tag = resolve_stage_tag(&id, parsed.stages_meta.get(&id), &path)?;
            let embedded_context = EMBEDDED_CONTEXTS
                .iter()
                .find(|(stage, _)| *stage == id)
                .map(|(_, context)| (*context).to_string());
            let context = match embedded_context {
                Some(context) => context,
                // §12.4: same optional-documentation rule as `load_dir` —
                // only load-bearing for a future embedded execute stage, but
                // kept symmetric rather than a silent divergence between the
                // two loaders.
                None if tag.kind == StageKind::Execute => String::new(),
                None => {
                    return Err(WorkflowError::MissingStage {
                        path: path.clone(),
                        stage: id.clone(),
                        missing: format!("<embedded>/{DEFAULT_WORKFLOW}/{id}/{CONTEXT_FILE}"),
                    });
                }
            };
            stages.push(StageDefinition {
                id,
                context,
                kind: tag.kind,
                harness: tag.harness,
                profile: tag.profile,
                requires_ask: tag.requires_ask,
                execute: tag.execute,
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

/// Authored fields from a workflow's own `index.md` front matter
/// (`docs/icm/record-shapes.md` §1) — the part of a [`CatalogEntry`] that
/// `workflow.toml` does not carry. `tags` is `None` when the front matter
/// declares no `tags:` key at all, never an empty list standing in for
/// "none" (§11.2's CatalogEntry table).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowIndexFrontMatter {
    /// `status:` verbatim (expected to always be `"published"` here: T2-39
    /// excludes drafts before a name ever reaches [`catalog`]).
    pub status: String,
    /// `description:` verbatim.
    pub description: String,
    /// `tags:` verbatim, or `None` if the front matter never declared the key.
    pub tags: Option<Vec<String>>,
    /// `edition:` verbatim (`docs/icm/record-shapes.md` §1, ADR 0016): the
    /// distro version that wrote this file as stock content. `None` if the
    /// front matter never declared the key — pre-ADR-0016 content, or a
    /// malformed fork.
    pub edition: Option<String>,
}

/// One entry of the read-only workflow catalog `GET /v1/workflows` projects
/// (§11.2): a resolved [`WorkflowDefinition`] plus whatever its own
/// `index.md` says. `front_matter` is `None` only for the embedded fallback,
/// which has no `index.md` to read (§11.2's "absent for the embedded entry").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    /// The resolved workflow: name, version, source, content hash, stages.
    pub definition: WorkflowDefinition,
    /// The workflow's own `index.md` front matter, when it has one.
    pub front_matter: Option<WorkflowIndexFrontMatter>,
}

/// Directory a named workflow would live in for a estate root.
pub fn workflow_dir(root: &Path, name: &str) -> PathBuf {
    root.join(WORKFLOW_ROOT).join(name)
}

/// Directory a named workflow's local fork would live in for a estate
/// root ([`LOCAL_WORKFLOW_ROOT`]).
pub fn local_workflow_dir(root: &Path, name: &str) -> PathBuf {
    root.join(LOCAL_WORKFLOW_ROOT).join(name)
}

/// Failure forking a stock workflow package into `.sergeant/local/workflows/`.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowForkError {
    /// The requested name is not a plain directory name (same rule as
    /// [`WorkflowDefinition::resolve`] — a name is joined straight onto both
    /// workflow roots).
    #[error("workflow name {name:?} is not a plain directory name")]
    InvalidName {
        /// The offending name.
        name: String,
    },
    /// No stock package of this name exists to fork. Forking the embedded
    /// `software-change` default is out of scope: it has no on-disk
    /// directory to copy from (§6 Phase 3 scopes `sgt workflow fork` to
    /// copying a stock *package*, not materializing the embedded one).
    #[error("no stock workflow package {name:?} at {searched} — nothing to fork")]
    NoStockPackage {
        /// The offending name.
        name: String,
        /// Where a stock package was searched for.
        searched: String,
    },
    /// A local package of this name already exists. `sgt workflow fork`
    /// never silently overwrites a user's work (§6 Phase 3).
    #[error("a local workflow package {name:?} already exists at {path}")]
    LocalAlreadyExists {
        /// The offending name.
        name: String,
        /// Where the existing local package lives.
        path: String,
    },
    /// Copying the stock package's files failed partway through.
    #[error("copying {from} to {to} failed: {source}")]
    Io {
        /// Source path.
        from: String,
        /// Destination path.
        to: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// `sgt workflow fork <name>` (§6 Phase 3, Ponytail R7 — the one genuinely
/// new verb the phase adds, claimed only because R2–R6 answer everything
/// else: R4 covers resolution precedence, R6 covers drift as a string
/// comparison, R2 covers surfacing drift through `sgt doctor`, and no
/// existing verb copies a stock package to local with its `edition` stamp
/// intact). Copies `.sergeant/workflows/<name>/` byte-for-byte into
/// `.sergeant/local/workflows/<name>/`, preserving whatever `edition` the
/// stock copy carried — a plain recursive copy leaves every file, including
/// `index.md`'s front matter, untouched, so the fork stays comparable via
/// ADR 0016's string comparison from the moment it's created. Refuses if a
/// local package of this name already exists, or if there is no stock
/// package to fork from.
pub fn fork(root: &Path, name: &str) -> Result<PathBuf, WorkflowForkError> {
    if !is_plain_name(name) {
        return Err(WorkflowForkError::InvalidName {
            name: name.to_string(),
        });
    }
    let stock_dir = workflow_dir(root, name);
    if !stock_dir.join(WORKFLOW_FILE).is_file() {
        return Err(WorkflowForkError::NoStockPackage {
            name: name.to_string(),
            searched: stock_dir.display().to_string(),
        });
    }
    let local_dir = local_workflow_dir(root, name);
    if local_dir.exists() {
        return Err(WorkflowForkError::LocalAlreadyExists {
            name: name.to_string(),
            path: local_dir.display().to_string(),
        });
    }
    copy_dir_recursive(&stock_dir, &local_dir)?;
    Ok(local_dir)
}

/// Recursively copy every file and subdirectory under `from` into `to`,
/// creating `to` (and any intermediate directories) as needed.
fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), WorkflowForkError> {
    std::fs::create_dir_all(to).map_err(|source| WorkflowForkError::Io {
        from: from.display().to_string(),
        to: to.display().to_string(),
        source,
    })?;
    let entries = std::fs::read_dir(from).map_err(|source| WorkflowForkError::Io {
        from: from.display().to_string(),
        to: to.display().to_string(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| WorkflowForkError::Io {
            from: from.display().to_string(),
            to: to.display().to_string(),
            source,
        })?;
        let src_path = entry.path();
        let dst_path = to.join(entry.file_name());
        let file_type = entry.file_type().map_err(|source| WorkflowForkError::Io {
            from: src_path.display().to_string(),
            to: dst_path.display().to_string(),
            source,
        })?;
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|source| WorkflowForkError::Io {
                from: src_path.display().to_string(),
                to: dst_path.display().to_string(),
                source,
            })?;
        }
    }
    Ok(())
}

/// The read-only workflow catalog `GET /v1/workflows` projects (§11.2,
/// Decisions T2-39/T2-40): every root-indexed `status: published` workflow
/// under `root`'s [`WORKFLOW_ROOT`], in the root catalog's own listed order,
/// each paired with its own `index.md` front matter. This never scans
/// [`WORKFLOW_ROOT`] itself for directories — a name the root catalog does
/// not list is never admitted, so drafts (which live under a different tree
/// entirely, `.sergeant/drafts/workflows/`) and unindexed directories are
/// excluded by construction, not by an extra filter.
///
/// A listed name whose `workflow.toml` fails to load, or whose own
/// `index.md` front matter is missing or malformed, is dropped from the
/// result rather than admitted half-described or failing the whole read —
/// this is what "missing/malformed/disagreeing records fail closed" (§19.4)
/// means at catalog scope: one bad entry costs that entry, not the read.
///
/// Returns an empty `Vec` when `root` has no [`ROOT_CATALOG_FILE`] at all —
/// "no admitted catalog here", the caller's cue to fall back to the embedded
/// workflow (§11.2's edge shapes).
pub fn catalog(root: &Path) -> Vec<CatalogEntry> {
    let Ok(text) = std::fs::read_to_string(root.join(ROOT_CATALOG_FILE)) else {
        return Vec::new();
    };
    root_catalog_names(&text)
        .into_iter()
        .filter_map(|name| {
            let definition = WorkflowDefinition::resolve(root, &name).ok()?;
            let front_matter = read_index_front_matter(&workflow_dir(root, &name))?;
            Some(CatalogEntry {
                definition,
                front_matter: Some(front_matter),
            })
        })
        .collect()
}

/// Parse [`ROOT_CATALOG_FILE`]'s Markdown table for the `Workflow` names
/// whose `Status` column reads exactly `published`. `.sergeant/index.md`'s
/// own text (`docs/icm/record-shapes.md` §1 rule 2) is the example shape:
///
/// ```text
/// | Workflow | Status | Index |
/// |---|---|---|
/// | `code-review` | published | [`workflows/code-review/index.md`](...) |
/// ```
///
/// Any line that is not a data row (the header, the separator, prose) simply
/// fails one of the two checks below and is skipped — there is no separate
/// "is this a data row" pass to keep in sync with them.
fn root_catalog_names(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line.trim().split('|').map(str::trim).collect();
            // A data row is `| name | status | index |`, which splits (on
            // the leading/trailing `|`) into `["", name, status, index, ""]`.
            if cells.len() < 4 {
                return None;
            }
            let name = cells[1].trim_matches('`');
            if name.is_empty() || cells[2] != "published" {
                return None;
            }
            Some(name.to_string())
        })
        .collect()
}

/// Read and parse one workflow directory's own `index.md` front matter
/// (`docs/icm/record-shapes.md` §1). `None` on any I/O failure, missing
/// closing delimiter, or missing required field (`status`/`description`) —
/// [`catalog`] treats that the same as the workflow not existing. `pub(crate)`
/// so `sgt doctor`'s edition-drift check (R2 — an existing surface, not a new
/// one) can read a local package's `edition` the same way [`catalog`] does.
pub(crate) fn read_index_front_matter(workflow_dir: &Path) -> Option<WorkflowIndexFrontMatter> {
    let text = std::fs::read_to_string(workflow_dir.join(INDEX_FILE)).ok()?;
    parse_index_front_matter(&text)
}

/// Parse `status`/`description`/`tags`/`edition` out of an `index.md`'s
/// front matter.
///
/// No general YAML parser is pulled in for this: the shape
/// `docs/icm/record-shapes.md` §1 normatively fixes is fixed and narrow — a
/// `---`-delimited block of `key: value` lines, an optional `>-`/`|-` folded
/// or literal block scalar for `description`, and an optional `- item` list
/// for `tags` — so a parser scoped to exactly that shape is the tiny local
/// composition R6 asks for over a heavyweight dependency for three known
/// fields. `kind`, `name`, and `version` are deliberately not read here:
/// [`CatalogEntry`] takes `name`/`version` from `workflow.toml` (§11.2's
/// table), and `kind`/`name` agreement is `docs/icm/record-shapes.md`'s own
/// authoring-time invariant, not something this read-only projection checks.
fn parse_index_front_matter(text: &str) -> Option<WorkflowIndexFrontMatter> {
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut status: Option<String> = None;
    let mut description: Option<String> = None;
    let mut tags: Option<Vec<String>> = None;
    let mut edition: Option<String> = None;
    let mut in_description_block = false;
    let mut in_tags_block = false;
    for line in lines {
        if line.trim() == "---" {
            return Some(WorkflowIndexFrontMatter {
                status: status?,
                description: description?,
                tags,
                edition,
            });
        }
        if in_description_block && (line.starts_with(' ') || line.starts_with('\t')) {
            let text = line.trim();
            if !text.is_empty()
                && let Some(d) = &mut description
            {
                if !d.is_empty() {
                    d.push(' ');
                }
                d.push_str(text);
            }
            continue;
        }
        in_description_block = false;
        if in_tags_block {
            let trimmed = line.trim_start();
            if let Some(item) = trimmed.strip_prefix("- ") {
                if let Some(tags) = &mut tags {
                    tags.push(item.trim().trim_matches('"').to_string());
                }
                continue;
            }
            in_tags_block = false;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "status" => status = Some(value.trim_matches('"').to_string()),
            "edition" => edition = Some(value.trim_matches('"').to_string()),
            "description" => {
                if matches!(value, ">-" | ">" | "|-" | "|") {
                    in_description_block = true;
                    description = Some(String::new());
                } else {
                    description = Some(value.trim_matches('"').to_string());
                }
            }
            "tags" => {
                if value.is_empty() {
                    in_tags_block = true;
                    tags = Some(Vec::new());
                } else {
                    tags = Some(
                        value
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    );
                }
            }
            _ => {}
        }
    }
    None
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

/// One stage's resolved executor tag (§12.2/§12.3): everything
/// [`resolve_stage_tag`] can determine about a stage before its `CONTEXT.md`
/// is read.
struct ResolvedStageTag {
    kind: StageKind,
    harness: Option<String>,
    profile: Option<String>,
    requires_ask: bool,
    execute: Option<ExecuteSpec>,
}

/// Resolve one stage's `[stage."<id>"]` table, if it declared one, into its
/// executor tag. No table (the legacy shape, §12.1) and a table with no
/// `kind` both mean `Actor` with no explicit harness/profile — the same
/// actor-default outcome either way. An explicit `kind` other than `"actor"`
/// or `"execute"` fails closed (§22.3).
///
/// Once the kind is known, every field belonging to the *other* kind is
/// refused rather than silently ignored (§22.3's "actor-only fields on
/// execute stages and execute-only fields on actor stages are rejected"),
/// and an execute stage's required fields (`image`, `command`, `workdir`,
/// `workspace_access`, `network`) are checked for presence one at a time so
/// the error names exactly what is missing.
fn resolve_stage_tag(
    id: &str,
    table: Option<&StageTable>,
    path: &str,
) -> Result<ResolvedStageTag, WorkflowError> {
    let Some(table) = table else {
        return Ok(ResolvedStageTag {
            kind: StageKind::Actor,
            harness: None,
            profile: None,
            requires_ask: false,
            execute: None,
        });
    };
    let kind = match table.kind.as_deref() {
        None | Some("actor") => StageKind::Actor,
        Some("execute") => StageKind::Execute,
        Some(other) => {
            return Err(WorkflowError::UnknownStageKind {
                path: path.to_string(),
                stage: id.to_string(),
                kind: other.to_string(),
            });
        }
    };
    let execute_field_on_actor = |field: &str| WorkflowError::ExecuteFieldOnActorStage {
        path: path.to_string(),
        stage: id.to_string(),
        field: field.to_string(),
    };
    let actor_field_on_execute = |field: &str| WorkflowError::ActorFieldOnExecuteStage {
        path: path.to_string(),
        stage: id.to_string(),
        field: field.to_string(),
    };
    match kind {
        StageKind::Actor => {
            if table.image.is_some() {
                return Err(execute_field_on_actor("image"));
            }
            if table.command.is_some() {
                return Err(execute_field_on_actor("command"));
            }
            if table.workdir.is_some() {
                return Err(execute_field_on_actor("workdir"));
            }
            if table.workspace_access.is_some() {
                return Err(execute_field_on_actor("workspace_access"));
            }
            if table.network.is_some() {
                return Err(execute_field_on_actor("network"));
            }
            if table.env.is_some() {
                return Err(execute_field_on_actor("env"));
            }
            Ok(ResolvedStageTag {
                kind,
                harness: table.harness.clone(),
                profile: table.profile.clone(),
                requires_ask: table.requires_ask,
                execute: None,
            })
        }
        StageKind::Execute => {
            if table.harness.is_some() {
                return Err(actor_field_on_execute("harness"));
            }
            if table.profile.is_some() {
                return Err(actor_field_on_execute("profile"));
            }
            if table.requires_ask {
                return Err(actor_field_on_execute("requires_ask"));
            }
            let missing = |field: &str| WorkflowError::MissingExecuteField {
                path: path.to_string(),
                stage: id.to_string(),
                field: field.to_string(),
            };
            let image = table.image.clone().ok_or_else(|| missing("image"))?;
            let command = table.command.clone().ok_or_else(|| missing("command"))?;
            if command.is_empty() {
                return Err(missing("command"));
            }
            let workdir = table.workdir.clone().ok_or_else(|| missing("workdir"))?;
            let workspace_access = match table.workspace_access.as_deref() {
                Some("read_only") => WorkspaceAccess::ReadOnly,
                Some("read_write") => WorkspaceAccess::ReadWrite,
                Some(other) => {
                    return Err(WorkflowError::UnknownWorkspaceAccess {
                        path: path.to_string(),
                        stage: id.to_string(),
                        value: other.to_string(),
                    });
                }
                None => return Err(missing("workspace_access")),
            };
            let network = match table.network.as_deref() {
                Some("none") => NetworkPolicy::None,
                Some(other) => {
                    return Err(WorkflowError::UnknownNetworkPolicy {
                        path: path.to_string(),
                        stage: id.to_string(),
                        value: other.to_string(),
                    });
                }
                None => return Err(missing("network")),
            };
            let env = table.env.clone().unwrap_or_default();
            Ok(ResolvedStageTag {
                kind,
                harness: None,
                profile: None,
                requires_ask: false,
                execute: Some(ExecuteSpec {
                    image,
                    command,
                    workdir,
                    workspace_access,
                    network,
                    env,
                }),
            })
        }
    }
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
    /// The pinned container spec (N4, §12.3): image, command, workdir,
    /// access and network policy, and env are all execution-relevant —
    /// changing any of them must change the hash the same way editing a
    /// stage's `CONTEXT.md` does.
    execute: Option<&'a ExecuteSpec>,
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
                execute: s.execute.as_ref(),
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

    /// §6 Phase 3 (R4): a local fork under `.sergeant/local/workflows/<name>/`
    /// must resolve over a same-named stock package under
    /// `.sergeant/workflows/<name>/` — directory precedence, not composition.
    /// guard-map: swapping the precedence order in `resolve` (checking stock
    /// before local) survives every stock-only test but fails this one.
    #[test]
    fn local_shadows_stock() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();

        let stock = workflow_dir(root, "example");
        std::fs::create_dir_all(stock.join("00-only")).expect("stage dir");
        std::fs::write(
            stock.join(WORKFLOW_FILE),
            "[workflow]\nname = \"example\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
        )
        .expect("descriptor");
        std::fs::write(stock.join("00-only").join(CONTEXT_FILE), "stock").expect("context");

        let local = local_workflow_dir(root, "example");
        std::fs::create_dir_all(local.join("00-only")).expect("stage dir");
        std::fs::write(
            local.join(WORKFLOW_FILE),
            "[workflow]\nname = \"example\"\nversion = \"2\"\nstages = [\"00-only\"]\n",
        )
        .expect("descriptor");
        std::fs::write(local.join("00-only").join(CONTEXT_FILE), "local").expect("context");

        let workflow = WorkflowDefinition::resolve(root, "example").expect("resolve");
        assert_eq!(workflow.version, "2", "local must win over stock");
        assert_eq!(workflow.stages[0].context, "local");
    }

    /// §6 Phase 3 (R4): with no local fork, resolution falls back to the
    /// stock package of the same name, unchanged from before local forks
    /// existed at all.
    /// guard-map: making resolution require a local package (or skip the
    /// stock fallback) survives `local_shadows_stock` but fails this one.
    #[test]
    fn resolution_falls_back_to_stock_when_no_local_fork_exists() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();

        let stock = workflow_dir(root, "example");
        std::fs::create_dir_all(stock.join("00-only")).expect("stage dir");
        std::fs::write(
            stock.join(WORKFLOW_FILE),
            "[workflow]\nname = \"example\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
        )
        .expect("descriptor");
        std::fs::write(stock.join("00-only").join(CONTEXT_FILE), "stock").expect("context");

        let workflow = WorkflowDefinition::resolve(root, "example").expect("resolve");
        assert_eq!(workflow.version, "1");
        assert_eq!(workflow.stages[0].context, "stock");
    }

    /// §6 Phase 3 (R7): `fork` copies a stock package to
    /// `.sergeant/local/workflows/<name>/`, preserving the `edition` marker
    /// its `index.md` carried — the fork stays comparable via ADR 0016's
    /// string comparison from the moment it exists.
    /// guard-map: a `fork` that rewrites `edition` (or drops `index.md`
    /// entirely) survives every other fork test but fails this one.
    #[test]
    fn fork_copies_stock_preserving_the_edition_marker() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();

        let stock = workflow_dir(root, "example");
        std::fs::create_dir_all(stock.join("00-only")).expect("stage dir");
        std::fs::write(
            stock.join(WORKFLOW_FILE),
            "[workflow]\nname = \"example\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
        )
        .expect("descriptor");
        std::fs::write(stock.join("00-only").join(CONTEXT_FILE), "stock").expect("context");
        std::fs::write(
            stock.join(INDEX_FILE),
            "---\nstatus: published\ndescription: an example\nedition: 0.1.0\n---\n",
        )
        .expect("index.md");

        let forked = fork(root, "example").expect("fork");
        assert_eq!(forked, local_workflow_dir(root, "example"));
        assert!(forked.join(WORKFLOW_FILE).is_file());
        assert!(forked.join("00-only").join(CONTEXT_FILE).is_file());
        let front_matter = read_index_front_matter(&forked).expect("index.md front matter");
        assert_eq!(front_matter.edition.as_deref(), Some("0.1.0"));

        let resolved = WorkflowDefinition::resolve(root, "example").expect("resolve");
        assert_eq!(resolved.version, "1", "the fork must now be what resolves");
    }

    /// §6 Phase 3 (R7): `fork` refuses to overwrite an existing local
    /// package rather than silently clobbering a user's already-rewritten
    /// fork.
    /// guard-map: dropping the pre-existence check survives every other fork
    /// test but fails this one (it would instead attempt — and possibly
    /// partially succeed at — copying over the existing local files).
    #[test]
    fn fork_refuses_to_overwrite_an_existing_local_package() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();

        let stock = workflow_dir(root, "example");
        std::fs::create_dir_all(stock.join("00-only")).expect("stage dir");
        std::fs::write(
            stock.join(WORKFLOW_FILE),
            "[workflow]\nname = \"example\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
        )
        .expect("descriptor");
        std::fs::write(stock.join("00-only").join(CONTEXT_FILE), "stock").expect("context");

        let local = local_workflow_dir(root, "example");
        std::fs::create_dir_all(&local).expect("local dir");
        std::fs::write(local.join("sentinel"), "user work, do not touch").expect("sentinel");

        let err = fork(root, "example").expect_err("must refuse");
        assert!(
            matches!(err, WorkflowForkError::LocalAlreadyExists { .. }),
            "{err}"
        );
        assert!(
            local.join("sentinel").is_file(),
            "the user's existing local package must be untouched"
        );
        assert!(!local.join(WORKFLOW_FILE).is_file());
    }

    /// §6 Phase 3 (R7): `fork` refuses when there is no stock package of
    /// that name to fork from at all.
    #[test]
    fn fork_refuses_when_no_stock_package_exists() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let err = fork(root, "ghost").expect_err("must refuse");
        assert!(
            matches!(err, WorkflowForkError::NoStockPackage { .. }),
            "{err}"
        );
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
                // N4 activates "execute" as legal vocabulary (below); this
                // probes a kind that stays illegal even now.
                "kind = \"shell\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&wf, "00-only", "context");

        let err = WorkflowDefinition::resolve(root, "bad-kind").expect_err("must refuse");
        assert!(
            matches!(&err, WorkflowError::UnknownStageKind { stage, kind, .. }
                if stage == "00-only" && kind == "shell"),
            "expected an unknown-kind refusal naming the offending stage and kind, got {err}"
        );
    }

    /// N4 (§12.3, §11.2): a well-formed `kind = "execute"` table parses into
    /// a pinned [`ExecuteSpec`], `CONTEXT.md` is optional for it (§12.4), and
    /// the resolved stage carries no actor fields.
    #[test]
    fn a_well_formed_execute_stage_parses_with_no_context_md_required() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "validate");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"validate\"\n",
                "version = \"1\"\n",
                "stages = [\"00-prepare\", \"10-validate\"]\n",
                "\n",
                "[stage.\"10-validate\"]\n",
                "kind = \"execute\"\n",
                "image = \"python:3.13-slim\"\n",
                "command = [\"python\", \"validate.py\"]\n",
                "workdir = \"/estate\"\n",
                "workspace_access = \"read_write\"\n",
                "network = \"none\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&wf, "00-prepare", "prepare context");
        // Deliberately no CONTEXT.md for 10-validate: §12.4 says it is
        // optional for execute stages, so this must not need `write_stage`.
        std::fs::create_dir_all(wf.join("10-validate")).expect("execute stage dir");

        let workflow = WorkflowDefinition::resolve(root, "validate").expect("resolve");
        let execute_stage = &workflow.stages[1];
        assert_eq!(execute_stage.kind, StageKind::Execute);
        assert_eq!(execute_stage.harness, None);
        assert_eq!(execute_stage.profile, None);
        assert!(!execute_stage.requires_ask);
        assert_eq!(execute_stage.context, "", "no CONTEXT.md/README.md present");
        let spec = execute_stage
            .execute
            .as_ref()
            .expect("execute stage must carry an ExecuteSpec");
        assert_eq!(spec.image, "python:3.13-slim");
        assert_eq!(
            spec.command,
            vec!["python".to_string(), "validate.py".to_string()]
        );
        assert_eq!(spec.workdir, "/estate");
        assert_eq!(spec.workspace_access, WorkspaceAccess::ReadWrite);
        assert_eq!(spec.network, NetworkPolicy::None);
        assert!(spec.env.is_empty());

        // The untouched actor stage still has no execute spec.
        assert_eq!(workflow.stages[0].execute, None);
    }

    /// §12.4: when an execute stage's `CONTEXT.md` is absent but a
    /// `README.md` exists, the README is carried as (undocumented,
    /// unconsumed) human-readable context instead.
    #[test]
    fn an_execute_stage_falls_back_to_readme_when_context_md_is_absent() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "readme-fallback");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"readme-fallback\"\n",
                "version = \"1\"\n",
                "stages = [\"00-validate\"]\n",
                "\n",
                "[stage.\"00-validate\"]\n",
                "kind = \"execute\"\n",
                "image = \"alpine:3\"\n",
                "command = [\"true\"]\n",
                "workdir = \"/estate\"\n",
                "workspace_access = \"read_only\"\n",
                "network = \"none\"\n",
            ),
        )
        .expect("descriptor");
        std::fs::create_dir_all(wf.join("00-validate")).expect("stage dir");
        std::fs::write(
            wf.join("00-validate").join("README.md"),
            "explains the check",
        )
        .expect("readme");

        let workflow = WorkflowDefinition::resolve(root, "readme-fallback").expect("resolve");
        assert_eq!(workflow.stages[0].context, "explains the check");
    }

    /// §12.3/§22.3: each required execute field, missing one at a time,
    /// fails closed and names the missing field rather than a generic
    /// malformed-file error.
    #[test]
    fn a_missing_required_execute_field_is_named_in_the_refusal() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let cases: &[(&str, &str)] = &[
            (
                "no-image",
                concat!(
                    "command = [\"true\"]\n",
                    "workdir = \"/estate\"\n",
                    "workspace_access = \"read_only\"\n",
                    "network = \"none\"\n"
                ),
            ),
            (
                "no-command",
                concat!(
                    "image = \"alpine:3\"\n",
                    "workdir = \"/estate\"\n",
                    "workspace_access = \"read_only\"\n",
                    "network = \"none\"\n"
                ),
            ),
            (
                "no-workdir",
                concat!(
                    "image = \"alpine:3\"\n",
                    "command = [\"true\"]\n",
                    "workspace_access = \"read_only\"\n",
                    "network = \"none\"\n"
                ),
            ),
            (
                "no-access",
                concat!(
                    "image = \"alpine:3\"\n",
                    "command = [\"true\"]\n",
                    "workdir = \"/estate\"\n",
                    "network = \"none\"\n"
                ),
            ),
            (
                "no-network",
                concat!(
                    "image = \"alpine:3\"\n",
                    "command = [\"true\"]\n",
                    "workdir = \"/estate\"\n",
                    "workspace_access = \"read_only\"\n"
                ),
            ),
        ];
        let expected_field: &[&str] =
            &["image", "command", "workdir", "workspace_access", "network"];
        for ((name, fields), expected) in cases.iter().zip(expected_field) {
            let wf = workflow_dir(root, name);
            std::fs::create_dir_all(&wf).expect("workflow dir");
            std::fs::write(
                wf.join(WORKFLOW_FILE),
                format!(
                    "[workflow]\nname = {name:?}\nversion = \"1\"\nstages = [\"00-only\"]\n\n[stage.\"00-only\"]\nkind = \"execute\"\n{fields}"
                ),
            )
            .expect("descriptor");
            std::fs::create_dir_all(wf.join("00-only")).expect("stage dir");

            let err = WorkflowDefinition::resolve(root, name).expect_err("must refuse");
            assert!(
                matches!(&err, WorkflowError::MissingExecuteField { field, .. } if field == expected),
                "case {name}: expected MissingExecuteField({expected:?}), got {err}"
            );
        }
    }

    /// §16.6/§16.7: an unrecognized `workspace_access` or `network` value
    /// fails closed by name rather than being coerced or silently ignored.
    #[test]
    fn unknown_workspace_access_and_network_values_fail_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();

        let wf = workflow_dir(root, "bad-access");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\nname = \"bad-access\"\nversion = \"1\"\nstages = [\"00-only\"]\n\n",
                "[stage.\"00-only\"]\nkind = \"execute\"\nimage = \"alpine:3\"\n",
                "command = [\"true\"]\nworkdir = \"/estate\"\n",
                "workspace_access = \"read_and_write_and_more\"\nnetwork = \"none\"\n",
            ),
        )
        .expect("descriptor");
        std::fs::create_dir_all(wf.join("00-only")).expect("stage dir");
        assert!(matches!(
            WorkflowDefinition::resolve(root, "bad-access"),
            Err(WorkflowError::UnknownWorkspaceAccess { value, .. }) if value == "read_and_write_and_more"
        ));

        let wf = workflow_dir(root, "bad-network");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\nname = \"bad-network\"\nversion = \"1\"\nstages = [\"00-only\"]\n\n",
                "[stage.\"00-only\"]\nkind = \"execute\"\nimage = \"alpine:3\"\n",
                "command = [\"true\"]\nworkdir = \"/estate\"\n",
                "workspace_access = \"read_only\"\nnetwork = \"bridge\"\n",
            ),
        )
        .expect("descriptor");
        std::fs::create_dir_all(wf.join("00-only")).expect("stage dir");
        assert!(matches!(
            WorkflowDefinition::resolve(root, "bad-network"),
            Err(WorkflowError::UnknownNetworkPolicy { value, .. }) if value == "bridge"
        ));
    }

    /// §22.3: an actor-only field (`harness`, `profile`, `requires_ask`) on
    /// a `kind = "execute"` table is refused by name — an execute stage has
    /// no harness to select and cannot ask.
    #[test]
    fn actor_only_fields_on_an_execute_stage_fail_closed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        let wf = workflow_dir(root, "misplaced-actor-field");
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\nname = \"misplaced-actor-field\"\nversion = \"1\"\n",
                "stages = [\"00-only\"]\n\n",
                "[stage.\"00-only\"]\nkind = \"execute\"\nimage = \"alpine:3\"\n",
                "command = [\"true\"]\nworkdir = \"/estate\"\n",
                "workspace_access = \"read_only\"\nnetwork = \"none\"\nharness = \"claude\"\n",
            ),
        )
        .expect("descriptor");
        std::fs::create_dir_all(wf.join("00-only")).expect("stage dir");

        let err = WorkflowDefinition::resolve(root, "misplaced-actor-field").expect_err("refuse");
        assert!(matches!(
            &err,
            WorkflowError::ActorFieldOnExecuteStage { field, .. } if field == "harness"
        ));
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

        let err = WorkflowDefinition::resolve(root, "misplaced").expect_err("must refuse");
        assert!(
            matches!(&err, WorkflowError::ExecuteFieldOnActorStage { field, .. } if field == "image"),
            "expected an ExecuteFieldOnActorStage refusal naming \"image\", got {err}"
        );
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

        // TH-06 (MVP-2 D3 fixer pass): an `ExecuteSpec` field must
        // participate in the hash too — the commit that added `execute`'s
        // participation (`compute_content_hash`'s `execute: s.execute.as_ref()`)
        // had no test that would fail if that line were deleted; every
        // pre-existing case above varies an actor-stage field, never an
        // execute stage's. Two otherwise-identical workflows differing only
        // in an execute stage's pinned `image` must hash differently.
        let execute_base_root = tempfile::TempDir::new().expect("tempdir");
        let execute_base = workflow_dir(execute_base_root.path(), "base");
        std::fs::create_dir_all(&execute_base).expect("workflow dir");
        std::fs::write(
            execute_base.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"base\"\n",
                "version = \"1\"\n",
                "stages = [\"00-a\", \"20-x\"]\n",
                "\n",
                "[stage.\"20-x\"]\n",
                "kind = \"execute\"\n",
                "image = \"alpine:3\"\n",
                "command = [\"true\"]\n",
                "workdir = \"/estate\"\n",
                "workspace_access = \"read_only\"\n",
                "network = \"none\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&execute_base, "00-a", "context a");
        let execute_baseline =
            WorkflowDefinition::load_dir(&execute_base).expect("resolve execute baseline");

        let execute_changed_root = tempfile::TempDir::new().expect("tempdir");
        let execute_changed = workflow_dir(execute_changed_root.path(), "base");
        std::fs::create_dir_all(&execute_changed).expect("workflow dir");
        std::fs::write(
            execute_changed.join(WORKFLOW_FILE),
            concat!(
                "[workflow]\n",
                "name = \"base\"\n",
                "version = \"1\"\n",
                "stages = [\"00-a\", \"20-x\"]\n",
                "\n",
                "[stage.\"20-x\"]\n",
                "kind = \"execute\"\n",
                "image = \"alpine:3.19\"\n", // the only difference from the baseline above
                "command = [\"true\"]\n",
                "workdir = \"/estate\"\n",
                "workspace_access = \"read_only\"\n",
                "network = \"none\"\n",
            ),
        )
        .expect("descriptor");
        write_stage(&execute_changed, "00-a", "context a");
        let execute_changed_def =
            WorkflowDefinition::load_dir(&execute_changed).expect("resolve execute changed");
        assert_ne!(
            execute_baseline.content_hash, execute_changed_def.content_hash,
            "an execute stage's pinned image must participate in content_hash: editing it is a \
             content change, exactly like editing an actor stage's context"
        );
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

    // --------------------------------------------- T2: catalog / front matter

    /// The concrete shape `docs/icm/record-shapes.md` §1 gives as its
    /// canonical example: a folded `>-` description and a bulleted `tags`
    /// list, both parsed out alongside the plain `status` scalar.
    #[test]
    fn front_matter_parses_the_documented_shape() {
        let text = concat!(
            "---\n",
            "kind: workflow\n",
            "name: diagnose-bug\n",
            "status: published\n",
            "version: 3\n",
            "description: >-\n",
            "  Reproduce, isolate, prove, remediate and verify a defect.\n",
            "tags:\n",
            "  - debugging\n",
            "  - defect\n",
            "  - investigation\n",
            "---\n",
            "\n",
            "# Diagnose Bug\n",
        );
        let fm = parse_index_front_matter(text).expect("parses");
        assert_eq!(fm.status, "published");
        assert_eq!(
            fm.description,
            "Reproduce, isolate, prove, remediate and verify a defect."
        );
        assert_eq!(
            fm.tags,
            Some(vec![
                "debugging".to_string(),
                "defect".to_string(),
                "investigation".to_string()
            ])
        );
    }

    /// A workflow whose front matter never declares `tags:` gets `None`, not
    /// an empty `Vec` standing in for "none" (§11.2's CatalogEntry table).
    #[test]
    fn front_matter_with_no_tags_key_is_none_not_an_empty_list() {
        let text = concat!(
            "---\n",
            "kind: workflow\n",
            "name: implement\n",
            "status: published\n",
            "version: 2\n",
            "description: A one-line description.\n",
            "---\n",
        );
        let fm = parse_index_front_matter(text).expect("parses");
        assert_eq!(fm.description, "A one-line description.");
        assert_eq!(fm.tags, None);
    }

    /// Missing the closing delimiter, or missing a required field, both fail
    /// closed rather than returning a partially-filled record.
    #[test]
    fn front_matter_fails_closed_when_malformed_or_incomplete() {
        assert!(parse_index_front_matter("status: published\n").is_none());
        assert!(parse_index_front_matter("---\nstatus: published\n").is_none());
        assert!(
            parse_index_front_matter("---\ndescription: only a description\n---\n").is_none(),
            "status is required too"
        );
    }

    /// The root catalog's own documented table shape: only `published` rows
    /// contribute a name, and the header/separator rows are silently skipped
    /// as a side effect of not matching a data row — not by special-casing
    /// their literal text.
    #[test]
    fn root_catalog_names_keeps_only_published_rows_in_order() {
        let text = concat!(
            "# Sergeant workflow catalog\n",
            "\n",
            "| Workflow | Status | Index |\n",
            "|---|---|---|\n",
            "| `code-review` | published | [`workflows/code-review/index.md`](workflows/code-review/index.md) |\n",
            "| `some-draft` | draft | [`workflows/some-draft/index.md`](workflows/some-draft/index.md) |\n",
            "| `implement` | published | [`workflows/implement/index.md`](workflows/implement/index.md) |\n",
        );
        assert_eq!(
            root_catalog_names(text),
            vec!["code-review".to_string(), "implement".to_string()]
        );
    }

    /// Writes a minimal admitted workflow (`workflow.toml` + one stage +
    /// `index.md`) under `root`'s `.sergeant/workflows/<name>/`.
    fn write_catalog_workflow(root: &Path, name: &str, status: &str) {
        let wf = workflow_dir(root, name);
        std::fs::create_dir_all(&wf).expect("workflow dir");
        std::fs::write(
            wf.join(WORKFLOW_FILE),
            format!("[workflow]\nname = \"{name}\"\nversion = \"1\"\nstages = [\"00-only\"]\n"),
        )
        .expect("descriptor");
        write_stage(&wf, "00-only", "do the thing");
        std::fs::write(
            wf.join(INDEX_FILE),
            format!(
                "---\nkind: workflow\nname: {name}\nstatus: {status}\nversion: 1\n\
                 description: A one-line description of {name}.\ntags:\n  - t\n---\n"
            ),
        )
        .expect("index.md");
    }

    /// Writes the root catalog naming exactly `names` as published.
    fn write_root_catalog(root: &Path, names: &[&str]) {
        let mut body = String::from(
            "# Sergeant workflow catalog\n\n| Workflow | Status | Index |\n|---|---|---|\n",
        );
        for name in names {
            body.push_str(&format!(
                "| `{name}` | published | [`workflows/{name}/index.md`](workflows/{name}/index.md) |\n"
            ));
        }
        std::fs::create_dir_all(root.join(".sergeant")).expect(".sergeant dir");
        std::fs::write(root.join(ROOT_CATALOG_FILE), body).expect("root catalog");
    }

    /// The end-to-end happy path: a root-indexed, published workflow with a
    /// well-formed `index.md` comes back as one fully-described entry.
    #[test]
    fn catalog_returns_a_root_indexed_published_workflow_fully_described() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        write_catalog_workflow(root, "implement", "published");
        write_root_catalog(root, &["implement"]);

        let entries = catalog(root);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].definition.name, "implement");
        let fm = entries[0].front_matter.as_ref().expect("front matter");
        assert_eq!(fm.status, "published");
        assert_eq!(fm.tags, Some(vec!["t".to_string()]));
    }

    /// A workflow directory that exists on disk but is never named in the
    /// root catalog is excluded — the catalog only ever walks names the root
    /// index gives it, never the directory itself (§19.4 "unindexed
    /// directory excluded").
    #[test]
    fn catalog_excludes_an_unindexed_directory() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        write_catalog_workflow(root, "implement", "published");
        write_catalog_workflow(root, "shadow", "published");
        write_root_catalog(root, &["implement"]); // "shadow" deliberately omitted

        let entries = catalog(root);
        let names: Vec<&str> = entries.iter().map(|e| e.definition.name.as_str()).collect();
        assert_eq!(names, vec!["implement"]);
    }

    /// A row the root catalog itself marks `draft` never contributes a name
    /// to resolve, so it cannot appear regardless of what sits on disk
    /// (§19.4 "drafts excluded").
    #[test]
    fn catalog_excludes_a_row_the_root_catalog_marks_draft() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        write_catalog_workflow(root, "implement", "published");
        write_catalog_workflow(root, "half-baked", "draft");
        std::fs::create_dir_all(root.join(".sergeant")).expect(".sergeant dir");
        std::fs::write(
            root.join(ROOT_CATALOG_FILE),
            "| Workflow | Status | Index |\n|---|---|---|\n\
             | `implement` | published | [x](x) |\n\
             | `half-baked` | draft | [x](x) |\n",
        )
        .expect("root catalog");

        let entries = catalog(root);
        let names: Vec<&str> = entries.iter().map(|e| e.definition.name.as_str()).collect();
        assert_eq!(names, vec!["implement"]);
    }

    /// A name the root catalog lists as published, but whose own `index.md`
    /// cannot be parsed, is dropped from the catalog rather than admitted
    /// half-described (§19.4 "missing/malformed/disagreeing records fail
    /// closed").
    #[test]
    fn catalog_drops_an_entry_with_a_malformed_index_md() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        write_catalog_workflow(root, "implement", "published");
        // A second, well-formed workflow whose index.md is then corrupted.
        write_catalog_workflow(root, "broken", "published");
        std::fs::write(
            workflow_dir(root, "broken").join(INDEX_FILE),
            "not front matter at all",
        )
        .expect("corrupt index.md");
        write_root_catalog(root, &["implement", "broken"]);

        let entries = catalog(root);
        let names: Vec<&str> = entries.iter().map(|e| e.definition.name.as_str()).collect();
        assert_eq!(names, vec!["implement"]);
    }

    /// No `.sergeant/index.md` at all under `root` is "no admitted catalog
    /// here" — an empty `Vec`, not a panic or an error, the caller's cue to
    /// fall back to the embedded workflow.
    #[test]
    fn catalog_is_empty_with_no_root_catalog_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        assert!(catalog(dir.path()).is_empty());
    }
}

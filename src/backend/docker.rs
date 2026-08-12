//! Docker executor: the `"docker"` backend that runs `kind = "execute"`
//! stages through the user's local Docker Engine (N4, proposal §16).
//!
//! ## Why this is a [`Backend`] impl, not a parallel subsystem
//!
//! §13.2 offers a conceptual `ContainerRuntime` contract distinct from the
//! harness-adapter one, but names it "a conceptual contract, not mandated
//! Rust spelling." Measured against what N3 already shipped, the existing
//! `Backend` trait's five effectful verbs are *already* §11.2's execute-stage
//! lifecycle, verb for verb:
//!
//! ```text
//! PREPARE   reserve a deterministic container name — no external effect
//! LAUNCH    resolve the image, create, start                 (§15.1)
//! OBSERVE   inspect: running, or exited with a mechanical exit-code mapping (§15.2)
//! INTERRUPT docker stop the exact owned container              (§15.5)
//! STOP      docker rm the exact owned container                (§16.12)
//! RESUME    re-adopt by deterministic name + label after a restart (§16.11)
//! ```
//!
//! Implementing `DockerBackend: Backend` and registering it under the fixed
//! name [`DOCKER_BACKEND_NAME`] means every piece of already-tested machinery
//! this milestone would otherwise have to rebuild comes free: the two-phase
//! PREPARE(under lock)/LAUNCH(outside lock) boundary and its
//! `execution.reserved` crash window (§14.2, §22.5), the core-lock discipline
//! instrumentation (§22.6), cancel/retry/recovery through the same
//! `Engine::settle_launch`/`settle_observe`/`settle_interrupt` "performer"
//! pattern the Claude adapter rides, and the generic
//! `BackendSignal → stage transition` fold that already knows nothing about
//! any particular executor. `runtime::engine` needed exactly one addition to
//! use it for execute stages: routing an execute stage to the fixed
//! `"docker"` backend instead of an author-chosen harness
//! (`Engine::bind_stages`), and carrying the stage's pinned [`ExecuteSpec`]
//! through [`StartRequest::execute`]. Nothing else in the engine's stage
//! state machine changed.
//!
//! ## What this adapter shells out to, and why not `bollard`
//!
//! §16.1 nominates `bollard` as "a nominated implementation choice, not a
//! permanent architectural dependency... If measurement finds a correctness
//! gap in Bollard, Sergeant may use a smaller direct client without changing
//! workflow semantics." This adapter instead shells out to the `docker` CLI
//! via [`std::process::Command`] — the same primitive `backend::claude`
//! already uses for its own external process, so this module adds no new
//! process-execution idiom to review, and no new async-runtime dependency to
//! an otherwise-synchronous trait. The CLI's `--format`/`inspect` JSON output
//! is exactly the Engine API's own response shape (`docker inspect` prints
//! the container/image inspect JSON verbatim), so this is "structured request
//! and response types" over a slightly different transport, not a return to
//! output-scraping. Measured on Cerberus (`docs/environments/cerberus.md`):
//! Docker 29.7.2, the invoking user in the `docker` group (no sudo), registry
//! pulls, digest-pinned pulls, bind-mount rw, `--network=none`, and exact
//! image removal all green.
//!
//! ## Scoped for MVP-2, stated rather than hidden
//!
//! - **Local Engine only** (§16.2): every call is the bare `docker` binary
//!   with no `-H`/`DOCKER_HOST` override, which resolves through the local
//!   CLI's own default context. A context pointed at a remote endpoint is not
//!   independently probed and refused here — that is `docker`'s own context
//!   configuration, outside this adapter's reach without adding a manifest
//!   inspection this milestone has no evidence to specify. Recorded as an
//!   honest gap, not a claim of enforcement.
//! - **Registry auth** (§16.5): public images pull automatically; an
//!   already-present private image runs by its local id; a missing private
//!   image fails closed with the pull's own stderr as evidence. No credential
//!   helper integration.
//! - **Image pin durability across a daemon restart** (§15.3): the first
//!   successful resolution for a `(work_id, stage_id)` pair is pinned to a
//!   small file under `<data_dir>/docker-adapter/image-pins/`, not inside the
//!   journal — see [`DockerBackend::pin_path`]'s doc for why. It is therefore
//!   durable across a restart of *this* daemon on *this* data dir (the
//!   acceptance shape §22.7 test 8 actually exercises), but is adapter-side
//!   evidence, not a journaled fact a projection can query. The `journal`
//!   still gets the resolved identity: [`DockerBackend::emit_image_resolved`]
//!   pushes it through the same [`EventSink`] `backend::claude` uses for its
//!   normalized events, so it is durable, projectable evidence too — just not
//!   the retry-pin's *source of truth*.
//! - **`network` accepts only `"none"`** (§16.7, enforced already at parse
//!   time in `domain::workflow`) — nothing to do here but honor it.
//! - **No timeout** (§15.6's explicit non-goal): a stage's container runs
//!   until it exits or an operator cancels.
//! - **§16.3's full lifecycle probe** (bind-mount round trip with a marker
//!   file) is [`DockerBackend::lifecycle_probe`], called by `sgt doctor`
//!   (§17.3: "a live Docker probe is not optional because container
//!   lifecycle itself is deterministic and token-free"). [`Backend::probe`]
//!   itself stays cheap (`docker version`) because `Engine::bind_stages`
//!   calls it on every submission touching an execute stage, and a container
//!   round trip on every submission would cost exactly what the "warm cache"
//!   comment beside `bind_stages` exists to avoid.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{
    Backend, BackendError, Capabilities, Completion, EventSink, ExecutionHandle, NativeEvent,
    NativeState, Observation, PreparedExecution, ProbeReport, ResumeRequest, RuntimeScope,
    StartRequest,
};
use crate::domain::event::{EventDraft, EventSource};
use crate::domain::workflow::{ExecuteSpec, NetworkPolicy, ResolvedImage, WorkspaceAccess};
use crate::runtime::blob::{BlobRef, BlobStore};

/// Name the daemon registers this backend under, and the fixed value every
/// `kind = "execute"` stage routes to (`runtime::engine::bind_stages`).
pub const DOCKER_BACKEND_NAME: &str = "docker";

/// Label key: every container this adapter creates carries `true` here
/// (§16.10). Doctor and cleanup sweeps filter on this label, never on the
/// name prefix alone.
pub const LABEL_MANAGED: &str = "io.sergeant.managed";
/// Label key: sergeant's execution id. The identity check every verb below
/// runs before it touches a container by name.
pub const LABEL_EXECUTION: &str = "io.sergeant.execution";
/// Label key: the Work this execution serves.
pub const LABEL_WORK: &str = "io.sergeant.work";
/// Label key: the stage id.
pub const LABEL_STAGE: &str = "io.sergeant.stage";
/// Label key: the attempt number.
pub const LABEL_ATTEMPT: &str = "io.sergeant.attempt";
/// Label key: the labeling scheme's own version, so a future revision can
/// tell an old-shape container apart from a new one during recovery.
pub const LABEL_SCHEMA: &str = "io.sergeant.schema";
/// Value [`LABEL_SCHEMA`] carries today.
pub const SCHEMA_VERSION: &str = "execution/v1";

/// Container-side mount point for the work surface (§16.6).
const WORKSPACE_MOUNT: &str = "/workspace";

/// The `--user uid:gid` value to run a container as, so its writes into a
/// `read_write` mount come out owned by whoever already owns the mounted
/// directory on the host (§16.8, INV-R1-04). `None` when the owner cannot be
/// read (non-unix, or the directory is gone) — the caller's fallback is to
/// run without `--user`, i.e. the image default.
#[cfg(unix)]
fn host_owner_user_flag(cwd: &Path) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(cwd).ok()?;
    Some(format!("{}:{}", meta.uid(), meta.gid()))
}

#[cfg(not(unix))]
fn host_owner_user_flag(_cwd: &Path) -> Option<String> {
    None
}

/// Bounded human tail length kept alongside the full blob (§16.9's "bounded
/// human tail" product). Small on purpose: it is display material, not
/// evidence — the blob is the evidence.
const TAIL_BYTES: usize = 8192;

/// Configuration for [`DockerBackend`].
#[derive(Debug, Clone)]
pub struct DockerConfig {
    /// Where adapter-side evidence lives: image pins, nothing else durable.
    pub data_dir: PathBuf,
    /// The `docker` executable to invoke. A field, not a hardcoded literal,
    /// so a contract test can point it at a script that fails in specific
    /// ways without touching `PATH`.
    pub docker_bin: String,
}

impl DockerConfig {
    /// The ordinary configuration: `docker` on `PATH`, adapter state under
    /// `data_dir`.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
            docker_bin: "docker".to_string(),
        }
    }
}

/// The `"docker"` backend (N4).
pub struct DockerBackend {
    config: DockerConfig,
    blobs: BlobStore,
    sink: Mutex<Option<EventSink>>,
}

impl std::fmt::Debug for DockerBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockerBackend")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl DockerBackend {
    /// Build the adapter. Fails only if the blob store cannot open under
    /// `config.data_dir` — the same failure mode the daemon already treats as
    /// fatal for its own blob store, so this mirrors that rather than
    /// inventing a second "adapter degraded" state.
    pub fn new(config: DockerConfig) -> std::io::Result<Self> {
        let blobs = BlobStore::open(&config.data_dir)
            .map_err(|e| std::io::Error::other(format!("docker adapter blob store: {e}")))?;
        Ok(Self {
            config,
            blobs,
            sink: Mutex::new(None),
        })
    }

    /// Install the sink normalized events flow through (mirrors
    /// `ClaudeBackend::set_event_sink`; see `daemon::start_with`).
    pub fn set_event_sink(&self, sink: EventSink) {
        *self.sink.lock().expect("docker sink lock") = Some(sink);
    }

    fn cmd(&self) -> Command {
        Command::new(&self.config.docker_bin)
    }

    fn failed(&self, detail: impl Into<String>) -> BackendError {
        BackendError::Failed {
            backend: DOCKER_BACKEND_NAME.to_string(),
            detail: detail.into(),
        }
    }

    fn unknown(&self, execution_id: &str) -> BackendError {
        BackendError::UnknownExecution {
            backend: DOCKER_BACKEND_NAME.to_string(),
            execution_id: execution_id.to_string(),
        }
    }

    /// Run `docker <args>`, returning stdout on success and a sanitized
    /// failure detail otherwise. "Sanitized" here means "stderr as Docker
    /// wrote it" — §16.5 forbids credentials in errors, and this adapter
    /// never has a credential in hand to leak in the first place (no
    /// credential-helper integration exists to construct one).
    fn run(&self, args: &[&str]) -> Result<String, String> {
        let output = self
            .cmd()
            .args(args)
            .output()
            .map_err(|e| format!("failed to run docker {args:?}: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "docker {args:?} exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// The deterministic container name for an execution (§16.10).
    fn container_name(&self, execution_id: &str) -> String {
        format!("sgt-{execution_id}")
    }

    /// Derive the container name a verb should act on from its handle,
    /// preferring the native id (what a restart's `resume` re-derives too)
    /// and falling back to the deterministic name — both name the same
    /// container by construction, this is just defense against a handle
    /// built by hand in a test.
    fn name_of(&self, handle: &ExecutionHandle) -> String {
        handle
            .native_id
            .clone()
            .unwrap_or_else(|| self.container_name(&handle.execution_id))
    }

    /// `docker inspect <name>`, parsed. `Ok(None)` means "no such container" —
    /// Docker's own "no such object" is not treated as this adapter's error,
    /// because "does the container exist" is a routine question every verb
    /// below asks, not a failure.
    fn inspect(&self, name: &str) -> Result<Option<Value>, BackendError> {
        let output = self
            .cmd()
            .args(["inspect", name])
            .output()
            .map_err(|e| self.failed(format!("docker inspect {name}: {e}")))?;
        if !output.status.success() {
            // Docker's own message for a missing object is stable enough to
            // gate on, but the exit code alone is sufficient and does not
            // depend on English stderr text: `docker inspect` on an unknown
            // name/id exits 1 and nothing else in this adapter's own usage
            // produces a nonzero `inspect` exit.
            return Ok(None);
        }
        let parsed: Vec<Value> = serde_json::from_slice(&output.stdout)
            .map_err(|e| self.failed(format!("docker inspect {name}: unparseable JSON: {e}")))?;
        Ok(parsed.into_iter().next())
    }

    /// Whether an inspected container's labels name this execution — the
    /// §16.10 identity check every verb here runs before acting on a name.
    fn labeled_for(info: &Value, execution_id: &str) -> bool {
        info["Config"]["Labels"][LABEL_EXECUTION].as_str() == Some(execution_id)
    }

    /// Path the resolved image identity for `(work_id, stage_id)` is pinned
    /// at (§15.3). Deliberately **not** a journal event the retry path reads
    /// back: threading a durable read of "what did the last attempt at this
    /// stage resolve" through `Engine::reserve_stage` would mean either
    /// widening `ExecutionRecord`/`StageBinding` (touching the shape every
    /// other backend's recovery code also depends on) or giving the engine
    /// Docker-specific knowledge it is this adapter's whole job not to need
    /// (§13.2). A small adapter-owned file under the data dir the daemon
    /// already owns exclusively (`daemon.lock`) gets the same restart-durable
    /// outcome — this pin survives a `sgt daemon` restart on the same data
    /// dir — without moving that line. The resolved identity is *also*
    /// journaled, separately, as evidence (`emit_image_resolved`); this file
    /// is what retry actually reads.
    fn pin_path(&self, work_id: &str, stage_id: &str) -> PathBuf {
        // Stage ids are already constrained to plain directory names
        // (`domain::is_plain_name`, enforced at workflow load); work ids are
        // ULIDs. Neither can carry a path separator, so joining them with
        // `__` cannot collide across different (work, stage) pairs and
        // cannot escape the pins directory.
        self.config
            .data_dir
            .join("docker-adapter")
            .join("image-pins")
            .join(format!("{work_id}__{stage_id}.json"))
    }

    fn read_pin(&self, work_id: &str, stage_id: &str) -> Option<ResolvedImage> {
        let bytes = std::fs::read(self.pin_path(work_id, stage_id)).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    fn write_pin(&self, work_id: &str, stage_id: &str, image: &ResolvedImage) {
        let path = self.pin_path(work_id, stage_id);
        if let Some(parent) = path.parent()
            && std::fs::create_dir_all(parent).is_err()
        {
            return; // best-effort: a failed pin means the next retry re-resolves, not a lost run
        }
        if let Ok(bytes) = serde_json::to_vec(image) {
            let _ = std::fs::write(&path, bytes);
        }
    }

    /// Resolve the image for `spec`, honoring an existing pin (§15.3: retry
    /// uses the immutable identity a prior attempt at this exact stage
    /// already resolved, never a silent re-resolution of a mutable tag).
    ///
    /// Policy, cheapest first (§16.4's lifecycle plus §16.5's lower-rung
    /// registry posture):
    /// 1. a pin exists for `(work_id, stage_id)` — use its `image_id`
    ///    directly, verified still resolvable;
    /// 2. otherwise a matching local image for the authored reference is
    ///    used as-is;
    /// 3. otherwise pull the authored reference (public images only —
    ///    §16.5's lower rung) and inspect what that produced.
    fn resolve_image(
        &self,
        work_id: &str,
        stage_id: &str,
        spec: &ExecuteSpec,
    ) -> Result<ResolvedImage, BackendError> {
        if let Some(pinned) = self.read_pin(work_id, stage_id) {
            match self.inspect_image(&pinned.image_id) {
                Ok(Some(_)) => return Ok(pinned),
                Ok(None) => {
                    // §16.4: "If it has only a local image ID that can no
                    // longer be resolved... the stage blocks with evidence
                    // rather than reinterpreting the mutable tag." Try the
                    // recorded digest once before giving up — that is
                    // explicitly still the pinned identity, just fetched
                    // again.
                    if let Some(digest) = pinned.repo_digests.first() {
                        self.run(&["pull", digest]).map_err(|e| {
                            self.failed(format!(
                                "pinned image {} was pruned and re-pull by digest failed: {e}",
                                pinned.image_id
                            ))
                        })?;
                        if let Some(info) = self.inspect_image(digest)? {
                            return self.image_evidence(&pinned.image_requested, &info);
                        }
                    }
                    return Err(self.failed(format!(
                        "pinned image {} for {work_id}/{stage_id} was pruned and could not be \
                         re-resolved; image refresh is an explicit operator action, not implicit \
                         retry behavior (§16.4)",
                        pinned.image_id
                    )));
                }
                Err(e) => return Err(e),
            }
        }

        // First resolution for this (work, stage): local image first, else
        // pull the authored reference.
        let info = match self.inspect_image(&spec.image)? {
            Some(info) => info,
            None => {
                self.run(&["pull", &spec.image]).map_err(|e| {
                    self.failed(format!("could not resolve image {:?}: {e}", spec.image))
                })?;
                self.inspect_image(&spec.image)?.ok_or_else(|| {
                    self.failed(format!(
                        "docker pull {:?} succeeded but the image cannot be inspected afterward",
                        spec.image
                    ))
                })?
            }
        };
        let resolved = self.image_evidence(&spec.image, &info)?;
        self.write_pin(work_id, stage_id, &resolved);
        Ok(resolved)
    }

    fn inspect_image(&self, reference: &str) -> Result<Option<Value>, BackendError> {
        let output = self
            .cmd()
            .args(["image", "inspect", reference])
            .output()
            .map_err(|e| self.failed(format!("docker image inspect {reference}: {e}")))?;
        if !output.status.success() {
            return Ok(None);
        }
        let parsed: Vec<Value> = serde_json::from_slice(&output.stdout)
            .map_err(|e| self.failed(format!("unparseable image inspect JSON: {e}")))?;
        Ok(parsed.into_iter().next())
    }

    fn image_evidence(
        &self,
        image_requested: &str,
        info: &Value,
    ) -> Result<ResolvedImage, BackendError> {
        let image_id = info["Id"]
            .as_str()
            .ok_or_else(|| self.failed("image inspect returned no Id"))?
            .to_string();
        let repo_digests = info["RepoDigests"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let os = info["Os"].as_str().unwrap_or("unknown");
        let arch = info["Architecture"].as_str().unwrap_or("unknown");
        Ok(ResolvedImage {
            image_requested: image_requested.to_string(),
            image_id,
            repo_digests,
            platform: format!("{os}/{arch}"),
        })
    }

    /// Push the resolved image identity through the event sink as durable,
    /// projectable evidence (§19.4). Purely observational: nothing reads
    /// this back to gate a decision (the pin file does that), so a missing
    /// sink or a journal that has not caught up yet costs nothing but the
    /// projection row.
    fn emit_image_resolved(&self, work_id: &str, execution_id: &str, image: &ResolvedImage) {
        self.emit(
            work_id,
            execution_id,
            "execute.image_resolved",
            json!({
                "image_requested": image.image_requested,
                "image_id": image.image_id,
                "repo_digests": image.repo_digests,
                "platform": image.platform,
            }),
        );
    }

    fn emit(&self, work_id: &str, execution_id: &str, kind: &str, payload: Value) {
        let sink = self.sink.lock().expect("docker sink lock").clone();
        if let Some(sink) = sink {
            let mut draft = EventDraft::new(
                EventSource::new("backend", DOCKER_BACKEND_NAME),
                kind,
                payload,
            )
            .with_work_id(work_id.to_string());
            draft.execution_id = Some(execution_id.to_string());
            draft.correlation_id = Some(execution_id.to_string());
            sink(draft);
        }
    }

    /// Create (but do not start) the container for one execute-stage attempt
    /// (§15.1 steps 3-5, §16.6, §16.7, §16.10).
    fn create_container(
        &self,
        name: &str,
        request: &StartRequest,
        spec: &ExecuteSpec,
    ) -> Result<(), BackendError> {
        // INV-R1-12: `--mount`'s value is a comma-delimited `key=value` CSV
        // grammar, built below by string interpolation. A surfaces root or
        // repository path containing `,` or `=` would split into a bogus or
        // silently misinterpreted option (measured: a `,`-bearing path
        // produces "invalid field '<segment-after-comma>' must be a
        // key=value pair"). Refuse closed with a named error rather than
        // hand Docker a mount spec that means something other than what was
        // intended — the CLI's own error here is aimed at a human debugging
        // a `docker` invocation by hand, not at an operator who never typed
        // `--mount` themselves. Checked before image resolution so this
        // refusal never depends on Docker (or a registry) being reachable.
        let cwd_str = request.cwd.to_string_lossy();
        if cwd_str.contains(',') || cwd_str.contains('=') {
            return Err(self.failed(format!(
                "worktree path {:?} contains ',' or '=', which `--mount`'s key=value CSV \
                 grammar cannot carry safely; refusing to build a bind mount that Docker could \
                 misparse (§16.6)",
                request.cwd
            )));
        }

        let image = self.resolve_image(&request.work_id, &request.stage_id, spec)?;
        self.emit_image_resolved(&request.work_id, &request.execution_id, &image);

        let mount = format!(
            "type=bind,source={},target={WORKSPACE_MOUNT}{}",
            request.cwd.display(),
            if spec.workspace_access == WorkspaceAccess::ReadOnly {
                ",readonly"
            } else {
                ""
            },
        );
        let network = match spec.network {
            NetworkPolicy::None => "none",
        };
        let mut args: Vec<String> = vec![
            "create".into(),
            "--name".into(),
            name.into(),
            "--label".into(),
            format!("{LABEL_MANAGED}=true"),
            "--label".into(),
            format!("{LABEL_EXECUTION}={}", request.execution_id),
            "--label".into(),
            format!("{LABEL_WORK}={}", request.work_id),
            "--label".into(),
            format!("{LABEL_STAGE}={}", request.stage_id),
            "--label".into(),
            format!("{LABEL_ATTEMPT}={}", request.attempt),
            "--label".into(),
            format!("{LABEL_SCHEMA}={SCHEMA_VERSION}"),
            // §16.7's default isolation posture: no privilege, no host
            // namespaces, no extra capabilities, no host devices, and
            // (§16.6) the Docker socket is never mounted in.
            "--network".into(),
            network.into(),
            "--workdir".into(),
            spec.workdir.clone(),
            "--mount".into(),
            mount,
        ];
        // §16.8 (INV-R1-04, MVP-2 D3 fixer pass): without `--user`, a
        // container runs as the image's default — root in every measured
        // probe image — and anything it writes into a `read_write` mount
        // lands root-owned on the host. Measured on Cerberus: a root-owned
        // *file* is merely un-editable by the host user, but a root-owned
        // *directory* the container creates is un-removable too (`rm -rf`
        // on the worktree fails), which breaks worktree teardown. Run the
        // container as the uid:gid that already owns the mounted worktree,
        // so anything it writes is immediately usable and cleanable by
        // whoever owns the surface — no new identity is invented, the
        // existing owner is just carried in. Best-effort: if the owner
        // cannot be read (non-unix, or the path vanished between the mount
        // string being built and now), the container still runs — as the
        // image default — rather than refusing the whole stage over a
        // stat() failure on a path Docker itself is about to bind-mount.
        if let Some(user) = host_owner_user_flag(&request.cwd) {
            args.push("--user".into());
            args.push(user);
        }
        for (key, value) in &spec.env {
            args.push("--env".into());
            args.push(format!("{key}={value}"));
        }
        args.push(image.image_id.clone());
        args.extend(spec.command.iter().cloned());

        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(&arg_refs)
            .map_err(|e| self.failed(format!("docker create {name}: {e}")))?;

        // §16.4 step 6: verify after creation that the container reports the
        // same image identity that was journaled.
        let info = self.inspect(name)?.ok_or_else(|| {
            self.failed(format!(
                "docker create {name} reported success, but the container cannot be inspected"
            ))
        })?;
        let actual_image = info["Image"].as_str().unwrap_or_default();
        if actual_image != image.image_id {
            return Err(self.failed(format!(
                "container {name} reports image {actual_image:?}, expected the resolved \
                 {:?} — refusing to start on a mismatched identity",
                image.image_id
            )));
        }
        Ok(())
    }

    /// Read a finished container's combined evidence (§16.9): full stdout and
    /// stderr streamed straight into the blob store (bounded memory — see
    /// `BlobStore::put_stream`), plus a small bounded tail of each kept for
    /// cheap display. Only called once the container has exited, so
    /// `docker logs` returns a finite, already-buffered stream rather than
    /// following forever.
    fn capture(&self, name: &str) -> CaptureEvidence {
        let mut child = match self
            .cmd()
            .args(["logs", name])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => return CaptureEvidence::failed(format!("docker logs {name}: {e}")),
        };
        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => return CaptureEvidence::failed("docker logs produced no stdout pipe"),
        };
        let stderr = match child.stderr.take() {
            Some(s) => s,
            None => return CaptureEvidence::failed("docker logs produced no stderr pipe"),
        };

        let blobs_for_stdout = &self.blobs;
        let stdout_result = std::thread::scope(|scope| {
            let stderr_handle = scope.spawn(|| stream_one(&self.blobs, stderr));
            let stdout_result = stream_one(blobs_for_stdout, stdout);
            let stderr_result = stderr_handle
                .join()
                .unwrap_or_else(|_| Err("stderr capture thread panicked".to_string()));
            (stdout_result, stderr_result)
        });
        let (stdout_result, stderr_result) = stdout_result;

        let _ = child.wait(); // reap; `docker logs` for a finished container ends on its own

        match (stdout_result, stderr_result) {
            (Ok(stdout), Ok(stderr)) => CaptureEvidence {
                capture: "complete".to_string(),
                stdout: Some(stdout.0),
                stdout_bytes: stdout.1,
                stdout_tail: stdout.2,
                stderr: Some(stderr.0),
                stderr_bytes: stderr.1,
                stderr_tail: stderr.2,
                error: None,
            },
            (stdout_r, stderr_r) => {
                // §16.9: "A disk-full or blob-write failure is recorded as a
                // named capture failure. It does not turn a known container
                // exit into an unknown exit" — the exit code is still
                // reported by the caller regardless of this outcome.
                let detail = [stdout_r.err(), stderr_r.err()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("; ");
                CaptureEvidence::failed(detail)
            }
        }
    }

    /// §16.3's real lifecycle probe, used by `sgt doctor` (not
    /// [`Backend::probe`], which stays cheap for the routing hot path — see
    /// the module docs). Mounts a scratch directory, verifies a write is
    /// visible on the host, and cleans up its own labeled container.
    /// `probe_image` should be a small image sergeant can rely on
    /// (`alpine:3` in production; a test may point this at a locally-built
    /// probe image instead, per the environments matrix — §22.7's
    /// probe-gating rule).
    pub fn lifecycle_probe(&self, probe_image: &str) -> DockerLifecycleProbe {
        let scratch =
            std::env::temp_dir().join(format!("sgt-docker-probe-{}", ulid::Ulid::generate()));
        if let Err(e) = std::fs::create_dir_all(&scratch) {
            return DockerLifecycleProbe::failed(format!("scratch dir: {e}"));
        }
        // Best-effort: this is a throwaway scratch dir under the OS temp
        // root, never inside a worktree or the data dir, so a leaked one on
        // an early return costs nothing durable.
        let _cleanup_scratch = ScratchDir(scratch.clone());
        let marker = scratch.join("probe-marker");
        let name = format!("sgt-probe-{}", ulid::Ulid::generate());
        let mount = format!("type=bind,source={},target=/probe", scratch.display());
        let cleanup = |backend: &Self| {
            let _ = backend.run(&["rm", "-f", &name]);
        };

        if let Err(e) = self.run(&[
            "run",
            "--name",
            &name,
            "--label",
            &format!("{LABEL_MANAGED}=true"),
            "--network",
            "none",
            "--mount",
            &mount,
            probe_image,
            "sh",
            "-c",
            "echo sergeant-probe > /probe/probe-marker",
        ]) {
            cleanup(self);
            return DockerLifecycleProbe::failed(format!("probe container run: {e}"));
        }
        let wrote = std::fs::read_to_string(&marker)
            .map(|s| s.trim() == "sergeant-probe")
            .unwrap_or(false);
        cleanup(self);
        if !wrote {
            return DockerLifecycleProbe::failed(
                "container ran but the bind-mounted marker was not visible on the host \
                 afterward"
                    .to_string(),
            );
        }
        DockerLifecycleProbe {
            available: true,
            detail: None,
            bind_mount: true,
        }
    }
}

/// Removes its directory (recursively) on drop. Used only for
/// [`DockerBackend::lifecycle_probe`]'s scratch dir, which cannot use
/// `tempfile::TempDir` — that crate is a dev-dependency here, unavailable to
/// non-test code — so this is the same "unique name under the OS temp root,
/// cleaned up on drop" shape `cli.rs`'s doctor module already uses by hand.
struct ScratchDir(PathBuf);

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Stream one pipe into the blob store, returning `(blob_ref, bytes, tail)`.
fn stream_one(blobs: &BlobStore, reader: impl Read) -> Result<(BlobRef, u64, String), String> {
    let mut tail = TailCapture::new(reader, TAIL_BYTES);
    let (blob_ref, total) = blobs
        .put_stream(&mut tail)
        .map_err(|e| format!("blob capture: {e}"))?;
    Ok((blob_ref, total, tail.into_tail()))
}

/// A [`Read`] wrapper that remembers only the last `limit` bytes it saw,
/// while still handing every byte through to the inner reader's caller
/// unchanged — the bounded "human tail" product (§16.9), taken in the same
/// pass as the streamed blob write rather than a second read of the content.
struct TailCapture<R> {
    inner: R,
    tail: std::collections::VecDeque<u8>,
    limit: usize,
}

impl<R: Read> TailCapture<R> {
    fn new(inner: R, limit: usize) -> Self {
        Self {
            inner,
            tail: std::collections::VecDeque::with_capacity(limit),
            limit,
        }
    }

    fn into_tail(self) -> String {
        let bytes: Vec<u8> = self.tail.into_iter().collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

impl<R: Read> Read for TailCapture<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        for &b in &buf[..n] {
            if self.tail.len() == self.limit {
                self.tail.pop_front();
            }
            self.tail.push_back(b);
        }
        Ok(n)
    }
}

/// What [`DockerBackend::capture`] produced.
struct CaptureEvidence {
    capture: String,
    stdout: Option<BlobRef>,
    stdout_bytes: u64,
    stdout_tail: String,
    stderr: Option<BlobRef>,
    stderr_bytes: u64,
    stderr_tail: String,
    error: Option<String>,
}

impl CaptureEvidence {
    fn failed(detail: impl Into<String>) -> Self {
        Self {
            capture: "failed".to_string(),
            stdout: None,
            stdout_bytes: 0,
            stdout_tail: String::new(),
            stderr: None,
            stderr_bytes: 0,
            stderr_tail: String::new(),
            error: Some(detail.into()),
        }
    }
}

/// Result of [`DockerBackend::lifecycle_probe`] (§16.3's capability record).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerLifecycleProbe {
    /// Whether the whole round trip (create, bind-mount, start, write, read
    /// back on the host, remove) succeeded.
    pub available: bool,
    /// Why not, when it did not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Whether the bind-mounted write was observed on the host — the one
    /// fact a bare `docker version` ping cannot prove (§16.3).
    pub bind_mount: bool,
}

impl DockerLifecycleProbe {
    fn failed(detail: String) -> Self {
        Self {
            available: false,
            detail: Some(detail),
            bind_mount: false,
        }
    }
}

impl Backend for DockerBackend {
    fn name(&self) -> &str {
        DOCKER_BACKEND_NAME
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            persistent_sessions: false,
            native_background: false,
            streaming: false,
            history: false,
            resume: true,
            interrupt: true,
            model_selection: false,
            profiles: false,
            approval_flow: false,
            human_attach: false,
            usage: false,
            native_subagents: false,
            ask: false,
        }
    }

    fn runtime_scope(&self) -> RuntimeScope {
        // The local Docker Engine exists outside sergeant's lifecycle
        // entirely — sergeant attaches to it (a socket the `docker` group
        // already grants access to) and never starts or owns it. This is
        // exactly `RuntimeScope::External`'s doc comment.
        RuntimeScope::External
    }

    /// Cheap by design (module docs): `Engine::bind_stages` calls this on
    /// every submission that touches an execute stage, so it stays a single
    /// `docker version` round trip rather than the full lifecycle probe
    /// `sgt doctor` runs (`lifecycle_probe`).
    fn probe(&self) -> ProbeReport {
        match self.run(&["version", "--format", "{{.Server.Version}}"]) {
            Ok(version) => ProbeReport {
                available: true,
                detail: Some(format!("Docker Engine {}", version.trim())),
            },
            Err(detail) => ProbeReport {
                available: false,
                detail: Some(detail),
            },
        }
    }

    fn prepare(&self, request: &StartRequest) -> Result<PreparedExecution, BackendError> {
        if request.execute.is_none() {
            return Err(self.failed(
                "the docker backend was asked to prepare a stage with no ExecuteSpec — \
                 this stage should have routed to an actor harness instead",
            ));
        }
        Ok(PreparedExecution {
            execution_id: request.execution_id.clone(),
            // §14.4: a deterministic container name derived from the
            // execution id, reserved (journaled by the engine) before any
            // external effect exists.
            native_id: Some(self.container_name(&request.execution_id)),
            request: request.clone(),
        })
    }

    fn launch(&self, prepared: &PreparedExecution) -> Result<ExecutionHandle, BackendError> {
        let request = &prepared.request;
        let spec = request
            .execute
            .as_ref()
            .ok_or_else(|| self.failed("prepared execution carries no ExecuteSpec"))?;
        let name = prepared
            .native_id
            .clone()
            .unwrap_or_else(|| self.container_name(&prepared.execution_id));

        match self.inspect(&name)? {
            Some(info) if Self::labeled_for(&info, &prepared.execution_id) => {
                // Idempotent re-launch of the same reservation (a caller
                // retrying `PendingLaunch::launch()` after a transient
                // failure before the engine ever saw a result — §22.5's
                // "external identity created but before started append"
                // window, closed by not creating a second container for the
                // same reservation).
            }
            Some(_) => {
                // §16.10: a deterministic name that already exists under
                // different labels is a collision, never an adoption target.
                return Err(self.failed(format!(
                    "container {name} already exists but is not labeled for execution \
                     {:?}; refusing to adopt or overwrite it",
                    prepared.execution_id
                )));
            }
            None => self.create_container(&name, request, spec)?,
        }

        self.run(&["start", &name])
            .map_err(|e| self.failed(format!("docker start {name}: {e}")))?;

        Ok(ExecutionHandle {
            execution_id: prepared.execution_id.clone(),
            native_id: Some(name),
        })
    }

    fn send(&self, _handle: &ExecutionHandle, _input: &str) -> Result<(), BackendError> {
        // §15.4: "An execute stage itself does not reason or ask questions."
        // There is no channel to deliver input into.
        Err(BackendError::Unsupported {
            backend: DOCKER_BACKEND_NAME.to_string(),
            verb: "send".to_string(),
            detail: "execute stages accept no input; a checkpoint that needs a human decision \
                     places an actor stage after this one (§15.4)"
                .to_string(),
        })
    }

    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError> {
        let name = self.name_of(handle);
        let Some(info) = self.inspect(&name)? else {
            return Err(self.unknown(&handle.execution_id));
        };
        if !Self::labeled_for(&info, &handle.execution_id) {
            return Err(self.unknown(&handle.execution_id));
        }
        let state = &info["State"];
        if state["Running"].as_bool().unwrap_or(false) {
            return Ok(Observation {
                native: NativeState::Running,
                signal: super::BackendSignal::Running,
                evidence: None,
            });
        }
        // §11.2/§15.2: the exit code is the whole mechanical result.
        let exit_code = state["ExitCode"].as_i64();
        let started_at = state["StartedAt"].as_str().unwrap_or_default();
        let finished_at = state["FinishedAt"].as_str().unwrap_or_default();
        let image_id = info["Image"].as_str().unwrap_or_default();
        let capture = self.capture(&name);
        let detail = json!({
            "exit_code": exit_code,
            "started_at": started_at,
            "finished_at": finished_at,
            "image_id": image_id,
            "capture": capture.capture,
            "stdout": capture.stdout.as_ref().map(|b| b.to_string()),
            "stdout_bytes": capture.stdout_bytes,
            "stdout_tail": capture.stdout_tail,
            "stderr": capture.stderr.as_ref().map(|b| b.to_string()),
            "stderr_bytes": capture.stderr_bytes,
            "stderr_tail": capture.stderr_tail,
            "capture_error": capture.error,
        });
        let detail_str = detail.to_string();
        let signal = match exit_code {
            Some(0) => super::BackendSignal::StageCompleted {
                summary: Some(detail_str.clone()),
            },
            Some(_) => super::BackendSignal::Failed {
                reason: detail_str.clone(),
            },
            None => super::BackendSignal::Blocked {
                reason: format!("container {name} exited with no reported exit code"),
            },
        };
        Ok(Observation {
            native: NativeState::Exited,
            signal,
            evidence: Some(detail_str),
        })
    }

    fn interrupt(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        let name = self.name_of(handle);
        // A no-op is not an error (trait doc): a container already stopped
        // is exactly the goal state this verb wants.
        match self.inspect(&name)? {
            None => Ok(Completion::immediate()),
            Some(info) if !Self::labeled_for(&info, &handle.execution_id) => {
                Err(self.unknown(&handle.execution_id))
            }
            Some(_) => {
                // §15.5: request the stop; whether it actually stopped is
                // OBSERVE's evidence to report, never this call's inference.
                let _ = self.run(&["stop", &name]);
                Ok(Completion::immediate())
            }
        }
    }

    fn resume(
        &self,
        handle: &ExecutionHandle,
        _request: &ResumeRequest,
    ) -> Result<(), BackendError> {
        // §16.11's recovery matrix, the part expressible through this verb:
        // re-derive the deterministic name, and adopt only an exact
        // name+label match. Everything else — no container, or one with
        // mismatched labels — fails closed rather than guessing.
        let name = self.name_of(handle);
        match self.inspect(&name)? {
            Some(info) if Self::labeled_for(&info, &handle.execution_id) => Ok(()),
            Some(_) => Err(self.failed(format!(
                "container {name} exists but is not labeled for execution {:?}; refusing to \
                 adopt an unrelated container (§16.11)",
                handle.execution_id
            ))),
            None => Err(self.unknown(&handle.execution_id)),
        }
    }

    fn history(&self, _handle: &ExecutionHandle) -> Result<Vec<NativeEvent>, BackendError> {
        Err(BackendError::Unsupported {
            backend: DOCKER_BACKEND_NAME.to_string(),
            verb: "history".to_string(),
            detail: "an execute stage's evidence is its captured stdout/stderr (blob \
                     references on the stage's detail), not a normalized conversation history"
                .to_string(),
        })
    }

    fn stop(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        let name = self.name_of(handle);
        match self.inspect(&name)? {
            None => Ok(Completion::immediate()),
            Some(info) if !Self::labeled_for(&info, &handle.execution_id) => {
                // §16.12: cleanup is exact. A name collision is never a
                // license to remove whatever happens to be sitting there.
                Err(self.unknown(&handle.execution_id))
            }
            Some(_) => {
                // Exact-owned removal only (§16.12): this container, by the
                // id/name/labels this call already verified, nothing global.
                let _ = self.run(&["rm", "-f", &name]);
                Ok(Completion::immediate())
            }
        }
    }
}

/// One row of `sgt doctor`'s Docker capability report (§17.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerDoctorReport {
    /// Cheap probe: is the Engine reachable at all.
    pub engine_available: bool,
    /// Detail from the cheap probe (version string or failure reason).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_detail: Option<String>,
    /// The §16.3 lifecycle round trip, when the cheap probe passed. `None`
    /// when the engine itself was unreachable — there is nothing to round
    /// trip against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<DockerLifecycleProbe>,
    /// Rule B / #23: disk pressure inside the data dir (blobs + docker
    /// adapter state), measured regardless of whether Docker itself is
    /// reachable — data-dir disk pressure is a core concern, not a Docker
    /// one, and folding it here keeps one doctor pass answering both.
    pub disk: DiskPressureReport,
}

/// Rule B / #23: `sgt doctor`'s disk-pressure check. Data-dir size, the blob
/// store's share of it, headroom on the filesystem it lives on, and growth
/// since a caller-supplied baseline (the daemon's own start-of-run size, when
/// known). No blob is written without a journal ref naming it (Rule B's
/// other half) is a property of `BlobStore::put`/`put_stream` themselves —
/// nothing in this reporter writes blobs, it only measures what is already
/// on disk.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DiskPressureReport {
    /// Total bytes under `<data_dir>`.
    pub data_dir_bytes: u64,
    /// Bytes under `<data_dir>/blobs`.
    pub blob_bytes: u64,
    /// Free bytes on the filesystem `<data_dir>` lives on, when the platform
    /// could report it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free_bytes: Option<u64>,
}

/// Measure [`DiskPressureReport`] for `data_dir`. Walks the tree once; a data
/// dir at rest (no live daemon churning it) is safe to walk without racing a
/// writer because `sgt doctor` never runs concurrently with the daemon it is
/// inspecting under the same lock discipline every other doctor check
/// already assumes (`cli.rs`'s doctor module never spawns a daemon of its
/// own).
pub fn measure_disk_pressure(data_dir: &Path) -> DiskPressureReport {
    let data_dir_bytes = dir_size(data_dir);
    let blob_bytes = dir_size(&data_dir.join("blobs"));
    let free_bytes = free_space(data_dir);
    DiskPressureReport {
        data_dir_bytes,
        blob_bytes,
        free_bytes,
    }
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                stack.push(entry.path());
            } else {
                total += metadata.len();
            }
        }
    }
    total
}

#[cfg(unix)]
fn free_space(path: &Path) -> Option<u64> {
    // `statvfs` is the portable POSIX call for this; sergeant is Linux-first
    // (`CLAUDE.md`) and this crate otherwise reaches such facts through
    // `std`, so a `df`-equivalent shell-out is used instead of adding a
    // libc-binding dependency for one syscall — the same "smaller direct
    // client" posture the module docs already take for Docker itself. `df`
    // is present on every measured environment (`docs/environments/`).
    let output = Command::new("df")
        .args(["-k", "--output=avail"])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let last_line = text.lines().last()?.trim();
    last_line.parse::<u64>().ok().map(|kb| kb * 1024)
}

#[cfg(not(unix))]
fn free_space(_path: &Path) -> Option<u64> {
    None
}

/// Build a doctor report: the cheap probe, the lifecycle round trip when the
/// engine is reachable, and disk pressure regardless.
pub fn doctor_report(
    backend: &DockerBackend,
    probe_image: &str,
    data_dir: &Path,
) -> DockerDoctorReport {
    let probe = backend.probe();
    let lifecycle = if probe.available {
        Some(backend.lifecycle_probe(probe_image))
    } else {
        None
    };
    DockerDoctorReport {
        engine_available: probe.available,
        engine_detail: probe.detail,
        lifecycle,
        disk: measure_disk_pressure(data_dir),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::domain::workspace::InstructionPolicy;

    fn config() -> (tempfile::TempDir, DockerConfig) {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let config = DockerConfig::new(dir.path());
        (dir, config)
    }

    #[test]
    fn container_name_is_deterministic_from_execution_id() {
        let (_dir, config) = config();
        let backend = DockerBackend::new(config).expect("backend");
        assert_eq!(backend.container_name("abc123"), "sgt-abc123");
        assert_eq!(backend.container_name("abc123"), "sgt-abc123");
    }

    /// §14.2: PREPARE performs no external effect and always names the
    /// deterministic identity, whether or not `docker` is even installed —
    /// this must never shell out.
    #[test]
    fn prepare_reserves_the_deterministic_name_without_any_external_effect() {
        let (_dir, mut config) = config();
        // A `docker_bin` that does not exist: if `prepare` ever tried to run
        // it, this test would fail with a spawn error instead of `Ok`.
        config.docker_bin = "/nonexistent/docker-binary-that-must-never-run".to_string();
        let backend = DockerBackend::new(config).expect("backend");
        let request = StartRequest {
            work_id: "w1".into(),
            execution_id: "e1".into(),
            stage_id: "10-validate".into(),
            attempt: 1,
            cwd: PathBuf::from("/tmp/whatever"),
            intent: "check it".into(),
            context: String::new(),
            model: None,
            profile: None,
            execute: Some(ExecuteSpec {
                image: "alpine:3".into(),
                command: vec!["true".into()],
                workdir: "/workspace".into(),
                workspace_access: WorkspaceAccess::ReadOnly,
                network: NetworkPolicy::None,
                env: BTreeMap::new(),
            }),
            instruction_policy: InstructionPolicy::default(),
        };
        let prepared = backend
            .prepare(&request)
            .expect("prepare must not touch docker");
        assert_eq!(prepared.native_id.as_deref(), Some("sgt-e1"));
        assert_eq!(prepared.execution_id, "e1");
    }

    /// INV-R1-12 (MVP-2 D3 fixer pass): `--mount`'s value is a CSV
    /// `key=value` grammar; a worktree path containing `,` or `=` must be
    /// refused before it ever reaches Docker, never handed through as a
    /// malformed or misinterpreted `--mount` flag. Uses a nonexistent
    /// `docker_bin` — like `prepare_reserves_the_deterministic_name...`
    /// above — to prove the refusal happens before any external effect;
    /// `resolve_image` (the first thing that would shell out) is never
    /// reached because the path check runs first.
    #[test]
    fn create_container_refuses_a_mount_path_with_special_csv_characters() {
        let (_dir, mut config) = config();
        config.docker_bin = "/nonexistent/docker-binary-that-must-never-run".to_string();
        let backend = DockerBackend::new(config).expect("backend");
        for bad_cwd in ["/tmp/has,a-comma", "/tmp/has=an-equals"] {
            let request = StartRequest {
                work_id: "w1".into(),
                execution_id: "e1".into(),
                stage_id: "10-validate".into(),
                attempt: 1,
                cwd: PathBuf::from(bad_cwd),
                intent: "check it".into(),
                context: String::new(),
                model: None,
                profile: None,
                execute: None,
                instruction_policy: InstructionPolicy::default(),
            };
            let spec = ExecuteSpec {
                image: "alpine:3".into(),
                command: vec!["true".into()],
                workdir: "/workspace".into(),
                workspace_access: WorkspaceAccess::ReadOnly,
                network: NetworkPolicy::None,
                env: BTreeMap::new(),
            };
            let err = backend
                .create_container("sgt-e1", &request, &spec)
                .expect_err(&format!("must refuse a mount path of {bad_cwd:?}"));
            assert!(matches!(err, BackendError::Failed { .. }));
            assert!(
                err.to_string().contains("cannot carry safely"),
                "refusal must name the mount grammar hazard: {err}"
            );
        }
    }

    #[test]
    fn prepare_refuses_a_request_with_no_execute_spec() {
        let (_dir, config) = config();
        let backend = DockerBackend::new(config).expect("backend");
        let request = StartRequest {
            work_id: "w1".into(),
            execution_id: "e1".into(),
            stage_id: "00-only".into(),
            attempt: 1,
            cwd: PathBuf::from("/tmp"),
            intent: "x".into(),
            context: String::new(),
            model: None,
            profile: None,
            execute: None,
            instruction_policy: InstructionPolicy::default(),
        };
        let err = backend.prepare(&request).expect_err("must refuse");
        assert!(matches!(err, BackendError::Failed { .. }));
    }

    #[test]
    fn image_pin_round_trips_through_disk() {
        let (_dir, config) = config();
        let backend = DockerBackend::new(config).expect("backend");
        assert!(backend.read_pin("w1", "s1").is_none());
        let image = ResolvedImage {
            image_requested: "alpine:3".into(),
            image_id: "sha256:deadbeef".into(),
            repo_digests: vec!["alpine@sha256:deadbeef".into()],
            platform: "linux/amd64".into(),
        };
        backend.write_pin("w1", "s1", &image);
        let read_back = backend
            .read_pin("w1", "s1")
            .expect("pin must be readable back");
        assert_eq!(read_back, image);
        // A different stage of the same work never sees this pin.
        assert!(backend.read_pin("w1", "s2").is_none());
    }

    /// `sgt doctor`'s disk-pressure measurement over a store that actually
    /// has blobs in it — the exact shape `docker.rs`'s own capture path
    /// writes, exercised without Docker at all.
    #[test]
    fn measure_disk_pressure_counts_blob_bytes_separately_from_the_whole_data_dir() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let blobs = BlobStore::open(dir.path()).expect("open blob store");
        blobs.put(b"some captured evidence").expect("put");
        std::fs::write(dir.path().join("unrelated.txt"), b"not a blob").expect("write");

        let report = measure_disk_pressure(dir.path());
        assert!(report.blob_bytes > 0, "blob bytes must be counted");
        assert!(
            report.data_dir_bytes >= report.blob_bytes,
            "the whole data dir must be at least as large as its blobs subset"
        );
        assert!(
            report.data_dir_bytes > report.blob_bytes,
            "the unrelated file must be counted in the whole-data-dir total but not in blob_bytes"
        );
    }
}

//! The daemon is the application (proposal §6): one long-lived process owns
//! the journal and projections; clients only ever talk to its loopback API.
//!
//! Startup order is the safety argument:
//!
//! 1. take the exclusive daemon lock in the data dir (second daemon fails
//!    closed before touching anything);
//! 2. open the journal (which holds its own exclusive lock — belt and
//!    braces) and rebuild the Work registry by full replay;
//! 3. bind `127.0.0.1` on an ephemeral port;
//! 4. journal `daemon.started`;
//! 5. write the runtime descriptor (endpoint, PID, API revision, random
//!    bearer token) atomically with owner-only permissions — the descriptor
//!    only ever points at a live, already-listening daemon.
//!
//! Clean shutdown journals `daemon.stopped` and removes the descriptor; a
//! crash leaves a stale descriptor, which clients detect (dead PID + refused
//! endpoint) and replace.

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{broadcast, oneshot, watch};

use crate::api::{API_REVISION, ApiState, Core, CoreError, router};
use crate::backend::claude::{CLAUDE_BACKEND_NAME, ClaudeBackend, ClaudeConfig};
use crate::backend::fake::FAKE_BACKEND_NAME;
use crate::backend::{BackendRegistry, EventSink};
use crate::domain::event::{EventDraft, EventSource};
use crate::runtime::analytics::{Analytics, AnalyticsError};
use crate::runtime::engine::{Engine, EngineError};
use crate::runtime::fsutil::{create_dir_all_durable, take_exclusive_lock, write_atomic_secret};
use crate::runtime::journal::{Journal, JournalError};
use crate::runtime::projection::{ProjectionError, work_registry_projection};
use crate::runtime::recovery;
use crate::telemetry::{Telemetry, TelemetryConfig, TelemetryError};

/// Runtime descriptor file name inside the data dir.
pub const DESCRIPTOR_FILE: &str = "runtime.json";
/// Exclusive daemon lock file name inside the data dir.
pub const DAEMON_LOCK_FILE: &str = "daemon.lock";
/// Schema identifier for the runtime descriptor.
pub const DESCRIPTOR_SCHEMA: &str = "sergeant.runtime/v1";

/// Event kind: the daemon came up and owns the journal.
pub const KIND_DAEMON_STARTED: &str = "daemon.started";
/// Event kind: the daemon shut down cleanly.
pub const KIND_DAEMON_STOPPED: &str = "daemon.stopped";
/// Event kind: a backend was registered and probed (§15 PROBE, recorded at
/// registration as the M4 contract requires).
pub const KIND_BACKEND_PROBED: &str = "backend.probed";

/// The runtime descriptor published for clients (proposal §6): endpoint,
/// PID, API revision, and the bearer token, protected by owner-only file
/// permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDescriptor {
    /// Descriptor schema identifier.
    pub schema: String,
    /// Loopback HTTP endpoint, e.g. `http://127.0.0.1:43210`.
    pub endpoint: String,
    /// PID of the daemon that wrote this descriptor.
    pub pid: u32,
    /// API revision the daemon serves.
    pub api_revision: String,
    /// Random bearer token required on all `/v1/*` routes.
    pub token: String,
}

/// Errors from daemon startup and shutdown.
#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    /// Another live daemon holds this data dir's exclusive lock.
    #[error("another daemon already owns this data dir (daemon.lock is held)")]
    Locked,
    /// Journal failure (open, replay, or append).
    #[error(transparent)]
    Journal(#[from] JournalError),
    /// Projection failure while rebuilding the registry.
    #[error(transparent)]
    Projection(#[from] ProjectionError),
    /// Event commit failure.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// Filesystem or network I/O failure.
    #[error("daemon io error: {0}")]
    Io(#[from] std::io::Error),
    /// Descriptor (de)serialization failure.
    #[error("descriptor serde error: {0}")]
    Serde(#[from] serde_json::Error),
    /// Startup reconciliation of in-flight work failed (§25).
    #[error(transparent)]
    Engine(#[from] EngineError),
    /// The disposable analytical projection could not be rebuilt.
    #[error(transparent)]
    Analytics(#[from] AnalyticsError),
    /// The §28 export pipeline could not be built from its configuration.
    #[error(transparent)]
    Telemetry(#[from] TelemetryError),
    /// The descriptor names a schema this build does not understand. Fail
    /// closed exactly as an unknown snapshot schema does: its fields may
    /// mean something else entirely, and acting on them could mean talking
    /// to the wrong process — or spawning a second daemon.
    #[error(
        "runtime descriptor {path} declares unknown schema {found:?} (this build understands {expected:?}); \
         refusing to use it"
    )]
    UnknownDescriptorSchema {
        /// Path of the offending descriptor.
        path: String,
        /// Schema the file declares.
        found: String,
        /// Schema this build understands.
        expected: &'static str,
    },
}

/// Handle to a running in-process daemon. Dropping it does NOT stop the
/// daemon; call [`DaemonHandle::shutdown`] for a clean stop (journals
/// `daemon.stopped`, removes the descriptor).
#[derive(Debug)]
pub struct DaemonHandle {
    /// Loopback endpoint the daemon is serving on.
    pub endpoint: String,
    /// Bearer token for this daemon instance.
    pub token: String,
    shutdown_tx: Option<oneshot::Sender<()>>,
    served: tokio::task::JoinHandle<()>,
}

impl DaemonHandle {
    /// Signal graceful shutdown and wait for the daemon to finish cleanup.
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        let _ = (&mut self.served).await;
    }
}

/// How a daemon instance is configured beyond its data dir.
///
/// This exists so tests can hand the daemon a scripted backend registry (§37's
/// deterministic core) without a configuration file format that M3 has no
/// second consumer for. The default is the compiled-in registry.
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Backends this daemon can route to (§13, §15).
    pub backends: Arc<BackendRegistry>,
    /// §13's global default tier.
    pub default_backend: Option<String>,
    /// Launch configuration for the Claude adapter this daemon registers
    /// itself. `None` is the system `claude` over this data dir. Tests point
    /// it at a stub binary so the *real* adapter — not a fake wearing its
    /// name — can be driven through the daemon's own request path.
    pub claude: Option<ClaudeConfig>,
    /// §28 OpenTelemetry export. `None` is **off**, and off is the default:
    /// with no pipeline here the daemon builds no provider, spawns no
    /// exporter task, and subscribes nothing to the event stream.
    pub telemetry: Option<Arc<Telemetry>>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            backends: Arc::new(BackendRegistry::default_registry()),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            telemetry: None,
        }
    }
}

/// Start the daemon on `data_dir` with the default backend registry.
pub async fn start(data_dir: &Path) -> Result<DaemonHandle, DaemonError> {
    start_with(data_dir, DaemonConfig::default()).await
}

/// Start the daemon on `data_dir`. Returns once it is serving and the
/// runtime descriptor is published.
pub async fn start_with(
    data_dir: &Path,
    config: DaemonConfig,
) -> Result<DaemonHandle, DaemonError> {
    create_dir_all_durable(data_dir)?;

    // 1. Exclusive daemon lock: a second daemon on the same data dir fails
    // closed here, before touching journal or descriptor. The OS releases
    // the advisory lock on process death, so it can never go stale. A second
    // daemon *process* is refused immediately; only a lock this same process
    // has held before is waited out, because only then can one of our own
    // forked git children still be holding a duplicate of it (see
    // `take_exclusive_lock`).
    let lock_path = data_dir.join(DAEMON_LOCK_FILE);
    let lock = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)?;
    if !take_exclusive_lock(&lock_path, &lock)? {
        return Err(DaemonError::Locked);
    }

    // 2. Own the journal and rebuild current state by full replay (§24;
    // snapshots are an optimization the M2 daemon does not need yet).
    let mut journal = Journal::open(data_dir)?;
    let mut registry = work_registry_projection();
    registry.catch_up(journal.replay()?)?;

    // 2b. The disposable projections (§21–§23, §40). The DuckDB file is
    // rebuilt from the journal on **every** start, so deleting it and
    // restarting is indistinguishable from restarting: no code path can come
    // to depend on state that only lives in there.
    let analytics = Analytics::rebuild(data_dir, journal.replay()?)?;

    // 2c. §28 export, when it is switched on. The journal's append timing is
    // the one metric whose input exists nowhere else, so the observer is
    // installed here — and only here, when export is on.
    if let Some(telemetry) = &config.telemetry {
        let telemetry = telemetry.clone();
        journal.set_append_observer(Arc::new(move |elapsed| {
            telemetry.record_journal_append(elapsed);
        }));
    }

    let (events_tx, _) = broadcast::channel(1024);
    let mut core = Core {
        journal,
        registry,
        events_tx,
    };

    // 3. Bind loopback on an ephemeral port before publishing anything.
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let endpoint = format!("http://{}", listener.local_addr()?);

    // 4. Lifecycle event: the journal records that this daemon took over.
    core.commit(EventDraft::new(
        EventSource::new("daemon", "sergeant"),
        KIND_DAEMON_STARTED,
        json!({"pid": std::process::id(), "version": env!("CARGO_PKG_VERSION"), "endpoint": endpoint}),
    ))?;

    // 4b. Register the real adapter alongside whatever the config supplied
    // (tests hand in scripted fakes; they keep them). It is added here, not
    // in `default_registry`, because the Claude adapter needs this data dir
    // for its raw-transcript archive and an event sink that only exists once
    // the core does. A config that already registered the name wins — that
    // is how tests substitute stubs. Codex is not registered at all: it is
    // descoped by deviation D6, and a backend nothing has measured must not
    // appear in routing output as something a user could ask for.
    let mut backends = (*config.backends).clone();
    let claude = if config.backends.get(CLAUDE_BACKEND_NAME).is_none() {
        let claude_config = config
            .claude
            .clone()
            .unwrap_or_else(|| ClaudeConfig::new(data_dir));
        let adapter = Arc::new(ClaudeBackend::new(claude_config));
        backends = backends.with(adapter.clone());
        Some(adapter)
    } else {
        None
    };
    let backends = Arc::new(backends);

    // 4b-ii. The capability/version probe, recorded at registration (M4
    // contract). Probing is offline and token-free (`--version`, `--help`),
    // and journaling the answer is the point: a version and flag set that
    // only ever appear inside a later refusal's message are not a record.
    // An unavailable backend is registered anyway and refuses work with this
    // same evidence — routing must be able to say *why*, not pretend the
    // backend does not exist.
    for name in backends.names() {
        let Some(backend) = backends.get(&name) else {
            continue;
        };
        let report = backend.probe();
        core.commit(EventDraft::new(
            EventSource::new("daemon", "sergeant"),
            KIND_BACKEND_PROBED,
            json!({
                "backend": name,
                "available": report.available,
                "detail": report.detail,
                "capabilities": backend.capabilities(),
                // §17: the adapter's declared runtime scope, recorded with
                // the probe because it is a claim about the adapter, and the
                // core is forbidden from assuming one.
                "runtime_scope": backend.runtime_scope(),
            }),
        ))?;
    }

    // 4c. Reconcile work believed in flight *before* serving (§25): no
    // request may observe — or act on — a work whose prior ownership has not
    // yet been settled.
    let engine = Arc::new(Engine::new(
        backends,
        config.default_backend.clone(),
        data_dir,
    ));
    let reconciled = recovery::reconcile(&engine, &mut core)?;
    if !reconciled.resumed.is_empty()
        || !reconciled.blocked.is_empty()
        || !reconciled.surfaces_retired.is_empty()
    {
        tracing::info!(
            resumed = ?reconciled.resumed,
            blocked = ?reconciled.blocked,
            surfaces_retired = ?reconciled.surfaces_retired,
            "reconciled in-flight work after restart"
        );
    }

    // 5. Publish the descriptor: atomic rename, owner-only permissions,
    // written only now that the listener is live — a descriptor never
    // points at a daemon that is not yet serving.
    let token = format!("{}{}", ulid::Ulid::generate(), ulid::Ulid::generate());
    let descriptor = RuntimeDescriptor {
        schema: DESCRIPTOR_SCHEMA.to_string(),
        endpoint: endpoint.clone(),
        pid: std::process::id(),
        api_revision: API_REVISION.to_string(),
        token: token.clone(),
    };
    let descriptor_path = data_dir.join(DESCRIPTOR_FILE);
    write_atomic_secret(&descriptor_path, &serde_json::to_vec_pretty(&descriptor)?)?;

    // Shutdown broadcast for endpoints whose response body never ends by
    // itself: graceful shutdown waits for in-flight responses, so a live SSE
    // tail would otherwise keep this daemon serving forever.
    let (closing_tx, closing_rx) = watch::channel(false);
    let state = ApiState {
        core: Arc::new(tokio::sync::Mutex::new(core)),
        token: token.clone(),
        data_dir: data_dir.to_path_buf(),
        closing: closing_rx,
        engine,
        analytics: Arc::new(tokio::sync::Mutex::new(analytics)),
    };
    // §28's export is a fold over the event stream, subscribed here and
    // nowhere else. With export off this task does not exist.
    if let Some(telemetry) = config.telemetry.clone() {
        let events = state.core.lock().await.events_tx.subscribe();
        tokio::spawn(export_events(telemetry, events));
    }
    // The Claude adapter's normalized events (§27) flow into the journal
    // through the core; the sink can only exist now that the core is shared.
    if let Some(claude) = claude {
        claude.set_event_sink(journaling_sink(state.core.clone()));
    }
    let app = router(state.clone());

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let served = tokio::spawn(async move {
        // The daemon lock lives exactly as long as the serve task.
        let _lock = lock;
        let serve = axum::serve(listener, app).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
            // Tell the SSE pumps to finish *before* handing control to the
            // graceful wait, so the wait has something finite to wait for.
            let _ = closing_tx.send(true);
        });
        if let Err(e) = serve.await {
            tracing::error!(error = %e, "daemon serve failed");
        }
        // Clean shutdown: journal the stop, then retire the descriptor.
        let mut core = state.core.lock().await;
        if let Err(e) = core.commit(EventDraft::new(
            EventSource::new("daemon", "sergeant"),
            KIND_DAEMON_STOPPED,
            json!({"pid": std::process::id()}),
        )) {
            tracing::warn!(error = %e, "failed to journal daemon.stopped");
        }
        if let Err(e) = std::fs::remove_file(&descriptor_path) {
            tracing::warn!(error = %e, "failed to remove runtime descriptor");
        }
    });

    Ok(DaemonHandle {
        endpoint,
        token,
        shutdown_tx: Some(shutdown_tx),
        served,
    })
}

/// Build the [`EventSink`] that journals an adapter's normalized events.
///
/// **The sink never blocks and never locks on its caller's thread.** It hands
/// the draft to a dedicated committer thread and returns. That is not an
/// optimization; it is what makes the sink safe to call from *any* thread an
/// adapter has, which is the only assumption an adapter can be held to:
///
/// - a real adapter emits from the request path. `Backend::start` is called
///   by the engine while the API handler holds the core lock on a tokio
///   worker; `ClaudeBackend::start` spawns the (token-burning) turn and then
///   emits `conversation.user` on that same thread. A sink that took
///   `blocking_lock` there would panic inside a runtime — after the child
///   process exists and before `execution.started` is journaled, i.e. an
///   orphaned native process with no durable record: an L6 crash window
///   opened by the logging path;
/// - a sink that blocked instead of panicking would be no better: the lock it
///   waits for is the one its own caller is holding.
///
/// So the committer thread owns all locking. It is also where causation
/// chaining belongs: journal-assigned event ids exist only after commit, so
/// only the committer can thread `causation_id` from one event to the next
/// within an execution, and being single-threaded it does so in a
/// deterministic order. Correlation (`correlation_id` = execution id) stays
/// the adapter's.
///
/// **Rung note (R2, a std thread where tokio primitives exist).** The
/// blocking-lock hazard above is real, but it alone does not force an OS
/// thread: `tokio::sync::mpsc::unbounded_channel` can be sent to from any
/// thread, and a spawned task would need no `Weak` dance. What forces this
/// shape is that the deterministic tests construct a sink from synchronous
/// `#[test]` functions, where there is no runtime for a task to live in and
/// `tokio::spawn` would panic. That is test ergonomics deciding a production
/// rung, recorded here rather than left to be rediscovered: the day the
/// daemon needs backpressure or ordered shutdown of this queue, the tokio
/// version is the right one and the tests move to `#[tokio::test]`.
///
/// Delivery is therefore asynchronous: a caller learns nothing about whether
/// the event landed, which is correct — the journal is the daemon's
/// single-owner surface, not the adapter's. The thread lives until every
/// sink clone is dropped.
pub fn journaling_sink(core: Arc<tokio::sync::Mutex<Core>>) -> EventSink {
    let (tx, rx) = std::sync::mpsc::channel::<EventDraft>();
    // The committer holds the core *weakly*: it must not keep the journal —
    // and the exclusive lock the journal holds on the data dir — alive past
    // the daemon that owns it, or a successor daemon would be locked out by
    // a logging thread. Events still queued when the daemon goes away are
    // dropped, which is the honest end state: delivery is best-effort, the
    // journal is the daemon's.
    let core = Arc::downgrade(&core);
    std::thread::Builder::new()
        .name("sergeant-event-sink".to_string())
        .spawn(move || commit_normalized_events(&core, &rx))
        .expect("spawn the normalized-event committer thread");
    let tx = std::sync::Mutex::new(tx);
    Arc::new(move |draft: EventDraft| {
        if let Err(e) = tx.lock().expect("sink queue lock").send(draft) {
            tracing::warn!(error = %e, "normalized backend event dropped: committer is gone");
        }
    })
}

/// How many executions the committer keeps a causation chain for.
///
/// The chain is one journal-event id per execution, and an execution is never
/// "finished" from the sink's point of view — a conversation can be sent to
/// again at any time — so nothing in the event stream says when an entry may
/// be dropped. Unbounded, that is a map that grows for the daemon's lifetime
/// with every execution it ever ran. Bounded, the oldest execution's chain is
/// forgotten and its next event starts a new chain (`causation_id: None`),
/// which is the same thing that happens across a daemon restart and is
/// already what the chain means: causation links events the daemon observed
/// in sequence, and correlation — which is never forgotten — is what groups
/// an execution's events for all time.
pub const SINK_CHAIN_CAPACITY: usize = 256;

/// The committer thread's loop: drain drafts in order, chain causation,
/// commit. Runs on a plain std thread, so `blocking_lock` is correct here and
/// only here.
fn commit_normalized_events(
    core: &std::sync::Weak<tokio::sync::Mutex<Core>>,
    rx: &std::sync::mpsc::Receiver<EventDraft>,
) {
    let mut chain: HashMap<String, String> = HashMap::new();
    // Insertion order of the keys in `chain`, so the bound evicts the
    // least-recently-started execution rather than an arbitrary one.
    let mut order: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    while let Ok(mut draft) = rx.recv() {
        let Some(core) = core.upgrade() else {
            tracing::debug!("normalized-event committer stopping: the core is gone");
            return;
        };
        let key = draft.execution_id.clone();
        if draft.causation_id.is_none()
            && let Some(key) = &key
        {
            draft.causation_id = chain.get(key).cloned();
        }
        let mut core = core.blocking_lock();
        match core.commit(draft) {
            Ok(event) => {
                if let Some(key) = key {
                    if chain.insert(key.clone(), event.id).is_none() {
                        order.push_back(key);
                    }
                    while order.len() > SINK_CHAIN_CAPACITY {
                        if let Some(evicted) = order.pop_front() {
                            chain.remove(&evicted);
                        }
                    }
                }
            }
            Err(e) => tracing::warn!(error = %e, "failed to journal normalized backend event"),
        }
    }
}

/// Feed the §28 export the committed event stream.
///
/// Lagging behind the broadcast buffer drops spans, and that is the correct
/// failure for an export projection: the journal still has everything, and a
/// telemetry backlog must never apply backpressure to execution.
async fn export_events(
    telemetry: Arc<Telemetry>,
    mut events: broadcast::Receiver<crate::domain::event::Event>,
) {
    loop {
        match events.recv().await {
            Ok(event) => telemetry.record(&event),
            Err(broadcast::error::RecvError::Lagged(missed)) => {
                tracing::warn!(missed, "otel export fell behind the event stream");
            }
            Err(broadcast::error::RecvError::Closed) => {
                telemetry.force_flush();
                return;
            }
        }
    }
}

/// Run the daemon in the foreground until SIGINT/SIGTERM, then shut down
/// cleanly. This is what `sgt daemon` (and therefore auto-spawn) executes.
///
/// This is the one place §28 export is configured from the environment, and
/// [`TelemetryConfig::from_env`] answers "off" unless explicitly switched on.
pub async fn run_until_signal(data_dir: &Path) -> Result<(), DaemonError> {
    let telemetry_config = TelemetryConfig::from_env();
    let telemetry = Telemetry::from_config(&telemetry_config)?.map(Arc::new);
    if telemetry.is_some() {
        tracing::info!(
            endpoint = telemetry_config.endpoint(),
            "otel export enabled"
        );
    }
    let handle = start_with(
        data_dir,
        DaemonConfig {
            telemetry: telemetry.clone(),
            ..DaemonConfig::default()
        },
    )
    .await?;
    tracing::info!(endpoint = %handle.endpoint, data_dir = %data_dir.display(), "daemon serving");
    wait_for_shutdown_signal().await;
    tracing::info!("shutdown signal received");
    handle.shutdown().await;
    if let Some(telemetry) = telemetry {
        telemetry.shutdown();
    }
    Ok(())
}

/// Resolve on SIGINT (Ctrl-C) or, on Unix, SIGTERM.
async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        let mut term =
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(term) => term,
                Err(e) => {
                    tracing::error!(error = %e, "cannot install SIGTERM handler");
                    let _ = tokio::signal::ctrl_c().await;
                    return;
                }
            };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

/// Read the runtime descriptor from a data dir, if present.
///
/// A malformed descriptor is an error, not `None`: it means something is
/// wrong with the runtime dir and silently spawning a second daemon on top
/// of it would be the worst response. A descriptor declaring a schema this
/// build does not understand is malformed in exactly that sense, so it is
/// refused here rather than half-interpreted — the reader that makes
/// [`DESCRIPTOR_SCHEMA`] a promise instead of a decoration.
pub fn read_descriptor(data_dir: &Path) -> Result<Option<RuntimeDescriptor>, DaemonError> {
    let path = data_dir.join(DESCRIPTOR_FILE);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let descriptor: RuntimeDescriptor = serde_json::from_slice(&bytes)?;
    if descriptor.schema != DESCRIPTOR_SCHEMA {
        return Err(DaemonError::UnknownDescriptorSchema {
            path: path.display().to_string(),
            found: descriptor.schema,
            expected: DESCRIPTOR_SCHEMA,
        });
    }
    Ok(Some(descriptor))
}

/// Whether a PID currently names a live process (Linux: `/proc/<pid>`;
/// elsewhere this is unknowable without more machinery, so report alive —
/// the fail-closed direction for spawn decisions).
pub fn pid_alive(pid: u32) -> bool {
    if cfg!(target_os = "linux") {
        Path::new(&format!("/proc/{pid}")).exists()
    } else {
        true
    }
}

/// Convenience: the descriptor path for a data dir.
pub fn descriptor_path(data_dir: &Path) -> PathBuf {
    data_dir.join(DESCRIPTOR_FILE)
}

/// Issue #35: the committer thread's own failure paths (`commit_normalized_events`'s
/// `Weak` upgrade, the `journaling_sink` closure's send-after-gone branch)
/// and `export_events`'s two non-`Ok` arms (`Lagged`, `Closed`).
///
/// These are unit tests against the private functions directly, not through
/// `start_with`: every scenario here is a race between the daemon going away
/// and something still trying to use it, and racing the real HTTP surface to
/// get there reliably would be slower and less precise than driving the
/// committer's channel and the export loop's broadcast receiver by hand.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::event::Event;
    use crate::domain::work::{KIND_WORK_COMPLETED, KIND_WORK_NEEDS_INPUT, KIND_WORK_SUBMITTED};
    use crate::telemetry::test_support::counter_total;
    use opentelemetry_sdk::metrics::InMemoryMetricExporter;
    use opentelemetry_sdk::trace::InMemorySpanExporter;
    use std::time::Duration;

    /// A minimal, directly-constructed [`Core`] over a fresh journal — no
    /// HTTP surface, no engine, just what the committer and the export loop
    /// actually touch.
    fn test_core(data_dir: &Path) -> Core {
        let journal = Journal::open(data_dir).expect("open journal");
        let mut registry = work_registry_projection();
        registry
            .catch_up(journal.replay().expect("replay"))
            .expect("catch up");
        let (events_tx, _) = broadcast::channel(16);
        Core {
            journal,
            registry,
            events_tx,
        }
    }

    /// Poll the journal (lock-free: [`Journal::replay_data_dir`] only reads
    /// segment files, so this works while another handle still holds the
    /// journal's own exclusive lock) until `kind` shows up, bounded so a
    /// regression that stops committing fails the test instead of hanging
    /// it.
    fn wait_for_kind(data_dir: &Path, kind: &str) {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let found = Journal::replay_data_dir(data_dir)
                .expect("replay")
                .filter_map(|e| e.ok())
                .any(|e| e.kind == kind);
            if found {
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "{kind:?} never appeared in the journal"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Wait for a thread to finish without ever blocking indefinitely: a
    /// committer that hangs instead of returning fails this assertion
    /// cleanly rather than hanging the whole test binary.
    fn join_with_deadline(handle: std::thread::JoinHandle<()>, timeout: Duration) {
        let deadline = std::time::Instant::now() + timeout;
        while !handle.is_finished() {
            assert!(
                std::time::Instant::now() < deadline,
                "thread did not finish within {timeout:?}"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        handle.join().expect("thread panicked");
    }

    /// A `work.needs_input` event: the fold's one unconditional, parent-free
    /// counter (`sergeant_needs_input_total`), so a test can prove something
    /// was recorded without also standing up a `work.submitted` span to
    /// parent it under.
    fn needs_input_event(seq: u64) -> Event {
        EventDraft::new(
            EventSource::new("daemon", "test"),
            KIND_WORK_NEEDS_INPUT,
            json!({}),
        )
        .with_work_id(format!("w{seq}"))
        .into_event(seq)
    }

    /// A `work.submitted`/`work.completed` pair for `work_id`: the fold's
    /// span-opening and span-closing events, exported the moment the span
    /// closes (`with_exporters` uses a simple, synchronous exporter — see
    /// its own doc comment) rather than on any periodic timer.
    fn work_submitted_event(seq: u64, work_id: &str) -> Event {
        EventDraft::new(
            EventSource::new("daemon", "test"),
            KIND_WORK_SUBMITTED,
            json!({"work": {"id": work_id, "intent": "x"}}),
        )
        .with_work_id(work_id)
        .into_event(seq)
    }

    /// See [`work_submitted_event`].
    fn work_completed_event(seq: u64, work_id: &str) -> Event {
        EventDraft::new(
            EventSource::new("daemon", "test"),
            KIND_WORK_COMPLETED,
            json!({}),
        )
        .with_work_id(work_id)
        .into_event(seq)
    }

    /// `commit_normalized_events` commits while its `Weak<Core>` still
    /// upgrades, and stops committing — without panicking or hanging — the
    /// moment the daemon's last strong reference is gone. This is the exact
    /// race `journaling_sink`'s doc comment names: "must not keep the
    /// journal alive past the daemon that owns it".
    ///
    /// A draft queued *before* the drop but only dequeued *after* it must
    /// still be dropped, not committed — queuing happens on whichever
    /// thread calls the sink, dequeuing happens later on the committer's own
    /// thread, and the two are allowed to interleave any way the scheduler
    /// likes.
    #[test]
    fn the_committer_commits_while_the_core_lives_and_drops_events_once_it_is_gone() {
        let data = tempfile::TempDir::new().expect("tempdir");
        let core = Arc::new(tokio::sync::Mutex::new(test_core(data.path())));
        let weak = Arc::downgrade(&core);
        let (tx, rx) = std::sync::mpsc::channel::<EventDraft>();

        let handle = std::thread::spawn(move || commit_normalized_events(&weak, &rx));

        tx.send(EventDraft::new(
            EventSource::new("test", "t"),
            "test.alive",
            json!({}),
        ))
        .expect("queue alive");
        wait_for_kind(data.path(), "test.alive");

        // The daemon's only strong reference to its own core goes away.
        drop(core);
        tx.send(EventDraft::new(
            EventSource::new("test", "t"),
            "test.after_drop",
            json!({}),
        ))
        .expect("queue after drop");

        // `tx` is deliberately still alive here. Stopping is the guard, not
        // discarding: a committer that merely skipped the draft and kept
        // draining would satisfy every assertion below, and would also keep
        // the journal's exclusive lock on the data dir alive for as long as
        // any sink handle exists — the successor-daemon lockout
        // `journaling_sink`'s doc comment exists to prevent. With the
        // sending half held open the *only* thing that can end this thread
        // is the failed `Weak::upgrade` returning.
        join_with_deadline(handle, Duration::from_secs(5));
        drop(tx);

        let kinds: Vec<String> = Journal::replay_data_dir(data.path())
            .expect("replay")
            .map(|e| e.expect("event").kind)
            .collect();
        assert!(
            kinds.contains(&"test.alive".to_string()),
            "the draft sent while the core was alive must be committed: {kinds:?}"
        );
        assert!(
            !kinds.contains(&"test.after_drop".to_string()),
            "a draft the committer only sees after the core is gone must \
             never be committed: {kinds:?}"
        );
    }

    /// How many live threads carry the name `journaling_sink` gives its
    /// committer. Linux truncates thread names to 15 bytes
    /// (`TASK_COMM_LEN - 1`), so `"sergeant-event-sink"` is published under
    /// `/proc/self/task/*/comm` as `"sergeant-event-"`.
    ///
    /// This is positive evidence that a committer has really exited, which
    /// the send-after-gone test needs and cannot get any other way: the sink
    /// swallows the delivery failure by design, and nothing else it can
    /// observe distinguishes "the send failed" from "the send succeeded and
    /// the draft was discarded". Nothing else in this test binary spawns
    /// that thread — `journaling_sink`'s only other caller is `start_with`,
    /// which no unit test in this crate runs — so a non-zero count here is
    /// this test's own committer.
    #[cfg(target_os = "linux")]
    fn committer_threads_alive() -> usize {
        std::fs::read_dir("/proc/self/task")
            .expect("read /proc/self/task")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                std::fs::read_to_string(entry.path().join("comm"))
                    .is_ok_and(|name| name.trim_end() == "sergeant-event-")
            })
            .count()
    }

    /// Block until [`committer_threads_alive`] reads `expected`, bounded so a
    /// committer that never stops fails the test instead of hanging it. A
    /// wait rather than a bare read in both directions: `Builder::spawn`
    /// returns before the new thread has published its own name, and exit is
    /// asynchronous by nature. Off Linux there is no `/proc` to read, so this
    /// degrades to a fixed pause — the weaker pre-existing shape, kept only
    /// so the test still compiles and runs there.
    fn wait_for_committer_threads(expected: usize, why: &str) {
        #[cfg(target_os = "linux")]
        {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while committer_threads_alive() != expected {
                assert!(std::time::Instant::now() < deadline, "{why}");
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (expected, why);
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    /// The sink `journaling_sink` returns is a plain closure over a channel
    /// `Sender`; once the committer thread has actually exited (its
    /// receiver goes with it), every further call must be swallowed as a
    /// warning, never propagated as a panic — an adapter emitting from
    /// *its* thread has no one to hand a delivery failure to.
    #[test]
    fn sink_calls_after_the_committer_is_gone_never_panic_and_never_land_in_the_journal() {
        let data = tempfile::TempDir::new().expect("tempdir");
        let core = Arc::new(tokio::sync::Mutex::new(test_core(data.path())));
        let sink = journaling_sink(core.clone());
        wait_for_committer_threads(
            1,
            "the sink must have spawned exactly the committer this test then \
             waits out",
        );

        // Drop the only strong reference the committer could ever upgrade
        // to, then wake it: the next draft it dequeues finds the core
        // already gone, so it returns — taking its receiver, the sink's
        // only counterpart, with it.
        drop(core);
        sink(EventDraft::new(
            EventSource::new("test", "t"),
            "test.wake",
            json!({}),
        ));

        // Not "wait a while and assume": wait until the receiving half is
        // provably gone, so every call below really is a failed send hitting
        // the swallow branch. If it panicked instead of logging, this would
        // abort right here.
        wait_for_committer_threads(
            0,
            "the committer must exit once the core is gone; while it is still \
             running the sink's send cannot fail and the branch under test is \
             never reached",
        );
        for i in 0..3 {
            sink(EventDraft::new(
                EventSource::new("test", "t"),
                format!("test.after_gone.{i}"),
                json!({}),
            ));
        }

        let kinds: Vec<String> = Journal::replay_data_dir(data.path())
            .expect("replay")
            .map(|e| e.expect("event").kind)
            .collect();
        assert!(
            !kinds.iter().any(|k| k.starts_with("test.")),
            "nothing sent after the core was dropped may ever reach the \
             journal: {kinds:?}"
        );
    }

    /// A receiver that lags does not end `export_events`: the surviving tail
    /// the ring buffer actually kept is still recorded after the gap.
    ///
    /// Three throwaway sends into a two-slot channel before anyone ever
    /// calls `recv()` guarantee (not race) that the first `recv()` reports
    /// `Lagged(3)` rather than delivering in order — broadcast's own
    /// documented overwrite contract, not scheduler luck — leaving exactly
    /// the last two sends, one work's open/close pair, in the buffer.
    #[tokio::test]
    async fn export_events_keeps_recording_after_the_broadcast_receiver_lags() {
        let spans = InMemorySpanExporter::default();
        let metrics = InMemoryMetricExporter::default();
        let telemetry = Arc::new(Telemetry::with_exporters(spans.clone(), metrics));

        let (tx, rx) = broadcast::channel::<Event>(2);
        for seq in 0..3u64 {
            tx.send(work_submitted_event(seq, &format!("junk{seq}")))
                .expect("buffer junk");
        }
        tx.send(work_submitted_event(10, "survivor"))
            .expect("buffer open");
        tx.send(work_completed_event(11, "survivor"))
            .expect("buffer close");
        drop(tx);

        // A clone, not a move: `export_events` drops its own reference on
        // return, and a `Telemetry` whose refcount reaches zero drops its
        // `SdkTracerProvider` with it, which discards this in-memory
        // exporter's already-recorded spans along the way — a live
        // reference must outlast the call, or the assertion below inspects
        // an exporter the drop has already emptied.
        tokio::time::timeout(Duration::from_secs(5), export_events(telemetry.clone(), rx))
            .await
            .expect(
                "export_events must return once the sender side is gone, \
             even after lagging",
            );

        let finished = spans.get_finished_spans().expect("spans");
        assert_eq!(
            finished.len(),
            1,
            "Lagged must not end the loop: the surviving open/close pair \
             must still open and close a span — if Lagged ended it \
             instead of continuing, this would be empty: {finished:?}"
        );
        assert_eq!(finished[0].name, "work");
    }

    /// `Closed` force-flushes before returning, rather than just returning:
    /// without the flush, whatever the fold already recorded stays stuck in
    /// the SDK's pending batch and this counter reads back 0.
    #[tokio::test]
    async fn export_events_force_flushes_and_returns_when_the_channel_closes() {
        let spans = InMemorySpanExporter::default();
        let metrics = InMemoryMetricExporter::default();
        let telemetry = Arc::new(Telemetry::with_exporters(spans, metrics.clone()));

        let (tx, rx) = broadcast::channel::<Event>(4);
        tx.send(needs_input_event(1)).expect("send");
        drop(tx);

        // If `export_events` did not return on `Closed`, this would hang
        // forever; the timeout turns that failure mode into a clean
        // assertion instead of a stuck test binary. A clone, not a move,
        // for the same reason as the Lagged test above: `telemetry` must
        // outlive the call so the assertion below is not reading an
        // exporter the drop already emptied.
        tokio::time::timeout(Duration::from_secs(5), export_events(telemetry.clone(), rx))
            .await
            .expect("export_events must return once the sender side is gone");

        assert_eq!(
            counter_total(&metrics, "sergeant_needs_input_total"),
            1,
            "Closed must force_flush before returning"
        );
    }

    /// …and that flush covers *both* pipelines, not just the meter half the
    /// counter above can see.
    ///
    /// `Telemetry::with_exporters` wires a *simple* span processor, which
    /// exports at span end and whose `force_flush` is therefore a no-op: on
    /// that pipeline the spans are already out before anything flushes, so
    /// `Telemetry::force_flush`'s `tracer_provider.force_flush()` line could
    /// be deleted with every other daemon test green. A **batched** processor
    /// holds finished spans in its own queue on a five-second timer, so the
    /// span is exported here if and only if `export_events` really flushed
    /// the tracer pipeline before returning.
    #[tokio::test]
    async fn export_events_force_flushes_the_span_pipeline_too_when_the_channel_closes() {
        let spans = InMemorySpanExporter::default();
        let metrics = InMemoryMetricExporter::default();
        let telemetry = Arc::new(Telemetry::with_batch_span_exporter(spans.clone(), metrics));

        let (tx, rx) = broadcast::channel::<Event>(4);
        tx.send(work_submitted_event(1, "flushed")).expect("open");
        tx.send(work_completed_event(2, "flushed")).expect("close");
        drop(tx);

        // A clone, not a move, for the same reason as the two tests above.
        tokio::time::timeout(Duration::from_secs(5), export_events(telemetry.clone(), rx))
            .await
            .expect("export_events must return once the sender side is gone");

        let finished = spans.get_finished_spans().expect("spans");
        assert_eq!(
            finished.len(),
            1,
            "Closed must force_flush the span pipeline before returning: an \
             unflushed batch processor is still holding this span in its own \
             queue, and the export never happened: {finished:?}"
        );
        assert_eq!(finished[0].name, "work");
    }
}

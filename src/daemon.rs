//! The daemon is the application (proposal §6): one long-lived process owns
//! the journal and projections; clients only ever talk to its loopback API.
//!
//! Startup order is the safety argument:
//!
//! 0. refuse outright if the data dir's filesystem is one where advisory
//!    locking is unreliable (#85, ADR 0003 D6) — the lock taken in step 1
//!    would not actually exclude a second daemon there, and that failure is
//!    silent until two daemons are racing the same data dir;
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
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{broadcast, oneshot, watch};

use crate::api::{
    API_REVISION, ApiState, COMPLETION_POLL_INTERVAL, Core, CoreError, CoreGuard,
    drive_completions, router,
};
use crate::backend::claude::{CLAUDE_BACKEND_NAME, ClaudeBackend, ClaudeConfig};
use crate::backend::docker::{DOCKER_BACKEND_NAME, DockerBackend, DockerConfig};
use crate::backend::fake::FAKE_BACKEND_NAME;
use crate::backend::{BackendRegistry, EventSink};
use crate::domain::estate::Estate;
use crate::domain::event::{EventDraft, EventSource};
use crate::platform::fs_locking::{self, Reliability};
use crate::runtime::analytics::{Analytics, AnalyticsError};
use crate::runtime::engine::{Engine, EngineError};
use crate::runtime::fsutil::{create_dir_all_durable, take_exclusive_lock, write_atomic_secret};
use crate::runtime::journal::{DEFAULT_SEGMENT_MAX_BYTES, Journal, JournalError};
use crate::runtime::projection::ProjectionError;
use crate::runtime::recovery;
use crate::runtime::startup::{self, StartupError};
use crate::telemetry::{Telemetry, TelemetryConfig, TelemetryError};

/// Runtime descriptor file name inside the data dir.
pub const DESCRIPTOR_FILE: &str = "runtime.json";
/// Exclusive daemon lock file name inside the data dir.
pub const DAEMON_LOCK_FILE: &str = "daemon.lock";
/// Schema identifier for the runtime descriptor.
pub const DESCRIPTOR_SCHEMA: &str = "sergeant.runtime/v2";

/// Event kind: the daemon came up and owns the journal.
pub const KIND_DAEMON_STARTED: &str = "daemon.started";
/// Event kind: the daemon shut down cleanly.
pub const KIND_DAEMON_STOPPED: &str = "daemon.stopped";
/// Event kind: a backend was registered and probed (§15 PROBE, recorded at
/// registration as the M4 contract requires).
pub const KIND_BACKEND_PROBED: &str = "backend.probed";

/// Event kind: admission paused — new `POST /v1/work` submissions are
/// refused until [`KIND_ADMISSION_RESUMED`] (MVP-3's drain flag, scoped
/// exactly to `sgt daemon stop`: pause, wait for in-flight work to finish,
/// then send the ordinary SIGTERM shutdown). Idempotent at the API layer
/// (`pause_admission` in `api.rs` journals nothing new if already paused),
/// which is what keeps a `daemon stop` retry from double-pausing — see
/// [`KIND_ADMISSION_RESUMED`]'s doc for the other half of L6's crash window.
pub const KIND_ADMISSION_PAUSED: &str = "admission.paused";
/// Event kind: admission resumed — the complement of
/// [`KIND_ADMISSION_PAUSED`].
///
/// **L6's crash window, closed by construction rather than by recovery
/// logic.** A daemon can die after journaling `admission.paused` but before
/// `sgt daemon stop` ever sends SIGTERM (killed mid-drain, or the CLI itself
/// was killed while waiting). A single `bool` folded from these two event
/// kinds is durable, so a naive restart would replay straight into "admission
/// still paused" forever — a fresh process that was never mid-drain itself,
/// permanently refusing all new work with no operator ever having asked for
/// that. `start_with` closes this the same way `reconcile_crashed_start`
/// closes its own crash windows: **unconditionally**, journal
/// `admission.resumed` once at startup whenever the freshly-replayed state
/// shows paused, before the descriptor is published and before any request
/// can be served. This is safe rather than a guess because admission-paused
/// is a fact about *this process's* live drain, and a new process starting
/// is unambiguous proof no drain by *it* is in progress — unlike Work-state
/// recovery (§25), there is no ambiguous case here to fail closed on.
pub const KIND_ADMISSION_RESUMED: &str = "admission.resumed";

/// The runtime descriptor published for clients (proposal §6, estate-root
/// §5.1): endpoint, PID, API revision, the bearer token, and — since
/// `sergeant.runtime/v2` — **the estate this daemon is bound to**, protected
/// by owner-only file permissions.
///
/// §5.1: "A client validates that the descriptor's root matches its exact
/// current root before using the endpoint. A live daemon bound to another
/// estate is a named refusal, never a reusable global service." That check
/// is [`RuntimeDescriptor::check_estate_root`], and every client path
/// (`ensure_daemon`, `observe_connect`) runs it before the endpoint is used
/// for anything.
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
    /// §5.1: the canonical estate root this daemon was started against.
    /// `None` only for a daemon bound to no estate at all — a test rig, or
    /// `sgt daemon` run somewhere that is not an estate root.
    #[serde(default)]
    pub estate_root: Option<PathBuf>,
    /// §5.1: `<estate_root>/sergeant.toml`, recorded alongside the root so a
    /// reader never has to reconstruct it.
    #[serde(default)]
    pub manifest_path: Option<PathBuf>,
}

/// A live daemon bound to a different estate than the client's own exact
/// root (§5.1, §15): named refusal, never a connection, and never a second
/// daemon over the same data dir.
#[derive(Debug, thiserror::Error)]
#[error(
    "the daemon running for {data_dir} is bound to a different estate\n\n\
     This estate root:\n  {wanted}\n\n\
     Daemon's estate root:\n  {found}\n\n\
     Refusing to connect, and refusing to start a second daemon over the same data dir.\n\
     Stop the running daemon and retry, or point this estate at its own data dir:\n  \
     sgt -C {found} daemon stop\n  \
     sgt --data-dir <dir> <command>"
)]
pub struct EstateBindingMismatch {
    /// Data dir both would own.
    pub data_dir: String,
    /// The root the client resolved.
    pub wanted: String,
    /// The root the running daemon is bound to, or the fact that it has
    /// none.
    pub found: String,
}

impl RuntimeDescriptor {
    /// §5.1's client-side gate: does this descriptor's estate root match the
    /// caller's own exact root?
    ///
    /// Both directions are refusals, deliberately: a daemon bound to estate
    /// A cannot serve a client standing in estate B, **and** a daemon bound
    /// to no estate cannot serve a client that has one (its engine would
    /// plan against nothing, so every submission would silently land
    /// `pending`). A client with no estate root of its own — no such client
    /// exists on the estate-scoped paths, since §4.3 admits the root first —
    /// is the only case that passes without comparison.
    pub fn check_estate_root(
        &self,
        data_dir: &Path,
        wanted: Option<&Path>,
    ) -> Result<(), EstateBindingMismatch> {
        let Some(wanted) = wanted else {
            return Ok(());
        };
        let matches = self
            .estate_root
            .as_deref()
            .is_some_and(|bound| bound == wanted);
        if matches {
            return Ok(());
        }
        Err(EstateBindingMismatch {
            data_dir: data_dir.display().to_string(),
            wanted: wanted.display().to_string(),
            found: match &self.estate_root {
                Some(root) => root.display().to_string(),
                None => "(none — this daemon was started outside any estate)".to_string(),
            },
        })
    }
}

/// Errors from daemon startup and shutdown.
#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    /// Another live daemon holds this data dir's exclusive lock.
    #[error("another daemon already owns this data dir (daemon.lock is held)")]
    Locked,
    /// #85, ADR 0003 D6: the data dir sits on a filesystem where advisory
    /// locking (`flock`) is unreliable, so the exclusive lock above cannot be
    /// trusted to actually exclude a second daemon. Refused before the lock
    /// is even attempted, never merely warned about.
    #[error(
        "{data_dir} sits on a {filesystem} filesystem, where advisory locking is unreliable; refusing to start there — {remedy}"
    )]
    UnreliableFilesystem {
        /// The data dir that was refused.
        data_dir: PathBuf,
        /// The offending filesystem type, as the mount table names it.
        filesystem: String,
        /// What to do about it.
        remedy: String,
    },
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
    /// W2: the shared startup replay or its cache failed.
    #[error(transparent)]
    Startup(#[from] StartupError),
    /// The §28 export pipeline could not be built from its configuration.
    #[error(transparent)]
    Telemetry(#[from] TelemetryError),
    /// §4.1: the estate root this daemon was started against is not one.
    /// Refused before the data dir is created, the journal opened, or the
    /// descriptor published — a daemon that cannot name its estate never
    /// comes up.
    #[error(transparent)]
    EstateRoot(#[from] crate::domain::estate::EstateRootError),
    /// The descriptor names a schema this build does not understand. Fail
    /// closed exactly as an unknown snapshot schema does: its fields may
    /// mean something else entirely, and acting on them could mean talking
    /// to the wrong process — or spawning a second daemon.
    ///
    /// **0.1.0, estate-root Phase D:** `sergeant.runtime/v1` descriptors are
    /// exactly this case. A v1 descriptor carries no `estate_root`, so a
    /// client could not verify §5.1's binding against it at all — it fails
    /// closed here rather than being read half-way, and the remedy is to
    /// stop the old daemon and let a v2 one publish. There is deliberately no
    /// compatibility shim: at 0.1.0, "the daemon you are talking to is from
    /// another build" is exactly the fact a client must not paper over.
    #[error(
        "runtime descriptor {path} declares unknown schema {found:?} (this build understands {expected:?}); \
         refusing to use it. If a daemon from an older build is still running, stop it \
         (`sgt daemon stop`, or kill its pid) and retry — a restarted daemon republishes the \
         descriptor in the schema this build reads."
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

/// How long shutdown waits for the completion driver to leave whatever it is
/// inside before journaling `daemon.stopped` and going.
///
/// Half of m6's SIGTERM grace: long enough that the ordinary case (one poll
/// interval, or one crank over in-memory observations) always finishes inside
/// it, short enough that a daemon whose driver is stuck in someone else's git
/// checkout still exits on its own rather than being escalated to SIGKILL.
const DRIVER_SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

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
    /// Launch configuration for the Docker adapter this daemon registers
    /// itself (N4). `None` is the ordinary `docker` on `PATH`, adapter state
    /// under this data dir — mirrors `claude` above for the same reason:
    /// tests point it at a scripted `docker_bin` to drive the real adapter's
    /// request path without a real Docker Engine.
    pub docker: Option<DockerConfig>,
    /// §28 OpenTelemetry export. `None` is **off**, and off is the default:
    /// with no pipeline here the daemon builds no provider, spawns no
    /// exporter task, and subscribes nothing to the event stream.
    pub telemetry: Option<Arc<Telemetry>>,
    /// How often the completion driver looks for turns that ended on their
    /// own (issue #46; see [`api::drive_completions`]).
    ///
    /// Configurable for the same reason `backends` is: a test that needs to
    /// stand *inside* the window between a turn ending and the daemon settling
    /// it has no other way to hold the window open, and the alternative — a
    /// sleep racing the driver — is the kind of test that passes for the wrong
    /// reason. Production uses the default.
    pub completion_poll: Duration,
    /// R-MVP1-7's per-turn wall-clock ceiling. Configurable for the same
    /// reason `completion_poll` is: a test proving a hung turn is
    /// interrupted within "ceiling + one interval" needs to hold both knobs
    /// down to something a test budget can wait out, and racing the
    /// production default (minutes) would make the test the thing that
    /// times out. Production uses the default.
    pub turn_ceiling: Duration,
    /// R-MVP1-1's daemon-wide surfaces-root override (`[estate]
    /// surfaces_dir` / `SGT_SURFACES_DIR`): `None` keeps
    /// [`Engine::new`]'s default, `<data_dir>/surfaces` — today's layout,
    /// nothing moves. `Some` is wired into the engine via
    /// [`Engine::with_surfaces_root`] in [`start_with`]. A per-estate
    /// `[estate] surfaces_dir` narrows this further, per-submission, in
    /// `Engine::plan` — this is only the daemon-wide fallback beneath that.
    pub surfaces_root: Option<PathBuf>,
    /// R-MVP1-7's daemon-wide turn-cap override (`SGT_TURN_CAP`): `None`
    /// keeps [`Engine::new`]'s default ([`crate::runtime::engine::
    /// DEFAULT_TURN_CAP`]). `Some` is wired into the engine via
    /// [`Engine::with_turn_cap`] in [`start_with`], the same way
    /// `turn_ceiling` and `surfaces_root` above are. R-MVP1-10's
    /// `Engine::extend_turn_envelope` is the per-Work door beneath this
    /// daemon-wide default.
    pub turn_cap: Option<u32>,
    /// W2 (`sgt daemon --rebuild-cache`): ignore any existing
    /// `projections/floor-state.json`, rebuild from a full floor-aware
    /// replay, and write a fresh one. `false` — the default and every
    /// auto-spawn — uses the cache when it verifies.
    pub rebuild_cache: bool,
    /// Segment rotation threshold for this daemon's journal. `None` is
    /// [`DEFAULT_SEGMENT_MAX_BYTES`] (8 MiB) — production, always.
    ///
    /// Configurable for the same reason `completion_poll` and `turn_ceiling`
    /// are: W2's I9 and floor-fallback tests need a journal with more than
    /// [`crate::runtime::startup::STARTUP_WINDOW_SEGMENTS`] segments, which
    /// at the production threshold is far outside a test budget. The
    /// alternative — shrinking the window instead — would mean the tests
    /// prove a window nothing ships.
    pub segment_max_bytes: Option<u64>,
    /// §5.1: the estate root this daemon is bound to, for its whole life.
    ///
    /// [`start_with`] admits it (§4.1's exact-root check —
    /// [`Estate::admit`]) before the data dir is created, the journal
    /// opened, or the descriptor published, so a daemon can never come up
    /// bound to something that is not an estate. The canonical form is
    /// pinned into the engine and published in the runtime descriptor, where
    /// every client verifies it against its own root before use.
    ///
    /// `None` is a daemon bound to no estate: `Engine::plan` answers "no
    /// repository context" and every submission stays `pending`. That is the
    /// test rigs' shape — never a silent fallback to discovery, of which
    /// there is none left.
    pub estate_root: Option<PathBuf>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            backends: Arc::new(BackendRegistry::default_registry()),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            docker: None,
            telemetry: None,
            completion_poll: COMPLETION_POLL_INTERVAL,
            turn_ceiling: crate::runtime::engine::DEFAULT_TURN_CEILING,
            surfaces_root: None,
            turn_cap: None,
            rebuild_cache: false,
            segment_max_bytes: None,
            estate_root: None,
        }
    }
}

/// Start the daemon on `data_dir` with the default backend registry.
pub async fn start(data_dir: &Path) -> Result<DaemonHandle, DaemonError> {
    start_with(data_dir, DaemonConfig::default()).await
}

/// #85, ADR 0003 D6: turn a filesystem-reliability verdict into the daemon's
/// refusal, or lack of one. Split out from the call to
/// [`fs_locking::detect_for_path`] itself so this decision — refuse only on
/// a *confirmed* bad filesystem, never on an inconclusive probe — is
/// testable without a real `drvfs`/NFS/SMB mount anywhere in the test
/// sandbox (`platform::fs_locking`'s own tests cover the detection half).
fn refuse_if_unreliable(data_dir: &Path, reliability: Reliability) -> Result<(), DaemonError> {
    match reliability {
        Reliability::Unreliable { filesystem } => Err(DaemonError::UnreliableFilesystem {
            data_dir: data_dir.to_path_buf(),
            remedy: fs_locking::remedy(&filesystem),
            filesystem,
        }),
        Reliability::Reliable | Reliability::Unknown { .. } => Ok(()),
    }
}

/// Start the daemon on `data_dir`. Returns once it is serving and the
/// runtime descriptor is published.
pub async fn start_with(
    data_dir: &Path,
    config: DaemonConfig,
) -> Result<DaemonHandle, DaemonError> {
    // 0a. §4.1/§5.1: admit the estate root **first**, before the data dir is
    // even created. A daemon that cannot name its estate must not come up at
    // all — §4.3's ordering applies to the daemon's own startup exactly as
    // it does to a client's dispatch, and the canonical root admitted here
    // is what the descriptor publishes and every client verifies against.
    let estate = match &config.estate_root {
        Some(root) => Some(Estate::admit(root)?),
        None => None,
    };

    create_dir_all_durable(data_dir)?;

    // 0. #85 / ADR 0003 D6: refuse outright on a filesystem where advisory
    // locking is unreliable — before the lock below is even attempted, since
    // a lock that silently does not hold fails in a way nobody notices until
    // two daemons are racing the same data dir. An inconclusive probe (e.g.
    // today's always-Unknown macOS arm) does not refuse — see
    // `refuse_if_unreliable`.
    refuse_if_unreliable(data_dir, fs_locking::detect_for_path(data_dir))?;

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

    // 2. Own the journal (§24). Recovers a crashed tail and learns the next
    // seq from the last non-empty segment alone (`Journal::open_with`'s own
    // doc explains why that is still fully seq-validated).
    let rebuild_started = Instant::now();
    let mut journal = Journal::open_with(
        data_dir,
        config
            .segment_max_bytes
            .unwrap_or(DEFAULT_SEGMENT_MAX_BYTES),
    )?;

    // §28 export, when it is switched on. The journal's append timing is the
    // one metric whose input exists nowhere else, so the observer is
    // installed here — before anything below can append through this handle.
    if let Some(telemetry) = &config.telemetry {
        let telemetry = telemetry.clone();
        journal.set_append_observer(Arc::new(move |elapsed| {
            telemetry.record_journal_append(elapsed);
        }));
    }

    // 2a-2b. W2's one shared startup pass: collapses what used to be four
    // separate full replays (`next_seq`'s own scan above, the Work registry,
    // the analytical projection, the claude capability watermark) into one
    // `startup::drive` over one `Replay` iterator, windowed by a persisted,
    // purely-derived cache of everything older than the window
    // (`runtime::startup` owns the whole mechanism; see its module doc).
    let plan = startup::Plan::resolve(data_dir, &journal, config.rebuild_cache)?;
    let mut registry = plan.seed_registry();
    // The disposable analytical projection (§21–§23, §40): rebuilt from the
    // journal on every start (windowed exactly like every other sink), so
    // deleting it and restarting is indistinguishable from restarting.
    let mut analytics = Analytics::begin_rebuild(data_dir)?;
    let mut capability_sink = startup::CapabilitySink {
        latest: plan.capability_seed(),
    };
    // Loaded once, never mutated by the pass (§26 Q8's below-window ledger) —
    // a separate seed from the one `LedgerSink` folds forward, so `Core`'s
    // copy always names exactly the below-window keys the cache carried in,
    // never anything the pass itself re-derived.
    let floor_ledger = plan.ledger_seed();
    let mut ledger_sink = startup::LedgerSink::seeded(floor_ledger.clone());
    let mut horizon_sink = startup::HorizonSink::default();
    let mut report = {
        let mut analytics_sink = startup::AnalyticsSink::new(analytics.fold()?, plan.window_seq());
        let report = startup::drive(
            plan.replay(&journal)?,
            &mut [
                &mut startup::RegistrySink(&mut registry),
                &mut analytics_sink,
                &mut capability_sink,
                &mut ledger_sink,
                &mut horizon_sink,
            ],
        )?;
        // `analytics_sink`'s `finish()` already ran inside `drive` above;
        // dropping it here ends its borrow of `analytics` before the cache
        // write below (and `analytics`'s later move into `ApiState`) need
        // one of their own.
        drop(analytics_sink);
        report
    };
    // `PassReport::from_seq`'s own doc explains why `drive` cannot recover
    // this when it sees zero events (a cache-hit restart with nothing
    // appended since the cache was written): the plan itself always knows.
    report.from_seq = plan.from_seq();

    // 2c. §2.6: the cache this start should leave behind — hit or miss,
    // extending it is always sound, since everything a new cache needs is
    // already in hand from the pass just run. Written under the daemon lock,
    // before the listener binds and before any event is appended, so it
    // always describes a prefix of the journal as it stood before this
    // process's first append.
    let floor_state = plan.next_cache(
        &journal,
        registry.state(),
        &ledger_sink,
        &capability_sink,
        &horizon_sink,
    )?;
    startup::persist_or_remove(floor_state.as_ref(), data_dir)?;
    let rebuild_ms = u64::try_from(rebuild_started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let startup_cache = plan.startup_cache_tag();
    // The oldest surviving seq (1 on every W2 production path — the floor
    // only moves above 1 once W3's prune lands).
    let replay_floor_seq = journal.floor_seq()?.unwrap_or(1);

    let (events_tx, _) = broadcast::channel(1024);
    let mut core =
        Core::new(journal, registry, events_tx).with_floor_ledger(Arc::new(floor_ledger));

    // 3. Bind loopback on an ephemeral port before publishing anything.
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let endpoint = format!("http://{}", listener.local_addr()?);

    // 4. Lifecycle event: the journal records that this daemon took over.
    core.commit(EventDraft::new(
        EventSource::new("daemon", "sergeant"),
        KIND_DAEMON_STARTED,
        json!({
            "pid": std::process::id(),
            "version": env!("CARGO_PKG_VERSION"),
            "endpoint": endpoint,
            // W2 (Q1/Q4): what the one shared startup pass cost, so
            // `sgt doctor` can re-argue the fixed window from evidence
            // rather than adapting it silently.
            "rebuild_duration_ms": rebuild_ms,
            "replayed_events": report.replayed_events,
            "startup_cache": startup_cache,
            "replay_floor_seq": replay_floor_seq,
            "replay_from_seq": report.from_seq,
        }),
    ))?;
    // `core` is bare here — no `CoreGuard`, because nothing else can see it
    // yet — so nothing makes this durable but an explicit flush (invariants
    // round 2, INV-R2-01). Startup is the one path that performs unbounded,
    // irreversible external effects (backend probes below, and recovery's
    // `git worktree remove` / harness relaunch further down) with no client
    // and no guard to fsync on drop; flushing after every commit that
    // precedes such an effect keeps the "durable before it could have been
    // observed" property `CoreGuard` gives every other path.
    core.flush()?;

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
        // MVP-2 D2 item 2 (capability provenance durability, cv2 item 7):
        // seed the fresh in-memory `ask` claim from the journal's own record
        // of a prior withdrawal *before* anything reads `capabilities()` —
        // the registration probe two steps below, first of all — so a
        // capability this exact CLI version already proved absent stays
        // withdrawn across a restart instead of re-defaulting to optimistic
        // on every fresh process. W2 replaces this walk's own separate
        // journal replay with the shared pass's own `CapabilitySink`: the
        // *fold* moved to step 2b, above; the *application* stays here,
        // still before the registration probe reads `capabilities()`.
        adapter.seed_capability_provenance_from(capability_sink.latest.as_ref());
        backends = backends.with(adapter.clone());
        Some(adapter)
    } else {
        None
    };
    // N4: the Docker executor, registered the same way and for the same
    // reason as Claude above — it needs this data dir (image pins, its own
    // blob-store instance) and an event sink that only exists once the core
    // does. `DockerBackend::new` only opens its blob store (durable,
    // infallible in ordinary operation); it never touches the Docker socket,
    // so a host with no Docker installed still starts a daemon that can
    // route actor-only workflows (§17.5's "degraded daemon, strict work
    // admission" — routing an execute stage to this backend is what actually
    // fails, at submit time, via its `probe()`).
    let docker = if config.backends.get(DOCKER_BACKEND_NAME).is_none() {
        let docker_config = config
            .docker
            .clone()
            .unwrap_or_else(|| DockerConfig::new(data_dir));
        let adapter = Arc::new(DockerBackend::new(docker_config)?);
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
    // Same reasoning as the daemon.started flush above: durable before
    // recovery starts acting on it (INV-R2-01).
    core.flush()?;

    // 4c. Reconcile work believed in flight *before* serving (§25): no
    // request may observe — or act on — a work whose prior ownership has not
    // yet been settled. `reconcile` itself flushes after each work it
    // touches, so its own effects (a `git worktree remove`, a relaunched
    // harness) are never left unsynced while unbounded — see its doc.
    let mut engine = Engine::new(backends, config.default_backend.clone(), data_dir)
        .with_turn_ceiling(config.turn_ceiling);
    // §5.2: the engine's one topology authority for the life of this
    // process. Nothing downstream ever re-derives it from a request cwd.
    if let Some(estate) = &estate {
        engine = engine.with_estate_root(estate.path.clone());
    }
    if let Some(surfaces_root) = config.surfaces_root.clone() {
        engine = engine.with_surfaces_root(surfaces_root);
    }
    if let Some(turn_cap) = config.turn_cap {
        engine = engine.with_turn_cap(turn_cap);
    }
    let engine = Arc::new(engine);
    let reconciled = recovery::reconcile(&engine, &mut core)?;
    // Backstop, not load-bearing: `reconcile` already leaves nothing open on
    // its own account, but a future edit there that forgets a flush must not
    // be able to publish the descriptor over an unsynced group. Free when
    // the group is already empty.
    core.flush()?;
    if !reconciled.resumed.is_empty()
        || !reconciled.blocked.is_empty()
        || !reconciled.surfaces_retired.is_empty()
        || !reconciled.reservations_retired.is_empty()
    {
        tracing::info!(
            resumed = ?reconciled.resumed,
            blocked = ?reconciled.blocked,
            surfaces_retired = ?reconciled.surfaces_retired,
            reservations_retired = ?reconciled.reservations_retired,
            "reconciled in-flight work after restart"
        );
    }

    // 4c-ii. Clear a stale admission pause (L6, `KIND_ADMISSION_RESUMED`'s
    // own doc): a predecessor that died mid-drain can leave the replayed
    // state paused with nothing left to ever resume it. This process was
    // never mid-drain itself, so the fact is unambiguous — resume
    // unconditionally, before the descriptor is published and before any
    // request can be served, exactly like the backend probes and recovery
    // above.
    if core.registry.state().admission_paused {
        core.commit(EventDraft::new(
            EventSource::new("daemon", "sergeant"),
            KIND_ADMISSION_RESUMED,
            json!({"reason": "startup: a fresh process was never mid-drain"}),
        ))?;
        core.flush()?;
        tracing::info!("cleared an admission pause inherited from a previous process life");
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
        estate_root: estate.as_ref().map(|e| e.path.clone()),
        manifest_path: estate.as_ref().map(|e| e.manifest_path.clone()),
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
        let events = CoreGuard::acquire(&state.core).await.events_tx.subscribe();
        tokio::spawn(export_events(telemetry, events));
    }
    // The Claude adapter's normalized events (§27) flow into the journal
    // through the core; the sink can only exist now that the core is shared.
    if let Some(claude) = claude {
        claude.set_event_sink(journaling_sink(state.core.clone()));
    }
    // The Docker adapter's own provenance events (`execute.image_resolved`)
    // flow through the identical sink (same reasoning as Claude's, directly
    // above).
    if let Some(docker) = docker {
        docker.set_event_sink(journaling_sink(state.core.clone()));
    }
    let app = router(state.clone());

    // The one writer that is not a request (issue #46): a turn that ends on
    // its own has no client to crank the engine, so the daemon carries the
    // observer itself. It is joined at shutdown below rather than merely
    // dropped, because it commits events and `daemon.stopped` must be the last
    // one this daemon writes.
    let completions = tokio::spawn(drive_completions(state.clone(), config.completion_poll));

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
        // The completion driver watches the same `closing` flag the SSE pumps
        // do, so this normally waits out one 200 ms tick and nothing else.
        //
        // Bounded, though, because the driver is the one writer that can be
        // *inside an external effect* when the signal arrives: a cascade it
        // started may be spawning a harness or running `git worktree remove`
        // on a checkout this daemon does not own and cannot bound. Waiting
        // that out unbounded would put an arbitrary repository's size on the
        // shutdown path — and m6's rig pins that a SIGTERMed daemon exits
        // within its grace and *by itself*, so a slow teardown would turn into
        // a SIGKILL and lose everything registered to run at exit.
        //
        // The trade the bound makes, stated rather than hidden: past it, the
        // driver's crank may commit an event after `daemon.stopped`. That is
        // the property the adapter's normalized-event committer thread already
        // has (see `journaling_sink`), the journal is append-only and
        // crash-tolerant per append, and `daemon.stopped` is a lifecycle
        // record that nothing reconciles against — whereas a daemon that will
        // not die is a lock nobody can take.
        match tokio::time::timeout(DRIVER_SHUTDOWN_GRACE, completions).await {
            Ok(Err(e)) => tracing::warn!(error = %e, "the completion driver panicked"),
            Err(_) => tracing::warn!(
                grace = ?DRIVER_SHUTDOWN_GRACE,
                "the completion driver was still inside an external effect at shutdown; \
                 stopping anyway"
            ),
            Ok(Ok(())) => {}
        }
        // Clean shutdown: journal the stop, then retire the descriptor.
        let mut core = CoreGuard::acquire(&state.core).await;
        if let Err(e) = core.commit(EventDraft::new(
            EventSource::new("daemon", "sergeant"),
            KIND_DAEMON_STOPPED,
            json!({"pid": std::process::id()}),
        )) {
            tracing::warn!(error = %e, "failed to journal daemon.stopped");
        }
        // Close the group here rather than leaving it to the guard's `Drop`,
        // only so the failure is warned about in the same voice as the append
        // above; the descriptor must not be retired over an unflushed journal.
        if let Err(e) = core.flush() {
            tracing::warn!(error = %e, "failed to fsync the shutdown group commit");
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
        let Some(live) = core.upgrade() else {
            tracing::debug!("normalized-event committer stopping: the core is gone");
            return;
        };
        let key = draft.execution_id.clone();
        if draft.causation_id.is_none()
            && let Some(key) = &key
        {
            draft.causation_id = chain.get(key).cloned();
        }
        // One draft per hold, so this thread's "group" is a single event and
        // its fsync is the same one it always paid. It goes through the guard
        // anyway (#44): the group boundary is a property of holding the core,
        // not of who is holding it, and a second door into the core is exactly
        // what `t11c` exists to prevent.
        let mut core = CoreGuard::acquire_blocking(&live);
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
pub async fn run_until_signal(
    data_dir: &Path,
    estate_root: Option<&Path>,
    rebuild_cache: bool,
) -> Result<(), DaemonError> {
    // **Handlers first, before anything makes this daemon reachable.**
    //
    // Publishing the runtime descriptor is what tells the world "there is a
    // daemon here"; clients and test rigs alike wait for exactly that file and
    // then start acting. Until a SIGTERM handler is installed, SIGTERM's
    // default disposition applies and the process is simply terminated —
    // nothing journals `daemon.stopped`, the descriptor is left pointing at a
    // dead pid, and nothing registered at exit runs. Installing after
    // `start_with` left that window open from the descriptor write to here.
    //
    // Measured, 2026-08-11 (Cerberus): m6's
    // `the_spawned_daemon_rig_stops_its_daemon_with_sigterm` failed with
    // `status.signal() == Some(15)` — killed *by* the signal — on run 3 of 40
    // m6 runs executed against a concurrently running suite. The rig returns
    // the instant the descriptor appears and signals immediately, so it lands
    // in that window whenever the scheduler lets it. The window predates the
    // completion driver, and the driver's spawn is more work inside it.
    let mut shutdown = ShutdownSignals::install();
    let telemetry_config = TelemetryConfig::from_env();
    let telemetry = Telemetry::from_config(&telemetry_config)?.map(Arc::new);
    if telemetry.is_some() {
        tracing::info!(
            endpoint = telemetry_config.endpoint(),
            "otel export enabled"
        );
    }
    // R-MVP1-1's daemon-wide surfaces-root override: read once at startup,
    // exactly as `Engine::with_surfaces_root`'s own doc has always promised.
    // An empty value is treated the same as unset (`SGT_SURFACES_DIR=`
    // exported-but-blank must not silently relocate every surface to the
    // data dir's own root).
    let surfaces_root = std::env::var_os("SGT_SURFACES_DIR")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty());
    // R-MVP1-7's daemon-wide turn-cap override: same discipline as
    // SGT_SURFACES_DIR above — unset or unparseable both mean "keep the
    // built-in default" (fail open to the default, never refuse the whole
    // daemon start over a malformed env var only an operator can fix by
    // restarting anyway). Failing open must not fail silently: the warn is
    // the only trace an operator gets that their override never applied,
    // and 0 — which parses — blocks every Work at its first turn spawn.
    let turn_cap = std::env::var("SGT_TURN_CAP")
        .ok()
        .and_then(|v| match v.parse::<u32>() {
            Ok(0) => {
                tracing::warn!(
                    "SGT_TURN_CAP=0 blocks every Work at its first turn spawn; honoring it"
                );
                Some(0)
            }
            Ok(n) => Some(n),
            Err(_) => {
                tracing::warn!(
                    value = %v,
                    "SGT_TURN_CAP is not a whole number of turns — keeping the built-in default"
                );
                None
            }
        });
    let handle = start_with(
        data_dir,
        DaemonConfig {
            telemetry: telemetry.clone(),
            surfaces_root,
            turn_cap,
            // §5.1: the estate this daemon is bound to for its whole life,
            // handed down from the invocation that already admitted it
            // (`sgt -C <root> daemon`, or `sgt daemon` from the root).
            estate_root: estate_root.map(Path::to_path_buf),
            rebuild_cache,
            ..DaemonConfig::default()
        },
    )
    .await?;
    tracing::info!(endpoint = %handle.endpoint, data_dir = %data_dir.display(), "daemon serving");
    shutdown.recv().await;
    tracing::info!("shutdown signal received");
    handle.shutdown().await;
    if let Some(telemetry) = telemetry {
        telemetry.shutdown();
    }
    Ok(())
}

/// Installed shutdown-signal handlers: SIGINT (Ctrl-C) and, on Unix, SIGTERM.
///
/// Split from the waiting so the *installation* can happen before the daemon
/// publishes anything — see [`run_until_signal`]'s first lines for the
/// measurement that made the ordering load-bearing. Constructing this
/// registers the handlers; delivery is queued from that moment, so a signal
/// that arrives during startup is still delivered to [`Self::recv`] later
/// rather than terminating the process.
struct ShutdownSignals {
    #[cfg(unix)]
    interrupt: Option<tokio::signal::unix::Signal>,
    #[cfg(unix)]
    terminate: Option<tokio::signal::unix::Signal>,
}

impl ShutdownSignals {
    fn install() -> Self {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            // Both explicitly, rather than leaning on `ctrl_c()`: that helper
            // registers SIGINT when its future is first polled, which is the
            // very thing this type exists to move earlier.
            //
            // Each registration is kept independently. Tokio's Unix signal
            // registration is process-global and is not undone when the
            // `Signal` is dropped, so discarding a successfully registered
            // stream because the *other* one failed would leave that signal
            // consumed by tokio's handler with nothing receiving it — a
            // daemon unkillable by exactly the signal that did register.
            let take = |kind: SignalKind, name: &str| match signal(kind) {
                Ok(stream) => Some(stream),
                Err(e) => {
                    tracing::error!(
                        error = ?e,
                        signal = name,
                        "cannot install shutdown signal handler"
                    );
                    None
                }
            };
            Self {
                interrupt: take(SignalKind::interrupt(), "SIGINT"),
                terminate: take(SignalKind::terminate(), "SIGTERM"),
            }
        }
        #[cfg(not(unix))]
        {
            Self {}
        }
    }

    /// Resolve on the first shutdown signal delivered since installation.
    async fn recv(&mut self) {
        #[cfg(unix)]
        match (self.interrupt.as_mut(), self.terminate.as_mut()) {
            (Some(interrupt), Some(terminate)) => {
                tokio::select! {
                    _ = interrupt.recv() => {}
                    _ = terminate.recv() => {}
                }
                return;
            }
            // A signal whose registration failed keeps its default
            // disposition (terminate the process), so waiting only on the
            // stream that exists loses nothing.
            (Some(interrupt), None) => {
                let _ = interrupt.recv().await;
                return;
            }
            (None, Some(terminate)) => {
                let _ = terminate.recv().await;
                return;
            }
            (None, None) => {}
        }
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

/// Whether a PID currently names a live process. Platform fact — see
/// [`crate::platform::process::process_alive`] (#18) for the per-platform
/// mechanism and its fail-closed posture elsewhere.
pub fn pid_alive(pid: u32) -> bool {
    crate::platform::process::process_alive(pid)
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

    /// A `sergeant.runtime/v2` descriptor bound to `estate_root`.
    fn descriptor_bound_to(estate_root: Option<&str>) -> RuntimeDescriptor {
        RuntimeDescriptor {
            schema: DESCRIPTOR_SCHEMA.to_string(),
            endpoint: "http://127.0.0.1:1".to_string(),
            pid: 1,
            api_revision: API_REVISION.to_string(),
            token: "t".to_string(),
            estate_root: estate_root.map(PathBuf::from),
            manifest_path: estate_root.map(|r| PathBuf::from(r).join("sergeant.toml")),
        }
    }

    /// §5.1: a descriptor whose estate root matches the client's own exact
    /// root is usable; one bound elsewhere is a refusal naming **both**
    /// roots, in both directions.
    #[test]
    fn a_descriptor_is_usable_only_from_the_estate_it_is_bound_to() {
        let data_dir = Path::new("/data");
        let here = descriptor_bound_to(Some("/estates/payments"));

        here.check_estate_root(data_dir, Some(Path::new("/estates/payments")))
            .expect("the daemon's own estate may use it");

        let err = here
            .check_estate_root(data_dir, Some(Path::new("/estates/billing")))
            .expect_err("another estate must be refused");
        let message = err.to_string();
        assert!(
            message.contains("/estates/billing") && message.contains("/estates/payments"),
            "§15: the refusal names both roots: {message}"
        );
        assert!(
            message.contains("refusing to start a second daemon"),
            "§5.1: never a second daemon over the same data dir: {message}"
        );

        // The other direction: a daemon bound to nothing cannot serve a
        // client that has an estate — its engine would plan against nothing.
        let unbound = descriptor_bound_to(None);
        let err = unbound
            .check_estate_root(data_dir, Some(Path::new("/estates/payments")))
            .expect_err("an unbound daemon must be refused too");
        assert!(
            err.to_string().contains("started outside any estate"),
            "got {err}"
        );
    }

    /// The v1→v2 bump has no compatibility shim: a descriptor written by an
    /// older build fails closed as an unknown schema, and the diagnostic
    /// names the stop/restart remedy rather than leaving an operator staring
    /// at a schema string.
    #[test]
    fn a_v1_descriptor_fails_closed_with_a_restart_remedy() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::write(
            dir.path().join(DESCRIPTOR_FILE),
            serde_json::json!({
                "schema": "sergeant.runtime/v1",
                "endpoint": "http://127.0.0.1:1",
                "pid": 1,
                "api_revision": API_REVISION,
                "token": "t",
            })
            .to_string(),
        )
        .expect("write v1 descriptor");

        let err = read_descriptor(dir.path()).expect_err("v1 must fail closed");
        let message = err.to_string();
        assert!(
            matches!(err, DaemonError::UnknownDescriptorSchema { .. }),
            "got {message}"
        );
        assert!(
            message.contains("sergeant.runtime/v1") && message.contains("sergeant.runtime/v2"),
            "both schemas must be named: {message}"
        );
        assert!(
            message.contains("sgt daemon stop"),
            "the remedy must tell the operator to restart the daemon: {message}"
        );
    }

    /// Complements `a_v1_descriptor_fails_closed_with_a_restart_remedy` above:
    /// the positive case for the *current* schema, going through the same
    /// on-disk file-write → `read_descriptor` path (not the in-memory
    /// `descriptor_bound_to()` helper, which never touches disk).
    #[test]
    fn a_v2_descriptor_round_trips_through_read_descriptor() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::write(
            dir.path().join(DESCRIPTOR_FILE),
            serde_json::json!({
                "schema": DESCRIPTOR_SCHEMA,
                "endpoint": "http://127.0.0.1:1",
                "pid": 1,
                "api_revision": API_REVISION,
                "token": "t",
                "estate_root": "/estates/payments",
                "manifest_path": "/estates/payments/sergeant.toml",
            })
            .to_string(),
        )
        .expect("write v2 descriptor");

        let descriptor = read_descriptor(dir.path())
            .expect("read must succeed")
            .expect("descriptor must be present");
        assert_eq!(descriptor.schema, DESCRIPTOR_SCHEMA);
        assert_eq!(descriptor.endpoint, "http://127.0.0.1:1");
        assert_eq!(descriptor.pid, 1);
        assert_eq!(descriptor.api_revision, API_REVISION);
        assert_eq!(descriptor.token, "t");
        assert_eq!(
            descriptor.estate_root.as_deref(),
            Some(Path::new("/estates/payments"))
        );
        assert_eq!(
            descriptor.manifest_path.as_deref(),
            Some(Path::new("/estates/payments/sergeant.toml"))
        );
    }

    /// A minimal, directly-constructed [`Core`] over a fresh journal — no
    /// HTTP surface, no engine, just what the committer and the export loop
    /// actually touch.
    fn test_core(data_dir: &Path) -> Core {
        let journal = Journal::open(data_dir).expect("open journal");
        let mut registry = crate::runtime::projection::work_registry_projection();
        registry
            .catch_up(journal.replay().expect("replay"))
            .expect("catch up");
        let (events_tx, _) = broadcast::channel(16);
        Core::new(journal, registry, events_tx)
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

    /// #85, ADR 0003 D6: `start_with`'s pre-lock refusal, tested at the seam
    /// that does not need a real `drvfs`/NFS/SMB mount — `refuse_if_unreliable`
    /// takes an already-computed [`Reliability`] rather than calling
    /// `fs_locking::detect_for_path` itself. Reverting the `Unreliable` match
    /// arm to also return `Ok(())` (i.e. dropping the refusal) fails this
    /// immediately.
    #[test]
    fn refuse_if_unreliable_blocks_on_a_confirmed_bad_filesystem() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let err = refuse_if_unreliable(
            dir.path(),
            Reliability::Unreliable {
                filesystem: "drvfs".to_string(),
            },
        )
        .expect_err("a confirmed-bad filesystem must be refused");
        match err {
            DaemonError::UnreliableFilesystem {
                data_dir,
                filesystem,
                remedy,
            } => {
                assert_eq!(data_dir, dir.path());
                assert_eq!(filesystem, "drvfs");
                assert!(
                    remedy.contains("drvfs"),
                    "the remedy must name the offending filesystem: {remedy}"
                );
            }
            other => panic!("expected UnreliableFilesystem, got {other:?}"),
        }
    }

    /// The other half of the asymmetry: neither an ordinary filesystem nor an
    /// inconclusive probe may refuse. Collapsing `Unknown` into the refusal
    /// arm would brick daemon start on the very platform (macOS, today)
    /// whose detection is unmeasured — this fails the moment that happens.
    #[test]
    fn refuse_if_unreliable_allows_reliable_and_unknown() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        assert!(refuse_if_unreliable(dir.path(), Reliability::Reliable).is_ok());
        assert!(
            refuse_if_unreliable(
                dir.path(),
                Reliability::Unknown {
                    reason: "cannot be measured on this platform".to_string(),
                },
            )
            .is_ok(),
            "an inconclusive probe must never refuse"
        );
    }
}

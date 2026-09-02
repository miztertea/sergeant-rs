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
//! 4. journal `daemon.started`, register the backend adapters, reconcile work
//!    believed in flight (§25) and clear a stale admission pause (L6) — no
//!    request may observe a Work whose prior ownership has not been settled,
//!    so those stay ahead of the descriptor;
//! 5. write the runtime descriptor (endpoint, PID, API revision, random
//!    bearer token) atomically with owner-only permissions — the descriptor
//!    only ever points at a live, already-listening daemon;
//! 6. **then** walk the backend probes, concurrently, journaling
//!    `backend.probed` per backend as each completes.
//!
//! Clean shutdown journals `daemon.stopped` and removes the descriptor; a
//! crash leaves a stale descriptor, which clients detect (dead PID + refused
//! endpoint) and replace.
//!
//! ## Publish, then probe (#293)
//!
//! Step 6 used to be step 4b-ii — the whole probe walk, serially, *before*
//! the bind and the descriptor. The argument for that ordering was that a
//! daemon should not be reachable before it knows what it can route to. The
//! measurement retires it: on a fully-provisioned host (all five adapters
//! installed) the descriptor appeared **~6.4s after exec across five cold
//! starts** on Cerberus, 2026-08-25, every millisecond of it serial probing —
//! agy ~2.3s, opencode ~3.2s over four invocations, codex ~0.4s, claude
//! ~0.2s — against a client auto-spawn budget of 10s (`SPAWN_WAIT`,
//! `src/cli.rs`). The ~3.6s of headroom left meant any concurrent load,
//! including a test suite's own parallel daemon spawns, pushed the first
//! `sgt run` past the budget; #293's A/B evidence showed that one defect
//! explaining every real-spawn failure across the m2/m3/m6/m8/m9 suites at
//! once. A daemon is healthy when it can accept and route requests, not when
//! every third-party CLI has printed `--help`.
//!
//! **After, measured the same way on the same host** (five cold starts each,
//! fresh estate and data dir per start, polled at 10ms from `exec`; Cerberus,
//! 2026-08-25): the descriptor lands at **0.06-0.10s**, and the whole probe
//! walk is durable — the sixth `backend.probed` flushed — at **3.22-3.27s**.
//! The same instrument re-run against the pre-#293 build measures **6.03-6.32s
//! for both**, which is what "both" meant back then: the descriptor could not
//! precede the walk it was waiting on. So the client-visible number falls by
//! ~85x and stops being the walk's hostage, while the walk itself roughly
//! halves — its floor is now its slowest single adapter (opencode ~3.2s)
//! rather than the sum of all six, exactly as `probe_walk`'s own doc predicts.
//!
//! **The capability-pending contract**, which is what makes the reordering
//! safe rather than merely faster. Between serving and a backend's probe
//! completing:
//!
//! - a caller whose preflight needs capability evidence **waits** for it —
//!   [`crate::backend::ProbeGate`], waited on by the router's tier walk at the
//!   one point where a backend name becomes decisive. The wait is per backend
//!   and bounded by that backend's own probe completing, so a Work bound for a
//!   fast adapter is never held up by a slow one it will never touch;
//! - no capability is ever fabricated, defaulted, or inferred to fill the
//!   window, and no submission is refused that the old ordering would have
//!   accepted. The only difference observable to a client is *when* the
//!   answer arrives;
//! - the journal's evidence order per backend is unchanged: `backend.probed`
//!   is durable before any Work routed to that backend is.
//!
//! What did *not* move is everything ahead of the descriptor in the list
//! above. Restart reconciliation and the L6 pause clear stay before it,
//! because those are claims about state a request could act on; a probe is a
//! claim about an adapter, and the request that needs it can wait for it.

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
use crate::backend::agy::{AGY_BACKEND_NAME, AgyBackend, AgyConfig};
use crate::backend::child::{self, ProbeChildren};
use crate::backend::claude::{CLAUDE_BACKEND_NAME, ClaudeBackend, ClaudeConfig};
use crate::backend::codex::{CODEX_BACKEND_NAME, CodexBackend, CodexConfig};
use crate::backend::docker::{DOCKER_BACKEND_NAME, DockerBackend, DockerConfig};
use crate::backend::fake::FAKE_BACKEND_NAME;
use crate::backend::opencode::{OPENCODE_BACKEND_NAME, OpencodeBackend, OpencodeConfig};
use crate::backend::{BackendRegistry, EventSink, ProbeGate};
use crate::domain::event::{EventDraft, EventSource};
use crate::platform::fs_locking::{self, Reliability};
use crate::runtime::atlas::db::{Analytics, AnalyticsError};
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
///
/// **v3 (H1, sprint-plan D3):** no estate fields. v2 published the one
/// estate root the process was bound to, because that binding *was* the
/// client-side safety check; a host daemon has no such binding, and the
/// admitted-estate set is dynamic state served live by `GET /v1/estates`
/// rather than something a file written once, atomically, at startup could
/// honestly describe. A v2 descriptor therefore fails closed through
/// [`DaemonError::UnknownDescriptorSchema`] like any other unknown schema —
/// no shim, deliberately: "the daemon you are talking to is from another
/// build" is exactly the fact a client must not paper over, and here it also
/// means "that daemon believes it owns one estate."
pub const DESCRIPTOR_SCHEMA: &str = "sergeant.runtime/v3";

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

/// The runtime descriptor published for clients (proposal §6, H1 §3):
/// endpoint, PID, API revision, and the bearer token, protected by
/// owner-only file permissions. Five fields, and — since
/// [`DESCRIPTOR_SCHEMA`]'s v3 bump — **nothing about estates** (D3).
///
/// **What the retired v2 fields did, and what replaced them.** v2 carried
/// `estate_root`/`manifest_path`, and §5.1's client gate compared the
/// caller's exact root against them: "a live daemon bound to another estate
/// is a named refusal, never a reusable global service." H1 makes one daemon
/// serving many estates the normal case, so that comparison has no object.
/// The property it protected — nothing is ever served for an estate nobody
/// validated — is now kept where the validation actually happens: the
/// admitted-estate registry ([`crate::runtime::estates::EstateRegistry`]),
/// re-checked per request, with
/// [`crate::runtime::estates::check_estate_root`] as its client-side half.
/// The set is dynamic, so it is answered live by `GET /v1/estates` rather
/// than frozen into a file written once at startup.
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
    ///
    /// D8: one token spans every admitted estate — the ratified single-user
    /// trust model. The widened blast radius of a leaked token is stated,
    /// not silently inherited.
    pub token: String,
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
    /// W3 Q9: this daemon's predecessor declared a prune it did not
    /// acknowledge, and completing it (from the intent's own recorded
    /// targets and residue) failed. Refused rather than served over: an
    /// unfinished destructive act on disk means answering reads from a
    /// journal whose floor is in an undeclared state.
    #[error(transparent)]
    Prune(#[from] crate::runtime::prune::PruneError),
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
    /// **Every superseded schema is exactly this case, in both directions.**
    /// A `sergeant.runtime/v1` descriptor carried no estate binding at all;
    /// a `sergeant.runtime/v2` one carried a binding to exactly one estate,
    /// which a host daemon does not have (D3). Neither can be read half-way
    /// — the v2 case is the sharper of the two, because its fields still
    /// *parse*, and acting on them would mean addressing a daemon that
    /// believes it owns one estate as though it served many. There is
    /// deliberately no compatibility shim in either direction; the remedy is
    /// H1 §6's cutover, and this refusal is its backstop: stop the old
    /// daemon and let a restarted one republish in the schema this build
    /// reads.
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

/// How long shutdown waits for the backend probe walk to finish before
/// journaling `daemon.stopped` and going.
///
/// The walk is joined at all — rather than dropped — for the same reason the
/// completion driver is: it commits events, and `daemon.stopped` should be
/// the last one this daemon writes. It gets the same grace for the same
/// reason too, and makes the same trade past it, stated rather than hidden: a
/// probe still inside a child process when the grace runs out may journal its
/// `backend.probed` after `daemon.stopped`. That is tolerable exactly where
/// the driver's version is — the journal is append-only and crash-tolerant
/// per append, `daemon.stopped` is a lifecycle record nothing reconciles
/// against, and `backend.probed` is on `prune::NON_WORK_ALLOWLIST` as a kind
/// with *no registry effect at all*, so a replay cannot tell the two orders
/// apart.
///
/// Why it is bounded at all, measured (#293, Cerberus 2026-08-25): a probe is
/// a real CLI invocation, and under `m6`'s own parallel load the walk ran past
/// ten seconds — long enough that a rig SIGTERMing a fresh daemon escalated to
/// SIGKILL, which flushes nothing at exit. Waiting that out unbounded would
/// put "how loaded is this box" on the shutdown path, and a daemon that will
/// not die is a lock nobody can take.
const PROBE_WALK_SHUTDOWN_GRACE: Duration = DRIVER_SHUTDOWN_GRACE;

/// How many backend probes the startup walk runs at once.
///
/// **Bounded rather than "all of them", and the number has provenance.**
/// Measured on Cerberus, 2026-08-25, with all five adapters installed: one
/// cold daemon alone completes its walk in ~2.5s, and **twenty cold daemons
/// started simultaneously take 13.8-14.7s each** — 20 × 6 unbounded lanes is
/// 120 concurrent third-party CLI startups on a 20-core box, and every one of
/// them gets slower. That is not a hypothetical: it is `m6`'s own suite at
/// default parallelism, where a daemon SIGTERMed the instant its descriptor
/// appeared could not finish its walk inside the rig's ten-second grace and
/// was escalated to SIGKILL, flushing nothing at exit.
///
/// Two lanes, not more, because the walk's floor is its slowest single
/// adapter either way: opencode alone is +3.24s of a ~2.5s-to-3.3s walk, and
/// the four fast adapters together are +0.74s, so two lanes finish in the
/// same time one unbounded fan-out does while forking a third as many
/// children. Raise it only against a measurement showing the *fast* adapters'
/// sum has overtaken the slowest one's time — that is the condition under
/// which a lane cap starts costing something, and nothing else is.
const PROBE_WALK_LANES: usize = 2;

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
    /// The probe children *this* daemon's walk has live (#310). Per-daemon,
    /// never global: `cargo test` runs many in-process daemons at once, and a
    /// global set would let one daemon's `kill` take a sibling's live probe
    /// child down and turn its probe into a spurious refusal.
    probe_children: Arc<ProbeChildren>,
}

impl DaemonHandle {
    /// Whether the in-process task serving this daemon is still running.
    ///
    /// This is [`DaemonHandle`]'s answer to the question
    /// [`crate::api::ApiClient::send_with_retry`] asks of a `pid`: is the
    /// thing on the other end of a failed request provably still there?
    /// There is no `pid` to check here — this daemon is a `tokio` task in
    /// the *same* OS process as the caller, not a separate one — so
    /// liveness reads off the serving task's own [`tokio::task::JoinHandle`]
    /// instead: alive for as long as that task has not finished, dead once
    /// it has (whether by orderly [`Self::shutdown`], an abrupt
    /// [`Self::kill`], or an internal panic). Never a duration: this reads
    /// the task's own completion state, not how long a caller has been
    /// waiting for one.
    pub fn is_alive(&self) -> bool {
        !self.served.is_finished()
    }

    /// Signal graceful shutdown and wait for the daemon to finish cleanup.
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        let _ = (&mut self.served).await;
    }

    /// Abruptly terminate the daemon without giving it a chance to run its
    /// own cooperative shutdown path — the in-process rig's analogue of
    /// SIGKILL. m6's out-of-process rig signals a real child process; this
    /// in-process one has no separate process to signal, so it aborts the
    /// serving task instead. The shutdown channel is dropped unsent —
    /// nothing here asks the daemon to stop, it is simply cut off
    /// mid-flight. Awaiting the aborted task still lets the runtime finish
    /// unwinding it (releasing the exclusive `daemon.lock`, closing the
    /// listener) before a fresh daemon can bind the same data dir.
    pub async fn kill(mut self) {
        self.shutdown_tx.take();
        self.served.abort();
        let _ = (&mut self.served).await;
        // #310: aborting the serve task does not reach the probe walk, and a
        // probe child is a *process* — a real `opencode serve` at ~265 MB —
        // that nothing in this process's memory owns once the handle is
        // gone. Reaped by recorded pgid, so the kill reaches whatever the
        // probe child itself spawned and nothing that belongs to anyone
        // else. This is the in-process analogue of the `PR_SET_PDEATHSIG`
        // that covers the out-of-process daemon; both exist because
        // destructors do not run for a killed process.
        let killed = self.probe_children.kill_all();
        if !killed.is_empty() {
            tracing::debug!(
                ?killed,
                "killed probe children still live when the daemon was killed"
            );
        }
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
    /// Launch configuration for the Codex adapter this daemon registers
    /// itself. `None` is the system `codex`, adapter state under this data
    /// dir — mirrors `claude`/`docker` above for the same reason.
    pub codex: Option<CodexConfig>,
    /// Launch configuration for the Opencode adapter this daemon registers
    /// itself. `None` is the system `opencode`, adapter state under this data
    /// dir — mirrors `claude`/`docker`/`codex` above for the same reason.
    pub opencode: Option<OpencodeConfig>,
    /// Launch configuration for the Agy adapter this daemon registers
    /// itself. `None` is the system `agy`, adapter state under this data
    /// dir — mirrors `claude`/`docker`/`codex`/`opencode` above for the same
    /// reason.
    pub agy: Option<AgyConfig>,
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
    /// W2fix (#293): whether [`start_with`] returns only after the backend
    /// probe walk has finished.
    ///
    /// The descriptor is published before the walk either way — that is the
    /// fix, and it is not negotiable by this flag. What the flag decides is
    /// what the *caller* of `start_with` is waiting for, and the two callers
    /// genuinely want different things:
    ///
    /// - `true`, the default, for an in-process embedder. Every rig in
    ///   `tests/` holds a [`DaemonHandle`] and reads it as "this daemon has
    ///   started"; one that snapshots the journal on the next line must not
    ///   race a startup append.
    /// - `false`, set by [`run_until_signal`], for the daemon binary. Its
    ///   caller is a signal loop, and a process that cannot answer SIGTERM
    ///   until its slowest adapter has finished printing `--help` is exactly
    ///   the "daemon that will not die is a lock nobody can take" this file
    ///   already refuses elsewhere. Measured on Cerberus 2026-08-25: with the
    ///   wait in place, a SIGTERM arriving the instant the descriptor
    ///   appeared took 3.27s to be answered on an idle host, and over ten
    ///   seconds under `m6`'s own parallel load — long enough that the rig
    ///   escalated to SIGKILL and the daemon flushed nothing at exit.
    pub await_probe_walk: bool,
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
    /// W3: retention cap for this daemon's journal, overriding `[estate]
    /// retention`. `None` — production, always — takes
    /// [`crate::domain::estate::DEFAULT_RETENTION`].
    ///
    /// **H1:** a host daemon has no one estate whose manifest could supply
    /// this, so the `Manifest` rung is gone from the *process-wide* policy.
    /// Each admitted estate's own `[estate] retention` is read at admission
    /// ([`crate::runtime::estates::AdmittedEstate::retention`]); partitioning
    /// the retention *decision* by estate is W4a's deliverable (D7 keeps the
    /// blob-reference scan journal-wide either way). Until then the
    /// process-wide policy is the explicit override or the built-in default,
    /// which is a widening — never a silent narrowing — of what any one
    /// estate declared.
    ///
    /// Configurable for the same reason `segment_max_bytes`/`completion_poll`
    /// are: a prune test has to stand on the far side of the cap, and
    /// building `DEFAULT_RETENTION` real Works is far outside a test budget.
    /// **[`crate::domain::estate::MIN_RETENTION`] deliberately does not apply
    /// here** — it is a *manifest-schema* refusal protecting an operator
    /// from a typo in a file, not a runtime bound; a test rig setting
    /// `Some(4)` has not typed anything into a manifest.
    pub retention: Option<u32>,
    /// H1-15 (W4b) execution lane: daemon-wide cap on native adapter
    /// processes admitted between PREPARE and LAUNCH concurrently
    /// (`SGT_EXECUTION_LANE_CAP`). `None` keeps [`Engine::new`]'s default
    /// ([`crate::runtime::engine::default_execution_lane_cap`]). `Some` is
    /// wired via [`Engine::with_execution_lane_cap`] in [`start_with`], the
    /// same way `turn_cap` above is.
    pub execution_lane_cap: Option<usize>,
    /// H1-15's second lane: config-only capacity for A1/S3's future
    /// intelligence workers (`SGT_INTELLIGENCE_LANE_CAP`). `None` keeps
    /// [`crate::runtime::engine::default_intelligence_lane_cap`]. Nothing
    /// in this build acquires from it yet (deliverable 3 — config surface,
    /// no scheduling behavior); it exists so the two lanes' independence is
    /// provable now rather than retrofitted later.
    pub intelligence_lane_cap: Option<usize>,
    /// W5 brief deliverable 1(a): how often the periodic multi-estate sweep
    /// caller ([`crate::api::maybe_run_periodic_sweep`]) re-walks every
    /// admitted estate's mounts. Configurable for the same reason
    /// `completion_poll` is: a test that needs a tick to always be due holds
    /// this at zero rather than racing the production default.
    pub sweep_interval: Duration,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            backends: Arc::new(BackendRegistry::default_registry()),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            docker: None,
            codex: None,
            opencode: None,
            agy: None,
            telemetry: None,
            completion_poll: COMPLETION_POLL_INTERVAL,
            turn_ceiling: crate::runtime::engine::DEFAULT_TURN_CEILING,
            surfaces_root: None,
            turn_cap: None,
            rebuild_cache: false,
            await_probe_walk: true,
            segment_max_bytes: None,
            retention: None,
            execution_lane_cap: None,
            intelligence_lane_cap: None,
            sweep_interval: crate::api::SWEEP_INTERVAL,
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

/// S5 W1b: retry one Work's overlay eviction as an idempotent startup
/// catch-up, logging the outcome. Shared by both halves of the sweep — the
/// terminal-cache pass right after Atlas opens, and the
/// `surfaces_retired`-keyed pass right after `recovery::reconcile` finishes
/// a crash-interrupted teardown — so there is exactly one place that decides
/// what "swept" and "failed to sweep" mean.
fn sweep_one_overlay_eviction(atlas: &mut crate::runtime::atlas::db::AtlasDb, work_id: &str) {
    match atlas.evict_work_overlays(work_id) {
        Ok(evicted) if !evicted.is_empty() => tracing::info!(
            target: "sergeant::atlas",
            work_id,
            generations = evicted.len(),
            "startup swept an overlay eviction a prior process's crash left unfinished"
        ),
        Ok(_) => {}
        Err(e) => tracing::warn!(
            target: "sergeant::atlas",
            work_id,
            error = %e,
            "could not sweep this Work's overlay eviction at startup; it stays queryable \
             until a later restart retries"
        ),
    }
}

/// Start the daemon on `data_dir`. Returns once startup is complete: serving,
/// runtime descriptor published, and the backend probe walk finished.
///
/// Those three do not happen at the same moment any more (#293). The
/// descriptor is published — and the daemon starts accepting — *before* the
/// probe walk runs; this call returns after the walk, because an in-process
/// handle is read as "started" and a rig that takes a journal snapshot right
/// after it must not race a startup append. A client waiting on the
/// descriptor file is unblocked at the earlier moment, which is the whole
/// point of the reordering; see this module's `Publish, then probe` doc.
pub async fn start_with(
    data_dir: &Path,
    config: DaemonConfig,
) -> Result<DaemonHandle, DaemonError> {
    // 0a. **Estate admission is no longer a startup step.** It used to be the
    // first thing this function did — `Estate::admit(config.estate_root)`,
    // before the data dir was even created, refusing the whole start when it
    // failed. H1 moves it out: a host daemon starts bound to **zero** estates
    // (the normal state, not a rig edge case) and admits each one lazily on
    // the first request that addresses it
    // ([`crate::runtime::estates::EstateRegistry`]). Keeping the old step
    // would mean one estate's broken `sergeant.toml` refusing to start the
    // daemon every *other* estate's Work depends on — which is precisely why
    // admission failure is now an estate-specific refusal to the caller that
    // asked for it, and never process death.
    let estates = Arc::new(crate::runtime::estates::EstateRegistry::new());

    create_dir_all_durable(data_dir)?;

    // 0. #85 / ADR 0003 D6: refuse outright on a filesystem where advisory
    // locking is unreliable — before the lock below is even attempted, since
    // a lock that silently does not hold fails in a way nobody notices until
    // two daemons are racing the same data dir. An inconclusive probe (e.g.
    // today's always-Unknown macOS arm) does not refuse — see
    // `refuse_if_unreliable`.
    //
    // H1: `data_dir` is now the **host runtime root**, so this is the host
    // half of a check that has become two. The estate half runs per estate
    // root at admission (`estates::admit_root`), because an estate on a
    // network share and a host root on local disk is a real shape, and one
    // probe at one path can no longer answer for both.
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
    //
    // Since S5 W1c this opens Atlas's database file under `atlas/` and drops its `ops`
    // schema; it no longer deletes a file, because four other schemas in that
    // file must survive (A1 §5, F1). One consequence is visible three
    // paragraphs down: this call *creates* the Atlas database if it is
    // absent, so by the time the reconciliation below runs, the file always
    // exists.
    let mut analytics = Analytics::begin_rebuild(data_dir)?;
    let mut capability_sink = startup::CapabilitySink::seeded(plan.capability_seed());
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

    // 2c (W3 §2.5): the process-lifetime FirstSeqIndex the prune horizon
    // needs — seeded from the cache's own rows (once a v2 cache has really
    // written `first_seq` honestly) union this pass's own `HorizonSink`
    // fold, exactly the same "already <= H_old <= H, always a legal answer"
    // reasoning `LedgerSink`'s cache seed already follows.
    let mut first_seq_by_work: crate::runtime::prune::FirstSeqIndex = match &plan {
        startup::Plan::Windowed { cache, .. } => cache
            .works
            .iter()
            .map(|row| (row.id.clone(), row.first_seq))
            .collect(),
        startup::Plan::Full { .. } => Default::default(),
    };
    for (id, seq) in horizon_sink.first_seq_by_work() {
        first_seq_by_work.entry(id.clone()).or_insert(*seq);
    }

    // 2d. §2.6: the cache this start should leave behind — hit or miss,
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
        &first_seq_by_work,
    )?;
    startup::persist_or_remove(floor_state.as_ref(), data_dir)?;

    // 2e (S3 X2, F1): Atlas's startup reconciliation.
    //
    // The two rebuild disciplines meet here, a few lines apart, over the same
    // file, and the difference between them is the whole of F1.
    // `Analytics::begin_rebuild` above dropped the `ops` *schema* and refolded
    // it from the journal, because the operations tables are a pure fold of
    // it. Everything else in that file is opened and **kept**:
    // its `source.*` and `meta.coverage` rows are derived from source bytes
    // plus extractor identity, and no journal replay reproduces them.
    // `record::reconcile_sources` is what closes the crash window, and it
    // needs both halves of the evidence — which is why it takes the journal
    // as well as the store. A generation a crash left `provisional` is
    // promoted when its `source.scanned` summary is in the journal (the scan
    // completed; only the confirming transaction was lost) and evicted, with
    // an explicit `generation_evicted` coverage row, when it is not. The
    // database's `state` column alone cannot tell those two apart, and they
    // have opposite correct answers.
    //
    // Opened and dropped rather than held: nothing in this build reads Atlas
    // while the daemon runs (`sgt intelligence status` and the `map` surface
    // land with their own wave), so keeping a second connection open for the
    // process lifetime would buy nothing. What must happen at startup is the
    // reconciliation, and that is what this does.
    //
    // The existence check is kept, and is now a cheap guard rather than the
    // cost-avoidance it was. It used to be load-bearing: opening creates the
    // file, and creating it here would have meant every host that never
    // declared a knowledge source paying a database creation on every start
    // for a feature it never used (R1). `Analytics::begin_rebuild` above now
    // creates that same file unconditionally, because `ops` lives in it — so
    // the cost is paid either way and the branch is no longer avoiding it.
    // What it still buys is honesty about the one case that could ever reach
    // it: a file that does not exist has, by definition, no crash-window
    // generation to reconcile, and a reconciliation that had to create its
    // own store to find nothing in it would be reporting on state it made up.
    if crate::runtime::atlas::db::atlas_db_path(data_dir).exists() {
        match analytics.atlas() {
            Ok(mut atlas) => {
                match crate::runtime::atlas::record::reconcile_sources(&mut atlas, &journal) {
                    Ok(resolved) => tracing::debug!(
                        target: "sergeant::atlas",
                        path = %atlas.path().display(),
                        promoted = resolved.promoted.len(),
                        evicted = resolved.evicted.len(),
                        "atlas opened and reconciled at startup"
                    ),
                    // Same reasoning as an unopenable store, one line down:
                    // derived evidence never costs the estate its daemon.
                    Err(e) => tracing::warn!(
                        target: "sergeant::atlas",
                        error = %e,
                        "atlas could not be reconciled at startup; a crash-window \
                         generation may remain unresolved (it stays unreadable)"
                    ),
                }
                // S5 W2 (F-SF-01): the lexical-index upgrade path. A store
                // written before W2 has confirmed generations with rows but
                // no postings — `reindex_lexical`'s own doc names this exact
                // condition, but nothing invoked it. `lexical_index_needs_rebuild`
                // is a cheap anti-join, so it is safe to check every startup
                // rather than only once at a version boundary this crate has
                // no other way to detect; the rebuild itself runs only when
                // that check finds something to do.
                match atlas.lexical_index_needs_rebuild() {
                    Ok(true) => match atlas.reindex_lexical() {
                        Ok(outcome) => tracing::debug!(
                            target: "sergeant::atlas",
                            indexed = outcome.indexed,
                            truncated = outcome.truncated,
                            "lexical index rebuilt at startup (S5 W2 upgrade path)"
                        ),
                        // Same reasoning as an unopenable store above:
                        // derived evidence never costs the estate its daemon.
                        Err(e) => tracing::warn!(
                            target: "sergeant::atlas",
                            error = %e,
                            "lexical index could not be rebuilt at startup; \
                             lexical_search may return empty for pre-existing \
                             generations this run"
                        ),
                    },
                    Ok(false) => {}
                    Err(e) => tracing::warn!(
                        target: "sergeant::atlas",
                        error = %e,
                        "could not check whether the lexical index needs a rebuild"
                    ),
                }
                // S5 W1b: the Work-overlay eviction reconciliation sweep,
                // first half — a terminal Work whose surface was ALREADY
                // torn down before this restart.
                //
                // Ordinary eviction runs as a detached task off the crank
                // (`api::run_work_overlay_hook`'s `Evict` arm), racing a
                // Work's teardown against nothing — a daemon killed between
                // `surface.torn_down` landing and that task finishing loses
                // the eviction permanently under that mechanism alone. This
                // closes it the same way `record::reconcile_sources` above
                // closes the analogous scan-side gap: one pass, right here,
                // before anything is served. See
                // `recovery::terminal_work_ids_with_a_torn_down_surface`'s
                // own doc for exactly which Works this selects (and why the
                // crash-before-teardown case is deliberately NOT this list —
                // it is closed below instead, once `recovery::reconcile` has
                // actually finished that teardown).
                //
                // `evict_work_overlays` is naturally idempotent (it only
                // ever touches generations not already `evicted`), so
                // retrying it for the overwhelming majority already cleanly
                // evicted is a safe no-op; only the rare one a crash caught
                // mid-flight changes. One-time pass at this restart, not a
                // periodic loop (out of this wave's scope).
                for work_id in crate::runtime::recovery::terminal_work_ids_with_a_torn_down_surface(
                    registry.state(),
                ) {
                    sweep_one_overlay_eviction(&mut atlas, &work_id);
                }
            }
            // Never fatal. Atlas is derived evidence; a daemon that refused
            // to start because a derived store was unreadable would trade
            // every Work in the estate for an index (A1-01: the journal, Git
            // and the original bytes are authority — Atlas is not).
            Err(e) => tracing::warn!(
                target: "sergeant::atlas",
                error = %e,
                "atlas could not be opened at startup; source intelligence is unavailable this run"
            ),
        }
    }

    let rebuild_ms = u64::try_from(rebuild_started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let startup_cache = plan.startup_cache_tag();
    // The oldest surviving seq (1 on every W2 production path — the floor
    // only moves above 1 once W3's prune lands).
    let replay_floor_seq = journal.floor_seq()?.unwrap_or(1);

    let (events_tx, _) = broadcast::channel(1024);
    let mut core = Core::new(journal, registry, events_tx)
        .with_floor_ledger(Arc::new(floor_ledger))
        .with_first_seq_index(first_seq_by_work);

    // W3 §1.2, as H1 leaves it: resolve the *process-wide* retention policy
    // once — `config.retention` (test rigs only) -> the built-in default.
    // Journaled on every prune event so the record names the authorization,
    // not just the number (A1).
    //
    // The `[estate] retention` rung is gone from **this** resolution, and
    // that is a deliberate, named change rather than an omission. It used to
    // re-read the one bound estate's manifest here; a host journal holds
    // many estates' Works, so "the manifest" has no referent at startup.
    // Each admitted estate's own declaration is read at admission
    // (`estates::admit_root` -> `AdmittedEstate::retention`) and W4a owns
    // partitioning the retention *decision* by estate — D7 keeps the
    // blob-reference scan journal-wide regardless, precisely so a per-estate
    // decision can never condemn another estate's live blobs. Until W4a
    // lands, the process-wide number is the explicit override or the
    // default: a widening of what any one estate declared, never a silent
    // narrowing.
    let prune_policy = match config.retention {
        Some(retention) => crate::runtime::prune::PrunePolicy {
            retention,
            source: crate::runtime::prune::PolicySource::Config,
        },
        None => crate::runtime::prune::PrunePolicy {
            retention: crate::domain::estate::DEFAULT_RETENTION,
            source: crate::runtime::prune::PolicySource::Default,
        },
    };
    // W5 brief deliverable 1(b): the startup trigger now partitions by
    // estate exactly like the rotation tick (`maybe_run_rotation_triggered_
    // prune`) does — built from the same, still-empty-at-this-instant
    // `estates` registry (admission is lazy; see step 0a above), so a
    // restart with no request yet answered falls fully back to
    // `prune_policy` above, and widens correctly once estates re-admit.
    let startup_prune_policies =
        crate::runtime::prune::EstatePolicies::from_registry(&estates, prune_policy);

    // 2e-2g (W3 §10.3, §6.6): Q9's crash completion — evidence-based, exactly
    // as `recovery::reconcile_terminal_surface` is: the intent is a durable,
    // fsynced record that a specific, enumerated deletion was authorized and
    // begun. Nothing here is inferred, nothing is widened, and no deletion
    // is ever started from suspicion — `recovery.rs`'s own refusal boundary
    // ("recovery acts on evidence that something was left unfinished, and
    // there is none here") is untouched: this acts only where an intent
    // exists, and only on the targets that intent names. Runs *before*
    // `recovery::reconcile` below: the completion's fold removes the pruned
    // Works from `works`/`runs`, and every pruned Work is retired whole
    // (I-W3-3) so reconcile would have found nothing to do for them either
    // way — the ordering is for determinism, not correctness.
    let prune_started = Instant::now();
    let had_interrupted_prune = core.registry.state().pending_prune.is_some();
    if had_interrupted_prune {
        crate::runtime::prune::complete_interrupted(&mut core, data_dir)?;
    }
    let first_seq_snapshot = core.first_seq_by_work.clone();
    let prune_outcome = match crate::runtime::prune::run_startup(
        &mut core,
        data_dir,
        &startup_prune_policies,
        &first_seq_snapshot,
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            tracing::error!(error = %e, "startup prune failed; serving anyway");
            core.prune_pending = true;
            crate::runtime::prune::PruneOutcome::default()
        }
    };
    let prune_duration_ms = u64::try_from(prune_started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let prune_outcome_tag = if had_interrupted_prune {
        "completed-interrupted".to_string()
    } else if prune_outcome.segments_unlinked > 0 {
        format!("pruned:{}", prune_outcome.segments_unlinked)
    } else {
        let stalled = core.journal.segment_bounds().ok().map(|bounds| {
            crate::runtime::prune::candidate_horizon_multi_estate(
                &bounds,
                core.registry.state(),
                &core.first_seq_by_work,
                &startup_prune_policies,
            )
            .1
        });
        match stalled.and_then(|s| s.blocking_work_id) {
            Some(work_id) => format!("stalled:{work_id}"),
            None => "none".to_string(),
        }
    };

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
            // W3: additive keys (§10.3) — what the startup prune trigger did
            // (or why it did nothing), so a slow start is explicable.
            "prune_duration_ms": prune_duration_ms,
            "prune_outcome": prune_outcome_tag,
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

    // 4b. Register the real adapters alongside whatever the config supplied
    // (tests hand in scripted stubs/fakes; they keep them). Added here, not
    // in `default_registry`, because each needs this data dir (raw-
    // transcript/blob archive) and an event sink that only exists once the
    // core does. A config that already registered the name wins — that is
    // how tests substitute stubs.
    //
    // Codex is registered the same way as Claude and Docker as of the
    // 2026-08-21 codex-adapter sprint (W2, closing the registration half of
    // deviation D6 — `knowledge/rulings/deviations/d6-codex-descoped.md`):
    // W1 landed `CodexBackend` measured against codex-cli 0.149.0 (H0
    // evidence packet). An unavailable build (binary missing, unmeasured
    // grammar) still registers and reports its own probe evidence at
    // `backend.probed` — the same posture Docker already takes for a host
    // with no Docker installed (§17.5's "degraded daemon, strict work
    // admission"): routing *to* it is what fails, at submission time, not
    // daemon startup.
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
    // W2: the Codex executor, registered the same way and for the same
    // reason as Claude/Docker above. No `seed_capability_provenance_from`
    // call here — that method exists only on `ClaudeBackend`, seeding a
    // journaled `ask`-withdrawal claim Codex's `Capabilities::ask` does not
    // have (Codex's `ask` is `false` unconditionally, per W1's module doc).
    let codex = if config.backends.get(CODEX_BACKEND_NAME).is_none() {
        let codex_config = config
            .codex
            .clone()
            .unwrap_or_else(|| CodexConfig::new(data_dir));
        let adapter = Arc::new(CodexBackend::new(codex_config));
        backends = backends.with(adapter.clone());
        Some(adapter)
    } else {
        None
    };
    // W2 (opencode-adapter sprint, mirrors the codex block directly above):
    // the Opencode executor, registered the same way and for the same reason
    // as Claude/Docker/Codex. No `seed_capability_provenance_from` call here
    // either — that method exists only on `ClaudeBackend`, and
    // `OpencodeBackend::capabilities().ask` is `false` unconditionally
    // (opencode.rs's own ADMISSION_ROWS `ask` row: no measured
    // actor-authored-question record on this transport), so there is no
    // journaled withdrawal to seed.
    let opencode = if config.backends.get(OPENCODE_BACKEND_NAME).is_none() {
        let opencode_config = config
            .opencode
            .clone()
            .unwrap_or_else(|| OpencodeConfig::new(data_dir));
        let adapter = Arc::new(OpencodeBackend::new(opencode_config));
        backends = backends.with(adapter.clone());
        Some(adapter)
    } else {
        None
    };
    // W2 (agy-adapter sprint, mirrors the opencode block directly above):
    // the Agy executor, registered the same way and for the same reason
    // as Claude/Docker/Codex/Opencode. No `seed_capability_provenance_from`
    // call — `AgyBackend::capabilities().ask` is `false` unconditionally
    // (agy.rs's own ADMISSION_ROWS `ask` row: no measured actor-authored-
    // question record on this transport), so there is no journaled
    // withdrawal to seed.
    let agy = if config.backends.get(AGY_BACKEND_NAME).is_none() {
        let agy_config = config
            .agy
            .clone()
            .unwrap_or_else(|| AgyConfig::new(data_dir));
        let adapter = Arc::new(AgyBackend::new(agy_config));
        backends = backends.with(adapter.clone());
        Some(adapter)
    } else {
        None
    };
    // 4b-ii. The capability/version probe is *scheduled* here and runs at
    // step 6, after the descriptor is published — see this module's
    // startup-order doc and [`ProbeGate`] for why (#293). What happens here is
    // only that the registry gains the readiness gate its routing path will
    // wait on; not one subprocess is forked, and the gate is still **unarmed**,
    // which is load-bearing: restart reconciliation two steps below reads
    // capabilities, and a gate armed this early would make it wait for a walk
    // that has not started.
    let probe_gate = Arc::new(ProbeGate::new());
    let backends = Arc::new(backends.with_probe_gate(probe_gate.clone()));
    let probe_backends = backends.clone();

    // 4c. Reconcile work believed in flight *before* serving (§25): no
    // request may observe — or act on — a work whose prior ownership has not
    // yet been settled. `reconcile` itself flushes after each work it
    // touches, so its own effects (a `git worktree remove`, a relaunched
    // harness) are never left unsynced while unbounded — see its doc.
    let mut engine = Engine::new(backends, config.default_backend.clone(), data_dir)
        .with_turn_ceiling(config.turn_ceiling);
    // D10: no `with_estate_root` here any more. The engine holds no estate;
    // each `plan` call is handed the one its request addressed, after the
    // registry admitted it.
    if let Some(surfaces_root) = config.surfaces_root.clone() {
        engine = engine.with_surfaces_root(surfaces_root);
    }
    if let Some(turn_cap) = config.turn_cap {
        engine = engine.with_turn_cap(turn_cap);
    }
    if let Some(cap) = config.execution_lane_cap {
        engine = engine.with_execution_lane_cap(cap);
    }
    if let Some(cap) = config.intelligence_lane_cap {
        engine = engine.with_intelligence_lane_cap(cap);
    }
    // C1 §3: install the compilation step, so a stage launch actually
    // compiles a world instead of the capability shipping green and
    // unreachable.
    //
    // Derived from `analytics`, never `AtlasDb::open` — one file is one
    // DuckDB instance, and a second `open` is a second instance whose writes
    // and the projection's silently overwrite each other (`Analytics::atlas`,
    // and this module's own Atlas startup-reconciliation doc). This is the
    // same `Connection::try_clone` handle `ApiState::atlas` is derived from,
    // taken once and held for the process because a compilation happens on
    // every stage entry.
    //
    // A host that cannot produce a handle installs no compiler and therefore
    // keeps §18's first rung: the existing stage launch path, unchanged
    // (§21 item 13). That is reported, not silent.
    if crate::runtime::atlas::db::atlas_db_path(data_dir).exists() {
        match analytics.atlas() {
            Ok(atlas) => {
                engine = engine.with_context_compiler(Arc::new(
                    crate::runtime::context::AtlasContextCompiler::new(atlas),
                ));
            }
            Err(e) => tracing::warn!(
                target: "sergeant::atlas",
                error = %e,
                "atlas could not be opened for C1 context compilation; stages launch on the \
                 existing context path with no compiled snapshot"
            ),
        }
    }
    let engine = Arc::new(engine);
    let reconciled = recovery::reconcile(&engine, &estates, &mut core)?;
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

    // S5 W1b: the Work-overlay eviction sweep's second half — a terminal
    // Work whose surface teardown itself never finished before the crash
    // (`recovery::reconcile_terminal_surface` bypassed the ordinary
    // lifecycle hook entirely, because it runs the crash-recovery path
    // directly rather than through the crank arm that hook is wired to).
    // `reconciled.surfaces_retired` names exactly the Works whose teardown
    // `reconcile` just finished above — this is that teardown's own overlay
    // eviction, run once it is actually true that a torn-down surface
    // stands to evict.
    //
    // Derived again rather than held from the block above: that block runs
    // before `reconcile`, and an Atlas handle is deliberately taken and
    // dropped rather than kept for the process lifetime (see this module's
    // Atlas startup-reconciliation doc). Derived from `analytics`, never
    // `AtlasDb::open` — one file is one DuckDB instance, and a second
    // `open` would be a second instance whose writes and the projection's
    // silently overwrite each other (`Analytics::atlas`).
    if !reconciled.surfaces_retired.is_empty()
        && crate::runtime::atlas::db::atlas_db_path(data_dir).exists()
    {
        match analytics.atlas() {
            Ok(mut atlas) => {
                for work_id in &reconciled.surfaces_retired {
                    sweep_one_overlay_eviction(&mut atlas, work_id);
                }
            }
            Err(e) => tracing::warn!(
                target: "sergeant::atlas",
                error = %e,
                "atlas could not be opened to sweep a crash-recovered teardown's overlay \
                 eviction; it stays queryable until a later restart retries"
            ),
        }
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
        estates,
        analytics: Arc::new(tokio::sync::Mutex::new(analytics)),
        // S3 X4: opened lazily by the first `map`/`intelligence` read that
        // finds a file to open — see `ApiState::atlas`.
        atlas: Arc::new(tokio::sync::Mutex::new(None)),
        prune_policy,
        sweep_interval: config.sweep_interval,
        last_swept: Arc::new(std::sync::Mutex::new(None)),
        // S6 scan front door: empty at start — an accepted scan is tracked
        // here, the journal keeps the durable record.
        scans: Default::default(),
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
    // The Codex adapter's normalized events flow through the identical sink
    // (same reasoning as Claude's/Docker's, directly above).
    if let Some(codex) = codex {
        codex.set_event_sink(journaling_sink(state.core.clone()));
    }
    // The Opencode adapter's normalized events flow through the identical
    // sink (same reasoning as Claude's/Docker's/Codex's, directly above).
    if let Some(opencode) = opencode {
        opencode.set_event_sink(journaling_sink(state.core.clone()));
    }
    // The Agy adapter's normalized events flow through the identical sink
    // (same reasoning as Claude's/Docker's/Codex's/Opencode's, directly
    // above).
    if let Some(agy) = agy {
        agy.set_event_sink(journaling_sink(state.core.clone()));
    }

    // 6. The capability/version probe walk (#293), started the moment the
    // daemon is reachable and run concurrently across backends. Spawned after
    // the adapter sinks are wired above so a probe can never race an adapter
    // that has no sink yet, and joined at shutdown below so `daemon.stopped`
    // stays the last event this daemon writes.
    //
    // Arming the gate is the line before the spawn, and both are ahead of the
    // serve task: the listener is bound, so a client that read the descriptor
    // a moment ago has its connection sitting in the accept backlog, and the
    // first request axum takes off it is therefore handled by a routing path
    // that already knows what evidence is outstanding.
    probe_gate.expect(probe_backends.names());
    let probe_lanes = Arc::new(tokio::sync::Semaphore::new(PROBE_WALK_LANES));
    // #310: the set every probe child this walk spawns records itself into,
    // and the set `DaemonHandle::kill` reaps. Created here, one per daemon.
    let probe_children = ProbeChildren::new();
    let probes = tokio::spawn(probe_walk(
        state.core.clone(),
        probe_backends,
        probe_gate.clone(),
        probe_lanes.clone(),
        probe_children.clone(),
    ));

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
        //
        // The probe walk (#293) is the other non-request writer and the same
        // rule binds it, so it is joined here too — **concurrently, not after**.
        // Two graces run one after the other add up, and their sum was
        // measured to be the whole budget a rig gives a SIGTERMed daemon: with
        // the joins sequential, `m6`'s
        // `the_dropped_spawned_daemon_leaves_the_evidence_of_a_clean_shutdown`
        // still escalated to SIGKILL under its own parallel load (Cerberus,
        // 2026-08-25). Neither wait needs the other's answer, so neither
        // should wait for it.
        let driver = async {
            match tokio::time::timeout(DRIVER_SHUTDOWN_GRACE, completions).await {
                Ok(Err(e)) => tracing::warn!(error = %e, "the completion driver panicked"),
                Err(_) => tracing::warn!(
                    grace = ?DRIVER_SHUTDOWN_GRACE,
                    "the completion driver was still inside an external effect at shutdown; \
                     stopping anyway"
                ),
                Ok(Ok(())) => {}
            }
        };
        // A stopping daemon starts no probe it has not already started: the
        // in-flight ones are uninterruptible child processes it must wait out,
        // but the queued ones are free to drop, and `PROBE_WALK_LANES` is what
        // bounds how many can be in the first category.
        probe_lanes.close();
        let walk = async {
            match tokio::time::timeout(PROBE_WALK_SHUTDOWN_GRACE, probes).await {
                Ok(Err(e)) => tracing::warn!(error = %e, "the backend probe walk panicked"),
                Err(_) => tracing::warn!(
                    grace = ?PROBE_WALK_SHUTDOWN_GRACE,
                    "the backend probe walk was still running at shutdown; stopping anyway"
                ),
                Ok(Ok(())) => {}
            }
        };
        tokio::join!(driver, walk);
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

    // Whether the handle is handed back mid-walk or after it is the one
    // question `await_probe_walk` answers — see its doc for why the two
    // callers want different things. What a *client* waits on is settled
    // either way and much earlier: the descriptor was published above, before
    // this walk was even armed.
    //
    // Waiting on the gate rather than on the `probes` join handle is what
    // lets the serve task keep that handle and join it at shutdown. The gate
    // is a `Condvar`, so the wait goes on the blocking pool, where blocking
    // waits belong.
    if config.await_probe_walk {
        let settled = probe_gate.clone();
        if let Err(e) = tokio::task::spawn_blocking(move || settled.wait_all()).await {
            tracing::warn!(error = %e, "waiting for the backend probe walk failed");
        }
    }

    Ok(DaemonHandle {
        endpoint,
        token,
        shutdown_tx: Some(shutdown_tx),
        served,
        probe_children,
    })
}

/// The capability/version probe walk, recorded at registration (M4 contract):
/// one `backend.probed` event per registered backend, carrying what the probe
/// found — a version and flag set that only ever appear inside a later
/// refusal's message are not a record. An unavailable backend is probed and
/// journaled anyway and refuses work with this same evidence; routing must be
/// able to say *why*, not pretend the backend does not exist.
///
/// **Concurrent, and on the blocking pool, both deliberately (#293).** A
/// probe is one or more real CLI invocations — measured per backend on
/// Cerberus 2026-08-25, from `daemon.started` to each `backend.probed`:
/// fake +0.03s, docker +0.05s, claude +0.27s, codex +0.39s, agy +2.40s,
/// opencode +3.24s — and they are independent of one another, so running them
/// one at a time bought nothing but their sum. They are also *blocking*
/// (`std::process::Command::output`), which the old serial walk ran straight
/// on the async runtime; `spawn_blocking` is where blocking work belongs, and
/// it is what makes the concurrency real rather than five futures taking
/// turns on one worker thread.
///
/// **Within-adapter invocation parallelism: measured, then declined (R1).**
/// The obvious next step is to fan out the invocations *inside* the two slow
/// adapters, and the same measurement says not to. Opencode's three probe
/// invocations are genuinely independent, but timed individually on this host
/// they are 0.354s / 0.359s / 0.362s — about 1.1s of its 3.24s. The rest is
/// `serve_gates`' probe child (`opencode serve --port 0` plus the
/// authenticated `/doc` fetch), and agy's 2.40s is almost entirely
/// `read_config_probe`'s `agy -p /config` child, neither of which is a
/// `--help` that fanning out helps. Parallelising opencode's helps would move
/// it to roughly agy's number and the walk's completion barely at all — while
/// costing a real behaviour change, since `run_probe` short-circuits today
/// and a fan-out would fork two more doomed children on every host with no
/// opencode installed (which is every CI runner). Revisit if a future
/// adapter's cost is actually in its help invocations; the walk is off the
/// startup critical path either way, and the per-backend gate means a Work
/// waits only on the adapter it routes to.
///
/// Each result is journaled and flushed as it lands, then marked on the gate
/// — in that order, so a caller released by the gate is released over durable
/// evidence, never over an append still in a group.
async fn probe_walk(
    core: Arc<tokio::sync::Mutex<Core>>,
    backends: Arc<BackendRegistry>,
    gate: Arc<ProbeGate>,
    lanes: Arc<tokio::sync::Semaphore>,
    probe_children: Arc<ProbeChildren>,
) {
    let mut walking = tokio::task::JoinSet::new();
    for name in backends.names() {
        let backends = backends.clone();
        let lanes = lanes.clone();
        let probe_children = probe_children.clone();
        walking.spawn(async move {
            // The lane permit is held across the blocking probe and released
            // when it returns — see `PROBE_WALK_LANES` for why the fan-out is
            // bounded at all. A closed semaphore means shutdown began before
            // this probe ever started one: skip it rather than fork a child
            // the stopping daemon would then have to wait out.
            let Ok(_lane) = lanes.acquire_owned().await else {
                return Ok((name, None));
            };
            tokio::task::spawn_blocking(move || {
                let Some(backend) = backends.get(&name) else {
                    return (name, None);
                };
                // #310: everything a probe spawns is recorded against this
                // walk for as long as this closure runs, which is exactly the
                // window in which a probe child is live. The owner is
                // installed around `capabilities()` too, because for the
                // opencode and agy adapters that call *is* a subprocess gate.
                let (report, capabilities) = child::owned_by(probe_children, || {
                    let report = backend.probe();
                    (report, backend.capabilities())
                });
                // `capabilities()` is read here, inside the blocking task,
                // and not on the runtime: for the opencode and agy adapters
                // it resolves the transport, which is itself a subprocess
                // gate, and for Claude it reads a claim the probe may have
                // withdrawn.
                let payload = json!({
                    "backend": name,
                    "available": report.available,
                    "detail": report.detail,
                    "capabilities": capabilities,
                    // §17: the adapter's declared runtime scope, recorded
                    // with the probe because it is a claim about the adapter,
                    // and the core is forbidden from assuming one.
                    "runtime_scope": backend.runtime_scope(),
                });
                (name, Some(payload))
            })
            .await
        });
    }
    while let Some(joined) = walking.join_next().await {
        let (name, payload) = match joined.and_then(|probed| probed) {
            Ok(probed) => probed,
            Err(e) => {
                // A panicked probe takes its own name with it, so nothing
                // can be marked here; `settle` below is what keeps that from
                // stranding a waiter.
                tracing::error!(error = %e, "a backend probe panicked");
                continue;
            }
        };
        let Some(payload) = payload else {
            // Either shutdown closed the lanes before this probe started, or
            // the backend left the registry between scheduling and running
            // (which nothing does today). Both leave a name owed a record it
            // will never get, and neither may wedge a submission waiting on
            // it — so the gate is released without one.
            tracing::debug!(backend = %name, "backend probe skipped; no evidence recorded");
            gate.mark(&name);
            continue;
        };
        {
            let mut core = CoreGuard::acquire(&core).await;
            let committed = core
                .commit(EventDraft::new(
                    EventSource::new("daemon", "sergeant"),
                    KIND_BACKEND_PROBED,
                    payload,
                ))
                .and_then(|_| core.flush());
            if let Err(e) = committed {
                tracing::error!(error = %e, backend = %name, "failed to journal backend.probed");
            }
        }
        gate.mark(&name);
    }
    // Unconditional: see `ProbeGate::settle`.
    gate.settle();
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
/// `data_dir` is the **host runtime root** (D2): journal, projections, blob
/// store, `daemon.lock` and the descriptor all live beneath it, and it is
/// resolved from `--data-dir` / `SGT_DATA_DIR` / the platform convention —
/// never from an estate. There is deliberately no estate parameter any more
/// (deliverable 8): `-C` on a `daemon` verb names the estate the invocation
/// is *addressing*, not one this process binds itself to for life.
///
/// This is the one place §28 export is configured from the environment, and
/// [`TelemetryConfig::from_env`] answers "off" unless explicitly switched on.
pub async fn run_until_signal(data_dir: &Path, rebuild_cache: bool) -> Result<(), DaemonError> {
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
    // H1-15's execution/intelligence lane overrides: same fail-open
    // discipline as `SGT_TURN_CAP` above — unset or unparseable both mean
    // "keep the built-in, host-parallelism-derived default", and 0 (which
    // parses) is honored with a warning rather than silently ignored.
    let execution_lane_cap = std::env::var("SGT_EXECUTION_LANE_CAP")
        .ok()
        .and_then(|v| match v.parse::<usize>() {
            Ok(0) => {
                tracing::warn!(
                    "SGT_EXECUTION_LANE_CAP=0 blocks every Work at LAUNCH; honoring it"
                );
                Some(0)
            }
            Ok(n) => Some(n),
            Err(_) => {
                tracing::warn!(
                    value = %v,
                    "SGT_EXECUTION_LANE_CAP is not a whole number of permits — keeping the built-in default"
                );
                None
            }
        });
    let intelligence_lane_cap = std::env::var("SGT_INTELLIGENCE_LANE_CAP")
        .ok()
        .and_then(|v| match v.parse::<usize>() {
            Ok(n) => Some(n),
            Err(_) => {
                tracing::warn!(
                    value = %v,
                    "SGT_INTELLIGENCE_LANE_CAP is not a whole number of permits — keeping the built-in default"
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
            execution_lane_cap,
            intelligence_lane_cap,
            rebuild_cache,
            // The signal loop below must be able to answer SIGTERM while the
            // probe walk is still running — see the field's own doc (#293).
            await_probe_walk: false,
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

    /// `DaemonHandle::is_alive` (wave `transport-timeout-is-not-a-verdict`,
    /// 2026-09-02): the in-process liveness carrier `tests/`'s
    /// `scan_to_completion` needs before it may retry a transport error
    /// (`send_with_retry`'s own doc comment, `src/api.rs::~7754`, names the
    /// shape — alive ⇒ retry, dead ⇒ fail naming it). `DaemonHandle` has no
    /// `pid` (it is a `tokio` task in the *same* OS process, unlike
    /// `send_with_retry`'s out-of-process client), so liveness reads off the
    /// serving task's own `JoinHandle` instead: alive while that task has not
    /// finished, dead once it has. Constructed directly here (`mod tests`
    /// shares `DaemonHandle`'s private fields via `use super::*`) rather than
    /// through `start_with`, so this is a fast, deterministic unit test of
    /// the accessor alone — no real daemon, no network, no sleep.
    #[tokio::test]
    async fn is_alive_reads_the_serving_task_join_handle_not_a_pid() {
        // A task that never returns: `is_alive()` must read `true` for as
        // long as it has not been polled to completion.
        let live = DaemonHandle {
            endpoint: "http://127.0.0.1:1".to_string(),
            token: "t".to_string(),
            shutdown_tx: None,
            served: tokio::spawn(std::future::pending::<()>()),
            probe_children: ProbeChildren::new(),
        };
        assert!(
            live.is_alive(),
            "a serving task that never completes must read as alive"
        );
        live.served.abort();

        // A task that has already finished: `is_alive()` must read `false`.
        // The loop below observes the join handle's own `is_finished()`
        // state rather than assuming a fixed duration is enough time for the
        // scheduler to have polled the spawned no-op task to completion —
        // exactly the "wait on state, never on time" shape this wave's own
        // governing ruling requires of `tests/`, applied to this new
        // accessor's own test.
        let dead = DaemonHandle {
            endpoint: "http://127.0.0.1:1".to_string(),
            token: "t".to_string(),
            shutdown_tx: None,
            served: tokio::spawn(async {}),
            probe_children: ProbeChildren::new(),
        };
        let mut observed_finished = false;
        for _ in 0..200 {
            if dead.served.is_finished() {
                observed_finished = true;
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(
            observed_finished,
            "the spawned no-op task never reached is_finished() within 200 yields"
        );
        assert!(
            !dead.is_alive(),
            "a serving task that has already finished must read as not alive"
        );
    }

    /// **Re-scoped, not deleted** (brief deliverable 4): this test used to be
    /// `a_descriptor_is_usable_only_from_the_estate_it_is_bound_to`, proving
    /// the strict-equality binding gate in both directions. That refusal
    /// class is retired with H1, so the negative it protected is re-pointed
    /// rather than dropped: a descriptor no longer *says* anything about
    /// estates, and what refuses an unvalidated estate is the admission
    /// check that replaced it.
    #[test]
    fn a_v3_descriptor_says_nothing_about_estates_and_admission_is_what_refuses() {
        let descriptor = RuntimeDescriptor {
            schema: DESCRIPTOR_SCHEMA.to_string(),
            endpoint: "http://127.0.0.1:1".to_string(),
            pid: 1,
            api_revision: API_REVISION.to_string(),
            token: "t".to_string(),
        };
        let serialized = serde_json::to_value(&descriptor).expect("descriptor json");
        let mut keys: Vec<&str> = serialized
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec!["api_revision", "endpoint", "pid", "schema", "token"],
            "D3: a v3 descriptor carries no estate binding to compare against"
        );

        // What refuses instead: the admission check, per estate, on its own
        // evidence rather than on what some file said at startup.
        let err = crate::runtime::estates::check_estate_root(
            Some(Path::new("/estates/definitely-not-one")),
            "sgt run",
        )
        .expect_err("an unvalidated root must still be refused");
        assert_eq!(err.code(), "invalid_estate", "got {err}");
    }

    /// The v1→v2 bump has no compatibility shim: a descriptor written by an
    /// older build fails closed as an unknown schema, and the diagnostic
    /// names the stop/restart remedy rather than leaving an operator staring
    /// at a schema string.
    ///
    /// **v2 joins v1 here at the D3 bump** — and it is the case that actually
    /// happens at cutover, since a v2 daemon is what every developer has
    /// running the moment before they install a host-mode build. Its fields
    /// still parse, which is precisely why it must be refused rather than
    /// read: they describe a process that believes it owns exactly one
    /// estate.
    #[test]
    fn a_superseded_descriptor_fails_closed_with_a_restart_remedy() {
        for superseded in ["sergeant.runtime/v1", "sergeant.runtime/v2"] {
            let dir = tempfile::TempDir::new().expect("tempdir");
            std::fs::write(
                dir.path().join(DESCRIPTOR_FILE),
                serde_json::json!({
                    "schema": superseded,
                    "endpoint": "http://127.0.0.1:1",
                    "pid": 1,
                    "api_revision": API_REVISION,
                    "token": "t",
                    "estate_root": "/estates/payments",
                })
                .to_string(),
            )
            .expect("write superseded descriptor");

            let err =
                read_descriptor(dir.path()).expect_err("a superseded schema must fail closed");
            let message = err.to_string();
            assert!(
                matches!(err, DaemonError::UnknownDescriptorSchema { .. }),
                "got {message}"
            );
            assert!(
                message.contains(superseded) && message.contains(DESCRIPTOR_SCHEMA),
                "both schemas must be named: {message}"
            );
            assert!(
                message.contains("sgt daemon stop"),
                "the remedy must tell the operator to restart the daemon: {message}"
            );
        }
    }

    /// Complements `a_superseded_descriptor_fails_closed_with_a_restart_remedy`:
    /// the positive case for the *current* schema, going through the same
    /// on-disk file-write → `read_descriptor` path.
    ///
    /// It also pins the half of D3 a struct definition cannot: a v3
    /// descriptor that someone has *added* estate fields to still reads back
    /// as the five fields this build knows, so no code path can start
    /// depending on an out-of-band estate binding smuggled through the file.
    #[test]
    fn a_v3_descriptor_round_trips_through_read_descriptor_ignoring_estate_fields() {
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
        .expect("write v3 descriptor");

        let descriptor = read_descriptor(dir.path())
            .expect("read must succeed")
            .expect("descriptor must be present");
        assert_eq!(descriptor.schema, DESCRIPTOR_SCHEMA);
        assert_eq!(descriptor.endpoint, "http://127.0.0.1:1");
        assert_eq!(descriptor.pid, 1);
        assert_eq!(descriptor.api_revision, API_REVISION);
        assert_eq!(descriptor.token, "t");
        let round_tripped = serde_json::to_value(&descriptor).expect("descriptor json");
        assert!(
            round_tripped.get("estate_root").is_none()
                && round_tripped.get("manifest_path").is_none(),
            "D3: nothing in this build carries an estate on the descriptor, got {round_tripped}"
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

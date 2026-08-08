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

use std::fs::{OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{broadcast, oneshot, watch};

use crate::api::{API_REVISION, ApiState, Core, CoreError, router};
use crate::domain::event::{EventDraft, EventSource};
use crate::runtime::fsutil::{create_dir_all_durable, write_atomic_secret};
use crate::runtime::journal::{Journal, JournalError};
use crate::runtime::projection::{ProjectionError, work_registry_projection};

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

/// Start the daemon on `data_dir`. Returns once it is serving and the
/// runtime descriptor is published.
pub async fn start(data_dir: &Path) -> Result<DaemonHandle, DaemonError> {
    create_dir_all_durable(data_dir)?;

    // 1. Exclusive daemon lock: a second daemon on the same data dir fails
    // closed here, before touching journal or descriptor. The OS releases
    // the advisory lock on process death, so it can never go stale.
    let lock = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(data_dir.join(DAEMON_LOCK_FILE))?;
    match lock.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return Err(DaemonError::Locked),
        Err(TryLockError::Error(e)) => return Err(e.into()),
    }

    // 2. Own the journal and rebuild current state by full replay (§24;
    // snapshots are an optimization the M2 daemon does not need yet).
    let journal = Journal::open(data_dir)?;
    let mut registry = work_registry_projection();
    registry.catch_up(journal.replay()?)?;

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
    };
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

/// Run the daemon in the foreground until SIGINT/SIGTERM, then shut down
/// cleanly. This is what `sgt daemon` (and therefore auto-spawn) executes.
pub async fn run_until_signal(data_dir: &Path) -> Result<(), DaemonError> {
    let handle = start(data_dir).await?;
    tracing::info!(endpoint = %handle.endpoint, data_dir = %data_dir.display(), "daemon serving");
    wait_for_shutdown_signal().await;
    tracing::info!("shutdown signal received");
    handle.shutdown().await;
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

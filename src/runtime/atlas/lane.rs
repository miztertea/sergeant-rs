//! The thin F6 glue: extraction on the intelligence lane (X3a).
//!
//! H1-15 shipped two capacity lanes and one consumer. The execution lane
//! bounds per-Work native processes; the intelligence lane was declared,
//! bounded, and — in that sprint's own words — acquired by nothing, because
//! the work it exists for had not been built. This file is where that stops
//! being true: Atlas's source extraction is the intelligence lane's first
//! real consumer, and F6 is the decision it implements.
//!
//! ```text
//! Engine::run_intelligence   permit, then the blocking pool
//!    -> scan_estate_git_with_worker       one pinned commit's objects, extracted (S4 Y8 dispatch)
//!    -> scan_work_overlay                 one Work surface, over its base
//!    -> scan_local_knowledge_with_worker  one declared knowledge source, walked (S4 Y5 G8; Y8)
//!    -> acquire_and_scan                  one external Git source, fetched then extracted (S4 Y5, G6)
//! ```
//!
//! `_with_worker` (S4 Y8): [`worker_runtime`] resolves this host's own
//! binary path once per call and threads it down as a real
//! [`super::worker::WorkerRuntime`], so a claimed Office/ZIP/mail resource
//! actually reaches [`super::worker::run_worker`] from these two production
//! entry points — the plain, worker-free [`super::scan::scan_local_knowledge`]/
//! [`super::git::scan_estate_git`] stay every other caller's default.
//! `scan_work_overlay`/`acquire_and_scan` are untouched: worker dispatch is
//! scoped to these two walks this wave (brief-y8-adapter-dispatch.md).
//!
//! # Why these two functions exist rather than a call site inlining them
//!
//! F6's adapter-shape mandate is that extraction is a pure function over
//! bytes with "DB-touching glue kept thin and separately reviewable". The
//! same argument applies to *daemon-state*-touching glue, which is what
//! acquiring a lane permit is. [`super::git`] and [`super::overlay`] know
//! nothing about an [`Engine`]; this file knows nothing about a database. Two
//! small files at the bottom of the module graph, each joining exactly one
//! thing to the pure middle, is the shape that keeps either half reviewable
//! on its own.
//!
//! # What is bounded, and what is not
//!
//! The lane bounds *concurrency*, not duration or size. A scan that would read
//! an enormous repository is not made small by waiting for a permit — the
//! per-resource ceiling and the batch budgets in [`super::git`] are what bound
//! that, and they are separate deliberately. The lane's one promise is the one
//! F6 asks for: however much extraction the daemon is asked to do, it never
//! spends the execution lane's budget doing it.

use crate::runtime::atlas::deny::{AcquisitionFilter, BadPattern};
use crate::runtime::atlas::external_git::{
    ExternalGitError, ExternalGitScan, ExternalGitSource, acquire_and_scan,
};
use crate::runtime::atlas::git::{
    EstateGitScan, EstateGitSource, GitScanError, scan_estate_git_with_worker,
};
use crate::runtime::atlas::overlay::{OverlayScan, WorkOverlay, scan_work_overlay};
use crate::runtime::atlas::scan::{KnowledgeSource, SourceScan, scan_local_knowledge_with_worker};
use crate::runtime::atlas::worker::{
    WORKER_RUNTIME_DEADLINE, WorkerIdentity, WorkerOutcome, WorkerRuntime, WorkerSpawn, run_worker,
};
use crate::runtime::engine::{Engine, IntelligenceError};

/// Why an extraction on the lane did not produce an answer.
#[derive(Debug, thiserror::Error)]
pub enum LaneError {
    /// The lane itself refused, or the job did not complete.
    #[error(transparent)]
    Lane(#[from] IntelligenceError),
    /// The extraction ran and failed.
    #[error(transparent)]
    Scan(#[from] GitScanError),
    /// A declared `[[knowledge]] ignore` glob does not compile.
    #[error(transparent)]
    Pattern(#[from] BadPattern),
    /// This host's own binary path could not be resolved (S4 Y8) —
    /// [`WorkerRuntime::program`] needs it before a single resource can be
    /// dispatched, and [`std::env::current_exe`] is the one call that can
    /// fail to produce it (an unlinked/deleted binary, a permission
    /// problem reading `/proc/self/exe` on Linux).
    #[error("cannot resolve this host's own binary path for the supervised worker: {0}")]
    WorkerProgram(#[from] std::io::Error),
}

/// Why an external-git acquisition on the lane did not produce an answer —
/// a separate enum from [`LaneError`] rather than one more variant on it,
/// because [`ExternalGitError`] already carries every failure shape this
/// call can produce (including its own [`BadPattern`]/[`GitScanError`]
/// cases) and re-wrapping each one individually here would just be a second
/// name for the same thing.
#[derive(Debug, thiserror::Error)]
pub enum ExternalGitLaneError {
    /// The lane itself refused, or the job did not complete.
    #[error(transparent)]
    Lane(#[from] IntelligenceError),
    /// Acquisition (locator, fetch, or extraction) failed.
    #[error(transparent)]
    Acquire(#[from] ExternalGitError),
}

/// Scan one estate-git source under an intelligence-lane permit, on the
/// blocking pool (F6).
pub async fn scan_estate_git_on_lane(
    engine: &Engine,
    source: EstateGitSource,
) -> Result<EstateGitScan, LaneError> {
    let worker = worker_runtime()?;
    Ok(engine
        .run_intelligence(move || scan_estate_git_with_worker(&source, &worker))
        .await??)
}

/// This host's own binary — the same one running right now — as the
/// supervised worker every Office/ZIP/mail resource this scan claims
/// dispatches to (S4 Y8), with [`WORKER_RUNTIME_DEADLINE`]'s provisional
/// bound.
///
/// [`std::env::current_exe`] is a plain, cheap syscall (`/proc/self/exe` on
/// Linux, `_NSGetExecutablePath`/`sysctl` on macOS) — no daemon state, no
/// database, nothing this lane-glue file doesn't already own the right to
/// call — resolved here rather than inside [`super::scan`]/[`super::git`]'s
/// own pure walks, which by design (their own module docs) know nothing
/// about a running process's own path.
fn worker_runtime() -> Result<WorkerRuntime, LaneError> {
    Ok(WorkerRuntime {
        program: std::env::current_exe()?,
        deadline: WORKER_RUNTIME_DEADLINE,
    })
}

/// Scan one Work surface as an overlay under an intelligence-lane permit, on
/// the blocking pool (F6).
pub async fn scan_work_overlay_on_lane(
    engine: &Engine,
    overlay: WorkOverlay,
) -> Result<OverlayScan, LaneError> {
    Ok(engine
        .run_intelligence(move || scan_work_overlay(&overlay))
        .await??)
}

/// Run one supervised parse worker under an intelligence-lane permit, on the
/// blocking pool (F6, S4 Y1's G2) — never the execution lane, exactly as
/// [`scan_estate_git_on_lane`] and [`scan_work_overlay_on_lane`] already
/// hold for the two extraction kinds that came before it. The permit is
/// acquired before the worker is even spawned and is held for the whole of
/// [`super::worker::run_worker`] — supervision included, so a worker that
/// has to be killed past its deadline still frees the permit only when that
/// kill and reap are done, never early.
pub async fn run_worker_on_lane(
    engine: &Engine,
    spawn: WorkerSpawn,
    identity: WorkerIdentity,
    deny: AcquisitionFilter,
) -> Result<WorkerOutcome, IntelligenceError> {
    engine
        .run_intelligence(move || run_worker(spawn, &identity, &deny))
        .await
}

/// Scan one declared local-knowledge source under an intelligence-lane
/// permit, on the blocking pool (F6, S4 Y5's G8 scan trigger) — the walk
/// [`super::scan::scan_local_knowledge`] already did as pure Rust; this is
/// the missing lane wrapper that lets `sgt knowledge scan` drive it the
/// identical way `scan_estate_git_on_lane` already drives an estate-git
/// walk. Its absence until now is exactly the gap
/// `tests/x5_a1a_acceptance.rs`'s cross-cutting-gap tripwire named: a walk
/// that exists but nothing calls off the execution lane.
pub async fn scan_local_knowledge_on_lane(
    engine: &Engine,
    source: KnowledgeSource,
) -> Result<SourceScan, LaneError> {
    let worker = worker_runtime()?;
    Ok(engine
        .run_intelligence(move || scan_local_knowledge_with_worker(&source, &worker))
        .await??)
}

/// Acquire and scan one external Git source under an intelligence-lane
/// permit, on the blocking pool (F6, S4 Y5's G6) — the whole of
/// [`super::external_git::acquire_and_scan`], including its own supervised
/// `git fetch` ([`crate::runtime::git::git_fetch_restricted`]'s #310
/// discipline), runs inside the one blocking closure the permit bounds, so
/// the permit is held for the fetch's own bounded lifetime exactly as
/// [`run_worker_on_lane`] holds one for a parse worker's.
pub async fn acquire_external_git_on_lane(
    engine: &Engine,
    source: ExternalGitSource,
) -> Result<ExternalGitScan, ExternalGitLaneError> {
    Ok(engine
        .run_intelligence(move || acquire_and_scan(&source))
        .await??)
}

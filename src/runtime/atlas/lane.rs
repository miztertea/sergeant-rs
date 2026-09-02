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
//! `_with_worker` (S4 Y8): [`worker_runtime`] resolves the real
//! `sgt-atlas-worker` binary once per call — **not** this host's own
//! `current_exe()` (S4 Y8 panel fix (c): the daemon's own binary has no
//! subcommand-free flag surface that would accept `--generation`/
//! `--extractor`, so re-execing it as the worker produced a clap parse
//! error on every real installation, not an extraction) — and threads it
//! down as a real [`super::worker::WorkerRuntime`], so a claimed
//! Office/ZIP/mail resource actually reaches [`super::worker::run_worker`]
//! from these two production entry points — the plain, worker-free
//! [`super::scan::scan_local_knowledge`]/[`super::git::scan_estate_git`]
//! stay every other caller's default. `scan_work_overlay`/
//! `acquire_and_scan` are untouched: worker dispatch is scoped to these two
//! walks this wave (brief-y8-adapter-dispatch.md).
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

use std::path::PathBuf;

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
    /// The `sgt-atlas-worker` binary's path could not be resolved (S4 Y8)
    /// — [`WorkerRuntime::program`] needs it before a single resource can
    /// be dispatched, and resolving it needs [`std::env::current_exe`] to
    /// find this host's own binary directory first, which is the one call
    /// that can fail to produce it (an unlinked/deleted binary, a
    /// permission problem reading `/proc/self/exe` on Linux) or return a
    /// path with no parent directory at all.
    #[error("cannot resolve the sgt-atlas-worker binary path for the supervised worker: {0}")]
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

/// The real `sgt-atlas-worker` binary — installed beside this host's own
/// running binary, the same one running right now — as the supervised
/// worker every Office/ZIP/mail resource this scan claims dispatches to
/// (S4 Y8), with [`WORKER_RUNTIME_DEADLINE`]'s provisional bound.
///
/// **S4 Y8 panel fix (c).** The originally landed form of this function
/// returned [`std::env::current_exe`] itself, unchanged — the *daemon's*
/// own binary path, not the worker's. `sgt`'s own CLI is a `#[command
/// (subcommand)]` parser with no bare-flag surface: spawning it with
/// `--generation <id> --extractor <name>` (the args
/// [`super::scan::dispatch_worker_resource`] actually sends) fails to
/// parse before a single byte of `input` is ever read, so every real,
/// non-test installation turned every claimed `.docx`/`.zip`/`.eml` into a
/// clap error, never an extraction — reachable from nowhere real despite
/// `tests/y8_adapter_dispatch.rs` passing, because that suite drives
/// [`super::scan::scan_local_knowledge_with_worker`] directly with a
/// hand-built [`WorkerRuntime`] pointed at the real
/// `CARGO_BIN_EXE_sgt-atlas-worker`, never this function.
///
/// The fix: Cargo's own build layout (and the shipped install layout —
/// this file's own module doc's binary-doc precedent) puts every `[[bin]]`
/// target in the SAME directory, so the worker binary's path is this
/// host's own binary's directory, joined with its own name. Still one
/// [`std::env::current_exe`] call — a plain, cheap syscall (`/proc/self/exe`
/// on Linux, `_NSGetExecutablePath`/`sysctl` on macOS), no daemon state, no
/// database, nothing this lane-glue file doesn't already own the right to
/// call — resolved here rather than inside [`super::scan`]/[`super::git`]'s
/// own pure walks, which by design (their own module docs) know nothing
/// about a running process's own path.
fn worker_runtime() -> Result<WorkerRuntime, LaneError> {
    Ok(WorkerRuntime {
        program: worker_binary_path(&std::env::current_exe()?)?,
        deadline: WORKER_RUNTIME_DEADLINE,
    })
}

/// The pure half of [`worker_runtime`]'s resolution — `sgt-atlas-worker`
/// (platform executable suffix included, R2: [`std::env::consts::EXE_SUFFIX`]
/// is empty on Unix and `.exe` on Windows, the same primitive
/// [`std::env::current_exe`]'s own callers elsewhere in this crate would
/// reach for) in the SAME directory as `exe`. Split out from
/// [`worker_runtime`] so the path arithmetic is testable against a
/// synthetic `exe` without needing a real running process at a controlled
/// location.
fn worker_binary_path(exe: &std::path::Path) -> Result<PathBuf, LaneError> {
    let dir = exe.parent().ok_or_else(|| {
        LaneError::WorkerProgram(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "{} (this host's own binary path) has no parent directory",
                exe.display()
            ),
        ))
    })?;
    Ok(dir.join(format!("sgt-atlas-worker{}", std::env::consts::EXE_SUFFIX)))
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

/// Run one supervised parse worker under an intelligence-lane permit of its
/// own, on the blocking pool (F6, S4 Y1's G2) — never the execution lane.
/// The permit is acquired before the worker is even spawned and is held for
/// the whole of [`super::worker::run_worker`] — supervision included, so a
/// worker that has to be killed past its deadline still frees the permit
/// only when that kill and reap are done, never early.
///
/// **Test-only as of S4 Y8, and pinned that way below
/// ([`tests::run_worker_on_lane_has_no_production_caller_yet`]).** When a
/// real scan walk needed per-resource worker dispatch, S4 Y8 gave
/// [`scan_local_knowledge_on_lane`]/[`scan_estate_git_on_lane`] their own
/// whole-scan permit instead and had
/// [`super::scan::dispatch_worker_resource`] call
/// [`super::worker::run_worker`] directly, once per claimed resource,
/// already inside that one permit — not this function, which would try to
/// acquire a SECOND permit per resource from code that (by design, this
/// module's own doc, "Why these two functions exist") knows nothing about
/// an [`Engine`] to acquire one from. This function stays for the shape of
/// caller `tests/y1_worker_transport.rs` and its Y2-Y4 siblings already
/// use — one worker call, one permit, driven straight from a test — and
/// remains available should a future caller want per-resource rather than
/// whole-scan permitting; it is not currently reachable from any real scan.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// **S4 Y8 fix-agent panel finding.** [`worker_binary_path`] is the
    /// pure half of the resolution bug this wave's own fix corrects
    /// (`worker_runtime`'s own doc, "S4 Y8 panel fix (c)"): the wrong
    /// version returned `current_exe()` itself; the fix joins its own
    /// directory with the real worker's name. Exercised here directly,
    /// against a synthetic path, rather than only through
    /// `tests/y8_adapter_dispatch.rs`'s end-to-end proof (which needs a
    /// real `sgt-atlas-worker` planted beside the test binary's own
    /// `current_exe()` to run at all) — this is the arithmetic alone.
    #[test]
    fn worker_binary_path_resolves_the_sibling_sgt_atlas_worker_binary() {
        let exe = std::path::Path::new("/opt/sergeant/bin/sgt");
        let resolved = worker_binary_path(exe).expect("resolves");
        assert_eq!(
            resolved,
            PathBuf::from(format!(
                "/opt/sergeant/bin/sgt-atlas-worker{}",
                std::env::consts::EXE_SUFFIX
            )),
            "must resolve the SIBLING binary, never current_exe() unchanged"
        );
    }

    #[test]
    fn worker_binary_path_refuses_an_exe_with_no_parent_directory() {
        let exe = std::path::Path::new("/");
        let err = worker_binary_path(exe).expect_err("root has no parent");
        assert!(matches!(err, LaneError::WorkerProgram(_)));
    }

    /// **S4 Y8 fix-agent panel finding.** [`run_worker_on_lane`] is a real,
    /// exported async fn with a real doc contract — and, unlike
    /// [`scan_local_knowledge_on_lane`]/[`scan_estate_git_on_lane`] beside
    /// it, is called from nowhere under `src/`: only
    /// `tests/y1_worker_transport.rs` and its Y2-Y4 siblings call it, and
    /// [`run_worker_on_lane`]'s own doc comment above now says so
    /// explicitly rather than leaving it silent. This is the tripwire that
    /// keeps that statement true — the same shape the package-identity
    /// module's own `the_derivation_has_no_production_caller_yet` uses (R2):
    /// a recursive `src/` sweep, not a handful of hardcoded files, so a
    /// caller wired through any file cannot slip past silently.
    ///
    /// The needle is the CALL form `run_worker_on_lane(`, not the bare
    /// identifier: this very file's own doc comments above (and
    /// `super::office`'s, `super::worker`'s) already name
    /// `run_worker_on_lane` in prose explaining exactly this gap, and a
    /// bare-identifier needle would trip on its own explanation rather than
    /// on a real call.
    ///
    /// **If this test fails**, something under `src/` now calls
    /// `run_worker_on_lane(...)` — good news: correct this function's own
    /// doc comment (it currently states the opposite) and delete or repoint
    /// this tripwire rather than leaving it stale.
    #[test]
    fn run_worker_on_lane_has_no_production_caller_yet() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        // The owner itself is exempt (R2, same reasoning the
        // package-identity module's own tripwire gives for skipping its own
        // file): this very file both defines and (in its own docs) discusses
        // `run_worker_on_lane`.
        let owner = root.join("runtime/atlas/lane.rs");
        let files = rust_sources(&root);
        assert!(
            files.len() > 50,
            "the scan must actually cover the whole src/ tree, not a handful of files: {} found",
            files.len()
        );
        for path in &files {
            if *path == owner {
                continue;
            }
            let text = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            assert!(
                !text.contains("run_worker_on_lane("),
                "{} now calls run_worker_on_lane(...) — it appears to have gained a \
                 production caller. See this test's own doc comment for what to do.",
                path.strip_prefix(&root).unwrap_or(path).display()
            );
        }
    }

    /// Every `.rs` file under `dir`, recursively — the same shape the
    /// package-identity module's own `rust_sources` helper uses (R2),
    /// duplicated rather than shared because that one is a private helper
    /// in a different module.
    fn rust_sources(dir: &std::path::Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(dir).expect("read dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                out.extend(rust_sources(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
        out
    }
}

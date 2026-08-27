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
//!    -> scan_estate_git      one pinned commit's objects, extracted
//!    -> scan_work_overlay    one Work surface, over its base
//! ```
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

use crate::runtime::atlas::git::{EstateGitScan, EstateGitSource, GitScanError, scan_estate_git};
use crate::runtime::atlas::overlay::{OverlayScan, WorkOverlay, scan_work_overlay};
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
}

/// Scan one estate-git source under an intelligence-lane permit, on the
/// blocking pool (F6).
pub async fn scan_estate_git_on_lane(
    engine: &Engine,
    source: EstateGitSource,
) -> Result<EstateGitScan, LaneError> {
    Ok(engine
        .run_intelligence(move || scan_estate_git(&source))
        .await??)
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

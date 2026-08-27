//! The supervised parse-worker transport (S4 Y1, G2).
//!
//! ```text
//! WorkerSpawn + WorkerIdentity   bytes in, and what the daemon already
//!                                knows to be true about them
//!    -> run_worker               spawn, supervise, kill+reap, parse
//!       -> validate_batch        the AUTHORITY over what came back
//! ```
//!
//! # Transport, not a parser (F6's adapter-shape mandate, carried across a
//! process boundary)
//!
//! S3 already made every extractor a pure function over bytes. This module
//! does not touch that shape — it moves the *call* across a process
//! boundary, so a malformed input can take down a child instead of the
//! daemon. Y1 ships no third-party parser (Y2's Anydoc spike is that), so
//! [`WorkerBatch`] is deliberately thin: identity, structure units in the
//! same shape [`super::scan::ScannedUnit`] already uses, and declared child
//! resources — enough for the real adapters Y2 onward plug into the same
//! wire contract.
//!
//! # The daemon validates; the worker's own checks are defense in depth
//!
//! [`validate_batch`] is the AUTHORITY the sprint plan's G2 decision names,
//! not a formality: identity (generation + resource hash + extractor —
//! [`WorkerIdentity`]), path safety on every declared child
//! ([`enclosed_relative_path`], `enclosed_name` semantics reused from
//! [`crate::domain::is_plain_name`] per path component — R2, the same rule
//! [`crate::domain::is_plain_name`]'s own doc already applies to a composed,
//! `/`-joined path), and F10 deny-set membership matched on the child's
//! declared NAME as well as its path ([`deny::AcquisitionFilter`], reused
//! whole). A worker that returns a batch failing any of these is refused,
//! and the refusal is a NAMED [`CoverageRow`] — never a silent drop and
//! never a partial write, because nothing here writes anything: this module
//! hands the daemon a validated [`WorkerBatch`] or a reason it did not get
//! one, and [`super::record`]'s three-step discipline is what a later wave
//! wires the accepted half into.
//!
//! # Supervision (#310, reused whole — R2)
//!
//! [`run_worker`] spawns the worker in its own process group with
//! [`crate::backend::child::harden_probe_child`] — the exact mechanism
//! every backend probe already uses, because a worker call's own lifetime
//! is the same shape a probe's is: spawned and killed inside one function,
//! on one thread, bounded by a deadline the caller names
//! ([`crate::backend::child::ChildLifetime::Probe`]'s own doc explains why
//! that shape, and only that shape, may be hardened this way). A deadline
//! exceeded, a non-zero exit, and a signal termination (which is how a
//! `SIGABRT`'d worker is observed) are all **kill the group, then reap** —
//! never kill without reap, which would leave a zombie an orphan check
//! cannot distinguish from a live leak.
//!
//! # Memory containment (G2 amendment — the fault class the deadline alone
//! left open)
//!
//! The deadline above is a HANG guard: it bounds how long a child may run,
//! not how much address space it may reserve while running. Without a
//! per-child ceiling, an allocation blowup raises host-global memory
//! pressure long before a slow-growing worker would ever trip its own
//! deadline, and once pressure is host-global the kernel OOM killer picks
//! *its own* victim rather than the runaway child — on this estate's own
//! host that has repeatedly meant infrastructure dies, not the hog. So
//! [`spawn_and_collect`] also arms [`WORKER_ADDRESS_SPACE_LIMIT_BYTES`] as
//! `RLIMIT_AS` on the child (`cap_worker_address_space`), in the identical
//! post-fork, pre-exec, one-thread window
//! [`crate::backend::child::harden_probe_child`]'s own `PR_SET_PDEATHSIG`
//! closure already documents as safe for this call shape — a second concern
//! armed the same way at the same call site, not a second mechanism. A
//! child that exceeds it dies on its own, well inside the deadline, and is
//! reported the same way every other Y1 fault is: a named [`CoverageRow`]
//! ([`WorkerFault::MemoryLimitExceeded`]), never a silent absorption into
//! host-wide pressure.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::backend::child;
use crate::domain::is_plain_name;
use crate::domain::source::{Coverage, CoverageRow, UnitKind};
use crate::runtime::atlas::deny::{AcquisitionFilter, Verdict};

/// How often [`run_worker`] polls a live child against its deadline.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// The address-space (`RLIMIT_AS`) ceiling armed on every worker child at
/// spawn (`cap_worker_address_space`) — the memory-fault class the deadline
/// alone cannot close (module doc, "Memory containment").
///
/// **512 MiB, PROVISIONAL.** This build's own rule is that a numeric
/// default is a suspect until traced to a dated measurement, and this one
/// has not been: Y1 ships no real parser (the worker body is trivial), so
/// there is no real document corpus to size this against yet. 512 MiB is a
/// working bound chosen to be far below ordinary host headroom, not a
/// validated ceiling. It **must be re-derived against a real corpus** once
/// the first real parser (the Y2 Anydoc/Office adapter) lands, and this
/// comment must be updated to cite that measurement when it does.
///
/// **This is a per-child cap, not a total.** Each worker child gets its own
/// independent `RLIMIT_AS` of this size; workers run concurrently up to the
/// intelligence lane's own concurrency cap
/// ([`crate::runtime::engine::default_intelligence_lane_cap`]), so the real
/// worst-case worker memory ceiling is `lane_cap * WORKER_ADDRESS_SPACE_LIMIT_BYTES`,
/// not this constant alone. Any future sizing of this constant, or of the
/// lane cap, must account for that product.
///
/// `pub` (not `pub(crate)`) so the deterministic memory-fault acceptance
/// test (`tests/y1_worker_transport.rs`) can size its own deadline against
/// the real value rather than duplicating it.
pub const WORKER_ADDRESS_SPACE_LIMIT_BYTES: u64 = 512 * 1024 * 1024;

/// How long [`run_worker`] waits, after the child has already exited or been
/// reaped, for its stdout-draining thread to hand back what it read.
///
/// Decoupled from the caller's own deadline on purpose: that deadline bounds
/// how long the *child* may run, and by the time this wait starts the child
/// is already gone — a pipe with a closed write end drains in microseconds,
/// so this is a generous bound against a wedged reader thread, not a second
/// copy of the run budget.
const STDOUT_DRAIN_GRACE: Duration = Duration::from_secs(5);

// ------------------------------------------------------------- wire shapes

/// One structure unit a worker derived, in [`super::scan::ScannedUnit`]'s own
/// shape minus the fields only the walk that found the file can fill in
/// (`ordinal`, heading metadata) — Y1 has no real adapter to populate those,
/// and a wire field nothing ever sets is exactly the false promise the
/// empty-table doctrine already refuses for a table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerUnit {
    /// Whole resource, or a container-defined span.
    pub kind: UnitKind,
    /// Offset into the original resource bytes.
    pub byte_start: u64,
    /// End offset into the original resource bytes, exclusive.
    pub byte_end: u64,
    /// The unit's own text.
    pub text: String,
}

/// One child resource a worker declares out of the bytes it was given (a
/// future archive entry, mail attachment, or similar container member).
///
/// Untrusted by construction: [`validate_batch`] is what decides whether
/// `relative_path` may ever reach a filesystem or a store, and the daemon
/// runs that check on every declared child before touching either.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredChild {
    /// The child's own name, exactly as the worker declared it — checked
    /// against F10's deny set independently of `relative_path`, because a
    /// deny pattern like the dotfile rule matches on a path *component*, and
    /// a worker could otherwise bury a denied name one path segment deep
    /// from the coordinate a naive check might use.
    pub name: String,
    /// Where the child lives relative to its parent resource, `/`-separated.
    pub relative_path: String,
}

/// One supervised worker's whole answer for one resource: bytes in,
/// normalized batch out (G2's own words).
///
/// Every field here is attacker-influenced input from the daemon's point of
/// view, whether the worker is buggy, compromised, or simply running old
/// code against a new deny set — which is the entire reason
/// [`validate_batch`] exists rather than trusting this struct on arrival.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerBatch {
    /// The generation this extraction claims to belong to.
    pub generation_id: String,
    /// The worker's own hash of the bytes it was given (BLAKE3 hex) — F7's
    /// content half, computed independently on both sides of the pipe so a
    /// mismatch is detectable rather than assumed.
    pub resource_hash: String,
    /// The extractor identity this batch claims to have run (F7's other
    /// half).
    pub extractor: String,
    /// Structure units this worker derived.
    pub units: Vec<WorkerUnit>,
    /// Child resources this worker declares out of the input bytes.
    pub declared_children: Vec<DeclaredChild>,
}

/// What the daemon already knows to be true about a dispatched job, checked
/// against what the worker's [`WorkerBatch`] claims.
///
/// Composed daemon-side from evidence the worker never chose: `resource_hash`
/// is the daemon's own hash of the bytes it is about to send, taken before
/// the worker ever runs, not copied from anything the worker said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerIdentity {
    /// The generation this job is extracting for.
    pub generation_id: String,
    /// BLAKE3 hex of the exact bytes handed to the worker.
    pub resource_hash: String,
    /// The extractor identity the daemon dispatched this job to run.
    pub extractor: String,
}

// --------------------------------------------------------- daemon-side authority

/// Why [`validate_batch`] refused a [`WorkerBatch`] — the AUTHORITY's own
/// verdict, never the worker's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchRefusal {
    /// A returned batch's identity does not match what the daemon dispatched
    /// — a stale answer, a worker that read the wrong job, or worse.
    IdentityMismatch {
        /// Which identity field disagreed.
        field: &'static str,
        /// What the daemon expected.
        expected: String,
        /// What the batch claimed.
        got: String,
    },
    /// A declared child's path is not `enclosed_name`-safe: an absolute
    /// path, a `..` component, or an empty segment.
    UnsafeChildPath {
        /// The child's declared name.
        child: String,
        /// The unsafe path it declared.
        path: String,
    },
    /// A declared child's name or path matches F10's deny set.
    DeniedChildName {
        /// The child's declared name.
        child: String,
        /// The deny pattern that matched.
        pattern: String,
    },
}

impl BatchRefusal {
    /// This refusal, as the named [`CoverageRow`] G2 requires: never a
    /// silent drop, and named specifically enough that an operator reading
    /// `sgt doctor`'s atlas row (or `source.scanned`'s coverage counts) can
    /// tell a refused worker batch from an ordinary extraction error.
    ///
    /// [`Coverage::Excluded`] only for [`Self::DeniedChildName`] — that is
    /// literally F10's own deny-set boundary, the same status an ordinary
    /// scan's own `ignore` refusal reports. The other two are not "excluded
    /// by policy", they are "this answer cannot be trusted", which
    /// [`Coverage::Error`] already means.
    pub fn coverage_row(&self) -> CoverageRow {
        let status = match self {
            Self::DeniedChildName { .. } => Coverage::Excluded,
            Self::IdentityMismatch { .. } | Self::UnsafeChildPath { .. } => Coverage::Error,
        };
        let detail = match self {
            Self::IdentityMismatch {
                field,
                expected,
                got,
            } => format!(
                "supervised parse worker's returned batch was refused: {field} was {got:?}, \
                 expected {expected:?}"
            ),
            Self::UnsafeChildPath { child, path } => format!(
                "supervised parse worker declared child {child:?} at unsafe path {path:?} \
                 (not enclosed-name-safe); refused before it could reach the store"
            ),
            Self::DeniedChildName { child, pattern } => format!(
                "supervised parse worker declared child {child:?}, which matches the F10 \
                 deny set ({pattern}); refused before it could reach the store"
            ),
        };
        CoverageRow {
            path: None,
            status,
            detail: Some(detail),
            bytes: None,
        }
    }
}

/// `enclosed_name` semantics (the `zip` crate's own term — no dependency on
/// it here, since Y1 ships no container format; the semantics are what G2
/// asks every declared child path to satisfy regardless of which wave adds
/// the first real container): a relative path with no absolute component, no
/// `..`, and no empty segment.
///
/// R2: reuses [`is_plain_name`] per `/`-separated component, exactly the
/// pattern its own doc already demonstrates for a composed hierarchical
/// path — a single guard fixed in one place rather than a second
/// path-traversal check copied for containers.
fn enclosed_relative_path(path: &str) -> bool {
    path.split('/').all(is_plain_name)
}

/// The AUTHORITY over a returned batch (G2, panel-adjudicated): identity,
/// then path safety, then F10 deny-set membership — on every declared
/// child's name AND its path, because a deny pattern can match either.
///
/// Checked in this order so the *first* thing wrong with a batch is what a
/// coverage row names — a batch failing on identity never gets far enough to
/// be told its child paths were also unsafe, which keeps one refusal one
/// reason.
pub fn validate_batch(
    identity: &WorkerIdentity,
    batch: &WorkerBatch,
    deny: &AcquisitionFilter,
) -> Result<(), BatchRefusal> {
    if batch.generation_id != identity.generation_id {
        return Err(BatchRefusal::IdentityMismatch {
            field: "generation_id",
            expected: identity.generation_id.clone(),
            got: batch.generation_id.clone(),
        });
    }
    if batch.resource_hash != identity.resource_hash {
        return Err(BatchRefusal::IdentityMismatch {
            field: "resource_hash",
            expected: identity.resource_hash.clone(),
            got: batch.resource_hash.clone(),
        });
    }
    if batch.extractor != identity.extractor {
        return Err(BatchRefusal::IdentityMismatch {
            field: "extractor",
            expected: identity.extractor.clone(),
            got: batch.extractor.clone(),
        });
    }
    for declared in &batch.declared_children {
        if !enclosed_relative_path(&declared.relative_path) {
            return Err(BatchRefusal::UnsafeChildPath {
                child: declared.name.clone(),
                path: declared.relative_path.clone(),
            });
        }
        // F10: matched on the declared NAME as well as its path — a name
        // like `.env` must be refused even when the path it was declared at
        // does not itself trip the dotfile rule (a container may place a
        // secrets-shaped entry under a name that reads fine as a path
        // component's sibling but not as itself).
        if let Verdict::Denied { pattern } = deny.verdict(&declared.name) {
            return Err(BatchRefusal::DeniedChildName {
                child: declared.name.clone(),
                pattern,
            });
        }
        if let Verdict::Denied { pattern } = deny.verdict(&declared.relative_path) {
            return Err(BatchRefusal::DeniedChildName {
                child: declared.name.clone(),
                pattern,
            });
        }
    }
    Ok(())
}

// ------------------------------------------------------------- supervision

/// One supervised worker call: the program to spawn, its arguments, the
/// bytes to feed its stdin, and the deadline it must finish inside.
///
/// `program` is a plain `PathBuf` rather than [`std::env::current_exe`]
/// baked in here, because the two callers need two different answers: the
/// daemon's own binary IS the worker binary's addressable path at runtime
/// (`std::env::current_exe()`, [`crate::cli`]'s `spawn_daemon` sets the same
/// precedent for re-exec'ing the running binary with a different verb), while
/// an integration test spawns the worker binary Cargo built for it
/// (`env!("CARGO_BIN_EXE_sgt-atlas-worker")`) — `current_exe()` inside a test
/// binary answers with the *test binary's own* path, which is not runnable
/// as a worker at all.
#[derive(Debug, Clone)]
pub struct WorkerSpawn {
    /// The worker binary to run.
    pub program: PathBuf,
    /// Its command-line arguments.
    pub args: Vec<String>,
    /// The resource bytes to write to its stdin, then close.
    pub input: Vec<u8>,
    /// How long the worker may run before [`run_worker`] kills its whole
    /// process group and reaps it.
    pub deadline: Duration,
}

/// What one supervised call produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerOutcome {
    /// A batch that passed [`validate_batch`].
    Accepted(WorkerBatch),
    /// The worker's process misbehaved, or its batch was refused — either
    /// way, a named [`CoverageRow`] rather than a silent gap.
    Refused(CoverageRow),
}

/// Why a supervised worker call did not produce a batch at all — a
/// process-level fault, as opposed to [`BatchRefusal`], which is about a
/// batch the worker *did* produce.
#[derive(Debug)]
enum WorkerFault {
    /// The worker could not even be spawned (missing binary, exec
    /// permission, …).
    SpawnFailed(String),
    /// The worker exceeded its deadline; its whole process group was killed
    /// and reaped.
    TimedOut,
    /// The worker was terminated by a signal (a `SIGABRT`'d worker lands
    /// here, `signal` is the raw number).
    Signaled { signal: i32, stderr_tail: String },
    /// The worker was terminated after exceeding
    /// [`WORKER_ADDRESS_SPACE_LIMIT_BYTES`] (`RLIMIT_AS`, armed by
    /// `cap_worker_address_space` at spawn).
    ///
    /// The kernel enforces the ceiling by failing the child's own
    /// allocation (`RLIMIT_AS` makes the mapping syscall behind it return
    /// `ENOMEM`), and what that failure does to the child varies by which
    /// allocator hit it: Rust's default allocator aborts (the worker
    /// self-terminates by signal, `signal` is `Some`); a fallible-allocation
    /// path or a third-party C library's own OOM handler instead panics or
    /// calls `_exit` and the process exits non-zero on its own (`signal` is
    /// `None`, `code` is the exit code). [`fault_for_exit`] tells either
    /// shape apart from an ordinary [`Self::Signaled`]/[`Self::ExitedNonZero`]
    /// by matching a known allocation-failure signature in `stderr_tail`
    /// (`matches_allocation_failure`), never by which raw signal number or
    /// exit code the platform happens to produce (both vary by target,
    /// allocator, and toolchain version; the message families do not).
    MemoryLimitExceeded {
        signal: Option<i32>,
        code: Option<i32>,
        stderr_tail: String,
    },
    /// The worker exited on its own, with a non-zero status.
    ExitedNonZero {
        code: Option<i32>,
        stderr_tail: String,
    },
    /// The worker exited zero but its stdout was not a [`WorkerBatch`].
    Malformed { detail: String },
}

impl WorkerFault {
    fn coverage_row(&self) -> CoverageRow {
        let detail = match self {
            Self::SpawnFailed(e) => {
                format!("supervised parse worker could not be spawned: {e}")
            }
            Self::TimedOut => {
                "supervised parse worker exceeded its deadline and was killed (group signalled, \
                 reaped)"
                    .to_string()
            }
            Self::Signaled {
                signal,
                stderr_tail,
            } => format!(
                "supervised parse worker was terminated by signal {signal}; stderr: \
                 {stderr_tail}"
            ),
            Self::MemoryLimitExceeded {
                signal,
                code,
                stderr_tail,
            } => {
                let how = match (signal, code) {
                    (Some(signal), _) => format!("signal {signal}"),
                    (None, Some(code)) => format!("exit code {code}"),
                    (None, None) => "an unknown exit".to_string(),
                };
                format!(
                    "supervised parse worker exceeded its {WORKER_ADDRESS_SPACE_LIMIT_BYTES}-byte \
                     address-space limit and was terminated ({how}); stderr: {stderr_tail}"
                )
            }
            Self::ExitedNonZero { code, stderr_tail } => format!(
                "supervised parse worker exited with status {code:?}; stderr: {stderr_tail}"
            ),
            Self::Malformed { detail } => {
                format!("supervised parse worker's stdout was not a usable batch: {detail}")
            }
        };
        CoverageRow {
            path: None,
            status: Coverage::Error,
            detail: Some(detail),
            bytes: None,
        }
    }
}

/// Run one supervised parse worker end to end: spawn (own process group,
/// `PR_SET_PDEATHSIG` via [`child::harden_probe_child`]), feed it `input` on
/// stdin, wait bounded by `deadline` (kill the group and reap past it, exit
/// or signal short of it), parse its stdout, and — only for a worker that
/// actually produced a batch — run it through [`validate_batch`].
///
/// The intelligence-lane permit is not acquired here: [`super::lane`]'s
/// [`super::lane::run_worker_on_lane`] is what a daemon caller actually
/// calls, and it wraps this function in [`crate::runtime::engine::Engine::run_intelligence`]
/// the same way [`super::lane::scan_estate_git_on_lane`] wraps
/// [`super::git::scan_estate_git`] — this function stays engine-agnostic so
/// it is independently testable without a daemon.
pub fn run_worker(
    spawn: WorkerSpawn,
    identity: &WorkerIdentity,
    deny: &AcquisitionFilter,
) -> WorkerOutcome {
    match spawn_and_collect(&spawn) {
        Ok(stdout) => match serde_json::from_slice::<WorkerBatch>(&stdout) {
            Ok(batch) => match validate_batch(identity, &batch, deny) {
                Ok(()) => WorkerOutcome::Accepted(batch),
                Err(refusal) => WorkerOutcome::Refused(refusal.coverage_row()),
            },
            Err(e) => WorkerOutcome::Refused(
                WorkerFault::Malformed {
                    detail: e.to_string(),
                }
                .coverage_row(),
            ),
        },
        Err(fault) => WorkerOutcome::Refused(fault.coverage_row()),
    }
}

/// The whole of one child's lifetime: spawn, hardened; stdin written and
/// closed on its own thread; stdout drained on its own thread; `try_wait`
/// polled against `spawn.deadline`; kill + reap on either a timeout or a
/// normal exit, always both, never one without the other (#310).
fn spawn_and_collect(spawn: &WorkerSpawn) -> Result<Vec<u8>, WorkerFault> {
    let mut command = Command::new(&spawn.program);
    command
        .args(&spawn.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Own process group + PR_SET_PDEATHSIG (Linux): a worker call is spawned
    // and killed inside this one function, on one thread — exactly the shape
    // `ChildLifetime::Probe` documents as safe to harden this way (#310).
    child::harden_probe_child(&mut command);
    // RLIMIT_AS (Linux): the memory-fault class #310's hardening above does
    // not cover — see the module doc's "Memory containment" section.
    cap_worker_address_space(&mut command);

    let mut process = command
        .spawn()
        .map_err(|e| WorkerFault::SpawnFailed(e.to_string()))?;
    let pgid = process.id();
    let registration = child::register_probe_child(pgid);

    if let Some(mut stdin) = process.stdin.take() {
        let input = spawn.input.clone();
        std::thread::spawn(move || {
            // A worker that never reads stdin (every Y1 fault mode) still
            // must not deadlock this write: inputs in this build are small
            // fixture bytes, and the OS pipe buffer absorbs them without the
            // writer blocking. A real Y2 adapter's own transport is free to
            // widen this if a resource ever needs to be larger than a pipe
            // buffer, which is a Y2-scoped concern, not this wave's.
            let _ = stdin.write_all(&input);
            // `stdin` drops here, closing the pipe — the EOF a worker that
            // *does* read stdin needs to see.
        });
    }

    let (stdout_tx, stdout_rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(1);
    if let Some(mut stdout) = process.stdout.take() {
        std::thread::spawn(move || {
            let mut buffer = Vec::new();
            let _ = stdout.read_to_end(&mut buffer);
            let _ = stdout_tx.send(buffer);
        });
    } else {
        let _ = stdout_tx.send(Vec::new());
    }

    let deadline_at = Instant::now() + spawn.deadline;
    let status = loop {
        match process.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline_at {
                    break None;
                }
                std::thread::sleep(POLL_INTERVAL);
            }
            // An error probing the child's state is treated the same as a
            // timeout below: kill the group and reap, rather than looping on
            // an error that will not resolve itself.
            Err(_) => break None,
        }
    };

    let Some(status) = status else {
        // Deadline exceeded (or `try_wait` itself failed): the group is
        // killed by recorded pgid — never by name — and reaped. Both halves,
        // always, or a killed-not-reaped child is a zombie an orphan check
        // cannot tell from a live leak (#310).
        child::kill_process_group(Some(pgid));
        let _ = process.kill();
        let reaped = process.wait();
        drop(registration);
        return match reaped {
            Ok(_) => Err(WorkerFault::TimedOut),
            Err(e) => Err(WorkerFault::SpawnFailed(format!(
                "kill/reap after the deadline itself failed: {e}"
            ))),
        };
    };
    // The child itself already exited (signalled, non-zero, or clean) — but
    // #310's contract is "kill the group, then reap" on every exit path, not
    // only the deadline one: a child that forked its own subprocess before
    // dying leaves that grandchild in the same pgid, unreachable once
    // `registration` is dropped below. Signalling an already-empty group is
    // a documented no-op (`kill_process_group`'s own doc), so this costs
    // nothing on the common case where the worker never forked anything.
    child::kill_process_group(Some(pgid));
    drop(registration);

    if let Some(fault) = fault_for_exit(status, &mut process) {
        return Err(fault);
    }

    match stdout_rx.recv_timeout(STDOUT_DRAIN_GRACE) {
        Ok(buffer) => Ok(buffer),
        Err(RecvTimeoutError::Timeout) => Err(WorkerFault::Malformed {
            detail: "the worker exited but its stdout-draining thread never finished".to_string(),
        }),
        Err(RecvTimeoutError::Disconnected) => Err(WorkerFault::Malformed {
            detail: "the worker exited and its stdout reader thread vanished without a report"
                .to_string(),
        }),
    }
}

/// Arm [`WORKER_ADDRESS_SPACE_LIMIT_BYTES`] as `RLIMIT_AS` on `command`'s
/// child, in the identical post-fork, pre-exec window
/// [`child::harden_probe_child`]'s own `PR_SET_PDEATHSIG` closure already
/// documents as safe for this call shape: spawned and killed inside one
/// function, on one thread ([`child::ChildLifetime::Probe`]'s own doc).
///
/// `setrlimit` is a plain syscall — no allocation, no libc global state
/// beyond the kernel's own per-process limit table — the same class of
/// async-signal-safe primitive `harden_probe_child`'s own SAFETY comment
/// already relies on for `prctl`/`getppid` in this same window.
///
/// Linux-only, matching `harden_probe_child`'s own `PR_SET_PDEATHSIG` gate:
/// this module makes no memory-containment promise on a platform where
/// neither mechanism exists (module doc, "Memory containment").
#[cfg(target_os = "linux")]
fn cap_worker_address_space(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    let limit = WORKER_ADDRESS_SPACE_LIMIT_BYTES as libc::rlim_t;
    // SAFETY: see this function's own doc, and `harden_probe_child`'s
    // matching SAFETY comment — identical post-fork window, identical
    // async-signal-safe-only constraint, satisfied the same way.
    unsafe {
        command.pre_exec(move || {
            let limit = libc::rlimit {
                rlim_cur: limit,
                rlim_max: limit,
            };
            if libc::setrlimit(libc::RLIMIT_AS, &limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

/// The non-Linux half of [`cap_worker_address_space`]: named rather than
/// omitted, exactly the reasoning [`child::harden_probe_child`]'s own
/// non-unix arm gives for doing the same.
#[cfg(not(target_os = "linux"))]
fn cap_worker_address_space(_command: &mut Command) {}

/// Well-known allocation-failure message signatures, each expressed as the
/// set of substrings that must **all** appear in a child's stderr tail for
/// that signature to match ([`matches_allocation_failure`]).
///
/// Deliberately narrow: every entry is an exact fragment of a real
/// allocator/runtime's own failure message, never a generic word ("error",
/// "abort", "fatal") that an unrelated fault could also print. A multi-part
/// entry requires every one of its fragments together, specifically so a
/// message that merely shares one common word with a real signature (e.g.
/// the standalone word "failed") cannot match on its own. Matching here only
/// ever *upgrades* an exit or signal termination that is already known to be
/// abnormal into the more specific [`WorkerFault::MemoryLimitExceeded`] — it
/// is never consulted on a clean exit, so a false positive here can at worst
/// relabel one already-bad outcome as another; it can never manufacture a
/// fault, and it can never produce [`WorkerFault::TimedOut`] (the deadline
/// path never calls this function at all).
const ALLOCATION_FAILURE_SIGNATURES: &[&[&str]] = &[
    // Rust's global-allocator abort (`alloc::alloc::handle_alloc_error`),
    // hit when an infallible `Vec`/`Box`/`String`/`Arc` allocation's
    // underlying `malloc`/mmap call returns null. The historical signature
    // this classifier already matched.
    &["memory allocation of", "failed"],
    // Rust's *fallible* allocation path (`Vec::try_reserve`,
    // `HashMap::try_reserve`, …) reports a real allocation failure as the
    // `AllocError` variant of `TryReserveErrorKind` rather than aborting; an
    // `.unwrap()`/`.expect()` on that `Err` panics (exit code 101, no
    // signal) with both fragments in the message. `AllocError` is required
    // alongside `TryReserveError` because the *other* variant of the same
    // error type, `CapacityOverflow`, is a logic bug (an oversized
    // `len * size_of::<T>()`) rather than a real allocation failure, and
    // must not be misclassified as the memory cap.
    &["TryReserveError", "AllocError"],
    // An `mmap`/allocation call surfaced through `std::io::Error` renders an
    // `ENOMEM` failure as libc's own `strerror(ENOMEM)` text — exact,
    // OS-supplied wording, not a fragment this build invented.
    &["Cannot allocate memory"],
    // glibc's own malloc-arena OOM abort, hit by any third-party C/C++
    // library linked into the worker (not only Rust code) — a real shape
    // review found `RLIMIT_AS` can produce without the child being Rust at
    // all.
    &["malloc(): unable to allocate memory"],
];

/// Whether `stderr_tail` carries one of [`ALLOCATION_FAILURE_SIGNATURES`] in
/// full (every fragment of at least one signature present).
fn matches_allocation_failure(stderr_tail: &str) -> bool {
    ALLOCATION_FAILURE_SIGNATURES.iter().any(|signature| {
        signature
            .iter()
            .all(|fragment| stderr_tail.contains(fragment))
    })
}

/// `None` for a clean exit; `Some` naming a signal termination or a
/// non-zero status, stderr tail attached either way.
///
/// Either shape — signalled or a plain non-zero exit — is reported as
/// [`WorkerFault::MemoryLimitExceeded`] instead of a plain
/// [`WorkerFault::Signaled`]/[`WorkerFault::ExitedNonZero`] when its stderr
/// carries a [`matches_allocation_failure`] signature: the message, not the
/// raw signal number or exit code (both platform- and toolchain-dependent),
/// is what reliably names an `RLIMIT_AS` kill, and an `RLIMIT_AS` kill does
/// not always exit the same way (module doc; [`WorkerFault::MemoryLimitExceeded`]'s
/// own doc). Absent a match, this falls back to the existing honest
/// [`WorkerFault::Signaled`]/[`WorkerFault::ExitedNonZero`] label rather than
/// guessing.
fn fault_for_exit(status: ExitStatus, process: &mut std::process::Child) -> Option<WorkerFault> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            let stderr_tail = drain_stderr(process);
            if matches_allocation_failure(&stderr_tail) {
                return Some(WorkerFault::MemoryLimitExceeded {
                    signal: Some(signal),
                    code: None,
                    stderr_tail,
                });
            }
            return Some(WorkerFault::Signaled {
                signal,
                stderr_tail,
            });
        }
    }
    if !status.success() {
        let stderr_tail = drain_stderr(process);
        let code = status.code();
        if matches_allocation_failure(&stderr_tail) {
            return Some(WorkerFault::MemoryLimitExceeded {
                signal: None,
                code,
                stderr_tail,
            });
        }
        return Some(WorkerFault::ExitedNonZero { code, stderr_tail });
    }
    None
}

/// Read a child's stderr to completion. Only ever called once the child has
/// already exited or been reaped, so the pipe's write end is already closed
/// and this is a bounded read, not a blocking wait on a live process.
fn drain_stderr(process: &mut std::process::Child) -> String {
    let Some(mut stderr) = process.stderr.take() else {
        return String::new();
    };
    let mut buffer = Vec::new();
    let _ = stderr.read_to_end(&mut buffer);
    String::from_utf8_lossy(&buffer).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> WorkerIdentity {
        WorkerIdentity {
            generation_id: "gen-1".to_string(),
            resource_hash: "hash-1".to_string(),
            extractor: "fixture/v1".to_string(),
        }
    }

    fn batch() -> WorkerBatch {
        WorkerBatch {
            generation_id: "gen-1".to_string(),
            resource_hash: "hash-1".to_string(),
            extractor: "fixture/v1".to_string(),
            units: Vec::new(),
            declared_children: Vec::new(),
        }
    }

    fn deny() -> AcquisitionFilter {
        AcquisitionFilter::new(&[]).expect("compile default deny set")
    }

    #[test]
    fn a_matching_batch_with_no_children_is_accepted() {
        assert_eq!(validate_batch(&identity(), &batch(), &deny()), Ok(()));
    }

    #[test]
    fn a_mismatched_generation_is_refused_by_identity() {
        let mut bad = batch();
        bad.generation_id = "gen-2".to_string();
        let err = validate_batch(&identity(), &bad, &deny()).expect_err("must refuse");
        assert_eq!(
            err,
            BatchRefusal::IdentityMismatch {
                field: "generation_id",
                expected: "gen-1".to_string(),
                got: "gen-2".to_string(),
            }
        );
        assert_eq!(err.coverage_row().status, Coverage::Error);
    }

    #[test]
    fn a_mismatched_resource_hash_is_refused_by_identity() {
        let mut bad = batch();
        bad.resource_hash = "hash-2".to_string();
        let err = validate_batch(&identity(), &bad, &deny()).expect_err("must refuse");
        assert_eq!(
            err,
            BatchRefusal::IdentityMismatch {
                field: "resource_hash",
                expected: "hash-1".to_string(),
                got: "hash-2".to_string(),
            }
        );
    }

    #[test]
    fn a_mismatched_extractor_is_refused_by_identity() {
        let mut bad = batch();
        bad.extractor = "fixture/v2".to_string();
        let err = validate_batch(&identity(), &bad, &deny()).expect_err("must refuse");
        assert_eq!(
            err,
            BatchRefusal::IdentityMismatch {
                field: "extractor",
                expected: "fixture/v1".to_string(),
                got: "fixture/v2".to_string(),
            }
        );
    }

    /// The brief's own example: a worker declaring a child named `.env` is
    /// refused daemon-side even though `.env` alone carries no `..` and no
    /// leading slash — this is the deny-set check, not the path-safety one.
    #[test]
    fn a_declared_child_named_dotenv_is_refused_by_the_deny_set() {
        let mut with_child = batch();
        with_child.declared_children.push(DeclaredChild {
            name: ".env".to_string(),
            relative_path: ".env".to_string(),
        });
        let err = validate_batch(&identity(), &with_child, &deny()).expect_err("must refuse");
        assert!(matches!(err, BatchRefusal::DeniedChildName { .. }));
    }

    /// The brief's other example: `../../etc/passwd` is a path-safety
    /// refusal, independent of whether the deny set would also catch it.
    #[test]
    fn a_declared_child_at_a_traversal_path_is_refused_by_path_safety() {
        let mut with_child = batch();
        with_child.declared_children.push(DeclaredChild {
            name: "innocuous.txt".to_string(),
            relative_path: "../../etc/passwd".to_string(),
        });
        let err = validate_batch(&identity(), &with_child, &deny()).expect_err("must refuse");
        assert_eq!(
            err,
            BatchRefusal::UnsafeChildPath {
                child: "innocuous.txt".to_string(),
                path: "../../etc/passwd".to_string(),
            }
        );
    }

    /// A denied *name* refuses even when the declared *path* alone would
    /// pass both checks — the brief is explicit that the deny set must match
    /// on the name as well as the path.
    #[test]
    fn a_denied_name_refuses_even_under_an_innocuous_looking_path() {
        let mut with_child = batch();
        with_child.declared_children.push(DeclaredChild {
            name: "id_rsa".to_string(),
            relative_path: "assets/id_rsa".to_string(),
        });
        let err = validate_batch(&identity(), &with_child, &deny()).expect_err("must refuse");
        assert!(matches!(err, BatchRefusal::DeniedChildName { .. }));
    }

    #[test]
    fn an_ordinary_nested_child_path_is_enclosed_and_allowed() {
        let mut with_child = batch();
        with_child.declared_children.push(DeclaredChild {
            name: "logo.png".to_string(),
            relative_path: "assets/images/logo.png".to_string(),
        });
        assert_eq!(validate_batch(&identity(), &with_child, &deny()), Ok(()));
    }

    #[test]
    fn enclosed_relative_path_rejects_absolute_and_traversal_and_empty_segments() {
        for unsafe_path in ["/etc/passwd", "../escape", "a/../b", "a//b", "", "a/"] {
            assert!(
                !enclosed_relative_path(unsafe_path),
                "{unsafe_path:?} must not be enclosed-safe"
            );
        }
        for safe_path in ["file.txt", "dir/file.txt", "a/b/c.md"] {
            assert!(
                enclosed_relative_path(safe_path),
                "{safe_path:?} must be enclosed-safe"
            );
        }
    }

    /// A worker binary that does not exist at all is a spawn failure, not a
    /// panic and not a hang — the same "reported, never absorbed" rule every
    /// other Atlas failure mode already follows.
    #[test]
    fn a_program_that_does_not_exist_is_a_named_spawn_failure() {
        let outcome = run_worker(
            WorkerSpawn {
                program: PathBuf::from("/definitely/not/a/real/sgt-atlas-worker/binary"),
                args: Vec::new(),
                input: Vec::new(),
                deadline: Duration::from_millis(500),
            },
            &identity(),
            &deny(),
        );
        let WorkerOutcome::Refused(row) = outcome else {
            panic!("a missing binary must not be accepted");
        };
        assert_eq!(row.status, Coverage::Error);
        assert!(
            row.detail
                .as_deref()
                .unwrap_or_default()
                .contains("could not be spawned"),
            "{row:?}"
        );
    }

    // --------------------------------------------------------- FIX 2 tests
    //
    // `matches_allocation_failure` is exercised directly against crafted
    // stderr text (no real memory exhaustion needed — the deterministic
    // `--fault allocate` acceptance test in `tests/y1_worker_transport.rs`
    // already proves the real RLIMIT_AS-kill path end to end). The
    // `run_worker`-level tests below use `/bin/sh` to produce a *real*
    // signalled/exited child whose stderr and exit shape are crafted, so
    // `fault_for_exit` itself — not just the pure matcher — is proven for
    // the two newly-classified shapes (a).(b) the review named.

    #[test]
    fn the_historical_rust_abort_message_matches() {
        assert!(matches_allocation_failure(
            "thread 'main' panicked at 'memory allocation of 4096 bytes failed'"
        ));
    }

    #[test]
    fn a_rust_try_reserve_alloc_error_matches() {
        assert!(matches_allocation_failure(
            "called `Result::unwrap()` on an `Err` value: TryReserveError(AllocError { \
             layout: ..., non_exhaustive: () })"
        ));
    }

    #[test]
    fn a_try_reserve_capacity_overflow_does_not_match() {
        // CapacityOverflow is a logic bug (an oversized `len * size_of::<T>()`
        // computation), not a real allocation failure — must not be
        // misclassified as the memory cap even though it shares the
        // `TryReserveError` fragment.
        assert!(!matches_allocation_failure(
            "called `Result::unwrap()` on an `Err` value: TryReserveError(CapacityOverflow)"
        ));
    }

    #[test]
    fn an_enomem_io_error_matches() {
        assert!(matches_allocation_failure(
            "mmap failed: Cannot allocate memory (os error 12)"
        ));
    }

    #[test]
    fn a_glibc_malloc_abort_matches() {
        assert!(matches_allocation_failure(
            "malloc(): unable to allocate memory\nAborted"
        ));
    }

    #[test]
    fn unrelated_stderr_does_not_match() {
        for stderr in [
            "",
            "failed",
            "error: unexpected token",
            "thread 'main' panicked at 'index out of bounds'",
            "out of memory", // deliberately excluded: too generic on its own
        ] {
            assert!(
                !matches_allocation_failure(stderr),
                "{stderr:?} must not match an allocation-failure signature"
            );
        }
    }

    /// Shape (a) from the review: a Rust panic path (e.g. an unwrapped
    /// `Vec::try_reserve` failure) exits non-zero (code 101, no signal) —
    /// this must now be named the memory cap, not `ExitedNonZero`.
    #[test]
    fn a_nonzero_exit_with_an_allocation_signature_is_classified_as_the_memory_cap() {
        let outcome = run_worker(
            WorkerSpawn {
                program: PathBuf::from("/bin/sh"),
                args: vec![
                    "-c".to_string(),
                    "printf '%s' \"called \\`Result::unwrap()\\` on an \\`Err\\` value: \
                     TryReserveError(AllocError)\" 1>&2; exit 101"
                        .to_string(),
                ],
                input: Vec::new(),
                deadline: Duration::from_secs(5),
            },
            &identity(),
            &deny(),
        );
        let WorkerOutcome::Refused(row) = outcome else {
            panic!("must be refused: {outcome:?}");
        };
        let detail = row.detail.unwrap_or_default();
        assert!(
            detail.contains("address-space limit") && detail.contains("exit code 101"),
            "a panicking allocation failure must be named the memory cap: {detail:?}"
        );
    }

    /// A non-zero exit whose stderr carries no allocation signature must
    /// stay the honest `ExitedNonZero` label — the conservative fallback
    /// the review required.
    #[test]
    fn a_nonzero_exit_without_an_allocation_signature_stays_exited_non_zero() {
        let outcome = run_worker(
            WorkerSpawn {
                program: PathBuf::from("/bin/sh"),
                args: vec![
                    "-c".to_string(),
                    "printf 'ordinary failure' 1>&2; exit 7".to_string(),
                ],
                input: Vec::new(),
                deadline: Duration::from_secs(5),
            },
            &identity(),
            &deny(),
        );
        let WorkerOutcome::Refused(row) = outcome else {
            panic!("must be refused: {outcome:?}");
        };
        let detail = row.detail.unwrap_or_default();
        assert!(
            !detail.contains("address-space limit"),
            "an unrelated non-zero exit must not be named the memory cap: {detail:?}"
        );
        assert!(detail.contains("exited with status"), "{detail:?}");
    }

    /// Shape (b) from the review: a third-party C library aborting on its
    /// own OOM path with glibc's message, terminated by `SIGABRT` rather
    /// than `RLIMIT_AS` directly — this must still be named the memory cap
    /// when the evidence (the message) supports it.
    #[test]
    fn a_signalled_exit_with_a_glibc_allocation_signature_is_classified_as_the_memory_cap() {
        let outcome = run_worker(
            WorkerSpawn {
                program: PathBuf::from("/bin/sh"),
                args: vec![
                    "-c".to_string(),
                    "printf 'malloc(): unable to allocate memory' 1>&2; kill -ABRT $$".to_string(),
                ],
                input: Vec::new(),
                deadline: Duration::from_secs(5),
            },
            &identity(),
            &deny(),
        );
        let WorkerOutcome::Refused(row) = outcome else {
            panic!("must be refused: {outcome:?}");
        };
        let detail = row.detail.unwrap_or_default();
        assert!(
            detail.contains("address-space limit") && detail.contains("signal"),
            "a glibc allocation abort must be named the memory cap: {detail:?}"
        );
    }

    /// A signalled exit whose stderr carries no allocation signature must
    /// stay the honest `Signaled` label — same conservative fallback,
    /// signal path this time.
    #[test]
    fn a_signalled_exit_without_an_allocation_signature_stays_signaled() {
        let outcome = run_worker(
            WorkerSpawn {
                program: PathBuf::from("/bin/sh"),
                args: vec![
                    "-c".to_string(),
                    "printf 'segfault, not an allocation failure' 1>&2; kill -SEGV $$".to_string(),
                ],
                input: Vec::new(),
                deadline: Duration::from_secs(5),
            },
            &identity(),
            &deny(),
        );
        let WorkerOutcome::Refused(row) = outcome else {
            panic!("must be refused: {outcome:?}");
        };
        let detail = row.detail.unwrap_or_default();
        assert!(
            !detail.contains("address-space limit"),
            "an unrelated signal must not be named the memory cap: {detail:?}"
        );
        assert!(detail.contains("terminated by signal"), "{detail:?}");
    }
}

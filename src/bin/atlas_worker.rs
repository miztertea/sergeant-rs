//! `sgt-atlas-worker` — the supervised parse-worker binary (S4 Y1, G2; the
//! real Office adapter, Y2, G3).
//!
//! Spawned by [`sergeant_rs::runtime::atlas::worker::run_worker`], never run
//! directly by an operator: reads the resource bytes to extract from stdin,
//! writes a [`WorkerBatch`] as JSON to stdout, and exits zero. Never opens
//! Atlas's store — the daemon is the sole writer, and this process has no
//! path to that database at all, which is what makes "a worker never opens
//! the store" true structurally rather than by convention.
//!
//! # Extraction is chosen by `--extractor`, exactly as it is in-process
//!
//! `--extractor` is dispatched on, not merely echoed: a value equal to
//! [`office::DOCX_EXTRACTOR`] runs the real Office adapter
//! ([`office::docx_units`]) over stdin; anything else falls through to the
//! trivial UTF-8-whole-document body Y1 shipped. This mirrors exactly how
//! `runtime::atlas::scan::extract_units` already dispatches on the same
//! extractor-identity strings in-process (R2) — the wire contract's
//! `extractor` field was always meant to be the dispatch key, once a second
//! real adapter existed to dispatch to.
//!
//! # A failed extraction exits non-zero — it never emits an empty batch
//!
//! [`office::docx_units`] returning `Err` is **not** reported as a
//! `WorkerBatch` with zero units: that would be indistinguishable on the
//! wire from a document that genuinely extracted to nothing, which is
//! exactly the "silent empty" F8 forbids (coverage honesty, brief item 5).
//! Instead this process prints the error and exits non-zero — the existing
//! [`WorkerFault::ExitedNonZero`](sergeant_rs::runtime::atlas::worker)
//! path Y1 already built, reused rather than given a second wire shape (R2):
//! the daemon-side transport already turns any non-zero exit into a named
//! `Coverage::Error` row with the stderr tail attached, which is exactly
//! what a real parser failure needs and nothing a wire-schema change would
//! add.
//!
//! # `--fault`: the test-only fixture modes Y1's acceptance needs
//!
//! Real OS-process behavior — a `SIGABRT`, a process that never exits, a
//! non-zero exit, unbounded allocation — cannot be produced by an in-process
//! fake, because the whole point of Y1's acceptance is proving the
//! **daemon's** supervision of a **real, separate process**. `--fault` is
//! how the test suite asks this binary to misbehave on purpose; it is
//! reachable from any caller (an operator invoking it directly gets the same
//! four modes), which is the honest alternative to a cfg(test)-only code
//! path that would make the fixture behavior invisible to `cargo build`'s
//! own compile check.

use std::io::{Read, Write};
use std::time::Duration;

use clap::Parser;

use sergeant_rs::domain::source::UnitKind;
use sergeant_rs::runtime::atlas::office;
use sergeant_rs::runtime::atlas::worker::{DeclaredChild, WorkerBatch, WorkerUnit};

/// One process-level misbehavior this binary can perform on command, in
/// place of producing a batch — Y1's fault-injection acceptance (brief item
/// 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Fault {
    /// Terminate itself with `SIGABRT` before producing any output.
    Abort,
    /// Never exit on its own — the caller's deadline and kill+reap are what
    /// end this.
    Hang,
    /// Exit immediately with a non-zero status.
    ExitNonzero,
    /// Allocate and touch memory in a growing, paced loop, forever — the
    /// caller's deadline is what ends this, exactly as [`Fault::Hang`], but
    /// exercising the same kill path under memory pressure instead of an
    /// idle sleep.
    Allocate,
}

#[derive(Debug, Parser)]
#[command(
    name = "sgt-atlas-worker",
    about = "sergeant-rs's supervised parse-worker binary — spawned by the daemon, not run by \
             hand"
)]
struct Args {
    /// The generation this job is extracting for — echoed into the batch's
    /// identity untouched.
    #[arg(long)]
    generation: String,
    /// The extractor identity this job was dispatched to run — echoed into
    /// the batch's identity untouched.
    #[arg(long)]
    extractor: String,
    /// Declare a child resource in the returned batch, `NAME=RELATIVE/PATH`.
    /// Repeatable. Test-only: Y1 has no real container adapter that would
    /// populate this on its own, so the acceptance suite uses it to hand the
    /// daemon-side validator both safe and deliberately unsafe/denied
    /// declarations.
    #[arg(long = "declare-child", value_parser = parse_declared_child)]
    declared_children: Vec<DeclaredChild>,
    /// Misbehave instead of producing a batch (test-only fixture mode).
    #[arg(long)]
    fault: Option<Fault>,
}

fn parse_declared_child(raw: &str) -> Result<DeclaredChild, String> {
    let (name, path) = raw
        .split_once('=')
        .ok_or_else(|| format!("{raw:?} is not NAME=PATH"))?;
    Ok(DeclaredChild {
        name: name.to_string(),
        relative_path: path.to_string(),
    })
}

fn main() -> std::process::ExitCode {
    let args = Args::parse();

    if let Some(fault) = args.fault {
        run_fault(fault);
    }

    let mut input = Vec::new();
    if let Err(e) = std::io::stdin().read_to_end(&mut input) {
        eprintln!("sgt-atlas-worker: could not read stdin: {e}");
        return std::process::ExitCode::FAILURE;
    }

    let batch = match normal_batch(&args, &input) {
        Ok(batch) => batch,
        Err(detail) => {
            // Not an empty `WorkerBatch` — see this file's own module doc,
            // "A failed extraction exits non-zero". `resource_hash`/
            // `generation`/`extractor` are never emitted either, because
            // nothing about this outcome is a batch: exiting non-zero here
            // is what turns into `WorkerFault::ExitedNonZero` daemon-side,
            // the same fault class Y1's own fixture modes already exercise.
            eprintln!("sgt-atlas-worker: extraction failed: {detail}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let Ok(json) = serde_json::to_vec(&batch) else {
        eprintln!("sgt-atlas-worker: could not serialize its own batch");
        return std::process::ExitCode::FAILURE;
    };
    if let Err(e) = std::io::stdout().write_all(&json) {
        eprintln!("sgt-atlas-worker: could not write stdout: {e}");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

/// Extraction, dispatched on `--extractor` (this file's own module doc).
/// `resource_hash` is computed here, independently of anything the daemon
/// sent — the whole point of
/// [`sergeant_rs::runtime::atlas::worker::validate_batch`] is that the
/// daemon never simply trusts this value back.
///
/// `Err` names why extraction failed, in one line — this process's `main`
/// prints it to stderr and exits non-zero rather than ever emitting a batch
/// (F8 coverage honesty: no partial units, never a silent empty).
fn normal_batch(args: &Args, input: &[u8]) -> Result<WorkerBatch, String> {
    let resource_hash = blake3::hash(input).to_hex().to_string();
    let units = if args.extractor == office::DOCX_EXTRACTOR {
        office::docx_units(input)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|unit| {
                // The whole-document unit truthfully spans the whole input,
                // in any container format — that much is always recoverable,
                // matching Y1's own whole-resource convention. A Section
                // unit is not byte-recoverable (`office::OfficeUnit`'s own
                // doc): `0`/`0` names "not applicable" honestly rather than
                // a real span, and `coordinate` carries what actually is
                // recoverable for it.
                let (byte_start, byte_end) = match unit.kind {
                    UnitKind::Document => (0, input.len() as u64),
                    UnitKind::Section => (0, 0),
                };
                WorkerUnit {
                    kind: unit.kind,
                    byte_start,
                    byte_end,
                    coordinate: unit.coordinate,
                    text: unit.text,
                }
            })
            .collect()
    } else {
        // Y1's trivial, honestly-labeled fallback body: one
        // [`UnitKind::Document`] unit spanning the whole input when it
        // decodes as UTF-8, no units when it does not.
        match std::str::from_utf8(input) {
            Ok(text) => vec![WorkerUnit {
                kind: UnitKind::Document,
                byte_start: 0,
                byte_end: input.len() as u64,
                coordinate: None,
                text: text.to_string(),
            }],
            Err(_) => Vec::new(),
        }
    };
    Ok(WorkerBatch {
        generation_id: args.generation.clone(),
        resource_hash,
        extractor: args.extractor.clone(),
        units,
        declared_children: args.declared_children.clone(),
    })
}

/// Perform one fixture fault and end the process — every arm ends this
/// process itself, by signal, by exit, or by never returning, which is why
/// this is typed `-> !` rather than merely never being followed by more
/// code at its one call site.
fn run_fault(fault: Fault) -> ! {
    match fault {
        Fault::Abort => {
            // `libc::abort` raises `SIGABRT` against this process — present
            // and behaving identically on every platform this crate builds
            // for (it is ISO C, not a Linux-only `prctl`-shaped mechanism),
            // so this fixture needs no `#[cfg(unix)]` of its own.
            unsafe { libc::abort() }
        }
        Fault::Hang => loop {
            // Un-catchable by design: the caller's `SIGKILL` (via
            // `kill_process_group`, past its deadline) is what ends this,
            // never a signal this process could choose to handle.
            std::thread::sleep(Duration::from_secs(3600));
        },
        Fault::ExitNonzero => {
            eprintln!("sgt-atlas-worker: --fault exit-nonzero, exiting 17 on purpose");
            std::process::exit(17);
        }
        Fault::Allocate => {
            // Paced growth: touches every page so the kernel actually
            // commits it (a `vec![0u8; n]` the process never writes could be
            // satisfied lazily and would not reproduce OOM-shaped pressure),
            // paused between chunks so a short test deadline bounds how much
            // this ever grows before the caller's kill+reap ends it.
            const CHUNK: usize = 8 * 1024 * 1024;
            let mut held: Vec<Vec<u8>> = Vec::new();
            loop {
                let mut chunk = vec![0u8; CHUNK];
                for byte in chunk.iter_mut().step_by(4096) {
                    *byte = 1;
                }
                held.push(chunk);
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

//! `sgt-atlas-worker` — the supervised parse-worker binary (S4 Y1, G2).
//!
//! Spawned by [`sergeant_rs::runtime::atlas::worker::run_worker`], never run
//! directly by an operator: reads the resource bytes to extract from stdin,
//! writes a [`WorkerBatch`] as JSON to stdout, and exits zero. Never opens
//! Atlas's store — the daemon is the sole writer, and this process has no
//! path to that database at all, which is what makes "a worker never opens
//! the store" true structurally rather than by convention.
//!
//! # Y1's body is a seam, not a placeholder to delete
//!
//! No third-party parser exists until Y2's Anydoc spike, so the normal path
//! below is deliberately trivial — one [`UnitKind::Document`] unit for
//! UTF-8 input, none for anything else — while still exercising every real
//! part of the wire contract (identity fields, declared children) that a
//! Y2+ adapter will fill in for real. The CLI surface (bytes on stdin, a
//! [`WorkerBatch`] on stdout, `--generation`/`--extractor` naming the job) is
//! meant to outlive Y1; only the extraction inside `normal_batch` is meant to
//! be replaced.
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

    let batch = normal_batch(&args, &input);
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

/// Y1's trivial, honestly-labeled body: one [`UnitKind::Document`] unit
/// spanning the whole input when it decodes as UTF-8, no units when it does
/// not. `resource_hash` is computed here, independently of anything the
/// daemon sent — the whole point of [`sergeant_rs::runtime::atlas::worker::validate_batch`]
/// is that the daemon never simply trusts this value back.
fn normal_batch(args: &Args, input: &[u8]) -> WorkerBatch {
    let resource_hash = blake3::hash(input).to_hex().to_string();
    let units = match std::str::from_utf8(input) {
        Ok(text) => vec![WorkerUnit {
            kind: UnitKind::Document,
            byte_start: 0,
            byte_end: input.len() as u64,
            text: text.to_string(),
        }],
        Err(_) => Vec::new(),
    };
    WorkerBatch {
        generation_id: args.generation.clone(),
        resource_hash,
        extractor: args.extractor.clone(),
        units,
        declared_children: args.declared_children.clone(),
    }
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

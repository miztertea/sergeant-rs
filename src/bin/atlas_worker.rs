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
//! ([`office::docx_units`]) over stdin; a value equal to
//! [`archive::ZIP_EXTRACTOR`] runs the real bounded-ZIP adapter
//! ([`archive::expand`], S4 Y3) over stdin; a value equal to
//! [`mail::MAIL_EXTRACTOR`] runs the real mail adapter
//! ([`mail::parse_message`], S4 Y4) over stdin; anything else falls through
//! to the trivial UTF-8-whole-document body Y1 shipped. This mirrors exactly
//! how `runtime::atlas::scan::extract_units` already dispatches on the same
//! extractor-identity strings in-process (R2) — the wire contract's
//! `extractor` field was always meant to be the dispatch key, once a second
//! real adapter existed to dispatch to.
//!
//! # A named gap: per-entry ZIP coverage has nowhere to go on the wire yet
//!
//! [`WorkerBatch`] carries `units` and `declared_children` — no field for a
//! [`CoverageRow`](sergeant_rs::domain::source::CoverageRow) at all. When the
//! ZIP adapter refuses the WHOLE archive (a malformed central directory, the
//! entry-count ceiling, or the overlapping/quine defence — all archive-level
//! refusals `archive::expand` reports as a `path: None` row), this process
//! exits non-zero with that row's own detail text, exactly as a malformed
//! Office document does — the existing `WorkerFault::ExitedNonZero` path
//! turns that into a named `Coverage::Error` row daemon-side. But a
//! WELL-FORMED archive that refuses only SOME entries (a symlink, a
//! duplicate name, one oversized entry) still SUCCEEDS as a batch — its
//! admitted children are declared, and the per-entry refusal detail
//! `archive::expand`'s own return value carries (proven exhaustively in
//! `archive.rs`'s own tests) does not reach this wire at all. Named here
//! rather than silently dropped — see `archive.rs`'s own module doc for the
//! same gap stated from the adapter's side. S5 W7 closed a DIFFERENT seam
//! (a child's content, hash and adapter, below) and deliberately did not
//! widen this one: `WorkerBatch` still has no `CoverageRow` field, so the
//! depth-ceiling refusal that stops a deeply nested archive from being opened
//! is visible as a not-opened container child, never as a row saying why.
//!
//! # Children carry their own bytes, and the whole tree is flattened (S5 W7)
//!
//! [`DeclaredChild`] carries the child's content, the worker's own BLAKE3 of
//! it, and the adapter [`scan::child_extractor_for`] derives for its path —
//! the same function the daemon re-derives with, so a fixture child and a
//! real one are composed identically ([`declared_child`]). The whole
//! expansion tree is flattened into ONE list at `/`-joined paths
//! ([`flatten_zip`]/[`flatten_mail`]), because that is what lets the daemon
//! land children without re-entering a container — one depth counter, one
//! whole-tree byte budget, both already spent by the adapter that walked the
//! tree. The daemon hashes what it receives and stores THAT
//! ([`DeclaredChild`]'s own doc for what that does and does not vouch for).
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
use sergeant_rs::runtime::atlas::archive;
use sergeant_rs::runtime::atlas::mail;
use sergeant_rs::runtime::atlas::office;
use sergeant_rs::runtime::atlas::scan;
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
    /// Declare a child resource in the returned batch,
    /// `NAME=RELATIVE/PATH[=CONTENT]` (CONTENT is UTF-8 text, empty when
    /// omitted). Repeatable. Test-only: it exists so the acceptance suite can
    /// hand the daemon-side validator both safe and deliberately
    /// unsafe/denied declarations without needing a container fixture that
    /// produces them. The hash and adapter fields are composed by
    /// [`declared_child`], never taken from the flag, so a fixture child is
    /// always self-consistent with what the daemon re-derives.
    #[arg(long = "declare-child", value_parser = parse_declared_child)]
    declared_children: Vec<DeclaredChild>,
    /// Misbehave instead of producing a batch (test-only fixture mode).
    #[arg(long)]
    fault: Option<Fault>,
}

fn parse_declared_child(raw: &str) -> Result<DeclaredChild, String> {
    let mut parts = raw.splitn(3, '=');
    let name = parts
        .next()
        .ok_or_else(|| format!("{raw:?} is not NAME=PATH[=CONTENT]"))?;
    let path = parts
        .next()
        .ok_or_else(|| format!("{raw:?} is not NAME=PATH[=CONTENT]"))?;
    let content = parts.next().unwrap_or("").as_bytes().to_vec();
    Ok(declared_child(name.to_string(), path.to_string(), content))
}

/// One [`DeclaredChild`], with the two fields the daemon cross-checks rather
/// than trusts filled in the only way that can ever pass: the worker's own
/// BLAKE3 of the bytes it is sending, and the adapter THIS BUILD'S OWN
/// routing table derives for the child's path
/// ([`scan::child_extractor_for`], the same function
/// [`sergeant_rs::runtime::atlas::worker::validate_batch`] re-derives with).
///
/// Composed in one place so no branch below can drift: a child whose hash or
/// adapter disagrees with the daemon's own derivation is a REFUSED BATCH, not
/// a warning, and that refusal should mean "this worker is wrong", never
/// "this worker filled the field differently over here".
fn declared_child(name: String, relative_path: String, content: Vec<u8>) -> DeclaredChild {
    let content_hash = blake3::hash(&content).to_hex().to_string();
    let entry_adapter = scan::child_extractor_for(&relative_path).map(str::to_string);
    DeclaredChild {
        name,
        relative_path,
        content,
        content_hash,
        entry_adapter,
    }
}

/// Flatten one whole ZIP expansion into declared children (S5 W7).
///
/// Every admitted entry at this level, then — for an entry that is itself a
/// container the adapter ALREADY opened — that container's own members, at
/// their [`scan::CHILD_PATH_SEPARATOR`]-joined path beneath it. That
/// separator, not a plain `/`, at EVERY level (S5 W7 F-SF-04): `/` is what
/// separates directory components *inside* one container, so joining nesting
/// levels with it too made `bundle.zip/report.docx` mean either "an entry
/// named that" or "an entry inside an entry", and the daemon could not tell
/// which. `!/` is reserved — `archive.rs` and `mail.rs` refuse an entry name
/// carrying it — so splitting a composed path on it recovers the exact
/// container chain. The recursion this walks is one
/// `archive::expand` already performed under
/// [`archive::MAX_NESTING_DEPTH`] and its whole-tree cumulative byte budget;
/// flattening it adds no depth and admits no byte those bounds did not
/// already admit. That is what keeps ONE depth counter and ONE budget: the
/// daemon lands what this list says and never re-enters a container to look
/// for more.
fn flatten_zip(expansion: &archive::ZipExpansion, prefix: &str, out: &mut Vec<DeclaredChild>) {
    for child in &expansion.children {
        let path = format!("{prefix}{}", child.relative_path);
        out.push(declared_child(
            entry_basename(&child.relative_path),
            path.clone(),
            child.content.clone(),
        ));
        if let Some(nested) = &child.nested {
            flatten_zip(
                nested,
                &format!("{path}{}", scan::CHILD_PATH_SEPARATOR),
                out,
            );
        }
        if let Some(nested) = &child.nested_mail {
            flatten_mail(
                nested,
                &format!("{path}{}", scan::CHILD_PATH_SEPARATOR),
                out,
            );
        }
    }
}

/// Flatten one whole parsed message's attachments into declared children (S5
/// W7) — the mail half of [`flatten_zip`], sharing its reasoning exactly:
/// `mail::parse_message` already walked this tree under the SAME shared
/// depth counter and whole-tree budget (`mail.rs`'s own module doc,
/// "Container recursion is one shared budget").
fn flatten_mail(message: &mail::MailMessage, prefix: &str, out: &mut Vec<DeclaredChild>) {
    for attachment in &message.attachments {
        let path = format!("{prefix}{}", attachment.filename);
        out.push(declared_child(
            attachment.filename.clone(),
            path.clone(),
            attachment.content.clone(),
        ));
        if let Some(nested) = &attachment.nested_message {
            flatten_mail(
                nested,
                &format!("{path}{}", scan::CHILD_PATH_SEPARATOR),
                out,
            );
        }
        if let Some(nested) = &attachment.nested_archive {
            flatten_zip(
                nested,
                &format!("{path}{}", scan::CHILD_PATH_SEPARATOR),
                out,
            );
        }
    }
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
    // `--declare-child` (test-only, module doc) and the real ZIP adapter
    // below are additive, not exclusive: a test can still hand the daemon-
    // side validator synthetic declarations under any extractor, and a real
    // archive's own admitted children are appended to whatever the flag
    // already supplied.
    let mut declared_children = args.declared_children.clone();
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
                    heading_level: unit.heading_level,
                    title: unit.title,
                    text: unit.text,
                }
            })
            .collect()
    } else if args.extractor == archive::ZIP_EXTRACTOR {
        // `parent_key` composes every child's F7 key inside
        // `archive::expand` (`archive.rs`'s own module doc). The DAEMON
        // composes the key it actually stores, from its own hash of the
        // bytes it receives (S5 W7, `scan::land_child`), so the value passed
        // here only has to keep this expansion's internally-computed keys
        // self-consistent; nothing this process emits depends on it.
        let expansion = archive::expand(input, &resource_hash);
        if let Some(refusal) = expansion.coverage.iter().find(|row| row.path.is_none()) {
            // An archive-level refusal (malformed open, the entry-count
            // ceiling, or the overlapping/quine defence) fails this whole
            // worker call — the same "no partial batch" shape a malformed
            // Office document already gets, above.
            return Err(refusal
                .detail
                .clone()
                .unwrap_or_else(|| "archive refused".to_string()));
        }
        flatten_zip(&expansion, "", &mut declared_children);
        // A container's own body carries no text unit of its own — its
        // whole content is in its declared children. An empty `units` here
        // is the honest answer for a ZIP resource, not a placeholder this
        // build forgot to fill (Y1's own "no wire field nothing ever sets"
        // doctrine, restated for the opposite field).
        Vec::new()
    } else if args.extractor == mail::MAIL_EXTRACTOR {
        // `parent_key` composes every attachment's F7 key inside
        // `mail::parse_message` (`mail.rs`'s own module doc) exactly as the
        // ZIP branch above does for `archive::expand` — same reasoning, and
        // the same S5 W7 answer: the daemon composes the key it stores.
        let message = mail::parse_message(input, &resource_hash).map_err(|e| e.to_string())?;
        // Two Document-kind units, not one — the wave's own schema decision
        // (mirrors office.rs's own "no new UnitKind variant" call, `mail.rs`'s
        // module doc): a mail message genuinely has up to two independent
        // bodies (A1 §6.5), and `coordinate` names which. Neither is
        // byte-exact recoverable into the original wire bytes (a decoded
        // Content-Transfer-Encoding is a transform, the same reason an
        // Office section carries a coordinate rather than a byte range), so
        // both get `0`/`0` — the same honest "not applicable" `office.rs`'s
        // own Section units already use.
        let mut mail_units = Vec::new();
        if let Some(text) = message.text_body.clone() {
            mail_units.push(WorkerUnit {
                kind: UnitKind::Document,
                byte_start: 0,
                byte_end: 0,
                coordinate: Some("text-body".to_string()),
                heading_level: None,
                // A1 §6.5 lists `subject` among the fields a mail message
                // must preserve rather than flatten into anonymous prose.
                // A unit's title is where this store keeps that, so both of
                // a message's bodies carry it.
                title: message.subject.clone(),
                text,
            });
        }
        if let Some(html) = message.html_body.clone() {
            mail_units.push(WorkerUnit {
                kind: UnitKind::Document,
                byte_start: 0,
                byte_end: 0,
                coordinate: Some("html-body".to_string()),
                heading_level: None,
                title: message.subject.clone(),
                text: html,
            });
        }
        // The WHOLE tree, flattened (S5 W7) — a grandchild nested inside an
        // attachment's own `nested_message`/`nested_archive` is declared too,
        // at its own `/`-joined path, exactly as the ZIP branch above
        // declares a nested entry's own members.
        flatten_mail(&message, "", &mut declared_children);
        mail_units
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
                heading_level: None,
                title: None,
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
        declared_children,
    })
}

/// The final path component of a ZIP entry's own enclosed path, for
/// [`DeclaredChild::name`] — falls back to the whole path only if it somehow
/// has no final component (unreachable in practice: `archive::expand`
/// already refuses an empty enclosed name before a child is ever produced).
fn entry_basename(relative_path: &str) -> String {
    std::path::Path::new(relative_path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| relative_path.to_string())
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

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
    /// Offset into the original resource bytes. `0` when this extractor
    /// cannot recover a byte-exact position (see `coordinate` below) — a
    /// text/Markdown extractor always can, so this stays the byte offset it
    /// always was for those.
    pub byte_start: u64,
    /// End offset into the original resource bytes, exclusive. Same caveat
    /// as `byte_start`.
    pub byte_end: u64,
    /// A stable, opaque, per-unit address into the *normalized* document
    /// (S4 Y2, A1 §6.3's provenance requirement) — `None` when
    /// `byte_start`/`byte_end` already say where this unit is (every text/
    /// Markdown unit), `Some` when they cannot: an Office container's
    /// original bytes are compressed and unpacked before extraction, so no
    /// byte offset in them corresponds to a position in the extracted unit.
    /// The contract a `Some` value must satisfy — present, unique per
    /// document, deterministic for the same bytes and extractor identity,
    /// and never a write-back claim into the original resource (A1-12,
    /// derived-not-canonical) — is our own, not any one extractor's; see
    /// [`super::office`]'s module doc, "The contract, in our own terms", for
    /// the full statement and for why no particular string shape (e.g. the
    /// Office adapter's own `block:<n>`) is part of it. The mail adapter
    /// (S4 Y4) reuses the same contract for a different asymmetry — two
    /// independent `Document`-kind units per message, distinguished by
    /// `"text-body"`/`"html-body"` — see [`super::mail`]'s own module doc.
    #[serde(default)]
    pub coordinate: Option<String>,
    /// The unit's own text.
    pub text: String,
}

/// The most bytes one declared child may carry across the pipe.
///
/// **Not a second, independently-sized number** (S5 W7, the brief's own
/// requirement): it is [`super::scan::MAX_RESOURCE_BYTES`] itself, the one
/// ceiling this build already states for "the most bytes one resource may
/// be", which [`super::archive::MAX_ENTRY_UNCOMPRESSED_BYTES`] already
/// aliases for the same reason (R2 — reuse the number that exists, do not
/// tune a new one). `tests/w7_container_children.rs`'s
/// `container_children_share_one_depth_counter_and_one_budget_not_a_second_pair`
/// fails if this ever stops being an alias.
pub const MAX_CHILD_CONTENT_BYTES: u64 = super::scan::MAX_RESOURCE_BYTES;

/// One child resource a worker declares out of the bytes it was given (an
/// archive entry, a mail attachment, or a descendant of either).
///
/// Untrusted by construction: [`validate_batch`] is what decides whether
/// `relative_path` may ever reach a filesystem or a store, whether `content`
/// is small enough to keep, whether the bytes that arrived are the bytes the
/// worker said it was sending, and whether the adapter claim is the one this
/// build's own routing table would make. The daemon runs every one of those
/// checks on every declared child before a byte reaches the store.
///
/// # What the child's content hash does and does not vouch for (H15)
///
/// The daemon hashes a top-level resource's bytes *itself*, before the
/// worker runs — [`WorkerIdentity::resource_hash`] is "evidence the worker
/// never chose". A child's bytes are inside a container the daemon does not
/// parse, so that exact property is not available for a child, and pretending
/// otherwise would be the dishonest part. W7 takes the other route (H15
/// option (b), the brief's recommendation, adopted here): **the worker
/// returns the bytes and the daemon hashes what it receives, on receipt,
/// before storing.** [`content_hash`](Self::content_hash) is the *worker's*
/// claim about those bytes; the daemon computes its own BLAKE3 of what
/// actually arrived, refuses the batch when the two disagree
/// ([`BatchRefusal::ChildHashMismatch`]), and stores its own value.
///
/// So the stored identity of a child says: *these are the bytes that reached
/// the store, and this is their hash.* It does **not** say "this is what is
/// really inside that archive" — the daemon never observed the inside of the
/// archive and cannot vouch for a correspondence it never saw. What this
/// route does preserve is the property that actually matters: the daemon
/// still hashes, still validates, still decides, and no ZIP or MIME parser
/// runs outside the supervised worker's own process group (option (a) —
/// re-opening the container daemon-side — would have moved exactly that
/// parsing into the sole writer, which is what PDEATHSIG/RLIMIT_AS/the own
/// process group exist to prevent).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredChild {
    /// The child's own name, exactly as the worker declared it — checked
    /// against F10's deny set independently of `relative_path`, because a
    /// deny pattern like the dotfile rule matches on a path *component*, and
    /// a worker could otherwise bury a denied name one path segment deep
    /// from the coordinate a naive check might use.
    pub name: String,
    /// Where the child lives relative to its parent resource, `/`-separated.
    /// For a descendant (an entry of an archive that was itself an
    /// attachment) this is the whole chain from the parent resource down,
    /// joined by `/` — the worker flattens the tree it already expanded
    /// under one shared depth counter rather than the daemon re-entering a
    /// container to find it (see [`validate_batch`]'s own doc).
    pub relative_path: String,
    /// The child's own bytes, hex on the wire (see [`child_content_hex`] for
    /// why hex and not a new base64 dependency), bounded by
    /// [`MAX_CHILD_CONTENT_BYTES`] **before the decoded buffer is
    /// allocated**.
    #[serde(with = "child_content_hex")]
    pub content: Vec<u8>,
    /// The worker's own BLAKE3 hex of `content` — a claim, cross-checked
    /// against the daemon's own hash of what arrived (this type's own doc,
    /// "What the child's content hash does and does not vouch for"), exactly
    /// as [`WorkerBatch::resource_hash`] is cross-checked for the parent.
    pub content_hash: String,
    /// A1 §6.6's fourth preserved field: the CHILD's own downstream
    /// extractor identity — never the container adapter that unpacked it.
    /// `None` when nothing in this build claims the child's extension, which
    /// is a named coverage gap daemon-side, not silence.
    ///
    /// A claim, like `content_hash`: the daemon re-derives it from the
    /// child's own path through [`super::scan::child_extractor_for`] — the
    /// same routing table a loose file of that name goes through — and
    /// refuses a batch whose claim disagrees
    /// ([`BatchRefusal::ChildAdapterMismatch`]).
    #[serde(default)]
    pub entry_adapter: Option<String>,
}

/// Hex codec for [`DeclaredChild::content`] on the JSON wire.
///
/// **Why hex, and not base64** (R3 then R5, checked in order): the wire is
/// `serde_json`, and `Vec<u8>` serializes there as an array of decimal
/// integers — around 3.4× expansion and a parse cost per byte. Base64 (1.33×)
/// would be smaller, but no crate in this build's own `Cargo.toml` exposes
/// base64 as a direct dependency, and `mail.rs`'s own test helper already set
/// this build's precedent of writing the few lines rather than taking the
/// dependency (`mail.rs`'s `base64_encode`, "no crate in this build's own
/// Cargo.toml exposes base64"). Hex at 2× is worse than base64 and better
/// than the default, needs no dependency decision, and — the property that
/// decided it — decodes with a ceiling checked from the ENCODED length alone,
/// before any decoded buffer exists.
pub mod child_content_hex {
    use serde::{Deserialize, Deserializer, Serializer};

    /// Refuse a declared payload whose ENCODED length already proves it is
    /// over `limit` — computed from the string's own length, before a single
    /// output byte is allocated. Then decode into a buffer sized by the same
    /// bounded length, so the largest allocation this function can be talked
    /// into is `limit` bytes, whatever the worker claimed.
    ///
    /// S4's own O(N²)-before-the-cap bug is the shape being avoided: the cap
    /// comes first, not after the work.
    pub fn decode_bounded(encoded: &str, limit: u64) -> Result<Vec<u8>, String> {
        if !encoded.len().is_multiple_of(2) {
            return Err(format!(
                "declared child content is {} hex characters, which is not a whole number of \
                 bytes",
                encoded.len()
            ));
        }
        let declared_bytes = encoded.len() as u64 / 2;
        if declared_bytes > limit {
            return Err(format!(
                "declared child content is {declared_bytes} bytes, over the {limit}-byte \
                 per-child ceiling; refused before the decoded buffer was allocated"
            ));
        }
        let mut out = Vec::with_capacity(declared_bytes as usize);
        let (pairs, rest) = encoded.as_bytes().as_chunks::<2>();
        debug_assert!(rest.is_empty(), "the even-length check above guarantees it");
        for [high, low] in pairs {
            out.push((hex_nibble(*high)? << 4) | hex_nibble(*low)?);
        }
        Ok(out)
    }

    /// Lower-case hex of `bytes`.
    pub fn encode(bytes: &[u8]) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            out.push(DIGITS[(byte >> 4) as usize] as char);
            out.push(DIGITS[(byte & 0x0f) as usize] as char);
        }
        out
    }

    fn hex_nibble(c: u8) -> Result<u8, String> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err(format!(
                "declared child content contains {:?}, which is not a hex digit",
                c as char
            )),
        }
    }

    /// `serde`'s serialize half.
    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&encode(bytes))
    }

    /// `serde`'s deserialize half — the transport-level ceiling. The
    /// AUTHORITY-level one is [`super::validate_batch`]'s own check, which
    /// runs whether or not a batch arrived through this function.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let encoded = String::deserialize(deserializer)?;
        decode_bounded(&encoded, super::MAX_CHILD_CONTENT_BYTES).map_err(serde::de::Error::custom)
    }
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
    /// A declared child carries more bytes than [`MAX_CHILD_CONTENT_BYTES`]
    /// (S5 W7). The AUTHORITY-level half of the ceiling
    /// [`child_content_hex::deserialize`] already enforces at the transport
    /// level — stated twice on purpose, because `validate_batch` is the
    /// daemon's authority over a batch however that batch was constructed,
    /// and a batch built in-process never passes through the wire decoder at
    /// all.
    ChildTooLarge {
        /// The child's declared name.
        child: String,
        /// How many bytes it carried.
        bytes: u64,
    },
    /// The bytes that arrived for a declared child do not hash to the value
    /// the worker declared for them (S5 W7). The daemon's own hash of what it
    /// received is what would have been stored — this refusal is what stops a
    /// batch whose two halves disagree from being stored at all.
    ChildHashMismatch {
        /// The child's declared name.
        child: String,
        /// What the worker claimed.
        declared: String,
        /// What the daemon computed over the bytes that actually arrived.
        computed: String,
    },
    /// A declared child claims a downstream adapter this build's own routing
    /// table would not choose for that path (S5 W7) — A1 §6.6's `entry
    /// adapter` field, cross-checked rather than trusted.
    ChildAdapterMismatch {
        /// The child's declared name.
        child: String,
        /// What the worker claimed.
        declared: Option<String>,
        /// What [`super::scan::child_extractor_for`] derives from the child's
        /// own path.
        derived: Option<String>,
    },
    /// The batch's declared children, summed, exceed
    /// [`super::archive::MAX_TOTAL_EXPANDED_BYTES`] — the SAME whole-tree
    /// budget a single container's own expansion walk already enforces
    /// (S5 W7 F-IN-01). A worker's own accounting inside one expansion is
    /// not proof that a batch it hands back stays under budget: the daemon
    /// re-derives the cumulative total across `declared_children` itself,
    /// exactly as it re-derives every other per-child property, rather than
    /// trusting the worker's claim for this one property alone.
    BatchTotalTooLarge {
        /// The summed byte length of every declared child in the batch.
        total_bytes: u64,
    },
    /// A declared child's path names a container it came out of that the
    /// batch never declared (S5 W7 F-SF-02).
    ///
    /// A flattened path joins every nesting level with
    /// [`super::scan::CHILD_PATH_SEPARATOR`], so `bundle.zip!/report.docx`
    /// asserts "this came out of the entry `bundle.zip`, which is in this
    /// same batch". The daemon composes each landed child's parent
    /// coordinate — parent path, parent F7 key — from that assertion, so a
    /// path whose named ancestor is absent, or is present but is not a
    /// container, would either dangle or be silently re-parented onto the
    /// root. Refused instead: the ancestor must be declared in this batch
    /// AND must route to a container adapter
    /// ([`super::scan::child_is_container`]).
    OrphanedChildPath {
        /// The child's declared name.
        child: String,
        /// The path it declared.
        path: String,
        /// The ancestor entry its path names but the batch does not declare
        /// as a container.
        missing_container: String,
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
            Self::IdentityMismatch { .. }
            | Self::UnsafeChildPath { .. }
            | Self::ChildTooLarge { .. }
            | Self::ChildHashMismatch { .. }
            | Self::ChildAdapterMismatch { .. }
            | Self::BatchTotalTooLarge { .. }
            | Self::OrphanedChildPath { .. } => Coverage::Error,
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
            Self::ChildTooLarge { child, bytes } => format!(
                "supervised parse worker declared child {child:?} carrying {bytes} bytes, over \
                 the {MAX_CHILD_CONTENT_BYTES}-byte per-child ceiling; refused before it could \
                 reach the store"
            ),
            Self::ChildHashMismatch {
                child,
                declared,
                computed,
            } => format!(
                "supervised parse worker declared child {child:?} with content hash {declared:?}, \
                 but the bytes that arrived hash to {computed:?}; refused rather than stored"
            ),
            Self::ChildAdapterMismatch {
                child,
                declared,
                derived,
            } => format!(
                "supervised parse worker declared child {child:?} claiming adapter {declared:?}, \
                 but this build's own routing table derives {derived:?} for that path; refused \
                 before it could reach the store"
            ),
            Self::BatchTotalTooLarge { total_bytes } => format!(
                "supervised parse worker's batch declared {total_bytes} total bytes across its \
                 children, over the {}-byte MAX_TOTAL_EXPANDED_BYTES whole-tree budget; refused \
                 before any child could reach the store",
                super::archive::MAX_TOTAL_EXPANDED_BYTES
            ),
            Self::OrphanedChildPath {
                child,
                path,
                missing_container,
            } => format!(
                "supervised parse worker declared child {child:?} at {path:?}, whose path names \
                 the container {missing_container:?} it came out of, but the batch declares no \
                 such container child; refused rather than silently re-parented onto the \
                 dispatched resource"
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
/// then the batch's cumulative declared size against the whole-tree budget
/// (S5 W7 F-IN-01), then per child, path safety, then F10 deny-set
/// membership — on every declared child's name AND its path, because a deny
/// pattern can match either — then (S5 W7) the per-child byte ceiling, the
/// child's content hash, its adapter claim, and (S5 W7 F-SF-02) whether the
/// container its own flattened path names is declared in this same batch as a
/// container child.
///
/// Checked in this order so the *first* thing wrong with a batch is what a
/// coverage row names — a batch failing on identity never gets far enough to
/// be told its child paths were also unsafe, which keeps one refusal one
/// reason. The cumulative-size check runs before any per-child work for the
/// same cheap-bound-first reason S4's own O(N²)-before-the-cap bug taught:
/// a batch already over budget is refused without hashing or path-checking
/// a single one of its children. Within a child, the ceiling is checked
/// before the hash for the same reason: the cheap bound comes first, so an
/// oversized payload is never hashed to find out it was oversized.
///
/// # Every declared child is checked before any byte reaches the store
///
/// This function is called by [`run_worker`] on the whole batch, and a single
/// refused child refuses the whole batch — `Err` here means nothing from this
/// batch is landed, not "land the good ones". That is deliberate: a batch
/// containing one child the daemon refuses is a batch whose producer is
/// buggy, compromised, or running against a different deny set, and partial
/// trust in such an answer is exactly what a validator exists to withhold.
///
/// # Why the daemon never re-enters a container to find a grandchild
///
/// A worker flattens the whole expansion tree it already walked — under the
/// ONE depth counter and the ONE whole-tree byte budget
/// [`super::archive::MAX_NESTING_DEPTH`]/
/// [`super::archive::MAX_TOTAL_EXPANDED_BYTES`] already govern, shared with
/// [`super::mail`] — into one flat `declared_children` list. The daemon
/// therefore lands children; it does not recurse into them looking for more.
/// A second, daemon-side recursion would have been a second depth counter and
/// a second budget, which the brief forbids and
/// `tests/w7_container_children.rs`'s
/// `container_children_share_one_depth_counter_and_one_budget_not_a_second_pair`
/// fails on.
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
    // F-IN-01: the per-child ceiling below bounds one entry; it never
    // bounded the batch as a whole. A batch of many individually-in-bounds
    // children can still exceed the SAME whole-tree budget
    // [`super::archive::MAX_TOTAL_EXPANDED_BYTES`] a single container's own
    // expansion walk already enforces — so the daemon re-derives the
    // cumulative total here too, rather than trusting the worker's own
    // accounting for this one property alone. Checked before the loop's
    // per-child hash/adapter work, in the same before-allocation spirit as
    // the per-child ceiling: the cheap sum comes first.
    let mut cumulative_bytes: u64 = 0;
    for declared in &batch.declared_children {
        cumulative_bytes = cumulative_bytes.saturating_add(declared.content.len() as u64);
        if cumulative_bytes > super::archive::MAX_TOTAL_EXPANDED_BYTES {
            return Err(BatchRefusal::BatchTotalTooLarge {
                total_bytes: cumulative_bytes,
            });
        }
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
        if declared.content.len() as u64 > MAX_CHILD_CONTENT_BYTES {
            return Err(BatchRefusal::ChildTooLarge {
                child: declared.name.clone(),
                bytes: declared.content.len() as u64,
            });
        }
        // The daemon's own hash of the bytes that actually arrived — this
        // value, not the worker's claim, is what a landed child is stored
        // under (`DeclaredChild`'s own doc, H15).
        let computed = crate::domain::source::content_hash(&declared.content);
        if computed != declared.content_hash {
            return Err(BatchRefusal::ChildHashMismatch {
                child: declared.name.clone(),
                declared: declared.content_hash.clone(),
                computed,
            });
        }
        let derived = super::scan::child_extractor_for(&declared.relative_path);
        if declared.entry_adapter.as_deref() != derived {
            return Err(BatchRefusal::ChildAdapterMismatch {
                child: declared.name.clone(),
                declared: declared.entry_adapter.clone(),
                derived: derived.map(str::to_string),
            });
        }
        // S5 W7 F-SF-02: a flattened path ASSERTS its own container chain,
        // and the daemon composes each landed child's parent coordinate from
        // that assertion. Checked here, with the batch's other declared
        // children in hand, because that is the only place the assertion can
        // be checked at all: `land_child` sees one child at a time. A path
        // naming a container this batch never declared — or naming one that
        // does not route to a container adapter — is refused rather than
        // re-parented onto the dispatched resource, which is exactly the
        // silent root-resolution F-SF-02 found.
        if let Some((ancestor, _)) = declared
            .relative_path
            .rsplit_once(super::scan::CHILD_PATH_SEPARATOR)
            && !batch.declared_children.iter().any(|other| {
                other.relative_path == ancestor && super::scan::child_is_container(ancestor)
            })
        {
            return Err(BatchRefusal::OrphanedChildPath {
                child: declared.name.clone(),
                path: declared.relative_path.clone(),
                missing_container: ancestor.to_string(),
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
/// daemon's own binary directory is where the real `sgt-atlas-worker`
/// binary's addressable path lives at runtime — resolved from
/// [`std::env::current_exe`]'s own parent directory
/// (`super::lane::worker_runtime`'s S4 Y8 fix; NOT `current_exe()` itself,
/// which is the *daemon's* binary — `sgt`'s own CLI has no bare-flag
/// surface, so re-execing it with `--generation`/`--extractor` is a clap
/// parse error, never a worker run, unlike [`crate::cli`]'s `spawn_daemon`
/// re-exec, which passes a real `sgt` subcommand `sgt`'s own parser
/// accepts) — while an integration test spawns the worker binary Cargo
/// built for it directly (`env!("CARGO_BIN_EXE_sgt-atlas-worker")`) —
/// `current_exe()` inside a test binary answers with the *test binary's
/// own* path, which is not runnable as a worker at all, sibling directory
/// or not.
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

/// The two invariants one whole scan's worth of worker dispatch shares —
/// where the binary lives, and how long a single call may run — threaded
/// down to whichever walk ([`super::scan::Walk`], [`super::git::extract_blobs`])
/// actually claims a resource for a supervised adapter (S4 Y8).
///
/// A scan claims Office/ZIP/mail resources one at a time, deep inside a
/// pure, engine-agnostic walk that (by design, module doc) knows nothing
/// about an [`crate::runtime::engine::Engine`] or a daemon's own binary
/// path — so this is resolved once, by whichever caller *does* know it
/// ([`super::lane::scan_local_knowledge_on_lane`]/
/// [`super::lane::scan_estate_git_on_lane`] in production; a test spawning
/// the real `sgt-atlas-worker` Cargo built), and passed down as one small
/// value rather than re-derived per resource.
#[derive(Debug, Clone)]
pub struct WorkerRuntime {
    /// The worker binary every claimed resource in this scan dispatches to
    /// — see [`WorkerSpawn::program`]'s own doc for why this is a plain
    /// `PathBuf` rather than [`std::env::current_exe`] baked in here.
    pub program: PathBuf,
    /// [`WorkerSpawn::deadline`] for every call this scan makes.
    pub deadline: Duration,
}

/// **PROVISIONAL — declared, not measured**, the same honesty
/// [`WORKER_ADDRESS_SPACE_LIMIT_BYTES`]'s own doc states for its number: S4
/// Y8 is this transport's first production caller, so there is no real
/// scanned corpus to time it against yet. Generous enough that an ordinary
/// `.docx`/`.zip`/`.eml` well under [`super::scan::MAX_RESOURCE_BYTES`]
/// finishes with room to spare; must be re-derived against a real corpus
/// once one exists, and this comment updated to cite that measurement when
/// it does.
pub const WORKER_RUNTIME_DEADLINE: Duration = Duration::from_secs(30);

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
/// The intelligence-lane permit is not acquired here: a real scan dispatch
/// ([`super::scan::dispatch_worker_resource`], called from
/// [`super::lane::scan_local_knowledge_on_lane`]/
/// [`super::lane::scan_estate_git_on_lane`]) calls this function directly,
/// already inside the ONE whole-scan permit those two entry points acquire
/// (S4 Y8); [`super::lane::run_worker_on_lane`] wraps this function in its
/// own per-call [`crate::runtime::engine::Engine::run_intelligence`] permit
/// instead, the same way [`super::lane::scan_estate_git_on_lane`] wraps
/// [`super::git::scan_estate_git`], but stays test-only — see its own doc.
/// Either shape leaves this function itself engine-agnostic, independently
/// testable without a daemon.
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

    /// One declared child carrying `content`, with the two cross-checked
    /// fields filled the only way that can pass — the same composition
    /// `sgt-atlas-worker`'s own `declared_child` performs, so a test that
    /// wants a REFUSAL has to go out of its way to make one, rather than
    /// getting one by forgetting a field.
    fn child(name: &str, relative_path: &str, content: &[u8]) -> DeclaredChild {
        DeclaredChild {
            name: name.to_string(),
            relative_path: relative_path.to_string(),
            content: content.to_vec(),
            content_hash: crate::domain::source::content_hash(content),
            entry_adapter: super::super::scan::child_extractor_for(relative_path)
                .map(str::to_string),
        }
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
        with_child
            .declared_children
            .push(child(".env", ".env", b"SECRET=1"));
        let err = validate_batch(&identity(), &with_child, &deny()).expect_err("must refuse");
        assert!(matches!(err, BatchRefusal::DeniedChildName { .. }));
    }

    /// The brief's other example: `../../etc/passwd` is a path-safety
    /// refusal, independent of whether the deny set would also catch it.
    #[test]
    fn a_declared_child_at_a_traversal_path_is_refused_by_path_safety() {
        let mut with_child = batch();
        with_child.declared_children.push(child(
            "innocuous.txt",
            "../../etc/passwd",
            b"root:x:0:0",
        ));
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
        with_child
            .declared_children
            .push(child("id_rsa", "assets/id_rsa", b"-----BEGIN"));
        let err = validate_batch(&identity(), &with_child, &deny()).expect_err("must refuse");
        assert!(matches!(err, BatchRefusal::DeniedChildName { .. }));
    }

    #[test]
    fn an_ordinary_nested_child_path_is_enclosed_and_allowed() {
        let mut with_child = batch();
        with_child
            .declared_children
            .push(child("logo.png", "assets/images/logo.png", b"\x89PNG"));
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

    /// A review finding: this AUTHORITY recheck is the daemon-side backstop
    /// against a worker-declared child path/name — `Path::new("C:")
    /// .components().count() == 1` on the Unix host this build runs on, so a
    /// Windows drive-letter-absolute path was, before the fix in
    /// [`crate::domain::is_plain_name`], admitted component-by-component as
    /// "plain" despite this module's own doc claiming `enclosed_name`
    /// semantics ("a relative path with no absolute component").
    #[test]
    fn enclosed_relative_path_rejects_a_windows_drive_letter_absolute_path() {
        for unsafe_path in ["C:/Windows/System32/evil.zip", "c:/evil.txt", "D:/x"] {
            assert!(
                !enclosed_relative_path(unsafe_path),
                "{unsafe_path:?} must not be enclosed-safe"
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

    /// **S4 Y7 closeout, boundary audit.** "A worker never opens the store"
    /// was asserted only in `src/bin/atlas_worker.rs`'s own module doc
    /// ("Never opens Atlas's store ... which is what makes ... true
    /// structurally rather than by convention") — a comment claiming a
    /// structural fact, not a test proving one. Item 12's own check
    /// (`x5_a1a_acceptance::a1a_item_12_no_atlas_write_path_is_reachable_from_the_cli`)
    /// pins the CLI-never-writes half of the daemon-sole-writer boundary;
    /// nothing pinned this sibling claim about the worker process. This is
    /// that pin, in the same token-scan style the one-owner DuckDB-crate
    /// tests use (`x1_atlas_substrate::atlas_database_has_exactly_one_owner`,
    /// `m5_projections::t2_the_duckdb_file_has_exactly_one_owner`) — watched
    /// red by hand before landing (a temporary `AtlasDb` token inserted into
    /// each file in turn failed this assertion, then was reverted) rather
    /// than assumed to work from the shape alone.
    #[test]
    fn a_worker_never_names_the_atlas_store() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // Needle spelled from parts so this test's own source does not trip
        // its own check the way scanning `worker.rs` in full — the file
        // this very test lives in — would (the same self-reference problem
        // `x5_a1a_acceptance.rs`'s production-caller check solves by cutting
        // its scan off at `\nmod tests {`).
        let needles = ["Atlas", "Db"].concat();

        // S4 Y8 panel fix (a): `src/bin/` is walked recursively rather than
        // one hardcoded `atlas_worker.rs` path, the same "one owner, checked
        // against everything in the tree" shape
        // `x1_atlas_substrate::atlas_database_has_exactly_one_owner` already
        // uses — a second worker binary, or `atlas_worker.rs` splitting into
        // a directory, stays covered without this test needing a second
        // edit. The `any(...)` assertion is the coverage guard: a typo'd
        // root that silently matched zero files would make the loop below
        // vacuously pass.
        let bin_files = rust_sources(&root.join("src/bin"));
        assert!(
            bin_files
                .iter()
                .any(|f| f.file_name().is_some_and(|n| n == "atlas_worker.rs")),
            "the recursive walk of src/bin/ must actually find atlas_worker.rs, or this check \
             proves nothing: {bin_files:?}"
        );
        let mut scanned: Vec<(String, String)> = bin_files
            .into_iter()
            .map(|path| {
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
                (
                    path.strip_prefix(&root)
                        .unwrap_or(&path)
                        .display()
                        .to_string(),
                    text,
                )
            })
            .collect();
        scanned.push((
            "src/runtime/atlas/worker.rs (production code only)".to_string(),
            std::fs::read_to_string(root.join("src/runtime/atlas/worker.rs"))
                .map(|whole| {
                    let cut = whole
                        .find("\n#[cfg(test)]\nmod tests {")
                        .unwrap_or(whole.len());
                    whole[..cut].to_string()
                })
                .unwrap_or_else(|e| panic!("read src/runtime/atlas/worker.rs: {e}")),
        ));

        for (relative, text) in scanned {
            assert!(
                !text.contains(&needles),
                "{relative} names the Atlas store type — a worker process must have no path to \
                 Atlas's store at all; the daemon is the sole writer AND the sole opener"
            );
            assert!(
                !text.contains("atlas::db::"),
                "{relative} names the store module path directly"
            );
        }
    }

    /// Every `.rs` file under `dir`, recursively — the same shape
    /// `tests/x1_atlas_substrate.rs`'s own `rust_sources` helper uses (R2),
    /// duplicated rather than shared because that one lives in a separate
    /// integration-test binary this `src/`-embedded unit test cannot import.
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

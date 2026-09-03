//! S4 Y1 acceptance: the supervised parse-worker transport (G2).
//!
//! **Proves the SUPERVISION CONTRACT, not a parser** (panel finding — no
//! third-party parser exists until Y2's Anydoc spike, so there is nothing
//! real to poison yet). Two halves:
//!
//! * A real worker binary ([`SGT_ATLAS_WORKER`]) returning a batch is
//!   validated daemon-side as the AUTHORITY (identity, `enclosed_name` path
//!   safety, F10 deny-set membership on declared child names) — proven
//!   through the real supervised process, not only through
//!   `runtime::atlas::worker`'s own pure-function unit tests.
//! * Four process-level faults (abort, hang past deadline, non-zero exit,
//!   allocate-until-killed) each leave the daemon (here: the [`Engine`])
//!   up, the intelligence-lane permit freed, no partial Atlas rows, and a
//!   named coverage row describing the failure — the acceptance
//!   [`a_fault_worker_leaves_the_daemon_up_the_permit_freed_and_a_named_coverage_row`]
//!   walks all four.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::domain::source::Coverage;
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::deny::AcquisitionFilter;
use sergeant_rs::runtime::atlas::lane::run_worker_on_lane;
use sergeant_rs::runtime::atlas::worker::{
    WORKER_ADDRESS_SPACE_LIMIT_BYTES, WorkerIdentity, WorkerOutcome, WorkerSpawn, run_worker,
};
use sergeant_rs::runtime::engine::Engine;

mod support;

/// The real worker binary Cargo built alongside this test binary (`sgt` is
/// the other one, addressed the same way elsewhere in this suite).
const SGT_ATLAS_WORKER: &str = env!("CARGO_BIN_EXE_sgt-atlas-worker");

/// Generous enough that a healthy worker (spawn, run, exit) never trips it
/// under either of the two-environment rule's hosts, short enough that the
/// hang/allocate faults below do not slow the suite.
const NORMAL_DEADLINE: Duration = Duration::from_secs(10);
/// Deliberately short: this bounds how long the hang/allocate fault cases
/// keep their worker alive before supervision kills it.
const FAULT_DEADLINE: Duration = Duration::from_millis(400);
/// Deliberately generous, and used by exactly one test
/// ([`an_allocating_worker_is_killed_by_its_address_space_cap_not_the_deadline`]):
/// the `--fault allocate` mode grows by 8 MiB every 20ms
/// (`src/bin/atlas_worker.rs`'s own `Fault::Allocate` doc), so it reaches
/// [`WORKER_ADDRESS_SPACE_LIMIT_BYTES`] (512 MiB) in roughly 1.3s regardless
/// of host memory pressure — `RLIMIT_AS` bounds virtual address space, not
/// resident memory, so this is deterministic on any host. This deadline is
/// wide enough that the address-space cap always wins the race, on purpose:
/// a deadline short enough to also plausibly fire (like [`FAULT_DEADLINE`]
/// above) would leave the test unable to tell "killed by the cap" apart from
/// "killed by the clock".
const MEMORY_CAP_TEST_DEADLINE: Duration = Duration::from_secs(8);

fn deny() -> AcquisitionFilter {
    AcquisitionFilter::new(&[]).expect("compile default deny set")
}

fn identity_for(input: &[u8]) -> WorkerIdentity {
    WorkerIdentity {
        generation_id: "gen-y1".to_string(),
        resource_hash: blake3::hash(input).to_hex().to_string(),
        extractor: "fixture/v1".to_string(),
    }
}

fn spawn(args: Vec<String>, input: Vec<u8>, deadline: Duration) -> WorkerSpawn {
    WorkerSpawn {
        program: PathBuf::from(SGT_ATLAS_WORKER),
        args,
        input,
        deadline,
    }
}

// --------------------------------------------------------------- the happy path

/// Bytes in, a normalized batch out, through a real subprocess — round-trips
/// content through the real worker binary rather than only through
/// `worker.rs`'s in-memory unit tests.
#[test]
fn a_worker_process_returns_units_that_pass_daemon_side_validation() {
    let input = b"hello atlas".to_vec();
    let identity = identity_for(&input);
    let outcome = run_worker(
        spawn(
            vec![
                "--generation".to_string(),
                identity.generation_id.clone(),
                "--extractor".to_string(),
                identity.extractor.clone(),
            ],
            input.clone(),
            NORMAL_DEADLINE,
        ),
        &identity,
        &deny(),
    );
    let WorkerOutcome::Accepted(batch) = outcome else {
        panic!("a well-formed worker run must be accepted: {outcome:?}");
    };
    assert_eq!(batch.generation_id, identity.generation_id);
    assert_eq!(batch.resource_hash, identity.resource_hash);
    assert_eq!(batch.units.len(), 1, "one Document unit for UTF-8 input");
    assert_eq!(batch.units[0].text, "hello atlas");
}

/// The identity the daemon expects is composed from what the daemon itself
/// knows — never from anything the worker said — so a worker that ran
/// against the *wrong* bytes (or a stale job) is refused even though it
/// exited cleanly and produced well-formed JSON.
#[test]
fn a_batch_computed_over_the_wrong_bytes_is_refused_by_identity_not_trusted() {
    let true_input = b"the real bytes".to_vec();
    // The daemon's own expectation is built over *different* bytes than what
    // actually got sent — simulating a worker that (bug, or a compromised
    // process) computed its hash over something else.
    let identity = identity_for(b"not what was actually sent");
    let outcome = run_worker(
        spawn(
            vec![
                "--generation".to_string(),
                identity.generation_id.clone(),
                "--extractor".to_string(),
                identity.extractor.clone(),
            ],
            true_input,
            NORMAL_DEADLINE,
        ),
        &identity,
        &deny(),
    );
    let WorkerOutcome::Refused(row) = outcome else {
        panic!("an identity mismatch must be refused: {outcome:?}");
    };
    assert_eq!(row.status, Coverage::Error);
    assert!(
        row.detail
            .as_deref()
            .unwrap_or_default()
            .contains("resource_hash"),
        "{row:?}"
    );
}

/// The brief's own example, through the real subprocess: a worker declaring
/// a child named `.env` is refused daemon-side, with a named coverage row —
/// worker-side, nothing stopped it; the daemon is the authority.
#[test]
fn a_real_worker_declaring_a_denied_child_name_is_refused_with_a_named_row() {
    let input = b"body".to_vec();
    let identity = identity_for(&input);
    let outcome = run_worker(
        spawn(
            vec![
                "--generation".to_string(),
                identity.generation_id.clone(),
                "--extractor".to_string(),
                identity.extractor.clone(),
                "--declare-child".to_string(),
                ".env=.env".to_string(),
            ],
            input,
            NORMAL_DEADLINE,
        ),
        &identity,
        &deny(),
    );
    let WorkerOutcome::Refused(row) = outcome else {
        panic!("a denied child name must be refused: {outcome:?}");
    };
    assert_eq!(row.status, Coverage::Excluded);
    assert!(
        row.detail.as_deref().unwrap_or_default().contains(".env"),
        "{row:?}"
    );
}

/// The brief's other example: a declared child at a traversal path is
/// refused by path safety, through the real subprocess.
#[test]
fn a_real_worker_declaring_a_traversal_child_path_is_refused_with_a_named_row() {
    let input = b"body".to_vec();
    let identity = identity_for(&input);
    let outcome = run_worker(
        spawn(
            vec![
                "--generation".to_string(),
                identity.generation_id.clone(),
                "--extractor".to_string(),
                identity.extractor.clone(),
                "--declare-child".to_string(),
                "evil=../../etc/passwd".to_string(),
            ],
            input,
            NORMAL_DEADLINE,
        ),
        &identity,
        &deny(),
    );
    let WorkerOutcome::Refused(row) = outcome else {
        panic!("a traversal child path must be refused: {outcome:?}");
    };
    assert_eq!(row.status, Coverage::Error);
    assert!(
        row.detail
            .as_deref()
            .unwrap_or_default()
            .contains("../../etc/passwd"),
        "{row:?}"
    );
}

// ------------------------------------------------------- the supervision contract

/// One fault-injection case: the `--fault` mode this binary is asked to
/// perform, and a substring the resulting coverage row's detail must name.
struct FaultCase {
    fault: &'static str,
    names: &'static str,
}

const FAULT_CASES: &[FaultCase] = &[
    FaultCase {
        fault: "abort",
        names: "signal",
    },
    FaultCase {
        fault: "hang",
        names: "deadline",
    },
    FaultCase {
        fault: "exit-nonzero",
        names: "status",
    },
    // With FAULT_DEADLINE this short (400ms), the allocate fault reaches
    // only ~160 MiB before the clock fires — well under
    // WORKER_ADDRESS_SPACE_LIMIT_BYTES (512 MiB) — so this case is still the
    // HANG-guard proof, same as it was before the memory cap existed. The
    // memory-guard proof, with a deadline wide enough for the cap to win
    // instead, is `an_allocating_worker_is_killed_by_its_address_space_cap_not_the_deadline`
    // below.
    FaultCase {
        fault: "allocate",
        names: "deadline",
    },
];

/// **The wave's own acceptance**: for each of the four fault-injection
/// modes, the daemon (here, the [`Engine`] running the worker under a real
/// intelligence-lane permit) stays up, the permit is freed, no partial Atlas
/// rows appear, and a named coverage row lands describing the failure.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_fault_worker_leaves_the_daemon_up_the_permit_freed_and_a_named_coverage_row() {
    // The outer bound the whole test is wrapped in. If supervision's
    // deadline enforcement were broken (a hung worker never actually
    // killed), the per-case FAULT_DEADLINE above would never fire and this
    // is what would catch it instead of the test hanging forever — wave
    // `timeout-the-function` (S6, item 2): raised from a bespoke, locally-
    // named 20s onto the crate's one shared hang-only bound.
    tokio::time::timeout(support::HANG_BUDGET, async {
        for case in FAULT_CASES {
            let data = TempDir::new().expect("tempdir");
            let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path())
                .with_intelligence_lane_cap(1);

            // "No partial rows are written": Atlas has no writer for a
            // worker's batch until a later wave wires `record`'s three-step
            // discipline onto it, and this transport never opens the store
            // at all (`AtlasDb` is not even reachable from `worker.rs`) — so
            // the decisive, checkable form of the claim today is that an
            // independent Atlas store is untouched by the call, before and
            // after.
            let atlas = AtlasDb::open_in_memory().expect("in-memory atlas");
            assert!(atlas.indexed_sources().expect("read").is_empty());

            let input = b"irrelevant for a fault run".to_vec();
            let identity = identity_for(&input);
            let outcome = run_worker_on_lane(
                &engine,
                spawn(
                    vec![
                        "--generation".to_string(),
                        identity.generation_id.clone(),
                        "--extractor".to_string(),
                        identity.extractor.clone(),
                        "--fault".to_string(),
                        case.fault.to_string(),
                    ],
                    input,
                    FAULT_DEADLINE,
                ),
                identity,
                deny(),
            )
            .await
            .unwrap_or_else(|e| panic!("[{}] the lane call itself must not fail: {e}", case.fault));

            let WorkerOutcome::Refused(row) = outcome else {
                panic!(
                    "[{}] a fault must be refused, not accepted: {outcome:?}",
                    case.fault
                );
            };
            assert_eq!(
                row.status,
                Coverage::Error,
                "[{}] a process-level fault is Coverage::Error: {row:?}",
                case.fault
            );
            let detail = row.detail.clone().unwrap_or_default();
            assert!(
                detail.contains(case.names),
                "[{}] coverage detail {detail:?} must name {:?}",
                case.fault,
                case.names
            );

            assert_eq!(
                engine.intelligence_lane.available_permits(),
                1,
                "[{}] the intelligence-lane permit must be freed",
                case.fault
            );

            // "The daemon stays up": the engine that just supervised a
            // killed/aborted/non-zero child still runs an ordinary job.
            let still_alive: usize = engine.run_intelligence(|| 7).await.unwrap_or_else(|e| {
                panic!("[{}] the engine must still be usable: {e}", case.fault)
            });
            assert_eq!(still_alive, 7);

            assert!(
                atlas.indexed_sources().expect("read").is_empty(),
                "[{}] no partial rows may appear from a faulted worker",
                case.fault
            );
        }
    })
    .await
    .expect("the whole fault-injection walk must finish well inside its outer bound");
}

/// **The memory-fault class the deadline alone left open (independent
/// review finding, refuter-confirmed, folded into the plan as G2's
/// amendment):** an allocation blowup must be killed by the address-space
/// cap itself, deterministically, and distinguishably from the deadline —
/// not merely "eventually killed by something". A deadline generous enough
/// that the cap has to fire first ([`MEMORY_CAP_TEST_DEADLINE`]), plus an
/// elapsed-time assertion showing the kill landed well before that deadline
/// could have fired, is what makes this a proof of the memory guard rather
/// than a second proof of the hang guard already covered above.
#[test]
fn an_allocating_worker_is_killed_by_its_address_space_cap_not_the_deadline() {
    let input = b"irrelevant for a memory-fault run".to_vec();
    let identity = identity_for(&input);

    let started = std::time::Instant::now();
    let outcome = run_worker(
        spawn(
            vec![
                "--generation".to_string(),
                identity.generation_id.clone(),
                "--extractor".to_string(),
                identity.extractor.clone(),
                "--fault".to_string(),
                "allocate".to_string(),
            ],
            input,
            MEMORY_CAP_TEST_DEADLINE,
        ),
        &identity,
        &deny(),
    );
    let elapsed = started.elapsed();

    let WorkerOutcome::Refused(row) = outcome else {
        panic!("an allocation past the address-space cap must be refused: {outcome:?}");
    };
    assert_eq!(row.status, Coverage::Error, "{row:?}");
    let detail = row.detail.clone().unwrap_or_default();

    // Distinguishes cap-kill from deadline-kill by what the row actually
    // says, not by inference: `WorkerFault::TimedOut`'s own detail names
    // "exceeded its deadline" and nothing else does.
    assert!(
        !detail.contains("exceeded its deadline"),
        "must be killed by the address-space cap, not the wall-clock deadline: {detail:?}"
    );
    assert!(
        detail.contains(&WORKER_ADDRESS_SPACE_LIMIT_BYTES.to_string())
            && detail.contains("address-space"),
        "coverage detail must name the address-space limit that killed it: {detail:?}"
    );

    // Distinguishes cap-kill from deadline-kill by *when* it happened, not
    // only by what the row says: a deadline-kill cannot possibly return
    // before MEMORY_CAP_TEST_DEADLINE has fully elapsed, so landing clearly
    // *before* the deadline — with a fixed, modest safety margin, not a
    // fraction of the budget — is proof the cap, not the clock, ended the
    // child. That is the actual, hardware-independent discriminator; "under
    // half the deadline" is not, and did flake on a CI runner that simply
    // allocates the fixture's 8 MiB steps more slowly than this workstation
    // (killed at 4.324s against a 4s budget, still comfortably under the 8s
    // deadline). A fixed margin below the deadline itself stays valid on any
    // host slow enough to still finish before the deadline at all — which it
    // must, since the whole test's premise is that the cap fires before the
    // clock does.
    const CAP_KILL_SAFETY_MARGIN: Duration = Duration::from_secs(1);
    assert!(
        elapsed + CAP_KILL_SAFETY_MARGIN < MEMORY_CAP_TEST_DEADLINE,
        "the address-space cap must fire before the deadline budget, not race it: killed after \
         {elapsed:?}, deadline was {MEMORY_CAP_TEST_DEADLINE:?} (required margin: \
         {CAP_KILL_SAFETY_MARGIN:?})"
    );
}

/// No `sgt`-shaped, `opencode`, or `codex` children may survive the suite
/// (#310's four patterns) — this wave adds a fifth species, and it gets the
/// same discipline: nothing named `sgt-atlas-worker` may still be running
/// once every case above (including the two hang/allocate kills) has
/// returned.
#[test]
fn no_worker_process_survives_the_fault_injection_walk() {
    // A fresh probe, run after the async test above has already executed in
    // this same binary (integration test binaries run all `#[test]`s in one
    // process) is not orderable against it, so this asserts the one thing
    // that is always true regardless of order: right now, nothing named
    // `sgt-atlas-worker` is alive. Run last alphabetically is not guaranteed
    // either, so the real proof is inside the fault walk's own per-case
    // deadline enforcement above (kill + reap, synchronously, before
    // `run_worker` returns) — this is a coarse whole-binary backstop, not
    // the decisive check.
    // Polled, not a single sample: `child.wait()` reaping inside this same
    // process and a system-wide `pgrep` snapshot observing that reap are two
    // different vantage points, and a process signalled microseconds ago can
    // still show up in one `/proc` scan — the same reason `child.rs`'s own
    // `wait_until_gone` polls rather than checking once.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let output = std::process::Command::new("pgrep")
            .arg("-f")
            .arg("sgt-atlas-worker")
            .output();
        let Ok(output) = output else {
            // `pgrep` missing entirely is a probe-environment gap, not a
            // failure — the decisive per-case kill+reap assertion above is
            // what this suite actually relies on.
            return;
        };
        let listing = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if listing.is_empty() {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!("an sgt-atlas-worker process is still alive after the grace period: {listing}");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

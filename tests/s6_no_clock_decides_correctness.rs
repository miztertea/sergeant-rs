//! **The owner's ruling** (2026-09-02,
//! `knowledge/rulings/owner-rulings/tests-must-be-deterministic-2026-09-02.md`
//! in the workspace): "Tests must be deterministic. Time is non
//! deterministic, it's an arbitrary duration." A test's verdict may depend
//! only on observed state. A sleep followed by an assertion is a defect at
//! any length — lengthening it moves the failure to a slower machine, it
//! does not remove it. Waiting happens on state, never on time: the test
//! observes the thing it is waiting for and proceeds when it sees it. A
//! wait budget is a termination bound, not a verdict — the only reason a
//! bounded wait exists is so a hung test ends instead of holding CI to its
//! cap; exhausting it reports "state X was never observed within the
//! bound," never a silent pass and never a claim that the code was slow.
//! This file is the structural guard for that ruling, over both halves of
//! the crate's own Rust: `src/`, where the class first recurred, and
//! `tests/`, which the ruling names explicitly as the gap that let it
//! reach a release (item 4 below).
//!
//! **`src/`: the scan-follow-retry guard.** Grown out of the AMENDMENT
//! (owner, 2026-08-30, `knowledge/evidence/resources/host-atlas-s6-series/
//! brief-scan-front-door.md`) — "a prose prohibition is what let this class
//! recur. Pin it: a structural test that fails when the rule is violated."
//! "Elapsed time may never decide whether something is correct...
//! `POLL_FAILURES_TOLERATED = 5` — a guessed number standing in for 'the
//! daemon is gone'. Ask the real question instead: is the daemon still
//! there? ... Distinguish the two by state, not by a count." Cadence
//! (`SCAN_POLL`) and reporting a duration are explicitly permitted; a count
//! or deadline that a caller's own success/failure branches on is not. Per
//! the AMENDMENT's own text (`brief-scan-front-door.md:130`, "every
//! `Instant`/`deadline`/`elapsed` construct in the crate sits on an
//! explicit allowlist") and its closing instruction
//! (`brief-scan-front-door.md:145-147`, "this covers *this* crate's Rust;
//! it does not reach the shell scripts under `scripts/`"): this half walks
//! the **production code** (everything before each file's own
//! `#[cfg(test)]` test-module boundary — this crate's own convention, see
//! e.g. `src/cli.rs:7553`) of every `.rs` file under `src/` — the whole
//! crate's Rust, not a named subset of it. `scripts/` is not Rust and is
//! outside this walk for exactly the reason the AMENDMENT names, not a
//! second, invented carve-out.
//!
//! **Every real construct this walk finds sits on `ALLOWLIST` below, each
//! with its own reviewed category and reason** — there is no file-level
//! exemption for any production `.rs` file, including the ones that bound
//! a process or resource this code directly owns and must reclaim (a
//! spawned child killed past its own budget in
//! `src/runtime/atlas/worker.rs`, whose own module doc frames its deadline
//! as a "HANG guard"; a filesystem lock's acquisition timeout in
//! `src/runtime/repolock.rs`/`src/runtime/fsutil.rs`; a supervised git
//! subprocess in `src/runtime/git.rs`; an interactive backend's turn
//! timeout in `src/backend/agy.rs`). That class is real and distinct from
//! the defect this wave fixed — see each entry's own `owned-wait-budget`
//! reason — but it is not exempted from the guard the way an earlier
//! version of this file exempted it by file name; it is allowlisted like
//! everything else, on its own reviewed merits.
//!
//! **`tests/`: the sleep guard (wave `brief-sleep-and-hope.md`,
//! 2026-09-02).** The `src/` guard above walked only production code; the
//! test suite's own sleeps were the actual gap, and the release failure
//! that proved it is named in the ruling: release run 33591670053, Gate D,
//! `tests/c2_light/e_periodic_sweep.rs`'s 200 ms sleep, which slept then
//! asserted a background tick had fired — passing on a fast dry-run
//! runner an hour earlier and failing on CI's slower one. `all_test_files`
//! walks every `.rs` file under `tests/` recursively, excluding
//! `tests/fixtures/` — data corpora this suite reads as *input* (including
//! deliberately malformed Rust, e.g. a corpus gate's own red-side fixture),
//! never test code for this guard to itself parse and classify. Unlike
//! `src/`, an integration-test file carries no `#[cfg(test)]` boundary, so
//! the whole file is in scope.
//!
//! A `sleep(` call is legitimate only inside a loop whose body actually
//! checks observed state and terminates on it (`loop_body_checks_state`):
//! a `while` condition that is not a bare `true`, or — for `loop`, `for`,
//! and a `while true` — a `break`/`continue`/`return`/panic reached only
//! through a conditional inside the body. Lexical nesting alone is not a
//! state check: a fixed-count `for _ in 0..N { sleep(...) }` with no
//! conditional exit is a fixed-count busy-wait, indistinguishable in
//! effect from the forbidden `POLL_FAILURES_TOLERATED` shape, and the
//! guard flags it exactly as it would a bare sleep outside any loop.
//!
//! Every sleep that fails that test and is not converted sits on
//! `ALLOWLIST` (the same `Allowed` shape and `allowlist_covers` check the
//! `src/` half already uses, R2). Two shapes recur in the `tests/` section
//! of `ALLOWLIST`, named per entry rather than once: (a) a fixed
//! observation window proving an **absence** — nothing happened — which
//! has no positive state to poll for because the assertion itself is
//! "still nothing by now"; (b) pacing one action against another, or
//! against a subprocess that exposes no readiness signal, where the delay
//! spaces two things rather than deciding an outcome. Say plainly: shape
//! (b) is the *same non-determinism* as the defect this wave fixed — a
//! duration standing in for a signal that was never actually observed —
//! and it is tolerated only as an enumerated, reasoned residue, never a
//! sanctioned pattern. Each such site is a product finding (a missing
//! readiness signal on the thing being paced against) awaiting its own
//! fix, not a closed matter. The one conversion target for a genuine wait
//! on state is `tests/support/mod.rs::wait_until`/`wait_until_sync`;
//! reach for it before reaching for the allowlist.

use std::path::{Path, PathBuf};

use syn::spanned::Spanned as _;
use syn::visit::{self, Visit};

/// Every `.rs` file under `src/`, walked the same way
/// `tests/a1_floor_awareness.rs::all_src_files` already does (R2 — reuse,
/// not reinvent) — this guard's actual file scope, rather than a hand-kept
/// list that silently drifts as the crate grows new modules.
fn all_src_files() -> Vec<PathBuf> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read src") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// Where a real `Instant`/`elapsed`/`deadline` construct in production code
/// is allowed to stand, and why it is cadence, reporting, or bounds a
/// locally-owned wait rather than guessing at a remote operation's outcome.
/// Adding an entry is the "deliberate, reviewable act" the AMENDMENT asks
/// for — every entry below names its category and its reason; there is no
/// bare pass.
struct Allowed {
    file: &'static str,
    /// A substring of the exact source line, specific enough to be found by
    /// no other construct — not a line number, so an unrelated edit
    /// elsewhere in the file cannot silently detune this guard.
    needle: &'static str,
    category: &'static str,
    reason: &'static str,
}

/// Shared by every converted one-shot `.timeout(` entry below — see the
/// block comment above those entries for the full reasoning; kept as one
/// named constant rather than repeated so a future reader (or `grep`) finds
/// the single real argument once.
///
/// F-SF-01 (review of this wave): the text that used to live here argued
/// these 23 sites could stay non-deterministic because "no failure of this
/// shape has been observed here" — exactly the runner-speed-dependent
/// argument the ruling rejects (a fast dry-run passing says nothing about a
/// starved runner) — plus a scope-narrowing claim ("materially larger than
/// this wave's stated boundary") that cited no J-rung. Both premises were
/// false distinctions, not settled ground, so they are not repeated here:
/// every one of these sites now goes through `support::send_while_alive`
/// (`tests/support/mod.rs`), the one-shot analogue of
/// `scan_to_completion`'s own retry-while-alive loop — reused (Ponytail
/// R2), not reinvented — and is thereby converted under the ruling's own
/// text rather than exempted from it.
const RETRIED_WHILE_ALIVE_REASON: &str = "converted (F-SF-01): this client is passed to \
     `support::send_while_alive`, which retries a transport-class failure — including this \
     client's own `.timeout(` expiring — for as long as the daemon is alive, exactly as \
     `scan_to_completion` already does for its own status poll. The bound named by this needle is \
     now a per-attempt cadence for that retry loop, never a verdict this file decides on its own; \
     a POST this loop can retry is idempotent by the endpoint's own `command_id` dedup (proven by \
     this file's or a sibling's own below-window/pruned-command_id-retry assertions) or is not \
     retried at all (see the call site's own comment where that applies).";

/// The one occurrence F-SF-01's fix pass left unconverted: `m2_daemon_api.rs`
/// makes hundreds of direct, ad hoc `.send()` calls spread across its own
/// ~5000 lines and dozens of tests — unlike every other entry below, there
/// is no small, shared `get`/`post`/`submit` choke point a fix could route
/// through, and many of those calls assert on structured error bodies
/// (401/404 status, not a transport outcome) where retry-while-alive
/// semantics do not apply uniformly. Converting it is the same class of fix
/// as the 23 above, at a scale (a call-site-by-call-site rewrite of a
/// 5000-line file) that is a distinct piece of scope from this fix pass, not
/// a difference of principle — escalated rather than attempted piecemeal
/// here, and named honestly below as still-open non-determinism rather than
/// as a closed exemption.
const RESIDUE_REASON: &str = "a reqwest client built for a single, direct one-shot request/response \
     this test makes itself — never through scan_to_completion's polling loop, and never through \
     support::send_while_alive either (see this constant's own doc comment above: no small choke \
     point exists in this file to route the fix through). A transport-class failure here is the \
     same defect class scan_to_completion had, and no exemption in the ruling ('What this does not \
     say') covers a test harness's own client `.timeout(`: this is still open non-determinism, not \
     a closed matter, escalated to a future wave rather than fixed piecemeal here.";

const ALLOWLIST: &[Allowed] = &[
    Allowed {
        file: "src/watch.rs",
        needle: "let deadline = Instant::now() + deadman;",
        category: "owned-wait-budget",
        reason: "bounds this call's own wait for a *local* file it named and controls the \
                  lifetime of (a dev/test rendezvous marker), not a guess at a remote \
                  operation's state; the bound is the caller-supplied `deadman` parameter, \
                  never a guessed crate constant.",
    },
    Allowed {
        file: "src/watch.rs",
        needle: "if Instant::now() >= deadline {",
        category: "owned-wait-budget",
        reason: "the paired check for the `deadman` wait immediately above.",
    },
    Allowed {
        file: "src/cli.rs",
        needle: "let deadline = Instant::now() + SPAWN_WAIT;",
        category: "owned-wait-budget",
        reason: "bounds this client's own wait for the daemon *it just spawned* to publish a \
                  healthy descriptor — a local startup budget for a process this call owns, \
                  not a guess at an independently progressing remote operation's completion \
                  (the exact class `POLL_FAILURES_TOLERATED` wrongly modeled, which this wave \
                  removed).",
    },
    Allowed {
        file: "src/cli.rs",
        needle: "while Instant::now() < deadline {",
        category: "owned-wait-budget",
        reason: "the paired loop condition for the spawn-wait budget immediately above.",
    },
    Allowed {
        file: "src/cli.rs",
        needle: "let drain_deadline = Instant::now() + DRAIN_TIMEOUT;",
        category: "owned-wait-budget",
        reason: "bounds waiting for this same locally spawned daemon's own active-work count \
                  to drain before stop; a courtesy grace period before escalating, not a \
                  verdict about remote work succeeding.",
    },
    Allowed {
        file: "src/cli.rs",
        needle: "if active == 0 || Instant::now() >= drain_deadline {",
        category: "owned-wait-budget",
        reason: "the paired check for the drain budget immediately above.",
    },
    Allowed {
        file: "src/cli.rs",
        needle: "let term_deadline = Instant::now() + STOP_TERM_GRACE;",
        category: "owned-wait-budget",
        reason: "bounds waiting for this same locally spawned (and just SIGTERM'd) daemon's \
                  PID to exit before escalating; the outcome checked is `pid_alive`, a state \
                  fact, not the elapsed time itself.",
    },
    Allowed {
        file: "src/cli.rs",
        needle: "while daemon::pid_alive(descriptor.pid) && Instant::now() < term_deadline {",
        category: "owned-wait-budget",
        reason: "the paired loop condition for the termination-grace budget immediately above \
                  — note the actual exit condition it ORs against is `pid_alive`, state, not \
                  time alone.",
    },
    Allowed {
        file: "src/api.rs",
        needle: "let budget_deadline = Instant::now().checked_add(budget);",
        category: "owned-wait-budget",
        reason: "bounds `send_with_retry`'s own overall wait for a single one-shot request \
                  this call owns, on top of its existing per-attempt `client_timeout()` — the \
                  request-owned budget itself (S6 retry-owned-budget), not a guess at the \
                  daemon's own remote progress. Exhausting it against a still-live PID \
                  (`daemon::pid_alive`, checked immediately above in source order) ends the \
                  request naming exactly that; the pid-alive check, a state fact, is what \
                  actually distinguishes stuck from gone, same shape as `SPAWN_WAIT`/\
                  `DRAIN_TIMEOUT`/`STOP_TERM_GRACE` above. `checked_add` (not `+`) because a \
                  saturated `Duration::MAX` budget would otherwise overflow `Instant`'s own \
                  internal representation and panic; the `None` case reads as \"no deadline\", \
                  the same open-ended wait the pid-less path below already gets.",
    },
    Allowed {
        file: "src/api.rs",
        needle: "if let Some(deadline) = budget_deadline",
        category: "owned-wait-budget",
        reason: "unwraps the request-retry budget deadline set up above; S6 pidless-and-related \
                  seam 1 (Captain ruling, J4) split the old single `&&` chain into its own \
                  early return on a dead pid followed by this `if`, so the `Some`/`None` split \
                  on the budget deadline now heads its own statement rather than continuing an \
                  outer condition. `None` (an overflowing budget) still short-circuits this \
                  whole `if` so the budget is treated as absent rather than compared against.",
    },
    Allowed {
        file: "src/api.rs",
        needle: "&& Instant::now() >= deadline",
        category: "owned-wait-budget",
        reason: "the paired check for the request-retry budget immediately above.",
    },
    Allowed {
        file: "src/api.rs",
        needle: "let overdue = state.engine.due_interrupts(&core, Instant::now());",
        category: "scheduling",
        reason: "the current wall-clock instant is the *input* to an operator-scheduled \
                  interrupt's own recorded due time — the feature itself (a scheduled \
                  interrupt fires at or after its due time), not a guess standing in for \
                  unknown remote state.",
    },
    Allowed {
        file: "src/api.rs",
        needle: "let now = Instant::now();",
        category: "cadence",
        reason: "gates how often `maybe_run_periodic_sweep` runs against `state.sweep_interval`; \
                  a missed or delayed sweep costs nothing but a skipped log line (this \
                  function's own doc comment), no outcome is decided by the exact value.",
    },
    Allowed {
        file: "src/api.rs",
        needle: "let started = Instant::now();",
        category: "reporting",
        reason: "measures how long a scan took, to attach to `duration_ms` in the completion \
                  event JSON below; nothing branches on the value — the scan's actual outcome \
                  comes from its own recorded state, never this timer.",
    },
    Allowed {
        file: "src/api.rs",
        needle: "\"duration_ms\": started.elapsed().as_millis() as u64,",
        category: "reporting",
        reason: "the paired read for the `started` timer immediately above.",
    },
    Allowed {
        file: "src/daemon.rs",
        needle: "let rebuild_started = Instant::now();",
        category: "reporting",
        reason: "measures how long journal rebuild took at startup, for the log line below; \
                  startup success or failure is decided by `Journal::open_with`'s own result, \
                  never by this timer.",
    },
    Allowed {
        file: "src/daemon.rs",
        needle: "let rebuild_ms = u64::try_from(rebuild_started.elapsed().as_millis()).unwrap_or(u64::MAX);",
        category: "reporting",
        reason: "the paired read for the `rebuild_started` timer immediately above.",
    },
    Allowed {
        file: "src/daemon.rs",
        needle: "let prune_started = Instant::now();",
        category: "reporting",
        reason: "measures how long startup prune took, for the log line below; the prune's own \
                  outcome comes from `prune::run_startup`'s result, never from this timer.",
    },
    Allowed {
        file: "src/daemon.rs",
        needle: "let prune_duration_ms = u64::try_from(prune_started.elapsed().as_millis()).unwrap_or(u64::MAX);",
        category: "reporting",
        reason: "the paired read for the `prune_started` timer immediately above.",
    },
    Allowed {
        file: "src/runtime/atlas/lane.rs",
        needle: "deadline: WORKER_RUNTIME_DEADLINE,",
        category: "owned-wait-budget",
        reason: "threads the fixed HANG-guard budget into `WorkerRuntime`, consumed only by \
                  `atlas/worker.rs`'s own kill-and-reap of a child process this crate directly \
                  spawned and owns — not a guess about a remote operation's completion.",
    },
    Allowed {
        file: "src/backend/fake.rs",
        needle: "let deadline = Instant::now() + timeout;",
        category: "owned-wait-budget",
        reason: "bounds `Gate::wait_for_waiting`'s own rendezvous for local test-double threads \
                  this call parks and owns; the return value is a bool the caller branches on, \
                  but what is bounded is a local synchronization primitive, not a remote \
                  operation.",
    },
    Allowed {
        file: "src/backend/fake.rs",
        needle: "let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {",
        category: "owned-wait-budget",
        reason: "the paired remaining-budget check for the rendezvous wait immediately above.",
    },
    Allowed {
        file: "src/runtime/fsutil.rs",
        needle: "let deadline = Instant::now() + LOCK_BUDGET;",
        category: "owned-wait-budget",
        reason: "bounds `take_exclusive_lock`'s wait for this process's *own* leaked duplicate \
                  file-descriptor lock (this file's own doc comment: a fork/exec window before \
                  `O_CLOEXEC` fires) to clear — a local resource this process owns, not a guess \
                  about a remote operation.",
    },
    Allowed {
        file: "src/runtime/fsutil.rs",
        needle: "if !may_be_our_own_leak || Instant::now() >= deadline {",
        category: "owned-wait-budget",
        reason: "the paired check for the lock-budget wait immediately above.",
    },
    Allowed {
        file: "src/tui/estate.rs",
        needle: "started: Instant::now(),",
        category: "reporting",
        reason: "records when a repo-add began so `add_repo_body` can render an elapsed-seconds \
                  spinner line (this file's own doc comment: \"no fabricated clone percentage\"); \
                  the add's own outcome comes from the oneshot receiver, never from this timer.",
    },
    Allowed {
        file: "src/tui/estate.rs",
        needle: "pending.started.elapsed().as_secs(),",
        category: "reporting",
        reason: "the paired read for the `started` timer immediately above.",
    },
    Allowed {
        file: "src/runtime/journal.rs",
        needle: "let started = self.append_observer.as_ref().map(|_| Instant::now());",
        category: "reporting",
        reason: "measures one `append_event` call's write latency to hand an optional observer \
                  callback below; the append's own success or failure comes from the write's \
                  `Result`, never from this timer.",
    },
    Allowed {
        file: "src/runtime/journal.rs",
        needle: "observer(started.elapsed());",
        category: "reporting",
        reason: "the paired read for the `started` timer immediately above.",
    },
    Allowed {
        file: "src/runtime/git.rs",
        needle: "#[error(\"git {args:?} in {dir} exceeded its {deadline_secs}s deadline and was killed\")]",
        category: "owned-wait-budget",
        reason: "the error text for `GitError::TimedOut`, reporting the budget `run_supervised` \
                  (below) applied to a git subprocess this call directly spawned and reaped — \
                  not a guess about a remote operation's completion.",
    },
    Allowed {
        file: "src/runtime/git.rs",
        needle: "deadline_secs: u64,",
        category: "owned-wait-budget",
        reason: "the `GitError::TimedOut` field the error text above reports.",
    },
    Allowed {
        file: "src/runtime/git.rs",
        needle: "deadline: std::time::Duration,",
        category: "owned-wait-budget",
        reason: "the budget parameter `git_fetch_restricted`/`run_supervised` thread through to \
                  the kill-and-reap loop below — a hardened, directly-spawned subprocess this \
                  code owns (#310 discipline), matching `atlas/worker.rs`'s own HANG guard.",
    },
    Allowed {
        file: "src/runtime/git.rs",
        needle: "run_supervised(dest_path, &args_ref, allow_protocol, deadline)?;",
        category: "owned-wait-budget",
        reason: "passes the owned-subprocess budget through to `run_supervised`.",
    },
    Allowed {
        file: "src/runtime/git.rs",
        needle: "let deadline_at = std::time::Instant::now() + deadline;",
        category: "owned-wait-budget",
        reason: "the kill-and-reap loop's own budget for the git subprocess it just spawned.",
    },
    Allowed {
        file: "src/runtime/git.rs",
        needle: "if std::time::Instant::now() >= deadline_at {",
        category: "owned-wait-budget",
        reason: "the paired check for the subprocess kill-and-reap budget immediately above.",
    },
    Allowed {
        file: "src/runtime/git.rs",
        needle: "deadline_secs: deadline.as_secs(),",
        category: "owned-wait-budget",
        reason: "reports the budget that was exceeded in `GitError::TimedOut`'s own field.",
    },
    Allowed {
        file: "src/runtime/repolock.rs",
        needle: "let started = Instant::now();",
        category: "owned-wait-budget",
        reason: "bounds this call's own wait to acquire a filesystem lock it is about to hold — \
                  a local resource this process owns, not a guess about a remote operation.",
    },
    Allowed {
        file: "src/runtime/repolock.rs",
        needle: "let deadline = started + budget;",
        category: "owned-wait-budget",
        reason: "the paired budget for the lock-acquisition wait immediately above.",
    },
    Allowed {
        file: "src/runtime/repolock.rs",
        needle: "let now = Instant::now();",
        category: "owned-wait-budget",
        reason: "the lock-wait loop's own tick, checked against `deadline` below.",
    },
    Allowed {
        file: "src/runtime/repolock.rs",
        needle: "if now >= deadline {",
        category: "owned-wait-budget",
        reason: "the paired check for the lock-acquisition budget immediately above.",
    },
    Allowed {
        file: "src/runtime/repolock.rs",
        needle: "waited: started.elapsed(),",
        category: "reporting",
        reason: "reports how long the caller waited in `RepoLockError::Timeout`'s own field; \
                  the timeout verdict itself was already decided by the `now >= deadline` check \
                  above, this only reports the duration.",
    },
    Allowed {
        file: "src/runtime/repolock.rs",
        needle: "std::thread::sleep(delay.min(deadline - now));",
        category: "owned-wait-budget",
        reason: "never sleeps past the lock-acquisition budget checked above.",
    },
    Allowed {
        file: "src/backend/agy.rs",
        needle: "self.lines.push_back((Instant::now(), line));",
        category: "reporting",
        reason: "stamps a captured stderr line for later attribution in `take_stderr` (was this \
                  line adjacent to a given turn); eviction from the log is decided by \
                  `STREAM_MEMORY_CAP` byte accounting immediately below, never by this \
                  timestamp's age.",
    },
    Allowed {
        file: "src/backend/agy.rs",
        needle: "let deadline = Instant::now() + LOOP_DEATH_RECORD_GRACE;",
        category: "owned-wait-budget",
        reason: "bounds `write_failure`'s wait for this call's *own* already-spawned loop \
                  child's death record to land, so the refusal it returns can be specific; the \
                  write already failed before this wait starts.",
    },
    Allowed {
        file: "src/backend/agy.rs",
        needle: "if Instant::now() >= deadline {",
        category: "owned-wait-budget",
        reason: "the paired check for the death-record-grace wait immediately above (also \
                  covers the identically-shaped check in `await_loop_settle` below it).",
    },
    Allowed {
        file: "src/backend/agy.rs",
        needle: "let deadline = Instant::now() + budget;",
        category: "owned-wait-budget",
        reason: "bounds `await_loop_settle`'s wait for this call's own in-flight turn to settle \
                  after stdin closed; expiry is not an error (this fn's own doc comment) — it \
                  falls through to the group kill it already owns.",
    },
    Allowed {
        file: "src/backend/agy.rs",
        needle: "if !in_flight || Instant::now() >= deadline {",
        category: "owned-wait-budget",
        reason: "the paired check for the loop-settle wait immediately above.",
    },
    Allowed {
        file: "src/backend/agy.rs",
        needle: "first_event_at = Some(Instant::now());",
        category: "reporting",
        reason: "stamps this turn's first parsed event for `take_stderr`'s own \
                  before/after-the-turn attribution of already-captured stderr lines; nothing \
                  about the turn's success or failure is decided by this timestamp.",
    },
    Allowed {
        file: "src/tui/mod.rs",
        needle: "tokio::time::Instant::now() + backoff.next_delay()",
        category: "cadence",
        reason: "schedules the next automatic reconnect attempt on `backoff`'s own schedule; \
                  each attempt's own success or failure is decided by `try_attach`/`reconnected`'s \
                  result, never by this timer (also covers the three identically-shaped \
                  `.reset(...)` call sites elsewhere in this reconnect loop).",
    },
    Allowed {
        file: "src/runtime/atlas/worker.rs",
        needle: "pub deadline: Duration,",
        category: "owned-wait-budget",
        reason: "`WorkerSpawn`/`WorkerRuntime`'s own HANG-guard field (this file's own module \
                  doc) — how long a supervised parse worker this call directly spawns may run \
                  before being killed and reaped; not a guess about a remote operation.",
    },
    Allowed {
        file: "src/runtime/atlas/worker.rs",
        needle: "\"supervised parse worker exceeded its deadline and was killed (group signalled, \\",
        category: "owned-wait-budget",
        reason: "the coverage-row text for `WorkerFault::TimedOut`, reporting the HANG-guard \
                  budget above was exceeded for a directly-spawned, directly-owned worker.",
    },
    Allowed {
        file: "src/runtime/atlas/worker.rs",
        needle: "let deadline_at = Instant::now() + spawn.deadline;",
        category: "owned-wait-budget",
        reason: "the kill-and-reap loop's own budget for the parse worker it just spawned.",
    },
    Allowed {
        file: "src/runtime/atlas/worker.rs",
        needle: "if Instant::now() >= deadline_at {",
        category: "owned-wait-budget",
        reason: "the paired check for the worker kill-and-reap budget immediately above.",
    },
    Allowed {
        file: "src/runtime/atlas/worker.rs",
        needle: "\"kill/reap after the deadline itself failed: {e}\"",
        category: "owned-wait-budget",
        reason: "the failure text for when even the post-deadline kill/reap of the owned worker \
                  process could not complete.",
    },
    Allowed {
        file: "src/runtime/atlas/scan.rs",
        needle: "deadline: worker.deadline,",
        category: "owned-wait-budget",
        reason: "threads the same HANG-guard budget from `WorkerRuntime` into a per-claim \
                  `WorkerSpawn` — see `atlas/worker.rs`'s own entries above.",
    },
    Allowed {
        file: "src/runtime/engine.rs",
        needle: ".insert(work_id.to_string(), Instant::now());",
        category: "scheduling",
        reason: "records when a turn started so `due_interrupts`/`due_observations` can compare \
                  it against an operator-scheduled due time — the same `scheduling` class \
                  already allowlisted at `src/api.rs`'s `due_interrupts` call site, not a guess \
                  standing in for unknown remote state.",
    },
    // ------------------------------------------------ tests/ (S6, item 3)
    //
    // Every entry below is a `sleep(` this wave's real syn-based walk found
    // outside any while/loop/for construct and judged genuinely behavioral
    // rather than converting to `support::wait_until` — brief-sleep-and-hope.md
    // item 3: "A site whose sleep is genuinely behavioral... goes on the
    // allowlist with the reason written for a reader who disagrees — the
    // panel will." Two shapes recur and are named per entry rather than once:
    // (a) a fixed observation window proving an ABSENCE (nothing happened),
    // which has no positive state to poll for because the assertion itself
    // is "still nothing by now"; (b) deliberately spacing two actions (a
    // scripted delay, a race setup) where the delay's role is pacing, not a
    // verdict.
    Allowed {
        file: "tests/c4_repo_lock.rs",
        needle: "std::thread::sleep(hold_for);",
        category: "cadence",
        reason: "spaces this thread's own release of a lock *it* holds so the acquisition \
                  under test is genuinely waiting on a live foreign holder when it starts, not \
                  deciding whether that acquisition succeeded — `repolock::acquire`'s own \
                  `taken` result decides that, checked separately below.",
    },
    Allowed {
        file: "tests/codex_backend.rs",
        needle: "std::thread::sleep(Duration::from_millis(200));",
        category: "cadence",
        reason: "paces a spawned scripted-stub thread's own turn-release relative to `launch`, \
                  modelling a backend that takes a moment to accept before it starts — the \
                  assertion below (`launch must succeed`) is decided by `launch`'s own Result, \
                  never by this delay.",
    },
    Allowed {
        file: "tests/codex_backend.rs",
        needle: "std::thread::sleep(Duration::from_millis(500));",
        category: "cadence",
        reason: "paces the interrupt relative to a just-sent long turn so the interrupt lands \
                  mid-stream rather than before the turn starts producing output — what the \
                  interrupt actually did is decided by `observe`'s reported signal below, never \
                  by this delay's exact length.",
    },
    Allowed {
        file: "tests/codex_backend.rs",
        needle: "std::thread::sleep(Duration::from_millis(400));",
        category: "cadence",
        reason: "a second pacing delay before interrupt, immediately after a `while` loop \
                  (already state-terminating, not flagged) that confirmed the turn had started; \
                  this widens the mid-stream window the same way the entry above does, and the \
                  interrupt's outcome is still decided by `observe`, not by this sleep.",
    },
    Allowed {
        file: "tests/m2_daemon_api.rs",
        needle: "std::thread::sleep(Duration::from_millis(300));",
        category: "cadence",
        reason: "spaces two racing client spawns (racer A, then this delay, then racer B) to \
                  deliberately construct the race the test exercises; both clients' own exit \
                  status is asserted afterward, never a property of this delay's length.",
    },
    Allowed {
        file: "tests/m2_daemon_api.rs",
        needle: "tokio::time::sleep(Duration::from_millis(50)).await;",
        category: "cadence",
        reason: "a task-scheduling race between the spawned `handle.shutdown()` future and \
                  this test releasing an already-confirmed-stalled observe (confirmed via \
                  `fake.await_stalled_observes` immediately above, a real state wait, not \
                  flagged): the fake's gate API reports how many observers are stalled but has \
                  no distinct signal for 'shutdown specifically is now the one blocked on it' \
                  as opposed to any other in-flight caller. Adding that signal touches \
                  `src/backend/fake.rs`, out of this wave's scope (brief: 'do not touch src/ \
                  unless a test's wait has no observable state to wait on — then that is a \
                  product finding'); flagged here rather than converted, for the panel to weigh \
                  whether that signal is worth adding in a later wave.",
    },
    Allowed {
        file: "tests/m2_daemon_api.rs",
        needle: "tokio::time::sleep(Duration::from_millis(300)).await;",
        category: "cadence",
        reason: "a bounded observation window proving an absence — no leaked permit lets B \
                  admit while A's stop is only requested, not yet confirmed stopped (the \
                  comment above this line: 'enough to catch a leaked permit without making the \
                  test itself slow'). There is no positive state to wait on since the property \
                  under test is precisely that nothing (an illegitimate admission) happens \
                  within the window.",
    },
    Allowed {
        file: "tests/m3_execution.rs",
        needle: "tokio::time::sleep(poll * 2).await;",
        category: "cadence",
        reason: "a bounded observation window proving an absence — the work must still read \
                  `active`, never having concluded early on an ambiguous signal (the comment \
                  above: 'a signal that never arrives must never read as a conclusion on its \
                  own'). No positive state to wait on: the property under test is that nothing \
                  (a premature conclusion) has happened by this point.",
    },
    Allowed {
        file: "tests/m4_backends.rs",
        needle: "tokio::time::sleep(Duration::from_millis(1000)).await;",
        category: "cadence",
        reason: "a bounded observation window proving an absence (TH-5 test-honesty finding, \
                  named in the comment above): the completion driver must journal nothing \
                  across ~5 polling ticks of quiet. No positive state exists to wait on — the \
                  property under test is that the journal stays exactly as long as it was.",
    },
    Allowed {
        file: "tests/m4_backends.rs",
        needle: "std::thread::sleep(Duration::from_millis(750));",
        category: "cadence",
        reason: "a bounded observation window proving an absence: no write lands after `stop` \
                  has already returned and `observe` already reports `Exited` (both checked \
                  above, real state, not flagged). No positive state to wait on — the property \
                  under test is that the directory listing stays byte-identical.",
    },
    Allowed {
        file: "tests/m4_backends.rs",
        needle: "std::thread::sleep(Duration::from_secs(6));",
        category: "cadence",
        reason: "a live external-API test (`claude_live_enabled`-gated, not exercised without \
                  a live credential) modelling how long a real turn needs to run before it is \
                  genuinely mid-generation, not merely just-started — the comment above cites a \
                  measured ~3.5s to first API activity. Converting to poll `observe()` for \
                  `Running` risks catching the turn the instant client-side state flips to \
                  Running, which per that same measurement can precede real generation activity \
                  — exactly the race this test needs to not have. Left as a measured, \
                  provenance-cited delay rather than guess-converted against a live surface this \
                  environment cannot exercise to verify the replacement is actually safe.",
    },
    Allowed {
        file: "tests/m4_backends.rs",
        needle: "std::thread::sleep(Duration::from_millis(200));",
        category: "cadence",
        reason: "a bounded observation window proving an absence (the comment above: 'give a \
                  wrongly-still-spawned background thread a chance to run before asserting, so \
                  a regression that double-fires is actually caught'). No positive state to \
                  wait on — the property under test is that the counter does not increment a \
                  second time.",
    },
    Allowed {
        file: "tests/m5_projections.rs",
        needle: "tokio::time::sleep(Duration::from_millis(750)).await;",
        category: "cadence",
        reason: "a bounded observation window proving an absence, after `shutdown` has already \
                  returned (real state, not this timer): a disabled daemon's export task must \
                  not have dialed the collector. No positive state to wait on — the property \
                  under test is that the collector's hit counter stays at zero.",
    },
    Allowed {
        file: "tests/m6_surfaces.rs",
        needle: "std::thread::sleep(Duration::from_secs(1));",
        category: "cadence",
        reason: "paces hanging up the pty until after the watch has had a chance to install \
                  itself (the comment above: 'the watch installs itself as the loop starts; \
                  hang up after it has'), a TUI subprocess with no externally observable \
                  'watch installed' signal short of parsing its own rendered output — the \
                  decisive assertion below (`pid_alive`) is a real state check, not this delay.",
    },
    Allowed {
        file: "tests/opencode_backend.rs",
        needle: "std::thread::sleep(Duration::from_millis(100));",
        category: "cadence",
        reason: "paces a directly-spawned stub subprocess (not this crate's own backend) until \
                  it has reached its own internal stall loop, the comment above: 'give the stub \
                  a moment to reach its own stall loop' — an external process with no exposed \
                  readiness signal; what the test actually asserts comes later, checked against \
                  the backend's own observed state, never this delay.",
    },
    Allowed {
        file: "tests/opencode_backend.rs",
        needle: "std::thread::sleep(Duration::from_secs(2));",
        category: "cadence",
        reason: "paces an abort against a live shell tool (`sleep 30 && echo done-sleeping`) so \
                  the tool has genuinely started before it is interrupted — the file's own later \
                  comment on this same test documents this transport's abort/settle timing as \
                  live-measured with no prior figure to cite more precisely; the actual \
                  outcome is decided by `wait_for_settled_within` below, a real state wait.",
    },
    Allowed {
        file: "tests/support/mod.rs",
        needle: "std::thread::sleep(std::time::Duration::from_millis(10));",
        category: "cadence",
        reason: "each of 8 spawned threads holds `CrossProcessLock` for a fixed slice so the \
                  others have a real window to race for it — the point under test is mutual \
                  exclusion, measured by the peak overlap counter checked after every thread \
                  joins, never by this hold's length.",
    },
    Allowed {
        file: "tests/v1d_probe_child_lifecycle.rs",
        needle: "std::thread::sleep(Duration::from_secs(3600));",
        category: "owned-wait-budget",
        reason: "the role helper's own comment above this loop: 'block forever — the parent \
                  kills this process; that is the event under test'. There is no state to check \
                  because the fixture is deliberately not supposed to end on its own; what is \
                  under test is the parent's kill, asserted separately, never this loop's \
                  own return.",
    },
    Allowed {
        file: "tests/w3_client_surface.rs",
        needle: "std::thread::sleep(Duration::from_millis(300));",
        category: "cadence",
        reason: "both occurrences in this file (before triggering B's transition, and again \
                  before the second `--all` watcher): pace triggering a Work transition until \
                  after a just-spawned `sgt watch` subprocess has attached (read the journal \
                  head, opened its SSE stream) — the comment above: 'otherwise this is racing \
                  an unstarted subscriber, not testing the filter'. The subprocess exposes no \
                  readiness signal short of parsing its own piped stdout stream, which this test \
                  does not otherwise read; what is actually asserted is the watcher's own \
                  captured output, checked separately after this delay.",
    },
    Allowed {
        file: "tests/support/mod.rs",
        needle: "std::thread::sleep(delay);",
        category: "cadence",
        reason: "wave `transport-timeout-is-not-a-verdict`: inside `spawn_scripted_http_server`, \
                  holding one *scripted stub connection's* response for a caller-chosen `delay` \
                  before writing it — the delay is an input the test author picked (how long \
                  this stub connection stalls), never a verdict the stub computes about the \
                  request it is answering. The ruling's own carve-out \
                  (`tests-must-be-deterministic-2026-09-02.md`, 'What this does not say') names \
                  exactly this shape: time in the product's behavior *under test*, which this \
                  stub's caller sets (usually to stall past a client's own timeout on purpose, \
                  proving the caller retries rather than panics) — not a duration this code \
                  decides pass/fail by.",
    },
    // ---- tests/ `.timeout(` (wave `transport-timeout-is-not-a-verdict`, item 3) ----
    //
    // Two shapes, per the wave's own instruction: "convert it to the same
    // retry-while-alive path... or leave it, say so" for a genuine hang
    // guard. A third, real shape turned out to need naming too — a client
    // whose `.timeout(` feeds `scan_to_completion` directly is *already*
    // converted, one call site up: the retry now lives in the shared
    // helper (previous commit), so the client's own per-attempt bound is
    // cadence for that retry, not a verdict in its own right.
    //
    // The seven entries below are that first shape — every file whose
    // `client()`/`http()` builds the client `scan_to_completion` is then
    // handed.
    Allowed {
        file: "tests/s6_semantic_crossing.rs",
        needle: ".timeout(Duration::from_secs(60))",
        category: "cadence",
        reason: "this client is handed straight to scan_to_completion (tests/support/mod.rs), \
                  whose status poll now retries a transport-class failure — including this \
                  client's own timeout expiring — for as long as the daemon is alive; the bound \
                  here is a per-attempt cadence for that retry loop, not a verdict this file \
                  decides on its own.",
    },
    Allowed {
        file: "tests/s6_scan_front_door.rs",
        needle: ".timeout(Duration::from_secs(30))",
        category: "cadence",
        reason: "this client is handed straight to scan_to_completion (tests/support/mod.rs), \
                  whose status poll now retries a transport-class failure — including this \
                  client's own timeout expiring — for as long as the daemon is alive; the bound \
                  here is a per-attempt cadence for that retry loop, not a verdict this file \
                  decides on its own.",
    },
    Allowed {
        file: "tests/w1b_overlay_lifecycle_trigger.rs",
        needle: ".timeout(Duration::from_secs(60))",
        category: "cadence",
        reason: "this client is handed straight to scan_to_completion (tests/support/mod.rs), \
                  whose status poll now retries a transport-class failure — including this \
                  client's own timeout expiring — for as long as the daemon is alive; the bound \
                  here is a per-attempt cadence for that retry loop, not a verdict this file \
                  decides on its own.",
    },
    Allowed {
        file: "tests/w1d_overlay_freshness.rs",
        needle: ".timeout(Duration::from_secs(60))",
        category: "cadence",
        reason: "this client is handed straight to scan_to_completion (tests/support/mod.rs), \
                  whose status poll now retries a transport-class failure — including this \
                  client's own timeout expiring — for as long as the daemon is alive; the bound \
                  here is a per-attempt cadence for that retry loop, not a verdict this file \
                  decides on its own.",
    },
    Allowed {
        file: "tests/y5_external_git_triggers.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: "this client is handed straight to scan_to_completion (tests/support/mod.rs), \
                  whose status poll now retries a transport-class failure — including this \
                  client's own timeout expiring — for as long as the daemon is alive; the bound \
                  here is a per-attempt cadence for that retry loop, not a verdict this file \
                  decides on its own.",
    },
    Allowed {
        file: "tests/y6a_estate_scoped_scan.rs",
        needle: ".timeout(Duration::from_secs(30))",
        category: "cadence",
        reason: "this client is handed straight to scan_to_completion (tests/support/mod.rs), \
                  whose status poll now retries a transport-class failure — including this \
                  client's own timeout expiring — for as long as the daemon is alive; the bound \
                  here is a per-attempt cadence for that retry loop, not a verdict this file \
                  decides on its own.",
    },
    Allowed {
        file: "tests/y6b_online_only.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: "this client is handed straight to scan_to_completion (tests/support/mod.rs), \
                  whose status poll now retries a transport-class failure — including this \
                  client's own timeout expiring — for as long as the daemon is alive; the bound \
                  here is a per-attempt cadence for that retry loop, not a verdict this file \
                  decides on its own.",
    },
    // This suite's own regression fixture: the client's `.timeout(` value
    // *is* the behavior under test — a caller-chosen bound short enough
    // that a scripted stub's delay reliably outruns it, proving the fix
    // above retries rather than panics.
    Allowed {
        file: "tests/s6_scan_poll_survives_a_transport_timeout.rs",
        needle: ".timeout(client_timeout)",
        category: "cadence",
        reason: "wave `transport-timeout-is-not-a-verdict`'s own regression fixture: the caller \
                  picks this client's timeout per test (short enough that the scripted stub's \
                  delay outruns it, or long enough that a hangup never touches it) specifically \
                  to exercise scan_to_completion's retry-while-alive path — the timeout is the \
                  input under test, not a duration this suite lets decide its own verdict.",
    },
    // The remaining 24 sites (F-SF-01): a `reqwest::Client::builder()
    // .timeout(...)` built for ordinary one-shot request/response calls this
    // suite makes directly (never through scan_to_completion's polling
    // loop) — POST/GET a daemon endpoint once, `.expect()`/`assert` the
    // answer. 23 of the 24 are now converted, not merely allowlisted: each
    // one's client is passed to `support::send_while_alive`
    // (`tests/support/mod.rs`), the one-shot analogue of
    // `scan_to_completion`'s own retry-while-alive loop, so a transport
    // failure here is retried while the daemon is alive exactly as the
    // polled status GET already is, instead of deciding the test's outcome.
    // The one remaining site (`tests/m2_daemon_api.rs`) has no small choke
    // point to route the fix through and is left as reasoned, still-open
    // residue — see `RESIDUE_REASON`'s own doc comment above.
    Allowed {
        file: "tests/c1a_compiled_context.rs",
        needle: ".timeout(std::time::Duration::from_secs(30))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/c1a_compiled_context.rs",
        needle: ".timeout(std::time::Duration::from_secs(60))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/c1d_attribution_nesting_audit.rs",
        needle: ".timeout(std::time::Duration::from_secs(60))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/c2_light/agy_routing.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/c2_light/codex_routing.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/c2_light/opencode_routing.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/c2_light/t2_workflow_catalog.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/c2_light/w2fix_probe_ordering.rs",
        needle: ".timeout(Duration::from_secs(10))",
        category: "cadence",
        reason: "converted (F-SF-01) for its two idempotent GETs (`healthz`, the pending-window \
                  `list`), both now through `support::send_while_alive`. The one non-idempotent \
                  call this file makes — the pending-window submission, which creates a Work — is \
                  deliberately *not* retried here: see that call site's own comment. It no longer \
                  uses this client (or any `.timeout(` of its own) at all, so the only duration in \
                  play for it is `RENDEZVOUS`, the test's real, single termination bound.",
    },
    Allowed {
        file: "tests/e_admission_uses_no_network_git.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/e_git_admission.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/i9_floor_pinning.rs",
        needle: ".timeout(std::time::Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/m11_nested_workflow.rs",
        needle: ".timeout(std::time::Duration::from_secs(30))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/m12_child_work.rs",
        needle: ".timeout(std::time::Duration::from_secs(30))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/m2_daemon_api.rs",
        needle: ".timeout(Duration::from_secs(10))",
        category: "reporting",
        reason: RESIDUE_REASON,
    },
    Allowed {
        file: "tests/m3_execution.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/m5_projections.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/m6_surfaces.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/w3_prune_engine.rs",
        needle: ".timeout(std::time::Duration::from_secs(20))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/w4_doctor_journal_growth.rs",
        needle: ".timeout(Duration::from_secs(10))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/w4_read_surfaces.rs",
        needle: ".timeout(Duration::from_secs(10))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/x4_tabular_map.rs",
        needle: ".timeout(std::time::Duration::from_secs(10))",
        category: "cadence",
        reason: RETRIED_WHILE_ALIVE_REASON,
    },
    Allowed {
        file: "tests/c4_repo_lock.rs",
        needle: "Instant::now() + Duration::from_secs(120);",
        category: "owned-wait-budget",
        reason: "the lock-holding helper process (repository_lock_helper_process) is a plain \
            std::process::Command child with no access to tests/support (a different \
            compilation unit) -- this module's own doc already names it as a different shape, \
            not this guard's fold-in target: a bounded 'hold the lock until told otherwise' \
            loop guarding against a dead parent, checking real state (the release file's \
            existence) every pass.",
    },
    Allowed {
        file: "tests/c4_repo_lock.rs",
        needle: "Instant::now() + Duration::from_secs(60);",
        category: "owned-wait-budget",
        reason: "same helper-process shape as this file's other entry above: a bounded wait for \
            the helper's own ready file to appear, checking real state every pass, with no \
            access to tests/support from the parent test process either (kept small and \
            self-contained rather than adding a dependency for two call sites).",
    },
    Allowed {
        file: "tests/y1_worker_transport.rs",
        needle: "Instant::now() + std::time::Duration::from_secs(5);",
        category: "owned-wait-budget",
        reason: "the module's own comment calls this exactly what it is: 'a coarse whole-binary \
            backstop, not the decisive check' -- a bounded pgrep poll proving an *absence* (no \
            sgt-atlas-worker process survives), the same shape-(a) residue this file's own \
            module doc already names as tolerated (a fixed observation window with no positive \
            state to poll for). The decisive per-case assertion is FAULT_DEADLINE's own \
            kill+reap, already allowlisted separately as a `.timeout(`-class construct.",
    },
    Allowed {
        file: "tests/s6_scan_answers_while_embedding.rs",
        needle: ".timeout(support::HANG_BUDGET)",
        category: "cadence",
        reason: "support::HANG_BUDGET itself -- the shared hang-only bound this ruling's own \
            seam 2 introduces -- used exactly as documented: it ends a genuine hang, never a \
            slow-but-real answer, and this test's own state assertion (writing_source) is what \
            decides pass/fail, not how long any single poll took.",
    },
    Allowed {
        file: "tests/support/mod.rs",
        needle: "let deadline = Instant::now() + budget;",
        category: "owned-wait-budget",
        reason: "three real sites share this needle text. Two ARE the implementation this \
            whole wave folds every other site onto: wait_until's and wait_until_sync's own \
            `Instant::now() + budget` deadline computation -- the shared primitive cannot be \
            defined in terms of itself. The third, wait_until_gone (data_dir, budget) -> bool, \
            is reap_daemons's own SIGTERM-then-SIGKILL escalation gate: never panics, returns \
            whether the daemon actually left within `budget`, and reap_daemons uses the false \
            case to decide whether to escalate at all -- the same 'best-effort teardown that \
            proceeds regardless' shape support::wait_until's own doc names for stop_daemon \
            (tests/m2_daemon_api.rs, tests/m3_execution.rs), just local to this same file.",
    },
    Allowed {
        file: "tests/w3_client_surface.rs",
        needle: "let end = Instant::now() + deadline;",
        category: "owned-wait-budget",
        reason: "the module's own `wait_for` helper, after this wave's fold, has exactly one \
            surviving caller: the D6 scoped-watch test's own decisive assertion that \
            estate A's watch stays *silent* on estate B's transition within a fixed 800ms \
            window (`assert!(!quiet, ...)`) -- the same shape-(a) residue \
            `tests/y1_worker_transport.rs`'s own kept entry above already names: a fixed \
            observation window proving an absence, with no positive state to poll for and no \
            'eventually true or panic' contract to fold into (wait_until_sync panics when the \
            predicate never becomes true, which is exactly the passing case here). The other \
            two former callers of `wait_for` (daemon-stop confirmation, the --all watch match) \
            wanted a genuine 'eventually true' wait and were folded directly onto \
            support::wait_until_sync/HANG_BUDGET in this same commit.",
    },
    Allowed {
        file: "tests/w4_read_surfaces.rs",
        needle: "let deadline = Instant::now() + support::HANG_BUDGET;",
        category: "owned-wait-budget",
        reason: "two sites share this needle text (F-SF-01 fix pass), both stateful multi-await \
            accumulators over the same `&mut reqwest::Response`, same shape as \
            tests/m2_daemon_api.rs::read_sse_events's own kept entry: giving a wait_until \
            closure mutable access to `resp` across its own `.await` points needs the \
            RefCell-wrapped-stream pattern this wave already used elsewhere, which itself trips \
            clippy::await_holding_refcell_ref -- verified directly (`cargo check` on the literal \
            wait_until rewrite of drain_until_closed fails with 'captured variable cannot \
            escape `FnMut` closure body'). Both are now folded to the extent the shape allows: \
            budget consolidated onto support::HANG_BUDGET (was a bespoke 10s / caller-supplied \
            timeout), and both now panic naming the exact shortfall -- \
            read_raw_sse_frames names the frame count never observed, drain_until_closed names \
            the stream never closing -- rather than returning a value for the caller's own \
            assert to catch.",
    },
    Allowed {
        file: "tests/m9_watch.rs",
        needle: "let deadline = Instant::now() + Duration::from_secs(30);",
        category: "owned-wait-budget",
        reason: "r_watch_10a's own journal-stability wait: three CONSECUTIVE 250ms-apart \
            samples must read the same length before this loop calls the journal settled -- \
            the comment above names the 250ms cadence itself as load-bearing ('long enough to \
            outlast committer batching, far shorter than any real gap'). \
            support::wait_until_sync polls at its own fixed WAIT_POLL_INTERVAL (20ms, private \
            to tests/support/mod.rs), which would shrink the 3-sample stability window from \
            ~750ms to ~60ms and could false-positive mid-batch -- folding this one would change \
            the property under test, not just its budget.",
    },
    Allowed {
        file: "tests/v1d_probe_child_lifecycle.rs",
        needle: "let deadline = Instant::now() + budget;",
        category: "owned-wait-budget",
        reason: "wait_until_gone's own bounded poll: it returns bool (`true` once the pid is \
            gone, `false` if `budget` elapses with it still alive) rather than panicking either \
            way. Both callers need the bool: one runs its own cleanup (hard_kill) before \
            asserting on it with a message this generic helper cannot compose, the other \
            negates it to prove the CONTROL child survives (an absence-proving wait, the same \
            shape tests/y1_worker_transport.rs's own kept entry already names) -- the same \
            'caller decides pass/fail from the returned value' shape as \
            tests/w4_read_surfaces.rs's own kept entries.",
    },
    Allowed {
        file: "tests/v1d_probe_child_lifecycle.rs",
        needle: "let gone_by = Instant::now() + support::HANG_BUDGET;",
        category: "owned-wait-budget",
        reason: "the survivor-collection loop after the #310 leaker assertion: it returns \
            whichever pids are still alive when the budget elapses (possibly none), never \
            panicking itself -- the caller's own assert!(survivors.is_empty(), ...) below is \
            the actual verdict, with its own richer message (process states, argv). Budget \
            consolidated onto the one shared support::HANG_BUDGET (was a locally-named \
            DEADLINE = 30s duplicating it) even though the loop shape itself stays hand-rolled.",
    },
    Allowed {
        file: "tests/v1d_probe_child_lifecycle.rs",
        needle: "let deadline = Instant::now() + support::HANG_BUDGET;",
        category: "owned-wait-budget",
        reason: "two sites share this needle text (both already consolidated onto the one \
            shared support::HANG_BUDGET, was a locally-named DEADLINE = 30s duplicating it): \
            (1) the quiet-descendants loop in \
            a_completed_probe_walk_leaves_no_child_of_its_own_behind -- its own exit is not \
            this test's verdict either way (saw_a_child/leftover.is_empty() below are, checked \
            from state this loop leaves behind regardless of why it exited); folding it into \
            wait_until_sync's panic-on-timeout would add a new failure mode this test never \
            had. (2) daemon_handle_kill_reaps_a_probe_child_its_walk_still_has_live's own \
            async wait for a live serve child -- returns an empty Vec on timeout rather than \
            panicking, and the caller's own assert!(!live.is_empty(), ...) is the verdict, same \
            shape as (1).",
    },
    Allowed {
        file: "tests/v1d_probe_child_lifecycle.rs",
        needle: "let deadline = Instant::now() + Duration::from_secs(10);",
        category: "owned-wait-budget",
        reason: "the_data_dir_guard_leaves_no_descendant_of_its_daemons_alive's own capture \
            window: `deadline - Duration::from_secs(7)` is read inside the loop body to keep \
            listening for at least 3 of this window's 10 seconds once something has been \
            captured, a relationship anchored to this loop's own 10s span -- routing it through \
            support::HANG_BUDGET (120s) would silently change the 3-second minimum-listen \
            window this loop's own early-exit math depends on, not just relax an unrelated \
            ceiling. The loop itself never panics (it always proceeds to the survivors check \
            below, the same 'caller decides pass/fail' shape as this file's other kept sites).",
    },
    Allowed {
        file: "tests/m2_daemon_api.rs",
        needle: "let deadline = Instant::now() + support::HANG_BUDGET;",
        category: "owned-wait-budget",
        reason: "read_sse_events's own bounded, stateful SSE collector (F-SF-01 fix pass): its \
            own stop_daemon-shaped sibling entry that used to share this file's deadline needle \
            has been folded into support::wait_until_sync directly; this one stays, narrowly, \
            because it is not a side-effect-free predicate wait_until's `FnMut() -> Fut<bool>` \
            shape covers -- it accumulates parsed frames into `events` across many chunk reads, \
            and the only way to give a closure mutable access to that accumulator across its own \
            `.await` points is the RefCell-wrapped-stream pattern this same file already uses \
            elsewhere (tests/m2_daemon_api.rs's history-replay wait, ~line 2723) -- which itself \
            holds that RefCell borrow across an `.await`, a real clippy::await_holding_refcell_ref \
            violation already present in this file before this fix pass (verified: `cargo clippy \
            --test m2_daemon_api` fails on it at HEAD, unrelated to this entry). Replicating a \
            known-broken pattern to force this site through the same helper would trade one \
            defect for another, so the loop stays hand-rolled; the budget is now the shared \
            support::HANG_BUDGET (was a bespoke 10s) and the loop now panics naming the exact \
            shortfall by count when the budget elapses, rather than silently returning a short \
            Vec for the caller's own assert to catch -- the determinism the ruling asks for, \
            just not routed through wait_until's own body.",
    },
    Allowed {
        file: "tests/m5_projections.rs",
        needle: "let deadline = Instant::now() + Duration::from_secs(30);",
        category: "owned-wait-budget",
        reason: "wait_for_a_quiet_journal's own single-sample stability check (`len == \
            previous` at a 250ms cadence): its own doc explains at length why the 250ms yield \
            interval is load-bearing on a current-thread runtime (the committer's fair \
            tokio::sync::Mutex needs the runtime to actually turn between samples) -- the same \
            'the cadence itself is the property under test' reasoning \
            tests/m9_watch.rs::r_watch_10a's own kept entry already establishes. \
            support::wait_until's own poll cadence is different and not tunable per call site.",
    },
    Allowed {
        file: "tests/m5_projections.rs",
        needle: "let deadline = Instant::now() + Duration::from_secs(20);",
        category: "owned-wait-budget",
        reason: "otlp_export_reaches_a_collector_listening_on_the_configured_endpoint's own \
            background TCP accept-loop thread: it is a stand-in collector SERVER, not a wait \
            for a condition -- `listener.accept()` inside the loop has nothing in common with \
            support::wait_until_sync's `FnMut() -> bool` predicate shape, and the thread's own \
            exit (timeout or a captured trace POST) is not this test's verdict either way: the \
            main thread's own assert! after `collector.join()` is.",
    },
    Allowed {
        file: "tests/m6_surfaces.rs",
        needle: "let deadline = Instant::now() + Duration::from_secs(20);",
        category: "owned-wait-budget",
        reason: "the live-SSE-tail test's own event-consuming loop: each pass calls \
            `stream.next_event()` (its own inner 5s tokio::time::timeout, awaited directly), \
            not a side-effect-free state check -- the shape support::wait_until's predicate \
            (`FnMut() -> Fut<Output = bool>`, polled with its own sleep-based backoff) does not \
            fit. The loop accumulates `asked_to_refresh` across possibly-many events and can \
            exit either by finding the target event or by the deadline; the post-loop \
            assert!(asked_to_refresh, ...) is the actual verdict either way.",
    },
    Allowed {
        file: "tests/m6_surfaces.rs",
        needle: "let deadline = Instant::now() + support::HANG_BUDGET;",
        category: "owned-wait-budget",
        reason: "the add-repo overlay test's own background-poll loop: `app` is a plain local \
            `&mut` used both before and after this wait across many more call sites in the same \
            test, and support::wait_until's predicate is `FnMut() -> Fut` -- its returned \
            future cannot hold a mutable borrow of a captured variable across its own await \
            points ('captured variable cannot escape `FnMut` closure body'). Where that bit a \
            value captured ONLY inside the wait elsewhere in this wave, a \
            std::cell::RefCell scoped to just that wait was the narrow fix; wrapping `app` \
            itself for this one call would thread interior mutability through a value this \
            whole test otherwise owns directly. Budget consolidated onto the shared \
            support::HANG_BUDGET even though the loop shape stays.",
    },
    Allowed {
        file: "tests/m6_surfaces.rs",
        needle: "let deadline = Instant::now() + Duration::from_secs(30);",
        category: "owned-wait-budget",
        reason: "two sites share this needle text, both named (directly or by shape) in \
            support::wait_until's own doc comment as staying hand-rolled. (1) the pty-hangup \
            test's own TUI-survival wait -- support::wait_until's doc names it explicitly: \
            'one that must run cleanup before failing (tests/m6_surfaces.rs's TUI-survival \
            wait)'; it never panics itself, `survived = pid_alive(tui)` after the loop decides, \
            and the caller kills the survivor before asserting. (2) SpawnedDaemon::start_at's \
            own descriptor-poll: kills and reaps its own child before panicking on timeout, the \
            same 'must run cleanup before failing' shape.",
    },
    Allowed {
        file: "tests/m6_surfaces.rs",
        needle: "let deadline = Instant::now() + DAEMON_TERM_GRACE;",
        category: "owned-wait-budget",
        reason: "SpawnedDaemon::stop's own SIGTERM-then-SIGKILL escalation: never panics, \
            returns a DaemonStop describing which signal actually worked -- the same \
            'best-effort teardown that proceeds regardless' shape support::wait_until's own \
            doc names for stop_daemon (tests/m2_daemon_api.rs, tests/m3_execution.rs), just \
            with an escalation action on timeout instead of a bare return.",
    },
];

/// Whether some entry in `allowlist` names both `file_label` and a `needle`
/// that occurs in `line` — the one membership check both the `src/` guard
/// (`unallowed_time_constructs`) and the `tests/` guard
/// (`unallowed_test_sleeps`) apply, so the two guards share it rather than
/// each carrying its own copy (R6).
fn allowlist_covers(allowlist: &[Allowed], file_label: &str, line: &str) -> bool {
    allowlist
        .iter()
        .any(|a| a.file == file_label && line.contains(a.needle))
}

/// Every `Instant::now()`, `.elapsed()`, or `deadline`-named construct in
/// `text`'s **production code** — everything before this crate's own
/// `#[cfg(test)]` test-module convention, when present — that is not
/// covered by `allowlist` for `file_label`. Doc/line comments (`//`, `///`,
/// `//!`) are not code and never match.
fn unallowed_time_constructs(
    file_label: &str,
    text: &str,
    allowlist: &[Allowed],
) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut cutoff = lines.len();
    for i in 0..lines.len() {
        // This crate names an inline test module several ways —
        // `mod tests {`, `pub(crate) mod tests {`, `pub(crate) mod
        // test_support {` (`src/lib.rs`, `src/runtime/journal.rs`,
        // `src/telemetry.rs`) — so the boundary check matches any `mod
        // <ident> {` immediately under `#[cfg(test)]`, not the single
        // literal spelling `mod tests {`. Widening this was necessary for
        // the crate-wide walk below to be honest: a narrower match left
        // `src/lib.rs`'s and `src/runtime/journal.rs`'s own `#[cfg(test)]`
        // modules unrecognized as test-only.
        if lines[i].trim() != "#[cfg(test)]" {
            continue;
        }
        let Some(next) = lines.get(i + 1).map(|l| l.trim()) else {
            continue;
        };
        let after_mod = next
            .strip_prefix("pub(crate) mod ")
            .or_else(|| next.strip_prefix("pub mod "))
            .or_else(|| next.strip_prefix("mod "));
        if let Some(rest) = after_mod
            && rest.trim_end().ends_with('{')
            && rest
                .trim_end()
                .trim_end_matches('{')
                .trim()
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            cutoff = i;
            break;
        }
    }
    let mut violations = Vec::new();
    for (i, line) in lines[..cutoff].iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue; // doc/line comment, not code
        }
        let is_construct = line.contains("Instant::now()")
            || line.contains(".elapsed()")
            || line.contains("deadline");
        if !is_construct {
            continue;
        }
        let covered = allowlist_covers(allowlist, file_label, line);
        if !covered {
            violations.push((i + 1, line.trim().to_string()));
        }
    }
    violations
}

/// The guard itself: every real `Instant`/`elapsed`/`deadline` construct in
/// the crate's own production code — every `.rs` file under `src/`, per
/// `all_src_files` above — sits on `ALLOWLIST`, each entry naming a category
/// (`cadence`, `reporting`, `scheduling`, or `owned-wait-budget`) and a
/// reason it is not a guessed verdict about a remote operation's completion
/// — never a bare, unexplained pass.
///
/// A count-based `POLL_FAILURES_TOLERATED`-shaped reintroduction into
/// `run_intelligence_scan`'s follow loop fails this test the same way it
/// would have failed before this wave: the new construct has no matching
/// `ALLOWLIST` entry, so it is reported by file and line rather than
/// silently passing. So does a new construct introduced anywhere else in
/// the crate — the walk is no longer scoped to the daemon follow surface.
#[test]
fn every_time_construct_in_the_crate_sits_on_an_explicit_allowlist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all_violations = Vec::new();
    for path in all_src_files() {
        let file = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_str()
            .expect("utf8 path")
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", file));
        for (line, content) in unallowed_time_constructs(&file, &text, ALLOWLIST) {
            all_violations.push(format!("{file}:{line}: {content}"));
        }
    }
    assert!(
        all_violations.is_empty(),
        "an Instant/elapsed/deadline construct exists with no ALLOWLIST entry explaining why \
         it is cadence, reporting, scheduling, or an owned-wait-budget rather than a guessed \
         verdict about a remote operation's completion — add a reviewed `Allowed` entry (or fix \
         the construct) in tests/s6_no_clock_decides_correctness.rs:\n{}",
        all_violations.join("\n")
    );

    // Every allowlist entry must actually have matched something in the
    // files it names, or it is dead weight documenting a construct that no
    // longer exists — silently making the guard weaker than it claims.
    for entry in ALLOWLIST {
        let text = std::fs::read_to_string(root.join(entry.file))
            .unwrap_or_else(|e| panic!("read {}: {e}", entry.file));
        assert!(
            text.contains(entry.needle),
            "ALLOWLIST entry for {}:{:?} (category {:?}) matches nothing in the file any \
             more — remove the stale entry",
            entry.file,
            entry.needle,
            entry.category,
        );
        assert!(
            entry.reason.len() > 20,
            "ALLOWLIST entry for {}:{:?} names no real reason — a category with no \
             explanation is the bare, unexplained pass this guard exists to refuse",
            entry.file,
            entry.needle,
        );
    }
}

/// Proof the guard above is not vacuous (AMENDMENT: "a guard nobody has
/// seen fail is not a guard"), kept as a standing regression rather than a
/// one-off manual demonstration: a synthetic file with exactly the
/// forbidden shape — `let deadline = Instant::now() + BUDGET; ... if
/// Instant::now() > deadline { <declare failure> }`, the AMENDMENT's own
/// worked example — and no allowlist entry for it. The checker must flag
/// it by file and line.
#[test]
fn the_guard_fails_on_a_real_deadline_decides_a_verdict_construct_with_no_allowlist_entry() {
    let synthetic = "\
async fn run_intelligence_scan_reintroduces_the_defect() {
    let deadline = Instant::now() + BUDGET;
    while Instant::now() < deadline {
        match poll().await {
            Ok(v) => return Ok(v),
            Err(_) => continue,
        }
    }
    return Err(\"scan did not complete within BUDGET\");
}
";
    let violations = unallowed_time_constructs("src/cli.rs", synthetic, ALLOWLIST);
    assert!(
        !violations.is_empty(),
        "the checker must flag an unlisted deadline-decides-a-verdict construct; it did not, \
         which means the guard test above is vacuous"
    );
    assert!(
        violations
            .iter()
            .any(|(line, text)| *line == 2 && text.contains("deadline")),
        "expected the flagged construct at its real line and content, got: {violations:?}"
    );

    // And the same shape, once given a matching allowlist entry, is no
    // longer flagged — proving the checker's *pass* path is exercised too,
    // not only its failure path.
    let permissive: Vec<Allowed> = vec![Allowed {
        file: "src/cli.rs",
        needle: "let deadline = Instant::now() + BUDGET;",
        category: "cadence",
        reason: "synthetic fixture only, proving the allowlist path itself is reachable",
    }];
    let still_flagged = unallowed_time_constructs("src/cli.rs", synthetic, &permissive);
    assert!(
        still_flagged.iter().any(|(line, _)| *line == 3),
        "the paired `while Instant::now() < deadline` line has no entry in `permissive` and \
         must still be flagged: {still_flagged:?}"
    );
}

// ---------------------------------------------------------------- tests/

/// Every `.rs` file under `tests/`, walked recursively. Unlike `src/`,
/// integration test files carry no `#[cfg(test)]` boundary — every line of
/// an integration-test file *is* test code, so the whole file is in scope
/// (brief-sleep-and-hope.md item 1: "extend the no-clock structural guard
/// to `tests/`").
fn all_test_files() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut found = Vec::new();
    let mut stack = vec![dir];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read tests") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                // `tests/fixtures/` is data corpora this suite reads as
                // *input* (deliberately malformed Rust included, e.g.
                // `tslp_corpus/malformed/broken.rs` — the corpus gate's own
                // red-side fixture per its own doc comment), never test
                // code the guard should itself parse and classify.
                if path.file_name().is_some_and(|n| n == "fixtures") {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// One raw `sleep(` call found by a real `syn` parse (`full` + `visit`,
/// 3.0.3 — already resolved transitively via `async-trait`, R5), with the
/// one fact this guard's classification turns on: does the `while`/`loop`/
/// `for` construct directly containing it actually check *observed state*
/// and terminate on it — a non-trivial `while` condition, or a body-level
/// conditional break/continue/return/panic ([`loop_body_checks_state`]) —
/// or is it bare, or lexically inside a loop that never checks anything.
///
/// A brace/regex scanner is explicitly rejected (brief-sleep-and-hope.md
/// item 1: "a real parse... beats a brace scanner that a reformat evades")
/// — this walks the actual AST, so reindenting or reformatting the file
/// cannot silently detune the check the way a column/brace count would.
///
/// A bounded `for _ in 0..N { ...check state...; sleep(...) }` counts as
/// "inside a loop" the same as `while`/`loop`: this crate's own dominant
/// polling idiom (e.g. `completed_work()` above:
/// `for _ in 0..200 { if <state>.is_terminal() { return } sleep(25ms) }`)
/// checks real observed state every iteration and returns/panics on it;
/// the fixed iteration count times the fixed sleep is just an owned wait
/// budget spelled as a count instead of an `Instant` deadline (the same
/// `owned-wait-budget` class the `src/` guard's own `ALLOWLIST` already
/// recognizes), not the forbidden `POLL_FAILURES_TOLERATED` shape — that
/// shape substitutes a *failure count* for a state check ("assume the
/// daemon is gone after 5 failures" instead of asking "is it still
/// there") rather than checking state every pass and merely bounding how
/// many passes that check gets.
struct SleepSite {
    line: usize,
    in_state_loop: bool,
}

/// Whether `path` names a panic-family macro (`panic!`, `assert!`,
/// `assert_eq!`, `assert_ne!`, `unreachable!`) — the "panics on it" half of
/// this module's own "checks real observed state every iteration and
/// returns/panics on it" claim (doc above).
fn is_panic_macro_path(path: &syn::Path) -> bool {
    path.segments.last().is_some_and(|s| {
        matches!(
            s.ident.to_string().as_str(),
            "panic" | "assert" | "assert_eq" | "assert_ne" | "unreachable"
        )
    })
}

/// Whether a `while` loop's condition is the literal `true` — the shape
/// that (like a bare `loop {}`) contributes no state check of its own, so
/// [`loop_body_checks_state`] must find one in the body instead.
fn is_literal_true(cond: &syn::Expr) -> bool {
    matches!(
        cond,
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) if b.value
    )
}

/// Whether `block` — a loop body — contains a `break`, `continue`,
/// `return`, or panic-family macro call that is *conditional*: reached only
/// through an `if` or `match` inside `block`, not a bare unconditional one.
/// This is the structural form of this module's own claim that a counted
/// loop "checks real observed state every iteration and returns/panics on
/// it" (doc above) — a `for _ in 0..N { sleep(...) }` with no such
/// conditional has nothing deciding correctness; it is a fixed-count
/// busy-wait wearing the `owned-wait-budget` shape, not the real thing.
/// Does not descend into a loop or closure nested inside `block` — that
/// construct must justify its own `sleep` independently.
fn loop_body_checks_state(block: &syn::Block) -> bool {
    struct CheckFinder {
        in_conditional: u32,
        found: bool,
    }

    impl<'ast> Visit<'ast> for CheckFinder {
        fn visit_expr_while(&mut self, _node: &'ast syn::ExprWhile) {}
        fn visit_expr_loop(&mut self, _node: &'ast syn::ExprLoop) {}
        fn visit_expr_for_loop(&mut self, _node: &'ast syn::ExprForLoop) {}
        fn visit_expr_closure(&mut self, _node: &'ast syn::ExprClosure) {}

        fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
            self.in_conditional += 1;
            visit::visit_expr_if(self, node);
            self.in_conditional -= 1;
        }

        fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
            self.in_conditional += 1;
            visit::visit_expr_match(self, node);
            self.in_conditional -= 1;
        }

        fn visit_expr_break(&mut self, node: &'ast syn::ExprBreak) {
            if self.in_conditional > 0 {
                self.found = true;
            }
            visit::visit_expr_break(self, node);
        }

        fn visit_expr_continue(&mut self, node: &'ast syn::ExprContinue) {
            if self.in_conditional > 0 {
                self.found = true;
            }
            visit::visit_expr_continue(self, node);
        }

        fn visit_expr_return(&mut self, node: &'ast syn::ExprReturn) {
            if self.in_conditional > 0 {
                self.found = true;
            }
            visit::visit_expr_return(self, node);
        }

        fn visit_macro(&mut self, node: &'ast syn::Macro) {
            if self.in_conditional > 0 && is_panic_macro_path(&node.path) {
                self.found = true;
            }
            visit::visit_macro(self, node);
        }
    }

    let mut finder = CheckFinder {
        in_conditional: 0,
        found: false,
    };
    finder.visit_block(block);
    finder.found
}

/// One `.timeout(` method call found by the same walk as [`SleepSite`]
/// (F-SI-01: folded into the one [`TestConstructVisitor`] below rather than
/// a second, parallel struct/impl/fn pipeline — Ponytail R2). Unlike a
/// `sleep(`, a client-builder `.timeout(` carries no loop-shape question —
/// there is no "inside a state-terminating loop" reading of a client's own
/// configured per-request bound, only "does something explain why this
/// construct exists" — so every real occurrence sits on `ALLOWLIST` or the
/// guard is red (wave `transport-timeout-is-not-a-verdict`, item 4: "Client
/// timeouts are the other half of the same class" `sleep(` already covers).
struct TimeoutSite {
    line: usize,
}

/// One hand-rolled `Instant::now() + <duration>` deadline construction
/// found by the same walk (seam 2, no-clock-decides). Unlike a `sleep(`,
/// this is never legitimate on its own merits the way an `owned-wait-budget`
/// `sleep(` can be: the one conversion target is
/// `tests/support::wait_until`/`wait_until_sync`, which already own this
/// exact shape (a deadline, a predicate, a named panic on exhaustion) behind
/// one shared, reviewed constant (`support::HANG_BUDGET`). A real occurrence
/// sits on `ALLOWLIST` — with a `hand-rolled-deadline` category naming why
/// it is not folded — or the guard is red.
///
/// A real `syn` parse, not a text/brace scanner, for the same reason this
/// file's own doc already gives for `sleep(`: reformatting or reindenting
/// the file must not silently detune this. The construct detected is the
/// binary `+` expression itself (`Instant::now() + Duration::...`), which is
/// where every real site's deadline is actually built, whatever the loop
/// around it looks like.
struct DeadlineSite {
    line: usize,
}

/// Whether `expr` is a call to `Instant::now()` — any path ending in `now`
/// with `Instant` somewhere in the path, so `std::time::Instant::now()`,
/// `Instant::now()`, and an aliased import all match without needing this
/// guard to resolve imports.
fn is_instant_now_call(expr: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = expr else {
        return false;
    };
    let syn::Expr::Path(path) = &*call.func else {
        return false;
    };
    let segments: Vec<String> = path
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect();
    segments.last().is_some_and(|last| last == "now") && segments.iter().any(|s| s == "Instant")
}

/// The one `syn` visitor for every `tests/` construct this file's guards
/// walk — a `sleep(`/`.timeout(` call, a hand-rolled `Instant::now() + `
/// deadline, and the loop state around them — in a single pass over a
/// single parse (F-SI-01: this used to be two structs, two `impl Visit`s
/// and two entry-point functions, each parsing the same file text again
/// from scratch).
struct TestConstructVisitor {
    /// Whether the loop directly containing the current position — the
    /// innermost one, per [`loop_body_checks_state`]'s doc — has been shown
    /// to check state and branch on it. Empty outside any loop.
    loop_checks_state: Vec<bool>,
    sleep_sites: Vec<SleepSite>,
    timeout_sites: Vec<TimeoutSite>,
    deadline_sites: Vec<DeadlineSite>,
}

impl<'ast> Visit<'ast> for TestConstructVisitor {
    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        let checks = !is_literal_true(&node.cond) || loop_body_checks_state(&node.body);
        self.loop_checks_state.push(checks);
        visit::visit_expr_while(self, node);
        self.loop_checks_state.pop();
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.loop_checks_state
            .push(loop_body_checks_state(&node.body));
        visit::visit_expr_loop(self, node);
        self.loop_checks_state.pop();
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.loop_checks_state
            .push(loop_body_checks_state(&node.body));
        visit::visit_expr_for_loop(self, node);
        self.loop_checks_state.pop();
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        let is_sleep = matches!(
            &*node.func,
            syn::Expr::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "sleep")
        );
        if is_sleep {
            self.sleep_sites.push(SleepSite {
                line: node.span().start().line,
                in_state_loop: self.loop_checks_state.last().copied().unwrap_or(false),
            });
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "timeout" {
            // `node.span()` covers the whole receiver chain (from
            // `reqwest::Client::builder()` through this call) — the method
            // name's own span is what actually sits on the `.timeout(` line
            // a reader (and `ALLOWLIST`'s needle matching) expects.
            self.timeout_sites.push(TimeoutSite {
                line: node.method.span().start().line,
            });
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if matches!(node.op, syn::BinOp::Add(_)) && is_instant_now_call(&node.left) {
            self.deadline_sites.push(DeadlineSite {
                line: node.span().start().line,
            });
        }
        visit::visit_expr_binary(self, node);
    }
}

/// Parses `text` once and walks it once, returning every `sleep(` site,
/// every `.timeout(` site, and every hand-rolled deadline site found — the
/// one parse/visit pass shared by [`unallowed_test_sleeps`],
/// [`unallowed_test_timeouts`], and [`unallowed_test_deadlines`] (F-SI-01,
/// seam 2 widens the same pass rather than adding a second one).
fn scan_test_constructs(text: &str) -> (Vec<SleepSite>, Vec<TimeoutSite>, Vec<DeadlineSite>) {
    let file = syn::parse_file(text).unwrap_or_else(|e| panic!("parse: {e}"));
    let mut visitor = TestConstructVisitor {
        loop_checks_state: Vec::new(),
        sleep_sites: Vec::new(),
        timeout_sites: Vec::new(),
        deadline_sites: Vec::new(),
    };
    visitor.visit_file(&file);
    (
        visitor.sleep_sites,
        visitor.timeout_sites,
        visitor.deadline_sites,
    )
}

/// Every `sleep(` call in `text` (a `tests/` file, `file_label`) that is
/// neither inside a state-terminating `while`/`loop` nor covered by
/// `allowlist` — reusing the exact same `Allowed` shape and `file`+`needle`
/// matching the `src/` guard above already uses (R2), so the same
/// "matches nothing"/"no real reason" self-checks in the test below cover
/// these entries too without a second, parallel check.
fn unallowed_test_sleeps(
    file_label: &str,
    text: &str,
    allowlist: &[Allowed],
) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut violations = Vec::new();
    for site in scan_test_constructs(text).0 {
        if site.in_state_loop {
            continue;
        }
        let content = lines
            .get(site.line - 1)
            .copied()
            .unwrap_or("")
            .trim()
            .to_string();
        let covered = allowlist_covers(allowlist, file_label, &content);
        if !covered {
            violations.push((site.line, content));
        }
    }
    violations
}

/// The `tests/` half of the guard (brief-sleep-and-hope.md item 1): every
/// raw `sleep(` call in `tests/**/*.rs` sits inside a loop that can
/// terminate on observed state, or on `ALLOWLIST` with a real reason — same
/// entry shape and same self-checks the `src/` guard above already has
/// (R2), just widened to a second file set and a second construct.
#[test]
fn every_sleep_in_tests_sits_inside_a_terminating_loop_or_an_explicit_allowlist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all_violations = Vec::new();
    for path in all_test_files() {
        let file = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_str()
            .expect("utf8 path")
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", file));
        for (line, content) in unallowed_test_sleeps(&file, &text, ALLOWLIST) {
            all_violations.push(format!("{file}:{line}: {content}"));
        }
    }
    assert!(
        all_violations.is_empty(),
        "a `sleep(` call exists in tests/ that is neither inside a state-terminating \
         while/loop nor explained by an ALLOWLIST entry — convert it to wait on the state it \
         is hoping for (tests/support/mod.rs::wait_until), or add a reviewed `Allowed` entry \
         naming why it is genuinely behavioral, in \
         tests/s6_no_clock_decides_correctness.rs:\n{}",
        all_violations.join("\n")
    );
}

/// Proof the sleep guard above is not vacuous, same standing-regression
/// shape as `the_guard_fails_on_a_real_deadline_decides_a_verdict_construct_with_no_allowlist_entry`
/// above: a synthetic bare `sleep(` with no loop and no allowlist entry
/// must be flagged; the same call once inside a `while` loop, or once
/// allowlisted, must not be.
#[test]
fn the_sleep_guard_fails_on_a_bare_sleep_with_no_loop_and_no_allowlist_entry() {
    let bare = "\
async fn hopes_for_the_best() {
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(some_background_task_finished());
}
";
    let violations = unallowed_test_sleeps("tests/x_example.rs", bare, ALLOWLIST);
    assert!(
        !violations.is_empty(),
        "the checker must flag a bare sleep-then-assert with no loop and no allowlist entry; \
         it did not, which means the guard test above is vacuous"
    );
    assert!(
        violations.iter().any(|(line, _)| *line == 2),
        "expected the flagged construct at its real line, got: {violations:?}"
    );

    let in_loop = "\
async fn polls_for_state() {
    while !some_background_task_finished() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
";
    let loop_violations = unallowed_test_sleeps("tests/x_example.rs", in_loop, ALLOWLIST);
    assert!(
        loop_violations.is_empty(),
        "a sleep inside a while loop that terminates on observed state must not be flagged: \
         {loop_violations:?}"
    );

    let allowlisted: Vec<Allowed> = vec![Allowed {
        file: "tests/x_example.rs",
        needle: "tokio::time::sleep(Duration::from_millis(200)).await;",
        category: "cadence",
        reason: "synthetic fixture only, proving the tests/ allowlist path itself is reachable",
    }];
    let now_covered = unallowed_test_sleeps("tests/x_example.rs", bare, &allowlisted);
    assert!(
        now_covered.is_empty(),
        "the same bare sleep, once given a matching allowlist entry, must no longer be \
         flagged: {now_covered:?}"
    );
}

/// F-TH-01: lexical nesting inside a `while`/`loop`/`for` is not by itself
/// proof of a state check — a `for _ in 0..N { sleep(...) }` with no
/// conditional exit is a fixed-count busy-wait, indistinguishable in effect
/// from the forbidden bare `POLL_FAILURES_TOLERATED` shape (this module's
/// own doc, above), and the guard must flag it exactly as it would flag a
/// bare sleep. The doctrine-sanctioned counterexample right above it — a
/// `for` loop whose body *does* check state and conditionally returns —
/// must still pass, so the fix is the conditional exit, not merely being
/// inside a loop.
#[test]
fn the_sleep_guard_fails_on_a_state_blind_loop_even_though_it_is_lexically_a_loop() {
    let state_blind = "\
async fn hopes_five_times() {
    for _ in 0..5 {
        std::thread::sleep(Duration::from_millis(200));
    }
}
";
    let violations = unallowed_test_sleeps("tests/x_example.rs", state_blind, ALLOWLIST);
    assert!(
        !violations.is_empty(),
        "a fixed-count loop whose body never checks state and never conditionally exits must \
         still be flagged — lexical nesting alone is not a state check"
    );

    let state_blind_bare_loop = "\
async fn spins_forever() {
    loop {
        std::thread::sleep(Duration::from_millis(200));
    }
}
";
    let bare_loop_violations =
        unallowed_test_sleeps("tests/x_example.rs", state_blind_bare_loop, ALLOWLIST);
    assert!(
        !bare_loop_violations.is_empty(),
        "a bare `loop` with no conditional break/return/panic must still be flagged"
    );

    let genuinely_checked = "\
async fn polls_five_times() {
    for _ in 0..5 {
        if some_background_task_finished() {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}
";
    let checked_violations =
        unallowed_test_sleeps("tests/x_example.rs", genuinely_checked, ALLOWLIST);
    assert!(
        checked_violations.is_empty(),
        "a fixed-count loop whose body conditionally returns on real state must not be \
         flagged: {checked_violations:?}"
    );
}

// ------------------------------------------------------ tests/ .timeout(

/// Every `.timeout(` call in `text` (a `tests/` file, `file_label`) not
/// covered by `allowlist` — the same `Allowed` shape and `file`+`needle`
/// matching (`allowlist_covers`) every other half of this guard already
/// uses (R2), so the same "matches nothing"/"no real reason" self-checks
/// cover these entries too without a second, parallel check.
fn unallowed_test_timeouts(
    file_label: &str,
    text: &str,
    allowlist: &[Allowed],
) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut violations = Vec::new();
    for site in scan_test_constructs(text).1 {
        let content = lines
            .get(site.line - 1)
            .copied()
            .unwrap_or("")
            .trim()
            .to_string();
        let covered = allowlist_covers(allowlist, file_label, &content);
        if !covered {
            violations.push((site.line, content));
        }
    }
    violations
}

/// The `tests/` `.timeout(` half of the guard (brief
/// `transport-timeout-is-not-a-verdict.md`, item 4): every client-builder
/// `.timeout(` call in `tests/**/*.rs` sits on `ALLOWLIST` with a real
/// reason — same entry shape and same self-checks the `sleep(` guard above
/// already has (R2), widened to a third construct.
#[test]
fn every_client_timeout_in_tests_sits_on_an_explicit_allowlist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all_violations = Vec::new();
    for path in all_test_files() {
        let file = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_str()
            .expect("utf8 path")
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", file));
        for (line, content) in unallowed_test_timeouts(&file, &text, ALLOWLIST) {
            all_violations.push(format!("{file}:{line}: {content}"));
        }
    }
    assert!(
        all_violations.is_empty(),
        "a `.timeout(` call exists in tests/ with no ALLOWLIST entry explaining why it is not a \
         duration deciding an expected-success test's verdict — convert the caller to retry \
         while the daemon is alive (tests/support/mod.rs::scan_to_completion) or add a reviewed \
         `Allowed` entry naming why it is genuinely behavioral (a hang guard, a stalled-backend \
         test), in tests/s6_no_clock_decides_correctness.rs:\n{}",
        all_violations.join("\n")
    );
}

/// Proof the `.timeout(` guard above is not vacuous, same standing-
/// regression shape as the `sleep(` guard's own vacuity test: a synthetic
/// client-builder `.timeout(` with no allowlist entry must be flagged; the
/// same call once allowlisted must not be.
#[test]
fn the_timeout_guard_fails_on_an_unallowlisted_client_timeout() {
    let synthetic = "\
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .expect(\"client\")
}
";
    let violations = unallowed_test_timeouts("tests/x_example.rs", synthetic, ALLOWLIST);
    assert!(
        !violations.is_empty(),
        "the checker must flag an unlisted client `.timeout(` construct; it did not, which \
         means the guard test above is vacuous"
    );
    assert!(
        violations
            .iter()
            .any(|(line, text)| *line == 3 && text.contains("timeout")),
        "expected the flagged construct at its real line and content, got: {violations:?}"
    );

    let permissive: Vec<Allowed> = vec![Allowed {
        file: "tests/x_example.rs",
        needle: ".timeout(Duration::from_secs(20))",
        category: "cadence",
        reason: "synthetic fixture only, proving the allowlist path itself is reachable",
    }];
    let still_clean = unallowed_test_timeouts("tests/x_example.rs", synthetic, &permissive);
    assert!(
        still_clean.is_empty(),
        "an allowlisted `.timeout(` must not be flagged: {still_clean:?}"
    );
}

/// Every hand-rolled `Instant::now() + <duration>` deadline construction in
/// `text` (a `tests/` file, `file_label`) that is not covered by
/// `allowlist` — the same `Allowed` shape and `file`+`needle` matching
/// (`allowlist_covers`) every other half of this guard already uses (R2).
fn unallowed_test_deadlines(
    file_label: &str,
    text: &str,
    allowlist: &[Allowed],
) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut violations = Vec::new();
    for site in scan_test_constructs(text).2 {
        let content = lines
            .get(site.line - 1)
            .copied()
            .unwrap_or("")
            .trim()
            .to_string();
        let covered = allowlist_covers(allowlist, file_label, &content);
        if !covered {
            violations.push((site.line, content));
        }
    }
    violations
}

/// The `tests/` hand-rolled-deadline half of the guard (seam 2,
/// no-clock-decides): every `Instant::now() + ` deadline construction in
/// `tests/**/*.rs` sits on `ALLOWLIST` with a real reason — same entry
/// shape and same self-checks the `sleep(`/`.timeout(` guards above already
/// have (R2), widened to a fourth construct.
///
/// **Not yet exhaustive over the whole tree.** `DEADLINE_LOOP_RESIDUE`
/// names every real site this stage found but did not fold into
/// `support::wait_until`/`wait_until_sync` within this seam's own scope —
/// escalated honestly, the same posture `RESIDUE_REASON` above already
/// established in this file for the `.timeout(` guard's own unconverted
/// `m2_daemon_api.rs` residue, not a closed exemption. A future pass folds
/// each one and removes its entry; this test is exhaustive over `tests/`
/// today in the sense that every real site is *named*, not in the sense
/// that every one is *fixed*.
#[test]
fn every_hand_rolled_deadline_in_tests_is_folded_or_named_in_the_residue() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all_violations = Vec::new();
    for path in all_test_files() {
        let file = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_str()
            .expect("utf8 path")
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", file));
        for (line, content) in unallowed_test_deadlines(&file, &text, ALLOWLIST) {
            all_violations.push(format!("{file}:{line}: {content}"));
        }
    }
    assert!(
        all_violations.is_empty(),
        "a hand-rolled `Instant::now() + ` deadline exists in tests/ that is neither folded \
         into tests/support::wait_until/wait_until_sync nor named in ALLOWLIST's \
         `hand-rolled-deadline`/`deadline-loop-residue` entries — convert it, or add a \
         reviewed `Allowed` entry, in tests/s6_no_clock_decides_correctness.rs:\n{}",
        all_violations.join("\n")
    );
}

/// Proof the deadline guard above is not vacuous, same standing-regression
/// shape as the `sleep(`/`.timeout(` guards' own vacuity tests: a synthetic
/// hand-rolled `Instant::now() + ` deadline loop with no allowlist entry
/// must be flagged; the same construct once allowlisted must not be.
#[test]
fn the_deadline_guard_fails_on_a_real_hand_rolled_loop_with_no_allowlist_entry() {
    let synthetic = "\
fn poll() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if ready() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!(\"never observed\");
}
";
    let violations = unallowed_test_deadlines("tests/x_example.rs", synthetic, ALLOWLIST);
    assert!(
        !violations.is_empty(),
        "the checker must flag an unlisted hand-rolled deadline construct; it did not, which \
         means the guard test above is vacuous"
    );
    assert!(
        violations
            .iter()
            .any(|(line, text)| *line == 2 && text.contains("Instant::now()")),
        "expected the flagged construct at its real line and content, got: {violations:?}"
    );

    let permissive: Vec<Allowed> = vec![Allowed {
        file: "tests/x_example.rs",
        needle: "Instant::now() + std::time::Duration::from_secs(5)",
        category: "hand-rolled-deadline",
        reason: "synthetic fixture only, proving the allowlist path itself is reachable",
    }];
    let still_clean = unallowed_test_deadlines("tests/x_example.rs", synthetic, &permissive);
    assert!(
        still_clean.is_empty(),
        "an allowlisted hand-rolled deadline must not be flagged: {still_clean:?}"
    );
}

//! S6 scan-follow-retry: the structural guard the AMENDMENT (owner,
//! 2026-08-30, `knowledge/evidence/resources/host-atlas-s6-series/
//! brief-scan-front-door.md`) asks for — "a prose prohibition is what let
//! this class recur. Pin it: a structural test that fails when the rule is
//! violated."
//!
//! **The rule.** "Elapsed time may never decide whether something is
//! correct... `POLL_FAILURES_TOLERATED = 5` — a guessed number standing in
//! for 'the daemon is gone'. Ask the real question instead: is the daemon
//! still there? ... Distinguish the two by state, not by a count." Cadence
//! (`SCAN_POLL`) and reporting a duration are explicitly permitted; a count
//! or deadline that a caller's own success/failure branches on is not.
//!
//! **What this test covers, and says so rather than implying more.** Per
//! the AMENDMENT's own text (`brief-scan-front-door.md:130`, "every
//! `Instant`/`deadline`/`elapsed` construct in the crate sits on an
//! explicit allowlist") and its closing instruction
//! (`brief-scan-front-door.md:145-147`, "this covers *this* crate's Rust;
//! it does not reach the shell scripts under `scripts/`"): this guard walks
//! the **production code** (everything before each file's own
//! `#[cfg(test)]` test-module boundary — this crate's own convention, see
//! e.g. `src/cli.rs:7553`) of every `.rs` file under `src/` — the whole
//! crate's Rust, not a named subset of it. `scripts/` is not Rust and is
//! outside this walk for exactly the reason the AMENDMENT names, not a
//! second, invented carve-out.
//!
//! **Every real construct this widened walk finds sits on `ALLOWLIST`
//! below, each with its own reviewed category and reason** — there is no
//! file-level exemption for any production `.rs` file, including the ones
//! that bound a process or resource this code directly owns and must
//! reclaim (a spawned child killed past its own budget in
//! `src/runtime/atlas/worker.rs`, whose own module doc frames its deadline
//! as a "HANG guard"; a filesystem lock's acquisition timeout in
//! `src/runtime/repolock.rs`/`src/runtime/fsutil.rs`; a supervised git
//! subprocess in `src/runtime/git.rs`; an interactive backend's turn
//! timeout in `src/backend/agy.rs`). That class is real and distinct from
//! the defect this wave fixed — see each entry's own `owned-wait-budget`
//! reason — but it is not exempted from the guard the way an earlier
//! version of this file exempted it by file name; it is allowlisted like
//! everything else, on its own reviewed merits.

use std::path::{Path, PathBuf};

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
];

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
        let covered = allowlist
            .iter()
            .any(|a| a.file == file_label && line.contains(a.needle));
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

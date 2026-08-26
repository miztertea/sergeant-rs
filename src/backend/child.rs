//! Child-process lifecycle shared by every adapter (#310).
//!
//! ## The measurement this module exists for
//!
//! Diagnosed on Cerberus 2026-08-26 after four OOM-driven session and host
//! deaths: `daemon::start_with` probes every registered backend, the opencode
//! probe runs a real `opencode serve --port 0` and the codex probe a real
//! `codex app-server --listen stdio://`, and the suites start daemons by the
//! hundred and kill them abruptly. A daemon killed while a probe child is
//! live leaves that child reparented to init, where it stays — measured at
//! ~265-342 MB RSS each, ages up to many hours, one killed at 74 GB total-vm.
//! Reproduced against this tree before the fix: ten `sgt daemon` starts
//! SIGKILLed at staggered delays left three `opencode serve --port 0
//! --hostname 127.0.0.1` processes at PPID 1.
//!
//! Neither doctrinal orphan-check pattern matched them (both are `sgt`-shaped;
//! the leaked species is named `opencode` and `codex`), so every wave's
//! hygiene check was honest and blind.
//!
//! ## The three mechanisms, and why all three
//!
//! 1. **Explicit kill and reap when the probe completes.** The gate functions
//!    already do this on every return path; it is the ordinary case and it is
//!    not what failed.
//! 2. **A `Drop` backstop on the owning handle.** Covers an adapter dropped
//!    without an explicit stop — a path that is not supported lifecycle, but
//!    must still not orphan a process.
//! 3. **[`harden_probe_child`], which is the one that closes #310.** Neither
//!    of the first two runs when the daemon dies by `SIGKILL`: no destructor
//!    of any kind executes for a killed process. Only the kernel can couple
//!    the child's death to the parent's, and on Linux
//!    `prctl(PR_SET_PDEATHSIG, SIGKILL)` is that coupling.
//!
//! ## Why a probe child keeps its *own* process group
//!
//! #310's requirement 3 names "the daemon's process group". This module puts
//! each probe child in a **new** group instead (`process_group(0)`), which is
//! what `opencode_serve` and `codex_appserver` already did for their launch
//! children and what [`kill_process_group`] is built to signal. The reason is
//! that the two goals want opposite things: sharing the daemon's group makes
//! `kill -KILL -<pgid>` unusable (it would take the daemon with it), so the
//! probe's own cleanup could then reach only the direct child and would leak
//! any grandchild it spawned. An own group plus `PR_SET_PDEATHSIG` gets both
//! halves — a precise subtree kill on completion *and* kernel-guaranteed
//! death when the daemon dies — so it is strictly stronger than the literal
//! wording, and it is what is implemented. J1: local, reversible, changes no
//! contract; the requirement's stated goal ("daemon death must take probe
//! children with it") is met, not narrowed.
//!
//! **Where the guarantee stops, stated rather than hidden.** `PR_SET_PDEATHSIG`
//! is Linux-only; macOS has no equivalent, so on macOS a probe child outlives
//! a `SIGKILL`ed daemon exactly as before. What covers macOS is the *reaper*
//! half of the fix — `platform::process::descendants` captured before the
//! kill (`tests/support`'s `DataDir` guard) — not this function.

use std::collections::BTreeSet;
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Kill a whole process group by the pgid recorded at spawn — `SIGKILL` to
/// the negated group id, through a shell rather than a `libc`/`nix` call for
/// one signal (R5). Through `/bin/sh -c` specifically, not by spawning `kill`
/// as a program: `kill` is a shell builtin every POSIX shell has, while
/// `kill(1)` as an executable on `PATH` is a package a host need not install,
/// and `Command::new("kill")` fails with `ENOENT` on such a host — a silent
/// no-op if the caller drops the result.
///
/// Nothing gates this on the leader being alive, and that is the whole point:
/// the group routinely outlives its leader (a command a turn started in the
/// background survives the agent process once it has exited and been reaped —
/// opencode probe 11's finding), so the group id is signalled unconditionally
/// and `ESRCH` (an already-empty group) is success, not an error to report.
/// Idempotent for the same reason.
///
/// **One copy, not four (R2).** `codex.rs`, `opencode.rs` and `agy.rs` each
/// carried a byte-identical private version of this, and #310 needed a fourth
/// for the probe path. They now all delegate here.
pub(crate) fn kill_process_group(pgid: Option<u32>) {
    let Some(pgid) = pgid else { return };
    #[cfg(unix)]
    {
        if let Err(e) = Command::new("/bin/sh")
            .arg("-c")
            .arg(format!("kill -KILL -{pgid}"))
            .output()
        {
            tracing::warn!(
                pgid,
                error = %e,
                "could not run the process-group kill; the direct child is all that was \
                 reached — any commands it spawned may still be running"
            );
        }
    }
    #[cfg(not(unix))]
    {
        tracing::warn!(
            pgid,
            "no process-group signal mechanism on this platform; killing only the direct \
             child — any commands it spawned may still be running"
        );
    }
}

/// Whether a spawned child is bounded by the call that spawns it, or owned
/// across a whole execution.
///
/// The distinction is load-bearing and must stay explicit at every call site,
/// because [`harden_probe_child`]'s hardening is only ever *correct* for the
/// first kind.
///
/// **What the parent-death signal is actually coupled to, measured here
/// rather than taken from the man page (Cerberus, Linux 7.0.0, 2026-08-26).**
/// `prctl(2)` documents the "parent" as *the thread that created the process*,
/// which would make a hardened child die when its spawning thread returned.
/// Measured on this kernel it does **not**: a C reproduction whose spawning
/// `pthread` exits leaves the child running, while `SIGKILL`ing the whole
/// parent process kills it immediately. So on this kernel it is process
/// death, not thread death, that fires it.
///
/// The split is kept anyway, and is not decoration: the man page's wording is
/// the portable contract, this crate's `rust-version` floor says nothing about
/// which kernel it runs on, and a build that *does* couple to thread death
/// would kill a live agent the moment its spawning worker returned to the
/// blocking pool. A probe child is spawned and killed inside one function on
/// one thread, so it is safe under either reading; an execution child is not,
/// so it never gets the hardening. `tests/v1d_probe_child_lifecycle.rs` pins
/// the process-death half, which is the half #310 depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChildLifetime {
    /// Killed before the call that spawned it returns (a probe gate).
    Probe,
    /// Owned across an execution, killed by the adapter's own lifecycle.
    Execution,
}

/// Put `command`'s child in its own process group and, on Linux, arrange for
/// the kernel to `SIGKILL` it if its parent dies first.
///
/// Call this only for [`ChildLifetime::Probe`] children — see that variant's
/// doc for why an execution child must not get it.
///
/// `pub` rather than `pub(crate)` so `tests/v1d_probe_child_lifecycle.rs` can
/// evidence the parent-death coupling from a real, separately-spawned
/// process: a unit test inside this crate's own test binary cannot be killed
/// to prove it, and the mechanism is exactly the one #310 turns on.
pub fn harden_probe_child(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // A new group, not the daemon's: see the module doc.
        command.process_group(0);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        let parent = std::process::id();
        // SAFETY (`pre_exec`'s own contract): this closure runs in the child
        // between `fork` and `exec`, in a process that has one thread and may
        // hold locks the parent's other threads were mid-way through. It may
        // therefore call only async-signal-safe functions and must allocate
        // nothing. All three calls below are on POSIX's async-signal-safe
        // list (`prctl` is Linux's own, documented signal-safe), none
        // allocates, and the captured `parent` is a plain `u32` copied into
        // the closure before the fork.
        unsafe {
            command.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                // The fork/prctl race: if the parent died between `fork` and
                // the `prctl` above, the death signal it would have triggered
                // has already been and gone, and this child would live
                // forever — which is precisely #310's failure. Re-reading the
                // parent after arming is the standard close: a mismatch means
                // the parent is already gone, so leave rather than exec.
                if libc::getppid() != parent as libc::pid_t {
                    libc::_exit(0);
                }
                Ok(())
            });
        }
    }
    #[cfg(not(any(unix, target_os = "linux")))]
    {
        let _ = command;
    }
}

/// The probe children one daemon's probe walk has live right now.
///
/// **Why ownership is per-walk rather than global.** `cargo test` runs many
/// tests in one process, so several in-process daemons can be probing at
/// once; a global set would let one daemon's `kill` take another's live probe
/// child down and turn its probe into a spurious refusal. The walk installs
/// its own set on the thread it calls `Backend::probe` from
/// ([`owned_by`]), and every hardened probe child spawned under that call
/// records itself into whichever set it finds there — exact attribution with
/// no change to the `Backend` trait, which has no daemon-shaped parameter to
/// carry one.
#[derive(Debug, Default)]
pub struct ProbeChildren {
    live: Mutex<BTreeSet<u32>>,
}

impl ProbeChildren {
    /// An empty set, ready to be installed by a probe walk.
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// The process groups this walk currently has live, sorted.
    pub fn live(&self) -> Vec<u32> {
        self.live
            .lock()
            .expect("probe children lock")
            .iter()
            .copied()
            .collect()
    }

    /// `SIGKILL` every live probe child's whole group, returning the pgids
    /// signalled. Idempotent, and safe to call on a walk that never spawned
    /// anything.
    ///
    /// The set is drained under the lock *before* any signal goes out, so a
    /// concurrent caller cannot signal the same group twice and a probe that
    /// finishes mid-kill deregisters into an empty set rather than blocking
    /// on one held across a subprocess.
    pub fn kill_all(&self) -> Vec<u32> {
        let killed: Vec<u32> = std::mem::take(&mut *self.live.lock().expect("probe children lock"))
            .into_iter()
            .collect();
        for pgid in &killed {
            kill_process_group(Some(*pgid));
        }
        killed
    }

    fn record(&self, pgid: u32) {
        self.live.lock().expect("probe children lock").insert(pgid);
    }

    fn forget(&self, pgid: u32) {
        self.live.lock().expect("probe children lock").remove(&pgid);
    }
}

thread_local! {
    /// The set probe children spawned on this thread record themselves into.
    static OWNER: std::cell::RefCell<Option<Arc<ProbeChildren>>> =
        const { std::cell::RefCell::new(None) };
}

/// Run `f` with `owner` installed as the probe-child owner for this thread,
/// restoring whatever was there before.
///
/// Restoring rather than clearing matters because the blocking pool reuses
/// threads: a walk that left `None` behind would be indistinguishable from a
/// thread that never had an owner, and the *next* walk scheduled onto that
/// thread would install its own anyway — but a nested call (a probe that
/// itself drives a probe) must not silently disown the outer one.
pub fn owned_by<R>(owner: Arc<ProbeChildren>, f: impl FnOnce() -> R) -> R {
    let previous = OWNER.with(|slot| slot.borrow_mut().replace(owner));
    let result = f();
    OWNER.with(|slot| *slot.borrow_mut() = previous);
    result
}

/// The probe-child owner installed on this thread, if any.
///
/// Public to the crate so an adapter that isolates a probe on a helper thread
/// (`opencode`'s serve gate runs its whole reqwest-touching body on a freshly
/// spawned OS thread) can carry the owner across that boundary — a
/// thread-local does not cross a `thread::spawn` on its own.
pub(crate) fn owner() -> Option<Arc<ProbeChildren>> {
    OWNER.with(|slot| slot.borrow().clone())
}

/// Deregisters one probe child from the walk that owns it.
///
/// Deliberately does **not** kill on drop: the handle that owns the child
/// (`ServeChild`, `AppServerChild`, `ConfigProbeChild`) already kills its
/// group in its own `Drop`, and two kills of one group is one wasted
/// subprocess per probe on the ordinary path.
#[derive(Debug)]
pub(crate) struct ProbeChildRegistration {
    owner: Option<Arc<ProbeChildren>>,
    pgid: u32,
}

impl Drop for ProbeChildRegistration {
    fn drop(&mut self) {
        if let Some(owner) = &self.owner {
            owner.forget(self.pgid);
        }
    }
}

/// Record a just-spawned probe child against whichever walk owns this thread.
///
/// A child spawned outside a walk (a unit test, a direct `probe()` call)
/// registers against nothing and is still hardened — the registration is what
/// makes a *live* child reachable by [`ProbeChildren::kill_all`], not what
/// makes it die.
pub(crate) fn register_probe_child(pgid: u32) -> ProbeChildRegistration {
    let owner = owner();
    if let Some(owner) = &owner {
        owner.record(pgid);
    }
    ProbeChildRegistration { owner, pgid }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    fn alive(pid: u32) -> bool {
        crate::platform::process::process_alive(pid)
    }

    fn wait_until_gone(pid: u32, budget: Duration) -> bool {
        let deadline = Instant::now() + budget;
        loop {
            if !alive(pid) {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn sleeper(harden: bool) -> std::process::Child {
        let mut command = Command::new("/bin/sh");
        command
            .arg("-c")
            .arg("sleep 300")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if harden {
            harden_probe_child(&mut command);
        }
        command.spawn().expect("spawn a sleeper")
    }

    /// The parent-death coupling `harden_probe_child` arms is *not*
    /// observable from inside this test binary — measured on this kernel
    /// (Cerberus, Linux 7.0.0, 2026-08-26) it fires on the parent
    /// **process**'s death, not the spawning thread's, so evidencing it means
    /// killing a process, which a unit test cannot do to itself.
    /// `tests/v1d_probe_child_lifecycle.rs` owns that assertion, against a
    /// separately spawned parent and against a real `SIGKILL`ed daemon. What
    /// stays here is everything a live process *can* prove about a hardened
    /// child: which process group it leads, and how the walk that owns it
    /// finds it again.
    ///
    /// Pinned as an explicit negative so nobody re-adds a thread-death
    /// assertion here and watches it fail for the right reason.
    #[cfg(target_os = "linux")]
    #[test]
    fn parent_death_coupling_is_not_thread_death_on_this_kernel() {
        let mut child = std::thread::spawn(|| sleeper(true))
            .join()
            .expect("spawner thread");
        let pid = child.id();
        let died_with_the_thread = wait_until_gone(pid, Duration::from_secs(2));
        kill_process_group(Some(pid));
        let _ = child.kill();
        let _ = child.wait();
        assert!(
            !died_with_the_thread,
            "PR_SET_PDEATHSIG now fires on the spawning thread's death on this kernel. That \
             is the portable reading of prctl(2) and it makes `ChildLifetime::Execution` \
             load-bearing rather than merely defensive — re-read that enum's doc before \
             changing anything here."
        );
    }

    /// `/proc/<pid>/stat`'s process-group id (field 5). The `comm` field
    /// before it is parenthesised and unescaped, so the positional fields
    /// after it are found by splitting at the **last** `)`, never the first.
    #[cfg(target_os = "linux")]
    fn pgid_of(pid: u32) -> Option<u32> {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let after_comm = stat.rsplit_once(") ")?.1;
        // After `) ` the fields are state, ppid, pgrp, ... — pgrp is index 2.
        after_comm.split_whitespace().nth(2)?.parse::<u32>().ok()
    }

    /// Its own group means the pgid *is* the child's pid, which is what every
    /// caller records at spawn. Had the child instead inherited this
    /// process's group, `kill -KILL -<pgid>` would signal the test runner
    /// itself — which is why the module deliberately does not put probe
    /// children in the daemon's group.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_hardened_child_leads_its_own_process_group_so_the_group_kill_is_precise() {
        let mut child = sleeper(true);
        let pid = child.id();
        let pgid = pgid_of(pid);
        kill_process_group(Some(pid));
        let _ = child.wait();
        assert_eq!(
            pgid,
            Some(pid),
            "a hardened probe child must lead its own process group"
        );
    }

    #[test]
    fn probe_children_are_recorded_against_the_walk_that_owns_the_thread() {
        let owner = ProbeChildren::new();
        let recorded = owned_by(owner.clone(), || {
            let child = sleeper(true);
            let registration = register_probe_child(child.id());
            let seen = owner.live();
            drop(registration);
            let mut child = child;
            kill_process_group(Some(child.id()));
            let _ = child.wait();
            seen
        });
        assert_eq!(
            recorded.len(),
            1,
            "the child was not recorded: {recorded:?}"
        );
        assert!(
            owner.live().is_empty(),
            "dropping the registration must deregister: {:?}",
            owner.live()
        );
    }

    #[test]
    fn a_child_spawned_outside_a_walk_records_against_nothing() {
        let mut child = sleeper(true);
        let registration = register_probe_child(child.id());
        drop(registration);
        kill_process_group(Some(child.id()));
        let _ = child.wait();
    }

    #[test]
    fn kill_all_reaps_every_live_probe_child_of_that_walk_and_no_other() {
        let mine = ProbeChildren::new();
        let theirs = ProbeChildren::new();
        let (mut my_child, my_registration) = owned_by(mine.clone(), || {
            let child = sleeper(true);
            let registration = register_probe_child(child.id());
            (child, registration)
        });
        let (mut their_child, their_registration) = owned_by(theirs.clone(), || {
            let child = sleeper(true);
            let registration = register_probe_child(child.id());
            (child, registration)
        });
        let (my_pid, their_pid) = (my_child.id(), their_child.id());

        assert_eq!(mine.kill_all(), vec![my_pid]);
        let _ = my_child.wait();
        assert!(
            wait_until_gone(my_pid, Duration::from_secs(10)),
            "kill_all left this walk's own probe child {my_pid} alive"
        );
        assert!(
            alive(their_pid),
            "kill_all reached another walk's probe child {their_pid} — a global set would \
             turn a sibling daemon's probe into a spurious refusal"
        );

        assert_eq!(theirs.kill_all(), vec![their_pid]);
        let _ = their_child.wait();
        assert!(wait_until_gone(their_pid, Duration::from_secs(10)));
        drop((my_registration, their_registration));
    }
}

//! The terminal lifecycle guard, preserved from the M6-era `tui.rs` rather
//! than rewritten: the key reader thread and TTY-presence watch that let an
//! orphaned session exit instead of spinning at ~80% of a core (issue #3),
//! and the termination-signal handling that restores the terminal on
//! SIGTERM/SIGHUP.
//!
//! The terminal can *disappear* — the emulator dies, the ssh session drops —
//! and a TUI whose pty is gone can never render again, so it leaves rather
//! than lingering: the loop watches for the hangup ([`TtyWatch`]), the key
//! reader treats the end of input as the end of its job ([`read_keys`]), and
//! shutdown never waits on that reader indefinitely ([`KeyReader::shutdown`]).
//! Each of those three is load-bearing on its own; together they are why an
//! orphaned session exits instead of spinning until somebody finds it with
//! SIGKILL.
//!
//! Those three are *composed* in [`TerminalProbes`] and driven by the
//! session loop in [`super`], and the composition is where the original bug
//! actually lived — a guard that exists but is not the tick the reader was
//! handed protects nothing — so the wiring is tested rather than the pieces
//! alone.

use std::time::Duration;

use ratatui::crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};

/// How long the key reader waits between polls before checking for shutdown.
pub const KEY_POLL: Duration = Duration::from_millis(200);

/// How often the loop checks that the terminal it is drawing on still exists.
/// One `open`/`close` of `/dev/tty` per interval (see [`TtyWatch`]).
pub const TTY_WATCH: Duration = Duration::from_millis(500);

/// How long shutdown waits for the key reader before leaving it behind.
pub const READER_JOIN_GRACE: Duration = Duration::from_secs(1);

/// The key reader thread, and the only two things the loop does with it:
/// start it, and stop waiting for it.
///
/// It is a type rather than four locals in the session loop because the
/// *shutdown* is the part that has to stay bounded (issue #3), and a bound
/// spelled out inline is a bound no test can hold on to: `shutdown` is both
/// what the loop calls and what `shutdown_leaves_a_wedged_reader_behind`
/// drives.
pub struct KeyReader {
    /// Asks the reader to stop at its next turn.
    stop: std::sync::mpsc::Sender<()>,
    /// Disconnects when the thread ends, however it ends.
    done: std::sync::mpsc::Receiver<()>,
    /// Joined only once `done` says joining cannot block.
    thread: std::thread::JoinHandle<()>,
}

impl KeyReader {
    /// Start reading keys off `tick` into `keys`.
    pub fn spawn(
        mut tick: impl FnMut() -> ReaderTick + Send + 'static,
        keys: tokio::sync::mpsc::UnboundedSender<KeyCode>,
    ) -> Self {
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        let thread = std::thread::spawn(move || {
            // Dropped when the thread ends, however it ends: that drop — not
            // a line the thread has to reach — is what `join_reader` waits on.
            let _done = done_tx;
            read_keys(&mut tick, &stop_rx, &keys);
        });
        Self {
            stop: stop_tx,
            done: done_rx,
            thread,
        }
    }

    /// Ask the reader to stop and wait only as long as [`READER_JOIN_GRACE`].
    pub fn shutdown(self) -> ReaderExit {
        let _ = self.stop.send(());
        let exit = join_reader(&self.done, READER_JOIN_GRACE);
        if exit == ReaderExit::Finished {
            let _ = self.thread.join();
        }
        exit
    }
}

/// One turn of the key reader: what the terminal produced, or that it is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderTick {
    /// A key was pressed.
    Key(KeyCode),
    /// Nothing happened within the poll window.
    Idle,
    /// The terminal ended — end of input, or a read that failed. Terminal in
    /// both senses: there is no later tick worth asking for.
    Ended,
}

/// The key reader's loop, over any source of ticks.
///
/// Written against a tick function rather than against crossterm directly so
/// the one property that matters here is testable without a terminal:
/// `Ended` **returns**. A reader that treats a dead input as "nothing yet"
/// and asks again is the shape that burned ~80% of a core in an orphaned TUI
/// (issue #3), and it is a shape a `continue` reintroduces in one character.
pub fn read_keys(
    mut tick: impl FnMut() -> ReaderTick,
    stop: &std::sync::mpsc::Receiver<()>,
    keys: &tokio::sync::mpsc::UnboundedSender<KeyCode>,
) {
    loop {
        // Asked to stop, or the loop that asked is gone: either way, done.
        if !matches!(stop.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)) {
            return;
        }
        match tick() {
            ReaderTick::Key(key) => {
                if keys.send(key).is_err() {
                    return;
                }
            }
            ReaderTick::Idle => {}
            ReaderTick::Ended => return,
        }
    }
}

/// What the session touches the terminal *with*: the loop's own hangup check
/// and the key reader's tick, built together so they cannot disagree.
///
/// The two production leaves — the `/dev/tty` probe and crossterm's poll —
/// are the only parts of the hangup path a test cannot drive, and they are
/// the only parts this type does not settle. Everything above them (the guard
/// runs *before* the poll; the loop's watch and the reader's guard read the
/// same probe) is [`TerminalProbes::over`], which the suite drives with a
/// scripted probe and a poll that fails the test if it is ever reached. That
/// wiring is what issue #3 turned on: the spin was a reader handed an
/// *unguarded* tick, which is not a property of the guard function — it is a
/// property of what the reader is handed.
pub struct TerminalProbes {
    /// The loop's periodic check that the terminal is still there.
    pub watch: TtyWatch,
    /// The reader thread's tick: that same check, then a poll.
    pub tick: Box<dyn FnMut() -> ReaderTick + Send>,
}

impl TerminalProbes {
    /// The real terminal: `/dev/tty` and crossterm.
    pub fn production() -> Self {
        Self::over(controlling_terminal_present, crossterm_poll)
    }

    /// The composition, over any probe and any poll.
    pub fn over(probe: fn() -> bool, poll: fn() -> ReaderTick) -> Self {
        let watch = TtyWatch::watching(probe);
        Self {
            watch,
            tick: Box::new(move || guarded_tick(&watch, poll)),
        }
    }
}

/// [`read_keys`]'s tick: one poll, guarded by the terminal probe.
///
/// The guard is not belt-and-braces. crossterm's own reader cannot report the
/// hangup — it takes the endless zero-length read a dead pty produces as "no
/// data yet" and polls again, inside `event::poll`, which therefore never
/// returns (see [`TtyWatch`] for the measurement). So the check has to happen
/// *before* the call, on this side of the library.
pub fn guarded_tick(tty: &TtyWatch, poll: fn() -> ReaderTick) -> ReaderTick {
    if tty.hung_up() {
        return ReaderTick::Ended;
    }
    poll()
}

/// The production poll: one crossterm turn, unguarded.
pub fn crossterm_poll() -> ReaderTick {
    match event::poll(KEY_POLL) {
        Ok(true) => match event::read() {
            Ok(TermEvent::Key(key)) if key.kind == KeyEventKind::Press => ReaderTick::Key(key.code),
            Ok(_) => ReaderTick::Idle,
            Err(_) => ReaderTick::Ended,
        },
        Ok(false) => ReaderTick::Idle,
        Err(_) => ReaderTick::Ended,
    }
}

/// What waiting for the key reader found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderExit {
    /// The thread returned; its handle can be joined immediately.
    Finished,
    /// It did not return in time and is left to die with the process.
    Abandoned,
}

/// Wait for the key reader to finish — but only for `grace`.
///
/// **Rung note (R1).** The signal is the *drop* of the sender the reader
/// thread owns, so this resolves the moment the thread returns, however it
/// returns, and never depends on it reaching a particular line. A bare
/// `join()` cannot be bounded, and an unbounded join is how an orphaned TUI
/// came to ignore SIGTERM: the shutdown path parked in `reader.join()` behind
/// a thread spinning inside crossterm, and only SIGKILL ended it (issue #3).
/// Leaving a doomed thread behind at exit costs nothing — the process is on
/// its way out and the thread holds nothing but its own stack — while waiting
/// for it costs a core, indefinitely. The bound belongs here even once the
/// spin itself is fixed: it is what keeps the *next* reader bug from becoming
/// an unkillable process.
pub fn join_reader(done: &std::sync::mpsc::Receiver<()>, grace: Duration) -> ReaderExit {
    match done.recv_timeout(grace) {
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => ReaderExit::Finished,
        _ => ReaderExit::Abandoned,
    }
}

/// Whether this process still has the terminal it started with.
///
/// **Measured, not assumed (L1).** When the terminal emulator goes away —
/// `tmux kill-server`, a dropped ssh session — the pty *master* closes while
/// the slave this process holds stays open, and on Linux that combination
/// reads as end-of-file forever rather than as an error: `read()` returns 0
/// every time (measured in this container, alongside `write()` → `EIO` and
/// `open("/dev/tty")` → `ENXIO`). crossterm reads a zero-length read as "no
/// data yet" and immediately polls again, so `event::poll`/`event::read` never
/// return and the reader thread spins at ~80% of a core from the moment the
/// pty dies — before any signal is involved (P1-PERF S7, issue #3).
///
/// The `ENXIO` half of that measurement is the probe: opening the controlling
/// terminal is cheap, needs no dependency this crate does not already have,
/// and — unlike reading — consumes nothing the key reader is waiting for.
///
/// Only a *transition* counts. A process that never had a controlling
/// terminal must not be told that its terminal vanished, so the state
/// recorded at install is half of every later answer.
#[derive(Debug, Clone, Copy)]
pub struct TtyWatch {
    had_tty: bool,
    /// How "is the terminal there?" is answered. A field rather than a direct
    /// call so the whole hangup path can be driven by a scripted terminal;
    /// production always passes [`controlling_terminal_present`].
    probe: fn() -> bool,
}

impl TtyWatch {
    /// Record the starting state, as `probe` sees it.
    pub fn watching(probe: fn() -> bool) -> Self {
        Self {
            had_tty: probe(),
            probe,
        }
    }

    /// The starting state, stated outright — the truth table's fixture.
    #[cfg(test)]
    pub fn from_probe(had_tty: bool) -> Self {
        Self {
            had_tty,
            probe: controlling_terminal_present,
        }
    }

    /// Whether the terminal this session started with has hung up.
    pub fn hung_up(&self) -> bool {
        self.decide((self.probe)())
    }

    /// [`TtyWatch::hung_up`]'s rule as a function of the probe — the half
    /// that can be tested without arranging a dying pty.
    pub fn decide(&self, present_now: bool) -> bool {
        self.had_tty && !present_now
    }
}

/// `ENXIO` — the errno the hangup measurement produced, and the only one that
/// means the terminal went away.
///
/// Spelled as its number because this crate has no libc dependency and is not
/// buying one for a constant: 6 on Linux and on every BSD including macOS.
#[cfg(unix)]
pub const ENXIO: i32 = 6;

/// Whether `/dev/tty` — this process's controlling terminal — can be opened.
#[cfg(unix)]
pub fn controlling_terminal_present() -> bool {
    terminal_present_at("/dev/tty")
}

/// The probe over an explicit path, so the rule below is measurable against a
/// failure a test can arrange.
///
/// **Only `ENXIO` counts** (L1: the discriminator is the measured one, not
/// "the open failed"). A live terminal can refuse to open for reasons that
/// are about this process rather than about the terminal — `EMFILE`/`ENFILE`
/// from a full descriptor table, `ENOMEM` — and reading those as a hangup
/// would end a perfectly good session silently, with exit 0 and nothing said,
/// which is the failure this whole path exists to prevent rather than to
/// cause.
#[cfg(unix)]
pub fn terminal_present_at(path: &str) -> bool {
    match std::fs::File::open(path) {
        Ok(_) => true,
        Err(e) => e.raw_os_error() != Some(ENXIO),
    }
}

/// No equivalent probe off Unix; the watch simply never fires there.
#[cfg(not(unix))]
pub fn controlling_terminal_present() -> bool {
    true
}

/// The termination signals the TUI must not die under without restoring the
/// terminal.
///
/// **Rung note (R1).** The loop *returns* on one of these rather than
/// restoring and re-raising the signal with its default disposition. Re-raising
/// is the more correct shell citizenship — it makes `$?` report `143` and lets
/// a parent see "killed by SIGTERM" — but it needs `libc` (or `signal-hook`),
/// a dependency the M6 budget does not name, to buy an exit code. Restoring
/// the terminal is the contracted property, and this gets it with what is
/// already here.
pub struct TerminationSignals {
    #[cfg(unix)]
    term: Option<tokio::signal::unix::Signal>,
    #[cfg(unix)]
    hangup: Option<tokio::signal::unix::Signal>,
}

impl TerminationSignals {
    pub fn install() -> Self {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            // A handler that cannot be installed is not fatal: the TUI still
            // works, it just loses this protection, and taking the session
            // down over it would be the worse trade.
            Self {
                term: signal(SignalKind::terminate()).ok(),
                hangup: signal(SignalKind::hangup()).ok(),
            }
        }
        #[cfg(not(unix))]
        {
            Self {}
        }
    }

    /// Resolve when a termination signal arrives; park forever where there
    /// are none to listen for, so the `select!` arm is simply never taken.
    pub async fn terminated(&mut self) {
        #[cfg(unix)]
        {
            match (self.term.as_mut(), self.hangup.as_mut()) {
                (Some(term), Some(hangup)) => {
                    tokio::select! {
                        _ = term.recv() => {}
                        _ = hangup.recv() => {}
                    }
                }
                (Some(one), None) | (None, Some(one)) => {
                    one.recv().await;
                }
                (None, None) => std::future::pending().await,
            }
        }
        #[cfg(not(unix))]
        std::future::pending().await
    }
}

/// Serializes every test in this crate that sends this process a real
/// signal or installs [`TerminationSignals`] and drives an `event_loop`
/// (`super::super::tests`, in `mod.rs`) — a stray SIGTERM reaches *every*
/// handler installed in this process, so two such tests running at once
/// could end the wrong one's loop through the wrong arm. Cheaper and more
/// honest than tolerating either outcome in the assertion.
#[cfg(test)]
pub(crate) static SIGNALS_QUIET: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(test)]
mod tests {
    use super::*;

    /// The signal arm, exercised for real: install the handlers, send this
    /// process a SIGTERM, and require the future to resolve.
    ///
    /// Not a name-based check — the point is that the handler is *installed*,
    /// because the default disposition for SIGTERM terminates the process,
    /// and a TUI terminated that way leaves the terminal in raw mode on the
    /// alternate screen (the one failure the contract names by hand). If the
    /// install silently stopped happening, this test would not merely fail:
    /// the signal would kill the test binary, which is the same evidence.
    #[cfg(unix)]
    #[tokio::test]
    async fn a_termination_signal_is_caught_so_the_terminal_can_be_restored() {
        let _quiet = SIGNALS_QUIET.lock().await;
        let mut signals = TerminationSignals::install();
        let status = std::process::Command::new("kill")
            .arg("-TERM")
            .arg(std::process::id().to_string())
            .status()
            .expect("send SIGTERM to this process");
        assert!(status.success(), "the test could not signal itself");
        tokio::time::timeout(Duration::from_secs(10), signals.terminated())
            .await
            .expect("SIGTERM must end the TUI's loop, not the process");
    }

    /// The end of the terminal ends the reader thread — it never asks again.
    ///
    /// The regression this pins is issue #3's first half. An orphaned TUI's
    /// reader sat on a pty whose master had closed, where every read returns
    /// end-of-file immediately and forever; treating that as "nothing yet"
    /// and looping is what turned an idle TUI (0.03% CPU, measured) into one
    /// burning ~80% of a core, from the instant the pty died and regardless of
    /// any signal. The scripted tick panics if it is called after saying
    /// `Ended`, so a regression fails the test instead of hanging it.
    #[test]
    fn the_key_reader_stops_when_the_terminal_ends() {
        let (keys_tx, mut keys) = tokio::sync::mpsc::unbounded_channel::<KeyCode>();
        // Kept alive: a dropped sender is itself a stop, and this test is
        // about the *tick* ending the loop.
        let (_stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();

        let script = [
            ReaderTick::Idle,
            ReaderTick::Key(KeyCode::Char('j')),
            ReaderTick::Ended,
        ];
        let mut calls = 0usize;
        read_keys(
            || {
                assert!(
                    calls < script.len(),
                    "the reader polled a terminal that had already ended ({} calls)",
                    calls + 1
                );
                let tick = script[calls];
                calls += 1;
                tick
            },
            &stop_rx,
            &keys_tx,
        );
        assert_eq!(calls, script.len(), "every tick was consumed, and no more");
        assert_eq!(keys.try_recv().ok(), Some(KeyCode::Char('j')));
        assert!(keys.try_recv().is_err(), "only the key was forwarded");

        // The other two ways the loop is over, for completeness: a stop
        // request, and an event loop that has dropped its receiver.
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        stop_tx.send(()).expect("request stop");
        read_keys(
            || panic!("a stopped reader must not poll at all"),
            &stop_rx,
            &keys_tx,
        );
        drop(keys);
        let (_stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        let mut sends = 0usize;
        read_keys(
            || {
                sends += 1;
                assert!(sends < 3, "a closed key channel must end the reader");
                ReaderTick::Key(KeyCode::Char('k'))
            },
            &stop_rx,
            &keys_tx,
        );
        assert_eq!(sends, 1, "the first failed send is the end");
    }

    /// Shutdown is bounded even when the reader will not come back.
    ///
    /// Issue #3's second half: the loop ended on SIGTERM exactly as designed,
    /// and then parked forever in `reader.join()` behind a thread spinning
    /// inside crossterm, so the orphan ignored SIGTERM and needed SIGKILL.
    /// The bound is what makes a *future* reader bug a slow exit instead of an
    /// unkillable process, so it is pinned separately from the spin itself.
    #[test]
    fn shutdown_does_not_park_on_a_reader_that_will_not_finish() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::Instant;

        // A reader that returns: the wait ends as soon as it does, and its
        // handle can then be joined for real.
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        let finished = std::thread::spawn(move || {
            let _done = done_tx;
        });
        assert_eq!(
            join_reader(&done_rx, Duration::from_secs(10)),
            ReaderExit::Finished,
            "a reader that ends is waited for, not abandoned"
        );
        finished.join().expect("the finished reader joins at once");

        // A reader that does not: the wait is over within the grace period,
        // and the caller is free to exit.
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        let wedged = Arc::new(AtomicBool::new(true));
        let held = Arc::clone(&wedged);
        let spinner = std::thread::spawn(move || {
            let _done = done_tx;
            while held.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let grace = Duration::from_millis(200);
        let started = Instant::now();
        assert_eq!(
            join_reader(&done_rx, grace),
            ReaderExit::Abandoned,
            "a wedged reader is left behind"
        );
        let waited = started.elapsed();
        assert!(
            waited < grace * 10,
            "shutdown waited {waited:?} on a reader that never finishes — \
             an unbounded join here is what needed SIGKILL"
        );
        wedged.store(false, Ordering::Relaxed);
        spinner.join().expect("release the helper thread");
    }

    /// The hangup rule: only a terminal that *was* there can go away.
    ///
    /// Both halves matter. Missing the transition leaves the orphan spinning;
    /// firing on a process that never had a controlling terminal would end a
    /// session that is working perfectly well.
    #[test]
    fn only_a_terminal_that_was_there_can_hang_up() {
        assert!(
            TtyWatch::from_probe(true).decide(false),
            "a terminal that was there and is not is a hangup — the session must end"
        );
        assert!(
            !TtyWatch::from_probe(true).decide(true),
            "a live terminal is not a hangup"
        );
        assert!(
            !TtyWatch::from_probe(false).decide(false),
            "a session that never had a controlling terminal must not be ended by \
             the absence of one"
        );
        assert!(!TtyWatch::from_probe(false).decide(true));
    }

    /// The probe answers "gone" for the *measured* errno and nothing else.
    ///
    /// `open("/dev/tty")` → `ENXIO` is what the hangup was measured to produce
    /// (L1). Treating any failed open as a hangup adds failure modes the
    /// measurement never showed — a full descriptor table (`EMFILE`/`ENFILE`)
    /// or `ENOMEM` on a terminal that is perfectly alive — and each of them
    /// would end the session the quiet way, exit 0 with nothing said.
    #[cfg(unix)]
    #[test]
    fn only_the_measured_hangup_errno_says_the_terminal_is_gone() {
        assert!(
            terminal_present_at("/dev/null"),
            "a path that opens is present"
        );
        assert!(
            terminal_present_at("/proc/self/no-such-terminal-here"),
            "an open that failed for a reason other than the hangup errno is this \
             process's problem, not evidence that the terminal went away"
        );
        // And where this suite runs without a controlling terminal — the usual
        // case under `cargo test` — the real probe re-measures L1's claim end
        // to end rather than trusting the constant above.
        if let Err(e) = std::fs::File::open("/dev/tty") {
            assert_eq!(
                e.raw_os_error(),
                Some(ENXIO),
                "the absent controlling terminal must fail with the measured errno"
            );
            assert!(
                !controlling_terminal_present(),
                "and that errno must read as absent"
            );
        }
    }

    /// The reader is handed a tick that checks the terminal *before* polling.
    ///
    /// This is the wiring, not the guard: issue #3's spin was crossterm being
    /// asked to poll a pty whose master had closed, where `event::poll` never
    /// returns, so what matters is that the tick the reader thread actually
    /// receives refuses to make that call. The poll here fails the test if it
    /// is ever reached on a hung-up terminal, and is counted on a live one so
    /// the guard cannot pass by simply never polling at all.
    #[test]
    fn the_reader_is_handed_a_tick_that_never_polls_a_hung_up_terminal() {
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

        static PRESENT: AtomicBool = AtomicBool::new(true);
        static POLLS: AtomicUsize = AtomicUsize::new(0);
        fn probe() -> bool {
            PRESENT.load(Ordering::SeqCst)
        }
        fn poll() -> ReaderTick {
            POLLS.fetch_add(1, Ordering::SeqCst);
            ReaderTick::Idle
        }

        PRESENT.store(true, Ordering::SeqCst);
        POLLS.store(0, Ordering::SeqCst);
        let mut probes = TerminalProbes::over(probe, poll);

        assert_eq!(
            (probes.tick)(),
            ReaderTick::Idle,
            "a live terminal is polled as usual"
        );
        assert_eq!(POLLS.load(Ordering::SeqCst), 1, "…and the poll ran");

        PRESENT.store(false, Ordering::SeqCst);
        assert!(
            probes.watch.hung_up(),
            "the loop's own watch and the reader's guard read the same probe"
        );
        assert_eq!(
            (probes.tick)(),
            ReaderTick::Ended,
            "a hung-up terminal ends the reader"
        );
        assert_eq!(
            POLLS.load(Ordering::SeqCst),
            1,
            "the tick handed to the reader must not poll a terminal that hung up — \
             that call is the one that never returns, and the spin is what it costs"
        );
    }

    /// Shutdown does not wait on a reader that will not come back — as the
    /// loop calls it, not as a helper called nowhere.
    ///
    /// Issue #3's second half was exactly this wiring: `join_reader` can be
    /// perfectly bounded and the session still hang, because what shutdown
    /// ran was a bare `reader.join()`. So the assertion is on elapsed time
    /// through [`KeyReader::shutdown`], which is what the session loop calls: a
    /// reader wedged for five seconds must not hold the exit for five seconds.
    #[test]
    fn shutdown_leaves_a_wedged_reader_behind() {
        use std::time::Instant;

        // A reader that returns promptly is waited for and joined.
        let (keys_tx, _keys) = tokio::sync::mpsc::unbounded_channel::<KeyCode>();
        let reader = KeyReader::spawn(|| ReaderTick::Ended, keys_tx);
        assert_eq!(
            reader.shutdown(),
            ReaderExit::Finished,
            "a reader that ends is joined, not abandoned"
        );

        // A reader stuck inside its tick — crossterm's poll on a dead pty is
        // the real one — is left to die with the process.
        const WEDGE: Duration = Duration::from_secs(5);
        let (keys_tx, _keys) = tokio::sync::mpsc::unbounded_channel::<KeyCode>();
        let (entered_tx, entered) = std::sync::mpsc::channel::<()>();
        let reader = KeyReader::spawn(
            move || {
                let _ = entered_tx.send(());
                std::thread::sleep(WEDGE);
                ReaderTick::Ended
            },
            keys_tx,
        );
        // Shut down while the reader is *inside* its tick: a reader that has
        // not started yet stops on the request itself, which is not the case
        // this is about.
        entered
            .recv_timeout(Duration::from_secs(10))
            .expect("the reader reached its tick");
        let started = Instant::now();
        let exit = reader.shutdown();
        let waited = started.elapsed();
        assert_eq!(exit, ReaderExit::Abandoned);
        assert!(
            waited < WEDGE / 2,
            "shutdown waited {waited:?} on a wedged reader (grace is \
             {READER_JOIN_GRACE:?}) — an unbounded join here is what made the \
             orphaned TUI ignore SIGTERM"
        );
    }
}

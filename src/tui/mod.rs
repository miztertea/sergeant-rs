//! The terminal UI (`reference/proposal-tui-t-series.md`): `sgt` with no
//! subcommand.
//!
//! T1a's application shell and top-level navigation — `Home / Fleet /
//! Workflows / Estate` (§7.1) — replacing the M6-era minimal two-screen
//! Fleet/Detail TUI. The replacement is a restructure, not a rewrite: the
//! terminal lifecycle guard, reconnect/backoff, and key-reading machinery
//! below are preserved from that version rather than reinvented, now split
//! into their own modules ([`terminal`], [`connection`]) alongside the new
//! screen code ([`app`], [`home`], [`fleet`], [`workflows`], [`estate`],
//! [`work_view`], [`attention`], [`overlay`], [`theme`]).
//!
//! **It talks to the daemon and to nothing else.** The only crate module any
//! file under this tree imports is [`crate::api`] — the client and the small
//! view helpers every screen shares: no journal, no projection, no engine,
//! no daemon module. That is §30's architectural test rather than a style
//! rule —
//!
//! > If the TUI needs a private shortcut, the API is incomplete.
//!
//! — and it is why `Home`'s New Work form takes explicit repository names
//! rather than expanding a declared group the way `sgt run --group` does:
//! that expansion reads the estate manifest from local disk
//! (`crate::domain::workspace::Workspace::declared_groups_scoped`), which no
//! module here may reach for, and the daemon does not expose declared
//! groups over the API yet (T3's job, §20.4).
//!
//! Liveness is the same SSE tail every other client uses, resumed from the
//! `journal_head` the first fetch reported, so attaching does not replay
//! history. An event does not carry the screen's data — it says *something
//! changed*, and the screen re-reads the API. One source of truth, one shape
//! of state, no client-side reducer to drift.
//!
//! The terminal is restored on every exit path: [`ratatui::try_init`]
//! installs a panic hook that restores before printing, [`run`] restores in
//! one place on the way out regardless of how the loop ended, and SIGTERM /
//! SIGHUP end the loop through that same exit instead of killing the process
//! with the screen still in raw mode (see [`terminal::TerminationSignals`]).
//!
//! The terminal can also *disappear* — the emulator dies, the ssh session
//! drops — and a TUI whose pty is gone can never render again, so it leaves
//! rather than lingering: see [`terminal`] for the full account of why and
//! how.

mod app;
mod attention;
mod composer;
mod connection;
mod estate;
mod fleet;
mod home;
mod overlay;
mod terminal;
mod theme;
mod work_view;
mod workflows;

/// T4's geometry matrix, Taste pre-flight, and real-screenshot generator
/// (§19.10/§19.11/§20.5) — internal rather than `tests/*.rs` because most of
/// its fixtures need an open [`work_view::WorkScreen`], reachable only
/// through `work_view::WorkScreen::from_parts` (`pub(crate)`), not from an
/// external test crate.
#[cfg(test)]
#[path = "geometry_matrix_tests.rs"]
mod geometry_matrix_tests;

pub use app::{Action, App, Destination, OpenWork};
pub use fleet::WorkRow;

use std::time::Duration;

use ratatui::Frame;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::api::{ApiClient, ClientError};

use connection::{Backoff, Live, TailStep, next_event, reconnected, try_attach};
use terminal::{KeyReader, TerminalProbes, TerminationSignals};
use theme::{Tier, Token};

/// A TUI failure that reaches the caller.
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    /// The terminal could not be initialized, drawn to, or restored.
    #[error("terminal error: {0}")]
    Terminal(#[from] std::io::Error),
    /// The daemon could not be reached for the first paint. Failures *after*
    /// the UI is up are shown in the status line instead of ending the
    /// session: a daemon restart should not take the TUI down with it.
    #[error("{0}")]
    Api(#[from] ClientError),
}

// -------------------------------------------------------------- rendering

/// Draw the current screen. Pure: everything it paints is already in `app`.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let tier = Tier::for_size(area.width, area.height);
    if tier == Tier::TooSmall {
        render_too_small(frame, area);
        return;
    }

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(area);
    frame.render_widget(header_line(app), header);
    draw_body(frame, body, app, tier);
    frame.render_widget(footer_line(app), footer);

    if let Some(kind) = app.overlay {
        overlay::render(frame, body, kind, app);
    }
}

fn draw_body(frame: &mut Frame, area: Rect, app: &App, tier: Tier) {
    if app.open_work.is_some() {
        work_view::render(frame, area, app.work_screen.as_ref(), app.live);
        return;
    }
    match tier {
        Tier::Narrow => {
            // §18.3: the drawer is a full-body overlay at Narrow, replacing
            // the primary body rather than sharing it.
            if app.drawer_open {
                attention::render(frame, area, &app.rows);
            } else {
                draw_primary(frame, area, app, tier);
            }
        }
        Tier::Medium => {
            if app.drawer_open {
                let [drawer, primary] = Layout::horizontal([
                    Constraint::Length(theme::DRAWER_WIDTH_MEDIUM),
                    Constraint::Min(0),
                ])
                .areas(area);
                attention::render(frame, drawer, &app.rows);
                draw_primary(frame, primary, app, tier);
            } else {
                draw_primary(frame, area, app, tier);
            }
        }
        Tier::Wide => draw_wide(frame, area, app),
        Tier::TooSmall => unreachable!("draw() returns before reaching draw_body"),
    }
}

/// §18.1: drawer | primary body | contextual rail, all inline. The rail's
/// content is destination-specific and only defined for Home (Recent
/// Outputs, §9.4) and Fleet (selected Work preview, §10.1) — §12 names no
/// rail content for Estate, so it (like Workflows) has none.
fn draw_wide(frame: &mut Frame, area: Rect, app: &App) {
    let has_rail = matches!(
        app.destination,
        Destination::Home | Destination::Fleet | Destination::Workflows
    );
    // §8.10's spacing scale: `region` (3 cells) is the outer margin between
    // the drawer or contextual rail and the primary body at Wide.
    let gap = Constraint::Length(theme::spacing::REGION);
    let mut constraints = Vec::new();
    if app.drawer_open {
        constraints.push(Constraint::Length(theme::DRAWER_WIDTH_WIDE));
        constraints.push(gap);
    }
    constraints.push(Constraint::Min(0));
    if has_rail {
        constraints.push(gap);
        constraints.push(Constraint::Length(theme::RAIL_WIDTH_WIDE));
    }
    let chunks = Layout::horizontal(constraints).split(area);
    let mut next = 0usize;
    if app.drawer_open {
        attention::render(frame, chunks[next], &app.rows);
        next += 2;
    }
    draw_primary(frame, chunks[next], app, Tier::Wide);
    next += 1;
    if has_rail {
        next += 1;
        match app.destination {
            Destination::Home => home::render_recent_outputs(frame, chunks[next], &app.rows),
            Destination::Fleet => {
                work_view::render_preview(frame, chunks[next], app.fleet.selected_row(&app.rows))
            }
            Destination::Workflows => {
                workflows::render_recent_work(frame, chunks[next], &app.rows, &app.workflows)
            }
            Destination::Estate => {}
        }
    }
}

fn draw_primary(frame: &mut Frame, area: Rect, app: &App, tier: Tier) {
    let admission_paused = app.system["admission_paused"].as_bool().unwrap_or(false);
    match app.destination {
        Destination::Home => home::render(frame, area, &app.home, admission_paused),
        Destination::Fleet => fleet::render(frame, area, &app.rows, &app.fleet, tier),
        Destination::Workflows => workflows::render(frame, area, &app.workflows),
        Destination::Estate => estate::render(frame, area, &app.estate),
    }
}

/// §17.7: below the 80×24 floor, a safe minimal notice — no composition is
/// attempted, and nothing here can panic or overlap a footer that this
/// render never draws.
fn render_too_small(frame: &mut Frame, area: Rect) {
    let text = format!(
        "Terminal too small: {}x{}\nsergeant needs at least 80x24.\n\nResize the window, or press q to quit.",
        area.width, area.height
    );
    frame.render_widget(
        Paragraph::new(text)
            .style(
                Style::default()
                    .bg(Token::Background.rgb())
                    .fg(Token::Text.rgb()),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

/// §7.1's representative header: destinations, then `? N` / `! N` /
/// connection truth, with admission-paused named when the daemon reports it.
fn header_line(app: &App) -> Paragraph<'_> {
    let mut spans = vec![Span::styled(
        "sergeant  ",
        Style::default().add_modifier(Modifier::BOLD),
    )];
    for destination in Destination::ALL {
        let style = if destination == app.destination {
            Style::default()
                .fg(Token::Focus.rgb())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Token::Muted.rgb())
        };
        spans.push(Span::styled(format!("{}  ", destination.label()), style));
    }
    let needs_input = attention::needs_input_count(&app.rows);
    let stopped = attention::stopped_count(&app.rows);
    spans.push(Span::raw("   "));
    spans.push(Span::styled(
        format!("? {needs_input}"),
        Style::default().fg(Token::Attention.rgb()),
    ));
    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        format!("! {stopped}"),
        Style::default().fg(Token::Danger.rgb()),
    ));
    spans.push(Span::raw("   "));
    spans.push(Span::raw(app.live.label()));
    if app.system["admission_paused"].as_bool().unwrap_or(false) {
        spans.push(Span::styled(
            "   ADMISSION PAUSED",
            Style::default().fg(Token::Warning.rgb()),
        ));
    }
    Paragraph::new(Line::from(spans)).style(
        Style::default()
            .bg(Token::Surface.rgb())
            .fg(Token::Text.rgb()),
    )
}

fn footer_line(app: &App) -> Paragraph<'_> {
    let keys = if app.overlay.is_some() {
        "Esc/q close"
    } else if app.open_work.is_some() {
        // §15.1/§15.2: the composer's own grammar while `ANSWER` has focus,
        // else the surface's ordinary navigation plus §13.10's action keys.
        match app.work_screen.as_ref().map(|s| s.answer_focused()) {
            Some(true) => {
                "Ctrl+Enter send · Enter newline · Tab focus Send · Esc leave (draft kept)"
            }
            _ => {
                "Tab/S-Tab or 1-5 switch view · j/k move · r/c/t/e/p act (where offered) · Esc/q back"
            }
        }
    } else if app.destination == Destination::Home && app.home.wants_text_focus() {
        if app.home.intent_focused() {
            "Ctrl+Enter submit · Enter newline · Tab next field · Esc leave the form"
        } else {
            "Tab next field · Enter next field/submit · Esc leave the form"
        }
    } else if app.destination == Destination::Fleet && app.fleet.wants_text_focus() {
        "type to filter · Enter/Esc apply"
    } else {
        "1-4 destinations · Tab cycle · ~ drawer · ? help · q quit"
    };
    Paragraph::new(Line::from(vec![
        Span::styled(keys, Style::default().fg(Token::Muted.rgb())),
        Span::raw("  "),
        Span::styled(app.status.clone(), Style::default().fg(Token::Text.rgb())),
    ]))
    .style(Style::default().bg(Token::SurfaceMuted.rgb()))
}

// ------------------------------------------------------------- the session

/// Open the TUI against a daemon and run until the user quits.
///
/// The terminal is restored exactly once, on the way out, whatever ended the
/// loop; `ratatui::try_init` covers the panic path with its own hook.
pub async fn run(client: ApiClient) -> Result<(), TuiError> {
    let mut app = App::new();
    // The first read happens before the terminal is touched: a daemon that
    // cannot answer should print an error to a normal terminal, not paint an
    // empty UI over an alternate screen.
    app.refresh(&client).await?;
    app.status = String::new();

    let mut terminal = ratatui::try_init()?;
    // §8.8's Decision T2-32: opportunistic, nonfatal — paired with the pop
    // below on every exit path, the same way raw mode itself is restored on
    // every path regardless of how the loop ended.
    terminal::push_keyboard_enhancement(&mut std::io::stdout());
    let result = event_loop(
        &mut terminal,
        &mut app,
        &client,
        TerminalProbes::production(),
    )
    .await;
    terminal::pop_keyboard_enhancement(&mut std::io::stdout());
    // Restore is attempted on every path, including the hung-up one — the
    // ioctl half (raw mode) can still land even when the write half cannot.
    let restored = ratatui::try_restore();
    finish(result, restored)
}

/// What the process answers, given how the loop ended and whether the
/// terminal could be restored.
///
/// A function rather than a `match` inside [`run`] because the interesting
/// rule — a hung-up terminal is *not* an error — is otherwise reachable only
/// through a real terminal, i.e. by nothing the suite can run.
fn finish(result: Result<Exit, TuiError>, restored: std::io::Result<()>) -> Result<(), TuiError> {
    match result {
        // The terminal is gone: there is nothing left to restore it *for*,
        // and nothing to report the failure *to* — both halves of the exit
        // are writes to a descriptor that now answers EIO. Leaving quietly is
        // the whole remedy; the alternative is a process that outlives its
        // screen (issue #3), and an error printed to a descriptor nobody can
        // read is not a report.
        Ok(Exit::TerminalGone) => Ok(()),
        Ok(Exit::Requested) => restored.map_err(TuiError::from),
        Err(e) => Err(e),
    }
}

/// Why the session ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Exit {
    /// The user left, or a termination signal asked for the same thing.
    Requested,
    /// The terminal itself went away (see [`terminal::TtyWatch`]).
    TerminalGone,
}

/// The session loop.
///
/// Generic over the backend, and handed its terminal probes rather than
/// building them, so the whole loop — including the exit paths a hangup
/// takes — runs under a `TestBackend` with a scripted terminal. Production
/// passes [`TerminalProbes::production`]; that composition is itself tested
/// (see `TerminalProbes::over`).
async fn event_loop<B>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut App,
    client: &ApiClient,
    probes: TerminalProbes,
) -> Result<Exit, TuiError>
where
    B: ratatui::backend::Backend,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    let (keys_tx, mut keys) = tokio::sync::mpsc::unbounded_channel::<KeyEvent>();
    let TerminalProbes { watch: tty, tick } = probes;
    // crossterm's reader is blocking, so it lives on its own OS thread and
    // posts keystrokes into the async loop. It polls rather than blocking
    // forever so it notices the loop going away, and its tick checks the
    // terminal is still there before every poll so it never hands a hung-up
    // pty to a library that would spin on it.
    let reader = KeyReader::spawn(tick, keys_tx);

    let mut backoff = Backoff::new();
    let mut stream = reconnected(
        client,
        app,
        &mut backoff,
        try_attach(client, app.last_seq).await,
    )
    .await;
    // A resettable one-shot timer for the next automatic reconnect attempt,
    // reused across loop iterations rather than recreated (the standard
    // shape for a `Sleep` awaited from inside `select!`). Its `select!` arm
    // below is guarded on `Live::Reconnecting`, so it is simply never polled
    // while the tail is up or while it has stopped retrying for good on an
    // auth failure — the same "disarm rather than fire uselessly" idea
    // `TerminalProbes` uses for signals nobody installed.
    let reconnect_at = tokio::time::sleep(Duration::ZERO);
    tokio::pin!(reconnect_at);
    if app.live == Live::Reconnecting {
        reconnect_at
            .as_mut()
            .reset(tokio::time::Instant::now() + backoff.next_delay());
    }
    let mut signals = TerminationSignals::install();
    let mut watch = tokio::time::interval(terminal::TTY_WATCH);

    let mut outcome: Result<Exit, TuiError> = Ok(Exit::Requested);
    'session: {
        if let Err(e) = terminal.draw(|frame| draw(frame, app)) {
            // Same rule as the in-loop draw below: a write that fails because
            // the pty already hung up is `TerminalGone`, not a reported
            // error — this is the one draw site a hangup landing before the
            // event loop starts would otherwise slip past.
            outcome = if tty.hung_up() {
                Ok(Exit::TerminalGone)
            } else {
                Err(terminal_error(e))
            };
            break 'session;
        }
        loop {
            let mut needs_refresh = false;
            tokio::select! {
                key = keys.recv() => {
                    let Some(key) = key else { break 'session };
                    let action = app.on_key(key);
                    match app.execute(client, action).await {
                        Action::Quit => break 'session,
                        Action::Refresh => {
                            needs_refresh = true;
                            // A refresh is the user asking for the truth, so it
                            // re-attaches the tail as well as re-reading: an `r`
                            // that repainted stale data and left the screen
                            // permanently detached would answer the wrong
                            // question. This is the same attempt the backoff
                            // timer below makes on its own schedule.
                            if stream.is_none() {
                                let attempt = try_attach(client, app.last_seq).await;
                                stream = reconnected(client, app, &mut backoff, attempt).await;
                                if app.live == Live::Reconnecting {
                                    reconnect_at.as_mut().reset(
                                        tokio::time::Instant::now() + backoff.next_delay(),
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                    if app.quit {
                        break 'session;
                    }
                }
                event = next_event(&mut stream) => {
                    match event {
                        TailStep::Event(event) => needs_refresh = app.observe(&event),
                        TailStep::Ended => {
                            // The tail ended (daemon restart or shutdown, a lagged
                            // subscriber the server dropped, a broken chunk).
                            // Keep the UI alive and say so durably, and start
                            // retrying with backoff instead of waiting on a
                            // keystroke (issue #16).
                            stream = None;
                            app.live = Live::Reconnecting;
                            app.status = "live tail closed — reconnecting…".to_string();
                            reconnect_at
                                .as_mut()
                                .reset(tokio::time::Instant::now() + backoff.next_delay());
                        }
                        TailStep::Malformed(detail) => {
                            // The daemon sent a frame this client could not
                            // decode, handled deliberately rather than falling
                            // through to the same message as an ordinary
                            // disconnect — but it still gets the same automatic
                            // reconnect, since whatever answers the next attempt
                            // need not be the process that sent the bad frame.
                            stream = None;
                            app.live = Live::Reconnecting;
                            app.status =
                                format!("live tail error: {detail} — reconnecting…");
                            reconnect_at
                                .as_mut()
                                .reset(tokio::time::Instant::now() + backoff.next_delay());
                        }
                    }
                }
                // The other half of the auto-reconnect: retried on its own
                // schedule rather than only when the user asks. The `if` guard
                // is what disarms this arm once the tail is live again or has
                // stopped retrying for good on an auth failure.
                _ = &mut reconnect_at, if app.live == Live::Reconnecting => {
                    let attempt = try_attach(client, app.last_seq).await;
                    stream = reconnected(client, app, &mut backoff, attempt).await;
                    if app.live == Live::Reconnecting {
                        reconnect_at
                            .as_mut()
                            .reset(tokio::time::Instant::now() + backoff.next_delay());
                    }
                }
                // A terminal left in raw mode + alternate screen is the one
                // failure this module's contract names by hand, and default
                // signal disposition is the path that produces it: SIGTERM from a
                // process manager, SIGHUP from a closing terminal emulator. Both
                // end the loop through the same exit as `q`, so `run`'s single
                // restore runs. (SIGINT is not here: raw mode delivers Ctrl-C as a
                // key event, which the keymap already handles.)
                _ = signals.terminated() => {
                    break 'session;
                }
                // The terminal can also leave without saying anything this
                // loop would otherwise notice: a screen nobody can see is a
                // session that must end, not one that idles forever holding a
                // spinning reader thread.
                _ = watch.tick() => {
                    if tty.hung_up() {
                        outcome = Ok(Exit::TerminalGone);
                        break 'session;
                    }
                }
            }
            // §12.1: a repo add kicked off the render loop is polled every
            // tick, key or not — a slow `git clone` must finish on its own
            // schedule, not wait for the next keystroke.
            if app.poll_background() {
                needs_refresh = true;
            }
            if needs_refresh && let Err(e) = app.refresh(client).await {
                app.status = format!("refresh failed: {e}");
            }
            if let Err(e) = terminal.draw(|frame| draw(frame, app)) {
                // A write that fails because the pty hung up is that same
                // exit, discovered by the drawing half instead of the watch.
                outcome = if tty.hung_up() {
                    Ok(Exit::TerminalGone)
                } else {
                    Err(terminal_error(e))
                };
                break 'session;
            }
        }
    }

    // The receiver goes first: a reader parked on a send ends on the closed
    // channel even before it is asked to stop.
    drop(keys);
    let _ = reader.shutdown();
    outcome
}

/// A backend's own draw failure, as this module's error.
///
/// Every ratatui backend names its own error type — crossterm's is
/// `io::Error`, the test backend's cannot be constructed at all — so the loop
/// converts instead of assuming one.
fn terminal_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> TuiError {
    TuiError::Terminal(std::io::Error::other(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyCode;
    use serde_json::json;

    /// The header line as the text a terminal would show — rendered into a
    /// buffer, not `Debug`-printed, so the assertion is about the screen.
    fn header_text(app: &App) -> String {
        use ratatui::buffer::Buffer;
        use ratatui::widgets::Widget;
        let area = Rect::new(0, 0, 200, 1);
        let mut buffer = Buffer::empty(area);
        header_line(app).render(area, &mut buffer);
        (0..area.width)
            .map(|x| buffer.cell((x, 0)).map_or(" ", |cell| cell.symbol()))
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    /// The whole screen as the text a terminal would show, at a Wide size so
    /// every pane in the composition is on screen at once.
    fn screen_text(app: &App) -> String {
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(200, 44))
            .expect("a test terminal");
        terminal
            .draw(|frame| draw(frame, app))
            .expect("draw the screen");
        let buffer = terminal.backend().buffer().clone();
        let area = buffer.area;
        (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buffer.cell((x, y)).map_or(" ", |cell| cell.symbol()))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn the_header_names_every_destination_and_the_current_one_is_marked() {
        let app = App::new();
        let header = header_text(&app);
        for label in ["Home", "Fleet", "Workflows", "Estate"] {
            assert!(header.contains(label), "header must list {label}: {header}");
        }
        assert!(
            header.contains("live"),
            "a fresh app assumes the tail: {header}"
        );
    }

    #[test]
    fn the_header_counts_needs_input_and_stopped_work() {
        let mut app = App::new();
        app.rows = fleet::fleet_rows(&json!({"works": [
            {"id": "a", "state": "needs_input", "intent": "x"},
            {"id": "b", "state": "blocked", "intent": "y"},
            {"id": "c", "state": "failed", "intent": "z"},
        ]}));
        let header = header_text(&app);
        assert!(header.contains("? 1"), "one needs_input: {header}");
        assert!(
            header.contains("! 2"),
            "two stopped (blocked+failed): {header}"
        );
    }

    /// Liveness is durable state, not a message.
    ///
    /// The regression: the only sign that the SSE tail had died was
    /// `App::status`, which `execute` overwrites with the next command's
    /// outcome — leaving a screen that looks live, is not, and never
    /// reconnects.
    #[tokio::test]
    async fn a_dead_tail_stays_visible_after_the_next_command() {
        let mut app = App::new();
        assert_eq!(app.live, Live::Attached, "a fresh app assumes the tail");
        assert!(header_text(&app).contains("live"));

        app.live = Live::Reconnecting;
        app.status = "live tail closed — reconnecting…".to_string();
        assert!(header_text(&app).contains("RECONNECTING"));

        // Any later command outcome replaces the status line…
        let client = ApiClient::new("http://127.0.0.1:1", "t").expect("client");
        let action = app
            .execute(
                &client,
                Action::Submit(json!({"intent": "x", "command_id": "c"})),
            )
            .await;
        assert_eq!(action, Action::Refresh);
        assert!(
            !app.status.contains("live tail closed"),
            "the status line is the command's now: {}",
            app.status
        );
        // …and the header still says the screen is not live.
        assert_eq!(app.live, Live::Reconnecting);
        assert!(header_text(&app).contains("RECONNECTING"));
    }

    #[test]
    fn below_the_floor_the_screen_is_the_too_small_notice_and_nothing_else() {
        let app = App::new();
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(79, 24))
            .expect("a test terminal");
        terminal.draw(|frame| draw(frame, &app)).expect("draw");
        let buffer = terminal.backend().buffer().clone();
        let text: String = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.cell((x, y)).map_or(" ", |cell| cell.symbol()))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("too small"), "must name the problem: {text}");
        assert!(text.contains("79x24"), "must name the actual size: {text}");
    }

    #[test]
    fn wide_home_shows_the_drawer_the_form_and_recent_outputs_at_once() {
        let mut app = App::new();
        app.home.intent.set_text("deliberately unused");
        app.rows = fleet::fleet_rows(&json!({"works": [
            {"id": "needs-a-look", "state": "needs_input", "intent": "waiting on an answer"},
            {"id": "shipped", "state": "completed", "intent": "already done"},
        ]}));
        let text = screen_text(&app);
        assert!(text.contains("Attention"), "the drawer pane: {text}");
        assert!(
            text.contains("waiting on an answer"),
            "drawer row content: {text}"
        );
        assert!(text.contains("Home / New Work"), "the form pane: {text}");
        assert!(text.contains("Recent Outputs"), "the rail pane: {text}");
        assert!(
            text.contains("already done"),
            "recent output content: {text}"
        );
    }

    #[test]
    fn fleet_enter_asks_to_open_the_work_surface() {
        let mut app = App::new();
        app.rows = fleet::fleet_rows(&json!({"works": [
            {"id": "01ABC", "state": "running", "intent": "an in-flight thing"},
        ]}));
        app.on_key(KeyCode::Esc);
        app.on_key(KeyCode::Char('2'));
        assert_eq!(
            app.on_key(KeyCode::Enter),
            Action::OpenWork("01ABC".to_string())
        );
    }

    /// Once `App::execute` has fetched a Work (T1b's real surface, not a
    /// stub), the canonical Work surface draws its Thread tab by default,
    /// with the Work's identity and intent on screen and the tab bar naming
    /// every §13.2 view.
    #[test]
    fn an_open_work_surface_draws_the_thread_tab_and_the_full_view_bar() {
        let mut app = App::new();
        app.rows = fleet::fleet_rows(&json!({"works": [
            {"id": "01ABC", "state": "running", "intent": "an in-flight thing"},
        ]}));
        app.open_work = Some(OpenWork {
            id: "01ABC".to_string(),
            from: Destination::Fleet,
        });
        app.work_screen = Some(work_view::WorkScreen::from_parts(
            "01ABC".to_string(),
            json!({"work": {"id": "01ABC", "intent": "an in-flight thing", "state": "running"}}),
            Vec::new(),
            Vec::new(),
            None,
        ));
        let text = screen_text(&app);
        assert!(text.contains("01ABC"), "the opened work's id: {text}");
        assert!(text.contains("an in-flight thing"), "its intent: {text}");
        for tab in ["Thread", "Workflow", "Evidence", "Graph", "Details"] {
            assert!(text.contains(tab), "the tab bar must name {tab}: {text}");
        }
    }

    /// The loop ends when the terminal goes away, without needing the reader
    /// to notice — driven through `event_loop` itself.
    ///
    /// The reader cannot be relied on for this: crossterm's poll on a dead pty
    /// does not return, so the tick here keeps saying "nothing yet" exactly as
    /// a wedged one would. The watch arm is the half that has to fire, and
    /// `TerminalGone` is the exit it must produce.
    #[cfg(unix)]
    #[tokio::test]
    async fn a_hangup_ends_the_session_through_the_watch() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let _quiet = terminal::SIGNALS_QUIET.lock().await;
        static PRESENT: AtomicBool = AtomicBool::new(true);
        fn probe() -> bool {
            PRESENT.load(Ordering::SeqCst)
        }

        PRESENT.store(true, Ordering::SeqCst);
        let watch = terminal::TtyWatch::watching(probe);
        PRESENT.store(false, Ordering::SeqCst);
        let probes = TerminalProbes {
            watch,
            tick: Box::new(|| {
                std::thread::sleep(Duration::from_millis(20));
                terminal::ReaderTick::Idle
            }),
        };

        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 24))
            .expect("a test terminal");
        let mut app = App::new();
        // Nothing is listening there: the tail simply fails to attach, which
        // is a state the screen already knows how to be in.
        let client = ApiClient::new("http://127.0.0.1:1", "token").expect("client");

        let exit = tokio::time::timeout(
            Duration::from_secs(20),
            event_loop(&mut terminal, &mut app, &client, probes),
        )
        .await
        .expect("a session whose terminal is gone must end, not idle on forever");
        assert_eq!(
            exit.expect("a hangup is not a failure"),
            Exit::TerminalGone,
            "the loop must name the hangup, so `run` knows there is nobody to report to"
        );
    }

    /// A backend whose `draw` always fails, wrapping a real `TestBackend` for
    /// every other call. `TestBackend` itself cannot fail a draw (its error
    /// type is `Infallible`), so a hangup landing on the very first,
    /// pre-loop draw is otherwise unreachable from a test.
    struct FailingBackend(ratatui::backend::TestBackend);

    impl ratatui::backend::Backend for FailingBackend {
        type Error = std::io::Error;

        fn draw<'a, I>(&mut self, _content: I) -> std::io::Result<()>
        where
            I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
        {
            Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
        }

        fn hide_cursor(&mut self) -> std::io::Result<()> {
            self.0.hide_cursor().unwrap();
            Ok(())
        }

        fn show_cursor(&mut self) -> std::io::Result<()> {
            self.0.show_cursor().unwrap();
            Ok(())
        }

        fn get_cursor_position(&mut self) -> std::io::Result<ratatui::layout::Position> {
            Ok(self.0.get_cursor_position().unwrap())
        }

        fn set_cursor_position<P: Into<ratatui::layout::Position>>(
            &mut self,
            position: P,
        ) -> std::io::Result<()> {
            self.0.set_cursor_position(position).unwrap();
            Ok(())
        }

        fn clear(&mut self) -> std::io::Result<()> {
            self.0.clear().unwrap();
            Ok(())
        }

        fn clear_region(&mut self, clear_type: ratatui::backend::ClearType) -> std::io::Result<()> {
            self.0.clear_region(clear_type).unwrap();
            Ok(())
        }

        fn size(&self) -> std::io::Result<ratatui::layout::Size> {
            Ok(self.0.size().unwrap())
        }

        fn window_size(&mut self) -> std::io::Result<ratatui::backend::WindowSize> {
            Ok(self.0.window_size().unwrap())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.0.flush().unwrap();
            Ok(())
        }
    }

    /// A hangup landing on the pre-loop draw — before the event loop's
    /// `select!` (and its own hung-up check) is ever reached — still exits
    /// as `TerminalGone`, not as a reported error.
    #[tokio::test]
    async fn a_hangup_at_the_pre_loop_draw_exits_cleanly() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let _quiet = terminal::SIGNALS_QUIET.lock().await;
        static PRESENT: AtomicBool = AtomicBool::new(true);
        fn probe() -> bool {
            PRESENT.load(Ordering::SeqCst)
        }

        PRESENT.store(true, Ordering::SeqCst);
        let watch = terminal::TtyWatch::watching(probe);
        // The terminal is already gone by the time the very first draw runs.
        PRESENT.store(false, Ordering::SeqCst);
        let probes = TerminalProbes {
            watch,
            tick: Box::new(|| terminal::ReaderTick::Idle),
        };

        let mut terminal =
            ratatui::Terminal::new(FailingBackend(ratatui::backend::TestBackend::new(80, 24)))
                .expect("a test terminal");
        let mut app = App::new();
        let client = ApiClient::new("http://127.0.0.1:1", "token").expect("client");

        let exit = tokio::time::timeout(
            Duration::from_secs(20),
            event_loop(&mut terminal, &mut app, &client, probes),
        )
        .await
        .expect("a pre-loop hangup must not hang the session");
        assert_eq!(
            exit.expect(
                "a hangup discovered on the pre-loop draw is not a failure — \
                 there is nobody left to report one to"
            ),
            Exit::TerminalGone,
            "the pre-loop draw must consult the same hung-up check every later draw site does"
        );
    }

    /// A terminal that is gone is not an error, and a live one's failed
    /// restore still is.
    #[test]
    fn a_gone_terminal_is_not_reported_as_a_failure() {
        let broken = || std::io::Error::from(std::io::ErrorKind::BrokenPipe);
        assert!(
            finish(Ok(Exit::TerminalGone), Err(broken())).is_ok(),
            "the restore of a terminal that no longer exists cannot fail *at* \
             anyone: reporting it writes an error to a descriptor answering EIO \
             and exits nonzero for a session that ended correctly"
        );
        assert!(
            finish(Ok(Exit::Requested), Err(broken())).is_err(),
            "a restore that fails on a terminal that is still there is a real \
             failure — the user is left in raw mode and must be told"
        );
        assert!(finish(Ok(Exit::Requested), Ok(())).is_ok());
        assert!(
            finish(Err(TuiError::Terminal(broken())), Ok(())).is_err(),
            "a loop that failed reports its own failure"
        );
    }
}

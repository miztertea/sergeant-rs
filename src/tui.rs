//! The terminal UI (proposal §30): `sgt` with no subcommand.
//!
//! Two screens — Fleet (every work, live) and Work detail (stage, execution,
//! surface bindings, recent events) — over ratatui and crossterm (§34).
//!
//! **It talks to the daemon and to nothing else.** The only crate module this
//! module imports is [`crate::api`] — the client and the small view helpers
//! both screens share: no journal, no projection, no engine, no daemon
//! module. That is §30's architectural test rather than a style rule —
//!
//! > If the TUI needs a private shortcut, the API is incomplete.
//!
//! — and it is the reason this milestone added `stage` to the fleet body,
//! `journal_head` to `/v1/system`, and `work_id`/`limit` to `/v1/events`:
//! each was a field a screen needed, and each was added to the API rather
//! than reached around it.
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
//! with the screen still in raw mode (see `TerminationSignals`).
//!
//! The terminal can also *disappear* — the emulator dies, the ssh session
//! drops — and a TUI whose pty is gone can never render again, so it leaves
//! rather than lingering: the loop watches for the hangup (`TtyWatch`), the
//! key reader treats the end of input as the end of its job (`read_keys`),
//! and shutdown never waits on that reader indefinitely
//! (`KeyReader::shutdown`). Each of those three is load-bearing on its own;
//! together they are why an orphaned session exits instead of spinning at
//! ~80% of a core until somebody finds it with SIGKILL.
//!
//! Those three are *composed* in `TerminalProbes` and driven by the loop, and
//! the composition is where the bug actually lived — a guard that exists but
//! is not the tick the reader was handed protects nothing — so the wiring is
//! tested rather than the pieces alone, and the whole of it once more end to
//! end against a real pty in `tests/m6_surfaces.rs`.

use std::time::Duration;

use ratatui::Frame;
use ratatui::crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};
use serde_json::Value;

use crate::api::{ApiClient, ClientError, field_text, stage_label};

/// How many of a work's most recent events the detail screen tails.
pub const DETAIL_EVENT_TAIL: usize = 40;

/// How long the key reader waits between polls before checking for shutdown.
const KEY_POLL: Duration = Duration::from_millis(200);

/// How often the loop checks that the terminal it is drawing on still exists.
/// One `open`/`close` of `/dev/tty` per interval (see [`TtyWatch`]).
const TTY_WATCH: Duration = Duration::from_millis(500);

/// How long shutdown waits for the key reader before leaving it behind.
const READER_JOIN_GRACE: Duration = Duration::from_secs(1);

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

/// Whether the live tail is attached.
///
/// This is *screen state*, not a transient message, and it is rendered in the
/// header next to the seq counter for exactly that reason. The one-line status
/// at the bottom is overwritten by the next command outcome ("canceled …"),
/// so a tail that died two keystrokes ago would leave a screen that looks
/// live and never reconnects — fixed data presented as live truth, which is
/// the failure mode §7 cares about. The dashboard keeps a dedicated live
/// indicator for the same reason (`#live` in `web/dashboard.js`); this is the
/// TUI's.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Live {
    /// Attached: the daemon's SSE tail is feeding this screen.
    #[default]
    Attached,
    /// Detached: the tail ended or could not be opened. The data on screen is
    /// as of the last read, and `r` re-reads *and* re-attaches.
    Detached,
}

impl Live {
    /// How the header says it, in the one place that decides.
    fn label(self) -> &'static str {
        match self {
            Live::Attached => "live",
            Live::Detached => "TAIL CLOSED (r reconnects)",
        }
    }
}

/// Which screen is in front.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Screen {
    /// The fleet list.
    #[default]
    Fleet,
    /// One work's detail.
    Detail,
}

/// What a keystroke asked for. Keeping intent separate from execution is what
/// makes the keymap testable without a terminal or a daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Nothing to do.
    None,
    /// Leave the TUI.
    Quit,
    /// Re-read the API.
    Refresh,
    /// `POST /v1/work/{id}/cancel`.
    Cancel(String),
    /// `POST /v1/work/{id}/input`.
    Respond(String, String),
}

/// One row of the fleet screen, projected from the `/v1/work` body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRow {
    /// Work id.
    pub id: String,
    /// §10 state.
    pub state: String,
    /// Stage coordinate, already rendered (`10-implement 2/4 · running`).
    pub stage: String,
    /// Backend the run resolved to.
    pub backend: String,
    /// The work's intent.
    pub intent: String,
}

/// The TUI's whole state: what was read from the API, and where the cursor is.
#[derive(Debug, Default)]
pub struct App {
    /// Fleet rows, in submission order.
    pub rows: Vec<WorkRow>,
    /// Index into [`App::rows`].
    pub selected: usize,
    /// Which screen is in front.
    pub screen: Screen,
    /// `GET /v1/system` body.
    pub system: Value,
    /// `GET /v1/work/{id}` body for [`App::detail_id`].
    pub detail: Value,
    /// Recent events for the detailed work, oldest first.
    pub detail_events: Vec<Value>,
    /// Which work the detail screen is showing.
    pub detail_id: Option<String>,
    /// Highest journal seq this client has seen (the SSE resume point).
    pub last_seq: u64,
    /// Whether the live tail is attached — durable, unlike [`App::status`].
    pub live: Live,
    /// One line of feedback: last command outcome, or the last error.
    pub status: String,
    /// `Some(buffer)` while the respond prompt is open.
    pub input: Option<String>,
    /// Whether a cancel is awaiting confirmation.
    pub confirming_cancel: bool,
    /// Set once the user asked to leave.
    pub quit: bool,
}

impl App {
    /// A fresh, empty app.
    pub fn new() -> Self {
        Self {
            status: "loading…".to_string(),
            ..Self::default()
        }
    }

    /// The currently selected work id, if the fleet is not empty.
    pub fn selected_id(&self) -> Option<&str> {
        self.rows.get(self.selected).map(|row| row.id.as_str())
    }

    /// Re-read everything this app shows from the API.
    ///
    /// This is the only way state enters the app. An SSE event never carries
    /// the new value — it only says the answer changed.
    pub async fn refresh(&mut self, client: &ApiClient) -> Result<(), ClientError> {
        self.system = client.system().await?;
        if let Some(head) = self.system["journal_head"].as_u64() {
            self.last_seq = self.last_seq.max(head);
        }
        let fleet = client.fleet().await?;
        self.rows = fleet_rows(&fleet);
        if self.selected >= self.rows.len() {
            self.selected = self.rows.len().saturating_sub(1);
        }
        if let Some(id) = self.detail_id.clone() {
            match client.work(&id).await {
                Ok(detail) => {
                    self.detail = detail;
                    let events = client.work_events(&id, DETAIL_EVENT_TAIL).await?;
                    self.detail_events = events["events"].as_array().cloned().unwrap_or_default();
                }
                Err(ClientError::Api { status: 404, .. }) => {
                    // The work vanished (a data dir replaced under us). Fall
                    // back rather than keep painting a corpse.
                    self.detail_id = None;
                    self.detail = Value::Null;
                    self.detail_events.clear();
                    self.screen = Screen::Fleet;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Fold one live event into the app. Returns whether the screens need
    /// re-reading — the only thing an event is allowed to decide.
    pub fn observe(&mut self, event: &Value) -> bool {
        if let Some(seq) = event["seq"].as_u64() {
            self.last_seq = self.last_seq.max(seq);
        }
        let kind = event["kind"].as_str().unwrap_or("");
        kind.starts_with("work.")
            || kind.starts_with("stage.")
            || kind.starts_with("execution.")
            || kind.starts_with("surface.")
            || kind.starts_with("conversation.")
            || kind.starts_with("tool.")
            || kind.starts_with("usage.")
    }

    /// Interpret one keystroke.
    ///
    /// The two write keys are deliberately awkward: cancel needs a `y`
    /// confirmation and respond needs an explicit Enter. A single keypress
    /// that mutates a fleet is how a read-only tool becomes a hazard.
    pub fn on_key(&mut self, key: KeyCode) -> Action {
        if let Some(buffer) = self.input.as_mut() {
            return match key {
                KeyCode::Esc => {
                    self.input = None;
                    self.status = "respond canceled".to_string();
                    Action::None
                }
                KeyCode::Enter => {
                    let text = buffer.clone();
                    self.input = None;
                    match (
                        self.detail_id
                            .clone()
                            .or_else(|| self.selected_id().map(str::to_string)),
                        text.is_empty(),
                    ) {
                        (Some(id), false) => Action::Respond(id, text),
                        (_, true) => {
                            self.status = "respond needs an answer".to_string();
                            Action::None
                        }
                        (None, _) => {
                            self.status = "no work selected".to_string();
                            Action::None
                        }
                    }
                }
                KeyCode::Backspace => {
                    buffer.pop();
                    Action::None
                }
                KeyCode::Char(c) => {
                    buffer.push(c);
                    Action::None
                }
                _ => Action::None,
            };
        }
        if self.confirming_cancel {
            self.confirming_cancel = false;
            return match key {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    match self
                        .detail_id
                        .clone()
                        .or_else(|| self.selected_id().map(str::to_string))
                    {
                        Some(id) => Action::Cancel(id),
                        None => Action::None,
                    }
                }
                _ => {
                    self.status = "cancel abandoned".to_string();
                    Action::None
                }
            };
        }
        match key {
            KeyCode::Char('q') | KeyCode::Esc if self.screen == Screen::Fleet => {
                self.quit = true;
                Action::Quit
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.screen = Screen::Fleet;
                self.detail_id = None;
                self.detail = Value::Null;
                self.detail_events.clear();
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.rows.len() {
                    self.selected += 1;
                }
                Action::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                Action::None
            }
            KeyCode::Enter => match self.selected_id() {
                Some(id) => {
                    self.detail_id = Some(id.to_string());
                    self.screen = Screen::Detail;
                    Action::Refresh
                }
                None => Action::None,
            },
            KeyCode::Char('r') => Action::Refresh,
            KeyCode::Char('c') => {
                if self.selected_id().is_some() || self.detail_id.is_some() {
                    self.confirming_cancel = true;
                    self.status = "cancel this work? y / n".to_string();
                }
                Action::None
            }
            KeyCode::Char('i') => {
                if self.selected_id().is_some() || self.detail_id.is_some() {
                    self.input = Some(String::new());
                    self.status = "answer, then Enter (Esc cancels)".to_string();
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    /// Execute an action against the API and record its outcome.
    ///
    /// A write becomes a [`Action::Refresh`] — the screen re-reads the API
    /// rather than believing the response — and everything else passes
    /// through for the loop to act on.
    pub async fn execute(&mut self, client: &ApiClient, action: Action) -> Action {
        match action {
            Action::Cancel(id) => {
                self.status = match client.cancel(&id).await {
                    Ok(result) => format!(
                        "canceled {id} ({})",
                        result["work"]["state"].as_str().unwrap_or("?")
                    ),
                    Err(e) => format!("cancel failed: {e}"),
                };
                Action::Refresh
            }
            Action::Respond(id, input) => {
                self.status = match client.respond(&id, &input).await {
                    Ok(result) => format!(
                        "answered {id} ({})",
                        result["work"]["state"].as_str().unwrap_or("?")
                    ),
                    Err(e) => format!("respond failed: {e}"),
                };
                Action::Refresh
            }
            other => other,
        }
    }
}

/// Project the `/v1/work` body into fleet rows.
pub fn fleet_rows(fleet: &Value) -> Vec<WorkRow> {
    fleet["works"]
        .as_array()
        .map(|works| {
            works
                .iter()
                .map(|work| WorkRow {
                    id: field(work, "id"),
                    state: field(work, "state"),
                    stage: stage_label(&work["stage"]),
                    backend: field(work, "resolved_backend"),
                    intent: field(work, "intent"),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// One field of a JSON object, by the shared `-`-for-missing rule.
fn field(value: &Value, key: &str) -> String {
    field_text(&value[key])
}

fn state_style(state: &str) -> Style {
    let color = match state {
        "completed" => Color::Green,
        "needs_input" | "waiting" => Color::Yellow,
        "failed" | "blocked" | "canceled" => Color::Red,
        _ => Color::Cyan,
    };
    Style::default().fg(color)
}

// -------------------------------------------------------------- rendering

/// Draw the current screen. Pure: everything it paints is already in `app`.
pub fn draw(frame: &mut Frame, app: &App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    frame.render_widget(header_line(app), header);
    match app.screen {
        Screen::Fleet => draw_fleet(frame, app, body),
        Screen::Detail => draw_detail(frame, app, body),
    }
    frame.render_widget(footer_line(app), footer);
}

fn header_line(app: &App) -> Paragraph<'_> {
    let text = format!(
        "sergeant {} · api {} · {} · seq {} · {}",
        app.system["version"].as_str().unwrap_or("?"),
        app.system["api_revision"].as_str().unwrap_or("?"),
        app.system["data_dir"].as_str().unwrap_or("?"),
        app.last_seq,
        app.live.label(),
    );
    Paragraph::new(text).style(Style::default().add_modifier(Modifier::REVERSED))
}

fn footer_line(app: &App) -> Paragraph<'_> {
    let keys = match (&app.input, app.screen) {
        (Some(buffer), _) => format!("answer> {buffer}"),
        (None, Screen::Fleet) => {
            "j/k move · enter detail · r refresh · i respond · c cancel · q quit".to_string()
        }
        (None, Screen::Detail) => "r refresh · i respond · c cancel · q back".to_string(),
    };
    Paragraph::new(Line::from(vec![
        Span::styled(keys, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::raw(app.status.clone()),
    ]))
}

fn draw_fleet(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered().title(format!("fleet — {} work", app.rows.len()));
    if app.rows.is_empty() {
        frame.render_widget(
            Paragraph::new("no work yet — sgt run \"…\"").block(block),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|row| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:<28}", row.id)),
                Span::styled(format!("{:<12}", row.state), state_style(&row.state)),
                Span::raw(format!("{:<26}", row.stage)),
                Span::raw(format!("{:<10}", row.backend)),
                Span::raw(row.intent.clone()),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(block)
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    let mut state = ListState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let [top, bottom] =
        Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(area);
    let id = app.detail_id.clone().unwrap_or_default();
    let work = &app.detail["work"];
    let execution = &app.detail["execution"];
    let workflow = &app.detail["workflow"];

    let mut lines = vec![
        labelled("work", &id),
        Line::from(vec![
            Span::styled("state          ", Style::default().fg(Color::DarkGray)),
            Span::styled(field(work, "state"), state_style(&field(work, "state"))),
        ]),
        labelled("intent", &field(work, "intent")),
        labelled("workspace", &field(work, "workspace")),
        labelled(
            "workflow",
            &format!(
                "{} v{}",
                field(workflow, "name"),
                field(workflow, "version")
            ),
        ),
        labelled("stage", &stage_label(&app.detail["stage"])),
        labelled("stage detail", &field(&app.detail["stage"], "detail")),
        labelled("backend", &field(&app.detail, "backend")),
        labelled("route source", &field(&app.detail, "route_source")),
        labelled("execution", &field(execution, "execution_id")),
        labelled("native session", &field(execution, "native_id")),
        labelled("stop requested", &field(execution, "stop_requested")),
    ];
    lines.push(labelled("surface", ""));
    match app.detail["surface"]["bindings"].as_array() {
        Some(bindings) if !bindings.is_empty() => {
            for binding in bindings {
                lines.push(Line::from(format!(
                    "  {} → {} [{}] base {}",
                    field(binding, "repository"),
                    field(binding, "worktree_path"),
                    field(binding, "work_branch"),
                    field(binding, "base_branch"),
                )));
            }
        }
        _ => lines.push(Line::from("  (no repository surface bound)")),
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::bordered().title("work detail"))
            .wrap(Wrap { trim: false }),
        top,
    );

    let events: Vec<ListItem> = app
        .detail_events
        .iter()
        .rev()
        .map(|event| {
            ListItem::new(Line::from(format!(
                "{:>6}  {:<30} {}",
                field(event, "seq"),
                field(event, "kind"),
                event_detail(event),
            )))
        })
        .collect();
    frame.render_widget(
        List::new(events).block(Block::bordered().title("recent events")),
        bottom,
    );
}

fn labelled<'a>(label: &'a str, value: &str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{label:<15}"), Style::default().fg(Color::DarkGray)),
        Span::raw(value.to_string()),
    ])
}

/// A single bounded line of payload for the event tail.
fn event_detail(event: &Value) -> String {
    let payload = &event["payload"];
    for key in [
        "reason", "prompt", "detail", "summary", "text", "name", "stage_id",
    ] {
        if let Some(text) = payload[key].as_str() {
            return truncate(text, 90);
        }
    }
    truncate(&payload.to_string(), 90)
}

fn truncate(text: &str, max: usize) -> String {
    let flat = text.replace('\n', " ");
    if flat.chars().count() <= max {
        return flat;
    }
    flat.chars().take(max).collect::<String>() + "…"
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
    let result = event_loop(
        &mut terminal,
        &mut app,
        &client,
        TerminalProbes::production(),
    )
    .await;
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
    /// The terminal itself went away (see [`TtyWatch`]).
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
    let (keys_tx, mut keys) = tokio::sync::mpsc::unbounded_channel::<KeyCode>();
    let TerminalProbes { watch: tty, tick } = probes;
    // crossterm's reader is blocking, so it lives on its own OS thread and
    // posts keystrokes into the async loop. It polls rather than blocking
    // forever so it notices the loop going away, and its tick checks the
    // terminal is still there before every poll so it never hands a hung-up
    // pty to a library that would spin on it.
    let reader = KeyReader::spawn(tick, keys_tx);

    let mut stream = attach(client, app).await;
    let mut signals = TerminationSignals::install();
    let mut watch = tokio::time::interval(TTY_WATCH);

    let mut outcome: Result<Exit, TuiError> = Ok(Exit::Requested);
    'session: {
        if let Err(e) = terminal.draw(|frame| draw(frame, app)) {
            outcome = Err(terminal_error(e));
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
                            // question.
                            if stream.is_none() {
                                stream = attach(client, app).await;
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
                            // subscriber the server dropped, a broken chunk). Keep
                            // the UI alive and say so — durably, in the header, not
                            // only in a status line the next command overwrites.
                            stream = None;
                            app.live = Live::Detached;
                            app.status = "live tail closed — press r to refresh".to_string();
                        }
                        TailStep::Malformed(detail) => {
                            // R-WATCH-7's new outcome, handled deliberately rather
                            // than falling through to the same message as an
                            // ordinary disconnect: the daemon sent a frame this
                            // client could not decode, which is protocol drift
                            // worth a distinct status line, not a quiet reconnect
                            // prompt.
                            stream = None;
                            app.live = Live::Detached;
                            app.status =
                                format!("live tail error: {detail} — press r to refresh");
                        }
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

/// The key reader thread, and the only two things the loop does with it:
/// start it, and stop waiting for it.
///
/// It is a type rather than four locals in [`event_loop`] because the
/// *shutdown* is the part that has to stay bounded (issue #3), and a bound
/// spelled out inline is a bound no test can hold on to: `shutdown` is both
/// what the loop calls and what `shutdown_leaves_a_wedged_reader_behind`
/// drives.
struct KeyReader {
    /// Asks the reader to stop at its next turn.
    stop: std::sync::mpsc::Sender<()>,
    /// Disconnects when the thread ends, however it ends.
    done: std::sync::mpsc::Receiver<()>,
    /// Joined only once `done` says joining cannot block.
    thread: std::thread::JoinHandle<()>,
}

impl KeyReader {
    /// Start reading keys off `tick` into `keys`.
    fn spawn(
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
    fn shutdown(self) -> ReaderExit {
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
enum ReaderTick {
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
fn read_keys(
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
struct TerminalProbes {
    /// The loop's periodic check that the terminal is still there.
    watch: TtyWatch,
    /// The reader thread's tick: that same check, then a poll.
    tick: Box<dyn FnMut() -> ReaderTick + Send>,
}

impl TerminalProbes {
    /// The real terminal: `/dev/tty` and crossterm.
    fn production() -> Self {
        Self::over(controlling_terminal_present, crossterm_poll)
    }

    /// The composition, over any probe and any poll.
    fn over(probe: fn() -> bool, poll: fn() -> ReaderTick) -> Self {
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
fn guarded_tick(tty: &TtyWatch, poll: fn() -> ReaderTick) -> ReaderTick {
    if tty.hung_up() {
        return ReaderTick::Ended;
    }
    poll()
}

/// The production poll: one crossterm turn, unguarded.
fn crossterm_poll() -> ReaderTick {
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
enum ReaderExit {
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
fn join_reader(done: &std::sync::mpsc::Receiver<()>, grace: Duration) -> ReaderExit {
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
struct TtyWatch {
    had_tty: bool,
    /// How "is the terminal there?" is answered. A field rather than a direct
    /// call so the whole hangup path can be driven by a scripted terminal;
    /// production always passes [`controlling_terminal_present`].
    probe: fn() -> bool,
}

impl TtyWatch {
    /// Record the starting state, as `probe` sees it.
    fn watching(probe: fn() -> bool) -> Self {
        Self {
            had_tty: probe(),
            probe,
        }
    }

    /// The starting state, stated outright — the truth table's fixture.
    #[cfg(test)]
    fn from_probe(had_tty: bool) -> Self {
        Self {
            had_tty,
            probe: controlling_terminal_present,
        }
    }

    /// Whether the terminal this session started with has hung up.
    fn hung_up(&self) -> bool {
        self.decide((self.probe)())
    }

    /// [`TtyWatch::hung_up`]'s rule as a function of the probe — the half
    /// that can be tested without arranging a dying pty.
    fn decide(&self, present_now: bool) -> bool {
        self.had_tty && !present_now
    }
}

/// `ENXIO` — the errno the hangup measurement produced, and the only one that
/// means the terminal went away.
///
/// Spelled as its number because this crate has no libc dependency and is not
/// buying one for a constant: 6 on Linux and on every BSD including macOS.
#[cfg(unix)]
const ENXIO: i32 = 6;

/// Whether `/dev/tty` — this process's controlling terminal — can be opened.
#[cfg(unix)]
fn controlling_terminal_present() -> bool {
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
fn terminal_present_at(path: &str) -> bool {
    match std::fs::File::open(path) {
        Ok(_) => true,
        Err(e) => e.raw_os_error() != Some(ENXIO),
    }
}

/// No equivalent probe off Unix; the watch simply never fires there.
#[cfg(not(unix))]
fn controlling_terminal_present() -> bool {
    true
}

/// Open the live tail and record the outcome where the screen can show it.
///
/// Used for the first attach and for every reconnect, so "attached" is
/// decided in one place and cannot drift from what the header says.
async fn attach(client: &ApiClient, app: &mut App) -> Option<crate::api::EventStream> {
    match client.stream_events(app.last_seq).await {
        Ok(stream) => {
            app.live = Live::Attached;
            Some(stream)
        }
        Err(e) => {
            app.live = Live::Detached;
            app.status = format!("live tail unavailable: {e}");
            None
        }
    }
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
struct TerminationSignals {
    #[cfg(unix)]
    term: Option<tokio::signal::unix::Signal>,
    #[cfg(unix)]
    hangup: Option<tokio::signal::unix::Signal>,
}

impl TerminationSignals {
    fn install() -> Self {
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
    async fn terminated(&mut self) {
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

/// What one step of the live tail produced, for the caller to act on
/// deliberately (R-WATCH-7: `EventStream::next_event` now distinguishes a
/// decoded event, a clean/transport stream end, and a malformed frame — this
/// audits that the TUI, the tail's other consumer besides `sgt watch`, does
/// not let the new variant collapse back into "the tail ended" the way a
/// naive `.ok()` would).
enum TailStep {
    /// A decoded event, as the JSON the rest of this module already expects.
    Event(Value),
    /// The stream ended — cleanly or via a transport failure; both leave the
    /// screen in the same detached state, as before this revision.
    Ended,
    /// R-WATCH-7's new outcome: the daemon sent a `data:` frame this client
    /// could not decode. Distinct from `Ended` so the status line says what
    /// actually happened instead of quietly relabeling protocol drift as an
    /// ordinary disconnect.
    Malformed(String),
}

/// Await the next SSE event, or park forever when there is no stream — so
/// `select!` keeps working on the keyboard arm alone.
async fn next_event(stream: &mut Option<crate::api::EventStream>) -> TailStep {
    match stream {
        Some(stream) => match stream.next_event().await {
            Ok(Some(event)) => serde_json::to_value(event)
                .map(TailStep::Event)
                .unwrap_or(TailStep::Ended),
            Ok(None) => TailStep::Ended,
            Err(malformed) => TailStep::Malformed(malformed.to_string()),
        },
        None => std::future::pending().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fleet_of(states: &[(&str, &str)]) -> Value {
        json!({"works": states.iter().map(|(id, state)| json!({
            "id": id,
            "state": state,
            "intent": format!("intent for {id}"),
            "stage": {"stage_id": "10-implement", "index": 1, "of": 2, "status": "running"},
            "resolved_backend": "fake",
        })).collect::<Vec<_>>()})
    }

    #[test]
    fn navigation_is_bounded_by_the_fleet() {
        let mut app = App::new();
        app.rows = fleet_rows(&fleet_of(&[("a", "pending"), ("b", "running")]));
        app.on_key(KeyCode::Up);
        assert_eq!(app.selected, 0, "up at the top stays at the top");
        app.on_key(KeyCode::Down);
        app.on_key(KeyCode::Down);
        assert_eq!(app.selected, 1, "down at the bottom stays at the bottom");
    }

    #[test]
    fn a_write_key_needs_a_second_keystroke() {
        let mut app = App::new();
        app.rows = fleet_rows(&fleet_of(&[("a", "running")]));
        assert_eq!(
            app.on_key(KeyCode::Char('c')),
            Action::None,
            "no cancel yet"
        );
        assert_eq!(
            app.on_key(KeyCode::Char('y')),
            Action::Cancel("a".to_string()),
            "confirmation issues the cancel"
        );
        assert_eq!(app.on_key(KeyCode::Char('c')), Action::None);
        assert_eq!(
            app.on_key(KeyCode::Char('n')),
            Action::None,
            "anything but y abandons"
        );
    }

    #[test]
    fn respond_collects_a_line_then_submits_it() {
        let mut app = App::new();
        app.rows = fleet_rows(&fleet_of(&[("a", "needs_input")]));
        app.on_key(KeyCode::Char('i'));
        for c in "yes".chars() {
            app.on_key(KeyCode::Char(c));
        }
        app.on_key(KeyCode::Backspace);
        assert_eq!(
            app.on_key(KeyCode::Enter),
            Action::Respond("a".to_string(), "ye".to_string())
        );
    }

    /// The respond and cancel flows are both abandonable, and each abandon
    /// sets a status the header shows — the regression this pins is a
    /// status that silently stays whatever it was before, which nobody
    /// would notice without reading the code, since both flows already
    /// return `Action::None` either way.
    #[test]
    fn abandoning_respond_or_cancel_leaves_a_status_saying_so() {
        let mut app = App::new();
        app.rows = fleet_rows(&fleet_of(&[("a", "needs_input")]));

        // Esc while composing a response abandons it without submitting.
        app.on_key(KeyCode::Char('i'));
        assert!(app.input.is_some(), "'i' opens the respond prompt");
        for c in "partial".chars() {
            app.on_key(KeyCode::Char(c));
        }
        assert_eq!(
            app.on_key(KeyCode::Esc),
            Action::None,
            "abandoning a response is not a submission"
        );
        assert!(app.input.is_none(), "the prompt closes");
        assert_eq!(app.status, "respond canceled");

        // Its sibling: anything but y/Y after 'c' abandons the cancel
        // confirmation.
        assert_eq!(
            app.on_key(KeyCode::Char('c')),
            Action::None,
            "no cancel yet — this keystroke only opens the confirmation"
        );
        assert!(app.confirming_cancel);
        assert_eq!(
            app.on_key(KeyCode::Char('n')),
            Action::None,
            "declining the confirmation issues no command"
        );
        assert!(!app.confirming_cancel, "the confirmation closes either way");
        assert_eq!(app.status, "cancel abandoned");
    }

    /// The rest of the keymap, in one pass: the keys that answer differently
    /// depending on which screen is in front, and the keystrokes that must
    /// answer with nothing at all.
    ///
    /// The suite's other key tests all drive the two write flows. What was
    /// unpinned is the read side — that `q` leaves the program from the fleet
    /// but only leaves the *screen* from a detail (and clears what that
    /// screen was showing), and that every key with no meaning here is inert
    /// rather than falling into a neighbour's arm.
    #[test]
    fn the_keymap_answers_by_screen_and_is_inert_everywhere_else() {
        let mut app = App::new();
        app.rows = fleet_rows(&fleet_of(&[("a", "running")]));

        // Enter opens the detail screen for the selected work…
        assert_eq!(app.on_key(KeyCode::Enter), Action::Refresh);
        assert_eq!(app.screen, Screen::Detail);
        app.detail = json!({"work": {"state": "running"}});
        app.detail_events = vec![json!({"seq": 1, "kind": "work.started"})];

        // …and `q` there is "back", not "quit", with nothing of the old work
        // left to paint over the next one.
        assert_eq!(app.on_key(KeyCode::Char('q')), Action::None);
        assert_eq!(app.screen, Screen::Fleet);
        assert!(!app.quit, "leaving a detail screen must not leave the TUI");
        assert_eq!(app.detail_id, None);
        assert_eq!(app.detail, Value::Null);
        assert!(app.detail_events.is_empty());

        // From the fleet, the same key is the exit. So is Esc, on both
        // screens — the two keys are one arm.
        assert_eq!(app.on_key(KeyCode::Enter), Action::Refresh);
        assert_eq!(app.on_key(KeyCode::Esc), Action::None);
        assert_eq!(app.screen, Screen::Fleet, "Esc backs out of a detail too");
        assert_eq!(app.on_key(KeyCode::Char('r')), Action::Refresh);
        assert_eq!(
            app.on_key(KeyCode::Char('q')),
            Action::Quit,
            "`q` from the fleet is the exit the footer advertises"
        );
        assert!(app.quit);
        app.quit = false;
        assert_eq!(app.on_key(KeyCode::Esc), Action::Quit);
        assert!(app.quit);

        // Keys with no meaning are inert, in every mode this app has.
        let mut app = App::new();
        assert_eq!(
            app.on_key(KeyCode::Enter),
            Action::None,
            "Enter on an empty fleet opens nothing"
        );
        assert_eq!(app.on_key(KeyCode::Char('z')), Action::None);
        assert_eq!(app.on_key(KeyCode::Tab), Action::None);
        assert!(!app.quit && app.screen == Screen::Fleet);
        app.rows = fleet_rows(&fleet_of(&[("a", "needs_input")]));
        app.on_key(KeyCode::Char('i'));
        assert_eq!(
            app.on_key(KeyCode::Tab),
            Action::None,
            "a key the composer has no use for must not close it"
        );
        assert!(app.input.is_some());

        // An empty answer is refused rather than sent as one…
        assert_eq!(app.on_key(KeyCode::Enter), Action::None);
        assert_eq!(app.status, "respond needs an answer");
        // …and so are both write verbs with nothing selected.
        let mut app = App::new();
        app.input = Some("an answer to nobody".to_string());
        assert_eq!(app.on_key(KeyCode::Enter), Action::None);
        assert_eq!(app.status, "no work selected");
        app.confirming_cancel = true;
        assert_eq!(
            app.on_key(KeyCode::Char('y')),
            Action::None,
            "a confirmed cancel with nothing selected cancels nothing"
        );
    }

    /// What the detail screen paints for a work in a terminal state whose
    /// surface is gone, while an answer is being typed.
    ///
    /// The acceptance render in the m6 suite only ever reaches this screen
    /// with a bound surface and no prompt open, so the "(no repository
    /// surface bound)" line, the composer's own footer, and the red state
    /// colour were never painted.
    #[test]
    fn the_detail_screen_paints_a_failed_work_with_no_surface_and_an_open_prompt() {
        let mut app = App::new();
        app.system = json!({"version": "0.1.0", "api_revision": "v1", "data_dir": "/tmp/d"});
        app.screen = Screen::Detail;
        app.detail_id = Some("01FAILED".to_string());
        app.detail = json!({
            "work": {"state": "failed", "intent": "a thing that failed"},
            "workflow": {"name": "wf", "version": 1},
            "stage": {"stage_id": "00-only", "index": 0, "of": 1, "status": "failed"},
            "surface": {"bindings": []},
            "execution": null,
        });
        app.input = Some("retry it".to_string());

        let screen = screen_text(&app);
        assert!(
            screen.contains("(no repository surface bound)"),
            "a work whose surface is gone says so where the bindings were: {screen}"
        );
        assert!(
            screen.contains("answer> retry it"),
            "the footer becomes the composer while a prompt is open, so the \
             keymap is not advertised as available: {screen}"
        );
        assert!(
            !screen.contains("j/k move"),
            "…and the keymap is gone while it is: {screen}"
        );
        assert_eq!(
            state_style("failed").fg,
            Some(Color::Red),
            "a terminal failure is red"
        );
        assert_eq!(state_style("canceled").fg, Some(Color::Red));
        assert_eq!(state_style("blocked").fg, Some(Color::Red));
        assert_eq!(
            state_style("active").fg,
            Some(Color::Cyan),
            "and anything still in flight is not"
        );
    }

    #[test]
    fn only_state_bearing_events_ask_for_a_refresh() {
        let mut app = App::new();
        assert!(app.observe(&json!({"seq": 7, "kind": "work.completed"})));
        assert!(!app.observe(&json!({"seq": 8, "kind": "daemon.started"})));
        assert_eq!(app.last_seq, 8, "the resume point tracks every event");
    }

    /// The projection itself, field by field.
    ///
    /// The other unit tests here use `fleet_rows` as a fixture builder and
    /// then assert about keys and navigation, which never looks at what it
    /// put in the row: before this test, only the `TestBackend` render in the
    /// acceptance suite could catch a `state`/`backend` swap, and nothing at
    /// all pinned the `-`-for-missing rule or the one-based stage position.
    #[test]
    fn a_fleet_row_is_the_api_body_projected_field_by_field() {
        let rows = fleet_rows(&fleet_of(&[("a", "needs_input")]));
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0],
            WorkRow {
                id: "a".to_string(),
                state: "needs_input".to_string(),
                // Zero-based on the wire, one-based on a screen.
                stage: "10-implement 2/2 · running".to_string(),
                backend: "fake".to_string(),
                intent: "intent for a".to_string(),
            },
            "every column of the fleet screen comes from a named field of the \
             `/v1/work` body — this is the mapping, stated"
        );

        // A work the daemon has not routed or staged yet is the common case
        // on the fleet screen's first paint, and it must read as absence
        // rather than as `null`.
        let unrouted = fleet_rows(&json!({"works": [{"id": "b", "state": "pending"}]}));
        assert_eq!(
            unrouted[0],
            WorkRow {
                id: "b".to_string(),
                state: "pending".to_string(),
                stage: "-".to_string(),
                backend: "-".to_string(),
                intent: "-".to_string(),
            },
            "missing fields read as `-`, the rule shared with the dashboard"
        );

        // A body with no `works` array at all is an empty fleet, not a panic.
        assert!(fleet_rows(&json!({})).is_empty());
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
        app.system = json!({"version": "0.1.0", "api_revision": "v1", "data_dir": "/tmp/d"});
        assert_eq!(app.live, Live::Attached, "a fresh app assumes the tail");
        assert!(
            header_text(&app).contains("live"),
            "the header states liveness: {}",
            header_text(&app)
        );

        app.live = Live::Detached;
        app.status = "live tail closed — press r to refresh".to_string();
        assert!(
            header_text(&app).contains("TAIL CLOSED"),
            "a detached tail is named in the header: {}",
            header_text(&app)
        );

        // Any later command outcome replaces the status line…
        let client = ApiClient::new("http://127.0.0.1:1", "t").expect("client");
        let action = app.execute(&client, Action::Cancel("a".to_string())).await;
        assert_eq!(action, Action::Refresh);
        assert!(
            !app.status.contains("live tail closed"),
            "the status line is the command's now: {}",
            app.status
        );
        // …and the header still says the screen is not live.
        assert_eq!(app.live, Live::Detached);
        assert!(
            header_text(&app).contains("TAIL CLOSED"),
            "the indicator must outlive the status line: {}",
            header_text(&app)
        );
    }

    /// The signal arm, exercised for real: install the handlers, send this
    /// process a SIGTERM, and require the future to resolve.
    ///
    /// Not a name-based check — the point is that the handler is *installed*,
    /// because the default disposition for SIGTERM terminates the process,
    /// and a TUI terminated that way leaves the terminal in raw mode on the
    /// alternate screen (the one failure the contract names by hand). If the
    /// install silently stopped happening, this test would not merely fail:
    /// the signal would kill the test binary, which is the same evidence.
    /// Serializes the tests that involve process-wide signals.
    ///
    /// A SIGTERM sent to this process reaches *every* handler installed in
    /// it, so the test below and any test running a session loop (which
    /// installs its own) cannot overlap: the stray signal would end the other
    /// test's loop through the wrong arm. Cheaper and more honest than
    /// tolerating either outcome in the assertion.
    static SIGNALS_QUIET: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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
    /// through [`KeyReader::shutdown`], which is what `event_loop` calls: a
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

    /// The loop ends when the terminal goes away, without needing the reader
    /// to notice — driven through `event_loop` itself.
    ///
    /// The reader cannot be relied on for this: crossterm's poll on a dead pty
    /// does not return, so the tick here keeps saying "nothing yet" exactly as
    /// a wedged one would. The watch arm is the half that has to fire, and
    /// `TerminalGone` is the exit it must produce — an ordinary `Requested`
    /// would send `run` down the restore-and-report path, into writes to a
    /// descriptor that answers EIO.
    #[cfg(unix)]
    #[tokio::test]
    async fn a_hangup_ends_the_session_through_the_watch() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let _quiet = SIGNALS_QUIET.lock().await;
        static PRESENT: AtomicBool = AtomicBool::new(true);
        fn probe() -> bool {
            PRESENT.load(Ordering::SeqCst)
        }

        PRESENT.store(true, Ordering::SeqCst);
        let watch = TtyWatch::watching(probe);
        PRESENT.store(false, Ordering::SeqCst);
        let probes = TerminalProbes {
            watch,
            tick: Box::new(|| {
                std::thread::sleep(Duration::from_millis(20));
                ReaderTick::Idle
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
            "the loop must name the hangup, so `run` knows there is nobody to \
             report to"
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

    /// The header line as the text a terminal would show — rendered into a
    /// buffer, not `Debug`-printed, so the assertion is about the screen.
    fn header_text(app: &App) -> String {
        use ratatui::buffer::Buffer;
        use ratatui::widgets::Widget;
        let area = Rect::new(0, 0, 160, 1);
        let mut buffer = Buffer::empty(area);
        header_line(app).render(area, &mut buffer);
        (0..area.width)
            .map(|x| buffer.cell((x, 0)).map_or(" ", |cell| cell.symbol()))
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    /// The whole screen as the text a terminal would show — same idea as
    /// `header_text`, one screen wider.
    fn screen_text(app: &App) -> String {
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40))
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
}

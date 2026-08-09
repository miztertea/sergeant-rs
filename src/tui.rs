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
    let result = event_loop(&mut terminal, &mut app, &client).await;
    ratatui::try_restore()?;
    result
}

async fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    client: &ApiClient,
) -> Result<(), TuiError> {
    let (keys_tx, mut keys) = tokio::sync::mpsc::unbounded_channel::<KeyCode>();
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    // crossterm's reader is blocking, so it lives on its own OS thread and
    // posts keystrokes into the async loop. It polls rather than blocking
    // forever so it notices the loop going away.
    let reader = std::thread::spawn(move || {
        loop {
            if stop_rx.try_recv().is_ok() {
                return;
            }
            match event::poll(KEY_POLL) {
                Ok(true) => match event::read() {
                    Ok(TermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                        if keys_tx.send(key.code).is_err() {
                            return;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => return,
                },
                Ok(false) => {}
                Err(_) => return,
            }
        }
    });

    let mut stream = attach(client, app).await;
    let mut signals = TerminationSignals::install();

    terminal.draw(|frame| draw(frame, app))?;
    loop {
        let mut needs_refresh = false;
        tokio::select! {
            key = keys.recv() => {
                let Some(key) = key else { break };
                let action = app.on_key(key);
                match app.execute(client, action).await {
                    Action::Quit => break,
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
                    break;
                }
            }
            event = next_event(&mut stream) => {
                match event {
                    Some(event) => needs_refresh = app.observe(&event),
                    None => {
                        // The tail ended (daemon restart or shutdown, a lagged
                        // subscriber the server dropped, a broken chunk). Keep
                        // the UI alive and say so — durably, in the header, not
                        // only in a status line the next command overwrites.
                        stream = None;
                        app.live = Live::Detached;
                        app.status = "live tail closed — press r to refresh".to_string();
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
                break;
            }
        }
        if needs_refresh && let Err(e) = app.refresh(client).await {
            app.status = format!("refresh failed: {e}");
        }
        terminal.draw(|frame| draw(frame, app))?;
    }

    let _ = stop_tx.send(());
    drop(keys);
    let _ = reader.join();
    Ok(())
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

/// Await the next SSE event, or park forever when there is no stream — so
/// `select!` keeps working on the keyboard arm alone.
async fn next_event(stream: &mut Option<crate::api::EventStream>) -> Option<Value> {
    match stream {
        Some(stream) => stream
            .next_event()
            .await
            .and_then(|event| serde_json::to_value(event).ok()),
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
    #[cfg(unix)]
    #[tokio::test]
    async fn a_termination_signal_is_caught_so_the_terminal_can_be_restored() {
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
}

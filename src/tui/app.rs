//! The application shell's state machine: top-level navigation (§7.1),
//! the global Attention drawer's open/closed flag (§7.3), the overlay
//! mechanism (§7.4), and the canonical-Work stub (§7.2) — composed over the
//! per-destination screens in [`super::home`], [`super::fleet`],
//! [`super::workflows`], and [`super::estate`].

use ratatui::crossterm::event::{KeyCode, KeyEvent};
use serde_json::Value;

use crate::api::{ApiClient, ClientError};

use super::connection::Live;
use super::estate::{
    EstateOutcome, EstateScreen, GroupFormOutcome, PendingRepoAdd, PendingRepoAddOutcome,
    RepoFormOutcome,
};
use super::fleet::{FleetOutcome, FleetScreen, WorkRow, fleet_rows};
use super::home::{HomeOutcome, NewWorkForm};
use super::overlay::{Overlay, SlashCommand};
use super::work_view::{Tab, WorkScreen, WorkScreenOutcome};
use super::workflows::{WorkflowsOutcome, WorkflowsScreen};

/// §7.1's top-level destinations, in their displayed order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Destination {
    #[default]
    Home,
    Fleet,
    Workflows,
    Estate,
}

impl Destination {
    pub const ALL: [Destination; 4] = [
        Destination::Home,
        Destination::Fleet,
        Destination::Workflows,
        Destination::Estate,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Destination::Home => "Home",
            Destination::Fleet => "Fleet",
            Destination::Workflows => "Workflows",
            Destination::Estate => "Estate",
        }
    }

    fn next(self) -> Destination {
        let i = Self::ALL.iter().position(|d| *d == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    fn prev(self) -> Destination {
        let i = Self::ALL.iter().position(|d| *d == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// The canonical Work surface currently open over a destination (§7.2):
/// `Esc` returns to exactly `from`, with that destination's own state
/// untouched underneath — see [`super::overlay`]'s note on why this needs no
/// separate focus-restore bookkeeping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenWork {
    pub id: String,
    pub from: Destination,
}

/// What a keystroke asked for. Keeping intent separate from execution is
/// what makes the keymap testable without a terminal or a daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Nothing to do.
    None,
    /// Leave the TUI.
    Quit,
    /// Re-read the API.
    Refresh,
    /// `POST /v1/work` with this body (§9.1/§9.2's New Work submission).
    Submit(Value),
    /// Open the canonical Work surface (§13) for this id: the four reads
    /// `WorkScreen::load` needs, none of which the keymap can do itself.
    OpenWork(String),
    /// §13.5's bounded "load older": grow the open Work's Evidence window
    /// and re-fetch.
    LoadOlderEvidence,
    /// §15.1's `ANSWER` composer, deliberately submitted (§15.2/§15.5).
    Respond { id: String, input: String },
    /// §15.5: confirmed from [`Overlay::CancelConfirmation`].
    Cancel(String),
    /// §15.5: confirmed from [`Overlay::RetryConfirmation`].
    Retry(String),
    /// §15.5: confirmed from [`Overlay::ExtendEnvelope`], with the explicit
    /// added-turns value the operator entered.
    Extend { id: String, additional_turns: u32 },
    /// §15.5: confirmed from [`Overlay::ReapConfirmation`].
    Reap(String),
    /// §12.1/§16.2: `POST /v1/estate/repos`, confirmed from
    /// [`Overlay::RepoAddRemove`]'s add mode. Runs off the render loop
    /// (§12.1's spinner/elapsed rule) — see [`App::execute`].
    AddRepo {
        name: String,
        origin: Option<String>,
        instructions: Option<String>,
    },
    /// §12.1/§16.2: `DELETE /v1/estate/repos/{name}`, confirmed from
    /// [`Overlay::RepoAddRemove`]'s remove mode.
    RemoveRepo(String),
    /// §12.2/§16.2: `POST /v1/estate/groups`, confirmed from
    /// [`Overlay::GroupEditRemove`]'s edit mode.
    AddGroup {
        name: String,
        repos: Vec<String>,
        brief: Option<String>,
    },
    /// §12.2/§16.2: `DELETE /v1/estate/groups/{name}`, confirmed from
    /// [`Overlay::GroupEditRemove`]'s remove mode.
    RemoveGroup { name: String, repos: Vec<String> },
    /// §12.3/§12.4: Health's disk-pressure detail asking to open
    /// [`Overlay::RetainedPreview`] — `GET /v1/retained`, the same
    /// estate-wide read [`Action::Reap`] already has a client method for.
    LoadRetained,
}

/// Live state for the one confirmation overlay open, if any (§7.4: the
/// mechanism in [`super::overlay`] is a fixed, dataless set of panels; this
/// is the one variant's worth of payload [`Overlay::ExtendEnvelope`] and its
/// siblings actually need — the digits typed so far, which control has
/// focus, and the last rejected attempt).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PendingAction {
    /// §15.5's "explicit added-turns input" — digits only.
    pub extend_turns: String,
    /// Whether the Confirm control (rather than the turns field) has focus —
    /// Tab moves between them (§15.2). Cancel/Retry/Reap have no field to
    /// leave, so Confirm is always the target there.
    pub confirm_focused: bool,
    /// Set when a confirmed mutation comes back with a structured error —
    /// shown inline rather than the overlay silently closing on a race
    /// (§13.10's closing lines).
    pub last_error: Option<String>,
}

/// The TUI's whole state: what was read from the API, and where the cursor is.
pub struct App {
    /// Fleet rows, in submission order — the one source every screen that
    /// shows Work (Fleet, the Attention drawer, Home's panes) reads from.
    pub rows: Vec<WorkRow>,
    /// Which top-level destination is in front.
    pub destination: Destination,
    /// Home's New Work form.
    pub home: NewWorkForm,
    /// Fleet's selection and local filters.
    pub fleet: FleetScreen,
    /// Workflows' loaded catalog, selection, and local `@` filter (§11).
    pub workflows: WorkflowsScreen,
    /// The canonical Work surface, if one is open over `destination`.
    pub open_work: Option<OpenWork>,
    /// The open Work surface's own fetched data and tab/scroll state
    /// (§13) — `None` while `open_work` is `Some` but the opening fetch is
    /// still in flight.
    pub work_screen: Option<WorkScreen>,
    /// The one open overlay, if any (§7.4).
    pub overlay: Option<Overlay>,
    /// The confirmation overlay's own live state (§15.5) — reset whenever an
    /// overlay opens or closes.
    pub pending_action: PendingAction,
    /// Estate's three sub-screens (§12) and their edit forms.
    pub estate: EstateScreen,
    /// The Work id [`Overlay::ReapConfirmation`] acts on when it was opened
    /// from [`Overlay::RetainedPreview`] (§12.4) rather than from an open
    /// Work surface — `None` the rest of the time. [`App::open_work_id`]
    /// stays the source inside an open Work; this is Estate's own source,
    /// so the same confirmation panel and [`Action::Reap`] serve both without
    /// a second reap implementation.
    pub retained_reap_id: Option<String>,
    /// [`Overlay::WorkflowChooser`]'s own live state: which catalog entry is
    /// highlighted (§15.4's "On Home, `@` selects the existing workflow
    /// request field"). Reset whenever the overlay opens or closes, the same
    /// as `pending_action` above.
    pub workflow_chooser_index: usize,
    /// [`Overlay::SlashPalette`]'s own live state: which [`SlashCommand`] is
    /// highlighted (§15.3) — reset to `0` every time the palette opens, the
    /// same as `workflow_chooser_index` above.
    pub slash_palette_index: usize,
    /// Whether the global Attention drawer is showing (§7.3: opens by
    /// default at Wide; `~` toggles it at any tier).
    pub drawer_open: bool,
    /// `GET /v1/system` body.
    pub system: Value,
    /// Highest journal seq this client has seen (the SSE resume point).
    pub last_seq: u64,
    /// Whether the live tail is attached — durable, unlike [`App::status`].
    pub live: Live,
    /// One line of feedback: last command outcome, or the last error.
    pub status: String,
    /// Set once the user asked to leave.
    pub quit: bool,
}

impl App {
    /// A fresh, empty app.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            destination: Destination::default(),
            home: NewWorkForm::default(),
            fleet: FleetScreen::default(),
            workflows: WorkflowsScreen::default(),
            open_work: None,
            work_screen: None,
            overlay: None,
            pending_action: PendingAction::default(),
            estate: EstateScreen::default(),
            retained_reap_id: None,
            workflow_chooser_index: 0,
            slash_palette_index: 0,
            drawer_open: true,
            system: Value::default(),
            last_seq: 0,
            live: Live::default(),
            status: "loading…".to_string(),
            quit: false,
        }
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
        self.fleet.clamp(&self.rows);
        // §11: the catalog Workflows and Home's `@` chooser both read is
        // fetched here alongside Fleet, not lazily on first visit to either
        // — best-effort, like the open Work's own refresh below, so a
        // daemon that cannot answer this one read does not blank the rest
        // of an otherwise-successful refresh.
        if let Ok(cwd) = std::env::current_dir() {
            match client.workflows(&cwd).await {
                Ok(body) => {
                    let entries = body["workflows"].as_array().cloned().unwrap_or_default();
                    self.workflows.set_entries(entries);
                    self.workflows.last_error = None;
                }
                Err(e) => self.workflows.last_error = Some(e.to_string()),
            }
        }
        if let Some(open) = &self.open_work {
            if !self.rows.iter().any(|row| row.id == open.id) {
                // The work vanished (a data dir replaced under us, or it aged
                // out of what the daemon reports). Fall back rather than keep
                // painting a corpse.
                self.open_work = None;
                self.work_screen = None;
            } else if let Some(screen) = &mut self.work_screen {
                // A live SSE-classified event or an explicit `r` re-reads
                // everything this screen shows too — best-effort: a failed
                // per-Work refresh must not blank a screen whose last
                // successful read is still good, so the error lands on the
                // screen itself rather than propagating out of here.
                if let Err(e) = screen.refresh(client).await {
                    screen.last_error = Some(e.to_string());
                }
            }
        }
        // §12: read only while Estate is in front — the same
        // fetch-what-the-current-screen-shows discipline as `work_screen`
        // above, so browsing Home or Fleet never costs an extra doctor run.
        // Best-effort, same reasoning as `work_screen`'s: a failed estate
        // read must not blank fleet/system data that loaded fine.
        if self.destination == Destination::Estate {
            match client.repos().await {
                Ok(result) => {
                    self.estate.repos = result["repos"].as_array().cloned().unwrap_or_default();
                }
                Err(e) => self.estate.last_error = Some(e.to_string()),
            }
            match client.groups().await {
                Ok(result) => {
                    self.estate.groups = result["groups"].as_array().cloned().unwrap_or_default();
                }
                Err(e) => self.estate.last_error = Some(e.to_string()),
            }
            match client.doctor().await {
                Ok(result) => self.estate.doctor = result,
                Err(e) => self.estate.last_error = Some(e.to_string()),
            }
            self.estate.clamp();
        }
        Ok(())
    }

    /// Non-blocking poll of any Estate mutation kicked off the render loop
    /// (§12.1) — called every loop tick, whether or not a key arrived, so a
    /// slow `git clone` finishes on its own schedule instead of waiting for
    /// the next keystroke. Returns whether the screen needs a re-read.
    pub fn poll_background(&mut self) -> bool {
        match self.estate.poll() {
            Some(PendingRepoAddOutcome::InProgress) | None => false,
            Some(PendingRepoAddOutcome::Done(result)) => {
                self.estate.pending_repo_add = None;
                match result {
                    Ok(_) => {
                        self.status = "repo added".to_string();
                        if self.overlay == Some(Overlay::RepoAddRemove) {
                            self.overlay = None;
                        }
                        true
                    }
                    Err(e) => {
                        self.status = format!("add repo failed: {e}");
                        self.estate.repo_form.last_error = Some(e.to_string());
                        false
                    }
                }
            }
        }
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

    fn goto(&mut self, destination: Destination) {
        self.destination = destination;
        if destination == Destination::Home {
            // §9.2: the intent editor is the primary focus — entering Home
            // (fresh, or back from elsewhere) always re-focuses it.
            self.home.refocus();
        }
    }

    /// Whether the active destination's own text input currently owns the
    /// keyboard — checked before any global nav key is allowed to fire, so
    /// typing into a field can never be hijacked as a shortcut.
    fn wants_text_focus(&self) -> bool {
        match self.destination {
            Destination::Home => self.home.wants_text_focus(),
            Destination::Fleet => self.fleet.wants_text_focus(),
            Destination::Workflows => self.workflows.wants_text_focus(),
            Destination::Estate => false,
        }
    }

    /// Interpret one keystroke. `impl Into<KeyEvent>` rather than a bare
    /// `KeyEvent` so every existing call site that only ever named a
    /// `KeyCode` — every test in this tree, and every one in
    /// `tests/m6_surfaces.rs` — keeps compiling unchanged (crossterm's own
    /// `From<KeyCode> for KeyEvent` fills in empty modifiers), while the
    /// production loop forwards the full event `Ctrl+Enter` needs (§15.2).
    pub fn on_key(&mut self, key: impl Into<KeyEvent>) -> Action {
        let key: KeyEvent = key.into();
        if self.overlay.is_some() {
            return self.on_key_overlay(key);
        }
        if self.open_work.is_some() {
            return self.on_key_open_work(key);
        }

        if !self.wants_text_focus()
            && let Some(action) = self.on_key_global(key.code)
        {
            return action;
        }

        match self.destination {
            Destination::Home => match self.home.on_key(key) {
                HomeOutcome::None => Action::None,
                HomeOutcome::Submit(body) => Action::Submit(body),
                // §15.4: `@` on Home's workflow field opens the live-catalog
                // chooser instead of typing a literal `@` — the overlay is
                // `App`-level state, so `Home` only asks for it.
                HomeOutcome::OpenWorkflowChooser => {
                    self.workflow_chooser_index = 0;
                    self.overlay = Some(Overlay::WorkflowChooser);
                    Action::None
                }
                // §15.3: `/` at the first non-whitespace position of the
                // INTENT composer, same wiring shape as `@`'s chooser above.
                HomeOutcome::OpenSlashPalette => {
                    self.slash_palette_index = 0;
                    self.overlay = Some(Overlay::SlashPalette);
                    Action::None
                }
            },
            Destination::Fleet => match self.fleet.on_key(key.code, &self.rows) {
                FleetOutcome::None => Action::None,
                FleetOutcome::Open(id) => Action::OpenWork(id),
            },
            // §11.4's "Use in New Work": hands the chosen name to Home's
            // form and switches there, refocusing it exactly as pressing
            // `1` would (§9.2's "entering Home always re-focuses the
            // intent editor").
            Destination::Workflows => match self.workflows.on_key(key.code) {
                WorkflowsOutcome::None => Action::None,
                WorkflowsOutcome::UseInNewWork(name) => {
                    self.home.workflow = name;
                    self.goto(Destination::Home);
                    Action::None
                }
                // §15.4's Workflows half (issue #153): `@` on the Workflows
                // screen opens the same live-catalog chooser Home's
                // workflow field opens — the overlay is `App`-level state,
                // so `Workflows` only asks for it, exactly as `Home` does
                // above.
                WorkflowsOutcome::OpenWorkflowChooser => {
                    self.workflow_chooser_index = 0;
                    self.overlay = Some(Overlay::WorkflowChooser);
                    Action::None
                }
            },
            // §12: Tab/BackTab are Estate's own (sub-tab switching, guarded
            // out of `on_key_global` below) — everything else it does not
            // recognize falls through to `EstateOutcome::None`.
            Destination::Estate => match self.estate.on_key(key.code) {
                EstateOutcome::None => Action::None,
                EstateOutcome::OpenAddRepo => {
                    self.estate.repo_form.reset_for_add();
                    self.overlay = Some(Overlay::RepoAddRemove);
                    Action::None
                }
                EstateOutcome::OpenRemoveRepo(name) => {
                    self.estate.repo_form.reset_for_remove(name);
                    self.overlay = Some(Overlay::RepoAddRemove);
                    Action::None
                }
                EstateOutcome::OpenGroupEdit => {
                    let existing = self.estate.selected_group().cloned();
                    self.estate.group_form.reset_for_edit(existing.as_ref());
                    self.overlay = Some(Overlay::GroupEditRemove);
                    Action::None
                }
                EstateOutcome::OpenGroupRemove(name) => {
                    self.estate.group_form.reset_for_remove(name);
                    self.overlay = Some(Overlay::GroupEditRemove);
                    Action::None
                }
                EstateOutcome::OpenRetained => Action::LoadRetained,
            },
        }
    }

    /// The keys that mean the same thing everywhere browsing is possible.
    /// `None` means "not a global key — let the destination see it."
    fn on_key_global(&mut self, key: KeyCode) -> Option<Action> {
        match key {
            KeyCode::Char('~') => self.drawer_open = !self.drawer_open,
            KeyCode::Char('?') => self.overlay = Some(Overlay::Help),
            // §7.4/§8.9: mouse capture stays disabled, so the connection
            // detail overlay (issue #154) needs its own dedicated keyboard
            // trigger rather than a click on the header's indicator.
            KeyCode::Char('c') => self.overlay = Some(Overlay::ConnectionDetail),
            KeyCode::Char('1') => self.goto(Destination::Home),
            KeyCode::Char('2') => self.goto(Destination::Fleet),
            KeyCode::Char('3') => self.goto(Destination::Workflows),
            KeyCode::Char('4') => self.goto(Destination::Estate),
            // §12: inside Estate, Tab/BackTab switch its own Repositories/
            // Groups/Health sub-tabs instead — the guard leaves them
            // unmatched here so they fall through to `EstateScreen::on_key`.
            KeyCode::Tab if self.destination != Destination::Estate => {
                self.goto(self.destination.next())
            }
            KeyCode::BackTab if self.destination != Destination::Estate => {
                self.goto(self.destination.prev())
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.quit = true;
                return Some(Action::Quit);
            }
            _ => return None,
        }
        Some(Action::None)
    }

    /// The id of the Work a confirmation overlay acts on — always the one
    /// currently open, since these overlays only ever reach the keymap from
    /// inside the canonical Work surface (§13.10).
    fn open_work_id(&self) -> Option<String> {
        self.open_work.as_ref().map(|w| w.id.clone())
    }

    /// Abort the open confirmation overlay without effect (§15.5's Esc
    /// path): nothing is sent, and the pending draft is dropped since it was
    /// never a durable Work draft to begin with.
    fn close_overlay(&mut self) {
        self.overlay = None;
        self.pending_action = PendingAction::default();
        self.retained_reap_id = None;
    }

    /// §15.3's fixed vocabulary, run: the palette closes unconditionally —
    /// Enter is the deliberate choice — and each command either changes
    /// state directly or opens the exact overlay/composer its keyboard
    /// mnemonic already does (§13.10's `r`/`c`/`t`/`e`/`p`, `?`, `q`), so
    /// durable/destructive actions still land on their own confirmation
    /// rather than skipping it (§15.5).
    fn run_slash_command(&mut self, command: SlashCommand) -> Action {
        self.overlay = None;
        match command {
            SlashCommand::Home => self.leave_open_work_and_goto(Destination::Home),
            SlashCommand::Fleet => self.leave_open_work_and_goto(Destination::Fleet),
            SlashCommand::Workflows => self.leave_open_work_and_goto(Destination::Workflows),
            SlashCommand::Estate => self.leave_open_work_and_goto(Destination::Estate),
            // §15.6's "q / Esc: back": leaves the open Work, the same
            // effect Esc has there. Unlike Esc/q, this does not also quit
            // from a top-level destination — `/quit` already names that
            // outright, so with no Work open this is a deliberate no-op
            // rather than an exit hidden behind a menu pick.
            SlashCommand::Back => {
                if self.open_work.is_some() {
                    self.open_work = None;
                    self.work_screen = None;
                }
            }
            SlashCommand::Refresh => return Action::Refresh,
            // §12.4's Retained preview: the `r` mnemonic only surfaces this
            // from Estate/Health with the `disk_pressure` check selected,
            // but the read itself (`GET /v1/retained`) is estate-wide, so
            // the palette reaches it from anywhere, same as /evidence,
            // /graph, and /details reach their tabs outside the mnemonic's
            // own digit-key context.
            SlashCommand::Retained => return Action::LoadRetained,
            SlashCommand::Help => self.overlay = Some(Overlay::Help),
            SlashCommand::Quit => {
                self.quit = true;
                return Action::Quit;
            }
            // §13.10's respond mnemonic (`r`), gated the same way: the
            // composer only gains focus where the action matrix offers it.
            SlashCommand::Answer => {
                if self.work_action_available("respond")
                    && let Some(screen) = self.work_screen.as_mut()
                {
                    screen.focus_answer();
                }
            }
            // Durable/destructive per §15.5: this only ever opens the same
            // confirmation the mnemonic key does — the mutation itself
            // still waits on that overlay's own deliberate Confirm.
            SlashCommand::Cancel => {
                self.open_action_confirmation("cancel", Overlay::CancelConfirmation)
            }
            SlashCommand::Retry => {
                self.open_action_confirmation("retry", Overlay::RetryConfirmation)
            }
            SlashCommand::Extend => {
                self.open_action_confirmation("extend", Overlay::ExtendEnvelope)
            }
            SlashCommand::Reap => self.open_action_confirmation("reap", Overlay::ReapConfirmation),
            SlashCommand::Evidence => self.show_work_tab(Tab::Evidence),
            SlashCommand::Graph => self.show_work_tab(Tab::Graph),
            SlashCommand::Details => self.show_work_tab(Tab::Details),
            // No TUI destination reads the daemon's `/v1/analytics` routes
            // yet (§16.1 lists the client method as a later addition) — said
            // plainly rather than pretending a screen exists.
            SlashCommand::AnalyticsPanel => {
                self.status = "/analytics is not built in this Work yet".to_string();
            }
        }
        Action::None
    }

    /// §15.3's Home/Fleet/Workflows/Estate commands: an open Work has no
    /// meaning once the destination underneath it changes, so it closes
    /// first — the same effect `/back` has on its own.
    fn leave_open_work_and_goto(&mut self, destination: Destination) {
        self.open_work = None;
        self.work_screen = None;
        self.goto(destination);
    }

    /// Whether `action` is legal for the currently open Work's reported
    /// state (§13.10) — `false` with no Work open, so every gated slash
    /// command is a quiet no-op outside a canonical Work surface.
    fn work_action_available(&self, action: &str) -> bool {
        self.work_screen
            .as_ref()
            .is_some_and(|screen| screen.action_available(action))
    }

    /// §15.3's cancel/retry/extend/reap commands: open the same confirmation
    /// overlay the mnemonic key does, only where the action matrix offers
    /// it — otherwise a no-op, exactly like the mnemonic key itself.
    fn open_action_confirmation(&mut self, action: &str, overlay: Overlay) {
        if self.work_action_available(action) {
            self.pending_action = PendingAction::default();
            self.overlay = Some(overlay);
        }
    }

    /// §15.3's evidence/graph/details commands: switch the open Work's tab,
    /// the same effect the `3`/`4`/`5` digit keys have — a no-op with no
    /// Work open.
    fn show_work_tab(&mut self, tab: Tab) {
        if let Some(screen) = self.work_screen.as_mut() {
            screen.show_tab(tab);
        }
    }

    fn on_key_overlay(&mut self, key: KeyEvent) -> Action {
        let Some(overlay) = self.overlay else {
            return Action::None;
        };
        match overlay {
            // T1c owns render real, live content but share the same fixed
            // set of controls: Esc/q aborts, Enter (on Confirm) fires the
            // mutation. Tab has nothing to move to.
            Overlay::CancelConfirmation | Overlay::RetryConfirmation => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.close_overlay(),
                    KeyCode::Enter => {
                        let Some(id) = self.open_work_id() else {
                            self.close_overlay();
                            return Action::None;
                        };
                        return match overlay {
                            Overlay::CancelConfirmation => Action::Cancel(id),
                            Overlay::RetryConfirmation => Action::Retry(id),
                            _ => unreachable!("matched above"),
                        };
                    }
                    _ => {}
                }
                Action::None
            }
            // Same fixed controls as above, but the Work id can come from
            // either an open Work surface (§13.10) or §12.4's Retained
            // preview — [`App::retained_reap_id`] is Estate's own source,
            // and takes precedence over an open Work when both are present,
            // so this stays the one Reap implementation either way.
            Overlay::ReapConfirmation => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        if self.retained_reap_id.is_some() {
                            self.overlay = Some(Overlay::RetainedPreview);
                            self.pending_action = PendingAction::default();
                            self.retained_reap_id = None;
                        } else {
                            self.close_overlay();
                        }
                    }
                    KeyCode::Enter => {
                        // `retained_reap_id` is the explicit selection made
                        // inside §12.4's Retained preview, so it takes
                        // precedence over an open Work — `/retained`
                        // (§15.3) can now reach this preview with a Work
                        // open, and reaping must target what the user
                        // actually picked there, not whatever Work happens
                        // to be open underneath.
                        let Some(id) = self
                            .retained_reap_id
                            .clone()
                            .or_else(|| self.open_work_id())
                        else {
                            self.close_overlay();
                            return Action::None;
                        };
                        return Action::Reap(id);
                    }
                    _ => {}
                }
                Action::None
            }
            // §12.1's Add/Remove Repository panel — form input reaches
            // `RepoForm::on_key`, and the outcome either closes the overlay
            // locally (an abort) or asks the loop to run the mutation.
            Overlay::RepoAddRemove => {
                if self.estate.pending_repo_add.is_some() {
                    if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                        self.overlay = None;
                    }
                    return Action::None;
                }
                match self.estate.repo_form.on_key(key) {
                    RepoFormOutcome::None => {}
                    RepoFormOutcome::Close => self.overlay = None,
                    RepoFormOutcome::Submit {
                        name,
                        origin,
                        instructions,
                    } => {
                        return Action::AddRepo {
                            name,
                            origin,
                            instructions,
                        };
                    }
                    RepoFormOutcome::ConfirmRemove(name) => return Action::RemoveRepo(name),
                }
                Action::None
            }
            // §12.2's create/extend/remove-members/remove-group panel, same
            // shape as `RepoAddRemove` above.
            Overlay::GroupEditRemove => {
                match self.estate.group_form.on_key(key) {
                    GroupFormOutcome::None => {}
                    GroupFormOutcome::Close => self.overlay = None,
                    GroupFormOutcome::Submit { name, repos, brief } => {
                        return Action::AddGroup { name, repos, brief };
                    }
                    GroupFormOutcome::ConfirmRemove { name, repos } => {
                        return Action::RemoveGroup { name, repos };
                    }
                }
                Action::None
            }
            // §12.4's estate-wide retained-state list: browse with j/k,
            // `p`/Enter opens the same [`Overlay::ReapConfirmation`] T1c
            // already built rather than a second reap implementation.
            Overlay::RetainedPreview => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.overlay = None,
                    KeyCode::Down | KeyCode::Char('j') => {
                        let len = self.estate.retained.len();
                        if len > 0 {
                            self.estate.retained_selected =
                                (self.estate.retained_selected + 1) % len;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let len = self.estate.retained.len();
                        if len > 0 {
                            self.estate.retained_selected =
                                (self.estate.retained_selected + len - 1) % len;
                        }
                    }
                    KeyCode::Char('p') | KeyCode::Enter => {
                        if let Some(id) = self
                            .estate
                            .selected_retained()
                            .and_then(|e| e["work_id"].as_str())
                        {
                            self.retained_reap_id = Some(id.to_string());
                            self.pending_action = PendingAction::default();
                            self.overlay = Some(Overlay::ReapConfirmation);
                        }
                    }
                    _ => {}
                }
                Action::None
            }
            Overlay::ExtendEnvelope => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.close_overlay(),
                    KeyCode::Tab | KeyCode::BackTab => {
                        self.pending_action.confirm_focused = !self.pending_action.confirm_focused;
                    }
                    KeyCode::Backspace if !self.pending_action.confirm_focused => {
                        self.pending_action.extend_turns.pop();
                    }
                    KeyCode::Char(c)
                        if !self.pending_action.confirm_focused && c.is_ascii_digit() =>
                    {
                        self.pending_action.extend_turns.push(c);
                    }
                    KeyCode::Enter if !self.pending_action.confirm_focused => {
                        self.pending_action.confirm_focused = true;
                    }
                    KeyCode::Enter if self.pending_action.confirm_focused => {
                        let parsed = self.pending_action.extend_turns.trim().parse::<u32>();
                        let Some(id) = self.open_work_id() else {
                            self.close_overlay();
                            return Action::None;
                        };
                        match parsed {
                            Ok(additional_turns) if additional_turns > 0 => {
                                return Action::Extend {
                                    id,
                                    additional_turns,
                                };
                            }
                            _ => {
                                self.pending_action.last_error = Some(
                                    "enter a whole number of turns greater than zero".to_string(),
                                );
                            }
                        }
                    }
                    _ => {}
                }
                Action::None
            }
            // §15.4/§11.4's live-catalog chooser, opened either from Home's
            // workflow field or from the Workflows screen itself (issue
            // #153): j/k move the highlighted entry, Esc/q aborts with
            // nothing touched. Enter's destination depends on which screen
            // asked for it — `self.destination` still names that screen,
            // since opening the overlay never changes it: Home gets the
            // pick onto its workflow field, Workflows moves its own
            // selection onto the picked entry instead.
            Overlay::WorkflowChooser => {
                let entries = self.workflows.entries.len();
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.close_overlay(),
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.workflow_chooser_index + 1 < entries {
                            self.workflow_chooser_index += 1;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.workflow_chooser_index = self.workflow_chooser_index.saturating_sub(1);
                    }
                    KeyCode::Enter => {
                        let picked = self
                            .workflows
                            .entries
                            .get(self.workflow_chooser_index)
                            .and_then(|e| e["name"].as_str())
                            .map(str::to_string);
                        if let Some(name) = picked {
                            match self.destination {
                                Destination::Workflows => self.workflows.select_by_name(&name),
                                _ => self.home.workflow = name,
                            }
                        }
                        self.close_overlay();
                    }
                    _ => {}
                }
                Action::None
            }
            // §15.3's fixed vocabulary picker: j/k move the highlighted
            // command, Enter runs it (`run_slash_command`, which closes the
            // palette itself), Esc/q aborts with nothing run — the same
            // shape as `Overlay::WorkflowChooser` above.
            Overlay::SlashPalette => {
                let len = SlashCommand::ALL.len();
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => self.close_overlay(),
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.slash_palette_index + 1 < len {
                            self.slash_palette_index += 1;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.slash_palette_index = self.slash_palette_index.saturating_sub(1);
                    }
                    KeyCode::Enter => {
                        return self.run_slash_command(SlashCommand::ALL[self.slash_palette_index]);
                    }
                    _ => {}
                }
                Action::None
            }
            // Help and ConnectionDetail have real content (T1a and issue
            // #154 respectively) but need nothing beyond the generic
            // close-only controls this Work built at T1a — open, close,
            // restore focus for free. RepoAddRemove, GroupEditRemove, and
            // RetainedPreview have their own real arms above (T3);
            // WorkflowChooser has its own real arm above (T2).
            Overlay::Help | Overlay::ConnectionDetail => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => self.overlay = None,
                    _ => {}
                }
                Action::None
            }
        }
    }

    /// Every key inside an open Work surface is the surface's own to
    /// interpret (§13.2's tab switching and each tab's local navigation) —
    /// `App` only acts on the outcomes that reach outside the surface
    /// itself: closing it, an API round trip Evidence's bounded "load
    /// older" needs, a deliberately submitted Answer (§15.1/§15.2), and
    /// opening one of §13.10's confirmation overlays (the mutation itself
    /// waits on that overlay's own deliberate Confirm, per §15.5).
    fn on_key_open_work(&mut self, key: KeyEvent) -> Action {
        let Some(screen) = self.work_screen.as_mut() else {
            // The opening fetch is still in flight: only Esc/q is
            // meaningful yet (there is nothing else to navigate).
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                self.open_work = None;
            }
            return Action::None;
        };
        match screen.on_key(key) {
            WorkScreenOutcome::None => Action::None,
            WorkScreenOutcome::Close => {
                self.open_work = None;
                self.work_screen = None;
                Action::None
            }
            WorkScreenOutcome::LoadOlder => Action::LoadOlderEvidence,
            WorkScreenOutcome::Respond(input) => {
                let Some(id) = self.open_work_id() else {
                    return Action::None;
                };
                Action::Respond { id, input }
            }
            WorkScreenOutcome::AnswerRefused => {
                self.status = "answer is empty".to_string();
                Action::None
            }
            WorkScreenOutcome::OpenCancelConfirm => {
                self.pending_action = PendingAction::default();
                self.overlay = Some(Overlay::CancelConfirmation);
                Action::None
            }
            WorkScreenOutcome::OpenRetryConfirm => {
                self.pending_action = PendingAction::default();
                self.overlay = Some(Overlay::RetryConfirmation);
                Action::None
            }
            WorkScreenOutcome::OpenExtendEnvelope => {
                self.pending_action = PendingAction::default();
                self.overlay = Some(Overlay::ExtendEnvelope);
                Action::None
            }
            WorkScreenOutcome::OpenReapConfirm => {
                self.pending_action = PendingAction::default();
                self.overlay = Some(Overlay::ReapConfirmation);
                Action::None
            }
            // §15.3: `/` at the first non-whitespace position of the
            // ANSWER composer, same wiring shape as the four confirmations
            // above.
            WorkScreenOutcome::OpenSlashPalette => {
                self.slash_palette_index = 0;
                self.overlay = Some(Overlay::SlashPalette);
                Action::None
            }
            // No local WorkScreen binding claimed this key: let the same
            // global keymap browsing uses everywhere else see it, so e.g.
            // `c` reaches `Overlay::ConnectionDetail` whenever cancel isn't
            // currently offered on the open Work.
            WorkScreenOutcome::Unhandled => {
                self.on_key_global(key.code).unwrap_or(Action::None)
            }
        }
    }

    /// Execute an action against the API and record its outcome.
    ///
    /// A write becomes a [`Action::Refresh`] — the screen re-reads the API
    /// rather than believing the response — and everything else passes
    /// through for the loop to act on.
    pub async fn execute(&mut self, client: &ApiClient, action: Action) -> Action {
        match action {
            Action::Submit(body) => {
                match client.post("/v1/work", &body).await {
                    Ok(result) => {
                        let id = result["work"]["id"].as_str().unwrap_or("?").to_string();
                        self.status = format!("submitted {id}");
                        self.home.clear_draft();
                    }
                    Err(e) => {
                        self.home.last_error = Some(format!("submit failed: {e}"));
                        self.status = format!("submit failed: {e}");
                    }
                }
                Action::Refresh
            }
            Action::OpenWork(id) => {
                match WorkScreen::load(client, &id).await {
                    Ok(screen) => {
                        self.work_screen = Some(screen);
                        self.open_work = Some(OpenWork {
                            id,
                            from: self.destination,
                        });
                    }
                    Err(e) => {
                        self.status = format!("could not open work {id}: {e}");
                    }
                }
                Action::None
            }
            Action::LoadOlderEvidence => {
                if let Some(screen) = &mut self.work_screen
                    && let Err(e) = screen.load_older(client).await
                {
                    self.status = format!("could not load older evidence: {e}");
                }
                Action::None
            }
            // §15.1's Answer, deliberately submitted. The draft clears only
            // once the daemon accepts it (§15.2/§9.2's rule, reused here) —
            // a failure leaves it exactly as the operator wrote it.
            Action::Respond { id, input } => {
                match client.respond(&id, &input).await {
                    Ok(_) => {
                        self.status = format!("responded to {id}");
                        if let Some(screen) = &mut self.work_screen {
                            screen.clear_answer();
                        }
                    }
                    Err(e) => {
                        self.status = format!("respond failed: {e}");
                    }
                }
                Action::Refresh
            }
            // §13.10's closing lines: the daemon is authoritative, so every
            // one of these four always asks for a refresh — a race that
            // rejects the mutation still gets the true current state, and
            // the error is recorded on the still-open overlay rather than
            // the overlay silently vanishing (§15.5).
            Action::Cancel(id) => {
                match client.cancel(&id).await {
                    Ok(_) => {
                        self.status = format!("canceled {id}");
                        self.close_overlay();
                    }
                    Err(e) => {
                        self.status = format!("cancel failed: {e}");
                        self.pending_action.last_error = Some(e.to_string());
                    }
                }
                Action::Refresh
            }
            Action::Retry(id) => {
                match client.retry(&id).await {
                    Ok(_) => {
                        self.status = format!("retrying {id}");
                        self.close_overlay();
                    }
                    Err(e) => {
                        self.status = format!("retry failed: {e}");
                        self.pending_action.last_error = Some(e.to_string());
                    }
                }
                Action::Refresh
            }
            Action::Extend {
                id,
                additional_turns,
            } => {
                match client.extend(&id, additional_turns).await {
                    Ok(_) => {
                        self.status = format!("extended {id} by {additional_turns} turns");
                        self.close_overlay();
                    }
                    Err(e) => {
                        self.status = format!("extend failed: {e}");
                        self.pending_action.last_error = Some(e.to_string());
                    }
                }
                Action::Refresh
            }
            Action::Reap(id) => {
                match client.reap(&id, true).await {
                    Ok(_) => {
                        self.status = format!("reaped {id}");
                        self.close_overlay();
                    }
                    Err(e) => {
                        self.status = format!("reap failed: {e}");
                        self.pending_action.last_error = Some(e.to_string());
                    }
                }
                Action::Refresh
            }
            // §12.1: kicked off, not awaited — a real `git clone` can take
            // real wall-clock time, and [`App::poll_background`] (called
            // every loop tick, key or not) is what notices it finished. The
            // overlay's own body shows the spinner and elapsed time while
            // `estate.pending_repo_add` is `Some`.
            Action::AddRepo {
                name,
                origin,
                instructions,
            } => {
                let client = client.clone();
                let (tx, rx) = tokio::sync::oneshot::channel();
                let (spawn_name, spawn_origin, spawn_instructions) =
                    (name.clone(), origin.clone(), instructions.clone());
                tokio::spawn(async move {
                    let result = client
                        .add_repo(
                            &spawn_name,
                            spawn_origin.as_deref(),
                            spawn_instructions.as_deref(),
                        )
                        .await;
                    let _ = tx.send(result);
                });
                self.estate.repo_form.last_error = None;
                self.estate.pending_repo_add = Some(PendingRepoAdd::new(name, rx));
                Action::None
            }
            Action::RemoveRepo(name) => {
                match client.remove_repo(&name).await {
                    Ok(_) => {
                        self.status = format!("removed repo {name}");
                        self.overlay = None;
                    }
                    Err(e) => {
                        self.status = format!("remove repo failed: {e}");
                        self.estate.repo_form.last_error = Some(e.to_string());
                    }
                }
                Action::Refresh
            }
            Action::AddGroup { name, repos, brief } => {
                match client.add_group(&name, &repos, brief.as_deref()).await {
                    Ok(_) => {
                        self.status = format!("group {name} updated");
                        self.overlay = None;
                    }
                    Err(e) => {
                        self.status = format!("group update failed: {e}");
                        self.estate.group_form.last_error = Some(e.to_string());
                    }
                }
                Action::Refresh
            }
            Action::RemoveGroup { name, repos } => {
                match client.remove_group(&name, &repos).await {
                    Ok(_) => {
                        self.status = format!("group {name} updated");
                        self.overlay = None;
                    }
                    Err(e) => {
                        self.status = format!("group remove failed: {e}");
                        self.estate.group_form.last_error = Some(e.to_string());
                    }
                }
                Action::Refresh
            }
            Action::LoadRetained => {
                match client.retained().await {
                    Ok(result) => {
                        self.estate.retained =
                            result["retained"].as_array().cloned().unwrap_or_default();
                        self.estate.retained_selected = 0;
                        self.overlay = Some(Overlay::RetainedPreview);
                    }
                    Err(e) => {
                        self.status = format!("could not load retained state: {e}");
                    }
                }
                Action::None
            }
            other => other,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
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
        })).collect::<Vec<_>>()})
    }

    #[test]
    fn digit_keys_switch_destinations() {
        let mut app = App::new();
        assert_eq!(app.destination, Destination::Home);
        app.on_key(KeyCode::Esc); // leave Home's form focus first
        app.on_key(KeyCode::Char('2'));
        assert_eq!(app.destination, Destination::Fleet);
        app.on_key(KeyCode::Char('3'));
        assert_eq!(app.destination, Destination::Workflows);
        app.on_key(KeyCode::Char('4'));
        assert_eq!(app.destination, Destination::Estate);
        app.on_key(KeyCode::Char('1'));
        assert_eq!(app.destination, Destination::Home);
    }

    #[test]
    fn tab_cycles_destinations_forward_and_back_tab_backward() {
        let mut app = App::new();
        app.destination = Destination::Fleet; // sidestep Home's own Tab use
        app.on_key(KeyCode::Tab);
        assert_eq!(app.destination, Destination::Workflows);
        app.on_key(KeyCode::BackTab);
        assert_eq!(app.destination, Destination::Fleet);
    }

    #[test]
    fn typing_in_the_intent_field_is_never_hijacked_as_a_shortcut() {
        let mut app = App::new();
        assert_eq!(
            app.destination,
            Destination::Home,
            "home focuses the form by default"
        );
        for c in "call 1-800 and say q, then ~ and 4".chars() {
            app.on_key(KeyCode::Char(c));
        }
        assert_eq!(app.home.intent.text(), "call 1-800 and say q, then ~ and 4");
        assert_eq!(
            app.destination,
            Destination::Home,
            "no digit switched destinations"
        );
        assert!(!app.quit, "the literal 'q' did not quit");
        assert!(app.drawer_open, "the literal '~' did not toggle the drawer");
    }

    #[test]
    fn escaping_the_form_returns_the_keyboard_to_global_navigation() {
        let mut app = App::new();
        app.on_key(KeyCode::Esc); // blur, preserving the (empty) draft
        app.on_key(KeyCode::Char('2'));
        assert_eq!(
            app.destination,
            Destination::Fleet,
            "global nav works again once blurred"
        );
    }

    #[test]
    fn re_entering_home_refocuses_the_form() {
        let mut app = App::new();
        app.on_key(KeyCode::Esc);
        app.on_key(KeyCode::Char('2'));
        app.on_key(KeyCode::Char('1'));
        assert_eq!(app.destination, Destination::Home);
        app.on_key(KeyCode::Char('x'));
        assert_eq!(
            app.home.intent.text(),
            "x",
            "the form is editable again, not treated as a nav key"
        );
    }

    #[test]
    fn the_drawer_toggles_and_help_opens_and_closes() {
        let mut app = App::new();
        app.on_key(KeyCode::Esc); // reach global focus from Home
        assert!(app.drawer_open, "opens by default (§7.3)");
        app.on_key(KeyCode::Char('~'));
        assert!(!app.drawer_open);
        app.on_key(KeyCode::Char('~'));
        assert!(app.drawer_open);

        assert!(app.overlay.is_none());
        app.on_key(KeyCode::Char('?'));
        assert_eq!(app.overlay, Some(Overlay::Help));
        app.on_key(KeyCode::Esc);
        assert!(app.overlay.is_none());
    }

    /// Issue #154: unlike every other overlay in §7.4's fixed set,
    /// `Overlay::ConnectionDetail` had no key binding anywhere — `c` opens
    /// it globally (mouse capture stays disabled, §8.9), and it closes the
    /// same way `Help`/`SlashPalette` already do.
    #[test]
    fn connection_detail_opens_on_c_and_closes_on_esc_q_or_question_mark() {
        let mut app = App::new();
        app.on_key(KeyCode::Esc); // reach global focus from Home

        assert!(app.overlay.is_none());
        app.on_key(KeyCode::Char('c'));
        assert_eq!(app.overlay, Some(Overlay::ConnectionDetail));
        app.on_key(KeyCode::Esc);
        assert!(app.overlay.is_none());

        app.on_key(KeyCode::Char('c'));
        assert_eq!(app.overlay, Some(Overlay::ConnectionDetail));
        app.on_key(KeyCode::Char('q'));
        assert!(app.overlay.is_none());

        app.on_key(KeyCode::Char('c'));
        assert_eq!(app.overlay, Some(Overlay::ConnectionDetail));
        app.on_key(KeyCode::Char('?'));
        assert!(app.overlay.is_none());
    }

    #[test]
    fn fleet_enter_asks_to_open_the_canonical_work_surface() {
        let mut app = App::new();
        app.rows = fleet_rows(&fleet_of(&[("a", "running"), ("b", "running")]));
        app.on_key(KeyCode::Esc); // leave Home's form focus
        app.on_key(KeyCode::Char('2')); // Fleet
        assert_eq!(app.destination, Destination::Fleet);
        assert_eq!(
            app.on_key(KeyCode::Enter),
            Action::OpenWork("a".to_string()),
            "opening the real surface needs the API reads WorkScreen::load makes — \
             the keymap only asks for it, `execute` does the fetch"
        );
        assert!(
            app.open_work.is_none(),
            "not open yet: the fetch this needs has not run"
        );
    }

    #[tokio::test]
    async fn opening_a_work_that_fails_to_load_reports_the_error_and_stays_closed() {
        let mut app = App::new();
        app.rows = fleet_rows(&fleet_of(&[("a", "running")]));
        let client = ApiClient::new("http://127.0.0.1:1", "t").expect("client");
        let outcome = app
            .execute(&client, Action::OpenWork("a".to_string()))
            .await;
        assert_eq!(outcome, Action::None);
        assert!(app.open_work.is_none());
        assert!(app.work_screen.is_none());
        assert!(app.status.contains("could not open work"), "{}", app.status);
    }

    #[test]
    fn esc_closes_an_open_work_surface_and_returns_to_where_it_was_opened_from() {
        let mut app = App::new();
        app.rows = fleet_rows(&fleet_of(&[("a", "running")]));
        app.destination = Destination::Fleet;
        app.open_work = Some(OpenWork {
            id: "a".to_string(),
            from: Destination::Fleet,
        });
        app.work_screen = Some(WorkScreen::from_parts(
            "a".to_string(),
            json!({"work": {"id": "a", "intent": "x", "state": "running"}}),
            Vec::new(),
            Vec::new(),
            None,
        ));
        app.on_key(KeyCode::Esc);
        assert!(app.open_work.is_none());
        assert!(app.work_screen.is_none());
        assert_eq!(
            app.destination,
            Destination::Fleet,
            "back to exactly where it was opened from"
        );
    }

    #[test]
    fn q_quits_only_from_top_level_browsing() {
        let mut app = App::new();
        app.on_key(KeyCode::Esc); // Home's form would otherwise eat 'q'
        assert_eq!(app.on_key(KeyCode::Char('q')), Action::Quit);
        assert!(app.quit);
    }

    #[test]
    fn q_closes_an_open_overlay_or_work_view_instead_of_quitting() {
        let mut app = App::new();
        app.overlay = Some(Overlay::Help);
        assert_eq!(app.on_key(KeyCode::Char('q')), Action::None);
        assert!(app.overlay.is_none());
        assert!(!app.quit);

        app.rows = fleet_rows(&fleet_of(&[("a", "running")]));
        app.open_work = Some(OpenWork {
            id: "a".to_string(),
            from: Destination::Fleet,
        });
        assert_eq!(app.on_key(KeyCode::Char('q')), Action::None);
        assert!(app.open_work.is_none());
        assert!(!app.quit);
    }

    #[test]
    fn only_state_bearing_events_ask_for_a_refresh() {
        let mut app = App::new();
        assert!(app.observe(&json!({"seq": 7, "kind": "work.completed"})));
        assert!(!app.observe(&json!({"seq": 8, "kind": "daemon.started"})));
        assert_eq!(app.last_seq, 8, "the resume point tracks every event");
    }

    #[tokio::test]
    async fn submitting_an_unreachable_daemon_preserves_the_draft_and_reports_the_error() {
        let mut app = App::new();
        app.home.intent.set_text("fix the thing");
        let client = ApiClient::new("http://127.0.0.1:1", "t").expect("client");
        // Route straight through the executor with a hand-built body instead
        // of walking every Tab press — the form's own submission-shape tests
        // already cover field mapping.
        let body = json!({"intent": "fix the thing", "command_id": "x"});
        let outcome = app.execute(&client, Action::Submit(body)).await;
        assert_eq!(
            outcome,
            Action::Refresh,
            "a write always asks for a refresh"
        );
        assert!(app.home.last_error.is_some(), "the failure is reported");
        assert_eq!(
            app.home.intent.text(),
            "fix the thing",
            "and the draft survives it"
        );
    }

    // ------------------------------------------------- T1c: mutation wiring

    /// An app with `id` open at `state` — every mutation-keymap test starts
    /// from here, since respond/cancel/retry/extend/reap only ever reach the
    /// keymap from inside the canonical Work surface.
    fn app_with_open_work(id: &str, state: &str) -> App {
        let mut app = App::new();
        app.rows = fleet_rows(&fleet_of(&[(id, state)]));
        app.destination = Destination::Fleet;
        app.open_work = Some(OpenWork {
            id: id.to_string(),
            from: Destination::Fleet,
        });
        app.work_screen = Some(WorkScreen::from_parts(
            id.to_string(),
            json!({
                "work": {"id": id, "intent": "fix the thing", "state": state},
                "stage": {"stage_id": "10-implement", "attempt": 2},
                "envelope": {"turn_cap": 12, "turns_spawned": 3},
            }),
            Vec::new(),
            Vec::new(),
            None,
        ));
        app
    }

    #[test]
    fn c_opens_the_cancel_confirmation_and_esc_aborts_it_without_effect() {
        let mut app = app_with_open_work("a", "blocked");
        assert_eq!(app.on_key(KeyCode::Char('c')), Action::None);
        assert_eq!(app.overlay, Some(Overlay::CancelConfirmation));

        assert_eq!(app.on_key(KeyCode::Esc), Action::None);
        assert!(app.overlay.is_none(), "Esc aborts, no request queued");
    }

    #[test]
    fn enter_on_the_cancel_confirmation_asks_to_cancel_the_open_work() {
        let mut app = app_with_open_work("a", "blocked");
        app.on_key(KeyCode::Char('c'));
        assert_eq!(
            app.on_key(KeyCode::Enter),
            Action::Cancel("a".to_string()),
            "the deliberate Enter on Confirm is what actually asks to mutate"
        );
    }

    #[test]
    fn an_action_key_is_a_no_op_when_the_matrix_does_not_offer_it() {
        // completed offers only "inspect" (§13.10) — retry/extend/reap/
        // respond must all be no-ops, not silently open a confirmation for a
        // mutation the daemon would refuse anyway. `c` is exempt: when
        // cancel isn't offered it falls through to the global keymap and
        // opens `Overlay::ConnectionDetail` instead (issue #154's follow-up,
        // covered separately below), so it isn't a plain no-op here.
        let mut app = app_with_open_work("a", "completed");
        for key in ['t', 'e', 'p', 'r'] {
            assert_eq!(app.on_key(KeyCode::Char(key)), Action::None);
            assert!(app.overlay.is_none(), "{key} must not open anything");
        }
    }

    /// Issue #154's follow-up: while a Work is open, `c` is the cancel
    /// mnemonic exactly when cancel is offered (§13.10) — and falls through
    /// to the same global `c` binding that opens `Overlay::ConnectionDetail`
    /// everywhere else the moment cancel is not offered, rather than being
    /// silently swallowed by the WorkScreen catch-all.
    #[test]
    fn c_opens_connection_detail_in_open_work_when_cancel_is_not_offered_and_still_cancels_when_it_is(
    ) {
        let mut app = app_with_open_work("a", "completed");
        assert!(!app.work_screen.as_ref().unwrap().action_available("cancel"));
        assert_eq!(app.on_key(KeyCode::Char('c')), Action::None);
        assert_eq!(app.overlay, Some(Overlay::ConnectionDetail));
        // still inside the open Work — didn't close it or the confirmation
        // matrix by accident.
        assert!(app.open_work.is_some());

        app.on_key(KeyCode::Esc); // close the overlay, back to the open Work
        assert!(app.overlay.is_none());
        assert!(app.open_work.is_some());

        let mut app = app_with_open_work("a", "blocked");
        assert!(app.work_screen.as_ref().unwrap().action_available("cancel"));
        assert_eq!(app.on_key(KeyCode::Char('c')), Action::None);
        assert_eq!(
            app.overlay,
            Some(Overlay::CancelConfirmation),
            "cancel still wins whenever it's offered"
        );
    }

    #[test]
    fn t_opens_retry_and_p_opens_reap_only_where_the_matrix_allows_them() {
        let mut app = app_with_open_work("a", "failed");
        assert_eq!(app.on_key(KeyCode::Char('t')), Action::None);
        assert_eq!(app.overlay, Some(Overlay::RetryConfirmation));
        assert_eq!(
            app.on_key(KeyCode::Char('p')),
            Action::None,
            "reap is not offered for failed"
        );

        let mut app = app_with_open_work("a", "completed_dirty");
        assert_eq!(app.on_key(KeyCode::Char('p')), Action::None);
        assert_eq!(app.overlay, Some(Overlay::ReapConfirmation));
    }

    #[test]
    fn extend_requires_a_nonzero_turn_count_before_confirming() {
        let mut app = app_with_open_work("a", "blocked");
        assert_eq!(app.on_key(KeyCode::Char('e')), Action::None);
        assert_eq!(app.overlay, Some(Overlay::ExtendEnvelope));

        // Confirm with nothing typed: refused, overlay stays open with a
        // reason rather than silently doing nothing (§15.5).
        app.on_key(KeyCode::Tab); // field -> Confirm
        assert_eq!(app.on_key(KeyCode::Enter), Action::None);
        assert!(app.overlay.is_some(), "the overlay stays open on refusal");
        assert!(app.pending_action.last_error.is_some());

        // Back to the field, type digits, confirm for real.
        app.on_key(KeyCode::Tab); // Confirm -> field
        for c in "5".chars() {
            app.on_key(KeyCode::Char(c));
        }
        app.on_key(KeyCode::Tab); // field -> Confirm
        assert_eq!(
            app.on_key(KeyCode::Enter),
            Action::Extend {
                id: "a".to_string(),
                additional_turns: 5,
            }
        );
    }

    #[test]
    fn r_focuses_the_answer_composer_and_ctrl_enter_asks_to_respond() {
        let mut app = app_with_open_work("a", "needs_input");
        assert_eq!(app.on_key(KeyCode::Char('r')), Action::None);
        for c in "go ahead".chars() {
            app.on_key(KeyCode::Char(c));
        }
        let ctrl_enter = KeyEvent::new(
            KeyCode::Enter,
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        );
        assert_eq!(
            app.on_key(ctrl_enter),
            Action::Respond {
                id: "a".to_string(),
                input: "go ahead".to_string(),
            }
        );
    }

    #[test]
    fn esc_on_reap_confirmation_opened_from_retained_preview_restores_the_preview() {
        // §12.4: Reap reached from the Retained preview (no open Work
        // behind it) must back out to that preview on Esc/q, not vanish
        // to no overlay at all — the review fix this guards against a
        // regression of.
        let mut app = App::new();
        app.retained_reap_id = Some("01RET".to_string());
        app.overlay = Some(Overlay::ReapConfirmation);

        assert_eq!(app.on_key(KeyCode::Esc), Action::None);
        assert_eq!(
            app.overlay,
            Some(Overlay::RetainedPreview),
            "Esc must restore the Retained preview, not close everything"
        );
        assert!(
            app.retained_reap_id.is_none(),
            "the reap target is cleared once the confirmation is backed out of"
        );
    }

    #[test]
    fn esc_on_reap_confirmation_opened_from_an_open_work_still_closes_fully() {
        // The open-Work path (§13.10) has no Retained preview to return
        // to, so Esc must behave as it always has: close the overlay.
        let mut app = app_with_open_work("a", "completed_dirty");
        app.on_key(KeyCode::Char('p'));
        assert_eq!(app.overlay, Some(Overlay::ReapConfirmation));

        assert_eq!(app.on_key(KeyCode::Esc), Action::None);
        assert!(app.overlay.is_none(), "no Retained preview to fall back to");
    }

    #[test]
    fn enter_on_reap_confirmation_with_both_an_open_work_and_a_retained_selection_reaps_the_retained_id()
     {
        // §15.3's `/retained` can now open the Retained preview without
        // first closing an open Work, so both `open_work` and
        // `retained_reap_id` can be set at once when Enter confirms the
        // reap — the explicit Retained selection must still win, exactly
        // as `reap_identity` displays it.
        let mut app = app_with_open_work("a", "completed_dirty");
        app.retained_reap_id = Some("01RET".to_string());
        app.overlay = Some(Overlay::ReapConfirmation);

        assert_eq!(
            app.on_key(KeyCode::Enter),
            Action::Reap("01RET".to_string()),
            "the Retained selection takes precedence over the open Work's id"
        );
    }

    #[test]
    fn repo_add_remove_ignores_further_keys_while_a_repo_add_is_pending() {
        // §12.1's review fix: once a background AddRepo is in flight,
        // repeated Enter presses on the still-open overlay must not
        // resubmit — only Esc/q may act, and only to close.
        let mut app = App::new();
        app.overlay = Some(Overlay::RepoAddRemove);
        let (_tx, rx) = tokio::sync::oneshot::channel();
        app.estate.pending_repo_add = Some(PendingRepoAdd::new("svc-a".to_string(), rx));

        assert_eq!(
            app.on_key(KeyCode::Enter),
            Action::None,
            "Enter while pending must not fire a second AddRepo"
        );
        assert_eq!(
            app.overlay,
            Some(Overlay::RepoAddRemove),
            "the overlay stays open showing the spinner"
        );
        assert!(
            app.estate.pending_repo_add.is_some(),
            "the in-flight add is left untouched"
        );

        assert_eq!(app.on_key(KeyCode::Esc), Action::None);
        assert!(app.overlay.is_none(), "Esc still closes the overlay");
    }

    #[tokio::test]
    async fn a_rejected_cancel_keeps_the_confirmation_open_with_the_error_and_still_refreshes() {
        let mut app = app_with_open_work("a", "blocked");
        app.overlay = Some(Overlay::CancelConfirmation);
        let client = ApiClient::new("http://127.0.0.1:1", "t").expect("client");
        let outcome = app.execute(&client, Action::Cancel("a".to_string())).await;
        assert_eq!(outcome, Action::Refresh, "a race still triggers a refresh");
        assert_eq!(
            app.overlay,
            Some(Overlay::CancelConfirmation),
            "the screen (the open confirmation) is preserved, not silently closed"
        );
        assert!(app.pending_action.last_error.is_some());
    }

    // --------------------------------------------------- T2: workflow discovery

    fn app_with_workflows(names: &[&str]) -> App {
        let mut app = App::new();
        app.destination = Destination::Workflows;
        app.workflows.set_entries(
            names
                .iter()
                .map(|name| {
                    json!({
                        "name": name,
                        "version": "1",
                        "source": "/repo/.sergeant/workflows/".to_string() + name,
                        "content_hash": "a".repeat(64),
                        "status": "published",
                        "stages": [],
                    })
                })
                .collect(),
        );
        app
    }

    /// §15.4/§11.4: choosing "Use in New Work" from Workflows hands the
    /// selected name to Home's field and switches there, refocused exactly
    /// as pressing `1` would.
    #[test]
    fn use_in_new_work_sets_homes_workflow_field_and_switches_to_home() {
        let mut app = app_with_workflows(&["implement", "diagnose-bug"]);
        app.workflows.on_key(KeyCode::Char('j')); // select diagnose-bug
        assert_eq!(app.on_key(KeyCode::Char('u')), Action::None);
        assert_eq!(app.destination, Destination::Home);
        assert_eq!(app.home.workflow, "diagnose-bug");
    }

    /// §15.4's Home half: `@` on the workflow field opens the live-catalog
    /// chooser; j/k move the highlight; Enter selects it onto the field and
    /// closes the overlay.
    #[test]
    fn at_sign_on_homes_workflow_field_opens_the_chooser_and_enter_selects_it() {
        let mut app = App::new();
        app.workflows.set_entries(vec![
            json!({"name": "implement", "version": "1", "source": "/x", "content_hash": "a", "stages": []}),
            json!({"name": "diagnose-bug", "version": "1", "source": "/x", "content_hash": "b", "stages": []}),
        ]);
        app.on_key(KeyCode::Tab); // intent -> workflow
        assert_eq!(app.on_key(KeyCode::Char('@')), Action::None);
        assert_eq!(app.overlay, Some(Overlay::WorkflowChooser));

        app.on_key(KeyCode::Char('j'));
        assert_eq!(app.on_key(KeyCode::Enter), Action::None);
        assert!(app.overlay.is_none(), "selecting closes the chooser");
        assert_eq!(app.home.workflow, "diagnose-bug");
    }

    /// Esc/q aborts the chooser without touching the field it was opened
    /// from (§15.5-shaped: nothing is applied on abort).
    #[test]
    fn esc_aborts_the_workflow_chooser_without_changing_the_field() {
        let mut app = App::new();
        app.home.workflow = "implement".to_string();
        app.workflows.set_entries(vec![
            json!({"name": "diagnose-bug", "version": "1", "source": "/x", "content_hash": "b", "stages": []}),
        ]);
        app.on_key(KeyCode::Tab);
        app.on_key(KeyCode::Char('@'));
        assert_eq!(app.on_key(KeyCode::Esc), Action::None);
        assert!(app.overlay.is_none());
        assert_eq!(app.home.workflow, "implement", "the field is untouched");
    }

    /// The `/` filter field on the Workflows destination owns the keyboard
    /// exactly like Home's and Fleet's own text fields — `2`/`4` etc. must
    /// not be hijacked as destination-nav while typing a filter.
    #[test]
    fn workflows_slash_filter_owns_the_keyboard_like_any_other_text_field() {
        let mut app = app_with_workflows(&["implement", "diagnose-bug"]);
        app.on_key(KeyCode::Char('/'));
        app.on_key(KeyCode::Char('4')); // would otherwise jump to Estate
        assert_eq!(app.destination, Destination::Workflows);
        app.on_key(KeyCode::Esc);
        app.on_key(KeyCode::Char('4'));
        assert_eq!(
            app.destination,
            Destination::Estate,
            "global nav works again once the filter is left"
        );
    }

    /// §15.4's Workflows half (issue #153): `@` while focused on the
    /// Workflows screen itself opens the same live-catalog chooser Home's
    /// workflow field opens; Enter here moves this screen's own selection
    /// onto the picked entry instead of touching Home's field (`Overlay::
    /// WorkflowChooser`'s Enter arm tells the two contexts apart by
    /// `self.destination`, which opening the overlay never changes).
    #[test]
    fn at_sign_on_workflows_opens_the_chooser_and_enter_selects_it() {
        let mut app = app_with_workflows(&["implement", "diagnose-bug"]);
        assert_eq!(app.on_key(KeyCode::Char('@')), Action::None);
        assert_eq!(app.overlay, Some(Overlay::WorkflowChooser));

        app.on_key(KeyCode::Char('j'));
        assert_eq!(app.on_key(KeyCode::Enter), Action::None);
        assert!(app.overlay.is_none(), "selecting closes the chooser");
        assert_eq!(
            app.destination,
            Destination::Workflows,
            "picking from Workflows stays on Workflows, unlike Home's field pick"
        );
        assert_eq!(
            app.workflows.selected_entry().unwrap()["name"],
            "diagnose-bug"
        );
        assert_eq!(
            app.home.workflow, "",
            "Home's own field is untouched by a pick made from Workflows"
        );
    }

    /// Esc/q aborts the chooser without changing Workflows' own selection.
    #[test]
    fn esc_aborts_the_workflow_chooser_opened_from_workflows_without_changing_selection() {
        let mut app = app_with_workflows(&["implement", "diagnose-bug"]);
        app.on_key(KeyCode::Char('@'));
        app.on_key(KeyCode::Char('j')); // highlight diagnose-bug in the chooser
        assert_eq!(app.on_key(KeyCode::Esc), Action::None);
        assert!(app.overlay.is_none());
        assert_eq!(
            app.workflows.selected_entry().unwrap()["name"],
            "implement",
            "the underlying selection is untouched"
        );
    }

    #[tokio::test]
    async fn a_failed_respond_preserves_the_draft() {
        let mut app = app_with_open_work("a", "needs_input");
        app.on_key(KeyCode::Char('r'));
        for c in "the answer".chars() {
            app.on_key(KeyCode::Char(c));
        }
        let client = ApiClient::new("http://127.0.0.1:1", "t").expect("client");
        let outcome = app
            .execute(
                &client,
                Action::Respond {
                    id: "a".to_string(),
                    input: "the answer".to_string(),
                },
            )
            .await;
        assert_eq!(outcome, Action::Refresh);
        assert!(app.status.contains("respond failed"), "{}", app.status);
        assert_eq!(
            app.work_screen.as_ref().unwrap().answer_text_for_test(),
            "the answer",
            "a failed respond must not clear the draft"
        );
    }
}

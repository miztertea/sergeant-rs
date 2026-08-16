//! The application shell's state machine: top-level navigation (§7.1),
//! the global Attention drawer's open/closed flag (§7.3), the overlay
//! mechanism (§7.4), and the canonical-Work stub (§7.2) — composed over the
//! per-destination screens in [`super::home`], [`super::fleet`],
//! [`super::workflows`], and [`super::estate`].

use ratatui::crossterm::event::KeyCode;
use serde_json::Value;

use crate::api::{ApiClient, ClientError};

use super::connection::Live;
use super::fleet::{FleetOutcome, FleetScreen, WorkRow, fleet_rows};
use super::home::{HomeOutcome, NewWorkForm};
use super::overlay::Overlay;
use super::work_view::{WorkScreen, WorkScreenOutcome};

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
    /// The canonical Work surface, if one is open over `destination`.
    pub open_work: Option<OpenWork>,
    /// The open Work surface's own fetched data and tab/scroll state
    /// (§13) — `None` while `open_work` is `Some` but the opening fetch is
    /// still in flight.
    pub work_screen: Option<WorkScreen>,
    /// The one open overlay, if any (§7.4).
    pub overlay: Option<Overlay>,
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
            open_work: None,
            work_screen: None,
            overlay: None,
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
            Destination::Workflows | Destination::Estate => false,
        }
    }

    /// Interpret one keystroke.
    pub fn on_key(&mut self, key: KeyCode) -> Action {
        if self.overlay.is_some() {
            return self.on_key_overlay(key);
        }
        if self.open_work.is_some() {
            return self.on_key_open_work(key);
        }

        if !self.wants_text_focus()
            && let Some(action) = self.on_key_global(key)
        {
            return action;
        }

        match self.destination {
            Destination::Home => match self.home.on_key(key) {
                HomeOutcome::None => Action::None,
                HomeOutcome::Submit(body) => Action::Submit(body),
            },
            Destination::Fleet => match self.fleet.on_key(key, &self.rows) {
                FleetOutcome::None => Action::None,
                FleetOutcome::Open(id) => Action::OpenWork(id),
            },
            Destination::Workflows | Destination::Estate => Action::None,
        }
    }

    /// The keys that mean the same thing everywhere browsing is possible.
    /// `None` means "not a global key — let the destination see it."
    fn on_key_global(&mut self, key: KeyCode) -> Option<Action> {
        match key {
            KeyCode::Char('~') => self.drawer_open = !self.drawer_open,
            KeyCode::Char('?') => self.overlay = Some(Overlay::Help),
            KeyCode::Char('1') => self.goto(Destination::Home),
            KeyCode::Char('2') => self.goto(Destination::Fleet),
            KeyCode::Char('3') => self.goto(Destination::Workflows),
            KeyCode::Char('4') => self.goto(Destination::Estate),
            KeyCode::Tab => self.goto(self.destination.next()),
            KeyCode::BackTab => self.goto(self.destination.prev()),
            KeyCode::Char('q') | KeyCode::Esc => {
                self.quit = true;
                return Some(Action::Quit);
            }
            _ => return None,
        }
        Some(Action::None)
    }

    fn on_key_overlay(&mut self, key: KeyCode) -> Action {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => self.overlay = None,
            _ => {}
        }
        Action::None
    }

    /// Every key inside an open Work surface is the surface's own to
    /// interpret (§13.2's tab switching and each tab's local navigation) —
    /// `App` only acts on the two outcomes that reach outside the surface
    /// itself: closing it, and asking for an API round trip Evidence's
    /// bounded "load older" needs.
    fn on_key_open_work(&mut self, key: KeyCode) -> Action {
        let Some(screen) = self.work_screen.as_mut() else {
            // The opening fetch is still in flight: only Esc/q is
            // meaningful yet (there is nothing else to navigate).
            if matches!(key, KeyCode::Esc | KeyCode::Char('q')) {
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
        assert_eq!(app.home.intent, "call 1-800 and say q, then ~ and 4");
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
            app.home.intent, "x",
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
        app.home.intent = "fix the thing".to_string();
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
            app.home.intent, "fix the thing",
            "and the draft survives it"
        );
    }
}

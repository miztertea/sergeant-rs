//! Overlays (§7.4, Decision T2-24): a small fixed set of contextual views,
//! not a modal framework or second navigation system.
//!
//! The *mechanism* — how one opens, closes, and restores focus — is T1a's
//! job; T1c fills in the four real confirmations respond/retry/extend/
//! cancel need (§13.10/§15.5). T2 and T3 fill in the workflow chooser and
//! the repo/group/retained-state panels respectively. The T-series
//! follow-up closes the three gaps T1c's own scope note had named and
//! left open: the Workflows-screen half of the `@` chooser (§15.4, issue
//! #153), the connection detail panel (§7.4/§8.1, issue #154), and the
//! slash palette (§15.3's Decision T2-55, `Overlay::SlashPalette` —
//! [`SlashCommand`]'s fixed vocabulary as a Rust enum and match, not a
//! shell or plugin language, issue #152). Opening an overlay never mutates
//! the destination state underneath it — [`super::app::App::on_key`]
//! simply routes every keystroke to the overlay instead of the destination
//! while one is open — so closing it "restores focus" for free: nothing
//! under it ever moved.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::api::field_text;

use super::app::App;
use super::connection::Live;
use super::estate::{
    RepoOverlayMode, add_repo_body, group_form_body, remove_repo_body, retained_body,
};
use super::theme::Token;

/// §7.4's fixed set, in the order it lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    SlashPalette,
    WorkflowChooser,
    Help,
    CancelConfirmation,
    RetryConfirmation,
    ExtendEnvelope,
    ReapConfirmation,
    RepoAddRemove,
    GroupEditRemove,
    RetainedPreview,
    ConnectionDetail,
}

impl Overlay {
    fn title(self) -> &'static str {
        match self {
            Overlay::SlashPalette => "Command Palette",
            Overlay::WorkflowChooser => "Choose a Workflow",
            Overlay::Help => "Help",
            Overlay::CancelConfirmation => "Cancel Work?",
            Overlay::RetryConfirmation => "Retry?",
            Overlay::ExtendEnvelope => "Extend Envelope",
            Overlay::ReapConfirmation => "Reap?",
            Overlay::RepoAddRemove => "Repository",
            Overlay::GroupEditRemove => "Group",
            Overlay::RetainedPreview => "Retained State",
            Overlay::ConnectionDetail => "Connection",
        }
    }
}

/// §15.3's fixed vocabulary (Decision T2-55): a Rust enum and match, not a
/// shell or plugin language. [`SlashCommand::ALL`] is the palette's whole
/// content, in the order the proposal lists it — explicitly *not*
/// `/interrupt`, `/files`, `/web`, `/watch`, `/daemon`, `/init`, `/claude`,
/// `/codex`, `/opencode`, or `/goose`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlashCommand {
    Home,
    Fleet,
    Workflows,
    Estate,
    Back,
    Refresh,
    Answer,
    Retry,
    Extend,
    Cancel,
    Evidence,
    Graph,
    Details,
    AnalyticsPanel,
    Retained,
    Reap,
    Help,
    Quit,
}

impl SlashCommand {
    pub const ALL: [SlashCommand; 18] = [
        SlashCommand::Home,
        SlashCommand::Fleet,
        SlashCommand::Workflows,
        SlashCommand::Estate,
        SlashCommand::Back,
        SlashCommand::Refresh,
        SlashCommand::Answer,
        SlashCommand::Retry,
        SlashCommand::Extend,
        SlashCommand::Cancel,
        SlashCommand::Evidence,
        SlashCommand::Graph,
        SlashCommand::Details,
        SlashCommand::AnalyticsPanel,
        SlashCommand::Retained,
        SlashCommand::Reap,
        SlashCommand::Help,
        SlashCommand::Quit,
    ];

    /// The literal `/word` §15.3 lists for this command.
    pub fn word(self) -> &'static str {
        match self {
            SlashCommand::Home => "/home",
            SlashCommand::Fleet => "/fleet",
            SlashCommand::Workflows => "/workflows",
            SlashCommand::Estate => "/estate",
            SlashCommand::Back => "/back",
            SlashCommand::Refresh => "/refresh",
            SlashCommand::Answer => "/answer",
            SlashCommand::Retry => "/retry",
            SlashCommand::Extend => "/extend",
            SlashCommand::Cancel => "/cancel",
            SlashCommand::Evidence => "/evidence",
            SlashCommand::Graph => "/graph",
            SlashCommand::Details => "/details",
            SlashCommand::AnalyticsPanel => "/analytics",
            SlashCommand::Retained => "/retained",
            SlashCommand::Reap => "/reap",
            SlashCommand::Help => "/help",
            SlashCommand::Quit => "/quit",
        }
    }

    /// One line of what selecting this command does — the palette is
    /// self-documenting rather than requiring a second lookup in `?` help.
    fn description(self) -> &'static str {
        match self {
            SlashCommand::Home => "go to Home",
            SlashCommand::Fleet => "go to Fleet",
            SlashCommand::Workflows => "go to Workflows",
            SlashCommand::Estate => "go to Estate",
            SlashCommand::Back => "leave the open Work",
            SlashCommand::Refresh => "re-read the API now",
            SlashCommand::Answer => "focus ANSWER (where respond is offered)",
            SlashCommand::Retry => "open the retry confirmation (where offered)",
            SlashCommand::Extend => "open the extend confirmation (where offered)",
            SlashCommand::Cancel => "open the cancel confirmation (where offered)",
            SlashCommand::Evidence => "open Work: Evidence tab",
            SlashCommand::Graph => "open Work: Graph tab",
            SlashCommand::Details => "open Work: Details tab",
            SlashCommand::AnalyticsPanel => "not built in this Work yet",
            SlashCommand::Retained => "estate-wide retained-state preview",
            SlashCommand::Reap => "open the reap confirmation (where offered)",
            SlashCommand::Help => "this help",
            SlashCommand::Quit => "leave the TUI",
        }
    }
}

/// §15.6's fixed keymap reference, derived from the same key/action table
/// the footer uses.
const HELP_TEXT: &str = "\
Navigation
  1-4         Home / Fleet / Workflows / Estate
  Tab / S-Tab cycle destinations
  ~           toggle the Attention drawer
  /           slash command palette (INTENT/ANSWER, first non-whitespace)
  ?           this help
  c           connection detail (live/reconnecting/auth failed)
  q / Esc     back, or quit from a top-level destination

Fleet
  j/k, ↑/↓    move
  Enter       open the selected Work
  /           filter by text
  s           cycle the state filter
  a           toggle nonterminal-only
  v           toggle the selected-preview pane (Medium tier)
  x           clear filters

Home
  Tab / S-Tab move between fields
  Ctrl+Enter  submit from the INTENT composer directly
  Enter       newline in INTENT, else next field / submit from [ Run Work ]

Open Work
  Tab / S-Tab or 1-5  switch view
  j/k                 move
  r                    respond (when offered)
  c / t / e / p        cancel / retry / extend / reap (when offered) —
                        opens a confirmation; Enter there sends it
  Esc / q              back
";

/// Draw the overlay over `area`, clearing what was there first (§8.3's
/// `Clear`) so a contextual panel never shows through stale cells. `app`
/// supplies the live content T1c's four real confirmations need (the open
/// Work's own facts, and the one overlay's worth of in-progress state on
/// [`super::app::PendingAction`]) — every other overlay in the fixed set
/// ignores it.
pub fn render(frame: &mut Frame, area: Rect, overlay: Overlay, app: &App) {
    let panel = centered(area, 70, 60);
    frame.render_widget(Clear, panel);
    let block = Block::bordered()
        .title(overlay.title())
        .border_style(Style::default().fg(Token::Focus.rgb()));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let body = match overlay {
        Overlay::Help => HELP_TEXT.to_string(),
        Overlay::CancelConfirmation => cancel_body(app),
        Overlay::RetryConfirmation => retry_body(app),
        Overlay::ExtendEnvelope => extend_body(app),
        Overlay::ReapConfirmation => reap_body(app),
        Overlay::RepoAddRemove => match &app.estate.repo_form.mode {
            RepoOverlayMode::Add => {
                add_repo_body(&app.estate.repo_form, app.estate.pending_repo_add.as_ref())
            }
            RepoOverlayMode::ConfirmRemove { name } => {
                remove_repo_body(name, app.estate.repo_form.last_error.as_deref())
            }
        },
        Overlay::GroupEditRemove => group_form_body(&app.estate.group_form),
        Overlay::RetainedPreview => retained_body(&app.estate),
        Overlay::WorkflowChooser => workflow_chooser_body(app),
        Overlay::ConnectionDetail => connection_detail_body(app),
        Overlay::SlashPalette => slash_palette_body(app),
    };
    frame.render_widget(Paragraph::new(body).wrap(Wrap { trim: false }), inner);
}

/// The open Work's id and intent — every confirmation names the Work it
/// acts on (§15.5).
fn work_identity(app: &App) -> (String, String) {
    let id = app
        .open_work
        .as_ref()
        .map(|w| w.id.clone())
        .unwrap_or_else(|| "-".to_string());
    let intent = app
        .work_screen
        .as_ref()
        .map(|s| field_text(&s.work()["work"]["intent"]))
        .unwrap_or_else(|| "-".to_string());
    (id, intent)
}

/// §15.5: "Cancel: confirmation naming Work."
fn cancel_body(app: &App) -> String {
    let (id, intent) = work_identity(app);
    let mut body = format!(
        "Cancel this Work?\n\n  {intent}\n  {id}\n\nThis asks the daemon to cancel it. \
         It cannot be undone.\n\nEnter confirms · Esc/q aborts, nothing is sent."
    );
    if let Some(error) = &app.pending_action.last_error {
        body.push_str(&format!("\n\nlast attempt failed: {error}"));
    }
    body
}

/// §15.5: "Retry: confirmation naming stage/attempt."
fn retry_body(app: &App) -> String {
    let (id, intent) = work_identity(app);
    let (stage, attempt) = app
        .work_screen
        .as_ref()
        .map(|s| {
            let w = s.work();
            (
                field_text(&w["stage"]["stage_id"]),
                field_text(&w["stage"]["attempt"]),
            )
        })
        .unwrap_or_else(|| ("-".to_string(), "-".to_string()));
    let mut body = format!(
        "Retry this Work?\n\n  {intent}\n  {id}\n\n  stage    {stage}\n  attempt  {attempt}\n\n\
         Retries re-enter the current stage.\n\nEnter confirms · Esc/q aborts, nothing is sent."
    );
    if let Some(error) = &app.pending_action.last_error {
        body.push_str(&format!("\n\nlast attempt failed: {error}"));
    }
    body
}

/// §15.5: "Extend: explicit added turns and resulting cap."
fn extend_body(app: &App) -> String {
    let (id, intent) = work_identity(app);
    let current_cap = app
        .work_screen
        .as_ref()
        .and_then(|s| s.work()["envelope"]["turn_cap"].as_u64());
    let pending = &app.pending_action;
    let added: Option<u64> = pending.extend_turns.trim().parse().ok();
    let resulting = match (current_cap, added) {
        (Some(cap), Some(added)) if added > 0 => cap.saturating_add(added).to_string(),
        _ => "-".to_string(),
    };
    let field_marker = if pending.confirm_focused { "" } else { "_" };
    let confirm_marker = if pending.confirm_focused { " <" } else { "" };
    let mut body = format!(
        "Extend the turn envelope?\n\n  {intent}\n  {id}\n\n  \
         current cap    {}\n  add turns       {}{field_marker}\n  resulting cap   {resulting}\n\n\
         [ confirm ]{confirm_marker}\n\n\
         Tab moves between the turns field and Confirm · Enter on the field \
         moves to Confirm · Enter on Confirm sends it · Esc/q aborts.",
        current_cap
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string()),
        if pending.extend_turns.is_empty() {
            "-"
        } else {
            pending.extend_turns.as_str()
        },
    );
    if let Some(error) = &pending.last_error {
        body.push_str(&format!("\n\n{error}"));
    }
    body
}

/// The Work identity a reap confirmation names: an open Work surface's own
/// (§13.10), or — when reached from Estate's Retained preview instead
/// (§12.4, [`super::app::App::retained_reap_id`]) — the retained entry's own
/// id and repository, since there is no open Work surface to read from in
/// that path.
///
/// **Regression this Work's geometry matrix caught**: this used to call
/// [`work_identity`] unconditionally, which reads only `app.open_work`/
/// `app.work_screen` — both `None` on the Retained-preview path — so the
/// confirmation silently named "-  -" instead of the Work actually being
/// reaped, even though [`super::app::App::execute`]'s own `Action::Reap`
/// correctly acts on `retained_reap_id` either way.
fn reap_identity(app: &App) -> (String, String) {
    // `retained_reap_id` is the explicit selection made inside §12.4's
    // Retained preview, so it takes precedence over an open Work — the
    // two can now both be set at once since §15.3's `/retained` reaches
    // that preview without first leaving an open Work (see the matching
    // precedence in `App::on_key_overlay`'s `ReapConfirmation` arm).
    if app.retained_reap_id.is_none() && app.open_work.is_some() {
        return work_identity(app);
    }
    let id = app
        .retained_reap_id
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let repository = app
        .estate
        .retained
        .iter()
        .find(|entry| entry["work_id"].as_str() == app.retained_reap_id.as_deref())
        .and_then(|entry| entry["repository"].as_str())
        .unwrap_or("-")
        .to_string();
    (id, repository)
}

/// §15.5: "Reap: preview exact paths/bytes and state that retained branch
/// remains." The current Work read does not surface paths/bytes (only
/// `/v1/retained`, an estate-wide read T3's Estate surface owns, §20.4) —
/// per this Work's own scope note, a minimal confirmation stating the
/// branch is retained is what ships here rather than blocking on that data.
///
/// The accept/abort line comes right after the identity, ahead of the
/// longer explanatory paragraph: this Work's geometry matrix found the
/// original order pushed "Enter confirms · Esc/q aborts" off the bottom of
/// an 80x24 frame — §18.4's own rule ("nonessential summaries disappear
/// before primary state/question/actions") applies to this panel too.
fn reap_body(app: &App) -> String {
    let (id, description) = reap_identity(app);
    let mut body = format!(
        "Reap this Work's retained state?\n\n  {description}\n  {id}\n\n\
         Reap disposes local retained artifacts (worktrees/branches this daemon \
         still holds for review). The retained branch itself is not deleted — \
         only local disposal happens here.\n\n\
         Enter confirms · Esc/q aborts, nothing is sent.\n\n\
         Exact paths and bytes are not previewed here: that detail comes from \
         the estate-wide `/v1/retained` read, outside this Work's four Work-\
         scoped reads (T3's Estate surface, §20.4)."
    );
    if let Some(error) = &app.pending_action.last_error {
        body.push_str(&format!("\n\nlast attempt failed: {error}"));
    }
    body
}

/// §15.4/§11.4: the `@` chooser, opened from either Home's workflow field
/// or the Workflows screen itself (issue #153), over the live catalog
/// `App::refresh` already loaded — no second query, just the same
/// `app.workflows.entries` the Workflows destination itself reads, and one
/// piece of content shared by both trigger contexts.
fn workflow_chooser_body(app: &App) -> String {
    if app.workflows.entries.is_empty() {
        return "No workflows available (the catalog has not loaded, or is empty).\n\n\
                Esc/q closes."
            .to_string();
    }
    let mut lines = vec!["j/k move · Enter selects · Esc/q aborts\n".to_string()];
    for (i, entry) in app.workflows.entries.iter().enumerate() {
        let name = field_text(&entry["name"]);
        let marker = if i == app.workflow_chooser_index {
            "▶ "
        } else {
            "  "
        };
        lines.push(format!("{marker}{name}"));
    }
    lines.join("\n")
}

/// §7.4/§8.1, issue #154: a read-only view over the connection state
/// already tracked in [`super::connection`] for the header's own indicator
/// (`app.live`, [`Live::label`]) and the footer's own status line
/// (`app.status`) — no new state, just both put in one place with the
/// explanation the one-line header has no room for.
fn connection_detail_body(app: &App) -> String {
    let (state, detail) = match app.live {
        Live::Attached => (
            "live",
            "The SSE tail is attached; events are applied as they arrive.",
        ),
        Live::Reconnecting => (
            "reconnecting",
            "The tail dropped and the loop is retrying on its own capped \
             exponential backoff (issue #16) — recovery does not wait on a \
             keystroke, but this screen does not claim to be live again \
             until it actually is.",
        ),
        Live::AuthFailed => (
            "auth failed",
            "The daemon rejected this client's token. Automatic retries \
             stopped: a rejected token will not start working just \
             because this process asks again — restart sgt to pick up a \
             fresh one.",
        ),
    };
    let mut body = format!(
        "state    {state}\nheader   {}\n\n{detail}",
        app.live.label()
    );
    if !app.status.is_empty() {
        body.push_str(&format!("\n\nlast status\n  {}", app.status));
    }
    body.push_str("\n\nEsc/q closes this panel.");
    body
}

/// §15.3's fixed vocabulary, one line per [`SlashCommand`] in
/// [`SlashCommand::ALL`]'s order, with the highlighted entry marked exactly
/// like [`workflow_chooser_body`] above.
fn slash_palette_body(app: &App) -> String {
    let mut lines = vec!["j/k move · Enter selects · Esc/q aborts\n".to_string()];
    for (i, command) in SlashCommand::ALL.into_iter().enumerate() {
        let marker = if i == app.slash_palette_index {
            "▶ "
        } else {
            "  "
        };
        lines.push(format!(
            "{marker}{:<12}{}",
            command.word(),
            command.description()
        ));
    }
    lines.join("\n")
}

/// A `percent_x` × `percent_y` box, centered in `area`.
fn centered(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .areas(area);
    area
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// §15.3's exact vocabulary (Decision T2-55) — no more, no fewer, and
    /// never one of the names §15.3 explicitly excludes.
    #[test]
    fn the_slash_vocabulary_is_exactly_15_3s_fixed_list() {
        let words: Vec<&str> = SlashCommand::ALL.iter().map(|c| c.word()).collect();
        assert_eq!(
            words,
            vec![
                "/home",
                "/fleet",
                "/workflows",
                "/estate",
                "/back",
                "/refresh",
                "/answer",
                "/retry",
                "/extend",
                "/cancel",
                "/evidence",
                "/graph",
                "/details",
                "/analytics",
                "/retained",
                "/reap",
                "/help",
                "/quit",
            ]
        );
        for excluded in [
            "/interrupt",
            "/files",
            "/web",
            "/watch",
            "/daemon",
            "/init",
            "/claude",
            "/codex",
            "/opencode",
            "/goose",
        ] {
            assert!(
                !words.contains(&excluded),
                "{excluded} is explicitly excluded by §15.3"
            );
        }
    }

    /// The palette lists every command with the highlighted entry marked,
    /// same shape as [`workflow_chooser_lists_the_live_catalog_with_the_highlighted_entry_marked`].
    #[test]
    fn slash_palette_lists_every_command_with_the_highlighted_entry_marked() {
        let mut app = App::new();
        app.slash_palette_index = 3;
        let body = slash_palette_body(&app);
        for command in SlashCommand::ALL {
            assert!(body.contains(command.word()), "{body}");
        }
        assert!(body.contains("▶ /estate"), "{body}");
    }

    /// §15.4/§11.4: the chooser lists the live catalog by name, with the
    /// highlighted entry marked.
    #[test]
    fn workflow_chooser_lists_the_live_catalog_with_the_highlighted_entry_marked() {
        let mut app = App::new();
        app.workflows.set_entries(vec![
            json!({"name": "implement", "version": "2", "source": "/x", "content_hash": "a", "stages": []}),
            json!({"name": "diagnose-bug", "version": "3", "source": "/x", "content_hash": "b", "stages": []}),
        ]);
        app.workflow_chooser_index = 1;
        let body = workflow_chooser_body(&app);
        assert!(body.contains("implement"), "{body}");
        assert!(body.contains("▶ diagnose-bug"), "{body}");
    }

    #[test]
    fn workflow_chooser_with_no_catalog_loaded_says_so_rather_than_showing_an_empty_list() {
        let app = App::new();
        let body = workflow_chooser_body(&app);
        assert!(body.to_lowercase().contains("no workflows"), "{body}");
    }

    fn app_with_open_work(state: &str) -> App {
        let mut app = App::new();
        app.rows = super::super::fleet::fleet_rows(&json!({"works": [
            {"id": "01WORK", "state": state, "intent": "fix the thing"},
        ]}));
        app.open_work = Some(super::super::OpenWork {
            id: "01WORK".to_string(),
            from: super::super::Destination::Fleet,
        });
        app.work_screen = Some(super::super::work_view::WorkScreen::from_parts(
            "01WORK".to_string(),
            json!({
                "work": {"id": "01WORK", "intent": "fix the thing", "state": state},
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
    fn cancel_confirmation_names_the_work() {
        let app = app_with_open_work("blocked");
        let body = cancel_body(&app);
        assert!(body.contains("fix the thing"));
        assert!(body.contains("01WORK"));
    }

    #[test]
    fn retry_confirmation_names_stage_and_attempt() {
        let app = app_with_open_work("failed");
        let body = retry_body(&app);
        assert!(body.contains("10-implement"));
        assert!(body.contains("2"));
    }

    #[test]
    fn extend_confirmation_shows_the_resulting_cap_once_turns_are_entered() {
        let mut app = app_with_open_work("blocked");
        app.pending_action.extend_turns = "5".to_string();
        let body = extend_body(&app);
        assert!(body.contains("current cap"));
        assert!(body.contains("12"));
        assert!(
            body.contains("17"),
            "12 + 5 must be shown before confirming: {body}"
        );
    }

    #[test]
    fn extend_confirmation_shows_no_resulting_cap_until_turns_are_entered() {
        let app = app_with_open_work("blocked");
        let body = extend_body(&app);
        assert!(body.contains("resulting cap   -"), "{body}");
    }

    #[test]
    fn extend_confirmation_saturates_instead_of_overflowing_u64() {
        // Review fix: `cap + added` panics (debug) / wraps (release) once
        // the sum exceeds u64::MAX; `saturating_add` must clamp instead of
        // either.
        let mut app = App::new();
        app.rows = super::super::fleet::fleet_rows(&json!({"works": [
            {"id": "01WORK", "state": "blocked", "intent": "fix the thing"},
        ]}));
        app.open_work = Some(super::super::OpenWork {
            id: "01WORK".to_string(),
            from: super::super::Destination::Fleet,
        });
        app.work_screen = Some(super::super::work_view::WorkScreen::from_parts(
            "01WORK".to_string(),
            json!({
                "work": {"id": "01WORK", "intent": "fix the thing", "state": "blocked"},
                "stage": {"stage_id": "10-implement", "attempt": 2},
                "envelope": {"turn_cap": u64::MAX - 1, "turns_spawned": 3},
            }),
            Vec::new(),
            Vec::new(),
            None,
        ));
        app.pending_action.extend_turns = "5".to_string();
        let body = extend_body(&app);
        assert!(
            body.contains(&u64::MAX.to_string()),
            "the sum must saturate at u64::MAX, not panic or wrap: {body}"
        );
    }

    #[test]
    fn connection_detail_shows_live_when_attached() {
        let app = App::new();
        assert_eq!(app.live, Live::Attached, "a fresh app assumes the tail");
        let body = connection_detail_body(&app);
        assert!(body.contains("live"), "{body}");
        assert!(!body.to_lowercase().contains("reconnecting"), "{body}");
    }

    #[test]
    fn connection_detail_names_reconnecting_and_the_automatic_retry() {
        let mut app = App::new();
        app.live = Live::Reconnecting;
        let body = connection_detail_body(&app);
        assert!(body.contains("reconnecting"), "{body}");
        assert!(
            body.to_lowercase().contains("backoff"),
            "the automatic-retry explanation must be present: {body}"
        );
    }

    #[test]
    fn connection_detail_names_auth_failure_and_that_retries_stopped() {
        let mut app = App::new();
        app.live = Live::AuthFailed;
        let body = connection_detail_body(&app);
        assert!(body.contains("auth failed"), "{body}");
        assert!(
            body.to_lowercase().contains("stopped"),
            "the terminal-failure explanation must be present: {body}"
        );
    }

    #[test]
    fn connection_detail_includes_the_last_status_line_when_present() {
        let mut app = App::new();
        app.live = Live::Reconnecting;
        app.status = "live tail closed — reconnecting…".to_string();
        let body = connection_detail_body(&app);
        assert!(body.contains("live tail closed — reconnecting…"), "{body}");
    }

    #[test]
    fn connection_detail_omits_the_last_status_section_when_empty() {
        let mut app = App::new();
        app.status = String::new();
        let body = connection_detail_body(&app);
        assert!(!body.contains("last status"), "{body}");
    }

    #[test]
    fn reap_confirmation_states_the_branch_is_retained() {
        let app = app_with_open_work("completed_dirty");
        let body = reap_body(&app);
        assert!(
            body.to_lowercase().contains("not deleted") || body.to_lowercase().contains("remains")
        );
    }
}

//! Home (§9): the command center for current Work and new intent.
//!
//! §9's wide composition is three panes — Attention | New Work | Recent
//! Outputs — but the left "Attention" pane is not a Home-owned concept: it
//! is the global drawer from [`super::attention`], which §18.1 says Home may
//! show simultaneously with its own two panes. This module owns only the
//! two panes that are actually Home's: the New Work form (§9.1/§9.2,
//! including §15.1's `INTENT` composer — the deliberate multiline
//! [`super::composer::Composer`], not a single-line buffer) and Recent
//! Outputs (§9.4).

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, Padding, Paragraph, Wrap};
use serde_json::{Value, json};

use super::composer::{Composer, ComposerOutcome};
use super::fleet::{WorkRow, age_label};
use super::theme::{Token, spacing};

/// One field of the New Work form, in tab order. `Submit` is the trailing
/// `[ Run Work ]` control (§9's ASCII layout), not a text field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Intent,
    Workflow,
    Backend,
    Profile,
    Workspace,
    Repositories,
    TurnCap,
    CeilingSecs,
    Submit,
}

const FIELDS: [Field; 9] = [
    Field::Intent,
    Field::Workflow,
    Field::Backend,
    Field::Profile,
    Field::Workspace,
    Field::Repositories,
    Field::TurnCap,
    Field::CeilingSecs,
    Field::Submit,
];

impl Field {
    fn label(self) -> &'static str {
        match self {
            Field::Intent => "intent",
            Field::Workflow => "workflow",
            Field::Backend => "backend",
            Field::Profile => "profile",
            Field::Workspace => "workspace",
            Field::Repositories => "repositories",
            Field::TurnCap => "turns",
            Field::CeilingSecs => "ceiling",
            Field::Submit => "",
        }
    }
}

/// What a form keystroke asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeOutcome {
    None,
    /// The `POST /v1/work` body, ready for `ApiClient::post`.
    Submit(Value),
    /// §15.4's Home half: "`@` selects the existing workflow request field
    /// and renders a chip outside durable intent" — pressed on the workflow
    /// field, this opens the live-catalog chooser (`Overlay::WorkflowChooser`,
    /// App-level state) instead of typing a literal `@`.
    OpenWorkflowChooser,
}

/// §9.1's New Work fields. §15.1's `INTENT` context: the intent field is
/// [`Composer`], the deliberate multiline composer (§8.7/§9.2/§15.2) — every
/// other field stays the plain single-line buffer it always was.
#[derive(Debug)]
pub struct NewWorkForm {
    focus: usize,
    /// Whether Esc has left the form (§9.2: "Esc leave focus and preserve
    /// draft"). While blurred the form does not claim keyboard focus — see
    /// [`NewWorkForm::wants_text_focus`] — so global destination-nav keys
    /// work again; entering Home afresh re-focuses it (§9.2's "the intent
    /// editor is the primary focus").
    blurred: bool,
    pub intent: Composer,
    pub workflow: String,
    pub backend: String,
    pub profile: String,
    pub workspace: String,
    /// Free text, split on whitespace/commas at submission — §9.1 names a
    /// group-expansion path shared with `sgt run --group`, but that path
    /// reads the estate manifest from local disk
    /// (`Workspace::declared_groups_scoped`), which no TUI module may do:
    /// the TUI reaches the crate only through `crate::api` (§30, `t5`/`t5b`),
    /// and the daemon does not expose declared repositories/groups over the
    /// API yet — that is T3's job (§20.4, the `/v1/estate/*` routes). Until
    /// then this field takes explicit repository names, exactly what
    /// `sgt run --repo` already accepts without `--group`.
    pub repositories: String,
    pub turn_cap: String,
    pub ceiling_secs: String,
    /// Set by a failed validation or a rejected submission; §17.5's rule
    /// that structured errors stay visible beside the action that caused
    /// them, and §9.2's "the draft clears only after the daemon accepts".
    pub last_error: Option<String>,
}

impl NewWorkForm {
    /// Whether the form currently owns the keyboard. `App` consults this
    /// before routing a keystroke to a global destination-nav key, so typing
    /// an intent that happens to contain `1` or `q` cannot be hijacked.
    pub fn wants_text_focus(&self) -> bool {
        !self.blurred
    }

    /// Whether the intent composer specifically has focus — the footer uses
    /// this to show §15.2's composer grammar rather than the plain-field one.
    pub fn intent_focused(&self) -> bool {
        !self.blurred && FIELDS[self.focus] == Field::Intent
    }

    /// Bring the intent field back into focus (§9.2: "the intent editor is
    /// the primary focus" — entering Home, fresh or returning, always
    /// re-focuses it rather than leaving the form blurred from last time).
    pub fn refocus(&mut self) {
        self.blurred = false;
        self.focus = 0;
    }

    pub fn on_key(&mut self, key: impl Into<KeyEvent>) -> HomeOutcome {
        let key: KeyEvent = key.into();
        if FIELDS[self.focus] == Field::Intent {
            match key.code {
                KeyCode::Esc => {
                    self.blurred = true;
                    return HomeOutcome::None;
                }
                KeyCode::Tab => {
                    self.focus = (self.focus + 1) % FIELDS.len();
                    return HomeOutcome::None;
                }
                KeyCode::BackTab => {
                    self.focus = (self.focus + FIELDS.len() - 1) % FIELDS.len();
                    return HomeOutcome::None;
                }
                _ => {}
            }
            // §9.2: Ctrl+Enter submits the whole form (not merely "this
            // field is nonblank") — `try_submit` re-validates every field,
            // exactly as reaching `[ Run Work ]` by Tab and Enter would.
            return match self.intent.on_key(key) {
                ComposerOutcome::Submit(_) | ComposerOutcome::Refused => self.try_submit(),
                ComposerOutcome::None => HomeOutcome::None,
            };
        }
        match key.code {
            KeyCode::Esc => self.blurred = true,
            KeyCode::Tab => self.focus = (self.focus + 1) % FIELDS.len(),
            KeyCode::BackTab => self.focus = (self.focus + FIELDS.len() - 1) % FIELDS.len(),
            KeyCode::Enter => {
                if FIELDS[self.focus] == Field::Submit {
                    return self.try_submit();
                }
                self.focus = (self.focus + 1) % FIELDS.len();
            }
            KeyCode::Backspace => {
                if let Some(field) = self.field_mut() {
                    field.pop();
                }
            }
            // §15.4: `@` on the workflow field opens the live-catalog
            // chooser rather than being typed literally — checked before the
            // generic `Char(c)` arm below, which still handles `@` typed
            // into any other field exactly as before.
            KeyCode::Char('@') if FIELDS[self.focus] == Field::Workflow => {
                return HomeOutcome::OpenWorkflowChooser;
            }
            KeyCode::Char(c) => {
                if let Some(field) = self.field_mut() {
                    field.push(c);
                }
            }
            _ => {}
        }
        HomeOutcome::None
    }

    fn field_mut(&mut self) -> Option<&mut String> {
        match FIELDS[self.focus] {
            Field::Intent => None,
            Field::Workflow => Some(&mut self.workflow),
            Field::Backend => Some(&mut self.backend),
            Field::Profile => Some(&mut self.profile),
            Field::Workspace => Some(&mut self.workspace),
            Field::Repositories => Some(&mut self.repositories),
            Field::TurnCap => Some(&mut self.turn_cap),
            Field::CeilingSecs => Some(&mut self.ceiling_secs),
            Field::Submit => None,
        }
    }

    fn try_submit(&mut self) -> HomeOutcome {
        if self.intent.is_empty() {
            self.last_error = Some("intent is required".to_string());
            return HomeOutcome::None;
        }
        for (label, value) in [("turns", &self.turn_cap), ("ceiling", &self.ceiling_secs)] {
            if !value.trim().is_empty() && value.trim().parse::<u64>().is_err() {
                self.last_error = Some(format!("{label} must be a whole number"));
                return HomeOutcome::None;
            }
        }
        self.last_error = None;
        HomeOutcome::Submit(self.body())
    }

    /// §9.1's field mapping onto current submission semantics — the same
    /// body shape `sgt run` already sends to `POST /v1/work`.
    fn body(&self) -> Value {
        let repositories: Vec<String> = self
            .repositories
            .split([',', ' '])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        let envelope = if !self.turn_cap.trim().is_empty() || !self.ceiling_secs.trim().is_empty() {
            Some(json!({
                "turn_cap": non_empty_u64(&self.turn_cap),
                "ceiling_secs": non_empty_u64(&self.ceiling_secs),
            }))
        } else {
            None
        };
        let intent = self.intent.text();
        json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "intent": intent.trim(),
            "workflow": non_empty(&self.workflow),
            "backend": non_empty(&self.backend),
            "profile": non_empty(&self.profile),
            "repositories": repositories,
            "workspace": non_empty(&self.workspace),
            "envelope": envelope,
            "created_by": "tui",
            "origin": {"client": "tui", "cwd": std::env::current_dir().ok()},
        })
    }

    /// The draft clears only after the daemon accepts the Work (§9.2).
    pub fn clear_draft(&mut self) {
        *self = NewWorkForm {
            focus: self.focus,
            ..NewWorkForm::default()
        };
    }
}

impl Default for NewWorkForm {
    fn default() -> Self {
        Self {
            focus: 0,
            blurred: false,
            intent: Composer::new().with_placeholder(
                "Describe the intent — Ctrl+Enter to submit, Enter for a newline",
            ),
            workflow: String::new(),
            backend: String::new(),
            profile: String::new(),
            workspace: String::new(),
            repositories: String::new(),
            turn_cap: String::new(),
            ceiling_secs: String::new(),
            last_error: None,
        }
    }
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn non_empty_u64(value: &str) -> Option<u64> {
    non_empty(value).and_then(|v| v.parse().ok())
}

// -------------------------------------------------------------- rendering

/// Draw the New Work form: §15.1's `INTENT` composer as its own bounded
/// sub-block (§18.4: "composer gets a bounded minimum"), then the remaining
/// single-line fields below it.
pub fn render(frame: &mut Frame, area: Rect, form: &NewWorkForm, admission_paused: bool) {
    let block = Block::bordered()
        .title("Home / New Work")
        .border_style(Style::default().fg(Token::Border.rgb()))
        // §8.10's spacing scale: `block` (2 cells) is a focusable Block's
        // interior left/right padding.
        .padding(Padding::horizontal(spacing::BLOCK));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let intent_focused = form.intent_focused();
    let composer_height = inner.height.saturating_sub(10).clamp(3, 6);
    let [composer_area, fields_area] =
        Layout::vertical([Constraint::Length(composer_height), Constraint::Min(1)]).areas(inner);
    render_intent_composer(frame, composer_area, form, intent_focused);

    let mut lines: Vec<Line> = vec![Line::default()];
    for field in FIELDS {
        if field == Field::Intent {
            continue;
        }
        let focused = FIELDS[form.focus] == field;
        lines.push(field_line(form, field, focused));
    }
    lines.push(Line::default());
    if admission_paused {
        lines.push(Line::from(Span::styled(
            "admission is paused — new work is refused while the daemon drains",
            Style::default().fg(Token::Warning.rgb()),
        )));
    }
    if let Some(error) = &form.last_error {
        lines.push(Line::from(Span::styled(
            error.clone(),
            Style::default().fg(Token::Danger.rgb()),
        )));
    }
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        fields_area,
    );
}

/// §15.1's persistent composer, this destination's context: `INTENT`,
/// submit new Work.
fn render_intent_composer(frame: &mut Frame, area: Rect, form: &NewWorkForm, focused: bool) {
    let title = if focused {
        "INTENT — Ctrl+Enter submit · Enter newline · Tab next field · Esc leave"
    } else {
        "INTENT"
    };
    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(if focused {
            Token::Focus.rgb()
        } else {
            Token::Border.rgb()
        }));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    form.intent.render(frame, inner);
}

fn field_line<'a>(form: &'a NewWorkForm, field: Field, focused: bool) -> Line<'a> {
    let base = if focused {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };
    if field == Field::Submit {
        return Line::from(Span::styled(
            "        [ Run Work ]",
            base.add_modifier(Modifier::BOLD),
        ));
    }
    let value = match field {
        Field::Intent => "",
        Field::Workflow => form.workflow.as_str(),
        Field::Backend => form.backend.as_str(),
        Field::Profile => form.profile.as_str(),
        Field::Workspace => form.workspace.as_str(),
        Field::Repositories => form.repositories.as_str(),
        Field::TurnCap => form.turn_cap.as_str(),
        Field::CeilingSecs => form.ceiling_secs.as_str(),
        Field::Submit => "",
    };
    let label_span = Span::styled(
        format!("{:<13}", field.label()),
        Style::default().fg(Token::Muted.rgb()),
    );
    // §15.4: a workflow chosen from the live catalog renders as a chip
    // (`[name]`) outside durable intent — visually distinct from the plain
    // text every other field is, since it is a selection, not free text.
    if field == Field::Workflow && !value.is_empty() {
        let cursor = if focused { "_" } else { "" };
        return Line::from(vec![
            label_span,
            Span::styled(
                format!("[{value}]"),
                base.fg(Token::Focus.rgb()).add_modifier(Modifier::BOLD),
            ),
            Span::raw(cursor),
        ]);
    }
    let cursor = if focused { "_" } else { "" };
    Line::from(vec![
        label_span,
        Span::styled(format!("{value}{cursor}"), base),
    ])
}

/// §9.4's Recent Outputs pane.
///
/// The full output pointer (retained branch, finalize commit) lives on each
/// Work's own detail body (`GET /v1/work/{id}`'s `output` key), not on the
/// Fleet projection this screen already has loaded — reading it per row here
/// would mean an unbounded fan-out of extra requests every Home render. T1b
/// owns that per-Work detail (§13.8's Output tab) over the same already-
/// shipped read. Until then this pane honors §9.4's "never imply more than
/// the API supplies": it lists terminal Work by intent and disposition alone,
/// using only fields the Fleet projection already carries, and claims
/// nothing about retained branches or finalize commits it has not read.
pub fn render_recent_outputs(frame: &mut Frame, area: Rect, rows: &[WorkRow]) {
    let block = Block::bordered()
        .title("Recent Outputs")
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let terminal: Vec<&WorkRow> = rows
        .iter()
        .filter(|row| matches!(row.state.as_str(), "completed" | "completed_dirty"))
        .collect();
    if terminal.is_empty() {
        frame.render_widget(Paragraph::new("No terminal Work yet."), inner);
        return;
    }
    let items: Vec<ListItem> = terminal
        .iter()
        .map(|row| {
            let disposition = if row.state == "completed_dirty" {
                Span::styled(
                    "output needs review",
                    Style::default().fg(Token::Attention.rgb()),
                )
            } else {
                Span::styled("completed", Style::default().fg(Token::Success.rgb()))
            };
            ListItem::new(Line::from(vec![
                Span::raw(row.intent.clone()),
                Span::raw("  "),
                disposition,
                Span::styled(
                    format!("  {}", age_label(&row.created_at)),
                    Style::default().fg(Token::Muted.rgb()),
                ),
            ]))
        })
        .collect();
    frame.render_widget(List::new(items), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn ctrl_enter() -> KeyEvent {
        KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)
    }

    #[test]
    fn tab_cycles_focus_and_typing_edits_the_focused_field() {
        let mut form = NewWorkForm::default();
        assert_eq!(form.on_key(KeyCode::Char('h')), HomeOutcome::None);
        assert_eq!(form.intent.text(), "h", "intent is focused first");
        form.on_key(KeyCode::Tab);
        form.on_key(KeyCode::Char('w'));
        assert_eq!(form.workflow, "w");
        assert_eq!(form.intent.text(), "h", "the previous field keeps its text");
    }

    #[test]
    fn backspace_edits_only_the_focused_field() {
        let mut form = NewWorkForm::default();
        form.on_key(KeyCode::Char('a'));
        form.on_key(KeyCode::Char('b'));
        form.on_key(KeyCode::Backspace);
        assert_eq!(form.intent.text(), "a");
    }

    #[test]
    fn enter_in_the_intent_field_is_a_newline_not_field_advance() {
        let mut form = NewWorkForm::default();
        form.on_key(KeyCode::Char('a'));
        form.on_key(KeyCode::Enter);
        form.on_key(KeyCode::Char('b'));
        assert_eq!(form.intent.text(), "a\nb");
        assert!(
            form.intent_focused(),
            "Enter must not have left the intent field"
        );
    }

    #[test]
    fn ctrl_enter_in_the_intent_field_submits_the_whole_form() {
        let mut form = NewWorkForm {
            workflow: "implement".to_string(),
            ..NewWorkForm::default()
        };
        form.intent.set_text("fix the thing");
        let outcome = form.on_key(ctrl_enter());
        let HomeOutcome::Submit(body) = outcome else {
            panic!("expected a submission, got {outcome:?}");
        };
        assert_eq!(body["intent"], "fix the thing");
        assert_eq!(body["workflow"], "implement");
    }

    #[test]
    fn ctrl_enter_on_a_blank_intent_reports_the_same_error_run_work_would() {
        let mut form = NewWorkForm::default();
        assert_eq!(form.on_key(ctrl_enter()), HomeOutcome::None);
        assert_eq!(form.last_error.as_deref(), Some("intent is required"));
    }

    #[test]
    fn tab_leaves_the_intent_composer_for_the_next_field() {
        let mut form = NewWorkForm::default();
        form.on_key(KeyCode::Char('h'));
        form.on_key(KeyCode::Tab);
        assert!(!form.intent_focused());
        form.on_key(KeyCode::Char('w'));
        assert_eq!(form.workflow, "w");
        assert_eq!(
            form.intent.text(),
            "h",
            "leaving the composer must not have inserted a tab character"
        );
    }

    #[test]
    fn submit_requires_a_nonempty_intent() {
        let mut form = NewWorkForm::default();
        for _ in 0..(FIELDS.len() - 1) {
            form.on_key(KeyCode::Tab);
        }
        assert_eq!(form.on_key(KeyCode::Enter), HomeOutcome::None);
        assert_eq!(form.last_error.as_deref(), Some("intent is required"));
    }

    #[test]
    fn a_complete_form_submits_the_v1_work_body_sgt_run_would_send() {
        let mut form = NewWorkForm {
            workflow: "implement".to_string(),
            repositories: "svc-a, svc-b".to_string(),
            turn_cap: "12".to_string(),
            ..NewWorkForm::default()
        };
        form.intent.set_text("fix the thing");
        for _ in 0..(FIELDS.len() - 1) {
            form.on_key(KeyCode::Tab);
        }
        let outcome = form.on_key(KeyCode::Enter);
        let HomeOutcome::Submit(body) = outcome else {
            panic!("expected a submission, got {outcome:?}");
        };
        assert_eq!(body["intent"], "fix the thing");
        assert_eq!(body["workflow"], "implement");
        assert_eq!(body["repositories"], json!(["svc-a", "svc-b"]));
        assert_eq!(body["envelope"]["turn_cap"], 12);
        assert_eq!(body["created_by"], "tui");
        assert_eq!(body["origin"]["client"], "tui");
        assert!(form.last_error.is_none());
    }

    #[test]
    fn a_non_numeric_turn_cap_is_refused_without_losing_the_draft() {
        let mut form = NewWorkForm {
            turn_cap: "not a number".to_string(),
            ..NewWorkForm::default()
        };
        form.intent.set_text("keep me");
        for _ in 0..(FIELDS.len() - 1) {
            form.on_key(KeyCode::Tab);
        }
        assert_eq!(form.on_key(KeyCode::Enter), HomeOutcome::None);
        assert_eq!(
            form.last_error.as_deref(),
            Some("turns must be a whole number")
        );
        assert_eq!(
            form.intent.text(),
            "keep me",
            "the draft survives a validation error"
        );
    }

    /// §15.4: `@` on the workflow field asks `App` to open the chooser
    /// rather than typing a literal `@` into the field.
    #[test]
    fn at_sign_on_the_workflow_field_asks_to_open_the_chooser() {
        let mut form = NewWorkForm::default();
        form.on_key(KeyCode::Tab); // intent -> workflow
        assert_eq!(
            form.on_key(KeyCode::Char('@')),
            HomeOutcome::OpenWorkflowChooser
        );
        assert_eq!(form.workflow, "", "no literal '@' was typed into the field");
    }

    /// `@` typed into any other field is still ordinary text — only the
    /// workflow field intercepts it.
    #[test]
    fn at_sign_on_another_field_is_ordinary_text() {
        let mut form = NewWorkForm::default();
        for _ in 0..2 {
            form.on_key(KeyCode::Tab); // intent -> workflow -> backend
        }
        assert_eq!(form.on_key(KeyCode::Char('@')), HomeOutcome::None);
        assert_eq!(form.backend, "@");
    }

    #[test]
    fn a_chosen_workflow_renders_as_a_bracketed_chip() {
        let form = NewWorkForm {
            workflow: "implement".to_string(),
            ..NewWorkForm::default()
        };
        let line = field_line(&form, Field::Workflow, false);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("[implement]"), "{text}");
    }

    #[test]
    fn clear_draft_resets_every_field_but_keeps_the_current_focus() {
        let mut form = NewWorkForm::default();
        form.intent.set_text("will be cleared");
        form.on_key(KeyCode::Tab);
        let focus_before = form.focus;
        form.clear_draft();
        assert_eq!(form.intent.text(), "");
        assert_eq!(form.focus, focus_before);
    }

    #[test]
    fn recent_outputs_shows_only_terminal_work_and_flags_completed_dirty() {
        let rows = super::super::fleet::fleet_rows(&json!({"works": [
            {"id": "a", "state": "completed", "intent": "done"},
            {"id": "b", "state": "completed_dirty", "intent": "messy"},
            {"id": "c", "state": "running", "intent": "still going"},
        ]}));
        let terminal: Vec<&str> = rows
            .iter()
            .filter(|r| matches!(r.state.as_str(), "completed" | "completed_dirty"))
            .map(|r| r.intent.as_str())
            .collect();
        assert_eq!(terminal, vec!["done", "messy"]);
    }
}

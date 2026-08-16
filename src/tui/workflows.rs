//! Workflows (§11): catalog list | workflow detail | recent-Work-using-this-
//! workflow, over `GET /v1/workflows` (§11.2's T0-finalized schema).
//!
//! `recent-Work-using-this-workflow` is derived from `App::rows` — the
//! already-loaded Fleet projection — not a second query (§11.3's own note);
//! this module only ever reads the `workflow` field an already-fetched
//! [`super::fleet::WorkRow`] carries.
//!
//! §6.2 excludes a generalized backend capability matrix: the detail pane
//! shows exactly what §11.3 lists (name/version/source/status/description/
//! tags/stage order/stage kind/declared harness/profile/content identity/
//! recent Work) and nothing a current public surface does not supply.

use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, Padding, Paragraph, Wrap};
use serde_json::Value;

use crate::api::field_text;

use super::fleet::{WorkRow, age_label};
use super::theme::{self, Token, spacing};

/// What a Workflows keystroke asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowsOutcome {
    None,
    /// §11.4's "Use in New Work": hand the selected workflow's name to
    /// Home's submission form. `App` owns what that means (setting the
    /// field and switching destinations) — this screen only names the
    /// workflow.
    UseInNewWork(String),
    /// §15.4/T2-56's Workflows half (issue #153): open the same live-catalog
    /// chooser Home's workflow field already opens (`Overlay::
    /// WorkflowChooser`, `App`-level state) — this screen only asks for it,
    /// the same way `HomeOutcome::OpenWorkflowChooser` does.
    OpenWorkflowChooser,
}

/// Workflows' own screen state: the catalog `App::refresh` last read, local
/// selection, and the `/` filter (mirrors Fleet's own local filter, §10.3).
/// `@` is reserved for §15.4's live-catalog chooser
/// (`WorkflowsOutcome::OpenWorkflowChooser`) rather than this local filter,
/// matching Home's own `@` binding on its workflow field.
#[derive(Debug, Default)]
pub struct WorkflowsScreen {
    /// `GET /v1/workflows`'s `workflows[]`, verbatim (§11.2's CatalogEntry
    /// shape) — this screen never reinterprets or caches a second shape of
    /// the same data.
    pub entries: Vec<Value>,
    /// The last fetch's failure, if any — kept visible rather than silently
    /// showing a stale or empty catalog.
    pub last_error: Option<String>,
    selected: usize,
    filter: String,
    filter_focus: bool,
}

impl WorkflowsScreen {
    /// Entries the current `/` filter admits: a case-insensitive substring
    /// match on name, description, or any tag (mirrors Fleet's own local
    /// filter, §10.3).
    pub fn visible(&self) -> Vec<&Value> {
        if self.filter.is_empty() {
            return self.entries.iter().collect();
        }
        let needle = self.filter.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| entry_matches(entry, &needle))
            .collect()
    }

    pub fn selected_entry(&self) -> Option<&Value> {
        self.visible().into_iter().nth(self.selected)
    }

    /// Move the local selection onto the catalog entry named `name` — how
    /// `Overlay::WorkflowChooser`'s Enter (opened via
    /// `WorkflowsOutcome::OpenWorkflowChooser`) hands its pick back to this
    /// screen, mirroring how the same overlay hands its pick to Home's
    /// workflow field. The chooser lists the full, unfiltered catalog
    /// (§15.4), so `name` may not pass this screen's own local `/` filter;
    /// in that case the filter is cleared so `name` becomes selectable.
    pub fn select_by_name(&mut self, name: &str) {
        if let Some(index) = self
            .visible()
            .iter()
            .position(|e| e["name"].as_str() == Some(name))
        {
            self.selected = index;
            return;
        }
        if !self
            .entries
            .iter()
            .any(|e| e["name"].as_str() == Some(name))
        {
            return;
        }
        self.filter.clear();
        if let Some(index) = self
            .visible()
            .iter()
            .position(|e| e["name"].as_str() == Some(name))
        {
            self.selected = index;
        }
    }

    /// Whether the `/` filter field currently owns the keyboard — `App`
    /// consults this before routing a keystroke to a global destination-nav
    /// key, exactly as Home and Fleet's own text fields already do.
    pub fn wants_text_focus(&self) -> bool {
        self.filter_focus
    }

    fn clamp(&mut self) {
        let n = self.visible().len();
        if self.selected >= n {
            self.selected = n.saturating_sub(1);
        }
    }

    /// Replace the loaded catalog (`App::refresh`), keeping the same
    /// workflow selected by name where it still exists rather than always
    /// resetting to the top of the list on every refresh.
    pub fn set_entries(&mut self, entries: Vec<Value>) {
        let selected_name = self
            .selected_entry()
            .and_then(|e| e["name"].as_str())
            .map(str::to_string);
        self.entries = entries;
        if let Some(name) = selected_name
            && let Some(index) = self
                .visible()
                .iter()
                .position(|e| e["name"].as_str() == Some(name.as_str()))
        {
            self.selected = index;
        }
        self.clamp();
    }

    /// Interpret one keystroke. Enter is deliberately not bound to anything
    /// separate from selection: the detail pane already tracks the selected
    /// entry live (§11.3's three panes are shown together, not opened), so
    /// "Enter inspect" is what moving the selection already does.
    pub fn on_key(&mut self, key: KeyCode) -> WorkflowsOutcome {
        if self.filter_focus {
            match key {
                KeyCode::Esc | KeyCode::Enter => self.filter_focus = false,
                KeyCode::Backspace => {
                    self.filter.pop();
                }
                KeyCode::Char(c) => self.filter.push(c),
                _ => {}
            }
            self.clamp();
            return WorkflowsOutcome::None;
        }
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                let n = self.visible().len();
                if self.selected + 1 < n {
                    self.selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('/') => self.filter_focus = true,
            KeyCode::Char('@') => return WorkflowsOutcome::OpenWorkflowChooser,
            KeyCode::Char('x') if !self.filter.is_empty() => {
                self.filter.clear();
                self.clamp();
            }
            KeyCode::Char('u') => {
                if let Some(name) = self.selected_entry().and_then(|e| e["name"].as_str()) {
                    return WorkflowsOutcome::UseInNewWork(name.to_string());
                }
            }
            _ => {}
        }
        WorkflowsOutcome::None
    }
}

fn entry_matches(entry: &Value, needle: &str) -> bool {
    let name = entry["name"].as_str().unwrap_or("").to_lowercase();
    if name.contains(needle) {
        return true;
    }
    let description = entry["description"].as_str().unwrap_or("").to_lowercase();
    if description.contains(needle) {
        return true;
    }
    entry["tags"].as_array().is_some_and(|tags| {
        tags.iter()
            .any(|t| t.as_str().unwrap_or("").to_lowercase().contains(needle))
    })
}

// -------------------------------------------------------------- rendering

/// The primary body: catalog list (left) | workflow detail (right) — §11.3's
/// third pane, recent Work, is drawn separately as the Wide contextual rail
/// (`render_recent_work`, wired the same way Home's and Fleet's own rail
/// content is, `mod.rs`'s `draw_wide`).
pub fn render(frame: &mut Frame, area: Rect, screen: &WorkflowsScreen) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Length(36), Constraint::Min(1)]).areas(area);
    render_list(frame, list_area, screen);
    render_detail(frame, detail_area, screen);
}

fn render_list(frame: &mut Frame, area: Rect, screen: &WorkflowsScreen) {
    let visible = screen.visible();
    let title = format!(
        "Workflows — {} workflow{}{}",
        visible.len(),
        if visible.len() == 1 { "" } else { "s" },
        if screen.filter.is_empty() {
            ""
        } else {
            " (filtered)"
        }
    );
    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [filter_area, list_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(inner);
    let filter_text = if screen.filter_focus {
        format!("/{}_", screen.filter)
    } else {
        format!(
            "filter: {}  ('/' filter · '@' chooser · 'u' use in new work)",
            if screen.filter.is_empty() {
                "-"
            } else {
                &screen.filter
            }
        )
    };
    frame.render_widget(
        Paragraph::new(filter_text).style(Style::default().fg(Token::Muted.rgb())),
        filter_area,
    );

    if let Some(error) = &screen.last_error {
        frame.render_widget(
            Paragraph::new(format!("could not load workflows: {error}"))
                .style(Style::default().fg(Token::Danger.rgb()))
                .wrap(Wrap { trim: false }),
            list_area,
        );
        return;
    }
    if visible.is_empty() {
        let msg = if screen.entries.is_empty() {
            "No workflows yet."
        } else {
            "No workflows match the current filter — 'x' clears it."
        };
        frame.render_widget(Paragraph::new(msg), list_area);
        return;
    }
    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let name = field_text(&entry["name"]);
            let embedded_note = if entry["source"].as_str() == Some("embedded") {
                " (embedded)"
            } else {
                ""
            };
            let style = if i == screen.selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!("{name}{embedded_note}"),
                style,
            )))
        })
        .collect();
    frame.render_widget(List::new(items), list_area);
}

fn render_detail(frame: &mut Frame, area: Rect, screen: &WorkflowsScreen) {
    let block = Block::bordered()
        .title("Detail")
        .border_style(Style::default().fg(Token::Border.rgb()))
        .padding(Padding::horizontal(spacing::BLOCK));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(entry) = screen.selected_entry() else {
        frame.render_widget(Paragraph::new("Select a workflow to inspect it."), inner);
        return;
    };
    frame.render_widget(
        Paragraph::new(detail_lines(entry)).wrap(Wrap { trim: false }),
        inner,
    );
}

fn kv(key: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{key:<14}"),
            Style::default().fg(Token::Muted.rgb()),
        ),
        Span::raw(value.to_string()),
    ])
}

/// §11.3's detail fields, in the order it lists them: name/version/source/
/// status, description/tags, stage order, stage kind, declared
/// harness/profile, content identity. Recent Work is not here — it is the
/// Wide rail (`render_recent_work`), since it is derived from a different
/// source (Fleet rows) than everything else on this pane (the catalog entry
/// itself).
fn detail_lines(entry: &Value) -> Vec<Line<'static>> {
    let name = field_text(&entry["name"]);
    let version = field_text(&entry["version"]);
    let embedded = entry["source"].as_str() == Some("embedded");

    let mut lines = vec![
        Line::from(Span::styled(
            format!("{name} v{version}"),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        // §11.5's Decision T2-41: this list is the live catalog — what new
        // Work can bind *now* — labeled as such so it is never confused
        // with a canonical Work's own pinned Workflow tab.
        Line::from(Span::styled(
            if embedded {
                "embedded fallback — no repository catalog resolved for this cwd"
            } else {
                "live catalog — what new Work can bind now"
            },
            Style::default().fg(Token::Muted.rgb()),
        )),
        Line::default(),
        kv("source", &field_text(&entry["source"])),
        kv("status", &field_text(&entry["status"])),
        kv("content hash", &field_text(&entry["content_hash"])),
        Line::default(),
    ];

    let description = field_text(&entry["description"]);
    if description != "-" {
        lines.push(Line::from(Span::styled(
            "description",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(format!("  {description}")));
        lines.push(Line::default());
    }

    if let Some(tags) = entry["tags"].as_array() {
        let joined = tags
            .iter()
            .filter_map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(kv("tags", &joined));
        lines.push(Line::default());
    }

    lines.push(Line::from(Span::styled(
        "stages",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    if let Some(stages) = entry["stages"].as_array() {
        for (i, stage) in stages.iter().enumerate() {
            let id = field_text(&stage["id"]);
            let kind = field_text(&stage["kind"]);
            let mut suffix = kind;
            if let Some(harness) = stage["harness"].as_str() {
                suffix.push_str(&format!(" · harness {harness}"));
            }
            if let Some(profile) = stage["profile"].as_str() {
                suffix.push_str(&format!(" · profile {profile}"));
            }
            if stage["requires_ask"].as_bool().unwrap_or(false) {
                suffix.push_str(" · asks");
            }
            lines.push(Line::from(format!("  {}. {id}  ({suffix})", i + 1)));
        }
    }
    lines
}

/// §11.3's third pane, drawn as the Wide contextual rail (`mod.rs`'s
/// `draw_wide`, the same mechanism Home's Recent Outputs and Fleet's
/// selected-Work preview already use): every Fleet row whose `workflow`
/// field names the selected catalog entry — no new query, just a filter
/// over data `App::refresh` already loaded (§11.3's own note).
pub fn render_recent_work(
    frame: &mut Frame,
    area: Rect,
    rows: &[WorkRow],
    screen: &WorkflowsScreen,
) {
    let block = Block::bordered()
        .title("Recent Work")
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(name) = screen.selected_entry().and_then(|e| e["name"].as_str()) else {
        frame.render_widget(
            Paragraph::new("Select a workflow to see recent Work using it."),
            inner,
        );
        return;
    };
    let matching: Vec<&WorkRow> = rows.iter().filter(|row| row.workflow == name).collect();
    if matching.is_empty() {
        frame.render_widget(
            Paragraph::new(format!("No recent Work used {name}.")),
            inner,
        );
        return;
    }
    let items: Vec<ListItem> = matching
        .iter()
        .map(|row| {
            ListItem::new(Line::from(vec![
                Span::raw(row.intent.clone()),
                Span::raw("  "),
                Span::styled(
                    row.state.clone(),
                    Style::default().fg(theme::state_token(&row.state).rgb()),
                ),
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
    use serde_json::json;

    fn entry(name: &str) -> Value {
        json!({
            "name": name,
            "version": "2",
            "source": "/repo/.sergeant/workflows/".to_string() + name,
            "content_hash": "a".repeat(64),
            "status": "published",
            "description": format!("does {name} things"),
            "tags": ["implementation"],
            "stages": [
                {"id": "10-implement", "kind": "actor", "harness": null, "profile": null, "requires_ask": false},
                {"id": "30-review", "kind": "actor", "harness": "claude", "profile": "review", "requires_ask": true},
            ],
        })
    }

    fn embedded_entry() -> Value {
        json!({
            "name": "software-change",
            "version": "1",
            "source": "embedded",
            "content_hash": "b".repeat(64),
            "stages": [
                {"id": "00-prepare", "kind": "actor", "harness": null, "profile": null, "requires_ask": false},
            ],
        })
    }

    #[test]
    fn j_k_move_selection_within_bounds() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement"), entry("diagnose-bug")]);
        assert_eq!(screen.selected_entry().unwrap()["name"], "implement");
        screen.on_key(KeyCode::Char('j'));
        assert_eq!(screen.selected_entry().unwrap()["name"], "diagnose-bug");
        screen.on_key(KeyCode::Char('j')); // already at the bottom
        assert_eq!(screen.selected_entry().unwrap()["name"], "diagnose-bug");
        screen.on_key(KeyCode::Char('k'));
        assert_eq!(screen.selected_entry().unwrap()["name"], "implement");
        screen.on_key(KeyCode::Char('k')); // already at the top
        assert_eq!(screen.selected_entry().unwrap()["name"], "implement");
    }

    #[test]
    fn slash_focuses_the_filter_and_typing_narrows_the_visible_list() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement"), entry("diagnose-bug")]);
        assert!(!screen.wants_text_focus());
        screen.on_key(KeyCode::Char('/'));
        assert!(screen.wants_text_focus());
        for c in "diag".chars() {
            screen.on_key(KeyCode::Char(c));
        }
        let names: Vec<&str> = screen
            .visible()
            .iter()
            .map(|e| e["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, vec!["diagnose-bug"]);
        screen.on_key(KeyCode::Enter);
        assert!(
            !screen.wants_text_focus(),
            "Enter applies and leaves the filter field"
        );
    }

    #[test]
    fn a_filter_matching_nothing_leaves_no_selection() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement")]);
        screen.on_key(KeyCode::Char('/'));
        for c in "nope".chars() {
            screen.on_key(KeyCode::Char(c));
        }
        assert!(screen.selected_entry().is_none());
    }

    /// §15.4/T2-56's Workflows half (issue #153): `@` asks `App` to open the
    /// live-catalog chooser rather than doing anything locally — it does not
    /// touch the `/` filter or the current selection.
    #[test]
    fn at_sign_asks_to_open_the_workflow_chooser() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement"), entry("diagnose-bug")]);
        assert_eq!(
            screen.on_key(KeyCode::Char('@')),
            WorkflowsOutcome::OpenWorkflowChooser
        );
        assert!(
            !screen.wants_text_focus(),
            "no literal '@' opened the filter"
        );
        assert_eq!(screen.selected_entry().unwrap()["name"], "implement");
    }

    #[test]
    fn select_by_name_moves_the_selection_to_the_named_entry() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement"), entry("diagnose-bug")]);
        screen.select_by_name("diagnose-bug");
        assert_eq!(screen.selected_entry().unwrap()["name"], "diagnose-bug");
    }

    #[test]
    fn select_by_name_with_an_unknown_name_leaves_the_selection_unchanged() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement"), entry("diagnose-bug")]);
        screen.select_by_name("does-not-exist");
        assert_eq!(screen.selected_entry().unwrap()["name"], "implement");
    }

    #[test]
    fn select_by_name_clears_a_stale_local_filter_that_hides_the_pick() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement"), entry("diagnose-bug")]);
        screen.on_key(KeyCode::Char('/'));
        for c in "diag".chars() {
            screen.on_key(KeyCode::Char(c));
        }
        screen.on_key(KeyCode::Enter);
        assert_eq!(
            screen.visible().len(),
            1,
            "the local filter is still narrowing the list"
        );

        screen.select_by_name("implement");

        assert_eq!(
            screen.selected_entry().unwrap()["name"],
            "implement",
            "a live-chooser pick must land even when it doesn't match a stale local filter"
        );
    }

    #[test]
    fn u_hands_the_selected_workflows_name_to_use_in_new_work() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement"), entry("diagnose-bug")]);
        screen.on_key(KeyCode::Char('j'));
        assert_eq!(
            screen.on_key(KeyCode::Char('u')),
            WorkflowsOutcome::UseInNewWork("diagnose-bug".to_string())
        );
    }

    #[test]
    fn u_with_no_workflows_loaded_is_a_no_op() {
        let mut screen = WorkflowsScreen::default();
        assert_eq!(screen.on_key(KeyCode::Char('u')), WorkflowsOutcome::None);
    }

    #[test]
    fn set_entries_keeps_the_same_workflow_selected_by_name() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement"), entry("diagnose-bug")]);
        screen.on_key(KeyCode::Char('j'));
        assert_eq!(screen.selected_entry().unwrap()["name"], "diagnose-bug");
        // A refresh reorders the catalog; the same workflow stays selected.
        screen.set_entries(vec![entry("diagnose-bug"), entry("implement")]);
        assert_eq!(screen.selected_entry().unwrap()["name"], "diagnose-bug");
    }

    fn render_to_text(screen: &WorkflowsScreen) -> String {
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 30))
            .expect("test terminal");
        terminal
            .draw(|frame| render(frame, frame.area(), screen))
            .expect("draw");
        let buffer = terminal.backend().buffer().clone();
        let area = buffer.area;
        (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buffer.cell((x, y)).map_or(" ", |c| c.symbol()))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn the_detail_pane_shows_11_3s_fields_for_a_repository_workflow() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement")]);
        let text = render_to_text(&screen);
        assert!(text.contains("implement v2"), "{text}");
        assert!(text.contains("live catalog"), "{text}");
        assert!(text.contains("published"), "{text}");
        assert!(text.contains("does implement things"), "{text}");
        assert!(text.contains("implementation"), "{text}");
        assert!(text.contains("10-implement"), "{text}");
        assert!(text.contains("30-review"), "{text}");
        assert!(text.contains("claude"), "{text}");
        assert!(text.contains("review"), "{text}");
        assert!(text.contains(&"a".repeat(64)[..20]), "{text}");
    }

    #[test]
    fn the_embedded_entry_is_labeled_distinctly_and_has_no_status_or_tags_line() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![embedded_entry()]);
        let text = render_to_text(&screen);
        assert!(text.contains("embedded fallback"), "{text}");
        assert!(text.contains("(embedded)"), "the list marks it too: {text}");
    }

    #[test]
    fn recent_work_shows_only_rows_using_the_selected_workflow() {
        let mut screen = WorkflowsScreen::default();
        screen.set_entries(vec![entry("implement"), entry("diagnose-bug")]);
        let rows = super::super::fleet::fleet_rows(&json!({"works": [
            {"id": "a", "state": "running", "intent": "using implement", "workflow": "implement"},
            {"id": "b", "state": "completed", "intent": "using diagnose-bug", "workflow": "diagnose-bug"},
        ]}));
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 20))
            .expect("test terminal");
        terminal
            .draw(|frame| render_recent_work(frame, frame.area(), &rows, &screen))
            .expect("draw");
        let buffer = terminal.backend().buffer().clone();
        let text: String = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.cell((x, y)).map_or(" ", |c| c.symbol()))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("using implement"), "{text}");
        assert!(!text.contains("using diagnose-bug"), "{text}");
    }
}

//! Fleet (§10): the complete Work browser.

use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState};
use serde_json::Value;

use crate::api::{field_text, stage_label};

use super::theme::{self, Tier, Token};

/// One row of the Fleet screen, projected from the `/v1/work` body (§10.2's
/// row priority: intent, state, stage, target, workflow, turn envelope,
/// backend/executor, age, short id).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkRow {
    pub id: String,
    pub state: String,
    pub stage: String,
    pub backend: String,
    pub intent: String,
    pub workflow: String,
    /// The canonical estate root this Work was submitted against
    /// (`estate_root`, H1 D1) — an identity, never a display name: two
    /// estates may share one, so this is deliberately not the old
    /// `workspace` field (dead since estate-root Phase C; `Work::workspace`'s
    /// own doc comment: "deprecated, never written"). `"-"` when the
    /// daemon has none recorded (a Work journaled before the envelope
    /// carried it, or with no estate context at all).
    pub estate: String,
    pub repositories: String,
    pub turns: String,
    pub created_at: String,
    /// The current stage's own detail text — a question when the state is
    /// `needs_input`, a reason when it is `blocked`/`failed` (§9.3's
    /// "question/reason when present").
    pub question: String,
}

/// Project the `/v1/work` body into fleet rows.
pub fn fleet_rows(fleet: &Value) -> Vec<WorkRow> {
    fleet["works"]
        .as_array()
        .map(|works| works.iter().map(row_from).collect())
        .unwrap_or_default()
}

fn row_from(work: &Value) -> WorkRow {
    let repositories = work["repositories"]
        .as_array()
        .map(|repos| {
            repos
                .iter()
                .filter_map(|r| r.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string());
    let turns = match (
        work["envelope"]["turns_spawned"].as_u64(),
        work["envelope"]["turn_cap"].as_u64(),
    ) {
        (Some(spawned), Some(cap)) => format!("{spawned}/{cap}"),
        (Some(spawned), None) => format!("{spawned}/-"),
        _ => "-".to_string(),
    };
    WorkRow {
        id: field(work, "id"),
        state: field(work, "state"),
        stage: stage_label(&work["stage"]),
        backend: field(work, "resolved_backend"),
        intent: field(work, "intent"),
        workflow: field(work, "workflow"),
        estate: field(work, "estate_root"),
        repositories,
        turns,
        created_at: field(work, "created_at"),
        question: field(&work["stage"], "detail"),
    }
}

/// A row's target: the estate when declared, else its repositories.
fn target(row: &WorkRow) -> String {
    if row.estate != "-" {
        row.estate.clone()
    } else {
        row.repositories.clone()
    }
}

fn field(value: &Value, key: &str) -> String {
    field_text(&value[key])
}

/// §10.4's reported states, in the order the Attention drawer (§7.3) and the
/// filter cycle both use.
pub const REPORTED_STATES: [&str; 9] = [
    "needs_input",
    "waiting",
    "blocked",
    "failed",
    "completed_dirty",
    "active",
    "pending",
    "completed",
    "canceled",
];

/// Whether a state is terminal (used by the terminal/nonterminal filter).
pub fn is_terminal(state: &str) -> bool {
    matches!(
        state,
        "completed" | "completed_dirty" | "failed" | "canceled"
    )
}

/// §10.3's local Fleet filters — fields already returned in the Fleet
/// projection, not Journal search.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Filters {
    pub text: String,
    pub state: Option<String>,
    pub nonterminal_only: bool,
    /// W4d deliverable 1 (H1 §9): the fleet-wide estate filter, following
    /// the exact `state` pattern above — `None` is "every estate" (host-wide,
    /// the honest default under host mode), `Some(root)` narrows to the one
    /// estate whose `WorkRow.estate` (its canonical root, D1 — see that
    /// field's own doc comment) matches exactly.
    pub estate: Option<String>,
}

impl Filters {
    pub fn matches(&self, row: &WorkRow) -> bool {
        if self.nonterminal_only && is_terminal(&row.state) {
            return false;
        }
        if let Some(state) = &self.state
            && &row.state != state
        {
            return false;
        }
        if let Some(estate) = &self.estate
            && &row.estate != estate
        {
            return false;
        }
        if !self.text.is_empty() {
            let needle = self.text.to_lowercase();
            if !row.intent.to_lowercase().contains(&needle)
                && !row.id.to_lowercase().contains(&needle)
            {
                return false;
            }
        }
        true
    }

    fn is_default(&self) -> bool {
        self.text.is_empty()
            && self.state.is_none()
            && !self.nonterminal_only
            && self.estate.is_none()
    }
}

/// What a Fleet keystroke asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FleetOutcome {
    None,
    /// Open the canonical Work surface for this id (§10.5: Enter opens it).
    Open(String),
}

/// Fleet's own screen state: selection, filters, whether the filter text
/// field currently has keyboard focus, and whether the Medium tier's
/// selected-preview toggle (§10.1/§18.2: "selected preview below or
/// toggle") is open.
#[derive(Debug, Default)]
pub struct FleetScreen {
    pub selected: usize,
    pub filters: Filters,
    pub filter_focus: bool,
    pub preview_open: bool,
}

impl FleetScreen {
    pub fn visible<'a>(&self, rows: &'a [WorkRow]) -> Vec<&'a WorkRow> {
        rows.iter()
            .filter(|row| self.filters.matches(row))
            .collect()
    }

    pub fn selected_id(&self, rows: &[WorkRow]) -> Option<String> {
        self.visible(rows)
            .get(self.selected)
            .map(|row| row.id.clone())
    }

    /// The selected row itself — the Wide contextual rail's "selected Work
    /// preview" (§10.1) reads this directly.
    pub fn selected_row<'a>(&self, rows: &'a [WorkRow]) -> Option<&'a WorkRow> {
        self.visible(rows).get(self.selected).copied()
    }

    /// Whether the filter text field currently owns the keyboard — App
    /// consults this before routing a keystroke to a global destination-nav
    /// key, so typing `/state` cannot be hijacked as a nav shortcut.
    pub fn wants_text_focus(&self) -> bool {
        self.filter_focus
    }

    /// Keep the selection in bounds after the visible set changes (e.g. a
    /// refresh, or a filter that just excluded the selected row).
    pub fn clamp(&mut self, rows: &[WorkRow]) {
        let n = self.visible(rows).len();
        if self.selected >= n {
            self.selected = n.saturating_sub(1);
        }
    }

    /// Interpret one keystroke. Mutations are explicitly out of this Work's
    /// scope (T1c owns respond/retry/extend/cancel) — Fleet only browses and
    /// opens the canonical Work surface.
    pub fn on_key(&mut self, key: KeyCode, rows: &[WorkRow]) -> FleetOutcome {
        if self.filter_focus {
            match key {
                KeyCode::Esc | KeyCode::Enter => self.filter_focus = false,
                KeyCode::Backspace => {
                    self.filters.text.pop();
                }
                KeyCode::Char(c) => self.filters.text.push(c),
                _ => {}
            }
            self.clamp(rows);
            return FleetOutcome::None;
        }
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                let n = self.visible(rows).len();
                if self.selected + 1 < n {
                    self.selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('/') => self.filter_focus = true,
            KeyCode::Char('a') => {
                self.filters.nonterminal_only = !self.filters.nonterminal_only;
                self.clamp(rows);
            }
            KeyCode::Char('s') => {
                self.filters.state = next_state_filter(self.filters.state.as_deref());
                self.clamp(rows);
            }
            KeyCode::Char('e') => {
                self.filters.estate = next_estate_filter(self.filters.estate.as_deref(), rows);
                self.clamp(rows);
            }
            KeyCode::Char('x') if !self.filters.is_default() => {
                self.filters = Filters::default();
            }
            // §10.1/§18.2's Medium-tier "selected preview below or toggle" —
            // a no-op at any other tier, harmlessly (nothing reads it there).
            KeyCode::Char('v') => self.preview_open = !self.preview_open,
            KeyCode::Enter => {
                if let Some(id) = self.selected_id(rows) {
                    return FleetOutcome::Open(id);
                }
            }
            _ => {}
        }
        FleetOutcome::None
    }
}

/// All-estates → per-estate cycling (W4d deliverable 1), the same shape as
/// [`next_state_filter`] but over whatever estate names are actually present
/// in `rows` — unlike `state`, the estate dimension has no fixed enum, so
/// the cycle is derived from the data rather than a `const` list, sorted for
/// a deterministic keystroke order.
fn next_estate_filter(current: Option<&str>, rows: &[WorkRow]) -> Option<String> {
    let mut estates: Vec<&str> = rows
        .iter()
        .map(|r| r.estate.as_str())
        .filter(|e| *e != "-")
        .collect();
    estates.sort_unstable();
    estates.dedup();
    match current {
        None => estates.first().map(|s| s.to_string()),
        Some(estate) => {
            let index = estates.iter().position(|s| *s == estate);
            index
                .and_then(|i| estates.get(i + 1))
                .map(|next| (*next).to_string())
        }
    }
}

fn next_state_filter(current: Option<&str>) -> Option<String> {
    match current {
        None => Some(REPORTED_STATES[0].to_string()),
        Some(state) => {
            let index = REPORTED_STATES.iter().position(|s| *s == state);
            index
                .and_then(|i| REPORTED_STATES.get(i + 1))
                .map(|next| (*next).to_string())
        }
    }
}

// -------------------------------------------------------------- rendering

/// Draw the Fleet browser: filters, then the Work list, composed per §10.1's
/// responsive layout — §18.1's Wide filters+table (the contextual rail's own
/// preview pane is `mod.rs`'s job, drawn beside this), §18.2's Medium
/// list-with-toggle (`v` opens a preview pane below the list), and §18.3's
/// Narrow stacked rows with the filter itself promoted to a full-body
/// overlay while it has focus (tight columns make a slim top line hard to
/// read while typing).
pub fn render(frame: &mut Frame, area: Rect, rows: &[WorkRow], screen: &FleetScreen, tier: Tier) {
    if tier == Tier::Narrow && screen.filter_focus {
        render_filter_overlay(frame, area, screen);
        return;
    }

    let visible = screen.visible(rows);
    let title = format!(
        "Fleet — {} work{}{}",
        visible.len(),
        if visible.len() == 1 { "" } else { "s" },
        if screen.filters.is_default() {
            ""
        } else {
            " (filtered)"
        }
    );
    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(Token::Border.rgb()))
        // §8.10's spacing scale: `none` — the filter line and the list touch
        // the border directly, no interior padding.
        .padding(ratatui::widgets::Padding::uniform(theme::spacing::NONE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let filter_line = filter_line(screen);
    let (filter_area, list_area) = split_top(inner, 1);
    frame.render_widget(filter_line, filter_area);

    if visible.is_empty() {
        let msg = if rows.is_empty() {
            "No Work yet\nDescribe an intent on Home."
        } else {
            "No Work matches the current filters — 'x' clears them."
        };
        frame.render_widget(Paragraph::new(msg), list_area);
        return;
    }

    // §18.2/§10.1: Medium's "selected preview below or toggle" — a bottom
    // pane that opens only when the operator asks for it (`v`), since
    // Medium has no reserved rail columns to spend on it unconditionally.
    let (list_area, preview_area) = if tier == Tier::Medium && screen.preview_open {
        let [list, preview] = ratatui::layout::Layout::vertical([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .areas(list_area);
        (list, Some(preview))
    } else {
        (list_area, None)
    };

    match tier {
        Tier::Narrow => render_stacked(frame, list_area, &visible, screen.selected),
        Tier::Medium => render_table(frame, list_area, &visible, screen.selected, false),
        _ => render_table(frame, list_area, &visible, screen.selected, true),
    }

    if let Some(preview_area) = preview_area {
        let selected = visible.get(screen.selected).copied();
        super::work_view::render_preview(frame, preview_area, selected);
    }
}

/// §18.3: Narrow's filter, promoted to a full-body overlay while it has
/// focus (mirrors how the Attention drawer and contextual rail themselves
/// become full-body overlays at Narrow, §18.3's own rule, rather than a
/// second, competing mechanism).
fn render_filter_overlay(frame: &mut Frame, area: Rect, screen: &FleetScreen) {
    let block = Block::bordered()
        .title("Fleet Filter")
        .border_style(Style::default().fg(Token::Focus.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = format!(
        "filter> {}_\n\nEnter/Esc applies and returns to the list.",
        screen.filters.text
    );
    frame.render_widget(Paragraph::new(text), inner);
}

fn filter_line(screen: &FleetScreen) -> Paragraph<'static> {
    let state = screen
        .filters
        .state
        .clone()
        .unwrap_or_else(|| "all".to_string());
    let nonterminal = if screen.filters.nonterminal_only {
        "on"
    } else {
        "off"
    };
    let estate = screen
        .filters
        .estate
        .clone()
        .unwrap_or_else(|| "all".to_string());
    let text = if screen.filter_focus {
        format!("filter> {}_", screen.filters.text)
    } else {
        format!(
            "filter: {} · state {state} · estate {estate} · nonterminal {nonterminal}  \
             ('/' filter · 's' state · 'e' estate · 'a' nonterminal · 'v' preview · 'x' clear)",
            if screen.filters.text.is_empty() {
                "-"
            } else {
                &screen.filters.text
            }
        )
    };
    Paragraph::new(text).style(Style::default().fg(Token::Muted.rgb()))
}

fn split_top(area: Rect, top: u16) -> (Rect, Rect) {
    let [head, rest] =
        ratatui::layout::Layout::vertical([Constraint::Length(top), Constraint::Min(0)])
            .areas(area);
    (head, rest)
}

/// Wide/Medium: a `Table`, whose own constraints truncate every cell — never
/// a hand-padded fixed-width string (§10.1's Decision T2-37).
fn render_table(frame: &mut Frame, area: Rect, rows: &[&WorkRow], selected: usize, wide: bool) {
    let mut header = vec![
        "", "state", "intent", "stage", "target", "workflow", "turns",
    ];
    let mut widths = vec![
        Constraint::Length(1),
        Constraint::Length(13),
        Constraint::Fill(3),
        Constraint::Fill(2),
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Length(7),
    ];
    if wide {
        header.extend(["backend", "age", "id"]);
        widths.extend([
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(14),
        ]);
    }

    let header_row = Row::new(header.iter().map(|h| Cell::from(*h))).style(
        Style::default()
            .fg(Token::Muted.rgb())
            .add_modifier(Modifier::BOLD),
    );

    let body_rows: Vec<Row> = rows
        .iter()
        .map(|row| {
            let mut cells = vec![
                Cell::from(theme::state_glyph(&row.state))
                    .style(Style::default().fg(theme::state_token(&row.state).rgb())),
                Cell::from(row.state.clone())
                    .style(Style::default().fg(theme::state_token(&row.state).rgb())),
                Cell::from(row.intent.clone()),
                Cell::from(row.stage.clone()),
                Cell::from(target(row)),
                // §8.1/§8.10: info/reference violet is reserved for
                // workflow references specifically, nowhere else.
                Cell::from(row.workflow.clone()).style(Style::default().fg(Token::Info.rgb())),
                Cell::from(row.turns.clone()),
            ];
            if wide {
                cells.push(Cell::from(row.backend.clone()));
                cells.push(Cell::from(age_label(&row.created_at)));
                cells.push(Cell::from(row.id.clone()));
            }
            Row::new(cells)
        })
        .collect();

    let table = Table::new(body_rows, widths)
        .header(header_row)
        // §8.10's spacing scale: `inline` (1 cell) is the blank column
        // between adjacent metadata fields within one row.
        .column_spacing(theme::spacing::INLINE)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");
    let mut state = TableState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(table, area, &mut state);
}

/// Narrow: intent-first, two-line stacked rows (§10.1/§18.3).
fn render_stacked(frame: &mut Frame, area: Rect, rows: &[&WorkRow], selected: usize) {
    let items: Vec<ListItem> = rows
        .iter()
        .map(|row| {
            let glyph_style = Style::default().fg(theme::state_token(&row.state).rgb());
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(theme::state_glyph(&row.state), glyph_style),
                    Span::raw(" "),
                    Span::styled(row.state.clone(), glyph_style),
                    Span::raw("  "),
                    Span::raw(row.intent.clone()),
                ]),
                Line::from(Span::styled(
                    format!(
                        "  {} · {} · {}",
                        row.stage,
                        target(row),
                        age_label(&row.created_at)
                    ),
                    Style::default().fg(Token::Muted.rgb()),
                )),
            ])
        })
        .collect();
    let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(list, area, &mut state);
}

/// A submission timestamp as a short relative age ("23m", "2h", "3d") —
/// §9.3/§10.2's "age labeled as submission age". Pure and dependency-free: no
/// date-and-time crate is added for this, so RFC3339 ("YYYY-MM-DDTHH:MM:SSZ",
/// the shape `Work::created_at`'s own doc comment promises) is parsed by
/// hand into seconds since the epoch.
pub fn age_label(created_at: &str) -> String {
    match (parse_rfc3339_secs(created_at), now_secs()) {
        (Some(then), Some(now)) if now >= then => format_age(now - then),
        _ => "-".to_string(),
    }
}

fn format_age(seconds: u64) -> String {
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h", seconds / 3600)
    } else {
        format!("{}d", seconds / 86400)
    }
}

fn now_secs() -> Option<u64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// A minimal RFC3339 UTC parser: `YYYY-MM-DDTHH:MM:SS[.fraction]Z`. Returns
/// `None` for anything else rather than guessing.
fn parse_rfc3339_secs(text: &str) -> Option<u64> {
    let bytes = text.as_bytes();
    if bytes.len() < 20 || bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'T' {
        return None;
    }
    let year: i64 = text.get(0..4)?.parse().ok()?;
    let month: u64 = text.get(5..7)?.parse().ok()?;
    let day: u64 = text.get(8..10)?.parse().ok()?;
    let hour: u64 = text.get(11..13)?.parse().ok()?;
    let minute: u64 = text.get(14..16)?.parse().ok()?;
    let second: u64 = text.get(17..19)?.parse().ok()?;
    if !(1970..=9999).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let days = days_since_epoch(year, month, day)?;
    Some(days * 86400 + hour * 3600 + minute * 60 + second)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_since_epoch(year: i64, month: u64, day: u64) -> Option<u64> {
    let mut days: i64 = 0;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    const DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (m, days_in_month) in DAYS_IN_MONTH.iter().enumerate().take((month - 1) as usize) {
        days += *days_in_month as i64;
        if m == 1 && is_leap_year(year) {
            days += 1;
        }
    }
    days += day as i64 - 1;
    u64::try_from(days).ok()
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
    fn a_fleet_row_is_the_api_body_projected_field_by_field() {
        let rows = fleet_rows(&fleet_of(&[("a", "needs_input")]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "a");
        assert_eq!(rows[0].state, "needs_input");
        assert_eq!(rows[0].stage, "10-implement 2/2 · running");
        assert_eq!(rows[0].backend, "fake");
        assert_eq!(rows[0].intent, "intent for a");

        let unrouted = fleet_rows(&json!({"works": [{"id": "b", "state": "pending"}]}));
        assert_eq!(unrouted[0].stage, "-");
        assert_eq!(unrouted[0].backend, "-");
        assert_eq!(unrouted[0].intent, "-");
        assert_eq!(unrouted[0].workflow, "-");

        assert!(fleet_rows(&json!({})).is_empty());
    }

    #[test]
    fn navigation_is_bounded_by_the_visible_rows() {
        let rows = fleet_rows(&fleet_of(&[("a", "pending"), ("b", "running")]));
        let mut screen = FleetScreen::default();
        screen.on_key(KeyCode::Up, &rows);
        assert_eq!(screen.selected, 0, "up at the top stays at the top");
        screen.on_key(KeyCode::Down, &rows);
        screen.on_key(KeyCode::Down, &rows);
        assert_eq!(screen.selected, 1, "down at the bottom stays at the bottom");
    }

    #[test]
    fn enter_opens_the_selected_work_and_nothing_else_does() {
        let rows = fleet_rows(&fleet_of(&[("a", "pending"), ("b", "running")]));
        let mut screen = FleetScreen::default();
        assert_eq!(screen.on_key(KeyCode::Char('j'), &rows), FleetOutcome::None);
        assert_eq!(
            screen.on_key(KeyCode::Enter, &rows),
            FleetOutcome::Open("b".to_string())
        );
    }

    #[test]
    fn a_completed_dirty_row_is_grouped_with_review_required_not_success() {
        assert_eq!(
            theme::state_token("completed_dirty"),
            theme::state_token("needs_input")
        );
        assert_ne!(
            theme::state_token("completed_dirty"),
            theme::state_token("completed")
        );
        assert!(
            is_terminal("completed_dirty"),
            "still terminal, for the nonterminal filter"
        );
    }

    #[test]
    fn the_text_filter_matches_intent_or_id_case_insensitively() {
        let rows = fleet_rows(&fleet_of(&[("alpha", "pending"), ("beta", "running")]));
        let filters = Filters {
            text: "BETA".to_string(),
            ..Filters::default()
        };
        let visible: Vec<&str> = rows
            .iter()
            .filter(|r| filters.matches(r))
            .map(|r| r.id.as_str())
            .collect();
        assert_eq!(visible, vec!["beta"]);
    }

    #[test]
    fn the_nonterminal_filter_drops_completed_and_canceled_and_failed() {
        let rows = fleet_rows(&fleet_of(&[
            ("a", "completed"),
            ("b", "running"),
            ("c", "canceled"),
        ]));
        let filters = Filters {
            nonterminal_only: true,
            ..Filters::default()
        };
        let visible: Vec<&str> = rows
            .iter()
            .filter(|r| filters.matches(r))
            .map(|r| r.id.as_str())
            .collect();
        assert_eq!(visible, vec!["b"]);
    }

    #[test]
    fn the_state_filter_cycles_through_every_reported_state_then_back_to_all() {
        let mut current = None;
        for expected in REPORTED_STATES {
            current = next_state_filter(current.as_deref());
            assert_eq!(current.as_deref(), Some(expected));
        }
        assert_eq!(
            next_state_filter(current.as_deref()),
            None,
            "cycles back to 'all'"
        );
    }

    #[test]
    fn the_estate_filter_matches_a_rows_estate_exactly() {
        let mut fleet = fleet_of(&[("a", "pending"), ("b", "running")]);
        fleet["works"][0]["estate_root"] = json!("payments");
        fleet["works"][1]["estate_root"] = json!("billing");
        let rows = fleet_rows(&fleet);
        let filters = Filters {
            estate: Some("payments".to_string()),
            ..Filters::default()
        };
        let visible: Vec<&str> = rows
            .iter()
            .filter(|r| filters.matches(r))
            .map(|r| r.id.as_str())
            .collect();
        assert_eq!(visible, vec!["a"]);
    }

    #[test]
    fn the_estate_filter_cycles_through_every_estate_present_then_back_to_all() {
        let mut fleet = fleet_of(&[("a", "pending"), ("b", "pending"), ("c", "pending")]);
        fleet["works"][0]["estate_root"] = json!("billing");
        fleet["works"][1]["estate_root"] = json!("payments");
        fleet["works"][2]["estate_root"] = json!("payments"); // duplicate, must not repeat
        let rows = fleet_rows(&fleet);

        let mut screen = FleetScreen::default();
        screen.on_key(KeyCode::Char('e'), &rows);
        assert_eq!(screen.filters.estate.as_deref(), Some("billing"));
        screen.on_key(KeyCode::Char('e'), &rows);
        assert_eq!(screen.filters.estate.as_deref(), Some("payments"));
        screen.on_key(KeyCode::Char('e'), &rows);
        assert_eq!(
            screen.filters.estate, None,
            "cycles back to 'all' after the last distinct estate"
        );
    }

    #[test]
    fn the_estate_filter_ignores_rows_with_no_estate() {
        // `"-"` is `field_text`'s absent-value marker (§10.2) — it must
        // never itself become a selectable filter value.
        let rows = fleet_rows(&fleet_of(&[("a", "pending")]));
        assert_eq!(rows[0].estate, "-");
        assert_eq!(next_estate_filter(None, &rows), None);
    }

    #[test]
    fn age_label_reads_a_relative_duration_from_rfc3339() {
        let now = now_secs().expect("system clock");
        let thirty_minutes_ago = now.saturating_sub(30 * 60);
        let stamp = seconds_to_rfc3339(thirty_minutes_ago);
        assert_eq!(age_label(&stamp), "30m");
        assert_eq!(age_label("not a timestamp"), "-");
        assert_eq!(age_label(""), "-");
    }

    /// The inverse of `parse_rfc3339_secs`, for this test's own fixture only.
    fn seconds_to_rfc3339(total: u64) -> String {
        let days = total / 86400;
        let rem = total % 86400;
        let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);
        let mut year = 1970i64;
        let mut remaining_days = days as i64;
        loop {
            let year_days = if is_leap_year(year) { 366 } else { 365 };
            if remaining_days < year_days {
                break;
            }
            remaining_days -= year_days;
            year += 1;
        }
        const DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut month = 1u64;
        for (index, &days_in_month) in DAYS_IN_MONTH.iter().enumerate() {
            let mut d = days_in_month;
            if index == 1 && is_leap_year(year) {
                d += 1;
            }
            if (remaining_days as u64) < d {
                month = index as u64 + 1;
                break;
            }
            remaining_days -= d as i64;
        }
        let day = remaining_days as u64 + 1;
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
    }

    #[test]
    fn parse_rfc3339_round_trips_a_known_instant() {
        // 2024-01-01T00:00:00Z is a widely-checkable fixed point.
        assert_eq!(
            parse_rfc3339_secs("2024-01-01T00:00:00Z"),
            Some(1_704_067_200)
        );
    }

    // --------------------------------------------- T1c: responsive Fleet

    fn render_text(rows: &[WorkRow], screen: &FleetScreen, tier: Tier, width: u16) -> String {
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, 30))
            .expect("terminal");
        terminal
            .draw(|frame| render(frame, frame.area(), rows, screen, tier))
            .expect("draw");
        let buffer = terminal.backend().buffer();
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
    fn v_toggles_the_medium_preview_pane() {
        let rows = fleet_rows(&fleet_of(&[("a", "running")]));
        let mut screen = FleetScreen::default();
        assert!(!screen.preview_open);
        screen.on_key(KeyCode::Char('v'), &rows);
        assert!(screen.preview_open);
        screen.on_key(KeyCode::Char('v'), &rows);
        assert!(!screen.preview_open);
    }

    #[test]
    fn medium_shows_the_preview_pane_only_once_toggled_open() {
        let rows = fleet_rows(&fleet_of(&[("needle-id", "running")]));
        let mut screen = FleetScreen::default();
        let text = render_text(&rows, &screen, Tier::Medium, 120);
        assert!(
            !text.contains("Enter opens the full Work surface"),
            "closed by default: {text}"
        );

        screen.preview_open = true;
        let text = render_text(&rows, &screen, Tier::Medium, 120);
        assert!(
            text.contains("Enter opens the full Work surface"),
            "the preview pane must appear once opened: {text}"
        );
    }

    #[test]
    fn wide_ignores_the_medium_toggle_since_the_rail_is_always_inline() {
        let rows = fleet_rows(&fleet_of(&[("a", "running")]));
        let screen = FleetScreen {
            preview_open: true,
            ..FleetScreen::default()
        };
        let text = render_text(&rows, &screen, Tier::Wide, 200);
        assert!(
            !text.contains("Enter opens the full Work surface"),
            "Wide's rail is `mod.rs`'s job, not Fleet's own preview toggle: {text}"
        );
    }

    #[test]
    fn narrow_promotes_a_focused_filter_to_a_full_body_overlay() {
        let rows = fleet_rows(&fleet_of(&[("a", "running")]));
        let screen = FleetScreen {
            filter_focus: true,
            filters: Filters {
                text: "bud".to_string(),
                ..Filters::default()
            },
            ..FleetScreen::default()
        };
        let text = render_text(&rows, &screen, Tier::Narrow, 80);
        assert!(text.contains("Fleet Filter"), "{text}");
        assert!(text.contains("bud"), "{text}");
    }

    #[test]
    fn narrow_shows_the_ordinary_stacked_list_when_the_filter_is_not_focused() {
        let rows = fleet_rows(&fleet_of(&[("a", "running")]));
        let screen = FleetScreen::default();
        let text = render_text(&rows, &screen, Tier::Narrow, 80);
        assert!(!text.contains("Fleet Filter"), "{text}");
        assert!(text.contains("intent for a"), "{text}");
    }
}

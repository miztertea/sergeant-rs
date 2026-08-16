//! The global Attention drawer (§7.3, Decision T2-23) and the row shape
//! (§9.3) it shares with Home's wide composition — Home's left "Attention"
//! pane in §9's wide layout *is* this drawer, not a second, Home-owned copy
//! of the same idea.
//!
//! It stores no notification state of its own: every render regroups the
//! already-loaded Fleet rows, so there is nothing here that can drift from
//! what Fleet itself would say.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, Paragraph};

use super::fleet::{WorkRow, age_label};
use super::theme::{self, Token};

/// §7.3's five groups, in the drawer's own display order.
#[derive(Debug, Default)]
pub struct Groups<'a> {
    pub needs_input: Vec<&'a WorkRow>,
    pub stopped: Vec<&'a WorkRow>,
    pub waiting: Vec<&'a WorkRow>,
    pub running: Vec<&'a WorkRow>,
    pub finished: Vec<&'a WorkRow>,
}

/// How many Finished rows the drawer shows (§7.3: "bounds Finished rows").
pub const FINISHED_LIMIT: usize = 5;

/// Group Fleet rows into §7.3's buckets. Watch does not treat `pending`/
/// `active` as notice-producing, but the drawer is a fleet view rather than
/// the Watch protocol itself, so both land under Running here (§7.3).
pub fn group(rows: &[WorkRow]) -> Groups<'_> {
    let mut groups = Groups::default();
    for row in rows {
        match row.state.as_str() {
            "needs_input" => groups.needs_input.push(row),
            "blocked" | "failed" | "completed_dirty" => groups.stopped.push(row),
            "waiting" => groups.waiting.push(row),
            "pending" | "active" => groups.running.push(row),
            "completed" | "canceled" => groups.finished.push(row),
            _ => {}
        }
    }
    groups.finished.truncate(FINISHED_LIMIT);
    groups
}

/// The header's `? N` count (§7.1).
pub fn needs_input_count(rows: &[WorkRow]) -> usize {
    rows.iter().filter(|row| row.state == "needs_input").count()
}

/// The header's `! N` count (§7.1: blocked/failed/completed_dirty).
pub fn stopped_count(rows: &[WorkRow]) -> usize {
    rows.iter()
        .filter(|row| matches!(row.state.as_str(), "blocked" | "failed" | "completed_dirty"))
        .count()
}

/// Draw the drawer over `area` — used both for the global overlay/inline
/// panel and for Home's left pane at Wide.
pub fn render(frame: &mut Frame, area: Rect, rows: &[WorkRow]) {
    let groups = group(rows);
    let block = Block::bordered()
        .title("Attention")
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut items: Vec<ListItem<'static>> = Vec::new();
    push_section(&mut items, "NEEDS INPUT", &groups.needs_input);
    push_section(&mut items, "STOPPED", &groups.stopped);
    push_section(&mut items, "WAITING", &groups.waiting);
    push_section(&mut items, "RUNNING", &groups.running);
    push_section(&mut items, "FINISHED", &groups.finished);

    if items.is_empty() {
        frame.render_widget(Paragraph::new("Nothing needs attention."), inner);
        return;
    }
    frame.render_widget(List::new(items), inner);
}

fn push_section(items: &mut Vec<ListItem<'static>>, title: &str, rows: &[&WorkRow]) {
    if rows.is_empty() {
        return;
    }
    items.push(ListItem::new(Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Token::Muted.rgb()),
    ))));
    for row in rows {
        items.push(ListItem::new(row_line(row)));
    }
}

/// §9.3's row shape: state glyph, intent leading (not the ULID), the
/// question/reason when the stage has one, turns spawned/cap for active
/// Work, and the submission age.
pub fn row_line(row: &WorkRow) -> Line<'static> {
    let style = Style::default().fg(theme::state_token(&row.state).rgb());
    let mut spans = vec![
        Span::styled(format!("  {} ", theme::state_glyph(&row.state)), style),
        Span::raw(row.intent.clone()),
    ];
    if row.question != "-" {
        spans.push(Span::raw(format!("  — {}", row.question)));
    }
    if row.state == "active" && row.turns != "-" {
        spans.push(Span::styled(
            format!("  turns {}", row.turns),
            Style::default().fg(Token::Muted.rgb()),
        ));
    }
    spans.push(Span::styled(
        format!("  {}", age_label(&row.created_at)),
        Style::default().fg(Token::Muted.rgb()),
    ));
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(id: &str, state: &str) -> WorkRow {
        let value = json!({"id": id, "state": state, "intent": format!("intent {id}")});
        super::super::fleet::fleet_rows(&json!({"works": [value]}))
            .into_iter()
            .next()
            .unwrap()
    }

    #[test]
    fn every_state_lands_in_exactly_one_of_the_five_groups() {
        let rows = vec![
            row("a", "needs_input"),
            row("b", "blocked"),
            row("c", "failed"),
            row("d", "completed_dirty"),
            row("e", "waiting"),
            row("f", "pending"),
            row("g", "active"),
            row("h", "completed"),
            row("i", "canceled"),
        ];
        let groups = group(&rows);
        assert_eq!(
            groups
                .needs_input
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a"]
        );
        assert_eq!(
            groups
                .stopped
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b", "c", "d"],
            "completed_dirty is stopped, not a plain finish"
        );
        assert_eq!(
            groups
                .waiting
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            vec!["e"]
        );
        assert_eq!(
            groups
                .running
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            vec!["f", "g"]
        );
        assert_eq!(
            groups
                .finished
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            vec!["h", "i"]
        );
    }

    #[test]
    fn finished_rows_are_bounded() {
        let rows: Vec<WorkRow> = (0..(FINISHED_LIMIT + 3))
            .map(|i| row(&format!("w{i}"), "completed"))
            .collect();
        let groups = group(&rows);
        assert_eq!(groups.finished.len(), FINISHED_LIMIT);
    }

    #[test]
    fn header_counts_match_the_drawer_groups() {
        let rows = vec![
            row("a", "needs_input"),
            row("b", "needs_input"),
            row("c", "blocked"),
            row("d", "completed"),
        ];
        assert_eq!(needs_input_count(&rows), 2);
        assert_eq!(stopped_count(&rows), 1);
    }
}

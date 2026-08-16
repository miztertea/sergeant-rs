//! The canonical Work surface (§7.2, Decision T2-22): a stub for T1a.
//!
//! Every Work selection — Fleet's Enter key today, more entry points as
//! later Works land — opens one canonical full-body Work surface rather than
//! a per-screen detail view of its own. T1b builds the real
//! Thread/Workflow/Evidence/Graph/Details tabs (§13.2-§13.7) over
//! already-shipped API reads; this Work only needs the destination to exist
//! and to actually be reachable, so Enter lands somewhere real instead of a
//! dead key.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Wrap};

use super::fleet::WorkRow;
use super::theme::{self, Token};

pub fn render(frame: &mut Frame, area: Rect, row: Option<&WorkRow>) {
    let title = match row {
        Some(row) => format!("Work — {}", row.id),
        None => "Work".to_string(),
    };
    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let body = match row {
        Some(row) => {
            // The id gets its own line, unshared and unabbreviated — the
            // Block title above (and Fleet's own "short id" column, §10.2)
            // can truncate under real terminal widths, but a screen that
            // claims to be *this* Work's canonical surface should say which
            // one somewhere that does not.
            let mut body = format!(
                "{}\n\n{}\n\nstate      {} {}\nstage      {}\nworkflow   {}\n",
                row.id,
                row.intent,
                theme::state_glyph(&row.state),
                row.state,
                row.stage,
                row.workflow,
            );
            if row.question != "-" {
                body.push_str(&format!("question   {}\n", row.question));
            }
            body.push_str(
                "\nThe Thread, Workflow rail, Evidence, Graph, and Details tabs are \
                 T1b's job (§13.2-§13.7) — this is a stub over what Fleet already \
                 knows about this Work. Esc returns to where you were.",
            );
            body
        }
        None => "No Work selected.".to_string(),
    };
    frame.render_widget(Paragraph::new(body).wrap(Wrap { trim: false }), inner);
}

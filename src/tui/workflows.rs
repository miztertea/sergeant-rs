//! Workflows (§11): stub destination for T1a.
//!
//! T2 builds the `GET /v1/workflows` catalog route, the `ApiClient` method,
//! this screen's list/detail, Home's `@` chooser, and live-versus-pinned
//! labeling (§20.3). Nothing here reaches for that data early — there is no
//! route to reach it through yet.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Wrap};

use super::theme::Token;

pub fn render(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("Workflows")
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(
            "Workflow discovery is not built yet.\n\n\
             T2 adds the `GET /v1/workflows` catalog route, this list/detail \
             screen, and Home's `@` chooser (§11, §20.3).",
        )
        .wrap(Wrap { trim: false }),
        inner,
    );
}

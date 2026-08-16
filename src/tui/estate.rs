//! Estate (§12): stub destination for T1a.
//!
//! T3 builds the `/v1/estate/repos`, `/v1/estate/groups`, and `/v1/doctor`
//! API routes as thin wrappers over `crate::domain::manifest`/`mod doctor`,
//! and the Repositories/Groups/Health screens consumed through `ApiClient`
//! (§20.4). Nothing here reaches for that data early — there is no route to
//! reach it through yet, and this Work does not touch `src/api.rs`.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Wrap};

use super::theme::Token;

pub fn render(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("Estate")
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(
            "Estate is not built yet.\n\n\
             T3 adds the repo/group/doctor API routes and the Repositories, \
             Groups, and Health screens here (§12, §20.4).",
        )
        .wrap(Wrap { trim: false }),
        inner,
    );
}

//! Overlays (§7.4, Decision T2-24): a small fixed set of contextual views,
//! not a modal framework or second navigation system.
//!
//! The *mechanism* — how one opens, closes, and restores focus — is this
//! Work's job; most of the individual overlays' content is later Works'
//! (the workflow chooser needs T2's catalog, repo/group add-remove and the
//! retained-state preview need T3's Estate routes, cancel confirmation and
//! extend-envelope need T1c's mutations). Opening an overlay never mutates
//! the destination state underneath it — [`super::app::App::on_key`] simply
//! routes every keystroke to the overlay instead of the destination while
//! one is open — so closing it "restores focus" for free: nothing under it
//! ever moved.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use super::theme::Token;

/// §7.4's fixed set, in the order it lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    SlashPalette,
    WorkflowChooser,
    Help,
    CancelConfirmation,
    ExtendEnvelope,
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
            Overlay::ExtendEnvelope => "Extend Envelope",
            Overlay::RepoAddRemove => "Repository",
            Overlay::GroupEditRemove => "Group",
            Overlay::RetainedPreview => "Retained State",
            Overlay::ConnectionDetail => "Connection",
        }
    }

    /// The later Work that actually implements this overlay's content —
    /// `None` for [`Overlay::Help`], which T1a implements in full.
    fn owner(self) -> Option<&'static str> {
        match self {
            Overlay::Help => None,
            Overlay::SlashPalette | Overlay::CancelConfirmation | Overlay::ExtendEnvelope => {
                Some("T1c (§20.2's mutation and composer work)")
            }
            Overlay::WorkflowChooser => Some("T2 (§20.3's workflow discovery)"),
            Overlay::RepoAddRemove | Overlay::GroupEditRemove | Overlay::RetainedPreview => {
                Some("T3 (§20.4's Estate work)")
            }
            Overlay::ConnectionDetail => Some("a later Work"),
        }
    }
}

/// The keymap this Work actually wires up — the one overlay with real
/// content (§15.6).
const HELP_TEXT: &str = "\
Navigation
  1-4         Home / Fleet / Workflows / Estate
  Tab / S-Tab cycle destinations
  ~           toggle the Attention drawer
  ?           this help
  q / Esc     back, or quit from a top-level destination

Fleet
  j/k, ↑/↓    move
  Enter       open the selected Work
  /           filter by text
  s           cycle the state filter
  a           toggle nonterminal-only
  x           clear filters

Home
  Tab / S-Tab move between fields
  Enter       next field, or submit from [ Run Work ]
";

/// Draw the overlay over `area`, clearing what was there first (§8.3's
/// `Clear`) so a contextual panel never shows through stale cells.
pub fn render(frame: &mut Frame, area: Rect, overlay: Overlay) {
    let panel = centered(area, 70, 60);
    frame.render_widget(Clear, panel);
    let block = Block::bordered()
        .title(overlay.title())
        .border_style(Style::default().fg(Token::Focus.rgb()));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let body = match overlay.owner() {
        None => HELP_TEXT.to_string(),
        Some(owner) => format!(
            "{} is not built in this Work.\n\n{owner} implements it.\n\nEsc closes this panel.",
            overlay.title()
        ),
    };
    frame.render_widget(Paragraph::new(body).wrap(Wrap { trim: false }), inner);
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

    #[test]
    fn only_help_is_implemented_in_this_work() {
        assert!(Overlay::Help.owner().is_none());
        for later in [
            Overlay::SlashPalette,
            Overlay::WorkflowChooser,
            Overlay::CancelConfirmation,
            Overlay::ExtendEnvelope,
            Overlay::RepoAddRemove,
            Overlay::GroupEditRemove,
            Overlay::RetainedPreview,
            Overlay::ConnectionDetail,
        ] {
            assert!(later.owner().is_some(), "{later:?} must name who builds it");
        }
    }
}

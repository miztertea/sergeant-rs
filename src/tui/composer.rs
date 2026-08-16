//! The deliberate multiline composer (§8.7, §9.2, §15.1-15.2): the one
//! narrow wrapper (Decision T2-31) around `ratatui-textarea`'s `TextArea`
//! that every screen needing multiline deliberate input goes through — the
//! crate itself is never named outside this module.
//!
//! T0's spike (§8.7) found the crate has no submit concept of its own:
//! `TextArea::input()` maps every `Enter`, with or without modifiers, to
//! `insert_newline`. So [`Composer::on_key`] decides *before* forwarding a
//! keystroke whether it means "newline" or "submit" — exactly the shape the
//! spike's condition 6 requires.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui_textarea::{TextArea, WrapMode};

use super::theme::Token;

/// What one keystroke handed to the composer asked for (§15.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposerOutcome {
    /// The keystroke edited the draft, moved the cursor, or did nothing.
    None,
    /// A submit was asked for (Ctrl+Enter, or the caller's own
    /// [`Composer::submit`]) and the draft was nonblank.
    Submit(String),
    /// A submit was asked for with nothing but whitespace in the draft
    /// (§15.2: "blank input is refused").
    Refused,
    /// §15.3's Decision T2-55: `/` was typed at the first non-whitespace
    /// position of the draft — the caller opens `Overlay::SlashPalette`
    /// instead of this composer taking the character as text.
    OpenSlashPalette,
}

/// The deliberate multiline composer: Home's `INTENT` field and an open
/// Work's `ANSWER` field both go through this, never a single-line `String`
/// buffer and never `ratatui_textarea::TextArea` directly (Decision T2-31).
#[derive(Debug, Clone)]
pub struct Composer {
    area: TextArea<'static>,
    placeholder: String,
}

impl Composer {
    pub fn new() -> Self {
        let placeholder = String::new();
        Self {
            area: area_from_lines(vec![String::new()], &placeholder),
            placeholder,
        }
    }

    /// Text shown when the draft is empty — never mistaken for real content
    /// since `ratatui-textarea` only shows it while `lines()` is blank.
    pub fn with_placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self.area.set_placeholder_text(self.placeholder.clone());
        self
    }

    /// One keystroke (§15.2). `Ctrl+Enter` asks to submit rather than
    /// inserting a newline; `/` at the first non-whitespace position asks to
    /// open the slash palette rather than being inserted (§15.3); every
    /// other key — including plain `Enter`, which the crate itself maps to a
    /// newline — is forwarded to the editor untouched.
    pub fn on_key(&mut self, key: KeyEvent) -> ComposerOutcome {
        if key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::CONTROL) {
            return self.submit();
        }
        if key.code == KeyCode::Char('/') && self.at_first_nonwhitespace() {
            return ComposerOutcome::OpenSlashPalette;
        }
        self.area.input(key);
        ComposerOutcome::None
    }

    /// §15.3's trigger condition (Decision T2-55): every line above the
    /// cursor, and every character on the cursor's own line up to the
    /// cursor, is blank — so a `/` typed here can only be the start of a
    /// command, never a character inside typed text.
    fn at_first_nonwhitespace(&self) -> bool {
        let cursor = self.area.cursor();
        let lines = self.area.lines();
        lines[..cursor.0].iter().all(|line| line.trim().is_empty())
            && lines[cursor.0]
                .chars()
                .take(cursor.1)
                .all(char::is_whitespace)
    }

    /// Ask to submit the current draft outright — the Tab-to-Send-then-Enter
    /// fallback (§15.2/§8.8) reaches this the same way Ctrl+Enter does,
    /// through whichever caller owns the Send/Run/Confirm control's focus.
    pub fn submit(&mut self) -> ComposerOutcome {
        if self.is_empty() {
            return ComposerOutcome::Refused;
        }
        ComposerOutcome::Submit(self.text())
    }

    /// The draft, lines joined by `\n`.
    pub fn text(&self) -> String {
        self.area.lines().join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.text().trim().is_empty()
    }

    /// Replace the draft outright — used by tests and by any caller that
    /// needs to seed a draft without walking keystrokes.
    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(str::to_string).collect()
        };
        self.area = area_from_lines(lines, &self.placeholder);
    }

    /// The draft clears only after an accepted mutation (§9.2/§15.2).
    pub fn clear(&mut self) {
        self.area = area_from_lines(vec![String::new()], &self.placeholder);
    }

    /// Draw the composer into `area`. The caller owns any surrounding
    /// block/label (§15.1's contextual `INTENT`/`ANSWER` framing, and
    /// whether it looks focused) — this only ever draws the editable text
    /// itself. `&self` rather than `&mut self` on purpose: the app's whole
    /// draw path is immutable (`draw(frame, app: &App)`, `mod.rs`'s own
    /// "pure: everything it paints is already in `app`" rule), so the
    /// cursor's own style is fixed at construction rather than toggled here.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.area, area);
    }
}

impl Default for Composer {
    fn default() -> Self {
        Self::new()
    }
}

fn area_from_lines(lines: Vec<String>, placeholder: &str) -> TextArea<'static> {
    let mut area = TextArea::new(lines);
    area.set_cursor_line_style(Style::default());
    area.set_wrap_mode(WrapMode::WordOrGlyph);
    area.set_placeholder_text(placeholder.to_string());
    area.set_placeholder_style(Style::default().fg(Token::Muted.rgb()));
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::from(code)
    }

    fn ctrl_enter() -> KeyEvent {
        KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)
    }

    #[test]
    fn typing_edits_the_draft_and_enter_alone_is_a_newline_not_a_submit() {
        let mut composer = Composer::new();
        for c in "line one".chars() {
            assert_eq!(
                composer.on_key(key(KeyCode::Char(c))),
                ComposerOutcome::None
            );
        }
        assert_eq!(composer.on_key(key(KeyCode::Enter)), ComposerOutcome::None);
        for c in "line two".chars() {
            composer.on_key(key(KeyCode::Char(c)));
        }
        assert_eq!(composer.text(), "line one\nline two");
    }

    #[test]
    fn ctrl_enter_submits_a_nonblank_draft_without_inserting_a_newline() {
        let mut composer = Composer::new();
        for c in "do the thing".chars() {
            composer.on_key(key(KeyCode::Char(c)));
        }
        assert_eq!(
            composer.on_key(ctrl_enter()),
            ComposerOutcome::Submit("do the thing".to_string())
        );
        assert_eq!(
            composer.text(),
            "do the thing",
            "Ctrl+Enter must not have inserted a newline (T0's condition 6: no \
             editor-owned submit behavior)"
        );
    }

    #[test]
    fn blank_input_is_refused() {
        let mut composer = Composer::new();
        assert_eq!(composer.on_key(ctrl_enter()), ComposerOutcome::Refused);
        for c in "   ".chars() {
            composer.on_key(key(KeyCode::Char(c)));
        }
        assert_eq!(
            composer.on_key(ctrl_enter()),
            ComposerOutcome::Refused,
            "whitespace-only is still blank"
        );
    }

    #[test]
    fn slash_on_an_empty_draft_opens_the_palette_instead_of_being_typed() {
        let mut composer = Composer::new();
        assert_eq!(
            composer.on_key(key(KeyCode::Char('/'))),
            ComposerOutcome::OpenSlashPalette
        );
        assert_eq!(composer.text(), "", "the '/' must not have been inserted");
    }

    #[test]
    fn slash_after_leading_whitespace_still_opens_the_palette() {
        let mut composer = Composer::new();
        for c in "   ".chars() {
            composer.on_key(key(KeyCode::Char(c)));
        }
        assert_eq!(
            composer.on_key(key(KeyCode::Char('/'))),
            ComposerOutcome::OpenSlashPalette
        );
        assert_eq!(composer.text(), "   ", "leading whitespace is untouched");
    }

    #[test]
    fn slash_elsewhere_in_the_draft_stays_literal_text() {
        let mut composer = Composer::new();
        for c in "fix the /path".chars() {
            assert_eq!(composer.on_key(key(KeyCode::Char(c))), ComposerOutcome::None);
        }
        assert_eq!(composer.text(), "fix the /path");
    }

    #[test]
    fn slash_on_a_later_line_is_literal_when_the_first_line_has_content() {
        let mut composer = Composer::new();
        for c in "first".chars() {
            composer.on_key(key(KeyCode::Char(c)));
        }
        composer.on_key(key(KeyCode::Enter));
        assert_eq!(
            composer.on_key(key(KeyCode::Char('/'))),
            ComposerOutcome::None,
            "an earlier nonblank line means this is not the first non-whitespace \
             position of the draft"
        );
        assert_eq!(composer.text(), "first\n/");
    }

    #[test]
    fn slash_at_the_start_of_a_blank_second_line_still_opens_the_palette() {
        let mut composer = Composer::new();
        composer.on_key(key(KeyCode::Enter));
        assert_eq!(
            composer.on_key(key(KeyCode::Char('/'))),
            ComposerOutcome::OpenSlashPalette,
            "every line above the cursor is blank, so this is still the draft's \
             first non-whitespace position"
        );
    }

    #[test]
    fn submit_is_the_same_refusal_rule_as_the_tab_to_send_fallback() {
        let mut composer = Composer::new();
        assert_eq!(composer.submit(), ComposerOutcome::Refused);
        composer.set_text("go");
        assert_eq!(composer.submit(), ComposerOutcome::Submit("go".to_string()));
    }

    #[test]
    fn clear_resets_to_an_empty_draft() {
        let mut composer = Composer::new();
        composer.set_text("keep me? no");
        composer.clear();
        assert!(composer.is_empty());
        assert_eq!(composer.text(), "");
    }

    #[test]
    fn set_text_round_trips_multiline_content() {
        let mut composer = Composer::new();
        composer.set_text("first\nsecond\nthird");
        assert_eq!(composer.text(), "first\nsecond\nthird");
    }
}

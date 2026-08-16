//! §8.10's locked visual tokens and responsive geometries — the concrete
//! numbers T1 builds against, so no screen module invents its own color or
//! breakpoint.

use ratatui::style::Color;

/// One semantic color role (§8.1/§8.10). §8.10 also locks an ANSI/16-color
/// fallback per token for a terminal that cannot report truecolor or
/// 256-color, but this codebase has no terminal-capability probe to gate
/// that fallback on yet (§8.1's "existing Crossterm/terminfo practice" does
/// not exist here today) — every named `Color` below is truecolor
/// ([`Color::Rgb`]) unconditionally, matching what the current M6-era
/// `src/tui.rs` already did for its own, smaller palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Background,
    Surface,
    SurfaceMuted,
    Border,
    Text,
    Muted,
    Focus,
    Success,
    Attention,
    Warning,
    Danger,
    Info,
}

impl Token {
    /// The truecolor value (§8.10's Color::Rgb column).
    pub const fn rgb(self) -> Color {
        match self {
            Token::Background => Color::Rgb(12, 15, 19),
            Token::Surface => Color::Rgb(18, 22, 28),
            Token::SurfaceMuted => Color::Rgb(24, 29, 37),
            Token::Border => Color::Rgb(43, 50, 60),
            Token::Text => Color::Rgb(230, 233, 239),
            Token::Muted => Color::Rgb(124, 134, 149),
            Token::Focus => Color::Rgb(87, 182, 201),
            Token::Success => Color::Rgb(63, 185, 104),
            Token::Attention => Color::Rgb(215, 167, 44),
            Token::Warning => Color::Rgb(214, 138, 60),
            Token::Danger => Color::Rgb(224, 86, 79),
            Token::Info => Color::Rgb(164, 135, 224),
        }
    }
}

/// The spacing scale (§8.10): a terminal cell is the only unit.
pub mod spacing {
    /// A divider or border glyph touches its neighboring region directly.
    pub const NONE: u16 = 0;
    /// Gap between a state glyph and its label; blank column between
    /// adjacent metadata fields within one row.
    pub const INLINE: u16 = 1;
    /// A focusable `Block`'s interior padding; a blank row separating two
    /// stacked sections with no divider between them.
    pub const BLOCK: u16 = 2;
    /// Outer margin between the Attention drawer or contextual rail and the
    /// primary body at Wide composition (§18.1).
    pub const REGION: u16 = 3;
}

/// The responsive tier a frame's interior falls into (§8.10/§18, locked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// ≥ 150 columns: drawer, primary body, and contextual rail all inline.
    Wide,
    /// 100-149 columns: drawer inline-optional, no reserved rail columns.
    Medium,
    /// 80-99 columns: primary body only; drawer/rail are overlays.
    Narrow,
    /// Below the 80-column or 24-row floor: §17.7's notice, no composition.
    TooSmall,
}

/// The column floor below which composition is refused (§18, §17.7).
pub const MIN_COLUMNS: u16 = 80;
/// The row floor below which composition is refused (§18.4).
pub const MIN_ROWS: u16 = 24;
/// The column floor of the Medium tier.
pub const MEDIUM_MIN_COLUMNS: u16 = 100;
/// The column floor of the Wide tier.
pub const WIDE_MIN_COLUMNS: u16 = 150;

/// The Attention drawer's fixed inline width at Wide.
pub const DRAWER_WIDTH_WIDE: u16 = 28;
/// The Attention drawer's fixed inline width at Medium, when open.
pub const DRAWER_WIDTH_MEDIUM: u16 = 24;
/// The contextual rail's fixed inline width at Wide.
pub const RAIL_WIDTH_WIDE: u16 = 32;

impl Tier {
    /// The tier a frame of `columns` × `rows` falls into.
    pub fn for_size(columns: u16, rows: u16) -> Tier {
        if columns < MIN_COLUMNS || rows < MIN_ROWS {
            Tier::TooSmall
        } else if columns >= WIDE_MIN_COLUMNS {
            Tier::Wide
        } else if columns >= MEDIUM_MIN_COLUMNS {
            Tier::Medium
        } else {
            Tier::Narrow
        }
    }
}

/// One important state's glyph (§8.4). Every glyph is exactly one cell wide
/// (`·`/`✓` included — narrow in every measured terminal font, not merely
/// ASCII) and printable in any UTF-8-capable terminal, so no fallback table
/// is needed for reduced Unicode capability.
pub fn state_glyph(state: &str) -> &'static str {
    match state {
        "pending" => "\u{b7}", // ·
        "active" => "*",       // spinner frame; the loop's tick animates it
        "waiting" => "o",      // a static, ASCII-safe stand-in for ○
        "needs_input" => "?",
        "blocked" | "completed_dirty" => "!",
        "failed" => "x",
        "completed" => "\u{2713}", // ✓
        "canceled" => "-",
        _ => "?",
    }
}

/// A state's semantic color token (§8.4/§8.10), shared by every screen that
/// paints a state — the fleet table, the Attention drawer, the header counts.
pub fn state_token(state: &str) -> Token {
    match state {
        "completed" => Token::Success,
        // ADR 0007(b): a stranded completion is terminal, not a plain
        // success — group it with the other "needs a look" states.
        "needs_input" | "waiting" | "completed_dirty" => Token::Attention,
        "failed" | "blocked" | "canceled" => Token::Danger,
        _ => Token::Focus,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_land_on_the_locked_breakpoints() {
        assert_eq!(
            Tier::for_size(79, 24),
            Tier::TooSmall,
            "below the column floor"
        );
        assert_eq!(
            Tier::for_size(80, 23),
            Tier::TooSmall,
            "below the row floor"
        );
        assert_eq!(Tier::for_size(80, 24), Tier::Narrow);
        assert_eq!(Tier::for_size(99, 24), Tier::Narrow);
        assert_eq!(Tier::for_size(100, 24), Tier::Medium);
        assert_eq!(Tier::for_size(149, 24), Tier::Medium);
        assert_eq!(Tier::for_size(150, 24), Tier::Wide);
        assert_eq!(Tier::for_size(180, 48), Tier::Wide);
    }

    #[test]
    fn every_glyph_is_exactly_one_cell_wide() {
        for state in [
            "pending",
            "active",
            "waiting",
            "needs_input",
            "blocked",
            "completed_dirty",
            "failed",
            "completed",
            "canceled",
        ] {
            assert_eq!(
                state_glyph(state).chars().count(),
                1,
                "glyph for {state} must be one cell wide"
            );
        }
    }

    #[test]
    fn completed_dirty_is_not_a_plain_success() {
        assert_eq!(state_token("completed_dirty"), Token::Attention);
        assert_ne!(state_token("completed_dirty"), state_token("active"));
    }
}

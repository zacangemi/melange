use ratatui::style::{Color, Modifier, Style};

// ARRAKIS color palette
pub const SAND: Color = Color::Rgb(212, 165, 116);       // #D4A574 — primary text
pub const SPICE_ORANGE: Color = Color::Rgb(232, 145, 58); // #E8913A — highlights
pub const DEEP_DESERT: Color = Color::Rgb(139, 105, 20);  // #8B6914 — borders
pub const FREMEN_BLUE: Color = Color::Rgb(59, 125, 216);  // #3B7DD8 — safe
pub const NIGHT_SKY: Color = Color::Rgb(26, 26, 46);      // #1A1A2E — background
pub const SANDWORM_GOLD: Color = Color::Rgb(201, 169, 89); // #C9A959 — titles
pub const HARKONNEN_RED: Color = Color::Rgb(178, 60, 50);  // #B23C32 — danger

// Convenience styles
pub fn title_style() -> Style {
    Style::default().fg(SANDWORM_GOLD).add_modifier(Modifier::BOLD)
}

pub fn text_style() -> Style {
    Style::default().fg(SAND)
}

pub fn highlight_style() -> Style {
    Style::default().fg(SPICE_ORANGE).add_modifier(Modifier::BOLD)
}

pub fn safe_style() -> Style {
    Style::default().fg(FREMEN_BLUE).add_modifier(Modifier::BOLD)
}

pub fn danger_style() -> Style {
    Style::default().fg(HARKONNEN_RED).add_modifier(Modifier::BOLD)
}

pub fn border_style() -> Style {
    Style::default().fg(DEEP_DESERT)
}

pub fn selected_style() -> Style {
    Style::default().fg(SPICE_ORANGE).bg(Color::Rgb(55, 40, 20)).add_modifier(Modifier::BOLD)
}

pub fn dim_style() -> Style {
    Style::default().fg(DEEP_DESERT)
}

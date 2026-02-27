use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::text::{Line, Span};

use super::theme;

const SANDWORM: &str = r#"
         ___
    ___/ o \____
       ___     \___________
      /   \___             \
               \___   ___  |
                   \_/  \_|
"#;

pub fn draw(f: &mut Frame) {
    let area = f.area();
    f.render_widget(Clear, area);

    let mut lines: Vec<Line> = Vec::new();

    // Add vertical padding
    let art_height = 10;
    let pad_top = area.height.saturating_sub(art_height as u16) / 2;
    for _ in 0..pad_top {
        lines.push(Line::from(""));
    }

    // Sandworm ASCII art
    for line in SANDWORM.lines() {
        lines.push(Line::from(Span::styled(line.to_string(), theme::highlight_style())));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "M E L A N G E  v0.1.0",
        theme::title_style(),
    )));
    lines.push(Line::from(Span::styled(
        "\"The memory must flow...\"",
        theme::dim_style(),
    )));

    let splash = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
        );

    f.render_widget(splash, area);
}

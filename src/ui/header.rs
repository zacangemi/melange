use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};
use ratatui::layout::Alignment;

use super::theme;

pub fn draw(f: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled("  ≈≈≈ ", theme::border_style()),
        Span::styled("M E L A N G E", theme::title_style()),
        Span::styled(" ≈≈≈  ", theme::border_style()),
        Span::styled("v0.1.0", theme::dim_style()),
        Span::styled("  \"The memory must flow\"", theme::highlight_style()),
    ]);

    let header = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
        );

    f.render_widget(header, area);
}

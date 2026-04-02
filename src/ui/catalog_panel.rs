use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};
use ratatui::layout::Alignment;

use super::theme;
use crate::app::App;

pub fn draw(f: &mut Frame, _app: &App, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Model Catalog — Coming Soon",
            theme::title_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Browse and discover models from HuggingFace,",
            theme::dim_style(),
        )),
        Line::from(Span::styled(
            "filtered by your hardware and installed engines.",
            theme::dim_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [Tab] to return to Local view",
            theme::text_style(),
        )),
    ];

    let panel = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(Span::styled(" CATALOG ", theme::title_style()))
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
        );

    f.render_widget(panel, area);
}

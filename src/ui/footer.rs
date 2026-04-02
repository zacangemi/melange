use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

use super::theme;
use crate::app::App;
use crate::dune::quotes;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let quote = quotes::get_quote(app.quote_index);

    let line = Line::from(vec![
        Span::styled(format!(" \"{}\"", quote), theme::dim_style()),
    ]);

    let footer = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
    );

    f.render_widget(footer, area);
}

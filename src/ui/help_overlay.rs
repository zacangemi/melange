use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::text::{Line, Span};

use super::theme;

pub fn draw(f: &mut Frame) {
    let area = centered_rect(44, 16, f.area());

    // Clear the area behind the popup
    f.render_widget(Clear, area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  q ", theme::highlight_style()),
            Span::styled("/ ", theme::dim_style()),
            Span::styled("Esc", theme::highlight_style()),
            Span::styled("      Quit", theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled("  j ", theme::highlight_style()),
            Span::styled("/ ", theme::dim_style()),
            Span::styled("k", theme::highlight_style()),
            Span::styled("        Navigate models", theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled("  Tab", theme::highlight_style()),
            Span::styled("          Switch view", theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled("  r", theme::highlight_style()),
            Span::styled("            Refresh", theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled("  v", theme::highlight_style()),
            Span::styled("            Toggle VPN IP", theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled("  ?", theme::highlight_style()),
            Span::styled("            This help", theme::text_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press any key to close", theme::dim_style()),
        ]),
    ];

    let popup = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(" KEYBINDINGS ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
    );

    f.render_widget(popup, area);
}

/// Create a centered rectangle of fixed size within the given area.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(area.height.saturating_sub(height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(area.width.saturating_sub(width) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vert[1]);

    horiz[1]
}

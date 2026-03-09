use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

use super::theme;
use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let mem = &app.hardware.memory;
    let usage_pct = mem.usage_percent();

    // Build gauge bar manually with block chars
    let bar_width = (area.width as usize).saturating_sub(6).min(40);
    let filled = ((usage_pct / 100.0) * bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);

    let gauge_color = if usage_pct > 90.0 {
        theme::HARKONNEN_RED
    } else if usage_pct > 75.0 {
        theme::SPICE_ORANGE
    } else {
        theme::FREMEN_BLUE
    };

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    let lines = vec![
        Line::from(vec![
            Span::styled("  ", theme::text_style()),
            Span::styled(&bar, ratatui::style::Style::default().fg(gauge_color)),
            Span::styled(format!(" {:.1}%", usage_pct), theme::highlight_style()),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  {:.1} / {:.1} GB used", mem.used_gb(), mem.total_gb()),
                theme::text_style(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  OS Reserved: ", theme::dim_style()),
            Span::styled("3.5G", theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled("  Available: ", theme::dim_style()),
            Span::styled(
                format!("{:.1}G", mem.available_gb()),
                theme::safe_style(),
            ),
        ]),
    ];

    let panel = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(" MEMORY RESERVES ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
    );

    f.render_widget(panel, area);
}

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};
use ratatui::layout::Alignment;

use super::theme;
use crate::app::DashboardTab;

pub fn draw(f: &mut Frame, active_tab: &DashboardTab, area: Rect) {
    let (local_style, catalog_style) = match active_tab {
        DashboardTab::Local => (theme::highlight_style(), theme::dim_style()),
        DashboardTab::Catalog => (theme::dim_style(), theme::highlight_style()),
    };

    let title = Line::from(vec![
        Span::styled("  ≈≈≈ ", theme::border_style()),
        Span::styled("M E L A N G E", theme::title_style()),
        Span::styled(" ≈≈≈  ", theme::border_style()),
        Span::styled("[Local]", local_style),
        Span::styled(" ", theme::dim_style()),
        Span::styled("[Catalog]", catalog_style),
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

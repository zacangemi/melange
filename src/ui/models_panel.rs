use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Row, Table, Cell};
use ratatui::text::Span;
use ratatui::layout::Constraint;

use super::theme;
use crate::app::App;
use crate::models::memory_calc::FitStatus;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from(Span::styled("  Model", theme::dim_style())),
        Cell::from(Span::styled("Params", theme::dim_style())),
        Cell::from(Span::styled("Type", theme::dim_style())),
        Cell::from(Span::styled("Quant", theme::dim_style())),
        Cell::from(Span::styled("Size", theme::dim_style())),
        Cell::from(Span::styled("Status", theme::dim_style())),
    ]);

    let rows: Vec<Row> = app
        .models
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let analysis = &app.analyses[i];
            let is_selected = i == app.selected_model;

            let marker = if is_selected { "▸ " } else { "  " };
            let name = format!("{}{}", marker, &model.name);

            let status_style = match analysis.status {
                FitStatus::Fits | FitStatus::Tight => theme::safe_style(),
                FitStatus::Limited => theme::highlight_style(),
                FitStatus::OOM => theme::danger_style(),
            };

            let row_style = if is_selected {
                theme::selected_style()
            } else {
                theme::text_style()
            };

            Row::new(vec![
                Cell::from(Span::styled(name, row_style)),
                Cell::from(Span::styled(format!("{:.1}B", model.params_billions()), row_style)),
                Cell::from(Span::styled(model.type_label().to_string(), row_style)),
                Cell::from(Span::styled(model.quant_label(), row_style)),
                Cell::from(Span::styled(format!("{:.1}G", model.size_gb()), row_style)),
                Cell::from(Span::styled(analysis.status.icon().to_string(), status_style)),
            ])
        })
        .collect();

    let title = format!(" LOCAL MODELS ({}) ", app.models.len());

    let table = Table::new(
        rows,
        [
            Constraint::Min(30),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(Span::styled(title, theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
    );

    f.render_widget(table, area);
}

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, Cell};
use ratatui::text::{Line, Span};

use super::theme;
use crate::app::App;
use crate::models::memory_calc::FitStatus;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let catalog_count = app.catalog_models.len() as u16;
    let table_height = catalog_count + 3; // rows + header + borders

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(table_height),
            Constraint::Min(10),
        ])
        .split(area);

    draw_table(f, app, chunks[0]);
    draw_detail(f, app, chunks[1]);
}

fn draw_table(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from(Span::styled("  Model", theme::dim_style())),
        Cell::from(Span::styled("Params", theme::dim_style())),
        Cell::from(Span::styled("Type", theme::dim_style())),
        Cell::from(Span::styled("Quant", theme::dim_style())),
        Cell::from(Span::styled("Size", theme::dim_style())),
        Cell::from(Span::styled("Status", theme::dim_style())),
    ]);

    let rows: Vec<Row> = app
        .catalog_models
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let analysis = &app.catalog_analyses[i];
            let is_selected = i == app.selected_catalog_model;

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

    let title = format!(" MODEL CATALOG ({}) ", app.catalog_models.len());

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

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    if app.catalog_models.is_empty() {
        return;
    }

    let model = &app.catalog_models[app.selected_catalog_model];
    let analysis = &app.catalog_analyses[app.selected_catalog_model];
    let total_ram = app.hardware.memory.total_bytes;

    let expert_info = if model.is_moe {
        format!(
            " ({} of {} experts active)",
            model.num_experts_per_tok, model.num_experts
        )
    } else {
        String::new()
    };

    let status_style = match analysis.status {
        FitStatus::Fits | FitStatus::Tight => theme::safe_style(),
        FitStatus::Limited => theme::highlight_style(),
        FitStatus::OOM => theme::danger_style(),
    };

    // Available = total RAM minus wired (non-reclaimable) memory
    let wired_gb = app.hardware.memory.wired_gb();
    let avail_gb = (total_ram as f64 / (1024.0 * 1024.0 * 1024.0)) - wired_gb;

    let horizon = if analysis.max_safe_context > 1024 {
        format!("{}K tokens", analysis.max_safe_context / 1024)
    } else {
        format!("{} tokens", analysis.max_safe_context)
    };

    let bar_width = (area.width as usize).saturating_sub(40).min(60).max(20);

    let weight_gb = model.size_gb();
    let kv_4k_gb = analysis.estimates.first().map(|e| e.kv_cache_gb()).unwrap_or(0.0);
    let overhead_gb = weight_gb * 0.10;

    let weight_bar = make_bar(bar_width, weight_gb, avail_gb);
    let kv_bar = make_bar(bar_width, kv_4k_gb, avail_gb);
    let overhead_bar = make_bar(bar_width, overhead_gb, avail_gb);

    let total_4k = analysis.estimates.first().map(|e| e.total_gb()).unwrap_or(0.0);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  Prefill:          ", theme::dim_style()),
        Span::styled(
            format!("~{:.0}-{:.0} tok/s", analysis.prefill_tok_s_low, analysis.prefill_tok_s_high),
            theme::highlight_style(),
        ),
        Span::styled(expert_info, theme::dim_style()),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Generation:       ", theme::dim_style()),
        Span::styled(
            format!("~{:.0}-{:.0} tok/s", analysis.tok_s_low, analysis.tok_s_high),
            theme::highlight_style(),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Horizon Limit:    ", theme::dim_style()),
        Span::styled(&horizon, theme::highlight_style()),
        Span::styled("  (max safe context before swap)", theme::dim_style()),
    ]));

    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  KV Cache Growth:", theme::title_style()),
    ]));

    for (i, est) in analysis.estimates.iter().take(6).enumerate() {
        let ctx_label = if est.context_length >= 1024 {
            format!("{}K", est.context_length / 1024)
        } else {
            format!("{}", est.context_length)
        };

        let est_total = est.total_gb();
        let fits = est_total <= avail_gb;
        let (indicator, ind_style) = if fits {
            ("  OK ", theme::safe_style())
        } else {
            (" SWAP", theme::danger_style())
        };

        let delta = if i > 0 {
            let prev_kv = analysis.estimates[i - 1].kv_cache_gb();
            format!("{:>+7.2}G", est.kv_cache_gb() - prev_kv)
        } else {
            "        ".to_string()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("    {:>6} ctx ", ctx_label), theme::dim_style()),
            Span::styled(format!("KV: {:>6.1} GB", est.kv_cache_gb()), theme::text_style()),
            Span::styled(format!(" {} ", delta), theme::dim_style()),
            Span::styled("│ ", theme::dim_style()),
            Span::styled(format!("Total: {:>6.1} GB", est_total), theme::text_style()),
            Span::styled(indicator, ind_style),
        ]));
    }

    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  Memory Breakdown:", theme::title_style()),
    ]));

    lines.push(Line::from(vec![
        Span::styled(format!("  Weights:  {: >5.1}G  ", weight_gb), theme::text_style()),
        Span::styled(format!("[{}]", weight_bar), theme::highlight_style()),
    ]));

    lines.push(Line::from(vec![
        Span::styled(format!("  KV @4K:   {: >5.1}G  ", kv_4k_gb), theme::text_style()),
        Span::styled(format!("[{}]", kv_bar), theme::safe_style()),
    ]));

    lines.push(Line::from(vec![
        Span::styled(format!("  Overhead: {: >5.1}G  ", overhead_gb), theme::text_style()),
        Span::styled(format!("[{}]", overhead_bar), theme::dim_style()),
    ]));

    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            format!("  Total @ 4K ctx: {:.1}G / {:.1}G available  ──  ", total_4k, avail_gb),
            theme::text_style(),
        ),
        Span::styled(analysis.status.label(), status_style),
    ]));

    let title = format!(" ▸ {}  ", model.name);
    let panel = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(title, theme::highlight_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
    );

    f.render_widget(panel, area);
}

fn make_bar(width: usize, value_gb: f64, max_gb: f64) -> String {
    let fraction = (value_gb / max_gb).min(1.0);
    let filled = ((fraction * width as f64).round() as usize).max(1);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

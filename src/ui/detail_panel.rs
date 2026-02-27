use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

use super::theme;
use crate::app::App;
use crate::models::memory_calc::SpiceStatus;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if app.models.is_empty() {
        let empty = Paragraph::new("  No models found.")
            .style(theme::dim_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme::border_style())
                    .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
            );
        f.render_widget(empty, area);
        return;
    }

    let model = &app.models[app.selected_model];
    let analysis = &app.analyses[app.selected_model];
    let total_ram = app.hardware.memory.total_bytes;

    // Expert info
    let expert_info = if model.is_moe {
        format!(
            " ({} of {} experts active)",
            model.num_experts_per_tok, model.num_experts
        )
    } else {
        String::new()
    };

    // Status style
    let status_style = match analysis.status {
        SpiceStatus::AbundantSpice | SpiceStatus::SpiceThinning => theme::safe_style(),
        SpiceStatus::SpiceScarcity => theme::highlight_style(),
        SpiceStatus::DesertDrought => theme::danger_style(),
    };

    // Available RAM for models (total - OS reserved)
    let avail_gb = (total_ram as f64 / (1024.0 * 1024.0 * 1024.0)) - 3.5;

    // Build KV cache summary from estimates
    let kv_summary = analysis
        .estimates
        .iter()
        .take(4) // Show first 4 context lengths
        .map(|e| {
            let ctx_label = if e.context_length >= 1024 {
                format!("{}K", e.context_length / 1024)
            } else {
                format!("{}", e.context_length)
            };
            format!("{}: {:.1} GB", ctx_label, e.kv_cache_gb())
        })
        .collect::<Vec<_>>()
        .join(" │ ");

    // Memory breakdown bars
    let bar_total = area.width.saturating_sub(24) as f64;
    let weight_frac = model.size_gb() / avail_gb;
    let kv_4k_frac = if let Some(e) = analysis.estimates.first() {
        e.kv_cache_gb() / avail_gb
    } else {
        0.0
    };
    let overhead_frac = (model.size_gb() * 0.10) / avail_gb;

    let weight_bar = make_bar(bar_total, weight_frac);
    let kv_bar = make_bar(bar_total, kv_4k_frac);
    let overhead_bar = make_bar(bar_total, overhead_frac);

    // Total at 4K
    let total_4k = if let Some(e) = analysis.estimates.first() {
        e.total_gb()
    } else {
        0.0
    };

    // Horizon limit display
    let horizon = if analysis.max_safe_context > 1024 {
        format!("{}K tokens", analysis.max_safe_context / 1024)
    } else {
        format!("{} tokens", analysis.max_safe_context)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!("  ▸ {}  (Selected)", model.name),
                theme::highlight_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(
                    "  Spice Flow Rate: ~{:.0}-{:.0} tok/s{}",
                    analysis.tok_s_low, analysis.tok_s_high, expert_info
                ),
                theme::text_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  Horizon Limit: {} (max safe context)", horizon),
                theme::text_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(format!("  KV Cache @ {}", kv_summary), theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled(format!("  [{}]", weight_bar), theme::highlight_style()),
            Span::styled(format!(" Weights: {:.1}G", model.size_gb()), theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled(format!("  [{}]", kv_bar), theme::safe_style()),
            Span::styled(
                format!(
                    " KV@4K:   {:.1}G",
                    analysis.estimates.first().map(|e| e.kv_cache_gb()).unwrap_or(0.0)
                ),
                theme::text_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(format!("  [{}]", overhead_bar), theme::dim_style()),
            Span::styled(format!(" Overhead: {:.1}G", model.size_gb() * 0.10), theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  Total @ 4K ctx: {:.1}G / {:.1}G avail ── ", total_4k, avail_gb),
                theme::text_style(),
            ),
            Span::styled(analysis.status.label(), status_style),
        ]),
    ];

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

fn make_bar(total_width: f64, fraction: f64) -> String {
    let filled = ((fraction * total_width).round() as usize).max(1);
    let empty = (total_width as usize).saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

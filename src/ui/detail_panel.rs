use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

use super::theme;
use crate::app::App;
use crate::compat::warnings::WarningSeverity;
use crate::models::memory_calc::FitStatus;

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
        FitStatus::Fits | FitStatus::Tight => theme::safe_style(),
        FitStatus::Limited => theme::highlight_style(),
        FitStatus::OOM => theme::danger_style(),
    };

    // Available RAM for models (total - OS reserved)
    // Available = total RAM minus wired (non-reclaimable) memory
    let wired_gb = app.hardware.memory.wired_gb();
    let avail_gb = (total_ram as f64 / (1024.0 * 1024.0 * 1024.0)) - wired_gb;

    // Horizon limit display
    let horizon = if analysis.max_safe_context > 1024 {
        format!("{}K tokens", analysis.max_safe_context / 1024)
    } else {
        format!("{} tokens", analysis.max_safe_context)
    };

    // Memory bar width (leave room for labels)
    let bar_width = (area.width as usize).saturating_sub(40).min(60).max(20);

    let weight_gb = model.size_gb();
    let kv_4k_gb = analysis.estimates.first().map(|e| e.kv_cache_gb()).unwrap_or(0.0);
    let overhead_gb = weight_gb * 0.10;

    let weight_bar = make_labeled_bar(bar_width, weight_gb, avail_gb);
    let kv_bar = make_labeled_bar(bar_width, kv_4k_gb, avail_gb);
    let overhead_bar = make_labeled_bar(bar_width, overhead_gb, avail_gb);

    // Total at 4K
    let total_4k = analysis.estimates.first().map(|e| e.total_gb()).unwrap_or(0.0);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));

    // Speed metrics
    lines.push(Line::from(vec![
        Span::styled("  Prefill:       ", theme::dim_style()),
        Span::styled(
            format!("~{:.0}-{:.0} tok/s", analysis.prefill_tok_s_low, analysis.prefill_tok_s_high),
            theme::highlight_style(),
        ),
        Span::styled(&expert_info, theme::dim_style()),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Generation:    ", theme::dim_style()),
        Span::styled(
            format!("~{:.0}-{:.0} tok/s", analysis.tok_s_low, analysis.tok_s_high),
            theme::highlight_style(),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Horizon Limit: ", theme::dim_style()),
        Span::styled(&horizon, theme::highlight_style()),
        Span::styled("  (max safe context before swap)", theme::dim_style()),
    ]));

    lines.push(Line::from(""));

    // KV Cache growth table
    lines.push(Line::from(vec![
        Span::styled("  KV Cache Growth:", theme::title_style()),
    ]));

    // Build context length rows with delta
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

    // Memory breakdown bars
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

    // Final verdict
    lines.push(Line::from(vec![
        Span::styled(
            format!("  Total @ 4K ctx: {:.1}G / {:.1}G available  ──  ", total_4k, avail_gb),
            theme::text_style(),
        ),
        Span::styled(analysis.status.label(), status_style),
    ]));

    // Engine warnings (from compat KB)
    let warnings = &app.warnings[app.selected_model];
    if !warnings.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("  Engine Warnings ({}):", warnings.len()),
                theme::title_style(),
            ),
            Span::styled("  press 'w' for detail", theme::dim_style()),
        ]));

        let max_inline = 3;
        for w in warnings.iter().take(max_inline) {
            let severity_style = match w.severity {
                WarningSeverity::Info => theme::dim_style(),
                WarningSeverity::Caution => theme::highlight_style(),
                WarningSeverity::Breaking => theme::highlight_style(),
            };

            lines.push(Line::from(vec![
                Span::styled(format!("    {} ", w.severity.icon()), severity_style),
                Span::styled(format!("[{}] ", w.engine), theme::dim_style()),
                Span::styled(&w.summary, severity_style),
            ]));
        }

        if warnings.len() > max_inline {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    ... and {} more (press 'w')", warnings.len() - max_inline),
                    theme::dim_style(),
                ),
            ]));
        }
    }

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

fn make_labeled_bar(width: usize, value_gb: f64, max_gb: f64) -> String {
    let fraction = (value_gb / max_gb).min(1.0);
    let filled = ((fraction * width as f64).round() as usize).max(1);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

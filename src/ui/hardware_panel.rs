use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

use super::theme;
use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let hw = &app.hardware;

    let mem_type = if hw.memory.is_unified { "Unified" } else { "Discrete" };

    // Pad all labels to same width so values align (longest: "Bandwidth" = 9 chars)
    let lines = vec![
        Line::from(vec![
            Span::styled("  OS:        ", theme::dim_style()),
            Span::styled(&hw.os.display, theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled("  CPU:       ", theme::dim_style()),
            Span::styled(&hw.cpu.brand, theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled("  Cores:     ", theme::dim_style()),
            Span::styled(
                format!("{}P + {}E ({} total)", hw.cpu.performance_cores, hw.cpu.efficiency_cores, hw.cpu.total_cores),
                theme::text_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  GPU:       ", theme::dim_style()),
            Span::styled(
                format!("{} Metal cores", hw.gpu.metal_cores),
                theme::text_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Memory:    ", theme::dim_style()),
            Span::styled(
                format!("{:.0} GB {}", hw.memory.total_gb(), mem_type),
                theme::text_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Bandwidth: ", theme::dim_style()),
            Span::styled(
                format!("{:.0} GB/s", hw.bandwidth_gbs),
                theme::text_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Disk:      ", theme::dim_style()),
            Span::styled(
                format!("{:.0} GB free", hw.disk.free_gb()),
                theme::text_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Engines:   ", theme::dim_style()),
            Span::styled(hw.engines.display(), theme::text_style()),
        ]),
        Line::from({
            let vpn_text = hw.vpn.display(app.vpn_visible);
            let vpn_style = if hw.vpn.active { theme::safe_style() } else { theme::dim_style() };
            let mut spans = vec![
                Span::styled("  VPN:       ", theme::dim_style()),
                Span::styled(vpn_text, vpn_style),
            ];
            if hw.vpn.active && !app.vpn_visible {
                spans.push(Span::styled("  [v] reveal", theme::dim_style()));
            } else if hw.vpn.active {
                spans.push(Span::styled("  [v] hide", theme::dim_style()));
            }
            spans
        }),
    ];

    let panel = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(" SYSTEM PROFILE ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
    );

    f.render_widget(panel, area);
}

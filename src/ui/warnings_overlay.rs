use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::text::{Line, Span};

use super::theme;
use crate::compat::warnings::{CompatWarning, WarningSeverity};

pub fn draw(f: &mut Frame, model_name: &str, warnings: &[CompatWarning]) {
    let popup_width = (f.area().width.saturating_sub(4)).min(72);
    let inner_width = (popup_width as usize).saturating_sub(4); // border + padding

    // Count unique engines
    let mut engine_names: Vec<&str> = warnings.iter().map(|w| w.engine.as_str()).collect();
    engine_names.sort();
    engine_names.dedup();
    let engine_count = engine_names.len();

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for (i, w) in warnings.iter().enumerate() {
        // Severity label + engine
        let (severity_label, severity_style) = match w.severity {
            WarningSeverity::Breaking => ("BREAKING", theme::highlight_style()),
            WarningSeverity::Caution => ("CAUTION", theme::highlight_style()),
            WarningSeverity::Info => ("INFO", theme::dim_style()),
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", w.severity.icon()), severity_style),
            Span::styled(format!("{:<8}", severity_label), severity_style),
            Span::styled(format!(" [{}]", w.engine), theme::dim_style()),
        ]));

        // Summary (word-wrapped with 4-space indent)
        let summary_indent = 4;
        let summary_width = inner_width.saturating_sub(summary_indent);
        for line in word_wrap(&w.summary, summary_width) {
            lines.push(Line::from(vec![
                Span::styled(format!("{}{}", " ".repeat(summary_indent), line), theme::text_style()),
            ]));
        }

        // Detail (word-wrapped, dim, 6-space indent)
        if let Some(ref detail) = w.detail {
            let detail_indent = 6;
            let detail_width = inner_width.saturating_sub(detail_indent);
            for line in word_wrap(detail, detail_width) {
                lines.push(Line::from(vec![
                    Span::styled(format!("{}{}", " ".repeat(detail_indent), line), theme::dim_style()),
                ]));
            }
        }

        // Workaround (word-wrapped, 6-space indent for continuation)
        if let Some(ref workaround) = w.workaround {
            let wa_indent = 6;
            let wa_width = inner_width.saturating_sub(wa_indent);
            let wa_lines = word_wrap(workaround, wa_width);
            for (j, line) in wa_lines.iter().enumerate() {
                if j == 0 {
                    lines.push(Line::from(vec![
                        Span::styled("    Workaround: ", theme::highlight_style()),
                        Span::styled(line.clone(), theme::text_style()),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{}{}", " ".repeat(wa_indent), line), theme::text_style()),
                    ]));
                }
            }
        }

        // Fixed in
        if let Some(ref fixed_in) = w.fixed_in {
            lines.push(Line::from(vec![
                Span::styled(format!("    Fixed in: {}", fixed_in), theme::dim_style()),
            ]));
        }

        // References
        if let Some(ref refs) = w.references {
            for r in refs {
                lines.push(Line::from(vec![
                    Span::styled(format!("    Ref: {}", r), theme::dim_style()),
                ]));
            }
        }

        // Separator between warnings
        if i + 1 < warnings.len() {
            lines.push(Line::from(""));
            let sep = "─".repeat(inner_width.saturating_sub(2));
            lines.push(Line::from(vec![
                Span::styled(format!("  {}", sep), theme::border_style()),
            ]));
        }
    }

    // Footer
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!(
                "  {} warning{} for {} engine{}. Uninstalled engines hidden.",
                warnings.len(),
                if warnings.len() == 1 { "" } else { "s" },
                engine_count,
                if engine_count == 1 { "" } else { "s" },
            ),
            theme::dim_style(),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Press any key to close", theme::dim_style()),
    ]));

    // Dynamic height based on actual line count
    let popup_height = ((lines.len() as u16) + 2).min(f.area().height.saturating_sub(4)); // +2 for borders
    let area = centered_rect(popup_width, popup_height, f.area());

    f.render_widget(Clear, area);

    let title = format!(" ENGINE WARNINGS — {} ", model_name);
    let popup = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(title, theme::title_style()))
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
        );

    f.render_widget(popup, area);
}

/// Word-wrap a string to fit within `max_width` characters.
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }
    let mut result = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            result.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        result.push(current_line);
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
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

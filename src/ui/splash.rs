use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::text::{Line, Span};

use super::theme;

// Arrakeen mansion mural — Shai-Hulud
// Circular mouth with radiating lines on left, thick segmented S-curve body,
// body loops upward with spiral curl, tiny Fremen figures along the bottom dunes.
const WORM_BODY: &[&str] = &[
    r"                          .  '  .                                                       ",
    r"            \  ' .  /    ' .  . '   .  '                                        ___     ",
    r"         --  \  |  / --      '                                                 / _ \    ",
    r"        '  ---\\|//---  .        .                                             | / \ |   ",
    r"          -====(●)====-                        .          .                    |  _  |   ",
    r"        .  ---/|\\---  '     .          ___---~~~~---___                        \ \_/ |  ",
    r"         --  /  |  \ --          ___--~~  //||||\\\\  ~~--___                    \___/   ",
    r"            /  ' .  \     ___--~~   ////  ||||||||  \\\\   ~~--___                 |    ",
    r"          '    .    ___--~~ //////  ||||  ||||||||  ||||  \\\\\\ ~~--___          /     ",
    r"           ___..--~~ //// ||||||||  ||||   ||||||   ||||  |||||||| \\\\ ~~--..__/      ",
    r"     __--~~  //////  |||| ||||||||  ||||    ||||    ||||  |||||||| ||||  \\\\\\  ~~--   ",
    r"   ~~  ////  ||||||  |||| ||||||||  ||||    ||||    ||||  |||||||| ||||  ||||||  \\\\   ",
    r"    \\\\  ||||  ||||  ||||  ||||||  ||||    ||||    ||||  ||||||  ||||  ||||  ////      ",
    r"      \\\\  ||||  ||||  \\\\  ||||  ||||    ||||    ||||  ||||  ////  ||||  ////        ",
    r"  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~",
    r"  ~~.~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~  ~~   ",
    r"      .  i i .    i i .    i i .    i i      i i .    i i .    i i .    i i .             ",
];

// Lines that are sand/dunes (rendered in DEEP_DESERT)
const DUNE_START: usize = 14;

pub fn draw(f: &mut Frame) {
    let area = f.area();
    f.render_widget(Clear, area);

    let mut lines: Vec<Line> = Vec::new();

    // Vertical centering
    let art_height: u16 = WORM_BODY.len() as u16 + 4;
    let pad_top = area.height.saturating_sub(art_height) / 2;
    for _ in 0..pad_top {
        lines.push(Line::from(""));
    }

    // Horizontal centering
    let max_art_width = WORM_BODY.iter().map(|l| l.len()).max().unwrap_or(0);
    let h_pad = (area.width as usize).saturating_sub(max_art_width) / 2;
    let pad_str = " ".repeat(h_pad);

    // Render with dual coloring: worm in SPICE_ORANGE, dunes in DEEP_DESERT
    for (i, line) in WORM_BODY.iter().enumerate() {
        let style = if i >= DUNE_START {
            theme::dim_style()
        } else {
            theme::highlight_style()
        };
        lines.push(Line::from(Span::styled(
            format!("{}{}", pad_str, line),
            style,
        )));
    }

    // Title + subtitle, manually centered
    let title = "M E L A N G E";
    let subtitle = "\"The memory must flow...\"";
    let title_pad = (area.width as usize).saturating_sub(title.len()) / 2;
    let sub_pad = (area.width as usize).saturating_sub(subtitle.len()) / 2;

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("{}{}", " ".repeat(title_pad), title),
        theme::title_style(),
    )));
    lines.push(Line::from(Span::styled(
        format!("{}{}", " ".repeat(sub_pad), subtitle),
        theme::dim_style(),
    )));

    let splash = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(ratatui::style::Style::default().bg(theme::NIGHT_SKY)),
        );

    f.render_widget(splash, area);
}

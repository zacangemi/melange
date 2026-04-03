use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::text::{Line, Span};

use super::theme;

// Shai-Hulud's maw — head-on, rising from the sands of Arrakis.
// Inspired by the Dune (2021) sandworm emergence scene.
// Radial teeth converge to dark throat center, thick body ring,
// sand dunes flank the base, two tiny Fremen figures stand below.
const WORM_BODY: &[&str] = &[
    r"~~~~.~~~~  ~~~~.~~~~  ~~~~                                     ~~~~  ~~~~.~~~~  ~~~~.~~~~",
    r"     ~~~~.~~~~  ~~~~                                                 ~~~~  ~~~~.~~~~",
    r"          ~~~~                                                             ~~~~",
    r"               ~~~~                @@@@@@@@@@@@@@                ~~~~",
    r"                              @@@@@@@@@@@@@@@@@@@@@@@@",
    r"                           @@@@@@@ \ \ \ \ | / / / / @@@@@@@",
    r"                         @@@@@@ \ \ \ \ \ \ | / / / / / / @@@@@@",
    r"                       @@@@@ \ \ \ \ \ \ \ \ | / / / / / / / / @@@@@",
    r"                      @@@@ \ \ \ \ \ \ \ \ \ | / / / / / / / / / @@@@",
    r"                     @@@@ \ \ \ \ \ \ \ \ \\ | // / / / / / / / / @@@@",
    r"                    @@@@ - \ \ \ \ \ \ \\\\ | //// / / / / / / - @@@@",
    r"                   @@@@ - - \ \ \ \ \\\\\\  |  ////// / / / / - - @@@@",
    r"                   @@@@ - - - \ \ \\\\\\\\     //////// / / - - - @@@@",
    r"                  @@@@ - - - - \ \\\\\\\\\\  .  ////////// / - - - - @@@@",
    r"                  @@@@ - - - - - \\\\\\\\\\  .  ////////// - - - - - @@@@",
    r"                  @@@@ - - - - - - - \\\\\  . .  ///// - - - - - - - @@@@",
    r"                  @@@@ - - - - - - - - - -  . .  - - - - - - - - - - @@@@",
    r"                  @@@@ - - - - - - - /////  . .  \\\\\ - - - - - - - @@@@",
    r"                  @@@@ - - - - - //////////  .  \\\\\\\\\\ - - - - - @@@@",
    r"                  @@@@ - - - - / //////////  .  \\\\\\\\\\ \ - - - - @@@@",
    r"                   @@@@ - - - / / ////////     \\\\\\\\ \ \ - - - @@@@",
    r"  ~~                @@@@ - - / / / / //////  |  \\\\\\ \ \ \ \ - - @@@@                ~~",
    r"  ~~~~               @@@@ - / / / / / / //// | \\\\ \ \ \ \ \ \ - @@@@               ~~~~",
    r"  ~~~~.~~              @@@@ / / / / / / / / // | \\ \ \ \ \ \ \ \ \ @@@@              ~~.~~~~",
    r"  ~~~~.~~~~              @@@@ / / / / / / / / / | \ \ \ \ \ \ \ \ \ @@@@              ~~~~.~~~~",
    r"  ~~~~.~~~~  ~~            @@@@@ / / / / / / / / | \ \ \ \ \ \ \ \ @@@@@            ~~  ~~~~.~~~~",
    r"  ~~~~.~~~~  ~~~~.~~          @@@@@@ / / / / / / | \ \ \ \ \ \ @@@@@@          ~~.~~~~  ~~~~.~~~~",
    r"  ~~~~.~~~~  ~~~~.~~~~  ~~       @@@@@@@@@@@@@@@@@@@@@@@@@@@@@       ~~  ~~~~.~~~~  ~~~~.~~~~",
    r"  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~   @@@@@@@@@@@@@@@@@@@@@   ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~",
    r"  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~",
    r"  ~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~",
    r"      .  i i .    i i .    i i .    i i      i i .    i i .    i i .    i i .    i i .",
    r"",
    r"                                   o         o",
    r"                                  /|\       /|\",
    r"                                  / \       / \",
];

// Coloring zones
const WORM_START: usize = 4;   // Worm body begins (SPICE_ORANGE)
const SAND_START: usize = 29;  // Flat sand begins (DEEP_DESERT)
const FREMEN_START: usize = 33; // Fremen figures (FREMEN_BLUE)

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

    // Four-zone coloring:
    //   Background dunes (0..WORM_START)     → DEEP_DESERT
    //   Worm body+teeth  (WORM_START..SAND)  → SPICE_ORANGE
    //   Flat sand         (SAND..FREMEN)      → DEEP_DESERT
    //   Fremen figures    (FREMEN..)          → FREMEN_BLUE
    for (i, line) in WORM_BODY.iter().enumerate() {
        let style = if i >= FREMEN_START {
            theme::safe_style()
        } else if i >= SAND_START {
            theme::dim_style()
        } else if i >= WORM_START {
            theme::highlight_style()
        } else {
            theme::dim_style()
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

pub mod theme;
pub mod header;
pub mod hardware_panel;
pub mod memory_panel;
pub mod models_panel;
pub mod detail_panel;
pub mod footer;
pub mod splash;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Main vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(10),     // Body
            Constraint::Length(3),   // Footer
        ])
        .split(size);

    header::draw(f, chunks[0]);
    draw_body(f, app, chunks[1]);
    footer::draw(f, app, chunks[2]);
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    // Body: top section (hardware+memory) and bottom section (models+detail)
    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),   // Hardware + Memory row
            Constraint::Min(6),      // Models table + detail
        ])
        .split(area);

    // Top row: Memory gauge | System profile
    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(body[0]);

    memory_panel::draw(f, app, top_row[0]);
    hardware_panel::draw(f, app, top_row[1]);

    // Bottom section: compact model table + expanded detail panel
    let model_count = app.models.len() as u16;
    let table_height = model_count + 3; // rows + header + borders

    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(table_height),  // Model table (compact)
            Constraint::Min(10),               // Detail panel (fills rest)
        ])
        .split(body[1]);

    models_panel::draw(f, app, bottom[0]);
    detail_panel::draw(f, app, bottom[1]);
}

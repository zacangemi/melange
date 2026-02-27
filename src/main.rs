mod app;
mod hardware;
mod models;
mod ui;
mod dune;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use serde::Serialize;

use app::{App, AppState};
use hardware::HardwareInfo;
use models::memory_calc;

/// Melange — "The memory must flow"
///
/// A Dune-themed terminal tool that scans your hardware and local model files
/// to tell you exactly what fits, how fast it'll run, and when you'll hit swap.
#[derive(Parser, Debug)]
#[command(name = "melange", version = "0.1.0")]
#[command(about = "The memory must flow — local model memory analyzer for Apple Silicon")]
struct Cli {
    /// Path to model directory (default: ~/AI_MODELS/models/)
    #[arg(long = "scan", value_name = "PATH")]
    model_dir: Option<PathBuf>,

    /// Output as JSON instead of launching TUI
    #[arg(long)]
    json: bool,
}

#[derive(Serialize)]
struct JsonOutput {
    hardware: HardwareInfo,
    models: Vec<JsonModelEntry>,
}

#[derive(Serialize)]
struct JsonModelEntry {
    name: String,
    model_type: String,
    params_billions: f64,
    quant_bits: u32,
    size_gb: f64,
    is_moe: bool,
    num_experts: u32,
    num_experts_per_tok: u32,
    analysis: JsonAnalysis,
}

#[derive(Serialize)]
struct JsonAnalysis {
    status: String,
    tok_s_range: [f64; 2],
    max_safe_context: u64,
    kv_per_token_bytes: u64,
    headroom_gb: f64,
    estimates: Vec<JsonEstimate>,
}

#[derive(Serialize)]
struct JsonEstimate {
    context_length: u64,
    weight_gb: f64,
    kv_cache_gb: f64,
    overhead_gb: f64,
    total_gb: f64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let default_model_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AI_MODELS/models");
    let model_dir = cli.model_dir.unwrap_or(default_model_dir);

    // Detect hardware
    let hardware = HardwareInfo::detect()?;

    // Scan models
    let models_result = models::scanner::scan_directory(&model_dir);
    let found_models = match models_result {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: Could not scan models at {}: {}", model_dir.display(), e);
            Vec::new()
        }
    };

    if cli.json {
        return output_json(&hardware, &found_models);
    }

    // Launch TUI
    run_tui(hardware, found_models, model_dir)
}

fn output_json(hardware: &HardwareInfo, models: &[models::ModelInfo]) -> Result<()> {
    let entries: Vec<JsonModelEntry> = models
        .iter()
        .map(|m| {
            let analysis = memory_calc::analyze(m, hardware.memory.total_bytes, hardware.bandwidth_gbs);

            JsonModelEntry {
                name: m.name.clone(),
                model_type: m.model_type.clone(),
                params_billions: m.params_billions(),
                quant_bits: m.quant_bits,
                size_gb: m.size_gb(),
                is_moe: m.is_moe,
                num_experts: m.num_experts,
                num_experts_per_tok: m.num_experts_per_tok,
                analysis: JsonAnalysis {
                    status: analysis.status.label().to_string(),
                    tok_s_range: [analysis.tok_s_low, analysis.tok_s_high],
                    max_safe_context: analysis.max_safe_context,
                    kv_per_token_bytes: analysis.kv_per_token_bytes,
                    headroom_gb: analysis.headroom_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                    estimates: analysis
                        .estimates
                        .iter()
                        .map(|e| JsonEstimate {
                            context_length: e.context_length,
                            weight_gb: e.weight_gb(),
                            kv_cache_gb: e.kv_cache_gb(),
                            overhead_gb: e.overhead_gb(),
                            total_gb: e.total_gb(),
                        })
                        .collect(),
                },
            }
        })
        .collect();

    let output = JsonOutput {
        hardware: hardware.clone(),
        models: entries,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn run_tui(hardware: HardwareInfo, models: Vec<models::ModelInfo>, model_dir: PathBuf) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(hardware, models, model_dir);

    // Main event loop
    loop {
        terminal.draw(|f| {
            match app.state {
                AppState::Splash => ui::splash::draw(f),
                AppState::Dashboard => ui::draw(f, &app),
            }
        })?;

        // Poll for events with a short timeout for tick updates
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.state == AppState::Splash {
                        // Any key skips splash
                        app.state = AppState::Dashboard;
                    } else {
                        app.on_key(key.code);
                    }
                }
            }
        }

        app.tick();

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

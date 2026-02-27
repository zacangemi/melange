use std::path::PathBuf;
use std::time::Instant;

use crate::hardware::HardwareInfo;
use crate::models::ModelInfo;
use crate::models::memory_calc::{self, ModelAnalysis};
use crate::dune::quotes;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Splash,
    Dashboard,
}

pub struct App {
    pub state: AppState,
    pub hardware: HardwareInfo,
    pub models: Vec<ModelInfo>,
    pub analyses: Vec<ModelAnalysis>,
    pub model_dir: PathBuf,
    pub selected_model: usize,
    pub quote_index: usize,
    pub splash_start: Instant,
    pub last_quote_change: Instant,
    pub should_quit: bool,
}

impl App {
    pub fn new(hardware: HardwareInfo, models: Vec<ModelInfo>, model_dir: PathBuf) -> Self {
        let analyses: Vec<ModelAnalysis> = models
            .iter()
            .map(|m| memory_calc::analyze(m, hardware.memory.total_bytes, hardware.bandwidth_gbs))
            .collect();

        let now = Instant::now();

        App {
            state: AppState::Splash,
            hardware,
            models,
            analyses,
            model_dir,
            selected_model: 0,
            quote_index: 0,
            splash_start: now,
            last_quote_change: now,
            should_quit: false,
        }
    }

    pub fn tick(&mut self) {
        // Transition from splash after 1.5s
        if self.state == AppState::Splash && self.splash_start.elapsed().as_millis() > 1500 {
            self.state = AppState::Dashboard;
        }

        // Rotate quotes every 10s
        if self.last_quote_change.elapsed().as_secs() >= 10 {
            self.quote_index = (self.quote_index + 1) % quotes::count();
            self.last_quote_change = Instant::now();
        }
    }

    pub fn on_key(&mut self, key: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.models.is_empty() {
                    self.selected_model = (self.selected_model + 1) % self.models.len();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if !self.models.is_empty() {
                    self.selected_model = if self.selected_model == 0 {
                        self.models.len() - 1
                    } else {
                        self.selected_model - 1
                    };
                }
            }
            KeyCode::Char('r') => {
                self.refresh();
            }
            _ => {}
        }
    }

    pub fn refresh(&mut self) {
        // Re-detect hardware
        if let Ok(hw) = HardwareInfo::detect() {
            self.hardware = hw;
        }
        // Re-scan models
        if let Ok(models) = crate::models::scanner::scan_directory(&self.model_dir) {
            self.analyses = models
                .iter()
                .map(|m| {
                    memory_calc::analyze(m, self.hardware.memory.total_bytes, self.hardware.bandwidth_gbs)
                })
                .collect();
            self.models = models;
            if self.selected_model >= self.models.len() {
                self.selected_model = 0;
            }
        }
    }
}

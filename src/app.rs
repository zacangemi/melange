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

#[derive(Debug, Clone, PartialEq)]
pub enum DashboardTab {
    Local,
    Catalog,
}

pub struct App {
    pub state: AppState,
    pub active_tab: DashboardTab,
    pub hardware: HardwareInfo,
    pub models: Vec<ModelInfo>,
    pub analyses: Vec<ModelAnalysis>,
    pub model_dirs: Vec<PathBuf>,
    pub selected_model: usize,
    pub quote_index: usize,
    pub splash_start: Instant,
    pub last_quote_change: Instant,
    pub should_quit: bool,
    pub vpn_visible: bool,
    pub compat_db: crate::compat::warnings::CompatDb,
    pub warnings: Vec<Vec<crate::compat::warnings::CompatWarning>>,
}

impl App {
    pub fn new(hardware: HardwareInfo, models: Vec<ModelInfo>, model_dirs: Vec<PathBuf>) -> Self {
        let analyses: Vec<ModelAnalysis> = models
            .iter()
            .map(|m| memory_calc::analyze(m, hardware.memory.total_bytes, hardware.bandwidth_gbs))
            .collect();

        let compat_db = crate::compat::warnings::load_compat_db();
        let warnings: Vec<Vec<crate::compat::warnings::CompatWarning>> = models
            .iter()
            .map(|m| {
                crate::compat::warnings::find_warnings(&compat_db, m, &hardware.engines)
                    .into_iter()
                    .cloned()
                    .collect()
            })
            .collect();

        let now = Instant::now();

        App {
            state: AppState::Splash,
            active_tab: DashboardTab::Local,
            hardware,
            models,
            analyses,
            model_dirs,
            selected_model: 0,
            quote_index: 0,
            splash_start: now,
            last_quote_change: now,
            should_quit: false,
            vpn_visible: false,
            compat_db,
            warnings,
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
            KeyCode::Tab => {
                self.active_tab = match self.active_tab {
                    DashboardTab::Local => DashboardTab::Catalog,
                    DashboardTab::Catalog => DashboardTab::Local,
                };
            }
            KeyCode::Char('r') => {
                self.refresh();
            }
            KeyCode::Char('v') => {
                self.vpn_visible = !self.vpn_visible;
            }
            _ => {}
        }
    }

    pub fn refresh(&mut self) {
        // Re-detect hardware
        if let Ok(hw) = HardwareInfo::detect() {
            self.hardware = hw;
        }
        // Re-scan all model directories
        let models = crate::models::scanner::scan_directories(&self.model_dirs);
        self.analyses = models
            .iter()
            .map(|m| {
                memory_calc::analyze(m, self.hardware.memory.total_bytes, self.hardware.bandwidth_gbs)
            })
            .collect();
        // Recompute compatibility warnings
        self.compat_db = crate::compat::warnings::load_compat_db();
        self.warnings = models
            .iter()
            .map(|m| {
                crate::compat::warnings::find_warnings(&self.compat_db, m, &self.hardware.engines)
                    .into_iter()
                    .cloned()
                    .collect()
            })
            .collect();
        self.models = models;
        if self.selected_model >= self.models.len() {
            self.selected_model = 0;
        }
    }
}

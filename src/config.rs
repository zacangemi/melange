use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::models;

#[derive(Debug, Serialize, Deserialize)]
pub struct MelangeConfig {
    #[serde(default)]
    pub model_dirs: Vec<String>,
    /// Optional: lock VPN detection to a specific provider.
    /// Values: "tailscale", "zerotier", "nebula", "wireguard"
    /// If unset, auto-detects in priority order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vpn: Option<String>,
}

/// Returns `~/.config/melange/config.toml`
pub fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .context("Could not determine home directory")?;
    Ok(home.join(".config").join("melange").join("config.toml"))
}

/// Load config from disk. Returns `Ok(None)` if file doesn't exist.
pub fn load_config() -> Result<Option<MelangeConfig>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config at {}", path.display()))?;
    let cfg: MelangeConfig = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config at {}", path.display()))?;
    Ok(Some(cfg))
}

/// Load config or return empty default.
pub fn load_config_or_default() -> Result<MelangeConfig> {
    Ok(load_config()?.unwrap_or(MelangeConfig { model_dirs: vec![], vpn: None }))
}

/// Save config to disk, creating parent dirs as needed.
pub fn save_config(cfg: &MelangeConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory {}", parent.display()))?;
    }
    let toml_str = toml::to_string_pretty(cfg)
        .context("Failed to serialize config")?;
    let contents = format!(
        "# Melange configuration\n\
         # Manage directories with: melange add, melange dirs, melange remove\n\n\
         {}",
        toml_str
    );
    std::fs::write(&path, contents)
        .with_context(|| format!("Failed to write config to {}", path.display()))?;
    Ok(())
}

/// Default model directory: ~/AI_MODELS/models
pub fn default_model_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AI_MODELS/models")
}

/// Expand `~/` prefix to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest)
    } else {
        PathBuf::from(path)
    }
}

/// Prompt the user for a model directory path. Returns the validated PathBuf.
fn prompt_model_dir(prompt_msg: &str) -> Result<PathBuf> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("{}", prompt_msg);
        stdout.flush()?;

        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let input = line.trim();

        if input.is_empty() {
            println!("  Please enter a path.");
            continue;
        }

        let path = expand_tilde(input);

        if !path.exists() {
            println!("  Directory does not exist: {}", path.display());
            println!("  Please enter a valid path.\n");
            continue;
        }

        if !path.is_dir() {
            println!("  Not a directory: {}", path.display());
            println!("  Please enter a valid directory path.\n");
            continue;
        }

        return Ok(path);
    }
}

/// Scan a directory and print what was found.
fn scan_and_report(path: &Path) -> usize {
    match models::scanner::scan_directory(path) {
        Ok(found) => {
            if found.is_empty() {
                println!("  No models found in {}", path.display());
                println!("  (Added anyway — models can be downloaded later.)");
            } else {
                println!("  Found {} model{}:", found.len(), if found.len() == 1 { "" } else { "s" });
                for m in &found {
                    println!("    {} ({:.1}B params, {}-bit)", m.name, m.params_billions(), m.quant_bits);
                }
            }
            found.len()
        }
        Err(_) => {
            println!("  Could not scan {} (added anyway)", path.display());
            0
        }
    }
}

/// First-run interactive setup. Asks for model dir, saves config, returns the paths.
pub fn first_run_setup() -> Result<Vec<PathBuf>> {
    println!();
    println!("  Welcome to Melange — \"The memory must flow\"");
    println!();
    println!("  First-time setup: I need to know where your models live.");
    println!("  This is the directory containing your model folders");
    println!("  (each with a config.json and safetensors files).");
    println!();

    let model_dir = prompt_model_dir("  Model directory path: ")?;
    println!();
    scan_and_report(&model_dir);

    let cfg = MelangeConfig {
        model_dirs: vec![model_dir.to_string_lossy().to_string()],
        vpn: None,
    };
    save_config(&cfg)?;

    println!();
    println!("  Saved to {}", config_path()?.display());
    println!("  Add more directories later with `melange add /path`.");
    println!();

    Ok(vec![model_dir])
}

/// `melange add /path` — register a new model directory.
pub fn run_add_command(path_str: &str) -> Result<()> {
    let path = expand_tilde(path_str);

    if !path.exists() || !path.is_dir() {
        anyhow::bail!("Not a valid directory: {}", path.display());
    }

    let canonical = path.canonicalize()
        .unwrap_or_else(|_| path.clone());

    let mut cfg = load_config_or_default()?;

    // Check for duplicates
    for existing in &cfg.model_dirs {
        let existing_canonical = PathBuf::from(existing).canonicalize()
            .unwrap_or_else(|_| PathBuf::from(existing));
        if existing_canonical == canonical {
            println!();
            println!("  Already registered: {}", path.display());
            println!();
            return Ok(());
        }
    }

    println!();
    println!("  Scanning {}...", path.display());
    println!();
    scan_and_report(&path);

    cfg.model_dirs.push(canonical.to_string_lossy().to_string());
    save_config(&cfg)?;

    println!();
    println!("  Directory added. ({} total)", cfg.model_dirs.len());
    println!();
    Ok(())
}

/// `melange dirs` — list registered model directories.
pub fn run_dirs_command() -> Result<()> {
    let cfg = load_config_or_default()?;

    println!();
    if cfg.model_dirs.is_empty() {
        println!("  No model directories registered.");
        println!("  Add one with: melange add /path/to/models");
    } else {
        println!("  Registered model directories:");
        println!();
        for (i, dir) in cfg.model_dirs.iter().enumerate() {
            let path = PathBuf::from(dir);
            let exists = path.exists();
            let status = if exists { "" } else { " (not found)" };
            println!("  {}. {}{}", i + 1, dir, status);
        }
    }
    println!();
    Ok(())
}

/// `melange remove /path` — unregister a model directory.
pub fn run_remove_command(path_str: &str) -> Result<()> {
    let path = expand_tilde(path_str);
    let canonical = path.canonicalize()
        .unwrap_or_else(|_| path.clone());

    let mut cfg = load_config_or_default()?;
    let before = cfg.model_dirs.len();

    cfg.model_dirs.retain(|existing| {
        let existing_canonical = PathBuf::from(existing).canonicalize()
            .unwrap_or_else(|_| PathBuf::from(existing));
        existing_canonical != canonical
    });

    println!();
    if cfg.model_dirs.len() < before {
        save_config(&cfg)?;
        println!("  Removed: {}", path.display());
        println!("  ({} director{} remaining)", cfg.model_dirs.len(),
            if cfg.model_dirs.len() == 1 { "y" } else { "ies" });
    } else {
        println!("  Not found in config: {}", path.display());
        println!("  Run `melange dirs` to see registered directories.");
    }
    println!();
    Ok(())
}

/// Interactive `melange config` subcommand.
pub fn run_config_command() -> Result<()> {
    let path = config_path()?;
    let cfg = load_config_or_default()?;

    println!();
    println!("  Melange Configuration");
    println!("  Config file: {}", path.display());
    println!();

    if cfg.model_dirs.is_empty() {
        println!("  No model directories registered.");
    } else {
        println!("  Model directories:");
        for (i, dir) in cfg.model_dirs.iter().enumerate() {
            println!("    {}. {}", i + 1, dir);
        }
    }

    println!();
    println!("  Commands:");
    println!("    melange add /path     Add a model directory");
    println!("    melange remove /path  Remove a model directory");
    println!("    melange dirs          List all directories");
    println!();

    Ok(())
}

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MelangeConfig {
    pub model_dir: String,
}

/// Returns `~/.config/melange/config.toml`
pub fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not determine config directory")?;
    Ok(config_dir.join("melange").join("config.toml"))
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
        "# Melange configuration\n# Run `melange config` to change these settings\n\n{}",
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
            println!("Please enter a path.");
            continue;
        }

        let path = expand_tilde(input);

        if !path.exists() {
            println!("Directory does not exist: {}", path.display());
            println!("Please enter a valid path.\n");
            continue;
        }

        if !path.is_dir() {
            println!("Not a directory: {}", path.display());
            println!("Please enter a valid directory path.\n");
            continue;
        }

        return Ok(path);
    }
}

/// First-run interactive setup. Asks for model dir, saves config, returns the path.
pub fn first_run_setup() -> Result<PathBuf> {
    println!();
    println!("  Welcome to Melange — \"The memory must flow\"");
    println!();
    println!("  First-time setup: I need to know where your models live.");
    println!("  This is the directory containing your model folders");
    println!("  (each with a config.json and safetensors files).");
    println!();

    let model_dir = prompt_model_dir("  Model directory path: ")?;

    let cfg = MelangeConfig {
        model_dir: model_dir.to_string_lossy().to_string(),
    };
    save_config(&cfg)?;

    println!();
    println!("  Saved to {}", config_path()?.display());
    println!("  You can change this later with `melange config`.");
    println!();

    Ok(model_dir)
}

/// Interactive `melange config` subcommand.
pub fn run_config_command() -> Result<()> {
    let path = config_path()?;

    println!();
    println!("  Melange Configuration");
    println!("  Config file: {}", path.display());
    println!();

    match load_config()? {
        Some(cfg) => {
            println!("  Current model directory: {}", cfg.model_dir);
        }
        None => {
            println!("  No config file found (using defaults).");
            println!("  Default model directory: {}", default_model_dir().display());
        }
    }

    println!();
    print!("  Change model directory? [y/N] ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;

    if answer.trim().eq_ignore_ascii_case("y") {
        let model_dir = prompt_model_dir("  New model directory path: ")?;
        let cfg = MelangeConfig {
            model_dir: model_dir.to_string_lossy().to_string(),
        };
        save_config(&cfg)?;
        println!();
        println!("  Updated! Model directory: {}", model_dir.display());
    } else {
        println!("  No changes made.");
    }

    println!();
    Ok(())
}

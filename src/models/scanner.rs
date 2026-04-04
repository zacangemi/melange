use anyhow::Result;
use std::path::{Path, PathBuf};

use super::ModelInfo;
use super::config_parser;
use super::index_parser;

/// Scan safetensors directories and Ollama models, merged into one list.
pub fn scan_all_models(dirs: &[PathBuf]) -> Vec<ModelInfo> {
    let mut models = scan_directories(dirs);
    models.extend(super::ollama::scan_ollama_models());
    models
}

/// Scan multiple directories and merge results.
pub fn scan_directories(dirs: &[PathBuf]) -> Vec<ModelInfo> {
    let mut all_models = Vec::new();
    for dir in dirs {
        match scan_directory(dir) {
            Ok(models) => all_models.extend(models),
            Err(e) => {
                eprintln!("Warning: Could not scan {}: {}", dir.display(), e);
            }
        }
    }
    all_models
}

/// Walk a directory looking for model subdirectories (those containing config.json).
pub fn scan_directory(model_dir: &Path) -> Result<Vec<ModelInfo>> {
    let mut models = Vec::new();

    if !model_dir.exists() {
        anyhow::bail!("Model directory does not exist: {}", model_dir.display());
    }

    let entries = std::fs::read_dir(model_dir)?;
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect();

    dirs.sort();

    for dir in dirs {
        let config_path = dir.join("config.json");
        if config_path.exists() {
            match parse_model(&dir) {
                Ok(model) => models.push(model),
                Err(e) => {
                    eprintln!("Warning: Failed to parse model in {}: {}", dir.display(), e);
                }
            }
        }
    }

    Ok(models)
}

fn parse_model(dir: &Path) -> Result<ModelInfo> {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let config = config_parser::parse_config(&dir.join("config.json"))?;
    let index = index_parser::parse_index(&dir.join("model.safetensors.index.json"))
        .unwrap_or_default();

    let is_moe = config.num_experts > 0;
    let head_dim = if config.head_dim > 0 {
        config.head_dim
    } else if config.num_attention_heads > 0 {
        config.hidden_size / config.num_attention_heads
    } else {
        128 // default fallback
    };

    Ok(ModelInfo {
        name,
        path: dir.to_string_lossy().to_string(),
        model_type: config.model_type.clone(),
        architecture: config.architectures.first().cloned().unwrap_or_default(),
        hidden_size: config.hidden_size,
        num_hidden_layers: config.num_hidden_layers,
        num_attention_heads: config.num_attention_heads,
        num_key_value_heads: config.num_key_value_heads,
        head_dim,
        max_position_embeddings: config.max_position_embeddings,
        num_experts: config.num_experts,
        num_experts_per_tok: config.num_experts_per_tok,
        moe_intermediate_size: config.moe_intermediate_size,
        v_head_dim: config.v_head_dim,
        qk_nope_head_dim: config.qk_nope_head_dim,
        qk_rope_head_dim: config.qk_rope_head_dim,
        quant_bits: config.quant_bits,
        quant_group_size: config.quant_group_size,
        total_size_bytes: index.total_size,
        total_parameters: index.total_parameters,
        is_moe,
    })
}

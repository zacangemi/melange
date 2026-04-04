use std::collections::HashSet;
use std::path::PathBuf;

use super::gguf_parser::{self, GgufMetadata};
use super::ModelInfo;

/// Returns the Ollama models directory, checking `$OLLAMA_MODELS` first,
/// then falling back to `~/.ollama/models/`.
pub fn ollama_models_dir() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("OLLAMA_MODELS") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }

    let home = dirs::home_dir()?;
    let default_path = home.join(".ollama").join("models");
    if default_path.exists() {
        Some(default_path)
    } else {
        None
    }
}

/// Scan Ollama's manifest directory for all installed models.
/// Returns an empty Vec if Ollama is not installed or has no models.
pub fn scan_ollama_models() -> Vec<ModelInfo> {
    let models_dir = match ollama_models_dir() {
        Some(d) => d,
        None => return Vec::new(),
    };

    let manifests_dir = models_dir
        .join("manifests")
        .join("registry.ollama.ai")
        .join("library");

    if !manifests_dir.exists() {
        return Vec::new();
    }

    // Collect (model_name, tag, blob_digest) from manifests
    let mut blob_entries: Vec<(String, String)> = Vec::new(); // (display_name, digest)
    let mut seen_digests: HashSet<String> = HashSet::new();

    let model_dirs = match std::fs::read_dir(&manifests_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    for model_entry in model_dirs.flatten() {
        if !model_entry.path().is_dir() {
            continue;
        }
        let model_name = model_entry.file_name().to_string_lossy().to_string();

        let tags = match std::fs::read_dir(model_entry.path()) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for tag_entry in tags.flatten() {
            let tag_path = tag_entry.path();
            if !tag_path.is_file() {
                continue;
            }
            let tag_name = tag_entry.file_name().to_string_lossy().to_string();
            let display_name = format!("{}:{}", model_name, tag_name);

            match parse_manifest_digest(&tag_path) {
                Some(digest) => {
                    if seen_digests.insert(digest.clone()) {
                        blob_entries.push((display_name, digest));
                    }
                    // If digest already seen, skip (deduplication)
                }
                None => {
                    eprintln!("Warning: Could not parse Ollama manifest: {}", tag_path.display());
                }
            }
        }
    }

    // Resolve each blob and parse GGUF metadata
    let mut models = Vec::new();
    for (display_name, digest) in blob_entries {
        let blob_path = resolve_blob_path(&models_dir, &digest);
        if !blob_path.exists() {
            eprintln!("Warning: Ollama blob not found: {}", blob_path.display());
            continue;
        }

        match gguf_parser::parse_gguf_metadata(&blob_path) {
            Ok(meta) => {
                models.push(gguf_to_model_info(&display_name, &blob_path, &meta));
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse GGUF for {}: {}", display_name, e);
            }
        }
    }

    models
}

/// Parse an Ollama manifest JSON to extract the model layer's blob digest.
fn parse_manifest_digest(manifest_path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(manifest_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let layers = json.get("layers")?.as_array()?;
    for layer in layers {
        let media_type = layer.get("mediaType")?.as_str()?;
        if media_type == "application/vnd.ollama.image.model" {
            let digest = layer.get("digest")?.as_str()?;
            return Some(digest.to_string());
        }
    }
    None
}

/// Convert a digest like "sha256:abc123..." to the blob file path.
/// Ollama stores blobs as `blobs/sha256-abc123...` (colon replaced with dash).
fn resolve_blob_path(models_dir: &std::path::Path, digest: &str) -> PathBuf {
    let blob_name = digest.replace(':', "-");
    models_dir.join("blobs").join(blob_name)
}

/// Map GGUF metadata to a ModelInfo struct.
fn gguf_to_model_info(name: &str, blob_path: &std::path::Path, meta: &GgufMetadata) -> ModelInfo {
    let arch = meta
        .get_str("general.architecture")
        .unwrap_or("unknown")
        .to_string();

    let block_count = meta
        .get_u32(&format!("{}.block_count", arch))
        .unwrap_or(0);

    let embedding_length = meta
        .get_u32(&format!("{}.embedding_length", arch))
        .unwrap_or(0);

    let head_count = meta
        .get_u32(&format!("{}.attention.head_count", arch))
        .unwrap_or(0);

    let head_count_kv = meta
        .get_u32(&format!("{}.attention.head_count_kv", arch))
        .unwrap_or(head_count); // default to head_count if not present (MHA)

    let context_length = meta
        .get_u64(&format!("{}.context_length", arch))
        .unwrap_or(4096); // safe default

    let expert_count = meta
        .get_u32(&format!("{}.expert_count", arch))
        .unwrap_or(0);

    let expert_used_count = meta
        .get_u32(&format!("{}.expert_used_count", arch))
        .unwrap_or(0);

    let ff_length = meta
        .get_u32(&format!("{}.feed_forward_length", arch))
        .unwrap_or(0);

    let is_moe = expert_count > 0;

    // Head dimension: check rope dimension count, then compute from embedding/heads
    let head_dim = meta
        .get_u32(&format!("{}.rope.dimension_count", arch))
        .unwrap_or_else(|| {
            if head_count > 0 {
                embedding_length / head_count
            } else {
                128
            }
        });

    // MLA fields (DeepSeek-style)
    let v_head_dim = meta.get_u32(&format!("{}.attention.value_length", arch));
    let qk_nope_head_dim = meta.get_u32(&format!("{}.attention.key_length_nope", arch));
    let qk_rope_head_dim = meta.get_u32(&format!("{}.attention.key_length_rope", arch));

    // Quantization bits from file_type
    let quant_bits = meta
        .get_u32("general.file_type")
        .map(file_type_to_bits)
        .unwrap_or(4);

    // Total parameters: prefer explicit metadata, then estimate
    let total_parameters = meta
        .get_u64("general.parameter_count")
        .unwrap_or_else(|| estimate_parameters(embedding_length, block_count, head_count, ff_length, expert_count, meta.tensor_count));

    // File size on disk
    let total_size_bytes = std::fs::metadata(blob_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Model type: use general.name if available, otherwise architecture
    let model_type = meta
        .get_str("general.name")
        .unwrap_or(&arch)
        .to_string();

    ModelInfo {
        name: name.to_string(),
        path: blob_path.to_string_lossy().to_string(),
        model_type,
        architecture: arch,
        hidden_size: embedding_length,
        num_hidden_layers: block_count,
        num_attention_heads: head_count,
        num_key_value_heads: head_count_kv,
        head_dim,
        max_position_embeddings: context_length,
        num_experts: expert_count,
        num_experts_per_tok: expert_used_count,
        moe_intermediate_size: if is_moe { ff_length } else { 0 },
        v_head_dim,
        qk_nope_head_dim,
        qk_rope_head_dim,
        quant_bits,
        quant_group_size: 128, // GGUF quantization uses 32-element super-blocks, but 128 is the effective group size for memory calc
        total_size_bytes,
        total_parameters,
        is_moe,
    }
}

/// Map GGUF file_type integer to approximate quantization bits.
fn file_type_to_bits(file_type: u32) -> u32 {
    match file_type {
        0 => 32,  // ALL_F32
        1 => 16,  // MOSTLY_F16
        2 => 4,   // MOSTLY_Q4_0
        3 => 4,   // MOSTLY_Q4_1
        7 => 8,   // MOSTLY_Q8_0
        8 => 5,   // MOSTLY_Q5_0
        9 => 5,   // MOSTLY_Q5_1
        10 => 2,  // MOSTLY_Q2_K
        11 => 3,  // MOSTLY_Q3_K_S
        12 => 3,  // MOSTLY_Q3_K_M
        13 => 3,  // MOSTLY_Q3_K_L
        14 => 4,  // MOSTLY_Q4_K_S
        15 => 4,  // MOSTLY_Q4_K_M
        16 => 5,  // MOSTLY_Q5_K_S
        17 => 5,  // MOSTLY_Q5_K_M
        18 => 6,  // MOSTLY_Q6_K
        19 => 2,  // MOSTLY_IQ2_XXS
        20 => 2,  // MOSTLY_IQ2_XS
        21 => 3,  // MOSTLY_IQ3_XXS
        22 => 1,  // MOSTLY_IQ1_S
        23 => 4,  // MOSTLY_IQ4_NL
        24 => 3,  // MOSTLY_IQ3_S
        25 => 4,  // MOSTLY_IQ4_XS
        26 => 2,  // MOSTLY_IQ2_S
        27 => 2,  // MOSTLY_IQ2_M
        28 => 3,  // MOSTLY_IQ3_M
        _ => 4,   // Unknown — safe default
    }
}

/// Estimate total parameters from architecture dimensions when
/// `general.parameter_count` is absent.
fn estimate_parameters(
    hidden_size: u32,
    num_layers: u32,
    num_heads: u32,
    ff_length: u32,
    num_experts: u32,
    _tensor_count: u64,
) -> u64 {
    if hidden_size == 0 || num_layers == 0 {
        return 0;
    }

    let h = hidden_size as u64;

    // Attention: Q, K, V, O projections = 4 * h * h per layer
    let attn_per_layer = 4 * h * h;

    // FFN: gate + up + down = 3 * h * ff_length per layer (per expert if MoE)
    let ff = if ff_length > 0 {
        ff_length as u64
    } else {
        // Default FFN multiplier when not specified
        let computed = (h * 8 / 3 + 255) / 256 * 256; // typical llama FFN sizing
        computed
    };
    let ffn_per_expert = 3 * h * ff;
    let experts = if num_experts > 0 { num_experts as u64 } else { 1 };
    let ffn_per_layer = ffn_per_expert * experts;

    // Layer norms: 2 per layer * hidden_size
    let norms_per_layer = 2 * h;

    let per_layer = attn_per_layer + ffn_per_layer + norms_per_layer;
    let transformer_params = per_layer * num_layers as u64;

    // Embedding + output head (often tied, but count both for estimate)
    // Vocab size is typically 32K-128K — use 32K as conservative estimate
    let vocab_size: u64 = 32_000;
    let embedding_params = vocab_size * h * 2;

    // Router for MoE (small)
    let router_params = if num_experts > 0 {
        num_layers as u64 * h * experts
    } else {
        0
    };

    let _ = num_heads; // used indirectly via hidden_size

    transformer_params + embedding_params + router_params
}

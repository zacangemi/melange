use anyhow::{Result, Context};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ParsedConfig {
    pub architectures: Vec<String>,
    pub model_type: String,
    pub hidden_size: u32,
    pub num_hidden_layers: u32,
    pub num_attention_heads: u32,
    pub num_key_value_heads: u32,
    pub head_dim: u32,
    pub max_position_embeddings: u64,
    pub num_experts: u32,
    pub num_experts_per_tok: u32,
    pub moe_intermediate_size: u32,
    pub v_head_dim: Option<u32>,
    pub qk_nope_head_dim: Option<u32>,
    pub qk_rope_head_dim: Option<u32>,
    pub quant_bits: u32,
    pub quant_group_size: u32,
}

/// Raw JSON structure — uses Option for all fields since configs vary widely
#[derive(Deserialize)]
struct RawConfig {
    architectures: Option<Vec<String>>,
    model_type: Option<String>,
    hidden_size: Option<u32>,
    num_hidden_layers: Option<u32>,
    num_attention_heads: Option<u32>,
    num_key_value_heads: Option<u32>,
    head_dim: Option<u32>,
    max_position_embeddings: Option<u64>,

    // MoE fields (Qwen style)
    num_experts: Option<u32>,
    num_experts_per_tok: Option<u32>,
    moe_intermediate_size: Option<u32>,

    // MoE fields (GLM style)
    n_routed_experts: Option<u32>,

    // MLA fields (GLM style)
    v_head_dim: Option<u32>,
    qk_nope_head_dim: Option<u32>,
    qk_rope_head_dim: Option<u32>,

    // Quantization
    quantization_config: Option<QuantConfig>,
    quantization: Option<QuantConfig>,
}

#[derive(Deserialize)]
struct QuantConfig {
    bits: Option<u32>,
    group_size: Option<u32>,
}

pub fn parse_config(path: &Path) -> Result<ParsedConfig> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let raw: RawConfig = serde_json::from_str(&text)
        .with_context(|| format!("Failed to parse config JSON: {}", path.display()))?;

    let quant = raw.quantization_config.as_ref().or(raw.quantization.as_ref());

    let num_experts = raw.num_experts
        .or(raw.n_routed_experts)
        .unwrap_or(0);

    Ok(ParsedConfig {
        architectures: raw.architectures.unwrap_or_default(),
        model_type: raw.model_type.unwrap_or_else(|| "unknown".to_string()),
        hidden_size: raw.hidden_size.unwrap_or(0),
        num_hidden_layers: raw.num_hidden_layers.unwrap_or(0),
        num_attention_heads: raw.num_attention_heads.unwrap_or(0),
        num_key_value_heads: raw.num_key_value_heads.unwrap_or(0),
        head_dim: raw.head_dim.unwrap_or(0),
        max_position_embeddings: raw.max_position_embeddings.unwrap_or(0),
        num_experts,
        num_experts_per_tok: raw.num_experts_per_tok.unwrap_or(0),
        moe_intermediate_size: raw.moe_intermediate_size.unwrap_or(0),
        v_head_dim: raw.v_head_dim,
        qk_nope_head_dim: raw.qk_nope_head_dim,
        qk_rope_head_dim: raw.qk_rope_head_dim,
        quant_bits: quant.and_then(|q| q.bits).unwrap_or(16),
        quant_group_size: quant.and_then(|q| q.group_size).unwrap_or(0),
    })
}

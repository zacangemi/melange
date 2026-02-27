pub mod scanner;
pub mod config_parser;
pub mod index_parser;
pub mod memory_calc;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub model_type: String,
    pub architecture: String,

    // Architecture params
    pub hidden_size: u32,
    pub num_hidden_layers: u32,
    pub num_attention_heads: u32,
    pub num_key_value_heads: u32,
    pub head_dim: u32,
    pub max_position_embeddings: u64,

    // MoE fields (0 for dense models)
    pub num_experts: u32,
    pub num_experts_per_tok: u32,
    pub moe_intermediate_size: u32,

    // MLA fields for GLM-style models
    pub v_head_dim: Option<u32>,
    pub qk_nope_head_dim: Option<u32>,
    pub qk_rope_head_dim: Option<u32>,

    // Quantization
    pub quant_bits: u32,
    pub quant_group_size: u32,

    // From index.json
    pub total_size_bytes: u64,
    pub total_parameters: u64,

    // Computed
    pub is_moe: bool,
}

impl ModelInfo {
    pub fn size_gb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn params_billions(&self) -> f64 {
        self.total_parameters as f64 / 1_000_000_000.0
    }

    pub fn active_params(&self) -> u64 {
        if !self.is_moe || self.num_experts == 0 || self.moe_intermediate_size == 0 {
            return self.total_parameters;
        }
        // Compute expert params from architecture: each expert has 3 matrices
        // (gate_proj, up_proj, down_proj) of size [hidden_size, moe_intermediate_size]
        let expert_params_per_layer = self.num_experts as u64
            * self.moe_intermediate_size as u64
            * self.hidden_size as u64
            * 3;
        let total_expert_params = expert_params_per_layer * self.num_hidden_layers as u64;

        let non_expert_params = self.total_parameters.saturating_sub(total_expert_params);
        let active_expert_params = (self.num_experts_per_tok as f64 / self.num_experts as f64
            * total_expert_params as f64) as u64;

        non_expert_params + active_expert_params
    }

    pub fn quant_label(&self) -> String {
        format!("{}-bit", self.quant_bits)
    }

    pub fn type_label(&self) -> &str {
        if self.is_moe { "MoE" } else { "Dense" }
    }
}

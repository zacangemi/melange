use serde::Serialize;
use super::ModelInfo;

const OS_RESERVED_BYTES: u64 = 3_758_096_384; // 3.5 GB
const RUNTIME_OVERHEAD_FRACTION: f64 = 0.10;
const BUFFER_BYTES: u64 = 1_073_741_824; // 1 GB safety buffer

/// Memory estimation for a model at a given context length
#[derive(Debug, Clone, Serialize)]
pub struct MemoryEstimate {
    pub weight_bytes: u64,
    pub kv_cache_bytes: u64,
    pub overhead_bytes: u64,
    pub os_reserved_bytes: u64,
    pub total_bytes: u64,
    pub context_length: u64,
}

impl MemoryEstimate {
    pub fn weight_gb(&self) -> f64 {
        self.weight_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn kv_cache_gb(&self) -> f64 {
        self.kv_cache_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn overhead_gb(&self) -> f64 {
        self.overhead_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn total_gb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

/// Full analysis result for a model on specific hardware
#[derive(Debug, Clone, Serialize)]
pub struct ModelAnalysis {
    pub estimates: Vec<MemoryEstimate>,  // at various context lengths
    pub max_safe_context: u64,
    pub headroom_bytes: i64,             // at lowest context (4K)
    pub status: SpiceStatus,
    pub tok_s_low: f64,
    pub tok_s_high: f64,
    pub kv_per_token_bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum SpiceStatus {
    AbundantSpice,   // > 4 GB headroom
    SpiceThinning,   // 1-4 GB headroom
    SpiceScarcity,   // 0-1 GB headroom
    DesertDrought,   // negative headroom = swap
}

impl SpiceStatus {
    pub fn label(&self) -> &str {
        match self {
            SpiceStatus::AbundantSpice => "Abundant Spice",
            SpiceStatus::SpiceThinning => "Spice Thinning",
            SpiceStatus::SpiceScarcity => "Spice Scarcity",
            SpiceStatus::DesertDrought => "Desert Drought",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            SpiceStatus::AbundantSpice => "✓ Flows",
            SpiceStatus::SpiceThinning => "⚠ Thin",
            SpiceStatus::SpiceScarcity => "⚠ Scarce",
            SpiceStatus::DesertDrought => "✗ Drought",
        }
    }
}

/// KV cache bytes per token for a given model architecture
pub fn kv_per_token(model: &ModelInfo) -> u64 {
    // Check for MLA-style attention (GLM models with v_head_dim)
    if let (Some(v_dim), Some(nope_dim), Some(rope_dim)) =
        (model.v_head_dim, model.qk_nope_head_dim, model.qk_rope_head_dim)
    {
        // MLA: key dim = qk_nope + qk_rope, value dim = v_head_dim
        let k_dim = (nope_dim + rope_dim) as u64;
        let v_dim = v_dim as u64;
        let layers = model.num_hidden_layers as u64;
        let kv_heads = model.num_key_value_heads as u64;
        // key_cache + value_cache, each in FP16 (2 bytes)
        return layers * kv_heads * (k_dim + v_dim) * 2;
    }

    // Standard GQA/MHA: 2 (K+V) × layers × kv_heads × head_dim × 2 bytes (FP16)
    2 * model.num_hidden_layers as u64
        * model.num_key_value_heads as u64
        * model.head_dim as u64
        * 2
}

/// Estimate memory usage at a specific context length
pub fn estimate_at_context(model: &ModelInfo, context_len: u64) -> MemoryEstimate {
    let weight_bytes = model.total_size_bytes;
    let kv_bytes = kv_per_token(model) * context_len;
    let overhead_bytes = (weight_bytes as f64 * RUNTIME_OVERHEAD_FRACTION) as u64;

    MemoryEstimate {
        weight_bytes,
        kv_cache_bytes: kv_bytes,
        overhead_bytes,
        os_reserved_bytes: OS_RESERVED_BYTES,
        total_bytes: weight_bytes + kv_bytes + overhead_bytes + OS_RESERVED_BYTES,
        context_length: context_len,
    }
}

/// Maximum safe context length before hitting swap
pub fn max_safe_context(model: &ModelInfo, total_ram_bytes: u64) -> u64 {
    let available = total_ram_bytes.saturating_sub(OS_RESERVED_BYTES);
    let weight_bytes = model.total_size_bytes;
    let overhead = (weight_bytes as f64 * RUNTIME_OVERHEAD_FRACTION) as u64;

    let remaining = available
        .saturating_sub(weight_bytes)
        .saturating_sub(overhead)
        .saturating_sub(BUFFER_BYTES);

    let kv_per_tok = kv_per_token(model);
    if kv_per_tok == 0 {
        return 0;
    }

    remaining / kv_per_tok
}

/// Estimate tok/s based on memory bandwidth
pub fn estimate_tok_s(model: &ModelInfo, bandwidth_gbs: f64) -> (f64, f64) {
    let active_params = model.active_params();
    let bytes_per_param = model.quant_bits as f64 / 8.0;
    let active_bytes = active_params as f64 * bytes_per_param;

    if active_bytes == 0.0 {
        return (0.0, 0.0);
    }

    let theoretical = (bandwidth_gbs * 1_000_000_000.0) / active_bytes;

    // Real-world efficiency factors
    let low = theoretical * 0.35;
    let high = theoretical * 0.55;

    (low, high)
}

/// Full analysis of a model against specific hardware
pub fn analyze(model: &ModelInfo, total_ram_bytes: u64, bandwidth_gbs: f64) -> ModelAnalysis {
    let context_lengths = [4096, 8192, 16384, 32768, 65536, 131072];

    let estimates: Vec<MemoryEstimate> = context_lengths
        .iter()
        .filter(|&&ctx| ctx <= model.max_position_embeddings)
        .map(|&ctx| estimate_at_context(model, ctx))
        .collect();

    let max_ctx = max_safe_context(model, total_ram_bytes);
    let (tok_s_low, tok_s_high) = estimate_tok_s(model, bandwidth_gbs);

    // Headroom at 4K context
    let est_4k = estimate_at_context(model, 4096);
    let headroom = total_ram_bytes as i64 - est_4k.total_bytes as i64;

    // Classify considering both headroom and practical usability
    let status = classify_status(headroom, max_ctx);

    let kv_per_tok = kv_per_token(model);

    ModelAnalysis {
        estimates,
        max_safe_context: max_ctx,
        headroom_bytes: headroom,
        status,
        tok_s_low,
        tok_s_high,
        kv_per_token_bytes: kv_per_tok,
    }
}

fn classify_status(headroom_bytes: i64, max_safe_ctx: u64) -> SpiceStatus {
    let headroom_gb = headroom_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    // If model doesn't fit at all at 4K, or max safe context is too small for real use
    if headroom_gb < 0.0 || max_safe_ctx < 8192 {
        return SpiceStatus::DesertDrought;
    }

    if max_safe_ctx < 16384 {
        return SpiceStatus::SpiceScarcity;
    }

    if headroom_gb > 4.0 {
        SpiceStatus::AbundantSpice
    } else if headroom_gb > 1.0 {
        SpiceStatus::SpiceThinning
    } else {
        SpiceStatus::SpiceScarcity
    }
}

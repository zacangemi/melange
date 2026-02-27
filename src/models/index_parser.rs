use anyhow::{Result, Context};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct IndexMetadata {
    pub total_size: u64,
    pub total_parameters: u64,
}

#[derive(Deserialize)]
struct RawIndex {
    metadata: Option<RawMetadata>,
}

#[derive(Deserialize)]
struct RawMetadata {
    total_size: Option<serde_json::Value>,
    total_parameters: Option<serde_json::Value>,
}

pub fn parse_index(path: &Path) -> Result<IndexMetadata> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read index: {}", path.display()))?;
    let raw: RawIndex = serde_json::from_str(&text)
        .with_context(|| format!("Failed to parse index JSON: {}", path.display()))?;

    let meta = raw.metadata.unwrap_or(RawMetadata {
        total_size: None,
        total_parameters: None,
    });

    Ok(IndexMetadata {
        total_size: parse_value_as_u64(meta.total_size),
        total_parameters: parse_value_as_u64(meta.total_parameters),
    })
}

/// Parse a JSON value that could be a number or a string containing a number
fn parse_value_as_u64(val: Option<serde_json::Value>) -> u64 {
    match val {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(serde_json::Value::String(s)) => s.parse::<u64>().unwrap_or(0),
        _ => 0,
    }
}

use anyhow::Result;
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct GpuInfo {
    pub name: String,
    pub metal_cores: u32,
    pub metal_version: String,
}

pub fn detect() -> Result<GpuInfo> {
    let output = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output()?;
    let text = String::from_utf8_lossy(&output.stdout);

    let name = extract_field(&text, "Chipset Model:").unwrap_or_else(|| "Unknown GPU".to_string());
    let metal_cores = extract_gpu_cores(&text);
    let metal_version = extract_field(&text, "Metal Support:")
        .or_else(|| extract_field(&text, "Metal Family:"))
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(GpuInfo {
        name,
        metal_cores,
        metal_version,
    })
}

fn extract_field(text: &str, label: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(label) {
            return Some(trimmed.trim_start_matches(label).trim().to_string());
        }
    }
    None
}

fn extract_gpu_cores(text: &str) -> u32 {
    // Try "Total Number of Cores:" first
    if let Some(val) = extract_field(text, "Total Number of Cores:") {
        if let Ok(n) = val.parse::<u32>() {
            return n;
        }
    }
    // Fallback: infer from chip name
    for line in text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("apple m") {
            return infer_gpu_cores_from_chip(&lower);
        }
    }
    0
}

fn infer_gpu_cores_from_chip(chip: &str) -> u32 {
    // Known Apple Silicon GPU core counts
    if chip.contains("m4 ultra") { return 80; }
    if chip.contains("m4 max") { return 40; }
    if chip.contains("m4 pro") { return 20; }
    if chip.contains("m4") { return 10; }
    if chip.contains("m3 ultra") { return 76; }
    if chip.contains("m3 max") { return 40; }
    if chip.contains("m3 pro") { return 18; }
    if chip.contains("m3") { return 10; }
    if chip.contains("m2 ultra") { return 76; }
    if chip.contains("m2 max") { return 38; }
    if chip.contains("m2 pro") { return 19; }
    if chip.contains("m2") { return 10; }
    if chip.contains("m1 ultra") { return 64; }
    if chip.contains("m1 max") { return 32; }
    if chip.contains("m1 pro") { return 16; }
    if chip.contains("m1") { return 8; }
    0
}

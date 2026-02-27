use anyhow::{Result, Context};
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct CpuInfo {
    pub brand: String,
    pub total_cores: u32,
    pub performance_cores: u32,
    pub efficiency_cores: u32,
    pub chip_variant: String,
}

pub fn detect() -> Result<CpuInfo> {
    let brand = sysctl_string("machdep.cpu.brand_string")
        .unwrap_or_else(|_| "Unknown CPU".to_string());

    let total_cores = sysctl_u32("hw.ncpu").unwrap_or(0);

    let p_cores = sysctl_u32("hw.perflevel0.logicalcpu").unwrap_or(0);
    let e_cores = sysctl_u32("hw.perflevel1.logicalcpu").unwrap_or(0);

    let (performance_cores, efficiency_cores) = if p_cores > 0 && e_cores > 0 {
        (p_cores, e_cores)
    } else {
        // Fallback: assume half and half
        (total_cores / 2, total_cores - total_cores / 2)
    };

    let chip_variant = parse_chip_variant(&brand);

    Ok(CpuInfo {
        brand,
        total_cores,
        performance_cores,
        efficiency_cores,
        chip_variant,
    })
}

fn sysctl_string(key: &str) -> Result<String> {
    let output = Command::new("sysctl")
        .arg("-n")
        .arg(key)
        .output()
        .context("Failed to run sysctl")?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn sysctl_u32(key: &str) -> Result<u32> {
    let s = sysctl_string(key)?;
    s.parse::<u32>().context("Failed to parse sysctl value")
}

fn parse_chip_variant(brand: &str) -> String {
    let brand_lower = brand.to_lowercase();
    let generations = ["m4", "m3", "m2", "m1"];
    let variants = ["ultra", "max", "pro"];

    for gen in &generations {
        if brand_lower.contains(gen) {
            for variant in &variants {
                if brand_lower.contains(variant) {
                    return format!("{} {}", gen.to_uppercase(), capitalize(variant));
                }
            }
            return gen.to_uppercase();
        }
    }
    "Unknown".to_string()
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

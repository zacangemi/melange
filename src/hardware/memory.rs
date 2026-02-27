use anyhow::Result;
use serde::Serialize;
use sysinfo::System;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub is_unified: bool,
}

impl MemoryInfo {
    pub fn total_gb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn used_gb(&self) -> f64 {
        self.used_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn available_gb(&self) -> f64 {
        self.available_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn usage_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        self.used_bytes as f64 / self.total_bytes as f64 * 100.0
    }
}

pub fn detect() -> Result<MemoryInfo> {
    let mut sys = System::new();
    sys.refresh_memory();

    // Use sysctl as authoritative total on macOS
    let total_bytes = sysctl_memsize().unwrap_or(sys.total_memory());
    let used_bytes = sys.used_memory();
    let available_bytes = total_bytes.saturating_sub(used_bytes);

    let swap_total_bytes = sys.total_swap();
    let swap_used_bytes = sys.used_swap();

    // Apple Silicon always has unified memory
    let is_unified = is_apple_silicon();

    Ok(MemoryInfo {
        total_bytes,
        used_bytes,
        available_bytes,
        swap_total_bytes,
        swap_used_bytes,
        is_unified,
    })
}

fn sysctl_memsize() -> Option<u64> {
    let output = Command::new("sysctl")
        .arg("-n")
        .arg("hw.memsize")
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&output.stdout);
    s.trim().parse::<u64>().ok()
}

fn is_apple_silicon() -> bool {
    let output = Command::new("sysctl")
        .arg("-n")
        .arg("machdep.cpu.brand_string")
        .output()
        .ok();
    match output {
        Some(o) => String::from_utf8_lossy(&o.stdout).contains("Apple"),
        None => false,
    }
}

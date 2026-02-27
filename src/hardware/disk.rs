use anyhow::Result;
use serde::Serialize;
use sysinfo::Disks;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub model_storage_bytes: u64,
}

impl DiskInfo {
    pub fn free_gb(&self) -> f64 {
        self.free_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    #[allow(dead_code)]
    pub fn model_storage_gb(&self) -> f64 {
        self.model_storage_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

pub fn detect() -> Result<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();

    let mut total_bytes = 0u64;
    let mut free_bytes = 0u64;

    // Find the root disk (or the largest one)
    for disk in disks.list() {
        if disk.mount_point() == Path::new("/") {
            total_bytes = disk.total_space();
            free_bytes = disk.available_space();
            break;
        }
    }

    // If no root found, sum all disks
    if total_bytes == 0 {
        for disk in disks.list() {
            total_bytes += disk.total_space();
            free_bytes += disk.available_space();
        }
    }

    Ok(DiskInfo {
        total_bytes,
        free_bytes,
        model_storage_bytes: 0, // Will be set after model scan
    })
}

#[allow(dead_code)]
pub fn measure_directory_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    walkdir_size(path)
}

fn walkdir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += walkdir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

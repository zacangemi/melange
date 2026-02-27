pub mod cpu;
pub mod memory;
pub mod gpu;
pub mod disk;
pub mod bandwidth;

use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HardwareInfo {
    pub cpu: cpu::CpuInfo,
    pub memory: memory::MemoryInfo,
    pub gpu: gpu::GpuInfo,
    pub disk: disk::DiskInfo,
    pub bandwidth_gbs: f64,
}

impl HardwareInfo {
    pub fn detect() -> Result<Self> {
        let cpu = cpu::detect()?;
        let memory = memory::detect()?;
        let gpu = gpu::detect()?;
        let disk = disk::detect()?;
        let bandwidth_gbs = bandwidth::lookup(&cpu.brand);

        Ok(Self {
            cpu,
            memory,
            gpu,
            disk,
            bandwidth_gbs,
        })
    }
}

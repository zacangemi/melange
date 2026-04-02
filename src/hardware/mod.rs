pub mod cpu;
pub mod memory;
pub mod gpu;
pub mod disk;
pub mod bandwidth;
pub mod os_info;
pub mod engines;
pub mod vpn;

use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HardwareInfo {
    pub os: os_info::OsInfo,
    pub cpu: cpu::CpuInfo,
    pub memory: memory::MemoryInfo,
    pub gpu: gpu::GpuInfo,
    pub disk: disk::DiskInfo,
    pub bandwidth_gbs: f64,
    pub engines: engines::InferenceEngines,
    pub vpn: vpn::VpnInfo,
}

impl HardwareInfo {
    pub fn detect() -> Result<Self> {
        let os = os_info::detect()?;
        let cpu = cpu::detect()?;
        let memory = memory::detect()?;
        let gpu = gpu::detect()?;
        let disk = disk::detect()?;
        let bandwidth_gbs = bandwidth::lookup(&cpu.brand);
        let engines = engines::detect();
        let vpn = vpn::detect();

        Ok(Self {
            os,
            cpu,
            memory,
            gpu,
            disk,
            bandwidth_gbs,
            engines,
            vpn,
        })
    }
}

use anyhow::Result;
use serde::Serialize;
use sysinfo::System;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct ProcessMemory {
    pub name: String,
    pub memory_bytes: u64,
}

impl ProcessMemory {
    pub fn memory_gb(&self) -> f64 {
        self.memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub is_unified: bool,
    pub top_processes: Vec<ProcessMemory>,
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

    // Detect top non-system processes by memory usage
    let top_processes = detect_top_processes(&mut sys);

    Ok(MemoryInfo {
        total_bytes,
        used_bytes,
        available_bytes,
        swap_total_bytes,
        swap_used_bytes,
        is_unified,
        top_processes,
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

/// Detect top non-system processes by memory, grouped by app name.
/// Returns the top 4 user-closable processes.
fn detect_top_processes(sys: &mut System) -> Vec<ProcessMemory> {
    use std::collections::HashMap;
    use sysinfo::ProcessesToUpdate;

    sys.refresh_processes(ProcessesToUpdate::All, true);

    // System processes the user can't/shouldn't close
    const SYSTEM_PROCS: &[&str] = &[
        "kernel_task", "launchd", "loginwindow", "WindowServer",
        "mds", "mds_stores", "mdworker", "mdworker_shared",
        "opendirectoryd", "fseventsd", "distnoted", "cfprefsd",
        "syslogd", "UserEventAgent", "trustd", "securityd",
        "coreduetd", "sharingd", "diagnosticd", "logd",
        "watchdogd", "powerd", "airportd", "bluetoothd",
        "containermanagerd", "symptomsd", "dasd", "remoted",
        "notifyd", "lsd", "cloudd", "nsurlsessiond",
        "sandboxd", "diskarbitrationd", "coreservicesd",
        "iconservicesagent", "coreauthd", "contextstored",
        "endpointsecurityd", "syspolicyd", "kernelmanagerd",
        "systemsoundserverd", "audiomxd", "corebrightnessd",
        "hidd", "mediaremoted", "CommCenter", "wifid",
        "thermald", "timed", "apsd", "biomed",
        "smd", "runningboardd", "revisiond",
        "corespotlightd", "Spotlight", "photolibraryd",
        "mediaanalysisd", "AMPDeviceDiscoveryAgent",
        "rapportd", "usermanagerd", "ctkd",
        "SoftwareUpdateNotificationManager",
        "ControlCenter", "Dock", "Finder", "SystemUIServer",
        "AirPlayUIAgent", "pboard", "universalaccessd",
        // Melange itself
        "melange",
    ];

    // Aggregate memory by process name (e.g., multiple Chrome Helper → "Chrome")
    let mut by_name: HashMap<String, u64> = HashMap::new();

    for (_pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_string();

        // Skip system processes and Apple frameworks
        if SYSTEM_PROCS.iter().any(|&s| name == s || name.starts_with(s)) {
            continue;
        }
        if name.starts_with("com.apple.") {
            continue;
        }

        // Skip very small processes (< 50 MB)
        let mem = process.memory();
        if mem < 50_000_000 {
            continue;
        }

        // Normalize helper process names to their parent app
        let display_name = normalize_process_name(&name);

        *by_name.entry(display_name).or_insert(0) += mem;
    }

    let mut processes: Vec<ProcessMemory> = by_name
        .into_iter()
        .map(|(name, memory_bytes)| ProcessMemory { name, memory_bytes })
        .collect();

    // Sort by memory descending, take top 4
    processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    processes.truncate(4);
    processes
}

/// Normalize helper process names to readable app names.
/// "Google Chrome Helper" → "Chrome", "Slack Helper (Renderer)" → "Slack", etc.
fn normalize_process_name(name: &str) -> String {
    // Common patterns: "AppName Helper", "AppName Helper (Renderer)", etc.
    let name = name
        .replace(" Helper (Renderer)", "")
        .replace(" Helper (GPU)", "")
        .replace(" Helper (Plugin)", "")
        .replace(" Helper", "")
        .replace(".app", "");

    // Specific renames for clarity
    match name.as_str() {
        "Google Chrome" => "Chrome".into(),
        "Firefox" | "firefox" => "Firefox".into(),
        "Microsoft Edge" => "Edge".into(),
        "com.docker.vmnetd" | "com.docker.hyperkit" | "Docker Desktop" | "com.docker.backend" => "Docker".into(),
        "com.apple.Safari" | "Safari" => "Safari".into(),
        "Code Helper" | "Electron" => "VS Code".into(),
        "Cursor" | "Cursor Helper" => "Cursor".into(),
        "node" => "Node.js".into(),
        "python3" | "python" | "Python" => "Python".into(),
        "ollama_llama_server" | "ollama" => "Ollama".into(),
        _ => name,
    }
}

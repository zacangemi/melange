use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct VpnInfo {
    pub name: String,
    pub active: bool,
    pub ip: Option<String>,
}

impl VpnInfo {
    pub fn display(&self, visible: bool) -> String {
        if !self.active {
            return "Not connected".to_string();
        }

        match &self.ip {
            Some(ip) if visible => format!("{}  {}", self.name, ip),
            Some(ip) => {
                // Mask last 3 octets: "100.64.1.23" -> "100.••.••.••"
                let first_octet = ip.split('.').next().unwrap_or("•••");
                format!("{}  {}.••.••.••", self.name, first_octet)
            }
            None => self.name.clone(),
        }
    }
}

pub fn detect() -> VpnInfo {
    // Try Tailscale first
    if let Some(info) = detect_tailscale() {
        return info;
    }

    // Try WireGuard
    if let Some(info) = detect_wireguard() {
        return info;
    }

    VpnInfo {
        name: "None".to_string(),
        active: false,
        ip: None,
    }
}

fn detect_tailscale() -> Option<VpnInfo> {
    // Try `tailscale status --json` first
    let output = Command::new("tailscale")
        .args(["status", "--json"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    let backend = json.get("BackendState")?.as_str()?;
    let active = backend == "Running";

    let ip = json
        .get("Self")
        .and_then(|s| s.get("TailscaleIPs"))
        .and_then(|ips| ips.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(VpnInfo {
        name: "Tailscale".to_string(),
        active,
        ip,
    })
}

fn detect_wireguard() -> Option<VpnInfo> {
    let output = Command::new("wg")
        .arg("show")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return None;
    }

    // WireGuard is active if `wg show` produces output
    Some(VpnInfo {
        name: "WireGuard".to_string(),
        active: true,
        ip: None,
    })
}

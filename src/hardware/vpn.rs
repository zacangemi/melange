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

/// Detect the active mesh VPN. Priority order reflects AI infrastructure usage.
/// If `preferred` is set (from config), only check that provider.
pub fn detect(preferred: Option<&str>) -> VpnInfo {
    // If user configured a specific VPN, only look for that one
    if let Some(pref) = preferred {
        return match pref.to_lowercase().as_str() {
            "tailscale" => detect_tailscale().unwrap_or(not_connected()),
            "zerotier"  => detect_zerotier().unwrap_or(not_connected()),
            "nebula"    => detect_nebula().unwrap_or(not_connected()),
            "wireguard" => detect_wireguard().unwrap_or(not_connected()),
            _ => not_connected(),
        };
    }

    // Auto-detect: mesh VPNs used for AI infrastructure, in priority order
    let detectors: &[fn() -> Option<VpnInfo>] = &[
        detect_tailscale,   // Industry standard for AI clusters
        detect_zerotier,    // Popular self-hosted alternative
        detect_nebula,      // Slack's mesh VPN, used in larger setups
        detect_wireguard,   // Raw WireGuard (catch-all for custom setups)
    ];

    for detector in detectors {
        if let Some(info) = detector() {
            return info;
        }
    }

    // Last resort: check for any active tunnel interface
    detect_tunnel_interface().unwrap_or(not_connected())
}

fn not_connected() -> VpnInfo {
    VpnInfo { name: "None".to_string(), active: false, ip: None }
}

// ---------------------------------------------------------------------------
// Tailscale
// ---------------------------------------------------------------------------

fn find_tailscale() -> Option<String> {
    // Check PATH first (Homebrew, apt, dnf, manual installs)
    if let Some(path) = find_in_path("tailscale") {
        return Some(path);
    }

    // Known locations that may not be in PATH
    let known_paths = [
        "/Applications/Tailscale.app/Contents/MacOS/Tailscale", // macOS App Store
        "/usr/bin/tailscale",
        "/usr/local/bin/tailscale",
        "/usr/sbin/tailscale",
        "/opt/homebrew/bin/tailscale",       // Homebrew Apple Silicon
        "/usr/local/Homebrew/bin/tailscale", // Homebrew Intel
        "/snap/bin/tailscale",              // Ubuntu Snap
    ];

    for path in known_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    None
}

fn detect_tailscale() -> Option<VpnInfo> {
    let bin = find_tailscale()?;
    let output = Command::new(&bin)
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

    Some(VpnInfo { name: "Tailscale".to_string(), active, ip })
}

// ---------------------------------------------------------------------------
// ZeroTier
// ---------------------------------------------------------------------------

fn detect_zerotier() -> Option<VpnInfo> {
    let bin = find_in_path("zerotier-cli")
        .or_else(|| check_path("/usr/local/bin/zerotier-cli"))
        .or_else(|| check_path("/opt/homebrew/bin/zerotier-cli"))
        .or_else(|| check_path("/usr/sbin/zerotier-cli"))?;

    let output = Command::new(&bin)
        .args(["info", "-j"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    let online = json.get("online")?.as_bool().unwrap_or(false);

    // Get IP from the first joined network
    let ip = Command::new(&bin)
        .args(["listnetworks", "-j"])
        .output()
        .ok()
        .and_then(|o| {
            let nets: serde_json::Value = serde_json::from_slice(&o.stdout).ok()?;
            nets.as_array()?
                .first()?
                .get("assignedAddresses")?
                .as_array()?
                .first()?
                .as_str()
                .map(|s| s.split('/').next().unwrap_or(s).to_string())
        });

    Some(VpnInfo { name: "ZeroTier".to_string(), active: online, ip })
}

// ---------------------------------------------------------------------------
// Nebula
// ---------------------------------------------------------------------------

fn detect_nebula() -> Option<VpnInfo> {
    // Nebula doesn't have a query CLI — check if the process is running
    // and look for its tun interface
    let _ = find_in_path("nebula").or_else(|| check_path("/usr/local/bin/nebula"))?;

    // Check if nebula process is running
    let output = Command::new("pgrep").arg("nebula").output().ok()?;
    if !output.status.success() {
        return None;
    }

    // Get IP from the nebula tun interface (typically nebula1 or tun0)
    let ip = get_interface_ip("nebula1").or_else(|| get_interface_ip("nebula0"));

    Some(VpnInfo { name: "Nebula".to_string(), active: true, ip })
}

// ---------------------------------------------------------------------------
// WireGuard (raw, not via Tailscale)
// ---------------------------------------------------------------------------

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

    // Extract interface name from first line, then get its IP
    let iface = stdout.lines().next()
        .and_then(|line| line.strip_prefix("interface: "))
        .map(|s| s.trim().to_string());

    let ip = iface.as_deref().and_then(get_interface_ip);

    Some(VpnInfo { name: "WireGuard".to_string(), active: true, ip })
}

// ---------------------------------------------------------------------------
// Fallback: detect any active tunnel interface
// ---------------------------------------------------------------------------

fn detect_tunnel_interface() -> Option<VpnInfo> {
    // On macOS/Linux, list interfaces and look for utun/tun devices with IPs
    // in private VPN ranges (10.x, 100.64-127.x, 172.16-31.x)
    let output = Command::new("ifconfig").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_iface = "";

    for line in stdout.lines() {
        // Interface header lines aren't indented
        if !line.starts_with('\t') && !line.starts_with(' ') {
            if let Some(name) = line.split(':').next() {
                current_iface = name;
            }
        }

        // Only look at tunnel interfaces
        let is_tunnel = current_iface.starts_with("utun")
            || current_iface.starts_with("tun")
            || current_iface.starts_with("wg");

        if !is_tunnel {
            continue;
        }

        // Look for an inet line with a VPN-like IP
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("inet ") {
            let ip = rest.split_whitespace().next().unwrap_or("");
            if is_vpn_ip(ip) {
                return Some(VpnInfo {
                    name: "VPN".to_string(),
                    active: true,
                    ip: Some(ip.to_string()),
                });
            }
        }
    }

    None
}

/// Check if an IP looks like a VPN/overlay address
fn is_vpn_ip(ip: &str) -> bool {
    let octets: Vec<u8> = ip.split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if octets.len() != 4 {
        return false;
    }

    match octets[0] {
        100 => (64..=127).contains(&octets[1]),  // CGNAT range (Tailscale, etc.)
        10 => true,                               // 10.0.0.0/8 (common VPN range)
        172 => (16..=31).contains(&octets[1]),    // 172.16.0.0/12
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn find_in_path(cmd: &str) -> Option<String> {
    let output = Command::new("which").arg(cmd).output().ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() { Some(path) } else { None }
    } else {
        None
    }
}

fn check_path(path: &str) -> Option<String> {
    if std::path::Path::new(path).exists() {
        Some(path.to_string())
    } else {
        None
    }
}

fn get_interface_ip(iface: &str) -> Option<String> {
    let output = Command::new("ifconfig").arg(iface).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("inet ") {
            let ip = rest.split_whitespace().next()?;
            return Some(ip.to_string());
        }
    }
    None
}

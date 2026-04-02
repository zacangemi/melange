use anyhow::Result;
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub display: String,
}

pub fn detect() -> Result<OsInfo> {
    // macOS: use sw_vers
    if let Ok(info) = detect_macos() {
        return Ok(info);
    }

    // Linux: parse /etc/os-release
    if let Ok(info) = detect_linux() {
        return Ok(info);
    }

    Ok(OsInfo {
        name: "Unknown OS".to_string(),
        version: String::new(),
        display: "Unknown OS".to_string(),
    })
}

fn detect_macos() -> Result<OsInfo> {
    let name_output = Command::new("sw_vers")
        .arg("-productName")
        .output()?;
    let name = String::from_utf8_lossy(&name_output.stdout).trim().to_string();

    if name.is_empty() {
        anyhow::bail!("sw_vers returned empty product name");
    }

    let version_output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()?;
    let version = String::from_utf8_lossy(&version_output.stdout).trim().to_string();

    let display = format!("{} {}", name, version);

    Ok(OsInfo { name, version, display })
}

fn detect_linux() -> Result<OsInfo> {
    let content = std::fs::read_to_string("/etc/os-release")?;

    let mut pretty_name = None;
    let mut name = None;
    let mut version = None;

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
            pretty_name = Some(val.trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("NAME=") {
            name = Some(val.trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("VERSION=") {
            version = Some(val.trim_matches('"').to_string());
        }
    }

    if let Some(display) = pretty_name {
        let n = name.unwrap_or_else(|| display.clone());
        let v = version.unwrap_or_default();
        return Ok(OsInfo { name: n, version: v, display });
    }

    anyhow::bail!("/etc/os-release missing PRETTY_NAME");
}

use std::io::Read;

use anyhow::{bail, Context, Result};

const GITHUB_REPO: &str = "zacangemi/melange";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(serde::Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

fn target_triple() -> Result<String> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let triple = match (os, arch) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        _ => bail!("Unsupported platform: {os}-{arch}"),
    };

    Ok(triple.to_string())
}

fn parse_version(s: &str) -> Result<(u64, u64, u64)> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        bail!("Invalid version format: {s}");
    }
    let major = parts[0].parse::<u64>().context("Invalid major version")?;
    let minor = parts[1].parse::<u64>().context("Invalid minor version")?;
    let patch = parts[2].parse::<u64>().context("Invalid patch version")?;
    Ok((major, minor, patch))
}

fn is_newer(remote: &str, local: &str) -> Result<bool> {
    let r = parse_version(remote)?;
    let l = parse_version(local)?;
    Ok(r > l)
}

pub fn run_update_command() -> Result<()> {
    println!();
    println!("  Melange Self-Update");
    println!("  Current version: {CURRENT_VERSION}");
    println!();

    // Step 1: Query GitHub Releases API
    print!("  Checking for updates...");
    let api_url = format!(
        "https://api.github.com/repos/{GITHUB_REPO}/releases/latest"
    );

    let mut response = ureq::get(&api_url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", &format!("melange/{CURRENT_VERSION}"))
        .call()
        .context("Failed to reach GitHub API. Check your internet connection.")?;

    let status = response.status();
    if status != 200 {
        if status == 404 {
            bail!("No releases found for {GITHUB_REPO}. The first release has not been published yet.");
        }
        bail!("GitHub API returned status {status}");
    }

    let body = response
        .body_mut()
        .read_to_string()
        .context("Failed to read GitHub API response")?;

    let release: GithubRelease =
        serde_json::from_str(&body).context("Failed to parse GitHub release JSON")?;

    println!(" found {}", release.tag_name);

    // Step 2: Compare versions
    if !is_newer(&release.tag_name, CURRENT_VERSION)? {
        println!();
        println!("  Already up to date.");
        println!();
        return Ok(());
    }

    println!("  New version available: {} -> {}", CURRENT_VERSION, release.tag_name);

    // Step 3: Find the correct asset for this platform
    let triple = target_triple()?;
    let expected_name = format!("melange-{}-{}.tar.gz", release.tag_name, triple);

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == expected_name)
        .with_context(|| {
            let available: Vec<&str> = release.assets.iter().map(|a| a.name.as_str()).collect();
            format!(
                "No asset found for this platform.\n  Expected: {expected_name}\n  Available: {available:?}"
            )
        })?;

    println!("  Downloading {}...", asset.name);

    // Step 4: Download the asset
    let mut download_response = ureq::get(&asset.browser_download_url)
        .header("User-Agent", &format!("melange/{CURRENT_VERSION}"))
        .call()
        .context("Failed to download release asset")?;

    let mut archive_bytes = Vec::new();
    download_response
        .body_mut()
        .as_reader()
        .read_to_end(&mut archive_bytes)
        .context("Failed to read downloaded archive")?;

    println!("  Downloaded {} bytes", archive_bytes.len());

    // Step 5: Extract the binary from tar.gz
    let gz_decoder = flate2::read::GzDecoder::new(&archive_bytes[..]);
    let mut archive = tar::Archive::new(gz_decoder);

    let mut binary_data: Option<Vec<u8>> = None;
    for entry_result in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry_result.context("Failed to read tar entry")?;
        let path = entry
            .path()
            .context("Failed to read entry path")?
            .to_path_buf();

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name == "melange" {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).context("Failed to read binary from archive")?;
            binary_data = Some(buf);
            break;
        }
    }

    let binary_data = binary_data.context(
        "Could not find 'melange' binary inside the downloaded archive"
    )?;

    println!("  Extracted binary ({} bytes)", binary_data.len());

    // Step 6: Replace self atomically
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("melange-update-tmp");
    std::fs::write(&temp_path, &binary_data)
        .context("Failed to write temporary binary")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755))
            .context("Failed to set executable permission on temp binary")?;
    }

    print!("  Replacing binary...");
    self_replace::self_replace(&temp_path)
        .context("Failed to replace the running binary")?;

    let _ = std::fs::remove_file(&temp_path);

    println!(" done.");
    println!();
    println!("  Updated to {}.", release.tag_name);
    println!("  The spice flows anew.");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("0.1.0").unwrap(), (0, 1, 0));
        assert_eq!(parse_version("v0.2.0").unwrap(), (0, 2, 0));
        assert_eq!(parse_version("v1.10.3").unwrap(), (1, 10, 3));
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("v0.2.0", "0.1.0").unwrap());
        assert!(!is_newer("v0.1.0", "0.1.0").unwrap());
        assert!(!is_newer("v0.0.9", "0.1.0").unwrap());
        assert!(is_newer("v1.0.0", "0.9.9").unwrap());
    }

    #[test]
    fn test_target_triple_succeeds() {
        let triple = target_triple().unwrap();
        assert!(triple.contains("apple-darwin") || triple.contains("unknown-linux"));
    }
}

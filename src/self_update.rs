use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const REPO: &str = "cinco/Tmuxido";
const BASE_URL: &str = "https://git.cincoeuzebio.com";

/// Check if running from cargo (development mode)
fn is_dev_build() -> bool {
    option_env!("CARGO_PKG_NAME").is_none()
}

/// Get current version from cargo
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Detect system architecture
fn detect_arch() -> Result<&'static str> {
    let arch = std::env::consts::ARCH;
    match arch {
        "x86_64" => Ok("tmuxido-x86_64-linux"),
        "aarch64" => Ok("tmuxido-aarch64-linux"),
        _ => Err(anyhow::anyhow!("Unsupported architecture: {}", arch)),
    }
}

/// Fetch latest release tag from Gitea API
fn fetch_latest_tag() -> Result<String> {
    let url = format!("{}/api/v1/repos/{}/releases?limit=1&page=1", BASE_URL, REPO);

    let output = Command::new("curl")
        .args(["-fsSL", &url])
        .output()
        .context("Failed to execute curl. Make sure curl is installed.")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch latest release: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let response = String::from_utf8_lossy(&output.stdout);

    // Parse JSON response to extract tag_name
    let tag: serde_json::Value =
        serde_json::from_str(&response).context("Failed to parse release API response")?;

    tag.get(0)
        .and_then(|r| r.get("tag_name"))
        .and_then(|t| t.as_str())
        .map(|t| t.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not extract tag_name from release"))
}

/// Get path to current executable
fn get_current_exe() -> Result<PathBuf> {
    std::env::current_exe().context("Failed to get current executable path")
}

/// Download binary to a temporary location
fn download_binary(tag: &str, arch: &str, temp_path: &std::path::Path) -> Result<()> {
    let url = format!("{}/{}/releases/download/{}/{}", BASE_URL, REPO, tag, arch);

    println!("Downloading {}...", url);

    let output = Command::new("curl")
        .args(["-fsSL", &url, "-o", &temp_path.to_string_lossy()])
        .output()
        .context("Failed to execute curl for download")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to download binary: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Make executable
    let mut perms = std::fs::metadata(temp_path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(temp_path, perms)?;

    Ok(())
}

/// Perform self-update
pub fn self_update() -> Result<()> {
    if is_dev_build() {
        println!("Development build detected. Skipping self-update.");
        return Ok(());
    }

    let current = current_version();
    println!("Current version: {}", current);

    let latest = fetch_latest_tag()?;
    let latest_clean = latest.trim_start_matches('v');
    println!("Latest version: {}", latest);

    // Compare versions (simple string comparison for semver without 'v' prefix)
    if latest_clean == current {
        println!("Already up to date!");
        return Ok(());
    }

    // Check if latest is actually newer
    match version_compare(latest_clean, current) {
        std::cmp::Ordering::Less => {
            println!("Current version is newer than release. Skipping update.");
            return Ok(());
        }
        std::cmp::Ordering::Equal => {
            println!("Already up to date!");
            return Ok(());
        }
        _ => {}
    }

    let arch = detect_arch()?;
    let exe_path = get_current_exe()?;

    // Create temporary file in same directory as target (for atomic rename)
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not determine executable directory"))?;
    let temp_path = exe_dir.join(".tmuxido.new");

    println!("Downloading update...");
    download_binary(&latest, arch, &temp_path)?;

    // Verify the downloaded binary works
    let verify = Command::new(&temp_path).arg("--version").output();
    if let Err(e) = verify {
        let _ = std::fs::remove_file(&temp_path);
        return Err(anyhow::anyhow!(
            "Downloaded binary verification failed: {}",
            e
        ));
    }

    // Atomic replace: rename old to .old, rename new to target
    let backup_path = exe_path.with_extension("old");

    // Remove old backup if exists
    let _ = std::fs::remove_file(&backup_path);

    // Rename current to backup
    std::fs::rename(&exe_path, &backup_path)
        .context("Failed to backup current binary (is tmuxido running?)")?;

    // Move new to current location
    if let Err(e) = std::fs::rename(&temp_path, &exe_path) {
        // Restore backup on failure
        let _ = std::fs::rename(&backup_path, &exe_path);
        return Err(anyhow::anyhow!("Failed to install new binary: {}", e));
    }

    // Remove backup on success
    let _ = std::fs::remove_file(&backup_path);

    println!("Successfully updated to {}!", latest);
    Ok(())
}

/// Compare two semver versions
fn version_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| {
        s.split('.')
            .filter_map(|n| n.parse::<u32>().ok())
            .collect::<Vec<_>>()
    };

    let a_parts = parse(a);
    let b_parts = parse(b);

    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        match a_part.cmp(b_part) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    a_parts.len().cmp(&b_parts.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_detect_current_version() {
        let version = current_version();
        // Version should be non-empty and contain dots
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    #[test]
    fn should_compare_versions_correctly() {
        assert_eq!(
            version_compare("0.3.0", "0.2.4"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(version_compare("0.2.4", "0.3.0"), std::cmp::Ordering::Less);
        assert_eq!(version_compare("0.3.0", "0.3.0"), std::cmp::Ordering::Equal);
        assert_eq!(
            version_compare("1.0.0", "0.9.9"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            version_compare("0.10.0", "0.9.0"),
            std::cmp::Ordering::Greater
        );
    }
}

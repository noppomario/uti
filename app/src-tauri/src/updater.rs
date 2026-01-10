//! Self-update functionality for uti
//!
//! This module provides functions to check for updates and install them
//! via GitHub Releases and RPM packages.

use reqwest::Client;
use semver::Version;
use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// GitHub API response for a release
#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

/// GitHub API response for a release asset
#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Result of update check
#[derive(Debug)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub uti_rpm_url: Option<String>,
    pub daemon_rpm_url: Option<String>,
    pub gnome_extension_url: Option<String>,
}

/// Error types for updater operations
#[derive(Debug)]
pub enum UpdateError {
    Network(String),
    Parse(String),
    Download(String),
    Install(String),
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Parse(msg) => write!(f, "Parse error: {}", msg),
            Self::Download(msg) => write!(f, "Download error: {}", msg),
            Self::Install(msg) => write!(f, "Install error: {}", msg),
        }
    }
}

impl std::error::Error for UpdateError {}

const GITHUB_API_URL: &str = "https://api.github.com/repos/noppomario/uti/releases/latest";
const USER_AGENT: &str = "uti-updater";

/// Check for updates from GitHub Releases
///
/// # Arguments
///
/// * `current_version` - The current version of the application
///
/// # Returns
///
/// Returns `UpdateCheckResult` with version comparison and download URLs
pub async fn check_for_updates(current_version: &str) -> Result<UpdateCheckResult, UpdateError> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    let response = client
        .get(GITHUB_API_URL)
        .send()
        .await
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(UpdateError::Network(format!(
            "GitHub API returned status: {}",
            response.status()
        )));
    }

    let release: Release = response
        .json()
        .await
        .map_err(|e| UpdateError::Parse(e.to_string()))?;

    // Parse version (strip 'v' prefix if present)
    let latest_version_str = release.tag_name.trim_start_matches('v');
    let current = Version::parse(current_version)
        .map_err(|e| UpdateError::Parse(format!("Invalid current version: {}", e)))?;
    let latest = Version::parse(latest_version_str)
        .map_err(|e| UpdateError::Parse(format!("Invalid latest version: {}", e)))?;

    let update_available = latest > current;

    // Find RPM URLs
    let uti_rpm_url = release
        .assets
        .iter()
        .find(|a| a.name.starts_with("uti-") && a.name.ends_with(".rpm"))
        .map(|a| a.browser_download_url.clone());

    let daemon_rpm_url = release
        .assets
        .iter()
        .find(|a| a.name.starts_with("uti-daemon-") && a.name.ends_with(".rpm"))
        .map(|a| a.browser_download_url.clone());

    let gnome_extension_url = release
        .assets
        .iter()
        .find(|a| a.name == "gnome-extension.zip")
        .map(|a| a.browser_download_url.clone());

    Ok(UpdateCheckResult {
        current_version: current_version.to_string(),
        latest_version: latest_version_str.to_string(),
        update_available,
        uti_rpm_url,
        daemon_rpm_url,
        gnome_extension_url,
    })
}

/// Download a file to /tmp directory
///
/// # Arguments
///
/// * `url` - The URL to download from
/// * `filename` - The filename to save as
///
/// # Returns
///
/// Returns the path to the downloaded file
pub async fn download_rpm(url: &str, filename: &str) -> Result<PathBuf, UpdateError> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| UpdateError::Download(e.to_string()))?;

    if !response.status().is_success() {
        return Err(UpdateError::Download(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| UpdateError::Download(e.to_string()))?;

    let path = PathBuf::from(format!("/tmp/{}", filename));
    let mut file = std::fs::File::create(&path)
        .map_err(|e| UpdateError::Download(format!("Failed to create file: {}", e)))?;

    file.write_all(&bytes)
        .map_err(|e| UpdateError::Download(format!("Failed to write file: {}", e)))?;

    Ok(path)
}

/// Download a file to /tmp directory (generic version for any file type)
///
/// # Arguments
///
/// * `url` - The URL to download from
/// * `filename` - The filename to save as
///
/// # Returns
///
/// Returns the path to the downloaded file
pub async fn download_file(url: &str, filename: &str) -> Result<PathBuf, UpdateError> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| UpdateError::Download(e.to_string()))?;

    if !response.status().is_success() {
        return Err(UpdateError::Download(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| UpdateError::Download(e.to_string()))?;

    let path = PathBuf::from(format!("/tmp/{}", filename));
    let mut file = std::fs::File::create(&path)
        .map_err(|e| UpdateError::Download(format!("Failed to create file: {}", e)))?;

    file.write_all(&bytes)
        .map_err(|e| UpdateError::Download(format!("Failed to write file: {}", e)))?;

    Ok(path)
}

/// Install GNOME extension from a zip file
///
/// # Arguments
///
/// * `zip_path` - The path to the zip file
///
/// # Returns
///
/// Returns `Ok(())` if installation was successful
pub fn install_gnome_extension(zip_path: &PathBuf) -> Result<(), UpdateError> {
    let ext_uuid = "uti@noppomario.github.io";
    let ext_dir = dirs::data_dir()
        .ok_or_else(|| UpdateError::Install("Could not find data directory".to_string()))?
        .join("gnome-shell/extensions")
        .join(ext_uuid);

    // Create extension directory
    std::fs::create_dir_all(&ext_dir).map_err(|e| {
        UpdateError::Install(format!("Failed to create extension directory: {}", e))
    })?;

    // Extract zip to extension directory
    let output = Command::new("unzip")
        .args(["-o", "-q"])
        .arg(zip_path)
        .arg("-d")
        .arg(&ext_dir)
        .output()
        .map_err(|e| UpdateError::Install(format!("Failed to run unzip: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UpdateError::Install(format!("unzip failed: {}", stderr)));
    }

    // Compile schemas if directory exists
    let schema_dir = ext_dir.join("schemas");
    if schema_dir.exists() {
        let _ = Command::new("glib-compile-schemas")
            .arg(&schema_dir)
            .output();
    }

    Ok(())
}

/// Install multiple RPM packages in a single pkexec session
///
/// This function installs all RPMs in one authentication prompt,
/// similar to how install.sh handles it.
///
/// # Arguments
///
/// * `rpm_paths` - Slice of paths to RPM files
///
/// # Returns
///
/// Returns `Ok(())` if installation was successful
pub fn install_rpms(rpm_paths: &[PathBuf]) -> Result<(), UpdateError> {
    if rpm_paths.is_empty() {
        return Ok(());
    }

    let mut cmd = Command::new("pkexec");
    cmd.args(["dnf", "install", "-y"]);
    for path in rpm_paths {
        cmd.arg(path);
    }

    let output = cmd
        .output()
        .map_err(|e| UpdateError::Install(format!("Failed to run pkexec: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UpdateError::Install(format!(
            "dnf install failed: {}",
            stderr
        )));
    }

    Ok(())
}

/// Perform a full update (download and install RPMs and GNOME extension)
///
/// # Arguments
///
/// * `result` - The update check result containing download URLs
///
/// # Returns
///
/// Returns `Ok(())` if update was successful
pub async fn perform_update(result: &UpdateCheckResult) -> Result<(), UpdateError> {
    let mut rpm_paths: Vec<PathBuf> = Vec::new();

    // Download daemon RPM
    if let Some(ref daemon_url) = result.daemon_rpm_url {
        println!("Downloading daemon RPM...");
        let daemon_path = download_rpm(
            daemon_url,
            &format!("uti-daemon-{}.rpm", result.latest_version),
        )
        .await?;
        rpm_paths.push(daemon_path);
    }

    // Download uti app RPM
    if let Some(ref uti_url) = result.uti_rpm_url {
        println!("Downloading uti RPM...");
        let uti_path = download_rpm(uti_url, &format!("uti-{}.rpm", result.latest_version)).await?;
        rpm_paths.push(uti_path);
    }

    // Install all RPMs in a single pkexec session (one authentication prompt)
    if !rpm_paths.is_empty() {
        println!("Installing RPM packages...");
        install_rpms(&rpm_paths)?;
    }

    // Download and install GNOME extension (if available and on GNOME)
    if let Some(ref ext_url) = result.gnome_extension_url {
        if is_gnome_environment() {
            println!("Downloading GNOME extension...");
            let ext_path = download_file(ext_url, "gnome-extension.zip").await?;
            println!("Installing GNOME extension...");
            install_gnome_extension(&ext_path)?;
            println!("GNOME extension updated. Log out and log back in to apply changes.");
        }
    }

    Ok(())
}

/// Check if running in a GNOME environment
fn is_gnome_environment() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| v.to_uppercase().contains("GNOME"))
        .unwrap_or(false)
        || std::path::Path::new("/usr/bin/gnome-extensions").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        // Test that semver parsing works correctly
        let v1 = Version::parse("0.1.0").unwrap();
        let v2 = Version::parse("0.2.0").unwrap();
        assert!(v2 > v1);

        let v3 = Version::parse("1.0.0").unwrap();
        assert!(v3 > v2);
    }
}

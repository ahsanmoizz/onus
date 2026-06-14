//! `onus upgrade` — download and install the latest version.

/// Download URL for the latest Onus release binary.
fn download_url() -> String {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "windows"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    };

    let ext = if cfg!(target_os = "windows") {
        "exe"
    } else {
        "tar.gz"
    };

    format!(
        "https://github.com/Gitlawb/onus/releases/latest/download/onus-{}-{}.{}",
        os, arch, ext
    )
}

/// Run the upgrade command.
pub fn run() -> anyhow::Result<()> {
    let current_exe = std::env::current_exe()?;
    let url = download_url();

    println!("Onus v{}", crate::VERSION);
    println!("Checking for updates...");
    println!("  Current: {}", current_exe.display());
    println!("  Latest:  {}", url);
    println!();

    // Try to fetch the latest version info from GitHub.
    let version_url = "https://api.github.com/repos/Gitlawb/onus/releases/latest";
    let version_info = fetch_url(version_url);
    match version_info {
        Ok(body) => {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(tag) = json["tag_name"].as_str() {
                    println!("  Latest release: {}", tag);
                    if tag == format!("v{}", crate::VERSION) || tag == crate::VERSION {
                        println!();
                        println!("Already up to date ({}).", tag);
                        return Ok(());
                    }
                }
            }
        }
        Err(_) => {
            println!("  (could not check GitHub for latest version)");
        }
    }

    println!();
    println!("To upgrade:");
    println!("  1. Download: {}", url);
    println!("  2. Replace: {}", current_exe.display());
    println!();
    println!("  Or re-run the installer:");
    println!("     curl -fsSL https://github.com/Gitlawb/onus/releases/latest/download/install.sh | bash");
    println!();
    println!("  Config and rules are preserved during upgrade.");
    println!("  Restart the daemon after upgrading:");
    println!("     onus daemon restart");

    Ok(())
}

/// Fetch a URL using curl, wget, or PowerShell.
fn fetch_url(url: &str) -> anyhow::Result<String> {
    if let Ok(output) = std::process::Command::new("curl")
        .args(["-fsSL", "-H", "Accept: application/json", url])
        .output()
    {
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout)?);
        }
    }

    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Invoke-WebRequest -Uri '{}' -Headers @{{'Accept'='application/json'}}).Content", url),
            ])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8(output.stdout)?);
            }
        }
    }

    Err(anyhow::anyhow!(
        "Could not fetch URL (no curl or powershell available)"
    ))
}

//! Google Antigravity integration helpers.
//!
//! Current supported Onus path:
//! - L2 ROUTED ONLY for MCP traffic explicitly configured through `onus mcp-proxy`.
//! - L1 BEST-EFFORT for future cooperative hooks. A hook bridge is not claimed here.
//!
//! Direct Antigravity agent actions that do not route through Onus remain outside
//! Onus control.

use clap::{Args, ValueEnum};
use std::path::{Path, PathBuf};

const SERVER_NAME: &str = "onus-mcp-proxy";

#[derive(Debug)]
pub enum AntigravityCheck {
    Available { version: String, path: PathBuf },
    NotFound,
    Error(String),
}

#[derive(Debug)]
pub enum ExtensionCheck {
    Installed { path: PathBuf, version: String },
    NotInstalled,
    Error(String),
}

#[derive(Debug)]
pub enum McpConfigCheck {
    Configured {
        server_name: String,
        config_paths: Vec<PathBuf>,
    },
    NotFound,
    Error(String),
}

#[derive(Clone, Debug, ValueEnum)]
pub enum AntigravitySurface {
    /// Configure both Antigravity CLI and IDE MCP config locations.
    All,
    /// Configure Antigravity CLI's dedicated MCP profile.
    Cli,
    /// Configure Antigravity IDE's MCP profile.
    Ide,
}

#[derive(Args)]
pub struct AntigravityMcpArgs {
    /// Upstream MCP server binary that Onus should wrap.
    #[arg(long)]
    pub server: PathBuf,

    /// Which Antigravity surface to configure.
    #[arg(long, value_enum, default_value_t = AntigravitySurface::All)]
    pub surface: AntigravitySurface,

    /// Arguments passed to the upstream MCP server after `--`.
    #[arg(last = true)]
    pub args: Vec<String>,
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string())
        .into()
}

pub fn antigravity_mcp_config_path() -> PathBuf {
    antigravity_cli_mcp_config_path()
}

pub fn antigravity_cli_mcp_config_path() -> PathBuf {
    home_dir()
        .join(".gemini")
        .join("antigravity-cli")
        .join("mcp_config.json")
}

pub fn antigravity_ide_mcp_config_path() -> PathBuf {
    home_dir()
        .join(".gemini")
        .join("antigravity")
        .join("mcp_config.json")
}

fn legacy_antigravity_mcp_config_path() -> PathBuf {
    home_dir()
        .join(".gemini")
        .join("config")
        .join("mcp_config.json")
}

fn selected_config_paths(surface: &AntigravitySurface) -> Vec<PathBuf> {
    match surface {
        AntigravitySurface::All => vec![
            antigravity_cli_mcp_config_path(),
            antigravity_ide_mcp_config_path(),
        ],
        AntigravitySurface::Cli => vec![antigravity_cli_mcp_config_path()],
        AntigravitySurface::Ide => vec![antigravity_ide_mcp_config_path()],
    }
}

fn known_config_paths() -> Vec<PathBuf> {
    vec![
        antigravity_cli_mcp_config_path(),
        antigravity_ide_mcp_config_path(),
    ]
}

fn candidate_binaries() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            for name in ["agy", "antigravity"] {
                candidates.push(dir.join(name));
                #[cfg(windows)]
                {
                    candidates.push(dir.join(format!("{name}.exe")));
                    candidates.push(dir.join(format!("{name}.cmd")));
                }
            }
        }
    }

    #[cfg(windows)]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            candidates.push(
                PathBuf::from(local_app_data.clone())
                    .join("Programs")
                    .join("Antigravity")
                    .join("bin")
                    .join("agy.cmd"),
            );
            candidates.push(
                PathBuf::from(local_app_data)
                    .join("Programs")
                    .join("Antigravity")
                    .join("bin")
                    .join("antigravity.cmd"),
            );
        }
        candidates.push(PathBuf::from(
            "C:\\Program Files\\Google\\Antigravity\\bin\\agy.cmd",
        ));
        candidates.push(PathBuf::from(
            "C:\\Program Files\\Google\\Antigravity\\bin\\antigravity.cmd",
        ));
        candidates.push(PathBuf::from("D:\\Antigravity\\bin\\agy.cmd"));
        candidates.push(PathBuf::from("D:\\Antigravity\\bin\\antigravity.cmd"));
    }

    candidates
}

pub fn find_antigravity() -> AntigravityCheck {
    for path in candidate_binaries() {
        if !path.is_file() {
            continue;
        }
        match get_version(&path) {
            Ok(version) => return AntigravityCheck::Available { version, path },
            Err(_) => continue,
        }
    }

    AntigravityCheck::NotFound
}

fn get_version(path: &Path) -> Result<String, String> {
    let output =
        command_output_with_timeout(path, &["--version"], std::time::Duration::from_secs(3))?;

    if !output.status.success() {
        return Err(format!(
            "{} --version exited with {}",
            path.display(),
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let version = if !stdout.is_empty() { stdout } else { stderr };
    let version = version.lines().next().unwrap_or("").trim().to_string();
    Ok(if version.is_empty() {
        "unknown".to_string()
    } else {
        version
    })
}

fn command_output_with_timeout(
    path: &Path,
    args: &[&str],
    timeout: std::time::Duration,
) -> Result<std::process::Output, String> {
    let mut child = std::process::Command::new(path)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run {} {}: {e}", path.display(), args.join(" ")))?;

    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                return child
                    .wait_with_output()
                    .map_err(|e| format!("cannot read {} output: {e}", path.display()));
            }
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{} {} timed out after {} ms",
                    path.display(),
                    args.join(" "),
                    timeout.as_millis()
                ));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("cannot poll {}: {e}", path.display()));
            }
        }
    }
}

pub fn check_extension_installed(_antigravity_path: &PathBuf) -> ExtensionCheck {
    ExtensionCheck::NotInstalled
}

pub fn install_extension(_antigravity_path: &PathBuf, _vsix_path: &PathBuf) -> anyhow::Result<()> {
    anyhow::bail!(
        "Antigravity extension installation is not implemented. Current supported path is MCP proxy routing only."
    )
}

pub fn uninstall_extension(_antigravity_path: &PathBuf) -> anyhow::Result<()> {
    Ok(())
}

fn proxy_entry(
    onus_path: &Path,
    upstream_server: &Path,
    upstream_args: &[String],
) -> serde_json::Value {
    let mut args = vec![
        "mcp-proxy".to_string(),
        "--experimental".to_string(),
        "--server".to_string(),
        upstream_server.to_string_lossy().to_string(),
    ];
    if !upstream_args.is_empty() {
        args.push("--".to_string());
        args.extend(upstream_args.iter().cloned());
    }

    serde_json::json!({
        "command": onus_path.to_string_lossy(),
        "args": args,
        "description": "Onus MCP gateway. L2 ROUTED ONLY: only traffic sent through this proxy is governed."
    })
}

fn has_valid_onus_proxy(entry: &serde_json::Value) -> bool {
    let command_ok = entry
        .get("command")
        .and_then(serde_json::Value::as_str)
        .map(|command| command.to_ascii_lowercase().contains("onus"))
        .unwrap_or(false);
    let args = entry.get("args").and_then(serde_json::Value::as_array);
    let args_ok = args
        .map(|args| {
            let values: Vec<&str> = args.iter().filter_map(serde_json::Value::as_str).collect();
            values.contains(&"mcp-proxy")
                && values.contains(&"--experimental")
                && values.contains(&"--server")
                && values
                    .iter()
                    .position(|value| *value == "--server")
                    .and_then(|index| values.get(index + 1))
                    .is_some()
        })
        .unwrap_or(false);
    command_ok && args_ok
}

pub fn check_mcp_config(_antigravity_path: &PathBuf) -> McpConfigCheck {
    let mut configured = Vec::new();
    let mut errors = Vec::new();

    for config_path in known_config_paths() {
        if !config_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(e) => {
                errors.push(format!("cannot read {}: {e}", config_path.display()));
                continue;
            }
        };
        if content.trim().is_empty() {
            continue;
        }

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(json) => json,
            Err(e) => {
                errors.push(format!("cannot parse {}: {e}", config_path.display()));
                continue;
            }
        };

        let Some(entry) = json
            .get("mcpServers")
            .and_then(serde_json::Value::as_object)
            .and_then(|servers| servers.get(SERVER_NAME))
        else {
            continue;
        };

        if has_valid_onus_proxy(entry) {
            configured.push(config_path);
        } else {
            errors.push(format!(
                "{} entry must run `onus mcp-proxy --experimental --server <upstream>`",
                config_path.display()
            ));
        }
    }

    if !errors.is_empty() {
        return McpConfigCheck::Error(errors.join("; "));
    }

    if configured.is_empty() {
        McpConfigCheck::NotFound
    } else {
        McpConfigCheck::Configured {
            server_name: SERVER_NAME.to_string(),
            config_paths: configured,
        }
    }
}

pub fn add_mcp_server(
    _antigravity_path: &PathBuf,
    onus_path: &Path,
    upstream_server: &Path,
    upstream_args: &[String],
    surface: &AntigravitySurface,
) -> anyhow::Result<()> {
    if !upstream_server.exists() {
        anyhow::bail!(
            "upstream MCP server not found at {}",
            upstream_server.display()
        );
    }

    for config_path in selected_config_paths(surface) {
        add_mcp_server_at(&config_path, onus_path, upstream_server, upstream_args)?;
    }

    Ok(())
}

fn add_mcp_server_at(
    config_path: &Path,
    onus_path: &Path,
    upstream_server: &Path,
    upstream_args: &[String],
) -> anyhow::Result<()> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !config.is_object() {
        config = serde_json::json!({});
    }
    let obj = config.as_object_mut().unwrap();
    let servers = obj
        .entry("mcpServers".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !servers.is_object() {
        *servers = serde_json::json!({});
    }
    servers.as_object_mut().unwrap().insert(
        SERVER_NAME.to_string(),
        proxy_entry(onus_path, upstream_server, upstream_args),
    );

    std::fs::write(config_path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

pub fn run_setup() -> anyhow::Result<()> {
    println!("Onus Setup - Google Antigravity");
    println!();

    match find_antigravity() {
        AntigravityCheck::Available { version, path } => {
            println!(
                "  Antigravity CLI found: v{} at {}",
                version,
                path.display()
            );
            println!("  Current supported Onus path: L2 ROUTED ONLY via MCP proxy.");
            println!();
            match check_mcp_config(&path) {
                McpConfigCheck::Configured {
                    server_name,
                    config_paths,
                } => {
                    println!("  MCP proxy configured: '{}'", server_name);
                    for path in config_paths {
                        println!("  Config file: {}", path.display());
                    }
                }
                McpConfigCheck::NotFound => {
                    println!("  MCP proxy not configured.");
                    println!("  Config files expected at:");
                    for path in known_config_paths() {
                        println!("    {}", path.display());
                    }
                    println!();
                    println!("  To configure a real routed MCP server, run:");
                    println!("    onus antigravity-mcp --server <UPSTREAM_MCP_SERVER> -- <UPSTREAM_ARGS>");
                    println!();
                    println!("  Until then, Antigravity remains UNVERIFIED for Onus protection.");
                }
                McpConfigCheck::Error(e) => {
                    println!("  MCP config check failed: {}", e);
                }
            }
        }
        AntigravityCheck::NotFound => {
            println!("  Antigravity CLI not found.");
            println!("  Install Google Antigravity, then rerun `onus setup --antigravity`.");
        }
        AntigravityCheck::Error(e) => {
            println!("  Antigravity check failed: {}", e);
        }
    }

    println!();
    println!("  Direct Antigravity actions outside an Onus-routed MCP proxy are not controlled.");
    Ok(())
}

pub fn run_mcp_setup(args: AntigravityMcpArgs) -> anyhow::Result<()> {
    let onus_path = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("cannot determine onus binary path: {e}"))?;

    let antigravity_path = match find_antigravity() {
        AntigravityCheck::Available { path, version } => {
            println!("Antigravity CLI found: v{} at {}", version, path.display());
            path
        }
        AntigravityCheck::NotFound => {
            println!(
                "Warning: Antigravity CLI not found on PATH; writing MCP config files anyway."
            );
            PathBuf::from("antigravity")
        }
        AntigravityCheck::Error(e) => {
            println!("Warning: Antigravity check failed ({e}); writing MCP config files anyway.");
            PathBuf::from("antigravity")
        }
    };

    add_mcp_server(
        &antigravity_path,
        &onus_path,
        &args.server,
        &args.args,
        &args.surface,
    )?;

    println!("Antigravity MCP proxy configured.");
    for config_path in selected_config_paths(&args.surface) {
        println!("  Config: {}", config_path.display());
    }
    println!("  Server: {}", args.server.display());
    println!("  Enforcement: L2 ROUTED ONLY");
    println!();
    println!("Only MCP calls routed through this proxy are governed by Onus.");
    Ok(())
}

pub fn run_uninstall() -> anyhow::Result<()> {
    let mut paths = known_config_paths();
    paths.push(legacy_antigravity_mcp_config_path());
    let mut touched = false;

    for config_path in paths {
        if !config_path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&config_path)?;
        let mut config: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(servers) = config
            .get_mut("mcpServers")
            .and_then(serde_json::Value::as_object_mut)
        {
            servers.remove(SERVER_NAME);
            if servers.is_empty() {
                config.as_object_mut().unwrap().remove("mcpServers");
            }
        }

        if config.as_object().is_none_or(|obj| obj.is_empty()) {
            std::fs::remove_file(&config_path)?;
            println!(
                "Removed empty Antigravity MCP config: {}",
                config_path.display()
            );
        } else {
            std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
            println!("Removed Onus MCP proxy from {}", config_path.display());
        }
        touched = true;
    }

    if !touched {
        println!("No Antigravity MCP config found.");
    }

    Ok(())
}

pub fn run_doctor() -> anyhow::Result<()> {
    println!("Onus Doctor - Google Antigravity");
    println!();

    match find_antigravity() {
        AntigravityCheck::Available { version, path } => {
            println!(
                "  [OK]  Antigravity CLI: v{} at {}",
                version,
                path.display()
            );
            match check_mcp_config(&path) {
                McpConfigCheck::Configured {
                    server_name,
                    config_paths,
                } => {
                    println!(
                        "  [OK]  MCP proxy: '{}' configured for {} surface(s)",
                        server_name,
                        config_paths.len()
                    );
                    for path in config_paths {
                        println!("        Config: {}", path.display());
                    }
                    println!("        Enforcement label: L2 ROUTED ONLY");
                }
                McpConfigCheck::NotFound => {
                    println!("  [WARN] MCP proxy: not configured");
                    for path in known_config_paths() {
                        println!("        Missing: {}", path.display());
                    }
                }
                McpConfigCheck::Error(e) => {
                    println!("  [FAIL] MCP proxy: {}", e);
                }
            }
        }
        AntigravityCheck::NotFound => {
            println!("  [WARN] Antigravity CLI: not installed or not on PATH");
        }
        AntigravityCheck::Error(e) => {
            println!("  [FAIL] Antigravity CLI: {}", e);
        }
    }

    println!();
    println!(
        "  Limit: direct Antigravity actions can bypass Onus unless routed through the proxy."
    );
    Ok(())
}

pub fn l3_workspace_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("bwrap")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

pub fn l3_workspace_advice() -> String {
    if l3_workspace_available() {
        "bubblewrap available - use `onus run --isolate -- <agent command>` for L3 workspaces."
            .to_string()
    } else {
        "not available on this platform (requires Linux + bubblewrap)".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    struct HomeGuard {
        _lock: MutexGuard<'static, ()>,
        old_home: Option<String>,
        old_userprofile: Option<String>,
    }

    impl HomeGuard {
        fn set(root: &Path) -> Self {
            static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            let guard = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
            let old_home = std::env::var("HOME").ok();
            let old_userprofile = std::env::var("USERPROFILE").ok();
            std::env::set_var("HOME", root);
            std::env::set_var("USERPROFILE", root);
            Self {
                _lock: guard,
                old_home,
                old_userprofile,
            }
        }
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            if let Some(value) = self.old_home.take() {
                std::env::set_var("HOME", value);
            } else {
                std::env::remove_var("HOME");
            }
            if let Some(value) = self.old_userprofile.take() {
                std::env::set_var("USERPROFILE", value);
            } else {
                std::env::remove_var("USERPROFILE");
            }
        }
    }

    #[test]
    fn test_find_antigravity_no_panic() {
        let _ = find_antigravity();
    }

    #[test]
    fn test_antigravity_mcp_config_path_format() {
        let path = antigravity_mcp_config_path();
        let rendered = path.to_string_lossy();
        assert!(rendered.contains(".gemini"));
        assert!(rendered.contains("antigravity-cli"));
        assert!(rendered.contains("mcp_config.json"));
    }

    #[test]
    fn test_proxy_entry_requires_upstream_server() {
        let entry = proxy_entry(
            Path::new("/usr/local/bin/onus"),
            Path::new("/usr/local/bin/example-mcp"),
            &["--flag".to_string()],
        );
        assert!(has_valid_onus_proxy(&entry));
        let args = entry["args"].as_array().unwrap();
        assert!(args.iter().any(|v| v.as_str() == Some("--server")));
        assert!(args
            .iter()
            .any(|v| v.as_str() == Some("/usr/local/bin/example-mcp")));
    }

    #[test]
    fn test_proxy_entry_rejects_missing_server_arg() {
        let entry = serde_json::json!({
            "command": "/usr/local/bin/onus",
            "args": ["mcp-proxy", "--experimental"]
        });
        assert!(!has_valid_onus_proxy(&entry));
    }

    #[test]
    fn test_add_mcp_server_writes_cli_and_ide_antigravity_configs() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("onus-antigravity-test-{stamp}"));
        std::fs::create_dir_all(&root).unwrap();
        let upstream = root.join("fake-mcp-server");
        std::fs::write(&upstream, "").unwrap();

        let _home = HomeGuard::set(&root);

        add_mcp_server(
            &PathBuf::from("antigravity"),
            &PathBuf::from("/usr/local/bin/onus"),
            &upstream,
            &["--demo".to_string()],
            &AntigravitySurface::All,
        )
        .unwrap();

        for config_path in [
            root.join(".gemini")
                .join("antigravity-cli")
                .join("mcp_config.json"),
            root.join(".gemini")
                .join("antigravity")
                .join("mcp_config.json"),
        ] {
            let content = std::fs::read_to_string(config_path).unwrap();
            let config: serde_json::Value = serde_json::from_str(&content).unwrap();
            let entry = &config["mcpServers"][SERVER_NAME];
            assert!(has_valid_onus_proxy(entry));
            assert_eq!(entry["args"][4], "--");
            assert_eq!(entry["args"][5], "--demo");
        }

        match check_mcp_config(&PathBuf::from("antigravity")) {
            McpConfigCheck::Configured { config_paths, .. } => {
                assert_eq!(config_paths.len(), 2);
            }
            other => panic!("expected configured Antigravity MCP paths, got {other:?}"),
        }

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn test_extension_install_is_explicitly_unsupported() {
        let result = install_extension(&PathBuf::from("agy"), &PathBuf::from("onus.vsix"));
        assert!(result.is_err());
    }

    #[test]
    fn test_l3_workspace_advice_format() {
        let advice = l3_workspace_advice();
        assert!(!advice.is_empty());
        assert!(advice.contains("bubblewrap") || advice.contains("L3"));
    }
}

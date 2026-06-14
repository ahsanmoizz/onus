//! `onus shell` — install or remove the shell wrapper for terminal-based agents.
//!
//! Installs a shell hook that intercepts every command through Onus Core,
//! blocking dangerous actions before they execute. Supports bash and zsh.

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ShellArgs {
    /// Action: install or remove
    #[arg(default_value = "install")]
    pub action: String,

    /// Path to write the shell wrapper script (default: config_dir/scripts/)
    #[arg(long)]
    pub path: Option<PathBuf>,
}

pub fn run(args: ShellArgs) -> anyhow::Result<()> {
    match args.action.as_str() {
        "install" => install(args.path),
        "remove" | "uninstall" => remove(args.path),
        other => anyhow::bail!("Unknown action: {other}. Use 'install' or 'remove'."),
    }
}

fn scripts_dir(custom_path: Option<PathBuf>) -> PathBuf {
    custom_path.unwrap_or_else(|| crate::config_dir().join("scripts"))
}

fn install(custom_path: Option<PathBuf>) -> anyhow::Result<()> {
    let scripts = scripts_dir(custom_path);
    std::fs::create_dir_all(&scripts)?;

    // Copy the shell wrapper script from the embedded resource or write fallback.
    let script_path = scripts.join("onus-shell-wrapper.sh");

    // Write the shell wrapper content.
    let content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/scripts/onus-shell-wrapper.sh"
    ));
    std::fs::write(&script_path, content)?;
    println!("Onus shell wrapper installed at: {}", script_path.display());

    // Add source line to .bashrc / .zshrc.
    let source_line = format!("\n# Onus shell wrapper — intercepts commands through the AI agent firewall\nsource \"{}\"\n", script_path.display());

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let home_path = PathBuf::from(home);

    let rc_files = [home_path.join(".bashrc"), home_path.join(".zshrc")];
    let mut installed_count = 0;

    for rc in &rc_files {
        if rc.exists() {
            let content = std::fs::read_to_string(rc)?;
            if content.contains("onus-shell-wrapper.sh") {
                println!("  Already sourced in {}", rc.display());
            } else {
                std::fs::write(rc, format!("{}{}", content, source_line))?;
                println!("  Added to {}", rc.display());
                installed_count += 1;
            }
        } else {
            // Create if it doesn't exist.
            std::fs::write(rc, &source_line)?;
            println!("  Created {} with Onus shell wrapper", rc.display());
            installed_count += 1;
        }
    }

    if installed_count > 0 {
        println!();
        println!("Onus shell wrapper installed!");
        println!(
            "Restart your terminal or run: source {}",
            script_path.display()
        );
        println!();
        println!("To start tracking a session:");
        println!(
            "  source {} && onus_shell_start \"my task\"",
            script_path.display()
        );
        println!();
        println!("To disable temporarily: export ONUS_SHELL_ENABLED=0");
    }

    Ok(())
}

fn remove(custom_path: Option<PathBuf>) -> anyhow::Result<()> {
    let scripts = scripts_dir(custom_path);
    let script_path = scripts.join("onus-shell-wrapper.sh");

    // Remove the script file.
    if script_path.exists() {
        std::fs::remove_file(&script_path)?;
        println!("Removed: {}", script_path.display());
    }

    // Remove source lines from rc files.
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let home_path = PathBuf::from(home);

    let rc_files = [home_path.join(".bashrc"), home_path.join(".zshrc")];
    for rc in &rc_files {
        if rc.exists() {
            let content = std::fs::read_to_string(rc)?;
            let new_content: String = content
                .lines()
                .filter(|line| !line.contains("onus-shell-wrapper.sh"))
                .collect::<Vec<_>>()
                .join("\n");
            if new_content != content {
                std::fs::write(rc, &new_content)?;
                println!("  Removed from {}", rc.display());
            }
        }
    }

    println!("Onus shell wrapper removed.");
    Ok(())
}

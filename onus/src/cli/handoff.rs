//! `onus handoff` — Cross-agent handoff manifest management.
//!
//! Subcommands:
//!   create  — create a handoff manifest for a session
//!   import  — import (validate) a handoff manifest from file
//!   show    — display a handoff manifest

use crate::handoff::{HandoffManifestBuilder, HandoffManifestV1, TaskContractSnapshot};
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct HandoffArgs {
    #[command(subcommand)]
    pub command: HandoffCommand,
}

#[derive(Subcommand)]
pub enum HandoffCommand {
    /// Create a handoff manifest for a session
    Create(CreateArgs),
    /// Import and validate a handoff manifest from file
    Import(ImportArgs),
    /// Display a handoff manifest
    Show(ShowArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    /// Session ID to hand off
    #[arg(long)]
    pub session: String,

    /// Source surface (claude-code-cli, codex-cli, cursor-ide, antigravity)
    #[arg(long, default_value = "claude_code_cli")]
    pub source: String,

    /// Target surface (claude-code-cli, codex-cli, cursor-ide, antigravity, any)
    #[arg(long, default_value = "any")]
    pub target: String,

    /// Human-readable reason for the handoff
    #[arg(long, default_value = "Cross-agent continuity handoff")]
    pub reason: String,

    /// Workspace root path (auto-detected if not provided)
    #[arg(long)]
    pub workspace: Option<PathBuf>,

    /// Output file path (default: handoff-<session>.json in current dir)
    #[arg(long, short)]
    pub output: Option<PathBuf>,

    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Task contract file (JSON) to include in manifest
    #[arg(long)]
    pub contract: Option<PathBuf>,
}

#[derive(Args)]
pub struct ImportArgs {
    /// Path to the handoff manifest JSON file
    pub path: PathBuf,

    /// Public key (hex) for signature verification (optional)
    #[arg(long)]
    pub public_key: Option<String>,
}

#[derive(Args)]
pub struct ShowArgs {
    /// Path to the handoff manifest JSON file
    pub path: PathBuf,
}

// ── Command implementations ──────────────────────────────────────────────────

pub fn run_create(args: CreateArgs) -> anyhow::Result<()> {
    let session = &args.session;

    // Gather git state if in a git repo
    let (git_head, git_branch) = detect_git_state(&args.workspace);

    // Gather contract if provided
    let contract_snapshot = if let Some(contract_path) = &args.contract {
        let json = std::fs::read_to_string(contract_path)?;
        Some(serde_json::from_str::<TaskContractSnapshot>(&json)?)
    } else {
        None
    };

    let builder = HandoffManifestBuilder::new()
        .source_surface(&args.source)
        .target_surface(&args.target)
        .reason(&args.reason)
        .session_id(session)
        .workspace_root(
            args.workspace
                .as_ref()
                .and_then(|p| p.to_str())
                .unwrap_or("."),
        );

    let builder = if let Some(ref h) = git_head {
        builder.git_head_hash(h)
    } else {
        builder
    };

    let builder = if let Some(ref b) = git_branch {
        builder.git_branch(b)
    } else {
        builder
    };

    let builder = if let Some(tc) = contract_snapshot {
        builder.task_contract(tc)
    } else {
        builder
    };

    let manifest = builder
        .build()
        .ok_or_else(|| anyhow::anyhow!("Failed to build manifest — session_id is required"))?;

    let output_path = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("handoff-{}.json", session)));

    manifest.to_file(&output_path)?;

    println!("Handoff manifest created:");
    println!("  Schema version: {}", manifest.schema_version);
    println!("  Source: {}", manifest.source_surface);
    println!("  Target: {}", manifest.target_surface);
    println!("  Session: {}", manifest.session_id);
    println!("  Workspace: {}", manifest.workspace_root.as_deref().unwrap_or("(none)"));
    println!("  Actions: {}", manifest.action_count);
    println!("  Hash: {}", &manifest.canonical_hash[..16]);
    println!("  Output: {}", output_path.display());
    println!();
    println!("  Run `onus handoff import {}` on the target surface.", output_path.display());

    Ok(())
}

pub fn run_import(args: ImportArgs) -> anyhow::Result<()> {
    let manifest = HandoffManifestV1::from_file(&args.path)?;

    println!("Handoff manifest loaded:");
    println!("  Schema version: {}", manifest.schema_version);
    println!("  Source: {}", manifest.source_surface);
    println!("  Target: {}", manifest.target_surface);
    println!("  Session: {}", manifest.session_id);
    println!();

    // Verify hash integrity
    if manifest.verify_hash() {
        println!("  [OK]  Hash integrity: canonical hash matches content");
    } else {
        println!("  [FAIL] Hash integrity: MANIFEST TAMPERED — canonical hash does not match content");
        return Err(anyhow::anyhow!("Manifest hash verification failed — file may be tampered"));
    }

    // Verify signature if public key provided
    if let Some(pk_hex) = &args.public_key {
        let pk_bytes = hex::decode(pk_hex)
            .map_err(|_| anyhow::anyhow!("Invalid public key hex"))?;
        if manifest.verify(Some(&pk_bytes)) {
            println!("  [OK]  Signature: valid (verified with provided public key)");
        } else {
            println!("  [FAIL] Signature: INVALID — signature does not match public key");
            return Err(anyhow::anyhow!("Manifest signature verification failed"));
        }
    } else {
        println!("  [WARN] No public key provided — signature was not verified");
    }

    // Display summary of what's being handed off
    println!();
    println!("Continuity summary:");
    if let Some(ref tc) = manifest.task_contract {
        println!("  Task contract: {}", tc.normalized_objective.as_deref().unwrap_or("(present)"));
        println!("    Allowed paths: {}", tc.allowed_paths.len());
        println!("    Protected paths: {}", tc.protected_paths.len());
        println!("    Max actions: {}", tc.max_actions);
    } else {
        println!("  Task contract: (none)");
    }
    println!("  Session memory entries: {}", manifest.session_memory_count);
    println!("  Project memory entries: {}", manifest.project_memory_count);
    println!("  Active rules: {}", manifest.active_rule_count);
    println!("  Open incidents: {}", manifest.open_incident_count);
    println!("  Audit actions: {}", manifest.action_count);
    if let Some(ref hash) = manifest.last_receipt_hash {
        println!("  Last receipt hash: {}", &hash[..16]);
    }

    Ok(())
}

pub fn run_show(args: ShowArgs) -> anyhow::Result<()> {
    let manifest = HandoffManifestV1::from_file(&args.path)?;
    let json = serde_json::to_string_pretty(&manifest)?;
    println!("{}", json);
    Ok(())
}

// ── Main dispatch ────────────────────────────────────────────────────────────

pub fn run(args: HandoffArgs) -> anyhow::Result<()> {
    match args.command {
        HandoffCommand::Create(a) => run_create(a),
        HandoffCommand::Import(a) => run_import(a),
        HandoffCommand::Show(a) => run_show(a),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn detect_git_state(workspace: &Option<PathBuf>) -> (Option<String>, Option<String>) {
    let default_path = PathBuf::from(".");
    let dir = match workspace {
        Some(p) => p.as_path(),
        None => default_path.as_path(),
    };

    let head = std::process::Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        });

    let branch = std::process::Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() || s == "HEAD" { None } else { Some(s) }
        });

    (head, branch)
}

// ── Help text ────────────────────────────────────────────────────────────────

pub fn help_text() -> String {
    r#"onus handoff create --session <id>    — create a handoff manifest for a session
onus handoff import <file>              — import and validate a handoff manifest
onus handoff show <file>                — display a handoff manifest in full JSON"#
        .to_string()
}

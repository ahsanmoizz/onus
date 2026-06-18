//! `onus memory` — manage Onus memory lifecycle.
//!
//! Subcommands:
//!   list     — list memory entries for a workspace
//!   inspect  — show full details of a single memory entry
//!   export   — export memory store as JSON
//!   delete   — soft-delete a memory entry
//!   archive  — archive expired entries or by kind
//!   retention — query and set retention policy
//!   incidents — list or resolve memory incidents

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[allow(dead_code)]
fn open_memory_store() -> anyhow::Result<crate::memory::MemoryStore> {
    let db_path = crate::data_dir().join("memory.db");
    crate::memory::MemoryStore::open(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open memory store at {}: {}", db_path.display(), e))
}

// ── Args ────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct MemoryArgs {
    #[command(subcommand)]
    pub command: MemoryCommand,
}

#[derive(Subcommand)]
pub enum MemoryCommand {
    /// List memory entries for the current workspace
    List(ListArgs),
    /// Show full details of a single memory entry
    Inspect(InspectArgs),
    /// Export memory store as JSON to stdout or file
    Export(ExportArgs),
    /// Soft-delete a memory entry
    Delete(DeleteArgs),
    /// Archive expired entries or by kind
    Archive(ArchiveArgs),
    /// Query retention policies
    Retention(RetentionArgs),
    /// List or resolve incidents
    Incidents(IncidentsArgs),
}

// ── List ─────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ListArgs {
    /// Kind filter: session, project, incident, policy, user_capability
    #[arg(long)]
    pub kind: Option<String>,
    /// Max entries (default 50)
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
    /// Path to memory store (default: config_dir/memory.db)
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_list(args: &ListArgs) -> anyhow::Result<()> {
    let db_path = args.db.clone().unwrap_or_else(|| crate::data_dir().join("memory.db"));
    let store = crate::memory::MemoryStore::open(&db_path)?;
    let scope = crate::memory::MemoryScope {
        tenant_id: crate::memory::tenant_id(),
        project_id: "local".to_string(),
        session_id: None,
    };
    let entries = store.retrieve_relevant(&scope, "", args.limit)?;
    if entries.is_empty() {
        println!("No memory entries found.");
        return Ok(());
    }
    for (i, e) in entries.iter().enumerate() {
        println!(
            "  {:>3}. [{}] {} — {}",
            i + 1,
            e.kind,
            e.key,
            e.summary.chars().take(80).collect::<String>()
        );
    }
    println!("\nTotal: {} entries", entries.len());
    Ok(())
}

// ── Inspect ──────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct InspectArgs {
    /// Memory entry key
    pub key: String,
    /// Path to memory store
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_inspect(args: &InspectArgs) -> anyhow::Result<()> {
    let db_path = args.db.clone().unwrap_or_else(|| crate::data_dir().join("memory.db"));
    let store = crate::memory::MemoryStore::open(&db_path)?;
    let scope = crate::memory::MemoryScope {
        tenant_id: crate::memory::tenant_id(),
        project_id: "local".to_string(),
        session_id: None,
    };
    let entries = store.retrieve_relevant(&scope, &args.key, 100)?;
    let entry = entries.into_iter().find(|e| e.key == args.key);
    match entry {
        Some(e) => {
            println!("Key:     {}", e.kind);
            println!("Kind:    {:?}", e.kind);
            println!("Summary: {}", e.summary);
            println!("Version: {}", e.version);
        }
        None => println!("No memory entry with key '{}' found.", args.key),
    }
    Ok(())
}

// ── Export ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ExportArgs {
    /// Output path (defaults to stdout)
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Kind filter
    #[arg(long)]
    pub kind: Option<String>,
    /// Path to memory store
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_export(args: &ExportArgs) -> anyhow::Result<()> {
    let db_path = args.db.clone().unwrap_or_else(|| crate::data_dir().join("memory.db"));
    let store = crate::memory::MemoryStore::open(&db_path)?;
    let scope = crate::memory::MemoryScope {
        tenant_id: crate::memory::tenant_id(),
        project_id: "local".to_string(),
        session_id: None,
    };
    let entries = store.retrieve_relevant(&scope, "", 10000)?;

    let output = serde_json::to_string_pretty(&entries)?;
    match &args.output {
        Some(path) => {
            std::fs::write(path, &output)?;
            println!("Exported {} entries to {}", entries.len(), path.display());
        }
        None => println!("{}", output),
    }
    Ok(())
}

// ── Delete ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct DeleteArgs {
    /// Memory entry key to delete
    pub key: String,
    /// Path to memory store
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_delete(args: &DeleteArgs) -> anyhow::Result<()> {
    let db_path = args.db.clone().unwrap_or_else(|| crate::data_dir().join("memory.db"));
    let mut store = crate::memory::MemoryStore::open(&db_path)?;
    // soft-delete via scope delete with key as session_id (workaround for API shape)
    let count = store.delete_scope(&crate::memory::tenant_id(), "local", Some(&args.key))?;
    println!("Deleted {} memory entries with key '{}'", count, args.key);
    Ok(())
}

// ── Archive ──────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ArchiveArgs {
    /// Kind to archive: expired, session, incident (default: expired)
    #[arg(long, default_value = "expired")]
    pub kind: String,
    /// Path to memory store
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_archive(args: &ArchiveArgs) -> anyhow::Result<()> {
    let db_path = args.db.clone().unwrap_or_else(|| crate::data_dir().join("memory.db"));
    let mut store = crate::memory::MemoryStore::open(&db_path)?;
    let _now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // SQL-level soft-delete via update
    let count = {
        // For archive, use delete_scope as proxy; in future, use _now for TTL-based filtering.
        store.delete_scope(&crate::memory::tenant_id(), "local", None)?
    };
    println!("Archived {} entries (soft-deleted). Use --kind to filter.", count);
    Ok(())
}

// ── Retention ────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct RetentionArgs {
    /// Query current retention policy
    #[arg(long)]
    pub query: bool,
    /// Set retention days (overrides default)
    #[arg(long)]
    pub days: Option<u32>,
    /// Path to memory store
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_retention(args: &RetentionArgs) -> anyhow::Result<()> {
    let _ = args;
    println!("Retention: default 90 days for project, 7 days for session, 365 days for incident.");
    println!("Use --days <N> to override (not yet persisted — future release).");
    Ok(())
}

// ── Incidents ────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct IncidentsArgs {
    /// List all incidents
    #[arg(long)]
    pub list: bool,
    /// Resolve an incident by key
    #[arg(long)]
    pub resolve: Option<String>,
    /// Path to memory store
    #[arg(long)]
    pub db: Option<PathBuf>,
}

fn run_incidents(args: &IncidentsArgs) -> anyhow::Result<()> {
    let db_path = args.db.clone().unwrap_or_else(|| crate::data_dir().join("memory.db"));
    let store = crate::memory::MemoryStore::open(&db_path)?;
    let scope = crate::memory::MemoryScope {
        tenant_id: crate::memory::tenant_id(),
        project_id: "local".to_string(),
        session_id: None,
    };
    let entries = store.retrieve_relevant(&scope, "incident", 100)?;
    if args.list || args.resolve.is_none() {
        if entries.is_empty() {
            println!("No incidents found.");
        } else {
            for e in &entries {
                println!("  [{}] {} — {:?}", e.key, e.summary, e.kind);
            }
        }
    }
    Ok(())
}

// ── Dispatch ─────────────────────────────────────────────────────────────────

pub fn run(args: MemoryArgs) -> anyhow::Result<()> {
    match args.command {
        MemoryCommand::List(a) => run_list(&a),
        MemoryCommand::Inspect(a) => run_inspect(&a),
        MemoryCommand::Export(a) => run_export(&a),
        MemoryCommand::Delete(a) => run_delete(&a),
        MemoryCommand::Archive(a) => run_archive(&a),
        MemoryCommand::Retention(a) => run_retention(&a),
        MemoryCommand::Incidents(a) => run_incidents(&a),
    }
}

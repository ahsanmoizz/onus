//! Linux L3 workspace management.
//!
//! This module owns the filesystem state for isolated workspaces. The actual
//! process boundary is provided on Linux by `bubblewrap`; on other platforms the
//! isolation runner fails closed and does not execute the requested command.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

const WORKSPACES_DIR: &str = "workspaces";
const METADATA_FILE: &str = "workspace.json";
const WORKTREE_DIR: &str = "worktree";
const CHECKPOINTS_DIR: &str = "checkpoints";
const ARTIFACTS_DIR: &str = "artifacts";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStatus {
    Created,
    Active,
    Destroyed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub schema_version: u32,
    pub session_id: String,
    pub original_repo: PathBuf,
    pub worktree: PathBuf,
    pub root: PathBuf,
    pub status: WorkspaceStatus,
    pub created_at_unix: u64,
    pub network_egress: NetworkPolicy,
    pub isolation_level: String,
    pub enforcement_label: String,
    pub boundary_verified: bool,
    pub last_isolated_run_at_unix: Option<u64>,
    pub protected_host_paths: Vec<PathBuf>,
    pub environment_allowlist: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub checkpoints: Vec<CheckpointMetadata>,
    pub copy_exclusions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    pub id: String,
    pub created_at_unix: u64,
    pub manifest_path: PathBuf,
    pub file_count: usize,
    pub manifest_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicy {
    DenyAll,
    AllowHost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_seconds: u64,
    pub memory_bytes: u64,
    pub max_processes: u64,
    pub max_open_files: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_seconds: 60,
            memory_bytes: 1024 * 1024 * 1024,
            max_processes: 256,
            max_open_files: 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateWorkspaceOptions {
    pub repo: PathBuf,
    pub session_id: Option<String>,
    pub allow_network: bool,
}

#[derive(Debug, Clone)]
pub struct RunWorkspaceOptions {
    pub session_id: String,
    pub command: Vec<String>,
    pub allow_network: bool,
    pub resource_limits: ResourceLimits,
}

pub fn workspace_store_root() -> PathBuf {
    crate::data_dir().join(WORKSPACES_DIR)
}

pub fn create_workspace(options: CreateWorkspaceOptions) -> Result<WorkspaceMetadata> {
    let repo = canonical_existing_dir(&options.repo)
        .with_context(|| format!("repository path does not exist: {}", options.repo.display()))?;
    let session_id = options
        .session_id
        .unwrap_or_else(|| format!("l3-{}", Uuid::new_v4()));
    validate_session_id(&session_id)?;

    let root = workspace_store_root().join(&session_id);
    if root.exists() {
        anyhow::bail!("workspace already exists for session {}", session_id);
    }

    let worktree = root.join(WORKTREE_DIR);
    let checkpoints = root.join(CHECKPOINTS_DIR);
    let artifacts = root.join(ARTIFACTS_DIR);
    fs::create_dir_all(&worktree)?;
    fs::create_dir_all(&checkpoints)?;
    fs::create_dir_all(&artifacts)?;

    let exclusions = default_copy_exclusions();
    copy_repo_snapshot(&repo, &worktree, &exclusions)?;

    let checkpoint = create_checkpoint(&root, &worktree, "initial")?;
    let metadata = WorkspaceMetadata {
        schema_version: 1,
        session_id,
        original_repo: repo,
        worktree,
        root,
        status: WorkspaceStatus::Created,
        created_at_unix: unix_now(),
        network_egress: if options.allow_network {
            NetworkPolicy::AllowHost
        } else {
            NetworkPolicy::DenyAll
        },
        isolation_level: "L3_PENDING_RUNTIME_VERIFICATION".to_string(),
        enforcement_label: "L3_LINUX_WORKSPACE_PENDING_VERIFICATION".to_string(),
        boundary_verified: false,
        last_isolated_run_at_unix: None,
        protected_host_paths: default_protected_host_paths(),
        environment_allowlist: safe_environment_allowlist(),
        resource_limits: ResourceLimits::default(),
        checkpoints: vec![checkpoint],
        copy_exclusions: exclusions,
    };
    save_metadata(&metadata)?;
    Ok(metadata)
}

pub fn load_workspace(session_id: &str) -> Result<WorkspaceMetadata> {
    validate_session_id(session_id)?;
    let path = workspace_store_root().join(session_id).join(METADATA_FILE);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("workspace metadata not found: {}", path.display()))?;
    let metadata: WorkspaceMetadata = serde_json::from_str(&raw)
        .with_context(|| format!("workspace metadata is malformed: {}", path.display()))?;
    if metadata.session_id != session_id {
        anyhow::bail!("workspace metadata session id mismatch");
    }
    Ok(metadata)
}

pub fn save_metadata(metadata: &WorkspaceMetadata) -> Result<()> {
    fs::create_dir_all(&metadata.root)?;
    let path = metadata.root.join(METADATA_FILE);
    let raw = serde_json::to_string_pretty(metadata)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn export_workspace(session_id: &str, destination: &Path) -> Result<PathBuf> {
    let metadata = load_workspace(session_id)?;
    ensure_workspace_live(&metadata)?;

    let export_root = destination.join(&metadata.session_id);
    if export_root.exists() {
        anyhow::bail!(
            "export destination already exists; refusing to overwrite: {}",
            export_root.display()
        );
    }
    fs::create_dir_all(&export_root)?;
    fs::copy(
        metadata.root.join(METADATA_FILE),
        export_root.join(METADATA_FILE),
    )?;
    let exported_worktree = export_root.join(WORKTREE_DIR);
    fs::create_dir_all(&exported_worktree)?;
    copy_tree_no_symlink_follow(&metadata.worktree, &exported_worktree)?;
    Ok(export_root)
}

pub fn destroy_workspace(session_id: &str) -> Result<()> {
    let metadata = load_workspace(session_id)?;
    let root = canonical_maybe_existing(&metadata.root)?;
    let store = canonical_maybe_existing(&workspace_store_root())?;
    if !root.starts_with(&store) {
        anyhow::bail!("refusing to destroy path outside Onus workspace store");
    }
    fs::remove_dir_all(&root)?;
    Ok(())
}

pub fn inspect_workspace(session_id: &str) -> Result<WorkspaceMetadata> {
    load_workspace(session_id)
}

pub fn latest_workspace_id() -> Result<String> {
    let root = workspace_store_root();
    let mut newest: Option<(u64, String)> = None;
    for entry in fs::read_dir(&root).with_context(|| {
        format!(
            "no workspaces found in {}; run `onus workspace create` first",
            root.display()
        )
    })? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if let Ok(metadata) = load_workspace(&id) {
            let created = metadata.created_at_unix;
            if newest.as_ref().map(|(ts, _)| created > *ts).unwrap_or(true) {
                newest = Some((created, id));
            }
        }
    }
    newest
        .map(|(_, id)| id)
        .ok_or_else(|| anyhow::anyhow!("no valid workspaces found"))
}

pub fn run_isolated(options: RunWorkspaceOptions) -> Result<i32> {
    if options.command.is_empty() {
        anyhow::bail!("no command provided after `--`");
    }

    let mut metadata = load_workspace(&options.session_id)?;
    ensure_workspace_live(&metadata)?;
    require_linux_l3_available()?;

    let status = run_isolated_platform(&metadata, &options)?;
    metadata.status = WorkspaceStatus::Active;
    metadata.isolation_level = "L3_LINUX_BUBBLEWRAP".to_string();
    metadata.enforcement_label = "L3_LINUX_WORKSPACE_RUNTIME_VERIFIED".to_string();
    metadata.boundary_verified = true;
    metadata.last_isolated_run_at_unix = Some(unix_now());
    save_metadata(&metadata)?;
    Ok(status)
}

pub fn filtered_environment(session_id: &str, worktree: &Path) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    env.insert(
        "PATH".to_string(),
        "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
    );
    env.insert("HOME".to_string(), "/tmp".to_string());
    env.insert("TMPDIR".to_string(), "/tmp".to_string());
    env.insert("ONUS_ISOLATED".to_string(), "1".to_string());
    env.insert("ONUS_ENFORCEMENT_LEVEL".to_string(), "L3".to_string());
    env.insert("ONUS_WORKSPACE_ID".to_string(), session_id.to_string());
    env.insert("ONUS_WORKSPACE_ROOT".to_string(), "/workspace".to_string());
    env.insert(
        "ONUS_HOST_WORKTREE_HASH".to_string(),
        crate::security::sha256_hex(&worktree.display().to_string()),
    );
    env
}

pub fn require_linux_l3_available() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if find_on_path("bwrap").is_none() {
            anyhow::bail!(
                "L3 workspace isolation requires bubblewrap (`bwrap`) on Linux; refusing to run without a real process/filesystem/network boundary"
            );
        }
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!(
            "L3 workspace isolation is currently implemented only on Linux; refusing to run without a real L3 boundary"
        );
    }
}

#[cfg(target_os = "linux")]
fn run_isolated_platform(
    metadata: &WorkspaceMetadata,
    options: &RunWorkspaceOptions,
) -> Result<i32> {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    let bwrap = find_on_path("bwrap").expect("checked by require_linux_l3_available");
    let mut args = vec![
        "--die-with-parent".to_string(),
        "--new-session".to_string(),
        "--unshare-all".to_string(),
        "--proc".to_string(),
        "/proc".to_string(),
        "--dev".to_string(),
        "/dev".to_string(),
        "--tmpfs".to_string(),
        "/tmp".to_string(),
        "--tmpfs".to_string(),
        "/run".to_string(),
        "--ro-bind".to_string(),
        metadata.original_repo.display().to_string(),
        "/original".to_string(),
        "--bind".to_string(),
        metadata.worktree.display().to_string(),
        "/workspace".to_string(),
        "--chdir".to_string(),
        "/workspace".to_string(),
        "--clearenv".to_string(),
    ];

    if options.allow_network || metadata.network_egress == NetworkPolicy::AllowHost {
        args.push("--share-net".to_string());
    }

    for (key, value) in filtered_environment(&metadata.session_id, &metadata.worktree) {
        args.push("--setenv".to_string());
        args.push(key);
        args.push(value);
    }

    for dir in ["/bin", "/usr", "/lib", "/lib64", "/sbin"] {
        if Path::new(dir).exists() {
            args.push("--ro-bind".to_string());
            args.push(dir.to_string());
            args.push(dir.to_string());
        }
    }

    args.push("--".to_string());
    args.extend(options.command.clone());

    let limits = options.resource_limits.clone();
    let mut command = Command::new(bwrap);
    command.args(args);
    unsafe {
        command.pre_exec(move || set_resource_limits(&limits));
    }
    let status = command.status()?;
    Ok(status.code().unwrap_or(128))
}

#[cfg(not(target_os = "linux"))]
fn run_isolated_platform(
    _metadata: &WorkspaceMetadata,
    _options: &RunWorkspaceOptions,
) -> Result<i32> {
    anyhow::bail!(
        "L3 workspace isolation is currently implemented only on Linux; refusing to execute"
    );
}

#[cfg(target_os = "linux")]
fn set_resource_limits(limits: &ResourceLimits) -> std::io::Result<()> {
    use libc::{rlimit, setrlimit, RLIMIT_AS, RLIMIT_CPU, RLIMIT_NOFILE, RLIMIT_NPROC};

    fn apply(resource: u32, value: u64) -> std::io::Result<()> {
        let lim = rlimit {
            rlim_cur: value as libc::rlim_t,
            rlim_max: value as libc::rlim_t,
        };
        let rc = unsafe { setrlimit(resource, &lim) };
        if rc == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    apply(RLIMIT_CPU, limits.cpu_seconds)?;
    apply(RLIMIT_AS, limits.memory_bytes)?;
    apply(RLIMIT_NPROC, limits.max_processes)?;
    apply(RLIMIT_NOFILE, limits.max_open_files)?;
    Ok(())
}

fn ensure_workspace_live(metadata: &WorkspaceMetadata) -> Result<()> {
    if metadata.status == WorkspaceStatus::Destroyed {
        anyhow::bail!("workspace is destroyed");
    }
    if !metadata.worktree.is_dir() {
        anyhow::bail!(
            "workspace worktree is missing: {}",
            metadata.worktree.display()
        );
    }
    if !metadata.original_repo.is_dir() {
        anyhow::bail!(
            "original repository path is missing: {}",
            metadata.original_repo.display()
        );
    }
    Ok(())
}

fn create_checkpoint(root: &Path, worktree: &Path, id: &str) -> Result<CheckpointMetadata> {
    let checkpoint_dir = root.join(CHECKPOINTS_DIR);
    fs::create_dir_all(&checkpoint_dir)?;
    let manifest = build_manifest(worktree)?;
    let manifest_raw = serde_json::to_string_pretty(&manifest)?;
    let manifest_hash = crate::security::sha256_hex(&manifest_raw);
    let manifest_path = checkpoint_dir.join(format!("{}.manifest.json", id));
    fs::write(&manifest_path, manifest_raw)?;
    Ok(CheckpointMetadata {
        id: id.to_string(),
        created_at_unix: unix_now(),
        manifest_path,
        file_count: manifest.len(),
        manifest_hash,
    })
}

fn build_manifest(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut manifest = BTreeMap::new();
    collect_manifest(root, root, &mut manifest)?;
    Ok(manifest)
}

fn collect_manifest(
    root: &Path,
    dir: &Path,
    manifest: &mut BTreeMap<String, String>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_manifest(root, &path, manifest)?;
        } else if meta.is_file() {
            let rel = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            manifest.insert(rel, sha256_file(&path)?);
        }
    }
    Ok(())
}

fn copy_repo_snapshot(src: &Path, dst: &Path, exclusions: &[String]) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if exclusions.iter().any(|excluded| excluded == &name) {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        let meta = fs::symlink_metadata(&from)?;
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            fs::create_dir_all(&to)?;
            copy_repo_snapshot(&from, &to, exclusions)?;
        } else if meta.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn copy_tree_no_symlink_follow(src: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let meta = fs::symlink_metadata(&from)?;
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            fs::create_dir_all(&to)?;
            copy_tree_no_symlink_follow(&from, &to)?;
        } else if meta.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn validate_session_id(value: &str) -> Result<()> {
    let ok = !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(())
    } else {
        anyhow::bail!("invalid workspace/session id");
    }
}

fn canonical_existing_dir(path: &Path) -> Result<PathBuf> {
    let resolved = fs::canonicalize(path)?;
    if !resolved.is_dir() {
        anyhow::bail!("not a directory: {}", resolved.display());
    }
    Ok(resolved)
}

fn canonical_maybe_existing(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return Ok(fs::canonicalize(path)?);
    }
    let mut absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    absolute = normalized;
    Ok(absolute)
}

fn default_copy_exclusions() -> Vec<String> {
    vec![
        ".git".to_string(),
        ".onus".to_string(),
        "target".to_string(),
        "node_modules".to_string(),
        "__pycache__".to_string(),
    ]
}

fn default_protected_host_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/"),
        PathBuf::from("/home"),
        PathBuf::from("/root"),
        PathBuf::from("/etc"),
        PathBuf::from("/var"),
        PathBuf::from("/mnt"),
        PathBuf::from("/media"),
        PathBuf::from("/run"),
        PathBuf::from("/proc"),
        PathBuf::from("/sys"),
    ]
}

fn safe_environment_allowlist() -> Vec<String> {
    vec![
        "PATH".to_string(),
        "HOME".to_string(),
        "TMPDIR".to_string(),
        "ONUS_ISOLATED".to_string(),
        "ONUS_ENFORCEMENT_LEVEL".to_string(),
        "ONUS_WORKSPACE_ID".to_string(),
        "ONUS_WORKSPACE_ROOT".to_string(),
        "ONUS_HOST_WORKTREE_HASH".to_string(),
    ]
}

#[cfg(target_os = "linux")]
fn find_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("onus-workspace-test-{}-{}", name, Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn filtered_environment_excludes_common_secrets() {
        let env = filtered_environment("session", Path::new("/tmp/worktree"));
        assert!(env.contains_key("PATH"));
        assert!(!env.contains_key("AWS_SECRET_ACCESS_KEY"));
        assert!(!env.contains_key("DATABASE_URL"));
        assert_eq!(env["ONUS_ENFORCEMENT_LEVEL"], "L3");
    }

    #[test]
    fn create_workspace_copies_repo_and_records_initial_checkpoint() {
        let repo = temp_dir("repo");
        fs::write(repo.join("README.md"), "hello").unwrap();
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::write(repo.join(".git").join("config"), "secret-ish").unwrap();

        let metadata = create_workspace(CreateWorkspaceOptions {
            repo: repo.clone(),
            session_id: Some(format!("test-{}", Uuid::new_v4())),
            allow_network: false,
        })
        .unwrap();

        assert!(metadata.worktree.join("README.md").is_file());
        assert!(!metadata.worktree.join(".git").exists());
        assert_eq!(metadata.network_egress, NetworkPolicy::DenyAll);
        assert_eq!(metadata.checkpoints.len(), 1);
        assert!(metadata.checkpoints[0].file_count >= 1);

        destroy_workspace(&metadata.session_id).unwrap();
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn export_refuses_to_overwrite_existing_destination() {
        let repo = temp_dir("repo-export");
        fs::write(repo.join("file.txt"), "contents").unwrap();
        let metadata = create_workspace(CreateWorkspaceOptions {
            repo: repo.clone(),
            session_id: Some(format!("export-{}", Uuid::new_v4())),
            allow_network: false,
        })
        .unwrap();
        let dest = temp_dir("export-dest");

        let exported = export_workspace(&metadata.session_id, &dest).unwrap();
        assert!(exported.join(WORKTREE_DIR).join("file.txt").is_file());
        let second = export_workspace(&metadata.session_id, &dest);
        assert!(second.is_err());

        destroy_workspace(&metadata.session_id).unwrap();
        let _ = fs::remove_dir_all(repo);
        let _ = fs::remove_dir_all(dest);
    }
}

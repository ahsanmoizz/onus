//! Scope tracker — monitors what the agent is supposed to be doing
//! and detects when actions drift outside the task boundary.

use crate::ipc::ActionRequest;
use crate::Verdict;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Complexity estimation for a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Complexity {
    Small,  // < 50 lines of change
    Medium, // 50-200 lines
    Large,  // 200+ lines
}

impl Complexity {
    pub fn max_diff_lines(&self) -> usize {
        match self {
            Complexity::Small => 50,
            Complexity::Medium => 200,
            Complexity::Large => 1000,
        }
    }
}

/// Tracks the scope of a single agent session.
#[derive(Debug, Clone)]
pub struct SessionScope {
    pub session_id: String,
    pub task_description: String,
    pub workspace_root: PathBuf,
    pub complexity: Complexity,
    /// Files declared by the agent at session start (e.g. "I will modify src/foo.rs").
    pub declared_files: Vec<String>,
    /// Additional paths explicitly allowed by policy.
    pub allowed_paths: Vec<String>,
    /// Files actually touched during the session.
    pub touched_files: Vec<String>,
    pub action_count: u32,
    pub retry_count: u32,
    /// Marked when goal drift is detected.
    pub drift_detected: bool,
}

/// Manages scope tracking across all active sessions.
#[derive(Debug)]
pub struct ScopeTracker {
    sessions: HashMap<String, SessionScope>,
}

impl ScopeTracker {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Start tracking a new session.
    pub fn start_session(
        &mut self,
        session_id: String,
        task_description: String,
        workspace_root: String,
        declared_files: Vec<String>,
        allowed_paths: Vec<String>,
    ) {
        let complexity = estimate_complexity(&task_description);

        self.sessions.insert(
            session_id.clone(),
            SessionScope {
                session_id,
                task_description,
                workspace_root: PathBuf::from(workspace_root),
                complexity,
                declared_files,
                allowed_paths,
                touched_files: Vec::new(),
                action_count: 0,
                retry_count: 0,
                drift_detected: false,
            },
        );
    }

    /// End tracking a session.
    pub fn end_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    /// Number of currently active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Evaluate an action for scope violations.
    pub fn evaluate(&self, request: &ActionRequest) -> Verdict {
        let session = match self.sessions.get(&request.session_id) {
            Some(s) => s,
            None => {
                // If no session is registered, allow by default (no scope to violate).
                return Verdict::Allow;
            }
        };

        // Check goal drift for file operations.
        if !session.declared_files.is_empty() || !session.allowed_paths.is_empty() {
            if let Some(path) = extract_path(&request.action.payload) {
                if self.is_drift(&request.session_id, &path) {
                    log::warn!(
                        "Goal drift detected: touching undeclared file {} in session {}",
                        path,
                        request.session_id
                    );
                    return Verdict::Warn;
                }
            }
        }

        match &request.action.action_type {
            crate::ActionType::FileWrite => {
                self.check_file_scope(session, &request.action.payload, "write")
            }
            crate::ActionType::FileDelete => {
                self.check_file_scope(session, &request.action.payload, "delete")
            }
            crate::ActionType::FileRead => {
                self.check_file_scope(session, &request.action.payload, "read")
            }
            _ => Verdict::Allow,
        }
    }

    /// Check if a file operation is within the workspace scope.
    fn check_file_scope(
        &self,
        session: &SessionScope,
        payload: &serde_json::Value,
        operation: &str,
    ) -> Verdict {
        let file_path = extract_path(payload);

        // If we can't extract a path, allow (can't evaluate scope).
        let file_path = match file_path {
            Some(p) => p,
            None => return Verdict::Allow,
        };

        // Check if path is under workspace_root.
        let abs_path = if Path::new(&file_path).is_absolute() {
            PathBuf::from(&file_path)
        } else {
            session.workspace_root.join(&file_path)
        };

        // Canonicalize if possible, otherwise check prefix.
        let in_scope = if let Ok(canon) = abs_path.canonicalize() {
            canon.starts_with(&session.workspace_root)
        } else {
            // If path doesn't exist yet (new file), check prefix.
            abs_path.starts_with(&session.workspace_root) || file_path.starts_with("./")
        };

        // Always allow writes/deletes within standard safe directories.
        if is_safe_path(Path::new(&file_path)) {
            return Verdict::Allow;
        }

        if !in_scope {
            log::warn!(
                "Scope violation: {} outside workspace. Path: {:?}, Workspace: {:?}",
                operation,
                file_path,
                session.workspace_root
            );
            Verdict::Block
        } else {
            Verdict::Allow
        }
    }

    /// Record an allowed action to update scope state.
    pub fn record_action(&mut self, request: &ActionRequest) {
        if let Some(session) = self.sessions.get_mut(&request.session_id) {
            session.action_count += 1;

            // Track touched files.
            if let Some(path) = extract_path(&request.action.payload) {
                if !session.touched_files.contains(&path) {
                    session.touched_files.push(path);
                }
            }
        }
    }

    /// Increment retry count for a session (blocked action, agent retrying).
    pub fn increment_retry(&mut self, session_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.retry_count += 1;
        }
    }

    /// Get session retry count (used to decide escalate vs block).
    pub fn retry_count(&self, session_id: &str) -> u32 {
        self.sessions
            .get(session_id)
            .map(|s| s.retry_count)
            .unwrap_or(0)
    }

    /// Detect goal drift: check if any touched file is outside declared_files + allowed_paths.
    /// Returns a list of undeclared files that were touched.
    pub fn detect_drift(&self, session_id: &str) -> Vec<String> {
        let session = match self.sessions.get(session_id) {
            Some(s) => s,
            None => return vec![],
        };

        if session.declared_files.is_empty() && session.allowed_paths.is_empty() {
            return vec![]; // No declared scope — can't detect drift.
        }

        let mut drifters = Vec::new();

        // Build allowed set: canonicalized declared files + explicit allowed paths.
        let workspace = &session.workspace_root;
        let mut allowed = session.allowed_paths.clone();

        for f in &session.declared_files {
            let abs = if Path::new(f).is_absolute() {
                PathBuf::from(f)
            } else {
                workspace.join(f)
            };
            allowed.push(abs.to_string_lossy().to_string());
        }

        for touched in &session.touched_files {
            let abs = if Path::new(touched).is_absolute() {
                PathBuf::from(touched)
            } else {
                workspace.join(touched)
            };

            let in_allowed = allowed.iter().any(|a| {
                let allowed_path = Path::new(a);
                abs.starts_with(allowed_path) || allowed_path.starts_with(&abs)
            });

            // Safe paths are always allowed.
            if is_safe_path(&abs) {
                continue;
            }

            if !in_allowed {
                drifters.push(touched.clone());
            }
        }

        drifters
    }

    /// Check if a specific file path would constitute drift for the given session.
    /// Used during evaluate() to catch drift on each action.
    pub fn is_drift(&self, session_id: &str, file_path: &str) -> bool {
        let session = match self.sessions.get(session_id) {
            Some(s) => s,
            None => return false,
        };

        if session.declared_files.is_empty() && session.allowed_paths.is_empty() {
            return false;
        }

        if is_safe_path(Path::new(file_path)) {
            return false;
        }

        let workspace = &session.workspace_root;
        let abs = if Path::new(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            workspace.join(file_path)
        };

        // Check against declared files.
        let in_declared = session.declared_files.iter().any(|d| {
            let d_abs = if Path::new(d).is_absolute() {
                PathBuf::from(d)
            } else {
                workspace.join(d)
            };
            abs.starts_with(&d_abs) || d_abs.starts_with(&abs)
        });

        if in_declared {
            return false;
        }

        // Check against allowed paths.
        let in_allowed = session.allowed_paths.iter().any(|a| {
            let a_abs = if Path::new(a).is_absolute() {
                PathBuf::from(a)
            } else {
                workspace.join(a)
            };
            abs.starts_with(&a_abs) || a_abs.starts_with(&abs)
        });

        !in_allowed
    }
}

/// Estimate complexity from task description.
fn estimate_complexity(description: &str) -> Complexity {
    let word_count = description.split_whitespace().count();
    let has_complex_keywords = [
        "refactor",
        "migrate",
        "rewrite",
        "implement",
        "architecture",
    ]
    .iter()
    .any(|kw| description.to_lowercase().contains(kw));

    if has_complex_keywords || word_count > 100 {
        Complexity::Large
    } else if word_count > 30 {
        Complexity::Medium
    } else {
        Complexity::Small
    }
}

/// Extract a file path from an action payload.
fn extract_path(payload: &serde_json::Value) -> Option<String> {
    match payload {
        serde_json::Value::String(s) => {
            // For shell commands, try to extract a file path.
            // This is a heuristic — first non-flag argument after the command.
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() >= 2 {
                let candidate = parts[parts.len() - 1];
                if candidate.contains('.') || candidate.contains('/') {
                    return Some(candidate.to_string());
                }
            }
            None
        }
        serde_json::Value::Object(map) => {
            if let Some(path) = map.get("path").and_then(|v| v.as_str()) {
                Some(path.to_string())
            } else if let Some(file_path) = map.get("file_path").and_then(|v| v.as_str()) {
                Some(file_path.to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Check if a path is a known-safe directory (build artifacts, caches).
fn is_safe_path(path: &Path) -> bool {
    let safe_dirs = [
        "node_modules",
        ".cache",
        "__pycache__",
        "target",
        "dist",
        "build",
        ".next",
        ".turbo",
        "coverage",
        ".venv",
        "venv",
        "tmp",
        ".tmp",
        ".git",
    ];

    safe_dirs.iter().any(|d| {
        path.starts_with(d)
            || path.to_string_lossy().contains(&format!("/{}/", d))
            || path.to_string_lossy().contains(&format!("\\{}\\", d))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_complexity_small() {
        assert_eq!(estimate_complexity("Fix typo in README"), Complexity::Small);
    }

    #[test]
    fn test_estimate_complexity_large() {
        assert_eq!(
            estimate_complexity("Refactor the entire authentication module to use OAuth 2.0"),
            Complexity::Large
        );
    }

    #[test]
    fn test_is_safe_path() {
        assert!(is_safe_path(Path::new("node_modules/react/index.js")));
        assert!(is_safe_path(Path::new("target/debug/build")));
        assert!(!is_safe_path(Path::new("src/main.rs")));
    }

    #[test]
    fn test_no_drift_with_declared_file() {
        let mut tracker = ScopeTracker::new();
        tracker.start_session(
            "s1".into(),
            "Fix the login module".into(),
            "/workspace".into(),
            vec!["src/auth/login.rs".into()],
            vec![],
        );
        assert!(!tracker.is_drift("s1", "src/auth/login.rs"));
    }

    #[test]
    fn test_drift_detected_for_undeclared_file() {
        let mut tracker = ScopeTracker::new();
        tracker.start_session(
            "s1".into(),
            "Fix the login module".into(),
            "/workspace".into(),
            vec!["src/auth/login.rs".into()],
            vec![],
        );
        assert!(tracker.is_drift("s1", "src/payments/process.rs"));
    }

    #[test]
    fn test_no_drift_with_allowed_path() {
        let mut tracker = ScopeTracker::new();
        tracker.start_session(
            "s1".into(),
            "Fix the login module".into(),
            "/workspace".into(),
            vec![],
            vec!["/workspace/src".into()],
        );
        assert!(!tracker.is_drift("s1", "src/auth/login.rs"));
    }

    #[test]
    fn test_detect_drift_returns_list() {
        let mut tracker = ScopeTracker::new();
        tracker.start_session(
            "s1".into(),
            "Fix login".into(),
            "/workspace".into(),
            vec!["src/auth/login.rs".into()],
            vec![],
        );
        // Simulate touching files directly.
        if let Some(s) = tracker.sessions.get_mut("s1") {
            s.touched_files.push("src/auth/login.rs".into());
            s.touched_files.push("src/payments/process.rs".into());
        }

        let drifters = tracker.detect_drift("s1");
        assert_eq!(drifters.len(), 1);
        assert!(drifters[0].contains("payments"));
    }
}

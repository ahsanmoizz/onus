//! Daemon manager — background process lifecycle for Onus Core.
//! Handles PID file management, socket cleanup, and process forking.
// Cross-platform: unused imports/vars on one platform are used on the other.
#![allow(unused_imports, unreachable_code, unused_variables)]

use crate::audit::AuditTrail;
use crate::config_dir;
use crate::ipc::server::{handle_client, ServerState};
use crate::policy::PolicyEngine;
use std::fs;
use std::path::PathBuf;
use std::process;

/// PID file stored in config directory.
pub fn pid_file() -> PathBuf {
    config_dir().join("onus.pid")
}

/// Check if the daemon is currently running.
pub fn is_running() -> bool {
    if let Ok(contents) = fs::read_to_string(pid_file()) {
        if let Ok(pid) = contents.trim().parse::<u32>() {
            return process_exists(pid);
        }
    }
    false
}

/// Get the PID of the running daemon.
pub fn get_pid() -> Option<u32> {
    fs::read_to_string(pid_file())
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .filter(|&pid| process_exists(pid))
}

/// Start the daemon in the background.
pub fn start_daemon(foreground: bool) -> anyhow::Result<()> {
    if is_running() {
        let pid = get_pid().unwrap_or(0);
        anyhow::bail!("Onus daemon is already running (PID: {})", pid);
    }

    if foreground {
        run_foreground()
    } else {
        // Fork to background.
        #[cfg(unix)]
        {
            let pid = unsafe { libc::fork() };
            if pid < 0 {
                anyhow::bail!("Failed to fork daemon process");
            }
            if pid > 0 {
                // Parent: write PID file and exit.
                let pid_file_path = pid_file();
                if let Some(parent) = pid_file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&pid_file_path, pid.to_string())?;
                println!("Onus daemon started (PID: {})", pid);
                return Ok(());
            }
            // Child: continue to run_foreground.
        }
        #[cfg(windows)]
        {
            // On Windows, spawn a detached process via CreateProcess.
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use winapi::um::processthreadsapi::CreateProcessW;
            use winapi::um::winbase::DETACHED_PROCESS;
            use winapi::um::processthreadsapi::STARTUPINFOW;
            use winapi::um::processthreadsapi::PROCESS_INFORMATION;

            let exe = std::env::current_exe()?;
            let cmd = format!(
                "\"{}\" daemon start --foreground",
                exe.to_string_lossy()
            );
            let wide: Vec<u16> = OsStr::new(&cmd)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
            si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
            let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

            let result = unsafe {
                CreateProcessW(
                    std::ptr::null(),
                    wide.as_ptr() as *mut _,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    0,
                    DETACHED_PROCESS,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    &mut si,
                    &mut pi,
                )
            };

            if result == 0 {
                anyhow::bail!(
                    "Failed to start daemon process: {}",
                    std::io::Error::last_os_error()
                );
            }

            // Write PID file with the new process ID.
            let pid_file_path = pid_file();
            if let Some(parent) = pid_file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&pid_file_path, pi.dwProcessId.to_string())?;

            unsafe {
                winapi::um::handleapi::CloseHandle(pi.hProcess);
                winapi::um::handleapi::CloseHandle(pi.hThread);
            }

            println!("Onus daemon started (PID: {})", pi.dwProcessId);
            return Ok(());
        }

        run_foreground()
    }
}

/// Run the daemon in the foreground (blocks until terminated).
fn run_foreground() -> anyhow::Result<()> {
    let pid = process::id();
    let pid_file_path = pid_file();

    // Write PID file.
    if let Some(parent) = pid_file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&pid_file_path, pid.to_string())?;

    // Initialize components.
    let rules_path = config_dir().join("rules").join("default.toml");

    // If rules don't exist in config, copy default rules from install location.
    if !rules_path.exists() {
        // Check for rules alongside the binary.
        let default_rules = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("rules").join("default.toml")));

        if let Some(ref default) = default_rules {
            if default.exists() {
                if let Some(parent) = rules_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(default, &rules_path)?;
            }
        }
    }

    let rules = crate::policy::rule::load_rules(&rules_path)
        .map_err(|e| anyhow::anyhow!("Failed to load rules: {}", e))?;

    let policy_engine = PolicyEngine::new(rules);

    let data_dir = crate::data_dir();
    fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("audit.db");
    let audit_trail = AuditTrail::open(&db_path)?;

    let state = ServerState::new(policy_engine, audit_trail);

    // Open IPC socket.
    let socket_path = crate::default_socket_path();

    log::info!("Onus v{} daemon starting on {:?}", env!("CARGO_PKG_VERSION"), socket_path);

    #[cfg(unix)]
    {
        use std::os::unix::net::UnixListener;

        // Remove stale socket.
        let _ = fs::remove_file(&socket_path);
        if let Some(parent) = socket_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let listener = UnixListener::bind(&socket_path)?;

        log::info!("Listening on {}", socket_path.display());

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let stream_clone = match stream.try_clone() {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    handle_client(&state, stream_clone).ok();
                }
                Err(e) => {
                    log::error!("Connection error: {}", e);
                }
            }
        }
    }

    #[cfg(windows)]
    {
        use winapi::um::namedpipeapi::{ConnectNamedPipe, CreateNamedPipeW};
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::um::fileapi::{ReadFile, WriteFile};
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::os::windows::io::{FromRawHandle, OwnedHandle, AsRawHandle};
        use std::io::{self, Read, Write};

        /// Wraps a Windows named pipe HANDLE to implement Read + Write.
        struct PipeStream {
            handle: OwnedHandle,
        }

        impl Read for PipeStream {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                let mut n = 0u32;
                let result = unsafe {
                    ReadFile(
                        self.handle.as_raw_handle() as _,
                        buf.as_mut_ptr() as _,
                        buf.len() as u32,
                        &mut n,
                        std::ptr::null_mut(),
                    )
                };
                if result == 0 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(n as usize)
                }
            }
        }

        impl Write for PipeStream {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let mut n = 0u32;
                let result = unsafe {
                    WriteFile(
                        self.handle.as_raw_handle() as _,
                        buf.as_ptr() as _,
                        buf.len() as u32,
                        &mut n,
                        std::ptr::null_mut(),
                    )
                };
                if result == 0 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(n as usize)
                }
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let pipe_name: Vec<u16> = OsStr::new(r"\\.\pipe\onus-ipc")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        loop {
            let handle = unsafe {
                CreateNamedPipeW(
                    pipe_name.as_ptr(),
                    3, // PIPE_ACCESS_DUPLEX
                    0, // PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT
                    255, // PIPE_UNLIMITED_INSTANCES
                    65536,
                    65536,
                    0,
                    std::ptr::null_mut(),
                )
            };

            if handle == INVALID_HANDLE_VALUE {
                log::error!("Failed to create named pipe");
                break;
            }

            unsafe { ConnectNamedPipe(handle, std::ptr::null_mut()) };

            unsafe {
                let pipe = PipeStream {
                    handle: OwnedHandle::from_raw_handle(handle as _),
                };
                handle_client(&state, pipe).ok();
            }
        }
    }

    // Cleanup.
    let _ = fs::remove_file(pid_file_path);
    let _ = fs::remove_file(&socket_path);

    Ok(())
}

/// Stop the running daemon.
pub fn stop_daemon() -> anyhow::Result<()> {
    let pid = get_pid().ok_or_else(|| anyhow::anyhow!("Onus daemon is not running"))?;

    #[cfg(unix)]
    {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }
    #[cfg(windows)]
    {
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
        use winapi::um::winnt::PROCESS_TERMINATE;

        let handle = unsafe {
            OpenProcess(PROCESS_TERMINATE, 0, pid)
        };
        if handle.is_null() {
            anyhow::bail!("Failed to open process {}: {}", pid, std::io::Error::last_os_error());
        }
        let result = unsafe { TerminateProcess(handle, 0) };
        unsafe { CloseHandle(handle); }
        if result == 0 {
            anyhow::bail!("Failed to terminate process {}: {}", pid, std::io::Error::last_os_error());
        }
    }

    // Remove PID file.
    let _ = fs::remove_file(pid_file());

    println!("Onus daemon stopped (PID: {})", pid);
    Ok(())
}

/// Check if a process with the given PID exists.
fn process_exists(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(windows)]
    {
        // Simplified check — try to open process.
        use std::ptr::null_mut;
        let handle = unsafe {
            winapi::um::processthreadsapi::OpenProcess(
                winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                pid,
            )
        };
        if handle.is_null() {
            return false;
        }
        unsafe { winapi::um::handleapi::CloseHandle(handle) };
        true
    }
}

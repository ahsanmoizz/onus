//! IPC client library for connecting to the Onus daemon.
//!
//! Provides a reusable client that connects via Unix socket (Linux/macOS)
//! or named pipe (Windows) and sends all supported command types.

use crate::ipc::protocol::{read_message_raw, write_message_raw};
use crate::ipc::{
    ActionRequest, ActionResponse, DaemonMessage, DaemonResponse, RulesResponse, ServerCommand,
    SessionCommand, SessionResponse, StatusResponse,
};
use std::io::{Read, Write};

/// A client connected to the Onus daemon.
pub struct OnusClient {
    stream: Box<dyn Stream>,
}

/// Abstracts over Unix socket and Windows named pipe.
pub trait Stream: Read + Write + Send {}
impl<T: Read + Write + Send> Stream for T {}

impl OnusClient {
    /// Connect to the daemon at the default socket/pipe path.
    pub fn connect() -> Result<Self, String> {
        let path = crate::default_socket_path();
        Self::connect_to(&path.to_string_lossy())
    }

    /// Connect to the daemon at a specific path.
    pub fn connect_to(path: &str) -> Result<Self, String> {
        let stream = connect_stream(path)?;
        Ok(OnusClient { stream })
    }

    /// Evaluate a single action.
    pub fn evaluate(&mut self, request: &ActionRequest) -> Result<ActionResponse, String> {
        let msg = DaemonMessage {
            action_request: Some(request.clone()),
            session_command: None,
            server_command: None,
        };
        let data = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;
        write_message_raw(&mut self.stream, &data).map_err(|e| e.to_string())?;

        let buf = read_message_raw(&mut self.stream).map_err(|e| e.to_string())?;
        let resp: DaemonResponse =
            serde_json::from_slice(&buf).map_err(|e| format!("Invalid response: {}", e))?;
        resp.action_response
            .ok_or_else(|| "No action response".into())
    }

    /// Send a session command (start/end).
    pub fn session(&mut self, cmd: &SessionCommand) -> Result<SessionResponse, String> {
        let msg = DaemonMessage {
            action_request: None,
            session_command: Some(cmd.clone()),
            server_command: None,
        };
        let data = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;
        write_message_raw(&mut self.stream, &data).map_err(|e| e.to_string())?;

        let buf = read_message_raw(&mut self.stream).map_err(|e| e.to_string())?;
        let resp: DaemonResponse =
            serde_json::from_slice(&buf).map_err(|e| format!("Invalid response: {}", e))?;
        resp.session_response
            .ok_or_else(|| "No session response".into())
    }

    /// Get daemon status.
    pub fn status(&mut self) -> Result<StatusResponse, String> {
        let msg = DaemonMessage {
            action_request: None,
            session_command: None,
            server_command: Some(ServerCommand::Status),
        };
        let data = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;
        write_message_raw(&mut self.stream, &data).map_err(|e| e.to_string())?;

        let buf = read_message_raw(&mut self.stream).map_err(|e| e.to_string())?;
        let resp: DaemonResponse =
            serde_json::from_slice(&buf).map_err(|e| format!("Invalid response: {}", e))?;
        resp.status_response
            .ok_or_else(|| "No status response".into())
    }

    /// Get loaded rules from daemon.
    pub fn rules(&mut self) -> Result<RulesResponse, String> {
        let msg = DaemonMessage {
            action_request: None,
            session_command: None,
            server_command: Some(ServerCommand::Rules),
        };
        let data = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;
        write_message_raw(&mut self.stream, &data).map_err(|e| e.to_string())?;

        let buf = read_message_raw(&mut self.stream).map_err(|e| e.to_string())?;
        let resp: DaemonResponse =
            serde_json::from_slice(&buf).map_err(|e| format!("Invalid response: {}", e))?;
        resp.rules_response
            .ok_or_else(|| "No rules response".into())
    }

    /// Request graceful daemon shutdown.
    pub fn shutdown(&mut self) -> Result<(), String> {
        let msg = DaemonMessage {
            action_request: None,
            session_command: None,
            server_command: Some(ServerCommand::Shutdown),
        };
        let data = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;
        write_message_raw(&mut self.stream, &data).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(unix)]
fn connect_stream(path: &str) -> Result<Box<dyn Stream>, String> {
    use std::os::unix::net::UnixStream;
    use std::time::Duration;
    let stream =
        UnixStream::connect(path).map_err(|e| format!("Cannot connect to {}: {}", path, e))?;
    stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
    Ok(Box::new(stream))
}

#[cfg(windows)]
fn connect_stream(path: &str) -> Result<Box<dyn Stream>, String> {
    use std::os::windows::io::FromRawHandle;
    use winapi::um::fileapi::CreateFileW;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE};

    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            3, // OPEN_EXISTING
            0,
            std::ptr::null_mut(),
        )
    };
    if handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        return Err(format!(
            "Cannot connect to pipe {}: {}",
            path,
            std::io::Error::last_os_error()
        ));
    }
    use std::fs::File;
    let file = unsafe { File::from_raw_handle(handle as _) };
    Ok(Box::new(file))
}

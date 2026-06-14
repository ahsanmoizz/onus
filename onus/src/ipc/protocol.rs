//! Length-prefixed JSON IPC protocol.
//!
//! Wire format: [4-byte big-endian payload length][JSON payload bytes]
//! Simple enough to implement in any language in <100 lines.

use serde::{de::DeserializeOwned, Serialize};
use std::io::{self, Read, Write};

/// Maximum message size (16 MB) to prevent memory exhaustion.
pub const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

/// Write a length-prefixed JSON message to a stream.
pub fn write_message<W: Write, T: Serialize>(writer: &mut W, message: &T) -> io::Result<()> {
    let json = serde_json::to_vec(message)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if json.len() > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Message too large: {} bytes", json.len()),
        ));
    }

    let len_bytes = (json.len() as u32).to_be_bytes();
    writer.write_all(&len_bytes)?;
    writer.write_all(&json)?;
    writer.flush()?;
    Ok(())
}

/// Read a length-prefixed JSON message from a stream.
pub fn read_message<R: Read, T: DeserializeOwned>(reader: &mut R) -> io::Result<T> {
    let buf = read_message_raw(reader)?;
    serde_json::from_slice(&buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Read raw bytes of a length-prefixed message without deserializing.
pub fn read_message_raw<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;

    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Message too large: {} bytes", len),
        ));
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

/// Write raw bytes as a length-prefixed message.
pub fn write_message_raw<W: Write>(writer: &mut W, data: &[u8]) -> io::Result<()> {
    if data.len() > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Message too large: {} bytes", data.len()),
        ));
    }
    let len_bytes = (data.len() as u32).to_be_bytes();
    writer.write_all(&len_bytes)?;
    writer.write_all(data)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let mut buf = Vec::new();
        let msg = crate::ipc::ActionResponse {
            version: 1,
            session_id: "test-session".into(),
            sequence: 1,
            decision: crate::Verdict::Block,
            rule_id: Some("SAFETY_001".into()),
            rule_name: Some("destructive-filesystem-command".into()),
            correction: Some("Blocked for safety".into()),
            latency_us: 1234,
            reversibility: Some(crate::Reversibility::Irreversible),
        };

        write_message(&mut buf, &msg).unwrap();
        let mut cursor = std::io::Cursor::new(&buf[..]);
        let decoded: crate::ipc::ActionResponse = read_message(&mut cursor).unwrap();

        assert_eq!(decoded.decision, crate::Verdict::Block);
        assert_eq!(decoded.rule_id.unwrap(), "SAFETY_001");
        assert_eq!(decoded.latency_us, 1234);
    }
}

//! Bounded serialization: do not allocate the entire wire object before checking.
use crate::LiteralError;
use serde::Serialize;
use std::io::{self, Write};
struct Bounded {
    bytes: Vec<u8>,
    limit: usize,
    exceeded: bool,
}
impl Write for Bounded {
    fn write(&mut self, value: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(value.len()) > self.limit {
            self.exceeded = true;
            return Err(io::Error::other("wire bound"));
        }
        self.bytes.extend_from_slice(value);
        Ok(value.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(crate) fn encode<T: Serialize>(value: &T, limit: usize) -> Result<Vec<u8>, LiteralError> {
    let mut writer = Bounded {
        bytes: Vec::new(),
        limit,
        exceeded: false,
    };
    if serde_json::to_writer(&mut writer, value).is_err() {
        return Err(if writer.exceeded {
            LiteralError::OutputLimit
        } else {
            LiteralError::InvalidWire
        });
    }
    Ok(writer.bytes)
}

use crate::{BridgeError, Result};

pub const MAX_FUEL: u64 = 500_000_000;
pub const MAX_MEMORY_BYTES: usize = 128 * 1024 * 1024;

/// Per-invocation limits, selected before admission and retained by the handle.
/// Fuel is a VM instruction budget, not elapsed time. Never retry at a higher
/// budget implicitly; a caller may explicitly admit a new handle instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub fuel: u64,
    pub memory_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            // Whole-corpus literals, unlike the original tiny probes, exceed
            // 100M in both guest optimization profiles. Keep the 500M hard cap.
            fuel: 250_000_000,
            memory_bytes: MAX_MEMORY_BYTES,
        }
    }
}
impl Limits {
    pub fn validate(self) -> Result<()> {
        if self.fuel == 0
            || self.fuel > MAX_FUEL
            || self.memory_bytes == 0
            || self.memory_bytes > MAX_MEMORY_BYTES
        {
            return Err(BridgeError::InvalidLimits);
        }
        Ok(())
    }
}

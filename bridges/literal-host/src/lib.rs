//! Narrow Wasm bridge for Ketho literal rendering; no generic plugin host, WASI,
//! network, filesystem, process, source acquisition or signature authority.
mod limits;
mod observed;
mod runtime;
mod slot;
pub use limits::{Limits, MAX_FUEL, MAX_MEMORY_BYTES};
pub use observed::{ObservedSnapshot, Usage};
pub use runtime::{ModuleHandle, Receipt};
use sha2::{Digest, Sha256};
pub use slot::{ModuleSlot, Selection, Snapshot};
pub use wow_render_contract::{
    LiteralBridge, LiteralError, LiteralInput, Request, Response, SCHEMA, SelectedModule,
};

pub fn module_digest(bytes: &[u8]) -> String {
    let hex: String = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("sha256:{hex}")
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BridgeError {
    InvalidLimits,
    ModuleSize,
    DigestMismatch,
    InvalidModule,
    ImportsDenied,
    IncompatibleAbi,
    ExecutionFailed,
    FuelExhausted,
    InvalidRange,
    InvalidResponse,
    StaleSelection,
    EpochExhausted,
    Poisoned,
    Input(LiteralError),
    Render(LiteralError),
}
impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "literal bridge: {self:?}")
    }
}
impl std::error::Error for BridgeError {}
pub type Result<T> = std::result::Result<T, BridgeError>;

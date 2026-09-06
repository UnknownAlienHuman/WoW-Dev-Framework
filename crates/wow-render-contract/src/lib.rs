//! Versioned, narrow bridge for literal rendering, not a generic plugin/RPC API.
mod codec;
mod types;
use serde::{Deserialize, Serialize};
pub use types::*;

pub const ABI_VERSION: i32 = 1;
pub const SCHEMA: u32 = 1;
pub const MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 2 * MAX_OUTPUT_BYTES + 1024;
pub const MAX_TEXT_BYTES: usize = 64 * 1024;
pub const MAX_ITEMS: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiteralError {
    InvalidIdentifier,
    UnsafeDocumentation,
    DuplicateName,
    InputLimit,
    OutputLimit,
    UnsupportedLiteral,
    InvalidWire,
    IncompatibleSchema,
    BridgeFailure,
}
impl std::fmt::Display for LiteralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "literal rendering failed: {self:?}")
    }
}
impl std::error::Error for LiteralError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LiteralInput {
    Events(Vec<EventLiteral>),
    CVars(Vec<String>),
    Enums {
        enums: Vec<EnumDeclaration>,
        constants: Vec<ConstantGroup>,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: u32,
    pub max_output_bytes: usize,
    pub input: LiteralInput,
}
impl Request {
    pub fn validate(&self) -> Result<(), LiteralError> {
        if self.schema != SCHEMA {
            return Err(LiteralError::IncompatibleSchema);
        }
        if self.max_output_bytes > MAX_OUTPUT_BYTES {
            return Err(LiteralError::OutputLimit);
        }
        let count = match &self.input {
            LiteralInput::Events(values) => values.len(),
            LiteralInput::CVars(values) => values.len(),
            LiteralInput::Enums { enums, constants } => enums.len().saturating_add(constants.len()),
        };
        if count > MAX_ITEMS {
            return Err(LiteralError::InputLimit);
        }
        Ok(())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, LiteralError> {
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(LiteralError::InputLimit);
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| LiteralError::InvalidWire)?;
        value.validate()?;
        Ok(value)
    }
    pub fn encode(&self) -> Result<Vec<u8>, LiteralError> {
        self.validate()?;
        let bytes = codec::encode(self, MAX_REQUEST_BYTES)?;
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(LiteralError::InputLimit);
        }
        // Reject non-finite f64 data that serde_json otherwise turns into null.
        Self::decode(&bytes)?;
        Ok(bytes)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub schema: u32,
    pub result: Result<String, LiteralError>,
}
impl Response {
    pub fn encode(&self) -> Result<Vec<u8>, LiteralError> {
        codec::encode(self, MAX_RESPONSE_BYTES)
    }
    pub fn decode(bytes: &[u8], output_limit: usize) -> Result<Self, LiteralError> {
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(LiteralError::OutputLimit);
        }
        let response: Self =
            serde_json::from_slice(bytes).map_err(|_| LiteralError::InvalidWire)?;
        if response.schema != SCHEMA {
            return Err(LiteralError::IncompatibleSchema);
        }
        if response
            .result
            .as_ref()
            .is_ok_and(|text| text.len() > output_limit || text.len() > MAX_OUTPUT_BYTES)
        {
            return Err(LiteralError::OutputLimit);
        }
        Ok(response)
    }
}
/// Both native and Wasm implementations consume the same closed operation set.
/// Callers retain the selected implementation identity for the whole operation.
pub trait LiteralBridge {
    fn render(&self, request: &Request) -> Result<String, LiteralError>;
}

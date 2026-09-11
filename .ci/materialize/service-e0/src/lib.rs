#![forbid(unsafe_code)]

//! Versioned in-process E0-F operations over one atomically captured evidence snapshot.
//!
//! This crate is not a daemon, CLI, LSP, MCP server, source loader, or live-client bridge.
//! It owns stable operation and result envelopes while preserving project/Reference/rule
//! authority boundaries.

mod error;
mod host;
mod identity;
mod model;
mod registry;

pub use error::{ServiceError, ServiceErrorCode, ServiceResult};
pub use host::ServiceHost;
pub use model::{
    OperationSelector, ServiceFailure, ServiceFailureCode, ServiceReadView, ServiceRequest,
    ServiceResponse, ServiceResponseStatus, ServiceSnapshot,
};
pub use registry::{OperationDescriptor, OperationRegistry};

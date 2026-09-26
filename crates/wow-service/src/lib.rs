#![forbid(unsafe_code)]

//! Transport-neutral E0 orchestration for exact World of Warcraft analysis contexts.
//!
//! The service owns request validation, exact context acquisition, operation
//! idempotency, conservative status derivation, deterministic presentation, and
//! canonical result envelopes. It does not parse source, execute Lua, select a
//! replacement generation, or reproduce lower-owner semantic algorithms.

pub mod annotation_admin;
mod backend;
mod configuration;
mod error;
pub mod graph;
mod identity;
pub mod local;
mod model;
mod operation;
mod presentation;
pub mod reference_admin;
mod service;

pub use backend::{OwnedServiceBackend, ServiceBackend, ServiceBackendStatus};
pub use configuration::{
    DeferredOperation, ServiceBudgets, ServiceConfiguration, ServiceConfigurationBuilder,
};
pub use error::{ServiceError, ServiceErrorCode, ServiceResult};
pub use model::{
    BlockerKind, CapabilityState, CausalRelation, CheckContext, CheckRequest, CheckScope,
    CleanEvaluation, ComponentHealth, ComponentSnapshot, ContextIdentity, ExactSourceLocation,
    FindingOrigin, GenerationSelector, GenericFinding, PresentationGraph, PresentationNode,
    PresentationNodeKind, PresentationRelation, PresentationRelationKind, RawFinding, RuleBlocker,
    RuleEvaluation, RuleEvaluationState, RuleFinding, ServiceSemanticStatus, StatusRequest,
};
pub use operation::{OperationId, OperationRegistrySnapshot};
pub use service::{CheckResult, Service, ServiceResultEnvelope, StatusResult};

pub use local::{
    LOCAL_INPUT_MAX_BYTES, LocalCommand, LocalOperationResult, LocalProjectBackend,
    LocalProjectInput, execute_local,
};

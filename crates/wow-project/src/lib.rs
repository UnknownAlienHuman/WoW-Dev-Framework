#![forbid(unsafe_code)]

//! Exact, immutable project-generation ownership for explicit World of Warcraft
//! Lua inputs.
//!
//! `wow-project` owns project configuration, source inventory, analyzer-report
//! binding, source handles, generation derivation, atomic publication, and
//! read-only views. It does not discover files, execute Lua, infer platform
//! truth from analyzer observations, or publish partial candidate state.

mod analyzer;
mod configuration;
pub mod disk;
mod error;
mod generation;
pub mod graph;
mod identity;
mod ids;
mod inventory;
pub mod load;
pub mod platform_source;
mod publication;
mod registry;
pub mod replay;
mod snapshot;
mod update;
pub mod xml_bindings;
pub mod xml_lua;

pub use analyzer::{
    ProjectAnalyzerBinding, ProjectAnalyzerCapabilityRecord, ProjectAnalyzerCapabilityScope,
    ProjectAnalyzerCapabilityState,
};
pub use configuration::{
    AnalyzerBindingDeclaration, PROJECT_CONFIGURATION_SCHEMA_VERSION, PROJECT_CONTRACT_ID,
    PROJECT_GENERATION_SCHEMA_VERSION, PROJECT_SNAPSHOT_SCHEMA_VERSION, PackageXmlBindingProfile,
    ProjectBudgetPolicy, ProjectCapabilityPolicy, ProjectConfiguration,
    ProjectConfigurationBuilder, ProjectKind,
};
pub use error::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};
pub use generation::ProjectGenerationCandidate;
pub use ids::{ProjectFileId, ProjectId, ProjectSourceOriginId, ProjectWorkspaceId};
pub use inventory::{
    ProjectFileManifestEntry, ProjectFileRole, ProjectInputBundle, ProjectInputFile,
    ProjectInputInventory, ProjectLanguageKind,
};
pub use publication::ProjectPublisher;
pub use registry::{
    ProjectFileRecord, ProjectSourceOrigin, ProjectSourceOriginKind, ProjectSourceRegistry,
};
pub use snapshot::{
    ProjectDeferredCapability, ProjectPublicationStatus, ProjectSnapshot, ProjectSourceArtifact,
    ProjectView,
};
pub use update::{
    ProjectFileOperation, ProjectLibraryOperation, ProjectUpdateOutcome, ProjectUpdateRequest,
};

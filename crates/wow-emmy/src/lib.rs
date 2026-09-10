#![forbid(unsafe_code)]

//! Explicit, deterministic workspace and analyzer-adapter boundaries for WoW Lua.
//!
//! `wow-emmy` never discovers a current directory, addon, client, Git checkout,
//! profile, or source generation implicitly. Lua semantic correctness is owned
//! by the single EmmyLua adapter; this crate does not contain a second parser.

pub mod compatibility;
pub mod syntax;
pub mod workspace;

pub use compatibility::{
    EmmyCompatibilityError, EmmyCompatibilityErrorCode, EmmyCompatibilityResult,
    backend_identity_from_report,
};
pub use syntax::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE,
    EmmyDiagnosticClassification, EmmyDiagnosticRollout, EmmyDiagnosticSeverity,
    EmmySyntaxDiagnostic, EmmySyntaxDiagnosticKind, EmmySyntaxError, EmmySyntaxErrorCode,
    EmmySyntaxFileReport, EmmySyntaxReport, EmmySyntaxResult, analyze_syntax,
};
pub use workspace::{
    EmmyBackendIdentity, EmmyWorkspaceError, EmmyWorkspaceErrorCode, EmmyWorkspaceResult,
    LuaWorkspaceFile, LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot,
    LuaWorkspaceUniverse,
};

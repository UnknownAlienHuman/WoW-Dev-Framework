//! Explicit configuration for the shared EmmyLua analysis owner.
//!
//! The framework fixes the target runtime to Lua 5.1. Callers may configure
//! upstream diagnostics, but this boundary never discovers or loads `.emmyrc`
//! files on its own.

use std::sync::Arc;

pub use emmylua_code_analysis::{DiagnosticSeveritySetting, EmmyrcDiagnostic};
use emmylua_code_analysis::{Emmyrc, EmmyrcLuaVersion};

/// Supported configuration for one shared Main + Library analysis.
#[derive(Debug, Clone)]
pub struct EmmyAnalysisConfiguration {
    diagnostics: EmmyrcDiagnostic,
}

impl EmmyAnalysisConfiguration {
    /// Use the caller-supplied upstream diagnostic settings with the fixed Lua
    /// 5.1 runtime selected by the framework.
    #[must_use]
    pub const fn new(diagnostics: EmmyrcDiagnostic) -> Self {
        Self { diagnostics }
    }

    /// Exact upstream diagnostic settings retained by this configuration.
    #[must_use]
    pub const fn diagnostics(&self) -> &EmmyrcDiagnostic {
        &self.diagnostics
    }

    /// Configuration equality for cache/session admission. JSON values are
    /// compared structurally, so map insertion order does not create a false
    /// configuration change.
    #[must_use]
    pub fn equivalent(&self, other: &Self) -> bool {
        serde_json::to_value(&self.diagnostics).ok()
            == serde_json::to_value(&other.diagnostics).ok()
    }

    pub(crate) fn upstream(&self) -> Arc<Emmyrc> {
        let mut configuration = Emmyrc::default();
        configuration.runtime.version = EmmyrcLuaVersion::Lua51;
        configuration.diagnostics = self.diagnostics.clone();
        Arc::new(configuration)
    }
}

impl Default for EmmyAnalysisConfiguration {
    fn default() -> Self {
        Self::new(EmmyrcDiagnostic::default())
    }
}

impl From<EmmyrcDiagnostic> for EmmyAnalysisConfiguration {
    fn from(diagnostics: EmmyrcDiagnostic) -> Self {
        Self::new(diagnostics)
    }
}

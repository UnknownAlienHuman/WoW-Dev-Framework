//! Explicit configuration for the shared EmmyLua analysis owner.
//!
//! The framework fixes the target runtime to Lua 5.1. Callers may configure
//! upstream diagnostics, but this boundary never discovers or loads `.emmyrc`
//! files on its own.

use std::{collections::HashSet, hash::Hash, sync::Arc};

pub use emmylua_code_analysis::{DiagnosticSeveritySetting, EmmyrcDiagnostic};
use emmylua_code_analysis::{Emmyrc, EmmyrcLuaVersion};
use lsp_types::DiagnosticSeverity;

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

    /// Compare analysis-relevant settings without serializing them. Upstream
    /// treats these lists as sets or OR-matched regexes; order and duplicates
    /// do not change analysis. The language-server interval is not engine state.
    #[must_use]
    pub fn equivalent(&self, other: &Self) -> bool {
        let left = &self.diagnostics;
        let right = &other.diagnostics;
        left.enable == right.enable
            && same_members(&left.disable, &right.disable)
            && same_members(&left.enables, &right.enables)
            && same_members(&left.globals, &right.globals)
            && same_members(&left.globals_regex, &right.globals_regex)
            && left.severity.len() == right.severity.len()
            && left.severity.iter().all(|(code, severity)| {
                right.severity.get(code).is_some_and(|other_severity| {
                    DiagnosticSeverity::from(*severity) == DiagnosticSeverity::from(*other_severity)
                })
            })
    }

    pub(crate) fn upstream(&self) -> Arc<Emmyrc> {
        let mut configuration = Emmyrc::default();
        configuration.runtime.version = EmmyrcLuaVersion::Lua51;
        configuration.diagnostics = self.diagnostics.clone();
        Arc::new(configuration)
    }
}

fn same_members<T: Eq + Hash>(left: &[T], right: &[T]) -> bool {
    left == right || left.iter().collect::<HashSet<_>>() == right.iter().collect::<HashSet<_>>()
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

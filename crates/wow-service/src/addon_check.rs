//! Direct addon language checking without the legacy generic platform.
//!
//! The first slice consumes exact already-acquired Main, Library and optional
//! virtual snapshots. Disk/TOC/XML acquisition and virtual-to-source projection
//! remain Project responsibilities; this owner composes one configured Emmy
//! session and publishes only diagnostics with exact user-source locations.

use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::SourceSpan;
use wow_emmy::{
    ConfiguredAnalyzerSession, ConfiguredSessionError, ConfiguredSessionErrorCode,
    EmmyAnalysisConfiguration, EmmyDiagnosticSeverity, EmmySyntaxDiagnostic,
    EmmySyntaxDiagnosticKind, LuaWorkspaceSnapshot,
};

use crate::{ExactSourceLocation, ServiceError, ServiceErrorCode, ServiceResult};

const RESULT_SCHEMA: &str = "wow-service/addon-check/1";

/// Exact optional mapped/virtual Main input from Project.
#[derive(Debug, Clone)]
pub struct AddonCheckVirtualInput {
    snapshot: LuaWorkspaceSnapshot,
}

impl AddonCheckVirtualInput {
    #[must_use]
    pub fn new(snapshot: LuaWorkspaceSnapshot) -> Self {
        Self { snapshot }
    }

    #[must_use]
    pub const fn snapshot(&self) -> &LuaWorkspaceSnapshot {
        &self.snapshot
    }
}

/// Exact source inputs for the first direct addon-check slice.
#[derive(Debug, Clone)]
pub struct AddonCheckRequest {
    main: LuaWorkspaceSnapshot,
    libraries: Vec<LuaWorkspaceSnapshot>,
    virtual_units: Option<AddonCheckVirtualInput>,
    input_limitations: Vec<AddonCheckLimitation>,
    configuration: EmmyAnalysisConfiguration,
}

impl AddonCheckRequest {
    #[must_use]
    pub fn new(
        main: LuaWorkspaceSnapshot,
        libraries: Vec<LuaWorkspaceSnapshot>,
        configuration: EmmyAnalysisConfiguration,
    ) -> Self {
        Self {
            main,
            libraries,
            virtual_units: None,
            input_limitations: Vec::new(),
            configuration,
        }
    }

    #[must_use]
    pub fn with_virtual_units(mut self, virtual_units: AddonCheckVirtualInput) -> Self {
        self.virtual_units = Some(virtual_units);
        self
    }

    /// Preserve limitations discovered while selecting and preparing exact
    /// Project input. A later, more specific located limitation replaces the
    /// same-code fallback that this first Service slice would otherwise add.
    #[must_use]
    pub fn with_input_limitations(mut self, input_limitations: Vec<AddonCheckLimitation>) -> Self {
        self.input_limitations = input_limitations;
        self
    }

    #[must_use]
    pub const fn main(&self) -> &LuaWorkspaceSnapshot {
        &self.main
    }

    #[must_use]
    pub fn libraries(&self) -> &[LuaWorkspaceSnapshot] {
        &self.libraries
    }

    #[must_use]
    pub const fn virtual_units(&self) -> Option<&AddonCheckVirtualInput> {
        self.virtual_units.as_ref()
    }

    #[must_use]
    pub fn input_limitations(&self) -> &[AddonCheckLimitation] {
        &self.input_limitations
    }

    #[must_use]
    pub const fn configuration(&self) -> &EmmyAnalysisConfiguration {
        &self.configuration
    }
}

/// Requested behavior implemented by this result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonCheckScope {
    LuaTypes,
}

/// Whether every required fact for the requested scope was available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonCheckCompleteness {
    Complete,
    Partial,
}

/// Which exact analysis source set produced a diagnostic. Virtual findings are
/// emitted only after Project supplies an exact source projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonCheckSourceRole {
    Main,
    Virtual,
}

/// One unchanged upstream finding projected through the direct service owner.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonCheckDiagnostic {
    source_role: AddonCheckSourceRole,
    category: Box<str>,
    code: Box<str>,
    message: Box<str>,
    kind: EmmySyntaxDiagnosticKind,
    severity: EmmyDiagnosticSeverity,
    path: Box<str>,
    content_sha256: Box<str>,
    span: SourceSpan,
}

impl AddonCheckDiagnostic {
    #[must_use]
    pub const fn source_role(&self) -> AddonCheckSourceRole {
        self.source_role
    }

    #[must_use]
    pub fn category(&self) -> &str {
        &self.category
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub const fn kind(&self) -> EmmySyntaxDiagnosticKind {
        self.kind
    }

    #[must_use]
    pub const fn severity(&self) -> EmmyDiagnosticSeverity {
        self.severity
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

/// Stable reasons why this first slice cannot claim full addon semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonCheckLimitationCode {
    AddonChunkContextUnavailable,
    VirtualSourceProjectionUnavailable,
    XmlHandlerContextUnavailable,
}

/// One explicit analysis limitation kept separate from findings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonCheckLimitation {
    code: AddonCheckLimitationCode,
    message: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<ExactSourceLocation>,
}

impl AddonCheckLimitation {
    #[must_use]
    pub fn new(code: AddonCheckLimitationCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
        }
    }

    #[must_use]
    pub fn with_location(mut self, location: ExactSourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    #[must_use]
    pub const fn code(&self) -> AddonCheckLimitationCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub const fn location(&self) -> Option<&ExactSourceLocation> {
        self.location.as_ref()
    }
}

/// Direct language-check result. Findings survive partial context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonCheckResult {
    schema: &'static str,
    scope: AddonCheckScope,
    completeness: AddonCheckCompleteness,
    diagnostics: Vec<AddonCheckDiagnostic>,
    limitations: Vec<AddonCheckLimitation>,
}

impl AddonCheckResult {
    #[must_use]
    pub const fn scope(&self) -> AddonCheckScope {
        self.scope
    }

    #[must_use]
    pub const fn completeness(&self) -> AddonCheckCompleteness {
        self.completeness
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[AddonCheckDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub fn limitations(&self) -> &[AddonCheckLimitation] {
        &self.limitations
    }
}

/// Run the first direct addon-language slice over exact caller-supplied inputs.
///
/// No Reference, Rules, compatibility report, Project publication, graph or
/// persistent store is created. The result is intentionally partial until addon
/// chunk context and virtual-source projection reach this same owner.
pub fn check_addon(
    request: AddonCheckRequest,
    stop: &AtomicBool,
) -> ServiceResult<AddonCheckResult> {
    let AddonCheckRequest {
        main,
        libraries,
        virtual_units,
        input_limitations,
        configuration,
    } = request;

    let configured_virtual = virtual_units.map(|input| input.snapshot);

    let mut session =
        ConfiguredAnalyzerSession::open(main, &libraries, configured_virtual, configuration, stop)
            .map_err(map_session_error)?;
    let reports = session.diagnostics(stop).map_err(map_session_error)?;

    let mut diagnostics = Vec::new();
    append_diagnostics(
        &mut diagnostics,
        AddonCheckSourceRole::Main,
        reports.main.diagnostics(),
    );
    // Virtual diagnostics remain part of the shared configured analysis but
    // cannot become user-visible until Project maps every virtual byte range to
    // exact XML source. Publishing the synthetic virtual path/span here would
    // violate the direct result's source-location contract.
    diagnostics.sort_unstable();

    let mut limitations = input_limitations;
    add_fallback_limitation(
        &mut limitations,
        AddonCheckLimitationCode::AddonChunkContextUnavailable,
        "addon-name/private-table chunk arguments are not yet bound to Main files",
    );
    if reports.virtual_units.is_some() {
        add_fallback_limitation(
            &mut limitations,
            AddonCheckLimitationCode::VirtualSourceProjectionUnavailable,
            "virtual Lua diagnostics are not yet projected through the exact XML source map",
        );
        add_fallback_limitation(
            &mut limitations,
            AddonCheckLimitationCode::XmlHandlerContextUnavailable,
            "implicit XML handler receiver and parameter context is not yet supplied",
        );
    }
    limitations.sort_unstable();
    limitations.dedup();

    Ok(AddonCheckResult {
        schema: RESULT_SCHEMA,
        scope: AddonCheckScope::LuaTypes,
        completeness: AddonCheckCompleteness::Partial,
        diagnostics,
        limitations,
    })
}

fn add_fallback_limitation(
    limitations: &mut Vec<AddonCheckLimitation>,
    code: AddonCheckLimitationCode,
    message: &'static str,
) {
    if limitations
        .iter()
        .all(|limitation| limitation.code() != code)
    {
        limitations.push(AddonCheckLimitation::new(code, message));
    }
}

fn append_diagnostics(
    output: &mut Vec<AddonCheckDiagnostic>,
    role: AddonCheckSourceRole,
    diagnostics: &[EmmySyntaxDiagnostic],
) {
    output.extend(diagnostics.iter().map(|diagnostic| AddonCheckDiagnostic {
        source_role: role,
        category: diagnostic.category().into(),
        code: diagnostic.upstream_code().into(),
        message: diagnostic.message().into(),
        kind: diagnostic.kind(),
        severity: diagnostic.normalized_severity(),
        path: diagnostic.path().into(),
        content_sha256: diagnostic.content_sha256().into(),
        span: diagnostic.span(),
    }));
}

fn map_session_error(source: ConfiguredSessionError) -> ServiceError {
    let code = match source.code() {
        ConfiguredSessionErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        ConfiguredSessionErrorCode::InvalidInputs => ServiceErrorCode::InvalidConfiguration,
        ConfiguredSessionErrorCode::NativeFailure => ServiceErrorCode::ComponentUnavailable,
        ConfiguredSessionErrorCode::Poisoned => ServiceErrorCode::InternalContractViolation,
    };
    ServiceError::new(code, source.message())
}

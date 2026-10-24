//! Direct addon language checking without the legacy generic platform.
//!
//! The first slice consumes exact already-acquired Main, Library and optional
//! virtual snapshots. Disk/TOC/XML acquisition remains a Project responsibility;
//! this owner composes one configured Emmy session and projects its diagnostics.

use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{ProjectGenerationId, SourceSpan};
use wow_emmy::{
    ConfiguredAnalyzerSession, ConfiguredSessionError, ConfiguredSessionErrorCode,
    EmmyAnalysisConfiguration, EmmyDiagnosticSeverity, EmmySyntaxDiagnostic,
    EmmySyntaxDiagnosticKind, LuaWorkspaceSnapshot,
};

use crate::{ServiceError, ServiceErrorCode, ServiceResult};

const RESULT_SCHEMA: &str = "wow-service/addon-check/1";

/// Exact optional mapped/virtual Main input from Project.
#[derive(Debug, Clone)]
pub struct AddonCheckVirtualInput {
    snapshot: LuaWorkspaceSnapshot,
    project_generation: ProjectGenerationId,
}

impl AddonCheckVirtualInput {
    #[must_use]
    pub fn new(snapshot: LuaWorkspaceSnapshot, project_generation: ProjectGenerationId) -> Self {
        Self {
            snapshot,
            project_generation,
        }
    }

    #[must_use]
    pub const fn snapshot(&self) -> &LuaWorkspaceSnapshot {
        &self.snapshot
    }

    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.project_generation
    }
}

/// Exact source inputs for the first direct addon-check slice.
#[derive(Debug, Clone)]
pub struct AddonCheckRequest {
    main: LuaWorkspaceSnapshot,
    libraries: Vec<LuaWorkspaceSnapshot>,
    virtual_units: Option<AddonCheckVirtualInput>,
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
            configuration,
        }
    }

    #[must_use]
    pub fn with_virtual_units(mut self, virtual_units: AddonCheckVirtualInput) -> Self {
        self.virtual_units = Some(virtual_units);
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

/// Which exact analysis source set produced a diagnostic.
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
}

impl AddonCheckLimitation {
    #[must_use]
    pub const fn code(&self) -> AddonCheckLimitationCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Upstream analysis identities used by this service result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonCheckAnalysis {
    main_analysis_id: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    virtual_analysis_id: Option<Box<str>>,
}

impl AddonCheckAnalysis {
    #[must_use]
    pub fn main_analysis_id(&self) -> &str {
        &self.main_analysis_id
    }

    #[must_use]
    pub fn virtual_analysis_id(&self) -> Option<&str> {
        self.virtual_analysis_id.as_deref()
    }
}

/// Direct language-check result. Findings survive partial context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddonCheckResult {
    schema: &'static str,
    scope: AddonCheckScope,
    completeness: AddonCheckCompleteness,
    main_snapshot_id: Box<str>,
    library_snapshot_ids: Vec<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    virtual_snapshot_id: Option<Box<str>>,
    analysis: AddonCheckAnalysis,
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
    pub fn main_snapshot_id(&self) -> &str {
        &self.main_snapshot_id
    }

    #[must_use]
    pub fn library_snapshot_ids(&self) -> &[Box<str>] {
        &self.library_snapshot_ids
    }

    #[must_use]
    pub fn virtual_snapshot_id(&self) -> Option<&str> {
        self.virtual_snapshot_id.as_deref()
    }

    #[must_use]
    pub const fn analysis(&self) -> &AddonCheckAnalysis {
        &self.analysis
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
        configuration,
    } = request;

    let main_snapshot_id: Box<str> = main.snapshot_id().into();
    let mut library_snapshot_ids = libraries
        .iter()
        .map(|library| Box::<str>::from(library.snapshot_id()))
        .collect::<Vec<_>>();
    library_snapshot_ids.sort_unstable();
    let virtual_snapshot_id = virtual_units
        .as_ref()
        .map(|input| Box::<str>::from(input.snapshot.snapshot_id()));
    let configured_virtual = virtual_units.map(|input| (input.snapshot, input.project_generation));

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
    if let Some(virtual_report) = &reports.virtual_units {
        append_diagnostics(
            &mut diagnostics,
            AddonCheckSourceRole::Virtual,
            virtual_report.diagnostics(),
        );
    }
    diagnostics.sort_unstable();

    let mut limitations = vec![AddonCheckLimitation {
        code: AddonCheckLimitationCode::AddonChunkContextUnavailable,
        message: "addon-name/private-table chunk arguments are not yet bound to Main files".into(),
    }];
    if reports.virtual_units.is_some() {
        limitations.extend([
            AddonCheckLimitation {
                code: AddonCheckLimitationCode::VirtualSourceProjectionUnavailable,
                message:
                    "virtual Lua diagnostics are not yet projected through the exact XML source map"
                        .into(),
            },
            AddonCheckLimitation {
                code: AddonCheckLimitationCode::XmlHandlerContextUnavailable,
                message: "implicit XML handler receiver and parameter context is not yet supplied"
                    .into(),
            },
        ]);
    }
    limitations.sort_unstable();

    Ok(AddonCheckResult {
        schema: RESULT_SCHEMA,
        scope: AddonCheckScope::LuaTypes,
        completeness: AddonCheckCompleteness::Partial,
        main_snapshot_id,
        library_snapshot_ids,
        virtual_snapshot_id,
        analysis: AddonCheckAnalysis {
            main_analysis_id: reports.main.analysis_id().into(),
            virtual_analysis_id: reports
                .virtual_units
                .as_ref()
                .map(|report| Box::<str>::from(report.analysis_id())),
        },
        diagnostics,
        limitations,
    })
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

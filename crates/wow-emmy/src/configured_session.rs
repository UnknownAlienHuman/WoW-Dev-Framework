//! Configured Main + Library + optional virtual-unit analysis.
//!
//! One retained upstream owner serves diagnostics and, only when requested,
//! member/call/local-flow facts. The session does not discover files or load a
//! framework compatibility report.

use std::{
    fmt,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use serde::Serialize;
use wow_core::ProjectGenerationId;

use crate::{
    EmmyLocalFlowReport, EmmySyntaxReport, LuaWorkspaceSnapshot,
    configuration::EmmyAnalysisConfiguration,
    references::{MemberCallSession, MemberCallSessionQueryProfile, RegisteredMemberAnalysis},
};

/// Stable failure class for the configured retained owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfiguredSessionErrorCode {
    Cancelled,
    InvalidInputs,
    NativeFailure,
    Poisoned,
}

/// Bounded failure without upstream panic payloads or host paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredSessionError {
    code: ConfiguredSessionErrorCode,
    message: Box<str>,
}

impl ConfiguredSessionError {
    #[must_use]
    pub const fn code(&self) -> ConfiguredSessionErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ConfiguredSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ConfiguredSessionError {}

pub type ConfiguredSessionResult<T> = Result<T, ConfiguredSessionError>;

/// Diagnostics from the physical Main and optional mapped virtual Main units.
#[derive(Debug)]
pub struct ConfiguredDiagnosticReports {
    pub main: EmmySyntaxReport,
    pub virtual_units: Option<EmmySyntaxReport>,
}

/// Optional semantic output from the same configured owner.
pub struct ConfiguredSemanticReports {
    pub syntax: EmmySyntaxReport,
    pub member_session: MemberCallSession,
    pub local_flow: EmmyLocalFlowReport,
}

/// One configured upstream analysis over explicit caller-supplied snapshots.
#[derive(Debug)]
pub struct ConfiguredAnalyzerSession {
    main: LuaWorkspaceSnapshot,
    libraries: Vec<LuaWorkspaceSnapshot>,
    virtual_units: Option<(LuaWorkspaceSnapshot, ProjectGenerationId)>,
    configuration: EmmyAnalysisConfiguration,
    semantic: RegisteredMemberAnalysis,
    healthy: bool,
}

impl ConfiguredAnalyzerSession {
    /// Open one retained analysis. Lua 5.1 is fixed by the configuration
    /// boundary; embedded STD declarations are initialized once in this owner.
    pub fn open(
        main: LuaWorkspaceSnapshot,
        libraries: &[LuaWorkspaceSnapshot],
        virtual_units: Option<(LuaWorkspaceSnapshot, ProjectGenerationId)>,
        configuration: EmmyAnalysisConfiguration,
        stop: &AtomicBool,
    ) -> ConfiguredSessionResult<Self> {
        checkpoint(stop)?;
        catch_unwind(AssertUnwindSafe(|| {
            let library_refs = libraries.iter().collect::<Vec<_>>();
            let virtual_ref = virtual_units.as_ref().map(|(snapshot, _)| snapshot);
            let mut semantic =
                RegisteredMemberAnalysis::new(&main, &library_refs, virtual_ref, stop)
                    .map_err(member_error)?;
            checkpoint(stop)?;

            // RegisteredMemberAnalysis already parses under Lua 5.1. Replace
            // its fixed diagnostic policy before any diagnostic collection,
            // then add upstream embedded STD declarations to the same owner.
            semantic.analysis.update_config(configuration.upstream());
            semantic.analysis.init_std_lib(None);
            checkpoint(stop)?;

            Ok(Self {
                main,
                libraries: libraries.to_vec(),
                virtual_units,
                configuration,
                semantic,
                healthy: true,
            })
        }))
        .unwrap_or_else(|_| {
            Err(error(
                ConfiguredSessionErrorCode::NativeFailure,
                "configured native analysis failed while opening the owner",
            ))
        })
    }

    #[must_use]
    pub const fn main(&self) -> &LuaWorkspaceSnapshot {
        &self.main
    }

    #[must_use]
    pub const fn configuration(&self) -> &EmmyAnalysisConfiguration {
        &self.configuration
    }

    #[must_use]
    pub fn virtual_units(&self) -> Option<(&LuaWorkspaceSnapshot, ProjectGenerationId)> {
        self.virtual_units
            .as_ref()
            .map(|(snapshot, generation)| (snapshot, *generation))
    }

    /// Admission check for retaining this owner. A changed Main snapshot,
    /// diagnostic policy, Library set, virtual snapshot or project generation
    /// requires a cold owner.
    #[must_use]
    pub fn compatible(
        &self,
        main: &LuaWorkspaceSnapshot,
        libraries: &[LuaWorkspaceSnapshot],
        virtual_units: Option<(&LuaWorkspaceSnapshot, ProjectGenerationId)>,
        configuration: &EmmyAnalysisConfiguration,
    ) -> bool {
        let mut expected_libraries = self
            .libraries
            .iter()
            .map(LuaWorkspaceSnapshot::snapshot_id)
            .collect::<Vec<_>>();
        let mut observed_libraries = libraries
            .iter()
            .map(LuaWorkspaceSnapshot::snapshot_id)
            .collect::<Vec<_>>();
        expected_libraries.sort_unstable();
        observed_libraries.sort_unstable();
        let expected_virtual = self
            .virtual_units()
            .map(|(snapshot, generation)| (snapshot.snapshot_id(), generation));
        let observed_virtual =
            virtual_units.map(|(snapshot, generation)| (snapshot.snapshot_id(), generation));

        self.healthy
            && self.main.snapshot_id() == main.snapshot_id()
            && expected_libraries == observed_libraries
            && expected_virtual == observed_virtual
            && self.configuration.equivalent(configuration)
    }

    /// Collect only configured language diagnostics. No member/call/local-flow
    /// collector runs on this path.
    pub fn diagnostics(
        &mut self,
        stop: &AtomicBool,
    ) -> ConfiguredSessionResult<ConfiguredDiagnosticReports> {
        if !self.healthy {
            return Err(error(
                ConfiguredSessionErrorCode::Poisoned,
                "configured native analysis owner is unavailable after failure",
            ));
        }
        checkpoint(stop)?;
        let result = catch_unwind(AssertUnwindSafe(|| self.collect_diagnostics(stop)))
            .unwrap_or_else(|_| {
                Err(error(
                    ConfiguredSessionErrorCode::NativeFailure,
                    "configured diagnostic collection failed",
                ))
            });
        self.healthy = !matches!(
            &result,
            Err(failure) if failure.code == ConfiguredSessionErrorCode::NativeFailure
        );
        result
    }

    /// Collect selected semantic facts from the same configured owner. Callers
    /// that only need Lua/type diagnostics must use [`Self::diagnostics`].
    pub fn semantic_reports(
        &mut self,
        profile: MemberCallSessionQueryProfile<'_>,
        include_function_calls: bool,
        stop: &AtomicBool,
    ) -> ConfiguredSessionResult<ConfiguredSemanticReports> {
        if !self.healthy {
            return Err(error(
                ConfiguredSessionErrorCode::Poisoned,
                "configured native analysis owner is unavailable after failure",
            ));
        }
        checkpoint(stop)?;
        let result = catch_unwind(AssertUnwindSafe(|| {
            let syntax = crate::syntax::analyzer::collect_registered(
                &self.semantic.analysis,
                &self.main,
                &self.semantic.main_root,
                stop,
            )
            .map_err(syntax_error)?;
            checkpoint(stop)?;
            let virtual_snapshot = self.virtual_units.as_ref().map(|(snapshot, _)| snapshot);
            let project_generation = self
                .virtual_units
                .as_ref()
                .map(|(_, generation)| *generation);
            let member_session = self
                .semantic
                .collect(
                    &self.main,
                    virtual_snapshot,
                    project_generation,
                    profile,
                    include_function_calls,
                    stop,
                )
                .map_err(member_error)?;
            checkpoint(stop)?;
            let local_flow = crate::flow::collect_registered(
                &self.semantic.analysis,
                &self.main,
                &self.semantic.main_root,
                &member_session.member_calls,
                stop,
            )
            .map_err(flow_error)?;
            checkpoint(stop)?;
            Ok(ConfiguredSemanticReports {
                syntax,
                member_session,
                local_flow,
            })
        }))
        .unwrap_or_else(|_| {
            Err(error(
                ConfiguredSessionErrorCode::NativeFailure,
                "configured semantic collection failed",
            ))
        });
        self.healthy = !matches!(
            &result,
            Err(failure) if failure.code == ConfiguredSessionErrorCode::NativeFailure
        );
        result
    }

    fn collect_diagnostics(
        &self,
        stop: &AtomicBool,
    ) -> ConfiguredSessionResult<ConfiguredDiagnosticReports> {
        let main = crate::syntax::analyzer::collect_registered(
            &self.semantic.analysis,
            &self.main,
            &self.semantic.main_root,
            stop,
        )
        .map_err(syntax_error)?;
        checkpoint(stop)?;
        let virtual_units = self
            .virtual_units
            .as_ref()
            .map(|(snapshot, _)| {
                let root = member_virtual_root("xml-virtual", snapshot.snapshot_id());
                crate::syntax::analyzer::collect_registered(
                    &self.semantic.analysis,
                    snapshot,
                    &root,
                    stop,
                )
                .map_err(syntax_error)
            })
            .transpose()?;
        checkpoint(stop)?;
        Ok(ConfiguredDiagnosticReports {
            main,
            virtual_units,
        })
    }
}

fn checkpoint(stop: &AtomicBool) -> ConfiguredSessionResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(error(
            ConfiguredSessionErrorCode::Cancelled,
            "configured analysis cancelled by the caller",
        ))
    } else {
        Ok(())
    }
}

fn member_error(source: crate::EmmyMemberCallError) -> ConfiguredSessionError {
    use crate::EmmyMemberCallErrorCode as Code;
    let code = match source.code() {
        Code::Cancelled => ConfiguredSessionErrorCode::Cancelled,
        Code::IncompatibleBackend
        | Code::DuplicateWorkspaceSnapshot
        | Code::InvalidMainWorkspace
        | Code::InvalidLibraryWorkspace => ConfiguredSessionErrorCode::InvalidInputs,
        _ => ConfiguredSessionErrorCode::NativeFailure,
    };
    error(code, source.to_string())
}

fn syntax_error(source: crate::EmmySyntaxError) -> ConfiguredSessionError {
    let code = match source.code() {
        crate::EmmySyntaxErrorCode::Cancelled => ConfiguredSessionErrorCode::Cancelled,
        crate::EmmySyntaxErrorCode::IncompatibleBackend => {
            ConfiguredSessionErrorCode::InvalidInputs
        }
        _ => ConfiguredSessionErrorCode::NativeFailure,
    };
    error(code, source.to_string())
}

fn flow_error(source: crate::EmmyLocalFlowError) -> ConfiguredSessionError {
    let code = if source.code() == crate::EmmyLocalFlowErrorCode::Cancelled {
        ConfiguredSessionErrorCode::Cancelled
    } else {
        ConfiguredSessionErrorCode::NativeFailure
    };
    error(code, source.to_string())
}

fn error(code: ConfiguredSessionErrorCode, message: impl Into<Box<str>>) -> ConfiguredSessionError {
    ConfiguredSessionError {
        code,
        message: message.into(),
    }
}

// Keep this path contract aligned with RegisteredMemberAnalysis registration.
// The snapshot identity remains exact; no disk directory is inspected.
fn member_virtual_root(role: &str, snapshot_id: &str) -> PathBuf {
    let stable = snapshot_id
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() {
                char::from(byte)
            } else {
                '_'
            }
        })
        .collect::<String>();
    std::env::temp_dir()
        .join("wow-emmy-member-calls-v1")
        .join(role)
        .join(stable)
}

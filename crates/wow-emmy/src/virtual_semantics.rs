//! Generation-bound semantic analysis of owner-supplied virtual Lua units.
//!
//! Virtual units share the exact Main/Library `EmmyLuaAnalysis` session used for
//! ordinary member facts. This module retains only normalized diagnostics and
//! direct-member facts; it does not create callback wrappers, implicit locals,
//! runtime dispatch claims, or public upstream handles.

use serde::Serialize;
use wow_core::ProjectGenerationId;

use crate::LuaWorkspaceSnapshot;
use crate::references::{EmmyMemberCallError, EmmyMemberCallErrorCode, EmmyMemberCallReport};
use crate::syntax::EmmySyntaxReport;

pub const VIRTUAL_SEMANTIC_PROFILE: &str = "wow-emmy/virtual-main-library-semantics/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VirtualSemanticReport {
    profile: &'static str,
    project_generation: ProjectGenerationId,
    main_snapshot_id: Box<str>,
    virtual_snapshot_id: Box<str>,
    library_snapshot_ids: Vec<Box<str>>,
    syntax_report: EmmySyntaxReport,
    member_call_report: EmmyMemberCallReport,
    wrapper_profile: &'static str,
    analysis_id: Box<str>,
}

impl VirtualSemanticReport {
    pub(crate) fn new(
        project_generation: ProjectGenerationId,
        main: &LuaWorkspaceSnapshot,
        virtual_units: &LuaWorkspaceSnapshot,
        library_snapshot_ids: Vec<Box<str>>,
        syntax_report: EmmySyntaxReport,
        member_call_report: EmmyMemberCallReport,
    ) -> Result<Self, EmmyMemberCallError> {
        if syntax_report.workspace_snapshot_id() != virtual_units.snapshot_id()
            || member_call_report.main_snapshot_id() != virtual_units.snapshot_id()
            || member_call_report
                .library_snapshot_ids()
                .ne(library_snapshot_ids.iter().map(|value| value.as_ref()))
        {
            return Err(EmmyMemberCallError::new(
                EmmyMemberCallErrorCode::AnalyzerFileRegistrationFailed,
                "virtual semantic reports do not match the exact session inputs",
                None,
            ));
        }
        let syntax_paths = syntax_report
            .files()
            .iter()
            .map(|file| file.path())
            .collect::<Vec<_>>();
        let fact_paths = member_call_report
            .files()
            .iter()
            .map(|file| file.path())
            .collect::<Vec<_>>();
        let workspace_paths = virtual_units
            .files()
            .iter()
            .map(|file| file.path())
            .collect::<Vec<_>>();
        if syntax_paths != workspace_paths || fact_paths != workspace_paths {
            return Err(EmmyMemberCallError::new(
                EmmyMemberCallErrorCode::AnalyzerFileRegistrationFailed,
                "virtual semantic file reports do not close over the virtual workspace",
                None,
            ));
        }
        let analysis_id = crate::references::canonical_id(
            "emmy-virtual-semantics:sha256:",
            &(
                VIRTUAL_SEMANTIC_PROFILE,
                project_generation,
                main.snapshot_id(),
                virtual_units.snapshot_id(),
                &library_snapshot_ids,
                syntax_report.analysis_id(),
                member_call_report.analysis_id(),
                "none_exact_unwrapped_source",
            ),
        )?;
        Ok(Self {
            profile: VIRTUAL_SEMANTIC_PROFILE,
            project_generation,
            main_snapshot_id: main.snapshot_id().into(),
            virtual_snapshot_id: virtual_units.snapshot_id().into(),
            library_snapshot_ids,
            syntax_report,
            member_call_report,
            wrapper_profile: "none_exact_unwrapped_source",
            analysis_id: analysis_id.into_boxed_str(),
        })
    }

    #[must_use]
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }

    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.project_generation
    }

    #[must_use]
    pub fn main_snapshot_id(&self) -> &str {
        &self.main_snapshot_id
    }

    #[must_use]
    pub fn virtual_snapshot_id(&self) -> &str {
        &self.virtual_snapshot_id
    }

    pub fn library_snapshot_ids(&self) -> impl Iterator<Item = &str> {
        self.library_snapshot_ids.iter().map(|value| value.as_ref())
    }

    #[must_use]
    pub const fn syntax_report(&self) -> &EmmySyntaxReport {
        &self.syntax_report
    }

    #[must_use]
    pub const fn member_call_report(&self) -> &EmmyMemberCallReport {
        &self.member_call_report
    }

    #[must_use]
    pub const fn wrapper_profile(&self) -> &'static str {
        self.wrapper_profile
    }
}

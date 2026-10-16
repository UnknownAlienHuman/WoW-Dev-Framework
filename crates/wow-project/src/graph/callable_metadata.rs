//! Borrowed callable facets from one genuine source-projection owner.
//! No source, shape, report reconstruction or graph authority is exposed.
use serde::Serialize;
use wow_core::{ContentDigest, SourceContent, SourceSpan};
use wow_emmy::function_calls::{
    FunctionCallReport, PhysicalFunctionDeclarationRecipe, PhysicalFunctionSignatureRecipe,
    SourceFunctionDeclaration, SourceFunctionFact, SourceFunctionKind, SourceFunctionSignature,
};

use super::{ProjectGraphFunction, ProjectGraphProvenance, invalid};
use crate::{ProjectPhase, ProjectResult};

/// State of the original selected native facet, without inventing a missing value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectCallableFacetState {
    NotSelected,
    Observed,
    Partial,
    Unavailable,
    Unsupported,
}

/// Nearest enclosing function scope; not immediate statement position or runtime loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectCallableScope {
    Chunk,
    ChunkScopeClosure,
    NestedClosure,
}

#[derive(Debug, Serialize)]
struct DeclarationMetadata<'a> {
    state: ProjectCallableFacetState,
    #[serde(skip_serializing_if = "Option::is_none")]
    recipe: Option<PhysicalFunctionDeclarationRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observation: Option<&'a SourceFunctionDeclaration>,
}

#[derive(Debug, Serialize)]
struct SignatureMetadata<'a> {
    state: ProjectCallableFacetState,
    #[serde(skip_serializing_if = "Option::is_none")]
    recipe: Option<PhysicalFunctionSignatureRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observation: Option<&'a SourceFunctionSignature>,
}

/// Serialize-only metadata borrowing exact immutable native records.
/// Only this owner can construct it; it does not attest published graph membership.
#[derive(Debug, Serialize)]
pub struct ProjectCallableMetadata<'a> {
    function: &'a ProjectGraphFunction,
    function_analysis_id: &'a str,
    content_digest: ContentDigest<SourceContent>,
    scope: ProjectCallableScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_function_id: Option<&'a str>,
    declaration: DeclarationMetadata<'a>,
    signature: SignatureMetadata<'a>,
}

impl<'a> ProjectCallableMetadata<'a> {
    #[must_use]
    pub const fn function(&self) -> &'a ProjectGraphFunction {
        self.function
    }
    #[must_use]
    pub const fn function_analysis_id(&self) -> &'a str {
        self.function_analysis_id
    }
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest<SourceContent> {
        self.content_digest
    }
    #[must_use]
    pub const fn scope(&self) -> ProjectCallableScope {
        self.scope
    }
    #[must_use]
    pub const fn parent_function_id(&self) -> Option<&'a str> {
        self.parent_function_id
    }
    #[must_use]
    pub const fn declaration_state(&self) -> ProjectCallableFacetState {
        self.declaration.state
    }
    #[must_use]
    pub const fn signature_state(&self) -> ProjectCallableFacetState {
        self.signature.state
    }
    #[must_use]
    pub const fn declaration_span(&self) -> Option<SourceSpan> {
        match self.declaration.observation {
            Some(SourceFunctionDeclaration::Observed {
                declaration_span, ..
            }) => Some(*declaration_span),
            _ => None,
        }
    }
}

impl ProjectGraphProvenance {
    /// Exact native-ID join. An absent ID is None; broken retained custody refuses.
    /// This borrows one function, its parent and original facets without cloning bodies.
    pub fn callable_metadata(
        &self,
        function_id: &str,
    ) -> ProjectResult<Option<ProjectCallableMetadata<'_>>> {
        let Ok(index) = self
            .functions()
            .binary_search_by(|function| function.function_id.as_str().cmp(function_id))
        else {
            return Ok(None);
        };
        let function = &self.functions()[index];
        let report = self.function_call_report().ok_or_else(invalid)?;
        let index = report
            .functions()
            .binary_search_by(|fact| fact.fact_id().cmp(function_id))
            .map_err(|_| invalid())?;
        let fact = &report.functions()[index];
        let digest =
            crate::identity::parse_source_digest(fact.content_digest(), ProjectPhase::View)?;
        let source = self
            .files()
            .binary_search_by(|file| file.path.as_str().cmp(fact.path()))
            .map(|index| &self.files()[index])
            .map_err(|_| invalid())?;
        let handle = self
            .source_handles()
            .get(&function.source_handle_id)
            .ok_or_else(invalid)?;
        let evidence = self
            .evidence()
            .get(&function.evidence_id)
            .ok_or_else(invalid)?;
        if function.function_id != fact.fact_id()
            || function.path != fact.path()
            || function.kind != fact.kind()
            || function.span != fact.span()
            || source.content_digest != digest
            || handle.handle_id() != function.source_handle_id
            || handle.path().as_str() != fact.path()
            || handle.span() != fact.span()
            || *handle.content_digest() != digest
            || self.context().project_generation().is_none()
            || handle.project_generation() != self.context().project_generation()
            || handle.reference_generation() != Some(self.context().reference_generation())
            || evidence.evidence_id() != function.evidence_id
            || evidence.context_id() != self.context().context_id()
            || !evidence
                .source_handle_ids()
                .contains(&function.source_handle_id)
        {
            return Err(invalid());
        }
        let scope = native_scope(report, fact)?;
        let declaration_state = match (report.declaration_recipe(), fact.declaration()) {
            (None, None) => ProjectCallableFacetState::NotSelected,
            (Some(_), Some(SourceFunctionDeclaration::Observed { .. })) => {
                ProjectCallableFacetState::Observed
            }
            (Some(_), Some(SourceFunctionDeclaration::Unavailable { .. })) => {
                ProjectCallableFacetState::Unavailable
            }
            (Some(_), Some(SourceFunctionDeclaration::Unsupported { .. })) => {
                ProjectCallableFacetState::Unsupported
            }
            _ => return Err(invalid()),
        };
        let signature_state = match (report.signature_recipe(), fact.signature()) {
            (None, None) => ProjectCallableFacetState::NotSelected,
            (Some(_), Some(observation)) => {
                if observation.observed_types().is_none() {
                    ProjectCallableFacetState::Unavailable
                } else if observation.comparable().is_some() {
                    ProjectCallableFacetState::Observed
                } else {
                    ProjectCallableFacetState::Partial
                }
            }
            _ => return Err(invalid()),
        };
        Ok(Some(ProjectCallableMetadata {
            function,
            function_analysis_id: report.analysis_id(),
            content_digest: digest,
            scope,
            parent_function_id: fact.parent_function_id(),
            declaration: DeclarationMetadata {
                state: declaration_state,
                recipe: report.declaration_recipe(),
                observation: fact.declaration(),
            },
            signature: SignatureMetadata {
                state: signature_state,
                recipe: report.signature_recipe(),
                observation: fact.signature(),
            },
        }))
    }
}

/// Source-owned native function custody, independent of the selected graph layout.
/// It does not attest accepted Graph membership, canonical APIs or runtime loading.
pub struct ProjectFunctionBindingMetadata<'a> {
    fact: &'a SourceFunctionFact,
    function_analysis_id: &'a str,
    source_handle: wow_core::SourceHandle,
    scope: ProjectCallableScope,
}
impl<'a> ProjectFunctionBindingMetadata<'a> {
    pub const fn fact(&self) -> &'a SourceFunctionFact {
        self.fact
    }
    pub const fn function_analysis_id(&self) -> &'a str {
        self.function_analysis_id
    }
    pub const fn source_handle(&self) -> &wow_core::SourceHandle {
        &self.source_handle
    }
    pub const fn scope(&self) -> ProjectCallableScope {
        self.scope
    }
    pub fn binding_name(&self) -> Option<&'a wow_emmy::function_calls::SourceFunctionBindingName> {
        self.fact.binding_name()
    }
    pub fn binding_name_state(&self) -> wow_emmy::function_calls::SourceFunctionBindingState {
        self.fact.binding_name_state()
    }
}
impl crate::ProjectView {
    /// Borrow one exact native function after joining the immutable source registry.
    pub fn function_binding_metadata(
        &self,
        id: &str,
    ) -> ProjectResult<Option<ProjectFunctionBindingMetadata<'_>>> {
        let Some(report) = self.function_call_report() else {
            return Ok(None);
        };
        let Ok(index) = report
            .functions()
            .binary_search_by(|fact| fact.fact_id().cmp(id))
        else {
            return Ok(None);
        };
        let fact = &report.functions()[index];
        let bad_binding = || {
            crate::ProjectError::new(
                crate::ProjectErrorCode::AnalyzerSnapshotMismatch,
                ProjectPhase::View,
                "native function fact does not bind this immutable Project source",
            )
        };
        let file = self.file_by_path(fact.path())?.ok_or_else(bad_binding)?;
        if report.main_snapshot_id() != self.local_flow_report().main_snapshot_id()
            || file.content_digest().to_string() != fact.content_digest()
        {
            return Err(bad_binding());
        }
        let source_handle = self.source_handle(fact.path(), fact.span(), None)?;
        Ok(Some(ProjectFunctionBindingMetadata {
            fact,
            function_analysis_id: report.analysis_id(),
            source_handle,
            scope: native_scope(report, fact)?,
        }))
    }
}
fn native_scope(
    report: &FunctionCallReport,
    fact: &SourceFunctionFact,
) -> ProjectResult<ProjectCallableScope> {
    match (fact.kind(), fact.parent_function_id()) {
        (SourceFunctionKind::Chunk, None) => Ok(ProjectCallableScope::Chunk),
        (SourceFunctionKind::Closure, Some(id)) => {
            let index = report
                .functions()
                .binary_search_by(|parent| parent.fact_id().cmp(id))
                .map_err(|_| invalid())?;
            let parent = &report.functions()[index];
            if parent.fact_id() == fact.fact_id()
                || parent.path() != fact.path()
                || parent.content_digest() != fact.content_digest()
            {
                return Err(invalid());
            }
            Ok(match parent.kind() {
                SourceFunctionKind::Chunk => ProjectCallableScope::ChunkScopeClosure,
                SourceFunctionKind::Closure => ProjectCallableScope::NestedClosure,
            })
        }
        _ => Err(invalid()),
    }
}

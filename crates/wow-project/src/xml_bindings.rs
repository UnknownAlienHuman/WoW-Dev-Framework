//! XML-to-Lua source linking. XML owners form data queries; only wow-emmy
//! resolves Lua declarations. No receiver construction or callback invocation.
mod receivers;
mod scripts;

pub use scripts::XmlInheritedScriptSource;

pub use receivers::{
    XmlReceiverBlocker, XmlReceiverBlockerKind, XmlReceiverMixinSource, XmlReceiverSources,
};

use crate::load::{ProjectLoadPlan, XmlElementRecord, XmlScriptSource, XmlSourceSpan};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;
use wow_core::{CanonicalResult, ContentDigest, ProjectGenerationId, SourceContent};
use wow_emmy::bindings::{SymbolLookupReport, SymbolLookupState, supported_path};

pub const XML_LUA_BINDING_PROFILE: &str = "wow-project/xml-lua-bindings/3";
const MAX_BINDINGS: usize = 4096;
const MAX_QUERY_REFS: usize = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlLuaBindingKind {
    Mixin,
    Function,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlLuaBindingState {
    UniqueAnalyzerDeclaration,
    DeclaredMixinCandidates,
    Ambiguous,
    NotObserved,
    Indeterminate,
    ReceiverNotResolved,
    UnsupportedPath,
    InvalidSource,
    SourceParseFailed,
}
impl XmlLuaBindingState {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UniqueAnalyzerDeclaration => "xml.lua.unique_analyzer_declaration",
            Self::DeclaredMixinCandidates => "xml.lua.declared_mixin_candidates",
            Self::Ambiguous => "xml.lua.ambiguous",
            Self::NotObserved => "xml.lua.not_observed",
            Self::Indeterminate => "xml.lua.indeterminate",
            Self::ReceiverNotResolved => "xml.lua.receiver_not_resolved",
            Self::UnsupportedPath => "xml.lua.unsupported_path",
            Self::InvalidSource => "xml.lua.invalid_source",
            Self::SourceParseFailed => "xml.lua.source_parse_failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaBinding {
    pub document: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub element_id: String,
    pub kind: XmlLuaBindingKind,
    pub ordinal: usize,
    pub attribute_span: XmlSourceSpan,
    /// Keys into the shared analyzer report, never executable expressions.
    pub queries: Vec<String>,
    /// Key into the shared receiver-source graph; omitted for non-method records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_source_id: Option<String>,
    /// Consuming XML declaration for an inherited handler source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_id: Option<String>,
    pub state: XmlLuaBindingState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectXmlLuaBindings {
    profile: &'static str,
    project_generation: ProjectGenerationId,
    load_plan_digest: ContentDigest<CanonicalResult>,
    bindings: Vec<XmlLuaBinding>,
    receiver_sources: BTreeMap<String, XmlReceiverSources>,
    inherited_script_sources: Vec<XmlInheritedScriptSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol_lookup: Option<SymbolLookupReport>,
    receiver_semantics: &'static str,
    analysis_id: Box<str>,
}
impl ProjectXmlLuaBindings {
    #[must_use]
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }
    #[must_use]
    pub fn bindings(&self) -> &[XmlLuaBinding] {
        &self.bindings
    }
    #[must_use]
    pub fn receiver_sources(&self) -> &BTreeMap<String, XmlReceiverSources> {
        &self.receiver_sources
    }
    #[must_use]
    pub fn inherited_script_sources(&self) -> &[XmlInheritedScriptSource] {
        &self.inherited_script_sources
    }
    /// Source enumeration only; effective inherited dispatch is not evaluated.
    #[must_use]
    pub fn inherited_sources_complete(&self) -> bool {
        self.receiver_sources
            .values()
            .all(|sources| sources.complete)
            && self
                .inherited_script_sources
                .iter()
                .all(|source| source.source_complete)
    }
    #[must_use]
    pub fn symbol_lookup(&self) -> Option<&SymbolLookupReport> {
        self.symbol_lookup.as_ref()
    }
    #[must_use]
    pub fn unresolved_count(&self) -> usize {
        self.bindings
            .iter()
            .filter(|b| b.state != XmlLuaBindingState::UniqueAnalyzerDeclaration)
            .count()
    }
}

pub(crate) struct PreparedBindings {
    bindings: Vec<XmlLuaBinding>,
    receiver_sources: BTreeMap<String, XmlReceiverSources>,
    inherited_script_sources: Vec<XmlInheritedScriptSource>,
    queries: Vec<String>,
}
impl PreparedBindings {
    pub(crate) fn queries(&self) -> &[String] {
        &self.queries
    }
}
fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::AnalyzerSnapshotMismatch,
        ProjectPhase::Analyzer,
        "XML Lua binding receipt disagrees with its captured owners",
    )
}
fn exhausted() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::Analyzer,
        "XML Lua binding budget exceeded",
    )
}
type PendingBinding = (
    XmlLuaBindingKind,
    usize,
    XmlSourceSpan,
    Vec<String>,
    XmlLuaBindingState,
    Option<String>,
);

fn attribute_span(element: &XmlElementRecord, name: &str) -> ProjectResult<XmlSourceSpan> {
    element
        .attributes
        .iter()
        .find(|a| a.qualified_name == name)
        .map(|a| a.value_span.clone())
        .ok_or_else(invalid)
}

/// Whether the captured XML actually requests symbol or receiver lookup. Merely
/// indexing a file-only Ui document must not introduce pending binding health.
#[must_use]
pub fn has_bindings(plan: &ProjectLoadPlan) -> bool {
    plan.xml_documents().values().any(|index| {
        index.elements().iter().any(|e| {
            e.declaration.as_ref().is_some_and(|d| !d.mixins.is_empty())
                || e.script
                    .as_ref()
                    .is_some_and(|s| s.function_reference.is_some() || s.method_reference.is_some())
        })
    })
}
#[must_use]
pub fn has_method_bindings(plan: &ProjectLoadPlan) -> bool {
    plan.xml_documents().values().any(|index| {
        index.scripts().any(|e| {
            e.script
                .as_ref()
                .is_some_and(|s| s.method_reference.is_some())
        })
    })
}

/// A syntax-only indication that inherited handler enumeration may be needed.
#[must_use]
pub fn has_script_inheritance(plan: &ProjectLoadPlan) -> bool {
    plan.xml_documents().values().any(|index| {
        index.declarations().any(|element| {
            element
                .declaration
                .as_ref()
                .is_some_and(|d| !d.inherits.is_empty())
        })
    }) && plan.xml_documents().values().any(|index| {
        index
            .scripts()
            .any(|element| element.role == crate::load::XmlElementRole::ScriptBinding)
    })
}

pub(crate) fn prepare(
    plan: &ProjectLoadPlan,
    stop: &AtomicBool,
) -> ProjectResult<PreparedBindings> {
    let mut bindings = Vec::new();
    let mut receivers = receivers::Resolver::new(plan, stop)?;
    let mut all_queries = BTreeSet::new();
    let mut query_refs = 0usize;
    let mut query_visits = 0usize;
    let mut text_bytes = 0usize;
    let scripts::BindingSites {
        sites,
        mut inherited,
    } = scripts::collect(plan, &mut receivers, stop)?;
    for site in sites {
        let path = site.document;
        let element = site.element;
        let first_binding = bindings.len();
        crate::analyzer::checkpoint(stop)?;
        let mut pending: Vec<PendingBinding> = Vec::new();
        if let Some(declaration) = &element.declaration {
            for (ordinal, mixin) in declaration.mixins.iter().enumerate() {
                pending.push((
                    XmlLuaBindingKind::Mixin,
                    ordinal,
                    attribute_span(element, "mixin")?,
                    vec![mixin.clone()],
                    XmlLuaBindingState::Indeterminate,
                    None,
                ));
            }
        }
        if let Some(script) = &element.script {
            if let Some(function) = &script.function_reference {
                pending.push((
                    XmlLuaBindingKind::Function,
                    0,
                    attribute_span(element, "function")?,
                    vec![function.clone()],
                    if script.source_kind == XmlScriptSource::ReferenceOnly {
                        XmlLuaBindingState::Indeterminate
                    } else {
                        XmlLuaBindingState::InvalidSource
                    },
                    None,
                ));
            }
            if let Some(method) = &script.method_reference {
                let owner = site.owner;
                let mut queries = Vec::new();
                let mut receiver_source_id = None;
                let state = if script.source_kind != XmlScriptSource::ReferenceOnly
                    || !element.issues.is_empty()
                    || owner.is_some_and(|e| !e.issues.is_empty())
                {
                    XmlLuaBindingState::InvalidSource
                } else if !supported_path(method) || method.contains('.') {
                    XmlLuaBindingState::UnsupportedPath
                } else if let Some(owner) = owner {
                    let sources = receivers.resolve(&owner.occurrence_id, stop)?;
                    let mut distinct = BTreeSet::new();
                    let mut query_bytes = 0usize;
                    let mut unsupported = false;
                    for mixin in &sources.mixins {
                        crate::analyzer::checkpoint(stop)?;
                        query_visits = query_visits.checked_add(1).ok_or_else(exhausted)?;
                        if query_visits > 262_144 {
                            return Err(exhausted());
                        }
                        let length = mixin
                            .name
                            .len()
                            .checked_add(method.len())
                            .and_then(|n| n.checked_add(1))
                            .ok_or_else(exhausted)?;
                        if length > 4096 {
                            unsupported = true;
                            break;
                        }
                        let query = format!("{}.{method}", mixin.name);
                        if !supported_path(&query) {
                            unsupported = true;
                            break;
                        }
                        // Bound expansion before retaining per-handler copies;
                        // repeated names keep their distinct XML origins above.
                        if distinct.insert(query) {
                            query_bytes = query_bytes.checked_add(length).ok_or_else(exhausted)?;
                            let total_refs = query_refs
                                .checked_add(distinct.len())
                                .ok_or_else(exhausted)?;
                            let total_bytes =
                                text_bytes.checked_add(query_bytes).ok_or_else(exhausted)?;
                            if total_refs > MAX_QUERY_REFS
                                || distinct.len() > 4096
                                || total_bytes > 16 * 1024 * 1024
                            {
                                return Err(exhausted());
                            }
                        }
                    }
                    receiver_source_id = Some(sources.owner_id.clone());
                    if unsupported {
                        XmlLuaBindingState::UnsupportedPath
                    } else {
                        queries = distinct.into_iter().collect();
                        if !sources.complete || queries.is_empty() {
                            XmlLuaBindingState::ReceiverNotResolved
                        } else {
                            XmlLuaBindingState::Indeterminate
                        }
                    }
                } else {
                    XmlLuaBindingState::ReceiverNotResolved
                };
                pending.push((
                    XmlLuaBindingKind::Method,
                    0,
                    attribute_span(element, "method")?,
                    queries,
                    state,
                    receiver_source_id,
                ));
            }
        }
        for (kind, ordinal, span, mut queries, mut state, receiver_source_id) in pending {
            crate::analyzer::checkpoint(stop)?;
            if bindings.len() >= MAX_BINDINGS {
                return Err(exhausted());
            }
            if !element.issues.is_empty() {
                state = XmlLuaBindingState::InvalidSource;
            }
            if queries.iter().any(|q| !supported_path(q)) {
                state = XmlLuaBindingState::UnsupportedPath;
            }
            if matches!(
                state,
                XmlLuaBindingState::InvalidSource | XmlLuaBindingState::UnsupportedPath
            ) {
                queries.clear();
            }
            query_refs = query_refs
                .checked_add(queries.len())
                .ok_or_else(exhausted)?;
            text_bytes = text_bytes
                .checked_add(
                    path.len()
                        + receiver_source_id.as_ref().map_or(0, String::len)
                        + site.consumer_id.map_or(0, str::len)
                        + queries.iter().map(String::len).sum::<usize>(),
                )
                .ok_or_else(exhausted)?;
            if query_refs > MAX_QUERY_REFS || text_bytes > 16 * 1024 * 1024 {
                return Err(exhausted());
            }
            all_queries.extend(queries.iter().cloned());
            if all_queries.len() > 4096 {
                return Err(exhausted());
            }
            bindings.push(XmlLuaBinding {
                document: path.to_owned(),
                content_digest: site.digest,
                element_id: element.occurrence_id.clone(),
                kind,
                ordinal,
                attribute_span: span,
                queries,
                receiver_source_id,
                consumer_id: site.consumer_id.map(str::to_owned),
                state,
            });
        }
        if let Some(index) = site.inherited_index {
            if bindings.len() == first_binding {
                return Err(invalid());
            }
            inherited
                .get_mut(index)
                .ok_or_else(invalid)?
                .binding_indices = (first_binding..bindings.len()).collect();
        }
    }
    Ok(PreparedBindings {
        bindings,
        receiver_sources: receivers.into_sources(),
        inherited_script_sources: inherited,
        queries: all_queries.into_iter().collect(),
    })
}

pub(crate) fn finish(
    mut prepared: PreparedBindings,
    lookup: Option<SymbolLookupReport>,
    generation: ProjectGenerationId,
    plan: &ProjectLoadPlan,
    stop: &AtomicBool,
) -> ProjectResult<ProjectXmlLuaBindings> {
    if prepared.queries.is_empty() != lookup.is_none() {
        return Err(invalid());
    }
    if let Some(report) = &lookup {
        if report.lookups().keys().ne(prepared.queries.iter()) {
            return Err(invalid());
        }
        for binding in &mut prepared.bindings {
            crate::analyzer::checkpoint(stop)?;
            if binding.queries.is_empty() {
                continue;
            }
            if !report.source_health_complete() {
                binding.state = XmlLuaBindingState::SourceParseFailed;
                continue;
            }
            if let Some(consumer) = &binding.consumer_id {
                let sources = prepared
                    .receiver_sources
                    .get(consumer)
                    .ok_or_else(invalid)?;
                if !sources.complete {
                    binding.state = XmlLuaBindingState::ReceiverNotResolved;
                    continue;
                }
            }
            if binding.kind == XmlLuaBindingKind::Method {
                let sources = binding
                    .receiver_source_id
                    .as_ref()
                    .and_then(|id| prepared.receiver_sources.get(id))
                    .ok_or_else(invalid)?;
                if !sources.complete {
                    binding.state = XmlLuaBindingState::ReceiverNotResolved;
                    continue;
                }
            }
            let results = binding
                .queries
                .iter()
                .map(|q| report.lookups().get(q).ok_or_else(invalid))
                .collect::<ProjectResult<Vec<_>>>()?;
            binding.state = if results
                .iter()
                .any(|r| r.state == SymbolLookupState::SourceParseFailed)
            {
                XmlLuaBindingState::SourceParseFailed
            } else if results
                .iter()
                .any(|r| r.state == SymbolLookupState::Indeterminate)
            {
                XmlLuaBindingState::Indeterminate
            } else if results
                .iter()
                .any(|r| r.state == SymbolLookupState::Ambiguous)
            {
                XmlLuaBindingState::Ambiguous
            } else if results
                .iter()
                .any(|r| r.state == SymbolLookupState::UnsupportedPath)
            {
                XmlLuaBindingState::UnsupportedPath
            } else if binding.kind == XmlLuaBindingKind::Method {
                // Direct and inherited source mixins provide candidates, not
                // a constructed receiver or runtime method precedence.
                let targets: BTreeSet<_> = results
                    .iter()
                    .filter(|r| r.state == SymbolLookupState::UniqueAnalyzerDeclaration)
                    .flat_map(|r| r.targets.iter())
                    .collect();
                if targets.len() > 1 {
                    XmlLuaBindingState::Ambiguous
                } else if targets.len() == 1 {
                    XmlLuaBindingState::DeclaredMixinCandidates
                } else {
                    XmlLuaBindingState::NotObserved
                }
            } else if results
                .iter()
                .all(|r| r.state == SymbolLookupState::UniqueAnalyzerDeclaration)
            {
                XmlLuaBindingState::UniqueAnalyzerDeclaration
            } else {
                XmlLuaBindingState::NotObserved
            };
        }
    }
    #[derive(Serialize)]
    struct Identity<'a> {
        generation: ProjectGenerationId,
        load_plan: ContentDigest<CanonicalResult>,
        bindings: &'a [XmlLuaBinding],
        receiver_sources: &'a BTreeMap<String, XmlReceiverSources>,
        inherited_script_sources: &'a [XmlInheritedScriptSource],
        #[serde(skip_serializing_if = "Option::is_none")]
        symbol_analysis_id: Option<&'a str>,
    }
    let analysis_id = crate::identity::canonical_id(
        "project-xml-lua-bindings:sha256:",
        XML_LUA_BINDING_PROFILE,
        &Identity {
            generation,
            load_plan: plan.digest(),
            bindings: &prepared.bindings,
            receiver_sources: &prepared.receiver_sources,
            inherited_script_sources: &prepared.inherited_script_sources,
            symbol_analysis_id: lookup.as_ref().map(SymbolLookupReport::analysis_id),
        },
        ProjectPhase::Analyzer,
    )?;
    Ok(ProjectXmlLuaBindings {
        profile: XML_LUA_BINDING_PROFILE,
        project_generation: generation,
        load_plan_digest: plan.digest(),
        bindings: prepared.bindings,
        receiver_sources: prepared.receiver_sources,
        inherited_script_sources: prepared.inherited_script_sources,
        symbol_lookup: lookup,
        receiver_semantics: "not_evaluated",
        analysis_id,
    })
}

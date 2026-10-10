//! Package-scoped named XML queries against one original Main/Library session.
//! Local XML IDs and binding indices never become aggregate or runtime IDs.
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, ProjectGenerationId, SourceContent};
use wow_emmy::bindings::SymbolLookupReport;

use super::{
    ClassificationBudget, PreparationBudget, PreparedBindings, XmlInheritedScriptSource,
    XmlLuaBinding, XmlLuaBindingKind, XmlLuaBindingState, XmlReceiverSources, attribute_span,
    classify, exhausted, invalid, prepare_with_budget,
};
use crate::load::{
    ProjectLoadPlan, ProjectPackageLoadPhase, ProjectPackageLoadPlan, ProjectPackageMainPlan,
    ProjectPackageNode, ProjectPackageReachability,
};
use crate::{ProjectPhase, ProjectResult};

pub const PACKAGE_XML_LUA_BINDING_PROFILE: &str = "wow-project/package-xml-lua-bindings/1";
const MAX_DOCUMENTS: usize = 65_536;
const MAX_SCOPE_TEXT_BYTES: usize = 16 * 1024 * 1024;
const MAX_SERIALIZED_BYTES: usize = 64 * 1024 * 1024;

/// Exact native package scope. Reachability and phase do not attest execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageXmlLuaBindingScope {
    package: String,
    selected_toc: String,
    load_plan_digest: ContentDigest<CanonicalResult>,
    reachability: ProjectPackageReachability,
    phase: ProjectPackageLoadPhase,
}
impl ProjectPackageXmlLuaBindingScope {
    fn from_node(node: &ProjectPackageNode) -> Self {
        Self {
            package: node.package.clone(),
            selected_toc: node.selected_toc.clone(),
            load_plan_digest: node.selected_plan_digest,
            reachability: node.reachability,
            phase: node.phase,
        }
    }

    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }
    #[must_use]
    pub fn selected_toc(&self) -> &str {
        &self.selected_toc
    }
    #[must_use]
    pub const fn load_plan_digest(&self) -> ContentDigest<CanonicalResult> {
        self.load_plan_digest
    }
    #[must_use]
    pub const fn reachability(&self) -> ProjectPackageReachability {
        self.reachability
    }
    #[must_use]
    pub const fn phase(&self) -> ProjectPackageLoadPhase {
        self.phase
    }
}

/// The map key is the original local document; this is its admitted qualification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageXmlDocument {
    qualified_document: String,
    content_digest: ContentDigest<SourceContent>,
    byte_length: u64,
}
impl ProjectPackageXmlDocument {
    #[must_use]
    pub fn qualified_document(&self) -> &str {
        &self.qualified_document
    }
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest<SourceContent> {
        self.content_digest
    }
    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }
}

/// Rows and receiver/reference IDs are local to this exact selected-TOC scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageXmlLuaBindingGroup {
    scope: ProjectPackageXmlLuaBindingScope,
    documents: BTreeMap<String, ProjectPackageXmlDocument>,
    bindings: Vec<XmlLuaBinding>,
    receiver_sources: BTreeMap<String, XmlReceiverSources>,
    inherited_script_sources: Vec<XmlInheritedScriptSource>,
}
impl ProjectPackageXmlLuaBindingGroup {
    #[must_use]
    pub fn scope(&self) -> &ProjectPackageXmlLuaBindingScope {
        &self.scope
    }
    #[must_use]
    pub fn documents(&self) -> &BTreeMap<String, ProjectPackageXmlDocument> {
        &self.documents
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
    /// Source enumeration only, with all native inheritance blockers retained.
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
}

/// An owner-created address; an occurrence or row index alone has no scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageXmlLuaBindingAddress {
    analysis_id: Box<str>,
    package: String,
    load_plan_digest: ContentDigest<CanonicalResult>,
    binding_index: usize,
}
impl ProjectPackageXmlLuaBindingAddress {
    #[must_use]
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }
    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }
    #[must_use]
    pub const fn load_plan_digest(&self) -> ContentDigest<CanonicalResult> {
        self.load_plan_digest
    }
    #[must_use]
    pub const fn binding_index(&self) -> usize {
        self.binding_index
    }
}

/// Sealed aggregate retaining the original union lookup exactly once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageXmlLuaBindings {
    profile: &'static str,
    project_generation: ProjectGenerationId,
    package_load_plan_digest: ContentDigest<CanonicalResult>,
    package_main_plan_digest: ContentDigest<CanonicalResult>,
    main_snapshot_id: Box<str>,
    library_snapshot_ids: Vec<Box<str>>,
    groups: Vec<ProjectPackageXmlLuaBindingGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol_lookup: Option<SymbolLookupReport>,
    receiver_semantics: &'static str,
    analysis_id: Box<str>,
    #[serde(skip)]
    serialized_byte_length: usize,
}
impl ProjectPackageXmlLuaBindings {
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
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
    pub const fn package_load_plan_digest(&self) -> ContentDigest<CanonicalResult> {
        self.package_load_plan_digest
    }
    #[must_use]
    pub const fn package_main_plan_digest(&self) -> ContentDigest<CanonicalResult> {
        self.package_main_plan_digest
    }
    #[must_use]
    pub fn main_snapshot_id(&self) -> &str {
        &self.main_snapshot_id
    }
    pub fn library_snapshot_ids(&self) -> impl Iterator<Item = &str> {
        self.library_snapshot_ids.iter().map(AsRef::as_ref)
    }
    #[must_use]
    pub fn groups(&self) -> &[ProjectPackageXmlLuaBindingGroup] {
        &self.groups
    }
    #[must_use]
    pub fn group(&self, package: &str) -> Option<&ProjectPackageXmlLuaBindingGroup> {
        self.groups
            .iter()
            .find(|group| group.scope.package == package)
    }
    #[must_use]
    pub fn symbol_lookup(&self) -> Option<&SymbolLookupReport> {
        self.symbol_lookup.as_ref()
    }
    /// Counted with a bounded, cancellable writer before this owner is returned.
    #[must_use]
    pub const fn serialized_byte_length(&self) -> usize {
        self.serialized_byte_length
    }
    #[must_use]
    pub fn unresolved_count(&self) -> usize {
        self.groups
            .iter()
            .flat_map(|group| &group.bindings)
            .filter(|binding| binding.state != XmlLuaBindingState::UniqueAnalyzerDeclaration)
            .count()
    }

    pub fn address(
        &self,
        package: &str,
        binding_index: usize,
    ) -> ProjectResult<ProjectPackageXmlLuaBindingAddress> {
        let group = self.group(package).ok_or_else(invalid)?;
        group.bindings.get(binding_index).ok_or_else(invalid)?;
        Ok(ProjectPackageXmlLuaBindingAddress {
            analysis_id: self.analysis_id.clone(),
            package: group.scope.package.clone(),
            load_plan_digest: group.scope.load_plan_digest,
            binding_index,
        })
    }

    pub fn resolve_binding(
        &self,
        address: &ProjectPackageXmlLuaBindingAddress,
    ) -> ProjectResult<&XmlLuaBinding> {
        if address.analysis_id != self.analysis_id {
            return Err(invalid());
        }
        let group = self.group(&address.package).ok_or_else(invalid)?;
        if address.load_plan_digest != group.scope.load_plan_digest {
            return Err(invalid());
        }
        group
            .bindings
            .get(address.binding_index)
            .ok_or_else(invalid)
    }
}

struct PreparedGroup {
    scope: ProjectPackageXmlLuaBindingScope,
    documents: BTreeMap<String, ProjectPackageXmlDocument>,
    prepared: PreparedBindings,
}
pub(crate) struct PreparedPackageBindings {
    load_digest: ContentDigest<CanonicalResult>,
    main_digest: ContentDigest<CanonicalResult>,
    groups: Vec<PreparedGroup>,
    queries: Vec<String>,
}
impl PreparedPackageBindings {
    pub(crate) fn queries(&self) -> &[String] {
        &self.queries
    }
}

#[derive(Default)]
struct ScopeBudget {
    documents: usize,
    text_bytes: usize,
}
impl ScopeBudget {
    fn text(&mut self, bytes: usize) -> ProjectResult<()> {
        self.text_bytes = self.text_bytes.checked_add(bytes).ok_or_else(exhausted)?;
        if self.text_bytes > MAX_SCOPE_TEXT_BYTES {
            return Err(exhausted());
        }
        Ok(())
    }
}

/// Check owner-produced receipts, without constructing source paths or analyzing Lua.
fn validate_inputs(
    load: &ProjectPackageLoadPlan,
    main: &ProjectPackageMainPlan,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    main.validate_load_plan(load)?;
    // The sealed native load owner already enforces its package-count bound.
    if load.packages().is_empty() {
        return Err(invalid());
    }
    let mut previous = None;
    let mut sources = BTreeMap::new();
    for node in load.packages() {
        crate::analyzer::checkpoint(stop)?;
        if previous.is_some_and(|package| package >= node.package.as_str()) {
            return Err(invalid());
        }
        previous = Some(node.package.as_str());
        let plan = load.package_plan(&node.package).ok_or_else(invalid)?;
        if plan.digest() != node.selected_plan_digest || plan.selected_toc() != node.selected_toc {
            return Err(invalid());
        }
        if node.reachability == ProjectPackageReachability::Unreachable {
            continue;
        }
        for source in plan
            .sources()
            .iter()
            .filter(|source| source.path.ends_with(".lua"))
        {
            crate::analyzer::checkpoint(stop)?;
            if sources
                .insert((node.package.as_str(), source.path.as_str()), source)
                .is_some()
            {
                return Err(invalid());
            }
        }
    }
    if sources.len() != main.files().len() {
        return Err(invalid());
    }
    let mut paths = BTreeSet::new();
    for file in main.files() {
        crate::analyzer::checkpoint(stop)?;
        let source = sources
            .remove(&(file.package.as_str(), file.source_path.as_str()))
            .ok_or_else(invalid)?;
        if source.content_digest != file.content_digest
            || source.byte_length != file.byte_length
            || !paths.insert(file.project_path.as_str())
            || load
                .source_path(&file.package, &file.source_path)
                .as_deref()
                != Some(file.project_path.as_str())
        {
            return Err(invalid());
        }
    }
    Ok(())
}

pub(crate) fn prepare_packages(
    load: &ProjectPackageLoadPlan,
    main: &ProjectPackageMainPlan,
    stop: &AtomicBool,
) -> ProjectResult<PreparedPackageBindings> {
    validate_inputs(load, main, stop)?;
    let mut budget = PreparationBudget {
        limit_query_union: true,
        ..PreparationBudget::default()
    };
    let mut scopes = ScopeBudget::default();
    let mut groups = Vec::new();
    for node in load.packages() {
        crate::analyzer::checkpoint(stop)?;
        scopes.text(
            node.package
                .len()
                .checked_add(node.selected_toc.len())
                .ok_or_else(exhausted)?,
        )?;
        let plan = load.package_plan(&node.package).ok_or_else(invalid)?;
        let sources: BTreeMap<_, _> = plan
            .sources()
            .iter()
            .map(|source| (source.path.as_str(), source))
            .collect();
        let mut documents = BTreeMap::new();
        for (local, index) in plan.xml_documents() {
            crate::analyzer::checkpoint(stop)?;
            if scopes.documents >= MAX_DOCUMENTS {
                return Err(exhausted());
            }
            let source = sources.get(local.as_str()).ok_or_else(invalid)?;
            if index.document() != local || index.source_digest() != source.content_digest {
                return Err(invalid());
            }
            let qualified_document = load.source_path(&node.package, local).ok_or_else(invalid)?;
            scopes.text(
                local
                    .len()
                    .checked_add(qualified_document.len())
                    .ok_or_else(exhausted)?,
            )?;
            scopes.documents += 1;
            documents.insert(
                local.clone(),
                ProjectPackageXmlDocument {
                    qualified_document,
                    content_digest: source.content_digest,
                    byte_length: source.byte_length,
                },
            );
        }
        let prepared = prepare_with_budget(plan, &mut budget, stop)?;
        groups.push(PreparedGroup {
            scope: ProjectPackageXmlLuaBindingScope::from_node(node),
            documents,
            prepared,
        });
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(PreparedPackageBindings {
        load_digest: load.digest(),
        main_digest: main.digest(),
        groups,
        queries: budget.queries.into_iter().collect(),
    })
}

fn validate_group(
    group: &PreparedGroup,
    load: &ProjectPackageLoadPlan,
    plan: &ProjectLoadPlan,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    if group.documents.len() != plan.xml_documents().len() {
        return Err(invalid());
    }
    let sources: BTreeMap<_, _> = plan
        .sources()
        .iter()
        .map(|source| (source.path.as_str(), source))
        .collect();
    let mut elements = BTreeMap::new();
    for (local, index) in plan.xml_documents() {
        crate::analyzer::checkpoint(stop)?;
        let document = group.documents.get(local).ok_or_else(invalid)?;
        let source = sources.get(local.as_str()).ok_or_else(invalid)?;
        if index.document() != local
            || document.content_digest != index.source_digest()
            || document.content_digest != source.content_digest
            || document.byte_length != source.byte_length
            || load.source_path(&group.scope.package, local).as_deref()
                != Some(document.qualified_document.as_str())
        {
            return Err(invalid());
        }
        for element in index.elements() {
            crate::analyzer::checkpoint(stop)?;
            if elements
                .insert(element.occurrence_id.as_str(), (local.as_str(), element))
                .is_some()
            {
                return Err(invalid());
            }
        }
    }
    for binding in &group.prepared.bindings {
        crate::analyzer::checkpoint(stop)?;
        let (local, element) = elements
            .get(binding.element_id.as_str())
            .ok_or_else(invalid)?;
        let document = group.documents.get(&binding.document).ok_or_else(invalid)?;
        let attribute = match binding.kind {
            XmlLuaBindingKind::Mixin => "mixin",
            XmlLuaBindingKind::Function => "function",
            XmlLuaBindingKind::Method => "method",
        };
        let ordinal_valid = match binding.kind {
            XmlLuaBindingKind::Mixin => element
                .declaration
                .as_ref()
                .is_some_and(|declaration| binding.ordinal < declaration.mixins.len()),
            XmlLuaBindingKind::Function => {
                binding.ordinal == 0
                    && element
                        .script
                        .as_ref()
                        .is_some_and(|script| script.function_reference.is_some())
            }
            XmlLuaBindingKind::Method => {
                binding.ordinal == 0
                    && element
                        .script
                        .as_ref()
                        .is_some_and(|script| script.method_reference.is_some())
            }
        };
        if *local != binding.document
            || document.content_digest != binding.content_digest
            || !ordinal_valid
            || attribute_span(element, attribute)? != binding.attribute_span
            || binding.attribute_span.byte_start > binding.attribute_span.byte_end
            || binding.attribute_span.byte_end > document.byte_length
            || binding
                .receiver_source_id
                .as_ref()
                .is_some_and(|id| !group.prepared.receiver_sources.contains_key(id))
            || binding
                .consumer_id
                .as_ref()
                .is_some_and(|id| !plan.xml_references().declarations().contains_key(id))
        {
            return Err(invalid());
        }
    }
    let references: BTreeSet<_> = plan
        .xml_references()
        .references()
        .iter()
        .map(|reference| reference.reference_id.as_str())
        .collect();
    for (id, source) in &group.prepared.receiver_sources {
        crate::analyzer::checkpoint(stop)?;
        if id != &source.owner_id
            || !elements.contains_key(id.as_str())
            || source
                .declarations
                .iter()
                .any(|id| !plan.xml_references().declarations().contains_key(id))
            || source
                .references
                .iter()
                .any(|id| !references.contains(id.as_str()))
            || source.mixins.iter().any(|mixin| {
                !plan
                    .xml_references()
                    .declarations()
                    .contains_key(&mixin.declaration_id)
            })
            || source.blockers.iter().any(|blocker| {
                !elements.contains_key(blocker.source_id.as_str())
                    || blocker
                        .reference_id
                        .as_ref()
                        .is_some_and(|id| !references.contains(id.as_str()))
            })
            || source.complete != source.blockers.is_empty()
        {
            return Err(invalid());
        }
    }
    for source in &group.prepared.inherited_script_sources {
        crate::analyzer::checkpoint(stop)?;
        let (_, element) = elements
            .get(source.script_id.as_str())
            .ok_or_else(invalid)?;
        let script = element.script.as_ref().ok_or_else(invalid)?;
        let receiver = group
            .prepared
            .receiver_sources
            .get(&source.consumer_id)
            .ok_or_else(invalid)?;
        if !plan
            .xml_references()
            .declarations()
            .contains_key(&source.consumer_id)
            || !plan
                .xml_references()
                .declarations()
                .contains_key(&source.declaring_owner_id)
            || script.owner_occurrence_id.as_deref() != Some(source.declaring_owner_id.as_str())
            || script.source_kind != source.source_kind
            || source.source_complete
                != (receiver.complete
                    && element.issues.is_empty()
                    && script.source_kind != crate::load::XmlScriptSource::Unresolved)
        {
            return Err(invalid());
        }
        for index in &source.binding_indices {
            crate::analyzer::checkpoint(stop)?;
            let binding = group.prepared.bindings.get(*index).ok_or_else(invalid)?;
            if binding.element_id != source.script_id
                || binding.consumer_id.as_deref() != Some(source.consumer_id.as_str())
            {
                return Err(invalid());
            }
        }
    }
    Ok(())
}

/// Consume the single native lookup after validating its exact aggregate query set.
#[allow(clippy::too_many_arguments)]
pub(crate) fn finish_packages(
    pending: PreparedPackageBindings,
    load: &ProjectPackageLoadPlan,
    main: &ProjectPackageMainPlan,
    generation: ProjectGenerationId,
    main_snapshot_id: &str,
    library_ids: &[Box<str>],
    lookup: Option<SymbolLookupReport>,
    stop: &AtomicBool,
) -> ProjectResult<ProjectPackageXmlLuaBindings> {
    validate_inputs(load, main, stop)?;
    if pending.load_digest != load.digest()
        || pending.main_digest != main.digest()
        || pending.groups.len() != load.packages().len()
        || pending.queries.is_empty() != lookup.is_none()
        || main_snapshot_id.is_empty()
        || main_snapshot_id.len() > 4096
        || library_ids
            .iter()
            .any(|id| id.is_empty() || id.len() > 4096 || id.as_ref() == main_snapshot_id)
        || library_ids.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(invalid());
    }
    if let Some(report) = &lookup
        && report.lookups().keys().ne(pending.queries.iter())
    {
        return Err(invalid());
    }
    let union: BTreeSet<_> = pending
        .groups
        .iter()
        .flat_map(|group| group.prepared.queries.iter())
        .collect();
    if union.into_iter().ne(pending.queries.iter()) {
        return Err(invalid());
    }
    let mut groups = Vec::new();
    let mut budget = ClassificationBudget::default();
    for (mut group, node) in pending.groups.into_iter().zip(load.packages()) {
        crate::analyzer::checkpoint(stop)?;
        if group.scope != ProjectPackageXmlLuaBindingScope::from_node(node) {
            return Err(invalid());
        }
        let plan = load.package_plan(&node.package).ok_or_else(invalid)?;
        validate_group(&group, load, plan, stop)?;
        if let Some(report) = &lookup {
            classify(&mut group.prepared, report, &mut budget, stop)?;
        }
        groups.push(ProjectPackageXmlLuaBindingGroup {
            scope: group.scope,
            documents: group.documents,
            bindings: group.prepared.bindings,
            receiver_sources: group.prepared.receiver_sources,
            inherited_script_sources: group.prepared.inherited_script_sources,
        });
    }
    #[derive(Serialize)]
    struct Identity<'a> {
        profile: &'static str,
        generation: ProjectGenerationId,
        package_load_plan_digest: ContentDigest<CanonicalResult>,
        package_main_plan_digest: ContentDigest<CanonicalResult>,
        main_snapshot_id: &'a str,
        library_snapshot_ids: &'a [Box<str>],
        groups: &'a [ProjectPackageXmlLuaBindingGroup],
        #[serde(skip_serializing_if = "Option::is_none")]
        symbol_analysis_id: Option<&'a str>,
    }
    let identity = Identity {
        profile: PACKAGE_XML_LUA_BINDING_PROFILE,
        generation,
        package_load_plan_digest: load.digest(),
        package_main_plan_digest: main.digest(),
        main_snapshot_id,
        library_snapshot_ids: library_ids,
        groups: &groups,
        symbol_analysis_id: lookup.as_ref().map(SymbolLookupReport::analysis_id),
    };
    // Include the original report in preflight before allocating canonical
    // identity bytes. This is a size check, not a second identity recipe.
    serialized_size(&(&identity, &lookup), stop)?;
    let analysis_id = crate::identity::canonical_id(
        "project-package-xml-lua-bindings:sha256:",
        PACKAGE_XML_LUA_BINDING_PROFILE,
        &identity,
        ProjectPhase::Analyzer,
    )?;
    crate::analyzer::checkpoint(stop)?;
    let mut result = ProjectPackageXmlLuaBindings {
        profile: PACKAGE_XML_LUA_BINDING_PROFILE,
        project_generation: generation,
        package_load_plan_digest: load.digest(),
        package_main_plan_digest: main.digest(),
        main_snapshot_id: main_snapshot_id.into(),
        library_snapshot_ids: library_ids.to_vec(),
        groups,
        symbol_lookup: lookup,
        receiver_semantics: "not_evaluated",
        analysis_id,
        serialized_byte_length: 0,
    };
    result.serialized_byte_length = serialized_size(&result, stop)?;
    Ok(result)
}

/// Bound canonical identity allocation and final report bytes without buffering.
fn serialized_size<T: Serialize>(value: &T, stop: &AtomicBool) -> ProjectResult<usize> {
    struct Counter<'a> {
        bytes: usize,
        stop: &'a AtomicBool,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.stop.load(Ordering::Acquire) {
                return Err(std::io::Error::other("XML binding serialization cancelled"));
            }
            self.bytes = self
                .bytes
                .checked_add(bytes.len())
                .filter(|length| *length <= MAX_SERIALIZED_BYTES)
                .ok_or_else(|| std::io::Error::other("XML binding serialization exceeded"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    crate::analyzer::checkpoint(stop)?;
    let mut counter = Counter { bytes: 0, stop };
    let result = serde_json::to_writer(&mut counter, value);
    crate::analyzer::checkpoint(stop)?;
    result.map_err(|_| exhausted())?;
    Ok(counter.bytes)
}

//! Selected source XML inventory proof; old load errors/Partial remain untouched.
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};
use wow_core::ProjectGenerationId;
use wow_emmy::{bindings::SymbolLookupState, function_calls::SourceCallTarget};

use super::super::{load_inputs, producer_budget::ProducerBudget};
use super::{PlatformGraphProvenance, ProjectResult, invalid};
use crate::{
    ProjectView,
    load::schema::members::{
        Admission, MEMBER_ROWS, MEMBER_WORK, MemberCatalog, MemberLedger, PermittedDocument,
    },
    load::{
        LoadRecordKind, LoadSelection, ProjectPackageReachability, XmlElementRole, XmlScriptSource,
    },
    xml_bindings::{XmlLuaBindingKind, XmlLuaBindingState},
};

struct Ledger<'a> {
    retained: &'a mut ProducerBudget,
    work: usize,
    rows: usize,
}
impl MemberLedger for Ledger<'_> {
    fn visit(&mut self, count: usize, stop: &AtomicBool) -> ProjectResult<()> {
        crate::analyzer::checkpoint(stop)?;
        self.work = self
            .work
            .checked_add(count)
            .filter(|n| *n <= MEMBER_WORK)
            .ok_or_else(super::super::exhausted)?;
        Ok(())
    }
    fn rows(&mut self, count: usize, stop: &AtomicBool) -> ProjectResult<()> {
        self.visit(count, stop)?;
        self.rows = self
            .rows
            .checked_add(count)
            .filter(|n| *n <= MEMBER_ROWS)
            .ok_or_else(super::super::exhausted)?;
        Ok(())
    }
    fn charge<T: Serialize + ?Sized>(&mut self, value: &T, stop: &AtomicBool) -> ProjectResult<()> {
        // &T is sized; the writer sees the SAME value, not a second serializer.
        self.retained.charge_serialized(&value, stop)
    }
}

/// No construction/deserialization port. Borrowed native catalog/index authority.
pub(super) struct SelectedXmlCensus<'a> {
    project: &'a ProjectView,
    generation: ProjectGenerationId,
    documents: Vec<PermittedDocument<'a>>,
}
impl SelectedXmlCensus<'_> {
    pub(super) fn is_for(
        &self,
        direct: &PlatformGraphProvenance<'_>,
        stop: &AtomicBool,
    ) -> ProjectResult<bool> {
        if !std::ptr::eq(self.project, direct.project())
            || self.generation != direct.project().project_generation()
        {
            return Ok(false);
        }
        let reviewed = direct
            .project()
            .configuration()
            .xml_classification_profile();
        for document in &self.documents {
            crate::analyzer::checkpoint(stop)?;
            if !reviewed.is_some_and(|profile| {
                std::ptr::eq(document.catalog(), profile.catalog())
                    && document.witnesses().len() == document.document().elements().len()
            }) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Independent selected enumeration. Semantic legacy issues are NEVER read,
/// filtered or modified here. A refusal is not a clean zero XML degree.
pub(super) fn admit<'a>(
    direct: &'a PlatformGraphProvenance<'_>,
    retained: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Option<SelectedXmlCensus<'a>>> {
    let project = direct.project();
    let configuration = project.configuration();
    let packages = configuration.platform_packages().ok_or_else(invalid)?;
    let load = packages.load_plan();
    let coverage = load.coverage();
    let mut ledger = Ledger {
        retained,
        work: 0,
        rows: 0,
    };
    ledger.visit(1, stop)?;
    if direct.source().context() != project.snapshot().generation_context()
        || direct.source().project_snapshot_id != project.snapshot_id()
        || direct.source().analyzer_snapshot_id != project.analyzer_snapshot_id()
    {
        return Err(invalid());
    }
    // Do not reinterpret the legacy file_closure_complete field.
    if !coverage.variants_complete
        || !coverage.dependencies_complete
        || !coverage.canonical_order_complete
    {
        return Ok(None);
    }
    let scopes = load_inputs::scopes(project, stop)?;
    let mut has_xml = false;
    for scope in &scopes {
        ledger.visit(1, stop)?;
        let plan = scope.plan();
        ledger.charge(
            &(plan.digest(), plan.native_file_enumeration_complete()),
            stop,
        )?;
        if !plan.native_file_enumeration_complete()
            || scope
                .node()
                .is_some_and(|node| node.reachability != ProjectPackageReachability::Reachable)
        {
            return Ok(None);
        }
        has_xml |= !plan.xml_documents().is_empty();
        let mut source_paths = BTreeSet::new();
        for source in plan.sources() {
            ledger.rows(1, stop)?;
            ledger.charge(&source.path, stop)?;
            if !source_paths.insert(source.path.as_str()) {
                return Err(invalid());
            }
        }
        for record in plan.records() {
            ledger.visit(1, stop)?;
            ledger.charge(&(record.ordinal, record.selection, record.kind), stop)?;
            if record.selection != LoadSelection::Included {
                return Ok(None);
            }
            if matches!(
                record.kind,
                LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
            ) {
                let Some(target) = record.target.as_deref() else {
                    return Ok(None);
                };
                if !source_paths.contains(target) {
                    return Ok(None);
                }
                if record.kind == LoadRecordKind::XmlFile
                    && !plan.xml_documents().contains_key(target)
                {
                    return Ok(None);
                }
            }
        }
    }
    if !has_xml {
        if !direct.source().script_sources().is_empty()
            || !direct.source().script_sites().is_empty()
            || !direct.source().script_bindings().is_empty()
        {
            return Err(invalid());
        }
        return Ok(Some(SelectedXmlCensus {
            project,
            generation: project.project_generation(),
            documents: Vec::new(),
        }));
    }
    let Some(reviewed) = configuration.xml_classification_profile() else {
        return Ok(None);
    };
    let catalog = reviewed.catalog();
    // Configuration retains the SAME sealed catalog/source. Validation never
    // supplies a reconstructed receipt; this checks the actual native source.
    catalog.validate_source(packages.source(), stop)?;
    ledger.charge(
        &(
            project.project_generation(),
            load.digest(),
            packages.main_plan().digest(),
            catalog.receipt().digest(),
            catalog.receipt().selection_digest(),
        ),
        stop,
    )?;
    let catalog_index = MemberCatalog::new(catalog, &mut ledger, stop)?;
    let functions = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or_else(invalid)?;
    let mut documents = Vec::new();
    let mut expected_sources = BTreeMap::new();
    let mut expected_sites = BTreeSet::new();
    let mut expected_bindings = BTreeSet::new();
    let mut expected_queries = BTreeSet::new();
    let mut seen_xml_documents = BTreeSet::new();
    let mut native_site_by_key = BTreeMap::new();
    for site in direct.source().script_sites() {
        ledger.rows(1, stop)?;
        ledger.charge(
            &(
                site.package.as_deref(),
                site.document.as_deref(),
                &site.script_id,
                site.inherited,
            ),
            stop,
        )?;
        if site.inherited
            || !site.blockers.is_empty()
            || site.package.is_none()
            || site.document.is_none()
        {
            return Ok(None);
        }
        let key = (
            site.package.clone(),
            site.document.clone(),
            site.script_id.clone(),
        );
        if native_site_by_key.insert(key, site).is_some() {
            return Err(invalid());
        }
    }
    for scope in &scopes {
        ledger.visit(1, stop)?;
        let plan = scope.plan();
        let package = scope.package().ok_or_else(invalid)?;
        let bindings = scope.bindings()?.ok_or_else(invalid)?;
        let mut binding_rows = BTreeMap::new();
        for (position, binding) in bindings.bindings.iter().enumerate() {
            ledger.rows(1, stop)?;
            ledger.charge(
                &(&binding.element_id, position, binding.kind, binding.state),
                stop,
            )?;
            if binding.kind != XmlLuaBindingKind::Function
                || binding.consumer_id.is_some()
                || binding.state != XmlLuaBindingState::UniqueAnalyzerDeclaration
                || binding.queries.is_empty()
            {
                return Ok(None);
            }
            if binding_rows
                .insert(binding.element_id.as_str(), (position, binding))
                .is_some()
            {
                return Ok(None);
            }
        }
        for (local, index) in plan.xml_documents() {
            ledger.rows(1, stop)?;
            let qualified = scope.qualified_path(local)?;
            ledger.charge(&qualified, stop)?;
            if !seen_xml_documents.insert(qualified.clone()) {
                return Err(invalid());
            }
            let permission = match catalog_index.admit(index, &mut ledger, stop)? {
                Admission::Closed(proof) => proof,
                Admission::Unavailable(why) => {
                    let _ = why;
                    return Ok(None);
                }
            };
            for witness in permission.witnesses() {
                ledger.visit(1, stop)?;
                ledger.charge(
                    &(
                        witness.declaration(),
                        witness.type_component(),
                        witness.member_component(),
                    ),
                    stop,
                )?;
                let element = &index.elements()[witness.occurrence_index()];
                if let Some(declaration) = &element.declaration
                    && (declaration.parent_reference.is_some()
                        || declaration.parent_key.is_some()
                        || declaration.parent_array.is_some()
                        || !declaration.inherits.is_empty()
                        || !declaration.mixins.is_empty())
                {
                    return Ok(None);
                }
                let Some(script) = &element.script else {
                    continue;
                };
                ledger.charge(
                    &(
                        package,
                        &qualified,
                        &element.occurrence_id,
                        script.source_kind,
                    ),
                    stop,
                )?;
                let key = (
                    Some(package.to_owned()),
                    Some(qualified.clone()),
                    element.occurrence_id.clone(),
                );
                if expected_sources
                    .insert(
                        (qualified.clone(), element.occurrence_id.as_str()),
                        script.source_kind,
                    )
                    .is_some()
                {
                    return Err(invalid());
                }
                if script.source_kind != XmlScriptSource::ReferenceOnly
                    || element.role != XmlElementRole::ScriptBinding
                    || script.function_reference.is_none()
                    || script.method_reference.is_some()
                    || script.file_reference.is_some()
                    || script.inherit.is_some()
                    || script.intrinsic_order.is_some()
                {
                    return Ok(None);
                }
                let Some((position, binding)) = binding_rows.remove(element.occurrence_id.as_str())
                else {
                    return Ok(None);
                };
                if binding.document != *local || binding.content_digest != index.source_digest() {
                    return Err(invalid());
                }
                let Some(site) = native_site_by_key.remove(&key) else {
                    return Ok(None);
                };
                if site.consumer_id.as_deref() != script.owner_occurrence_id.as_deref()
                    || site.binding_index != Some(position)
                    || site.queries.len() != binding.queries.len()
                    || site.binding_ids.len() != 1
                    || !site.blockers.is_empty()
                {
                    return Ok(None);
                }
                let address = bindings.binding_address(position)?.ok_or_else(invalid)?;
                if site.package_binding_address.as_ref() != Some(&address) {
                    return Err(invalid());
                }
                let Some(lookup) = bindings.symbol_lookup else {
                    return Ok(None);
                };
                if !functions
                    .symbol_lookup_analysis_ids()
                    .iter()
                    .any(|id| id == lookup.analysis_id())
                {
                    return Err(invalid());
                }
                for (query, observed) in binding.queries.iter().zip(&site.queries) {
                    ledger.visit(1, stop)?;
                    ledger.charge(&(query, &observed.outcome), stop)?;
                    if observed.query != *query {
                        return Err(invalid());
                    }
                    let Some(resolved) = lookup.lookups().get(query) else {
                        return Ok(None);
                    };
                    if resolved.state != SymbolLookupState::UniqueAnalyzerDeclaration {
                        return Ok(None);
                    }
                    let exact = match (functions.named_targets().get(query), &observed.outcome) {
                        (
                            Some(SourceCallTarget::MainFunction { function_id }),
                            super::super::scripts::ProjectGraphScriptQueryOutcome::MainFunction {
                                function_id: found,
                            },
                        ) => function_id == found,
                        _ => false,
                    };
                    if !exact {
                        return Ok(None);
                    }
                    ledger.charge(&query, stop)?;
                    expected_queries.insert(query.as_str());
                }
                for id in &site.binding_ids {
                    ledger.charge(&id, stop)?;
                    if !expected_bindings.insert(id.as_str()) {
                        return Err(invalid());
                    }
                }
                ledger.charge(&site.site_id, stop)?;
                expected_sites.insert(site.site_id.as_str());
            }
            ledger.charge(&(index.digest(), permission.witnesses()), stop)?;
            documents.push(permission);
        }
        if !binding_rows.is_empty() {
            return Ok(None);
        }
    }
    if !native_site_by_key.is_empty() {
        return Ok(None);
    }
    for source in direct.source().script_sources() {
        ledger.visit(1, stop)?;
        ledger.charge(&(&source.document, &source.script_id), stop)?;
        if expected_sources.remove(&(source.document.clone(), source.script_id.as_str()))
            != Some(source.source_kind)
        {
            return Ok(None);
        }
    }
    if !expected_sources.is_empty() {
        return Ok(None);
    }
    for binding in direct.source().script_bindings() {
        ledger.visit(1, stop)?;
        ledger.charge(
            &(&binding.binding_id, &binding.site_id, binding.confidence),
            stop,
        )?;
        if !expected_sites.contains(binding.site_id.as_str())
            || !expected_bindings.remove(binding.binding_id.as_str())
            || binding.handler_kind != "lua_source_function"
            || binding.confidence != wow_graph::GraphConfidence::Derived
        {
            return Ok(None);
        }
    }
    if !expected_bindings.is_empty() {
        return Ok(None);
    }
    // Exact native lookup union is already admitted by finish_packages; require
    // all selected queries still have held outcomes. Unrelated mixin queries are
    // excluded above rather than repurposed as evidence of a handler.
    let package_bindings = project
        .snapshot()
        .analyzer_binding()
        .package_xml_bindings()
        .ok_or_else(invalid)?;
    match package_bindings.symbol_lookup() {
        Some(lookup)
            if lookup
                .lookups()
                .keys()
                .map(String::as_str)
                .eq(expected_queries.iter().copied()) => {}
        None if expected_queries.is_empty() => {}
        _ => return Ok(None),
    }
    ledger.visit(1, stop)?;
    Ok(Some(SelectedXmlCensus {
        project,
        generation: project.project_generation(),
        documents,
    }))
}

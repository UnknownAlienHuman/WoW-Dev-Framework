//! XML structural families through the declarative matcher. Parsing, declaration
//! linking and inline-body ownership stay in the load owner. Each family reads a
//! real preceding graph snapshot, publishes its own partition, and the final
//! crosswalk runs only after every producer has published.
use super::source_addresses::SourceGraphAddressCrosswalk;
use super::*;

use std::collections::{BTreeMap, BTreeSet};
use wow_graph::GraphAssertionKind;
use wow_project::load::xml_references::{XmlReferenceOrder, XmlReferenceResolution};
use wow_project::load::{XmlElementRole, XmlScriptSource};
use wow_recognizers::source_scripts::SourceScriptSemanticContext;
use wow_recognizers::source_xml::{
    SourceXmlAssertionInput, SourceXmlElementRole, SourceXmlFact, SourceXmlFactKind,
    SourceXmlFamily, SourceXmlInput, SourceXmlParentResolution, SourceXmlRecognition,
    SourceXmlReferenceOrder, SourceXmlScriptBinding, SourceXmlScriptSource, SourceXmlTemplateState,
    recognize_source_xml, recognize_source_xml_assertions,
};

fn recognizer_error(
    family: SourceXmlFamily,
    failure: wow_recognizers::RecognizerError,
) -> ServiceError {
    let code = match failure.code() {
        wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        wow_recognizers::RecognizerErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::InternalContractViolation,
    };
    ServiceError::new(
        code,
        format!(
            "{} recognition rejected ({:?})",
            family.rule_id(),
            failure.code()
        ),
    )
}

fn element_role(value: XmlElementRole) -> SourceXmlElementRole {
    match value {
        XmlElementRole::Ui => SourceXmlElementRole::Ui,
        XmlElementRole::Include => SourceXmlElementRole::Include,
        XmlElementRole::Script => SourceXmlElementRole::Script,
        XmlElementRole::Scripts => SourceXmlElementRole::Scripts,
        XmlElementRole::ScriptBinding => SourceXmlElementRole::ScriptBinding,
        XmlElementRole::Element => SourceXmlElementRole::Element,
        XmlElementRole::UnknownNamespace => SourceXmlElementRole::UnknownNamespace,
    }
}

/// Absent means the attribute was never spelled; it is never a default value.
fn template_state(value: Option<bool>) -> SourceXmlTemplateState {
    match value {
        None => SourceXmlTemplateState::Absent,
        Some(false) => SourceXmlTemplateState::False,
        Some(true) => SourceXmlTemplateState::True,
    }
}

fn script_source(value: XmlScriptSource) -> SourceXmlScriptSource {
    match value {
        XmlScriptSource::ExternalFile => SourceXmlScriptSource::ExternalFile,
        XmlScriptSource::ReferenceOnly => SourceXmlScriptSource::ReferenceOnly,
        XmlScriptSource::InlineBody => SourceXmlScriptSource::InlineBody,
        XmlScriptSource::Unresolved => SourceXmlScriptSource::Unresolved,
    }
}

fn reference_order(value: Option<XmlReferenceOrder>) -> Option<SourceXmlReferenceOrder> {
    value.map(|order| match order {
        XmlReferenceOrder::TargetBeforeSource => SourceXmlReferenceOrder::TargetBeforeSource,
        XmlReferenceOrder::TargetAfterSource => SourceXmlReferenceOrder::TargetAfterSource,
        XmlReferenceOrder::SelfReference => SourceXmlReferenceOrder::SelfReference,
        XmlReferenceOrder::RepeatedLoad => SourceXmlReferenceOrder::RepeatedLoad,
        XmlReferenceOrder::Unrecorded => SourceXmlReferenceOrder::Unrecorded,
    })
}

/// Direct, linear projection of the retained facts. Facts borrow from the owner
/// records; nothing is reparsed, reselected, deduplicated or reordered. Inheritance
/// ordinals are the loader ordinals for the explicit `inherits` list; declaration,
/// parent and script facts take the producer position in the fact_id order, which
/// is deterministic across runs and carries no source-order claim.
fn convert(facts: &[wow_project::graph::ProjectXmlFact]) -> Vec<SourceXmlFact<'_>> {
    facts
        .iter()
        .enumerate()
        .map(|(index, fact)| {
            let ordinal = match &fact.kind {
                wow_project::graph::ProjectXmlFactKind::Inheritance { ordinal, .. }
                | wow_project::graph::ProjectXmlFactKind::InheritanceUnresolved {
                    ordinal, ..
                } => *ordinal,
                _ => index as u64,
            };
            SourceXmlFact {
                fact_id: fact.fact_id.as_str(),
                context_id: fact.context_id,
                selected_toc: fact.scope.selected_toc.as_str(),
                flavor: fact.scope.flavor.as_str(),
                package: fact.scope.package.as_deref(),
                document: fact.document.as_str(),
                occurrence_id: fact.occurrence_id.as_str(),
                ordinal,
                content_digest: fact.content_digest,
                span: fact.span,
                source_handle_id: fact.source_handle_id,
                evidence_id: fact.evidence_id,
                kind: convert_kind(fact),
            }
        })
        .collect()
}

fn convert_kind(fact: &wow_project::graph::ProjectXmlFact) -> SourceXmlFactKind<'_> {
    match &fact.kind {
        wow_project::graph::ProjectXmlFactKind::Declaration { role, declaration } => {
            SourceXmlFactKind::Declaration {
                role: element_role(*role),
                element_name: fact.element_name.as_str(),
                name: declaration.name.as_deref(),
                virtual_template: template_state(declaration.virtual_template),
                intrinsic: template_state(declaration.intrinsic),
                mixin_names: &declaration.mixin_names,
                valid_declaration: declaration.valid_declaration,
                parent_occurrence_id: declaration.parent_occurrence_id.as_deref(),
            }
        }
        wow_project::graph::ProjectXmlFactKind::Parent {
            reference_id,
            name,
            resolution,
            order,
            cycle_id,
            ..
        } => SourceXmlFactKind::Parent {
            reference_id: reference_id.as_str(),
            name: name.as_str(),
            resolution: parent_resolution(resolution),
            order: reference_order(*order),
            cycle_id: cycle_id.as_deref(),
        },
        wow_project::graph::ProjectXmlFactKind::Inheritance {
            reference_id,
            target_occurrence_id,
            order,
            cycle_id,
            ..
        } => SourceXmlFactKind::Inheritance {
            reference_id: reference_id.as_str(),
            target_occurrence_id: target_occurrence_id.as_str(),
            order: reference_order(*order),
            cycle_id: cycle_id.as_deref(),
        },
        wow_project::graph::ProjectXmlFactKind::InheritanceUnresolved {
            reference_id,
            name,
            ..
        } => SourceXmlFactKind::InheritanceUnresolved {
            reference_id: reference_id.as_str(),
            name: name.as_str(),
        },
        wow_project::graph::ProjectXmlFactKind::Script {
            script_name,
            source_kind,
            owner_occurrence_id,
            inherit,
            intrinsic_order,
            file_reference,
            function_reference,
            method_reference,
            ..
        } => SourceXmlFactKind::Script {
            reference_id: fact.fact_id.as_str(),
            script_name: script_name.as_str(),
            source_kind: script_source(*source_kind),
            owner_occurrence_id: owner_occurrence_id.as_deref(),
            inherit: inherit.as_deref(),
            intrinsic_order: intrinsic_order.as_deref(),
            file_reference: file_reference.as_deref(),
            function_reference: function_reference.as_deref(),
            method_reference: method_reference.as_deref(),
        },
    }
}

/// Preserve the loader's verdict; an invalid target never becomes Unique.
fn parent_resolution(value: &XmlReferenceResolution) -> SourceXmlParentResolution<'_> {
    match value {
        XmlReferenceResolution::UniqueLocalDeclaration { declaration_id } => {
            SourceXmlParentResolution::Unique {
                target_occurrence_id: declaration_id,
            }
        }
        XmlReferenceResolution::AmbiguousName { name_group } => {
            SourceXmlParentResolution::Ambiguous {
                name_group: name_group.as_str(),
            }
        }
        XmlReferenceResolution::NotInCapturedScope => SourceXmlParentResolution::NotInCapturedScope,
        XmlReferenceResolution::DynamicName => SourceXmlParentResolution::DynamicName,
        XmlReferenceResolution::UnsupportedName => SourceXmlParentResolution::UnsupportedName,
        XmlReferenceResolution::InvalidSource => SourceXmlParentResolution::InvalidSource,
        XmlReferenceResolution::InvalidTarget { declaration_id } => {
            SourceXmlParentResolution::InvalidTarget {
                target_occurrence_id: declaration_id.as_str(),
            }
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub(super) struct XmlTopology {
    pub nodes: Vec<XmlNode>,
    pub edges: Vec<XmlEdge>,
    pub omissions: Vec<XmlOmission>,
}
#[derive(Debug, Serialize)]
pub(super) struct XmlNode {
    pub rule_id: String,
    pub fact_ids: Vec<String>,
    pub proposal_id: String,
    pub node_id: wow_graph::GraphNodeId,
}
#[derive(Debug, Serialize)]
pub(super) struct XmlEdge {
    pub rule_id: String,
    pub fact_ids: Vec<String>,
    pub proposal_id: String,
    pub edge_id: wow_graph::GraphEdgeId,
    pub from_node_id: wow_graph::GraphNodeId,
    pub to_node_id: wow_graph::GraphNodeId,
    pub relation: wow_graph::GraphRelationKind,
    pub confidence: wow_graph::GraphConfidence,
}
#[derive(Debug, Serialize)]
pub(super) struct XmlOmission {
    pub rule_id: String,
    pub fact_ids: Vec<String>,
    pub blocker: String,
}

fn bindings(provenance: &ProjectGraphProvenance) -> ServiceResult<Vec<SourceXmlScriptBinding<'_>>> {
    let mut sites = std::collections::BTreeMap::new();
    for site in provenance.script_sites() {
        if sites.insert(site.site_id.as_str(), site).is_some() {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
    }
    provenance
        .script_bindings()
        .iter()
        .map(|binding| {
            let site = sites
                .get(binding.site_id.as_str())
                .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
            if !site.binding_ids.contains(&binding.binding_id) || site.consumer_id.is_none() {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            }
            Ok(SourceXmlScriptBinding {
                binding_id: &binding.binding_id,
                site_id: &binding.site_id,
                script_id: &site.script_id,
                receiver_proposal_id: &binding.receiver_proposal_id,
                handler_proposal_id: &binding.handler_proposal_id,
                handler_kind: binding.handler_kind,
                consumer_occurrence_id: site.consumer_id.as_deref(),
                inherited: site.inherited,
                confidence: binding.confidence,
                semantic_context: binding.semantic_context.as_ref().map(|context| {
                    SourceScriptSemanticContext {
                        context_id: context.context_id(),
                        script_site: context.script_site(),
                        implicit_receiver: context.implicit_receiver(),
                        runtime_dispatch: context.runtime_dispatch(),
                    }
                }),
                source_handle_ids: &binding.source_handle_ids,
                evidence_ids: &binding.evidence_ids,
            })
        })
        .collect()
}

pub(super) fn publish(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, Vec<SourceXmlRecognition>)> {
    checkpoint(stop)?;
    let facts = convert(provenance.xml_facts());
    let script_bindings = bindings(provenance)?;
    let mut snapshot = source.clone();
    let mut recognitions = Vec::new();
    for family in SourceXmlFamily::ALL {
        checkpoint(stop)?;
        let proposals = recognize_source_xml(
            SourceXmlInput {
                owner: &snapshot,
                source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
                context: provenance.context(),
                facts: &facts,
                script_bindings: &script_bindings,
                source_handles: provenance.source_handles(),
                evidence: provenance.evidence(),
            },
            family,
            stop,
        )
        .map_err(|failure| recognizer_error(family, failure))?;
        checkpoint(stop)?;
        let prepared = snapshot
            .prepare_replacement(
                GraphPartitionReplacement {
                    expected_snapshot_id: snapshot.snapshot().snapshot_id().clone(),
                    expected_partition_digest: snapshot
                        .partition(family.partition_id())
                        .map(|p| p.partition_digest().into()),
                    producer_version: env!("CARGO_PKG_VERSION").into(),
                    batch: proposals.batch,
                    coverage: proposals.coverage,
                },
                stop,
            )
            .map_err(graph_error)?;
        snapshot = prepared.candidate().clone();
        recognitions.push(proposals.recognition);
    }
    Ok((snapshot, recognitions))
}

pub(super) fn publish_bound(
    source: &GraphPartitionSnapshot,
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, Vec<SourceXmlRecognition>)> {
    if crosswalk.direct_provenance().is_none() {
        return publish(source, crosswalk.source(), stop);
    }
    checkpoint(stop)?;
    let provenance = crosswalk.source();
    if provenance.xml_facts().len() > 32_768 || provenance.script_bindings().len() > 8192 {
        return Err(error(ServiceErrorCode::BudgetExceeded));
    }
    let facts = convert(provenance.xml_facts());
    let script_bindings = bindings(provenance)?;
    checkpoint(stop)?;
    let mut documents = BTreeSet::new();
    for fact in &facts {
        checkpoint(stop)?;
        documents.insert(fact.document);
    }
    let mut source_files = BTreeMap::new();
    for file in provenance.files() {
        checkpoint(stop)?;
        if documents.contains(file.path.as_str())
            && source_files
                .insert(
                    file.path.as_str(),
                    crosswalk.copied_assertion(
                        GraphAssertionKind::Entity,
                        &file.proposal_id,
                        stop,
                    )?,
                )
                .is_some()
        {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
    }
    if source_files.len() != documents.len() {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    let mut source_entities = BTreeMap::new();
    for binding in &script_bindings {
        checkpoint(stop)?;
        for proposal_id in [binding.receiver_proposal_id, binding.handler_proposal_id] {
            if let std::collections::btree_map::Entry::Vacant(entry) =
                source_entities.entry(proposal_id)
            {
                entry.insert(crosswalk.copied_assertion(
                    GraphAssertionKind::Entity,
                    proposal_id,
                    stop,
                )?);
            }
        }
    }
    let mut snapshot = source.clone();
    let mut recognitions = Vec::new();
    for family in SourceXmlFamily::ALL {
        checkpoint(stop)?;
        let proposals = recognize_source_xml_assertions(
            SourceXmlAssertionInput {
                owner: &snapshot,
                scope: crosswalk.scope(),
                context: provenance.context(),
                facts: &facts,
                script_bindings: &script_bindings,
                source_files: &source_files,
                source_entities: &source_entities,
                source_handles: provenance.source_handles(),
                evidence: provenance.evidence(),
            },
            family,
            stop,
        )
        .map_err(|failure| recognizer_error(family, failure))?;
        checkpoint(stop)?;
        crosswalk.reserve("xml_recognition", &proposals.recognition, stop)?;
        let prepared = snapshot
            .prepare_replacement(
                GraphPartitionReplacement {
                    expected_snapshot_id: snapshot.snapshot().snapshot_id().clone(),
                    expected_partition_digest: snapshot
                        .partition(family.partition_id())
                        .map(|partition| partition.partition_digest().into()),
                    producer_version: env!("CARGO_PKG_VERSION").into(),
                    batch: proposals.batch,
                    coverage: proposals.coverage,
                },
                stop,
            )
            .map_err(graph_error)?;
        snapshot = prepared.candidate().clone();
        recognitions.push(proposals.recognition);
        checkpoint(stop)?;
    }
    Ok((snapshot, recognitions))
}

pub(super) fn maps(
    snapshot: &GraphPartitionSnapshot,
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    recognitions: &[SourceXmlRecognition],
    stop: &AtomicBool,
) -> ServiceResult<XmlTopology> {
    let limits = snapshot.snapshot().limits();
    let nodes = super::materialized::nodes(snapshot, stop)?;
    let mut topology = XmlTopology::default();
    for recognition in recognitions {
        let rule_id = recognition.family.rule_id().to_owned();
        let partition_id = recognition.family.partition_id();
        for receipt in &recognition.receipts {
            checkpoint(stop)?;
            for proposal_id in &receipt.entity_proposal_ids {
                crosswalk.append(
                    "xml_topology.nodes",
                    &mut topology.nodes,
                    XmlNode {
                        rule_id: rule_id.clone(),
                        fact_ids: receipt.fact_ids.clone(),
                        proposal_id: proposal_id.clone(),
                        node_id: materialized_partition_node_id(
                            snapshot,
                            partition_id,
                            proposal_id,
                            limits,
                        )?,
                    },
                    stop,
                )?;
            }
            for proposal_id in &receipt.relation_proposal_ids {
                let edge_id =
                    super::materialized::edge_id(snapshot, partition_id, proposal_id, &nodes)?;
                let edge = snapshot
                    .snapshot()
                    .edge(&edge_id)
                    .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
                crosswalk.append(
                    "xml_topology.edges",
                    &mut topology.edges,
                    XmlEdge {
                        rule_id: rule_id.clone(),
                        fact_ids: receipt.fact_ids.clone(),
                        proposal_id: proposal_id.clone(),
                        edge_id,
                        from_node_id: edge.from().clone(),
                        to_node_id: edge.to().clone(),
                        relation: edge.relation(),
                        confidence: edge.confidence(),
                    },
                    stop,
                )?;
            }
        }
        for omission in &recognition.omissions {
            crosswalk.append(
                "xml_topology.omissions",
                &mut topology.omissions,
                XmlOmission {
                    rule_id: rule_id.clone(),
                    fact_ids: omission.fact_ids.clone(),
                    blocker: omission.blocker.to_owned(),
                },
                stop,
            )?;
        }
    }
    Ok(topology)
}

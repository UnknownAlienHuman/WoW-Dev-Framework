//! Derivation prerequisites retain their exact native producer addresses.
use super::*;
use std::collections::BTreeSet;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRecords, GraphAssertionRef,
    GraphDerivationRecord, GraphLocalAssertion,
};

pub(super) fn records(
    provenance: &ProjectGraphProvenance,
    scope: &GraphAssertionRecordScope,
    entities: &[GraphEntityProposal],
    relations: &[&RelationDraft],
    addresses: &BTreeMap<Box<str>, GraphAssertionRef>,
    budget: &mut producer_budget::ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<GraphAssertionRecords> {
    crate::analyzer::checkpoint(stop)?;
    let files = provenance
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.proposal_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut records = Vec::new();
    for entity in entities {
        crate::analyzer::checkpoint(stop)?;
        if entity.confidence() == GraphConfidence::Proven {
            continue;
        }
        let mut inputs = BTreeSet::new();
        for id in entity.source_handle_ids() {
            crate::analyzer::checkpoint(stop)?;
            let handle = provenance.source_handles.get(id).ok_or_else(invalid)?;
            let file = files.get(handle.path().as_str()).ok_or_else(invalid)?;
            inputs.insert(entity_address(file, addresses)?);
        }
        let record = GraphDerivationRecord {
            output: GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: entity.proposal_id().into(),
            },
            rule_id: format!(
                "wow-project.source-graph.entity.{}",
                entity.entity_kind_id()
            )
            .into(),
            rule_version: 1,
            inputs: inputs.into_iter().collect(),
            rebuttals: Vec::new(),
            missing: Vec::new(),
        };
        budget.charge_serialized(&record, stop)?;
        crate::analyzer::checkpoint(stop)?;
        records.push(record);
    }
    for relation in relations {
        crate::analyzer::checkpoint(stop)?;
        if relation.input.confidence == GraphConfidence::Proven {
            continue;
        }
        let mut inputs = BTreeSet::new();
        for endpoint in [&relation.input.source, &relation.input.target] {
            crate::analyzer::checkpoint(stop)?;
            let GraphProposalEndpoint::Proposed(id) = endpoint else {
                return Err(invalid());
            };
            inputs.insert(entity_address(id, addresses)?);
        }
        for id in &relation.input.source_handle_ids {
            crate::analyzer::checkpoint(stop)?;
            let handle = provenance.source_handles.get(id).ok_or_else(invalid)?;
            let file = files.get(handle.path().as_str()).ok_or_else(invalid)?;
            inputs.insert(entity_address(file, addresses)?);
        }
        let record = GraphDerivationRecord {
            output: GraphLocalAssertion {
                kind: GraphAssertionKind::Relation,
                proposal_id: relation.proposal_id.clone(),
            },
            rule_id: format!(
                "wow-project.source-graph.relation.{}",
                relation.relation_kind_id
            )
            .into(),
            rule_version: 1,
            inputs: inputs.into_iter().collect(),
            rebuttals: Vec::new(),
            missing: Vec::new(),
        };
        budget.charge_serialized(&record, stop)?;
        crate::analyzer::checkpoint(stop)?;
        records.push(record);
    }
    crate::analyzer::checkpoint(stop)?;
    let result = GraphAssertionRecords::build(scope.clone(), records, Vec::new())
        .map_err(platform_producers::graph_error)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(result)
}

fn entity_address(
    id: &str,
    addresses: &BTreeMap<Box<str>, GraphAssertionRef>,
) -> ProjectResult<GraphAssertionRef> {
    let reference = addresses.get(id).ok_or_else(invalid)?;
    let assertion = match reference {
        GraphAssertionRef::Local { assertion } | GraphAssertionRef::Producer { assertion, .. } => {
            assertion
        }
    };
    if assertion.kind != GraphAssertionKind::Entity || assertion.proposal_id.as_ref() != id {
        return Err(invalid());
    }
    Ok(reference.clone())
}

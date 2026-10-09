//! Source-projection derivations use exact captured file assertions and typed
//! proposal endpoints. The existing analyzer remains the only source interpreter.
use super::*;
use std::collections::BTreeSet;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRecords, GraphAssertionRef,
    GraphDerivationRecord, GraphLocalAssertion,
};

pub(super) fn records(
    provenance: &ProjectGraphProvenance,
    universe: &GraphUniverseId,
    generation: &GraphGenerationId,
    entities: &[GraphEntityProposal],
    relations: &[GraphRelationProposal],
    stop: &AtomicBool,
) -> ProjectResult<GraphAssertionRecords> {
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
            let handle = provenance.source_handles.get(id).ok_or_else(invalid)?;
            let file = files.get(handle.path().as_str()).ok_or_else(invalid)?;
            inputs.insert(local(file));
        }
        records.push(GraphDerivationRecord {
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
        });
    }
    for relation in relations {
        crate::analyzer::checkpoint(stop)?;
        if relation.confidence() == GraphConfidence::Proven {
            continue;
        }
        let mut inputs = BTreeSet::new();
        for endpoint in relation.endpoints() {
            let wow_graph::GraphProposalEndpoint::Proposed(id) = endpoint else {
                return Err(invalid());
            };
            inputs.insert(local(id));
        }
        for id in relation.source_handle_ids() {
            let handle = provenance.source_handles.get(id).ok_or_else(invalid)?;
            let file = files.get(handle.path().as_str()).ok_or_else(invalid)?;
            inputs.insert(local(file));
        }
        records.push(GraphDerivationRecord {
            output: GraphLocalAssertion {
                kind: GraphAssertionKind::Relation,
                proposal_id: relation.proposal_id().into(),
            },
            rule_id: format!(
                "wow-project.source-graph.relation.{}",
                relation.relation_kind_id()
            )
            .into(),
            rule_version: 1,
            inputs: inputs.into_iter().collect(),
            rebuttals: Vec::new(),
            missing: Vec::new(),
        });
    }
    GraphAssertionRecords::build(
        GraphAssertionRecordScope {
            universe: universe.clone(),
            generation: generation.clone(),
            source_context_id: provenance.context.context_id(),
        },
        records,
        Vec::new(),
    )
    .map_err(|_| invalid())
}
fn local(id: &str) -> GraphAssertionRef {
    GraphAssertionRef::Local {
        assertion: GraphLocalAssertion {
            kind: GraphAssertionKind::Entity,
            proposal_id: id.into(),
        },
    }
}

//! Requalify selected native API/use producers under the held replay pair.
//! Persisted partitions are comparison inputs; only native publication mints proof.
use super::native_api::{self, NativeApiProvenance};
use super::{
    SourceGraphAddressCrosswalk, SourceGraphDiagnostics, calls, checkpoint, graph_error,
    source_prefix,
};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
use wow_graph::{GraphEdge, GraphNode, GraphNodeId, GraphPartitionRead, GraphPartitionSnapshot};
use wow_project::{
    NativeApiUseRecipe, ProjectKind,
    graph::{NATIVE_API_SOURCE_GRAPH_PROFILE, source_graph_profile},
    replay::publication::AcquiredProjectPair,
};

/// Return no new authority on the unchanged, unselected route. On selection,
/// rebuild only source + native API + call producers, retaining the native proof.
pub(crate) fn validate(
    pair: &AcquiredProjectPair,
    stop: &AtomicBool,
) -> ServiceResult<Option<NativeApiProvenance>> {
    super::matcher_witness_reopen::validate(pair, stop)?;
    let project = pair.project();
    let configuration = project.configuration();
    let Some(recipe) = configuration.native_api_uses() else {
        return Ok(None);
    };
    checkpoint(stop)?;
    if recipe != NativeApiUseRecipe::ReferenceV1
        || configuration.native_library_input().is_none()
        || configuration.platform_graph_profile().is_some()
        || !matches!(
            configuration.project_kind(),
            ProjectKind::Fixture | ProjectKind::Repository
        )
        || source_graph_profile(configuration) != NATIVE_API_SOURCE_GRAPH_PROFILE
        || pair.source_provenance().is_none()
    {
        return Err(mismatch());
    }

    let source_prefix::SourcePrefix {
        owner: source_owner,
        projection,
    } = source_prefix::prepare(project, stop, &mut SourceGraphDiagnostics::default())?;
    let source_prefix::SourceProjection::Legacy(provenance) = projection else {
        return Err(mismatch());
    };
    same_stream(&source_owner, pair.graph(), stop)?;
    let addresses = SourceGraphAddressCrosswalk::legacy(&provenance, &source_owner, stop)?;
    // Reuse precisely the publication producers. No decoded receipt, report,
    // caller callback or reconstructed Library label can supply this proof.
    let (api_owner, api) = native_api::publish(project, &source_owner, stop)?;
    same_stream(&source_owner, &api_owner, stop)?;
    let (rebuilt, _recognition) = calls::publish(&api_owner, &addresses, Some(&api), stop)?;
    same_stream(&source_owner, &rebuilt, stop)?;
    same_stream(&rebuilt, pair.graph(), stop)?;

    // Compare complete producer records: batch, accepted/rejected proposals,
    // coverage, version and digest. Later unrelated producers remain retained;
    // this does not rebuild them or compare an intermediate snapshot ID to final.
    for partition in rebuilt.partitions() {
        checkpoint(stop)?;
        if pair.graph().partition(partition.partition_id()) != Some(partition) {
            return Err(mismatch());
        }
    }
    // The read is minted by the already-validated persisted Graph owner.
    // It borrows the exact held owner without another graph rebuild or clone.
    let read = pair.graph_read(stop).map_err(graph_error)?;
    validate_final_membership(&rebuilt, &read, stop)?;
    checkpoint(stop)?;
    Ok(Some(api))
}

fn same_stream(
    expected: &GraphPartitionSnapshot,
    actual: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ServiceResult<()> {
    checkpoint(stop)?;
    if actual.registry() != expected.registry()
        || actual.foundation() != expected.foundation()
        || actual.source_context_id() != expected.source_context_id()
    {
        return Err(mismatch());
    }
    checkpoint(stop)
}

fn validate_final_membership(
    rebuilt: &GraphPartitionSnapshot,
    read: &GraphPartitionRead<'_>,
    stop: &AtomicBool,
) -> ServiceResult<()> {
    let persisted = read.owner();
    let snapshot = persisted.snapshot();
    // Borrowed index only: no input_view/materialized graph copy. The genuine
    // graph owner bounds total accepted entities, including duplicate identities.
    let mut nodes = BTreeMap::<&GraphNodeId, &GraphNode>::new();
    for partition in rebuilt.partitions() {
        checkpoint(stop)?;
        for accepted in partition.report().accepted_entities() {
            checkpoint(stop)?;
            if partition
                .batch()
                .entity_proposal(accepted.proposal_id())
                .is_none()
            {
                return Err(mismatch());
            }
            let node = read
                .published_entity(partition.partition_id(), accepted.proposal_id(), stop)
                .map_err(graph_error)?;
            let input_id = accepted.node().node_id();
            if let Some(previous) = nodes.get(input_id) {
                if *previous != node {
                    return Err(mismatch());
                }
            } else {
                if nodes.len() >= snapshot.limits().max_nodes as usize {
                    return Err(ServiceError::new(
                        ServiceErrorCode::BudgetExceeded,
                        "native API reopen node index exceeded retained graph limits",
                    ));
                }
                nodes.insert(input_id, node);
            }
        }
    }
    for partition in rebuilt.partitions() {
        checkpoint(stop)?;
        for accepted in partition.report().accepted_relations() {
            checkpoint(stop)?;
            if partition
                .batch()
                .relation_proposal(accepted.proposal_id())
                .is_none()
            {
                return Err(mismatch());
            }
            let input = accepted.edge();
            let from = nodes.get(input.from()).ok_or_else(mismatch)?;
            let to = nodes.get(input.to()).ok_or_else(mismatch)?;
            let edge = GraphEdge::new(
                from.node_id().clone(),
                to.node_id().clone(),
                input.relation(),
                input.confidence(),
                input.evidence_ids().to_vec(),
                snapshot.limits(),
            )
            .map_err(graph_error)?;
            // Edge IDs bind final endpoints, relation, confidence and exact
            // evidence. Producer-input IDs alone cannot prove final membership.
            if snapshot.edge(edge.edge_id()) != Some(&edge) {
                return Err(mismatch());
            }
            checkpoint(stop)?;
        }
    }
    checkpoint(stop)
}

fn mismatch() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::IdentityMismatch,
        "retained native API/use producers differ from genuine reconstruction",
    )
}

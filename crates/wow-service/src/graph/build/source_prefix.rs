//! Prepare one source-only predecessor. Native direct authority is finished by
//! the caller while this graph remains held, before recognizers are admitted.
use super::*;
use wow_project::graph::{PlatformGraphProposalPlan, build_platform_graph_proposal_plan};

pub(super) enum SourceProjection<'project> {
    Legacy(Box<ProjectGraphProvenance>),
    Direct(Box<PlatformGraphProposalPlan<'project>>),
}

pub(super) struct SourcePrefix<'project> {
    pub owner: GraphPartitionSnapshot,
    pub projection: SourceProjection<'project>,
}

pub(super) fn prepare<'project>(
    project: &'project wow_project::ProjectView,
    stop: &AtomicBool,
) -> ServiceResult<SourcePrefix<'project>> {
    checkpoint(stop)?;
    if project.configuration().platform_graph_profile()
        == Some(wow_project::PlatformGraphProfile::DirectPlatformProducersWithRawInventoryV1)
    {
        let mut plan = build_platform_graph_proposal_plan(project, stop).map_err(project_error)?;
        let mut owner = GraphPartitionSnapshot::new(
            plan.registry().clone(),
            plan.foundation().clone(),
            plan.scope().source_context_id,
            stop,
        )
        .map_err(graph_error)?;
        let raw = plan
            .raw_inventory_batch()
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?
            .clone();
        checkpoint(stop)?;
        owner = admit(
            &owner,
            raw,
            Vec::new(),
            plan.raw_inventory_producer_version(),
            stop,
        )?;
        for &producer in plan.producer_order() {
            let stage = plan
                .build_stage(producer, &owner, stop)
                .map_err(project_error)?;
            let version = stage.producer_version();
            let (batch, coverage) = stage.into_parts();
            owner = admit(&owner, batch, coverage, version, stop)?;
        }
        Ok(SourcePrefix {
            owner,
            projection: SourceProjection::Direct(Box::new(plan)),
        })
    } else {
        let proposals = build_source_graph_proposals(project, stop).map_err(project_error)?;
        let inventory_batch = proposals.inventory_batch().cloned();
        if inventory_batch.is_some()
            != (project.configuration().platform_graph_profile()
                == Some(wow_project::PlatformGraphProfile::PackageProjectionWithRawInventoryV1))
        {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        let (registry, batch, coverage, provenance, limits) = proposals.into_parts();
        checkpoint(stop)?;
        let foundation = GraphSnapshot::build(
            batch.universe().clone(),
            batch.generation().clone(),
            limits,
            Vec::new(),
            Vec::new(),
            coverage.clone(),
        )
        .map_err(graph_error)?;
        let mut owner =
            GraphPartitionSnapshot::new(registry, foundation, batch.source_context_id(), stop)
                .map_err(graph_error)?;
        if let Some(batch) = inventory_batch {
            owner = admit(&owner, batch, Vec::new(), env!("CARGO_PKG_VERSION"), stop)?;
        }
        owner = admit(&owner, batch, coverage, env!("CARGO_PKG_VERSION"), stop)?;
        Ok(SourcePrefix {
            owner,
            projection: SourceProjection::Legacy(Box::new(provenance)),
        })
    }
}

fn admit(
    owner: &GraphPartitionSnapshot,
    batch: wow_graph::GraphProposalBatch,
    coverage: Vec<wow_graph::GraphCoverageRecord>,
    version: &str,
    stop: &AtomicBool,
) -> ServiceResult<GraphPartitionSnapshot> {
    checkpoint(stop)?;
    let prepared = owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: version.into(),
                batch,
                coverage,
            },
            stop,
        )
        .map_err(graph_error)?;
    let owner = prepared.candidate().clone();
    checkpoint(stop)?;
    Ok(owner)
}

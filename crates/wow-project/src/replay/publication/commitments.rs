//! The identical pair identity recipe for publication and checked retained reads.
//! Read-side graph commitments come from the qualified graph owner, not from
//! trusting manifest versions after semantic decoding.
use std::{collections::BTreeMap, sync::atomic::AtomicBool};

use wow_graph::{GraphPartitionSnapshot, ValidatedGraphPartitionSnapshot};
use wow_store::project::{PartitionRecord, PartitionVersionId, ReadSnapshot};

use crate::{ProjectPhase, ProjectResult, ProjectView};

use super::{
    HEADER_KEY, HEADER_SCHEMA, PairHeader, ProjectReplay, REPLAY_KEY, graph_error, invalid,
};

pub(super) fn validate_read(
    read: &ReadSnapshot,
    replay: &ProjectReplay,
    project: &ProjectView,
    graph: &ValidatedGraphPartitionSnapshot,
    header: &PairHeader,
    companion_records: Vec<PartitionRecord>,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    let graph_members = graph
        .validate_storage_membership(read, stop)
        .map_err(graph_error)?;
    crate::analyzer::checkpoint(stop)?;
    let replay_record = PartitionRecord::new(REPLAY_KEY, replay.storage_schema()?, replay)
        .map_err(|_| super::super::exhausted())?;
    let owner = graph.as_snapshot();
    let (expected_header, mut bindings) = {
        let mut membership = graph_members
            .iter()
            .map(|member| (member.key.as_str(), &member.version))
            .collect::<Vec<_>>();
        membership.push((replay_record.key(), replay_record.version()));
        membership.sort_by(|left, right| left.0.cmp(right.0));
        metadata(project, owner, &membership)?
    };
    // Dispatch only from the actual epoch held by this lease. No legacy fallback
    // may bypass failed namespace admission.
    let namespace = read.epoch().namespace();
    if let Some(namespace) = namespace {
        super::namespace::bind(&mut bindings, namespace, project, owner)?;
    }
    let expected_owner = namespace.map_or_else(
        || owner.snapshot().universe().as_str(),
        |namespace| namespace.id().as_str(),
    );
    if header != &expected_header
        || bindings != read.manifest().bindings
        || read.manifest().owner != expected_owner
        || read.epoch().owner() != expected_owner
        || graph_members
            .len()
            .checked_add(2)
            .and_then(|count| count.checked_add(companion_records.len()))
            .is_none_or(|count| count != read.manifest().members.len())
    {
        return Err(invalid());
    }
    let header_record =
        PartitionRecord::new(HEADER_KEY, "wow-project.live-pair.v1", &expected_header)
            .map_err(|_| invalid())?;
    let mut records = vec![replay_record, header_record];
    for record in companion_records {
        crate::analyzer::checkpoint(stop)?;
        records.push(record);
    }
    let mut membership = graph_members
        .iter()
        .map(|member| (member.key.as_str(), &member.version))
        .chain(
            records
                .iter()
                .map(|record| (record.key(), record.version())),
        )
        .collect::<Vec<_>>();
    membership.sort_by(|left, right| left.0.cmp(right.0));
    // A duplicate companion key cannot replace or stand in for any real owner.
    if membership.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(invalid());
    }
    for ((key, version), member) in membership.iter().zip(&read.manifest().members) {
        crate::analyzer::checkpoint(stop)?;
        if key != &member.key || *version != &member.version {
            return Err(invalid());
        }
    }
    crate::analyzer::checkpoint(stop)
}

pub(super) fn metadata(
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    membership: &[(&str, &PartitionVersionId)],
) -> ProjectResult<(PairHeader, BTreeMap<String, String>)> {
    // Only sorted graph/replay membership enters this existing recipe. Pair
    // header and companions remain outside it, exactly as on publication.
    let publication_set_id = crate::identity::canonical_id(
        "project-publication-set:sha256:",
        "wow-project/live-project-publication-set/1",
        &(
            project.snapshot_id(),
            project.analyzer_snapshot_id(),
            graph.snapshot().snapshot_id(),
            membership,
        ),
        ProjectPhase::Publication,
    )?
    .into_string();
    let header = PairHeader {
        schema: HEADER_SCHEMA.into(),
        project_snapshot_id: project.snapshot_id().into(),
        analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
        graph_snapshot_id: graph.snapshot().snapshot_id().as_str().into(),
        publication_set_id: publication_set_id.clone(),
    };
    let bindings = BTreeMap::from([
        ("scope".into(), "native-live-project-pair-v1".into()),
        ("project_publication_set_id".into(), publication_set_id),
        ("project_snapshot_id".into(), project.snapshot_id().into()),
        (
            "analyzer_snapshot_id".into(),
            project.analyzer_snapshot_id().into(),
        ),
        (
            "graph_snapshot_id".into(),
            graph.snapshot().snapshot_id().as_str().into(),
        ),
        (
            "project_generation_id".into(),
            project.project_generation().to_string(),
        ),
        (
            "source_context_id".into(),
            graph.source_context_id().to_string(),
        ),
        (
            "profile_id".into(),
            project
                .configuration()
                .selected_profile()
                .profile_id()
                .to_string(),
        ),
        (
            "reference_generation_id".into(),
            project.configuration().reference_generation().to_string(),
        ),
    ]);
    Ok((header, bindings))
}

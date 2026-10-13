//! Re-admission of decoded owner records, without native read custody.

use std::collections::BTreeSet;

use wow_core::CoverageStatus;
use wow_graph::GraphCoverageRecord;
use wow_reference::ReferencePartition;

use super::project_map_wire::{
    CoverageObservationWire, PROJECT_MAP_PROJECTION_LIMITS, ProjectMapWire,
    ProjectMapWireCheckError,
};

type CheckResult<E> = Result<(), ProjectMapWireCheckError<E>>;

pub(super) fn check<E>(
    map: &ProjectMapWire,
    reserve: &mut impl FnMut(usize) -> Result<(), E>,
) -> CheckResult<E> {
    let binding = &map.core.binding;
    reserve(1).map_err(ProjectMapWireCheckError::Work)?;
    let generation = binding.project_generation.canonical();
    let fixed = [
        (
            "project.fixture.configuration.valid",
            "project.configuration",
            None,
        ),
        (
            "project.fixture.files.complete",
            "project.workspace",
            Some(map.core.project_container.workspace_id.as_str()),
        ),
        (
            "project.source.registry.complete",
            "project.source_origin",
            Some(map.core.project_container.origin_id.as_str()),
        ),
        (
            "project.source.handle.resolve",
            "project.source_registry",
            Some(binding.source_registry_id.as_str()),
        ),
        (
            "project.generation.coherent",
            "project.generation",
            Some(generation.as_str()),
        ),
        (
            "project.analyzer.snapshot.available",
            "project.analyzer",
            Some(binding.analyzer_snapshot_id.as_str()),
        ),
    ];
    let mut fixed_seen = [false; 6];
    let mut previous = None;
    let mut files = BTreeSet::new();
    let mut diagnostics = BTreeSet::new();
    for record in &map.core.project_coverage {
        reserve(1).map_err(ProjectMapWireCheckError::Work)?;
        let partition = record.partition_id();
        let key_bytes = partition.key().map_or(0, str::len);
        let nested = record
            .missing_input_ids()
            .len()
            .checked_add(record.conflict_ids().len())
            .and_then(|value| value.checked_add(record.truncation_refs().len()))
            .and_then(|value| value.checked_mul(256))
            .and_then(|value| value.checked_add(key_bytes.saturating_mul(3)))
            .ok_or(ProjectMapWireCheckError::Invalid("coverage work overflow"))?;
        reserve(nested).map_err(ProjectMapWireCheckError::Work)?;
        for id in record.missing_input_ids() {
            reserve(id.len()).map_err(ProjectMapWireCheckError::Work)?;
        }
        for truncation in record.truncation_refs() {
            reserve(truncation.collection_id().len()).map_err(ProjectMapWireCheckError::Work)?;
        }
        let canonical_partition = partition.canonical();
        let order = (record.capability_id(), canonical_partition);
        if record.validate().is_err()
            || record.context_id() != binding.source_context_id
            || previous.as_ref().is_some_and(|previous| previous >= &order)
        {
            return Err(ProjectMapWireCheckError::Invalid(
                "project coverage owner admission, binding or order mismatch",
            ));
        }
        previous = Some(order);
        if let Some(index) = fixed
            .iter()
            .position(|entry| entry.0 == record.capability_id().as_str())
        {
            let (_, scope, key) = fixed[index];
            if fixed_seen[index]
                || partition.scope() != scope
                || partition.key() != key
                || record.status() != CoverageStatus::Complete
            {
                return Err(ProjectMapWireCheckError::Invalid(
                    "project coverage exact partition mismatch",
                ));
            }
            fixed_seen[index] = true;
        } else {
            match (
                record.capability_id().as_str(),
                partition.scope(),
                partition.key(),
            ) {
                ("project.analyzer.facts.available", "project.file", Some(file)) => {
                    files.insert(file);
                }
                (
                    "project.analyzer.generic_diagnostics.available",
                    "project.file.diagnostics",
                    Some(file),
                ) if record.status() == CoverageStatus::Complete => {
                    diagnostics.insert(file);
                }
                ("project.analyzer.facts.available", "project.xml_lua_unit", Some(_))
                    if record.status() == CoverageStatus::Complete => {}
                _ => {
                    return Err(ProjectMapWireCheckError::Invalid(
                        "project coverage is outside native owner partitions",
                    ));
                }
            }
        }
    }
    if fixed_seen.contains(&false) || files != diagnostics {
        return Err(ProjectMapWireCheckError::Invalid(
            "project coverage required population mismatch",
        ));
    }
    let mut previous = None;
    for deferred in &map.core.deferred_project_capabilities {
        let record = &deferred.record;
        reserve(
            record
                .subject_kind()
                .len()
                .saturating_add(record.subject_id().len())
                .saturating_add(1),
        )
        .map_err(ProjectMapWireCheckError::Work)?;
        let nested = record
            .blocking_capability_ids()
            .len()
            .checked_add(record.blocking_partitions().len())
            .and_then(|value| value.checked_add(record.conflict_ids().len()))
            .and_then(|value| value.checked_mul(256))
            .ok_or(ProjectMapWireCheckError::Invalid(
                "deferred capability work overflow",
            ))?;
        reserve(nested).map_err(ProjectMapWireCheckError::Work)?;
        if record.validate().is_err()
            || record.context_id() != binding.source_context_id
            || record.subject_kind() != "capability"
            || record.subject_id() != deferred.capability_id.as_str()
            || record.blocking_capability_ids() != std::slice::from_ref(&deferred.capability_id)
            || !record.blocking_partitions().is_empty()
            || !record.conflict_ids().is_empty()
            || previous.is_some_and(|previous| previous >= &deferred.capability_id)
        {
            return Err(ProjectMapWireCheckError::Invalid(
                "deferred capability owner admission, binding or order mismatch",
            ));
        }
        previous = Some(&deferred.capability_id);
    }
    let mut previous = None;
    for coverage in &map.core.reference_coverage {
        reserve(coverage.partition_id.len().saturating_add(1))
            .map_err(ProjectMapWireCheckError::Work)?;
        if ReferencePartition::validate_id(&coverage.partition_id).is_err()
            || previous.is_some_and(|previous| previous >= coverage.partition_id.as_str())
        {
            return Err(ProjectMapWireCheckError::Invalid(
                "Reference coverage partition admission or order mismatch",
            ));
        }
        previous = Some(coverage.partition_id.as_str());
    }
    let mut previous = None;
    for conflict in &map.core.reference_conflicts {
        reserve(
            conflict
                .partition_id()
                .len()
                .saturating_add(conflict.key().len())
                .saturating_add(1),
        )
        .map_err(ProjectMapWireCheckError::Work)?;
        for value in conflict
            .candidate_digests()
            .iter()
            .chain(conflict.source_ids())
        {
            reserve(value.len().saturating_add(1)).map_err(ProjectMapWireCheckError::Work)?;
        }
        let key = (conflict.partition_id(), conflict.key());
        if conflict.validate().is_err() || previous.is_some_and(|previous| previous >= key) {
            return Err(ProjectMapWireCheckError::Invalid(
                "Reference conflict owner admission or order mismatch",
            ));
        }
        previous = Some(key);
        let mut partition_present = false;
        for coverage in &map.core.reference_coverage {
            reserve(1).map_err(ProjectMapWireCheckError::Work)?;
            partition_present |= coverage.partition_id == conflict.partition_id();
        }
        if !partition_present {
            return Err(ProjectMapWireCheckError::Invalid(
                "Reference conflict partition is absent",
            ));
        }
    }
    for receipt in &map.core.supports {
        reserve(1).map_err(ProjectMapWireCheckError::Work)?;
        for coverage in &receipt.explanation.coverage {
            reserve(1).map_err(ProjectMapWireCheckError::Work)?;
            if let CoverageObservationWire::Retained(record) = &coverage.observation {
                check_graph_coverage(record, reserve)?;
            }
        }
    }
    Ok(())
}

pub(super) fn check_graph_coverage<E>(
    record: &GraphCoverageRecord,
    reserve: &mut impl FnMut(usize) -> Result<(), E>,
) -> CheckResult<E> {
    let sorting = record.blocker_ids().len().max(1).ilog2() as usize + 1;
    reserve(1).map_err(ProjectMapWireCheckError::Work)?;
    for blocker in record.blocker_ids() {
        let work = blocker
            .len()
            .checked_add(1)
            .and_then(|value| value.checked_mul(sorting))
            .ok_or(ProjectMapWireCheckError::Invalid(
                "graph coverage work overflow",
            ))?;
        reserve(work).map_err(ProjectMapWireCheckError::Work)?;
    }
    record
        .validate(PROJECT_MAP_PROJECTION_LIMITS)
        .map_err(|_| ProjectMapWireCheckError::Invalid("graph coverage owner admission failed"))
}

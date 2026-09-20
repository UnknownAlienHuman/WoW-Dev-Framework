//! Shared admission for the two correctness-affecting coverage consumers.
//! A deserialized summary is a claim, not a substitute for its owner records.
use std::collections::{BTreeMap, BTreeSet};

use super::{
    BlockingPartitionRef, CapabilitySummary, ConflictRecord, CoreErrorCode, CoreResult,
    CoverageRecord, GenerationContextId, NotEvaluatedRecord, ensure_sorted_unique,
    validation_error,
};

pub(super) fn validate_inputs(
    context_id: GenerationContextId,
    summaries: &[CapabilitySummary],
    records: &[CoverageRecord],
    conflicts: &[ConflictRecord],
) -> CoreResult<()> {
    const OPERATION: &str = "validate_capability_summary";
    if summaries.is_empty() || records.is_empty() {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::CoverageRecordMissing,
            "required_coverage",
        ));
    }

    let mut conflict_index = BTreeMap::new();
    let mut affected = BTreeMap::<_, BTreeSet<_>>::new();
    for conflict in conflicts {
        conflict.validate()?;
        if conflict.context_id() != context_id {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ConflictContextMismatch,
                "conflicts.context_id",
            ));
        }
        if conflict_index
            .insert(conflict.conflict_id(), conflict)
            .is_some()
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::DuplicateConflictReference,
                "conflicts",
            ));
        }
        ensure_sorted_unique(
            conflict.affected_refs(),
            OPERATION,
            "conflicts.affected_refs",
            CoreErrorCode::ConflictScopeEmpty,
        )?;
        for scope in conflict.affected_refs() {
            affected
                .entry((scope.capability_id(), scope.partition_id()))
                .or_default()
                .insert(conflict.conflict_id());
        }
    }

    let mut by_capability = BTreeMap::<_, Vec<CoverageRecord>>::new();
    let mut record_keys = BTreeSet::new();
    for record in records {
        record.validate()?;
        if record.context_id != context_id {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::CoverageContextMismatch,
                "coverage_records.context_id",
            ));
        }
        if !record_keys.insert((
            &record.capability_id,
            &record.partition_id,
            &record.producer_id,
        )) {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::DuplicateCoverageRecord,
                "coverage_records",
            ));
        }
        for conflict_id in &record.conflict_ids {
            if !conflict_index.contains_key(conflict_id) {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::MissingConflictReference,
                    "coverage_records.conflict_ids",
                ));
            }
        }
        let mut expected_conflicts = BTreeSet::new();
        for key in [
            (&record.capability_id, None),
            (&record.capability_id, Some(&record.partition_id)),
        ] {
            if let Some(ids) = affected.get(&key) {
                expected_conflicts.extend(ids.iter().copied());
            }
        }
        if expected_conflicts
            .iter()
            .copied()
            .ne(record.conflict_ids.iter().copied())
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::CoverageConflict,
                "coverage_records.conflict_ids",
            ));
        }
        by_capability
            .entry(&record.capability_id)
            .or_default()
            .push(record.clone());
    }

    let mut summary_keys = BTreeSet::new();
    for summary in summaries {
        if summary.context_id != context_id {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::CoverageContextMismatch,
                "summaries.context_id",
            ));
        }
        if !summary_keys.insert((&summary.capability_id, &summary.producer_id)) {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::DuplicateCoverageRecord,
                "summaries",
            ));
        }
        let selected = by_capability.get(&summary.capability_id).ok_or_else(|| {
            validation_error(
                OPERATION,
                CoreErrorCode::CoverageRecordMissing,
                "coverage_records",
            )
        })?;
        // Recompute from ALL supplied records for this required capability, not
        // only IDs the summary chose to advertise. Otherwise it can hide a worse
        // partition by dropping its ref as well as changing its status.
        let selected_ids = selected
            .iter()
            .map(CoverageRecord::coverage_id)
            .collect::<BTreeSet<_>>();
        if summary
            .partition_refs
            .iter()
            .any(|reference| !selected_ids.contains(&reference.coverage_id))
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::CoverageRecordMissing,
                "summaries.partition_refs",
            ));
        }
        summary.validate(selected)?;
    }
    Ok(())
}

pub(super) fn validate_evaluation(
    context_id: GenerationContextId,
    evaluation: &NotEvaluatedRecord,
    records: &[CoverageRecord],
    conflicts: &[ConflictRecord],
) -> CoreResult<()> {
    const OPERATION: &str = "evaluate_negative_authority";
    evaluation.validate()?;
    if evaluation.context_id != context_id {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::ResultContextViolation,
            "evaluation.context_id",
        ));
    }
    let index = records
        .iter()
        .map(|record| (record.coverage_id, record))
        .collect::<BTreeMap<_, _>>();
    for blocker in &evaluation.blocking_partitions {
        let record = index.get(&blocker.coverage_id).ok_or_else(|| {
            validation_error(
                OPERATION,
                CoreErrorCode::CoverageRecordMissing,
                "evaluation.blocking_partitions",
            )
        })?;
        if BlockingPartitionRef::from_record(record) != *blocker
            || !evaluation
                .blocking_capability_ids
                .contains(&blocker.capability_id)
            || blocker
                .conflict_ids
                .iter()
                .any(|id| !evaluation.conflict_ids.contains(id))
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::CoverageConflict,
                "evaluation.blocking_partitions",
            ));
        }
    }
    for conflict_id in &evaluation.conflict_ids {
        let conflict = conflicts
            .iter()
            .find(|conflict| conflict.conflict_id() == *conflict_id)
            .ok_or_else(|| {
                validation_error(
                    OPERATION,
                    CoreErrorCode::MissingConflictReference,
                    "evaluation.conflict_ids",
                )
            })?;
        if !conflict.affected_refs().iter().any(|scope| {
            evaluation
                .blocking_capability_ids
                .contains(scope.capability_id())
        }) {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::CoverageConflict,
                "evaluation.conflict_ids",
            ));
        }
    }
    Ok(())
}

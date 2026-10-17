//! Exact-key observations over two genuine native callable projections.
//! Association stays Candidate. Selected field equality is neither continuity nor
//! whole-contract equality; source coordinates are evidence, not field values.
use std::{collections::BTreeSet, sync::atomic::AtomicBool};

use serde::{Deserialize, Serialize};

use super::{
    NATIVE_API_PARTITION, NATIVE_VIEW_AUTHORITY_PROFILE, NATIVE_VIEW_PROFILE, NativeCallableField,
    NativeCallableSignature, NativeViewProjection, NativeViewSource, callable_origin_id,
    checkpoint, error,
};
use crate::{
    LookupResult, LookupUnknownReason, ReferenceConflict, ReferenceRecord, ReferenceViewError,
    native::{NativeError, NativeErrorCode, RawKind, RawValue, Span},
};

pub const CALLABLE_COMPARISON_PROFILE: &str = "wow-reference/native-callable-comparison/1";

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CallableComparisonLimits {
    pub max_keys: usize,
    pub max_view_bytes: usize,
    pub max_steps: usize,
}
impl CallableComparisonLimits {
    pub fn validate(self) -> Result<(), NativeError> {
        if self.max_keys == 0
            || self.max_keys > 4096
            || self.max_view_bytes == 0
            || self.max_view_bytes > 64 * 1024 * 1024
            || self.max_steps == 0
            || self.max_steps > 32 * 1024 * 1024
        {
            return Err(error(NativeErrorCode::Limit));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct NativeCallableComparison {
    #[serde(skip_serializing_if = "String::is_empty")]
    comparison_digest: String,
    profile: &'static str,
    before_generation: String,
    before_view_digest: String,
    after_generation: String,
    after_view_digest: String,
    association_ceiling: &'static str,
    accepted_lineage_count: usize,
    lineage_negative_authority: bool,
    scope: &'static str,
    not_evaluated: &'static [&'static str],
    observations: Vec<CallableObservation>,
}
impl NativeCallableComparison {
    pub fn is_partial(&self) -> bool {
        self.observations
            .iter()
            .any(|row| matches!(row.comparison, FieldComparison::NotEvaluated))
    }
}

#[derive(Debug, Serialize)]
struct CallableObservation {
    key: String,
    before: CallableSide,
    after: CallableSide,
    comparison: FieldComparison,
}

#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum CallableSide {
    Found {
        record_digest: String,
        source: NativeViewSource,
        arguments: Vec<FieldObservation>,
        returns: Vec<FieldObservation>,
    },
    Conflict {
        conflict: ReferenceConflict,
        sources: Vec<NativeViewSource>,
    },
    AuthoritativeExactKeyAbsence {},
    Unknown {
        reason: &'static str,
    },
}

#[derive(Debug, Serialize)]
struct FieldObservation {
    name: String,
    type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    inner_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nilable: Option<bool>,
    default_stated: bool,
    stride_index_stated: bool,
    source_span: Span,
}

#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum FieldComparison {
    SelectedFieldsEqual {},
    SelectedFieldsDiffer { differences: Vec<FieldDifference> },
    NotEvaluated,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum FieldLane {
    Argument,
    Return,
}

#[derive(Debug, Serialize)]
struct FieldDifference {
    lane: FieldLane,
    /// One-based original source position, not a name-based alignment.
    ordinal: usize,
    changed: Vec<FieldFacet>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum FieldFacet {
    Added,
    NotPresentAfter,
    Name,
    TypeName,
    InnerType,
    Nilable,
    Default,
    StrideIndex,
}

struct Budget<'a> {
    steps: usize,
    stop: &'a AtomicBool,
}
impl Budget<'_> {
    fn charge(&mut self, amount: usize) -> Result<(), NativeError> {
        checkpoint(self.stop)?;
        self.steps = self
            .steps
            .checked_sub(amount)
            .ok_or_else(|| error(NativeErrorCode::Limit))?;
        Ok(())
    }
}

/// Compare the supported ordered fields for explicitly selected exact keys.
/// Revisions are exact source-origin guards, never current selectors. This port
/// performs no IO and cannot manufacture native signature catalog membership.
pub fn compare_reference_generations(
    before: &NativeViewProjection,
    before_revision: &str,
    after: &NativeViewProjection,
    after_revision: &str,
    keys: &[String],
    limits: CallableComparisonLimits,
    stop: &AtomicBool,
) -> Result<NativeCallableComparison, NativeError> {
    checkpoint(stop)?;
    limits.validate()?;
    if keys.is_empty() || keys.len() > limits.max_keys {
        return Err(error(NativeErrorCode::Limit));
    }
    let mut budget = Budget {
        steps: limits.max_steps,
        stop,
    };
    let mut selected = BTreeSet::new();
    for key in keys {
        budget.charge(1)?;
        ReferenceRecord::validate_key(key).map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
        if !key.starts_with("function:") || !selected.insert(key.as_str()) {
            return Err(error(NativeErrorCode::InvalidIdentity));
        }
    }
    if before.schema != after.schema {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    for projection in [before, after] {
        if !matches!(
            projection.schema,
            NATIVE_VIEW_PROFILE | NATIVE_VIEW_AUTHORITY_PROFILE
        ) || projection.native_documents().is_some()
            || projection.type_resolution_profile().is_some()
        {
            return Err(error(NativeErrorCode::InvalidIdentity));
        }
        projection
            .signatures
            .require_view(projection.schema, &projection.view, stop)?;
        let (_, steps) = projection
            .view
            .validate_bounded(limits.max_view_bytes, budget.steps, stop)
            .map_err(view_error)?;
        budget.charge(steps)?;
    }
    let mut observations = Vec::with_capacity(selected.len());
    for key in selected {
        budget.charge(1)?;
        let (before_side, before_signature) = observe(before, before_revision, key, &mut budget)?;
        let (after_side, after_signature) = observe(after, after_revision, key, &mut budget)?;
        let comparison = match (before_signature, after_signature) {
            (Some(left), Some(right)) => compare_fields(left, right, &mut budget)?,
            _ => FieldComparison::NotEvaluated,
        };
        observations.push(CallableObservation {
            key: key.to_owned(),
            before: before_side,
            after: after_side,
            comparison,
        });
    }
    checkpoint(stop)?;
    let mut result = NativeCallableComparison {
        comparison_digest: String::new(),
        profile: CALLABLE_COMPARISON_PROFILE,
        before_generation: before.view.generation_id().to_owned(),
        before_view_digest: before.view.self_digest().to_owned(),
        after_generation: after.view.generation_id().to_owned(),
        after_view_digest: after.view.self_digest().to_owned(),
        association_ceiling: "candidate",
        accepted_lineage_count: 0,
        lineage_negative_authority: false,
        scope: "original_ordered_callable_fields_only",
        not_evaluated: &[
            "stable_identity",
            "rename_move",
            "deprecation_replacement",
            "introduced_removed",
            "restrictions_runtime",
            "correction_type_resolution",
            "documentation_unknown_raw_fields",
            "graph_lineage_review_migration_impact",
        ],
        observations,
    };
    let (digest, _, steps) = crate::deterministic_view::bounded_digest(
        &result,
        limits.max_view_bytes,
        budget.steps,
        stop,
    )
    .map_err(view_error)?;
    budget.charge(steps)?;
    result.comparison_digest = digest;
    Ok(result)
}

fn view_error(error_value: ReferenceViewError) -> NativeError {
    error(match error_value {
        ReferenceViewError::Cancelled => NativeErrorCode::Cancelled,
        ReferenceViewError::LimitExceeded => NativeErrorCode::Limit,
        _ => NativeErrorCode::InvalidIdentity,
    })
}

fn observe<'a>(
    projection: &'a NativeViewProjection,
    revision: &str,
    key: &str,
    budget: &mut Budget<'_>,
) -> Result<(CallableSide, Option<NativeCallableSignature<'a>>), NativeError> {
    // Charge a conservative upper bound for the existing owner's linear lookup.
    for partition in projection.view.partitions() {
        budget.charge(partition.records().len().saturating_add(1))?;
    }
    budget.charge(projection.view.conflicts().len())?;
    match projection.view.lookup(NATIVE_API_PARTITION, key) {
        LookupResult::Found(record) => {
            let signature = projection.callable_signature(record, budget.stop)?;
            if signature.reference_generation() != projection.view.generation_id()
                || signature.reference_view_digest() != projection.view.self_digest()
            {
                return Err(error(NativeErrorCode::InvalidIdentity));
            }
            let source = original_source(projection, revision, signature.source_id(), budget)?;
            let side = CallableSide::Found {
                record_digest: signature.record_digest().to_owned(),
                source,
                arguments: field_observations(signature.arguments(), budget)?,
                returns: field_observations(signature.returns(), budget)?,
            };
            Ok((side, Some(signature)))
        }
        LookupResult::Conflict(conflict) => {
            let sources = conflict
                .source_ids()
                .iter()
                .map(|id| original_source(projection, revision, id, budget))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                CallableSide::Conflict {
                    conflict: conflict.clone(),
                    sources,
                },
                None,
            ))
        }
        LookupResult::AuthoritativeAbsence => {
            Ok((CallableSide::AuthoritativeExactKeyAbsence {}, None))
        }
        LookupResult::Unknown(reason) => Ok((
            CallableSide::Unknown {
                reason: match reason {
                    LookupUnknownReason::PartitionMissing => "partition_missing",
                    LookupUnknownReason::PartialCoverage => "partial_coverage",
                    LookupUnknownReason::NotEvaluated => "not_evaluated",
                },
            },
            None,
        )),
    }
}

fn original_source(
    projection: &NativeViewProjection,
    revision: &str,
    id: &str,
    budget: &mut Budget<'_>,
) -> Result<NativeViewSource, NativeError> {
    let mut source = None;
    for candidate in &projection.sources {
        budget.charge(1)?;
        if candidate.id == id {
            if callable_origin_id(
                projection.schema,
                revision,
                &candidate.path,
                &candidate.sha256,
                candidate.span,
            )? != id
                || source.is_some_and(|prior| prior != candidate)
            {
                return Err(error(NativeErrorCode::InvalidIdentity));
            }
            source = Some(candidate);
        }
    }
    source
        .cloned()
        .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))
}

fn field_observations(
    fields: &[NativeCallableField],
    budget: &mut Budget<'_>,
) -> Result<Vec<FieldObservation>, NativeError> {
    fields
        .iter()
        .map(|field| {
            budget.charge(1)?;
            Ok(FieldObservation {
                name: field.name().to_owned(),
                type_name: field.type_name().to_owned(),
                inner_type: field.inner_type().map(str::to_owned),
                nilable: field.nilable(),
                default_stated: field.raw_default().is_some(),
                stride_index_stated: field.raw_stride_index().is_some(),
                source_span: field.source_span(),
            })
        })
        .collect()
}

fn compare_fields(
    left: NativeCallableSignature<'_>,
    right: NativeCallableSignature<'_>,
    budget: &mut Budget<'_>,
) -> Result<FieldComparison, NativeError> {
    let mut differences = Vec::new();
    for (lane, left, right) in [
        (FieldLane::Argument, left.arguments(), right.arguments()),
        (FieldLane::Return, left.returns(), right.returns()),
    ] {
        for index in 0..left.len().max(right.len()) {
            budget.charge(1)?;
            let mut changed = Vec::new();
            match (left.get(index), right.get(index)) {
                (None, Some(_)) => changed.push(FieldFacet::Added),
                (Some(_), None) => changed.push(FieldFacet::NotPresentAfter),
                (Some(a), Some(b)) => {
                    if a.name() != b.name() {
                        changed.push(FieldFacet::Name);
                    }
                    if a.type_name() != b.type_name() {
                        changed.push(FieldFacet::TypeName);
                    }
                    if a.inner_type() != b.inner_type() {
                        changed.push(FieldFacet::InnerType);
                    }
                    if a.nilable() != b.nilable() {
                        changed.push(FieldFacet::Nilable);
                    }
                    if !raw_option_equal(a.raw_default(), b.raw_default(), budget)? {
                        changed.push(FieldFacet::Default);
                    }
                    if !raw_option_equal(a.raw_stride_index(), b.raw_stride_index(), budget)? {
                        changed.push(FieldFacet::StrideIndex);
                    }
                }
                (None, None) => return Err(error(NativeErrorCode::InvalidIdentity)),
            }
            if !changed.is_empty() {
                differences.push(FieldDifference {
                    lane,
                    ordinal: index + 1,
                    changed,
                });
            }
        }
    }
    Ok(if differences.is_empty() {
        FieldComparison::SelectedFieldsEqual {}
    } else {
        FieldComparison::SelectedFieldsDiffer { differences }
    })
}

fn raw_option_equal(
    left: Option<&RawValue>,
    right: Option<&RawValue>,
    budget: &mut Budget<'_>,
) -> Result<bool, NativeError> {
    match (left, right) {
        (None, None) => Ok(true),
        (Some(a), Some(b)) => raw_equal(a, b, 0, budget),
        _ => Ok(false),
    }
}

fn raw_equal(
    left: &RawValue,
    right: &RawValue,
    depth: usize,
    budget: &mut Budget<'_>,
) -> Result<bool, NativeError> {
    budget.charge(1)?;
    if depth > 48 {
        return Err(error(NativeErrorCode::Limit));
    }
    Ok(match (&left.kind, &right.kind) {
        (RawKind::Nil, RawKind::Nil) => true,
        (RawKind::Boolean(a), RawKind::Boolean(b)) => a == b,
        (RawKind::Number(a), RawKind::Number(b))
        | (RawKind::String(a), RawKind::String(b))
        | (RawKind::UnresolvedName(a), RawKind::UnresolvedName(b)) => a == b,
        (RawKind::Reference(a), RawKind::Reference(b)) => a == b,
        (
            RawKind::BinaryExpression {
                op: a_op,
                left: a_left,
                right: a_right,
            },
            RawKind::BinaryExpression {
                op: b_op,
                left: b_left,
                right: b_right,
            },
        ) => {
            a_op == b_op
                && raw_equal(a_left, b_left, depth + 1, budget)?
                && raw_equal(a_right, b_right, depth + 1, budget)?
        }
        (RawKind::Table(a), RawKind::Table(b)) => {
            if a.len() != b.len() {
                false
            } else {
                let mut equal = true;
                for (a, b) in a.iter().zip(b) {
                    budget.charge(1)?;
                    if a.key != b.key || !raw_equal(&a.value, &b.value, depth + 1, budget)? {
                        equal = false;
                        break;
                    }
                }
                equal
            }
        }
        _ => false,
    })
}

//! Genuine held Reference owners -> owner-neutral Candidate inputs. Project-local
//! API projections remain distinct from original Reference record identities.
use super::*;
use crate::live_project::LiveProjectRead;
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity, ReferenceGenerationId};
use wow_graph::lineage::*;
use wow_reference::{
    LookupResult,
    native_view::comparison::{
        CallableComparisonLimits, NativeCallableComparison, compare_reference_generations,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceLineageGuard {
    pub profile: ProfileIdentity,
    pub reference_generation: ReferenceGenerationId,
    pub view_digest: ContentDigest<CanonicalResult>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceLineageSelection {
    pub schema: Box<str>,
    pub comparison_profile: Box<str>,
    pub privacy: Box<str>,
    pub before: ReferenceLineageGuard,
    pub after: ReferenceLineageGuard,
    pub keys: Vec<String>,
    pub limits: CallableComparisonLimits,
}
impl ReferenceLineageSelection {
    pub(super) fn validate(&self) -> ServiceResult<()> {
        if self.schema.as_ref() != "wow-service/reference-lineage-selection/1"
            || self.comparison_profile.as_ref() != REFERENCE_COMPARISON_PROFILE
            || self.privacy.as_ref() != "local_metadata_only"
            || self.keys.is_empty()
            || self.keys.len() > self.limits.max_keys
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        self.limits
            .validate()
            .map_err(|_| failure(ServiceErrorCode::InvalidRequest))?;
        self.before
            .profile
            .validate()
            .map_err(|_| failure(ServiceErrorCode::InvalidRequest))?;
        self.after
            .profile
            .validate()
            .map_err(|_| failure(ServiceErrorCode::InvalidRequest))?;
        let mut keys = std::collections::BTreeSet::new();
        for key in &self.keys {
            wow_reference::ReferenceRecord::validate_key(key)
                .map_err(|_| failure(ServiceErrorCode::InvalidRequest))?;
            if !key.starts_with("function:") || !keys.insert(key) {
                return Err(failure(ServiceErrorCode::InvalidRequest));
            }
        }
        Ok(())
    }
}
pub(super) fn validate_capture(
    value: Option<&serde_json::Value>,
    binding: &LineageComparisonBinding,
) -> ServiceResult<()> {
    match (value, binding.reference.as_ref()) {
        (None, None) => Ok(()),
        (Some(value), Some(reference)) => {
            if capture_digest(value)? != reference.comparison_capture_digest
                || value
                    .get("comparison_digest")
                    .and_then(serde_json::Value::as_str)
                    != Some(reference.comparison_digest.to_string().as_str())
                || value.get("profile").and_then(serde_json::Value::as_str)
                    != Some(REFERENCE_COMPARISON_PROFILE)
                || value
                    .get("before_generation")
                    .and_then(serde_json::Value::as_str)
                    != Some(reference.before.reference_generation.to_string().as_str())
                || value
                    .get("after_generation")
                    .and_then(serde_json::Value::as_str)
                    != Some(reference.after.reference_generation.to_string().as_str())
                || value
                    .get("association_ceiling")
                    .and_then(serde_json::Value::as_str)
                    != Some("candidate")
                || value
                    .get("accepted_lineage_count")
                    .and_then(serde_json::Value::as_u64)
                    != Some(0)
                || value
                    .get("lineage_negative_authority")
                    .and_then(serde_json::Value::as_bool)
                    != Some(false)
            {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            Ok(())
        }
        _ => Err(failure(ServiceErrorCode::IdentityMismatch)),
    }
}
fn guarded_library<'a>(
    read: &'a LiveProjectRead,
    guard: &ReferenceLineageGuard,
) -> ServiceResult<&'a crate::local::RequalifiedNativeLibrary> {
    let api = read
        .native_api()
        .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
    let library = api.original_reference_library();
    if library.profile() != &guard.profile
        || library.reference().view.generation_id() != guard.reference_generation.to_string()
        || library.reference().view.self_digest() != guard.view_digest.to_string()
        || read.project().configuration().selected_profile() != library.profile()
        || read
            .project()
            .configuration()
            .reference_generation()
            .to_string()
            != library.reference().view.generation_id()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    if library.reference_target().is_none()
        || library.reference().native_documents().is_some()
        || library.reference().type_resolution_profile().is_some()
    {
        return Err(failure(ServiceErrorCode::ComponentUnavailable));
    }
    Ok(library)
}
pub(super) struct QualifiedReferenceInput {
    pub(super) input: ValidatedLineageInput,
    pub(super) comparison: NativeCallableComparison,
}
pub(super) fn capture_digest(
    value: &serde_json::Value,
) -> ServiceResult<ContentDigest<CanonicalResult>> {
    wow_core::domain_separated_digest("wow-service/reference-comparison-capture/1", value)
        .map(ContentDigest::from_bytes)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))
}
pub(super) fn build_input(
    before: &LiveProjectRead,
    after: &LiveProjectRead,
    selection: &ReferenceLineageSelection,
    limits: LineageLimits,
    stop: &AtomicBool,
) -> ServiceResult<QualifiedReferenceInput> {
    selection.validate()?;
    let left = guarded_library(before, &selection.before)?;
    let right = guarded_library(after, &selection.after)?;
    let a_target = left
        .reference_target()
        .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
    let b_target = right
        .reference_target()
        .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
    if a_target != b_target
        || left.environment() != right.environment()
        || left.profile().profile_kind() != right.profile().profile_kind()
        || left.profile().flavor_id() != right.profile().flavor_id()
        || left.profile().edition_id() != right.profile().edition_id()
        || left.profile().schema_versions() != right.profile().schema_versions()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let comparison = compare_reference_generations(
        left.reference(),
        left.callable_bindings().revision(),
        right.reference(),
        right.callable_bindings().revision(),
        &selection.keys,
        selection.limits,
        stop,
    )
    .map_err(|error| {
        failure(match error.code {
            wow_reference::native::NativeErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            wow_reference::native::NativeErrorCode::Limit => ServiceErrorCode::BudgetExceeded,
            _ => ServiceErrorCode::IdentityMismatch,
        })
    })?;
    let l = before.pair().graph_read(stop).map_err(graph_error)?;
    let r = after.pair().graph_read(stop).map_err(graph_error)?;
    let metadata=|library:&crate::local::RequalifiedNativeLibrary|->ServiceResult<LineageReferenceGenerationMetadata>{
        let target=library.reference_target().ok_or_else(||failure(ServiceErrorCode::ComponentUnavailable))?;
        Ok(LineageReferenceGenerationMetadata {product:target.product().into(),channel:target.channel().into(),profile:library.profile().clone(),reference_generation:library.reference().view.generation_id().parse().map_err(|_|failure(ServiceErrorCode::IdentityMismatch))?,reference_view_digest:library.reference().view.self_digest().parse().map_err(|_|failure(ServiceErrorCode::IdentityMismatch))?,native_projection_profile:library.reference().schema.into(),revision:library.callable_bindings().revision().into(),annotation_artifact_id:library.artifact().artifact_id().into(),source_map_id:library.sidecars().source_map().source_map_id().into()})
    };
    let mut keys = selection
        .keys
        .iter()
        .map(|key| key.as_str().into())
        .collect::<Vec<Box<str>>>();
    keys.sort();
    let reference = LineageReferenceComparisonBinding {
        comparison_profile_id: REFERENCE_COMPARISON_PROFILE.into(),
        comparison_digest: comparison
            .comparison_digest()
            .parse()
            .map_err(|_| failure(ServiceErrorCode::IdentityMismatch))?,
        comparison_capture_digest: capture_digest(
            &serde_json::to_value(&comparison)
                .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?,
        )?,
        environment: left.environment().into(),
        projection_scope: "project_local_native_api".into(),
        before: metadata(left)?,
        after: metadata(right)?,
        selected_keys: keys,
    };
    let pair_id = crate::identity::canonical_digest(
        "reference-generation-pair:sha256:",
        &(
            &reference,
            before.store_generation_id(),
            after.store_generation_id(),
            before.publication_set_id(),
            after.publication_set_id(),
        ),
    )?;
    let mut data = ProjectFingerprintPartition {
        schema: REFERENCE_KEY_INPUT_SCHEMA.into(),
        producer_id: REFERENCE_KEY_PRODUCER.into(),
        binding: LineageComparisonBinding {
            universe_class: LineageUniverseClass::ReferencePlatformContract,
            profile_id: REFERENCE_KEY_LINEAGE_PROFILE.into(),
            pair_id,
            before: wow_project::graph::lineage_input::retained_lineage_generation_binding(
                before.pair(),
                l.owner()
                    .producer_lookup(stop)
                    .map_err(graph_error)?
                    .scope(),
            ),
            after: wow_project::graph::lineage_input::retained_lineage_generation_binding(
                after.pair(),
                r.owner()
                    .producer_lookup(stop)
                    .map_err(graph_error)?
                    .scope(),
            ),
            source: None,
            reference: Some(reference),
        },
        records: Vec::new(),
        reference_records: Vec::new(),
        reference_omissions: Vec::new(),
        partitions: Vec::new(),
        coverage: "partial".into(),
        source_bodies: "excluded".into(),
        proof_ceiling: wow_graph::GraphConfidence::Candidate,
    };
    for (role, read, library, graph) in [
        (LineageRole::Before, before, left, &l),
        (LineageRole::After, after, right, &r),
    ] {
        checkpoint(stop)?;
        let api = read
            .native_api()
            .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
        for key in &selection.keys {
            checkpoint(stop)?;
            let state = match library
                .reference()
                .view
                .lookup(wow_reference::native_view::NATIVE_API_PARTITION, key)
            {
                LookupResult::Found(record) => {
                    let digest = record
                        .digest()
                        .map_err(|_| failure(ServiceErrorCode::IdentityMismatch))?;
                    if let Some(origin) = api.reference_lineage_origin(graph, key, &digest, stop)? {
                        data.reference_records.push(LineageReferenceCallableRecord {
                            role,
                            reference_entity: LineageReferenceEntityAddress {
                                reference_profile_id: library.profile().profile_id().clone(),
                                reference_generation: library
                                    .reference()
                                    .view
                                    .generation_id()
                                    .parse()
                                    .map_err(|_| failure(ServiceErrorCode::IdentityMismatch))?,
                                reference_view_digest: library
                                    .reference()
                                    .view
                                    .self_digest()
                                    .parse()
                                    .map_err(|_| failure(ServiceErrorCode::IdentityMismatch))?,
                                record_key: key.as_str().into(),
                                record_digest: digest
                                    .parse()
                                    .map_err(|_| failure(ServiceErrorCode::IdentityMismatch))?,
                            },
                            node: origin.node,
                            source_assertion: origin.source_assertion,
                            source_handle: origin.source_handle.clone(),
                            evidence: origin.evidence.clone(),
                            feature: reference_key_feature(key).map_err(graph_error)?,
                        });
                        continue;
                    }
                    LineageReferenceOmissionState::NativeProjectionUnavailable
                }
                LookupResult::Conflict(_) => LineageReferenceOmissionState::Conflict,
                LookupResult::AuthoritativeAbsence => {
                    LineageReferenceOmissionState::AuthoritativeExactKeyAbsence
                }
                LookupResult::Unknown(_) => LineageReferenceOmissionState::PartialOrNotEvaluated,
            };
            data.reference_omissions.push(LineageReferenceOmission {
                role,
                original_key: key.as_str().into(),
                state,
            });
        }
        for partition in graph.owner().partitions() {
            checkpoint(stop)?;
            data.partitions.push(LineagePartitionCoverage {
                role,
                partition_id: partition.partition_id().into(),
                partition_digest: partition.partition_digest().into(),
                producer_version: partition.producer_version().into(),
                coverage: partition.coverage().to_vec(),
                conflicts: partition
                    .batch()
                    .assertion_records()
                    .map_or_else(Vec::new, |records| records.conflicts.clone()),
            });
        }
    }
    #[cfg(test)]
    eprintln!(
        "reference-input stage=assembled records={} omissions={}",
        data.reference_records.len(),
        data.reference_omissions.len()
    );
    let input =
        validate_reference_callable_partition(data, &l, &r, limits, stop).map_err(|error| {
            #[cfg(test)]
            eprintln!(
                "reference-input graph-refusal code={:?} message={}",
                error.code(),
                error.message()
            );
            graph_error(error)
        })?;
    Ok(QualifiedReferenceInput { input, comparison })
}

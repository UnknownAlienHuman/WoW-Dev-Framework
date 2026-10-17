//! Owner-neutral original Reference identities and their genuine local native
//! API projections. Exact-key discovery is Candidate, never continuity.
use super::*;
use crate::{GraphNode, GraphPartitionRead, GraphProposalValue};
use serde::{Deserialize, Serialize};
use wow_core::{
    CanonicalResult, ContentDigest, EvidenceRecord, ProfileIdentity, ReferenceGenerationId,
    SourceHandle,
};

pub const REFERENCE_KEY_LINEAGE_PROFILE: &str =
    "wow-graph/lineage/reference-callable-key-candidates/1";
pub const REFERENCE_KEY_INPUT_SCHEMA: &str = "wow-graph/lineage-reference-callable-input/1";
pub const REFERENCE_KEY_PRODUCER: &str = "wow.reference.lineage.original_callable_key";
pub const REFERENCE_KEY_FEATURE_PROFILE: &str =
    "wow-graph/reference-original-callable-key-feature/1";
pub const REFERENCE_COMPARISON_PROFILE: &str = "wow-reference/native-callable-comparison/1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageReferenceEntityAddress {
    pub reference_profile_id: wow_core::ProfileId,
    pub reference_generation: ReferenceGenerationId,
    pub reference_view_digest: ContentDigest<CanonicalResult>,
    pub record_key: Box<str>,
    pub record_digest: ContentDigest<CanonicalResult>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageReferenceGenerationMetadata {
    pub product: Box<str>,
    pub channel: Box<str>,
    pub profile: ProfileIdentity,
    pub reference_generation: ReferenceGenerationId,
    pub reference_view_digest: ContentDigest<CanonicalResult>,
    pub native_projection_profile: Box<str>,
    pub revision: Box<str>,
    pub annotation_artifact_id: Box<str>,
    pub source_map_id: Box<str>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageReferenceComparisonBinding {
    pub comparison_profile_id: Box<str>,
    pub comparison_digest: ContentDigest<CanonicalResult>,
    pub comparison_capture_digest: ContentDigest<CanonicalResult>,
    pub environment: Box<str>,
    pub projection_scope: Box<str>,
    pub before: LineageReferenceGenerationMetadata,
    pub after: LineageReferenceGenerationMetadata,
    pub selected_keys: Vec<Box<str>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageReferenceKeyFeature {
    pub profile_id: Box<str>,
    pub original_key: Box<str>,
    pub fingerprint: ContentDigest<CanonicalResult>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageReferenceCallableRecord {
    pub role: LineageRole,
    pub reference_entity: LineageReferenceEntityAddress,
    pub node: GraphNode,
    pub source_assertion: crate::GraphAssertionRef,
    pub source_handle: SourceHandle,
    pub evidence: EvidenceRecord,
    pub feature: LineageReferenceKeyFeature,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageReferenceOmissionState {
    Conflict,
    PartialOrNotEvaluated,
    AuthoritativeExactKeyAbsence,
    NativeProjectionUnavailable,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageReferenceOmission {
    pub role: LineageRole,
    pub original_key: Box<str>,
    pub state: LineageReferenceOmissionState,
}

pub fn reference_key_feature(key: &str) -> GraphResult<LineageReferenceKeyFeature> {
    validate_key(key)?;
    let digest = wow_core::domain_separated_digest(REFERENCE_KEY_FEATURE_PROFILE, &key)
        .map_err(|_| invalid())?;
    Ok(LineageReferenceKeyFeature {
        profile_id: REFERENCE_KEY_FEATURE_PROFILE.into(),
        original_key: key.into(),
        fingerprint: ContentDigest::from_bytes(digest),
    })
}
fn validate_key(key: &str) -> GraphResult<()> {
    if !key.starts_with("function:")
        || key.len() <= "function:".len()
        || key.len() > 512
        || key.chars().any(char::is_control)
    {
        return Err(invalid());
    }
    Ok(())
}

pub(super) fn validate_binding(data: &ProjectFingerprintPartition) -> GraphResult<()> {
    let binding = &data.binding;
    let reference = binding.reference.as_ref().ok_or_else(invalid)?;
    let (a, b) = (&reference.before, &reference.after);
    if binding.universe_class != LineageUniverseClass::ReferencePlatformContract
        || binding.profile_id.as_ref() != REFERENCE_KEY_LINEAGE_PROFILE
        || binding.source.is_some()
        || reference.comparison_profile_id.as_ref() != REFERENCE_COMPARISON_PROFILE
        || reference.projection_scope.as_ref() != "project_local_native_api"
        || reference.environment.is_empty()
        || reference.environment.len() > 128
        || !reference
            .environment
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || value == b'_')
        || !data.records.is_empty()
        || reference.selected_keys.is_empty()
        || reference.selected_keys.len() > 4096
        || a.profile.profile_kind() != b.profile.profile_kind()
        || a.profile.flavor_id() != b.profile.flavor_id()
        || a.profile.edition_id() != b.profile.edition_id()
        || a.profile.schema_versions() != b.profile.schema_versions()
        || a.product != b.product
        || a.channel != b.channel
        || a.native_projection_profile != b.native_projection_profile
        || !matches!(
            a.native_projection_profile.as_ref(),
            "wow-reference/native-callable-view/1" | "wow-reference/native-callable-view/3"
        )
        || binding.before.selected_profile != a.profile
        || binding.after.selected_profile != b.profile
        || binding.before.reference_generation_id.as_ref() != a.reference_generation.to_string()
        || binding.after.reference_generation_id.as_ref() != b.reference_generation.to_string()
        || !matches!(
            binding.before.project_kind.as_ref(),
            "fixture" | "repository"
        )
        || !matches!(
            binding.after.project_kind.as_ref(),
            "fixture" | "repository"
        )
    {
        return Err(invalid());
    }
    for side in [a, b] {
        side.profile.validate().map_err(|_| invalid())?;
        for label in [&side.product, &side.channel] {
            if label.is_empty()
                || label.len() > 64
                || !label.bytes().all(|c| {
                    c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'_' | b'-')
                })
            {
                return Err(invalid());
            }
        }
        if !matches!(side.revision.len(), 40 | 64)
            || !side
                .revision
                .bytes()
                .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
            || side.annotation_artifact_id.is_empty()
            || side.annotation_artifact_id.len() > 512
            || side.source_map_id.is_empty()
            || side.source_map_id.len() > 512
        {
            return Err(invalid());
        }
    }
    for key in &reference.selected_keys {
        validate_key(key)?;
    }
    if reference
        .selected_keys
        .windows(2)
        .any(|keys| keys[0] >= keys[1])
    {
        return Err(invalid());
    }
    Ok(())
}

pub fn validate_reference_callable_partition(
    data: ProjectFingerprintPartition,
    before: &GraphPartitionRead<'_>,
    after: &GraphPartitionRead<'_>,
    limits: LineageLimits,
    stop: &AtomicBool,
) -> GraphResult<ValidatedLineageInput> {
    if data.binding.universe_class != LineageUniverseClass::ReferencePlatformContract {
        return Err(invalid());
    }
    model::validate_fingerprint_partition(data, before, after, limits, stop)
}

pub(super) fn validate_records(
    data: &mut ProjectFingerprintPartition,
    before: &GraphPartitionRead<'_>,
    after: &GraphPartitionRead<'_>,
    ledger: &mut OperationBudget<'_>,
    stop: &AtomicBool,
) -> GraphResult<()> {
    validate_binding(data)?;
    if before.owner().registry() != after.owner().registry() {
        return Err(invalid());
    }
    let reference = data.binding.reference.as_ref().ok_or_else(invalid)?;
    data.reference_records
        .sort_by(|a, b| (a.role, a.node.node_id()).cmp(&(b.role, b.node.node_id())));
    if data.reference_records.windows(2).any(|rows| {
        rows[0].role == rows[1].role
            && (rows[0].node.node_id() == rows[1].node.node_id()
                || rows[0].reference_entity.record_key == rows[1].reference_entity.record_key)
    }) {
        return Err(invalid());
    }
    let left = before.owner().producer_lookup(stop)?;
    let right = after.owner().producer_lookup(stop)?;
    let mut seen = std::collections::BTreeSet::new();
    for record in &data.reference_records {
        ledger.work(1)?;
        let (binding, metadata, graph, lookup) = match record.role {
            LineageRole::Before => (&data.binding.before, &reference.before, before, &left),
            LineageRole::After => (&data.binding.after, &reference.after, after, &right),
        };
        let original = &record.reference_entity;
        if original.reference_profile_id != *metadata.profile.profile_id()
            || original.reference_generation != metadata.reference_generation
            || original.reference_view_digest != metadata.reference_view_digest
            || reference
                .selected_keys
                .binary_search(&original.record_key)
                .is_err()
            || record.feature != reference_key_feature(&original.record_key)?
            || !seen.insert((record.role, original.record_key.as_ref()))
            || &binding.original_graph_scope != lookup.scope()
        {
            return Err(invalid());
        }
        let entity = lookup.entity(
            &binding.original_graph_scope,
            &record.source_assertion,
            stop,
        )?;
        let crate::GraphAssertionRef::Producer {
            partition_id,
            assertion,
            ..
        } = &record.source_assertion
        else {
            return Err(invalid());
        };
        let node = graph.published_entity(partition_id, &assertion.proposal_id, stop)?;
        if node != &record.node
            || node.kind() != "native_api_callable"
            || entity.proposal().entity_kind_id() != "native_api_callable"
            || !entity
                .proposal()
                .source_handle_ids()
                .contains(&record.source_handle.handle_id())
            || !entity
                .proposal()
                .evidence_ids()
                .contains(&record.evidence.evidence_id())
            || record.evidence.context_id() != binding.original_graph_scope.source_context_id
            || !record
                .evidence
                .source_handle_ids()
                .contains(&record.source_handle.handle_id())
        {
            return Err(invalid());
        }
        for (key, expected) in [
            (
                "reference_profile_id",
                original.reference_profile_id.as_str(),
            ),
            (
                "reference_generation_id",
                &original.reference_generation.to_string(),
            ),
            (
                "reference_digest",
                &original.reference_view_digest.to_string(),
            ),
            ("record_key", original.record_key.as_ref()),
            ("record_digest", &original.record_digest.to_string()),
        ] {
            if !matches!(entity.proposal().semantic_key().get(key),Some(GraphProposalValue::String(value)) if value.as_ref()==expected)
            {
                return Err(invalid());
            }
        }
    }
    data.reference_omissions
        .sort_by(|a, b| (a.role, a.original_key.as_ref()).cmp(&(b.role, b.original_key.as_ref())));
    for omission in &data.reference_omissions {
        ledger.work(1)?;
        if reference
            .selected_keys
            .binary_search(&omission.original_key)
            .is_err()
            || !seen.insert((omission.role, omission.original_key.as_ref()))
        {
            return Err(invalid());
        }
    }
    if seen.len()
        != reference
            .selected_keys
            .len()
            .checked_mul(2)
            .ok_or_else(budget)?
    {
        return Err(invalid());
    }
    Ok(())
}

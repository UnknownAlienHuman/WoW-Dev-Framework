use serde::{Deserialize, Serialize};

use crate::digest::{
    ExternalGenerationId, GenerationContextId, ProjectGenerationId, ReferenceGenerationId,
};
use crate::error::{CoreErrorCode, CoreResult, mismatch_error, validation_error};
use crate::ids::{ProducerId, ToolVersion, validate_lower_segment};
use crate::profile::{ProfileIdentity, SchemaVersionEntry};

/// Exact producer implementation version in one generation context.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerVersionEntry {
    producer_id: ProducerId,
    version: ToolVersion,
}

impl ProducerVersionEntry {
    /// Creates a producer/version entry.
    #[must_use]
    pub const fn new(producer_id: ProducerId, version: ToolVersion) -> Self {
        Self {
            producer_id,
            version,
        }
    }

    /// Producer ID.
    #[must_use]
    pub const fn producer_id(&self) -> &ProducerId {
        &self.producer_id
    }

    /// Producer version.
    #[must_use]
    pub const fn version(&self) -> &ToolVersion {
        &self.version
    }
}

/// Provider-scoped external generation kept separate from project/reference identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalGeneration {
    provider_id: String,
    scope_id: String,
    external_generation_id: ExternalGenerationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_revision: Option<String>,
}

impl ExternalGeneration {
    /// Creates and validates an external generation entry.
    pub fn new(
        provider_id: impl Into<String>,
        scope_id: impl Into<String>,
        external_generation_id: ExternalGenerationId,
        source_revision: Option<String>,
    ) -> CoreResult<Self> {
        let provider_id = provider_id.into();
        let scope_id = scope_id.into();
        validate_lower_segment(
            &provider_id,
            "validate_generation_context",
            "external_generations.provider_id",
        )?;
        if external_generation_id.provider() != provider_id {
            return Err(mismatch_error(
                "validate_generation_context",
                CoreErrorCode::GenerationMismatch,
                "external_generations.external_generation_id",
            ));
        }
        validate_bounded_text(
            &scope_id,
            "external_generations.scope_id",
            "validate_generation_context",
        )?;
        if let Some(revision) = &source_revision {
            validate_bounded_text(
                revision,
                "external_generations.source_revision",
                "validate_generation_context",
            )?;
        }
        Ok(Self {
            provider_id,
            scope_id,
            external_generation_id,
            source_revision,
        })
    }

    /// Provider identifier.
    #[must_use]
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Provider-local scope identifier.
    #[must_use]
    pub fn scope_id(&self) -> &str {
        &self.scope_id
    }

    /// Exact external generation ID.
    #[must_use]
    pub const fn external_generation_id(&self) -> &ExternalGenerationId {
        &self.external_generation_id
    }
}

/// Explicit generation-context merge mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeMode {
    Strict,
    ExtendMissingOptional,
    ExternalUnion,
}

/// One coherent exact generation/profile/tool set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerationContext {
    context_id: GenerationContextId,
    #[serde(rename = "profile")]
    profile_identity: ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_generation: Option<ProjectGenerationId>,
    external_generations: Vec<ExternalGeneration>,
    schema_versions: Vec<SchemaVersionEntry>,
    producer_versions: Vec<ProducerVersionEntry>,
}

impl GenerationContext {
    /// Exact context ID.
    #[must_use]
    pub const fn context_id(&self) -> GenerationContextId {
        self.context_id
    }

    /// Structured profile identity.
    #[must_use]
    pub const fn profile_identity(&self) -> &ProfileIdentity {
        &self.profile_identity
    }

    /// Reference generation.
    #[must_use]
    pub const fn reference_generation(&self) -> ReferenceGenerationId {
        self.reference_generation
    }

    /// Optional project generation.
    #[must_use]
    pub const fn project_generation(&self) -> Option<ProjectGenerationId> {
        self.project_generation
    }

    /// Exact external generations.
    #[must_use]
    pub fn external_generations(&self) -> &[ExternalGeneration] {
        &self.external_generations
    }

    /// Schema versions.
    #[must_use]
    pub fn schema_versions(&self) -> &[SchemaVersionEntry] {
        &self.schema_versions
    }

    /// Producer versions.
    #[must_use]
    pub fn producer_versions(&self) -> &[ProducerVersionEntry] {
        &self.producer_versions
    }

    /// Revalidates ordering, uniqueness, profile structure, and context ID.
    pub fn validate(&self) -> CoreResult<()> {
        self.profile_identity.validate()?;
        validate_sorted_unique_schema(&self.schema_versions)?;
        validate_sorted_unique_producers(&self.producer_versions)?;
        validate_sorted_unique_external(&self.external_generations)?;
        let expected = derive_context_id(
            &self.profile_identity,
            self.reference_generation,
            self.project_generation,
            &self.external_generations,
            &self.schema_versions,
            &self.producer_versions,
        )?;
        if expected != self.context_id {
            return Err(mismatch_error(
                "validate_generation_context",
                CoreErrorCode::GenerationMismatch,
                "context_id",
            ));
        }
        Ok(())
    }

    /// Requires byte-for-byte exact generation context identity.
    pub fn require_same_generation(&self, other: &Self) -> CoreResult<()> {
        if self.context_id == other.context_id {
            Ok(())
        } else {
            Err(mismatch_error(
                "require_same_generation",
                CoreErrorCode::GenerationMismatch,
                "context_id",
            ))
        }
    }

    /// Merges two contexts only under the requested explicit mode.
    pub fn merge(&self, other: &Self, mode: MergeMode) -> CoreResult<Self> {
        self.validate()?;
        other.validate()?;
        if self.profile_identity != other.profile_identity {
            return Err(mismatch_error(
                "merge_generation_context",
                CoreErrorCode::ProfileMismatch,
                "profile",
            ));
        }
        if self.reference_generation != other.reference_generation {
            return Err(mismatch_error(
                "merge_generation_context",
                CoreErrorCode::GenerationMismatch,
                "reference_generation",
            ));
        }
        let project_generation =
            merge_project_generation(self.project_generation, other.project_generation, mode)?;
        let schema_versions = merge_identical_entries(
            &self.schema_versions,
            &other.schema_versions,
            "schema_versions",
            CoreErrorCode::DuplicateSchemaId,
        )?;
        let producer_versions = merge_identical_entries(
            &self.producer_versions,
            &other.producer_versions,
            "producer_versions",
            CoreErrorCode::DuplicateProducerId,
        )?;
        let external_generations = merge_external_generations(
            &self.external_generations,
            &other.external_generations,
            mode,
        )?;

        GenerationContextBuilder::new(self.profile_identity.clone(), self.reference_generation)
            .project_generation_option(project_generation)
            .external_generations(external_generations)
            .schema_versions(schema_versions)
            .producer_versions(producer_versions)
            .build()
    }
}

/// Builder for a canonical `GenerationContext`.
#[derive(Debug, Clone)]
pub struct GenerationContextBuilder {
    profile_identity: ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    project_generation: Option<ProjectGenerationId>,
    external_generations: Vec<ExternalGeneration>,
    schema_versions: Vec<SchemaVersionEntry>,
    producer_versions: Vec<ProducerVersionEntry>,
}

impl GenerationContextBuilder {
    /// Starts a context from mandatory profile/reference identity.
    #[must_use]
    pub const fn new(
        profile_identity: ProfileIdentity,
        reference_generation: ReferenceGenerationId,
    ) -> Self {
        Self {
            profile_identity,
            reference_generation,
            project_generation: None,
            external_generations: Vec::new(),
            schema_versions: Vec::new(),
            producer_versions: Vec::new(),
        }
    }

    /// Adds a project generation.
    #[must_use]
    pub const fn project_generation(mut self, project_generation: ProjectGenerationId) -> Self {
        self.project_generation = Some(project_generation);
        self
    }

    /// Sets an optional project generation.
    #[must_use]
    pub(crate) const fn project_generation_option(
        mut self,
        project_generation: Option<ProjectGenerationId>,
    ) -> Self {
        self.project_generation = project_generation;
        self
    }

    /// Replaces external generation entries.
    #[must_use]
    pub fn external_generations(mut self, external_generations: Vec<ExternalGeneration>) -> Self {
        self.external_generations = external_generations;
        self
    }

    /// Replaces context schema versions.
    #[must_use]
    pub fn schema_versions(mut self, schema_versions: Vec<SchemaVersionEntry>) -> Self {
        self.schema_versions = schema_versions;
        self
    }

    /// Replaces producer versions.
    #[must_use]
    pub fn producer_versions(mut self, producer_versions: Vec<ProducerVersionEntry>) -> Self {
        self.producer_versions = producer_versions;
        self
    }

    /// Sorts, validates, derives the context ID, and builds the context.
    pub fn build(mut self) -> CoreResult<GenerationContext> {
        self.profile_identity.validate()?;
        self.external_generations.sort();
        self.external_generations.dedup();
        self.schema_versions.sort();
        self.producer_versions.sort();
        validate_sorted_unique_external(&self.external_generations)?;
        validate_sorted_unique_schema(&self.schema_versions)?;
        validate_sorted_unique_producers(&self.producer_versions)?;
        let context_id = derive_context_id(
            &self.profile_identity,
            self.reference_generation,
            self.project_generation,
            &self.external_generations,
            &self.schema_versions,
            &self.producer_versions,
        )?;
        Ok(GenerationContext {
            context_id,
            profile_identity: self.profile_identity,
            reference_generation: self.reference_generation,
            project_generation: self.project_generation,
            external_generations: self.external_generations,
            schema_versions: self.schema_versions,
            producer_versions: self.producer_versions,
        })
    }
}

#[derive(Serialize)]
struct ContextIdentityProjection<'a> {
    external_generations: &'a [ExternalGeneration],
    producer_versions: &'a [ProducerVersionEntry],
    profile: &'a ProfileIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_generation: Option<ProjectGenerationId>,
    reference_generation: ReferenceGenerationId,
    schema_versions: &'a [SchemaVersionEntry],
}

fn derive_context_id(
    profile: &ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    project_generation: Option<ProjectGenerationId>,
    external_generations: &[ExternalGeneration],
    schema_versions: &[SchemaVersionEntry],
    producer_versions: &[ProducerVersionEntry],
) -> CoreResult<GenerationContextId> {
    GenerationContextId::derive(&ContextIdentityProjection {
        external_generations,
        producer_versions,
        profile,
        project_generation,
        reference_generation,
        schema_versions,
    })
}

fn merge_project_generation(
    left: Option<ProjectGenerationId>,
    right: Option<ProjectGenerationId>,
    mode: MergeMode,
) -> CoreResult<Option<ProjectGenerationId>> {
    match (left, right, mode) {
        (left, right, MergeMode::Strict) if left == right => Ok(left),
        (Some(left), Some(right), _) if left == right => Ok(Some(left)),
        (None, Some(value), MergeMode::ExtendMissingOptional)
        | (Some(value), None, MergeMode::ExtendMissingOptional) => Ok(Some(value)),
        (None, None, _) => Ok(None),
        (Some(_), Some(_), _) => Err(mismatch_error(
            "merge_generation_context",
            CoreErrorCode::GenerationMismatch,
            "project_generation",
        )),
        _ => Err(validation_error(
            "merge_generation_context",
            CoreErrorCode::MergeModeViolation,
            "project_generation",
        )),
    }
}

fn merge_identical_entries<T: Clone + Ord + PartialEq>(
    left: &[T],
    right: &[T],
    field: &'static str,
    code: CoreErrorCode,
) -> CoreResult<Vec<T>> {
    if left == right {
        return Ok(left.to_vec());
    }
    let mut combined = left.to_vec();
    combined.extend_from_slice(right);
    combined.sort();
    combined.dedup();
    if combined.len() == left.len().max(right.len())
        && left.iter().all(|entry| combined.contains(entry))
        && right.iter().all(|entry| combined.contains(entry))
    {
        Ok(combined)
    } else {
        Err(validation_error("merge_generation_context", code, field))
    }
}

fn merge_external_generations(
    left: &[ExternalGeneration],
    right: &[ExternalGeneration],
    mode: MergeMode,
) -> CoreResult<Vec<ExternalGeneration>> {
    if mode != MergeMode::ExternalUnion {
        if left == right {
            return Ok(left.to_vec());
        }
        return Err(validation_error(
            "merge_generation_context",
            CoreErrorCode::MergeModeViolation,
            "external_generations",
        ));
    }

    let mut combined = left.to_vec();
    combined.extend_from_slice(right);
    combined.sort();
    let mut result: Vec<ExternalGeneration> = Vec::new();
    for entry in combined {
        if let Some(previous) = result.last()
            && previous.provider_id == entry.provider_id
            && previous.scope_id == entry.scope_id
        {
            if previous == &entry {
                continue;
            }
            return Err(validation_error(
                "merge_generation_context",
                CoreErrorCode::DuplicateExternalGenerationScope,
                "external_generations",
            ));
        }
        result.push(entry);
    }
    Ok(result)
}

fn validate_sorted_unique_schema(entries: &[SchemaVersionEntry]) -> CoreResult<()> {
    for pair in entries.windows(2) {
        if pair[0].schema_id() == pair[1].schema_id() {
            return Err(validation_error(
                "validate_generation_context",
                CoreErrorCode::DuplicateSchemaId,
                "schema_versions",
            ));
        }
        if pair[0] > pair[1] {
            return Err(validation_error(
                "validate_generation_context",
                CoreErrorCode::GenerationMismatch,
                "schema_versions",
            )
            .with_argument("reason", "noncanonical_order"));
        }
    }
    Ok(())
}

fn validate_sorted_unique_producers(entries: &[ProducerVersionEntry]) -> CoreResult<()> {
    for pair in entries.windows(2) {
        if pair[0].producer_id == pair[1].producer_id {
            return Err(validation_error(
                "validate_generation_context",
                CoreErrorCode::DuplicateProducerId,
                "producer_versions",
            ));
        }
        if pair[0] > pair[1] {
            return Err(validation_error(
                "validate_generation_context",
                CoreErrorCode::GenerationMismatch,
                "producer_versions",
            )
            .with_argument("reason", "noncanonical_order"));
        }
    }
    Ok(())
}

fn validate_sorted_unique_external(entries: &[ExternalGeneration]) -> CoreResult<()> {
    for pair in entries.windows(2) {
        if pair[0].provider_id == pair[1].provider_id && pair[0].scope_id == pair[1].scope_id {
            return Err(validation_error(
                "validate_generation_context",
                CoreErrorCode::DuplicateExternalGenerationScope,
                "external_generations",
            ));
        }
        if pair[0] > pair[1] {
            return Err(validation_error(
                "validate_generation_context",
                CoreErrorCode::GenerationMismatch,
                "external_generations",
            )
            .with_argument("reason", "noncanonical_order"));
        }
    }
    Ok(())
}

fn validate_bounded_text(
    value: &str,
    field: &'static str,
    operation: &'static str,
) -> CoreResult<()> {
    if value.is_empty()
        || value.len() > 1_024
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        Err(validation_error(
            operation,
            CoreErrorCode::InvalidIdentifier,
            field,
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::digest::{
        ContentDigest, ProjectGenerationId, ReferenceGenerationId, SourceLogicalSnapshot,
    };
    use crate::ids::{ProducerId, ProfileId, SchemaId, ToolVersion};
    use crate::profile::{ProfileIdentityBuilder, ProfileKind, SchemaVersionEntry, SourceKind};

    use super::{GenerationContextBuilder, ProducerVersionEntry};

    #[test]
    fn fixture_context_matches_normative_hash_vector() -> crate::CoreResult<()> {
        let fixture_profile_schema = SchemaVersionEntry::new(
            SchemaId::from_str("schema:wow:fixture-profile")?,
            ToolVersion::from_str("0.1.0")?,
        );
        let profile = ProfileIdentityBuilder::new(
            ProfileId::from_str("profile:fixture:e0-retail-120100")?,
            ProfileKind::Fixture,
            "retail",
            120_100,
            SourceKind::SyntheticFixture,
            "fixture:e0-rev1",
            ContentDigest::<SourceLogicalSnapshot>::from_bytes([1_u8; 32]),
        )
        .client_version(ToolVersion::from_str("12.1.0")?)
        .client_build(69_497)
        .schema_versions(vec![fixture_profile_schema])
        .fixture_scope("e0 vertical slice only")
        .build()?;

        let reference = ReferenceGenerationId::from_str(
            "generation:reference:sha256:8e56faa8b5c7efae0e0c8468c48101ed8d2cfef206b40e6e96106993096d2786",
        )?;
        let project = ProjectGenerationId::from_str(
            "generation:project:sha256:e606bd7594eb932275741dbe51afbd46ecdf59388f370b71eaa1422dd0259463",
        )?;
        let schemas = vec![
            SchemaVersionEntry::new(
                SchemaId::from_str("schema:wow:check-result")?,
                ToolVersion::from_str("0.1.0")?,
            ),
            SchemaVersionEntry::new(
                SchemaId::from_str("schema:wow:core-contract")?,
                ToolVersion::from_str("0.1.0")?,
            ),
        ];
        let producer_names = [
            "wow.core",
            "wow.emmy",
            "wow.project",
            "wow.reference",
            "wow.rules",
            "wow.service",
        ];
        let mut producers = Vec::new();
        for producer in producer_names {
            producers.push(ProducerVersionEntry::new(
                ProducerId::from_str(producer)?,
                ToolVersion::from_str("0.0.0-e0")?,
            ));
        }

        let context = GenerationContextBuilder::new(profile, reference)
            .project_generation(project)
            .schema_versions(schemas)
            .producer_versions(producers)
            .build()?;
        assert_eq!(
            context.context_id().to_string(),
            "context:sha256:bb8a207574e382f9c102613a9c24c5dc1923e1fe8b3e0434b67bd695877d1871"
        );
        Ok(())
    }
}

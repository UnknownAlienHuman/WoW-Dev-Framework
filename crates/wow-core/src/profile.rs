use serde::{Deserialize, Serialize};

use crate::digest::{ContentDigest, CorrectionSet, SourceLogicalSnapshot};
use crate::error::{CoreErrorCode, CoreResult, validation_error};
use crate::ids::{ProducerId, ProfileId, SchemaId, ToolVersion, validate_lower_segment};

/// Profile identity class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    Fixture,
    Release,
}

/// Source class used to construct a profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    SyntheticFixture,
    BlizzardSnapshot,
}

/// Exact schema version entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaVersionEntry {
    schema_id: SchemaId,
    version: ToolVersion,
}

impl SchemaVersionEntry {
    /// Creates an exact schema-version pair.
    #[must_use]
    pub const fn new(schema_id: SchemaId, version: ToolVersion) -> Self {
        Self { schema_id, version }
    }

    /// Schema ID.
    #[must_use]
    pub const fn schema_id(&self) -> &SchemaId {
        &self.schema_id
    }

    /// Schema version.
    #[must_use]
    pub const fn version(&self) -> &ToolVersion {
        &self.version
    }
}

/// Validated structured identity for one fixture or release profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileIdentity {
    profile_id: ProfileId,
    profile_kind: ProfileKind,
    flavor_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    edition_id: Option<String>,
    interface: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_version: Option<ToolVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_build: Option<u64>,
    source_kind: SourceKind,
    source_revision: String,
    source_logical_digest: ContentDigest<SourceLogicalSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    builder_id: Option<ProducerId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    builder_version: Option<ToolVersion>,
    schema_versions: Vec<SchemaVersionEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    correction_set_digest: Option<ContentDigest<CorrectionSet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fixture_scope: Option<String>,
}

impl ProfileIdentity {
    /// Validates an already materialized profile.
    pub fn validate(&self) -> CoreResult<()> {
        validate_profile_fields(self)
    }

    /// Stable profile label.
    #[must_use]
    pub const fn profile_id(&self) -> &ProfileId {
        &self.profile_id
    }

    /// Fixture or release class.
    #[must_use]
    pub const fn profile_kind(&self) -> ProfileKind {
        self.profile_kind
    }

    /// Flavor segment.
    #[must_use]
    pub fn flavor_id(&self) -> &str {
        &self.flavor_id
    }

    /// Optional edition segment.
    #[must_use]
    pub fn edition_id(&self) -> Option<&str> {
        self.edition_id.as_deref()
    }

    /// Interface number.
    #[must_use]
    pub const fn interface(&self) -> u64 {
        self.interface
    }

    /// Exact source logical digest.
    #[must_use]
    pub const fn source_logical_digest(&self) -> &ContentDigest<SourceLogicalSnapshot> {
        &self.source_logical_digest
    }

    /// Schema versions in canonical order.
    #[must_use]
    pub fn schema_versions(&self) -> &[SchemaVersionEntry] {
        &self.schema_versions
    }

    /// Compares every identity field and reports label/material distinctions.
    #[must_use]
    pub fn compare(&self, other: &Self) -> ProfileComparison {
        if self == other {
            return ProfileComparison::Identical;
        }

        let same_label = self.profile_id == other.profile_id;
        let same_material = self.material_eq(other);
        if same_label {
            ProfileComparison::SameLabelDifferentIdentity {
                differing_fields: self.differing_fields(other),
            }
        } else if same_material {
            ProfileComparison::DifferentLabelSameMaterial
        } else {
            ProfileComparison::Different
        }
    }

    fn material_eq(&self, other: &Self) -> bool {
        self.profile_kind == other.profile_kind
            && self.flavor_id == other.flavor_id
            && self.edition_id == other.edition_id
            && self.interface == other.interface
            && self.client_version == other.client_version
            && self.client_build == other.client_build
            && self.source_kind == other.source_kind
            && self.source_revision == other.source_revision
            && self.source_logical_digest == other.source_logical_digest
            && self.builder_id == other.builder_id
            && self.builder_version == other.builder_version
            && self.schema_versions == other.schema_versions
            && self.correction_set_digest == other.correction_set_digest
            && self.fixture_scope == other.fixture_scope
    }

    fn differing_fields(&self, other: &Self) -> Vec<String> {
        let mut fields = Vec::new();
        macro_rules! compare_field {
            ($field:ident) => {
                if self.$field != other.$field {
                    fields.push(stringify!($field).to_owned());
                }
            };
        }
        compare_field!(profile_kind);
        compare_field!(flavor_id);
        compare_field!(edition_id);
        compare_field!(interface);
        compare_field!(client_version);
        compare_field!(client_build);
        compare_field!(source_kind);
        compare_field!(source_revision);
        compare_field!(source_logical_digest);
        compare_field!(builder_id);
        compare_field!(builder_version);
        compare_field!(schema_versions);
        compare_field!(correction_set_digest);
        compare_field!(fixture_scope);
        fields
    }
}

/// Exact profile comparison result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileComparison {
    Identical,
    SameLabelDifferentIdentity { differing_fields: Vec<String> },
    DifferentLabelSameMaterial,
    Different,
}

/// Builder that prevents invalid-empty `ProfileIdentity` construction.
#[derive(Debug, Clone)]
pub struct ProfileIdentityBuilder {
    profile_id: ProfileId,
    profile_kind: ProfileKind,
    flavor_id: String,
    edition_id: Option<String>,
    interface: u64,
    client_version: Option<ToolVersion>,
    client_build: Option<u64>,
    source_kind: SourceKind,
    source_revision: String,
    source_logical_digest: ContentDigest<SourceLogicalSnapshot>,
    builder_id: Option<ProducerId>,
    builder_version: Option<ToolVersion>,
    schema_versions: Vec<SchemaVersionEntry>,
    correction_set_digest: Option<ContentDigest<CorrectionSet>>,
    fixture_scope: Option<String>,
}

impl ProfileIdentityBuilder {
    /// Starts a profile with mandatory identity fields.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        profile_id: ProfileId,
        profile_kind: ProfileKind,
        flavor_id: impl Into<String>,
        interface: u64,
        source_kind: SourceKind,
        source_revision: impl Into<String>,
        source_logical_digest: ContentDigest<SourceLogicalSnapshot>,
    ) -> Self {
        Self {
            profile_id,
            profile_kind,
            flavor_id: flavor_id.into(),
            edition_id: None,
            interface,
            client_version: None,
            client_build: None,
            source_kind,
            source_revision: source_revision.into(),
            source_logical_digest,
            builder_id: None,
            builder_version: None,
            schema_versions: Vec::new(),
            correction_set_digest: None,
            fixture_scope: None,
        }
    }

    /// Adds an edition segment.
    #[must_use]
    pub fn edition_id(mut self, edition_id: impl Into<String>) -> Self {
        self.edition_id = Some(edition_id.into());
        self
    }

    /// Adds a client version.
    #[must_use]
    pub fn client_version(mut self, client_version: ToolVersion) -> Self {
        self.client_version = Some(client_version);
        self
    }

    /// Adds a client build.
    #[must_use]
    pub const fn client_build(mut self, client_build: u64) -> Self {
        self.client_build = Some(client_build);
        self
    }

    /// Adds the release builder identity.
    #[must_use]
    pub fn builder(mut self, builder_id: ProducerId, builder_version: ToolVersion) -> Self {
        self.builder_id = Some(builder_id);
        self.builder_version = Some(builder_version);
        self
    }

    /// Replaces schema entries.
    #[must_use]
    pub fn schema_versions(mut self, schema_versions: Vec<SchemaVersionEntry>) -> Self {
        self.schema_versions = schema_versions;
        self
    }

    /// Adds a correction-set digest.
    #[must_use]
    pub const fn correction_set_digest(
        mut self,
        correction_set_digest: ContentDigest<CorrectionSet>,
    ) -> Self {
        self.correction_set_digest = Some(correction_set_digest);
        self
    }

    /// Adds the explicit fixture capability boundary.
    #[must_use]
    pub fn fixture_scope(mut self, fixture_scope: impl Into<String>) -> Self {
        self.fixture_scope = Some(fixture_scope.into());
        self
    }

    /// Validates, sorts, and constructs the profile.
    pub fn build(mut self) -> CoreResult<ProfileIdentity> {
        self.schema_versions.sort();
        let profile = ProfileIdentity {
            profile_id: self.profile_id,
            profile_kind: self.profile_kind,
            flavor_id: self.flavor_id,
            edition_id: self.edition_id,
            interface: self.interface,
            client_version: self.client_version,
            client_build: self.client_build,
            source_kind: self.source_kind,
            source_revision: self.source_revision,
            source_logical_digest: self.source_logical_digest,
            builder_id: self.builder_id,
            builder_version: self.builder_version,
            schema_versions: self.schema_versions,
            correction_set_digest: self.correction_set_digest,
            fixture_scope: self.fixture_scope,
        };
        profile.validate()?;
        Ok(profile)
    }
}

fn validate_profile_fields(profile: &ProfileIdentity) -> CoreResult<()> {
    const OPERATION: &str = "validate_profile_identity";
    validate_lower_segment(&profile.flavor_id, OPERATION, "flavor_id")?;
    if let Some(edition_id) = &profile.edition_id {
        validate_lower_segment(edition_id, OPERATION, "edition_id")?;
    }
    if profile.interface == 0 {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidProfileIdentity,
            "interface",
        ));
    }
    if profile.source_revision.is_empty()
        || profile.source_revision.chars().any(char::is_control)
        || profile.source_revision.trim() != profile.source_revision
    {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidProfileIdentity,
            "source_revision",
        ));
    }
    if profile.schema_versions.is_empty() {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidProfileIdentity,
            "schema_versions",
        ));
    }
    for pair in profile.schema_versions.windows(2) {
        if pair[0].schema_id == pair[1].schema_id {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::DuplicateSchemaId,
                "schema_versions",
            ));
        }
        if pair[0] > pair[1] {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidProfileIdentity,
                "schema_versions",
            )
            .with_argument("reason", "noncanonical_order"));
        }
    }

    match profile.profile_kind {
        ProfileKind::Fixture => validate_fixture_profile(profile),
        ProfileKind::Release => validate_release_profile(profile),
    }
}

fn validate_fixture_profile(profile: &ProfileIdentity) -> CoreResult<()> {
    const OPERATION: &str = "validate_profile_identity";
    if profile.profile_id.namespace() != "fixture" {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::ProfileKindViolation,
            "profile_id.namespace",
        ));
    }
    if profile.fixture_scope.as_deref().is_none_or(str::is_empty) {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::ProfileKindViolation,
            "fixture_scope",
        ));
    }
    if profile.correction_set_digest.is_some() && profile.builder_id.is_none() {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidProfileIdentity,
            "builder_id",
        ));
    }
    Ok(())
}

fn validate_release_profile(profile: &ProfileIdentity) -> CoreResult<()> {
    const OPERATION: &str = "validate_profile_identity";
    if profile.profile_id.namespace() != "wow" {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::ProfileKindViolation,
            "profile_id.namespace",
        ));
    }
    if profile.source_kind == SourceKind::SyntheticFixture {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::ProfileKindViolation,
            "source_kind",
        ));
    }
    if profile.fixture_scope.is_some() {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::ProfileKindViolation,
            "fixture_scope",
        ));
    }
    if profile.client_version.is_none() {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidProfileIdentity,
            "client_version",
        ));
    }
    if profile.client_build.is_none_or(|value| value == 0) {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidProfileIdentity,
            "client_build",
        ));
    }
    if profile.builder_id.is_none() || profile.builder_version.is_none() {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidProfileIdentity,
            "builder_id",
        ));
    }
    if profile.correction_set_digest.is_none() {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidProfileIdentity,
            "correction_set_digest",
        ));
    }
    if is_floating_revision(&profile.source_revision) {
        return Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidProfileIdentity,
            "source_revision",
        ));
    }
    Ok(())
}

fn is_floating_revision(revision: &str) -> bool {
    matches!(
        revision.to_ascii_lowercase().as_str(),
        "main" | "master" | "current" | "latest" | "live" | "head" | "default" | "auto"
    )
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::digest::{ContentDigest, SourceLogicalSnapshot};
    use crate::ids::{ProfileId, SchemaId, ToolVersion};

    use super::{ProfileIdentityBuilder, ProfileKind, SchemaVersionEntry, SourceKind};

    fn digest() -> ContentDigest<SourceLogicalSnapshot> {
        ContentDigest::from_bytes([1_u8; 32])
    }

    fn schema() -> crate::CoreResult<SchemaVersionEntry> {
        Ok(SchemaVersionEntry::new(
            SchemaId::from_str("schema:wow:fixture-profile")?,
            ToolVersion::from_str("0.1.0")?,
        ))
    }

    #[test]
    fn fixture_profile_requires_scope() -> crate::CoreResult<()> {
        let profile_id = ProfileId::from_str("profile:fixture:e0-retail-120100")?;
        let builder = ProfileIdentityBuilder::new(
            profile_id,
            ProfileKind::Fixture,
            "retail",
            120_100,
            SourceKind::SyntheticFixture,
            "fixture:e0-rev1",
            digest(),
        )
        .schema_versions(vec![schema()?]);
        assert!(builder.build().is_err());
        Ok(())
    }

    #[test]
    fn valid_fixture_profile_builds() -> crate::CoreResult<()> {
        let profile_id = ProfileId::from_str("profile:fixture:e0-retail-120100")?;
        let profile = ProfileIdentityBuilder::new(
            profile_id,
            ProfileKind::Fixture,
            "retail",
            120_100,
            SourceKind::SyntheticFixture,
            "fixture:e0-rev1",
            digest(),
        )
        .schema_versions(vec![schema()?])
        .fixture_scope("e0 vertical slice only")
        .build();
        assert!(profile.is_ok());
        Ok(())
    }
}

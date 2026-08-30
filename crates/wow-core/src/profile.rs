use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    ContentDigest, CoreError, CoreErrorCode, CoreResult, FlavorId, ProfileId, SchemaVersion,
    ToolVersion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    Fixture,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ProfileIdentity {
    id: ProfileId,
    kind: ProfileKind,
    flavor: Option<FlavorId>,
    interface: Option<u32>,
    client_build: Option<String>,
    source_revision: Option<String>,
    source_digest: Option<ContentDigest>,
    builder_version: Option<ToolVersion>,
    schema_version: SchemaVersion,
    correction_set_digest: Option<ContentDigest>,
}

#[derive(Debug, Deserialize)]
struct ProfileIdentityWire {
    id: ProfileId,
    kind: ProfileKind,
    flavor: Option<FlavorId>,
    interface: Option<u32>,
    client_build: Option<String>,
    source_revision: Option<String>,
    source_digest: Option<ContentDigest>,
    builder_version: Option<ToolVersion>,
    schema_version: SchemaVersion,
    correction_set_digest: Option<ContentDigest>,
}

impl ProfileIdentity {
    pub fn fixture(id: ProfileId, schema_version: SchemaVersion) -> CoreResult<Self> {
        Self::new(ProfileIdentityWire {
            id,
            kind: ProfileKind::Fixture,
            flavor: None,
            interface: None,
            client_build: None,
            source_revision: None,
            source_digest: None,
            builder_version: None,
            schema_version,
            correction_set_digest: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn release(
        id: ProfileId,
        flavor: FlavorId,
        interface: u32,
        client_build: impl Into<String>,
        source_revision: impl Into<String>,
        source_digest: ContentDigest,
        builder_version: ToolVersion,
        schema_version: SchemaVersion,
        correction_set_digest: ContentDigest,
    ) -> CoreResult<Self> {
        Self::new(ProfileIdentityWire {
            id,
            kind: ProfileKind::Release,
            flavor: Some(flavor),
            interface: Some(interface),
            client_build: Some(client_build.into()),
            source_revision: Some(source_revision.into()),
            source_digest: Some(source_digest),
            builder_version: Some(builder_version),
            schema_version,
            correction_set_digest: Some(correction_set_digest),
        })
    }

    fn new(wire: ProfileIdentityWire) -> CoreResult<Self> {
        let identity = Self {
            id: wire.id,
            kind: wire.kind,
            flavor: wire.flavor,
            interface: wire.interface,
            client_build: wire.client_build,
            source_revision: wire.source_revision,
            source_digest: wire.source_digest,
            builder_version: wire.builder_version,
            schema_version: wire.schema_version,
            correction_set_digest: wire.correction_set_digest,
        };
        identity.validate()?;
        Ok(identity)
    }

    pub fn with_fixture_platform(
        mut self,
        flavor: FlavorId,
        interface: u32,
        client_build: impl Into<String>,
        source_revision: impl Into<String>,
        source_digest: ContentDigest,
    ) -> CoreResult<Self> {
        if self.kind != ProfileKind::Fixture {
            return Err(CoreError::new(
                CoreErrorCode::InvalidProfileIdentity,
                "with_fixture_platform",
                "only fixture profiles may be enriched by this operation",
            ));
        }
        self.flavor = Some(flavor);
        self.interface = Some(interface);
        self.client_build = Some(client_build.into());
        self.source_revision = Some(source_revision.into());
        self.source_digest = Some(source_digest);
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> CoreResult<()> {
        ProfileId::parse_exact(self.id.as_str())?;

        if matches!(self.interface, Some(0)) {
            return Err(self.invalid("Interface must be greater than zero"));
        }
        validate_optional_token("client_build", self.client_build.as_deref(), 96)?;
        validate_optional_token("source_revision", self.source_revision.as_deref(), 160)?;

        if self.source_revision.is_some() != self.source_digest.is_some() {
            return Err(self.invalid("source revision and source digest must be present together"));
        }

        if self.correction_set_digest.is_some() && self.source_digest.is_none() {
            return Err(self.invalid("a correction-set digest requires a source digest"));
        }

        if self.kind == ProfileKind::Release
            && (self.flavor.is_none()
                || self.interface.is_none()
                || self.client_build.is_none()
                || self.source_revision.is_none()
                || self.source_digest.is_none()
                || self.builder_version.is_none()
                || self.correction_set_digest.is_none())
        {
            return Err(self.invalid(
                "release profiles require flavor, Interface, build, source revision/digest, builder version, and correction-set digest",
            ));
        }
        Ok(())
    }

    fn invalid(&self, detail: impl Into<String>) -> CoreError {
        CoreError::new(
            CoreErrorCode::InvalidProfileIdentity,
            "validate_profile_identity",
            format!("profile {}: {}", self.id, detail.into()),
        )
    }

    #[must_use]
    pub fn id(&self) -> &ProfileId {
        &self.id
    }

    #[must_use]
    pub fn kind(&self) -> ProfileKind {
        self.kind
    }

    #[must_use]
    pub fn flavor(&self) -> Option<&FlavorId> {
        self.flavor.as_ref()
    }

    #[must_use]
    pub fn interface(&self) -> Option<u32> {
        self.interface
    }

    #[must_use]
    pub fn client_build(&self) -> Option<&str> {
        self.client_build.as_deref()
    }

    #[must_use]
    pub fn source_revision(&self) -> Option<&str> {
        self.source_revision.as_deref()
    }

    #[must_use]
    pub fn source_digest(&self) -> Option<ContentDigest> {
        self.source_digest
    }

    #[must_use]
    pub fn builder_version(&self) -> Option<&ToolVersion> {
        self.builder_version.as_ref()
    }

    #[must_use]
    pub fn schema_version(&self) -> &SchemaVersion {
        &self.schema_version
    }

    #[must_use]
    pub fn correction_set_digest(&self) -> Option<ContentDigest> {
        self.correction_set_digest
    }
}

fn validate_optional_token(
    field: &'static str,
    value: Option<&str>,
    maximum_length: usize,
) -> CoreResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty()
        || value.len() > maximum_length
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(CoreError::new(
            CoreErrorCode::InvalidProfileIdentity,
            "validate_profile_identity",
            format!("{field} is empty, oversized, or non-canonical"),
        ));
    }
    Ok(())
}

impl<'de> Deserialize<'de> for ProfileIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProfileIdentityWire::deserialize(deserializer)?;
        Self::new(wire).map_err(D::Error::custom)
    }
}

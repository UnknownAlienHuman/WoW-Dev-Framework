use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::{CoreError, CoreResult};

const MAX_CANONICAL_ID_LEN: usize = 160;
const MAX_ENTITY_KEY_LEN: usize = 512;

fn validate_canonical_id(kind: &'static str, value: &str) -> CoreResult<()> {
    if value.is_empty() {
        return Err(CoreError::invalid_identifier(kind, "value is empty"));
    }
    if value.len() > MAX_CANONICAL_ID_LEN {
        return Err(CoreError::invalid_identifier(kind, "value exceeds 160 bytes"));
    }
    if value.trim() != value {
        return Err(CoreError::invalid_identifier(
            kind,
            "surrounding whitespace is forbidden",
        ));
    }
    if !value.is_ascii() {
        return Err(CoreError::invalid_identifier(kind, "only ASCII is permitted"));
    }
    let bytes = value.as_bytes();
    if !bytes[0].is_ascii_alphanumeric() || !bytes[bytes.len() - 1].is_ascii_alphanumeric() {
        return Err(CoreError::invalid_identifier(
            kind,
            "first and last characters must be alphanumeric",
        ));
    }
    if value.chars().any(|character| {
        !(character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '.' | '-' | '_' | ':' | '/' | '@' | '+'))
    }) {
        return Err(CoreError::invalid_identifier(
            kind,
            "use lowercase ASCII alphanumerics and .-_:/@+ separators",
        ));
    }
    if value.contains("//") || value.contains("..") {
        return Err(CoreError::invalid_identifier(
            kind,
            "empty and parent-like segments are forbidden",
        ));
    }
    Ok(())
}

macro_rules! define_canonical_id {
    ($name:ident, $kind:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl AsRef<str>) -> CoreResult<Self> {
                let value = value.as_ref();
                validate_canonical_id($kind, value)?;
                Ok(Self(value.to_owned()))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            #[must_use]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl FromStr for $name {
            type Err = CoreError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::parse(value).map_err(D::Error::custom)
            }
        }
    };
}

define_canonical_id!(ProfileId, "profile_id");
define_canonical_id!(FlavorId, "flavor_id");
define_canonical_id!(ReferenceGenerationId, "reference_generation_id");
define_canonical_id!(ProjectGenerationId, "project_generation_id");
define_canonical_id!(ExternalGenerationId, "external_generation_id");
define_canonical_id!(RuleId, "rule_id");
define_canonical_id!(ProducerId, "producer_id");
define_canonical_id!(CapabilityId, "capability_id");
define_canonical_id!(CoveragePartitionId, "coverage_partition_id");
define_canonical_id!(StableHandleId, "stable_handle_id");
define_canonical_id!(SchemaVersion, "schema_version");
define_canonical_id!(ToolVersion, "tool_version");
define_canonical_id!(EvidenceId, "evidence_id");
define_canonical_id!(FindingKey, "finding_key");
define_canonical_id!(MessageKey, "message_key");
define_canonical_id!(OperationId, "operation_id");
define_canonical_id!(RootCauseKey, "root_cause_key");
define_canonical_id!(RevisionId, "revision_id");

impl ProfileId {
    pub fn parse_exact(value: impl AsRef<str>) -> CoreResult<Self> {
        let parsed = Self::parse(value)?;
        if matches!(
            parsed.as_str(),
            "current" | "latest" | "live" | "default" | "head" | "main"
        ) {
            return Err(CoreError::invalid_identifier(
                "profile_id",
                "floating profile aliases are forbidden",
            ));
        }
        Ok(parsed)
    }
}

impl RevisionId {
    pub fn parse_exact(value: impl AsRef<str>) -> CoreResult<Self> {
        let parsed = Self::parse(value)?;
        if matches!(parsed.as_str(), "head" | "main" | "master" | "latest") {
            return Err(CoreError::invalid_identifier(
                "revision_id",
                "floating revisions are forbidden in source identities",
            ));
        }
        Ok(parsed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepositoryId(String);

impl RepositoryId {
    pub fn parse(value: impl AsRef<str>) -> CoreResult<Self> {
        let canonical = value.as_ref().to_ascii_lowercase();
        if canonical.trim() != canonical || !canonical.is_ascii() {
            return Err(CoreError::invalid_identifier(
                "repository_id",
                "repository must be canonical ASCII without whitespace",
            ));
        }
        let mut parts = canonical.split('/');
        let owner = parts.next().unwrap_or_default();
        let repository = parts.next().unwrap_or_default();
        if owner.is_empty() || repository.is_empty() || parts.next().is_some() {
            return Err(CoreError::invalid_identifier(
                "repository_id",
                "expected exactly owner/repository",
            ));
        }
        for part in [owner, repository] {
            if part.len() > 100
                || !part.chars().all(|character| {
                    character.is_ascii_lowercase()
                        || character.is_ascii_digit()
                        || matches!(character, '.' | '-' | '_')
                })
            {
                return Err(CoreError::invalid_identifier(
                    "repository_id",
                    "invalid owner or repository component",
                ));
            }
        }
        Ok(Self(canonical))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for RepositoryId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RepositoryId {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for RepositoryId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RepositoryId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityKey(String);

impl EntityKey {
    pub fn parse(value: impl AsRef<str>) -> CoreResult<Self> {
        let value = value.as_ref();
        if value.is_empty() || value.len() > MAX_ENTITY_KEY_LEN || value.trim() != value {
            return Err(CoreError::invalid_identifier(
                "entity_key",
                "entity key must be 1..=512 bytes without surrounding whitespace",
            ));
        }
        if !value.is_ascii() || value.chars().any(char::is_control) {
            return Err(CoreError::invalid_identifier(
                "entity_key",
                "entity key must be printable ASCII",
            ));
        }
        if value.chars().any(char::is_whitespace) {
            return Err(CoreError::invalid_identifier(
                "entity_key",
                "whitespace is forbidden",
            ));
        }
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for EntityKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for EntityKey {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for EntityKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EntityKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentDigest([u8; 32]);

impl ContentDigest {
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
        let mut output = [0_u8; 32];
        output.copy_from_slice(&Sha256::digest(bytes.as_ref()));
        Self(output)
    }

    pub fn parse(value: impl AsRef<str>) -> CoreResult<Self> {
        let value = value.as_ref();
        let hex = value.strip_prefix("sha256:").ok_or_else(|| {
            CoreError::invalid_identifier("content_digest", "expected sha256:<64 hex characters>")
        })?;
        if hex.len() != 64 {
            return Err(CoreError::invalid_identifier(
                "content_digest",
                "SHA-256 digest must contain 64 hex characters",
            ));
        }
        let mut bytes = [0_u8; 32];
        for (index, chunk) in hex.as_bytes().chunks_exact(2).enumerate() {
            let pair = std::str::from_utf8(chunk).map_err(|_| {
                CoreError::invalid_identifier("content_digest", "digest is not ASCII hex")
            })?;
            bytes[index] = u8::from_str_radix(pair, 16).map_err(|_| {
                CoreError::invalid_identifier("content_digest", "digest contains non-hex data")
            })?;
        }
        Ok(Self(bytes))
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub fn canonical_string(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(71);
        output.push_str("sha256:");
        for byte in self.0 {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }
}

impl Display for ContentDigest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.canonical_string())
    }
}

impl FromStr for ContentDigest {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for ContentDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.canonical_string())
    }
}

impl<'de> Deserialize<'de> for ContentDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(D::Error::custom)
    }
}

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::canonical::domain_separated_digest;
use crate::error::{
    CoreError, CoreErrorCode, CoreResult, ErrorCategory, RetryClass, mismatch_error,
    unsupported_error, validation_error,
};
use crate::ids::{Parsed, validate_lower_segment};

/// Marker trait for field-specific digest purposes.
pub trait DigestPurpose:
    Clone + Copy + fmt::Debug + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash + 'static
{
    /// Stable purpose name used in diagnostics and typed comparisons.
    const NAME: &'static str;
}

macro_rules! digest_purpose {
    ($name:ident, $value:literal) => {
        #[doc = concat!("Digest purpose marker for `", $value, "`.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name;

        impl DigestPurpose for $name {
            const NAME: &'static str = $value;
        }
    };
}

digest_purpose!(SourceContent, "source_content");
digest_purpose!(SourceLogicalSnapshot, "source_logical_snapshot");
digest_purpose!(CorrectionSet, "correction_set");
digest_purpose!(CanonicalResult, "canonical_result");

/// Full SHA-256 digest carrying a compile-time purpose.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentDigest<P: DigestPurpose> {
    bytes: [u8; 32],
    marker: PhantomData<P>,
}

impl<P: DigestPurpose> ContentDigest<P> {
    /// Creates a digest from exact bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self {
            bytes,
            marker: PhantomData,
        }
    }

    /// Parses `sha256:<64 hex>`, accepting uppercase hex only as a
    /// noncanonical input suggestion.
    pub fn parse(candidate: &str) -> CoreResult<Parsed<Self>> {
        const OPERATION: &str = "parse_content_digest";
        let Some((algorithm, payload)) = candidate.split_once(':') else {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidDigest,
                "candidate",
            ));
        };

        if algorithm != "sha256" {
            return Err(unsupported_error(
                OPERATION,
                CoreErrorCode::UnsupportedDigestAlgorithm,
                "candidate.algorithm",
            )
            .with_argument("algorithm", algorithm));
        }

        if payload.len() != 64 || !payload.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(
                validation_error(OPERATION, CoreErrorCode::InvalidDigest, "candidate.hex")
                    .with_argument("length", payload.len().to_string()),
            );
        }

        let was_canonical = payload.bytes().all(|byte| !byte.is_ascii_uppercase());
        let bytes = decode_hex_32(payload).ok_or_else(|| {
            validation_error(OPERATION, CoreErrorCode::InvalidDigest, "candidate.hex")
        })?;

        Ok(Parsed::new(Self::from_bytes(bytes), was_canonical))
    }

    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Returns the stable purpose of this digest type.
    #[must_use]
    pub const fn purpose() -> &'static str {
        P::NAME
    }

    /// Returns the canonical `sha256:<hex>` representation.
    #[must_use]
    pub fn canonical(&self) -> String {
        format!("sha256:{}", encode_hex(&self.bytes))
    }

    /// Verifies equality with another digest of the same purpose.
    pub fn verify(&self, supplied: &Self) -> CoreResult<()> {
        if self == supplied {
            Ok(())
        } else {
            Err(mismatch_error(
                "verify_content_digest",
                CoreErrorCode::DigestMismatch,
                "digest",
            )
            .with_argument("purpose", P::NAME))
        }
    }
}

impl<P: DigestPurpose> fmt::Debug for ContentDigest<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ContentDigest")
            .field(&self.canonical())
            .field(&P::NAME)
            .finish()
    }
}

impl<P: DigestPurpose> fmt::Display for ContentDigest<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.canonical())
    }
}

impl<P: DigestPurpose> FromStr for ContentDigest<P> {
    type Err = CoreError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parsed = Self::parse(input)?;
        if parsed.was_canonical() {
            Ok(parsed.into_value())
        } else {
            Err(validation_error(
                "parse_content_digest",
                CoreErrorCode::InvalidDigest,
                "candidate",
            )
            .with_argument("reason", "noncanonical"))
        }
    }
}

impl<P: DigestPurpose> Serialize for ContentDigest<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.canonical())
    }
}

impl<'de, P: DigestPurpose> Deserialize<'de> for ContentDigest<P> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(D::Error::custom)
    }
}

macro_rules! fixed_digest_id {
    ($name:ident, $prefix:literal, $domain:literal) => {
        #[doc = concat!("Typed SHA-256 identifier with prefix `", $prefix, "`.")]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Derives the identifier from canonical domain-separated material.
            pub fn derive<T: Serialize + ?Sized>(value: &T) -> CoreResult<Self> {
                domain_separated_digest($domain, value).map(Self)
            }

            /// Creates the identifier from exact hash bytes.
            #[must_use]
            pub const fn from_hash(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            /// Returns exact hash bytes.
            #[must_use]
            pub const fn as_hash(&self) -> &[u8; 32] {
                &self.0
            }

            /// Returns the canonical string.
            #[must_use]
            pub fn canonical(&self) -> String {
                format!(concat!($prefix, "{}"), encode_hex(&self.0))
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.canonical())
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.canonical())
            }
        }

        impl FromStr for $name {
            type Err = CoreError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let payload = input.strip_prefix($prefix).ok_or_else(|| {
                    validation_error(
                        "parse_typed_digest_id",
                        CoreErrorCode::InvalidDigest,
                        "candidate",
                    )
                })?;
                let bytes = decode_hex_32(payload).ok_or_else(|| {
                    validation_error(
                        "parse_typed_digest_id",
                        CoreErrorCode::InvalidDigest,
                        "candidate.hex",
                    )
                })?;
                Ok(Self(bytes))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.canonical())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                value.parse().map_err(D::Error::custom)
            }
        }
    };
}

fixed_digest_id!(
    ReferenceGenerationId,
    "generation:reference:sha256:",
    "wow-core/reference-generation/e0-1"
);
fixed_digest_id!(
    ProjectGenerationId,
    "generation:project:sha256:",
    "wow-core/project-generation/e0-1"
);
fixed_digest_id!(
    StableHandleId,
    "handle:sha256:",
    "wow-core/source-handle/e0-1"
);
fixed_digest_id!(EvidenceId, "evidence:sha256:", "wow-core/evidence/e0-1");
fixed_digest_id!(ConflictId, "conflict:sha256:", "wow-core/conflict/e0-1");
fixed_digest_id!(CoverageId, "coverage:sha256:", "wow-core/coverage/e0-1");
fixed_digest_id!(
    FindingFingerprint,
    "finding-fingerprint:sha256:",
    "wow-core/finding-fingerprint/e0-1"
);
fixed_digest_id!(FindingId, "finding:sha256:", "wow-core/finding/e0-1");
fixed_digest_id!(
    GenerationContextId,
    "context:sha256:",
    "wow-core/generation-context/e0-1"
);
fixed_digest_id!(
    RootCauseKey,
    "root-cause:sha256:",
    "wow-core/root-cause/e0-1"
);
fixed_digest_id!(
    NotEvaluatedId,
    "not-evaluated:sha256:",
    "wow-core/not-evaluated/e0-1"
);
fixed_digest_id!(WarningId, "warning:sha256:", "wow-core/warning/e0-1");

/// Provider-scoped external generation identifier.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalGenerationId {
    provider: String,
    hash: [u8; 32],
}

impl ExternalGenerationId {
    /// Derives an external generation ID with the provider included in hash material.
    pub fn derive<T: Serialize + ?Sized>(provider: &str, value: &T) -> CoreResult<Self> {
        validate_lower_segment(provider, "derive_external_generation_id", "provider")?;

        #[derive(Serialize)]
        struct Material<'a, T: Serialize + ?Sized> {
            provider: &'a str,
            value: &'a T,
        }

        let hash = domain_separated_digest(
            "wow-core/external-generation/e0-1",
            &Material { provider, value },
        )?;
        Ok(Self {
            provider: provider.to_owned(),
            hash,
        })
    }

    /// Provider segment.
    #[must_use]
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// Canonical string.
    #[must_use]
    pub fn canonical(&self) -> String {
        format!(
            "generation:external:{}:sha256:{}",
            self.provider,
            encode_hex(&self.hash)
        )
    }
}

impl fmt::Debug for ExternalGenerationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ExternalGenerationId")
            .field(&self.canonical())
            .finish()
    }
}

impl fmt::Display for ExternalGenerationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.canonical())
    }
}

impl FromStr for ExternalGenerationId {
    type Err = CoreError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        const PREFIX: &str = "generation:external:";
        let rest = input.strip_prefix(PREFIX).ok_or_else(|| {
            validation_error(
                "parse_external_generation_id",
                CoreErrorCode::InvalidDigest,
                "candidate",
            )
        })?;
        let Some((provider, payload)) = rest.split_once(":sha256:") else {
            return Err(validation_error(
                "parse_external_generation_id",
                CoreErrorCode::InvalidDigest,
                "candidate",
            ));
        };
        validate_lower_segment(provider, "parse_external_generation_id", "provider")?;
        let hash = decode_hex_32(payload).ok_or_else(|| {
            validation_error(
                "parse_external_generation_id",
                CoreErrorCode::InvalidDigest,
                "candidate.hex",
            )
        })?;
        Ok(Self {
            provider: provider.to_owned(),
            hash,
        })
    }
}

impl Serialize for ExternalGenerationId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.canonical())
    }
}

impl<'de> Deserialize<'de> for ExternalGenerationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Returns a purpose mismatch without comparing unlike digest values.
#[must_use]
pub fn digest_purpose_mismatch(expected: &str, actual: &str) -> CoreError {
    CoreError::new(
        CoreErrorCode::DigestPurposeMismatch,
        ErrorCategory::Mismatch,
        "compare_content_digest",
        RetryClass::AfterInputChange,
    )
    .with_argument("expected", expected)
    .with_argument("actual", actual)
}

pub(crate) fn encode_hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(TABLE[usize::from(byte >> 4)]));
        output.push(char::from(TABLE[usize::from(byte & 0x0f)]));
    }
    output
}

fn decode_hex_32(input: &str) -> Option<[u8; 32]> {
    if input.len() != 64 {
        return None;
    }

    let bytes = input.as_bytes();
    let mut output = [0_u8; 32];
    for index in 0..32 {
        let high = decode_nibble(bytes[index * 2])?;
        let low = decode_nibble(bytes[index * 2 + 1])?;
        output[index] = (high << 4) | low;
    }
    Some(output)
}

const fn decode_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Parses one field-purpose-specific SHA-256 digest.
pub fn parse_content_digest<P: DigestPurpose>(
    candidate: &str,
) -> CoreResult<Parsed<ContentDigest<P>>> {
    ContentDigest::<P>::parse(candidate)
}

/// Hashes exact canonical material bytes and prepends a supported typed-ID family tag.
pub fn derive_typed_digest_id(family_tag: &str, canonical_material: &[u8]) -> CoreResult<String> {
    use sha2::{Digest as _, Sha256};

    let prefix = match family_tag {
        "reference_generation" => "generation:reference:sha256:",
        "project_generation" => "generation:project:sha256:",
        "source_handle" => "handle:sha256:",
        "evidence" => "evidence:sha256:",
        "conflict" => "conflict:sha256:",
        "coverage" => "coverage:sha256:",
        "finding_fingerprint" => "finding-fingerprint:sha256:",
        "finding" => "finding:sha256:",
        "generation_context" => "context:sha256:",
        "root_cause" => "root-cause:sha256:",
        "not_evaluated" => "not-evaluated:sha256:",
        "warning" => "warning:sha256:",
        _ => {
            return Err(unsupported_error(
                "derive_typed_digest_id",
                CoreErrorCode::UnsupportedIdentifierFamily,
                "family_tag",
            ));
        }
    };
    let hash = Sha256::digest(canonical_material);
    Ok(format!("{prefix}{}", encode_hex(&hash)))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{ContentDigest, EvidenceId, SourceContent};

    #[test]
    fn uppercase_digest_is_accepted_as_noncanonical() {
        let input = format!("sha256:{}", "AB".repeat(32));
        let parsed = ContentDigest::<SourceContent>::parse(&input);
        assert!(parsed.is_ok());
        let parsed = parsed.ok();
        assert_eq!(
            parsed.as_ref().map(|value| value.was_canonical()),
            Some(false)
        );
    }

    #[test]
    fn typed_id_requires_exact_prefix() {
        let valid = format!("evidence:sha256:{}", "00".repeat(32));
        assert!(EvidenceId::from_str(&valid).is_ok());
        assert!(EvidenceId::from_str(&format!("coverage:sha256:{}", "00".repeat(32))).is_err());
    }
}

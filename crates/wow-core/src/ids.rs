use std::fmt;
use std::str::FromStr;

use semver::Version;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{CoreError, CoreErrorCode, CoreResult, validation_error};

const RESERVED_SEGMENTS: &[&str] = &[
    "current", "latest", "live", "head", "default", "auto", "implicit", "unknown",
];

/// A parsed value plus whether the original text was already canonical.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed<T> {
    value: T,
    was_canonical: bool,
}

impl<T> Parsed<T> {
    /// Creates a parse result.
    #[must_use]
    pub const fn new(value: T, was_canonical: bool) -> Self {
        Self {
            value,
            was_canonical,
        }
    }

    /// Returns the parsed value by reference.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the result and returns the value.
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }

    /// Whether the original input was canonical.
    #[must_use]
    pub const fn was_canonical(&self) -> bool {
        self.was_canonical
    }
}

macro_rules! dotted_id {
    ($name:ident, $operation:literal) => {
        #[doc = concat!("Validated dotted identifier for `", stringify!($name), "`.")]
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Parses the family-specific dotted identifier.
            pub fn parse(candidate: &str) -> CoreResult<Parsed<Self>> {
                parse_dotted(candidate, $operation).map(|parsed| parsed.map(Self))
            }

            /// Canonical text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = CoreError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let parsed = Self::parse(input)?;
                require_canonical(parsed, $operation)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
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

trait ParsedMap<T> {
    fn map<U>(self, transform: impl FnOnce(T) -> U) -> Parsed<U>;
}

impl<T> ParsedMap<T> for Parsed<T> {
    fn map<U>(self, transform: impl FnOnce(T) -> U) -> Parsed<U> {
        Parsed::new(transform(self.value), self.was_canonical)
    }
}

dotted_id!(RuleId, "parse_rule_id");
dotted_id!(ProducerId, "parse_producer_id");
dotted_id!(CapabilityId, "parse_capability_id");
dotted_id!(OperationId, "parse_operation_id");
/// Validated diagnostic, warning, or reason code.
///
/// Public finding and warning codes may be dotted, while bounded reason codes
/// may be one canonical `snake_case` segment.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageCode(String);

impl MessageCode {
    /// Parses one canonical message code without requiring a dot.
    pub fn parse(candidate: &str) -> CoreResult<Parsed<Self>> {
        const OPERATION: &str = "parse_message_code";
        reject_hidden_text(candidate, OPERATION)?;
        if !candidate.is_ascii() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate",
            ));
        }
        let canonical = candidate.to_ascii_lowercase();
        validate_qualified_id(&canonical, false, OPERATION, "candidate")?;
        let was_canonical = canonical == candidate;
        Ok(Parsed::new(Self(canonical), was_canonical))
    }

    /// Canonical text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for MessageCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("MessageCode").field(&self.0).finish()
    }
}

impl fmt::Display for MessageCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for MessageCode {
    type Err = CoreError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parsed = Self::parse(input)?;
        require_canonical(parsed, "parse_message_code")
    }
}

impl Serialize for MessageCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for MessageCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Stable profile label. Structured profile fields remain authoritative.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileId {
    canonical: String,
    namespace: String,
    slug: String,
}

impl ProfileId {
    /// Parses `profile:<namespace>:<slug>`.
    pub fn parse(candidate: &str) -> CoreResult<Parsed<Self>> {
        const OPERATION: &str = "parse_profile_id";
        reject_hidden_text(candidate, OPERATION)?;
        if !candidate.is_ascii() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate",
            ));
        }

        let canonical = candidate.to_ascii_lowercase();
        let mut parts = canonical.split(':');
        if parts.next() != Some("profile") {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate.prefix",
            ));
        }
        let namespace = parts.next().ok_or_else(|| {
            validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate.namespace",
            )
        })?;
        let slug = parts.next().ok_or_else(|| {
            validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate.slug",
            )
        })?;
        if parts.next().is_some() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate",
            ));
        }

        validate_lower_segment(namespace, OPERATION, "candidate.namespace")?;
        validate_slug(slug, OPERATION, "candidate.slug")?;
        validate_reserved_components(slug, OPERATION, "candidate.slug")?;

        Ok(Parsed::new(
            Self {
                canonical: canonical.clone(),
                namespace: namespace.to_owned(),
                slug: slug.to_owned(),
            },
            canonical == candidate,
        ))
    }

    /// Namespace segment.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Slug segment.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Canonical text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }
}

impl fmt::Debug for ProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ProfileId")
            .field(&self.canonical)
            .finish()
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.canonical)
    }
}

impl FromStr for ProfileId {
    type Err = CoreError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        require_canonical(Self::parse(input)?, "parse_profile_id")
    }
}

impl Serialize for ProfileId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.canonical)
    }
}

impl<'de> Deserialize<'de> for ProfileId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Versioned schema identifier.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaId {
    canonical: String,
    namespace: String,
    slug: String,
}

impl SchemaId {
    /// Parses `schema:<namespace>:<slug>`.
    pub fn parse(candidate: &str) -> CoreResult<Parsed<Self>> {
        const OPERATION: &str = "parse_schema_id";
        reject_hidden_text(candidate, OPERATION)?;
        if !candidate.is_ascii() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate",
            ));
        }
        let canonical = candidate.to_ascii_lowercase();
        let mut parts = canonical.split(':');
        if parts.next() != Some("schema") {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate.prefix",
            ));
        }
        let namespace = parts.next().ok_or_else(|| {
            validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate.namespace",
            )
        })?;
        let slug = parts.next().ok_or_else(|| {
            validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate.slug",
            )
        })?;
        if parts.next().is_some() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidIdentifier,
                "candidate",
            ));
        }
        validate_lower_segment(namespace, OPERATION, "candidate.namespace")?;
        validate_slug(slug, OPERATION, "candidate.slug")?;
        validate_reserved_components(slug, OPERATION, "candidate.slug")?;

        Ok(Parsed::new(
            Self {
                canonical: canonical.clone(),
                namespace: namespace.to_owned(),
                slug: slug.to_owned(),
            },
            canonical == candidate,
        ))
    }

    /// Namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Slug.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Canonical text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }
}

impl fmt::Debug for SchemaId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("SchemaId")
            .field(&self.canonical)
            .finish()
    }
}

impl fmt::Display for SchemaId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.canonical)
    }
}

impl FromStr for SchemaId {
    type Err = CoreError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        require_canonical(Self::parse(input)?, "parse_schema_id")
    }
}

impl Serialize for SchemaId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.canonical)
    }
}

impl<'de> Deserialize<'de> for SchemaId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Canonical Semantic Version without build metadata.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ToolVersion(Version);

impl ToolVersion {
    /// Parses a canonical Semantic Version. Build metadata is rejected because
    /// it is not part of E0 identity.
    pub fn parse(candidate: &str) -> CoreResult<Self> {
        let version = Version::parse(candidate).map_err(|error| {
            validation_error(
                "parse_tool_version",
                CoreErrorCode::InvalidIdentifier,
                "candidate",
            )
            .with_argument("reason", error.to_string())
        })?;
        if !version.build.is_empty() {
            return Err(validation_error(
                "parse_tool_version",
                CoreErrorCode::InvalidIdentifier,
                "candidate.build",
            ));
        }
        if version.to_string() != candidate {
            return Err(validation_error(
                "parse_tool_version",
                CoreErrorCode::InvalidIdentifier,
                "candidate",
            )
            .with_argument("reason", "noncanonical"));
        }
        Ok(Self(version))
    }

    /// Underlying Semantic Version.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.0
    }
}

impl fmt::Debug for ToolVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("ToolVersion").field(&self.0).finish()
    }
}

impl fmt::Display for ToolVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for ToolVersion {
    type Err = CoreError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

/// Exact entity key with a validated qualified kind and UTF-8 payload.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityKey {
    kind: String,
    key: String,
}

impl EntityKey {
    /// Builds an exact entity key from its unencoded components.
    pub fn new(kind: &str, key: &str) -> CoreResult<Self> {
        validate_qualified_id(kind, false, "parse_entity_key", "kind")?;
        validate_exact_payload(key, "parse_entity_key", "key")?;
        Ok(Self {
            kind: kind.to_owned(),
            key: key.to_owned(),
        })
    }

    /// Qualified kind.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Exact decoded payload.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Canonical string with uppercase UTF-8 byte percent encoding.
    #[must_use]
    pub fn canonical(&self) -> String {
        format!("entity:{}:{}", self.kind, percent_encode(&self.key))
    }
}

impl fmt::Debug for EntityKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("EntityKey")
            .field(&self.canonical())
            .finish()
    }
}

impl fmt::Display for EntityKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.canonical())
    }
}

impl FromStr for EntityKey {
    type Err = CoreError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let rest = input.strip_prefix("entity:").ok_or_else(|| {
            validation_error(
                "parse_entity_key",
                CoreErrorCode::InvalidEntityKey,
                "candidate",
            )
        })?;
        let Some((kind, encoded)) = rest.split_once(':') else {
            return Err(validation_error(
                "parse_entity_key",
                CoreErrorCode::InvalidEntityKey,
                "candidate",
            ));
        };
        let key = percent_decode_canonical(encoded, "parse_entity_key", "candidate.key")?;
        let value = Self::new(kind, &key)?;
        if value.canonical() != input {
            return Err(validation_error(
                "parse_entity_key",
                CoreErrorCode::NoncanonicalPercentEncoding,
                "candidate.key",
            ));
        }
        Ok(value)
    }
}

impl Serialize for EntityKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.canonical())
    }
}

impl<'de> Deserialize<'de> for EntityKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Exact coverage partition key with optional payload.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoveragePartitionId {
    scope: String,
    key: Option<String>,
}

impl CoveragePartitionId {
    /// Builds a coverage partition ID from unencoded components.
    pub fn new(scope: &str, key: Option<&str>) -> CoreResult<Self> {
        validate_qualified_id(scope, false, "parse_coverage_partition_id", "scope")?;
        if let Some(value) = key {
            validate_exact_payload(value, "parse_coverage_partition_id", "key")?;
        }
        Ok(Self {
            scope: scope.to_owned(),
            key: key.map(str::to_owned),
        })
    }

    /// Scope.
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// Optional exact key.
    #[must_use]
    pub fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    /// Canonical string.
    #[must_use]
    pub fn canonical(&self) -> String {
        self.key.as_ref().map_or_else(
            || format!("partition:{}", self.scope),
            |key| format!("partition:{}:{}", self.scope, percent_encode(key)),
        )
    }
}

impl fmt::Debug for CoveragePartitionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CoveragePartitionId")
            .field(&self.canonical())
            .finish()
    }
}

impl fmt::Display for CoveragePartitionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.canonical())
    }
}

impl FromStr for CoveragePartitionId {
    type Err = CoreError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let rest = input.strip_prefix("partition:").ok_or_else(|| {
            validation_error(
                "parse_coverage_partition_id",
                CoreErrorCode::InvalidEntityKey,
                "candidate",
            )
        })?;
        let (scope, key) = match rest.split_once(':') {
            Some((scope, encoded)) => {
                if encoded.is_empty() {
                    return Err(validation_error(
                        "parse_coverage_partition_id",
                        CoreErrorCode::InvalidEntityKey,
                        "candidate.key",
                    ));
                }
                let decoded = percent_decode_canonical(
                    encoded,
                    "parse_coverage_partition_id",
                    "candidate.key",
                )?;
                (scope, Some(decoded))
            }
            None => (rest, None),
        };
        let value = Self::new(scope, key.as_deref())?;
        if value.canonical() != input {
            return Err(validation_error(
                "parse_coverage_partition_id",
                CoreErrorCode::NoncanonicalPercentEncoding,
                "candidate.key",
            ));
        }
        Ok(value)
    }
}

impl Serialize for CoveragePartitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.canonical())
    }
}

impl<'de> Deserialize<'de> for CoveragePartitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

fn parse_dotted(candidate: &str, operation: &'static str) -> CoreResult<Parsed<String>> {
    reject_hidden_text(candidate, operation)?;
    if !candidate.is_ascii() {
        return Err(validation_error(
            operation,
            CoreErrorCode::InvalidIdentifier,
            "candidate",
        ));
    }
    let canonical = candidate.to_ascii_lowercase();
    validate_qualified_id(&canonical, true, operation, "candidate")?;
    Ok(Parsed::new(canonical.clone(), canonical == candidate))
}

fn require_canonical<T>(parsed: Parsed<T>, operation: &'static str) -> CoreResult<T> {
    if parsed.was_canonical {
        Ok(parsed.value)
    } else {
        Err(
            validation_error(operation, CoreErrorCode::InvalidIdentifier, "candidate")
                .with_argument("reason", "noncanonical"),
        )
    }
}

pub(crate) fn validate_lower_segment(
    candidate: &str,
    operation: &'static str,
    field: &'static str,
) -> CoreResult<()> {
    let valid_length = (1..=63).contains(&candidate.len());
    let valid_first = candidate
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_lowercase);
    let valid_rest = candidate.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
    });

    if !valid_length {
        return Err(validation_error(
            operation,
            CoreErrorCode::IdentifierTooLong,
            field,
        ));
    }
    if !valid_first || !valid_rest {
        return Err(validation_error(
            operation,
            CoreErrorCode::InvalidIdentifier,
            field,
        ));
    }
    if is_reserved(candidate) {
        return Err(validation_error(
            operation,
            CoreErrorCode::ReservedIdentifierSegment,
            field,
        ));
    }
    Ok(())
}

fn validate_qualified_id(
    candidate: &str,
    require_multiple: bool,
    operation: &'static str,
    field: &'static str,
) -> CoreResult<()> {
    let segments = candidate.split('.').collect::<Vec<_>>();
    if require_multiple && segments.len() < 2 {
        return Err(validation_error(
            operation,
            CoreErrorCode::InvalidIdentifier,
            field,
        ));
    }
    if segments.is_empty() {
        return Err(validation_error(
            operation,
            CoreErrorCode::InvalidIdentifier,
            field,
        ));
    }
    for segment in segments {
        validate_dotted_segment(segment, operation, field)?;
    }
    Ok(())
}

fn validate_dotted_segment(
    candidate: &str,
    operation: &'static str,
    field: &'static str,
) -> CoreResult<()> {
    if candidate.is_empty() || candidate.len() > 63 {
        return Err(validation_error(
            operation,
            if candidate.len() > 63 {
                CoreErrorCode::IdentifierTooLong
            } else {
                CoreErrorCode::InvalidIdentifier
            },
            field,
        ));
    }
    let bytes = candidate.as_bytes();
    if !bytes[0].is_ascii_lowercase()
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
    {
        return Err(validation_error(
            operation,
            CoreErrorCode::InvalidIdentifier,
            field,
        ));
    }
    if is_reserved(candidate) {
        return Err(validation_error(
            operation,
            CoreErrorCode::ReservedIdentifierSegment,
            field,
        ));
    }
    Ok(())
}

fn validate_slug(candidate: &str, operation: &'static str, field: &'static str) -> CoreResult<()> {
    if candidate.is_empty() || candidate.len() > 96 {
        return Err(validation_error(
            operation,
            if candidate.len() > 96 {
                CoreErrorCode::IdentifierTooLong
            } else {
                CoreErrorCode::InvalidIdentifier
            },
            field,
        ));
    }
    let bytes = candidate.as_bytes();
    if !(bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        || !bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-')
        })
    {
        return Err(validation_error(
            operation,
            CoreErrorCode::InvalidIdentifier,
            field,
        ));
    }

    let mut previous_separator = false;
    for byte in bytes {
        let separator = matches!(*byte, b'.' | b'_' | b'-');
        if separator && previous_separator {
            return Err(validation_error(
                operation,
                CoreErrorCode::InvalidIdentifier,
                field,
            ));
        }
        previous_separator = separator;
    }
    if previous_separator {
        return Err(validation_error(
            operation,
            CoreErrorCode::InvalidIdentifier,
            field,
        ));
    }
    Ok(())
}

fn validate_reserved_components(
    candidate: &str,
    operation: &'static str,
    field: &'static str,
) -> CoreResult<()> {
    if candidate.split(['.', '_', '-']).any(is_reserved) {
        Err(validation_error(
            operation,
            CoreErrorCode::ReservedIdentifierSegment,
            field,
        ))
    } else {
        Ok(())
    }
}

fn reject_hidden_text(candidate: &str, operation: &'static str) -> CoreResult<()> {
    if candidate.is_empty()
        || candidate.trim() != candidate
        || candidate.chars().any(|character| character.is_control())
        || candidate.chars().any(char::is_whitespace)
    {
        Err(validation_error(
            operation,
            CoreErrorCode::InvalidIdentifier,
            "candidate",
        ))
    } else {
        Ok(())
    }
}

fn validate_exact_payload(
    candidate: &str,
    operation: &'static str,
    field: &'static str,
) -> CoreResult<()> {
    if candidate.is_empty() || candidate.chars().any(char::is_control) {
        Err(validation_error(
            operation,
            CoreErrorCode::InvalidEntityKey,
            field,
        ))
    } else {
        Ok(())
    }
}

fn is_reserved(candidate: &str) -> bool {
    RESERVED_SEGMENTS.contains(&candidate)
}

fn percent_encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    output
}

fn percent_decode_canonical(
    encoded: &str,
    operation: &'static str,
    field: &'static str,
) -> CoreResult<String> {
    let bytes = encoded.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            output.push(bytes[index]);
            index += 1;
            continue;
        }
        if index + 2 >= bytes.len() {
            return Err(validation_error(
                operation,
                CoreErrorCode::NoncanonicalPercentEncoding,
                field,
            ));
        }
        let high = decode_upper_hex(bytes[index + 1]).ok_or_else(|| {
            validation_error(operation, CoreErrorCode::NoncanonicalPercentEncoding, field)
        })?;
        let low = decode_upper_hex(bytes[index + 2]).ok_or_else(|| {
            validation_error(operation, CoreErrorCode::NoncanonicalPercentEncoding, field)
        })?;
        let decoded = (high << 4) | low;
        if decoded.is_ascii_alphanumeric() || matches!(decoded, b'-' | b'.' | b'_' | b'~') {
            return Err(validation_error(
                operation,
                CoreErrorCode::NoncanonicalPercentEncoding,
                field,
            ));
        }
        output.push(decoded);
        index += 3;
    }
    String::from_utf8(output)
        .map_err(|_| validation_error(operation, CoreErrorCode::InvalidEntityKey, field))
}

const fn decode_upper_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Parses a profile identifier while preserving canonicalization information.
pub fn parse_profile_id(candidate: &str) -> CoreResult<Parsed<ProfileId>> {
    ProfileId::parse(candidate)
}

/// Parses a rule identifier while preserving canonicalization information.
pub fn parse_rule_id(candidate: &str) -> CoreResult<Parsed<RuleId>> {
    RuleId::parse(candidate)
}

/// Parses a producer identifier while preserving canonicalization information.
pub fn parse_producer_id(candidate: &str) -> CoreResult<Parsed<ProducerId>> {
    ProducerId::parse(candidate)
}

/// Parses a capability identifier while preserving canonicalization information.
pub fn parse_capability_id(candidate: &str) -> CoreResult<Parsed<CapabilityId>> {
    CapabilityId::parse(candidate)
}

/// Parses an operation identifier while preserving canonicalization information.
pub fn parse_operation_id(candidate: &str) -> CoreResult<Parsed<OperationId>> {
    OperationId::parse(candidate)
}

/// Constructs an exact entity key from its kind and payload.
pub fn parse_entity_key(kind: &str, key: &str) -> CoreResult<EntityKey> {
    EntityKey::new(kind, key)
}

/// Constructs an exact coverage partition from its scope and optional payload.
pub fn parse_coverage_partition_id(
    scope: &str,
    key: Option<&str>,
) -> CoreResult<CoveragePartitionId> {
    CoveragePartitionId::new(scope, key)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{CoveragePartitionId, EntityKey, ProfileId, RuleId};

    #[test]
    fn profile_uppercase_is_noncanonical_but_parseable() {
        let parsed = ProfileId::parse("PROFILE:FIXTURE:E0-RETAIL-120100");
        assert!(parsed.is_ok());
        let parsed = parsed.ok();
        assert_eq!(
            parsed.as_ref().map(|value| value.was_canonical()),
            Some(false)
        );
        assert_eq!(
            parsed.as_ref().map(|value| value.value().as_str()),
            Some("profile:fixture:e0-retail-120100")
        );
    }

    #[test]
    fn dotted_id_requires_multiple_segments() {
        assert!(RuleId::from_str("wow.api.exists").is_ok());
        assert!(RuleId::from_str("single").is_err());
    }

    #[test]
    fn entity_key_preserves_case_and_encodes_slash() {
        let key = EntityKey::new("api", "C_UnitAuras/Thing");
        assert_eq!(
            key.as_ref().map(EntityKey::canonical),
            Ok("entity:api:C_UnitAuras%2FThing".to_owned())
        );
    }

    #[test]
    fn noncanonical_percent_encoding_is_rejected() {
        assert!(EntityKey::from_str("entity:api:Core%2fInit").is_err());
        assert!(EntityKey::from_str("entity:api:%41").is_err());
    }

    #[test]
    fn partition_without_key_has_no_trailing_colon() {
        let partition = CoveragePartitionId::new("project.file", None);
        assert_eq!(
            partition.as_ref().map(CoveragePartitionId::canonical),
            Ok("partition:project.file".to_owned())
        );
    }
}

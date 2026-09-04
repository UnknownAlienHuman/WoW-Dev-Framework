from pathlib import Path


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return source.replace(old, new, 1)


ids = Path("crates/wow-core/src/ids.rs")
source = ids.read_text(encoding="utf-8")
message_code_impl = '''/// Validated diagnostic, warning, or reason code.
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
'''
source = replace_once(
    source,
    'dotted_id!(MessageCode, "parse_message_code");',
    message_code_impl,
    "message code parser",
)
ids.write_text(source, encoding="utf-8")

finding = Path("crates/wow-core/src/finding.rs")
source = finding.read_text(encoding="utf-8")
for old, new, label in (
    (
        "identity_relevant_message_arguments: Vec<&'a MessageArgument>,",
        "identity_message_arguments: Vec<&'a MessageArgument>,",
        "fingerprint projection field",
    ),
    (
        "let identity_relevant_message_arguments = self",
        "let identity_message_arguments = self",
        "fingerprint projection local",
    ),
    (
        "            identity_relevant_message_arguments,",
        "            identity_message_arguments,",
        "fingerprint projection initializer",
    ),
    (
        "            fingerprint: FindingFingerprint,",
        "            finding_fingerprint: FindingFingerprint,",
        "finding identity field",
    ),
    (
        "            fingerprint,\n        })?;",
        "            finding_fingerprint: fingerprint,\n        })?;",
        "finding identity initializer",
    ),
):
    source = replace_once(source, old, new, label)
finding.write_text(source, encoding="utf-8")

integrity = Path("crates/wow-core/src/integrity.rs")
source = integrity.read_text(encoding="utf-8")
source = replace_once(
    source,
    '        "identity_relevant_message_arguments".to_owned(),',
    '        "identity_message_arguments".to_owned(),',
    "integrity fingerprint projection field",
)
source = replace_once(
    source,
    '        "fingerprint".to_owned(),\n        Value::String(supplied_fingerprint.to_owned()),',
    '        "finding_fingerprint".to_owned(),\n        Value::String(supplied_fingerprint.to_owned()),',
    "integrity finding identity field",
)
integrity.write_text(source, encoding="utf-8")

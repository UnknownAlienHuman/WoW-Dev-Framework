use std::fmt;

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::workspace::{EmmyBackendIdentity, EmmyWorkspaceError};

const REPORT_SCHEMA: &str = "wow-dev-framework/emmylua-compatibility-report";
const REPORT_SCHEMA_VERSION: u64 = 1;
const MAX_REPORT_BYTES: usize = 64 * 1024 * 1024;

/// Stable compatibility-report failure class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EmmyCompatibilityErrorCode {
    InputTooLarge,
    InvalidJson,
    UnsupportedSchema,
    InvalidDigest,
    InvalidSource,
    InvalidSurface,
    Incompatible,
    InvalidBackendIdentity,
}

/// One bounded compatibility-report import failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmmyCompatibilityError {
    code: EmmyCompatibilityErrorCode,
    message: Box<str>,
}

impl EmmyCompatibilityError {
    fn new(code: EmmyCompatibilityErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Stable failure class.
    #[must_use]
    pub const fn code(&self) -> EmmyCompatibilityErrorCode {
        self.code
    }

    /// Safe explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for EmmyCompatibilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EmmyCompatibilityError {}

/// Result type for compatibility-report imports.
pub type EmmyCompatibilityResult<T> = Result<T, EmmyCompatibilityError>;

/// Imports one exact analyzer identity from a verified rolling compatibility report.
pub fn backend_identity_from_report(
    bytes: &[u8],
) -> EmmyCompatibilityResult<EmmyBackendIdentity> {
    if bytes.len() > MAX_REPORT_BYTES {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InputTooLarge,
            "EmmyLua compatibility report exceeds the import limit",
        ));
    }
    let value = serde_json::from_slice::<Value>(bytes).map_err(|source| {
        report_error(
            EmmyCompatibilityErrorCode::InvalidJson,
            format!("EmmyLua compatibility report is not valid JSON: {source}"),
        )
    })?;
    let root = object(&value, "compatibility report")?;
    allowed_keys(
        root,
        &[
            "schema",
            "schema_version",
            "producer_version",
            "source",
            "workspace",
            "analysis_crate",
            "surface",
            "compatibility",
            "limitations",
            "report_sha256",
        ],
        "compatibility report",
    )?;
    if text(root, "schema", "report schema")? != REPORT_SCHEMA
        || unsigned(root, "schema_version", "report schema version")? != REPORT_SCHEMA_VERSION
    {
        return Err(report_error(
            EmmyCompatibilityErrorCode::UnsupportedSchema,
            "unsupported EmmyLua compatibility report schema",
        ));
    }
    let producer_version = unsigned(root, "producer_version", "producer version")?;
    if producer_version == 0 {
        return Err(report_error(
            EmmyCompatibilityErrorCode::UnsupportedSchema,
            "EmmyLua compatibility report producer version must be positive",
        ));
    }

    let supplied_digest = text(root, "report_sha256", "report digest")?.to_owned();
    canonical_sha256(&supplied_digest, "report digest")?;
    let mut projection = value.clone();
    object_mut(&mut projection, "compatibility report projection")?.remove("report_sha256");
    let canonical = canonical_json_bytes(&projection).map_err(|source| {
        report_error(
            EmmyCompatibilityErrorCode::InvalidDigest,
            format!("compatibility report cannot be canonicalized: {source}"),
        )
    })?;
    if supplied_digest != sha256(&canonical) {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InvalidDigest,
            "compatibility report digest does not match content",
        ));
    }

    let compatibility = object(
        required(root, "compatibility", "compatibility result")?,
        "compatibility result",
    )?;
    allowed_keys(
        compatibility,
        &["required_symbols", "missing_symbols", "status"],
        "compatibility result",
    )?;
    let required_symbols = string_array(
        required(compatibility, "required_symbols", "required symbols")?,
        "required symbols",
    )?;
    let missing_symbols = string_array(
        required(compatibility, "missing_symbols", "missing symbols")?,
        "missing symbols",
    )?;
    if !strictly_sorted_unique(&required_symbols) || !strictly_sorted_unique(&missing_symbols) {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InvalidJson,
            "compatibility symbol lists are not uniquely byte-sorted",
        ));
    }
    if text(compatibility, "status", "compatibility status")? != "compatible"
        || !missing_symbols.is_empty()
    {
        return Err(report_error(
            EmmyCompatibilityErrorCode::Incompatible,
            "EmmyLua compatibility report does not authorize this backend",
        ));
    }

    let source = object(required(root, "source", "source identity")?, "source identity")?;
    allowed_keys(
        source,
        &[
            "branch",
            "revision",
            "tree",
            "relation",
            "remote_head",
            "network_checked",
        ],
        "source identity",
    )?;
    let revision = text(source, "revision", "source revision")?.to_owned();
    let tree = text(source, "tree", "source tree")?.to_owned();
    validate_object_id(&revision, None, "source revision")?;
    validate_object_id(&tree, Some(revision.len()), "source tree")?;
    let relation = text(source, "relation", "source relation")?;
    if !matches!(relation, "current" | "unverified_current") {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InvalidSource,
            "compatibility report source is stale, diverged, or unsafe",
        ));
    }
    if source
        .get("network_checked")
        .and_then(Value::as_bool)
        .is_none()
    {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InvalidSource,
            "compatibility report network-check state is invalid",
        ));
    }
    if relation == "current" {
        let remote_head = text(source, "remote_head", "current remote head")?;
        if remote_head != revision {
            return Err(report_error(
                EmmyCompatibilityErrorCode::InvalidSource,
                "current compatibility report does not match its remote head",
            ));
        }
    }

    let analysis_crate = object(
        required(root, "analysis_crate", "analysis crate")?,
        "analysis crate",
    )?;
    allowed_keys(
        analysis_crate,
        &[
            "manifest_path",
            "name",
            "version",
            "edition",
            "rust_version",
            "license",
            "features",
        ],
        "analysis crate",
    )?;
    let crate_name = text(analysis_crate, "name", "analysis crate name")?.to_owned();
    let crate_version = optional_scalar_text(analysis_crate, "version")?;
    let manifest_path = text(
        analysis_crate,
        "manifest_path",
        "analysis crate manifest path",
    )?;
    if manifest_path.is_empty()
        || manifest_path.starts_with('/')
        || manifest_path.contains('\\')
        || manifest_path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || !manifest_path.ends_with("/Cargo.toml")
    {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InvalidSource,
            "analysis crate manifest path is noncanonical",
        ));
    }

    let surface = object(required(root, "surface", "public surface")?, "public surface")?;
    allowed_keys(
        surface,
        &["files", "symbols", "surface_sha256"],
        "public surface",
    )?;
    let surface_sha256 = text(surface, "surface_sha256", "surface digest")?.to_owned();
    canonical_sha256(&surface_sha256, "surface digest")?;
    if !matches!(surface.get("files"), Some(Value::Array(_)))
        || !matches!(surface.get("symbols"), Some(Value::Array(_)))
    {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InvalidSurface,
            "compatibility report public surface collections are invalid",
        ));
    }

    EmmyBackendIdentity::new(
        crate_name,
        crate_version,
        revision,
        tree,
        surface_sha256,
        supplied_digest,
    )
    .map_err(map_backend_error)
}

fn map_backend_error(error: EmmyWorkspaceError) -> EmmyCompatibilityError {
    report_error(
        EmmyCompatibilityErrorCode::InvalidBackendIdentity,
        format!("compatibility report backend identity is invalid: {error}"),
    )
}

fn allowed_keys(
    object: &Map<String, Value>,
    allowed: &[&str],
    label: &str,
) -> EmmyCompatibilityResult<()> {
    if let Some(unexpected) = object
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
    {
        return Err(report_error(
            EmmyCompatibilityErrorCode::UnsupportedSchema,
            format!("{label} contains unsupported field {unexpected:?}"),
        ));
    }
    Ok(())
}

fn object<'a>(
    value: &'a Value,
    label: &str,
) -> EmmyCompatibilityResult<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| {
        report_error(
            EmmyCompatibilityErrorCode::InvalidJson,
            format!("{label} must be an object"),
        )
    })
}

fn object_mut<'a>(
    value: &'a mut Value,
    label: &str,
) -> EmmyCompatibilityResult<&'a mut Map<String, Value>> {
    value.as_object_mut().ok_or_else(|| {
        report_error(
            EmmyCompatibilityErrorCode::InvalidJson,
            format!("{label} must be an object"),
        )
    })
}

fn required<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> EmmyCompatibilityResult<&'a Value> {
    object.get(key).ok_or_else(|| {
        report_error(
            EmmyCompatibilityErrorCode::InvalidJson,
            format!("{label} is missing"),
        )
    })
}

fn text<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> EmmyCompatibilityResult<&'a str> {
    required(object, key, label)?
        .as_str()
        .ok_or_else(|| {
            report_error(
                EmmyCompatibilityErrorCode::InvalidJson,
                format!("{label} must be text"),
            )
        })
}

fn optional_scalar_text(
    object: &Map<String, Value>,
    key: &str,
) -> EmmyCompatibilityResult<Option<String>> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.is_empty() => Ok(Some(value.clone())),
        Some(_) => Err(report_error(
            EmmyCompatibilityErrorCode::InvalidJson,
            format!("optional field {key:?} must be nonempty text or null"),
        )),
    }
}

fn unsigned(
    object: &Map<String, Value>,
    key: &str,
    label: &str,
) -> EmmyCompatibilityResult<u64> {
    required(object, key, label)?.as_u64().ok_or_else(|| {
        report_error(
            EmmyCompatibilityErrorCode::InvalidJson,
            format!("{label} must be an unsigned integer"),
        )
    })
}

fn string_array(value: &Value, label: &str) -> EmmyCompatibilityResult<Vec<String>> {
    let values = value.as_array().ok_or_else(|| {
        report_error(
            EmmyCompatibilityErrorCode::InvalidJson,
            format!("{label} must be an array"),
        )
    })?;
    values
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                report_error(
                    EmmyCompatibilityErrorCode::InvalidJson,
                    format!("{label} contains a non-text value"),
                )
            })
        })
        .collect()
}

fn strictly_sorted_unique(values: &[String]) -> bool {
    values
        .windows(2)
        .all(|pair| pair[0].as_bytes() < pair[1].as_bytes())
}

fn validate_object_id(
    value: &str,
    expected_length: Option<usize>,
    label: &str,
) -> EmmyCompatibilityResult<()> {
    if !matches!(value.len(), 40 | 64)
        || expected_length.is_some_and(|length| length != value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InvalidSource,
            format!("{label} is not a canonical Git object identifier"),
        ));
    }
    Ok(())
}

fn canonical_sha256(value: &str, label: &str) -> EmmyCompatibilityResult<()> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InvalidDigest,
            format!("{label} is not a canonical SHA-256 identifier"),
        ));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(report_error(
            EmmyCompatibilityErrorCode::InvalidDigest,
            format!("{label} is not a SHA-256 digest"),
        ));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}

fn report_error(
    code: EmmyCompatibilityErrorCode,
    message: impl Into<Box<str>>,
) -> EmmyCompatibilityError {
    EmmyCompatibilityError::new(code, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn report(relation: &str) -> Value {
        json!({
            "schema": REPORT_SCHEMA,
            "schema_version": REPORT_SCHEMA_VERSION,
            "producer_version": 1,
            "source": {
                "branch": "main",
                "revision": "1111111111111111111111111111111111111111",
                "tree": "2222222222222222222222222222222222222222",
                "relation": relation,
                "remote_head": if relation == "current" {
                    Value::String("1111111111111111111111111111111111111111".to_owned())
                } else {
                    Value::Null
                },
                "network_checked": relation == "current"
            },
            "workspace": {
                "resolver": "2",
                "edition": "2024",
                "rust_version": "1.85",
                "license": "MIT"
            },
            "analysis_crate": {
                "manifest_path": "crates/emmylua_code_analysis/Cargo.toml",
                "name": "emmylua_code_analysis",
                "version": "0.21.0",
                "edition": "2024",
                "rust_version": "1.85",
                "license": "MIT",
                "features": []
            },
            "surface": {
                "files": [],
                "symbols": [],
                "surface_sha256": "sha256:3333333333333333333333333333333333333333333333333333333333333333"
            },
            "compatibility": {
                "required_symbols": [],
                "missing_symbols": [],
                "status": "compatible"
            },
            "limitations": ["compile fixtures remain authoritative"]
        })
    }

    fn seal(mut value: Value) -> EmmyCompatibilityResult<Vec<u8>> {
        let bytes = canonical_json_bytes(&value).map_err(|source| {
            report_error(
                EmmyCompatibilityErrorCode::InvalidDigest,
                format!("test report cannot be canonicalized: {source}"),
            )
        })?;
        object_mut(&mut value, "test report")?.insert(
            "report_sha256".to_owned(),
            Value::String(sha256(&bytes)),
        );
        serde_json::to_vec(&value).map_err(|source| {
            report_error(
                EmmyCompatibilityErrorCode::InvalidJson,
                format!("test report cannot be serialized: {source}"),
            )
        })
    }

    #[test]
    fn current_report_yields_exact_backend_identity() -> EmmyCompatibilityResult<()> {
        let identity = backend_identity_from_report(&seal(report("current"))?)?;
        assert_eq!(identity.crate_name(), "emmylua_code_analysis");
        assert_eq!(identity.crate_version(), Some("0.21.0"));
        assert_eq!(
            identity.revision(),
            "1111111111111111111111111111111111111111"
        );
        assert_eq!(
            identity.surface_sha256(),
            "sha256:3333333333333333333333333333333333333333333333333333333333333333"
        );
        Ok(())
    }

    #[test]
    fn exact_offline_report_remains_usable_without_freshness_claim() -> EmmyCompatibilityResult<()> {
        let identity = backend_identity_from_report(&seal(report("unverified_current"))?)?;
        assert_eq!(identity.tree(), "2222222222222222222222222222222222222222");
        Ok(())
    }

    #[test]
    fn stale_report_is_rejected() -> EmmyCompatibilityResult<()> {
        let error = backend_identity_from_report(&seal(report("behind"))?)
            .err()
            .ok_or_else(|| {
                report_error(
                    EmmyCompatibilityErrorCode::InvalidSource,
                    "stale report unexpectedly imported",
                )
            })?;
        assert_eq!(error.code(), EmmyCompatibilityErrorCode::InvalidSource);
        Ok(())
    }

    #[test]
    fn missing_required_symbols_are_rejected() -> EmmyCompatibilityResult<()> {
        let mut value = report("current");
        let compatibility = object_mut(
            object_mut(&mut value, "test report")?
                .get_mut("compatibility")
                .ok_or_else(|| {
                    report_error(
                        EmmyCompatibilityErrorCode::InvalidJson,
                        "test compatibility is missing",
                    )
                })?,
            "test compatibility",
        )?;
        compatibility.insert(
            "missing_symbols".to_owned(),
            json!(["RequiredAdapterSeam"]),
        );
        compatibility.insert("status".to_owned(), Value::String("incompatible".to_owned()));
        let error = backend_identity_from_report(&seal(value)?)
            .err()
            .ok_or_else(|| {
                report_error(
                    EmmyCompatibilityErrorCode::Incompatible,
                    "incompatible report unexpectedly imported",
                )
            })?;
        assert_eq!(error.code(), EmmyCompatibilityErrorCode::Incompatible);
        Ok(())
    }

    #[test]
    fn tampering_breaks_report_identity() -> EmmyCompatibilityResult<()> {
        let bytes = seal(report("current"))?;
        let mut value = serde_json::from_slice::<Value>(&bytes).map_err(|source| {
            report_error(
                EmmyCompatibilityErrorCode::InvalidJson,
                format!("test report cannot be parsed: {source}"),
            )
        })?;
        object_mut(&mut value, "test report")?.insert(
            "producer_version".to_owned(),
            Value::from(2_u64),
        );
        let tampered = serde_json::to_vec(&value).map_err(|source| {
            report_error(
                EmmyCompatibilityErrorCode::InvalidJson,
                format!("test report cannot be serialized: {source}"),
            )
        })?;
        let error = backend_identity_from_report(&tampered)
            .err()
            .ok_or_else(|| {
                report_error(
                    EmmyCompatibilityErrorCode::InvalidDigest,
                    "tampered report unexpectedly imported",
                )
            })?;
        assert_eq!(error.code(), EmmyCompatibilityErrorCode::InvalidDigest);
        Ok(())
    }
}

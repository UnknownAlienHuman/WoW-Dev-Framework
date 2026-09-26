//! Pure, deterministic annotation artifact identities and publication selectors.
//!
//! This module owns canonical artifact bytes and semantic identity only. Durable
//! objects, catalogs, operations, leases, retention and garbage collection are
//! coordinated by `wow-service`; `wow-annotations` has no storage dependency.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::native::NativeLibrary;

pub const ANNOTATION_ARTIFACT_SCHEMA: &str = "wow-annotations/artifact/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationArtifactErrorCode {
    ArtifactIdentityInvalid,
    ArtifactPayloadInvalid,
    ArtifactSchemaMismatch,
    PublicationKeyInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationArtifactError {
    code: AnnotationArtifactErrorCode,
    message: Box<str>,
}

impl AnnotationArtifactError {
    fn new(code: AnnotationArtifactErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> AnnotationArtifactErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AnnotationArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AnnotationArtifactError {}

pub type AnnotationArtifactResult<T> = Result<T, AnnotationArtifactError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationArtifact {
    schema: Box<str>,
    producer_id: Box<str>,
    producer_version: Box<str>,
    profile_id: Box<str>,
    source_generation_id: Box<str>,
    payload_sha256: Box<str>,
    canonical_payload: Box<[u8]>,
    artifact_id: Box<str>,
}

impl AnnotationArtifact {
    pub fn from_native_library(
        producer_version: impl Into<Box<str>>,
        profile_id: impl Into<Box<str>>,
        source_generation_id: impl Into<Box<str>>,
        library: &NativeLibrary<'_>,
    ) -> AnnotationArtifactResult<Self> {
        Self::from_serializable(
            "wow-annotations",
            producer_version,
            profile_id,
            source_generation_id,
            library,
        )
    }

    pub fn from_serializable<T: Serialize + ?Sized>(
        producer_id: impl Into<Box<str>>,
        producer_version: impl Into<Box<str>>,
        profile_id: impl Into<Box<str>>,
        source_generation_id: impl Into<Box<str>>,
        value: &T,
    ) -> AnnotationArtifactResult<Self> {
        let producer_id = producer_id.into();
        let producer_version = producer_version.into();
        let profile_id = profile_id.into();
        let source_generation_id = source_generation_id.into();
        for value in [
            producer_id.as_ref(),
            producer_version.as_ref(),
            profile_id.as_ref(),
            source_generation_id.as_ref(),
        ] {
            validate_identity_component(value)?;
        }
        let canonical_payload = canonical_json_bytes(value).map_err(|_| {
            AnnotationArtifactError::new(
                AnnotationArtifactErrorCode::ArtifactPayloadInvalid,
                "annotation payload cannot be canonicalized",
            )
        })?;
        serde_json::from_slice::<serde_json::Value>(&canonical_payload).map_err(|_| {
            AnnotationArtifactError::new(
                AnnotationArtifactErrorCode::ArtifactPayloadInvalid,
                "annotation payload is not valid canonical JSON",
            )
        })?;
        let payload_sha256 =
            format!("sha256:{}", hex(&Sha256::digest(&canonical_payload))).into_boxed_str();
        let artifact_id = artifact_id(
            &producer_id,
            &producer_version,
            &profile_id,
            &source_generation_id,
            &payload_sha256,
        )?;
        Ok(Self {
            schema: ANNOTATION_ARTIFACT_SCHEMA.into(),
            producer_id,
            producer_version,
            profile_id,
            source_generation_id,
            payload_sha256,
            canonical_payload: canonical_payload.into_boxed_slice(),
            artifact_id,
        })
    }

    pub fn validate(&self) -> AnnotationArtifactResult<()> {
        if self.schema.as_ref() != ANNOTATION_ARTIFACT_SCHEMA {
            return Err(AnnotationArtifactError::new(
                AnnotationArtifactErrorCode::ArtifactSchemaMismatch,
                "annotation artifact schema is unsupported",
            ));
        }
        for value in [
            self.producer_id.as_ref(),
            self.producer_version.as_ref(),
            self.profile_id.as_ref(),
            self.source_generation_id.as_ref(),
        ] {
            validate_identity_component(value)?;
        }
        serde_json::from_slice::<serde_json::Value>(&self.canonical_payload).map_err(|_| {
            AnnotationArtifactError::new(
                AnnotationArtifactErrorCode::ArtifactPayloadInvalid,
                "annotation artifact payload is invalid JSON",
            )
        })?;
        let payload_sha256 = format!("sha256:{}", hex(&Sha256::digest(&self.canonical_payload)));
        if self.payload_sha256.as_ref() != payload_sha256 {
            return Err(AnnotationArtifactError::new(
                AnnotationArtifactErrorCode::ArtifactPayloadInvalid,
                "annotation artifact payload digest does not match",
            ));
        }
        let expected = artifact_id(
            &self.producer_id,
            &self.producer_version,
            &self.profile_id,
            &self.source_generation_id,
            &self.payload_sha256,
        )?;
        if self.artifact_id != expected {
            return Err(AnnotationArtifactError::new(
                AnnotationArtifactErrorCode::ArtifactIdentityInvalid,
                "annotation artifact identity does not match its inputs",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn producer_id(&self) -> &str {
        &self.producer_id
    }

    #[must_use]
    pub fn producer_version(&self) -> &str {
        &self.producer_version
    }

    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub fn source_generation_id(&self) -> &str {
        &self.source_generation_id
    }

    #[must_use]
    pub fn payload_sha256(&self) -> &str {
        &self.payload_sha256
    }

    #[must_use]
    pub fn canonical_payload(&self) -> &[u8] {
        &self.canonical_payload
    }
}

/// Store-neutral selector for one current annotation artifact.
///
/// `catalog_path` deliberately remains a validated string so the serialized key
/// is byte-compatible with the former transparent `wow_store::CatalogPath` field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationPublicationKey {
    profile: Box<str>,
    environment: Box<str>,
    artifact_family: Box<str>,
    catalog_path: Box<str>,
}

impl AnnotationPublicationKey {
    pub fn new(
        profile: impl Into<Box<str>>,
        environment: impl Into<Box<str>>,
        artifact_family: impl Into<Box<str>>,
    ) -> AnnotationArtifactResult<Self> {
        let profile = profile.into();
        let environment = environment.into();
        let artifact_family = artifact_family.into();
        for value in [
            profile.as_ref(),
            environment.as_ref(),
            artifact_family.as_ref(),
        ] {
            validate_publication_component(value)?;
        }
        let catalog_path = format!("{profile}/{environment}/{artifact_family}").into_boxed_str();
        Ok(Self {
            profile,
            environment,
            artifact_family,
            catalog_path,
        })
    }

    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }

    #[must_use]
    pub fn artifact_family(&self) -> &str {
        &self.artifact_family
    }

    #[must_use]
    pub fn catalog_path(&self) -> &str {
        &self.catalog_path
    }
}

fn artifact_id(
    producer_id: &str,
    producer_version: &str,
    profile_id: &str,
    source_generation_id: &str,
    payload_sha256: &str,
) -> AnnotationArtifactResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        producer_id: &'a str,
        producer_version: &'a str,
        profile_id: &'a str,
        source_generation_id: &'a str,
        payload_sha256: &'a str,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: ANNOTATION_ARTIFACT_SCHEMA,
        producer_id,
        producer_version,
        profile_id,
        source_generation_id,
        payload_sha256,
    })
    .map_err(|_| {
        AnnotationArtifactError::new(
            AnnotationArtifactErrorCode::ArtifactIdentityInvalid,
            "annotation artifact identity cannot be canonicalized",
        )
    })?;
    Ok(format!("annotation-artifact:sha256:{}", hex(&Sha256::digest(bytes))).into())
}

fn validate_identity_component(value: &str) -> AnnotationArtifactResult<()> {
    if !valid_component(value, 512) {
        return Err(AnnotationArtifactError::new(
            AnnotationArtifactErrorCode::ArtifactIdentityInvalid,
            "annotation identity component is invalid",
        ));
    }
    Ok(())
}

fn validate_publication_component(value: &str) -> AnnotationArtifactResult<()> {
    if !valid_component(value, 512) {
        return Err(AnnotationArtifactError::new(
            AnnotationArtifactErrorCode::PublicationKeyInvalid,
            "annotation publication key component is invalid",
        ));
    }
    Ok(())
}

fn valid_component(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
        })
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn artifact(value: serde_json::Value) -> AnnotationArtifactResult<AnnotationArtifact> {
        AnnotationArtifact::from_serializable(
            "wow-annotations",
            "1.0.0",
            "retail-12.1",
            "reference-generation:fixture",
            &value,
        )
    }

    #[test]
    fn canonical_payload_and_inputs_define_artifact_identity()
    -> Result<(), Box<dyn std::error::Error>> {
        let left = artifact(json!({"zeta":2,"alpha":1}))?;
        let right = artifact(json!({"alpha":1,"zeta":2}))?;
        assert_eq!(left, right);
        assert!(
            left.artifact_id()
                .starts_with("annotation-artifact:sha256:")
        );
        left.validate()?;
        Ok(())
    }

    #[test]
    fn tampered_artifact_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
        let mut value = serde_json::to_value(artifact(json!({"value":1}))?)?;
        value["payload_sha256"] = json!(format!("sha256:{}", "0".repeat(64)));
        let tampered: AnnotationArtifact = serde_json::from_value(value)?;
        assert_eq!(
            tampered
                .validate()
                .err()
                .ok_or("expected digest mismatch")?
                .code(),
            AnnotationArtifactErrorCode::ArtifactPayloadInvalid
        );
        Ok(())
    }

    #[test]
    fn publication_key_is_store_neutral_and_path_safe() -> Result<(), Box<dyn std::error::Error>> {
        let key = AnnotationPublicationKey::new("retail-12.1", "mainline", "emmy-library")?;
        assert_eq!(key.catalog_path(), "retail-12.1/mainline/emmy-library");
        assert!(AnnotationPublicationKey::new("../retail", "mainline", "emmy-library").is_err());
        Ok(())
    }

    #[test]
    fn native_library_type_remains_a_serializable_generation_input() {
        fn assert_serializable<T: Serialize + ?Sized>() {}
        assert_serializable::<NativeLibrary<'_>>();
    }
}

use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::{CanonicalResult, ContentDigest, SourceContent, domain_separated_digest};

use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};

pub(crate) fn source_digest(bytes: &[u8]) -> ContentDigest<SourceContent> {
    let digest = Sha256::digest(bytes);
    let mut output = [0_u8; 32];
    output.copy_from_slice(&digest);
    ContentDigest::from_bytes(output)
}

pub(crate) fn canonical_digest<T: Serialize + ?Sized>(
    domain: &str,
    value: &T,
    phase: ProjectPhase,
) -> ProjectResult<ContentDigest<CanonicalResult>> {
    let bytes = domain_separated_digest(domain, value).map_err(|source| {
        ProjectError::new(
            ProjectErrorCode::CanonicalizationFailed,
            phase,
            format!("canonical identity derivation failed: {source}"),
        )
    })?;
    Ok(ContentDigest::from_bytes(bytes))
}

pub(crate) fn canonical_id<T: Serialize + ?Sized>(
    prefix: &str,
    domain: &str,
    value: &T,
    phase: ProjectPhase,
) -> ProjectResult<Box<str>> {
    let bytes = domain_separated_digest(domain, value).map_err(|source| {
        ProjectError::new(
            ProjectErrorCode::CanonicalizationFailed,
            phase,
            format!("canonical identity derivation failed: {source}"),
        )
    })?;
    Ok(format!("{prefix}{}", encode_hex(&bytes)).into_boxed_str())
}

pub(crate) fn parse_source_digest(
    value: &str,
    phase: ProjectPhase,
) -> ProjectResult<ContentDigest<SourceContent>> {
    let parsed = ContentDigest::<SourceContent>::parse(value).map_err(|source| {
        ProjectError::new(
            ProjectErrorCode::FileDigestMismatch,
            phase,
            format!("source digest is invalid: {source}"),
        )
    })?;
    if parsed.was_canonical() {
        Ok(parsed.into_value())
    } else {
        Err(ProjectError::new(
            ProjectErrorCode::FileDigestMismatch,
            phase,
            "source digest is not canonical",
        ))
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

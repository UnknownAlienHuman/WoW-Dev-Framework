use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{ServiceError, ServiceErrorCode, ServiceResult};

pub(crate) fn canonical_id<T: Serialize + ?Sized>(
    prefix: &str,
    value: &T,
) -> ServiceResult<Box<str>> {
    let bytes = canonical_json_bytes(value).map_err(|error| {
        ServiceError::new(
            ServiceErrorCode::CanonicalizationFailed,
            format!("canonical service identity failed: {error}"),
        )
    })?;
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(prefix.len() + digest.len() * 2);
    encoded.push_str(prefix);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(encoded.into_boxed_str())
}

pub(crate) fn validate_identity(value: &str) -> ServiceResult<()> {
    if value.is_empty()
        || value.len() > 1024
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(ServiceError::new(
            ServiceErrorCode::InvalidIdentity,
            "service identity is empty, oversized, or contains whitespace/control characters",
        ));
    }
    Ok(())
}

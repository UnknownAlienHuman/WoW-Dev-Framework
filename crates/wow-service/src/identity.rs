use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{ServiceError, ServiceErrorCode, ServiceResult};

pub(crate) fn canonical_digest<T: Serialize + ?Sized>(
    prefix: &str,
    value: &T,
) -> ServiceResult<Box<str>> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        ServiceError::new(
            ServiceErrorCode::CanonicalizationFailed,
            format!("canonical serialization failed: {error}"),
        )
    })?;
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(prefix.len() + 64);
    encoded.push_str(prefix);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").map_err(|_| {
            ServiceError::new(
                ServiceErrorCode::CanonicalizationFailed,
                "digest encoding failed",
            )
        })?;
    }
    Ok(encoded.into_boxed_str())
}

pub(crate) fn validate_identifier(value: &str, field: &str) -> ServiceResult<()> {
    if value.is_empty()
        || value.len() > 512
        || value.chars().any(char::is_control)
        || value.trim() != value
    {
        return Err(ServiceError::new(
            ServiceErrorCode::InvalidConfiguration,
            format!("invalid {field}"),
        ));
    }
    Ok(())
}

pub(crate) fn validate_digest(value: &str, field: &str) -> ServiceResult<()> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(ServiceError::new(
            ServiceErrorCode::InvalidContext,
            format!("{field} is not a sha256 identity"),
        ));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ServiceError::new(
            ServiceErrorCode::InvalidContext,
            format!("invalid {field}"),
        ));
    }
    Ok(())
}

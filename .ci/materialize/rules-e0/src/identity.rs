use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{RuleError, RuleErrorCode, RuleResult};

pub(crate) fn canonical_id<T: Serialize + ?Sized>(prefix: &str, value: &T) -> RuleResult<Box<str>> {
    let bytes = canonical_json_bytes(value).map_err(|error| {
        RuleError::new(
            RuleErrorCode::CanonicalizationFailed,
            format!("canonical serialization failed: {error}"),
            None,
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

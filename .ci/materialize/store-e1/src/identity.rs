use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{StoreError, StoreErrorCode, StoreResult};

pub(crate) fn sha256_id(bytes: &[u8]) -> Box<str> {
    let digest = Sha256::digest(bytes);
    let mut value = String::with_capacity(7 + digest.len() * 2);
    value.push_str("sha256:");
    encode_hex(&digest, &mut value);
    value.into_boxed_str()
}

pub(crate) fn canonical_id<T: Serialize + ?Sized>(
    prefix: &str,
    value: &T,
) -> StoreResult<Box<str>> {
    let bytes = canonical_json_bytes(value).map_err(|error| {
        StoreError::new(
            StoreErrorCode::CanonicalizationFailed,
            format!("canonical serialization failed: {error}"),
            None,
        )
    })?;
    let digest = Sha256::digest(bytes);
    let mut result = String::with_capacity(prefix.len() + digest.len() * 2);
    result.push_str(prefix);
    encode_hex(&digest, &mut result);
    Ok(result.into_boxed_str())
}

pub(crate) fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn encode_hex(bytes: &[u8], output: &mut String) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
}

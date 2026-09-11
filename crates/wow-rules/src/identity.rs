use serde::Serialize;
use wow_core::domain_separated_digest;

use crate::{RuleError, RuleErrorCode, RuleResult};

pub(crate) fn canonical_id<T: Serialize + ?Sized>(
    prefix: &str,
    domain: &'static str,
    value: &T,
) -> RuleResult<Box<str>> {
    let digest = domain_separated_digest(domain, value).map_err(|error| {
        RuleError::new(
            RuleErrorCode::CanonicalizationFailed,
            format!("canonical identity derivation failed: {error}"),
        )
    })?;
    Ok(format!("{prefix}{}", encode_hex(&digest)).into_boxed_str())
}

fn encode_hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(TABLE[usize::from(byte >> 4)]));
        output.push(char::from(TABLE[usize::from(byte & 0x0f)]));
    }
    output
}

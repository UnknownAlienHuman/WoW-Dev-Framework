use super::*;
use serde::{Deserialize, Serialize};

pub(super) const INPUT_SCHEMA: &str = "wow-reference/configured-compatibility-alias-input/1";

const MAX_TEXT_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Input {
    pub(super) schema: String,
    pub(super) profile_id: String,
    pub(super) profile_digest: String,
    pub(super) base_generation: String,
    pub(super) base_view_digest: String,
    pub(super) declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Declaration {
    pub(super) value: String,
    pub(super) target: Target,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Target {
    pub(super) partition_id: String,
    pub(super) record_key: String,
    pub(super) record_digest: String,
}

impl Input {
    pub(super) fn parse(bytes: &[u8], stop: &AtomicBool) -> Result<Self, NativeError> {
        checkpoint(stop)?;

        if bytes.len() > MAX_INPUT_BYTES {
            return Err(error(NativeErrorCode::Limit));
        }

        let input: Input =
            serde_json::from_slice(bytes).map_err(|_| error(NativeErrorCode::InvalidIdentity))?;

        if input.schema != INPUT_SCHEMA {
            return Err(error(NativeErrorCode::InvalidIdentity));
        }

        validate_text(&input.profile_id)?;
        validate_digest(&input.profile_digest)?;
        validate_text(&input.base_generation)?;
        validate_digest(&input.base_view_digest)?;

        if input.declarations.len() > MAX_DECLARATIONS {
            return Err(error(NativeErrorCode::Limit));
        }

        for declaration in &input.declarations {
            checkpoint(stop)?;

            validate_text(&declaration.value)?;
            validate_text(&declaration.target.partition_id)?;
            validate_text(&declaration.target.record_key)?;
            validate_digest(&declaration.target.record_digest)?;

            crate::ReferenceRecord::validate_key(&declaration.target.record_key)
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
        }

        Ok(input)
    }
}

fn validate_text(value: &str) -> Result<(), NativeError> {
    if value.is_empty()
        || value.len() > MAX_TEXT_BYTES
        || value.chars().any(char::is_control)
        || value.starts_with(char::is_whitespace)
        || value.ends_with(char::is_whitespace)
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }

    Ok(())
}

fn validate_digest(value: &str) -> Result<(), NativeError> {
    let value = value
        .strip_prefix("sha256:")
        .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }

    Ok(())
}

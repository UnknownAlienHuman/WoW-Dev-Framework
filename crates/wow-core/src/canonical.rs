use serde::Serialize;

use crate::{ContentDigest, CoreResult};

pub fn canonical_json<T>(value: &T) -> CoreResult<String>
where
    T: Serialize,
{
    Ok(serde_json::to_string(value)?)
}

pub fn canonical_json_digest<T>(value: &T) -> CoreResult<ContentDigest>
where
    T: Serialize,
{
    let json = canonical_json(value)?;
    Ok(ContentDigest::from_bytes(json.as_bytes()))
}

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{StoreError, StoreErrorCode, StoreResult};

macro_rules! text_id {
    ($name:ident, $label:literal, $max:expr, $validator:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Box<str>);

        impl $name {
            pub fn new(value: impl Into<Box<str>>) -> StoreResult<Self> {
                let value = value.into();
                if value.is_empty() || value.len() > $max || !$validator(&value) {
                    return Err(StoreError::new(
                        StoreErrorCode::IdentifierInvalid,
                        concat!("invalid ", $label),
                    ));
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

fn component(value: &str) -> bool {
    value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
    })
}

fn path(value: &str) -> bool {
    !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains("//")
        && value
            .split('/')
            .all(|part| part != "." && part != ".." && component(part))
}

fn digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn object_id(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("store-object:sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

text_id!(ObjectId, "object id", 96, object_id);
text_id!(OperationId, "operation id", 256, component);
text_id!(LeaseId, "lease id", 256, component);
text_id!(CatalogName, "catalog name", 128, component);
text_id!(CatalogPath, "catalog path", 1024, path);
text_id!(RequestDigest, "request digest", 71, digest);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LogicalEpoch(u64);

impl LogicalEpoch {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

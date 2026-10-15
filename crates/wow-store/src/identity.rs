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

/// Opaque identity minted by Store for one physical owned database.
///
/// There is intentionally no public constructor or deserializer: callers can
/// read and carry an admitted identity, but cannot assert or rebind one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct StoreInstanceId(Box<str>);

impl StoreInstanceId {
    pub(crate) fn from_sqlite_entropy(bytes: &[u8]) -> StoreResult<Self> {
        if bytes.len() != 16 {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "SQLite returned an invalid store identity entropy value",
            ));
        }
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut value = String::with_capacity(47);
        value.push_str("store-instance:");
        for byte in bytes {
            value.push(char::from(HEX[usize::from(byte >> 4)]));
            value.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        Ok(Self(value.into_boxed_str()))
    }

    pub(crate) fn from_stored(value: String) -> StoreResult<Self> {
        let valid = value.strip_prefix("store-instance:").is_some_and(|hex| {
            hex.len() == 32
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        });
        if !valid {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "stored store instance identity is malformed",
            ));
        }
        Ok(Self(value.into_boxed_str()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StoreInstanceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

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

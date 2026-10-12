//! Exact producer input custody. Only the service's native producer can qualify
//! these bytes; retaining them grants no Reference or annotation authority.
use std::{
    fmt,
    sync::{Arc, atomic::AtomicBool},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use wow_core::{CanonicalResult, ContentDigest};

use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};

/// Finite raw producer-input allowance, separate from the full replay wire bound.
pub const NATIVE_LIBRARY_INPUT_MAX_BYTES: usize = 16 * 1024 * 1024;

/// Immutable service-owned input data, bound into project configuration identity.
#[derive(Clone, PartialEq, Eq)]
pub struct NativeLibraryInput {
    text: Arc<str>,
    digest: ContentDigest<CanonicalResult>,
}

impl NativeLibraryInput {
    pub fn new(text: String, stop: &AtomicBool) -> ProjectResult<Self> {
        crate::analyzer::checkpoint(stop)?;
        if text.is_empty() || text.len() > NATIVE_LIBRARY_INPUT_MAX_BYTES {
            return Err(ProjectError::new(
                ProjectErrorCode::SourceBudgetExceeded,
                ProjectPhase::Configuration,
                "native Library producer input exceeds its byte allowance",
            ));
        }
        let mut hash = Sha256::new();
        hash.update(b"wow-project/native-library-input/1\0");
        for bytes in text.as_bytes().chunks(32 * 1024) {
            crate::analyzer::checkpoint(stop)?;
            hash.update(bytes);
        }
        crate::analyzer::checkpoint(stop)?;
        Ok(Self {
            text: Arc::from(text),
            digest: ContentDigest::from_bytes(hash.finalize().into()),
        })
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }

    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
}

impl fmt::Debug for NativeLibraryInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeLibraryInput")
            .field("digest", &self.digest)
            .field("byte_length", &self.text.len())
            .finish_non_exhaustive()
    }
}

impl Serialize for NativeLibraryInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.text)
    }
}

impl<'de> Deserialize<'de> for NativeLibraryInput {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // The containing replay has already enforced its complete raw-wire limit.
        // Requalification is cancellable and occurs after strict archive decode.
        Self::new(String::deserialize(deserializer)?, &AtomicBool::new(false))
            .map_err(serde::de::Error::custom)
    }
}

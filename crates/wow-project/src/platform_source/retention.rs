//! Explicit native package retention. Encoded syntax is an accounting unit,
//! not an allocator or resident-memory guarantee.
use serde::{Deserialize, Serialize};

use super::{invalid, profile::MAX_TOCS};
use crate::ProjectResult;

const MAX_PLAN_ENCODING_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum RetentionProfile {
    #[serde(rename = "wow-project/platform-package-retention/1")]
    BoundedV1,
}

/// Caller-selected limits for one native package universe. No source, Main,
/// analyzer, graph or runtime capability can be constructed from these values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformPackageRetentionLimits {
    profile: RetentionProfile,
    max_packages: usize,
    max_total_plan_encoding_bytes: u64,
}

impl PlatformPackageRetentionLimits {
    pub fn new(max_packages: usize, max_total_plan_encoding_bytes: u64) -> ProjectResult<Self> {
        let limits = Self {
            profile: RetentionProfile::BoundedV1,
            max_packages,
            max_total_plan_encoding_bytes,
        };
        limits.validate()?;
        Ok(limits)
    }

    #[must_use]
    pub const fn max_packages(self) -> usize {
        self.max_packages
    }

    #[must_use]
    pub const fn max_total_plan_encoding_bytes(self) -> u64 {
        self.max_total_plan_encoding_bytes
    }

    pub(crate) fn validate(self) -> ProjectResult<()> {
        if self.max_packages == 0
            || self.max_packages > MAX_TOCS
            || self.max_total_plan_encoding_bytes == 0
            || self.max_total_plan_encoding_bytes > MAX_PLAN_ENCODING_BYTES
        {
            return Err(invalid("invalid native package retention limits"));
        }
        Ok(())
    }
}

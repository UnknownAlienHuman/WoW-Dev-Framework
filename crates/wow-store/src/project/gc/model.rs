//! Bounded policy inputs for project-store garbage collection.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::super::model::{
    MAX_GENERATION_BYTES, MAX_GENERATIONS, MAX_VERSIONS, StoreGenerationId, digest, encode,
    failure, invalid, named,
};
use crate::{StoreErrorCode, StoreResult};

const SCHEMA: &str = "wow-store/project-gc-policy/1";

/// Exact retained generations and finite collection budgets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGcPolicy {
    schema: String,
    policy_id: String,
    retained_generations: BTreeSet<StoreGenerationId>,
    max_generations: usize,
    max_partition_versions: usize,
    max_payload_bytes: u64,
}

impl ProjectGcPolicy {
    pub fn new(
        policy_id: &str,
        retained_generations: BTreeSet<StoreGenerationId>,
        max_generations: usize,
        max_partition_versions: usize,
        max_payload_bytes: u64,
    ) -> StoreResult<Self> {
        if !named(policy_id) {
            return Err(failure(StoreErrorCode::IdentifierInvalid));
        }
        if retained_generations.len() > MAX_GENERATIONS as usize
            || !(1..=MAX_GENERATIONS as usize).contains(&max_generations)
            || !(1..=MAX_VERSIONS as usize).contains(&max_partition_versions)
            || !(1..=MAX_GENERATION_BYTES as u64).contains(&max_payload_bytes)
        {
            return Err(failure(StoreErrorCode::ConfigurationInvalid));
        }
        Ok(Self {
            schema: SCHEMA.into(),
            policy_id: policy_id.into(),
            retained_generations,
            max_generations,
            max_partition_versions,
            max_payload_bytes,
        })
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn policy_id(&self) -> &str {
        &self.policy_id
    }

    pub fn retained_generations(&self) -> &BTreeSet<StoreGenerationId> {
        &self.retained_generations
    }

    pub fn max_generations(&self) -> usize {
        self.max_generations
    }

    pub fn max_partition_versions(&self) -> usize {
        self.max_partition_versions
    }

    pub fn max_payload_bytes(&self) -> u64 {
        self.max_payload_bytes
    }

    pub(super) fn validate(&self) -> StoreResult<()> {
        let expected = Self::new(
            &self.policy_id,
            self.retained_generations.clone(),
            self.max_generations,
            self.max_partition_versions,
            self.max_payload_bytes,
        )?;
        if self != &expected {
            return Err(invalid());
        }
        Ok(())
    }

    pub(super) fn digest(&self) -> StoreResult<String> {
        self.validate()?;
        Ok(digest("project-gc-policy", &encode(self, 256 * 1024)?))
    }
}

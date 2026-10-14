//! Opaque in-process receipts for verified progressive delivery budgets.

/// An in-process receipt from one validated native prefix and its exact closed
/// envelope. The cumulative history is reconstructed and checked against the
/// issued usage; the replay pair accounts the native validation performed now.
/// Service separately verifies persisted issuance. This type is deliberately
/// neither serializable nor deserializable.
pub struct ContextBudgetReceipt {
    request_id: Box<str>,
    continuation_id: Box<str>,
    semantic_pack_id: Box<str>,
    expansion_id: Box<str>,
    universe_set_id: Box<str>,
    profile_id: Box<str>,
    round: u8,
    max_context_work: u64,
    max_metadata_bytes: u64,
    historical_cumulative_work_reserved: u64,
    historical_cumulative_bytes_reserved: u64,
    replay_work_reserved: u64,
    replay_bytes_reserved: u64,
}

impl ContextBudgetReceipt {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_verified_pack(
        request_id: Box<str>,
        continuation_id: Box<str>,
        semantic_pack_id: Box<str>,
        expansion_id: Box<str>,
        universe_set_id: Box<str>,
        profile_id: Box<str>,
        round: u8,
        max_context_work: u64,
        max_metadata_bytes: u64,
        historical_cumulative_work_reserved: u64,
        historical_cumulative_bytes_reserved: u64,
        replay_work_reserved: u64,
        replay_bytes_reserved: u64,
    ) -> Self {
        Self {
            request_id,
            continuation_id,
            semantic_pack_id,
            expansion_id,
            universe_set_id,
            profile_id,
            round,
            max_context_work,
            max_metadata_bytes,
            historical_cumulative_work_reserved,
            historical_cumulative_bytes_reserved,
            replay_work_reserved,
            replay_bytes_reserved,
        }
    }

    pub(crate) fn request_id(&self) -> &str {
        &self.request_id
    }

    pub(crate) fn continuation_id(&self) -> &str {
        &self.continuation_id
    }

    pub(crate) fn semantic_pack_id(&self) -> &str {
        &self.semantic_pack_id
    }

    pub(crate) fn expansion_id(&self) -> &str {
        &self.expansion_id
    }

    pub(crate) fn universe_set_id(&self) -> &str {
        &self.universe_set_id
    }

    pub(crate) fn profile_id(&self) -> &str {
        &self.profile_id
    }

    pub(crate) const fn round(&self) -> u8 {
        self.round
    }

    pub(crate) const fn max_context_work(&self) -> u64 {
        self.max_context_work
    }

    pub(crate) const fn max_metadata_bytes(&self) -> u64 {
        self.max_metadata_bytes
    }

    pub(crate) const fn historical_cumulative_work_reserved(&self) -> u64 {
        self.historical_cumulative_work_reserved
    }

    pub(crate) const fn historical_cumulative_bytes_reserved(&self) -> u64 {
        self.historical_cumulative_bytes_reserved
    }

    pub(crate) const fn replay_work_reserved(&self) -> u64 {
        self.replay_work_reserved
    }

    pub(crate) const fn replay_bytes_reserved(&self) -> u64 {
        self.replay_bytes_reserved
    }
}

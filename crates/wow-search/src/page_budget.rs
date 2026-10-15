//! Continuing page budget for one exact retrieval result.
//!
//! The budget is the serialized ledger of every charge already made over one
//! immutable candidate set. Each counter is cumulative and derived from bytes
//! actually delivered, so a replayed chain resumes from its recorded position
//! rather than from a caller-supplied remainder.

use serde::{Deserialize, Serialize};

use crate::{SearchError, SearchResult};

/// Plain-unit view of one admitted continuing page bound.
///
/// A policy keeps its typed nonzero bounds; a charge needs the plain units that
/// a counter is actually compared against.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PageLimits {
    pub(crate) max_pages: u32,
    pub(crate) max_delivered_candidates: u32,
    pub(crate) max_candidate_bytes: u64,
    pub(crate) max_page_bytes: u64,
    pub(crate) max_replayed_manifest_bytes: u64,
}

/// Cumulative charge accumulated by the pages delivered over one exact result.
///
/// Field names and order are part of the retained cursor bytes, so they are not
/// renamed or reordered without a real compatibility consumer. Integrity is
/// supplemented by the service's exact prior completion receipt. A digest alone
/// is never authorization to reset this cumulative budget.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PageBudget {
    pub(crate) pages: u32,
    pub(crate) delivered_candidates: u32,
    pub(crate) delivered_candidate_bytes: u64,
    pub(crate) delivered_page_bytes: u64,
    pub(crate) replayed_manifest_bytes: u64,
}

impl PageBudget {
    /// Refuse a replay whose cumulative counters or replay bound are already at
    /// their limit, before the service repeats any query work.
    pub(crate) fn admit_replay(&self, limits: PageLimits, manifest_bytes: u64) -> SearchResult<()> {
        if self.pages >= limits.max_pages
            || self.delivered_candidates >= limits.max_delivered_candidates
            || self.delivered_candidate_bytes >= limits.max_candidate_bytes
            || self.delivered_page_bytes >= limits.max_page_bytes
            || self
                .replayed_manifest_bytes
                .checked_add(manifest_bytes)
                .is_none_or(|bytes| bytes > limits.max_replayed_manifest_bytes)
        {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        Ok(())
    }

    /// Charge one more page over the same result and refuse any counter that
    /// would pass its bound. Delivered page bytes are charged separately by
    /// [`PageBudget::check_page_bytes`] once the canonical page length is known.
    pub(crate) fn next(
        &self,
        limits: PageLimits,
        next_candidate: u32,
        candidate_bytes: u64,
        manifest_bytes: u64,
    ) -> SearchResult<Self> {
        let charged = Self {
            pages: self
                .pages
                .checked_add(1)
                .ok_or(SearchError::FileSetBudgetExceeded)?,
            delivered_candidates: next_candidate,
            delivered_candidate_bytes: self
                .delivered_candidate_bytes
                .checked_add(candidate_bytes)
                .ok_or(SearchError::FileSetBudgetExceeded)?,
            delivered_page_bytes: self.delivered_page_bytes,
            replayed_manifest_bytes: self
                .replayed_manifest_bytes
                .checked_add(manifest_bytes)
                .ok_or(SearchError::FileSetBudgetExceeded)?,
        };
        if charged.pages > limits.max_pages
            || charged.delivered_candidates > limits.max_delivered_candidates
            || charged.delivered_candidate_bytes > limits.max_candidate_bytes
            || charged.replayed_manifest_bytes > limits.max_replayed_manifest_bytes
        {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        Ok(charged)
    }

    /// Refuse a charged canonical page size above the admitted page bound.
    pub(crate) fn check_page_bytes(limits: PageLimits, charged: u64) -> SearchResult<()> {
        if charged > limits.max_page_bytes {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        Ok(())
    }
}

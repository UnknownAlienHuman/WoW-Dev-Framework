//! Whole-candidate pages over one immutable result; no mutable selectors.
use super::*;
use crate::page_budget::{PageBudget, PageLimits};

const POLICY_SCHEMA: &str = "wow-search/project-file-result-page-policy/1";
const PAGE_SCHEMA: &str = "wow-search/project-file-result-page/1";
const PAGE_PROFILE: &str = "wow-search/whole-candidate-exact-replay/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalPagePolicy {
    schema: String,
    page_size: NonZeroU32,
    max_pages: NonZeroU32,
    max_delivered_candidates: NonZeroU32,
    max_delivered_candidate_bytes: NonZeroU64,
    max_delivered_page_bytes: NonZeroU64,
    max_replayed_manifest_bytes: NonZeroU64,
}

impl RetrievalPagePolicy {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> SearchResult<Self> {
        check(stop)?;
        if bytes.len() > 4096 {
            return Err(SearchError::InvalidProfile);
        }
        let policy: Self =
            serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
        if policy.schema != POLICY_SCHEMA {
            return Err(SearchError::InvalidProfile);
        }
        Ok(policy)
    }

    /// Plain-unit charge view of this exact policy. The policy stays the public
    /// typed profile; a budget charge compares counters against plain units.
    fn limits(&self) -> PageLimits {
        PageLimits {
            max_pages: self.max_pages.get(),
            max_delivered_candidates: self.max_delivered_candidates.get(),
            max_candidate_bytes: self.max_delivered_candidate_bytes.get(),
            max_page_bytes: self.max_delivered_page_bytes.get(),
            max_replayed_manifest_bytes: self.max_replayed_manifest_bytes.get(),
        }
    }
}

/// Integrity is supplemented by the service's exact prior completion receipt.
/// A digest alone is never authorization to reset this cumulative budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalCursor {
    cursor_id: Box<str>,
    result_set_id: Box<str>,
    query_id: Box<str>,
    universe_set_id: Box<str>,
    shard_id: Box<str>,
    shard_profile_digest: Box<str>,
    page_policy: RetrievalPagePolicy,
    next_candidate: u32,
    budget: PageBudget,
}

impl RetrievalCursor {
    /// Reject an exhausted chain before the service repeats any query work.
    pub fn check_replay_budget(
        &self,
        policy: &RetrievalPagePolicy,
        manifest_bytes: u64,
    ) -> SearchResult<()> {
        if &self.page_policy != policy || self.cursor_id != self.id()? {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        self.budget.admit_replay(policy.limits(), manifest_bytes)
    }

    fn id(&self) -> SearchResult<Box<str>> {
        digest(
            "search-page-cursor:",
            &(
                PAGE_PROFILE,
                &self.result_set_id,
                &self.query_id,
                &self.universe_set_id,
                &self.shard_id,
                &self.shard_profile_digest,
                &self.page_policy,
                self.next_candidate,
                &self.budget,
            ),
        )
    }
}

#[derive(Serialize)]
struct PagePayload<'a> {
    schema: &'static str,
    page_profile: &'static str,
    result_set_id: &'a str,
    normalized_query_id: &'a str,
    universe_set_id: &'a str,
    shard_id: &'a str,
    shard_profile_digest: &'a str,
    query_profile: &'static str,
    ranking_profile: &'static str,
    normalized_request: &'a RetrievalRequest,
    normalization: &'static str,
    rr_scale: u32,
    rr_offset: u32,
    rank_tuple_fields: [&'static str; 3],
    owner: &'a OwnerManifestEvidence,
    retrieval_state: RetrievalState,
    absence_authoritative: bool,
    document_coverage: &'static str,
    lanes: &'a [LaneResult],
    candidate_enumeration_complete: bool,
    candidate_signals_omitted_by_cap: u32,
    total_retained_candidates: u32,
    first_candidate: u32,
    next_candidate: u32,
    end: bool,
    candidates: &'a [RankedCandidate],
    page_policy: &'a RetrievalPagePolicy,
    cumulative_budget: PageBudget,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<&'a RetrievalCursor>,
}

pub struct RetrievalPage {
    bytes: Box<[u8]>,
    next_cursor: Option<RetrievalCursor>,
}

impl RetrievalPage {
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn next_cursor(&self) -> Option<&RetrievalCursor> {
        self.next_cursor.as_ref()
    }
}

impl RetrievalResult {
    /// The service must reacquire the exact shard and compare this result's
    /// complete bytes with its retained manifest before continuing a page.
    pub fn page(
        &self,
        policy: &RetrievalPagePolicy,
        cursor: Option<&RetrievalCursor>,
        stop: &AtomicBool,
    ) -> SearchResult<RetrievalPage> {
        check(stop)?;
        if policy.schema != POLICY_SCHEMA {
            return Err(SearchError::InvalidProfile);
        }
        let (first, mut budget) = if let Some(cursor) = cursor {
            if cursor.cursor_id != cursor.id()?
                || cursor.result_set_id != self.result_set_id
                || cursor.query_id != self.payload.normalized_query_id
                || cursor.universe_set_id != self.payload.universe_set_id
                || cursor.shard_id != self.payload.shard_id
                || cursor.shard_profile_digest != self.payload.shard_profile_digest
                || &cursor.page_policy != policy
                || cursor.next_candidate != cursor.budget.delivered_candidates
            {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            (cursor.next_candidate, cursor.budget.clone())
        } else {
            (0, PageBudget::default())
        };
        let total = u32::try_from(self.payload.candidates.len())
            .map_err(|_| SearchError::FileSetBudgetExceeded)?;
        if first > total {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let next = first.saturating_add(policy.page_size.get()).min(total);
        let candidates = &self.payload.candidates[first as usize..next as usize];
        let candidate_bytes = wow_core::canonical_json_bytes(candidates)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        budget = budget.next(
            policy.limits(),
            next,
            candidate_bytes.len() as u64,
            self.bytes.len() as u64,
        )?;
        let prior_page_bytes = budget.delivered_page_bytes;
        // Charge the exact canonical page, including repeated owner metadata,
        // budget and cursor. Only decimal widths can change its byte length;
        // bounded fixed-point encoding includes the accounting record itself.
        for _ in 0..8 {
            check(stop)?;
            let next_cursor = if next < total {
                let mut cursor = RetrievalCursor {
                    cursor_id: "".into(),
                    result_set_id: self.result_set_id.clone(),
                    query_id: self.payload.normalized_query_id.clone(),
                    universe_set_id: self.payload.universe_set_id.clone(),
                    shard_id: self.payload.shard_id.clone(),
                    shard_profile_digest: self.payload.shard_profile_digest.clone(),
                    page_policy: policy.clone(),
                    next_candidate: next,
                    budget: budget.clone(),
                };
                cursor.cursor_id = cursor.id()?;
                Some(cursor)
            } else {
                None
            };
            let payload = PagePayload {
                schema: PAGE_SCHEMA,
                page_profile: PAGE_PROFILE,
                result_set_id: &self.result_set_id,
                normalized_query_id: &self.payload.normalized_query_id,
                universe_set_id: &self.payload.universe_set_id,
                shard_id: &self.payload.shard_id,
                shard_profile_digest: &self.payload.shard_profile_digest,
                query_profile: QUERY_PROFILE,
                ranking_profile: RANKING_PROFILE,
                normalized_request: &self.payload.normalized_request,
                normalization: self.payload.normalization,
                rr_scale: self.payload.rr_scale,
                rr_offset: self.payload.rr_offset,
                rank_tuple_fields: self.payload.rank_tuple_fields,
                owner: &self.payload.owner,
                retrieval_state: self.state(),
                absence_authoritative: self.payload.absence_authoritative,
                document_coverage: self.payload.document_coverage,
                lanes: &self.payload.lanes,
                candidate_enumeration_complete: self.payload.candidate_enumeration_complete,
                candidate_signals_omitted_by_cap: self.payload.candidate_signals_omitted_by_cap,
                total_retained_candidates: total,
                first_candidate: first,
                next_candidate: next,
                end: next == total,
                candidates,
                page_policy: policy,
                cumulative_budget: budget.clone(),
                next_cursor: next_cursor.as_ref(),
            };
            let bytes = wow_core::canonical_json_bytes(&payload)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
            let charged = prior_page_bytes
                .checked_add(bytes.len() as u64)
                .ok_or(SearchError::FileSetBudgetExceeded)?;
            PageBudget::check_page_bytes(policy.limits(), charged)?;
            if charged == budget.delivered_page_bytes {
                return Ok(RetrievalPage {
                    bytes: bytes.into_boxed_slice(),
                    next_cursor,
                });
            }
            budget.delivered_page_bytes = charged;
        }
        Err(SearchError::OwnerEvidenceInvalid)
    }
}

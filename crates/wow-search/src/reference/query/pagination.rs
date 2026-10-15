//! Pages of the owning Search result representation, never decoded Reference data.
use super::*;
use crate::page_budget::{PageBudget, PageLimits};
const POLICY_SCHEMA: &str = "wow-search/native-reference-page-policy/1";
const PAGE_SCHEMA: &str = "wow-search/native-reference-result-page/1";
const PAGE_PROFILE: &str = "wow-search/reference-whole-candidate-exact-replay/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePagePolicy {
    schema: String,
    page_size: NonZeroU32,
    max_pages: NonZeroU32,
    max_delivered_candidates: NonZeroU32,
    max_delivered_candidate_bytes: NonZeroU64,
    max_delivered_page_bytes: NonZeroU64,
    max_replayed_manifest_bytes: NonZeroU64,
}
impl ReferencePagePolicy {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePageCursor {
    cursor_id: Box<str>,
    result_set_id: Box<str>,
    normalized_query_id: Box<str>,
    universe_id: Box<str>,
    shard_id: Box<str>,
    shard_profile_digest: Box<str>,
    page_policy: ReferencePagePolicy,
    next_candidate: u32,
    budget: PageBudget,
}
impl ReferencePageCursor {
    pub fn check_replay_budget(
        &self,
        policy: &ReferencePagePolicy,
        manifest_bytes: u64,
    ) -> SearchResult<()> {
        if &self.page_policy != policy
            || self.cursor_id != self.id()?
            || policy.schema != POLICY_SCHEMA
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        self.budget.admit_replay(policy.limits(), manifest_bytes)
    }
    fn id(&self) -> SearchResult<Box<str>> {
        identity(
            "reference-page-cursor:",
            &(
                PAGE_PROFILE,
                &self.result_set_id,
                &self.normalized_query_id,
                &self.universe_id,
                &self.shard_id,
                &self.shard_profile_digest,
                &self.page_policy,
                self.next_candidate,
                &self.budget,
            ),
        )
    }
}
pub struct ReferencePage {
    bytes: Box<[u8]>,
    next_cursor: Option<ReferencePageCursor>,
}
impl ReferencePage {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn next_cursor(&self) -> Option<&ReferencePageCursor> {
        self.next_cursor.as_ref()
    }
}

impl ReferenceQueryResult {
    /// This sealed result is produced only by the native owner query. A service
    /// continuing a retained session must first prove full result byte equality.
    pub fn page(
        &self,
        policy: &ReferencePagePolicy,
        cursor: Option<&ReferencePageCursor>,
        stop: &AtomicBool,
    ) -> SearchResult<ReferencePage> {
        check(stop)?;
        if policy.schema != POLICY_SCHEMA {
            return Err(SearchError::InvalidProfile);
        }
        // Search owns this representation. No Reference payload, source record
        // or cached public report is parsed into a native owner here.
        let mut envelope: serde_json::Value =
            serde_json::from_slice(&self.bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let result_id = text_field(&envelope, "result_set_id")?
            .to_owned()
            .into_boxed_str();
        let mut summary = envelope
            .as_object_mut()
            .ok_or(SearchError::OwnerEvidenceInvalid)?
            .remove("payload")
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let query_id = text_field(&summary, "normalized_query_id")?
            .to_owned()
            .into_boxed_str();
        let universe_id = text_field(&summary, "universe_id")?
            .to_owned()
            .into_boxed_str();
        let shard_id = text_field(&summary, "shard_id")?
            .to_owned()
            .into_boxed_str();
        let profile_digest = text_field(
            summary
                .get("owner")
                .ok_or(SearchError::OwnerEvidenceInvalid)?,
            "profile_digest",
        )?
        .to_owned()
        .into_boxed_str();
        let candidates = summary
            .as_object_mut()
            .ok_or(SearchError::OwnerEvidenceInvalid)?
            .remove("candidates")
            .and_then(|value| match value {
                serde_json::Value::Array(rows) => Some(rows),
                _ => None,
            })
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let total =
            u32::try_from(candidates.len()).map_err(|_| SearchError::FileSetBudgetExceeded)?;
        let (first, budget) = if let Some(cursor) = cursor {
            if cursor.cursor_id != cursor.id()?
                || cursor.result_set_id != result_id
                || cursor.normalized_query_id != query_id
                || cursor.universe_id != universe_id
                || cursor.shard_id != shard_id
                || cursor.shard_profile_digest != profile_digest
                || &cursor.page_policy != policy
                || cursor.next_candidate != cursor.budget.delivered_candidates
            {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            (cursor.next_candidate, cursor.budget.clone())
        } else {
            (0, PageBudget::default())
        };
        if first > total {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let next = first.saturating_add(policy.page_size.get()).min(total);
        let rows = &candidates[first as usize..next as usize];
        let candidate_bytes = canonical_size(rows)?;
        let mut budget = budget.next(
            policy.limits(),
            next,
            candidate_bytes,
            self.bytes.len() as u64,
        )?;
        let prior_page_bytes = budget.delivered_page_bytes;
        for _ in 0..8 {
            check(stop)?;
            let next_cursor = if next < total {
                let mut cursor = ReferencePageCursor {
                    cursor_id: "".into(),
                    result_set_id: result_id.clone(),
                    normalized_query_id: query_id.clone(),
                    universe_id: universe_id.clone(),
                    shard_id: shard_id.clone(),
                    shard_profile_digest: profile_digest.clone(),
                    page_policy: policy.clone(),
                    next_candidate: next,
                    budget: budget.clone(),
                };
                cursor.cursor_id = cursor.id()?;
                Some(cursor)
            } else {
                None
            };
            #[derive(Serialize)]
            struct PagePayload<'a> {
                schema: &'static str,
                page_profile: &'static str,
                result_set_id: &'a str,
                result_summary: &'a serde_json::Value,
                total_retained_candidates: u32,
                first_candidate: u32,
                next_candidate: u32,
                end: bool,
                candidates: &'a [serde_json::Value],
                page_policy: &'a ReferencePagePolicy,
                cumulative_budget: &'a PageBudget,
                #[serde(skip_serializing_if = "Option::is_none")]
                next_cursor: Option<&'a ReferencePageCursor>,
            }
            let payload = PagePayload {
                schema: PAGE_SCHEMA,
                page_profile: PAGE_PROFILE,
                result_set_id: &result_id,
                result_summary: &summary,
                total_retained_candidates: total,
                first_candidate: first,
                next_candidate: next,
                end: next == total,
                candidates: rows,
                page_policy: policy,
                cumulative_budget: &budget,
                next_cursor: next_cursor.as_ref(),
            };
            let bytes = wow_core::canonical_json_bytes(&payload)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
            let charged = prior_page_bytes
                .checked_add(bytes.len() as u64)
                .ok_or(SearchError::FileSetBudgetExceeded)?;
            PageBudget::check_page_bytes(policy.limits(), charged)?;
            if charged == budget.delivered_page_bytes {
                return Ok(ReferencePage {
                    bytes: bytes.into_boxed_slice(),
                    next_cursor,
                });
            }
            budget.delivered_page_bytes = charged;
        }
        Err(SearchError::OwnerEvidenceInvalid)
    }
}
fn text_field<'a>(value: &'a serde_json::Value, field: &str) -> SearchResult<&'a str> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or(SearchError::OwnerEvidenceInvalid)
}

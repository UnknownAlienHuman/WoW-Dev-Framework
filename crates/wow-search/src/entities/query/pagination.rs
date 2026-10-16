//! Whole-candidate pages of an immutable Search capture, never native Graph truth.
use super::*;
use crate::page_budget::{PageBudget, PageLimits};
const POLICY_SCHEMA: &str = "wow-search/project-entity-page-policy/1";
const PAGE_SCHEMA: &str = "wow-search/project-entity-result-page/1";
const PAGE_PROFILE: &str = "wow-search/project-entity-captured-whole-candidate/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityPagePolicy {
    schema: String,
    page_size: NonZeroU32,
    max_pages: NonZeroU32,
    max_delivered_candidates: NonZeroU32,
    max_delivered_candidate_bytes: NonZeroU64,
    max_delivered_page_bytes: NonZeroU64,
    max_replayed_manifest_bytes: NonZeroU64,
}
impl EntityPagePolicy {
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
pub struct EntityPageCursor {
    cursor_id: Box<str>,
    result_set_id: Box<str>,
    result_query_id: Box<str>,
    universe_id: Box<str>,
    shard_id: Box<str>,
    shard_profile_digest: Box<str>,
    page_policy: EntityPagePolicy,
    next_candidate: u32,
    budget: PageBudget,
}
impl EntityPageCursor {
    pub fn check_replay_budget(
        &self,
        policy: &EntityPagePolicy,
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
        digest(
            "entity-page-cursor:",
            &(
                PAGE_PROFILE,
                &self.result_set_id,
                &self.result_query_id,
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
pub struct EntityPage {
    bytes: Box<[u8]>,
    next_cursor: Option<EntityPageCursor>,
}
impl EntityPage {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn next_cursor(&self) -> Option<&EntityPageCursor> {
        self.next_cursor.as_ref()
    }
}

// Service authorizes a retained capture using its prior immutable session/receipt.
fn render(
    bytes: &[u8],
    shard_profile_digest: &str,
    policy: &EntityPagePolicy,
    cursor: Option<&EntityPageCursor>,
    stop: &AtomicBool,
) -> SearchResult<EntityPage> {
    check(stop)?;
    if policy.schema != POLICY_SCHEMA {
        return Err(SearchError::InvalidProfile);
    }
    // Search owns this representation. No Entity payload, source record
    // or cached public report is parsed into a native owner here.
    let mut envelope: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let result_id = text_field(&envelope, "result_set_id")?
        .to_owned()
        .into_boxed_str();
    let mut summary = envelope
        .as_object_mut()
        .ok_or(SearchError::OwnerEvidenceInvalid)?
        .remove("payload")
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    let query_id = text_field(&summary, "result_query_id")?
        .to_owned()
        .into_boxed_str();
    let universe_id = text_field(&summary, "universe_id")?
        .to_owned()
        .into_boxed_str();
    let shard_id = text_field(&summary, "shard_id")?
        .to_owned()
        .into_boxed_str();
    let profile_digest = shard_profile_digest.to_owned().into_boxed_str();
    let candidates = summary
        .as_object_mut()
        .ok_or(SearchError::OwnerEvidenceInvalid)?
        .remove("candidates")
        .and_then(|value| match value {
            serde_json::Value::Array(rows) => Some(rows),
            _ => None,
        })
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    let total = u32::try_from(candidates.len()).map_err(|_| SearchError::FileSetBudgetExceeded)?;
    let (first, budget) = if let Some(cursor) = cursor {
        if cursor.cursor_id != cursor.id()?
            || cursor.result_set_id != result_id
            || cursor.result_query_id != query_id
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
    let candidate_bytes = size(rows)?;
    let mut budget = budget.next(policy.limits(), next, candidate_bytes, bytes.len() as u64)?;
    let prior_page_bytes = budget.delivered_page_bytes;
    for _ in 0..8 {
        check(stop)?;
        let next_cursor = if next < total {
            let mut cursor = EntityPageCursor {
                cursor_id: "".into(),
                result_set_id: result_id.clone(),
                result_query_id: query_id.clone(),
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
            page_policy: &'a EntityPagePolicy,
            cumulative_budget: &'a PageBudget,
            #[serde(skip_serializing_if = "Option::is_none")]
            next_cursor: Option<&'a EntityPageCursor>,
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
            return Ok(EntityPage {
                bytes: bytes.into_boxed_slice(),
                next_cursor,
            });
        }
        budget.delivered_page_bytes = charged;
    }
    Err(SearchError::OwnerEvidenceInvalid)
}
fn text_field<'a>(value: &'a serde_json::Value, field: &str) -> SearchResult<&'a str> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or(SearchError::OwnerEvidenceInvalid)
}

impl EntityQueryResult {
    pub fn page(
        &self,
        policy: &EntityPagePolicy,
        cursor: Option<&EntityPageCursor>,
        stop: &AtomicBool,
    ) -> SearchResult<EntityPage> {
        render(&self.bytes, &self.profile_digest, policy, cursor, stop)
    }
}
impl EntitySearchShard {
    /// Page Search's recorded result after the caller proves the exact retained
    /// session and preceding receipt. This returns a page of captured data only,
    /// never a reconstructed EntityQueryResult or native Graph owner handle.
    pub fn page_retained_result(
        &self,
        bytes: &[u8],
        request: &EntityQueryRequest,
        policy: &EntityPagePolicy,
        cursor: Option<&EntityPageCursor>,
        stop: &AtomicBool,
    ) -> SearchResult<EntityPage> {
        check(stop)?;
        if bytes.len() as u64 > request.max_result_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let envelope: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let payload = envelope
            .get("payload")
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let query_id = digest(
            "entity-search-query:",
            &(QUERY_PROFILE, self.logical_id(), request),
        )?;
        let universe = digest(
            "entity-search-universe:",
            &(
                &self.manifest.owner.source,
                &self.manifest.graph_registry_digest,
            ),
        )?;
        if envelope.as_object().is_none_or(|object| object.len() != 2)
            || wow_core::canonical_json_bytes(&envelope)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?
                != bytes
            || text_field(&envelope, "result_set_id")?
                != digest("project-entity-result-set:", payload)?.as_ref()
            || text_field(payload, "schema")? != "wow-search/project-entity-result-set/1"
            || text_field(payload, "query_profile")? != QUERY_PROFILE
            || text_field(payload, "result_query_id")? != query_id.as_ref()
            || text_field(payload, "universe_id")? != universe.as_ref()
            || text_field(payload, "shard_id")? != self.logical_id()
            || text_field(payload, "graph_registry_digest")?
                != self.manifest.graph_registry_digest.as_ref()
            || payload.get("owner")
                != Some(
                    &serde_json::to_value(&self.manifest.owner)
                        .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
                )
            || payload.get("request")
                != Some(
                    &serde_json::to_value(request)
                        .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
                )
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let rows = payload
            .get("candidates")
            .and_then(serde_json::Value::as_array)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if rows.len() > request.max_candidates.get() as usize {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut previous: Option<([u32; 3], &str)> = None;
        for candidate in rows {
            check(stop)?;
            let id = text_field(candidate, "stable_tie_key")?;
            let index = self
                .manifest
                .documents
                .binary_search_by(|document| document.entity_id().as_str().cmp(id))
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
            let document = &self.manifest.documents[index];
            let rank: [u32; 3] = serde_json::from_value(
                candidate
                    .get("rank_tuple")
                    .cloned()
                    .ok_or(SearchError::OwnerEvidenceInvalid)?,
            )
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
            if !ids.insert(id)
                || previous.is_some_and(|old| old >= (rank, id))
                || candidate.get("document")
                    != Some(
                        &serde_json::to_value(document)
                            .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
                    )
                || text_field(candidate, "candidate_id")?
                    != digest(
                        "project-entity-candidate:",
                        &(&query_id, document.entity_id()),
                    )?
                    .as_ref()
            {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            previous = Some((rank, id));
        }
        render(bytes, self.profile_digest(), policy, cursor, stop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn captured_pages_preserve_whole_records_summary_and_cumulative_budget() -> SearchResult<()> {
        let stop = AtomicBool::new(false);
        let policy_json = json!({"schema":POLICY_SCHEMA,"page_size":1,"max_pages":3,
            "max_delivered_candidates":3,"max_delivered_candidate_bytes":65536,
            "max_delivered_page_bytes":65536,"max_replayed_manifest_bytes":65536});
        let encode = |value: &serde_json::Value| {
            wow_core::canonical_json_bytes(value).map_err(|_| SearchError::OwnerEvidenceInvalid)
        };
        let policy = EntityPagePolicy::from_json_slice(&encode(&policy_json)?, &stop)?;
        let candidates = json!([{"candidate_id":"one","graph_path":{"seed":"recorded","steps":[{"edge":"original"}]}},
            {"candidate_id":"two","confidence":"possible"}]);
        let envelope = json!({"result_set_id":"recorded-result", "payload":{
            "result_query_id":"recorded-query","universe_id":"recorded-universe","shard_id":"recorded-shard",
            "status":"candidate_only","explanations":[{"subject":"original","coverage":"partial"}],"candidates":candidates}});
        // Page mechanics consume an already computed Search representation;
        // these records are test data and never become Graph owner authority.
        let result = EntityQueryResult {
            bytes: encode(&envelope)?.into_boxed_slice(),
            status: "candidate_only",
            profile_digest: "recorded-profile".into(),
        };
        let first = result.page(&policy, None, &stop)?;
        let cursor = first
            .next_cursor()
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let second = result.page(&policy, Some(cursor), &stop)?;
        let first_json: serde_json::Value = serde_json::from_slice(first.canonical_bytes())
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let second_json: serde_json::Value = serde_json::from_slice(second.canonical_bytes())
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        assert_eq!(first_json["candidates"][0], candidates[0]);
        assert_eq!(second_json["candidates"][0], candidates[1]);
        assert_eq!(
            second_json["result_summary"]["explanations"],
            envelope["payload"]["explanations"]
        );
        assert_eq!(second_json["cumulative_budget"]["pages"], 2);
        assert_eq!(second_json["cumulative_budget"]["delivered_candidates"], 2);
        assert_eq!(
            second_json["cumulative_budget"]["replayed_manifest_bytes"],
            result.bytes.len() as u64 * 2
        );
        assert_eq!(
            second_json["cumulative_budget"]["delivered_page_bytes"],
            (first.canonical_bytes().len() + second.canonical_bytes().len()) as u64
        );
        assert!(second.next_cursor().is_none());
        let mut tampered = cursor.clone();
        tampered.budget.pages = 0;
        assert!(matches!(
            result.page(&policy, Some(&tampered), &stop),
            Err(SearchError::OwnerEvidenceInvalid)
        ));
        let mut limited = policy.clone();
        limited.max_pages = NonZeroU32::new(1).ok_or(SearchError::InvalidProfile)?;
        let limited_first = result.page(&limited, None, &stop)?;
        assert!(matches!(
            limited_first
                .next_cursor()
                .ok_or(SearchError::OwnerEvidenceInvalid)?
                .check_replay_budget(&limited, result.bytes.len() as u64),
            Err(SearchError::FileSetBudgetExceeded)
        ));
        assert!(matches!(
            result.page(&policy, None, &AtomicBool::new(true)),
            Err(SearchError::Cancelled)
        ));
        Ok(())
    }
}

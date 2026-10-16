//! Whole-candidate pages of an immutable Search capture, never native Graph truth.
use super::*;
use crate::page_budget::{PageBudget, PageLimits};
const POLICY_SCHEMA: &str = "wow-search/federated-page-policy/1";
const PAGE_SCHEMA: &str = "wow-search/federated-result-page/1";
const PAGE_PROFILE: &str = "wow-search/federated-captured-whole-candidate/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FederationPagePolicy {
    schema: String,
    page_size: NonZeroU32,
    max_pages: NonZeroU32,
    max_delivered_candidates: NonZeroU32,
    max_delivered_candidate_bytes: NonZeroU64,
    max_delivered_page_bytes: NonZeroU64,
    max_replayed_manifest_bytes: NonZeroU64,
}
impl FederationPagePolicy {
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
pub struct FederationPageCursor {
    cursor_id: Box<str>,
    result_set_id: Box<str>,
    query_id: Box<str>,
    universe_set_id: Box<str>,
    page_policy: FederationPagePolicy,
    next_candidate: u32,
    budget: PageBudget,
}
impl FederationPageCursor {
    pub fn check_replay_budget(
        &self,
        policy: &FederationPagePolicy,
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
            "federated-page-cursor:",
            &(
                PAGE_PROFILE,
                &self.result_set_id,
                &self.query_id,
                &self.universe_set_id,
                &self.page_policy,
                self.next_candidate,
                &self.budget,
            ),
        )
    }
}
pub struct FederationPage {
    bytes: Box<[u8]>,
    next_cursor: Option<FederationPageCursor>,
}
impl FederationPage {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn next_cursor(&self) -> Option<&FederationPageCursor> {
        self.next_cursor.as_ref()
    }
}

// Service authorizes a retained capture using its prior immutable session/receipt.
fn render(
    bytes: &[u8],
    policy: &FederationPagePolicy,
    cursor: Option<&FederationPageCursor>,
    stop: &AtomicBool,
) -> SearchResult<FederationPage> {
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
    let query_id = text_field(&summary, "query_id")?
        .to_owned()
        .into_boxed_str();
    let universe_set_id = text_field(&summary, "universe_set_id")?
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
    let total = u32::try_from(candidates.len()).map_err(|_| SearchError::FileSetBudgetExceeded)?;
    let (first, budget) = if let Some(cursor) = cursor {
        if cursor.cursor_id != cursor.id()?
            || cursor.result_set_id != result_id
            || cursor.query_id != query_id
            || cursor.universe_set_id != universe_set_id
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
            let mut cursor = FederationPageCursor {
                cursor_id: "".into(),
                result_set_id: result_id.clone(),
                query_id: query_id.clone(),
                universe_set_id: universe_set_id.clone(),
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
            page_policy: &'a FederationPagePolicy,
            cumulative_budget: &'a PageBudget,
            #[serde(skip_serializing_if = "Option::is_none")]
            next_cursor: Option<&'a FederationPageCursor>,
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
            return Ok(FederationPage {
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

fn size<T: Serialize + ?Sized>(value: &T) -> SearchResult<u64> {
    wow_core::canonical_json_bytes(value)
        .map(|bytes| bytes.len() as u64)
        .map_err(|_| SearchError::OwnerEvidenceInvalid)
}
impl FederationQueryResult {
    pub fn page(
        &self,
        policy: &FederationPagePolicy,
        cursor: Option<&FederationPageCursor>,
        stop: &AtomicBool,
    ) -> SearchResult<FederationPage> {
        render(&self.bytes, policy, cursor, stop)
    }
}

/// Pages authenticated captured Search data only. The service must prove the
/// exact prior completion/session/retention receipts before invoking this port.
/// It never produces a reconstructed sealed query result or native owner.
#[allow(clippy::too_many_arguments)]
pub fn page_retained_federation(
    primary: &crate::entities::EntitySearchShard,
    reference: &crate::reference::ReferenceSearchShard,
    bytes: &[u8],
    request: &FederationQueryRequest,
    policy: &FederationPagePolicy,
    cursor: Option<&FederationPageCursor>,
    stop: &AtomicBool,
) -> SearchResult<FederationPage> {
    page_retained_impl(
        RetainedSourceShard::Entities(primary),
        None,
        reference,
        bytes,
        request,
        policy,
        cursor,
        stop,
    )
}
#[allow(clippy::too_many_arguments)]
pub fn page_retained_federation_with_blizzard(
    primary: &crate::entities::EntitySearchShard,
    blizzard: &crate::entities::EntitySearchShard,
    reference: &crate::reference::ReferenceSearchShard,
    bytes: &[u8],
    request: &FederationQueryRequest,
    policy: &FederationPagePolicy,
    cursor: Option<&FederationPageCursor>,
    stop: &AtomicBool,
) -> SearchResult<FederationPage> {
    page_retained_impl(
        RetainedSourceShard::Entities(primary),
        Some(RetainedSourceShard::Entities(blizzard)),
        reference,
        bytes,
        request,
        policy,
        cursor,
        stop,
    )
}
/// Validate a captured result against finite exact source shard kinds.
#[allow(clippy::too_many_arguments)]
pub fn page_retained_source_federation(
    primary: RetainedSourceShard<'_>,
    blizzard: Option<RetainedSourceShard<'_>>,
    reference: &crate::reference::ReferenceSearchShard,
    bytes: &[u8],
    request: &FederationQueryRequest,
    policy: &FederationPagePolicy,
    cursor: Option<&FederationPageCursor>,
    stop: &AtomicBool,
) -> SearchResult<FederationPage> {
    page_retained_impl(
        primary, blizzard, reference, bytes, request, policy, cursor, stop,
    )
}
#[allow(clippy::too_many_arguments)]
fn page_retained_impl(
    primary: RetainedSourceShard<'_>,
    blizzard: Option<RetainedSourceShard<'_>>,
    reference: &crate::reference::ReferenceSearchShard,
    bytes: &[u8],
    request: &FederationQueryRequest,
    policy: &FederationPagePolicy,
    cursor: Option<&FederationPageCursor>,
    stop: &AtomicBool,
) -> SearchResult<FederationPage> {
    check(stop)?;
    request.validate()?;
    if request.requires_blizzard() != blizzard.is_some()
        || primary.uses_observations() != request.uses_observations()
        || blizzard.is_some_and(|b| b.uses_observations() != request.uses_observations())
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    if bytes.len() as u64 > request.max_result_bytes.get() {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    let envelope: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let payload = field(&envelope, "payload")?;
    if envelope.as_object().is_none_or(|object| object.len() != 2)
        || wow_core::canonical_json_bytes(&envelope)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?
            != bytes
        || text_field(&envelope, "result_set_id")?
            != digest("federated-search-result-set:", payload)?.as_ref()
        || text_field(payload, "schema")? != "wow-search/federated-result-set/1"
        || text_field(payload, "federation_profile")? != request.profile()
        || text_field(payload, "ranking_profile")? != request.ranking_profile()
        || field(payload, "request")?
            != &serde_json::to_value(request).map_err(|_| SearchError::OwnerEvidenceInvalid)?
        || text_field(payload, "privacy_scope")? != "local_metadata_only"
        || text_field(payload, "license_state")? != "not_evaluated"
        || field(payload, "absence_authoritative")?.as_bool() != Some(false)
        || field(payload, "blizzard_ui")?
            != &serde_json::json!({"state":if request.requires_blizzard(){"required_exact"}else{"excluded_by_request"}})
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let components = field(payload, "components")?;
    let primary_capture = field(components, "primary_user_project")?;
    let reference_capture = field(components, "reference_platform_contract")?;
    let p = field(primary_capture, "result_summary")?;
    let r = field(reference_capture, "result_summary")?;
    let b_capture = if blizzard.is_some() {
        Some(field(components, "blizzard_ui_source")?)
    } else {
        None
    };
    let b = b_capture
        .map(|capture| field(capture, "result_summary"))
        .transpose()?;
    primary.validate_federation_summary(p, stop)?;
    if let (Some(shard), Some(summary)) = (blizzard, b) {
        shard.validate_federation_summary(summary, stop)?;
    }
    let reference_request = reference.validate_federation_summary(r, stop)?;
    if p.get("candidates").is_some()
        || r.get("candidates").is_some()
        || text_field(primary_capture, "candidate_delivery")?
            != "selected_federated_candidates_only"
        || text_field(reference_capture, "candidate_delivery")?
            != "selected_federated_candidates_only"
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let p_count = number(primary_capture, "source_candidate_count")?;
    let r_count = number(reference_capture, "source_candidate_count")?;
    let b_count = b_capture
        .map(|capture| number(capture, "source_candidate_count"))
        .transpose()?
        .unwrap_or(0);
    if let Some(capture) = b_capture
        && (text_field(capture, "candidate_delivery")? != "selected_federated_candidates_only"
            || field(capture, "result_summary")?
                .get("candidates")
                .is_some())
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let total = p_count
        .checked_add(r_count)
        .and_then(|count| count.checked_add(b_count))
        .filter(|total| *total <= u64::from(request.max_input_candidates.get()))
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    let p_owner = field(field(p, "owner")?, "source")?;
    let r_owner = field(r, "owner")?;
    if field(p_owner, "project_profile")? != field(field(r_owner, "binding")?, "profile")?
        || field(p_owner, "reference_generation_id")? != field(r_owner, "reference_generation")?
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    if let Some(b) = b {
        let owner = field(field(b, "owner")?, "source")?;
        if field(owner, "project_profile")? != field(p_owner, "project_profile")?
            || field(owner, "reference_generation_id")?
                != field(p_owner, "reference_generation_id")?
            || field(owner, "graph_universe_id")? == field(p_owner, "graph_universe_id")?
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
    }
    let bindings = field(payload, "universes")?
        .as_array()
        .filter(|rows| rows.len() == if request.requires_blizzard() { 3 } else { 2 })
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    let kind = text_field(&bindings[0], "project_kind")?;
    if !matches!(kind, "repository" | "fixture")
        || request.uses_observations() && text_field(p, "project_kind")? != kind
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let mut expected = vec![
        serde_json::json!({"project_kind":kind,"universe_class":"primary_user_project","universe_id":text_field(p,"universe_id")?,"shard_id":text_field(p,"shard_id")?,"component_profile":text_field(p,"query_profile")?}),
    ];
    if let Some(b) = b {
        let binding = &bindings[1];
        let profile = field(binding, "platform_profile")?;
        let package = field(binding, "platform_binding")?;
        if request.uses_observations()
            && (text_field(b, "project_kind")? != "blizzard_ui_platform_source"
                || field(b, "platform_source_profile")? != profile
                || field(b, "platform_package_binding")? != package)
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let target = field(field(profile, "request")?, "target")?;
        if text_field(profile, "schema")? != "wow-project/platform-source-profile/1"
            || text_field(package, "schema")? != "wow-project/platform-package-binding/1"
            || field(package, "profile_digest")? != field(profile, "digest")?
            || field(package, "target")? != target
            || field(target, "reference_profile")? != field(p_owner, "project_profile")?
            || field(target, "reference_generation")? != field(p_owner, "reference_generation_id")?
            || !matches!(
                (
                    text_field(field(profile, "request")?, "source_class")?,
                    text_field(field(target, "reference_profile")?, "profile_kind")?
                ),
                ("synthetic_fixture", "fixture") | ("vendor_ui_source_mirror", "release")
            )
            || text_field(package, "source_snapshot_id")?.is_empty()
            || text_field(package, "universe_id")?.is_empty()
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        expected.push(serde_json::json!({"project_kind":"blizzard_ui_platform_source","platform_profile":profile,"platform_binding":package,
            "universe_class":"blizzard_ui_source","universe_id":text_field(b,"universe_id")?,"shard_id":text_field(b,"shard_id")?,"component_profile":text_field(b,"query_profile")?}));
    }
    expected.push(serde_json::json!({"universe_class":"reference_platform_contract","universe_id":text_field(r,"universe_id")?,"shard_id":text_field(r,"shard_id")?,"component_profile":text_field(r,"query_profile")?}));
    if field(payload, "universes")? != &serde_json::Value::Array(expected) {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let universe = digest(
        "search-universe-set:",
        &(
            request.profile(),
            bindings,
            field(p_owner, "project_profile")?,
            field(p_owner, "reference_generation_id")?,
            if request.requires_blizzard() {
                "blizzard_ui_required_exact"
            } else {
                "blizzard_ui_excluded_by_request"
            },
        ),
    )?;
    let query_id = if let Some(b) = b {
        digest(
            "federated-search-query:",
            &(
                request.profile(),
                &universe,
                text_field(p, "result_query_id")?,
                text_field(b, "result_query_id")?,
                text_field(r, "normalized_query_id")?,
                request,
            ),
        )?
    } else {
        digest(
            "federated-search-query:",
            &(
                if request.uses_observations() {
                    request.profile()
                } else {
                    PROFILE
                },
                &universe,
                text_field(p, "result_query_id")?,
                text_field(r, "normalized_query_id")?,
                request,
            ),
        )?
    };
    if text_field(payload, "universe_set_id")? != universe.as_ref()
        || text_field(payload, "query_id")? != query_id.as_ref()
        || number(payload, "rr_scale")? != 1_000_000
        || number(payload, "rr_offset")? != 60
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let candidates = rows(payload)?;
    let retained = total.min(u64::from(request.max_candidates.get()));
    if candidates.len() as u64 != retained
        || number(payload, "candidate_signals_omitted_by_cap")? != total - retained
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let mut seen = BTreeSet::new();
    let mut previous = None;
    for candidate in candidates {
        check(stop)?;
        let class = match text_field(candidate, "universe_class")? {
            "primary_user_project" => FederationUniverse::PrimaryUserProject,
            "blizzard_ui_source" => FederationUniverse::BlizzardUiSource,
            "reference_platform_contract" => FederationUniverse::ReferencePlatformContract,
            _ => return Err(SearchError::OwnerEvidenceInvalid),
        };
        let (summary, capture, count) = if class == FederationUniverse::PrimaryUserProject {
            (p, primary_capture, p_count)
        } else if class == FederationUniverse::BlizzardUiSource {
            (
                b.ok_or(SearchError::OwnerEvidenceInvalid)?,
                b_capture.ok_or(SearchError::OwnerEvidenceInvalid)?,
                b_count,
            )
        } else {
            (r, reference_capture, r_count)
        };
        let original = field(candidate, "original_candidate")?;
        let component_query = if class != FederationUniverse::ReferencePlatformContract {
            text_field(summary, "result_query_id")?
        } else {
            text_field(summary, "normalized_query_id")?
        };
        let component_universe = text_field(summary, "universe_id")?;
        if class == FederationUniverse::PrimaryUserProject {
            primary.validate_federation_candidate(component_query, original, stop)?;
        } else if class == FederationUniverse::BlizzardUiSource {
            blizzard
                .ok_or(SearchError::OwnerEvidenceInvalid)?
                .validate_federation_candidate(component_query, original, stop)?;
        } else {
            reference.validate_federation_candidate(
                component_query,
                component_universe,
                original,
                &reference_request,
                stop,
            )?;
        }
        let band = field(original, "rank_tuple")?
            .as_array()
            .and_then(|rows| rows.first())
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .filter(|band| *band <= 10)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let (effective, cap) = effective_band(request, class, original, band)?;
        let ordinal = number(candidate, "component_candidate_ordinal")?;
        if ordinal == 0 || ordinal > count {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let ordinal = u32::try_from(ordinal).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let rr = 1_000_000
            / (60u32
                .checked_add(ordinal)
                .ok_or(SearchError::OwnerEvidenceInvalid)?);
        let priority = request
            .universe_priority
            .iter()
            .position(|value| *value == class)
            .ok_or(SearchError::InvalidProfile)? as u32;
        let rank = [effective, priority, ordinal, u32::MAX - rr];
        let id = digest(
            "federated-search-candidate:",
            &(
                &query_id,
                component_universe,
                text_field(original, "candidate_id")?,
            ),
        )?;
        let stable = text_field(original, "stable_tie_key")?;
        if text_field(candidate, "candidate_id")? != id.as_ref()
            || !seen.insert(text_field(candidate, "candidate_id")?)
            || text_field(candidate, "universe_id")? != component_universe
            || text_field(candidate, "shard_id")? != text_field(summary, "shard_id")?
            || text_field(candidate, "component_result_set_id")?
                != text_field(capture, "source_result_set_id")?
            || number(candidate, "source_authority_band")? != u64::from(band)
            || number(candidate, "effective_authority_band")? != u64::from(effective)
            || text_field(candidate, "authority_cap")? != cap
            || number(candidate, "reciprocal_rank_contribution")? != u64::from(rr)
            || field(candidate, "rank_tuple")? != &serde_json::json!(rank)
            || text_field(candidate, "stable_tie_key")? != stable
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let order = (
            rank,
            component_universe,
            stable,
            text_field(candidate, "candidate_id")?,
        );
        if previous.is_some_and(|old| old >= order) {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        previous = Some(order);
    }
    render(bytes, policy, cursor, stop)
}
fn number(value: &serde_json::Value, key: &str) -> SearchResult<u64> {
    field(value, key)?
        .as_u64()
        .ok_or(SearchError::OwnerEvidenceInvalid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn frozen_federated_pages_keep_whole_records_and_exact_budget_chain() -> SearchResult<()> {
        let stop = AtomicBool::new(false);
        let policy_value = json!({"schema":POLICY_SCHEMA,"page_size":1,"max_pages":3,"max_delivered_candidates":3,
            "max_delivered_candidate_bytes":65536,"max_delivered_page_bytes":65536,"max_replayed_manifest_bytes":65536});
        let encode = |value: &serde_json::Value| {
            wow_core::canonical_json_bytes(value).map_err(|_| SearchError::OwnerEvidenceInvalid)
        };
        let policy = FederationPagePolicy::from_json_slice(&encode(&policy_value)?, &stop)?;
        let records = json!([{"universe":"primary","original_candidate":{"graph_path":{"steps":["recorded-original-edge"]}}},
            {"universe":"reference","original_candidate":{"confidence":"possible"}}]);
        let envelope = json!({"result_set_id":"recorded-result","payload":{"query_id":"recorded-query","universe_set_id":"recorded-set",
            "components":{"recorded":{"coverage":"partial"}},"status":"candidate_only","candidates":records}});
        // Data-only rendering fixture; no native Graph/Reference authority.
        let result = FederationQueryResult {
            bytes: encode(&envelope)?.into_boxed_slice(),
            status: "candidate_only",
        };
        let first = result.page(&policy, None, &stop)?;
        let cursor = first
            .next_cursor()
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let next = result.page(&policy, Some(cursor), &stop)?;
        let first_value: serde_json::Value = serde_json::from_slice(first.canonical_bytes())
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let next_value: serde_json::Value = serde_json::from_slice(next.canonical_bytes())
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        assert_eq!(first_value["candidates"][0], records[0]);
        assert_eq!(next_value["candidates"][0], records[1]);
        assert_eq!(
            next_value["result_summary"]["components"],
            envelope["payload"]["components"]
        );
        assert_eq!(next_value["cumulative_budget"]["pages"], 2);
        assert_eq!(
            next_value["cumulative_budget"]["replayed_manifest_bytes"],
            result.bytes.len() as u64 * 2
        );
        assert_eq!(
            next_value["cumulative_budget"]["delivered_page_bytes"],
            (first.canonical_bytes().len() + next.canonical_bytes().len()) as u64
        );
        assert!(next.next_cursor().is_none());
        let mut changed = cursor.clone();
        changed.budget.pages = 0;
        assert!(matches!(
            result.page(&policy, Some(&changed), &stop),
            Err(SearchError::OwnerEvidenceInvalid)
        ));
        changed = cursor.clone();
        changed.universe_set_id = "other-universe-set".into();
        assert!(matches!(
            result.page(&policy, Some(&changed), &stop),
            Err(SearchError::OwnerEvidenceInvalid)
        ));
        assert!(matches!(
            result.page(&policy, None, &AtomicBool::new(true)),
            Err(SearchError::Cancelled)
        ));
        Ok(())
    }
}

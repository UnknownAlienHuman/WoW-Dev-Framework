//! Normal exact primary/Reference binding over genuine Search-owned results.
//! No native owner reconstruction, same-name collapse or cross-shard FTS scores.
use super::*;
use crate::{entities::EntityQueryResult, reference::ReferenceQueryResult};
use std::collections::BTreeSet;
mod observation_bands;
mod pagination;
pub use pagination::{
    FederationPage, FederationPageCursor, FederationPagePolicy, page_retained_federation,
    page_retained_federation_with_blizzard, page_retained_source_federation,
};

const REQUEST_SCHEMA: &str = "wow-search/exact-primary-reference-query/1";
const PROFILE: &str = "wow-search/exact-primary-reference-federation/1";
const PLATFORM_REQUEST_SCHEMA: &str = "wow-search/exact-primary-platform-reference-query/1";
const PLATFORM_PROFILE: &str = "wow-search/exact-primary-platform-reference-federation/1";
const RANK_PROFILE: &str = "wow-search/federated-authority-universe-ordinal/1";
const OBSERVATION_REQUEST_SCHEMA: &str = "wow-search/exact-observed-source-reference-query/1";
const OBSERVATION_PLATFORM_REQUEST_SCHEMA: &str =
    "wow-search/exact-observed-source-platform-reference-query/1";
const OBSERVATION_PROFILE: &str = "wow-search/exact-observed-source-reference-federation/1";
const OBSERVATION_PLATFORM_PROFILE: &str =
    "wow-search/exact-observed-source-platform-reference-federation/1";
const OBSERVATION_RANK_PROFILE: &str =
    "wow-search/federated-observed-source-authority-universe-ordinal/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FederationUniverse {
    PrimaryUserProject,
    BlizzardUiSource,
    ReferencePlatformContract,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BlizzardSelection {
    Excluded,
    Required,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FederationQueryRequest {
    schema: String,
    blizzard_ui: BlizzardSelection,
    universe_priority: Vec<FederationUniverse>,
    max_component_result_bytes: NonZeroU64,
    max_input_candidates: NonZeroU32,
    max_candidates: NonZeroU32,
    max_result_bytes: NonZeroU64,
}
impl FederationQueryRequest {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> SearchResult<Self> {
        check(stop)?;
        if bytes.len() > 4096 {
            return Err(SearchError::InvalidProfile);
        }
        let request: Self =
            serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
        request.validate()?;
        Ok(request)
    }
    fn validate(&self) -> SearchResult<()> {
        let platform = self.requires_blizzard();
        let expected = if platform { 3 } else { 2 };
        if self.schema
            != if self.uses_observations() && platform {
                OBSERVATION_PLATFORM_REQUEST_SCHEMA
            } else if self.uses_observations() {
                OBSERVATION_REQUEST_SCHEMA
            } else if platform {
                PLATFORM_REQUEST_SCHEMA
            } else {
                REQUEST_SCHEMA
            }
            || self.universe_priority.len() != expected
            || self
                .universe_priority
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                != expected
            || (!platform
                && self
                    .universe_priority
                    .contains(&FederationUniverse::BlizzardUiSource))
        {
            return Err(SearchError::InvalidProfile);
        }
        Ok(())
    }
    pub fn requires_blizzard(&self) -> bool {
        matches!(self.blizzard_ui, BlizzardSelection::Required)
    }
    pub fn uses_observations(&self) -> bool {
        matches!(
            self.schema.as_str(),
            OBSERVATION_REQUEST_SCHEMA | OBSERVATION_PLATFORM_REQUEST_SCHEMA
        )
    }
    fn source_schema(&self, value: &str) -> bool {
        if self.uses_observations() {
            matches!(
                value,
                "wow-search/source-observation-result-set/1"
                    | "wow-search/source-observation-declaration-result-set/1"
            )
        } else {
            value == "wow-search/project-entity-result-set/1"
        }
    }
    fn ranking_profile(&self) -> &'static str {
        if self.uses_observations() {
            OBSERVATION_RANK_PROFILE
        } else {
            RANK_PROFILE
        }
    }
    fn profile(&self) -> &'static str {
        if self.uses_observations() && self.requires_blizzard() {
            OBSERVATION_PLATFORM_PROFILE
        } else if self.uses_observations() {
            OBSERVATION_PROFILE
        } else if self.requires_blizzard() {
            PLATFORM_PROFILE
        } else {
            PROFILE
        }
    }
}
pub struct FederationQueryResult {
    bytes: Box<[u8]>,
    status: &'static str,
}
impl FederationQueryResult {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn status(&self) -> &'static str {
        self.status
    }
}
#[derive(Serialize)]
struct Binding<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_profile: Option<&'a wow_project::platform_source::BlizzardUiSourceProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_binding: Option<&'a wow_project::platform_source::PlatformPackageBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_kind: Option<wow_project::ProjectKind>,
    universe_class: FederationUniverse,
    universe_id: &'a str,
    shard_id: &'a str,
    component_profile: &'a str,
}
#[derive(Serialize)]
struct Candidate<'a> {
    candidate_id: Box<str>,
    universe_class: FederationUniverse,
    universe_id: &'a str,
    shard_id: &'a str,
    component_result_set_id: &'a str,
    component_candidate_ordinal: u32,
    source_authority_band: u32,
    effective_authority_band: u32,
    authority_cap: &'static str,
    reciprocal_rank_contribution: u32,
    rank_tuple: [u32; 4],
    stable_tie_key: &'a str,
    original_candidate: &'a serde_json::Value,
}

/// Both inputs are sealed outputs of their genuine owning Search query ports.
/// This method cannot admit arbitrary serialized reports as native owner truth.
pub fn federate(
    primary: &EntityQueryResult,
    reference: &ReferenceQueryResult,
    request: &FederationQueryRequest,
    stop: &AtomicBool,
) -> SearchResult<FederationQueryResult> {
    check(stop)?;
    request.validate()?;
    if request.requires_blizzard() || request.uses_observations() {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let bytes = primary
        .canonical_bytes()
        .len()
        .checked_add(reference.canonical_bytes().len())
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    if bytes as u64 > request.max_component_result_bytes.get() {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    let primary_kind = primary.project_kind();
    let primary: serde_json::Value = serde_json::from_slice(primary.canonical_bytes())
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let reference: serde_json::Value = serde_json::from_slice(reference.canonical_bytes())
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    fuse(&primary, &reference, primary_kind, request, stop)
}
pub fn federate_with_blizzard(
    primary: &EntityQueryResult,
    blizzard: &EntityQueryResult,
    reference: &ReferenceQueryResult,
    request: &FederationQueryRequest,
    stop: &AtomicBool,
) -> SearchResult<FederationQueryResult> {
    check(stop)?;
    request.validate()?;
    if request.uses_observations()
        || !request.requires_blizzard()
        || blizzard.project_kind() != wow_project::ProjectKind::BlizzardUiPlatformSource
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let (platform_profile, platform_binding) = blizzard
        .platform_proof()
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    platform_profile
        .validate()
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    if platform_binding.profile_digest() != platform_profile.digest()
        || platform_binding.target() != platform_profile.target()
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let size = primary
        .canonical_bytes()
        .len()
        .checked_add(blizzard.canonical_bytes().len())
        .and_then(|bytes| bytes.checked_add(reference.canonical_bytes().len()))
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    if size as u64 > request.max_component_result_bytes.get() {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    let p: serde_json::Value = serde_json::from_slice(primary.canonical_bytes())
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let b: serde_json::Value = serde_json::from_slice(blizzard.canonical_bytes())
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    let r: serde_json::Value = serde_json::from_slice(reference.canonical_bytes())
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    fuse_components(
        &p,
        &r,
        primary.project_kind(),
        Some(BlizzardCapture {
            capture: &b,
            profile: platform_profile,
            binding: platform_binding,
        }),
        request,
        stop,
    )
}
struct BlizzardCapture<'a> {
    capture: &'a serde_json::Value,
    profile: &'a wow_project::platform_source::BlizzardUiSourceProfile,
    binding: &'a wow_project::platform_source::PlatformPackageBinding,
}
/// Initial observed-source fusion requires actual acquired Project bindings.
pub fn federate_observations(
    primary: &crate::source_observations::BoundObservationQueryResult,
    blizzard: Option<&crate::source_observations::BoundObservationQueryResult>,
    reference: &ReferenceQueryResult,
    request: &FederationQueryRequest,
    stop: &AtomicBool,
) -> SearchResult<FederationQueryResult> {
    check(stop)?;
    request.validate()?;
    if !request.uses_observations() || request.requires_blizzard() != blizzard.is_some() {
        return Err(SearchError::InvalidProfile);
    }
    let size = primary
        .canonical_bytes()
        .len()
        .checked_add(reference.canonical_bytes().len())
        .and_then(|n| n.checked_add(blizzard.map_or(0, |b| b.canonical_bytes().len())))
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    if size as u64 > request.max_component_result_bytes.get() {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    let parse = |bytes| {
        serde_json::from_slice::<serde_json::Value>(bytes)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)
    };
    let p = parse(primary.canonical_bytes())?;
    let r = parse(reference.canonical_bytes())?;
    let b = blizzard.map(|b| parse(b.canonical_bytes())).transpose()?;
    let captured = if let (Some(owner), Some(b)) = (blizzard, b.as_ref()) {
        if owner.project_kind() != wow_project::ProjectKind::BlizzardUiPlatformSource {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let (profile, binding) = owner
            .platform_proof()
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        profile
            .validate()
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if binding.profile_digest() != profile.digest() || binding.target() != profile.target() {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        Some(BlizzardCapture {
            capture: b,
            profile,
            binding,
        })
    } else {
        None
    };
    fuse_components(&p, &r, primary.project_kind(), captured, request, stop)
}
fn effective_band(
    request: &FederationQueryRequest,
    class: FederationUniverse,
    candidate: &serde_json::Value,
    source_band: u32,
) -> SearchResult<(u32, &'static str)> {
    if class != FederationUniverse::ReferencePlatformContract {
        let match_class = text_field(candidate, "match_class")?;
        if request.uses_observations() {
            return observation_bands::authority_band(match_class, source_band);
        }
        if match_class == "exact_recorded_owner_key" {
            return Ok((
                source_band.max(4),
                "recorded_owner_key_is_not_canonical_name",
            ));
        }
    }
    Ok((source_band, "original_component_band"))
}

/// A finite view of an already validated immutable Search shard, not a native owner.
#[derive(Clone, Copy)]
pub enum RetainedSourceShard<'a> {
    Entities(&'a crate::entities::EntitySearchShard),
    Observations(&'a crate::source_observations::ObservationSearchShard),
}
impl RetainedSourceShard<'_> {
    fn uses_observations(self) -> bool {
        matches!(self, Self::Observations(_))
    }
    fn validate_federation_summary(
        self,
        summary: &serde_json::Value,
        stop: &AtomicBool,
    ) -> SearchResult<()> {
        match self {
            Self::Entities(s) => s.validate_federation_summary(summary, stop),
            Self::Observations(s) => s.validate_federation_summary(summary, stop),
        }
    }
    fn validate_federation_candidate(
        self,
        query: &str,
        candidate: &serde_json::Value,
        stop: &AtomicBool,
    ) -> SearchResult<()> {
        match self {
            Self::Entities(s) => s.validate_federation_candidate(query, candidate, stop),
            Self::Observations(s) => s.validate_federation_candidate(query, candidate, stop),
        }
    }
}
fn field<'a>(value: &'a serde_json::Value, name: &str) -> SearchResult<&'a serde_json::Value> {
    value.get(name).ok_or(SearchError::OwnerEvidenceInvalid)
}
fn text_field<'a>(value: &'a serde_json::Value, name: &str) -> SearchResult<&'a str> {
    field(value, name)?
        .as_str()
        .ok_or(SearchError::OwnerEvidenceInvalid)
}
fn rows(value: &serde_json::Value) -> SearchResult<&[serde_json::Value]> {
    field(value, "candidates")?
        .as_array()
        .map(Vec::as_slice)
        .ok_or(SearchError::OwnerEvidenceInvalid)
}
fn digest<T: Serialize + ?Sized>(prefix: &str, value: &T) -> SearchResult<Box<str>> {
    let bytes =
        wow_core::canonical_json_bytes(value).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    Ok(format!("{prefix}sha256:{}", hex(&Sha256::digest(bytes))).into())
}
fn check(stop: &AtomicBool) -> SearchResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(SearchError::Cancelled)
    } else {
        Ok(())
    }
}
fn summary(capture: &serde_json::Value) -> SearchResult<serde_json::Value> {
    let payload = field(capture, "payload")?
        .as_object()
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    let retained = payload
        .iter()
        .filter(|(key, _)| key.as_str() != "candidates")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<serde_json::Map<_, _>>();
    Ok(
        serde_json::json!({"source_result_set_id":text_field(capture,"result_set_id")?,"source_candidate_count":rows(field(capture,"payload")?)?.len(),
        "result_summary":retained,"candidate_delivery":"selected_federated_candidates_only"}),
    )
}

fn fuse(
    primary: &serde_json::Value,
    reference: &serde_json::Value,
    primary_kind: wow_project::ProjectKind,
    request: &FederationQueryRequest,
    stop: &AtomicBool,
) -> SearchResult<FederationQueryResult> {
    fuse_components(primary, reference, primary_kind, None, request, stop)
}
fn fuse_components(
    primary: &serde_json::Value,
    reference: &serde_json::Value,
    primary_kind: wow_project::ProjectKind,
    blizzard: Option<BlizzardCapture<'_>>,
    request: &FederationQueryRequest,
    stop: &AtomicBool,
) -> SearchResult<FederationQueryResult> {
    if request.requires_blizzard() != blizzard.is_some() {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    if primary_kind == wow_project::ProjectKind::BlizzardUiPlatformSource {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let p = field(primary, "payload")?;
    let r = field(reference, "payload")?;
    if !request.source_schema(text_field(p, "schema")?)
        || text_field(r, "schema")? != "wow-search/native-reference-result-set/1"
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let source = field(field(p, "owner")?, "source")?;
    let reference_owner = field(r, "owner")?;
    let binding = field(reference_owner, "binding")?;
    // Normal E4 binding is exact. Neither same name/build nor a caller's
    // compatibility flag substitutes for the complete original profile/generation.
    if field(source, "project_profile")? != field(binding, "profile")?
        || field(source, "reference_generation_id")?
            != field(reference_owner, "reference_generation")?
        || text_field(reference_owner, "privacy_scope")? != "local_metadata_only"
        || text_field(reference_owner, "license_state")? != "not_evaluated"
    {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let mut bindings = vec![Binding {
        platform_profile: None,
        platform_binding: None,
        project_kind: Some(primary_kind),
        universe_class: FederationUniverse::PrimaryUserProject,
        universe_id: text_field(p, "universe_id")?,
        shard_id: text_field(p, "shard_id")?,
        component_profile: text_field(p, "query_profile")?,
    }];
    let mut captures = vec![(primary, p)];
    if let Some(blizzard) = &blizzard {
        let b = field(blizzard.capture, "payload")?;
        let b_source = field(field(b, "owner")?, "source")?;
        if !request.source_schema(text_field(b, "schema")?)
            || field(b_source, "project_profile")? != field(source, "project_profile")?
            || field(b_source, "reference_generation_id")?
                != field(source, "reference_generation_id")?
            || field(b_source, "graph_universe_id")? == field(source, "graph_universe_id")?
            || serde_json::to_value(&blizzard.profile.target().reference_profile)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?
                != *field(source, "project_profile")?
            || blizzard.profile.target().reference_generation.to_string()
                != text_field(source, "reference_generation_id")?
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        bindings.push(Binding {
            platform_profile: Some(blizzard.profile),
            platform_binding: Some(blizzard.binding),
            project_kind: Some(wow_project::ProjectKind::BlizzardUiPlatformSource),
            universe_class: FederationUniverse::BlizzardUiSource,
            universe_id: text_field(b, "universe_id")?,
            shard_id: text_field(b, "shard_id")?,
            component_profile: text_field(b, "query_profile")?,
        });
        captures.push((blizzard.capture, b));
    }
    bindings.push(Binding {
        platform_profile: None,
        platform_binding: None,
        project_kind: None,
        universe_class: FederationUniverse::ReferencePlatformContract,
        universe_id: text_field(r, "universe_id")?,
        shard_id: text_field(r, "shard_id")?,
        component_profile: text_field(r, "query_profile")?,
    });
    captures.push((reference, r));
    let universe_set_id = digest(
        "search-universe-set:",
        &(
            request.profile(),
            &bindings,
            field(source, "project_profile")?,
            field(source, "reference_generation_id")?,
            if request.requires_blizzard() {
                "blizzard_ui_required_exact"
            } else {
                "blizzard_ui_excluded_by_request"
            },
        ),
    )?;
    let query_id = if let Some(blizzard) = &blizzard {
        digest(
            "federated-search-query:",
            &(
                request.profile(),
                &universe_set_id,
                text_field(p, "result_query_id")?,
                text_field(field(blizzard.capture, "payload")?, "result_query_id")?,
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
                &universe_set_id,
                text_field(p, "result_query_id")?,
                text_field(r, "normalized_query_id")?,
                request,
            ),
        )?
    };
    let count = captures.iter().try_fold(0usize, |count, (_, payload)| {
        count
            .checked_add(rows(payload)?.len())
            .ok_or(SearchError::FileSetBudgetExceeded)
    })?;
    if count > request.max_input_candidates.get() as usize {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    let mut candidates = Vec::new();
    for (index, (capture, payload)) in captures.iter().copied().enumerate() {
        let binding = &bindings[index];
        let priority = request
            .universe_priority
            .iter()
            .position(|value| *value == binding.universe_class)
            .ok_or(SearchError::InvalidProfile)? as u32;
        let component_result = text_field(capture, "result_set_id")?;
        let mut seen = BTreeSet::new();
        for (ordinal, candidate) in rows(payload)?.iter().enumerate() {
            check(stop)?;
            let key = text_field(candidate, "stable_tie_key")?;
            if !seen.insert(key) {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            let source_band = field(candidate, "rank_tuple")?
                .as_array()
                .and_then(|rank| rank.first())
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
                .filter(|value| *value <= 10)
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            let (band, cap) =
                effective_band(request, binding.universe_class, candidate, source_band)?;
            let ordinal =
                u32::try_from(ordinal + 1).map_err(|_| SearchError::FileSetBudgetExceeded)?;
            let rr = 1_000_000
                / (60u32
                    .checked_add(ordinal)
                    .ok_or(SearchError::FileSetBudgetExceeded)?);
            candidates.push(Candidate {
                candidate_id: digest(
                    "federated-search-candidate:",
                    &(
                        &query_id,
                        binding.universe_id,
                        text_field(candidate, "candidate_id")?,
                    ),
                )?,
                universe_class: binding.universe_class,
                universe_id: binding.universe_id,
                shard_id: binding.shard_id,
                component_result_set_id: component_result,
                component_candidate_ordinal: ordinal,
                source_authority_band: source_band,
                effective_authority_band: band,
                authority_cap: cap,
                reciprocal_rank_contribution: rr,
                rank_tuple: [band, priority, ordinal, u32::MAX - rr],
                stable_tie_key: key,
                original_candidate: candidate,
            });
        }
    }
    candidates.sort_by(|a, b| {
        a.rank_tuple
            .cmp(&b.rank_tuple)
            .then_with(|| a.universe_id.cmp(b.universe_id))
            .then_with(|| a.stable_tie_key.cmp(b.stable_tie_key))
            .then_with(|| a.candidate_id.cmp(&b.candidate_id))
    });
    let omitted = candidates
        .len()
        .saturating_sub(request.max_candidates.get() as usize);
    candidates.truncate(request.max_candidates.get() as usize);
    let states = captures
        .iter()
        .map(|(_, payload)| text_field(payload, "status"))
        .collect::<SearchResult<Vec<_>>>()?;
    let status = if omitted > 0 || states.contains(&"truncated") {
        "truncated"
    } else if states.contains(&"lane_unavailable") {
        "lane_unavailable"
    } else if states.contains(&"conflict_blocked") {
        "conflict_blocked"
    } else if candidates
        .iter()
        .any(|candidate| candidate.effective_authority_band <= 4)
    {
        "exact_found"
    } else if !candidates.is_empty() {
        "candidate_only"
    } else if states
        .iter()
        .any(|state| state.starts_with("exact_not_found"))
    {
        "exact_not_found_partial"
    } else {
        "no_candidates_under_executed_lanes"
    };
    let mut payload = serde_json::json!({"schema":"wow-search/federated-result-set/1","federation_profile":request.profile(),"ranking_profile":request.ranking_profile(),
        "query_id":query_id,"universe_set_id":universe_set_id,"request":request,"universes":bindings,"status":status,
        "absence_authoritative":false,"blizzard_ui":{"state":"excluded_by_request"},"privacy_scope":"local_metadata_only","license_state":"not_evaluated",
        "candidate_signals_omitted_by_cap":omitted,"rr_scale":1000000,"rr_offset":60,"candidates":candidates,
        "components":{"primary_user_project":summary(primary)?,"reference_platform_contract":summary(reference)?}});
    if let Some(blizzard) = &blizzard {
        payload["blizzard_ui"] = serde_json::json!({"state":"required_exact"});
        payload["components"]["blizzard_ui_source"] = summary(blizzard.capture)?;
    }
    let result_set_id = digest("federated-search-result-set:", &payload)?;
    let bytes = wow_core::canonical_json_bytes(
        &serde_json::json!({"result_set_id":result_set_id,"payload":payload}),
    )
    .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    if bytes.len() as u64 > request.max_result_bytes.get() {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    check(stop)?;
    Ok(FederationQueryResult {
        bytes: bytes.into_boxed_slice(),
        status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn observed_source_schema_and_rank_interpretation_are_explicit() -> SearchResult<()> {
        let stop = AtomicBool::new(false);
        let mut input = json!({"schema":OBSERVATION_REQUEST_SCHEMA,"blizzard_ui":"excluded","universe_priority":["primary_user_project","reference_platform_contract"],
            "max_component_result_bytes":65536,"max_input_candidates":8,"max_candidates":8,"max_result_bytes":65536});
        let parse = |v: &serde_json::Value| {
            FederationQueryRequest::from_json_slice(
                &serde_json::to_vec(v).map_err(|_| SearchError::InvalidProfile)?,
                &stop,
            )
        };
        let request = parse(&input)?;
        assert!(request.uses_observations());
        assert_eq!(request.ranking_profile(), OBSERVATION_RANK_PROFILE);
        input["schema"] = json!(OBSERVATION_PLATFORM_REQUEST_SCHEMA);
        assert!(parse(&input).is_err());
        input["blizzard_ui"] = json!("required");
        input["universe_priority"] = json!([
            "primary_user_project",
            "blizzard_ui_source",
            "reference_platform_contract"
        ]);
        assert!(parse(&input)?.requires_blizzard());
        input["schema"] = json!(PLATFORM_REQUEST_SCHEMA);
        assert!(!parse(&input)?.uses_observations());
        Ok(())
    }
    #[test]
    fn third_universe_policy_is_explicit_closed_and_does_not_widen_legacy() -> SearchResult<()> {
        let stop = AtomicBool::new(false);
        let value = json!({"schema":PLATFORM_REQUEST_SCHEMA,"blizzard_ui":"required","universe_priority":["primary_user_project","blizzard_ui_source","reference_platform_contract"],
            "max_component_result_bytes":65536,"max_input_candidates":8,"max_candidates":8,"max_result_bytes":65536});
        let parse = |value: &serde_json::Value| {
            FederationQueryRequest::from_json_slice(
                &wow_core::canonical_json_bytes(value).map_err(|_| SearchError::InvalidProfile)?,
                &stop,
            )
        };
        assert!(parse(&value)?.requires_blizzard());
        let mut wrong = value.clone();
        wrong["schema"] = json!(REQUEST_SCHEMA);
        assert!(parse(&wrong).is_err());
        wrong = value.clone();
        wrong["blizzard_ui"] = json!("excluded");
        assert!(parse(&wrong).is_err());
        wrong = value.clone();
        wrong["universe_priority"] = json!(["primary_user_project", "reference_platform_contract"]);
        assert!(parse(&wrong).is_err());
        wrong = value.clone();
        wrong["universe_priority"] = json!([
            "blizzard_ui_source",
            "blizzard_ui_source",
            "reference_platform_contract"
        ]);
        assert!(parse(&wrong).is_err());
        Ok(())
    }
    #[test]
    fn fixture_source_admission_cannot_take_vendor_grade() -> Result<(), Box<dyn std::error::Error>>
    {
        use wow_core::{
            ContentDigest, ProfileIdentityBuilder, ProfileKind, ReferenceGenerationId,
            SchemaVersionEntry, SourceKind, SourceLogicalSnapshot,
        };
        use wow_project::platform_source::{
            BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformRootSpec,
            PlatformSourceClass, PlatformTarget, SourceAdmissionLimits,
        };
        let profile = ProfileIdentityBuilder::new(
            "profile:fixture:three-admission".parse()?,
            ProfileKind::Fixture,
            "retail",
            1,
            SourceKind::SyntheticFixture,
            "fixture-reference",
            ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),
        )
        .schema_versions(vec![SchemaVersionEntry::new(
            "schema:wow:fixture-e0".parse()?,
            "1.0.0".parse()?,
        )])
        .fixture_scope("typed source-class admission control only")
        .build()?;
        let mut input = BlizzardUiSourceProfileRequest {
            profile_id: "profile:fixture:three-source".parse()?,
            source_class: PlatformSourceClass::SyntheticFixture,
            target: PlatformTarget {
                product: "fixture".into(),
                channel: "fixture".into(),
                reference_profile: profile,
                reference_generation: ReferenceGenerationId::derive(&"fixture-reference")?,
            },
            roots: vec![PlatformRootSpec {
                root: "UI".into(),
                selected_tocs: vec!["UI/Fixture.toc".into()],
            }],
            exclusions: Vec::new(),
            limits: SourceAdmissionLimits::new(8, 65536, 65536, 65536)?,
        };
        assert!(BlizzardUiSourceProfile::new(input.clone()).is_ok());
        input.source_class = PlatformSourceClass::VendorUiSourceMirror;
        assert!(BlizzardUiSourceProfile::new(input).is_err());
        Ok(())
    }
    #[test]
    fn exact_compatibility_authority_caps_priority_and_complete_input_bounds() -> SearchResult<()> {
        let fuse = |primary: &serde_json::Value,
                    reference: &serde_json::Value,
                    request: &FederationQueryRequest,
                    stop: &AtomicBool| {
            super::fuse(
                primary,
                reference,
                wow_project::ProjectKind::Repository,
                request,
                stop,
            )
        };
        let stop = AtomicBool::new(false);
        let query = json!({"schema":REQUEST_SCHEMA,"blizzard_ui":"excluded","universe_priority":["primary_user_project","reference_platform_contract"],
            "max_component_result_bytes":65536,"max_input_candidates":8,"max_candidates":8,"max_result_bytes":65536});
        let encoded =
            wow_core::canonical_json_bytes(&query).map_err(|_| SearchError::InvalidProfile)?;
        let request = FederationQueryRequest::from_json_slice(&encoded, &stop)?;
        let profile = json!({"exact":"profile"});
        let p = json!({"result_set_id":"primary-result","payload":{"schema":"wow-search/project-entity-result-set/1","query_profile":"primary-profile",
            "result_query_id":"primary-query","universe_id":"primary-universe","shard_id":"primary-shard","status":"exact_found",
            "owner":{"source":{"project_profile":profile,"reference_generation_id":"exact-reference"}},
            "candidates":[{"candidate_id":"p","stable_tie_key":"Same","match_class":"exact_recorded_owner_key","rank_tuple":[1,0,0]}]}});
        let r = json!({"result_set_id":"reference-result","payload":{"schema":"wow-search/native-reference-result-set/1","query_profile":"reference-profile",
            "normalized_query_id":"reference-query","universe_id":"reference-universe","shard_id":"reference-shard","status":"exact_found",
            "owner":{"binding":{"profile":profile},"reference_generation":"exact-reference","privacy_scope":"local_metadata_only","license_state":"not_evaluated"},
            "candidates":[{"candidate_id":"r","stable_tie_key":"Same","rank_tuple":[1,1,1]}]}});
        let decode = |result: FederationQueryResult| -> SearchResult<serde_json::Value> {
            serde_json::from_slice(result.canonical_bytes())
                .map_err(|_| SearchError::OwnerEvidenceInvalid)
        };
        let result = decode(fuse(&p, &r, &request, &stop)?)?;
        assert!(matches!(
            super::fuse(
                &p,
                &r,
                wow_project::ProjectKind::BlizzardUiPlatformSource,
                &request,
                &stop
            ),
            Err(SearchError::OwnerEvidenceInvalid)
        ));
        let candidates = result["payload"]["candidates"]
            .as_array()
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        assert_eq!(candidates.len(), 2); // Same spelling in distinct universes never collapses.
        assert_eq!(
            candidates[0]["universe_class"],
            "reference_platform_contract"
        );
        assert_eq!(candidates[1]["effective_authority_band"], 4);
        assert_eq!(candidates[1]["source_authority_band"], 1);
        assert_eq!(
            candidates[1]["reciprocal_rank_contribution"],
            1_000_000 / 61
        );
        assert!(
            result["payload"]["components"]["primary_user_project"]["result_summary"]
                .get("candidates")
                .is_none()
        );
        let mut other = r.clone();
        other["payload"]["owner"]["reference_generation"] = json!("foreign");
        assert!(matches!(
            fuse(&p, &other, &request, &stop),
            Err(SearchError::OwnerEvidenceInvalid)
        ));
        other = r.clone();
        other["payload"]["owner"]["binding"]["profile"] = json!({"exact":"foreign"});
        assert!(matches!(
            fuse(&p, &other, &request, &stop),
            Err(SearchError::OwnerEvidenceInvalid)
        ));
        let mut limited = request.clone();
        limited.max_input_candidates = NonZeroU32::new(1).ok_or(SearchError::InvalidProfile)?;
        assert!(matches!(
            fuse(&p, &r, &limited, &stop),
            Err(SearchError::FileSetBudgetExceeded)
        ));
        limited = request.clone();
        limited.max_candidates = NonZeroU32::new(1).ok_or(SearchError::InvalidProfile)?;
        let capped = decode(fuse(&p, &r, &limited, &stop)?)?;
        assert_eq!(capped["payload"]["status"], "truncated");
        assert_eq!(capped["payload"]["candidate_signals_omitted_by_cap"], 1);
        assert!(matches!(
            fuse(&p, &r, &request, &AtomicBool::new(true)),
            Err(SearchError::Cancelled)
        ));
        let mut approximate = p.clone();
        approximate["payload"]["status"] = json!("candidate_only");
        approximate["payload"]["candidates"][0]["match_class"] =
            json!("literal_opaque_owner_key_candidate");
        approximate["payload"]["candidates"][0]["rank_tuple"] = json!([7, 0, 0]);
        other = r.clone();
        other["payload"]["status"] = json!("candidate_only");
        other["payload"]["candidates"][0]["rank_tuple"] = json!([7, 0, 0]);
        assert_eq!(
            decode(fuse(&approximate, &other, &request, &stop)?)?["payload"]["candidates"][0]["universe_class"],
            "primary_user_project"
        );
        let mut reverse = request.clone();
        reverse.universe_priority.reverse();
        assert_eq!(
            decode(fuse(&approximate, &other, &reverse, &stop)?)?["payload"]["candidates"][0]["universe_class"],
            "reference_platform_contract"
        );
        Ok(())
    }
}

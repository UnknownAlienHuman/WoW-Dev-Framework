//! Normal exact primary/Reference binding over genuine Search-owned results.
//! No native owner reconstruction, same-name collapse or cross-shard FTS scores.
use super::*;
use crate::{entities::EntityQueryResult, reference::ReferenceQueryResult};
use std::collections::BTreeSet;

const REQUEST_SCHEMA: &str = "wow-search/exact-primary-reference-query/1";
const PROFILE: &str = "wow-search/exact-primary-reference-federation/1";
const RANK_PROFILE: &str = "wow-search/federated-authority-universe-ordinal/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FederationUniverse {
    PrimaryUserProject,
    ReferencePlatformContract,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BlizzardSelection {
    Excluded,
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
        if self.schema != REQUEST_SCHEMA
            || self.universe_priority.len() != 2
            || self
                .universe_priority
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                != 2
        {
            return Err(SearchError::InvalidProfile);
        }
        Ok(())
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
    if primary_kind == wow_project::ProjectKind::BlizzardUiPlatformSource {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let p = field(primary, "payload")?;
    let r = field(reference, "payload")?;
    if text_field(p, "schema")? != "wow-search/project-entity-result-set/1"
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
    let bindings = [
        Binding {
            project_kind: Some(primary_kind),
            universe_class: FederationUniverse::PrimaryUserProject,
            universe_id: text_field(p, "universe_id")?,
            shard_id: text_field(p, "shard_id")?,
            component_profile: text_field(p, "query_profile")?,
        },
        Binding {
            project_kind: None,
            universe_class: FederationUniverse::ReferencePlatformContract,
            universe_id: text_field(r, "universe_id")?,
            shard_id: text_field(r, "shard_id")?,
            component_profile: text_field(r, "query_profile")?,
        },
    ];
    let universe_set_id = digest(
        "search-universe-set:",
        &(
            PROFILE,
            &bindings,
            field(source, "project_profile")?,
            field(source, "reference_generation_id")?,
            "blizzard_ui_excluded_by_request",
        ),
    )?;
    let query_id = digest(
        "federated-search-query:",
        &(
            PROFILE,
            &universe_set_id,
            text_field(p, "result_query_id")?,
            text_field(r, "normalized_query_id")?,
            request,
        ),
    )?;
    let count = rows(p)?
        .len()
        .checked_add(rows(r)?.len())
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    if count > request.max_input_candidates.get() as usize {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    let mut candidates = Vec::new();
    for (index, (capture, payload)) in [(primary, p), (reference, r)].into_iter().enumerate() {
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
            let metadata_cap =
                index == 0 && text_field(candidate, "match_class")? == "exact_recorded_owner_key";
            let band = if metadata_cap {
                source_band.max(4)
            } else {
                source_band
            };
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
                authority_cap: if metadata_cap {
                    "recorded_owner_key_is_not_canonical_name"
                } else {
                    "original_component_band"
                },
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
    let states = [text_field(p, "status")?, text_field(r, "status")?];
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
    let payload = serde_json::json!({"schema":"wow-search/federated-result-set/1","federation_profile":PROFILE,"ranking_profile":RANK_PROFILE,
        "query_id":query_id,"universe_set_id":universe_set_id,"request":request,"universes":bindings,"status":status,
        "absence_authoritative":false,"blizzard_ui":{"state":"excluded_by_request"},"privacy_scope":"local_metadata_only","license_state":"not_evaluated",
        "candidate_signals_omitted_by_cap":omitted,"rr_scale":1000000,"rr_offset":60,"candidates":candidates,
        "components":{"primary_user_project":summary(primary)?,"reference_platform_contract":summary(reference)?}});
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

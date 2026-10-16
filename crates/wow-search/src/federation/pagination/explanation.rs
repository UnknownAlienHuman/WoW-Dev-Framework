//! Original fused candidate and selected component data, never a query rerun.
use super::*;

pub struct FederationCandidateExplanation {
    bytes: Box<[u8]>,
    status: &'static str,
}
impl FederationCandidateExplanation {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn status(&self) -> &'static str {
        self.status
    }
}
#[allow(clippy::too_many_arguments)]
pub fn explain_retained_source_federation(
    primary: RetainedSourceShard<'_>,
    blizzard: Option<RetainedSourceShard<'_>>,
    reference: &crate::reference::ReferenceSearchShard,
    bytes: &[u8],
    request: &FederationQueryRequest,
    expected_result_set_id: &str,
    candidate_id: &str,
    max_output_bytes: u64,
    stop: &AtomicBool,
) -> SearchResult<FederationCandidateExplanation> {
    check(stop)?;
    if max_output_bytes == 0 {
        return Err(SearchError::InvalidProfile);
    }
    let envelope = validate_retained_impl(primary, blizzard, reference, bytes, request, stop)?;
    let result_id = text_field(&envelope, "result_set_id")?;
    if result_id != expected_result_set_id {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let payload = field(&envelope, "payload")?;
    let rows = field(payload, "candidates")?
        .as_array()
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    let candidate = rows
        .iter()
        .find(|row| {
            row.get("candidate_id").and_then(serde_json::Value::as_str) == Some(candidate_id)
        })
        .ok_or(SearchError::OwnerEvidenceInvalid)?;
    let status = match text_field(payload, "status")? {
        "exact_found" => "exact_found",
        "candidate_only" => "candidate_only",
        "truncated" => "truncated",
        "lane_unavailable" => "lane_unavailable",
        "conflict_blocked" => "conflict_blocked",
        "exact_not_found_partial" => "exact_not_found_partial",
        "no_candidates_under_executed_lanes" => "no_candidates_under_executed_lanes",
        _ => return Err(SearchError::OwnerEvidenceInvalid),
    };
    let mut context = payload.clone();
    context
        .as_object_mut()
        .ok_or(SearchError::OwnerEvidenceInvalid)?
        .remove("candidates");
    let value = serde_json::json!({
        "schema":"wow-search/federated-candidate-explanation/1",
        "result_set_id":result_id,"candidate_id":candidate_id,
        "query_id":text_field(payload,"query_id")?,
        "universe_set_id":text_field(payload,"universe_set_id")?,
        "federation_profile":request.profile(),"ranking_profile":request.ranking_profile(),
        "candidate":candidate,"query_context":context,"status":status
    });
    let bytes =
        wow_core::canonical_json_bytes(&value).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    if bytes.len() as u64 > max_output_bytes {
        return Err(SearchError::FileSetBudgetExceeded);
    }
    check(stop)?;
    Ok(FederationCandidateExplanation {
        bytes: bytes.into_boxed_slice(),
        status,
    })
}

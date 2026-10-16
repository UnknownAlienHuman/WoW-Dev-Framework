//! Exact original Reference Search data, not a rerun or native signature decoder.
use super::*;
use std::collections::BTreeSet;

pub struct ReferenceCandidateExplanation {
    bytes: Box<[u8]>,
    status: &'static str,
}
impl ReferenceCandidateExplanation {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn status(&self) -> &'static str {
        self.status
    }
}
impl ReferenceSearchShard {
    pub fn explain_retained_result(
        &self,
        bytes: &[u8],
        original_request: &ReferenceQueryRequest,
        expected_result_set_id: &str,
        candidate_id: &str,
        max_output_bytes: u64,
        stop: &AtomicBool,
    ) -> SearchResult<ReferenceCandidateExplanation> {
        check(stop)?;
        if max_output_bytes == 0 {
            return Err(SearchError::InvalidProfile);
        }
        if bytes.len() as u64 > original_request.max_result_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let envelope: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let payload = envelope
            .get("payload")
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let result_id = text(&envelope, "result_set_id")?;
        if envelope.as_object().is_none_or(|object| object.len() != 2)
            || wow_core::canonical_json_bytes(&envelope)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?
                != bytes
            || result_id != expected_result_set_id
            || result_id != identity("reference-search-result-set:", payload)?.as_ref()
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let request = self.validate_federation_summary(payload, stop)?;
        if serde_json::to_value(&request).map_err(|_| SearchError::OwnerEvidenceInvalid)?
            != serde_json::to_value(original_request)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let query_id = text(payload, "normalized_query_id")?;
        let universe_id = text(payload, "universe_id")?;
        let rows = payload
            .get("candidates")
            .and_then(serde_json::Value::as_array)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if rows.len() > request.max_candidates.get() as usize {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let mut ids = BTreeSet::new();
        let mut previous: Option<([u32; 3], &str)> = None;
        let mut signal_count = 0usize;
        for row in rows {
            check(stop)?;
            self.validate_federation_candidate(query_id, universe_id, row, &request, stop)?;
            let key = text(row, "stable_tie_key")?;
            let rank: [u32; 3] = serde_json::from_value(
                row.get("rank_tuple")
                    .cloned()
                    .ok_or(SearchError::OwnerEvidenceInvalid)?,
            )
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
            if !ids.insert(key) || previous.is_some_and(|old| old >= (rank, key)) {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            previous = Some((rank, key));
            let signals = row
                .get("signals")
                .and_then(serde_json::Value::as_array)
                .filter(|signals| !signals.is_empty())
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            signal_count = signal_count
                .checked_add(signals.len())
                .ok_or(SearchError::FileSetBudgetExceeded)?;
            if signal_count > request.max_signals.get() as usize {
                return Err(SearchError::FileSetBudgetExceeded);
            }
        }
        let candidate = rows
            .iter()
            .find(|row| {
                row.get("candidate_id").and_then(serde_json::Value::as_str) == Some(candidate_id)
            })
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let status = match text(payload, "status")? {
            "exact_found" => "exact_found",
            "candidate_only" => "candidate_only",
            "truncated" => "truncated",
            "lane_unavailable" => "lane_unavailable",
            "conflict_blocked" => "conflict_blocked",
            "exact_not_found_partial" => "exact_not_found_partial",
            "exact_not_found_with_authority" => "exact_not_found_with_authority",
            "no_candidates_under_executed_lanes" => "no_candidates_under_executed_lanes",
            _ => return Err(SearchError::OwnerEvidenceInvalid),
        };
        let mut context = payload.clone();
        context
            .as_object_mut()
            .ok_or(SearchError::OwnerEvidenceInvalid)?
            .remove("candidates");
        let value = serde_json::json!({
            "schema":"wow-search/native-reference-candidate-explanation/1",
            "result_set_id":result_id,"candidate_id":candidate_id,
            "normalized_query_id":query_id,"universe_id":universe_id,
            "shard_id":self.physical.search_shard_id(),
            "search_profile_digest":self.manifest.profile_digest,
            "candidate":candidate,"query_context":context,"status":status
        });
        let bytes = wow_core::canonical_json_bytes(&value)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if bytes.len() as u64 > max_output_bytes {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        check(stop)?;
        Ok(ReferenceCandidateExplanation {
            bytes: bytes.into_boxed_slice(),
            status,
        })
    }
}
fn text<'a>(value: &'a serde_json::Value, field: &str) -> SearchResult<&'a str> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or(SearchError::OwnerEvidenceInvalid)
}

//! Projection of original Search data, never a new Graph explanation query.
use super::*;
use std::collections::BTreeSet;

pub struct EntityCandidateExplanation {
    bytes: Box<[u8]>,
    status: &'static str,
}
impl EntityCandidateExplanation {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn status(&self) -> &'static str {
        self.status
    }
}

impl EntitySearchShard {
    pub fn explain_retained_result(
        &self,
        bytes: &[u8],
        request: &EntityQueryRequest,
        expected_result_set_id: &str,
        candidate_id: &str,
        max_output_bytes: u64,
        stop: &AtomicBool,
    ) -> SearchResult<EntityCandidateExplanation> {
        check(stop)?;
        if max_output_bytes == 0 {
            return Err(SearchError::InvalidProfile);
        }
        let envelope = self.validate_retained_result(bytes, request, stop)?;
        let result_set_id = text_field(&envelope, "result_set_id")?;
        if result_set_id != expected_result_set_id {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let payload = envelope
            .get("payload")
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let candidates = payload
            .get("candidates")
            .and_then(serde_json::Value::as_array)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let candidate = candidates
            .iter()
            .find(|row| {
                row.get("candidate_id").and_then(serde_json::Value::as_str) == Some(candidate_id)
            })
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        // A candidate address does not upgrade the original query's authority.
        let status = match text_field(payload, "status")? {
            "exact_found" => "exact_found",
            "candidate_only" => "candidate_only",
            "truncated" => "truncated",
            _ => return Err(SearchError::OwnerEvidenceInvalid),
        };
        let mut required = BTreeSet::new();
        required.insert((
            "entity".to_owned(),
            text_field(candidate, "stable_tie_key")?.to_owned(),
        ));
        if let Some(path) = candidate.get("graph_path") {
            let steps = path
                .get("steps")
                .and_then(serde_json::Value::as_array)
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            for step in steps {
                check(stop)?;
                let edge = step
                    .get("original_edge")
                    .ok_or(SearchError::OwnerEvidenceInvalid)?;
                required.insert((
                    "relation".to_owned(),
                    text_field(edge, "edge_id")?.to_owned(),
                ));
            }
        }
        let entries = payload
            .get("explanations")
            .and_then(serde_json::Value::as_array)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let mut seen = BTreeSet::new();
        let mut supporting = Vec::new();
        for entry in entries {
            check(stop)?;
            let subject = entry
                .get("subject")
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            let key = (
                text_field(subject, "kind")?.to_owned(),
                text_field(subject, "id")?.to_owned(),
            );
            if required.contains(&key) {
                if !seen.insert(key) {
                    return Err(SearchError::OwnerEvidenceInvalid);
                }
                // Preserve the entry even when its original support is partial,
                // empty, conflicted or NotEvaluated. Never synthesize support.
                supporting.push(entry.clone());
            }
        }
        if seen != required {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let mut context = payload.clone();
        let fields = context
            .as_object_mut()
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        fields.remove("candidates");
        fields.remove("explanations");
        let result = serde_json::json!({
            "schema":"wow-search/project-entity-candidate-explanation/1",
            "result_set_id":result_set_id,
            "candidate_id":candidate_id,
            "result_query_id":text_field(payload,"result_query_id")?,
            "universe_id":text_field(payload,"universe_id")?,
            "shard_id":self.logical_id(),
            "search_profile_digest":self.profile_digest(),
            "candidate":candidate,
            "supporting_explanations":supporting,
            "query_context":context,
            "status":status
        });
        let bytes = wow_core::canonical_json_bytes(&result)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if bytes.len() as u64 > max_output_bytes {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        check(stop)?;
        Ok(EntityCandidateExplanation {
            bytes: bytes.into_boxed_slice(),
            status,
        })
    }
}

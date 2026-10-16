//! Validate and project original query data without executing another lane.
use super::*;

pub struct ObservationCandidateExplanation {
    bytes: Box<[u8]>,
    status: &'static str,
}
impl ObservationCandidateExplanation {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn status(&self) -> &'static str {
        self.status
    }
}
impl ObservationSearchShard {
    pub fn explain_retained_result(
        &self,
        bytes: &[u8],
        expected_result_set_id: &str,
        candidate_id: &str,
        max_retained_bytes: u64,
        max_output_bytes: u64,
        stop: &AtomicBool,
    ) -> SearchResult<ObservationCandidateExplanation> {
        check(stop)?;
        if max_output_bytes == 0 || max_retained_bytes == 0 {
            return Err(SearchError::InvalidProfile);
        }
        if bytes.len() as u64 > max_retained_bytes {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let original: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if wow_core::canonical_json_bytes(&original)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?
            != bytes
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let mut context = original.clone();
        let object = context
            .as_object_mut()
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let result_id = object
            .remove("result_set_id")
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if result_id.as_str() != Some(expected_result_set_id)
            || identity("source-observation-result:", &context)?.as_ref() != expected_result_set_id
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let request = ObservationQueryRequest::from_json_slice(
            &wow_core::canonical_json_bytes(
                original
                    .get("request")
                    .ok_or(SearchError::OwnerEvidenceInvalid)?,
            )
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
            stop,
        )?;
        if bytes.len() as u64 > request.max_result_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        if request.max_candidates > self.bounds.max_candidate_results
            || matches!(&request.query, Query::LiteralName { text }
                if text.len() > self.bounds.max_literal_query_bytes.get() as usize)
        {
            return Err(SearchError::InvalidProfile);
        }
        let rows = context
            .as_object_mut()
            .ok_or(SearchError::OwnerEvidenceInvalid)?
            .remove("candidates")
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let rows = rows.as_array().ok_or(SearchError::OwnerEvidenceInvalid)?;
        // Reuse the existing owner summary validator with its two mechanically
        // derived federation guard fields. The returned context stays original.
        let mut summary = context.clone();
        let summary_object = summary
            .as_object_mut()
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        for key in summary_object.keys() {
            if !matches!(
                key.as_str(),
                "schema"
                    | "query_profile"
                    | "query_id"
                    | "request"
                    | "shard_id"
                    | "profile_digest"
                    | "owner"
                    | "project_kind"
                    | "local_analysis_id"
                    | "member_analysis_id"
                    | "main_snapshot_id"
                    | "local_file_coverage"
                    | "member_file_coverage"
                    | "status"
                    | "scanned_documents"
                    | "matched_documents"
                    | "candidate_omissions"
                    | "unscanned_documents"
                    | "prefiltered_documents"
                    | "charged_distance_cells"
                    | "excluded_long_identifiers"
                    | "lane_unavailable"
                    | "absence_authority"
                    | "continuation"
                    | "nonclaims"
                    | "function_names"
                    | "global_accesses"
                    | "platform_source_profile"
                    | "platform_package_binding"
            ) {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
        }
        summary_object.insert(
            "result_query_id".into(),
            text(&original, "query_id")?.into(),
        );
        summary_object.insert(
            "universe_id".into(),
            identity(
                "source-observation-search-universe:",
                &(&self.manifest.owner.source, self.profile_digest()),
            )?
            .to_string()
            .into(),
        );
        self.validate_federation_summary(&summary, stop)?;
        let scanned = number(&original, "scanned_documents")?;
        let matched = number(&original, "matched_documents")?;
        if rows.len() as u64 != matched.min(u64::from(request.max_candidates.get())) {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let query_id = text(&original, "query_id")?;
        let mut previous: Option<([u32; 2], &str)> = None;
        let mut ids = BTreeSet::new();
        for row in rows {
            check(stop)?;
            self.validate_federation_candidate(query_id, row, stop)?;
            let key = text(row, "stable_tie_key")?;
            let index = self
                .manifest
                .documents
                .binary_search_by(|document| document.fact_id().cmp(key))
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
            let document = &self.manifest.documents[index];
            if index as u64 >= scanned || !request.kinds.contains(&document.kind) {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            let (band, class, distance) = validate_match(&request, document, row)?;
            let expected = Candidate {
                candidate_id: identity("source-observation-candidate:", &(query_id, key))?,
                document,
                match_class: class,
                authority_band: band,
                rank_tuple: [band, distance.unwrap_or(0)],
                stable_tie_key: key,
                edit_distance: distance,
                entity_authority: "recorded_source_observation_only",
            };
            if row != &value(&expected)?
                || !ids.insert(key)
                || previous.is_some_and(|old| old >= (expected.rank_tuple, key))
            {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            previous = Some((expected.rank_tuple, key));
        }
        let unavailable = original
            .get("lane_unavailable")
            .and_then(serde_json::Value::as_bool)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        if unavailable && !matches!(request.query, Query::LiteralName { .. }) {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let truncated = unavailable
            || number(&original, "unscanned_documents")? > 0
            || number(&original, "candidate_omissions")? > 0;
        let status = match text(&original, "status")? {
            "candidate_only" if !rows.is_empty() && !truncated => "candidate_only",
            "no_candidates_under_executed_lanes" if rows.is_empty() && !truncated => {
                "no_candidates_under_executed_lanes"
            }
            "truncated" => "truncated",
            _ => return Err(SearchError::OwnerEvidenceInvalid),
        };
        let candidate = rows
            .iter()
            .find(|row| {
                row.get("candidate_id").and_then(serde_json::Value::as_str) == Some(candidate_id)
            })
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let explanation = serde_json::json!({
            "schema":"wow-search/source-observation-candidate-explanation/1",
            "result_set_id":expected_result_set_id,"candidate_id":candidate_id,
            "query_id":query_id,"shard_id":self.logical_id(),
            "search_profile_digest":self.profile_digest(),
            "candidate":candidate,"query_context":context,"status":status
        });
        let bytes = wow_core::canonical_json_bytes(&explanation)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if bytes.len() as u64 > max_output_bytes {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        check(stop)?;
        Ok(ObservationCandidateExplanation {
            bytes: bytes.into_boxed_slice(),
            status,
        })
    }
}

fn validate_match(
    request: &ObservationQueryRequest,
    document: &ObservationDocument,
    row: &serde_json::Value,
) -> SearchResult<(u32, &'static str, Option<u32>)> {
    Ok(match &request.query {
        Query::ExactFact { fact_id } if fact_id == document.fact_id() => {
            (0, "exact_owner_fact_identity", None)
        }
        Query::RecordedName { name }
            if name == document.recorded_name()
                || document.recorded_short_name.as_deref() == Some(name) =>
        {
            (1, "exact_recorded_source_name", None)
        }
        Query::RecordedMember { receiver, member }
            if matches!(
                document.kind,
                Kind::MemberReference | Kind::FunctionDeclaration
            ) && document.recorded_receiver.as_deref() == Some(receiver)
                && member
                    == document
                        .recorded_short_name
                        .as_deref()
                        .unwrap_or(document.recorded_name()) =>
        {
            (1, "exact_recorded_receiver_member_syntax", None)
        }
        Query::Prefix { prefix }
            if document.recorded_name.starts_with(prefix)
                || document
                    .recorded_short_name
                    .as_deref()
                    .is_some_and(|name| name.starts_with(prefix)) =>
        {
            (2, "recorded_name_prefix_candidate", None)
        }
        Query::RecordedSlot {
            root,
            keys,
            alias_policy,
            access_kind,
        } => {
            let slot = document
                .global_slot
                .as_ref()
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if !slot.path_complete
                || slot.alias_blocked
                || slot.root.as_ref() != root
                || slot.keys != *keys
                || !access_kind.matches(slot.access_kind)
            {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            if slot.alias {
                if !matches!(alias_policy, SlotAliasPolicy::IncludePossible) {
                    return Err(SearchError::OwnerEvidenceInvalid);
                }
                (2, "possible_alias_global_slot_candidate", None)
            } else {
                (1, "exact_recorded_global_slot_observation", None)
            }
        }
        Query::LiteralName { .. } => (3, "literal_recorded_name_candidate", None),
        Query::IdentifierSimilarity { max_distance, .. } => {
            let distance = u32::try_from(number(row, "edit_distance")?)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
            if distance > *max_distance {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            (4, "identifier_similarity_candidate", Some(distance))
        }
        _ => return Err(SearchError::OwnerEvidenceInvalid),
    })
}
fn text<'a>(data: &'a serde_json::Value, key: &str) -> SearchResult<&'a str> {
    data.get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or(SearchError::OwnerEvidenceInvalid)
}
fn number(data: &serde_json::Value, key: &str) -> SearchResult<u64> {
    data.get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or(SearchError::OwnerEvidenceInvalid)
}

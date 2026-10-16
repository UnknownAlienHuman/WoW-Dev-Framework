//! One explicitly selected, bounded retrieval lane over recorded source facts.
use super::*;
use crate::identifier_distance::{DistanceResult, bounded_distance, normalize};
use std::collections::BTreeSet;
const QUERY_PROFILE: &str = "wow-search/recorded-source-fact-name-member-prefix-text-similarity/1";
const DECLARATION_QUERY_PROFILE: &str =
    "wow-search/recorded-source-declaration-name-member-prefix-text-similarity/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "class", rename_all = "snake_case", deny_unknown_fields)]
enum Query {
    ExactFact {
        fact_id: String,
    },
    RecordedName {
        name: String,
    },
    RecordedMember {
        receiver: String,
        member: String,
    },
    Prefix {
        prefix: String,
    },
    LiteralName {
        text: String,
    },
    IdentifierSimilarity {
        identifier: String,
        ascii_fold: bool,
        max_distance: u32,
        max_prefiltered_documents: NonZeroU32,
        max_distance_cells: NonZeroU64,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationQueryRequest {
    schema: String,
    query: Query,
    kinds: Vec<Kind>,
    max_scanned_documents: NonZeroU32,
    max_candidates: NonZeroU32,
    max_result_bytes: NonZeroU64,
}
impl ObservationQueryRequest {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> SearchResult<Self> {
        check(stop)?;
        if bytes.len() > 65536 {
            return Err(SearchError::InvalidProfile);
        }
        let request: Self =
            serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
        let declarations = request.schema == "wow-search/source-observation-declaration-query/1";
        if (!declarations && request.schema != "wow-search/source-observation-query/1")
            || request.kinds.is_empty()
            || request.kinds.len() > if declarations { 3 } else { 2 }
            || request
                .kinds
                .iter()
                .enumerate()
                .any(|(index, kind)| request.kinds[..index].contains(kind))
            || !declarations && request.kinds.contains(&Kind::FunctionDeclaration)
        {
            return Err(SearchError::InvalidProfile);
        }
        match &request.query {
            Query::ExactFact { fact_id } => valid_text(fact_id)?,
            Query::RecordedName { name } => valid_text(name)?,
            Query::RecordedMember { receiver, member } => {
                valid_text(receiver)?;
                valid_text(member)?;
            }
            Query::Prefix { prefix } => valid_text(prefix)?,
            Query::LiteralName { text } => valid_text(text)?,
            Query::IdentifierSimilarity {
                identifier,
                max_distance,
                ..
            } => {
                valid_text(identifier)?;
                if identifier.chars().count() > 128 || *max_distance > 4 {
                    return Err(SearchError::InvalidProfile);
                }
            }
        }
        Ok(request)
    }
}
pub struct ObservationQueryResult {
    bytes: Box<[u8]>,
    status: &'static str,
}
impl ObservationQueryResult {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn status(&self) -> &'static str {
        self.status
    }
}
#[derive(Serialize)]
struct Candidate<'a> {
    candidate_id: Box<str>,
    document: &'a ObservationDocument,
    match_class: &'static str,
    authority_band: u32,
    rank_tuple: [u32; 2],
    stable_tie_key: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    edit_distance: Option<u32>,
    entity_authority: &'static str,
}
impl ObservationSearchShard {
    pub fn retrieve(
        &self,
        request: &ObservationQueryRequest,
        stop: &AtomicBool,
    ) -> SearchResult<ObservationQueryResult> {
        check(stop)?;
        let declarations = self.manifest.function_names.is_some();
        if (request.schema == "wow-search/source-observation-declaration-query/1") != declarations {
            return Err(SearchError::InvalidProfile);
        }
        let query_profile = if declarations {
            DECLARATION_QUERY_PROFILE
        } else {
            QUERY_PROFILE
        };
        if request.max_candidates > self.bounds.max_candidate_results {
            return Err(SearchError::InvalidProfile);
        }
        let query_id = identity(
            "source-observation-query:",
            &(query_profile, self.logical_id(), request),
        )?;
        let mut text_ids = BTreeSet::new();
        let mut truncated = false;
        let mut lane_unavailable = false;
        if let Query::LiteralName { text } = &request.query {
            if text.len() > self.bounds.max_literal_query_bytes.get() as usize {
                return Err(SearchError::InvalidProfile);
            }
            match self.physical.reader().literal_candidates(text) {
                Ok(rows) => {
                    for row in rows {
                        check(stop)?;
                        text_ids.insert(row.document_id().to_owned());
                    }
                }
                Err(error) if error.code() == StoreErrorCode::BudgetExceeded => {
                    truncated = true;
                    lane_unavailable = true;
                }
                Err(error) => return Err(SearchError::Store(error)),
            }
        }
        let mut remaining_cells = match request.query {
            Query::IdentifierSimilarity {
                max_distance_cells, ..
            } => max_distance_cells.get(),
            _ => 0,
        };
        let initial_cells = remaining_cells;
        let similarity_query = match &request.query {
            Query::IdentifierSimilarity {
                identifier,
                ascii_fold,
                ..
            } => normalize(identifier, *ascii_fold),
            _ => Vec::new(),
        };
        let mut prefiltered = 0u32;
        let mut excluded_long_identifiers = 0u32;
        let mut scanned = 0u32;
        let mut matched = 0u32;
        let mut candidates: Vec<Candidate<'_>> = Vec::new();
        let mut candidate_bytes = 0u64;
        if !lane_unavailable {
            for document in &self.manifest.documents {
                check(stop)?;
                if scanned == request.max_scanned_documents.get() {
                    truncated = true;
                    break;
                }
                scanned += 1;
                if !request.kinds.contains(&document.kind) {
                    continue;
                }
                let (band, class, distance) = match &request.query {
                    Query::ExactFact { fact_id } if fact_id.as_str() == document.fact_id() => {
                        (0, "exact_owner_fact_identity", None)
                    }
                    Query::RecordedName { name }
                        if name.as_str() == document.recorded_name()
                            || document.recorded_short_name.as_deref() == Some(name) =>
                    {
                        (1, "exact_recorded_source_name", None)
                    }
                    Query::RecordedMember { receiver, member }
                        if matches!(
                            document.kind,
                            Kind::MemberReference | Kind::FunctionDeclaration
                        ) && document.recorded_receiver.as_deref() == Some(receiver)
                            && member.as_str()
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
                                .is_some_and(|s| s.starts_with(prefix)) =>
                    {
                        (2, "recorded_name_prefix_candidate", None)
                    }
                    Query::LiteralName { .. } if text_ids.contains(document.fact_id()) => {
                        (3, "literal_recorded_name_candidate", None)
                    }
                    Query::IdentifierSimilarity {
                        ascii_fold,
                        max_distance,
                        max_prefiltered_documents,
                        ..
                    } => {
                        let fields = [
                            Some(document.recorded_name()),
                            document
                                .recorded_short_name
                                .as_deref()
                                .filter(|name| *name != document.recorded_name()),
                        ];
                        let mut eligible = false;
                        for field in fields.into_iter().flatten() {
                            let length = field.chars().count();
                            if length > 132 {
                                excluded_long_identifiers += 1;
                            } else if similarity_query.len().abs_diff(length)
                                <= *max_distance as usize
                            {
                                eligible = true;
                            }
                        }
                        if !eligible {
                            continue;
                        }
                        if prefiltered == max_prefiltered_documents.get() {
                            truncated = true;
                            break;
                        }
                        prefiltered += 1;
                        let mut best: Option<u32> = None;
                        let mut exhausted = false;
                        for field in fields.into_iter().flatten() {
                            check(stop)?;
                            let length = field.chars().count();
                            if length > 132
                                || similarity_query.len().abs_diff(length) > *max_distance as usize
                            {
                                continue;
                            }
                            match bounded_distance(
                                &similarity_query,
                                &normalize(field, *ascii_fold),
                                *max_distance,
                                &mut remaining_cells,
                                stop,
                            )? {
                                DistanceResult::Within(distance) => {
                                    best = Some(best.map_or(distance, |old| old.min(distance)));
                                }
                                DistanceResult::Outside => {}
                                DistanceResult::Exhausted => {
                                    exhausted = true;
                                    break;
                                }
                            }
                        }
                        if exhausted {
                            truncated = true;
                            break;
                        }
                        let Some(distance) = best else { continue };
                        (4, "identifier_similarity_candidate", Some(distance))
                    }
                    _ => continue,
                };
                matched += 1;
                let candidate = Candidate {
                    candidate_id: identity(
                        "source-observation-candidate:",
                        &(&query_id, document.fact_id()),
                    )?,
                    document,
                    match_class: class,
                    authority_band: band,
                    rank_tuple: [band, distance.unwrap_or(0)],
                    stable_tie_key: document.fact_id(),
                    edit_distance: distance,
                    entity_authority: "recorded_source_observation_only",
                };
                // Keep the globally best bounded set, even if a better hit occurs late.
                if candidates.len() < request.max_candidates.get() as usize {
                    candidate_bytes = candidate_bytes
                        .checked_add(size(&candidate)?)
                        .ok_or(SearchError::FileSetBudgetExceeded)?;
                    if candidate_bytes > request.max_result_bytes.get() {
                        return Err(SearchError::FileSetBudgetExceeded);
                    }
                    let index = candidates.partition_point(|old| {
                        (old.rank_tuple, old.stable_tie_key)
                            < (candidate.rank_tuple, candidate.stable_tie_key)
                    });
                    candidates.insert(index, candidate);
                } else {
                    truncated = true;
                    let last = candidates.last().ok_or(SearchError::OwnerEvidenceInvalid)?;
                    if (candidate.rank_tuple, candidate.stable_tie_key)
                        < (last.rank_tuple, last.stable_tie_key)
                    {
                        candidate_bytes = candidate_bytes
                            .checked_sub(size(last)?)
                            .ok_or(SearchError::OwnerEvidenceInvalid)?
                            .checked_add(size(&candidate)?)
                            .ok_or(SearchError::FileSetBudgetExceeded)?;
                        if candidate_bytes > request.max_result_bytes.get() {
                            return Err(SearchError::FileSetBudgetExceeded);
                        }
                        candidates.pop();
                        let index = candidates.partition_point(|old| {
                            (old.rank_tuple, old.stable_tie_key)
                                < (candidate.rank_tuple, candidate.stable_tie_key)
                        });
                        candidates.insert(index, candidate);
                    }
                }
            }
        }
        let status = if truncated || excluded_long_identifiers > 0 {
            "truncated"
        } else if candidates.is_empty() {
            "no_candidates_under_executed_lanes"
        } else {
            "candidate_only"
        };
        let value = serde_json::json!({
            "schema":if declarations {"wow-search/source-observation-declaration-result-set/1"} else {"wow-search/source-observation-result-set/1"}, "query_profile":query_profile,
            "query_id":query_id, "request":request, "shard_id":self.logical_id(), "profile_digest":self.profile_digest(),
            "owner":self.manifest.owner, "project_kind":self.manifest.project_kind,
            "platform_source_profile":self.manifest.platform_source_profile, "platform_package_binding":self.manifest.platform_package_binding,
            "local_analysis_id":self.manifest.local_analysis_id, "member_analysis_id":self.manifest.member_analysis_id,
            "main_snapshot_id":self.manifest.main_snapshot_id,
            "local_file_coverage":self.manifest.local_file_coverage, "member_file_coverage":self.manifest.member_file_coverage,
            "candidates":candidates, "status":status, "scanned_documents":scanned, "matched_documents":matched,
            "candidate_omissions":matched.saturating_sub(candidates.len() as u32), "unscanned_documents":self.manifest.documents.len().saturating_sub(scanned as usize),
            "prefiltered_documents":prefiltered, "charged_distance_cells":initial_cells-remaining_cells,
            "excluded_long_identifiers":excluded_long_identifiers, "lane_unavailable":lane_unavailable,
            "absence_authority":"not_evaluated", "continuation":"not_supported",
            "nonclaims":["canonical_name","alias","runtime_target","platform_availability","complete_source_fact_coverage"]
        });
        let mut value = value;
        if let Some(coverage) = &self.manifest.function_names {
            value
                .as_object_mut()
                .ok_or(SearchError::OwnerEvidenceInvalid)?
                .insert("function_names".into(), super::value(coverage)?);
        }
        // E0 canonical JSON excludes null; absent platform evidence stays absent.
        if self.manifest.platform_source_profile.is_none() {
            value
                .as_object_mut()
                .ok_or(SearchError::OwnerEvidenceInvalid)?
                .remove("platform_source_profile");
        }
        if self.manifest.platform_package_binding.is_none() {
            value
                .as_object_mut()
                .ok_or(SearchError::OwnerEvidenceInvalid)?
                .remove("platform_package_binding");
        }
        let result_set_id = identity("source-observation-result:", &value)?;
        value
            .as_object_mut()
            .ok_or(SearchError::OwnerEvidenceInvalid)?
            .insert("result_set_id".into(), result_set_id.to_string().into());
        let bytes = wow_core::canonical_json_bytes(&value)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if bytes.len() as u64 > request.max_result_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        check(stop)?;
        Ok(ObservationQueryResult {
            bytes: bytes.into_boxed_slice(),
            status,
        })
    }
}
fn valid_text(value: &str) -> SearchResult<()> {
    if value.trim().is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
        Err(SearchError::InvalidProfile)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_source_query_rejects_implicit_authority_and_unbounded_work() -> SearchResult<()> {
        let stop = AtomicBool::new(false);
        let query = serde_json::json!({"schema":"wow-search/source-observation-query/1",
            "query":{"class":"identifier_similarity","identifier":"Read","ascii_fold":false,
                "max_distance":0,"max_prefiltered_documents":4,"max_distance_cells":16},
            "kinds":["local_binding","member_reference"],"max_scanned_documents":8,"max_candidates":4,"max_result_bytes":65536});
        let parse = |v: &serde_json::Value, stop: &AtomicBool| {
            ObservationQueryRequest::from_json_slice(
                &serde_json::to_vec(v).map_err(|_| SearchError::InvalidProfile)?,
                stop,
            )
        };
        let request = parse(&query, &stop)?;
        assert!(matches!(
            request.query,
            Query::IdentifierSimilarity {
                max_distance: 0,
                ascii_fold: false,
                ..
            }
        ));
        for (key, bad) in [
            ("max_distance", serde_json::json!(5)),
            ("max_distance_cells", serde_json::json!(0)),
            ("identifier", serde_json::json!("a".repeat(129))),
            ("alias", serde_json::json!(true)),
        ] {
            let mut changed = query.clone();
            changed["query"][key] = bad;
            assert!(matches!(
                parse(&changed, &stop),
                Err(SearchError::InvalidProfile)
            ));
        }
        let mut changed = query.clone();
        changed["kinds"] = serde_json::json!(["local_binding", "local_binding"]);
        assert!(matches!(
            parse(&changed, &stop),
            Err(SearchError::InvalidProfile)
        ));
        assert!(matches!(
            parse(&query, &AtomicBool::new(true)),
            Err(SearchError::Cancelled)
        ));
        Ok(())
    }
}

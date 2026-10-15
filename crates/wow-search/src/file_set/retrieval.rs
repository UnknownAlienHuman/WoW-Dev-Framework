//! Closed metadata retrieval and authority-preserving fusion over one exact shard.
use std::collections::BTreeMap;

use super::*;

mod pagination;
pub use pagination::{RetrievalCursor, RetrievalPage, RetrievalPagePolicy};

const REQUEST_SCHEMA: &str = "wow-search/project-file-set-retrieval-request/1";
const RESULT_SCHEMA: &str = "wow-search/project-file-set-result-set/1";
const QUERY_PROFILE: &str = "wow-search/project-file-metadata-identity-prefix-literal/1";
const RANKING_PROFILE: &str = "wow-search/metadata-authority-ordinal-fusion/1";
const REQUEST_MAX_BYTES: usize = 64 * 1024;
const RR_SCALE: u32 = 1_000_000;
const RR_OFFSET: u32 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchRetrievalLane {
    ExactIdentity,
    IdentifierPrefix,
    LiteralText,
    ExactCanonicalName,
    ExactExplicitAlias,
    NamespaceMember,
    IdentifierSimilarity,
    StructuredShape,
    SeededGraph,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "class", deny_unknown_fields, rename_all = "snake_case")]
pub enum SearchQuery {
    ExactEntity {
        entity_id: ProjectFileId,
    },
    IdentifierPrefix {
        prefix: String,
    },
    LiteralText {
        text: String,
    },
    Mixed {
        #[serde(skip_serializing_if = "Option::is_none")]
        entity_id: Option<ProjectFileId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalRequest {
    schema: String,
    query: SearchQuery,
    enabled_lanes: Vec<SearchRetrievalLane>,
    required_lanes: Vec<SearchRetrievalLane>,
    max_scanned_documents: NonZeroU32,
    max_candidates: NonZeroU32,
    max_signals: NonZeroU32,
    max_manifest_bytes: NonZeroU64,
}

impl RetrievalRequest {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> SearchResult<Self> {
        check(stop)?;
        if bytes.len() > REQUEST_MAX_BYTES {
            return Err(SearchError::InvalidProfile);
        }
        let mut request: Self =
            serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
        if request.schema != REQUEST_SCHEMA
            || request.enabled_lanes.len() > 9
            || request.required_lanes.len() > 9
            || request.enabled_lanes.is_empty()
        {
            return Err(SearchError::InvalidProfile);
        }
        request.enabled_lanes.sort();
        request.required_lanes.sort();
        if request
            .enabled_lanes
            .windows(2)
            .any(|pair| pair[0] == pair[1])
            || request
                .required_lanes
                .windows(2)
                .any(|pair| pair[0] == pair[1])
            || request
                .required_lanes
                .iter()
                .any(|lane| !request.enabled_lanes.contains(lane))
        {
            return Err(SearchError::InvalidProfile);
        }
        let (id, prefix, text) = request.parts();
        if id.is_none() && prefix.is_none() && text.is_none() {
            return Err(SearchError::InvalidProfile);
        }
        for (present, lane) in [
            (id.is_some(), SearchRetrievalLane::ExactIdentity),
            (prefix.is_some(), SearchRetrievalLane::IdentifierPrefix),
            (text.is_some(), SearchRetrievalLane::LiteralText),
        ] {
            if present && !request.enabled_lanes.contains(&lane) {
                return Err(SearchError::InvalidProfile);
            }
        }
        if prefix.is_some_and(|value| {
            value.len() < 2 || value.len() > 4096 || value.chars().any(char::is_control)
        }) || text.is_some_and(|value| {
            value.trim().is_empty() || value.len() > 4096 || value.chars().any(char::is_control)
        }) {
            return Err(SearchError::InvalidProfile);
        }
        check(stop)?;
        Ok(request)
    }

    fn parts(&self) -> (Option<&ProjectFileId>, Option<&str>, Option<&str>) {
        match &self.query {
            SearchQuery::ExactEntity { entity_id } => (Some(entity_id), None, None),
            SearchQuery::IdentifierPrefix { prefix } => (None, Some(prefix), None),
            SearchQuery::LiteralText { text } => (None, None, Some(text)),
            SearchQuery::Mixed {
                entity_id,
                prefix,
                text,
            } => (entity_id.as_ref(), prefix.as_deref(), text.as_deref()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalState {
    ExactFound,
    ExactNotFoundPartial,
    CandidateOnly,
    NoCandidatesUnderExecutedLanes,
    LaneUnavailable,
    Truncated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum LaneState {
    Complete,
    ExcludedByRequest,
    UnsupportedQueryClass,
    MissingShardCapability,
    Truncated,
}

#[derive(Debug, Clone, Serialize)]
struct LaneResult {
    lane: SearchRetrievalLane,
    state: LaneState,
    returned_signals: u32,
    required: bool,
}

#[derive(Debug, Clone, Serialize)]
struct FieldOrigin {
    owner_entity_id: ProjectFileId,
    owner_record_digest: Box<str>,
    owner_field: &'static str,
    original_value_digest: Box<str>,
    normalized_value_digest: Box<str>,
    normalization_profile: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct Signal {
    signal_id: Box<str>,
    lane: SearchRetrievalLane,
    match_class: &'static str,
    evidence: &'static str,
    lane_rank: u32,
    reciprocal_rank_contribution: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix_byte_range: Option<[u32; 2]>,
    origin: FieldOrigin,
}

#[derive(Debug, Clone, Serialize)]
struct RankedCandidate {
    candidate_id: Box<str>,
    entity_id: ProjectFileId,
    owner_metadata_label: OwnerMetadataLabel,
    authority_band: u32,
    rank_tuple: [u32; 3],
    stable_tie_key: Box<str>,
    signals: Vec<Signal>,
}

#[derive(Debug, Clone, Serialize)]
struct Payload {
    schema: &'static str,
    query_profile: &'static str,
    ranking_profile: &'static str,
    normalized_query_id: Box<str>,
    normalized_request: RetrievalRequest,
    original_query_data_digest: Box<str>,
    universe_set_id: Box<str>,
    shard_id: Box<str>,
    shard_profile_digest: Box<str>,
    owner: OwnerManifestEvidence,
    state: RetrievalState,
    absence_authoritative: bool,
    document_coverage: &'static str,
    normalization: &'static str,
    lanes: Vec<LaneResult>,
    candidates: Vec<RankedCandidate>,
    candidate_signals_omitted_by_cap: u32,
    candidate_enumeration_complete: bool,
    rr_scale: u32,
    rr_offset: u32,
    rank_tuple_fields: [&'static str; 3],
}

/// Immutable complete retained candidate ordering for this exact bounded query.
pub struct RetrievalResult {
    result_set_id: Box<str>,
    payload: Payload,
    bytes: Box<[u8]>,
}

impl RetrievalResult {
    #[must_use]
    pub fn result_set_id(&self) -> &str {
        &self.result_set_id
    }
    #[must_use]
    pub const fn state(&self) -> RetrievalState {
        self.payload.state
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Default)]
struct Work {
    scanned: u32,
    signals: u32,
    omitted: u32,
}

impl SearchFileSetShard {
    pub fn retrieve(
        &self,
        request: &RetrievalRequest,
        stop: &AtomicBool,
    ) -> SearchResult<RetrievalResult> {
        check(stop)?;
        let query_id = digest(
            "search-normalized-query:",
            &(
                QUERY_PROFILE,
                self.logical_id(),
                self.profile_digest(),
                request,
            ),
        )?;
        let universe_set_id = digest(
            "search-universe-set:",
            &(
                self.logical_id(),
                self.profile_digest(),
                &self.manifest.owner.source,
            ),
        )?;
        let (exact_id, prefix, text) = request.parts();
        let mut candidates = BTreeMap::<ProjectFileId, RankedCandidate>::new();
        let mut lanes = Vec::new();
        let mut work = Work::default();
        let mut truncated = false;
        let mut missing_required = false;
        let mut exact_found = false;
        for lane in [
            SearchRetrievalLane::ExactIdentity,
            SearchRetrievalLane::IdentifierPrefix,
            SearchRetrievalLane::LiteralText,
            SearchRetrievalLane::ExactCanonicalName,
            SearchRetrievalLane::ExactExplicitAlias,
            SearchRetrievalLane::NamespaceMember,
            SearchRetrievalLane::IdentifierSimilarity,
            SearchRetrievalLane::StructuredShape,
            SearchRetrievalLane::SeededGraph,
        ] {
            check(stop)?;
            let before = work.signals;
            let mut state = if request.enabled_lanes.contains(&lane) {
                LaneState::Complete
            } else {
                LaneState::ExcludedByRequest
            };
            if state == LaneState::Complete {
                match lane {
                    SearchRetrievalLane::ExactIdentity => {
                        if let Some(id) = exact_id {
                            if let Some(hit) = self.exact_id_lookup(id)? {
                                exact_found = true;
                                if !include(
                                    &mut candidates,
                                    hit.document(),
                                    lane,
                                    1,
                                    request,
                                    &query_id,
                                    &mut work,
                                )? {
                                    state = LaneState::Truncated;
                                }
                            }
                        } else {
                            state = LaneState::UnsupportedQueryClass;
                        }
                    }
                    SearchRetrievalLane::IdentifierPrefix => {
                        if let Some(prefix) = prefix {
                            let mut rank = 0_u32;
                            for document in &self.manifest.documents {
                                check(stop)?;
                                if work.scanned >= request.max_scanned_documents.get() {
                                    state = LaneState::Truncated;
                                    break;
                                }
                                work.scanned += 1;
                                if document.owner_metadata_label.as_str().starts_with(prefix) {
                                    rank = rank
                                        .checked_add(1)
                                        .ok_or(SearchError::FileSetBudgetExceeded)?;
                                    if !include(
                                        &mut candidates,
                                        document,
                                        lane,
                                        rank,
                                        request,
                                        &query_id,
                                        &mut work,
                                    )? {
                                        state = LaneState::Truncated;
                                    }
                                }
                            }
                        } else {
                            state = LaneState::UnsupportedQueryClass;
                        }
                    }
                    SearchRetrievalLane::LiteralText => {
                        if let Some(text) = text {
                            match self.literal_candidates(text) {
                                Ok(mut found) => {
                                    // The physical ordinal is not a semantic rank.
                                    found.sort_by(|left, right| {
                                        left.entity_id().cmp(right.entity_id())
                                    });
                                    for (index, candidate) in found.iter().enumerate() {
                                        check(stop)?;
                                        let document = self.document(
                                            candidate.entity_id().as_str(),
                                            candidate.owner_metadata_label().as_str(),
                                        )?;
                                        let rank = u32::try_from(index + 1)
                                            .map_err(|_| SearchError::FileSetBudgetExceeded)?;
                                        if !include(
                                            &mut candidates,
                                            document,
                                            lane,
                                            rank,
                                            request,
                                            &query_id,
                                            &mut work,
                                        )? {
                                            state = LaneState::Truncated;
                                        }
                                    }
                                }
                                Err(SearchError::Store(error))
                                    if error.code() == StoreErrorCode::BudgetExceeded =>
                                {
                                    state = LaneState::Truncated;
                                }
                                Err(error) => return Err(error),
                            }
                        } else {
                            state = LaneState::UnsupportedQueryClass;
                        }
                    }
                    _ => state = LaneState::MissingShardCapability,
                }
            }
            let required = request.required_lanes.contains(&lane);
            if state == LaneState::Truncated {
                truncated = true;
            }
            if required && state != LaneState::Complete && state != LaneState::Truncated {
                missing_required = true;
            }
            lanes.push(LaneResult {
                lane,
                state,
                returned_signals: work.signals - before,
                required,
            });
        }
        let mut candidates = candidates.into_values().collect::<Vec<_>>();
        for candidate in &mut candidates {
            check(stop)?;
            candidate.signals.sort_by_key(|signal| signal.lane);
            let best_rank = candidate
                .signals
                .iter()
                .map(|signal| signal.lane_rank)
                .min()
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            let rr = candidate.signals.iter().try_fold(0_u32, |sum, signal| {
                sum.checked_add(signal.reciprocal_rank_contribution)
                    .ok_or(SearchError::FileSetBudgetExceeded)
            })?;
            candidate.rank_tuple = [candidate.authority_band, best_rank, u32::MAX - rr];
        }
        candidates.sort_by(|left, right| {
            left.rank_tuple
                .cmp(&right.rank_tuple)
                .then_with(|| left.entity_id.cmp(&right.entity_id))
        });
        let state = if missing_required {
            RetrievalState::LaneUnavailable
        } else if truncated {
            RetrievalState::Truncated
        } else if exact_found {
            RetrievalState::ExactFound
        } else if !candidates.is_empty() {
            RetrievalState::CandidateOnly
        } else if exact_id.is_some() {
            RetrievalState::ExactNotFoundPartial
        } else {
            RetrievalState::NoCandidatesUnderExecutedLanes
        };
        let candidate_enumeration_complete = lanes
            .iter()
            .filter(|lane| lane.state != LaneState::ExcludedByRequest)
            .all(|lane| lane.state == LaneState::Complete);
        let payload = Payload {
            schema: RESULT_SCHEMA,
            query_profile: QUERY_PROFILE,
            ranking_profile: RANKING_PROFILE,
            normalized_query_id: query_id,
            normalized_request: request.clone(),
            original_query_data_digest: digest("", &request.query)?,
            universe_set_id,
            shard_id: self.logical_id.clone(),
            shard_profile_digest: self.manifest.profile_digest.clone(),
            owner: self.manifest.owner.clone(),
            state,
            absence_authoritative: false,
            document_coverage: "complete_explicit_selection_only",
            normalization: "utf8-case-and-punctuation-preserving/1",
            lanes,
            candidates,
            candidate_signals_omitted_by_cap: work.omitted,
            candidate_enumeration_complete,
            rr_scale: RR_SCALE,
            rr_offset: RR_OFFSET,
            rank_tuple_fields: [
                "authority_band_ascending",
                "best_canonical_lane_rank_ascending",
                "reverse_reciprocal_rank_sum_ascending",
            ],
        };
        let result_set_id = digest("search-result-set:", &payload)?;
        #[derive(Serialize)]
        struct Envelope<'a> {
            result_set_id: &'a str,
            payload: &'a Payload,
        }
        let bytes = wow_core::canonical_json_bytes(&Envelope {
            result_set_id: &result_set_id,
            payload: &payload,
        })
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if bytes.len() as u64 > request.max_manifest_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        check(stop)?;
        Ok(RetrievalResult {
            result_set_id,
            payload,
            bytes: bytes.into_boxed_slice(),
        })
    }
}

fn include(
    candidates: &mut BTreeMap<ProjectFileId, RankedCandidate>,
    document: &ProjectFileSearchDocument,
    lane: SearchRetrievalLane,
    rank: u32,
    request: &RetrievalRequest,
    query_id: &str,
    work: &mut Work,
) -> SearchResult<bool> {
    if work.signals >= request.max_signals.get() {
        return Ok(false);
    }
    if !candidates.contains_key(&document.entity_id)
        && candidates.len() >= request.max_candidates.get() as usize
    {
        work.omitted = work
            .omitted
            .checked_add(1)
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        return Ok(false);
    }
    let (band, class, evidence, field, value) = match lane {
        SearchRetrievalLane::ExactIdentity => (
            0,
            "exact_entity_identity",
            "exact_entity_identity",
            "ProjectFileRecord.file_id()",
            document.entity_id.as_str(),
        ),
        SearchRetrievalLane::IdentifierPrefix => (
            5,
            "case_sensitive_owner_metadata_label_prefix",
            "candidate",
            "ProjectFileRecord.relative_path()",
            document.owner_metadata_label.as_str(),
        ),
        SearchRetrievalLane::LiteralText => (
            7,
            "literal_owner_metadata_label_text",
            "candidate",
            "ProjectFileRecord.relative_path()",
            document.owner_metadata_label.as_str(),
        ),
        _ => return Err(SearchError::InvalidProfile),
    };
    let candidate_id = digest("search-candidate:", &(query_id, &document.entity_id))?;
    let candidate = candidates
        .entry(document.entity_id.clone())
        .or_insert_with(|| RankedCandidate {
            candidate_id,
            entity_id: document.entity_id.clone(),
            owner_metadata_label: document.owner_metadata_label.clone(),
            authority_band: band,
            rank_tuple: [0; 3],
            stable_tie_key: document.entity_id.as_str().into(),
            signals: Vec::new(),
        });
    candidate.authority_band = candidate.authority_band.min(band);
    let origin = FieldOrigin {
        owner_entity_id: document.entity_id.clone(),
        owner_record_digest: digest("", document)?,
        owner_field: field,
        original_value_digest: digest("", value)?,
        normalized_value_digest: digest("", value)?,
        normalization_profile: "utf8-case-and-punctuation-preserving/1",
    };
    let contribution = RR_SCALE
        / RR_OFFSET
            .checked_add(rank)
            .ok_or(SearchError::FileSetBudgetExceeded)?;
    let signal_id = digest(
        "search-candidate-signal:",
        &(query_id, lane, &document.entity_id, rank, &origin),
    )?;
    candidate.signals.push(Signal {
        signal_id,
        lane,
        match_class: class,
        evidence,
        lane_rank: rank,
        reciprocal_rank_contribution: contribution,
        prefix_byte_range: if lane == SearchRetrievalLane::IdentifierPrefix {
            Some([
                0,
                u32::try_from(request.parts().1.ok_or(SearchError::InvalidProfile)?.len())
                    .map_err(|_| SearchError::FileSetBudgetExceeded)?,
            ])
        } else {
            None
        },
        origin,
    });
    work.signals += 1;
    Ok(true)
}

fn digest<T: Serialize + ?Sized>(prefix: &str, value: &T) -> SearchResult<Box<str>> {
    let bytes =
        wow_core::canonical_json_bytes(value).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
    Ok(format!("{prefix}sha256:{}", hex(&Sha256::digest(bytes))).into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request() -> serde_json::Value {
        json!({"schema":REQUEST_SCHEMA,"query":{"class":"mixed","entity_id":"project-file:a.lua","prefix":"a.","text":"lua"},
            "enabled_lanes":["literal_text","identifier_prefix","exact_identity"],"required_lanes":["exact_identity"],
            "max_scanned_documents":8,"max_candidates":4,"max_signals":12,"max_manifest_bytes":65536})
    }
    fn parse(value: &serde_json::Value) -> SearchResult<RetrievalRequest> {
        RetrievalRequest::from_json_slice(
            &serde_json::to_vec(value).map_err(|_| SearchError::InvalidProfile)?,
            &AtomicBool::new(false),
        )
    }

    #[test]
    fn query_schema_rejects_hidden_fields_duplicate_and_unenabled_required_lanes() {
        let mut value = request();
        value["sql"] = json!("untrusted");
        assert!(parse(&value).is_err());
        let mut value = request();
        value["enabled_lanes"] = json!(["exact_identity", "exact_identity"]);
        assert!(parse(&value).is_err());
        let mut value = request();
        value["required_lanes"] = json!(["exact_explicit_alias"]);
        assert!(parse(&value).is_err());
    }

    #[test]
    fn optional_mixed_fields_remain_absent_in_canonical_request() -> SearchResult<()> {
        let mut value = request();
        value["query"] = json!({"class":"mixed","prefix":"a."});
        value["enabled_lanes"] = json!(["identifier_prefix"]);
        value["required_lanes"] = json!(["identifier_prefix"]);
        let parsed = parse(&value)?;
        let bytes = wow_core::canonical_json_bytes(&parsed)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let encoded: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        assert_eq!(encoded["query"], value["query"]);
        assert_eq!(
            RetrievalRequest::from_json_slice(&bytes, &AtomicBool::new(false))?,
            parsed
        );
        Ok(())
    }

    #[test]
    fn lane_order_normalization_keeps_original_case_and_literal_punctuation() -> SearchResult<()> {
        let mut first = request();
        first["query"]["text"] = json!("Name: \"Quoted\" OR literal*");
        let mut second = first.clone();
        second["enabled_lanes"] = json!(["exact_identity", "identifier_prefix", "literal_text"]);
        assert_eq!(parse(&first)?, parse(&second)?);
        assert_eq!(
            parse(&first)?.parts().2,
            Some("Name: \"Quoted\" OR literal*")
        );
        assert!(matches!(
            RetrievalRequest::from_json_slice(
                &serde_json::to_vec(&first).map_err(|_| SearchError::InvalidProfile)?,
                &AtomicBool::new(true)
            ),
            Err(SearchError::Cancelled)
        ));
        Ok(())
    }
}

//! Closed name/member/signature lanes over genuine captured callable fields.
use super::*;
use crate::SearchRetrievalLane as Lane;
mod alias_lane;
mod declaration_shape;
mod explanation;
mod field_shape;
mod filters;
pub use explanation::ReferenceCandidateExplanation;
mod pagination;
mod similar_lane;
mod similarity;
use field_shape::{FieldFilter, fields_match, validate_fields};
pub use pagination::{ReferencePage, ReferencePageCursor, ReferencePagePolicy};
const REQUEST_SCHEMA: &str = "wow-search/native-reference-query/1";
const FIELD_SHAPE_REQUEST_SCHEMA: &str = "wow-search/native-reference-field-shape-query/1";
const FIELD_SHAPE_QUERY_PROFILE: &str = "wow-search/native-reference-field-shape/1";
const DOCUMENTATION_REQUEST_SCHEMA: &str = "wow-search/native-reference-documentation-query/1";
const DOCUMENTATION_QUERY_PROFILE: &str = "wow-search/native-reference-documentation-retrieval/1";
const INVENTORY_REQUEST_SCHEMA: &str = "wow-search/native-reference-inventory-query/1";
const INVENTORY_QUERY_PROFILE: &str = "wow-search/native-reference-inventory-retrieval/1";
const INVENTORY_RANK_PROFILE: &str = "wow-search/reference-inventory-authority-ordinal-fusion/1";
const ALIAS_REQUEST_SCHEMA: &str = "wow-search/native-reference-alias-query/1";
const ALIAS_QUERY_PROFILE: &str = "wow-search/native-reference-compatibility-alias-retrieval/1";
const ALIAS_RANK_PROFILE: &str = "wow-search/reference-compatibility-alias-ordinal-fusion/1";
const FILTERED_REQUEST_SCHEMA: &str = "wow-search/native-reference-filtered-query/1";
const FILTERED_QUERY_PROFILE: &str = "wow-search/native-reference-owner-hard-filters/1";
const QUERY_PROFILE: &str = "wow-search/native-callable-name-member-shape/1";
const RANK_PROFILE: &str = "wow-search/reference-authority-ordinal-fusion/1";
const SIM_QUERY_PROFILE: &str = "wow-search/native-callable-name-member-shape-similarity/1";
const SIM_RANK_PROFILE: &str = "wow-search/reference-authority-ordinal-fusion-similarity/1";
const DOCUMENT_QUERY_PROFILE: &str = "wow-search/native-callable-event-name-payload/1";
const DOCUMENT_SIM_QUERY_PROFILE: &str =
    "wow-search/native-callable-event-name-payload-similarity/1";
const DECLARATION_QUERY_PROFILE: &str = "wow-search/native-source-declarations-name-shape/1";
const DECLARATION_SIM_QUERY_PROFILE: &str =
    "wow-search/native-source-declarations-name-shape-similarity/1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SimilarityField {
    QualifiedName,
    ShortName,
    Both,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SimilarityCase {
    Preserve,
    AsciiFold,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SimilaritySelector {
    identifier: String,
    field: SimilarityField,
    case_policy: SimilarityCase,
    max_distance: u32,
    max_prefiltered_documents: NonZeroU32,
    max_distance_cells: NonZeroU64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    namespace: String,
    member: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Shape {
    #[serde(skip_serializing_if = "Option::is_none")]
    restriction: Option<restrictions::RestrictionShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    declaration: Option<declaration_shape::DeclarationShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    argument_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    arguments: Vec<FieldFilter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    returns: Vec<FieldFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_payload_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    event_payload: Vec<FieldFilter>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Selectors {
    #[serde(skip_serializing_if = "Option::is_none")]
    explicit_alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    identifier_similarity: Option<SimilaritySelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qualified_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace_member: Option<Member>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    literal_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shape: Option<Shape>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    base_component: Option<filters::Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<filters::Filters>,
    schema: String,
    selectors: Selectors,
    enabled_lanes: Vec<Lane>,
    required_lanes: Vec<Lane>,
    max_scanned_documents: NonZeroU32,
    max_candidates: NonZeroU32,
    max_signals: NonZeroU32,
    max_result_bytes: NonZeroU64,
}
impl ReferenceQueryRequest {
    fn effective_schema(&self) -> &str {
        if self.schema == FILTERED_REQUEST_SCHEMA {
            self.base_component.map_or("", filters::Component::schema)
        } else {
            &self.schema
        }
    }
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> SearchResult<Self> {
        check(stop)?;
        if bytes.len() > 65536 {
            return Err(SearchError::InvalidProfile);
        }
        let mut request: Self =
            serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
        request.enabled_lanes.sort();
        request.required_lanes.sort();
        if !matches!(
            request.schema.as_str(),
            REQUEST_SCHEMA
                | FIELD_SHAPE_REQUEST_SCHEMA
                | DOCUMENTATION_REQUEST_SCHEMA
                | INVENTORY_REQUEST_SCHEMA
                | ALIAS_REQUEST_SCHEMA
                | FILTERED_REQUEST_SCHEMA
        ) || request.enabled_lanes.is_empty()
            || request.enabled_lanes.len() > 9
            || request.required_lanes.len() > 9
            || request.enabled_lanes.windows(2).any(|p| p[0] == p[1])
            || request.required_lanes.windows(2).any(|p| p[0] == p[1])
            || request
                .required_lanes
                .iter()
                .any(|l| !request.enabled_lanes.contains(l))
        {
            return Err(SearchError::InvalidProfile);
        }
        if request.schema == FILTERED_REQUEST_SCHEMA {
            if request.base_component.is_none() {
                return Err(SearchError::InvalidProfile);
            }
            request
                .filters
                .as_mut()
                .ok_or(SearchError::InvalidProfile)?
                .validate()?;
        } else {
            let raw: serde_json::Value =
                serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
            if raw.as_object().is_some_and(|root| {
                root.contains_key("filters") || root.contains_key("base_component")
            }) {
                return Err(SearchError::InvalidProfile);
            }
        }
        if request.effective_schema() == REQUEST_SCHEMA {
            validate_legacy_shape(bytes)?;
        }
        if request.effective_schema() != INVENTORY_REQUEST_SCHEMA
            && request.effective_schema() != ALIAS_REQUEST_SCHEMA
        {
            let value: serde_json::Value =
                serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
            if value
                .pointer("/selectors/shape")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|shape| shape.contains_key("restriction"))
            {
                return Err(SearchError::InvalidProfile);
            }
        }
        let s = &request.selectors;
        if request.effective_schema() != ALIAS_REQUEST_SCHEMA {
            let value: serde_json::Value =
                serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
            if value
                .get("selectors")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|selectors| selectors.contains_key("explicit_alias"))
            {
                return Err(SearchError::InvalidProfile);
            }
        }
        if let Some(key) = &s.entity_key {
            wow_reference::ReferenceRecord::validate_key(key)
                .map_err(|_| SearchError::InvalidProfile)?;
        }
        let present = [
            (s.explicit_alias.is_some(), Lane::ExactExplicitAlias),
            (s.entity_key.is_some(), Lane::ExactIdentity),
            (
                s.qualified_name.is_some() || s.short_name.is_some(),
                Lane::ExactCanonicalName,
            ),
            (s.namespace_member.is_some(), Lane::NamespaceMember),
            (s.prefix.is_some(), Lane::IdentifierPrefix),
            (s.literal_text.is_some(), Lane::LiteralText),
            (s.shape.is_some(), Lane::StructuredShape),
            (
                s.identifier_similarity.is_some(),
                Lane::IdentifierSimilarity,
            ),
        ];
        if !present.iter().any(|(p, _)| *p)
            || present
                .iter()
                .any(|(p, l)| *p && !request.enabled_lanes.contains(l))
        {
            return Err(SearchError::InvalidProfile);
        }
        for value in [
            &s.explicit_alias,
            &s.entity_key,
            &s.qualified_name,
            &s.short_name,
            &s.prefix,
            &s.literal_text,
        ]
        .into_iter()
        .flatten()
        {
            valid_text(value)?;
        }
        if s.prefix.as_ref().is_some_and(|p| p.len() < 2) {
            return Err(SearchError::InvalidProfile);
        }
        if let Some(m) = &s.namespace_member {
            valid_text(&m.namespace)?;
            valid_text(&m.member)?;
        }
        if let Some(shape) = &s.shape {
            if (shape.argument_count.is_none()
                && shape.return_count.is_none()
                && shape.arguments.is_empty()
                && shape.returns.is_empty()
                && shape.event_payload_count.is_none()
                && shape.event_payload.is_empty()
                && shape.declaration.is_none())
                && shape.restriction.is_none()
                || shape.arguments.len() > 128
                || shape.returns.len() > 128
                || shape.event_payload.len() > 128
                || ((shape.argument_count.is_some()
                    || shape.return_count.is_some()
                    || !shape.arguments.is_empty()
                    || !shape.returns.is_empty())
                    && (shape.event_payload_count.is_some() || !shape.event_payload.is_empty()))
            {
                return Err(SearchError::InvalidProfile);
            }
            if let Some(declaration) = &shape.declaration {
                if shape.argument_count.is_some()
                    || shape.return_count.is_some()
                    || !shape.arguments.is_empty()
                    || !shape.returns.is_empty()
                    || shape.event_payload_count.is_some()
                    || !shape.event_payload.is_empty()
                    || shape.restriction.is_some()
                {
                    return Err(SearchError::InvalidProfile);
                }
                declaration.validate()?;
            }
            if let Some(restriction) = &shape.restriction {
                if shape.declaration.is_some()
                    || shape.argument_count.is_some()
                    || shape.return_count.is_some()
                    || !shape.arguments.is_empty()
                    || !shape.returns.is_empty()
                    || shape.event_payload_count.is_some()
                    || !shape.event_payload.is_empty()
                {
                    return Err(SearchError::InvalidProfile);
                }
                restriction.validate()?;
            }
            for fields in [&shape.arguments, &shape.returns, &shape.event_payload] {
                validate_fields(fields)?;
            }
        }
        if let Some(similarity) = &s.identifier_similarity {
            valid_text(&similarity.identifier)?;
            let length = similarity.identifier.chars().count();
            if !(2..=128).contains(&length)
                || similarity.max_distance > 4
                || similarity.max_distance > length as u32
            {
                return Err(SearchError::InvalidProfile);
            }
        }
        check(stop)?;
        Ok(request)
    }
}

// Preserve the old closed schema, including explicit empty/null extension keys.
// Typed decoding above still rejects duplicate fields before this compatibility check.
fn validate_legacy_shape(bytes: &[u8]) -> SearchResult<()> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
    let Some(shape) = value
        .pointer("/selectors/shape")
        .and_then(serde_json::Value::as_object)
    else {
        return Ok(());
    };
    if shape.contains_key("returns") {
        return Err(SearchError::InvalidProfile);
    }
    let contains_inner = |object: &serde_json::Map<String, serde_json::Value>, key: &str| {
        object
            .get(key)
            .and_then(serde_json::Value::as_array)
            .is_some_and(|fields| {
                fields.iter().any(|field| {
                    field
                        .as_object()
                        .is_some_and(|field| field.contains_key("inner_type"))
                })
            })
    };
    if ["arguments", "event_payload"]
        .iter()
        .any(|key| contains_inner(shape, key))
        || shape
            .get("declaration")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|declaration| {
                ["fields", "arguments", "returns"]
                    .iter()
                    .any(|key| contains_inner(declaration, key))
            })
    {
        return Err(SearchError::InvalidProfile);
    }
    Ok(())
}
pub(super) fn valid_text(value: &str) -> SearchResult<()> {
    if value.trim().is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
        Err(SearchError::InvalidProfile)
    } else {
        Ok(())
    }
}

#[derive(Serialize)]
struct FilterScope {
    unavailable_kinds: Vec<ReferenceRecordKind>,
    predicate_evaluations: u64,
    excluded_evaluations: u64,
}
fn filter_accepts(
    request: &ReferenceQueryRequest,
    document: &Document,
    scope: &mut Option<FilterScope>,
) -> SearchResult<bool> {
    let Some(filters) = &request.filters else {
        return Ok(true);
    };
    let scope = scope.as_mut().ok_or(SearchError::OwnerEvidenceInvalid)?;
    scope.predicate_evaluations = scope
        .predicate_evaluations
        .checked_add(1)
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    let accepted = filters.accepts(document);
    if !accepted {
        scope.excluded_evaluations = scope
            .excluded_evaluations
            .checked_add(1)
            .ok_or(SearchError::FileSetBudgetExceeded)?;
    }
    Ok(accepted)
}

#[derive(Serialize)]
struct Origin {
    owner_field: &'static str,
    owner_record_digest: String,
    source_id: String,
    original_value_digest: Box<str>,
    normalized_value_digest: Box<str>,
    normalization: &'static str,
}
#[derive(Serialize)]
struct Signal {
    #[serde(skip_serializing_if = "Option::is_none")]
    explicit_alias: Option<compatibility_aliases::AliasEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    identifier_similarity: Option<similar_lane::SimilarityEvidence>,
    lane: Lane,
    match_class: &'static str,
    authority_band: u32,
    lane_rank: u32,
    reciprocal_rank_contribution: u32,
    origin: Origin,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix_byte_range: Option<[usize; 2]>,
}
#[derive(Serialize)]
struct Candidate<'a> {
    candidate_id: Box<str>,
    document: &'a Document,
    rank_tuple: [u32; 3],
    stable_tie_key: &'a str,
    signals: Vec<Signal>,
}
#[derive(Serialize)]
struct LaneResult {
    lane: Lane,
    state: &'static str,
    required: bool,
    returned_signals: u32,
}
pub struct ReferenceQueryResult {
    bytes: Box<[u8]>,
    status: &'static str,
}
impl ReferenceQueryResult {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn status(&self) -> &'static str {
        self.status
    }
}

impl ReferenceSearchShard {
    pub fn retrieve(
        &self,
        request: &ReferenceQueryRequest,
        owner: &ReferenceView,
        stop: &AtomicBool,
    ) -> SearchResult<ReferenceQueryResult> {
        check(stop)?;
        self.validate_query_component(request)?;
        if owner.generation_id() != self.manifest.reference_generation
            || owner.self_digest() != self.manifest.reference_view_digest
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let shard_id = self.physical.search_shard_id();
        let universe = identity(
            "reference-search-universe:",
            &(
                &self.manifest.binding,
                &self.manifest.reference_generation,
                &self.manifest.reference_view_digest,
            ),
        )?;
        let query_profile = capture_query_profile(self.manifest.native_documents, request);
        let ranking_profile = if self.manifest.compatibility_aliases.is_some() {
            ALIAS_RANK_PROFILE
        } else if self.manifest.include_restrictions.is_some() {
            INVENTORY_RANK_PROFILE
        } else if request.selectors.identifier_similarity.is_some() {
            SIM_RANK_PROFILE
        } else {
            RANK_PROFILE
        };
        let query_id = identity(
            "reference-search-query:",
            &(query_profile, shard_id, request),
        )?;
        let mut candidates = BTreeMap::<&str, Candidate<'_>>::new();
        let mut lanes = Vec::new();
        let mut scanned = 0u32;
        let mut signals = 0u32;
        let mut omitted = 0u32;
        let mut truncated = false;
        let mut filter_scope = request.filters.as_ref().map(|filters| FilterScope {
            unavailable_kinds: filters.unsupported_kinds(&self.manifest),
            predicate_evaluations: 0,
            excluded_evaluations: 0,
        });
        let filter_unavailable = filter_scope
            .as_ref()
            .is_some_and(|scope| !scope.unavailable_kinds.is_empty());
        let mut missing = filter_unavailable;
        let mut similarity_work = None;
        let mut alias_match = None;
        for lane in [
            Lane::ExactIdentity,
            Lane::ExactCanonicalName,
            Lane::NamespaceMember,
            Lane::IdentifierPrefix,
            Lane::LiteralText,
            Lane::StructuredShape,
            Lane::ExactExplicitAlias,
            Lane::IdentifierSimilarity,
            Lane::SeededGraph,
        ] {
            check(stop)?;
            let before = signals;
            let mut state = "excluded_by_request";
            if request.enabled_lanes.contains(&lane) {
                let s = &request.selectors;
                let supported = match lane {
                    Lane::ExactIdentity => s.entity_key.is_some(),
                    Lane::ExactCanonicalName => {
                        s.qualified_name.is_some() || s.short_name.is_some()
                    }
                    Lane::NamespaceMember => s.namespace_member.is_some(),
                    Lane::IdentifierPrefix => s.prefix.is_some(),
                    Lane::LiteralText => s.literal_text.is_some(),
                    Lane::StructuredShape => s.shape.is_some(),
                    Lane::IdentifierSimilarity => s.identifier_similarity.is_some(),
                    Lane::ExactExplicitAlias => s.explicit_alias.is_some(),
                    _ => false,
                };
                state = if filter_unavailable {
                    "missing_filter_capability"
                } else if lane == Lane::SeededGraph
                    || (lane == Lane::ExactExplicitAlias
                        && self.manifest.compatibility_aliases.is_none())
                    || (lane == Lane::StructuredShape
                        && self.manifest.include_restrictions.is_none()
                        && s.shape
                            .as_ref()
                            .is_some_and(|shape| shape.restriction.is_some()))
                    || (lane == Lane::IdentifierSimilarity && s.identifier_similarity.is_none())
                    || (lane == Lane::StructuredShape
                        && self.manifest.native_documents.is_none()
                        && s.shape.as_ref().is_some_and(|shape| {
                            shape.event_payload_count.is_some() || !shape.event_payload.is_empty()
                        }))
                    || (lane == Lane::StructuredShape
                        && self.manifest.native_documents
                            != Some(NativeReferenceDocuments::SourceDeclarationsV1)
                        && s.shape
                            .as_ref()
                            .is_some_and(|shape| shape.declaration.is_some()))
                {
                    "missing_shard_capability"
                } else if !supported {
                    "unsupported_query_class"
                } else {
                    "complete"
                };
                if state == "complete" && lane == Lane::ExactExplicitAlias {
                    let (lane_state, report) = alias_lane::run(
                        &self.manifest,
                        request,
                        &universe,
                        &query_id,
                        &mut candidates,
                        &mut scanned,
                        &mut signals,
                        &mut omitted,
                        &mut filter_scope,
                        stop,
                    )?;
                    state = lane_state;
                    alias_match = Some(report);
                } else if state == "complete" && lane == Lane::IdentifierSimilarity {
                    let (lane_state, work) = similar_lane::run(
                        &self.manifest.documents,
                        request,
                        &universe,
                        &query_id,
                        &mut candidates,
                        &mut scanned,
                        &mut signals,
                        &mut omitted,
                        &mut filter_scope,
                        stop,
                    )?;
                    state = lane_state;
                    similarity_work = Some(work);
                } else if state == "complete" {
                    let mut text_ids = std::collections::BTreeSet::new();
                    if lane == Lane::LiteralText {
                        match self.physical.reader().literal_candidates(
                            s.literal_text
                                .as_deref()
                                .ok_or(SearchError::InvalidProfile)?,
                        ) {
                            Ok(rows) => {
                                for row in rows {
                                    text_ids.insert(row.document_id().to_owned());
                                }
                            }
                            Err(error) if error.code() == StoreErrorCode::BudgetExceeded => {
                                state = "truncated"
                            }
                            Err(error) => return Err(SearchError::Store(error)),
                        }
                    }
                    let mut rank = 0u32;
                    if state == "complete" {
                        for document in &self.manifest.documents {
                            check(stop)?;
                            if scanned >= request.max_scanned_documents.get() {
                                state = "truncated";
                                break;
                            }
                            scanned += 1;
                            if !filter_accepts(request, document, &mut filter_scope)? {
                                continue;
                            }
                            let matched: Option<MatchMaterial> = match lane {
                                Lane::ExactIdentity
                                    if s.entity_key.as_deref() == Some(&document.entity_key) =>
                                {
                                    Some((
                                        0,
                                        "exact_entity_identity",
                                        "ReferenceRecord.key()",
                                        identity("", &document.entity_key)?,
                                        identity("", &document.entity_key)?,
                                    ))
                                }
                                Lane::ExactCanonicalName
                                    if document.restriction.is_none()
                                        && s.qualified_name.as_deref()
                                            == Some(&document.canonical_qualified_name) =>
                                {
                                    Some((
                                        1,
                                        if document.event.is_some() {
                                            "exact_event_literal_name"
                                        } else {
                                            "exact_canonical_qualified_name"
                                        },
                                        name_origin(document, true),
                                        name_digest(document)?,
                                        identity("", &document.canonical_qualified_name)?,
                                    ))
                                }
                                Lane::ExactCanonicalName
                                    if document.restriction.is_none()
                                        && s.short_name.as_deref()
                                            == Some(&document.canonical_short_name) =>
                                {
                                    Some((
                                        2,
                                        if document.event.is_some() {
                                            "exact_event_literal_name"
                                        } else {
                                            "exact_canonical_short_name"
                                        },
                                        name_origin(document, false),
                                        identity("", &document.canonical_short_name)?,
                                        identity("", &document.canonical_short_name)?,
                                    ))
                                }
                                Lane::NamespaceMember
                                    if s.namespace_member.as_ref().is_some_and(|m| {
                                        document.namespace.as_deref() == Some(&m.namespace)
                                            && document.canonical_short_name == m.member
                                    }) =>
                                {
                                    Some((
                                        4,
                                        "exact_namespace_member",
                                        "NativeCallableSignature.namespace()/name()",
                                        name_digest(document)?,
                                        name_digest(document)?,
                                    ))
                                }
                                Lane::IdentifierPrefix
                                    if s.prefix.as_ref().is_some_and(|p| {
                                        document.canonical_qualified_name.starts_with(p)
                                            || document.canonical_short_name.starts_with(p)
                                    }) =>
                                {
                                    let qualified = s.prefix.as_ref().is_some_and(|p| {
                                        document.canonical_qualified_name.starts_with(p)
                                    });
                                    Some((
                                        5,
                                        "case_sensitive_identifier_prefix_candidate",
                                        name_origin(document, qualified),
                                        if qualified {
                                            name_digest(document)?
                                        } else {
                                            identity("", &document.canonical_short_name)?
                                        },
                                        identity(
                                            "",
                                            if qualified {
                                                &document.canonical_qualified_name
                                            } else {
                                                &document.canonical_short_name
                                            },
                                        )?,
                                    ))
                                }
                                Lane::LiteralText if text_ids.contains(&document.entity_key) => {
                                    Some(if self.manifest.include_documentation.is_some() {
                                        (
                                            7,
                                            "literal_reference_documentation_candidate",
                                            documentation::origin(document),
                                            identity("", &document.documentation)?,
                                            identity(
                                                "",
                                                &documentation::row_text(document, self.bounds)?,
                                            )?,
                                        )
                                    } else {
                                        (
                                            7,
                                            "literal_canonical_identifier_candidate",
                                            name_origin(document, true),
                                            name_digest(document)?,
                                            identity("", &document.canonical_qualified_name)?,
                                        )
                                    })
                                }
                                Lane::StructuredShape
                                    if s.shape
                                        .as_ref()
                                        .is_some_and(|shape| shape_matches(document, shape)) =>
                                {
                                    Some((
                                        7,
                                        if document.restriction.is_some() {
                                            "native_restriction_shape_candidate"
                                        } else {
                                            "original_signature_shape_candidate"
                                        },
                                        if document.restriction.is_some() {
                                            "NativeRestrictionSignature.kind()/facets()/subject_key()"
                                        } else if document.declaration.is_some() {
                                            "NativeDeclarationSignature.kind()+fields()+arguments()+returns()+values()"
                                        } else if document.event.is_some() {
                                            "NativeEventSignature.payload()"
                                        } else {
                                            "NativeCallableSignature.arguments()/returns()"
                                        },
                                        shape_digest(document)?,
                                        shape_digest(document)?,
                                    ))
                                }
                                _ => None,
                            };
                            let mut matches = matched.into_iter().collect::<Vec<_>>();
                            if lane == Lane::ExactCanonicalName
                                && document.restriction.is_none()
                                && s.qualified_name.as_deref()
                                    == Some(&document.canonical_qualified_name)
                                && s.short_name.as_deref() == Some(&document.canonical_short_name)
                            {
                                matches.push((
                                    2,
                                    if document.event.is_some() {
                                        "exact_event_literal_name"
                                    } else {
                                        "exact_canonical_short_name"
                                    },
                                    name_origin(document, false),
                                    identity("", &document.canonical_short_name)?,
                                    identity("", &document.canonical_short_name)?,
                                ));
                            }
                            for (band, class, field, original, normalized) in matches {
                                rank = rank
                                    .checked_add(1)
                                    .ok_or(SearchError::FileSetBudgetExceeded)?;
                                if signals >= request.max_signals.get() {
                                    state = "truncated";
                                    break;
                                }
                                if !include_signal(
                                    &mut candidates,
                                    document,
                                    lane,
                                    rank,
                                    (band, class, field, original, normalized),
                                    request,
                                    &universe,
                                    &query_id,
                                    &mut signals,
                                    &mut omitted,
                                    None,
                                    None,
                                )? {
                                    state = "truncated";
                                }
                            }
                        }
                    }
                }
            }
            let required = request.required_lanes.contains(&lane);
            truncated |= state == "truncated";
            missing |= required && !matches!(state, "complete" | "truncated");
            lanes.push(LaneResult {
                lane,
                state,
                required,
                returned_signals: signals - before,
            });
        }
        let mut candidates = candidates.into_values().collect::<Vec<_>>();
        candidates.sort_by(|a, b| {
            a.rank_tuple
                .cmp(&b.rank_tuple)
                .then_with(|| a.stable_tie_key.cmp(b.stable_tie_key))
        });
        let enumeration_complete = lanes
            .iter()
            .filter(|l| l.state != "excluded_by_request")
            .all(|l| l.state == "complete");
        let exact = request.selectors.entity_key.is_some()
            || request.selectors.explicit_alias.is_some()
            || request.selectors.qualified_name.is_some()
            || request.selectors.short_name.is_some()
            || request.selectors.namespace_member.is_some();
        let conflict = request.selectors.entity_key.as_ref().is_some_and(|key| {
            self.manifest.conflicts.iter().any(|c| {
                (c.partition_id() == NATIVE_API_PARTITION
                    || (self.manifest.native_documents.is_some()
                        && c.partition_id() == NATIVE_EVENT_PARTITION)
                    || (self.manifest.native_documents
                        == Some(NativeReferenceDocuments::SourceDeclarationsV1)
                        && c.partition_id() == NATIVE_DECLARATION_PARTITION))
                    && c.key() == key
                    && request
                        .filters
                        .as_ref()
                        .is_none_or(|filters| match c.partition_id() {
                            NATIVE_API_PARTITION => {
                                filters.entity_kinds.contains(&ReferenceRecordKind::Api)
                            }
                            NATIVE_EVENT_PARTITION => {
                                filters.entity_kinds.contains(&ReferenceRecordKind::Event)
                            }
                            NATIVE_DECLARATION_PARTITION => {
                                filters.entity_kinds.iter().any(|kind| {
                                    matches!(
                                        kind,
                                        ReferenceRecordKind::Event
                                            | ReferenceRecordKind::Other
                                            | ReferenceRecordKind::Enumeration
                                            | ReferenceRecordKind::Widget
                                    )
                                })
                            }
                            _ => false,
                        })
            })
        });
        let authoritative = !missing
            && request
                .filters
                .as_ref()
                .is_none_or(|filters| filters.entity_kinds.contains(&ReferenceRecordKind::Api))
            && !truncated
            && enumeration_complete
            && !self.manifest.binding.acquisition_partial
            && self.manifest.owner_negative_authority
            && self.manifest.native_documents.is_none()
            && request
                .selectors
                .entity_key
                .as_ref()
                .is_some_and(|key| owner.is_authoritatively_absent(NATIVE_API_PARTITION, key));
        let status = if missing {
            "lane_unavailable"
        } else if truncated {
            "truncated"
        } else if request.required_lanes.contains(&Lane::ExactExplicitAlias)
            && alias_match
                .as_ref()
                .is_some_and(|report| report.conflicted > 0)
        {
            "conflict_blocked"
        } else if request.required_lanes.contains(&Lane::ExactExplicitAlias)
            && alias_match
                .as_ref()
                .is_some_and(|report| report.unresolved > 0)
        {
            "exact_not_found_partial"
        } else if candidates.iter().any(|c| c.rank_tuple[0] <= 4) {
            "exact_found"
        } else if conflict {
            "conflict_blocked"
        } else if !candidates.is_empty() {
            "candidate_only"
        } else if authoritative {
            "exact_not_found_with_authority"
        } else if exact {
            "exact_not_found_partial"
        } else {
            "no_candidates_under_executed_lanes"
        };
        #[derive(Serialize)]
        struct OwnerSummary<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            compatibility_aliases: Option<compatibility_aliases::AliasSummary<'a>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            include_restrictions: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            include_documentation: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            documentation_coverage: Option<documentation::Coverage>,
            #[serde(skip_serializing_if = "Option::is_none")]
            native_documents: Option<NativeReferenceDocuments>,
            schema: &'a str,
            profile_digest: &'a str,
            binding: &'a ReferenceSearchBinding,
            reference_generation: &'a str,
            reference_view_digest: &'a str,
            native_projection_profile: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            available_type_resolution_profile: Option<&'a str>,
            source_origins: &'a [SourceOrigin],
            partitions: &'a [Partition],
            conflicts: &'a [ReferenceConflict],
            issues: &'a [ProjectionIssue],
            owner_negative_authority: bool,
            projected_document_count: usize,
            privacy_scope: &'a str,
            license_state: &'a str,
        }
        let summary = OwnerSummary {
            compatibility_aliases: compatibility_aliases::summary(
                self.manifest.compatibility_aliases.as_ref(),
            ),
            include_restrictions: self.manifest.include_restrictions,
            include_documentation: self.manifest.include_documentation,
            documentation_coverage: documentation::coverage(&self.manifest),
            native_documents: self.manifest.native_documents,
            schema: &self.manifest.schema,
            profile_digest: &self.manifest.profile_digest,
            binding: &self.manifest.binding,
            reference_generation: &self.manifest.reference_generation,
            reference_view_digest: &self.manifest.reference_view_digest,
            native_projection_profile: &self.manifest.native_projection_profile,
            available_type_resolution_profile: self
                .manifest
                .available_type_resolution_profile
                .as_deref(),
            source_origins: &self.manifest.source_origins,
            partitions: &self.manifest.partitions,
            conflicts: &self.manifest.conflicts,
            issues: &self.manifest.issues,
            owner_negative_authority: self.manifest.owner_negative_authority,
            projected_document_count: self.manifest.documents.len(),
            privacy_scope: &self.manifest.privacy_scope,
            license_state: &self.manifest.license_state,
        };
        #[derive(Serialize)]
        struct Payload<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            filter_scope: Option<FilterScope>,
            #[serde(skip_serializing_if = "Option::is_none")]
            alias_match: Option<alias_lane::AliasMatch>,
            schema: &'static str,
            query_profile: &'static str,
            ranking_profile: &'static str,
            normalized_query_id: &'a str,
            universe_id: &'a str,
            shard_id: &'a str,
            request: &'a ReferenceQueryRequest,
            owner: OwnerSummary<'a>,
            status: &'static str,
            absence_authoritative: bool,
            candidate_enumeration_complete: bool,
            candidate_signals_omitted_by_cap: u32,
            scanned_documents: u32,
            rr_scale: u32,
            rr_offset: u32,
            candidates: &'a [Candidate<'a>],
            #[serde(skip_serializing_if = "Option::is_none")]
            identifier_similarity_work: Option<similar_lane::SimilarityWork>,
        }
        // Owner summary excludes the full document list; each selected candidate
        // carries its whole document. Nonmatching source fields are not delivered.
        let payload = Payload {
            filter_scope,
            alias_match,
            schema: "wow-search/native-reference-result-set/1",
            query_profile,
            ranking_profile,
            normalized_query_id: &query_id,
            universe_id: &universe,
            shard_id,
            request,
            owner: summary,
            status,
            absence_authoritative: authoritative && candidates.is_empty(),
            candidate_enumeration_complete: enumeration_complete,
            candidate_signals_omitted_by_cap: omitted,
            scanned_documents: scanned,
            rr_scale: 1_000_000,
            rr_offset: 60,
            candidates: &candidates,
            identifier_similarity_work: similarity_work,
        };
        let result_set_id = identity("reference-search-result-set:", &payload)?;
        #[derive(Serialize)]
        struct Envelope<'a, T: Serialize> {
            result_set_id: &'a str,
            payload: &'a T,
        }
        let bytes = wow_core::canonical_json_bytes(&Envelope {
            result_set_id: &result_set_id,
            payload: &payload,
        })
        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        if bytes.len() as u64 > request.max_result_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        check(stop)?;
        Ok(ReferenceQueryResult {
            bytes: bytes.into_boxed_slice(),
            status,
        })
    }
}
type MatchMaterial = (u32, &'static str, &'static str, Box<str>, Box<str>);
#[allow(clippy::too_many_arguments)]
fn include_signal<'a>(
    candidates: &mut BTreeMap<&'a str, Candidate<'a>>,
    document: &'a Document,
    lane: Lane,
    rank: u32,
    material: MatchMaterial,
    request: &ReferenceQueryRequest,
    universe: &str,
    query_id: &str,
    signals: &mut u32,
    omitted: &mut u32,
    similarity: Option<similar_lane::SimilarityEvidence>,
    explicit_alias: Option<compatibility_aliases::AliasEvidence>,
) -> SearchResult<bool> {
    if *signals >= request.max_signals.get() {
        return Ok(false);
    }
    if !candidates.contains_key(document.entity_key.as_str())
        && candidates.len() >= request.max_candidates.get() as usize
    {
        *omitted = omitted
            .checked_add(1)
            .ok_or(SearchError::FileSetBudgetExceeded)?;
        return Ok(false);
    }
    let (band, class, field, original, normalized) = material;
    let band = if band != 0
        && document
            .restriction
            .as_ref()
            .is_some_and(|data| data.subject_record_digest.is_none())
    {
        band.max(10)
    } else {
        band
    };
    let candidate_id = identity(
        "reference-search-candidate:",
        &(universe, query_id, &document.entity_key),
    )?;
    let candidate = candidates.entry(&document.entity_key).or_insert(Candidate {
        candidate_id,
        document,
        rank_tuple: [band, u32::MAX, u32::MAX],
        stable_tie_key: &document.entity_key,
        signals: Vec::new(),
    });
    let rr = 1_000_000
        / (60u32
            .checked_add(rank)
            .ok_or(SearchError::FileSetBudgetExceeded)?);
    candidate.rank_tuple[0] = candidate.rank_tuple[0].min(band);
    candidate.rank_tuple[1] = candidate.rank_tuple[1].min(rank);
    candidate.rank_tuple[2] = candidate.rank_tuple[2]
        .checked_sub(rr)
        .ok_or(SearchError::FileSetBudgetExceeded)?;
    let normalization = similarity.as_ref().map_or_else(
        || {
            if class == "literal_reference_documentation_candidate" {
                documentation::INDEX_PROFILE
            } else if field == "NativeCallableSignature.namespace()+name()" {
                "owner-namespace-dot-name/1"
            } else {
                "case-preserving-owner-fields/1"
            }
        },
        |proof| proof.normalization_profile,
    );
    let (origin_digest, origin_source) = if let Some(alias) = &explicit_alias {
        (
            alias.record_digest.clone(),
            alias
                .source_ids
                .first()
                .ok_or(SearchError::OwnerEvidenceInvalid)?
                .clone(),
        )
    } else {
        (
            document.owner_record_digest.clone(),
            document.source.source_id.clone(),
        )
    };
    candidate.signals.push(Signal {
        explicit_alias: explicit_alias.clone(),
        lane,
        match_class: class,
        authority_band: band,
        lane_rank: rank,
        reciprocal_rank_contribution: rr,
        origin: Origin {
            owner_field: field,
            owner_record_digest: origin_digest,
            source_id: origin_source,
            original_value_digest: original,
            normalized_value_digest: normalized,
            normalization,
        },
        prefix_byte_range: if lane == Lane::IdentifierPrefix {
            Some([
                0,
                request
                    .selectors
                    .prefix
                    .as_ref()
                    .ok_or(SearchError::InvalidProfile)?
                    .len(),
            ])
        } else {
            None
        },
        identifier_similarity: similarity,
    });
    *signals += 1;
    Ok(true)
}
fn name_digest(document: &Document) -> SearchResult<Box<str>> {
    if document.restriction.is_some() {
        return identity("", &document.canonical_short_name);
    }
    if let Some(declaration) = &document.declaration {
        return identity("", &declaration.name);
    }
    if let Some(event) = &document.event {
        return identity("", &event.literal_name);
    }
    #[derive(Serialize)]
    struct Name<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        namespace: Option<&'a str>,
        name: &'a str,
    }
    identity(
        "",
        &Name {
            namespace: document.namespace.as_deref(),
            name: &document.canonical_short_name,
        },
    )
}
fn shape_matches(document: &Document, shape: &Shape) -> bool {
    if let Some(filter) = &shape.restriction {
        return document
            .restriction
            .as_ref()
            .is_some_and(|data| filter.matches(data));
    }
    if document.restriction.is_some() {
        return false;
    }
    if let Some(declaration) = &shape.declaration {
        return document
            .declaration
            .as_ref()
            .is_some_and(|data| declaration.matches(data));
    }
    if document.declaration.is_some() {
        return false;
    }
    if let Some(event) = &document.event {
        return shape.argument_count.is_none()
            && shape.return_count.is_none()
            && shape.arguments.is_empty()
            && shape.returns.is_empty()
            && shape
                .event_payload_count
                .is_none_or(|count| count as usize == event.payload.len())
            && fields_match(&event.payload, &shape.event_payload);
    }
    if shape.event_payload_count.is_some() || !shape.event_payload.is_empty() {
        return false;
    }
    shape
        .argument_count
        .is_none_or(|n| n as usize == document.arguments.len())
        && shape
            .return_count
            .is_none_or(|n| n as usize == document.returns.len())
        && fields_match(&document.arguments, &shape.arguments)
        && fields_match(&document.returns, &shape.returns)
}
fn shape_digest(document: &Document) -> SearchResult<Box<str>> {
    if let Some(restriction) = &document.restriction {
        identity("", restriction)
    } else if let Some(declaration) = &document.declaration {
        identity("", declaration)
    } else if let Some(event) = &document.event {
        identity("", &event.payload)
    } else {
        identity("", &(&document.arguments, &document.returns))
    }
}
fn name_origin(document: &Document, qualified: bool) -> &'static str {
    if document.restriction.is_some() {
        "NativeRestrictionSignature.facets().id()"
    } else if document.declaration.is_some() {
        "NativeDeclarationSignature.name()"
    } else if document.event.is_some() {
        "NativeEventSignature.literal_name()"
    } else if qualified {
        "NativeCallableSignature.namespace()+name()"
    } else {
        "NativeCallableSignature.name()"
    }
}

fn capture_query_profile(
    selection: Option<NativeReferenceDocuments>,
    request: &ReferenceQueryRequest,
) -> &'static str {
    if request.schema == FILTERED_REQUEST_SCHEMA {
        return FILTERED_QUERY_PROFILE;
    }
    if request.schema == ALIAS_REQUEST_SCHEMA {
        return ALIAS_QUERY_PROFILE;
    }
    if request.schema == INVENTORY_REQUEST_SCHEMA {
        return INVENTORY_QUERY_PROFILE;
    }
    if request.schema == DOCUMENTATION_REQUEST_SCHEMA {
        return DOCUMENTATION_QUERY_PROFILE;
    }
    if request.schema == FIELD_SHAPE_REQUEST_SCHEMA {
        return FIELD_SHAPE_QUERY_PROFILE;
    }
    let similar = request.selectors.identifier_similarity.is_some();
    match (selection, similar) {
        (Some(NativeReferenceDocuments::SourceDeclarationsV1), true) => {
            DECLARATION_SIM_QUERY_PROFILE
        }
        (Some(NativeReferenceDocuments::SourceDeclarationsV1), false) => DECLARATION_QUERY_PROFILE,
        (Some(NativeReferenceDocuments::CallablesAndEventsV1), true) => DOCUMENT_SIM_QUERY_PROFILE,
        (Some(NativeReferenceDocuments::CallablesAndEventsV1), false) => DOCUMENT_QUERY_PROFILE,
        (None, true) => SIM_QUERY_PROFILE,
        (None, false) => QUERY_PROFILE,
    }
}
impl ReferenceSearchShard {
    fn validate_query_component(&self, request: &ReferenceQueryRequest) -> SearchResult<()> {
        if self.manifest.compatibility_aliases.is_some() {
            if request.effective_schema() != ALIAS_REQUEST_SCHEMA {
                return Err(SearchError::InvalidProfile);
            }
        } else if request.effective_schema() == ALIAS_REQUEST_SCHEMA {
            return Err(SearchError::InvalidProfile);
        } else if self.manifest.include_restrictions.is_some() {
            if request.effective_schema() != INVENTORY_REQUEST_SCHEMA {
                return Err(SearchError::InvalidProfile);
            }
        } else if self.manifest.include_documentation.is_some()
            != (request.effective_schema() == DOCUMENTATION_REQUEST_SCHEMA)
            || request.effective_schema() == INVENTORY_REQUEST_SCHEMA
        {
            return Err(SearchError::InvalidProfile);
        }
        Ok(())
    }
    pub(crate) fn validate_federation_summary(
        &self,
        summary: &serde_json::Value,
        stop: &AtomicBool,
    ) -> SearchResult<ReferenceQueryRequest> {
        check(stop)?;
        let request_value = summary
            .get("request")
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let request = ReferenceQueryRequest::from_json_slice(
            &wow_core::canonical_json_bytes(request_value)
                .map_err(|_| SearchError::OwnerEvidenceInvalid)?,
            stop,
        )?;
        self.validate_query_component(&request)?;
        let profile = capture_query_profile(self.manifest.native_documents, &request);
        let query = identity(
            "reference-search-query:",
            &(profile, self.physical.search_shard_id(), &request),
        )?;
        let universe = identity(
            "reference-search-universe:",
            &(
                &self.manifest.binding,
                &self.manifest.reference_generation,
                &self.manifest.reference_view_digest,
            ),
        )?;
        let mut owner = serde_json::json!({"schema":self.manifest.schema,"profile_digest":self.manifest.profile_digest,"binding":self.manifest.binding,
            "reference_generation":self.manifest.reference_generation,"reference_view_digest":self.manifest.reference_view_digest,
            "native_projection_profile":self.manifest.native_projection_profile,"source_origins":self.manifest.source_origins,
            "partitions":self.manifest.partitions,"conflicts":self.manifest.conflicts,"issues":self.manifest.issues,"owner_negative_authority":self.manifest.owner_negative_authority,
            "projected_document_count":self.manifest.documents.len(),"privacy_scope":self.manifest.privacy_scope,"license_state":self.manifest.license_state});
        if let Some(include) = self.manifest.include_documentation {
            owner["include_documentation"] = serde_json::json!(include);
            owner["documentation_coverage"] =
                serde_json::to_value(documentation::coverage(&self.manifest))
                    .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        }
        if let Some(include) = self.manifest.include_restrictions {
            owner["include_restrictions"] = serde_json::json!(include);
        }
        if let Some(summary) =
            compatibility_aliases::summary(self.manifest.compatibility_aliases.as_ref())
        {
            owner["compatibility_aliases"] =
                serde_json::to_value(summary).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        }
        if let Some(selection) = self.manifest.native_documents {
            owner["native_documents"] =
                serde_json::to_value(selection).map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        }
        if let Some(profile) = &self.manifest.available_type_resolution_profile {
            owner["available_type_resolution_profile"] = serde_json::json!(profile);
        }
        if summary.get("schema").and_then(serde_json::Value::as_str)
            != Some("wow-search/native-reference-result-set/1")
            || summary
                .get("query_profile")
                .and_then(serde_json::Value::as_str)
                != Some(profile)
            || summary
                .get("normalized_query_id")
                .and_then(serde_json::Value::as_str)
                != Some(query.as_ref())
            || summary
                .get("universe_id")
                .and_then(serde_json::Value::as_str)
                != Some(universe.as_ref())
            || summary.get("shard_id").and_then(serde_json::Value::as_str)
                != Some(self.physical.search_shard_id())
            || summary.get("owner") != Some(&owner)
            || request_value
                != &serde_json::to_value(&request).map_err(|_| SearchError::OwnerEvidenceInvalid)?
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }

        Ok(request)
    }
    /// Validate recorded Search fields, never reconstruct native signatures.
    pub(crate) fn validate_federation_candidate(
        &self,
        query_id: &str,
        universe_id: &str,
        candidate: &serde_json::Value,
        request: &ReferenceQueryRequest,
        stop: &AtomicBool,
    ) -> SearchResult<()> {
        check(stop)?;
        let key = candidate
            .get("stable_tie_key")
            .and_then(serde_json::Value::as_str)
            .ok_or(SearchError::OwnerEvidenceInvalid)?;
        let index = self
            .manifest
            .documents
            .binary_search_by(|document| document.entity_key.as_str().cmp(key))
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let document = &self.manifest.documents[index];
        if request
            .filters
            .as_ref()
            .is_some_and(|filters| !filters.accepts(document))
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        if candidate.get("document")
            != Some(&serde_json::to_value(document).map_err(|_| SearchError::OwnerEvidenceInvalid)?)
            || candidate
                .get("candidate_id")
                .and_then(serde_json::Value::as_str)
                != Some(
                    identity(
                        "reference-search-candidate:",
                        &(universe_id, query_id, &document.entity_key),
                    )?
                    .as_ref(),
                )
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        if let Some(signals) = candidate
            .get("signals")
            .and_then(serde_json::Value::as_array)
        {
            for signal in signals {
                check(stop)?;
                let Some(evidence) = signal.get("explicit_alias") else {
                    continue;
                };
                let record_key = evidence
                    .get("record_key")
                    .and_then(serde_json::Value::as_str)
                    .ok_or(SearchError::OwnerEvidenceInvalid)?;
                let row = self
                    .manifest
                    .compatibility_aliases
                    .as_ref()
                    .and_then(|data| data.records.iter().find(|row| row.record_key == record_key))
                    .ok_or(SearchError::OwnerEvidenceInvalid)?;
                let value_digest = identity("", &row.value)?;
                if !row.target_matches(document)
                    || evidence
                        != &serde_json::to_value(row.evidence())
                            .map_err(|_| SearchError::OwnerEvidenceInvalid)?
                    || signal.get("lane").and_then(serde_json::Value::as_str)
                        != Some("exact_explicit_alias")
                    || signal
                        .get("match_class")
                        .and_then(serde_json::Value::as_str)
                        != Some("exact_configured_owner_compatibility_alias")
                    || signal
                        .get("authority_band")
                        .and_then(serde_json::Value::as_u64)
                        != Some(3)
                    || signal
                        .pointer("/origin/owner_record_digest")
                        .and_then(serde_json::Value::as_str)
                        != Some(row.record_digest.as_str())
                    || signal
                        .pointer("/origin/source_id")
                        .and_then(serde_json::Value::as_str)
                        != row.source_ids.first().map(String::as_str)
                    || signal
                        .pointer("/origin/original_value_digest")
                        .and_then(serde_json::Value::as_str)
                        != Some(value_digest.as_ref())
                    || signal
                        .pointer("/origin/normalized_value_digest")
                        .and_then(serde_json::Value::as_str)
                        != Some(value_digest.as_ref())
                {
                    return Err(SearchError::OwnerEvidenceInvalid);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn filtered_query_is_closed_explicit_and_retains_original_component_rules() -> SearchResult<()>
    {
        let stop = AtomicBool::new(false);
        let input = json!({"schema":FILTERED_REQUEST_SCHEMA,"base_component":"compatibility",
            "filters":{"entity_kinds":["event","api"]},"selectors":{"explicit_alias":"Legacy"},
            "enabled_lanes":["exact_explicit_alias"],"required_lanes":["exact_explicit_alias"],
            "max_scanned_documents":32,"max_candidates":8,"max_signals":16,"max_result_bytes":65536});
        let encode = |value: &serde_json::Value| {
            serde_json::to_vec(value).map_err(|_| SearchError::InvalidProfile)
        };
        let request = ReferenceQueryRequest::from_json_slice(&encode(&input)?, &stop)?;
        assert_eq!(request.effective_schema(), ALIAS_REQUEST_SCHEMA);
        assert_eq!(
            capture_query_profile(None, &request),
            FILTERED_QUERY_PROFILE
        );
        let normalized = serde_json::to_value(&request).map_err(|_| SearchError::InvalidProfile)?;
        assert_eq!(
            normalized["filters"]["entity_kinds"],
            json!(["api", "event"])
        );
        for field in ["base_component", "filters"] {
            let mut bad = input.clone();
            bad[field] = serde_json::Value::Null;
            assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        }
        let mut bad = input.clone();
        bad["schema"] = json!(ALIAS_REQUEST_SCHEMA);
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        let mut bad = input.clone();
        bad["filters"]["entity_kinds"] = json!(["api", "api"]);
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        let mut bad = input.clone();
        bad["filters"]["entity_kinds"] = json!(["event"]);
        bad["filters"]["callable_namespace"] = json!({"kind":"exact","value":"C_Example"});
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        let mut bad = input;
        bad["base_component"] = json!("native");
        bad["selectors"] = json!({"shape":{"returns":[{"position":0,"original_type":"table"}]}});
        bad["enabled_lanes"] = json!(["structured_shape"]);
        bad["required_lanes"] = json!(["structured_shape"]);
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        Ok(())
    }
    #[test]
    fn field_shape_schema_is_explicit_and_preserves_kind_boundaries() -> SearchResult<()> {
        let stop = AtomicBool::new(false);
        let input = json!({"schema":FIELD_SHAPE_REQUEST_SCHEMA,"selectors":{"shape":{
            "returns":[{"position":0,"original_type":"table","inner_type":{"kind":"equals","original_type":"number"},"nilability":"allows_nil"}]}},
            "enabled_lanes":["structured_shape"],"required_lanes":["structured_shape"],
            "max_scanned_documents":4,"max_candidates":4,"max_signals":8,"max_result_bytes":65536});
        let encode = |value: &serde_json::Value| {
            serde_json::to_vec(value).map_err(|_| SearchError::InvalidProfile)
        };
        let request = ReferenceQueryRequest::from_json_slice(&encode(&input)?, &stop)?;
        assert_eq!(
            capture_query_profile(None, &request),
            FIELD_SHAPE_QUERY_PROFILE
        );
        let canonical =
            wow_core::canonical_json_bytes(&request).map_err(|_| SearchError::InvalidProfile)?;
        assert_eq!(
            canonical,
            wow_core::canonical_json_bytes(&input).map_err(|_| SearchError::InvalidProfile)?
        );
        let mut bad = input.clone();
        bad["schema"] = json!(REQUEST_SCHEMA);
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        bad["selectors"]["shape"]["returns"] = json!([]);
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        bad["selectors"]["shape"] =
            json!({"arguments":[{"position":0,"original_type":"table","inner_type":null}]});
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        bad = input.clone();
        bad["selectors"]["shape"]["event_payload_count"] = json!(0);
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        bad = input;
        bad["selectors"]["shape"]["declaration"] = json!({"kind":"callback"});
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        Ok(())
    }
    #[test]
    fn closed_reference_request_preserves_missing_fields_and_owner_key_grammar() -> SearchResult<()>
    {
        let stop = AtomicBool::new(false);
        let input = json!({"schema":REQUEST_SCHEMA,"selectors":{"entity_key":"function:Read"},
            "enabled_lanes":["exact_identity"],"required_lanes":["exact_identity"],
            "max_scanned_documents":4,"max_candidates":4,"max_signals":8,"max_result_bytes":65536});
        let encode = |value: &serde_json::Value| {
            serde_json::to_vec(value).map_err(|_| SearchError::InvalidProfile)
        };
        let request = ReferenceQueryRequest::from_json_slice(&encode(&input)?, &stop)?;
        let canonical = wow_core::canonical_json_bytes(&request)
            .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
        let decoded: serde_json::Value =
            serde_json::from_slice(&canonical).map_err(|_| SearchError::InvalidProfile)?;
        assert_eq!(decoded["selectors"], input["selectors"]);
        let mut bad = input.clone();
        bad["selectors"]["entity_key"] = json!("invalid owner key");
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        let mut bad = input.clone();
        bad["enabled_lanes"] = json!(["exact_identity", "exact_identity"]);
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        let mut bad = input;
        bad["selectors"]["sql"] = json!("untrusted");
        assert!(ReferenceQueryRequest::from_json_slice(&encode(&bad)?, &stop).is_err());
        assert!(matches!(
            ReferenceQueryRequest::from_json_slice(&canonical, &AtomicBool::new(true)),
            Err(SearchError::Cancelled)
        ));
        Ok(())
    }
}

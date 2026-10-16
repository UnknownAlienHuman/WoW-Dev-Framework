//! Closed ProjectEntity retrieval and owner-executed seeded graph expansion.
use super::*;
mod pagination;
pub use pagination::{EntityPage, EntityPageCursor, EntityPagePolicy};
use std::collections::BTreeMap;
use wow_graph::{
    GraphDirection, GraphExplainLimits, GraphExplainQuery, GraphExplainSubject, GraphQueryState,
    GraphRelationKind, GraphSubgraphConfidence, GraphSubgraphLimits, GraphSubgraphQuery,
};
const REQUEST_SCHEMA: &str = "wow-search/project-entity-query/1";
const QUERY_PROFILE: &str = "wow-search/project-entity-identity-key-text-seeded-graph/1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExpansionProfile {
    Structure,
    Code,
    State,
    Callbacks,
}
impl ExpansionProfile {
    fn relations(self) -> Vec<GraphRelationKind> {
        use GraphRelationKind::*;
        match self {
            Self::Structure => vec![
                Contains,
                Defines,
                Owns,
                Loads,
                LoadsBefore,
                DependsOn,
                OptionalDependsOn,
            ],
            Self::Code => vec![Calls, UsesApi],
            Self::State => vec![ReadsState, WritesState],
            Self::Callbacks => vec![
                RegistersNativeEvent,
                HandlesNativeEvent,
                BridgesNativeEvent,
                EmitsCustomSignal,
                HandlesCustomSignal,
                RegistersCvarCallback,
                SetsScript,
                HooksScript,
                SecureHooksFunction,
            ],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "class", deny_unknown_fields, rename_all = "snake_case")]
enum Query {
    ExactEntity {
        entity_id: GraphNodeId,
    },
    ExactOwnerKey {
        owner_key: String,
    },
    LiteralOwnerKey {
        text: String,
    },
    Related {
        seeds: Vec<GraphNodeId>,
        expansion_profile: ExpansionProfile,
        direction: GraphDirection,
        confidence: GraphSubgraphConfidence,
        limits: GraphSubgraphLimits,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityQueryRequest {
    schema: String,
    query: Query,
    max_scanned_documents: NonZeroU32,
    max_candidates: NonZeroU32,
    explanation_limits: GraphExplainLimits,
    max_total_support_scans: NonZeroU64,
    max_result_bytes: NonZeroU64,
}
impl EntityQueryRequest {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> SearchResult<Self> {
        check(stop)?;
        if bytes.len() > 65536 {
            return Err(SearchError::InvalidProfile);
        }
        let mut request: Self =
            serde_json::from_slice(bytes).map_err(|_| SearchError::InvalidProfile)?;
        if request.schema != REQUEST_SCHEMA {
            return Err(SearchError::InvalidProfile);
        }
        request.explanation_limits.validate().map_err(graph_error)?;
        match &mut request.query {
            Query::ExactOwnerKey { owner_key } => valid_text(owner_key)?,
            Query::LiteralOwnerKey { text } => valid_text(text)?,
            Query::Related { seeds, limits, .. } => {
                seeds.sort();
                limits.validate().map_err(graph_error)?;
                if seeds.is_empty() || seeds.len() > 8 || seeds.windows(2).any(|p| p[0] == p[1]) {
                    return Err(SearchError::InvalidProfile);
                }
            }
            Query::ExactEntity { entity_id } => {
                GraphNodeId::new(entity_id.as_str()).map_err(graph_error)?;
            }
        }
        Ok(request)
    }
}
fn valid_text(value: &str) -> SearchResult<()> {
    if value.trim().is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
        Err(SearchError::InvalidProfile)
    } else {
        Ok(())
    }
}
pub struct EntityQueryResult {
    bytes: Box<[u8]>,
    status: &'static str,
    profile_digest: Box<str>,
}
impl EntityQueryResult {
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
    document: &'a EntitySearchDocument,
    match_class: &'static str,
    authority_band: u32,
    rank_tuple: [u32; 3],
    stable_tie_key: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    graph_path: Option<super::discovery::DiscoveryPath<'a>>,
}
#[derive(Serialize)]
struct Explanation {
    subject: GraphExplainSubject,
    owner_result: serde_json::Value,
}

impl EntitySearchShard {
    pub fn retrieve(
        &self,
        read: &ReadSnapshot,
        pair: &AcquiredProjectPair,
        request: &EntityQueryRequest,
        stop: &AtomicBool,
    ) -> SearchResult<EntityQueryResult> {
        check(stop)?;
        let owner = owner_manifest_evidence(read, pair, self.bounds, stop)?;
        let graph = pair.graph().snapshot();
        if owner.source != self.manifest.owner.source
            || graph.limits() != self.manifest.graph_limits
            || digest("", pair.graph().registry())? != self.manifest.graph_registry_digest
            || !graph
                .nodes()
                .iter()
                .eq(self.manifest.documents.iter().map(|d| &d.entity))
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let query_id = digest(
            "entity-search-query:",
            &(QUERY_PROFILE, self.logical_id(), request),
        )?;
        let universe = digest(
            "entity-search-universe:",
            &(&owner.source, &self.manifest.graph_registry_digest),
        )?;
        let mut candidates = Vec::<Candidate<'_>>::new();
        let mut truncated = false;
        let mut scanned = 0u32;
        let mut graph_result = None;
        let mut subjects = BTreeMap::new();
        match &request.query {
            Query::ExactEntity { entity_id } => {
                if let Ok(index) = self
                    .manifest
                    .documents
                    .binary_search_by(|d| d.entity_id().cmp(entity_id))
                {
                    let doc = &self.manifest.documents[index];
                    candidates.push(candidate(
                        &query_id,
                        doc,
                        "exact_entity_identity",
                        0,
                        [0, 0, 0],
                        None,
                    )?);
                }
            }
            Query::ExactOwnerKey { owner_key } => {
                for doc in &self.manifest.documents {
                    check(stop)?;
                    if scanned >= request.max_scanned_documents.get() {
                        truncated = true;
                        break;
                    }
                    scanned += 1;
                    if doc.owner_metadata_key() == owner_key {
                        if candidates.len() >= request.max_candidates.get() as usize {
                            truncated = true;
                            break;
                        }
                        candidates.push(candidate(
                            &query_id,
                            doc,
                            "exact_recorded_owner_key",
                            1,
                            [1, 0, 0],
                            None,
                        )?);
                    }
                }
            }
            Query::LiteralOwnerKey { text } => {
                let mut found = self.physical.reader().literal_candidates(text)?;
                found.sort_by(|a, b| a.document_id().cmp(b.document_id()));
                for (rank, hit) in found.iter().enumerate() {
                    check(stop)?;
                    if candidates.len() >= request.max_candidates.get() as usize {
                        truncated = true;
                        break;
                    }
                    let id = GraphNodeId::new(hit.document_id()).map_err(graph_error)?;
                    let index = self
                        .manifest
                        .documents
                        .binary_search_by(|d| d.entity_id().cmp(&id))
                        .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
                    let doc = &self.manifest.documents[index];
                    if hit.owner_metadata_label() != doc.owner_metadata_key() {
                        return Err(SearchError::OwnerEvidenceInvalid);
                    }
                    candidates.push(candidate(
                        &query_id,
                        doc,
                        "literal_opaque_owner_key_candidate",
                        7,
                        [7, rank as u32, 0],
                        None,
                    )?);
                }
            }
            Query::Related {
                seeds,
                expansion_profile,
                direction,
                confidence,
                limits,
            } => {
                let expansion = GraphSubgraphQuery::new(
                    graph.snapshot_id().clone(),
                    seeds.clone(),
                    *direction,
                    expansion_profile.relations(),
                    *limits,
                )
                .map_err(graph_error)?
                .with_confidence(*confidence);
                graph_result = Some(expansion.execute(graph, stop).map_err(graph_error)?);
            }
        }
        if let Some(result) = &graph_result {
            let Query::Related { seeds, .. } = &request.query else {
                return Err(SearchError::InvalidProfile);
            };
            truncated |= result.state() == GraphQueryState::Truncated;
            let mut paths = super::discovery::discovery_paths(result, seeds, stop)?;
            paths.sort_by(|a, b| {
                a.steps
                    .len()
                    .cmp(&b.steps.len())
                    .then_with(|| a.weakest_confidence.cmp(&b.weakest_confidence))
                    .then_with(|| a.target.cmp(b.target))
            });
            for path in paths {
                check(stop)?;
                if candidates.len() >= request.max_candidates.get() as usize {
                    truncated = true;
                    break;
                }
                let index = self
                    .manifest
                    .documents
                    .binary_search_by(|d| d.entity_id().cmp(path.target))
                    .map_err(|_| SearchError::OwnerEvidenceInvalid)?;
                for step in &path.steps {
                    subjects.insert(
                        (1u8, step.original_edge.edge_id().as_str().to_owned()),
                        GraphExplainSubject::Relation(step.original_edge.edge_id().clone()),
                    );
                }
                let tuple = [9, path.steps.len() as u32, path.weakest_confidence as u32];
                candidates.push(candidate(
                    &query_id,
                    &self.manifest.documents[index],
                    "exact_seed_graph_discovery_candidate",
                    9,
                    tuple,
                    Some(path),
                )?);
            }
        }
        candidates.sort_by(|a, b| {
            a.rank_tuple
                .cmp(&b.rank_tuple)
                .then_with(|| a.stable_tie_key.cmp(b.stable_tie_key))
        });
        for c in &candidates {
            subjects.insert(
                (0u8, c.document.entity_id().as_str().to_owned()),
                GraphExplainSubject::Entity(c.document.entity_id().clone()),
            );
        }
        let mut total_scans = 0u64;
        let mut explanations = Vec::new();
        for subject in subjects.into_values() {
            check(stop)?;
            let scan = pair.graph().partitions().iter().try_fold(0u64, |n, p| {
                n.checked_add(match subject {
                    GraphExplainSubject::Entity(_) => p.report().accepted_entities().len() as u64,
                    GraphExplainSubject::Relation(_) => {
                        p.report().accepted_relations().len() as u64
                    }
                })
                .ok_or(SearchError::FileSetBudgetExceeded)
            })?;
            if total_scans
                .checked_add(scan)
                .is_none_or(|n| n > request.max_total_support_scans.get())
            {
                return Err(SearchError::FileSetBudgetExceeded);
            }
            let remaining = request
                .max_total_support_scans
                .get()
                .checked_sub(total_scans)
                .ok_or(SearchError::FileSetBudgetExceeded)?;
            if remaining == 0 {
                return Err(SearchError::FileSetBudgetExceeded);
            }
            let limits = GraphExplainLimits {
                max_scanned_assertions: request
                    .explanation_limits
                    .max_scanned_assertions
                    .min(u32::try_from(remaining).unwrap_or(u32::MAX)),
                ..request.explanation_limits
            };
            let query =
                GraphExplainQuery::new(graph.snapshot_id().clone(), subject.clone(), limits)
                    .map_err(graph_error)?;
            let explanation = query.execute(pair.graph(), stop).map_err(graph_error)?;
            total_scans = total_scans
                .checked_add(u64::from(explanation.scanned_assertions()))
                .ok_or(SearchError::FileSetBudgetExceeded)?;
            if total_scans > request.max_total_support_scans.get() {
                return Err(SearchError::FileSetBudgetExceeded);
            }
            truncated |= !explanation.truncations().is_empty();
            explanations.push(Explanation {
                subject,
                owner_result: support_metadata(&explanation)?,
            });
        }
        let exact = matches!(
            request.query,
            Query::ExactEntity { .. } | Query::ExactOwnerKey { .. }
        );
        let status = if truncated {
            "truncated"
        } else if !candidates.is_empty() {
            if exact {
                "exact_found"
            } else {
                "candidate_only"
            }
        } else if exact {
            "exact_not_found_partial"
        } else {
            "no_candidates_under_executed_lanes"
        };
        #[derive(Serialize)]
        struct Payload<'a> {
            schema: &'static str,
            query_profile: &'static str,
            result_query_id: &'a str,
            universe_id: &'a str,
            shard_id: &'a str,
            owner: &'a OwnerManifestEvidence,
            graph_registry_digest: &'a str,
            request: &'a EntityQueryRequest,
            status: &'static str,
            absence_authoritative: bool,
            scanned_documents: u32,
            scanned_support_assertions: u64,
            reason_path_coverage: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            graph_owner_result: Option<&'a wow_graph::GraphSubgraphResult>,
            candidates: &'a [Candidate<'a>],
            explanations: &'a [Explanation],
        }
        let payload = Payload {
            schema: "wow-search/project-entity-result-set/1",
            query_profile: QUERY_PROFILE,
            result_query_id: &query_id,
            universe_id: &universe,
            shard_id: self.logical_id(),
            owner: &owner,
            graph_registry_digest: &self.manifest.graph_registry_digest,
            request,
            status,
            absence_authoritative: false,
            scanned_documents: scanned,
            scanned_support_assertions: total_scans,
            reason_path_coverage: "one_canonical_owner_discovery_witness_per_target_not_all_paths",
            graph_owner_result: graph_result.as_ref(),
            candidates: &candidates,
            explanations: &explanations,
        };
        let result_set_id = digest("project-entity-result-set:", &payload)?;
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
        Ok(EntityQueryResult {
            bytes: bytes.into_boxed_slice(),
            status,
            profile_digest: self.profile_digest().into(),
        })
    }
}
fn support_metadata(owner: &wow_graph::GraphExplanation<'_>) -> SearchResult<serde_json::Value> {
    use wow_graph::GraphAssertionSupport;
    let project = |support: &GraphAssertionSupport<'_>| -> serde_json::Value {
        match support {
            GraphAssertionSupport::FoundationEntity { node } => {
                serde_json::json!({"origin":"foundation_entity","node":node})
            }
            GraphAssertionSupport::FoundationRelation { edge } => {
                serde_json::json!({"origin":"foundation_relation","edge":edge})
            }
            GraphAssertionSupport::ProducerEntity {
                producer,
                proposal,
                accepted,
                ..
            } => serde_json::json!({
            "origin":"producer_entity","producer":producer,"proposal_id":proposal.proposal_id(),
            "entity_kind_id":proposal.entity_kind_id(),"confidence":proposal.confidence(),
            "source_handle_ids":proposal.source_handle_ids(),"evidence_ids":proposal.evidence_ids(),
            "accepted":accepted,"semantic_key_projection":"not_delivered_by_metadata_profile"}),
            GraphAssertionSupport::ProducerRelation {
                producer,
                proposal,
                accepted,
                ..
            } => serde_json::json!({
            "origin":"producer_relation","producer":producer,"proposal":proposal,"accepted":accepted}),
        }
    };
    // Preserve producer addresses, input-generation accepted records and full
    // support/source/evidence/coverage/conflict/derivation closure. Arbitrary
    // semantic-key properties remain in the original graph owner, not Search.
    Ok(
        serde_json::json!({"schema":"wow-search/graph-support-metadata/1",
        "query":owner.query(),"query_digest":owner.query_digest(),"record":owner.record(),"registry":owner.registry(),
        "supports":owner.supports().iter().map(project).collect::<Vec<_>>(),
        "assertion_supports":owner.assertion_supports().iter().map(project).collect::<Vec<_>>(),
        "derivations":owner.derivations(),"conflicts":owner.conflicts(),"derivation_complete":owner.derivation_complete(),
        "coverage":owner.coverage(),"scanned_assertions":owner.scanned_assertions(),"total_supports":owner.total_supports(),
        "support_complete":owner.support_complete(),"truncations":owner.truncations(),"boundaries":owner.boundaries(),
        "absence_authoritative":owner.absence_authoritative(),
        "owner_proposal_properties":"not_delivered_use_exact_owner_handle_with_separate_detail_policy"}),
    )
}
fn candidate<'a>(
    query_id: &str,
    document: &'a EntitySearchDocument,
    class: &'static str,
    band: u32,
    tuple: [u32; 3],
    path: Option<super::discovery::DiscoveryPath<'a>>,
) -> SearchResult<Candidate<'a>> {
    Ok(Candidate {
        candidate_id: digest(
            "project-entity-candidate:",
            &(query_id, document.entity_id()),
        )?,
        document,
        match_class: class,
        authority_band: band,
        rank_tuple: tuple,
        stable_tie_key: document.entity_id().as_str(),
        graph_path: path,
    })
}

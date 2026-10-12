//! Consuming combined native metadata pack; original owners remain distinct.
use super::*;
use crate::combined::{CombinedContextInspect, CombinedInspectArtifact};
use crate::platform::encode_retained;
use wow_graph::{GraphGenerationId, GraphNodeId, GraphUniverseId};
use wow_project::ProjectId;

pub const COMBINED_CONTEXT_PACK_PROFILE: &str = "wow-context/combined-native-structural-pack/1";
pub const COMBINED_CONTEXT_JSON_RENDERER_PROFILE: &str = "wow-context/combined-pack-json/1";
pub const COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE: &str = "wow-context/combined-pack-markdown/1";
pub(crate) const COMBINED_PACK_SCHEMA: &str = "wow-context/semantic-pack/combined/1";
pub(crate) const COMBINED_RENDER_SCHEMA: &str = "wow-context/rendered-context-artifact/combined/1";
const COMBINED_NOT_EVALUATED: &[&str] = &[
    "reference_fact_enrichment",
    "control_effect_projection",
    "source_excerpts_and_tokens",
    "cross_universe_relations",
    "deeper_expansion_and_continuation",
    "complete_corpus_and_runtime",
];
const COMBINED_PACK_PROFILE: PackProfile = PackProfile {
    profile_id: COMBINED_CONTEXT_PACK_PROFILE,
    deduplication: "qualified_owner_universe_generation_native_node_or_edge_id",
    layer_order: &["primary_l0", "primary_l1", "platform_native_inspect"],
    ordering: "primary_then_platform_each_native_node_id_then_native_edge_id",
    selection: "mandatory_exact_inspect_roots_and_selected_original_native_records",
    ..PACK_PROFILE
};
const COMBINED_JSON_PROFILE: RendererProfile = RendererProfile {
    profile_id: COMBINED_CONTEXT_JSON_RENDERER_PROFILE,
    semantic_schema: COMBINED_PACK_SCHEMA,
    ..JSON_PROFILE
};
const COMBINED_MARKDOWN_PROFILE: RendererProfile = RendererProfile {
    profile_id: COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE,
    semantic_schema: COMBINED_PACK_SCHEMA,
    templates: &[
        "pack_header",
        "pack_profile",
        "combined_native_origins",
        "pack_request",
        "boundary_notice",
        "qualified_native_entity_item",
        "qualified_native_relation_item",
        "selection_trace",
        "coverage_and_omissions",
        "pack_budget",
    ],
    ..MARKDOWN_PROFILE
};
pub const COMBINED_CONTEXT_PROFILES_BYTES: &[u8] = br#"{"json":{"encoding":"utf-8","layout":"lossless_canonical_pack_json","line_endings":"no_trailing_lf","media_type":"application/json","profile_id":"wow-context/combined-pack-json/1","rendering_loss":"none","semantic_schema":"wow-context/semantic-pack/combined/1","source_boundary":"canonical_json_strings","templates":["canonical_pack_json"],"trace":"exact_utf8_byte_ranges"},"markdown":{"encoding":"utf-8","layout":"fixed_typed_metadata_records","line_endings":"lf","media_type":"text/markdown","profile_id":"wow-context/combined-pack-markdown/1","rendering_loss":"none","semantic_schema":"wow-context/semantic-pack/combined/1","source_boundary":"markdown_safe_quoted_canonical_json_metadata","templates":["pack_header","pack_profile","combined_native_origins","pack_request","boundary_notice","qualified_native_entity_item","qualified_native_relation_item","selection_trace","coverage_and_omissions","pack_budget"],"trace":"exact_utf8_byte_ranges"},"pack":{"authority":"preserve_original_native_confidence_coverage_conflicts","boundary":"untrusted_native_metadata_is_data","budgets":"continued_cumulative_work_and_encoded_reservations","continuation":"not_evaluated","deduplication":"qualified_owner_universe_generation_native_node_or_edge_id","item_kinds":["entity","relation"],"layer_order":["primary_l0","primary_l1","platform_native_inspect"],"license":"source_inclusion_not_permitted","ordering":"primary_then_platform_each_native_node_id_then_native_edge_id","privacy":"local_metadata_only","profile_id":"wow-context/combined-native-structural-pack/1","selection":"mandatory_exact_inspect_roots_and_selected_original_native_records","source_bodies":"excluded","tokens":"unavailable"}}"#;

fn profiles(limit: u64, budget: &mut Budget, stop: &AtomicBool) -> ContextResult<()> {
    #[derive(Serialize)]
    struct Profiles {
        json: &'static RendererProfile,
        markdown: &'static RendererProfile,
        pack: &'static PackProfile,
    }
    let catalog = Profiles {
        json: &COMBINED_JSON_PROFILE,
        markdown: &COMBINED_MARKDOWN_PROFILE,
        pack: &COMBINED_PACK_PROFILE,
    };
    budget.work(COMBINED_CONTEXT_PROFILES_BYTES.len(), stop)?;
    budget.retain(&catalog, stop)?;
    if canonical(&catalog, limit, stop)?.as_slice() != COMBINED_CONTEXT_PROFILES_BYTES {
        return Err(encoding());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NativeOwner {
    Primary,
    Platform,
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum NativeScope<'a> {
    Project { project_id: &'a ProjectId },
    Entity { root_id: &'a GraphNodeId },
}
#[derive(Serialize)]
pub(crate) struct QualifiedItemCore<'a> {
    schema: &'static str,
    owner: NativeOwner,
    universe: &'a GraphUniverseId,
    generation: &'a GraphGenerationId,
    scope: NativeScope<'a>,
    origin_map_id: &'a str,
    pub(crate) payload: ContextItemPayload<'a>,
    payload_digest: Box<str>,
    inclusion_reason: &'static str,
    privacy: &'static str,
    license: &'static str,
    source_boundary: &'static str,
}
#[derive(Serialize)]
pub(crate) struct QualifiedItem<'a> {
    context_item_id: Box<str>,
    pub(crate) core: QualifiedItemCore<'a>,
}
impl QualifiedItem<'_> {
    pub(crate) fn context_item_id(&self) -> &str {
        &self.context_item_id
    }
}
#[derive(Serialize)]
pub(crate) struct QualifiedSelection<'a> {
    item_id: Box<str>,
    owner: NativeOwner,
    origin_map_id: &'a str,
    reason: &'static str,
    required: bool,
}
#[derive(Serialize)]
struct CombinedRequestCore {
    schema: &'static str,
    profile_id: &'static str,
    origin_inspect_id: Box<str>,
    combined_map_id: Box<str>,
    primary_map_id: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_l0_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_l1_id: Option<Box<str>>,
    platform_inspect_id: Box<str>,
    platform_map_id: Box<str>,
}
#[derive(Serialize)]
pub(crate) struct CombinedRequest {
    context_request_id: Box<str>,
    core: CombinedRequestCore,
}
#[derive(Serialize)]
struct OmissionOrigin {
    owner: NativeOwner,
    artifact_id: Box<str>,
}
#[derive(Serialize)]
pub(crate) struct CombinedOmissions {
    upstream: Vec<OmissionOrigin>,
    interpretation: &'static str,
    not_evaluated: &'static [&'static str],
    continuation: &'static str,
}
#[derive(Serialize)]
pub(crate) struct CombinedPackCore<'m, 'p, 'f> {
    pub(crate) schema: &'static str,
    pub(crate) profile: &'static PackProfile,
    pub(crate) request: CombinedRequest,
    pub(crate) boundary_notice: &'static str,
    pub(crate) origins: CombinedInspectArtifact<'m, 'p, 'f>,
    pub(crate) items: Vec<QualifiedItem<'m>>,
    pub(crate) selection_trace: Vec<QualifiedSelection<'m>>,
    pub(crate) omissions: CombinedOmissions,
    pub(crate) budget: PackBudget,
    pub(crate) state: ProjectMapState,
    pub(crate) absence_authoritative: bool,
}
#[derive(Serialize)]
struct CombinedPackArtifact<'m, 'p, 'f> {
    semantic_pack_id: Box<str>,
    core: CombinedPackCore<'m, 'p, 'f>,
}
/// No decoding creates this authority or its continuing borrowed ledger.
#[derive(Serialize)]
pub struct CombinedContextSemanticPack<'m, 'p, 'f> {
    #[serde(flatten)]
    artifact: CombinedPackArtifact<'m, 'p, 'f>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    origin_canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    budget: &'m mut Budget,
}
impl<'m, 'p, 'f> CombinedContextSemanticPack<'m, 'p, 'f> {
    pub fn semantic_pack_id(&self) -> &str {
        &self.artifact.semantic_pack_id
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    pub const fn state(&self) -> ProjectMapState {
        self.artifact.core.state
    }
    pub fn remaining_metadata_bytes(&self) -> u64 {
        self.render_limit()
            .saturating_sub(self.budget.retained_bytes)
    }
    pub fn operation_budget_usage(&self) -> ContextOperationBudgetUsage {
        ContextOperationBudgetUsage {
            reservation_basis: "before_service_delivery",
            work_reserved: self.budget.work,
            bytes_reserved: self.budget.retained_bytes,
        }
    }
    pub(crate) fn render_limit(&self) -> u64 {
        self.artifact.core.origins.limits().max_metadata_bytes
    }
    pub(crate) fn render_parts(
        &mut self,
    ) -> (&str, &CombinedPackCore<'m, 'p, 'f>, &[u8], &mut Budget) {
        (
            &self.artifact.semantic_pack_id,
            &self.artifact.core,
            &self.canonical_bytes,
            &mut *self.budget,
        )
    }
}
pub(crate) fn combined_renderer_profiles(
    core: &CombinedPackCore<'_, '_, '_>,
) -> ContextResult<(&'static RendererProfile, &'static RendererProfile)> {
    if core.schema != COMBINED_PACK_SCHEMA
        || core.profile.profile_id != COMBINED_CONTEXT_PACK_PROFILE
        || core.request.core.profile_id != COMBINED_CONTEXT_PACK_PROFILE
    {
        return Err(encoding());
    }
    Ok((&COMBINED_JSON_PROFILE, &COMBINED_MARKDOWN_PROFILE))
}

struct NativeRecord<'a> {
    owner: NativeOwner,
    scope: NativeScope<'a>,
    map_id: &'a str,
    context: &'a GraphNode,
    payload: ContextItemPayload<'a>,
    reason: &'static str,
}
fn same_context(a: &GraphNode, b: &GraphNode) -> ContextResult<()> {
    if a.universe() != b.universe() || a.generation() != b.generation() {
        return Err(ContextError::new(ContextErrorCode::UniverseMismatch));
    }
    Ok(())
}
fn visit<'m, 'p, 'f>(
    origins: &CombinedInspectArtifact<'m, 'p, 'f>,
    budget: &mut Budget,
    stop: &AtomicBool,
    mut accept: impl FnMut(NativeRecord<'m>, &mut Budget) -> ContextResult<()>,
) -> ContextResult<()> {
    let primary = open_project_map_view(origins.primary_map(), stop)?;
    let native = origins.platform_page().neighborhood();
    let count = primary
        .nodes()
        .len()
        .checked_add(primary.edges().len())
        .and_then(|n| n.checked_add(native.adjacent_nodes().len()))
        .and_then(|n| n.checked_add(native.edges().len()))
        .and_then(|n| n.checked_add(1))
        .ok_or_else(exhausted)?;
    // Each relation resolves both endpoints in one owner population. Charge
    // twice the larger search depth even when the other owner is much smaller.
    let depth = primary
        .nodes()
        .len()
        .max(1)
        .ilog2()
        .max(native.adjacent_nodes().len().max(1).ilog2()) as usize;
    let lookups = depth
        .checked_mul(2)
        .and_then(|value| value.checked_add(4))
        .ok_or_else(exhausted)?;
    budget.work(count.checked_mul(lookups).ok_or_else(exhausted)?, stop)?;
    let primary_scope = match primary.root_node() {
        Some(node) => NativeScope::Entity {
            root_id: node.node_id(),
        },
        None => NativeScope::Project {
            project_id: primary.project_container().project_id(),
        },
    };
    for nodes in [primary.nodes(), native.adjacent_nodes()] {
        for pair in nodes.windows(2) {
            checkpoint(stop)?;
            if pair[0].node_id() >= pair[1].node_id() {
                return Err(ContextError::new(ContextErrorCode::SupportMismatch));
            }
        }
    }
    for edges in [primary.edges(), native.edges()] {
        for pair in edges.windows(2) {
            checkpoint(stop)?;
            if pair[0].edge_id() >= pair[1].edge_id() {
                return Err(ContextError::new(ContextErrorCode::SupportMismatch));
            }
        }
    }
    if let Some(root) = primary.root_node()
        && primary
            .node(root.node_id())
            .is_none_or(|node| !std::ptr::eq(root, node))
    {
        return Err(ContextError::new(ContextErrorCode::MissingRequiredCustody));
    }
    for node in primary.nodes() {
        checkpoint(stop)?;
        let reason = if primary.root_node().is_some_and(|r| std::ptr::eq(r, *node)) {
            "exact_native_root"
        } else {
            "selected_original_primary_native_record"
        };
        accept(
            NativeRecord {
                owner: NativeOwner::Primary,
                scope: primary_scope,
                map_id: origins.primary_map().project_map_id(),
                context: node,
                payload: ContextItemPayload::Entity { node },
                reason,
            },
            budget,
        )?;
    }
    for edge in primary.edges() {
        checkpoint(stop)?;
        let from = primary
            .node(edge.from())
            .ok_or_else(|| ContextError::new(ContextErrorCode::MissingRequiredCustody))?;
        let to = primary
            .node(edge.to())
            .ok_or_else(|| ContextError::new(ContextErrorCode::MissingRequiredCustody))?;
        same_context(from, to)?;
        accept(
            NativeRecord {
                owner: NativeOwner::Primary,
                scope: primary_scope,
                map_id: origins.primary_map().project_map_id(),
                context: from,
                payload: ContextItemPayload::Relation { edge },
                reason: "original_selected_direct_native_assertion",
            },
            budget,
        )?;
    }
    let root = native.root();
    let platform_scope = NativeScope::Entity {
        root_id: root.node_id(),
    };
    let mut root_added = false;
    for node in native.adjacent_nodes() {
        checkpoint(stop)?;
        same_context(root, node)?;
        if node.node_id() == root.node_id() {
            // A genuine self-loop may retain the root among adjacent nodes.
            // It remains one original root, never a second selected entity.
            if root_added || !std::ptr::eq(root, *node) {
                return Err(ContextError::new(ContextErrorCode::SupportMismatch));
            }
            accept(
                NativeRecord {
                    owner: NativeOwner::Platform,
                    scope: platform_scope,
                    map_id: origins.platform_map_id(),
                    context: root,
                    payload: ContextItemPayload::Entity { node: root },
                    reason: "exact_native_root",
                },
                budget,
            )?;
            root_added = true;
            continue;
        }
        if !root_added && root.node_id() < node.node_id() {
            accept(
                NativeRecord {
                    owner: NativeOwner::Platform,
                    scope: platform_scope,
                    map_id: origins.platform_map_id(),
                    context: root,
                    payload: ContextItemPayload::Entity { node: root },
                    reason: "exact_native_root",
                },
                budget,
            )?;
            root_added = true;
        }
        accept(
            NativeRecord {
                owner: NativeOwner::Platform,
                scope: platform_scope,
                map_id: origins.platform_map_id(),
                context: root,
                payload: ContextItemPayload::Entity { node },
                reason: "selected_original_platform_native_neighbor",
            },
            budget,
        )?;
    }
    if !root_added {
        accept(
            NativeRecord {
                owner: NativeOwner::Platform,
                scope: platform_scope,
                map_id: origins.platform_map_id(),
                context: root,
                payload: ContextItemPayload::Entity { node: root },
                reason: "exact_native_root",
            },
            budget,
        )?;
    }
    for edge in native.edges() {
        checkpoint(stop)?;
        for id in [edge.from(), edge.to()] {
            if id != root.node_id()
                && native
                    .adjacent_nodes()
                    .binary_search_by(|n| n.node_id().cmp(id))
                    .is_err()
            {
                return Err(ContextError::new(ContextErrorCode::MissingRequiredCustody));
            }
        }
        accept(
            NativeRecord {
                owner: NativeOwner::Platform,
                scope: platform_scope,
                map_id: origins.platform_map_id(),
                context: root,
                payload: ContextItemPayload::Relation { edge },
                reason: "original_selected_direct_native_assertion",
            },
            budget,
        )?;
    }
    Ok(())
}
fn qualified_item<'a>(
    record: NativeRecord<'a>,
    budget: &mut Budget,
    limit: u64,
    stop: &AtomicBool,
) -> ContextResult<QualifiedItem<'a>> {
    budget.work(1, stop)?;
    budget.retain(&record.payload, stop)?;
    let payload_digest = identity(
        "wow-context/context-item-payload/1",
        "sha256:",
        &record.payload,
        limit,
        stop,
    )?;
    let core = QualifiedItemCore {
        schema: "wow-context/context-item/combined/1",
        owner: record.owner,
        universe: record.context.universe(),
        generation: record.context.generation(),
        scope: record.scope,
        origin_map_id: record.map_id,
        payload: record.payload,
        payload_digest,
        inclusion_reason: record.reason,
        privacy: "local_metadata_only",
        license: "source_inclusion_not_permitted",
        source_boundary: "untrusted_native_metadata",
    };
    let context_item_id = identity(core.schema, "context-item:sha256:", &core, limit, stop)?;
    let item = QualifiedItem {
        context_item_id,
        core,
    };
    budget.retain(&item, stop)?;
    Ok(item)
}
#[derive(Serialize)]
struct RequestRef<'a> {
    schema: &'static str,
    profile_id: &'static str,
    origin_inspect_id: &'a str,
    combined_map_id: &'a str,
    primary_map_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_l0_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_l1_id: Option<&'a str>,
    platform_inspect_id: &'a str,
    platform_map_id: &'a str,
}
fn request_ref<'a>(origins: &'a CombinedInspectArtifact<'_, '_, '_>) -> RequestRef<'a> {
    RequestRef {
        schema: "wow-context/combined-pack-request/1",
        profile_id: COMBINED_CONTEXT_PACK_PROFILE,
        origin_inspect_id: origins.inspect_id(),
        combined_map_id: origins.combined_map_id(),
        primary_map_id: origins.primary_map().project_map_id(),
        primary_l0_id: origins.primary_skeletons().l0_id(),
        primary_l1_id: origins.primary_skeletons().l1_id(),
        platform_inspect_id: origins.platform_inspect_id(),
        platform_map_id: origins.platform_map_id(),
    }
}
pub fn build_combined_context_semantic_pack<'m, 'p, 'f>(
    inspect: CombinedContextInspect<'m, 'p, 'f>,
    stop: &AtomicBool,
) -> ContextResult<CombinedContextSemanticPack<'m, 'p, 'f>> {
    checkpoint(stop)?;
    let (origins, origin_canonical_bytes, budget) = inspect.into_pack_parts();
    let limits = origins.limits();
    profiles(limits.max_metadata_bytes, budget, stop)?;
    origins.validate_for_pack(&origin_canonical_bytes, budget, stop)?;
    let request = request_ref(&origins);
    budget.retain(&request, stop)?;
    let context_request_id = identity(
        request.schema,
        "context-pack-request:sha256:",
        &request,
        limits.max_metadata_bytes,
        stop,
    )?;
    let request = CombinedRequest {
        context_request_id,
        core: CombinedRequestCore {
            schema: request.schema,
            profile_id: request.profile_id,
            origin_inspect_id: request.origin_inspect_id.into(),
            combined_map_id: request.combined_map_id.into(),
            primary_map_id: request.primary_map_id.into(),
            primary_l0_id: request.primary_l0_id.map(Into::into),
            primary_l1_id: request.primary_l1_id.map(Into::into),
            platform_inspect_id: request.platform_inspect_id.into(),
            platform_map_id: request.platform_map_id.into(),
        },
    };
    let mut items = Vec::new();
    visit(&origins, budget, stop, |record, budget| {
        items.push(qualified_item(
            record,
            budget,
            limits.max_metadata_bytes,
            stop,
        )?);
        Ok(())
    })?;
    let mut selection_trace = Vec::new();
    budget.work(items.len(), stop)?;
    #[derive(Serialize)]
    struct TraceRef<'a> {
        item_id: &'a str,
        owner: NativeOwner,
        origin_map_id: &'a str,
        reason: &'static str,
        required: bool,
    }
    for item in &items {
        let trace = TraceRef {
            item_id: &item.context_item_id,
            owner: item.core.owner,
            origin_map_id: item.core.origin_map_id,
            reason: item.core.inclusion_reason,
            required: true,
        };
        budget.retain(&trace, stop)?;
        selection_trace.push(QualifiedSelection {
            item_id: trace.item_id.into(),
            owner: trace.owner,
            origin_map_id: item.core.origin_map_id,
            reason: trace.reason,
            required: true,
        });
    }
    let mut upstream = Vec::new();
    #[derive(Serialize)]
    struct OmissionRef<'a> {
        owner: NativeOwner,
        artifact_id: &'a str,
    }
    for (owner, id) in [
        (NativeOwner::Primary, origins.primary_map().project_map_id()),
        (NativeOwner::Platform, origins.platform_inspect_id()),
    ] {
        budget.work(1, stop)?;
        budget.retain(
            &OmissionRef {
                owner,
                artifact_id: id,
            },
            stop,
        )?;
        upstream.push(OmissionOrigin {
            owner,
            artifact_id: id.into(),
        });
    }
    let state = origins.state();
    let core = CombinedPackCore {
        schema: COMBINED_PACK_SCHEMA,
        profile: &COMBINED_PACK_PROFILE,
        request,
        boundary_notice: BOUNDARY_NOTICE,
        origins,
        items,
        selection_trace,
        omissions: CombinedOmissions {
            upstream,
            interpretation: "producer_local_omissions_retained_in_original_artifacts",
            not_evaluated: COMBINED_NOT_EVALUATED,
            continuation: "not_evaluated",
        },
        budget: PackBudget {
            max_metadata_bytes: limits.max_metadata_bytes,
            max_context_work: limits.max_context_work,
            reservation_basis: "before_semantic_pack_finalization",
            work_reserved: budget.work,
            bytes_reserved: budget.retained_bytes,
            tokens: "unavailable",
        },
        state,
        absence_authoritative: false,
    };
    let semantic_pack_id = identity(
        core.schema,
        "context-pack:sha256:",
        &core,
        limits.max_metadata_bytes,
        stop,
    )?;
    let artifact = CombinedPackArtifact {
        semantic_pack_id,
        core,
    };
    let canonical_bytes = encode_retained(&artifact, budget, limits, stop)?;
    Ok(CombinedContextSemanticPack {
        artifact,
        canonical_bytes,
        origin_canonical_bytes,
        budget,
    })
}

pub fn validate_combined_context_semantic_pack(
    pack: &mut CombinedContextSemanticPack<'_, '_, '_>,
    stop: &AtomicBool,
) -> ContextResult<()> {
    checkpoint(stop)?;
    let core = &pack.artifact.core;
    let budget = &mut *pack.budget;
    let limits = core.origins.limits();
    budget.work(1, stop)?;
    profiles(limits.max_metadata_bytes, budget, stop)?;
    combined_renderer_profiles(core)?;
    core.origins
        .validate_for_pack(&pack.origin_canonical_bytes, budget, stop)?;
    if core.absence_authoritative
        || core.state != core.origins.state()
        || core.boundary_notice != BOUNDARY_NOTICE
        || core.items.len() != core.selection_trace.len()
        || core.omissions.interpretation
            != "producer_local_omissions_retained_in_original_artifacts"
        || core.omissions.not_evaluated != COMBINED_NOT_EVALUATED
        || core.omissions.continuation != "not_evaluated"
        || core.budget.max_metadata_bytes != limits.max_metadata_bytes
        || core.budget.max_context_work != limits.max_context_work
        || core.budget.reservation_basis != "before_semantic_pack_finalization"
        || core.budget.tokens != "unavailable"
        || core.budget.work_reserved > budget.work
        || core.budget.bytes_reserved > budget.retained_bytes
    {
        return Err(encoding());
    }
    let request = request_ref(&core.origins);
    budget.retain(&request, stop)?;
    budget.retain(&core.request.core, stop)?;
    if canonical(&request, limits.max_metadata_bytes, stop)?
        != canonical(&core.request.core, limits.max_metadata_bytes, stop)?
        || identity(
            request.schema,
            "context-pack-request:sha256:",
            &request,
            limits.max_metadata_bytes,
            stop,
        )? != core.request.context_request_id
    {
        return Err(encoding());
    }
    let mut index = 0usize;
    visit(&core.origins, budget, stop, |record, budget| {
        let item = core.items.get(index).ok_or_else(encoding)?;
        let trace = core.selection_trace.get(index).ok_or_else(encoding)?;
        let same_record = match (&item.core.payload, &record.payload) {
            (ContextItemPayload::Entity { node: a }, ContextItemPayload::Entity { node: b }) => {
                std::ptr::eq(*a, *b)
            }
            (
                ContextItemPayload::Relation { edge: a },
                ContextItemPayload::Relation { edge: b },
            ) => std::ptr::eq(*a, *b),
            _ => false,
        };
        if !same_record
            || item.core.owner != record.owner
            || item.core.scope != record.scope
            || item.core.origin_map_id != record.map_id
            || item.core.universe != record.context.universe()
            || item.core.generation != record.context.generation()
            || item.core.inclusion_reason != record.reason
            || item.core.schema != "wow-context/context-item/combined/1"
            || item.core.privacy != "local_metadata_only"
            || item.core.license != "source_inclusion_not_permitted"
            || item.core.source_boundary != "untrusted_native_metadata"
            || trace.item_id != item.context_item_id
            || trace.owner != item.core.owner
            || trace.origin_map_id != item.core.origin_map_id
            || trace.reason != item.core.inclusion_reason
            || !trace.required
        {
            return Err(ContextError::new(ContextErrorCode::SupportMismatch));
        }
        budget.work(1, stop)?;
        if identity(
            "wow-context/context-item-payload/1",
            "sha256:",
            &item.core.payload,
            limits.max_metadata_bytes,
            stop,
        )? != item.core.payload_digest
            || identity(
                item.core.schema,
                "context-item:sha256:",
                &item.core,
                limits.max_metadata_bytes,
                stop,
            )? != item.context_item_id
        {
            return Err(encoding());
        }
        index = index.checked_add(1).ok_or_else(exhausted)?;
        Ok(())
    })?;
    if index != core.items.len()
        || core.omissions.upstream.len() != 2
        || core.omissions.upstream[0].owner != NativeOwner::Primary
        || core.omissions.upstream[0].artifact_id.as_ref()
            != core.origins.primary_map().project_map_id()
        || core.omissions.upstream[1].owner != NativeOwner::Platform
        || core.omissions.upstream[1].artifact_id.as_ref() != core.origins.platform_inspect_id()
    {
        return Err(encoding());
    }
    let id = identity(
        core.schema,
        "context-pack:sha256:",
        core,
        limits.max_metadata_bytes,
        stop,
    )?;
    budget.retain(&pack.artifact, stop)?;
    if id != pack.artifact.semantic_pack_id
        || canonical(&pack.artifact, limits.max_metadata_bytes, stop)?.as_slice()
            != pack.canonical_bytes.as_ref()
    {
        return Err(encoding());
    }
    checkpoint(stop)
}

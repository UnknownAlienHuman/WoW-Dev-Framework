//! Exact primary semantic-pack acquisition and explicit owner rendering.
//! The shared runner retains native custody through encoding and checked close.
use super::operation::{
    Envelope, FrontierExpansion, current_record_digest, encode, failed_operation,
    with_checked_context_read, with_native_frontier, with_native_origins,
    with_native_origins_with_catalog, with_native_progressive,
};
use super::request::ProgressiveExpansion;
use super::{
    ContextInspectLevel, ContextL0Profile, ContextL1Profile, ContextMapCurrentGuard,
    ContextMapHostInput, ContextMapPublication, ContextMapReferenceGuard, ContextMapResult,
    ContextMapRoot, ContextMapStatus, ContextRenderSelection, PRIMARY_L0_PROFILE_ID,
    PRIMARY_L1_PROFILE_ID, PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
    PRIMARY_STATE_DETAIL_L1_PROFILE_ID, checkpoint, context_error, failure,
};
use crate::{ServiceErrorCode, ServiceResult, live_project::LiveProjectRead};
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use wow_core::{CanonicalResult, ContentDigest};
use wow_store::project::ReadSelector;

pub const CONTEXT_BUILD_REQUEST_SCHEMA: &str = "wow-service/context-build-request/1";
pub const CONTEXT_BUILD_RESULT_SCHEMA: &str = "wow-service/context-build-result/1";
pub const PRIMARY_CONTEXT_PACK_PROFILE_ID: &str = wow_context::PRIMARY_CONTEXT_PACK_PROFILE;
pub const PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID: &str =
    wow_context::PRIMARY_CONTEXT_JSON_RENDERER_PROFILE;
pub const PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID: &str =
    wow_context::PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE;
pub const PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID: &str =
    wow_context::PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE;
pub const PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID: &str =
    wow_context::PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE;
pub const PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID: &str =
    wow_context::PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE;
pub const PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID: &str =
    wow_context::PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE;
pub const PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID: &str =
    wow_context::PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE;
pub const PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID: &str =
    wow_context::PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE;

const JSON_RENDERER_PROFILE_IDS: &[&str] = &[PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID];
const MARKDOWN_RENDERER_PROFILE_IDS: &[&str] = &[PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID];
const BOTH_RENDERER_PROFILE_IDS: &[&str] = &[
    PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
    PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
];
const FRONTIER_JSON_RENDERER_PROFILE_IDS: &[&str] =
    &[PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID];
const FRONTIER_MARKDOWN_RENDERER_PROFILE_IDS: &[&str] =
    &[PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID];
const FRONTIER_BOTH_RENDERER_PROFILE_IDS: &[&str] = &[
    PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID,
    PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
];
const PROGRESSIVE_JSON_RENDERER_PROFILE_IDS: &[&str] =
    &[PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID];
const PROGRESSIVE_MARKDOWN_RENDERER_PROFILE_IDS: &[&str] =
    &[PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID];
const PROGRESSIVE_BOTH_RENDERER_PROFILE_IDS: &[&str] = &[
    PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID,
    PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
];

pub const PRIMARY_SOURCE_CONTEXT_PACK_PROFILE_ID: &str =
    wow_context::PRIMARY_SOURCE_CONTEXT_PACK_PROFILE;
pub const PRIMARY_SOURCE_CONTEXT_JSON_RENDERER_PROFILE_ID: &str =
    wow_context::PRIMARY_SOURCE_CONTEXT_JSON_RENDERER_PROFILE;
pub const PRIMARY_SOURCE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID: &str =
    wow_context::PRIMARY_SOURCE_CONTEXT_MARKDOWN_RENDERER_PROFILE;
const SOURCE_JSON_RENDERER_PROFILE_IDS: &[&str] =
    &[PRIMARY_SOURCE_CONTEXT_JSON_RENDERER_PROFILE_ID];
const SOURCE_MARKDOWN_RENDERER_PROFILE_IDS: &[&str] =
    &[PRIMARY_SOURCE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID];
const SOURCE_BOTH_RENDERER_PROFILE_IDS: &[&str] = &[
    PRIMARY_SOURCE_CONTEXT_JSON_RENDERER_PROFILE_ID,
    PRIMARY_SOURCE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
];

#[derive(Debug, Clone, Copy, Serialize)]
struct SourceExcerptRequest {
    selection: wow_context::PrimarySourceExcerptSelection,
    policy: wow_context::PrimarySourceExcerptPolicy,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextBuildRequest {
    schema: &'static str,
    project_store_id: Box<str>,
    publication: ContextMapPublication,
    platform: &'static str,
    level: ContextInspectLevel,
    root: ContextMapRoot,
    render: ContextRenderSelection,
    context_profile_id: &'static str,
    renderer_profile_ids: &'static [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    l0_profile_id: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    l1_profile_id: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expansion: Option<FrontierExpansion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    progressive: Option<ProgressiveExpansion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current: Option<ContextMapCurrentGuard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<ContextMapReferenceGuard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_excerpt: Option<SourceExcerptRequest>,
}
impl ContextBuildRequest {
    pub fn primary(
        store: &str,
        publication: ContextMapPublication,
        level: ContextInspectLevel,
        root: ContextMapRoot,
        render: ContextRenderSelection,
        current: Option<ContextMapCurrentGuard>,
        reference: Option<ContextMapReferenceGuard>,
    ) -> ServiceResult<Self> {
        super::request::registration_id(store)?;
        if current.is_some() && !matches!(publication, ContextMapPublication::Current) {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        let renderer_profile_ids: &'static [&'static str] = match &render {
            ContextRenderSelection::None => &[],
            ContextRenderSelection::Json => JSON_RENDERER_PROFILE_IDS,
            ContextRenderSelection::Markdown => MARKDOWN_RENDERER_PROFILE_IDS,
            ContextRenderSelection::Both => BOTH_RENDERER_PROFILE_IDS,
        };
        Ok(Self {
            schema: CONTEXT_BUILD_REQUEST_SCHEMA,
            project_store_id: store.into(),
            publication,
            platform: "omitted",
            level,
            root,
            render,
            context_profile_id: PRIMARY_CONTEXT_PACK_PROFILE_ID,
            renderer_profile_ids,
            l0_profile_id: matches!(level, ContextInspectLevel::L0 | ContextInspectLevel::Both)
                .then_some(PRIMARY_L0_PROFILE_ID),
            l1_profile_id: matches!(level, ContextInspectLevel::L1 | ContextInspectLevel::Both)
                .then_some(PRIMARY_L1_PROFILE_ID),
            expansion: None,
            progressive: None,
            current,
            reference,
            source_excerpt: None,
        })
    }

    pub fn with_l0_profile(mut self, profile: ContextL0Profile) -> ServiceResult<Self> {
        if self.level == ContextInspectLevel::L1
            || (profile == ContextL0Profile::StateRootV1
                && matches!(&self.root, ContextMapRoot::Project(_)))
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        self.l0_profile_id = Some(profile.profile_id());
        Ok(self)
    }

    pub fn with_l1_profile(mut self, profile: ContextL1Profile) -> ServiceResult<Self> {
        if (self.source_excerpt.is_some() && profile != ContextL1Profile::EntityV1)
            || !matches!(
                self.level,
                ContextInspectLevel::L1 | ContextInspectLevel::Both
            )
            || (matches!(
                profile,
                ContextL1Profile::StateDetailV1 | ContextL1Profile::ReferenceFactsV1
            ) && (self.level != ContextInspectLevel::L1
                || !matches!(&self.root, ContextMapRoot::GraphEntity(_))
                || self.expansion.is_some()
                || self.progressive.is_some()))
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        self.l1_profile_id = Some(match profile {
            ContextL1Profile::EntityV1 => PRIMARY_L1_PROFILE_ID,
            ContextL1Profile::StateDetailV1 => PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            ContextL1Profile::ReferenceFactsV1 => PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
        });
        Ok(self)
    }

    pub fn with_frontier_expansion(mut self, frontier: Vec<ContextMapRoot>) -> ServiceResult<Self> {
        if self.source_excerpt.is_some()
            || self.l1_profile_id == Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID)
            || self.l1_profile_id == Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID)
            || self.progressive.is_some()
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        self.expansion = Some(FrontierExpansion::new(self.level, &self.root, frontier)?);
        self.context_profile_id = PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID;
        self.renderer_profile_ids = match &self.render {
            ContextRenderSelection::None => &[],
            ContextRenderSelection::Json => FRONTIER_JSON_RENDERER_PROFILE_IDS,
            ContextRenderSelection::Markdown => FRONTIER_MARKDOWN_RENDERER_PROFILE_IDS,
            ContextRenderSelection::Both => FRONTIER_BOTH_RENDERER_PROFILE_IDS,
        };
        Ok(self)
    }

    pub fn with_progressive_expansion(
        mut self,
        rounds: Vec<Vec<ContextMapRoot>>,
    ) -> ServiceResult<Self> {
        if self.source_excerpt.is_some()
            || self.expansion.is_some()
            || self.l1_profile_id == Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID)
            || self.l1_profile_id == Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID)
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        self.progressive = Some(ProgressiveExpansion::new(self.level, &self.root, rounds)?);
        self.context_profile_id = PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID;
        self.renderer_profile_ids = match &self.render {
            ContextRenderSelection::None => &[],
            ContextRenderSelection::Json => PROGRESSIVE_JSON_RENDERER_PROFILE_IDS,
            ContextRenderSelection::Markdown => PROGRESSIVE_MARKDOWN_RENDERER_PROFILE_IDS,
            ContextRenderSelection::Both => PROGRESSIVE_BOTH_RENDERER_PROFILE_IDS,
        };
        Ok(self)
    }

    pub fn with_source_excerpt(
        mut self,
        selection: wow_context::PrimarySourceExcerptSelection,
        policy: wow_context::PrimarySourceExcerptPolicy,
    ) -> ServiceResult<Self> {
        if self.level != ContextInspectLevel::L1
            || !matches!(self.root, ContextMapRoot::GraphEntity(_))
            || self.l1_profile_id != Some(PRIMARY_L1_PROFILE_ID)
            || self.expansion.is_some()
            || self.progressive.is_some()
            || self.source_excerpt.is_some()
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        self.source_excerpt = Some(SourceExcerptRequest { selection, policy });
        self.context_profile_id = PRIMARY_SOURCE_CONTEXT_PACK_PROFILE_ID;
        self.renderer_profile_ids = match self.render {
            ContextRenderSelection::None => &[],
            ContextRenderSelection::Json => SOURCE_JSON_RENDERER_PROFILE_IDS,
            ContextRenderSelection::Markdown => SOURCE_MARKDOWN_RENDERER_PROFILE_IDS,
            ContextRenderSelection::Both => SOURCE_BOTH_RENDERER_PROFILE_IDS,
        };
        Ok(self)
    }
}

pub fn execute_context_build(
    host: &ContextMapHostInput,
    request: &ContextBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    execute_context_build_with_source(host, request, None, stop)
}

/// The source root is an explicit local capability, separate from the canonical
/// request and artifact identities. The old entrypoint supplies no capability.
pub fn execute_context_build_with_source(
    host: &ContextMapHostInput,
    request: &ContextBuildRequest,
    source_root: Option<&Path>,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    if request.source_excerpt.is_some() != source_root.is_some() {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let l0_profile = super::l0_profile_from_id(request.l0_profile_id)?;
    let l1_profile = super::l1_profile_from_id(request.l1_profile_id)?;
    let digest = wow_core::domain_separated_digest(CONTEXT_BUILD_REQUEST_SCHEMA, request)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let id = format!(
        "context-build-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    let selector = match &request.publication {
        ContextMapPublication::Current => ReadSelector::Current,
        ContextMapPublication::StoreGeneration(id) => ReadSelector::Exact(id.clone()),
    };
    let (result, closure) =
        with_checked_context_read(host, &request.project_store_id, &selector, stop, |read| {
            build(
                read,
                host,
                request,
                &id,
                (l0_profile, l1_profile),
                source_root,
                stop,
            )
        });
    match result {
        Ok((bytes, status)) => Ok(ContextMapResult {
            bytes: bytes.into_boxed_slice(),
            status,
            code: None,
        }),
        Err(error) => failed_operation(
            request,
            &id,
            error.code(),
            closure,
            CONTEXT_BUILD_RESULT_SCHEMA,
            "context_build",
        ),
    }
}

// This private envelope borrows the exact sealed owner value. It does not
// reconstruct owner authority from its serialization or copy semantic records.
#[derive(Serialize)]
struct Payload<'a, 'm, 'view> {
    resolved_request_id: Box<str>,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_at_acquisition: Option<&'a wow_store::project::CurrentPublication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_record_digest: Option<Box<str>>,
    configured_registration_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_project_store_id: Option<&'a str>,
    semantic_pack: &'a wow_context::ContextSemanticPack<'m, 'view>,
    semantic_pack_byte_length: usize,
    rendered_artifacts: &'a [wow_context::RenderedContextArtifact],
    operation_budget_usage: wow_context::ContextOperationBudgetUsage,
    validation: &'static str,
}

fn build(
    read: &LiveProjectRead,
    host: &ContextMapHostInput,
    request: &ContextBuildRequest,
    request_id: &str,
    profiles: (ContextL0Profile, ContextL1Profile),
    source_root: Option<&Path>,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus)> {
    let (l0_profile, l1_profile) = profiles;
    checkpoint(stop)?;
    let project = read.project();
    if project.configuration().project_id() != &host.project_id {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let current = read.current_at_acquisition();
    let current_digest = current.map(current_record_digest).transpose()?;
    if let Some(guard) = &request.current {
        let actual =
            current.ok_or_else(|| failure(ServiceErrorCode::CurrentGenerationUnavailable))?;
        if guard
            .record
            .as_ref()
            .is_some_and(|id| id != &actual.record_id)
            || guard
                .digest
                .as_ref()
                .is_some_and(|digest| Some(digest) != current_digest.as_ref())
        {
            return Err(failure(ServiceErrorCode::StoreCurrentConflict));
        }
    }
    if let Some(guard) = &request.reference
        && (&guard.profile != host.reference_profile.profile_id()
            || guard.generation.to_string() != host.reference.generation_id()
            || guard.view.as_ref() != host.reference.self_digest())
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let scope = match &request.root {
        ContextMapRoot::Project(id) => {
            if id != project.configuration().project_id()
                || request.level != ContextInspectLevel::L0
            {
                return Err(failure(ServiceErrorCode::InvalidRequest));
            }
            wow_context::L0Scope::Project(id.clone())
        }
        ContextMapRoot::GraphEntity(id) => wow_context::L0Scope::Entity(id.clone()),
    };
    let universe = wow_context::bind_context_universe_set(
        read.pair(),
        &host.reference,
        &host.reference_profile,
        host.limits.owner_limits()?,
        stop,
    )
    .map_err(context_error)?;
    let reference_api = if matches!(l1_profile, ContextL1Profile::ReferenceFactsV1) {
        Some(
            read.native_api()
                .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?,
        )
    } else {
        None
    };
    let resolved = wow_core::domain_separated_digest(
        "wow-service/context-build-resolved-request/1",
        &(
            request_id,
            read.publication_set_id(),
            read.store_generation_id(),
            universe.universe_set_id(),
            &request.root,
            &host.limits,
        ),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let resolved_request_id = format!(
        "context-build-resolved-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(resolved)
    )
    .into_boxed_str();
    let deliver = |mut pack: wow_context::ContextSemanticPack<'_, '_>| {
        wow_context::validate_context_semantic_pack(&universe, &mut pack, stop)
            .map_err(context_error)?;
        let mut rendered_artifacts = Vec::new();
        if matches!(
            &request.render,
            ContextRenderSelection::Json | ContextRenderSelection::Both
        ) {
            let artifact =
                wow_context::render_context_pack_json(&mut pack, stop).map_err(context_error)?;
            wow_context::validate_rendered_context_artifact(&mut pack, &artifact, stop)
                .map_err(context_error)?;
            rendered_artifacts.push(artifact);
        }
        if matches!(
            &request.render,
            ContextRenderSelection::Markdown | ContextRenderSelection::Both
        ) {
            let artifact = wow_context::render_context_pack_markdown(&mut pack, stop)
                .map_err(context_error)?;
            wow_context::validate_rendered_context_artifact(&mut pack, &artifact, stop)
                .map_err(context_error)?;
            rendered_artifacts.push(artifact);
        }
        // Snapshot the continuing private ledger only after every requested
        // rendering and validation. This delivery usage is not pack identity
        // or an exact-token/RSS measurement.
        let operation_budget_usage = pack.operation_budget_usage();
        let status = match pack.state() {
            wow_context::ProjectMapState::Partial => ContextMapStatus::Partial,
            wow_context::ProjectMapState::Truncated => ContextMapStatus::Truncated,
        };
        let payload = Payload {
            resolved_request_id,
            publication_set_id: read.publication_set_id(),
            store_generation_id: read.store_generation_id().as_str(),
            current_at_acquisition: current,
            current_record_digest: current_digest,
            configured_registration_id: &host.project_store_id,
            native_project_store_id: read.namespace().map(|namespace| namespace.id().as_str()),
            semantic_pack: &pack,
            semantic_pack_byte_length: pack.canonical_bytes().len(),
            rendered_artifacts: &rendered_artifacts,
            operation_budget_usage,
            validation: "exact_native_binding_and_canonical_bytes",
        };
        let envelope = Envelope {
            schema: CONTEXT_BUILD_RESULT_SCHEMA,
            operation: "context_build",
            request_id,
            request,
            status,
            resource_closure: "closed",
            failure_code: None,
            payload: Some(payload),
        };
        let remaining = if request.source_excerpt.is_some()
            || request.expansion.is_some()
            || request.progressive.is_some()
            || matches!(
                l1_profile,
                ContextL1Profile::StateDetailV1 | ContextL1Profile::ReferenceFactsV1
            ) {
            pack.remaining_metadata_bytes()
        } else {
            host.limits.max_metadata_bytes
        };
        let limit =
            usize::try_from(remaining).map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
        Ok((encode(&envelope, limit, stop)?, status))
    };
    if let Some(excerpt_selection) = &request.source_excerpt {
        let root = source_root.ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
        return with_native_origins_with_catalog(
            &universe,
            read,
            host,
            request.root.clone(),
            true,
            stop,
            |map, graph, catalog| {
                let mut pack = wow_context::build_context_semantic_pack_with_profile(
                    &universe,
                    map,
                    graph,
                    &scope,
                    request.level,
                    l0_profile,
                    stop,
                )
                .map_err(context_error)?;
                let prepared = wow_context::prepare_primary_source_excerpt(
                    &mut pack,
                    map,
                    catalog,
                    excerpt_selection.selection,
                    excerpt_selection.policy,
                    stop,
                )
                .map_err(context_error)?;
                let manifest =
                    wow_project::graph::RetainedProjectSourceManifest::admit_project_handles(
                        project,
                        catalog,
                        &[prepared.source_handle()],
                        stop,
                    )
                    .map_err(crate::live_project::project_error)?;
                let directory = wow_project::disk::ProjectInputDirectory::open(root)
                    .map_err(crate::live_project::project_error)?;
                let outcome = manifest.read_sources(
                    &directory,
                    catalog,
                    &[prepared.source_handle()],
                    prepared.read_limits(),
                    prepared.max_output_bytes(),
                    stop,
                );
                // This read-only directory owner closes synchronously on drop;
                // retained Project/store owners use the outer checked-close path.
                drop(directory);
                let report = outcome.map_err(crate::live_project::project_error)?;
                wow_context::adopt_primary_source_excerpt(&mut pack, prepared, &report, stop)
                    .map_err(context_error)?;
                deliver(pack)
            },
        );
    }
    if let Some(progressive) = &request.progressive {
        with_native_progressive(
            &universe,
            read,
            host,
            request.root.clone(),
            &progressive.rounds,
            stop,
            |step| {
                let pack = wow_context::build_context_semantic_pack_from_expansion(
                    &universe, &scope, step, stop,
                )
                .map_err(context_error)?;
                deliver(pack)
            },
        )
    } else if let Some(expansion) = &request.expansion {
        with_native_frontier(
            &universe,
            read,
            host,
            request.root.clone(),
            expansion,
            stop,
            |step| {
                let pack = wow_context::build_context_semantic_pack_from_expansion(
                    &universe, &scope, step, stop,
                )
                .map_err(context_error)?;
                deliver(pack)
            },
        )
    } else {
        with_native_origins(
            &universe,
            read,
            host,
            request.root.clone(),
            request.level != ContextInspectLevel::L0,
            stop,
            |map, graph| {
                if let Some(api) = reference_api {
                    let origins = api.reference_origins(map, graph, stop)?;
                    let pack = wow_context::build_context_semantic_pack_with_reference_facts(
                        &universe, map, graph, &scope, &origins, stop,
                    )
                    .map_err(context_error)?;
                    // Keep origin input through pack validation/rendering/encoding.
                    return deliver(pack);
                }
                // Renderer selection is intentionally absent from this owner request.
                let pack = if matches!(l1_profile, ContextL1Profile::StateDetailV1) {
                    wow_context::build_context_semantic_pack_with_l1_profile(
                        &universe, map, graph, &scope, l1_profile, stop,
                    )
                } else {
                    wow_context::build_context_semantic_pack_with_profile(
                        &universe,
                        map,
                        graph,
                        &scope,
                        request.level,
                        l0_profile,
                        stop,
                    )
                }
                .map_err(context_error)?;
                deliver(pack)
            },
        )
    }
}

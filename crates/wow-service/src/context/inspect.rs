//! Exact native skeleton acquisition. The common runner owns checked closure;
//! this operation selects typed context artifacts, never a second semantic map call.
use super::operation::{
    Envelope, current_record_digest, encode, failed_operation, with_checked_context_read,
    with_native_origins_with_catalog,
};
use super::{
    ContextInspectLevel, ContextL0Profile, ContextMapCurrentGuard, ContextMapHostInput,
    ContextMapPublication, ContextMapReferenceGuard, ContextMapResult, ContextMapRoot,
    ContextMapStatus, PRIMARY_FRONTIER_EXPANSION_PROFILE_ID, PRIMARY_L0_PROFILE_ID,
    PRIMARY_L1_PROFILE_ID, checkpoint, context_error, failure, graph_error,
};
use crate::{ServiceErrorCode, ServiceResult, live_project::LiveProjectRead};
use serde::Serialize;
use std::sync::atomic::AtomicBool;
use wow_core::{CanonicalResult, ContentDigest};
use wow_store::project::ReadSelector;

const RESULT_SCHEMA: &str = "wow-service/context-inspect-result/1";

#[derive(Debug, Clone, Serialize)]
struct FrontierExpansion {
    profile_id: &'static str,
    frontier: Vec<wow_graph::GraphNodeId>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextInspectRequest {
    schema: &'static str,
    project_store_id: Box<str>,
    publication: ContextMapPublication,
    platform: &'static str,
    level: ContextInspectLevel,
    root: ContextMapRoot,
    #[serde(skip_serializing_if = "Option::is_none")]
    l0_profile_id: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    l1_profile_id: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expansion: Option<FrontierExpansion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current: Option<ContextMapCurrentGuard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<ContextMapReferenceGuard>,
}
impl ContextInspectRequest {
    pub fn primary(
        store: &str,
        publication: ContextMapPublication,
        level: ContextInspectLevel,
        root: ContextMapRoot,
        current: Option<ContextMapCurrentGuard>,
        reference: Option<ContextMapReferenceGuard>,
    ) -> ServiceResult<Self> {
        super::request::registration_id(store)?;
        if current.is_some() && !matches!(publication, ContextMapPublication::Current) {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        Ok(Self {
            schema: "wow-service/context-inspect-request/1",
            project_store_id: store.into(),
            publication,
            platform: "omitted",
            level,
            root,
            l0_profile_id: matches!(level, ContextInspectLevel::L0 | ContextInspectLevel::Both)
                .then_some(PRIMARY_L0_PROFILE_ID),
            l1_profile_id: matches!(level, ContextInspectLevel::L1 | ContextInspectLevel::Both)
                .then_some(PRIMARY_L1_PROFILE_ID),
            expansion: None,
            current,
            reference,
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

    pub fn with_frontier_expansion(mut self, frontier: Vec<ContextMapRoot>) -> ServiceResult<Self> {
        if self.level != ContextInspectLevel::L1
            || !matches!(&self.root, ContextMapRoot::GraphEntity(_))
            || frontier.is_empty()
            || frontier.len() > 32
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        let mut ids = frontier
            .into_iter()
            .map(|root| match root {
                ContextMapRoot::GraphEntity(id) => Ok(id),
                ContextMapRoot::Project(_) => Err(failure(ServiceErrorCode::InvalidRequest)),
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        ids.sort();
        if ids.windows(2).any(|pair| pair[0] == pair[1])
            || matches!(&self.root, ContextMapRoot::GraphEntity(root) if ids.contains(root))
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        self.expansion = Some(FrontierExpansion {
            profile_id: PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
            frontier: ids,
        });
        Ok(self)
    }
}

pub fn execute_context_inspect(
    host: &ContextMapHostInput,
    request: &ContextInspectRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let l0_profile = super::l0_profile_from_id(request.l0_profile_id)?;
    let digest =
        wow_core::domain_separated_digest("wow-service/context-inspect-request/1", request)
            .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let id = format!(
        "context-inspect-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    let selector = match &request.publication {
        ContextMapPublication::Current => ReadSelector::Current,
        ContextMapPublication::StoreGeneration(id) => ReadSelector::Exact(id.clone()),
    };
    let (result, closure) =
        with_checked_context_read(host, &request.project_store_id, &selector, stop, |read| {
            build(read, host, request, &id, l0_profile, stop)
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
            RESULT_SCHEMA,
            "context_inspect",
        ),
    }
}

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
    skeletons: &'a wow_context::PrimarySkeletons<'m, 'view>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expansion: Option<&'a wow_context::ContextExpansionStep<'view>>,
}

fn build(
    read: &LiveProjectRead,
    host: &ContextMapHostInput,
    request: &ContextInspectRequest,
    request_id: &str,
    l0_profile: ContextL0Profile,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus)> {
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
    let resolved = wow_core::domain_separated_digest(
        "wow-service/context-inspect-resolved-request/1",
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
        "context-inspect-resolved-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(resolved)
    )
    .into_boxed_str();
    with_native_origins_with_catalog(
        &universe,
        read,
        host,
        request.root.clone(),
        request.level != ContextInspectLevel::L0,
        stop,
        |map, graph, catalog| {
            // Keep native receipts borrowed by the selected step through encoding.
            let mut receipts;
            let expansion = if let Some(selection) = &request.expansion {
                let mut plan = wow_context::plan_context_expansion(
                    &universe,
                    map,
                    graph,
                    &selection.frontier,
                    stop,
                )
                .map_err(context_error)?;
                for index in 0..plan.navigation_queries().len() {
                    checkpoint(stop)?;
                    let view = graph
                        .neighbors(&plan.navigation_queries()[index], stop)
                        .map_err(graph_error)?;
                    plan.admit_neighborhood(view, stop).map_err(context_error)?;
                }
                let mut prepared =
                    wow_context::prepare_context_expansion(plan, stop).map_err(context_error)?;
                receipts = Vec::with_capacity(prepared.explanation_queries().len());
                for index in 0..prepared.explanation_queries().len() {
                    checkpoint(stop)?;
                    let receipt = graph
                        .explain_with_evidence(
                            &prepared.explanation_queries()[index],
                            catalog,
                            host.limits.evidence,
                            stop,
                        )
                        .map_err(graph_error)?;
                    prepared
                        .admit_explanation(&receipt, stop)
                        .map_err(context_error)?;
                    receipts.push(receipt);
                }
                Some(
                    wow_context::expand_context_frontier(prepared, &receipts, stop)
                        .map_err(context_error)?,
                )
            } else {
                None
            };
            let unexpanded;
            let skeletons = if let Some(step) = &expansion {
                step.skeletons()
            } else {
                unexpanded = wow_context::build_primary_skeletons_with_profile(
                    &universe,
                    map,
                    graph,
                    &scope,
                    request.level,
                    l0_profile,
                    stop,
                )
                .map_err(context_error)?;
                &unexpanded
            };
            let state = match &expansion {
                Some(step) => step.state(),
                None => skeletons.state(),
            };
            let status = match state {
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
                skeletons,
                expansion: expansion.as_ref(),
            };
            let envelope = Envelope {
                schema: RESULT_SCHEMA,
                operation: "context_inspect",
                request_id,
                request,
                status,
                resource_closure: "closed",
                failure_code: None,
                payload: Some(payload),
            };
            let remaining = expansion
                .as_ref()
                .map_or(host.limits.max_metadata_bytes, |step| {
                    step.remaining_metadata_bytes()
                });
            let limit = usize::try_from(remaining)
                .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
            Ok((encode(&envelope, limit, stop)?, status))
        },
    )
}

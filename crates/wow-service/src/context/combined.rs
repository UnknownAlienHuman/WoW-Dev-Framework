//! One selected native operation retaining distinct Primary/platform authority.
use std::{cell::Cell, sync::atomic::AtomicBool};

use serde::{Deserialize, Serialize};
use wow_core::{CanonicalResult, ContentDigest};
use wow_store::project::{ReadSelector, StoreGenerationId};

use super::{
    ContextMapHostInput, ContextMapPublication, ContextMapRequest, ContextMapResult,
    ContextMapRoot, ContextMapStatus, ContextPlatformHostInput, ContextPlatformOperation,
    ContextPlatformRequest, ContextRenderSelection, PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
    PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID, PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
    PRIMARY_STATE_ROOT_L0_PROFILE_ID, checkpoint, context_error, failure,
    operation::{
        Envelope, encode, failed_operation, preflight, with_checked_context_read,
        with_primary_composition,
    },
    platform::{self, with_checked_platform_read, with_qualified_platform_input},
    request,
};
use crate::{ServiceErrorCode, ServiceResult, live_project::LiveProjectRead};

pub const CONTEXT_COMBINED_HOST_SCHEMA: &str = "wow-service/context-combined-native-host/1";
pub const CONTEXT_COMBINED_REQUEST_SCHEMA: &str = "wow-service/context-combined-request/1";
pub const CONTEXT_COMBINED_RESULT_SCHEMA: &str = "wow-service/context-combined-result/1";
pub const CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES: usize = super::CONTEXT_MAP_HOST_INPUT_MAX_BYTES;
pub const CONTEXT_COMBINED_REQUEST_MAX_BYTES: usize = super::CONTEXT_PLATFORM_REQUEST_MAX_BYTES;
pub const COMBINED_NATIVE_MAP_PROFILE_ID: &str = wow_context::COMBINED_NATIVE_ONE_HOP_PROFILE;
pub const COMBINED_CONTEXT_PACK_PROFILE_ID: &str = wow_context::COMBINED_CONTEXT_PACK_PROFILE;
pub const COMBINED_CONTEXT_JSON_RENDERER_PROFILE_ID: &str =
    wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE;
pub const COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID: &str =
    wow_context::COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextCombinedOperation {
    Map,
    Inspect,
    Build,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostDocument {
    schema: Box<str>,
    primary: ContextMapHostInput,
    platform: platform::HostDocument,
}

/// Strict inert registrations; paths are never semantic identity or output.
pub struct ContextCombinedHostInput {
    primary: ContextMapHostInput,
    platform: ContextPlatformHostInput,
}
impl ContextCombinedHostInput {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let document: HostDocument = crate::graph::input::decode(
            bytes,
            CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES,
            2_000_000,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|error| failure(error.code))?;
        if document.schema.as_ref() != CONTEXT_COMBINED_HOST_SCHEMA {
            return Err(failure(ServiceErrorCode::InvalidConfiguration));
        }
        let host = Self {
            primary: document.primary,
            platform: ContextPlatformHostInput::from_document(document.platform, stop)?,
        };
        host.validate(stop)?;
        Ok(host)
    }
    fn validate(&self, stop: &AtomicBool) -> ServiceResult<()> {
        self.primary.validate(stop)?;
        self.platform.validate(stop)?;
        if self.primary.limits.owner_limits()? != self.platform.limits.owner_limits()?
            || self.primary.limits.evidence != self.platform.limits.evidence
            || self.primary.reference_profile != self.platform.reference_profile
            || self.primary.reference.schema() != self.platform.reference.schema()
            || self.primary.reference.generation_id() != self.platform.reference.generation_id()
            || self.primary.reference.self_digest() != self.platform.reference.self_digest()
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        checkpoint(stop)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PrimarySelection {
    project_store_id: Box<str>,
    store_generation_id: StoreGenerationId,
    publication_set_id: Box<str>,
    map_profile_id: Box<str>,
    root: ContextMapRoot,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestDocument {
    schema: Box<str>,
    operation: Box<str>,
    profile_id: Box<str>,
    primary: PrimarySelection,
    platform: platform::RequestDocument,
    #[serde(default, deserialize_with = "non_null_inspection")]
    inspection: Option<Inspection>,
    #[serde(default, deserialize_with = "non_null_build")]
    build: Option<BuildSelection>,
}

fn non_null_inspection<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Inspection>, D::Error> {
    Inspection::deserialize(deserializer).map(Some)
}

fn non_null_build<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BuildSelection>, D::Error> {
    BuildSelection::deserialize(deserializer).map(Some)
}

fn non_null_profile_id<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Box<str>>, D::Error> {
    Box::<str>::deserialize(deserializer).map(Some)
}

fn select_l0_recipe(
    profile_id: Option<&str>,
    l1_profile_id_is_explicit: bool,
    level: wow_context::SkeletonLevel,
    root: &ContextMapRoot,
) -> ServiceResult<Option<CombinedL0Recipe>> {
    match profile_id {
        None => Ok(None),
        Some(PRIMARY_STATE_ROOT_L0_PROFILE_ID)
            if !l1_profile_id_is_explicit
                && level == wow_context::SkeletonLevel::L0
                && matches!(root, ContextMapRoot::GraphEntity(_)) =>
        {
            Ok(Some(CombinedL0Recipe::StateRoot))
        }
        Some(_) => Err(failure(ServiceErrorCode::InvalidRequest)),
    }
}

fn select_l1_recipe(
    profile_id: Option<&str>,
    level: wow_context::SkeletonLevel,
    root: &ContextMapRoot,
) -> ServiceResult<Option<CombinedL1Recipe>> {
    match profile_id {
        None => Ok(None),
        Some(PRIMARY_STATE_DETAIL_L1_PROFILE_ID)
            if matches!(
                level,
                wow_context::SkeletonLevel::L1 | wow_context::SkeletonLevel::Both
            ) && matches!(root, ContextMapRoot::GraphEntity(_)) =>
        {
            Ok(Some(CombinedL1Recipe::StateDetail))
        }
        Some(PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID)
            if level == wow_context::SkeletonLevel::L1
                && matches!(root, ContextMapRoot::GraphEntity(_)) =>
        {
            Ok(Some(CombinedL1Recipe::ReferenceFacts))
        }
        Some(_) => Err(failure(ServiceErrorCode::InvalidRequest)),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CombinedL1Recipe {
    StateDetail,
    ReferenceFacts,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CombinedL0Recipe {
    StateRoot,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Inspection {
    level: Box<str>,
    #[serde(
        default,
        deserialize_with = "non_null_profile_id",
        skip_serializing_if = "Option::is_none"
    )]
    l0_profile_id: Option<Box<str>>,
    #[serde(
        default,
        deserialize_with = "non_null_profile_id",
        skip_serializing_if = "Option::is_none"
    )]
    l1_profile_id: Option<Box<str>>,
}
impl Inspection {
    fn level(&self) -> ServiceResult<wow_context::SkeletonLevel> {
        skeleton_level(&self.level)
    }
}

fn skeleton_level(value: &str) -> ServiceResult<wow_context::SkeletonLevel> {
    match value {
        "l0" => Ok(wow_context::SkeletonLevel::L0),
        "l1" => Ok(wow_context::SkeletonLevel::L1),
        "both" => Ok(wow_context::SkeletonLevel::Both),
        _ => Err(failure(ServiceErrorCode::InvalidRequest)),
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BuildSelection {
    level: Box<str>,
    #[serde(
        default,
        deserialize_with = "non_null_profile_id",
        skip_serializing_if = "Option::is_none"
    )]
    l0_profile_id: Option<Box<str>>,
    context_profile_id: Box<str>,
    renderer_profile_ids: Vec<Box<str>>,
    #[serde(
        default,
        deserialize_with = "non_null_profile_id",
        skip_serializing_if = "Option::is_none"
    )]
    l1_profile_id: Option<Box<str>>,
}
impl BuildSelection {
    fn normalize(
        &mut self,
        stop: &AtomicBool,
    ) -> ServiceResult<(wow_context::SkeletonLevel, ContextRenderSelection)> {
        let level = skeleton_level(&self.level)?;
        if self.context_profile_id.as_ref() != COMBINED_CONTEXT_PACK_PROFILE_ID
            || self.renderer_profile_ids.len() > 2
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        let mut json = false;
        let mut markdown = false;
        for profile in &self.renderer_profile_ids {
            checkpoint(stop)?;
            match profile.as_ref() {
                COMBINED_CONTEXT_JSON_RENDERER_PROFILE_ID if !json => json = true,
                COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID if !markdown => markdown = true,
                _ => return Err(failure(ServiceErrorCode::InvalidRequest)),
            }
        }
        // Both uses one fixed order in intent and delivery, regardless of input order.
        if json
            && markdown
            && self.renderer_profile_ids[0].as_ref()
                == COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID
        {
            self.renderer_profile_ids.swap(0, 1);
        }
        let render = match (json, markdown) {
            (false, false) => ContextRenderSelection::None,
            (true, false) => ContextRenderSelection::Json,
            (false, true) => ContextRenderSelection::Markdown,
            (true, true) => ContextRenderSelection::Both,
        };
        checkpoint(stop)?;
        Ok((level, render))
    }
}

/// Exact inert selections; decoded records cannot create retained owner views.
#[derive(Serialize)]
pub struct ContextCombinedRequest {
    schema: &'static str,
    operation: &'static str,
    profile_id: &'static str,
    primary: PrimarySelection,
    platform: ContextPlatformRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    inspection: Option<Inspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<BuildSelection>,
    #[serde(skip)]
    selected_level: Option<wow_context::SkeletonLevel>,
    #[serde(skip)]
    l1_recipe: Option<CombinedL1Recipe>,
    #[serde(skip)]
    l0_recipe: Option<CombinedL0Recipe>,
    #[serde(skip)]
    render: ContextRenderSelection,
    #[serde(skip)]
    primary_request: ContextMapRequest,
}
impl ContextCombinedRequest {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let mut document: RequestDocument = crate::graph::input::decode(
            bytes,
            CONTEXT_COMBINED_REQUEST_MAX_BYTES,
            65_536,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|error| failure(error.code))?;
        if document.schema.as_ref() != CONTEXT_COMBINED_REQUEST_SCHEMA
            || !matches!(document.operation.as_ref(), "map" | "inspect" | "build")
            || document.profile_id.as_ref() != COMBINED_NATIVE_MAP_PROFILE_ID
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        let (operation, selected_level, render, l1_recipe, l0_recipe) = match (
            document.operation.as_ref(),
            document.inspection.as_ref(),
            document.build.as_mut(),
        ) {
            ("map", None, None) => (
                ContextCombinedOperation::Map,
                None,
                ContextRenderSelection::None,
                None,
                None,
            ),
            ("inspect", Some(selection), None) => {
                let level = selection.level()?;
                let l1_recipe = select_l1_recipe(
                    selection.l1_profile_id.as_deref(),
                    level,
                    &document.primary.root,
                )?;
                let l0_recipe = select_l0_recipe(
                    selection.l0_profile_id.as_deref(),
                    selection.l1_profile_id.is_some(),
                    level,
                    &document.primary.root,
                )?;
                (
                    ContextCombinedOperation::Inspect,
                    Some(level),
                    ContextRenderSelection::None,
                    l1_recipe,
                    l0_recipe,
                )
            }
            ("build", None, Some(selection)) => {
                let (level, render) = selection.normalize(stop)?;
                let l1_recipe = select_l1_recipe(
                    selection.l1_profile_id.as_deref(),
                    level,
                    &document.primary.root,
                )?;
                let l0_recipe = select_l0_recipe(
                    selection.l0_profile_id.as_deref(),
                    selection.l1_profile_id.is_some(),
                    level,
                    &document.primary.root,
                )?;
                (
                    ContextCombinedOperation::Build,
                    Some(level),
                    render,
                    l1_recipe,
                    l0_recipe,
                )
            }
            _ => return Err(failure(ServiceErrorCode::InvalidRequest)),
        };
        if matches!(&document.primary.root, ContextMapRoot::Project(_))
            && selected_level.is_some_and(|level| level != wow_context::SkeletonLevel::L0)
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        request::identifier(&document.primary.publication_set_id)?;
        // Reuse the original root/request admission, rather than accepting DTO authority.
        let root = match &document.primary.root {
            ContextMapRoot::Project(id) => ContextMapRoot::project(id.as_str())?,
            ContextMapRoot::GraphEntity(id) => ContextMapRoot::graph_entity(id.as_str())?,
        };
        let primary_request = ContextMapRequest::primary(
            &document.primary.project_store_id,
            ContextMapPublication::StoreGeneration(document.primary.store_generation_id.clone()),
            &document.primary.map_profile_id,
            Some(root),
            None,
            None,
        )?;
        let platform = ContextPlatformRequest::from_document(document.platform, stop)?;
        if platform.operation() != ContextPlatformOperation::Map {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        checkpoint(stop)?;
        Ok(Self {
            schema: CONTEXT_COMBINED_REQUEST_SCHEMA,
            operation: match operation {
                ContextCombinedOperation::Map => "map",
                ContextCombinedOperation::Inspect => "inspect",
                ContextCombinedOperation::Build => "build",
            },
            profile_id: COMBINED_NATIVE_MAP_PROFILE_ID,
            primary: document.primary,
            platform,
            inspection: document.inspection,
            build: document.build,
            selected_level,
            l1_recipe,
            l0_recipe,
            render,
            primary_request,
        })
    }

    pub fn operation(&self) -> ContextCombinedOperation {
        if self.build.is_some() {
            ContextCombinedOperation::Build
        } else if self.inspection.is_some() {
            ContextCombinedOperation::Inspect
        } else {
            ContextCombinedOperation::Map
        }
    }

    fn result_operation(&self) -> &'static str {
        match self.operation() {
            ContextCombinedOperation::Map => "context_combined_map",
            ContextCombinedOperation::Inspect => "context_combined_inspect",
            ContextCombinedOperation::Build => "context_combined_build",
        }
    }
}

pub fn execute_context_combined(
    host: &ContextCombinedHostInput,
    request: &ContextCombinedRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let digest = wow_core::domain_separated_digest(CONTEXT_COMBINED_REQUEST_SCHEMA, request)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let request_id = format!(
        "context-combined-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    let platform_closure = Cell::new("not_acquired");
    let outcome: ServiceResult<()> = (|| {
        host.validate(stop)?;
        let limit = usize::try_from(host.primary.limits.max_input_bytes)
            .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
        // Aggregate semantic-input reservation before either expensive acquisition.
        preflight(
            &(
                request,
                &host.primary.reference_profile,
                &host.primary.reference,
                &host.platform.reference_profile,
                &host.platform.reference,
                host.platform.selection.intent(),
                &host.primary.limits,
            ),
            limit,
            stop,
        )?;
        Ok(())
    })();
    if let Err(error) = outcome {
        return failed_operation(
            request,
            &request_id,
            error.code(),
            "not_acquired",
            CONTEXT_COMBINED_RESULT_SCHEMA,
            request.result_operation(),
        );
    }
    let selector = ReadSelector::Exact(request.primary.store_generation_id.clone());
    let (outcome, primary_closure) = with_checked_context_read(
        &host.primary,
        &request.primary_request.project_store_id,
        &selector,
        stop,
        |primary_read| {
            let root = qualify_primary(primary_read, host, request)?;
            let universe = wow_context::bind_context_universe_set(
                primary_read.pair(),
                &host.primary.reference,
                &host.primary.reference_profile,
                host.primary.limits.owner_limits()?,
                stop,
            )
            .map_err(context_error)?;
            with_primary_composition(
                &universe,
                primary_read,
                &host.primary,
                root.clone(),
                request
                    .selected_level
                    .is_some_and(|level| level != wow_context::SkeletonLevel::L0),
                stop,
                |primary, graph| {
                    let native_api = if request.l1_recipe == Some(CombinedL1Recipe::ReferenceFacts)
                    {
                        Some(
                            primary_read
                                .native_api()
                                .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?,
                        )
                    } else {
                        None
                    };
                    let (outcome, closure) = with_checked_platform_read(
                        &host.platform,
                        &request.platform,
                        stop,
                        |platform_read| {
                            with_qualified_platform_input(
                                platform_read,
                                &request.platform,
                                stop,
                                |view| {
                                    let mut map = wow_context::build_combined_context_map(
                                        primary,
                                        view,
                                        &host.platform.reference,
                                        &host.platform.reference_profile,
                                        stop,
                                    )
                                    .map_err(context_error)?;
                                    if let Some(level) = request.selected_level {
                                        let origins = native_api
                                            .map(|api| {
                                                api.combined_reference_origins(
                                                    &mut map, graph, stop,
                                                )
                                            })
                                            .transpose()?;
                                        let scope = match &root {
                                            ContextMapRoot::Project(id) => {
                                                wow_context::L0Scope::Project(id.clone())
                                            }
                                            ContextMapRoot::GraphEntity(id) => {
                                                wow_context::L0Scope::Entity(id.clone())
                                            }
                                        };
                                        let inspect = match (request.l0_recipe, request.l1_recipe) {
                                            (Some(CombinedL0Recipe::StateRoot), None) => {
                                                wow_context::build_combined_state_root_l0_inspect(
                                                    &mut map, &universe, graph, &scope, stop,
                                                )
                                            }
                                            (None, Some(CombinedL1Recipe::StateDetail)) => {
                                                wow_context::build_combined_state_detail_inspect_with_level(
                                                    &mut map,
                                                    &universe,
                                                    graph,
                                                    &scope,
                                                    level,
                                                    stop,
                                                )
                                            }
                                            (None, Some(CombinedL1Recipe::ReferenceFacts)) => {
                                                wow_context::build_combined_reference_facts_inspect(
                                                    &mut map,
                                                    &universe,
                                                    graph,
                                                    &scope,
                                                    origins.as_deref().ok_or_else(|| {
                                                        failure(ServiceErrorCode::InternalContractViolation)
                                                    })?,
                                                    stop,
                                                )
                                            }
                                            (None, None) => wow_context::build_combined_context_inspect(
                                                &mut map, &universe, graph, &scope, level, stop,
                                            ),
                                            (Some(_), Some(_)) => {
                                                return Err(failure(
                                                    ServiceErrorCode::InternalContractViolation,
                                                ));
                                            }
                                        }
                                        .map_err(context_error)?;
                                        if request.build.is_some() {
                                            let result = encode_build_result(
                                                host,
                                                request,
                                                &request_id,
                                                primary_read,
                                                platform_read,
                                                inspect,
                                                stop,
                                            );
                                            drop(origins);
                                            result
                                        } else {
                                            let delivery = Delivery {
                                                artifact: InspectPayload {
                                                    combined_inspect_byte_length: inspect
                                                        .canonical_bytes()
                                                        .len(),
                                                    combined_inspect: &inspect,
                                                },
                                                state: inspect.state(),
                                                remaining_metadata_bytes: inspect
                                                    .remaining_metadata_bytes(),
                                            };
                                            let result = encode_result(
                                                host,
                                                request,
                                                &request_id,
                                                primary_read,
                                                platform_read,
                                                delivery,
                                                stop,
                                            );
                                            drop(origins);
                                            result
                                        }
                                    } else {
                                        let delivery = Delivery {
                                            artifact: MapPayload {
                                                combined_map_byte_length: map
                                                    .canonical_bytes()
                                                    .len(),
                                                combined_map: &map,
                                            },
                                            state: map.state(),
                                            remaining_metadata_bytes: map
                                                .remaining_metadata_bytes(),
                                        };
                                        encode_result(
                                            host,
                                            request,
                                            &request_id,
                                            primary_read,
                                            platform_read,
                                            delivery,
                                            stop,
                                        )
                                    }
                                },
                            )
                        },
                    );
                    platform_closure.set(closure);
                    outcome
                },
            )
        },
    );
    let closure = combined_closure(primary_closure, platform_closure.get());
    match outcome {
        Ok((bytes, status)) if closure == "closed" => Ok(ContextMapResult {
            bytes: bytes.into_boxed_slice(),
            status,
            code: None,
        }),
        Ok(_) => failed_operation(
            request,
            &request_id,
            ServiceErrorCode::InternalContractViolation,
            closure,
            CONTEXT_COMBINED_RESULT_SCHEMA,
            request.result_operation(),
        ),
        Err(error) => failed_operation(
            request,
            &request_id,
            error.code(),
            closure,
            CONTEXT_COMBINED_RESULT_SCHEMA,
            request.result_operation(),
        ),
    }
}

fn combined_closure(primary: &'static str, platform: &'static str) -> &'static str {
    if primary == "failed" || platform == "failed" {
        "failed"
    } else if matches!(primary, "pending" | "not_evaluated")
        || matches!(platform, "pending" | "not_evaluated")
    {
        "not_evaluated"
    } else if primary == "closed" && matches!(platform, "closed" | "not_acquired") {
        "closed"
    } else {
        "not_acquired"
    }
}

fn qualify_primary(
    read: &LiveProjectRead,
    host: &ContextCombinedHostInput,
    request: &ContextCombinedRequest,
) -> ServiceResult<ContextMapRoot> {
    if read.project().configuration().project_id() != &host.primary.project_id
        || read.store_generation_id() != &request.primary.store_generation_id
        || read.publication_set_id() != request.primary.publication_set_id.as_ref()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    match &request.primary.root {
        ContextMapRoot::GraphEntity(id) => Ok(ContextMapRoot::GraphEntity(id.clone())),
        ContextMapRoot::Project(id)
            if request.primary_request.map_profile_id
                == PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID
                && id == read.project().configuration().project_id() =>
        {
            Ok(ContextMapRoot::Project(id.clone()))
        }
        _ => Err(failure(ServiceErrorCode::InvalidRequest)),
    }
}

#[derive(Serialize)]
struct ReadBinding<'a> {
    configured_registration_id: &'a str,
    store_generation_id: &'a str,
    publication_set_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_project_store_id: Option<&'a str>,
}
fn read_binding<'a>(read: &'a LiveProjectRead, registration: &'a str) -> ReadBinding<'a> {
    ReadBinding {
        configured_registration_id: registration,
        store_generation_id: read.store_generation_id().as_str(),
        publication_set_id: read.publication_set_id(),
        native_project_store_id: read.namespace().map(|namespace| namespace.id().as_str()),
    }
}
#[derive(Serialize)]
struct Payload<'a, S: Serialize, T: Serialize> {
    resolved_request_id: &'a str,
    primary: ReadBinding<'a>,
    platform: ReadBinding<'a>,
    platform_selection: &'a S,
    #[serde(flatten)]
    artifact: T,
}
#[derive(Serialize)]
struct MapPayload<'a, T: Serialize> {
    combined_map_byte_length: usize,
    combined_map: &'a T,
}
#[derive(Serialize)]
struct InspectPayload<'a, T: Serialize> {
    combined_inspect_byte_length: usize,
    combined_inspect: &'a T,
}
#[derive(Serialize)]
struct BuildPayload<'a, 'm, 'p, 'f> {
    semantic_pack_byte_length: usize,
    semantic_pack: &'a wow_context::CombinedContextSemanticPack<'m, 'p, 'f>,
    rendered_artifacts: &'a [wow_context::RenderedContextArtifact],
    operation_budget_usage: wow_context::ContextOperationBudgetUsage,
    validation: &'static str,
}
fn encode_build_result(
    host: &ContextCombinedHostInput,
    request: &ContextCombinedRequest,
    request_id: &str,
    primary_read: &LiveProjectRead,
    platform_read: &LiveProjectRead,
    inspect: wow_context::CombinedContextInspect<'_, '_, '_>,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus)> {
    let mut pack =
        wow_context::build_combined_context_semantic_pack(inspect, stop).map_err(context_error)?;
    wow_context::validate_combined_context_semantic_pack(&mut pack, stop).map_err(context_error)?;
    let mut rendered_artifacts = Vec::new();
    if matches!(
        &request.render,
        ContextRenderSelection::Json | ContextRenderSelection::Both
    ) {
        let artifact = wow_context::render_combined_context_pack_json(&mut pack, stop)
            .map_err(context_error)?;
        wow_context::validate_rendered_combined_context_artifact(&mut pack, &artifact, stop)
            .map_err(context_error)?;
        rendered_artifacts.push(artifact);
    }
    if matches!(
        &request.render,
        ContextRenderSelection::Markdown | ContextRenderSelection::Both
    ) {
        let artifact = wow_context::render_combined_context_pack_markdown(&mut pack, stop)
            .map_err(context_error)?;
        wow_context::validate_rendered_combined_context_artifact(&mut pack, &artifact, stop)
            .map_err(context_error)?;
        rendered_artifacts.push(artifact);
    }
    // The inspect was consumed; snapshot only after every render and validation.
    let delivery = Delivery {
        artifact: BuildPayload {
            semantic_pack_byte_length: pack.canonical_bytes().len(),
            semantic_pack: &pack,
            rendered_artifacts: &rendered_artifacts,
            operation_budget_usage: pack.operation_budget_usage(),
            validation: "exact_native_binding_and_canonical_bytes",
        },
        state: pack.state(),
        remaining_metadata_bytes: pack.remaining_metadata_bytes(),
    };
    encode_result(
        host,
        request,
        request_id,
        primary_read,
        platform_read,
        delivery,
        stop,
    )
}

struct Delivery<T> {
    artifact: T,
    state: wow_context::ProjectMapState,
    remaining_metadata_bytes: u64,
}
fn encode_result<T: Serialize>(
    host: &ContextCombinedHostInput,
    request: &ContextCombinedRequest,
    request_id: &str,
    primary_read: &LiveProjectRead,
    platform_read: &LiveProjectRead,
    delivery: Delivery<T>,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus)> {
    let primary = read_binding(primary_read, &host.primary.project_store_id);
    let platform = read_binding(platform_read, &host.platform.project_store_id);
    let digest = wow_core::domain_separated_digest(
        "wow-service/context-combined-resolved-request/1",
        &(
            request_id,
            &primary,
            &platform,
            host.platform.selection.intent(),
            &host.primary.reference_profile,
            host.primary.reference.schema(),
            host.primary.reference.generation_id(),
            host.primary.reference.self_digest(),
            &host.primary.limits,
        ),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let resolved_request_id = format!(
        "context-combined-resolved-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    let status = match delivery.state {
        wow_context::ProjectMapState::Partial => ContextMapStatus::Partial,
        wow_context::ProjectMapState::Truncated => ContextMapStatus::Truncated,
    };
    let payload = Payload {
        resolved_request_id: &resolved_request_id,
        primary,
        platform,
        platform_selection: host.platform.selection.intent(),
        artifact: delivery.artifact,
    };
    let envelope = Envelope {
        schema: CONTEXT_COMBINED_RESULT_SCHEMA,
        operation: request.result_operation(),
        request_id,
        request,
        status,
        resource_closure: "closed",
        failure_code: None,
        payload: Some(payload),
    };
    let limit = usize::try_from(delivery.remaining_metadata_bytes)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    Ok((encode(&envelope, limit, stop)?, status))
}

#[cfg(test)]
mod tests;

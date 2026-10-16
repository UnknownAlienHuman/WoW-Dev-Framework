//! Strict Build transport request; existing exact Context plan owns execution.
use super::*;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Render {
    None,
    Json,
    Markdown,
    Both,
}
impl Render {
    fn native(self) -> ContextRenderSelection {
        match self {
            Self::None => ContextRenderSelection::None,
            Self::Json => ContextRenderSelection::Json,
            Self::Markdown => ContextRenderSelection::Markdown,
            Self::Both => ContextRenderSelection::Both,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildInput {
    schema: String,
    selection_operation_id: String,
    selection_request_digest: String,
    selection_result_id: String,
    receipt_id: String,
    project_store_id: String,
    level: Level,
    render: Render,
    #[serde(default, deserialize_with = "selected_progressive")]
    progressive: Option<ProgressiveInput>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgressiveInput {
    context_profile_id: String,
    expansion_profile_id: String,
    continuation_retention_profile_id: String,
    frontier_rounds: Vec<Vec<String>>,
}
fn selected_progressive<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> Result<Option<ProgressiveInput>, D::Error> {
    ProgressiveInput::deserialize(decoder).map(Some)
}
pub struct SearchContextBuildRequest {
    base: SearchContextRequest,
    render: Render,
    progressive: Option<Vec<Vec<ContextMapRoot>>>,
}
impl SearchContextBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        input_json: &[u8],
        host: ContextMapHostInput,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if input_json.len() > 8192 {
            return Err(invalid());
        }
        let input: BuildInput = serde_json::from_slice(input_json).map_err(|_| invalid())?;
        if input.schema != "wow-service/search-context-build-input/1" {
            return Err(invalid());
        }
        let progressive = input
            .progressive
            .map(|selected| {
                if !matches!(input.level, Level::L1)
                    || selected.context_profile_id
                        != crate::context::PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID
                    || selected.expansion_profile_id != "wow-context/primary-native-progressive/1"
                    || selected.continuation_retention_profile_id
                        != crate::context::CONTEXT_CONTINUATION_RETENTION_PROFILE_ID
                    || !(2..=4).contains(&selected.frontier_rounds.len())
                    || selected
                        .frontier_rounds
                        .iter()
                        .any(|batch| batch.is_empty() || batch.len() > 32)
                {
                    return Err(invalid());
                }
                let rounds = selected
                    .frontier_rounds
                    .into_iter()
                    .map(|batch| {
                        batch
                            .into_iter()
                            .map(|id| ContextMapRoot::graph_entity(&id))
                            .collect::<ServiceResult<Vec<_>>>()
                    })
                    .collect::<ServiceResult<Vec<_>>>()?;
                // E3 validates semantic ordering/membership against the actual
                // receipt-selected root in execute_plan, never a transport root.
                Ok(rounds)
            })
            .transpose()?;
        let common = ContextInput {
            schema: "wow-service/search-context-input/1".into(),
            selection_operation_id: input.selection_operation_id,
            selection_request_digest: input.selection_request_digest,
            selection_result_id: input.selection_result_id,
            receipt_id: input.receipt_id,
            project_store_id: input.project_store_id,
            level: input.level,
        };
        let base = SearchContextRequest::new(
            operation_id,
            search_store,
            &serde_json::to_vec(&common).map_err(|_| invalid())?,
            host,
            store_limits,
            max_output_bytes,
            permission,
            stop,
        )?;
        Ok(Self {
            base,
            render: input.render,
            progressive,
        })
    }
}
pub(in crate::search) fn execute(
    request: &SearchContextBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    execute_plan(
        &request.base,
        match &request.progressive {
            Some(rounds) => {
                ContextAction::ProgressiveBuild(request.render.native(), rounds.clone())
            }
            None => ContextAction::Build(request.render.native()),
        },
        stop,
    )
}

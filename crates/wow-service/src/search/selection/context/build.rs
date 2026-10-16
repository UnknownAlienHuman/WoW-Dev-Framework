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
}
pub struct SearchContextBuildRequest {
    base: SearchContextRequest,
    render: Render,
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
        })
    }
}
pub(in crate::search) fn execute(
    request: &SearchContextBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    execute_plan(
        &request.base,
        ContextAction::Build(request.render.native()),
        stop,
    )
}

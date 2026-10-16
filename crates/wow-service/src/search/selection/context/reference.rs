//! Reference Search identity stays distinct from its exact native API projection.
use super::*;
use crate::context::reference_selection::{self, ReferenceContextAction};

pub(super) fn execute(
    request: &SearchContextRequest,
    receipt: &serde_json::Value,
    explained: &serde_json::Value,
    action: ContextAction,
    public_digest: &str,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let projection = request
        .input
        .reference_projection
        .as_ref()
        .ok_or_else(invalid)?;
    let selected = receipt.get("owner_entity").ok_or_else(mismatch)?;
    if field(selected, "kind")? != "reference_document" {
        return Err(mismatch());
    }
    let federated = field(explained, "schema")? == "wow-search/federated-candidate-explanation/1";
    let (summary, candidate) = if federated {
        if explained
            .pointer("/candidate/universe_class")
            .and_then(serde_json::Value::as_str)
            != Some("reference_platform_contract")
        {
            return Err(mismatch());
        }
        (
            explained
                .pointer("/query_context/components/reference_platform_contract/result_summary")
                .ok_or_else(mismatch)?,
            explained
                .pointer("/candidate/original_candidate")
                .ok_or_else(mismatch)?,
        )
    } else {
        (
            explained.get("query_context").ok_or_else(mismatch)?,
            explained.get("candidate").ok_or_else(mismatch)?,
        )
    };
    let owner = summary.get("owner").ok_or_else(mismatch)?;
    let document = candidate.get("document").ok_or_else(mismatch)?;
    let key = field(document, "entity_key")?;
    let record_digest = field(document, "owner_record_digest")?;
    if key != field(selected, "id")? {
        return Err(mismatch());
    }
    // Compare complete captured profile/view metadata before native acquisition.
    // Search data cannot reconstruct the held Reference or attest its membership.
    request.host.reference_guard(
        field(owner, "reference_generation")?,
        owner.pointer("/binding/profile").ok_or_else(mismatch)?,
        Some(field(owner, "reference_view_digest")?),
    )?;
    let operation = action.operation();
    let native_action = match action {
        ContextAction::Inspect => ReferenceContextAction::Inspect,
        ContextAction::Build(render) => ReferenceContextAction::Build(render),
        ContextAction::ProgressiveBuild(..) => return Err(invalid()),
    };
    let context = reference_selection::execute(
        &request.host,
        &projection.project_store_generation_id,
        key,
        record_digest,
        native_action,
        stop,
    )?;
    let payload = match context {
        None => serde_json::json!({"status":"not_evaluated","resource_closure":"closed",
            "context_admission":"not_evaluated","reason":"native_reference_projection_unavailable",
            "reference_projection":projection,
            "selection_receipt":receipt,"search_explanation":explained,"context_host_binding":request.host_binding}),
        Some(context) => {
            let (value, status) = checked_native_result(
                &context,
                field(explained, "status")?,
                &projection.project_store_generation_id,
                request.max_output_bytes.get(),
                stop,
            )?;
            serde_json::json!({"status":status,"resource_closure":"closed",
                "context_admission":"exact_native_reference_projection",
                "reference_projection":projection,
                "selection_receipt":receipt,"search_explanation":explained,"context":value,
                "context_host_binding":request.host_binding})
        }
    };
    durable_response(
        operation,
        &request.operation_id,
        public_digest,
        &payload,
        request.max_output_bytes.get(),
    )
}

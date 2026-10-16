//! Registered frontend uses the real retained owner and preserves its envelope.
use super::*;
use crate::frontend::{
    FrontendClientIdentity, FrontendContextInspectRequest, FrontendService,
    FrontendSessionOpenRequest, MCP_CONTEXT_PROFILE_ID, MCP_PROTOCOL_REVISION,
};
use crate::search::LocalMetadataPermission;

fn open(service: &FrontendService) -> TestResult<crate::frontend::FrontendSessionId> {
    Ok(service
        .frontend_session_open(FrontendSessionOpenRequest {
            protocol_revision: MCP_PROTOCOL_REVISION.into(),
            profile_id: MCP_CONTEXT_PROFILE_ID.into(),
            transport: "stdio".into(),
            client: FrontendClientIdentity::new("native-fixture", "1")?,
        })?
        .session_id)
}

#[test]
fn registered_frontend_preserves_native_inspect_and_refuses_other_scope() -> TestResult {
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let root = scratch.root.join("registered-store");
    let (publisher, graph, reference, profile) = owners(LUA, &stop)?;
    let (file, _, _) = source_file_root(&graph)?;
    let mut store = LiveProjectStore::create(&root, graph.snapshot().universe().as_str())?;
    let publication = store.publish(&publisher, &graph, "frontend-inspect-fixture", None, &stop)?;
    store.close()?;
    drop(graph);
    drop(publisher);
    let host_json = host_value(&root, &profile, &reference);
    let native_request = ContextInspectRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(file.as_str())?,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?;
    let native = execute_context_inspect(&decode_host(&host_json, &stop)?, &native_request, &stop)?;
    assert_eq!(native.status(), ContextMapStatus::Partial);
    let service = FrontendService::new_with_context_registration(
        decode_host(&host_json, &stop)?,
        LocalMetadataPermission::authorize_local_metadata_only(),
        &stop,
    )?;
    let session = open(&service)?;
    let arguments = json!({
        "projectStoreId": STORE_ID,
        "storeGenerationId": publication.generation_id.as_str(),
        "root": {"kind":"graph_entity", "id":file.as_str()},
        "level":"l1"
    });
    let request =
        FrontendContextInspectRequest::from_json_slice(&serde_json::to_vec(&arguments)?, &stop)?;
    let result = service.frontend_context_inspect(session, &request, &stop)?;
    assert_eq!(result.canonical_bytes(), native.canonical_bytes());
    let manifest = serde_json::to_string(service.frontend_capabilities())?;
    assert!(manifest.contains("configured_metadata_only"));
    assert!(!manifest.contains(crate::frontend::MCP_STDIO_PROFILE_ID));
    assert_eq!(
        service.frontend_session_get(session)?.schema.as_ref(),
        "wow-service/frontend-context-session/1"
    );
    assert!(!manifest.contains(root.to_str().ok_or("fixture root encoding")?));
    assert!(!format!("{service:?}").contains(root.to_str().ok_or("fixture root encoding")?));

    // A distinct service cannot inherit this client's session/registration.
    let other = FrontendService::new_with_context_registration(
        decode_host(&host_json, &stop)?,
        LocalMetadataPermission::authorize_local_metadata_only(),
        &stop,
    )?;
    let other_session = open(&other)?;
    assert!(
        other
            .frontend_context_inspect(session, &request, &stop)
            .is_err()
    );
    assert!(
        service
            .frontend_context_inspect(other_session, &request, &stop)
            .is_err()
    );
    let mut wrong = arguments.clone();
    wrong["projectStoreId"] = "other-registration".into();
    let wrong =
        FrontendContextInspectRequest::from_json_slice(&serde_json::to_vec(&wrong)?, &stop)?;
    assert!(
        service
            .frontend_context_inspect(session, &wrong, &stop)
            .is_err()
    );
    for (field, value) in [
        ("storeGenerationId", json!("current")),
        ("path", json!("private.lua")),
        ("level", json!("l2")),
    ] {
        let mut invalid = arguments.clone();
        invalid[field] = value;
        assert!(
            FrontendContextInspectRequest::from_json_slice(&serde_json::to_vec(&invalid)?, &stop)
                .is_err()
        );
    }
    assert_eq!(
        service
            .frontend_context_inspect(session, &request, &AtomicBool::new(true))
            .err()
            .map(|error| error.code()),
        Some(ServiceErrorCode::Cancelled)
    );
    service.frontend_session_close(session)?;
    assert!(
        service
            .frontend_context_inspect(session, &request, &stop)
            .is_err()
    );
    other.frontend_session_close(other_session)?;
    // Both the direct operation and frontend operation released native custody.
    LiveProjectStore::open(&root)?.close()?;
    Ok(())
}

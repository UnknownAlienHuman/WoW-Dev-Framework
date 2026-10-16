//! Exact original Reference record -> accepted native project API association.
//! Construction uses the same E3 engine and the same checked native acquisition.
use super::*;
use wow_store::project::{ReadSelector, StoreGenerationId};

pub(crate) enum ReferenceContextAction {
    Inspect,
    Build(ContextRenderSelection),
}

pub(crate) fn execute(
    host: &ContextMapHostInput,
    generation: &str,
    record_key: &str,
    record_digest: &str,
    action: ReferenceContextAction,
    stop: &AtomicBool,
) -> ServiceResult<Option<ContextMapResult>> {
    checkpoint(stop)?;
    let generation = StoreGenerationId::parse(generation)
        .map_err(|_| failure(ServiceErrorCode::InvalidRequest))?;
    let (result, _) = operation::with_checked_context_read(
        host,
        host.registration_id(),
        &ReadSelector::Exact(generation.clone()),
        stop,
        |read| {
            if read.project().configuration().project_id() != &host.project_id {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            let Some(api) = read.native_api() else {
                return Ok(None);
            };
            if !api.matches_reference(&host.reference_profile, &host.reference) {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            let graph = read.pair().graph_read(stop).map_err(graph_error)?;
            let Some(root) = api.reference_root(&graph, record_key, record_digest, stop)? else {
                return Ok(None);
            };
            let (profile, reference_generation, view) = host.reference_identity();
            let reference = Some(ContextMapReferenceGuard::new(
                profile,
                reference_generation,
                view,
            )?);
            let publication = ContextMapPublication::store_generation(generation.as_str())?;
            let root = ContextMapRoot::graph_entity(root.as_str())?;
            let draft = match action {
                ReferenceContextAction::Inspect => {
                    let request = ContextInspectRequest::primary(
                        host.registration_id(),
                        publication,
                        ContextInspectLevel::L1,
                        root,
                        None,
                        reference,
                    )?
                    .with_l1_profile(ContextL1Profile::ReferenceFactsV1)?;
                    inspect::metadata_in_read(read, host, &request, stop)?
                }
                ReferenceContextAction::Build(render) => {
                    let request = ContextBuildRequest::primary(
                        host.registration_id(),
                        publication,
                        ContextInspectLevel::L1,
                        root,
                        render,
                        None,
                        reference,
                    )?
                    .with_l1_profile(ContextL1Profile::ReferenceFactsV1)?;
                    build::metadata_in_read(read, host, &request, stop)?
                }
            };
            Ok(Some(draft))
        },
    );
    // A draft is private until the checked driver has released read and Store.
    Ok(result?.map(|(bytes, status)| ContextMapResult {
        bytes: bytes.into_boxed_slice(),
        status,
        code: None,
    }))
}

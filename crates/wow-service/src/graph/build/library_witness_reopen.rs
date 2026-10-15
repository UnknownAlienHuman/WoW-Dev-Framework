//! Recompute only the explicitly selected W6 producer from the held native pair.
use super::{
    SourceGraphAddressCrosswalk, SourceGraphDiagnostics, checkpoint, error, project_error,
    source_prefix,
};
use crate::{ServiceErrorCode, ServiceResult};
use std::sync::atomic::AtomicBool;
use wow_project::replay::publication::AcquiredProjectPair;
use wow_recognizers::source_state::{
    SOURCE_STATE_LIBRARY_PARTITION, SourceLibraryInput, recognize_source_library_witnesses,
};

pub(super) fn validate(pair: &AcquiredProjectPair, stop: &AtomicBool) -> ServiceResult<()> {
    let project = pair.project();
    match project.configuration().library_witnesses() {
        None => return Ok(()),
        Some(wow_project::LibraryWitnessRecipe::MatcherV1) => {}
    }
    checkpoint(stop)?;
    let mut diagnostics = SourceGraphDiagnostics::default();
    let source_prefix::SourcePrefix { owner, projection } =
        source_prefix::prepare(project, stop, &mut diagnostics)?;
    let (legacy, direct) = match projection {
        source_prefix::SourceProjection::Legacy(source) => (Some(*source), None),
        source_prefix::SourceProjection::Direct(plan) => {
            let native = (*plan)
                .finish_diagnosed(&owner, stop, &mut diagnostics)
                .map_err(project_error)?;
            (None, Some(native))
        }
    };
    let addresses = match (&legacy, &direct) {
        (Some(source), None) => SourceGraphAddressCrosswalk::legacy(source, &owner, stop)?,
        (None, Some(native)) => SourceGraphAddressCrosswalk::direct(native, stop)?,
        _ => return Err(error(ServiceErrorCode::IdentityMismatch)),
    };
    let source = addresses.source();
    let rebuilt = recognize_source_library_witnesses(
        SourceLibraryInput {
            owner: pair.graph(),
            source_partition: addresses.analyzer_partition(pair.graph(), stop)?,
            report: source
                .function_call_report()
                .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?,
            context: source.context(),
            function_proposals: source
                .functions()
                .iter()
                .map(|function| (function.function_id.as_str(), function.proposal_id.as_str()))
                .collect(),
            call_support: source
                .call_sites()
                .iter()
                .map(|site| {
                    (
                        site.call_id.as_str(),
                        (site.source_handle_id, site.evidence_id),
                    )
                })
                .collect(),
            source_handles: source.source_handles(),
            evidence: source.evidence(),
        },
        stop,
    )
    .map_err(|failure| {
        error(match failure.code() {
            wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            wow_recognizers::RecognizerErrorCode::BudgetExceeded => {
                ServiceErrorCode::BudgetExceeded
            }
            _ => ServiceErrorCode::IdentityMismatch,
        })
    })?;
    let retained = pair
        .graph()
        .partition(SOURCE_STATE_LIBRARY_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
    if retained.batch() != &rebuilt.batch || retained.coverage() != rebuilt.coverage {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    checkpoint(stop)
}

//! Recompute selected W1/W3/W4/W6 proof producers from the held native pair.
use super::{
    SourceGraphAddressCrosswalk, SourceGraphDiagnostics, checkpoint, error, project_error,
    source_prefix,
};
use crate::{ServiceErrorCode, ServiceResult};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use wow_project::replay::publication::AcquiredProjectPair;
use wow_recognizers::source_signals::{
    W1_SIGNAL_PARTITION, W1Input, W3_SIGNAL_PARTITION, W3Input, W4_PARTITION,
    recognize_source_cvar_witnesses, w1_recognize_native_frame_event_witnesses,
    w3_recognize_signal_witnesses,
};
use wow_recognizers::source_state::{
    SOURCE_STATE_LIBRARY_PARTITION, SourceLibraryInput, recognize_source_library_witnesses,
};

pub(super) fn validate(pair: &AcquiredProjectPair, stop: &AtomicBool) -> ServiceResult<()> {
    let project = pair.project();
    if project.configuration().library_witnesses().is_none()
        && project.configuration().signal_witnesses().is_none()
    {
        return Ok(());
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
    if let Some(wow_project::SignalWitnessRecipe::MatcherV1) =
        project.configuration().signal_witnesses()
    {
        let report = source
            .function_call_report()
            .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
        let partition = addresses.analyzer_partition(pair.graph(), stop)?;
        let functions = source
            .functions()
            .iter()
            .map(|function| (function.function_id.as_str(), function.proposal_id.as_str()))
            .collect::<BTreeMap<_, _>>();
        let calls = source
            .call_sites()
            .iter()
            .map(|site| {
                (
                    site.call_id.as_str(),
                    (site.source_handle_id, site.evidence_id),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let frame = w1_recognize_native_frame_event_witnesses(
            W1Input {
                owner: pair.graph(),
                source_partition: partition,
                report,
                context: source.context(),
                function_proposals: functions.clone(),
                call_support: calls.clone(),
                source_handles: source.source_handles(),
                evidence: source.evidence(),
            },
            stop,
        )
        .map_err(recognizer_error)?;
        compare(pair, W1_SIGNAL_PARTITION, &frame.batch, &frame.coverage)?;
        let declarations = source
            .lua_declarations()
            .iter()
            .map(|declaration| {
                (
                    (declaration.path.clone(), declaration.span),
                    declaration.proposal_id.as_str(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let input = || W3Input {
            owner: pair.graph(),
            source_partition: partition,
            report,
            context: source.context(),
            function_proposals: functions.clone(),
            call_support: calls.clone(),
            declaration_proposals: declarations.clone(),
            source_handles: source.source_handles(),
            evidence: source.evidence(),
        };
        let custom = w3_recognize_signal_witnesses(input(), stop).map_err(recognizer_error)?;
        compare(pair, W3_SIGNAL_PARTITION, &custom.batch, &custom.coverage)?;
        let cvar = recognize_source_cvar_witnesses(input(), stop).map_err(recognizer_error)?;
        compare(pair, W4_PARTITION, &cvar.batch, &cvar.coverage)?;
    }
    if project.configuration().library_witnesses().is_none() {
        return checkpoint(stop);
    }
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
    .map_err(recognizer_error)?;
    compare(
        pair,
        SOURCE_STATE_LIBRARY_PARTITION,
        &rebuilt.batch,
        &rebuilt.coverage,
    )?;
    checkpoint(stop)
}

fn compare(
    pair: &AcquiredProjectPair,
    partition: &str,
    batch: &wow_graph::GraphProposalBatch,
    coverage: &[wow_graph::GraphCoverageRecord],
) -> ServiceResult<()> {
    let retained = pair
        .graph()
        .partition(partition)
        .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
    if retained.batch() != batch || retained.coverage() != coverage {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    Ok(())
}
fn recognizer_error(failure: wow_recognizers::RecognizerError) -> crate::ServiceError {
    error(match failure.code() {
        wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        wow_recognizers::RecognizerErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::IdentityMismatch,
    })
}

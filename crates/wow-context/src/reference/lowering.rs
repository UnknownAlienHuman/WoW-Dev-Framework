//! Admit existing emitter observations against Reference and verified Library bytes.
//! The Core value is data; neither its constructor nor token agreement proves
//! consumer semantics, complete types, runtime behavior or clean absence.
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{
    CallableLoweringLane, CallableLoweringObservation, ContentDigest, ProjectGenerationId,
    SourceContent, SourceHandle, SourceSpan,
};
use wow_project::ProjectLibraryFileView;
use wow_reference::native_view::NativeCallableSignature;

use super::mismatch;
use crate::budget::{Budget, checkpoint, exhausted};
use crate::{ContextError, ContextErrorCode, ContextResult, ContextUniverseSet};

#[derive(Clone, Copy, Serialize)]
pub(super) struct AnnotationLoweringInput<'a> {
    pub(super) observation: CallableLoweringObservation<'a>,
    #[serde(skip)]
    pub(super) file: ProjectLibraryFileView<'a>,
    #[serde(skip)]
    pub(super) alias_file: Option<ProjectLibraryFileView<'a>>,
}

#[derive(Serialize)]
pub(super) struct AnnotationLoweringFacet<'a> {
    observation: CallableLoweringObservation<'a>,
    library_snapshot_id: &'a str,
    project_snapshot_id: &'a str,
    project_generation: ProjectGenerationId,
    byte_verification: &'static str,
    annotation_producer_custody: &'static str,
}

pub(super) fn admit<'a>(
    input: AnnotationLoweringInput<'a>,
    signature: NativeCallableSignature<'a>,
    source: &SourceHandle,
    universe: &ContextUniverseSet<'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<AnnotationLoweringFacet<'a>> {
    let observation = input.observation;
    let file = input.file;
    let project = universe.pair.project();
    let comparisons = [
        (
            observation.profile_id(),
            project
                .configuration()
                .selected_profile()
                .profile_id()
                .as_str(),
        ),
        (
            observation.reference_generation(),
            signature.reference_generation(),
        ),
        (
            observation.reference_view_digest(),
            signature.reference_view_digest(),
        ),
        (observation.record_key(), signature.record_key()),
        (observation.record_digest(), signature.record_digest()),
        (observation.source_id(), signature.source_id()),
        (observation.source_path(), source.path().as_str()),
        (observation.generated_path(), file.path()),
        (
            file.project_snapshot_id(),
            universe.binding.project_snapshot_id,
        ),
    ];
    let work = comparisons
        .iter()
        .try_fold(144usize, |work, (actual, expected)| {
            work.checked_add(actual.len())
                .and_then(|work| work.checked_add(expected.len()))
        })
        .ok_or_else(exhausted)?;
    budget.work(work, stop)?;
    for (actual, expected) in comparisons {
        checkpoint(stop)?;
        if actual != expected {
            return Err(mismatch());
        }
    }
    if file.project_generation() != universe.binding.project_generation
        || !project
            .snapshot()
            .analyzer_binding()
            .library_snapshot_ids()
            .eq([file.library_snapshot_id()])
        || ContentDigest::<SourceContent>::parse(observation.source_sha256())
            .map_err(|_| mismatch())?
            .into_value()
            != *source.content_digest()
        || ContentDigest::<SourceContent>::parse(observation.generated_sha256())
            .map_err(|_| mismatch())?
            .into_value()
            != ContentDigest::<SourceContent>::parse(file.sha256())
                .map_err(|_| mismatch())?
                .into_value()
    {
        return Err(mismatch());
    }
    let (start, end) = range(observation.declaration_span())?;
    // Slicing verifies final UTF-8 geometry, without parsing the declaration.
    verified_slice(file, start, end, stop)?;
    let count = signature
        .arguments()
        .len()
        .checked_add(signature.returns().len())
        .ok_or_else(exhausted)?;
    budget.work(
        count
            .checked_add(observation.fields().len())
            .ok_or_else(exhausted)?,
        stop,
    )?;
    if count != observation.fields().len() {
        return Err(mismatch());
    }
    for field in observation.fields() {
        checkpoint(stop)?;
        let original = match field.lane() {
            CallableLoweringLane::Parameter => signature.arguments().get(field.ordinal()),
            CallableLoweringLane::Return => signature.returns().get(field.ordinal()),
        }
        .ok_or_else(mismatch)?;
        if range(field.original_source_span())?
            != (original.source_span().start, original.source_span().end)
        {
            return Err(mismatch());
        }
        let (member_start, member_end) = range(field.generated_member_span())?;
        let (token_start, token_end) = range(field.type_token_span())?;
        if member_start < start
            || member_end > end
            || token_start < member_start
            || token_end > member_end
        {
            return Err(mismatch());
        }
        verified_slice(file, member_start, member_end, stop)?;
        budget.work(field.lowered_type().len(), stop)?;
        let token = verified_slice(file, token_start, token_end, stop)?;
        if token != field.lowered_type() {
            return Err(mismatch());
        }
        for atom in field.type_atoms() {
            let (atom_start, atom_end) = range(atom.generated_span())?;
            budget.work(
                atom.source_name()
                    .len()
                    .checked_add(atom.rendered_name().len())
                    .ok_or_else(exhausted)?,
                stop,
            )?;
            if atom_start < token_start
                || atom_end > token_end
                || verified_slice(file, atom_start, atom_end, stop)? != atom.rendered_name()
            {
                return Err(mismatch());
            }
        }
        if let Some(marker) = field.optional_marker_span() {
            let (marker_start, marker_end) = range(marker)?;
            budget.work(1, stop)?;
            if marker_start < member_start
                || marker_end > member_end
                || verified_slice(file, marker_start, marker_end, stop)? != "?"
            {
                return Err(mismatch());
            }
        }
    }
    let aliases = observation.alias_definitions();
    if aliases.is_empty() != input.alias_file.is_none() {
        return Err(mismatch());
    }
    if let Some(alias_file) = input.alias_file {
        budget.work(aliases.len().checked_add(144).ok_or_else(exhausted)?, stop)?;
        let alias_digest = ContentDigest::<SourceContent>::parse(alias_file.sha256())
            .map_err(|_| mismatch())?
            .into_value();
        if alias_file.project_snapshot_id() != file.project_snapshot_id()
            || alias_file.project_generation() != file.project_generation()
            || alias_file.library_snapshot_id() != file.library_snapshot_id()
        {
            return Err(mismatch());
        }
        for definition in aliases {
            checkpoint(stop)?;
            let work = definition
                .generated_path()
                .len()
                .checked_add(71)
                .and_then(|work| work.checked_add(definition.lowered_definition().len()))
                .ok_or_else(exhausted)?;
            budget.work(work, stop)?;
            let (declaration_start, declaration_end) = range(definition.declaration_span())?;
            let (token_start, token_end) = range(definition.type_token_span())?;
            if definition.generated_path() != alias_file.path()
                || definition.generated_sha256() != &alias_digest
                || token_start < declaration_start
                || token_end > declaration_end
                || verified_slice(alias_file, token_start, token_end, stop)?
                    != definition.lowered_definition()
            {
                return Err(mismatch());
            }
            verified_slice(alias_file, declaration_start, declaration_end, stop)?;
        }
    }
    Ok(AnnotationLoweringFacet {
        observation,
        library_snapshot_id: file.library_snapshot_id(),
        project_snapshot_id: file.project_snapshot_id(),
        project_generation: file.project_generation(),
        byte_verification: "exact_admitted_library_file_and_emitted_token_ranges",
        annotation_producer_custody: "NotEvaluated",
    })
}

fn verified_slice<'a>(
    file: ProjectLibraryFileView<'a>,
    start: usize,
    end: usize,
    stop: &AtomicBool,
) -> ContextResult<&'a str> {
    file.slice(start, end, stop)
        .map_err(|error| match error.code() {
            wow_project::ProjectErrorCode::AnalysisCancelled => {
                ContextError::new(ContextErrorCode::Cancelled)
            }
            _ => mismatch(),
        })
}

fn range(span: SourceSpan) -> ContextResult<(usize, usize)> {
    match (span.byte_start(), span.byte_end()) {
        (Some(start), Some(end)) if start < end => Ok((
            usize::try_from(start).map_err(|_| mismatch())?,
            usize::try_from(end).map_err(|_| mismatch())?,
        )),
        _ => Err(mismatch()),
    }
}

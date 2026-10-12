//! Exact native Library target associations, without accepted graph authority.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use wow_annotations::native::AnnotationFile;
use wow_annotations::navigation::{
    LookupError, MappingPrecision, NavigationIndex, SourceLocation, SourceLookup,
};
use wow_annotations::reference_links::{
    NativeCallableBinding, NativeCallableBindings, NativeCallableMapping,
    NativeCallableReferenceStatus,
};
use wow_annotations::sidecars::{NativeSourceMapEntry, NativeSourceMapFile};
use wow_core::SourceSpanKind;
use wow_emmy::bindings::SymbolTarget;
use wow_emmy::function_calls::SourceCallTarget;
use wow_project::ProjectView;
use wow_reference::native::Span;
use wow_reference::native_view::NATIVE_API_PARTITION;
use wow_reference::{
    LookupResult, ReferenceConflict, ReferenceRecord, ReferenceRecordKind, ReferenceView,
    ReferenceViewError,
};

use super::cancelled;
use super::native_library_input::RequalifiedNativeLibrary;
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

/// A positive origin join only. Fields cannot be supplied or deserialized.
/// Its target is borrowed from the supplied project's actual native report.
pub struct NativeApiTargetAssociation<'a> {
    binding: &'a NativeCallableBinding,
    record: &'a ReferenceRecord,
    target: &'a SymbolTarget,
}
impl<'a> NativeApiTargetAssociation<'a> {
    #[must_use]
    pub const fn binding(&self) -> &'a NativeCallableBinding {
        self.binding
    }
    #[must_use]
    pub const fn reference_record(&self) -> &'a ReferenceRecord {
        self.record
    }
    #[must_use]
    pub const fn target(&self) -> &'a SymbolTarget {
        self.target
    }
}

/// Unmapped outcomes retain their reason; none proves native API absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeApiTargetOmission {
    NotObservedInNativeCallableReport,
    NoSourceMapping,
    NoNativeCallableOrigin,
}

pub enum NativeApiTargetResolution<'a> {
    Bound(NativeApiTargetAssociation<'a>),
    Unmapped(NativeApiTargetOmission),
    AmbiguousMapping {
        precision: MappingPrecision,
        candidate_count: usize,
    },
    /// Member, literal-file and external alias mappings cannot become API origins.
    UnsupportedMapping {
        precision: MappingPrecision,
    },
    ReferenceConflict {
        binding: &'a NativeCallableBinding,
        conflict: &'a ReferenceConflict,
        target: &'a SymbolTarget,
    },
    AmbiguousOrigin,
}

/// One prepared navigation index and native target set for one exact project.
/// Batch consumers retain this resolver instead of preparing once per record.
pub struct NativeApiTargetResolver<'a> {
    library: &'a RequalifiedNativeLibrary,
    library_snapshot_id: &'a str,
    navigation: NavigationIndex<'a>,
    files: BTreeMap<&'a str, (&'a AnnotationFile, &'a NativeSourceMapFile)>,
    targets: BTreeSet<&'a SymbolTarget>,
}

impl RequalifiedNativeLibrary {
    pub fn target_resolver<'a>(
        &'a self,
        project: &'a ProjectView,
        stop: &AtomicBool,
    ) -> ServiceResult<NativeApiTargetResolver<'a>> {
        NativeApiTargetResolver::new(self, project, stop)
    }

    /// Single-target convenience; batches must retain target_resolver().
    pub fn resolve_target<'a>(
        &'a self,
        project: &'a ProjectView,
        target: &SymbolTarget,
        stop: &AtomicBool,
    ) -> ServiceResult<NativeApiTargetResolution<'a>> {
        self.target_resolver(project, stop)?
            .resolve_target(target, stop)
    }
}

impl<'a> NativeApiTargetResolver<'a> {
    pub fn new(
        library: &'a RequalifiedNativeLibrary,
        project: &'a ProjectView,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        cancelled(stop)?;
        let configuration = project.configuration();
        let reference = &library.reference().view;
        let artifact = library.artifact();
        let map = library.sidecars().source_map();
        let bindings = library.callable_bindings();
        if project.snapshot_id() != library.project_snapshot_id()
            || configuration.native_library_input().is_none()
            || configuration.selected_profile() != library.profile()
            || configuration.reference_generation().to_string() != reference.generation_id()
            || artifact.profile_id() != library.profile().profile_id().as_str()
            || artifact.source_generation_id() != reference.generation_id()
            || map.annotation_artifact_id() != artifact.artifact_id()
            || map.profile_id() != artifact.profile_id()
            || map.source_generation_id() != reference.generation_id()
            || bindings.annotation_artifact_id() != artifact.artifact_id()
            || bindings.source_map_id() != map.source_map_id()
            || bindings.profile_id() != artifact.profile_id()
            || bindings.reference_generation() != reference.generation_id()
            || bindings.reference_digest() != reference.self_digest()
            || bindings.revision() != map.revision()
        {
            return Err(identity_error());
        }
        // Requalification admits exactly one rebuilt Library snapshot.
        // Check it before accepting targets rather than choosing a first Library.
        let analyzer = project.snapshot().analyzer_binding();
        let mut library_ids = analyzer.library_snapshot_ids();
        let library_id = library_ids.next().ok_or_else(identity_error)?;
        if library_ids.next().is_some() {
            return Err(identity_error());
        }
        let navigation = NavigationIndex::from_retained(
            library.annotation_files(),
            library.navigation_sources(),
            stop,
        )
        .map_err(navigation_error)?;
        let mut generated = BTreeMap::new();
        for file in library.annotation_files() {
            cancelled(stop)?;
            if generated.insert(file.path.as_str(), file).is_some() {
                return Err(identity_error());
            }
        }
        let mut files = BTreeMap::new();
        for file in map.files() {
            cancelled(stop)?;
            let original = generated.remove(file.path()).ok_or_else(identity_error)?;
            if original.sha256 != file.sha256() || original.text.len() as u64 != file.byte_length()
            {
                return Err(identity_error());
            }
            if files.insert(file.path(), (original, file)).is_some() {
                return Err(identity_error());
            }
        }
        if !generated.is_empty() {
            return Err(identity_error());
        }
        // SymbolTarget fields are public. Only native callable observations in
        // this held ProjectView can provide a target; matching text is insufficient.
        let mut targets = BTreeSet::new();
        if let Some(report) = analyzer.function_call_report() {
            if report.main_snapshot_id() != analyzer.main_workspace().snapshot_id()
                || report
                    .library_snapshot_ids()
                    .iter()
                    .map(String::as_str)
                    .ne([library_id])
            {
                return Err(identity_error());
            }
            for candidate in report
                .calls()
                .iter()
                .map(|call| call.target())
                .chain(report.named_targets().values())
            {
                cancelled(stop)?;
                if let SourceCallTarget::LibraryFunction { target } = candidate {
                    if target.role != "library" || target.workspace_id != library_id {
                        return Err(identity_error());
                    }
                    targets.insert(target);
                }
            }
        }
        cancelled(stop)?;
        Ok(Self {
            library,
            library_snapshot_id: library_id,
            navigation,
            files,
            targets,
        })
    }

    pub fn resolve_target(
        &self,
        target: &SymbolTarget,
        stop: &AtomicBool,
    ) -> ServiceResult<NativeApiTargetResolution<'a>> {
        cancelled(stop)?;
        if target.role != "library" {
            return Err(invalid_target());
        }
        if target.workspace_id != self.library_snapshot_id {
            return Err(identity_error());
        }
        let &(file, map_file) = self
            .files
            .get(target.path.as_str())
            .ok_or_else(identity_error)?;
        if file.sha256 != target.content_digest {
            return Err(identity_error());
        }
        target.span.validate().map_err(|_| invalid_target())?;
        if target.span.kind() != SourceSpanKind::ByteRange {
            return Err(invalid_target());
        }
        let start = target.span.byte_start().ok_or_else(invalid_target)?;
        let end = target.span.byte_end().ok_or_else(invalid_target)?;
        let start = usize::try_from(start).map_err(|_| invalid_target())?;
        let end = usize::try_from(end).map_err(|_| invalid_target())?;
        if start >= end || file.text.get(start..end).is_none() {
            return Err(invalid_target());
        }
        // Return the native report's immutable target, not the caller's public value.
        let Some(&target) = self.targets.get(target) else {
            cancelled(stop)?;
            return Ok(NativeApiTargetResolution::Unmapped(
                NativeApiTargetOmission::NotObservedInNativeCallableReport,
            ));
        };
        let lookup = self
            .navigation
            .source_for_range(
                &target.path,
                &target.content_digest,
                Span { start, end },
                stop,
            )
            .map_err(navigation_error)?;
        cancelled(stop)?;
        let SourceLookup::Mapped {
            precision,
            candidates,
        } = lookup
        else {
            return Ok(NativeApiTargetResolution::Unmapped(
                NativeApiTargetOmission::NoSourceMapping,
            ));
        };
        if candidates.len() > 1 {
            return Ok(NativeApiTargetResolution::AmbiguousMapping {
                precision,
                candidate_count: candidates.len(),
            });
        }
        let [location] = candidates.as_slice() else {
            return Err(identity_error());
        };
        if precision != MappingPrecision::Declaration || location.source.scope.is_some() {
            return Ok(NativeApiTargetResolution::UnsupportedMapping { precision });
        }
        if location.granularity != "declaration"
            || location.revision != self.library.callable_bindings().revision()
        {
            return Err(identity_error());
        }
        let mut selected = None;
        for mapping in map_file.mappings() {
            cancelled(stop)?;
            if same_mapping(mapping, location) && selected.replace(mapping).is_some() {
                return Err(identity_error());
            }
        }
        let mapping = selected.ok_or_else(identity_error)?;
        let bindings: &'a NativeCallableBindings = self.library.callable_bindings();
        let binding = match bindings.for_mapping(&target.path, &target.content_digest, mapping) {
            NativeCallableMapping::Unmatched => {
                return Ok(NativeApiTargetResolution::Unmapped(
                    NativeApiTargetOmission::NoNativeCallableOrigin,
                ));
            }
            NativeCallableMapping::AmbiguousOrigin => {
                return Ok(NativeApiTargetResolution::AmbiguousOrigin);
            }
            NativeCallableMapping::Bound(binding) => binding,
        };
        cancelled(stop)?;
        let reference: &'a ReferenceView = &self.library.reference().view;
        match reference.lookup(NATIVE_API_PARTITION, binding.key()) {
            LookupResult::Found(record) => {
                if binding.status() != NativeCallableReferenceStatus::Found
                    || record.kind() != ReferenceRecordKind::Api
                    || !record
                        .source_ids()
                        .iter()
                        .any(|id| id.as_ref() == binding.source_id())
                {
                    return Err(identity_error());
                }
                // Reuse the actual producer's finite reservation, not a new cap.
                let (digest, _, _) = record
                    .digest_bounded(bindings.metadata_bytes(), bindings.work(), stop)
                    .map_err(reference_error)?;
                if digest != binding.record_digest() {
                    return Err(identity_error());
                }
                cancelled(stop)?;
                Ok(NativeApiTargetResolution::Bound(
                    NativeApiTargetAssociation {
                        binding,
                        record,
                        target,
                    },
                ))
            }
            LookupResult::Conflict(conflict) => {
                if binding.status() != NativeCallableReferenceStatus::Conflict
                    || conflict.partition_id() != NATIVE_API_PARTITION
                    || conflict.key() != binding.key()
                    || !conflict
                        .candidate_digests()
                        .iter()
                        .any(|digest| digest.as_ref() == binding.record_digest())
                    || !conflict
                        .source_ids()
                        .iter()
                        .any(|id| id.as_ref() == binding.source_id())
                {
                    return Err(identity_error());
                }
                cancelled(stop)?;
                Ok(NativeApiTargetResolution::ReferenceConflict {
                    binding,
                    conflict,
                    target,
                })
            }
            // A sealed binding to this exact view must be Found or Conflict.
            // Unknown/absence here is a coherence error, never an API negative.
            LookupResult::Unknown(_) | LookupResult::AuthoritativeAbsence => Err(identity_error()),
        }
    }
}

fn same_mapping(mapping: &NativeSourceMapEntry, location: &SourceLocation<'_>) -> bool {
    mapping.granularity() == location.granularity
        && Some(mapping.generated().start()) == u64::try_from(location.generated.start).ok()
        && Some(mapping.generated().end()) == u64::try_from(location.generated.end).ok()
        && mapping.source().scope() == location.source.scope
        && mapping.source().path() == location.source.path
        && mapping.source().sha256() == location.source.sha256
        && Some(mapping.source().span().start()) == u64::try_from(location.source.span.start).ok()
        && Some(mapping.source().span().end()) == u64::try_from(location.source.span.end).ok()
}

fn navigation_error(error: LookupError) -> ServiceError {
    match error {
        LookupError::Cancelled => ServiceError::new(
            ServiceErrorCode::Cancelled,
            "native target navigation cancelled",
        ),
        LookupError::InputLimit => ServiceError::new(
            ServiceErrorCode::BudgetExceeded,
            "native target navigation exceeds its bound",
        ),
        _ => identity_error(),
    }
}
fn reference_error(error: ReferenceViewError) -> ServiceError {
    match error {
        ReferenceViewError::Cancelled => ServiceError::new(
            ServiceErrorCode::Cancelled,
            "native target Reference read cancelled",
        ),
        ReferenceViewError::LimitExceeded => ServiceError::new(
            ServiceErrorCode::BudgetExceeded,
            "native target Reference read exceeds its bound",
        ),
        _ => identity_error(),
    }
}
fn identity_error() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::IdentityMismatch,
        "native target identities differ",
    )
}
fn invalid_target() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::InvalidRequest,
        "native target requires a nonempty Library byte range",
    )
}

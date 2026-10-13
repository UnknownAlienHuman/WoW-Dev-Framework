//! Native declaration-origin bindings retained before the producer views move.
//! These are source observations, not accepted graph receipts or runtime facts.
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{CallableFieldLowering, CallableLoweringObservation, SourceSpan};

use wow_reference::native_view::{
    NATIVE_API_PARTITION, NATIVE_VIEW_AUTHORITY_PROFILE, NATIVE_VIEW_PROFILE, NativeViewProjection,
    callable_origin_id,
};
use wow_reference::{ReferenceRecord, ReferenceRecordKind, ReferenceViewError};
mod lowering;

use crate::sidecars::{
    NativeProjectionSidecars, NativeSidecarSource, NativeSidecarSpan, NativeSourceMapEntry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCallableBindingError {
    Cancelled,
    Limit,
    IncompatibleInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCallableReferenceStatus {
    Found,
    Conflict,
}

/// Association of an already navigation-selected mapping. This never performs
/// range selection or resolves competing original callable candidates.
pub enum NativeCallableMapping<'a> {
    Unmatched,
    AmbiguousOrigin,
    Bound(&'a NativeCallableBinding),
}

/// One exact generated declaration and its original native API candidate.
/// A Conflict retains that origin without selecting a Reference winner.
#[derive(Debug)]
pub struct NativeCallableBinding {
    generated_path: Box<str>,
    generated_sha256: Box<str>,
    generated: NativeSidecarSpan,
    mapping_id: Box<str>,
    source: NativeSidecarSource,
    source_id: Box<str>,
    key: Box<str>,
    record_digest: Box<str>,
    status: NativeCallableReferenceStatus,
    ambiguous_origin: bool,
    lowering: Option<Arc<[CallableFieldLowering]>>,
    record_digest_bytes: [u8; 32],
    record_bytes: usize,
    record_steps: usize,
}

impl NativeCallableBinding {
    #[must_use]
    pub fn generated_path(&self) -> &str {
        &self.generated_path
    }
    #[must_use]
    pub fn generated_sha256(&self) -> &str {
        &self.generated_sha256
    }
    #[must_use]
    pub const fn generated(&self) -> NativeSidecarSpan {
        self.generated
    }
    #[must_use]
    pub fn mapping_id(&self) -> &str {
        &self.mapping_id
    }
    #[must_use]
    pub const fn source(&self) -> &NativeSidecarSource {
        &self.source
    }
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
    #[must_use]
    pub fn record_digest(&self) -> &str {
        &self.record_digest
    }
    #[must_use]
    pub const fn status(&self) -> NativeCallableReferenceStatus {
        self.status
    }

    /// More than one native API candidate has this exact raw descriptor.
    #[must_use]
    pub const fn ambiguous_origin(&self) -> bool {
        self.ambiguous_origin
    }
}

/// Sealed, nonserialized join over one exact source map and native Reference view.
/// Unmatched declarations are not evidence of API absence. Target-range selection
/// remains the existing navigation owner's responsibility, including its ties.
#[derive(Debug)]
pub struct NativeCallableBindings {
    source_map_id: Box<str>,
    annotation_artifact_id: Box<str>,
    profile_id: Box<str>,
    reference_generation: Box<str>,
    reference_digest: Box<str>,
    revision: Box<str>,
    bindings: Box<[NativeCallableBinding]>,
    lowering_index: Box<[usize]>,
    lowering_metadata_bytes: usize,
    metadata_bytes: usize,
    work: usize,
}

struct Budget<'a> {
    bytes: usize,
    work: usize,
    max_bytes: usize,
    max_work: usize,
    stop: &'a AtomicBool,
}
impl Budget<'_> {
    fn charge(&mut self, bytes: usize) -> Result<(), NativeCallableBindingError> {
        self.consume(bytes, 1)
    }

    fn consume(&mut self, bytes: usize, steps: usize) -> Result<(), NativeCallableBindingError> {
        if self.stop.load(Ordering::Acquire) {
            return Err(NativeCallableBindingError::Cancelled);
        }
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(NativeCallableBindingError::Limit)?;
        self.work = self
            .work
            .checked_add(steps)
            .ok_or(NativeCallableBindingError::Limit)?;
        if self.bytes > self.max_bytes || self.work > self.max_work {
            return Err(NativeCallableBindingError::Limit);
        }
        Ok(())
    }
}

impl NativeCallableBindings {
    /// Capture the join from the genuine producer views. Limits cover temporary
    /// index reservations and retained descriptors; source/payload bytes are borrowed.
    pub fn build(
        sidecars: &NativeProjectionSidecars,
        reference: &NativeViewProjection,
        max_metadata_bytes: usize,
        max_work: usize,
        stop: &AtomicBool,
    ) -> Result<Self, NativeCallableBindingError> {
        let invalid = NativeCallableBindingError::IncompatibleInput;
        // Only the owner can mint these immutable, already validated sidecars.
        let map = sidecars.source_map();
        if max_metadata_bytes == 0
            || max_metadata_bytes > 64 * 1024 * 1024
            || max_work == 0
            || max_work > 1_000_000
        {
            return Err(NativeCallableBindingError::Limit);
        }
        let mut budget = Budget {
            bytes: 0,
            work: 0,
            max_bytes: max_metadata_bytes,
            max_work,
            stop,
        };
        budget.charge(0)?;
        let (bytes, steps) = reference
            .view
            .validate_bounded(
                max_metadata_bytes - budget.bytes,
                max_work - budget.work,
                stop,
            )
            .map_err(binding_error)?;
        budget.consume(bytes, steps)?;
        budget.charge(0)?;
        if map.source_generation_id() != reference.view.generation_id()
            || !matches!(
                reference.schema,
                NATIVE_VIEW_PROFILE | NATIVE_VIEW_AUTHORITY_PROFILE
            )
        {
            return Err(invalid);
        }
        let mut sources = BTreeMap::new();
        for source in &reference.sources {
            // Conservative fixed reservation includes the borrowed index entry
            // and tree bookkeeping. It is not an OS allocation measurement.
            budget.charge(256)?;
            if sources
                .insert(source.id.as_str(), source)
                .is_some_and(|previous| previous != source)
            {
                return Err(invalid);
            }
        }
        let mut origins = BTreeMap::new();
        let mut admitted = BTreeMap::new();
        let mut conflicts = BTreeMap::new();
        if let Some(partition) = reference
            .view
            .partitions()
            .iter()
            .find(|partition| partition.id() == NATIVE_API_PARTITION)
        {
            for record in partition.records() {
                budget.charge(256)?;
                admitted.insert(record.key(), record);
            }
        }
        for conflict in reference.view.conflicts() {
            budget.charge(0)?;
            if conflict.partition_id() == NATIVE_API_PARTITION {
                budget.charge(256)?;
                conflicts.insert(conflict.key(), conflict);
            }
        }
        for record in &reference.candidates {
            budget.charge(0)?;
            if record.kind() != ReferenceRecordKind::Api {
                continue;
            }
            let [id] = record.source_ids() else {
                return Err(invalid);
            };
            let source = sources.get(id.as_ref()).ok_or(invalid)?;
            budget.charge(
                6 * (source.path.len()
                    + source.sha256.len()
                    + map.revision().len()
                    + reference.schema.len())
                    + 256,
            )?;
            let expected = callable_origin_id(
                reference.schema,
                map.revision(),
                &source.path,
                &source.sha256,
                source.span,
            )
            .map_err(|_| invalid)?;
            if expected != id.as_ref() {
                return Err(invalid);
            }
            let (digest, bytes, steps) = record
                .digest_bounded(
                    max_metadata_bytes - budget.bytes,
                    max_work - budget.work,
                    stop,
                )
                .map_err(binding_error)?;
            budget.consume(bytes, steps)?;
            let status = if let Some(conflict) = conflicts.get(record.key()) {
                if conflict
                    .candidate_digests()
                    .binary_search_by(|item| item.as_ref().cmp(&digest))
                    .is_err()
                    || conflict.source_ids().binary_search(id).is_err()
                {
                    return Err(invalid);
                }
                NativeCallableReferenceStatus::Conflict
            } else if admitted
                .get(record.key())
                .is_some_and(|found| **found == *record)
            {
                NativeCallableReferenceStatus::Found
            } else {
                return Err(invalid);
            };
            budget.charge(256 + digest.len())?;
            let key = (
                source.path.as_str(),
                source.sha256.as_str(),
                source.span.start,
                source.span.end,
            );
            origins
                .entry(key)
                .or_insert_with(Vec::new)
                .push((record, digest, status, bytes, steps));
        }
        let mut captured = BTreeMap::new();
        let mut lowering_metadata_bytes = 0usize;
        for (row, file) in sidecars.callable_lowerings() {
            budget.consume(
                256,
                row.fields
                    .len()
                    .checked_add(1)
                    .ok_or(NativeCallableBindingError::Limit)?,
            )?;
            let start = row.declaration_span.byte_start().ok_or(invalid)?;
            let end = row.declaration_span.byte_end().ok_or(invalid)?;
            if captured.insert((file.path(), start, end), row).is_some() {
                return Err(invalid);
            }
        }
        let mut bindings = Vec::new();
        for file in map.files() {
            for mapping in file.mappings() {
                budget.charge(0)?;
                if mapping.granularity() != "declaration" || mapping.source().scope().is_some() {
                    continue;
                }
                let source = mapping.source();
                let span = source.span();
                let key = (
                    source.path(),
                    source.sha256(),
                    usize::try_from(span.start()).map_err(|_| invalid)?,
                    usize::try_from(span.end()).map_err(|_| invalid)?,
                );
                let Some(candidates) = origins.get(&key) else {
                    continue;
                };
                for (record, digest, status, record_bytes, record_steps) in candidates {
                    let id = &record.source_ids()[0];
                    budget.charge(
                        2 * std::mem::size_of::<NativeCallableBinding>()
                            + file.path().len()
                            + file.sha256().len()
                            + mapping.mapping_id().len()
                            + source.path().len()
                            + source.sha256().len()
                            + id.len()
                            + record.key().len()
                            + digest.len(),
                    )?;
                    let lowering = if *status == NativeCallableReferenceStatus::Found
                        && candidates.len() == 1
                    {
                        captured
                            .get(&(
                                file.path(),
                                mapping.generated().start(),
                                mapping.generated().end(),
                            ))
                            .map(|row| {
                                if row.source_path.as_ref() != source.path()
                                    || row.source_sha256.as_ref() != source.sha256()
                                    || row.source_span.byte_start() != Some(source.span().start())
                                    || row.source_span.byte_end() != Some(source.span().end())
                                {
                                    return Err(invalid);
                                }
                                // The native report has public fields; qualify this new lookup
                                // against the Reference owner's sealed original signature catalog.
                                let scratch = record_bytes
                                    .checked_mul(4)
                                    .ok_or(NativeCallableBindingError::Limit)?;
                                budget.consume(
                                    scratch,
                                    scratch
                                        .checked_add(*record_steps)
                                        .ok_or(NativeCallableBindingError::Limit)?,
                                )?;
                                let signature = reference
                                    .callable_signature(record, stop)
                                    .map_err(|error| match error.code {
                                        wow_reference::native::NativeErrorCode::Cancelled => {
                                            NativeCallableBindingError::Cancelled
                                        }
                                        wow_reference::native::NativeErrorCode::Limit => {
                                            NativeCallableBindingError::Limit
                                        }
                                        _ => invalid,
                                    })?;
                                if signature.reference_generation()
                                    != reference.view.generation_id()
                                    || signature.reference_view_digest()
                                        != reference.view.self_digest()
                                    || signature.record_key() != record.key()
                                    || signature.record_digest() != digest
                                    || signature.source_id() != id.as_ref()
                                {
                                    return Err(invalid);
                                }
                                // This Arc shares the field allocation owned and charged by
                                // the source library/sidecar path. Count only this retained
                                // handle as a subset of the already reserved binding record;
                                // do not charge field tokens or spans twice.
                                let retained_handle =
                                    std::mem::size_of::<Arc<[CallableFieldLowering]>>();
                                lowering_metadata_bytes = lowering_metadata_bytes
                                    .checked_add(retained_handle)
                                    .ok_or(NativeCallableBindingError::Limit)?;
                                Ok(Arc::clone(&row.fields))
                            })
                            .transpose()?
                    } else {
                        None
                    };
                    bindings.push(NativeCallableBinding {
                        generated_path: file.path().into(),
                        generated_sha256: file.sha256().into(),
                        generated: mapping.generated(),
                        mapping_id: mapping.mapping_id().into(),
                        source: source.clone(),
                        source_id: id.clone(),
                        key: record.key().into(),
                        record_digest: digest.clone().into_boxed_str(),
                        status: *status,
                        ambiguous_origin: candidates.len() > 1,
                        lowering,
                        record_digest_bytes: lowering::digest_bytes(digest)?,
                        record_bytes: *record_bytes,
                        record_steps: *record_steps,
                    });
                }
            }
        }
        let count = bindings
            .iter()
            .filter(|binding| binding.lowering.is_some())
            .count();
        let bytes = count
            .checked_mul(2 * std::mem::size_of::<usize>())
            .ok_or(NativeCallableBindingError::Limit)?;
        let steps = count
            .checked_mul(count.max(1).ilog2() as usize + 2)
            .and_then(|steps| steps.checked_add(bindings.len()))
            .ok_or(NativeCallableBindingError::Limit)?;
        budget.consume(bytes, steps)?;
        lowering_metadata_bytes = lowering_metadata_bytes
            .checked_add(bytes)
            .ok_or(NativeCallableBindingError::Limit)?;
        let mut lowering_index = Vec::with_capacity(count);
        for (index, binding) in bindings.iter().enumerate() {
            budget.charge(0)?;
            if binding.lowering.is_some() {
                lowering_index.push(index);
            }
        }
        lowering_index.sort_by(|left, right| bindings[*left].key().cmp(bindings[*right].key()));
        budget.charge(
            std::mem::size_of::<Self>()
                + map.source_map_id().len()
                + map.annotation_artifact_id().len()
                + map.profile_id().len()
                + reference.view.generation_id().len()
                + reference.view.self_digest().len()
                + map.revision().len(),
        )?;
        Ok(Self {
            source_map_id: map.source_map_id().into(),
            annotation_artifact_id: map.annotation_artifact_id().into(),
            profile_id: map.profile_id().into(),
            reference_generation: reference.view.generation_id().into(),
            reference_digest: reference.view.self_digest().into(),
            revision: map.revision().into(),
            bindings: bindings.into_boxed_slice(),
            lowering_index: lowering_index.into_boxed_slice(),
            lowering_metadata_bytes,
            metadata_bytes: budget.bytes,
            work: budget.work,
        })
    }

    #[must_use]
    pub fn source_map_id(&self) -> &str {
        &self.source_map_id
    }
    #[must_use]
    pub fn annotation_artifact_id(&self) -> &str {
        &self.annotation_artifact_id
    }
    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }
    #[must_use]
    pub fn reference_generation(&self) -> &str {
        &self.reference_generation
    }
    #[must_use]
    pub fn reference_digest(&self) -> &str {
        &self.reference_digest
    }
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }
    #[must_use]
    pub fn bindings(&self) -> &[NativeCallableBinding] {
        &self.bindings
    }

    /// Retained lowering subset; already included in metadata_bytes(), do not add it twice.
    pub const fn callable_lowering_metadata_bytes(&self) -> usize {
        self.lowering_metadata_bytes
    }

    /// Unique Found/nonambiguous original record only. Tokens and source remain borrowed;
    /// Core family admission may use bounded identity scratch on a successful read.
    /// Missing custody is None; a same-key mismatch or duplicate generated binding is an error.
    pub fn callable_lowering(
        &self,
        record: &ReferenceRecord,
        stop: &AtomicBool,
    ) -> Result<Option<CallableLoweringObservation<'_>>, NativeCallableBindingError> {
        if stop.load(Ordering::Acquire) {
            return Err(NativeCallableBindingError::Cancelled);
        }
        let invalid = NativeCallableBindingError::IncompatibleInput;
        if record.kind() != ReferenceRecordKind::Api
            || record.key().len() > 1024
            || record.payload().len() > 65_536
        {
            return Err(invalid);
        }
        let index = self
            .lowering_index
            .partition_point(|index| self.bindings[*index].key() < record.key());
        let Some(&binding_index) = self.lowering_index.get(index) else {
            return Ok(None);
        };
        let binding = &self.bindings[binding_index];
        if binding.key() != record.key() {
            return Ok(None);
        }
        if self
            .lowering_index
            .get(index + 1)
            .is_some_and(|index| self.bindings[*index].key() == record.key())
            || binding.status != NativeCallableReferenceStatus::Found
            || binding.ambiguous_origin
            || record.payload().len() > binding.record_bytes
            || record.source_ids().len() != 1
            || record.source_ids()[0].as_ref() != binding.source_id()
        {
            return Err(invalid);
        }
        lowering::verify_record(
            record,
            &binding.record_digest_bytes,
            binding.record_bytes,
            binding.record_steps,
            stop,
        )?;
        let fields = binding.lowering.as_deref().ok_or(invalid)?;
        let declaration =
            SourceSpan::byte_range(binding.generated.start(), binding.generated.end())
                .map_err(|_| invalid)?;
        let result = CallableLoweringObservation::admit(
            &self.profile_id,
            &self.reference_generation,
            &self.reference_digest,
            binding.key(),
            binding.record_digest(),
            binding.source_id(),
            binding.source.path(),
            binding.source.sha256(),
            &self.annotation_artifact_id,
            &self.source_map_id,
            binding.generated_path(),
            binding.generated_sha256(),
            declaration,
            fields,
        )
        .map_err(|_| invalid)?;
        if stop.load(Ordering::Acquire) {
            return Err(NativeCallableBindingError::Cancelled);
        }
        Ok(Some(result))
    }

    /// Resolve an already selected declaration mapping into its unique native binding.
    /// No range selection occurs here; ambiguous origins remain explicit.
    /// A conflict binding retains its status and never selects a Reference winner.
    #[must_use]
    pub fn for_mapping(
        &self,
        generated_path: &str,
        generated_sha256: &str,
        mapping: &NativeSourceMapEntry,
    ) -> NativeCallableMapping<'_> {
        if mapping.granularity() != "declaration" || mapping.source().scope().is_some() {
            return NativeCallableMapping::Unmatched;
        }
        // The sealed map orders files by path and mappings by mapping_id. Build
        // preserves that order, with origin candidates contiguous per mapping.
        let key = (generated_path, mapping.mapping_id());
        let index = self
            .bindings
            .partition_point(|binding| (binding.generated_path(), binding.mapping_id()) < key);
        let Some(binding) = self.bindings.get(index) else {
            return NativeCallableMapping::Unmatched;
        };
        if binding.generated_path() != generated_path
            || binding.generated_sha256() != generated_sha256
            || binding.mapping_id() != mapping.mapping_id()
            || binding.generated() != mapping.generated()
            || binding.source() != mapping.source()
        {
            return NativeCallableMapping::Unmatched;
        }
        if binding.ambiguous_origin() {
            NativeCallableMapping::AmbiguousOrigin
        } else {
            NativeCallableMapping::Bound(binding)
        }
    }

    /// Cumulative conservative reservation, including temporary indexes.
    #[must_use]
    pub const fn metadata_bytes(&self) -> usize {
        self.metadata_bytes
    }
    #[must_use]
    pub const fn work(&self) -> usize {
        self.work
    }
}

fn binding_error(error: ReferenceViewError) -> NativeCallableBindingError {
    match error {
        ReferenceViewError::Cancelled => NativeCallableBindingError::Cancelled,
        ReferenceViewError::LimitExceeded => NativeCallableBindingError::Limit,
        _ => NativeCallableBindingError::IncompatibleInput,
    }
}

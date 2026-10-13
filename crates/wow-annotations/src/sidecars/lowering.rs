//! Validate private emission custody against final bytes and the existing immutable map.
use super::{NativeSidecarError, NativeSidecarErrorCode, NativeSidecarResult, NativeSourceMap};
use crate::native::{CapturedCallable, NativeLibrary};
use std::collections::BTreeMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wow_core::{CallableLoweringLane, SourceSpan};

type ValidatedLowering = (Arc<[CapturedCallable]>, Box<[usize]>);

pub(super) fn checkpoint(stop: &AtomicBool) -> NativeSidecarResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(NativeSidecarError::new(
            NativeSidecarErrorCode::Cancelled,
            "native lowering custody cancelled",
        ))
    } else {
        Ok(())
    }
}

fn invalid() -> NativeSidecarError {
    NativeSidecarError::new(
        NativeSidecarErrorCode::InvalidSourceMap,
        "captured lowering does not match final bytes and maps",
    )
}
fn limit() -> NativeSidecarError {
    NativeSidecarError::new(
        NativeSidecarErrorCode::InputLimit,
        "captured lowering custody exceeds its finite allowance",
    )
}

pub(super) fn preflight(library: &NativeLibrary<'_>, stop: &AtomicBool) -> NativeSidecarResult<()> {
    checkpoint(stop)?;
    if library.files.len() > 4096 || library.lowering_catalog.len() > 65_536 {
        return Err(limit());
    }
    let mut bytes = 0usize;
    let mut mappings = 0usize;
    for file in &library.files {
        checkpoint(stop)?;
        bytes = bytes.checked_add(file.text.len()).ok_or_else(limit)?;
        mappings = mappings
            .checked_add(file.mappings.len())
            .ok_or_else(limit)?;
        if file.text.len() > 8 * 1024 * 1024 || bytes > 64 * 1024 * 1024 || mappings > 262_144 {
            return Err(limit());
        }
    }
    Ok(())
}

pub(super) fn validate(
    library: &NativeLibrary<'_>,
    map: &NativeSourceMap,
    stop: &AtomicBool,
) -> NativeSidecarResult<ValidatedLowering> {
    checkpoint(stop)?;
    let index_bytes = library
        .lowering_catalog
        .len()
        .checked_mul(2 * std::mem::size_of::<usize>())
        .ok_or_else(limit)?;
    if index_bytes > 64 * 1024 * 1024 {
        return Err(limit());
    }
    let mut file_indices = Vec::with_capacity(library.lowering_catalog.len());
    let mut mappings = BTreeMap::new();
    let mut previous_file = None;
    let mut map_file_index = 0usize;
    let mut previous_end = 0u64;
    let mut fields = 0usize;
    let mut bytes = index_bytes;
    let mut steps = 0usize;
    for row in library.lowering_catalog.iter() {
        checkpoint(stop)?;
        let file = library.files.get(row.file_index).ok_or_else(invalid)?;
        if previous_file.is_some_and(|index| row.file_index < index) {
            return Err(invalid());
        }
        if previous_file != Some(row.file_index) {
            mappings.clear();
            previous_end = 0;
            previous_file = Some(row.file_index);
            let index = map
                .files()
                .binary_search_by(|value| value.path().cmp(&file.path))
                .map_err(|_| invalid())?;
            map_file_index = index;
            let mapped = &map.files()[index];
            if mapped.sha256() != file.sha256 || mapped.byte_length() != file.text.len() as u64 {
                return Err(invalid());
            }
            for entry in mapped.mappings() {
                checkpoint(stop)?;
                // Reserve temporary tree bookkeeping before insertion; reservations never reset.
                bytes = bytes.checked_add(256).ok_or_else(limit)?;
                steps = steps.checked_add(1).ok_or_else(limit)?;
                if bytes > 64 * 1024 * 1024 || steps > 64 * 1024 * 1024 {
                    return Err(limit());
                }
                let key = (
                    entry.generated().start(),
                    entry.generated().end(),
                    entry.granularity(),
                );
                if mappings.insert(key, entry).is_some() {
                    return Err(invalid());
                }
            }
        }
        file_indices.push(map_file_index);
        let (start, end) = range(row.declaration_span)?;
        if start < previous_end || end > file.text.len() as u64 {
            return Err(invalid());
        }
        previous_end = end;
        let declaration = mappings
            .get(&(start, end, "declaration"))
            .ok_or_else(invalid)?;
        if declaration.source().scope().is_some()
            || declaration.source().path() != row.source_path.as_ref()
            || declaration.source().sha256() != row.source_sha256.as_ref()
            || (
                declaration.source().span().start(),
                declaration.source().span().end(),
            ) != range(row.source_span)?
        {
            return Err(invalid());
        }
        let mut previous_member_end = start;
        let mut parameters = 0;
        let mut returns = 0;
        fields = fields.checked_add(row.fields.len()).ok_or_else(limit)?;
        if fields > 65_536 {
            return Err(limit());
        }
        for field in row.fields.iter() {
            checkpoint(stop)?;
            bytes = bytes
                .checked_add(field.lowered_type().len())
                .ok_or_else(limit)?;
            steps = steps
                .checked_add(field.lowered_type().len().saturating_add(1))
                .ok_or_else(limit)?;
            if bytes > 64 * 1024 * 1024 || steps > 64 * 1024 * 1024 {
                return Err(limit());
            }
            let (next, granularity) = match field.lane() {
                CallableLoweringLane::Parameter if returns == 0 => (&mut parameters, "parameter"),
                CallableLoweringLane::Return => (&mut returns, "return"),
                _ => return Err(invalid()),
            };
            let (member_start, member_end) = range(field.generated_member_span())?;
            let (token_start, token_end) = range(field.type_token_span())?;
            let member = mappings
                .get(&(member_start, member_end, granularity))
                .ok_or_else(invalid)?;
            if field.ordinal() != *next
                || member_start < previous_member_end
                || member_end > end
                || token_start < member_start
                || token_end > member_end
                || member.source().scope().is_some()
                || member.source().path() != row.source_path.as_ref()
                || member.source().sha256() != row.source_sha256.as_ref()
                || (member.source().span().start(), member.source().span().end())
                    != range(field.original_source_span())?
                || file.text.get(
                    usize::try_from(token_start).map_err(|_| invalid())?
                        ..usize::try_from(token_end).map_err(|_| invalid())?,
                ) != Some(field.lowered_type())
            {
                return Err(invalid());
            }
            *next += 1;
            previous_member_end = member_end;
        }
    }
    checkpoint(stop)?;
    // Shared immutable metadata only; no second token/source/file-byte mirror.
    Ok((
        Arc::clone(&library.lowering_catalog),
        file_indices.into_boxed_slice(),
    ))
}

fn range(span: SourceSpan) -> NativeSidecarResult<(u64, u64)> {
    span.validate().map_err(|_| invalid())?;
    match (span.byte_start(), span.byte_end()) {
        (Some(start), Some(end)) if start < end => Ok((start, end)),
        _ => Err(invalid()),
    }
}

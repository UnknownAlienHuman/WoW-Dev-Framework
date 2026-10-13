//! Validate private emission custody against final bytes and the existing immutable map.
use super::{NativeSidecarError, NativeSidecarErrorCode, NativeSidecarResult, NativeSourceMap};
use crate::native::{CapturedCallable, NativeLibrary};
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wow_core::{CallableLoweringLane, ContentDigest, SourceContent, SourceSpan};

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
            let mut previous_atom_end = token_start;
            for atom in field.type_atoms() {
                checkpoint(stop)?;
                let (atom_start, atom_end) = range(atom.generated_span())?;
                bytes = bytes
                    .checked_add(atom.source_name().len())
                    .and_then(|n| n.checked_add(atom.rendered_name().len()))
                    .ok_or_else(limit)?;
                steps = steps
                    .checked_add(
                        atom.source_name()
                            .len()
                            .saturating_add(atom.rendered_name().len())
                            .saturating_add(1),
                    )
                    .ok_or_else(limit)?;
                if atom.source_name().is_empty()
                    || atom.rendered_name().is_empty()
                    || atom_start < previous_atom_end
                    || atom_end > token_end
                    || file.text.get(
                        usize::try_from(atom_start).map_err(|_| invalid())?
                            ..usize::try_from(atom_end).map_err(|_| invalid())?,
                    ) != Some(atom.rendered_name())
                    || bytes > 64 * 1024 * 1024
                    || steps > 64 * 1024 * 1024
                {
                    return Err(invalid());
                }
                previous_atom_end = atom_end;
            }
            *next += 1;
            previous_member_end = member_end;
        }
    }
    let mut validated_alias_catalogs = BTreeSet::new();
    for row in library.lowering_catalog.iter() {
        checkpoint(stop)?;
        if row.alias_definitions.is_empty() {
            continue;
        }
        let identity = row.alias_definitions.as_ptr() as usize;
        bytes = bytes
            .checked_add(3 * std::mem::size_of::<usize>())
            .ok_or_else(limit)?;
        steps = steps.checked_add(1).ok_or_else(limit)?;
        if bytes > 64 * 1024 * 1024 || steps > 64 * 1024 * 1024 {
            return Err(limit());
        }
        if validated_alias_catalogs.insert(identity) {
            validate_alias_definitions(
                row.alias_definitions.as_ref(),
                &row.fields,
                library,
                map,
                &mut bytes,
                &mut steps,
                stop,
            )?;
        }
    }
    checkpoint(stop)?;
    // Shared immutable metadata only; no second token/source/file-byte mirror.
    Ok((
        Arc::clone(&library.lowering_catalog),
        file_indices.into_boxed_slice(),
    ))
}

fn validate_alias_definitions(
    definitions: &[wow_core::CallableAliasDefinition],
    fields: &[wow_core::CallableFieldLowering],
    library: &NativeLibrary<'_>,
    map: &NativeSourceMap,
    bytes: &mut usize,
    steps: &mut usize,
    stop: &AtomicBool,
) -> NativeSidecarResult<()> {
    if definitions.is_empty() || definitions.len() > 4096 {
        return Err(invalid());
    }
    let first = &definitions[0];
    let mut by_name = BTreeMap::new();
    let mut previous_name = None;
    for (index, definition) in definitions.iter().enumerate() {
        checkpoint(stop)?;
        let identity_bytes = definition
            .name()
            .len()
            .checked_add(definition.source_revision().len())
            .and_then(|n| n.checked_add(definition.source_path().len()))
            .and_then(|n| n.checked_add(definition.generated_path().len()))
            .and_then(|n| n.checked_add(definition.lowered_definition().len()))
            .and_then(|n| n.checked_add(64))
            .ok_or_else(limit)?;
        let dependency_bytes = definition
            .dependency_names()
            .iter()
            .try_fold(
                definition
                    .dependency_names()
                    .len()
                    .checked_mul(std::mem::size_of::<String>())
                    .ok_or_else(limit)?,
                |total, name| total.checked_add(name.len()),
            )
            .ok_or_else(limit)?;
        let retained_identity_bytes = definition
            .name()
            .len()
            .checked_add(definition.source_revision().len())
            .and_then(|n| n.checked_add(definition.source_path().len()))
            .and_then(|n| n.checked_add(definition.generated_path().len()))
            .and_then(|n| n.checked_add(definition.lowered_definition().len()))
            .ok_or_else(limit)?;
        *bytes = bytes
            .checked_add(retained_identity_bytes)
            .and_then(|n| n.checked_add(2 * 32))
            .and_then(|n| n.checked_add(64))
            .and_then(|n| n.checked_add(dependency_bytes))
            .and_then(|n| n.checked_add(std::mem::size_of::<wow_core::CallableAliasDefinition>()))
            .ok_or_else(limit)?;
        *steps = steps
            .checked_add(
                identity_bytes
                    .saturating_add(dependency_bytes)
                    .saturating_add(1),
            )
            .ok_or_else(limit)?;
        if *bytes > 64 * 1024 * 1024 || *steps > 64 * 1024 * 1024 {
            return Err(limit());
        }
        if definition.name().is_empty()
            || previous_name.is_some_and(|name: &str| name >= definition.name())
            || definition.generated_path() != first.generated_path()
            || definition.generated_sha256() != first.generated_sha256()
            || by_name.insert(definition.name(), index).is_some()
        {
            return Err(invalid());
        }
        previous_name = Some(definition.name());

        let aliases = library.aliases.as_ref().ok_or_else(invalid)?;
        let documents =
            std::iter::once(aliases.source).chain(aliases.additional_sources.iter().copied());
        let source_digest = *definition.source_sha256();
        let generated_digest = *definition.generated_sha256();
        let source_document = documents
            .into_iter()
            .find(|document| {
                document.revision() == definition.source_revision()
                    && document.path() == definition.source_path()
                    && ContentDigest::<SourceContent>::from_str(document.sha256())
                        .is_ok_and(|digest| digest == source_digest)
            })
            .ok_or_else(invalid)?;
        let source_span = range(definition.source_span())?;
        if source_span.0 >= source_span.1
            || source_span.1 > source_document.text().len() as u64
            || source_document
                .text()
                .get(
                    usize::try_from(source_span.0).map_err(|_| invalid())?
                        ..usize::try_from(source_span.1).map_err(|_| invalid())?,
                )
                .is_none()
        {
            return Err(invalid());
        }

        let library_file = library
            .files
            .iter()
            .find(|file| {
                file.path == definition.generated_path()
                    && ContentDigest::<SourceContent>::from_str(&file.sha256)
                        .is_ok_and(|digest| digest == generated_digest)
            })
            .ok_or_else(invalid)?;
        let map_index = map
            .files()
            .binary_search_by(|file| file.path().cmp(definition.generated_path()))
            .map_err(|_| invalid())?;
        let map_file = &map.files()[map_index];
        if ContentDigest::<SourceContent>::from_str(map_file.sha256()).map_err(|_| invalid())?
            != generated_digest
            || map_file.byte_length() != library_file.text.len() as u64
        {
            return Err(invalid());
        }
        let (decl_start, decl_end) = range(definition.declaration_span())?;
        let (type_start, type_end) = range(definition.type_token_span())?;
        let mapping = map_file
            .mappings()
            .iter()
            .find(|entry| {
                entry.granularity() == "declaration"
                    && (entry.generated().start(), entry.generated().end())
                        == (decl_start, decl_end)
            })
            .ok_or_else(invalid)?;
        let source = mapping.source();
        let declaration = library_file
            .text
            .get(
                usize::try_from(decl_start).map_err(|_| invalid())?
                    ..usize::try_from(decl_end).map_err(|_| invalid())?,
            )
            .ok_or_else(invalid)?;
        let type_bytes = "---@alias "
            .len()
            .checked_add(definition.name().len())
            .and_then(|n| n.checked_add(1))
            .ok_or_else(limit)?;
        if source.scope() != Some("annotation_alias_catalog")
            || source.path() != definition.source_path()
            || ContentDigest::<SourceContent>::from_str(source.sha256()).map_err(|_| invalid())?
                != source_digest
            || (source.span().start(), source.span().end()) != source_span
            || type_start < decl_start
            || type_end > decl_end
            || library_file.text.get(
                usize::try_from(type_start).map_err(|_| invalid())?
                    ..usize::try_from(type_end).map_err(|_| invalid())?,
            ) != Some(definition.lowered_definition())
            || declaration.len()
                != type_bytes
                    .checked_add(definition.lowered_definition().len())
                    .ok_or_else(limit)?
            || declaration.get(.."---@alias ".len()) != Some("---@alias ")
            || declaration.get("---@alias ".len()..type_bytes - 1) != Some(definition.name())
            || declaration.get(type_bytes - 1..type_bytes) != Some(" ")
            || declaration.get(type_bytes..) != Some(definition.lowered_definition())
        {
            return Err(invalid());
        }
    }
    for definition in definitions {
        checkpoint(stop)?;
        let mut previous_dependency = None;
        for dependency in definition.dependency_names() {
            *steps = steps.checked_add(1).ok_or_else(limit)?;
            if *steps > 64 * 1024 * 1024
                || previous_dependency.is_some_and(|name: &str| name >= dependency.as_ref())
                || !by_name.contains_key(dependency.as_ref())
            {
                return Err(invalid());
            }
            previous_dependency = Some(dependency.as_ref());
        }
    }

    // The retained definitions must be exactly the finite alias closure rooted
    // in the atoms captured by this callable's one renderer pass.
    let scratch_bytes = definitions
        .len()
        .checked_mul(4 * std::mem::size_of::<usize>() + 64)
        .ok_or_else(limit)?;
    *bytes = bytes.checked_add(scratch_bytes).ok_or_else(limit)?;
    if *bytes > 64 * 1024 * 1024 {
        return Err(limit());
    }
    let mut closure = BTreeSet::new();
    let mut frontier = Vec::with_capacity(definitions.len());
    for field in fields {
        checkpoint(stop)?;
        for atom in field.type_atoms() {
            *steps = steps.checked_add(1).ok_or_else(limit)?;
            if *steps > 64 * 1024 * 1024 {
                return Err(limit());
            }
            let name = atom.rendered_name();
            if by_name.contains_key(name) && closure.insert(name) {
                frontier.push(name);
            }
        }
    }
    let mut cursor = 0;
    while cursor < frontier.len() {
        checkpoint(stop)?;
        let name = frontier[cursor];
        cursor += 1;
        let index = *by_name.get(name).ok_or_else(invalid)?;
        for dependency in definitions[index].dependency_names() {
            *steps = steps.checked_add(1).ok_or_else(limit)?;
            if *steps > 64 * 1024 * 1024 {
                return Err(limit());
            }
            let dependency = dependency.as_ref();
            if !by_name.contains_key(dependency) {
                return Err(invalid());
            }
            if closure.insert(dependency) {
                frontier.push(dependency);
            }
        }
    }
    if closure.len() != definitions.len()
        || definitions
            .iter()
            .any(|definition| !closure.contains(definition.name()))
    {
        return Err(invalid());
    }
    Ok(())
}

fn range(span: SourceSpan) -> NativeSidecarResult<(u64, u64)> {
    span.validate().map_err(|_| invalid())?;
    match (span.byte_start(), span.byte_end()) {
        (Some(start), Some(end)) if start < end => Ok((start, end)),
        _ => Err(invalid()),
    }
}

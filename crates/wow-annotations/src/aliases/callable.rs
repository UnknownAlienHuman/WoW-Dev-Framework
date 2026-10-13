//! Private handoff from the existing alias projection graph to selected callables.
use super::RenderError;
use crate::native::CapturedCallable;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wow_core::CallableAliasDefinition;
use wow_reference::native::Span;
use wow_reference::native_aliases::AliasDocument;

const MAX_ALIAS_DEFINITIONS: usize = 4096;
const MAX_ALIAS_LINK_STEPS: usize = 16 * 1024 * 1024;

pub(crate) struct PendingDefinition<'a> {
    pub(super) source: &'a AliasDocument,
    pub(super) name: Box<str>,
    pub(super) source_span: Span,
    pub(super) declaration_span: Span,
    pub(super) type_token_span: Span,
    pub(super) lowered_definition: Box<str>,
    pub(super) dependency_names: Box<[Box<str>]>,
}

/// Bind only exact alias atoms observed by the existing emitter. Equal closures
/// share one immutable slice; no per-field alias text or generated-file copy is
/// retained. Every retained clone is charged before allocation.
pub(crate) fn attach(
    callables: &mut [CapturedCallable],
    pending: &[PendingDefinition<'_>],
    generated_path: &str,
    generated_sha256: &str,
    budget: &mut super::super::native::lowering::CaptureBudget,
    stop: &AtomicBool,
) -> Result<(), RenderError> {
    if pending.len() > MAX_ALIAS_DEFINITIONS
        || generated_path.is_empty()
        || generated_sha256.is_empty()
    {
        return Err(RenderError::InputLimit);
    }
    let mut by_name = BTreeMap::new();
    budget.reserve(
        0,
        0,
        pending
            .len()
            .checked_mul(2 * std::mem::size_of::<usize>() + 96)
            .ok_or(RenderError::OutputLimit)?,
        stop,
    )?;
    for (index, definition) in pending.iter().enumerate() {
        checkpoint(stop)?;
        if by_name.insert(definition.name.as_ref(), index).is_some() {
            return Err(RenderError::InvalidSource);
        }
    }

    let mut interned = BTreeMap::<Vec<usize>, Arc<[CallableAliasDefinition]>>::new();
    let mut link_steps = 0usize;
    for callable in callables {
        checkpoint(stop)?;
        let mut selected = BTreeSet::new();
        let mut frontier = Vec::new();
        for field in callable.fields.iter() {
            for atom in field.type_atoms() {
                let name = atom.rendered_name();
                step(&mut link_steps, budget, stop)?;
                if let Some(&index) = by_name.get(name)
                    && !selected.contains(&index)
                {
                    budget.reserve(0, 0, std::mem::size_of::<usize>() * 4 + 64, stop)?;
                    selected.insert(index);
                    frontier.push(index);
                }
            }
        }
        let mut cursor = 0;
        while cursor < frontier.len() {
            checkpoint(stop)?;
            let index = frontier[cursor];
            cursor += 1;
            for dependency in pending[index].dependency_names.iter() {
                step(&mut link_steps, budget, stop)?;
                let Some(&target) = by_name.get(dependency.as_ref()) else {
                    return Err(RenderError::InvalidSource);
                };
                if !selected.contains(&target) {
                    budget.reserve(0, 0, std::mem::size_of::<usize>() * 4 + 64, stop)?;
                    selected.insert(target);
                    frontier.push(target);
                }
            }
        }
        if selected.is_empty() {
            callable.alias_definitions = Arc::from([]);
            continue;
        }
        budget.reserve(
            0,
            0,
            selected
                .len()
                .checked_mul(std::mem::size_of::<usize>())
                .ok_or(RenderError::OutputLimit)?,
            stop,
        )?;
        let key = selected.into_iter().collect::<Vec<_>>();
        if let Some(shared) = interned.get(&key) {
            budget.reserve(
                0,
                0,
                std::mem::size_of::<Arc<[CallableAliasDefinition]>>(),
                stop,
            )?;
            callable.alias_definitions = Arc::clone(shared);
            continue;
        }

        let mut estimate = key
            .len()
            .checked_mul(2 * std::mem::size_of::<CallableAliasDefinition>() + 64)
            .ok_or(RenderError::OutputLimit)?;
        for &index in &key {
            step(&mut link_steps, budget, stop)?;
            let definition = &pending[index];
            let strings = definition
                .name
                .len()
                .checked_add(definition.source.revision().len())
                .and_then(|n| n.checked_add(definition.source.path().len()))
                .and_then(|n| n.checked_add(generated_path.len()))
                .and_then(|n| n.checked_add(definition.lowered_definition.len()))
                .ok_or(RenderError::OutputLimit)?;
            let dependency_bytes = definition
                .dependency_names
                .iter()
                .try_fold(
                    definition
                        .dependency_names
                        .len()
                        .checked_mul(std::mem::size_of::<String>())
                        .ok_or(RenderError::OutputLimit)?,
                    |total, name| total.checked_add(name.len()),
                )
                .ok_or(RenderError::OutputLimit)?;
            estimate = estimate
                .checked_add(strings)
                .and_then(|n| n.checked_add(dependency_bytes))
                .and_then(|n| {
                    n.checked_add(
                        definition
                            .dependency_names
                            .len()
                            .checked_mul(2 * std::mem::size_of::<usize>())?,
                    )
                })
                .ok_or(RenderError::OutputLimit)?;
        }
        budget.reserve(0, 0, estimate, stop)?;
        let mut definitions = Vec::with_capacity(key.len());
        for &index in &key {
            checkpoint(stop)?;
            let pending = &pending[index];
            let source_span = span(pending.source_span)?;
            let declaration_span = span(pending.declaration_span)?;
            let type_token_span = span(pending.type_token_span)?;
            let dependencies = pending
                .dependency_names
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<_>>();
            let definition = CallableAliasDefinition::admit(
                pending.name.clone(),
                pending.source.revision(),
                pending.source.path(),
                pending.source.sha256(),
                source_span,
                generated_path,
                generated_sha256,
                declaration_span,
                type_token_span,
                pending.lowered_definition.clone(),
                dependencies,
            )
            .map_err(|_| RenderError::InvalidSource)?;
            definitions.push(definition);
        }
        let shared: Arc<[CallableAliasDefinition]> = definitions.into();
        budget.reserve(0, 0, std::mem::size_of::<usize>() * 4 + 64, stop)?;
        interned.insert(key, Arc::clone(&shared));
        budget.reserve(
            0,
            0,
            std::mem::size_of::<Arc<[CallableAliasDefinition]>>(),
            stop,
        )?;
        callable.alias_definitions = shared;
    }
    Ok(())
}

fn step(
    total: &mut usize,
    budget: &mut super::super::native::lowering::CaptureBudget,
    stop: &AtomicBool,
) -> Result<(), RenderError> {
    checkpoint(stop)?;
    *total = total.checked_add(1).ok_or(RenderError::OutputLimit)?;
    if *total > MAX_ALIAS_LINK_STEPS {
        return Err(RenderError::InputLimit);
    }
    budget.reserve(0, 0, 1, stop)
}

fn checkpoint(stop: &AtomicBool) -> Result<(), RenderError> {
    if stop.load(Ordering::Acquire) {
        Err(RenderError::Cancelled)
    } else {
        Ok(())
    }
}

fn span(span: Span) -> Result<wow_core::SourceSpan, RenderError> {
    wow_core::SourceSpan::byte_range(
        u64::try_from(span.start).map_err(|_| RenderError::OutputLimit)?,
        u64::try_from(span.end).map_err(|_| RenderError::OutputLimit)?,
    )
    .map_err(|_| RenderError::InvalidSource)
}

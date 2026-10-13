//! Bounded links from used callable type atoms to exact emitted declarations.
use super::{CaptureBudget, CapturedCallable, RenderError};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wow_core::{CallableTypeDeclaration, CallableTypeDeclarationKind};

pub(crate) const MAX_SELECTED_DECLARATIONS: usize = 4096;
pub(crate) const MAX_DECLARATION_BYTES: usize = 16 * 1024 * 1024;
const MAX_TYPE_LINK_STEPS: usize = 16 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct Catalog {
    declarations: Vec<CallableTypeDeclaration>,
    bytes: usize,
}

impl Catalog {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn push(
        &mut self,
        kind: CallableTypeDeclarationKind,
        source_name: &str,
        rendered_name: &str,
        source_revision: &str,
        source_path: &str,
        source_sha256: &str,
        source_span: wow_core::SourceSpan,
        generated_path: &str,
        generated_sha256: &str,
        declaration_span: wow_core::SourceSpan,
        name_span: wow_core::SourceSpan,
        budget: &mut CaptureBudget,
        stop: &AtomicBool,
    ) -> Result<(), RenderError> {
        checkpoint(stop)?;
        let estimated = std::mem::size_of::<CallableTypeDeclaration>()
            .checked_add(source_name.len())
            .and_then(|n| n.checked_add(rendered_name.len()))
            .and_then(|n| n.checked_add(source_revision.len()))
            .and_then(|n| n.checked_add(source_path.len()))
            .and_then(|n| n.checked_add(generated_path.len()))
            .ok_or(RenderError::OutputLimit)?;
        let next_bytes = self
            .bytes
            .checked_add(estimated)
            .ok_or(RenderError::OutputLimit)?;
        if self.declarations.len() >= super::MAX_UNITS
            || next_bytes > super::lowering::MAX_CAPTURE_BYTES
        {
            return Err(RenderError::OutputLimit);
        }
        budget.reserve(0, 0, estimated, stop)?;
        let declaration = CallableTypeDeclaration::admit(
            kind,
            source_name,
            rendered_name,
            source_revision,
            source_path,
            source_sha256,
            source_span,
            generated_path,
            generated_sha256,
            declaration_span,
            name_span,
        )
        .map_err(|_| RenderError::InvalidSource)?;
        self.bytes = next_bytes;
        super::lowering::reserve_vec(&mut self.declarations, 1, budget, stop)?;
        self.declarations.push(declaration);
        Ok(())
    }

    /// Attach canonical unique-name slices using atoms already retained by the emitters.
    pub(crate) fn attach(
        self,
        callables: &mut [CapturedCallable],
        budget: &mut CaptureBudget,
        stop: &AtomicBool,
    ) -> Result<(), RenderError> {
        let mut work = 0usize;
        let mut by_name = BTreeMap::<Box<str>, Option<usize>>::new();
        for (index, declaration) in self.declarations.iter().enumerate() {
            step(&mut work, stop)?;
            checkpoint(stop)?;
            let name = declaration.rendered_name();
            let node_bytes = std::mem::size_of::<(Box<str>, Option<usize>)>()
                .checked_add(name.len())
                .and_then(|n| n.checked_add(48))
                .ok_or(RenderError::OutputLimit)?;
            budget.reserve(0, 0, node_bytes, stop)?;
            match by_name.entry(name.into()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(Some(index));
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    entry.insert(None);
                }
            }
        }

        let mut interned = BTreeMap::<Vec<usize>, Arc<[CallableTypeDeclaration]>>::new();
        for callable in callables {
            step(&mut work, stop)?;
            checkpoint(stop)?;
            let mut names = BTreeSet::<Box<str>>::new();
            for field in callable.fields.iter() {
                for atom in field.type_atoms() {
                    step(&mut work, stop)?;
                    budget.reserve(0, 0, atom.rendered_name().len() + 48, stop)?;
                    names.insert(atom.rendered_name().into());
                }
            }
            for alias in callable.alias_definitions.iter() {
                for atom in alias.type_atoms() {
                    step(&mut work, stop)?;
                    budget.reserve(0, 0, atom.rendered_name().len() + 48, stop)?;
                    names.insert(atom.rendered_name().into());
                }
            }
            budget.reserve(0, 0, std::mem::size_of::<Vec<usize>>(), stop)?;
            let mut selected = Vec::new();
            for name in names {
                step(&mut work, stop)?;
                if let Some(Some(index)) = by_name.get(name.as_ref()) {
                    if selected.len() >= MAX_SELECTED_DECLARATIONS {
                        return Err(RenderError::OutputLimit);
                    }
                    super::lowering::reserve_vec(&mut selected, 1, budget, stop)?;
                    budget.reserve(0, 0, std::mem::size_of::<usize>(), stop)?;
                    selected.push(*index);
                }
            }
            let mut descriptor_bytes = 0usize;
            for &index in &selected {
                step(&mut work, stop)?;
                descriptor_bytes = descriptor_bytes
                    .checked_add(self.declarations[index].metadata_bytes())
                    .ok_or(RenderError::OutputLimit)?;
            }
            if descriptor_bytes > MAX_DECLARATION_BYTES {
                return Err(RenderError::OutputLimit);
            }
            step(&mut work, stop)?;
            if let Some(shared) = interned.get(&selected) {
                budget.reserve(
                    0,
                    0,
                    std::mem::size_of::<Arc<[CallableTypeDeclaration]>>(),
                    stop,
                )?;
                callable.type_declarations = Arc::clone(shared);
                continue;
            }
            let retained = descriptor_bytes
                .checked_add(
                    selected
                        .len()
                        .checked_mul(std::mem::size_of::<CallableTypeDeclaration>())
                        .ok_or(RenderError::OutputLimit)?,
                )
                .ok_or(RenderError::OutputLimit)?;
            budget.reserve(0, 0, retained, stop)?;
            let declarations = selected
                .iter()
                .map(|&index| self.declarations[index].clone())
                .collect::<Vec<_>>();
            let shared: Arc<[CallableTypeDeclaration]> = declarations.into();
            budget.reserve(0, 0, 96, stop)?;
            interned.insert(selected, Arc::clone(&shared));
            budget.reserve(
                0,
                0,
                std::mem::size_of::<Arc<[CallableTypeDeclaration]>>(),
                stop,
            )?;
            callable.type_declarations = shared;
        }
        Ok(())
    }
}

fn step(work: &mut usize, stop: &AtomicBool) -> Result<(), RenderError> {
    checkpoint(stop)?;
    *work = work.checked_add(1).ok_or(RenderError::OutputLimit)?;
    if *work > MAX_TYPE_LINK_STEPS {
        return Err(RenderError::OutputLimit);
    }
    Ok(())
}

fn checkpoint(stop: &AtomicBool) -> Result<(), RenderError> {
    if stop.load(Ordering::Acquire) {
        Err(RenderError::Cancelled)
    } else {
        Ok(())
    }
}

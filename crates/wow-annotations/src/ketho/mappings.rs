//! Exact nested ranges from the emitter, never searches over generated text.
use super::{Field, Output, Position, RenderError, Renderer};

pub(super) const MAX_MEMBERS: usize = 262_144;

/// The ordered source-field collection addressed by a rendered member.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberPosition {
    Parameter,
    Return,
    Field,
}

/// Half-open UTF-8 byte range in the complete rendered system. Callback returns
/// map their type component, because that syntax does not render a return name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedMember {
    pub position: MemberPosition,
    pub index: usize,
    pub start: usize,
    pub end: usize,
    pub(crate) lowering: RenderedTypeToken,
}

/// One input atom and its exact lowered spelling, with a range relative to the
/// final lowered type token. It is captured during the renderer's single pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoweredTypeAtom {
    pub(crate) source_name: String,
    pub(crate) rendered_name: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoweredType {
    pub(crate) text: String,
    pub(crate) atoms: Vec<LoweredTypeAtom>,
}

impl LoweredType {
    pub(crate) fn is_union(&self) -> bool {
        self.atoms.len() > 1
    }

    pub(crate) fn shift_atoms(&mut self, amount: usize) -> Result<(), RenderError> {
        for atom in &mut self.atoms {
            atom.start = atom
                .start
                .checked_add(amount)
                .ok_or(RenderError::OutputLimit)?;
            atom.end = atom
                .end
                .checked_add(amount)
                .ok_or(RenderError::OutputLimit)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderedTypeToken {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) optional_marker: Option<(usize, usize)>,
    pub(crate) lowered_type: String,
    pub(crate) optional: bool,
    pub(crate) atoms: Vec<LoweredTypeAtom>,
}

impl Renderer {
    pub(super) fn field_mapped(
        &self,
        out: &mut Output,
        position: MemberPosition,
        index: usize,
        field: &Field,
        members: &mut Vec<RenderedMember>,
    ) -> Result<(), RenderError> {
        let start = out.bytes.len();
        let label = match position {
            MemberPosition::Parameter => Position::Param,
            MemberPosition::Return => Position::Return,
            MemberPosition::Field => Position::Field,
        };
        let lowering = self.field(out, label, field)?;
        members.push(RenderedMember {
            position,
            index,
            start,
            end: out.bytes.len(),
            lowering,
        });
        Ok(())
    }
}

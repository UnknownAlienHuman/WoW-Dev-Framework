//! Join emitter-owned field ranges to the exact admitted source descriptors.
use super::{
    CallableFact, DocumentationDocument, RenderError, SourceMapping, Span, TableFact, link,
};
use crate::ketho::{MemberPosition, RenderedSystem};

pub(super) const MAX_MEMBERS: usize = 65_536;

pub(super) fn append_members(
    rendered: &RenderedSystem,
    document: &DocumentationDocument,
    functions: &[&CallableFact<'_>],
    tables: &[&TableFact<'_>],
    mappings: &mut Vec<SourceMapping>,
    budget: &mut usize,
) -> Result<(), RenderError> {
    let count = rendered
        .declarations
        .iter()
        .map(|d| d.members.len())
        .sum::<usize>();
    *budget = budget.checked_sub(count).ok_or(RenderError::OutputLimit)?;
    for declaration in &rendered.declarations {
        for member in &declaration.members {
            if member.start >= member.end
                || member.start < declaration.start
                || member.end > declaration.end
                || rendered.text.get(member.start..member.end).is_none()
            {
                return Err(RenderError::InvalidSource);
            }
            let fields = if declaration.table {
                match (tables.get(declaration.index), member.position) {
                    (Some(TableFact::Structure { fields, .. }), MemberPosition::Field) => fields,
                    (Some(TableFact::Callback { arguments, .. }), MemberPosition::Parameter) => {
                        arguments
                    }
                    (Some(TableFact::Callback { returns, .. }), MemberPosition::Return) => returns,
                    _ => return Err(RenderError::InvalidSource),
                }
            } else {
                let function = functions
                    .get(declaration.index)
                    .ok_or(RenderError::InvalidSource)?;
                match member.position {
                    MemberPosition::Parameter => &function.arguments,
                    MemberPosition::Return => &function.returns,
                    MemberPosition::Field => return Err(RenderError::InvalidSource),
                }
            };
            let field = fields.get(member.index).ok_or(RenderError::InvalidSource)?;
            mappings.push(SourceMapping {
                granularity: match member.position {
                    MemberPosition::Parameter => "parameter",
                    MemberPosition::Return => "return",
                    MemberPosition::Field => "field",
                },
                generated: Span {
                    start: member.start,
                    end: member.end,
                },
                source: link(document, field.raw),
            });
        }
    }
    Ok(())
}

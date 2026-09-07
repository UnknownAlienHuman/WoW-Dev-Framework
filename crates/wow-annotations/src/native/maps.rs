//! Join emitter-owned field ranges to the exact admitted source descriptors.
use super::{
    CallableFact, DocumentationDocument, FieldFact, RenderError, SourceMapping, Span, TableFact,
    link,
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
    if rendered.declarations.len() != functions.len() + tables.len() {
        return Err(RenderError::InvalidSource);
    }
    let count = rendered
        .declarations
        .iter()
        .try_fold(0usize, |count, declaration| {
            count.checked_add(declaration.members.len())
        })
        .ok_or(RenderError::OutputLimit)?;
    *budget = budget.checked_sub(count).ok_or(RenderError::OutputLimit)?;
    let mut next_function = 0;
    let mut next_table = 0;
    for declaration in &rendered.declarations {
        let empty: &[FieldFact<'_>] = &[];
        let (parameters, returns, fields) = if declaration.table {
            if declaration.index != next_table {
                return Err(RenderError::InvalidSource);
            }
            next_table += 1;
            match tables.get(declaration.index) {
                Some(TableFact::Structure { fields, .. }) => (empty, empty, fields.as_slice()),
                Some(TableFact::Callback {
                    arguments, returns, ..
                }) => (arguments.as_slice(), returns.as_slice(), empty),
                _ => return Err(RenderError::InvalidSource),
            }
        } else {
            if declaration.index != next_function || next_table != 0 {
                return Err(RenderError::InvalidSource);
            }
            next_function += 1;
            let function = functions
                .get(declaration.index)
                .ok_or(RenderError::InvalidSource)?;
            (
                function.arguments.as_slice(),
                function.returns.as_slice(),
                empty,
            )
        };
        let mut ordinal = 0;
        let mut previous_end = declaration.start;
        for (position, collection, granularity) in [
            (MemberPosition::Parameter, parameters, "parameter"),
            (MemberPosition::Return, returns, "return"),
            (MemberPosition::Field, fields, "field"),
        ] {
            for (index, field) in collection.iter().enumerate() {
                let member = declaration
                    .members
                    .get(ordinal)
                    .ok_or(RenderError::InvalidSource)?;
                if member.position != position
                    || member.index != index
                    || member.start >= member.end
                    || member.start < previous_end
                    || member.end > declaration.end
                    || rendered.text.get(member.start..member.end).is_none()
                    || field.raw.span.start >= field.raw.span.end
                    || field.raw.span.end > document.source_bytes()
                {
                    return Err(RenderError::InvalidSource);
                }
                // Bound local tables retain their definition spans, which need
                // not be nested inside the source callable's byte range.
                mappings.push(SourceMapping {
                    granularity,
                    generated: Span {
                        start: member.start,
                        end: member.end,
                    },
                    source: link(document, field.raw),
                });
                previous_end = member.end;
                ordinal += 1;
            }
        }
        if ordinal != declaration.members.len() {
            return Err(RenderError::InvalidSource);
        }
    }
    if next_function != functions.len() || next_table != tables.len() {
        return Err(RenderError::InvalidSource);
    }
    Ok(())
}

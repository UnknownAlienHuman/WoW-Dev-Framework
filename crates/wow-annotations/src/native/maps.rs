//! Join emitter-owned field ranges to the exact admitted source descriptors.
use super::lowering::{CaptureBudget, CapturedCallable, span};
use super::{
    CallableFact, DocumentationDocument, FieldFact, RenderError, SourceMapping, Span, TableFact,
    link,
};
use crate::ketho::{MemberPosition, RenderedSystem};
use std::sync::atomic::AtomicBool;
use wow_core::{CallableFieldLowering, CallableLoweringLane};

pub(super) const MAX_MEMBERS: usize = 65_536;

#[allow(clippy::too_many_arguments)]
pub(super) fn append_members(
    rendered: &mut RenderedSystem,
    document: &DocumentationDocument,
    functions: &[&CallableFact<'_>],
    tables: &[&TableFact<'_>],
    mappings: &mut Vec<SourceMapping>,
    budget: &mut usize,
    file_index: usize,
    catalog: &mut Vec<CapturedCallable>,
    capture_budget: &mut CaptureBudget,
    stop: &AtomicBool,
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
    for declaration in &mut rendered.declarations {
        capture_budget.reserve(0, 0, 0, stop)?;
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
        let mut captured_fields = Vec::new();
        if !declaration.table {
            let count = parameters
                .len()
                .checked_add(returns.len())
                .ok_or(RenderError::OutputLimit)?;
            let bytes = count
                .checked_mul(2 * std::mem::size_of::<CallableFieldLowering>())
                .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<CapturedCallable>()))
                .and_then(|bytes| bytes.checked_add(document.path().len()))
                .and_then(|bytes| bytes.checked_add(document.sha256().len()))
                .and_then(|bytes| bytes.checked_add(64))
                .ok_or(RenderError::OutputLimit)?;
            if count != 0 {
                capture_budget.reserve(1, count, bytes, stop)?;
                captured_fields = Vec::with_capacity(count);
            }
        }
        let mut ordinal = 0;
        let mut previous_end = declaration.start;
        for (position, collection, granularity) in [
            (MemberPosition::Parameter, parameters, "parameter"),
            (MemberPosition::Return, returns, "return"),
            (MemberPosition::Field, fields, "field"),
        ] {
            for (index, field) in collection.iter().enumerate() {
                capture_budget.reserve(0, 0, 0, stop)?;
                let member = declaration
                    .members
                    .get_mut(ordinal)
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
                let token = &mut member.lowering;
                if token.start < member.start
                    || token.start >= token.end
                    || token.end > member.end
                    || rendered.text.get(token.start..token.end)
                        != Some(token.lowered_type.as_str())
                {
                    return Err(RenderError::InvalidSource);
                }
                if !declaration.table {
                    capture_budget.reserve(0, 0, token.lowered_type.len(), stop)?;
                    let lane = match position {
                        MemberPosition::Parameter => CallableLoweringLane::Parameter,
                        MemberPosition::Return => CallableLoweringLane::Return,
                        MemberPosition::Field => return Err(RenderError::InvalidSource),
                    };
                    captured_fields.push(
                        CallableFieldLowering::admit(
                            lane,
                            index,
                            span(field.raw.span.start, field.raw.span.end)?,
                            span(member.start, member.end)?,
                            span(token.start, token.end)?,
                            token
                                .optional_marker
                                .map(|(start, end)| span(start, end))
                                .transpose()?,
                            std::mem::take(&mut token.lowered_type).into_boxed_str(),
                            token.optional,
                        )
                        .map_err(|_| RenderError::InvalidSource)?,
                    );
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
        if !declaration.table && !captured_fields.is_empty() {
            capture_budget.reserve(0, 0, 0, stop)?;
            let function = functions
                .get(declaration.index)
                .ok_or(RenderError::InvalidSource)?;
            catalog.push(CapturedCallable {
                file_index,
                declaration_span: span(declaration.start, declaration.end)?,
                source_path: document.path().into(),
                source_sha256: document.sha256().into(),
                source_span: span(function.raw.span.start, function.raw.span.end)?,
                fields: captured_fields.into(),
            });
        }
    }
    if next_function != functions.len() || next_table != tables.len() {
        return Err(RenderError::InvalidSource);
    }
    Ok(())
}

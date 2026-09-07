//! Native field projection keeps an unresolved symbolic default as provenance,
//! not a fabricated scalar or a reason to erase the containing API declaration.
use super::{
    Field, FieldFact, ProjectionIssue, RawKind, RawValue, RenderError, ScalarError, ScalarProjection,
    ScalarValue, issue,
};

pub(super) fn convert(
    fields: &[FieldFact<'_>],
    scalars: &mut ScalarProjection<'_, '_>,
    issues: &mut Vec<ProjectionIssue>,
) -> Result<Vec<Field>, RenderError> {
    fields
        .iter()
        .map(|field| {
            let default_text = match field.default {
                None
                | Some(RawValue {
                    kind: RawKind::Nil, ..
                }) => None,
                Some(raw) => Some(default(raw, scalars, issues)?),
            };
            let variadic = match field.stride_index.map(|value| &value.kind) {
                None | Some(RawKind::Nil | RawKind::Boolean(false)) => false,
                Some(RawKind::Number(n)) if n.parse::<u64>().is_ok_and(|v| v > 0) => true,
                _ => return Err(RenderError::InvalidVariadic),
            };
            Ok(Field {
                name: field.name.into(),
                type_name: field.type_name.into(),
                inner_type: field.inner_type.map(Into::into),
                nilable: field.nilable.unwrap_or(false),
                default_text,
                variadic,
            })
        })
        .collect()
}

fn default(
    raw: &RawValue,
    scalars: &mut ScalarProjection<'_, '_>,
    issues: &mut Vec<ProjectionIssue>,
) -> Result<String, RenderError> {
    let previous = scalars.records.len();
    match scalars.resolve(raw, None) {
        Ok(ScalarValue::Boolean(value)) => Ok(value.to_string()),
        Ok(ScalarValue::Number(value) | ScalarValue::String(value)) => Ok(value),
        Err(error) => {
            // Only a newly recorded, specifically unresolved bare name is a
            // symbolic default. Conflicts, cycles, unsupported expressions,
            // cancellation and exhausted budgets never take this path.
            let unresolved = scalars.records.get(previous).is_some_and(|record| {
                matches!(&record.result, Err(ScalarError::UnresolvedReference))
            });
            let RawKind::UnresolvedName(name) = &raw.kind else {
                return Err(error);
            };
            if error != RenderError::UnsupportedType || !unresolved {
                return Err(error);
            }
            crate::ketho::identifier(name)?;
            issues.push(issue(scalars.document, raw, "unresolved_symbolic_default"));
            // Default presence follows the descriptor, as in the Ketho profile;
            // its runtime value remains unknown and keeps projection partial.
            Ok(format!("{name} (unresolved source symbol)"))
        }
    }
}

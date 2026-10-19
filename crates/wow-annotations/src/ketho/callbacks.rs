//! Native callback signatures beyond the donor's argument-only byte profile.
use super::{
    Field, LoweredType, MemberPosition, Output, RenderError, RenderedMember, RenderedTypeToken,
    Renderer, validate_fields,
};

impl Renderer {
    pub(super) fn validate_callback_signature(
        &self,
        arguments: &[Field],
        returns: &[Field],
    ) -> Result<(), RenderError> {
        validate_fields(arguments, true)?;
        // Variadic return packs are a distinct consumer contract, not a tuple.
        validate_fields(returns, false)?;
        for argument in arguments {
            if argument.variadic && argument.default_text.is_some() {
                return Err(RenderError::InvalidVariadic);
            }
            self.validate_field(argument)?;
        }
        for result in returns {
            self.validate_field(result)?;
        }
        Ok(())
    }

    /// The same type lowering serves fields and function-type components.
    /// Parentheses preserve union precedence before arrays and nilability.
    pub(super) fn field_type(&self, field: &Field) -> Result<LoweredType, RenderError> {
        let mut lowered =
            self.lower_type_mapped(field.inner_type.as_deref().unwrap_or(&field.type_name))?;
        if field.inner_type.is_some() {
            if lowered.is_union() {
                lowered.text.insert(0, '(');
                lowered.text.push(')');
                lowered.shift_atoms(1)?;
            }
            lowered.text.push_str("[]");
        }
        if (field.nilable || field.default_text.is_some())
            && lowered.is_union()
            && field.inner_type.is_none()
        {
            lowered.text.insert(0, '(');
            lowered.text.push(')');
            lowered.shift_atoms(1)?;
        }
        Ok(lowered)
    }

    pub(super) fn callback_signature(
        &self,
        out: &mut Output,
        name: &str,
        arguments: &[Field],
        returns: &[Field],
        members: &mut Vec<RenderedMember>,
    ) -> Result<(usize, usize), RenderError> {
        self.validate_callback_signature(arguments, returns)?;
        out.push("---@alias ")?;
        let name_start = out.bytes.len();
        out.push(name)?;
        let name_span = (name_start, out.bytes.len());
        out.push(" FunctionContainer|fun(")?;
        for (index, argument) in arguments.iter().enumerate() {
            if index != 0 {
                out.push(", ")?;
            }
            let start = out.bytes.len();
            out.push(if argument.variadic {
                "..."
            } else {
                &argument.name
            })?;
            let optional_marker =
                if !argument.variadic && (argument.nilable || argument.default_text.is_some()) {
                    let marker_start = out.bytes.len();
                    out.push("?")?;
                    Some((marker_start, out.bytes.len()))
                } else {
                    None
                };
            out.push(": ")?;
            let lowered = self.field_type(argument)?;
            let type_start = out.bytes.len();
            out.push(&lowered.text)?;
            let type_end = out.bytes.len();
            let optional_marker = if argument.variadic && argument.nilable {
                let marker_start = out.bytes.len();
                out.push("?")?;
                Some((marker_start, out.bytes.len()))
            } else {
                optional_marker
            };
            members.push(RenderedMember {
                position: MemberPosition::Parameter,
                index,
                start,
                end: out.bytes.len(),
                lowering: RenderedTypeToken {
                    start: type_start,
                    end: type_end,
                    optional_marker,
                    lowered_type: lowered.text,
                    optional: argument.nilable || argument.default_text.is_some(),
                    atoms: lowered.atoms,
                },
            });
        }
        out.push(")")?;
        if !returns.is_empty() {
            out.push(": (")?;
        }
        for (index, result) in returns.iter().enumerate() {
            if index != 0 {
                out.push(", ")?;
            }
            let start = out.bytes.len();
            let lowered = self.field_type(result)?;
            let type_start = out.bytes.len();
            out.push(&lowered.text)?;
            let type_end = out.bytes.len();
            let optional_marker = if result.nilable || result.default_text.is_some() {
                let marker_start = out.bytes.len();
                out.push("?")?;
                Some((marker_start, out.bytes.len()))
            } else {
                None
            };
            members.push(RenderedMember {
                position: MemberPosition::Return,
                index,
                start,
                end: out.bytes.len(),
                lowering: RenderedTypeToken {
                    start: type_start,
                    end: type_end,
                    optional_marker,
                    lowered_type: lowered.text,
                    optional: result.nilable || result.default_text.is_some(),
                    atoms: lowered.atoms,
                },
            });
        }
        if !returns.is_empty() {
            out.push(")")?;
        }
        Ok(name_span)
    }
}

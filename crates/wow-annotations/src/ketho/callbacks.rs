//! Native callback signatures beyond the donor's argument-only byte profile.
use super::{Field, Output, RenderError, Renderer, safe_text, validate_fields};

impl Renderer {
    /// The same type lowering serves fields and function-type components.
    /// Parentheses preserve union precedence before arrays and nilability.
    pub(super) fn field_type(&self, field: &Field) -> Result<String, RenderError> {
        let mut name = self.lower_type(field.inner_type.as_deref().unwrap_or(&field.type_name))?;
        if field.inner_type.is_some() {
            if name.contains('|') {
                name = format!("({name})");
            }
            name.push_str("[]");
        }
        if (field.nilable || field.default_text.is_some())
            && name.contains('|')
            && field.inner_type.is_none()
        {
            name = format!("({name})");
        }
        Ok(name)
    }

    pub(super) fn callback_signature(
        &self,
        out: &mut Output,
        name: &str,
        arguments: &[Field],
        returns: &[Field],
    ) -> Result<(), RenderError> {
        validate_fields(arguments, true)?;
        // Variadic return packs are a distinct consumer contract, not a tuple.
        validate_fields(returns, false)?;
        for field in arguments.iter().chain(returns) {
            if let Some(default) = &field.default_text {
                safe_text(default)?;
            }
        }
        out.push("---@alias ")?;
        out.push(name)?;
        out.push(" FunctionContainer|fun(")?;
        for (index, argument) in arguments.iter().enumerate() {
            if argument.variadic && argument.default_text.is_some() {
                return Err(RenderError::InvalidVariadic);
            }
            if index != 0 {
                out.push(", ")?;
            }
            out.push(if argument.variadic {
                "..."
            } else {
                &argument.name
            })?;
            if !argument.variadic && (argument.nilable || argument.default_text.is_some()) {
                out.push("?")?;
            }
            out.push(": ")?;
            out.push(&self.field_type(argument)?)?;
            if argument.variadic && argument.nilable {
                out.push("?")?;
            }
        }
        out.push(")")?;
        if !returns.is_empty() {
            out.push(": (")?;
        }
        for (index, result) in returns.iter().enumerate() {
            if index != 0 {
                out.push(", ")?;
            }
            out.push(&self.field_type(result)?)?;
            if result.nilable || result.default_text.is_some() {
                out.push("?")?;
            }
        }
        if !returns.is_empty() {
            out.push(")")?;
        }
        Ok(())
    }
}

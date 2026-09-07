//! Lower source numeric lexemes without treating constants as integer enums.
//! The reference resolver keeps the original lexeme and transitive evidence;
//! only the rendering value uses the existing binary64 literal contract.
use super::{LiteralValue, RenderError, ScalarValue};
use wow_reference::native_constants::exact_integer_magnitude;

const MAX_EXACT_INTEGER: i64 = 9_007_199_254_740_991;

pub(super) fn literal(value: ScalarValue, constants: bool) -> Result<LiteralValue, RenderError> {
    match value {
        ScalarValue::Boolean(value) => Ok(LiteralValue::Boolean(value)),
        ScalarValue::String(value) => Ok(LiteralValue::String(value)),
        ScalarValue::Number(text) => number(&text, constants),
    }
}

fn number(text: &str, constants: bool) -> Result<LiteralValue, RenderError> {
    let (negative, magnitude) = text
        .strip_prefix('-')
        .map_or((false, text), |value| (true, value));
    let hex = magnitude
        .strip_prefix("0x")
        .or_else(|| magnitude.strip_prefix("0X"));
    if hex.is_some() || !magnitude.contains(['.', 'e', 'E']) {
        // Integer overflow must never fall back to a rounded floating point value.
        let value = match hex {
            Some(hex) => i64::from_str_radix(hex, 16),
            None => magnitude.parse::<i64>(),
        }
        .map_err(|_| RenderError::UnsupportedType)?;
        if !(0..=MAX_EXACT_INTEGER).contains(&value) {
            return Err(RenderError::UnsupportedType);
        }
        if constants && negative && value == 0 {
            return Ok(LiteralValue::Number(-0.0));
        }
        return Ok(LiteralValue::Integer(if negative { -value } else { value }));
    }
    // Ketho emits decimal/scientific constant values, but its enum lane has a
    // distinct integer contract. Do not widen enum membership as a side effect.
    if !constants {
        return Err(RenderError::UnsupportedType);
    }
    let value = text
        .parse::<f64>()
        .map_err(|_| RenderError::UnsupportedType)?;
    if !value.is_finite() || value.abs() > MAX_EXACT_INTEGER as f64 {
        return Err(RenderError::UnsupportedType);
    }
    if value.fract() == 0.0 {
        // A nonzero literal underflowing to zero, or a fractional/wide integer
        // rounded to an integer, cannot silently become a different declaration.
        let exact = exact_integer_magnitude(magnitude).map_err(|_| RenderError::UnsupportedType)?;
        if exact != value.abs() as u64 {
            return Err(RenderError::UnsupportedType);
        }
        if value == 0.0 && value.is_sign_negative() {
            return Ok(LiteralValue::Number(-0.0));
        }
        return Ok(LiteralValue::Integer(value as i64));
    }
    Ok(LiteralValue::Number(value))
}

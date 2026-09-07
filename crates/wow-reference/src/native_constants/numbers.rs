//! Exact numeric lowering shared by scalar arithmetic and annotation projection.
use super::{MAX_EXACT_INTEGER, ScalarError};

/// Read the unsigned magnitude of a source numeric lexeme as an exact integer.
/// Decimal points and scientific notation are accepted only when their exact
/// mathematical value is integral. No binary64 rounding participates here.
///
/// The native parser still owns Lua syntax admission. This helper additionally
/// rejects signs, whitespace and malformed numeric components at its boundary.
///
/// # Errors
/// Returns `NonIntegralArithmetic` for fractional or malformed input, `OutOfRange`
/// for values outside the conservative Lua 5.1 exact-integer interval, and `Limit`
/// for lexemes over 2,048 bytes. The caller retains sign and original source text.
pub fn exact_integer_magnitude(text: &str) -> Result<u64, ScalarError> {
    if text.len() > 2048 {
        return Err(ScalarError::Limit);
    }
    let value = if let Some(hex) = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
    {
        if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ScalarError::NonIntegralArithmetic);
        }
        u64::from_str_radix(hex, 16).map_err(|_| ScalarError::OutOfRange)?
    } else {
        decimal_magnitude(text)?
    };
    if value > MAX_EXACT_INTEGER as u64 {
        return Err(ScalarError::OutOfRange);
    }
    Ok(value)
}

fn decimal_magnitude(text: &str) -> Result<u64, ScalarError> {
    let (mantissa, exponent) = text.split_once(['e', 'E']).unwrap_or((text, "0"));
    let exponent_digits = exponent
        .strip_prefix('+')
        .or_else(|| exponent.strip_prefix('-'))
        .unwrap_or(exponent);
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if (whole.is_empty() && fraction.is_empty())
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || exponent_digits.is_empty()
        || !exponent_digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ScalarError::NonIntegralArithmetic);
    }
    let exponent = exponent.parse::<i64>().map_err(|_| ScalarError::OutOfRange)?;
    let digits = [whole, fraction].concat();
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(0);
    }
    let coefficient = digits.trim_end_matches('0');
    let scale = exponent
        .checked_sub(i64::try_from(fraction.len()).map_err(|_| ScalarError::Limit)?)
        .and_then(|scale| {
            scale.checked_add(i64::try_from(digits.len() - coefficient.len()).ok()?)
        })
        .ok_or(ScalarError::OutOfRange)?;
    if scale < 0 {
        return Err(ScalarError::NonIntegralArithmetic);
    }
    // Sixteen decimal digits cover the entire permitted integer interval.
    if scale > 16 || coefficient.len() as u64 + scale as u64 > 16 {
        return Err(ScalarError::OutOfRange);
    }
    coefficient
        .parse::<u64>()
        .ok()
        .and_then(|value| value.checked_mul(10_u64.checked_pow(scale as u32)?))
        .ok_or(ScalarError::OutOfRange)
}

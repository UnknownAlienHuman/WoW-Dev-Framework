//! Typed, consumer-neutral literal inputs. No acquisition or runtime handles.
/// An already resolved event literal and its source-owned payload display text.
/// Payload text is a comment, not a function signature or a runtime access claim.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EventLiteral {
    pub name: String,
    pub payload: String,
}

/// Typed scalar data, never an executable expression. Strings remain strings,
/// including decimal text used by source data to preserve wide enum values.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum LiteralValue {
    Boolean(bool),
    Integer(i64),
    /// Finite floating-point constants. This initial enum profile admits only
    /// integers, booleans and strings; fractional enum forms are unsupported.
    Number(f64),
    String(String),
}

/// One unique field in an enum or constant group.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LiteralMember {
    pub name: String,
    pub value: LiteralValue,
}

/// Numeric display policy only. Hexadecimal formatting does not assert that
/// the input is a bitmask, nor infer it from a known enum name.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerFormat {
    Decimal,
    Hexadecimal,
}

/// One named enum. Values are canonically sorted by type, value, then name, as
/// in the donor; equal booleans also get a deterministic name tie-breaker.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EnumDeclaration {
    pub name: String,
    pub values: Vec<LiteralMember>,
    pub integer_format: IntegerFormat,
}

/// The reference/projection owner explicitly selects a constant group's order.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberOrder {
    Name,
    Value,
}

/// Constants are separate from enums, even when their values are all numbers.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ConstantGroup {
    pub name: String,
    pub values: Vec<LiteralMember>,
    pub order: MemberOrder,
}

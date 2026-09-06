//! Ketho literal algorithm shared by the native adapter and separately built Wasm.
//! No source acquisition, VM, IO, editor state or compiled donor inventory.
mod support;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use support::*;
pub use wow_render_contract::*;
const MAX_MEMBERS: usize = 65_536;
const MAX_EXACT_INTEGER: i64 = 9_007_199_254_740_991;

/// Bounded, dependency-free literal renderer. No partially rendered output is
/// returned on failure. Results are UTF-8/LF with the donor's analysis-only meta
/// header and contain no source-provided code or annotation directives.
#[derive(Clone, Debug)]
pub struct LiteralRenderer {
    max_output_bytes: usize,
}

impl LiteralRenderer {
    pub fn new(max_output_bytes: usize) -> Result<Self, LiteralError> {
        if max_output_bytes > MAX_OUTPUT_BYTES {
            return Err(LiteralError::InputLimit);
        }
        Ok(Self { max_output_bytes })
    }

    /// Ketho `GetEventLiterals`. Retains the open `string` base of FrameEvent;
    /// this artifact is not an exhaustive runtime event whitelist.
    pub fn render_events(&self, events: &[EventLiteral]) -> Result<String, LiteralError> {
        item_limit(events.len())?;
        let mut sorted = events.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        let mut previous = None;
        let mut out = self.output("---@meta _\n---@alias FrameEvent string\n")?;
        for event in sorted {
            literal_name(&event.name)?;
            unique(&mut previous, &event.name)?;
            safe_text(&event.payload)?;
            if event.payload.contains('`') {
                return Err(LiteralError::UnsafeDocumentation);
            }
            out.push("---|")?;
            quoted(&mut out, &event.name)?;
            if !event.payload.is_empty() {
                out.push(" # `")?;
                out.push(&event.payload)?;
                out.push("`")?;
            }
            out.push("\n")?;
        }
        Ok(out.bytes)
    }

    /// Ketho `GetCVarLiterals`, without downloading/executing a resource file.
    /// CVar names are string literals, not identifiers or injected globals.
    pub fn render_cvars(&self, names: &[String]) -> Result<String, LiteralError> {
        item_limit(names.len())?;
        let mut sorted = names.iter().collect::<Vec<_>>();
        sorted.sort();
        let mut previous = None;
        let mut out = self.output("---@meta _\n---@alias CVar string\n")?;
        for name in sorted {
            literal_name(name)?;
            unique(&mut previous, name)?;
            out.push("---|")?;
            quoted(&mut out, name)?;
            out.push("\n")?;
        }
        Ok(out.bytes)
    }

    /// Ketho `GetEnumTable`: separate Enum declarations and Constants groups.
    /// Family/name/value ordering is independent of input iteration order.
    pub fn render_enums(
        &self,
        enums: &[EnumDeclaration],
        constants: &[ConstantGroup],
    ) -> Result<String, LiteralError> {
        item_limit(enums.len().saturating_add(constants.len()))?;
        let member_count = enums
            .iter()
            .map(|v| v.values.len())
            .chain(constants.iter().map(|v| v.values.len()))
            .fold(0usize, usize::saturating_add);
        if member_count > MAX_MEMBERS {
            return Err(LiteralError::InputLimit);
        }
        let mut enums = enums.iter().collect::<Vec<_>>();
        enums.sort_by(|a, b| a.name.cmp(&b.name));
        let mut constants = constants.iter().collect::<Vec<_>>();
        constants.sort_by(|a, b| a.name.cmp(&b.name));
        let mut out = self.output("---@meta _\nEnum = {}\n\n")?;
        let mut previous = None;
        for declaration in enums {
            identifier(&declaration.name)?;
            unique(&mut previous, &declaration.name)?;
            let members = members(&declaration.values, MemberOrder::Value, true)?;
            out.push("---@enum Enum.")?;
            out.push(&declaration.name)?;
            out.push("\nEnum.")?;
            out.push(&declaration.name)?;
            out.push(" = {\n")?;
            for member in members {
                out.push("\t")?;
                out.push(&member.name)?;
                out.push(" = ")?;
                scalar(&mut out, &member.value, declaration.integer_format)?;
                out.push(",\n")?;
            }
            out.push("}\n\n")?;
        }
        out.push("Constants = {\n")?;
        previous = None;
        for group in constants {
            identifier(&group.name)?;
            unique(&mut previous, &group.name)?;
            let members = members(&group.values, group.order, false)?;
            out.push("\t")?;
            out.push(&group.name)?;
            out.push(" = {\n")?;
            for member in members {
                out.push("\t\t")?;
                out.push(&member.name)?;
                out.push(" = ")?;
                scalar(&mut out, &member.value, IntegerFormat::Decimal)?;
                out.push(",\n")?;
            }
            out.push("\t},\n")?;
        }
        out.push("}\n")?;
        Ok(out.bytes)
    }

    fn output(&self, header: &str) -> Result<Output, LiteralError> {
        let mut out = Output {
            bytes: String::new(),
            limit: self.max_output_bytes,
        };
        out.push(header)?;
        Ok(out)
    }
}

fn item_limit(count: usize) -> Result<(), LiteralError> {
    if count > MAX_ITEMS {
        Err(LiteralError::InputLimit)
    } else {
        Ok(())
    }
}

fn literal_name(name: &str) -> Result<(), LiteralError> {
    if name.is_empty() {
        return Err(LiteralError::InvalidIdentifier);
    }
    safe_text(name)
}

fn unique<'a>(previous: &mut Option<&'a str>, name: &'a str) -> Result<(), LiteralError> {
    if *previous == Some(name) {
        return Err(LiteralError::DuplicateName);
    }
    *previous = Some(name);
    Ok(())
}

fn members(
    input: &[LiteralMember],
    order: MemberOrder,
    is_enum: bool,
) -> Result<Vec<&LiteralMember>, LiteralError> {
    item_limit(input.len())?;
    let mut names = BTreeSet::new();
    for member in input {
        identifier(&member.name)?;
        if !names.insert(&member.name) {
            return Err(LiteralError::DuplicateName);
        }
        match &member.value {
            LiteralValue::Integer(value) if value.unsigned_abs() > MAX_EXACT_INTEGER as u64 => {
                return Err(LiteralError::UnsupportedLiteral);
            }
            LiteralValue::Number(value) if is_enum || !value.is_finite() => {
                return Err(LiteralError::UnsupportedLiteral);
            }
            LiteralValue::String(value) if value.len() > MAX_TEXT_BYTES => {
                return Err(LiteralError::InputLimit);
            }
            _ => {}
        }
    }
    let mut sorted = input.iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| {
        if order == MemberOrder::Value {
            compare_values(&a.value, &b.value).then_with(|| a.name.cmp(&b.name))
        } else {
            a.name.cmp(&b.name)
        }
    });
    Ok(sorted)
}

fn compare_values(a: &LiteralValue, b: &LiteralValue) -> Ordering {
    use LiteralValue::{Boolean, Integer, Number, String};
    match (a, b) {
        (Boolean(a), Boolean(b)) => b.cmp(a), // donor: true first
        (Integer(a), Integer(b)) => a.cmp(b),
        (Number(a), Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (Integer(a), Number(b)) => (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal),
        (Number(a), Integer(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal),
        (String(a), String(b)) => a.cmp(b),
        _ => value_rank(a).cmp(&value_rank(b)),
    }
}

fn value_rank(value: &LiteralValue) -> u8 {
    match value {
        LiteralValue::Boolean(_) => 0,
        LiteralValue::Integer(_) | LiteralValue::Number(_) => 1,
        LiteralValue::String(_) => 2,
    }
}

fn scalar(
    out: &mut Output,
    value: &LiteralValue,
    style: IntegerFormat,
) -> Result<(), LiteralError> {
    match value {
        LiteralValue::Boolean(true) => out.push("true"),
        LiteralValue::Boolean(false) => out.push("false"),
        LiteralValue::Integer(value) => {
            let text = if style == IntegerFormat::Hexadecimal && *value >= 0 {
                format!("0x{value:X}")
            } else {
                value.to_string()
            };
            out.push(&text)
        }
        LiteralValue::Number(value) => {
            let text = if value.is_sign_negative() && *value == 0.0 {
                "-0.0".to_owned()
            } else {
                value.to_string()
            };
            out.push(&text)
        }
        LiteralValue::String(value) => quoted(out, value),
    }
}

// Byte-wise escaping also preserves arbitrary UTF-8 text exactly. Lua 5.1
// understands three-digit decimal escapes; no Lua-version-specific \u syntax.
// Escaping all non-ASCII bytes keeps control/separator characters out of source
// and prevents an input string from introducing a physical annotation line.
fn quoted(out: &mut Output, value: &str) -> Result<(), LiteralError> {
    if value.len() > MAX_TEXT_BYTES {
        return Err(LiteralError::InputLimit);
    }
    out.push("\"")?;
    for byte in value.bytes() {
        match byte {
            b'"' => out.push("\\\"")?,
            b'\\' => out.push("\\\\")?,
            0x20..=0x7e => {
                let mut buffer = [0; 4];
                out.push(char::from(byte).encode_utf8(&mut buffer))?;
            }
            _ => out.push(&format!("\\{byte:03}"))?,
        }
    }
    out.push("\"")
}

impl LiteralBridge for LiteralRenderer {
    fn render(&self, request: &Request) -> Result<String, LiteralError> {
        request.validate()?;
        let renderer = Self::new(request.max_output_bytes)?;
        match &request.input {
            LiteralInput::Events(values) => renderer.render_events(values),
            LiteralInput::CVars(values) => renderer.render_cvars(values),
            LiteralInput::Enums { enums, constants } => renderer.render_enums(enums, constants),
        }
    }
}

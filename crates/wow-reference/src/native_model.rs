//! Typed, source-borrowing normalization of native APIDocumentation registrations.
//!
//! Follows Ketho's system/ScriptObject and table-Type dispatch. The full raw tree
//! remains reachable; this lane never treats successful parsing as absence proof.
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

use crate::native::{DocumentationDocument, RawKey, RawKind, RawValue, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ModelErrorCode {
    ExpectedTable,
    DuplicateKey,
    ExpectedArray,
    MissingField,
    WrongType,
    UnsupportedSystem,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModelError {
    pub code: ModelErrorCode,
    pub span: Span,
}
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "native documentation normalization rejected: {:?}",
            self.code
        )
    }
}
impl std::error::Error for ModelError {}
type Result<T> = std::result::Result<T, ModelError>;
fn error(code: ModelErrorCode, value: &RawValue) -> ModelError {
    ModelError {
        code,
        span: value.span,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemOwner<'a> {
    Global,
    Namespace(&'a str),
    ScriptObject(&'a str),
}

/// Ordered, unresolved reference types; rendering syntax is a later projection.
#[derive(Clone, Debug)]
pub struct FieldFact<'a> {
    pub name: &'a str,
    pub type_name: &'a str,
    pub inner_type: Option<&'a str>,
    pub nilable: Option<bool>,
    pub default: Option<&'a RawValue>,
    pub stride_index: Option<&'a RawValue>,
    pub raw: &'a RawValue,
}
#[derive(Clone, Debug)]
pub struct CallableFact<'a> {
    pub name: &'a str,
    pub documentation: Option<Vec<&'a str>>,
    pub arguments: Vec<FieldFact<'a>>,
    pub returns: Vec<FieldFact<'a>>,
    pub raw: &'a RawValue,
}
#[derive(Clone, Debug)]
pub struct EventFact<'a> {
    pub name: &'a str,
    pub literal_name: &'a str,
    pub payload: Vec<FieldFact<'a>>,
    pub raw: &'a RawValue,
}
#[derive(Clone, Debug)]
pub struct ValueFact<'a> {
    pub name: &'a str,
    pub value: &'a RawValue,
    pub raw: &'a RawValue,
}

#[derive(Clone, Debug)]
pub enum TableFact<'a> {
    Structure {
        name: &'a str,
        fields: Vec<FieldFact<'a>>,
        raw: &'a RawValue,
    },
    Callback {
        name: &'a str,
        arguments: Vec<FieldFact<'a>>,
        returns: Vec<FieldFact<'a>>,
        raw: &'a RawValue,
    },
    Enumeration {
        name: &'a str,
        values: Vec<ValueFact<'a>>,
        raw: &'a RawValue,
    },
    Constants {
        name: &'a str,
        values: Vec<ValueFact<'a>>,
        raw: &'a RawValue,
    },
    Unsupported {
        name: &'a str,
        type_name: &'a str,
        raw: &'a RawValue,
    },
}
#[derive(Clone, Debug)]
pub struct SystemFacts<'a> {
    pub owner: SystemOwner<'a>,
    pub name: Option<&'a str>,
    pub environment: Option<&'a str>,
    pub functions: Vec<CallableFact<'a>>,
    pub events: Vec<EventFact<'a>>,
    pub tables: Vec<TableFact<'a>>,
    pub raw: &'a RawValue,
    pub registration_ordinal: usize,
}

/// Successful raw evaluation is distinct from successful schema normalization.
/// Failed registrations stay in `source`; no generation or coverage is fabricated.
pub struct NormalizedDocument<'a> {
    pub source: &'a DocumentationDocument,
    pub systems: Vec<std::result::Result<SystemFacts<'a>, ModelError>>,
}

pub fn normalize_document(source: &DocumentationDocument) -> NormalizedDocument<'_> {
    let systems = source
        .registrations()
        .iter()
        .map(|registration| normalize_system(&registration.value, registration.ordinal))
        .collect();
    NormalizedDocument { source, systems }
}

/// Checked object access. Duplicates and mixed table shapes cannot be overwritten.
pub fn object(value: &RawValue) -> Result<BTreeMap<&str, &RawValue>> {
    let fields = value
        .fields()
        .ok_or_else(|| error(ModelErrorCode::ExpectedTable, value))?;
    let mut result = BTreeMap::new();
    for field in fields {
        let RawKey::Name(ref name) = field.key else {
            return Err(error(ModelErrorCode::WrongType, &field.value));
        };
        if result.insert(name.as_str(), &field.value).is_some() {
            return Err(error(ModelErrorCode::DuplicateKey, &field.value));
        }
    }
    Ok(result)
}

/// Semantic array order is numeric index, not constructor enumeration order.
/// Missing/nil slots, nonpositive/duplicate indices and map fields are rejected.
pub fn array(value: &RawValue) -> Result<Vec<&RawValue>> {
    let fields = value
        .fields()
        .ok_or_else(|| error(ModelErrorCode::ExpectedArray, value))?;
    let mut result = BTreeMap::new();
    for field in fields {
        let RawKey::Index(index) = field.key else {
            return Err(error(ModelErrorCode::ExpectedArray, &field.value));
        };
        if result.insert(index, &field.value).is_some() {
            return Err(error(ModelErrorCode::DuplicateKey, &field.value));
        }
    }
    for (position, (&index, item)) in result.iter().enumerate() {
        if index != (position + 1) as u64 || matches!(item.kind, RawKind::Nil) {
            return Err(error(ModelErrorCode::ExpectedArray, item));
        }
    }
    Ok(result.into_values().collect())
}
fn text(value: &RawValue) -> Result<&str> {
    match &value.kind {
        RawKind::String(s) => Ok(s),
        _ => Err(error(ModelErrorCode::WrongType, value)),
    }
}
fn required<'a>(
    map: &BTreeMap<&str, &'a RawValue>,
    name: &str,
    parent: &RawValue,
) -> Result<&'a RawValue> {
    map.get(name)
        .copied()
        .ok_or_else(|| error(ModelErrorCode::MissingField, parent))
}
fn required_text<'a>(
    map: &BTreeMap<&str, &'a RawValue>,
    name: &str,
    parent: &RawValue,
) -> Result<&'a str> {
    let value = required(map, name, parent)?;
    let s = text(value)?;
    if s.is_empty() {
        return Err(error(ModelErrorCode::WrongType, value));
    }
    Ok(s)
}
fn optional_text<'a>(map: &BTreeMap<&str, &'a RawValue>, name: &str) -> Result<Option<&'a str>> {
    map.get(name).map(|v| text(v)).transpose()
}
fn collection<'a>(map: &BTreeMap<&str, &'a RawValue>, name: &str) -> Result<Vec<&'a RawValue>> {
    map.get(name)
        .map(|v| array(v))
        .transpose()
        .map(Option::unwrap_or_default)
}
fn fields<'a>(map: &BTreeMap<&str, &'a RawValue>, name: &str) -> Result<Vec<FieldFact<'a>>> {
    collection(map, name)?.into_iter().map(field).collect()
}
fn field(value: &RawValue) -> Result<FieldFact<'_>> {
    let map = object(value)?;
    let nilable = map
        .get("Nilable")
        .map(|v| match v.kind {
            RawKind::Boolean(b) => Ok(b),
            _ => Err(error(ModelErrorCode::WrongType, v)),
        })
        .transpose()?;
    Ok(FieldFact {
        name: required_text(&map, "Name", value)?,
        type_name: required_text(&map, "Type", value)?,
        inner_type: optional_text(&map, "InnerType")?,
        nilable,
        default: map.get("Default").copied(),
        stride_index: map.get("StrideIndex").copied(),
        raw: value,
    })
}
fn callable(value: &RawValue) -> Result<CallableFact<'_>> {
    let map = object(value)?;
    if optional_text(&map, "Type")?.is_some_and(|t| t != "Function" && t != "CallbackType") {
        return Err(error(ModelErrorCode::WrongType, value));
    }
    let documentation = map
        .get("Documentation")
        .map(|v| array(v)?.into_iter().map(text).collect::<Result<Vec<_>>>())
        .transpose()?;
    Ok(CallableFact {
        name: required_text(&map, "Name", value)?,
        documentation,
        arguments: fields(&map, "Arguments")?,
        returns: fields(&map, "Returns")?,
        raw: value,
    })
}
fn event(value: &RawValue) -> Result<EventFact<'_>> {
    let map = object(value)?;
    Ok(EventFact {
        name: required_text(&map, "Name", value)?,
        literal_name: required_text(&map, "LiteralName", value)?,
        payload: fields(&map, "Payload")?,
        raw: value,
    })
}
fn values<'a>(map: &BTreeMap<&str, &'a RawValue>, key: &str) -> Result<Vec<ValueFact<'a>>> {
    collection(map, "Fields")?
        .into_iter()
        .map(|v| {
            let fields = object(v)?;
            Ok(ValueFact {
                name: required_text(&fields, "Name", v)?,
                value: required(&fields, key, v)?,
                raw: v,
            })
        })
        .collect()
}
fn table(value: &RawValue) -> Result<TableFact<'_>> {
    let map = object(value)?;
    let name = required_text(&map, "Name", value)?;
    let type_name = required_text(&map, "Type", value)?;
    Ok(match type_name {
        "Structure" => TableFact::Structure {
            name,
            fields: fields(&map, "Fields")?,
            raw: value,
        },
        "CallbackType" => TableFact::Callback {
            name,
            arguments: fields(&map, "Arguments")?,
            returns: fields(&map, "Returns")?,
            raw: value,
        },
        "Enumeration" => TableFact::Enumeration {
            name,
            values: values(&map, "EnumValue")?,
            raw: value,
        },
        "Constants" => TableFact::Constants {
            name,
            values: values(&map, "Value")?,
            raw: value,
        },
        _ => TableFact::Unsupported {
            name,
            type_name,
            raw: value,
        },
    })
}
fn normalize_system(value: &RawValue, ordinal: usize) -> Result<SystemFacts<'_>> {
    let map = object(value)?;
    let name = optional_text(&map, "Name")?;
    let namespace = optional_text(&map, "Namespace")?;
    let owner = match optional_text(&map, "Type")? {
        Some("ScriptObject") => {
            if namespace.is_some() {
                return Err(error(ModelErrorCode::UnsupportedSystem, value));
            }
            SystemOwner::ScriptObject(required_text(&map, "Name", value)?)
        }
        Some("System") if name.is_some_and(|v| !v.is_empty()) => match namespace {
            Some(n) => SystemOwner::Namespace(n),
            None => SystemOwner::Global,
        },
        None if name.is_none()
            && map.contains_key("Tables")
            && !map.contains_key("Functions")
            && !map.contains_key("Events")
            && namespace.is_none() =>
        {
            SystemOwner::Global
        }
        _ => return Err(error(ModelErrorCode::UnsupportedSystem, value)),
    };
    Ok(SystemFacts {
        owner,
        name,
        environment: optional_text(&map, "Environment")?,
        functions: collection(&map, "Functions")?
            .into_iter()
            .map(callable)
            .collect::<Result<_>>()?,
        events: collection(&map, "Events")?
            .into_iter()
            .map(event)
            .collect::<Result<_>>()?,
        tables: collection(&map, "Tables")?
            .into_iter()
            .map(table)
            .collect::<Result<_>>()?,
        raw: value,
        registration_ordinal: ordinal,
    })
}

//! Source-bound scalar resolution for Ketho's Enum/Constants/default lane.
//!
//! Unlike the donor's runtime resource loading, this resolver never executes Lua
//! or imports a global environment. Only unique declarations from the explicitly
//! selected document corpus can resolve references. Raw expressions remain intact.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use crate::native::{AdditiveOp, DocumentationDocument, RawKind, RawValue, Span};
use crate::native_model::{SystemFacts, TableFact};

const MAX_STEPS: usize = 4096;
const MAX_DEPTH: usize = 48;
const MAX_DEFINITIONS: usize = 65_536;
const MAX_EXACT_INTEGER: i64 = 9_007_199_254_740_991;

/// Numeric lexemes and wide string values remain distinct. Arithmetic produces
/// an exact integer lexeme only; it never coerces strings or rounds through f64.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ScalarValue {
    Boolean(bool),
    Number(String),
    String(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScalarEvidence {
    pub path: String,
    pub sha256: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ResolvedScalar {
    pub value: ScalarValue,
    /// All participating source values, including transitive dependencies.
    pub evidence: Vec<ScalarEvidence>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ScalarError {
    InvalidSource,
    UnresolvedReference,
    Conflict,
    Cycle,
    UnsupportedValue,
    NonIntegralArithmetic,
    OutOfRange,
    Limit,
    Cancelled,
}
impl fmt::Display for ScalarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native scalar resolution failed: {self:?}")
    }
}
impl std::error::Error for ScalarError {}

#[derive(Clone, Copy)]
struct Definition<'a> {
    document: &'a DocumentationDocument,
    value: &'a RawValue,
    declared_type: Option<&'a str>,
}

/// Immutable reference-owned data index. The caller filters environments before
/// constructing it; it cannot discover another profile/source to fill a gap.
pub struct ScalarCatalog<'a> {
    revision: &'a str,
    definitions: BTreeMap<Vec<String>, Vec<Definition<'a>>>,
    conflicting_groups: BTreeSet<Vec<String>>,
    enum_names: BTreeSet<&'a str>,
}

impl<'a> ScalarCatalog<'a> {
    pub fn new(
        revision: &'a str,
        systems: &[(&'a DocumentationDocument, &SystemFacts<'a>)],
    ) -> Result<Self, ScalarError> {
        let mut result = Self {
            revision,
            definitions: BTreeMap::new(),
            conflicting_groups: BTreeSet::new(),
            enum_names: BTreeSet::new(),
        };
        let mut groups = BTreeSet::new();
        let mut count = 0usize;
        for (document, system) in systems {
            if document.revision() != revision {
                return Err(ScalarError::InvalidSource);
            }
            for table in &system.tables {
                let (root, name, values) = match table {
                    TableFact::Enumeration { name, values, .. } => {
                        result.enum_names.insert(*name);
                        ("Enum", *name, values)
                    }
                    TableFact::Constants { name, values, .. } => ("Constants", *name, values),
                    _ => continue,
                };
                let group = vec![root.to_owned(), name.to_owned()];
                if !groups.insert(group.clone()) {
                    result.conflicting_groups.insert(group);
                }
                for value in values {
                    count += 1;
                    if count > MAX_DEFINITIONS {
                        return Err(ScalarError::Limit);
                    }
                    result
                        .definitions
                        .entry(vec![root.into(), name.into(), value.name.into()])
                        .or_default()
                        .push(Definition {
                            document,
                            value: value.value,
                            // EnumValue is already a scalar, not an enum-member
                            // name. Only Constants Values use Type="AnEnum".
                            declared_type: if root == "Constants" {
                                value.type_name
                            } else {
                                None
                            },
                        });
                }
            }
        }
        Ok(result)
    }

    pub fn resolve(
        &self,
        document: &DocumentationDocument,
        value: &RawValue,
        declared_type: Option<&str>,
        cancelled: &AtomicBool,
    ) -> Result<ResolvedScalar, ScalarError> {
        if document.revision() != self.revision {
            return Err(ScalarError::InvalidSource);
        }
        let mut state = State {
            remaining: MAX_STEPS,
            active: BTreeSet::new(),
            evidence: BTreeMap::new(),
            cancelled,
        };
        let value = self.value(document, value, declared_type, 0, &mut state)?;
        Ok(ResolvedScalar {
            value,
            evidence: state.evidence.into_values().collect(),
        })
    }

    fn value(
        &self,
        document: &DocumentationDocument,
        raw: &RawValue,
        declared_type: Option<&str>,
        depth: usize,
        state: &mut State<'_>,
    ) -> Result<ScalarValue, ScalarError> {
        state.tick(depth)?;
        state.evidence.insert(
            (document.path().to_owned(), raw.span.start, raw.span.end),
            ScalarEvidence {
                path: document.path().into(),
                sha256: document.sha256().into(),
                span: raw.span,
            },
        );
        match &raw.kind {
            RawKind::Boolean(value) => Ok(ScalarValue::Boolean(*value)),
            RawKind::Number(value) => Ok(ScalarValue::Number(value.clone())),
            RawKind::String(value) => {
                // Blizzard's typed constant descriptors use e.g. Type="Mode",
                // Value="First" to name an enum member, not a runtime string.
                if let Some(name) = declared_type.filter(|name| self.enum_names.contains(name)) {
                    self.reference(
                        &["Enum".into(), name.into(), value.clone()],
                        depth + 1,
                        state,
                    )
                } else if declared_type.is_none_or(|name| {
                    matches!(
                        name,
                        "string" | "cstring" | "number" | "luaIndex" | "bool" | "boolean"
                    )
                }) {
                    Ok(ScalarValue::String(value.clone()))
                } else {
                    // A named descriptor type may be an enum missing from this
                    // corpus. Do not mispublish its member name as a string.
                    Err(ScalarError::UnresolvedReference)
                }
            }
            RawKind::Reference(path) => self.reference(path, depth + 1, state),
            RawKind::UnresolvedName(_) => Err(ScalarError::UnresolvedReference),
            RawKind::BinaryExpression { op, left, right } => {
                let left = self.value(document, left, None, depth + 1, state)?;
                let right = self.value(document, right, None, depth + 1, state)?;
                let left = exact_integer(&left)?;
                let right = exact_integer(&right)?;
                let value = match op {
                    AdditiveOp::Add => left.checked_add(right),
                    AdditiveOp::Subtract => left.checked_sub(right),
                }
                .filter(|value| (-MAX_EXACT_INTEGER..=MAX_EXACT_INTEGER).contains(value))
                .ok_or(ScalarError::OutOfRange)?;
                Ok(ScalarValue::Number(value.to_string()))
            }
            _ => Err(ScalarError::UnsupportedValue),
        }
    }

    fn reference(
        &self,
        path: &[String],
        depth: usize,
        state: &mut State<'_>,
    ) -> Result<ScalarValue, ScalarError> {
        state.tick(depth)?;
        if path.len() != 3 || !matches!(path[0].as_str(), "Enum" | "Constants") {
            return Err(ScalarError::UnresolvedReference);
        }
        if self.conflicting_groups.contains(&path[..2]) {
            return Err(ScalarError::Conflict);
        }
        let entries = self
            .definitions
            .get(path)
            .ok_or(ScalarError::UnresolvedReference)?;
        if entries.len() != 1 {
            return Err(ScalarError::Conflict);
        }
        if !state.active.insert(path.to_vec()) {
            return Err(ScalarError::Cycle);
        }
        let entry = entries[0];
        let result = self.value(
            entry.document,
            entry.value,
            entry.declared_type,
            depth + 1,
            state,
        );
        state.active.remove(path);
        result
    }
}

struct State<'a> {
    remaining: usize,
    active: BTreeSet<Vec<String>>,
    evidence: BTreeMap<(String, usize, usize), ScalarEvidence>,
    cancelled: &'a AtomicBool,
}
impl State<'_> {
    fn tick(&mut self, depth: usize) -> Result<(), ScalarError> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(ScalarError::Cancelled);
        }
        if depth > MAX_DEPTH {
            return Err(ScalarError::Limit);
        }
        self.remaining = self.remaining.checked_sub(1).ok_or(ScalarError::Limit)?;
        Ok(())
    }
}

fn exact_integer(value: &ScalarValue) -> Result<i64, ScalarError> {
    let ScalarValue::Number(text) = value else {
        return Err(ScalarError::NonIntegralArithmetic);
    };
    let (negative, magnitude) = text
        .strip_prefix('-')
        .map_or((false, text.as_str()), |v| (true, v));
    let number = if let Some(hex) = magnitude
        .strip_prefix("0x")
        .or_else(|| magnitude.strip_prefix("0X"))
    {
        i64::from_str_radix(hex, 16)
    } else {
        magnitude.parse::<i64>()
    }
    .map_err(|_| ScalarError::NonIntegralArithmetic)?;
    let number = if negative {
        if number == 0 {
            return Err(ScalarError::NonIntegralArithmetic);
        }
        number.checked_neg().ok_or(ScalarError::OutOfRange)?
    } else {
        number
    };
    if !(-MAX_EXACT_INTEGER..=MAX_EXACT_INTEGER).contains(&number) {
        return Err(ScalarError::OutOfRange);
    }
    Ok(number)
}

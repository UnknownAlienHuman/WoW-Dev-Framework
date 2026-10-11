//! Owned observations from the same-session LuaSignature index. No source scan.
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::sync::atomic::AtomicBool;

use emmylua_code_analysis::{
    AsyncState, LuaArrayLen, LuaSignature, LuaType, LuaUnionType, SignatureReturnStatus,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::declaration::DeclarationBudget;
use super::{FunctionCallBudget, SourceFunctionKind, invalid, resource_budget, visit};
use crate::bindings::checkpoint;
use crate::references::EmmyMemberCallResult;

/// Analyzer-observed typed shape, not proof of source annotations or runtime types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PhysicalFunctionSignatureRecipe {
    #[serde(rename = "wow-emmy/physical-function-indexed-types/1")]
    IndexedTypesV1,
}
impl<'de> Deserialize<'de> for PhysicalFunctionSignatureRecipe {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let recipe = String::deserialize(deserializer)?;
        match recipe.as_str() {
            "wow-emmy/physical-function-indexed-types/1" => Ok(Self::IndexedTypesV1),
            _ => Err(serde::de::Error::custom(
                "unsupported physical-function signature recipe",
            )),
        }
    }
}
impl PhysicalFunctionSignatureRecipe {
    pub const fn wire_name(self) -> &'static str {
        "wow-emmy/physical-function-indexed-types/1"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSignatureUnavailable {
    ChunkHasNoSignature,
    MissingSignatureIndex,
    GenericSignature,
    CallableOverloads,
    VariadicSignature,
    UnmatchedParameterDocuments,
    UnresolvedReturns,
    UnresolvedOrUnsupportedType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SourceSignatureTypeUnavailable {
    MissingParameterType = 1,
    UnresolvedType = 2,
    DynamicType = 3,
    GenericType = 4,
    NominalType = 5,
    CallableType = 6,
    LiteralType = 7,
    ObjectType = 8,
    VariadicType = 9,
    UnsupportedNativeType = 10,
    EmptyUnion = 11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SourceSignatureArrayLength {
    Unspecified,
    Maximum(i64),
}

/// A finite reviewed vocabulary. Unavailable leaves never become Any.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceSignatureType {
    Nil,
    Boolean,
    Integer,
    Number,
    String,
    Thread,
    Userdata,
    Never,
    Array {
        length: SourceSignatureArrayLength,
        element: Box<SourceSignatureType>,
    },
    Tuple {
        elements: Vec<SourceSignatureType>,
    },
    Union {
        members: Vec<SourceSignatureType>,
    },
    Unavailable {
        reason: SourceSignatureTypeUnavailable,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSignatureParameter {
    type_shape: SourceSignatureType,
    /// Exact backend flag; not parameter omissibility or source annotation proof.
    #[serde(skip_serializing_if = "Option::is_none")]
    backend_nullable: Option<bool>,
}
impl SourceSignatureParameter {
    pub fn type_shape(&self) -> &SourceSignatureType {
        &self.type_shape
    }
    pub const fn backend_nullable(&self) -> Option<bool> {
        self.backend_nullable
    }
}

/// One ordered native return row. Empty remains empty, never a synthesized Nil.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSignatureReturn {
    types: Vec<SourceSignatureType>,
}
impl SourceSignatureReturn {
    pub fn types(&self) -> &[SourceSignatureType] {
        &self.types
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSignatureReturnResolution {
    Unresolved,
    DocumentResolved,
    InferenceResolved,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSignatureAsyncState {
    Unspecified,
    Async,
    Sync,
}

/// Comparable only through SourceFunctionSignature::comparable().
/// No names, locations, upstream IDs, resolution labels or generation bindings.
/// Parameters are ordinal; primary return row precedes ordered alternative rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSignatureComparable {
    colon_definition: bool,
    parameters: Vec<SourceSignatureParameter>,
    returns: Vec<SourceSignatureReturn>,
}
impl SourceSignatureComparable {
    pub const fn colon_definition(&self) -> bool {
        self.colon_definition
    }
    pub fn parameters(&self) -> &[SourceSignatureParameter] {
        &self.parameters
    }
    pub fn returns(&self) -> &[SourceSignatureReturn] {
        &self.returns
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum SignatureObservation {
    Unavailable {
        reason: SourceSignatureUnavailable,
    },
    Observed {
        types: SourceSignatureComparable,
        return_resolution: SourceSignatureReturnResolution,
        async_state: SourceSignatureAsyncState,
        vararg: bool,
        generic_parameter_count: u64,
        callable_overload_count: u64,
        unmatched_parameter_document_count: u64,
        unavailable_reasons: Vec<SourceSignatureUnavailable>,
    },
}

/// Serialize-only, fully owned, immutable output of the held native index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceFunctionSignature {
    #[serde(flatten)]
    observation: SignatureObservation,
}
impl SourceFunctionSignature {
    /// Eligibility is this finite static recipe, never runtime/no-return proof.
    pub fn comparable(&self) -> Option<&SourceSignatureComparable> {
        match &self.observation {
            SignatureObservation::Observed {
                types,
                unavailable_reasons,
                ..
            } if unavailable_reasons.is_empty() => Some(types),
            _ => None,
        }
    }
    /// Retains supported fragments and explicit unavailable leaves on Partial.
    pub fn observed_types(&self) -> Option<&SourceSignatureComparable> {
        match &self.observation {
            SignatureObservation::Observed { types, .. } => Some(types),
            SignatureObservation::Unavailable { .. } => None,
        }
    }
    pub fn unavailable_reasons(&self) -> &[SourceSignatureUnavailable] {
        match &self.observation {
            SignatureObservation::Observed {
                unavailable_reasons,
                ..
            } => unavailable_reasons,
            SignatureObservation::Unavailable { reason } => std::slice::from_ref(reason),
        }
    }
    pub const fn return_resolution(&self) -> Option<SourceSignatureReturnResolution> {
        match &self.observation {
            SignatureObservation::Observed {
                return_resolution, ..
            } => Some(*return_resolution),
            SignatureObservation::Unavailable { .. } => None,
        }
    }
    pub const fn async_state(&self) -> Option<SourceSignatureAsyncState> {
        match &self.observation {
            SignatureObservation::Observed { async_state, .. } => Some(*async_state),
            SignatureObservation::Unavailable { .. } => None,
        }
    }
    pub const fn vararg(&self) -> Option<bool> {
        match &self.observation {
            SignatureObservation::Observed { vararg, .. } => Some(*vararg),
            SignatureObservation::Unavailable { .. } => None,
        }
    }
    pub const fn generic_parameter_count(&self) -> Option<u64> {
        match &self.observation {
            SignatureObservation::Observed {
                generic_parameter_count,
                ..
            } => Some(*generic_parameter_count),
            SignatureObservation::Unavailable { .. } => None,
        }
    }
    pub const fn callable_overload_count(&self) -> Option<u64> {
        match &self.observation {
            SignatureObservation::Observed {
                callable_overload_count,
                ..
            } => Some(*callable_overload_count),
            SignatureObservation::Unavailable { .. } => None,
        }
    }
    pub const fn unmatched_parameter_document_count(&self) -> Option<u64> {
        match &self.observation {
            SignatureObservation::Observed {
                unmatched_parameter_document_count,
                ..
            } => Some(*unmatched_parameter_document_count),
            SignatureObservation::Unavailable { .. } => None,
        }
    }
    pub(super) const fn chunk() -> Self {
        Self {
            observation: SignatureObservation::Unavailable {
                reason: SourceSignatureUnavailable::ChunkHasNoSignature,
            },
        }
    }

    pub(super) fn validate(
        &self,
        kind: SourceFunctionKind,
        limits: FunctionCallBudget,
        stop: Option<&AtomicBool>,
    ) -> EmmyMemberCallResult<()> {
        if let Some(stop) = stop {
            checkpoint(stop)?;
        }
        if kind == SourceFunctionKind::Chunk {
            return if self == &Self::chunk() {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        match &self.observation {
            SignatureObservation::Unavailable {
                reason: SourceSignatureUnavailable::MissingSignatureIndex,
            } => Ok(()),
            SignatureObservation::Unavailable { .. } => Err(invalid()),
            SignatureObservation::Observed {
                types,
                return_resolution,
                vararg,
                generic_parameter_count,
                callable_overload_count,
                unmatched_parameter_document_count,
                unavailable_reasons,
                ..
            } => {
                // Structural validation only; no new execution ledger or allowance.
                let mut population = 0_u64;
                let mut step = || {
                    if let Some(stop) = stop {
                        checkpoint(stop)?;
                    }
                    population = population.checked_add(1).ok_or_else(invalid)?;
                    if population > limits.max_ast_visits() {
                        return Err(invalid());
                    }
                    Ok(())
                };
                let complete_types = validate_types(types, &mut step)?;
                let expected = reasons(
                    *generic_parameter_count,
                    *callable_overload_count,
                    *vararg,
                    *unmatched_parameter_document_count,
                    *return_resolution,
                    complete_types,
                );
                if *unavailable_reasons != expected {
                    return Err(invalid());
                }
                Ok(())
            }
        }
    }
}

fn tick(
    visits: &mut u64,
    limits: FunctionCallBudget,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<()> {
    checkpoint(stop)?;
    visit(visits, limits.max_ast_visits())
}
fn count(value: usize) -> EmmyMemberCallResult<u64> {
    u64::try_from(value).map_err(|_| invalid())
}
fn depth_guard(depth: usize) -> EmmyMemberCallResult<()> {
    if depth > super::MAX_SCOPE_DEPTH {
        return Err(resource_budget(
            "signature type depth",
            depth as u128,
            super::MAX_SCOPE_DEPTH as u64,
            None,
        ));
    }
    Ok(())
}

/// Fixed typed framing is used ONLY to order/deduplicate union members.
/// This private key is not a signature fingerprint or public identity.
fn type_key(
    value: &SourceSignatureType,
    depth: usize,
    step: &mut impl FnMut() -> EmmyMemberCallResult<()>,
) -> EmmyMemberCallResult<([u8; 32], bool)> {
    step()?;
    depth_guard(depth)?;
    let mut hash = Sha256::new();
    hash.update(b"wow-emmy/physical-function-indexed-types/1/union-order\0");
    let mut complete = true;
    let tag: u8 = match value {
        SourceSignatureType::Nil => 1,
        SourceSignatureType::Boolean => 2,
        SourceSignatureType::Integer => 3,
        SourceSignatureType::Number => 4,
        SourceSignatureType::String => 5,
        SourceSignatureType::Thread => 6,
        SourceSignatureType::Userdata => 7,
        SourceSignatureType::Never => 8,
        SourceSignatureType::Array { .. } => 9,
        SourceSignatureType::Tuple { .. } => 10,
        SourceSignatureType::Union { .. } => 11,
        SourceSignatureType::Unavailable { .. } => 12,
    };
    hash.update([tag]);
    match value {
        SourceSignatureType::Array { length, element } => {
            match length {
                SourceSignatureArrayLength::Unspecified => hash.update([0]),
                SourceSignatureArrayLength::Maximum(length) => {
                    hash.update([1]);
                    hash.update(length.to_be_bytes());
                }
            }
            let (key, supported) = type_key(element, depth + 1, step)?;
            hash.update(key);
            complete &= supported;
        }
        SourceSignatureType::Tuple { elements } => {
            hash.update(count(elements.len())?.to_be_bytes());
            for element in elements {
                let (key, supported) = type_key(element, depth + 1, step)?;
                hash.update(key);
                complete &= supported;
            }
        }
        SourceSignatureType::Union { members } => {
            if members.len() < 2 {
                return Err(invalid());
            }
            hash.update(count(members.len())?.to_be_bytes());
            let mut previous = None;
            for member in members {
                if matches!(member, SourceSignatureType::Union { .. }) {
                    return Err(invalid());
                }
                let (key, supported) = type_key(member, depth + 1, step)?;
                if previous.is_some_and(|previous| previous >= key) {
                    return Err(invalid());
                }
                previous = Some(key);
                hash.update(key);
                complete &= supported;
            }
        }
        SourceSignatureType::Unavailable { reason } => {
            hash.update([*reason as u8]);
            complete = false;
        }
        _ => {}
    }
    Ok((hash.finalize().into(), complete))
}

fn validate_types(
    types: &SourceSignatureComparable,
    step: &mut impl FnMut() -> EmmyMemberCallResult<()>,
) -> EmmyMemberCallResult<bool> {
    step()?;
    if types.returns.is_empty() {
        return Err(invalid());
    }
    let mut complete = true;
    for parameter in &types.parameters {
        step()?;
        let missing = matches!(
            parameter.type_shape,
            SourceSignatureType::Unavailable {
                reason: SourceSignatureTypeUnavailable::MissingParameterType
            }
        );
        if parameter.backend_nullable.is_none() != missing {
            return Err(invalid());
        }
        complete &= type_key(&parameter.type_shape, 1, step)?.1;
    }
    for row in &types.returns {
        step()?;
        // Resolved empty primary/alternative rows are honest observed empty rows.
        for value in &row.types {
            complete &= type_key(value, 1, step)?.1;
        }
    }
    Ok(complete)
}

fn reasons(
    generics: u64,
    overloads: u64,
    vararg: bool,
    unmatched_docs: u64,
    returns: SourceSignatureReturnResolution,
    complete_types: bool,
) -> Vec<SourceSignatureUnavailable> {
    let mut result = Vec::new();
    if generics != 0 {
        result.push(SourceSignatureUnavailable::GenericSignature);
    }
    if overloads != 0 {
        result.push(SourceSignatureUnavailable::CallableOverloads);
    }
    if vararg {
        result.push(SourceSignatureUnavailable::VariadicSignature);
    }
    if unmatched_docs != 0 {
        result.push(SourceSignatureUnavailable::UnmatchedParameterDocuments);
    }
    if returns == SourceSignatureReturnResolution::Unresolved {
        result.push(SourceSignatureUnavailable::UnresolvedReturns);
    }
    if !complete_types {
        result.push(SourceSignatureUnavailable::UnresolvedOrUnsupportedType);
    }
    result
}

fn insert_union_member(
    value: SourceSignatureType,
    members: &mut BTreeMap<[u8; 32], SourceSignatureType>,
    visits: &mut u64,
    limits: FunctionCallBudget,
    bytes: &mut DeclarationBudget,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<()> {
    tick(visits, limits, stop)?;
    if let SourceSignatureType::Union { members: nested } = value {
        for member in nested {
            insert_union_member(member, members, visits, limits, bytes, stop)?;
        }
        return Ok(());
    }
    let (key, _) = type_key(&value, 1, &mut || tick(visits, limits, stop))?;
    // Temporary index keys remain charged even if an equal member is discarded.
    bytes.charge(&key, stop)?;
    match members.entry(key) {
        Entry::Vacant(entry) => {
            entry.insert(value);
        }
        Entry::Occupied(entry) => {
            if entry.get() != &value {
                return Err(invalid());
            }
        }
    }
    checkpoint(stop)?;
    Ok(())
}

fn project_type(
    native: &LuaType,
    depth: usize,
    visits: &mut u64,
    limits: FunctionCallBudget,
    bytes: &mut DeclarationBudget,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<SourceSignatureType> {
    tick(visits, limits, stop)?;
    depth_guard(depth)?;
    use SourceSignatureType as T;
    use SourceSignatureTypeUnavailable as U;
    let value = match native {
        LuaType::Nil => T::Nil,
        LuaType::Boolean => T::Boolean,
        LuaType::Integer => T::Integer,
        LuaType::Number => T::Number,
        LuaType::String => T::String,
        LuaType::Thread => T::Thread,
        LuaType::Userdata => T::Userdata,
        LuaType::Never => T::Never,
        LuaType::Array(array) => {
            let length = match array.get_len() {
                LuaArrayLen::None => SourceSignatureArrayLength::Unspecified,
                LuaArrayLen::Max(maximum) => SourceSignatureArrayLength::Maximum(*maximum),
            };
            // Charge construction inputs before allocating the owned child.
            bytes.charge(&("array", length), stop)?;
            let element = project_type(array.get_base(), depth + 1, visits, limits, bytes, stop)?;
            return Ok(T::Array {
                length,
                element: Box::new(element),
            });
        }
        LuaType::Tuple(tuple) => {
            let mut elements = Vec::new();
            bytes.charge(
                &T::Tuple {
                    elements: Vec::new(),
                },
                stop,
            )?;
            for native in tuple.get_types() {
                let element = project_type(native, depth + 1, visits, limits, bytes, stop)?;
                bytes.charge(&0_u8, stop)?;
                elements.push(element);
            }
            return Ok(T::Tuple { elements });
        }
        LuaType::Union(union) => {
            bytes.charge(
                &T::Union {
                    members: Vec::new(),
                },
                stop,
            )?;
            let mut members = BTreeMap::new();
            let mut add = |native: &LuaType| {
                let value = project_type(native, depth + 1, visits, limits, bytes, stop)?;
                insert_union_member(value, &mut members, visits, limits, bytes, stop)
            };
            match union.as_ref() {
                LuaUnionType::Basic(basic) => {
                    for native in basic.iter() {
                        add(&native)?;
                    }
                }
                LuaUnionType::Nullable(native) => {
                    add(native)?;
                    add(&LuaType::Nil)?;
                }
                LuaUnionType::Multi(native) => {
                    for native in native {
                        add(native)?;
                    }
                }
            }
            match members.len() {
                0 => T::Unavailable {
                    reason: U::EmptyUnion,
                },
                1 => members.pop_first().ok_or_else(invalid)?.1,
                _ => {
                    let mut values = Vec::new();
                    for value in members.into_values() {
                        tick(visits, limits, stop)?;
                        bytes.charge(&0_u8, stop)?;
                        values.push(value);
                    }
                    return Ok(T::Union { members: values });
                }
            }
        }
        LuaType::Unknown | LuaType::SelfInfer => T::Unavailable {
            reason: U::UnresolvedType,
        },
        LuaType::Any | LuaType::Table | LuaType::Global => T::Unavailable {
            reason: U::DynamicType,
        },
        LuaType::Generic(_)
        | LuaType::TableGeneric(_)
        | LuaType::TplRef(_)
        | LuaType::StrTplRef(_) => T::Unavailable {
            reason: U::GenericType,
        },
        LuaType::Ref(_)
        | LuaType::Def(_)
        | LuaType::Instance(_)
        | LuaType::Namespace(_)
        | LuaType::ModuleRef(_) => T::Unavailable {
            reason: U::NominalType,
        },
        LuaType::Function | LuaType::DocFunction(_) | LuaType::Signature(_) => T::Unavailable {
            reason: U::CallableType,
        },
        LuaType::BooleanConst(_)
        | LuaType::StringConst(_)
        | LuaType::IntegerConst(_)
        | LuaType::FloatConst(_)
        | LuaType::DocStringConst(_)
        | LuaType::DocIntegerConst(_)
        | LuaType::DocBooleanConst(_) => T::Unavailable {
            reason: U::LiteralType,
        },
        LuaType::Object(_) | LuaType::TableConst(_) => T::Unavailable {
            reason: U::ObjectType,
        },
        LuaType::Variadic(_) => T::Unavailable {
            reason: U::VariadicType,
        },
        LuaType::Io
        | LuaType::Intersection(_)
        | LuaType::Call(_)
        | LuaType::MultiLineUnion(_)
        | LuaType::TypeGuard(_)
        | LuaType::Language(_)
        | LuaType::Conditional(_)
        | LuaType::Mapped(_) => T::Unavailable {
            reason: U::UnsupportedNativeType,
        },
    };
    bytes.charge(&value, stop)?;
    Ok(value)
}

pub(super) fn observe(
    native: Option<&LuaSignature>,
    visits: &mut u64,
    limits: FunctionCallBudget,
    bytes: &mut DeclarationBudget,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<SourceFunctionSignature> {
    tick(visits, limits, stop)?;
    let Some(native) = native else {
        let value = SourceFunctionSignature {
            observation: SignatureObservation::Unavailable {
                reason: SourceSignatureUnavailable::MissingSignatureIndex,
            },
        };
        bytes.charge(&value, stop)?;
        return Ok(value);
    };
    let mut types = SourceSignatureComparable {
        colon_definition: native.is_colon_define,
        parameters: Vec::new(),
        returns: Vec::new(),
    };
    bytes.charge(&types, stop)?;
    let mut matched_docs = 0_usize;
    // Exact ordinal association only; names are never read or joined.
    for position in 0..native.params.len() {
        tick(visits, limits, stop)?;
        let parameter = if let Some(doc) = native.param_docs.get(&position) {
            matched_docs = matched_docs.checked_add(1).ok_or_else(invalid)?;
            bytes.charge(&doc.nullable, stop)?;
            SourceSignatureParameter {
                type_shape: project_type(&doc.type_ref, 1, visits, limits, bytes, stop)?,
                backend_nullable: Some(doc.nullable),
            }
        } else {
            SourceSignatureParameter {
                type_shape: SourceSignatureType::Unavailable {
                    reason: SourceSignatureTypeUnavailable::MissingParameterType,
                },
                backend_nullable: None,
            }
        };
        bytes.charge(&parameter, stop)?;
        types.parameters.push(parameter);
    }
    let mut primary = SourceSignatureReturn { types: Vec::new() };
    bytes.charge(&primary, stop)?;
    for doc in &native.return_docs {
        let value = project_type(&doc.type_ref, 1, visits, limits, bytes, stop)?;
        bytes.charge(&0_u8, stop)?;
        primary.types.push(value);
    }
    bytes.charge(&primary, stop)?;
    types.returns.push(primary);
    for native_row in &native.return_overloads {
        tick(visits, limits, stop)?;
        let mut row = SourceSignatureReturn { types: Vec::new() };
        bytes.charge(&row, stop)?;
        for native in &native_row.type_refs {
            let value = project_type(native, 1, visits, limits, bytes, stop)?;
            bytes.charge(&0_u8, stop)?;
            row.types.push(value);
        }
        bytes.charge(&row, stop)?;
        types.returns.push(row);
    }
    let return_resolution = match native.resolve_return {
        SignatureReturnStatus::UnResolve => SourceSignatureReturnResolution::Unresolved,
        SignatureReturnStatus::DocResolve => SourceSignatureReturnResolution::DocumentResolved,
        SignatureReturnStatus::InferResolve => SourceSignatureReturnResolution::InferenceResolved,
    };
    let async_state = match native.async_state {
        AsyncState::None => SourceSignatureAsyncState::Unspecified,
        AsyncState::Async => SourceSignatureAsyncState::Async,
        AsyncState::Sync => SourceSignatureAsyncState::Sync,
    };
    let generic_parameter_count = count(native.generic_params.len())?;
    let callable_overload_count = count(native.overloads.len())?;
    let unmatched_parameter_document_count = count(
        native
            .param_docs
            .len()
            .checked_sub(matched_docs)
            .ok_or_else(invalid)?,
    )?;
    let complete_types = validate_types(&types, &mut || tick(visits, limits, stop))?;
    let unavailable_reasons = reasons(
        generic_parameter_count,
        callable_overload_count,
        native.is_vararg,
        unmatched_parameter_document_count,
        return_resolution,
        complete_types,
    );
    bytes.charge(&unavailable_reasons, stop)?;
    let value = SourceFunctionSignature {
        observation: SignatureObservation::Observed {
            types,
            return_resolution,
            async_state,
            vararg: native.is_vararg,
            generic_parameter_count,
            callable_overload_count,
            unmatched_parameter_document_count,
            unavailable_reasons,
        },
    };
    // Conservative cumulative retained-payload charge; dropped work stays charged.
    // The caller still owns the final combined report-byte gate.
    bytes.charge(&value, stop)?;
    checkpoint(stop)?;
    Ok(value)
}

//! Function scopes and call targets from an existing semantic session.
//! A target is an analyzer-observed concrete closure, not a runtime dispatch claim.
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use emmylua_code_analysis::{
    EmmyLuaAnalysis, LuaDeclId, LuaSemanticDeclId, LuaSignatureId, LuaType, SemanticDeclLevel,
    SemanticModel,
};
use emmylua_parser::{
    LuaAst, LuaAstNode, LuaAstToken, LuaCallExpr, LuaClosureExpr, LuaExpr, LuaLiteralToken,
    LuaNameExpr, LuaSyntaxId, LuaSyntaxKind, LuaVarExpr, NumberResult,
};
use serde::{Deserialize, Serialize};
use wow_core::SourceSpan;

use crate::bindings::{SymbolTarget, checkpoint};
use crate::references::{
    EmmyMemberCallError, EmmyMemberCallErrorCode, EmmyMemberCallResult, ast_span, canonical_id,
    semantic_model,
};

mod budget;
mod declaration;
mod hook_targets;
mod location_index;
mod names;
mod shape;
mod signature;
#[cfg(test)]
mod tests;
use crate::{EmmyBackendIdentity, LuaWorkspaceFile, LuaWorkspaceSnapshot};
pub use budget::FunctionCallBudget;
pub use declaration::{
    PhysicalFunctionDeclarationRecipe, SourceDeclaredParameter, SourceFunctionDeclaration,
    SourceFunctionDeclarationForm, SourceFunctionDeclarationUnavailable,
    SourceFunctionDeclarationUnsupported,
};
pub use hook_targets::{HookTargetRecipe, SourceHookTarget, SourceHookTargetForm};
use location_index::AccessLocationIndex;
pub use names::{SourceFunctionBindingName, SourceFunctionBindingState};
pub use shape::{
    FunctionShapeEvent, FunctionShapeNode, FunctionShapeToken, SourceFunctionShape,
    SourceFunctionShapeUnavailable, SourceFunctionShapeUnsupported,
};
pub use signature::{
    PhysicalFunctionSignatureRecipe, SourceFunctionSignature, SourceSignatureArrayLength,
    SourceSignatureAsyncState, SourceSignatureComparable, SourceSignatureParameter,
    SourceSignatureReturn, SourceSignatureReturnResolution, SourceSignatureType,
    SourceSignatureTypeUnavailable, SourceSignatureUnavailable,
};

pub const FUNCTION_CALL_PROFILE: &str = "wow-emmy/function-call-facts/7";
/// Explicit budget-bound sidecar; absent selection preserves profile7.
pub const FUNCTION_CALL_BUDGET_PROFILE: &str = "wow-emmy/function-call-facts/8";
/// Explicit global-access budget and local-constant custody profile (report10).
pub const FUNCTION_CALL_GLOBAL_BUDGET_PROFILE: &str = "wow-emmy/function-call-facts/10";
/// Opt-in exact local call-result custody for the named AceAddon consumer.
pub const FUNCTION_CALL_LOCAL_RESULT_PROFILE: &str = "wow-emmy/function-call-facts/11";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionCallEvidenceProfile {
    AceAddonLocalResultV1,
}

// Existing callable/call occurrence keys keep their original recipe. Added
// normalized call arguments/callable keys change report identity, not old occurrence keys.
const OCCURRENCE_PROFILE: &str = "wow-emmy/function-call-facts/1";
const MAX_FUNCTIONS: usize = 65_536;
const MAX_CALLS: usize = 65_536;
const MAX_CALL_ARGUMENTS: usize = 256;
const MAX_CALL_LITERAL_BYTES: usize = 4096;
const MAX_AST_VISITS: usize = 2_000_000;
const MAX_SCOPE_DEPTH: usize = 256;
const MAX_REPORT_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionKind {
    Chunk,
    Closure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceFunctionFact {
    fact_id: String,
    path: String,
    content_digest: String,
    kind: SourceFunctionKind,
    span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_function_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    declaration: Option<SourceFunctionDeclaration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shape: Option<SourceFunctionShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<SourceFunctionSignature>,
    /// Native syntax cache. Existing report/replay identities bind the original
    /// whole-file digest and function occurrence; their serialized bytes stay intact.
    #[serde(skip)]
    binding_name: names::BindingNameObservation,
}
impl SourceFunctionFact {
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
    pub const fn kind(&self) -> SourceFunctionKind {
        self.kind
    }
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
    pub fn parent_function_id(&self) -> Option<&str> {
        self.parent_function_id.as_deref()
    }
    /// Selected syntactic header only; never a complete semantic signature.
    pub fn declaration(&self) -> Option<&SourceFunctionDeclaration> {
        self.declaration.as_ref()
    }
    /// Selected full body syntax, without identifier/literal payloads or trivia.
    pub fn shape(&self) -> Option<&SourceFunctionShape> {
        self.shape.as_ref()
    }
    /// Selected typed observation from the same native signature index.
    pub fn signature(&self) -> Option<&SourceFunctionSignature> {
        self.signature.as_ref()
    }
    /// Recorded declaration syntax, never an inferred assignment/alias/runtime name.
    pub fn binding_name(&self) -> Option<&SourceFunctionBindingName> {
        self.binding_name.name()
    }
    pub fn binding_name_state(&self) -> SourceFunctionBindingState {
        self.binding_name.state()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SourceCallTarget {
    MainFunction {
        function_id: String,
    },
    LibraryFunction {
        target: SymbolTarget,
    },
    /// A signature need not identify an executable closure (for example a doc type).
    SignatureNotCaptured,
    /// Any/unknown/error: no fabricated endpoint and no negative claim.
    Unresolved,
    /// Union, callable object, documented function or another non-single-signature type.
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SourceCallLiteral {
    Nil,
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SourceCallConstant {
    Integer(i64),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceCallLocalConstant {
    name: String,
    declaration_span: SourceSpan,
    initializer_span: SourceSpan,
    value: SourceCallConstant,
}
impl SourceCallLocalConstant {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub const fn declaration_span(&self) -> SourceSpan {
        self.declaration_span
    }
    pub const fn initializer_span(&self) -> SourceSpan {
        self.initializer_span
    }
    pub fn value(&self) -> &SourceCallConstant {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceCallArgument {
    span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    literal: Option<SourceCallLiteral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_constant: Option<SourceCallLocalConstant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_target: Option<SymbolTarget>,
}
impl SourceCallArgument {
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
    pub fn literal(&self) -> Option<&SourceCallLiteral> {
        self.literal.as_ref()
    }
    pub fn local_constant(&self) -> Option<&SourceCallLocalConstant> {
        self.local_constant.as_ref()
    }
    pub fn reference_key(&self) -> Option<&str> {
        self.reference_key.as_deref()
    }
    pub fn reference_target(&self) -> Option<&SymbolTarget> {
        self.reference_target.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCallReceiver {
    key: String,
    target: SymbolTarget,
}
impl SourceCallReceiver {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub const fn target(&self) -> &SymbolTarget {
        &self.target
    }
}

/// Exact local declaration initialized by a bounded multiple-return call.
/// Result slots are zero-based; the current profile admits only slots 0 and 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceCallResultBinding {
    binding_id: String,
    path: String,
    content_digest: String,
    scope_function_id: String,
    declaration_span: SourceSpan,
    initializer_call_fact_id: String,
    result_slot: u8,
}

impl SourceCallResultBinding {
    pub fn binding_id(&self) -> &str {
        &self.binding_id
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
    pub fn scope_function_id(&self) -> &str {
        &self.scope_function_id
    }
    pub const fn declaration_span(&self) -> SourceSpan {
        self.declaration_span
    }
    pub fn initializer_call_fact_id(&self) -> &str {
        &self.initializer_call_fact_id
    }
    pub const fn result_slot(&self) -> u8 {
        self.result_slot
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceCallFact {
    fact_id: String,
    path: String,
    content_digest: String,
    caller_function_id: String,
    call_span: SourceSpan,
    callee_span: SourceSpan,
    arguments: Vec<SourceCallArgument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hook_target: Option<SourceHookTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_callable_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiver_binding_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiver_span: Option<SourceSpan>,
    colon_call: bool,
    target: SourceCallTarget,
}
impl SourceCallFact {
    /// Same-call literal lookup, selected explicitly by the Project W5 recipe.
    pub fn hook_target(&self) -> Option<&SourceHookTarget> {
        self.hook_target.as_ref()
    }
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
    pub fn caller_function_id(&self) -> &str {
        &self.caller_function_id
    }
    pub const fn call_span(&self) -> SourceSpan {
        self.call_span
    }
    pub const fn callee_span(&self) -> SourceSpan {
        self.callee_span
    }
    pub fn arguments(&self) -> &[SourceCallArgument] {
        &self.arguments
    }
    pub fn resolved_callable_key(&self) -> Option<&str> {
        self.resolved_callable_key.as_deref()
    }
    pub fn receiver_binding_id(&self) -> Option<&str> {
        self.receiver_binding_id.as_deref()
    }
    pub const fn receiver_span(&self) -> Option<SourceSpan> {
        self.receiver_span
    }
    pub const fn is_colon_call(&self) -> bool {
        self.colon_call
    }
    pub fn target(&self) -> &SourceCallTarget {
        &self.target
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceFunctionFile {
    pub path: String,
    pub content_digest: String,
    pub parse_error_count: usize,
    pub function_count: usize,
    pub call_count: usize,
}

/// Immutable independently identified sidecar; legacy member-call bytes stay unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionCallReport {
    profile: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    hook_target_recipe: Option<HookTargetRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_call_result_profile: Option<FunctionCallEvidenceProfile>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    call_result_bindings: Vec<SourceCallResultBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    budget: Option<FunctionCallBudget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    declaration_recipe: Option<PhysicalFunctionDeclarationRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature_recipe: Option<PhysicalFunctionSignatureRecipe>,
    backend: EmmyBackendIdentity,
    main_snapshot_id: String,
    library_snapshot_ids: Vec<String>,
    files: Vec<SourceFunctionFile>,
    functions: Vec<SourceFunctionFact>,
    calls: Vec<SourceCallFact>,
    global_accesses: Vec<crate::global_access::SourceGlobalAccess>,
    #[serde(skip)]
    access_locations: std::sync::Arc<AccessLocationIndex>,
    /// Only unique full-path lookups with a concrete signature enter this map.
    /// Missing names are not proven non-callable; inspect the symbol report.
    named_targets: BTreeMap<String, SourceCallTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol_lookup_analysis_id: Option<String>,
    /// Exact reports that contributed callable lookups in this semantic session.
    symbol_lookup_analysis_ids: Vec<String>,
    analysis_id: String,
}
impl FunctionCallReport {
    pub const fn hook_target_recipe(&self) -> Option<HookTargetRecipe> {
        self.hook_target_recipe
    }
    pub const fn local_call_result_profile(&self) -> Option<FunctionCallEvidenceProfile> {
        self.local_call_result_profile
    }
    pub fn call_result_bindings(&self) -> &[SourceCallResultBinding] {
        &self.call_result_bindings
    }
    pub fn call_result_binding(&self, binding_id: &str) -> Option<&SourceCallResultBinding> {
        self.call_result_bindings
            .binary_search_by(|binding| binding.binding_id().cmp(binding_id))
            .ok()
            .and_then(|index| self.call_result_bindings.get(index))
    }
    #[must_use]
    pub const fn budget(&self) -> Option<FunctionCallBudget> {
        self.budget
    }
    #[must_use]
    pub const fn declaration_recipe(&self) -> Option<PhysicalFunctionDeclarationRecipe> {
        self.declaration_recipe
    }
    #[must_use]
    pub const fn signature_recipe(&self) -> Option<PhysicalFunctionSignatureRecipe> {
        self.signature_recipe
    }
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }
    pub fn main_snapshot_id(&self) -> &str {
        &self.main_snapshot_id
    }
    pub fn library_snapshot_ids(&self) -> &[String] {
        &self.library_snapshot_ids
    }
    pub fn files(&self) -> &[SourceFunctionFile] {
        &self.files
    }
    pub fn functions(&self) -> &[SourceFunctionFact] {
        &self.functions
    }
    pub fn global_accesses(&self) -> &[crate::global_access::SourceGlobalAccess] {
        &self.global_accesses
    }
    pub fn calls(&self) -> &[SourceCallFact] {
        &self.calls
    }
    /// Returns the exact simple global receiver for one colon call when the
    /// existing global-access owner observed one unique unaliased declaration.
    /// Nested/member receivers, locals, aliases and unresolved roots remain
    /// unavailable instead of being guessed from source text.
    pub fn exact_call_receiver(&self, call: &SourceCallFact) -> Option<SourceCallReceiver> {
        if !call.is_colon_call()
            || self
                .calls
                .binary_search_by(|candidate| candidate.fact_id().cmp(call.fact_id()))
                .is_err()
        {
            return None;
        }
        let mut receivers = self
            .access_locations
            .at(
                &self.global_accesses,
                call.path(),
                call.caller_function_id(),
                call.callee_span(),
            )
            .filter_map(|access| {
                let [crate::global_access::GlobalAccessKey::String(_member)] = access.keys() else {
                    return None;
                };
                exact_callable_access_key(access)?;
                Some(SourceCallReceiver {
                    key: access.root_name().to_owned(),
                    target: access.declaration()?.clone(),
                })
            });
        let receiver = receivers.next()?;
        receivers.next().is_none().then_some(receiver)
    }
    pub fn named_targets(&self) -> &BTreeMap<String, SourceCallTarget> {
        &self.named_targets
    }
    pub fn symbol_lookup_analysis_id(&self) -> Option<&str> {
        self.symbol_lookup_analysis_id.as_deref()
    }
    pub fn symbol_lookup_analysis_ids(&self) -> &[String] {
        &self.symbol_lookup_analysis_ids
    }
    pub fn source_health_complete(&self) -> bool {
        self.files.iter().all(|file| file.parse_error_count == 0)
    }
    fn identity(&self) -> EmmyMemberCallResult<String> {
        if let Some(recipe) = self.hook_target_recipe {
            return canonical_id(
                "emmy-function-calls:sha256:",
                &(recipe, self.identity_without_hook_recipe()?),
            );
        }
        self.identity_without_hook_recipe()
    }
    fn identity_without_hook_recipe(&self) -> EmmyMemberCallResult<String> {
        let facts = (
            self.profile,
            &self.backend,
            &self.main_snapshot_id,
            &self.library_snapshot_ids,
            &self.files,
            &self.functions,
            &self.calls,
            &self.global_accesses,
            &self.named_targets,
            self.symbol_lookup_analysis_id.as_slice(),
            &self.symbol_lookup_analysis_ids,
        );
        if let Some(profile) = self.local_call_result_profile {
            return canonical_id(
                "emmy-function-calls:sha256:",
                &(
                    profile,
                    self.declaration_recipe.as_slice(),
                    self.signature_recipe.as_slice(),
                    self.budget.as_slice(),
                    &self.call_result_bindings,
                    facts,
                ),
            );
        }
        if let Some(recipe) = self.signature_recipe {
            return canonical_id(
                "emmy-function-calls:sha256:",
                &(
                    recipe,
                    self.declaration_recipe.as_slice(),
                    self.budget.as_slice(),
                    facts,
                ),
            );
        }
        match (self.declaration_recipe, self.budget) {
            (Some(recipe), Some(budget)) => {
                canonical_id("emmy-function-calls:sha256:", &(recipe, budget, facts))
            }
            (Some(recipe), None) => canonical_id("emmy-function-calls:sha256:", &(recipe, facts)),
            (None, Some(budget)) => canonical_id("emmy-function-calls:sha256:", &(budget, facts)),
            (None, None) => canonical_id("emmy-function-calls:sha256:", &facts),
        }
    }
    pub fn validate(&self) -> EmmyMemberCallResult<()> {
        self.validate_with_declaration_stop(None)
    }
    fn validate_with_declaration_stop(
        &self,
        stop: Option<&AtomicBool>,
    ) -> EmmyMemberCallResult<()> {
        let limits = match (self.profile, self.budget) {
            (FUNCTION_CALL_PROFILE, None) => FunctionCallBudget::default(),
            (FUNCTION_CALL_BUDGET_PROFILE, Some(limits))
                if limits.global_access_budget().is_none() =>
            {
                limits
            }
            (FUNCTION_CALL_GLOBAL_BUDGET_PROFILE, Some(limits))
                if limits.global_access_budget().is_some() =>
            {
                limits
            }
            (FUNCTION_CALL_LOCAL_RESULT_PROFILE, limits)
                if self.local_call_result_profile
                    == Some(FunctionCallEvidenceProfile::AceAddonLocalResultV1) =>
            {
                limits.unwrap_or_default()
            }
            _ => return Err(invalid()),
        };
        let lookup_ids = self
            .symbol_lookup_analysis_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let expected_lookup_id = lookup_identity(&self.main_snapshot_id, &lookup_ids)?;
        let expected_local_result_profile = (self.profile == FUNCTION_CALL_LOCAL_RESULT_PROFILE)
            .then_some(FunctionCallEvidenceProfile::AceAddonLocalResultV1);
        if self.local_call_result_profile != expected_local_result_profile
            || self.identity()? != self.analysis_id
            || self.symbol_lookup_analysis_id != expected_lookup_id
            || self.functions.len() as u128 > u128::from(limits.max_functions())
            || self.calls.len() as u128 > u128::from(limits.max_calls())
            || self.global_accesses.len() as u128 > u128::from(limits.max_global_accesses())
            || self
                .global_accesses
                .windows(2)
                .any(|w| w[0].fact_id() >= w[1].fact_id())
            || self
                .functions
                .windows(2)
                .any(|w| w[0].fact_id >= w[1].fact_id)
            || self.calls.windows(2).any(|w| w[0].fact_id >= w[1].fact_id)
            || self.call_result_bindings.len() as u128
                > u128::from(limits.max_calls()).saturating_mul(2)
            || self
                .call_result_bindings
                .windows(2)
                .any(|w| w[0].binding_id >= w[1].binding_id)
            || self.files.windows(2).any(|w| w[0].path >= w[1].path)
        {
            return Err(invalid());
        }
        self.access_locations.validate(&self.global_accesses)?;
        hook_targets::validate(self, stop)?;
        let functions = self
            .functions
            .iter()
            .map(|f| (f.fact_id(), f))
            .collect::<BTreeMap<_, _>>();
        let files = self
            .files
            .iter()
            .map(|f| (f.path.as_str(), f))
            .collect::<BTreeMap<_, _>>();
        for function in &self.functions {
            function
                .binding_name
                .validate(function.kind, function.span, stop)?;
            match (self.signature_recipe, &function.signature) {
                (None, None) => {}
                (Some(_), Some(observation)) => {
                    observation.validate(function.kind, limits, stop)?
                }
                _ => return Err(invalid()),
            }
            match (self.declaration_recipe, &function.declaration) {
                (None, None) => {}
                (Some(_), Some(observation)) => {
                    observation.validate(function.kind, function.span, stop)?;
                }
                _ => return Err(invalid()),
            }
            match (self.declaration_recipe, &function.shape) {
                (
                    Some(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1),
                    Some(shape),
                ) => {
                    shape.validate(function.kind, limits, stop)?;
                }
                (None | Some(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1), None) => {}
                _ => return Err(invalid()),
            }
            let file = files.get(function.path()).ok_or_else(invalid)?;
            if file.content_digest != function.content_digest || file.parse_error_count != 0 {
                return Err(invalid());
            }
            if let Some(parent) = function.parent_function_id() {
                let parent = functions.get(parent).ok_or_else(invalid)?;
                if parent.path != function.path || parent.fact_id == function.fact_id {
                    return Err(invalid());
                }
            }
        }
        for call in &self.calls {
            let caller = functions
                .get(call.caller_function_id())
                .ok_or_else(invalid)?;
            if caller.path != call.path || caller.content_digest != call.content_digest {
                return Err(invalid());
            }
            let (Some(call_start), Some(call_end)) =
                (call.call_span.byte_start(), call.call_span.byte_end())
            else {
                return Err(invalid());
            };
            if call.arguments.len() > MAX_CALL_ARGUMENTS
                || call
                    .resolved_callable_key
                    .as_ref()
                    .is_some_and(|key| !crate::bindings::supported_path(key))
            {
                return Err(invalid());
            }
            for argument in &call.arguments {
                let (Some(start), Some(end)) =
                    (argument.span.byte_start(), argument.span.byte_end())
                else {
                    return Err(invalid());
                };
                if start < call_start || end > call_end || start > end {
                    return Err(invalid());
                }
                if let Some(SourceCallLiteral::String(value)) = &argument.literal
                    && value.len() > MAX_CALL_LITERAL_BYTES
                {
                    return Err(invalid());
                }
                if let Some(local_constant) = &argument.local_constant {
                    let (Some(declaration_start), Some(declaration_end)) = (
                        local_constant.declaration_span.byte_start(),
                        local_constant.declaration_span.byte_end(),
                    ) else {
                        return Err(invalid());
                    };
                    let (Some(initializer_start), Some(initializer_end)) = (
                        local_constant.initializer_span.byte_start(),
                        local_constant.initializer_span.byte_end(),
                    ) else {
                        return Err(invalid());
                    };
                    if !matches!(
                        self.profile,
                        FUNCTION_CALL_GLOBAL_BUDGET_PROFILE | FUNCTION_CALL_LOCAL_RESULT_PROFILE
                    ) || local_constant.name.is_empty()
                        || local_constant.name.len() > MAX_CALL_LITERAL_BYTES
                        || declaration_start > declaration_end
                        || declaration_end > initializer_start
                        || initializer_start > initializer_end
                        || initializer_end > start
                        || argument.literal.is_some()
                        || argument.reference_key.is_some()
                        || argument.reference_target.is_some()
                        || matches!(
                            &local_constant.value,
                            SourceCallConstant::String(value)
                                if value.len() > MAX_CALL_LITERAL_BYTES
                        )
                    {
                        return Err(invalid());
                    }
                }
                if argument
                    .reference_key
                    .as_ref()
                    .is_some_and(|key| !crate::bindings::supported_path(key))
                    || argument.reference_key.is_some() != argument.reference_target.is_some()
                    || argument.literal.is_some()
                        && (argument.reference_key.is_some() || argument.reference_target.is_some())
                {
                    return Err(invalid());
                }
                if let Some(target) = &argument.reference_target {
                    target.span.validate().map_err(|_| invalid())?;
                    let role_valid = target.role == "main"
                        && target.workspace_id == self.main_snapshot_id
                        || target.role == "library"
                            && self.library_snapshot_ids.contains(&target.workspace_id);
                    if !role_valid
                        || target.path.is_empty()
                        || target.content_digest.is_empty()
                        || target.span.byte_start() == target.span.byte_end()
                    {
                        return Err(invalid());
                    }
                }
            }
            if let SourceCallTarget::MainFunction { function_id } = &call.target
                && functions
                    .get(function_id.as_str())
                    .is_none_or(|f| f.kind != SourceFunctionKind::Closure)
            {
                return Err(invalid());
            }
        }
        let calls_by_id = self
            .calls
            .iter()
            .map(|call| (call.fact_id(), call))
            .collect::<BTreeMap<_, _>>();
        let mut call_result_bindings = BTreeMap::new();
        for binding in &self.call_result_bindings {
            let initializer = calls_by_id
                .get(binding.initializer_call_fact_id())
                .ok_or_else(invalid)?;
            let file = files.get(binding.path.as_str()).ok_or_else(invalid)?;
            binding.declaration_span.validate().map_err(|_| invalid())?;
            let (Some(declaration_start), Some(declaration_end)) = (
                binding.declaration_span.byte_start(),
                binding.declaration_span.byte_end(),
            ) else {
                return Err(invalid());
            };
            let Some(initializer_start) = initializer.call_span().byte_start() else {
                return Err(invalid());
            };
            if binding.binding_id
                != local_call_result_binding_id(
                    &self.main_snapshot_id,
                    &binding.path,
                    &binding.content_digest,
                    &binding.scope_function_id,
                    binding.declaration_span,
                    binding.initializer_call_fact_id(),
                    binding.result_slot,
                )?
                || binding.result_slot > 1
                || declaration_start >= declaration_end
                || declaration_end > initializer_start
                || file.content_digest != binding.content_digest
                || initializer.path() != binding.path
                || initializer.content_digest() != binding.content_digest
                || initializer.caller_function_id() != binding.scope_function_id
                || !functions.contains_key(binding.scope_function_id())
                || call_result_bindings
                    .insert(binding.binding_id(), binding)
                    .is_some()
            {
                return Err(invalid());
            }
        }
        for call in &self.calls {
            match (&call.receiver_binding_id, call.receiver_span) {
                (Some(binding_id), Some(receiver_span)) => {
                    let binding = call_result_bindings
                        .get(binding_id.as_str())
                        .ok_or_else(invalid)?;
                    let (Some(start), Some(end), Some(callee_start), Some(callee_end)) = (
                        receiver_span.byte_start(),
                        receiver_span.byte_end(),
                        call.callee_span().byte_start(),
                        call.callee_span().byte_end(),
                    ) else {
                        return Err(invalid());
                    };
                    if self.local_call_result_profile.is_none()
                        || !call.is_colon_call()
                        || binding.path() != call.path()
                        || binding.content_digest() != call.content_digest()
                        || binding.scope_function_id() != call.caller_function_id()
                        || start >= end
                        || start < callee_start
                        || end > callee_end
                    {
                        return Err(invalid());
                    }
                }
                (None, None) => {}
                _ => return Err(invalid()),
            }
        }
        if self.local_call_result_profile.is_none() && !self.call_result_bindings.is_empty() {
            return Err(invalid());
        }
        for access in &self.global_accesses {
            access.validate(&self.main_snapshot_id)?;
            let function = functions.get(access.function_id()).ok_or_else(invalid)?;
            if function.path() != access.path()
                || function.content_digest() != access.content_digest()
            {
                return Err(invalid());
            }
        }
        for call in &self.calls {
            for argument in &call.arguments {
                let Some(expected) = argument
                    .reference_key
                    .as_ref()
                    .zip(argument.reference_target.as_ref())
                else {
                    continue;
                };
                let mut observed = self
                    .access_locations
                    .at(
                        &self.global_accesses,
                        &call.path,
                        &call.caller_function_id,
                        argument.span,
                    )
                    .filter_map(exact_access_reference)
                    .filter(|actual| actual.0 == *expected.0 && actual.1 == *expected.1);
                if observed.next().is_none() || observed.next().is_some() {
                    return Err(invalid());
                }
            }
        }
        if self.named_targets.len() as u128 > u128::from(limits.max_named_targets())
            || (!self.named_targets.is_empty() && self.symbol_lookup_analysis_id.is_none())
        {
            return Err(invalid());
        }
        for (query, target) in &self.named_targets {
            if !crate::bindings::supported_path(query) {
                return Err(invalid());
            }
            match target {
                SourceCallTarget::MainFunction { function_id } => {
                    if functions
                        .get(function_id.as_str())
                        .is_none_or(|f| f.kind != SourceFunctionKind::Closure)
                    {
                        return Err(invalid());
                    }
                }
                SourceCallTarget::LibraryFunction { target } => {
                    if target.role != "library"
                        || !self.library_snapshot_ids.contains(&target.workspace_id)
                    {
                        return Err(invalid());
                    }
                }
                SourceCallTarget::SignatureNotCaptured => {}
                _ => return Err(invalid()),
            }
        }
        Ok(())
    }
}

fn exact_callable_access_key(access: &crate::global_access::SourceGlobalAccess) -> Option<String> {
    use crate::global_access::{GlobalAccessKey, GlobalAccessKind, GlobalAccessResolution};

    if !access.path_complete()
        || access.kind() != GlobalAccessKind::Read
        || access.resolution() == GlobalAccessResolution::Unresolved
        || access.is_alias()
        || access.alias_blocker().is_some()
        || !crate::bindings::supported_path(access.root_name())
    {
        return None;
    }
    let mut key = access.root_name().to_owned();
    for component in access.keys() {
        let GlobalAccessKey::String(component) = component else {
            return None;
        };
        if !crate::bindings::supported_path(component) || component.contains('.') {
            return None;
        }
        key.push('.');
        key.push_str(component);
    }
    crate::bindings::supported_path(&key).then_some(key)
}

fn exact_access_reference(
    access: &crate::global_access::SourceGlobalAccess,
) -> Option<(String, SymbolTarget)> {
    Some((
        exact_callable_access_key(access)?,
        access.declaration()?.clone(),
    ))
}

fn call_literal(mut expr: LuaExpr) -> Option<SourceCallLiteral> {
    for _ in 0..MAX_SCOPE_DEPTH {
        match expr {
            LuaExpr::ParenExpr(paren) => expr = paren.get_expr()?,
            LuaExpr::LiteralExpr(value) => {
                return match value.get_literal()? {
                    LuaLiteralToken::Nil(_) => Some(SourceCallLiteral::Nil),
                    LuaLiteralToken::Bool(value) => {
                        Some(SourceCallLiteral::Boolean(value.is_true()))
                    }
                    LuaLiteralToken::String(value) => {
                        let value = value.get_value();
                        (value.len() <= MAX_CALL_LITERAL_BYTES)
                            .then_some(SourceCallLiteral::String(value))
                    }
                    _ => None,
                };
            }
            _ => return None,
        }
    }
    None
}

fn call_scalar_constant(mut expr: LuaExpr) -> Option<SourceCallConstant> {
    for _ in 0..MAX_SCOPE_DEPTH {
        match expr {
            LuaExpr::ParenExpr(paren) => expr = paren.get_expr()?,
            LuaExpr::LiteralExpr(value) => {
                return match value.get_literal()? {
                    LuaLiteralToken::String(value) => {
                        let value = value.get_value();
                        (value.len() <= MAX_CALL_LITERAL_BYTES)
                            .then_some(SourceCallConstant::String(value))
                    }
                    LuaLiteralToken::Number(value) => match value.get_number_value() {
                        NumberResult::Int(value) => Some(SourceCallConstant::Integer(value)),
                        NumberResult::Uint(_) | NumberResult::Float(_) | NumberResult::Number => {
                            None
                        }
                    },
                    _ => None,
                };
            }
            _ => return None,
        }
    }
    None
}

fn local_constant_for_argument(
    argument: &LuaExpr,
    model: &SemanticModel<'_>,
    file: &LuaWorkspaceFile,
    workspace: &str,
    caller_function_id: &str,
    cache: &mut HashMap<LuaDeclId, Option<SourceCallLocalConstant>>,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<Option<SourceCallLocalConstant>> {
    let mut expression = argument.clone();
    let mut argument_name = None;
    for _ in 0..MAX_SCOPE_DEPTH {
        match expression {
            LuaExpr::ParenExpr(paren) => {
                let Some(inner) = paren.get_expr() else {
                    return Ok(None);
                };
                expression = inner;
            }
            LuaExpr::NameExpr(name) => {
                argument_name = Some(name);
                break;
            }
            _ => return Ok(None),
        }
    }
    let Some(name) = argument_name else {
        return Ok(None);
    };
    let file_id = model.get_file_id();
    let Some(decl_id) = model
        .get_db()
        .get_reference_index()
        .get_local_reference(&file_id)
        .and_then(|references| references.get_decl_id(&name.get_range()))
    else {
        return Ok(None);
    };
    if let Some(cached) = cache.get(&decl_id) {
        return Ok(cached.clone());
    }

    let result = (|| -> EmmyMemberCallResult<Option<SourceCallLocalConstant>> {
        let Some(declaration) = model.get_db().get_decl_index().get_decl(&decl_id) else {
            return Ok(None);
        };
        if declaration.get_file_id() != file_id
            || !declaration.is_local()
            || declaration.is_param()
            || name.get_name_text().as_deref() != Some(declaration.get_name())
        {
            return Ok(None);
        }
        let Some(references) = model
            .get_db()
            .get_reference_index()
            .get_decl_references(&file_id, &decl_id)
        else {
            return Ok(None);
        };
        let Some(initializer_id) = declaration.get_value_syntax_id() else {
            return Ok(None);
        };
        let root = model.get_root().syntax();
        let Some(initializer_node) = initializer_id.to_node_from_root(root) else {
            return Ok(None);
        };
        let Some(initializer) = LuaExpr::cast(initializer_node) else {
            return Ok(None);
        };
        if initializer.get_syntax_id() != initializer_id {
            return Ok(None);
        }
        let Some(value) = call_scalar_constant(initializer) else {
            return Ok(None);
        };
        let Some(declaration_node) = declaration.get_syntax_id().to_node_from_root(root) else {
            return Ok(None);
        };
        let declaration_function_id = caller(workspace, file, &declaration_node)?;
        if declaration_function_id != caller_function_id {
            return Ok(None);
        }

        let mut found_use = false;
        for reference in &references.cells {
            checkpoint(stop)?;
            if reference.range == declaration.get_range() {
                continue;
            }
            if reference.is_write {
                return Ok(None);
            }
            found_use |= reference.range == name.get_range();
            let reference_id = LuaSyntaxId::new(LuaSyntaxKind::NameExpr.into(), reference.range);
            let Some(reference_node) = reference_id.to_node_from_root(root) else {
                return Ok(None);
            };
            let Some(reference_name) = LuaNameExpr::cast(reference_node) else {
                return Ok(None);
            };
            if caller(workspace, file, reference_name.syntax())? != declaration_function_id {
                return Ok(None);
            }
        }
        if !found_use {
            return Ok(None);
        }

        Ok(Some(SourceCallLocalConstant {
            name: name.get_name_text().ok_or_else(invalid)?,
            declaration_span: ast_span(file, declaration.get_range())?,
            initializer_span: ast_span(file, initializer_id.get_range())?,
            value,
        }))
    })()?;
    cache.insert(decl_id, result.clone());
    Ok(result)
}

fn invalid() -> EmmyMemberCallError {
    EmmyMemberCallError::new(
        EmmyMemberCallErrorCode::SemanticModelUnavailable,
        "function-call facts disagree with the captured semantic session",
        None,
    )
}
fn budget() -> EmmyMemberCallError {
    EmmyMemberCallError::new(
        EmmyMemberCallErrorCode::FactBudgetExceeded,
        "function-call facts exceed their bounded profile",
        None,
    )
}
fn resource_budget(
    resource: &str,
    observed: u128,
    limit: u64,
    path: Option<&str>,
) -> EmmyMemberCallError {
    EmmyMemberCallError::new(
        EmmyMemberCallErrorCode::FactBudgetExceeded,
        format!("function-call {resource} budget exceeded: observed {observed}, limit {limit}"),
        path,
    )
}
fn visit(count: &mut u64, limit: u64) -> EmmyMemberCallResult<()> {
    let next = u128::from(*count) + 1;
    *count = count
        .checked_add(1)
        .ok_or_else(|| resource_budget("AST visits", next, limit, None))?;
    if *count > limit {
        return Err(resource_budget("AST visits", next, limit, None));
    }
    Ok(())
}
fn function_id(
    workspace: &str,
    file: &LuaWorkspaceFile,
    kind: SourceFunctionKind,
    span: SourceSpan,
) -> EmmyMemberCallResult<String> {
    canonical_id(
        "emmy-function:sha256:",
        &(
            OCCURRENCE_PROFILE,
            workspace,
            file.path(),
            file.content_sha256(),
            kind,
            span,
        ),
    )
}
pub(crate) fn caller(
    workspace: &str,
    file: &LuaWorkspaceFile,
    syntax: &emmylua_parser::LuaSyntaxNode,
) -> EmmyMemberCallResult<String> {
    // Ancestors are bounded independently; no recursive walk or source parsing.
    for (depth, ancestor) in syntax.ancestors().skip(1).enumerate() {
        if depth >= MAX_SCOPE_DEPTH {
            return Err(resource_budget(
                "scope depth",
                depth as u128 + 1,
                MAX_SCOPE_DEPTH as u64,
                Some(file.path()),
            ));
        }
        if let Some(closure) = LuaClosureExpr::cast(ancestor) {
            return function_id(
                workspace,
                file,
                SourceFunctionKind::Closure,
                ast_span(file, closure.get_range())?,
            );
        }
    }
    function_id(
        workspace,
        file,
        SourceFunctionKind::Chunk,
        SourceSpan::whole_file(),
    )
}

const CALL_RESULT_BINDING_PROFILE: &str = "wow-emmy/local-call-result-binding/1";

fn local_call_result_binding_id(
    snapshot_id: &str,
    path: &str,
    content_digest: &str,
    scope_function_id: &str,
    declaration_span: SourceSpan,
    initializer_call_fact_id: &str,
    result_slot: u8,
) -> EmmyMemberCallResult<String> {
    canonical_id(
        "emmy-call-result-binding:sha256:",
        &(
            CALL_RESULT_BINDING_PROFILE,
            snapshot_id,
            path,
            content_digest,
            scope_function_id,
            declaration_span,
            initializer_call_fact_id,
            result_slot,
        ),
    )
}

fn enclosing_closure(node: &emmylua_parser::LuaSyntaxNode) -> Option<rowan::TextRange> {
    node.ancestors().skip(1).find_map(|ancestor| {
        LuaClosureExpr::cast(ancestor).map(|closure| closure.syntax().text_range())
    })
}

fn direct_local_colon_receiver(call: &LuaCallExpr) -> Option<LuaNameExpr> {
    if !call.is_colon_call() {
        return None;
    }
    let LuaExpr::IndexExpr(index) = call.get_prefix_expr()? else {
        return None;
    };
    let LuaExpr::NameExpr(receiver) = index.get_prefix_expr()? else {
        return None;
    };
    Some(receiver)
}

struct PendingCallResultBinding {
    binding: SourceCallResultBinding,
    declaration_scope: Option<rowan::TextRange>,
    invalid: bool,
}

/// Retain only direct one-call local declarations with at most two Lua result
/// slots. The bounded census rejects writes, alias receivers, captures, and
/// declaration ambiguity rather than trying to evaluate Lua value flow.
#[allow(clippy::too_many_arguments)]
fn collect_local_call_result_bindings(
    analysis: &EmmyLuaAnalysis,
    main: &LuaWorkspaceSnapshot,
    main_root: &Path,
    calls: &mut [SourceCallFact],
    visits: &mut u64,
    limits: FunctionCallBudget,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<Vec<SourceCallResultBinding>> {
    let mut call_locations = HashMap::<(String, u64, u64), usize>::with_capacity(calls.len());
    for (index, call) in calls.iter().enumerate() {
        checkpoint(stop)?;
        let (Some(start), Some(end)) = (call.call_span().byte_start(), call.call_span().byte_end())
        else {
            return Err(invalid());
        };
        if call_locations
            .insert((call.path().to_owned(), start, end), index)
            .is_some()
        {
            return Err(invalid());
        }
    }

    let mut result_bindings = Vec::new();
    let mut receiver_uses = Vec::<(usize, usize, SourceSpan)>::new();
    for file in main.files() {
        checkpoint(stop)?;
        let model = semantic_model(analysis, main_root, file)?;
        if model
            .get_file_parse_error()
            .is_some_and(|errors| !errors.is_empty())
        {
            continue;
        }
        let root = model.get_root();
        let mut pending = Vec::<PendingCallResultBinding>::new();
        let mut semantic_bindings = HashMap::<LuaSemanticDeclId, usize>::new();

        for node in root.descendants::<LuaAst>() {
            checkpoint(stop)?;
            visit(visits, limits.max_ast_visits())
                .map_err(|error| crate::global_access::at_path(error, file.path()))?;
            let LuaAst::LuaLocalStat(stat) = node else {
                continue;
            };
            let names = stat.get_local_name_list().collect::<Vec<_>>();
            let values = stat.get_value_exprs().collect::<Vec<_>>();
            if names.is_empty() || names.len() > 2 || values.len() != 1 {
                continue;
            }
            let Some(LuaExpr::CallExpr(initializer)) = values.first().cloned() else {
                continue;
            };
            let initializer_span = ast_span(file, initializer.get_range())?;
            let (Some(start), Some(end)) =
                (initializer_span.byte_start(), initializer_span.byte_end())
            else {
                return Err(invalid());
            };
            let Some(call_index) = call_locations
                .get(&(file.path().to_owned(), start, end))
                .copied()
            else {
                continue;
            };
            let initializer_fact = &calls[call_index];
            let scope_function_id = caller(main.snapshot_id(), file, stat.syntax())?;
            if initializer_fact.path() != file.path()
                || initializer_fact.content_digest() != file.content_sha256()
                || initializer_fact.caller_function_id() != scope_function_id
            {
                continue;
            }
            for (slot, name) in names.iter().enumerate() {
                let Some(token) = name.get_name_token() else {
                    continue;
                };
                let Some(semantic_decl) =
                    model.find_decl(name.syntax().clone().into(), SemanticDeclLevel::default())
                else {
                    continue;
                };
                let result_slot = u8::try_from(slot).map_err(|_| invalid())?;
                let declaration_span = ast_span(file, token.syntax().text_range())?;
                let binding_id = local_call_result_binding_id(
                    main.snapshot_id(),
                    file.path(),
                    file.content_sha256(),
                    &scope_function_id,
                    declaration_span,
                    initializer_fact.fact_id(),
                    result_slot,
                )?;
                let index = pending.len();
                let binding = SourceCallResultBinding {
                    binding_id,
                    path: file.path().into(),
                    content_digest: file.content_sha256().into(),
                    scope_function_id: scope_function_id.clone(),
                    declaration_span,
                    initializer_call_fact_id: initializer_fact.fact_id().into(),
                    result_slot,
                };
                if let Some(previous) = semantic_bindings.insert(semantic_decl.clone(), index) {
                    pending[previous].invalid = true;
                }
                pending.push(PendingCallResultBinding {
                    binding,
                    declaration_scope: enclosing_closure(name.syntax()),
                    invalid: false,
                });
            }
        }
        if pending.is_empty() {
            continue;
        }

        for node in root.descendants::<LuaAst>() {
            checkpoint(stop)?;
            visit(visits, limits.max_ast_visits())
                .map_err(|error| crate::global_access::at_path(error, file.path()))?;
            match node {
                LuaAst::LuaAssignStat(stat) => {
                    let (variables, _) = stat.get_var_and_expr_list();
                    for variable in variables {
                        let LuaVarExpr::NameExpr(name) = variable else {
                            continue;
                        };
                        let Some(decl) = model
                            .find_decl(name.syntax().clone().into(), SemanticDeclLevel::default())
                        else {
                            continue;
                        };
                        if let Some(index) = semantic_bindings.get(&decl).copied() {
                            pending[index].invalid = true;
                        }
                    }
                }
                LuaAst::LuaFuncStat(stat) => {
                    if let Some(LuaVarExpr::NameExpr(name)) = stat.get_func_name()
                        && let Some(decl) = model
                            .find_decl(name.syntax().clone().into(), SemanticDeclLevel::default())
                        && let Some(index) = semantic_bindings.get(&decl).copied()
                    {
                        pending[index].invalid = true;
                    }
                }
                LuaAst::LuaNameExpr(name) => {
                    let Some(decl) =
                        model.find_decl(name.syntax().clone().into(), SemanticDeclLevel::default())
                    else {
                        continue;
                    };
                    let Some(index) = semantic_bindings.get(&decl).copied() else {
                        continue;
                    };
                    if enclosing_closure(name.syntax()) != pending[index].declaration_scope {
                        pending[index].invalid = true;
                    }
                }
                LuaAst::LuaCallExpr(call) => {
                    let Some(receiver) = direct_local_colon_receiver(&call) else {
                        continue;
                    };
                    let Some(decl) = model.find_decl(
                        receiver.syntax().clone().into(),
                        SemanticDeclLevel::default(),
                    ) else {
                        continue;
                    };
                    let Some(index) = semantic_bindings.get(&decl).copied() else {
                        continue;
                    };
                    let caller_id = caller(main.snapshot_id(), file, call.syntax())?;
                    if caller_id != pending[index].binding.scope_function_id {
                        pending[index].invalid = true;
                        continue;
                    }
                    let call_span = ast_span(file, call.get_range())?;
                    let (Some(start), Some(end)) = (call_span.byte_start(), call_span.byte_end())
                    else {
                        return Err(invalid());
                    };
                    let Some(call_index) = call_locations
                        .get(&(file.path().to_owned(), start, end))
                        .copied()
                    else {
                        continue;
                    };
                    let receiver_span = ast_span(file, receiver.syntax().text_range())?;
                    receiver_uses.push((call_index, index, receiver_span));
                }
                _ => {}
            }
        }
        let valid_indices = pending
            .iter()
            .enumerate()
            .filter_map(|(index, value)| (!value.invalid).then_some(index))
            .collect::<BTreeSet<_>>();
        for index in valid_indices.iter().copied() {
            result_bindings.push(pending[index].binding.clone());
        }
        for (call_index, binding_index, receiver_span) in receiver_uses.iter().copied() {
            if !valid_indices.contains(&binding_index) {
                continue;
            }
            let call = calls.get_mut(call_index).ok_or_else(invalid)?;
            if call.receiver_binding_id.is_some() || call.receiver_span.is_some() {
                return Err(invalid());
            }
            call.receiver_binding_id = Some(pending[binding_index].binding.binding_id.clone());
            call.receiver_span = Some(receiver_span);
        }
        receiver_uses.clear();
    }
    result_bindings.sort_by(|left, right| left.binding_id.cmp(&right.binding_id));
    Ok(result_bindings)
}
enum CapturedFunction {
    Main(String),
    Library(SymbolTarget),
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn collect(
    analysis: &EmmyLuaAnalysis,
    main: &LuaWorkspaceSnapshot,
    main_root: &Path,
    libraries: &[(&LuaWorkspaceSnapshot, PathBuf)],
    callable_signatures: &BTreeMap<String, LuaSignatureId>,
    symbol_lookup_analysis_ids: &[&str],
    selected_budget: Option<FunctionCallBudget>,
    local_call_result_profile: Option<FunctionCallEvidenceProfile>,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<FunctionCallReport> {
    collect_with_declarations(
        analysis,
        main,
        main_root,
        libraries,
        callable_signatures,
        symbol_lookup_analysis_ids,
        selected_budget,
        None,
        local_call_result_profile,
        stop,
    )
}

/// Collect selected declaration observations in this same registered analysis.
/// An absent recipe preserves the existing report bytes and identity inputs.
#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_with_declarations(
    analysis: &EmmyLuaAnalysis,
    main: &LuaWorkspaceSnapshot,
    main_root: &Path,
    libraries: &[(&LuaWorkspaceSnapshot, PathBuf)],
    callable_signatures: &BTreeMap<String, LuaSignatureId>,
    symbol_lookup_analysis_ids: &[&str],
    selected_budget: Option<FunctionCallBudget>,
    declaration_recipe: Option<PhysicalFunctionDeclarationRecipe>,
    local_call_result_profile: Option<FunctionCallEvidenceProfile>,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<FunctionCallReport> {
    collect_with_components(
        analysis,
        main,
        main_root,
        libraries,
        callable_signatures,
        symbol_lookup_analysis_ids,
        selected_budget,
        declaration_recipe,
        None,
        local_call_result_profile,
        None,
        stop,
    )
}

/// Select typed observations while retaining the same native session and ledgers.
#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_with_components(
    analysis: &EmmyLuaAnalysis,
    main: &LuaWorkspaceSnapshot,
    main_root: &Path,
    libraries: &[(&LuaWorkspaceSnapshot, PathBuf)],
    callable_signatures: &BTreeMap<String, LuaSignatureId>,
    symbol_lookup_analysis_ids: &[&str],
    selected_budget: Option<FunctionCallBudget>,
    declaration_recipe: Option<PhysicalFunctionDeclarationRecipe>,
    signature_recipe: Option<PhysicalFunctionSignatureRecipe>,
    local_call_result_profile: Option<FunctionCallEvidenceProfile>,
    hook_targets: Option<(HookTargetRecipe, usize)>,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<FunctionCallReport> {
    let limits = selected_budget.unwrap_or_default();
    let mut declaration_bytes = declaration::DeclarationBudget::new(limits.max_report_bytes());
    let mut signatures = HashMap::new();
    let mut access_sources = HashMap::new();
    let mut functions = Vec::new();
    let mut files = BTreeMap::new();
    let mut visits = 0u64;
    // Capture actual closure positions first, so forward/cross-file calls never
    // depend on traversal order. Library closures remain foreign targets only.
    for (workspace, root, is_main) in std::iter::once((main, main_root, true))
        .chain(libraries.iter().map(|(w, r)| (*w, r.as_path(), false)))
    {
        for file in workspace.files() {
            checkpoint(stop)?;
            let model = semantic_model(analysis, root, file)?;
            access_sources.insert(
                model.get_file_id(),
                crate::global_access::AccessSource {
                    workspace_id: workspace.snapshot_id(),
                    file,
                    is_main,
                },
            );
            let errors = model
                .get_file_parse_error()
                .map_or(0, |errors| errors.len());
            if is_main {
                files.insert(
                    file.path().to_owned(),
                    SourceFunctionFile {
                        path: file.path().into(),
                        content_digest: file.content_sha256().into(),
                        parse_error_count: errors,
                        function_count: 0,
                        call_count: 0,
                    },
                );
            }
            if errors != 0 {
                continue;
            }
            if is_main {
                if functions.len() as u128 >= u128::from(limits.max_functions()) {
                    return Err(resource_budget(
                        "functions",
                        functions.len() as u128 + 1,
                        limits.max_functions(),
                        Some(file.path()),
                    ));
                }
                functions.push(SourceFunctionFact {
                    fact_id: function_id(
                        workspace.snapshot_id(),
                        file,
                        SourceFunctionKind::Chunk,
                        SourceSpan::whole_file(),
                    )?,
                    path: file.path().into(),
                    content_digest: file.content_sha256().into(),
                    kind: SourceFunctionKind::Chunk,
                    span: SourceSpan::whole_file(),
                    parent_function_id: None,
                    binding_name: names::BindingNameObservation::chunk(),
                    signature: if signature_recipe.is_some() {
                        let observation = SourceFunctionSignature::chunk();
                        declaration_bytes.charge(&observation, stop)?;
                        Some(observation)
                    } else {
                        None
                    },
                    shape: if declaration_recipe
                        == Some(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1)
                    {
                        let observation = SourceFunctionShape::chunk();
                        declaration_bytes.charge(&observation, stop)?;
                        Some(observation)
                    } else {
                        None
                    },
                    declaration: if declaration_recipe.is_some() {
                        let observation = SourceFunctionDeclaration::chunk();
                        declaration_bytes.charge(&observation, stop)?;
                        Some(observation)
                    } else {
                        None
                    },
                });
                files
                    .get_mut(file.path())
                    .ok_or_else(invalid)?
                    .function_count += 1;
            }
            for ast in model.get_root().descendants::<LuaAst>() {
                checkpoint(stop)?;
                visit(&mut visits, limits.max_ast_visits())
                    .map_err(|error| crate::global_access::at_path(error, file.path()))?;
                let LuaAst::LuaClosureExpr(closure) = ast else {
                    continue;
                };
                if signatures.len() as u128 >= u128::from(limits.max_functions()) {
                    return Err(resource_budget(
                        "captured signatures",
                        signatures.len() as u128 + 1,
                        limits.max_functions(),
                        Some(file.path()),
                    ));
                }
                if functions.len() as u128 >= u128::from(limits.max_functions()) {
                    return Err(resource_budget(
                        "functions",
                        functions.len() as u128 + 1,
                        limits.max_functions(),
                        Some(file.path()),
                    ));
                }
                let span = ast_span(file, closure.get_range())?;
                let signature = LuaSignatureId::from_closure(model.get_file_id(), &closure);
                let captured = if is_main {
                    let id = function_id(
                        workspace.snapshot_id(),
                        file,
                        SourceFunctionKind::Closure,
                        span,
                    )?;
                    let declaration = declaration_recipe
                        .map(|_| {
                            declaration::observe(
                                &closure,
                                file,
                                &mut visits,
                                limits,
                                &mut declaration_bytes,
                                stop,
                            )
                        })
                        .transpose()
                        .map_err(|error| crate::global_access::at_path(error, file.path()))?;
                    let observed_signature = signature_recipe
                        .map(|_| {
                            signature::observe(
                                model.get_db().get_signature_index().get(&signature),
                                &mut visits,
                                limits,
                                &mut declaration_bytes,
                                stop,
                            )
                        })
                        .transpose()
                        .map_err(|error| crate::global_access::at_path(error, file.path()))?;
                    let shape = if declaration_recipe
                        == Some(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1)
                    {
                        Some(
                            shape::observe(
                                &closure,
                                declaration.as_ref().ok_or_else(invalid)?,
                                &mut visits,
                                limits,
                                &mut declaration_bytes,
                                stop,
                            )
                            .map_err(|error| crate::global_access::at_path(error, file.path()))?,
                        )
                    } else {
                        None
                    };
                    functions.push(SourceFunctionFact {
                        fact_id: id.clone(),
                        binding_name: names::observe(&closure, file, stop)?,
                        path: file.path().into(),
                        content_digest: file.content_sha256().into(),
                        kind: SourceFunctionKind::Closure,
                        span,
                        parent_function_id: Some(caller(
                            workspace.snapshot_id(),
                            file,
                            closure.syntax(),
                        )?),
                        declaration,
                        shape,
                        signature: observed_signature,
                    });
                    files
                        .get_mut(file.path())
                        .ok_or_else(invalid)?
                        .function_count += 1;
                    CapturedFunction::Main(id)
                } else {
                    CapturedFunction::Library(SymbolTarget {
                        workspace_id: workspace.snapshot_id().into(),
                        role: "library",
                        path: file.path().into(),
                        content_digest: file.content_sha256().into(),
                        span,
                    })
                };
                if signatures.insert(signature, captured).is_some() {
                    return Err(invalid());
                }
            }
        }
    }
    let mut callable_keys = HashMap::<LuaSignatureId, Option<String>>::new();
    for (key, signature) in callable_signatures {
        match callable_keys.entry(*signature) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Some(key.clone()));
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if entry.get().as_deref() != Some(key.as_str()) {
                    entry.insert(None);
                }
            }
        }
    }
    let mut calls = Vec::new();
    let mut hook_requests = hook_targets::Requests::default();
    let mut global_accesses = Vec::new();
    let mut access_budget =
        crate::global_access::AccessBudget::new(limits.global_access_budget().unwrap_or_default());
    let include_local_constants =
        local_call_result_profile.is_some() || limits.global_access_budget().is_some();
    for file in main.files() {
        checkpoint(stop)?;
        if files
            .get(file.path())
            .ok_or_else(invalid)?
            .parse_error_count
            != 0
        {
            continue;
        }
        let model = semantic_model(analysis, main_root, file)?;
        let mut local_constant_cache = HashMap::new();
        let aliases =
            crate::global_access::AliasIndex::collect(&model, file, &mut access_budget, stop)
                .map_err(|error| crate::global_access::at_path(error, file.path()))?;
        for ast in model.get_root().descendants::<LuaAst>() {
            checkpoint(stop)?;
            visit(&mut visits, limits.max_ast_visits())
                .map_err(|error| crate::global_access::at_path(error, file.path()))?;
            if let LuaAst::LuaNameExpr(name) = &ast {
                if global_accesses.len() as u128 >= u128::from(limits.max_global_accesses()) {
                    return Err(resource_budget(
                        "global-access candidates",
                        global_accesses.len() as u128 + 1,
                        limits.max_global_accesses(),
                        Some(file.path()),
                    ));
                }
                if let Some(access) = crate::global_access::collect_one(
                    &model,
                    main.snapshot_id(),
                    file,
                    name,
                    &access_sources,
                    &aliases,
                    &mut access_budget,
                )
                .map_err(|error| crate::global_access::at_path(error, file.path()))?
                {
                    global_accesses.push(access);
                }
            }
            let LuaAst::LuaCallExpr(call) = ast else {
                continue;
            };
            if calls.len() as u128 >= u128::from(limits.max_calls()) {
                return Err(resource_budget(
                    "calls",
                    calls.len() as u128 + 1,
                    limits.max_calls(),
                    Some(file.path()),
                ));
            }
            let prefix = call.get_prefix_expr().ok_or_else(invalid)?;
            let call_span = ast_span(file, call.get_range())?;
            let callee_span = ast_span(file, prefix.syntax().text_range())?;
            let (target, resolved_callable_key) = match model.infer_expr(prefix) {
                Ok(LuaType::Signature(id)) => {
                    let key = callable_keys.get(&id).and_then(Clone::clone);
                    let target = match signatures.get(&id) {
                        Some(CapturedFunction::Main(id)) => SourceCallTarget::MainFunction {
                            function_id: id.clone(),
                        },
                        Some(CapturedFunction::Library(target)) => {
                            SourceCallTarget::LibraryFunction {
                                target: target.clone(),
                            }
                        }
                        None => SourceCallTarget::SignatureNotCaptured,
                    };
                    (target, key)
                }
                Ok(LuaType::Unknown | LuaType::Any) | Err(_) => {
                    (SourceCallTarget::Unresolved, None)
                }
                _ => (SourceCallTarget::Indeterminate, None),
            };
            if let Some((_, query_limit)) = hook_targets {
                hook_requests.observe(
                    &call,
                    &target,
                    &model,
                    &access_sources,
                    callable_signatures,
                    &signatures,
                    calls.len(),
                    limits,
                    query_limit,
                    stop,
                )?;
            }
            let caller_function_id = caller(main.snapshot_id(), file, call.syntax())?;
            let arguments = call
                .get_args_list()
                .map(|list| {
                    list.get_args()
                        .map(|argument| {
                            let span = ast_span(file, argument.syntax().text_range())?;
                            let local_constant = if include_local_constants {
                                local_constant_for_argument(
                                    &argument,
                                    &model,
                                    file,
                                    main.snapshot_id(),
                                    &caller_function_id,
                                    &mut local_constant_cache,
                                    stop,
                                )?
                            } else {
                                None
                            };
                            Ok(SourceCallArgument {
                                span,
                                literal: call_literal(argument),
                                local_constant,
                                reference_key: None,
                                reference_target: None,
                            })
                        })
                        .collect::<EmmyMemberCallResult<Vec<_>>>()
                })
                .transpose()?
                .unwrap_or_default();
            if arguments.len() > MAX_CALL_ARGUMENTS {
                return Err(resource_budget(
                    "call arguments",
                    arguments.len() as u128,
                    MAX_CALL_ARGUMENTS as u64,
                    Some(file.path()),
                ));
            }
            let colon_call = call.is_colon_call();
            let fact_id = canonical_id(
                "emmy-source-call:sha256:",
                &(
                    OCCURRENCE_PROFILE,
                    main.snapshot_id(),
                    file.path(),
                    file.content_sha256(),
                    &caller_function_id,
                    call_span,
                    callee_span,
                    colon_call,
                    &target,
                ),
            )?;
            calls.push(SourceCallFact {
                fact_id,
                path: file.path().into(),
                content_digest: file.content_sha256().into(),
                caller_function_id,
                call_span,
                callee_span,
                arguments,
                hook_target: None,
                resolved_callable_key,
                receiver_binding_id: None,
                receiver_span: None,
                colon_call,
                target,
            });
            files.get_mut(file.path()).ok_or_else(invalid)?.call_count += 1;
        }
    }
    global_accesses.sort_by(|a, b| a.fact_id().cmp(b.fact_id()));
    let access_locations = std::sync::Arc::new(AccessLocationIndex::build(
        &global_accesses,
        limits.max_global_accesses(),
        stop,
    )?);
    for call in &mut calls {
        checkpoint(stop)?;
        let mut observed = access_locations
            .at(
                &global_accesses,
                &call.path,
                &call.caller_function_id,
                call.callee_span,
            )
            .filter_map(exact_callable_access_key);
        let access_key = observed.next();
        if observed.next().is_some() {
            return Err(invalid());
        }
        match (
            call.resolved_callable_key.as_deref(),
            access_key,
            &call.target,
        ) {
            // Canonical Library closures retain their selected method key when
            // accessed through independently declared receiver tables.
            (Some(existing), Some(access), SourceCallTarget::LibraryFunction { .. })
                if existing != access.as_str() && callable_signatures.contains_key(existing) => {}
            (Some(existing), Some(access), _) if existing != access.as_str() => {
                return Err(invalid());
            }
            (None, Some(access), _) => call.resolved_callable_key = Some(access),
            _ => {}
        }
        for argument in &mut call.arguments {
            if argument.literal.is_some() {
                continue;
            }
            checkpoint(stop)?;
            let mut references = access_locations
                .at(
                    &global_accesses,
                    &call.path,
                    &call.caller_function_id,
                    argument.span,
                )
                .filter_map(exact_access_reference);
            if let Some((key, target)) = references.next() {
                argument.reference_key = Some(key);
                argument.reference_target = Some(target);
            }
            if references.next().is_some() {
                return Err(invalid());
            }
        }
    }
    functions.sort_by(|a, b| a.fact_id.cmp(&b.fact_id));
    let call_result_bindings = if local_call_result_profile.is_some() {
        collect_local_call_result_bindings(
            analysis,
            main,
            main_root,
            &mut calls,
            &mut visits,
            limits,
            stop,
        )?
    } else {
        Vec::new()
    };
    hook_requests.resolve(
        analysis,
        main,
        main_root,
        libraries,
        &signatures,
        &global_accesses,
        &mut calls,
        stop,
    )?;
    calls.sort_by(|a, b| a.fact_id.cmp(&b.fact_id));
    let library_snapshot_ids = libraries
        .iter()
        .map(|(w, _)| w.snapshot_id().to_owned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut named_targets = BTreeMap::new();
    for (query, signature) in callable_signatures {
        if named_targets.len() as u128 >= u128::from(limits.max_named_targets()) {
            return Err(resource_budget(
                "named targets",
                named_targets.len() as u128 + 1,
                limits.max_named_targets(),
                None,
            ));
        }
        checkpoint(stop)?;
        let target = match signatures.get(signature) {
            Some(CapturedFunction::Main(id)) => SourceCallTarget::MainFunction {
                function_id: id.clone(),
            },
            Some(CapturedFunction::Library(target)) => SourceCallTarget::LibraryFunction {
                target: target.clone(),
            },
            None => SourceCallTarget::SignatureNotCaptured,
        };
        named_targets.insert(query.clone(), target);
    }
    let mut report = FunctionCallReport {
        hook_target_recipe: hook_targets.map(|(recipe, _)| recipe),
        profile: if local_call_result_profile.is_some() {
            FUNCTION_CALL_LOCAL_RESULT_PROFILE
        } else if selected_budget.is_some_and(|budget| budget.global_access_budget().is_some()) {
            FUNCTION_CALL_GLOBAL_BUDGET_PROFILE
        } else if selected_budget.is_some() {
            FUNCTION_CALL_BUDGET_PROFILE
        } else {
            FUNCTION_CALL_PROFILE
        },
        local_call_result_profile,
        call_result_bindings,
        budget: selected_budget,
        declaration_recipe,
        signature_recipe,
        backend: main.backend().clone(),
        main_snapshot_id: main.snapshot_id().into(),
        library_snapshot_ids,
        files: files.into_values().collect(),
        functions,
        calls,
        global_accesses,
        access_locations,
        named_targets,
        symbol_lookup_analysis_id: lookup_identity(main.snapshot_id(), symbol_lookup_analysis_ids)?,
        symbol_lookup_analysis_ids: symbol_lookup_analysis_ids
            .iter()
            .map(|id| (*id).to_owned())
            .collect(),
        analysis_id: String::new(),
    };
    struct Count {
        used: u64,
        limit: u64,
        attempted: u128,
    }
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.attempted = u128::from(self.used) + bytes.len() as u128;
            let length = u64::try_from(bytes.len())
                .map_err(|_| std::io::Error::other("function-call report limit"))?;
            let next = self
                .used
                .checked_add(length)
                .ok_or_else(|| std::io::Error::other("function-call report limit"))?;
            if next > self.limit {
                return Err(std::io::Error::other("function-call report limit"));
            }
            self.used = next;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count {
        used: 128,
        limit: limits.max_report_bytes(),
        attempted: 128,
    };
    serde_json::to_writer(&mut count, &report).map_err(|_| {
        EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::FactBudgetExceeded,
            format!(
                "function-call report bytes budget exceeded: observed {}, limit {}; functions {}, calls {}, global accesses {}, named targets {}",
                count.attempted,
                count.limit,
                report.functions.len(),
                report.calls.len(),
                report.global_accesses.len(),
                report.named_targets.len(),
            ),
            None,
        )
    })?;
    checkpoint(stop)?;
    report.analysis_id = report.identity()?;
    if signature_recipe.is_some() {
        declaration_bytes.charge(&report, stop)?;
    }
    report.validate_with_declaration_stop(
        (declaration_recipe.is_some() || signature_recipe.is_some()).then_some(stop),
    )?;
    checkpoint(stop)?;
    Ok(report)
}

fn lookup_identity(main: &str, ids: &[&str]) -> EmmyMemberCallResult<Option<String>> {
    if ids.len() > 2
        || ids.windows(2).any(|pair| pair[0] >= pair[1])
        || ids.iter().any(|id| id.is_empty() || id.len() > 1024)
    {
        return Err(invalid());
    }
    if ids.is_empty() {
        Ok(None)
    } else {
        canonical_id("emmy-function-call-lookups:sha256:", &(main, ids)).map(Some)
    }
}

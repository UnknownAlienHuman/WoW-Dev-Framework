//! Syntactic declared headers only; exact native observations/support stay separate.
//! This child is inert until the parent wires the selected native declaration owner.
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::{ContentDigest, DigestPurpose};
use wow_emmy::function_calls::{
    FunctionCallReport, PhysicalFunctionDeclarationRecipe, SourceDeclaredParameter,
    SourceFunctionDeclaration, SourceFunctionDeclarationForm, SourceFunctionDeclarationUnavailable,
    SourceFunctionDeclarationUnsupported,
};

use super::super::producer_budget::ProducerBudget;
use super::{FunctionFingerprintManifest, invalid};
use crate::{ProjectPhase, ProjectResult};

const HEADER_DOMAIN: &[u8] = b"wow.project.fingerprint.physical-function.declared-header.v1\0";

/// Fixed comparison recipe, independent of exact source/generation identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FunctionDeclaredHeaderProfile {
    #[serde(rename = "wow-project/physical-function-declared-header/1")]
    DeclaredSyntaxV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionDeclaredHeaderFingerprint;
impl DigestPurpose for FunctionDeclaredHeaderFingerprint {
    const NAME: &'static str = "function_declared_header_fingerprint";
}

/// Available means only the explicitly declared syntactic header.
#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FunctionDeclaredHeaderComponent {
    Available {
        digest: ContentDigest<FunctionDeclaredHeaderFingerprint>,
    },
    /// No selected native declaration recipe; never parse source as a fallback.
    MissingHeader,
    Unavailable {
        reason: SourceFunctionDeclarationUnavailable,
    },
    Unsupported {
        reason: SourceFunctionDeclarationUnsupported,
    },
    /// A conservative native-scope gate, not an assertion of differing headers.
    /// The original records remain in the containing manifest's scope_conflicts.
    Conflict,
}

/// Serialize-only owned syntax observation. The containing selected record retains
/// its original ExactFunctionOwnerKey and FunctionFingerprintSupport unchanged.
/// Neither those bindings nor this observation's spans enter the comparable bytes.
#[derive(Debug, Serialize)]
pub struct FunctionDeclaredHeader {
    profile: FunctionDeclaredHeaderProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_recipe: Option<PhysicalFunctionDeclarationRecipe>,
    component: FunctionDeclaredHeaderComponent,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_observation: Option<SourceFunctionDeclaration>,
}
impl FunctionDeclaredHeader {
    #[must_use]
    pub const fn profile(&self) -> FunctionDeclaredHeaderProfile {
        self.profile
    }

    #[must_use]
    pub const fn native_recipe(&self) -> Option<PhysicalFunctionDeclarationRecipe> {
        self.native_recipe
    }

    #[must_use]
    pub const fn component(&self) -> &FunctionDeclaredHeaderComponent {
        &self.component
    }

    #[must_use]
    pub fn native_observation(&self) -> Option<&SourceFunctionDeclaration> {
        self.native_observation.as_ref()
    }
}

#[derive(Serialize)]
struct BorrowedFunctionDeclaredHeader<'a> {
    profile: FunctionDeclaredHeaderProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_recipe: Option<PhysicalFunctionDeclarationRecipe>,
    component: &'a FunctionDeclaredHeaderComponent,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_observation: Option<&'a SourceFunctionDeclaration>,
}

/// Parent-only integration: use its genuine selected record index and the same
/// cumulatively charged metadata budget, without resetting it for this component.
/// The parent must retain this receipt with that record's exact support/binding,
/// preserve overall Partial, and count the final enlarged manifest envelope.
/// The native manifest builder already checks report health, budgets and joins.
pub(super) fn build_declared_header_component(
    report: &FunctionCallReport,
    manifest: &FunctionFingerprintManifest,
    selected_record_index: usize,
    retained: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<FunctionDeclaredHeader> {
    crate::analyzer::checkpoint(stop)?;
    if retained.limit() != manifest.limits().max_metadata_bytes()
        || report.analysis_id() != manifest.binding().function_analysis_id()
    {
        return Err(invalid());
    }
    let record = manifest
        .records()
        .get(selected_record_index)
        .ok_or_else(invalid)?;
    if record.exact_owner().scope() != manifest.binding().graph_scope() {
        return Err(invalid());
    }
    // Native functions are strictly fact_id sorted, enforced by their owner.
    // Selection/F1 checks have already admitted this exact manifest record.
    let index = report
        .functions()
        .binary_search_by(|fact| fact.fact_id().cmp(record.function_id()))
        .map_err(|_| invalid())?;
    crate::analyzer::checkpoint(stop)?;
    let fact = &report.functions()[index];
    let support = record.support();
    if fact.kind() != record.kind
        || fact.path() != support.path()
        || fact.span() != support.span()
        || crate::identity::parse_source_digest(fact.content_digest(), ProjectPhase::View)?
            != support.file_content_digest
    {
        return Err(invalid());
    }

    let native_recipe = report.declaration_recipe();
    let observation = match (native_recipe, fact.declaration()) {
        (None, None) => None,
        (Some(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1), Some(observation)) => {
            // Charge the borrowed native observation before projection/hash work.
            // This also bounds every later token/string clone and cancels on writes.
            retained.charge_serialized(observation, stop)?;
            Some(observation)
        }
        // A broken native report/recipe join is not a successful missing header.
        _ => return Err(invalid()),
    };

    let component = match observation {
        None => FunctionDeclaredHeaderComponent::MissingHeader,
        Some(SourceFunctionDeclaration::Unavailable { reason }) => {
            FunctionDeclaredHeaderComponent::Unavailable { reason: *reason }
        }
        Some(SourceFunctionDeclaration::Unsupported { reason }) => {
            FunctionDeclaredHeaderComponent::Unsupported { reason: *reason }
        }
        Some(SourceFunctionDeclaration::Observed {
            form,
            colon_definition,
            parameters,
            ..
        }) => {
            if manifest.scope_conflicts().is_empty() {
                FunctionDeclaredHeaderComponent::Available {
                    digest: header_digest(*form, *colon_definition, parameters, stop)?,
                }
            } else {
                // O(1) conservative scope gate; no per-record conflict rescan,
                // invented declaration conflict or duplicated graph provenance.
                FunctionDeclaredHeaderComponent::Conflict
            }
        }
    };
    let borrowed = BorrowedFunctionDeclaredHeader {
        profile: FunctionDeclaredHeaderProfile::DeclaredSyntaxV1,
        native_recipe,
        component: &component,
        native_observation: observation,
    };
    // Exact escaped output row including option omission/framing, before ownership.
    // The observation preflight above remains conservatively charged as work input.
    retained.charge_serialized(&borrowed, stop)?;
    let native_observation = observation
        .map(|value| clone_observation(value, stop))
        .transpose()?;
    crate::analyzer::checkpoint(stop)?;
    Ok(FunctionDeclaredHeader {
        profile: FunctionDeclaredHeaderProfile::DeclaredSyntaxV1,
        native_recipe,
        component,
        native_observation,
    })
}

/// Fixed typed framing: domain; form:u8; colon:u8; token_count:u64 big-endian;
/// each token in original order: name=0,length:u64,exact UTF-8; vararg=1.
/// Tag 1 denotes the native vararg token "..." and carries no inferred parameter.
/// All lengths count bytes. No spans, IDs, paths, support, types or body enter it.
fn header_digest(
    form: SourceFunctionDeclarationForm,
    colon_definition: bool,
    parameters: &[SourceDeclaredParameter],
    stop: &AtomicBool,
) -> ProjectResult<ContentDigest<FunctionDeclaredHeaderFingerprint>> {
    crate::analyzer::checkpoint(stop)?;
    let mut hash = Sha256::new();
    hash.update(HEADER_DOMAIN);
    let form_tag = match form {
        SourceFunctionDeclarationForm::LocalFunction => 0_u8,
        SourceFunctionDeclarationForm::NamedFunction => 1_u8,
        SourceFunctionDeclarationForm::ClosureExpression => 2_u8,
    };
    hash.update([form_tag, u8::from(colon_definition)]);
    let count = u64::try_from(parameters.len()).map_err(|_| invalid())?;
    hash.update(count.to_be_bytes());
    for parameter in parameters {
        crate::analyzer::checkpoint(stop)?;
        match parameter {
            SourceDeclaredParameter::Name { name, .. } => {
                hash.update([0_u8]);
                let length = u64::try_from(name.len()).map_err(|_| invalid())?;
                hash.update(length.to_be_bytes());
                // Checkpoint granularity only, not an admission ceiling.
                for piece in name.as_bytes().chunks(64 * 1024) {
                    crate::analyzer::checkpoint(stop)?;
                    hash.update(piece);
                }
            }
            SourceDeclaredParameter::Vararg { .. } => hash.update([1_u8]),
        }
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(ContentDigest::from_bytes(hash.finalize().into()))
}

/// Retain exactly the native observation; no parser, normalized spans or self token.
fn clone_observation(
    observation: &SourceFunctionDeclaration,
    stop: &AtomicBool,
) -> ProjectResult<SourceFunctionDeclaration> {
    crate::analyzer::checkpoint(stop)?;
    let owned = match observation {
        SourceFunctionDeclaration::Unavailable { reason } => {
            SourceFunctionDeclaration::Unavailable { reason: *reason }
        }
        SourceFunctionDeclaration::Unsupported { reason } => {
            SourceFunctionDeclaration::Unsupported { reason: *reason }
        }
        SourceFunctionDeclaration::Observed {
            form,
            colon_definition,
            declaration_span,
            parameters,
        } => {
            let mut owned_parameters = Vec::new();
            for parameter in parameters {
                crate::analyzer::checkpoint(stop)?;
                let owned_parameter = match parameter {
                    SourceDeclaredParameter::Name { name, span } => SourceDeclaredParameter::Name {
                        name: name.clone(),
                        span: *span,
                    },
                    SourceDeclaredParameter::Vararg { span } => {
                        SourceDeclaredParameter::Vararg { span: *span }
                    }
                };
                crate::analyzer::checkpoint(stop)?;
                owned_parameters.push(owned_parameter);
            }
            SourceFunctionDeclaration::Observed {
                form: *form,
                colon_definition: *colon_definition,
                declaration_span: *declaration_span,
                parameters: owned_parameters,
            }
        }
    };
    crate::analyzer::checkpoint(stop)?;
    Ok(owned)
}

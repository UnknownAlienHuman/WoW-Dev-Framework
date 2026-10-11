//! Normalized declared form/parameters and complete native body AST events.
use std::sync::atomic::AtomicBool;

use super::super::producer_budget::ProducerBudget;
use super::{FunctionFingerprintManifest, invalid};
use crate::ProjectResult;
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use wow_core::{ContentDigest, DigestPurpose};
use wow_emmy::function_calls::{
    FunctionCallReport, FunctionShapeEvent, PhysicalFunctionDeclarationRecipe,
    SourceDeclaredParameter, SourceFunctionDeclaration, SourceFunctionDeclarationForm,
    SourceFunctionShape, SourceFunctionShapeUnavailable, SourceFunctionShapeUnsupported,
};

pub const DECLARATION_BODY_SHAPE_RECIPE: &str =
    "wow-project/physical-function-declaration-body-shape/1";
const DOMAIN: &[u8] = b"wow.project.fingerprint.physical-function.declaration-body-shape.v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionDeclarationShapeFingerprint;
impl DigestPurpose for FunctionDeclarationShapeFingerprint {
    const NAME: &'static str = "function_declaration_shape_fingerprint";
}

/// MissingShape retains the original wire string for old, unselected producers.
#[derive(Debug)]
pub enum FunctionDeclarationShapeComponent {
    Available {
        digest: ContentDigest<FunctionDeclarationShapeFingerprint>,
    },
    MissingShape,
    Unavailable {
        reason: SourceFunctionShapeUnavailable,
    },
    Unsupported {
        reason: SourceFunctionShapeUnsupported,
    },
    Conflict,
}
impl Serialize for FunctionDeclarationShapeComponent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(tag = "state", rename_all = "snake_case")]
        enum Selected {
            Available {
                digest: ContentDigest<FunctionDeclarationShapeFingerprint>,
            },
            Unavailable {
                reason: SourceFunctionShapeUnavailable,
            },
            Unsupported {
                reason: SourceFunctionShapeUnsupported,
            },
            Conflict,
        }
        match self {
            Self::MissingShape => serializer.serialize_str("missing_shape"),
            Self::Available { digest } => {
                Selected::Available { digest: *digest }.serialize(serializer)
            }
            Self::Unavailable { reason } => {
                Selected::Unavailable { reason: *reason }.serialize(serializer)
            }
            Self::Unsupported { reason } => {
                Selected::Unsupported { reason: *reason }.serialize(serializer)
            }
            Self::Conflict => Selected::Conflict.serialize(serializer),
        }
    }
}
impl FunctionDeclarationShapeComponent {
    pub const fn digest(&self) -> Option<ContentDigest<FunctionDeclarationShapeFingerprint>> {
        match self {
            Self::Available { digest } => Some(*digest),
            _ => None,
        }
    }
}

pub(super) fn build_shape_component(
    report: &FunctionCallReport,
    manifest: &FunctionFingerprintManifest,
    index: usize,
    retained: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<FunctionDeclarationShapeComponent> {
    crate::analyzer::checkpoint(stop)?;
    if report.declaration_recipe()
        != Some(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1)
        || retained.limit() != manifest.limits().max_metadata_bytes()
        || report.analysis_id() != manifest.binding().function_analysis_id()
    {
        return Err(invalid());
    }
    let record = manifest.records().get(index).ok_or_else(invalid)?;
    let fact_index = report
        .functions()
        .binary_search_by(|fact| fact.fact_id().cmp(record.function_id()))
        .map_err(|_| invalid())?;
    let fact = &report.functions()[fact_index];
    let support = record.support();
    if record.exact_owner().scope() != manifest.binding().graph_scope()
        || fact.kind() != record.kind
        || fact.path() != support.path()
        || fact.span() != support.span()
        || crate::identity::parse_source_digest(fact.content_digest(), crate::ProjectPhase::View)?
            != support.file_content_digest
    {
        return Err(invalid());
    }
    let shape = fact.shape().ok_or_else(invalid)?;
    // Charge actual native input before walking it; the same ledger admits output.
    retained.charge_serialized(shape, stop)?;
    let component = match shape {
        SourceFunctionShape::Unavailable { reason } => {
            FunctionDeclarationShapeComponent::Unavailable { reason: *reason }
        }
        SourceFunctionShape::Unsupported { reason } => {
            FunctionDeclarationShapeComponent::Unsupported { reason: *reason }
        }
        SourceFunctionShape::Observed { events } => {
            let Some(SourceFunctionDeclaration::Observed {
                form,
                colon_definition,
                parameters,
                ..
            }) = fact.declaration()
            else {
                return Err(invalid());
            };
            if manifest.scope_conflicts().is_empty() {
                FunctionDeclarationShapeComponent::Available {
                    digest: shape_digest(*form, *colon_definition, parameters, events, stop)?,
                }
            } else {
                FunctionDeclarationShapeComponent::Conflict
            }
        }
    };
    retained.charge_serialized(&component, stop)?;
    Ok(component)
}

/// Fixed bytes: domain; form:u8; colon:u8; param_count:u64 BE;
/// parameter kinds (anonymous name=0,vararg=1); event_count:u64 BE;
/// each event (enter=0,leave=1,token=2):u8 + owned explicit tag:u16 BE.
/// Neither native enum ordinals, JSON, source identity nor spelling enter it.
fn shape_digest(
    form: SourceFunctionDeclarationForm,
    colon: bool,
    parameters: &[SourceDeclaredParameter],
    events: &[FunctionShapeEvent],
    stop: &AtomicBool,
) -> ProjectResult<ContentDigest<FunctionDeclarationShapeFingerprint>> {
    crate::analyzer::checkpoint(stop)?;
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    let tag = match form {
        SourceFunctionDeclarationForm::LocalFunction => 0_u8,
        SourceFunctionDeclarationForm::NamedFunction => 1,
        SourceFunctionDeclarationForm::ClosureExpression => 2,
    };
    hash.update([tag, u8::from(colon)]);
    hash.update(
        u64::try_from(parameters.len())
            .map_err(|_| invalid())?
            .to_be_bytes(),
    );
    for parameter in parameters {
        crate::analyzer::checkpoint(stop)?;
        hash.update([match parameter {
            SourceDeclaredParameter::Name { .. } => 0_u8,
            SourceDeclaredParameter::Vararg { .. } => 1,
        }]);
    }
    hash.update(
        u64::try_from(events.len())
            .map_err(|_| invalid())?
            .to_be_bytes(),
    );
    for event in events {
        crate::analyzer::checkpoint(stop)?;
        let (event_tag, tag) = match event {
            FunctionShapeEvent::Enter(tag) => (0_u8, tag.encoding_tag()),
            FunctionShapeEvent::Leave(tag) => (1_u8, tag.encoding_tag()),
            FunctionShapeEvent::Token(tag) => (2_u8, tag.encoding_tag()),
        };
        hash.update([event_tag]);
        hash.update(tag.to_be_bytes());
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(ContentDigest::from_bytes(hash.finalize().into()))
}

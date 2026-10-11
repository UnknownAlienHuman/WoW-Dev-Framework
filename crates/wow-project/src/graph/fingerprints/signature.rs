//! Comparison of the native owner's reviewed indexed parameter/return types.
use std::sync::atomic::AtomicBool;

use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use wow_core::{ContentDigest, DigestPurpose};
use wow_emmy::function_calls::{
    PhysicalFunctionSignatureRecipe, SourceFunctionSignature, SourceSignatureArrayLength,
    SourceSignatureComparable, SourceSignatureType, SourceSignatureUnavailable,
};

use super::super::producer_budget::ProducerBudget;
use super::invalid;
use crate::ProjectResult;

pub const FUNCTION_SIGNATURE_RECIPE: &str = "wow-project/physical-function-indexed-types/1";
const DOMAIN: &[u8] = b"wow-project/physical-function-indexed-types/1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSignatureFingerprint;
impl DigestPurpose for FunctionSignatureFingerprint {
    const NAME: &'static str = "function_signature_fingerprint";
}

/// Availability refers only to the selected finite static type recipe.
#[derive(Debug)]
pub enum FunctionSignatureComponent {
    MissingSignature,
    Available {
        digest: ContentDigest<FunctionSignatureFingerprint>,
    },
    Unavailable {
        reasons: Vec<SourceSignatureUnavailable>,
    },
    IncompleteSourceHealth,
    Conflict,
}
impl Serialize for FunctionSignatureComponent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(tag = "state", rename_all = "snake_case")]
        enum Selected<'a> {
            Available {
                recipe: &'static str,
                digest: ContentDigest<FunctionSignatureFingerprint>,
            },
            Unavailable {
                recipe: &'static str,
                reasons: &'a [SourceSignatureUnavailable],
            },
            IncompleteSourceHealth,
            Conflict,
        }
        match self {
            Self::MissingSignature => serializer.serialize_str("missing_signature"),
            Self::Available { digest } => Selected::Available {
                recipe: FUNCTION_SIGNATURE_RECIPE,
                digest: *digest,
            }
            .serialize(serializer),
            Self::Unavailable { reasons } => Selected::Unavailable {
                recipe: FUNCTION_SIGNATURE_RECIPE,
                reasons,
            }
            .serialize(serializer),
            Self::IncompleteSourceHealth => Selected::IncompleteSourceHealth.serialize(serializer),
            Self::Conflict => Selected::Conflict.serialize(serializer),
        }
    }
}
impl FunctionSignatureComponent {
    pub const fn digest(&self) -> Option<ContentDigest<FunctionSignatureFingerprint>> {
        match self {
            Self::Available { digest } => Some(*digest),
            _ => None,
        }
    }
}

/// The parent has admitted this fact and retains its exact original native support.
/// Native observation/recipe remain bound by the parent analysis ID and selection.
pub(super) fn build_component(
    recipe: PhysicalFunctionSignatureRecipe,
    observation: &SourceFunctionSignature,
    source_health_complete: bool,
    unconflicted: bool,
    retained: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<FunctionSignatureComponent> {
    crate::analyzer::checkpoint(stop)?;
    match recipe {
        PhysicalFunctionSignatureRecipe::IndexedTypesV1 => {}
    }
    retained.charge_serialized(observation, stop)?;
    let component = if !source_health_complete {
        FunctionSignatureComponent::IncompleteSourceHealth
    } else if !unconflicted {
        FunctionSignatureComponent::Conflict
    } else if let Some(value) = observation.comparable() {
        FunctionSignatureComponent::Available {
            digest: signature_digest(value, stop)?,
        }
    } else {
        let reasons = observation.unavailable_reasons();
        if reasons.is_empty() {
            return Err(invalid());
        }
        // The native owner exposes at most its closed seven eligibility reasons.
        // Charge before copying, and retain exact reason order.
        retained.charge_serialized(&reasons, stop)?;
        let mut owned = Vec::new();
        for reason in reasons {
            crate::analyzer::checkpoint(stop)?;
            owned.push(*reason);
        }
        FunctionSignatureComponent::Unavailable { reasons: owned }
    };
    retained.charge_serialized(&component, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(component)
}

/// Fixed typed bytes: domain+NUL; colon:u8; parameter_count:u64 BE;
/// ordered parameters (backend_nullable:u8, type); return_row_count:u64 BE;
/// ordered rows (type_count:u64 BE, ordered types). Type tags below are owned
/// by this recipe; array lengths are explicit and unions use the native
/// reviewed deterministic member order. Names, IDs, paths, source offsets,
/// return-resolution/async metadata and generation bindings are excluded.
fn signature_digest(
    value: &SourceSignatureComparable,
    stop: &AtomicBool,
) -> ProjectResult<ContentDigest<FunctionSignatureFingerprint>> {
    crate::analyzer::checkpoint(stop)?;
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    hash.update([u8::from(value.colon_definition())]);
    count(&mut hash, value.parameters().len())?;
    for parameter in value.parameters() {
        crate::analyzer::checkpoint(stop)?;
        hash.update([u8::from(parameter.backend_nullable().ok_or_else(invalid)?)]);
        type_bytes(&mut hash, parameter.type_shape(), 1, stop)?;
    }
    count(&mut hash, value.returns().len())?;
    for row in value.returns() {
        crate::analyzer::checkpoint(stop)?;
        count(&mut hash, row.types().len())?;
        for value in row.types() {
            type_bytes(&mut hash, value, 1, stop)?;
        }
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(ContentDigest::from_bytes(hash.finalize().into()))
}

fn count(hash: &mut Sha256, length: usize) -> ProjectResult<()> {
    hash.update(u64::try_from(length).map_err(|_| invalid())?.to_be_bytes());
    Ok(())
}

fn type_bytes(
    hash: &mut Sha256,
    value: &SourceSignatureType,
    depth: usize,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    // The native recipe has this same finite depth bound; no new allowance.
    if depth > 256 {
        return Err(invalid());
    }
    let tag = match value {
        SourceSignatureType::Nil => 1_u8,
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
        SourceSignatureType::Unavailable { .. } => return Err(invalid()),
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
            type_bytes(hash, element, depth + 1, stop)?;
        }
        SourceSignatureType::Tuple { elements } => {
            count(hash, elements.len())?;
            for element in elements {
                type_bytes(hash, element, depth + 1, stop)?;
            }
        }
        SourceSignatureType::Union { members } => {
            count(hash, members.len())?;
            for member in members {
                type_bytes(hash, member, depth + 1, stop)?;
            }
        }
        _ => {}
    }
    Ok(())
}

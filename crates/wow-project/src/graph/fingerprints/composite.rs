//! Composite of five complete native physical-function comparison components.
//! Component domains/digests only; exact owner, custody and lineage stay separate.
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{ContentDigest, DigestPurpose};

use super::super::producer_budget::ProducerBudget;
use super::{
    DECLARATION_BODY_SHAPE_RECIPE, FUNCTION_DIRECT_NEIGHBORHOOD_RECIPE,
    FUNCTION_FINGERPRINT_PROFILE, FUNCTION_PACKAGE_LOAD_ROLE_RECIPE,
    FunctionDeclarationShapeFingerprint, FunctionDirectNeighborhoodFingerprint,
    FunctionDirectNeighborhoodSelection, FunctionFingerprintRecord, FunctionNeighborhoodComponent,
    FunctionPackageLoadRoleComponent, FunctionPackageLoadRoleCoverage,
    FunctionPackageLoadRoleFingerprint, FunctionPackageLoadRoleSelection,
    FunctionSignatureFingerprint, FunctionSourceSpanComponent, FunctionSourceSpanFingerprint,
    SOURCE_SPAN_DOMAIN, invalid, signature::FUNCTION_SIGNATURE_RECIPE,
};
use crate::ProjectResult;

pub const FUNCTION_COMPOSITE_RECIPE: &str =
    "wow-project/physical-function-composite/five-complete-components/1";
// Frozen component digest domains, excluding their framing NUL. Their existing
// recipes own these bytes; this query neither rewrites nor re-hashes components.
const SIGNATURE_DOMAIN: &str = "wow-project/physical-function-indexed-types/1";
const SHAPE_DOMAIN: &str = "wow.project.fingerprint.physical-function.declaration-body-shape.v1";
const NEIGHBORHOOD_DOMAIN: &str =
    "wow.project.fingerprint.physical-function.composed-calls-and-xml-handlers.v1";
const LOAD_ROLE_DOMAIN: &str =
    "wow-project/structural-fingerprint/package-load-role/selected-physical-lua/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FunctionCompositeRecipe {
    #[serde(rename = "wow-project/physical-function-composite/five-complete-components/1")]
    FiveCompleteComponentsV1,
}
impl<'de> Deserialize<'de> for FunctionCompositeRecipe {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match String::deserialize(deserializer)?.as_str() {
            FUNCTION_COMPOSITE_RECIPE => Ok(Self::FiveCompleteComponentsV1),
            _ => Err(serde::de::Error::custom(
                "unsupported function composite recipe",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionCompositeFingerprint;
impl DigestPurpose for FunctionCompositeFingerprint {
    const NAME: &'static str = "function_composite_fingerprint";
}

/// Recipe/domain are fixed interpretation, not caller strings or entity labels.
#[derive(Debug, Serialize)]
#[serde(bound(serialize = ""))]
pub struct FunctionCompositePart<P: DigestPurpose> {
    recipe: &'static str,
    domain: &'static str,
    digest: ContentDigest<P>,
}
impl<P: DigestPurpose> FunctionCompositePart<P> {
    #[must_use]
    pub const fn recipe(&self) -> &'static str {
        self.recipe
    }
    #[must_use]
    pub const fn domain(&self) -> &'static str {
        self.domain
    }
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<P> {
        self.digest
    }
}

/// Fixed order: indexed signature, declaration/body shape, composed direct
/// neighborhood, exact source span, selected physical package/load role.
#[derive(Debug, Serialize)]
pub struct FunctionCompositeComparable {
    indexed_signature: FunctionCompositePart<FunctionSignatureFingerprint>,
    declaration_body_shape: FunctionCompositePart<FunctionDeclarationShapeFingerprint>,
    direct_neighborhood: FunctionCompositePart<FunctionDirectNeighborhoodFingerprint>,
    exact_source_span: FunctionCompositePart<FunctionSourceSpanFingerprint>,
    package_load_role: FunctionCompositePart<FunctionPackageLoadRoleFingerprint>,
}
impl FunctionCompositeComparable {
    #[must_use]
    pub const fn indexed_signature(&self) -> &FunctionCompositePart<FunctionSignatureFingerprint> {
        &self.indexed_signature
    }
    #[must_use]
    pub const fn declaration_body_shape(
        &self,
    ) -> &FunctionCompositePart<FunctionDeclarationShapeFingerprint> {
        &self.declaration_body_shape
    }
    #[must_use]
    pub const fn direct_neighborhood(
        &self,
    ) -> &FunctionCompositePart<FunctionDirectNeighborhoodFingerprint> {
        &self.direct_neighborhood
    }
    #[must_use]
    pub const fn exact_source_span(&self) -> &FunctionCompositePart<FunctionSourceSpanFingerprint> {
        &self.exact_source_span
    }
    #[must_use]
    pub const fn package_load_role(
        &self,
    ) -> &FunctionCompositePart<FunctionPackageLoadRoleFingerprint> {
        &self.package_load_role
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionCompositeRequiredComponent {
    IndexedSignature,
    DeclarationBodyShape,
    DirectNeighborhood,
    ExactSourceSpan,
    PackageLoadRole,
}

/// Unavailable retains which mandatory components lack complete digests.
/// Original typed reasons/support remain in the containing function record.
#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FunctionCompositeComponent {
    Available {
        recipe: FunctionCompositeRecipe,
        comparable: Box<FunctionCompositeComparable>,
        digest: ContentDigest<FunctionCompositeFingerprint>,
    },
    Unavailable {
        recipe: FunctionCompositeRecipe,
        incomplete_components: Vec<FunctionCompositeRequiredComponent>,
    },
}
impl FunctionCompositeComponent {
    #[must_use]
    pub const fn recipe(&self) -> FunctionCompositeRecipe {
        match self {
            Self::Available { recipe, .. } | Self::Unavailable { recipe, .. } => *recipe,
        }
    }
    #[must_use]
    pub const fn digest(&self) -> Option<ContentDigest<FunctionCompositeFingerprint>> {
        match self {
            Self::Available { digest, .. } => Some(*digest),
            Self::Unavailable { .. } => None,
        }
    }
    #[must_use]
    pub fn comparable(&self) -> Option<&FunctionCompositeComparable> {
        match self {
            Self::Available { comparable, .. } => Some(comparable),
            Self::Unavailable { .. } => None,
        }
    }
    #[must_use]
    pub fn incomplete_components(&self) -> &[FunctionCompositeRequiredComponent] {
        match self {
            Self::Available { .. } => &[],
            Self::Unavailable {
                incomplete_components,
                ..
            } => incomplete_components,
        }
    }
}

/// Internal only: parent already owns native eligibility/support and completes
/// all component stages before calling, with its SAME still-live ledger.
pub(super) fn build_component(
    recipe: FunctionCompositeRecipe,
    record: &FunctionFingerprintRecord,
    retained: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<FunctionCompositeComponent> {
    crate::analyzer::checkpoint(stop)?;
    match recipe {
        FunctionCompositeRecipe::FiveCompleteComponentsV1 => {}
    }
    let signature = record.signature().digest();
    let shape = record.declaration_shape().digest();
    let neighborhood = match record.direct_neighborhood() {
        FunctionDirectNeighborhoodSelection::Selected(value) => match value.as_ref() {
            FunctionNeighborhoodComponent::Comparable { digest, .. } => Some(*digest),
            FunctionNeighborhoodComponent::Unavailable { .. } => None,
        },
        FunctionDirectNeighborhoodSelection::NotSelected => None,
    };
    let span = match record.source_span() {
        FunctionSourceSpanComponent::Available { digest, .. } => Some(*digest),
        FunctionSourceSpanComponent::MissingExactSpan => None,
    };
    let role = match record.package_load_role() {
        FunctionPackageLoadRoleSelection::Selected(value)
            if value.recipe() == FUNCTION_PACKAGE_LOAD_ROLE_RECIPE
                && value.coverage() == FunctionPackageLoadRoleCoverage::Complete =>
        {
            match value.component() {
                FunctionPackageLoadRoleComponent::Comparable { digest, .. } => Some(*digest),
                FunctionPackageLoadRoleComponent::Unavailable { .. } => None,
            }
        }
        _ => None,
    };
    retained.charge_serialized(&(recipe, signature, shape, neighborhood, span, role), stop)?;
    let mut incomplete = Vec::new();
    for (kind, missing) in [
        (
            FunctionCompositeRequiredComponent::IndexedSignature,
            signature.is_none(),
        ),
        (
            FunctionCompositeRequiredComponent::DeclarationBodyShape,
            shape.is_none(),
        ),
        (
            FunctionCompositeRequiredComponent::DirectNeighborhood,
            neighborhood.is_none(),
        ),
        (
            FunctionCompositeRequiredComponent::ExactSourceSpan,
            span.is_none(),
        ),
        (
            FunctionCompositeRequiredComponent::PackageLoadRole,
            role.is_none(),
        ),
    ] {
        crate::analyzer::checkpoint(stop)?;
        if missing {
            retained.charge_serialized(&kind, stop)?;
            incomplete.push(kind);
        }
    }
    let component = if incomplete.is_empty() {
        let comparable = FunctionCompositeComparable {
            indexed_signature: FunctionCompositePart {
                recipe: FUNCTION_SIGNATURE_RECIPE,
                domain: SIGNATURE_DOMAIN,
                digest: signature.ok_or_else(invalid)?,
            },
            declaration_body_shape: FunctionCompositePart {
                recipe: DECLARATION_BODY_SHAPE_RECIPE,
                domain: SHAPE_DOMAIN,
                digest: shape.ok_or_else(invalid)?,
            },
            direct_neighborhood: FunctionCompositePart {
                recipe: FUNCTION_DIRECT_NEIGHBORHOOD_RECIPE,
                domain: NEIGHBORHOOD_DOMAIN,
                digest: neighborhood.ok_or_else(invalid)?,
            },
            exact_source_span: FunctionCompositePart {
                recipe: FUNCTION_FINGERPRINT_PROFILE,
                domain: SOURCE_SPAN_DOMAIN,
                digest: span.ok_or_else(invalid)?,
            },
            package_load_role: FunctionCompositePart {
                recipe: FUNCTION_PACKAGE_LOAD_ROLE_RECIPE,
                domain: LOAD_ROLE_DOMAIN,
                digest: role.ok_or_else(invalid)?,
            },
        };
        // Fixed owned value is precharged before its single allocation.
        retained.charge_serialized(&comparable, stop)?;
        let digest = composite_digest(&comparable, stop)?;
        FunctionCompositeComponent::Available {
            recipe,
            comparable: Box::new(comparable),
            digest,
        }
    } else {
        FunctionCompositeComponent::Unavailable {
            recipe,
            incomplete_components: incomplete,
        }
    };
    retained.charge_serialized(&component, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(component)
}

/// Canonical binary: composite recipe UTF-8 + NUL, count:u8=5; five fixed
/// ordinals:u8=1..5, each recipe length:u64 BE + UTF-8, digest-domain length:u64
/// BE + UTF-8, then the COMPLETE component's raw 32-byte SHA-256.
/// No generic JSON, whole record, identity or supported prefix enters the hash.
fn composite_digest(
    value: &FunctionCompositeComparable,
    stop: &AtomicBool,
) -> ProjectResult<ContentDigest<FunctionCompositeFingerprint>> {
    crate::analyzer::checkpoint(stop)?;
    let mut hash = Sha256::new();
    hash.update(FUNCTION_COMPOSITE_RECIPE.as_bytes());
    hash.update([0, 5]);
    part(&mut hash, 1, &value.indexed_signature, stop)?;
    part(&mut hash, 2, &value.declaration_body_shape, stop)?;
    part(&mut hash, 3, &value.direct_neighborhood, stop)?;
    part(&mut hash, 4, &value.exact_source_span, stop)?;
    part(&mut hash, 5, &value.package_load_role, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(ContentDigest::from_bytes(hash.finalize().into()))
}
fn part<P: DigestPurpose>(
    hash: &mut Sha256,
    ordinal: u8,
    value: &FunctionCompositePart<P>,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    hash.update([ordinal]);
    for text in [value.recipe, value.domain] {
        crate::analyzer::checkpoint(stop)?;
        hash.update(
            u64::try_from(text.len())
                .map_err(|_| invalid())?
                .to_be_bytes(),
        );
        hash.update(text.as_bytes());
    }
    hash.update(value.digest.as_bytes());
    crate::analyzer::checkpoint(stop)
}

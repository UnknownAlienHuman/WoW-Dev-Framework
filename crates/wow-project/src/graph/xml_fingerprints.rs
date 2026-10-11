//! One reviewed schema-class comparison component from sealed native inclusion.
//! No signature, body shape, neighborhood, lineage or runtime identity is supplied.
mod type_shape;

pub use type_shape::{
    XML_SCHEMA_TYPE_SHAPE_RECIPE, XmlSchemaTypeShapeComparable, XmlSchemaTypeShapeComponent,
    XmlSchemaTypeShapeEvent, XmlSchemaTypeShapeOccurs, XmlSchemaTypeShapeOmission,
    XmlSchemaTypeShapeOmissionReason, XmlSchemaTypeShapeRecipe, XmlSchemaTypeShapeRecord,
    XmlSchemaTypeShapeUnavailable, build_selected_xml_schema_type_shapes,
};

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest};
use wow_graph::GraphCoverageState;

use super::producer_budget::ProducerBudget;
use super::{
    ProjectXmlClassificationOccurrence, XmlClassificationCatalogWitness,
    XmlClassificationInclusionManifest,
};
use crate::load::schema::{
    XmlExpandedName, XmlSchemaComponentKind, XmlSchemaComponentState, XmlSchemaQNameState,
    XmlSchemaReferenceKind, XmlSchemaReferenceState,
    classification::{
        XmlClassOutcome, XmlClassificationConflict, XmlClassificationUnknown,
        XmlClassificationWitness, XmlOccurrenceClassification, XmlSchemaClass,
    },
};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};

/// The finite named-positive-proof recipe, independent of source/generation custody.
pub const XML_SCHEMA_CLASS_COMPONENT_RECIPE: &str =
    "wow-project/xml-schema-class-component/named-proof-witnesses/1";

/// The only reviewed target component families admitted by this recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QualifiedWitnessKind {
    GlobalElement,
    NamedSimpleType,
    NamedComplexType,
}

/// A normalized proof target, not topology, order or a complete type shape.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct QualifiedProofTarget {
    reference_kind: XmlSchemaReferenceKind,
    target_kind: QualifiedWitnessKind,
    name: XmlExpandedName,
}
impl QualifiedProofTarget {
    #[must_use]
    pub const fn reference_kind(&self) -> XmlSchemaReferenceKind {
        self.reference_kind
    }
    #[must_use]
    pub const fn target_kind(&self) -> QualifiedWitnessKind {
        self.target_kind
    }
    #[must_use]
    pub const fn name(&self) -> &XmlExpandedName {
        &self.name
    }
}

/// Only finite reviewed tags and expanded names participate in comparable bytes.
/// XmlExpandedName omits namespace when absent; no null namespace is introduced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaClassComparable {
    class: XmlSchemaClass,
    declaration: XmlExpandedName,
    qualified_proof_targets: Vec<QualifiedProofTarget>,
}
impl XmlSchemaClassComparable {
    #[must_use]
    pub const fn class(&self) -> XmlSchemaClass {
        self.class
    }
    #[must_use]
    pub const fn declaration(&self) -> &XmlExpandedName {
        &self.declaration
    }
    #[must_use]
    pub fn qualified_proof_targets(&self) -> &[QualifiedProofTarget] {
        &self.qualified_proof_targets
    }
}

/// A conservative missing component; no digest accompanies an unavailable value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "reason", content = "detail", rename_all = "snake_case")]
pub enum XmlSchemaClassUnavailable {
    ClassificationUnknown(XmlClassificationUnknown),
    ClassificationConflict(XmlClassificationConflict),
    MissingDeclarationWitness,
    MissingNamedTypeWitness,
    MissingReferenceWitness,
    AmbiguousReferenceWitness,
    UnsupportedReferenceWitness,
    UnsupportedTargetWitness,
    InconsistentProof,
}

/// Serialize-only observation; decoding cannot mint the native inclusion capability.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum XmlSchemaClassComponentValue {
    Available {
        comparable: XmlSchemaClassComparable,
        component_digest: ContentDigest<CanonicalResult>,
    },
    Unavailable {
        reason: XmlSchemaClassUnavailable,
    },
}
impl XmlSchemaClassComponentValue {
    #[must_use]
    pub const fn comparable(&self) -> Option<&XmlSchemaClassComparable> {
        match self {
            Self::Available { comparable, .. } => Some(comparable),
            Self::Unavailable { .. } => None,
        }
    }
    #[must_use]
    pub const fn component_digest(&self) -> Option<ContentDigest<CanonicalResult>> {
        match self {
            Self::Available {
                component_digest, ..
            } => Some(*component_digest),
            Self::Unavailable { .. } => None,
        }
    }
    #[must_use]
    pub const fn unavailable_reason(&self) -> Option<XmlSchemaClassUnavailable> {
        match self {
            Self::Available { .. } => None,
            Self::Unavailable { reason } => Some(*reason),
        }
    }
    /// Complete describes only this finite component, never the source universe.
    #[must_use]
    pub const fn coverage(&self) -> GraphCoverageState {
        match self {
            Self::Available { .. } => GraphCoverageState::Complete,
            Self::Unavailable { .. } => GraphCoverageState::Partial,
        }
    }
}

type ComponentId = ContentDigest<CanonicalResult>;
struct CatalogIndex<'a> {
    components: BTreeMap<ComponentId, &'a XmlClassificationCatalogWitness>,
    references: BTreeMap<ComponentId, Vec<&'a XmlClassificationCatalogWitness>>,
}

/// Project strictly ordered, unique exact occurrence proposal IDs.
/// Only selected rows are normalized; native occurrence/catalog indexes remain bounded.
/// The caller's used counter is retained in ONE cumulative ProducerBudget, and the
/// returned counter includes indexes, normalization and selected output retention.
/// Budget/cancellation/invalid-selection refusal returns no truncated success.
/// Source/profile/catalog/generation/membership custody stays with the sealed input
/// and the service result; selected IDs NEVER enter comparable digest material.
pub fn build_selected_xml_schema_class_components(
    inclusion: &XmlClassificationInclusionManifest<'_>,
    ids: &[&str],
    used: usize,
    limit: usize,
    stop: &AtomicBool,
) -> ProjectResult<(Vec<(String, XmlSchemaClassComponentValue)>, usize)> {
    crate::analyzer::checkpoint(stop)?;
    if limit == 0 {
        return Err(ProjectError::new(
            ProjectErrorCode::InvalidBudgetPolicy,
            ProjectPhase::View,
            "XML schema-class component metadata limit must be positive",
        ));
    }
    let mut budget = ProducerBudget::new(used, limit)?;
    let classification = inclusion.classification();
    let mut occurrences = BTreeMap::<
        &str,
        (
            &XmlOccurrenceClassification,
            &ProjectXmlClassificationOccurrence,
        ),
    >::new();
    let mut max_native_id_bytes = 0_usize;
    for document in classification.documents() {
        crate::analyzer::checkpoint(stop)?;
        if document.report.occurrences().len() != document.occurrences.len() {
            return Err(invalid());
        }
        for (observation, projection) in document
            .report
            .occurrences()
            .iter()
            .zip(&document.occurrences)
        {
            crate::analyzer::checkpoint(stop)?;
            if observation.occurrence() != projection.occurrence {
                return Err(invalid());
            }
            budget.charge_serialized(
                &(
                    "xml-class-component-occurrence-index",
                    &projection.occurrence_proposal_id,
                ),
                stop,
            )?;
            max_native_id_bytes = max_native_id_bytes.max(projection.occurrence_proposal_id.len());
            if occurrences
                .insert(
                    projection.occurrence_proposal_id.as_str(),
                    (observation, projection),
                )
                .is_some()
            {
                return Err(invalid());
            }
        }
    }
    if ids.len() > occurrences.len() {
        return Err(invalid_selection());
    }
    // Reject oversized strings in constant time before lexical comparisons.
    for &id in ids {
        crate::analyzer::checkpoint(stop)?;
        if id.is_empty() || id.len() > max_native_id_bytes || !occurrences.contains_key(id) {
            return Err(invalid_selection());
        }
    }
    // Every ID now equals a bounded genuine native ID.
    for pair in ids.windows(2) {
        crate::analyzer::checkpoint(stop)?;
        if pair[0] >= pair[1] {
            return Err(invalid_selection());
        }
    }
    budget.charge_serialized(&("xml-class-component-selection", ids), stop)?;
    let mut index = CatalogIndex {
        components: BTreeMap::new(),
        references: BTreeMap::new(),
    };
    for witness in classification.catalog_support().witnesses() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(
            &(
                "xml-class-component-catalog-index",
                witness.component_id,
                witness.reference.as_ref().map(|reference| reference.kind()),
            ),
            stop,
        )?;
        if witness.reference.is_some() {
            index
                .references
                .entry(witness.component_id)
                .or_default()
                .push(witness);
        } else if index
            .components
            .insert(witness.component_id, witness)
            .is_some()
        {
            return Err(invalid());
        }
    }
    let mut selected = Vec::new();
    for &id in ids {
        crate::analyzer::checkpoint(stop)?;
        let &(observation, projection) = occurrences.get(id).ok_or_else(invalid)?;
        match (
            observation.outcome(),
            &projection.class_proposal_id,
            &projection.relation_proposal_id,
        ) {
            (XmlClassOutcome::Classified(_), Some(_), Some(_))
            | (XmlClassOutcome::Unknown(_) | XmlClassOutcome::Conflict(_), None, None) => {}
            _ => return Err(invalid()),
        }
        let value = match comparable(observation, &index, &mut budget, stop)? {
            Ok(value) => {
                budget.charge_serialized(&value, stop)?;
                // Hash the fixed typed semantic projection, never IDs, bindings,
                // source text, arbitrary JSON or a serialized report.
                let component_digest = crate::identity::canonical_digest(
                    XML_SCHEMA_CLASS_COMPONENT_RECIPE,
                    &value,
                    ProjectPhase::View,
                )?;
                crate::analyzer::checkpoint(stop)?;
                XmlSchemaClassComponentValue::Available {
                    comparable: value,
                    component_digest,
                }
            }
            Err(reason) => XmlSchemaClassComponentValue::Unavailable { reason },
        };
        budget.charge_serialized(&(id, &value), stop)?;
        crate::analyzer::checkpoint(stop)?;
        selected.push((id.to_owned(), value));
    }
    // Include exact array/tuple framing under the SAME accumulated ledger.
    budget.charge_serialized(&selected, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok((selected, budget.used()))
}

fn comparable(
    occurrence: &XmlOccurrenceClassification,
    index: &CatalogIndex<'_>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Result<XmlSchemaClassComparable, XmlSchemaClassUnavailable>> {
    use XmlSchemaClassUnavailable as Missing;
    crate::analyzer::checkpoint(stop)?;
    let class = match occurrence.outcome() {
        XmlClassOutcome::Unknown(reason) => return Ok(Err(Missing::ClassificationUnknown(reason))),
        XmlClassOutcome::Conflict(reason) => {
            return Ok(Err(Missing::ClassificationConflict(reason)));
        }
        XmlClassOutcome::Classified(class) => class,
    };
    let [declaration] = occurrence.candidates() else {
        return Ok(Err(Missing::MissingDeclarationWitness));
    };
    let Some(native) = index.components.get(declaration) else {
        return Ok(Err(Missing::MissingDeclarationWitness));
    };
    let Some(name) = occurrence.qname().name() else {
        return Ok(Err(Missing::MissingDeclarationWitness));
    };
    if native.component_kind != XmlSchemaComponentKind::GlobalElement
        || native.component_state != XmlSchemaComponentState::Observed
        || occurrence.qname().state() != XmlSchemaQNameState::Expanded
    {
        return Ok(Err(Missing::MissingDeclarationWitness));
    }
    // The sealed classifier resolves this instance QName to this UNIQUE global
    // declaration; no component name is guessed from path, ID or spelling.
    if occurrence.proofs().is_empty()
        || !occurrence.blockers().is_empty()
        || occurrence
            .proofs()
            .iter()
            .any(|proof| proof.class() != class)
    {
        return Ok(Err(Missing::InconsistentProof));
    }
    let mut targets = BTreeSet::new();
    let mut has_named_type = false;
    for proof in occurrence.proofs() {
        for witness in proof.witnesses() {
            crate::analyzer::checkpoint(stop)?;
            let XmlClassificationWitness::Reference {
                source,
                reference_kind,
                target,
                value_span,
                decoded_value_digest,
            } = witness
            else {
                continue;
            };
            if !matches!(
                *reference_kind,
                XmlSchemaReferenceKind::Type
                    | XmlSchemaReferenceKind::Base
                    | XmlSchemaReferenceKind::SubstitutionGroup
                    | XmlSchemaReferenceKind::Element
            ) {
                continue;
            }
            let Some(rows) = index.references.get(source) else {
                return Ok(Err(Missing::MissingReferenceWitness));
            };
            let mut matched = None;
            for row in rows {
                crate::analyzer::checkpoint(stop)?;
                let Some(reference) = &row.reference else {
                    return Err(invalid());
                };
                if reference.kind() == *reference_kind
                    && reference.value_span() == value_span
                    && reference.decoded_value_digest() == *decoded_value_digest
                    && matched.replace(reference).is_some()
                {
                    return Ok(Err(Missing::AmbiguousReferenceWitness));
                }
            }
            let Some(reference) = matched else {
                return Ok(Err(Missing::MissingReferenceWitness));
            };
            if reference.state() != XmlSchemaReferenceState::Unique
                || reference.candidates() != [*target]
                || reference.target().state() != XmlSchemaQNameState::Expanded
            {
                return Ok(Err(Missing::UnsupportedReferenceWitness));
            }
            let Some(target_component) = index.components.get(target) else {
                return Ok(Err(Missing::UnsupportedTargetWitness));
            };
            if target_component.component_state != XmlSchemaComponentState::Observed {
                return Ok(Err(Missing::UnsupportedTargetWitness));
            }
            let target_kind = match (*reference_kind, target_component.component_kind) {
                (
                    XmlSchemaReferenceKind::Type | XmlSchemaReferenceKind::Base,
                    XmlSchemaComponentKind::NamedSimpleType,
                ) => QualifiedWitnessKind::NamedSimpleType,
                (
                    XmlSchemaReferenceKind::Type | XmlSchemaReferenceKind::Base,
                    XmlSchemaComponentKind::NamedComplexType,
                ) => QualifiedWitnessKind::NamedComplexType,
                (
                    XmlSchemaReferenceKind::SubstitutionGroup | XmlSchemaReferenceKind::Element,
                    XmlSchemaComponentKind::GlobalElement,
                ) => QualifiedWitnessKind::GlobalElement,
                _ => return Ok(Err(Missing::UnsupportedTargetWitness)),
            };
            let Some(target_name) = reference.target().name() else {
                return Ok(Err(Missing::UnsupportedTargetWitness));
            };
            budget.charge_serialized(
                &(
                    "xml-class-component-qualified-target",
                    reference_kind,
                    target_kind,
                    target_name,
                ),
                stop,
            )?;
            let normalized = QualifiedProofTarget {
                reference_kind: *reference_kind,
                target_kind,
                name: target_name.clone(),
            };
            has_named_type |= target_kind != QualifiedWitnessKind::GlobalElement;
            targets.insert(normalized);
        }
    }
    if !has_named_type {
        return Ok(Err(Missing::MissingNamedTypeWitness));
    }
    budget.charge_serialized(&("xml-class-component-declaration", name), stop)?;
    let value = XmlSchemaClassComparable {
        class,
        declaration: name.clone(),
        qualified_proof_targets: targets.into_iter().collect(),
    };
    crate::analyzer::checkpoint(stop)?;
    Ok(Ok(value))
}

fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SnapshotInvalid,
        ProjectPhase::View,
        "XML schema-class component disagrees with its sealed native handoff",
    )
}
fn invalid_selection() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::InvalidFingerprintSelection,
        ProjectPhase::View,
        "XML schema-class component selection is invalid",
    )
}

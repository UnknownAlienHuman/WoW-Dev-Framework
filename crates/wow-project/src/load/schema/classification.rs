//! Reviewed schema-class policy over a genuine catalog and BORROWED native indexes.
//! This leaf does not assert project generation, placement, templates or runtime.
mod catalog;
mod model;
mod policy;

pub use model::*;

use std::{
    collections::BTreeSet,
    fmt,
    sync::{Arc, atomic::AtomicBool},
};
use wow_core::{CanonicalResult, ContentDigest};

use crate::{
    ProjectPhase, ProjectResult, disk::checkpoint, load::XmlDocumentIndex,
    platform_source::AdmittedPlatformSource,
};

use super::{
    AdmittedXmlSchema, SchemaBudget, XML_SCHEMA_POLICY_PROFILE, XML_SCHEMA_PROFILE,
    XmlSchemaComponentKind as Kind, XmlSchemaComponentState as State, XmlSchemaQName,
    XmlSchemaQNameState, XmlSchemaReferenceKind as RefKind, invalid, names::Names,
};

use catalog::{CatalogIndex, Link, PathState, count, push, push_reference};

/// Policy interpretation and existing schema ledger bounds are fixed by this
/// version. Source/catalog/anchor choices remain explicit identity inputs.
pub const XML_SCHEMA_CLASSIFICATION_POLICY: &str = "wow-project/reviewed-xml-classification/1";

/// Sealed retained owner. Rebuild by admitting selection against the SAME source
/// and native catalog; serialize-only output cannot reconstruct this capability.
pub struct ReviewedXmlClassificationProfile {
    catalog: Arc<AdmittedXmlSchema>,
    selection: XmlClassificationProfileSelection,
    digest: ContentDigest<CanonicalResult>,
    index: CatalogIndex,
    region_witnesses: Vec<Vec<XmlClassificationWitness>>,
}

impl fmt::Debug for ReviewedXmlClassificationProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReviewedXmlClassificationProfile")
            .field("digest", &self.digest)
            .finish_non_exhaustive()
    }
}

/// One finite ledger across ALL documents of a selected classifier operation.
/// It reuses schema work/row/metadata bounds; it grants no new graph/result budget.
pub struct XmlClassificationBudget {
    inner: SchemaBudget,
    profile: Option<ContentDigest<CanonicalResult>>,
}

impl Default for XmlClassificationBudget {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlClassificationBudget {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: SchemaBudget::new(),
            profile: None,
        }
    }

    fn bind(
        &mut self,
        profile: &ReviewedXmlClassificationProfile,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        checkpoint(stop)?;
        if let Some(expected) = self.profile {
            if expected != profile.digest {
                return Err(invalid("classification operation mixes reviewed profiles"));
            }
            return Ok(());
        }
        self.inner.charge(&profile.selection, stop)?;
        profile.index.charge(&mut self.inner, stop)?;
        self.inner.charge(&profile.region_witnesses, stop)?;
        self.profile = Some(profile.digest);
        Ok(())
    }
}

/// Verify strict intent after decoding; no discovery, source acquisition, parser
/// or authority deserialization. Admission has its own existing finite ledger.
pub fn admit_xml_classification_profile(
    catalog: &Arc<AdmittedXmlSchema>,
    source: &AdmittedPlatformSource,
    selection: &XmlClassificationProfileSelection,
    stop: &AtomicBool,
) -> ProjectResult<Arc<ReviewedXmlClassificationProfile>> {
    checkpoint(stop)?;
    let receipt = catalog.receipt();
    let binding = &selection.binding;
    if receipt.profile() != XML_SCHEMA_PROFILE
        || receipt.policy_profile() != XML_SCHEMA_POLICY_PROFILE
        || binding.source_snapshot_id != receipt.source_snapshot_id()
        || binding.source_profile_digest != receipt.source_profile_digest()
        || binding.content_manifest_digest != receipt.content_manifest_digest()
        || binding.admission_digest != receipt.admission_digest()
        || binding.catalog_selection_digest != receipt.selection_digest()
        || binding.catalog_digest != receipt.digest()
    {
        return Err(invalid(
            "reviewed classification belongs to another source or catalog",
        ));
    }
    catalog.validate_source(source, stop)?;
    let mut budget = SchemaBudget::new();
    let selection = policy::canonical_selection(selection, &mut budget, stop)?;
    let index = CatalogIndex::build(catalog, &mut budget, stop)?;
    let region_witnesses = policy::verify_anchors(catalog, &index, &selection, &mut budget, stop)?;
    let identity = (
        XML_SCHEMA_CLASSIFICATION_POLICY,
        XML_SCHEMA_PROFILE,
        XML_SCHEMA_POLICY_PROFILE,
        &selection,
        &region_witnesses,
    );
    budget.charge(&identity, stop)?;
    let digest = crate::identity::canonical_digest(
        XML_SCHEMA_CLASSIFICATION_POLICY,
        &identity,
        ProjectPhase::Configuration,
    )?;
    checkpoint(stop)?;
    Ok(Arc::new(ReviewedXmlClassificationProfile {
        catalog: Arc::clone(catalog),
        selection,
        digest,
        index,
        region_witnesses,
    }))
}

struct Decision {
    candidates: Vec<XmlSchemaComponentId>,
    outcome: XmlClassOutcome,
    proofs: Vec<XmlClassProof>,
    blockers: Vec<XmlClassBlocker>,
}

impl ReviewedXmlClassificationProfile {
    #[must_use]
    pub const fn selection(&self) -> &XmlClassificationProfileSelection {
        &self.selection
    }
    #[must_use]
    pub const fn binding(&self) -> &XmlClassificationBinding {
        &self.selection.binding
    }
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
    #[must_use]
    pub fn catalog(&self) -> &AdmittedXmlSchema {
        &self.catalog
    }

    /// No addon XML reparse. The caller's graph scope MUST validate this native
    /// retained index against its project generation before projecting support.
    pub fn classify_document(
        &self,
        document: &XmlDocumentIndex,
        budget: &mut XmlClassificationBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<XmlDocumentClassification> {
        budget.bind(self, stop)?;
        let ledger = &mut budget.inner;
        ledger.charge(
            &(
                document.document(),
                document.source_digest(),
                document.digest(),
            ),
            stop,
        )?;
        let mut names = Names::new(document.document());
        let mut occurrences = Vec::new();
        let mut coverage = XmlClassificationCoverage {
            occurrences: 0,
            classified: 0,
            unknown: 0,
            conflicted: 0,
            catalog_issues: self.catalog.issues().len(),
        };
        for record in document.elements() {
            ledger.rows(1, stop)?;
            names.enter(record, ledger, stop)?;
            let qname = names.qname(&record.qualified_name, ledger, stop)?;
            let decision = self.classify_qname(&qname, ledger, stop)?;
            coverage.occurrences = count(coverage.occurrences, 1)?;
            match decision.outcome {
                XmlClassOutcome::Classified(_) => {
                    coverage.classified = count(coverage.classified, 1)?
                }
                XmlClassOutcome::Unknown(_) => coverage.unknown = count(coverage.unknown, 1)?,
                XmlClassOutcome::Conflict(_) => {
                    coverage.conflicted = count(coverage.conflicted, 1)?
                }
            }
            ledger.charge(&(&record.occurrence_id, &record.span), stop)?;
            let occurrence = XmlOccurrenceClassification {
                occurrence: record.occurrence_id.clone(),
                span: record.span.clone(),
                qname,
                candidates: decision.candidates,
                outcome: decision.outcome,
                proofs: decision.proofs,
                blockers: decision.blockers,
            };
            ledger.charge(&occurrence, stop)?;
            occurrences.push(occurrence);
        }
        let identity = (
            XML_SCHEMA_CLASSIFICATION_POLICY,
            self.digest,
            document.document(),
            document.source_digest(),
            document.digest(),
            &coverage,
            &occurrences,
        );
        ledger.charge(&identity, stop)?;
        let digest = crate::identity::canonical_digest(
            "wow-project/xml-document-classification/1",
            &identity,
            ProjectPhase::Snapshot,
        )?;
        checkpoint(stop)?;
        let output = XmlDocumentClassification {
            policy: XML_SCHEMA_CLASSIFICATION_POLICY,
            document: document.document().to_owned(),
            source_digest: document.source_digest(),
            index_digest: document.digest(),
            profile_digest: self.digest,
            catalog_digest: self.catalog.receipt().digest(),
            coverage,
            occurrences,
            digest,
        };
        ledger.charge(&output, stop)?;
        checkpoint(stop)?;
        Ok(output)
    }

    // A QName cannot bypass the instance document's namespace walk at the public API.
    fn classify_qname(
        &self,
        qname: &XmlSchemaQName,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<Decision> {
        budget.visit(1, stop)?;
        let mut decision = Decision {
            candidates: Vec::new(),
            outcome: XmlClassOutcome::Unknown(XmlClassificationUnknown::NoReviewedClass),
            proofs: Vec::new(),
            blockers: Vec::new(),
        };
        let Some(name) = qname.name() else {
            decision.outcome =
                XmlClassOutcome::Unknown(if qname.state() == XmlSchemaQNameState::UnboundPrefix {
                    XmlClassificationUnknown::UnboundPrefix
                } else {
                    XmlClassificationUnknown::InvalidQName
                });
            return Ok(decision);
        };
        let candidates = self.index.candidates(name);
        budget.charge(&candidates, stop)?;
        decision.candidates = candidates.to_vec();
        let [element] = candidates else {
            decision.outcome = if candidates.is_empty() {
                XmlClassOutcome::Unknown(XmlClassificationUnknown::MissingGlobalDeclaration)
            } else {
                XmlClassOutcome::Conflict(XmlClassificationConflict::GlobalDeclarations)
            };
            return Ok(decision);
        };
        let component = self.index.component(&self.catalog, *element)?;
        if component.state() != State::Observed
            || !self
                .index
                .unique_global(&self.catalog, *element, Kind::GlobalElement)?
        {
            decision.outcome =
                XmlClassOutcome::Unknown(XmlClassificationUnknown::UnsupportedDeclaration);
            return Ok(decision);
        }
        if policy::abstract_element(&self.catalog, &self.index, *element)? {
            decision.outcome =
                XmlClassOutcome::Unknown(XmlClassificationUnknown::AbstractOccurrence);
            return Ok(decision);
        }
        self.frame_proof(*element, &mut decision, budget, stop)?;
        self.region_proofs(*element, &mut decision, budget, stop)?;
        for anchor in &self.selection.explicit {
            budget.visit(1, stop)?;
            if anchor.element == *element {
                let mut witnesses = Vec::new();
                push(
                    &mut witnesses,
                    XmlClassificationWitness::Component {
                        component: *element,
                    },
                    budget,
                    stop,
                )?;
                add_proof(
                    &mut decision,
                    XmlSchemaClass::from(anchor.class),
                    witnesses,
                    budget,
                    stop,
                )?;
            }
        }
        let mut classes = BTreeSet::new();
        for proof in &decision.proofs {
            budget.visit(1, stop)?;
            budget.charge(&proof.class, stop)?;
            classes.insert(proof.class);
        }
        decision.outcome = if classes.len() > 1 {
            XmlClassOutcome::Conflict(XmlClassificationConflict::ClassOverlap)
        } else if let Some(blocker) = decision.blockers.first() {
            XmlClassOutcome::Unknown(blocker.reason)
        } else if let Some(class) = classes.first() {
            XmlClassOutcome::Classified(*class)
        } else {
            XmlClassOutcome::Unknown(XmlClassificationUnknown::NoReviewedClass)
        };
        Ok(decision)
    }

    fn frame_proof(
        &self,
        element: XmlSchemaComponentId,
        decision: &mut Decision,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        let mut substitution = self.index.substitution_path(
            &self.catalog,
            element,
            self.selection.frame_like.element,
            budget,
            stop,
        )?;
        match substitution.state {
            PathState::Absent => (),
            PathState::Blocked(reason) => {
                add_blocker(
                    decision,
                    XmlSchemaClass::FrameLike,
                    reason,
                    substitution.witnesses,
                    budget,
                    stop,
                )?;
            }
            PathState::Reached => {
                let mut types = self.index.type_path(
                    &self.catalog,
                    element,
                    self.selection.frame_like.type_component,
                    budget,
                    stop,
                )?;
                substitution.witnesses.append(&mut types.witnesses);
                match types.state {
                    PathState::Reached => add_proof(
                        decision,
                        XmlSchemaClass::FrameLike,
                        substitution.witnesses,
                        budget,
                        stop,
                    )?,
                    PathState::Absent => (),
                    PathState::Blocked(reason) => add_blocker(
                        decision,
                        XmlSchemaClass::FrameLike,
                        reason,
                        substitution.witnesses,
                        budget,
                        stop,
                    )?,
                }
            }
        }
        Ok(())
    }

    fn region_proofs(
        &self,
        element: XmlSchemaComponentId,
        decision: &mut Decision,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        match self
            .index
            .link(&self.catalog, element, RefKind::Type, budget, stop)?
        {
            Link::Absent => (),
            Link::Blocked(reason) => {
                let mut witnesses = Vec::new();
                push(
                    &mut witnesses,
                    XmlClassificationWitness::Component { component: element },
                    budget,
                    stop,
                )?;
                add_blocker(
                    decision,
                    XmlSchemaClass::RegionLike,
                    reason,
                    witnesses,
                    budget,
                    stop,
                )?;
            }
            Link::Unique(reference, target) => {
                for (position, member) in self.selection.region_like.members.iter().enumerate() {
                    budget.visit(1, stop)?;
                    if member.type_component != target {
                        continue;
                    }
                    let admitted = &self.region_witnesses[position];
                    budget.charge(admitted, stop)?;
                    let mut witnesses = admitted.clone();
                    push(
                        &mut witnesses,
                        XmlClassificationWitness::Component { component: element },
                        budget,
                        stop,
                    )?;
                    push_reference(&mut witnesses, reference, target, budget, stop)?;
                    add_proof(
                        decision,
                        XmlSchemaClass::RegionLike,
                        witnesses,
                        budget,
                        stop,
                    )?;
                }
            }
        }
        Ok(())
    }
}

fn add_proof(
    decision: &mut Decision,
    class: XmlSchemaClass,
    witnesses: Vec<XmlClassificationWitness>,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.rows(1, stop)?;
    let proof = XmlClassProof { class, witnesses };
    budget.charge(&proof, stop)?;
    decision.proofs.push(proof);
    Ok(())
}

fn add_blocker(
    decision: &mut Decision,
    class: XmlSchemaClass,
    reason: XmlClassificationUnknown,
    witnesses: Vec<XmlClassificationWitness>,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.rows(1, stop)?;
    let blocker = XmlClassBlocker {
        class,
        reason,
        witnesses,
    };
    budget.charge(&blocker, stop)?;
    decision.blockers.push(blocker);
    Ok(())
}

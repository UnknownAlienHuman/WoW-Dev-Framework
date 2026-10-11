//! Explicit policy intent and serialize-only schema-class observations.
use serde::{Deserialize, Serialize};
use wow_core::{CanonicalResult, ContentDigest, SourceContent};

use crate::load::{
    XmlSourceSpan,
    schema::{XmlSchemaQName, XmlSchemaReferenceKind},
};

pub type XmlSchemaComponentId = ContentDigest<CanonicalResult>;

/// Caller assertions; only admission against a genuine catalog verifies them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XmlClassificationBinding {
    pub source_snapshot_id: String,
    pub source_profile_digest: ContentDigest<CanonicalResult>,
    pub content_manifest_digest: ContentDigest<CanonicalResult>,
    pub admission_digest: ContentDigest<CanonicalResult>,
    pub catalog_selection_digest: ContentDigest<CanonicalResult>,
    pub catalog_digest: ContentDigest<CanonicalResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameLikeAnchors {
    pub element: XmlSchemaComponentId,
    pub type_component: XmlSchemaComponentId,
}

/// Each row asserts both UNIQUE native reference targets, not a tag-name policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionTypeWitness {
    pub element_reference: XmlSchemaComponentId,
    pub element: XmlSchemaComponentId,
    pub type_component: XmlSchemaComponentId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionLikeAnchors {
    pub layers_element: XmlSchemaComponentId,
    pub layers_type: XmlSchemaComponentId,
    pub layers_sequence: XmlSchemaComponentId,
    pub layer_element: XmlSchemaComponentId,
    pub layer_type: XmlSchemaComponentId,
    pub layer_sequence: XmlSchemaComponentId,
    pub choice: XmlSchemaComponentId,
    pub members: [RegionTypeWitness; 5],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlExplicitSchemaClass {
    Structural,
    Script,
    Load,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplicitClassAnchor {
    pub element: XmlSchemaComponentId,
    pub class: XmlExplicitSchemaClass,
}

/// Strict transport intent. It contains no serialized owner capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XmlClassificationProfileSelection {
    pub binding: XmlClassificationBinding,
    pub frame_like: FrameLikeAnchors,
    pub region_like: RegionLikeAnchors,
    pub explicit: Vec<ExplicitClassAnchor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaClass {
    FrameLike,
    RegionLike,
    Structural,
    Script,
    Load,
}

impl From<XmlExplicitSchemaClass> for XmlSchemaClass {
    fn from(value: XmlExplicitSchemaClass) -> Self {
        match value {
            XmlExplicitSchemaClass::Structural => Self::Structural,
            XmlExplicitSchemaClass::Script => Self::Script,
            XmlExplicitSchemaClass::Load => Self::Load,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlClassificationUnknown {
    InvalidQName,
    UnboundPrefix,
    MissingGlobalDeclaration,
    UnsupportedDeclaration,
    AbstractOccurrence,
    NoReviewedClass,
    UnsupportedPath,
    AmbiguousPath,
    Cycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlClassificationConflict {
    GlobalDeclarations,
    ClassOverlap,
}

/// This describes reviewed schema class only. It never describes placement,
/// template membership, instance validity, loading, execution or runtime safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum XmlClassOutcome {
    Classified(XmlSchemaClass),
    Unknown(XmlClassificationUnknown),
    Conflict(XmlClassificationConflict),
}

/// Resolve component IDs through the SAME admitted catalog for source spans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum XmlClassificationWitness {
    Component {
        component: XmlSchemaComponentId,
    },
    Parent {
        child: XmlSchemaComponentId,
        parent: XmlSchemaComponentId,
    },
    Reference {
        source: XmlSchemaComponentId,
        reference_kind: XmlSchemaReferenceKind,
        target: XmlSchemaComponentId,
        value_span: XmlSourceSpan,
        decoded_value_digest: ContentDigest<SourceContent>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlClassProof {
    pub(super) class: XmlSchemaClass,
    pub(super) witnesses: Vec<XmlClassificationWitness>,
}

impl XmlClassProof {
    #[must_use]
    pub const fn class(&self) -> XmlSchemaClass {
        self.class
    }
    #[must_use]
    pub fn witnesses(&self) -> &[XmlClassificationWitness] {
        &self.witnesses
    }
}

/// A path that could not establish or safely exclude a selected schema class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlClassBlocker {
    pub(super) class: XmlSchemaClass,
    pub(super) reason: XmlClassificationUnknown,
    pub(super) witnesses: Vec<XmlClassificationWitness>,
}

impl XmlClassBlocker {
    #[must_use]
    pub const fn class(&self) -> XmlSchemaClass {
        self.class
    }
    #[must_use]
    pub const fn reason(&self) -> XmlClassificationUnknown {
        self.reason
    }
    #[must_use]
    pub fn witnesses(&self) -> &[XmlClassificationWitness] {
        &self.witnesses
    }
}

/// Serialize-only owner result; no public construction or deserialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlOccurrenceClassification {
    pub(super) occurrence: String,
    pub(super) span: XmlSourceSpan,
    pub(super) qname: XmlSchemaQName,
    pub(super) candidates: Vec<XmlSchemaComponentId>,
    pub(super) outcome: XmlClassOutcome,
    pub(super) proofs: Vec<XmlClassProof>,
    pub(super) blockers: Vec<XmlClassBlocker>,
}

impl XmlOccurrenceClassification {
    #[must_use]
    pub fn occurrence(&self) -> &str {
        &self.occurrence
    }
    #[must_use]
    pub const fn span(&self) -> &XmlSourceSpan {
        &self.span
    }
    #[must_use]
    pub const fn qname(&self) -> &XmlSchemaQName {
        &self.qname
    }
    #[must_use]
    pub fn candidates(&self) -> &[XmlSchemaComponentId] {
        &self.candidates
    }
    #[must_use]
    pub const fn outcome(&self) -> XmlClassOutcome {
        self.outcome
    }
    #[must_use]
    pub fn proofs(&self) -> &[XmlClassProof] {
        &self.proofs
    }
    #[must_use]
    pub fn blockers(&self) -> &[XmlClassBlocker] {
        &self.blockers
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlClassificationCoverage {
    pub(super) occurrences: usize,
    pub(super) classified: usize,
    pub(super) unknown: usize,
    pub(super) conflicted: usize,
    pub(super) catalog_issues: usize,
}

impl XmlClassificationCoverage {
    #[must_use]
    pub const fn occurrences(&self) -> usize {
        self.occurrences
    }
    #[must_use]
    pub const fn classified(&self) -> usize {
        self.classified
    }
    #[must_use]
    pub const fn unknown(&self) -> usize {
        self.unknown
    }
    #[must_use]
    pub const fn conflicted(&self) -> usize {
        self.conflicted
    }
    #[must_use]
    pub const fn catalog_issues(&self) -> usize {
        self.catalog_issues
    }
}

/// No generation is asserted here. The graph owner validates the retained
/// load-plan scope/index binding before turning these observations into support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlDocumentClassification {
    pub(super) policy: &'static str,
    pub(super) document: String,
    pub(super) source_digest: ContentDigest<SourceContent>,
    pub(super) index_digest: ContentDigest<CanonicalResult>,
    pub(super) profile_digest: ContentDigest<CanonicalResult>,
    pub(super) catalog_digest: ContentDigest<CanonicalResult>,
    pub(super) coverage: XmlClassificationCoverage,
    pub(super) occurrences: Vec<XmlOccurrenceClassification>,
    pub(super) digest: ContentDigest<CanonicalResult>,
}

impl XmlDocumentClassification {
    #[must_use]
    pub fn document(&self) -> &str {
        &self.document
    }
    #[must_use]
    pub const fn source_digest(&self) -> ContentDigest<SourceContent> {
        self.source_digest
    }
    #[must_use]
    pub const fn index_digest(&self) -> ContentDigest<CanonicalResult> {
        self.index_digest
    }
    #[must_use]
    pub const fn profile_digest(&self) -> ContentDigest<CanonicalResult> {
        self.profile_digest
    }
    #[must_use]
    pub const fn catalog_digest(&self) -> ContentDigest<CanonicalResult> {
        self.catalog_digest
    }
    #[must_use]
    pub const fn coverage(&self) -> &XmlClassificationCoverage {
        &self.coverage
    }
    #[must_use]
    pub fn occurrences(&self) -> &[XmlOccurrenceClassification] {
        &self.occurrences
    }
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
}

//! Sealed source observations. These records do not validate XML instances or
//! establish runtime component classes.
use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, SourceContent};

use crate::load::XmlSourceSpan;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct XmlExpandedName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) namespace: Option<String>,
    pub(super) local_name: String,
}

impl XmlExpandedName {
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaComponentKind {
    Schema,
    GlobalElement,
    LocalElement,
    ElementReference,
    NamedSimpleType,
    AnonymousSimpleType,
    NamedComplexType,
    AnonymousComplexType,
    Sequence,
    Choice,
    All,
    NamedGroup,
    GroupReference,
    NamedAttributeGroup,
    AttributeGroupReference,
    GlobalAttribute,
    LocalAttribute,
    AttributeReference,
    SimpleContent,
    ComplexContent,
    Extension,
    Restriction,
    Enumeration,
    MinInclusive,
    MaxInclusive,
    Facet,
    List,
    Union,
    Any,
    AnyAttribute,
    Include,
    Import,
    Redefine,
    Annotation,
    Documentation,
    AppInfo,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaComponentState {
    Observed,
    Unsupported,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaForm {
    Qualified,
    Unqualified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaAttributeUse {
    Optional,
    Required,
    Prohibited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum XmlSchemaOccurs {
    Count(u64),
    Unbounded,
}

/// An explicit namespace declaration witnessing one QName interpretation.
/// The reserved `xml` binding needs no source declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaNamespaceBinding {
    pub(super) document: String,
    pub(super) occurrence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) prefix: Option<String>,
    pub(super) namespace: String,
    pub(super) span: XmlSourceSpan,
    pub(super) value_span: XmlSourceSpan,
    pub(super) decoded_value_digest: ContentDigest<SourceContent>,
}

impl XmlSchemaNamespaceBinding {
    #[must_use]
    pub fn document(&self) -> &str {
        &self.document
    }
    #[must_use]
    pub fn occurrence(&self) -> &str {
        &self.occurrence
    }
    #[must_use]
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
    #[must_use]
    pub const fn span(&self) -> &XmlSourceSpan {
        &self.span
    }
    #[must_use]
    pub const fn value_span(&self) -> &XmlSourceSpan {
        &self.value_span
    }
    #[must_use]
    pub const fn decoded_value_digest(&self) -> ContentDigest<SourceContent> {
        self.decoded_value_digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaQNameState {
    Expanded,
    Invalid,
    UnboundPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaQName {
    pub(super) lexical: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: Option<XmlExpandedName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) binding: Option<Box<XmlSchemaNamespaceBinding>>,
    pub(super) state: XmlSchemaQNameState,
}

impl XmlSchemaQName {
    #[must_use]
    pub fn lexical(&self) -> &str {
        &self.lexical
    }
    #[must_use]
    pub const fn name(&self) -> Option<&XmlExpandedName> {
        self.name.as_ref()
    }
    #[must_use]
    pub fn binding(&self) -> Option<&XmlSchemaNamespaceBinding> {
        self.binding.as_deref()
    }
    #[must_use]
    pub const fn state(&self) -> XmlSchemaQNameState {
        self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum XmlSchemaAttributeValue {
    Text,
    Boolean(bool),
    Form(XmlSchemaForm),
    Use(XmlSchemaAttributeUse),
    Occurs(XmlSchemaOccurs),
    QName(XmlSchemaQName),
    QNameList(Vec<XmlSchemaQName>),
    Tokens(Vec<String>),
    Invalid,
    Unsupported,
}

/// Original decoded attribute and its bounded, explicitly supported interpretation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaAttribute {
    pub(super) qualified_name: String,
    pub(super) name: XmlExpandedName,
    pub(super) value: String,
    pub(super) span: XmlSourceSpan,
    pub(super) value_span: XmlSourceSpan,
    pub(super) decoded_value_digest: ContentDigest<SourceContent>,
    pub(super) interpretation: XmlSchemaAttributeValue,
}

impl XmlSchemaAttribute {
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }
    #[must_use]
    pub const fn name(&self) -> &XmlExpandedName {
        &self.name
    }
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
    #[must_use]
    pub const fn span(&self) -> &XmlSourceSpan {
        &self.span
    }
    #[must_use]
    pub const fn value_span(&self) -> &XmlSourceSpan {
        &self.value_span
    }
    #[must_use]
    pub const fn decoded_value_digest(&self) -> ContentDigest<SourceContent> {
        self.decoded_value_digest
    }
    #[must_use]
    pub const fn interpretation(&self) -> &XmlSchemaAttributeValue {
        &self.interpretation
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaComponent {
    pub(super) id: ContentDigest<CanonicalResult>,
    pub(super) kind: XmlSchemaComponentKind,
    pub(super) state: XmlSchemaComponentState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: Option<XmlExpandedName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) parent: Option<ContentDigest<CanonicalResult>>,
    pub(super) document: String,
    pub(super) content_digest: ContentDigest<SourceContent>,
    pub(super) occurrence: String,
    pub(super) span: XmlSourceSpan,
    pub(super) attributes: Vec<XmlSchemaAttribute>,
}

impl XmlSchemaComponent {
    #[must_use]
    pub const fn id(&self) -> ContentDigest<CanonicalResult> {
        self.id
    }
    #[must_use]
    pub const fn kind(&self) -> XmlSchemaComponentKind {
        self.kind
    }
    #[must_use]
    pub const fn state(&self) -> XmlSchemaComponentState {
        self.state
    }
    #[must_use]
    pub const fn name(&self) -> Option<&XmlExpandedName> {
        self.name.as_ref()
    }
    #[must_use]
    pub const fn parent(&self) -> Option<ContentDigest<CanonicalResult>> {
        self.parent
    }
    #[must_use]
    pub fn document(&self) -> &str {
        &self.document
    }
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest<SourceContent> {
        self.content_digest
    }
    #[must_use]
    pub fn occurrence(&self) -> &str {
        &self.occurrence
    }
    #[must_use]
    pub const fn span(&self) -> &XmlSourceSpan {
        &self.span
    }
    #[must_use]
    pub fn attributes(&self) -> &[XmlSchemaAttribute] {
        &self.attributes
    }
    /// Ambiguous expanded attributes never select a first or last value.
    #[must_use]
    pub fn attribute(&self, local_name: &str) -> Option<&XmlSchemaAttribute> {
        let mut matches = self.attributes.iter().filter(|attribute| {
            attribute.name.namespace.is_none() && attribute.name.local_name == local_name
        });
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaReferenceKind {
    Type,
    Base,
    SubstitutionGroup,
    Element,
    Attribute,
    AttributeGroup,
    Group,
    ItemType,
    MemberType,
}

/// Resolution describes source catalog membership, not XSD validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaReferenceState {
    Unique,
    Missing,
    Conflict,
    ExternalNamespace,
    InvalidQName,
    UnsupportedContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaReference {
    pub(super) source: ContentDigest<CanonicalResult>,
    pub(super) kind: XmlSchemaReferenceKind,
    pub(super) attribute: String,
    pub(super) value_span: XmlSourceSpan,
    pub(super) decoded_value_digest: ContentDigest<SourceContent>,
    pub(super) target: XmlSchemaQName,
    pub(super) state: XmlSchemaReferenceState,
    pub(super) candidates: Vec<ContentDigest<CanonicalResult>>,
}

impl XmlSchemaReference {
    #[must_use]
    pub const fn source(&self) -> ContentDigest<CanonicalResult> {
        self.source
    }
    #[must_use]
    pub const fn kind(&self) -> XmlSchemaReferenceKind {
        self.kind
    }
    #[must_use]
    pub fn attribute(&self) -> &str {
        &self.attribute
    }
    #[must_use]
    pub const fn value_span(&self) -> &XmlSourceSpan {
        &self.value_span
    }
    #[must_use]
    pub const fn decoded_value_digest(&self) -> ContentDigest<SourceContent> {
        self.decoded_value_digest
    }
    #[must_use]
    pub const fn target(&self) -> &XmlSchemaQName {
        &self.target
    }
    #[must_use]
    pub const fn state(&self) -> XmlSchemaReferenceState {
        self.state
    }
    #[must_use]
    pub fn candidates(&self) -> &[ContentDigest<CanonicalResult>] {
        &self.candidates
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaIssueKind {
    UnsupportedConstruct,
    UnsupportedAttribute,
    UnsupportedDependency,
    InvalidContext,
    InvalidAttribute,
    AmbiguousAttribute,
    InvalidName,
    InvalidQName,
    UnboundPrefix,
    MissingReference,
    ConflictingDeclarations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaIssue {
    pub(super) kind: XmlSchemaIssueKind,
    pub(super) component: ContentDigest<CanonicalResult>,
    pub(super) document: String,
    pub(super) occurrence: String,
    pub(super) span: XmlSourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) attribute: Option<String>,
}

impl XmlSchemaIssue {
    #[must_use]
    pub const fn kind(&self) -> XmlSchemaIssueKind {
        self.kind
    }
    #[must_use]
    pub const fn component(&self) -> ContentDigest<CanonicalResult> {
        self.component
    }
    #[must_use]
    pub fn document(&self) -> &str {
        &self.document
    }
    #[must_use]
    pub fn occurrence(&self) -> &str {
        &self.occurrence
    }
    #[must_use]
    pub const fn span(&self) -> &XmlSourceSpan {
        &self.span
    }
    #[must_use]
    pub fn attribute(&self) -> Option<&str> {
        self.attribute.as_deref()
    }
}

#[derive(Serialize)]
pub(super) struct NormalizedSchema {
    pub(super) components: Vec<XmlSchemaComponent>,
    pub(super) references: Vec<XmlSchemaReference>,
    pub(super) issues: Vec<XmlSchemaIssue>,
}

//! Whole selected closure of one finite reviewed normalized XSD vocabulary.
//! No parser, instance validator, supported-prefix digest or runtime authority.
use super::super::{
    PlatformGraphProvenance, XmlClassificationCatalogWitness, XmlClassificationInclusionManifest,
    producer_budget::ProducerBudget,
};
use super::{invalid, invalid_selection};
use crate::load::{
    XmlSourceSpan,
    schema::{
        AdmittedXmlSchema, XmlExpandedName, XmlSchemaAttributeValue as Value, XmlSchemaComponent,
        XmlSchemaComponentKind as Kind, XmlSchemaComponentState as State, XmlSchemaIssueKind,
        XmlSchemaOccurs, XmlSchemaQNameState, XmlSchemaReference,
        XmlSchemaReferenceKind as RefKind, XmlSchemaReferenceState as RefState,
        classification::XmlClassificationWitness as Witness,
    },
};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};
use serde::{Serialize, Serializer};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};
use wow_core::{CanonicalResult, ContentDigest, SourceSpan};
use wow_graph::{GraphAssertionKind, GraphCoverageState, GraphLocalAssertion};

type Id = ContentDigest<CanonicalResult>;
const XMLNS: &str = "http://www.w3.org/2000/xmlns/";
const XML: &str = "http://www.w3.org/XML/1998/namespace";
pub const XML_SCHEMA_TYPE_SHAPE_RECIPE: &str =
    "wow-project/xml-schema-type-shape/closed-complex-tree/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum XmlSchemaTypeShapeRecipe {
    #[serde(rename = "wow-project/xml-schema-type-shape/closed-complex-tree/1")]
    ClosedComplexTreeV1,
}

/// Balanced preorder; End closes one typed node/reference event.
/// Named type labels are excluded; element and resolved reference QNames are
/// semantic, expanded names. Absent namespaces retain the native omitted field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum XmlSchemaTypeShapeEvent {
    ComplexType {
        mixed: bool,
        abstract_type: bool,
    },
    Sequence {
        min_occurs: u64,
        max_occurs: XmlSchemaOccurs,
    },
    Choice {
        min_occurs: u64,
        max_occurs: XmlSchemaOccurs,
    },
    Element {
        name: XmlExpandedName,
        nillable: bool,
        abstract_element: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        occurs: Option<XmlSchemaTypeShapeOccurs>,
    },
    ElementReference {
        min_occurs: u64,
        max_occurs: XmlSchemaOccurs,
    },
    Reference {
        reference_kind: RefKind,
        target: XmlExpandedName,
    },
    End,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaTypeShapeOccurs {
    min_occurs: u64,
    max_occurs: XmlSchemaOccurs,
}
impl XmlSchemaTypeShapeOccurs {
    #[must_use]
    pub const fn min_occurs(&self) -> u64 {
        self.min_occurs
    }
    #[must_use]
    pub const fn max_occurs(&self) -> XmlSchemaOccurs {
        self.max_occurs
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaTypeShapeComparable {
    events: Vec<XmlSchemaTypeShapeEvent>,
}
impl XmlSchemaTypeShapeComparable {
    #[must_use]
    pub fn events(&self) -> &[XmlSchemaTypeShapeEvent] {
        &self.events
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "reason", content = "detail", rename_all = "snake_case")]
pub enum XmlSchemaTypeShapeUnavailable {
    NativeComponentState(State),
    NativeIssue(XmlSchemaIssueKind),
    UnsupportedComponent(Kind),
    UnsupportedSemanticAttribute(XmlExpandedName),
    UnsupportedTopology,
    MissingExplicitType,
    UnsupportedReferenceKind(RefKind),
    UnresolvedReference(RefState),
    AmbiguousReference,
    Cycle,
}
#[derive(Debug, PartialEq, Eq)]
pub enum XmlSchemaTypeShapeComponent {
    NotSelected,
    Available {
        comparable: XmlSchemaTypeShapeComparable,
        digest: Id,
    },
    Unavailable {
        component: Id,
        reason: XmlSchemaTypeShapeUnavailable,
    },
}
impl Serialize for XmlSchemaTypeShapeComponent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(tag = "state", rename_all = "snake_case")]
        enum Selected<'a> {
            Available {
                recipe: &'static str,
                comparable: &'a XmlSchemaTypeShapeComparable,
                digest: Id,
            },
            Unavailable {
                recipe: &'static str,
                component: Id,
                reason: &'a XmlSchemaTypeShapeUnavailable,
            },
        }
        match self {
            Self::NotSelected => serializer.serialize_str("not_selected"),
            Self::Available { comparable, digest } => Selected::Available {
                recipe: XML_SCHEMA_TYPE_SHAPE_RECIPE,
                comparable,
                digest: *digest,
            }
            .serialize(serializer),
            Self::Unavailable { component, reason } => Selected::Unavailable {
                recipe: XML_SCHEMA_TYPE_SHAPE_RECIPE,
                component: *component,
                reason,
            }
            .serialize(serializer),
        }
    }
}
impl XmlSchemaTypeShapeComponent {
    #[must_use]
    pub const fn comparable(&self) -> Option<&XmlSchemaTypeShapeComparable> {
        match self {
            Self::Available { comparable, .. } => Some(comparable),
            _ => None,
        }
    }
    #[must_use]
    pub const fn digest(&self) -> Option<Id> {
        match self {
            Self::Available { digest, .. } => Some(*digest),
            _ => None,
        }
    }
    #[must_use]
    pub const fn unavailable_reason(&self) -> Option<&XmlSchemaTypeShapeUnavailable> {
        match self {
            Self::Unavailable { reason, .. } => Some(reason),
            _ => None,
        }
    }
    #[must_use]
    pub const fn coverage(&self) -> GraphCoverageState {
        match self {
            Self::Available { .. } => GraphCoverageState::Complete,
            Self::NotSelected | Self::Unavailable { .. } => GraphCoverageState::Partial,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSchemaTypeShapeOmissionReason {
    PresentationAttribute,
    ExpandedNameMaterialized,
    ExpandedReferenceMaterialized,
    NamespaceContextMaterialized,
    EmptyDerivationControl,
    AnnotationPayload,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaTypeShapeOmission {
    component: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    attribute: Option<XmlExpandedName>,
    span: XmlSourceSpan,
    reason: XmlSchemaTypeShapeOmissionReason,
}
impl XmlSchemaTypeShapeOmission {
    #[must_use]
    pub const fn component(&self) -> Id {
        self.component
    }
    #[must_use]
    pub const fn attribute(&self) -> Option<&XmlExpandedName> {
        self.attribute.as_ref()
    }
    #[must_use]
    pub const fn span(&self) -> &XmlSourceSpan {
        &self.span
    }
    #[must_use]
    pub const fn reason(&self) -> XmlSchemaTypeShapeOmissionReason {
        self.reason
    }
}
/// Witnesses/references/omissions are exact custody, NEVER comparable bytes.
/// IDs resolve through the same sealed catalog's native raw support.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct XmlSchemaTypeShapeRecord {
    type_component: Id,
    component: XmlSchemaTypeShapeComponent,
    witnesses: Vec<Witness>,
    references: Vec<XmlSchemaReference>,
    omissions: Vec<XmlSchemaTypeShapeOmission>,
}
impl XmlSchemaTypeShapeRecord {
    #[must_use]
    pub const fn type_component(&self) -> Id {
        self.type_component
    }
    #[must_use]
    pub const fn component(&self) -> &XmlSchemaTypeShapeComponent {
        &self.component
    }
    #[must_use]
    pub fn witnesses(&self) -> &[Witness] {
        &self.witnesses
    }
    #[must_use]
    pub fn references(&self) -> &[XmlSchemaReference] {
        &self.references
    }
    #[must_use]
    pub fn omissions(&self) -> &[XmlSchemaTypeShapeOmission] {
        &self.omissions
    }
}

struct Index<'a> {
    components: BTreeMap<Id, &'a XmlSchemaComponent>,
    children: BTreeMap<Id, Vec<Id>>,
    roots: BTreeMap<Id, Id>,
    references: BTreeMap<Id, Vec<&'a XmlSchemaReference>>,
    incoming_substitutions: BTreeMap<Id, Vec<&'a XmlSchemaReference>>,
    unresolved_substitution: Option<&'a XmlSchemaReference>,
    issues: BTreeMap<Id, Vec<XmlSchemaIssueKind>>,
    support: BTreeMap<Id, &'a XmlClassificationCatalogWitness>,
    reference_support: BTreeMap<Id, Vec<&'a XmlClassificationCatalogWitness>>,
}

/// Strictly ordered unique native TYPE IDs; decoding cannot mint authority.
/// One cumulative existing ProducerBudget: parent passes/retains used across
/// ALL recipe calls. Cancellation/budget/invalid-selection never truncate rows.
pub fn build_selected_xml_schema_type_shapes(
    direct: &PlatformGraphProvenance<'_>,
    recipe: XmlSchemaTypeShapeRecipe,
    ids: &[Id],
    used: usize,
    limit: usize,
    stop: &AtomicBool,
) -> ProjectResult<(Vec<XmlSchemaTypeShapeRecord>, usize)> {
    crate::analyzer::checkpoint(stop)?;
    if limit == 0 {
        return Err(ProjectError::new(
            ProjectErrorCode::InvalidBudgetPolicy,
            ProjectPhase::View,
            "XML type-shape metadata limit must be positive",
        ));
    }
    let mut budget = ProducerBudget::new(used, limit)?;
    let inclusion = direct.xml_classification_inclusion().ok_or_else(invalid)?;
    let reviewed = direct
        .project()
        .configuration()
        .xml_classification_profile()
        .ok_or_else(invalid)?;
    if inclusion.membership().scope() != direct.scope()
        || direct.source().context().context_id() != direct.scope().source_context_id
        || inclusion.classification().binding() != reviewed.binding()
        || inclusion.classification().profile_digest() != reviewed.digest()
    {
        return Err(invalid());
    }
    let catalog = reviewed.catalog();
    budget.charge_serialized(
        &(
            recipe,
            catalog.receipt(),
            reviewed.digest(),
            inclusion.classification().digest(),
            inclusion.membership().digest(),
            direct.project().snapshot_id(),
            direct.project().project_generation(),
            direct.project().configuration().configuration_digest(),
            direct.scope(),
        ),
        stop,
    )?;
    let index = index(catalog, &inclusion, &mut budget, stop)?;
    if ids.len() > index.components.len() {
        return Err(invalid_selection());
    }
    for pair in ids.windows(2) {
        crate::analyzer::checkpoint(stop)?;
        if pair[0] >= pair[1] {
            return Err(invalid_selection());
        }
    }
    for id in ids {
        crate::analyzer::checkpoint(stop)?;
        if !index.components.get(id).is_some_and(|component| {
            matches!(
                component.kind(),
                Kind::NamedSimpleType
                    | Kind::AnonymousSimpleType
                    | Kind::NamedComplexType
                    | Kind::AnonymousComplexType
            )
        }) {
            return Err(invalid_selection());
        }
    }
    budget.charge_serialized(&("xml-type-shape-selection", ids), stop)?;
    let mut records = Vec::new();
    for &id in ids {
        crate::analyzer::checkpoint(stop)?;
        let mut row = XmlSchemaTypeShapeRecord {
            type_component: id,
            component: XmlSchemaTypeShapeComponent::NotSelected,
            witnesses: Vec::new(),
            references: Vec::new(),
            omissions: Vec::new(),
        };
        row.component = match observe(direct, &index, id, &mut row, &mut budget, stop)? {
            Ok(comparable) => {
                budget.charge_serialized(&comparable, stop)?;
                let digest = crate::identity::canonical_digest(
                    XML_SCHEMA_TYPE_SHAPE_RECIPE,
                    &comparable,
                    ProjectPhase::View,
                )?;
                crate::analyzer::checkpoint(stop)?;
                XmlSchemaTypeShapeComponent::Available { comparable, digest }
            }
            Err((component, reason)) => {
                XmlSchemaTypeShapeComponent::Unavailable { component, reason }
            }
        };
        budget.charge_serialized(&row, stop)?;
        records.push(row);
    }
    budget.charge_serialized(&records, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok((records, budget.used()))
}

fn index<'a>(
    catalog: &'a AdmittedXmlSchema,
    inclusion: &'a XmlClassificationInclusionManifest<'_>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Index<'a>> {
    let mut result = Index {
        components: BTreeMap::new(),
        children: BTreeMap::new(),
        roots: BTreeMap::new(),
        references: BTreeMap::new(),
        incoming_substitutions: BTreeMap::new(),
        unresolved_substitution: None,
        issues: BTreeMap::new(),
        support: BTreeMap::new(),
        reference_support: BTreeMap::new(),
    };
    for witness in inclusion.classification().catalog_support().witnesses() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(
            &(
                "xml-type-shape-support-index",
                witness.component_id,
                witness.reference.as_ref().map(XmlSchemaReference::kind),
            ),
            stop,
        )?;
        if witness.reference.is_some() {
            result
                .reference_support
                .entry(witness.component_id)
                .or_default()
                .push(witness);
        } else if result
            .support
            .insert(witness.component_id, witness)
            .is_some()
        {
            return Err(invalid());
        }
    }
    for component in catalog.components() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(
            &(
                "xml-type-shape-native-index",
                component.id(),
                component.parent(),
            ),
            stop,
        )?;
        let support = result.support.get(&component.id()).ok_or_else(invalid)?;
        if support.component_kind != component.kind()
            || support.component_state != component.state()
            || support.component_parent != component.parent()
            || support.document != component.document()
            || support.content_digest != component.content_digest()
            || &support.span != component.span()
            || result
                .components
                .insert(component.id(), component)
                .is_some()
        {
            return Err(invalid());
        }
        let root = if let Some(parent) = component.parent() {
            let enclosing = result.components.get(&parent).ok_or_else(invalid)?;
            if enclosing.document() != component.document() {
                return Err(invalid());
            }
            budget.charge_serialized(
                &("xml-type-shape-child-index", parent, component.id()),
                stop,
            )?;
            result
                .children
                .entry(parent)
                .or_default()
                .push(component.id());
            *result.roots.get(&parent).ok_or_else(invalid)?
        } else {
            if component.kind() != Kind::Schema {
                return Err(invalid());
            }
            component.id()
        };
        budget.charge_serialized(&("xml-type-shape-root-index", component.id(), root), stop)?;
        result.roots.insert(component.id(), root);
    }
    for reference in catalog.references() {
        crate::analyzer::checkpoint(stop)?;
        if !result.components.contains_key(&reference.source()) {
            return Err(invalid());
        }
        budget.charge_serialized(
            &(
                "xml-type-shape-reference-index",
                reference.source(),
                reference.kind(),
                reference.value_span(),
            ),
            stop,
        )?;
        result
            .references
            .entry(reference.source())
            .or_default()
            .push(reference);
        if reference.kind() == RefKind::SubstitutionGroup {
            if reference.state() != RefState::Unique
                || reference.candidates().len() != 1
                || reference.target().state() != XmlSchemaQNameState::Expanded
            {
                result.unresolved_substitution.get_or_insert(reference);
            }
            for &target in reference.candidates() {
                crate::analyzer::checkpoint(stop)?;
                if !result.components.contains_key(&target) {
                    return Err(invalid());
                }
                budget.charge_serialized(
                    &("xml-type-shape-incoming-substitution", target, reference),
                    stop,
                )?;
                result
                    .incoming_substitutions
                    .entry(target)
                    .or_default()
                    .push(reference);
            }
        }
    }
    for issue in catalog.issues() {
        crate::analyzer::checkpoint(stop)?;
        if !result.components.contains_key(&issue.component()) {
            return Err(invalid());
        }
        budget.charge_serialized(
            &(
                "xml-type-shape-issue-index",
                issue.component(),
                issue.kind(),
            ),
            stop,
        )?;
        result
            .issues
            .entry(issue.component())
            .or_default()
            .push(issue.kind());
    }
    Ok(result)
}

#[derive(Serialize)]
enum Task {
    Enter(Id),
    Exit(Id, bool),
    Reference(Id, RefKind),
    EndReference,
}
type Blocked = (Id, XmlSchemaTypeShapeUnavailable);
fn task(
    stack: &mut Vec<Task>,
    value: Task,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.charge_serialized(&("xml-type-shape-task", &value), stop)?;
    stack.push(value);
    Ok(())
}
fn event(
    events: &mut Vec<XmlSchemaTypeShapeEvent>,
    value: XmlSchemaTypeShapeEvent,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.charge_serialized(&value, stop)?;
    events.push(value);
    Ok(())
}
fn witness(
    row: &mut XmlSchemaTypeShapeRecord,
    value: Witness,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.charge_serialized(&value, stop)?;
    row.witnesses.push(value);
    Ok(())
}
fn retain(
    direct: &PlatformGraphProvenance<'_>,
    index: &Index<'_>,
    id: Id,
    row: &mut XmlSchemaTypeShapeRecord,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    let native = index.components.get(&id).ok_or_else(invalid)?;
    let support = index.support.get(&id).ok_or_else(invalid)?;
    // Full borrowed input is charged BEFORE names, spans or reference copies.
    budget.charge_serialized(&(native, support), stop)?;
    let key = GraphLocalAssertion {
        kind: GraphAssertionKind::Entity,
        proposal_id: support.raw_member_proposal_id.as_str().into(),
    };
    let raw = direct.assertion(&key).ok_or_else(invalid)?;
    let handle = direct
        .evidence_catalog()
        .source_handle(&support.source_handle_id)
        .ok_or_else(invalid)?;
    let evidence = direct
        .evidence_catalog()
        .evidence(&support.evidence_id)
        .ok_or_else(invalid)?;
    if handle.path().as_str() != support.document
        || *handle.content_digest() != support.content_digest
        || handle.span() != SourceSpan::whole_file()
        || handle.project_generation() != Some(direct.project().project_generation())
        || handle.reference_generation()
            != Some(direct.project().configuration().reference_generation())
        || evidence.context_id() != direct.scope().source_context_id
        || !evidence
            .source_handle_ids()
            .contains(&support.source_handle_id)
    {
        return Err(invalid());
    }
    // finish validated exact raw admission and catalog ranges. Reuse that
    // sealed capability; never register XSD as a captured Main source_file.
    budget.charge_serialized(&(raw, handle, evidence), stop)?;
    witness(row, Witness::Component { component: id }, budget, stop)?;
    if let Some(parent) = native.parent() {
        witness(row, Witness::Parent { child: id, parent }, budget, stop)?;
    }
    if let Some(references) = index.references.get(&id) {
        for reference in references {
            budget.charge_serialized(reference, stop)?;
            row.references.push((**reference).clone());
        }
    }
    Ok(())
}
fn observed(index: &Index<'_>, id: Id) -> Option<Blocked> {
    let native = index.components.get(&id)?;
    if native.state() != State::Observed {
        return Some((
            id,
            XmlSchemaTypeShapeUnavailable::NativeComponentState(native.state()),
        ));
    }
    index
        .issues
        .get(&id)
        .and_then(|issues| issues.first())
        .map(|issue| (id, XmlSchemaTypeShapeUnavailable::NativeIssue(*issue)))
}
fn omit(
    row: &mut XmlSchemaTypeShapeRecord,
    id: Id,
    attribute: Option<&XmlExpandedName>,
    span: &XmlSourceSpan,
    reason: XmlSchemaTypeShapeOmissionReason,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.charge_serialized(&(id, attribute, span, reason), stop)?;
    row.omissions.push(XmlSchemaTypeShapeOmission {
        component: id,
        attribute: attribute.cloned(),
        span: span.clone(),
        reason,
    });
    Ok(())
}

fn attributes(
    native: &XmlSchemaComponent,
    row: &mut XmlSchemaTypeShapeRecord,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Option<Blocked>> {
    use XmlSchemaTypeShapeOmissionReason as O;
    for attribute in native.attributes() {
        crate::analyzer::checkpoint(stop)?;
        let name = attribute.name();
        let local = name.local_name();
        let omission = if name.namespace() == Some(XMLNS) {
            Some(O::NamespaceContextMaterialized)
        } else if name.namespace() == Some(XML) && matches!(local, "lang" | "space") {
            Some(O::PresentationAttribute)
        } else if name.namespace().is_some() {
            None
        } else {
            match local {
                "id" => Some(O::PresentationAttribute),
                "version" if native.kind() == Kind::Schema => Some(O::PresentationAttribute),
                "source" if matches!(native.kind(), Kind::Documentation | Kind::AppInfo) => {
                    Some(O::PresentationAttribute)
                }
                "targetNamespace" | "elementFormDefault" | "attributeFormDefault"
                    if native.kind() == Kind::Schema =>
                {
                    Some(O::NamespaceContextMaterialized)
                }
                "name" if native.kind() == Kind::NamedComplexType => Some(O::PresentationAttribute),
                "name" | "form"
                    if matches!(native.kind(), Kind::LocalElement | Kind::GlobalElement) =>
                {
                    Some(O::ExpandedNameMaterialized)
                }
                "type" if matches!(native.kind(), Kind::LocalElement | Kind::GlobalElement) => {
                    Some(O::ExpandedReferenceMaterialized)
                }
                "ref" if native.kind() == Kind::ElementReference => {
                    Some(O::ExpandedReferenceMaterialized)
                }
                "blockDefault" | "finalDefault" if native.kind() == Kind::Schema => {
                    if matches!(attribute.interpretation(), Value::Tokens(tokens) if tokens.is_empty())
                    {
                        Some(O::EmptyDerivationControl)
                    } else {
                        None
                    }
                }
                "block" | "final"
                    if matches!(
                        native.kind(),
                        Kind::NamedComplexType | Kind::LocalElement | Kind::GlobalElement
                    ) =>
                {
                    if matches!(attribute.interpretation(), Value::Tokens(tokens) if tokens.is_empty())
                    {
                        Some(O::EmptyDerivationControl)
                    } else {
                        None
                    }
                }
                "mixed"
                    if matches!(
                        native.kind(),
                        Kind::NamedComplexType | Kind::AnonymousComplexType
                    ) =>
                {
                    if matches!(attribute.interpretation(), Value::Boolean(_)) {
                        continue;
                    }
                    None
                }
                "abstract"
                    if matches!(native.kind(), Kind::NamedComplexType | Kind::GlobalElement) =>
                {
                    if matches!(attribute.interpretation(), Value::Boolean(_)) {
                        continue;
                    }
                    None
                }
                "nillable" if matches!(native.kind(), Kind::LocalElement | Kind::GlobalElement) => {
                    if matches!(attribute.interpretation(), Value::Boolean(_)) {
                        continue;
                    }
                    None
                }
                "minOccurs" | "maxOccurs"
                    if matches!(
                        native.kind(),
                        Kind::Sequence | Kind::Choice | Kind::LocalElement | Kind::ElementReference
                    ) =>
                {
                    if matches!(attribute.interpretation(), Value::Occurs(_)) {
                        continue;
                    }
                    None
                }
                _ => None,
            }
        };
        if let Some(reason) = omission {
            omit(
                row,
                native.id(),
                Some(name),
                attribute.span(),
                reason,
                budget,
                stop,
            )?;
        } else {
            budget.charge_serialized(name, stop)?;
            return Ok(Some((
                native.id(),
                XmlSchemaTypeShapeUnavailable::UnsupportedSemanticAttribute(name.clone()),
            )));
        }
    }
    Ok(None)
}
fn boolean(native: &XmlSchemaComponent, name: &str) -> bool {
    matches!(
        native
            .attribute(name)
            .map(|attribute| attribute.interpretation()),
        Some(Value::Boolean(true))
    )
}
fn occurs(native: &XmlSchemaComponent) -> Result<XmlSchemaTypeShapeOccurs, Blocked> {
    let minimum = match native
        .attribute("minOccurs")
        .map(|attribute| attribute.interpretation())
    {
        None => 1,
        Some(Value::Occurs(XmlSchemaOccurs::Count(value))) => *value,
        _ => {
            return Err((
                native.id(),
                XmlSchemaTypeShapeUnavailable::UnsupportedTopology,
            ));
        }
    };
    let maximum = match native
        .attribute("maxOccurs")
        .map(|attribute| attribute.interpretation())
    {
        None => XmlSchemaOccurs::Count(1),
        Some(Value::Occurs(value)) => *value,
        _ => {
            return Err((
                native.id(),
                XmlSchemaTypeShapeUnavailable::UnsupportedTopology,
            ));
        }
    };
    if matches!(maximum, XmlSchemaOccurs::Count(value) if minimum > value) {
        return Err((
            native.id(),
            XmlSchemaTypeShapeUnavailable::UnsupportedTopology,
        ));
    }
    Ok(XmlSchemaTypeShapeOccurs {
        min_occurs: minimum,
        max_occurs: maximum,
    })
}

fn observe(
    direct: &PlatformGraphProvenance<'_>,
    index: &Index<'_>,
    root: Id,
    row: &mut XmlSchemaTypeShapeRecord,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Result<XmlSchemaTypeShapeComparable, Blocked>> {
    use XmlSchemaTypeShapeEvent as E;
    use XmlSchemaTypeShapeUnavailable as U;
    let mut active = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    let mut stack = Vec::new();
    let mut events = Vec::new();
    task(&mut stack, Task::Enter(root), budget, stop)?;
    while let Some(next) = stack.pop() {
        crate::analyzer::checkpoint(stop)?;
        match next {
            Task::Exit(id, emit) => {
                if !active.remove(&id) {
                    return Err(invalid());
                }
                if emit {
                    event(&mut events, E::End, budget, stop)?;
                }
            }
            Task::EndReference => event(&mut events, E::End, budget, stop)?,
            Task::Reference(source, kind) => {
                let references = index.references.get(&source).ok_or_else(invalid)?;
                let mut matches = references
                    .iter()
                    .filter(|reference| reference.kind() == kind);
                let reference = *matches.next().ok_or_else(invalid)?;
                if matches.next().is_some() {
                    return Ok(Err((source, U::AmbiguousReference)));
                }
                if reference.state() != RefState::Unique
                    || reference.candidates().len() != 1
                    || reference.target().state() != XmlSchemaQNameState::Expanded
                {
                    return Ok(Err((source, U::UnresolvedReference(reference.state()))));
                }
                let target = reference.candidates()[0];
                let target_component = index.components.get(&target).ok_or_else(invalid)?;
                let expected = match kind {
                    RefKind::Type => Kind::NamedComplexType,
                    RefKind::Element => Kind::GlobalElement,
                    _ => return Ok(Err((source, U::UnsupportedReferenceKind(kind)))),
                };
                if target_component.kind() != expected {
                    retain(direct, index, target, row, budget, stop)?;
                    return Ok(Err((
                        target,
                        U::UnsupportedComponent(target_component.kind()),
                    )));
                }
                let name = reference.target().name().ok_or_else(invalid)?;
                if target_component.name() != Some(name) {
                    return Err(invalid());
                }
                let mut support_count = 0;
                for support in index.reference_support.get(&source).into_iter().flatten() {
                    crate::analyzer::checkpoint(stop)?;
                    if support.reference.as_ref() == Some(reference) {
                        budget.charge_serialized(support, stop)?;
                        support_count += 1;
                    }
                }
                if support_count != 1 {
                    return Err(invalid());
                }
                budget.charge_serialized(reference, stop)?;
                witness(
                    row,
                    Witness::Reference {
                        source,
                        reference_kind: kind,
                        target,
                        value_span: reference.value_span().clone(),
                        decoded_value_digest: reference.decoded_value_digest(),
                    },
                    budget,
                    stop,
                )?;
                event(
                    &mut events,
                    E::Reference {
                        reference_kind: kind,
                        target: name.clone(),
                    },
                    budget,
                    stop,
                )?;
                task(&mut stack, Task::EndReference, budget, stop)?;
                task(&mut stack, Task::Enter(target), budget, stop)?;
            }
            Task::Enter(id) => {
                retain(direct, index, id, row, budget, stop)?;
                if let Some(reason) = observed(index, id) {
                    return Ok(Err(reason));
                }
                budget.charge_serialized(&("xml-type-shape-active", id), stop)?;
                if !active.insert(id) {
                    return Ok(Err((id, U::Cycle)));
                }
                let native = *index.components.get(&id).ok_or_else(invalid)?;
                if native.kind() == Kind::GlobalElement {
                    if let Some(reference) = index.unresolved_substitution {
                        retain(direct, index, reference.source(), row, budget, stop)?;
                        return Ok(Err((
                            reference.source(),
                            U::UnresolvedReference(reference.state()),
                        )));
                    }
                    if let Some(reference) = index
                        .incoming_substitutions
                        .get(&id)
                        .and_then(|references| references.first())
                    {
                        retain(direct, index, reference.source(), row, budget, stop)?;
                        return Ok(Err((
                            reference.source(),
                            U::UnsupportedReferenceKind(RefKind::SubstitutionGroup),
                        )));
                    }
                }
                let schema = *index.roots.get(&id).ok_or_else(invalid)?;
                if !contexts.contains(&schema) {
                    budget.charge_serialized(&("xml-type-shape-context", schema), stop)?;
                    contexts.insert(schema);
                    retain(direct, index, schema, row, budget, stop)?;
                    if let Some(reason) = observed(index, schema) {
                        return Ok(Err(reason));
                    }
                    let context = index.components.get(&schema).ok_or_else(invalid)?;
                    if let Some(reason) = attributes(context, row, budget, stop)? {
                        return Ok(Err(reason));
                    }
                    // Dependency declarations may add unknown external type structure.
                    // No unused-import assumption or builtin namespace whitelist.
                    for child in index.children.get(&schema).into_iter().flatten() {
                        crate::analyzer::checkpoint(stop)?;
                        budget.charge_serialized(
                            &("xml-type-shape-dependency-check", schema, child),
                            stop,
                        )?;
                        let dependency = index.components.get(child).ok_or_else(invalid)?;
                        if matches!(
                            dependency.kind(),
                            Kind::Include | Kind::Import | Kind::Redefine
                        ) {
                            retain(direct, index, *child, row, budget, stop)?;
                            return Ok(Err((*child, U::UnsupportedComponent(dependency.kind()))));
                        }
                    }
                }
                if let Some(reason) = attributes(native, row, budget, stop)? {
                    return Ok(Err(reason));
                }
                let children = index.children.get(&id).map_or(&[][..], Vec::as_slice);
                let mut semantic = Vec::new();
                for &child in children {
                    crate::analyzer::checkpoint(stop)?;
                    if index.components.get(&child).ok_or_else(invalid)?.kind() != Kind::Annotation
                    {
                        budget
                            .charge_serialized(&("xml-type-shape-semantic-child", child), stop)?;
                        semantic.push(child);
                    }
                }
                let references = index.references.get(&id).map_or(&[][..], Vec::as_slice);
                let mut selected_reference = None;
                for reference in references {
                    crate::analyzer::checkpoint(stop)?;
                    let allowed = match native.kind() {
                        Kind::LocalElement | Kind::GlobalElement => RefKind::Type,
                        Kind::ElementReference => RefKind::Element,
                        _ => return Ok(Err((id, U::UnsupportedReferenceKind(reference.kind())))),
                    };
                    if reference.kind() != allowed {
                        return Ok(Err((id, U::UnsupportedReferenceKind(reference.kind()))));
                    }
                    if selected_reference.replace(allowed).is_some() {
                        return Ok(Err((id, U::AmbiguousReference)));
                    }
                }
                let node = match native.kind() {
                    Kind::NamedComplexType | Kind::AnonymousComplexType => {
                        if semantic.len() > 1 {
                            return Ok(Err((id, U::UnsupportedTopology)));
                        }
                        if let Some(child) = semantic.first() {
                            let component = index.components[child];
                            if !matches!(component.kind(), Kind::Sequence | Kind::Choice) {
                                retain(direct, index, *child, row, budget, stop)?;
                                return Ok(Err((
                                    *child,
                                    U::UnsupportedComponent(component.kind()),
                                )));
                            }
                        }
                        Some(E::ComplexType {
                            mixed: boolean(native, "mixed"),
                            abstract_type: boolean(native, "abstract"),
                        })
                    }
                    Kind::Sequence | Kind::Choice => {
                        for &child in &semantic {
                            crate::analyzer::checkpoint(stop)?;
                            let component = index.components[&child];
                            if !matches!(
                                component.kind(),
                                Kind::Sequence
                                    | Kind::Choice
                                    | Kind::LocalElement
                                    | Kind::ElementReference
                            ) {
                                retain(direct, index, child, row, budget, stop)?;
                                return Ok(Err((child, U::UnsupportedComponent(component.kind()))));
                            }
                        }
                        if native.kind() == Kind::Choice && semantic.is_empty() {
                            return Ok(Err((id, U::UnsupportedTopology)));
                        }
                        let cardinality = match occurs(native) {
                            Ok(value) => value,
                            Err(reason) => return Ok(Err(reason)),
                        };
                        Some(if native.kind() == Kind::Sequence {
                            E::Sequence {
                                min_occurs: cardinality.min_occurs,
                                max_occurs: cardinality.max_occurs,
                            }
                        } else {
                            E::Choice {
                                min_occurs: cardinality.min_occurs,
                                max_occurs: cardinality.max_occurs,
                            }
                        })
                    }
                    Kind::LocalElement | Kind::GlobalElement => {
                        if (selected_reference.is_some() && !semantic.is_empty())
                            || semantic.len() > 1
                        {
                            return Ok(Err((id, U::UnsupportedTopology)));
                        }
                        if let Some(child) = semantic.first() {
                            let component = index.components[child];
                            if component.kind() != Kind::AnonymousComplexType {
                                retain(direct, index, *child, row, budget, stop)?;
                                return Ok(Err((
                                    *child,
                                    U::UnsupportedComponent(component.kind()),
                                )));
                            }
                        }
                        if selected_reference.is_none() && semantic.is_empty() {
                            return Ok(Err((id, U::MissingExplicitType)));
                        }
                        let cardinality = if native.kind() == Kind::LocalElement {
                            Some(match occurs(native) {
                                Ok(value) => value,
                                Err(reason) => return Ok(Err(reason)),
                            })
                        } else {
                            None
                        };
                        Some(E::Element {
                            name: native.name().ok_or_else(invalid)?.clone(),
                            nillable: boolean(native, "nillable"),
                            abstract_element: boolean(native, "abstract"),
                            occurs: cardinality,
                        })
                    }
                    Kind::ElementReference => {
                        if !semantic.is_empty() || selected_reference != Some(RefKind::Element) {
                            return Ok(Err((id, U::UnsupportedTopology)));
                        }
                        let cardinality = match occurs(native) {
                            Ok(value) => value,
                            Err(reason) => return Ok(Err(reason)),
                        };
                        Some(E::ElementReference {
                            min_occurs: cardinality.min_occurs,
                            max_occurs: cardinality.max_occurs,
                        })
                    }
                    Kind::Annotation | Kind::Documentation | Kind::AppInfo => {
                        for &child in children {
                            crate::analyzer::checkpoint(stop)?;
                            let component = index.components[&child];
                            if !matches!(
                                component.kind(),
                                Kind::Annotation | Kind::Documentation | Kind::AppInfo
                            ) {
                                retain(direct, index, child, row, budget, stop)?;
                                return Ok(Err((child, U::UnsupportedComponent(component.kind()))));
                            }
                        }
                        omit(
                            row,
                            id,
                            None,
                            native.span(),
                            XmlSchemaTypeShapeOmissionReason::AnnotationPayload,
                            budget,
                            stop,
                        )?;
                        None
                    }
                    kind => return Ok(Err((id, U::UnsupportedComponent(kind)))),
                };
                let emit = node.is_some();
                if let Some(value) = node {
                    event(&mut events, value, budget, stop)?;
                }
                task(&mut stack, Task::Exit(id, emit), budget, stop)?;
                for &child in children.iter().rev() {
                    task(&mut stack, Task::Enter(child), budget, stop)?;
                }
                if let Some(kind) = selected_reference {
                    task(&mut stack, Task::Reference(id, kind), budget, stop)?;
                }
            }
        }
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(Ok(XmlSchemaTypeShapeComparable { events }))
}

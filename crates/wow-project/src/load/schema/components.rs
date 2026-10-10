//! Reviewed XSD component observations over native XML records, without parsing
//! source again or evaluating instance/runtime semantics.
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};

use wow_core::{CanonicalResult, ContentDigest};

use crate::{
    ProjectPhase, ProjectResult,
    load::{XmlAttributeRecord, XmlDocumentIndex, XmlElementRecord, XmlSourceSpan},
};

use super::{
    SchemaBudget, exhausted, invalid,
    model::*,
    names::{self, Names, ResolvedName},
};

type Kind = XmlSchemaComponentKind;
type State = XmlSchemaComponentState;

#[derive(Clone, Copy)]
struct SchemaContext<'a> {
    namespace: Option<&'a str>,
    element_form: XmlSchemaForm,
    attribute_form: XmlSchemaForm,
}

struct Frame<'a> {
    record: &'a XmlElementRecord,
    index: usize,
    kind: Kind,
}

#[derive(serde::Serialize)]
struct ComponentIdentity<'a> {
    scope: ContentDigest<CanonicalResult>,
    document: &'a str,
    content_digest: wow_core::ContentDigest<wow_core::SourceContent>,
    occurrence: &'a str,
    kind: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<ContentDigest<CanonicalResult>>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
enum SymbolSpace {
    Element,
    Type,
    Attribute,
    Group,
    AttributeGroup,
}

pub(super) fn normalize(
    documents: &[XmlDocumentIndex],
    scope: ContentDigest<CanonicalResult>,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<NormalizedSchema> {
    let mut output = NormalizedSchema {
        components: Vec::new(),
        references: Vec::new(),
        issues: Vec::new(),
    };
    for document in documents {
        budget.visit(1, stop)?;
        let mut names = Names::new(document.document());
        let mut stack: Vec<Frame<'_>> = Vec::new();
        let mut context = None;
        for record in document.elements() {
            budget.visit(1, stop)?;
            names.enter(record, budget, stop)?;
            let expanded = names.element(record, budget, stop)?;
            let parent_occurrence = record.parent_occurrence_id.as_deref();
            while stack
                .last()
                .is_some_and(|frame| Some(frame.record.occurrence_id.as_str()) != parent_occurrence)
            {
                budget.visit(1, stop)?;
                stack.pop();
            }
            let parent_kind = stack.last().map(|frame| frame.kind);
            if parent_kind.is_none() {
                if expanded.namespace != Some(names::XSD)
                    || expanded.local != "schema"
                    || context.is_some()
                {
                    return Err(invalid("selected schema root is not an XSD schema"));
                }
                context = Some(schema_context(record, budget, stop)?);
            }
            let context = context.ok_or_else(|| invalid("schema context is missing"))?;
            let kind = component_kind(expanded, record, parent_kind, budget, stop)?;
            let parent = stack.last().map(|frame| output.components[frame.index].id);
            let inherited = stack
                .last()
                .map(|frame| output.components[frame.index].state);
            let identity = ComponentIdentity {
                scope,
                document: document.document(),
                content_digest: document.source_digest(),
                occurrence: &record.occurrence_id,
                kind,
                parent,
            };
            budget.charge(&identity, stop)?;
            let id = crate::identity::canonical_digest(
                "wow-project/xml-schema-component/1",
                &identity,
                ProjectPhase::Inventory,
            )?;
            let mut state = if inherited.is_some_and(|state| state != State::Observed) {
                State::Unsupported
            } else {
                State::Observed
            };
            if unsupported(kind) {
                state = State::Unsupported;
                let issue = if matches!(kind, Kind::Include | Kind::Import | Kind::Redefine) {
                    XmlSchemaIssueKind::UnsupportedDependency
                } else {
                    XmlSchemaIssueKind::UnsupportedConstruct
                };
                issue_for(
                    &mut output.issues,
                    issue,
                    id,
                    document.document(),
                    record,
                    None,
                    budget,
                    stop,
                )?;
            } else if !valid_context(kind, parent_kind) {
                state = State::Invalid;
                issue_for(
                    &mut output.issues,
                    XmlSchemaIssueKind::InvalidContext,
                    id,
                    document.document(),
                    record,
                    None,
                    budget,
                    stop,
                )?;
            }
            let (name, invalid_name) = component_name(kind, record, context, budget, stop)?;
            if invalid_name {
                state = State::Invalid;
                issue_for(
                    &mut output.issues,
                    XmlSchemaIssueKind::InvalidName,
                    id,
                    document.document(),
                    record,
                    raw(record, "name", budget, stop)?,
                    budget,
                    stop,
                )?;
            }
            let attributes = attributes(
                &names,
                record,
                kind,
                id,
                document.document(),
                &mut state,
                &mut output.references,
                &mut output.issues,
                budget,
                stop,
            )?;
            if output.components.len() >= super::budget::MAX_COMPONENTS {
                return Err(exhausted());
            }
            budget.rows(1, stop)?;
            budget.charge(
                &(
                    id,
                    kind,
                    state,
                    &name,
                    parent,
                    document.document(),
                    document.source_digest(),
                    &record.occurrence_id,
                    &record.span,
                    &attributes,
                ),
                stop,
            )?;
            let index = output.components.len();
            output.components.push(XmlSchemaComponent {
                id,
                kind,
                state,
                name,
                parent,
                document: document.document().to_owned(),
                content_digest: document.source_digest(),
                occurrence: record.occurrence_id.clone(),
                span: record.span.clone(),
                attributes,
            });
            budget.charge(&(&record.occurrence_id, index, kind), stop)?;
            stack.push(Frame {
                record,
                index,
                kind,
            });
        }
        if context.is_none() {
            return Err(invalid("selected schema has no root"));
        }
    }
    validate_declarations(&mut output, budget, stop)?;
    propagate_context(&mut output, budget, stop)?;
    resolve_references(&mut output, budget, stop)?;
    budget.visit(1, stop)?;
    Ok(output)
}

fn raw<'a>(
    record: &'a XmlElementRecord,
    name: &str,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<Option<&'a XmlAttributeRecord>> {
    for attribute in &record.attributes {
        budget.visit(1, stop)?;
        if attribute.qualified_name == name {
            return Ok(Some(attribute));
        }
    }
    Ok(None)
}

fn schema_context<'a>(
    record: &'a XmlElementRecord,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<SchemaContext<'a>> {
    let namespace = raw(record, "targetNamespace", budget, stop)?.map(XmlAttributeRecord::value);
    if let Some(namespace) = namespace {
        if namespace.is_empty() {
            return Err(invalid("schema target namespace is empty"));
        }
        for character in namespace.chars() {
            budget.visit(1, stop)?;
            if names::xml_space(character) {
                return Err(invalid("schema target namespace contains whitespace"));
            }
        }
    }
    let element_form = match raw(record, "elementFormDefault", budget, stop)? {
        Some(attribute) => {
            budget.visit(attribute.value().len().max(1), stop)?;
            form(attribute.value())
                .ok_or_else(|| invalid("schema element form default is invalid"))?
        }
        None => XmlSchemaForm::Unqualified,
    };
    let attribute_form = match raw(record, "attributeFormDefault", budget, stop)? {
        Some(attribute) => {
            budget.visit(attribute.value().len().max(1), stop)?;
            form(attribute.value())
                .ok_or_else(|| invalid("schema attribute form default is invalid"))?
        }
        None => XmlSchemaForm::Unqualified,
    };
    Ok(SchemaContext {
        namespace,
        element_form,
        attribute_form,
    })
}

fn component_kind(
    name: ResolvedName<'_>,
    record: &XmlElementRecord,
    parent: Option<Kind>,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<Kind> {
    if name.namespace != Some(names::XSD) {
        return Ok(Kind::Unsupported);
    }
    let global = parent == Some(Kind::Schema);
    Ok(match name.local {
        "schema" => Kind::Schema,
        "element" => {
            if raw(record, "ref", budget, stop)?.is_some() {
                Kind::ElementReference
            } else if global {
                Kind::GlobalElement
            } else {
                Kind::LocalElement
            }
        }
        "simpleType" => {
            if raw(record, "name", budget, stop)?.is_some() {
                Kind::NamedSimpleType
            } else {
                Kind::AnonymousSimpleType
            }
        }
        "complexType" => {
            if raw(record, "name", budget, stop)?.is_some() {
                Kind::NamedComplexType
            } else {
                Kind::AnonymousComplexType
            }
        }
        "sequence" => Kind::Sequence,
        "choice" => Kind::Choice,
        "all" => Kind::All,
        "group" => {
            if raw(record, "ref", budget, stop)?.is_some() {
                Kind::GroupReference
            } else {
                Kind::NamedGroup
            }
        }
        "attributeGroup" => {
            if raw(record, "ref", budget, stop)?.is_some() {
                Kind::AttributeGroupReference
            } else {
                Kind::NamedAttributeGroup
            }
        }
        "attribute" => {
            if raw(record, "ref", budget, stop)?.is_some() {
                Kind::AttributeReference
            } else if global {
                Kind::GlobalAttribute
            } else {
                Kind::LocalAttribute
            }
        }
        "simpleContent" => Kind::SimpleContent,
        "complexContent" => Kind::ComplexContent,
        "extension" => Kind::Extension,
        "restriction" => Kind::Restriction,
        "enumeration" => Kind::Enumeration,
        "minInclusive" => Kind::MinInclusive,
        "maxInclusive" => Kind::MaxInclusive,
        "length" | "minLength" | "maxLength" | "pattern" | "whiteSpace" | "minExclusive"
        | "maxExclusive" | "totalDigits" | "fractionDigits" => Kind::Facet,
        "list" => Kind::List,
        "union" => Kind::Union,
        "any" => Kind::Any,
        "anyAttribute" => Kind::AnyAttribute,
        "include" => Kind::Include,
        "import" => Kind::Import,
        "redefine" => Kind::Redefine,
        "annotation" => Kind::Annotation,
        "documentation" => Kind::Documentation,
        "appinfo" => Kind::AppInfo,
        _ => Kind::Unsupported,
    })
}

fn unsupported(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Unsupported
            | Kind::Facet
            | Kind::List
            | Kind::Union
            | Kind::Any
            | Kind::AnyAttribute
            | Kind::Include
            | Kind::Import
            | Kind::Redefine
    )
}

fn complex_type(kind: Kind) -> bool {
    matches!(kind, Kind::NamedComplexType | Kind::AnonymousComplexType)
}

fn valid_context(kind: Kind, parent: Option<Kind>) -> bool {
    let Some(parent) = parent else {
        return kind == Kind::Schema;
    };
    match kind {
        Kind::Schema => false,
        Kind::GlobalElement
        | Kind::NamedSimpleType
        | Kind::NamedComplexType
        | Kind::NamedGroup
        | Kind::NamedAttributeGroup
        | Kind::GlobalAttribute => parent == Kind::Schema,
        Kind::LocalElement | Kind::ElementReference => {
            matches!(parent, Kind::Sequence | Kind::Choice | Kind::All)
        }
        Kind::AnonymousComplexType => matches!(parent, Kind::GlobalElement | Kind::LocalElement),
        Kind::AnonymousSimpleType => matches!(
            parent,
            Kind::GlobalElement
                | Kind::LocalElement
                | Kind::GlobalAttribute
                | Kind::LocalAttribute
                | Kind::Restriction
                | Kind::List
                | Kind::Union
        ),
        Kind::Sequence | Kind::Choice | Kind::GroupReference => {
            complex_type(parent)
                || matches!(
                    parent,
                    Kind::Extension
                        | Kind::Restriction
                        | Kind::NamedGroup
                        | Kind::Sequence
                        | Kind::Choice
                )
        }
        Kind::All => {
            complex_type(parent)
                || matches!(
                    parent,
                    Kind::Extension | Kind::Restriction | Kind::NamedGroup
                )
        }
        Kind::LocalAttribute | Kind::AttributeReference | Kind::AttributeGroupReference => {
            complex_type(parent)
                || matches!(
                    parent,
                    Kind::Extension | Kind::Restriction | Kind::NamedAttributeGroup
                )
        }
        Kind::SimpleContent | Kind::ComplexContent => complex_type(parent),
        Kind::Extension => matches!(parent, Kind::SimpleContent | Kind::ComplexContent),
        Kind::Restriction => matches!(
            parent,
            Kind::NamedSimpleType
                | Kind::AnonymousSimpleType
                | Kind::SimpleContent
                | Kind::ComplexContent
        ),
        Kind::Enumeration | Kind::MinInclusive | Kind::MaxInclusive | Kind::Facet => {
            parent == Kind::Restriction
        }
        Kind::Documentation | Kind::AppInfo => parent == Kind::Annotation,
        Kind::Annotation => !matches!(parent, Kind::Documentation | Kind::AppInfo),
        _ => false,
    }
}

fn component_name(
    kind: Kind,
    record: &XmlElementRecord,
    context: SchemaContext<'_>,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<(Option<XmlExpandedName>, bool)> {
    let named = matches!(
        kind,
        Kind::GlobalElement
            | Kind::LocalElement
            | Kind::NamedSimpleType
            | Kind::NamedComplexType
            | Kind::NamedGroup
            | Kind::NamedAttributeGroup
            | Kind::GlobalAttribute
            | Kind::LocalAttribute
    );
    if !named {
        return Ok((None, false));
    }
    let Some(attribute) = raw(record, "name", budget, stop)? else {
        return Ok((None, true));
    };
    budget.visit(attribute.value().len().max(1), stop)?;
    let local = attribute.value().trim_matches(names::xml_space);
    if !names::ncname(local, budget, stop)? {
        return Ok((None, true));
    }
    let namespace = if matches!(kind, Kind::LocalElement | Kind::LocalAttribute) {
        let declared = match raw(record, "form", budget, stop)? {
            Some(attribute) => {
                budget.visit(attribute.value().len().max(1), stop)?;
                match form(attribute.value()) {
                    Some(form) => form,
                    None => return Ok((None, true)),
                }
            }
            None => {
                if kind == Kind::LocalElement {
                    context.element_form
                } else {
                    context.attribute_form
                }
            }
        };
        if declared == XmlSchemaForm::Qualified {
            context.namespace
        } else {
            None
        }
    } else {
        context.namespace
    };
    budget.charge(&(namespace, local), stop)?;
    Ok((
        Some(XmlExpandedName {
            namespace: namespace.map(str::to_owned),
            local_name: local.to_owned(),
        }),
        false,
    ))
}

fn allowed_attribute(kind: Kind, name: &str) -> bool {
    if name == "id" {
        return true;
    }
    match kind {
        Kind::Schema => matches!(
            name,
            "targetNamespace"
                | "elementFormDefault"
                | "attributeFormDefault"
                | "blockDefault"
                | "finalDefault"
                | "version"
        ),
        Kind::GlobalElement => matches!(
            name,
            "name"
                | "type"
                | "substitutionGroup"
                | "abstract"
                | "nillable"
                | "default"
                | "fixed"
                | "block"
                | "final"
        ),
        Kind::LocalElement => matches!(
            name,
            "name"
                | "type"
                | "form"
                | "minOccurs"
                | "maxOccurs"
                | "nillable"
                | "default"
                | "fixed"
                | "block"
        ),
        Kind::ElementReference => matches!(name, "ref" | "minOccurs" | "maxOccurs"),
        Kind::NamedSimpleType | Kind::AnonymousSimpleType => matches!(name, "name" | "final"),
        Kind::NamedComplexType | Kind::AnonymousComplexType => {
            matches!(name, "name" | "abstract" | "mixed" | "block" | "final")
        }
        Kind::Sequence | Kind::Choice | Kind::All => matches!(name, "minOccurs" | "maxOccurs"),
        Kind::NamedGroup | Kind::NamedAttributeGroup => name == "name",
        Kind::GroupReference => matches!(name, "ref" | "minOccurs" | "maxOccurs"),
        Kind::AttributeGroupReference => name == "ref",
        Kind::GlobalAttribute => matches!(name, "name" | "type" | "default" | "fixed"),
        Kind::LocalAttribute => {
            matches!(name, "name" | "type" | "form" | "use" | "default" | "fixed")
        }
        Kind::AttributeReference => matches!(name, "ref" | "use" | "default" | "fixed"),
        Kind::ComplexContent => name == "mixed",
        Kind::SimpleContent => false,
        Kind::Extension | Kind::Restriction => name == "base",
        Kind::Enumeration | Kind::MinInclusive | Kind::MaxInclusive | Kind::Facet => {
            matches!(name, "value" | "fixed")
        }
        Kind::List => name == "itemType",
        Kind::Union => name == "memberTypes",
        Kind::Any | Kind::AnyAttribute => matches!(
            name,
            "namespace" | "processContents" | "minOccurs" | "maxOccurs"
        ),
        Kind::Include | Kind::Redefine => name == "schemaLocation",
        Kind::Import => matches!(name, "namespace" | "schemaLocation"),
        Kind::Documentation | Kind::AppInfo => name == "source",
        Kind::Annotation | Kind::Unsupported => false,
    }
}

fn reference_kind(kind: Kind, attribute: &str) -> Option<XmlSchemaReferenceKind> {
    use XmlSchemaReferenceKind as Reference;
    match attribute {
        "type" => Some(Reference::Type),
        "base" => Some(Reference::Base),
        "substitutionGroup" => Some(Reference::SubstitutionGroup),
        "itemType" => Some(Reference::ItemType),
        "memberTypes" => Some(Reference::MemberType),
        "ref" => match kind {
            Kind::ElementReference => Some(Reference::Element),
            Kind::AttributeReference => Some(Reference::Attribute),
            Kind::AttributeGroupReference => Some(Reference::AttributeGroup),
            Kind::GroupReference => Some(Reference::Group),
            _ => None,
        },
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn attributes<'a>(
    names: &Names<'a>,
    record: &'a XmlElementRecord,
    kind: Kind,
    id: ContentDigest<CanonicalResult>,
    document: &str,
    state: &mut State,
    references: &mut Vec<XmlSchemaReference>,
    issues: &mut Vec<XmlSchemaIssue>,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<Vec<XmlSchemaAttribute>> {
    let mut expanded = Vec::new();
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for attribute in &record.attributes {
        budget.visit(1, stop)?;
        let name = names.attribute(attribute, budget, stop)?;
        let key = (name.namespace, name.local);
        budget.charge(&key, stop)?;
        if !seen.insert(key) {
            budget.charge(&key, stop)?;
            duplicates.insert(key);
        }
        budget.charge(&(&attribute.qualified_name, key), stop)?;
        expanded.push((name, attribute));
    }
    let mut output = Vec::new();
    for (name, attribute) in expanded {
        budget.visit(1, stop)?;
        let duplicate = duplicates.contains(&(name.namespace, name.local));
        let known = name.namespace == Some(names::XMLNS)
            || (name.namespace == Some(names::XML)
                && name.local == "lang"
                && kind == Kind::Documentation)
            || (name.namespace.is_none() && allowed_attribute(kind, name.local));
        let mut interpretation = if duplicate {
            *state = State::Invalid;
            issue_for(
                issues,
                XmlSchemaIssueKind::AmbiguousAttribute,
                id,
                document,
                record,
                Some(attribute),
                budget,
                stop,
            )?;
            XmlSchemaAttributeValue::Invalid
        } else if name.namespace.is_none() {
            interpret(names, name.local, attribute.value(), budget, stop)?
        } else {
            XmlSchemaAttributeValue::Text
        };
        if !known && !duplicate {
            if *state != State::Invalid {
                *state = State::Unsupported;
            }
            issue_for(
                issues,
                XmlSchemaIssueKind::UnsupportedAttribute,
                id,
                document,
                record,
                Some(attribute),
                budget,
                stop,
            )?;
            // Exact QName-valued observations remain available in unsupported context.
            if reference_kind(kind, name.local).is_none() || name.namespace.is_some() {
                interpretation = XmlSchemaAttributeValue::Unsupported;
            }
        }
        match &interpretation {
            XmlSchemaAttributeValue::Invalid => {
                *state = State::Invalid;
                if !duplicate {
                    issue_for(
                        issues,
                        XmlSchemaIssueKind::InvalidAttribute,
                        id,
                        document,
                        record,
                        Some(attribute),
                        budget,
                        stop,
                    )?;
                }
            }
            XmlSchemaAttributeValue::QName(qname) => {
                add_qname(
                    qname, kind, name.local, attribute, id, document, record, known, state,
                    references, issues, budget, stop,
                )?;
            }
            XmlSchemaAttributeValue::QNameList(qnames) => {
                for qname in qnames {
                    budget.visit(1, stop)?;
                    add_qname(
                        qname, kind, name.local, attribute, id, document, record, known, state,
                        references, issues, budget, stop,
                    )?;
                }
            }
            _ => {}
        }
        let name = name.own(budget, stop)?;
        budget.charge(
            &(
                &attribute.qualified_name,
                &name,
                attribute.value(),
                &attribute.span,
                &attribute.value_span,
                attribute.decoded_value_digest,
                &interpretation,
            ),
            stop,
        )?;
        output.push(XmlSchemaAttribute {
            qualified_name: attribute.qualified_name.clone(),
            name,
            value: attribute.value().to_owned(),
            span: attribute.span.clone(),
            value_span: attribute.value_span.clone(),
            decoded_value_digest: attribute.decoded_value_digest,
            interpretation,
        });
    }
    Ok(output)
}

fn form(value: &str) -> Option<XmlSchemaForm> {
    match value.trim_matches(names::xml_space) {
        "qualified" => Some(XmlSchemaForm::Qualified),
        "unqualified" => Some(XmlSchemaForm::Unqualified),
        _ => None,
    }
}

fn interpret<'a>(
    names: &Names<'a>,
    attribute: &str,
    value: &'a str,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<XmlSchemaAttributeValue> {
    use XmlSchemaAttributeValue as Value;
    budget.visit(value.len().max(1), stop)?;
    let token = value.trim_matches(names::xml_space);
    Ok(match attribute {
        "type" | "base" | "ref" | "substitutionGroup" | "itemType" => {
            Value::QName(names.qname(value, budget, stop)?)
        }
        "memberTypes" => {
            let mut values = Vec::new();
            for token in value
                .split(names::xml_space)
                .filter(|part| !part.is_empty())
            {
                budget.visit(1, stop)?;
                let qname = names.qname(token, budget, stop)?;
                budget.charge(&qname, stop)?;
                values.push(qname);
            }
            if values.is_empty() {
                Value::Invalid
            } else {
                Value::QNameList(values)
            }
        }
        "abstract" | "nillable" | "mixed" | "virtual" | "intrinsic" => match token {
            "true" | "1" => Value::Boolean(true),
            "false" | "0" => Value::Boolean(false),
            _ => Value::Invalid,
        },
        "form" | "elementFormDefault" | "attributeFormDefault" => {
            form(token).map_or(Value::Invalid, Value::Form)
        }
        "use" => match token {
            "optional" => Value::Use(XmlSchemaAttributeUse::Optional),
            "required" => Value::Use(XmlSchemaAttributeUse::Required),
            "prohibited" => Value::Use(XmlSchemaAttributeUse::Prohibited),
            _ => Value::Invalid,
        },
        "minOccurs" | "maxOccurs" => {
            if attribute == "maxOccurs" && token == "unbounded" {
                Value::Occurs(XmlSchemaOccurs::Unbounded)
            } else if !token.is_empty() {
                let digits = token
                    .strip_prefix('+')
                    .or_else(|| token.strip_prefix('-'))
                    .unwrap_or(token);
                if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                    Value::Invalid
                } else {
                    let value = digits.parse::<u64>().map_err(|_| exhausted())?;
                    if token.starts_with('-') && value != 0 {
                        Value::Invalid
                    } else {
                        Value::Occurs(XmlSchemaOccurs::Count(value))
                    }
                }
            } else {
                Value::Invalid
            }
        }
        "block" | "final" | "blockDefault" | "finalDefault" => {
            let mut values = Vec::new();
            let mut seen = BTreeSet::new();
            let mut valid = true;
            for token in value
                .split(names::xml_space)
                .filter(|part| !part.is_empty())
            {
                budget.visit(1, stop)?;
                let allowed = match attribute {
                    "block" | "blockDefault" => {
                        matches!(token, "#all" | "extension" | "restriction" | "substitution")
                    }
                    _ => matches!(
                        token,
                        "#all" | "extension" | "restriction" | "list" | "union"
                    ),
                };
                budget.charge(&token, stop)?;
                valid &= allowed && seen.insert(token);
                values.push(token.to_owned());
            }
            valid &= values.len() <= 1 || !seen.contains("#all");
            if valid {
                Value::Tokens(values)
            } else {
                Value::Invalid
            }
        }
        _ => Value::Text,
    })
}

#[allow(clippy::too_many_arguments)]
fn add_qname(
    qname: &XmlSchemaQName,
    kind: Kind,
    attribute_name: &str,
    attribute: &XmlAttributeRecord,
    id: ContentDigest<CanonicalResult>,
    document: &str,
    record: &XmlElementRecord,
    known: bool,
    state: &mut State,
    references: &mut Vec<XmlSchemaReference>,
    issues: &mut Vec<XmlSchemaIssue>,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    if qname.state != XmlSchemaQNameState::Expanded {
        *state = State::Invalid;
        let issue = if qname.state == XmlSchemaQNameState::UnboundPrefix {
            XmlSchemaIssueKind::UnboundPrefix
        } else {
            XmlSchemaIssueKind::InvalidQName
        };
        issue_for(
            issues,
            issue,
            id,
            document,
            record,
            Some(attribute),
            budget,
            stop,
        )?;
    }
    let Some(kind) = reference_kind(kind, attribute_name) else {
        return Ok(());
    };
    let reference_state = if qname.state != XmlSchemaQNameState::Expanded {
        XmlSchemaReferenceState::InvalidQName
    } else if !known || *state != State::Observed {
        XmlSchemaReferenceState::UnsupportedContext
    } else {
        XmlSchemaReferenceState::Missing
    };
    budget.rows(1, stop)?;
    budget.charge(
        &(
            id,
            kind,
            attribute_name,
            &attribute.value_span,
            attribute.decoded_value_digest,
            qname,
            reference_state,
        ),
        stop,
    )?;
    references.push(XmlSchemaReference {
        source: id,
        kind,
        attribute: attribute_name.to_owned(),
        value_span: attribute.value_span.clone(),
        decoded_value_digest: attribute.decoded_value_digest,
        target: qname.clone(),
        state: reference_state,
        candidates: Vec::new(),
    });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn issue_for(
    output: &mut Vec<XmlSchemaIssue>,
    kind: XmlSchemaIssueKind,
    component: ContentDigest<CanonicalResult>,
    document: &str,
    record: &XmlElementRecord,
    attribute: Option<&XmlAttributeRecord>,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    let span = attribute.map_or(&record.span, |attribute| &attribute.value_span);
    push_issue(
        output,
        kind,
        component,
        document,
        &record.occurrence_id,
        span,
        attribute.map(|attribute| attribute.qualified_name.as_str()),
        budget,
        stop,
    )
}

#[allow(clippy::too_many_arguments)]
fn push_issue(
    output: &mut Vec<XmlSchemaIssue>,
    kind: XmlSchemaIssueKind,
    component: ContentDigest<CanonicalResult>,
    document: &str,
    occurrence: &str,
    span: &XmlSourceSpan,
    attribute: Option<&str>,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.rows(1, stop)?;
    budget.charge(
        &(kind, component, document, occurrence, span, attribute),
        stop,
    )?;
    output.push(XmlSchemaIssue {
        kind,
        component,
        document: document.to_owned(),
        occurrence: occurrence.to_owned(),
        span: span.clone(),
        attribute: attribute.map(str::to_owned),
    });
    Ok(())
}

fn validate_declarations(
    output: &mut NormalizedSchema,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    let mut anonymous = BTreeMap::new();
    for component in &output.components {
        budget.visit(1, stop)?;
        if matches!(
            component.kind,
            Kind::AnonymousSimpleType | Kind::AnonymousComplexType
        ) && let Some(parent) = component.parent
        {
            budget.charge(&(parent, 1usize), stop)?;
            let count = anonymous.entry(parent).or_insert(0usize);
            *count = count.checked_add(1).ok_or_else(exhausted)?;
        }
    }
    for component in &mut output.components {
        let probes = component
            .attributes
            .len()
            .checked_mul(10)
            .and_then(|count| count.checked_add(1))
            .ok_or_else(exhausted)?;
        budget.visit(probes, stop)?;
        let has = |name: &str| component.attribute(name).is_some();
        let types = anonymous.get(&component.id).copied().unwrap_or(0);
        let conflicting = types > 1
            || (types > 0 && has("type"))
            || (has("name") && has("ref"))
            || (has("default") && has("fixed"));
        let required = match component.kind {
            Kind::ElementReference
            | Kind::AttributeReference
            | Kind::AttributeGroupReference
            | Kind::GroupReference => has("ref"),
            Kind::Extension => has("base"),
            Kind::Restriction => has("base") || types == 1,
            Kind::Enumeration | Kind::MinInclusive | Kind::MaxInclusive => has("value"),
            _ => true,
        };
        if conflicting || !required {
            component.state = State::Invalid;
            push_issue(
                &mut output.issues,
                XmlSchemaIssueKind::InvalidAttribute,
                component.id,
                &component.document,
                &component.occurrence,
                &component.span,
                None,
                budget,
                stop,
            )?;
        }
        let min = component.attribute("minOccurs").and_then(|attribute| {
            match &attribute.interpretation {
                XmlSchemaAttributeValue::Occurs(XmlSchemaOccurs::Count(value)) => Some(*value),
                _ => None,
            }
        });
        let max = component.attribute("maxOccurs").and_then(|attribute| {
            match &attribute.interpretation {
                XmlSchemaAttributeValue::Occurs(XmlSchemaOccurs::Count(value)) => Some(*value),
                _ => None,
            }
        });
        if min.zip(max).is_some_and(|(min, max)| min > max) {
            component.state = State::Invalid;
            push_issue(
                &mut output.issues,
                XmlSchemaIssueKind::InvalidAttribute,
                component.id,
                &component.document,
                &component.occurrence,
                &component.span,
                None,
                budget,
                stop,
            )?;
        }
    }
    Ok(())
}

fn propagate_context(
    output: &mut NormalizedSchema,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    let mut states = BTreeMap::new();
    for component in &mut output.components {
        budget.visit(1, stop)?;
        if let Some(parent) = component.parent {
            budget.visit(1, stop)?;
            let parent_state = states
                .get(&parent)
                .ok_or_else(|| invalid("schema component has no preceding native parent"))?;
            if *parent_state != State::Observed && component.state == State::Observed {
                component.state = State::Unsupported;
            }
        }
        budget.charge(&(component.id, component.state), stop)?;
        if states.insert(component.id, component.state).is_some() {
            return Err(invalid("schema component identities collide"));
        }
    }
    Ok(())
}

fn symbol_space(kind: Kind) -> Option<SymbolSpace> {
    match kind {
        Kind::GlobalElement => Some(SymbolSpace::Element),
        Kind::NamedSimpleType | Kind::NamedComplexType => Some(SymbolSpace::Type),
        Kind::GlobalAttribute => Some(SymbolSpace::Attribute),
        Kind::NamedGroup => Some(SymbolSpace::Group),
        Kind::NamedAttributeGroup => Some(SymbolSpace::AttributeGroup),
        _ => None,
    }
}

fn reference_space(kind: XmlSchemaReferenceKind) -> SymbolSpace {
    match kind {
        XmlSchemaReferenceKind::Type
        | XmlSchemaReferenceKind::Base
        | XmlSchemaReferenceKind::ItemType
        | XmlSchemaReferenceKind::MemberType => SymbolSpace::Type,
        XmlSchemaReferenceKind::Element | XmlSchemaReferenceKind::SubstitutionGroup => {
            SymbolSpace::Element
        }
        XmlSchemaReferenceKind::Attribute => SymbolSpace::Attribute,
        XmlSchemaReferenceKind::AttributeGroup => SymbolSpace::AttributeGroup,
        XmlSchemaReferenceKind::Group => SymbolSpace::Group,
    }
}

fn resolve_references(
    output: &mut NormalizedSchema,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    let mut symbols: BTreeMap<(SymbolSpace, Option<&str>, &str), Vec<usize>> = BTreeMap::new();
    let mut by_id = BTreeMap::new();
    for (index, component) in output.components.iter().enumerate() {
        budget.visit(1, stop)?;
        budget.charge(&(component.id, index), stop)?;
        if by_id.insert(component.id, index).is_some() {
            return Err(invalid("schema component identities collide"));
        }
        budget.visit(1, stop)?;
        let global = component
            .parent
            .and_then(|parent| by_id.get(&parent))
            .is_some_and(|parent| {
                let parent = &output.components[*parent];
                parent.kind == Kind::Schema
                    && parent.parent.is_none()
                    && parent.document == component.document
            });
        // Invalid nested named observations retain their context issues, but
        // cannot compete with genuine declarations in a global symbol space.
        if !global {
            continue;
        }
        if let (Some(space), Some(name)) = (symbol_space(component.kind), &component.name) {
            let key = (space, name.namespace.as_deref(), name.local_name.as_str());
            budget.charge(&(key, index), stop)?;
            symbols.entry(key).or_default().push(index);
        }
    }
    for candidates in symbols.values() {
        budget.visit(1, stop)?;
        if candidates.len() > 1 {
            for index in candidates {
                budget.visit(1, stop)?;
                let component = &output.components[*index];
                push_issue(
                    &mut output.issues,
                    XmlSchemaIssueKind::ConflictingDeclarations,
                    component.id,
                    &component.document,
                    &component.occurrence,
                    &component.span,
                    None,
                    budget,
                    stop,
                )?;
            }
        }
    }
    for reference in &mut output.references {
        budget.visit(1, stop)?;
        let source = &output.components[*by_id
            .get(&reference.source)
            .ok_or_else(|| invalid("schema reference has no owning component"))?];
        if reference.state == XmlSchemaReferenceState::InvalidQName {
            continue;
        }
        if source.state != State::Observed {
            reference.state = XmlSchemaReferenceState::UnsupportedContext;
        }
        let Some(name) = &reference.target.name else {
            return Err(invalid("expanded schema reference has no QName"));
        };
        let key = (
            reference_space(reference.kind),
            name.namespace.as_deref(),
            name.local_name.as_str(),
        );
        budget.visit(1, stop)?;
        if let Some(candidates) = symbols.get(&key) {
            for index in candidates {
                budget.visit(1, stop)?;
                let id = output.components[*index].id;
                budget.charge(&id, stop)?;
                reference.candidates.push(id);
            }
            if reference.state != XmlSchemaReferenceState::UnsupportedContext {
                reference.state = if candidates.len() > 1 {
                    XmlSchemaReferenceState::Conflict
                } else if output.components[candidates[0]].state != State::Observed {
                    XmlSchemaReferenceState::UnsupportedContext
                } else {
                    XmlSchemaReferenceState::Unique
                };
            }
        } else if reference.state != XmlSchemaReferenceState::UnsupportedContext {
            reference.state = if name.namespace() == Some(names::XSD) {
                XmlSchemaReferenceState::ExternalNamespace
            } else {
                XmlSchemaReferenceState::Missing
            };
            if reference.state == XmlSchemaReferenceState::Missing {
                push_issue(
                    &mut output.issues,
                    XmlSchemaIssueKind::MissingReference,
                    source.id,
                    &source.document,
                    &source.occurrence,
                    &reference.value_span,
                    Some(&reference.attribute),
                    budget,
                    stop,
                )?;
            }
        }
    }
    Ok(())
}

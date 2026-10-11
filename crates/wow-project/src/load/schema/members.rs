//! Permitted source members from ONE sealed catalog and borrowed native index.
//! No parser, QName policy table, class->placement inference or runtime authority.
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};

use super::{
    AdmittedXmlSchema, XmlExpandedName, XmlSchemaAttributeUse as U, XmlSchemaAttributeValue as V,
    XmlSchemaComponent, XmlSchemaComponentKind as K, XmlSchemaComponentState as S, XmlSchemaOccurs,
    XmlSchemaQName, XmlSchemaQNameState, XmlSchemaReferenceKind as R,
    XmlSchemaReferenceState as RS,
    budget::SchemaLedger,
    invalid,
    names::{Names, XMLNS},
};
use crate::{
    ProjectError, ProjectErrorCode, ProjectResult,
    load::{XmlDocumentIndex, XmlElementRecord},
};
use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest};

pub(crate) use super::budget::SchemaLedger as MemberLedger;
pub(crate) use super::budget::{MAX_ROWS as MEMBER_ROWS, MAX_WORK as MEMBER_WORK};
type Id = ContentDigest<CanonicalResult>;

// Same nesting boundary as the existing XML namespace/index owner. Referenced
// models cannot escape it by constructing an unbounded Rust recursion stack.
const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, Copy)]
pub(crate) enum MemberUnavailable {
    UnknownQName,
    Unsupported,
    Ambiguous,
    Cycle,
    Placement,
    Attribute,
    Content,
}
enum Failure {
    Unavailable(MemberUnavailable),
    Fatal(ProjectError),
}
type Walk<T> = Result<T, Failure>;
impl From<ProjectError> for Failure {
    fn from(error: ProjectError) -> Self {
        Self::Fatal(error)
    }
}
fn refuse<T>(why: MemberUnavailable) -> Walk<T> {
    Err(Failure::Unavailable(why))
}
fn namespace<T>(result: ProjectResult<T>) -> Walk<T> {
    result.map_err(|error| {
        if error.code() == ProjectErrorCode::SourceRegistryInvalid {
            Failure::Unavailable(MemberUnavailable::UnknownQName)
        } else {
            Failure::Fatal(error)
        }
    })
}

#[derive(Debug, Clone, Serialize)]
enum Hop {
    Parent {
        child: Id,
        parent: Id,
    },
    Reference {
        source: Id,
        kind: R,
        target: Id,
        native_reference: usize,
    },
}
#[derive(Debug, Serialize)]
pub(crate) struct MemberWitness {
    occurrence: usize,
    qname: XmlSchemaQName,
    declaration: Id,
    type_component: Id,
    parent_declaration: Option<Id>,
    member_component: Id,
    path: Vec<Hop>,
}
impl MemberWitness {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence
    }
    pub(crate) const fn declaration(&self) -> Id {
        self.declaration
    }
    pub(crate) const fn type_component(&self) -> Id {
        self.type_component
    }
    pub(crate) const fn member_component(&self) -> Id {
        self.member_component
    }
}

/// Fields/constructor are private; printed witnesses are not an admission port.
pub(crate) struct PermittedDocument<'a> {
    catalog: &'a AdmittedXmlSchema,
    document: &'a XmlDocumentIndex,
    witnesses: Vec<MemberWitness>,
}
impl PermittedDocument<'_> {
    pub(crate) fn witnesses(&self) -> &[MemberWitness] {
        &self.witnesses
    }
    pub(crate) fn document(&self) -> &XmlDocumentIndex {
        self.document
    }
    pub(crate) fn catalog(&self) -> &AdmittedXmlSchema {
        self.catalog
    }
}
pub(crate) enum Admission<'a> {
    Closed(PermittedDocument<'a>),
    Unavailable(MemberUnavailable),
}

#[derive(Clone, Serialize)]
struct Attribute {
    declaration: Id,
    name: XmlExpandedName,
    usage: U,
    path: Vec<Hop>,
}
#[derive(Clone, Serialize)]
struct Member {
    particle: Id,
    declaration: Id,
    name: XmlExpandedName,
    path: Vec<Hop>,
}
#[derive(Serialize)]
enum ParticleKind {
    Element(Member),
    Sequence(Vec<Particle>),
    Choice(Vec<Particle>),
}
#[derive(Serialize)]
struct Particle {
    component: Id,
    optional: bool,
    kind: ParticleKind,
}
struct Model {
    particle: Option<Particle>,
    attributes: Vec<Attribute>,
}
#[derive(Clone, Serialize)]
struct Matched {
    end: usize,
    members: Vec<(usize, Member)>,
}

/// Index only native IDs, parent coordinates, global element space and references.
/// Namespace resolution of instance names still uses the INSTANCE document.
pub(crate) struct MemberCatalog<'a> {
    catalog: &'a AdmittedXmlSchema,
    ids: BTreeMap<Id, usize>,
    children: BTreeMap<Id, Vec<Id>>,
    elements: BTreeMap<XmlExpandedName, Vec<Id>>,
    references: BTreeMap<(Id, R), Vec<usize>>,
}
impl<'a> MemberCatalog<'a> {
    pub(crate) fn new(
        catalog: &'a AdmittedXmlSchema,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        let mut result = Self {
            catalog,
            ids: BTreeMap::new(),
            children: BTreeMap::new(),
            elements: BTreeMap::new(),
            references: BTreeMap::new(),
        };
        for (position, c) in catalog.components().iter().enumerate() {
            ledger.rows(1, stop)?;
            ledger.charge(&(c.id(), position), stop)?;
            if result.ids.insert(c.id(), position).is_some() {
                return Err(invalid("member catalog has duplicate native component IDs"));
            }
            if let Some(parent) = c.parent() {
                let p = result
                    .get(parent)
                    .map_err(|_| invalid("member catalog parent is missing"))?;
                if p.document() != c.document() {
                    return Err(invalid("member catalog parent crosses documents"));
                }
                ledger.charge(&(parent, c.id()), stop)?;
                result.children.entry(parent).or_default().push(c.id());
                if p.kind() == K::Schema && p.parent().is_none() && c.kind() == K::GlobalElement {
                    let Some(name) = c.name() else { continue };
                    ledger.charge(&(name, c.id()), stop)?;
                    result
                        .elements
                        .entry(name.clone())
                        .or_default()
                        .push(c.id());
                }
            }
        }
        for (position, reference) in catalog.references().iter().enumerate() {
            ledger.rows(1, stop)?;
            ledger.charge(&(reference.source(), reference.kind(), position), stop)?;
            if !result.ids.contains_key(&reference.source()) {
                return Err(invalid("member reference source is missing"));
            }
            result
                .references
                .entry((reference.source(), reference.kind()))
                .or_default()
                .push(position);
        }
        Ok(result)
    }
    fn get(&self, id: Id) -> Walk<&'a XmlSchemaComponent> {
        self.ids
            .get(&id)
            .and_then(|p| self.catalog.components().get(*p))
            .ok_or(Failure::Unavailable(MemberUnavailable::Unsupported))
    }
    fn observed(&self, id: Id) -> Walk<&'a XmlSchemaComponent> {
        let c = self.get(id)?;
        if c.state() != S::Observed {
            return refuse(MemberUnavailable::Unsupported);
        }
        Ok(c)
    }
    fn kids(&self, id: Id) -> &[Id] {
        self.children.get(&id).map_or(&[], Vec::as_slice)
    }
    fn hop(
        path: &mut Vec<Hop>,
        hop: Hop,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<()> {
        ledger.rows(1, stop)?;
        ledger.charge(&hop, stop)?;
        path.push(hop);
        Ok(())
    }
    fn copy_path(
        path: &[Hop],
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<Vec<Hop>> {
        ledger.charge(&path, stop)?;
        Ok(path.to_vec())
    }
    fn link(
        &self,
        source: Id,
        kind: R,
        path: &mut Vec<Hop>,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<Id> {
        ledger.visit(1, stop)?;
        let positions = self
            .references
            .get(&(source, kind))
            .ok_or(Failure::Unavailable(MemberUnavailable::Unsupported))?;
        let [position] = positions.as_slice() else {
            return refuse(MemberUnavailable::Ambiguous);
        };
        let reference = &self.catalog.references()[*position];
        if reference.state() != RS::Unique
            || reference.target().state() != XmlSchemaQNameState::Expanded
        {
            return refuse(MemberUnavailable::Unsupported);
        }
        let [target] = reference.candidates() else {
            return refuse(MemberUnavailable::Ambiguous);
        };
        self.observed(source)?;
        self.observed(*target)?;
        Self::hop(
            path,
            Hop::Reference {
                source,
                kind,
                target: *target,
                native_reference: *position,
            },
            ledger,
            stop,
        )?;
        Ok(*target)
    }
    fn global(&self, name: &XmlExpandedName) -> Walk<Id> {
        let candidates = self
            .elements
            .get(name)
            .ok_or(Failure::Unavailable(MemberUnavailable::UnknownQName))?;
        let [id] = candidates.as_slice() else {
            return refuse(MemberUnavailable::Ambiguous);
        };
        self.observed(*id)?;
        Ok(*id)
    }
    fn flags(&self, c: &XmlSchemaComponent) -> Walk<()> {
        // These controls would require value/default/nil/substitution evaluation.
        // A structural witness never silently ignores them.
        for name in ["default", "fixed", "block", "final"] {
            if c.attribute(name).is_some() {
                return refuse(MemberUnavailable::Unsupported);
            }
        }
        for name in ["abstract", "nillable"] {
            if let Some(a) = c.attribute(name)
                && !matches!(a.interpretation(), V::Boolean(false))
            {
                return refuse(MemberUnavailable::Unsupported);
            }
        }
        Ok(())
    }
    fn schema_controls(
        &self,
        c: &XmlSchemaComponent,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<()> {
        // The owning schema's defaults/dependency declarations can affect
        // these models; unsupported acquisition/redefinition is never ignored.
        let mut owner = c;
        let mut owners = BTreeSet::new();
        while let Some(parent) = owner.parent() {
            ledger.visit(1, stop)?;
            ledger.charge(&parent, stop)?;
            if !owners.insert(parent) {
                return refuse(MemberUnavailable::Cycle);
            }
            owner = self.observed(parent)?;
        }
        if owner.kind() != K::Schema
            || owner.attribute("blockDefault").is_some()
            || owner.attribute("finalDefault").is_some()
        {
            return refuse(MemberUnavailable::Unsupported);
        }
        for child in self.kids(owner.id()) {
            ledger.visit(1, stop)?;
            if matches!(
                self.get(*child)?.kind(),
                K::Include | K::Import | K::Redefine
            ) {
                return refuse(MemberUnavailable::Unsupported);
            }
        }
        Ok(())
    }
    fn element_type(
        &self,
        element: Id,
        path: &mut Vec<Hop>,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<Id> {
        ledger.visit(1, stop)?;
        let c = self.observed(element)?;
        if !matches!(c.kind(), K::GlobalElement | K::LocalElement) {
            return refuse(MemberUnavailable::Unsupported);
        }
        self.flags(c)?;
        self.schema_controls(c, ledger, stop)?;
        let mut substitution = element;
        let mut ancestors = BTreeSet::new();
        while self
            .references
            .contains_key(&(substitution, R::SubstitutionGroup))
        {
            ledger.visit(1, stop)?;
            ledger.charge(&substitution, stop)?;
            if !ancestors.insert(substitution) {
                return refuse(MemberUnavailable::Cycle);
            }
            substitution = self.link(substitution, R::SubstitutionGroup, path, ledger, stop)?;
            if self.observed(substitution)?.kind() != K::GlobalElement {
                return refuse(MemberUnavailable::Unsupported);
            }
        }
        let mut anonymous = None;
        for child in self.kids(element) {
            ledger.visit(1, stop)?;
            let child_c = self.observed(*child)?;
            match child_c.kind() {
                K::AnonymousComplexType => {
                    if anonymous.replace(*child).is_some() {
                        return refuse(MemberUnavailable::Ambiguous);
                    }
                }
                K::Annotation => self.annotation(*child, 0, ledger, stop)?,
                _ => return refuse(MemberUnavailable::Unsupported),
            }
        }
        let has_type = self.references.contains_key(&(element, R::Type));
        match (has_type, anonymous) {
            (true, None) => {
                let target = self.link(element, R::Type, path, ledger, stop)?;
                if self.observed(target)?.kind() != K::NamedComplexType {
                    return refuse(MemberUnavailable::Unsupported);
                }
                Ok(target)
            }
            (false, Some(target)) => {
                Self::hop(
                    path,
                    Hop::Parent {
                        child: target,
                        parent: element,
                    },
                    ledger,
                    stop,
                )?;
                Ok(target)
            }
            _ => refuse(MemberUnavailable::Unsupported),
        }
    }
    fn annotation(
        &self,
        id: Id,
        depth: usize,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<()> {
        ledger.visit(1, stop)?;
        if depth >= MAX_DEPTH {
            return refuse(MemberUnavailable::Unsupported);
        }
        for child in self.kids(id) {
            let c = self.observed(*child)?;
            if !matches!(c.kind(), K::Documentation | K::AppInfo) {
                return refuse(MemberUnavailable::Unsupported);
            }
            self.annotation(*child, depth + 1, ledger, stop)?;
        }
        Ok(())
    }
    fn occurs(&self, c: &XmlSchemaComponent) -> Walk<bool> {
        // Small exact domain: optional/single occurrence Sequence/Choice/member.
        // Repetition/All are explicitly unsupported, never approximated.
        let read = |name: &str| -> Walk<u64> {
            match c.attribute(name).map(|a| a.interpretation()) {
                None => Ok(1),
                Some(V::Occurs(XmlSchemaOccurs::Count(v))) => Ok(*v),
                _ => refuse(MemberUnavailable::Unsupported),
            }
        };
        let min = read("minOccurs")?;
        let max = read("maxOccurs")?;
        if max != 1 || min > 1 {
            return refuse(MemberUnavailable::Unsupported);
        }
        Ok(min == 0)
    }
    fn model(
        &self,
        id: Id,
        path: &[Hop],
        active: &mut BTreeSet<Id>,
        depth: usize,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<Model> {
        ledger.visit(1, stop)?;
        if depth >= MAX_DEPTH {
            return refuse(MemberUnavailable::Unsupported);
        }
        ledger.charge(&id, stop)?;
        if !active.insert(id) {
            return refuse(MemberUnavailable::Cycle);
        }
        let c = self.observed(id)?;
        if !matches!(
            c.kind(),
            K::NamedComplexType
                | K::AnonymousComplexType
                | K::ComplexContent
                | K::Extension
                | K::NamedGroup
                | K::NamedAttributeGroup
        ) {
            return refuse(MemberUnavailable::Unsupported);
        }
        self.flags(c)?;
        self.schema_controls(c, ledger, stop)?;
        if c.attribute("mixed")
            .is_some_and(|a| !matches!(a.interpretation(), V::Boolean(false)))
        {
            return refuse(MemberUnavailable::Content);
        }
        let mut result = Model {
            particle: None,
            attributes: Vec::new(),
        };
        let mut local_particle_seen = false;
        let mut content_seen = false;
        let mut attributes_started = false;
        if c.kind() == K::Extension {
            let mut base_path = Self::copy_path(path, ledger, stop)?;
            let base = self.link(id, R::Base, &mut base_path, ledger, stop)?;
            if !matches!(self.observed(base)?.kind(), K::NamedComplexType) {
                return refuse(MemberUnavailable::Unsupported);
            }
            result = self.model(base, &base_path, active, depth + 1, ledger, stop)?;
        }
        for child in self.kids(id) {
            ledger.visit(1, stop)?;
            let cc = self.observed(*child)?;
            let mut child_path = Self::copy_path(path, ledger, stop)?;
            Self::hop(
                &mut child_path,
                Hop::Parent {
                    child: *child,
                    parent: id,
                },
                ledger,
                stop,
            )?;
            match cc.kind() {
                K::Sequence | K::Choice | K::GroupReference => {
                    if local_particle_seen || content_seen || attributes_started {
                        return refuse(MemberUnavailable::Ambiguous);
                    }
                    local_particle_seen = true;
                    let particle =
                        self.particle(*child, &child_path, active, depth + 1, ledger, stop)?;
                    result.particle = Some(match result.particle.take() {
                        None => particle,
                        Some(base) if c.kind() == K::Extension => {
                            ledger.rows(1, stop)?;
                            ledger.charge(&(id, &base, &particle), stop)?;
                            Particle {
                                component: id,
                                optional: false,
                                kind: ParticleKind::Sequence(vec![base, particle]),
                            }
                        }
                        Some(_) => return refuse(MemberUnavailable::Ambiguous),
                    });
                }
                K::ComplexContent | K::Extension => {
                    if content_seen || local_particle_seen || attributes_started {
                        return refuse(MemberUnavailable::Ambiguous);
                    }
                    content_seen = true;
                    let nested =
                        self.model(*child, &child_path, active, depth + 1, ledger, stop)?;
                    if result.particle.is_some() || !result.attributes.is_empty() {
                        return refuse(MemberUnavailable::Ambiguous);
                    }
                    result = nested;
                }
                K::LocalAttribute | K::AttributeReference => {
                    if content_seen {
                        return refuse(MemberUnavailable::Unsupported);
                    }
                    attributes_started = true;
                    result
                        .attributes
                        .push(self.attribute(*child, child_path, ledger, stop)?);
                }
                K::AttributeGroupReference => {
                    if content_seen {
                        return refuse(MemberUnavailable::Unsupported);
                    }
                    attributes_started = true;
                    let target =
                        self.link(*child, R::AttributeGroup, &mut child_path, ledger, stop)?;
                    if self.observed(target)?.kind() != K::NamedAttributeGroup {
                        return refuse(MemberUnavailable::Unsupported);
                    }
                    let nested =
                        self.model(target, &child_path, active, depth + 1, ledger, stop)?;
                    if nested.particle.is_some() {
                        return refuse(MemberUnavailable::Unsupported);
                    }
                    ledger.charge(&nested.attributes, stop)?;
                    result.attributes.extend(nested.attributes);
                }
                K::Annotation => self.annotation(*child, depth + 1, ledger, stop)?,
                _ => return refuse(MemberUnavailable::Unsupported),
            }
        }
        if (c.kind() == K::ComplexContent && !content_seen)
            || (c.kind() == K::NamedGroup && !local_particle_seen)
        {
            return refuse(MemberUnavailable::Unsupported);
        }
        let mut names = BTreeSet::new();
        for a in &result.attributes {
            ledger.visit(1, stop)?;
            ledger.charge(&a.name, stop)?;
            if !names.insert(&a.name) {
                return refuse(MemberUnavailable::Ambiguous);
            }
        }
        active.remove(&id);
        Ok(result)
    }
    fn attribute(
        &self,
        id: Id,
        mut path: Vec<Hop>,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<Attribute> {
        ledger.visit(1, stop)?;
        let use_component = self.observed(id)?;
        self.flags(use_component)?;
        let declaration = if use_component.kind() == K::AttributeReference {
            let target = self.link(id, R::Attribute, &mut path, ledger, stop)?;
            if self.observed(target)?.kind() != K::GlobalAttribute {
                return refuse(MemberUnavailable::Unsupported);
            }
            target
        } else {
            id
        };
        let c = self.observed(declaration)?;
        self.flags(c)?;
        // Presence permission only. Typed value/restriction semantics are not
        // silently treated as valid: selected typed attributes require an owner.
        if !self.kids(declaration).is_empty()
            || self.references.contains_key(&(declaration, R::Type))
        {
            return refuse(MemberUnavailable::Unsupported);
        }
        let usage = match use_component.attribute("use").map(|a| a.interpretation()) {
            None => U::Optional,
            Some(V::Use(u)) => *u,
            _ => return refuse(MemberUnavailable::Unsupported),
        };
        let name = c
            .name()
            .ok_or(Failure::Unavailable(MemberUnavailable::Attribute))?;
        ledger.rows(1, stop)?;
        ledger.charge(&(declaration, name, usage, &path), stop)?;
        Ok(Attribute {
            declaration,
            name: name.clone(),
            usage,
            path,
        })
    }
    fn particle(
        &self,
        id: Id,
        path: &[Hop],
        active: &mut BTreeSet<Id>,
        depth: usize,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<Particle> {
        ledger.visit(1, stop)?;
        if depth >= MAX_DEPTH {
            return refuse(MemberUnavailable::Unsupported);
        }
        ledger.charge(&id, stop)?;
        if !active.insert(id) {
            return refuse(MemberUnavailable::Cycle);
        }
        let c = self.observed(id)?;
        let optional = self.occurs(c)?;
        let kind = match c.kind() {
            K::LocalElement | K::ElementReference => {
                let mut member_path = Self::copy_path(path, ledger, stop)?;
                let declaration = if c.kind() == K::ElementReference {
                    self.link(id, R::Element, &mut member_path, ledger, stop)?
                } else {
                    id
                };
                let element = self.observed(declaration)?;
                self.flags(element)?;
                let name = element
                    .name()
                    .ok_or(Failure::Unavailable(MemberUnavailable::UnknownQName))?;
                // A referenced head with substitutions is an unsupported model,
                // not a first-match permit for a different occurrence QName.
                for reference in self.catalog.references() {
                    ledger.visit(1, stop)?;
                    if reference.kind() == R::SubstitutionGroup
                        && reference.target().name() == Some(name)
                    {
                        return refuse(MemberUnavailable::Unsupported);
                    }
                }
                // Resolve its type now, even for an unselected Choice branch.
                // Every relevant alternative must have a determinate native type.
                let mut type_path = Self::copy_path(&member_path, ledger, stop)?;
                let ty = self.element_type(declaration, &mut type_path, ledger, stop)?;
                let _ = self.model(ty, &type_path, active, depth + 1, ledger, stop)?;
                ledger.rows(1, stop)?;
                ledger.charge(&(id, declaration, name, &member_path), stop)?;
                ParticleKind::Element(Member {
                    particle: id,
                    declaration,
                    name: name.clone(),
                    path: member_path,
                })
            }
            K::Sequence | K::Choice => {
                let mut children = Vec::new();
                for child in self.kids(id) {
                    let cc = self.observed(*child)?;
                    if cc.kind() == K::Annotation {
                        self.annotation(*child, depth + 1, ledger, stop)?;
                        continue;
                    }
                    let mut child_path = Self::copy_path(path, ledger, stop)?;
                    Self::hop(
                        &mut child_path,
                        Hop::Parent {
                            child: *child,
                            parent: id,
                        },
                        ledger,
                        stop,
                    )?;
                    let p = self.particle(*child, &child_path, active, depth + 1, ledger, stop)?;
                    ledger.charge(&p, stop)?;
                    children.push(p);
                }
                if c.kind() == K::Sequence {
                    ParticleKind::Sequence(children)
                } else {
                    ParticleKind::Choice(children)
                }
            }
            K::GroupReference => {
                let mut group_path = Self::copy_path(path, ledger, stop)?;
                let target = self.link(id, R::Group, &mut group_path, ledger, stop)?;
                if self.observed(target)?.kind() != K::NamedGroup {
                    return refuse(MemberUnavailable::Unsupported);
                }
                let model = self.model(target, &group_path, active, depth + 1, ledger, stop)?;
                if !model.attributes.is_empty() {
                    return refuse(MemberUnavailable::Unsupported);
                }
                ParticleKind::Sequence(model.particle.into_iter().collect())
            }
            _ => return refuse(MemberUnavailable::Unsupported),
        };
        active.remove(&id);
        ledger.rows(1, stop)?;
        ledger.charge(&(id, optional, &kind), stop)?;
        Ok(Particle {
            component: id,
            optional,
            kind,
        })
    }

    fn push_match(
        output: &mut Vec<Matched>,
        value: Matched,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<()> {
        ledger.rows(1, stop)?;
        ledger.charge(&value, stop)?;
        output.push(value);
        Ok(())
    }
    fn matches(
        p: &Particle,
        names: &[&XmlExpandedName],
        start: usize,
        depth: usize,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<Vec<Matched>> {
        ledger.visit(1, stop)?;
        if depth >= MAX_DEPTH {
            return refuse(MemberUnavailable::Unsupported);
        }
        let mut output = Vec::new();
        match &p.kind {
            ParticleKind::Element(member) => {
                if names.get(start).is_some_and(|name| **name == member.name) {
                    ledger.charge(member, stop)?;
                    Self::push_match(
                        &mut output,
                        Matched {
                            end: start + 1,
                            members: vec![(start, member.clone())],
                        },
                        ledger,
                        stop,
                    )?;
                }
            }
            ParticleKind::Choice(children) => {
                for child in children {
                    for value in Self::matches(child, names, start, depth + 1, ledger, stop)? {
                        Self::push_match(&mut output, value, ledger, stop)?;
                    }
                }
            }
            ParticleKind::Sequence(children) => {
                let mut states = Vec::new();
                Self::push_match(
                    &mut states,
                    Matched {
                        end: start,
                        members: Vec::new(),
                    },
                    ledger,
                    stop,
                )?;
                for child in children {
                    let mut next = Vec::new();
                    for state in states {
                        for tail in Self::matches(child, names, state.end, depth + 1, ledger, stop)?
                        {
                            ledger.charge(&state.members, stop)?;
                            let mut members = state.members.clone();
                            ledger.charge(&tail.members, stop)?;
                            members.extend(tail.members);
                            Self::push_match(
                                &mut next,
                                Matched {
                                    end: tail.end,
                                    members,
                                },
                                ledger,
                                stop,
                            )?;
                        }
                    }
                    states = next;
                }
                for state in states {
                    Self::push_match(&mut output, state, ledger, stop)?;
                }
            }
        }
        if p.optional {
            Self::push_match(
                &mut output,
                Matched {
                    end: start,
                    members: Vec::new(),
                },
                ledger,
                stop,
            )?;
        }
        Ok(output)
    }

    fn document(
        &self,
        document: &'a XmlDocumentIndex,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<PermittedDocument<'a>> {
        ledger.visit(1, stop)?;
        if !document.outside_script_content_clear() {
            return refuse(MemberUnavailable::Content);
        }
        let mut namespaces = Names::new(document.document());
        let mut qnames = Vec::new();
        let mut children = BTreeMap::<usize, Vec<usize>>::new();
        let mut occurrences = BTreeMap::<&str, usize>::new();
        for (i, record) in document.elements().iter().enumerate() {
            ledger.rows(1, stop)?;
            if !record.issues.is_empty() {
                return refuse(MemberUnavailable::Content);
            }
            namespace(namespaces.enter(record, ledger, stop))?;
            let qname = namespace(namespaces.qname(&record.qualified_name, ledger, stop))?;
            if qname.name().is_none() {
                return refuse(MemberUnavailable::UnknownQName);
            }
            ledger.charge(&qname, stop)?;
            qnames.push(qname);
            ledger.charge(&(&record.occurrence_id, i), stop)?;
            if occurrences.insert(&record.occurrence_id, i).is_some() {
                return refuse(MemberUnavailable::Ambiguous);
            }
            if let Some(parent) = record.parent_occurrence_id.as_deref() {
                let parent = *occurrences
                    .get(parent)
                    .ok_or(Failure::Unavailable(MemberUnavailable::Placement))?;
                if parent == i {
                    return refuse(MemberUnavailable::Cycle);
                }
                ledger.charge(&(parent, i), stop)?;
                children.entry(parent).or_default().push(i);
            } else if i != 0 {
                return refuse(MemberUnavailable::Placement);
            }
        }
        if qnames.is_empty() {
            return refuse(MemberUnavailable::Placement);
        }
        let root = self.global(
            qnames[0]
                .name()
                .ok_or(Failure::Unavailable(MemberUnavailable::UnknownQName))?,
        )?;
        let mut assigned = BTreeMap::new();
        ledger.charge(&(0usize, root, qnames[0].name()), stop)?;
        assigned.insert(
            0usize,
            Member {
                particle: root,
                declaration: root,
                name: qnames[0]
                    .name()
                    .ok_or(Failure::Unavailable(MemberUnavailable::UnknownQName))?
                    .clone(),
                path: Vec::new(),
            },
        );
        let mut witnesses = Vec::new();
        let mut namespace_attributes = Names::new(document.document());
        for (i, record) in document.elements().iter().enumerate() {
            ledger.rows(1, stop)?;
            namespace(namespace_attributes.enter(record, ledger, stop))?;
            let member = assigned
                .remove(&i)
                .ok_or(Failure::Unavailable(MemberUnavailable::Placement))?;
            let mut path = Self::copy_path(&member.path, ledger, stop)?;
            let ty = self.element_type(member.declaration, &mut path, ledger, stop)?;
            let model = self.model(ty, &path, &mut BTreeSet::new(), 0, ledger, stop)?;
            self.instance_attributes(
                record,
                &model.attributes,
                &namespace_attributes,
                ledger,
                stop,
            )?;
            let child_rows = children.get(&i).map_or(&[][..], Vec::as_slice);
            let mut names = Vec::new();
            for child in child_rows {
                ledger.visit(1, stop)?;
                let name = qnames[*child]
                    .name()
                    .ok_or(Failure::Unavailable(MemberUnavailable::UnknownQName))?;
                ledger.charge(&(*child, name), stop)?;
                names.push(name);
            }
            let matches = match &model.particle {
                Some(p) => Self::matches(p, &names, 0, 0, ledger, stop)?,
                None => {
                    let mut output = Vec::new();
                    Self::push_match(
                        &mut output,
                        Matched {
                            end: 0,
                            members: Vec::new(),
                        },
                        ledger,
                        stop,
                    )?;
                    output
                }
            };
            let mut complete = None;
            for matching in matches {
                ledger.visit(1, stop)?;
                if matching.end == child_rows.len() && complete.replace(matching).is_some() {
                    return refuse(MemberUnavailable::Ambiguous);
                }
            }
            let complete = complete.ok_or(Failure::Unavailable(MemberUnavailable::Placement))?;
            for (local, child_member) in complete.members {
                let child = child_rows[local];
                ledger.charge(&(child, &child_member), stop)?;
                if assigned.insert(child, child_member).is_some() {
                    return refuse(MemberUnavailable::Ambiguous);
                }
            }
            let parent_declaration = record
                .parent_occurrence_id
                .as_deref()
                .and_then(|parent| occurrences.get(parent))
                .and_then(|parent| witnesses.get(*parent))
                .map(|w: &MemberWitness| w.declaration);
            ledger.charge(
                &(
                    i,
                    &qnames[i],
                    member.declaration,
                    ty,
                    parent_declaration,
                    member.particle,
                    &path,
                ),
                stop,
            )?;
            witnesses.push(MemberWitness {
                occurrence: i,
                qname: qnames[i].clone(),
                declaration: member.declaration,
                type_component: ty,
                parent_declaration,
                member_component: member.particle,
                path,
            });
        }
        if !assigned.is_empty() || witnesses.len() != document.elements().len() {
            return refuse(MemberUnavailable::Placement);
        }
        Ok(PermittedDocument {
            catalog: self.catalog,
            document,
            witnesses,
        })
    }
    fn instance_attributes<'d>(
        &self,
        record: &'d XmlElementRecord,
        allowed: &[Attribute],
        names: &Names<'d>,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> Walk<()> {
        let mut seen = BTreeSet::new();
        for attribute in &record.attributes {
            ledger.visit(1, stop)?;
            let name = namespace(names.attribute(attribute, ledger, stop))?;
            if name.namespace == Some(XMLNS) {
                continue;
            }
            let name = name.own(ledger, stop)?;
            ledger.charge(&name, stop)?;
            if !seen.insert(name.clone()) {
                return refuse(MemberUnavailable::Attribute);
            }
            let mut selected = None;
            for a in allowed {
                ledger.visit(1, stop)?;
                if a.name == name && selected.replace(a).is_some() {
                    return refuse(MemberUnavailable::Ambiguous);
                }
            }
            let a = selected.ok_or(Failure::Unavailable(MemberUnavailable::Attribute))?;
            if a.usage == U::Prohibited {
                return refuse(MemberUnavailable::Attribute);
            }
            ledger.charge(
                &(
                    a.declaration,
                    &a.path,
                    &attribute.value_span,
                    attribute.decoded_value_digest,
                ),
                stop,
            )?;
        }
        for a in allowed {
            ledger.visit(1, stop)?;
            if a.usage == U::Required && !seen.contains(&a.name) {
                return refuse(MemberUnavailable::Attribute);
            }
        }
        Ok(())
    }
    pub(crate) fn admit(
        &self,
        document: &'a XmlDocumentIndex,
        ledger: &mut impl SchemaLedger,
        stop: &AtomicBool,
    ) -> ProjectResult<Admission<'a>> {
        match self.document(document, ledger, stop) {
            Ok(proof) => Ok(Admission::Closed(proof)),
            Err(Failure::Unavailable(why)) => Ok(Admission::Unavailable(why)),
            Err(Failure::Fatal(error)) => Err(error),
        }
    }
}

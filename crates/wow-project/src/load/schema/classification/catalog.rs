//! Bounded catalog lookup and typed edge walks; no source parser or name policy.
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};

use crate::{
    ProjectResult,
    load::schema::{
        AdmittedXmlSchema, SchemaBudget, XmlExpandedName, XmlSchemaComponent,
        XmlSchemaComponentKind as Kind, XmlSchemaComponentState as State, XmlSchemaQNameState,
        XmlSchemaReference, XmlSchemaReferenceKind as RefKind, XmlSchemaReferenceState as RefState,
        exhausted, invalid,
    },
};

use super::{
    XmlClassificationUnknown as Unknown, XmlClassificationWitness as Witness,
    XmlSchemaComponentId as Id,
};

pub(super) struct CatalogIndex {
    by_id: BTreeMap<Id, usize>,
    children: BTreeMap<Id, Vec<Id>>,
    elements: BTreeMap<XmlExpandedName, Vec<Id>>,
    types: BTreeMap<XmlExpandedName, Vec<Id>>,
    references: BTreeMap<(Id, RefKind), Vec<usize>>,
}

pub(super) enum Link<'a> {
    Absent,
    Unique(&'a XmlSchemaReference, Id),
    Blocked(Unknown),
}

pub(super) enum PathState {
    Reached,
    Absent,
    Blocked(Unknown),
}

pub(super) struct Path {
    pub(super) state: PathState,
    pub(super) witnesses: Vec<Witness>,
}

pub(super) fn push(
    output: &mut Vec<Witness>,
    witness: Witness,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.rows(1, stop)?;
    budget.charge(&witness, stop)?;
    output.push(witness);
    Ok(())
}

impl CatalogIndex {
    pub(super) fn build(
        catalog: &AdmittedXmlSchema,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        let mut index = Self {
            by_id: BTreeMap::new(),
            children: BTreeMap::new(),
            elements: BTreeMap::new(),
            types: BTreeMap::new(),
            references: BTreeMap::new(),
        };
        for (position, component) in catalog.components().iter().enumerate() {
            budget.visit(1, stop)?;
            budget.charge(&(component.id(), position), stop)?;
            if index.by_id.insert(component.id(), position).is_some() {
                return Err(invalid(
                    "classification catalog component identities collide",
                ));
            }
            if let Some(parent) = component.parent() {
                let enclosing = index.component(catalog, parent)?;
                if enclosing.document() != component.document() {
                    return Err(invalid("classification catalog parent crosses a document"));
                }
                budget.charge(&(parent, component.id()), stop)?;
                index
                    .children
                    .entry(parent)
                    .or_default()
                    .push(component.id());
                // Match the native catalog resolver's global symbol boundary:
                // root Schema parent, SAME document; nested names never compete.
                if enclosing.kind() != Kind::Schema || enclosing.parent().is_some() {
                    continue;
                }
                let Some(name) = component.name() else {
                    continue;
                };
                let symbols = match component.kind() {
                    Kind::GlobalElement => &mut index.elements,
                    Kind::NamedSimpleType | Kind::NamedComplexType => &mut index.types,
                    _ => continue,
                };
                budget.charge(&(name, component.id()), stop)?;
                symbols
                    .entry(name.clone())
                    .or_default()
                    .push(component.id());
            }
        }
        for (position, reference) in catalog.references().iter().enumerate() {
            budget.visit(1, stop)?;
            index.component(catalog, reference.source())?;
            budget.charge(&(reference.source(), reference.kind(), position), stop)?;
            index
                .references
                .entry((reference.source(), reference.kind()))
                .or_default()
                .push(position);
        }
        Ok(index)
    }

    pub(super) fn charge(&self, budget: &mut SchemaBudget, stop: &AtomicBool) -> ProjectResult<()> {
        for row in &self.by_id {
            budget.visit(1, stop)?;
            budget.charge(&row, stop)?;
        }
        for row in &self.children {
            budget.visit(1, stop)?;
            budget.charge(&row, stop)?;
        }
        for row in &self.elements {
            budget.visit(1, stop)?;
            budget.charge(&row, stop)?;
        }
        for row in &self.types {
            budget.visit(1, stop)?;
            budget.charge(&row, stop)?;
        }
        for row in &self.references {
            budget.visit(1, stop)?;
            budget.charge(&row, stop)?;
        }
        Ok(())
    }

    pub(super) fn component<'a>(
        &self,
        catalog: &'a AdmittedXmlSchema,
        id: Id,
    ) -> ProjectResult<&'a XmlSchemaComponent> {
        self.by_id
            .get(&id)
            .and_then(|position| catalog.components().get(*position))
            .ok_or_else(|| invalid("classification component is outside the admitted catalog"))
    }

    pub(super) fn children(&self, id: Id) -> &[Id] {
        self.children.get(&id).map_or(&[], Vec::as_slice)
    }

    pub(super) fn candidates(&self, name: &XmlExpandedName) -> &[Id] {
        self.elements.get(name).map_or(&[], Vec::as_slice)
    }

    pub(super) fn unique_global(
        &self,
        catalog: &AdmittedXmlSchema,
        id: Id,
        kind: Kind,
    ) -> ProjectResult<bool> {
        let component = self.component(catalog, id)?;
        if component.kind() != kind || component.state() != State::Observed {
            return Ok(false);
        }
        let Some(name) = component.name() else {
            return Ok(false);
        };
        let symbols = match kind {
            Kind::GlobalElement => &self.elements,
            Kind::NamedComplexType | Kind::NamedSimpleType => &self.types,
            _ => return Ok(false),
        };
        Ok(symbols.get(name).is_some_and(|ids| ids.as_slice() == [id]))
    }

    pub(super) fn observed_child(
        &self,
        catalog: &AdmittedXmlSchema,
        id: Id,
        parent: Id,
        kind: Kind,
    ) -> ProjectResult<bool> {
        let component = self.component(catalog, id)?;
        Ok(component.state() == State::Observed
            && component.kind() == kind
            && component.parent() == Some(parent)
            && component.document() == self.component(catalog, parent)?.document())
    }

    pub(super) fn link<'a>(
        &self,
        catalog: &'a AdmittedXmlSchema,
        source: Id,
        kind: RefKind,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<Link<'a>> {
        budget.visit(1, stop)?;
        let Some(positions) = self.references.get(&(source, kind)) else {
            return Ok(Link::Absent);
        };
        if positions.len() != 1 {
            return Ok(Link::Blocked(Unknown::AmbiguousPath));
        }
        let reference = &catalog.references()[positions[0]];
        if reference.state() != RefState::Unique
            || reference.target().state() != XmlSchemaQNameState::Expanded
            || reference.candidates().len() != 1
            || self.component(catalog, source)?.state() != State::Observed
        {
            return Ok(Link::Blocked(if reference.state() == RefState::Conflict {
                Unknown::AmbiguousPath
            } else {
                Unknown::UnsupportedPath
            }));
        }
        let target = reference.candidates()[0];
        let target_component = self.component(catalog, target)?;
        let expected = if matches!(kind, RefKind::SubstitutionGroup | RefKind::Element) {
            Kind::GlobalElement
        } else {
            target_component.kind()
        };
        if !matches!(
            expected,
            Kind::GlobalElement | Kind::NamedSimpleType | Kind::NamedComplexType
        ) || !self.unique_global(catalog, target, expected)?
        {
            return Ok(Link::Blocked(Unknown::UnsupportedPath));
        }
        Ok(Link::Unique(reference, target))
    }

    pub(super) fn required_link(
        &self,
        catalog: &AdmittedXmlSchema,
        expected: (Id, RefKind, Id),
        output: &mut Vec<Witness>,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        let (source, kind, target) = expected;
        match self.link(catalog, source, kind, budget, stop)? {
            Link::Unique(reference, actual) if actual == target => {
                push_reference(output, reference, actual, budget, stop)
            }
            _ => Err(invalid(
                "reviewed classification anchor lacks its exact UNIQUE reference",
            )),
        }
    }

    pub(super) fn substitution_path(
        &self,
        catalog: &AdmittedXmlSchema,
        start: Id,
        anchor: Id,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<Path> {
        let mut current = start;
        let mut seen = BTreeSet::new();
        let mut reached = false;
        let mut witnesses = Vec::new();
        loop {
            budget.visit(1, stop)?;
            budget.charge(&current, stop)?;
            if !seen.insert(current) {
                return Ok(Path {
                    state: PathState::Blocked(Unknown::Cycle),
                    witnesses,
                });
            }
            if !self.unique_global(catalog, current, Kind::GlobalElement)? {
                return Ok(Path {
                    state: PathState::Blocked(Unknown::UnsupportedPath),
                    witnesses,
                });
            }
            push(
                &mut witnesses,
                Witness::Component { component: current },
                budget,
                stop,
            )?;
            reached |= current == anchor;
            match self.link(catalog, current, RefKind::SubstitutionGroup, budget, stop)? {
                Link::Absent => {
                    return Ok(Path {
                        state: if reached {
                            PathState::Reached
                        } else {
                            PathState::Absent
                        },
                        witnesses,
                    });
                }
                Link::Blocked(reason) => {
                    return Ok(Path {
                        state: PathState::Blocked(reason),
                        witnesses,
                    });
                }
                Link::Unique(reference, target) => {
                    push_reference(&mut witnesses, reference, target, budget, stop)?;
                    current = target;
                }
            }
        }
    }

    pub(super) fn type_path(
        &self,
        catalog: &AdmittedXmlSchema,
        element: Id,
        anchor: Id,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<Path> {
        let mut witnesses = Vec::new();
        push(
            &mut witnesses,
            Witness::Component { component: element },
            budget,
            stop,
        )?;
        let mut current = match self.link(catalog, element, RefKind::Type, budget, stop)? {
            Link::Absent => {
                return Ok(Path {
                    state: PathState::Absent,
                    witnesses,
                });
            }
            Link::Blocked(reason) => {
                return Ok(Path {
                    state: PathState::Blocked(reason),
                    witnesses,
                });
            }
            Link::Unique(reference, target) => {
                push_reference(&mut witnesses, reference, target, budget, stop)?;
                target
            }
        };
        let mut seen = BTreeSet::new();
        let mut reached = false;
        loop {
            budget.visit(1, stop)?;
            budget.charge(&current, stop)?;
            if !seen.insert(current) {
                return Ok(Path {
                    state: PathState::Blocked(Unknown::Cycle),
                    witnesses,
                });
            }
            if !self.unique_global(catalog, current, Kind::NamedComplexType)? {
                return Ok(Path {
                    state: PathState::Blocked(Unknown::UnsupportedPath),
                    witnesses,
                });
            }
            push(
                &mut witnesses,
                Witness::Component { component: current },
                budget,
                stop,
            )?;
            reached |= current == anchor;
            let derivative =
                match self.base_owner(catalog, current, &mut witnesses, budget, stop)? {
                    Ok(Some(id)) => id,
                    Ok(None) => {
                        return Ok(Path {
                            state: if reached {
                                PathState::Reached
                            } else {
                                PathState::Absent
                            },
                            witnesses,
                        });
                    }
                    Err(reason) => {
                        return Ok(Path {
                            state: PathState::Blocked(reason),
                            witnesses,
                        });
                    }
                };
            match self.link(catalog, derivative, RefKind::Base, budget, stop)? {
                Link::Absent => {
                    return Ok(Path {
                        state: PathState::Blocked(Unknown::UnsupportedPath),
                        witnesses,
                    });
                }
                Link::Blocked(reason) => {
                    return Ok(Path {
                        state: PathState::Blocked(reason),
                        witnesses,
                    });
                }
                Link::Unique(reference, target) => {
                    push_reference(&mut witnesses, reference, target, budget, stop)?;
                    current = target;
                }
            }
        }
    }

    /// Base belongs to Extension/Restriction, NOT to the containing type.
    fn base_owner(
        &self,
        catalog: &AdmittedXmlSchema,
        parent: Id,
        witnesses: &mut Vec<Witness>,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<Result<Option<Id>, Unknown>> {
        let mut wrapper = None;
        for id in self.children(parent) {
            budget.visit(1, stop)?;
            let child = self.component(catalog, *id)?;
            if matches!(child.kind(), Kind::SimpleContent | Kind::ComplexContent) {
                if child.state() != State::Observed {
                    return Ok(Err(Unknown::UnsupportedPath));
                }
                if wrapper.replace(*id).is_some() {
                    return Ok(Err(Unknown::AmbiguousPath));
                }
            } else if child.kind() == Kind::Unsupported {
                return Ok(Err(Unknown::UnsupportedPath));
            }
        }
        let Some(wrapper) = wrapper else {
            return Ok(Ok(None));
        };
        push(
            witnesses,
            Witness::Component { component: wrapper },
            budget,
            stop,
        )?;
        push(
            witnesses,
            Witness::Parent {
                child: wrapper,
                parent,
            },
            budget,
            stop,
        )?;
        let mut derivative = None;
        for id in self.children(wrapper) {
            budget.visit(1, stop)?;
            let child = self.component(catalog, *id)?;
            if matches!(child.kind(), Kind::Extension | Kind::Restriction) {
                if child.state() != State::Observed {
                    return Ok(Err(Unknown::UnsupportedPath));
                }
                if derivative.replace(*id).is_some() {
                    return Ok(Err(Unknown::AmbiguousPath));
                }
            } else if child.kind() != Kind::Annotation {
                return Ok(Err(Unknown::UnsupportedPath));
            }
        }
        let Some(derivative) = derivative else {
            return Ok(Err(Unknown::UnsupportedPath));
        };
        push(
            witnesses,
            Witness::Component {
                component: derivative,
            },
            budget,
            stop,
        )?;
        push(
            witnesses,
            Witness::Parent {
                child: derivative,
                parent: wrapper,
            },
            budget,
            stop,
        )?;
        Ok(Ok(Some(derivative)))
    }
}

pub(super) fn push_reference(
    output: &mut Vec<Witness>,
    reference: &XmlSchemaReference,
    target: Id,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.charge(
        &(reference.value_span(), reference.decoded_value_digest()),
        stop,
    )?;
    push(
        output,
        Witness::Reference {
            source: reference.source(),
            reference_kind: reference.kind(),
            target,
            value_span: reference.value_span().clone(),
            decoded_value_digest: reference.decoded_value_digest(),
        },
        budget,
        stop,
    )
}

pub(super) fn count(value: usize, additional: usize) -> ProjectResult<usize> {
    value.checked_add(additional).ok_or_else(exhausted)
}

//! Verification of the explicitly reviewed source-bound policy intent.
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};

use crate::{
    ProjectResult,
    load::schema::{
        AdmittedXmlSchema, SchemaBudget, XmlSchemaAttributeValue as Value,
        XmlSchemaComponentKind as Kind, XmlSchemaReferenceKind as RefKind, budget::MAX_COMPONENTS,
        exhausted, invalid,
    },
};

use super::{
    catalog::{CatalogIndex, PathState, push},
    *,
};

pub(super) fn abstract_element(
    catalog: &AdmittedXmlSchema,
    index: &CatalogIndex,
    id: XmlSchemaComponentId,
) -> ProjectResult<bool> {
    Ok(index
        .component(catalog, id)?
        .attribute("abstract")
        .is_some_and(|attribute| matches!(attribute.interpretation(), Value::Boolean(true))))
}

pub(super) fn canonical_selection(
    selection: &XmlClassificationProfileSelection,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<XmlClassificationProfileSelection> {
    if selection.explicit.len() > MAX_COMPONENTS {
        return Err(exhausted());
    }
    // Charge all caller data BEFORE copying it, including after serde decoding.
    budget.charge(selection, stop)?;
    let mut explicit = BTreeMap::new();
    for anchor in &selection.explicit {
        budget.rows(1, stop)?;
        budget.charge(anchor, stop)?;
        if explicit
            .insert((anchor.element, anchor.class), *anchor)
            .is_some()
        {
            return Err(invalid(
                "reviewed classification contains a duplicate explicit anchor",
            ));
        }
    }
    let mut ordered = Vec::new();
    for anchor in explicit.into_values() {
        budget.visit(1, stop)?;
        budget.charge(&anchor, stop)?;
        ordered.push(anchor);
    }
    let mut canonical = selection.clone();
    canonical.explicit = ordered;
    // A fixed five-row array, not an unbounded caller sort.
    canonical
        .region_like
        .members
        .sort_by_key(|member| member.element_reference);
    budget.visit(5, stop)?;
    Ok(canonical)
}

pub(super) fn verify_anchors(
    catalog: &AdmittedXmlSchema,
    index: &CatalogIndex,
    selection: &XmlClassificationProfileSelection,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<Vec<Vec<XmlClassificationWitness>>> {
    let frame = selection.frame_like;
    require_global(
        catalog,
        index,
        frame.element,
        Kind::GlobalElement,
        budget,
        stop,
    )?;
    require_global(
        catalog,
        index,
        frame.type_component,
        Kind::NamedComplexType,
        budget,
        stop,
    )?;
    if !abstract_element(catalog, index, frame.element)? {
        return Err(invalid(
            "reviewed frame-family anchor must be an abstract global element",
        ));
    }
    let mut frame_witness = Vec::new();
    index.required_link(
        catalog,
        (frame.element, RefKind::Type, frame.type_component),
        &mut frame_witness,
        budget,
        stop,
    )?;
    // Validate cycles/unsupported ancestors even for the selected head.
    if !matches!(
        index
            .substitution_path(catalog, frame.element, frame.element, budget, stop)?
            .state,
        PathState::Reached
    ) || !matches!(
        index
            .type_path(catalog, frame.element, frame.type_component, budget, stop)?
            .state,
        PathState::Reached
    ) {
        return Err(invalid(
            "reviewed frame-family anchor has an unsupported or cyclic typed path",
        ));
    }
    for anchor in &selection.explicit {
        budget.visit(1, stop)?;
        require_global(
            catalog,
            index,
            anchor.element,
            Kind::GlobalElement,
            budget,
            stop,
        )?;
    }
    let region = &selection.region_like;
    require_global(
        catalog,
        index,
        region.layers_element,
        Kind::GlobalElement,
        budget,
        stop,
    )?;
    let chain = [
        (
            region.layers_type,
            region.layers_element,
            Kind::AnonymousComplexType,
        ),
        (region.layers_sequence, region.layers_type, Kind::Sequence),
        (
            region.layer_element,
            region.layers_sequence,
            Kind::LocalElement,
        ),
        (
            region.layer_type,
            region.layer_element,
            Kind::AnonymousComplexType,
        ),
        (region.layer_sequence, region.layer_type, Kind::Sequence),
        (region.choice, region.layer_sequence, Kind::Choice),
    ];
    let mut parent_witnesses = Vec::new();
    push(
        &mut parent_witnesses,
        XmlClassificationWitness::Component {
            component: region.layers_element,
        },
        budget,
        stop,
    )?;
    for (child, parent, kind) in chain {
        budget.visit(1, stop)?;
        if !index.observed_child(catalog, child, parent, kind)? {
            return Err(invalid(
                "reviewed region policy lacks its exact typed parent chain",
            ));
        }
        push(
            &mut parent_witnesses,
            XmlClassificationWitness::Component { component: child },
            budget,
            stop,
        )?;
        push(
            &mut parent_witnesses,
            XmlClassificationWitness::Parent { child, parent },
            budget,
            stop,
        )?;
    }
    let mut actual_refs = BTreeSet::new();
    for id in index.children(region.choice) {
        budget.visit(1, stop)?;
        let component = index.component(catalog, *id)?;
        match component.kind() {
            Kind::Annotation => (),
            Kind::ElementReference => {
                budget.charge(id, stop)?;
                actual_refs.insert(*id);
            }
            _ => {
                return Err(invalid(
                    "reviewed region choice has an unselected content branch",
                ));
            }
        }
    }
    if actual_refs.len() != 5 {
        return Err(invalid(
            "reviewed region choice must retain exactly five Element references",
        ));
    }
    let mut selected_refs = BTreeSet::new();
    let mut selected_elements = BTreeSet::new();
    let mut output = Vec::new();
    for member in &region.members {
        budget.rows(1, stop)?;
        budget.charge(member, stop)?;
        if !selected_refs.insert(member.element_reference)
            || !selected_elements.insert(member.element)
            || !index.observed_child(
                catalog,
                member.element_reference,
                region.choice,
                Kind::ElementReference,
            )?
        {
            return Err(invalid(
                "reviewed region reference witnesses are duplicate or outside the choice",
            ));
        }
        require_global(
            catalog,
            index,
            member.element,
            Kind::GlobalElement,
            budget,
            stop,
        )?;
        require_global(
            catalog,
            index,
            member.type_component,
            Kind::NamedComplexType,
            budget,
            stop,
        )?;
        if abstract_element(catalog, index, member.element)? {
            return Err(invalid(
                "reviewed region reference targets must be concrete global elements",
            ));
        }
        budget.charge(&parent_witnesses, stop)?;
        let mut witnesses = parent_witnesses.clone();
        push(
            &mut witnesses,
            XmlClassificationWitness::Component {
                component: member.element_reference,
            },
            budget,
            stop,
        )?;
        push(
            &mut witnesses,
            XmlClassificationWitness::Parent {
                child: member.element_reference,
                parent: region.choice,
            },
            budget,
            stop,
        )?;
        index.required_link(
            catalog,
            (member.element_reference, RefKind::Element, member.element),
            &mut witnesses,
            budget,
            stop,
        )?;
        push(
            &mut witnesses,
            XmlClassificationWitness::Component {
                component: member.element,
            },
            budget,
            stop,
        )?;
        index.required_link(
            catalog,
            (member.element, RefKind::Type, member.type_component),
            &mut witnesses,
            budget,
            stop,
        )?;
        let type_path =
            index.type_path(catalog, member.element, member.type_component, budget, stop)?;
        if !matches!(type_path.state, PathState::Reached) {
            return Err(invalid(
                "reviewed region member has an unsupported or cyclic typed path",
            ));
        }
        for witness in type_path.witnesses {
            push(&mut witnesses, witness, budget, stop)?;
        }
        push(
            &mut witnesses,
            XmlClassificationWitness::Component {
                component: member.type_component,
            },
            budget,
            stop,
        )?;
        output.push(witnesses);
    }
    if selected_refs != actual_refs {
        return Err(invalid(
            "reviewed region selection differs from the exact retained choice",
        ));
    }
    Ok(output)
}

fn require_global(
    catalog: &AdmittedXmlSchema,
    index: &CatalogIndex,
    id: XmlSchemaComponentId,
    kind: Kind,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    budget.visit(1, stop)?;
    if !index.unique_global(catalog, id, kind)? {
        return Err(invalid(
            "reviewed classification anchor is not a UNIQUE Observed global declaration",
        ));
    }
    Ok(())
}

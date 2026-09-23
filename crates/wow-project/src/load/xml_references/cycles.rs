//! Iterative SCC traversal: parent and inheritance graphs are never conflated.
use super::*;

fn components(edges: &[Vec<usize>], stop: &AtomicBool) -> ProjectResult<Vec<Vec<usize>>> {
    let mut reverse = vec![Vec::new(); edges.len()];
    for (source, targets) in edges.iter().enumerate() {
        checkpoint(stop)?;
        for target in targets {
            reverse[*target].push(source);
        }
    }
    let mut seen = vec![false; edges.len()];
    let mut finish = Vec::with_capacity(edges.len());
    for root in 0..edges.len() {
        checkpoint(stop)?;
        if seen[root] {
            continue;
        }
        seen[root] = true;
        let mut stack = vec![(root, 0)];
        while let Some((node, next)) = stack.last_mut() {
            checkpoint(stop)?;
            if let Some(target) = edges[*node].get(*next) {
                *next += 1;
                if !seen[*target] {
                    seen[*target] = true;
                    stack.push((*target, 0));
                }
            } else {
                finish.push(*node);
                stack.pop();
            }
        }
    }
    seen.fill(false);
    let mut result = Vec::new();
    for root in finish.into_iter().rev() {
        checkpoint(stop)?;
        if seen[root] {
            continue;
        }
        seen[root] = true;
        let mut stack = vec![root];
        let mut component = Vec::new();
        while let Some(node) = stack.pop() {
            checkpoint(stop)?;
            component.push(node);
            for target in &reverse[node] {
                if !seen[*target] {
                    seen[*target] = true;
                    stack.push(*target);
                }
            }
        }
        if component.len() > 1 || edges[root].contains(&root) {
            component.sort_unstable();
            result.push(component);
        }
    }
    result.sort();
    Ok(result)
}

pub(super) fn resolve(
    declarations: &BTreeMap<String, XmlDeclarationSite>,
    references: &mut [XmlReferenceRecord],
    stop: &AtomicBool,
) -> ProjectResult<Vec<XmlReferenceCycle>> {
    let ids: Vec<_> = declarations.keys().cloned().collect();
    let positions: BTreeMap<_, _> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();
    let mut cycles = Vec::new();
    for kind in [XmlReferenceKind::Parent, XmlReferenceKind::Inherits] {
        let mut edges = vec![Vec::new(); ids.len()];
        for reference in references.iter().filter(|r| r.kind == kind) {
            checkpoint(stop)?;
            if let XmlReferenceResolution::UniqueLocalDeclaration { declaration_id } =
                &reference.resolution
            {
                edges[positions[reference.source_id.as_str()]]
                    .push(positions[declaration_id.as_str()]);
            }
        }
        let components = components(&edges, stop)?;
        // One linear edge pass attaches references to components. No enumeration
        // of simple cycles or expansion of ambiguous name candidate combinations.
        let mut membership = vec![None; ids.len()];
        let mut refs = vec![Vec::new(); components.len()];
        for (group, nodes) in components.iter().enumerate() {
            for node in nodes {
                membership[*node] = Some(group);
            }
        }
        for (i, reference) in references
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == kind)
        {
            checkpoint(stop)?;
            if let XmlReferenceResolution::UniqueLocalDeclaration { declaration_id } =
                &reference.resolution
            {
                let source = membership[positions[reference.source_id.as_str()]];
                let target = membership[positions[declaration_id.as_str()]];
                if let Some(group) = source
                    && source == target
                {
                    refs[group].push(i);
                }
            }
        }
        for (nodes, indices) in components.into_iter().zip(refs) {
            checkpoint(stop)?;
            let declaration_ids: Vec<_> = nodes.into_iter().map(|node| ids[node].clone()).collect();
            let reference_ids: Vec<_> = indices
                .iter()
                .map(|i| references[*i].reference_id.clone())
                .collect();
            if reference_ids.is_empty() {
                return Err(super::super::invalid("XML cycle has no edges"));
            }
            let digest = crate::identity::canonical_digest(
                "wow-project/xml-reference-cycle/1",
                &(kind, &declaration_ids, &reference_ids),
                ProjectPhase::Inventory,
            )?;
            let cycle_id = format!("xml-cycle:{digest}");
            for i in indices {
                references[i].cycle_id = Some(cycle_id.clone());
            }
            cycles.push(XmlReferenceCycle {
                cycle_id,
                kind,
                declaration_ids,
                reference_ids,
            });
        }
    }
    Ok(cycles)
}

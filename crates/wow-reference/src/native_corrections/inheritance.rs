//! Guard a single-parent widget graph before any normalized projection changes.
use super::{
    Application, Correction, CorrectionError, CorrectionSet, DocumentationDocument, Location,
    Projection, Result, Status, SystemFacts, SystemOwner, Value, raw_digest, selected,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) fn validate(
    set: &CorrectionSet,
    systems: &[(&DocumentationDocument, SystemFacts<'_>)],
    locations: &[Option<Location>],
    applications: &mut [Application],
    cancelled: &AtomicBool,
) -> Result<()> {
    // Different parent guards still compete for the same child's single base.
    let mut counts = BTreeMap::new();
    for record in &set.records {
        if matches!(record.target.projection, Projection::WidgetBase { .. }) {
            *counts
                .entry((&record.target.path, record.target.registration))
                .or_insert(0usize) += 1;
        }
    }
    let mut edges = BTreeMap::new();
    let mut parent_for = BTreeMap::new();
    for (index, record) in set.records.iter().enumerate() {
        if cancelled.load(Ordering::Relaxed) {
            return Err(CorrectionError::Cancelled);
        }
        let Some(location) = locations[index].filter(|l| l.base) else {
            continue;
        };
        if applications[index].status != Status::Applied {
            continue;
        }
        let result = if counts[&(&record.target.path, record.target.registration)] != 1 {
            Err((Status::Conflict, "multiple_widget_bases"))
        } else {
            parent(systems, record, &set.environment).and_then(|parent| {
                if parent == location.system {
                    Err((Status::Conflict, "widget_base_self_reference"))
                } else {
                    Ok(parent)
                }
            })
        };
        match result {
            Ok(parent) => {
                edges.insert(location.system, index);
                parent_for.insert(index, parent);
            }
            Err((status, reason)) => {
                applications[index].status = status;
                applications[index].reason = reason;
            }
        }
    }
    // Topological admission, without recursion or transitive method copying.
    // Unvisited edges are cycles or depend on cycles; all stay unapplied.
    let mut waiting = BTreeMap::<usize, Vec<usize>>::new();
    let mut ready = BTreeSet::new();
    for (&index, parent) in &parent_for {
        if let Some(parent_record) = edges.get(parent) {
            waiting.entry(*parent_record).or_default().push(index);
        } else {
            ready.insert(index);
        }
    }
    while let Some(index) = ready.pop_first() {
        if cancelled.load(Ordering::Relaxed) {
            return Err(CorrectionError::Cancelled);
        }
        parent_for.remove(&index);
        if let Some(children) = waiting.remove(&index) {
            ready.extend(children);
        }
    }
    for index in parent_for.keys() {
        applications[*index].status = Status::Conflict;
        applications[*index].reason = "widget_base_cycle_or_dependency";
    }
    Ok(())
}

fn parent(
    systems: &[(&DocumentationDocument, SystemFacts<'_>)],
    record: &Correction,
    environment: &str,
) -> std::result::Result<usize, (Status, &'static str)> {
    let Projection::WidgetBase {
        parent_path,
        parent_registration,
        expected_parent_source_sha256,
        expected_parent_raw_sha256,
    } = &record.target.projection
    else {
        return Err((Status::Rejected, "unsupported_widget_base"));
    };
    let mut matches = systems
        .iter()
        .enumerate()
        .filter(|(_, (document, system))| {
            document.path() == parent_path && system.registration_ordinal == *parent_registration
        });
    let (index, (document, system)) = matches
        .next()
        .ok_or((Status::Expired, "widget_base_parent_missing"))?;
    if matches.next().is_some() {
        return Err((Status::Conflict, "widget_base_parent_not_unique"));
    }
    let SystemOwner::ScriptObject(name) = system.owner else {
        return Err((Status::Rejected, "widget_base_parent_not_script_object"));
    };
    let hash = raw_digest(system.raw)
        .map_err(|_| (Status::Rejected, "widget_base_parent_raw_unavailable"))?;
    if document.sha256() != expected_parent_source_sha256
        || &hash != expected_parent_raw_sha256
        || record.after != Value::Text(name.into())
        || !selected(system, environment)
    {
        return Err((Status::Expired, "widget_base_parent_guard_changed"));
    }
    if systems
        .iter()
        .filter(|(_, s)| selected(s, environment))
        .filter(|(_, s)| matches!(s.owner, SystemOwner::ScriptObject(owner) if owner == name))
        .count()
        != 1
    {
        return Err((Status::Conflict, "widget_base_parent_owner_conflict"));
    }
    Ok(index)
}

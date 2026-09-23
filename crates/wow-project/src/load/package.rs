//! Preflight package-wide TOC filters before any referenced file is resolved.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use super::{
    LoadRecordKind, LoadSelection, MAX_RECORDS, Record, TocCondition, TocConditionKind,
    TocLoadContext, budget, conditions,
};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult, disk::checkpoint};

/// Only one active declaration of each filter is admitted. Duplicate definitions
/// have no guessed first/last/union semantics, even when their strings agree.
/// Excluded conditional metadata is inert. Unresolved metadata cannot remove a gate.
pub(super) fn admit(
    text: &str,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<BTreeMap<usize, TocCondition>> {
    let mut filters = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut decision = LoadSelection::Included;
    let mut offset = 0;
    let mut declarations = 0;
    for (ordinal, line) in text.split_inclusive('\n').enumerate() {
        checkpoint(stop)?;
        if ordinal >= MAX_RECORDS || line.len() > 16_384 {
            return Err(budget());
        }
        let end = offset + line.len();
        let content = if offset == 0 {
            line.trim_start_matches('\u{feff}')
        } else {
            line
        };
        if let Some(metadata) = content.trim().strip_prefix("##") {
            let mut record = Record::new(LoadRecordKind::Metadata, offset, end);
            let raw_key = metadata
                .split_once(':')
                .map(|(key, _)| key)
                .unwrap_or(metadata);
            let raw_key = raw_key.trim().to_ascii_lowercase();
            let raw_filter_like =
                raw_key.starts_with("allowload") || raw_key.starts_with("excludeload");
            let metadata = conditions::project(metadata.trim(), &mut record, context, true)?;
            // A malformed trailing bracket must not erase a known filter key.
            if raw_filter_like && record.selection == LoadSelection::Unresolved {
                decision = LoadSelection::Unresolved;
            }
            let (key, value) = metadata.split_once(':').unwrap_or((&metadata, ""));
            let key = key.trim().to_ascii_lowercase();
            let kind = TocConditionKind::parse(&key);
            // A misspelled/future package filter is not an ordinary descriptive
            // tag: opening its descendants would guess that it had no effect.
            let filter_like =
                kind.is_some() || key.starts_with("allowload") || key.starts_with("excludeload");
            if filter_like && record.selection != LoadSelection::Excluded {
                declarations += 1;
                if declarations > 64 {
                    return Err(budget());
                }
                match kind {
                    Some(kind) if record.selection == LoadSelection::Included => {
                        let condition = conditions::condition(kind, value.trim(), context)?;
                        if !seen.insert(key) {
                            decision = LoadSelection::Unresolved;
                        }
                        decision = decision.and(condition.selection);
                        filters.insert(offset, condition);
                    }
                    _ => decision = LoadSelection::Unresolved,
                }
            }
        }
        offset = end;
    }
    checkpoint(stop)?;
    match decision {
        LoadSelection::Included => Ok(filters),
        LoadSelection::Excluded => Err(ProjectError::new(
            ProjectErrorCode::PackageTargetExcluded,
            ProjectPhase::Inventory,
            "selected TOC package is excluded by its target filters",
        )),
        LoadSelection::Unresolved => Err(ProjectError::new(
            ProjectErrorCode::PackageTargetUnresolved,
            ProjectPhase::Inventory,
            "selected TOC package filters are ambiguous or lack target context",
        )),
    }
}

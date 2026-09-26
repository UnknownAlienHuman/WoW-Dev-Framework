//! Read-only selection of a generated-document corpus using the existing TOC parser.
//! Source selection is not package dependency closure or successful client loading.
use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;

use serde::Serialize;

use super::{
    LoadIssue, LoadIssueKind, LoadRecord, LoadRecordKind, LoadSelection, LoadSource,
    TocLoadContext, invalid, resolve, toc,
};
use crate::ProjectResult;
use crate::disk::{DISK_INVENTORY_MAX_FILES, DISK_SOURCE_MAX_BYTES, checkpoint, validate_path};

/// Exact lexical selection receipt. Construction is restricted to this crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocumentTocSelection {
    pub(crate) profile: &'static str,
    pub(crate) source: LoadSource,
    pub(crate) target_interface: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) load_context: Option<TocLoadContext>,
    pub(crate) source_order: Vec<String>,
    pub(crate) records: Vec<LoadRecord>,
    pub(crate) issues: Vec<LoadIssue>,
}

/// Select only direct Lua entries. XML/unknown entries and unresolved predicates
/// reject the operation; explicit exclusions and non-load metadata stay in the receipt.
/// This function neither opens files nor registers analyzer Main/Library inputs.
pub(crate) fn select(
    path: &str,
    text: &str,
    interface: u64,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<DocumentTocSelection> {
    checkpoint(stop)?;
    validate_path(path)?;
    if !path.ends_with(".toc") || text.len() > DISK_SOURCE_MAX_BYTES || text.contains('\0') {
        return Err(invalid("invalid bounded documentation TOC"));
    }
    if let Some(context) = context {
        context.validate()?;
    }
    let parsed = toc::parse(text, interface, context, stop)?;
    let mut source_order = Vec::new();
    let mut records = Vec::with_capacity(parsed.len());
    let mut issues = Vec::new();
    let mut seen = BTreeSet::new();
    for record in parsed {
        checkpoint(stop)?;
        if record.selection == LoadSelection::Unresolved
            || (record.selection == LoadSelection::Included
                && matches!(
                    record.kind,
                    LoadRecordKind::Unknown | LoadRecordKind::XmlFile
                ))
        {
            return Err(invalid(
                "documentation TOC has an unsupported or unresolved entry",
            ));
        }
        let target = record
            .target
            .as_deref()
            .map(|raw| resolve(path, raw))
            .transpose()?;
        if let Some(target) = &target {
            if record.kind != LoadRecordKind::LuaFile || !target.ends_with(".lua") {
                return Err(invalid("documentation TOC must select Lua data files"));
            }
            if source_order.len() >= DISK_INVENTORY_MAX_FILES {
                return Err(super::budget());
            }
            if !seen.insert(target.to_lowercase()) {
                return Err(invalid(
                    "documentation TOC repeats or case-collides a source path",
                ));
            }
            source_order.push(target.clone());
        }
        for kind in record.issues {
            issues.push(LoadIssue {
                kind,
                document: path.to_owned(),
                byte_start: record.start as u64,
                byte_end: record.end as u64,
                blocks_complete: kind != LoadIssueKind::OptionalDependencyUnresolved,
            });
        }
        records.push(LoadRecord {
            ordinal: records.len() as u64,
            document: path.to_owned(),
            byte_start: record.start as u64,
            byte_end: record.end as u64,
            kind: record.kind,
            raw_digest: crate::identity::source_digest(&text.as_bytes()[record.start..record.end]),
            target,
            bootstrap: record.bootstrap,
            selection: record.selection,
            declared_target: record.declared_target,
            conditions: record.conditions,
            saved_variables: record.saved_variables,
        });
    }
    if source_order.is_empty() {
        return Err(invalid("documentation TOC selects no Lua inputs"));
    }
    checkpoint(stop)?;
    Ok(DocumentTocSelection {
        profile: "wow-project/document-toc/1",
        source: LoadSource {
            path: path.to_owned(),
            content_digest: crate::identity::source_digest(text.as_bytes()),
            byte_length: text.len() as u64,
        },
        target_interface: interface,
        load_context: context.cloned(),
        source_order,
        records,
        issues,
    })
}

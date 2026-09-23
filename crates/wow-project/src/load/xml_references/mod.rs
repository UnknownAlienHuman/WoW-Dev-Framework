//! Bounded linking of source declarations in one captured XML closure. A unique
//! local declaration is not a runtime object, an XSD type, or a Lua symbol.
mod cycles;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, SourceContent};

use super::{LoadRecord, LoadRecordKind, XmlDocumentIndex, XmlElementRecord, XmlSourceSpan};
use crate::{ProjectPhase, ProjectResult, disk::checkpoint};

pub const XML_REFERENCE_PROFILE: &str = "wow-project/xml-local-references/1";
const MAX_DECLARATIONS: usize = 32_768;
const MAX_REFERENCES: usize = 65_536;
const MAX_ISSUES: usize = 131_072;
const MAX_RETAINED_TEXT_BYTES: usize = 16 * 1024 * 1024;

// Bound replicated source spellings/paths, not just record counts. IDs and spans
// have fixed bounds; this additionally prevents a long document path per node
// from multiplying a small input into an excessive reference report.
fn retain_text(used: &mut usize, bytes: usize) -> ProjectResult<()> {
    *used = used.checked_add(bytes).ok_or_else(super::budget)?;
    if *used > MAX_RETAINED_TEXT_BYTES {
        return Err(super::budget());
    }
    Ok(())
}

/// Source identity is separate from name lookup: duplicate and anonymous
/// declarations retain their own occurrence IDs. XML containment is not `parent`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlDeclarationSite {
    pub document: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub occurrence_id: String,
    pub element_name: String,
    pub span: XmlSourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_occurrence_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_template: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intrinsic: Option<bool>,
    pub valid_declaration: bool,
    /// Actual load-record ordinals, including repeated include occurrences.
    pub load_ordinals: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlReferenceKind {
    Parent,
    Inherits,
}

/// Lookup is restricted to the current captured XML declarations. In particular,
/// NotInCapturedScope does not prove absence from dependencies or the client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum XmlReferenceResolution {
    UniqueLocalDeclaration {
        declaration_id: String,
    },
    /// Key into the report's shared `names` table; no quadratic candidate copies.
    AmbiguousName {
        name_group: String,
    },
    NotInCapturedScope,
    DynamicName,
    UnsupportedName,
    InvalidSource,
    InvalidTarget {
        declaration_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlReferenceOrder {
    TargetBeforeSource,
    TargetAfterSource,
    SelfReference,
    RepeatedLoad,
    Unrecorded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlReferenceRecord {
    pub reference_id: String,
    pub source_id: String,
    pub kind: XmlReferenceKind,
    /// Zero-based position in the source inheritance list; parent uses zero.
    pub ordinal: u64,
    pub name: String,
    /// Complete original attribute value, not a guessed decoded-substring span.
    pub attribute_span: XmlSourceSpan,
    pub resolution: XmlReferenceResolution,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<XmlReferenceOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlReferenceCycle {
    pub cycle_id: String,
    pub kind: XmlReferenceKind,
    pub declaration_ids: Vec<String>,
    pub reference_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlReferenceIssueKind {
    DuplicateName,
    InvalidDeclaration,
    AmbiguousReference,
    NotInCapturedScope,
    DynamicReference,
    UnsupportedReference,
    InvalidSource,
    InvalidTarget,
    DuplicateInheritance,
    TargetAfterSource,
    RepeatedLoad,
    UnrecordedLoad,
    ParentCycle,
    InheritanceCycle,
}
impl XmlReferenceIssueKind {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DuplicateName => "xml.duplicate_name",
            Self::InvalidDeclaration => "xml.invalid_declaration",
            Self::AmbiguousReference => "xml.reference.ambiguous",
            Self::NotInCapturedScope => "xml.reference.not_in_captured_scope",
            Self::DynamicReference => "xml.reference.dynamic",
            Self::UnsupportedReference => "xml.reference.unsupported_spelling",
            Self::InvalidSource => "xml.reference.invalid_source",
            Self::InvalidTarget => "xml.reference.invalid_target",
            Self::DuplicateInheritance => "xml.inherits.duplicate_entry",
            Self::TargetAfterSource => "xml.reference.target_after_source",
            Self::RepeatedLoad => "xml.reference.repeated_load",
            Self::UnrecordedLoad => "xml.reference.unrecorded_load",
            Self::ParentCycle => "xml.parent.source_cycle",
            Self::InheritanceCycle => "xml.inherits.source_cycle",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlReferenceIssue {
    pub kind: XmlReferenceIssueKind,
    pub source_id: String,
    pub span: XmlSourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_id: Option<String>,
}

/// Constructed from owner indexes, never from untrusted serialized links. The
/// enclosing load plan binds the target profile, source closure and generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlReferenceReport {
    profile: &'static str,
    scope: &'static str,
    document_indexes: BTreeMap<String, ContentDigest<CanonicalResult>>,
    declarations: BTreeMap<String, XmlDeclarationSite>,
    names: BTreeMap<String, Vec<String>>,
    references: Vec<XmlReferenceRecord>,
    cycles: Vec<XmlReferenceCycle>,
    issues: Vec<XmlReferenceIssue>,
    digest: ContentDigest<CanonicalResult>,
}
impl XmlReferenceReport {
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
    #[must_use]
    pub fn declarations(&self) -> &BTreeMap<String, XmlDeclarationSite> {
        &self.declarations
    }
    #[must_use]
    pub fn names(&self) -> &BTreeMap<String, Vec<String>> {
        &self.names
    }
    #[must_use]
    pub fn references(&self) -> &[XmlReferenceRecord] {
        &self.references
    }
    #[must_use]
    pub fn cycles(&self) -> &[XmlReferenceCycle] {
        &self.cycles
    }
    #[must_use]
    pub fn issues(&self) -> &[XmlReferenceIssue] {
        &self.issues
    }
    /// This narrow flag says only that emitted local name links have no blockers.
    /// It does not establish complete XML, dependency, type or runtime coverage.
    #[must_use]
    pub fn local_links_resolved(&self) -> bool {
        self.issues.is_empty()
    }
    #[must_use]
    pub fn unique_link_count(&self) -> usize {
        self.references
            .iter()
            .filter(|r| {
                matches!(
                    r.resolution,
                    XmlReferenceResolution::UniqueLocalDeclaration { .. }
                )
            })
            .count()
    }
}

fn issue(
    issues: &mut Vec<XmlReferenceIssue>,
    kind: XmlReferenceIssueKind,
    site: &XmlDeclarationSite,
    span: &XmlSourceSpan,
    reference_id: Option<&str>,
) -> ProjectResult<()> {
    if issues.len() >= MAX_ISSUES {
        return Err(super::budget());
    }
    issues.push(XmlReferenceIssue {
        kind,
        source_id: site.occurrence_id.clone(),
        span: span.clone(),
        reference_id: reference_id.map(str::to_owned),
        cycle_id: None,
    });
    Ok(())
}

/// Exact matching only. Do not expand $parent, choose a last definition, infer
/// a Lua symbol, or treat a dependency not captured here as missing from WoW.
fn lookup(
    name: &str,
    source: &XmlDeclarationSite,
    names: &BTreeMap<String, Vec<String>>,
    sites: &BTreeMap<String, XmlDeclarationSite>,
) -> XmlReferenceResolution {
    if !source.valid_declaration {
        return XmlReferenceResolution::InvalidSource;
    }
    if name.contains(['$', '[', ']']) {
        return XmlReferenceResolution::DynamicName;
    }
    let mut bytes = name.bytes();
    if name.len() > 4096
        || !bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        || !bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return XmlReferenceResolution::UnsupportedName;
    }
    let Some(group) = names.get(name) else {
        return XmlReferenceResolution::NotInCapturedScope;
    };
    if group.len() != 1 {
        return XmlReferenceResolution::AmbiguousName {
            name_group: name.to_owned(),
        };
    }
    let target = &group[0];
    if !sites[target].valid_declaration {
        return XmlReferenceResolution::InvalidTarget {
            declaration_id: target.clone(),
        };
    }
    XmlReferenceResolution::UniqueLocalDeclaration {
        declaration_id: target.clone(),
    }
}

fn order(source: &XmlDeclarationSite, target: &XmlDeclarationSite) -> XmlReferenceOrder {
    if source.load_ordinals.len() > 1 || target.load_ordinals.len() > 1 {
        return XmlReferenceOrder::RepeatedLoad;
    }
    let ([first], [second]) = (
        source.load_ordinals.as_slice(),
        target.load_ordinals.as_slice(),
    ) else {
        return XmlReferenceOrder::Unrecorded;
    };
    if source.occurrence_id == target.occurrence_id {
        XmlReferenceOrder::SelfReference
    } else if second < first {
        XmlReferenceOrder::TargetBeforeSource
    } else {
        XmlReferenceOrder::TargetAfterSource
    }
}

fn attribute_span<'a>(
    element: &'a XmlElementRecord,
    name: &str,
) -> ProjectResult<&'a XmlSourceSpan> {
    element
        .attributes
        .iter()
        .find(|a| a.qualified_name == name)
        .map(|a| &a.value_span)
        .ok_or_else(|| super::invalid("XML reference has no source attribute"))
}

pub(super) fn resolve(
    documents: &BTreeMap<String, XmlDocumentIndex>,
    load_records: &[LoadRecord],
    stop: &AtomicBool,
) -> ProjectResult<XmlReferenceReport> {
    checkpoint(stop)?;
    let mut declarations = BTreeMap::new();
    let mut retained_text_bytes = 0;
    let mut names: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut offsets = BTreeMap::new();
    let mut issues = Vec::new();
    for (document, index) in documents {
        for element in index.declarations() {
            checkpoint(stop)?;
            if declarations.len() >= MAX_DECLARATIONS {
                return Err(super::budget());
            }
            let declaration = element
                .declaration
                .as_ref()
                .ok_or_else(|| super::invalid("XML declaration index is inconsistent"))?;
            let id = &element.occurrence_id;
            let name_bytes = declaration.name.as_ref().map_or(0, String::len);
            retain_text(
                &mut retained_text_bytes,
                document.len() + element.qualified_name.len() + name_bytes * 2,
            )?;
            let site = XmlDeclarationSite {
                document: document.clone(),
                content_digest: index.source_digest(),
                occurrence_id: id.clone(),
                element_name: element.qualified_name.clone(),
                span: element.start_tag_span.clone(),
                name: declaration.name.clone(),
                container_occurrence_id: element.parent_occurrence_id.clone(),
                virtual_template: declaration.virtual_template,
                intrinsic: declaration.intrinsic,
                valid_declaration: element.issues.is_empty(),
                load_ordinals: Vec::new(),
            };
            if !site.valid_declaration {
                issue(
                    &mut issues,
                    XmlReferenceIssueKind::InvalidDeclaration,
                    &site,
                    &site.span,
                    None,
                )?;
            }
            if let Some(name) = &site.name {
                names.entry(name.clone()).or_default().push(id.clone());
            }
            // End offset also works when a BOM precedes the first tag.
            if offsets
                .insert(
                    (document.as_str(), element.start_tag_span.byte_end),
                    id.clone(),
                )
                .is_some()
                || declarations.insert(id.clone(), site).is_some()
            {
                return Err(super::invalid("XML source occurrence is not unique"));
            }
        }
    }
    for record in load_records {
        checkpoint(stop)?;
        if record.kind == LoadRecordKind::XmlElement
            && let Some(id) = offsets.get(&(record.document.as_str(), record.byte_end))
        {
            declarations
                .get_mut(id)
                .ok_or_else(|| super::invalid("XML load occurrence is missing"))?
                .load_ordinals
                .push(record.ordinal);
        }
    }
    // Keep all duplicate definitions, including malformed ones. Filtering an
    // invalid candidate before lookup could falsely make another one unique.
    for group in names.values_mut() {
        checkpoint(stop)?;
        group.sort();
        if group.len() > 1 {
            for id in group.iter() {
                checkpoint(stop)?;
                let site = &declarations[id];
                issue(
                    &mut issues,
                    XmlReferenceIssueKind::DuplicateName,
                    site,
                    &site.span,
                    None,
                )?;
            }
        }
    }
    let mut references = Vec::new();
    for index in documents.values() {
        for element in index.declarations() {
            checkpoint(stop)?;
            let declaration = element
                .declaration
                .as_ref()
                .ok_or_else(|| super::invalid("XML declaration is missing"))?;
            let source = &declarations[&element.occurrence_id];
            let mut seen = BTreeSet::new();
            let parent = declaration
                .parent_reference
                .iter()
                .map(|s| (XmlReferenceKind::Parent, 0, s));
            let inherits = declaration
                .inherits
                .iter()
                .enumerate()
                .map(|(ordinal, s)| (XmlReferenceKind::Inherits, ordinal as u64, s));
            for (kind, ordinal, name) in parent.chain(inherits) {
                checkpoint(stop)?;
                if references.len() >= MAX_REFERENCES {
                    return Err(super::budget());
                }
                retain_text(&mut retained_text_bytes, name.len() * 2)?;
                let span = attribute_span(
                    element,
                    match kind {
                        XmlReferenceKind::Parent => "parent",
                        XmlReferenceKind::Inherits => "inherits",
                    },
                )?;
                let resolution = lookup(name, source, &names, &declarations);
                let load_order = match &resolution {
                    XmlReferenceResolution::UniqueLocalDeclaration { declaration_id } => {
                        Some(order(source, &declarations[declaration_id]))
                    }
                    _ => None,
                };
                let digest = crate::identity::canonical_digest(
                    XML_REFERENCE_PROFILE,
                    &(&source.occurrence_id, kind, ordinal, name, span),
                    ProjectPhase::Inventory,
                )?;
                let reference_id = format!("xml-reference:{digest}");
                let problem = match &resolution {
                    XmlReferenceResolution::UniqueLocalDeclaration { .. } => None,
                    XmlReferenceResolution::AmbiguousName { .. } => {
                        Some(XmlReferenceIssueKind::AmbiguousReference)
                    }
                    XmlReferenceResolution::NotInCapturedScope => {
                        Some(XmlReferenceIssueKind::NotInCapturedScope)
                    }
                    XmlReferenceResolution::DynamicName => {
                        Some(XmlReferenceIssueKind::DynamicReference)
                    }
                    XmlReferenceResolution::UnsupportedName => {
                        Some(XmlReferenceIssueKind::UnsupportedReference)
                    }
                    XmlReferenceResolution::InvalidSource => {
                        Some(XmlReferenceIssueKind::InvalidSource)
                    }
                    XmlReferenceResolution::InvalidTarget { .. } => {
                        Some(XmlReferenceIssueKind::InvalidTarget)
                    }
                };
                if let Some(problem) = problem {
                    issue(&mut issues, problem, source, span, Some(&reference_id))?;
                }
                let order_problem = match load_order {
                    Some(XmlReferenceOrder::TargetAfterSource) => {
                        Some(XmlReferenceIssueKind::TargetAfterSource)
                    }
                    Some(XmlReferenceOrder::RepeatedLoad) => {
                        Some(XmlReferenceIssueKind::RepeatedLoad)
                    }
                    Some(XmlReferenceOrder::Unrecorded) => {
                        Some(XmlReferenceIssueKind::UnrecordedLoad)
                    }
                    _ => None,
                };
                if let Some(problem) = order_problem {
                    issue(&mut issues, problem, source, span, Some(&reference_id))?;
                }
                if kind == XmlReferenceKind::Inherits && !seen.insert(name.as_str()) {
                    issue(
                        &mut issues,
                        XmlReferenceIssueKind::DuplicateInheritance,
                        source,
                        span,
                        Some(&reference_id),
                    )?;
                }
                references.push(XmlReferenceRecord {
                    reference_id,
                    source_id: source.occurrence_id.clone(),
                    kind,
                    ordinal,
                    name: name.clone(),
                    attribute_span: span.clone(),
                    resolution,
                    order: load_order,
                    cycle_id: None,
                });
            }
        }
    }
    let cycles = cycles::resolve(&declarations, &mut references, stop)?;
    let by_reference: BTreeMap<_, _> = references
        .iter()
        .map(|r| (r.reference_id.as_str(), r))
        .collect();
    for cycle in &cycles {
        checkpoint(stop)?;
        let first = by_reference
            .get(cycle.reference_ids[0].as_str())
            .ok_or_else(|| super::invalid("XML cycle has no source reference"))?;
        let source = &declarations[&first.source_id];
        let kind = match cycle.kind {
            XmlReferenceKind::Parent => XmlReferenceIssueKind::ParentCycle,
            XmlReferenceKind::Inherits => XmlReferenceIssueKind::InheritanceCycle,
        };
        issue(
            &mut issues,
            kind,
            source,
            &first.attribute_span,
            Some(&first.reference_id),
        )?;
        if let Some(last) = issues.last_mut() {
            last.cycle_id = Some(cycle.cycle_id.clone());
        }
    }
    drop(by_reference);
    let document_indexes: BTreeMap<String, ContentDigest<CanonicalResult>> = documents
        .iter()
        .map(|(path, index)| (path.clone(), index.digest()))
        .collect();
    let scope = "captured_xml_declarations_only";
    let digest = crate::identity::canonical_digest(
        XML_REFERENCE_PROFILE,
        &(
            XML_REFERENCE_PROFILE,
            scope,
            &document_indexes,
            &declarations,
            &names,
            &references,
            &cycles,
            &issues,
        ),
        ProjectPhase::Inventory,
    )?;
    checkpoint(stop)?;
    Ok(XmlReferenceReport {
        profile: XML_REFERENCE_PROFILE,
        scope,
        document_indexes,
        declarations,
        names,
        references,
        cycles,
        issues,
        digest,
    })
}

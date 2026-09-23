//! Enumerate source-declared mixins across the existing XML inheritance graph.
//! This retains candidate provenance, not effective receiver state or precedence.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::AtomicBool;

use serde::Serialize;

use super::{MAX_BINDINGS, attribute_span, exhausted, invalid};
use crate::ProjectResult;
use crate::load::xml_references::{
    XmlDeclarationSite, XmlReferenceKind, XmlReferenceOrder, XmlReferenceRecord,
    XmlReferenceResolution,
};
use crate::load::{ProjectLoadPlan, XmlElementRecord, XmlSourceSpan};

const MAX_EXPANSION_STEPS: usize = 262_144;
const MAX_RECEIVER_RECORDS: usize = 65_536;
const MAX_RECEIVER_TEXT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlReceiverMixinSource {
    /// Key into the load plan's XML declaration table, not a Lua symbol ID.
    pub declaration_id: String,
    /// Source order within this declaration only; not inheritance precedence.
    pub ordinal: usize,
    pub name: String,
    pub attribute_span: XmlSourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlReceiverBlockerKind {
    NoDeclaration,
    InvalidDeclaration,
    DeclarationConflict,
    SourceLoadUnresolved,
    UnresolvedInheritance,
    TargetNotDeclaredTemplate,
    InheritanceOrderUnresolved,
    InheritanceConflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlReceiverBlocker {
    pub source_id: String,
    pub reason: XmlReceiverBlockerKind,
    /// Key into the existing XML reference table, which retains the full cause.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

/// Cached once per exact method owner; all its handlers share this source graph.
/// A complete enumeration still does not establish a runtime receiver or method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlReceiverSources {
    pub owner_id: String,
    /// Distinct visited XML declaration IDs. Diamond paths do not copy nodes.
    pub declarations: Vec<String>,
    /// All inspected inheritance references, including rejected and shared edges.
    pub references: Vec<String>,
    pub mixins: Vec<XmlReceiverMixinSource>,
    pub blockers: Vec<XmlReceiverBlocker>,
    pub complete: bool,
}

#[derive(Default)]
struct Budget {
    steps: usize,
    records: usize,
    text_bytes: usize,
}
impl Budget {
    fn step(&mut self) -> ProjectResult<()> {
        self.steps = self.steps.checked_add(1).ok_or_else(exhausted)?;
        if self.steps > MAX_EXPANSION_STEPS {
            return Err(exhausted());
        }
        Ok(())
    }

    fn retain(&mut self, bytes: usize) -> ProjectResult<()> {
        self.records = self.records.checked_add(1).ok_or_else(exhausted)?;
        self.text_bytes = self.text_bytes.checked_add(bytes).ok_or_else(exhausted)?;
        if self.records > MAX_RECEIVER_RECORDS || self.text_bytes > MAX_RECEIVER_TEXT_BYTES {
            return Err(exhausted());
        }
        Ok(())
    }
}

/// Borrow owner-produced indexes. Never reread source, expand ambiguous name
/// groups, follow `parent`, or derive templates from display names.
pub(super) struct Resolver<'a> {
    elements: BTreeMap<&'a str, &'a XmlElementRecord>,
    sites: &'a BTreeMap<String, XmlDeclarationSite>,
    inherits: BTreeMap<&'a str, Vec<&'a XmlReferenceRecord>>,
    blocked_references: BTreeSet<&'a str>,
    blocked_declarations: BTreeSet<&'a str>,
    budget: Budget,
    receivers: BTreeMap<String, XmlReceiverSources>,
}
impl<'a> Resolver<'a> {
    pub(super) fn new(plan: &'a ProjectLoadPlan, stop: &AtomicBool) -> ProjectResult<Self> {
        let mut elements = BTreeMap::new();
        for index in plan.xml_documents().values() {
            for element in index.elements() {
                crate::analyzer::checkpoint(stop)?;
                if elements
                    .insert(element.occurrence_id.as_str(), element)
                    .is_some()
                {
                    return Err(invalid());
                }
            }
        }
        let links = plan.xml_references();
        let mut inherits: BTreeMap<&str, Vec<&XmlReferenceRecord>> = BTreeMap::new();
        for reference in links.references() {
            crate::analyzer::checkpoint(stop)?;
            if reference.kind == XmlReferenceKind::Inherits {
                inherits
                    .entry(reference.source_id.as_str())
                    .or_default()
                    .push(reference);
            }
        }
        for edges in inherits.values_mut() {
            crate::analyzer::checkpoint(stop)?;
            edges.sort_by(|first, second| {
                first
                    .ordinal
                    .cmp(&second.ordinal)
                    .then_with(|| first.reference_id.cmp(&second.reference_id))
            });
        }
        let mut blocked_references = BTreeSet::new();
        let mut blocked_declarations = BTreeSet::new();
        for issue in links.issues() {
            crate::analyzer::checkpoint(stop)?;
            if let Some(id) = issue.reference_id.as_deref() {
                blocked_references.insert(id);
            } else {
                blocked_declarations.insert(issue.source_id.as_str());
            }
        }
        Ok(Self {
            elements,
            sites: links.declarations(),
            inherits,
            blocked_references,
            blocked_declarations,
            budget: Budget::default(),
            receivers: BTreeMap::new(),
        })
    }

    pub(super) fn resolve(
        &mut self,
        owner_id: &str,
        stop: &AtomicBool,
    ) -> ProjectResult<&XmlReceiverSources> {
        crate::analyzer::checkpoint(stop)?;
        if !self.receivers.contains_key(owner_id) {
            if self.receivers.len() >= MAX_BINDINGS {
                return Err(exhausted());
            }
            // Charge both the shared-table key and the explicit owner ID.
            self.budget
                .retain(owner_id.len().checked_mul(2).ok_or_else(exhausted)?)?;
            let receipt = self.collect(owner_id, stop)?;
            self.receivers.insert(owner_id.to_owned(), receipt);
        }
        self.receivers.get(owner_id).ok_or_else(invalid)
    }

    pub(super) fn into_sources(self) -> BTreeMap<String, XmlReceiverSources> {
        self.receivers
    }

    fn collect(&mut self, owner_id: &str, stop: &AtomicBool) -> ProjectResult<XmlReceiverSources> {
        // Get a reference owned by the frozen XML index, not by this cache.
        let owner = self.elements.get(owner_id).copied().ok_or_else(invalid)?;
        let mut seen = BTreeSet::from([owner.occurrence_id.as_str()]);
        let mut pending = VecDeque::from([owner.occurrence_id.as_str()]);
        let mut receipt = XmlReceiverSources {
            owner_id: owner_id.to_owned(),
            declarations: Vec::new(),
            references: Vec::new(),
            mixins: Vec::new(),
            blockers: Vec::new(),
            complete: false,
        };
        while let Some(id) = pending.pop_front() {
            crate::analyzer::checkpoint(stop)?;
            self.budget.step()?;
            let element = self.elements.get(id).copied().ok_or_else(invalid)?;
            let Some(declaration) = element.declaration.as_ref() else {
                block(
                    &mut receipt,
                    &mut self.budget,
                    id,
                    XmlReceiverBlockerKind::NoDeclaration,
                    None,
                )?;
                continue;
            };
            let site = self.sites.get(id).ok_or_else(invalid)?;
            self.budget.retain(id.len())?;
            receipt.declarations.push(id.to_owned());
            if !element.ui_namespace || !site.valid_declaration || !element.issues.is_empty() {
                block(
                    &mut receipt,
                    &mut self.budget,
                    id,
                    XmlReceiverBlockerKind::InvalidDeclaration,
                    None,
                )?;
                continue;
            }
            if self.blocked_declarations.contains(id) {
                block(
                    &mut receipt,
                    &mut self.budget,
                    id,
                    XmlReceiverBlockerKind::DeclarationConflict,
                    None,
                )?;
            }
            if site.load_ordinals.len() != 1 {
                block(
                    &mut receipt,
                    &mut self.budget,
                    id,
                    XmlReceiverBlockerKind::SourceLoadUnresolved,
                    None,
                )?;
            }
            for (ordinal, name) in declaration.mixins.iter().enumerate() {
                crate::analyzer::checkpoint(stop)?;
                self.budget
                    .retain(id.len().checked_add(name.len()).ok_or_else(exhausted)?)?;
                receipt.mixins.push(XmlReceiverMixinSource {
                    declaration_id: id.to_owned(),
                    ordinal,
                    name: name.clone(),
                    attribute_span: attribute_span(element, "mixin")?,
                });
            }
            let edges = self.inherits.get(id).map(Vec::as_slice).unwrap_or_default();
            if edges.len() != declaration.inherits.len() {
                return Err(invalid());
            }
            for reference in edges {
                crate::analyzer::checkpoint(stop)?;
                self.budget.step()?;
                self.budget.retain(reference.reference_id.len())?;
                receipt.references.push(reference.reference_id.clone());
                let reference_id = Some(reference.reference_id.as_str());
                let XmlReferenceResolution::UniqueLocalDeclaration { declaration_id } =
                    &reference.resolution
                else {
                    block(
                        &mut receipt,
                        &mut self.budget,
                        id,
                        XmlReceiverBlockerKind::UnresolvedInheritance,
                        reference_id,
                    )?;
                    continue;
                };
                let target = self.sites.get(declaration_id).ok_or_else(invalid)?;
                if !target.valid_declaration
                    || !(target.virtual_template == Some(true) || target.intrinsic == Some(true))
                {
                    block(
                        &mut receipt,
                        &mut self.budget,
                        id,
                        XmlReceiverBlockerKind::TargetNotDeclaredTemplate,
                        reference_id,
                    )?;
                    continue;
                }
                if reference.cycle_id.is_some()
                    || self
                        .blocked_references
                        .contains(reference.reference_id.as_str())
                {
                    block(
                        &mut receipt,
                        &mut self.budget,
                        id,
                        XmlReceiverBlockerKind::InheritanceConflict,
                        reference_id,
                    )?;
                }
                if reference.order != Some(XmlReferenceOrder::TargetBeforeSource) {
                    block(
                        &mut receipt,
                        &mut self.budget,
                        id,
                        XmlReceiverBlockerKind::InheritanceOrderUnresolved,
                        reference_id,
                    )?;
                }
                // Retain source candidates even on forward/cyclic edges, with
                // blockers intact. Never enumerate all routes through a diamond.
                if seen.insert(declaration_id.as_str()) {
                    pending.push_back(declaration_id.as_str());
                }
            }
        }
        receipt.complete = receipt.blockers.is_empty();
        Ok(receipt)
    }
}

fn block(
    receipt: &mut XmlReceiverSources,
    budget: &mut Budget,
    source_id: &str,
    reason: XmlReceiverBlockerKind,
    reference_id: Option<&str>,
) -> ProjectResult<()> {
    budget.retain(
        source_id
            .len()
            .checked_add(reference_id.map_or(0, str::len))
            .ok_or_else(exhausted)?,
    )?;
    receipt.blockers.push(XmlReceiverBlocker {
        source_id: source_id.to_owned(),
        reason,
        reference_id: reference_id.map(str::to_owned),
    });
    Ok(())
}

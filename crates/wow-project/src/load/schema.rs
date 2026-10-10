//! Explicit same-source XSD component admission through the existing XML owner.
//! The capability records source structure, not XSD instance or runtime validity.
mod budget;
mod components;
mod model;
mod names;

use budget::{MAX_COMPONENTS, MAX_DOCUMENTS, MAX_ROWS, SchemaBudget};
pub use model::*;

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    sync::{Arc, atomic::AtomicBool},
};
use wow_core::{CanonicalResult, ContentDigest, SourceContent};

use super::{XmlDocumentIndex, xml};
use crate::{
    ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult,
    disk::{DISK_INVENTORY_MAX_BYTES, DISK_SOURCE_MAX_BYTES, checkpoint},
    platform_source::{AdmittedPlatformSource, PlatformFileKind},
};

pub const XML_SCHEMA_PROFILE: &str = "wow-project/xml-schema-components/1";
pub const XML_SCHEMA_POLICY_PROFILE: &str = "wow-project/xml-schema-component-policy/1";

/// An exact caller assertion, verified against the native Included member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XmlSchemaMemberSelection {
    pub path: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
}

/// Explicit bounded selection; decoding it does not admit a schema capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XmlSchemaSelection {
    pub source_snapshot_id: String,
    pub profile_digest: ContentDigest<CanonicalResult>,
    pub content_manifest_digest: ContentDigest<CanonicalResult>,
    pub admission_digest: ContentDigest<CanonicalResult>,
    pub members: Vec<XmlSchemaMemberSelection>,
}

impl XmlSchemaSelection {
    /// Form an explicit selection from actual held members, without discovery.
    pub fn for_source(
        source: &AdmittedPlatformSource,
        paths: &[&str],
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        checkpoint(stop)?;
        if paths.is_empty() {
            return Err(invalid(
                "schema selection requires at least one explicit member",
            ));
        }
        if paths.len() > MAX_DOCUMENTS {
            return Err(exhausted());
        }
        let mut seen = BTreeSet::new();
        let mut members = Vec::new();
        let mut total = 0;
        for path in paths {
            checkpoint(stop)?;
            if path.len() > 4096 || !seen.insert(*path) {
                return Err(invalid("schema paths must be unique bounded native paths"));
            }
            let member = source.raw_member(path, stop)?;
            require_schema(member.kind(), member.path())?;
            charge_input(member.byte_length(), &mut total)?;
            members.push(XmlSchemaMemberSelection {
                path: member.path().to_owned(),
                content_digest: member.content_digest(),
                byte_length: member.byte_length(),
            });
        }
        members.sort_by(|a, b| a.path.cmp(&b.path));
        let receipt = source.receipt();
        checkpoint(stop)?;
        Ok(Self {
            source_snapshot_id: receipt.source_snapshot_id().to_owned(),
            profile_digest: receipt.profile_digest(),
            content_manifest_digest: receipt.content_manifest_digest(),
            admission_digest: receipt.admission_digest(),
            members,
        })
    }
}

/// Serialize-only source observations; they cannot reconstruct the native owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSchemaReceipt {
    profile: &'static str,
    policy_profile: &'static str,
    source_snapshot_id: String,
    source_profile_digest: ContentDigest<CanonicalResult>,
    content_manifest_digest: ContentDigest<CanonicalResult>,
    admission_digest: ContentDigest<CanonicalResult>,
    selection_digest: ContentDigest<CanonicalResult>,
    digest: ContentDigest<CanonicalResult>,
    members: Vec<XmlSchemaMemberSelection>,
    component_count: usize,
    reference_count: usize,
    issue_count: usize,
}

impl XmlSchemaReceipt {
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
    }
    #[must_use]
    pub const fn policy_profile(&self) -> &'static str {
        self.policy_profile
    }
    #[must_use]
    pub fn source_snapshot_id(&self) -> &str {
        &self.source_snapshot_id
    }
    #[must_use]
    pub const fn source_profile_digest(&self) -> ContentDigest<CanonicalResult> {
        self.source_profile_digest
    }
    #[must_use]
    pub const fn content_manifest_digest(&self) -> ContentDigest<CanonicalResult> {
        self.content_manifest_digest
    }
    #[must_use]
    pub const fn admission_digest(&self) -> ContentDigest<CanonicalResult> {
        self.admission_digest
    }
    #[must_use]
    pub const fn selection_digest(&self) -> ContentDigest<CanonicalResult> {
        self.selection_digest
    }
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
    #[must_use]
    pub fn members(&self) -> &[XmlSchemaMemberSelection] {
        &self.members
    }
    #[must_use]
    pub const fn component_count(&self) -> usize {
        self.component_count
    }
    #[must_use]
    pub const fn reference_count(&self) -> usize {
        self.reference_count
    }
    #[must_use]
    pub const fn issue_count(&self) -> usize {
        self.issue_count
    }
}

/// Genuine retained source plus its native parsed and normalized observations.
/// No public construction, deserialization or caller-built syntax admission.
pub struct AdmittedXmlSchema {
    source: Arc<AdmittedPlatformSource>,
    receipt: XmlSchemaReceipt,
    documents: Vec<XmlDocumentIndex>,
    components: Vec<XmlSchemaComponent>,
    references: Vec<XmlSchemaReference>,
    issues: Vec<XmlSchemaIssue>,
}

impl AdmittedXmlSchema {
    #[must_use]
    pub fn source(&self) -> &AdmittedPlatformSource {
        &self.source
    }
    #[must_use]
    pub const fn receipt(&self) -> &XmlSchemaReceipt {
        &self.receipt
    }
    #[must_use]
    pub fn documents(&self) -> &[XmlDocumentIndex] {
        &self.documents
    }
    #[must_use]
    pub fn components(&self) -> &[XmlSchemaComponent] {
        &self.components
    }
    #[must_use]
    pub fn references(&self) -> &[XmlSchemaReference] {
        &self.references
    }
    #[must_use]
    pub fn issues(&self) -> &[XmlSchemaIssue] {
        &self.issues
    }

    /// Reject reuse under a different native source closure, even for equal XSD bytes.
    pub fn validate_source(
        &self,
        source: &AdmittedPlatformSource,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        checkpoint(stop)?;
        let receipt = source.receipt();
        if receipt.source_snapshot_id() != self.receipt.source_snapshot_id
            || receipt.profile_digest() != self.receipt.source_profile_digest
            || receipt.content_manifest_digest() != self.receipt.content_manifest_digest
            || receipt.admission_digest() != self.receipt.admission_digest
        {
            return Err(invalid(
                "schema capability belongs to another native source",
            ));
        }
        for selected in &self.receipt.members {
            checkpoint(stop)?;
            let member = source.raw_member(&selected.path, stop)?;
            require_schema(member.kind(), member.path())?;
            if member.content_digest() != selected.content_digest
                || member.byte_length() != selected.byte_length
            {
                return Err(invalid(
                    "schema capability member no longer matches its source",
                ));
            }
        }
        checkpoint(stop)
    }
}

/// Admit explicitly selected XSDs from one real retained source; never follow locations.
pub fn admit_xml_schema(
    source: &Arc<AdmittedPlatformSource>,
    selection: &XmlSchemaSelection,
    stop: &AtomicBool,
) -> ProjectResult<AdmittedXmlSchema> {
    checkpoint(stop)?;
    let source_receipt = source.receipt();
    if selection.source_snapshot_id.len() > 4096
        || selection.source_snapshot_id != source_receipt.source_snapshot_id()
        || selection.profile_digest != source_receipt.profile_digest()
        || selection.content_manifest_digest != source_receipt.content_manifest_digest()
        || selection.admission_digest != source_receipt.admission_digest()
    {
        return Err(invalid("schema selection belongs to another native source"));
    }
    if selection.members.is_empty() {
        return Err(invalid(
            "schema selection requires at least one explicit member",
        ));
    }
    if selection.members.len() > MAX_DOCUMENTS {
        return Err(exhausted());
    }
    if selection
        .members
        .iter()
        .any(|member| member.path.len() > 4096)
    {
        return Err(exhausted());
    }
    let mut budget = SchemaBudget::new();
    budget.charge(selection, stop)?;
    let mut selected = selection.clone();
    selected.members.sort_by(|a, b| a.path.cmp(&b.path));
    let mut total_bytes = 0;
    let mut previous = None;
    // Validate the complete selection before the first parser or index allocation.
    for member in &selected.members {
        budget.visit(1, stop)?;
        if previous == Some(member.path.as_str()) {
            return Err(invalid("schema selection contains duplicate members"));
        }
        previous = Some(member.path.as_str());
        let native = source.raw_member(&member.path, stop)?;
        require_schema(native.kind(), native.path())?;
        if member.content_digest != native.content_digest()
            || member.byte_length != native.byte_length()
        {
            return Err(invalid("schema selection differs from an admitted member"));
        }
        charge_input(member.byte_length, &mut total_bytes)?;
    }
    budget.charge(
        &(XML_SCHEMA_PROFILE, XML_SCHEMA_POLICY_PROFILE, &selected),
        stop,
    )?;
    let selection_digest = crate::identity::canonical_digest(
        "wow-project/xml-schema-selection/1",
        &(XML_SCHEMA_PROFILE, XML_SCHEMA_POLICY_PROFILE, &selected),
        ProjectPhase::Inventory,
    )?;
    let mut documents = Vec::new();
    let mut events = 0usize;
    let mut elements = 0usize;
    let mut attributes = 0usize;
    for member in &selected.members {
        budget.visit(1, stop)?;
        let native = source.raw_member(&member.path, stop)?;
        let text = std::str::from_utf8(native.bytes()).map_err(|_| {
            invalid("selected schema member is not UTF-8").with_relative_path(member.path.as_str())
        })?;
        // Native parser scratch retains its existing finite byte/count/depth caps.
        // This ledger charges retention, not allocations already made by that parser.
        let (records, index) = xml::parse(&member.path, text, stop)?;
        if index.source_digest() != member.content_digest || index.document() != member.path {
            return Err(invalid(
                "schema syntax does not bind its selected source member",
            ));
        }
        add_count(&mut events, records.len(), MAX_COMPONENTS)?;
        add_count(&mut elements, index.elements().len(), MAX_COMPONENTS)?;
        drop(records);
        // A mislabeled UI document can produce serde-skipped inline payloads.
        // Keep those finite parser allocations scratch rather than retaining them
        // across selected schemas. Genuine XSD indexes have no native UI scripts.
        for element in index.elements() {
            budget.visit(1, stop)?;
            if element.script.is_some() {
                return Err(invalid("selected schema contains native UI script records"));
            }
        }
        budget.charge(&index, stop)?;
        for element in index.elements() {
            budget.visit(1, stop)?;
            add_count(&mut attributes, element.attributes.len(), MAX_ROWS)?;
            for attribute in &element.attributes {
                budget.visit(1, stop)?;
                // XmlAttributeRecord serializes its digest/spans, but skips this value.
                budget.charge(attribute.value(), stop)?;
            }
        }
        documents.push(index);
    }
    let normalized = components::normalize(&documents, selection_digest, &mut budget, stop)?;
    let component_count = normalized.components.len();
    let reference_count = normalized.references.len();
    let issue_count = normalized.issues.len();
    let rows = component_count
        .checked_add(reference_count)
        .and_then(|value| value.checked_add(issue_count))
        .ok_or_else(exhausted)?;
    if component_count > MAX_COMPONENTS || rows > MAX_ROWS {
        return Err(exhausted());
    }
    let content = (&selected, &documents, &normalized);
    let hash_input = (selection_digest, &content);
    budget.charge(&hash_input, stop)?;
    let digest = crate::identity::canonical_digest(
        "wow-project/xml-schema-components/1",
        &hash_input,
        ProjectPhase::Inventory,
    )?;
    budget.charge(&selected.source_snapshot_id, stop)?;
    let receipt = XmlSchemaReceipt {
        profile: XML_SCHEMA_PROFILE,
        policy_profile: XML_SCHEMA_POLICY_PROFILE,
        source_snapshot_id: selected.source_snapshot_id,
        source_profile_digest: selected.profile_digest,
        content_manifest_digest: selected.content_manifest_digest,
        admission_digest: selected.admission_digest,
        selection_digest,
        digest,
        members: selected.members,
        component_count,
        reference_count,
        issue_count,
    };
    budget.charge(&receipt, stop)?;
    checkpoint(stop)?;
    Ok(AdmittedXmlSchema {
        source: Arc::clone(source),
        receipt,
        documents,
        components: normalized.components,
        references: normalized.references,
        issues: normalized.issues,
    })
}

fn require_schema(kind: PlatformFileKind, path: &str) -> ProjectResult<()> {
    if kind != PlatformFileKind::Schema {
        return Err(ProjectError::new(
            ProjectErrorCode::InvalidFileLanguage,
            ProjectPhase::Inventory,
            "explicit schema selection requires an Included Schema member",
        )
        .with_relative_path(path));
    }
    Ok(())
}

fn charge_input(bytes: u64, total: &mut usize) -> ProjectResult<()> {
    let bytes = usize::try_from(bytes).map_err(|_| exhausted())?;
    if bytes > DISK_SOURCE_MAX_BYTES {
        return Err(exhausted());
    }
    add_count(total, bytes, DISK_INVENTORY_MAX_BYTES)
}

fn add_count(total: &mut usize, count: usize, limit: usize) -> ProjectResult<()> {
    *total = total
        .checked_add(count)
        .filter(|value| *value <= limit)
        .ok_or_else(exhausted)?;
    Ok(())
}

pub(super) fn invalid(message: &'static str) -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceRegistryInvalid,
        ProjectPhase::Inventory,
        message,
    )
}

pub(super) fn exhausted() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::Inventory,
        "selected XML schema exceeds its bounded component policy",
    )
}

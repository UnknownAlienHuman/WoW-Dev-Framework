//! Search data copied from the actual Reference compatibility metadata owner.
use super::*;
use wow_reference::compatibility_aliases::{AliasCatalog, AliasReceipt, AliasState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AliasDocument {
    pub record_key: String,
    pub record_digest: String,
    pub source_ids: Vec<String>,
    pub value: String,
    pub target_partition: String,
    pub target_key: String,
    pub target_digest: String,
    pub state: AliasState,
    pub declaration_indices: Vec<usize>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AliasInventory {
    pub receipt: AliasReceipt,
    pub generation: String,
    pub view_digest: String,
    pub records: Vec<AliasDocument>,
}
impl AliasInventory {
    pub(super) fn capture(
        catalog: &AliasCatalog,
        bounds: SearchBounds,
        stop: &AtomicBool,
    ) -> SearchResult<Self> {
        check(stop)?;
        if catalog.records().len() > bounds.max_owner_evidence_records.get() as usize {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        let mut records = Vec::new();
        for record in catalog.records() {
            check(stop)?;
            let data = AliasDocument {
                record_key: record.record_key().into(),
                record_digest: record.record_digest().into(),
                source_ids: record.source_ids().to_vec(),
                value: record.value().into(),
                target_partition: record.target_partition().into(),
                target_key: record.target_key().into(),
                target_digest: record.target_digest().into(),
                state: record.state(),
                declaration_indices: record.declaration_indices().to_vec(),
            };
            if canonical_size(&data)? > bounds.max_owner_manifest_bytes.get() {
                return Err(SearchError::FileSetBudgetExceeded);
            }
            records.push(data);
        }
        let data = Self {
            receipt: catalog.receipt().clone(),
            generation: catalog.generation().into(),
            view_digest: catalog.view_digest().into(),
            records,
        };
        if canonical_size(&data)? > bounds.max_owner_manifest_bytes.get() {
            return Err(SearchError::FileSetBudgetExceeded);
        }
        Ok(data)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct AliasEvidence {
    pub value: String,
    pub record_key: String,
    pub record_digest: String,
    pub source_ids: Vec<String>,
    pub declaration_indices: Vec<usize>,
    pub target_partition: String,
    pub target_key: String,
    pub target_digest: String,
    pub scope: &'static str,
    pub authority_class: &'static str,
}
impl AliasDocument {
    pub(super) fn target_matches(&self, document: &Document) -> bool {
        let part = if document.restriction.is_some() {
            wow_reference::native_view::NATIVE_RESTRICTION_PARTITION
        } else if document.declaration.is_some() {
            NATIVE_DECLARATION_PARTITION
        } else if document.event.is_some() {
            NATIVE_EVENT_PARTITION
        } else {
            NATIVE_API_PARTITION
        };
        self.state == AliasState::Resolved
            && self.target_key == document.entity_key
            && self.target_digest == document.owner_record_digest
            && self.target_partition == part
    }
    pub(super) fn evidence(&self) -> AliasEvidence {
        AliasEvidence {
            value: self.value.clone(),
            record_key: self.record_key.clone(),
            record_digest: self.record_digest.clone(),
            source_ids: self.source_ids.clone(),
            declaration_indices: self.declaration_indices.clone(),
            target_partition: self.target_partition.clone(),
            target_key: self.target_key.clone(),
            target_digest: self.target_digest.clone(),
            scope: "exact_reference_universe",
            authority_class: "configured_compatibility_declaration",
        }
    }
}
#[derive(Serialize)]
pub(super) struct AliasSummary<'a> {
    receipt: &'a AliasReceipt,
    records: usize,
    resolved: usize,
    unresolved: usize,
    conflicted: usize,
    coverage: &'static str,
    kind: &'static str,
}
pub(super) fn summary(inventory: Option<&AliasInventory>) -> Option<AliasSummary<'_>> {
    inventory.map(|data| {
        let mut summary = AliasSummary {
            receipt: &data.receipt,
            records: data.records.len(),
            resolved: 0,
            unresolved: 0,
            conflicted: 0,
            coverage: "partial",
            kind: "configured_owner_compatibility",
        };
        for row in &data.records {
            match row.state {
                AliasState::Resolved => summary.resolved += 1,
                AliasState::AliasConflict | AliasState::TargetConflict => summary.conflicted += 1,
                _ => summary.unresolved += 1,
            }
        }
        summary
    })
}

//! Hard query scope over original, already validated Reference document fields.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Component {
    Native,
    FieldShape,
    Documentation,
    Inventory,
    Compatibility,
}
impl Component {
    pub(super) fn schema(self) -> &'static str {
        match self {
            Self::Native => REQUEST_SCHEMA,
            Self::FieldShape => FIELD_SHAPE_REQUEST_SCHEMA,
            Self::Documentation => DOCUMENTATION_REQUEST_SCHEMA,
            Self::Inventory => INVENTORY_REQUEST_SCHEMA,
            Self::Compatibility => ALIAS_REQUEST_SCHEMA,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Namespace {
    Global,
    Exact { value: String },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Filters {
    pub(super) entity_kinds: Vec<ReferenceRecordKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) callable_namespace: Option<Namespace>,
}
impl Filters {
    pub(super) fn validate(&mut self) -> SearchResult<()> {
        if self.entity_kinds.is_empty() || self.entity_kinds.len() > 7 {
            return Err(SearchError::InvalidProfile);
        }
        self.entity_kinds.sort();
        if self.entity_kinds.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(SearchError::InvalidProfile);
        }
        if let Some(namespace) = &self.callable_namespace {
            if !self.entity_kinds.contains(&ReferenceRecordKind::Api) {
                return Err(SearchError::InvalidProfile);
            }
            if let Namespace::Exact { value } = namespace {
                valid_text(value)?;
            }
        }
        Ok(())
    }
    pub(super) fn accepts(&self, document: &Document) -> bool {
        let kind = document.record_kind();
        self.entity_kinds.contains(&kind)
            && self.callable_namespace.as_ref().is_none_or(|namespace| {
                kind == ReferenceRecordKind::Api
                    && match namespace {
                        Namespace::Global => document.namespace.is_none(),
                        Namespace::Exact { value } => {
                            document.namespace.as_deref() == Some(value.as_str())
                        }
                    }
            })
    }
    pub(super) fn unsupported_kinds(&self, manifest: &Manifest) -> Vec<ReferenceRecordKind> {
        self.entity_kinds
            .iter()
            .copied()
            .filter(|kind| !match kind {
                ReferenceRecordKind::Api => true,
                ReferenceRecordKind::Event => manifest.native_documents.is_some(),
                ReferenceRecordKind::Other
                | ReferenceRecordKind::Enumeration
                | ReferenceRecordKind::Widget => {
                    manifest.native_documents
                        == Some(NativeReferenceDocuments::SourceDeclarationsV1)
                }
                ReferenceRecordKind::Restriction => manifest.include_restrictions == Some(true),
                ReferenceRecordKind::Global => false,
            })
            .collect()
    }
}

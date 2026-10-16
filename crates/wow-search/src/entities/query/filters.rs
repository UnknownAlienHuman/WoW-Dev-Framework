//! Exact requested Graph kind IDs; capability comes from the original registry.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Filters {
    pub(super) entity_kinds: Vec<String>,
}
impl Filters {
    pub(super) fn validate(&mut self) -> SearchResult<()> {
        if self.entity_kinds.is_empty() || self.entity_kinds.len() > 512 {
            return Err(SearchError::InvalidProfile);
        }
        for kind in &self.entity_kinds {
            valid_text(kind)?;
        }
        self.entity_kinds.sort();
        if self.entity_kinds.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(SearchError::InvalidProfile);
        }
        Ok(())
    }
    pub(super) fn accepts_kind(&self, kind: &str) -> bool {
        self.entity_kinds
            .binary_search_by(|value| value.as_str().cmp(kind))
            .is_ok()
    }
}

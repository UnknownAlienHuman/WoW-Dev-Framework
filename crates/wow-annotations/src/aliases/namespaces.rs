//! Exact empty namespace bindings from a separately selected Ketho resource.
use super::{AliasDocument, AliasOutcome, source};
use crate::ketho::{RenderError, identifier};
use crate::native::{ProjectionIssue, SourceMapping};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_reference::native::Span;
use wow_reference::native_aliases::NamespaceFact;

pub(super) struct Namespaces<'a> {
    entries: Vec<(&'a AliasDocument, &'a NamespaceFact)>,
    emitted: Vec<usize>,
    pub outcomes: Vec<AliasOutcome>,
    pub issues: Vec<ProjectionIssue>,
}

impl<'a> Namespaces<'a> {
    pub fn prepare(
        resources: &[&'a AliasDocument],
        counts: &BTreeMap<&str, usize>,
        defined: &BTreeSet<String>,
        reserved: &BTreeSet<String>,
        cancelled: &AtomicBool,
    ) -> Result<Self, RenderError> {
        let entries = resources
            .iter()
            .flat_map(|document| {
                document
                    .namespaces()
                    .iter()
                    .map(move |fact| (*document, fact))
            })
            .collect::<Vec<_>>();
        let mut emitted = Vec::new();
        let mut outcomes = Vec::with_capacity(entries.len());
        let mut issues = Vec::new();
        for (ordinal, (document, fact)) in entries.iter().enumerate() {
            if cancelled.load(Ordering::Relaxed) {
                return Err(RenderError::Cancelled);
            }
            let status = if identifier(&fact.name).is_err() || !fact.name.starts_with("C_") {
                "invalid_namespace_name"
            } else if counts.get(fact.name.as_str()) != Some(&1) {
                "duplicate_namespace"
            } else if reserved.contains(&fact.name) || defined.contains(&fact.name) {
                "source_name_conflict"
            } else {
                emitted.push(ordinal);
                "emitted"
            };
            outcomes.push(AliasOutcome {
                ordinal,
                name: fact.name.clone(),
                status,
            });
            if status != "emitted" {
                issues.push(ProjectionIssue {
                    code: status.into(),
                    source: source(document, fact.span),
                });
            }
        }
        Ok(Self {
            entries,
            emitted,
            outcomes,
            issues,
        })
    }

    pub fn has_output(&self) -> bool {
        !self.emitted.is_empty()
    }

    pub fn append(
        &self,
        text: &mut String,
        mappings: &mut Vec<SourceMapping>,
        cancelled: &AtomicBool,
    ) -> Result<(), RenderError> {
        let mut order = self.emitted.clone();
        order.sort_by_key(|&index| &self.entries[index].1.name);
        for index in order {
            if cancelled.load(Ordering::Relaxed) {
                return Err(RenderError::Cancelled);
            }
            let (document, fact) = self.entries[index];
            let fragment = format!("{} = {{}}", fact.name);
            if text.len().saturating_add(fragment.len()).saturating_add(1)
                > crate::ketho::MAX_OUTPUT_BYTES
            {
                return Err(RenderError::OutputLimit);
            }
            let start = text.len();
            text.push_str(&fragment);
            let end = text.len();
            text.push('\n');
            mappings.push(SourceMapping {
                granularity: "declaration",
                generated: Span { start, end },
                source: source(document, fact.span),
            });
        }
        Ok(())
    }
}

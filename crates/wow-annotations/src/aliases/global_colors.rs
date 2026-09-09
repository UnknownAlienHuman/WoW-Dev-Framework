//! Inert declarations for exact external `CreateColor` assignments.
use super::{AliasDocument, AliasOutcome, source};
use crate::ketho::{RenderError, identifier};
use crate::native::{ProjectionIssue, SourceLink, SourceMapping};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_reference::native::Span;
use wow_reference::native_aliases::GlobalColorFact;

const COLOR_TYPE: &str = "colorRGBA";

pub(super) struct GlobalColors<'a> {
    entries: Vec<(&'a AliasDocument, &'a GlobalColorFact)>,
    emitted: Vec<usize>,
    pub outcomes: Vec<AliasOutcome>,
    pub issues: Vec<ProjectionIssue>,
}

impl<'a> GlobalColors<'a> {
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
                    .global_colors()
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
            let status = if identifier(&fact.name).is_err() {
                "invalid_global_color_name"
            } else if counts.get(fact.name.as_str()) != Some(&1) {
                "duplicate_global_color"
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

    /// The assignment is still useful to consumers when its type is not in the
    /// selected type graph, but that missing dependency remains explicit.
    pub fn unresolved_types(
        &mut self,
        known: &BTreeSet<String>,
        cancelled: &AtomicBool,
    ) -> Result<Vec<SourceLink>, RenderError> {
        if known.contains(COLOR_TYPE) {
            return Ok(Vec::new());
        }
        let mut unresolved = Vec::with_capacity(self.emitted.len());
        for &index in &self.emitted {
            if cancelled.load(Ordering::Relaxed) {
                return Err(RenderError::Cancelled);
            }
            let (document, fact) = self.entries[index];
            let link = source(document, fact.span);
            self.issues.push(ProjectionIssue {
                code: "unresolved_global_color_type".into(),
                source: link.clone(),
            });
            unresolved.push(link);
        }
        Ok(unresolved)
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
            let fragment = format!("---@type {COLOR_TYPE}\n{} = nil", fact.name);
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

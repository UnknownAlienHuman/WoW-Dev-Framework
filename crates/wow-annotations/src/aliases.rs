//! Additive Ketho-style alias projection. External annotations are explicitly
//! selected consumer input, never a correction to Blizzard facts.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_reference::native::Span;
use wow_reference::native_aliases::AliasDocument;

use crate::ketho::{RenderError, Renderer, qualified_identifier};
use crate::native::{ProjectionIssue, SourceLink, SourceMapping};

#[derive(Debug, Serialize)]
pub struct AliasOutcome {
    pub ordinal: usize,
    pub name: String,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AliasReport<'a> {
    pub schema: &'static str,
    pub authority: &'static str,
    pub source: &'a AliasDocument,
    pub outcomes: Vec<AliasOutcome>,
    pub limitations: Vec<&'static str>,
}

pub(crate) struct ProjectedAliases<'a> {
    pub report: AliasReport<'a>,
    pub text: String,
    pub mappings: Vec<SourceMapping>,
    pub issues: Vec<ProjectionIssue>,
}

fn primitive(name: &str) -> bool {
    matches!(
        name,
        "nil"
            | "boolean"
            | "integer"
            | "number"
            | "string"
            | "table"
            | "function"
            | "userdata"
            | "thread"
            | "any"
            | "unknown"
    )
}
fn reserved_name(name: &str, reserved: &BTreeSet<String>) -> bool {
    let mut current = name;
    loop {
        if reserved.contains(current) || crate::ketho::reserved_type_name(current) {
            return true;
        }
        let Some((parent, _)) = current.rsplit_once('.') else {
            return false;
        };
        current = parent;
    }
}
fn source(document: &AliasDocument, span: Span) -> SourceLink {
    SourceLink {
        scope: Some("annotation_alias_catalog"),
        path: document.path().into(),
        sha256: document.sha256().into(),
        span,
    }
}

fn string_union(values: &[String]) -> Option<String> {
    if values.is_empty()
        || values.len() > 256
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
        || values.iter().any(|value| {
            value.len() > 128
                || !value
                    .bytes()
                    .all(|b| (b' '..=b'~').contains(&b) && b != b'"' && b != b'\\')
        })
    {
        return None;
    }
    Some(values.iter().map(|value| format!("\"{value}\"")).collect::<Vec<_>>().join("|"))
}

/// Only types actually emitted by the native lane can satisfy an external alias
/// dependency. Reserved but rejected source declarations cannot lend authority.
/// The dependency graph is processed without recursion and without expanding
/// aliases into signatures, so a shared dependency cannot cause exponential output.
pub(crate) fn project<'a>(
    document: &'a AliasDocument,
    defined: &BTreeSet<String>,
    reserved: &BTreeSet<String>,
    cancelled: &AtomicBool,
) -> Result<ProjectedAliases<'a>, RenderError> {
    let renderer = Renderer::new(BTreeSet::new(), 8 * 1024 * 1024)?;
    let facts = document.aliases();
    let extended = facts.iter().any(|fact| fact.string_values.is_some());
    let mut counts = BTreeMap::<&str, usize>::new();
    for fact in facts {
        *counts.entry(&fact.name).or_default() += 1;
    }
    let mut states = vec![None; facts.len()];
    let mut lowered = vec![None; facts.len()];
    let mut candidates = BTreeMap::new();
    for (index, fact) in facts.iter().enumerate() {
        if cancelled.load(Ordering::Relaxed) {
            return Err(RenderError::Cancelled);
        }
        let status = if qualified_identifier(&fact.name).is_err()
            || crate::ketho::reserved_type_name(&fact.name)
        {
            Some("invalid_alias_name")
        } else if counts[fact.name.as_str()] != 1 {
            Some("duplicate_alias")
        } else if reserved_name(&fact.name, reserved) || defined.contains(&fact.name) {
            Some("source_name_conflict")
        } else if fact.syntax_error {
            Some("alias_syntax_error")
        } else if let Some(values) = &fact.string_values {
            match string_union(values).filter(|_| fact.terms.is_none()) {
                Some(ty) => {
                    lowered[index] = Some(ty);
                    None
                }
                None => Some("unsupported_alias_type"),
            }
        } else if let Some(terms) = &fact.terms {
            match renderer.lower_type(&terms.join("|")) {
                Ok(ty) => {
                    lowered[index] = Some(ty);
                    None
                }
                Err(_) => Some("unsupported_alias_type"),
            }
        } else {
            Some("unsupported_alias_type")
        };
        states[index] = status;
        if status.is_none() {
            candidates.insert(fact.name.as_str(), index);
        }
    }
    let mut indegree = vec![0usize; facts.len()];
    let mut dependents = vec![Vec::new(); facts.len()];
    let mut ready = BTreeSet::new();
    for (&name, &index) in &candidates {
        // Literal values are not type names, even when they contain a pipe.
        if facts[index].string_values.is_none()
            && let Some(ty) = &lowered[index]
        {
            for target in ty.split('|') {
                if primitive(target) || defined.contains(target) {
                    continue;
                }
                if let Some(&dependency) = candidates.get(target) {
                    indegree[index] += 1;
                    dependents[dependency].push(index);
                } else {
                    states[index] = Some("unresolved_alias_target");
                }
            }
        }
        if indegree[index] == 0 && states[index].is_none() {
            ready.insert((name, index));
        }
    }
    while let Some((_, index)) = ready.pop_first() {
        if cancelled.load(Ordering::Relaxed) {
            return Err(RenderError::Cancelled);
        }
        states[index] = Some("emitted");
        for &dependent in &dependents[index] {
            indegree[dependent] -= 1;
            if indegree[dependent] == 0 && states[dependent].is_none() {
                ready.insert((facts[dependent].name.as_str(), dependent));
            }
        }
    }
    let mut outcomes = Vec::new();
    let mut issues = Vec::new();
    let mut emitted = Vec::new();
    for (index, fact) in facts.iter().enumerate() {
        let status = states[index].unwrap_or("unresolved_or_cyclic_alias_dependency");
        outcomes.push(AliasOutcome {
            ordinal: index,
            name: fact.name.clone(),
            status,
        });
        if status == "emitted" {
            emitted.push(index);
        } else {
            issues.push(ProjectionIssue {
                code: status.into(),
                source: source(document, fact.span),
            });
        }
    }
    emitted.sort_by_key(|&index| &facts[index].name);
    let mut text = String::new();
    let mut mappings = Vec::new();
    if !emitted.is_empty() {
        text.push_str("---@meta _\n-- Explicit external annotation overlay; not Blizzard reference evidence.\n");
        // Retain the port donor's license with generated derivative annotations.
        // This static, framework-owned text cannot become source directives.
        for line in include_str!("../THIRD_PARTY_NOTICES.md").lines() {
            text.push_str("-- ");
            text.push_str(line);
            text.push('\n');
        }
        text.push('\n');
        for index in emitted {
            let start = text.len();
            let ty = lowered[index]
                .as_deref()
                .ok_or(RenderError::InvalidSource)?;
            text.push_str(&format!("---@alias {} {ty}\n", facts[index].name));
            mappings.push(SourceMapping {
                granularity: "declaration",
                generated: Span {
                    start,
                    end: text.len() - 1,
                },
                source: source(document, facts[index].span),
            });
        }
    }
    let mut limitations = vec![
        "explicit consumer overlay; not source-confirmed Blizzard types or runtime safety",
        "only named/primitive unions; unsupported declarations and dependencies remain explicit",
        "description and non-directive comments are preserved in the raw resource, not rendered",
        "no automatic discovery, source correction, widget inheritance or language-server certification",
    ];
    if extended {
        limitations[1] = "named/primitive unions and closed printable-ASCII string enums; no mixed/open literal unions or escape decoding";
    }
    Ok(ProjectedAliases {
        report: AliasReport {
            schema: if extended {
                "wow-native-alias-projection/2"
            } else {
                "wow-native-alias-projection/1"
            },
            authority: "external_annotation_overlay",
            source: document,
            outcomes,
            limitations,
        },
        text,
        mappings,
        issues,
    })
}

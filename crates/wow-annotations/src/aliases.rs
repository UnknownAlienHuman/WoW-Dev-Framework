//! Additive Ketho-style alias projection. External annotations are explicitly
//! selected consumer input, never a correction to Blizzard facts.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_reference::native::Span;
use wow_reference::native_aliases::AliasDocument;

use crate::ketho::{RenderError, Renderer, qualified_identifier};
use crate::native::{ProjectionIssue, SourceLink, SourceMapping};
mod function_containers;
mod global_colors;
mod namespaces;
mod structures;

/// Bounds apply to the whole selected resource generation, not each file alone.
pub const MAX_CATALOG_FILES: usize = 32;
pub const MAX_CATALOG_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_CATALOG_ALIASES: usize = 4096;
const MAX_STRING_VALUES: usize = 512;

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
    /// First resource in canonical path order. Single-resource wire bytes are unchanged.
    pub source: &'a AliasDocument,
    /// Further resources from the same external revision, in canonical path order.
    /// In v3, outcome ordinals address the concatenated source alias lists.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub additional_sources: Vec<&'a AliasDocument>,
    pub outcomes: Vec<AliasOutcome>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub structure_outcomes: Vec<AliasOutcome>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unresolved_structure_fields: Vec<SourceLink>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub namespace_outcomes: Vec<AliasOutcome>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub function_container_outcomes: Vec<AliasOutcome>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unresolved_function_container_returns: Vec<SourceLink>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub global_color_outcomes: Vec<AliasOutcome>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unresolved_global_color_types: Vec<SourceLink>,
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
        || values.len() > MAX_STRING_VALUES
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
    Some(
        values
            .iter()
            .map(|value| format!("\"{value}\""))
            .collect::<Vec<_>>()
            .join("|"),
    )
}

/// Only types actually emitted by the native lane can satisfy an external alias
/// dependency. Reserved but rejected source declarations cannot lend authority.
/// The dependency graph is processed without recursion and without expanding
/// aliases into signatures, so a shared dependency cannot cause exponential output.
pub(crate) fn project<'a>(
    catalogs: &[&'a AliasDocument],
    defined: &BTreeSet<String>,
    reserved: &BTreeSet<String>,
    cancelled: &AtomicBool,
) -> Result<ProjectedAliases<'a>, RenderError> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(RenderError::Cancelled);
    }
    if catalogs.is_empty() || catalogs.len() > MAX_CATALOG_FILES {
        return Err(RenderError::InputLimit);
    }
    let mut sources = catalogs.to_vec();
    sources.sort_by_key(|source| source.path());
    let document = sources[0];
    if sources
        .windows(2)
        .any(|pair| pair[0].path() == pair[1].path())
        || sources
            .iter()
            .any(|source| source.revision() != document.revision())
    {
        return Err(RenderError::InvalidSource);
    }
    if sources.iter().map(|s| s.text().len()).sum::<usize>() > MAX_CATALOG_BYTES
        || sources
            .iter()
            .map(|s| {
                s.aliases().len()
                    + s.structures().len()
                    + s.namespaces().len()
                    + s.function_containers()
                        .iter()
                        .map(|container| 1 + container.methods.len())
                        .sum::<usize>()
                    + s.global_colors().len()
            })
            .sum::<usize>()
            > MAX_CATALOG_ALIASES
    {
        return Err(RenderError::InputLimit);
    }
    // Resolve one graph across all files. Per-file rendering would incorrectly
    // reject forward references and miss duplicates/cycles across resources.
    let entries = sources
        .iter()
        .flat_map(|source| source.aliases().iter().map(move |fact| (*source, fact)))
        .collect::<Vec<_>>();
    let facts = entries.iter().map(|(_, fact)| *fact).collect::<Vec<_>>();
    let renderer = Renderer::new(BTreeSet::new(), 8 * 1024 * 1024)?;
    let extended = facts.iter().any(|fact| fact.string_values.is_some());
    let open = facts.iter().any(|fact| fact.string_base.is_some());
    let mut counts = BTreeMap::<&str, usize>::new();
    for fact in &facts {
        *counts.entry(&fact.name).or_default() += 1;
    }
    for resource in &sources {
        for fact in resource.structures() {
            *counts.entry(&fact.name).or_default() += 1;
        }
        for fact in resource.namespaces() {
            *counts.entry(&fact.name).or_default() += 1;
        }
        for fact in resource.function_containers() {
            *counts.entry(&fact.name).or_default() += 1;
        }
        for fact in resource.global_colors() {
            *counts.entry(&fact.name).or_default() += 1;
        }
    }
    let mut namespaces =
        namespaces::Namespaces::prepare(&sources, &counts, defined, reserved, cancelled)?;
    let mut structures = structures::Structures::prepare(
        &sources, &renderer, &counts, defined, reserved, cancelled,
    )?;
    let mut function_containers = function_containers::FunctionContainers::prepare(
        &sources, &renderer, &counts, defined, reserved, cancelled,
    )?;
    let mut global_colors =
        global_colors::GlobalColors::prepare(&sources, &counts, defined, reserved, cancelled)?;
    let mut known = defined.clone();
    known.extend(structures.defined.iter().cloned());
    known.extend(function_containers.defined.iter().cloned());
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
            match string_union(values).filter(|_| {
                fact.terms.is_none() && matches!(fact.string_base, None | Some("string"))
            }) {
                Some(ty) => {
                    lowered[index] = Some(if fact.string_base.is_some() {
                        format!("string|{ty}")
                    } else {
                        ty
                    });
                    None
                }
                None => Some("unsupported_alias_type"),
            }
        } else if fact.string_base.is_some() {
            Some("unsupported_alias_type")
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
                if primitive(target) || known.contains(target) {
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
                source: source(entries[index].0, fact.span),
            });
        }
    }
    known.extend(emitted.iter().map(|&index| facts[index].name.clone()));
    let unresolved_structure_fields = structures.unresolved_fields(&renderer, &known, cancelled)?;
    let unresolved_function_container_returns =
        function_containers.unresolved_returns(&renderer, &known, cancelled)?;
    let unresolved_global_color_types = global_colors.unresolved_types(&known, cancelled)?;
    emitted.sort_by_key(|&index| &facts[index].name);
    let mut text = String::new();
    let mut mappings = Vec::new();
    if !emitted.is_empty()
        || structures.has_output()
        || namespaces.has_output()
        || function_containers.has_output()
        || global_colors.has_output()
    {
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
                source: source(entries[index].0, facts[index].span),
            });
        }
    }
    namespaces.append(&mut text, &mut mappings, cancelled)?;
    function_containers.append(&mut text, &mut mappings, cancelled)?;
    global_colors.append(&mut text, &mut mappings, cancelled)?;
    structures.append(&mut text, &mut mappings, cancelled)?;
    issues.append(&mut structures.issues);
    issues.append(&mut namespaces.issues);
    issues.append(&mut function_containers.issues);
    issues.append(&mut global_colors.issues);
    let has_structures = sources.iter().any(|source| !source.structures().is_empty());
    let has_namespaces = sources.iter().any(|source| !source.namespaces().is_empty());
    let has_function_containers = sources
        .iter()
        .any(|source| !source.function_containers().is_empty());
    let has_global_colors = sources
        .iter()
        .any(|source| !source.global_colors().is_empty());
    let mut limitations = vec![
        "explicit consumer overlay; not source-confirmed Blizzard types or runtime safety",
        "only named/primitive unions; unsupported declarations and dependencies remain explicit",
        "description and non-directive comments are preserved in the raw resource, not rendered",
        "no automatic discovery, source correction, widget inheritance or language-server certification",
    ];
    if open {
        limitations[1] = "named/primitive unions and printable-ASCII string enums; explicit open string bases retain completion hints, not whitelists; no other mixed unions or escape decoding";
    } else if extended {
        limitations[1] = "named/primitive unions and closed printable-ASCII string enums; no mixed/open literal unions or escape decoding";
    }
    if has_structures {
        limitations.push("external classes retain declaration and ordered field maps; no inheritance, methods, generic/indexer or runtime claims");
    }
    if has_namespaces {
        limitations.push("external namespaces are explicit empty table bindings from a standalone resource; no members, absence or runtime availability are inferred");
    }
    if has_function_containers {
        limitations.push("external function containers retain exact class and method maps; empty bodies are syntax-only declarations and do not imply runtime behavior");
    }
    if has_global_colors {
        limitations.push("external CreateColor components are retained as source lexemes; output declares inert colorRGBA globals and does not execute calls or claim exact runtime values");
    }
    Ok(ProjectedAliases {
        report: AliasReport {
            schema: if has_global_colors {
                "wow-native-alias-projection/9"
            } else if has_function_containers {
                "wow-native-alias-projection/8"
            } else if has_namespaces {
                "wow-native-alias-projection/7"
            } else if has_structures {
                "wow-native-alias-projection/6"
            } else if open {
                "wow-native-alias-projection/4"
            } else if sources.len() > 1 {
                "wow-native-alias-projection/3"
            } else if extended {
                "wow-native-alias-projection/2"
            } else {
                "wow-native-alias-projection/1"
            },
            authority: "external_annotation_overlay",
            source: document,
            additional_sources: sources[1..].to_vec(),
            outcomes,
            structure_outcomes: structures.outcomes,
            unresolved_structure_fields,
            namespace_outcomes: namespaces.outcomes,
            function_container_outcomes: function_containers.outcomes,
            unresolved_function_container_returns,
            global_color_outcomes: global_colors.outcomes,
            unresolved_global_color_types,
            limitations,
        },
        text,
        mappings,
        issues,
    })
}

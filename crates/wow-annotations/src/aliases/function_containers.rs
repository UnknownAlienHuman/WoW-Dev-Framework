//! Static external classes with empty colon-method stubs and explicit returns.
use super::{AliasDocument, AliasOutcome, primitive, reserved_name, source};
use crate::ketho::{RenderError, Renderer, identifier};
use crate::native::{ProjectionIssue, SourceLink, SourceMapping};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_reference::native::Span;
use wow_reference::native_aliases::FunctionContainerFact;

struct RenderedContainer {
    text: String,
    header: Span,
    methods: Vec<Span>,
}

pub(super) struct FunctionContainers<'a> {
    entries: Vec<(&'a AliasDocument, &'a FunctionContainerFact)>,
    rendered: Vec<Option<RenderedContainer>>,
    pub outcomes: Vec<AliasOutcome>,
    pub defined: BTreeSet<String>,
    pub issues: Vec<ProjectionIssue>,
}

impl<'a> FunctionContainers<'a> {
    pub fn prepare(
        resources: &[&'a AliasDocument],
        renderer: &Renderer,
        counts: &BTreeMap<&str, usize>,
        defined: &BTreeSet<String>,
        reserved: &BTreeSet<String>,
        cancelled: &AtomicBool,
    ) -> Result<Self, RenderError> {
        let entries = resources
            .iter()
            .flat_map(|document| {
                document
                    .function_containers()
                    .iter()
                    .map(move |fact| (*document, fact))
            })
            .collect::<Vec<_>>();
        if entries
            .iter()
            .map(|(_, container)| container.methods.len())
            .sum::<usize>()
            > 65_536
        {
            return Err(RenderError::InputLimit);
        }
        let mut result = Self {
            rendered: Vec::with_capacity(entries.len()),
            outcomes: Vec::with_capacity(entries.len()),
            defined: BTreeSet::new(),
            issues: Vec::new(),
            entries,
        };
        for (ordinal, (document, fact)) in result.entries.iter().enumerate() {
            if cancelled.load(Ordering::Relaxed) {
                return Err(RenderError::Cancelled);
            }
            let mut rendered = None;
            let status = if identifier(&fact.name).is_err()
                || crate::ketho::reserved_type_name(&fact.name)
            {
                "invalid_function_container_name"
            } else if counts.get(fact.name.as_str()) != Some(&1) {
                "duplicate_function_container"
            } else if reserved_name(&fact.name, reserved) || defined.contains(&fact.name) {
                "source_name_conflict"
            } else {
                match declaration(fact, renderer) {
                    Ok(text) => {
                        rendered = Some(text);
                        result.defined.insert(fact.name.clone());
                        "emitted"
                    }
                    Err(error @ (RenderError::OutputLimit | RenderError::InputLimit)) => {
                        return Err(error);
                    }
                    Err(_) => "unsupported_function_container",
                }
            };
            result.outcomes.push(AliasOutcome {
                ordinal,
                name: fact.name.clone(),
                status,
            });
            if status != "emitted" {
                result.issues.push(ProjectionIssue {
                    code: status.into(),
                    source: source(document, fact.span),
                });
            }
            result.rendered.push(rendered);
        }
        Ok(result)
    }

    pub fn has_output(&self) -> bool {
        !self.defined.is_empty()
    }

    pub fn unresolved_returns(
        &mut self,
        renderer: &Renderer,
        known: &BTreeSet<String>,
        cancelled: &AtomicBool,
    ) -> Result<Vec<SourceLink>, RenderError> {
        let mut unresolved = Vec::new();
        for (index, (document, fact)) in self.entries.iter().enumerate() {
            if self.rendered[index].is_none() {
                continue;
            }
            for method in &fact.methods {
                for returned in &method.returns {
                    if cancelled.load(Ordering::Relaxed) {
                        return Err(RenderError::Cancelled);
                    }
                    let lowered = renderer.lower_type(&returned.terms.join("|"))?;
                    if lowered
                        .split('|')
                        .any(|name| !primitive(name) && !known.contains(name))
                    {
                        let link = source(document, returned.span);
                        self.issues.push(ProjectionIssue {
                            code: "unresolved_function_container_return_type".into(),
                            source: link.clone(),
                        });
                        unresolved.push(link);
                    }
                }
            }
        }
        Ok(unresolved)
    }

    pub fn append(
        &self,
        text: &mut String,
        mappings: &mut Vec<SourceMapping>,
        cancelled: &AtomicBool,
    ) -> Result<(), RenderError> {
        let mut order = (0..self.entries.len()).collect::<Vec<_>>();
        order.sort_by_key(|&index| &self.entries[index].1.name);
        for index in order {
            if cancelled.load(Ordering::Relaxed) {
                return Err(RenderError::Cancelled);
            }
            let Some(fragment) = &self.rendered[index] else {
                continue;
            };
            if text
                .len()
                .saturating_add(fragment.text.len())
                .saturating_add(1)
                > crate::ketho::MAX_OUTPUT_BYTES
            {
                return Err(RenderError::OutputLimit);
            }
            let start = text.len();
            text.push_str(&fragment.text);
            text.push('\n');
            let (document, fact) = self.entries[index];
            mappings.push(SourceMapping {
                granularity: "declaration",
                generated: Span {
                    start: start + fragment.header.start,
                    end: start + fragment.header.end,
                },
                source: source(document, fact.header_span),
            });
            for (span, method) in fragment.methods.iter().zip(&fact.methods) {
                mappings.push(SourceMapping {
                    granularity: "declaration",
                    generated: Span {
                        start: start + span.start,
                        end: start + span.end,
                    },
                    source: source(document, method.span),
                });
            }
        }
        Ok(())
    }
}

fn declaration(
    fact: &FunctionContainerFact,
    renderer: &Renderer,
) -> Result<RenderedContainer, RenderError> {
    let mut names = BTreeSet::new();
    let mut text = format!("---@class {}\nlocal {} = {{}}", fact.name, fact.name);
    let header = Span {
        start: 0,
        end: text.len(),
    };
    let mut methods = Vec::with_capacity(fact.methods.len());
    for method in &fact.methods {
        identifier(&method.name)?;
        if !names.insert(method.name.as_str()) || method.returns.len() > 16 {
            return Err(RenderError::DuplicateName);
        }
        text.push_str("\n\n");
        let start = text.len();
        for returned in &method.returns {
            let lowered = renderer.lower_type(&returned.terms.join("|"))?;
            text.push_str("---@return ");
            text.push_str(&lowered);
            text.push('\n');
        }
        text.push_str("function ");
        text.push_str(&fact.name);
        text.push(':');
        text.push_str(&method.name);
        text.push_str("() end");
        methods.push(Span {
            start,
            end: text.len(),
        });
    }
    if text.len() > crate::ketho::MAX_OUTPUT_BYTES {
        return Err(RenderError::OutputLimit);
    }
    Ok(RenderedContainer {
        text,
        header,
        methods,
    })
}

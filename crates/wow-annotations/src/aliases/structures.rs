//! External structures use the existing Ketho emitter, never raw resource text.
use super::{AliasDocument, AliasOutcome, primitive, reserved_name, source};
use crate::ketho::{
    Field, MemberPosition, Owner, RenderError, Renderer, System, Table, identifier,
};
use crate::native::{ProjectionIssue, SourceLink, SourceMapping};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_reference::native::Span;
use wow_reference::native_aliases::StructureFact;

struct RenderedStructure {
    text: String,
    fields: Vec<Span>,
}

pub(super) struct Structures<'a> {
    entries: Vec<(&'a AliasDocument, &'a StructureFact)>,
    rendered: Vec<Option<RenderedStructure>>,
    pub outcomes: Vec<AliasOutcome>,
    pub defined: BTreeSet<String>,
    pub issues: Vec<ProjectionIssue>,
}

impl<'a> Structures<'a> {
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
            .flat_map(|document| document.structures().iter().map(move |s| (*document, s)))
            .collect::<Vec<_>>();
        if entries.iter().map(|(_, s)| s.fields.len()).sum::<usize>() > 65_536 {
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
                "invalid_structure_name"
            } else if counts.get(fact.name.as_str()) != Some(&1) {
                "duplicate_structure"
            } else if reserved_name(&fact.name, reserved) || defined.contains(&fact.name) {
                "source_name_conflict"
            } else if fact.syntax_error {
                "structure_syntax_error"
            } else if !fact.header_supported {
                "unsupported_structure"
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
                    Err(_) => "unsupported_structure",
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

    /// Unknown named field types remain present, but explicitly keep the overall
    /// overlay partial. Do not replace them by `any`, drop fields, or claim that
    /// a structurally representable class proves full named-type closure.
    pub fn unresolved_fields(
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
            for field in &fact.fields {
                if cancelled.load(Ordering::Relaxed) {
                    return Err(RenderError::Cancelled);
                }
                let ty = field
                    .field_type
                    .as_ref()
                    .ok_or(RenderError::InvalidSource)?;
                let lowered = renderer.lower_type(&ty.terms.join("|"))?;
                if lowered
                    .split('|')
                    .any(|name| !primitive(name) && !known.contains(name))
                {
                    let link = source(document, field.span);
                    self.issues.push(ProjectionIssue {
                        code: "unresolved_structure_field_type".into(),
                        source: link.clone(),
                    });
                    unresolved.push(link);
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
        let mut field_maps = Vec::new();
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
            let end = text.len();
            text.push('\n');
            let (document, fact) = self.entries[index];
            mappings.push(SourceMapping {
                granularity: "declaration",
                generated: Span { start, end },
                source: source(document, fact.span),
            });
            for (span, field) in fragment.fields.iter().zip(&fact.fields) {
                if cancelled.load(Ordering::Relaxed) {
                    return Err(RenderError::Cancelled);
                }
                field_maps.push(SourceMapping {
                    granularity: "field",
                    generated: Span {
                        start: start + span.start,
                        end: start + span.end,
                    },
                    source: source(document, field.span),
                });
            }
        }
        // Keep all declaration maps before their ordered member maps, matching
        // the native field-map profile and the independent artifact verifier.
        mappings.extend(field_maps);
        Ok(())
    }
}

fn declaration(
    fact: &StructureFact,
    renderer: &Renderer,
) -> Result<RenderedStructure, RenderError> {
    let fields = fact
        .fields
        .iter()
        .map(|field| {
            let name = field.name.as_ref().ok_or(RenderError::UnsupportedType)?;
            let ty = field
                .field_type
                .as_ref()
                .ok_or(RenderError::UnsupportedType)?;
            let type_name = ty.terms.join("|");
            Ok(Field {
                name: name.clone(),
                type_name: if ty.array {
                    "table".into()
                } else {
                    type_name.clone()
                },
                inner_type: ty.array.then_some(type_name),
                nilable: ty.nullable,
                default_text: None,
                variadic: false,
            })
        })
        .collect::<Result<Vec<_>, RenderError>>()?;
    let rendered = renderer.render_mapped(&System {
        owner: Owner::Global,
        functions: Vec::new(),
        tables: vec![Table::Structure {
            name: fact.name.clone(),
            fields,
        }],
    })?;
    let declaration = rendered
        .declarations
        .first()
        .ok_or(RenderError::InvalidSource)?;
    let text = rendered
        .text
        .get(declaration.start..declaration.end)
        .ok_or(RenderError::InvalidSource)?;
    if rendered.declarations.len() != 1 || declaration.members.len() != fact.fields.len() {
        return Err(RenderError::InvalidSource);
    }
    let mut fields = Vec::with_capacity(fact.fields.len());
    let mut previous = declaration.start;
    for (index, member) in declaration.members.iter().enumerate() {
        if member.position != MemberPosition::Field
            || member.index != index
            || member.start < previous
            || member.start >= member.end
            || member.end > declaration.end
            || rendered.text.get(member.start..member.end).is_none()
        {
            return Err(RenderError::InvalidSource);
        }
        fields.push(Span {
            start: member.start - declaration.start,
            end: member.end - declaration.start,
        });
        previous = member.end;
    }
    Ok(RenderedStructure {
        text: text.to_owned(),
        fields,
    })
}

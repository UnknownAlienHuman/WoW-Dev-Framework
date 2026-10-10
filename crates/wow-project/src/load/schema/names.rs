//! Namespace interpretation over the existing native syntax index.
use std::sync::atomic::AtomicBool;

use crate::{
    ProjectResult,
    load::{XmlAttributeRecord, XmlElementRecord},
};

use super::{
    SchemaBudget, invalid,
    model::{XmlExpandedName, XmlSchemaNamespaceBinding, XmlSchemaQName, XmlSchemaQNameState},
};

pub(super) const XSD: &str = "http://www.w3.org/2001/XMLSchema";
pub(super) const XML: &str = "http://www.w3.org/XML/1998/namespace";
pub(super) const XMLNS: &str = "http://www.w3.org/2000/xmlns/";

#[derive(Clone, Copy)]
struct Binding<'a> {
    prefix: Option<&'a str>,
    attribute: &'a XmlAttributeRecord,
    occurrence: &'a str,
}

struct Frame<'a> {
    record: &'a XmlElementRecord,
    bindings: Vec<Binding<'a>>,
}

pub(super) struct Names<'a> {
    document: &'a str,
    frames: Vec<Frame<'a>>,
    previous_start: Option<u64>,
}

#[derive(Clone, Copy)]
pub(super) struct ResolvedName<'a> {
    pub(super) namespace: Option<&'a str>,
    pub(super) local: &'a str,
    binding: Option<Binding<'a>>,
}

impl ResolvedName<'_> {
    pub(super) fn own(
        self,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<XmlExpandedName> {
        budget.charge(&(self.namespace, self.local), stop)?;
        Ok(XmlExpandedName {
            namespace: self.namespace.map(str::to_owned),
            local_name: self.local.to_owned(),
        })
    }
}

impl<'a> Names<'a> {
    pub(super) fn new(document: &'a str) -> Self {
        Self {
            document,
            frames: Vec::new(),
            previous_start: None,
        }
    }

    pub(super) fn enter(
        &mut self,
        record: &'a XmlElementRecord,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        budget.visit(1, stop)?;
        if self
            .previous_start
            .is_some_and(|start| start >= record.span.byte_start)
        {
            return Err(invalid("schema occurrences are not in native source order"));
        }
        let parent = record.parent_occurrence_id.as_deref();
        while self
            .frames
            .last()
            .is_some_and(|frame| Some(frame.record.occurrence_id.as_str()) != parent)
        {
            budget.visit(1, stop)?;
            self.frames.pop();
        }
        if parent.is_some() && self.frames.is_empty() {
            return Err(invalid("schema occurrence has no native parent"));
        }
        if let Some(frame) = self.frames.last()
            && (record.span.byte_start < frame.record.start_tag_span.byte_end
                || record.span.byte_end > frame.record.span.byte_end)
        {
            return Err(invalid("schema occurrence escapes its native parent span"));
        }
        if self.frames.is_empty() && self.previous_start.is_some() {
            return Err(invalid("schema document contains multiple roots"));
        }
        if self.frames.len() >= 64 {
            return Err(super::exhausted());
        }
        let mut bindings = Vec::new();
        for attribute in &record.attributes {
            budget.visit(1, stop)?;
            let prefix = if attribute.qualified_name == "xmlns" {
                None
            } else if let Some(prefix) = attribute.qualified_name.strip_prefix("xmlns:") {
                if !ncname(prefix, budget, stop)? {
                    return Err(invalid("schema namespace prefix is malformed"));
                }
                Some(prefix)
            } else {
                continue;
            };
            let uri = attribute.value();
            if uri == XMLNS
                || (uri == XML && prefix != Some("xml"))
                || prefix == Some("xmlns")
                || (prefix == Some("xml") && uri != XML)
                || (prefix.is_some() && uri.is_empty())
            {
                return Err(invalid("schema namespace declaration is invalid"));
            }
            for character in uri.chars() {
                budget.visit(1, stop)?;
                if xml_space(character) {
                    return Err(invalid("schema namespace contains whitespace"));
                }
            }
            budget.charge(
                &(
                    prefix,
                    uri,
                    &record.occurrence_id,
                    &attribute.span,
                    &attribute.value_span,
                ),
                stop,
            )?;
            bindings.push(Binding {
                prefix,
                attribute,
                occurrence: &record.occurrence_id,
            });
        }
        budget.charge(&(&record.occurrence_id, &record.span), stop)?;
        self.frames.push(Frame { record, bindings });
        self.previous_start = Some(record.span.byte_start);
        Ok(())
    }

    pub(super) fn element(
        &self,
        record: &'a XmlElementRecord,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<ResolvedName<'a>> {
        let (resolved, _) = self.resolve(&record.qualified_name, true, budget, stop)?;
        let resolved =
            resolved.ok_or_else(|| invalid("schema element QName is invalid or unbound"))?;
        if resolved.namespace == Some(XMLNS) {
            return Err(invalid("schema element uses the reserved xmlns namespace"));
        }
        Ok(resolved)
    }

    pub(super) fn attribute(
        &self,
        attribute: &'a XmlAttributeRecord,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<ResolvedName<'a>> {
        let spelling = attribute.qualified_name.as_str();
        if spelling == "xmlns" {
            return Ok(ResolvedName {
                namespace: Some(XMLNS),
                local: "xmlns",
                binding: None,
            });
        }
        if let Some(local) = spelling.strip_prefix("xmlns:") {
            if !ncname(local, budget, stop)? {
                return Err(invalid("schema namespace attribute is malformed"));
            }
            return Ok(ResolvedName {
                namespace: Some(XMLNS),
                local,
                binding: None,
            });
        }
        let (resolved, _) = self.resolve(spelling, false, budget, stop)?;
        resolved.ok_or_else(|| invalid("schema attribute QName is invalid or unbound"))
    }

    pub(super) fn qname(
        &self,
        lexical: &'a str,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<XmlSchemaQName> {
        let token = lexical.trim_matches(xml_space);
        let (resolved, state) = self.resolve(token, true, budget, stop)?;
        budget.charge(&lexical, stop)?;
        let (name, binding) = if let Some(resolved) = resolved {
            let name = resolved.own(budget, stop)?;
            let binding = if let Some(binding) = resolved.binding {
                let attribute = binding.attribute;
                budget.charge(
                    &(
                        self.document,
                        binding.occurrence,
                        binding.prefix,
                        attribute.value(),
                        &attribute.span,
                        &attribute.value_span,
                        attribute.decoded_value_digest,
                    ),
                    stop,
                )?;
                Some(Box::new(XmlSchemaNamespaceBinding {
                    document: self.document.to_owned(),
                    occurrence: binding.occurrence.to_owned(),
                    prefix: binding.prefix.map(str::to_owned),
                    namespace: attribute.value().to_owned(),
                    span: attribute.span.clone(),
                    value_span: attribute.value_span.clone(),
                    decoded_value_digest: attribute.decoded_value_digest,
                }))
            } else {
                None
            };
            (Some(name), binding)
        } else {
            (None, None)
        };
        Ok(XmlSchemaQName {
            lexical: lexical.to_owned(),
            name,
            binding,
            state,
        })
    }

    fn resolve(
        &self,
        name: &'a str,
        use_default: bool,
        budget: &mut SchemaBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<(Option<ResolvedName<'a>>, XmlSchemaQNameState)> {
        budget.visit(1, stop)?;
        let (prefix, local) = match name.split_once(':') {
            Some((prefix, local)) => {
                if !ncname(prefix, budget, stop)? || !ncname(local, budget, stop)? {
                    return Ok((None, XmlSchemaQNameState::Invalid));
                }
                (Some(prefix), local)
            }
            None => {
                if !ncname(name, budget, stop)? {
                    return Ok((None, XmlSchemaQNameState::Invalid));
                }
                (None, name)
            }
        };
        if prefix == Some("xmlns") {
            return Ok((None, XmlSchemaQNameState::Invalid));
        }
        if prefix.is_none() && !use_default {
            return Ok((
                Some(ResolvedName {
                    namespace: None,
                    local,
                    binding: None,
                }),
                XmlSchemaQNameState::Expanded,
            ));
        }
        for frame in self.frames.iter().rev() {
            budget.visit(1, stop)?;
            for binding in frame.bindings.iter().rev() {
                budget.visit(1, stop)?;
                if binding.prefix == prefix {
                    let namespace = (!binding.attribute.value().is_empty())
                        .then_some(binding.attribute.value());
                    return Ok((
                        Some(ResolvedName {
                            namespace,
                            local,
                            binding: Some(*binding),
                        }),
                        XmlSchemaQNameState::Expanded,
                    ));
                }
            }
        }
        if prefix == Some("xml") {
            return Ok((
                Some(ResolvedName {
                    namespace: Some(XML),
                    local,
                    binding: None,
                }),
                XmlSchemaQNameState::Expanded,
            ));
        }
        if prefix.is_some() {
            Ok((None, XmlSchemaQNameState::UnboundPrefix))
        } else {
            Ok((
                Some(ResolvedName {
                    namespace: None,
                    local,
                    binding: None,
                }),
                XmlSchemaQNameState::Expanded,
            ))
        }
    }
}

pub(super) fn xml_space(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\r' | '\n')
}

/// XML 1.0 NCName syntax; this validates names, not source XML tokenization.
pub(super) fn ncname(
    name: &str,
    budget: &mut SchemaBudget,
    stop: &AtomicBool,
) -> ProjectResult<bool> {
    let mut characters = name.chars();
    budget.visit(1, stop)?;
    let Some(first) = characters.next() else {
        return Ok(false);
    };
    if !name_start(first) {
        return Ok(false);
    }
    for character in characters {
        budget.visit(1, stop)?;
        if !name_start(character)
            && !matches!(character, '-' | '.' | '0'..='9' | '\u{b7}' | '\u{300}'..='\u{36f}' | '\u{203f}'..='\u{2040}')
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn name_start(character: char) -> bool {
    matches!(character,
        '_' | 'A'..='Z' | 'a'..='z' | '\u{c0}'..='\u{d6}' | '\u{d8}'..='\u{f6}'
        | '\u{f8}'..='\u{2ff}' | '\u{370}'..='\u{37d}' | '\u{37f}'..='\u{1fff}'
        | '\u{200c}'..='\u{200d}' | '\u{2070}'..='\u{218f}' | '\u{2c00}'..='\u{2fef}'
        | '\u{3001}'..='\u{d7ff}' | '\u{f900}'..='\u{fdcf}' | '\u{fdf0}'..='\u{fffd}'
        | '\u{10000}'..='\u{effff}'
    )
}

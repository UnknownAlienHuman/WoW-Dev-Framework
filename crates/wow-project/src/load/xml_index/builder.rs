use super::inline::{InlineBuilder, valid_characters};
use super::*;
use crate::load::{budget, invalid};
use crate::{ProjectPhase, ProjectResult};
use std::sync::Arc;

struct OpenElement {
    index: usize,
    active_script: Option<usize>,
    ui_scope: bool,
}

pub(in crate::load) struct Builder<'a> {
    document: &'a str,
    text: &'a str,
    source_digest: ContentDigest<SourceContent>,
    lines: Arc<Vec<usize>>,
    elements: Vec<XmlElementRecord>,
    stack: Vec<OpenElement>,
    bodies: std::collections::BTreeMap<usize, InlineBuilder>,
    attribute_count: usize,
    segment_count: usize,
}

impl<'a> Builder<'a> {
    pub(in crate::load) fn new(document: &'a str, text: &'a str) -> Self {
        let mut lines = vec![0];
        let bytes = text.as_bytes();
        for (index, byte) in bytes.iter().enumerate() {
            if *byte == b'\n' || (*byte == b'\r' && bytes.get(index + 1) != Some(&b'\n')) {
                lines.push(index + 1);
            }
        }
        Self {
            document,
            text,
            source_digest: crate::identity::source_digest(bytes),
            lines: Arc::new(lines),
            elements: Vec::new(),
            stack: Vec::new(),
            bodies: std::collections::BTreeMap::new(),
            attribute_count: 0,
            segment_count: 0,
        }
    }

    /// Locate a validated quick-xml attribute in the original start tag. This
    /// cursor only recovers lexical spans; quick-xml owns syntax/entity decoding.
    pub(in crate::load) fn attribute(
        &mut self,
        cursor: &mut usize,
        end: usize,
        name: &str,
        value: String,
    ) -> ProjectResult<XmlAttributeRecord> {
        self.attribute_count += 1;
        if self.attribute_count > 65_536 {
            return Err(budget());
        }
        valid_characters(&value)?;
        let bytes = self.text.as_bytes();
        while *cursor < end && bytes[*cursor].is_ascii_whitespace() {
            *cursor += 1;
        }
        let start = *cursor;
        let name_end = start.checked_add(name.len()).ok_or_else(budget)?;
        if name_end >= end || self.text.get(start..name_end) != Some(name) {
            return Err(invalid("XML attribute span does not match its token"));
        }
        *cursor = name_end;
        while *cursor < end && bytes[*cursor].is_ascii_whitespace() {
            *cursor += 1;
        }
        if bytes.get(*cursor) != Some(&b'=') {
            return Err(invalid("invalid XML attribute span"));
        }
        *cursor += 1;
        while *cursor < end && bytes[*cursor].is_ascii_whitespace() {
            *cursor += 1;
        }
        let quote = *bytes
            .get(*cursor)
            .ok_or_else(|| invalid("missing XML attribute quote"))?;
        if !matches!(quote, b'\'' | b'"') {
            return Err(invalid("invalid XML attribute quote"));
        }
        *cursor += 1;
        let value_start = *cursor;
        while *cursor < end && bytes[*cursor] != quote {
            *cursor += 1;
        }
        if *cursor >= end {
            return Err(invalid("unterminated XML attribute span"));
        }
        let value_end = *cursor;
        *cursor += 1;
        Ok(XmlAttributeRecord {
            qualified_name: name.to_owned(),
            span: span(&self.lines, start, *cursor),
            value_span: span(&self.lines, value_start, value_end),
            decoded_value_digest: crate::identity::source_digest(value.as_bytes()),
            value,
        })
    }

    pub(in crate::load) fn begin(
        &mut self,
        start: usize,
        end: usize,
        name: String,
        attributes: Vec<XmlAttributeRecord>,
        ui_namespace: bool,
        empty: bool,
    ) -> ProjectResult<()> {
        if self.elements.len() >= 32_768 {
            return Err(budget());
        }
        let index = self.elements.len();
        let parent_index = self.stack.last().map(|parent| parent.index);
        let parent = parent_index.map(|index| &self.elements[index]);
        let outer_script = self.stack.last().and_then(|parent| parent.active_script);
        let ui_scope = ui_namespace
            && !name.contains(':')
            && self
                .stack
                .last()
                .map_or(name == "Ui", |parent| parent.ui_scope);
        let role = if !ui_namespace || name.contains(':') {
            XmlElementRole::UnknownNamespace
        } else if !ui_scope || outer_script.is_some() {
            XmlElementRole::Element
        } else if parent.is_none() && name == "Ui" {
            XmlElementRole::Ui
        } else if parent.is_some_and(|p| p.role == XmlElementRole::Ui) && name == "Include" {
            XmlElementRole::Include
        } else if parent.is_some_and(|p| p.role == XmlElementRole::Ui) && name == "Script" {
            XmlElementRole::Script
        } else if parent.is_some_and(|p| p.role == XmlElementRole::Scripts) {
            XmlElementRole::ScriptBinding
        } else if name == "Scripts" {
            XmlElementRole::Scripts
        } else {
            XmlElementRole::Element
        };
        let parent_occurrence_id = parent.map(|parent| parent.occurrence_id.clone());
        // This is a source occurrence key, not the future semantic XmlObjectId.
        let identity = crate::identity::canonical_digest(
            "wow-project/xml-occurrence/1",
            &(self.document, self.source_digest, start, &name),
            ProjectPhase::Inventory,
        )?;
        let occurrence_id = format!("xml-node:{identity}");
        let mut issues = Vec::new();
        let declaration = if ui_scope && outer_script.is_none() && role == XmlElementRole::Element {
            declaration(&attributes, &mut issues)?
        } else {
            None
        };
        let is_script = matches!(role, XmlElementRole::Script | XmlElementRole::ScriptBinding);
        let script = if is_script {
            let owner = if role == XmlElementRole::ScriptBinding {
                parent.and_then(|parent| parent.parent_occurrence_id.clone())
            } else {
                parent_occurrence_id.clone()
            };
            Some(XmlScriptRecord {
                owner_occurrence_id: owner,
                source_kind: XmlScriptSource::Unresolved,
                file_reference: value(&attributes, "file"),
                function_reference: value(&attributes, "function"),
                method_reference: value(&attributes, "method"),
                inherit: value(&attributes, "inherit"),
                intrinsic_order: value(&attributes, "intrinsicOrder"),
                body_span: span(&self.lines, end, end),
                inline_lua: None,
            })
        } else {
            None
        };
        if let Some(outer) = outer_script {
            let issues = &mut self.elements[outer].issues;
            if !issues.contains(&XmlStructureIssue::NestedScriptMarkup) {
                issues.push(XmlStructureIssue::NestedScriptMarkup);
            }
        }
        self.elements.push(XmlElementRecord {
            occurrence_id,
            parent_occurrence_id,
            qualified_name: name,
            ui_namespace,
            role,
            span: span(&self.lines, start, end),
            start_tag_span: span(&self.lines, start, end),
            end_tag_span: None,
            attributes,
            declaration,
            script,
            issues,
        });
        if is_script {
            self.bodies
                .insert(index, InlineBuilder::new(Arc::clone(&self.lines)));
        }
        if empty {
            self.finish_script(index, end)?;
        } else {
            self.stack.push(OpenElement {
                index,
                ui_scope,
                active_script: if is_script { Some(index) } else { outer_script },
            });
        }
        Ok(())
    }

    pub(in crate::load) fn end(&mut self, start: usize, end: usize) -> ProjectResult<()> {
        let open = self
            .stack
            .pop()
            .ok_or_else(|| invalid("missing XML index parent"))?;
        let element = &mut self.elements[open.index];
        element.span = span(&self.lines, element.span.byte_start as usize, end);
        element.end_tag_span = Some(span(&self.lines, start, end));
        self.finish_script(open.index, start)
    }

    pub(in crate::load) fn literal(&mut self, start: usize, end: usize) -> ProjectResult<()> {
        let text = self
            .text
            .get(start..end)
            .ok_or_else(|| invalid("invalid XML character-data span"))?;
        valid_characters(text)?;
        if let Some(index) = self.stack.last().map(|parent| parent.index)
            && let Some(body) = self.bodies.get_mut(&index)
        {
            body.literal(text, start)?;
        }
        Ok(())
    }
    pub(in crate::load) fn entity(
        &mut self,
        value: &str,
        start: usize,
        end: usize,
    ) -> ProjectResult<()> {
        valid_characters(value)?;
        if let Some(index) = self.stack.last().map(|parent| parent.index)
            && let Some(body) = self.bodies.get_mut(&index)
        {
            body.entity(value, start, end)?;
        }
        Ok(())
    }

    fn finish_script(&mut self, index: usize, body_end: usize) -> ProjectResult<()> {
        let Some(body) = self.bodies.remove(&index) else {
            return Ok(());
        };
        let element = &mut self.elements[index];
        let script = element
            .script
            .as_mut()
            .ok_or_else(|| invalid("missing XML script record"))?;
        script.body_span = span(&self.lines, script.body_span.byte_start as usize, body_end);
        let references = [
            &script.file_reference,
            &script.function_reference,
            &script.method_reference,
        ];
        let count = references
            .iter()
            .filter(|reference| reference.is_some())
            .count();
        let malformed = references.iter().any(|reference| {
            reference
                .as_ref()
                .is_some_and(|value| value.trim().is_empty())
        });
        if count > 1 || malformed || (count != 0 && body.nonempty()) {
            element
                .issues
                .push(XmlStructureIssue::AmbiguousScriptSource);
        }
        if !element.issues.is_empty() {
            script.source_kind = XmlScriptSource::Unresolved;
        } else if script.file_reference.is_some() {
            script.source_kind = XmlScriptSource::ExternalFile;
        } else if count != 0 {
            script.source_kind = XmlScriptSource::ReferenceOnly;
        } else {
            let body = body.finish(&element.occurrence_id)?;
            self.segment_count += body.segments().len();
            if self.segment_count > 65_536 {
                return Err(budget());
            }
            script.inline_lua = Some(body);
            script.source_kind = XmlScriptSource::InlineBody;
        }
        Ok(())
    }

    pub(in crate::load) fn finish(self) -> ProjectResult<XmlDocumentIndex> {
        if !self.stack.is_empty() || !self.bodies.is_empty() {
            return Err(invalid("XML syntax index is incomplete"));
        }
        let digest = crate::identity::canonical_digest(
            "wow-project/xml-index/1",
            &(
                XML_INDEX_PROFILE,
                self.document,
                self.source_digest,
                &self.elements,
            ),
            ProjectPhase::Inventory,
        )?;
        Ok(XmlDocumentIndex {
            profile: XML_INDEX_PROFILE,
            document: self.document.to_owned(),
            source_digest: self.source_digest,
            elements: self.elements,
            digest,
        })
    }
}

fn value(attributes: &[XmlAttributeRecord], name: &str) -> Option<String> {
    attributes
        .iter()
        .find(|attribute| attribute.qualified_name == name)
        .map(|attribute| attribute.value.clone())
}
fn boolean(
    attributes: &[XmlAttributeRecord],
    name: &str,
    issues: &mut Vec<XmlStructureIssue>,
) -> Option<bool> {
    match value(attributes, name).as_deref() {
        None => None,
        Some("true" | "1") => Some(true),
        Some("false" | "0") => Some(false),
        _ => {
            if !issues.contains(&XmlStructureIssue::InvalidDeclaration) {
                issues.push(XmlStructureIssue::InvalidDeclaration);
            }
            None
        }
    }
}
fn references(
    attributes: &[XmlAttributeRecord],
    name: &str,
    issues: &mut Vec<XmlStructureIssue>,
) -> ProjectResult<Vec<String>> {
    let Some(raw) = value(attributes, name) else {
        return Ok(Vec::new());
    };
    let mut result = Vec::new();
    for part in raw.split(',') {
        if result.len() >= 64 || part.len() > 4096 {
            return Err(budget());
        }
        if part.trim().is_empty() && !issues.contains(&XmlStructureIssue::InvalidDeclaration) {
            issues.push(XmlStructureIssue::InvalidDeclaration);
        }
        result.push(part.trim().to_owned());
    }
    Ok(result)
}
fn declaration(
    attributes: &[XmlAttributeRecord],
    issues: &mut Vec<XmlStructureIssue>,
) -> ProjectResult<Option<XmlDeclaration>> {
    if !attributes.iter().any(|attribute| {
        matches!(
            attribute.qualified_name.as_str(),
            "name"
                | "virtual"
                | "intrinsic"
                | "parent"
                | "parentKey"
                | "parentArray"
                | "inherits"
                | "mixin"
        )
    }) {
        return Ok(None);
    }
    Ok(Some(XmlDeclaration {
        name: value(attributes, "name"),
        virtual_template: boolean(attributes, "virtual", issues),
        intrinsic: boolean(attributes, "intrinsic", issues),
        parent_reference: value(attributes, "parent"),
        parent_key: value(attributes, "parentKey"),
        parent_array: value(attributes, "parentArray"),
        inherits: references(attributes, "inherits", issues)?,
        mixins: references(attributes, "mixin", issues)?,
    }))
}

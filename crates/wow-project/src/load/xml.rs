use super::xml_index::{Builder, XmlDocumentIndex};
use super::{LoadIssueKind as Issue, LoadRecordKind as Kind, MAX_RECORDS, Record, budget, invalid};
use crate::{ProjectResult, disk::checkpoint};
use quick_xml::{Reader, events::Event};
use std::sync::atomic::AtomicBool;

const UI_NAMESPACE: &str = "http://www.blizzard.com/wow/ui/";

struct Frame {
    ui_namespace: bool,
    is_ui: bool,
}

/// One tokenizer pass drives both external-file projection and the syntax index.
/// Syntax records do not certify complete target-schema or runtime semantics.
pub(super) fn parse(
    document: &str,
    text: &str,
    stop: &AtomicBool,
) -> ProjectResult<(Vec<Record>, XmlDocumentIndex)> {
    let mut index = Builder::new(document, text);
    let mut reader = Reader::from_str(text);
    // The slice reader removes an initial UTF-8 BOM without counting it in
    // buffer_position. Every retained span addresses the original source bytes.
    let bom_bytes = usize::from(text.starts_with('\u{feff}')) * '\u{feff}'.len_utf8();
    reader.config_mut().check_comments = true;
    reader.config_mut().check_end_names = true;
    let mut stack: Vec<Frame> = Vec::new();
    let mut records = Vec::new();
    let mut root_seen = false;
    let mut declaration_seen = false;
    loop {
        checkpoint(stop)?;
        let start = (reader.buffer_position() as usize)
            .checked_add(bom_bytes)
            .ok_or_else(budget)?;
        let event = reader
            .read_event()
            .map_err(|_| invalid("XML source is malformed"))?;
        let end = (reader.buffer_position() as usize)
            .checked_add(bom_bytes)
            .ok_or_else(budget)?;
        if matches!(&event, Event::Eof) {
            if !root_seen || !stack.is_empty() {
                return Err(invalid("XML document is incomplete"));
            }
            break;
        }
        if records.len() >= MAX_RECORDS {
            return Err(budget());
        }
        let empty = matches!(&event, Event::Empty(_));
        let mut record = Record::new(Kind::XmlElement, start, end);
        // XML element/site permission is proved separately from the same native index.
        record.file_enumeration_complete = true;
        match event {
            Event::Start(element) | Event::Empty(element) => {
                if stack.len() >= 64 || element.name().as_ref().len() > 256 {
                    return Err(budget());
                }
                let root = stack.is_empty();
                if root {
                    if root_seen {
                        return Err(invalid("XML contains more than one root"));
                    }
                    root_seen = true;
                }
                let mut ui_namespace = stack.last().is_none_or(|parent| parent.ui_namespace);
                let name = element.name();
                let name = name.as_ref();
                // Locate the tag in the original, position-corrected event range.
                let tag_start = start
                    + text
                        .get(start..end)
                        .and_then(|raw| raw.find('<'))
                        .ok_or_else(|| invalid("XML start tag has no source span"))?;
                let mut cursor = tag_start + 1 + name.len();
                let mut attributes = Vec::new();
                let mut file = None;
                let mut extra_attributes = false;
                for (ordinal, attribute) in element.attributes().enumerate() {
                    if ordinal >= 64 {
                        return Err(budget());
                    }
                    let attribute =
                        attribute.map_err(|_| invalid("invalid or duplicate XML attribute"))?;
                    if attribute.key.as_ref().len() > 256 || attribute.value.len() > 16_384 {
                        return Err(budget());
                    }
                    let value = attribute
                        .decode_and_unescape_value(reader.decoder())
                        .map_err(|_| invalid("XML attribute encoding or entity was rejected"))?;
                    let attribute_name = std::str::from_utf8(attribute.key.as_ref())
                        .map_err(|_| invalid("XML attribute name is not UTF-8"))?;
                    attributes.push(index.attribute(
                        &mut cursor,
                        end,
                        attribute_name,
                        value.to_string(),
                    )?);
                    match attribute.key.as_ref() {
                        b"xmlns" => ui_namespace = value == UI_NAMESPACE || value.is_empty(),
                        b"xmlns:xsi"
                            if root && value == "http://www.w3.org/2001/XMLSchema-instance" => {}
                        b"xsi:schemaLocation" if root => {} // Location is inert data, never fetched.
                        b"file" => file = Some(value.into_owned()),
                        _ => extra_attributes = true,
                    }
                }
                let is_ui = ui_namespace && name == b"Ui" && root;
                let load_element = ui_namespace
                    && stack.len() == 1
                    && stack.last().is_some_and(|parent| parent.is_ui)
                    && matches!(name, b"Include" | b"Script");
                if is_ui {
                    if extra_attributes || file.is_some() {
                        record.file_enumeration_complete = false;
                        record.issues.push(Issue::UnsupportedXmlAttributes);
                    }
                } else if load_element {
                    if extra_attributes {
                        record.file_enumeration_complete = false;
                        record.issues.push(Issue::UnsupportedXmlAttributes);
                    } else if let Some(file) = file {
                        let xml = name == b"Include";
                        if (xml && file.ends_with(".xml")) || (!xml && file.ends_with(".lua")) {
                            record.kind = if xml { Kind::XmlFile } else { Kind::LuaFile };
                            record.target = Some(file);
                        } else {
                            record.file_enumeration_complete = false;
                            record.issues.push(Issue::UnsupportedFileKind);
                        }
                    } else if name == b"Script" {
                        record.issues.push(Issue::InlineLuaNotAnalyzed);
                    } else {
                        record.file_enumeration_complete = false;
                        record.issues.push(Issue::UnsupportedXmlAttributes);
                    }
                } else {
                    record.file_enumeration_complete =
                        ui_namespace && !name.contains(&b':') && file.is_none();
                    record
                        .issues
                        .push(if !ui_namespace || name.contains(&b':') {
                            Issue::UnsupportedXmlNamespace
                        } else {
                            Issue::XmlSemanticsUnresolved
                        });
                }
                index.begin(
                    tag_start,
                    end,
                    std::str::from_utf8(name)
                        .map_err(|_| invalid("XML element name is not UTF-8"))?
                        .to_owned(),
                    attributes,
                    ui_namespace,
                    empty,
                )?;
                if !empty {
                    stack.push(Frame {
                        ui_namespace,
                        is_ui,
                    });
                }
            }
            Event::End(_) => {
                if stack.pop().is_none() {
                    return Err(invalid("unmatched XML closing element"));
                }
                index.end(start, end)?;
                record.kind = Kind::XmlEnd;
            }
            Event::Text(text) => {
                index.literal(start, end, stop)?;
                record.kind = Kind::XmlText;
                let nonempty = text.iter().any(|b| !b.is_ascii_whitespace());
                if nonempty {
                    if stack.is_empty() {
                        return Err(invalid("XML text outside its root"));
                    }
                    record.issues.push(Issue::InlineLuaNotAnalyzed);
                }
            }
            Event::CData(_) => {
                if stack.is_empty() {
                    return Err(invalid("XML CDATA outside its root"));
                }
                if text
                    .get(start..end)
                    .is_none_or(|raw| !raw.starts_with("<![CDATA[") || !raw.ends_with("]]>"))
                {
                    return Err(invalid("XML CDATA source span does not match its token"));
                }
                index.literal(start + 9, end - 3, stop)?;
                record.kind = Kind::XmlText;
                record.issues.push(Issue::InlineLuaNotAnalyzed);
            }
            Event::GeneralRef(reference) => {
                if stack.is_empty() {
                    return Err(invalid("XML entity outside its root"));
                }
                let name = std::str::from_utf8(reference.as_ref())
                    .map_err(|_| invalid("invalid XML entity"))?;
                let encoded = format!("&{name};");
                let value = quick_xml::escape::unescape(&encoded)
                    .map_err(|_| invalid("custom XML entities are disabled"))?;
                index.entity(&value, start, end, stop)?;
                record.kind = Kind::XmlText;
                record.issues.push(Issue::InlineLuaNotAnalyzed);
            }
            Event::Comment(_) => record.kind = Kind::Comment,
            Event::Decl(declaration) => {
                if root_seen || declaration_seen {
                    return Err(invalid("XML declaration must precede its root"));
                }
                declaration_seen = true;
                if declaration
                    .version()
                    .map_err(|_| invalid("invalid XML declaration"))?
                    .as_ref()
                    != b"1.0"
                {
                    return Err(invalid("unsupported XML version"));
                }
                if let Some(encoding) = declaration.encoding()
                    && !encoding
                        .map_err(|_| invalid("invalid XML encoding"))?
                        .eq_ignore_ascii_case(b"UTF-8")
                {
                    return Err(invalid("XML input must declare UTF-8"));
                }
                record.kind = Kind::XmlDeclaration;
            }
            Event::DocType(_) | Event::PI(_) => {
                return Err(invalid("XML DTD and processing instructions are disabled"));
            }
            Event::Eof => return Err(invalid("unexpected XML end of input")),
        }
        records.push(record);
    }
    Ok((records, index.finish()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProjectErrorCode;

    #[test]
    fn bom_positions_retain_original_attribute_and_inline_source_ranges() -> ProjectResult<()> {
        for (prefix, newline) in [("", "\n"), ("\u{feff}", "\n"), ("\u{feff}", "\r\n")] {
            let attribute_value = format!("a{newline}λ");
            let inline_xml = "local x = \"λ&amp;\"";
            let text = format!(
                "{prefix}<?xml version=\"1.0\"?>{newline}<Ui xmlns=\"{UI_NAMESPACE}\" path='{attribute_value}'><Frame><Scripts><OnLoad>{inline_xml}</OnLoad></Scripts></Frame></Ui>"
            );
            let (records, index) = parse("frames.xml", &text, &AtomicBool::new(false))?;
            let root = &index.elements()[0];
            assert_eq!(
                root.start_tag_span.byte_start as usize,
                text.find("<Ui")
                    .ok_or_else(|| invalid("missing test root"))?
            );
            let attribute = &root.attributes[1];
            assert_eq!(
                &text[attribute.value_span.byte_start as usize
                    ..attribute.value_span.byte_end as usize],
                attribute_value
            );
            assert_eq!(
                &text[attribute.span.byte_start as usize..attribute.span.byte_end as usize],
                format!("path='{attribute_value}'")
            );
            assert_eq!(attribute.value_span.start_line, 2);
            assert_eq!(attribute.value_span.end_line, 3);
            let handler = index
                .scripts()
                .next()
                .ok_or_else(|| invalid("missing test handler"))?;
            let script = handler
                .script
                .as_ref()
                .ok_or_else(|| invalid("missing test script"))?;
            assert_eq!(
                &text[script.body_span.byte_start as usize..script.body_span.byte_end as usize],
                inline_xml
            );
            let body = script
                .inline_lua
                .as_ref()
                .ok_or_else(|| invalid("missing test inline body"))?;
            assert_eq!(body.text(), "local x = \"λ&\"");
            let start = body
                .text()
                .find("λ&")
                .ok_or_else(|| invalid("missing test expression"))?;
            let pieces = body.map_range(start, start + "λ&".len())?;
            let original = pieces
                .iter()
                .map(|piece| &text[piece.byte_start as usize..piece.byte_end as usize])
                .collect::<String>();
            assert_eq!(original, "λ&amp;");
            assert_eq!(records[0].start, prefix.len());
            assert_eq!(
                records
                    .last()
                    .ok_or_else(|| invalid("missing test end record"))?
                    .end,
                text.len()
            );
        }
        for invalid in [
            "\u{feff}<Ui name='a' name='b'/>",
            "\u{feff}<!DOCTYPE Ui [<!ENTITY x 'y'>]><Ui/>",
            "\u{feff}<Ui><Frame></Ui>",
        ] {
            assert!(parse("frames.xml", invalid, &AtomicBool::new(false)).is_err());
        }
        assert_eq!(
            parse("frames.xml", "\u{feff}<Ui/>", &AtomicBool::new(true))
                .err()
                .ok_or_else(|| invalid("missing test cancellation"))?
                .code(),
            ProjectErrorCode::SourceReadCancelled
        );
        Ok(())
    }
}

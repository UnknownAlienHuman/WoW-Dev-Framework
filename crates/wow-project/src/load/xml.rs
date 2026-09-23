use super::{LoadIssueKind as Issue, LoadRecordKind as Kind, MAX_RECORDS, Record, budget, invalid};
use crate::{ProjectResult, disk::checkpoint};
use quick_xml::{Reader, events::Event};
use std::sync::atomic::AtomicBool;

const UI_NAMESPACE: &str = "http://www.blizzard.com/wow/ui/";

struct Frame {
    ui_namespace: bool,
    is_ui: bool,
}

/// A streaming projection of unprefixed Ui/Include/Script file forms. Other XML
/// is retained by spans/digests and is explicitly not a complete semantic index.
pub(super) fn parse(text: &str, stop: &AtomicBool) -> ProjectResult<Vec<Record>> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().check_comments = true;
    reader.config_mut().check_end_names = true;
    let mut stack: Vec<Frame> = Vec::new();
    let mut records = Vec::new();
    let mut root_seen = false;
    let mut declaration_seen = false;
    loop {
        checkpoint(stop)?;
        let start = reader.buffer_position() as usize;
        let event = reader
            .read_event()
            .map_err(|_| invalid("XML source is malformed"))?;
        let end = reader.buffer_position() as usize;
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
                let mut file = None;
                let mut extra_attributes = false;
                for (index, attribute) in element.attributes().enumerate() {
                    if index >= 64 {
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
                        record.issues.push(Issue::UnsupportedXmlAttributes);
                    }
                } else if load_element {
                    if extra_attributes {
                        record.issues.push(Issue::UnsupportedXmlAttributes);
                    } else if let Some(file) = file {
                        let xml = name == b"Include";
                        if (xml && file.ends_with(".xml")) || (!xml && file.ends_with(".lua")) {
                            record.kind = if xml { Kind::XmlFile } else { Kind::LuaFile };
                            record.target = Some(file);
                        } else {
                            record.issues.push(Issue::UnsupportedFileKind);
                        }
                    } else if name == b"Script" {
                        record.issues.push(Issue::InlineLuaNotMaterialized);
                    } else {
                        record.issues.push(Issue::UnsupportedXmlAttributes);
                    }
                } else {
                    record
                        .issues
                        .push(if !ui_namespace || name.contains(&b':') {
                            Issue::UnsupportedXmlNamespace
                        } else {
                            Issue::XmlStructureNotIndexed
                        });
                }
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
                record.kind = Kind::XmlEnd;
            }
            Event::Text(text) => {
                record.kind = Kind::XmlText;
                let nonempty = text.iter().any(|b| !b.is_ascii_whitespace());
                if nonempty {
                    if stack.is_empty() {
                        return Err(invalid("XML text outside its root"));
                    }
                    record.issues.push(Issue::InlineLuaNotMaterialized);
                }
            }
            Event::CData(_) => {
                if stack.is_empty() {
                    return Err(invalid("XML CDATA outside its root"));
                }
                record.kind = Kind::XmlText;
                record.issues.push(Issue::InlineLuaNotMaterialized);
            }
            Event::GeneralRef(reference) => {
                if stack.is_empty() {
                    return Err(invalid("XML entity outside its root"));
                }
                let name = std::str::from_utf8(reference.as_ref())
                    .map_err(|_| invalid("invalid XML entity"))?;
                quick_xml::escape::unescape(&format!("&{name};"))
                    .map_err(|_| invalid("custom XML entities are disabled"))?;
                record.kind = Kind::XmlText;
                record.issues.push(Issue::InlineLuaNotMaterialized);
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
    Ok(records)
}

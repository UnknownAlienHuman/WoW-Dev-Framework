use super::super::{
    LoadRecordKind, LoadSelection, Record, TocLoadContext, XmlScriptSource, budget, invalid, toc,
    xml,
};
use super::{CensusDocumentOutcome, CensusRefusal, CensusSyntaxCounts};
use crate::{
    ProjectErrorCode, ProjectResult,
    disk::{DISK_SOURCE_MAX_BYTES, checkpoint},
    platform_source::{PlatformFileKind, PlatformRawMember},
};
use serde::Serialize;
use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
};

// Same finite envelope ceiling as native artifacts; serialization is counted
// through a sink, without retaining every document index or allocating its JSON.
const MAX_ENCODED_BYTES: u64 = 64 * 1024 * 1024;

pub(super) fn add(value: &mut u64, count: u64) -> ProjectResult<()> {
    *value = value.checked_add(count).ok_or_else(budget)?;
    Ok(())
}

pub(super) fn document(
    member: &PlatformRawMember<'_>,
    interface: u64,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<CensusDocumentOutcome> {
    checkpoint(stop)?;
    if member.byte_length() > DISK_SOURCE_MAX_BYTES as u64 {
        return Ok(CensusDocumentOutcome::Refused(
            CensusRefusal::DocumentByteLimit,
        ));
    }
    let text = match std::str::from_utf8(member.bytes()) {
        Ok(text) if !text.contains('\0') => text,
        _ => {
            return Ok(CensusDocumentOutcome::Refused(
                CensusRefusal::InvalidEncoding,
            ));
        }
    };
    let measured: ProjectResult<CensusSyntaxCounts> = (|| {
        let mut counts = CensusSyntaxCounts {
            documents: 1,
            ..CensusSyntaxCounts::default()
        };
        if member.kind() == PlatformFileKind::Toc {
            records(
                &toc::parse(text, interface, context, stop)?,
                &mut counts,
                stop,
            )?;
        } else {
            let (parsed, index) = xml::parse(member.path(), text, stop)?;
            records(&parsed, &mut counts, stop)?;
            counts.index_json_bytes = encoded_size(&index, stop)?;
            for element in index.elements() {
                checkpoint(stop)?;
                add(&mut counts.elements, 1)?;
                add(&mut counts.attributes, element.attributes.len() as u64)?;
                for attribute in &element.attributes {
                    checkpoint(stop)?;
                    add(
                        &mut counts.decoded_attribute_value_bytes,
                        attribute.value().len() as u64,
                    )?;
                }
                if let Some(script) = &element.script {
                    add(&mut counts.script_sites, 1)?;
                    add(
                        match script.source_kind {
                            XmlScriptSource::ExternalFile => &mut counts.external_scripts,
                            XmlScriptSource::ReferenceOnly => &mut counts.reference_scripts,
                            XmlScriptSource::InlineBody => &mut counts.inline_scripts,
                            XmlScriptSource::Unresolved => &mut counts.unresolved_scripts,
                        },
                        1,
                    )?;
                    if let Some(body) = &script.inline_lua {
                        add(&mut counts.inline_units, 1)?;
                        add(&mut counts.inline_bytes, body.byte_length)?;
                        add(&mut counts.map_segments, body.segments().len() as u64)?;
                    }
                }
            }
        }
        checkpoint(stop)?;
        Ok(counts)
    })();
    match measured {
        Ok(counts) => Ok(CensusDocumentOutcome::Measured(counts)),
        Err(error) if error.code() == ProjectErrorCode::SourceBudgetExceeded => {
            Ok(CensusDocumentOutcome::Refused(CensusRefusal::ParserBudget))
        }
        Err(error) if error.code() == ProjectErrorCode::InvalidInputInventory => {
            Ok(CensusDocumentOutcome::Refused(CensusRefusal::ParserInvalid))
        }
        Err(error) => Err(error),
    }
}

fn records(
    records: &[Record],
    counts: &mut CensusSyntaxCounts,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    add(&mut counts.lexical_records, records.len() as u64)?;
    for record in records {
        checkpoint(stop)?;
        match record.selection {
            LoadSelection::Included
                if matches!(
                    record.kind,
                    LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
                ) =>
            {
                add(&mut counts.included_file_records, 1)?
            }
            LoadSelection::Excluded => add(&mut counts.excluded_records, 1)?,
            LoadSelection::Unresolved => add(&mut counts.unresolved_records, 1)?,
            LoadSelection::Included => {}
        }
        add(&mut counts.record_issues, record.issues.len() as u64)?;
    }
    Ok(())
}

pub(super) fn encoded_size<T: Serialize + ?Sized>(
    value: &T,
    stop: &AtomicBool,
) -> ProjectResult<u64> {
    let mut counter = Counter {
        used: 0,
        overflow: false,
        stop,
    };
    let result = serde_json::to_writer(&mut counter, value);
    checkpoint(stop)?;
    if counter.overflow {
        return Err(budget());
    }
    result.map_err(|_| invalid("census metadata cannot be encoded"))?;
    Ok(counter.used)
}

struct Counter<'a> {
    used: u64,
    overflow: bool,
    stop: &'a AtomicBool,
}
impl io::Write for Counter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.stop.load(Ordering::Acquire) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"));
        }
        let Some(next) = self
            .used
            .checked_add(bytes.len() as u64)
            .filter(|next| *next <= MAX_ENCODED_BYTES)
        else {
            self.overflow = true;
            return Err(io::Error::other("census metadata budget exceeded"));
        };
        self.used = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

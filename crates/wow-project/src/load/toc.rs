use super::{
    LoadIssueKind as Issue, LoadRecordKind as Kind, LoadSelection, MAX_RECORDS, Record,
    TocLoadContext, TocMetadata, TocMetadataKind, budget, conditions, invalid, package,
};
use crate::ProjectResult;
use crate::disk::checkpoint;
use std::sync::atomic::AtomicBool;

pub(super) fn parse(
    text: &str,
    interface: u64,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<Vec<Record>> {
    // Scan the complete captured TOC before expansion. Filters after a file line
    // still govern that file, and excluded targets must not open any descendants.
    let package_filters = package::admit(text, context, stop)?;
    let mut records = Vec::new();
    let mut offset = 0;
    let mut interfaces = 0;
    for line in text.split_inclusive('\n') {
        checkpoint(stop)?;
        if records.len() >= MAX_RECORDS || line.len() > 16_384 {
            return Err(budget());
        }
        let end = offset + line.len();
        let content = if offset == 0 {
            line.trim_start_matches('\u{feff}')
        } else {
            line
        };
        let content = content.trim();
        let mut record = Record::new(Kind::Unknown, offset, end);
        if content.is_empty() {
            record.kind = Kind::Blank;
        } else if let Some(metadata) = content.strip_prefix("##") {
            record.kind = Kind::Metadata;
            let metadata = conditions::project(metadata.trim(), &mut record, context, true)?;
            let retained = metadata.split_once(':').map(|(key, value)| {
                TocMetadata::new(
                    key.trim().to_ascii_lowercase(),
                    value.trim().to_owned(),
                )
            });
            let kind = retained.as_ref().map(TocMetadata::kind);
            record.metadata = retained;
            if record.selection != LoadSelection::Included {
                record.file_enumeration_complete = record.selection == LoadSelection::Excluded;
                records.push(record);
                offset = end;
                continue;
            }
            if let Some(filter) = package_filters.get(&offset) {
                record.kind = Kind::PackageGate;
                record.conditions.push(filter.clone());
                record.file_enumeration_complete = true;
                records.push(record);
                offset = end;
                continue;
            }
            match (kind, record.metadata.as_ref()) {
                (Some(TocMetadataKind::Interface), Some(metadata)) => {
                    interfaces += 1;
                    let versions = metadata
                        .value
                        .split(',')
                        .map(str::trim)
                        .collect::<Vec<_>>();
                    if versions.len() > 64 {
                        return Err(budget());
                    }
                    let mut compatible = false;
                    for version in versions {
                        if version.is_empty() || !version.bytes().all(|b| b.is_ascii_digit()) {
                            return Err(invalid("TOC Interface directive is malformed"));
                        }
                        let number = version
                            .parse::<u64>()
                            .map_err(|_| invalid("TOC Interface value overflow"))?;
                        if number == 0 {
                            return Err(invalid("TOC Interface must be positive"));
                        }
                        compatible |= number == interface;
                    }
                    if !compatible {
                        return Err(invalid(
                            "selected TOC Interface does not match the selected profile",
                        ));
                    }
                    if interfaces > 1 {
                        record.issues.push(Issue::ConflictingInterface);
                    }
                }
                (Some(TocMetadataKind::RequiredDependencies), Some(metadata)) => {
                    if !metadata.value.is_empty() {
                        record.issues.push(Issue::RequiredDependencyUnresolved);
                    }
                }
                (Some(TocMetadataKind::OptionalDependencies), Some(metadata)) => {
                    if !metadata.value.is_empty() {
                        record.issues.push(Issue::OptionalDependencyUnresolved);
                    }
                }
                (Some(TocMetadataKind::LoadOnDemand), Some(metadata)) => {
                    if !matches!(metadata.value.as_str(), "0" | "1") {
                        record.issues.push(Issue::UnknownDirective);
                    }
                }
                (Some(kind), Some(metadata)) if kind.saved_variables() => {
                    record.saved_variables =
                        super::saved_variables::parse(&metadata.key, &metadata.value)?;
                }
                (Some(kind), _) if kind.unsupported_load_context() => {
                    record.issues.push(Issue::UnknownDirective);
                }
                (Some(TocMetadataKind::Unknown), _) => {
                    record.issues.push(Issue::UnknownMetadata);
                }
                // Package filters have already been projected by package::admit.
                // Known lifecycle, persistence, error-policy, retrieval,
                // display and custom metadata remain source-backed and do not
                // make independent file enumeration incomplete.
                (Some(_), _) => {}
                (None, None) => record.issues.push(Issue::UnknownDirective),
                (None, Some(_)) => unreachable!("retained metadata always has a kind"),
            }
        } else if content.starts_with('#') {
            record.kind = Kind::Comment;
        } else {
            let path = conditions::project(content, &mut record, context, false)?;
            if path.ends_with(".lua") || path.ends_with(".xml") {
                record.kind = if path.ends_with(".lua") {
                    Kind::LuaFile
                } else {
                    Kind::XmlFile
                };
                if record.selection == LoadSelection::Included {
                    record.target = Some(path);
                }
            } else if record.selection == LoadSelection::Included {
                record.issues.push(Issue::UnsupportedFileKind);
            }
        }
        record.file_enumeration_complete = match record.selection {
            LoadSelection::Included => record
                .issues
                .iter()
                .all(|issue| !issue.blocks_complete()),
            LoadSelection::Excluded => true,
            LoadSelection::Unresolved => false,
        };
        records.push(record);
        offset = end;
    }
    if interfaces == 0 {
        if records.len() >= MAX_RECORDS {
            return Err(budget());
        }
        records.push(Record::new(Kind::Metadata, 0, 0).issue(Issue::MissingInterface));
    }
    Ok(records)
}

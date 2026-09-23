use super::{
    LoadIssueKind as Issue, LoadRecordKind as Kind, LoadSelection, MAX_RECORDS, Record,
    TocLoadContext, budget, conditions, invalid, package,
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
            if record.selection != LoadSelection::Included {
                records.push(record);
                offset = end;
                continue;
            }
            if let Some(filter) = package_filters.get(&offset) {
                record.kind = Kind::PackageGate;
                record.conditions.push(filter.clone());
                records.push(record);
                offset = end;
                continue;
            }
            if let Some((key, value)) = metadata.split_once(':') {
                let key = key.trim().to_ascii_lowercase();
                let value = value.trim();
                match key.as_str() {
                    "interface" => {
                        interfaces += 1;
                        let versions = value.split(',').map(str::trim).collect::<Vec<_>>();
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
                    "dependencies" | "requireddeps" | "dependson" => {
                        if !value.is_empty() {
                            record.issues.push(Issue::RequiredDependencyUnresolved);
                        }
                    }
                    "optionaldeps" => {
                        if !value.is_empty() {
                            record.issues.push(Issue::OptionalDependencyUnresolved);
                        }
                    }
                    "loadondemand" => {
                        if !matches!(value, "0" | "1") {
                            record.issues.push(Issue::UnknownDirective);
                        }
                    }
                    // These records are retained as exact source spans, not an
                    // invented runtime/storage model. SavedVariables are not read.
                    "title"
                    | "notes"
                    | "author"
                    | "version"
                    | "savedvariables"
                    | "savedvariablespercharacter" => {}
                    _ => record.issues.push(Issue::UnknownDirective),
                }
            } else {
                record.issues.push(Issue::UnknownDirective);
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

//! Strict Primary context-status CLI grammar. Status is a metadata-only Service operation.
use super::{Arguments, MAX_VALUE_BYTES, Request, id_syntax};
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};
use wow_service::context::{
    ContextMapCurrentGuard, ContextMapPublication, ContextMapReferenceGuard, ContextStatusDetail,
    ContextStatusRequest,
};

fn set<T>(slot: &mut Option<T>, value: T) -> Result<(), &'static str> {
    if slot.is_some() {
        return Err("duplicate context status option");
    }
    *slot = Some(value);
    Ok(())
}

/// Parse only `wow context status`; every unlisted option is rejected here.
pub(super) fn parse(values: &[OsString]) -> Result<Option<Arguments>, &'static str> {
    if values.first().map(OsString::as_os_str) != Some(OsStr::new("context"))
        || values.get(1).map(OsString::as_os_str) != Some(OsStr::new("status"))
    {
        return Ok(None);
    }

    let mut host_config = None;
    let mut project_store = None;
    let mut publication = None;
    let mut current_record = None;
    let mut current_digest = None;
    let mut reference_profile = None;
    let mut reference_generation = None;
    let mut reference_view = None;
    let mut detail = None;
    let mut platform_omitted = false;
    let mut output_json = false;

    let mut options = values[2..].iter();
    while let Some(option) = options.next() {
        let option = option
            .to_str()
            .ok_or("invalid context status option encoding")?;
        if !matches!(
            option,
            "--native-host-config"
                | "--project-store"
                | "--publication"
                | "--expect-current-record"
                | "--expect-current-digest"
                | "--platform"
                | "--expect-profile"
                | "--expect-reference-generation"
                | "--expect-reference-view"
                | "--detail"
                | "--output"
        ) {
            return Err("unknown or unsupported context status option");
        }
        let value = options
            .next()
            .ok_or("missing context status option value")?;
        if value.is_empty() || value.len() > MAX_VALUE_BYTES {
            return Err("invalid context status option length");
        }
        if option == "--native-host-config" {
            set(&mut host_config, PathBuf::from(value.as_os_str()))?;
            continue;
        }
        let value = value
            .to_str()
            .ok_or("invalid context status value encoding")?;
        if value.chars().any(char::is_control) || value.trim() != value {
            return Err("invalid context status option value");
        }
        match option {
            "--project-store" => {
                id_syntax(value)?;
                set(&mut project_store, value.to_owned())?;
            }
            "--publication" => {
                let selected = if value == "current" {
                    ContextMapPublication::Current
                } else if let Some(id) = value.strip_prefix("store-generation:") {
                    id_syntax(id)?;
                    ContextMapPublication::store_generation(id)
                        .map_err(|_| "invalid exact context store generation")?
                } else {
                    return Err("unsupported context status publication selector");
                };
                set(&mut publication, selected)?;
            }
            "--expect-current-record" => {
                id_syntax(value)?;
                set(&mut current_record, value.to_owned())?;
            }
            "--expect-current-digest" => {
                id_syntax(value)?;
                set(&mut current_digest, value.to_owned())?;
            }
            "--expect-profile" => {
                id_syntax(value)?;
                set(&mut reference_profile, value.to_owned())?;
            }
            "--expect-reference-generation" => {
                id_syntax(value)?;
                set(&mut reference_generation, value.to_owned())?;
            }
            "--expect-reference-view" => {
                id_syntax(value)?;
                set(&mut reference_view, value.to_owned())?;
            }
            "--detail" => {
                let selected = match value {
                    "summary" => ContextStatusDetail::Summary,
                    "capabilities" => ContextStatusDetail::Capabilities,
                    "profiles" => ContextStatusDetail::Profiles,
                    "generations" => ContextStatusDetail::Generations,
                    _ => return Err("unsupported context status detail"),
                };
                set(&mut detail, selected)?;
            }
            "--platform" => {
                if value != "omitted" {
                    return Err("context status supports omitted platform only");
                }
                if platform_omitted {
                    return Err("duplicate context status option");
                }
                platform_omitted = true;
            }
            "--output" => {
                if value != "envelope-json" {
                    return Err("context status supports envelope-json only");
                }
                if output_json {
                    return Err("duplicate context status option");
                }
                output_json = true;
            }
            _ => return Err("unknown or unsupported context status option"),
        }
    }

    let host_config = host_config.ok_or("context status requires explicit native host input")?;
    let project_store = project_store.ok_or("context status requires --project-store")?;
    let publication = publication.ok_or("context status requires --publication")?;
    let current_guard = if current_record.is_some() || current_digest.is_some() {
        if !matches!(&publication, ContextMapPublication::Current) {
            return Err("current guards require --publication current");
        }
        Some(
            ContextMapCurrentGuard::new(current_record.as_deref(), current_digest.as_deref())
                .map_err(|_| "invalid exact context current guard")?,
        )
    } else {
        None
    };
    let reference_guard = match (reference_profile, reference_generation, reference_view) {
        (None, None, None) => None,
        (Some(profile), Some(generation), Some(view)) => Some(
            ContextMapReferenceGuard::new(&profile, &generation, &view)
                .map_err(|_| "invalid exact context reference guards")?,
        ),
        _ => return Err("context reference guards require the complete three-option group"),
    };
    let detail = detail.unwrap_or(ContextStatusDetail::Summary);
    let request = ContextStatusRequest::primary(
        &project_store,
        publication,
        detail,
        current_guard,
        reference_guard,
    )
    .map_err(|_| "invalid context status request syntax")?;

    Ok(Some(Arguments {
        host_config,
        request: Request::Status(request),
        source_root: None,
    }))
}

use crate::Result;
use std::collections::BTreeSet;
use std::path::{Component, Path};

pub(super) fn tracked_directories(tracked: &BTreeSet<String>) -> BTreeSet<String> {
    let mut directories = BTreeSet::from([String::new()]);
    for path in tracked {
        let mut current = Path::new(path).parent();
        while let Some(directory) = current {
            if directory.as_os_str().is_empty() {
                break;
            }
            directories.insert(path_string(directory));
            current = directory.parent();
        }
    }
    directories
}

pub(super) fn resolve_destination(
    source: &str,
    destination: &str,
) -> Result<Option<(String, Option<String>)>> {
    let destination = destination.trim();
    if destination.is_empty()
        || destination.starts_with("//")
        || destination.starts_with('/')
        || has_uri_scheme(destination)
    {
        return Ok(None);
    }

    let (path_part, fragment) = match destination.split_once('#') {
        Some((path, fragment)) => (path, Some(percent_decode(fragment)?)),
        None => (destination, None),
    };
    let path_part = path_part
        .split_once('?')
        .map_or(path_part, |(path, _)| path);
    let path_part = unescape_destination(path_part);
    let decoded = percent_decode(&path_part)?;
    let target = if decoded.is_empty() {
        source.to_owned()
    } else {
        normalize_path(
            Path::new(source).parent().unwrap_or(Path::new("")),
            &decoded,
        )?
    };
    let fragment = fragment.filter(|value| !value.is_empty());
    Ok(Some((target, fragment)))
}

pub(super) fn display_target(target: &str) -> &str {
    if target.is_empty() { "." } else { target }
}

fn has_uri_scheme(value: &str) -> bool {
    let Some(index) = value.find(':') else {
        return false;
    };
    let prefix = &value[..index];
    !prefix.is_empty()
        && prefix.bytes().enumerate().all(|(index, byte)| {
            if index == 0 {
                byte.is_ascii_alphabetic()
            } else {
                byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.')
            }
        })
}

fn unescape_destination(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            if let Some(escaped) = characters.next() {
                output.push(escaped);
            }
        } else {
            output.push(character);
        }
    }
    output
}

fn normalize_path(base: &Path, destination: &str) -> Result<String> {
    if destination.contains('\\') || destination.chars().any(char::is_control) {
        return Err("local Markdown destination is not a canonical repository path".into());
    }
    let joined = base.join(destination);
    let mut components = Vec::<String>::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => components.push(
                value
                    .to_str()
                    .ok_or("local Markdown destination must be UTF-8")?
                    .to_owned(),
            ),
            Component::ParentDir => {
                if components.pop().is_none() {
                    return Err("local Markdown destination escapes the repository".into());
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("local Markdown destination must be relative".into());
            }
        }
    }
    Ok(components.join("/"))
}

fn percent_decode(value: &str) -> Result<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] == b'%' {
            let high = *bytes.get(cursor + 1).ok_or("incomplete percent escape")?;
            let low = *bytes.get(cursor + 2).ok_or("incomplete percent escape")?;
            decoded.push((hex(high)? << 4) | hex(low)?);
            cursor += 3;
        } else {
            decoded.push(bytes[cursor]);
            cursor += 1;
        }
    }
    Ok(String::from_utf8(decoded)?)
}

fn hex(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("invalid percent escape".into()),
    }
}

fn path_string(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

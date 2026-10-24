mod anchors;
mod links;
mod path;
mod syntax;

use crate::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const MAX_ERRORS: usize = 64;

pub(crate) fn check(root: &Path) -> Result<()> {
    let listing = crate::git::run(
        root,
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        None,
        16 * 1024 * 1024,
    )?;
    let mut tracked = BTreeSet::new();
    let mut markdown = Vec::new();
    for raw in listing.split(|byte| *byte == 0).filter(|entry| !entry.is_empty()) {
        let relative = std::str::from_utf8(raw)?;
        crate::manifest::validate_path(relative)?;
        tracked.insert(relative.to_owned());
        if !relative.ends_with(".md") {
            continue;
        }
        let bytes = fs::read(root.join(relative))?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| format!("non-UTF-8 Markdown document: {relative}"))?;
        markdown.push((relative.to_owned(), text.to_owned()));
    }
    validate(&tracked, &markdown)
}

fn validate(tracked: &BTreeSet<String>, markdown: &[(String, String)]) -> Result<()> {
    let directories = path::tracked_directories(tracked);
    let mut anchors = BTreeMap::<&str, BTreeSet<String>>::new();
    for (file, text) in markdown {
        anchors.insert(file, anchors::collect(text));
    }

    let mut errors = Vec::new();
    let mut truncated = false;
    'documents: for (source, text) in markdown {
        for link in links::collect(text) {
            if errors.len() == MAX_ERRORS {
                truncated = true;
                break 'documents;
            }
            let resolved = path::resolve_destination(source, &link.destination).map_err(|error| {
                format!(
                    "invalid local Markdown link in {source}:{} (`{}`): {error}",
                    link.line, link.destination
                )
            })?;
            let Some((target, fragment)) = resolved else {
                continue;
            };
            if !tracked.contains(&target) && !directories.contains(&target) {
                errors.push(format!(
                    "{source}:{}: missing local Markdown target `{}` (resolved `{}`)",
                    link.line,
                    link.destination,
                    path::display_target(&target)
                ));
                continue;
            }
            if let Some(fragment) = fragment
                && target.to_ascii_lowercase().ends_with(".md")
                && !anchors
                    .get(target.as_str())
                    .is_some_and(|values| values.contains(&fragment))
            {
                errors.push(format!(
                    "{source}:{}: missing Markdown anchor `#{fragment}` in `{target}`",
                    link.line
                ));
            }
        }
    }

    if !errors.is_empty() {
        let mut message = String::from("invalid local Markdown links:\n");
        for error in errors {
            message.push_str("- ");
            message.push_str(&error);
            message.push('\n');
        }
        if truncated {
            message.push_str("- additional errors omitted\n");
        }
        return Err(message.into());
    }

    println!(
        "Checked {} Markdown files; local paths and anchors passed",
        markdown.len()
    );
    Ok(())
}

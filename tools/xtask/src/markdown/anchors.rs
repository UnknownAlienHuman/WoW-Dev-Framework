use super::syntax::{
    fence_run, mask_html_comments, mask_inline_code, opening_fence, parse_destination,
    quoted_attributes,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect(text: &str) -> BTreeSet<String> {
    let mut anchors = BTreeSet::new();
    let mut counts = BTreeMap::<String, usize>::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut previous: Option<String> = None;
    let mut html_comment = false;

    for raw_line in text.lines() {
        let indentation = raw_line.len() - raw_line.trim_start_matches(' ').len();
        let trimmed = raw_line.trim_start();
        if let Some((marker, length)) = fence {
            if fence_run(trimmed, marker) >= length {
                fence = None;
            }
            previous = None;
            continue;
        }
        if indentation <= 3
            && let Some((marker, length)) = opening_fence(trimmed)
        {
            fence = Some((marker, length));
            previous = None;
            continue;
        }
        if indentation >= 4 || raw_line.starts_with('\t') {
            previous = None;
            continue;
        }

        let line = mask_html_comments(raw_line, &mut html_comment);
        let explicit = mask_inline_code(&line);
        collect_explicit_anchors(&explicit, &mut anchors);
        if let Some(heading) = atx_heading(&line) {
            insert_heading_anchor(&heading, &mut counts, &mut anchors);
            previous = None;
            continue;
        }
        if is_setext_underline(&line)
            && let Some(heading) = previous.take()
        {
            insert_heading_anchor(&heading, &mut counts, &mut anchors);
            continue;
        }
        previous = if line.trim().is_empty() {
            None
        } else {
            Some(line.trim().to_owned())
        };
    }
    anchors
}

fn atx_heading(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let hashes = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&hashes)
        || trimmed
            .as_bytes()
            .get(hashes)
            .is_some_and(|byte| !byte.is_ascii_whitespace())
    {
        return None;
    }
    let mut heading = trimmed[hashes..].trim().to_owned();
    while heading.ends_with('#') {
        heading.pop();
    }
    Some(heading.trim_end().to_owned())
}

fn is_setext_underline(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && (trimmed.bytes().all(|byte| byte == b'=')
            || trimmed.bytes().all(|byte| byte == b'-'))
}

fn insert_heading_anchor(
    heading: &str,
    counts: &mut BTreeMap<String, usize>,
    anchors: &mut BTreeSet<String>,
) {
    let base = github_slug(heading);
    if base.is_empty() {
        return;
    }
    let count = counts.entry(base.clone()).or_default();
    let anchor = if *count == 0 {
        base
    } else {
        format!("{base}-{count}")
    };
    *count += 1;
    anchors.insert(anchor);
}

fn github_slug(heading: &str) -> String {
    let visible = visible_heading_text(heading);
    let mut slug = String::new();
    let mut in_tag = false;
    for character in visible.chars().flat_map(char::to_lowercase) {
        match character {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if in_tag => {}
            '`' | '*' | '~' => {}
            value if value.is_alphanumeric() || matches!(value, '-' | '_') => slug.push(value),
            value if value.is_whitespace() => slug.push('-'),
            _ => {}
        }
    }
    slug
}

fn visible_heading_text(heading: &str) -> String {
    let bytes = heading.as_bytes();
    let mut output = String::with_capacity(heading.len());
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] == b']'
            && bytes.get(cursor + 1) == Some(&b'(')
            && let Some((_, end)) = parse_destination(heading, cursor + 2, Some(b')'))
        {
            cursor = end;
            if bytes.get(cursor) == Some(&b')') {
                cursor += 1;
            }
            continue;
        }
        let Some(character) = heading[cursor..].chars().next() else {
            break;
        };
        output.push(character);
        cursor += character.len_utf8();
    }
    output
}

fn collect_explicit_anchors(line: &str, anchors: &mut BTreeSet<String>) {
    for value in quoted_attributes(line, &["id", "name"]) {
        if !value.is_empty() {
            anchors.insert(value);
        }
    }
}

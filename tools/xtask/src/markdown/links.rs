use super::syntax::{
    fence_run, mask_html_comments, mask_inline_code, opening_fence, parse_destination,
    quoted_attributes,
};

#[derive(Debug)]
pub(super) struct MarkdownLink {
    pub(super) line: usize,
    pub(super) destination: String,
}

pub(super) fn collect(text: &str) -> Vec<MarkdownLink> {
    let mut links = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut html_comment = false;
    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let indentation = raw_line.len() - raw_line.trim_start_matches(' ').len();
        let trimmed = raw_line.trim_start();
        if let Some((marker, length)) = fence {
            if fence_run(trimmed, marker) >= length {
                fence = None;
            }
            continue;
        }
        if indentation <= 3
            && let Some((marker, length)) = opening_fence(trimmed)
        {
            fence = Some((marker, length));
            continue;
        }
        if indentation >= 4 || raw_line.starts_with('\t') {
            continue;
        }

        let without_comments = mask_html_comments(raw_line, &mut html_comment);
        let line = mask_inline_code(&without_comments);
        collect_inline_links(&line, line_number, &mut links);
        collect_reference_definition(&line, line_number, &mut links);
        collect_html_links(&line, line_number, &mut links);
    }
    links
}

fn collect_inline_links(line: &str, line_number: usize, links: &mut Vec<MarkdownLink>) {
    let bytes = line.as_bytes();
    let mut cursor = 0;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b']'
            && bytes[cursor + 1] == b'('
            && !escaped(bytes, cursor)
            && let Some((destination, end)) = parse_destination(line, cursor + 2, Some(b')'))
        {
            if !destination.is_empty() {
                links.push(MarkdownLink {
                    line: line_number,
                    destination,
                });
            }
            cursor = end;
            continue;
        }
        cursor += 1;
    }
}

fn collect_reference_definition(line: &str, line_number: usize, links: &mut Vec<MarkdownLink>) {
    let bytes = line.as_bytes();
    let mut cursor = bytes.iter().take_while(|byte| **byte == b' ').count();
    if cursor > 3 || bytes.get(cursor) != Some(&b'[') || bytes.get(cursor + 1) == Some(&b'^') {
        return;
    }
    let Some(close) = line[cursor + 1..].find("]:") else {
        return;
    };
    cursor += close + 3;
    if let Some((destination, _)) = parse_destination(line, cursor, None)
        && !destination.is_empty()
    {
        links.push(MarkdownLink {
            line: line_number,
            destination,
        });
    }
}

fn collect_html_links(line: &str, line_number: usize, links: &mut Vec<MarkdownLink>) {
    for destination in quoted_attributes(line, &["href", "src"]) {
        if !destination.is_empty() {
            links.push(MarkdownLink {
                line: line_number,
                destination,
            });
        }
    }
}

fn escaped(bytes: &[u8], index: usize) -> bool {
    let mut backslashes = 0usize;
    let mut cursor = index;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        backslashes += 1;
        cursor -= 1;
    }
    !backslashes.is_multiple_of(2)
}

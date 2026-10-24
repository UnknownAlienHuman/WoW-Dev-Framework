pub(super) fn opening_fence(line: &str) -> Option<(u8, usize)> {
    let marker = *line.as_bytes().first()?;
    if !matches!(marker, b'`' | b'~') {
        return None;
    }
    let length = fence_run(line, marker);
    (length >= 3).then_some((marker, length))
}

pub(super) fn fence_run(line: &str, marker: u8) -> usize {
    line.as_bytes()
        .iter()
        .take_while(|byte| **byte == marker)
        .count()
}

pub(super) fn mask_inline_code(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut output = bytes.to_vec();
    let mut cursor = 0;
    let mut delimiter = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] != b'`' {
            if delimiter != 0 {
                output[cursor] = b' ';
            }
            cursor += 1;
            continue;
        }
        let start = cursor;
        while bytes.get(cursor) == Some(&b'`') {
            cursor += 1;
        }
        let run = cursor - start;
        if delimiter == 0 {
            delimiter = run;
        } else if run == delimiter {
            delimiter = 0;
        }
        output[start..cursor].fill(b' ');
    }
    String::from_utf8_lossy(&output).into_owned()
}

pub(super) fn mask_html_comments(line: &str, in_comment: &mut bool) -> String {
    let mut output = line.as_bytes().to_vec();
    let mut cursor = 0;
    while cursor < output.len() {
        if *in_comment {
            if let Some(end) = line[cursor..].find("-->") {
                let finish = cursor + end + 3;
                output[cursor..finish].fill(b' ');
                cursor = finish;
                *in_comment = false;
            } else {
                output[cursor..].fill(b' ');
                break;
            }
        } else if let Some(start) = line[cursor..].find("<!--") {
            let begin = cursor + start;
            output[begin..begin + 4].fill(b' ');
            cursor = begin + 4;
            *in_comment = true;
        } else {
            break;
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

pub(super) fn quoted_attributes(line: &str, names: &[&str]) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut values = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while bytes
            .get(cursor)
            .is_some_and(|byte| !byte.is_ascii_alphabetic())
        {
            cursor += 1;
        }
        let start = cursor;
        while bytes.get(cursor).is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_' | b':')
        }) {
            cursor += 1;
        }
        if start == cursor
            || !names
                .iter()
                .any(|name| line[start..cursor].eq_ignore_ascii_case(name))
        {
            continue;
        }
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            continue;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let Some(quote @ (b'\'' | b'"')) = bytes.get(cursor).copied() else {
            continue;
        };
        cursor += 1;
        let value_start = cursor;
        while let Some(byte) = bytes.get(cursor) {
            if *byte == quote {
                values.push(line[value_start..cursor].to_owned());
                cursor += 1;
                break;
            }
            cursor += 1;
        }
    }
    values
}

pub(super) fn parse_destination(
    line: &str,
    start: usize,
    terminator: Option<u8>,
) -> Option<(String, usize)> {
    let bytes = line.as_bytes();
    let mut cursor = start;
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    if bytes.get(cursor) == Some(&b'<') {
        cursor += 1;
        let begin = cursor;
        while let Some(byte) = bytes.get(cursor) {
            if *byte == b'>' && !escaped(bytes, cursor) {
                return Some((line[begin..cursor].to_owned(), cursor + 1));
            }
            cursor += 1;
        }
        return None;
    }

    let begin = cursor;
    let mut nested = 0usize;
    while let Some(byte) = bytes.get(cursor) {
        if *byte == b'\\' {
            cursor = cursor.saturating_add(2);
            continue;
        }
        if *byte == b'(' {
            nested += 1;
        } else if *byte == b')' {
            if nested == 0 && terminator == Some(b')') {
                break;
            }
            nested = nested.saturating_sub(1);
        } else if nested == 0
            && (byte.is_ascii_whitespace() || terminator.is_some_and(|value| *byte == value))
        {
            break;
        }
        cursor += 1;
    }
    Some((line[begin..cursor].to_owned(), cursor))
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

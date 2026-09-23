use super::{XmlSourceSpan, span};
use crate::load::{budget, invalid};
use crate::{ProjectPhase, ProjectResult};
use serde::Serialize;
use std::sync::Arc;
use wow_core::{CanonicalResult, ContentDigest, SourceContent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlLuaMapKind {
    Identity,
    XmlNewline,
    XmlEntity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaMapSegment {
    pub lua_byte_start: u64,
    pub lua_byte_end: u64,
    pub xml_span: XmlSourceSpan,
    pub kind: XmlLuaMapKind,
}

/// Extracted XML character data without a synthetic function wrapper or inferred
/// callback parameters. Analyzer registration requires a separate virtual-unit adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlInlineLua {
    pub unit_id: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
    segments: Vec<XmlLuaMapSegment>,
    #[serde(skip)]
    text: String,
    #[serde(skip)]
    lines: Arc<Vec<usize>>,
}
impl XmlInlineLua {
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
    #[must_use]
    pub fn segments(&self) -> &[XmlLuaMapSegment] {
        &self.segments
    }

    /// Map a nonempty UTF-8 Lua range to exact XML source pieces. XML comments
    /// and CDATA delimiters remain gaps; entities map to their entire spelling.
    /// Empty/caret ranges are intentionally not assigned a guessed adjacent span.
    pub fn map_range(&self, start: usize, end: usize) -> ProjectResult<Vec<XmlSourceSpan>> {
        if start >= end
            || end > self.text.len()
            || !self.text.is_char_boundary(start)
            || !self.text.is_char_boundary(end)
        {
            return Err(invalid("invalid nonempty inline Lua range"));
        }
        let mut result = Vec::new();
        let first = self
            .segments
            .partition_point(|segment| segment.lua_byte_end <= start as u64);
        for segment in self.segments[first..]
            .iter()
            .take_while(|segment| segment.lua_byte_start < end as u64)
        {
            let low = start.max(segment.lua_byte_start as usize);
            let high = end.min(segment.lua_byte_end as usize);
            if low >= high {
                continue;
            }
            if segment.kind == XmlLuaMapKind::Identity {
                let base = segment.xml_span.byte_start as usize;
                let lua = segment.lua_byte_start as usize;
                result.push(span(&self.lines, base + low - lua, base + high - lua));
            } else {
                result.push(segment.xml_span.clone());
            }
        }
        Ok(result)
    }

    /// Map a UTF-8 caret to every exact XML boundary. At a removed comment or
    /// CDATA delimiter the two neighboring boundaries stay separate candidates.
    /// Never widen the result across a gap or choose an arbitrary neighbor.
    pub fn map_position(&self, offset: usize) -> ProjectResult<Vec<XmlSourceSpan>> {
        if offset > self.text.len() || !self.text.is_char_boundary(offset) {
            return Err(invalid("invalid inline Lua caret"));
        }
        let offset = offset as u64;
        let first = self
            .segments
            .partition_point(|segment| segment.lua_byte_end < offset);
        let mut result = Vec::new();
        for segment in self.segments[first..]
            .iter()
            .take_while(|segment| segment.lua_byte_start <= offset)
        {
            let position = if segment.kind == XmlLuaMapKind::Identity {
                segment.xml_span.byte_start + offset - segment.lua_byte_start
            } else if offset == segment.lua_byte_start {
                segment.xml_span.byte_start
            } else if offset == segment.lua_byte_end {
                segment.xml_span.byte_end
            } else {
                return Err(invalid(
                    "inline Lua caret lies inside a transformed character",
                ));
            };
            let mapped = span(&self.lines, position as usize, position as usize);
            if result.last() != Some(&mapped) {
                result.push(mapped);
            }
        }
        if result.is_empty() {
            return Err(invalid("inline Lua caret has no source mapping"));
        }
        Ok(result)
    }
}

pub(super) struct InlineBuilder {
    text: String,
    segments: Vec<XmlLuaMapSegment>,
    lines: Arc<Vec<usize>>,
}
impl InlineBuilder {
    pub(super) fn new(lines: Arc<Vec<usize>>) -> Self {
        Self {
            text: String::new(),
            segments: Vec::new(),
            lines,
        }
    }
    /// XML 1.0 literal line endings normalize to LF; character references do not.
    pub(super) fn literal(&mut self, text: &str, source_start: usize) -> ProjectResult<()> {
        valid_characters(text)?;
        let bytes = text.as_bytes();
        let mut run = 0;
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] != b'\r' {
                index += 1;
                continue;
            }
            self.append(
                &text[run..index],
                source_start + run,
                source_start + index,
                XmlLuaMapKind::Identity,
            )?;
            let width = if bytes.get(index + 1) == Some(&b'\n') {
                2
            } else {
                1
            };
            self.append(
                "\n",
                source_start + index,
                source_start + index + width,
                XmlLuaMapKind::XmlNewline,
            )?;
            index += width;
            run = index;
        }
        self.append(
            &text[run..],
            source_start + run,
            source_start + text.len(),
            XmlLuaMapKind::Identity,
        )
    }
    pub(super) fn entity(&mut self, text: &str, start: usize, end: usize) -> ProjectResult<()> {
        valid_characters(text)?;
        self.append(text, start, end, XmlLuaMapKind::XmlEntity)
    }
    fn append(
        &mut self,
        text: &str,
        start: usize,
        end: usize,
        kind: XmlLuaMapKind,
    ) -> ProjectResult<()> {
        if text.is_empty() {
            return Ok(());
        }
        if self.text.len().saturating_add(text.len()) > crate::disk::DISK_SOURCE_MAX_BYTES {
            return Err(budget());
        }
        let lua_start = self.text.len() as u64;
        self.text.push_str(text);
        if let Some(last) = self.segments.last_mut()
            && kind == XmlLuaMapKind::Identity
            && last.kind == kind
            && last.xml_span.byte_end == start as u64
        {
            last.lua_byte_end = self.text.len() as u64;
            last.xml_span = span(&self.lines, last.xml_span.byte_start as usize, end);
            return Ok(());
        }
        if self.segments.len() >= 32_768 {
            return Err(budget());
        }
        self.segments.push(XmlLuaMapSegment {
            lua_byte_start: lua_start,
            lua_byte_end: self.text.len() as u64,
            xml_span: span(&self.lines, start, end),
            kind,
        });
        Ok(())
    }
    pub(super) fn nonempty(&self) -> bool {
        self.text
            .bytes()
            .any(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
    }
    pub(super) fn finish(self, owner: &str) -> ProjectResult<XmlInlineLua> {
        let content_digest = crate::identity::source_digest(self.text.as_bytes());
        let digest: ContentDigest<CanonicalResult> = crate::identity::canonical_digest(
            "wow-project/xml-inline-source/1",
            &(owner, content_digest, &self.segments),
            ProjectPhase::Inventory,
        )?;
        Ok(XmlInlineLua {
            unit_id: format!("xml-inline:{digest}"),
            content_digest,
            byte_length: self.text.len() as u64,
            segments: self.segments,
            text: self.text,
            lines: self.lines,
        })
    }
}

pub(super) fn valid_characters(value: &str) -> ProjectResult<()> {
    if value.chars().any(|c| !matches!(c, '\u{9}' | '\u{a}' | '\u{d}' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')) {
        return Err(invalid("XML contains a disallowed XML 1.0 character"));
    }
    Ok(())
}

//! Source-backed XML syntax index. Declarations are observations, not runtime
//! objects or accepted graph facts. Inline bodies are not physical Lua files.
mod builder;
mod inline;

pub(super) use builder::Builder;
pub use inline::{XmlInlineLua, XmlLuaMapKind, XmlLuaMapSegment};
use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, SourceContent};

pub const XML_INDEX_PROFILE: &str = "wow-project/xml-structure/1";

/// Zero-based half-open UTF-8 byte range, with one-based line and byte-column
/// coordinates. End coordinates denote the position immediately after the span.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSourceSpan {
    pub byte_start: u64,
    pub byte_end: u64,
    pub start_line: u64,
    pub start_byte_column: u64,
    pub end_line: u64,
    pub end_byte_column: u64,
}

pub(super) fn span(lines: &[usize], start: usize, end: usize) -> XmlSourceSpan {
    let first = lines
        .partition_point(|offset| *offset <= start)
        .saturating_sub(1);
    let last = lines
        .partition_point(|offset| *offset <= end)
        .saturating_sub(1);
    XmlSourceSpan {
        byte_start: start as u64,
        byte_end: end as u64,
        start_line: first as u64 + 1,
        start_byte_column: (start - lines[first]) as u64 + 1,
        end_line: last as u64 + 1,
        end_byte_column: (end - lines[last]) as u64 + 1,
    }
}

/// Attribute values remain accessible to owner consumers, but arbitrary source
/// strings are not copied into service output. Known declarations are projected below.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlAttributeRecord {
    pub qualified_name: String,
    pub span: XmlSourceSpan,
    pub value_span: XmlSourceSpan,
    pub decoded_value_digest: ContentDigest<SourceContent>,
    #[serde(skip)]
    value: String,
}
impl XmlAttributeRecord {
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlElementRole {
    Ui,
    Include,
    Script,
    Scripts,
    ScriptBinding,
    Element,
    UnknownNamespace,
}

/// Unresolved spelling references. Order and duplicates are intentional; a name
/// never selects a unique runtime object, template, or Lua symbol by itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlDeclaration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_template: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intrinsic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_array: Option<String>,
    pub inherits: Vec<String>,
    pub mixins: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlStructureIssue {
    InvalidDeclaration,
    AmbiguousScriptSource,
    NestedScriptMarkup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlScriptSource {
    ExternalFile,
    ReferenceOnly,
    InlineBody,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlScriptRecord {
    /// Containing XML declaration for a Scripts child; Ui for a top-level Script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_occurrence_id: Option<String>,
    pub source_kind: XmlScriptSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intrinsic_order: Option<String>,
    /// Exact XML representation between tags, including comments/CDATA delimiters.
    pub body_span: XmlSourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_lua: Option<XmlInlineLua>,
}

/// IDs address exact source occurrences within a document, not semantic object
/// identity across versions. Same-name declarations are never merged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlElementRecord {
    pub occurrence_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_occurrence_id: Option<String>,
    pub qualified_name: String,
    pub ui_namespace: bool,
    pub role: XmlElementRole,
    pub span: XmlSourceSpan,
    pub start_tag_span: XmlSourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_tag_span: Option<XmlSourceSpan>,
    pub attributes: Vec<XmlAttributeRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<XmlDeclaration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<XmlScriptRecord>,
    pub issues: Vec<XmlStructureIssue>,
}

/// One unique captured document, independent of how many include occurrences
/// load it. The enclosing ProjectLoadPlan supplies target/generation binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlDocumentIndex {
    profile: &'static str,
    document: String,
    source_digest: ContentDigest<SourceContent>,
    elements: Vec<XmlElementRecord>,
    digest: ContentDigest<CanonicalResult>,
}
impl XmlDocumentIndex {
    #[must_use]
    pub fn document(&self) -> &str {
        &self.document
    }
    #[must_use]
    pub fn elements(&self) -> &[XmlElementRecord] {
        &self.elements
    }
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
    #[must_use]
    pub fn element(&self, occurrence_id: &str) -> Option<&XmlElementRecord> {
        self.elements
            .iter()
            .find(|element| element.occurrence_id == occurrence_id)
    }
    pub fn declarations(&self) -> impl Iterator<Item = &XmlElementRecord> {
        self.elements
            .iter()
            .filter(|element| element.declaration.is_some())
    }
    pub fn scripts(&self) -> impl Iterator<Item = &XmlElementRecord> {
        self.elements
            .iter()
            .filter(|element| element.script.is_some())
    }
    pub(super) fn attribute_count(&self) -> usize {
        self.elements
            .iter()
            .map(|element| element.attributes.len())
            .sum()
    }
    pub(super) fn map_segment_count(&self) -> usize {
        self.scripts()
            .filter_map(|element| element.script.as_ref()?.inline_lua.as_ref())
            .map(|body| body.segments().len())
            .sum()
    }
}

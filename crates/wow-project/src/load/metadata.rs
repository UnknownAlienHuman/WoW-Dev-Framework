//! One normalized TOC metadata directive retained by the single source parser.
//!
//! [TocMetadata] is a lexical projection of a directive normalized by the load
//! owner's parser. The containing record retains conditional predicates and
//! selection, including unknown or excluded conditions. Downstream owners read
//! the retained key/value pair without re-splitting raw source spans.
//!
//! Retention does not grant semantic support to unknown directives.
use serde::Serialize;

/// One normalized TOC metadata directive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TocMetadata {
    /// Lowercased key, trimmed, with conditional clauses already stripped.
    pub key: String,
    /// Trimmed value, with conditional clauses already stripped.
    pub value: String,
}

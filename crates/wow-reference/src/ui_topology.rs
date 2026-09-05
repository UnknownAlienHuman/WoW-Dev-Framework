//! Validated import boundary for Blizzard TOC/XML topology drafts.
//!
//! The producer output is untrusted at this boundary. Import repeats canonical
//! identity, source, coverage, ordering, graph, and cycle checks before exact
//! load relationships are exposed to the rest of `wow-reference`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::wire_json::canonical_json_bytes;
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const SCHEMA: &str = "wow-dev-framework/blizzard-ui-topology-draft";
const SCHEMA_VERSION: u64 = 1;
const PRODUCER_ID: &str = "blizzard-ui-topology";
const INTERFACE_ROOT: &str = "Interface/";
const MAX_DRAFT_BYTES: usize = 1024 * 1024 * 1024;
const REFERENCE_ISSUES: [&str; 4] = [
    "invalid_reference",
    "missing_target",
    "case_mismatch",
    "ambiguous_case",
];

/// Stable topology import failure class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UiTopologyImportErrorCode {
    InputTooLarge,
    InvalidJson,
    UnsupportedSchema,
    InvalidProducer,
    InvalidDigest,
    InvalidSource,
    InvalidCoverage,
    InvalidOrdering,
    InvalidDocument,
    InvalidEdge,
    InvalidIssue,
    InvalidCycle,
}

/// One bounded topology import failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTopologyImportError {
    code: UiTopologyImportErrorCode,
    message: String,
}

impl UiTopologyImportError {
    fn new(code: UiTopologyImportErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Stable error class.
    #[must_use]
    pub const fn code(&self) -> UiTopologyImportErrorCode {
        self.code
    }

    /// Safe explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for UiTopologyImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for UiTopologyImportError {}

/// Result type for TOC/XML topology imports.
pub type UiTopologyImportResult<T> = Result<T, UiTopologyImportError>;

/// Kind of one exact load edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLoadEdgeKind {
    TocLoad,
    XmlInclude,
    XmlScript,
}

impl UiLoadEdgeKind {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "toc_load" => Some(Self::TocLoad),
            "xml_include" => Some(Self::XmlInclude),
            "xml_script" => Some(Self::XmlScript),
            _ => None,
        }
    }
}

/// Resolution state of one declared file reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UiReferenceResolution {
    Exact,
    Invalid,
    Missing,
    CaseMismatch,
    AmbiguousCase,
}

impl UiReferenceResolution {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "exact" => Some(Self::Exact),
            "invalid" => Some(Self::Invalid),
            "missing" => Some(Self::Missing),
            "case_mismatch" => Some(Self::CaseMismatch),
            "ambiguous_case" => Some(Self::AmbiguousCase),
            _ => None,
        }
    }
}

/// Coverage state of the imported TOC/XML corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTopologyCoverageStatus {
    Complete,
    Partial,
}

/// Exact source identity attached to a descriptor or XML document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiTopologySource {
    path: String,
    line_start: u64,
    line_end: u64,
    git_object: String,
    sha256: String,
}

impl UiTopologySource {
    /// Canonical repository-relative source path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Inclusive one-based start line.
    #[must_use]
    pub const fn line_start(&self) -> u64 {
        self.line_start
    }

    /// Inclusive one-based end line.
    #[must_use]
    pub const fn line_end(&self) -> u64 {
        self.line_end
    }

    /// Exact Git blob object identifier.
    #[must_use]
    pub fn git_object(&self) -> &str {
        &self.git_object
    }

    /// SHA-256 identity of the source bytes.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Exact source and producer identity retained by the import.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UiTopologyProvenance {
    manifest_sha256: String,
    manifest_declared_digest: Option<String>,
    source_id: Option<String>,
    selector: Option<String>,
    revision: String,
    version: Option<String>,
    producer_version: u64,
    configuration: BTreeMap<String, Value>,
}

impl UiTopologyProvenance {
    /// Digest of the exact source manifest bytes consumed by the producer.
    #[must_use]
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }

    /// Producer-declared manifest digest, when supplied.
    #[must_use]
    pub fn manifest_declared_digest(&self) -> Option<&str> {
        self.manifest_declared_digest.as_deref()
    }

    /// Public source identifier, when supplied.
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }

    /// Moving selector resolved for this operation, when supplied.
    #[must_use]
    pub fn selector(&self) -> Option<&str> {
        self.selector.as_deref()
    }

    /// Exact source revision used for every imported document.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Source-reported client version, when supplied.
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Producer implementation version.
    #[must_use]
    pub const fn producer_version(&self) -> u64 {
        self.producer_version
    }

    /// Producer configuration participating in imported identity.
    #[must_use]
    pub const fn configuration(&self) -> &BTreeMap<String, Value> {
        &self.configuration
    }
}

/// One immutable TOC descriptor.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UiTocDescriptor {
    path: String,
    addon_directory: String,
    descriptor_name: String,
    source: UiTopologySource,
    normalized: Value,
}

impl UiTocDescriptor {
    /// Canonical descriptor path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Directory containing the descriptor.
    #[must_use]
    pub fn addon_directory(&self) -> &str {
        &self.addon_directory
    }

    /// Descriptor file stem.
    #[must_use]
    pub fn descriptor_name(&self) -> &str {
        &self.descriptor_name
    }

    /// Exact descriptor source identity.
    #[must_use]
    pub const fn source(&self) -> &UiTopologySource {
        &self.source
    }

    /// Complete normalized descriptor payload.
    #[must_use]
    pub const fn normalized(&self) -> &Value {
        &self.normalized
    }
}

/// One immutable XML topology document.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UiXmlDocument {
    path: String,
    root: String,
    element_count: u64,
    inline_scripts: u64,
    source: UiTopologySource,
    normalized: Value,
}

impl UiXmlDocument {
    /// Canonical XML path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Local root element name.
    #[must_use]
    pub fn root(&self) -> &str {
        &self.root
    }

    /// Parsed element count.
    #[must_use]
    pub const fn element_count(&self) -> u64 {
        self.element_count
    }

    /// Inline `<Script>` element count. Contents remain non-executed data.
    #[must_use]
    pub const fn inline_scripts(&self) -> u64 {
        self.inline_scripts
    }

    /// Exact XML source identity.
    #[must_use]
    pub const fn source(&self) -> &UiTopologySource {
        &self.source
    }

    /// Complete normalized XML topology payload.
    #[must_use]
    pub const fn normalized(&self) -> &Value {
        &self.normalized
    }
}

/// One exact declared load edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiLoadEdge {
    kind: UiLoadEdgeKind,
    source: String,
    target: Option<String>,
    declared: String,
    resolution: UiReferenceResolution,
    line: u64,
}

impl UiLoadEdge {
    /// Edge kind.
    #[must_use]
    pub const fn kind(&self) -> UiLoadEdgeKind {
        self.kind
    }

    /// Descriptor or XML source path.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Canonical resolved target, when a target could be normalized.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// Original source declaration, not a validated path.
    ///
    /// Invalid declarations retain their diagnostic text, including whitespace
    /// and controls. Use `target` and `resolution` for navigation; escape raw
    /// declarations when rendering them outside structured JSON.
    #[must_use]
    pub fn declared(&self) -> &str {
        &self.declared
    }

    /// Resolution state.
    #[must_use]
    pub const fn resolution(&self) -> UiReferenceResolution {
        self.resolution
    }

    /// One-based source line.
    #[must_use]
    pub const fn line(&self) -> u64 {
        self.line
    }
}

/// One producer-preserved topology issue.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UiTopologyIssue {
    code: String,
    source_path: String,
    line: u64,
    normalized: Value,
}

impl UiTopologyIssue {
    /// Stable issue code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Affected source path.
    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    /// One-based source line, or zero when the issue is descriptor-wide.
    #[must_use]
    pub const fn line(&self) -> u64 {
        self.line
    }

    /// Complete normalized issue payload.
    #[must_use]
    pub const fn normalized(&self) -> &Value {
        &self.normalized
    }
}

/// Validated TOC/XML coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiTopologyCoverage {
    status: UiTopologyCoverageStatus,
    negative_authority: bool,
    candidate_toc_files: u64,
    parsed_toc_files: u64,
    candidate_xml_files: u64,
    parsed_xml_files: u64,
    failed_files: u64,
    unresolved_references: u64,
    failures: Vec<Value>,
    limitations: Vec<String>,
}

impl UiTopologyCoverage {
    /// Coverage status.
    #[must_use]
    pub const fn status(&self) -> UiTopologyCoverageStatus {
        self.status
    }

    /// Whether absence is authoritative for the exact TOC/XML reference scope.
    #[must_use]
    pub const fn negative_authority(&self) -> bool {
        self.negative_authority
    }

    /// Candidate TOC count.
    #[must_use]
    pub const fn candidate_toc_files(&self) -> u64 {
        self.candidate_toc_files
    }

    /// Parsed TOC count.
    #[must_use]
    pub const fn parsed_toc_files(&self) -> u64 {
        self.parsed_toc_files
    }

    /// Candidate XML count.
    #[must_use]
    pub const fn candidate_xml_files(&self) -> u64 {
        self.candidate_xml_files
    }

    /// Parsed XML count.
    #[must_use]
    pub const fn parsed_xml_files(&self) -> u64 {
        self.parsed_xml_files
    }

    /// Failed candidate count.
    #[must_use]
    pub const fn failed_files(&self) -> u64 {
        self.failed_files
    }

    /// Unresolved local file-reference count.
    #[must_use]
    pub const fn unresolved_references(&self) -> u64 {
        self.unresolved_references
    }

    /// Producer failures retained by partial coverage.
    #[must_use]
    pub fn failures(&self) -> &[Value] {
        &self.failures
    }

    /// Explicit evidence limitations.
    #[must_use]
    pub fn limitations(&self) -> &[String] {
        &self.limitations
    }
}

/// Exact document lookup outcome.
#[derive(Debug, Clone, PartialEq)]
pub enum UiTopologyDocumentLookup<'a> {
    Toc(&'a UiTocDescriptor),
    Xml(&'a UiXmlDocument),
    AbsentAuthoritative,
    NotAuthoritative,
    OutOfScope,
}

/// Immutable, independently validated TOC/XML topology index.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UiTopologyIndex {
    index_id: String,
    topology_sha256: String,
    provenance: UiTopologyProvenance,
    coverage: UiTopologyCoverage,
    descriptors: Vec<UiTocDescriptor>,
    xml_documents: Vec<UiXmlDocument>,
    edges: Vec<UiLoadEdge>,
    issues: Vec<UiTopologyIssue>,
    include_cycles: Vec<Vec<String>>,
}

impl UiTopologyIndex {
    /// Content-addressed imported index identity.
    #[must_use]
    pub fn index_id(&self) -> &str {
        &self.index_id
    }

    /// Validated producer draft digest.
    #[must_use]
    pub fn topology_sha256(&self) -> &str {
        &self.topology_sha256
    }

    /// Exact producer and source provenance.
    #[must_use]
    pub const fn provenance(&self) -> &UiTopologyProvenance {
        &self.provenance
    }

    /// Validated coverage.
    #[must_use]
    pub const fn coverage(&self) -> &UiTopologyCoverage {
        &self.coverage
    }

    /// Canonically ordered TOC descriptors.
    #[must_use]
    pub fn descriptors(&self) -> &[UiTocDescriptor] {
        &self.descriptors
    }

    /// Canonically ordered XML documents.
    #[must_use]
    pub fn xml_documents(&self) -> &[UiXmlDocument] {
        &self.xml_documents
    }

    /// Canonically ordered load edges.
    #[must_use]
    pub fn edges(&self) -> &[UiLoadEdge] {
        &self.edges
    }

    /// Canonically ordered issues.
    #[must_use]
    pub fn issues(&self) -> &[UiTopologyIssue] {
        &self.issues
    }

    /// Canonically ordered representative XML include cycles.
    #[must_use]
    pub fn include_cycles(&self) -> &[Vec<String>] {
        &self.include_cycles
    }

    /// Returns all exact outgoing load declarations for one source path.
    #[must_use]
    pub fn outgoing(&self, source: &str) -> Vec<&UiLoadEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.source == source)
            .collect()
    }

    /// Performs one exact case-sensitive descriptor/XML document lookup.
    #[must_use]
    pub fn lookup_document(&self, path: &str) -> UiTopologyDocumentLookup<'_> {
        if !path.starts_with(INTERFACE_ROOT) || !(path.ends_with(".toc") || path.ends_with(".xml"))
        {
            return UiTopologyDocumentLookup::OutOfScope;
        }
        if let Some(descriptor) = self.descriptors.iter().find(|item| item.path == path) {
            return UiTopologyDocumentLookup::Toc(descriptor);
        }
        if let Some(document) = self.xml_documents.iter().find(|item| item.path == path) {
            return UiTopologyDocumentLookup::Xml(document);
        }
        if self.coverage.negative_authority {
            UiTopologyDocumentLookup::AbsentAuthoritative
        } else {
            UiTopologyDocumentLookup::NotAuthoritative
        }
    }
}

/// Imports one topology draft after independently validating it.
pub fn import_ui_topology_draft(bytes: &[u8]) -> UiTopologyImportResult<UiTopologyIndex> {
    if bytes.len() > MAX_DRAFT_BYTES {
        return Err(failure(
            UiTopologyImportErrorCode::InputTooLarge,
            "topology draft exceeds the import limit",
        ));
    }
    let value = serde_json::from_slice::<Value>(bytes).map_err(|source| {
        failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("topology draft is not valid JSON: {source}"),
        )
    })?;
    let root = object(&value, "topology root")?;
    allowed_keys(
        root,
        &[
            "schema",
            "schema_version",
            "producer",
            "source",
            "coverage",
            "descriptors",
            "xml_documents",
            "edges",
            "issues",
            "include_cycles",
            "topology_sha256",
        ],
        "topology root",
    )?;
    if text(root, "schema", "topology schema")? != SCHEMA
        || unsigned(root, "schema_version", "topology schema version")? != SCHEMA_VERSION
    {
        return Err(failure(
            UiTopologyImportErrorCode::UnsupportedSchema,
            "unsupported topology draft schema",
        ));
    }

    let supplied_digest = text(root, "topology_sha256", "topology digest")?.to_owned();
    canonical_sha256(&supplied_digest, "topology digest")?;
    let mut projection = value.clone();
    object_mut(&mut projection, "topology projection")?.remove("topology_sha256");
    let projection_bytes = canonical_json_bytes(&projection).map_err(|source| {
        failure(
            UiTopologyImportErrorCode::InvalidDigest,
            format!("topology draft cannot be canonicalized: {source}"),
        )
    })?;
    if supplied_digest != sha256(&projection_bytes) {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidDigest,
            "topology digest does not match its content",
        ));
    }

    let producer = object(required(root, "producer", "producer")?, "producer")?;
    allowed_keys(producer, &["id", "version", "configuration"], "producer")?;
    if text(producer, "id", "producer id")? != PRODUCER_ID {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidProducer,
            "unexpected topology producer",
        ));
    }
    let producer_version = unsigned(producer, "version", "producer version")?;
    if producer_version == 0 {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidProducer,
            "topology producer version must be positive",
        ));
    }
    let configuration = value_map(object(
        required(producer, "configuration", "producer configuration")?,
        "producer configuration",
    )?);

    let source = object(required(root, "source", "source")?, "source")?;
    allowed_keys(
        source,
        &[
            "manifest_sha256",
            "manifest_declared_digest",
            "source_id",
            "selector",
            "revision",
            "version",
        ],
        "source",
    )?;
    let manifest_sha256 = text(source, "manifest_sha256", "manifest digest")?.to_owned();
    canonical_sha256(&manifest_sha256, "manifest digest")?;
    let manifest_declared_digest = optional_text(source, "manifest_declared_digest")?;
    if let Some(digest) = manifest_declared_digest.as_deref() {
        flexible_sha256(digest, "declared manifest digest")?;
    }
    let revision = text(source, "revision", "source revision")?.to_owned();
    object_id(&revision, None, "source revision")?;
    let provenance = UiTopologyProvenance {
        manifest_sha256,
        manifest_declared_digest,
        source_id: optional_text(source, "source_id")?,
        selector: optional_text(source, "selector")?,
        revision: revision.clone(),
        version: optional_text(source, "version")?,
        producer_version,
        configuration,
    };

    let descriptor_values = array(required(root, "descriptors", "descriptors")?, "descriptors")?;
    let xml_values = array(
        required(root, "xml_documents", "XML documents")?,
        "XML documents",
    )?;
    validate_document_order(descriptor_values, "path", "descriptors")?;
    validate_document_order(xml_values, "path", "XML documents")?;

    let mut descriptors = Vec::with_capacity(descriptor_values.len());
    let mut descriptor_paths = BTreeSet::new();
    let mut reference_multiset = BTreeMap::<EdgeKey, u64>::new();
    for value in descriptor_values {
        let descriptor = parse_descriptor(value, &revision, &mut reference_multiset)?;
        if !descriptor_paths.insert(descriptor.path.clone()) {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidOrdering,
                "duplicate TOC descriptor path",
            ));
        }
        descriptors.push(descriptor);
    }

    let mut xml_documents = Vec::with_capacity(xml_values.len());
    let mut xml_paths = BTreeSet::new();
    for value in xml_values {
        let document = parse_xml_document(value, &revision, &mut reference_multiset)?;
        if !xml_paths.insert(document.path.clone()) {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidOrdering,
                "duplicate XML document path",
            ));
        }
        xml_documents.push(document);
    }
    if !descriptor_paths.is_disjoint(&xml_paths) {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidDocument,
            "one path is both a TOC descriptor and XML document",
        ));
    }

    let mut document_paths = descriptor_paths.clone();
    document_paths.extend(xml_paths.iter().cloned());
    let edge_values = array(required(root, "edges", "edges")?, "edges")?;
    let edges = parse_edges(edge_values, &document_paths)?;
    let edge_multiset = edges.iter().fold(BTreeMap::new(), |mut counts, edge| {
        let current = counts.entry(EdgeKey::from_edge(edge)).or_insert(0_u64);
        *current = current.saturating_add(1);
        counts
    });
    if edge_multiset != reference_multiset {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidEdge,
            "descriptor/XML references do not match the common edge graph",
        ));
    }

    let issue_values = array(required(root, "issues", "issues")?, "issues")?;
    let issues = parse_issues(issue_values)?;
    validate_reference_issues(&edges, &issues)?;
    let unresolved_references = u64::try_from(
        issues
            .iter()
            .filter(|issue| REFERENCE_ISSUES.contains(&issue.code.as_str()))
            .count(),
    )
    .map_err(|_| {
        failure(
            UiTopologyImportErrorCode::InvalidCoverage,
            "unresolved reference count exceeds u64",
        )
    })?;

    let coverage = parse_coverage(
        required(root, "coverage", "coverage")?,
        descriptors.len(),
        xml_documents.len(),
        unresolved_references,
    )?;

    let cycle_values = array(
        required(root, "include_cycles", "include cycles")?,
        "include cycles",
    )?;
    let include_cycles = parse_cycles(cycle_values, &xml_paths, &edges)?;

    #[derive(Serialize)]
    struct IndexProjection<'a> {
        topology_sha256: &'a str,
        provenance: &'a UiTopologyProvenance,
        coverage: &'a UiTopologyCoverage,
        descriptors: &'a [UiTocDescriptor],
        xml_documents: &'a [UiXmlDocument],
        edges: &'a [UiLoadEdge],
        issues: &'a [UiTopologyIssue],
        include_cycles: &'a [Vec<String>],
    }
    let index_bytes = canonical_json_bytes(&IndexProjection {
        topology_sha256: &supplied_digest,
        provenance: &provenance,
        coverage: &coverage,
        descriptors: &descriptors,
        xml_documents: &xml_documents,
        edges: &edges,
        issues: &issues,
        include_cycles: &include_cycles,
    })
    .map_err(|source| {
        failure(
            UiTopologyImportErrorCode::InvalidDigest,
            format!("topology index cannot be canonicalized: {source}"),
        )
    })?;

    Ok(UiTopologyIndex {
        index_id: sha256(&index_bytes),
        topology_sha256: supplied_digest,
        provenance,
        coverage,
        descriptors,
        xml_documents,
        edges,
        issues,
        include_cycles,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeKey {
    kind: UiLoadEdgeKind,
    source: String,
    target: Option<String>,
    declared: String,
    resolution: UiReferenceResolution,
    line: u64,
}

impl EdgeKey {
    fn from_edge(edge: &UiLoadEdge) -> Self {
        Self {
            kind: edge.kind,
            source: edge.source.clone(),
            target: edge.target.clone(),
            declared: edge.declared.clone(),
            resolution: edge.resolution,
            line: edge.line,
        }
    }
}

fn parse_descriptor(
    value: &Value,
    revision: &str,
    references: &mut BTreeMap<EdgeKey, u64>,
) -> UiTopologyImportResult<UiTocDescriptor> {
    let descriptor = object(value, "TOC descriptor")?;
    allowed_keys(
        descriptor,
        &[
            "path",
            "addon_directory",
            "descriptor_name",
            "metadata",
            "entries",
            "source",
        ],
        "TOC descriptor",
    )?;
    let path = nonempty_text(descriptor, "path", "descriptor path")?.to_owned();
    interface_path(&path, Some("toc"), "descriptor path")?;
    let addon_directory =
        nonempty_text(descriptor, "addon_directory", "addon directory")?.to_owned();
    let descriptor_name =
        nonempty_text(descriptor, "descriptor_name", "descriptor name")?.to_owned();
    let source = parse_source(
        required(descriptor, "source", "descriptor source")?,
        revision,
        &path,
    )?;
    object(
        required(descriptor, "metadata", "TOC metadata")?,
        "TOC metadata",
    )?;
    let entries = array(
        required(descriptor, "entries", "TOC entries")?,
        "TOC entries",
    )?;
    let mut previous_line = 0_u64;
    for entry_value in entries {
        let entry = object(entry_value, "TOC entry")?;
        allowed_keys(
            entry,
            &[
                "declared",
                "target",
                "resolution",
                "candidate",
                "candidates",
                "kind",
                "line",
            ],
            "TOC entry",
        )?;
        let line = unsigned(entry, "line", "TOC entry line")?;
        if line == 0 || line <= previous_line {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidOrdering,
                "TOC entries are not strictly source ordered",
            ));
        }
        previous_line = line;
        let key = parse_reference_key(UiLoadEdgeKind::TocLoad, &path, entry)?;
        increment(references, key)?;
    }
    Ok(UiTocDescriptor {
        path,
        addon_directory,
        descriptor_name,
        source,
        normalized: value.clone(),
    })
}

fn parse_xml_document(
    value: &Value,
    revision: &str,
    references: &mut BTreeMap<EdgeKey, u64>,
) -> UiTopologyImportResult<UiXmlDocument> {
    let document = object(value, "XML document")?;
    allowed_keys(
        document,
        &[
            "path",
            "root",
            "element_count",
            "inline_scripts",
            "references",
            "templates",
            "source",
        ],
        "XML document",
    )?;
    let path = nonempty_text(document, "path", "XML path")?.to_owned();
    interface_path(&path, Some("xml"), "XML path")?;
    let root = nonempty_text(document, "root", "XML root")?.to_owned();
    let element_count = unsigned(document, "element_count", "XML element count")?;
    if element_count == 0 {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidDocument,
            "XML element count must be positive",
        ));
    }
    let inline_scripts = unsigned(document, "inline_scripts", "inline script count")?;
    let source = parse_source(required(document, "source", "XML source")?, revision, &path)?;
    array(
        required(document, "templates", "XML templates")?,
        "XML templates",
    )?;
    let reference_values = array(
        required(document, "references", "XML references")?,
        "XML references",
    )?;
    let mut previous_line = 0_u64;
    for reference_value in reference_values {
        let reference = object(reference_value, "XML reference")?;
        allowed_keys(
            reference,
            &[
                "kind",
                "declared",
                "line",
                "target",
                "resolution",
                "candidate",
                "candidates",
            ],
            "XML reference",
        )?;
        let kind = UiLoadEdgeKind::parse(text(reference, "kind", "XML reference kind")?)
            .filter(|candidate| {
                matches!(
                    candidate,
                    UiLoadEdgeKind::XmlInclude | UiLoadEdgeKind::XmlScript
                )
            })
            .ok_or_else(|| {
                failure(
                    UiTopologyImportErrorCode::InvalidEdge,
                    "XML reference kind is invalid",
                )
            })?;
        let line = unsigned(reference, "line", "XML reference line")?;
        if line == 0 || line < previous_line {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidOrdering,
                "XML references are not source ordered",
            ));
        }
        previous_line = line;
        let key = parse_reference_key(kind, &path, reference)?;
        increment(references, key)?;
    }
    Ok(UiXmlDocument {
        path,
        root,
        element_count,
        inline_scripts,
        source,
        normalized: value.clone(),
    })
}

fn parse_reference_key(
    kind: UiLoadEdgeKind,
    source: &str,
    record: &Map<String, Value>,
) -> UiTopologyImportResult<EdgeKey> {
    let declared = text(record, "declared", "declared reference")?.to_owned();
    let resolution =
        UiReferenceResolution::parse(text(record, "resolution", "reference resolution")?)
            .ok_or_else(|| {
                failure(
                    UiTopologyImportErrorCode::InvalidEdge,
                    "reference resolution is invalid",
                )
            })?;
    // `declared` is source evidence, not a resolved path. The producer trims
    // surrounding whitespace before resolution and preserves invalid declarations.
    // Such diagnostics must remain importable without gaining a valid target.
    let trimmed = declared.trim();
    if resolution != UiReferenceResolution::Invalid
        && (trimmed.is_empty() || trimmed.chars().any(char::is_control))
    {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidEdge,
            "resolved reference contains an invalid declaration",
        ));
    }
    let target = optional_text(record, "target")?;
    match resolution {
        UiReferenceResolution::Invalid if target.is_some() => {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidEdge,
                "invalid reference unexpectedly has a target",
            ));
        }
        UiReferenceResolution::Invalid => {}
        _ => {
            let candidate = target.as_deref().ok_or_else(|| {
                failure(
                    UiTopologyImportErrorCode::InvalidEdge,
                    "normalized reference has no target",
                )
            })?;
            interface_path(candidate, None, "reference target")?;
        }
    }
    let line = unsigned(record, "line", "reference line")?;
    if line == 0 {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidEdge,
            "reference line must be positive",
        ));
    }
    Ok(EdgeKey {
        kind,
        source: source.to_owned(),
        target,
        declared,
        resolution,
        line,
    })
}

fn parse_edges(
    values: &[Value],
    document_paths: &BTreeSet<String>,
) -> UiTopologyImportResult<Vec<UiLoadEdge>> {
    let mut output = Vec::with_capacity(values.len());
    let mut previous: Option<EdgeSortKey> = None;
    for value in values {
        let edge = object(value, "load edge")?;
        allowed_keys(
            edge,
            &["kind", "source", "target", "declared", "resolution", "line"],
            "load edge",
        )?;
        let kind = UiLoadEdgeKind::parse(text(edge, "kind", "edge kind")?).ok_or_else(|| {
            failure(
                UiTopologyImportErrorCode::InvalidEdge,
                "load edge kind is invalid",
            )
        })?;
        let source = nonempty_text(edge, "source", "edge source")?.to_owned();
        if !document_paths.contains(&source) {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidEdge,
                "load edge source is not an imported descriptor or XML document",
            ));
        }
        let key = parse_reference_key(kind, &source, edge)?;
        let sort_key = EdgeSortKey::from_key(&key);
        if previous
            .as_ref()
            .is_some_and(|candidate| candidate > &sort_key)
        {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidOrdering,
                "load edges are not canonically ordered",
            ));
        }
        previous = Some(sort_key);
        output.push(UiLoadEdge {
            kind: key.kind,
            source: key.source,
            target: key.target,
            declared: key.declared,
            resolution: key.resolution,
            line: key.line,
        });
    }
    Ok(output)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeSortKey {
    source: Vec<u8>,
    line: u64,
    kind: UiLoadEdgeKind,
    declared: Vec<u8>,
    target: Vec<u8>,
}

impl EdgeSortKey {
    fn from_key(key: &EdgeKey) -> Self {
        Self {
            source: key.source.as_bytes().to_vec(),
            line: key.line,
            kind: key.kind,
            declared: key.declared.as_bytes().to_vec(),
            target: key
                .target
                .as_deref()
                .unwrap_or_default()
                .as_bytes()
                .to_vec(),
        }
    }
}

fn parse_issues(values: &[Value]) -> UiTopologyImportResult<Vec<UiTopologyIssue>> {
    let mut output = Vec::with_capacity(values.len());
    let mut previous: Option<IssueSortKey> = None;
    for value in values {
        let issue = object(value, "topology issue")?;
        let code = nonempty_text(issue, "code", "issue code")?.to_owned();
        let source_path = optional_text(issue, "source_path")?
            .or_else(|| optional_text(issue, "path").ok().flatten())
            .ok_or_else(|| {
                failure(
                    UiTopologyImportErrorCode::InvalidIssue,
                    "topology issue has no source path",
                )
            })?;
        interface_path(&source_path, None, "issue source path")?;
        let line = optional_unsigned(issue, "line")?.unwrap_or(0);
        let key = IssueSortKey {
            code: code.clone(),
            source_path: source_path.clone(),
            line,
            declared: optional_text(issue, "declared")?.unwrap_or_default(),
            target: optional_text(issue, "target")?.unwrap_or_default(),
            metadata_key: optional_text(issue, "key")?.unwrap_or_default(),
        };
        if previous.as_ref().is_some_and(|candidate| candidate > &key) {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidOrdering,
                "topology issues are not canonically ordered",
            ));
        }
        previous = Some(key);
        output.push(UiTopologyIssue {
            code,
            source_path,
            line,
            normalized: value.clone(),
        });
    }
    Ok(output)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct IssueSortKey {
    code: String,
    source_path: String,
    line: u64,
    declared: String,
    target: String,
    metadata_key: String,
}

// Every unresolved edge needs its matching diagnostic, and every reference
// diagnostic must describe an actual edge. Counts alone could otherwise turn
// an invalid source declaration into complete coverage after issue removal.
fn validate_reference_issues(
    edges: &[UiLoadEdge],
    issues: &[UiTopologyIssue],
) -> UiTopologyImportResult<()> {
    let mut expected = BTreeMap::new();
    for edge in edges {
        let code = match edge.resolution {
            UiReferenceResolution::Exact => continue,
            UiReferenceResolution::Invalid => "invalid_reference",
            UiReferenceResolution::Missing => "missing_target",
            UiReferenceResolution::CaseMismatch => "case_mismatch",
            UiReferenceResolution::AmbiguousCase => "ambiguous_case",
        };
        let key = (
            code.to_owned(),
            edge.source.clone(),
            edge.line,
            edge.declared.clone(),
            edge.target.clone(),
        );
        *expected.entry(key).or_insert(0_usize) += 1;
    }
    let mut actual = BTreeMap::new();
    for issue in issues {
        if !REFERENCE_ISSUES.contains(&issue.code.as_str()) {
            continue;
        }
        let record = object(&issue.normalized, "reference issue")?;
        let key = (
            issue.code.clone(),
            issue.source_path.clone(),
            issue.line,
            text(record, "declared", "issue declaration")?.to_owned(),
            optional_text(record, "target")?,
        );
        *actual.entry(key).or_insert(0_usize) += 1;
    }
    if actual != expected {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidIssue,
            "unresolved edges and reference diagnostics do not match",
        ));
    }
    Ok(())
}

fn parse_coverage(
    value: &Value,
    descriptors: usize,
    xml_documents: usize,
    unresolved: u64,
) -> UiTopologyImportResult<UiTopologyCoverage> {
    let coverage = object(value, "coverage")?;
    allowed_keys(
        coverage,
        &[
            "status",
            "negative_authority",
            "candidate_toc_files",
            "parsed_toc_files",
            "candidate_xml_files",
            "parsed_xml_files",
            "failed_files",
            "unresolved_references",
            "failures",
            "limitations",
        ],
        "coverage",
    )?;
    let candidate_toc_files = unsigned(coverage, "candidate_toc_files", "candidate TOC count")?;
    let parsed_toc_files = unsigned(coverage, "parsed_toc_files", "parsed TOC count")?;
    let candidate_xml_files = unsigned(coverage, "candidate_xml_files", "candidate XML count")?;
    let parsed_xml_files = unsigned(coverage, "parsed_xml_files", "parsed XML count")?;
    let failed_files = unsigned(coverage, "failed_files", "failed file count")?;
    let unresolved_references = unsigned(
        coverage,
        "unresolved_references",
        "unresolved reference count",
    )?;
    if parsed_toc_files
        != u64::try_from(descriptors).map_err(|_| {
            failure(
                UiTopologyImportErrorCode::InvalidCoverage,
                "descriptor count exceeds u64",
            )
        })?
        || parsed_xml_files
            != u64::try_from(xml_documents).map_err(|_| {
                failure(
                    UiTopologyImportErrorCode::InvalidCoverage,
                    "XML document count exceeds u64",
                )
            })?
        || unresolved_references != unresolved
        || candidate_toc_files < parsed_toc_files
        || candidate_xml_files < parsed_xml_files
        || candidate_toc_files
            .checked_sub(parsed_toc_files)
            .and_then(|toc| {
                candidate_xml_files
                    .checked_sub(parsed_xml_files)
                    .and_then(|xml| toc.checked_add(xml))
            })
            != Some(failed_files)
    {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidCoverage,
            "topology coverage counts are inconsistent",
        ));
    }
    let failures = array(
        required(coverage, "failures", "coverage failures")?,
        "coverage failures",
    )?;
    if failures.len()
        != usize::try_from(failed_files).map_err(|_| {
            failure(
                UiTopologyImportErrorCode::InvalidCoverage,
                "failure count exceeds usize",
            )
        })?
    {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidCoverage,
            "topology failure records do not match failure count",
        ));
    }
    let complete = failed_files == 0
        && candidate_toc_files == parsed_toc_files
        && candidate_xml_files == parsed_xml_files
        && unresolved_references == 0;
    let status = match text(coverage, "status", "coverage status")? {
        "complete" if complete => UiTopologyCoverageStatus::Complete,
        "partial" if !complete => UiTopologyCoverageStatus::Partial,
        _ => {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidCoverage,
                "topology coverage status does not match counts",
            ));
        }
    };
    let negative_authority = boolean(coverage, "negative_authority", "negative authority")?;
    if negative_authority != complete {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidCoverage,
            "topology negative authority does not match completeness",
        ));
    }
    let limitation_values = array(
        required(coverage, "limitations", "coverage limitations")?,
        "coverage limitations",
    )?;
    if limitation_values.is_empty() {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidCoverage,
            "topology limitations are missing",
        ));
    }
    let limitations = limitation_values
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                failure(
                    UiTopologyImportErrorCode::InvalidCoverage,
                    "topology limitation is not text",
                )
            })
        })
        .collect::<UiTopologyImportResult<Vec<_>>>()?;
    Ok(UiTopologyCoverage {
        status,
        negative_authority,
        candidate_toc_files,
        parsed_toc_files,
        candidate_xml_files,
        parsed_xml_files,
        failed_files,
        unresolved_references,
        failures: failures.to_vec(),
        limitations,
    })
}

fn parse_cycles(
    values: &[Value],
    xml_paths: &BTreeSet<String>,
    edges: &[UiLoadEdge],
) -> UiTopologyImportResult<Vec<Vec<String>>> {
    let includes = edges
        .iter()
        .filter(|edge| {
            edge.kind == UiLoadEdgeKind::XmlInclude
                && edge.resolution == UiReferenceResolution::Exact
        })
        .filter_map(|edge| {
            edge.target
                .as_ref()
                .map(|target| (edge.source.as_str(), target.as_str()))
        })
        .collect::<BTreeSet<_>>();
    let mut output = Vec::with_capacity(values.len());
    let mut previous: Option<Vec<Vec<u8>>> = None;
    for value in values {
        let node_values = array(value, "include cycle")?;
        if node_values.is_empty() {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidCycle,
                "include cycle is empty",
            ));
        }
        let mut nodes = Vec::with_capacity(node_values.len());
        let mut unique = BTreeSet::new();
        for node_value in node_values {
            let node = node_value.as_str().ok_or_else(|| {
                failure(
                    UiTopologyImportErrorCode::InvalidCycle,
                    "include cycle node is not text",
                )
            })?;
            interface_path(node, Some("xml"), "include cycle path")?;
            if !xml_paths.contains(node) || !unique.insert(node) {
                return Err(failure(
                    UiTopologyImportErrorCode::InvalidCycle,
                    "include cycle contains an unknown or repeated XML path",
                ));
            }
            nodes.push(node.to_owned());
        }
        for index in 0..nodes.len() {
            let next = (index + 1) % nodes.len();
            if !includes.contains(&(nodes[index].as_str(), nodes[next].as_str())) {
                return Err(failure(
                    UiTopologyImportErrorCode::InvalidCycle,
                    "include cycle is not supported by exact XML include edges",
                ));
            }
        }
        let canonical = canonical_cycle(&nodes);
        if canonical != nodes {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidOrdering,
                "include cycle is not canonically rotated",
            ));
        }
        let key = nodes
            .iter()
            .map(|node| node.as_bytes().to_vec())
            .collect::<Vec<_>>();
        if previous.as_ref().is_some_and(|candidate| candidate >= &key) {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidOrdering,
                "include cycles are not uniquely ordered",
            ));
        }
        previous = Some(key);
        output.push(nodes);
    }
    Ok(output)
}

fn canonical_cycle(nodes: &[String]) -> Vec<String> {
    let mut best = nodes.to_vec();
    for offset in 1..nodes.len() {
        let candidate = nodes[offset..]
            .iter()
            .chain(nodes[..offset].iter())
            .cloned()
            .collect::<Vec<_>>();
        if candidate
            .iter()
            .map(String::as_bytes)
            .cmp(best.iter().map(String::as_bytes))
            .is_lt()
        {
            best = candidate;
        }
    }
    best
}

fn parse_source(
    value: &Value,
    revision: &str,
    expected_path: &str,
) -> UiTopologyImportResult<UiTopologySource> {
    let source = object(value, "source identity")?;
    allowed_keys(
        source,
        &["path", "line_start", "line_end", "git_object", "sha256"],
        "source identity",
    )?;
    let path = text(source, "path", "source path")?.to_owned();
    if path != expected_path {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidSource,
            "source identity path does not match owning document",
        ));
    }
    interface_path(&path, None, "source path")?;
    let line_start = unsigned(source, "line_start", "source start line")?;
    let line_end = unsigned(source, "line_end", "source end line")?;
    if line_start == 0 || line_end < line_start {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidSource,
            "source line span is invalid",
        ));
    }
    let git_object = text(source, "git_object", "source Git object")?.to_owned();
    object_id(&git_object, Some(revision.len()), "source Git object")?;
    let sha256 = text(source, "sha256", "source digest")?.to_owned();
    canonical_sha256(&sha256, "source digest")?;
    Ok(UiTopologySource {
        path,
        line_start,
        line_end,
        git_object,
        sha256,
    })
}

fn validate_document_order(values: &[Value], key: &str, label: &str) -> UiTopologyImportResult<()> {
    let mut previous: Option<Vec<u8>> = None;
    for value in values {
        let record = object(value, label)?;
        let current = nonempty_text(record, key, label)?.as_bytes().to_vec();
        if previous
            .as_ref()
            .is_some_and(|candidate| candidate >= &current)
        {
            return Err(failure(
                UiTopologyImportErrorCode::InvalidOrdering,
                format!("{label} are not uniquely byte-sorted"),
            ));
        }
        previous = Some(current);
    }
    Ok(())
}

fn interface_path(
    value: &str,
    required_extension: Option<&str>,
    label: &str,
) -> UiTopologyImportResult<()> {
    if !value.starts_with(INTERFACE_ROOT)
        || value.starts_with('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || required_extension.is_some_and(|extension| {
            !value
                .rsplit_once('.')
                .is_some_and(|(_, suffix)| suffix.eq_ignore_ascii_case(extension))
        })
    {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidSource,
            format!("{label} is not a canonical Interface path"),
        ));
    }
    Ok(())
}

fn increment(counts: &mut BTreeMap<EdgeKey, u64>, key: EdgeKey) -> UiTopologyImportResult<()> {
    let current = counts.entry(key).or_insert(0);
    *current = current.checked_add(1).ok_or_else(|| {
        failure(
            UiTopologyImportErrorCode::InvalidEdge,
            "edge multiplicity overflow",
        )
    })?;
    Ok(())
}

fn canonical_sha256(value: &str, label: &str) -> UiTopologyImportResult<()> {
    let digest = value.strip_prefix("sha256:").ok_or_else(|| {
        failure(
            UiTopologyImportErrorCode::InvalidDigest,
            format!("{label} is not a canonical SHA-256 identifier"),
        )
    })?;
    raw_sha256(digest, label)
}

fn flexible_sha256(value: &str, label: &str) -> UiTopologyImportResult<()> {
    raw_sha256(value.strip_prefix("sha256:").unwrap_or(value), label)
}

fn raw_sha256(value: &str, label: &str) -> UiTopologyImportResult<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidDigest,
            format!("{label} is not a SHA-256 digest"),
        ));
    }
    Ok(())
}

fn object_id(
    value: &str,
    expected_length: Option<usize>,
    label: &str,
) -> UiTopologyImportResult<()> {
    if !matches!(value.len(), 40 | 64)
        || expected_length.is_some_and(|length| length != value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidSource,
            format!("{label} is not a canonical Git object identifier"),
        ));
    }
    Ok(())
}

fn allowed_keys(
    object: &Map<String, Value>,
    allowed: &[&str],
    label: &str,
) -> UiTopologyImportResult<()> {
    if let Some(unexpected) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(failure(
            UiTopologyImportErrorCode::UnsupportedSchema,
            format!("{label} contains unsupported field {unexpected:?}"),
        ));
    }
    Ok(())
}

fn object<'a>(value: &'a Value, label: &str) -> UiTopologyImportResult<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| {
        failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("{label} must be an object"),
        )
    })
}

fn object_mut<'a>(
    value: &'a mut Value,
    label: &str,
) -> UiTopologyImportResult<&'a mut Map<String, Value>> {
    value.as_object_mut().ok_or_else(|| {
        failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("{label} must be an object"),
        )
    })
}

fn array<'a>(value: &'a Value, label: &str) -> UiTopologyImportResult<&'a [Value]> {
    value.as_array().map(Vec::as_slice).ok_or_else(|| {
        failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("{label} must be an array"),
        )
    })
}

fn required<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> UiTopologyImportResult<&'a Value> {
    object.get(key).ok_or_else(|| {
        failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("{label} is missing"),
        )
    })
}

fn text<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> UiTopologyImportResult<&'a str> {
    required(object, key, label)?.as_str().ok_or_else(|| {
        failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("{label} must be text"),
        )
    })
}

fn nonempty_text<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> UiTopologyImportResult<&'a str> {
    let value = text(object, key, label)?;
    if value.is_empty() {
        return Err(failure(
            UiTopologyImportErrorCode::InvalidDocument,
            format!("{label} must not be empty"),
        ));
    }
    Ok(value)
}

fn optional_text(object: &Map<String, Value>, key: &str) -> UiTopologyImportResult<Option<String>> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("optional field {key:?} must be text or null"),
        )),
    }
}

fn unsigned(object: &Map<String, Value>, key: &str, label: &str) -> UiTopologyImportResult<u64> {
    required(object, key, label)?.as_u64().ok_or_else(|| {
        failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("{label} must be an unsigned integer"),
        )
    })
}

fn optional_unsigned(
    object: &Map<String, Value>,
    key: &str,
) -> UiTopologyImportResult<Option<u64>> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value.as_u64().map(Some).ok_or_else(|| {
            failure(
                UiTopologyImportErrorCode::InvalidJson,
                format!("optional field {key:?} must be an unsigned integer"),
            )
        }),
        Some(_) => Err(failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("optional field {key:?} must be an unsigned integer"),
        )),
    }
}

fn boolean(object: &Map<String, Value>, key: &str, label: &str) -> UiTopologyImportResult<bool> {
    required(object, key, label)?.as_bool().ok_or_else(|| {
        failure(
            UiTopologyImportErrorCode::InvalidJson,
            format!("{label} must be a boolean"),
        )
    })
}

fn value_map(object: &Map<String, Value>) -> BTreeMap<String, Value> {
    object
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn failure(code: UiTopologyImportErrorCode, message: impl Into<String>) -> UiTopologyImportError {
    UiTopologyImportError::new(code, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TOC: &str = "Interface/AddOns/Blizzard_Test/Blizzard_Test.toc";
    const XML: &str = "Interface/AddOns/Blizzard_Test/Main.xml";
    const LUA: &str = "Interface/AddOns/Blizzard_Test/Logic.lua";

    fn source(path: &str) -> Value {
        json!({
            "path": path,
            "line_start": 1,
            "line_end": 4,
            "git_object": "1111111111111111111111111111111111111111",
            "sha256": "sha256:2222222222222222222222222222222222222222222222222222222222222222"
        })
    }

    fn draft(complete: bool) -> Value {
        let resolution = if complete { "exact" } else { "missing" };
        let issue = if complete {
            Vec::<Value>::new()
        } else {
            vec![json!({
                "code": "missing_target",
                "source_path": TOC,
                "line": 3,
                "declared": "Logic.lua",
                "target": LUA,
                "message": "missing"
            })]
        };
        json!({
            "schema": SCHEMA,
            "schema_version": SCHEMA_VERSION,
            "producer": {
                "id": PRODUCER_ID,
                "version": 1,
                "configuration": {"scope": "Interface/**/*.toc+xml"}
            },
            "source": {
                "manifest_sha256": "sha256:3333333333333333333333333333333333333333333333333333333333333333",
                "manifest_declared_digest": null,
                "source_id": "public-source",
                "selector": "live",
                "revision": "4444444444444444444444444444444444444444",
                "version": "99.1.2.34567"
            },
            "coverage": {
                "status": if complete { "complete" } else { "partial" },
                "negative_authority": complete,
                "candidate_toc_files": 1,
                "parsed_toc_files": 1,
                "candidate_xml_files": 1,
                "parsed_xml_files": 1,
                "failed_files": 0,
                "unresolved_references": if complete { 0 } else { 1 },
                "failures": [],
                "limitations": ["runtime behavior remains separate"]
            },
            "descriptors": [{
                "path": TOC,
                "addon_directory": "Blizzard_Test",
                "descriptor_name": "Blizzard_Test",
                "metadata": {"Interface": [{"value": "99999", "line": 1}]},
                "entries": [{
                    "declared": "Logic.lua",
                    "target": LUA,
                    "resolution": resolution,
                    "kind": "lua",
                    "line": 3
                }],
                "source": source(TOC)
            }],
            "xml_documents": [{
                "path": XML,
                "root": "Ui",
                "element_count": 2,
                "inline_scripts": 0,
                "references": [],
                "templates": [],
                "source": source(XML)
            }],
            "edges": [{
                "kind": "toc_load",
                "source": TOC,
                "target": LUA,
                "declared": "Logic.lua",
                "resolution": resolution,
                "line": 3
            }],
            "issues": issue,
            "include_cycles": []
        })
    }

    fn seal(mut value: Value) -> UiTopologyImportResult<Vec<u8>> {
        let bytes = canonical_json_bytes(&value).map_err(|source| {
            failure(
                UiTopologyImportErrorCode::InvalidDigest,
                format!("test topology cannot be canonicalized: {source}"),
            )
        })?;
        object_mut(&mut value, "test topology")?
            .insert("topology_sha256".to_owned(), Value::String(sha256(&bytes)));
        serde_json::to_vec(&value).map_err(|source| {
            failure(
                UiTopologyImportErrorCode::InvalidJson,
                format!("test topology cannot be serialized: {source}"),
            )
        })
    }

    #[test]
    fn complete_topology_supports_exact_lookup_and_absence() -> UiTopologyImportResult<()> {
        let index = import_ui_topology_draft(&seal(draft(true))?)?;
        assert!(matches!(
            index.lookup_document(TOC),
            UiTopologyDocumentLookup::Toc(_)
        ));
        assert_eq!(index.outgoing(TOC).len(), 1);
        assert!(matches!(
            index.lookup_document("Interface/AddOns/Blizzard_Test/Missing.xml"),
            UiTopologyDocumentLookup::AbsentAuthoritative
        ));
        Ok(())
    }

    #[test]
    fn unresolved_reference_blocks_authoritative_absence() -> UiTopologyImportResult<()> {
        let index = import_ui_topology_draft(&seal(draft(false))?)?;
        assert!(!index.coverage().negative_authority());
        assert!(matches!(
            index.lookup_document("Interface/AddOns/Blizzard_Test/Missing.xml"),
            UiTopologyDocumentLookup::NotAuthoritative
        ));
        Ok(())
    }

    fn invalid_declaration(declared: &str) -> Value {
        let mut value = draft(false);
        value["descriptors"][0]["entries"][0]["declared"] = json!(declared);
        value["descriptors"][0]["entries"][0]["target"] = Value::Null;
        value["descriptors"][0]["entries"][0]["resolution"] = json!("invalid");
        value["edges"][0]["declared"] = json!(declared);
        value["edges"][0]["target"] = Value::Null;
        value["edges"][0]["resolution"] = json!("invalid");
        value["issues"][0]["declared"] = json!(declared);
        value["issues"][0]["target"] = Value::Null;
        value["issues"][0]["code"] = json!("invalid_reference");
        value
    }

    #[test]
    fn invalid_source_text_remains_diagnostic_not_a_target() -> UiTopologyImportResult<()> {
        for declared in ["", "   ", "Bad\nName.lua", "Bad\0Name.lua"] {
            let index = import_ui_topology_draft(&seal(invalid_declaration(declared))?)?;
            assert!(!index.coverage().negative_authority());
            let outgoing = index.outgoing(TOC);
            assert_eq!(outgoing.len(), 1);
            assert_eq!(outgoing[0].declared(), declared);
            assert_eq!(outgoing[0].target(), None);
            assert_eq!(outgoing[0].resolution(), UiReferenceResolution::Invalid);
            assert!(matches!(
                index.lookup_document("Interface/AddOns/Missing.xml"),
                UiTopologyDocumentLookup::NotAuthoritative
            ));
        }
        Ok(())
    }

    #[test]
    fn source_whitespace_is_preserved_around_resolved_reference() -> UiTopologyImportResult<()> {
        let mut value = draft(true);
        let declared = "\tLogic.lua\r\n";
        value["descriptors"][0]["entries"][0]["declared"] = json!(declared);
        value["edges"][0]["declared"] = json!(declared);
        let index = import_ui_topology_draft(&seal(value)?)?;
        let outgoing = index.outgoing(TOC);
        assert_eq!(outgoing[0].declared(), declared);
        assert_eq!(outgoing[0].target(), Some(LUA));
        Ok(())
    }

    #[test]
    fn invalid_diagnostics_cannot_be_removed_or_promoted() -> UiTopologyImportResult<()> {
        let mut hidden = invalid_declaration("Bad\nName.lua");
        hidden["issues"] = json!([]);
        hidden["coverage"]["unresolved_references"] = json!(0);
        hidden["coverage"]["status"] = json!("complete");
        hidden["coverage"]["negative_authority"] = json!(true);
        assert!(import_ui_topology_draft(&seal(hidden)?).is_err());

        let mut promoted = draft(true);
        promoted["descriptors"][0]["entries"][0]["declared"] = json!("Bad\nName.lua");
        promoted["edges"][0]["declared"] = json!("Bad\nName.lua");
        assert!(import_ui_topology_draft(&seal(promoted)?).is_err());

        let mut orphan = draft(false);
        orphan["issues"][0]["declared"] = json!("unrelated.lua");
        assert!(import_ui_topology_draft(&seal(orphan)?).is_err());
        Ok(())
    }

    #[test]
    fn edge_graph_must_match_descriptor_references() -> UiTopologyImportResult<()> {
        let mut value = draft(true);
        let edges = object_mut(&mut value, "test topology")?
            .get_mut("edges")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                failure(
                    UiTopologyImportErrorCode::InvalidJson,
                    "test edges are missing",
                )
            })?;
        edges.clear();
        let error = import_ui_topology_draft(&seal(value)?)
            .err()
            .ok_or_else(|| {
                failure(
                    UiTopologyImportErrorCode::InvalidEdge,
                    "inconsistent edge graph unexpectedly imported",
                )
            })?;
        assert_eq!(error.code(), UiTopologyImportErrorCode::InvalidEdge);
        Ok(())
    }

    #[test]
    fn tampering_breaks_self_digest() -> UiTopologyImportResult<()> {
        let bytes = seal(draft(true))?;
        let mut value = serde_json::from_slice::<Value>(&bytes).map_err(|source| {
            failure(
                UiTopologyImportErrorCode::InvalidJson,
                format!("test topology cannot be parsed: {source}"),
            )
        })?;
        object_mut(&mut value, "test topology")?
            .insert("producer".to_owned(), serde_json::json!({"tampered": true}));
        let tampered = serde_json::to_vec(&value).map_err(|source| {
            failure(
                UiTopologyImportErrorCode::InvalidJson,
                format!("test topology cannot be serialized: {source}"),
            )
        })?;
        let error = import_ui_topology_draft(&tampered).err().ok_or_else(|| {
            failure(
                UiTopologyImportErrorCode::InvalidDigest,
                "tampered topology unexpectedly imported",
            )
        })?;
        assert_eq!(error.code(), UiTopologyImportErrorCode::InvalidDigest);
        Ok(())
    }
}

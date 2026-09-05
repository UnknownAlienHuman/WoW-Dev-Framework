//! Validated import boundary for normalized Blizzard generated API documents.
//!
//! This module never executes Lua. It accepts only the normalized producer
//! schema, repeats identity and coverage checks, and exposes immutable exact
//! lookup facts for one operation-scoped source revision.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::wire_json::canonical_json_bytes;
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const SCHEMA: &str = "wow-dev-framework/blizzard-api-reference-draft";
const SCHEMA_VERSION: u64 = 1;
const PRODUCER_ID: &str = "blizzard-generated-api-reference";
const GENERATED_ROOT: &str = "Interface/AddOns/Blizzard_APIDocumentationGenerated/";
const GENERATED_SUFFIX: &str = "Documentation.lua";
const MAX_DRAFT_BYTES: usize = 512 * 1024 * 1024;

/// Stable generated API import failure class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GeneratedApiImportErrorCode {
    InputTooLarge,
    InvalidJson,
    UnsupportedSchema,
    InvalidProducer,
    InvalidDigest,
    InvalidSource,
    InvalidCoverage,
    InvalidOrdering,
    InvalidFact,
    InvalidConflict,
}

/// One generated API import failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiImportError {
    code: GeneratedApiImportErrorCode,
    message: String,
}

impl GeneratedApiImportError {
    fn new(code: GeneratedApiImportErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Stable error class.
    #[must_use]
    pub const fn code(&self) -> GeneratedApiImportErrorCode {
        self.code
    }

    /// Safe explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for GeneratedApiImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GeneratedApiImportError {}

/// Result type for generated API imports.
pub type GeneratedApiImportResult<T> = Result<T, GeneratedApiImportError>;

/// Normalized generated API entity kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedApiFactKind {
    Function,
    Event,
    Table,
    Enumeration,
    Constant,
    Predicate,
}

impl GeneratedApiFactKind {
    /// Canonical singular name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Event => "event",
            Self::Table => "table",
            Self::Enumeration => "enumeration",
            Self::Constant => "constant",
            Self::Predicate => "predicate",
        }
    }

    fn collection(self) -> &'static str {
        match self {
            Self::Function => "functions",
            Self::Event => "events",
            Self::Table => "tables",
            Self::Enumeration => "enumerations",
            Self::Constant => "constants",
            Self::Predicate => "predicates",
        }
    }

    fn all() -> [Self; 6] {
        [
            Self::Function,
            Self::Event,
            Self::Table,
            Self::Enumeration,
            Self::Constant,
            Self::Predicate,
        ]
    }
}

/// Validated generated API coverage state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedApiCoverageStatus {
    Complete,
    Partial,
}

/// Exact source span and content identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratedApiSourceSpan {
    path: String,
    line_start: u64,
    line_end: u64,
    git_object: String,
    sha256: String,
}

impl GeneratedApiSourceSpan {
    /// Canonical repository-relative path.
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

/// Exact producer and source identity retained by the import.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeneratedApiProvenance {
    manifest_sha256: String,
    manifest_declared_digest: Option<String>,
    source_id: Option<String>,
    selector: Option<String>,
    revision: String,
    version: Option<String>,
    producer_version: u64,
    parser: String,
    configuration: BTreeMap<String, Value>,
}

impl GeneratedApiProvenance {
    /// SHA-256 identity of the exact source manifest bytes.
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

    /// Moving selector resolved for the operation, when supplied.
    #[must_use]
    pub fn selector(&self) -> Option<&str> {
        self.selector.as_deref()
    }

    /// Exact source revision used for every imported fact.
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

    /// Declarative parser identity.
    #[must_use]
    pub fn parser(&self) -> &str {
        &self.parser
    }

    /// Producer configuration participating in index identity.
    #[must_use]
    pub const fn configuration(&self) -> &BTreeMap<String, Value> {
        &self.configuration
    }
}

/// One content-addressed generated API fact.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeneratedApiFact {
    fact_id: String,
    kind: GeneratedApiFactKind,
    name: String,
    qualified_name: String,
    member_type: Option<String>,
    source: GeneratedApiSourceSpan,
    normalized: Value,
}

impl GeneratedApiFact {
    /// Content-addressed fact identity.
    #[must_use]
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }

    /// Fact kind.
    #[must_use]
    pub const fn kind(&self) -> GeneratedApiFactKind {
        self.kind
    }

    /// Unqualified generated name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Exact case-sensitive lookup name.
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }

    /// Generated member type, when declared.
    #[must_use]
    pub fn member_type(&self) -> Option<&str> {
        self.member_type.as_deref()
    }

    /// Exact source span.
    #[must_use]
    pub const fn source(&self) -> &GeneratedApiSourceSpan {
        &self.source
    }

    /// Complete normalized producer payload for this member.
    #[must_use]
    pub const fn normalized(&self) -> &Value {
        &self.normalized
    }

    /// Generated restrictions object, when present.
    #[must_use]
    pub fn restrictions(&self) -> Option<&Map<String, Value>> {
        self.normalized
            .get("restrictions")
            .and_then(Value::as_object)
    }
}

/// One source locator retained by a duplicate-symbol conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratedApiConflictSource {
    path: String,
    line_start: u64,
}

impl GeneratedApiConflictSource {
    /// Canonical source path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// One-based source line.
    #[must_use]
    pub const fn line_start(&self) -> u64 {
        self.line_start
    }
}

/// One producer-preserved duplicate-symbol conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratedApiConflict {
    kind: String,
    collection: String,
    qualified_name: String,
    sources: Vec<GeneratedApiConflictSource>,
}

impl GeneratedApiConflict {
    /// Conflict kind.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Generated collection.
    #[must_use]
    pub fn collection(&self) -> &str {
        &self.collection
    }

    /// Affected exact lookup name.
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }

    /// Canonically ordered source locators.
    #[must_use]
    pub fn sources(&self) -> &[GeneratedApiConflictSource] {
        &self.sources
    }
}

/// One failure retained by partial producer coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratedApiFailure {
    code: String,
    message: String,
    path: Option<String>,
}

impl GeneratedApiFailure {
    /// Stable producer failure code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Safe failure explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Affected path, when known.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

/// Validated generated API coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratedApiCoverage {
    status: GeneratedApiCoverageStatus,
    negative_authority: bool,
    candidate_files: u64,
    parsed_files: u64,
    failed_files: u64,
    parsed_paths: Vec<String>,
    failures: Vec<GeneratedApiFailure>,
    entity_counts: BTreeMap<String, u64>,
    limitations: Vec<String>,
}

impl GeneratedApiCoverage {
    /// Coverage status.
    #[must_use]
    pub const fn status(&self) -> GeneratedApiCoverageStatus {
        self.status
    }

    /// Whether exact absence may be authoritative for this scope.
    #[must_use]
    pub const fn negative_authority(&self) -> bool {
        self.negative_authority
    }

    /// Candidate generated-document count.
    #[must_use]
    pub const fn candidate_files(&self) -> u64 {
        self.candidate_files
    }

    /// Parsed generated-document count.
    #[must_use]
    pub const fn parsed_files(&self) -> u64 {
        self.parsed_files
    }

    /// Failed generated-document count.
    #[must_use]
    pub const fn failed_files(&self) -> u64 {
        self.failed_files
    }

    /// Canonically ordered parsed paths.
    #[must_use]
    pub fn parsed_paths(&self) -> &[String] {
        &self.parsed_paths
    }

    /// Partial-coverage failures.
    #[must_use]
    pub fn failures(&self) -> &[GeneratedApiFailure] {
        &self.failures
    }

    /// Entity counts by generated collection.
    #[must_use]
    pub const fn entity_counts(&self) -> &BTreeMap<String, u64> {
        &self.entity_counts
    }

    /// Explicit limitations retained from the producer.
    #[must_use]
    pub fn limitations(&self) -> &[String] {
        &self.limitations
    }
}

/// Exact lookup outcome.
#[derive(Debug, Clone, PartialEq)]
pub enum GeneratedApiLookup<'a> {
    Found(&'a GeneratedApiFact),
    Conflicted(Vec<&'a GeneratedApiFact>),
    AbsentAuthoritative,
    NotAuthoritative,
}

/// Immutable generated API index for one exact source generation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeneratedApiIndex {
    index_id: String,
    draft_sha256: String,
    provenance: GeneratedApiProvenance,
    coverage: GeneratedApiCoverage,
    conflicts: Vec<GeneratedApiConflict>,
    facts: Vec<GeneratedApiFact>,
}

impl GeneratedApiIndex {
    /// Content-addressed imported index identity.
    #[must_use]
    pub fn index_id(&self) -> &str {
        &self.index_id
    }

    /// Validated producer draft digest.
    #[must_use]
    pub fn draft_sha256(&self) -> &str {
        &self.draft_sha256
    }

    /// Exact producer and source provenance.
    #[must_use]
    pub const fn provenance(&self) -> &GeneratedApiProvenance {
        &self.provenance
    }

    /// Validated coverage.
    #[must_use]
    pub const fn coverage(&self) -> &GeneratedApiCoverage {
        &self.coverage
    }

    /// Preserved duplicate-symbol conflicts.
    #[must_use]
    pub fn conflicts(&self) -> &[GeneratedApiConflict] {
        &self.conflicts
    }

    /// Canonically ordered facts.
    #[must_use]
    pub fn facts(&self) -> &[GeneratedApiFact] {
        &self.facts
    }

    /// Exact case-sensitive lookup.
    #[must_use]
    pub fn lookup(
        &self,
        kind: GeneratedApiFactKind,
        qualified_name: &str,
    ) -> GeneratedApiLookup<'_> {
        let matches = self
            .facts
            .iter()
            .filter(|fact| fact.kind == kind && fact.qualified_name == qualified_name)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [fact] => GeneratedApiLookup::Found(fact),
            [] if self.coverage.negative_authority => GeneratedApiLookup::AbsentAuthoritative,
            [] => GeneratedApiLookup::NotAuthoritative,
            _ => GeneratedApiLookup::Conflicted(matches),
        }
    }
}

/// Imports and independently validates one normalized generated API draft.
pub fn import_generated_api_draft(bytes: &[u8]) -> GeneratedApiImportResult<GeneratedApiIndex> {
    if bytes.len() > MAX_DRAFT_BYTES {
        return Err(error(
            GeneratedApiImportErrorCode::InputTooLarge,
            "generated API draft exceeds the import limit",
        ));
    }
    let value = serde_json::from_slice::<Value>(bytes).map_err(|source| {
        error(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("generated API draft is not valid JSON: {source}"),
        )
    })?;
    let root = object(&value, "draft root")?;
    allowed_keys(
        root,
        &[
            "schema",
            "schema_version",
            "producer",
            "source",
            "coverage",
            "conflicts",
            "systems",
            "draft_sha256",
        ],
        "draft root",
    )?;
    if string(root, "schema", "draft schema")? != SCHEMA
        || unsigned(root, "schema_version", "draft schema version")? != SCHEMA_VERSION
    {
        return Err(error(
            GeneratedApiImportErrorCode::UnsupportedSchema,
            "unsupported generated API draft schema",
        ));
    }

    let supplied_digest = string(root, "draft_sha256", "draft digest")?.to_owned();
    canonical_sha256(&supplied_digest, "draft digest")?;
    let mut projection = value.clone();
    object_mut(&mut projection, "draft projection")?.remove("draft_sha256");
    let projection_bytes = canonical_json_bytes(&projection).map_err(|source| {
        error(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("generated API draft cannot be canonicalized: {source}"),
        )
    })?;
    if supplied_digest != sha256(&projection_bytes) {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidDigest,
            "generated API draft digest does not match its content",
        ));
    }

    let producer = object(required(root, "producer", "producer")?, "producer")?;
    allowed_keys(
        producer,
        &["id", "version", "parser", "configuration"],
        "producer",
    )?;
    if string(producer, "id", "producer id")? != PRODUCER_ID {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidProducer,
            "unexpected generated API producer",
        ));
    }
    let producer_version = unsigned(producer, "version", "producer version")?;
    let parser = string(producer, "parser", "producer parser")?.to_owned();
    if producer_version == 0 || parser.is_empty() {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidProducer,
            "generated API producer version or parser is invalid",
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
    let manifest_sha256 = string(source, "manifest_sha256", "manifest digest")?.to_owned();
    canonical_sha256(&manifest_sha256, "manifest digest")?;
    let manifest_declared_digest = optional_string(source, "manifest_declared_digest")?;
    if let Some(digest) = manifest_declared_digest.as_deref() {
        flexible_sha256(digest, "declared manifest digest")?;
    }
    let revision = string(source, "revision", "source revision")?.to_owned();
    object_id(&revision, None, "source revision")?;
    let provenance = GeneratedApiProvenance {
        manifest_sha256,
        manifest_declared_digest,
        source_id: optional_string(source, "source_id")?,
        selector: optional_string(source, "selector")?,
        revision: revision.clone(),
        version: optional_string(source, "version")?,
        producer_version,
        parser,
        configuration,
    };

    let coverage_value = required(root, "coverage", "coverage")?;
    let coverage = parse_coverage(coverage_value)?;
    let parsed_paths = coverage
        .parsed_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    let systems = array(required(root, "systems", "systems")?, "systems")?;
    validate_system_order(systems)?;
    let mut facts = Vec::new();
    let mut counts = BTreeMap::new();
    for kind in GeneratedApiFactKind::all() {
        counts.insert(kind.collection().to_owned(), 0_u64);
    }
    for system in systems {
        let system_object = object(system, "system")?;
        allowed_keys(
            system_object,
            &[
                "name",
                "namespace",
                "type",
                "environment",
                "documentation",
                "attributes",
                "source",
                "functions",
                "events",
                "tables",
                "enumerations",
                "constants",
                "predicates",
            ],
            "system",
        )?;
        nonempty_string(system_object, "name", "system name")?;
        parse_source_span(
            required(system_object, "source", "system source")?,
            &parsed_paths,
            &revision,
        )?;
        for kind in GeneratedApiFactKind::all() {
            let members = array(
                required(system_object, kind.collection(), kind.collection())?,
                kind.collection(),
            )?;
            validate_member_order(members, kind)?;
            for member in members {
                facts.push(parse_fact(kind, member, &parsed_paths, &revision)?);
            }
            let member_count = u64::try_from(members.len()).map_err(|_| {
                error(
                    GeneratedApiImportErrorCode::InvalidCoverage,
                    "generated API entity count exceeds u64",
                )
            })?;
            let current = counts.get_mut(kind.collection()).ok_or_else(|| {
                error(
                    GeneratedApiImportErrorCode::InvalidCoverage,
                    "generated API entity counter is missing",
                )
            })?;
            *current = current.checked_add(member_count).ok_or_else(|| {
                error(
                    GeneratedApiImportErrorCode::InvalidCoverage,
                    "generated API entity count overflow",
                )
            })?;
        }
    }
    if counts != coverage.entity_counts {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API entity counts do not match imported facts",
        ));
    }
    facts.sort_by(|left, right| fact_key(left).cmp(&fact_key(right)));

    let conflicts = parse_conflicts(
        array(required(root, "conflicts", "conflicts")?, "conflicts")?,
        &facts,
        &parsed_paths,
    )?;

    #[derive(Serialize)]
    struct IndexProjection<'a> {
        draft_sha256: &'a str,
        provenance: &'a GeneratedApiProvenance,
        coverage: &'a GeneratedApiCoverage,
        conflicts: &'a [GeneratedApiConflict],
        facts: &'a [GeneratedApiFact],
    }
    let index_projection = IndexProjection {
        draft_sha256: &supplied_digest,
        provenance: &provenance,
        coverage: &coverage,
        conflicts: &conflicts,
        facts: &facts,
    };
    let index_bytes = canonical_json_bytes(&index_projection).map_err(|source| {
        error(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("generated API index cannot be canonicalized: {source}"),
        )
    })?;
    Ok(GeneratedApiIndex {
        index_id: sha256(&index_bytes),
        draft_sha256: supplied_digest,
        provenance,
        coverage,
        conflicts,
        facts,
    })
}

fn parse_coverage(value: &Value) -> GeneratedApiImportResult<GeneratedApiCoverage> {
    let coverage = object(value, "coverage")?;
    allowed_keys(
        coverage,
        &[
            "scope",
            "status",
            "negative_authority",
            "candidate_files",
            "parsed_files",
            "failed_files",
            "parsed_paths",
            "failures",
            "entity_counts",
            "limitations",
        ],
        "coverage",
    )?;
    if string(coverage, "scope", "coverage scope")?
        != format!("{GENERATED_ROOT}*{GENERATED_SUFFIX}")
    {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API coverage scope is incomplete or unexpected",
        ));
    }
    let candidate_files = unsigned(coverage, "candidate_files", "candidate files")?;
    let parsed_files = unsigned(coverage, "parsed_files", "parsed files")?;
    let failed_files = unsigned(coverage, "failed_files", "failed files")?;
    if candidate_files
        != parsed_files.checked_add(failed_files).ok_or_else(|| {
            error(
                GeneratedApiImportErrorCode::InvalidCoverage,
                "generated API file count overflow",
            )
        })?
    {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API file counts are inconsistent",
        ));
    }
    let complete = failed_files == 0 && candidate_files == parsed_files;
    let status = match string(coverage, "status", "coverage status")? {
        "complete" if complete => GeneratedApiCoverageStatus::Complete,
        "partial" if !complete => GeneratedApiCoverageStatus::Partial,
        _ => {
            return Err(error(
                GeneratedApiImportErrorCode::InvalidCoverage,
                "generated API coverage status does not match file counts",
            ));
        }
    };
    let negative_authority = boolean(coverage, "negative_authority", "negative authority")?;
    if negative_authority != complete {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API negative authority does not match completeness",
        ));
    }

    let parsed_values = array(
        required(coverage, "parsed_paths", "parsed paths")?,
        "parsed paths",
    )?;
    let mut parsed_paths = Vec::with_capacity(parsed_values.len());
    for path_value in parsed_values {
        let path = path_value.as_str().ok_or_else(|| {
            error(
                GeneratedApiImportErrorCode::InvalidCoverage,
                "generated API parsed path is not text",
            )
        })?;
        generated_path(path)?;
        parsed_paths.push(path.to_owned());
    }
    if parsed_paths.len()
        != usize::try_from(parsed_files).map_err(|_| {
            error(
                GeneratedApiImportErrorCode::InvalidCoverage,
                "generated API parsed file count exceeds usize",
            )
        })?
        || !strictly_sorted_unique(&parsed_paths)
    {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidOrdering,
            "generated API parsed paths are not uniquely byte-sorted",
        ));
    }

    let failure_values = array(
        required(coverage, "failures", "coverage failures")?,
        "coverage failures",
    )?;
    if failure_values.len()
        != usize::try_from(failed_files).map_err(|_| {
            error(
                GeneratedApiImportErrorCode::InvalidCoverage,
                "generated API failed file count exceeds usize",
            )
        })?
    {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API failure records do not match failed file count",
        ));
    }
    let mut failures = Vec::with_capacity(failure_values.len());
    for failure_value in failure_values {
        let failure = object(failure_value, "coverage failure")?;
        allowed_keys(failure, &["code", "message", "path"], "coverage failure")?;
        let path = optional_string(failure, "path")?;
        if let Some(candidate) = path.as_deref() {
            generated_path(candidate)?;
        }
        failures.push(GeneratedApiFailure {
            code: nonempty_string(failure, "code", "failure code")?.to_owned(),
            message: nonempty_string(failure, "message", "failure message")?.to_owned(),
            path,
        });
    }

    let count_map = object(
        required(coverage, "entity_counts", "entity counts")?,
        "entity counts",
    )?;
    let expected_collections = GeneratedApiFactKind::all()
        .into_iter()
        .map(GeneratedApiFactKind::collection)
        .collect::<BTreeSet<_>>();
    if count_map
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != expected_collections
    {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API entity-count collections are incomplete or unexpected",
        ));
    }
    let mut entity_counts = BTreeMap::new();
    for (key, count) in count_map {
        let value = count.as_u64().ok_or_else(|| {
            error(
                GeneratedApiImportErrorCode::InvalidCoverage,
                "generated API entity count is not an unsigned integer",
            )
        })?;
        entity_counts.insert(key.clone(), value);
    }

    let limitation_values = array(
        required(coverage, "limitations", "limitations")?,
        "limitations",
    )?;
    if limitation_values.is_empty() {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API limitations are missing",
        ));
    }
    let limitations = limitation_values
        .iter()
        .map(|value| {
            value.as_str().map(str::to_owned).ok_or_else(|| {
                error(
                    GeneratedApiImportErrorCode::InvalidCoverage,
                    "generated API limitation is not text",
                )
            })
        })
        .collect::<GeneratedApiImportResult<Vec<_>>>()?;

    Ok(GeneratedApiCoverage {
        status,
        negative_authority,
        candidate_files,
        parsed_files,
        failed_files,
        parsed_paths,
        failures,
        entity_counts,
        limitations,
    })
}

fn parse_fact(
    kind: GeneratedApiFactKind,
    value: &Value,
    parsed_paths: &BTreeSet<&str>,
    revision: &str,
) -> GeneratedApiImportResult<GeneratedApiFact> {
    let member = object(value, kind.as_str())?;
    allowed_keys(
        member,
        &[
            "name",
            "qualified_name",
            "type",
            "literal_name",
            "documentation",
            "restrictions",
            "attributes",
            "arguments",
            "returns",
            "payload",
            "fields",
            "values",
            "source",
        ],
        kind.as_str(),
    )?;
    let name = nonempty_string(member, "name", "member name")?.to_owned();
    let qualified_name =
        nonempty_string(member, "qualified_name", "qualified member name")?.to_owned();
    let member_type = optional_string(member, "type")?;
    let source = parse_source_span(
        required(member, "source", "member source")?,
        parsed_paths,
        revision,
    )?;
    for collection in ["arguments", "returns", "payload", "fields", "values"] {
        // The producer omits empty child collections; absence is an empty list.
        let Some(child_values) = member.get(collection) else {
            continue;
        };
        let records = array(child_values, collection)?;
        for record in records {
            let record_object = object(record, collection)?;
            parse_source_span(
                required(record_object, "source", "nested source")?,
                parsed_paths,
                revision,
            )?;
        }
    }
    array(
        required(member, "documentation", "member documentation")?,
        "member documentation",
    )?;
    object(
        required(member, "restrictions", "member restrictions")?,
        "member restrictions",
    )?;
    object(
        required(member, "attributes", "member attributes")?,
        "member attributes",
    )?;

    #[derive(Serialize)]
    struct FactProjection<'a> {
        kind: GeneratedApiFactKind,
        normalized: &'a Value,
    }
    let fact_bytes = canonical_json_bytes(&FactProjection {
        kind,
        normalized: value,
    })
    .map_err(|source| {
        error(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("generated API fact cannot be canonicalized: {source}"),
        )
    })?;
    Ok(GeneratedApiFact {
        fact_id: sha256(&fact_bytes),
        kind,
        name,
        qualified_name,
        member_type,
        source,
        normalized: value.clone(),
    })
}

fn parse_conflicts(
    values: &[Value],
    facts: &[GeneratedApiFact],
    parsed_paths: &BTreeSet<&str>,
) -> GeneratedApiImportResult<Vec<GeneratedApiConflict>> {
    let mut output = Vec::with_capacity(values.len());
    let mut previous: Option<(String, String, String)> = None;
    for value in values {
        let conflict = object(value, "conflict")?;
        allowed_keys(
            conflict,
            &["kind", "collection", "qualified_name", "sources"],
            "conflict",
        )?;
        let kind = nonempty_string(conflict, "kind", "conflict kind")?.to_owned();
        let collection = nonempty_string(conflict, "collection", "conflict collection")?.to_owned();
        let qualified_name =
            nonempty_string(conflict, "qualified_name", "conflict name")?.to_owned();
        let key = (kind.clone(), collection.clone(), qualified_name.clone());
        if previous.as_ref().is_some_and(|candidate| candidate >= &key) {
            return Err(error(
                GeneratedApiImportErrorCode::InvalidOrdering,
                "generated API conflicts are not uniquely ordered",
            ));
        }
        previous = Some(key);
        let fact_kind = kind_for_collection(&collection).ok_or_else(|| {
            error(
                GeneratedApiImportErrorCode::InvalidConflict,
                "generated API conflict names an unknown collection",
            )
        })?;
        let matching = facts
            .iter()
            .filter(|fact| fact.kind == fact_kind && fact.qualified_name == qualified_name)
            .collect::<Vec<_>>();
        if kind != "duplicate_symbol" || matching.len() < 2 {
            return Err(error(
                GeneratedApiImportErrorCode::InvalidConflict,
                "generated API conflict is not supported by duplicate facts",
            ));
        }
        let valid_locations = matching
            .iter()
            .map(|fact| (fact.source.path.as_str(), fact.source.line_start))
            .collect::<BTreeSet<_>>();
        let source_values = array(
            required(conflict, "sources", "conflict sources")?,
            "conflict sources",
        )?;
        if source_values.len() < 2 {
            return Err(error(
                GeneratedApiImportErrorCode::InvalidConflict,
                "generated API duplicate conflict has fewer than two sources",
            ));
        }
        let mut sources = Vec::with_capacity(source_values.len());
        let mut seen = BTreeSet::new();
        for source_value in source_values {
            let source = object(source_value, "conflict source")?;
            allowed_keys(source, &["path", "line_start"], "conflict source")?;
            let path = string(source, "path", "conflict path")?;
            let line_start = unsigned(source, "line_start", "conflict line")?;
            generated_path(path)?;
            if line_start == 0
                || !parsed_paths.contains(path)
                || !valid_locations.contains(&(path, line_start))
                || !seen.insert((path, line_start))
            {
                return Err(error(
                    GeneratedApiImportErrorCode::InvalidConflict,
                    "generated API conflict source is invalid or unsupported",
                ));
            }
            sources.push(GeneratedApiConflictSource {
                path: path.to_owned(),
                line_start,
            });
        }
        sources.sort_by(|left, right| {
            (left.path.as_str(), left.line_start).cmp(&(right.path.as_str(), right.line_start))
        });
        output.push(GeneratedApiConflict {
            kind,
            collection,
            qualified_name,
            sources,
        });
    }
    Ok(output)
}

fn parse_source_span(
    value: &Value,
    parsed_paths: &BTreeSet<&str>,
    revision: &str,
) -> GeneratedApiImportResult<GeneratedApiSourceSpan> {
    let source = object(value, "source span")?;
    allowed_keys(
        source,
        &["path", "line_start", "line_end", "git_object", "sha256"],
        "source span",
    )?;
    let path = string(source, "path", "source path")?.to_owned();
    generated_path(&path)?;
    if !parsed_paths.contains(path.as_str()) {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidSource,
            "generated API source path is outside parsed coverage",
        ));
    }
    let line_start = unsigned(source, "line_start", "source start line")?;
    let line_end = unsigned(source, "line_end", "source end line")?;
    if line_start == 0 || line_end < line_start {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidSource,
            "generated API source line span is invalid",
        ));
    }
    let git_object = string(source, "git_object", "source Git object")?.to_owned();
    object_id(&git_object, Some(revision.len()), "source Git object")?;
    let source_sha256 = string(source, "sha256", "source digest")?.to_owned();
    canonical_sha256(&source_sha256, "source digest")?;
    Ok(GeneratedApiSourceSpan {
        path,
        line_start,
        line_end,
        git_object,
        sha256: source_sha256,
    })
}

fn validate_system_order(systems: &[Value]) -> GeneratedApiImportResult<()> {
    let mut previous: Option<(String, String, String)> = None;
    for system in systems {
        let record = object(system, "system")?;
        let namespace = optional_string(record, "namespace")?.unwrap_or_default();
        let name = nonempty_string(record, "name", "system name")?.to_owned();
        let source = object(
            required(record, "source", "system source")?,
            "system source",
        )?;
        let path = string(source, "path", "system source path")?.to_owned();
        let key = (namespace, name, path);
        if previous.as_ref().is_some_and(|candidate| candidate >= &key) {
            return Err(error(
                GeneratedApiImportErrorCode::InvalidOrdering,
                "generated API systems are not uniquely ordered",
            ));
        }
        previous = Some(key);
    }
    Ok(())
}

fn validate_member_order(
    members: &[Value],
    kind: GeneratedApiFactKind,
) -> GeneratedApiImportResult<()> {
    let mut previous: Option<(String, String, u64)> = None;
    for member in members {
        let record = object(member, kind.as_str())?;
        let qualified_name =
            nonempty_string(record, "qualified_name", "qualified member name")?.to_owned();
        let member_type = optional_string(record, "type")?.unwrap_or_default();
        let source = object(
            required(record, "source", "member source")?,
            "member source",
        )?;
        let line = unsigned(source, "line_start", "member source line")?;
        let key = (qualified_name, member_type, line);
        if previous.as_ref().is_some_and(|candidate| candidate > &key) {
            return Err(error(
                GeneratedApiImportErrorCode::InvalidOrdering,
                format!(
                    "generated API {} records are not canonically ordered",
                    kind.collection()
                ),
            ));
        }
        previous = Some(key);
    }
    Ok(())
}

fn fact_key(fact: &GeneratedApiFact) -> (GeneratedApiFactKind, &str, &str) {
    (fact.kind, &fact.qualified_name, &fact.fact_id)
}

fn kind_for_collection(collection: &str) -> Option<GeneratedApiFactKind> {
    GeneratedApiFactKind::all()
        .into_iter()
        .find(|kind| kind.collection() == collection)
}

fn generated_path(path: &str) -> GeneratedApiImportResult<()> {
    if !path.starts_with(GENERATED_ROOT)
        || !path.ends_with(GENERATED_SUFFIX)
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidSource,
            "generated API path is noncanonical or outside the generated-document root",
        ));
    }
    Ok(())
}

fn canonical_sha256(value: &str, label: &str) -> GeneratedApiImportResult<()> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("{label} is not a canonical SHA-256 identifier"),
        ));
    };
    raw_sha256(digest, label)
}

fn flexible_sha256(value: &str, label: &str) -> GeneratedApiImportResult<()> {
    raw_sha256(value.strip_prefix("sha256:").unwrap_or(value), label)
}

fn raw_sha256(value: &str, label: &str) -> GeneratedApiImportResult<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("{label} is not a SHA-256 digest"),
        ));
    }
    Ok(())
}

fn object_id(
    value: &str,
    expected_length: Option<usize>,
    label: &str,
) -> GeneratedApiImportResult<()> {
    if !matches!(value.len(), 40 | 64)
        || expected_length.is_some_and(|length| length != value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidSource,
            format!("{label} is not a canonical Git object identifier"),
        ));
    }
    Ok(())
}

fn strictly_sorted_unique(values: &[String]) -> bool {
    values
        .windows(2)
        .all(|pair| pair[0].as_bytes() < pair[1].as_bytes())
}

fn allowed_keys(
    object: &Map<String, Value>,
    allowed: &[&str],
    label: &str,
) -> GeneratedApiImportResult<()> {
    if let Some(unexpected) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(error(
            GeneratedApiImportErrorCode::UnsupportedSchema,
            format!("{label} contains unsupported field {unexpected:?}"),
        ));
    }
    Ok(())
}

fn object<'a>(value: &'a Value, label: &str) -> GeneratedApiImportResult<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| {
        error(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("{label} must be an object"),
        )
    })
}

fn object_mut<'a>(
    value: &'a mut Value,
    label: &str,
) -> GeneratedApiImportResult<&'a mut Map<String, Value>> {
    value.as_object_mut().ok_or_else(|| {
        error(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("{label} must be an object"),
        )
    })
}

fn array<'a>(value: &'a Value, label: &str) -> GeneratedApiImportResult<&'a [Value]> {
    value.as_array().map(Vec::as_slice).ok_or_else(|| {
        error(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("{label} must be an array"),
        )
    })
}

fn required<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> GeneratedApiImportResult<&'a Value> {
    object.get(key).ok_or_else(|| {
        error(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("{label} is missing"),
        )
    })
}

fn string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> GeneratedApiImportResult<&'a str> {
    required(object, key, label)?.as_str().ok_or_else(|| {
        error(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("{label} must be text"),
        )
    })
}

fn nonempty_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> GeneratedApiImportResult<&'a str> {
    let value = string(object, key, label)?;
    if value.is_empty() {
        return Err(error(
            GeneratedApiImportErrorCode::InvalidFact,
            format!("{label} must not be empty"),
        ));
    }
    Ok(value)
}

fn optional_string(
    object: &Map<String, Value>,
    key: &str,
) -> GeneratedApiImportResult<Option<String>> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(error(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("optional field {key:?} must be text or null"),
        )),
    }
}

fn unsigned(object: &Map<String, Value>, key: &str, label: &str) -> GeneratedApiImportResult<u64> {
    required(object, key, label)?.as_u64().ok_or_else(|| {
        error(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("{label} must be an unsigned integer"),
        )
    })
}

fn boolean(object: &Map<String, Value>, key: &str, label: &str) -> GeneratedApiImportResult<bool> {
    required(object, key, label)?.as_bool().ok_or_else(|| {
        error(
            GeneratedApiImportErrorCode::InvalidJson,
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

fn error(code: GeneratedApiImportErrorCode, message: impl Into<String>) -> GeneratedApiImportError {
    GeneratedApiImportError::new(code, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn source(path: &str, line: u64) -> Value {
        json!({
            "path": path,
            "line_start": line,
            "line_end": line + 1,
            "git_object": "1111111111111111111111111111111111111111",
            "sha256": "sha256:2222222222222222222222222222222222222222222222222222222222222222"
        })
    }

    fn member(path: &str, line: u64) -> Value {
        json!({
            "name": "GetAuraDataByIndex",
            "qualified_name": "C_UnitAuras.GetAuraDataByIndex",
            "type": "Function",
            "documentation": ["Returns one aura record."],
            "restrictions": {"has_restrictions": true},
            "attributes": {},
            "arguments": [{"name": "unitToken", "type": "UnitToken", "source": source(path, line + 1)}],
            "returns": [{"name": "aura", "type": "AuraData", "source": source(path, line + 2)}],
            "payload": [],
            "fields": [],
            "values": [],
            "source": source(path, line)
        })
    }

    fn draft(complete: bool, duplicate: bool) -> Value {
        let path = "Interface/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua";
        let functions = if duplicate {
            vec![member(path, 10), member(path, 30)]
        } else {
            vec![member(path, 10)]
        };
        json!({
            "schema": SCHEMA,
            "schema_version": SCHEMA_VERSION,
            "producer": {
                "id": PRODUCER_ID,
                "version": 1,
                "parser": "declarative-lua-table-v1",
                "configuration": {"generated_root": GENERATED_ROOT}
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
                "scope": format!("{GENERATED_ROOT}*{GENERATED_SUFFIX}"),
                "status": if complete { "complete" } else { "partial" },
                "negative_authority": complete,
                "candidate_files": if complete { 1 } else { 2 },
                "parsed_files": 1,
                "failed_files": if complete { 0 } else { 1 },
                "parsed_paths": [path],
                "failures": if complete { Vec::<Value>::new() } else { vec![json!({"code": "parse", "message": "unsupported", "path": path})] },
                "entity_counts": {
                    "functions": functions.len(),
                    "events": 0,
                    "tables": 0,
                    "enumerations": 0,
                    "constants": 0,
                    "predicates": 0
                },
                "limitations": ["runtime behavior requires a client probe"]
            },
            "conflicts": if duplicate { vec![json!({
                "kind": "duplicate_symbol",
                "collection": "functions",
                "qualified_name": "C_UnitAuras.GetAuraDataByIndex",
                "sources": [
                    {"path": path, "line_start": 10},
                    {"path": path, "line_start": 30}
                ]
            })] } else { Vec::<Value>::new() },
            "systems": [{
                "name": "UnitAuras",
                "namespace": "C_UnitAuras",
                "type": "System",
                "environment": "All",
                "documentation": [],
                "attributes": {},
                "source": source(path, 1),
                "functions": functions,
                "events": [],
                "tables": [],
                "enumerations": [],
                "constants": [],
                "predicates": []
            }]
        })
    }

    fn seal(mut value: Value) -> GeneratedApiImportResult<Vec<u8>> {
        let bytes = canonical_json_bytes(&value).map_err(|source| {
            error(
                GeneratedApiImportErrorCode::InvalidDigest,
                format!("test draft cannot be canonicalized: {source}"),
            )
        })?;
        object_mut(&mut value, "test draft")?
            .insert("draft_sha256".to_owned(), Value::String(sha256(&bytes)));
        serde_json::to_vec(&value).map_err(|source| {
            error(
                GeneratedApiImportErrorCode::InvalidJson,
                format!("test draft cannot be serialized: {source}"),
            )
        })
    }

    #[test]
    fn complete_import_supports_exact_positive_and_negative_lookup() -> GeneratedApiImportResult<()>
    {
        let index = import_generated_api_draft(&seal(draft(true, false))?)?;
        match index.lookup(
            GeneratedApiFactKind::Function,
            "C_UnitAuras.GetAuraDataByIndex",
        ) {
            GeneratedApiLookup::Found(fact) => {
                assert!(fact.restrictions().is_some());
            }
            other => {
                return Err(error(
                    GeneratedApiImportErrorCode::InvalidFact,
                    format!("unexpected lookup result: {other:?}"),
                ));
            }
        }
        assert_eq!(
            index.lookup(GeneratedApiFactKind::Function, "C_UnitAuras.Missing"),
            GeneratedApiLookup::AbsentAuthoritative
        );
        Ok(())
    }

    #[test]
    fn partial_import_never_proves_absence() -> GeneratedApiImportResult<()> {
        let index = import_generated_api_draft(&seal(draft(false, false))?)?;
        assert_eq!(
            index.lookup(GeneratedApiFactKind::Function, "C_UnitAuras.Missing"),
            GeneratedApiLookup::NotAuthoritative
        );
        assert!(!index.coverage().negative_authority());
        Ok(())
    }

    #[test]
    fn duplicate_facts_remain_conflicted() -> GeneratedApiImportResult<()> {
        let index = import_generated_api_draft(&seal(draft(true, true))?)?;
        match index.lookup(
            GeneratedApiFactKind::Function,
            "C_UnitAuras.GetAuraDataByIndex",
        ) {
            GeneratedApiLookup::Conflicted(facts) => assert_eq!(facts.len(), 2),
            other => {
                return Err(error(
                    GeneratedApiImportErrorCode::InvalidConflict,
                    format!("unexpected lookup result: {other:?}"),
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn tampering_breaks_the_draft_digest() -> GeneratedApiImportResult<()> {
        let bytes = seal(draft(true, false))?;
        let mut value = serde_json::from_slice::<Value>(&bytes).map_err(|source| {
            error(
                GeneratedApiImportErrorCode::InvalidJson,
                format!("test draft cannot be parsed: {source}"),
            )
        })?;
        object_mut(&mut value, "test draft")?
            .insert("producer".to_owned(), serde_json::json!({"tampered": true}));
        let tampered = serde_json::to_vec(&value).map_err(|source| {
            error(
                GeneratedApiImportErrorCode::InvalidJson,
                format!("test draft cannot be serialized: {source}"),
            )
        })?;
        let failure = import_generated_api_draft(&tampered).err().ok_or_else(|| {
            error(
                GeneratedApiImportErrorCode::InvalidDigest,
                "tampered draft unexpectedly imported",
            )
        })?;
        assert_eq!(failure.code(), GeneratedApiImportErrorCode::InvalidDigest);
        Ok(())
    }
}

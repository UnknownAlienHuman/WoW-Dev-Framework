//! Import boundary for normalized Blizzard generated API documentation.
//!
//! The producer output is untrusted input at this boundary. Import repeats the
//! canonical digest, source, coverage, ordering, and conflict checks before it
//! exposes immutable facts to the rest of `wow-reference`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

const SCHEMA: &str = "wow-dev-framework/blizzard-api-reference-draft";
const SCHEMA_VERSION: u64 = 1;
const PRODUCER_ID: &str = "blizzard-generated-api-reference";
const GENERATED_ROOT: &str = "Interface/AddOns/Blizzard_APIDocumentationGenerated/";
const GENERATED_SUFFIX: &str = "Documentation.lua";
const MAX_DRAFT_BYTES: usize = 512 * 1024 * 1024;

/// Stable failure classes returned by the generated-API import boundary.
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

/// One bounded generated-API import failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiImportError {
    code: GeneratedApiImportErrorCode,
    message: Box<str>,
}

impl GeneratedApiImportError {
    fn new(code: GeneratedApiImportErrorCode, message: impl Into<Box<str>>) -> Self {
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

    /// Safe bounded explanation.
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

/// Result type for generated-API imports.
pub type GeneratedApiImportResult<T> = Result<T, GeneratedApiImportError>;

/// Kind of one normalized generated API fact.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
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
    /// Canonical singular text.
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

/// Coverage state retained from the producer after independent validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedApiCoverageStatus {
    Complete,
    Partial,
}

/// Exact source span and content identity for one generated record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedApiSourceSpan {
    path: Box<str>,
    line_start: u64,
    line_end: u64,
    git_object: Box<str>,
    sha256: Box<str>,
}

impl GeneratedApiSourceSpan {
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

    /// Exact Git blob object identity.
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

/// Exact operation-scoped source and producer provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedApiProvenance {
    manifest_sha256: Box<str>,
    manifest_declared_digest: Option<Box<str>>,
    source_id: Option<Box<str>>,
    selector: Option<Box<str>>,
    revision: Box<str>,
    version: Option<Box<str>>,
    producer_version: u64,
    parser: Box<str>,
    configuration: BTreeMap<String, Value>,
}

impl GeneratedApiProvenance {
    /// Digest of the exact source manifest bytes consumed by the producer.
    #[must_use]
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }

    /// Exact source revision used for every imported file.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Moving selector resolved for this operation, when supplied.
    #[must_use]
    pub fn selector(&self) -> Option<&str> {
        self.selector.as_deref()
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

    /// Producer parser identity.
    #[must_use]
    pub fn parser(&self) -> &str {
        &self.parser
    }
}

/// One immutable normalized API fact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedApiFact {
    fact_id: Box<str>,
    kind: GeneratedApiFactKind,
    name: Box<str>,
    qualified_name: Box<str>,
    member_type: Option<Box<str>>,
    literal_name: Option<Box<str>>,
    documentation: Vec<Box<str>>,
    restrictions: BTreeMap<String, Value>,
    attributes: BTreeMap<String, Value>,
    arguments: Vec<Value>,
    returns: Vec<Value>,
    payload: Vec<Value>,
    fields: Vec<Value>,
    values: Vec<Value>,
    source: GeneratedApiSourceSpan,
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

    /// Exact normalized lookup name.
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }

    /// Generated member type, when declared.
    #[must_use]
    pub fn member_type(&self) -> Option<&str> {
        self.member_type.as_deref()
    }

    /// Generated restriction metadata. It remains source evidence and does not
    /// replace target-client runtime checks.
    #[must_use]
    pub const fn restrictions(&self) -> &BTreeMap<String, Value> {
        &self.restrictions
    }

    /// Exact source span and content identity.
    #[must_use]
    pub const fn source(&self) -> &GeneratedApiSourceSpan {
        &self.source
    }
}

/// One producer-preserved conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedApiConflict {
    kind: Box<str>,
    collection: Box<str>,
    qualified_name: Box<str>,
    sources: Vec<GeneratedApiConflictSource>,
}

impl GeneratedApiConflict {
    /// Conflict kind emitted by the producer.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Qualified name affected by the conflict.
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }
}

/// Source locator retained by a producer conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedApiConflictSource {
    path: Box<str>,
    line_start: u64,
}

/// Validated generated API coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedApiCoverage {
    status: GeneratedApiCoverageStatus,
    negative_authority: bool,
    candidate_files: u64,
    parsed_files: u64,
    failed_files: u64,
    parsed_paths: Vec<Box<str>>,
    failures: Vec<GeneratedApiFailure>,
    entity_counts: BTreeMap<Box<str>, u64>,
    limitations: Vec<Box<str>>,
}

impl GeneratedApiCoverage {
    /// Coverage status.
    #[must_use]
    pub const fn status(&self) -> GeneratedApiCoverageStatus {
        self.status
    }

    /// Whether this exact generated-document scope may support an authoritative
    /// negative. Conflicts still block the affected lookup independently.
    #[must_use]
    pub const fn negative_authority(&self) -> bool {
        self.negative_authority
    }

    /// Candidate file count.
    #[must_use]
    pub const fn candidate_files(&self) -> u64 {
        self.candidate_files
    }

    /// Successfully imported file count.
    #[must_use]
    pub const fn parsed_files(&self) -> u64 {
        self.parsed_files
    }

    /// Failed file count.
    #[must_use]
    pub const fn failed_files(&self) -> u64 {
        self.failed_files
    }
}

/// One producer failure retained by a partial draft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedApiFailure {
    code: Box<str>,
    message: Box<str>,
    path: Option<Box<str>>,
}

/// Exact lookup outcome from one generated API index.
#[derive(Debug, Clone, PartialEq)]
pub enum GeneratedApiLookup<'a> {
    Found(&'a GeneratedApiFact),
    Conflicted(Vec<&'a GeneratedApiFact>),
    AbsentAuthoritative,
    NotAuthoritative,
}

/// Immutable, independently validated index produced from one draft.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedApiIndex {
    index_id: Box<str>,
    draft_sha256: Box<str>,
    provenance: GeneratedApiProvenance,
    coverage: GeneratedApiCoverage,
    conflicts: Vec<GeneratedApiConflict>,
    facts: Vec<GeneratedApiFact>,
}

impl GeneratedApiIndex {
    /// Content-addressed index identity.
    #[must_use]
    pub fn index_id(&self) -> &str {
        &self.index_id
    }

    /// Producer draft digest validated by this import.
    #[must_use]
    pub fn draft_sha256(&self) -> &str {
        &self.draft_sha256
    }

    /// Exact source and producer provenance.
    #[must_use]
    pub const fn provenance(&self) -> &GeneratedApiProvenance {
        &self.provenance
    }

    /// Validated coverage.
    #[must_use]
    pub const fn coverage(&self) -> &GeneratedApiCoverage {
        &self.coverage
    }

    /// Preserved conflicts.
    #[must_use]
    pub fn conflicts(&self) -> &[GeneratedApiConflict] {
        &self.conflicts
    }

    /// Canonically ordered facts.
    #[must_use]
    pub fn facts(&self) -> &[GeneratedApiFact] {
        &self.facts
    }

    /// Performs one exact, case-sensitive lookup.
    #[must_use]
    pub fn lookup(
        &self,
        kind: GeneratedApiFactKind,
        qualified_name: &str,
    ) -> GeneratedApiLookup<'_> {
        let matches = self
            .facts
            .iter()
            .filter(|fact| fact.kind == kind && fact.qualified_name.as_ref() == qualified_name)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [fact] => GeneratedApiLookup::Found(fact),
            [] if self.coverage.negative_authority => GeneratedApiLookup::AbsentAuthoritative,
            [] => GeneratedApiLookup::NotAuthoritative,
            _ => GeneratedApiLookup::Conflicted(matches),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DraftRoot {
    schema: String,
    schema_version: u64,
    producer: DraftProducer,
    source: DraftSource,
    coverage: DraftCoverage,
    #[serde(default)]
    conflicts: Vec<DraftConflict>,
    systems: Vec<DraftSystem>,
    draft_sha256: String,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DraftProducer {
    id: String,
    version: u64,
    parser: String,
    #[serde(default)]
    configuration: BTreeMap<String, Value>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DraftSource {
    manifest_sha256: String,
    manifest_declared_digest: Option<String>,
    source_id: Option<String>,
    selector: Option<String>,
    revision: String,
    version: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DraftCoverage {
    scope: String,
    status: String,
    negative_authority: bool,
    candidate_files: u64,
    parsed_files: u64,
    failed_files: u64,
    parsed_paths: Vec<String>,
    failures: Vec<DraftFailure>,
    entity_counts: BTreeMap<String, u64>,
    limitations: Vec<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DraftFailure {
    code: String,
    message: String,
    path: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DraftConflict {
    kind: String,
    collection: String,
    qualified_name: String,
    sources: Vec<DraftConflictSource>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DraftConflictSource {
    path: String,
    line_start: u64,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DraftSystem {
    name: String,
    namespace: Option<String>,
    #[serde(rename = "type")]
    system_type: Option<String>,
    environment: Option<String>,
    #[serde(default)]
    documentation: Vec<String>,
    #[serde(default)]
    attributes: BTreeMap<String, Value>,
    source: DraftSourceSpan,
    #[serde(default)]
    functions: Vec<DraftMember>,
    #[serde(default)]
    events: Vec<DraftMember>,
    #[serde(default)]
    tables: Vec<DraftMember>,
    #[serde(default)]
    enumerations: Vec<DraftMember>,
    #[serde(default)]
    constants: Vec<DraftMember>,
    #[serde(default)]
    predicates: Vec<DraftMember>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DraftMember {
    name: String,
    qualified_name: String,
    #[serde(rename = "type")]
    member_type: Option<String>,
    literal_name: Option<String>,
    #[serde(default)]
    documentation: Vec<String>,
    #[serde(default)]
    restrictions: BTreeMap<String, Value>,
    #[serde(default)]
    attributes: BTreeMap<String, Value>,
    #[serde(default)]
    arguments: Vec<Value>,
    #[serde(default)]
    returns: Vec<Value>,
    #[serde(default)]
    payload: Vec<Value>,
    #[serde(default)]
    fields: Vec<Value>,
    #[serde(default)]
    values: Vec<Value>,
    source: DraftSourceSpan,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct DraftSourceSpan {
    path: String,
    line_start: u64,
    line_end: u64,
    git_object: String,
    sha256: String,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

/// Imports one normalized producer draft after independently validating it.
pub fn import_generated_api_draft(bytes: &[u8]) -> GeneratedApiImportResult<GeneratedApiIndex> {
    if bytes.len() > MAX_DRAFT_BYTES {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InputTooLarge,
            "generated API draft exceeds the import size limit",
        ));
    }

    let value = serde_json::from_slice::<Value>(bytes).map_err(|error| {
        GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("generated API draft is not valid JSON: {error}"),
        )
    })?;
    let supplied_digest = value
        .get("draft_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidDigest,
                "generated API draft has no draft_sha256",
            )
        })?;
    validate_sha256(supplied_digest, "draft_sha256")?;

    let mut digest_projection = value.clone();
    let projection = digest_projection.as_object_mut().ok_or_else(|| {
        GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidJson,
            "generated API draft root must be an object",
        )
    })?;
    projection.remove("draft_sha256");
    let digest_bytes = canonical_json_bytes(&digest_projection).map_err(|error| {
        GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("generated API draft cannot be canonicalized: {error}"),
        )
    })?;
    let expected_digest = sha256_prefixed(&digest_bytes);
    if supplied_digest != expected_digest {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidDigest,
            "generated API draft digest does not match its content",
        ));
    }

    let draft = serde_json::from_value::<DraftRoot>(value).map_err(|error| {
        GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidJson,
            format!("generated API draft shape is invalid: {error}"),
        )
    })?;
    validate_root(&draft)?;

    let parsed_paths = draft
        .coverage
        .parsed_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut facts = Vec::new();
    let mut observed_counts = BTreeMap::<String, u64>::new();

    for system in &draft.systems {
        validate_system(system, &parsed_paths, &draft.source.revision)?;
        for kind in GeneratedApiFactKind::all() {
            let members = members_for_kind(system, kind);
            validate_member_order(members, kind)?;
            for member in members {
                facts.push(import_member(kind, member)?);
            }
            let count = u64::try_from(members.len()).map_err(|_| {
                GeneratedApiImportError::new(
                    GeneratedApiImportErrorCode::InvalidCoverage,
                    "generated API member count exceeds u64",
                )
            })?;
            let current = observed_counts.entry(kind.collection().to_owned()).or_default();
            *current = current.checked_add(count).ok_or_else(|| {
                GeneratedApiImportError::new(
                    GeneratedApiImportErrorCode::InvalidCoverage,
                    "generated API member count overflow",
                )
            })?;
        }
    }

    for kind in GeneratedApiFactKind::all() {
        observed_counts.entry(kind.collection().to_owned()).or_default();
    }
    if observed_counts != draft.coverage.entity_counts {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API entity counts do not match imported facts",
        ));
    }

    facts.sort_by(|left, right| fact_sort_key(left).cmp(&fact_sort_key(right)));
    let conflicts = import_conflicts(&draft.conflicts, &facts, &parsed_paths)?;
    let coverage = import_coverage(&draft.coverage)?;
    let provenance = GeneratedApiProvenance {
        manifest_sha256: draft.source.manifest_sha256.clone().into_boxed_str(),
        manifest_declared_digest: draft
            .source
            .manifest_declared_digest
            .clone()
            .map(String::into_boxed_str),
        source_id: draft.source.source_id.clone().map(String::into_boxed_str),
        selector: draft.source.selector.clone().map(String::into_boxed_str),
        revision: draft.source.revision.clone().into_boxed_str(),
        version: draft.source.version.clone().map(String::into_boxed_str),
        producer_version: draft.producer.version,
        parser: draft.producer.parser.clone().into_boxed_str(),
        configuration: draft.producer.configuration.clone(),
    };

    #[derive(Serialize)]
    struct IndexProjection<'a> {
        draft_sha256: &'a str,
        provenance: &'a GeneratedApiProvenance,
        coverage: &'a GeneratedApiCoverage,
        conflicts: &'a [GeneratedApiConflict],
        facts: &'a [GeneratedApiFact],
    }
    let projection = IndexProjection {
        draft_sha256: &draft.draft_sha256,
        provenance: &provenance,
        coverage: &coverage,
        conflicts: &conflicts,
        facts: &facts,
    };
    let index_bytes = canonical_json_bytes(&projection).map_err(|error| {
        GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("generated API index cannot be canonicalized: {error}"),
        )
    })?;

    Ok(GeneratedApiIndex {
        index_id: sha256_prefixed(&index_bytes).into_boxed_str(),
        draft_sha256: draft.draft_sha256.into_boxed_str(),
        provenance,
        coverage,
        conflicts,
        facts,
    })
}

fn validate_root(draft: &DraftRoot) -> GeneratedApiImportResult<()> {
    if draft.schema != SCHEMA || draft.schema_version != SCHEMA_VERSION {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::UnsupportedSchema,
            "unsupported generated API draft schema",
        ));
    }
    if draft.producer.id != PRODUCER_ID
        || draft.producer.version == 0
        || draft.producer.parser.is_empty()
    {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidProducer,
            "generated API producer identity is invalid",
        ));
    }
    reject_nonempty_extensions(&draft.extra, "draft root")?;
    reject_nonempty_extensions(&draft.producer.extra, "producer")?;
    reject_nonempty_extensions(&draft.source.extra, "source")?;
    reject_nonempty_extensions(&draft.coverage.extra, "coverage")?;
    validate_sha256(&draft.draft_sha256, "draft_sha256")?;
    validate_sha256(&draft.source.manifest_sha256, "manifest_sha256")?;
    if let Some(digest) = &draft.source.manifest_declared_digest {
        validate_sha256(digest, "manifest_declared_digest")?;
    }
    validate_object_id(&draft.source.revision, None, "source revision")?;
    if draft.coverage.scope != format!("{GENERATED_ROOT}*{GENERATED_SUFFIX}") {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API coverage scope is not the complete generated-document scope",
        ));
    }
    validate_coverage_shape(&draft.coverage)?;
    validate_system_order(&draft.systems)?;
    Ok(())
}

fn validate_coverage_shape(coverage: &DraftCoverage) -> GeneratedApiImportResult<()> {
    let expected_status = if coverage.failed_files == 0
        && coverage.candidate_files == coverage.parsed_files
    {
        "complete"
    } else {
        "partial"
    };
    if coverage.status != expected_status
        || coverage.negative_authority != (expected_status == "complete")
        || coverage.candidate_files
            != coverage
                .parsed_files
                .checked_add(coverage.failed_files)
                .ok_or_else(|| {
                    GeneratedApiImportError::new(
                        GeneratedApiImportErrorCode::InvalidCoverage,
                        "generated API file-count overflow",
                    )
                })?
    {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API coverage counts or authority are inconsistent",
        ));
    }
    let parsed_len = u64::try_from(coverage.parsed_paths.len()).map_err(|_| {
        GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API parsed path count exceeds u64",
        )
    })?;
    let failure_len = u64::try_from(coverage.failures.len()).map_err(|_| {
        GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API failure count exceeds u64",
        )
    })?;
    if parsed_len != coverage.parsed_files || failure_len != coverage.failed_files {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API coverage arrays do not match their counts",
        ));
    }
    let mut previous: Option<&str> = None;
    let mut seen = BTreeSet::new();
    for path in &coverage.parsed_paths {
        validate_generated_path(path)?;
        if previous.is_some_and(|candidate| candidate.as_bytes() >= path.as_bytes())
            || !seen.insert(path.as_str())
        {
            return Err(GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidOrdering,
                "generated API parsed paths are not uniquely byte-sorted",
            ));
        }
        previous = Some(path);
    }
    for failure in &coverage.failures {
        reject_nonempty_extensions(&failure.extra, "coverage failure")?;
        if failure.code.is_empty() || failure.message.is_empty() {
            return Err(GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidCoverage,
                "generated API failure has an empty code or message",
            ));
        }
        if let Some(path) = &failure.path {
            validate_generated_path(path)?;
        }
    }
    if coverage.limitations.is_empty() {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidCoverage,
            "generated API limitations must remain explicit",
        ));
    }
    Ok(())
}

fn validate_system_order(systems: &[DraftSystem]) -> GeneratedApiImportResult<()> {
    for pair in systems.windows(2) {
        let left = system_sort_key(&pair[0]);
        let right = system_sort_key(&pair[1]);
        if left >= right {
            return Err(GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidOrdering,
                "generated API systems are not canonically ordered",
            ));
        }
    }
    Ok(())
}

fn system_sort_key(system: &DraftSystem) -> (&str, &str, &str) {
    (
        system.namespace.as_deref().unwrap_or_default(),
        &system.name,
        &system.source.path,
    )
}

fn validate_system(
    system: &DraftSystem,
    parsed_paths: &BTreeSet<&str>,
    revision: &str,
) -> GeneratedApiImportResult<()> {
    if system.name.is_empty() {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidFact,
            "generated API system name is empty",
        ));
    }
    reject_nonempty_extensions(&system.extra, "system")?;
    validate_source_span(&system.source, parsed_paths, revision)?;
    if system.documentation.iter().any(String::is_empty) {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidFact,
            "generated API system contains empty documentation",
        ));
    }
    let _ = (&system.system_type, &system.environment, &system.attributes);
    Ok(())
}

fn validate_member_order(
    members: &[DraftMember],
    kind: GeneratedApiFactKind,
) -> GeneratedApiImportResult<()> {
    for pair in members.windows(2) {
        let left = member_sort_key(&pair[0]);
        let right = member_sort_key(&pair[1]);
        if left > right {
            return Err(GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidOrdering,
                format!("generated API {} records are not canonically ordered", kind.collection()),
            ));
        }
    }
    Ok(())
}

fn member_sort_key(member: &DraftMember) -> (&str, &str, u64) {
    (
        &member.qualified_name,
        member.member_type.as_deref().unwrap_or_default(),
        member.source.line_start,
    )
}

fn members_for_kind(system: &DraftSystem, kind: GeneratedApiFactKind) -> &[DraftMember] {
    match kind {
        GeneratedApiFactKind::Function => &system.functions,
        GeneratedApiFactKind::Event => &system.events,
        GeneratedApiFactKind::Table => &system.tables,
        GeneratedApiFactKind::Enumeration => &system.enumerations,
        GeneratedApiFactKind::Constant => &system.constants,
        GeneratedApiFactKind::Predicate => &system.predicates,
    }
}

fn import_member(
    kind: GeneratedApiFactKind,
    member: &DraftMember,
) -> GeneratedApiImportResult<GeneratedApiFact> {
    if member.name.is_empty() || member.qualified_name.is_empty() {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidFact,
            "generated API member name is empty",
        ));
    }
    reject_nonempty_extensions(&member.extra, "member")?;
    if member.documentation.iter().any(String::is_empty) {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidFact,
            "generated API member contains empty documentation",
        ));
    }
    let source = import_source_span(&member.source);
    #[derive(Serialize)]
    struct FactProjection<'a> {
        kind: GeneratedApiFactKind,
        name: &'a str,
        qualified_name: &'a str,
        member_type: Option<&'a str>,
        literal_name: Option<&'a str>,
        documentation: &'a [String],
        restrictions: &'a BTreeMap<String, Value>,
        attributes: &'a BTreeMap<String, Value>,
        arguments: &'a [Value],
        returns: &'a [Value],
        payload: &'a [Value],
        fields: &'a [Value],
        values: &'a [Value],
        source: &'a GeneratedApiSourceSpan,
    }
    let projection = FactProjection {
        kind,
        name: &member.name,
        qualified_name: &member.qualified_name,
        member_type: member.member_type.as_deref(),
        literal_name: member.literal_name.as_deref(),
        documentation: &member.documentation,
        restrictions: &member.restrictions,
        attributes: &member.attributes,
        arguments: &member.arguments,
        returns: &member.returns,
        payload: &member.payload,
        fields: &member.fields,
        values: &member.values,
        source: &source,
    };
    let bytes = canonical_json_bytes(&projection).map_err(|error| {
        GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("generated API fact cannot be canonicalized: {error}"),
        )
    })?;
    Ok(GeneratedApiFact {
        fact_id: sha256_prefixed(&bytes).into_boxed_str(),
        kind,
        name: member.name.clone().into_boxed_str(),
        qualified_name: member.qualified_name.clone().into_boxed_str(),
        member_type: member.member_type.clone().map(String::into_boxed_str),
        literal_name: member.literal_name.clone().map(String::into_boxed_str),
        documentation: member
            .documentation
            .iter()
            .cloned()
            .map(String::into_boxed_str)
            .collect(),
        restrictions: member.restrictions.clone(),
        attributes: member.attributes.clone(),
        arguments: member.arguments.clone(),
        returns: member.returns.clone(),
        payload: member.payload.clone(),
        fields: member.fields.clone(),
        values: member.values.clone(),
        source,
    })
}

fn validate_source_span(
    source: &DraftSourceSpan,
    parsed_paths: &BTreeSet<&str>,
    revision: &str,
) -> GeneratedApiImportResult<()> {
    reject_nonempty_extensions(&source.extra, "source span")?;
    validate_generated_path(&source.path)?;
    if !parsed_paths.contains(source.path.as_str()) {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidSource,
            "generated API source path is outside parsed coverage",
        ));
    }
    if source.line_start == 0 || source.line_end < source.line_start {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidSource,
            "generated API source line span is invalid",
        ));
    }
    validate_sha256(&source.sha256, "source sha256")?;
    validate_object_id(
        &source.git_object,
        Some(revision.len()),
        "source Git object",
    )?;
    Ok(())
}

fn import_source_span(source: &DraftSourceSpan) -> GeneratedApiSourceSpan {
    GeneratedApiSourceSpan {
        path: source.path.clone().into_boxed_str(),
        line_start: source.line_start,
        line_end: source.line_end,
        git_object: source.git_object.clone().into_boxed_str(),
        sha256: source.sha256.clone().into_boxed_str(),
    }
}

fn import_coverage(coverage: &DraftCoverage) -> GeneratedApiImportResult<GeneratedApiCoverage> {
    let status = match coverage.status.as_str() {
        "complete" => GeneratedApiCoverageStatus::Complete,
        "partial" => GeneratedApiCoverageStatus::Partial,
        _ => {
            return Err(GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidCoverage,
                "generated API coverage status is invalid",
            ));
        }
    };
    Ok(GeneratedApiCoverage {
        status,
        negative_authority: coverage.negative_authority,
        candidate_files: coverage.candidate_files,
        parsed_files: coverage.parsed_files,
        failed_files: coverage.failed_files,
        parsed_paths: coverage
            .parsed_paths
            .iter()
            .cloned()
            .map(String::into_boxed_str)
            .collect(),
        failures: coverage
            .failures
            .iter()
            .map(|failure| GeneratedApiFailure {
                code: failure.code.clone().into_boxed_str(),
                message: failure.message.clone().into_boxed_str(),
                path: failure.path.clone().map(String::into_boxed_str),
            })
            .collect(),
        entity_counts: coverage
            .entity_counts
            .iter()
            .map(|(key, value)| (key.clone().into_boxed_str(), *value))
            .collect(),
        limitations: coverage
            .limitations
            .iter()
            .cloned()
            .map(String::into_boxed_str)
            .collect(),
    })
}

fn import_conflicts(
    conflicts: &[DraftConflict],
    facts: &[GeneratedApiFact],
    parsed_paths: &BTreeSet<&str>,
) -> GeneratedApiImportResult<Vec<GeneratedApiConflict>> {
    let mut output = Vec::with_capacity(conflicts.len());
    let mut previous: Option<(&str, &str, &str)> = None;
    for conflict in conflicts {
        reject_nonempty_extensions(&conflict.extra, "conflict")?;
        let key = (
            conflict.kind.as_str(),
            conflict.collection.as_str(),
            conflict.qualified_name.as_str(),
        );
        if previous.is_some_and(|candidate| candidate >= key) {
            return Err(GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidOrdering,
                "generated API conflicts are not uniquely ordered",
            ));
        }
        previous = Some(key);
        let kind = kind_for_collection(&conflict.collection).ok_or_else(|| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidConflict,
                "generated API conflict names an unknown collection",
            )
        })?;
        if conflict.kind != "duplicate_symbol"
            || conflict.sources.len() < 2
            || facts
                .iter()
                .filter(|fact| {
                    fact.kind == kind
                        && fact.qualified_name.as_ref() == conflict.qualified_name.as_str()
                })
                .count()
                < 2
        {
            return Err(GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidConflict,
                "generated API conflict is not supported by duplicate facts",
            ));
        }
        let mut sources = Vec::with_capacity(conflict.sources.len());
        let mut source_seen = BTreeSet::new();
        for source in &conflict.sources {
            reject_nonempty_extensions(&source.extra, "conflict source")?;
            validate_generated_path(&source.path)?;
            if source.line_start == 0
                || !parsed_paths.contains(source.path.as_str())
                || !source_seen.insert((source.path.as_str(), source.line_start))
            {
                return Err(GeneratedApiImportError::new(
                    GeneratedApiImportErrorCode::InvalidConflict,
                    "generated API conflict source is invalid or duplicated",
                ));
            }
            sources.push(GeneratedApiConflictSource {
                path: source.path.clone().into_boxed_str(),
                line_start: source.line_start,
            });
        }
        sources.sort_by(|left, right| {
            (left.path.as_ref(), left.line_start).cmp(&(right.path.as_ref(), right.line_start))
        });
        output.push(GeneratedApiConflict {
            kind: conflict.kind.clone().into_boxed_str(),
            collection: conflict.collection.clone().into_boxed_str(),
            qualified_name: conflict.qualified_name.clone().into_boxed_str(),
            sources,
        });
    }
    Ok(output)
}

fn kind_for_collection(collection: &str) -> Option<GeneratedApiFactKind> {
    GeneratedApiFactKind::all()
        .into_iter()
        .find(|kind| kind.collection() == collection)
}

fn fact_sort_key(fact: &GeneratedApiFact) -> (GeneratedApiFactKind, &str, &str) {
    (fact.kind, &fact.qualified_name, &fact.fact_id)
}

fn validate_generated_path(path: &str) -> GeneratedApiImportResult<()> {
    if !path.starts_with(GENERATED_ROOT)
        || !path.ends_with(GENERATED_SUFFIX)
        || path.starts_with('/')
        || path.contains('\\')
        || path.split('/').any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidSource,
            "generated API source path is not canonical or is outside the generated-document root",
        ));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> GeneratedApiImportResult<()> {
    let digest = value.strip_prefix("sha256:").ok_or_else(|| {
        GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("{label} is not a canonical sha256 identifier"),
        )
    })?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidDigest,
            format!("{label} is not a canonical sha256 identifier"),
        ));
    }
    Ok(())
}

fn validate_object_id(
    value: &str,
    expected_length: Option<usize>,
    label: &str,
) -> GeneratedApiImportResult<()> {
    if !matches!(value.len(), 40 | 64)
        || expected_length.is_some_and(|length| length != value.len())
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(GeneratedApiImportError::new(
            GeneratedApiImportErrorCode::InvalidSource,
            format!("{label} is not a canonical Git object identifier"),
        ));
    }
    Ok(())
}

fn reject_nonempty_extensions(
    extensions: &BTreeMap<String, Value>,
    label: &str,
) -> GeneratedApiImportResult<()> {
    if extensions.is_empty() {
        return Ok(());
    }
    Err(GeneratedApiImportError::new(
        GeneratedApiImportErrorCode::UnsupportedSchema,
        format!("{label} contains fields unsupported by schema version {SCHEMA_VERSION}"),
    ))
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, json};

    fn source(path: &str, line_start: u64) -> Value {
        json!({
            "path": path,
            "line_start": line_start,
            "line_end": line_start + 2,
            "git_object": "1111111111111111111111111111111111111111",
            "sha256": "sha256:2222222222222222222222222222222222222222222222222222222222222222"
        })
    }

    fn base_draft(complete: bool, duplicate: bool) -> Value {
        let path = "Interface/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua";
        let function = json!({
            "name": "GetAuraDataByIndex",
            "qualified_name": "C_UnitAuras.GetAuraDataByIndex",
            "type": "Function",
            "documentation": ["Returns one aura record."],
            "restrictions": {"has_restrictions": true},
            "attributes": {},
            "arguments": [{"name": "unitToken", "type": "UnitToken", "source": source(path, 15)}],
            "returns": [{"name": "aura", "type": "AuraData", "source": source(path, 20)}],
            "source": source(path, 10)
        });
        let functions = if duplicate {
            vec![function.clone(), function]
        } else {
            vec![function]
        };
        let conflicts = if duplicate {
            vec![json!({
                "kind": "duplicate_symbol",
                "collection": "functions",
                "qualified_name": "C_UnitAuras.GetAuraDataByIndex",
                "sources": [
                    {"path": path, "line_start": 10},
                    {"path": path, "line_start": 30}
                ]
            })]
        } else {
            Vec::new()
        };
        let parsed_files = 1_u64;
        let failed_files = if complete { 0_u64 } else { 1_u64 };
        json!({
            "schema": SCHEMA,
            "schema_version": SCHEMA_VERSION,
            "producer": {
                "id": PRODUCER_ID,
                "version": 1,
                "parser": "declarative-lua-table-v1",
                "configuration": {
                    "generated_root": GENERATED_ROOT,
                    "generated_suffix": GENERATED_SUFFIX
                }
            },
            "source": {
                "manifest_sha256": "sha256:3333333333333333333333333333333333333333333333333333333333333333",
                "manifest_declared_digest": null,
                "source_id": "gethe-wow-ui-source",
                "selector": "live",
                "revision": "4444444444444444444444444444444444444444",
                "version": "99.1.2.34567"
            },
            "coverage": {
                "scope": format!("{GENERATED_ROOT}*{GENERATED_SUFFIX}"),
                "status": if complete { "complete" } else { "partial" },
                "negative_authority": complete,
                "candidate_files": parsed_files + failed_files,
                "parsed_files": parsed_files,
                "failed_files": failed_files,
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
            "conflicts": conflicts,
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
        let bytes = canonical_json_bytes(&value).map_err(|error| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidDigest,
                format!("test value cannot be canonicalized: {error}"),
            )
        })?;
        let object = value.as_object_mut().ok_or_else(|| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidJson,
                "test value is not an object",
            )
        })?;
        object.insert("draft_sha256".to_owned(), Value::String(sha256_prefixed(&bytes)));
        serde_json::to_vec(&value).map_err(|error| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidJson,
                format!("test value cannot be serialized: {error}"),
            )
        })
    }

    #[test]
    fn complete_import_supports_exact_lookup_and_authoritative_absence(
    ) -> GeneratedApiImportResult<()> {
        let bytes = seal(base_draft(true, false))?;
        let index = import_generated_api_draft(&bytes)?;
        match index.lookup(
            GeneratedApiFactKind::Function,
            "C_UnitAuras.GetAuraDataByIndex",
        ) {
            GeneratedApiLookup::Found(fact) => {
                assert!(fact.restrictions().contains_key("has_restrictions"));
            }
            other => {
                return Err(GeneratedApiImportError::new(
                    GeneratedApiImportErrorCode::InvalidFact,
                    format!("unexpected lookup result: {other:?}"),
                ));
            }
        }
        assert_eq!(
            index.lookup(GeneratedApiFactKind::Function, "C_UnitAuras.Missing"),
            GeneratedApiLookup::AbsentAuthoritative
        );
        assert!(index.index_id().starts_with("sha256:"));
        Ok(())
    }

    #[test]
    fn partial_coverage_never_proves_absence() -> GeneratedApiImportResult<()> {
        let bytes = seal(base_draft(false, false))?;
        let index = import_generated_api_draft(&bytes)?;
        assert_eq!(
            index.lookup(GeneratedApiFactKind::Function, "C_UnitAuras.Missing"),
            GeneratedApiLookup::NotAuthoritative
        );
        assert_eq!(index.coverage().status(), GeneratedApiCoverageStatus::Partial);
        assert!(!index.coverage().negative_authority());
        Ok(())
    }

    #[test]
    fn duplicate_facts_are_conflicted() -> GeneratedApiImportResult<()> {
        let bytes = seal(base_draft(true, true))?;
        let index = import_generated_api_draft(&bytes)?;
        match index.lookup(
            GeneratedApiFactKind::Function,
            "C_UnitAuras.GetAuraDataByIndex",
        ) {
            GeneratedApiLookup::Conflicted(facts) => assert_eq!(facts.len(), 2),
            other => {
                return Err(GeneratedApiImportError::new(
                    GeneratedApiImportErrorCode::InvalidConflict,
                    format!("unexpected lookup result: {other:?}"),
                ));
            }
        }
        assert_eq!(index.conflicts().len(), 1);
        Ok(())
    }

    #[test]
    fn tampered_draft_digest_is_rejected() -> GeneratedApiImportResult<()> {
        let bytes = seal(base_draft(true, false))?;
        let mut value = serde_json::from_slice::<Value>(&bytes).map_err(|error| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidJson,
                format!("test draft cannot be parsed: {error}"),
            )
        })?;
        let systems = value
            .get_mut("systems")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                GeneratedApiImportError::new(
                    GeneratedApiImportErrorCode::InvalidJson,
                    "test systems are missing",
                )
            })?;
        let system = systems.first_mut().and_then(Value::as_object_mut).ok_or_else(|| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidJson,
                "test system is missing",
            )
        })?;
        system.insert("name".to_owned(), Value::String("Tampered".to_owned()));
        let tampered = serde_json::to_vec(&value).map_err(|error| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidJson,
                format!("test draft cannot be serialized: {error}"),
            )
        })?;
        let error = import_generated_api_draft(&tampered).err().ok_or_else(|| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidDigest,
                "tampered draft unexpectedly imported",
            )
        })?;
        assert_eq!(error.code(), GeneratedApiImportErrorCode::InvalidDigest);
        Ok(())
    }

    #[test]
    fn unsupported_root_extension_is_rejected() -> GeneratedApiImportResult<()> {
        let mut value = base_draft(true, false);
        let object = value.as_object_mut().ok_or_else(|| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::InvalidJson,
                "test draft root is missing",
            )
        })?;
        object.insert("future_semantics".to_owned(), Value::Object(Map::new()));
        let bytes = seal(value)?;
        let error = import_generated_api_draft(&bytes).err().ok_or_else(|| {
            GeneratedApiImportError::new(
                GeneratedApiImportErrorCode::UnsupportedSchema,
                "unsupported extension unexpectedly imported",
            )
        })?;
        assert_eq!(error.code(), GeneratedApiImportErrorCode::UnsupportedSchema);
        Ok(())
    }
}

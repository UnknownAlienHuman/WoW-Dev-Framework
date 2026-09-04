//! Deterministic, generation-bound reference views.
//!
//! This module provides the smallest executable I0-B boundary: canonical
//! records grouped into explicitly covered partitions, preserved conflicts,
//! exact lookup, conservative negative authority, and a self-digest over the
//! complete unsigned view.

use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Schema identifier for deterministic reference views.
pub const REFERENCE_VIEW_SCHEMA: &str = "wow-reference/reference-view/v1";

/// Result type for deterministic reference-view operations.
pub type ReferenceViewResult<T> = Result<T, ReferenceViewError>;

/// Stable record families exposed by a reference view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRecordKind {
    /// Callable API surface.
    Api,
    /// Event or callback surface.
    Event,
    /// Global value or namespace.
    Global,
    /// Widget or frame surface.
    Widget,
    /// Enumeration or constant family.
    Enumeration,
    /// Restriction or security fact.
    Restriction,
    /// Another explicitly identified reference family.
    Other,
}

/// Coverage state for one reference partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    /// The partition was completely evaluated for this generation.
    Complete,
    /// Only a bounded subset of the partition was evaluated.
    Partial,
    /// The partition was not evaluated.
    NotEvaluated,
}

/// State of one restriction facet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionState {
    /// The operation or surface is allowed in the described context.
    Allowed,
    /// The operation is conditionally restricted.
    Restricted,
    /// The operation is forbidden in the described context.
    Forbidden,
    /// The available evidence cannot establish a state.
    Unknown,
}

/// Why an exact lookup cannot provide positive or negative authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LookupUnknownReason {
    /// The requested partition is absent from the view.
    PartitionMissing,
    /// The partition exists but is only partially covered.
    PartialCoverage,
    /// The partition was explicitly not evaluated.
    NotEvaluated,
}

/// Exact lookup outcome.
///
/// `AuthoritativeAbsence` is emitted only for a conflict-free key in a
/// completely covered partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupResult<'a> {
    /// One exact record was found.
    Found(&'a ReferenceRecord),
    /// Complete coverage proves that the key is absent.
    AuthoritativeAbsence,
    /// Conflicting candidates prevent a single answer.
    Conflict(&'a ReferenceConflict),
    /// Coverage is insufficient for either a positive or negative answer.
    Unknown(LookupUnknownReason),
}

/// Validation or canonicalization failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceViewError {
    /// A required identifier is missing or non-canonical.
    InvalidIdentifier {
        /// Stable field path.
        field: &'static str,
        /// Rejected identifier.
        value: Box<str>,
    },
    /// A bounded text field is empty or exceeds its limit.
    InvalidText {
        /// Stable field path.
        field: &'static str,
    },
    /// A collection contains a duplicate key.
    DuplicateKey {
        /// Stable collection path.
        field: &'static str,
        /// Duplicate key.
        value: Box<str>,
    },
    /// A collection is not in its canonical strict order.
    NonCanonicalOrder {
        /// Stable collection path.
        field: &'static str,
    },
    /// A conflict contains fewer than two distinct candidates.
    InsufficientConflictCandidates,
    /// A conflict refers to a partition absent from the view.
    MissingConflictPartition {
        /// Missing partition identifier.
        partition_id: Box<str>,
    },
    /// A SHA-256 digest is malformed.
    InvalidDigest {
        /// Stable field path.
        field: &'static str,
        /// Rejected digest.
        value: Box<str>,
    },
    /// The stored view digest differs from the recomputed digest.
    DigestMismatch {
        /// Stored digest.
        stored: Box<str>,
        /// Recomputed digest.
        computed: Box<str>,
    },
    /// Serialization failed.
    Serialization {
        /// Serializer detail.
        detail: Box<str>,
    },
}

impl fmt::Display for ReferenceViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier { field, value } => {
                write!(formatter, "invalid identifier at {field}: {value}")
            }
            Self::InvalidText { field } => write!(formatter, "invalid text at {field}"),
            Self::DuplicateKey { field, value } => {
                write!(formatter, "duplicate key at {field}: {value}")
            }
            Self::NonCanonicalOrder { field } => {
                write!(formatter, "non-canonical order at {field}")
            }
            Self::InsufficientConflictCandidates => {
                formatter.write_str("a reference conflict requires at least two candidates")
            }
            Self::MissingConflictPartition { partition_id } => {
                write!(formatter, "conflict partition is missing: {partition_id}")
            }
            Self::InvalidDigest { field, value } => {
                write!(formatter, "invalid digest at {field}: {value}")
            }
            Self::DigestMismatch { stored, computed } => {
                write!(
                    formatter,
                    "reference-view digest mismatch: {stored} != {computed}"
                )
            }
            Self::Serialization { detail } => {
                write!(formatter, "reference-view serialization failed: {detail}")
            }
        }
    }
}

impl Error for ReferenceViewError {}

/// One context-sensitive restriction attached to a reference record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictionFacet {
    id: Box<str>,
    state: RestrictionState,
    evidence_ids: Box<[Box<str>]>,
}

impl RestrictionFacet {
    /// Builds a canonical restriction facet.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier or evidence list is invalid.
    pub fn new(
        id: impl Into<String>,
        state: RestrictionState,
        evidence_ids: Vec<String>,
    ) -> ReferenceViewResult<Self> {
        let id = id.into();
        validate_identifier("restriction.id", &id)?;
        let evidence_ids = canonical_ids("restriction.evidence_ids", evidence_ids)?;
        Ok(Self {
            id: id.into_boxed_str(),
            state,
            evidence_ids,
        })
    }

    /// Restriction identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Restriction state.
    #[must_use]
    pub const fn state(&self) -> RestrictionState {
        self.state
    }

    /// Evidence identifiers in canonical order.
    #[must_use]
    pub fn evidence_ids(&self) -> &[Box<str>] {
        &self.evidence_ids
    }

    fn validate(&self) -> ReferenceViewResult<()> {
        validate_identifier("restriction.id", self.id())?;
        validate_id_slice("restriction.evidence_ids", self.evidence_ids())
    }
}

/// One exact reference record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceRecord {
    key: Box<str>,
    kind: ReferenceRecordKind,
    payload: Box<str>,
    source_ids: Box<[Box<str>]>,
    restrictions: Box<[RestrictionFacet]>,
}

impl ReferenceRecord {
    /// Builds one canonical record.
    ///
    /// # Errors
    ///
    /// Returns an error when the key, payload, sources, or restrictions are
    /// invalid or duplicate.
    pub fn new(
        key: impl Into<String>,
        kind: ReferenceRecordKind,
        payload: impl Into<String>,
        source_ids: Vec<String>,
        mut restrictions: Vec<RestrictionFacet>,
    ) -> ReferenceViewResult<Self> {
        let key = key.into();
        validate_identifier("record.key", &key)?;
        let payload = payload.into();
        validate_payload(&payload)?;
        let source_ids = canonical_ids("record.source_ids", source_ids)?;
        restrictions.sort_by(|left, right| left.id().cmp(right.id()));
        validate_restrictions(&restrictions)?;
        Ok(Self {
            key: key.into_boxed_str(),
            kind,
            payload: payload.into_boxed_str(),
            source_ids,
            restrictions: restrictions.into_boxed_slice(),
        })
    }

    /// Canonical lookup key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Record family.
    #[must_use]
    pub const fn kind(&self) -> ReferenceRecordKind {
        self.kind
    }

    /// Canonical payload.
    #[must_use]
    pub fn payload(&self) -> &str {
        &self.payload
    }

    /// Source identifiers in canonical order.
    #[must_use]
    pub fn source_ids(&self) -> &[Box<str>] {
        &self.source_ids
    }

    /// Restriction facets in canonical order.
    #[must_use]
    pub fn restrictions(&self) -> &[RestrictionFacet] {
        &self.restrictions
    }

    /// Computes the deterministic SHA-256 digest of this record.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn digest(&self) -> ReferenceViewResult<String> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(serialization_error)?;
        Ok(sha256_digest(&bytes))
    }

    fn validate(&self) -> ReferenceViewResult<()> {
        validate_identifier("record.key", self.key())?;
        validate_payload(self.payload())?;
        validate_id_slice("record.source_ids", self.source_ids())?;
        validate_restrictions(self.restrictions())
    }
}

/// One preserved set of mutually incompatible candidates.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceConflict {
    partition_id: Box<str>,
    key: Box<str>,
    candidate_digests: Box<[Box<str>]>,
    source_ids: Box<[Box<str>]>,
}

impl ReferenceConflict {
    /// Builds one canonical conflict.
    ///
    /// # Errors
    ///
    /// Returns an error when identifiers, candidate digests, or source
    /// identifiers are invalid.
    pub fn new(
        partition_id: impl Into<String>,
        key: impl Into<String>,
        candidate_digests: Vec<String>,
        source_ids: Vec<String>,
    ) -> ReferenceViewResult<Self> {
        let partition_id = partition_id.into();
        let key = key.into();
        validate_identifier("conflict.partition_id", &partition_id)?;
        validate_identifier("conflict.key", &key)?;
        let candidate_digests = canonical_digests(candidate_digests)?;
        if candidate_digests.len() < 2 {
            return Err(ReferenceViewError::InsufficientConflictCandidates);
        }
        let source_ids = canonical_ids("conflict.source_ids", source_ids)?;
        Ok(Self {
            partition_id: partition_id.into_boxed_str(),
            key: key.into_boxed_str(),
            candidate_digests,
            source_ids,
        })
    }

    /// Partition identifier.
    #[must_use]
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }

    /// Conflicting lookup key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Candidate record digests in canonical order.
    #[must_use]
    pub fn candidate_digests(&self) -> &[Box<str>] {
        &self.candidate_digests
    }

    /// Source identifiers in canonical order.
    #[must_use]
    pub fn source_ids(&self) -> &[Box<str>] {
        &self.source_ids
    }

    fn validate(&self) -> ReferenceViewResult<()> {
        validate_identifier("conflict.partition_id", self.partition_id())?;
        validate_identifier("conflict.key", self.key())?;
        validate_digest_slice(self.candidate_digests())?;
        if self.candidate_digests.len() < 2 {
            return Err(ReferenceViewError::InsufficientConflictCandidates);
        }
        validate_id_slice("conflict.source_ids", self.source_ids())
    }
}

/// One independently covered reference partition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePartition {
    id: Box<str>,
    coverage: CoverageStatus,
    records: Box<[ReferenceRecord]>,
}

impl ReferencePartition {
    /// Builds one canonical partition.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier or records are invalid.
    pub fn new(
        id: impl Into<String>,
        coverage: CoverageStatus,
        mut records: Vec<ReferenceRecord>,
    ) -> ReferenceViewResult<Self> {
        let id = id.into();
        validate_identifier("partition.id", &id)?;
        records.sort_by(|left, right| left.key().cmp(right.key()));
        validate_records(&records)?;
        Ok(Self {
            id: id.into_boxed_str(),
            coverage,
            records: records.into_boxed_slice(),
        })
    }

    /// Partition identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Coverage state.
    #[must_use]
    pub const fn coverage(&self) -> CoverageStatus {
        self.coverage
    }

    /// Records in canonical key order.
    #[must_use]
    pub fn records(&self) -> &[ReferenceRecord] {
        &self.records
    }

    fn validate(&self) -> ReferenceViewResult<()> {
        validate_identifier("partition.id", self.id())?;
        validate_records(self.records())
    }
}

/// Generation-bound deterministic reference view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceView {
    schema: Box<str>,
    generation_id: Box<str>,
    partitions: Box<[ReferencePartition]>,
    conflicts: Box<[ReferenceConflict]>,
    self_digest: Box<str>,
}

impl ReferenceView {
    /// Builds, canonicalizes, and self-digests a reference view.
    ///
    /// # Errors
    ///
    /// Returns an error when any nested record is invalid or serialization
    /// fails.
    pub fn new(
        generation_id: impl Into<String>,
        mut partitions: Vec<ReferencePartition>,
        mut conflicts: Vec<ReferenceConflict>,
    ) -> ReferenceViewResult<Self> {
        let generation_id = generation_id.into();
        validate_identifier("view.generation_id", &generation_id)?;
        partitions.sort_by(|left, right| left.id().cmp(right.id()));
        conflicts.sort_by(|left, right| {
            (left.partition_id(), left.key()).cmp(&(right.partition_id(), right.key()))
        });
        let mut view = Self {
            schema: REFERENCE_VIEW_SCHEMA.into(),
            generation_id: generation_id.into_boxed_str(),
            partitions: partitions.into_boxed_slice(),
            conflicts: conflicts.into_boxed_slice(),
            self_digest: String::new().into_boxed_str(),
        };
        view.validate_contents()?;
        view.self_digest = view.recompute_digest()?.into_boxed_str();
        view.validate()?;
        Ok(view)
    }

    /// View schema.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Bound reference generation.
    #[must_use]
    pub fn generation_id(&self) -> &str {
        &self.generation_id
    }

    /// Covered partitions in canonical order.
    #[must_use]
    pub fn partitions(&self) -> &[ReferencePartition] {
        &self.partitions
    }

    /// Preserved conflicts in canonical order.
    #[must_use]
    pub fn conflicts(&self) -> &[ReferenceConflict] {
        &self.conflicts
    }

    /// Stored self-digest.
    #[must_use]
    pub fn self_digest(&self) -> &str {
        &self.self_digest
    }

    /// Validates nested invariants and the stored self-digest.
    ///
    /// # Errors
    ///
    /// Returns an error on any contract violation.
    pub fn validate(&self) -> ReferenceViewResult<()> {
        if self.schema() != REFERENCE_VIEW_SCHEMA {
            return Err(ReferenceViewError::InvalidIdentifier {
                field: "view.schema",
                value: self.schema().into(),
            });
        }
        self.validate_contents()?;
        validate_digest("view.self_digest", self.self_digest())?;
        let computed = self.recompute_digest()?;
        if computed != self.self_digest() {
            return Err(ReferenceViewError::DigestMismatch {
                stored: self.self_digest().into(),
                computed: computed.into_boxed_str(),
            });
        }
        Ok(())
    }

    /// Returns canonical JSON bytes including the verified self-digest.
    ///
    /// # Errors
    ///
    /// Returns an error when validation or serialization fails.
    pub fn canonical_bytes(&self) -> ReferenceViewResult<Vec<u8>> {
        self.validate()?;
        serde_json::to_vec(self).map_err(serialization_error)
    }

    /// Recomputes the digest over the complete unsigned view.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn recompute_digest(&self) -> ReferenceViewResult<String> {
        let unsigned = UnsignedReferenceView {
            schema: self.schema(),
            generation_id: self.generation_id(),
            partitions: self.partitions(),
            conflicts: self.conflicts(),
        };
        let bytes = serde_json::to_vec(&unsigned).map_err(serialization_error)?;
        Ok(sha256_digest(&bytes))
    }

    /// Performs an exact lookup under conservative negative-authority rules.
    #[must_use]
    pub fn lookup(&self, partition_id: &str, key: &str) -> LookupResult<'_> {
        if let Some(conflict) = self
            .conflicts
            .iter()
            .find(|conflict| conflict.partition_id() == partition_id && conflict.key() == key)
        {
            return LookupResult::Conflict(conflict);
        }

        let Some(partition) = self
            .partitions
            .iter()
            .find(|partition| partition.id() == partition_id)
        else {
            return LookupResult::Unknown(LookupUnknownReason::PartitionMissing);
        };

        if let Some(record) = partition
            .records()
            .iter()
            .find(|record| record.key() == key)
        {
            return LookupResult::Found(record);
        }

        match partition.coverage() {
            CoverageStatus::Complete => LookupResult::AuthoritativeAbsence,
            CoverageStatus::Partial => LookupResult::Unknown(LookupUnknownReason::PartialCoverage),
            CoverageStatus::NotEvaluated => {
                LookupResult::Unknown(LookupUnknownReason::NotEvaluated)
            }
        }
    }

    /// Whether complete coverage proves the key absent.
    #[must_use]
    pub fn is_authoritatively_absent(&self, partition_id: &str, key: &str) -> bool {
        matches!(
            self.lookup(partition_id, key),
            LookupResult::AuthoritativeAbsence
        )
    }

    fn validate_contents(&self) -> ReferenceViewResult<()> {
        validate_identifier("view.generation_id", self.generation_id())?;
        validate_partition_slice(self.partitions())?;
        validate_conflict_slice(self.conflicts())?;
        for conflict in self.conflicts() {
            if !self
                .partitions()
                .iter()
                .any(|partition| partition.id() == conflict.partition_id())
            {
                return Err(ReferenceViewError::MissingConflictPartition {
                    partition_id: conflict.partition_id().into(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct UnsignedReferenceView<'a> {
    schema: &'a str,
    generation_id: &'a str,
    partitions: &'a [ReferencePartition],
    conflicts: &'a [ReferenceConflict],
}

fn validate_identifier(field: &'static str, value: &str) -> ReferenceViewResult<()> {
    let mut bytes = value.bytes();
    let first_valid = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphanumeric());
    let rest_valid = bytes.all(|byte| {
        byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'_' | b'-' | b'.' | b':' | b'/' | b'%' | b'+' | b'@' | b'#'
            )
    });
    if value.len() > 512 || !first_valid || !rest_valid {
        return Err(ReferenceViewError::InvalidIdentifier {
            field,
            value: value.into(),
        });
    }
    Ok(())
}

fn validate_payload(payload: &str) -> ReferenceViewResult<()> {
    if payload.is_empty() || payload.len() > 65_536 || payload.contains('\0') {
        return Err(ReferenceViewError::InvalidText {
            field: "record.payload",
        });
    }
    Ok(())
}

fn canonical_ids(
    field: &'static str,
    mut values: Vec<String>,
) -> ReferenceViewResult<Box<[Box<str>]>> {
    for value in &values {
        validate_identifier(field, value)?;
    }
    values.sort();
    for pair in values.windows(2) {
        if pair[0] == pair[1] {
            return Err(ReferenceViewError::DuplicateKey {
                field,
                value: pair[0].clone().into_boxed_str(),
            });
        }
    }
    Ok(values
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn canonical_digests(mut values: Vec<String>) -> ReferenceViewResult<Box<[Box<str>]>> {
    for value in &values {
        validate_digest("conflict.candidate_digests", value)?;
    }
    values.sort();
    for pair in values.windows(2) {
        if pair[0] == pair[1] {
            return Err(ReferenceViewError::DuplicateKey {
                field: "conflict.candidate_digests",
                value: pair[0].clone().into_boxed_str(),
            });
        }
    }
    Ok(values
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn validate_id_slice(field: &'static str, values: &[Box<str>]) -> ReferenceViewResult<()> {
    for value in values {
        validate_identifier(field, value)?;
    }
    if !is_strictly_sorted(values) {
        return Err(ReferenceViewError::NonCanonicalOrder { field });
    }
    Ok(())
}

fn validate_digest_slice(values: &[Box<str>]) -> ReferenceViewResult<()> {
    for value in values {
        validate_digest("conflict.candidate_digests", value)?;
    }
    if !is_strictly_sorted(values) {
        return Err(ReferenceViewError::NonCanonicalOrder {
            field: "conflict.candidate_digests",
        });
    }
    Ok(())
}

fn validate_restrictions(values: &[RestrictionFacet]) -> ReferenceViewResult<()> {
    for value in values {
        value.validate()?;
    }
    for pair in values.windows(2) {
        if pair[0].id() >= pair[1].id() {
            let error = if pair[0].id() == pair[1].id() {
                ReferenceViewError::DuplicateKey {
                    field: "record.restrictions",
                    value: pair[0].id().into(),
                }
            } else {
                ReferenceViewError::NonCanonicalOrder {
                    field: "record.restrictions",
                }
            };
            return Err(error);
        }
    }
    Ok(())
}

fn validate_records(values: &[ReferenceRecord]) -> ReferenceViewResult<()> {
    for value in values {
        value.validate()?;
    }
    for pair in values.windows(2) {
        if pair[0].key() >= pair[1].key() {
            let error = if pair[0].key() == pair[1].key() {
                ReferenceViewError::DuplicateKey {
                    field: "partition.records",
                    value: pair[0].key().into(),
                }
            } else {
                ReferenceViewError::NonCanonicalOrder {
                    field: "partition.records",
                }
            };
            return Err(error);
        }
    }
    Ok(())
}

fn validate_partition_slice(values: &[ReferencePartition]) -> ReferenceViewResult<()> {
    for value in values {
        value.validate()?;
    }
    for pair in values.windows(2) {
        if pair[0].id() >= pair[1].id() {
            let error = if pair[0].id() == pair[1].id() {
                ReferenceViewError::DuplicateKey {
                    field: "view.partitions",
                    value: pair[0].id().into(),
                }
            } else {
                ReferenceViewError::NonCanonicalOrder {
                    field: "view.partitions",
                }
            };
            return Err(error);
        }
    }
    Ok(())
}

fn validate_conflict_slice(values: &[ReferenceConflict]) -> ReferenceViewResult<()> {
    for value in values {
        value.validate()?;
    }
    for pair in values.windows(2) {
        let left = (pair[0].partition_id(), pair[0].key());
        let right = (pair[1].partition_id(), pair[1].key());
        if left >= right {
            let error = if left == right {
                ReferenceViewError::DuplicateKey {
                    field: "view.conflicts",
                    value: format!("{}:{}", pair[0].partition_id(), pair[0].key()).into_boxed_str(),
                }
            } else {
                ReferenceViewError::NonCanonicalOrder {
                    field: "view.conflicts",
                }
            };
            return Err(error);
        }
    }
    Ok(())
}

fn is_strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn validate_digest(field: &'static str, value: &str) -> ReferenceViewResult<()> {
    let valid = value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    if !valid {
        return Err(ReferenceViewError::InvalidDigest {
            field,
            value: value.into(),
        });
    }
    Ok(())
}

fn sha256_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn serialization_error(error: serde_json::Error) -> ReferenceViewError {
    ReferenceViewError::Serialization {
        detail: error.to_string().into_boxed_str(),
    }
}

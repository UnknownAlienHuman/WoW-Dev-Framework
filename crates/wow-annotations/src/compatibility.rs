//! Canonical parity and consumer-probe evidence for native annotation artifacts.
//!
//! External adapters perform oracle extraction and consumer execution. This owner
//! validates their bounded canonical reports and exact artifact/profile/generation
//! bindings. It never starts a process, mutates editor configuration or upgrades
//! candidate evidence into ReferenceView authority.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

pub const NATIVE_PARITY_REPORT_SCHEMA: &str = "wow-annotations/native-semantic-parity/1";
pub const NATIVE_CONSUMER_PROBE_SCHEMA: &str = "wow-annotations/native-consumer-probe/1";
pub const NATIVE_COMPATIBILITY_EVIDENCE_SCHEMA: &str =
    "wow-annotations/native-compatibility-evidence/1";
pub const MAX_COMPATIBILITY_REPORT_BYTES: usize = 8 * 1024 * 1024;
const MAX_RECORDS: usize = 262_144;
const MAX_CONSUMERS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeCompatibilityErrorCode {
    InvalidBinding,
    InvalidParityReport,
    InvalidConsumerProbe,
    IdentityMismatch,
    InputLimit,
    SerializationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCompatibilityError {
    code: NativeCompatibilityErrorCode,
    message: Box<str>,
}

impl NativeCompatibilityError {
    fn new(code: NativeCompatibilityErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> NativeCompatibilityErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for NativeCompatibilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for NativeCompatibilityError {}

pub type NativeCompatibilityResult<T> = Result<T, NativeCompatibilityError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeCompatibilityStatus {
    Passed,
    Failed,
    NotEvaluated,
}

impl NativeCompatibilityStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::NotEvaluated => "not_evaluated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeParityClassification {
    Equal,
    SemanticallyEquivalent,
    ExpectedProjectionDifference,
    OurDefect,
    OracleDefectOrStale,
    InputMismatch,
    ConsumerDisagreement,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeConsumerKind {
    EmmyLua,
    LuaLs,
}

impl NativeConsumerKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmmyLua => "emmylua",
            Self::LuaLs => "luals",
        }
    }

    #[must_use]
    pub const fn member_name(self) -> &'static str {
        match self {
            Self::EmmyLua => "emmylua.json",
            Self::LuaLs => "luals.json",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeConsumerAssertionKind {
    Positive,
    Negative,
    SemanticType,
    SourceSpan,
    DiagnosticBaseline,
    ConfigurationMutation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCompatibilityBinding {
    annotation_artifact_id: Box<str>,
    source_map_id: Box<str>,
    loss_report_id: Box<str>,
    profile_id: Box<str>,
    source_generation_id: Box<str>,
}

impl NativeCompatibilityBinding {
    pub fn new(
        annotation_artifact_id: impl Into<Box<str>>,
        source_map_id: impl Into<Box<str>>,
        loss_report_id: impl Into<Box<str>>,
        profile_id: impl Into<Box<str>>,
        source_generation_id: impl Into<Box<str>>,
    ) -> NativeCompatibilityResult<Self> {
        let value = Self {
            annotation_artifact_id: annotation_artifact_id.into(),
            source_map_id: source_map_id.into(),
            loss_report_id: loss_report_id.into(),
            profile_id: profile_id.into(),
            source_generation_id: source_generation_id.into(),
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> NativeCompatibilityResult<()> {
        if !valid_component(&self.annotation_artifact_id)
            || !valid_component(&self.source_map_id)
            || !valid_component(&self.loss_report_id)
            || !valid_component(&self.profile_id)
            || !valid_component(&self.source_generation_id)
        {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidBinding,
                "native compatibility binding contains an invalid identity",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn annotation_artifact_id(&self) -> &str {
        &self.annotation_artifact_id
    }

    #[must_use]
    pub fn source_map_id(&self) -> &str {
        &self.source_map_id
    }

    #[must_use]
    pub fn loss_report_id(&self) -> &str {
        &self.loss_report_id
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub fn source_generation_id(&self) -> &str {
        &self.source_generation_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeParityRecord {
    record_id: Box<str>,
    subject_id: Box<str>,
    our_semantic_id: Box<str>,
    oracle_semantic_id: Box<str>,
    comparison_rule_id: Box<str>,
    difference_sha256: Box<str>,
    classification: NativeParityClassification,
    mandatory: bool,
}

impl NativeParityRecord {
    fn validate(&self) -> NativeCompatibilityResult<()> {
        if !valid_component(&self.subject_id)
            || !valid_component(&self.our_semantic_id)
            || !valid_component(&self.oracle_semantic_id)
            || !valid_component(&self.comparison_rule_id)
            || !canonical_sha256(&self.difference_sha256)
            || self.record_id != parity_record_id(self)?
        {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidParityReport,
                "native parity record identity or evidence is invalid",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn record_id(&self) -> &str {
        &self.record_id
    }

    #[must_use]
    pub const fn classification(&self) -> NativeParityClassification {
        self.classification
    }

    #[must_use]
    pub const fn mandatory(&self) -> bool {
        self.mandatory
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSemanticParityReport {
    schema: Box<str>,
    report_id: Box<str>,
    annotation_artifact_id: Box<str>,
    source_map_id: Box<str>,
    loss_report_id: Box<str>,
    profile_id: Box<str>,
    source_generation_id: Box<str>,
    oracle_kind: Box<str>,
    oracle_revision: Box<str>,
    oracle_artifact_sha256: Box<str>,
    oracle_profile_id: Box<str>,
    comparison_profile_id: Box<str>,
    source_equivalence: NativeCompatibilityStatus,
    records: Box<[NativeParityRecord]>,
}

impl NativeSemanticParityReport {
    pub fn from_canonical_slice(bytes: &[u8]) -> NativeCompatibilityResult<Self> {
        bounded(bytes, NativeCompatibilityErrorCode::InvalidParityReport)?;
        let value: Self = serde_json::from_slice(bytes).map_err(|_| {
            failure(
                NativeCompatibilityErrorCode::InvalidParityReport,
                "native parity report failed strict decoding",
            )
        })?;
        value.validate()?;
        if value.canonical_bytes()?.as_ref() != bytes {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidParityReport,
                "native parity report bytes are not canonical",
            ));
        }
        Ok(value)
    }

    pub fn validate(&self) -> NativeCompatibilityResult<()> {
        if self.schema.as_ref() != NATIVE_PARITY_REPORT_SCHEMA
            || !valid_component(&self.annotation_artifact_id)
            || !valid_component(&self.source_map_id)
            || !valid_component(&self.loss_report_id)
            || !valid_component(&self.profile_id)
            || !valid_component(&self.source_generation_id)
            || !valid_component(&self.oracle_kind)
            || !exact_revision(&self.oracle_revision)
            || !canonical_sha256(&self.oracle_artifact_sha256)
            || !valid_component(&self.oracle_profile_id)
            || !valid_component(&self.comparison_profile_id)
            || self.records.is_empty()
            || self.records.len() > MAX_RECORDS
            || !self.records.iter().any(|record| record.mandatory)
        {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidParityReport,
                "native parity report header or bounds are invalid",
            ));
        }
        for record in &self.records {
            record.validate()?;
        }
        if !strictly_sorted_unique(self.records.iter().map(|record| record.record_id.as_ref()))
            || self.report_id != parity_report_id(self)?
        {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidParityReport,
                "native parity report records or identity are not canonical",
            ));
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> NativeCompatibilityResult<Box<[u8]>> {
        canonical_bounded(
            self,
            NativeCompatibilityErrorCode::InvalidParityReport,
            "native parity report cannot be canonicalized",
        )
    }

    #[must_use]
    pub fn status(&self) -> NativeCompatibilityStatus {
        match self.source_equivalence {
            NativeCompatibilityStatus::Failed => return NativeCompatibilityStatus::Failed,
            NativeCompatibilityStatus::NotEvaluated => {
                return NativeCompatibilityStatus::NotEvaluated;
            }
            NativeCompatibilityStatus::Passed => {}
        }
        let mut status = NativeCompatibilityStatus::Passed;
        for record in self.records.iter().filter(|record| record.mandatory) {
            match record.classification {
                NativeParityClassification::Equal
                | NativeParityClassification::SemanticallyEquivalent => {}
                NativeParityClassification::OurDefect
                | NativeParityClassification::InputMismatch
                | NativeParityClassification::ConsumerDisagreement => {
                    return NativeCompatibilityStatus::Failed;
                }
                NativeParityClassification::ExpectedProjectionDifference
                | NativeParityClassification::OracleDefectOrStale
                | NativeParityClassification::Unresolved => {
                    status = NativeCompatibilityStatus::NotEvaluated;
                }
            }
        }
        status
    }

    fn matches_binding(&self, binding: &NativeCompatibilityBinding) -> bool {
        self.annotation_artifact_id.as_ref() == binding.annotation_artifact_id()
            && self.source_map_id.as_ref() == binding.source_map_id()
            && self.loss_report_id.as_ref() == binding.loss_report_id()
            && self.profile_id.as_ref() == binding.profile_id()
            && self.source_generation_id.as_ref() == binding.source_generation_id()
    }

    #[must_use]
    pub fn report_id(&self) -> &str {
        &self.report_id
    }

    #[must_use]
    pub fn annotation_artifact_id(&self) -> &str {
        &self.annotation_artifact_id
    }

    #[must_use]
    pub fn source_map_id(&self) -> &str {
        &self.source_map_id
    }

    #[must_use]
    pub fn loss_report_id(&self) -> &str {
        &self.loss_report_id
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub fn source_generation_id(&self) -> &str {
        &self.source_generation_id
    }

    #[must_use]
    pub fn records(&self) -> &[NativeParityRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeConsumerAssertionRecord {
    assertion_id: Box<str>,
    kind: NativeConsumerAssertionKind,
    subject_id: Box<str>,
    status: NativeCompatibilityStatus,
    evidence_sha256: Box<str>,
    mandatory: bool,
}

impl NativeConsumerAssertionRecord {
    fn validate(&self) -> NativeCompatibilityResult<()> {
        if !valid_component(&self.subject_id)
            || !canonical_sha256(&self.evidence_sha256)
            || self.assertion_id != consumer_assertion_id(self)?
        {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidConsumerProbe,
                "native consumer assertion identity or evidence is invalid",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn assertion_id(&self) -> &str {
        &self.assertion_id
    }

    #[must_use]
    pub const fn kind(&self) -> NativeConsumerAssertionKind {
        self.kind
    }

    #[must_use]
    pub const fn status(&self) -> NativeCompatibilityStatus {
        self.status
    }

    #[must_use]
    pub const fn mandatory(&self) -> bool {
        self.mandatory
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeConsumerProbeResult {
    schema: Box<str>,
    result_id: Box<str>,
    annotation_artifact_id: Box<str>,
    source_map_id: Box<str>,
    loss_report_id: Box<str>,
    profile_id: Box<str>,
    source_generation_id: Box<str>,
    consumer_kind: NativeConsumerKind,
    consumer_version: Box<str>,
    consumer_revision: Box<str>,
    executable_sha256: Box<str>,
    capability_profile_id: Box<str>,
    probe_manifest_sha256: Box<str>,
    raw_output_sha256: Box<str>,
    load_status: NativeCompatibilityStatus,
    index_status: NativeCompatibilityStatus,
    forbidden_mutation_count: u64,
    suppressed_diagnostic_count: u64,
    assertions: Box<[NativeConsumerAssertionRecord]>,
}

impl NativeConsumerProbeResult {
    pub fn from_canonical_slice(bytes: &[u8]) -> NativeCompatibilityResult<Self> {
        bounded(bytes, NativeCompatibilityErrorCode::InvalidConsumerProbe)?;
        let value: Self = serde_json::from_slice(bytes).map_err(|_| {
            failure(
                NativeCompatibilityErrorCode::InvalidConsumerProbe,
                "native consumer probe failed strict decoding",
            )
        })?;
        value.validate()?;
        if value.canonical_bytes()?.as_ref() != bytes {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidConsumerProbe,
                "native consumer probe bytes are not canonical",
            ));
        }
        Ok(value)
    }

    pub fn validate(&self) -> NativeCompatibilityResult<()> {
        if self.schema.as_ref() != NATIVE_CONSUMER_PROBE_SCHEMA
            || !valid_component(&self.annotation_artifact_id)
            || !valid_component(&self.source_map_id)
            || !valid_component(&self.loss_report_id)
            || !valid_component(&self.profile_id)
            || !valid_component(&self.source_generation_id)
            || !valid_component(&self.consumer_version)
            || !exact_revision(&self.consumer_revision)
            || !canonical_sha256(&self.executable_sha256)
            || !valid_component(&self.capability_profile_id)
            || !canonical_sha256(&self.probe_manifest_sha256)
            || !canonical_sha256(&self.raw_output_sha256)
            || self.assertions.is_empty()
            || self.assertions.len() > MAX_RECORDS
        {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidConsumerProbe,
                "native consumer probe header or bounds are invalid",
            ));
        }
        for assertion in &self.assertions {
            assertion.validate()?;
        }
        if !strictly_sorted_unique(
            self.assertions
                .iter()
                .map(|assertion| assertion.assertion_id.as_ref()),
        ) || !required_assertion_kinds_present(&self.assertions)
            || self.result_id != consumer_result_id(self)?
        {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidConsumerProbe,
                "native consumer assertions or result identity are not canonical",
            ));
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> NativeCompatibilityResult<Box<[u8]>> {
        canonical_bounded(
            self,
            NativeCompatibilityErrorCode::InvalidConsumerProbe,
            "native consumer probe cannot be canonicalized",
        )
    }

    #[must_use]
    pub fn status(&self) -> NativeCompatibilityStatus {
        if self.load_status == NativeCompatibilityStatus::Failed
            || self.index_status == NativeCompatibilityStatus::Failed
            || self.forbidden_mutation_count != 0
            || self.suppressed_diagnostic_count != 0
            || self.assertions.iter().any(|record| {
                record.mandatory && record.status == NativeCompatibilityStatus::Failed
            })
        {
            return NativeCompatibilityStatus::Failed;
        }
        if self.load_status != NativeCompatibilityStatus::Passed
            || self.index_status != NativeCompatibilityStatus::Passed
            || self.assertions.iter().any(|record| {
                record.mandatory && record.status != NativeCompatibilityStatus::Passed
            })
        {
            return NativeCompatibilityStatus::NotEvaluated;
        }
        NativeCompatibilityStatus::Passed
    }

    fn matches_binding(&self, binding: &NativeCompatibilityBinding) -> bool {
        self.annotation_artifact_id.as_ref() == binding.annotation_artifact_id()
            && self.source_map_id.as_ref() == binding.source_map_id()
            && self.loss_report_id.as_ref() == binding.loss_report_id()
            && self.profile_id.as_ref() == binding.profile_id()
            && self.source_generation_id.as_ref() == binding.source_generation_id()
    }

    #[must_use]
    pub fn result_id(&self) -> &str {
        &self.result_id
    }

    #[must_use]
    pub const fn consumer_kind(&self) -> NativeConsumerKind {
        self.consumer_kind
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub fn source_generation_id(&self) -> &str {
        &self.source_generation_id
    }

    #[must_use]
    pub fn assertions(&self) -> &[NativeConsumerAssertionRecord] {
        &self.assertions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeConsumerProbeArtifact {
    result: NativeConsumerProbeResult,
    bytes: Box<[u8]>,
}

impl NativeConsumerProbeArtifact {
    #[must_use]
    pub fn result(&self) -> &NativeConsumerProbeResult {
        &self.result
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCompatibilityEvidence {
    binding: NativeCompatibilityBinding,
    evidence_id: Box<str>,
    parity_report: NativeSemanticParityReport,
    parity_bytes: Box<[u8]>,
    consumers: Box<[NativeConsumerProbeArtifact]>,
}

impl NativeCompatibilityEvidence {
    pub fn from_canonical_reports(
        binding: NativeCompatibilityBinding,
        parity_bytes: &[u8],
        consumer_reports: &[&[u8]],
    ) -> NativeCompatibilityResult<Self> {
        binding.validate()?;
        if consumer_reports.is_empty() || consumer_reports.len() > MAX_CONSUMERS {
            return Err(failure(
                NativeCompatibilityErrorCode::InputLimit,
                "native compatibility consumer report count is outside the reviewed bounds",
            ));
        }
        let parity_report = NativeSemanticParityReport::from_canonical_slice(parity_bytes)?;
        if !parity_report.matches_binding(&binding) {
            return Err(failure(
                NativeCompatibilityErrorCode::IdentityMismatch,
                "native parity report belongs to another annotation artifact",
            ));
        }
        let mut consumers = Vec::with_capacity(consumer_reports.len());
        for bytes in consumer_reports {
            let result = NativeConsumerProbeResult::from_canonical_slice(bytes)?;
            if !result.matches_binding(&binding) {
                return Err(failure(
                    NativeCompatibilityErrorCode::IdentityMismatch,
                    "native consumer probe belongs to another annotation artifact",
                ));
            }
            consumers.push(NativeConsumerProbeArtifact {
                result,
                bytes: bytes.to_vec().into_boxed_slice(),
            });
        }
        consumers.sort_by_key(|artifact| artifact.result.consumer_kind);
        if consumers
            .windows(2)
            .any(|pair| pair[0].result.consumer_kind == pair[1].result.consumer_kind)
        {
            return Err(failure(
                NativeCompatibilityErrorCode::InvalidConsumerProbe,
                "native consumer probe kinds must be unique",
            ));
        }
        let evidence_id =
            compatibility_evidence_id(&binding, parity_report.report_id(), &consumers)?;
        Ok(Self {
            binding,
            evidence_id,
            parity_report,
            parity_bytes: parity_bytes.to_vec().into_boxed_slice(),
            consumers: consumers.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn status(&self) -> NativeCompatibilityStatus {
        let parity = self.parity_report.status();
        if parity == NativeCompatibilityStatus::Failed
            || self
                .consumers
                .iter()
                .any(|artifact| artifact.result.status() == NativeCompatibilityStatus::Failed)
        {
            return NativeCompatibilityStatus::Failed;
        }
        let emmy = self
            .consumers
            .iter()
            .find(|artifact| artifact.result.consumer_kind == NativeConsumerKind::EmmyLua);
        let luals = self
            .consumers
            .iter()
            .find(|artifact| artifact.result.consumer_kind == NativeConsumerKind::LuaLs);
        if parity != NativeCompatibilityStatus::Passed
            || emmy.is_none_or(|artifact| {
                artifact.result.status() != NativeCompatibilityStatus::Passed
            })
            || luals.is_none_or(|artifact| {
                artifact.result.status() != NativeCompatibilityStatus::Passed
            })
            || self
                .consumers
                .iter()
                .any(|artifact| artifact.result.status() != NativeCompatibilityStatus::Passed)
        {
            return NativeCompatibilityStatus::NotEvaluated;
        }
        NativeCompatibilityStatus::Passed
    }

    #[must_use]
    pub fn binding(&self) -> &NativeCompatibilityBinding {
        &self.binding
    }

    #[must_use]
    pub fn evidence_id(&self) -> &str {
        &self.evidence_id
    }

    #[must_use]
    pub fn parity_report(&self) -> &NativeSemanticParityReport {
        &self.parity_report
    }

    #[must_use]
    pub fn parity_bytes(&self) -> &[u8] {
        &self.parity_bytes
    }

    #[must_use]
    pub fn consumers(&self) -> &[NativeConsumerProbeArtifact] {
        &self.consumers
    }
}

fn required_assertion_kinds_present(assertions: &[NativeConsumerAssertionRecord]) -> bool {
    [
        NativeConsumerAssertionKind::Positive,
        NativeConsumerAssertionKind::Negative,
        NativeConsumerAssertionKind::SemanticType,
        NativeConsumerAssertionKind::SourceSpan,
        NativeConsumerAssertionKind::DiagnosticBaseline,
        NativeConsumerAssertionKind::ConfigurationMutation,
    ]
    .into_iter()
    .all(|required| {
        assertions
            .iter()
            .any(|assertion| assertion.mandatory && assertion.kind == required)
    })
}

fn parity_record_id(record: &NativeParityRecord) -> NativeCompatibilityResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        subject_id: &'a str,
        our_semantic_id: &'a str,
        oracle_semantic_id: &'a str,
        comparison_rule_id: &'a str,
        difference_sha256: &'a str,
        classification: NativeParityClassification,
        mandatory: bool,
    }
    content_id(
        "native-parity-record",
        &Identity {
            schema: "wow-annotations/native-parity-record/1",
            subject_id: &record.subject_id,
            our_semantic_id: &record.our_semantic_id,
            oracle_semantic_id: &record.oracle_semantic_id,
            comparison_rule_id: &record.comparison_rule_id,
            difference_sha256: &record.difference_sha256,
            classification: record.classification,
            mandatory: record.mandatory,
        },
    )
}

fn parity_report_id(report: &NativeSemanticParityReport) -> NativeCompatibilityResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        annotation_artifact_id: &'a str,
        source_map_id: &'a str,
        loss_report_id: &'a str,
        profile_id: &'a str,
        source_generation_id: &'a str,
        oracle_kind: &'a str,
        oracle_revision: &'a str,
        oracle_artifact_sha256: &'a str,
        oracle_profile_id: &'a str,
        comparison_profile_id: &'a str,
        source_equivalence: NativeCompatibilityStatus,
        records: &'a [NativeParityRecord],
    }
    content_id(
        "native-semantic-parity",
        &Identity {
            schema: NATIVE_PARITY_REPORT_SCHEMA,
            annotation_artifact_id: &report.annotation_artifact_id,
            source_map_id: &report.source_map_id,
            loss_report_id: &report.loss_report_id,
            profile_id: &report.profile_id,
            source_generation_id: &report.source_generation_id,
            oracle_kind: &report.oracle_kind,
            oracle_revision: &report.oracle_revision,
            oracle_artifact_sha256: &report.oracle_artifact_sha256,
            oracle_profile_id: &report.oracle_profile_id,
            comparison_profile_id: &report.comparison_profile_id,
            source_equivalence: report.source_equivalence,
            records: &report.records,
        },
    )
}

fn consumer_assertion_id(
    assertion: &NativeConsumerAssertionRecord,
) -> NativeCompatibilityResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        kind: NativeConsumerAssertionKind,
        subject_id: &'a str,
        status: NativeCompatibilityStatus,
        evidence_sha256: &'a str,
        mandatory: bool,
    }
    content_id(
        "native-consumer-assertion",
        &Identity {
            schema: "wow-annotations/native-consumer-assertion/1",
            kind: assertion.kind,
            subject_id: &assertion.subject_id,
            status: assertion.status,
            evidence_sha256: &assertion.evidence_sha256,
            mandatory: assertion.mandatory,
        },
    )
}

fn consumer_result_id(result: &NativeConsumerProbeResult) -> NativeCompatibilityResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        annotation_artifact_id: &'a str,
        source_map_id: &'a str,
        loss_report_id: &'a str,
        profile_id: &'a str,
        source_generation_id: &'a str,
        consumer_kind: NativeConsumerKind,
        consumer_version: &'a str,
        consumer_revision: &'a str,
        executable_sha256: &'a str,
        capability_profile_id: &'a str,
        probe_manifest_sha256: &'a str,
        raw_output_sha256: &'a str,
        load_status: NativeCompatibilityStatus,
        index_status: NativeCompatibilityStatus,
        forbidden_mutation_count: u64,
        suppressed_diagnostic_count: u64,
        assertions: &'a [NativeConsumerAssertionRecord],
    }
    content_id(
        "native-consumer-probe",
        &Identity {
            schema: NATIVE_CONSUMER_PROBE_SCHEMA,
            annotation_artifact_id: &result.annotation_artifact_id,
            source_map_id: &result.source_map_id,
            loss_report_id: &result.loss_report_id,
            profile_id: &result.profile_id,
            source_generation_id: &result.source_generation_id,
            consumer_kind: result.consumer_kind,
            consumer_version: &result.consumer_version,
            consumer_revision: &result.consumer_revision,
            executable_sha256: &result.executable_sha256,
            capability_profile_id: &result.capability_profile_id,
            probe_manifest_sha256: &result.probe_manifest_sha256,
            raw_output_sha256: &result.raw_output_sha256,
            load_status: result.load_status,
            index_status: result.index_status,
            forbidden_mutation_count: result.forbidden_mutation_count,
            suppressed_diagnostic_count: result.suppressed_diagnostic_count,
            assertions: &result.assertions,
        },
    )
}

fn compatibility_evidence_id(
    binding: &NativeCompatibilityBinding,
    parity_report_id: &str,
    consumers: &[NativeConsumerProbeArtifact],
) -> NativeCompatibilityResult<Box<str>> {
    #[derive(Serialize)]
    struct Consumer<'a> {
        kind: NativeConsumerKind,
        result_id: &'a str,
    }
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        binding: &'a NativeCompatibilityBinding,
        parity_report_id: &'a str,
        consumers: Vec<Consumer<'a>>,
    }
    content_id(
        "native-compatibility-evidence",
        &Identity {
            schema: NATIVE_COMPATIBILITY_EVIDENCE_SCHEMA,
            binding,
            parity_report_id,
            consumers: consumers
                .iter()
                .map(|artifact| Consumer {
                    kind: artifact.result.consumer_kind,
                    result_id: &artifact.result.result_id,
                })
                .collect(),
        },
    )
}

fn content_id(prefix: &str, value: &impl Serialize) -> NativeCompatibilityResult<Box<str>> {
    let bytes = canonical_json_bytes(value).map_err(|_| {
        failure(
            NativeCompatibilityErrorCode::SerializationFailed,
            "native compatibility identity cannot be canonicalized",
        )
    })?;
    Ok(format!("{prefix}:sha256:{}", hex(&Sha256::digest(bytes))).into_boxed_str())
}

fn canonical_bounded(
    value: &impl Serialize,
    code: NativeCompatibilityErrorCode,
    message: &'static str,
) -> NativeCompatibilityResult<Box<[u8]>> {
    let bytes = canonical_json_bytes(value).map_err(|_| failure(code, message))?;
    bounded(&bytes, code)?;
    Ok(bytes.into_boxed_slice())
}

fn bounded(bytes: &[u8], code: NativeCompatibilityErrorCode) -> NativeCompatibilityResult<()> {
    if bytes.is_empty() || bytes.len() > MAX_COMPATIBILITY_REPORT_BYTES {
        return Err(failure(
            code,
            "native compatibility report is empty or exceeds the reviewed byte budget",
        ));
    }
    Ok(())
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'\\' && byte != b'"')
}

fn exact_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn canonical_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn strictly_sorted_unique<'a>(values: impl Iterator<Item = &'a str>) -> bool {
    let values = values.collect::<Vec<_>>();
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn failure(
    code: NativeCompatibilityErrorCode,
    message: impl Into<Box<str>>,
) -> NativeCompatibilityError {
    NativeCompatibilityError::new(code, message)
}

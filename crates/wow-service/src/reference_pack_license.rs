//! Canonical license, notice and redistribution decisions for one native Reference Pack.
//!
//! Legal review remains external. This module validates a bounded canonical manifest,
//! exact component bindings, notice bytes and explicit redistribution decisions. It does
//! not infer rights from source names, repository visibility, SPDX-looking prose or tool output.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

pub(crate) const NATIVE_DISTRIBUTION_MANIFEST_SCHEMA: &str =
    "wow-service/reference-pack-distribution/1";
pub(crate) const PACK_METADATA_PROFILE_ID: &str =
    "wow-reference-pack/metadata-class/local-native/1";
pub(crate) const MAX_DISTRIBUTION_MANIFEST_BYTES: usize = 8 * 1024 * 1024;
const MAX_DECISIONS: usize = 8;
const MAX_NOTICES: usize = 64;
const MAX_NOTICE_BYTES: usize = 256 * 1024;
const MAX_NOTICE_TOTAL_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NativeDistributionErrorCode {
    InvalidBinding,
    InvalidManifest,
    InvalidDecision,
    InvalidNotice,
    IdentityMismatch,
    InputLimit,
    SerializationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeDistributionError {
    code: NativeDistributionErrorCode,
    message: Box<str>,
}

impl NativeDistributionError {
    fn new(code: NativeDistributionErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub(crate) const fn code(&self) -> NativeDistributionErrorCode {
        self.code
    }
}

impl fmt::Display for NativeDistributionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for NativeDistributionError {}

pub(crate) type NativeDistributionResult<T> = Result<T, NativeDistributionError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NativeDistributionStatus {
    Passed,
    Failed,
    NotEvaluated,
}

impl NativeDistributionStatus {
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::NotEvaluated => "not_evaluated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NativeRedistributionClass {
    Permitted,
    Prohibited,
    NotEvaluated,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NativeDistributionSubject {
    SourceSnapshot,
    ReferenceData,
    AnnotationArtifact,
    CompatibilityEvidence,
    PackMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeDistributionBinding {
    profile_id: Box<str>,
    source_generation_id: Box<str>,
    source_manifest_sha256: Box<str>,
    reference_view_digest: Box<str>,
    annotation_artifact_id: Box<str>,
    source_map_id: Box<str>,
    loss_report_id: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compatibility_evidence_id: Option<Box<str>>,
    pack_metadata_profile_id: Box<str>,
}

impl NativeDistributionBinding {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        profile_id: impl Into<Box<str>>,
        source_generation_id: impl Into<Box<str>>,
        source_manifest_sha256: impl Into<Box<str>>,
        reference_view_digest: impl Into<Box<str>>,
        annotation_artifact_id: impl Into<Box<str>>,
        source_map_id: impl Into<Box<str>>,
        loss_report_id: impl Into<Box<str>>,
        compatibility_evidence_id: Option<&str>,
    ) -> NativeDistributionResult<Self> {
        let value = Self {
            profile_id: profile_id.into(),
            source_generation_id: source_generation_id.into(),
            source_manifest_sha256: source_manifest_sha256.into(),
            reference_view_digest: reference_view_digest.into(),
            annotation_artifact_id: annotation_artifact_id.into(),
            source_map_id: source_map_id.into(),
            loss_report_id: loss_report_id.into(),
            compatibility_evidence_id: compatibility_evidence_id.map(Into::into),
            pack_metadata_profile_id: PACK_METADATA_PROFILE_ID.into(),
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> NativeDistributionResult<()> {
        if !valid_component(&self.profile_id)
            || !valid_component(&self.source_generation_id)
            || !canonical_sha256(&self.source_manifest_sha256)
            || !valid_component(&self.reference_view_digest)
            || !valid_component(&self.annotation_artifact_id)
            || !valid_component(&self.source_map_id)
            || !valid_component(&self.loss_report_id)
            || self
                .compatibility_evidence_id
                .as_deref()
                .is_some_and(|value| !valid_component(value))
            || self.pack_metadata_profile_id.as_ref() != PACK_METADATA_PROFILE_ID
        {
            return Err(failure(
                NativeDistributionErrorCode::InvalidBinding,
                "native distribution binding contains an invalid identity",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub(crate) fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub(crate) fn source_generation_id(&self) -> &str {
        &self.source_generation_id
    }

    #[must_use]
    pub(crate) fn source_manifest_sha256(&self) -> &str {
        &self.source_manifest_sha256
    }

    #[must_use]
    pub(crate) fn reference_view_digest(&self) -> &str {
        &self.reference_view_digest
    }

    #[must_use]
    pub(crate) fn annotation_artifact_id(&self) -> &str {
        &self.annotation_artifact_id
    }

    #[must_use]
    pub(crate) fn source_map_id(&self) -> &str {
        &self.source_map_id
    }

    #[must_use]
    pub(crate) fn loss_report_id(&self) -> &str {
        &self.loss_report_id
    }

    #[must_use]
    pub(crate) fn compatibility_evidence_id(&self) -> Option<&str> {
        self.compatibility_evidence_id.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeDistributionReview {
    status: NativeDistributionStatus,
    authority: Box<str>,
    reviewer_id: Box<str>,
    review_record_sha256: Box<str>,
    private_location_audit: NativeDistributionStatus,
    scope_complete: bool,
}

impl NativeDistributionReview {
    fn validate(&self) -> NativeDistributionResult<()> {
        if self.authority.as_ref() != "external_explicit_review"
            || !valid_component(&self.reviewer_id)
            || !canonical_sha256(&self.review_record_sha256)
        {
            return Err(failure(
                NativeDistributionErrorCode::InvalidManifest,
                "native distribution review identity is invalid",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeDistributionDecision {
    decision_id: Box<str>,
    subject: NativeDistributionSubject,
    content_identity: Box<str>,
    embedded: bool,
    redistribution: NativeRedistributionClass,
    license_expression: Box<str>,
    provenance_id: Box<str>,
    required_notice_ids: Box<[Box<str>]>,
}

impl NativeDistributionDecision {
    fn validate(&self) -> NativeDistributionResult<()> {
        if !valid_component(&self.content_identity)
            || !valid_license_expression(&self.license_expression)
            || !valid_component(&self.provenance_id)
            || self.required_notice_ids.len() > MAX_NOTICES
            || !strictly_sorted_unique(self.required_notice_ids.iter().map(AsRef::as_ref))
            || self
                .required_notice_ids
                .iter()
                .any(|id| !valid_component(id))
            || self.decision_id != distribution_decision_id(self)?
        {
            return Err(failure(
                NativeDistributionErrorCode::InvalidDecision,
                "native distribution decision is invalid or noncanonical",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub(crate) const fn embedded(&self) -> bool {
        self.embedded
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeDistributionNotice {
    notice_id: Box<str>,
    path: Box<str>,
    sha256: Box<str>,
    text: Box<str>,
}

impl NativeDistributionNotice {
    fn validate(&self) -> NativeDistributionResult<()> {
        if !valid_notice_path(&self.path)
            || self.text.is_empty()
            || self.text.len() > MAX_NOTICE_BYTES
            || self
                .text
                .chars()
                .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
            || sha256(self.text.as_bytes()) != self.sha256.as_ref()
            || self.notice_id != distribution_notice_id(self)?
        {
            return Err(failure(
                NativeDistributionErrorCode::InvalidNotice,
                "native distribution notice is invalid or does not match its identity",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub(crate) fn notice_id(&self) -> &str {
        &self.notice_id
    }

    #[must_use]
    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub(crate) fn text_bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeDistributionManifest {
    schema: Box<str>,
    manifest_id: Box<str>,
    binding: NativeDistributionBinding,
    review: NativeDistributionReview,
    decisions: Box<[NativeDistributionDecision]>,
    notices: Box<[NativeDistributionNotice]>,
}

impl NativeDistributionManifest {
    pub(crate) fn from_canonical_slice(
        expected_binding: NativeDistributionBinding,
        bytes: &[u8],
    ) -> NativeDistributionResult<Self> {
        if bytes.is_empty() || bytes.len() > MAX_DISTRIBUTION_MANIFEST_BYTES {
            return Err(failure(
                NativeDistributionErrorCode::InputLimit,
                "native distribution manifest is empty or exceeds the reviewed byte budget",
            ));
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| {
            failure(
                NativeDistributionErrorCode::InvalidManifest,
                "native distribution manifest failed strict decoding",
            )
        })?;
        value.validate()?;
        if value.binding != expected_binding {
            return Err(failure(
                NativeDistributionErrorCode::IdentityMismatch,
                "native distribution manifest belongs to another exact component set",
            ));
        }
        if value.canonical_bytes()?.as_ref() != bytes {
            return Err(failure(
                NativeDistributionErrorCode::InvalidManifest,
                "native distribution manifest bytes are not canonical",
            ));
        }
        Ok(value)
    }

    pub(crate) fn validate(&self) -> NativeDistributionResult<()> {
        self.binding.validate()?;
        self.review.validate()?;
        if self.schema.as_ref() != NATIVE_DISTRIBUTION_MANIFEST_SCHEMA
            || self.decisions.is_empty()
            || self.decisions.len() > MAX_DECISIONS
            || self.notices.len() > MAX_NOTICES
        {
            return Err(failure(
                NativeDistributionErrorCode::InvalidManifest,
                "native distribution manifest header or bounds are invalid",
            ));
        }
        for decision in &self.decisions {
            decision.validate()?;
        }
        for notice in &self.notices {
            notice.validate()?;
        }
        if self
            .decisions
            .windows(2)
            .any(|pair| pair[0].subject >= pair[1].subject)
            || !strictly_sorted_unique(self.notices.iter().map(|notice| notice.notice_id.as_ref()))
            || !strictly_sorted_unique(self.notices.iter().map(|notice| notice.path.as_ref()))
        {
            return Err(failure(
                NativeDistributionErrorCode::InvalidManifest,
                "native distribution inventory is not canonical",
            ));
        }
        let required = required_subjects(&self.binding);
        let observed = self
            .decisions
            .iter()
            .map(|decision| decision.subject)
            .collect::<Vec<_>>();
        if observed != required {
            return Err(failure(
                NativeDistributionErrorCode::InvalidManifest,
                "native distribution decisions do not cover the exact component set",
            ));
        }
        for decision in &self.decisions {
            let (expected_identity, expected_embedded) =
                expected_subject_identity(&self.binding, decision.subject)?;
            if decision.content_identity.as_ref() != expected_identity
                || decision.embedded != expected_embedded
            {
                return Err(failure(
                    NativeDistributionErrorCode::IdentityMismatch,
                    "native distribution decision does not match its exact component",
                ));
            }
        }
        let notices = self
            .notices
            .iter()
            .map(|notice| (notice.notice_id.as_ref(), notice))
            .collect::<BTreeMap<_, _>>();
        let referenced = self
            .decisions
            .iter()
            .flat_map(|decision| decision.required_notice_ids.iter().map(AsRef::as_ref))
            .collect::<BTreeSet<_>>();
        if referenced.len() != notices.len()
            || referenced
                .iter()
                .any(|notice_id| !notices.contains_key(notice_id))
        {
            return Err(failure(
                NativeDistributionErrorCode::InvalidManifest,
                "native distribution notice closure is incomplete or contains extras",
            ));
        }
        let total_notice_bytes = self
            .notices
            .iter()
            .try_fold(0usize, |total, notice| total.checked_add(notice.text.len()))
            .ok_or_else(|| {
                failure(
                    NativeDistributionErrorCode::InputLimit,
                    "native distribution notice byte count overflow",
                )
            })?;
        if total_notice_bytes > MAX_NOTICE_TOTAL_BYTES
            || self.manifest_id != distribution_manifest_id(self)?
        {
            return Err(failure(
                NativeDistributionErrorCode::InvalidManifest,
                "native distribution manifest identity or notice budget is invalid",
            ));
        }
        Ok(())
    }

    pub(crate) fn canonical_bytes(&self) -> NativeDistributionResult<Box<[u8]>> {
        let bytes = canonical_json_bytes(self).map_err(|_| {
            failure(
                NativeDistributionErrorCode::SerializationFailed,
                "native distribution manifest cannot be canonicalized",
            )
        })?;
        if bytes.len() > MAX_DISTRIBUTION_MANIFEST_BYTES {
            return Err(failure(
                NativeDistributionErrorCode::InputLimit,
                "native distribution manifest exceeds the reviewed byte budget",
            ));
        }
        Ok(bytes.into_boxed_slice())
    }

    #[must_use]
    pub(crate) fn status(&self) -> NativeDistributionStatus {
        if self.review.status == NativeDistributionStatus::Failed
            || self.review.private_location_audit == NativeDistributionStatus::Failed
            || self.decisions.iter().any(|decision| {
                decision.embedded
                    && decision.redistribution == NativeRedistributionClass::Prohibited
            })
        {
            return NativeDistributionStatus::Failed;
        }
        if self.review.status != NativeDistributionStatus::Passed
            || self.review.private_location_audit != NativeDistributionStatus::Passed
            || !self.review.scope_complete
            || self.decisions.iter().any(|decision| {
                if decision.embedded {
                    decision.redistribution != NativeRedistributionClass::Permitted
                } else {
                    decision.redistribution == NativeRedistributionClass::NotEvaluated
                }
            })
        {
            return NativeDistributionStatus::NotEvaluated;
        }
        NativeDistributionStatus::Passed
    }

    #[must_use]
    pub(crate) fn manifest_id(&self) -> &str {
        &self.manifest_id
    }

    #[must_use]
    pub(crate) fn binding(&self) -> &NativeDistributionBinding {
        &self.binding
    }

    #[must_use]
    pub(crate) fn notices(&self) -> &[NativeDistributionNotice] {
        &self.notices
    }

    #[must_use]
    pub(crate) fn decision(
        &self,
        subject: NativeDistributionSubject,
    ) -> Option<&NativeDistributionDecision> {
        self.decisions
            .binary_search_by_key(&subject, |decision| decision.subject)
            .ok()
            .map(|index| &self.decisions[index])
    }
}

fn required_subjects(binding: &NativeDistributionBinding) -> Vec<NativeDistributionSubject> {
    let mut subjects = vec![
        NativeDistributionSubject::SourceSnapshot,
        NativeDistributionSubject::ReferenceData,
        NativeDistributionSubject::AnnotationArtifact,
        NativeDistributionSubject::PackMetadata,
    ];
    if binding.compatibility_evidence_id.is_some() {
        subjects.push(NativeDistributionSubject::CompatibilityEvidence);
    }
    subjects.sort();
    subjects
}

fn expected_subject_identity(
    binding: &NativeDistributionBinding,
    subject: NativeDistributionSubject,
) -> NativeDistributionResult<(&str, bool)> {
    match subject {
        NativeDistributionSubject::SourceSnapshot => Ok((binding.source_manifest_sha256(), false)),
        NativeDistributionSubject::ReferenceData => Ok((binding.reference_view_digest(), true)),
        NativeDistributionSubject::AnnotationArtifact => {
            Ok((binding.annotation_artifact_id(), true))
        }
        NativeDistributionSubject::CompatibilityEvidence => binding
            .compatibility_evidence_id()
            .map(|identity| (identity, true))
            .ok_or_else(|| {
                failure(
                    NativeDistributionErrorCode::IdentityMismatch,
                    "compatibility distribution decision has no selected evidence",
                )
            }),
        NativeDistributionSubject::PackMetadata => Ok((PACK_METADATA_PROFILE_ID, true)),
    }
}

fn distribution_decision_id(
    decision: &NativeDistributionDecision,
) -> NativeDistributionResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        subject: NativeDistributionSubject,
        content_identity: &'a str,
        embedded: bool,
        redistribution: NativeRedistributionClass,
        license_expression: &'a str,
        provenance_id: &'a str,
        required_notice_ids: &'a [Box<str>],
    }
    content_id(
        "distribution-decision",
        &Identity {
            schema: "wow-service/reference-pack-distribution-decision/1",
            subject: decision.subject,
            content_identity: &decision.content_identity,
            embedded: decision.embedded,
            redistribution: decision.redistribution,
            license_expression: &decision.license_expression,
            provenance_id: &decision.provenance_id,
            required_notice_ids: &decision.required_notice_ids,
        },
    )
}

fn distribution_notice_id(notice: &NativeDistributionNotice) -> NativeDistributionResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        path: &'a str,
        sha256: &'a str,
    }
    content_id(
        "distribution-notice",
        &Identity {
            schema: "wow-service/reference-pack-distribution-notice/1",
            path: &notice.path,
            sha256: &notice.sha256,
        },
    )
}

fn distribution_manifest_id(
    manifest: &NativeDistributionManifest,
) -> NativeDistributionResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        binding: &'a NativeDistributionBinding,
        review: &'a NativeDistributionReview,
        decisions: &'a [NativeDistributionDecision],
        notices: &'a [NativeDistributionNotice],
    }
    content_id(
        "distribution-manifest",
        &Identity {
            schema: NATIVE_DISTRIBUTION_MANIFEST_SCHEMA,
            binding: &manifest.binding,
            review: &manifest.review,
            decisions: &manifest.decisions,
            notices: &manifest.notices,
        },
    )
}

fn content_id(prefix: &str, value: &impl Serialize) -> NativeDistributionResult<Box<str>> {
    let bytes = canonical_json_bytes(value).map_err(|_| {
        failure(
            NativeDistributionErrorCode::SerializationFailed,
            "native distribution identity cannot be canonicalized",
        )
    })?;
    Ok(format!("{prefix}:sha256:{}", hex(&Sha256::digest(bytes))).into_boxed_str())
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'\\' && byte != b'"')
}

fn valid_license_expression(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .chars()
            .all(|character| !character.is_control() && character != '\\')
}

fn valid_notice_path(value: &str) -> bool {
    let Some(name) = value.strip_prefix("licenses/notices/") else {
        return false;
    };
    !name.is_empty()
        && name.len() <= 128
        && !name.contains('/')
        && name.ends_with(".txt")
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
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

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(&Sha256::digest(bytes)))
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
    code: NativeDistributionErrorCode,
    message: impl Into<Box<str>>,
) -> NativeDistributionError {
    NativeDistributionError::new(code, message)
}

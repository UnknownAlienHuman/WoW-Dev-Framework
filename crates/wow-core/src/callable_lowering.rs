//! One admitted callable display-lowering observation shared by concrete owners.
//! Admission checks data shape only. It supplies no producer, consumer or runtime proof.
use serde::Serialize;

use crate::error::validation_error;
use crate::{
    CanonicalResult, ContentDigest, CoreErrorCode, CoreResult, ProfileId, ReferenceGenerationId,
    SourceContent, SourceSpan,
};

const OPERATION: &str = "admit_callable_lowering";
const MAX_FIELDS: usize = 65_536;
const MAX_TYPE_BYTES: usize = 64 * 1024;
const MAX_TYPES_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallableLoweringLane {
    Parameter,
    Return,
}

/// Immutable, validated ranges and an owner-supplied emitted type token.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CallableFieldLowering {
    lane: CallableLoweringLane,
    ordinal: usize,
    original_source_span: SourceSpan,
    generated_member_span: SourceSpan,
    type_token_span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    optional_marker_span: Option<SourceSpan>,
    lowered_type: Box<str>,
    optional: bool,
}

impl CallableFieldLowering {
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        lane: CallableLoweringLane,
        ordinal: usize,
        original_source_span: SourceSpan,
        generated_member_span: SourceSpan,
        type_token_span: SourceSpan,
        optional_marker_span: Option<SourceSpan>,
        lowered_type: Box<str>,
        optional: bool,
    ) -> CoreResult<Self> {
        range(original_source_span)?;
        let (member_start, member_end) = range(generated_member_span)?;
        let (type_start, type_end) = range(type_token_span)?;
        let marker_is_valid = match optional_marker_span {
            Some(span) => {
                let (marker_start, marker_end) = range(span)?;
                marker_end - marker_start == 1
                    && marker_start >= member_start
                    && marker_end <= member_end
                    && (marker_end <= type_start || marker_start >= type_end)
            }
            None => true,
        };
        if ordinal >= MAX_FIELDS
            || lowered_type.is_empty()
            || lowered_type.len() > MAX_TYPE_BYTES
            || lowered_type.chars().any(char::is_control)
            || type_start < member_start
            || type_end > member_end
            || type_end - type_start != lowered_type.len() as u64
            || optional != optional_marker_span.is_some()
            || !marker_is_valid
        {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::ContractViolation,
                "field",
            ));
        }
        Ok(Self {
            lane,
            ordinal,
            original_source_span,
            generated_member_span,
            type_token_span,
            optional_marker_span,
            lowered_type,
            optional,
        })
    }

    pub const fn lane(&self) -> CallableLoweringLane {
        self.lane
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn original_source_span(&self) -> SourceSpan {
        self.original_source_span
    }
    pub const fn generated_member_span(&self) -> SourceSpan {
        self.generated_member_span
    }
    pub const fn type_token_span(&self) -> SourceSpan {
        self.type_token_span
    }
    pub const fn optional_marker_span(&self) -> Option<SourceSpan> {
        self.optional_marker_span
    }
    pub fn lowered_type(&self) -> &str {
        &self.lowered_type
    }
    pub const fn optional(&self) -> bool {
        self.optional
    }
}

/// Borrowed data only. Exact Library membership/bytes must be verified by its consumer.
/// No public fields or deserialization can bypass admission. Owner IDs are never reminted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CallableLoweringObservation<'a> {
    schema: &'static str,
    profile_id: &'a str,
    reference_generation: &'a str,
    reference_view_digest: &'a str,
    record_key: &'a str,
    record_digest: &'a str,
    source_id: &'a str,
    source_path: &'a str,
    source_sha256: &'a str,
    annotation_artifact_id: &'a str,
    source_map_id: &'a str,
    generated_path: &'a str,
    generated_sha256: &'a str,
    declaration_span: SourceSpan,
    fields: &'a [CallableFieldLowering],
    lowering_syntax: &'static str,
    consumer_semantics: &'static str,
}

impl<'a> CallableLoweringObservation<'a> {
    /// Validate bounded exact borrowed owner descriptors without normalization.
    /// Existing purpose/family parsers may use bounded identity scratch; nothing is retained.
    /// Existing native source/artifact/map descriptors stay in their owner families.
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        profile_id: &'a str,
        reference_generation: &'a str,
        reference_view_digest: &'a str,
        record_key: &'a str,
        record_digest: &'a str,
        source_id: &'a str,
        source_path: &'a str,
        source_sha256: &'a str,
        annotation_artifact_id: &'a str,
        source_map_id: &'a str,
        generated_path: &'a str,
        generated_sha256: &'a str,
        declaration_span: SourceSpan,
        fields: &'a [CallableFieldLowering],
    ) -> CoreResult<Self> {
        for value in [
            profile_id,
            reference_generation,
            record_key,
            source_id,
            annotation_artifact_id,
            source_map_id,
        ] {
            if value.is_empty()
                || value.len() > 1024
                || value.split([':', '/']).any(str::is_empty)
                || !value.bytes().any(|b| b.is_ascii_alphanumeric())
                || !value.bytes().all(|b| {
                    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b':' | b'/')
                })
            {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::InvalidIdentifier,
                    "identity",
                ));
            }
        }
        let _: ProfileId = profile_id.parse()?;
        let _: ReferenceGenerationId = reference_generation.parse()?;
        for value in [reference_view_digest, record_digest] {
            if !ContentDigest::<CanonicalResult>::parse(value)?.was_canonical() {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::InvalidDigest,
                    "digest",
                ));
            }
        }
        for value in [source_sha256, generated_sha256] {
            if !ContentDigest::<SourceContent>::parse(value)?.was_canonical() {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::InvalidDigest,
                    "digest",
                ));
            }
        }
        for value in [source_path, generated_path] {
            if value.is_empty()
                || value.len() > 4096
                || value.starts_with('/')
                || value.contains(['\\', ':'])
                || value.chars().any(char::is_control)
                || value
                    .split('/')
                    .any(|part| part.is_empty() || part == "." || part == "..")
            {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::InvalidSourcePath,
                    "path",
                ));
            }
        }
        let (start, end) = range(declaration_span)?;
        if fields.len() > MAX_FIELDS {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::BudgetExceeded,
                "fields",
            ));
        }
        let mut parameters = 0;
        let mut returns = 0;
        let mut previous_end = start;
        let mut bytes = 0usize;
        for field in fields {
            let next = match field.lane {
                CallableLoweringLane::Parameter if returns == 0 => &mut parameters,
                CallableLoweringLane::Return => &mut returns,
                _ => {
                    return Err(validation_error(
                        OPERATION,
                        CoreErrorCode::ContractViolation,
                        "lane_order",
                    ));
                }
            };
            let (member_start, member_end) = range(field.generated_member_span)?;
            bytes = bytes.checked_add(field.lowered_type.len()).ok_or_else(|| {
                validation_error(OPERATION, CoreErrorCode::UsageOverflow, "types")
            })?;
            if field.ordinal != *next
                || member_start < previous_end
                || member_end > end
                || bytes > MAX_TYPES_BYTES
            {
                return Err(validation_error(
                    OPERATION,
                    CoreErrorCode::ContractViolation,
                    "field_order",
                ));
            }
            *next += 1;
            previous_end = member_end;
        }
        Ok(Self {
            schema: "wow-core/callable-lowering-observation/1",
            profile_id,
            reference_generation,
            reference_view_digest,
            record_key,
            record_digest,
            source_id,
            source_path,
            source_sha256,
            annotation_artifact_id,
            source_map_id,
            generated_path,
            generated_sha256,
            declaration_span,
            fields,
            lowering_syntax: "emitted_analysis_only",
            consumer_semantics: "NotEvaluated",
        })
    }

    pub const fn schema(&self) -> &'static str {
        self.schema
    }
    pub const fn profile_id(&self) -> &'a str {
        self.profile_id
    }
    pub const fn reference_generation(&self) -> &'a str {
        self.reference_generation
    }
    pub const fn reference_view_digest(&self) -> &'a str {
        self.reference_view_digest
    }
    pub const fn record_key(&self) -> &'a str {
        self.record_key
    }
    pub const fn record_digest(&self) -> &'a str {
        self.record_digest
    }
    pub const fn source_id(&self) -> &'a str {
        self.source_id
    }
    pub const fn source_path(&self) -> &'a str {
        self.source_path
    }
    pub const fn source_sha256(&self) -> &'a str {
        self.source_sha256
    }
    pub const fn annotation_artifact_id(&self) -> &'a str {
        self.annotation_artifact_id
    }
    pub const fn source_map_id(&self) -> &'a str {
        self.source_map_id
    }
    pub const fn generated_path(&self) -> &'a str {
        self.generated_path
    }
    pub const fn generated_sha256(&self) -> &'a str {
        self.generated_sha256
    }
    pub const fn declaration_span(&self) -> SourceSpan {
        self.declaration_span
    }
    pub const fn fields(&self) -> &'a [CallableFieldLowering] {
        self.fields
    }
    pub const fn lowering_syntax(&self) -> &'static str {
        self.lowering_syntax
    }
    pub const fn consumer_semantics(&self) -> &'static str {
        self.consumer_semantics
    }
}

fn range(span: SourceSpan) -> CoreResult<(u64, u64)> {
    span.validate()?;
    match (span.byte_start(), span.byte_end()) {
        (Some(start), Some(end)) if start < end => Ok((start, end)),
        _ => Err(validation_error(
            OPERATION,
            CoreErrorCode::InvalidSourceSpan,
            "span",
        )),
    }
}

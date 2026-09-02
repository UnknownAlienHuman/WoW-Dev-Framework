use serde::{Deserialize, Serialize};

use crate::error::{
    CoreError, CoreErrorCode, CoreResult, ErrorCategory, RetryClass, validation_error,
};
use crate::ids::{CapabilityId, MessageCode, validate_lower_segment};

const MAX_COLLECTION_LIMIT: u64 = 10_000_000;
const MAX_OUTPUT_LIMIT: u64 = 1_073_741_824;

/// Explicit upper bounds for every E0 result collection and final output bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetLimits {
    pub max_coverage_records: u64,
    pub max_capability_summaries: u64,
    pub max_source_handles: u64,
    pub max_evidence_records: u64,
    pub max_conflicts: u64,
    pub max_findings: u64,
    pub max_not_evaluated: u64,
    pub max_warnings: u64,
    pub max_output_bytes: u64,
}

impl BudgetLimits {
    /// Validates nonzero bounded limits.
    pub fn validate(&self) -> CoreResult<()> {
        let collections = [
            ("max_coverage_records", self.max_coverage_records),
            ("max_capability_summaries", self.max_capability_summaries),
            ("max_source_handles", self.max_source_handles),
            ("max_evidence_records", self.max_evidence_records),
            ("max_conflicts", self.max_conflicts),
            ("max_findings", self.max_findings),
            ("max_not_evaluated", self.max_not_evaluated),
            ("max_warnings", self.max_warnings),
        ];
        for (field, value) in collections {
            if value == 0 || value > MAX_COLLECTION_LIMIT {
                return Err(validation_error(
                    "validate_budget",
                    CoreErrorCode::BudgetInvalid,
                    field,
                ));
            }
        }
        if self.max_output_bytes == 0 || self.max_output_bytes > MAX_OUTPUT_LIMIT {
            return Err(validation_error(
                "validate_budget",
                CoreErrorCode::BudgetInvalid,
                "max_output_bytes",
            ));
        }
        Ok(())
    }
}

/// Observed result usage. Arithmetic is checked and never wraps.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetUsage {
    pub coverage_records: u64,
    pub capability_summaries: u64,
    pub source_handles: u64,
    pub evidence_records: u64,
    pub conflicts: u64,
    pub findings: u64,
    pub not_evaluated: u64,
    pub warnings: u64,
    pub output_bytes: u64,
}

impl BudgetUsage {
    /// Adds usage with checked arithmetic.
    pub fn checked_add(self, other: Self) -> CoreResult<Self> {
        macro_rules! add {
            ($field:ident) => {
                self.$field.checked_add(other.$field).ok_or_else(|| {
                    CoreError::new(
                        CoreErrorCode::UsageOverflow,
                        ErrorCategory::Invariant,
                        "accumulate_budget_usage",
                        RetryClass::Never,
                    )
                    .at_field(stringify!($field))
                })?
            };
        }
        Ok(Self {
            coverage_records: add!(coverage_records),
            capability_summaries: add!(capability_summaries),
            source_handles: add!(source_handles),
            evidence_records: add!(evidence_records),
            conflicts: add!(conflicts),
            findings: add!(findings),
            not_evaluated: add!(not_evaluated),
            warnings: add!(warnings),
            output_bytes: add!(output_bytes),
        })
    }
}

/// Explicit truncation state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum TruncationState {
    NotTruncated,
    Truncated { entries: Vec<TruncationEntry> },
}

impl TruncationState {
    /// Whether any collection is explicitly truncated.
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        matches!(self, Self::Truncated { .. })
    }

    /// Canonical entries, when truncated.
    #[must_use]
    pub fn entries(&self) -> &[TruncationEntry] {
        match self {
            Self::NotTruncated => &[],
            Self::Truncated { entries } => entries,
        }
    }
}

/// One affected collection/capability truncation record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TruncationEntry {
    collection_id: String,
    capability_ids: Vec<CapabilityId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    omitted_count: Option<u64>,
    count_unknown: bool,
    reason_code: MessageCode,
}

impl TruncationEntry {
    /// Constructs a canonical truncation entry.
    pub fn new(
        collection_id: impl Into<String>,
        mut capability_ids: Vec<CapabilityId>,
        omitted_count: Option<u64>,
        count_unknown: bool,
        reason_code: MessageCode,
    ) -> CoreResult<Self> {
        let collection_id = collection_id.into();
        validate_lower_segment(
            &collection_id,
            "classify_truncation",
            "entries.collection_id",
        )?;
        if omitted_count.is_some() == count_unknown {
            return Err(validation_error(
                "classify_truncation",
                CoreErrorCode::ContractViolation,
                "entries.omitted_count",
            ));
        }
        capability_ids.sort();
        capability_ids.dedup();
        Ok(Self {
            collection_id,
            capability_ids,
            omitted_count,
            count_unknown,
            reason_code,
        })
    }
}

/// Complete E0 budget truth record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    limits: BudgetLimits,
    usage: BudgetUsage,
    truncation: TruncationState,
}

impl Budget {
    /// Constructs and validates a budget record.
    pub fn new(
        limits: BudgetLimits,
        usage: BudgetUsage,
        mut truncation: TruncationState,
    ) -> CoreResult<Self> {
        limits.validate()?;
        if let TruncationState::Truncated { entries } = &mut truncation {
            if entries.is_empty() {
                return Err(validation_error(
                    "classify_truncation",
                    CoreErrorCode::ContractViolation,
                    "truncation.entries",
                ));
            }
            entries.sort();
            for pair in entries.windows(2) {
                if pair[0].collection_id == pair[1].collection_id {
                    return Err(validation_error(
                        "classify_truncation",
                        CoreErrorCode::ContractViolation,
                        "truncation.entries",
                    )
                    .with_argument("reason", "duplicate_collection"));
                }
            }
        }
        let budget = Self {
            limits,
            usage,
            truncation,
        };
        budget.validate_limits()?;
        Ok(budget)
    }

    /// Limits.
    #[must_use]
    pub const fn limits(&self) -> BudgetLimits {
        self.limits
    }

    /// Usage.
    #[must_use]
    pub const fn usage(&self) -> BudgetUsage {
        self.usage
    }

    /// Truncation truth.
    #[must_use]
    pub const fn truncation(&self) -> &TruncationState {
        &self.truncation
    }

    /// Replaces output-byte usage after canonical serialization.
    pub fn with_output_bytes(mut self, output_bytes: u64) -> CoreResult<Self> {
        self.usage.output_bytes = output_bytes;
        self.validate_limits()?;
        Ok(self)
    }

    /// Validates observed usage against explicit limits.
    pub fn validate_limits(&self) -> CoreResult<()> {
        self.limits.validate()?;
        let pairs = [
            (self.usage.coverage_records, self.limits.max_coverage_records),
            (
                self.usage.capability_summaries,
                self.limits.max_capability_summaries,
            ),
            (self.usage.source_handles, self.limits.max_source_handles),
            (
                self.usage.evidence_records,
                self.limits.max_evidence_records,
            ),
            (self.usage.conflicts, self.limits.max_conflicts),
            (self.usage.findings, self.limits.max_findings),
            (
                self.usage.not_evaluated,
                self.limits.max_not_evaluated,
            ),
            (self.usage.warnings, self.limits.max_warnings),
            (self.usage.output_bytes, self.limits.max_output_bytes),
        ];
        if pairs.iter().any(|(usage, limit)| usage > limit) {
            return Err(CoreError::new(
                CoreErrorCode::BudgetExceeded,
                ErrorCategory::Budget,
                "validate_budget",
                RetryClass::AfterInputChange,
            ));
        }
        Ok(())
    }
}

/// Classifies explicit omission information into a canonical truncation state.
pub fn classify_truncation(mut entries: Vec<TruncationEntry>) -> CoreResult<TruncationState> {
    if entries.is_empty() {
        return Ok(TruncationState::NotTruncated);
    }
    entries.sort();
    for pair in entries.windows(2) {
        if pair[0].collection_id == pair[1].collection_id {
            return Err(validation_error(
                "classify_truncation",
                CoreErrorCode::ContractViolation,
                "entries",
            ));
        }
    }
    Ok(TruncationState::Truncated { entries })
}

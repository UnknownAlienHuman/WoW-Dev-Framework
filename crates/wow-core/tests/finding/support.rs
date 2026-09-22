use serde::de::DeserializeOwned;
use serde_json::Value;
use std::error::Error;
use wow_core::{
    CanonicalResult, ClaimScope, ContentDigest, CoreErrorCode, CoreResult, E0CheckResultDraft,
    E0CheckResultEnvelope, EvidenceConfidence, EvidenceId, EvidenceRecord, Finding, FindingDraft,
    GenerationContext, GenerationContextId, ProvenanceClass, Remediation, SourceHandle, WarningId,
    WarningRecord, canonical_json_bytes, domain_separated_digest,
};

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub const FINDINGS: &str = include_str!("../../examples/e0-findings-result.json");

pub fn field<T: DeserializeOwned>(value: &Value, name: &str) -> TestResult<T> {
    Ok(serde_json::from_value(
        value.get(name).ok_or("missing fixture field")?.clone(),
    )?)
}

pub struct Fixture {
    pub context: GenerationContextId,
    pub sources: Vec<SourceHandle>,
    pub evidence: Vec<EvidenceRecord>,
    pub finding: Finding,
    pub value: Value,
}

impl Fixture {
    pub fn new() -> TestResult<Self> {
        let value: Value = serde_json::from_str(FINDINGS)?;
        Ok(Self {
            context: field::<GenerationContext>(&value, "context")?.context_id(),
            sources: field(&value, "source_handles")?,
            evidence: field(&value, "evidence_records")?,
            finding: serde_json::from_value(value["findings"][1].clone())?,
            value,
        })
    }

    pub fn draft(&self) -> TestResult<FindingDraft> {
        draft(&self.value["findings"][1])
    }

    pub fn warning(&self) -> TestResult<WarningRecord> {
        Ok(WarningRecord::new(
            self.context,
            "fixture.diagnostics".parse()?,
            "0.1.0".parse()?,
            "fixture.optional_unavailable".parse()?,
            Some(("lane".to_owned(), "optional".to_owned())),
            Some(self.finding.primary_source_handle_id()),
            self.sources.iter().map(SourceHandle::handle_id).collect(),
            self.evidence
                .iter()
                .map(EvidenceRecord::evidence_id)
                .collect(),
            Vec::new(),
        )?)
    }

    pub fn check(&self, value: Value) -> TestResult<CoreResult<()>> {
        let finding: Finding = serde_json::from_value(value)?;
        Ok(finding.validate(self.context, &self.sources, &self.evidence))
    }
}

pub fn draft(value: &Value) -> TestResult<FindingDraft> {
    let mut result = FindingDraft::new(
        field(value, "context_id")?,
        field(value, "rule_id")?,
        field(value, "rule_version")?,
        field(value, "finding_code")?,
        field(value, "severity")?,
        field(value, "policy")?,
        field(value, "primary_source_handle_id")?,
        field(value, "coverage_status")?,
    )
    .related_source_handle_ids(field(value, "related_source_handle_ids")?)
    .evidence_ids(field(value, "evidence_ids")?)
    .required_capability_ids(field(value, "required_capability_ids")?)
    .message_arguments(field(value, "message_arguments")?)?
    .root_causes(
        serde_json::from_value(value["root_cause_key"].clone())?,
        serde_json::from_value(value["caused_by_root_cause_key"].clone())?,
    );
    if let Some(subject) = value.get("subject_entity_key") {
        result = result.subject_entity_key(serde_json::from_value(subject.clone())?);
    }
    if let Some(remediation) = value.get("remediation") {
        result = result.remediation(serde_json::from_value::<Remediation>(remediation.clone())?);
    }
    Ok(result)
}

pub fn leaf(context: GenerationContextId, candidate: bool) -> TestResult<EvidenceRecord> {
    Ok(EvidenceRecord::new(
        context,
        if candidate {
            ProvenanceClass::SemanticCandidate
        } else {
            ProvenanceClass::ProjectSource
        },
        if candidate {
            EvidenceConfidence::Candidate
        } else {
            EvidenceConfidence::Proven
        },
        if candidate {
            ClaimScope::CandidateRelation
        } else {
            ClaimScope::ProjectFact
        },
        "fixture.diagnostic_leaf".parse()?,
        "0.1.0".parse()?,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )?)
}

pub fn other_context() -> TestResult<GenerationContextId> {
    Ok(format!("context:sha256:{}", "ef".repeat(32)).parse()?)
}

pub fn assert_error<T>(result: CoreResult<T>, code: CoreErrorCode, field: &str) -> TestResult {
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err(format!("expected {code:?} at {field}").into()),
    };
    assert_eq!(error.code(), code);
    assert_eq!(error.field_path(), Some(field));
    error.validate()?;
    Ok(())
}

pub fn reseal_warning(value: &mut Value) -> TestResult<WarningRecord> {
    let mut identity = value.clone();
    identity
        .as_object_mut()
        .ok_or("warning object")?
        .remove("warning_id");
    value["warning_id"] = WarningId::derive(&identity)?.to_string().into();
    Ok(serde_json::from_value(value.clone())?)
}

pub fn reseal_evidence(value: &mut Value) -> TestResult<EvidenceRecord> {
    let mut identity = value.clone();
    identity
        .as_object_mut()
        .ok_or("evidence object")?
        .remove("evidence_id");
    value["evidence_id"] = EvidenceId::derive(&identity)?.to_string().into();
    Ok(serde_json::from_value(value.clone())?)
}

pub fn envelope_draft(value: &Value) -> TestResult<E0CheckResultDraft> {
    Ok(E0CheckResultDraft::new(
        field(value, "schema")?,
        field(value, "operation_id")?,
        field(value, "context")?,
        field(value, "status")?,
        field(value, "coverage_records")?,
        field(value, "capability_summaries")?,
        field(value, "source_handles")?,
        field(value, "evidence_records")?,
        field(value, "conflicts")?,
        field(value, "findings")?,
        field(value, "not_evaluated")?,
        field(value, "warnings")?,
        field(&value["budget"], "limits")?,
        field(&value["budget"], "truncation")?,
    ))
}

// Reseal wire mutations without a validating finalizer. Each rejection must
// come from the target invariant, not a stale digest or output-byte count.
pub fn reseal_envelope(value: &mut Value) -> TestResult<E0CheckResultEnvelope> {
    for _ in 0..6 {
        let mut material = value.clone();
        material
            .as_object_mut()
            .ok_or("envelope object")?
            .remove("canonical_digest");
        value["canonical_digest"] = ContentDigest::<CanonicalResult>::from_bytes(
            domain_separated_digest("wow-core/result/e0-1", &material)?,
        )
        .to_string()
        .into();
        let bytes = u64::try_from(canonical_json_bytes(value)?.len())?;
        if value["budget"]["usage"]["output_bytes"].as_u64() == Some(bytes) {
            return Ok(serde_json::from_value(value.clone())?);
        }
        value["budget"]["usage"]["output_bytes"] = bytes.into();
    }
    Err("test resealing did not converge".into())
}

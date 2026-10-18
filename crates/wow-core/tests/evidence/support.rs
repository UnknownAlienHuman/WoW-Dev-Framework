use std::error::Error;

use serde_json::Value;
use wow_core::{
    ClaimScope, ConflictId, CoreErrorCode, CoreResult, EvidenceConfidence, EvidenceId,
    EvidenceRecord, GenerationContext, GenerationContextId, ProvenanceClass,
};

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub fn context() -> TestResult<GenerationContext> {
    let fixture: Value = serde_json::from_str(include_str!("../../examples/e0-clean-result.json"))?;
    Ok(serde_json::from_value(fixture["context"].clone())?)
}

pub fn record(
    label: &str,
    confidence: EvidenceConfidence,
    provenance: ProvenanceClass,
    scope: ClaimScope,
    inputs: Vec<EvidenceId>,
) -> TestResult<EvidenceRecord> {
    record_in(
        context()?.context_id(),
        label,
        confidence,
        provenance,
        scope,
        inputs,
    )
}

pub fn record_in(
    context: GenerationContextId,
    label: &str,
    confidence: EvidenceConfidence,
    provenance: ProvenanceClass,
    scope: ClaimScope,
    inputs: Vec<EvidenceId>,
) -> TestResult<EvidenceRecord> {
    // Synthetic structural records, not assertions about actual platform/runtime facts.
    Ok(EvidenceRecord::new(
        context,
        provenance,
        confidence,
        scope,
        format!("fixture.evidence.{label}").parse()?,
        "0.1.0".parse()?,
        Vec::new(),
        Vec::new(),
        inputs,
    )?)
}

pub fn leaf(label: &str) -> TestResult<EvidenceRecord> {
    record(
        label,
        EvidenceConfidence::Proven,
        ProvenanceClass::ProjectSource,
        ClaimScope::ProjectFact,
        Vec::new(),
    )
}

pub fn derived(label: &str, inputs: &[&EvidenceRecord]) -> TestResult<EvidenceRecord> {
    record(
        label,
        EvidenceConfidence::Derived,
        ProvenanceClass::ProjectSource,
        ClaimScope::ProjectFact,
        inputs.iter().map(|input| input.evidence_id()).collect(),
    )
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

pub fn reseal_evidence(value: &mut Value) -> TestResult<EvidenceRecord> {
    let mut identity = value.clone();
    identity
        .as_object_mut()
        .ok_or("evidence object")?
        .remove("evidence_id");
    value["evidence_id"] = EvidenceId::derive(&identity)?.to_string().into();
    Ok(serde_json::from_value(value.clone())?)
}

pub fn reseal_conflict(value: &mut Value) -> TestResult<wow_core::ConflictRecord> {
    let mut identity = value.clone();
    identity
        .as_object_mut()
        .ok_or("conflict object")?
        .remove("conflict_id");
    value["conflict_id"] = ConflictId::derive(&identity)?.to_string().into();
    Ok(serde_json::from_value(value.clone())?)
}

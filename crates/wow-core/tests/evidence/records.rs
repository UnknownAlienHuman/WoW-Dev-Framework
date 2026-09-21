use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use wow_core::{
    ClaimScope, CoreErrorCode, CoreResult, EvidenceConfidence as Confidence, EvidenceCoverageRef,
    EvidenceId, EvidenceRecord, ProvenanceClass as Provenance, StableHandleId,
    canonical_json_bytes, derive_evidence_id, validate_evidence_record,
};

use super::support::*;

fn field<T: DeserializeOwned>(value: &Value, name: &str) -> TestResult<T> {
    Ok(serde_json::from_value(
        value.get(name).ok_or("missing evidence field")?.clone(),
    )?)
}

fn rebuild(value: &Value) -> TestResult<CoreResult<EvidenceRecord>> {
    Ok(EvidenceRecord::new(
        field(value, "context_id")?,
        field(value, "provenance")?,
        field(value, "confidence")?,
        field(value, "claim_scope")?,
        field(value, "producer_id")?,
        field(value, "producer_version")?,
        field(value, "source_handle_ids")?,
        field(value, "coverage_refs")?,
        field(value, "derivation_input_ids")?,
    ))
}

fn with_references() -> TestResult<EvidenceRecord> {
    Ok(EvidenceRecord::new(
        context()?.context_id(),
        Provenance::ProjectSource,
        Confidence::Derived,
        ClaimScope::ProjectFact,
        "fixture.evidence".parse()?,
        "0.1.0".parse()?,
        vec![StableHandleId::derive(&"a")?, StableHandleId::derive(&"b")?],
        vec![
            EvidenceCoverageRef::new(
                "fixture.read".parse()?,
                "partition:fixture:a".parse()?,
                "fixture.source".parse()?,
            ),
            EvidenceCoverageRef::new(
                "fixture.read".parse()?,
                "partition:fixture:b".parse()?,
                "fixture.source".parse()?,
            ),
        ],
        vec![EvidenceId::derive(&"a")?, EvidenceId::derive(&"b")?],
    )?)
}

#[test]
fn evidence_001_002_023_preserve_all_committed_evidence_bytes_and_ids() -> TestResult {
    for source in [
        include_str!("../../examples/e0-clean-result.json"),
        include_str!("../../examples/e0-findings-result.json"),
        include_str!("../../examples/e0-not-evaluated-result.json"),
        include_str!("../../examples/e0-conflict-not-evaluated-result.json"),
    ] {
        let fixture: Value = serde_json::from_str(source)?;
        for wire in fixture["evidence_records"]
            .as_array()
            .ok_or("evidence array")?
        {
            let evidence: EvidenceRecord = serde_json::from_value(wire.clone())?;
            evidence.validate()?;
            validate_evidence_record(&evidence)?;
            assert_eq!(derive_evidence_id(&evidence)?, evidence.evidence_id());
            assert_eq!(
                canonical_json_bytes(&evidence)?,
                canonical_json_bytes(wire)?
            );
            assert_eq!(rebuild(wire)??, evidence);
        }
    }
    Ok(())
}

#[test]
fn evidence_003_004_005_006_candidate_provenance_cannot_claim_stronger_confidence() -> TestResult {
    for provenance in [Provenance::SemanticCandidate, Provenance::ModelInference] {
        for confidence in [
            Confidence::Proven,
            Confidence::Derived,
            Confidence::Possible,
            Confidence::Candidate,
        ] {
            let mut wire = serde_json::to_value(leaf("shape")?)?;
            wire["provenance"] = serde_json::to_value(provenance)?;
            wire["confidence"] = serde_json::to_value(confidence)?;
            let decoded = reseal_evidence(&mut wire)?;
            if confidence == Confidence::Candidate {
                decoded.validate()?;
                assert_eq!(rebuild(&wire)??, decoded);
            } else {
                assert_error(
                    decoded.validate(),
                    CoreErrorCode::EvidenceAuthorityViolation,
                    "confidence",
                )?;
                assert_error(
                    rebuild(&wire)?,
                    CoreErrorCode::EvidenceAuthorityViolation,
                    "confidence",
                )?;
            }
        }
    }
    let mut wire = serde_json::to_value(leaf("shape")?)?;
    wire["confidence"] = "derived".into();
    assert_error(
        reseal_evidence(&mut wire)?.validate(),
        CoreErrorCode::DerivedEvidenceMissingInputs,
        "derivation_input_ids",
    )?;
    assert_error(
        rebuild(&wire)?,
        CoreErrorCode::DerivedEvidenceMissingInputs,
        "derivation_input_ids",
    )
}

#[test]
fn evidence_009_013_021_resealed_reference_arrays_require_canonical_unique_entries() -> TestResult {
    let base = serde_json::to_value(with_references()?)?;
    for name in ["coverage_refs", "source_handle_ids", "derivation_input_ids"] {
        let expected = if name == "coverage_refs" {
            CoreErrorCode::DuplicateCoverageRecord
        } else {
            CoreErrorCode::DuplicateEvidenceReference
        };
        let mut duplicate = base.clone();
        let entries = duplicate[name].as_array_mut().ok_or("references")?;
        entries.insert(0, entries[0].clone());
        let decoded = reseal_evidence(&mut duplicate)?;
        assert_error(decoded.validate(), expected, name)?;
        assert_error(rebuild(&duplicate)?, expected, name)?;
        let mut reversed = base.clone();
        reversed[name].as_array_mut().ok_or("references")?.reverse();
        assert_error(reseal_evidence(&mut reversed)?.validate(), expected, name)?;
        // Deliberate constructor normalization is allowed, decoded repair is not.
        assert_eq!(rebuild(&reversed)??, rebuild(&base)??);
    }
    Ok(())
}

#[test]
fn evidence_010_rejects_unknown_fields_and_duplicate_json_members() -> TestResult {
    let base = serde_json::to_value(leaf("shape")?)?;
    for name in [
        "note",
        "explanation",
        "source_excerpt",
        "conflict_ids",
        "timestamp",
    ] {
        let mut wire = base.clone();
        wire[name] = json!("not part of the evidence contract");
        assert!(
            serde_json::from_value::<EvidenceRecord>(wire).is_err(),
            "{name}"
        );
    }
    let text = serde_json::to_string(&base)?;
    let duplicate = text.replacen(
        "\"confidence\":",
        "\"confidence\":\"candidate\",\"confidence\":",
        1,
    );
    assert_ne!(text, duplicate);
    assert!(serde_json::from_str::<EvidenceRecord>(&duplicate).is_err());
    Ok(())
}

#[test]
fn evidence_011_012_020_identity_includes_confidence_producer_and_every_reference() -> TestResult {
    let base = serde_json::to_value(with_references()?)?;
    let original: EvidenceRecord = serde_json::from_value(base.clone())?;
    for (name, replacement) in [
        ("confidence", json!("possible")),
        ("producer_version", json!("0.2.0")),
        ("producer_id", json!("fixture.other")),
        ("claim_scope", json!("source_observation")),
        ("source_handle_ids", json!([])),
        ("coverage_refs", json!([])),
    ] {
        let mut wire = base.clone();
        wire[name] = replacement;
        let changed = reseal_evidence(&mut wire)?;
        changed.validate()?;
        assert_ne!(changed.evidence_id(), original.evidence_id(), "{name}");
    }
    let mut wire = base;
    wire["derivation_input_ids"]
        .as_array_mut()
        .ok_or("inputs")?
        .pop();
    let changed = reseal_evidence(&mut wire)?;
    changed.validate()?;
    assert_ne!(changed.evidence_id(), original.evidence_id());
    Ok(())
}

#[test]
fn evidence_014_proven_runtime_record_stays_scenario_scoped() -> TestResult {
    let base = record(
        "runtime",
        Confidence::Proven,
        Provenance::RuntimeProbe,
        ClaimScope::RuntimeScenario,
        Vec::new(),
    )?;
    base.validate()?;
    for scope in [
        ClaimScope::PlatformContract,
        ClaimScope::ProjectFact,
        ClaimScope::SourceObservation,
    ] {
        let mut wire = serde_json::to_value(&base)?;
        wire["claim_scope"] = serde_json::to_value(scope)?;
        assert_error(
            reseal_evidence(&mut wire)?.validate(),
            CoreErrorCode::EvidenceAuthorityViolation,
            "claim_scope",
        )?;
        assert_error(
            rebuild(&wire)?,
            CoreErrorCode::EvidenceAuthorityViolation,
            "claim_scope",
        )?;
    }
    Ok(())
}

#[test]
fn evidence_018_proven_record_cannot_hide_derivation_inputs() -> TestResult {
    let mut wire = serde_json::to_value(with_references()?)?;
    wire["confidence"] = "proven".into();
    assert_error(
        reseal_evidence(&mut wire)?.validate(),
        CoreErrorCode::EvidenceAuthorityViolation,
        "derivation_input_ids",
    )?;
    assert_error(
        rebuild(&wire)?,
        CoreErrorCode::EvidenceAuthorityViolation,
        "derivation_input_ids",
    )
}

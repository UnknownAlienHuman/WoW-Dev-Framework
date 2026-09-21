use serde_json::{Value, json};
use wow_core::{
    ConflictAffectedRef, ConflictRecord, CoreErrorCode, CoreResult, EvidenceId,
    canonical_json_bytes, derive_conflict_id, relate_evidence_conflict, validate_conflict_record,
};

use super::support::*;

fn conflict() -> TestResult<ConflictRecord> {
    Ok(relate_evidence_conflict(
        context()?.context_id(),
        "fixture.conflict".parse()?,
        vec![
            EvidenceId::derive(&"first")?,
            EvidenceId::derive(&"second")?,
        ],
        vec![
            ConflictAffectedRef::new(
                "fixture.read".parse()?,
                Some("partition:fixture:a".parse()?),
            ),
            ConflictAffectedRef::new("fixture.read".parse()?, None),
        ],
        None,
    )?)
}

fn rebuild(wire: &Value) -> TestResult<CoreResult<ConflictRecord>> {
    Ok(ConflictRecord::new(
        serde_json::from_value(wire["context_id"].clone())?,
        serde_json::from_value(wire["conflict_code"].clone())?,
        serde_json::from_value(wire["evidence_ids"].clone())?,
        serde_json::from_value(wire["affected_refs"].clone())?,
        wire.get("subject_entity_key")
            .cloned()
            .map(serde_json::from_value)
            .transpose()?,
    ))
}

#[test]
fn conflict_001_007_008_009_preserves_canonical_scope_identity_and_order() -> TestResult {
    let original = conflict()?;
    validate_conflict_record(&original)?;
    assert_eq!(derive_conflict_id(&original)?, original.conflict_id());
    let mut wire = serde_json::to_value(&original)?;
    wire["evidence_ids"]
        .as_array_mut()
        .ok_or("evidence refs")?
        .reverse();
    wire["affected_refs"]
        .as_array_mut()
        .ok_or("affected refs")?
        .reverse();
    assert_eq!(rebuild(&wire)??, original);
    let fixture: Value = serde_json::from_str(include_str!(
        "../../examples/e0-conflict-not-evaluated-result.json"
    ))?;
    for wire in fixture["conflicts"].as_array().ok_or("conflicts")? {
        let record: ConflictRecord = serde_json::from_value(wire.clone())?;
        record.validate()?;
        assert_eq!(rebuild(wire)??, record);
        assert_eq!(canonical_json_bytes(&record)?, canonical_json_bytes(wire)?);
    }
    Ok(())
}

#[test]
fn conflict_002_003_006_constructor_and_resealed_wire_reject_invalid_sets() -> TestResult {
    let base = serde_json::to_value(conflict()?)?;
    for name in ["affected_refs", "evidence_ids"] {
        let minimum = if name == "evidence_ids" { 2 } else { 1 };
        for count in 0..minimum {
            let mut wire = base.clone();
            wire[name]
                .as_array_mut()
                .ok_or("references")?
                .truncate(count);
            assert_error(
                reseal_conflict(&mut wire)?.validate(),
                CoreErrorCode::ConflictScopeEmpty,
                name,
            )?;
            assert_error(rebuild(&wire)?, CoreErrorCode::ConflictScopeEmpty, name)?;
        }
        let expected = if name == "evidence_ids" {
            CoreErrorCode::DuplicateEvidenceReference
        } else {
            CoreErrorCode::ConflictScopeEmpty
        };
        let mut wire = base.clone();
        let entries = wire[name].as_array_mut().ok_or("references")?;
        entries.insert(0, entries[0].clone());
        assert_error(reseal_conflict(&mut wire)?.validate(), expected, name)?;
        assert_error(rebuild(&wire)?, expected, name)?;
        let mut wire = base.clone();
        wire[name].as_array_mut().ok_or("references")?.reverse();
        assert_error(reseal_conflict(&mut wire)?.validate(), expected, name)?;
        assert_eq!(rebuild(&wire)??, rebuild(&base)??);
    }
    Ok(())
}

#[test]
fn conflict_010_rejects_unknown_fields_including_silent_resolution() -> TestResult {
    let base = serde_json::to_value(conflict()?)?;
    for name in ["winner", "resolved", "note", "confidence"] {
        let mut wire = base.clone();
        wire[name] = json!("not a core conflict field");
        assert!(
            serde_json::from_value::<ConflictRecord>(wire).is_err(),
            "{name}"
        );
    }
    Ok(())
}

#[test]
fn conflict_013_id_revalidation_is_not_bypassed_by_valid_scope_shape() -> TestResult {
    let mut wire = serde_json::to_value(conflict()?)?;
    wire["conflict_code"] = "fixture.other_conflict".into();
    let decoded: ConflictRecord = serde_json::from_value(wire.clone())?;
    assert_error(
        decoded.validate(),
        CoreErrorCode::CanonicalDigestMismatch,
        "conflict_id",
    )?;
    let resealed = reseal_conflict(&mut wire)?;
    resealed.validate()?;
    assert_ne!(resealed.conflict_id(), decoded.conflict_id());
    Ok(())
}

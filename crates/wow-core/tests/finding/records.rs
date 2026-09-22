use super::support::*;
use serde_json::json;
use wow_core::{
    ClaimScope, CoreErrorCode, EvidenceConfidence, EvidenceRecord, Finding, FindingId,
    MessageArgument, MessageArgumentKind, ProvenanceClass, Remediation, RemediationClass,
    SourceHandle, WarningRecord, canonical_json_bytes, deduplicate_findings,
    derive_finding_fingerprint, derive_warning_id,
};

fn registry_error(
    f: &Fixture,
    sources: &[SourceHandle],
    evidence: &[EvidenceRecord],
    code: CoreErrorCode,
    field: &str,
) -> TestResult {
    assert_error(f.draft()?.bind(f.context, sources, evidence), code, field)?;
    assert_error(
        f.finding.validate(f.context, sources, evidence),
        code,
        field,
    )?;
    assert_error(
        f.warning()?.validate(f.context, sources, evidence),
        code,
        field,
    )
}

#[test]
fn finding_bind_001_007_rebuilds_golden_records_with_separate_authority_sources() -> TestResult {
    let f = Fixture::new()?;
    for value in f.value["findings"].as_array().ok_or("findings")? {
        let rebuilt = draft(value)?.bind(f.context, &f.sources, &f.evidence)?;
        let decoded: Finding = serde_json::from_value(value.clone())?;
        decoded.validate(f.context, &f.sources, &f.evidence)?;
        assert_eq!(rebuilt, decoded);
        assert_eq!(
            canonical_json_bytes(&rebuilt)?,
            canonical_json_bytes(value)?
        );
    }
    assert!(f.evidence.iter().any(|e| {
        e.claim_scope() == ClaimScope::PlatformContract
            && !e
                .source_handle_ids()
                .contains(&f.finding.primary_source_handle_id())
    }));
    Ok(())
}

#[test]
fn finding_fingerprint_001_006_identity_and_presentation_are_independent() -> TestResult {
    let f = Fixture::new()?;
    let base = f.value["findings"][1].clone();
    let fingerprint = derive_finding_fingerprint(&draft(&base)?)?;
    for (key, value) in [("severity", json!("hint")), ("policy", json!("shadow"))] {
        let mut changed = base.clone();
        changed[key] = value;
        assert_eq!(derive_finding_fingerprint(&draft(&changed)?)?, fingerprint);
    }
    let mut changed = base.clone();
    changed["message_arguments"][1]["value"] = json!("display-only");
    assert_eq!(draft(&changed)?.fingerprint()?, fingerprint);
    for (key, value) in [
        ("rule_version", json!("0.1.1")),
        ("finding_code", json!("fixture.other_finding")),
        ("subject_entity_key", json!("entity:api:C_Fixture.Other")),
    ] {
        let mut changed = base.clone();
        changed[key] = value;
        assert_ne!(draft(&changed)?.fingerprint()?, fingerprint);
    }
    let mut changed = base.clone();
    changed["message_arguments"][0]["value"] = json!("C_Fixture.Other");
    assert_ne!(draft(&changed)?.fingerprint()?, fingerprint);
    changed = base;
    changed["context_id"] = other_context()?.to_string().into();
    let moved =
        draft(&changed)?
            .evidence_ids(Vec::new())
            .bind(other_context()?, &f.sources, &[])?;
    assert_eq!(moved.fingerprint(), fingerprint);
    assert_ne!(moved.finding_id(), f.finding.finding_id());
    Ok(())
}

#[test]
fn finding_bind_002_and_warning_002_keep_the_public_context_guard() -> TestResult {
    let f = Fixture::new()?;
    assert_error(
        f.draft()?.bind(other_context()?, &[], &[]),
        CoreErrorCode::FindingContextMismatch,
        "context_id",
    )?;
    assert_error(
        f.finding.validate(other_context()?, &[], &[]),
        CoreErrorCode::FindingContextMismatch,
        "context_id",
    )?;
    assert_error(
        f.warning()?.validate(other_context()?, &[], &[]),
        CoreErrorCode::WarningContextMismatch,
        "context_id",
    )
}

#[test]
fn finding_bind_003_004_reports_missing_primary_related_and_evidence() -> TestResult {
    let f = Fixture::new()?;
    let missing = format!("handle:sha256:{}", "ff".repeat(32));
    let mut value = f.value["findings"][1].clone();
    value["primary_source_handle_id"] = missing.clone().into();
    assert_error(
        draft(&value)?.bind(f.context, &f.sources, &f.evidence),
        CoreErrorCode::MissingSourceHandle,
        "source_handle_ids",
    )?;
    value = f.value["findings"][1].clone();
    value["related_source_handle_ids"] = json!([missing]);
    assert_error(
        f.check(value.clone())?,
        CoreErrorCode::MissingSourceHandle,
        "source_handle_ids",
    )?;
    assert_error(
        draft(&value)?.bind(f.context, &f.sources, &f.evidence),
        CoreErrorCode::MissingSourceHandle,
        "source_handle_ids",
    )?;
    value = f.value["findings"][1].clone();
    value["evidence_ids"] = json!([format!("evidence:sha256:{}", "ff".repeat(32))]);
    assert_error(
        f.check(value.clone())?,
        CoreErrorCode::MissingEvidenceReference,
        "evidence_ids",
    )?;
    assert_error(
        draft(&value)?.bind(f.context, &f.sources, &f.evidence),
        CoreErrorCode::MissingEvidenceReference,
        "evidence_ids",
    )
}

#[test]
fn finding_bind_008_rejects_foreign_evidence_before_accepting_a_matching_id() -> TestResult {
    let f = Fixture::new()?;
    let foreign = leaf(other_context()?, false)?;
    let evidence = vec![foreign.clone()];
    assert_error(
        f.draft()?
            .evidence_ids(vec![foreign.evidence_id()])
            .bind(f.context, &f.sources, &evidence),
        CoreErrorCode::EvidenceContextMismatch,
        "evidence_records.context_id",
    )?;
    registry_error(
        &f,
        &f.sources,
        &evidence,
        CoreErrorCode::EvidenceContextMismatch,
        "evidence_records.context_id",
    )?;
    let mut mixed = f.evidence.clone();
    mixed.push(foreign);
    registry_error(
        &f,
        &f.sources,
        &mixed,
        CoreErrorCode::EvidenceContextMismatch,
        "evidence_records.context_id",
    )
}

#[test]
fn finding_bind_009_validates_handle_content_before_registry_membership() -> TestResult {
    let f = Fixture::new()?;
    let mut sources = f.sources.clone();
    let mut value = serde_json::to_value(&sources[0])?;
    value["revision"] = json!("fixture:changed-with-stale-id");
    sources[0] = serde_json::from_value(value)?;
    registry_error(
        &f,
        &sources,
        &f.evidence,
        CoreErrorCode::CanonicalDigestMismatch,
        "handle_id",
    )?;
    let mut sources = f.sources.clone();
    sources.push(sources[0].clone());
    registry_error(
        &f,
        &sources,
        &f.evidence,
        CoreErrorCode::ResultDuplicateId,
        "source_handles",
    )
}

#[test]
fn finding_bind_010_validates_evidence_before_reading_confidence() -> TestResult {
    let f = Fixture::new()?;
    let mut evidence = f.evidence.clone();
    let mut value = serde_json::to_value(&evidence[0])?;
    value["producer_version"] = json!("99.0.0");
    evidence[0] = serde_json::from_value(value)?;
    registry_error(
        &f,
        &f.sources,
        &evidence,
        CoreErrorCode::CanonicalDigestMismatch,
        "evidence_id",
    )?;
    let mut evidence = f.evidence.clone();
    evidence.push(evidence[0].clone());
    registry_error(
        &f,
        &f.sources,
        &evidence,
        CoreErrorCode::DuplicateEvidenceReference,
        "evidence_records.evidence_id",
    )
}

#[test]
fn finding_bind_011_requires_full_ancestry_and_rejects_confidence_laundering() -> TestResult {
    let mut f = Fixture::new()?;
    let candidate = leaf(f.context, true)?;
    let child = EvidenceRecord::new(
        f.context,
        ProvenanceClass::ProjectSource,
        EvidenceConfidence::Derived,
        ClaimScope::ProjectFact,
        "fixture.child".parse()?,
        "0.1.0".parse()?,
        Vec::new(),
        Vec::new(),
        vec![candidate.evidence_id()],
    )?;
    f.value["findings"][1]["evidence_ids"] = json!([child.evidence_id()]);
    f.finding = serde_json::from_value(f.value["findings"][1].clone())?;
    f.evidence = vec![child.clone()];
    registry_error(
        &f,
        &f.sources,
        std::slice::from_ref(&child),
        CoreErrorCode::MissingEvidenceReference,
        "evidence_records.derivation_input_ids",
    )?;
    registry_error(
        &f,
        &f.sources,
        &[candidate, child],
        CoreErrorCode::EvidenceAuthorityViolation,
        "evidence_records.derivation_input_ids",
    )
}

#[test]
fn finding_bind_012_requires_evidence_source_closure() -> TestResult {
    let mut f = Fixture::new()?;
    let mut value = serde_json::to_value(leaf(f.context, false)?)?;
    value["source_handle_ids"] = json!([format!("handle:sha256:{}", "ff".repeat(32))]);
    let evidence = reseal_evidence(&mut value)?;
    evidence.validate()?;
    f.value["findings"][1]["evidence_ids"] = json!([evidence.evidence_id()]);
    f.finding = serde_json::from_value(f.value["findings"][1].clone())?;
    f.evidence = vec![evidence.clone()];
    registry_error(
        &f,
        &f.sources,
        &[evidence],
        CoreErrorCode::MissingSourceHandle,
        "evidence_records.source_handle_ids",
    )
}

#[test]
fn finding_bind_013_retained_reference_sets_are_not_silently_repaired() -> TestResult {
    let f = Fixture::new()?;
    let mut base = f.value["findings"][1].clone();
    base["related_source_handle_ids"] = json!([f.sources[0].handle_id(), f.sources[1].handle_id()]);
    for key in [
        "evidence_ids",
        "related_source_handle_ids",
        "required_capability_ids",
    ] {
        for duplicate in [false, true] {
            let mut value = base.clone();
            let entries = value[key].as_array_mut().ok_or("reference array")?;
            if duplicate {
                entries.insert(0, entries[0].clone());
            } else {
                entries.reverse();
            }
            // These fields are outside fingerprint material: IDs still match.
            assert_error(f.check(value)?, CoreErrorCode::ResultDuplicateId, key)?;
        }
    }
    Ok(())
}

#[test]
fn finding_bind_014_decoded_remediation_requires_recipe_and_plan_reference() -> TestResult {
    let f = Fixture::new()?;
    for class in [
        RemediationClass::ExactEdit,
        RemediationClass::ValidatedRecipe,
    ] {
        assert_error(
            Remediation::new(class, None, None),
            CoreErrorCode::RemediationAuthorityViolation,
            "remediation.recipe_id",
        )?;
        let wire = json!({"class":class});
        let mut value = f.value["findings"][1].clone();
        value["remediation"] = wire.clone();
        assert_error(
            f.check(value)?,
            CoreErrorCode::RemediationAuthorityViolation,
            "remediation.recipe_id",
        )?;
        let decoded: Remediation = serde_json::from_value(wire)?;
        assert_error(
            f.draft()?
                .remediation(decoded)
                .bind(f.context, &f.sources, &f.evidence),
            CoreErrorCode::RemediationAuthorityViolation,
            "remediation.recipe_id",
        )?;
    }
    let mut value = f.value["findings"][1].clone();
    value["remediation"] =
        json!({"class":"plan_only","plan_handle_id":format!("handle:sha256:{}", "ff".repeat(32))});
    assert_error(
        f.check(value)?,
        CoreErrorCode::MissingSourceHandle,
        "remediation.plan_handle_id",
    )
}

#[test]
fn finding_bind_005_015_candidate_and_empty_evidence_cannot_authorize_exact_edit() -> TestResult {
    let f = Fixture::new()?;
    let candidate = leaf(f.context, true)?;
    let edit = Remediation::new(
        RemediationClass::ExactEdit,
        Some("fixture.recipe".parse()?),
        None,
    )?;
    for records in [Vec::new(), vec![candidate.clone()]] {
        let ids = records.iter().map(EvidenceRecord::evidence_id).collect();
        assert_error(
            f.draft()?
                .evidence_ids(ids)
                .remediation(edit.clone())
                .bind(f.context, &f.sources, &records),
            CoreErrorCode::RemediationAuthorityViolation,
            "remediation.class",
        )?;
    }
    for class in [RemediationClass::PlanOnly, RemediationClass::CandidateOnly] {
        let finding = f
            .draft()?
            .evidence_ids(vec![candidate.evidence_id()])
            .remediation(Remediation::new(class, None, None)?)
            .bind(f.context, &f.sources, std::slice::from_ref(&candidate))?;
        finding.validate(f.context, &f.sources, std::slice::from_ref(&candidate))?;
    }
    // Unreferenced optional candidates do not taint admitted local evidence.
    let mut evidence = f.evidence.clone();
    evidence.push(candidate);
    let accepted = f
        .draft()?
        .remediation(edit)
        .bind(f.context, &f.sources, &evidence)?;
    accepted.validate(f.context, &f.sources, &evidence)?;
    Ok(())
}

#[test]
fn finding_bind_016_valid_remediation_preserves_nonexecuting_metadata() -> TestResult {
    let f = Fixture::new()?;
    for class in [
        RemediationClass::ExactEdit,
        RemediationClass::ValidatedRecipe,
        RemediationClass::PlanOnly,
        RemediationClass::CandidateOnly,
    ] {
        let remediation = Remediation::new(
            class,
            Some("fixture.recipe".parse()?),
            Some(f.sources[0].handle_id()),
        )?;
        let finding =
            f.draft()?
                .remediation(remediation.clone())
                .bind(f.context, &f.sources, &f.evidence)?;
        assert_eq!(
            serde_json::to_value(finding)?["remediation"],
            serde_json::to_value(remediation)?
        );
    }
    Ok(())
}

#[test]
fn finding_dedup_001_003_preserves_distinct_semantics_and_rejects_conflicting_copies() -> TestResult
{
    let f = Fixture::new()?;
    assert_eq!(
        deduplicate_findings(vec![f.finding.clone(), f.finding.clone()])?,
        vec![f.finding.clone()]
    );
    let mut value = serde_json::to_value(&f.finding)?;
    value["severity"] = json!("hint");
    let changed: Finding = serde_json::from_value(value)?;
    changed.validate(f.context, &f.sources, &f.evidence)?;
    assert_error(
        deduplicate_findings(vec![f.finding.clone(), changed]),
        CoreErrorCode::ResultDuplicateId,
        "findings",
    )?;
    let changed = f
        .draft()?
        .message_arguments(vec![MessageArgument::new(
            "api",
            MessageArgumentKind::Identifier,
            "C_Fixture.Other",
            true,
        )?])?
        .bind(f.context, &f.sources, &f.evidence)?;
    assert_eq!(deduplicate_findings(vec![f.finding, changed])?.len(), 2);
    Ok(())
}

#[test]
fn warning_001_006_rederives_identity_after_canonical_construction() -> TestResult {
    let f = Fixture::new()?;
    let warning = f.warning()?;
    warning.validate(f.context, &f.sources, &f.evidence)?;
    assert_eq!(derive_warning_id(&warning)?, warning.warning_id());
    let mut value = serde_json::to_value(&warning)?;
    value["evidence_ids"].as_array_mut().ok_or("ids")?.reverse();
    value["related_source_handle_ids"]
        .as_array_mut()
        .ok_or("ids")?
        .reverse();
    let rebuilt = WarningRecord::new(
        f.context,
        field(&value, "producer_id")?,
        field(&value, "producer_version")?,
        field(&value, "warning_code")?,
        Some((field(&value, "subject_kind")?, field(&value, "subject_id")?)),
        field(&value, "primary_source_handle_id")?,
        field(&value, "related_source_handle_ids")?,
        field(&value, "evidence_ids")?,
        field(&value, "message_arguments")?,
    )?;
    assert_eq!(rebuilt, warning);
    Ok(())
}

#[test]
fn warning_007_resealed_subject_requires_pair_grammar_and_bounded_text() -> TestResult {
    let f = Fixture::new()?;
    let base = serde_json::to_value(f.warning()?)?;
    for missing in ["subject_kind", "subject_id"] {
        let mut value = base.clone();
        value.as_object_mut().ok_or("object")?.remove(missing);
        let warning = reseal_warning(&mut value)?;
        assert_error(
            warning.validate(f.context, &f.sources, &f.evidence),
            CoreErrorCode::InvalidMessageArgument,
            "subject",
        )?;
    }
    for id in [
        "".to_owned(),
        " value".to_owned(),
        "value\n".to_owned(),
        "x".repeat(4097),
    ] {
        let mut value = base.clone();
        value["subject_id"] = id.into();
        let warning = reseal_warning(&mut value)?;
        assert_error(
            warning.validate(f.context, &f.sources, &f.evidence),
            CoreErrorCode::InvalidMessageArgument,
            "subject_id",
        )?;
    }
    let mut value = base;
    value["subject_kind"] = json!("BadKind");
    let warning = reseal_warning(&mut value)?;
    assert_error(
        warning.validate(f.context, &f.sources, &f.evidence),
        CoreErrorCode::InvalidIdentifier,
        "subject_kind",
    )
}

#[test]
fn warning_008_resealed_reference_sets_require_canonical_uniqueness() -> TestResult {
    let f = Fixture::new()?;
    for key in ["related_source_handle_ids", "evidence_ids"] {
        for duplicate in [false, true] {
            let mut value = serde_json::to_value(f.warning()?)?;
            let entries = value[key].as_array_mut().ok_or("array")?;
            if duplicate {
                entries.insert(0, entries[0].clone());
            } else {
                entries.reverse();
            }
            let warning = reseal_warning(&mut value)?;
            assert_error(
                warning.validate(f.context, &f.sources, &f.evidence),
                CoreErrorCode::ResultDuplicateId,
                key,
            )?;
        }
    }
    Ok(())
}

#[test]
fn finding_warning_009_reject_changed_identity_and_unknown_wire_fields() -> TestResult {
    let f = Fixture::new()?;
    let mut value = serde_json::to_value(&f.finding)?;
    value["finding_id"] = FindingId::derive(&"changed")?.to_string().into();
    assert_error(
        f.check(value)?,
        CoreErrorCode::CanonicalDigestMismatch,
        "finding_id",
    )?;
    let mut value = serde_json::to_value(f.warning()?)?;
    value["warning_code"] = json!("fixture.changed");
    let warning: WarningRecord = serde_json::from_value(value)?;
    assert_error(
        warning.validate(f.context, &f.sources, &f.evidence),
        CoreErrorCode::CanonicalDigestMismatch,
        "warning_id",
    )?;
    for key in ["rendered_message", "timestamp", "rule_id"] {
        let mut value = serde_json::to_value(f.warning()?)?;
        value[key] = json!("forbidden");
        assert!(serde_json::from_value::<WarningRecord>(value).is_err());
    }
    let mut value = serde_json::to_value(&f.finding)?;
    value["rendered_message"] = json!("forbidden");
    assert!(serde_json::from_value::<Finding>(value).is_err());
    Ok(())
}

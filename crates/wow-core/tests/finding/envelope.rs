use super::support::*;
use serde_json::{Value, json};
use wow_core::{
    CoreErrorCode, E0CheckResultEnvelope, Finding, canonical_finding_order, canonical_json_bytes,
};

#[test]
fn envelope_027_resealed_finding_sets_and_remediation_reject_before_success() -> TestResult {
    let f = Fixture::new()?;
    for (key, payload, code, path) in [
        (
            "evidence_ids",
            json!([
                f.value["findings"][1]["evidence_ids"][0],
                f.value["findings"][1]["evidence_ids"][0]
            ]),
            CoreErrorCode::ResultDuplicateId,
            "evidence_ids",
        ),
        (
            "remediation",
            json!({"class":"exact_edit"}),
            CoreErrorCode::RemediationAuthorityViolation,
            "remediation.recipe_id",
        ),
        (
            "remediation",
            json!({"class":"plan_only","plan_handle_id":format!("handle:sha256:{}","ff".repeat(32))}),
            CoreErrorCode::MissingSourceHandle,
            "remediation.plan_handle_id",
        ),
    ] {
        let mut value = f.value.clone();
        value["findings"][1][key] = payload;
        assert_error(reseal_envelope(&mut value)?.validate(), code, path)?;
        assert_error(envelope_draft(&value)?.finalize(), code, path)?;
    }
    Ok(())
}

#[test]
fn envelope_028_resealed_warning_subject_and_reference_sets_reject() -> TestResult {
    let f = Fixture::new()?;
    for bad_subject in [false, true] {
        let mut warning = serde_json::to_value(f.warning()?)?;
        let field = if bad_subject {
            warning
                .as_object_mut()
                .ok_or("warning")?
                .remove("subject_kind");
            "subject"
        } else {
            let refs = warning["evidence_ids"].as_array_mut().ok_or("refs")?;
            refs.insert(0, refs[0].clone());
            "evidence_ids"
        };
        let warning = reseal_warning(&mut warning)?;
        let mut value = f.value.clone();
        value["warnings"] = json!([warning]);
        value["budget"]["usage"]["warnings"] = 1.into();
        let code = if bad_subject {
            CoreErrorCode::InvalidMessageArgument
        } else {
            CoreErrorCode::ResultDuplicateId
        };
        assert_error(reseal_envelope(&mut value)?.validate(), code, field)?;
        assert_error(envelope_draft(&value)?.finalize(), code, field)?;
    }
    Ok(())
}

#[test]
fn finding_order_001_012_and_envelope_019_seeded_orders_keep_golden_bytes() -> TestResult {
    let f = Fixture::new()?;
    let expected: E0CheckResultEnvelope = serde_json::from_value(f.value.clone())?;
    let bytes = expected.canonical_bytes()?;
    let mut sorted: Vec<Finding> = field(&f.value, "findings")?;
    canonical_finding_order(&mut sorted, &f.sources)?;
    for seed in 0_u64..64 {
        let mut state = seed;
        let mut value = f.value.clone();
        for key in [
            "findings",
            "source_handles",
            "evidence_records",
            "coverage_records",
            "capability_summaries",
        ] {
            let entries = value[key].as_array_mut().ok_or("collection")?;
            for index in (1..entries.len()).rev() {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let other = usize::try_from(state % u64::try_from(index + 1)?)?;
                entries.swap(index, other);
            }
        }
        let mut findings: Vec<Finding> = field(&value, "findings")?;
        let sources = field::<Vec<_>>(&value, "source_handles")?;
        canonical_finding_order(&mut findings, &sources)?;
        assert_eq!(findings, sorted, "seed {seed}");
        assert_eq!(
            envelope_draft(&value)?.finalize()?.canonical_bytes()?,
            bytes,
            "seed {seed}"
        );
    }
    Ok(())
}

#[test]
fn finalize_003_diagnostic_admission_preserves_all_committed_result_bytes() -> TestResult {
    for text in [
        FINDINGS,
        include_str!("../../examples/e0-clean-result.json"),
        include_str!("../../examples/e0-not-evaluated-result.json"),
        include_str!("../../examples/e0-conflict-not-evaluated-result.json"),
    ] {
        let value: Value = serde_json::from_str(text)?;
        let decoded: E0CheckResultEnvelope = serde_json::from_value(value.clone())?;
        assert_eq!(decoded.canonical_bytes()?, canonical_json_bytes(&value)?);
        assert_eq!(
            envelope_draft(&value)?.finalize()?.canonical_bytes()?,
            decoded.canonical_bytes()?
        );
    }
    Ok(())
}

use serde_json::{Value, json};
use wow_core::{
    Budget, BudgetUsage, CoreErrorCode, NegativeAuthorityOutcome, NegativeAuthorityReason,
    TruncationEntry, TruncationState, canonical_json_bytes, classify_truncation,
};

use super::support::*;

#[test]
fn truncation_001_empty_classification_is_not_an_empty_truncated_state() -> TestResult {
    assert_eq!(
        classify_truncation(Vec::new())?,
        TruncationState::NotTruncated
    );
    assert_eq!(
        Budget::new(
            limits()?,
            BudgetUsage::default(),
            TruncationState::NotTruncated
        )?
        .truncation(),
        &TruncationState::NotTruncated
    );
    assert_error(
        Budget::new(
            limits()?,
            BudgetUsage::default(),
            TruncationState::Truncated {
                entries: Vec::new(),
            },
        ),
        CoreErrorCode::ContractViolation,
        Some("truncation.entries"),
    )?;
    Ok(())
}

#[test]
fn truncation_002_003_known_counts_remain_distinct_from_unknown() -> TestResult {
    // The contract allows an exact nonnegative count, including zero. Unknown
    // must be represented by no count plus count_unknown, never invented zero.
    for (count, unknown) in [
        (Some(0), false),
        (Some(1), false),
        (Some(u64::MAX), false),
        (None, true),
    ] {
        let entry = TruncationEntry::new(
            "findings",
            Vec::new(),
            count,
            unknown,
            "fixture.output_limited".parse()?,
        )?;
        let encoded = serde_json::to_value(&entry)?;
        assert_eq!(encoded.get("omitted_count").and_then(Value::as_u64), count);
        assert_eq!(encoded["count_unknown"], unknown);
        let state = classify_truncation(vec![entry])?;
        assert!(state.is_truncated());
        assert_eq!(state.entries().len(), 1);
        let budget = Budget::new(limits()?, BudgetUsage::default(), state.clone())?;
        let decoded: Budget = serde_json::from_slice(&canonical_json_bytes(&budget)?)?;
        decoded.validate_limits()?;
        assert_eq!(decoded, budget);
        let decision = negative(&state)??;
        assert_eq!(
            decision.outcome(),
            NegativeAuthorityOutcome::NotAuthoritative
        );
        assert!(
            decision
                .reasons()
                .contains(&NegativeAuthorityReason::ResultTruncated)
        );
    }
    Ok(())
}

#[test]
fn truncation_006_builders_order_sets_without_changing_omission_truth() -> TestResult {
    let ordered = entry("findings", &["fixture.alpha", "fixture.beta"])?;
    let duplicate = entry(
        "findings",
        &["fixture.beta", "fixture.alpha", "fixture.beta"],
    )?;
    assert_eq!(
        canonical_json_bytes(&ordered)?,
        canonical_json_bytes(&duplicate)?
    );
    let entries = vec![
        ordered,
        entry("warnings", &[])?,
        entry("evidence_records", &["fixture.alpha"])?,
    ];
    let expected = classify_truncation(entries.clone())?;
    let golden = canonical_json_bytes(&expected)?;
    for seed in 0_u64..64 {
        let mut shuffled = entries.clone();
        let mut state = seed;
        for index in (1..shuffled.len()).rev() {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let other = usize::try_from(state % u64::try_from(index + 1)?)?;
            shuffled.swap(index, other);
        }
        assert_eq!(
            canonical_json_bytes(&classify_truncation(shuffled.clone())?)?,
            golden,
            "seed {seed}"
        );
        let budget = Budget::new(
            limits()?,
            BudgetUsage::default(),
            TruncationState::Truncated { entries: shuffled },
        )?;
        assert_eq!(
            canonical_json_bytes(budget.truncation())?,
            golden,
            "seed {seed}"
        );
    }
    Ok(())
}

#[test]
fn truncation_007_decoded_budget_rejects_empty_duplicate_and_unordered_records() -> TestResult {
    let a = serde_json::to_value(entry("findings", &[])?)?;
    let b = serde_json::to_value(entry("warnings", &[])?)?;
    let mut different_a = a.clone();
    different_a["omitted_count"] = 2.into();
    for malformed in [
        state(vec![]),
        state(vec![a.clone(), a.clone()]),
        state(vec![a.clone(), different_a]),
        state(vec![b, a]),
    ] {
        let budget = decoded_budget(&malformed)?;
        assert_error(
            budget.validate_limits(),
            CoreErrorCode::ContractViolation,
            Some("truncation.entries"),
        )?;
        assert_error(
            budget.with_output_bytes(0),
            CoreErrorCode::ContractViolation,
            Some("truncation.entries"),
        )?;
    }
    Ok(())
}

#[test]
fn truncation_008_classifier_and_budget_reject_decoded_count_contradictions() -> TestResult {
    for (count, unknown) in [(Some(0_u64), true), (Some(5), true), (None, false)] {
        let mut value = serde_json::to_value(entry("findings", &[])?)?;
        value["count_unknown"] = unknown.into();
        if let Some(count) = count {
            value["omitted_count"] = count.into();
        } else {
            value
                .as_object_mut()
                .ok_or("entry object")?
                .remove("omitted_count");
        }
        let decoded: TruncationEntry = serde_json::from_value(value.clone())?;
        assert_error(
            classify_truncation(vec![decoded.clone()]),
            CoreErrorCode::ContractViolation,
            Some("entries.omitted_count"),
        )?;
        assert_error(
            Budget::new(
                limits()?,
                BudgetUsage::default(),
                TruncationState::Truncated {
                    entries: vec![decoded],
                },
            ),
            CoreErrorCode::ContractViolation,
            Some("entries.omitted_count"),
        )?;
        assert_error(
            decoded_budget(&state(vec![value]))?.validate_limits(),
            CoreErrorCode::ContractViolation,
            Some("entries.omitted_count"),
        )?;
        assert_error(
            TruncationEntry::new(
                "findings",
                Vec::new(),
                count,
                unknown,
                "fixture.output_limited".parse()?,
            ),
            CoreErrorCode::ContractViolation,
            Some("entries.omitted_count"),
        )?;
    }
    Ok(())
}

#[test]
fn truncation_009_negative_authority_validates_retained_truncation() -> TestResult {
    let clean = negative(&TruncationState::NotTruncated)??;
    assert_eq!(
        clean.outcome(),
        NegativeAuthorityOutcome::AuthoritativeAbsent
    );
    assert!(clean.reasons().is_empty());
    let empty = TruncationState::Truncated {
        entries: Vec::new(),
    };
    assert_error(
        negative(&empty)?,
        CoreErrorCode::ContractViolation,
        Some("truncation.entries"),
    )?;
    let mut value = serde_json::to_value(entry("findings", &[])?)?;
    value["count_unknown"] = true.into();
    let malformed: TruncationState = serde_json::from_value(state(vec![value]))?;
    assert_error(
        negative(&malformed)?,
        CoreErrorCode::ContractViolation,
        Some("entries.omitted_count"),
    )?;
    Ok(())
}

#[test]
fn truncation_010_decoded_capability_sets_are_not_silently_repaired() -> TestResult {
    for capabilities in [
        json!(["fixture.alpha", "fixture.alpha"]),
        json!(["fixture.beta", "fixture.alpha"]),
    ] {
        let mut value = serde_json::to_value(entry("findings", &[])?)?;
        value["capability_ids"] = capabilities;
        let decoded: TruncationEntry = serde_json::from_value(value.clone())?;
        assert_error(
            classify_truncation(vec![decoded.clone()]),
            CoreErrorCode::ContractViolation,
            Some("entries.capability_ids"),
        )?;
        assert_error(
            Budget::new(
                limits()?,
                BudgetUsage::default(),
                TruncationState::Truncated {
                    entries: vec![decoded],
                },
            ),
            CoreErrorCode::ContractViolation,
            Some("entries.capability_ids"),
        )?;
        assert_error(
            decoded_budget(&state(vec![value]))?.validate_limits(),
            CoreErrorCode::ContractViolation,
            Some("entries.capability_ids"),
        )?;
    }
    Ok(())
}

#[test]
fn truncation_011_decoded_collection_names_match_constructor_admission_without_echo() -> TestResult
{
    for (name, code) in [
        ("", CoreErrorCode::IdentifierTooLong),
        ("Findings", CoreErrorCode::InvalidIdentifier),
        ("../private-input", CoreErrorCode::InvalidIdentifier),
        ("findings\u{0}", CoreErrorCode::InvalidIdentifier),
        ("latest", CoreErrorCode::ReservedIdentifierSegment),
    ] {
        let mut value = serde_json::to_value(entry("findings", &[])?)?;
        value["collection_id"] = name.into();
        let decoded: TruncationEntry = serde_json::from_value(value.clone())?;
        let result = classify_truncation(vec![decoded]);
        if let Err(error) = &result
            && !name.is_empty()
        {
            assert!(!serde_json::to_string(error)?.contains(name));
        }
        assert_error(result, code, Some("entries.collection_id"))?;
        assert_error(
            decoded_budget(&state(vec![value]))?.validate_limits(),
            code,
            Some("entries.collection_id"),
        )?;
    }
    Ok(())
}

#[test]
fn truncation_012_duplicate_collections_reject_even_when_their_payloads_differ() -> TestResult {
    for second in [
        entry("findings", &[])?,
        entry("findings", &["fixture.other"])?,
    ] {
        let entries = vec![entry("findings", &[])?, second];
        assert_error(
            classify_truncation(entries.clone()),
            CoreErrorCode::ContractViolation,
            Some("truncation.entries"),
        )?;
        assert_error(
            Budget::new(
                limits()?,
                BudgetUsage::default(),
                TruncationState::Truncated { entries },
            ),
            CoreErrorCode::ContractViolation,
            Some("truncation.entries"),
        )?;
    }
    Ok(())
}

#[test]
fn truncation_013_strict_wire_rejects_unknown_variants_and_payload_fields() -> TestResult {
    let valid = json!({"status":"not_truncated"});
    let decoded: TruncationState = serde_json::from_value(valid.clone())?;
    assert_eq!(decoded, TruncationState::NotTruncated);
    assert_eq!(
        canonical_json_bytes(&decoded)?,
        canonical_json_bytes(&valid)?
    );
    for value in [
        json!({"status":"invented"}),
        json!({"status":"not_truncated","entries":[]}),
        json!({"status":"not_truncated","hidden":true}),
        json!({"status":"not_truncated","entries":[{"omitted_count":5}]}),
        json!({"status":"truncated","entries":[],"hidden":true}),
    ] {
        assert!(
            serde_json::from_value::<TruncationState>(value.clone()).is_err(),
            "{value}"
        );
    }
    assert!(
        serde_json::from_str::<TruncationState>(
            r#"{"status":"not_truncated","status":"truncated","entries":[]}"#,
        )
        .is_err()
    );
    let mut value = serde_json::to_value(entry("findings", &[])?)?;
    value["hidden"] = true.into();
    assert!(serde_json::from_value::<TruncationEntry>(value).is_err());
    let wire = serde_json::to_string(&entry("findings", &[])?)?;
    assert!(
        serde_json::from_str::<TruncationEntry>(&wire.replacen('{', "{\"count_unknown\":true,", 1))
            .is_err()
    );
    Ok(())
}

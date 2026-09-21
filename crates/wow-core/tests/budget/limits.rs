use serde_json::{Value, json};
use wow_core::{
    Budget, BudgetLimits, BudgetUsage, CoreErrorCode, TruncationState, accumulate_budget_usage,
    canonical_json_bytes, classify_truncation, validate_budget,
};

use super::support::*;

#[test]
fn budget_001_each_dimension_accepts_both_implementation_boundaries() -> TestResult {
    for (limit, _) in DIMENSIONS {
        for boundary in [
            1_u64,
            if limit == "max_output_bytes" {
                1_073_741_824
            } else {
                10_000_000
            },
        ] {
            let mut value = serde_json::to_value(limits()?)?;
            value[limit] = boundary.into();
            let decoded: BudgetLimits = serde_json::from_value(value.clone())?;
            validate_budget(&decoded)?;
            assert_eq!(serde_json::to_value(decoded)?, value);
        }
    }
    Ok(())
}

#[test]
fn budget_002_003_each_dimension_rejects_zero_and_above_maximum() -> TestResult {
    for (limit, _) in DIMENSIONS {
        let max = if limit == "max_output_bytes" {
            1_073_741_824_u64
        } else {
            10_000_000
        };
        for invalid in [0, max + 1, u64::MAX] {
            let mut value = serde_json::to_value(limits()?)?;
            value[limit] = invalid.into();
            let decoded: BudgetLimits = serde_json::from_value(value)?;
            assert_error(
                validate_budget(&decoded),
                CoreErrorCode::BudgetInvalid,
                Some(limit),
            )?;
        }
    }
    Ok(())
}

#[test]
fn budget_004_wire_dimensions_and_scalar_types_are_strict() -> TestResult {
    let mut value = serde_json::to_value(limits()?)?;
    value["unknown_dimension"] = 1.into();
    assert!(serde_json::from_value::<BudgetLimits>(value).is_err());
    let mut value = serde_json::to_value(BudgetUsage::default())?;
    value["unknown_dimension"] = 1.into();
    assert!(serde_json::from_value::<BudgetUsage>(value).is_err());
    for (limit, usage) in DIMENSIONS {
        for invalid in [json!(-1), json!(1.5), json!("1"), Value::Null] {
            let mut value = serde_json::to_value(limits()?)?;
            value[limit] = invalid.clone();
            assert!(
                serde_json::from_value::<BudgetLimits>(value).is_err(),
                "{limit}"
            );
            let mut value = serde_json::to_value(BudgetUsage::default())?;
            value[usage] = invalid;
            assert!(
                serde_json::from_value::<BudgetUsage>(value).is_err(),
                "{usage}"
            );
        }
    }
    let wire = serde_json::to_string(&limits()?)?;
    let duplicate = wire.replacen('{', "{\"max_findings\":1,", 1);
    assert!(serde_json::from_str::<BudgetLimits>(&duplicate).is_err());
    Ok(())
}

#[test]
fn budget_usage_001_all_dimensions_add_exactly_and_preserve_operands() -> TestResult {
    let mut left = serde_json::to_value(BudgetUsage::default())?;
    let mut right = left.clone();
    let mut expected = left.clone();
    for (index, (_, usage)) in DIMENSIONS.iter().enumerate() {
        let value = u64::try_from(index)? + 1;
        left[*usage] = value.into();
        right[*usage] = (value * 2).into();
        expected[*usage] = (value * 3).into();
    }
    let a: BudgetUsage = serde_json::from_value(left.clone())?;
    let b: BudgetUsage = serde_json::from_value(right.clone())?;
    assert_eq!(
        serde_json::to_value(accumulate_budget_usage(a, b)?)?,
        expected
    );
    assert_eq!(
        accumulate_budget_usage(a, b)?,
        accumulate_budget_usage(b, a)?
    );
    assert_eq!(accumulate_budget_usage(a, BudgetUsage::default())?, a);
    assert_eq!(serde_json::to_value(a)?, left);
    assert_eq!(serde_json::to_value(b)?, right);
    Ok(())
}

#[test]
fn budget_usage_002_each_dimension_checks_u64_overflow_without_wrapping() -> TestResult {
    for (_, usage) in DIMENSIONS {
        let mut left = serde_json::to_value(BudgetUsage::default())?;
        let mut right = left.clone();
        left[usage] = (u64::MAX - 1).into();
        right[usage] = 1.into();
        let a: BudgetUsage = serde_json::from_value(left)?;
        let b: BudgetUsage = serde_json::from_value(right)?;
        let maximum = accumulate_budget_usage(a, b)?;
        assert_eq!(
            serde_json::to_value(maximum)?[usage].as_u64(),
            Some(u64::MAX)
        );
        assert_error(
            accumulate_budget_usage(maximum, b),
            CoreErrorCode::UsageOverflow,
            Some(usage),
        )?;
        assert_error(
            accumulate_budget_usage(b, maximum),
            CoreErrorCode::UsageOverflow,
            Some(usage),
        )?;
    }
    Ok(())
}

#[test]
fn budget_005_truncation_does_not_waive_any_actual_usage_limit() -> TestResult {
    let limits = limits()?;
    let encoded = serde_json::to_value(limits)?;
    let truncated = classify_truncation(vec![entry("findings", &["wow.api.exists"])?])?;
    for (limit, usage) in DIMENSIONS {
        let maximum = encoded[limit].as_u64().ok_or("limit")?;
        let mut value = serde_json::to_value(BudgetUsage::default())?;
        value[usage] = maximum.into();
        let exact: BudgetUsage = serde_json::from_value(value.clone())?;
        for state in [TruncationState::NotTruncated, truncated.clone()] {
            let admitted = Budget::new(limits, exact, state.clone())?;
            admitted.validate_limits()?;
            assert_eq!(admitted.usage(), exact);
            value[usage] = (maximum + 1).into();
            assert_error(
                Budget::new(limits, serde_json::from_value(value.clone())?, state),
                CoreErrorCode::BudgetExceeded,
                None,
            )?;
        }
    }
    Ok(())
}

#[test]
fn budget_006_output_byte_replacement_is_checked_and_other_truth_is_unchanged() -> TestResult {
    let budget = decoded_budget(&json!({"status":"not_truncated"}))?;
    let original = canonical_json_bytes(&budget)?;
    let replaced = budget
        .clone()
        .with_output_bytes(budget.limits().max_output_bytes)?;
    assert_eq!(replaced.limits(), budget.limits());
    assert_eq!(replaced.truncation(), budget.truncation());
    let mut expected = budget.usage();
    expected.output_bytes = budget.limits().max_output_bytes;
    assert_eq!(replaced.usage(), expected);
    assert_error(
        budget
            .clone()
            .with_output_bytes(budget.limits().max_output_bytes + 1),
        CoreErrorCode::BudgetExceeded,
        None,
    )?;
    assert_eq!(canonical_json_bytes(&budget)?, original);
    Ok(())
}

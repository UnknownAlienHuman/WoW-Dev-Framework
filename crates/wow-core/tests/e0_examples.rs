use wow_core::{E0CheckResultEnvelope, E0OperationErrorEnvelope};

#[test]
fn clean_result_validates_and_round_trips_canonically() -> Result<(), Box<dyn std::error::Error>> {
    validate_result(include_str!("../examples/e0-clean-result.json"))
}

#[test]
fn findings_result_validates_and_round_trips_canonically() -> Result<(), Box<dyn std::error::Error>>
{
    validate_result(include_str!("../examples/e0-findings-result.json"))
}

#[test]
fn not_evaluated_result_validates_and_round_trips_canonically()
-> Result<(), Box<dyn std::error::Error>> {
    validate_result(include_str!("../examples/e0-not-evaluated-result.json"))
}

#[test]
fn conflict_blocked_result_validates_and_round_trips_canonically()
-> Result<(), Box<dyn std::error::Error>> {
    validate_result(include_str!(
        "../examples/e0-conflict-not-evaluated-result.json"
    ))
}

#[test]
fn generation_mismatch_error_validates_and_round_trips_canonically()
-> Result<(), Box<dyn std::error::Error>> {
    let envelope: E0OperationErrorEnvelope = serde_json::from_str(include_str!(
        "../examples/e0-generation-mismatch-error.json"
    ))?;
    envelope.validate()?;
    let bytes = envelope.canonical_bytes()?;
    let reparsed: E0OperationErrorEnvelope = serde_json::from_slice(&bytes)?;
    reparsed.validate()?;
    assert_eq!(envelope, reparsed);
    Ok(())
}

fn validate_result(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let envelope: E0CheckResultEnvelope = serde_json::from_str(source)?;
    envelope.validate()?;
    let bytes = envelope.canonical_bytes()?;
    assert_eq!(bytes, envelope.canonical_bytes()?);
    let reparsed: E0CheckResultEnvelope = serde_json::from_slice(&bytes)?;
    reparsed.validate()?;
    assert_eq!(envelope, reparsed);
    Ok(())
}

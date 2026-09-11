use std::ffi::OsString;
use std::io::Cursor;

use wow_app::{EXIT_DIAGNOSTICS, EXIT_INPUT_ERROR, EXIT_SUCCESS, run};
use wow_rules::{
    AnalyzerResolution, ApiCallObservation, ApiPresence, ByteSpan, CoverageState,
    ReferenceApiEvidence, RuleEvaluationInput, SourceLocation, ValueRestriction,
};

fn boxed(value: &str) -> Box<str> {
    value.into()
}

fn input(presence: ApiPresence) -> RuleEvaluationInput {
    RuleEvaluationInput {
        project_snapshot_id: boxed("project-snapshot:test"),
        project_generation: boxed("project-generation:test"),
        reference_view_id: boxed("reference-view:test"),
        target_profile: boxed("mainline-test"),
        calls: vec![ApiCallObservation {
            fact_id: boxed("call:test"),
            source: SourceLocation::new(
                "main/test.lua",
                format!("sha256:{}", "a".repeat(64)),
                ByteSpan::new(0, 10).expect("valid test span"),
            ),
            receiver: boxed("C_Test"),
            member: boxed(if presence == ApiPresence::Present {
                "KnownApi"
            } else {
                "RemovedApi"
            }),
            resolution: if presence == ApiPresence::Present {
                AnalyzerResolution::Resolved
            } else {
                AnalyzerResolution::Unresolved
            },
        }],
        bindings: Vec::new(),
        operations: Vec::new(),
        guards: Vec::new(),
        dominance: Vec::new(),
        reference: vec![ReferenceApiEvidence {
            evidence_id: boxed("reference:test"),
            target_profile: boxed("mainline-test"),
            receiver: boxed("C_Test"),
            member: boxed(if presence == ApiPresence::Present {
                "KnownApi"
            } else {
                "RemovedApi"
            }),
            presence_coverage: CoverageState::Complete,
            presence,
            restriction_coverage: if presence == ApiPresence::Present {
                CoverageState::Complete
            } else {
                CoverageState::Unavailable
            },
            restriction: if presence == ApiPresence::Present {
                ValueRestriction::NonSecret
            } else {
                ValueRestriction::Unknown
            },
        }],
    }
}

fn execute(args: &[&str], bytes: &[u8]) -> (u8, Vec<u8>, Vec<u8>) {
    let args = args.iter().map(|value| OsString::from(*value));
    let mut output = Vec::new();
    let mut error = Vec::new();
    let code = run(args, Cursor::new(bytes), &mut output, &mut error);
    (code, output, error)
}

#[test]
fn operations_lists_the_versioned_service_contract() {
    let (code, output, error) = execute(&["operations"], &[]);
    assert_eq!(code, EXIT_SUCCESS);
    assert!(error.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output).expect("operations JSON");
    assert_eq!(value["schema"], "wow-cli/operations/1");
    assert_eq!(value["operations"][0]["operation_id"], "rules.evaluate");
    assert_eq!(value["operations"][0]["version"], 1);
}

#[test]
fn clean_and_diagnostic_results_use_distinct_exit_codes() {
    let clean = serde_json::to_vec(&input(ApiPresence::Present)).expect("clean input");
    let (code, output, error) = execute(&["rules-evaluate", "--input", "-"], &clean);
    assert_eq!(code, EXIT_SUCCESS);
    assert!(error.is_empty());
    let response: serde_json::Value = serde_json::from_slice(&output).expect("clean response");
    assert_eq!(response["status"], "completed");
    assert_eq!(response["result"]["diagnostics"].as_array().map(Vec::len), Some(0));

    let diagnostic = serde_json::to_vec(&input(ApiPresence::Absent)).expect("diagnostic input");
    let (code, output, error) = execute(&["rules-evaluate", "--input", "-"], &diagnostic);
    assert_eq!(code, EXIT_DIAGNOSTICS);
    assert!(error.is_empty());
    let response: serde_json::Value = serde_json::from_slice(&output).expect("diagnostic response");
    assert_eq!(response["result"]["diagnostics"].as_array().map(Vec::len), Some(1));
}

#[test]
fn malformed_input_and_invalid_invocation_fail_without_result_bytes() {
    let (code, output, error) = execute(&["rules-evaluate", "--input", "-"], b"{broken");
    assert_eq!(code, EXIT_INPUT_ERROR);
    assert!(output.is_empty());
    assert!(!error.is_empty());

    let (code, output, error) = execute(&["unknown"], &[]);
    assert_eq!(code, EXIT_INPUT_ERROR);
    assert!(output.is_empty());
    assert!(!error.is_empty());
}

#[test]
fn identical_inputs_produce_identical_canonical_bytes() {
    let bytes = serde_json::to_vec(&input(ApiPresence::Present)).expect("input");
    let first = execute(&["rules-evaluate", "--input", "-"], &bytes);
    let second = execute(&["rules-evaluate", "--input", "-"], &bytes);
    assert_eq!(first, second);
}

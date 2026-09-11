use std::path::PathBuf;
use std::process::Command;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_wow")
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/e0-clean-context.json")
}

#[test]
fn status_writes_one_json_result_to_stdout() {
    let output = Command::new(binary())
        .args([
            "status",
            "--input",
            fixture().to_str().expect("UTF-8 fixture path"),
            "--operation-id",
            "operation:cli:status",
        ])
        .output()
        .expect("run wow status");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("status JSON output");
    assert_eq!(value["schema"], "wow-service/result-envelope/1");
    assert_eq!(value["health"], "ready");
    assert!(value.get("semantic_status").is_none());
}

#[test]
fn clean_check_uses_explicit_current_selector_and_all_scope() {
    let output = Command::new(binary())
        .args([
            "check",
            "--input",
            fixture().to_str().expect("UTF-8 fixture path"),
            "--operation-id",
            "operation:cli:check",
            "--current",
            "project:fixture",
            "--all",
        ])
        .output()
        .expect("run wow check");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check JSON output");
    assert_eq!(value["semantic_status"], "clean");
    assert_eq!(value["raw_findings"], serde_json::json!([]));
    assert_eq!(value["rule_evaluations"].as_array().map(Vec::len), Some(2));
}

#[test]
fn missing_selector_is_usage_error_on_stderr_only() {
    let output = Command::new(binary())
        .args([
            "check",
            "--input",
            fixture().to_str().expect("UTF-8 fixture path"),
            "--operation-id",
            "operation:cli:invalid",
            "--all",
        ])
        .output()
        .expect("run invalid wow check");
    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("error JSON output");
    assert_eq!(value["schema"], "wow-app/error/1");
    assert_eq!(value["code"], "usage");
}

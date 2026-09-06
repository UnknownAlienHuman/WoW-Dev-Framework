//! Real semantic checks, deliberately separate from syntax and Ketho byte parity.
mod consumer_support;
use consumer_support::{
    Result, Workspace, fixture, mutations, package,
    process::{Consumer, Executable},
    report,
};
use serde_json::json;
use std::fs;

#[test]
#[ignore = "requires both explicitly approved consumer executables; mandatory in the consumer CI job"]
fn both_consumers_interpret_generated_library() -> Result<()> {
    let executables = [
        Executable::approved(Consumer::Emmy)?,
        Executable::approved(Consumer::LuaLs)?,
    ];
    let workspace = Workspace::new()?;
    let input = workspace.path.join("input");
    let before = fixture::prepare(&input)?;
    let mut results = Vec::new();
    for executable in executables {
        let package_before = package::snapshot(&executable)?;
        let version = executable.version(&workspace.path)?;
        let (status, raw) = executable.check(&workspace.path)?;
        let diagnostics = report::normalize(executable.kind, &raw, &input)?;
        let unchanged = fixture::snapshot(&input)? == before;
        let assertion = report::assert_behavior(&diagnostics, status);
        let mutations = mutations::run(&executable, &workspace.path)?;
        let controls_pass = mutations
            .iter()
            .all(|m| m["detected"] == true && m["input_unchanged"] == true);
        let package_unchanged = package::snapshot(&executable)? == package_before;
        results.push(json!({"consumer":executable.kind.name(),"version":version,"executable_sha256":executable.digest,
            "exit":status,"diagnostics":diagnostics,"input_unchanged":unchanged,"passed":assertion.is_ok()&&unchanged&&controls_pass&&package_unchanged,"mutations":mutations,"package_unchanged":package_unchanged,"package_files":package_before.0,"package_sha256":package_before.1,
            "failure":assertion.err().map(|e|e.to_string())}));
    }
    fs::write(
        workspace.path.join("consumer-report.json"),
        serde_json::to_vec_pretty(&json!({
            "schema":"wow-annotation-consumer-probe/1","scope":"synthetic-native-signatures/1",
            "input_sha256":before,"results":results,"negative_cases":fixture::NEGATIVES.len(),
            "runtime_correctness":"not_evaluated","full_catalog_compatibility":"not_evaluated",
            "namespace_absence_authority":"unavailable: Ketho namespaces are open tables",
            "configuration_policy":"explicit JSON; no suppression or added globals",
            "semantic_adapter":"not_implemented_by_this_test"
        }))?,
    )?;
    if results.iter().any(|r| r["passed"] != true) {
        eprintln!("{}", serde_json::to_string_pretty(&results)?);
        return Err("annotation semantic consumer assertions failed".into());
    }
    Ok(())
}
#[test]
fn empty_or_positive_only_reports_never_pass() {
    assert!(report::assert_behavior(&[], 0).is_err());
    assert!(report::assert_behavior(&[], 1).is_err());
}
#[test]
fn native_probe_fixture_is_complete_and_readback_is_exact() -> Result<()> {
    let workspace = Workspace::new()?;
    let input = workspace.path.join("input");
    let before = fixture::prepare(&input)?;
    assert_eq!(before, fixture::snapshot(&input)?);
    assert!(before.contains_key("positive.lua"));
    assert_eq!(fixture::NEGATIVES.len(), 9);
    fs::write(input.join("positive.lua"), "changed")?;
    assert_ne!(before, fixture::snapshot(&input)?);
    Ok(())
}

fn expected_diagnostics() -> Vec<serde_json::Value> {
    fixture::NEGATIVES.iter().map(|(name, _, line)| {
        let code=match *name {
            "global.lua"=>"undefined-global", "missing.lua"=>"undefined-field",
            "return.lua"|"multiple.lua"|"structure.lua"|"array.lua"=>"assign-type-mismatch",
            _=>"param-type-mismatch",
        };
        json!({"file":name,"code":code,"severity":2,"range":{"start":{"line":line,"character":0},"end":{"line":line,"character":1}}})
    }).collect()
}
#[test]
fn every_negative_case_is_required_at_the_correct_location() -> Result<()> {
    let expected = expected_diagnostics();
    report::assert_behavior(&expected, 1)?;
    for i in 0..expected.len() {
        let mut missing = expected.clone();
        missing.remove(i);
        assert!(report::assert_behavior(&missing, 1).is_err());
        let mut shifted = expected.clone();
        shifted[i]["range"]["start"]["line"] = json!(99);
        assert!(report::assert_behavior(&shifted, 1).is_err());
    }
    Ok(())
}
#[test]
fn wrong_codes_or_hint_only_diagnostics_do_not_satisfy_negative_checks() {
    for field in ["code", "severity"] {
        let mut diagnostics = expected_diagnostics();
        diagnostics[0][field] = if field == "code" {
            json!("unrelated")
        } else {
            json!(4)
        };
        assert!(report::assert_behavior(&diagnostics, 1).is_err());
    }
}
#[test]
fn errors_in_valid_fixtures_or_generated_libraries_are_not_ignored() {
    for file in ["positive.lua", "api-0000.lua", "values-0001.lua"] {
        let mut diagnostics = expected_diagnostics();
        let mut extra = diagnostics[0].clone();
        extra["file"] = json!(file);
        diagnostics.push(extra);
        assert!(report::assert_behavior(&diagnostics, 1).is_err());
    }
}
#[test]
fn malformed_consumer_schemas_are_not_empty_success() -> Result<()> {
    let workspace = Workspace::new()?;
    let input = workspace.path.join("input");
    fixture::prepare(&input)?;
    for (kind, text) in [
        (Consumer::Emmy, "{}"),
        (Consumer::LuaLs, "[]"),
        (Consumer::Emmy, "[{}]"),
    ] {
        assert!(report::normalize(kind, text.as_bytes(), &input).is_err());
    }
    Ok(())
}
#[test]
fn foreign_paths_reversed_and_out_of_bounds_ranges_reject() -> Result<()> {
    let workspace = Workspace::new()?;
    let input = workspace.path.join("input");
    fixture::prepare(&input)?;
    for range in [
        json!({"start":{"line":99,"character":0},"end":{"line":99,"character":1}}),
        json!({"start":{"line":0,"character":3},"end":{"line":0,"character":0}}),
        json!({"start":{"line":0,"character":0},"end":{"line":0,"character":1000}}),
    ] {
        let raw = json!([{"file":input.join("argument.lua"),"diagnostics":[{"code":"param-type-mismatch","severity":2,"range":range}]}]);
        assert!(report::normalize(Consumer::Emmy, &serde_json::to_vec(&raw)?, &input).is_err());
    }
    fs::write(workspace.path.join("foreign.lua"), "return 1")?;
    let raw = json!([{"file":workspace.path.join("foreign.lua"),"diagnostics":[]}]);
    assert!(report::normalize(Consumer::Emmy, &serde_json::to_vec(&raw)?, &input).is_err());
    Ok(())
}
#[test]
fn duplicate_file_records_are_not_last_writer_wins() -> Result<()> {
    let workspace = Workspace::new()?;
    let input = workspace.path.join("input");
    fixture::prepare(&input)?;
    let row = json!({"file":input.join("argument.lua"),"diagnostics":[]});
    assert!(
        report::normalize(
            Consumer::Emmy,
            &serde_json::to_vec(&vec![row.clone(), row])?,
            &input
        )
        .is_err()
    );
    Ok(())
}

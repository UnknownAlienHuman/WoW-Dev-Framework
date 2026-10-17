use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
    retain: bool,
}
impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let selected = std::env::var_os("WOW_REFERENCE_COMPARISON_PROOF_ROOT");
        let retain = selected.is_some();
        let root = selected.map_or_else(
            || {
                std::env::temp_dir().join(format!(
                    "wow-reference-comparison-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_or(0, |value| value.as_nanos())
                ))
            },
            PathBuf::from,
        );
        fs::create_dir(&root)?;
        fs::create_dir(root.join("before"))?;
        fs::create_dir(root.join("after"))?;
        Ok(Self { root, retain })
    }
    fn config(
        &self,
        role: &str,
        revision: &str,
        body: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        fs::write(self.root.join(role).join("API.lua"), body)?;
        let value = json!({"schema":SOURCE_REFERENCE_INPUT_SCHEMA,
            "profile":{"profile_id":format!("profile:wow:reference-comparison-{role}"),"flavor":"retail","interface":1,"client_version":"1.0.0","client_build":1,"revision":revision,"environment":"Mainline"},
            "native_source":{"root":role,"files":[{"path":"API.lua","content_digest":wow_reference::native::source_digest(body.as_bytes()),"byte_length":body.len()}]}});
        let path = self.root.join(format!("{role}.json"));
        fs::write(&path, serde_json::to_vec(&value)?)?;
        Ok(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.retain {
            for name in [
                "before/API.lua",
                "after/API.lua",
                "before.json",
                "after.json",
                "request.json",
                "expected.json",
                "old-before.json",
                "old-after.json",
            ] {
                let _ = fs::remove_file(self.root.join(name));
            }
            for name in ["before", "after"] {
                let _ = fs::remove_dir(self.root.join(name));
            }
            let _ = fs::remove_dir(&self.root);
        }
    }
}

fn expected(result: &SourceReferenceResult) -> Value {
    json!({"configuration_content_digest":result.configuration_content_digest,"reference_generation":result.reference_generation,"reference_view_digest":result.reference.view.self_digest()})
}

#[test]
fn explicit_native_target_is_captured_in_profile_identity_and_legacy_schema_refuses_it()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let text = r#"APIDocumentation:AddDocumentationTable({Name="Example",Type="System",Namespace="C_Example",Environment="All",Functions={{Name="Read",Type="Function"}}})"#;
    let config = fixture.config("before", "1111111111111111111111111111111111111111", text)?;
    let stop = AtomicBool::new(false);
    let original = read_local_source_reference(&config, &stop)?;
    let mut value: Value = serde_json::from_slice(&fs::read(&config)?)?;
    value["profile"]["reference_target"] =
        json!({"schema":"wow-service/native-reference-target/1","product":"wow","channel":"live"});
    fs::write(&config, serde_json::to_vec(&value)?)?;
    assert!(
        matches!(read_local_source_reference(&config,&stop),Err(error) if error.code()==ServiceErrorCode::InvalidConfiguration)
    );
    value["schema"] = json!(SOURCE_REFERENCE_TARGET_INPUT_SCHEMA);
    fs::write(&config, serde_json::to_vec(&value)?)?;
    let targeted = read_local_source_reference(&config, &stop)?;
    assert_ne!(targeted.profile(), original.profile());
    assert_ne!(
        targeted.reference_generation(),
        original.reference_generation()
    );
    let captured: Value = serde_json::from_slice(&targeted.canonical_bytes(&stop)?)?;
    assert_eq!(
        captured["reference_target"],
        value["profile"]["reference_target"]
    );
    assert_eq!(
        captured["schema"],
        "wow-service/source-reference-target-result/1"
    );
    value["profile"]["reference_target"]["channel"] = json!("ptr");
    fs::write(&config, serde_json::to_vec(&value)?)?;
    let changed = read_local_source_reference(&config, &stop)?;
    assert_ne!(targeted.profile(), changed.profile());
    assert_ne!(
        targeted.reference_generation(),
        changed.reference_generation()
    );
    value["profile"]["reference_target"] = Value::Null;
    fs::write(&config, serde_json::to_vec(&value)?)?;
    assert!(read_local_source_reference(&config, &stop).is_err());
    Ok(())
}

#[test]
fn native_reference_comparison_service_reaches_original_owner_and_guards_inputs()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let text = r#"APIDocumentation:AddDocumentationTable({Name="Example",Type="System",Namespace="C_Example",Environment="All",Functions={{Name="Read",Type="Function",Arguments={{Name="value",Type="number",Nilable=false,Default=0}},Returns={{Name="value",Type="number"}}}}})"#;
    let before_config =
        fixture.config("before", "1111111111111111111111111111111111111111", text)?;
    let after_text = text.replace(
        "Type=\"number\",Nilable=false,Default=0",
        "Type=\"bool\",Nilable=true,Default=false",
    );
    let after_config = fixture.config(
        "after",
        "2222222222222222222222222222222222222222",
        &after_text,
    )?;
    let stop = AtomicBool::new(false);
    let before = read_local_source_reference(&before_config, &stop)?;
    let after = read_local_source_reference(&after_config, &stop)?;
    let old_before = before.canonical_bytes(&stop)?;
    let old_after = after.canonical_bytes(&stop)?;
    let request_file = fixture.root.join("request.json");
    let original = json!({"schema":REQUEST_SCHEMA,"comparison_profile":CALLABLE_COMPARISON_PROFILE,"privacy":"local_metadata_only",
        "before":expected(&before),"after":expected(&after),"keys":["function:C_Example.Read","function:C_Example.Missing"],
        "limits":{"max_keys":16,"max_view_bytes":1048576,"max_steps":1048576},"max_output_bytes":1048576});
    let write_request = |value: &Value| -> Result<(), Box<dyn std::error::Error>> {
        fs::write(&request_file, serde_json::to_vec(value)?)?;
        Ok(())
    };
    write_request(&original)?;
    let result =
        compare_local_source_references(&before_config, &after_config, &request_file, &stop)?;
    assert!(result.is_partial());
    let bytes = result.canonical_bytes(&stop)?;
    let value: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(
        value["comparison"]["observations"][1]["comparison"]["state"],
        "selected_fields_differ"
    );
    assert_eq!(
        value["comparison"]["observations"][1]["comparison"]["differences"][0]["changed"],
        json!(["type_name", "nilable", "default"])
    );
    assert_eq!(
        value["before"]["profile"],
        serde_json::to_value(before.profile())?
    );
    assert_eq!(
        value["after"]["reference_generation"],
        serde_json::to_value(after.reference_generation())?
    );
    assert_eq!(value["comparison"]["accepted_lineage_count"], 0);
    assert_eq!(value["comparison"]["lineage_negative_authority"], false);
    let mut shuffled = original.clone();
    shuffled["keys"] = json!(["function:C_Example.Missing", "function:C_Example.Read"]);
    write_request(&shuffled)?;
    assert_eq!(
        compare_local_source_references(&before_config, &after_config, &request_file, &stop)?
            .canonical_bytes(&stop)?,
        bytes
    );
    for field in [
        "configuration_content_digest",
        "reference_generation",
        "reference_view_digest",
    ] {
        let mut wrong = original.clone();
        wrong["before"][field] = original["after"][field].clone();
        write_request(&wrong)?;
        assert!(
            matches!(compare_local_source_references(&before_config,&after_config,&request_file,&stop),Err(error) if error.code()==ServiceErrorCode::ExactGenerationUnavailable)
        );
    }
    for (field, value) in [
        ("schema", json!(null)),
        ("privacy", json!(null)),
        ("privacy", json!({"local_metadata_only": null})),
        ("unexpected", json!(true)),
        ("comparison_profile", json!("current")),
    ] {
        let mut wrong = original.clone();
        wrong[field] = value;
        write_request(&wrong)?;
        assert!(
            matches!(compare_local_source_references(&before_config,&after_config,&request_file,&stop),Err(error) if error.code()==ServiceErrorCode::InvalidConfiguration)
        );
    }
    let mut wrong = original.clone();
    wrong["max_output_bytes"] = json!(4);
    write_request(&wrong)?;
    assert!(
        matches!(compare_local_source_references(&before_config,&after_config,&request_file,&stop),Err(error) if error.code()==ServiceErrorCode::BudgetExceeded)
    );
    let mut wrong = original.clone();
    wrong["limits"]["max_steps"] = json!(1);
    write_request(&wrong)?;
    assert!(
        matches!(compare_local_source_references(&before_config,&after_config,&request_file,&stop),Err(error) if error.code()==ServiceErrorCode::BudgetExceeded)
    );
    write_request(&original)?;
    assert!(
        matches!(compare_local_source_references(&before_config,&after_config,&request_file,&AtomicBool::new(true)),Err(error) if error.code()==ServiceErrorCode::Cancelled)
    );
    let mut altered: Value = serde_json::from_slice(&fs::read(&after_config)?)?;
    altered["profile"]["environment"] = json!("Classic");
    fs::write(&after_config, serde_json::to_vec(&altered)?)?;
    let other = read_local_source_reference(&after_config, &stop)?;
    let mut incompatible = original.clone();
    incompatible["after"] = expected(&other);
    write_request(&incompatible)?;
    assert!(
        matches!(compare_local_source_references(&before_config,&after_config,&request_file,&stop),Err(error) if error.code()==ServiceErrorCode::InvalidConfiguration)
    );
    altered["profile"]["environment"] = json!("Mainline");
    fs::write(&after_config, serde_json::to_vec(&altered)?)?;
    assert_eq!(
        read_local_source_reference(&before_config, &stop)?.canonical_bytes(&stop)?,
        old_before
    );
    assert_eq!(
        read_local_source_reference(&after_config, &stop)?.canonical_bytes(&stop)?,
        old_after
    );
    fs::write(fixture.root.join("after/API.lua"), "changed input")?;
    write_request(&original)?;
    assert!(
        compare_local_source_references(&before_config, &after_config, &request_file, &stop)
            .is_err()
    );
    fs::write(fixture.root.join("after/API.lua"), &after_text)?;
    // Retain exact real operation inputs/results only for explicitly requested CLI proof.
    if fixture.retain {
        fs::write(fixture.root.join("expected.json"), &bytes)?;
        fs::write(fixture.root.join("old-before.json"), old_before)?;
        fs::write(fixture.root.join("old-after.json"), old_after)?;
    }
    Ok(())
}

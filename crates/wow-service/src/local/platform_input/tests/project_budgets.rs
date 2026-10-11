use super::*;
use wow_project::{ProjectBudgetPolicy, ProjectErrorCode, ProjectPublisher, replay::ProjectReplay};

fn selected(config: &mut Value, policy: ProjectBudgetPolicy) -> TestResult {
    config["schema"] = json!(LOCAL_PLATFORM_BUDGET_SCHEMA);
    config["retained_package_limits"] =
        serde_json::to_value(PlatformPackageRetentionLimits::new(2, 1024 * 1024)?)?;
    config["project_budgets"] = serde_json::to_value(policy)?;
    Ok(())
}

#[test]
fn explicit_project_budgets_reject_invalid_decode_and_schema_selection() -> TestResult {
    let policy = ProjectBudgetPolicy::new(8, 1024 * 1024, 1024 * 1024, 8, 512, 16, 64 * 1024)?;
    let wire = serde_json::to_value(policy)?;
    assert_eq!(
        serde_json::from_value::<ProjectBudgetPolicy>(wire.clone())?,
        policy
    );
    for field in wire.as_object().ok_or("budget fields")?.keys() {
        let mut zero = wire.clone();
        zero[field] = json!(0);
        assert!(serde_json::from_value::<ProjectBudgetPolicy>(zero).is_err());
        let mut missing = wire.clone();
        missing
            .as_object_mut()
            .ok_or("budget fields")?
            .remove(field);
        assert!(serde_json::from_value::<ProjectBudgetPolicy>(missing).is_err());
    }
    for (field, value) in [
        ("max_single_file_bytes", json!(1024 * 1024 + 1)),
        ("max_files", json!(-1)),
        ("max_files", json!(1.5)),
        ("max_files", Value::Null),
        ("unknown", json!(1)),
    ] {
        let mut bad = wire.clone();
        bad[field] = value;
        assert!(serde_json::from_value::<ProjectBudgetPolicy>(bad).is_err());
    }
    let duplicate = format!(
        "{{\"max_files\":8,{}",
        serde_json::to_string(&wire)?
            .strip_prefix('{')
            .ok_or("budget object")?
    );
    assert!(serde_json::from_str::<ProjectBudgetPolicy>(&duplicate).is_err());

    let (root, mut config) = fixture()?;
    selected(&mut config, policy)?;
    let path = root.0.join("project.json");
    let stop = AtomicBool::new(false);
    for change in 0..5 {
        let mut bad = config.clone();
        match change {
            0 => bad["schema"] = json!(LOCAL_PLATFORM_SCHEMA),
            1 => {
                bad.as_object_mut()
                    .ok_or("config")?
                    .remove("project_budgets");
            }
            2 => bad["project_budgets"] = Value::Null,
            3 => {
                bad.as_object_mut()
                    .ok_or("config")?
                    .remove("retained_package_limits");
            }
            _ => bad["project_budgets"]["max_output_bytes"] = json!(0),
        }
        write_json(&root, "project.json", &bad)?;
        assert_eq!(
            LocalProjectInput::from_config_path(&path, &stop)
                .err()
                .ok_or("invalid budget selection admitted")?
                .code(),
            crate::ServiceErrorCode::InvalidConfiguration
        );
    }
    write_json(&root, "project.json", &config)?;
    assert_eq!(
        LocalProjectInput::from_config_path(&path, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled input admitted")?
            .code(),
        crate::ServiceErrorCode::Cancelled
    );
    Ok(())
}

#[test]
fn explicit_project_budgets_bind_native_admission_publication_and_cold_replay() -> TestResult {
    let (root, mut config) = fixture()?;
    let path = root.0.join("project.json");
    let stop = AtomicBool::new(false);
    let ordinary = LocalProjectInput::from_config_path(&path, &stop)?;
    let fixed = ordinary.bundle.configuration().budget_policy();
    assert_eq!(fixed.max_output_bytes(), 16 * 1024 * 1024);
    selected(&mut config, fixed)?;
    config["schema"] = json!(LOCAL_PLATFORM_SCHEMA);
    config
        .as_object_mut()
        .ok_or("config")?
        .remove("project_budgets");
    write_json(&root, "project.json", &config)?;
    let retained_v1 = LocalProjectInput::from_config_path(&path, &stop)?;
    selected(&mut config, fixed)?;
    write_json(&root, "project.json", &config)?;
    let retained_v2 = LocalProjectInput::from_config_path(&path, &stop)?;
    // A transport schema is not a semantic generation input: identical selected
    // owner policies must retain the existing configuration recipe.
    assert_eq!(retained_v1.bundle, retained_v2.bundle);

    let policy = ProjectBudgetPolicy::new(8, 1024 * 1024, 1024 * 1024, 8, 512, 16, 64 * 1024)?;
    selected(&mut config, policy)?;
    write_json(&root, "project.json", &config)?;
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    assert_eq!(input.bundle.configuration().budget_policy(), policy);
    assert_ne!(
        input.bundle.configuration().configuration_digest(),
        retained_v2.bundle.configuration().configuration_digest()
    );
    let mut publisher = ProjectPublisher::with_function_call_facts();
    let snapshot = publisher.publish_initial_cancellable(input.bundle, &stop)?;
    let binding = snapshot.analyzer_binding();
    let output_bytes = u64::try_from(
        serde_json::to_vec(&(
            binding.syntax_report(),
            binding.member_call_report(),
            binding.local_flow_report(),
            binding.xml_lua_analysis(),
        ))?
        .len()
            + binding
                .package_xml_bindings()
                .ok_or("native XML bindings")?
                .serialized_byte_length(),
    )?;
    assert!(output_bytes > 1 && output_bytes < policy.max_output_bytes());

    // Prove the selected policy reaches the final owner gate: admit the exact
    // encoded population, then refuse one byte below it without publication.
    for (limit, admits) in [(output_bytes, true), (output_bytes - 1, false)] {
        config["project_budgets"]["max_output_bytes"] = json!(limit);
        write_json(&root, "project.json", &config)?;
        let input = LocalProjectInput::from_config_path(&path, &stop)?;
        let mut bounded = ProjectPublisher::with_function_call_facts();
        let result = bounded.publish_initial_cancellable(input.bundle, &stop);
        if admits {
            assert!(result.is_ok(), "exact output budget refused: {result:?}");
        } else {
            assert_eq!(
                result.err().ok_or("output budget ignored")?.code(),
                ProjectErrorCode::UpdateBudgetExceeded
            );
            assert!(bounded.current_snapshot().is_none());
        }
    }
    selected(&mut config, policy)?;
    config["project_budgets"]["max_single_file_bytes"] = json!(1);
    write_json(&root, "project.json", &config)?;
    assert!(LocalProjectInput::from_config_path(&path, &stop).is_err());

    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let wire = serde_json::to_value(&replay)?;
    assert_eq!(wire["schema"], "wow-project/native-project-replay/11");
    assert_eq!(
        wire["configuration"]["budgets"],
        serde_json::to_value(policy)?
    );
    std::fs::remove_file(root.0.join("UI/main.lua"))?;
    std::fs::remove_file(root.0.join("UI/frames.xml"))?;
    let restored = ProjectReplay::from_json(&serde_json::to_vec(&wire)?, &stop)?.hydrate(&stop)?;
    assert_eq!(restored.configuration().budget_policy(), policy);
    assert_eq!(restored.snapshot_id(), snapshot.snapshot_id());
    assert_eq!(
        restored.snapshot().analyzer_binding(),
        snapshot.analyzer_binding()
    );
    let mut changed = wire;
    changed["configuration"]["budgets"]["max_output_bytes"] = json!(policy.max_output_bytes() + 1);
    assert!(
        ProjectReplay::from_json(&serde_json::to_vec(&changed)?, &stop)?
            .hydrate(&stop)
            .is_err()
    );
    Ok(())
}

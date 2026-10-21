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
fn graph_metadata_budget_is_strict_and_requires_retained_graph_selection() -> TestResult {
    use wow_project::{
        ProjectConfigurationBuilder,
        graph::{SourceGraphBudget, SourceGraphXmlCountBudget},
    };
    let ordinary = ProjectBudgetPolicy::new(8, 1024 * 1024, 1024 * 1024, 8, 512, 16, 64 * 1024)?;
    let absent = serde_json::to_value(ordinary)?;
    assert!(absent.get("source_graph").is_none());
    assert_eq!(
        serde_json::from_value::<ProjectBudgetPolicy>(absent.clone())?,
        ordinary
    );
    let budget = SourceGraphBudget::new(32 * 1024 * 1024)?;
    assert_eq!(
        serde_json::to_string(&budget)?,
        "{\"max_metadata_bytes\":33554432}"
    );
    assert_eq!(budget.xml_counts(), None);
    let selected_policy = ordinary.with_source_graph_budget(budget);
    assert_eq!(
        serde_json::from_value::<ProjectBudgetPolicy>(serde_json::to_value(selected_policy)?)?,
        selected_policy
    );
    assert!(SourceGraphBudget::new(0).is_err());
    let xml = SourceGraphXmlCountBudget::new(32768, 8192)?;
    let xml_budget = budget.with_xml_counts(xml);
    assert_eq!(
        serde_json::from_value::<SourceGraphBudget>(serde_json::to_value(xml_budget)?)?,
        xml_budget
    );
    assert!(SourceGraphXmlCountBudget::new(0, 8192).is_err());
    assert!(SourceGraphXmlCountBudget::new(32768, 0).is_err());
    for value in [
        Value::Null,
        json!({}),
        json!({"max_declarations":32768}),
        json!({"max_inheritance_references":8192}),
        json!({"max_declarations":0,"max_inheritance_references":8192}),
        json!({"max_declarations":32768,"max_inheritance_references":0}),
        json!({"max_declarations":-1,"max_inheritance_references":8192}),
        json!({"max_declarations":1.5,"max_inheritance_references":8192}),
        json!({"max_declarations":32768,"max_inheritance_references":8192,"unknown":1}),
    ] {
        let mut changed = serde_json::to_value(budget)?;
        changed["xml_counts"] = value;
        assert!(serde_json::from_value::<SourceGraphBudget>(changed).is_err());
    }
    for wire in [
        "{\"max_declarations\":1,\"max_declarations\":2,\"max_inheritance_references\":1}",
        "{\"max_declarations\":1,\"max_inheritance_references\":1,\"max_inheritance_references\":2}",
    ] {
        assert!(serde_json::from_str::<SourceGraphXmlCountBudget>(wire).is_err());
    }
    assert!(serde_json::from_str::<SourceGraphBudget>(
        "{\"max_metadata_bytes\":1,\"xml_counts\":{\"max_declarations\":1,\"max_inheritance_references\":1},\"xml_counts\":{\"max_declarations\":2,\"max_inheritance_references\":1}}"
    ).is_err());
    for value in [
        Value::Null,
        json!({}),
        json!({"max_metadata_bytes":0}),
        json!({"max_metadata_bytes":-1}),
        json!({"max_metadata_bytes":1.5}),
        json!({"max_metadata_bytes":1,"unknown":1}),
    ] {
        let mut changed = absent.clone();
        changed["source_graph"] = value;
        assert!(serde_json::from_value::<ProjectBudgetPolicy>(changed).is_err());
    }
    assert!(
        serde_json::from_str::<SourceGraphBudget>(
            "{\"max_metadata_bytes\":1,\"max_metadata_bytes\":2}"
        )
        .is_err()
    );
    let duplicate = format!(
        "{{\"source_graph\":{{\"max_metadata_bytes\":1}},{}",
        serde_json::to_string(&selected_policy)?
            .strip_prefix('{')
            .ok_or("policy object")?
    );
    assert!(serde_json::from_str::<ProjectBudgetPolicy>(&duplicate).is_err());

    let selected_policy = selected_policy.with_source_graph_budget(xml_budget);
    let (root, mut config) = fixture()?;
    let path = root.0.join("project.json");
    let stop = AtomicBool::new(false);
    let unretained = LocalProjectInput::from_config_path(&path, &stop)?;
    selected(&mut config, selected_policy)?;
    write_json(&root, "project.json", &config)?;
    let retained = LocalProjectInput::from_config_path(&path, &stop)?;
    let declarations: Vec<ProjectPackageInput> =
        serde_json::from_value(config["packages"].clone())?;
    for previous in [
        unretained.bundle.configuration(),
        retained.bundle.configuration(),
    ] {
        let owner = previous.platform_packages().ok_or("platform")?;
        let packages = Arc::new(match owner.retention_limits() {
            Some(limits) => owner.source().specialize_packages_with_retention(
                &declarations,
                None,
                limits,
                &stop,
            )?,
            None => owner
                .source()
                .specialize_packages(&declarations, None, &stop)?,
        });
        // An ordinary native platform input lacks retention; the retained one
        // below lacks graph selection. Neither may consume this component.
        let refused = ProjectConfigurationBuilder::new(
            previous.project_id().clone(),
            previous.project_kind(),
            previous.selected_profile().clone(),
            previous.reference_generation(),
            previous.analyzer_binding().clone(),
        )
        .workspace_id(previous.workspace_id().clone())
        .source_origin_id(previous.source_origin_id().clone())
        .logical_root(previous.logical_root().as_str())
        .capability_policy(previous.capability_policy().clone())
        .budget_policy(selected_policy)
        .platform_packages(packages)?
        .with_package_xml_bindings(wow_project::PackageXmlBindingProfile::SameSessionV1)
        .build();
        assert_eq!(
            refused
                .err()
                .ok_or("unsupported component admitted")?
                .code(),
            ProjectErrorCode::InvalidBudgetPolicy
        );
    }
    Ok(())
}

#[test]
fn graph_diagnostics_keep_results_exact_and_refusal_conservative() -> TestResult {
    use crate::graph::{
        GraphBuildRequest, GraphReadStatus, SourceGraphDiagnostics, execute_graph_build,
        execute_graph_build_diagnosed, source_graph_diagnostics_bytes,
    };
    use wow_project::graph::SourceGraphBudget;
    let (root, mut config) = fixture()?;
    let path = root.0.join("project.json");
    let stop = AtomicBool::new(false);
    let ordinary = LocalProjectInput::from_config_path(&path, &stop)?;
    let policy = ordinary.bundle.configuration().budget_policy();
    selected(
        &mut config,
        policy.with_source_graph_budget(SourceGraphBudget::new(32 * 1024 * 1024)?),
    )?;
    write_json(&root, "project.json", &config)?;
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let expected = execute_graph_build(input, &request, &stop)?;
    assert_eq!(expected.status(), GraphReadStatus::Partial);
    let mut diagnostics = SourceGraphDiagnostics::enabled();
    let actual = execute_graph_build_diagnosed(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &request,
        &stop,
        &mut diagnostics,
    )?;
    assert_eq!(actual.canonical_bytes()?, expected.canonical_bytes()?);
    let observed: Value = serde_json::from_slice(&source_graph_diagnostics_bytes(&diagnostics)?)?;
    let callable_estimates = [
        observed["input"]["function_total_metadata_estimate_bytes"]
            .as_u64()
            .ok_or("function total missing")?,
        observed["input"]["call_base_total_metadata_estimate_bytes"]
            .as_u64()
            .ok_or("call base total missing")?,
        observed["input"]["callable_base_total_metadata_estimate_bytes"]
            .as_u64()
            .ok_or("combined callable base total missing")?,
    ];
    assert_eq!(
        callable_estimates[0]
            .checked_add(callable_estimates[1])
            .ok_or("callable sum overflow")?,
        callable_estimates[2]
    );
    assert_eq!(
        observed["input"]["callable_charge_counting_method"],
        "conservative_estimate"
    );
    for name in [
        "collector_encoding",
        "direct_preparation",
        "inventory_stage",
        "toc_load_stage",
        "analyzer_structure_stage",
        "xml_structure_stage",
        "direct_support",
    ] {
        let phase = observed["phases"]
            .as_array()
            .ok_or("phases")?
            .iter()
            .find(|phase| phase["phase"] == name)
            .ok_or("phase missing")?;
        assert_eq!(phase["outcome"], "completed", "{name}");
        assert!(phase["metadata_refusal"].is_null());
        assert!(phase["elapsed_microseconds"].is_number());
    }
    selected(
        &mut config,
        policy.with_source_graph_budget(SourceGraphBudget::new(1)?),
    )?;
    write_json(&root, "project.json", &config)?;
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let refused = execute_graph_build_diagnosed(input, &request, &stop, &mut diagnostics)?;
    assert_eq!(refused.status(), GraphReadStatus::Failed);
    assert!(refused.snapshot_bytes()?.is_none());
    let semantic = refused.canonical_bytes()?;
    assert!(!std::str::from_utf8(&semantic)?.contains("elapsed_microseconds"));
    let observed: Value = serde_json::from_slice(&source_graph_diagnostics_bytes(&diagnostics)?)?;
    assert!(observed["input"]["functions"].is_number());
    assert!(!std::str::from_utf8(&semantic)?.contains("function_total_metadata_estimate_bytes"));
    for (field, total) in [
        "function_total_metadata_estimate_bytes",
        "call_base_total_metadata_estimate_bytes",
        "callable_base_total_metadata_estimate_bytes",
    ]
    .into_iter()
    .zip(callable_estimates)
    {
        assert_eq!(observed["input"][field].as_u64(), Some(total));
    }
    let phases = observed["phases"].as_array().ok_or("phases")?;
    let first = &phases[0];
    assert_eq!(first["outcome"], "refused");
    assert_eq!(first["metadata_refusal"]["configured_limit_bytes"], 1);
    assert_eq!(
        first["metadata_refusal"]["attempted_bytes"],
        first["counter_after_bytes"]
    );
    assert!(
        phases
            .iter()
            .skip(1)
            .all(|phase| phase["outcome"] == "not_evaluated")
    );
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    stop.store(true, std::sync::atomic::Ordering::Release);
    let cancelled = execute_graph_build_diagnosed(input, &request, &stop, &mut diagnostics)?;
    assert_eq!(cancelled.status(), GraphReadStatus::Cancelled);
    let cancelled_observation: Value =
        serde_json::from_slice(&source_graph_diagnostics_bytes(&diagnostics)?)?;
    assert!(cancelled_observation["input"].is_null());
    assert!(
        cancelled_observation["phases"]
            .as_array()
            .ok_or("phases")?
            .iter()
            .all(|phase| phase["outcome"] == "not_evaluated"
                && phase["counter_after_bytes"].is_null())
    );
    Ok(())
}

#[test]
fn native_publication_refusal_preserves_runtime_cause_outside_graph_wire() -> TestResult {
    let (root, mut config) = fixture()?;
    let path = root.0.join("project.json");
    let stop = AtomicBool::new(false);
    let ordinary = LocalProjectInput::from_config_path(&path, &stop)?;
    let budget = wow_emmy::function_calls::FunctionCallBudget::new(1, 16, 16, 16, 16, 64 * 1024)?;
    selected(
        &mut config,
        ordinary
            .bundle
            .configuration()
            .budget_policy()
            .with_function_call_budget(budget),
    )?;
    write_json(&root, "project.json", &config)?;
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let request = crate::graph::GraphBuildRequest::new(
        "fixture-platform-transport".into(),
        "current".into(),
    )?
    .with_input_profile(&input);
    let result = crate::graph::execute_graph_build(input, &request, &stop)?;
    assert_eq!(result.status(), crate::graph::GraphReadStatus::Failed);
    assert!(result.snapshot_bytes()?.is_none());
    let message = result.failure_message().ok_or("owner cause discarded")?;
    assert!(
        message.contains("SourceBudgetExceeded")
            && message.contains("function-call AST visits budget exceeded")
            && message.contains("observed 2, limit 1"),
        "actual retained native refusal: {message}"
    );
    let bytes = result.canonical_bytes()?;
    let wire: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(wire["failure"], "budget_exceeded");
    assert_eq!(wire["schema"], "wow-service/graph-build-result/20");
    assert!(wire.get("failure_error").is_none());
    assert!(!std::str::from_utf8(&bytes)?.contains(message));
    let cancelled = result.into_cancelled()?;
    assert!(cancelled.failure_message().is_none());
    assert_eq!(cancelled.status(), crate::graph::GraphReadStatus::Cancelled);
    assert_eq!(serde_json::to_value(cancelled)?["failure"], "cancelled");
    Ok(())
}

#[test]
fn selected_sidecar_budget_binds_native_report_and_replay_without_downgrade() -> TestResult {
    use crate::live_project::{
        LiveProjectPublishRequest, LiveProjectStore, PlatformStoreSelection,
        publish_input_in_namespace,
    };
    use wow_emmy::function_calls::{FUNCTION_CALL_BUDGET_PROFILE, FunctionCallBudget};
    use wow_project::ProjectConfigurationBuilder;
    use wow_store::project::ReadSelector;

    let (root, mut config) = fixture()?;
    let path = root.0.join("project.json");
    let stop = AtomicBool::new(false);
    let ordinary = LocalProjectInput::from_config_path(&path, &stop)?;
    let budgets = ordinary.bundle.configuration().budget_policy();
    let sidecar = FunctionCallBudget::new(64, 16, 16, 16, 32, 64 * 1024)?;
    let selected_budgets = budgets.with_function_call_budget(sidecar);
    assert_eq!(
        serde_json::from_value::<ProjectBudgetPolicy>(serde_json::to_value(selected_budgets)?)?,
        selected_budgets
    );
    let mut null = serde_json::to_value(budgets)?;
    null["function_calls"] = Value::Null;
    assert!(serde_json::from_value::<ProjectBudgetPolicy>(null).is_err());
    // Public configuration must reject the extension on an ordinary platform
    // owner, independently of the service transport's own retention check.
    let previous = ordinary.bundle.configuration();
    let declarations: Vec<ProjectPackageInput> =
        serde_json::from_value(config["packages"].clone())?;
    let packages = Arc::new(
        previous
            .platform_packages()
            .ok_or("platform owner")?
            .source()
            .specialize_packages(&declarations, None, &stop)?,
    );
    let invalid = ProjectConfigurationBuilder::new(
        previous.project_id().clone(),
        previous.project_kind(),
        previous.selected_profile().clone(),
        previous.reference_generation(),
        previous.analyzer_binding().clone(),
    )
    .workspace_id(previous.workspace_id().clone())
    .source_origin_id(previous.source_origin_id().clone())
    .logical_root(previous.logical_root().as_str())
    .capability_policy(previous.capability_policy().clone())
    .budget_policy(selected_budgets)
    .platform_packages(packages)?
    .build();
    assert_eq!(
        invalid.err().ok_or("unretained budget admitted")?.code(),
        ProjectErrorCode::InvalidBudgetPolicy
    );

    selected(&mut config, selected_budgets)?;
    write_json(&root, "project.json", &config)?;
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let store_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let selection = PlatformStoreSelection::new(
        input.bundle.configuration().project_id().clone(),
        input
            .bundle
            .configuration()
            .platform_packages()
            .ok_or("platform owner")?
            .source()
            .profile()
            .profile_id()
            .clone(),
    )?;
    let request = crate::graph::GraphBuildRequest::new(
        "fixture-platform-transport".into(),
        "current".into(),
    )?
    .with_input_profile(&input);
    // The service must own and retain the same data, rather than reopen files.
    std::fs::remove_file(root.0.join("UI/main.lua"))?;
    std::fs::remove_file(root.0.join("UI/frames.xml"))?;
    let (publication, _) = crate::graph::live_publication(input, &request, &stop)?;
    let (records, bindings) = publication.into_parts();
    let replay: ProjectReplay = records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("replay record")?
        .decode()?;
    let wire = serde_json::to_value(&replay)?;
    assert_eq!(
        wire["configuration"]["budgets"]["function_calls"],
        serde_json::to_value(sidecar)?
    );
    let reopened = ProjectReplay::from_json(&serde_json::to_vec(&wire)?, &stop)?.hydrate(&stop)?;
    assert_eq!(reopened.configuration().budget_policy(), selected_budgets);
    assert_eq!(
        Some(reopened.snapshot_id()),
        bindings.get("project_snapshot_id").map(String::as_str)
    );
    assert_eq!(
        Some(reopened.analyzer_snapshot_id()),
        bindings.get("analyzer_snapshot_id").map(String::as_str)
    );
    let report = reopened
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("sidecar report")?;
    assert_eq!(report.budget(), Some(sidecar));
    assert_eq!(
        serde_json::to_value(report)?["profile"],
        FUNCTION_CALL_BUDGET_PROFILE
    );
    report.validate()?;
    let second = replay.hydrate(&stop)?;
    assert_eq!(
        second.snapshot().analyzer_binding(),
        reopened.snapshot().analyzer_binding()
    );

    let store_root = root.0.join("selected-store");
    let publish = LiveProjectPublishRequest::new("fixture:budget-current", "absent", true, true)?;
    publish_input_in_namespace(
        store_input,
        &request,
        &store_root,
        &selection,
        &publish,
        &stop,
    )?;
    let store = LiveProjectStore::open_in_namespace(&store_root, &selection)?;
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), reopened.snapshot_id());
    assert_eq!(
        read.project().snapshot().analyzer_binding(),
        reopened.snapshot().analyzer_binding()
    );
    assert_eq!(
        read.project().configuration().budget_policy(),
        selected_budgets
    );
    drop(read);
    drop(store);

    for mutation in 0..5 {
        let mut changed = wire.clone();
        match mutation {
            0 => changed["obsolete_schema"] = json!("obsolete-project-replay-schema"),
            1 => {
                changed["configuration"]["budgets"]
                    .as_object_mut()
                    .ok_or("budgets")?
                    .remove("function_calls");
            }
            2 => {
                changed["configuration"]["budgets"]["function_calls"]["max_ast_visits"] = json!(65)
            }
            3 => changed["configuration"]["budgets"]["function_calls"]["max_calls"] = json!(0),
            _ => changed["configuration"]["budgets"]["function_calls"] = Value::Null,
        }
        let rejected = ProjectReplay::from_json(&serde_json::to_vec(&changed)?, &stop)
            .and_then(|replay| replay.hydrate(&stop));
        assert!(rejected.is_err(), "replay mutation admitted: {mutation}");
    }
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

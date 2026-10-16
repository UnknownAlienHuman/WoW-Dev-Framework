//! One real retained A/B pair, ordered roles and checked native closure.
use super::*;
use serde_json::Value;
use std::sync::atomic::Ordering;
use wow_project::replay::publication::RetainedGenerationPair;

#[test]
fn exact_generation_pair_holds_roles_and_closes_every_path() -> TestResult {
    let stop = AtomicBool::new(false);
    let exported = std::env::var_os("WOW_NATIVE_PAIR_EXPORT");
    let location = exported
        .clone()
        .map(std::path::PathBuf::from)
        .unwrap_or(root("generation-pair")?);
    if !location.is_absolute() || location.as_os_str().len() > 32768 {
        return Err("fresh absolute fixture root required".into());
    }
    std::fs::create_dir(&location)?;
    let store_root = location.join("store");
    let (first, first_graph) = owners("local value = External()\nreturn value\n")?;
    let (second, second_graph) = owners("local value = External() + 1\nreturn value\n")?;
    let (third, third_graph) = owners("local value = External() + 2\nreturn value\n")?;
    let mut store =
        LiveProjectStore::create(&store_root, first_graph.snapshot().universe().as_str())?;
    let a = store.publish(&first, &first_graph, "pair-a", None, &stop)?;
    let guard = store.current()?.ok_or("first Current")?.record_id;
    let b = store.publish(&second, &second_graph, "pair-b", Some(guard), &stop)?;
    store.close()?;
    drop(first);
    drop(first_graph);
    drop(second);
    drop(second_graph);
    let request = ProjectGenerationPairRequest::new(
        a.generation_id.as_str(),
        b.generation_id.as_str(),
        PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES,
    )?;
    let result = read_live_project_generation_pair(&store_root, &request, &stop)?;
    let data: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(data["resource_closure"], "closed");
    assert_eq!(data["pair"]["status"], "valid_partial");
    assert_eq!(
        data["pair"]["before"]["store_generation_id"],
        a.generation_id.as_str()
    );
    assert_eq!(
        data["pair"]["after"]["store_generation_id"],
        b.generation_id.as_str()
    );
    assert_ne!(
        data["pair"]["before"]["project_snapshot_id"],
        data["pair"]["after"]["project_snapshot_id"]
    );
    assert_eq!(data["pair"]["lineage_evaluation"], "not_evaluated");
    assert_eq!(data["pair"]["negative_authority"], false);
    assert!(
        !String::from_utf8_lossy(result.canonical_bytes())
            .contains(location.to_str().ok_or("fixture path")?)
    );
    let reverse = ProjectGenerationPairRequest::new(
        b.generation_id.as_str(),
        a.generation_id.as_str(),
        PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES,
    )?;
    let reversed: Value = serde_json::from_slice(
        read_live_project_generation_pair(&store_root, &reverse, &stop)?.canonical_bytes(),
    )?;
    assert_ne!(data["pair"]["pair_id"], reversed["pair"]["pair_id"]);
    assert_eq!(reversed["pair"]["before"], data["pair"]["after"]);
    let same = ProjectGenerationPairRequest::new(
        a.generation_id.as_str(),
        a.generation_id.as_str(),
        PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES,
    )?;
    let same: Value = serde_json::from_slice(
        read_live_project_generation_pair(&store_root, &same, &stop)?.canonical_bytes(),
    )?;
    assert_eq!(same["pair"]["same_generation"], true);
    assert_eq!(same["pair"]["status"], "valid_partial");

    let mut store = LiveProjectStore::open(&store_root)?;
    let guard = store.current()?.ok_or("second Current")?.record_id;
    store.publish(&third, &third_graph, "pair-c", Some(guard), &stop)?;
    store.close()?;
    assert_eq!(
        read_live_project_generation_pair(&store_root, &request, &stop)?.canonical_bytes(),
        result.canonical_bytes()
    );
    let tiny =
        ProjectGenerationPairRequest::new(a.generation_id.as_str(), b.generation_id.as_str(), 1)?;
    assert_eq!(
        read_live_project_generation_pair(&store_root, &tiny, &stop)
            .err()
            .map(|error| error.code()),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    let missing = ProjectGenerationPairRequest::new(
        a.generation_id.as_str(),
        &format!("project-store-generation:sha256:{}", "0".repeat(64)),
        PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES,
    )?;
    assert!(read_live_project_generation_pair(&store_root, &missing, &stop).is_err());
    assert_eq!(
        read_live_project_generation_pair(&store_root, &request, &AtomicBool::new(true))
            .err()
            .map(|error| error.code()),
        Some(ServiceErrorCode::Cancelled)
    );
    let late =
        super::super::generation_pair::with_pair(&store_root, &request, &stop, |before, after| {
            assert_eq!(before.store_generation_id(), &a.generation_id);
            assert_eq!(after.store_generation_id(), &b.generation_id);
            stop.store(true, Ordering::Release);
            Ok(())
        });
    assert_eq!(
        late.err().map(|error| error.code()),
        Some(ServiceErrorCode::Cancelled)
    );
    stop.store(false, Ordering::Release);
    let foreign_root = location.join("foreign");
    let mut foreign =
        LiveProjectStore::create(&foreign_root, third_graph.snapshot().universe().as_str())?;
    let other = foreign.publish(&third, &third_graph, "foreign-pair", None, &stop)?;
    let store = LiveProjectStore::open(&store_root)?;
    let before = store.read(&ReadSelector::Exact(a.generation_id.clone()), &stop)?;
    let after = foreign.read(&ReadSelector::Exact(other.generation_id), &stop)?;
    assert_eq!(
        RetainedGenerationPair::bind(
            before.pair(),
            after.pair(),
            before.read.custody(),
            after.read.custody(),
            &stop
        )
        .err()
        .map(|error| error.code()),
        Some(wow_project::ProjectErrorCode::IncompatibleGenerationPair)
    );
    after.close()?;
    before.close()?;
    foreign.close()?;
    store.close()?;
    // Equal logical IDs do not attest a lease after its real snapshot closes.
    let store = LiveProjectStore::open(&store_root)?;
    let raw = store
        .store
        .read(&ReadSelector::Exact(a.generation_id.clone()), &stop)?;
    let detached = wow_project::replay::publication::AcquiredProjectPair::read(&raw, &stop)?;
    raw.close()?;
    let fresh = store
        .store
        .read(&ReadSelector::Exact(a.generation_id.clone()), &stop)?;
    assert_eq!(
        RetainedGenerationPair::bind(
            &detached,
            &detached,
            fresh.custody(),
            fresh.custody(),
            &stop
        )
        .err()
        .map(|e| e.code()),
        Some(wow_project::ProjectErrorCode::IncompatibleGenerationPair)
    );
    fresh.close()?;
    store.close()?;
    drop(detached);
    let policy = ProjectBudgetPolicy::fixture_e0()?;
    let changed_policy = ProjectBudgetPolicy::new(
        policy.max_files() + 1,
        policy.max_total_source_bytes(),
        policy.max_single_file_bytes(),
        policy.max_update_operations(),
        policy.max_analyzer_facts(),
        policy.max_generic_findings(),
        policy.max_output_bytes(),
    )?;
    let changed_bundle = input_bundle_with_budget(
        vec![ProjectInputFile::new(
            "main.lua",
            "return External() + 4\n",
        )?],
        None,
        None,
        changed_policy,
    )?;
    let (changed, changed_graph) = owners_from_bundle(changed_bundle)?;
    let mut store = LiveProjectStore::open(&store_root)?;
    let guard = store.current()?.ok_or("third Current")?.record_id;
    let d = store.publish(
        &changed,
        &changed_graph,
        "pair-d-changed-configuration",
        Some(guard),
        &stop,
    )?;
    store.close()?;
    let changed_request = ProjectGenerationPairRequest::new(
        a.generation_id.as_str(),
        d.generation_id.as_str(),
        PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES,
    )?;
    let changed_data: Value = serde_json::from_slice(
        read_live_project_generation_pair(&store_root, &changed_request, &stop)?.canonical_bytes(),
    )?;
    assert_eq!(changed_data["pair"]["status"], "not_evaluated");
    assert_eq!(changed_data["pair"]["configuration_equal"], false);
    assert_eq!(
        read_live_project_generation_pair(&store_root, &request, &stop)?.canonical_bytes(),
        result.canonical_bytes()
    );
    std::fs::write(location.join("pair.json"), result.canonical_bytes())?;
    std::fs::write(
        location.join("request.json"),
        wow_core::canonical_json_bytes(&request)?,
    )?;
    if exported.is_none() {
        std::fs::remove_dir_all(location)?;
    }
    Ok(())
}

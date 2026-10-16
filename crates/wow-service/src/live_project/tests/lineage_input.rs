//! Project producer over genuine source-absent native generations; no accepted lineage.
use super::*;
use std::sync::atomic::AtomicBool;
use wow_graph::lineage::{
    LineageFingerprintFeature, LineageFunctionKind, LineageLimits, LineageRole,
};
use wow_project::graph::{
    fingerprints::FunctionFingerprintLimits,
    lineage_input::build_project_source_fingerprint_partition,
};
use wow_project::replay::publication::RetainedGenerationPair;

#[test]
fn native_lineage_fingerprints_preserve_original_functions_and_copy_ambiguity() -> TestResult {
    let stop = AtomicBool::new(false);
    let text = "local function Shared() return 1 end\nreturn Shared()\n";
    let (first, first_graph) = owners_with_files(vec![ProjectInputFile::new("main.lua", text)?])?;
    let (second, second_graph) = owners_with_files(vec![
        ProjectInputFile::new("copy.lua", text)?,
        ProjectInputFile::new("moved.lua", text)?,
    ])?;
    let location = root("lineage-native-input")?;
    let mut store =
        LiveProjectStore::create(&location, first_graph.snapshot().universe().as_str())?;
    let a = store.publish(&first, &first_graph, "lineage-input-a", None, &stop)?;
    let expected = store.current()?.ok_or("before Current")?.record_id;
    let b = store.publish(
        &second,
        &second_graph,
        "lineage-input-b",
        Some(expected),
        &stop,
    )?;
    store.close()?;
    drop(first);
    drop(first_graph);
    drop(second);
    drop(second_graph);
    let store = LiveProjectStore::open(&location)?;
    let before = store.read(&ReadSelector::Exact(a.generation_id), &stop)?;
    let after = store.read(&ReadSelector::Exact(b.generation_id), &stop)?;
    let pair = RetainedGenerationPair::bind(
        before.pair(),
        after.pair(),
        before.read.custody(),
        after.read.custody(),
        &stop,
    )?;
    let limits = LineageLimits {
        max_entities: 32,
        max_bucket_members: 16,
        max_pairs: 64,
        max_component_nodes: 32,
        max_component_edges: 64,
        max_work: 16_777_216,
        max_metadata_bytes: 4 * 1024 * 1024,
    };
    let fp = FunctionFingerprintLimits::new(32, 4 * 1024 * 1024, 4 * 1024 * 1024)?;
    let input = build_project_source_fingerprint_partition(&pair, fp, limits, &stop)?;
    assert_eq!(
        input.data().proof_ceiling,
        wow_graph::GraphConfidence::Candidate
    );
    assert_eq!(input.data().coverage.as_ref(), "partial");
    assert_eq!(input.data().records.len(), 6);
    for kind in [LineageFunctionKind::Chunk, LineageFunctionKind::Closure] {
        let left = input
            .data()
            .records
            .iter()
            .filter(|r| r.role == LineageRole::Before && r.function_kind == kind)
            .collect::<Vec<_>>();
        let right = input
            .data()
            .records
            .iter()
            .filter(|r| r.role == LineageRole::After && r.function_kind == kind)
            .collect::<Vec<_>>();
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 2);
        let LineageFingerprintFeature::Available {
            source_span: before_feature,
        } = &left[0].fingerprint
        else {
            return Err("native span absent".into());
        };
        for record in right {
            let LineageFingerprintFeature::Available {
                source_span: after_feature,
            } = &record.fingerprint
            else {
                return Err("native copied span absent".into());
            };
            assert_eq!(
                before_feature, after_feature,
                "path names cannot change exact span fingerprint"
            );
            assert_ne!(
                left[0].node.node_id(),
                record.node.node_id(),
                "generation entities remain distinct"
            );
        }
    }
    let encoded = wow_core::canonical_json_bytes(input.data())?;
    assert!(!String::from_utf8_lossy(&encoded).contains("local function Shared()"));
    let again = build_project_source_fingerprint_partition(&pair, fp, limits, &stop)?;
    assert_eq!(again.partition_id(), input.partition_id());
    let candidates = wow_graph::lineage::generate_lineage_candidate_components(&input, &stop)?;
    assert_eq!(candidates.proposals.len(), 4);
    assert_eq!(candidates.components.len(), 2);
    assert!(
        candidates.components.iter().all(|c| c.shape
            == wow_graph::lineage::LineageComponentShape::OneToMany
            && c.ambiguous)
    );
    assert_eq!(candidates.accepted_assertion_count, 0);
    assert!(candidates.usage().work > 0);
    for (bounded_limits, expected_reason) in [
        (
            LineageLimits {
                max_bucket_members: 2,
                ..limits
            },
            wow_graph::lineage::LineageBucketSkipReason::BucketMembers,
        ),
        (
            LineageLimits {
                max_pairs: 1,
                max_component_edges: 1,
                ..limits
            },
            wow_graph::lineage::LineageBucketSkipReason::PairBudget,
        ),
        (
            LineageLimits {
                max_component_nodes: 2,
                ..limits
            },
            wow_graph::lineage::LineageBucketSkipReason::ComponentNodes,
        ),
        (
            LineageLimits {
                max_component_edges: 1,
                ..limits
            },
            wow_graph::lineage::LineageBucketSkipReason::ComponentEdges,
        ),
    ] {
        let bounded_input =
            build_project_source_fingerprint_partition(&pair, fp, bounded_limits, &stop)?;
        let result =
            wow_graph::lineage::generate_lineage_candidate_components(&bounded_input, &stop)?;
        assert!(result.proposals.is_empty());
        assert_eq!(result.components.len(), 6);
        assert_eq!(result.status.as_ref(), "truncated");
        assert_eq!(result.skipped_buckets.len(), 2);
        assert!(
            result
                .skipped_buckets
                .iter()
                .all(|b| b.reason == expected_reason)
        );
        assert!(!result.negative_authority);
    }
    assert_eq!(
        wow_graph::lineage::generate_lineage_candidate_components(&input, &AtomicBool::new(true))
            .err()
            .map(|e| e.code()),
        Some(wow_graph::GraphErrorCode::Cancelled)
    );
    let before_graph = before.pair().graph_read(&stop)?;
    let after_graph = after.pair().graph_read(&stop)?;
    let mut swapped = input.data().clone();
    swapped.records[0].role = LineageRole::After;
    assert!(
        wow_graph::lineage::validate_project_fingerprint_partition(
            swapped,
            &before_graph,
            &after_graph,
            limits,
            &stop
        )
        .is_err()
    );
    let small = FunctionFingerprintLimits::new(32, 1, 4 * 1024 * 1024)?;
    assert_eq!(
        build_project_source_fingerprint_partition(&pair, small, limits, &stop)
            .err()
            .map(|e| e.code()),
        Some(wow_project::ProjectErrorCode::SourceBudgetExceeded)
    );
    assert_eq!(
        build_project_source_fingerprint_partition(&pair, fp, limits, &AtomicBool::new(true))
            .err()
            .map(|e| e.code()),
        Some(wow_project::ProjectErrorCode::AnalysisCancelled)
    );
    drop(pair);
    before.close()?;
    after.close()?;
    store.close()?;
    std::fs::remove_dir_all(location)?;
    Ok(())
}

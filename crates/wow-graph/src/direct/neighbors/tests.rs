use super::*;
use crate::{GraphConfidence, GraphDirection, GraphLimits};

#[test]
fn cumulative_admission_precedes_growth_and_preserves_exact_prefix_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let stop = AtomicBool::new(false);
    let limits = GraphLimits::default();
    let universe = GraphUniverseId::new("project:batch-admission")?;
    let generation = GraphGenerationId::new("generation:batch-admission")?;
    let nodes = ["root", "child", "other"]
        .into_iter()
        .map(|key| {
            GraphNode::new(
                universe.clone(),
                generation.clone(),
                "function",
                key,
                vec![format!("evidence:{key}").into()],
                limits,
            )
        })
        .collect::<GraphResult<Vec<_>>>()?;
    let edges = (0..12)
        .map(|index| {
            let (from, to) = if index % 3 == 0 {
                (1 + index % 2, 0)
            } else {
                (0, 1 + index % 2)
            };
            GraphEdge::new(
                nodes[from].node_id().clone(),
                nodes[to].node_id().clone(),
                GraphRelationKind::Calls,
                GraphConfidence::Derived,
                (0..12)
                    .map(|item| format!("evidence:{index}:{item}:{}", "a".repeat(240)).into())
                    .collect(),
                limits,
            )
        })
        .collect::<GraphResult<Vec<_>>>()?;
    let root = nodes[0].node_id().clone();
    let snapshot = GraphSnapshot::build(universe, generation, limits, nodes, edges, Vec::new())?;
    let query = GraphNeighborReadQuery::new(
        snapshot.snapshot_id().clone(),
        GraphNeighborQuery::new(
            root,
            GraphDirection::Both,
            vec![GraphRelationKind::Calls],
            12,
        )?,
        GraphNeighborReadLimits {
            max_output_bytes: 16_384,
            ..GraphNeighborReadLimits::default()
        },
    )?;
    let mut output = BatchOutputBudget {
        used: 0,
        limit: 16_384,
    };
    let mut collector = GraphNeighborCollector::new(&query, &snapshot, &mut output, &stop)?;
    let initial_bytes = output.used;
    let initial_selection_charge = collector.used;
    output.limit = initial_bytes;
    assert_eq!(
        collector
            .observe(&snapshot, &snapshot.edges()[0], &mut output, &stop)
            .err()
            .ok_or("output grew past the cumulative allowance")?
            .code(),
        crate::GraphErrorCode::BudgetExceeded,
    );
    assert!(collector.result.edges.is_empty());
    assert!(collector.nodes.is_empty());
    assert!(collector.result.truncations.is_empty());
    assert_eq!(collector.used, initial_selection_charge);
    assert_eq!(output.used, initial_bytes);

    // Collector/dispatch preparation for a repeated root is charged as well.
    output.limit = initial_bytes * 2 - 1;
    assert!(GraphNeighborCollector::new(&query, &snapshot, &mut output, &stop).is_err());
    assert_eq!(output.used, initial_bytes);

    let single = query.execute(&snapshot, &stop)?;
    let legacy = query.query().execute(&snapshot)?;
    assert_eq!(single.matching_edges() as usize, legacy.edges().len());
    for (edge, original) in single.edges().iter().zip(legacy.edges()) {
        assert_eq!(*edge, original);
    }
    assert_eq!(
        single.truncations(),
        &[GraphNeighborTruncation::OutputBytes]
    );
    assert!(!single.edges().is_empty());
    let bytes = wow_core::canonical_json_bytes(&single)?;
    let batch_limits = GraphNeighborBatchLimits {
        max_logical_scanned_edges: (2 * snapshot.edges().len()) as u64,
        max_output_bytes: (2 * bytes.len()) as u64,
    };
    PHYSICAL_EDGE_VISITS.with(|visits| visits.set(0));
    let before = crate::model::snapshot_work_counts();
    let timer = std::time::Instant::now();
    let views = collect_batch(
        &[query.clone(), query.clone()],
        &snapshot,
        batch_limits,
        &stop,
    )?;
    let elapsed = timer.elapsed();
    assert_eq!(crate::model::snapshot_work_counts(), before);
    let visits = PHYSICAL_EDGE_VISITS.with(std::cell::Cell::get);
    assert_eq!(visits, snapshot.edges().len());
    for view in views {
        assert_eq!(wow_core::canonical_json_bytes(&view)?, bytes);
    }
    let refused = GraphNeighborBatchLimits {
        max_output_bytes: batch_limits.max_output_bytes - 1,
        ..batch_limits
    };
    assert_eq!(
        collect_batch(&[query.clone(), query], &snapshot, refused, &stop)
            .err()
            .ok_or("batch accepted one byte below its exact output")?
            .code(),
        crate::GraphErrorCode::BudgetExceeded,
    );
    eprintln!(
        "batch admission: debug={}, roots=2, inventory={}, physical_visits={visits}, encoded_bytes={}, elapsed_us={}, graph_rebuilds=0",
        cfg!(debug_assertions),
        snapshot.edges().len(),
        batch_limits.max_output_bytes,
        elapsed.as_micros(),
    );
    Ok(())
}

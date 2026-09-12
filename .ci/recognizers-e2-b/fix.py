from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement target, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


replace_once(
    "crates/wow-recognizers/src/model.rs",
    "use std::collections::BTreeSet;\n\n",
    "",
)
replace_once(
    "crates/wow-recognizers/src/model.rs",
    """#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredObservation {
""",
    """#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredObservationInput {
    pub source_snapshot_id: GraphSnapshotId,
    pub family: ObservationFamily,
    pub from: GraphNodeId,
    pub to: GraphNodeId,
    pub origin: ObservationOrigin,
    pub confidence: GraphConfidence,
    pub evidence_ids: Vec<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredObservation {
""",
)
replace_once(
    "crates/wow-recognizers/src/model.rs",
    """    pub fn new(
        source_snapshot_id: GraphSnapshotId,
        family: ObservationFamily,
        from: GraphNodeId,
        to: GraphNodeId,
        origin: ObservationOrigin,
        confidence: GraphConfidence,
        evidence_ids: Vec<Box<str>>,
        limits: RecognizerLimits,
    ) -> RecognizerResult<Self> {
        limits.validate()?;
""",
    """    pub fn new(
        input: StructuredObservationInput,
        limits: RecognizerLimits,
    ) -> RecognizerResult<Self> {
        let StructuredObservationInput {
            source_snapshot_id,
            family,
            from,
            to,
            origin,
            confidence,
            evidence_ids,
        } = input;
        limits.validate()?;
""",
)
replace_once(
    "crates/wow-recognizers/src/model.rs",
    """        let rebuilt = Self::new(
            self.source_snapshot_id.clone(),
            self.family,
            self.from.clone(),
            self.to.clone(),
            self.origin,
            self.confidence,
            self.evidence_ids.clone(),
            limits,
        )?;
""",
    """        let rebuilt = Self::new(
            StructuredObservationInput {
                source_snapshot_id: self.source_snapshot_id.clone(),
                family: self.family,
                from: self.from.clone(),
                to: self.to.clone(),
                origin: self.origin,
                confidence: self.confidence,
                evidence_ids: self.evidence_ids.clone(),
            },
            limits,
        )?;
""",
)
replace_once(
    "crates/wow-recognizers/src/lib.rs",
    "    RecognizerRegistry, StructuredObservation,\n",
    "    RecognizerRegistry, StructuredObservation, StructuredObservationInput,\n",
)
replace_once(
    "crates/wow-recognizers/tests/engine_e2.rs",
    """    RecognizerErrorCode, RecognizerLimits, RecognizerRegistry, StructuredObservation,
    project_graph_coverage, project_graph_edges, run_recognizers,
""",
    """    RecognizerErrorCode, RecognizerLimits, RecognizerRegistry, StructuredObservation,
    StructuredObservationInput, project_graph_coverage, project_graph_edges, run_recognizers,
""",
)
replace_once(
    "crates/wow-recognizers/tests/engine_e2.rs",
    """    Ok(StructuredObservation::new(
        snapshot.snapshot_id().clone(),
        family,
        nodes[0].node_id().clone(),
        nodes[target].node_id().clone(),
        origin,
        confidence,
        vec![evidence.into()],
        limits,
    )?)
""",
    """    Ok(StructuredObservation::new(
        StructuredObservationInput {
            source_snapshot_id: snapshot.snapshot_id().clone(),
            family,
            from: nodes[0].node_id().clone(),
            to: nodes[target].node_id().clone(),
            origin,
            confidence,
            evidence_ids: vec![evidence.into()],
        },
        limits,
    )?)
""",
)
replace_once(
    "crates/wow-recognizers/tests/engine_e2.rs",
    """    let stale = StructuredObservation::new(
        other_snapshot.snapshot_id().clone(),
        ObservationFamily::DirectCall,
        nodes[0].node_id().clone(),
        nodes[1].node_id().clone(),
        ObservationOrigin::AnalyzerFact,
        GraphConfidence::Derived,
        vec!["emmy-call:stale".into()],
        limits,
    )?;
""",
    """    let stale = StructuredObservation::new(
        StructuredObservationInput {
            source_snapshot_id: other_snapshot.snapshot_id().clone(),
            family: ObservationFamily::DirectCall,
            from: nodes[0].node_id().clone(),
            to: nodes[1].node_id().clone(),
            origin: ObservationOrigin::AnalyzerFact,
            confidence: GraphConfidence::Derived,
            evidence_ids: vec!["emmy-call:stale".into()],
        },
        limits,
    )?;
""",
)

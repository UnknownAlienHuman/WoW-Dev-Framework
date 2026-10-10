//! Real pack/matcher execution and typed graph proposal projection.
use super::*;
pub(super) fn execute(
    input: &CoreInput<'_>,
    family: SourceStateCoreFamily,
    admitted: adapt::AdmittedFacts,
    stop: &AtomicBool,
) -> RecognizerResult<SourceStateCoreProposals> {
    let adapt::AdmittedFacts {
        graph,
        facts,
        origins,
        limits,
        admission_partition_digest,
        endpoints,
        prerequisites,
    } = admitted;
    let mut entities =
        BTreeMap::<Vec<u8>, (GraphEntityProposal, BTreeSet<wow_core::CoverageId>)>::new();
    let mut relations = Vec::new();
    let mut receipts = BTreeMap::<String, SourceStateCoreReceipt>::new();
    let mut evaluations = Vec::new();
    for has_path in [false, true] {
        checkpoint(stop)?;
        let selected: Vec<_> = facts
            .iter()
            .filter(|fact| fact.fields().get("has_path") == Some(&Value::Boolean(has_path)))
            .cloned()
            .collect();
        let coverage = RecognizerFactCoverage::new(
            RecognizerFactCoverageInput {
                context_id: input.context.context_id(),
                partition_id: family.partition_id().into(),
                capability_id: family.capability_id().into(),
                producer_id: "wow.recognizers.state-admission".into(),
                producer_version: FACT_PROFILE.into(),
                state: if selected.is_empty() {
                    RecognizerFactCoverageState::NotEvaluated
                } else {
                    RecognizerFactCoverageState::Partial
                },
                blocker_ids: vec!["state.admitted_slots_no_complete_or_runtime_authority".into()],
            },
            limits,
        )?;
        let bundle = RecognizerFactBundle::build(
            input.context,
            family.partition_id(),
            vec![SOURCE_STATE_PARTITION.into()],
            selected,
            vec![coverage],
            limits,
        )?;
        let pack = pack::compile(family, has_path, input.owner.registry().bundle_id())?;
        let plan = compile_recognizer_plan(&pack)?;
        let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, limits, stop)?;
        for outcome in output.outcomes() {
            checkpoint(stop)?;
            if outcome.rule_id() != family.rule_id() || outcome.rule_version() != 1 {
                return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
            }
            for matched in outcome.matches() {
                let mut fact_ids = BTreeSet::new();
                for id in matched.decisive_fact_ids() {
                    fact_ids.extend(
                        origins
                            .get(id)
                            .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?
                            .iter()
                            .cloned(),
                    );
                }
                let match_id = matched.match_id().to_string();
                if receipts
                    .insert(
                        match_id.clone(),
                        SourceStateCoreReceipt {
                            match_id,
                            fact_ids: fact_ids.into_iter().collect(),
                            entity_proposal_ids: Vec::new(),
                            relation_proposal_ids: Vec::new(),
                        },
                    )
                    .is_some()
                {
                    return Err(fail(RecognizerErrorCode::AdapterBindingDuplicate));
                }
            }
            let mut proposed = BTreeMap::new();
            for assertion in outcome.proposals() {
                checkpoint(stop)?;
                if let RecognizerProposedAssertion::Entity {
                    proposal_id,
                    output_id,
                    entity_kind_id,
                    semantic_key,
                    confidence,
                    match_id,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                    ..
                } = assertion
                {
                    if entity_kind_id.as_ref() != "state_path" || output_id.as_ref() != "path" {
                        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
                    }
                    let key = semantic_key
                        .iter()
                        .map(|(name, value)| {
                            let value = match value {
                                Value::Identifier(value) => {
                                    GraphProposalValue::Identifier(value.clone())
                                }
                                Value::String(value) => GraphProposalValue::String(value.clone()),
                                _ => return Err(fail(RecognizerErrorCode::AdapterFactMismatch)),
                            };
                            Ok((name.clone(), value))
                        })
                        .collect::<RecognizerResult<BTreeMap<_, _>>>()?;
                    let group_key = canonical_json_bytes(&key)
                        .map_err(|_| fail(RecognizerErrorCode::AdapterFactMismatch))?;
                    let mut entity = GraphEntityProposal::new(
                        proposal_id.as_str(),
                        "state_path",
                        key,
                        confidence_of(*confidence),
                        source_handle_ids.clone(),
                        evidence_ids.clone(),
                        coverage_ids.clone(),
                    )
                    .map_err(graph_error)?;
                    let mut coverage = coverage_ids.iter().copied().collect::<BTreeSet<_>>();
                    if let Some((previous, previous_coverage)) = entities.get(&group_key) {
                        coverage.extend(previous_coverage.iter().copied());
                        entity = merge_entity(previous, &entity, &coverage)?;
                    }
                    let id = entity.proposal_id().to_owned();
                    entities.insert(group_key, (entity, coverage));
                    proposed.insert((match_id.to_string(), output_id.to_string()), id.clone());
                    receipts
                        .get_mut(&match_id.to_string())
                        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?
                        .entity_proposal_ids
                        .push(id);
                }
            }
            for assertion in outcome.proposals() {
                checkpoint(stop)?;
                let RecognizerProposedAssertion::Relation {
                    proposal_id,
                    relation_kind_id,
                    source,
                    target,
                    confidence,
                    match_id,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                    ..
                } = assertion
                else {
                    continue;
                };
                if relation_kind_id.as_ref() != family.definition_id() {
                    return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
                }
                let bind = |value: &Value| -> RecognizerResult<GraphProposalEndpoint> {
                    let Value::Reference(token) = value else {
                        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
                    };
                    if let Some(role) = token.strip_prefix("entity:") {
                        let id = proposed
                            .get(&(match_id.to_string(), role.to_owned()))
                            .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
                        Ok(GraphProposalEndpoint::Proposed(id.clone().into()))
                    } else {
                        let id = endpoints
                            .get(token.as_ref())
                            .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?
                            .clone();
                        if graph.node(&id).is_none() {
                            return Err(fail(RecognizerErrorCode::AdapterBindingMissing));
                        }
                        Ok(GraphProposalEndpoint::Existing(id))
                    }
                };
                let id = proposal_id.to_string();
                relations.push(
                    GraphRelationProposal::new(
                        id.as_str(),
                        family.definition_id(),
                        GraphRelationProposalInput {
                            source: bind(source)?,
                            target: bind(target)?,
                            confidence: confidence_of(*confidence),
                            source_handle_ids: source_handle_ids.clone(),
                            evidence_ids: evidence_ids.clone(),
                            coverage_ids: coverage_ids.clone(),
                        },
                    )
                    .map_err(graph_error)?,
                );
                receipts
                    .get_mut(&match_id.to_string())
                    .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?
                    .relation_proposal_ids
                    .push(id);
            }
        }
        evaluations.push(SourceStateCoreEvaluation {
            recipe: if has_path {
                "literal_path"
            } else {
                "root_slot"
            },
            pack_digest: pack.pack_digest().into(),
            fact_bundle: bundle,
            output,
        });
    }
    let coverage = input
        .owner
        .registry()
        .relation_kinds()
        .iter()
        .map(|definition| definition.relation())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|relation| {
            GraphCoverageRecord::new(
                relation,
                if relation == family.relation() && !facts.is_empty() {
                    GraphCoverageState::Partial
                } else {
                    GraphCoverageState::NotEvaluated
                },
                false,
                vec!["state.admitted_slots_no_complete_or_runtime_authority".into()],
                graph.limits(),
            )
            .map_err(graph_error)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;
    let batch = GraphProposalBatch::build(
        input.owner.registry().bundle_id(),
        input.owner.registry().registry_digest(),
        graph.universe().clone(),
        graph.generation().clone(),
        input.context.context_id(),
        family.partition_id(),
        entities.into_values().map(|(entity, _)| entity).collect(),
        relations,
    )
    .map_err(graph_error)?;
    Ok(SourceStateCoreProposals {
        batch: records::attach(family, &receipts, &prerequisites, batch, stop)?,
        coverage,
        recognition: SourceStateCoreRecognition {
            profile: "wow-recognizers/state-structural/2",
            family,
            analyzer_report_id: input.recognition.analyzer_report_id().into(),
            evaluations,
            receipts: receipts.into_values().collect(),
            admission_partition_digest,
        },
    })
}

// One path assertion retains every access witness while receipts keep the
// original matcher proposals in evaluations and point to the merged entity.
fn merge_entity(
    left: &GraphEntityProposal,
    right: &GraphEntityProposal,
    coverage: &BTreeSet<wow_core::CoverageId>,
) -> RecognizerResult<GraphEntityProposal> {
    if left.entity_kind_id() != right.entity_kind_id()
        || left.semantic_key() != right.semantic_key()
    {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    let handles = left
        .source_handle_ids()
        .iter()
        .chain(right.source_handle_ids())
        .copied()
        .collect::<BTreeSet<_>>();
    let evidence = left
        .evidence_ids()
        .iter()
        .chain(right.evidence_ids())
        .copied()
        .collect::<BTreeSet<_>>();
    if handles.len() > 64 || evidence.len() > 64 {
        return Err(fail(RecognizerErrorCode::BudgetExceeded));
    }
    GraphEntityProposal::new(
        left.proposal_id(),
        left.entity_kind_id(),
        left.semantic_key().clone(),
        if left.confidence() == GraphConfidence::Possible
            || right.confidence() == GraphConfidence::Possible
        {
            GraphConfidence::Possible
        } else {
            GraphConfidence::Derived
        },
        handles.into_iter().collect(),
        evidence.into_iter().collect(),
        coverage.iter().copied().collect(),
    )
    .map_err(graph_error)
}

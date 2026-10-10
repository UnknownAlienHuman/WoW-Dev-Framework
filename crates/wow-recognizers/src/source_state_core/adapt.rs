//! Exact predecessor admission to bounded normalized state facts.
use super::*;
pub(super) struct AdmittedFacts {
    pub graph: wow_graph::GraphSnapshot,
    pub facts: Vec<RecognizerFact>,
    pub origins: BTreeMap<crate::RecognizerFactId, Vec<String>>,
    pub limits: RecognizerFactLimits,
    pub admission_partition_digest: String,
    pub endpoints: BTreeMap<String, GraphNodeId>,
    pub prerequisites: BTreeMap<String, BTreeSet<GraphAssertionRef>>,
}
pub(super) fn normalize(
    input: &CoreInput<'_>,
    family: SourceStateCoreFamily,
    stop: &AtomicBool,
) -> RecognizerResult<AdmittedFacts> {
    checkpoint(stop)?;
    input
        .context
        .validate()
        .map_err(|_| fail(RecognizerErrorCode::AdapterIdentityMismatch))?;
    input.owner.validate(stop).map_err(graph_error)?;
    if input.recognition.receipts().len() > MAX_BINDINGS {
        return Err(fail(RecognizerErrorCode::BudgetExceeded));
    }
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(fail(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    let lookup = input.owner.producer_lookup(stop).map_err(graph_error)?;
    if let CoreRecognition::Assertions(recognition) = input.recognition
        && recognition.scope() != lookup.scope()
    {
        return Err(fail(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    let graph = lookup.input_view().clone();
    checkpoint(stop)?;
    let legacy = input
        .owner
        .partition(SOURCE_STATE_PARTITION)
        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
    if legacy.report().accepted_relations().len() != input.recognition.receipts().len()
        || !legacy.batch().entity_proposals().is_empty()
    {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    if let CoreRecognition::Assertions(recognition) = input.recognition {
        let borrowed = recognition
            .receipts()
            .iter()
            .map(|receipt| {
                let refs = recognition
                    .endpoints(&receipt.binding_id)
                    .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
                checkpoint(stop)?;
                Ok((
                    &receipt.binding_id,
                    legacy.partition_id(),
                    legacy.batch().batch_id(),
                    &receipt.proposal_id,
                    &refs.caller,
                    &refs.target,
                ))
            })
            .collect::<RecognizerResult<Vec<_>>>()?;
        crate::source_assertions::preflight(&(recognition, &borrowed), stop)?;
    }
    let assertions = input
        .recognition
        .recognition()
        .assertions()
        .iter()
        .map(|assertion| (assertion.assertion_id().as_str(), assertion))
        .collect::<BTreeMap<_, _>>();
    let mut source_nodes = BTreeMap::new();
    // Source state keys are admitted in this partition, before any core rule.
    let source_partition = match input.recognition {
        CoreRecognition::Legacy(recognition) => Some(
            input
                .owner
                .partition(recognition.source_partition())
                .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?,
        ),
        CoreRecognition::Assertions(_) => None,
    };
    if let Some(partition) = source_partition {
        for accepted in partition.report().accepted_entities() {
            checkpoint(stop)?;
            let proposal = partition
                .batch()
                .entity_proposal(accepted.proposal_id())
                .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
            if (proposal.entity_kind_id() == "state_root"
                && proposal.confidence() == GraphConfidence::Proven)
                || (proposal.entity_kind_id() == "state_path"
                    && proposal.confidence() == GraphConfidence::Derived)
            {
                source_nodes
                    .entry(accepted.node().node_id().clone())
                    .or_insert(proposal);
            }
        }
    }
    // Preserve the legacy reverse-map choice for explanation prerequisites.
    let source_ids = source_partition.map(|partition| {
        partition
            .report()
            .accepted_entities()
            .iter()
            .map(|accepted| (accepted.node().node_id(), accepted.proposal_id()))
            .collect::<BTreeMap<_, _>>()
    });
    let limits = RecognizerFactLimits::new(MAX_BINDINGS as u32, 8, 8, 16, 32, 32, 8)?;
    let mut facts = Vec::new();
    let mut origins = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut bytes = 0usize;
    let mut endpoints = BTreeMap::new();
    let mut prerequisites = BTreeMap::new();
    for receipt in input.recognition.receipts() {
        checkpoint(stop)?;
        if !seen.insert(receipt.proposal_id.as_str()) {
            return Err(fail(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        let accepted = legacy.report().accepted_relations();
        let index = accepted
            .binary_search_by(|entry| entry.proposal_id().cmp(&receipt.proposal_id))
            .map_err(|_| fail(RecognizerErrorCode::AdapterBindingMissing))?;
        let edge = accepted[index].edge();
        let proposal = legacy
            .batch()
            .relation_proposal(&receipt.proposal_id)
            .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
        let assertion = assertions
            .get(receipt.assertion_id.as_str())
            .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
        if receipt.assertion_id != receipt.proposal_id
            || assertion.observation_id().as_str() != receipt.observation_id
            || assertion.from() != edge.from()
            || assertion.to() != edge.to()
            || assertion.relation() != edge.relation()
            || assertion.confidence() != edge.confidence()
            || proposal.evidence_ids().len() != edge.evidence_ids().len()
            || proposal
                .evidence_ids()
                .iter()
                .zip(edge.evidence_ids())
                .any(|(id, witness)| id.to_string().as_str() != witness.as_ref())
        {
            return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
        }
        let exact = if matches!(input.recognition, CoreRecognition::Assertions(_)) {
            Some(exact_target(
                input,
                &lookup,
                receipt,
                edge,
                legacy.batch(),
                stop,
            )?)
        } else {
            None
        };
        if edge.relation() != family.relation() {
            continue;
        }
        if proposal.relation_kind_id() != family.definition_id()
            || graph
                .node(edge.from())
                .is_none_or(|node| node.kind() != "lua_source_function")
            || proposal.source_handle_ids().is_empty()
            || proposal.source_handle_ids().len() > 32
            || proposal.evidence_ids().is_empty()
            || proposal.evidence_ids().len() > 32
        {
            return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
        }
        let target = match exact {
            Some(target) => target,
            None => *source_nodes
                .get(edge.to())
                .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?,
        };
        let has_path = target.entity_kind_id() == "state_path";
        let root_id = if has_path {
            let Some(GraphProposalValue::Identifier(root)) = target.semantic_key().get("root")
            else {
                return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
            };
            root.as_ref()
        } else {
            target.proposal_id()
        };
        // Bind the transferred handles as well as the admitted edge. The
        // original access identity includes the entire exact support vectors.
        let identity = wow_core::domain_separated_digest(
            "wow-project/saved-access/2",
            &(
                &receipt.access_id,
                root_id,
                match family {
                    SourceStateCoreFamily::Read => wow_emmy::global_access::GlobalAccessKind::Read,
                    SourceStateCoreFamily::Write => {
                        wow_emmy::global_access::GlobalAccessKind::Write
                    }
                },
                edge.confidence(),
                target.proposal_id(),
                proposal.source_handle_ids(),
                proposal.evidence_ids(),
            ),
        )
        .map_err(|_| fail(RecognizerErrorCode::AdapterIdentityMismatch))?;
        if receipt.binding_id
            != format!(
                "saved-access:{}",
                wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(identity)
            )
        {
            return Err(fail(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        let mut inputs = BTreeSet::from([GraphAssertionRef::Producer {
            partition_id: legacy.partition_id().into(),
            batch_id: legacy.batch().batch_id().into(),
            assertion: GraphLocalAssertion {
                kind: GraphAssertionKind::Relation,
                proposal_id: receipt.proposal_id.clone().into(),
            },
        }]);
        match input.recognition {
            CoreRecognition::Legacy(_) => {
                let partition = source_partition
                    .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
                let ids = source_ids
                    .as_ref()
                    .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
                for node in [edge.from(), edge.to()] {
                    let id = ids
                        .get(node)
                        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
                    inputs.insert(GraphAssertionRef::Producer {
                        partition_id: partition.partition_id().into(),
                        batch_id: partition.batch().batch_id().into(),
                        assertion: GraphLocalAssertion {
                            kind: GraphAssertionKind::Entity,
                            proposal_id: (*id).into(),
                        },
                    });
                }
            }
            CoreRecognition::Assertions(recognition) => {
                let refs = recognition
                    .endpoints(&receipt.binding_id)
                    .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
                inputs.insert(refs.caller.clone());
                inputs.insert(refs.target.clone());
            }
        }
        prerequisites.insert(receipt.binding_id.clone(), inputs);
        checkpoint(stop)?;
        endpoints.insert(edge.from().to_string(), edge.from().clone());
        if !has_path {
            endpoints.insert(edge.to().to_string(), edge.to().clone());
        }
        bytes = bytes
            .saturating_add(
                canonical_json_bytes(target.semantic_key())
                    .map_err(|_| fail(RecognizerErrorCode::AdapterFactMismatch))?
                    .len(),
            )
            .saturating_add(receipt.binding_id.len() + receipt.access_id.len() + 2048)
            .saturating_add(
                (proposal.source_handle_ids().len() + proposal.evidence_ids().len()) * 64,
            );
        if bytes > MAX_FACT_BYTES {
            return Err(fail(RecognizerErrorCode::BudgetExceeded));
        }
        let mut fields = BTreeMap::from([
            (
                "binding_id".into(),
                Value::String(receipt.binding_id.clone().into()),
            ),
            (
                "access_id".into(),
                Value::String(receipt.access_id.clone().into()),
            ),
            ("has_path".into(), Value::Boolean(has_path)),
            (
                "source".into(),
                Value::Reference(edge.from().to_string().into()),
            ),
            (
                "target".into(),
                Value::Reference(if has_path {
                    "entity:path".into()
                } else {
                    edge.to().to_string().into()
                }),
            ),
        ]);
        if has_path {
            let (
                Some(GraphProposalValue::Identifier(root)),
                Some(GraphProposalValue::String(path)),
            ) = (
                target.semantic_key().get("root"),
                target.semantic_key().get("path"),
            )
            else {
                return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
            };
            fields.insert("root".into(), Value::Identifier(root.clone()));
            fields.insert("path".into(), Value::String(path.clone()));
        }
        let fact = RecognizerFact::new(
            input.context.context_id(),
            RecognizerFactInput {
                kind: "state_access".into(),
                partition_id: family.partition_id().into(),
                scope: RecognizerFactScope::new(
                    RecognizerFactScopeKind::Function,
                    edge.from().to_string(),
                )?,
                producer_id: "wow.recognizers.state-admission".into(),
                producer_version: FACT_PROFILE.into(),
                confidence: edge.confidence(),
                fields,
                source_handle_ids: proposal.source_handle_ids().to_vec(),
                evidence_ids: proposal.evidence_ids().to_vec(),
            },
            limits,
        )?;
        let mut origin_ids = vec![receipt.binding_id.clone(), receipt.access_id.clone()];
        origin_ids.sort();
        origin_ids.dedup();
        origins.insert(fact.fact_id().clone(), origin_ids);
        facts.push(fact);
    }
    Ok(AdmittedFacts {
        graph,
        facts,
        origins,
        limits,
        admission_partition_digest: legacy.partition_digest().into(),
        endpoints,
        prerequisites,
    })
}

fn exact_target<'a>(
    input: &CoreInput<'a>,
    lookup: &wow_graph::GraphProducerLookup<'a>,
    receipt: &crate::source_state::SourceStateReceipt,
    edge: &wow_graph::GraphEdge,
    admission: &GraphProposalBatch,
    stop: &AtomicBool,
) -> RecognizerResult<&'a GraphEntityProposal> {
    let CoreRecognition::Assertions(recognition) = input.recognition else {
        return Err(fail(RecognizerErrorCode::AdapterBindingInvalid));
    };
    let refs = recognition
        .endpoints(&receipt.binding_id)
        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
    let resolve = |reference: &GraphAssertionRef| {
        let GraphAssertionRef::Producer { assertion, .. } = reference else {
            return Err(fail(RecognizerErrorCode::AdapterBindingInvalid));
        };
        crate::source_assertions::entity(
            lookup,
            recognition.scope(),
            reference,
            &assertion.proposal_id,
            stop,
        )
    };
    let root = resolve(&refs.root)?;
    let caller = resolve(&refs.caller)?;
    let target = resolve(&refs.target)?;
    if root.proposal().entity_kind_id() != "state_root"
        || root.proposal().confidence() != GraphConfidence::Proven
        || caller.proposal().entity_kind_id() != "lua_source_function"
        || caller.proposal().confidence() != GraphConfidence::Derived
        || caller.accepted().node().node_id() != edge.from()
        || target.accepted().node().node_id() != edge.to()
    {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    if refs.root == refs.target {
        if target.proposal() != root.proposal() {
            return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
        }
    } else if target.proposal().entity_kind_id() != "state_path"
        || target.proposal().confidence() != GraphConfidence::Derived
        || !matches!(target.proposal().semantic_key().get("root"),
            Some(GraphProposalValue::Identifier(id)) if id.as_ref() == root.proposal().proposal_id())
    {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    let records = admission
        .assertion_records()
        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
    let derivation = records
        .derivations
        .iter()
        .find(|record| {
            record.output.kind == GraphAssertionKind::Relation
                && record.output.proposal_id.as_ref() == receipt.proposal_id
        })
        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
    let rule = match edge.relation() {
        GraphRelationKind::ReadsState => "wow-recognizers.state-read-admission",
        GraphRelationKind::WritesState => "wow-recognizers.state-write-admission",
        _ => return Err(fail(RecognizerErrorCode::AdapterFactMismatch)),
    };
    if &records.scope != recognition.scope()
        || derivation.rule_id.as_ref() != rule
        || derivation.rule_version != 1
        || !derivation.rebuttals.is_empty()
        || !derivation.missing.is_empty()
        || derivation.inputs.iter().collect::<BTreeSet<_>>()
            != BTreeSet::from([&refs.root, &refs.caller, &refs.target])
    {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    checkpoint(stop)?;
    Ok(target.proposal())
}

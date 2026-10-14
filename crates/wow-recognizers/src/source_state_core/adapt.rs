//! Exact predecessor admission to bounded normalized state facts.
use super::*;
pub(super) struct AdmittedFacts {
    pub graph: wow_graph::GraphSnapshot,
    pub facts: Vec<RecognizerFact>,
    pub origins: BTreeMap<crate::RecognizerFactId, Vec<String>>,
    pub limits: RecognizerFactLimits,
    pub admission_partition_digest: String,
    pub producer_partitions: Vec<String>,
    pub incomplete_access_ids: BTreeSet<String>,
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
    let incomplete_count = match input.incomplete.as_ref() {
        Some(IncompleteInput::Legacy(source)) => source.facts.len(),
        Some(IncompleteInput::Assertions(source)) => source.facts.len(),
        None => 0,
    };
    if input
        .recognition
        .receipts()
        .len()
        .checked_add(incomplete_count)
        .is_none_or(|count| count > MAX_BINDINGS)
    {
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
    let mut producer_partitions = BTreeSet::from([SOURCE_STATE_PARTITION.to_owned()]);
    let mut incomplete_access_ids = BTreeSet::new();
    let mut seen_access_ids = input
        .recognition
        .receipts()
        .iter()
        .map(|receipt| receipt.access_id.clone())
        .collect::<BTreeSet<_>>();
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
    let mut dynamic = DynamicAdmission::default();
    let incomplete_context = IncompleteContext {
        family,
        lookup: &lookup,
        graph: &graph,
        limits,
        stop,
    };
    admit_incomplete_input(
        input,
        &incomplete_context,
        &mut seen_access_ids,
        &mut dynamic,
    )?;
    bytes = bytes.saturating_add(dynamic.bytes);
    if bytes > MAX_FACT_BYTES {
        return Err(fail(RecognizerErrorCode::BudgetExceeded));
    }
    facts.extend(dynamic.facts);
    for (id, origin_ids) in dynamic.origins {
        if origins.insert(id, origin_ids).is_some() {
            return Err(fail(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    for (id, node) in dynamic.endpoints {
        if endpoints
            .insert(id, node.clone())
            .is_some_and(|previous| previous != node)
        {
            return Err(fail(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    for (id, inputs) in dynamic.prerequisites {
        if prerequisites.insert(id, inputs).is_some() {
            return Err(fail(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    incomplete_access_ids.extend(dynamic.access_ids);
    producer_partitions.extend(dynamic.producer_partitions);
    Ok(AdmittedFacts {
        graph,
        facts,
        origins,
        limits,
        admission_partition_digest: legacy.partition_digest().into(),
        producer_partitions: producer_partitions.into_iter().collect(),
        incomplete_access_ids,
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

struct IncompleteEndpoints<'a> {
    root: &'a GraphEntityProposal,
    caller: &'a GraphEntityProposal,
    target: &'a GraphEntityProposal,
    root_node: GraphNodeId,
    caller_node: GraphNodeId,
    target_node: GraphNodeId,
    references: BTreeSet<GraphAssertionRef>,
}

struct IncompleteSupport<'a> {
    context: &'a GenerationContext,
    report: &'a wow_emmy::function_calls::FunctionCallReport,
    source_handles: &'a BTreeMap<wow_core::StableHandleId, wow_core::SourceHandle>,
    evidence: &'a BTreeMap<wow_core::EvidenceId, wow_core::EvidenceRecord>,
    graph: &'a wow_graph::GraphSnapshot,
    limits: RecognizerFactLimits,
}

#[derive(Default)]
struct DynamicAdmission {
    facts: Vec<RecognizerFact>,
    origins: BTreeMap<crate::RecognizerFactId, Vec<String>>,
    endpoints: BTreeMap<String, GraphNodeId>,
    prerequisites: BTreeMap<String, BTreeSet<GraphAssertionRef>>,
    access_ids: BTreeSet<String>,
    producer_partitions: BTreeSet<String>,
    bytes: usize,
}

struct IncompleteContext<'a, 'owner> {
    family: SourceStateCoreFamily,
    lookup: &'a wow_graph::GraphProducerLookup<'owner>,
    graph: &'a wow_graph::GraphSnapshot,
    limits: RecognizerFactLimits,
    stop: &'a AtomicBool,
}

fn admit_incomplete_fact(
    support: &IncompleteSupport<'_>,
    family: SourceStateCoreFamily,
    fact: &crate::source_state::SourceStateFact<'_>,
    access: &wow_emmy::global_access::SourceGlobalAccess,
    endpoints: IncompleteEndpoints<'_>,
    admitted: &mut DynamicAdmission,
    seen_access_ids: &mut BTreeSet<String>,
) -> RecognizerResult<()> {
    let expected_kind = match family {
        SourceStateCoreFamily::Read => wow_emmy::global_access::GlobalAccessKind::Read,
        SourceStateCoreFamily::Write => wow_emmy::global_access::GlobalAccessKind::Write,
    };
    if !seen_access_ids.insert(fact.access_id.into())
        || fact.access_id != access.fact_id()
        || fact.kind != expected_kind
        || access.kind() != expected_kind
        || access.path_complete()
        || access.alias_blocker().is_some()
        || access.resolution() != wow_emmy::global_access::GlobalAccessResolution::MainGlobal
        || fact.confidence != GraphConfidence::Possible
    {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    let declaration = access
        .declaration()
        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
    if declaration.role != "main" || declaration.workspace_id != support.report.main_snapshot_id() {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    let source_index = support
        .report
        .files()
        .binary_search_by(|file| file.path.cmp(&declaration.path))
        .map_err(|_| fail(RecognizerErrorCode::AdapterFactMismatch))?;
    let source = &support.report.files()[source_index];
    if source.parse_error_count != 0 || source.content_digest != declaration.content_digest {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    validate_incomplete_support(support, fact)?;

    if endpoints.root.proposal_id() != fact.root_proposal_id
        || endpoints.root.entity_kind_id() != "state_root"
        || endpoints.root.confidence() != GraphConfidence::Proven
        || !matches!(
            endpoints.root.semantic_key().get("name"),
            Some(GraphProposalValue::String(name)) if name.as_ref() == access.root_name()
        )
        || endpoints.caller.proposal_id() != fact.caller_proposal_id
        || endpoints.caller.entity_kind_id() != "lua_source_function"
        || endpoints.caller.confidence() != GraphConfidence::Derived
        || !matches!(
            endpoints.caller.semantic_key().get("document"),
            Some(GraphProposalValue::String(document)) if document.as_ref() == access.path()
        )
        || !matches!(
            endpoints.caller.semantic_key().get("function"),
            Some(GraphProposalValue::String(function)) if function.as_ref() == access.function_id()
        )
        || endpoints.target.proposal_id() != fact.target_proposal_id
        || support
            .graph
            .node(&endpoints.root_node)
            .is_none_or(|node| node.kind() != "state_root")
        || support
            .graph
            .node(&endpoints.caller_node)
            .is_none_or(|node| node.kind() != "lua_source_function")
        || support.graph.node(&endpoints.target_node).is_none()
    {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }

    let has_path = !access.keys().is_empty();
    if has_path {
        let expected_path = String::from_utf8(
            canonical_json_bytes(access.keys())
                .map_err(|_| fail(RecognizerErrorCode::AdapterFactMismatch))?,
        )
        .map_err(|_| fail(RecognizerErrorCode::AdapterFactMismatch))?;
        if endpoints.target.entity_kind_id() != "state_path"
            || endpoints.target.confidence() != GraphConfidence::Derived
            || !matches!(
                endpoints.target.semantic_key().get("root"),
                Some(GraphProposalValue::Identifier(root))
                    if root.as_ref() == endpoints.root.proposal_id()
            )
            || !matches!(
                endpoints.target.semantic_key().get("path"),
                Some(GraphProposalValue::String(path)) if path.as_ref() == expected_path
            )
        {
            return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
        }
    } else if endpoints.target.proposal_id() != endpoints.root.proposal_id()
        || endpoints.target != endpoints.root
        || fact.target_proposal_id != fact.root_proposal_id
    {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }

    let identity = wow_core::domain_separated_digest(
        "wow-project/saved-access/2",
        &(
            access.fact_id(),
            fact.root_proposal_id,
            fact.kind,
            fact.confidence,
            fact.target_proposal_id,
            fact.source_handle_ids,
            fact.evidence_ids,
        ),
    )
    .map_err(|_| fail(RecognizerErrorCode::AdapterIdentityMismatch))?;
    if fact.fact_id
        != format!(
            "saved-access:{}",
            wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(identity)
        )
    {
        return Err(fail(RecognizerErrorCode::AdapterIdentityMismatch));
    }

    let mut fields = BTreeMap::from([
        ("binding_id".into(), Value::String(fact.fact_id.into())),
        ("access_id".into(), Value::String(fact.access_id.into())),
        ("has_path".into(), Value::Boolean(has_path)),
        (
            "source".into(),
            Value::Reference(endpoints.caller_node.to_string().into()),
        ),
        (
            "target".into(),
            Value::Reference(if has_path {
                "entity:path".into()
            } else {
                endpoints.target_node.to_string().into()
            }),
        ),
    ]);
    if has_path {
        let (Some(GraphProposalValue::Identifier(root)), Some(GraphProposalValue::String(path))) = (
            endpoints.target.semantic_key().get("root"),
            endpoints.target.semantic_key().get("path"),
        ) else {
            return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
        };
        fields.insert("root".into(), Value::Identifier(root.clone()));
        fields.insert("path".into(), Value::String(path.clone()));
    }
    let recognizer_fact = RecognizerFact::new(
        support.context.context_id(),
        RecognizerFactInput {
            kind: "state_access".into(),
            partition_id: family.partition_id().into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Function,
                endpoints.caller_node.to_string(),
            )?,
            producer_id: "wow.recognizers.state-admission".into(),
            producer_version: FACT_PROFILE.into(),
            confidence: fact.confidence,
            fields,
            source_handle_ids: fact.source_handle_ids.to_vec(),
            evidence_ids: fact.evidence_ids.to_vec(),
        },
        support.limits,
    )?;
    let mut origin_ids = vec![fact.fact_id.into(), fact.access_id.into()];
    origin_ids.sort();
    origin_ids.dedup();
    let mut endpoints_to_add = vec![(endpoints.caller_node.to_string(), endpoints.caller_node)];
    if !has_path {
        endpoints_to_add.push((endpoints.target_node.to_string(), endpoints.target_node));
    }
    admitted.bytes = admitted
        .bytes
        .saturating_add(
            canonical_json_bytes(endpoints.target.semantic_key())
                .map_err(|_| fail(RecognizerErrorCode::AdapterFactMismatch))?
                .len(),
        )
        .saturating_add(fact.fact_id.len() + fact.access_id.len() + 2048)
        .saturating_add((fact.source_handle_ids.len() + fact.evidence_ids.len()) * 64);
    if admitted.bytes > MAX_FACT_BYTES {
        return Err(fail(RecognizerErrorCode::BudgetExceeded));
    }
    let fact_id = recognizer_fact.fact_id().clone();
    if admitted.origins.insert(fact_id, origin_ids).is_some()
        || admitted
            .prerequisites
            .insert(fact.fact_id.into(), endpoints.references)
            .is_some()
        || !admitted.access_ids.insert(fact.access_id.into())
    {
        return Err(fail(RecognizerErrorCode::AdapterBindingDuplicate));
    }
    for (id, node) in endpoints_to_add {
        if admitted
            .endpoints
            .insert(id, node.clone())
            .is_some_and(|previous| previous != node)
        {
            return Err(fail(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    admitted.facts.push(recognizer_fact);
    Ok(())
}

fn validate_incomplete_support(
    support: &IncompleteSupport<'_>,
    fact: &crate::source_state::SourceStateFact<'_>,
) -> RecognizerResult<()> {
    if fact.source_handle_ids.is_empty()
        || fact.source_handle_ids.len() > 32
        || fact.evidence_ids.is_empty()
        || fact.evidence_ids.len() > 32
        || fact
            .source_handle_ids
            .windows(2)
            .any(|window| window[0] >= window[1])
        || fact
            .evidence_ids
            .windows(2)
            .any(|window| window[0] >= window[1])
    {
        return Err(fail(RecognizerErrorCode::AdapterBindingInvalid));
    }
    let mut witnessed = BTreeSet::new();
    for id in fact.evidence_ids {
        let record = support
            .evidence
            .get(id)
            .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
        record
            .validate()
            .map_err(|_| fail(RecognizerErrorCode::AdapterFactMismatch))?;
        let [handle_id] = record.source_handle_ids() else {
            return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
        };
        if record.evidence_id() != *id
            || record.context_id() != support.context.context_id()
            || record.provenance() != wow_core::ProvenanceClass::ProjectSource
            || record.confidence() != wow_core::EvidenceConfidence::Proven
            || record.claim_scope() != wow_core::ClaimScope::SourceObservation
            || !record.derivation_input_ids().is_empty()
            || !record.coverage_refs().is_empty()
            || fact.source_handle_ids.binary_search(handle_id).is_err()
        {
            return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
        }
        let handle = support
            .source_handles
            .get(handle_id)
            .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
        handle
            .validate()
            .map_err(|_| fail(RecognizerErrorCode::AdapterFactMismatch))?;
        if handle.handle_id() != *handle_id
            || handle.project_generation() != support.context.project_generation()
            || handle.reference_generation() != Some(support.context.reference_generation())
        {
            return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
        }
        witnessed.insert(*handle_id);
    }
    if witnessed.iter().ne(fact.source_handle_ids.iter()) {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}

fn accepted_project_entity<'a>(
    partition: &'a wow_graph::GraphProducerPartition,
    graph: &wow_graph::GraphSnapshot,
    proposal_id: &str,
) -> RecognizerResult<(&'a GraphEntityProposal, GraphNodeId)> {
    let accepted = partition.report().accepted_entities();
    let index = accepted
        .binary_search_by(|entity| entity.proposal_id().cmp(proposal_id))
        .map_err(|_| fail(RecognizerErrorCode::AdapterBindingMissing))?;
    let node = accepted[index].node().node_id().clone();
    let proposal = partition
        .batch()
        .entity_proposal(proposal_id)
        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
    if graph
        .node(&node)
        .is_none_or(|actual| actual.kind() != proposal.entity_kind_id())
    {
        return Err(fail(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok((proposal, node))
}

fn admit_incomplete_input(
    input: &CoreInput<'_>,
    context: &IncompleteContext<'_, '_>,
    seen_access_ids: &mut BTreeSet<String>,
    admitted: &mut DynamicAdmission,
) -> RecognizerResult<()> {
    let Some(incomplete) = input.incomplete.as_ref() else {
        return Ok(());
    };
    match (input.recognition, incomplete) {
        (CoreRecognition::Legacy(recognition), IncompleteInput::Legacy(source)) => {
            validate_incomplete_owner(input, source.owner, source.context, source.report)?;
            if source.source_partition != recognition.source_partition() {
                return Err(fail(RecognizerErrorCode::AdapterIdentityMismatch));
            }
            let partition = source
                .owner
                .partition(source.source_partition)
                .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
            admitted
                .producer_partitions
                .insert(source.source_partition.into());
            let support = IncompleteSupport {
                context: source.context,
                report: source.report,
                source_handles: source.source_handles,
                evidence: source.evidence,
                graph: context.graph,
                limits: context.limits,
            };
            for fact in &source.facts {
                checkpoint(context.stop)?;
                let accesses = source.report.global_accesses();
                let access_index = accesses
                    .binary_search_by(|access| access.fact_id().cmp(fact.access_id))
                    .map_err(|_| fail(RecognizerErrorCode::AdapterBindingMissing))?;
                let access = &accesses[access_index];
                let (root, root_node) =
                    accepted_project_entity(partition, context.graph, fact.root_proposal_id)?;
                let (caller, caller_node) =
                    accepted_project_entity(partition, context.graph, fact.caller_proposal_id)?;
                let (target, target_node) =
                    accepted_project_entity(partition, context.graph, fact.target_proposal_id)?;
                let references = [
                    fact.root_proposal_id,
                    fact.caller_proposal_id,
                    fact.target_proposal_id,
                ]
                .into_iter()
                .map(|proposal_id| GraphAssertionRef::Producer {
                    partition_id: partition.partition_id().into(),
                    batch_id: partition.batch().batch_id().into(),
                    assertion: GraphLocalAssertion {
                        kind: GraphAssertionKind::Entity,
                        proposal_id: proposal_id.into(),
                    },
                })
                .collect();
                admit_incomplete_fact(
                    &support,
                    context.family,
                    fact,
                    access,
                    IncompleteEndpoints {
                        root,
                        caller,
                        target,
                        root_node,
                        caller_node,
                        target_node,
                        references,
                    },
                    admitted,
                    seen_access_ids,
                )?;
            }
        }
        (CoreRecognition::Assertions(recognition), IncompleteInput::Assertions(source)) => {
            validate_incomplete_owner(input, source.owner, source.context, source.report)?;
            if source.scope != recognition.scope() || source.scope != context.lookup.scope() {
                return Err(fail(RecognizerErrorCode::AdapterIdentityMismatch));
            }
            let support = IncompleteSupport {
                context: source.context,
                report: source.report,
                source_handles: source.source_handles,
                evidence: source.evidence,
                graph: context.graph,
                limits: context.limits,
            };
            for assertion_fact in &source.facts {
                checkpoint(context.stop)?;
                let fact = &assertion_fact.fact;
                let accesses = source.report.global_accesses();
                let access_index = accesses
                    .binary_search_by(|access| access.fact_id().cmp(fact.access_id))
                    .map_err(|_| fail(RecognizerErrorCode::AdapterBindingMissing))?;
                let access = &accesses[access_index];
                let root = crate::source_assertions::entity(
                    context.lookup,
                    source.scope,
                    &assertion_fact.endpoints.root,
                    fact.root_proposal_id,
                    context.stop,
                )?;
                let caller = crate::source_assertions::entity(
                    context.lookup,
                    source.scope,
                    &assertion_fact.endpoints.caller,
                    fact.caller_proposal_id,
                    context.stop,
                )?;
                let target = crate::source_assertions::entity(
                    context.lookup,
                    source.scope,
                    &assertion_fact.endpoints.target,
                    fact.target_proposal_id,
                    context.stop,
                )?;
                for reference in [
                    &assertion_fact.endpoints.root,
                    &assertion_fact.endpoints.caller,
                    &assertion_fact.endpoints.target,
                ] {
                    let GraphAssertionRef::Producer { partition_id, .. } = reference else {
                        return Err(fail(RecognizerErrorCode::AdapterBindingInvalid));
                    };
                    admitted
                        .producer_partitions
                        .insert(partition_id.to_string());
                }
                let references = BTreeSet::from([
                    assertion_fact.endpoints.root.clone(),
                    assertion_fact.endpoints.caller.clone(),
                    assertion_fact.endpoints.target.clone(),
                ]);
                admit_incomplete_fact(
                    &support,
                    context.family,
                    fact,
                    access,
                    IncompleteEndpoints {
                        root: root.proposal(),
                        caller: caller.proposal(),
                        target: target.proposal(),
                        root_node: root.accepted().node().node_id().clone(),
                        caller_node: caller.accepted().node().node_id().clone(),
                        target_node: target.accepted().node().node_id().clone(),
                        references,
                    },
                    admitted,
                    seen_access_ids,
                )?;
            }
        }
        _ => return Err(fail(RecognizerErrorCode::AdapterIdentityMismatch)),
    }
    Ok(())
}

fn validate_incomplete_owner(
    input: &CoreInput<'_>,
    owner: &GraphPartitionSnapshot,
    context: &GenerationContext,
    report: &wow_emmy::function_calls::FunctionCallReport,
) -> RecognizerResult<()> {
    if owner.snapshot().snapshot_id() != input.owner.snapshot().snapshot_id()
        || owner.source_context_id() != input.owner.source_context_id()
        || context.context_id() != input.context.context_id()
        || report.analysis_id() != input.recognition.analyzer_report_id()
    {
        return Err(fail(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    context
        .validate()
        .map_err(|_| fail(RecognizerErrorCode::AdapterIdentityMismatch))?;
    report
        .validate()
        .map_err(|_| fail(RecognizerErrorCode::AdapterFactMismatch))?;
    Ok(())
}

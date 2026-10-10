//! One native caller over retained direct assertions; no service route selection.
use super::{PlatformFixtureSource, TestResult, platform_bundle_with_profile_source, root};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};
use wow_emmy::global_access::GlobalAccessKind;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphCoverageRecord, GraphLocalAssertion,
    GraphPartitionReplacement, GraphPartitionSnapshot, GraphProposalBatch,
};
use wow_project::graph::{
    PlatformGraphProducer, PlatformGraphProvenance, ProjectGraphProvenance, ProjectTocFact,
    ProjectTocFactKind, SOURCE_GRAPH_PARTITION, build_platform_graph_proposal_plan,
};
use wow_project::load::xml_references::{XmlReferenceOrder, XmlReferenceResolution};
use wow_project::load::{XmlElementRole, XmlScriptSource};
use wow_project::{PlatformGraphProfile, ProjectPublisher};
use wow_recognizers::source_scripts::{
    SourceScriptAssertionEndpoints, SourceScriptAssertionFact, SourceScriptAssertionInput,
    SourceScriptFact, SourceScriptSemanticContext, recognize_source_script_assertions,
};
use wow_recognizers::source_state::{
    SOURCE_STATE_PARTITION, SourceStateAssertionEndpoints, SourceStateAssertionFact,
    SourceStateAssertionInput, SourceStateFact, recognize_source_state_assertions,
};
use wow_recognizers::source_state_core::{
    SourceStateCoreAssertionInput, SourceStateCoreFamily, recognize_source_state_core_assertions,
};
use wow_recognizers::{RecognizerErrorCode, source_toc::*, source_xml::*};

fn entity(direct: &PlatformGraphProvenance<'_>, id: &str) -> TestResult<GraphAssertionRef> {
    Ok(direct
        .assertion(&GraphLocalAssertion {
            kind: GraphAssertionKind::Entity,
            proposal_id: id.into(),
        })
        .ok_or("missing native direct entity")?
        .clone())
}

fn admit(
    owner: &GraphPartitionSnapshot,
    batch: GraphProposalBatch,
    coverage: Vec<GraphCoverageRecord>,
    version: &str,
    stop: &AtomicBool,
) -> TestResult<GraphPartitionSnapshot> {
    let id = batch.producer_partition_id().to_owned();
    let entities = batch.entity_proposals().len();
    let candidate = owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: owner
                    .partition(&id)
                    .map(|p| p.partition_digest().into()),
                producer_version: version.into(),
                batch,
                coverage,
            },
            stop,
        )?
        .candidate()
        .clone();
    let partition = candidate
        .partition(&id)
        .ok_or("admitted partition missing")?;
    assert!(partition.report().rejections().is_empty());
    assert_eq!(partition.report().accepted_entities().len(), entities);
    assert!(partition.coverage().iter().all(|c| !c.negative_authority()));
    Ok(candidate)
}

fn prerequisites(
    owner: &GraphPartitionSnapshot,
    partition_id: &str,
    direct: &PlatformGraphProvenance<'_>,
    stop: &AtomicBool,
) -> TestResult<BTreeSet<String>> {
    let partition = owner
        .partition(partition_id)
        .ok_or("prerequisite partition missing")?;
    let records = partition
        .batch()
        .assertion_records()
        .ok_or("derivations missing")?;
    assert_eq!(&records.scope, direct.scope());
    let lookup = owner.producer_lookup(stop)?;
    let mut partitions = BTreeSet::new();
    for derivation in &records.derivations {
        for reference in &derivation.inputs {
            if let GraphAssertionRef::Producer {
                partition_id,
                assertion,
                ..
            } = reference
            {
                match assertion.kind {
                    GraphAssertionKind::Entity => {
                        let resolved = lookup.entity(direct.scope(), reference, stop)?;
                        assert_eq!(&resolved.reference(), reference);
                        assert!(
                            lookup
                                .input_view()
                                .node(resolved.accepted().node().node_id())
                                .is_some()
                        );
                    }
                    GraphAssertionKind::Relation => {
                        assert_eq!(
                            &lookup
                                .relation(direct.scope(), reference, stop)?
                                .reference(),
                            reference
                        );
                    }
                }
                if let Some(original) = direct.assertion(assertion) {
                    assert_eq!(original, reference);
                }
                partitions.insert(partition_id.to_string());
            }
        }
    }
    Ok(partitions)
}

fn state_facts<'a>(
    direct: &'a PlatformGraphProvenance<'_>,
) -> TestResult<Vec<SourceStateAssertionFact<'a>>> {
    let source = direct.source();
    source
        .state_bindings()
        .iter()
        .map(|binding| {
            Ok(SourceStateAssertionFact {
                fact: SourceStateFact {
                    fact_id: &binding.binding_id,
                    access_id: &binding.access_id,
                    root_proposal_id: &binding.root_id,
                    caller_proposal_id: &binding.caller_proposal_id,
                    target_proposal_id: &binding.target_proposal_id,
                    kind: binding.kind,
                    confidence: binding.confidence,
                    source_handle_ids: &binding.source_handle_ids,
                    evidence_ids: &binding.evidence_ids,
                },
                endpoints: SourceStateAssertionEndpoints {
                    root: entity(direct, &binding.root_id)?,
                    caller: entity(direct, &binding.caller_proposal_id)?,
                    target: entity(direct, &binding.target_proposal_id)?,
                },
            })
        })
        .collect()
}

fn script_facts<'a>(
    direct: &'a PlatformGraphProvenance<'_>,
) -> TestResult<Vec<SourceScriptAssertionFact<'a>>> {
    direct
        .source()
        .script_bindings()
        .iter()
        .map(|binding| {
            Ok(SourceScriptAssertionFact {
                fact: SourceScriptFact {
                    fact_id: &binding.binding_id,
                    receiver_proposal_id: &binding.receiver_proposal_id,
                    handler_proposal_id: &binding.handler_proposal_id,
                    semantic_context: binding.semantic_context.as_ref().map(|context| {
                        SourceScriptSemanticContext {
                            context_id: context.context_id(),
                            script_site: context.script_site(),
                            implicit_receiver: context.implicit_receiver(),
                            runtime_dispatch: context.runtime_dispatch(),
                        }
                    }),
                    confidence: binding.confidence,
                    source_handle_ids: &binding.source_handle_ids,
                    evidence_ids: &binding.evidence_ids,
                },
                endpoints: SourceScriptAssertionEndpoints {
                    receiver: entity(direct, &binding.receiver_proposal_id)?,
                    handler: entity(direct, &binding.handler_proposal_id)?,
                },
            })
        })
        .collect()
}

fn xml_bindings(source: &ProjectGraphProvenance) -> TestResult<Vec<SourceXmlScriptBinding<'_>>> {
    source
        .script_bindings()
        .iter()
        .map(|binding| {
            let site = source
                .script_sites()
                .iter()
                .find(|site| site.site_id == binding.site_id)
                .ok_or("script site missing")?;
            assert!(site.binding_ids.contains(&binding.binding_id));
            assert!(site.consumer_id.is_some());
            Ok(SourceXmlScriptBinding {
                binding_id: &binding.binding_id,
                site_id: &binding.site_id,
                script_id: &site.script_id,
                receiver_proposal_id: &binding.receiver_proposal_id,
                handler_proposal_id: &binding.handler_proposal_id,
                handler_kind: binding.handler_kind,
                consumer_occurrence_id: site.consumer_id.as_deref(),
                inherited: site.inherited,
                confidence: binding.confidence,
                semantic_context: binding.semantic_context.as_ref().map(|context| {
                    SourceScriptSemanticContext {
                        context_id: context.context_id(),
                        script_site: context.script_site(),
                        implicit_receiver: context.implicit_receiver(),
                        runtime_dispatch: context.runtime_dispatch(),
                    }
                }),
                source_handle_ids: &binding.source_handle_ids,
                evidence_ids: &binding.evidence_ids,
            })
        })
        .collect()
}

#[test]
fn native_assertion_chain_uses_exact_direct_predecessors() -> TestResult {
    let path = root("platform-native-assertions")?;
    {
        let stop = AtomicBool::new(false);
        let toc = format!(
            "## SavedVariables: StateCoreAccountDB\n## SavedVariablesPerCharacter: StateCoreCharacterDB\n{}",
            include_str!("../../../../../wow-project/tests/data/xml-facts/Fixture.toc")
        );
        let lua = format!(
            "{}\n{}",
            include_str!("../../../../../wow-project/tests/data/xml-facts/defs.lua"),
            include_str!("../../../../tests/data/state-core-pipeline/state.lua")
        );
        let fixture = PlatformFixtureSource {
            toc: toc.as_bytes(),
            lua: lua.as_bytes(),
        };
        let input_path = path.join("input");
        let bundle = platform_bundle_with_profile_source(
            &input_path,
            1,
            true,
            Some(PlatformGraphProfile::PackageProjectionWithRawInventoryV1),
            Some(&fixture),
            &stop,
        )?;
        assert!(!input_path.exists());
        let mut publisher = ProjectPublisher::with_function_call_facts();
        publisher.publish_initial_cancellable(bundle, &stop)?;
        let view = publisher.open_current()?;
        let mut plan = build_platform_graph_proposal_plan(&view, &stop)?;
        let mut direct_owner = GraphPartitionSnapshot::new(
            plan.registry().clone(),
            plan.foundation().clone(),
            plan.scope().source_context_id,
            &stop,
        )?;
        let raw = plan
            .raw_inventory_batch()
            .ok_or("raw prelude missing")?
            .clone();
        direct_owner = admit(
            &direct_owner,
            raw,
            Vec::new(),
            plan.raw_inventory_producer_version(),
            &stop,
        )?;
        for &producer in plan.producer_order() {
            let stage = plan.build_stage(producer, &direct_owner, &stop)?;
            let version = stage.producer_version();
            let (batch, coverage) = stage.into_parts();
            direct_owner = admit(&direct_owner, batch, coverage, version, &stop)?;
        }
        assert_eq!(direct_owner.partitions().len(), 5);
        let direct = plan.finish(&direct_owner, &stop)?;
        let source = direct.source();
        let report = source
            .function_call_report()
            .ok_or("actual access report missing")?;
        assert!(source.state_roots().iter().all(|root| !root.ambiguous));
        for kind in [GlobalAccessKind::Read, GlobalAccessKind::Write] {
            assert!(
                source
                    .state_bindings()
                    .iter()
                    .any(|binding| binding.kind == kind)
            );
        }
        assert!(!source.script_bindings().is_empty());
        let mut owner = direct_owner.clone();
        let state_input = |facts| SourceStateAssertionInput {
            owner: &owner,
            scope: direct.scope(),
            report,
            context: source.context(),
            facts,
            source_handles: source.source_handles(),
            evidence: source.evidence(),
        };
        let mut substituted = state_facts(&direct)?;
        let first = substituted.first_mut().ok_or("state fact missing")?;
        assert_ne!(first.endpoints.root, first.endpoints.caller);
        first.endpoints.root = first.endpoints.caller.clone();
        let before = owner.snapshot().snapshot_id().clone();
        assert_eq!(
            recognize_source_state_assertions(state_input(substituted), &stop)
                .err()
                .ok_or("substitution accepted")?
                .code(),
            RecognizerErrorCode::AdapterFactMismatch
        );
        assert_eq!(owner.snapshot().snapshot_id(), &before);
        assert_eq!(
            recognize_source_state_assertions(
                state_input(state_facts(&direct)?),
                &AtomicBool::new(true)
            )
            .err()
            .ok_or("cancellation accepted")?
            .code(),
            RecognizerErrorCode::Cancelled
        );
        let state = recognize_source_state_assertions(state_input(state_facts(&direct)?), &stop)?;
        assert_eq!(
            state.recognition.receipts().len(),
            source.state_bindings().len()
        );
        for receipt in state.recognition.receipts() {
            let binding = source
                .state_bindings()
                .iter()
                .find(|b| b.binding_id == receipt.binding_id)
                .ok_or("original state binding missing")?;
            let proposal = state
                .batch
                .relation_proposal(&receipt.proposal_id)
                .ok_or("state proposal missing")?;
            assert_eq!(
                proposal.source_handle_ids(),
                binding.source_handle_ids.as_slice()
            );
            assert_eq!(proposal.evidence_ids(), binding.evidence_ids.as_slice());
        }
        let state_count = state.recognition.receipts().len();
        owner = admit(
            &owner,
            state.batch,
            state.coverage,
            env!("CARGO_PKG_VERSION"),
            &stop,
        )?;
        let toc = toc_facts(source.toc_facts());
        let files = source
            .files()
            .iter()
            .map(|file| Ok((file.path.as_str(), entity(&direct, &file.proposal_id)?)))
            .collect::<TestResult<BTreeMap<_, _>>>()?;
        let toc_files = toc
            .iter()
            .filter_map(|fact| match fact.kind {
                SourceTocFactKind::File {
                    path: Some(path), ..
                } => Some(path),
                _ => None,
            })
            .map(|path| {
                Ok((
                    path,
                    files.get(path).ok_or("TOC file assertion missing")?.clone(),
                ))
            })
            .collect::<TestResult<BTreeMap<_, _>>>()?;
        for family in SourceTocFamily::ALL
            .into_iter()
            .chain([SourceTocFamily::SavedVariableRoot])
        {
            let result = recognize_source_toc_assertions(
                SourceTocAssertionInput {
                    owner: &owner,
                    scope: direct.scope(),
                    context: source.context(),
                    facts: &toc,
                    source_files: &toc_files,
                    source_handles: source.source_handles(),
                    evidence: source.evidence(),
                },
                family,
                &stop,
            )?;
            if family == SourceTocFamily::SavedVariableRoot {
                assert!(!result.recognition.receipts.is_empty());
                assert!(!result.batch.entity_proposals().is_empty());
                assert!(
                    result
                        .batch
                        .entity_proposals()
                        .iter()
                        .all(|p| p.entity_kind_id() == "state_root")
                );
            }
            owner = admit(
                &owner,
                result.batch,
                result.coverage,
                env!("CARGO_PKG_VERSION"),
                &stop,
            )?;
        }
        let scripts = recognize_source_script_assertions(
            SourceScriptAssertionInput {
                owner: &owner,
                scope: direct.scope(),
                context: source.context(),
                facts: script_facts(&direct)?,
                source_handles: source.source_handles(),
                evidence: source.evidence(),
            },
            &stop,
        )?;
        assert_eq!(
            scripts.recognition.receipts().len(),
            source.script_bindings().len()
        );
        assert!(!scripts.recognition.receipts().is_empty());
        assert_eq!(scripts.recognition.scope(), direct.scope());
        assert_eq!(
            scripts.recognition.endpoints().len(),
            source.script_bindings().len()
        );
        let script_lookup = owner.producer_lookup(&stop)?;
        for receipt in scripts.recognition.receipts() {
            let binding = source
                .script_bindings()
                .iter()
                .find(|b| b.binding_id == receipt.binding_id)
                .ok_or("original script binding missing")?;
            let endpoints = scripts
                .recognition
                .endpoints()
                .get(&receipt.binding_id)
                .ok_or("script endpoint references missing")?;
            for (reference, proposal_id, producer) in [
                (
                    &endpoints.receiver,
                    binding.receiver_proposal_id.as_str(),
                    PlatformGraphProducer::XmlStructure,
                ),
                (
                    &endpoints.handler,
                    binding.handler_proposal_id.as_str(),
                    PlatformGraphProducer::AnalyzerStructure,
                ),
            ] {
                assert_eq!(reference, &entity(&direct, proposal_id)?);
                let resolved =
                    script_lookup.entity(scripts.recognition.scope(), reference, &stop)?;
                assert_eq!(&resolved.reference(), reference);
                assert_eq!(resolved.partition().partition_id(), producer.partition_id());
                assert_eq!(resolved.proposal().proposal_id(), proposal_id);
                assert!(
                    script_lookup
                        .input_view()
                        .node(resolved.accepted().node().node_id())
                        .is_some()
                );
            }
            let proposal = scripts
                .batch
                .relation_proposal(&receipt.proposal_id)
                .ok_or("script proposal missing")?;
            assert_eq!(proposal.relation_kind_id(), "source_xml_sets_script");
            assert_eq!(
                proposal.source_handle_ids(),
                binding.source_handle_ids.as_slice()
            );
            assert_eq!(proposal.evidence_ids(), binding.evidence_ids.as_slice());
        }
        let script_count = scripts.recognition.receipts().len();
        owner = admit(
            &owner,
            scripts.batch,
            scripts.coverage,
            env!("CARGO_PKG_VERSION"),
            &stop,
        )?;
        let xml = xml_facts(source.xml_facts());
        let xml_bindings = xml_bindings(source)?;
        let mut source_entities = BTreeMap::new();
        for producer in [
            PlatformGraphProducer::XmlStructure,
            PlatformGraphProducer::AnalyzerStructure,
        ] {
            let partition = direct_owner
                .partition(producer.partition_id())
                .ok_or("direct partition missing")?;
            for proposal in partition.batch().entity_proposals() {
                if matches!(
                    proposal.entity_kind_id(),
                    "xml_source_declaration" | "xml_source_handler" | "lua_source_function"
                ) {
                    source_entities.insert(
                        proposal.proposal_id(),
                        entity(&direct, proposal.proposal_id())?,
                    );
                }
            }
        }
        let mut xml_count = 0;
        for family in SourceXmlFamily::ALL {
            let result = recognize_source_xml_assertions(
                SourceXmlAssertionInput {
                    owner: &owner,
                    scope: direct.scope(),
                    context: source.context(),
                    facts: &xml,
                    script_bindings: &xml_bindings,
                    source_files: &files,
                    source_entities: &source_entities,
                    source_handles: source.source_handles(),
                    evidence: source.evidence(),
                },
                family,
                &stop,
            )?;
            if family == SourceXmlFamily::Script {
                assert!(!result.recognition.receipts.is_empty());
            }
            xml_count += result
                .recognition
                .receipts
                .iter()
                .flat_map(|receipt| &receipt.relation_proposal_ids)
                .collect::<BTreeSet<_>>()
                .len();
            owner = admit(
                &owner,
                result.batch,
                result.coverage,
                env!("CARGO_PKG_VERSION"),
                &stop,
            )?;
        }
        let mut core_counts = Vec::new();
        for (family, kind) in [
            (SourceStateCoreFamily::Read, "source_reads_state"),
            (SourceStateCoreFamily::Write, "source_writes_state"),
        ] {
            let result = recognize_source_state_core_assertions(
                SourceStateCoreAssertionInput {
                    owner: &owner,
                    context: source.context(),
                    recognition: &state.recognition,
                },
                family,
                &stop,
            )?;
            assert!(!result.recognition.receipts.is_empty());
            let relation_ids = result
                .recognition
                .receipts
                .iter()
                .flat_map(|receipt| &receipt.relation_proposal_ids)
                .collect::<BTreeSet<_>>();
            assert!(!relation_ids.is_empty());
            for id in &relation_ids {
                assert_eq!(
                    result
                        .batch
                        .relation_proposal(id)
                        .ok_or("core proposal missing")?
                        .relation_kind_id(),
                    kind
                );
            }
            assert!(
                result
                    .batch
                    .entity_proposals()
                    .iter()
                    .all(|p| p.entity_kind_id() == "state_path")
            );
            let relation_count = relation_ids.len();
            core_counts.push((kind, relation_count));
            owner = admit(
                &owner,
                result.batch,
                result.coverage,
                env!("CARGO_PKG_VERSION"),
                &stop,
            )?;
            assert_eq!(
                owner
                    .partition(family.partition_id())
                    .ok_or("core partition missing")?
                    .report()
                    .accepted_relations()
                    .len(),
                relation_count
            );
            let inputs = prerequisites(&owner, family.partition_id(), &direct, &stop)?;
            for partition in [
                SOURCE_STATE_PARTITION,
                PlatformGraphProducer::TocLoad.partition_id(),
                PlatformGraphProducer::AnalyzerStructure.partition_id(),
            ] {
                assert!(inputs.contains(partition));
            }
        }
        owner.validate(&stop)?;
        assert!(owner.partition(SOURCE_GRAPH_PARTITION).is_none());
        assert_eq!(direct_owner.partitions().len(), 5);
        println!(
            "native assertion chain: access={state_count}, sets_script={script_count}, xml={xml_count}, core={core_counts:?}"
        );
    }
    std::fs::remove_dir_all(&path)?;
    Ok(())
}

// Borrowed enum/field conversions preserve the existing service projections.
fn selection(value: wow_project::load::LoadSelection) -> SourceTocSelection {
    match value {
        wow_project::load::LoadSelection::Included => SourceTocSelection::Included,
        wow_project::load::LoadSelection::Excluded => SourceTocSelection::Excluded,
        wow_project::load::LoadSelection::Unresolved => SourceTocSelection::Unresolved,
    }
}

fn load_state(value: wow_project::load::TocLoadOnDemandState) -> SourceTocLoadState {
    match value {
        wow_project::load::TocLoadOnDemandState::NotDeclared => SourceTocLoadState::NotDeclared,
        wow_project::load::TocLoadOnDemandState::False => SourceTocLoadState::False,
        wow_project::load::TocLoadOnDemandState::True => SourceTocLoadState::True,
        wow_project::load::TocLoadOnDemandState::Unknown => SourceTocLoadState::Unknown,
    }
}

fn scope(value: wow_project::load::TocSavedVariableScope) -> SourceTocScope {
    match value {
        wow_project::load::TocSavedVariableScope::Account => SourceTocScope::Account,
        wow_project::load::TocSavedVariableScope::Character => SourceTocScope::Character,
    }
}

fn toc_facts<'a>(facts: &'a [ProjectTocFact]) -> Vec<SourceTocFact<'a>> {
    let mut output = Vec::new();
    for fact in facts {
        let kind = match &fact.kind {
            ProjectTocFactKind::Package {
                source_complete, ..
            } => SourceTocFactKind::Package {
                source_complete: *source_complete,
            },
            ProjectTocFactKind::File { path, repeated, .. } => SourceTocFactKind::File {
                path: path.as_deref(),
                repeated: *repeated,
            },
            ProjectTocFactKind::Dependency {
                name,
                dependency_kind,
                resolved_package,
                ..
            } => SourceTocFactKind::Dependency {
                name,
                optional: matches!(
                    dependency_kind,
                    wow_project::load::TocDependencyKind::Optional
                ),
                resolved_package: resolved_package.as_deref(),
            },
            ProjectTocFactKind::LoadOnDemand {
                effective_state,
                conflicting,
                ..
            } => SourceTocFactKind::LoadOnDemand {
                state: load_state(*effective_state),
                conflicting: *conflicting,
            },
            ProjectTocFactKind::SavedVariable {
                name,
                scope: variable_scope,
                state,
                ..
            } => SourceTocFactKind::SavedVariable {
                name,
                scope: scope(*variable_scope),
                declared: matches!(state, wow_project::load::TocSavedVariableState::Declared),
            },
        };
        output.push(SourceTocFact {
            fact_id: fact.fact_id.as_str(),
            context_id: fact.context_id,
            package: fact.package.as_deref(),
            selected_toc: fact.selected_toc.as_str(),
            flavor: fact.flavor.as_str(),
            ordinal: fact.ordinal,
            selection: selection(fact.selection),
            content_digest: fact.content_digest,
            span: fact.span,
            source_handle_id: fact.source_handle_id,
            evidence_id: fact.evidence_id,
            kind,
        });
    }
    output
}

fn element_role(value: XmlElementRole) -> SourceXmlElementRole {
    match value {
        XmlElementRole::Ui => SourceXmlElementRole::Ui,
        XmlElementRole::Include => SourceXmlElementRole::Include,
        XmlElementRole::Script => SourceXmlElementRole::Script,
        XmlElementRole::Scripts => SourceXmlElementRole::Scripts,
        XmlElementRole::ScriptBinding => SourceXmlElementRole::ScriptBinding,
        XmlElementRole::Element => SourceXmlElementRole::Element,
        XmlElementRole::UnknownNamespace => SourceXmlElementRole::UnknownNamespace,
    }
}

/// Absent means the attribute was never spelled; it is never a default value.
fn template_state(value: Option<bool>) -> SourceXmlTemplateState {
    match value {
        None => SourceXmlTemplateState::Absent,
        Some(false) => SourceXmlTemplateState::False,
        Some(true) => SourceXmlTemplateState::True,
    }
}

fn script_source(value: XmlScriptSource) -> SourceXmlScriptSource {
    match value {
        XmlScriptSource::ExternalFile => SourceXmlScriptSource::ExternalFile,
        XmlScriptSource::ReferenceOnly => SourceXmlScriptSource::ReferenceOnly,
        XmlScriptSource::InlineBody => SourceXmlScriptSource::InlineBody,
        XmlScriptSource::Unresolved => SourceXmlScriptSource::Unresolved,
    }
}

fn reference_order(value: Option<XmlReferenceOrder>) -> Option<SourceXmlReferenceOrder> {
    value.map(|order| match order {
        XmlReferenceOrder::TargetBeforeSource => SourceXmlReferenceOrder::TargetBeforeSource,
        XmlReferenceOrder::TargetAfterSource => SourceXmlReferenceOrder::TargetAfterSource,
        XmlReferenceOrder::SelfReference => SourceXmlReferenceOrder::SelfReference,
        XmlReferenceOrder::RepeatedLoad => SourceXmlReferenceOrder::RepeatedLoad,
        XmlReferenceOrder::Unrecorded => SourceXmlReferenceOrder::Unrecorded,
    })
}

/// Direct, linear projection of the retained facts. Facts borrow from the owner
/// records; nothing is reparsed, reselected, deduplicated or reordered. Inheritance
/// ordinals are the loader ordinals for the explicit `inherits` list; declaration,
/// parent and script facts take the producer position in the fact_id order, which
/// is deterministic across runs and carries no source-order claim.
fn xml_facts(facts: &[wow_project::graph::ProjectXmlFact]) -> Vec<SourceXmlFact<'_>> {
    facts
        .iter()
        .enumerate()
        .map(|(index, fact)| {
            let ordinal = match &fact.kind {
                wow_project::graph::ProjectXmlFactKind::Inheritance { ordinal, .. }
                | wow_project::graph::ProjectXmlFactKind::InheritanceUnresolved {
                    ordinal, ..
                } => *ordinal,
                _ => index as u64,
            };
            SourceXmlFact {
                fact_id: fact.fact_id.as_str(),
                context_id: fact.context_id,
                selected_toc: fact.scope.selected_toc.as_str(),
                flavor: fact.scope.flavor.as_str(),
                package: fact.scope.package.as_deref(),
                document: fact.document.as_str(),
                occurrence_id: fact.occurrence_id.as_str(),
                ordinal,
                content_digest: fact.content_digest,
                span: fact.span,
                source_handle_id: fact.source_handle_id,
                evidence_id: fact.evidence_id,
                kind: convert_kind(fact),
            }
        })
        .collect()
}

fn convert_kind(fact: &wow_project::graph::ProjectXmlFact) -> SourceXmlFactKind<'_> {
    match &fact.kind {
        wow_project::graph::ProjectXmlFactKind::Declaration { role, declaration } => {
            SourceXmlFactKind::Declaration {
                role: element_role(*role),
                element_name: fact.element_name.as_str(),
                name: declaration.name.as_deref(),
                virtual_template: template_state(declaration.virtual_template),
                intrinsic: template_state(declaration.intrinsic),
                mixin_names: &declaration.mixin_names,
                valid_declaration: declaration.valid_declaration,
                parent_occurrence_id: declaration.parent_occurrence_id.as_deref(),
            }
        }
        wow_project::graph::ProjectXmlFactKind::Parent {
            reference_id,
            name,
            resolution,
            order,
            cycle_id,
            ..
        } => SourceXmlFactKind::Parent {
            reference_id: reference_id.as_str(),
            name: name.as_str(),
            resolution: parent_resolution(resolution),
            order: reference_order(*order),
            cycle_id: cycle_id.as_deref(),
        },
        wow_project::graph::ProjectXmlFactKind::Inheritance {
            reference_id,
            target_occurrence_id,
            order,
            cycle_id,
            ..
        } => SourceXmlFactKind::Inheritance {
            reference_id: reference_id.as_str(),
            target_occurrence_id: target_occurrence_id.as_str(),
            order: reference_order(*order),
            cycle_id: cycle_id.as_deref(),
        },
        wow_project::graph::ProjectXmlFactKind::InheritanceUnresolved {
            reference_id,
            name,
            ..
        } => SourceXmlFactKind::InheritanceUnresolved {
            reference_id: reference_id.as_str(),
            name: name.as_str(),
        },
        wow_project::graph::ProjectXmlFactKind::Script {
            script_name,
            source_kind,
            owner_occurrence_id,
            inherit,
            intrinsic_order,
            file_reference,
            function_reference,
            method_reference,
            ..
        } => SourceXmlFactKind::Script {
            reference_id: fact.fact_id.as_str(),
            script_name: script_name.as_str(),
            source_kind: script_source(*source_kind),
            owner_occurrence_id: owner_occurrence_id.as_deref(),
            inherit: inherit.as_deref(),
            intrinsic_order: intrinsic_order.as_deref(),
            file_reference: file_reference.as_deref(),
            function_reference: function_reference.as_deref(),
            method_reference: method_reference.as_deref(),
        },
    }
}

/// Preserve the loader's verdict; an invalid target never becomes Unique.
fn parent_resolution(value: &XmlReferenceResolution) -> SourceXmlParentResolution<'_> {
    match value {
        XmlReferenceResolution::UniqueLocalDeclaration { declaration_id } => {
            SourceXmlParentResolution::Unique {
                target_occurrence_id: declaration_id,
            }
        }
        XmlReferenceResolution::AmbiguousName { name_group } => {
            SourceXmlParentResolution::Ambiguous {
                name_group: name_group.as_str(),
            }
        }
        XmlReferenceResolution::NotInCapturedScope => SourceXmlParentResolution::NotInCapturedScope,
        XmlReferenceResolution::DynamicName => SourceXmlParentResolution::DynamicName,
        XmlReferenceResolution::UnsupportedName => SourceXmlParentResolution::UnsupportedName,
        XmlReferenceResolution::InvalidSource => SourceXmlParentResolution::InvalidSource,
        XmlReferenceResolution::InvalidTarget { declaration_id } => {
            SourceXmlParentResolution::InvalidTarget {
                target_occurrence_id: declaration_id.as_str(),
            }
        }
    }
}

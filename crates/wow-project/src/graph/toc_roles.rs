//! Selected TOC roles from retained facts and native package receipts. Ordering
//! is lexical source topology, never a claim about client execution.
use super::producer_budget::ProducerBudget;
use super::*;
use crate::load::{
    PACKAGE_MAIN_NAMESPACE_ROOT, ProjectPackageLoadPhase, ProjectPackageLoadUnit,
    ProjectPackageReachability, TocLoadOnDemandState,
};
use serde::ser::SerializeMap;
use wow_core::{CanonicalResult, ContentDigest, CoverageId};

// Native proposal IDs use this exact prefix followed by a fixed SHA-256 hex ID.
const ID_ENCODING: &str =
    "toc-role:0000000000000000000000000000000000000000000000000000000000000000";
const MAX_DIRECT_ROWS: usize = 200_000;

pub(super) fn append(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    entities: &mut Vec<EntityDraft>,
    relations: &mut Vec<RelationDraft>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    let mut rows = entities
        .len()
        .checked_add(relations.len())
        .filter(|count| *count <= MAX_DIRECT_ROWS)
        .ok_or_else(exhausted)?;
    let config = project.configuration();
    let packages = config.package_load_plan().ok_or_else(invalid)?;
    if config.project_kind() != ProjectKind::BlizzardUiPlatformSource
        || source.context() != project.snapshot().generation_context()
        || source.project_snapshot_id != project.snapshot_id()
        || source.analyzer_snapshot_id != project.analyzer_snapshot_id()
        || source.package_load_plan.as_ref().map(|plan| plan.digest()) != Some(packages.digest())
    {
        return Err(invalid());
    }

    let mut files = BTreeMap::new();
    for file in source.files() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(
            &("toc-role-file-index", &file.path, &file.proposal_id),
            stop,
        )?;
        if files.insert(file.path.as_str(), file).is_some() {
            return Err(invalid());
        }
    }
    let mut package_sources = BTreeMap::new();
    for package in source.packages() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(&("toc-role-package-index", package), stop)?;
        if package_sources
            .insert(package.package.as_str(), package)
            .is_some()
        {
            return Err(invalid());
        }
    }
    if package_sources.len() != packages.packages().len() {
        return Err(invalid());
    }
    let mut groups = BTreeMap::new();
    for group in packages.order_groups() {
        for package in &group.packages {
            crate::analyzer::checkpoint(stop)?;
            budget.charge_serialized(&("toc-role-order-index", package, group.ordinal), stop)?;
            if groups.insert(package.as_str(), group.ordinal).is_some() {
                return Err(invalid());
            }
        }
    }
    if groups.len() != packages.packages().len() {
        return Err(invalid());
    }

    let mut package_facts = BTreeMap::new();
    let mut file_facts = BTreeMap::new();
    let mut policy_facts = BTreeMap::new();
    for fact in source.toc_facts() {
        crate::analyzer::checkpoint(stop)?;
        let package = fact.package.as_deref().ok_or_else(invalid)?;
        let plan = packages.package_plan(package).ok_or_else(invalid)?;
        if !qualified(&fact.selected_toc, package, plan.selected_toc()) {
            return Err(invalid());
        }
        validate_support(source, fact, config.selected_profile().flavor_id())?;
        if !matches!(
            fact.kind,
            ProjectTocFactKind::Package { .. }
                | ProjectTocFactKind::File { .. }
                | ProjectTocFactKind::LoadOnDemand { .. }
        ) {
            continue;
        }
        budget.charge_serialized(
            &("toc-role-fact-index", package, fact.ordinal, &fact.fact_id),
            stop,
        )?;
        let duplicate = match &fact.kind {
            ProjectTocFactKind::Package { .. } => package_facts.insert(package, fact).is_some(),
            ProjectTocFactKind::File { .. } => {
                file_facts.insert((package, fact.ordinal), fact).is_some()
            }
            ProjectTocFactKind::LoadOnDemand { .. } => {
                policy_facts.insert((package, fact.ordinal), fact).is_some()
            }
            _ => return Err(invalid()),
        };
        if duplicate {
            return Err(invalid());
        }
    }
    if package_facts.len() != packages.packages().len() {
        return Err(invalid());
    }

    let mut units = BTreeMap::new();
    for unit in packages.units() {
        crate::analyzer::checkpoint(stop)?;
        let plan = packages.package_plan(&unit.package).ok_or_else(invalid)?;
        if unit.document != plan.selected_toc() {
            continue;
        }
        budget.charge_serialized(&("toc-role-unit-index", unit), stop)?;
        if units
            .insert((unit.package.as_str(), unit.source_ordinal), unit)
            .is_some()
        {
            return Err(invalid());
        }
    }

    let mut seen_files = 0_usize;
    let mut seen_policies = 0_usize;
    let mut seen_units = 0_usize;
    for node in packages.packages() {
        crate::analyzer::checkpoint(stop)?;
        let package = node.package.as_str();
        let plan = packages.package_plan(package).ok_or_else(invalid)?;
        let package_source = package_sources.get(package).copied().ok_or_else(invalid)?;
        let package_fact = package_facts.get(package).copied().ok_or_else(invalid)?;
        let order_group = *groups.get(package).ok_or_else(invalid)?;
        let ProjectTocFactKind::Package {
            plan_digest,
            target_interface,
            source_complete,
        } = &package_fact.kind
        else {
            return Err(invalid());
        };
        let toc_file = files
            .get(package_fact.selected_toc.as_str())
            .copied()
            .ok_or_else(invalid)?;
        if *plan_digest != plan.digest()
            || *plan_digest != node.selected_plan_digest
            || plan.selected_toc() != node.selected_toc
            || *target_interface != config.selected_profile().interface()
            || *source_complete != plan.external_files_complete()
            || package_fact.ordinal != 0
            || package_fact.selection != LoadSelection::Included
            || package_fact.span != SourceSpan::whole_file()
            || package_source.selected_toc != package_fact.selected_toc
            || package_source.source_handle_id != package_fact.source_handle_id
            || package_source.evidence_id != package_fact.evidence_id
            || package_source.order_group != order_group
            || package_source.reachability != node.reachability
            || package_source.phase != node.phase
            || toc_file.content_digest != package_fact.content_digest
        {
            return Err(invalid());
        }
        let mut selected = None;
        for variant in &node.variants {
            crate::analyzer::checkpoint(stop)?;
            if variant.selected && selected.replace(variant).is_some() {
                return Err(invalid());
            }
        }
        let selected = selected.ok_or_else(invalid)?;
        if selected.package != package
            || selected.toc != node.selected_toc
            || selected.content_digest != toc_file.content_digest
            || selected.byte_length != toc_file.byte_length
        {
            return Err(invalid());
        }

        let manifest = entity(
            "source_toc_manifest",
            &[
                ("document", Value::String(&package_fact.selected_toc)),
                ("plan_digest", Value::Digest(*plan_digest)),
            ],
            package_fact,
            entities,
            &mut rows,
            budget,
            stop,
        )?;
        let variant = entity(
            "source_toc_variant",
            &[
                ("document", Value::String(&package_fact.selected_toc)),
                ("flavor", Value::Identifier(&package_fact.flavor)),
                ("selected_root", Value::Boolean(node.selected_root)),
                (
                    "load_on_demand",
                    Value::Identifier(lod(node.load_on_demand)),
                ),
                ("static_phase", Value::Identifier(phase(node.phase))),
                ("order_group", Value::Integer(integer(order_group)?)),
                (
                    "reachability",
                    Value::Identifier(reachability(node.reachability)),
                ),
            ],
            package_fact,
            entities,
            &mut rows,
            budget,
            stop,
        )?;
        relation(
            "source_package_selects_toc",
            &package_source.proposal_id,
            &manifest,
            GraphConfidence::Derived,
            package_fact,
            None,
            relations,
            &mut rows,
            budget,
            stop,
        )?;
        relation(
            "source_toc_defines_variant",
            &manifest,
            &variant,
            GraphConfidence::Derived,
            package_fact,
            None,
            relations,
            &mut rows,
            budget,
            stop,
        )?;

        let mut repetitions = BTreeMap::new();
        for record in plan.records() {
            crate::analyzer::checkpoint(stop)?;
            if record.document == plan.selected_toc()
                && matches!(
                    record.kind,
                    LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
                )
                && let Some(target) = record.target.as_deref()
            {
                budget.charge_serialized(&("toc-role-repetition-index", package, target), stop)?;
                let count = repetitions.entry(target).or_insert(0_usize);
                *count = count.checked_add(1).ok_or_else(exhausted)?;
            }
        }
        let mut previous: Option<(Box<str>, &ProjectTocFact)> = None;
        for record in plan.records() {
            crate::analyzer::checkpoint(stop)?;
            if record.document != plan.selected_toc() {
                continue;
            }
            let span = SourceSpan::byte_range(record.byte_start, record.byte_end)
                .map_err(|_| invalid())?;
            if matches!(
                record.kind,
                LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
            ) {
                let fact = file_facts
                    .get(&(package, record.ordinal))
                    .copied()
                    .ok_or_else(invalid)?;
                let ProjectTocFactKind::File {
                    path,
                    declared_target,
                    file_kind,
                    bootstrap,
                    conditions,
                    repeated,
                } = &fact.kind
                else {
                    return Err(invalid());
                };
                let target_matches = match (path.as_deref(), record.target.as_deref()) {
                    (None, None) => true,
                    (Some(path), Some(target)) => qualified(path, package, target),
                    _ => false,
                };
                if fact.span != span
                    || fact.selection != record.selection
                    || *file_kind != record.kind
                    || *bootstrap != record.bootstrap
                    || *declared_target != record.declared_target
                    || *conditions != record.conditions
                    || !target_matches
                    || *repeated
                        != record.target.as_deref().is_some_and(|target| {
                            repetitions.get(target).is_some_and(|count| *count > 1)
                        })
                {
                    return Err(invalid());
                }
                let unit = units.get(&(package, record.ordinal)).copied();
                let static_phase = if record.bootstrap {
                    ProjectPackageLoadPhase::Bootstrap
                } else {
                    node.phase
                };
                if record.selection == LoadSelection::Included && record.target.is_some() {
                    let unit = unit.ok_or_else(invalid)?;
                    if unit.document != record.document
                        || unit.target != *record.target.as_ref().ok_or_else(invalid)?
                        || unit.kind != record.kind
                        || unit.bootstrap != record.bootstrap
                        || unit.order_group != order_group
                        || unit.reachability != node.reachability
                        || unit.phase != static_phase
                    {
                        return Err(invalid());
                    }
                    seen_units = seen_units.checked_add(1).ok_or_else(exhausted)?;
                } else if unit.is_some() {
                    return Err(invalid());
                }
                let witness = unit.map_or(UnitWitness::NotAdmitted, UnitWitness::Admitted);
                budget.charge_serialized(&("toc-role-witness", conditions, &witness), stop)?;
                let conditions_digest = crate::identity::canonical_digest(
                    "wow-project/platform-toc-conditions/1",
                    conditions,
                    ProjectPhase::View,
                )?;
                let unit_digest = crate::identity::canonical_digest(
                    "wow-project/platform-toc-load-unit-witness/1",
                    &witness,
                    ProjectPhase::View,
                )?;
                crate::analyzer::checkpoint(stop)?;
                let occurrence = entity(
                    "source_toc_load_occurrence",
                    &[
                        ("fact_id", Value::String(&fact.fact_id)),
                        ("document", Value::String(&fact.selected_toc)),
                        ("ordinal", Value::Integer(integer(fact.ordinal)?)),
                        (
                            "file_kind",
                            Value::Identifier(if record.kind == LoadRecordKind::LuaFile {
                                "lua_file"
                            } else {
                                "xml_file"
                            }),
                        ),
                        ("selection", Value::Identifier(selection(fact.selection))),
                        ("conditions_digest", Value::Digest(conditions_digest)),
                        ("repeated", Value::Boolean(*repeated)),
                        ("bootstrap", Value::Boolean(*bootstrap)),
                        ("load_unit_witness", Value::Digest(unit_digest)),
                        ("static_phase", Value::Identifier(phase(static_phase))),
                        ("order_group", Value::Integer(integer(order_group)?)),
                        (
                            "reachability",
                            Value::Identifier(reachability(node.reachability)),
                        ),
                    ],
                    fact,
                    entities,
                    &mut rows,
                    budget,
                    stop,
                )?;
                relation(
                    "source_toc_contains_occurrence",
                    &variant,
                    &occurrence,
                    GraphConfidence::Derived,
                    fact,
                    None,
                    relations,
                    &mut rows,
                    budget,
                    stop,
                )?;
                if let (Some(unit), Some(path)) = (unit, path.as_deref())
                    && unit.reachability != ProjectPackageReachability::Unreachable
                    && let Some(file) = files.get(path).copied()
                {
                    relation(
                        "source_toc_occurrence_loads",
                        &occurrence,
                        &file.proposal_id,
                        if unit.reachability == ProjectPackageReachability::Reachable {
                            GraphConfidence::Derived
                        } else {
                            GraphConfidence::Possible
                        },
                        fact,
                        None,
                        relations,
                        &mut rows,
                        budget,
                        stop,
                    )?;
                }
                if let Some((prior, prior_fact)) = previous.as_ref() {
                    if prior_fact.ordinal >= fact.ordinal {
                        return Err(invalid());
                    }
                    relation(
                        "source_toc_occurs_before",
                        prior,
                        &occurrence,
                        GraphConfidence::Derived,
                        prior_fact,
                        Some(fact),
                        relations,
                        &mut rows,
                        budget,
                        stop,
                    )?;
                }
                previous = Some((occurrence, fact));
                seen_files = seen_files.checked_add(1).ok_or_else(exhausted)?;
            }
            if let Some(metadata) = record
                .metadata
                .as_ref()
                .filter(|metadata| metadata.key == "loadondemand")
            {
                let fact = policy_facts
                    .get(&(package, record.ordinal))
                    .copied()
                    .ok_or_else(invalid)?;
                let ProjectTocFactKind::LoadOnDemand {
                    value,
                    declared_state,
                    effective_state,
                    conflicting,
                } = &fact.kind
                else {
                    return Err(invalid());
                };
                if fact.span != span
                    || fact.selection != record.selection
                    || *value != metadata.value
                    || *effective_state != node.load_on_demand
                {
                    return Err(invalid());
                }
                let policy = entity(
                    "source_toc_load_policy",
                    &[
                        ("fact_id", Value::String(&fact.fact_id)),
                        ("selection", Value::Identifier(selection(fact.selection))),
                        ("declared_state", Value::Identifier(lod(*declared_state))),
                        ("effective_state", Value::Identifier(lod(*effective_state))),
                        ("conflicting", Value::Boolean(*conflicting)),
                    ],
                    fact,
                    entities,
                    &mut rows,
                    budget,
                    stop,
                )?;
                relation(
                    "source_toc_defines_load_policy",
                    &variant,
                    &policy,
                    GraphConfidence::Derived,
                    fact,
                    None,
                    relations,
                    &mut rows,
                    budget,
                    stop,
                )?;
                seen_policies = seen_policies.checked_add(1).ok_or_else(exhausted)?;
            }
        }
    }
    if seen_files != file_facts.len()
        || seen_policies != policy_facts.len()
        || seen_units != units.len()
    {
        return Err(invalid());
    }
    crate::analyzer::checkpoint(stop)
}

fn qualified(path: &str, package: &str, local: &str) -> bool {
    path.strip_prefix(PACKAGE_MAIN_NAMESPACE_ROOT)
        .and_then(|path| path.strip_prefix('/'))
        .and_then(|path| path.split_once('/'))
        == Some((package, local))
}

fn validate_support(
    source: &ProjectGraphProvenance,
    fact: &ProjectTocFact,
    flavor: &str,
) -> ProjectResult<()> {
    let handle = source
        .source_handles()
        .get(&fact.source_handle_id)
        .ok_or_else(invalid)?;
    let evidence = source
        .evidence()
        .get(&fact.evidence_id)
        .ok_or_else(invalid)?;
    if fact.context_id != source.context().context_id()
        || fact.flavor != flavor
        || handle.handle_id() != fact.source_handle_id
        || handle.path().as_str() != fact.selected_toc
        || handle.span() != fact.span
        || *handle.content_digest() != fact.content_digest
        || evidence.evidence_id() != fact.evidence_id
        || evidence.context_id() != fact.context_id
        || evidence.source_handle_ids() != [fact.source_handle_id]
        || evidence.provenance() != ProvenanceClass::ProjectSource
        || evidence.claim_scope() != ClaimScope::SourceObservation
        || evidence.confidence() != EvidenceConfidence::Proven
    {
        return Err(invalid());
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(tag = "state", content = "unit", rename_all = "snake_case")]
enum UnitWitness<'a> {
    NotAdmitted,
    Admitted(&'a ProjectPackageLoadUnit),
}

// Borrowed native proposal framing allows exact charging before owned copies.
#[derive(Clone, Copy, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum Value<'a> {
    Boolean(bool),
    Integer(i64),
    String(&'a str),
    Identifier(&'a str),
    #[serde(rename = "string")]
    Digest(ContentDigest<CanonicalResult>),
}
struct Fields<'a>(&'a [(&'a str, Value<'a>)]);
impl Serialize for Fields<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (field, value) in self.0 {
            map.serialize_entry(field, value)?;
        }
        map.end()
    }
}
#[derive(Serialize)]
struct EntityMetadata<'a> {
    proposal_id: &'a str,
    entity_kind_id: &'a str,
    semantic_key: Fields<'a>,
    confidence: GraphConfidence,
    source_handle_ids: &'a [StableHandleId],
    evidence_ids: &'a [EvidenceId],
    coverage_ids: &'a [CoverageId],
}
#[derive(Serialize)]
struct EntityDraftMetadata<'a> {
    producer: PlatformGraphProducer,
    proposal: EntityMetadata<'a>,
}
fn entity(
    kind: &str,
    fields: &[(&str, Value<'_>)],
    fact: &ProjectTocFact,
    output: &mut Vec<EntityDraft>,
    rows: &mut usize,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Box<str>> {
    reserve_row(rows)?;
    let handles = [fact.source_handle_id];
    let evidence = [fact.evidence_id];
    budget.charge_serialized(
        &EntityDraftMetadata {
            producer: PlatformGraphProducer::TocLoad,
            proposal: EntityMetadata {
                proposal_id: ID_ENCODING,
                entity_kind_id: kind,
                semantic_key: Fields(fields),
                confidence: GraphConfidence::Derived,
                source_handle_ids: &handles,
                evidence_ids: &evidence,
                coverage_ids: &[],
            },
        },
        stop,
    )?;
    budget.charge_serialized(&("toc-role-address", ID_ENCODING), stop)?;
    let id = crate::identity::canonical_id(
        "toc-role:",
        "wow-project/platform-toc-role-entity/1",
        &(kind, Fields(fields)),
        ProjectPhase::View,
    )?;
    let keys = fields
        .iter()
        .map(|(field, value)| {
            let value = match value {
                Value::Boolean(value) => GraphProposalValue::Boolean(*value),
                Value::Integer(value) => GraphProposalValue::Integer(*value),
                Value::String(value) => GraphProposalValue::String((*value).into()),
                Value::Identifier(value) => GraphProposalValue::Identifier((*value).into()),
                Value::Digest(value) => GraphProposalValue::String(value.canonical().into()),
            };
            ((*field).into(), value)
        })
        .collect();
    let draft = EntityDraft::new(
        PlatformGraphProducer::TocLoad,
        GraphEntityProposal::new(
            id.clone(),
            kind,
            keys,
            GraphConfidence::Derived,
            handles.to_vec(),
            evidence.to_vec(),
            Vec::new(),
        )
        .map_err(platform_producers::graph_error)?,
    );
    crate::analyzer::checkpoint(stop)?;
    output.push(draft);
    Ok(id)
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum Endpoint<'a> {
    Proposed(&'a str),
}
#[derive(Serialize)]
struct RelationInputMetadata<'a> {
    source: Endpoint<'a>,
    target: Endpoint<'a>,
    confidence: GraphConfidence,
    source_handle_ids: &'a [StableHandleId],
    evidence_ids: &'a [EvidenceId],
    coverage_ids: &'a [CoverageId],
}
#[derive(Serialize)]
struct RelationDraftMetadata<'a> {
    producer: PlatformGraphProducer,
    proposal_id: &'a str,
    relation_kind_id: &'a str,
    input: RelationInputMetadata<'a>,
}
#[allow(clippy::too_many_arguments)]
fn relation(
    kind: &str,
    from: &str,
    to: &str,
    confidence: GraphConfidence,
    fact: &ProjectTocFact,
    other: Option<&ProjectTocFact>,
    output: &mut Vec<RelationDraft>,
    rows: &mut usize,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    reserve_row(rows)?;
    let other = other.unwrap_or(fact);
    let mut handles = [fact.source_handle_id, other.source_handle_id];
    let mut evidence = [fact.evidence_id, other.evidence_id];
    handles.sort_unstable();
    evidence.sort_unstable();
    let handles = &handles[..if handles[0] == handles[1] { 1 } else { 2 }];
    let evidence = &evidence[..if evidence[0] == evidence[1] { 1 } else { 2 }];
    budget.charge_serialized(
        &RelationDraftMetadata {
            producer: PlatformGraphProducer::TocLoad,
            proposal_id: ID_ENCODING,
            relation_kind_id: kind,
            input: RelationInputMetadata {
                source: Endpoint::Proposed(from),
                target: Endpoint::Proposed(to),
                confidence,
                source_handle_ids: handles,
                evidence_ids: evidence,
                coverage_ids: &[],
            },
        },
        stop,
    )?;
    let id = crate::identity::canonical_id(
        "toc-role:",
        "wow-project/platform-toc-role-relation/1",
        &(kind, from, to),
        ProjectPhase::View,
    )?;
    let draft = RelationDraft::new(
        PlatformGraphProducer::TocLoad,
        id,
        kind,
        GraphRelationProposalInput {
            source: GraphProposalEndpoint::Proposed(from.into()),
            target: GraphProposalEndpoint::Proposed(to.into()),
            confidence,
            source_handle_ids: handles.to_vec(),
            evidence_ids: evidence.to_vec(),
            coverage_ids: Vec::new(),
        },
    )
    .map_err(platform_producers::graph_error)?;
    crate::analyzer::checkpoint(stop)?;
    output.push(draft);
    Ok(())
}

fn reserve_row(rows: &mut usize) -> ProjectResult<()> {
    *rows = rows
        .checked_add(1)
        .filter(|count| *count <= MAX_DIRECT_ROWS)
        .ok_or_else(exhausted)?;
    Ok(())
}

fn integer(value: u64) -> ProjectResult<i64> {
    i64::try_from(value).map_err(|_| invalid())
}
fn selection(value: LoadSelection) -> &'static str {
    match value {
        LoadSelection::Included => "included",
        LoadSelection::Excluded => "excluded",
        LoadSelection::Unresolved => "unresolved",
    }
}
fn lod(value: TocLoadOnDemandState) -> &'static str {
    match value {
        TocLoadOnDemandState::NotDeclared => "not_declared",
        TocLoadOnDemandState::False => "false",
        TocLoadOnDemandState::True => "true",
        TocLoadOnDemandState::Unknown => "unknown",
    }
}
fn phase(value: ProjectPackageLoadPhase) -> &'static str {
    match value {
        ProjectPackageLoadPhase::DependencyPrerequisite => "dependency_prerequisite",
        ProjectPackageLoadPhase::Bootstrap => "bootstrap",
        ProjectPackageLoadPhase::Normal => "normal",
        ProjectPackageLoadPhase::OptionalDependencyConditional => "optional_dependency_conditional",
        ProjectPackageLoadPhase::Deferred => "deferred",
    }
}
fn reachability(value: ProjectPackageReachability) -> &'static str {
    match value {
        ProjectPackageReachability::Unreachable => "unreachable",
        ProjectPackageReachability::ConditionallyReachable => "conditionally_reachable",
        ProjectPackageReachability::Reachable => "reachable",
    }
}

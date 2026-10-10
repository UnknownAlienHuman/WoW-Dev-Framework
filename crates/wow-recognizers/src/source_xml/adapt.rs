//! Bounded structural joins over retained facts, exact analyzer-owned script
//! bindings, and accepted input-generation nodes.
use super::*;
use crate::RecognizerFactValue as Value;
use std::collections::BTreeSet;
use wow_core::{ClaimScope, EvidenceConfidence, ProvenanceClass, canonical_json_bytes};
use wow_graph::{GraphConfidence, GraphProposalValue};

const XML_CONTEXT_ID_PREFIX: &str = "project-xml-lua-context:sha256:";
const EXACT_XML_SCRIPT_SITE: &str = "exact_xml_script_site";
const IMPLICIT_RECEIVER_NOT_EVALUATED: &str = "not_evaluated_unwrapped_source";
const RUNTIME_DISPATCH_NOT_EVALUATED: &str = "not_evaluated_static_load_evidence_only";

/// A materialized accepted node from one preceding partition. No identity here
/// is derived from a source spelling or an ID-shaped string.
#[derive(Clone)]
struct Endpoint {
    token: String,
    kind: String,
    key: BTreeMap<Box<str>, GraphProposalValue>,
    handles: Vec<StableHandleId>,
    evidence: Vec<EvidenceId>,
}

type NodeKey = (String, Vec<u8>);

struct Nodes {
    source: BTreeMap<NodeKey, Endpoint>,
    template: BTreeMap<NodeKey, Endpoint>,
    object: BTreeMap<NodeKey, Endpoint>,
    toc: BTreeMap<NodeKey, Endpoint>,
    source_proposals: BTreeMap<String, Endpoint>,
}
impl Nodes {
    fn read(input: &SourceXmlInput<'_>, stop: &AtomicBool) -> RecognizerResult<Self> {
        let collect = |partition: &str| -> RecognizerResult<BTreeMap<NodeKey, Endpoint>> {
            let mut result = BTreeMap::<NodeKey, Endpoint>::new();
            let Some(partition) = input.owner.partition(partition) else {
                return Ok(result);
            };
            for accepted in partition.report().accepted_entities() {
                checkpoint(stop)?;
                if result.len() >= MAX_FACTS * 2 {
                    return Err(failure(RecognizerErrorCode::BudgetExceeded));
                }
                let proposal = partition
                    .batch()
                    .entity_proposal(accepted.proposal_id())
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                let key = (
                    proposal.entity_kind_id().to_owned(),
                    canonical_json_bytes(proposal.semantic_key())
                        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                );
                let endpoint = Endpoint {
                    token: accepted.node().node_id().to_string(),
                    kind: proposal.entity_kind_id().to_owned(),
                    key: proposal.semantic_key().clone(),
                    handles: proposal.source_handle_ids().to_vec(),
                    evidence: proposal.evidence_ids().to_vec(),
                };
                // Duplicate entity witnesses share one graph node while the
                // support of every contributing witness stays retained.
                if let Some(previous) = result.get_mut(&key) {
                    if previous.token != endpoint.token {
                        return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
                    }
                    previous.handles.extend(endpoint.handles);
                    previous.handles.sort();
                    previous.handles.dedup();
                    previous.evidence.extend(endpoint.evidence);
                    previous.evidence.sort();
                    previous.evidence.dedup();
                    if previous.handles.len() > 64 || previous.evidence.len() > 64 {
                        return Err(failure(RecognizerErrorCode::BudgetExceeded));
                    }
                } else {
                    result.insert(key, endpoint);
                }
            }
            Ok(result)
        };
        let mut source_proposals = BTreeMap::new();
        let partition = input
            .owner
            .partition(input.source_partition)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        for accepted in partition.report().accepted_entities() {
            checkpoint(stop)?;
            if source_proposals.len() >= MAX_FACTS * 2 {
                return Err(failure(RecognizerErrorCode::BudgetExceeded));
            }
            let proposal = partition
                .batch()
                .entity_proposal(accepted.proposal_id())
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let endpoint = Endpoint {
                token: accepted.node().node_id().to_string(),
                kind: proposal.entity_kind_id().to_owned(),
                key: proposal.semantic_key().clone(),
                handles: proposal.source_handle_ids().to_vec(),
                evidence: proposal.evidence_ids().to_vec(),
            };
            if source_proposals
                .insert(accepted.proposal_id().to_owned(), endpoint)
                .is_some()
            {
                return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
            }
        }
        Ok(Self {
            source: collect(input.source_partition)?,
            template: collect(SourceXmlFamily::Template.partition_id())?,
            object: collect(SourceXmlFamily::Object.partition_id())?,
            toc: collect(crate::source_toc::SourceTocFamily::Package.partition_id())?,
            source_proposals,
        })
    }
    fn find<'a>(
        &self,
        nodes: &'a BTreeMap<NodeKey, Endpoint>,
        kind: &str,
        document: &str,
        occurrence: &str,
    ) -> RecognizerResult<Option<&'a Endpoint>> {
        let key: BTreeMap<Box<str>, GraphProposalValue> = BTreeMap::from([
            (
                "document".into(),
                GraphProposalValue::String(document.into()),
            ),
            (
                "occurrence".into(),
                GraphProposalValue::String(occurrence.into()),
            ),
        ]);
        let bytes = canonical_json_bytes(&key)
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        Ok(nodes.get(&(kind.to_owned(), bytes)))
    }
    fn file(&self, document: &str) -> RecognizerResult<Option<&Endpoint>> {
        let key: BTreeMap<Box<str>, GraphProposalValue> =
            BTreeMap::from([("path".into(), GraphProposalValue::String(document.into()))]);
        let bytes = canonical_json_bytes(&key)
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        Ok(self.source.get(&("source_file".to_owned(), bytes)))
    }
    fn template(&self, document: &str, occurrence: &str) -> RecognizerResult<Option<&Endpoint>> {
        self.find(&self.template, "xml_template", document, occurrence)
    }
    fn object(&self, document: &str, occurrence: &str) -> RecognizerResult<Option<&Endpoint>> {
        self.find(&self.object, "xml_object", document, occurrence)
    }
    fn owner(&self, fact: &SourceXmlFact<'_>) -> RecognizerResult<Option<&Endpoint>> {
        let (kind, key): (&str, BTreeMap<Box<str>, GraphProposalValue>) = match fact.package {
            Some(package) => (
                "addon_package",
                BTreeMap::from([("package".into(), GraphProposalValue::String(package.into()))]),
            ),
            None => (
                "toc_variant",
                BTreeMap::from([
                    (
                        "document".into(),
                        GraphProposalValue::String(fact.selected_toc.into()),
                    ),
                    (
                        "flavor".into(),
                        GraphProposalValue::String(fact.flavor.into()),
                    ),
                ]),
            ),
        };
        let bytes = canonical_json_bytes(&key)
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        Ok(self.toc.get(&(kind.into(), bytes)))
    }
    fn declaration(&self, fact: &SourceXmlFact<'_>) -> RecognizerResult<Option<&Endpoint>> {
        if is_template(fact) {
            self.template(fact.document, fact.occurrence_id)
        } else {
            self.object(fact.document, fact.occurrence_id)
        }
    }
}

pub(super) struct Seed {
    pub recipe: Recipe,
    pub origins: Vec<String>,
    pub document: String,
    pub fields: BTreeMap<Box<str>, Value>,
    pub confidence: GraphConfidence,
    pub handles: BTreeSet<StableHandleId>,
    pub evidence: BTreeSet<EvidenceId>,
}
impl Seed {
    fn new(recipe: Recipe, fact: &SourceXmlFact<'_>) -> Self {
        Self {
            recipe,
            origins: vec![fact.fact_id.into()],
            document: fact.document.into(),
            fields: BTreeMap::from([
                ("admitted".into(), Value::Boolean(true)),
                ("origin".into(), Value::String(fact.fact_id.into())),
                ("document".into(), Value::String(fact.document.into())),
                (
                    "occurrence".into(),
                    Value::String(fact.occurrence_id.into()),
                ),
                (
                    "selected_toc".into(),
                    Value::String(fact.selected_toc.into()),
                ),
                ("flavor".into(), Value::String(fact.flavor.into())),
            ]),
            confidence: GraphConfidence::Derived,
            handles: BTreeSet::from([fact.source_handle_id]),
            evidence: BTreeSet::from([fact.evidence_id]),
        }
    }
    fn text(&mut self, field: &str, value: &str) {
        self.fields
            .insert(field.into(), Value::String(value.into()));
    }
    fn role(&mut self, field: &str, role: &str) {
        self.fields.insert(
            field.into(),
            Value::Reference(format!("entity:{role}").into()),
        );
    }
    fn endpoint(&mut self, field: &str, endpoint: &Endpoint) {
        self.fields.insert(
            field.into(),
            Value::Reference(endpoint.token.clone().into()),
        );
        self.handles.extend(endpoint.handles.iter().copied());
        self.evidence.extend(endpoint.evidence.iter().copied());
    }
    fn origin(&mut self, fact: &SourceXmlFact<'_>) {
        self.origins.push(fact.fact_id.into());
        self.origins.sort();
        self.origins.dedup();
        self.handles.insert(fact.source_handle_id);
        self.evidence.insert(fact.evidence_id);
    }
}

pub(super) struct Seeds {
    pub values: Vec<Seed>,
    pub omissions: Vec<SourceXmlOmission>,
    used_bytes: usize,
}
impl Seeds {
    fn omit(&mut self, facts: &[&SourceXmlFact<'_>], blocker: &'static str) {
        let mut fact_ids = facts
            .iter()
            .map(|fact| fact.fact_id.to_owned())
            .collect::<Vec<_>>();
        fact_ids.sort();
        fact_ids.dedup();
        self.omissions.push(SourceXmlOmission { fact_ids, blocker });
    }
    fn push(&mut self, seed: Seed) -> RecognizerResult<()> {
        let bytes = canonical_json_bytes(&seed.fields)
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?
            .len()
            .saturating_add((seed.handles.len() + seed.evidence.len()).saturating_mul(64));
        self.used_bytes = self.used_bytes.saturating_add(bytes);
        if self.values.len() >= MAX_FACTS || seed.handles.len() > 64 || seed.evidence.len() > 64 {
            return Err(failure(RecognizerErrorCode::BudgetExceeded));
        }
        if self.used_bytes > 16 * 1024 * 1024 {
            return Err(failure(RecognizerErrorCode::BudgetExceeded));
        }
        self.values.push(seed);
        Ok(())
    }
}

pub(super) fn validate(input: &SourceXmlInput<'_>, stop: &AtomicBool) -> RecognizerResult<()> {
    input
        .context
        .validate()
        .map_err(|_| failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    input.owner.validate(stop).map_err(graph_error)?;
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    if input.facts.len() > MAX_FACTS || input.script_bindings.len() > 8192 {
        return Err(failure(RecognizerErrorCode::BudgetExceeded));
    }
    let mut ids = BTreeSet::new();
    let mut declarations = BTreeMap::new();
    for fact in input.facts {
        checkpoint(stop)?;
        if !ids.insert(fact.fact_id)
            || fact.fact_id.is_empty()
            || fact.fact_id.len() > 1024
            || fact.context_id != input.context.context_id()
            || [
                fact.selected_toc,
                fact.flavor,
                fact.document,
                fact.occurrence_id,
            ]
            .iter()
            .any(|value| value.is_empty() || value.len() > 16_384)
        {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        let handle = input
            .source_handles
            .get(&fact.source_handle_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        if handle.path().as_str() != fact.document
            || handle.content_digest() != &fact.content_digest
            || handle.span() != fact.span
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        validate_support(input, &[fact.source_handle_id], &[fact.evidence_id])?;
        if matches!(fact.kind, SourceXmlFactKind::Declaration { .. })
            && declarations
                .insert(occurrence_key(fact, fact.occurrence_id), fact)
                .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    for fact in input.facts {
        checkpoint(stop)?;
        let occurrence = match &fact.kind {
            SourceXmlFactKind::Declaration { .. } => continue,
            SourceXmlFactKind::Script {
                owner_occurrence_id,
                ..
            } => *owner_occurrence_id,
            _ => Some(fact.occurrence_id),
        };
        if let Some(occurrence) = occurrence {
            let Some(declaration) = declarations.get(&occurrence_key(fact, occurrence)) else {
                // Script lexical owners include Ui; only captured declaration
                // owners can produce semantic ownership or callback edges.
                if matches!(fact.kind, SourceXmlFactKind::Script { .. }) {
                    continue;
                }
                return Err(missing_occurrence(&declarations, occurrence));
            };
            if !same_scope(fact, declaration) {
                return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
            }
        }
        let target = match &fact.kind {
            SourceXmlFactKind::Parent { resolution, .. } => resolution.target(),
            SourceXmlFactKind::Inheritance {
                target_occurrence_id,
                ..
            } => Some(*target_occurrence_id),
            _ => None,
        };
        if let Some(target) = target {
            let declaration = declarations
                .get(&occurrence_key(fact, target))
                .ok_or_else(|| missing_occurrence(&declarations, target))?;
            if !same_scope(fact, declaration) {
                return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
            }
        }
    }
    let mut bindings = BTreeSet::new();
    for binding in input.script_bindings {
        checkpoint(stop)?;
        if !content_id(binding.binding_id, "xml-script-binding:sha256:")
            || !content_id(binding.site_id, "xml-script-site:sha256:")
            || !bindings.insert(binding.binding_id)
            || !matches!(
                binding.confidence,
                GraphConfidence::Derived | GraphConfidence::Possible
            )
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
        }
        validate_support(input, binding.source_handle_ids, binding.evidence_ids)?;
    }
    Ok(())
}

fn validate_support(
    input: &SourceXmlInput<'_>,
    handles: &[StableHandleId],
    evidence: &[EvidenceId],
) -> RecognizerResult<()> {
    if handles.is_empty()
        || evidence.is_empty()
        || handles.len() > 64
        || evidence.len() > 64
        || handles.windows(2).any(|w| w[0] >= w[1])
        || evidence.windows(2).any(|w| w[0] >= w[1])
    {
        return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
    }
    let mut witnessed = BTreeSet::new();
    for id in evidence {
        let record = input
            .evidence
            .get(id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        record
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        let [handle_id] = record.source_handle_ids() else {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let handle = input
            .source_handles
            .get(handle_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        handle
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        if record.evidence_id() != *id
            || record.context_id() != input.context.context_id()
            || record.provenance() != ProvenanceClass::ProjectSource
            || record.confidence() != EvidenceConfidence::Proven
            || record.claim_scope() != ClaimScope::SourceObservation
            || !record.derivation_input_ids().is_empty()
            || !record.coverage_refs().is_empty()
            || handle.handle_id() != *handle_id
            || handle.project_generation() != input.context.project_generation()
            || handle.reference_generation() != Some(input.context.reference_generation())
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        witnessed.insert(*handle_id);
    }
    if witnessed.iter().ne(handles.iter()) {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}

fn same_scope(left: &SourceXmlFact<'_>, right: &SourceXmlFact<'_>) -> bool {
    left.selected_toc == right.selected_toc
        && left.flavor == right.flavor
        && left.package == right.package
}
type OccurrenceKey<'a> = (&'a str, &'a str, Option<&'a str>, &'a str);
fn occurrence_key<'a>(fact: &SourceXmlFact<'a>, occurrence: &'a str) -> OccurrenceKey<'a> {
    (fact.selected_toc, fact.flavor, fact.package, occurrence)
}
fn missing_occurrence(
    declarations: &BTreeMap<OccurrenceKey<'_>, &SourceXmlFact<'_>>,
    occurrence: &str,
) -> RecognizerError {
    failure(if declarations.keys().any(|key| key.3 == occurrence) {
        RecognizerErrorCode::AdapterIdentityMismatch
    } else {
        RecognizerErrorCode::AdapterBindingMissing
    })
}
fn is_template(fact: &SourceXmlFact<'_>) -> bool {
    matches!(
        fact.kind,
        SourceXmlFactKind::Declaration {
            virtual_template: SourceXmlTemplateState::True,
            ..
        } | SourceXmlFactKind::Declaration {
            intrinsic: SourceXmlTemplateState::True,
            ..
        }
    )
}
fn valid_element(fact: &SourceXmlFact<'_>) -> bool {
    matches!(
        fact.kind,
        SourceXmlFactKind::Declaration {
            role: SourceXmlElementRole::Element,
            valid_declaration: true,
            ..
        }
    )
}
fn content_id(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|value| {
        value.len() == 64
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

pub(super) fn seeds(
    input: &SourceXmlInput<'_>,
    family: SourceXmlFamily,
    stop: &AtomicBool,
) -> RecognizerResult<Seeds> {
    let nodes = Nodes::read(input, stop)?;
    let declarations = input
        .facts
        .iter()
        .filter(|f| matches!(f.kind, SourceXmlFactKind::Declaration { .. }))
        .map(|f| (occurrence_key(f, f.occurrence_id), f))
        .collect::<BTreeMap<_, _>>();
    let scripts = input
        .facts
        .iter()
        .filter(|f| matches!(f.kind, SourceXmlFactKind::Script { .. }))
        .map(|f| (occurrence_key(f, f.occurrence_id), f))
        .collect::<BTreeMap<_, _>>();
    // A binding's native scope comes from its actual accepted receiver endpoint,
    // whose qualified document and local occurrence are both owner-validated.
    let mut declaration_endpoints = BTreeMap::new();
    for fact in declarations.values() {
        checkpoint(stop)?;
        if declaration_endpoints
            .insert((fact.document, fact.occurrence_id), *fact)
            .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    let mut result = Seeds {
        values: Vec::new(),
        omissions: Vec::new(),
        used_bytes: 0,
    };
    for fact in input.facts {
        checkpoint(stop)?;
        match &fact.kind {
            SourceXmlFactKind::Declaration {
                role,
                element_name,
                name,
                virtual_template,
                intrinsic,
                mixin_names,
                valid_declaration,
                parent_occurrence_id,
            } => {
                let desired = if is_template(fact) {
                    SourceXmlFamily::Template
                } else {
                    SourceXmlFamily::Object
                };
                if family != desired || *role != SourceXmlElementRole::Element {
                    continue;
                }
                if !valid_declaration {
                    result.omit(&[fact], "xml.invalid_declaration");
                    continue;
                }
                let Some((file, owner)) = nodes.file(fact.document)?.zip(nodes.owner(fact)?) else {
                    result.omit(&[fact], "xml.declaration_endpoint_not_materialized");
                    continue;
                };
                let template = is_template(fact);
                let mut seed = Seed::new(
                    if template {
                        Recipe::TemplateDeclared
                    } else {
                        Recipe::ObjectDeclared
                    },
                    fact,
                );
                seed.endpoint("file", file);
                seed.endpoint("owner", owner);
                seed.role("entity", if template { "template" } else { "object" });
                seed.text("role", role.name());
                seed.text("element_name", element_name);
                seed.text("virtual_template", virtual_template.name());
                seed.text("intrinsic", intrinsic.name());
                if let Some(name) = name {
                    seed.text("name", name);
                }
                if let Some(parent) = parent_occurrence_id {
                    seed.text("lexical_parent", parent);
                }
                seed.text(
                    "mixins",
                    &String::from_utf8(
                        canonical_json_bytes(mixin_names)
                            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                    )
                    .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                );
                result.push(seed)?;
            }
            SourceXmlFactKind::Parent {
                reference_id,
                name,
                resolution,
                order,
                cycle_id,
            } if family == SourceXmlFamily::ObjectParentage => {
                let SourceXmlParentResolution::Unique {
                    target_occurrence_id,
                } = resolution
                else {
                    result.omit(&[fact], "xml.parent_reference_unresolved");
                    continue;
                };
                let child = declarations[&occurrence_key(fact, fact.occurrence_id)];
                let parent = declarations[&occurrence_key(fact, target_occurrence_id)];
                if !valid_element(child)
                    || !valid_element(parent)
                    || is_template(child)
                    || is_template(parent)
                    || fact.occurrence_id == *target_occurrence_id
                    || cycle_id.is_some()
                    || *order != Some(SourceXmlReferenceOrder::TargetBeforeSource)
                {
                    result.omit(
                        &[fact, child, parent],
                        "xml.parent_reference_not_admissible",
                    );
                    continue;
                }
                let Some((source, target)) =
                    nodes.declaration(parent)?.zip(nodes.declaration(child)?)
                else {
                    result.omit(
                        &[fact, child, parent],
                        "xml.parent_endpoint_not_materialized",
                    );
                    continue;
                };
                let mut seed = Seed::new(Recipe::ParentOf, fact);
                seed.origin(child);
                seed.origin(parent);
                seed.text("reference_id", reference_id);
                seed.text("name", name);
                seed.text("resolution", resolution.name());
                if let Some(order) = order {
                    seed.text("load_order", order.name());
                }
                seed.endpoint("source", source);
                seed.endpoint("target", target);
                result.push(seed)?;
            }
            SourceXmlFactKind::Inheritance {
                reference_id,
                target_occurrence_id,
                order,
                cycle_id,
            } if family == SourceXmlFamily::Inherits => {
                let source_fact = declarations[&occurrence_key(fact, fact.occurrence_id)];
                let target_fact = declarations[&occurrence_key(fact, target_occurrence_id)];
                if !valid_element(source_fact)
                    || !valid_element(target_fact)
                    || !is_template(target_fact)
                    || fact.occurrence_id == *target_occurrence_id
                    || cycle_id.is_some()
                    || *order != Some(SourceXmlReferenceOrder::TargetBeforeSource)
                {
                    result.omit(
                        &[fact, source_fact, target_fact],
                        "xml.inheritance_not_admissible",
                    );
                    continue;
                }
                let Some((source, target)) = nodes
                    .declaration(source_fact)?
                    .zip(nodes.declaration(target_fact)?)
                else {
                    result.omit(
                        &[fact, source_fact, target_fact],
                        "xml.inheritance_endpoint_not_materialized",
                    );
                    continue;
                };
                for recipe in [Recipe::InheritsTemplate, Recipe::ReferencesTemplate] {
                    let mut seed = Seed::new(recipe, fact);
                    seed.origin(source_fact);
                    seed.origin(target_fact);
                    seed.text("reference_id", reference_id);
                    if let Some(order) = order {
                        seed.text("load_order", order.name());
                    }
                    seed.endpoint("source", source);
                    seed.endpoint("target", target);
                    result.push(seed)?;
                }
            }
            SourceXmlFactKind::InheritanceUnresolved { .. }
                if family == SourceXmlFamily::Inherits =>
            {
                result.omit(&[fact], "xml.inheritance_unresolved");
            }
            SourceXmlFactKind::Script {
                owner_occurrence_id,
                source_kind,
                script_name,
                inherit,
                intrinsic_order,
                file_reference,
                function_reference,
                method_reference,
                ..
            } if family == SourceXmlFamily::Script => {
                let Some(owner_fact) = owner_occurrence_id
                    .and_then(|owner| declarations.get(&occurrence_key(fact, owner)).copied())
                else {
                    result.omit(&[fact], "xml.script_owner_not_captured");
                    continue;
                };
                let Some(owner) = nodes.declaration(owner_fact)? else {
                    result.omit(&[fact, owner_fact], "xml.script_owner_not_materialized");
                    continue;
                };
                let mut seed = Seed::new(Recipe::ScriptSite, fact);
                seed.origin(owner_fact);
                seed.endpoint("owner", owner);
                seed.role("site", "site");
                seed.text("script_source", source_kind.name());
                seed.text("script_name", script_name);
                for (field, value) in [
                    ("inherit", inherit),
                    ("intrinsic_order", intrinsic_order),
                    ("file_reference", file_reference),
                    ("function_reference", function_reference),
                    ("method_reference", method_reference),
                ] {
                    if let Some(value) = value {
                        seed.text(field, value);
                    }
                }
                result.push(seed)?;
                if !input.script_bindings.iter().any(|binding| {
                    binding.script_id == fact.occurrence_id
                        && binding.consumer_occurrence_id == Some(owner_fact.occurrence_id)
                        && nodes
                            .source_proposals
                            .get(binding.receiver_proposal_id)
                            .is_some_and(|receiver| {
                                receiver.key.get("document")
                                    == Some(&GraphProposalValue::String(owner_fact.document.into()))
                            })
                }) {
                    result.omit(&[fact, owner_fact], "xml.script_handler_not_admitted");
                }
            }
            _ => {}
        }
    }
    if family == SourceXmlFamily::Script {
        for binding in input.script_bindings {
            checkpoint(stop)?;
            let owner_id = binding
                .consumer_occurrence_id
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let receiver = nodes
                .source_proposals
                .get(binding.receiver_proposal_id)
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let Some(GraphProposalValue::String(document)) = receiver.key.get("document") else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let owner_fact = declaration_endpoints
                .get(&(document.as_ref(), owner_id))
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let fact = scripts
                .get(&occurrence_key(owner_fact, binding.script_id))
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            if !same_scope(fact, owner_fact) {
                return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
            }
            let handler = nodes
                .source_proposals
                .get(binding.handler_proposal_id)
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let SourceXmlFactKind::Script {
                source_kind,
                owner_occurrence_id,
                ..
            } = &fact.kind
            else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            if (!binding.inherited && *owner_occurrence_id != Some(owner_id))
                || (binding.inherited && binding.confidence != GraphConfidence::Possible)
            {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            match (source_kind, handler.kind.as_str()) {
                (SourceXmlScriptSource::InlineBody, "xml_source_handler") => {
                    if handler.key.get("document")
                        != Some(&GraphProposalValue::String(fact.document.into()))
                        || handler.key.get("occurrence")
                            != Some(&GraphProposalValue::Identifier(fact.occurrence_id.into()))
                    {
                        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                    }
                }
                (SourceXmlScriptSource::ReferenceOnly, "lua_source_function") => {}
                _ => return Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
            }
            if receiver.kind != "xml_source_declaration"
                || receiver.key.get("document")
                    != Some(&GraphProposalValue::String(owner_fact.document.into()))
                || receiver.key.get("occurrence")
                    != Some(&GraphProposalValue::Identifier(owner_id.into()))
                || handler.kind != binding.handler_kind
                || !binding.source_handle_ids.contains(&fact.source_handle_id)
                || !binding.evidence_ids.contains(&fact.evidence_id)
                || [receiver, handler].iter().any(|endpoint| {
                    endpoint
                        .handles
                        .iter()
                        .any(|id| binding.source_handle_ids.binary_search(id).is_err())
                        || endpoint
                            .evidence
                            .iter()
                            .any(|id| binding.evidence_ids.binary_search(id).is_err())
                })
            {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            validate_context(binding, handler)?;
            let Some(owner) = nodes.declaration(owner_fact)? else {
                result.omit(&[fact, owner_fact], "xml.script_consumer_not_materialized");
                continue;
            };
            let mut seed = Seed::new(Recipe::ScriptBinding, fact);
            seed.origins.push(binding.binding_id.into());
            seed.origins.sort();
            seed.origins.dedup();
            seed.origin(owner_fact);
            seed.endpoint("owner", owner);
            seed.endpoint("handler", handler);
            seed.role("site", "site");
            seed.text("binding_id", binding.binding_id);
            seed.text("site_id", binding.site_id);
            seed.fields
                .insert("inherited".into(), Value::Boolean(binding.inherited));
            seed.confidence = binding.confidence;
            seed.handles
                .extend(binding.source_handle_ids.iter().copied());
            seed.evidence.extend(binding.evidence_ids.iter().copied());
            result.push(seed)?;
        }
    }
    result
        .omissions
        .sort_by(|a, b| a.fact_ids.cmp(&b.fact_ids).then(a.blocker.cmp(b.blocker)));
    Ok(result)
}

fn validate_context(
    binding: &SourceXmlScriptBinding<'_>,
    handler: &Endpoint,
) -> RecognizerResult<()> {
    match binding.handler_kind {
        "xml_source_handler" => {
            let context = binding
                .semantic_context
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingInvalid))?;
            if binding.confidence != GraphConfidence::Possible
                || !content_id(context.context_id, XML_CONTEXT_ID_PREFIX)
                || context.script_site != EXACT_XML_SCRIPT_SITE
                || context.implicit_receiver != IMPLICIT_RECEIVER_NOT_EVALUATED
                || context.runtime_dispatch != RUNTIME_DISPATCH_NOT_EVALUATED
                || handler.key.get("semantic_context_id")
                    != Some(&GraphProposalValue::String(context.context_id.into()))
            {
                return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
            }
        }
        "lua_source_function"
            if binding.semantic_context.is_none()
                && !handler.key.contains_key("semantic_context_id") => {}
        _ => return Err(failure(RecognizerErrorCode::AdapterBindingInvalid)),
    }
    Ok(())
}

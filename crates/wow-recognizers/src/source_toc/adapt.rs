//! Bounded structural joins over retained facts and accepted input-generation nodes.
use super::*;
use crate::RecognizerFactValue as Value;
use std::collections::BTreeSet;
use wow_core::{ClaimScope, EvidenceConfidence, ProvenanceClass, canonical_json_bytes};
use wow_graph::{GraphConfidence, GraphProposalValue};

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
    fn new(recipe: Recipe, fact: &SourceTocFact<'_>) -> Self {
        Self {
            recipe,
            origins: vec![fact.fact_id.into()],
            document: fact.selected_toc.into(),
            fields: BTreeMap::from([
                ("admitted".into(), Value::Boolean(true)),
                ("origin".into(), Value::String(fact.fact_id.into())),
                ("document".into(), Value::String(fact.selected_toc.into())),
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
        self.handles.extend(&endpoint.handles);
        self.evidence.extend(&endpoint.evidence);
    }
    fn include(&mut self, fact: &SourceTocFact<'_>) {
        self.origins.push(fact.fact_id.into());
        self.origins.sort();
        self.origins.dedup();
        self.fields.insert(
            "origins".into(),
            Value::String(self.origins.join("|").into()),
        );
        self.handles.insert(fact.source_handle_id);
        self.evidence.insert(fact.evidence_id);
    }
}
#[derive(Clone)]
struct Endpoint {
    token: String,
    handles: Vec<StableHandleId>,
    evidence: Vec<EvidenceId>,
}
type NodeKey = (String, Vec<u8>);
struct Nodes {
    source: BTreeMap<NodeKey, Endpoint>,
    toc: BTreeMap<NodeKey, Endpoint>,
}
impl Nodes {
    fn read(input: &SourceTocInput<'_>, stop: &AtomicBool) -> RecognizerResult<Self> {
        let collect = |partition: &str| -> RecognizerResult<BTreeMap<NodeKey, Endpoint>> {
            let mut result = BTreeMap::<NodeKey, Endpoint>::new();
            if let Some(partition) = input.owner.partition(partition) {
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
                        handles: proposal.source_handle_ids().to_vec(),
                        evidence: proposal.evidence_ids().to_vec(),
                    };
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
            }
            Ok(result)
        };
        Ok(Self {
            source: collect(input.source_partition)?,
            toc: collect(SourceTocFamily::Package.partition_id())?,
        })
    }
    fn find(
        &self,
        source: bool,
        kind: &str,
        fields: &[(&str, &str)],
    ) -> RecognizerResult<Option<&Endpoint>> {
        let key = fields
            .iter()
            .map(|(key, value)| ((*key).into(), GraphProposalValue::String((*value).into())))
            .collect::<BTreeMap<Box<str>, GraphProposalValue>>();
        let bytes = canonical_json_bytes(&key)
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        Ok(if source { &self.source } else { &self.toc }.get(&(kind.to_owned(), bytes)))
    }
    fn variant(&self, fact: &SourceTocFact<'_>) -> RecognizerResult<Option<&Endpoint>> {
        self.find(
            false,
            "toc_variant",
            &[("document", fact.selected_toc), ("flavor", fact.flavor)],
        )
    }
    fn package(&self, name: &str) -> RecognizerResult<Option<&Endpoint>> {
        self.find(false, "addon_package", &[("package", name)])
    }
    fn file(&self, path: &str) -> RecognizerResult<Option<&Endpoint>> {
        self.find(true, "source_file", &[("path", path)])
    }
}
pub(super) struct Seeds {
    pub values: Vec<Seed>,
    pub omissions: Vec<SourceTocOmission>,
    used_bytes: usize,
}
impl Seeds {
    fn omit(&mut self, facts: &[&SourceTocFact<'_>], blocker: &'static str) {
        let mut fact_ids = facts
            .iter()
            .map(|f| f.fact_id.to_owned())
            .collect::<Vec<_>>();
        fact_ids.sort();
        fact_ids.dedup();
        self.omissions.push(SourceTocOmission { fact_ids, blocker });
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

pub(super) fn validate(input: &SourceTocInput<'_>, stop: &AtomicBool) -> RecognizerResult<()> {
    input
        .context
        .validate()
        .map_err(|_| failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    input.owner.validate(stop).map_err(graph_error)?;
    if input.owner.source_context_id() != input.context.context_id()
        || input.facts.len() > MAX_FACTS
    {
        return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    let mut ids = BTreeSet::new();
    let mut plans = BTreeMap::new();
    for fact in input.facts {
        checkpoint(stop)?;
        if !ids.insert(fact.fact_id)
            || fact.fact_id.is_empty()
            || fact.fact_id.len() > 1024
            || fact.context_id != input.context.context_id()
        {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        let handle = input
            .source_handles
            .get(&fact.source_handle_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let evidence = input
            .evidence
            .get(&fact.evidence_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        handle
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        evidence
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        if handle.handle_id() != fact.source_handle_id
            || handle.path().as_str() != fact.selected_toc
            || handle.content_digest() != &fact.content_digest
            || handle.span() != fact.span
            || handle.project_generation() != input.context.project_generation()
            || handle.reference_generation() != Some(input.context.reference_generation())
            || evidence.evidence_id() != fact.evidence_id
            || evidence.context_id() != fact.context_id
            || evidence.source_handle_ids() != [fact.source_handle_id]
            || evidence.provenance() != ProvenanceClass::ProjectSource
            || evidence.confidence() != EvidenceConfidence::Proven
            || evidence.claim_scope() != ClaimScope::SourceObservation
            || !evidence.derivation_input_ids().is_empty()
            || !evidence.coverage_refs().is_empty()
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        if matches!(fact.kind, SourceTocFactKind::Package { .. })
            && plans
                .insert((fact.selected_toc, fact.flavor), fact.package)
                .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    for fact in input.facts {
        if plans.get(&(fact.selected_toc, fact.flavor)) != Some(&fact.package) {
            return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
        }
    }
    Ok(())
}

pub(super) fn seeds(
    input: &SourceTocInput<'_>,
    family: SourceTocFamily,
    stop: &AtomicBool,
) -> RecognizerResult<Seeds> {
    let nodes = Nodes::read(input, stop)?;
    let mut result = Seeds {
        values: Vec::new(),
        omissions: Vec::new(),
        used_bytes: 0,
    };
    for fact in input.facts {
        checkpoint(stop)?;
        let belongs = match fact.kind {
            SourceTocFactKind::Package { .. } => SourceTocFamily::Package,
            SourceTocFactKind::File { .. } => SourceTocFamily::FileOrder,
            SourceTocFactKind::Dependency { .. } => SourceTocFamily::Dependencies,
            SourceTocFactKind::LoadOnDemand { .. } => SourceTocFamily::LoadOnDemand,
            SourceTocFactKind::SavedVariable { .. } => SourceTocFamily::SavedVariables,
        };
        if belongs != family {
            continue;
        }
        if fact.selection != SourceTocSelection::Included {
            result.omit(&[fact], "toc.selection_not_included");
            continue;
        }
        let seed = match &fact.kind {
            SourceTocFactKind::Package { .. } => {
                let mut seed = Seed::new(
                    if fact.package.is_some() {
                        Recipe::PackageNamed
                    } else {
                        Recipe::PackageIsolated
                    },
                    fact,
                );
                if let Some(package) = fact.package {
                    seed.text("package", package);
                    seed.role("package_ref", "package");
                } else {
                    result.omit(&[fact], "toc.package_identity_not_supplied");
                }
                seed.role("manifest_ref", "manifest");
                seed.role("variant_ref", "variant");
                seed
            }
            SourceTocFactKind::File { path, .. } => {
                let target = match path {
                    Some(path) => nodes.file(path)?,
                    None => None,
                };
                let Some((variant, target)) = nodes.variant(fact)?.zip(target) else {
                    result.omit(&[fact], "toc.file_endpoint_not_materialized");
                    continue;
                };
                let mut seed = Seed::new(Recipe::FileLoads, fact);
                seed.endpoint("source", variant);
                seed.endpoint("target", target);
                seed
            }
            SourceTocFactKind::Dependency {
                optional,
                resolved_package,
                ..
            } => {
                let Some((source, target)) = fact.package.zip(*resolved_package) else {
                    result.omit(&[fact], "toc.dependency_unresolved");
                    continue;
                };
                let Some((source, target)) = nodes.package(source)?.zip(nodes.package(target)?)
                else {
                    result.omit(&[fact], "toc.dependency_endpoint_not_materialized");
                    continue;
                };
                if source.token == target.token {
                    result.omit(&[fact], "toc.self_dependency_not_representable");
                    continue;
                }
                let mut seed = Seed::new(
                    if *optional {
                        Recipe::OptionalDependency
                    } else {
                        Recipe::RequiredDependency
                    },
                    fact,
                );
                seed.endpoint("source", source);
                seed.endpoint("target", target);
                seed
            }
            SourceTocFactKind::LoadOnDemand { state, conflicting } => {
                let Some(source) = nodes.variant(fact)? else {
                    result.omit(&[fact], "toc.variant_not_materialized");
                    continue;
                };
                let mut seed = Seed::new(Recipe::LoadPolicy, fact);
                seed.text("state", state.name());
                seed.endpoint("source", source);
                seed.role("policy_ref", "policy");
                if *conflicting || *state == SourceTocLoadState::Unknown {
                    seed.confidence = GraphConfidence::Possible;
                    result.omit(&[fact], "toc.load_policy_uncertain");
                }
                seed
            }
            SourceTocFactKind::SavedVariable {
                name,
                scope,
                declared,
            } => {
                if !*declared {
                    result.omit(&[fact], "toc.saved_variable_identifier_unsupported");
                    continue;
                }
                let source = match fact.package {
                    Some(package) => nodes.package(package)?,
                    None => nodes.variant(fact)?,
                };
                let Some(source) = source else {
                    result.omit(&[fact], "toc.state_owner_not_materialized");
                    continue;
                };
                let mut seed = Seed::new(Recipe::SavedVariable, fact);
                seed.text("name", name);
                seed.text("scope", scope.name());
                seed.endpoint("source", source);
                seed.role("root_ref", "root");
                seed
            }
        };
        result.push(seed)?;
    }
    if family == SourceTocFamily::FileOrder {
        order(input, &nodes, &mut result, stop)?;
    }
    result
        .omissions
        .sort_by(|a, b| (&a.fact_ids, a.blocker).cmp(&(&b.fact_ids, b.blocker)));
    result.omissions.dedup();
    Ok(result)
}

fn order(
    input: &SourceTocInput<'_>,
    nodes: &Nodes,
    result: &mut Seeds,
    stop: &AtomicBool,
) -> RecognizerResult<()> {
    let mut documents = BTreeMap::<(&str, &str), Vec<&SourceTocFact<'_>>>::new();
    for fact in input.facts {
        if matches!(fact.kind, SourceTocFactKind::File { .. }) {
            documents
                .entry((fact.selected_toc, fact.flavor))
                .or_default()
                .push(fact);
        }
    }
    for mut files in documents.into_values() {
        files.sort_by_key(|fact| fact.ordinal);
        for pair in files.windows(2) {
            checkpoint(stop)?;
            if pair[0].ordinal == pair[1].ordinal {
                return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
            }
            let (
                SourceTocFactKind::File {
                    path: left,
                    repeated: left_repeated,
                },
                SourceTocFactKind::File {
                    path: right,
                    repeated: right_repeated,
                },
            ) = (&pair[0].kind, &pair[1].kind)
            else {
                continue;
            };
            if *left_repeated
                || *right_repeated
                || pair
                    .iter()
                    .any(|f| f.selection != SourceTocSelection::Included)
            {
                result.omit(pair, "toc.adjacent_order_ambiguous_or_unselected");
                continue;
            }
            let endpoints = match (*left).zip(*right) {
                Some((left, right)) => nodes.file(left)?.zip(nodes.file(right)?),
                None => None,
            };
            let Some((left, right)) = endpoints else {
                result.omit(pair, "toc.adjacent_endpoint_not_materialized");
                continue;
            };
            if left.token == right.token {
                result.omit(pair, "toc.self_order_not_representable");
                continue;
            }
            let mut seed = Seed::new(Recipe::FileBefore, pair[0]);
            seed.include(pair[1]);
            seed.endpoint("source", left);
            seed.endpoint("target", right);
            result.push(seed)?;
        }
    }
    Ok(())
}

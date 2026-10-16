//! Exact-span buckets form disjoint complete bipartite graphs. Retain every
//! permitted edge; overflowing blocks remain explicit and never select a winner.
use super::*;
use crate::{GraphConfidence, GraphNodeId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wow_core::{CanonicalResult, ContentDigest};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageEntityAddress {
    pub role: LineageRole,
    pub node_id: GraphNodeId,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageCandidateProposal {
    pub proposal_id: Box<str>,
    pub input_partition_id: Box<str>,
    pub relation: Box<str>,
    pub before: LineageEntityAddress,
    pub after: LineageEntityAddress,
    pub confidence: GraphConfidence,
    pub proof_ceiling: GraphConfidence,
    pub feature_profile_id: Box<str>,
    pub fingerprint: ContentDigest<CanonicalResult>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageComponentShape {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
    BeforeOnly,
    AfterOnly,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageCandidateComponent {
    pub component_id: Box<str>,
    pub input_partition_id: Box<str>,
    pub shape: LineageComponentShape,
    pub before: Vec<LineageEntityAddress>,
    pub after: Vec<LineageEntityAddress>,
    pub proposal_ids: Vec<Box<str>>,
    pub ambiguous: bool,
    pub coverage: Box<str>,
    pub negative_authority: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageBucketSkipReason {
    BucketMembers,
    PairBudget,
    ComponentNodes,
    ComponentEdges,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageSkippedBucket {
    pub function_kind: LineageFunctionKind,
    pub fingerprint: ContentDigest<CanonicalResult>,
    pub before_members: usize,
    pub after_members: usize,
    pub reason: LineageBucketSkipReason,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageUnavailableFeature {
    pub entity: LineageEntityAddress,
    pub state: Box<str>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageCandidateSet {
    pub schema: Box<str>,
    pub profile_id: Box<str>,
    pub input_partition_id: Box<str>,
    pub proposals: Vec<LineageCandidateProposal>,
    pub components: Vec<LineageCandidateComponent>,
    pub skipped_buckets: Vec<LineageSkippedBucket>,
    pub unavailable_features: Vec<LineageUnavailableFeature>,
    pub status: Box<str>,
    pub accepted_assertion_count: usize,
    pub promotion: Box<str>,
    pub negative_authority: bool,
    usage: LineageUsage,
}
impl LineageCandidateSet {
    pub const fn usage(&self) -> LineageUsage {
        self.usage
    }
}
struct Bucket<'a> {
    before: Vec<LineageEntityAddress>,
    after: Vec<LineageEntityAddress>,
    feature: &'a LineageSourceSpanFeature,
}

pub fn generate_lineage_candidate_components(
    input: &ValidatedLineageInput,
    stop: &AtomicBool,
) -> GraphResult<LineageCandidateSet> {
    let mut ledger = OperationBudget::resume(input.limits, input.usage, stop)?;
    let mut buckets: BTreeMap<(LineageFunctionKind, ContentDigest<CanonicalResult>), Bucket<'_>> =
        BTreeMap::new();
    let mut output = LineageCandidateSet {
        schema: "wow-graph/lineage-source-span-candidates/1".into(),
        profile_id: PROJECT_SPAN_LINEAGE_PROFILE.into(),
        input_partition_id: input.partition_id.clone(),
        proposals: Vec::new(),
        components: Vec::new(),
        skipped_buckets: Vec::new(),
        unavailable_features: Vec::new(),
        status: "partial".into(),
        accepted_assertion_count: 0,
        promotion: "not_evaluated".into(),
        negative_authority: false,
        usage: LineageUsage::default(),
    };
    ledger.metadata(&(
        &output.schema,
        &output.profile_id,
        &output.input_partition_id,
    ))?;
    for record in &input.data.records {
        ledger.work(1)?;
        let address = LineageEntityAddress {
            role: record.role,
            node_id: record.node.node_id().clone(),
        };
        ledger.metadata(&address)?;
        match &record.fingerprint {
            LineageFingerprintFeature::Available { source_span } => {
                let key = (record.function_kind, source_span.fingerprint);
                ledger.metadata(&key)?;
                let bucket = buckets.entry(key).or_insert_with(|| Bucket {
                    before: Vec::new(),
                    after: Vec::new(),
                    feature: source_span,
                });
                if bucket.feature.profile_id != source_span.profile_id
                    || bucket.feature.content_sha256 != source_span.content_sha256
                    || bucket.feature.byte_length != source_span.byte_length
                {
                    return Err(invalid());
                }
                match record.role {
                    LineageRole::Before => bucket.before.push(address),
                    LineageRole::After => bucket.after.push(address),
                }
            }
            feature => {
                let unavailable = LineageUnavailableFeature {
                    entity: address.clone(),
                    state: match feature {
                        LineageFingerprintFeature::MissingExactSpan => "missing_exact_span",
                        _ => "conflict_blocked",
                    }
                    .into(),
                };
                ledger.metadata(&unavailable)?;
                output.unavailable_features.push(unavailable);
                singleton(&mut output, &mut ledger, address, "feature_unavailable")?;
            }
        }
    }
    for ((kind, fingerprint), mut bucket) in buckets {
        ledger.work(
            bucket
                .before
                .len()
                .checked_add(bucket.after.len())
                .ok_or_else(budget)?,
        )?;
        // Input is canonical; explicit sorting keeps this stage independently
        // deterministic if an owner changes its internal enumeration later.
        bucket.before.sort();
        bucket.after.sort();
        let members = bucket
            .before
            .len()
            .checked_add(bucket.after.len())
            .ok_or_else(budget)?;
        let pairs = bucket.before.len().checked_mul(bucket.after.len());
        let reason = if members > input.limits.max_bucket_members {
            Some(LineageBucketSkipReason::BucketMembers)
        } else if pairs.is_none_or(|n| {
            n > input
                .limits
                .max_pairs
                .saturating_sub(output.proposals.len())
        }) {
            Some(LineageBucketSkipReason::PairBudget)
        } else if pairs != Some(0) && members > input.limits.max_component_nodes {
            Some(LineageBucketSkipReason::ComponentNodes)
        } else if pairs.is_none_or(|n| n > input.limits.max_component_edges) {
            Some(LineageBucketSkipReason::ComponentEdges)
        } else {
            None
        };
        if let Some(reason) = reason {
            let skipped = LineageSkippedBucket {
                function_kind: kind,
                fingerprint,
                before_members: bucket.before.len(),
                after_members: bucket.after.len(),
                reason,
            };
            ledger.metadata(&skipped)?;
            output.skipped_buckets.push(skipped);
            output.status = "truncated".into();
            for node in bucket.before.into_iter().chain(bucket.after) {
                singleton(&mut output, &mut ledger, node, "bucket_skipped")?;
            }
            continue;
        }
        if pairs == Some(0) {
            for node in bucket.before.into_iter().chain(bucket.after) {
                singleton(
                    &mut output,
                    &mut ledger,
                    node,
                    "unmatched_under_executed_block",
                )?;
            }
            continue;
        }
        let mut ids = Vec::new();
        for before in &bucket.before {
            for after in &bucket.after {
                ledger.work(1)?;
                let mut proposal = LineageCandidateProposal {
                    proposal_id: "".into(),
                    input_partition_id: input.partition_id.clone(),
                    relation: "lineage_successor_of".into(),
                    before: before.clone(),
                    after: after.clone(),
                    confidence: GraphConfidence::Candidate,
                    proof_ceiling: GraphConfidence::Candidate,
                    feature_profile_id: SOURCE_SPAN_FEATURE_PROFILE.into(),
                    fingerprint,
                };
                ledger.metadata(&proposal)?;
                proposal.proposal_id =
                    identity("wow-graph/lineage-candidate-proposal/1", &proposal)?;
                ledger.metadata(&proposal.proposal_id)?;
                ids.push(proposal.proposal_id.clone());
                output.proposals.push(proposal);
            }
        }
        ids.sort();
        ledger.metadata(&ids)?;
        component(
            &mut output,
            &mut ledger,
            bucket.before,
            bucket.after,
            ids,
            "partial",
        )?;
    }
    output
        .proposals
        .sort_by(|a, b| a.proposal_id.cmp(&b.proposal_id));
    output
        .components
        .sort_by(|a, b| a.component_id.cmp(&b.component_id));
    output
        .unavailable_features
        .sort_by(|a, b| a.entity.cmp(&b.entity));
    // A digest collision is an error, never exact-pair or component dedup.
    for pair in output.proposals.windows(2) {
        ledger.work(1)?;
        if pair[0].proposal_id == pair[1].proposal_id {
            return Err(invalid());
        }
    }
    for pair in output.components.windows(2) {
        ledger.work(1)?;
        if pair[0].component_id == pair[1].component_id {
            return Err(invalid());
        }
    }
    output.usage = ledger.usage;
    checkpoint(stop)?;
    Ok(output)
}
fn singleton(
    output: &mut LineageCandidateSet,
    ledger: &mut OperationBudget<'_>,
    node: LineageEntityAddress,
    coverage: &str,
) -> GraphResult<()> {
    match node.role {
        LineageRole::Before => component(output, ledger, vec![node], vec![], vec![], coverage),
        LineageRole::After => component(output, ledger, vec![], vec![node], vec![], coverage),
    }
}
fn component(
    output: &mut LineageCandidateSet,
    ledger: &mut OperationBudget<'_>,
    before: Vec<LineageEntityAddress>,
    after: Vec<LineageEntityAddress>,
    proposal_ids: Vec<Box<str>>,
    coverage: &str,
) -> GraphResult<()> {
    ledger.work(1)?;
    let shape = match (before.len(), after.len()) {
        (1, 0) => LineageComponentShape::BeforeOnly,
        (0, 1) => LineageComponentShape::AfterOnly,
        (1, 1) => LineageComponentShape::OneToOne,
        (1, _) => LineageComponentShape::OneToMany,
        (_, 1) => LineageComponentShape::ManyToOne,
        _ => LineageComponentShape::ManyToMany,
    };
    let ambiguous = before.len() > 1 || after.len() > 1;
    let mut value = LineageCandidateComponent {
        component_id: "".into(),
        input_partition_id: output.input_partition_id.clone(),
        shape,
        before,
        after,
        proposal_ids,
        ambiguous,
        coverage: coverage.into(),
        negative_authority: false,
    };
    ledger.metadata(&value)?;
    value.component_id = identity("wow-graph/lineage-candidate-component/1", &value)?;
    ledger.metadata(&value.component_id)?;
    output.components.push(value);
    Ok(())
}

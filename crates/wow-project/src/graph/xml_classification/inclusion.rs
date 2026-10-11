//! Borrowed typed handoff for the future real candidate; no candidate is constructed here.
use super::*;
use wow_graph::GraphProposalRejectionCode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "disposition", rename_all = "snake_case")]
pub enum XmlClassificationProposalDisposition {
    Accepted {
        proposal: GraphLocalAssertion,
        reference: GraphAssertionRef,
    },
    Rejected {
        proposal: GraphLocalAssertion,
        code: GraphProposalRejectionCode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlClassificationPartitionIdentity {
    pub partition_id: String,
    pub partition_digest: String,
    pub batch_id: String,
    pub report_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlClassificationMembership {
    graph_snapshot_id: String,
    registry_digest: String,
    scope: GraphAssertionRecordScope,
    partitions: Vec<XmlClassificationPartitionIdentity>,
    dispositions: Vec<XmlClassificationProposalDisposition>,
    digest: ContentDigest<CanonicalResult>,
}

/// This borrows genuine classification and graph-admission records. Decoding
/// output cannot mint this capability or assert E3 candidate acceptance.
#[derive(Debug, Serialize)]
pub struct XmlClassificationInclusionManifest<'a> {
    schema: &'static str,
    classification: &'a ProjectXmlClassification,
    membership: &'a XmlClassificationMembership,
}

impl<'a> XmlClassificationInclusionManifest<'a> {
    pub(in crate::graph) const fn new(
        classification: &'a ProjectXmlClassification,
        membership: &'a XmlClassificationMembership,
    ) -> Self {
        Self {
            schema: "wow-project/xml-classification-inclusion/1",
            classification,
            membership,
        }
    }
    #[must_use]
    pub const fn classification(&self) -> &ProjectXmlClassification {
        self.classification
    }
    #[must_use]
    pub const fn membership(&self) -> &XmlClassificationMembership {
        self.membership
    }
}

impl XmlClassificationMembership {
    #[must_use]
    pub fn scope(&self) -> &GraphAssertionRecordScope {
        &self.scope
    }
    #[must_use]
    pub fn partitions(&self) -> &[XmlClassificationPartitionIdentity] {
        &self.partitions
    }
    #[must_use]
    pub fn dispositions(&self) -> &[XmlClassificationProposalDisposition] {
        &self.dispositions
    }
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
}

pub(in crate::graph) fn membership(
    classification: &ProjectXmlClassification,
    owner: &GraphPartitionSnapshot,
    scope: &GraphAssertionRecordScope,
    addresses: &BTreeMap<GraphLocalAssertion, GraphAssertionRef>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<XmlClassificationMembership> {
    crate::analyzer::checkpoint(stop)?;
    let lookup = owner
        .producer_lookup(stop)
        .map_err(platform_producers::graph_error)?;
    if lookup.scope() != scope || scope.source_context_id != classification.context_id {
        return Err(invalid());
    }
    let mut keys = BTreeSet::new();
    for proposal in &classification.proposals {
        budget.charge_serialized(&("xml-classification-membership-key", proposal), stop)?;
        keys.insert(proposal.clone());
    }
    for document in &classification.documents {
        for occurrence in &document.occurrences {
            crate::analyzer::checkpoint(stop)?;
            let key = local_entity(&occurrence.occurrence_proposal_id);
            budget.charge_serialized(&("xml-classification-membership-key", &key), stop)?;
            keys.insert(key);
        }
    }
    for witness in classification.catalog_support.witnesses() {
        crate::analyzer::checkpoint(stop)?;
        let key = local_entity(&witness.raw_member_proposal_id);
        budget.charge_serialized(&("xml-classification-membership-key", &key), stop)?;
        keys.insert(key);
    }
    let mut partitions = BTreeMap::new();
    let mut dispositions = Vec::new();
    for key in keys {
        crate::analyzer::checkpoint(stop)?;
        let reference = addresses.get(&key).ok_or_else(invalid)?;
        let partition = match key.kind {
            GraphAssertionKind::Entity => lookup
                .entity(scope, reference, stop)
                .map_err(platform_producers::graph_error)?
                .partition(),
            GraphAssertionKind::Relation => lookup
                .relation(scope, reference, stop)
                .map_err(platform_producers::graph_error)?
                .partition(),
        };
        if !partition.report().ready_for_publication()
            || partition.report().batch_id() != partition.batch().batch_id()
        {
            return Err(invalid());
        }
        if !partitions.contains_key(partition.partition_id()) {
            let row = XmlClassificationPartitionIdentity {
                partition_id: partition.partition_id().into(),
                partition_digest: partition.partition_digest().into(),
                batch_id: partition.batch().batch_id().into(),
                report_id: partition.report().report_id().into(),
            };
            budget.charge_serialized(&row, stop)?;
            partitions.insert(row.partition_id.clone(), row);
        }
        // finish already requires every proposal accepted. Rejected admission
        // refuses that owner; never synthesize a successful rejected manifest.
        let disposition = XmlClassificationProposalDisposition::Accepted {
            proposal: key,
            reference: reference.clone(),
        };
        budget.charge_serialized(&disposition, stop)?;
        dispositions.push(disposition);
    }
    let partitions = partitions.into_values().collect::<Vec<_>>();
    let graph_snapshot_id = owner.snapshot().snapshot_id().as_str().to_owned();
    let registry_digest = owner.registry().registry_digest().to_owned();
    let identity = (
        &graph_snapshot_id,
        &registry_digest,
        scope,
        &partitions,
        &dispositions,
        classification.digest(),
    );
    budget.charge_serialized(&identity, stop)?;
    let digest = crate::identity::canonical_digest(
        "wow-project/xml-classification-membership/1",
        &identity,
        ProjectPhase::View,
    )?;
    let result = XmlClassificationMembership {
        graph_snapshot_id,
        registry_digest,
        scope: scope.clone(),
        partitions,
        dispositions,
        digest,
    };
    budget.charge_serialized(&result, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(result)
}

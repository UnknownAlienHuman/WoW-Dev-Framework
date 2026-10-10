//! Direct ownership is assigned where native proposals are created.
use super::*;

/// Independently admitted direct platform responsibilities. Raw membership is
/// a separate Inventory capability, preserving its existing exact batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformGraphProducer {
    Inventory,
    TocLoad,
    AnalyzerStructure,
    XmlStructure,
}

impl PlatformGraphProducer {
    #[must_use]
    pub const fn partition_id(self) -> &'static str {
        match self {
            Self::Inventory => "wow-project.platform-inventory",
            Self::TocLoad => "wow-project.platform-toc-load",
            Self::AnalyzerStructure => "wow-project.platform-analyzer-structure",
            Self::XmlStructure => "wow-project.platform-xml-structure",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct EntityDraft {
    pub producer: PlatformGraphProducer,
    pub proposal: GraphEntityProposal,
}

impl EntityDraft {
    pub fn new(producer: PlatformGraphProducer, proposal: GraphEntityProposal) -> Self {
        Self { producer, proposal }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct RelationDraft {
    pub producer: PlatformGraphProducer,
    pub proposal_id: Box<str>,
    pub relation_kind_id: Box<str>,
    pub input: GraphRelationProposalInput,
}

impl RelationDraft {
    pub fn new(
        producer: PlatformGraphProducer,
        proposal_id: impl Into<Box<str>>,
        relation_kind_id: impl Into<Box<str>>,
        input: GraphRelationProposalInput,
    ) -> wow_graph::GraphResult<Self> {
        // Retain the typed constructor input before immutable proposal creation.
        // Validation still occurs at the original emission boundary.
        let proposal_id = proposal_id.into();
        let relation_kind_id = relation_kind_id.into();
        GraphRelationProposal::new(proposal_id.clone(), relation_kind_id.clone(), input.clone())?;
        Ok(Self {
            producer,
            proposal_id,
            relation_kind_id,
            input,
        })
    }

    pub fn into_proposal(self) -> wow_graph::GraphResult<GraphRelationProposal> {
        GraphRelationProposal::new(self.proposal_id, self.relation_kind_id, self.input)
    }
}

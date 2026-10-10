//! Mixed native producer endpoints over the existing script fact pipeline.
use super::*;
use serde::ser::{SerializeMap, SerializeSeq};

pub const SOURCE_SCRIPT_ASSERTION_PROFILE: &str = "wow-recognizers/source-xml-script-assertions/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceScriptAssertionEndpoints {
    pub receiver: GraphAssertionRef,
    pub handler: GraphAssertionRef,
}

pub struct SourceScriptAssertionFact<'a> {
    pub fact: SourceScriptFact<'a>,
    pub endpoints: SourceScriptAssertionEndpoints,
}

pub struct SourceScriptAssertionInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub scope: &'a GraphAssertionRecordScope,
    pub context: &'a GenerationContext,
    pub facts: Vec<SourceScriptAssertionFact<'a>>,
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}

/// The exact admitted prerequisite addresses, without a single-partition claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceScriptAssertionRecognition {
    profile: &'static str,
    scope: GraphAssertionRecordScope,
    endpoints: BTreeMap<String, SourceScriptAssertionEndpoints>,
    recognition: RecognitionReport,
    receipts: Vec<SourceScriptReceipt>,
}
impl SourceScriptAssertionRecognition {
    pub fn profile(&self) -> &str {
        self.profile
    }
    pub fn scope(&self) -> &GraphAssertionRecordScope {
        &self.scope
    }
    pub fn endpoints(&self) -> &BTreeMap<String, SourceScriptAssertionEndpoints> {
        &self.endpoints
    }
    pub fn recognition(&self) -> &RecognitionReport {
        &self.recognition
    }
    pub fn receipts(&self) -> &[SourceScriptReceipt] {
        &self.receipts
    }
}

pub struct SourceScriptAssertionProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceScriptAssertionRecognition,
}

pub fn recognize_source_script_assertions(
    input: SourceScriptAssertionInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceScriptAssertionProposals> {
    checkpoint(stop)?;
    if input.facts.len() > MAX_BINDINGS {
        return Err(failure(RecognizerErrorCode::BudgetExceeded));
    }
    crate::source_assertions::preflight(
        &InputMetadata {
            scope: input.scope,
            context: input.context,
            facts: FactMetadata(&input.facts),
            source_handles: input.source_handles,
            evidence: input.evidence,
        },
        stop,
    )?;
    input
        .context
        .validate()
        .map_err(|_| failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    let lookup = input.owner.producer_lookup(stop).map_err(graph_error)?;
    if lookup.scope() != input.scope {
        return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    let output = recognize_script_facts(
        &ScriptData {
            owner: input.owner,
            context: input.context,
            source_handles: input.source_handles,
            evidence: input.evidence,
        },
        ScriptFacts::Assertions(&input.facts),
        ScriptEndpoints::Assertions {
            lookup: &lookup,
            scope: input.scope,
            facts: &input.facts,
        },
        lookup.input_view(),
        stop,
    )?;
    // This is the complete returned envelope, borrowed before address/key copies.
    crate::source_assertions::preflight(
        &OutputMetadata {
            batch: &output.batch,
            coverage: &output.coverage,
            recognition: RecognitionMetadata {
                profile: SOURCE_SCRIPT_ASSERTION_PROFILE,
                scope: input.scope,
                endpoints: EndpointMetadata(&input.facts),
                recognition: &output.recognition,
                receipts: &output.receipts,
            },
        },
        stop,
    )?;
    let mut endpoints = BTreeMap::new();
    for fact in &input.facts {
        checkpoint(stop)?;
        if endpoints
            .insert(fact.fact.fact_id.to_owned(), fact.endpoints.clone())
            .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    let recognition = SourceScriptAssertionRecognition {
        profile: SOURCE_SCRIPT_ASSERTION_PROFILE,
        scope: input.scope.clone(),
        endpoints,
        recognition: output.recognition,
        receipts: output.receipts,
    };
    checkpoint(stop)?;
    Ok(SourceScriptAssertionProposals {
        batch: output.batch,
        coverage: output.coverage,
        recognition,
    })
}

#[derive(Serialize)]
struct InputMetadata<'a, 'b> {
    scope: &'b GraphAssertionRecordScope,
    context: &'b GenerationContext,
    facts: FactMetadata<'a, 'b>,
    source_handles: &'b BTreeMap<StableHandleId, SourceHandle>,
    evidence: &'b BTreeMap<EvidenceId, EvidenceRecord>,
}
struct FactMetadata<'a, 'b>(&'b [SourceScriptAssertionFact<'a>]);
impl Serialize for FactMetadata<'_, '_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Fact<'a> {
            fact_id: &'a str,
            receiver_proposal_id: &'a str,
            handler_proposal_id: &'a str,
            semantic_context: Option<ContextMetadata<'a>>,
            confidence: GraphConfidence,
            source_handle_ids: &'a [StableHandleId],
            evidence_ids: &'a [EvidenceId],
            endpoints: &'a SourceScriptAssertionEndpoints,
        }
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for assertion in self.0 {
            let fact = &assertion.fact;
            seq.serialize_element(&Fact {
                fact_id: fact.fact_id,
                receiver_proposal_id: fact.receiver_proposal_id,
                handler_proposal_id: fact.handler_proposal_id,
                semantic_context: fact.semantic_context.map(ContextMetadata::from),
                confidence: fact.confidence,
                source_handle_ids: fact.source_handle_ids,
                evidence_ids: fact.evidence_ids,
                endpoints: &assertion.endpoints,
            })?;
        }
        seq.end()
    }
}

#[derive(Serialize)]
pub(crate) struct ContextMetadata<'a> {
    context_id: &'a str,
    script_site: &'a str,
    implicit_receiver: &'a str,
    runtime_dispatch: &'a str,
}
impl<'a> From<SourceScriptSemanticContext<'a>> for ContextMetadata<'a> {
    fn from(context: SourceScriptSemanticContext<'a>) -> Self {
        Self {
            context_id: context.context_id,
            script_site: context.script_site,
            implicit_receiver: context.implicit_receiver,
            runtime_dispatch: context.runtime_dispatch,
        }
    }
}

struct EndpointMetadata<'a, 'b>(&'b [SourceScriptAssertionFact<'a>]);
impl Serialize for EndpointMetadata<'_, '_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for fact in self.0 {
            map.serialize_entry(fact.fact.fact_id, &fact.endpoints)?;
        }
        map.end()
    }
}
#[derive(Serialize)]
struct RecognitionMetadata<'a, 'b> {
    profile: &'static str,
    scope: &'b GraphAssertionRecordScope,
    endpoints: EndpointMetadata<'a, 'b>,
    recognition: &'b RecognitionReport,
    receipts: &'b [SourceScriptReceipt],
}
#[derive(Serialize)]
struct OutputMetadata<'a, 'b> {
    batch: &'b GraphProposalBatch,
    coverage: &'b [GraphCoverageRecord],
    recognition: RecognitionMetadata<'a, 'b>,
}

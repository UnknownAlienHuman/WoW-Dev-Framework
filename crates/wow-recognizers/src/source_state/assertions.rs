//! Exact mixed source endpoints, retaining a separate native recognition envelope.
use super::*;

pub const SOURCE_STATE_ASSERTION_PROFILE: &str =
    "wow-recognizers/source-saved-variable-assertions/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceStateAssertionEndpoints {
    pub root: GraphAssertionRef,
    pub caller: GraphAssertionRef,
    pub target: GraphAssertionRef,
}
#[derive(Serialize)]
pub struct SourceStateAssertionFact<'a> {
    pub fact: SourceStateFact<'a>,
    pub endpoints: SourceStateAssertionEndpoints,
}
pub struct SourceStateAssertionInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub scope: &'a GraphAssertionRecordScope,
    pub report: &'a FunctionCallReport,
    pub context: &'a GenerationContext,
    pub facts: Vec<SourceStateAssertionFact<'a>>,
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceStateAssertionRecognition {
    profile: &'static str,
    scope: GraphAssertionRecordScope,
    analyzer_report_id: String,
    recognition: RecognitionReport,
    receipts: Vec<SourceStateReceipt>,
    endpoints_by_binding: BTreeMap<String, SourceStateAssertionEndpoints>,
}
impl SourceStateAssertionRecognition {
    pub fn profile(&self) -> &'static str {
        self.profile
    }
    pub fn scope(&self) -> &GraphAssertionRecordScope {
        &self.scope
    }
    pub fn analyzer_report_id(&self) -> &str {
        &self.analyzer_report_id
    }
    pub fn recognition(&self) -> &RecognitionReport {
        &self.recognition
    }
    pub fn receipts(&self) -> &[SourceStateReceipt] {
        &self.receipts
    }
    pub fn endpoints(&self, binding_id: &str) -> Option<&SourceStateAssertionEndpoints> {
        self.endpoints_by_binding.get(binding_id)
    }
}
#[derive(Serialize)]
pub struct SourceStateAssertionProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceStateAssertionRecognition,
}
#[derive(Serialize)]
struct OutputMetadata<'a> {
    batch: &'a GraphProposalBatch,
    coverage: &'a [GraphCoverageRecord],
    recognition: RecognitionMetadata<'a>,
}
#[derive(Serialize)]
struct RecognitionMetadata<'a> {
    profile: &'static str,
    scope: &'a GraphAssertionRecordScope,
    analyzer_report_id: &'a str,
    recognition: &'a RecognitionReport,
    receipts: &'a [SourceStateReceipt],
    endpoints_by_binding: &'a BTreeMap<&'a str, &'a SourceStateAssertionEndpoints>,
}

pub fn recognize_source_state_assertions(
    input: SourceStateAssertionInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceStateAssertionProposals> {
    checkpoint(stop)?;
    if input.facts.len() > MAX_BINDINGS {
        return Err(failure(RecognizerErrorCode::BudgetExceeded));
    }
    let mut endpoints = BTreeMap::new();
    for binding in &input.facts {
        checkpoint(stop)?;
        if endpoints
            .insert(binding.fact.fact_id, &binding.endpoints)
            .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    // Conservative provisional input tally; final object framing is checked below.
    crate::source_assertions::preflight(
        &(
            SOURCE_STATE_ASSERTION_PROFILE,
            input.scope,
            input.report.analysis_id(),
            &input.facts,
            &endpoints,
        ),
        stop,
    )?;
    let facts = input
        .facts
        .iter()
        .map(|binding| {
            let fact = binding.fact;
            checkpoint(stop)?;
            Ok(fact)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;
    checkpoint(stop)?;
    let common = StateInput {
        owner: input.owner,
        report: input.report,
        context: input.context,
        facts,
        source_handles: input.source_handles,
        evidence: input.evidence,
    };
    let result = recognize_state(
        &common,
        &StateBindings::Assertions {
            scope: input.scope,
            endpoints: &endpoints,
        },
        stop,
    )?;
    crate::source_assertions::preflight(
        &OutputMetadata {
            batch: &result.batch,
            coverage: &result.coverage,
            recognition: RecognitionMetadata {
                profile: SOURCE_STATE_ASSERTION_PROFILE,
                scope: input.scope,
                analyzer_report_id: input.report.analysis_id(),
                recognition: &result.recognition,
                receipts: &result.receipts,
                endpoints_by_binding: &endpoints,
            },
        },
        stop,
    )?;
    let mut endpoints_by_binding = BTreeMap::new();
    for receipt in &result.receipts {
        let refs = endpoints
            .get(receipt.binding_id.as_str())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        endpoints_by_binding.insert(receipt.binding_id.clone(), (**refs).clone());
        checkpoint(stop)?;
    }
    if endpoints_by_binding.len() != endpoints.len() {
        return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
    }
    let scope = input.scope.clone();
    checkpoint(stop)?;
    let analyzer_report_id = input.report.analysis_id().to_string();
    checkpoint(stop)?;
    Ok(SourceStateAssertionProposals {
        batch: result.batch,
        coverage: result.coverage,
        recognition: SourceStateAssertionRecognition {
            profile: SOURCE_STATE_ASSERTION_PROFILE,
            scope,
            analyzer_report_id,
            recognition: result.recognition,
            receipts: result.receipts,
            endpoints_by_binding,
        },
    })
}

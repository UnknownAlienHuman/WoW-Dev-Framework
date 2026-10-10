//! Declarative state rules over the exact preceding access admission. The
//! analyzer, global resolution and span/support validator run in source_state.
mod adapt;
mod pack;
mod project;
mod records;

use crate::source_state::{
    SOURCE_STATE_PARTITION, SourceStateAssertionRecognition, SourceStateRecognition,
};
use crate::{
    RecognizerError, RecognizerErrorCode, RecognizerFact, RecognizerFactBundle,
    RecognizerFactCoverage, RecognizerFactCoverageInput, RecognizerFactCoverageState,
    RecognizerFactInput, RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind,
    RecognizerFactValue as Value, RecognizerOutputConfidence, RecognizerOutputPartition,
    RecognizerProposedAssertion, RecognizerResult, compile_recognizer_plan,
    execute_recognizer_plan,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{GenerationContext, canonical_json_bytes};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphCoverageRecord,
    GraphCoverageState, GraphEntityProposal, GraphLocalAssertion, GraphNodeId,
    GraphPartitionSnapshot, GraphProposalBatch, GraphProposalEndpoint, GraphProposalValue,
    GraphRelationKind, GraphRelationProposal, GraphRelationProposalInput,
};

const FACT_PROFILE: &str = "wow-recognizers-state-facts-1";
const MAX_BINDINGS: usize = 8192;
const MAX_FACT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceStateCoreFamily {
    Read,
    Write,
}
impl SourceStateCoreFamily {
    pub const ALL: [Self; 2] = [Self::Read, Self::Write];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
    pub const fn rule_id(self) -> &'static str {
        match self {
            Self::Read => "core.state.literal_path_read",
            Self::Write => "core.state.literal_path_write",
        }
    }
    pub const fn partition_id(self) -> &'static str {
        match self {
            Self::Read => "wow-recognizers.state-read",
            Self::Write => "wow-recognizers.state-write",
        }
    }
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::Read => "project.state.literal_path_read",
            Self::Write => "project.state.literal_path_write",
        }
    }
    const fn definition_id(self) -> &'static str {
        match self {
            Self::Read => "source_reads_state",
            Self::Write => "source_writes_state",
        }
    }
    const fn relation(self) -> GraphRelationKind {
        match self {
            Self::Read => GraphRelationKind::ReadsState,
            Self::Write => GraphRelationKind::WritesState,
        }
    }
}
pub struct SourceStateCoreInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub context: &'a GenerationContext,
    pub recognition: &'a SourceStateRecognition,
}
pub struct SourceStateCoreAssertionInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub context: &'a GenerationContext,
    pub recognition: &'a SourceStateAssertionRecognition,
}
struct CoreInput<'a> {
    owner: &'a GraphPartitionSnapshot,
    context: &'a GenerationContext,
    recognition: CoreRecognition<'a>,
}
#[derive(Clone, Copy)]
enum CoreRecognition<'a> {
    Legacy(&'a SourceStateRecognition),
    Assertions(&'a SourceStateAssertionRecognition),
}
impl CoreRecognition<'_> {
    fn analyzer_report_id(&self) -> &str {
        match self {
            Self::Legacy(value) => value.analyzer_report_id(),
            Self::Assertions(value) => value.analyzer_report_id(),
        }
    }
    fn recognition(&self) -> &crate::RecognitionReport {
        match self {
            Self::Legacy(value) => value.recognition(),
            Self::Assertions(value) => value.recognition(),
        }
    }
    fn receipts(&self) -> &[crate::source_state::SourceStateReceipt] {
        match self {
            Self::Legacy(value) => value.receipts(),
            Self::Assertions(value) => value.receipts(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceStateCoreEvaluation {
    pub recipe: &'static str,
    pub pack_digest: String,
    pub fact_bundle: RecognizerFactBundle,
    pub output: RecognizerOutputPartition,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceStateCoreReceipt {
    pub match_id: String,
    pub fact_ids: Vec<String>,
    pub entity_proposal_ids: Vec<String>,
    pub relation_proposal_ids: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceStateCoreRecognition {
    pub profile: &'static str,
    pub family: SourceStateCoreFamily,
    pub analyzer_report_id: String,
    pub admission_partition_digest: String,
    pub evaluations: Vec<SourceStateCoreEvaluation>,
    pub receipts: Vec<SourceStateCoreReceipt>,
}
pub struct SourceStateCoreProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceStateCoreRecognition,
}
#[derive(Serialize)]
struct CoreOutputMetadata<'a> {
    batch: &'a GraphProposalBatch,
    coverage: &'a [GraphCoverageRecord],
    recognition: &'a SourceStateCoreRecognition,
}

pub fn recognize_source_state_core(
    input: SourceStateCoreInput<'_>,
    family: SourceStateCoreFamily,
    stop: &AtomicBool,
) -> RecognizerResult<SourceStateCoreProposals> {
    let input = CoreInput {
        owner: input.owner,
        context: input.context,
        recognition: CoreRecognition::Legacy(input.recognition),
    };
    let admitted = adapt::normalize(&input, family, stop)?;
    project::execute(&input, family, admitted, stop)
}
pub fn recognize_source_state_core_assertions(
    input: SourceStateCoreAssertionInput<'_>,
    family: SourceStateCoreFamily,
    stop: &AtomicBool,
) -> RecognizerResult<SourceStateCoreProposals> {
    crate::source_assertions::preflight(input.recognition, stop)?;
    let input = CoreInput {
        owner: input.owner,
        context: input.context,
        recognition: CoreRecognition::Assertions(input.recognition),
    };
    let admitted = adapt::normalize(&input, family, stop)?;
    let output = project::execute(&input, family, admitted, stop)?;
    crate::source_assertions::preflight(
        &CoreOutputMetadata {
            batch: &output.batch,
            coverage: &output.coverage,
            recognition: &output.recognition,
        },
        stop,
    )?;
    checkpoint(stop)?;
    Ok(output)
}
fn confidence_of(value: RecognizerOutputConfidence) -> GraphConfidence {
    match value {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}
fn checkpoint(stop: &AtomicBool) -> RecognizerResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(fail(RecognizerErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn fail(code: RecognizerErrorCode) -> RecognizerError {
    RecognizerError::new(
        code,
        "admitted state access and core matcher bindings disagree",
    )
}
fn graph_error(error: wow_graph::GraphError) -> RecognizerError {
    fail(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}

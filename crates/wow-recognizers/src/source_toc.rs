//! Selected-TOC structure through the declarative matcher. Parsing and selection
//! remain in the load owner. Each family reads a real preceding graph snapshot.
mod adapt;
mod pack;
mod project;

use crate::{RecognizerError, RecognizerErrorCode, RecognizerOutputPartition, RecognizerResult};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{
    ContentDigest, EvidenceId, EvidenceRecord, GenerationContext, GenerationContextId,
    SourceContent, SourceHandle, SourceSpan, StableHandleId,
};
use wow_graph::{GraphCoverageRecord, GraphPartitionSnapshot, GraphProposalBatch};

pub const SOURCE_TOC_PROFILE: &str = "wow-recognizers/toc-structural/1";
const FACT_PROFILE: &str = "wow-recognizers-toc-facts-1";
const MAX_FACTS: usize = 32_768;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceTocFamily {
    Package,
    FileOrder,
    Dependencies,
    LoadOnDemand,
    SavedVariables,
    SavedVariableRoot,
}
impl SourceTocFamily {
    /// The five original core.toc families keep their exact identities, order
    /// and partition digests. SavedVariableRoot is excluded from ALL because the
    /// service publishes it explicitly after these five; listing it here would
    /// publish the state-root partition twice.
    pub const ALL: [Self; 5] = [
        Self::Package,
        Self::FileOrder,
        Self::Dependencies,
        Self::LoadOnDemand,
        Self::SavedVariables,
    ];
    pub const fn rule_id(self) -> &'static str {
        match self {
            Self::Package => "core.toc.package",
            Self::FileOrder => "core.toc.file_order",
            Self::Dependencies => "core.toc.dependencies",
            Self::LoadOnDemand => "core.toc.load_on_demand",
            Self::SavedVariables => "core.toc.saved_variables",
            Self::SavedVariableRoot => "core.state.saved_variable_root",
        }
    }
    pub const fn partition_id(self) -> &'static str {
        match self {
            Self::Package => "wow-recognizers.toc-package",
            Self::FileOrder => "wow-recognizers.toc-file-order",
            Self::Dependencies => "wow-recognizers.toc-dependencies",
            Self::LoadOnDemand => "wow-recognizers.toc-load-on-demand",
            Self::SavedVariables => "wow-recognizers.toc-saved-variables",
            Self::SavedVariableRoot => "wow-recognizers.state-root",
        }
    }
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::Package => "project.toc.package",
            Self::FileOrder => "project.toc.file_order",
            Self::Dependencies => "project.toc.dependencies",
            Self::LoadOnDemand => "project.toc.load_on_demand",
            Self::SavedVariables => "project.toc.saved_variables",
            Self::SavedVariableRoot => "project.toc.saved_variable_root",
        }
    }
}

/// The operation's retained facts, including excluded/unresolved occurrences.
/// Omitted payload details remain available in the project-owned provenance.
pub struct SourceTocFact<'a> {
    pub fact_id: &'a str,
    pub context_id: GenerationContextId,
    pub package: Option<&'a str>,
    pub selected_toc: &'a str,
    pub flavor: &'a str,
    pub ordinal: u64,
    pub selection: SourceTocSelection,
    pub content_digest: ContentDigest<SourceContent>,
    pub span: SourceSpan,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
    pub kind: SourceTocFactKind<'a>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTocSelection {
    Included,
    Excluded,
    Unresolved,
}
pub enum SourceTocFactKind<'a> {
    Package {
        source_complete: bool,
    },
    File {
        path: Option<&'a str>,
        repeated: bool,
    },
    Dependency {
        name: &'a str,
        optional: bool,
        resolved_package: Option<&'a str>,
    },
    LoadOnDemand {
        state: SourceTocLoadState,
        conflicting: bool,
    },
    SavedVariable {
        name: &'a str,
        scope: SourceTocScope,
        declared: bool,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTocLoadState {
    NotDeclared,
    False,
    True,
    Unknown,
}
impl SourceTocLoadState {
    const fn name(self) -> &'static str {
        match self {
            Self::NotDeclared => "not_declared",
            Self::False => "false",
            Self::True => "true",
            Self::Unknown => "unknown",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTocScope {
    Account,
    Character,
}
impl SourceTocScope {
    const fn name(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Character => "character",
        }
    }
}
pub struct SourceTocInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub source_partition: &'a str,
    pub context: &'a GenerationContext,
    pub facts: &'a [SourceTocFact<'a>],
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceTocOmission {
    pub fact_ids: Vec<String>,
    pub blocker: &'static str,
}
/// Exact matcher outcomes, matches, support and proposal identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceTocEvaluation {
    pub recipe: &'static str,
    pub pack_digest: String,
    pub fact_bundle: crate::RecognizerFactBundle,
    pub output: RecognizerOutputPartition,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceTocReceipt {
    pub match_id: String,
    pub fact_ids: Vec<String>,
    pub entity_proposal_ids: Vec<String>,
    pub relation_proposal_ids: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceTocRecognition {
    pub profile: &'static str,
    pub family: SourceTocFamily,
    pub evaluations: Vec<SourceTocEvaluation>,
    pub receipts: Vec<SourceTocReceipt>,
    pub omissions: Vec<SourceTocOmission>,
}
pub struct SourceTocProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceTocRecognition,
}
pub fn recognize_source_toc(
    input: SourceTocInput<'_>,
    family: SourceTocFamily,
    stop: &AtomicBool,
) -> RecognizerResult<SourceTocProposals> {
    checkpoint(stop)?;
    adapt::validate(&input, stop)?;
    let seeds = adapt::seeds(&input, family, stop)?;
    project::execute(&input, family, seeds, stop)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Recipe {
    PackageNamed,
    PackageIsolated,
    FileLoads,
    FileBefore,
    RequiredDependency,
    OptionalDependency,
    LoadPolicy,
    SavedVariable,
    StateRoot,
}
impl Recipe {
    const fn family(self) -> SourceTocFamily {
        match self {
            Self::PackageNamed | Self::PackageIsolated => SourceTocFamily::Package,
            Self::FileLoads | Self::FileBefore => SourceTocFamily::FileOrder,
            Self::RequiredDependency | Self::OptionalDependency => SourceTocFamily::Dependencies,
            Self::LoadPolicy => SourceTocFamily::LoadOnDemand,
            Self::SavedVariable => SourceTocFamily::SavedVariables,
            Self::StateRoot => SourceTocFamily::SavedVariableRoot,
        }
    }
    const fn name(self) -> &'static str {
        match self {
            Self::PackageNamed => "toc_package_named",
            Self::PackageIsolated => "toc_package_isolated",
            Self::FileLoads => "toc_file_load",
            Self::FileBefore => "toc_file_before",
            Self::RequiredDependency => "toc_dependency_required",
            Self::OptionalDependency => "toc_dependency_optional",
            Self::LoadPolicy => "toc_load_policy",
            Self::SavedVariable => "toc_saved_variable",
            Self::StateRoot => "toc_state_root",
        }
    }
    fn for_family(family: SourceTocFamily) -> Vec<Self> {
        match family {
            SourceTocFamily::Package => vec![Self::PackageNamed, Self::PackageIsolated],
            SourceTocFamily::FileOrder => vec![Self::FileLoads, Self::FileBefore],
            SourceTocFamily::Dependencies => {
                vec![Self::RequiredDependency, Self::OptionalDependency]
            }
            SourceTocFamily::LoadOnDemand => vec![Self::LoadPolicy],
            SourceTocFamily::SavedVariables => vec![Self::SavedVariable],
            SourceTocFamily::SavedVariableRoot => vec![Self::StateRoot],
        }
    }
}
fn checkpoint(stop: &AtomicBool) -> RecognizerResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(RecognizerErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn failure(code: RecognizerErrorCode) -> RecognizerError {
    RecognizerError::new(code, "selected TOC facts or graph bindings disagree")
}
fn graph_error(error: wow_graph::GraphError) -> RecognizerError {
    failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}

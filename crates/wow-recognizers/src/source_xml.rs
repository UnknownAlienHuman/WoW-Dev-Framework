//! Static XML structure through the declarative matcher. Parsing, declaration
//! linking and inline-body ownership stay in the load owner. Each family reads a
//! real preceding graph snapshot and never reopens, reparses or executes source.
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
use wow_graph::{GraphConfidence, GraphCoverageRecord, GraphPartitionSnapshot, GraphProposalBatch};

pub const SOURCE_XML_PROFILE: &str = "wow-recognizers/xml-structural/1";
const FACT_PROFILE: &str = "wow-recognizers-xml-facts-1";
const MAX_FACTS: usize = 32_768;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceXmlFamily {
    Template,
    Object,
    ObjectParentage,
    Inherits,
    Script,
}
impl SourceXmlFamily {
    pub const ALL: [Self; 5] = [
        Self::Template,
        Self::Object,
        Self::ObjectParentage,
        Self::Inherits,
        Self::Script,
    ];
    pub const fn rule_id(self) -> &'static str {
        match self {
            Self::Template => "core.xml.template",
            Self::Object => "core.xml.object",
            Self::ObjectParentage => "core.xml.object",
            Self::Inherits => "core.xml.inherits",
            Self::Script => "core.xml.script",
        }
    }
    pub const fn partition_id(self) -> &'static str {
        match self {
            Self::Template => "wow-recognizers.xml-template",
            Self::Object => "wow-recognizers.xml-object",
            Self::ObjectParentage => "wow-recognizers.xml-object-parentage",
            Self::Inherits => "wow-recognizers.xml-inherits",
            Self::Script => "wow-recognizers.xml-script",
        }
    }
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::Template => "project.xml.template",
            Self::Object => "project.xml.object",
            Self::ObjectParentage => "project.xml.object",
            Self::Inherits => "project.xml.inherits",
            Self::Script => "project.xml.script",
        }
    }
}

/// The operation's retained facts, including excluded and unresolved occurrences.
/// Scope stays selected_toc/flavor/package as supplied by the standalone load
/// plan; a missing package identity is a reported limitation, never an inferred
/// owner.
pub struct SourceXmlFact<'a> {
    pub fact_id: &'a str,
    pub context_id: GenerationContextId,
    pub selected_toc: &'a str,
    pub flavor: &'a str,
    pub package: Option<&'a str>,
    pub document: &'a str,
    pub occurrence_id: &'a str,
    pub ordinal: u64,
    pub content_digest: ContentDigest<SourceContent>,
    pub span: SourceSpan,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
    pub kind: SourceXmlFactKind<'a>,
}

/// Source-declared element classification. `Element` stays generic; a Frame or
/// Region kind is never certified from an XML tag alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceXmlElementRole {
    Ui,
    Include,
    Script,
    Scripts,
    ScriptBinding,
    Element,
    UnknownNamespace,
}
impl SourceXmlElementRole {
    const fn name(self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Include => "include",
            Self::Script => "script",
            Self::Scripts => "scripts",
            Self::ScriptBinding => "script_binding",
            Self::Element => "element",
            Self::UnknownNamespace => "unknown_namespace",
        }
    }
}

/// Source-declared template state. Absent means the attribute was never spelled
/// in this document; it is never normalized into a default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceXmlTemplateState {
    Absent,
    False,
    True,
}
impl SourceXmlTemplateState {
    const fn name(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::False => "false",
            Self::True => "true",
        }
    }
}

/// Explicit `parent` reference resolution as retained by the load owner. Every
/// outcome except Unique keeps its exact spelling and reason preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceXmlParentResolution<'a> {
    Unique { target_occurrence_id: &'a str },
    Ambiguous { name_group: &'a str },
    NotInCapturedScope,
    DynamicName,
    UnsupportedName,
    InvalidSource,
    InvalidTarget { target_occurrence_id: &'a str },
}
impl SourceXmlParentResolution<'_> {
    const fn name(&self) -> &'static str {
        match self {
            Self::Unique { .. } => "unique",
            Self::Ambiguous { .. } => "ambiguous",
            Self::NotInCapturedScope => "not_in_captured_scope",
            Self::DynamicName => "dynamic_name",
            Self::UnsupportedName => "unsupported_name",
            Self::InvalidSource => "invalid_source",
            Self::InvalidTarget { .. } => "invalid_target",
        }
    }
    fn target(&self) -> Option<&str> {
        match self {
            Self::Unique {
                target_occurrence_id,
            }
            | Self::InvalidTarget {
                target_occurrence_id,
            } => Some(target_occurrence_id),
            _ => None,
        }
    }
}

/// Loader-exact order verdict for one reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceXmlReferenceOrder {
    TargetBeforeSource,
    TargetAfterSource,
    SelfReference,
    RepeatedLoad,
    Unrecorded,
}
impl SourceXmlReferenceOrder {
    const fn name(self) -> &'static str {
        match self {
            Self::TargetBeforeSource => "target_before_source",
            Self::TargetAfterSource => "target_after_source",
            Self::SelfReference => "self_reference",
            Self::RepeatedLoad => "repeated_load",
            Self::Unrecorded => "unrecorded",
        }
    }
}

/// Source-declared script binding classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceXmlScriptSource {
    ExternalFile,
    ReferenceOnly,
    InlineBody,
    Unresolved,
}
impl SourceXmlScriptSource {
    const fn name(self) -> &'static str {
        match self {
            Self::ExternalFile => "external_file",
            Self::ReferenceOnly => "reference_only",
            Self::InlineBody => "inline_body",
            Self::Unresolved => "unresolved",
        }
    }
}

pub enum SourceXmlFactKind<'a> {
    /// Structural declaration state of one element occurrence.
    Declaration {
        role: SourceXmlElementRole,
        element_name: &'a str,
        name: Option<&'a str>,
        virtual_template: SourceXmlTemplateState,
        intrinsic: SourceXmlTemplateState,
        mixin_names: &'a [String],
        valid_declaration: bool,
        /// Captured lexical containment, kept separate from the explicit
        /// `parent` name reference and never a runtime frame parent.
        parent_occurrence_id: Option<&'a str>,
    },
    /// The explicit `parent` attribute as resolved by the load owner.
    Parent {
        reference_id: &'a str,
        name: &'a str,
        resolution: SourceXmlParentResolution<'a>,
        order: Option<SourceXmlReferenceOrder>,
        cycle_id: Option<&'a str>,
    },
    /// A resolved local inheritance name.
    Inheritance {
        reference_id: &'a str,
        target_occurrence_id: &'a str,
        order: Option<SourceXmlReferenceOrder>,
        cycle_id: Option<&'a str>,
    },
    /// An inheritance name that produced no admissible target. The spelling and
    /// resolution stay retained so an unresolved name is never a clean negative.
    InheritanceUnresolved {
        reference_id: &'a str,
        name: &'a str,
    },
    /// A captured script binding. Embedded code is never executed or reparsed.
    Script {
        reference_id: &'a str,
        script_name: &'a str,
        source_kind: SourceXmlScriptSource,
        owner_occurrence_id: Option<&'a str>,
        inherit: Option<&'a str>,
        intrinsic_order: Option<&'a str>,
        file_reference: Option<&'a str>,
        function_reference: Option<&'a str>,
        method_reference: Option<&'a str>,
    },
}

pub struct SourceXmlInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub source_partition: &'a str,
    pub context: &'a GenerationContext,
    pub facts: &'a [SourceXmlFact<'a>],
    /// Analyzer-owned script bindings, already resolved to exact proposal
    /// identities. No name lookup or callable inference happens here.
    pub script_bindings: &'a [SourceXmlScriptBinding<'a>],
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}

/// One exact project-owned script binding. The receiver and handler are always
/// accepted proposal identities from the source input partition; the optional
/// semantic context is only present for inline XML handlers and always evaluates
/// to Possible. A named Lua function handler carries no context at all.
#[derive(Clone, Copy)]
pub struct SourceXmlScriptBinding<'a> {
    pub binding_id: &'a str,
    pub site_id: &'a str,
    pub script_id: &'a str,
    pub receiver_proposal_id: &'a str,
    pub handler_proposal_id: &'a str,
    /// Exact registered entity kind of the handler endpoint:
    /// `xml_source_handler` or `lua_source_function`.
    pub handler_kind: &'a str,
    /// The consuming element occurrence that inherited this binding site.
    pub consumer_occurrence_id: Option<&'a str>,
    pub inherited: bool,
    pub confidence: GraphConfidence,
    pub semantic_context: Option<crate::source_scripts::SourceScriptSemanticContext<'a>>,
    pub source_handle_ids: &'a [StableHandleId],
    pub evidence_ids: &'a [EvidenceId],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceXmlOmission {
    pub fact_ids: Vec<String>,
    pub blocker: &'static str,
}
/// Exact matcher outcomes, matches, support and proposal identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceXmlEvaluation {
    pub recipe: &'static str,
    pub pack_digest: String,
    pub fact_bundle: crate::RecognizerFactBundle,
    pub output: RecognizerOutputPartition,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceXmlReceipt {
    pub match_id: String,
    pub fact_ids: Vec<String>,
    pub entity_proposal_ids: Vec<String>,
    pub relation_proposal_ids: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceXmlRecognition {
    pub profile: &'static str,
    pub family: SourceXmlFamily,
    pub evaluations: Vec<SourceXmlEvaluation>,
    pub receipts: Vec<SourceXmlReceipt>,
    pub omissions: Vec<SourceXmlOmission>,
}
pub struct SourceXmlProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceXmlRecognition,
}
pub fn recognize_source_xml(
    input: SourceXmlInput<'_>,
    family: SourceXmlFamily,
    stop: &AtomicBool,
) -> RecognizerResult<SourceXmlProposals> {
    checkpoint(stop)?;
    adapt::validate(&input, stop)?;
    let seeds = adapt::seeds(&input, family, stop)?;
    project::execute(&input, family, seeds, stop)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Recipe {
    TemplateDeclared,
    ObjectDeclared,
    ParentOf,
    InheritsTemplate,
    ReferencesTemplate,
    ScriptSite,
    ScriptBinding,
}
impl Recipe {
    const fn family(self) -> SourceXmlFamily {
        match self {
            Self::TemplateDeclared => SourceXmlFamily::Template,
            Self::ObjectDeclared => SourceXmlFamily::Object,
            Self::ParentOf => SourceXmlFamily::ObjectParentage,
            Self::InheritsTemplate | Self::ReferencesTemplate => SourceXmlFamily::Inherits,
            Self::ScriptBinding => SourceXmlFamily::Script,
            Self::ScriptSite => SourceXmlFamily::Script,
        }
    }
    const fn name(self) -> &'static str {
        match self {
            Self::TemplateDeclared => "xml_template_declared",
            Self::ObjectDeclared => "xml_object_declared",
            Self::ParentOf => "xml_parent_of",
            Self::InheritsTemplate => "xml_inherits_template",
            Self::ReferencesTemplate => "xml_references_template",
            Self::ScriptSite => "xml_script_site",
            Self::ScriptBinding => "xml_script_binding",
        }
    }
    fn for_family(family: SourceXmlFamily) -> Vec<Self> {
        match family {
            SourceXmlFamily::Template => vec![Self::TemplateDeclared],
            SourceXmlFamily::Object => vec![Self::ObjectDeclared],
            SourceXmlFamily::ObjectParentage => vec![Self::ParentOf],
            SourceXmlFamily::Inherits => vec![Self::InheritsTemplate, Self::ReferencesTemplate],
            SourceXmlFamily::Script => vec![Self::ScriptSite, Self::ScriptBinding],
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
    RecognizerError::new(code, "retained XML facts or graph bindings disagree")
}
fn graph_error(error: wow_graph::GraphError) -> RecognizerError {
    failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}

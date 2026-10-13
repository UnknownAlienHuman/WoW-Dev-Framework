//! Closed, decode-only projection of the current `skeleton::Declaration` wire.
//!
//! This is a private handoff fragment for the first default Primary metadata
//! L0 GraphEntity pack reader. It does not construct native Project provenance,
//! graph views, or `ContextSemanticPack`. Mandatory fields are required; the
//! only omitted fields below are options the source writer explicitly omits.

use serde::{Deserialize, Serialize};
use wow_core::{ContentDigest, EvidenceId, SourceContent, SourceSpan, StableHandleId};
use wow_graph::GraphNode;

/// Matches the internally tagged `kind` enum in `wow-context/src/skeleton.rs`.
/// All current variants parse field-for-field. The selected default Primary L0
/// admission is narrower and is represented separately below.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DeclarationWire {
    Callable {
        node: GraphNode,
        metadata: ProjectCallableMetadataWire,
    },
    Lua {
        node: GraphNode,
        origin: ProjectGraphLuaDeclarationWire,
    },
    Xml {
        node: GraphNode,
        origin: ProjectGraphXmlDeclarationWire,
    },
    XmlHandler {
        node: GraphNode,
        origin: ProjectGraphInlineHandlerWire,
        source: ProjectGraphScriptSourceWire,
    },
    StateRoot {
        node: GraphNode,
        origin: ProjectGraphStateRootWire,
        declarations: Vec<ProjectGraphStateDeclarationWire>,
    },
    StatePath {
        node: GraphNode,
        origin: ProjectGraphStatePathWire,
    },
}

/// A structural parse is not enough to admit every writer variant to this
/// finite reader. XmlHandler and state rows belong to other L0 profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultPrimaryL0Admission {
    SupportedMetadata,
    UnsupportedProfileVariant,
}

impl DeclarationWire {
    #[must_use]
    pub const fn default_primary_l0_admission(&self) -> DefaultPrimaryL0Admission {
        match self {
            Self::Callable { .. } | Self::Lua { .. } | Self::Xml { .. } => {
                DefaultPrimaryL0Admission::SupportedMetadata
            }
            Self::XmlHandler { .. } | Self::StateRoot { .. } | Self::StatePath { .. } => {
                DefaultPrimaryL0Admission::UnsupportedProfileVariant
            }
        }
    }
}

/// Exact writer record from `wow-project/src/graph/callable_metadata.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectCallableMetadataWire {
    pub function: ProjectGraphFunctionWire,
    pub function_analysis_id: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub scope: ProjectCallableScopeWire,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_function_id: Option<String>,
    pub declaration: CallableDeclarationMetadataWire,
    pub signature: CallableSignatureMetadataWire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectCallableScopeWire {
    Chunk,
    ChunkScopeClosure,
    NestedClosure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGraphFunctionWire {
    pub function_id: String,
    pub path: String,
    pub kind: SourceFunctionKindWire,
    pub span: SourceSpan,
    pub proposal_id: String,
    pub ownership_proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionKindWire {
    Chunk,
    Closure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallableDeclarationMetadataWire {
    pub state: ProjectCallableFacetStateWire,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<PhysicalFunctionDeclarationRecipeWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation: Option<SourceFunctionDeclarationWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallableSignatureMetadataWire {
    pub state: ProjectCallableFacetStateWire,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<PhysicalFunctionSignatureRecipeWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation: Option<SourceFunctionSignatureWire>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectCallableFacetStateWire {
    NotSelected,
    Observed,
    Partial,
    Unavailable,
    Unsupported,
}

/// Local projections avoid adding wow-emmy as a direct wow-context dependency.
/// These are the exact two recipe strings accepted by the source owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalFunctionDeclarationRecipeWire {
    #[serde(rename = "wow-emmy/physical-function-declaration-syntax/1")]
    DeclaredSyntaxV1,
    #[serde(rename = "wow-emmy/physical-function-declaration-syntax-and-body-shape/1")]
    DeclaredSyntaxAndBodyShapeV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalFunctionSignatureRecipeWire {
    #[serde(rename = "wow-emmy/physical-function-indexed-types/1")]
    IndexedTypesV1,
}

/// SourceFunctionDeclaration is Serialize-only; mirror its exact tagged form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceFunctionDeclarationWire {
    Observed {
        form: SourceFunctionDeclarationFormWire,
        colon_definition: bool,
        declaration_span: SourceSpan,
        parameters: Vec<SourceDeclaredParameterWire>,
    },
    Unavailable {
        reason: SourceFunctionDeclarationUnavailableWire,
    },
    Unsupported {
        reason: SourceFunctionDeclarationUnsupportedWire,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionDeclarationFormWire {
    LocalFunction,
    NamedFunction,
    ClosureExpression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceDeclaredParameterWire {
    Name { name: String, span: SourceSpan },
    Vararg { span: SourceSpan },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionDeclarationUnavailableWire {
    ChunkHasNoDeclaration,
    MissingParent,
    MissingDeclarationName,
    MissingParameterList,
    MissingParameterToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionDeclarationUnsupportedWire {
    ExtendedClosure,
    ExtendedDeclaration,
    NamedIndexForm,
    VarargPlacement,
}

/// SourceFunctionSignature is Serialize-only and flattens this tagged enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceFunctionSignatureWire {
    Unavailable {
        reason: SourceSignatureUnavailableWire,
    },
    Observed {
        types: SourceSignatureComparableWire,
        return_resolution: SourceSignatureReturnResolutionWire,
        async_state: SourceSignatureAsyncStateWire,
        vararg: bool,
        generic_parameter_count: u64,
        callable_overload_count: u64,
        unmatched_parameter_document_count: u64,
        unavailable_reasons: Vec<SourceSignatureUnavailableWire>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSignatureComparableWire {
    pub colon_definition: bool,
    pub parameters: Vec<SourceSignatureParameterWire>,
    pub returns: Vec<SourceSignatureReturnWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSignatureParameterWire {
    pub type_shape: SourceSignatureTypeWire,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_nullable: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSignatureReturnWire {
    pub types: Vec<SourceSignatureTypeWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceSignatureTypeWire {
    Nil,
    Boolean,
    Integer,
    Number,
    String,
    Thread,
    Userdata,
    Never,
    Array {
        length: SourceSignatureArrayLengthWire,
        element: Box<SourceSignatureTypeWire>,
    },
    Tuple {
        elements: Vec<SourceSignatureTypeWire>,
    },
    Union {
        members: Vec<SourceSignatureTypeWire>,
    },
    Unavailable {
        reason: SourceSignatureTypeUnavailableWire,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SourceSignatureArrayLengthWire {
    Unspecified,
    Maximum(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSignatureReturnResolutionWire {
    Unresolved,
    DocumentResolved,
    InferenceResolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSignatureAsyncStateWire {
    Unspecified,
    Async,
    Sync,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSignatureUnavailableWire {
    ChunkHasNoSignature,
    MissingSignatureIndex,
    GenericSignature,
    CallableOverloads,
    VariadicSignature,
    UnmatchedParameterDocuments,
    UnresolvedReturns,
    UnresolvedOrUnsupportedType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSignatureTypeUnavailableWire {
    MissingParameterType,
    UnresolvedType,
    DynamicType,
    GenericType,
    NominalType,
    CallableType,
    LiteralType,
    ObjectType,
    VariadicType,
    UnsupportedNativeType,
    EmptyUnion,
}

/// Exact ProjectGraphLuaDeclaration, XmlDeclaration, and script-origin records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGraphLuaDeclarationWire {
    pub declaration_id: String,
    pub path: String,
    pub span: SourceSpan,
    pub proposal_id: String,
    pub ownership_proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGraphXmlDeclarationWire {
    pub occurrence_id: String,
    pub path: String,
    pub proposal_id: String,
    pub ownership_proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGraphInlineHandlerWire {
    pub script_id: String,
    pub unit_id: String,
    pub document: String,
    pub semantic_context: XmlLuaSemanticContextWire,
    pub proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XmlLuaSemanticContextWire {
    pub profile: String,
    pub context_id: String,
    pub script_site: String,
    pub implicit_receiver: String,
    pub runtime_dispatch: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGraphScriptSourceWire {
    pub script_id: String,
    pub document: String,
    pub script_name: String,
    pub source_kind: XmlScriptSourceWire,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaring_owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intrinsic_order: Option<String>,
    pub span: SourceSpan,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlScriptSourceWire {
    ExternalFile,
    ReferenceOnly,
    InlineBody,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGraphStateRootWire {
    pub root_id: String,
    pub name: String,
    pub scope: TocSavedVariableScopeWire,
    pub document: String,
    pub proposal_id: String,
    pub ambiguous: bool,
    pub source_handle_ids: Vec<StableHandleId>,
    pub evidence_ids: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGraphStateDeclarationWire {
    pub record_ordinal: u64,
    pub entry_ordinal: u32,
    pub name: String,
    pub scope: TocSavedVariableScopeWire,
    pub state: TocSavedVariableStateWire,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGraphStatePathWire {
    pub path_id: String,
    pub root_id: String,
    pub keys: Vec<GlobalAccessKeyWire>,
    pub proposal_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum GlobalAccessKeyWire {
    String(String),
    // The owner serializes this signed value as a decimal string, not a JSON
    // number. This custom decoder keeps that exact representation and range.
    Integer(ExactLuaIntegerWire),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactLuaIntegerWire(i64);

impl Serialize for ExactLuaIntegerWire {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for ExactLuaIntegerWire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let value = raw.parse::<i64>().map_err(serde::de::Error::custom)?;
        if !(-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value)
            || value.to_string() != raw
        {
            return Err(serde::de::Error::custom("noncanonical Lua integer key"));
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TocSavedVariableScopeWire {
    Account,
    Character,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TocSavedVariableStateWire {
    Declared,
    UnsupportedIdentifier,
}

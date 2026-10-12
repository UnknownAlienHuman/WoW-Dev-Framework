//! Finite transport intent for existing native namespace/catalog consumers.
//! Decoding selection data never constructs a native source or candidate.
use super::{
    LiveProjectPublishRequest, LiveProjectResult, LiveProjectStore, LiveProjectUpdateRequest,
    PlatformStoreSelection, fail, publication_checkpoint, publish_input_in_namespace,
    update_input_in_namespace,
};
use crate::{LocalProjectInput, ServiceErrorCode, ServiceResult, graph::GraphBuildRequest};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_core::ProfileId;
use wow_emmy::function_calls::{
    PhysicalFunctionDeclarationRecipe, PhysicalFunctionSignatureRecipe,
};
use wow_project::{ProjectId, skeleton_input::SkeletonInputBinding};

pub const PLATFORM_PROJECT_SELECTION_SCHEMA: &str = "wow-service/platform-project-selection/1";
pub const PLATFORM_PROJECT_SELECTION_MAX_BYTES: usize =
    super::super::graph::GRAPH_REQUEST_MAX_BYTES;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DeclarationCatalog {
    Absent,
    DeclaredSyntaxV1,
    DeclaredSyntaxAndBodyShapeV1,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SignatureCatalog {
    Absent,
    IndexedTypesV1,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogIntent {
    xml_schema_classification: bool,
    physical_function_declarations: DeclarationCatalog,
    physical_function_signatures: SignatureCatalog,
    composed_candidate: bool,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SelectionDocument {
    schema: String,
    owner_project_id: ProjectId,
    source_profile_id: ProfileId,
    catalogs: CatalogIntent,
}

/// Explicit transport selection, validated against the actual retained epoch
/// and genuine configuration by the existing namespace operations.
pub struct PlatformProjectSelection {
    document: SelectionDocument,
    selection: PlatformStoreSelection,
}
impl PlatformProjectSelection {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let document: SelectionDocument = crate::graph::input::decode(
            bytes,
            PLATFORM_PROJECT_SELECTION_MAX_BYTES,
            256,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|failure| fail(failure.code))?;
        Self::from_document(document, stop)
    }
    pub(crate) fn from_document(
        document: SelectionDocument,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        if document.schema != PLATFORM_PROJECT_SELECTION_SCHEMA {
            return Err(fail(ServiceErrorCode::InvalidRequest));
        }
        let mut selection = PlatformStoreSelection::new(
            document.owner_project_id.clone(),
            document.source_profile_id.clone(),
        )?;
        if document.catalogs.xml_schema_classification {
            selection = selection.with_xml_classification_catalog();
        }
        selection = match document.catalogs.physical_function_declarations {
            DeclarationCatalog::Absent => selection,
            DeclarationCatalog::DeclaredSyntaxV1 => selection
                .with_physical_function_declarations_recipe_catalog(
                    PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1,
                ),
            DeclarationCatalog::DeclaredSyntaxAndBodyShapeV1 => selection
                .with_physical_function_declarations_recipe_catalog(
                    PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1,
                ),
        };
        if matches!(
            document.catalogs.physical_function_signatures,
            SignatureCatalog::IndexedTypesV1
        ) {
            selection = selection.with_physical_function_signatures_catalog(
                PhysicalFunctionSignatureRecipe::IndexedTypesV1,
            );
        }
        if document.catalogs.composed_candidate {
            selection = selection.with_composed_candidate_catalog();
        }
        selection.validate()?;
        publication_checkpoint(stop)?;
        Ok(Self {
            document,
            selection,
        })
    }
    pub(crate) fn owner(&self) -> &PlatformStoreSelection {
        &self.selection
    }
    pub(crate) fn require_candidate(&self) -> ServiceResult<()> {
        if !self.document.catalogs.composed_candidate {
            return Err(fail(ServiceErrorCode::InvalidRequest));
        }
        Ok(())
    }
    pub(crate) fn intent(&self) -> &impl Serialize {
        &self.document
    }
}

/// One acquisition and the existing publication/CAS owner. Recipe selection is
/// bound to this admitted input; namespace/catalog mismatch never falls back.
pub fn publish_selected_local_project(
    config: &Path,
    graph: &GraphBuildRequest,
    root: &Path,
    selection: &PlatformProjectSelection,
    request: &LiveProjectPublishRequest,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectResult> {
    publication_checkpoint(stop)?;
    let input = LocalProjectInput::from_config_path(config, stop)?;
    let graph = graph.clone().with_input_profile(&input);
    super::skeleton::preflight(&graph, crate::graph::GRAPH_REQUEST_MAX_BYTES, stop)?;
    publish_input_in_namespace(input, &graph, root, selection.owner(), request, stop)
}

/// Apply explicit final inputs in the exact selected namespace. Candidate epochs
/// require native/core/query recomposition and the existing validated store CAS.
pub fn update_selected_local_project(
    config: &Path,
    graph: &GraphBuildRequest,
    root: &Path,
    selection: &PlatformProjectSelection,
    request: &LiveProjectUpdateRequest,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectResult> {
    publication_checkpoint(stop)?;
    let input = LocalProjectInput::from_config_path(config, stop)?;
    let graph = graph.clone().with_input_profile(&input);
    super::skeleton::preflight(&graph, crate::graph::GRAPH_REQUEST_MAX_BYTES, stop)?;
    update_input_in_namespace(input, &graph, root, selection.owner(), request, stop)
}

pub struct SelectedLiveProjectReadResult {
    bytes: Vec<u8>,
}
impl SelectedLiveProjectReadResult {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn exit_code(&self) -> u8 {
        2
    }
}
#[derive(Serialize)]
struct CandidateReadIdentity<'a> {
    candidate_id: &'a str,
    candidate_digest: &'a str,
    skeleton_binding: SkeletonInputBinding,
}
#[derive(Serialize)]
struct SelectedReadBody<'a> {
    schema: &'static str,
    selection: &'a SelectionDocument,
    requested_store_generation: &'a str,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    project_snapshot_id: &'a str,
    graph_snapshot_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate: Option<CandidateReadIdentity<'a>>,
    state: &'static str,
}

/// Source-absent reopen uses the selected native replay/recognizer owner once.
/// The returned identities are inert data for a subsequent explicit read.
pub fn read_selected_live_project(
    root: &Path,
    selection: &PlatformProjectSelection,
    generation: &str,
    stop: &AtomicBool,
) -> ServiceResult<SelectedLiveProjectReadResult> {
    publication_checkpoint(stop)?;
    let selector = super::operations::read_selector(generation)?;
    let store = LiveProjectStore::open_in_namespace(root, selection.owner())?;
    let read = store.read_in_namespace(selection.owner(), &selector, stop)?;
    let candidate = read
        .candidate()
        .map(|candidate| {
            Ok::<_, crate::ServiceError>(CandidateReadIdentity {
                candidate_id: candidate.manifest().candidate_id(),
                candidate_digest: candidate.manifest().candidate_digest(),
                skeleton_binding: SkeletonInputBinding::from_native(
                    candidate.native(),
                    candidate.graph(),
                )
                .map_err(super::project_error)?,
            })
        })
        .transpose()?;
    // Selected candidate catalog may never yield an ordinary pair response.
    if selection.document.catalogs.composed_candidate && candidate.is_none() {
        return Err(fail(ServiceErrorCode::IdentityMismatch));
    }
    let body = SelectedReadBody {
        schema: "wow-service/selected-live-project-read/1",
        selection: &selection.document,
        requested_store_generation: generation,
        publication_set_id: read.publication_set_id(),
        store_generation_id: read.store_generation_id().as_str(),
        project_snapshot_id: read.project().snapshot_id(),
        graph_snapshot_id: read.graph().snapshot().snapshot_id().as_str(),
        candidate,
        state: "partial",
    };
    let bytes = super::skeleton::encode(&body, 128 * 1024, stop)?;
    publication_checkpoint(stop)?;
    Ok(SelectedLiveProjectReadResult { bytes })
}

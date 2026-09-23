use serde::Deserialize;
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity, ProfileKind, SourceContent};
use wow_emmy::{
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
    ProjectConfigurationBuilder, ProjectFileRole, ProjectId, ProjectInputBundle, ProjectInputFile,
    ProjectKind, ProjectLanguageKind, ProjectSourceOriginId, ProjectWorkspaceId,
};
use wow_reference::ReferenceView;

use crate::{ServiceError, ServiceErrorCode, ServiceResult};

/// Hard transport ceiling, checked before deserializing a materialized project.
pub const LOCAL_INPUT_MAX_BYTES: usize = 32 * 1024 * 1024;
pub const LOCAL_INPUT_SCHEMA: &str = "wow-service/local-project-input/1";

/// Explicit materialized inputs, not precomputed findings or a trusted clean result.
/// Source bodies are retained only inside the lower project/analyzer owners.
pub struct LocalProjectInput {
    pub(super) bundle: ProjectInputBundle,
    pub(super) reference: ReferenceView,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireInput {
    schema: String,
    project_id: ProjectId,
    workspace_id: ProjectWorkspaceId,
    source_origin_id: ProjectSourceOriginId,
    logical_root: String,
    profile: ProfileIdentity,
    reference_view: ReferenceView,
    analyzer: AnalyzerInput,
    main_files: Vec<SourceInput>,
    library_files: Vec<SourceInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnalyzerInput {
    /// Original compatibility-report JSON; the analyzer owns its verification.
    compatibility_report_json: String,
    accepted_pin_id: String,
    configuration_digest: ContentDigest<CanonicalResult>,
    contract_id: String,
    fixture_contract_id: String,
    library_contract_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceInput {
    path: String,
    text: String,
    content_digest: ContentDigest<SourceContent>,
    byte_length: u64,
}

impl LocalProjectInput {
    /// Loads explicit input bytes without filesystem discovery or Lua execution.
    /// No analyzer or rule execution occurs until the service receives `check`.
    pub fn from_json_slice(bytes: &[u8]) -> ServiceResult<Self> {
        if bytes.len() > LOCAL_INPUT_MAX_BYTES {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "local input exceeds its byte limit",
            ));
        }
        let input: WireInput = serde_json::from_slice(bytes).map_err(|_| {
            ServiceError::new(
                ServiceErrorCode::InvalidConfiguration,
                "invalid local project input",
            )
        })?;
        if input.schema != LOCAL_INPUT_SCHEMA {
            return Err(invalid("unsupported local project input schema"));
        }
        input
            .profile
            .validate()
            .map_err(|_| invalid("invalid selected profile"))?;
        input
            .reference_view
            .validate()
            .map_err(|_| invalid("invalid reference view"))?;
        let reference_generation = input
            .reference_view
            .generation_id()
            .parse()
            .map_err(|_| invalid("invalid reference generation"))?;
        let backend = wow_emmy::backend_identity_from_report(
            input.analyzer.compatibility_report_json.as_bytes(),
        )
        .map_err(|_| invalid("analyzer compatibility report was rejected"))?;
        let probe = backend.compatibility_report_sha256().to_owned();
        let declaration = AnalyzerBindingDeclaration::new(
            input.analyzer.contract_id,
            input.analyzer.accepted_pin_id,
            probe,
            input.analyzer.configuration_digest,
            input.analyzer.fixture_contract_id,
            input.analyzer.library_contract_id,
            backend.clone(),
        )
        .map_err(|_| invalid("analyzer declaration does not match the compiled backend"))?;
        let (kind, universe) = match input.profile.profile_kind() {
            ProfileKind::Fixture => (ProjectKind::Fixture, LuaWorkspaceUniverse::Fixture),
            ProfileKind::Release => (ProjectKind::Repository, LuaWorkspaceUniverse::Project),
        };
        let policy = ProjectBudgetPolicy::new(
            1024,
            16 * 1024 * 1024,
            1024 * 1024,
            1024,
            262_144,
            65_536,
            16 * 1024 * 1024,
        )
        .map_err(|_| invalid("invalid local project budgets"))?;
        let configuration = ProjectConfigurationBuilder::new(
            input.project_id,
            kind,
            input.profile,
            reference_generation,
            declaration,
        )
        .workspace_id(input.workspace_id)
        .source_origin_id(input.source_origin_id)
        .logical_root(input.logical_root)
        .capability_policy(
            ProjectCapabilityPolicy::degraded_e0()
                .map_err(|_| invalid("invalid capability policy"))?,
        )
        .budget_policy(policy)
        .build()
        .map_err(|_| invalid("project configuration was rejected"))?;
        let main = admit_files(input.main_files, ProjectFileRole::FirstPartyMain)?;
        // Libraries use the same exact-byte admission, but never enter Main.
        let library_inputs = input.library_files;
        if library_inputs.is_empty() {
            return Err(invalid(
                "an explicit nonempty Library inventory is required",
            ));
        }
        let _ = admit_files(
            library_inputs
                .iter()
                .map(|file| SourceInput {
                    path: file.path.clone(),
                    text: file.text.clone(),
                    content_digest: file.content_digest,
                    byte_length: file.byte_length,
                })
                .collect(),
            ProjectFileRole::FirstPartyMain,
        )?;
        let libraries = LuaWorkspaceSnapshot::build(
            backend,
            universe,
            library_inputs
                .into_iter()
                .map(|file| LuaWorkspaceFileInput::new(file.path, file.text))
                .collect(),
            LuaWorkspaceLimits::new(1024, 4096, 1024 * 1024, 16 * 1024 * 1024)
                .map_err(|_| invalid("invalid Library limits"))?,
        )
        .map_err(|_| invalid("Library input was rejected"))?;
        let bundle = ProjectInputBundle::closed(configuration, main, vec![libraries])
            .map_err(|_| invalid("Main input inventory was rejected"))?;
        Ok(Self {
            bundle,
            reference: input.reference_view,
        })
    }

    /// Compose already validated lower-owner inputs without a transport decoder.
    pub fn new(bundle: ProjectInputBundle, reference: ReferenceView) -> ServiceResult<Self> {
        bundle
            .configuration()
            .validate()
            .map_err(|_| invalid("project configuration was rejected"))?;
        reference
            .validate()
            .map_err(|_| invalid("reference view was rejected"))?;
        if reference.generation_id() != bundle.configuration().reference_generation().to_string() {
            return Err(invalid("reference generation does not match the project"));
        }
        Ok(Self { bundle, reference })
    }
}

fn admit_files(
    files: Vec<SourceInput>,
    role: ProjectFileRole,
) -> ServiceResult<Vec<ProjectInputFile>> {
    if files.is_empty() || files.len() > 1024 {
        return Err(invalid(
            "source inventory must contain between 1 and 1024 files",
        ));
    }
    let mut total = 0usize;
    files
        .into_iter()
        .map(|file| {
            total = total
                .checked_add(file.text.len())
                .ok_or_else(|| invalid("source size overflow"))?;
            if file.text.len() > 1024 * 1024 || total > 16 * 1024 * 1024 {
                return Err(invalid("source inventory exceeds local byte limits"));
            }
            ProjectInputFile::declared_with_identity(
                file.path,
                file.text.into_bytes(),
                ProjectLanguageKind::Lua,
                role,
                file.content_digest,
                file.byte_length,
                None::<String>,
            )
            .map_err(|_| invalid("source path, content digest or length was rejected"))
        })
        .collect()
}

fn invalid(message: &'static str) -> ServiceError {
    ServiceError::new(ServiceErrorCode::InvalidConfiguration, message)
}

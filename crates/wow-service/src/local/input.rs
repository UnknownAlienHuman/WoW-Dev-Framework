use serde::Deserialize;
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity, ProfileKind, SourceContent};
use wow_emmy::{LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
    ProjectConfigurationBuilder, ProjectFileRole, ProjectId, ProjectInputBundle, ProjectInputFile,
    ProjectKind, ProjectLanguageKind, ProjectSourceOriginId, ProjectWorkspaceId,
};
use wow_reference::ReferenceView;

use crate::{ServiceError, ServiceErrorCode, ServiceResult};

/// Hard transport ceiling, checked before deserializing a materialized project.
pub const LOCAL_INPUT_MAX_BYTES: usize = wow_project::disk::DISK_CONFIGURATION_MAX_BYTES;
pub const LOCAL_INPUT_SCHEMA: &str = "wow-service/local-project-input/1";

/// Explicit materialized inputs, not precomputed findings or a trusted clean result.
/// Source bodies are retained only inside the lower project/analyzer owners.
pub struct LocalProjectInput {
    pub(super) bundle: ProjectInputBundle,
    pub(super) reference: ReferenceView,
    pub(super) load: LocalProjectLoad,
    pub(super) native_input: Option<std::sync::Arc<super::native_input::NativeInputEvidence>>,
    pub(super) native_artifact:
        Option<std::sync::Arc<super::native_artifact::NativeArtifactEvidence>>,
}

impl LocalProjectInput {
    pub(crate) fn project_bundle(&self) -> &ProjectInputBundle {
        &self.bundle
    }
}

pub(super) enum LocalProjectLoad {
    Explicit,
    SelectedToc(wow_project::load::ProjectLoadPlan),
    PackageUniverse {
        load_plan: wow_project::load::ProjectPackageLoadPlan,
        main_plan: wow_project::load::ProjectPackageMainPlan,
    },
}

#[derive(Clone, Copy)]
pub(super) enum LocalProjectLoadEvidence<'a> {
    Explicit,
    SelectedToc(&'a wow_project::load::ProjectLoadPlan),
    PackageUniverse {
        load_plan: &'a wow_project::load::ProjectPackageLoadPlan,
        main_plan: &'a wow_project::load::ProjectPackageMainPlan,
    },
}

impl LocalProjectLoad {
    #[must_use]
    pub(super) const fn evidence(&self) -> LocalProjectLoadEvidence<'_> {
        match self {
            Self::Explicit => LocalProjectLoadEvidence::Explicit,
            Self::SelectedToc(plan) => LocalProjectLoadEvidence::SelectedToc(plan),
            Self::PackageUniverse {
                load_plan,
                main_plan,
            } => LocalProjectLoadEvidence::PackageUniverse {
                load_plan,
                main_plan,
            },
        }
    }
}

impl<'a> LocalProjectLoadEvidence<'a> {
    #[must_use]
    pub(super) const fn selected_toc(self) -> Option<&'a wow_project::load::ProjectLoadPlan> {
        match self {
            Self::SelectedToc(plan) => Some(plan),
            Self::Explicit | Self::PackageUniverse { .. } => None,
        }
    }
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
pub(super) struct AnalyzerInput {
    /// Original compatibility-report JSON; the analyzer owns its verification.
    pub(super) compatibility_report_json: String,
    pub(super) accepted_pin_id: String,
    pub(super) configuration_digest: ContentDigest<CanonicalResult>,
    pub(super) contract_id: String,
    pub(super) fixture_contract_id: String,
    pub(super) library_contract_id: String,
}

pub(super) struct ProjectMetadata {
    pub(super) project_id: ProjectId,
    pub(super) workspace_id: ProjectWorkspaceId,
    pub(super) source_origin_id: ProjectSourceOriginId,
    pub(super) logical_root: String,
    pub(super) profile: ProfileIdentity,
    pub(super) analyzer: AnalyzerInput,
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
    #[must_use]
    pub fn reference_view(&self) -> &ReferenceView {
        &self.reference
    }

    /// Loads explicit input bytes without filesystem discovery or Lua execution.
    /// No analyzer or rule execution occurs until the service receives `check`.
    pub fn from_json_slice(bytes: &[u8]) -> ServiceResult<Self> {
        if bytes.len() > LOCAL_INPUT_MAX_BYTES {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "local input exceeds its byte limit",
            ));
        }
        let input: WireInput =
            serde_json::from_slice(bytes).map_err(|_| invalid("invalid local project input"))?;
        if input.schema != LOCAL_INPUT_SCHEMA {
            return Err(invalid("unsupported local project input schema"));
        }
        Self::assemble(
            ProjectMetadata {
                project_id: input.project_id,
                workspace_id: input.workspace_id,
                source_origin_id: input.source_origin_id,
                logical_root: input.logical_root,
                profile: input.profile,
                analyzer: input.analyzer,
            },
            input.reference_view,
            admit_files(input.main_files, ProjectFileRole::FirstPartyMain)?,
            admit_files(input.library_files, ProjectFileRole::Library)?,
        )
    }

    /// Both inline and on-disk transports converge on the same owner composition.
    pub(super) fn assemble(
        input: ProjectMetadata,
        reference: ReferenceView,
        main: Vec<ProjectInputFile>,
        libraries: Vec<ProjectInputFile>,
    ) -> ServiceResult<Self> {
        Self::assemble_with_load(
            input,
            reference,
            main,
            libraries,
            LocalProjectLoad::Explicit,
        )
    }

    pub(super) fn assemble_with_load_plan(
        input: ProjectMetadata,
        reference: ReferenceView,
        main: Vec<ProjectInputFile>,
        libraries: Vec<ProjectInputFile>,
        load_plan: Option<wow_project::load::ProjectLoadPlan>,
    ) -> ServiceResult<Self> {
        let load = load_plan.map_or(LocalProjectLoad::Explicit, LocalProjectLoad::SelectedToc);
        Self::assemble_with_load(input, reference, main, libraries, load)
    }

    pub(super) fn assemble_with_package_load(
        input: ProjectMetadata,
        reference: ReferenceView,
        main: Vec<ProjectInputFile>,
        libraries: Vec<ProjectInputFile>,
        load_plan: wow_project::load::ProjectPackageLoadPlan,
        main_plan: wow_project::load::ProjectPackageMainPlan,
    ) -> ServiceResult<Self> {
        Self::assemble_with_load(
            input,
            reference,
            main,
            libraries,
            LocalProjectLoad::PackageUniverse {
                load_plan,
                main_plan,
            },
        )
    }

    fn assemble_with_load(
        input: ProjectMetadata,
        reference: ReferenceView,
        main: Vec<ProjectInputFile>,
        libraries: Vec<ProjectInputFile>,
        load: LocalProjectLoad,
    ) -> ServiceResult<Self> {
        match &load {
            LocalProjectLoad::Explicit => {}
            LocalProjectLoad::SelectedToc(plan) => {
                plan.validate_main_files(&main)
                    .map_err(|_| invalid("TOC plan does not match Main input"))?;
            }
            LocalProjectLoad::PackageUniverse {
                load_plan,
                main_plan,
            } => {
                load_plan
                    .validate_profile(&input.profile)
                    .map_err(|_| invalid("package load plan target mismatch"))?;
                main_plan
                    .validate_load_plan(load_plan)
                    .map_err(|_| invalid("package Main namespace plan mismatch"))?;
                main_plan
                    .validate_main_files(&main)
                    .map_err(|_| invalid("package Main namespace does not match Main input"))?;
            }
        }
        input
            .profile
            .validate()
            .map_err(|_| invalid("invalid selected profile"))?;
        reference
            .validate()
            .map_err(|_| invalid("invalid reference view"))?;
        let reference_generation = reference
            .generation_id()
            .parse()
            .map_err(|_| invalid("invalid reference generation"))?;
        let backend = wow_emmy::backend_identity_from_report(
            input.analyzer.compatibility_report_json.as_bytes(),
        )
        .map_err(|_| invalid("analyzer compatibility report was rejected"))?;
        let declaration = AnalyzerBindingDeclaration::new(
            input.analyzer.contract_id,
            input.analyzer.accepted_pin_id,
            backend.compatibility_report_sha256().to_owned(),
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
        let (max_files, max_total_source_bytes) = match &load {
            LocalProjectLoad::PackageUniverse { .. } => (4096, 64 * 1024 * 1024),
            LocalProjectLoad::Explicit | LocalProjectLoad::SelectedToc(_) => {
                (1024, 16 * 1024 * 1024)
            }
        };
        let policy = ProjectBudgetPolicy::new(
            max_files,
            max_total_source_bytes,
            1024 * 1024,
            max_files,
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
        .budget_policy(policy);
        let configuration = match &load {
            LocalProjectLoad::Explicit => configuration,
            LocalProjectLoad::SelectedToc(plan) => configuration
                .load_plan(plan)
                .map_err(|_| invalid("TOC plan target mismatch"))?,
            LocalProjectLoad::PackageUniverse {
                load_plan,
                main_plan,
            } => configuration
                .package_load_plan(load_plan, main_plan)
                .map_err(|_| invalid("package load plan target mismatch"))?,
        };
        let configuration = configuration
            .build()
            .map_err(|_| invalid("project configuration was rejected"))?;
        let libraries = LuaWorkspaceSnapshot::build(
            backend,
            universe,
            libraries
                .into_iter()
                .map(ProjectInputFile::into_workspace_input)
                .collect(),
            LuaWorkspaceLimits::new(1024, 4096, 1024 * 1024, 16 * 1024 * 1024)
                .map_err(|_| invalid("invalid Library limits"))?,
        )
        .map_err(|_| invalid("Library input was rejected"))?;
        let bundle = ProjectInputBundle::closed(configuration, main, vec![libraries])
            .map_err(|_| invalid("Main input inventory was rejected"))?;
        Self::new_with_load(bundle, reference, load)
    }

    /// Compose already validated lower-owner inputs without a transport decoder.
    pub fn new(bundle: ProjectInputBundle, reference: ReferenceView) -> ServiceResult<Self> {
        Self::new_with_load(bundle, reference, LocalProjectLoad::Explicit)
    }

    pub fn new_with_load_plan(
        bundle: ProjectInputBundle,
        reference: ReferenceView,
        load_plan: Option<wow_project::load::ProjectLoadPlan>,
    ) -> ServiceResult<Self> {
        let load = load_plan.map_or(LocalProjectLoad::Explicit, LocalProjectLoad::SelectedToc);
        Self::new_with_load(bundle, reference, load)
    }

    /// Compose exact package and Main namespace receipts from their native owner.
    pub fn new_with_package_plans(
        bundle: ProjectInputBundle,
        reference: ReferenceView,
        load_plan: wow_project::load::ProjectPackageLoadPlan,
        main_plan: wow_project::load::ProjectPackageMainPlan,
    ) -> ServiceResult<Self> {
        Self::new_with_load(
            bundle,
            reference,
            LocalProjectLoad::PackageUniverse {
                load_plan,
                main_plan,
            },
        )
    }

    fn new_with_load(
        bundle: ProjectInputBundle,
        reference: ReferenceView,
        load: LocalProjectLoad,
    ) -> ServiceResult<Self> {
        let configuration = bundle.configuration();
        match &load {
            LocalProjectLoad::Explicit => {
                if configuration.load_plan_digest().is_some()
                    || configuration.package_load_plan_digest().is_some()
                    || configuration.package_main_plan_digest().is_some()
                {
                    return Err(invalid(
                        "project configuration unexpectedly retains load provenance",
                    ));
                }
            }
            LocalProjectLoad::SelectedToc(plan) => {
                if configuration.load_plan_digest() != Some(plan.digest())
                    || configuration.package_load_plan_digest().is_some()
                    || configuration.package_main_plan_digest().is_some()
                {
                    return Err(invalid("project configuration and TOC receipt differ"));
                }
                plan.validate_profile(configuration.selected_profile())
                    .map_err(|_| invalid("TOC plan target mismatch"))?;
                plan.validate_main_files(bundle.inventory().files())
                    .map_err(|_| invalid("TOC plan source mismatch"))?;
            }
            LocalProjectLoad::PackageUniverse {
                load_plan,
                main_plan,
            } => {
                if configuration.load_plan_digest().is_some()
                    || configuration.package_load_plan_digest() != Some(load_plan.digest())
                    || configuration.package_main_plan_digest() != Some(main_plan.digest())
                {
                    return Err(invalid(
                        "project configuration and package load receipts differ",
                    ));
                }
                load_plan
                    .validate_profile(configuration.selected_profile())
                    .map_err(|_| invalid("package load plan target mismatch"))?;
                main_plan
                    .validate_load_plan(load_plan)
                    .map_err(|_| invalid("package Main namespace plan mismatch"))?;
                main_plan
                    .validate_main_files(bundle.inventory().files())
                    .map_err(|_| invalid("package Main namespace source mismatch"))?;
            }
        }
        configuration
            .validate()
            .map_err(|_| invalid("project configuration was rejected"))?;
        reference
            .validate()
            .map_err(|_| invalid("reference view was rejected"))?;
        if reference.generation_id() != configuration.reference_generation().to_string() {
            return Err(invalid("reference generation does not match the project"));
        }
        Ok(Self {
            bundle,
            reference,
            load,
            native_input: None,
            native_artifact: None,
        })
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

pub(super) fn invalid(message: &'static str) -> ServiceError {
    ServiceError::new(ServiceErrorCode::InvalidConfiguration, message)
}

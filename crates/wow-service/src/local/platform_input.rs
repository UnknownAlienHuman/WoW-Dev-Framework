//! Data-only platform transport. Source, package/Main, XML and graph authority
//! comes from retained native owners, never from a decoded owner receipt.
use std::{sync::Arc, sync::atomic::AtomicBool};

use serde::Deserialize;
use wow_project::{
    PlatformGraphProfile, ProjectBudgetPolicy, ProjectFileRole, ProjectId, ProjectSourceOriginId,
    ProjectWorkspaceId,
    disk::{ManifestedPlatformInput, ProjectDiskFile, ProjectInputDirectory},
    load::{
        ProjectPackageInput, TocLoadContext,
        schema::{
            XmlSchemaSelection, admit_xml_schema,
            classification::{
                ReviewedXmlClassificationProfile, XmlClassificationProfileSelection,
                admit_xml_classification_profile,
            },
        },
    },
    platform_source::{
        AdmittedPlatformSource, BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest,
        PlatformPackageRetentionLimits,
    },
};
use wow_reference::ReferenceView;

use super::{
    LocalProjectInput,
    disk_input::{DiskAnalyzer, DiskInventory, acquisition_error},
    input::{LocalProjectLoad, PlatformAssembly, ProjectMetadata, invalid},
    native_reference::{NativeProfile, project_native_reference},
    native_resources::present,
    native_source::NativeSourceInput,
};
use crate::ServiceResult;

pub const LOCAL_PLATFORM_SCHEMA: &str = "wow-service/local-platform-source/1";
/// Retained native platform input with explicit validated project budgets.
pub const LOCAL_PLATFORM_BUDGET_SCHEMA: &str = "wow-service/local-platform-source/2";

#[cfg(test)]
mod tests;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformInput {
    schema: String,
    project_id: ProjectId,
    workspace_id: ProjectWorkspaceId,
    source_origin_id: ProjectSourceOriginId,
    logical_root: String,
    source_profile: BlizzardUiSourceProfileRequest,
    source: ManifestedPlatformInput,
    #[serde(default, deserialize_with = "present")]
    reference_view: Option<ProjectDiskFile>,
    #[serde(default, deserialize_with = "present")]
    native_reference: Option<NativeReferenceInput>,
    analyzer: DiskAnalyzer,
    packages: Vec<ProjectPackageInput>,
    #[serde(default, deserialize_with = "present")]
    retained_package_limits: Option<PlatformPackageRetentionLimits>,
    #[serde(default, deserialize_with = "present")]
    project_budgets: Option<ProjectBudgetPolicy>,
    #[serde(default)]
    load_context: Option<TocLoadContext>,
    graph_profile: PlatformGraphProfile,
    #[serde(default, deserialize_with = "present")]
    physical_function_declarations:
        Option<wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe>,
    #[serde(default, deserialize_with = "present")]
    physical_function_signatures: Option<wow_emmy::function_calls::PhysicalFunctionSignatureRecipe>,
    #[serde(default, deserialize_with = "present")]
    xml_schema_classification: Option<XmlClassificationInput>,
    #[serde(default)]
    library: Option<DiskInventory>,
}

/// Explicit intent only: neither decoded field reconstructs a native owner.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct XmlClassificationInput {
    schema_selection: XmlSchemaSelection,
    profile_selection: XmlClassificationProfileSelection,
}
impl XmlClassificationInput {
    fn admit(
        self,
        source: &Arc<AdmittedPlatformSource>,
        stop: &AtomicBool,
    ) -> ServiceResult<Arc<ReviewedXmlClassificationProfile>> {
        let catalog = Arc::new(
            admit_xml_schema(source, &self.schema_selection, stop).map_err(acquisition_error)?,
        );
        admit_xml_classification_profile(&catalog, source.as_ref(), &self.profile_selection, stop)
            .map_err(acquisition_error)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeReferenceInput {
    profile: NativeProfile,
    native_source: NativeSourceInput,
}

enum ReferenceInput {
    Artifact(ProjectDiskFile),
    Native(Box<NativeReferenceInput>),
}
impl ReferenceInput {
    fn read(
        self,
        directory: &ProjectInputDirectory,
        source: &ManifestedPlatformInput,
        profile: &BlizzardUiSourceProfile,
        stop: &AtomicBool,
    ) -> ServiceResult<ReferenceView> {
        match self {
            Self::Artifact(file) => serde_json::from_slice(
                &directory
                    .read_json_artifact(&file, stop)
                    .map_err(acquisition_error)?,
            )
            .map_err(|_| invalid("reference artifact was rejected")),
            Self::Native(input) => {
                input.profile.validate_selector()?;
                input.native_source.validate_platform_manifest(source)?;
                let version = format!(
                    "{}.{}",
                    input.profile.client_version, input.profile.client_build
                );
                if input.profile.revision != source.source_revision
                    || version != source.source_version
                {
                    return Err(invalid(
                        "native Reference and platform source revision/version differ",
                    ));
                }
                let (captured, receipt) = input.native_source.read(
                    directory,
                    &input.profile.revision,
                    &version,
                    input.profile.interface,
                    stop,
                )?;
                let built = project_native_reference(
                    &input.profile,
                    &captured,
                    receipt.as_ref(),
                    None,
                    stop,
                )?;
                if built.profile != profile.target().reference_profile
                    || built.reference.view.generation_id()
                        != profile.target().reference_generation.to_string()
                {
                    return Err(invalid(
                        "native Reference differs from the exact platform target",
                    ));
                }
                Ok(built.reference.view)
            }
        }
    }
}

impl LocalProjectInput {
    pub(super) fn from_platform_manifest(
        bytes: &[u8],
        directory: &ProjectInputDirectory,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        super::cancelled(stop)?;
        let input: PlatformInput =
            serde_json::from_slice(bytes).map_err(|_| invalid("invalid platform source input"))?;
        match (
            input.schema.as_str(),
            input.project_budgets,
            input.retained_package_limits,
        ) {
            (LOCAL_PLATFORM_SCHEMA, None, _) => {}
            (LOCAL_PLATFORM_BUDGET_SCHEMA, Some(_), Some(_)) => {}
            _ => return Err(invalid("unsupported platform source budget selection")),
        }
        let reference_input = match (input.reference_view, input.native_reference) {
            (Some(file), None) => ReferenceInput::Artifact(file),
            (None, Some(native)) => ReferenceInput::Native(Box::new(native)),
            _ => {
                return Err(invalid(
                    "platform input requires exactly one Reference choice",
                ));
            }
        };
        let profile =
            BlizzardUiSourceProfile::new(input.source_profile).map_err(acquisition_error)?;
        let source = Arc::new(
            directory
                .admit_platform_source_manifest(&profile, &input.source, stop)
                .map_err(acquisition_error)?
                .into_parts()
                .0,
        );
        // Admit against the exact held source before configuration/generation.
        // Capability/binding equality remains the configuration owner's gate.
        let xml_schema_classification = input
            .xml_schema_classification
            .map(|selection| selection.admit(&source, stop))
            .transpose()?;
        let packages = Arc::new(
            match input.retained_package_limits {
                Some(limits) => source.specialize_packages_with_retention(
                    &input.packages,
                    input.load_context.as_ref(),
                    limits,
                    stop,
                ),
                None => {
                    source.specialize_packages(&input.packages, input.load_context.as_ref(), stop)
                }
            }
            .map_err(acquisition_error)?,
        );
        let reference = reference_input.read(directory, &input.source, &profile, stop)?;
        let analyzer = input.analyzer.read(directory, stop)?;
        let libraries = match input.library {
            Some(library) => directory
                .read_lua_inventory(
                    &library.root,
                    &library.files,
                    ProjectFileRole::Library,
                    stop,
                )
                .map_err(acquisition_error)?,
            None => Vec::new(),
        };
        super::cancelled(stop)?;
        Self::assemble_with_platform(
            ProjectMetadata {
                project_id: input.project_id,
                workspace_id: input.workspace_id,
                source_origin_id: input.source_origin_id,
                logical_root: input.logical_root,
                profile: profile.target().reference_profile.clone(),
                analyzer,
            },
            reference,
            packages.files().to_vec(),
            libraries,
            LocalProjectLoad::PackageUniverse {
                load_plan: packages.load_plan().clone(),
                main_plan: packages.main_plan().clone(),
            },
            Some(PlatformAssembly {
                packages,
                graph_profile: input.graph_profile,
                budgets: input.project_budgets,
                physical_function_declarations: input.physical_function_declarations,
                physical_function_signatures: input.physical_function_signatures,
                xml_schema_classification,
            }),
            stop,
        )
    }
}

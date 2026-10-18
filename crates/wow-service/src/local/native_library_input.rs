//! Requalify native Library producers from retained input data, never reports.
use std::collections::BTreeSet;
use std::fmt;
use std::sync::atomic::AtomicBool;

use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use wow_annotations::artifact::AnnotationArtifact;
use wow_annotations::native::AnnotationFile;
use wow_annotations::navigation::{LookupError, NavigationSourceInventory};
use wow_annotations::reference_links::{NativeCallableBindingError, NativeCallableBindings};
use wow_annotations::sidecars::NativeProjectionSidecars;
use wow_core::{ContentDigest, CorrectionSet, ProfileIdentity, ProfileKind, SourceContent};
use wow_project::disk::{
    DISK_INVENTORY_MAX_BYTES, DISK_INVENTORY_MAX_FILES, DISK_SOURCE_MAX_BYTES,
    ManifestedLuaReplayInput, PinnedLuaSource, SourceManifestReceipt,
};
use wow_project::{
    NATIVE_LIBRARY_INPUT_MAX_BYTES, NativeLibraryInput, ProjectError, ProjectErrorCode,
    ProjectLibraryBinding,
};
use wow_reference::native_view::NativeViewProjection;

use super::cancelled;
use super::input::invalid;
use super::native_input::{NativeFileIdentity, report_bytes_with_limit};
use super::native_reference::{
    InputFailure, NativeProfile, NativeReferenceBuild, project_native_reference,
    project_native_reference_with_type_resolution,
};
use super::native_resources::{NativeAnnotationResources, present};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

const INPUT_SCHEMA: &str = "wow-service/native-library-producer-input/1";
// Selected Reference type overlays change producer identities and interpretation.
// Legacy inputs continue to reconstruct their original Reference projection.
const TYPE_RESOLUTION_INPUT_SCHEMA: &str = "wow-service/native-library-producer-input/2";
const TARGET_INPUT_SCHEMA: &str = "wow-service/native-library-producer-input/3";
// Existing native report/join ceilings; the raw input has its separate Project bound.
const METADATA_MAX_BYTES: usize = 64 * 1024 * 1024;
const BINDING_MAX_WORK: usize = 1_000_000;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum AnnotationInputs {
    #[serde(rename = "default_no_extra_inputs")]
    Default,
    #[serde(rename = "selected_resources")]
    Selected {
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present"
        )]
        correction_json: Option<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present"
        )]
        alias_revision: Option<String>,
        #[serde(deserialize_with = "alias_sources")]
        aliases: Vec<SourceInput>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceInput {
    path: String,
    text: String,
    content_digest: ContentDigest<SourceContent>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducerIds {
    reference_digest: String,
    annotation_artifact_id: String,
    source_map_id: String,
    loss_report_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducerInput {
    schema: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    normalization: Option<String>,
    profile: NativeProfile,
    #[serde(deserialize_with = "sources")]
    sources: Vec<SourceInput>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    source_manifest: Option<ManifestedLuaReplayInput>,
    annotation_inputs: AnnotationInputs,
    expected: ProducerIds,
}

/// Borrow the original producers and all selected raw resources before assembly.
pub(super) struct NativeLibraryCapture<'a> {
    pub(super) profile: &'a NativeProfile,
    pub(super) sources: &'a [PinnedLuaSource],
    pub(super) source_manifest: Option<&'a SourceManifestReceipt>,
    pub(super) annotation_inputs: Option<&'a NativeAnnotationResources>,
    pub(super) reference: &'a NativeViewProjection,
    pub(super) artifact: &'a AnnotationArtifact,
    pub(super) sidecars: &'a NativeProjectionSidecars,
}

pub(super) fn capture(
    original: NativeLibraryCapture<'_>,
    stop: &AtomicBool,
) -> ServiceResult<NativeLibraryInput> {
    cancelled(stop)?;
    original.profile.validate_selector()?;
    original
        .artifact
        .validate()
        .map_err(|_| invalid("native Library artifact is invalid"))?;
    let map = original.sidecars.source_map();
    let loss = original.sidecars.loss_report();
    if original.artifact.producer_id() != "wow-annotations"
        || original.artifact.producer_version() != "1.0.0"
        || original.artifact.profile_id() != original.profile.profile_id.as_str()
        || original.artifact.source_generation_id() != original.reference.view.generation_id()
        || map.annotation_artifact_id() != original.artifact.artifact_id()
        || map.profile_id() != original.artifact.profile_id()
        || map.source_generation_id() != original.artifact.source_generation_id()
        || loss.annotation_artifact_id() != original.artifact.artifact_id()
        || loss.profile_id() != original.artifact.profile_id()
        || loss.source_generation_id() != original.artifact.source_generation_id()
    {
        return Err(identity_error());
    }
    validate_sources(original.sources, stop)?;
    #[derive(Serialize)]
    struct CapturedSource<'a> {
        path: &'a str,
        text: &'a str,
        content_digest: ContentDigest<SourceContent>,
    }
    #[derive(Serialize)]
    struct CapturedInput<'a> {
        schema: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        normalization: Option<&'static str>,
        profile: &'a NativeProfile,
        sources: Vec<CapturedSource<'a>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_manifest: Option<&'a ManifestedLuaReplayInput>,
        annotation_inputs: AnnotationInputs,
        expected: ProducerIds,
    }
    let mut sources = Vec::with_capacity(original.sources.len());
    for file in original.sources {
        cancelled(stop)?;
        sources.push(CapturedSource {
            path: file.path(),
            text: file.text(),
            content_digest: file.content_digest(),
        });
    }
    // Include failed-ingestion files too. The same Reference producer must
    // reconsider each original input rather than reconstructing successful rows.
    let bytes = canonical_bytes(
        &CapturedInput {
            schema: if original.profile.reference_target.is_some() {
                TARGET_INPUT_SCHEMA
            } else if original.reference.type_resolution_profile().is_some() {
                TYPE_RESOLUTION_INPUT_SCHEMA
            } else {
                INPUT_SCHEMA
            },
            normalization: original.profile.reference_target.as_ref().map(|_| {
                if original.reference.type_resolution_profile().is_some() {
                    "type_resolution"
                } else {
                    "original"
                }
            }),
            profile: original.profile,
            sources,
            source_manifest: original
                .source_manifest
                .map(SourceManifestReceipt::retained_input),
            annotation_inputs: capture_annotation_inputs(original.annotation_inputs, stop)?,
            expected: ProducerIds {
                reference_digest: original.reference.view.self_digest().to_owned(),
                annotation_artifact_id: original.artifact.artifact_id().to_owned(),
                source_map_id: map.source_map_id().to_owned(),
                loss_report_id: loss.report_id().to_owned(),
            },
        },
        stop,
    )?;
    let text =
        String::from_utf8(bytes).map_err(|_| invalid("native Library input is not UTF-8"))?;
    NativeLibraryInput::new(text, stop).map_err(project_error)
}

/// Genuine, owned lower-producer results. No borrowed NativeLibrary or decoded
/// report carries semantic authority across the retained-input boundary.
pub struct RequalifiedNativeLibrary {
    pub(super) project_snapshot_id: Box<str>,
    pub(super) profile: ProfileIdentity,
    pub(super) reference: NativeViewProjection,
    pub(super) source_files: Vec<NativeFileIdentity>,
    pub(super) input_failures: Vec<InputFailure>,
    pub(super) artifact: AnnotationArtifact,
    pub(super) sidecars: NativeProjectionSidecars,
    pub(super) callable_bindings: NativeCallableBindings,
    pub(super) annotation_files: Vec<AnnotationFile>,
    pub(super) library_binding: ProjectLibraryBinding,
    pub(super) navigation_sources: NavigationSourceInventory,
}

impl RequalifiedNativeLibrary {
    #[must_use]
    pub fn project_snapshot_id(&self) -> &str {
        &self.project_snapshot_id
    }
    #[must_use]
    pub fn annotation_files(&self) -> &[AnnotationFile] {
        &self.annotation_files
    }

    pub(crate) const fn library_binding(&self) -> &ProjectLibraryBinding {
        &self.library_binding
    }
    #[must_use]
    pub fn navigation_sources(&self) -> &NavigationSourceInventory {
        &self.navigation_sources
    }
    #[must_use]
    pub fn profile(&self) -> &ProfileIdentity {
        &self.profile
    }
    #[must_use]
    pub fn reference(&self) -> &NativeViewProjection {
        &self.reference
    }
    #[must_use]
    pub fn source_files(&self) -> &[NativeFileIdentity] {
        &self.source_files
    }
    #[must_use]
    pub fn input_failure_count(&self) -> usize {
        self.input_failures.len()
    }
    #[must_use]
    pub fn artifact(&self) -> &AnnotationArtifact {
        &self.artifact
    }
    #[must_use]
    pub fn sidecars(&self) -> &NativeProjectionSidecars {
        &self.sidecars
    }
    #[must_use]
    pub fn callable_bindings(&self) -> &NativeCallableBindings {
        &self.callable_bindings
    }
}

/// Requalify a genuinely published or replay-hydrated project. Configured profile,
/// Reference and actual analyzer Library identities fence all rebuilt outputs.
pub(crate) fn rebuild_for_project(
    project: &wow_project::ProjectView,
    stop: &AtomicBool,
) -> ServiceResult<RequalifiedNativeLibrary> {
    cancelled(stop)?;
    let configuration = project.snapshot().configuration();
    configuration.validate().map_err(project_error)?;
    let retained = configuration.native_library_input().ok_or_else(|| {
        ServiceError::new(
            ServiceErrorCode::ComponentUnavailable,
            "native Library input is absent",
        )
    })?;
    let decoded = from_bytes(retained.bytes(), stop)?;
    let type_resolution = selected_type_resolution(&decoded);
    let mut captured = Vec::with_capacity(decoded.sources.len());
    for file in decoded.sources {
        cancelled(stop)?;
        captured.push(
            PinnedLuaSource::from_retained(file.path, file.text, file.content_digest, stop)
                .map_err(project_error)?,
        );
    }
    validate_sources(&captured, stop)?;
    let version = format!(
        "{}.{}",
        decoded.profile.client_version, decoded.profile.client_build
    );
    let manifest = decoded
        .source_manifest
        .as_ref()
        .map(|original| {
            original
                .rebuild(
                    &decoded.profile.revision,
                    &version,
                    decoded.profile.interface,
                    &captured,
                    stop,
                )
                .map_err(project_error)
        })
        .transpose()?;
    let (corrections, aliases) = rebuild_annotation_inputs(decoded.annotation_inputs, stop)?;
    let correction_digest: Option<ContentDigest<CorrectionSet>> = corrections
        .as_ref()
        .map(|set| set.id().parse().map_err(|_| identity_error()))
        .transpose()?;
    let NativeReferenceBuild {
        profile,
        reference,
        documents,
        failures,
        source_files,
    } = if type_resolution {
        project_native_reference_with_type_resolution(
            &decoded.profile,
            &captured,
            manifest.as_ref(),
            corrections
                .as_ref()
                .ok_or_else(|| invalid("retained type overlay requires corrections"))?,
            stop,
        )
    } else {
        project_native_reference(
            &decoded.profile,
            &captured,
            manifest.as_ref(),
            correction_digest,
            stop,
        )
    }?;
    cancelled(stop)?;
    if &profile != configuration.selected_profile()
        || reference.view.generation_id() != configuration.reference_generation().to_string()
        || reference.view.self_digest() != decoded.expected.reference_digest
    {
        return Err(identity_error());
    }
    let borrowed_aliases = aliases.iter().collect::<Vec<_>>();
    let library = wow_annotations::native::project_with_alias_catalogs(
        &documents,
        &decoded.profile.environment,
        corrections.as_ref(),
        &borrowed_aliases,
        stop,
    )
    .map_err(|error| match error {
        wow_annotations::ketho::RenderError::Cancelled => ServiceError::new(
            ServiceErrorCode::Cancelled,
            "native Library reconstruction cancelled",
        ),
        wow_annotations::ketho::RenderError::InputLimit
        | wow_annotations::ketho::RenderError::OutputLimit => budget_error(),
        _ => invalid("native Library reconstruction rejected"),
    })?;
    cancelled(stop)?;
    // Bound the actual reconstructed producer material before canonical artifact
    // allocation, then reserve maps and the join on the same remaining allowance.
    #[derive(Serialize)]
    struct ProducerMaterial<'a, 'b> {
        profile: &'a ProfileIdentity,
        sources: &'a [NativeFileIdentity],
        input_failures: &'a [InputFailure],
        reference: &'a NativeViewProjection,
        library: &'a wow_annotations::native::NativeLibrary<'b>,
    }
    let producer_bytes = report_bytes_with_limit(
        &ProducerMaterial {
            profile: &profile,
            sources: &source_files,
            input_failures: &failures,
            reference: &reference,
            library: &library,
        },
        METADATA_MAX_BYTES,
        stop,
    )?
    .len();
    let artifact = AnnotationArtifact::from_native_library(
        "1.0.0",
        profile.profile_id().as_str(),
        reference.view.generation_id(),
        &library,
    )
    .map_err(|_| invalid("reconstructed annotation artifact rejected"))?;
    cancelled(stop)?;
    let sidecars = NativeProjectionSidecars::from_library_cancellable(&artifact, &library, stop)
        .map_err(|_| invalid("reconstructed annotation sidecars rejected"))?;
    cancelled(stop)?;
    if artifact.artifact_id() != decoded.expected.annotation_artifact_id
        || sidecars.source_map().source_map_id() != decoded.expected.source_map_id
        || sidecars.loss_report().report_id() != decoded.expected.loss_report_id
    {
        return Err(identity_error());
    }
    let navigation_sources =
        NavigationSourceInventory::from_library(&library, stop).map_err(navigation_error)?;
    let remaining = METADATA_MAX_BYTES
        .checked_sub(producer_bytes)
        .and_then(|bytes| bytes.checked_sub(sidecars.metadata_bytes()))
        .and_then(|bytes| bytes.checked_sub(sidecars.source_map_bytes().len()))
        .and_then(|bytes| bytes.checked_sub(sidecars.loss_report_bytes().len()))
        .and_then(|bytes| bytes.checked_sub(navigation_sources.metadata_bytes()))
        .and_then(|bytes| bytes.checked_sub(project.snapshot_id().len()))
        .ok_or_else(budget_error)?;
    let library_binding = verify_library_files(
        project,
        &library.files,
        remaining.min(DISK_INVENTORY_MAX_BYTES),
        stop,
    )?;
    let remaining = remaining
        .checked_sub(library_binding.metadata_bytes())
        .ok_or_else(budget_error)?;
    let callable_bindings =
        NativeCallableBindings::build(&sidecars, &reference, remaining, BINDING_MAX_WORK, stop)
            .map_err(|error| match error {
                NativeCallableBindingError::Cancelled => ServiceError::new(
                    ServiceErrorCode::Cancelled,
                    "native callable reconstruction cancelled",
                ),
                NativeCallableBindingError::Limit => budget_error(),
                NativeCallableBindingError::IncompatibleInput => identity_error(),
            })?;
    cancelled(stop)?;
    Ok(RequalifiedNativeLibrary {
        project_snapshot_id: project.snapshot_id().into(),
        profile,
        reference,
        source_files,
        input_failures: failures,
        artifact,
        sidecars,
        callable_bindings,
        annotation_files: library.files,
        library_binding,
        navigation_sources,
    })
}

pub(super) fn navigation_error(error: LookupError) -> ServiceError {
    match error {
        LookupError::Cancelled => {
            ServiceError::new(ServiceErrorCode::Cancelled, "native navigation cancelled")
        }
        LookupError::InputLimit => budget_error(),
        _ => identity_error(),
    }
}

fn capture_annotation_inputs(
    input: Option<&NativeAnnotationResources>,
    stop: &AtomicBool,
) -> ServiceResult<AnnotationInputs> {
    let Some(input) = input else {
        return Ok(AnnotationInputs::Default);
    };
    cancelled(stop)?;
    let correction_json = input
        .correction_input
        .as_ref()
        .map(|bytes| {
            std::str::from_utf8(bytes)
                .map(str::to_owned)
                .map_err(|_| invalid("correction input is not UTF-8"))
        })
        .transpose()?;
    let mut aliases = Vec::with_capacity(input.alias_inputs.len());
    for file in &input.alias_inputs {
        cancelled(stop)?;
        aliases.push(SourceInput {
            path: file.path().to_owned(),
            text: file.text().to_owned(),
            content_digest: file.content_digest(),
        });
    }
    Ok(AnnotationInputs::Selected {
        correction_json,
        alias_revision: input.selection.alias_revision.clone(),
        aliases,
    })
}

fn rebuild_annotation_inputs(
    input: AnnotationInputs,
    stop: &AtomicBool,
) -> ServiceResult<(
    Option<wow_reference::native_corrections::ValidatedCorrections>,
    Vec<wow_reference::native_aliases::AliasDocument>,
)> {
    use wow_annotations::aliases::{MAX_CATALOG_BYTES, MAX_CATALOG_FILES};
    use wow_reference::native_aliases::{MAX_ALIAS_RESOURCE_BYTES, ingest_alias_catalog};
    use wow_reference::native_corrections::{MAX_CORRECTION_SET_BYTES, ValidatedCorrections};
    cancelled(stop)?;
    let AnnotationInputs::Selected {
        correction_json,
        alias_revision,
        aliases,
    } = input
    else {
        return Ok((None, Vec::new()));
    };
    if (correction_json.is_none() && alias_revision.is_none())
        || alias_revision.is_some() != !aliases.is_empty()
        || aliases.len() > MAX_CATALOG_FILES
    {
        return Err(invalid("incomplete retained annotation resource selection"));
    }
    let corrections = correction_json
        .map(|text| {
            if text.len() > MAX_CORRECTION_SET_BYTES {
                return Err(budget_error());
            }
            let result = ValidatedCorrections::from_json(text.as_bytes());
            cancelled(stop)?;
            result.map_err(|_| invalid("retained correction input rejected"))
        })
        .transpose()?;
    let mut documents = Vec::new();
    let mut total = 0usize;
    let mut previous = None;
    let mut paths = BTreeSet::new();
    if let Some(revision) = alias_revision {
        if !matches!(revision.len(), 40 | 64)
            || !revision
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(invalid("retained alias revision is not exact"));
        }
        for source in aliases {
            cancelled(stop)?;
            total = total
                .checked_add(source.text.len())
                .ok_or_else(budget_error)?;
            if source.text.len() > MAX_ALIAS_RESOURCE_BYTES || total > MAX_CATALOG_BYTES {
                return Err(budget_error());
            }
            if previous
                .as_ref()
                .is_some_and(|path: &String| path >= &source.path)
                || !paths.insert(source.path.to_lowercase())
            {
                return Err(invalid("retained alias inputs are not unique and ordered"));
            }
            previous = Some(source.path.clone());
            let source = PinnedLuaSource::from_retained(
                source.path,
                source.text,
                source.content_digest,
                stop,
            )
            .map_err(project_error)?;
            documents.push(
                ingest_alias_catalog(
                    &revision,
                    source.path(),
                    source.text(),
                    &source.content_digest().to_string(),
                    stop,
                )
                .map_err(super::native_reference::native_error)?,
            );
        }
    }
    cancelled(stop)?;
    Ok((corrections, documents))
}

fn from_bytes(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<ProducerInput> {
    cancelled(stop)?;
    if bytes.is_empty() || bytes.len() > NATIVE_LIBRARY_INPUT_MAX_BYTES {
        return Err(budget_error());
    }
    let input: ProducerInput = serde_json::from_slice(bytes)
        .map_err(|_| invalid("invalid or ambiguous native Library input"))?;
    cancelled(stop)?;
    if !matches!(
        input.schema.as_str(),
        INPUT_SCHEMA | TYPE_RESOLUTION_INPUT_SCHEMA | TARGET_INPUT_SCHEMA
    ) || (input.schema == TARGET_INPUT_SCHEMA) != input.profile.reference_target.is_some()
        || (input.schema == TARGET_INPUT_SCHEMA) != input.normalization.is_some()
        || input
            .normalization
            .as_deref()
            .is_some_and(|mode| !matches!(mode, "original" | "type_resolution"))
    {
        return Err(invalid("unsupported native Library producer-input schema"));
    }
    if selected_type_resolution(&input)
        && !matches!(
            &input.annotation_inputs,
            AnnotationInputs::Selected {
                correction_json: Some(_),
                ..
            }
        )
    {
        return Err(invalid("retained type overlay requires corrections"));
    }
    input.profile.validate_selector()?;
    if canonical_bytes(&input, stop)? != bytes {
        return Err(invalid("native Library input is not exact canonical JSON"));
    }
    cancelled(stop)?;
    Ok(input)
}

fn selected_type_resolution(input: &ProducerInput) -> bool {
    input.schema == TYPE_RESOLUTION_INPUT_SCHEMA
        || (input.schema == TARGET_INPUT_SCHEMA
            && input.normalization.as_deref() == Some("type_resolution"))
}

fn canonical_bytes(value: &impl Serialize, stop: &AtomicBool) -> ServiceResult<Vec<u8>> {
    cancelled(stop)?;
    // Use the existing bounded writer before core canonical map allocation.
    let size = report_bytes_with_limit(value, NATIVE_LIBRARY_INPUT_MAX_BYTES, stop)?.len();
    let bytes = wow_core::canonical_json_bytes(value)
        .map_err(|_| invalid("native Library input cannot be canonicalized"))?;
    cancelled(stop)?;
    if bytes.len() != size || bytes.len() > NATIVE_LIBRARY_INPUT_MAX_BYTES {
        return Err(budget_error());
    }
    Ok(bytes)
}

fn validate_sources(files: &[PinnedLuaSource], stop: &AtomicBool) -> ServiceResult<()> {
    if files.is_empty() || files.len() > DISK_INVENTORY_MAX_FILES {
        return Err(invalid("invalid retained native source count"));
    }
    let mut seen = BTreeSet::new();
    let mut previous = None;
    let mut total = 0usize;
    for file in files {
        cancelled(stop)?;
        if previous.is_some_and(|path: &str| path >= file.path())
            || !seen.insert(file.path().to_lowercase())
        {
            return Err(invalid(
                "retained native sources are not unique and ordered",
            ));
        }
        previous = Some(file.path());
        total = total
            .checked_add(file.text().len())
            .ok_or_else(budget_error)?;
        if file.text().len() > DISK_SOURCE_MAX_BYTES || total > DISK_INVENTORY_MAX_BYTES {
            return Err(budget_error());
        }
    }
    cancelled(stop)
}

fn verify_library_files(
    project: &wow_project::ProjectView,
    generated: &[AnnotationFile],
    max_metadata_bytes: usize,
    stop: &AtomicBool,
) -> ServiceResult<ProjectLibraryBinding> {
    cancelled(stop)?;
    if generated.is_empty() || generated.len() > DISK_INVENTORY_MAX_FILES {
        return Err(identity_error());
    }
    let mut total = 0usize;
    let mut files = Vec::with_capacity(generated.len());
    for file in generated {
        cancelled(stop)?;
        total = total
            .checked_add(file.text.len())
            .ok_or_else(budget_error)?;
        if file.text.len() > DISK_SOURCE_MAX_BYTES || total > DISK_INVENTORY_MAX_BYTES {
            return Err(budget_error());
        }
        files.push(wow_emmy::LuaWorkspaceFileInput::new(
            file.path.clone(),
            file.text.clone(),
        ));
    }
    let configuration = project.snapshot().configuration();
    let universe = match configuration.selected_profile().profile_kind() {
        ProfileKind::Fixture => wow_emmy::LuaWorkspaceUniverse::Fixture,
        ProfileKind::Release => wow_emmy::LuaWorkspaceUniverse::BlizzardUi,
    };
    let rebuilt = wow_emmy::LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        universe,
        files,
        wow_emmy::LuaWorkspaceLimits::new(
            1024,
            4096,
            DISK_SOURCE_MAX_BYTES as u64,
            DISK_INVENTORY_MAX_BYTES as u64,
        )
        .map_err(|_| budget_error())?,
    )
    .map_err(|_| identity_error())?;
    if !project
        .snapshot()
        .analyzer_binding()
        .library_snapshot_ids()
        .eq([rebuilt.snapshot_id()])
    {
        return Err(identity_error());
    }
    cancelled(stop)?;
    ProjectLibraryBinding::qualify(project, &rebuilt, max_metadata_bytes, stop).map_err(|error| {
        match error.code() {
            ProjectErrorCode::SourceReadCancelled | ProjectErrorCode::AnalysisCancelled => {
                ServiceError::new(
                    ServiceErrorCode::Cancelled,
                    "Library qualification cancelled",
                )
            }
            ProjectErrorCode::SourceBudgetExceeded => budget_error(),
            _ => identity_error(),
        }
    })
}

fn sources<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<SourceInput>, D::Error> {
    bounded_sources(deserializer, DISK_INVENTORY_MAX_FILES, true)
}
fn alias_sources<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<SourceInput>, D::Error> {
    bounded_sources(
        deserializer,
        wow_annotations::aliases::MAX_CATALOG_FILES,
        false,
    )
}
fn bounded_sources<'de, D: Deserializer<'de>>(
    deserializer: D,
    limit: usize,
    required: bool,
) -> Result<Vec<SourceInput>, D::Error> {
    struct Sources(usize, bool);
    impl<'de> Visitor<'de> for Sources {
        type Value = Vec<SourceInput>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("one to 1024 retained native Lua inputs")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
            let mut files = Vec::new();
            while files.len() < self.0 {
                let Some(file) = sequence.next_element()? else {
                    if self.1 && files.is_empty() {
                        return Err(serde::de::Error::custom("native input sources are empty"));
                    }
                    return Ok(files);
                };
                files.push(file);
            }
            if sequence.next_element::<IgnoredAny>()?.is_some() {
                return Err(serde::de::Error::custom("native input source-count limit"));
            }
            Ok(files)
        }
    }
    deserializer.deserialize_seq(Sources(limit, required))
}

fn project_error(error: ProjectError) -> ServiceError {
    let code = match error.code() {
        ProjectErrorCode::SourceReadCancelled | ProjectErrorCode::AnalysisCancelled => {
            ServiceErrorCode::Cancelled
        }
        ProjectErrorCode::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::InvalidConfiguration,
    };
    ServiceError::new(
        code,
        "native Library retained input requalification rejected",
    )
}
fn identity_error() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::IdentityMismatch,
        "native Library producer identities differ",
    )
}
fn budget_error() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::BudgetExceeded,
        "native Library reconstruction exceeds budget",
    )
}

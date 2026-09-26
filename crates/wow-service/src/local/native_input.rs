//! One explicit pinned native corpus -> existing reference/annotation owners ->
//! the ordinary project/analyzer path. No interpreter, discovery or source writes.
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{
    CanonicalResult, ContentDigest, CorrectionSet, ProfileId, ProfileIdentity,
    ProfileIdentityBuilder, ProfileKind, ReferenceGenerationId, SchemaVersionEntry, SourceKind,
    SourceLogicalSnapshot, ToolVersion,
};
use wow_project::disk::{ProjectInputDirectory, SourceManifestReceipt};
use wow_project::{
    ProjectFileRole, ProjectId, ProjectInputFile, ProjectLanguageKind, ProjectSourceOriginId,
    ProjectWorkspaceId,
};
use wow_reference::native::{NativeError, NativeErrorCode, ingest_document};

use super::disk_input::{DiskAnalyzer, MainInventory};
use super::input::{ProjectMetadata, invalid};
use super::native_resources::{NativeAnnotationInputs, NativeAnnotationInputsReceipt, present};
use super::native_source::NativeSourceInput;
use super::{LocalProjectInput, cancelled};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

pub const LOCAL_NATIVE_SCHEMA: &str = "wow-service/local-project-native/1";
const NATIVE_REPORT_LIMIT: usize = 64 * 1024 * 1024;
const PROFILE_VERSION: &str = "1.0.0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeInput {
    schema: String,
    project_id: ProjectId,
    workspace_id: ProjectWorkspaceId,
    source_origin_id: ProjectSourceOriginId,
    logical_root: String,
    profile: NativeProfile,
    analyzer: DiskAnalyzer,
    main: MainInventory,
    native_source: NativeSourceInput,
    #[serde(default, deserialize_with = "present")]
    annotation_inputs: Option<NativeAnnotationInputs>,
}

/// Labels describe an explicitly selected source corpus, not a currentness or
/// release-acceptance claim. Identities are derived from the actual pinned bytes.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeProfile {
    profile_id: ProfileId,
    flavor: String,
    #[serde(default)]
    edition: Option<String>,
    interface: u64,
    client_version: ToolVersion,
    client_build: u64,
    revision: String,
    environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeFileIdentity {
    pub path: String,
    pub sha256: String,
    pub byte_length: u64,
}

/// Compact public identity; the complete TOC selection record stays in the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeManifestIdentity {
    pub manifest: NativeFileIdentity,
    pub manifest_sha256: String,
    pub toc: NativeFileIdentity,
    pub version_file: NativeFileIdentity,
    pub declared_included_files: u64,
    pub declared_generated_api_files: u64,
    pub selected_generated_api_files: usize,
    pub generated_api_closure: &'static str,
    pub generated_api_closure_complete: bool,
    pub verified_source_files: usize,
    pub selected_document_files: usize,
    pub git_membership: &'static str,
    pub unconsumed_source_bytes: &'static str,
    pub negative_authority: bool,
}

impl From<&SourceManifestReceipt> for NativeManifestIdentity {
    fn from(receipt: &SourceManifestReceipt) -> Self {
        let file = |value: &wow_project::load::LoadSource| NativeFileIdentity {
            path: value.path.clone(),
            sha256: value.content_digest.to_string(),
            byte_length: value.byte_length,
        };
        Self {
            manifest: file(&receipt.manifest),
            manifest_sha256: receipt.manifest_sha256.clone(),
            toc: file(receipt.selected_toc()),
            version_file: file(&receipt.version_file),
            declared_included_files: receipt.declared_included_files,
            declared_generated_api_files: receipt.declared_generated_api_files,
            selected_generated_api_files: receipt.selected_generated_api_files,
            generated_api_closure: receipt.generated_api_closure,
            generated_api_closure_complete: receipt.generated_api_closure_complete,
            verified_source_files: receipt.verified_source_files,
            selected_document_files: receipt.selected_file_count(),
            git_membership: receipt.git_membership,
            unconsumed_source_bytes: receipt.unconsumed_source_bytes,
            negative_authority: receipt.negative_authority,
        }
    }
}

/// Small public receipt. The complete raw/projection/source-map sidecar is kept
/// separately, identified by its exact digest, not copied into ordinary findings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeInputReceipt {
    pub schema: &'static str,
    pub profile: ProfileIdentity,
    pub revision: String,
    pub environment: String,
    pub source_binding: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_manifest: Option<NativeManifestIdentity>,
    pub freshness: &'static str,
    pub candidate_files: usize,
    pub admitted_files: usize,
    pub input_failures: usize,
    pub reference_issues: usize,
    pub reference_conflicts: usize,
    pub annotation_issues: usize,
    pub metadata_sidecars: usize,
    pub analyzer_input_configuration: ContentDigest<CanonicalResult>,
    pub analyzer_bound_configuration: ContentDigest<CanonicalResult>,
    pub annotation_projection: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_inputs: Option<NativeAnnotationInputsReceipt>,
    pub library_files: Vec<NativeFileIdentity>,
    pub report_sha256: String,
    pub report_bytes: usize,
    pub negative_authority: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_authority_scope: Option<&'static str>,
    pub semantic_consumer_acceptance: &'static str,
}

pub(super) struct NativeInputEvidence {
    pub(super) receipt: NativeInputReceipt,
    report: Box<[u8]>,
}

#[derive(Serialize)]
struct InputFailure {
    path: String,
    sha256: String,
    error: NativeError,
}

impl LocalProjectInput {
    /// Original native raw metadata, source maps and loss/conflict records.
    /// This is data, never agent instructions or a semantic-acceptance report.
    /// Imported artifacts retain the original producer bytes; their historical
    /// analyzer fields do not describe the current artifact admission receipt.
    #[must_use]
    pub fn native_source_report(&self) -> Option<&[u8]> {
        self.native_input
            .as_ref()
            .map(|evidence| evidence.report.as_ref())
            .or_else(|| {
                self.native_artifact
                    .as_ref()
                    .map(|evidence| evidence.report.as_ref())
            })
    }

    #[must_use]
    pub fn native_input_receipt(&self) -> Option<&NativeInputReceipt> {
        self.native_input.as_ref().map(|evidence| &evidence.receipt)
    }

    pub(super) fn from_native_manifest(
        bytes: &[u8],
        directory: &ProjectInputDirectory,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        cancelled(stop)?;
        let input: NativeInput = serde_json::from_slice(bytes)
            .map_err(|_| invalid("invalid native project manifest"))?;
        if input.schema != LOCAL_NATIVE_SCHEMA {
            return Err(invalid("unsupported native project schema"));
        }
        input.profile.validate_selector()?;
        // Paths, counts and all expected byte identities are checked by the
        // existing confined reader before any native owner sees these sources.
        let expected_version = format!(
            "{}.{}",
            input.profile.client_version, input.profile.client_build
        );
        let (captured, source_manifest) = input.native_source.read(
            directory,
            &input.profile.revision,
            &expected_version,
            input.profile.interface,
            stop,
        )?;
        let manifested = source_manifest.is_some();
        let source_binding = if manifested {
            "source_manifest_selected_toc"
        } else {
            "explicit_digest_pinned_manifest"
        };
        let source_files = captured
            .iter()
            .map(|file| NativeFileIdentity {
                path: file.path().to_owned(),
                sha256: file.content_digest().to_string(),
                byte_length: file.text().len() as u64,
            })
            .collect::<Vec<_>>();
        // Preserve the existing explicit-file identity profile. Manifested input
        // also binds exact TOC/version/manifest bytes, source order and load context.
        let selection = if let Some(receipt) = &source_manifest {
            wow_core::canonical_json_bytes(&(
                "wow-service/native-source-selection/2",
                &input.profile.revision,
                &input.profile.environment,
                &source_files,
                receipt,
            ))
        } else {
            wow_core::canonical_json_bytes(&(
                "wow-service/native-source-selection/1",
                &input.profile.revision,
                &input.profile.environment,
                &source_files,
            ))
        }
        .map_err(|_| invalid("native source selection cannot be canonicalized"))?;
        let annotation_inputs = input
            .annotation_inputs
            .as_ref()
            .map(|inputs| inputs.read(directory, stop))
            .transpose()?;
        let correction_digest = annotation_inputs
            .as_ref()
            .map(|inputs| inputs.correction_digest())
            .transpose()?
            .flatten();
        let profile = input.profile.build(
            ContentDigest::from_bytes(Sha256::digest(&selection).into()),
            correction_digest,
        )?;
        let callable_corpus = source_manifest.as_ref().map_or_else(
            wow_reference::native_view::NativeCallableCorpus::explicit_partial,
            |receipt| {
                wow_reference::native_view::NativeCallableCorpus::manifest_toc(
                    receipt.declared_generated_api_files,
                    receipt.selected_generated_api_files,
                    receipt.generated_api_closure_complete,
                )
            },
        );
        let generation = ReferenceGenerationId::derive(&(callable_corpus.profile(), &profile))
            .map_err(|_| invalid("native reference generation cannot be derived"))?;
        let mut documents = Vec::new();
        let mut failures = Vec::new();
        for file in &captured {
            cancelled(stop)?;
            match ingest_document(
                &input.profile.revision,
                file.path(),
                file.text(),
                &file.content_digest().to_string(),
                stop,
            ) {
                Ok(document) => documents.push(document),
                Err(error)
                    if matches!(
                        error.code,
                        NativeErrorCode::Cancelled | NativeErrorCode::Limit
                    ) =>
                {
                    return Err(native_error(error));
                }
                Err(error) => failures.push(InputFailure {
                    path: file.path().to_owned(),
                    sha256: file.content_digest().to_string(),
                    error,
                }),
            }
        }
        if documents.is_empty() {
            return Err(invalid("no native documentation was admitted"));
        }
        let reference = wow_reference::native_view::project_callables(
            &documents,
            &input.profile.environment,
            generation,
            callable_corpus,
            stop,
        )
        .map_err(native_error)?;
        let aliases = annotation_inputs
            .as_ref()
            .map(|inputs| inputs.aliases.iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let corrections = annotation_inputs
            .as_ref()
            .and_then(|inputs| inputs.corrections.as_ref());
        let library = wow_annotations::native::project_with_alias_catalogs(
            &documents,
            &input.profile.environment,
            corrections,
            &aliases,
            stop,
        )
        .map_err(|error| match error {
            wow_annotations::ketho::RenderError::Cancelled => ServiceError::new(
                ServiceErrorCode::Cancelled,
                "native annotation projection cancelled",
            ),
            wow_annotations::ketho::RenderError::InputLimit
            | wow_annotations::ketho::RenderError::OutputLimit => ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "native annotation projection exceeds budget",
            ),
            _ => invalid("native annotation projection rejected"),
        })?;
        cancelled(stop)?;
        if library.files.is_empty() {
            return Err(invalid(
                "native projection produced no usable Library files",
            ));
        }
        if library.files.len() > 1024 {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "native Library file count exceeds budget",
            ));
        }
        let mut library_bytes = 0usize;
        for file in &library.files {
            cancelled(stop)?;
            library_bytes = library_bytes.checked_add(file.text.len()).ok_or_else(|| {
                ServiceError::new(
                    ServiceErrorCode::BudgetExceeded,
                    "native Library byte count overflow",
                )
            })?;
            if file.text.len() > 1024 * 1024 || library_bytes > 16 * 1024 * 1024 {
                return Err(ServiceError::new(
                    ServiceErrorCode::BudgetExceeded,
                    "native Library bytes exceed budget",
                ));
            }
        }
        let library_files = library
            .files
            .iter()
            .map(|file| NativeFileIdentity {
                path: file.path.clone(),
                sha256: file.sha256.clone(),
                byte_length: file.text.len() as u64,
            })
            .collect::<Vec<_>>();
        let mut analyzer = input.analyzer.read(directory, stop)?;
        let analyzer_input_configuration = analyzer.configuration_digest;
        // Project generation derivation includes the analyzer configuration, not
        // the Library bytes directly. Bind the actual generated output here so
        // a projection change cannot silently reuse an old analyzer generation.
        let bound_configuration = if let Some(resources) = &annotation_inputs {
            wow_core::canonical_json_bytes(&(
                "wow-service/native-analyzer-binding/2",
                analyzer_input_configuration,
                &profile,
                reference.view.self_digest(),
                library.schema,
                library.source_map_profile,
                &library_files,
                &resources.selection,
            ))
        } else {
            wow_core::canonical_json_bytes(&(
                "wow-service/native-analyzer-binding/1",
                analyzer_input_configuration,
                &profile,
                reference.view.self_digest(),
                library.schema,
                library.source_map_profile,
                &library_files,
            ))
        }
        .map_err(|_| invalid("native analyzer binding cannot be canonicalized"))?;
        analyzer.configuration_digest =
            ContentDigest::from_bytes(Sha256::digest(&bound_configuration).into());
        let analyzer_bound_configuration = analyzer.configuration_digest;
        let annotation_receipt = annotation_inputs
            .as_ref()
            .map(|inputs| inputs.receipt(&library));
        let reference_issues = reference.issues.len();
        let reference_conflicts = reference.view.conflicts().len();
        // Streaming serialization enforces the report bound before a large JSON
        // Value or an unbounded output buffer could be allocated.
        #[derive(Serialize)]
        struct Report<'a, 'b> {
            schema: &'static str,
            profile: &'a ProfileIdentity,
            analyzer_input_configuration: ContentDigest<CanonicalResult>,
            analyzer_bound_configuration: ContentDigest<CanonicalResult>,
            environment: &'a str,
            source_binding: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            source_manifest: Option<&'a SourceManifestReceipt>,
            freshness: &'static str,
            source_files: &'a [NativeFileIdentity],
            input_failures: &'a [InputFailure],
            reference: &'a wow_reference::native_view::NativeViewProjection,
            library: &'a wow_annotations::native::NativeLibrary<'b>,
            #[serde(skip_serializing_if = "Option::is_none")]
            annotation_inputs: Option<&'a NativeAnnotationInputsReceipt>,
            negative_authority: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            negative_authority_scope: Option<&'static str>,
        }
        let report = report_bytes(
            &Report {
                schema: if manifested {
                    "wow-service/native-input-report/4"
                } else if annotation_inputs.is_some() {
                    "wow-service/native-input-report/3"
                } else {
                    "wow-service/native-input-report/1"
                },
                profile: &profile,
                analyzer_input_configuration,
                analyzer_bound_configuration,
                environment: &input.profile.environment,
                source_binding,
                source_manifest: source_manifest.as_ref(),
                freshness: "unverified-current",
                source_files: &source_files,
                input_failures: &failures,
                reference: &reference,
                library: &library,
                annotation_inputs: annotation_receipt.as_ref(),
                negative_authority: reference.negative_authority,
                negative_authority_scope: reference
                    .negative_authority
                    .then_some(wow_reference::native_view::NATIVE_API_PARTITION),
            },
            stop,
        )?;
        let receipt = NativeInputReceipt {
            schema: if manifested {
                "wow-service/native-input-receipt/4"
            } else if annotation_inputs.is_some() {
                "wow-service/native-input-receipt/3"
            } else {
                "wow-service/native-input-receipt/1"
            },
            profile: profile.clone(),
            revision: input.profile.revision.clone(),
            environment: input.profile.environment.clone(),
            source_binding,
            source_manifest: source_manifest.as_ref().map(NativeManifestIdentity::from),
            freshness: "unverified-current",
            candidate_files: source_files.len(),
            admitted_files: documents.len(),
            input_failures: failures.len(),
            analyzer_input_configuration,
            analyzer_bound_configuration,
            reference_issues,
            reference_conflicts,
            annotation_issues: library.issues.len(),
            metadata_sidecars: library.metadata_sidecars.len(),
            annotation_projection: library.projection,
            annotation_inputs: annotation_receipt,
            library_files,
            report_sha256: wow_reference::native::source_digest(&report),
            report_bytes: report.len(),
            negative_authority: reference.negative_authority,
            negative_authority_scope: reference
                .negative_authority
                .then_some(wow_reference::native_view::NATIVE_API_PARTITION),
            semantic_consumer_acceptance: "not_evaluated",
        };
        let libraries = library
            .files
            .into_iter()
            .map(|file| {
                let digest = file
                    .sha256
                    .parse()
                    .map_err(|_| invalid("native annotation digest rejected"))?;
                let length = file.text.len() as u64;
                ProjectInputFile::declared_with_identity(
                    file.path,
                    file.text.into_bytes(),
                    ProjectLanguageKind::Lua,
                    ProjectFileRole::Library,
                    digest,
                    length,
                    None::<String>,
                )
                .map_err(|_| invalid("native annotation file identity rejected"))
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        let (main, load_plan) = input.main.read(directory, &profile, stop)?;
        let mut assembled = Self::assemble_with_load_plan(
            ProjectMetadata {
                project_id: input.project_id,
                workspace_id: input.workspace_id,
                source_origin_id: input.source_origin_id,
                logical_root: input.logical_root,
                profile,
                analyzer,
            },
            reference.view,
            main,
            libraries,
            load_plan,
        )?;
        assembled.native_input = Some(Arc::new(NativeInputEvidence {
            receipt,
            report: report.into_boxed_slice(),
        }));
        cancelled(stop)?;
        Ok(assembled)
    }
}

impl NativeProfile {
    fn validate_selector(&self) -> ServiceResult<()> {
        if self.profile_id.as_str() == wow_rules::FIXTURE_PROFILE_ID {
            return Err(invalid(
                "native production input cannot use the fixture rule profile",
            ));
        }
        if !matches!(self.revision.len(), 40 | 64)
            || !self
                .revision
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.environment.is_empty()
            || self.environment.len() > 128
            || !self
                .environment
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(invalid(
                "native source requires an exact revision and explicit environment",
            ));
        }
        Ok(())
    }

    fn build(
        &self,
        source_digest: ContentDigest<SourceLogicalSnapshot>,
        correction_digest: Option<ContentDigest<CorrectionSet>>,
    ) -> ServiceResult<ProfileIdentity> {
        // Bind the canonical explicitly selected correction set; omission retains
        // the original no-corrections identity. Raw source observations stay intact.
        let corrections =
            wow_core::canonical_json_bytes(&("wow-service/native-corrections/1", "none"))
                .map_err(|_| invalid("native correction selection rejected"))?;
        let mut builder = ProfileIdentityBuilder::new(
            self.profile_id.clone(),
            ProfileKind::Release,
            self.flavor.clone(),
            self.interface,
            SourceKind::BlizzardSnapshot,
            self.revision.clone(),
            source_digest,
        )
        .client_version(self.client_version.clone())
        .client_build(self.client_build)
        .builder(
            "wow.service.native"
                .parse()
                .map_err(|_| invalid("native builder identity rejected"))?,
            PROFILE_VERSION
                .parse()
                .map_err(|_| invalid("native builder version rejected"))?,
        )
        .schema_versions(vec![SchemaVersionEntry::new(
            "schema:wow:native-source-input"
                .parse()
                .map_err(|_| invalid("native schema identity rejected"))?,
            PROFILE_VERSION
                .parse()
                .map_err(|_| invalid("native schema version rejected"))?,
        )])
        .correction_set_digest(correction_digest.unwrap_or_else(|| {
            ContentDigest::<CorrectionSet>::from_bytes(Sha256::digest(&corrections).into())
        }));
        if let Some(edition) = &self.edition {
            builder = builder.edition_id(edition.clone());
        }
        builder
            .build()
            .map_err(|_| invalid("native release profile rejected"))
    }
}

fn native_error(error: NativeError) -> ServiceError {
    match error.code {
        NativeErrorCode::Cancelled => ServiceError::new(
            ServiceErrorCode::Cancelled,
            "native source operation cancelled",
        ),
        NativeErrorCode::Limit => ServiceError::new(
            ServiceErrorCode::BudgetExceeded,
            "native source operation exceeds budget",
        ),
        _ => invalid("native reference projection rejected"),
    }
}

fn report_bytes(value: &impl Serialize, stop: &AtomicBool) -> ServiceResult<Vec<u8>> {
    report_bytes_with_limit(value, NATIVE_REPORT_LIMIT, stop)
}

pub(super) fn report_bytes_with_limit(
    value: &impl Serialize,
    limit: usize,
    stop: &AtomicBool,
) -> ServiceResult<Vec<u8>> {
    if limit == 0 || limit > NATIVE_REPORT_LIMIT {
        return Err(ServiceError::new(
            ServiceErrorCode::BudgetExceeded,
            "native output limit is invalid",
        ));
    }
    struct Bounded<'a>(Vec<u8>, &'a AtomicBool, usize);
    impl Write for Bounded<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.1.load(std::sync::atomic::Ordering::Acquire) {
                return Err(std::io::Error::other("native report cancelled"));
            }
            if self.0.len().saturating_add(bytes.len()) > self.2 {
                return Err(std::io::Error::other("native report byte budget"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = Bounded(Vec::new(), stop, limit);
    let encoded = serde_json::to_writer(&mut buffer, value);
    cancelled(stop)?;
    encoded.map_err(|_| {
        ServiceError::new(
            ServiceErrorCode::BudgetExceeded,
            "native report encoding failed or exceeds budget",
        )
    })?;
    Ok(buffer.0)
}

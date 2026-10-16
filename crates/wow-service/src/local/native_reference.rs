//! Shared exact native source/profile/callable projection, independent of Main.
use super::{
    cancelled, input::invalid, native_input::NativeFileIdentity, native_source::NativeSourceInput,
};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_core::{
    ContentDigest, CorrectionSet, ProfileId, ProfileIdentity, ProfileIdentityBuilder, ProfileKind,
    ReferenceGenerationId, SchemaVersionEntry, SourceContent, SourceKind, SourceLogicalSnapshot,
    ToolVersion,
};
use wow_project::disk::{PinnedLuaSource, ProjectInputDirectory, SourceManifestReceipt};
use wow_reference::{
    native::{DocumentationDocument, NativeError, NativeErrorCode, ingest_document},
    native_corrections::ValidatedCorrections,
    native_view::NativeViewProjection,
};

const PROFILE_VERSION: &str = "1.0.0";
pub const SOURCE_REFERENCE_INPUT_SCHEMA: &str = "wow-service/source-reference-input/1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceReferenceInput {
    schema: String,
    profile: NativeProfile,
    native_source: NativeSourceInput,
    #[serde(default, deserialize_with = "super::native_resources::present")]
    native_documents: Option<wow_reference::native_view::NativeReferenceDocuments>,
    #[serde(default, deserialize_with = "super::native_resources::present")]
    compatibility_aliases: Option<wow_project::disk::ProjectDiskFile>,
}

/// Exact selected callable projection; no project, annotation or runtime claim.
#[derive(Serialize)]
pub struct SourceReferenceResult {
    schema: &'static str,
    configuration_content_digest: ContentDigest<SourceContent>,
    profile: ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    revision: String,
    environment: String,
    source_binding: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_manifest: Option<SourceManifestReceipt>,
    freshness: &'static str,
    source_files: Vec<NativeFileIdentity>,
    input_failures: Vec<InputFailure>,
    reference: NativeViewProjection,
    #[serde(skip_serializing_if = "Option::is_none")]
    compatibility_aliases: Option<wow_reference::compatibility_aliases::AliasReceipt>,
}

impl SourceReferenceResult {
    pub fn has_compatibility_aliases(&self) -> bool {
        self.compatibility_aliases.is_some()
    }
    pub fn native_documents(&self) -> Option<wow_reference::native_view::NativeReferenceDocuments> {
        self.reference.native_documents()
    }
    pub(crate) fn source_revision(&self) -> &str {
        &self.revision
    }
    pub(crate) fn configuration_digest(&self) -> &ContentDigest<SourceContent> {
        &self.configuration_content_digest
    }
    pub(crate) fn acquisition_failures(&self) -> impl Iterator<Item = (&str, &str, &NativeError)> {
        self.input_failures.iter().map(|failure| {
            (
                failure.path.as_str(),
                failure.sha256.as_str(),
                &failure.error,
            )
        })
    }
    pub fn profile(&self) -> &ProfileIdentity {
        &self.profile
    }
    pub fn reference_generation(&self) -> ReferenceGenerationId {
        self.reference_generation
    }
    pub fn reference(&self) -> &NativeViewProjection {
        &self.reference
    }
    pub fn is_partial(&self) -> bool {
        !self.input_failures.is_empty()
            || !self.reference.negative_authority
            || self.compatibility_aliases.is_some()
    }
    pub fn canonical_bytes(&self, stop: &AtomicBool) -> ServiceResult<Vec<u8>> {
        super::native_input::report_bytes_with_limit(self, 64 * 1024 * 1024, stop)
    }
}

/// Read one explicit bounded config and pinned corpus, with no source effects.
pub fn read_local_source_reference(
    config: &Path,
    stop: &AtomicBool,
) -> ServiceResult<SourceReferenceResult> {
    cancelled(stop)?;
    let parent = config
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = config
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.ends_with(".json"))
        .ok_or_else(|| invalid("source reference requires an explicit JSON configuration"))?;
    let directory =
        ProjectInputDirectory::open(parent).map_err(super::disk_input::acquisition_error)?;
    let bytes = directory
        .read_configuration(name, stop)
        .map_err(super::disk_input::acquisition_error)?;
    let input: SourceReferenceInput = serde_json::from_slice(&bytes)
        .map_err(|_| invalid("invalid source reference configuration"))?;
    if input.schema != SOURCE_REFERENCE_INPUT_SCHEMA {
        return Err(invalid("unsupported source reference schema"));
    }
    input.profile.validate_selector()?;
    let version = format!(
        "{}.{}",
        input.profile.client_version, input.profile.client_build
    );
    let (captured, source_manifest) = input.native_source.read(
        &directory,
        &input.profile.revision,
        &version,
        input.profile.interface,
        stop,
    )?;
    let mut built = build_native_reference(
        &input.profile,
        &captured,
        source_manifest.as_ref(),
        None,
        None,
        input.native_documents,
        stop,
    )?;
    let compatibility_aliases = if let Some(selected) = &input.compatibility_aliases {
        let bytes = directory
            .read_json_artifact_with_limit(
                selected,
                wow_reference::compatibility_aliases::MAX_INPUT_BYTES,
                stop,
            )
            .map_err(super::disk_input::acquisition_error)?;
        built.reference = built
            .reference
            .with_compatibility_aliases(&built.profile, selected.path(), &bytes, stop)
            .map_err(native_error)?;
        Some(
            wow_reference::compatibility_aliases::read(&built.reference.view, &built.profile, stop)
                .map_err(native_error)?
                .ok_or_else(|| invalid("configured alias owner is unavailable"))?
                .receipt()
                .clone(),
        )
    } else {
        None
    };
    let reference_generation = built
        .reference
        .view
        .generation_id()
        .parse()
        .map_err(|_| invalid("native reference owner returned an invalid generation"))?;
    cancelled(stop)?;
    Ok(SourceReferenceResult {
        schema: if compatibility_aliases.is_some() {
            "wow-service/source-reference-compatibility-result/1"
        } else {
            "wow-service/source-reference-result/1"
        },
        configuration_content_digest: ContentDigest::from_bytes(Sha256::digest(bytes).into()),
        profile: built.profile,
        reference_generation,
        revision: input.profile.revision,
        environment: input.profile.environment,
        source_binding: source_binding(source_manifest.as_ref()),
        source_manifest,
        freshness: "unverified-current",
        source_files: built.source_files,
        input_failures: built.failures,
        reference: built.reference,
        compatibility_aliases,
    })
}

pub(super) struct NativeReferenceBuild {
    pub(super) profile: ProfileIdentity,
    pub(super) source_files: Vec<NativeFileIdentity>,
    pub(super) documents: Vec<DocumentationDocument>,
    pub(super) failures: Vec<InputFailure>,
    pub(super) reference: NativeViewProjection,
}

pub(super) fn source_binding(manifest: Option<&SourceManifestReceipt>) -> &'static str {
    if manifest.is_some() {
        "source_manifest_selected_toc"
    } else {
        "explicit_digest_pinned_manifest"
    }
}

pub(super) fn project_native_reference(
    requested: &NativeProfile,
    captured: &[PinnedLuaSource],
    source_manifest: Option<&SourceManifestReceipt>,
    correction_digest: Option<ContentDigest<CorrectionSet>>,
    stop: &AtomicBool,
) -> ServiceResult<NativeReferenceBuild> {
    build_native_reference(
        requested,
        captured,
        source_manifest,
        correction_digest,
        None,
        None,
        stop,
    )
}

/// Explicitly selected corrections also select the Reference-owned type overlay.
/// The legacy entry above remains the retained v1 producer's exact reconstruction.
pub(super) fn project_native_reference_with_type_resolution(
    requested: &NativeProfile,
    captured: &[PinnedLuaSource],
    source_manifest: Option<&SourceManifestReceipt>,
    corrections: &ValidatedCorrections,
    stop: &AtomicBool,
) -> ServiceResult<NativeReferenceBuild> {
    let digest = corrections
        .id()
        .parse()
        .map_err(|_| invalid("native correction identity rejected"))?;
    build_native_reference(
        requested,
        captured,
        source_manifest,
        Some(digest),
        Some(corrections),
        None,
        stop,
    )
}

fn build_native_reference(
    requested: &NativeProfile,
    captured: &[PinnedLuaSource],
    source_manifest: Option<&SourceManifestReceipt>,
    correction_digest: Option<ContentDigest<CorrectionSet>>,
    corrections: Option<&ValidatedCorrections>,
    native_documents: Option<wow_reference::native_view::NativeReferenceDocuments>,
    stop: &AtomicBool,
) -> ServiceResult<NativeReferenceBuild> {
    cancelled(stop)?;
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
            &requested.revision,
            &requested.environment,
            &source_files,
            receipt,
        ))
    } else {
        wow_core::canonical_json_bytes(&(
            "wow-service/native-source-selection/1",
            &requested.revision,
            &requested.environment,
            &source_files,
        ))
    }
    .map_err(|_| invalid("native source selection cannot be canonicalized"))?;
    let profile = requested.build(
        ContentDigest::from_bytes(Sha256::digest(&selection).into()),
        correction_digest,
    )?;
    let callable_corpus = source_manifest.map_or_else(
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
    for file in captured {
        cancelled(stop)?;
        match ingest_document(
            &requested.revision,
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
    let reference = if let Some(documents_kind) = native_documents {
        wow_reference::native_view::project_native_documents(
            &documents,
            &requested.environment,
            generation,
            callable_corpus,
            corrections,
            documents_kind,
            stop,
        )
    } else if let Some(corrections) = corrections {
        wow_reference::native_view::project_callables_with_type_resolution(
            &documents,
            &requested.environment,
            generation,
            callable_corpus,
            Some(corrections),
            stop,
        )
    } else {
        wow_reference::native_view::project_callables(
            &documents,
            &requested.environment,
            generation,
            callable_corpus,
            stop,
        )
    }
    .map_err(native_error)?;
    cancelled(stop)?;
    Ok(NativeReferenceBuild {
        profile,
        source_files,
        documents,
        failures,
        reference,
    })
}
#[derive(Serialize)]
pub(super) struct InputFailure {
    path: String,
    sha256: String,
    error: NativeError,
}

/// Labels describe an explicitly selected source corpus, not a currentness or
/// release-acceptance claim. Identities are derived from the actual pinned bytes.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeProfile {
    pub(super) profile_id: ProfileId,
    pub(super) flavor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) edition: Option<String>,
    pub(super) interface: u64,
    pub(super) client_version: ToolVersion,
    pub(super) client_build: u64,
    pub(super) revision: String,
    pub(super) environment: String,
}

impl NativeProfile {
    pub(super) fn validate_selector(&self) -> ServiceResult<()> {
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

pub(super) fn native_error(error: NativeError) -> ServiceError {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct InputDirectory(PathBuf);
    impl Drop for InputDirectory {
        fn drop(&mut self) {
            for file in ["source/API.lua", "source/Bad.lua", "config.json"] {
                let _ = fs::remove_file(self.0.join(file));
            }
            let _ = fs::remove_dir(self.0.join("source"));
            let _ = fs::remove_dir(&self.0);
        }
    }

    #[test]
    fn source_only_identity_retains_partial_failures_and_refuses_tampering()
    -> Result<(), Box<dyn std::error::Error>> {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = std::env::temp_dir().join(format!(
            "wow-source-reference-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir(&root)?;
        let root = InputDirectory(root);
        fs::create_dir(root.0.join("source"))?;
        let good = r#"APIDocumentation:AddDocumentationTable({Name="Example",Type="System",Namespace="C_Example",Environment="All",Functions={{Name="Read",Type="Function"}}})"#;
        let bad = "return unknown()";
        fs::write(root.0.join("source/API.lua"), good)?;
        fs::write(root.0.join("source/Bad.lua"), bad)?;
        let mut configuration = serde_json::json!({
            "schema": SOURCE_REFERENCE_INPUT_SCHEMA,
            "profile": {
                "profile_id": "profile:wow:source-reference-test", "flavor": "retail",
                "interface": 1, "client_version": "1.0.0", "client_build": 1,
                "revision": "1111111111111111111111111111111111111111", "environment": "Mainline"
            },
            "native_source": {"root": "source", "files": [
                {"path": "API.lua", "content_digest": wow_reference::native::source_digest(good.as_bytes()), "byte_length": good.len()},
                {"path": "Bad.lua", "content_digest": wow_reference::native::source_digest(bad.as_bytes()), "byte_length": bad.len()}
            ]}
        });
        let config = root.0.join("config.json");
        fs::write(&config, serde_json::to_vec(&configuration)?)?;
        let stop = AtomicBool::new(false);
        let first = read_local_source_reference(&config, &stop)?;
        assert!(first.is_partial());
        assert_eq!(first.profile().profile_kind(), ProfileKind::Release);
        assert_eq!(
            first.reference_generation().to_string(),
            first.reference().view.generation_id()
        );
        assert_eq!(first.source_files.len(), 2);
        assert_eq!(first.input_failures.len(), 1);
        assert_eq!(first.input_failures[0].path, "Bad.lua");
        assert!(matches!(
            first.reference().view.lookup(
                wow_reference::native_view::NATIVE_API_PARTITION,
                "function:C_Example.Read"
            ),
            wow_reference::LookupResult::Found(_)
        ));
        assert!(!first.reference().negative_authority);
        // Source-only transport must not depend on configuration formatting.
        fs::write(&config, serde_json::to_vec_pretty(&configuration)?)?;
        let second = read_local_source_reference(&config, &stop)?;
        assert_eq!(first.profile(), second.profile());
        assert_eq!(first.reference_generation(), second.reference_generation());
        assert_eq!(first.reference(), second.reference());
        assert_ne!(
            first.configuration_content_digest,
            second.configuration_content_digest
        );
        first.canonical_bytes(&stop)?;
        configuration["main"] = serde_json::json!({"root": "unused"});
        fs::write(&config, serde_json::to_vec(&configuration)?)?;
        assert!(
            matches!(read_local_source_reference(&config, &stop), Err(error) if error.code() == ServiceErrorCode::InvalidConfiguration)
        );
        configuration
            .as_object_mut()
            .ok_or("configuration object")?
            .remove("main");
        fs::write(&config, serde_json::to_vec(&configuration)?)?;
        fs::write(root.0.join("source/API.lua"), "changed")?;
        assert!(read_local_source_reference(&config, &stop).is_err());
        let cancelled = AtomicBool::new(true);
        assert!(
            matches!(read_local_source_reference(&config, &cancelled), Err(error) if error.code() == ServiceErrorCode::Cancelled)
        );
        assert!(
            matches!(first.canonical_bytes(&cancelled), Err(error) if error.code() == ServiceErrorCode::Cancelled)
        );
        Ok(())
    }
}

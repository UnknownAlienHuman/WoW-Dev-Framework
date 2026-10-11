//! One read-only operation over an explicitly supplied local source manifest.
//! No acquisition, selector resolution, analyzer, graph or publication effects.
use crate::{ServiceError, ServiceErrorCode, ServiceResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_core::{ContentDigest, SourceContent};
pub use wow_project::load::census::CensusCoverage;
use wow_project::{
    ProjectError, ProjectErrorCode,
    disk::{ManifestedPlatformInput, PlatformSourceManifestReceipt, ProjectInputDirectory},
    load::{
        ProjectPackageInput,
        census::{
            PlatformCensusSelection, PlatformPackageLoadCensus, PlatformSourceCensus,
            census_platform_packages, census_platform_source,
        },
    },
    platform_source::{
        BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformSourceInventory,
    },
};

pub const SOURCE_CENSUS_INPUT_SCHEMA: &str = "wow-service/source-census-input/1";
pub const SOURCE_CENSUS_MANIFEST_INPUT_SCHEMA: &str = "wow-service/source-census-manifest-input/1";
pub const SOURCE_PACKAGE_CENSUS_INPUT_SCHEMA: &str = "wow-service/source-package-census-input/1";
const RESULT_SCHEMA: &str = "wow-service/source-census-result/1";
const MANIFEST_RESULT_SCHEMA: &str = "wow-service/source-census-manifest-result/1";
const PACKAGE_RESULT_SCHEMA: &str = "wow-service/source-package-census-result/2";
const MAX_RESULT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(tag = "schema", deny_unknown_fields)]
enum Input {
    #[serde(rename = "wow-service/source-census-input/1")]
    Inventory {
        profile: BlizzardUiSourceProfileRequest,
        inventory: Box<PlatformSourceInventory>,
        selection: PlatformCensusSelection,
    },
    #[serde(rename = "wow-service/source-census-manifest-input/1")]
    Manifest {
        profile: BlizzardUiSourceProfileRequest,
        source: Box<ManifestedPlatformInput>,
        selection: PlatformCensusSelection,
    },
    #[serde(rename = "wow-service/source-package-census-input/1")]
    PackageManifest {
        profile: BlizzardUiSourceProfileRequest,
        source: Box<ManifestedPlatformInput>,
        selection: PlatformCensusSelection,
        packages: Vec<ProjectPackageInput>,
    },
}

#[derive(Debug, Serialize)]
pub struct SourceCensusResult {
    schema: &'static str,
    configuration_content_digest: ContentDigest<SourceContent>,
    census: PlatformSourceCensus,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_manifest: Option<PlatformSourceManifestReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_loads: Option<PlatformPackageLoadCensus>,
}
impl SourceCensusResult {
    pub const fn census(&self) -> &PlatformSourceCensus {
        &self.census
    }
    pub const fn package_loads(&self) -> Option<&PlatformPackageLoadCensus> {
        self.package_loads.as_ref()
    }
    pub fn coverage(&self) -> CensusCoverage {
        if self.census.coverage() == CensusCoverage::Partial
            || self
                .package_loads
                .as_ref()
                .is_some_and(|loads| loads.coverage() == CensusCoverage::Partial)
        {
            CensusCoverage::Partial
        } else {
            CensusCoverage::DeclaredMembersMeasured
        }
    }

    /// Service-approved bounded output. The frontend writes these exact bytes.
    pub fn canonical_bytes(&self) -> ServiceResult<Vec<u8>> {
        let mut buffer = Output {
            bytes: Vec::new(),
            exhausted: false,
        };
        if serde_json::to_writer(&mut buffer, self).is_err() {
            return Err(failure(if buffer.exhausted {
                ServiceErrorCode::BudgetExceeded
            } else {
                ServiceErrorCode::CanonicalizationFailed
            }));
        }
        Ok(buffer.bytes)
    }
}

/// Register the explicit configuration parent once. Every source member is
/// resolved by the project disk owner relative to that retained handle.
pub fn census_local_source(config: &Path, stop: &AtomicBool) -> ServiceResult<SourceCensusResult> {
    checkpoint(stop)?;
    let parent = config
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = config
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.ends_with(".json"))
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    let directory = ProjectInputDirectory::open(parent).map_err(owner_error)?;
    let bytes = directory
        .read_configuration(name, stop)
        .map_err(owner_error)?;
    let input: Input =
        serde_json::from_slice(&bytes).map_err(|_| failure(ServiceErrorCode::InvalidRequest))?;
    let (source, selection, source_manifest, packages) = match input {
        Input::Inventory {
            profile,
            inventory,
            selection,
        } => {
            let profile = BlizzardUiSourceProfile::new(profile).map_err(owner_error)?;
            let source = directory
                .admit_platform_source(&profile, *inventory, stop)
                .map_err(owner_error)?;
            (source, selection, None, None)
        }
        Input::Manifest {
            profile,
            source,
            selection,
        } => {
            let profile = BlizzardUiSourceProfile::new(profile).map_err(owner_error)?;
            let (source, receipt) = directory
                .admit_platform_source_manifest(&profile, &source, stop)
                .map_err(owner_error)?
                .into_parts();
            (source, selection, Some(receipt), None)
        }
        Input::PackageManifest {
            profile,
            source,
            selection,
            packages,
        } => {
            let profile = BlizzardUiSourceProfile::new(profile).map_err(owner_error)?;
            let (source, receipt) = directory
                .admit_platform_source_manifest(&profile, &source, stop)
                .map_err(owner_error)?
                .into_parts();
            (source, selection, Some(receipt), Some(packages))
        }
    };
    let census = census_platform_source(&source, &selection, stop).map_err(owner_error)?;
    let package_loads = packages
        .as_ref()
        .map(|packages| {
            census_platform_packages(&source, packages, selection.load_context.as_ref(), stop)
                .map_err(owner_error)
        })
        .transpose()?;
    checkpoint(stop)?;
    Ok(SourceCensusResult {
        schema: if package_loads.is_some() {
            PACKAGE_RESULT_SCHEMA
        } else if source_manifest.is_some() {
            MANIFEST_RESULT_SCHEMA
        } else {
            RESULT_SCHEMA
        },
        configuration_content_digest: ContentDigest::from_bytes(Sha256::digest(bytes).into()),
        census,
        source_manifest,
        package_loads,
    })
}

fn owner_error(error: ProjectError) -> ServiceError {
    let code = match error.code() {
        ProjectErrorCode::SourceReadCancelled | ProjectErrorCode::AnalysisCancelled => {
            ServiceErrorCode::Cancelled
        }
        ProjectErrorCode::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
        ProjectErrorCode::SourceReadFailed => ServiceErrorCode::ComponentUnavailable,
        _ => ServiceErrorCode::IdentityMismatch,
    };
    // Project acquisition/load errors contain fixed owner explanations, never
    // source text, host paths or raw OS errors, as in the local disk adapter.
    ServiceError::new(code, error.message())
}
fn checkpoint(stop: &AtomicBool) -> ServiceResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(ServiceErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn failure(code: ServiceErrorCode) -> ServiceError {
    ServiceError::new(
        code,
        "local source census request or owner input was rejected",
    )
}

struct Output {
    bytes: Vec<u8>,
    exhausted: bool,
}
impl io::Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|size| size > MAX_RESULT_BYTES)
            || self.bytes.try_reserve(bytes.len()).is_err()
        {
            self.exhausted = true;
            return Err(io::Error::other("source census output budget exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

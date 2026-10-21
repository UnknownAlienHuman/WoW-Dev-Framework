//! Genuine platform archives hold requests and every Included raw member.
//! Hydration re-enters source and package owners, never decoded receipts.
use std::sync::{Arc, atomic::AtomicBool};

use serde::{Deserialize, Serialize, ser::SerializeSeq};
use wow_core::{CanonicalResult, ContentDigest};

use super::{MAX_FILE_BYTES, MAX_FILES, MAX_SOURCE_BYTES, exhausted, invalid};
use crate::{
    ProjectInputFile, ProjectResult,
    platform_source::{
        AdmittedPlatformSource, BlizzardUiSourceProfileRequest, PlatformPackageRequest,
        PlatformPackageSpecialization, PlatformSourceInventory, RetainedPlatformFile,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureRef {
    path: String,
    reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReplayPlatform {
    profile: BlizzardUiSourceProfileRequest,
    inventory: PlatformSourceInventory,
    files: Vec<RetainedPlatformFile>,
    package_request: PlatformPackageRequest,
    expected_binding_digest: ContentDigest<CanonicalResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    main_fixture_refs: Vec<FixtureRef>,
}

/// Exact borrowed wire projection lets capture charge encoded raw bytes before
/// cloning. Binary data remains a JSON byte sequence, never decoded source text.
#[derive(Serialize)]
struct BorrowedPlatform<'a> {
    profile: &'a BlizzardUiSourceProfileRequest,
    inventory: &'a PlatformSourceInventory,
    files: BorrowedFiles<'a>,
    package_request: &'a PlatformPackageRequest,
    expected_binding_digest: ContentDigest<CanonicalResult>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    main_fixture_refs: &'a Vec<FixtureRef>,
}
struct BorrowedFiles<'a>(&'a AdmittedPlatformSource);
impl Serialize for BorrowedFiles<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct File<'a> {
            path: &'a str,
            bytes: &'a [u8],
        }
        let mut sequence =
            serializer.serialize_seq(Some(self.0.receipt().coverage().verified_files()))?;
        for (path, bytes) in self.0.retained_files() {
            sequence.serialize_element(&File { path, bytes })?;
        }
        sequence.end()
    }
}

impl ReplayPlatform {
    pub(super) fn capture(
        owner: &PlatformPackageSpecialization,
        main: &[ProjectInputFile],
        stop: &AtomicBool,
        count: &mut usize,
        bytes: &mut usize,
        encoded_limit: usize,
    ) -> ProjectResult<Self> {
        owner
            .binding()
            .validate(owner.source(), owner.load_plan(), owner.main_plan())?;
        owner.main_plan().validate_main_files(main)?;
        for (path, raw) in owner.source().retained_files() {
            charge(path, raw, count, bytes, stop)?;
        }
        let mut main_fixture_refs = main
            .iter()
            .filter_map(|file| {
                file.source_fixture_ref().map(|reference| FixtureRef {
                    path: file.relative_path().as_str().into(),
                    reference: reference.into(),
                })
            })
            .collect::<Vec<_>>();
        main_fixture_refs.sort_by(|a, b| a.path.cmp(&b.path));
        super::encoded_size_with_limit(
            &BorrowedPlatform {
                profile: owner.source().profile().request(),
                inventory: owner.source().receipt().inventory(),
                files: BorrowedFiles(owner.source()),
                package_request: owner.request(),
                expected_binding_digest: owner.binding().binding_digest(),
                main_fixture_refs: &main_fixture_refs,
            },
            encoded_limit,
            stop,
        )?;
        let mut files = Vec::new();
        for (path, bytes) in owner.source().retained_files() {
            crate::analyzer::checkpoint(stop)?;
            files.push(RetainedPlatformFile {
                path: path.into(),
                bytes: bytes.into(),
            });
        }
        Ok(Self {
            profile: owner.source().profile().request().clone(),
            inventory: owner.source().receipt().inventory().clone(),
            files,
            package_request: owner.request().clone(),
            expected_binding_digest: owner.binding().binding_digest(),
            main_fixture_refs,
        })
    }

    pub(super) fn validate_budget(
        &self,
        count: &mut usize,
        bytes: &mut usize,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        if self
            .files
            .windows(2)
            .any(|pair| pair[0].path >= pair[1].path)
            || self.main_fixture_refs.len() > MAX_FILES
            || self
                .main_fixture_refs
                .windows(2)
                .any(|pair| pair[0].path >= pair[1].path)
            || self
                .main_fixture_refs
                .iter()
                .any(|item| item.path.len() > 4096 || item.reference.len() > 4096)
        {
            return Err(invalid());
        }
        for file in &self.files {
            charge(&file.path, &file.bytes, count, bytes, stop)?;
        }
        Ok(())
    }

    pub(super) fn rebuild(
        &self,
        stop: &AtomicBool,
    ) -> ProjectResult<Arc<PlatformPackageSpecialization>> {
        let source = Arc::new(AdmittedPlatformSource::readmit(
            self.profile.clone(),
            self.inventory.clone(),
            &self.files,
            stop,
        )?);
        let owner = self.package_request.rebuild(&source, stop)?;
        if owner.binding().binding_digest() != self.expected_binding_digest {
            return Err(invalid());
        }
        Ok(Arc::new(owner))
    }

    pub(super) fn main_files(
        &self,
        owner: &PlatformPackageSpecialization,
    ) -> ProjectResult<Vec<ProjectInputFile>> {
        let mut expected = self.main_fixture_refs.iter().peekable();
        let mut files = owner.files().iter().collect::<Vec<_>>();
        files.sort_by(|a, b| a.relative_path().as_str().cmp(b.relative_path().as_str()));
        let mut restored = Vec::new();
        for file in files {
            let path = file.relative_path().as_str();
            let fixture = if expected.peek().is_some_and(|item| item.path == path) {
                Some(expected.next().ok_or_else(invalid)?.reference.clone())
            } else {
                None
            };
            restored.push(ProjectInputFile::declared(
                path,
                file.retained_text(),
                file.language_kind(),
                file.role(),
                fixture,
            )?);
        }
        if expected.next().is_some() {
            return Err(invalid());
        }
        Ok(restored)
    }
}

fn charge(
    path: &str,
    raw: &[u8],
    count: &mut usize,
    bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    *count = count.checked_add(1).ok_or_else(exhausted)?;
    *bytes = bytes.checked_add(raw.len()).ok_or_else(exhausted)?;
    if *count > MAX_FILES
        || *bytes > MAX_SOURCE_BYTES
        || raw.len() > MAX_FILE_BYTES
        || path.len() > 4096
    {
        return Err(exhausted());
    }
    Ok(())
}

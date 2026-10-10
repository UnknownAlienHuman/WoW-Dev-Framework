//! Exact package specialization of one held source admission. Parsing, static
//! dependency closure and Main namespacing remain the existing load owners.
use std::{
    collections::BTreeSet,
    sync::{Arc, atomic::AtomicBool},
};

use serde::{Deserialize, Serialize};

use super::{
    AdmittedPlatformSource, PlatformEntryDisposition, PlatformFileKind, invalid,
    package_binding::PlatformPackageBinding, path, under,
};
use crate::{
    ProjectError, ProjectErrorCode, ProjectInputFile, ProjectPhase, ProjectResult,
    disk::{self, ProjectDiskFile},
    load::{
        ProjectPackageInput, ProjectPackageLoadPlan, ProjectPackageMainInput,
        ProjectPackageMainPlan, ProjectPackageVariantInput, TocLoadContext, read_admitted_packages,
        validate_package_declarations,
    },
};

/// Original caller declarations, not an admitted source or native owner receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlatformPackageRequest {
    packages: Vec<ProjectPackageInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    context: Option<TocLoadContext>,
}

impl PlatformPackageRequest {
    /// Re-admit original roots, variant pins and selection through native owners.
    pub(crate) fn rebuild(
        &self,
        source: &Arc<AdmittedPlatformSource>,
        stop: &AtomicBool,
    ) -> ProjectResult<PlatformPackageSpecialization> {
        source.specialize_packages(&self.packages, self.context.as_ref(), stop)
    }
}

/// Actual native package/Main owners bound to their retained original source.
/// This is not a platform project/analyzer/graph or publication capability.
pub struct PlatformPackageSpecialization {
    source: Arc<AdmittedPlatformSource>,
    binding: PlatformPackageBinding,
    main: ProjectPackageMainInput,
    request: PlatformPackageRequest,
}
impl PlatformPackageSpecialization {
    #[must_use]
    pub const fn source(&self) -> &Arc<AdmittedPlatformSource> {
        &self.source
    }
    #[must_use]
    pub const fn binding(&self) -> &PlatformPackageBinding {
        &self.binding
    }
    #[must_use]
    pub fn files(&self) -> &[ProjectInputFile] {
        self.main.files()
    }
    #[must_use]
    pub fn load_plan(&self) -> &ProjectPackageLoadPlan {
        self.main.load_plan()
    }
    #[must_use]
    pub fn main_plan(&self) -> &ProjectPackageMainPlan {
        self.main.main_plan()
    }
    #[must_use]
    pub(crate) const fn request(&self) -> &PlatformPackageRequest {
        &self.request
    }
}

impl AdmittedPlatformSource {
    /// Resolve only explicit package/root/variant requests against retained bytes.
    /// Unknown inventory bytes are never decoded unless actually demanded.
    pub fn specialize_packages(
        self: &Arc<Self>,
        packages: &[ProjectPackageInput],
        context: Option<&TocLoadContext>,
        stop: &AtomicBool,
    ) -> ProjectResult<PlatformPackageSpecialization> {
        disk::checkpoint(stop)?;
        validate_package_declarations(packages)?;
        if !packages.iter().any(ProjectPackageInput::selected_root) {
            return Err(invalid(
                "platform packages require an explicit selected root",
            ));
        }
        if let Some(context) = context {
            context.validate()?;
        }
        let expected: BTreeSet<_> = self
            .profile()
            .roots()
            .iter()
            .flat_map(|root| root.selected_tocs.iter().map(String::as_str))
            .collect();
        let mut selected = BTreeSet::new();
        let mut pinned = Vec::with_capacity(packages.len());
        for package in packages {
            disk::checkpoint(stop)?;
            path(package.root())?;
            if !self
                .profile()
                .roots()
                .iter()
                .any(|root| package.root() == root.root || under(package.root(), &root.root))
            {
                return Err(invalid(
                    "platform package root is outside its admitted scope",
                ));
            }
            let mut variants = Vec::with_capacity(package.variants().len());
            for variant in package.variants() {
                disk::checkpoint(stop)?;
                let full = format!("{}/{}", package.root(), variant.toc().path());
                path(&full)?;
                let entry = self
                    .receipt()
                    .inventory()
                    .entries
                    .binary_search_by(|entry| entry.path.as_str().cmp(&full))
                    .ok()
                    .and_then(|index| self.receipt().inventory().entries.get(index))
                    .ok_or_else(|| {
                        failure(
                            ProjectErrorCode::MissingDeclaredFile,
                            "platform TOC variant is not declared",
                            &full,
                        )
                    })?;
                if entry.kind != PlatformFileKind::Toc {
                    return Err(failure(
                        ProjectErrorCode::InvalidFileLanguage,
                        "platform package variant must be an admitted TOC",
                        &full,
                    ));
                }
                let bytes = self.load_member(
                    package.root(),
                    variant.toc(),
                    disk::DISK_SOURCE_MAX_BYTES,
                    stop,
                )?;
                if variant.selected()
                    && (!expected.contains(full.as_str()) || !selected.insert(full.clone()))
                {
                    return Err(invalid(
                        "platform selected TOC differs from its exact profile selection",
                    ));
                }
                variants.push(ProjectPackageVariantInput::new(
                    variant
                        .toc()
                        .clone()
                        .with_identity(crate::identity::source_digest(bytes), bytes.len() as u64),
                    variant.selected(),
                ));
            }
            pinned.push(ProjectPackageInput::new(
                package.name(),
                package.root(),
                package.selected_root(),
                variants,
            ));
        }
        if selected.iter().map(String::as_str).collect::<BTreeSet<_>>() != expected {
            return Err(invalid("platform package set omits a selected profile TOC"));
        }
        disk::checkpoint(stop)?;
        let loaded = read_admitted_packages(
            self,
            &pinned,
            &self.profile().target().reference_profile,
            context,
            stop,
        )?;
        let main = loaded.into_namespaced_main()?;
        let binding = PlatformPackageBinding::new(self, main.load_plan(), main.main_plan())?;
        binding.validate(self, main.load_plan(), main.main_plan())?;
        let request = PlatformPackageRequest {
            packages: packages.to_vec(),
            context: context.cloned(),
        };
        disk::checkpoint(stop)?;
        Ok(PlatformPackageSpecialization {
            source: Arc::clone(self),
            binding,
            main,
            request,
        })
    }

    /// Crate-private demand port used by the native loader. Returned bytes are
    /// immutable admission bytes; explicit omissions are never missing-disk guesses.
    pub(crate) fn load_member<'a>(
        &'a self,
        root: &str,
        selected: &ProjectDiskFile,
        limit: usize,
        stop: &AtomicBool,
    ) -> ProjectResult<&'a [u8]> {
        disk::checkpoint(stop)?;
        path(root)?;
        selected.validate()?;
        if !self
            .profile()
            .roots()
            .iter()
            .any(|declared| root == declared.root || under(root, &declared.root))
        {
            return Err(invalid("platform load root is outside the admitted source"));
        }
        let full = format!("{root}/{}", selected.path());
        path(&full)?;
        let entry = self
            .receipt()
            .inventory()
            .entries
            .binary_search_by(|entry| entry.path.as_str().cmp(&full))
            .ok()
            .and_then(|index| self.receipt().inventory().entries.get(index))
            .ok_or_else(|| {
                failure(
                    ProjectErrorCode::MissingDeclaredFile,
                    "platform load target has no declared inventory entry",
                    &full,
                )
            })?;
        match &entry.disposition {
            PlatformEntryDisposition::Included { byte_length, .. } => {
                if *byte_length > limit as u64 {
                    return Err(failure(
                        ProjectErrorCode::SourceBudgetExceeded,
                        "platform load target exceeds the native consumed-byte limit",
                        &full,
                    ));
                }
                let bytes = self
                    .source_bytes(&full)
                    .map_err(|_| invalid("included platform member is not retained"))?;
                selected.verify(bytes)?;
                disk::checkpoint(stop)?;
                Ok(bytes)
            }
            PlatformEntryDisposition::Excluded { .. } => Err(failure(
                ProjectErrorCode::PackageTargetExcluded,
                "platform load target is an explicitly excluded inventory entry",
                &full,
            )),
            PlatformEntryDisposition::Unsupported { .. } => Err(failure(
                ProjectErrorCode::InvalidFileLanguage,
                "platform load target is an unsupported special entry",
                &full,
            )),
            PlatformEntryDisposition::External { .. } => Err(failure(
                ProjectErrorCode::PackageTargetUnresolved,
                "platform load target is not materialized external content",
                &full,
            )),
            PlatformEntryDisposition::Conflict { .. } => Err(failure(
                ProjectErrorCode::PackageTargetUnresolved,
                "platform load target has conflicting inventory evidence",
                &full,
            )),
            PlatformEntryDisposition::Failed { .. } => Err(failure(
                ProjectErrorCode::PackageTargetUnresolved,
                "platform load target has failed materialization evidence",
                &full,
            )),
        }
    }
}
fn failure(code: ProjectErrorCode, message: &'static str, relative: &str) -> ProjectError {
    ProjectError::new(code, ProjectPhase::Inventory, message).with_relative_path(relative)
}

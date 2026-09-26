//! Explicit consumer resources for the native annotation pipeline. Acquisition
//! belongs to wow-project; correction/catalog semantics stay with their owners.
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Deserializer, Serialize};
use wow_annotations::aliases::{MAX_CATALOG_ALIASES, MAX_CATALOG_BYTES, MAX_CATALOG_FILES};
use wow_annotations::native::NativeLibrary;
use wow_core::{ContentDigest, CorrectionSet};
use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
use wow_reference::native::{NativeError, NativeErrorCode, source_digest};
use wow_reference::native_aliases::{
    AliasDocument, MAX_ALIAS_RESOURCE_BYTES, ingest_alias_catalog,
};
use wow_reference::native_corrections::{
    CorrectionError, MAX_CORRECTION_SET_BYTES, Status, ValidatedCorrections,
};

use super::cancelled;
use super::disk_input::acquisition_error;
use super::input::invalid;
use super::native_input::NativeFileIdentity;
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

/// Omission is supported; explicit null is not an instruction to disable input.
pub(super) fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeAnnotationInputs {
    #[serde(default, deserialize_with = "present")]
    corrections: Option<ProjectDiskFile>,
    #[serde(default, deserialize_with = "present")]
    alias_catalogs: Option<CatalogInputs>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogInputs {
    root: String,
    revision: String,
    files: Vec<ProjectDiskFile>,
}

/// Separate input provenance, not platform truth or authenticated review.
/// Host roots do not participate in semantic identity or escape into reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeAnnotationSelection {
    pub schema: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_file: Option<NativeFileIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_set_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias_revision: Option<String>,
    pub alias_files: Vec<NativeFileIdentity>,
    pub catalog_ingestion: &'static str,
    pub reference_authority: &'static str,
    pub git_membership: &'static str,
    pub review_authority: &'static str,
}

/// Exact producer outcome counts without duplicating the raw resources or maps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeAnnotationInputsReceipt {
    pub selection: NativeAnnotationSelection,
    pub correction_outcomes: BTreeMap<&'static str, usize>,
    pub alias_outcomes: BTreeMap<&'static str, BTreeMap<&'static str, usize>>,
    pub unresolved_structure_fields: usize,
    pub unresolved_function_container_returns: usize,
    pub unresolved_global_color_types: usize,
}

pub(super) struct NativeAnnotationResources {
    pub(super) corrections: Option<ValidatedCorrections>,
    pub(super) aliases: Vec<AliasDocument>,
    pub(super) selection: NativeAnnotationSelection,
}

impl NativeAnnotationInputs {
    pub(super) fn read(
        &self,
        directory: &ProjectInputDirectory,
        stop: &AtomicBool,
    ) -> ServiceResult<NativeAnnotationResources> {
        cancelled(stop)?;
        if self.corrections.is_none() && self.alias_catalogs.is_none() {
            return Err(invalid(
                "annotation_inputs must explicitly select a resource",
            ));
        }
        if let Some(catalogs) = &self.alias_catalogs {
            if !matches!(catalogs.revision.len(), 40 | 64)
                || !catalogs
                    .revision
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(invalid(
                    "alias catalogs require one exact external revision",
                ));
            }
            if catalogs.files.is_empty() {
                return Err(invalid("alias catalog selection must not be empty"));
            }
            if catalogs.files.len() > MAX_CATALOG_FILES {
                return Err(limit("alias catalog file count exceeds budget"));
            }
        }
        let mut selection = NativeAnnotationSelection {
            schema: "wow-service/native-annotation-inputs/1",
            correction_file: None,
            correction_set_sha256: None,
            alias_revision: self.alias_catalogs.as_ref().map(|v| v.revision.clone()),
            alias_files: Vec::new(),
            catalog_ingestion: "wow-reference::native_aliases::ingest_alias_catalog",
            reference_authority: "raw_source_only",
            git_membership: "not_attested",
            review_authority: "caller_declared_not_authenticated",
        };
        let corrections = if let Some(file) = &self.corrections {
            let bytes = directory
                .read_json_artifact_with_limit(file, MAX_CORRECTION_SET_BYTES, stop)
                .map_err(acquisition_error)?;
            cancelled(stop)?;
            let result = ValidatedCorrections::from_json(&bytes);
            cancelled(stop)?;
            let corrections = result.map_err(|error| match error {
                CorrectionError::Cancelled => ServiceError::new(
                    ServiceErrorCode::Cancelled,
                    "native correction admission cancelled",
                ),
                CorrectionError::Limit => limit("native correction set exceeds budget"),
                _ => invalid("native correction set was rejected"),
            })?;
            selection.correction_file = Some(NativeFileIdentity {
                path: file.path().to_owned(),
                sha256: source_digest(&bytes),
                byte_length: bytes.len() as u64,
            });
            selection.correction_set_sha256 = Some(corrections.id().to_owned());
            Some(corrections)
        } else {
            None
        };
        let mut aliases = Vec::new();
        if let Some(catalogs) = &self.alias_catalogs {
            let sources = directory
                .read_pinned_lua_sources_with_limits(
                    &catalogs.root,
                    &catalogs.files,
                    MAX_ALIAS_RESOURCE_BYTES,
                    MAX_CATALOG_BYTES,
                    stop,
                )
                .map_err(acquisition_error)?;
            let mut declarations = 0usize;
            for file in sources {
                cancelled(stop)?;
                let document = ingest_alias_catalog(
                    &catalogs.revision,
                    file.path(),
                    file.text(),
                    &file.content_digest().to_string(),
                    stop,
                )
                .map_err(catalog_error)?;
                declarations = declarations
                    .checked_add(
                        document.aliases().len()
                            + document.structures().len()
                            + document.namespaces().len()
                            + document.global_colors().len()
                            + document
                                .function_containers()
                                .iter()
                                .map(|container| 1 + container.methods.len())
                                .sum::<usize>(),
                    )
                    .ok_or_else(|| limit("alias declaration count overflow"))?;
                if declarations > MAX_CATALOG_ALIASES {
                    return Err(limit("alias catalog declaration count exceeds budget"));
                }
                selection.alias_files.push(NativeFileIdentity {
                    path: file.path().to_owned(),
                    sha256: file.content_digest().to_string(),
                    byte_length: file.text().len() as u64,
                });
                aliases.push(document);
            }
        }
        cancelled(stop)?;
        Ok(NativeAnnotationResources {
            corrections,
            aliases,
            selection,
        })
    }
}

impl NativeAnnotationResources {
    pub(super) fn correction_digest(&self) -> ServiceResult<Option<ContentDigest<CorrectionSet>>> {
        self.corrections
            .as_ref()
            .map(|set| {
                set.id()
                    .parse()
                    .map_err(|_| invalid("native correction identity rejected"))
            })
            .transpose()
    }

    pub(super) fn receipt(&self, library: &NativeLibrary<'_>) -> NativeAnnotationInputsReceipt {
        let mut receipt = NativeAnnotationInputsReceipt {
            selection: self.selection.clone(),
            correction_outcomes: BTreeMap::new(),
            alias_outcomes: BTreeMap::new(),
            unresolved_structure_fields: 0,
            unresolved_function_container_returns: 0,
            unresolved_global_color_types: 0,
        };
        if let Some(report) = &library.corrections {
            for application in &report.applications {
                let status = match application.status {
                    Status::Applied => "applied",
                    Status::Expired => "expired",
                    Status::Rejected => "rejected",
                    Status::Conflict => "conflict",
                    Status::NotApplicable => "not_applicable",
                };
                *receipt.correction_outcomes.entry(status).or_default() += 1;
            }
        }
        if let Some(report) = &library.aliases {
            for (family, outcomes) in [
                ("aliases", &report.outcomes),
                ("structures", &report.structure_outcomes),
                ("namespaces", &report.namespace_outcomes),
                ("function_containers", &report.function_container_outcomes),
                ("global_colors", &report.global_color_outcomes),
            ] {
                let counts = receipt.alias_outcomes.entry(family).or_default();
                for outcome in outcomes {
                    *counts.entry(outcome.status).or_default() += 1;
                }
            }
            receipt.unresolved_structure_fields = report.unresolved_structure_fields.len();
            receipt.unresolved_function_container_returns =
                report.unresolved_function_container_returns.len();
            receipt.unresolved_global_color_types = report.unresolved_global_color_types.len();
        }
        receipt
    }
}

fn limit(message: &'static str) -> ServiceError {
    ServiceError::new(ServiceErrorCode::BudgetExceeded, message)
}

fn catalog_error(error: NativeError) -> ServiceError {
    match error.code {
        NativeErrorCode::Cancelled => ServiceError::new(
            ServiceErrorCode::Cancelled,
            "native alias catalog admission cancelled",
        ),
        NativeErrorCode::Limit => limit("native alias catalog exceeds budget"),
        _ => invalid("native alias catalog was rejected"),
    }
}

//! Explicit bounded observation selection; it cannot reconstruct native authority.
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use wow_project::graph::{
    PlatformGraphProvenance,
    fingerprints::{
        FunctionFingerprintLimits, FunctionFingerprintManifest,
        build_platform_function_fingerprints, build_platform_function_fingerprints_with_load_roles,
    },
};

use super::{checkpoint, error, project_error};
use crate::{ServiceErrorCode, ServiceResult};
pub use wow_project::graph::fingerprints::FunctionPackageLoadRoleRecipe;

const SELECTION_SCHEMA: &str = "wow-service/function-fingerprint-selection/1";
const LOAD_ROLE_SELECTION_SCHEMA: &str =
    "wow-service/function-fingerprint-selection/package-load-roles/1";

/// Exact current physical function IDs and cumulative observation limits.
/// Selection says nothing about functions omitted from this request.
#[derive(Debug, Clone, Serialize)]
pub struct FunctionFingerprintSelection {
    schema: &'static str,
    function_ids: Vec<Box<str>>,
    max_records: usize,
    max_source_bytes: usize,
    max_metadata_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_load_roles: Option<FunctionPackageLoadRoleRecipe>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSelection {
    schema: String,
    function_ids: Vec<Box<str>>,
    max_records: usize,
    max_source_bytes: usize,
    max_metadata_bytes: usize,
    #[serde(default, deserialize_with = "non_null_load_roles")]
    package_load_roles: Option<FunctionPackageLoadRoleRecipe>,
}

fn non_null_load_roles<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<FunctionPackageLoadRoleRecipe>, D::Error> {
    FunctionPackageLoadRoleRecipe::deserialize(deserializer).map(Some)
}

impl FunctionFingerprintSelection {
    pub fn new(
        function_ids: Vec<Box<str>>,
        max_records: usize,
        max_source_bytes: usize,
        max_metadata_bytes: usize,
    ) -> ServiceResult<Self> {
        if max_records == 0 || max_source_bytes == 0 || max_metadata_bytes == 0 {
            return Err(error(ServiceErrorCode::InvalidRequest));
        }
        if function_ids.len() > max_records {
            return Err(error(ServiceErrorCode::BudgetExceeded));
        }
        let selection = Self {
            schema: SELECTION_SCHEMA,
            function_ids,
            max_records,
            max_source_bytes,
            max_metadata_bytes,
            package_load_roles: None,
        };
        // Bound the entire selection before any later request clone or native lookup.
        super::preflight_cancellable(
            &selection,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            &AtomicBool::new(false),
        )?;
        Ok(selection)
    }

    pub fn with_package_load_roles(
        mut self,
        recipe: FunctionPackageLoadRoleRecipe,
    ) -> ServiceResult<Self> {
        self.schema = LOAD_ROLE_SELECTION_SCHEMA;
        self.package_load_roles = Some(recipe);
        super::preflight_cancellable(
            &self,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            &AtomicBool::new(false),
        )?;
        Ok(self)
    }

    pub const fn package_load_roles(&self) -> Option<FunctionPackageLoadRoleRecipe> {
        self.package_load_roles
    }

    pub fn from_json_slice(bytes: &[u8]) -> ServiceResult<Self> {
        Self::from_json_slice_cancellable(bytes, &AtomicBool::new(false))
    }

    pub fn from_json_slice_cancellable(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        checkpoint(stop)?;
        let wire: WireSelection = super::super::input::decode(
            bytes,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            super::super::GraphReadStage::Request,
            stop,
        )
        .map_err(|failure| error(failure.code))?;
        let expected_schema = if wire.package_load_roles.is_some() {
            LOAD_ROLE_SELECTION_SCHEMA
        } else {
            SELECTION_SCHEMA
        };
        if wire.schema != expected_schema {
            return Err(error(ServiceErrorCode::InvalidRequest));
        }
        let mut selection = Self::new(
            wire.function_ids,
            wire.max_records,
            wire.max_source_bytes,
            wire.max_metadata_bytes,
        )?;
        if let Some(recipe) = wire.package_load_roles {
            selection = selection.with_package_load_roles(recipe)?;
        }
        checkpoint(stop)?;
        Ok(selection)
    }

    pub(super) fn build(
        &self,
        direct: &PlatformGraphProvenance<'_>,
        stop: &AtomicBool,
    ) -> ServiceResult<FunctionFingerprintManifest> {
        checkpoint(stop)?;
        let ids = self
            .function_ids
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();
        let limits = FunctionFingerprintLimits::new(
            self.max_records,
            self.max_source_bytes,
            self.max_metadata_bytes,
        )
        .map_err(project_error)?;
        let result = match self.package_load_roles {
            None => build_platform_function_fingerprints(direct, &ids, limits, stop),
            Some(recipe) => build_platform_function_fingerprints_with_load_roles(
                direct, &ids, limits, recipe, stop,
            ),
        };
        result.map_err(|failure| {
            if failure.code() == wow_project::ProjectErrorCode::InvalidFingerprintSelection {
                error(ServiceErrorCode::InvalidRequest)
            } else {
                project_error(failure)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn selected_load_roles_require_exact_schema_recipe_and_non_null_input() -> TestResult {
        let original = FunctionFingerprintSelection::new(Vec::new(), 1, 1, 4096)?;
        let old = serde_json::to_value(&original)?;
        assert_eq!(old["schema"], SELECTION_SCHEMA);
        assert!(old.get("package_load_roles").is_none());
        let selected = original
            .with_package_load_roles(FunctionPackageLoadRoleRecipe::SelectedPhysicalLuaV1)?;
        let bytes = serde_json::to_vec(&selected)?;
        assert_eq!(
            FunctionFingerprintSelection::from_json_slice(&bytes)?.package_load_roles(),
            Some(FunctionPackageLoadRoleRecipe::SelectedPhysicalLuaV1)
        );
        let selected = serde_json::to_value(&selected)?;
        for (field, value) in [
            ("schema", serde_json::json!(SELECTION_SCHEMA)),
            ("package_load_roles", serde_json::Value::Null),
            (
                "package_load_roles",
                serde_json::json!("wow-project/physical-function-package-load-role/2"),
            ),
        ] {
            let mut invalid = selected.clone();
            invalid[field] = value;
            let error =
                FunctionFingerprintSelection::from_json_slice(&serde_json::to_vec(&invalid)?)
                    .err()
                    .ok_or("expected invalid selector")?;
            assert_eq!(error.code(), ServiceErrorCode::InvalidRequest);
        }
        let mut missing = selected.clone();
        missing
            .as_object_mut()
            .ok_or("selector object")?
            .remove("package_load_roles");
        assert_eq!(
            FunctionFingerprintSelection::from_json_slice(&serde_json::to_vec(&missing)?)
                .err()
                .ok_or("expected schema mismatch")?
                .code(),
            ServiceErrorCode::InvalidRequest
        );
        assert_eq!(
            FunctionFingerprintSelection::from_json_slice_cancellable(
                &bytes,
                &AtomicBool::new(true)
            )
            .err()
            .ok_or("expected cancellation")?
            .code(),
            ServiceErrorCode::Cancelled
        );
        Ok(())
    }
}

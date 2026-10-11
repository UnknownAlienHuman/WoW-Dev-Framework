//! Explicit bounded observation selection; it cannot reconstruct native authority.
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use wow_project::graph::{
    PlatformGraphProvenance,
    fingerprints::{
        FunctionFingerprintLimits, FunctionFingerprintManifest, FunctionNeighborhoodComposition,
        build_platform_function_fingerprints, build_platform_function_fingerprints_composed,
        build_platform_function_fingerprints_with_composite,
        build_platform_function_fingerprints_with_load_roles,
    },
};

use super::{checkpoint, error, project_error};
use crate::{ServiceErrorCode, ServiceResult};
pub use wow_project::graph::fingerprints::{
    FunctionCompositeRecipe, FunctionDirectNeighborhoodRecipe, FunctionPackageLoadRoleRecipe,
};

const SELECTION_SCHEMA: &str = "wow-service/function-fingerprint-selection/1";
const LOAD_ROLE_SELECTION_SCHEMA: &str =
    "wow-service/function-fingerprint-selection/package-load-roles/1";
const SELECTED_COMPONENTS_SELECTION_SCHEMA: &str =
    "wow-service/function-fingerprint-selection/selected-components/1";

fn selection_schema(
    load_roles: Option<FunctionPackageLoadRoleRecipe>,
    neighborhood: Option<FunctionDirectNeighborhoodRecipe>,
    composite: Option<FunctionCompositeRecipe>,
) -> &'static str {
    if neighborhood.is_some() || composite.is_some() {
        SELECTED_COMPONENTS_SELECTION_SCHEMA
    } else if load_roles.is_some() {
        LOAD_ROLE_SELECTION_SCHEMA
    } else {
        SELECTION_SCHEMA
    }
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    direct_neighborhood: Option<FunctionDirectNeighborhoodRecipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    composite: Option<FunctionCompositeRecipe>,
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
    #[serde(default, deserialize_with = "non_null_neighborhood")]
    direct_neighborhood: Option<FunctionDirectNeighborhoodRecipe>,
    #[serde(default, deserialize_with = "non_null_composite")]
    composite: Option<FunctionCompositeRecipe>,
}

fn non_null_composite<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<FunctionCompositeRecipe>, D::Error> {
    FunctionCompositeRecipe::deserialize(deserializer).map(Some)
}

fn non_null_neighborhood<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<FunctionDirectNeighborhoodRecipe>, D::Error> {
    FunctionDirectNeighborhoodRecipe::deserialize(deserializer).map(Some)
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
            direct_neighborhood: None,
            composite: None,
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
        self.package_load_roles = Some(recipe);
        self.schema = selection_schema(
            self.package_load_roles,
            self.direct_neighborhood,
            self.composite,
        );
        super::preflight_cancellable(
            &self,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            &AtomicBool::new(false),
        )?;
        Ok(self)
    }

    pub fn with_direct_neighborhood(
        mut self,
        recipe: FunctionDirectNeighborhoodRecipe,
    ) -> ServiceResult<Self> {
        self.direct_neighborhood = Some(recipe);
        self.schema = selection_schema(
            self.package_load_roles,
            self.direct_neighborhood,
            self.composite,
        );
        super::preflight_cancellable(
            &self,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            &AtomicBool::new(false),
        )?;
        Ok(self)
    }

    /// Select only the composite. Its required components remain explicit selections.
    pub fn with_composite(mut self, recipe: FunctionCompositeRecipe) -> ServiceResult<Self> {
        self.composite = Some(recipe);
        self.schema = selection_schema(
            self.package_load_roles,
            self.direct_neighborhood,
            self.composite,
        );
        super::preflight_cancellable(
            &self,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            &AtomicBool::new(false),
        )?;
        Ok(self)
    }

    pub const fn composite(&self) -> Option<FunctionCompositeRecipe> {
        self.composite
    }

    pub const fn direct_neighborhood(&self) -> Option<FunctionDirectNeighborhoodRecipe> {
        self.direct_neighborhood
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
        let expected_schema = selection_schema(
            wire.package_load_roles,
            wire.direct_neighborhood,
            wire.composite,
        );
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
        if let Some(recipe) = wire.direct_neighborhood {
            selection = selection.with_direct_neighborhood(recipe)?;
        }
        if let Some(recipe) = wire.composite {
            selection = selection.with_composite(recipe)?;
        }
        checkpoint(stop)?;
        Ok(selection)
    }

    pub(super) fn build(
        &self,
        direct: &PlatformGraphProvenance<'_>,
        stop: &AtomicBool,
    ) -> ServiceResult<FunctionFingerprintManifest> {
        self.build_with_composition(direct, None, stop)
    }

    pub(super) fn build_composed(
        &self,
        direct: &PlatformGraphProvenance<'_>,
        composition: &FunctionNeighborhoodComposition<'_>,
        stop: &AtomicBool,
    ) -> ServiceResult<FunctionFingerprintManifest> {
        self.build_with_composition(direct, Some(composition), stop)
    }

    fn build_with_composition(
        &self,
        direct: &PlatformGraphProvenance<'_>,
        composition: Option<&FunctionNeighborhoodComposition<'_>>,
        stop: &AtomicBool,
    ) -> ServiceResult<FunctionFingerprintManifest> {
        checkpoint(stop)?;
        if self.direct_neighborhood.is_some() != composition.is_some() {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
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
        let result = if let Some(recipe) = self.composite {
            build_platform_function_fingerprints_with_composite(
                direct,
                &ids,
                limits,
                self.package_load_roles,
                self.direct_neighborhood.zip(composition),
                recipe,
                stop,
            )
        } else {
            match (
                self.direct_neighborhood,
                composition,
                self.package_load_roles,
            ) {
                (Some(recipe), Some(composition), load_roles) => {
                    build_platform_function_fingerprints_composed(
                        direct,
                        &ids,
                        limits,
                        load_roles,
                        recipe,
                        composition,
                        stop,
                    )
                }
                (None, None, None) => {
                    build_platform_function_fingerprints(direct, &ids, limits, stop)
                }
                (None, None, Some(recipe)) => build_platform_function_fingerprints_with_load_roles(
                    direct, &ids, limits, recipe, stop,
                ),
                _ => return Err(error(ServiceErrorCode::IdentityMismatch)),
            }
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
    fn selected_neighborhood_requires_exact_recipe_and_preserves_selector_order() -> TestResult {
        let plain = FunctionFingerprintSelection::new(Vec::new(), 1, 1, 4096)?;
        let old = serde_json::to_value(&plain)?;
        assert_eq!(old["schema"], SELECTION_SCHEMA);
        assert!(old.get("direct_neighborhood").is_none());
        let recipe = FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1;
        let selected = plain.clone().with_direct_neighborhood(recipe)?;
        let bytes = serde_json::to_vec(&selected)?;
        let restored = FunctionFingerprintSelection::from_json_slice(&bytes)?;
        assert_eq!(restored.direct_neighborhood(), Some(recipe));
        assert_eq!(restored.package_load_roles(), None);
        let wire = serde_json::to_value(&selected)?;
        assert_eq!(wire["schema"], SELECTED_COMPONENTS_SELECTION_SCHEMA);
        for (field, value) in [
            ("schema", serde_json::json!(SELECTION_SCHEMA)),
            ("direct_neighborhood", serde_json::Value::Null),
            ("direct_neighborhood", serde_json::json!("unsupported")),
        ] {
            let mut invalid = wire.clone();
            invalid[field] = value;
            assert_eq!(
                FunctionFingerprintSelection::from_json_slice(&serde_json::to_vec(&invalid)?)
                    .err()
                    .ok_or("invalid neighborhood selector admitted")?
                    .code(),
                ServiceErrorCode::InvalidRequest
            );
        }
        let role = FunctionPackageLoadRoleRecipe::SelectedPhysicalLuaV1;
        let role_first = plain
            .clone()
            .with_package_load_roles(role)?
            .with_direct_neighborhood(recipe)?;
        let neighborhood_first = plain
            .with_direct_neighborhood(recipe)?
            .with_package_load_roles(role)?;
        assert_eq!(
            serde_json::to_vec(&role_first)?,
            serde_json::to_vec(&neighborhood_first)?
        );
        let restored =
            FunctionFingerprintSelection::from_json_slice(&serde_json::to_vec(&role_first)?)?;
        assert_eq!(restored.direct_neighborhood(), Some(recipe));
        assert_eq!(restored.package_load_roles(), Some(role));
        Ok(())
    }

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

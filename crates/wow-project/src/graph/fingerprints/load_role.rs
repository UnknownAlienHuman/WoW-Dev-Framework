//! Reviewed static package/load roles for selected physical Main functions.
//! Exact native receipts bind the comparison; runtime execution is not inferred.
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize, Serializer};
use sha2::{Digest, Sha256};
use wow_core::{CanonicalResult, ContentDigest, DigestPurpose};

use super::super::producer_budget::ProducerBudget;
use super::{FunctionFingerprintManifest, PlatformGraphProvenance, invalid};
use crate::load::{
    LoadRecord, LoadRecordKind, LoadSelection, ProjectPackageLoadCoverage, ProjectPackageLoadPhase,
    ProjectPackageLoadPlan, ProjectPackageLoadUnit, ProjectPackageMainFile, ProjectPackageMainPlan,
    ProjectPackageNode, ProjectPackageReachability, ProjectPackageVariantReceipt,
    TocLoadOnDemandState,
};
use crate::{PlatformGraphProfile, ProjectResult};

pub const FUNCTION_PACKAGE_LOAD_ROLE_RECIPE: &str =
    "wow-project/physical-function-package-load-role/1";
const ROLE_DOMAIN: &[u8] =
    b"wow-project/structural-fingerprint/package-load-role/selected-physical-lua/1\0";
// Exhaustively matched native enums: five phases, three reachabilities, two bootstrap states.
const ROLE_GROUPS: usize = 5 * 3 * 2;

/// Explicit query recipe. It changes comparison output, never native owners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FunctionPackageLoadRoleRecipe {
    #[serde(rename = "wow-project/physical-function-package-load-role/1")]
    SelectedPhysicalLuaV1,
}
impl<'de> Deserialize<'de> for FunctionPackageLoadRoleRecipe {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let recipe = String::deserialize(deserializer)?;
        match recipe.as_str() {
            FUNCTION_PACKAGE_LOAD_ROLE_RECIPE => Ok(Self::SelectedPhysicalLuaV1),
            _ => Err(serde::de::Error::custom(
                "unsupported package/load-role recipe",
            )),
        }
    }
}

/// Preserve the original field bytes when this component was not requested.
#[derive(Debug)]
pub enum FunctionPackageLoadRoleSelection {
    NotSelected,
    Selected(Box<FunctionPackageLoadRole>),
}
impl Serialize for FunctionPackageLoadRoleSelection {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::NotSelected => serializer.serialize_str("not_selected"),
            Self::Selected(row) => row.serialize(serializer),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionPackageLoadRoleFingerprint;
impl DigestPurpose for FunctionPackageLoadRoleFingerprint {
    const NAME: &'static str = "function_package_load_role_fingerprint";
}

/// Completeness of this static component only; the containing manifest stays Partial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionPackageLoadRoleCoverage {
    Complete,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionPackageLoadRoleUnavailable {
    MissingPhysicalPackageFile,
    MissingLoadUnit,
    IncompletePackageClosure,
    UnknownLoadOnDemand,
    /// Conservative scope gate; original conflicts remain in the parent manifest.
    ScopeConflict,
}

/// One finite role group. Multiplicity counts admitted physical Lua load occurrences,
/// not functions, execution count, global ordinals, or client-loaded state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FunctionPackageLoadUnitRole {
    phase: ProjectPackageLoadPhase,
    reachability: ProjectPackageReachability,
    bootstrap: bool,
    occurrences: u64,
}
impl FunctionPackageLoadUnitRole {
    #[must_use]
    pub const fn phase(&self) -> ProjectPackageLoadPhase {
        self.phase
    }

    #[must_use]
    pub const fn reachability(&self) -> ProjectPackageReachability {
        self.reachability
    }

    #[must_use]
    pub const fn bootstrap(&self) -> bool {
        self.bootstrap
    }

    #[must_use]
    pub const fn occurrences(&self) -> u64 {
        self.occurrences
    }
}

/// Finite typed comparison data; no identity, source path, text or variant filename.
#[derive(Debug, Serialize)]
pub struct FunctionPackageLoadRoleComparable {
    selected_root: bool,
    load_on_demand: TocLoadOnDemandState,
    package_phase: ProjectPackageLoadPhase,
    package_reachability: ProjectPackageReachability,
    lua_load_roles: Vec<FunctionPackageLoadUnitRole>,
}
impl FunctionPackageLoadRoleComparable {
    #[must_use]
    pub const fn selected_root(&self) -> bool {
        self.selected_root
    }

    #[must_use]
    pub const fn load_on_demand(&self) -> TocLoadOnDemandState {
        self.load_on_demand
    }

    #[must_use]
    pub const fn package_phase(&self) -> ProjectPackageLoadPhase {
        self.package_phase
    }

    #[must_use]
    pub const fn package_reachability(&self) -> ProjectPackageReachability {
        self.package_reachability
    }

    #[must_use]
    pub fn lua_load_roles(&self) -> &[FunctionPackageLoadUnitRole] {
        &self.lua_load_roles
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FunctionPackageLoadRoleComponent {
    Comparable {
        digest: ContentDigest<FunctionPackageLoadRoleFingerprint>,
        value: FunctionPackageLoadRoleComparable,
    },
    Unavailable {
        reason: FunctionPackageLoadRoleUnavailable,
    },
}

/// Descriptive, Serialize-only native witness. These fields NEVER enter ROLE_DOMAIN.
/// The selected plan digest also binds its owner's explicit load context.
/// No TOC flavor is inferred from a filename.
#[derive(Debug, Serialize)]
pub struct FunctionPackageLoadRoleWitness {
    package_load_plan_digest: ContentDigest<CanonicalResult>,
    package_main_plan_digest: ContentDigest<CanonicalResult>,
    selected_plan_digest: ContentDigest<CanonicalResult>,
    selected_toc: ProjectPackageVariantReceipt,
    main_file: ProjectPackageMainFile,
    selected_root: bool,
    load_on_demand: TocLoadOnDemandState,
    package_phase: ProjectPackageLoadPhase,
    package_reachability: ProjectPackageReachability,
    load_units: Vec<ProjectPackageLoadUnit>,
}
impl FunctionPackageLoadRoleWitness {
    #[must_use]
    pub const fn package_load_plan_digest(&self) -> ContentDigest<CanonicalResult> {
        self.package_load_plan_digest
    }

    #[must_use]
    pub const fn package_main_plan_digest(&self) -> ContentDigest<CanonicalResult> {
        self.package_main_plan_digest
    }

    #[must_use]
    pub const fn selected_plan_digest(&self) -> ContentDigest<CanonicalResult> {
        self.selected_plan_digest
    }

    #[must_use]
    pub const fn selected_toc(&self) -> &ProjectPackageVariantReceipt {
        &self.selected_toc
    }

    #[must_use]
    pub const fn main_file(&self) -> &ProjectPackageMainFile {
        &self.main_file
    }

    #[must_use]
    pub fn load_units(&self) -> &[ProjectPackageLoadUnit] {
        &self.load_units
    }

    #[must_use]
    pub const fn selected_root(&self) -> bool {
        self.selected_root
    }

    #[must_use]
    pub const fn load_on_demand(&self) -> TocLoadOnDemandState {
        self.load_on_demand
    }

    #[must_use]
    pub const fn package_phase(&self) -> ProjectPackageLoadPhase {
        self.package_phase
    }

    #[must_use]
    pub const fn package_reachability(&self) -> ProjectPackageReachability {
        self.package_reachability
    }
}

/// Parent retains this alongside the unchanged exact function owner/support.
/// Decoding this receipt cannot construct native project or package authority.
#[derive(Debug, Serialize)]
pub struct FunctionPackageLoadRole {
    recipe: &'static str,
    coverage: FunctionPackageLoadRoleCoverage,
    native_load_coverage: ProjectPackageLoadCoverage,
    component: FunctionPackageLoadRoleComponent,
    #[serde(skip_serializing_if = "Option::is_none")]
    witness: Option<FunctionPackageLoadRoleWitness>,
}
impl FunctionPackageLoadRole {
    #[must_use]
    pub const fn recipe(&self) -> &'static str {
        self.recipe
    }

    #[must_use]
    pub const fn coverage(&self) -> FunctionPackageLoadRoleCoverage {
        self.coverage
    }

    #[must_use]
    pub const fn native_load_coverage(&self) -> ProjectPackageLoadCoverage {
        self.native_load_coverage
    }

    #[must_use]
    pub const fn component(&self) -> &FunctionPackageLoadRoleComponent {
        &self.component
    }

    #[must_use]
    pub fn witness(&self) -> Option<&FunctionPackageLoadRoleWitness> {
        self.witness.as_ref()
    }
}

#[derive(Clone, Copy)]
struct PackageJoin<'a> {
    node: &'a ProjectPackageNode,
    selected_toc: &'a ProjectPackageVariantReceipt,
}

struct LoadRoleIndex<'a> {
    packages: BTreeMap<&'a str, PackageJoin<'a>>,
    files: BTreeMap<&'a str, &'a ProjectPackageMainFile>,
    units: BTreeMap<(&'a str, &'a str), Vec<&'a ProjectPackageLoadUnit>>,
}

impl<'a> LoadRoleIndex<'a> {
    fn build(
        load: &'a ProjectPackageLoadPlan,
        main: &'a ProjectPackageMainPlan,
        retained: &mut ProducerBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        let mut packages = BTreeMap::new();
        let mut records = BTreeMap::<(&str, u64), &LoadRecord>::new();
        for node in load.packages() {
            crate::analyzer::checkpoint(stop)?;
            let plan = load.package_plan(&node.package).ok_or_else(invalid)?;
            if plan.digest() != node.selected_plan_digest
                || plan.selected_toc() != node.selected_toc
            {
                return Err(invalid());
            }
            let mut selected_toc = None;
            for variant in &node.variants {
                crate::analyzer::checkpoint(stop)?;
                if variant.selected
                    && (variant.package != node.package
                        || variant.toc != node.selected_toc
                        || selected_toc.replace(variant).is_some())
                {
                    return Err(invalid());
                }
            }
            let selected_toc = selected_toc.ok_or_else(invalid)?;
            retained.charge_serialized(
                &(
                    "package-load-role-package-index",
                    &node.package,
                    node.selected_plan_digest,
                    selected_toc,
                ),
                stop,
            )?;
            if packages
                .insert(node.package.as_str(), PackageJoin { node, selected_toc })
                .is_some()
            {
                return Err(invalid());
            }
            for record in plan.records() {
                crate::analyzer::checkpoint(stop)?;
                // Same physical-Lua branch of the native unit projection filter.
                if record.selection != LoadSelection::Included
                    || record.kind != LoadRecordKind::LuaFile
                    || record.target.is_none()
                {
                    continue;
                }
                retained.charge_serialized(
                    &("package-load-role-record-index", &node.package, record),
                    stop,
                )?;
                if records
                    .insert((node.package.as_str(), record.ordinal), record)
                    .is_some()
                {
                    return Err(invalid());
                }
            }
        }
        let mut files = BTreeMap::new();
        for file in main.files() {
            crate::analyzer::checkpoint(stop)?;
            retained.charge_serialized(&("package-load-role-file-index", file), stop)?;
            if !packages.contains_key(file.package.as_str())
                || files.insert(file.project_path.as_str(), file).is_some()
            {
                return Err(invalid());
            }
        }
        let mut units = BTreeMap::<(&str, &str), Vec<&ProjectPackageLoadUnit>>::new();
        for unit in load.units() {
            crate::analyzer::checkpoint(stop)?;
            if unit.kind != LoadRecordKind::LuaFile {
                continue;
            }
            let package = packages.get(unit.package.as_str()).ok_or_else(invalid)?;
            let record = records
                .get(&(unit.package.as_str(), unit.source_ordinal))
                .copied()
                .ok_or_else(invalid)?;
            let phase = if unit.bootstrap {
                ProjectPackageLoadPhase::Bootstrap
            } else {
                package.node.phase
            };
            if record.document != unit.document
                || record.target.as_deref() != Some(unit.target.as_str())
                || record.kind != unit.kind
                || record.bootstrap != unit.bootstrap
                || unit.reachability != package.node.reachability
                || unit.phase != phase
            {
                return Err(invalid());
            }
            retained.charge_serialized(&("package-load-role-unit-index", unit), stop)?;
            units
                .entry((unit.package.as_str(), unit.target.as_str()))
                .or_default()
                .push(unit);
        }
        crate::analyzer::checkpoint(stop)?;
        Ok(Self {
            packages,
            files,
            units,
        })
    }
}

#[derive(Serialize)]
struct BorrowedWitness<'a> {
    package_load_plan_digest: ContentDigest<CanonicalResult>,
    package_main_plan_digest: ContentDigest<CanonicalResult>,
    selected_plan_digest: ContentDigest<CanonicalResult>,
    selected_toc: &'a ProjectPackageVariantReceipt,
    main_file: &'a ProjectPackageMainFile,
    selected_root: bool,
    load_on_demand: TocLoadOnDemandState,
    package_phase: ProjectPackageLoadPhase,
    package_reachability: ProjectPackageReachability,
    load_units: &'a [&'a ProjectPackageLoadUnit],
}
impl BorrowedWitness<'_> {
    fn into_owned(self, stop: &AtomicBool) -> ProjectResult<FunctionPackageLoadRoleWitness> {
        crate::analyzer::checkpoint(stop)?;
        let selected_toc = self.selected_toc.clone();
        crate::analyzer::checkpoint(stop)?;
        let main_file = self.main_file.clone();
        let mut load_units = Vec::new();
        for unit in self.load_units {
            crate::analyzer::checkpoint(stop)?;
            load_units.push((*unit).clone());
        }
        crate::analyzer::checkpoint(stop)?;
        Ok(FunctionPackageLoadRoleWitness {
            package_load_plan_digest: self.package_load_plan_digest,
            package_main_plan_digest: self.package_main_plan_digest,
            selected_plan_digest: self.selected_plan_digest,
            selected_toc,
            main_file,
            selected_root: self.selected_root,
            load_on_demand: self.load_on_demand,
            package_phase: self.package_phase,
            package_reachability: self.package_reachability,
            load_units,
        })
    }
}

#[derive(Serialize)]
struct BorrowedRole<'a> {
    recipe: &'static str,
    coverage: FunctionPackageLoadRoleCoverage,
    native_load_coverage: ProjectPackageLoadCoverage,
    component: &'a FunctionPackageLoadRoleComponent,
    #[serde(skip_serializing_if = "Option::is_none")]
    witness: Option<&'a BorrowedWitness<'a>>,
}

/// Parent-only selected integration. The sealed manifest already proves exact
/// physical-function entities and support; do not replace those original receipts.
/// Rows preserve its strict selection order. Use the parent's SAME live budget,
/// retain its Partial/conflicts, and include the enlarged final envelope in its cap.
pub(super) fn build_package_load_role_components(
    direct: &PlatformGraphProvenance<'_>,
    manifest: &FunctionFingerprintManifest,
    retained: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Vec<FunctionPackageLoadRole>> {
    crate::analyzer::checkpoint(stop)?;
    let project = direct.project();
    let configuration = project.configuration();
    let Some(PlatformGraphProfile::DirectCapabilitiesV1(selected)) =
        configuration.platform_graph_profile()
    else {
        return Err(invalid());
    };
    let owner = configuration.platform_packages().ok_or_else(invalid)?;
    let report = direct.source().function_call_report().ok_or_else(invalid)?;
    let binding = manifest.binding();
    if direct.capabilities() != Some(selected)
        || retained.limit() != manifest.limits().max_metadata_bytes()
        || binding.project_snapshot_id() != project.snapshot_id()
        || binding.project_generation() != project.project_generation()
        || binding.configuration_digest() != configuration.configuration_digest()
        || binding.graph_scope() != direct.scope()
        || binding.published_graph_snapshot_id() != direct.graph().snapshot().snapshot_id()
        || binding.published_graph_generation() != direct.graph().snapshot().generation()
        || binding.package_binding_digest != owner.binding().binding_digest()
        || binding.function_analysis_id() != report.analysis_id()
    {
        return Err(invalid());
    }
    let load = owner.load_plan();
    let main = owner.main_plan();
    if configuration.package_load_plan_digest() != Some(load.digest())
        || configuration.package_main_plan_digest() != Some(main.digest())
        || main.package_load_plan_digest() != load.digest()
    {
        return Err(invalid());
    }
    for pair in manifest.records().windows(2) {
        crate::analyzer::checkpoint(stop)?;
        if pair[0].function_id() >= pair[1].function_id() {
            return Err(invalid());
        }
    }
    if manifest.records().is_empty() {
        return Ok(Vec::new());
    }
    let index = LoadRoleIndex::build(load, main, retained, stop)?;
    let mut rows = Vec::new();
    for record in manifest.records() {
        crate::analyzer::checkpoint(stop)?;
        if record.exact_owner().scope() != direct.scope() {
            return Err(invalid());
        }
        // The private parent already proved this selected physical function.
        // Retain its original support; only native Main/load joins follow here.
        let witness = match index.files.get(record.support().path()).copied() {
            None => None,
            Some(file) => {
                if file.content_digest != record.support.file_content_digest {
                    return Err(invalid());
                }
                let package = index
                    .packages
                    .get(file.package.as_str())
                    .ok_or_else(invalid)?;
                let units = index
                    .units
                    .get(&(file.package.as_str(), file.source_path.as_str()))
                    .map_or(&[][..], Vec::as_slice);
                Some(BorrowedWitness {
                    package_load_plan_digest: load.digest(),
                    package_main_plan_digest: main.digest(),
                    selected_plan_digest: package.node.selected_plan_digest,
                    selected_toc: package.selected_toc,
                    main_file: file,
                    selected_root: package.node.selected_root,
                    load_on_demand: package.node.load_on_demand,
                    package_phase: package.node.phase,
                    package_reachability: package.node.reachability,
                    load_units: units,
                })
            }
        };
        if let Some(witness) = &witness {
            // Input accounting before projecting, hashing or copying selected evidence.
            retained.charge_serialized(&("package-load-role-native-input", witness), stop)?;
        }
        use FunctionPackageLoadRoleUnavailable as Missing;
        let unavailable = if !manifest.scope_conflicts().is_empty() {
            Some(Missing::ScopeConflict)
        } else if !load.coverage().complete() {
            // No fabricated per-package coverage: this is a conservative universe gate.
            Some(Missing::IncompletePackageClosure)
        } else if let Some(witness) = &witness {
            if witness.load_on_demand == TocLoadOnDemandState::Unknown {
                Some(Missing::UnknownLoadOnDemand)
            } else if witness.load_units.is_empty() {
                Some(Missing::MissingLoadUnit)
            } else {
                None
            }
        } else {
            Some(Missing::MissingPhysicalPackageFile)
        };
        let (coverage, component) = match unavailable {
            Some(reason) => (
                FunctionPackageLoadRoleCoverage::Partial,
                FunctionPackageLoadRoleComponent::Unavailable { reason },
            ),
            None => {
                let witness = witness.as_ref().ok_or_else(invalid)?;
                let value = comparable(witness, stop)?;
                retained.charge_serialized(&("package-load-role-comparable", &value), stop)?;
                let digest = role_digest(&value, stop)?;
                (
                    FunctionPackageLoadRoleCoverage::Complete,
                    FunctionPackageLoadRoleComponent::Comparable { digest, value },
                )
            }
        };
        let borrowed = BorrowedRole {
            recipe: FUNCTION_PACKAGE_LOAD_ROLE_RECIPE,
            coverage,
            native_load_coverage: load.coverage(),
            component: &component,
            witness: witness.as_ref(),
        };
        // Exact escaped output row charged before all owned native/path/ID copies.
        retained.charge_serialized(&borrowed, stop)?;
        let witness = witness.map(|value| value.into_owned(stop)).transpose()?;
        crate::analyzer::checkpoint(stop)?;
        rows.push(FunctionPackageLoadRole {
            recipe: FUNCTION_PACKAGE_LOAD_ROLE_RECIPE,
            coverage,
            native_load_coverage: load.coverage(),
            component,
            witness,
        });
    }
    // Rows move directly into the parent's records; its real final manifest,
    // not a temporary row-array envelope, is charged to this same ledger.
    crate::analyzer::checkpoint(stop)?;
    Ok(rows)
}

fn comparable(
    witness: &BorrowedWitness<'_>,
    stop: &AtomicBool,
) -> ProjectResult<FunctionPackageLoadRoleComparable> {
    let mut counts = [0_u64; ROLE_GROUPS];
    for unit in witness.load_units {
        crate::analyzer::checkpoint(stop)?;
        let slot = usize::from(phase_tag(unit.phase)) * 6
            + usize::from(reachability_tag(unit.reachability)) * 2
            + usize::from(unit.bootstrap);
        counts[slot] = counts[slot].checked_add(1).ok_or_else(invalid)?;
    }
    let mut lua_load_roles = Vec::new();
    for phase in [
        ProjectPackageLoadPhase::DependencyPrerequisite,
        ProjectPackageLoadPhase::Bootstrap,
        ProjectPackageLoadPhase::Normal,
        ProjectPackageLoadPhase::OptionalDependencyConditional,
        ProjectPackageLoadPhase::Deferred,
    ] {
        for reachability in [
            ProjectPackageReachability::Unreachable,
            ProjectPackageReachability::ConditionallyReachable,
            ProjectPackageReachability::Reachable,
        ] {
            for bootstrap in [false, true] {
                crate::analyzer::checkpoint(stop)?;
                let slot = usize::from(phase_tag(phase)) * 6
                    + usize::from(reachability_tag(reachability)) * 2
                    + usize::from(bootstrap);
                let occurrences = counts[slot];
                if occurrences != 0 {
                    lua_load_roles.push(FunctionPackageLoadUnitRole {
                        phase,
                        reachability,
                        bootstrap,
                        occurrences,
                    });
                }
            }
        }
    }
    Ok(FunctionPackageLoadRoleComparable {
        selected_root: witness.selected_root,
        load_on_demand: witness.load_on_demand,
        package_phase: witness.package_phase,
        package_reachability: witness.package_reachability,
        lua_load_roles,
    })
}

/// Fixed typed bytes: domain including NUL; root:u8, LOD:u8, package_phase:u8,
/// package_reachability:u8, role_group_count:u64 BE; then each nonzero group in
/// phase/reachability/bootstrap tag order: phase:u8, reachability:u8, bootstrap:u8,
/// occurrences:u64 BE. No other field enters this digest.
fn role_digest(
    value: &FunctionPackageLoadRoleComparable,
    stop: &AtomicBool,
) -> ProjectResult<ContentDigest<FunctionPackageLoadRoleFingerprint>> {
    crate::analyzer::checkpoint(stop)?;
    let lod = match value.load_on_demand {
        TocLoadOnDemandState::NotDeclared => 0_u8,
        TocLoadOnDemandState::False => 1_u8,
        TocLoadOnDemandState::True => 2_u8,
        TocLoadOnDemandState::Unknown => return Err(invalid()),
    };
    let mut hash = Sha256::new();
    hash.update(ROLE_DOMAIN);
    hash.update([
        u8::from(value.selected_root),
        lod,
        phase_tag(value.package_phase),
        reachability_tag(value.package_reachability),
    ]);
    let count = u64::try_from(value.lua_load_roles.len()).map_err(|_| invalid())?;
    hash.update(count.to_be_bytes());
    for role in &value.lua_load_roles {
        crate::analyzer::checkpoint(stop)?;
        hash.update([
            phase_tag(role.phase),
            reachability_tag(role.reachability),
            u8::from(role.bootstrap),
        ]);
        hash.update(role.occurrences.to_be_bytes());
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(ContentDigest::from_bytes(hash.finalize().into()))
}

const fn phase_tag(phase: ProjectPackageLoadPhase) -> u8 {
    match phase {
        ProjectPackageLoadPhase::DependencyPrerequisite => 0,
        ProjectPackageLoadPhase::Bootstrap => 1,
        ProjectPackageLoadPhase::Normal => 2,
        ProjectPackageLoadPhase::OptionalDependencyConditional => 3,
        ProjectPackageLoadPhase::Deferred => 4,
    }
}

const fn reachability_tag(reachability: ProjectPackageReachability) -> u8 {
    match reachability {
        ProjectPackageReachability::Unreachable => 0,
        ProjectPackageReachability::ConditionallyReachable => 1,
        ProjectPackageReachability::Reachable => 2,
    }
}

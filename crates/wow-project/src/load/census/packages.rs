//! Measurement-only use of the native package load/dependency owner. The
//! expanded plan never escapes as a Main, graph, publication or replay input.
use super::{CensusCoverage, measure};
use crate::{
    ProjectPhase, ProjectResult,
    disk::checkpoint,
    load::{
        LoadIssue, LoadSelection, ProjectPackageInput, ProjectPackageLoadCoverage,
        ProjectPackageLoadIssue, ProjectPackageOrderGroup, ProjectPackageReachability,
        TocDependencyDeclaration, TocDependencyKind, TocDependencyResolution, TocLoadContext,
        package_capacity::PackageLoadCapacity,
        package_closure::read_admitted_packages_with_capacity,
    },
    platform_source::AdmittedPlatformSource,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};
use wow_core::{CanonicalResult, ContentDigest, CoverageStatus};

pub const PLATFORM_PACKAGE_CENSUS_PROFILE: &str = "wow-project/platform-package-census/1";

/// Sources are package-qualified retained occurrences, including selected TOCs.
/// Expanded records include repeated includes; native units can retain a missing
/// target, so their count does not prove that source was acquired or executed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PackageLoadCounts {
    pub selected_sources: u64,
    pub selected_source_bytes: u64,
    pub unselected_variants: u64,
    pub unselected_variant_bytes: u64,
    pub main_lua_files: u64,
    pub expanded_records: u64,
    pub excluded_records: u64,
    pub unresolved_records: u64,
    pub load_units: u64,
    pub units_without_retained_target: u64,
    pub load_issues: u64,
    pub xml_documents: u64,
    pub xml_elements: u64,
    pub xml_script_sites: u64,
    pub xml_inline_units: u64,
    pub xml_map_segments: u64,
    pub native_plan_json_bytes: u64,
}
impl PackageLoadCounts {
    fn add(&mut self, other: &Self) -> ProjectResult<()> {
        macro_rules! fields { ($($field:ident),+ $(,)?) => { $(measure::add(&mut self.$field, other.$field)?;)+ }; }
        fields!(
            selected_sources,
            selected_source_bytes,
            unselected_variants,
            unselected_variant_bytes,
            main_lua_files,
            expanded_records,
            excluded_records,
            unresolved_records,
            load_units,
            units_without_retained_target,
            load_issues,
            xml_documents,
            xml_elements,
            xml_script_sites,
            xml_inline_units,
            xml_map_segments,
            native_plan_json_bytes
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PackageDependencyCounts {
    pub declarations: u64,
    pub required: u64,
    pub optional: u64,
    pub resolved: u64,
    pub missing: u64,
    pub condition_unresolved: u64,
    pub invalid_name: u64,
    pub excluded: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackageLoadMeasurement {
    pub package: String,
    pub root: String,
    pub selected_toc: String,
    pub selected_root: bool,
    pub reachability: ProjectPackageReachability,
    pub native_load_plan_digest: ContentDigest<CanonicalResult>,
    pub counts: PackageLoadCounts,
    pub load_issues: Vec<LoadIssue>,
}

/// Serialize-only exact measurement receipt. No source bodies, runtime claims,
/// graph capability or persisted package input can be recovered from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlatformPackageLoadCensus {
    profile: &'static str,
    source_snapshot_id: String,
    source_profile_digest: ContentDigest<CanonicalResult>,
    admission_digest: ContentDigest<CanonicalResult>,
    content_manifest_digest: ContentDigest<CanonicalResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_context: Option<TocLoadContext>,
    selected_toc_capacity: usize,
    native_package_plan_digest: ContentDigest<CanonicalResult>,
    native_package_plan_json_bytes: u64,
    native_coverage: ProjectPackageLoadCoverage,
    coverage: CensusCoverage,
    packages: Vec<PackageLoadMeasurement>,
    loads: PackageLoadCounts,
    dependencies: PackageDependencyCounts,
    dependency_declarations: Vec<TocDependencyDeclaration>,
    native_package_issues: Vec<ProjectPackageLoadIssue>,
    order_groups: Vec<ProjectPackageOrderGroup>,
    cyclic_groups: u64,
    package_issues: u64,
    blocking_package_issues: u64,
    digest: ContentDigest<CanonicalResult>,
}
impl PlatformPackageLoadCensus {
    pub const fn coverage(&self) -> CensusCoverage {
        self.coverage
    }
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
    pub const fn loads(&self) -> &PackageLoadCounts {
        &self.loads
    }
    pub const fn dependencies(&self) -> &PackageDependencyCounts {
        &self.dependencies
    }
    pub fn packages(&self) -> &[PackageLoadMeasurement] {
        &self.packages
    }
    pub fn order_groups(&self) -> &[ProjectPackageOrderGroup] {
        &self.order_groups
    }
}

/// Measure one explicit package universe against retained admission bytes.
/// Invalid declarations, acquisition refusals and budgets abort with the native
/// error; cancellation never returns a partial sum. Missing load targets and
/// ambiguous dependencies retained by the loader preserve its Partial coverage.
pub fn census_platform_packages(
    source: &AdmittedPlatformSource,
    packages: &[ProjectPackageInput],
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<PlatformPackageLoadCensus> {
    checkpoint(stop)?;
    let capacity = PackageLoadCapacity::census(source)?;
    let pinned = source.pin_packages(packages, context, capacity, stop)?;
    let input = read_admitted_packages_with_capacity(
        source,
        &pinned,
        &source.profile().target().reference_profile,
        context,
        stop,
        capacity,
    )?;
    let (loaded, plan) = input.into_parts();
    let mut sources_by_package = BTreeMap::new();
    for package in &loaded {
        checkpoint(stop)?;
        sources_by_package.insert(
            package.package(),
            package
                .plan()
                .sources()
                .iter()
                .map(|source| source.path.as_str())
                .collect::<BTreeSet<_>>(),
        );
    }
    let mut units_by_package = BTreeMap::<&str, (u64, u64)>::new();
    for unit in plan.units() {
        checkpoint(stop)?;
        let counts = units_by_package.entry(&unit.package).or_default();
        measure::add(&mut counts.0, 1)?;
        if !sources_by_package
            .get(unit.package.as_str())
            .is_some_and(|sources| sources.contains(unit.target.as_str()))
        {
            measure::add(&mut counts.1, 1)?;
        }
    }
    // Counts borrow names from the native plan, not the loaded packages.
    drop(sources_by_package);
    let mut measurements = Vec::with_capacity(loaded.len());
    let mut loads = PackageLoadCounts::default();
    for package in loaded {
        checkpoint(stop)?;
        let node = plan
            .packages()
            .iter()
            .find(|node| node.package == package.package())
            .ok_or_else(|| super::invalid("census native package node is missing"))?;
        let declaration = pinned
            .iter()
            .find(|item| item.name() == package.package())
            .ok_or_else(|| super::invalid("census package declaration is missing"))?;
        let mut counts = PackageLoadCounts {
            main_lua_files: package.files().len() as u64,
            expanded_records: package.plan().records().len() as u64,
            load_issues: package.plan().issues().len() as u64,
            native_plan_json_bytes: measure::encoded_size(package.plan(), stop)?,
            ..PackageLoadCounts::default()
        };
        for source in package.plan().sources() {
            checkpoint(stop)?;
            measure::add(&mut counts.selected_sources, 1)?;
            measure::add(&mut counts.selected_source_bytes, source.byte_length)?;
        }
        for variant in node.variants.iter().filter(|variant| !variant.selected) {
            checkpoint(stop)?;
            measure::add(&mut counts.unselected_variants, 1)?;
            measure::add(&mut counts.unselected_variant_bytes, variant.byte_length)?;
        }
        for record in package.plan().records() {
            checkpoint(stop)?;
            match record.selection {
                LoadSelection::Excluded => measure::add(&mut counts.excluded_records, 1)?,
                LoadSelection::Unresolved => measure::add(&mut counts.unresolved_records, 1)?,
                LoadSelection::Included => {}
            }
        }
        let units = units_by_package
            .get(package.package())
            .copied()
            .unwrap_or_default();
        counts.load_units = units.0;
        counts.units_without_retained_target = units.1;
        for index in package.plan().xml_documents().values() {
            checkpoint(stop)?;
            measure::add(&mut counts.xml_documents, 1)?;
            measure::add(&mut counts.xml_elements, index.elements().len() as u64)?;
            measure::add(
                &mut counts.xml_map_segments,
                index.map_segment_count() as u64,
            )?;
            for element in index.elements() {
                checkpoint(stop)?;
                if let Some(script) = &element.script {
                    measure::add(&mut counts.xml_script_sites, 1)?;
                    if script.inline_lua.is_some() {
                        measure::add(&mut counts.xml_inline_units, 1)?;
                    }
                }
            }
        }
        loads.add(&counts)?;
        measurements.push(PackageLoadMeasurement {
            package: package.package().to_owned(),
            root: declaration.root().to_owned(),
            selected_toc: node.selected_toc.clone(),
            selected_root: node.selected_root,
            reachability: node.reachability,
            native_load_plan_digest: package.plan().digest(),
            counts,
            load_issues: package.plan().issues().to_vec(),
        });
    }
    let mut dependencies = PackageDependencyCounts::default();
    for dependency in plan.dependencies() {
        checkpoint(stop)?;
        measure::add(&mut dependencies.declarations, 1)?;
        measure::add(
            match dependency.kind {
                TocDependencyKind::Required => &mut dependencies.required,
                TocDependencyKind::Optional => &mut dependencies.optional,
            },
            1,
        )?;
        measure::add(
            match dependency.resolution {
                TocDependencyResolution::Resolved => &mut dependencies.resolved,
                TocDependencyResolution::Missing => &mut dependencies.missing,
                TocDependencyResolution::ConditionUnresolved => {
                    &mut dependencies.condition_unresolved
                }
                TocDependencyResolution::InvalidName => &mut dependencies.invalid_name,
                TocDependencyResolution::Excluded => &mut dependencies.excluded,
            },
            1,
        )?;
    }
    let receipt = source.receipt();
    let mut result = PlatformPackageLoadCensus {
        profile: PLATFORM_PACKAGE_CENSUS_PROFILE,
        source_snapshot_id: receipt.source_snapshot_id().to_owned(),
        source_profile_digest: receipt.profile_digest(),
        admission_digest: receipt.admission_digest(),
        content_manifest_digest: receipt.content_manifest_digest(),
        load_context: context.cloned(),
        selected_toc_capacity: capacity.packages(),
        native_package_plan_digest: plan.digest(),
        native_package_plan_json_bytes: measure::encoded_size(&plan, stop)?,
        native_coverage: plan.coverage(),
        coverage: if plan.coverage().complete()
            && receipt.coverage().inventory() == CoverageStatus::Complete
        {
            CensusCoverage::DeclaredMembersMeasured
        } else {
            CensusCoverage::Partial
        },
        packages: measurements,
        loads,
        dependencies,
        dependency_declarations: plan.dependencies().to_vec(),
        native_package_issues: plan.issues().to_vec(),
        order_groups: plan.order_groups().to_vec(),
        cyclic_groups: plan
            .order_groups()
            .iter()
            .filter(|group| group.cyclic)
            .count() as u64,
        package_issues: plan.issues().len() as u64,
        blocking_package_issues: plan
            .issues()
            .iter()
            .filter(|issue| issue.blocks_complete)
            .count() as u64,
        digest: ContentDigest::from_bytes([0; 32]),
    };
    measure::encoded_size(&result, stop)?;
    result.digest = crate::identity::canonical_digest(
        PLATFORM_PACKAGE_CENSUS_PROFILE,
        &result,
        ProjectPhase::Inventory,
    )?;
    checkpoint(stop)?;
    Ok(result)
}

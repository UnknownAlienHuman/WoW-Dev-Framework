//! Measurement-only use of the native package load/dependency owner. The
//! expanded plan never escapes as a Main, graph, publication or replay input.
use super::{CensusCoverage, measure};
use crate::{
    ProjectPhase, ProjectResult,
    disk::checkpoint,
    load::{
        LoadIssue, LoadRecordKind, LoadSelection, ProjectPackageInput, ProjectPackageLoadCoverage,
        ProjectPackageLoadIssue, ProjectPackageOrderGroup, ProjectPackageReachability,
        TocDependencyDeclaration, TocDependencyKind, TocDependencyResolution, TocLoadContext,
        package_capacity::PackageLoadCapacity, package_closure::stream_admitted_package_census,
        xml_references::XmlReferenceKind,
    },
    platform_source::AdmittedPlatformSource,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};
use wow_core::{CanonicalResult, ContentDigest, CoverageStatus};

pub const PLATFORM_PACKAGE_CENSUS_PROFILE: &str = "wow-project/platform-package-census/2";

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
    /// All native declaration entries, independent of validity or resolution.
    pub xml_reference_declarations: u64,
    /// All native Inherits records, including unresolved and cyclic references.
    pub xml_inheritance_references: u64,
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
            xml_reference_declarations,
            xml_inheritance_references,
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
    let mut measured = Vec::with_capacity(pinned.len());
    let mut loads = PackageLoadCounts::default();
    let plan = stream_admitted_package_census(
        source,
        &pinned,
        context,
        stop,
        |package, variants, plan_bytes, remaining| {
            let declaration = pinned
                .iter()
                .find(|item| item.name() == package.package())
                .ok_or_else(|| super::invalid("census package declaration is missing"))?;
            let observation = measure_package(package, declaration, variants, plan_bytes, stop)?;
            let encoded = measure::encoded_size(&observation, stop)?;
            if encoded > remaining {
                return Err(crate::load::budget_at(
                    "package census retained measurement encoding limit exceeded",
                ));
            }
            loads.add(&observation.counts)?;
            measured.push(observation);
            Ok(encoded)
        },
    )?;
    let mut native_units = BTreeMap::<&str, u64>::new();
    for unit in &plan.units {
        checkpoint(stop)?;
        measure::add(native_units.entry(unit.package.as_str()).or_default(), 1)?;
    }
    let mut measurements = Vec::with_capacity(measured.len());
    for observation in measured {
        checkpoint(stop)?;
        let node = plan
            .packages
            .iter()
            .find(|node| node.package == observation.package)
            .ok_or_else(|| super::invalid("census native package node is missing"))?;
        if node.selected_plan_digest != observation.native_load_plan_digest
            || node.selected_toc != observation.selected_toc
            || native_units
                .get(node.package.as_str())
                .copied()
                .unwrap_or_default()
                != observation.counts.load_units
        {
            return Err(super::invalid(
                "census native package projection differs from observation",
            ));
        }
        measurements.push(PackageLoadMeasurement {
            package: observation.package,
            root: observation.root,
            selected_toc: node.selected_toc.clone(),
            selected_root: node.selected_root,
            reachability: node.reachability,
            native_load_plan_digest: observation.native_load_plan_digest,
            counts: observation.counts,
            load_issues: observation.load_issues,
        });
    }
    let mut dependencies = PackageDependencyCounts::default();
    for dependency in &plan.dependencies {
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
        native_package_plan_digest: plan.digest,
        native_package_plan_json_bytes: measure::encoded_size(&plan, stop)?,
        native_coverage: plan.coverage,
        coverage: if plan.coverage.complete()
            && receipt.coverage().inventory() == CoverageStatus::Complete
        {
            CensusCoverage::DeclaredMembersMeasured
        } else {
            CensusCoverage::Partial
        },
        packages: measurements,
        loads,
        dependencies,
        dependency_declarations: plan.dependencies.clone(),
        native_package_issues: plan.issues.clone(),
        order_groups: plan.order_groups.clone(),
        cyclic_groups: plan
            .order_groups
            .iter()
            .filter(|group| group.cyclic)
            .count() as u64,
        package_issues: plan.issues.len() as u64,
        blocking_package_issues: plan
            .issues
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

/// Exact measured data retained after one package's syntax has been dropped.
/// Universe reachability is added only after the real native builder returns.
#[derive(Serialize)]
struct MeasuredPackage {
    package: String,
    root: String,
    selected_toc: String,
    native_load_plan_digest: ContentDigest<CanonicalResult>,
    counts: PackageLoadCounts,
    load_issues: Vec<LoadIssue>,
}

fn measure_package(
    package: &crate::load::ProjectLoadedPackage,
    declaration: &ProjectPackageInput,
    variants: &[crate::load::ProjectPackageVariantReceipt],
    plan_bytes: u64,
    stop: &AtomicBool,
) -> ProjectResult<MeasuredPackage> {
    checkpoint(stop)?;
    let plan = package.plan();
    let mut counts = PackageLoadCounts {
        main_lua_files: package.files().len() as u64,
        expanded_records: plan.records().len() as u64,
        load_issues: plan.issues().len() as u64,
        native_plan_json_bytes: plan_bytes,
        ..PackageLoadCounts::default()
    };
    let references = plan.xml_references();
    counts.xml_reference_declarations =
        u64::try_from(references.declarations().len()).map_err(|_| crate::load::budget())?;
    for reference in references.references() {
        checkpoint(stop)?;
        if reference.kind == XmlReferenceKind::Inherits {
            measure::add(&mut counts.xml_inheritance_references, 1)?;
        }
    }
    let retained = plan
        .sources()
        .iter()
        .map(|source| source.path.as_str())
        .collect::<BTreeSet<_>>();
    for source in plan.sources() {
        checkpoint(stop)?;
        measure::add(&mut counts.selected_sources, 1)?;
        measure::add(&mut counts.selected_source_bytes, source.byte_length)?;
    }
    for variant in variants.iter().filter(|variant| !variant.selected) {
        checkpoint(stop)?;
        measure::add(&mut counts.unselected_variants, 1)?;
        measure::add(&mut counts.unselected_variant_bytes, variant.byte_length)?;
    }
    for record in plan.records() {
        checkpoint(stop)?;
        match record.selection {
            LoadSelection::Excluded => measure::add(&mut counts.excluded_records, 1)?,
            LoadSelection::Unresolved => measure::add(&mut counts.unresolved_records, 1)?,
            LoadSelection::Included => {}
        }
        if record.selection == LoadSelection::Included
            && matches!(
                record.kind,
                LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
            )
            && let Some(target) = &record.target
        {
            measure::add(&mut counts.load_units, 1)?;
            if !retained.contains(target.as_str()) {
                measure::add(&mut counts.units_without_retained_target, 1)?;
            }
        }
    }
    for index in plan.xml_documents().values() {
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
    Ok(MeasuredPackage {
        package: package.package().to_owned(),
        root: declaration.root().to_owned(),
        selected_toc: plan.selected_toc().to_owned(),
        native_load_plan_digest: plan.digest(),
        counts,
        load_issues: plan.issues().to_vec(),
    })
}

//! Native package records derived from normalized load projections. These
//! records do not retain source bodies or grant Main/graph/replay capability.
use super::*;
use crate::load::census;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PackagePlanRecords {
    pub(crate) profile: &'static str,
    pub(crate) target_flavor: String,
    pub(crate) target_interface: u64,
    pub(crate) target_profile_digest: ContentDigest<CanonicalResult>,
    pub(crate) packages: Vec<ProjectPackageNode>,
    pub(crate) dependencies: Vec<TocDependencyDeclaration>,
    pub(crate) order_groups: Vec<ProjectPackageOrderGroup>,
    pub(crate) units: Vec<ProjectPackageLoadUnit>,
    pub(crate) issues: Vec<ProjectPackageLoadIssue>,
    pub(crate) coverage: ProjectPackageLoadCoverage,
    pub(crate) digest: ContentDigest<CanonicalResult>,
}

/// One real selected load receipt projected before its syntax can be dropped.
#[derive(Serialize)]
pub(crate) struct PackageProjection {
    pub(super) package: String,
    pub(super) selected_toc: String,
    pub(super) plan_digest: ContentDigest<CanonicalResult>,
    pub(super) variants: Vec<ProjectPackageVariantReceipt>,
    pub(super) metadata: MetadataProjection,
    pub(super) file_closure_complete: bool,
    pub(super) units: Vec<PackageUnitDraft>,
}

pub(crate) fn project_package(
    package: &str,
    plan: &ProjectLoadPlan,
    variants: &[ProjectPackageVariantReceipt],
    stop: &AtomicBool,
) -> ProjectResult<PackageProjection> {
    checkpoint(stop)?;
    Ok(PackageProjection {
        package: package.to_owned(),
        selected_toc: plan.selected_toc().to_owned(),
        plan_digest: plan.digest(),
        variants: variants.to_vec(),
        metadata: project_metadata(package, plan)?,
        file_closure_complete: local_file_closure_complete(plan),
        units: project_units(package, plan, stop)?,
    })
}
pub(crate) fn build_records(
    declarations: &[ProjectPackageInput],
    projected: &[PackageProjection],
    profile: &ProfileIdentity,
    stop: &AtomicBool,
    capacity: PackageLoadCapacity,
) -> ProjectResult<PackagePlanRecords> {
    let declaration_by_name = declarations
        .iter()
        .map(|package| (package.name.clone(), package))
        .collect::<BTreeMap<_, _>>();
    let canonical_by_folded = declarations
        .iter()
        .map(|package| (package.name.to_ascii_lowercase(), package.name.clone()))
        .collect::<BTreeMap<_, _>>();
    let plan_by_name = projected
        .iter()
        .map(|package| (package.package.clone(), package))
        .collect::<BTreeMap<_, _>>();

    let mut dependencies = Vec::new();
    let mut load_on_demand = BTreeMap::new();
    let mut issues = Vec::new();
    let mut duplicate_edges = BTreeSet::new();
    for (package, plan) in &plan_by_name {
        checkpoint(stop)?;
        let projection = &plan.metadata;
        load_on_demand.insert(package.clone(), projection.load_on_demand);
        if projection.conflicting_load_on_demand {
            issues.push(issue(
                ProjectPackageLoadIssueKind::ConflictingLoadOnDemand,
                package,
                None,
                None,
                true,
            ));
        }
        for mut dependency in projection.dependencies.iter().cloned() {
            checkpoint(stop)?;
            if capacity.is_census() && dependencies.len() >= MAX_PACKAGES * MAX_DEPENDENCIES {
                return Err(budget());
            }
            let active = dependency.selection == LoadSelection::Included;
            if dependency.selection == LoadSelection::Unresolved {
                issues.push(issue_from_dependency(
                    ProjectPackageLoadIssueKind::DependencyConditionUnresolved,
                    &dependency,
                    dependency.kind == TocDependencyKind::Required,
                ));
                dependency.resolution = TocDependencyResolution::ConditionUnresolved;
            } else if dependency.selection == LoadSelection::Excluded {
                dependency.resolution = TocDependencyResolution::Excluded;
            } else if !valid_package_name(&dependency.dependency) {
                issues.push(issue_from_dependency(
                    ProjectPackageLoadIssueKind::InvalidDependencyName,
                    &dependency,
                    dependency.kind == TocDependencyKind::Required,
                ));
                dependency.resolution = TocDependencyResolution::InvalidName;
            } else if active {
                let folded = dependency.dependency.to_ascii_lowercase();
                if let Some(resolved) = canonical_by_folded.get(&folded) {
                    dependency.resolution = TocDependencyResolution::Resolved;
                    dependency.resolved_package = Some(resolved.clone());
                    let edge = (package.clone(), dependency.kind, resolved.clone());
                    if !duplicate_edges.insert(edge) {
                        issues.push(issue_from_dependency(
                            ProjectPackageLoadIssueKind::DuplicateDependency,
                            &dependency,
                            false,
                        ));
                    }
                    if package == resolved {
                        issues.push(issue_from_dependency(
                            ProjectPackageLoadIssueKind::SelfDependency,
                            &dependency,
                            true,
                        ));
                    }
                } else {
                    dependency.resolution = TocDependencyResolution::Missing;
                    issues.push(issue_from_dependency(
                        if dependency.kind == TocDependencyKind::Required {
                            ProjectPackageLoadIssueKind::MissingRequiredDependency
                        } else {
                            ProjectPackageLoadIssueKind::MissingOptionalDependency
                        },
                        &dependency,
                        dependency.kind == TocDependencyKind::Required,
                    ));
                }
            }
            dependencies.push(dependency);
        }
        if !plan.file_closure_complete {
            issues.push(issue(
                ProjectPackageLoadIssueKind::PackageFileClosurePartial,
                package,
                None,
                None,
                true,
            ));
        }
    }

    // Ordinals are package-local in the parser projection. Rebind them to one
    // deterministic global sequence after canonical package ordering.
    for (ordinal, dependency) in dependencies.iter_mut().enumerate() {
        dependency.ordinal = ordinal as u64;
    }

    let adjacency = resolved_adjacency(declarations, &dependencies);
    let components = strongly_connected_components(
        declaration_by_name.keys().cloned().collect(),
        &adjacency,
        stop,
    )?;
    let (order_groups, group_by_package, cycle_issues) =
        order_components(&components, &dependencies, &adjacency, stop)?;
    issues.extend(cycle_issues);

    let reachability = classify_reachability(declarations, &dependencies, stop)?;
    let mut nodes = Vec::with_capacity(declarations.len());
    for declaration in declaration_by_name.values() {
        checkpoint(stop)?;
        let plan = plan_by_name
            .get(&declaration.name)
            .ok_or_else(|| invalid("loaded package plan is missing"))?;
        let state = *reachability
            .get(&declaration.name)
            .unwrap_or(&ProjectPackageReachability::Unreachable);
        let lod = *load_on_demand
            .get(&declaration.name)
            .unwrap_or(&TocLoadOnDemandState::NotDeclared);
        nodes.push(ProjectPackageNode {
            package: declaration.name.clone(),
            selected_root: declaration.selected_root,
            selected_toc: plan.selected_toc.clone(),
            selected_plan_digest: plan.plan_digest,
            variants: plan.variants.clone(),
            load_on_demand: lod,
            reachability: state,
            phase: package_phase(declaration, state, lod),
        });
    }
    nodes.sort_by(|left, right| left.package.cmp(&right.package));

    let node_by_name = nodes
        .iter()
        .map(|node| (node.package.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let mut units = Vec::new();
    for group in &order_groups {
        for package in &group.packages {
            checkpoint(stop)?;
            let node = node_by_name
                .get(package.as_str())
                .ok_or_else(|| invalid("package order references an unknown node"))?;
            let plan = plan_by_name
                .get(package)
                .ok_or_else(|| invalid("package load plan is missing"))?;
            for draft in &plan.units {
                checkpoint(stop)?;
                units.push(ProjectPackageLoadUnit {
                    ordinal: units.len() as u64,
                    unit_digest: draft.unit_digest,
                    package: package.clone(),
                    order_group: *group_by_package
                        .get(package)
                        .ok_or_else(|| invalid("package order group is missing"))?,
                    source_ordinal: draft.source_ordinal,
                    document: draft.document.clone(),
                    target: draft.target.clone(),
                    kind: draft.kind,
                    bootstrap: draft.bootstrap,
                    reachability: node.reachability,
                    phase: if draft.bootstrap {
                        ProjectPackageLoadPhase::Bootstrap
                    } else {
                        node.phase
                    },
                });
            }
        }
    }

    issues.sort_by(|left, right| {
        (
            &left.package,
            &left.dependency,
            left.byte_start,
            left.kind as u8,
        )
            .cmp(&(
                &right.package,
                &right.dependency,
                right.byte_start,
                right.kind as u8,
            ))
    });
    let dependencies_complete = !issues.iter().any(|issue| {
        issue.blocks_complete
            && !matches!(
                issue.kind,
                ProjectPackageLoadIssueKind::PackageFileClosurePartial
                    | ProjectPackageLoadIssueKind::RequiredDependencyCycle
                    | ProjectPackageLoadIssueKind::OptionalDependencyCycle
                    | ProjectPackageLoadIssueKind::MixedDependencyCycle
            )
    });
    let canonical_order_complete = !issues.iter().any(|issue| {
        matches!(
            issue.kind,
            ProjectPackageLoadIssueKind::RequiredDependencyCycle
                | ProjectPackageLoadIssueKind::OptionalDependencyCycle
                | ProjectPackageLoadIssueKind::MixedDependencyCycle
        )
    });
    let file_closure_complete = !issues
        .iter()
        .any(|issue| issue.kind == ProjectPackageLoadIssueKind::PackageFileClosurePartial);
    let coverage = ProjectPackageLoadCoverage {
        variants_complete: true,
        dependencies_complete,
        canonical_order_complete,
        file_closure_complete,
    };
    let target_profile_digest = profile_digest(profile)?;
    #[derive(Serialize)]
    struct Identity<'a> {
        profile: &'static str,
        target_profile_digest: ContentDigest<CanonicalResult>,
        packages: &'a [ProjectPackageNode],
        dependencies: &'a [TocDependencyDeclaration],
        order_groups: &'a [ProjectPackageOrderGroup],
        units: &'a [ProjectPackageLoadUnit],
        issues: &'a [ProjectPackageLoadIssue],
        coverage: ProjectPackageLoadCoverage,
    }
    let identity = Identity {
        profile: PACKAGE_LOAD_PROFILE,
        target_profile_digest,
        packages: &nodes,
        dependencies: &dependencies,
        order_groups: &order_groups,
        units: &units,
        issues: &issues,
        coverage,
    };
    if capacity.is_census() {
        census::bounded_encoded_size(&identity, stop)?;
    }
    let digest = crate::identity::canonical_digest(
        "wow-project/package-load-plan/1",
        &identity,
        ProjectPhase::Inventory,
    )?;
    checkpoint(stop)?;
    Ok(PackagePlanRecords {
        profile: PACKAGE_LOAD_PROFILE,
        target_flavor: profile.flavor_id().to_owned(),
        target_interface: profile.interface(),
        target_profile_digest,
        packages: nodes,
        dependencies,
        order_groups,
        units,
        issues,
        coverage,
        digest,
    })
}

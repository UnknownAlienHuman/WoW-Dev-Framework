//! Project exact package, dependency and static load facts from the retained
//! package-universe receipt. These are source topology facts, not runtime load
//! observations or addon lifecycle claims.
use std::collections::BTreeSet;

use super::*;
use crate::load::{
    PACKAGE_MAIN_NAMESPACE_ROOT, ProjectPackageLoadPhase, ProjectPackageReachability,
    TocDependencyKind, TocDependencyResolution,
};
use wow_core::{CanonicalResult, ContentDigest};

pub(super) const MAX_PACKAGE_NODES: usize = 64;
pub(super) const MAX_PACKAGE_RELATIONS: usize = 12_288;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphPackage {
    pub package: String,
    pub selected_toc: String,
    pub order_group: u64,
    pub reachability: ProjectPackageReachability,
    pub phase: ProjectPackageLoadPhase,
    pub proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphPackageFile {
    pub package: String,
    pub path: String,
    pub proposal_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ProjectGraphPackageDependencyOutcome {
    Projected {
        proposal_id: String,
        confidence: GraphConfidence,
    },
    Excluded,
    ConditionUnresolved,
    Missing,
    InvalidName,
    SelfDependency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphPackageDependency {
    pub ordinal: u64,
    pub package: String,
    pub dependency: String,
    pub kind: TocDependencyKind,
    pub outcome: ProjectGraphPackageDependencyOutcome,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ProjectGraphPackageLoadOutcome {
    Projected {
        proposal_id: String,
        confidence: GraphConfidence,
    },
    UnreachablePackage,
    SourceNotRegistered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphPackageLoad {
    pub unit_digest: ContentDigest<CanonicalResult>,
    pub package: String,
    pub target: String,
    pub outcome: ProjectGraphPackageLoadOutcome,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

pub(super) struct PackageProposals {
    pub entities: Vec<GraphEntityProposal>,
    pub relations: Vec<GraphRelationProposal>,
}

pub(super) fn project(
    project: &ProjectView,
    file_ids: &BTreeMap<&str, String>,
    sources: &BTreeMap<&str, &LoadSource>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<PackageProposals> {
    let mut output = PackageProposals {
        entities: Vec::new(),
        relations: Vec::new(),
    };
    let config = project.configuration();
    let (Some(plan), Some(main_plan)) = (config.package_load_plan(), config.package_main_plan())
    else {
        return Ok(output);
    };
    plan.validate_profile(config.selected_profile())?;
    main_plan.validate_load_plan(plan)?;
    if plan.packages().len() > MAX_PACKAGE_NODES {
        return Err(exhausted());
    }

    let group_by_package = plan
        .order_groups()
        .iter()
        .flat_map(|group| {
            group
                .packages
                .iter()
                .map(move |package| (package.as_str(), group.ordinal))
        })
        .collect::<BTreeMap<_, _>>();
    let mut package_ids = BTreeMap::<&str, String>::new();
    for package in plan.packages() {
        crate::analyzer::checkpoint(stop)?;
        let selected_plan = plan.package_plan(&package.package).ok_or_else(invalid)?;
        let selected_path = package_path(&package.package, selected_plan.selected_toc());
        let source = sources
            .get(selected_path.as_str())
            .copied()
            .ok_or_else(invalid)?;
        verify_document(selected_plan, selected_plan.selected_toc(), source)?;
        charge(text_bytes, selected_path.len().saturating_mul(8))?;
        let (handle, evidence) = support(project, source, SourceSpan::whole_file(), provenance)?;
        let key = crate::identity::canonical_digest(
            "wow-project/source-package-proposal/1",
            &package.package,
            ProjectPhase::View,
        )?;
        let proposal_id = format!("package:{key}");
        output.entities.push(
            GraphEntityProposal::new(
                proposal_id.as_str(),
                "source_package",
                BTreeMap::from([(
                    "package".into(),
                    GraphProposalValue::Identifier(package.package.clone().into()),
                )]),
                GraphConfidence::Proven,
                vec![handle],
                vec![evidence],
                Vec::new(),
            )
            .map_err(|_| invalid())?,
        );
        package_ids.insert(package.package.as_str(), proposal_id.clone());
        provenance.packages.push(ProjectGraphPackage {
            package: package.package.clone(),
            selected_toc: selected_path,
            order_group: *group_by_package
                .get(package.package.as_str())
                .ok_or_else(invalid)?,
            reachability: package.reachability,
            phase: package.phase,
            proposal_id,
            source_handle_id: handle,
            evidence_id: evidence,
        });
    }

    let file_support = provenance
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    let package_support = provenance
        .packages
        .iter()
        .map(|package| (package.package.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let mut owned = BTreeSet::new();
    for package in plan.packages() {
        let selected_plan = plan.package_plan(&package.package).ok_or_else(invalid)?;
        for source in selected_plan.sources() {
            crate::analyzer::checkpoint(stop)?;
            let path = package_path(&package.package, &source.path);
            let Some(file_proposal_id) = file_ids.get(path.as_str()) else {
                continue;
            };
            if !owned.insert((package.package.as_str(), path.clone())) {
                continue;
            }
            if output.relations.len() >= MAX_PACKAGE_RELATIONS {
                return Err(exhausted());
            }
            let package_record = package_support
                .get(package.package.as_str())
                .copied()
                .ok_or_else(invalid)?;
            let file_record = file_support
                .get(path.as_str())
                .copied()
                .ok_or_else(invalid)?;
            let key = crate::identity::canonical_digest(
                "wow-project/source-package-file-proposal/1",
                &(&package.package, &path),
                ProjectPhase::View,
            )?;
            let proposal_id = format!("package-file:{key}");
            output.relations.push(
                GraphRelationProposal::new(
                    proposal_id.as_str(),
                    "source_package_owns",
                    GraphRelationProposalInput {
                        source: GraphProposalEndpoint::Proposed(
                            package_record.proposal_id.clone().into(),
                        ),
                        target: GraphProposalEndpoint::Proposed(file_proposal_id.clone().into()),
                        confidence: GraphConfidence::Proven,
                        source_handle_ids: vec![
                            package_record.source_handle_id,
                            file_record.source_handle_id,
                        ]
                        .into_iter()
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect(),
                        evidence_ids: [package_record.evidence_id, file_record.evidence_id]
                            .into_iter()
                            .collect::<BTreeSet<_>>()
                            .into_iter()
                            .collect(),
                        coverage_ids: Vec::new(),
                    },
                )
                .map_err(|_| invalid())?,
            );
            provenance.package_files.push(ProjectGraphPackageFile {
                package: package.package.clone(),
                path,
                proposal_id,
            });
        }
    }

    for dependency in plan.dependencies() {
        crate::analyzer::checkpoint(stop)?;
        let selected_plan = plan.package_plan(&dependency.package).ok_or_else(invalid)?;
        let document_path = package_path(&dependency.package, &dependency.document);
        let source = sources
            .get(document_path.as_str())
            .copied()
            .ok_or_else(invalid)?;
        verify_span(
            selected_plan,
            &dependency.document,
            dependency.byte_start,
            dependency.byte_end,
            source,
        )?;
        let span = SourceSpan::byte_range(dependency.byte_start, dependency.byte_end)
            .map_err(|_| invalid())?;
        let (handle, evidence) = support(project, source, span, provenance)?;
        let outcome = if dependency.package == dependency.dependency
            || dependency.resolved_package.as_deref() == Some(dependency.package.as_str())
        {
            ProjectGraphPackageDependencyOutcome::SelfDependency
        } else {
            match dependency.resolution {
                TocDependencyResolution::Resolved => {
                    let target = dependency.resolved_package.as_deref().ok_or_else(invalid)?;
                    let confidence = match dependency.kind {
                        TocDependencyKind::Required => GraphConfidence::Proven,
                        TocDependencyKind::Optional => GraphConfidence::Possible,
                    };
                    let key = crate::identity::canonical_digest(
                        "wow-project/source-package-dependency-proposal/1",
                        &(
                            dependency.ordinal,
                            &dependency.package,
                            target,
                            dependency.kind,
                        ),
                        ProjectPhase::View,
                    )?;
                    let proposal_id = format!("package-dependency:{key}");
                    if output.relations.len() >= MAX_PACKAGE_RELATIONS {
                        return Err(exhausted());
                    }
                    output.relations.push(
                        GraphRelationProposal::new(
                            proposal_id.as_str(),
                            "source_package_depends_on",
                            GraphRelationProposalInput {
                                source: GraphProposalEndpoint::Proposed(
                                    package_ids[dependency.package.as_str()].clone().into(),
                                ),
                                target: GraphProposalEndpoint::Proposed(
                                    package_ids[target].clone().into(),
                                ),
                                confidence,
                                source_handle_ids: vec![handle],
                                evidence_ids: vec![evidence],
                                coverage_ids: Vec::new(),
                            },
                        )
                        .map_err(|_| invalid())?,
                    );
                    ProjectGraphPackageDependencyOutcome::Projected {
                        proposal_id,
                        confidence,
                    }
                }
                TocDependencyResolution::Excluded => ProjectGraphPackageDependencyOutcome::Excluded,
                TocDependencyResolution::ConditionUnresolved => {
                    ProjectGraphPackageDependencyOutcome::ConditionUnresolved
                }
                TocDependencyResolution::Missing => ProjectGraphPackageDependencyOutcome::Missing,
                TocDependencyResolution::InvalidName => {
                    ProjectGraphPackageDependencyOutcome::InvalidName
                }
            }
        };
        provenance
            .package_dependencies
            .push(ProjectGraphPackageDependency {
                ordinal: dependency.ordinal,
                package: dependency.package.clone(),
                dependency: dependency.dependency.clone(),
                kind: dependency.kind,
                outcome,
                source_handle_id: handle,
                evidence_id: evidence,
            });
    }

    for unit in plan.units() {
        crate::analyzer::checkpoint(stop)?;
        let selected_plan = plan.package_plan(&unit.package).ok_or_else(invalid)?;
        let record = selected_plan
            .records()
            .iter()
            .find(|record| record.ordinal == unit.source_ordinal)
            .ok_or_else(invalid)?;
        if record.document != unit.document
            || record.target.as_deref() != Some(unit.target.as_str())
            || record.kind != unit.kind
        {
            return Err(invalid());
        }
        let document_path = package_path(&unit.package, &unit.document);
        let source = sources
            .get(document_path.as_str())
            .copied()
            .ok_or_else(invalid)?;
        verify_span(
            selected_plan,
            &unit.document,
            record.byte_start,
            record.byte_end,
            source,
        )?;
        let span =
            SourceSpan::byte_range(record.byte_start, record.byte_end).map_err(|_| invalid())?;
        let (handle, evidence) = support(project, source, span, provenance)?;
        let target_path = package_path(&unit.package, &unit.target);
        let outcome = if unit.reachability == ProjectPackageReachability::Unreachable {
            ProjectGraphPackageLoadOutcome::UnreachablePackage
        } else if let Some(target) = file_ids.get(target_path.as_str()) {
            let confidence = if unit.reachability == ProjectPackageReachability::Reachable {
                GraphConfidence::Proven
            } else {
                GraphConfidence::Possible
            };
            let proposal_id = format!("package-load:{}", unit.unit_digest);
            if output.relations.len() >= MAX_PACKAGE_RELATIONS {
                return Err(exhausted());
            }
            output.relations.push(
                GraphRelationProposal::new(
                    proposal_id.as_str(),
                    "source_package_loads",
                    GraphRelationProposalInput {
                        source: GraphProposalEndpoint::Proposed(
                            package_ids[unit.package.as_str()].clone().into(),
                        ),
                        target: GraphProposalEndpoint::Proposed(target.clone().into()),
                        confidence,
                        source_handle_ids: vec![handle],
                        evidence_ids: vec![evidence],
                        coverage_ids: Vec::new(),
                    },
                )
                .map_err(|_| invalid())?,
            );
            ProjectGraphPackageLoadOutcome::Projected {
                proposal_id,
                confidence,
            }
        } else {
            ProjectGraphPackageLoadOutcome::SourceNotRegistered
        };
        provenance.package_loads.push(ProjectGraphPackageLoad {
            unit_digest: unit.unit_digest,
            package: unit.package.clone(),
            target: target_path,
            outcome,
            source_handle_id: handle,
            evidence_id: evidence,
        });
    }

    Ok(output)
}

fn package_path(package: &str, local: &str) -> String {
    format!("{PACKAGE_MAIN_NAMESPACE_ROOT}/{package}/{local}")
}

fn verify_document(
    plan: &crate::load::ProjectLoadPlan,
    local: &str,
    source: &LoadSource,
) -> ProjectResult<()> {
    let text = plan.document_text(local).ok_or_else(invalid)?;
    if text.len() as u64 != source.byte_length
        || crate::identity::source_digest(text.as_bytes()) != source.content_digest
    {
        return Err(invalid());
    }
    Ok(())
}

fn verify_span(
    plan: &crate::load::ProjectLoadPlan,
    local: &str,
    start: u64,
    end: u64,
    source: &LoadSource,
) -> ProjectResult<()> {
    verify_document(plan, local, source)?;
    let text = plan.document_text(local).ok_or_else(invalid)?;
    let start = usize::try_from(start).map_err(|_| invalid())?;
    let end = usize::try_from(end).map_err(|_| invalid())?;
    if text.get(start..end).is_none() {
        return Err(invalid());
    }
    Ok(())
}

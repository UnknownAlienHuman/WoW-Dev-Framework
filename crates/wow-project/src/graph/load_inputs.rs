//! Borrowed local load coordinates and genuine owner-resolved graph coordinates.
use super::*;
use crate::load::{
    ProjectLoadPlan, ProjectPackageLoadPlan, ProjectPackageMainPlan, ProjectPackageNode,
    ProjectPackageReachability,
};
use crate::xml_bindings::{
    ProjectPackageXmlLuaBindingAddress, ProjectPackageXmlLuaBindings, XmlInheritedScriptSource,
    XmlLuaBinding, XmlReceiverSources,
};
use wow_emmy::bindings::SymbolLookupReport;

enum Scope<'a> {
    Standalone,
    Package {
        node: &'a ProjectPackageNode,
        packages: &'a ProjectPackageLoadPlan,
        main: &'a ProjectPackageMainPlan,
        bindings: &'a ProjectPackageXmlLuaBindings,
    },
}

pub(super) struct LoadPlanGraphInput<'a> {
    project: &'a ProjectView,
    plan: &'a ProjectLoadPlan,
    scope: Scope<'a>,
}

pub(super) struct ScopedXmlBindings<'a> {
    pub bindings: &'a [XmlLuaBinding],
    pub receiver_sources: &'a BTreeMap<String, XmlReceiverSources>,
    pub inherited_script_sources: &'a [XmlInheritedScriptSource],
    pub symbol_lookup: Option<&'a SymbolLookupReport>,
    address_owner: Option<(&'a ProjectPackageXmlLuaBindings, &'a str)>,
}
impl ScopedXmlBindings<'_> {
    pub fn binding_address(
        &self,
        index: usize,
    ) -> ProjectResult<Option<ProjectPackageXmlLuaBindingAddress>> {
        let binding = self.bindings.get(index).ok_or_else(invalid)?;
        match self.address_owner {
            None => Ok(None),
            Some((owner, package)) => {
                let address = owner.address(package, index)?;
                if owner.resolve_binding(&address)? != binding {
                    return Err(invalid());
                }
                Ok(Some(address))
            }
        }
    }
}

impl<'a> LoadPlanGraphInput<'a> {
    pub fn plan(&self) -> &'a ProjectLoadPlan {
        self.plan
    }
    pub fn node(&self) -> Option<&'a ProjectPackageNode> {
        match self.scope {
            Scope::Standalone => None,
            Scope::Package { node, .. } => Some(node),
        }
    }
    pub fn package(&self) -> Option<&'a str> {
        self.node().map(|node| node.package.as_str())
    }
    pub fn qualified_path(&self, local: &str) -> ProjectResult<String> {
        if !self
            .plan
            .sources()
            .iter()
            .any(|source| source.path == local)
        {
            return Err(invalid());
        }
        match self.scope {
            Scope::Standalone => Ok(local.to_owned()),
            Scope::Package { node, packages, .. } => packages
                .source_path(&node.package, local)
                .ok_or_else(invalid),
        }
    }
    pub fn mapped_source(&self, local: &str) -> ProjectResult<LoadSource> {
        let source = self
            .plan
            .sources()
            .iter()
            .find(|s| s.path == local)
            .ok_or_else(invalid)?;
        Ok(LoadSource {
            path: self.qualified_path(local)?,
            content_digest: source.content_digest,
            byte_length: source.byte_length,
        })
    }
    pub fn proposal_id(&self, prefix: &str, document: &str, raw_id: &str) -> ProjectResult<String> {
        match self.scope {
            Scope::Standalone => Ok(format!("{prefix}:{raw_id}")),
            Scope::Package { node, .. } => {
                let digest = crate::identity::canonical_digest(
                    "wow-project/package-graph-address/1",
                    &(
                        prefix,
                        &node.package,
                        self.plan.digest(),
                        self.qualified_path(document)?,
                        raw_id,
                    ),
                    ProjectPhase::View,
                )?;
                Ok(format!("{prefix}:{digest}"))
            }
        }
    }
    /// Ordinals are local to this plan. Targets in another package deliberately
    /// have no entry, so they cannot satisfy a same-plan source-order guard.
    pub fn lua_loads(&self, stop: &AtomicBool) -> ProjectResult<BTreeMap<String, (u64, usize)>> {
        let mut loads = BTreeMap::new();
        for record in self.plan.records() {
            crate::analyzer::checkpoint(stop)?;
            if record.kind != LoadRecordKind::LuaFile || record.selection != LoadSelection::Included
            {
                continue;
            }
            let Some(local) = record.target.as_deref() else {
                continue;
            };
            let path = match self.scope {
                Scope::Standalone => local.to_owned(),
                Scope::Package { node, main, .. } => {
                    let Some(path) = main.resolve_source(&node.package, local) else {
                        if node.reachability == ProjectPackageReachability::Unreachable {
                            continue;
                        }
                        // The native loader retains the Included occurrence and
                        // a MissingFile issue without capturing bytes or Main.
                        if !self
                            .plan
                            .sources()
                            .iter()
                            .any(|source| source.path == local)
                            && self.plan.issues().iter().any(|issue| {
                                issue.kind == crate::load::LoadIssueKind::MissingFile
                                    && issue.document == record.document
                                    && issue.byte_start == record.byte_start
                                    && issue.byte_end == record.byte_end
                            })
                        {
                            continue;
                        }
                        return Err(invalid());
                    };
                    let receipt = main
                        .files()
                        .iter()
                        .find(|file| file.project_path == path)
                        .ok_or_else(invalid)?;
                    let source = self.mapped_source(local)?;
                    let file = self.project.file_by_path(path)?.ok_or_else(invalid)?;
                    if receipt.package != node.package
                        || receipt.source_path != local
                        || source.path != path
                        || source.content_digest != receipt.content_digest
                        || source.byte_length != receipt.byte_length
                        || file.content_digest() != receipt.content_digest
                        || file.byte_length() != receipt.byte_length
                    {
                        return Err(invalid());
                    }
                    path.to_owned()
                }
            };
            let entry = loads.entry(path).or_insert((record.ordinal, 0usize));
            entry.1 = entry.1.checked_add(1).ok_or_else(exhausted)?;
        }
        Ok(loads)
    }
    pub fn bindings(&self) -> ProjectResult<Option<ScopedXmlBindings<'a>>> {
        match self.scope {
            Scope::Standalone => Ok(self
                .project
                .snapshot()
                .analyzer_binding()
                .xml_bindings()
                .map(|report| ScopedXmlBindings {
                    bindings: report.bindings(),
                    receiver_sources: report.receiver_sources(),
                    inherited_script_sources: report.inherited_script_sources(),
                    symbol_lookup: report.symbol_lookup(),
                    address_owner: None,
                })),
            Scope::Package { node, bindings, .. } => {
                let group = bindings.group(&node.package).ok_or_else(invalid)?;
                Ok(Some(ScopedXmlBindings {
                    bindings: group.bindings(),
                    receiver_sources: group.receiver_sources(),
                    inherited_script_sources: group.inherited_script_sources(),
                    symbol_lookup: bindings.symbol_lookup(),
                    address_owner: Some((bindings, &node.package)),
                }))
            }
        }
    }
}

pub(super) fn scopes<'a>(
    project: &'a ProjectView,
    stop: &AtomicBool,
) -> ProjectResult<Vec<LoadPlanGraphInput<'a>>> {
    crate::analyzer::checkpoint(stop)?;
    let config = project.configuration();
    if config.platform_graph_profile().is_none() {
        return Ok(config
            .load_plan()
            .map(|plan| {
                vec![LoadPlanGraphInput {
                    project,
                    plan,
                    scope: Scope::Standalone,
                }]
            })
            .unwrap_or_default());
    }
    config.validate()?;
    let packages = config.package_load_plan().ok_or_else(invalid)?;
    let main = config.package_main_plan().ok_or_else(invalid)?;
    let analyzer = project.snapshot().analyzer_binding();
    let bindings = analyzer.package_xml_bindings().ok_or_else(invalid)?;
    if bindings.project_generation() != project.project_generation()
        || bindings.package_load_plan_digest() != packages.digest()
        || bindings.package_main_plan_digest() != main.digest()
        || bindings.main_snapshot_id() != analyzer.main_workspace().snapshot_id()
        || !bindings
            .library_snapshot_ids()
            .eq(analyzer.library_snapshot_ids())
        || bindings.groups().len() != packages.packages().len()
    {
        return Err(invalid());
    }
    if packages.packages().len() > super::packages::MAX_PACKAGE_NODES {
        return Err(exhausted());
    }
    let mut result = Vec::with_capacity(packages.packages().len());
    for node in packages.packages() {
        crate::analyzer::checkpoint(stop)?;
        let plan = packages.package_plan(&node.package).ok_or_else(invalid)?;
        let group = bindings.group(&node.package).ok_or_else(invalid)?;
        let scope = group.scope();
        if plan.digest() != node.selected_plan_digest
            || plan.selected_toc() != node.selected_toc
            || scope.package() != node.package
            || scope.load_plan_digest() != plan.digest()
            || scope.selected_toc() != node.selected_toc
            || scope.reachability() != node.reachability
            || scope.phase() != node.phase
            || group.documents().len() != plan.xml_documents().len()
        {
            return Err(invalid());
        }
        for (local, index) in plan.xml_documents() {
            crate::analyzer::checkpoint(stop)?;
            let document = group.documents().get(local).ok_or_else(invalid)?;
            let source = plan
                .sources()
                .iter()
                .find(|s| s.path == *local)
                .ok_or_else(invalid)?;
            if document.qualified_document()
                != packages
                    .source_path(&node.package, local)
                    .ok_or_else(invalid)?
                || document.content_digest() != source.content_digest
                || document.byte_length() != source.byte_length
                || index.source_digest() != source.content_digest
            {
                return Err(invalid());
            }
        }
        result.push(LoadPlanGraphInput {
            project,
            plan,
            scope: Scope::Package {
                node,
                packages,
                main,
                bindings,
            },
        });
    }
    Ok(result)
}

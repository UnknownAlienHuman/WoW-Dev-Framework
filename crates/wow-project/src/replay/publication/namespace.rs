//! Native platform/store binding. A namespace is caller intent until checked
//! against the actual admitted source, configuration and graph owners.
use std::collections::BTreeMap;

use wow_graph::GraphPartitionSnapshot;
use wow_store::project::ProjectStoreNamespace;

use crate::{ProjectKind, ProjectResult, ProjectView};

use super::invalid;

pub(super) fn bind(
    bindings: &mut BTreeMap<String, String>,
    namespace: &ProjectStoreNamespace,
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
) -> ProjectResult<()> {
    namespace.validate().map_err(|_| invalid())?;
    let config = project.configuration();
    if config.project_kind() != ProjectKind::BlizzardUiPlatformSource
        || namespace.owner_project_id() != config.project_id().as_str()
    {
        return Err(invalid());
    }
    let packages = config.platform_packages().ok_or_else(invalid)?;
    let source = packages.source();
    let profile_label = source.profile().profile_id().to_string();
    if namespace.logical_namespace() != profile_label
        || graph.snapshot().universe().as_str() != packages.binding().universe_id()
    {
        return Err(invalid());
    }
    packages
        .binding()
        .validate(source, packages.load_plan(), packages.main_plan())?;
    bindings.insert(
        "scope".into(),
        "native-platform-project-pair-namespace-v1".into(),
    );
    bindings.extend([
        ("project_store_id".into(), namespace.id().as_str().into()),
        (
            "project_store_namespace".into(),
            namespace.logical_namespace().into(),
        ),
        (
            "project_store_owner_project_id".into(),
            config.project_id().as_str().into(),
        ),
        ("platform_source_profile_id".into(), profile_label),
        (
            "platform_source_profile_digest".into(),
            source.profile().digest().to_string(),
        ),
        (
            "platform_source_snapshot_id".into(),
            source.receipt().source_snapshot_id().into(),
        ),
        (
            "graph_universe_id".into(),
            graph.snapshot().universe().as_str().into(),
        ),
    ]);
    Ok(())
}

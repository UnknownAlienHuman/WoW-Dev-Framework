//! Exact selected-TOC records for downstream structural recognizers. Parsing,
//! selection and dependency resolution stay in the existing load owner.
use super::*;
use crate::load::{
    LoadRecord, ProjectLoadPlan, TocCondition, TocDependencyDeclaration, TocDependencyKind,
    TocDependencyResolution, TocLoadOnDemandState, TocSavedVariableScope, TocSavedVariableState,
};
use wow_core::{CanonicalResult, ContentDigest, GenerationContextId, SourceContent};

pub const PROJECT_TOC_FACT_PROFILE: &str = "wow-project/toc-recognizer-facts/1";
const MAX_FACTS: usize = 32_768;

/// Constructed only from the generation's retained load plans. Unknown and
/// excluded records remain visible; an exact observation is not complete scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectTocFact {
    pub fact_id: String,
    pub context_id: GenerationContextId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    pub selected_toc: String,
    pub flavor: String,
    pub ordinal: u64,
    pub selection: LoadSelection,
    pub content_digest: ContentDigest<SourceContent>,
    pub span: SourceSpan,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
    pub kind: ProjectTocFactKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectTocFactKind {
    Package {
        plan_digest: ContentDigest<CanonicalResult>,
        target_interface: u64,
        source_complete: bool,
    },
    File {
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        declared_target: Option<String>,
        file_kind: LoadRecordKind,
        bootstrap: bool,
        conditions: Vec<TocCondition>,
        repeated: bool,
    },
    Dependency {
        name: String,
        dependency_kind: TocDependencyKind,
        resolution: TocDependencyResolution,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolved_package: Option<String>,
    },
    LoadOnDemand {
        value: String,
        declared_state: TocLoadOnDemandState,
        effective_state: TocLoadOnDemandState,
        conflicting: bool,
    },
    SavedVariable {
        entry_ordinal: u32,
        name: String,
        scope: TocSavedVariableScope,
        state: TocSavedVariableState,
    },
}

pub(super) fn project(
    project: &ProjectView,
    sources: &BTreeMap<&str, &LoadSource>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<Vec<ProjectTocFact>> {
    let mut output = Vec::new();
    let config = project.configuration();
    if let Some(plan) = config.load_plan() {
        append_plan(
            project,
            plan,
            None,
            sources,
            provenance,
            text_bytes,
            &mut output,
            stop,
        )?;
    } else if let Some(packages) = config.package_load_plan() {
        for package in packages.packages() {
            crate::analyzer::checkpoint(stop)?;
            let plan = packages
                .package_plan(&package.package)
                .ok_or_else(invalid)?;
            append_plan(
                project,
                plan,
                Some(&package.package),
                sources,
                provenance,
                text_bytes,
                &mut output,
                stop,
            )?;
        }
    }
    output.sort_by(|a, b| a.fact_id.cmp(&b.fact_id));
    if output
        .windows(2)
        .any(|pair| pair[0].fact_id == pair[1].fact_id)
    {
        return Err(invalid());
    }
    Ok(output)
}

#[allow(clippy::too_many_arguments)]
fn append_plan(
    project: &ProjectView,
    plan: &ProjectLoadPlan,
    package: Option<&str>,
    sources: &BTreeMap<&str, &LoadSource>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    output: &mut Vec<ProjectTocFact>,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    plan.validate_profile(project.configuration().selected_profile())?;
    let document = mapped_path(package, plan.selected_toc());
    let source = sources
        .get(document.as_str())
        .copied()
        .ok_or_else(invalid)?;
    let text = plan
        .document_text(plan.selected_toc())
        .ok_or_else(invalid)?;
    if source.content_digest != crate::identity::source_digest(text.as_bytes())
        || source.byte_length != text.len() as u64
    {
        return Err(invalid());
    }
    let metadata = crate::load::project_metadata(package.unwrap_or(""), plan)?;
    let mut builder = FactBuilder {
        project,
        package,
        document: &document,
        source,
        provenance,
        text_bytes,
        output,
        stop,
    };
    builder.push(
        0,
        LoadSelection::Included,
        SourceSpan::whole_file(),
        ProjectTocFactKind::Package {
            plan_digest: plan.digest(),
            target_interface: project.configuration().selected_profile().interface(),
            source_complete: plan.external_files_complete(),
        },
    )?;

    let records = plan
        .records()
        .iter()
        .filter(|r| r.document == plan.selected_toc());
    let mut counts = BTreeMap::<&str, usize>::new();
    for record in records.clone() {
        if matches!(
            record.kind,
            LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
        ) && let Some(target) = record.target.as_deref()
        {
            *counts.entry(target).or_default() += 1;
        }
    }
    for record in records {
        crate::analyzer::checkpoint(stop)?;
        verify_record(text, record)?;
        let span =
            SourceSpan::byte_range(record.byte_start, record.byte_end).map_err(|_| invalid())?;
        if matches!(
            record.kind,
            LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
        ) {
            let path = record.target.as_deref().map(|p| mapped_path(package, p));
            let repeated = record
                .target
                .as_deref()
                .is_some_and(|p| counts.get(p).is_some_and(|count| *count > 1));
            builder.push(
                record.ordinal,
                record.selection,
                span,
                ProjectTocFactKind::File {
                    path,
                    declared_target: record.declared_target.clone(),
                    file_kind: record.kind,
                    bootstrap: record.bootstrap,
                    conditions: record.conditions.clone(),
                    repeated,
                },
            )?;
        }
        if let Some(value) = record.metadata.as_ref().filter(|m| m.key == "loadondemand") {
            let declared_state = match (record.selection, value.value.as_str()) {
                (LoadSelection::Included, "0") => TocLoadOnDemandState::False,
                (LoadSelection::Included, "1") => TocLoadOnDemandState::True,
                _ => TocLoadOnDemandState::Unknown,
            };
            builder.push(
                record.ordinal,
                record.selection,
                span,
                ProjectTocFactKind::LoadOnDemand {
                    value: value.value.clone(),
                    declared_state,
                    effective_state: metadata.load_on_demand,
                    conflicting: metadata.conflicting_load_on_demand,
                },
            )?;
        }
        for variable in &record.saved_variables {
            builder.push(
                record.ordinal,
                record.selection,
                span,
                ProjectTocFactKind::SavedVariable {
                    entry_ordinal: variable.ordinal,
                    name: variable.name.clone(),
                    scope: variable.scope,
                    state: variable.state,
                },
            )?;
        }
    }
    let retained_dependencies: Vec<&TocDependencyDeclaration> = match package {
        Some(package) => project
            .configuration()
            .package_load_plan()
            .ok_or_else(invalid)?
            .dependencies()
            .iter()
            .filter(|dependency| dependency.package == package)
            .collect(),
        None => metadata.dependencies.iter().collect(),
    };
    for dependency in retained_dependencies {
        crate::analyzer::checkpoint(stop)?;
        if dependency.document != plan.selected_toc()
            || !plan.records().iter().any(|record| {
                record.document == dependency.document
                    && record.byte_start == dependency.byte_start
                    && record.byte_end == dependency.byte_end
                    && record.selection == dependency.selection
                    && record.kind == LoadRecordKind::Metadata
                    && record
                        .metadata
                        .as_ref()
                        .is_some_and(|metadata| metadata.key == dependency.source_key)
            })
        {
            return Err(invalid());
        }
        builder.push(
            dependency.ordinal,
            dependency.selection,
            SourceSpan::byte_range(dependency.byte_start, dependency.byte_end)
                .map_err(|_| invalid())?,
            ProjectTocFactKind::Dependency {
                name: dependency.dependency.clone(),
                dependency_kind: dependency.kind,
                resolution: dependency.resolution,
                resolved_package: dependency.resolved_package.clone(),
            },
        )?;
    }
    Ok(())
}

struct FactBuilder<'a> {
    project: &'a ProjectView,
    package: Option<&'a str>,
    document: &'a str,
    source: &'a LoadSource,
    provenance: &'a mut ProjectGraphProvenance,
    text_bytes: &'a mut usize,
    output: &'a mut Vec<ProjectTocFact>,
    stop: &'a AtomicBool,
}
impl FactBuilder<'_> {
    fn push(
        &mut self,
        ordinal: u64,
        selection: LoadSelection,
        span: SourceSpan,
        kind: ProjectTocFactKind,
    ) -> ProjectResult<()> {
        crate::analyzer::checkpoint(self.stop)?;
        if self.output.len() >= MAX_FACTS {
            return Err(exhausted());
        }
        let context_id = self.project.snapshot().generation_context().context_id();
        let flavor = self.project.configuration().selected_profile().flavor_id();
        #[derive(Serialize)]
        struct Identity<'a> {
            profile: &'static str,
            context_id: GenerationContextId,
            #[serde(skip_serializing_if = "Option::is_none")]
            package: Option<&'a str>,
            document: &'a str,
            flavor: &'a str,
            ordinal: u64,
            selection: LoadSelection,
            content_digest: ContentDigest<SourceContent>,
            span: SourceSpan,
            kind: &'a ProjectTocFactKind,
        }
        let identity = Identity {
            profile: PROJECT_TOC_FACT_PROFILE,
            context_id,
            package: self.package,
            document: self.document,
            flavor,
            ordinal,
            selection,
            content_digest: self.source.content_digest,
            span,
            kind: &kind,
        };
        let bytes = wow_core::canonical_json_bytes(&identity).map_err(|_| invalid())?;
        charge(self.text_bytes, bytes.len())?;
        let digest = crate::identity::canonical_digest(
            PROJECT_TOC_FACT_PROFILE,
            &identity,
            ProjectPhase::View,
        )?;
        let (source_handle_id, evidence_id) =
            support(self.project, self.source, span, self.provenance)?;
        self.output.push(ProjectTocFact {
            fact_id: format!("project-toc-fact:{digest}"),
            context_id,
            package: self.package.map(str::to_owned),
            selected_toc: self.document.to_owned(),
            flavor: flavor.to_owned(),
            ordinal,
            selection,
            content_digest: self.source.content_digest,
            span,
            source_handle_id,
            evidence_id,
            kind,
        });
        Ok(())
    }
}

fn mapped_path(package: Option<&str>, path: &str) -> String {
    package.map_or_else(
        || path.to_owned(),
        |package| {
            format!(
                "{}/{package}/{path}",
                crate::load::PACKAGE_MAIN_NAMESPACE_ROOT
            )
        },
    )
}

fn verify_record(text: &str, record: &LoadRecord) -> ProjectResult<()> {
    let start = usize::try_from(record.byte_start).map_err(|_| invalid())?;
    let end = usize::try_from(record.byte_end).map_err(|_| invalid())?;
    let raw = text.get(start..end).ok_or_else(invalid)?;
    if crate::identity::source_digest(raw.as_bytes()) != record.raw_digest {
        return Err(invalid());
    }
    Ok(())
}

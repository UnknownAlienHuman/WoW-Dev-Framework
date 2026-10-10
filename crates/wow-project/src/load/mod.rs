//! Selected-TOC acquisition and source-backed XML syntax/inline-body indexing.
//! Not a client emulator, complete semantic graph, or persistent E2 candidate.
pub mod census;
mod conditions;
pub(crate) mod document_toc;
mod metadata;
mod package;
mod package_closure;
pub(crate) use package_closure::{
    read_admitted_packages, validate_declarations as validate_package_declarations,
};
mod saved_variables;
pub mod schema;
mod toc;
pub use document_toc::DocumentTocSelection;
pub use metadata::TocMetadata;
pub use package_closure::{
    PACKAGE_LOAD_PROFILE, PACKAGE_MAIN_NAMESPACE_PROFILE, PACKAGE_MAIN_NAMESPACE_ROOT,
    ProjectLoadedPackage, ProjectPackageCycleKind, ProjectPackageInput, ProjectPackageLoadCoverage,
    ProjectPackageLoadInput, ProjectPackageLoadIssue, ProjectPackageLoadIssueKind,
    ProjectPackageLoadPhase, ProjectPackageLoadPlan, ProjectPackageLoadUnit,
    ProjectPackageMainFile, ProjectPackageMainInput, ProjectPackageMainPlan, ProjectPackageNode,
    ProjectPackageOrderGroup, ProjectPackageReachability, ProjectPackageVariantInput,
    ProjectPackageVariantReceipt, TocDependencyDeclaration, TocDependencyKind,
    TocDependencyResolution, TocLoadOnDemandState,
};
pub use saved_variables::{TocSavedVariable, TocSavedVariableScope, TocSavedVariableState};

// Intra-crate re-export for the source-graph fact projection. It is not a
// public API: the normalized metadata projection stays an internal owner seam.
pub(crate) use package_closure::project_metadata;
pub(crate) use package_closure::read_retained_packages;
mod xml;
mod xml_index;
pub mod xml_references;

pub use xml_index::{
    XML_INDEX_PROFILE, XmlAttributeRecord, XmlDeclaration, XmlDocumentIndex, XmlElementRecord,
    XmlElementRole, XmlInlineLua, XmlLuaMapKind, XmlLuaMapSegment, XmlScriptRecord,
    XmlScriptSource, XmlSourceSpan, XmlStructureIssue,
};

pub use conditions::{
    LoadSelection, TocCondition, TocConditionKind, TocLoadContext, TocLoadLocation,
    TocLuaEnvironment,
};

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity, SourceContent};

use crate::disk::{
    DISK_INVENTORY_MAX_BYTES, DISK_INVENTORY_MAX_FILES, DISK_SOURCE_MAX_BYTES, ProjectDiskFile,
    ProjectInputDirectory, checkpoint, validate_path,
};
use crate::platform_source::AdmittedPlatformSource;
use crate::{ProjectError, ProjectErrorCode, ProjectInputFile, ProjectPhase, ProjectResult};

/// Versioned, deliberately restricted acquisition semantics; never a WoW build.
pub const LOAD_PROFILE: &str = "wow-project/toc-xml-files/7";
const MAX_RECORDS: usize = 32_768;
const MAX_INCLUDE_DEPTH: usize = 32;

/// A retained lexical occurrence. Source bodies are never serialized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoadRecord {
    pub ordinal: u64,
    pub document: String,
    pub byte_start: u64,
    pub byte_end: u64,
    pub kind: LoadRecordKind,
    pub raw_digest: ContentDigest<SourceContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub bootstrap: bool,
    pub selection: LoadSelection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_target: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<TocCondition>,
    /// Normalized metadata directive retained at parse time. Absent for
    /// non-metadata records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TocMetadata>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub saved_variables: Vec<TocSavedVariable>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadRecordKind {
    Blank,
    Comment,
    Metadata,
    PackageGate,
    LuaFile,
    XmlFile,
    XmlElement,
    XmlEnd,
    XmlText,
    XmlDeclaration,
    Unknown,
}

/// Reasons why the selected file projection cannot claim complete load coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadIssueKind {
    MissingFile,
    IncludeCycle,
    RepeatedLoad,
    MissingInterface,
    ConflictingInterface,
    RequiredDependencyUnresolved,
    OptionalDependencyUnresolved,
    UnknownDirective,
    UnknownTocSyntax,
    LoadContextRequired,
    LoadConditionUnresolved,
    UnsupportedFileKind,
    XmlSemanticsUnresolved,
    UnsupportedXmlNamespace,
    UnsupportedXmlAttributes,
    InlineLuaNotAnalyzed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoadIssue {
    pub kind: LoadIssueKind,
    pub document: String,
    pub byte_start: u64,
    pub byte_end: u64,
    pub blocks_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoadSource {
    pub path: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
}

/// Exact plan, source receipts and encounter order. Fields cannot be supplied by
/// deserialization; only the bounded loader constructs this owner receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectLoadPlan {
    profile: &'static str,
    selected_toc: String,
    target_flavor: String,
    target_interface: u64,
    target_profile_digest: ContentDigest<CanonicalResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_context: Option<TocLoadContext>,
    sources: Vec<LoadSource>,
    records: Vec<LoadRecord>,
    issues: Vec<LoadIssue>,
    xml_documents: BTreeMap<String, XmlDocumentIndex>,
    xml_references: xml_references::XmlReferenceReport,
    digest: ContentDigest<CanonicalResult>,
    #[serde(skip)]
    captured_sources: Arc<BTreeMap<String, Arc<str>>>,
}

impl ProjectLoadPlan {
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
    #[must_use]
    pub fn selected_toc(&self) -> &str {
        &self.selected_toc
    }
    #[must_use]
    pub fn sources(&self) -> &[LoadSource] {
        &self.sources
    }
    #[must_use]
    pub fn records(&self) -> &[LoadRecord] {
        &self.records
    }
    #[must_use]
    pub fn issues(&self) -> &[LoadIssue] {
        &self.issues
    }
    #[must_use]
    pub fn external_files_complete(&self) -> bool {
        !self.issues.iter().any(|issue| issue.blocks_complete)
    }
    #[must_use]
    pub fn package_gate_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.kind == LoadRecordKind::PackageGate)
            .count()
    }
    #[must_use]
    pub fn excluded_records(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.selection == LoadSelection::Excluded)
            .count()
    }
    #[must_use]
    pub fn unresolved_records(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.selection == LoadSelection::Unresolved)
            .count()
    }
    #[must_use]
    pub fn load_context(&self) -> Option<&TocLoadContext> {
        self.load_context.as_ref()
    }
    /// Source-mapped XML syntax, one entry per unique captured document.
    #[must_use]
    pub fn xml_documents(&self) -> &BTreeMap<String, XmlDocumentIndex> {
        &self.xml_documents
    }
    /// Local declaration links, ambiguities and cycles in the captured XML scope.
    #[must_use]
    pub fn xml_references(&self) -> &xml_references::XmlReferenceReport {
        &self.xml_references
    }
    /// Verify that this exact retained plan belongs to the configured target.
    pub fn validate_profile(&self, profile: &ProfileIdentity) -> ProjectResult<()> {
        if self.target_interface != profile.interface()
            || self.target_flavor != profile.flavor_id()
            || self.target_profile_digest != profile_digest(profile)?
        {
            return Err(invalid(
                "load plan target differs from the selected profile",
            ));
        }
        Ok(())
    }
    /// Prevent attaching an unrelated/stale load receipt to an arbitrary Main inventory.
    pub fn validate_main_files(&self, files: &[ProjectInputFile]) -> ProjectResult<()> {
        let expected: Vec<_> = self
            .sources
            .iter()
            .filter(|source| source.path.ends_with(".lua"))
            .collect();
        if expected.len() != files.len() {
            return Err(invalid("load plan and Main inventory differ"));
        }
        for source in expected {
            if !files.iter().any(|file| {
                file.relative_path().as_str() == source.path
                    && file.content_digest() == source.content_digest
                    && file.byte_length() == source.byte_length
                    && file.role() == crate::ProjectFileRole::FirstPartyMain
            }) {
                return Err(invalid("load plan source identity differs from Main input"));
            }
        }
        Ok(())
    }
    /// Explicit access to the retained original TOC/XML bytes behind record spans.
    /// This never reopens a file that may have changed since acquisition.
    #[must_use]
    pub fn document_text(&self, path: &str) -> Option<&str> {
        (path == self.selected_toc || self.xml_documents.contains_key(path))
            .then(|| self.captured_text(path))
            .flatten()
    }
    /// Complete captured scope, including package-local Lua outside analyzer Main.
    pub(crate) fn captured_text(&self, path: &str) -> Option<&str> {
        self.captured_sources.get(path).map(AsRef::as_ref)
    }
}

/// Main source units and the exact load metadata that selected them.
pub struct ProjectLoadInput {
    files: Vec<ProjectInputFile>,
    plan: ProjectLoadPlan,
}

impl ProjectLoadInput {
    #[must_use]
    pub fn into_parts(self) -> (Vec<ProjectInputFile>, ProjectLoadPlan) {
        (self.files, self.plan)
    }
}

impl ProjectInputDirectory {
    /// Expand one explicitly selected TOC inside one declared root. No variant
    /// discovery, dependency fetching, source execution, or Library role changes.
    pub fn read_toc_project(
        &self,
        root: &str,
        selected_toc: &ProjectDiskFile,
        profile: &ProfileIdentity,
        stop: &AtomicBool,
    ) -> ProjectResult<ProjectLoadInput> {
        self.read_toc_project_with_context(root, selected_toc, profile, None, stop)
    }

    /// Expand conditional TOC entries using only the explicitly supplied context.
    /// The complete context and target profile are bound into the resulting plan.
    pub fn read_toc_project_with_context(
        &self,
        root: &str,
        selected_toc: &ProjectDiskFile,
        profile: &ProfileIdentity,
        context: Option<&TocLoadContext>,
        stop: &AtomicBool,
    ) -> ProjectResult<ProjectLoadInput> {
        checkpoint(stop)?;
        read_toc_source(
            LoaderSource::Disk(self.subdirectory(root)?),
            selected_toc,
            profile,
            context,
            stop,
        )
    }
}

/// Replays only the captured closure through the same loader. Missing entries
/// remain missing; surplus archived sources cannot silently enter another scope.
pub(crate) fn read_retained_toc(
    sources: BTreeMap<&str, &str>,
    selected_toc: &ProjectDiskFile,
    profile: &ProfileIdentity,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<ProjectLoadInput> {
    if sources.len() > DISK_INVENTORY_MAX_FILES {
        return Err(budget());
    }
    let mut total = 0usize;
    for (path, text) in &sources {
        checkpoint(stop)?;
        validate_path(path)?;
        total = total.checked_add(text.len()).ok_or_else(budget)?;
        if text.len() > DISK_SOURCE_MAX_BYTES || total > DISK_INVENTORY_MAX_BYTES {
            return Err(budget());
        }
        if text.contains('\0') {
            return Err(invalid("load source contains a NUL character"));
        }
    }
    read_toc_source(
        LoaderSource::Retained(sources),
        selected_toc,
        profile,
        context,
        stop,
    )
}

/// Expand a selected TOC from owner-held bytes through the existing Main loader.
pub(crate) fn read_admitted_toc(
    source: &AdmittedPlatformSource,
    root: &str,
    selected_toc: &ProjectDiskFile,
    profile: &ProfileIdentity,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<ProjectLoadInput> {
    checkpoint(stop)?;
    if profile != &source.profile().target().reference_profile {
        return Err(invalid(
            "admitted load profile differs from the source target",
        ));
    }
    read_toc_source(
        LoaderSource::Admitted { source, root },
        selected_toc,
        profile,
        context,
        stop,
    )
}

enum LoaderSource<'a> {
    Disk(ProjectInputDirectory),
    Retained(BTreeMap<&'a str, &'a str>),
    Admitted {
        source: &'a AdmittedPlatformSource,
        root: &'a str,
    },
}

fn read_toc_source(
    source: LoaderSource<'_>,
    selected_toc: &ProjectDiskFile,
    profile: &ProfileIdentity,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<ProjectLoadInput> {
    checkpoint(stop)?;
    if let Some(context) = context {
        context.validate()?;
    }
    profile
        .validate()
        .map_err(|_| invalid("selected TOC profile is invalid"))?;
    selected_toc.validate()?;
    if !selected_toc.path().ends_with(".toc") {
        return Err(invalid("selected TOC must name a .toc file"));
    }
    let mut loader = Loader {
        source,
        stop,
        total_bytes: 0,
        parsed_bytes: 0,
        captured: BTreeMap::new(),
        case_paths: BTreeMap::new(),
        records: Vec::new(),
        issues: Vec::new(),
        active: BTreeSet::new(),
        files: BTreeMap::new(),
        xml_documents: BTreeMap::new(),
        xml_nodes: 0,
        xml_attributes: 0,
        xml_segments: 0,
    };
    let path = selected_toc.path();
    let text = loader.capture(selected_toc)?;
    loader.charge_parse(text.len().saturating_mul(2))?;
    let parsed = toc::parse(&text, profile.interface(), context, stop)?;
    loader.expand(path, &text, parsed, 0, false)?;
    if let LoaderSource::Retained(sources) = &loader.source
        && sources.len() != loader.captured.len()
    {
        return Err(invalid(
            "retained load sources differ from the consumed closure",
        ));
    }
    if loader.files.is_empty() {
        return Err(invalid(
            "selected TOC closure contains no analyzable Lua files",
        ));
    }
    let sources = loader
        .captured
        .iter()
        .map(|(path, text)| LoadSource {
            path: path.clone(),
            content_digest: crate::identity::source_digest(text.as_bytes()),
            byte_length: text.len() as u64,
        })
        .collect::<Vec<_>>();
    // Source digests bind comments, order, directives and inline/unknown XML,
    // even when the unique Lua file inventory happens to remain unchanged.
    let xml_references = xml_references::resolve(&loader.xml_documents, &loader.records, stop)?;
    let target_profile_digest = profile_digest(profile)?;
    #[derive(Serialize)]
    struct Identity<'a> {
        profile: &'static str,
        selected_toc: &'a str,
        target_profile_digest: ContentDigest<CanonicalResult>,
        #[serde(skip_serializing_if = "Option::is_none")]
        load_context: Option<&'a TocLoadContext>,
        sources: &'a [LoadSource],
        records: &'a [LoadRecord],
        issues: &'a [LoadIssue],
        xml_documents: &'a BTreeMap<String, XmlDocumentIndex>,
        xml_references_digest: ContentDigest<CanonicalResult>,
    }
    let digest = crate::identity::canonical_digest(
        "wow-project/load-plan/7",
        &Identity {
            profile: LOAD_PROFILE,
            selected_toc: path,
            target_profile_digest,
            load_context: context,
            sources: &sources,
            records: &loader.records,
            issues: &loader.issues,
            xml_documents: &loader.xml_documents,
            xml_references_digest: xml_references.digest(),
        },
        ProjectPhase::Inventory,
    )?;
    checkpoint(stop)?;
    Ok(ProjectLoadInput {
        files: loader.files.into_values().collect(),
        plan: ProjectLoadPlan {
            profile: LOAD_PROFILE,
            selected_toc: path.to_owned(),
            target_flavor: profile.flavor_id().to_owned(),
            target_interface: profile.interface(),
            target_profile_digest,
            load_context: context.cloned(),
            sources,
            records: loader.records,
            issues: loader.issues,
            xml_documents: loader.xml_documents,
            xml_references,
            digest,
            captured_sources: Arc::new(loader.captured),
        },
    })
}

struct Loader<'a> {
    source: LoaderSource<'a>,
    stop: &'a AtomicBool,
    total_bytes: usize,
    parsed_bytes: usize,
    captured: BTreeMap<String, Arc<str>>,
    case_paths: BTreeMap<String, String>,
    records: Vec<LoadRecord>,
    issues: Vec<LoadIssue>,
    active: BTreeSet<String>,
    files: BTreeMap<String, ProjectInputFile>,
    xml_documents: BTreeMap<String, XmlDocumentIndex>,
    xml_nodes: usize,
    xml_attributes: usize,
    xml_segments: usize,
}

impl Loader<'_> {
    fn charge_parse(&mut self, bytes: usize) -> ProjectResult<()> {
        self.parsed_bytes = self.parsed_bytes.checked_add(bytes).ok_or_else(budget)?;
        if self.parsed_bytes > 64 * 1024 * 1024 {
            return Err(budget());
        }
        Ok(())
    }

    fn capture(&mut self, selected: &ProjectDiskFile) -> ProjectResult<Arc<str>> {
        checkpoint(self.stop)?;
        selected.validate()?;
        let path = selected.path();
        let folded = path.to_lowercase();
        if self
            .case_paths
            .get(&folded)
            .is_some_and(|prior| prior != path)
        {
            return Err(invalid("load references collide ignoring case"));
        }
        self.case_paths.insert(folded, path.to_owned());
        if let Some(text) = self.captured.get(path) {
            return Ok(Arc::clone(text));
        }
        if self.captured.len() >= DISK_INVENTORY_MAX_FILES {
            return Err(budget());
        }
        let remaining = DISK_INVENTORY_MAX_BYTES.saturating_sub(self.total_bytes);
        let limit = remaining.min(DISK_SOURCE_MAX_BYTES);
        let text: Arc<str> = match &self.source {
            LoaderSource::Disk(directory) => {
                let bytes = directory.read(selected, limit, self.stop)?;
                String::from_utf8(bytes)
                    .map_err(|_| invalid("load source must contain UTF-8"))?
                    .into()
            }
            LoaderSource::Retained(sources) => {
                let text = sources.get(path).ok_or_else(|| {
                    ProjectError::new(
                        ProjectErrorCode::MissingDeclaredFile,
                        ProjectPhase::Inventory,
                        "declared source is outside the retained closure",
                    )
                })?;
                if text.len() > limit {
                    return Err(budget());
                }
                selected.verify(text.as_bytes())?;
                Arc::from(*text)
            }
            LoaderSource::Admitted { source, root } => {
                let bytes = source.load_member(root, selected, limit, self.stop)?;
                let text = std::str::from_utf8(bytes).map_err(|_| {
                    ProjectError::new(
                        ProjectErrorCode::InvalidEncoding,
                        ProjectPhase::Inventory,
                        "admitted load source must contain UTF-8",
                    )
                    .with_relative_path(path)
                })?;
                checkpoint(self.stop)?;
                Arc::from(text)
            }
        };
        self.total_bytes += text.len();
        if text.contains('\0') {
            return Err(invalid("load source contains a NUL character"));
        }
        self.captured.insert(path.to_owned(), Arc::clone(&text));
        Ok(text)
    }

    fn expand(
        &mut self,
        document: &str,
        text: &str,
        parsed: Vec<Record>,
        depth: usize,
        bootstrap: bool,
    ) -> ProjectResult<()> {
        if depth > MAX_INCLUDE_DEPTH {
            return Err(budget());
        }
        self.active.insert(document.to_owned());
        for mut record in parsed {
            record.bootstrap |= bootstrap;
            checkpoint(self.stop)?;
            if self.records.len() >= MAX_RECORDS {
                return Err(budget());
            }
            let target = record
                .target
                .as_deref()
                .map(|raw| resolve(document, raw))
                .transpose()?;
            let index = self.records.len();
            self.records.push(LoadRecord {
                ordinal: index as u64,
                document: document.to_owned(),
                byte_start: record.start as u64,
                byte_end: record.end as u64,
                kind: record.kind,
                raw_digest: crate::identity::source_digest(
                    &text.as_bytes()[record.start..record.end],
                ),
                target: target.clone(),
                bootstrap: record.bootstrap,
                selection: record.selection,
                declared_target: record.declared_target,
                conditions: record.conditions,
                metadata: record.metadata,
                saved_variables: record.saved_variables,
            });
            for kind in record.issues {
                self.issue(kind, document, record.start, record.end);
            }
            let Some(target) = target else {
                continue;
            };
            if self.active.contains(&target) {
                self.issue(
                    LoadIssueKind::IncludeCycle,
                    document,
                    record.start,
                    record.end,
                );
                continue;
            }
            if self.captured.contains_key(&target) {
                // Occurrences remain in the plan; the analyzer only registers one
                // immutable physical file. Repeated execution is not certified.
                self.issue(
                    LoadIssueKind::RepeatedLoad,
                    document,
                    record.start,
                    record.end,
                );
            }
            if record.kind == LoadRecordKind::XmlFile && depth >= MAX_INCLUDE_DEPTH {
                return Err(budget());
            }
            let content = match self.capture(&ProjectDiskFile::new(&target)) {
                Ok(content) => content,
                Err(error) if error.code() == ProjectErrorCode::MissingDeclaredFile => {
                    self.issue(
                        LoadIssueKind::MissingFile,
                        document,
                        record.start,
                        record.end,
                    );
                    continue;
                }
                Err(error) => return Err(error),
            };
            match record.kind {
                LoadRecordKind::LuaFile => {
                    if let std::collections::btree_map::Entry::Vacant(entry) =
                        self.files.entry(target.clone())
                    {
                        entry.insert(ProjectInputFile::new(&target, content.as_ref())?);
                    }
                }
                LoadRecordKind::XmlFile => {
                    self.charge_parse(content.len())?;
                    let (parsed, index) = xml::parse(&target, &content, self.stop)?;
                    if !self.xml_documents.contains_key(&target) {
                        self.xml_nodes += index.elements().len();
                        self.xml_attributes += index.attribute_count();
                        self.xml_segments += index.map_segment_count();
                        if self.xml_nodes > MAX_RECORDS
                            || self.xml_attributes > 65_536
                            || self.xml_segments > 65_536
                        {
                            return Err(budget());
                        }
                        self.xml_documents.insert(target.clone(), index);
                    }
                    self.expand(&target, &content, parsed, depth + 1, record.bootstrap)?;
                }
                _ => return Err(invalid("unsupported load reference kind")),
            }
        }
        self.active.remove(document);
        Ok(())
    }

    fn issue(&mut self, kind: LoadIssueKind, document: &str, start: usize, end: usize) {
        self.issues.push(LoadIssue {
            kind,
            document: document.to_owned(),
            byte_start: start as u64,
            byte_end: end as u64,
            blocks_complete: kind != LoadIssueKind::OptionalDependencyUnresolved,
        });
    }
}

struct Record {
    kind: LoadRecordKind,
    start: usize,
    end: usize,
    target: Option<String>,
    bootstrap: bool,
    selection: LoadSelection,
    declared_target: Option<String>,
    conditions: Vec<TocCondition>,
    metadata: Option<TocMetadata>,
    issues: Vec<LoadIssueKind>,
    saved_variables: Vec<TocSavedVariable>,
}
impl Record {
    fn new(kind: LoadRecordKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            start,
            end,
            target: None,
            bootstrap: false,
            selection: LoadSelection::Included,
            declared_target: None,
            conditions: Vec::new(),
            metadata: None,
            issues: Vec::new(),
            saved_variables: Vec::new(),
        }
    }
    fn issue(mut self, issue: LoadIssueKind) -> Self {
        self.issues.push(issue);
        self
    }
}

fn resolve(document: &str, raw: &str) -> ProjectResult<String> {
    if raw.is_empty()
        || raw.len() > 4096
        || raw.chars().any(char::is_control)
        || raw.contains(['[', ']'])
    {
        return Err(invalid("invalid load reference"));
    }
    let path = raw.replace('\\', "/");
    if path.starts_with('/') || path.contains(':') {
        return Err(invalid("load reference leaves its declared relative root"));
    }
    let parent = document.rsplit_once('/').map(|(parent, _)| parent);
    let mut parts: Vec<&str> = parent
        .map(|parent| parent.split('/').collect())
        .unwrap_or_default();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(invalid("load reference escapes its declared root"));
                }
            }
            part => parts.push(part),
        }
    }
    let path = parts.join("/");
    validate_path(&path)?;
    Ok(path)
}
fn profile_digest(profile: &ProfileIdentity) -> ProjectResult<ContentDigest<CanonicalResult>> {
    crate::identity::canonical_digest(
        "wow-project/load-target-profile/1",
        profile,
        ProjectPhase::Inventory,
    )
}
fn invalid(message: &'static str) -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::InvalidInputInventory,
        ProjectPhase::Inventory,
        message,
    )
}
fn budget() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::Inventory,
        "TOC/XML acquisition exceeds its bounded load profile",
    )
}

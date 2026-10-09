//! Exact multi-package TOC variant selection and static dependency/load closure.
//! This owner consumes only caller-declared packages and the existing selected-TOC
//! loader. It never scans addon directories, fetches dependencies, or claims
//! runtime load success.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity, SourceContent};

use super::{
    LoadIssueKind, LoadRecordKind, LoadSelection, ProjectLoadPlan, TocLoadContext, budget, invalid,
    profile_digest,
};
use crate::disk::{
    DISK_SOURCE_MAX_BYTES, ProjectDiskFile, ProjectInputDirectory, checkpoint, validate_path,
};
use crate::{ProjectInputFile, ProjectPhase, ProjectResult};

/// Versioned multi-package static load closure. This is not runtime evidence.
pub const PACKAGE_LOAD_PROFILE: &str = "wow-project/package-load-closure/2";
/// Collision-free analyzer Main namespace derived from one exact package closure.
pub const PACKAGE_MAIN_NAMESPACE_PROFILE: &str = "wow-project/package-main-namespace/1";
/// Public logical root used only inside the project/analyzer source universe.
pub const PACKAGE_MAIN_NAMESPACE_ROOT: &str = "packages";
const MAX_PACKAGES: usize = 64;
const MAX_VARIANTS_PER_PACKAGE: usize = 32;
const MAX_DEPENDENCIES: usize = 4_096;
const MAX_PACKAGE_SOURCES: usize = 4_096;
const MAX_PACKAGE_SOURCE_BYTES: usize = 64 * 1024 * 1024;

/// One explicitly declared TOC variant. Exactly one variant per package must be
/// marked selected. Other variants are retained by exact identity only and never
/// contribute active files or metadata.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectPackageVariantInput {
    toc: ProjectDiskFile,
    selected: bool,
}

impl ProjectPackageVariantInput {
    #[must_use]
    pub fn new(toc: ProjectDiskFile, selected: bool) -> Self {
        Self { toc, selected }
    }

    #[must_use]
    pub const fn toc(&self) -> &ProjectDiskFile {
        &self.toc
    }

    #[must_use]
    pub const fn selected(&self) -> bool {
        self.selected
    }
}

/// One package inside an explicit package universe. `root` is a logical path
/// below the already registered input directory, never an ambient host path.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectPackageInput {
    name: String,
    root: String,
    selected_root: bool,
    variants: Vec<ProjectPackageVariantInput>,
}

impl ProjectPackageInput {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        root: impl Into<String>,
        selected_root: bool,
        variants: Vec<ProjectPackageVariantInput>,
    ) -> Self {
        Self {
            name: name.into(),
            root: root.into(),
            selected_root,
            variants,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn root(&self) -> &str {
        &self.root
    }

    #[must_use]
    pub const fn selected_root(&self) -> bool {
        self.selected_root
    }

    #[must_use]
    pub fn variants(&self) -> &[ProjectPackageVariantInput] {
        &self.variants
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TocDependencyKind {
    Required,
    Optional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TocDependencyResolution {
    Resolved,
    Missing,
    ConditionUnresolved,
    InvalidName,
    Excluded,
}

/// One declaration in selected-TOC source order. Duplicates are retained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TocDependencyDeclaration {
    pub ordinal: u64,
    pub package: String,
    pub dependency: String,
    pub kind: TocDependencyKind,
    pub source_key: String,
    pub document: String,
    pub byte_start: u64,
    pub byte_end: u64,
    pub selection: LoadSelection,
    pub resolution: TocDependencyResolution,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_package: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TocLoadOnDemandState {
    #[default]
    NotDeclared,
    False,
    True,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageReachability {
    Unreachable,
    ConditionallyReachable,
    Reachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageLoadPhase {
    DependencyPrerequisite,
    Bootstrap,
    Normal,
    OptionalDependencyConditional,
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageCycleKind {
    Required,
    Optional,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageLoadIssueKind {
    MissingRequiredDependency,
    MissingOptionalDependency,
    DependencyConditionUnresolved,
    InvalidDependencyName,
    DuplicateDependency,
    SelfDependency,
    RequiredDependencyCycle,
    OptionalDependencyCycle,
    MixedDependencyCycle,
    ConflictingLoadOnDemand,
    PackageFileClosurePartial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageLoadIssue {
    pub kind: ProjectPackageLoadIssueKind,
    pub package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_end: Option<u64>,
    pub blocks_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageVariantReceipt {
    pub package: String,
    pub toc: String,
    pub selected: bool,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageNode {
    pub package: String,
    pub selected_root: bool,
    pub selected_toc: String,
    pub selected_plan_digest: ContentDigest<CanonicalResult>,
    pub variants: Vec<ProjectPackageVariantReceipt>,
    pub load_on_demand: TocLoadOnDemandState,
    pub reachability: ProjectPackageReachability,
    pub phase: ProjectPackageLoadPhase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageOrderGroup {
    pub ordinal: u64,
    pub packages: Vec<String>,
    pub cyclic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_kind: Option<ProjectPackageCycleKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageLoadUnit {
    pub ordinal: u64,
    pub unit_digest: ContentDigest<CanonicalResult>,
    pub package: String,
    pub order_group: u64,
    pub source_ordinal: u64,
    pub document: String,
    pub target: String,
    pub kind: LoadRecordKind,
    pub bootstrap: bool,
    pub reachability: ProjectPackageReachability,
    pub phase: ProjectPackageLoadPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProjectPackageLoadCoverage {
    pub variants_complete: bool,
    pub dependencies_complete: bool,
    pub canonical_order_complete: bool,
    pub file_closure_complete: bool,
}

impl ProjectPackageLoadCoverage {
    #[must_use]
    pub const fn complete(self) -> bool {
        self.variants_complete
            && self.dependencies_complete
            && self.canonical_order_complete
            && self.file_closure_complete
    }
}

/// Exact mapping from one package-local source path to the collision-free path
/// registered in the shared analyzer Main workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageMainFile {
    pub package: String,
    pub source_path: String,
    pub project_path: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
}

/// Owner receipt for package-scoped Main registration. The package closure
/// digest is retained explicitly so dependency/order changes invalidate project
/// identity even when the Lua bytes and namespaced paths are unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageMainPlan {
    profile: &'static str,
    namespace_root: &'static str,
    package_load_plan_digest: ContentDigest<CanonicalResult>,
    files: Vec<ProjectPackageMainFile>,
    digest: ContentDigest<CanonicalResult>,
}

impl ProjectPackageMainPlan {
    fn build(
        load_plan: &ProjectPackageLoadPlan,
        files: Vec<ProjectPackageMainFile>,
    ) -> ProjectResult<Self> {
        #[derive(Serialize)]
        struct Identity<'a> {
            profile: &'static str,
            namespace_root: &'static str,
            package_load_plan_digest: ContentDigest<CanonicalResult>,
            files: &'a [ProjectPackageMainFile],
        }
        let package_load_plan_digest = load_plan.digest();
        let digest = crate::identity::canonical_digest(
            "wow-project/package-main-plan/1",
            &Identity {
                profile: PACKAGE_MAIN_NAMESPACE_PROFILE,
                namespace_root: PACKAGE_MAIN_NAMESPACE_ROOT,
                package_load_plan_digest,
                files: &files,
            },
            ProjectPhase::Inventory,
        )?;
        Ok(Self {
            profile: PACKAGE_MAIN_NAMESPACE_PROFILE,
            namespace_root: PACKAGE_MAIN_NAMESPACE_ROOT,
            package_load_plan_digest,
            files,
            digest,
        })
    }

    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }

    #[must_use]
    pub const fn package_load_plan_digest(&self) -> ContentDigest<CanonicalResult> {
        self.package_load_plan_digest
    }

    #[must_use]
    pub const fn namespace_root(&self) -> &'static str {
        self.namespace_root
    }

    #[must_use]
    pub fn files(&self) -> &[ProjectPackageMainFile] {
        &self.files
    }

    /// Resolve one exact package-local Lua source to the project path that was
    /// actually registered in Main. Callers must not reconstruct this namespace.
    #[must_use]
    pub fn resolve_source(&self, package: &str, source_path: &str) -> Option<&str> {
        self.files
            .iter()
            .find(|file| file.package == package && file.source_path == source_path)
            .map(|file| file.project_path.as_str())
    }

    pub fn validate_load_plan(&self, load_plan: &ProjectPackageLoadPlan) -> ProjectResult<()> {
        if self.profile != PACKAGE_MAIN_NAMESPACE_PROFILE
            || self.namespace_root != PACKAGE_MAIN_NAMESPACE_ROOT
            || self.package_load_plan_digest != load_plan.digest()
        {
            return Err(invalid(
                "package Main namespace does not match its package load plan",
            ));
        }
        Ok(())
    }

    /// Verify the exact collision-free Main inventory generated by this plan.
    pub fn validate_main_files(&self, files: &[ProjectInputFile]) -> ProjectResult<()> {
        if files.len() != self.files.len() {
            return Err(invalid(
                "package Main namespace and project inventory differ",
            ));
        }
        let mut expected = BTreeMap::new();
        for receipt in &self.files {
            if expected
                .insert(receipt.project_path.as_str(), receipt)
                .is_some()
            {
                return Err(invalid("package Main namespace contains duplicate paths"));
            }
        }
        for file in files {
            let Some(receipt) = expected.remove(file.relative_path().as_str()) else {
                return Err(invalid(
                    "package Main namespace contains an unexpected project path",
                ));
            };
            if file.role() != crate::ProjectFileRole::FirstPartyMain
                || file.content_digest() != receipt.content_digest
                || file.byte_length() != receipt.byte_length
            {
                return Err(invalid(
                    "package Main namespace source identity differs from its receipt",
                ));
            }
        }
        if !expected.is_empty() {
            return Err(invalid(
                "package Main namespace is missing a declared project path",
            ));
        }
        Ok(())
    }
}

/// Canonical package/dependency/phase/file-order receipt. Source bodies and host
/// roots are absent; each loaded package retains its ordinary ProjectLoadPlan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageLoadPlan {
    profile: &'static str,
    target_flavor: String,
    target_interface: u64,
    target_profile_digest: ContentDigest<CanonicalResult>,
    packages: Vec<ProjectPackageNode>,
    dependencies: Vec<TocDependencyDeclaration>,
    order_groups: Vec<ProjectPackageOrderGroup>,
    units: Vec<ProjectPackageLoadUnit>,
    issues: Vec<ProjectPackageLoadIssue>,
    coverage: ProjectPackageLoadCoverage,
    digest: ContentDigest<CanonicalResult>,
    #[serde(skip)]
    retained_plans: BTreeMap<String, ProjectLoadPlan>,
    #[serde(skip)]
    retained_variants: BTreeMap<String, BTreeMap<String, Arc<str>>>,
}

impl ProjectPackageLoadPlan {
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }

    #[must_use]
    pub fn packages(&self) -> &[ProjectPackageNode] {
        &self.packages
    }

    #[must_use]
    pub fn dependencies(&self) -> &[TocDependencyDeclaration] {
        &self.dependencies
    }

    #[must_use]
    pub fn order_groups(&self) -> &[ProjectPackageOrderGroup] {
        &self.order_groups
    }

    #[must_use]
    pub fn units(&self) -> &[ProjectPackageLoadUnit] {
        &self.units
    }

    #[must_use]
    pub fn issues(&self) -> &[ProjectPackageLoadIssue] {
        &self.issues
    }

    #[must_use]
    pub const fn coverage(&self) -> ProjectPackageLoadCoverage {
        self.coverage
    }

    /// Reopen the exact selected-TOC receipt for one admitted package. These
    /// source-backed plans are deliberately excluded from serialized output,
    /// but their digests are already committed by every package node.
    #[must_use]
    pub fn package_plan(&self, package: &str) -> Option<&ProjectLoadPlan> {
        self.retained_plans.get(package)
    }

    pub(crate) fn variant_text(&self, package: &str, toc: &str) -> Option<&str> {
        self.retained_variants
            .get(package)?
            .get(toc)
            .map(AsRef::as_ref)
    }

    /// Resolve one retained TOC/XML/Lua source to its package-qualified logical
    /// project path. The source must belong to the exact selected package plan.
    #[must_use]
    pub fn source_path(&self, package: &str, source_path: &str) -> Option<String> {
        let plan = self.retained_plans.get(package)?;
        plan.sources()
            .iter()
            .any(|source| source.path == source_path)
            .then(|| format!("{PACKAGE_MAIN_NAMESPACE_ROOT}/{package}/{source_path}"))
    }

    fn validate_retained_plans(&self) -> ProjectResult<()> {
        if self.retained_plans.len() != self.packages.len() {
            return Err(invalid(
                "package load plan does not retain every selected TOC receipt",
            ));
        }
        for package in &self.packages {
            let Some(plan) = self.retained_plans.get(&package.package) else {
                return Err(invalid("package selected TOC receipt is missing"));
            };
            if plan.digest() != package.selected_plan_digest
                || plan.selected_toc() != package.selected_toc
            {
                return Err(invalid(
                    "package selected TOC receipt differs from its canonical node",
                ));
            }
            let variants = self
                .retained_variants
                .get(&package.package)
                .ok_or_else(|| invalid("package variant sources are missing"))?;
            if variants.len() != package.variants.len() {
                return Err(invalid("package variant source set differs"));
            }
            for variant in &package.variants {
                let text = variants
                    .get(&variant.toc)
                    .ok_or_else(|| invalid("package variant source is missing"))?;
                if text.len() as u64 != variant.byte_length
                    || crate::identity::source_digest(text.as_bytes()) != variant.content_digest
                {
                    return Err(invalid("package variant source identity differs"));
                }
            }
        }
        Ok(())
    }

    pub fn validate_profile(&self, profile: &ProfileIdentity) -> ProjectResult<()> {
        if self.target_interface != profile.interface()
            || self.target_flavor != profile.flavor_id()
            || self.target_profile_digest != profile_digest(profile)?
        {
            return Err(invalid(
                "package load plan target differs from the selected profile",
            ));
        }
        self.validate_retained_plans()?;
        Ok(())
    }
}

/// One package's already admitted analyzer inputs and ordinary selected-TOC plan.
pub struct ProjectLoadedPackage {
    package: String,
    files: Vec<ProjectInputFile>,
    plan: ProjectLoadPlan,
}

impl ProjectLoadedPackage {
    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }

    #[must_use]
    pub fn files(&self) -> &[ProjectInputFile] {
        &self.files
    }

    #[must_use]
    pub const fn plan(&self) -> &ProjectLoadPlan {
        &self.plan
    }

    #[must_use]
    pub fn into_parts(self) -> (String, Vec<ProjectInputFile>, ProjectLoadPlan) {
        (self.package, self.files, self.plan)
    }
}

/// Multi-package acquisition result. Files remain package-scoped until the public
/// service defines its collision-free Main path namespace.
pub struct ProjectPackageLoadInput {
    packages: Vec<ProjectLoadedPackage>,
    plan: ProjectPackageLoadPlan,
}

/// Flattened analyzer Main inputs plus both exact package receipts. Source bytes
/// remain private inside `ProjectInputFile` and are never serialized by the plans.
pub struct ProjectPackageMainInput {
    files: Vec<ProjectInputFile>,
    load_plan: ProjectPackageLoadPlan,
    main_plan: ProjectPackageMainPlan,
}

impl ProjectPackageMainInput {
    #[must_use]
    pub fn files(&self) -> &[ProjectInputFile] {
        &self.files
    }

    #[must_use]
    pub const fn load_plan(&self) -> &ProjectPackageLoadPlan {
        &self.load_plan
    }

    #[must_use]
    pub const fn main_plan(&self) -> &ProjectPackageMainPlan {
        &self.main_plan
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        Vec<ProjectInputFile>,
        ProjectPackageLoadPlan,
        ProjectPackageMainPlan,
    ) {
        (self.files, self.load_plan, self.main_plan)
    }
}

impl ProjectPackageLoadInput {
    #[must_use]
    pub fn packages(&self) -> &[ProjectLoadedPackage] {
        &self.packages
    }

    #[must_use]
    pub const fn plan(&self) -> &ProjectPackageLoadPlan {
        &self.plan
    }

    #[must_use]
    pub fn into_parts(self) -> (Vec<ProjectLoadedPackage>, ProjectPackageLoadPlan) {
        (self.packages, self.plan)
    }

    /// Register every package-local Lua file under
    /// `packages/<package>/<source-path>`. The fixed prefix and admitted package
    /// grammar make cross-package collisions impossible without case-folding or
    /// path ambiguity being detected explicitly.
    pub fn into_namespaced_main(self) -> ProjectResult<ProjectPackageMainInput> {
        let mut files = Vec::new();
        let mut receipts = Vec::new();
        let mut folded_paths = BTreeSet::new();
        let reachability = self
            .plan
            .packages()
            .iter()
            .map(|package| (package.package.as_str(), package.reachability))
            .collect::<BTreeMap<_, _>>();
        for loaded in self.packages {
            let package = loaded.package;
            if reachability.get(package.as_str()) == Some(&ProjectPackageReachability::Unreachable)
            {
                continue;
            }
            for file in loaded.files {
                let source_path = file.relative_path().as_str().to_owned();
                let project_path = format!("{PACKAGE_MAIN_NAMESPACE_ROOT}/{package}/{source_path}");
                if !folded_paths.insert(project_path.to_ascii_lowercase()) {
                    return Err(invalid(
                        "package Main namespace paths collide ignoring case",
                    ));
                }
                receipts.push(ProjectPackageMainFile {
                    package: package.clone(),
                    source_path,
                    project_path: project_path.clone(),
                    content_digest: file.content_digest(),
                    byte_length: file.byte_length(),
                });
                files.push(file.with_project_path(project_path)?);
            }
        }
        if files.is_empty() {
            return Err(invalid("package Main namespace contains no Lua files"));
        }
        files.sort_by(|left, right| left.relative_path().cmp(right.relative_path()));
        receipts.sort_by(|left, right| left.project_path.cmp(&right.project_path));
        let main_plan = ProjectPackageMainPlan::build(&self.plan, receipts)?;
        main_plan.validate_main_files(&files)?;
        Ok(ProjectPackageMainInput {
            files,
            load_plan: self.plan,
            main_plan,
        })
    }
}

impl ProjectInputDirectory {
    /// Read an explicit package universe, select exactly one caller-declared TOC
    /// variant per package, expand each selected closure through the existing
    /// loader, then resolve package dependencies without source discovery.
    pub fn read_package_project_with_context(
        &self,
        packages: &[ProjectPackageInput],
        profile: &ProfileIdentity,
        context: Option<&TocLoadContext>,
        stop: &AtomicBool,
    ) -> ProjectResult<ProjectPackageLoadInput> {
        read_package_sources(PackageSources::Disk(self), packages, profile, context, stop)
    }
}

pub(crate) fn read_retained_packages(
    sources: BTreeMap<&str, BTreeMap<&str, &str>>,
    packages: &[ProjectPackageInput],
    profile: &ProfileIdentity,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<ProjectPackageLoadInput> {
    validate_declarations(packages)?;
    if sources.len() != packages.len()
        || packages
            .iter()
            .any(|package| !sources.contains_key(package.name.as_str()))
    {
        return Err(invalid(
            "retained package source scopes differ from declarations",
        ));
    }
    let mut count = 0usize;
    let mut bytes = 0usize;
    for scope in sources.values() {
        for (path, text) in scope {
            checkpoint(stop)?;
            validate_path(path)?;
            count = count.checked_add(1).ok_or_else(budget)?;
            bytes = bytes.checked_add(text.len()).ok_or_else(budget)?;
            if count > MAX_PACKAGE_SOURCES
                || bytes > MAX_PACKAGE_SOURCE_BYTES
                || text.len() > DISK_SOURCE_MAX_BYTES
            {
                return Err(budget());
            }
            if text.contains('\0') {
                return Err(invalid("retained package source contains a NUL character"));
            }
        }
    }
    read_package_sources(
        PackageSources::Retained(sources),
        packages,
        profile,
        context,
        stop,
    )
}

enum PackageSources<'a> {
    Disk(&'a ProjectInputDirectory),
    Retained(BTreeMap<&'a str, BTreeMap<&'a str, &'a str>>),
}

fn read_package_sources(
    sources: PackageSources<'_>,
    packages: &[ProjectPackageInput],
    profile: &ProfileIdentity,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
) -> ProjectResult<ProjectPackageLoadInput> {
    checkpoint(stop)?;
    profile
        .validate()
        .map_err(|_| invalid("package load profile is invalid"))?;
    if let Some(context) = context {
        context.validate()?;
    }
    let declarations = validate_declarations(packages)?;
    let mut loaded = Vec::with_capacity(declarations.len());
    let mut variants = BTreeMap::<String, Vec<ProjectPackageVariantReceipt>>::new();
    let mut variant_sources = BTreeMap::<String, BTreeMap<String, Arc<str>>>::new();
    let mut total_sources = 0usize;
    let mut total_bytes = 0usize;

    for package in declarations {
        checkpoint(stop)?;
        let selected = package
            .variants
            .iter()
            .find(|variant| variant.selected)
            .ok_or_else(|| invalid("package has no selected TOC variant"))?;
        let input = match &sources {
            PackageSources::Disk(directory) => directory.read_toc_project_with_context(
                &package.root,
                &selected.toc,
                profile,
                context,
                stop,
            )?,
            PackageSources::Retained(scopes) => {
                let scope = &scopes[package.name.as_str()];
                let selected_sources = scope
                    .iter()
                    .filter(|(path, _)| !path.ends_with(".toc") || **path == selected.toc.path())
                    .map(|(path, text)| (*path, *text))
                    .collect();
                super::read_retained_toc(selected_sources, &selected.toc, profile, context, stop)?
            }
        };
        let (files, plan) = input.into_parts();
        let selected_source = plan
            .sources()
            .iter()
            .find(|source| source.path == selected.toc.path())
            .ok_or_else(|| invalid("selected TOC is absent from its load receipt"))?;
        let mut receipts = vec![ProjectPackageVariantReceipt {
            package: package.name.clone(),
            toc: selected.toc.path().to_owned(),
            selected: true,
            content_digest: selected_source.content_digest,
            byte_length: selected_source.byte_length,
        }];
        let mut texts = BTreeMap::from([(
            selected.toc.path().to_owned(),
            Arc::<str>::from(
                plan.document_text(selected.toc.path())
                    .ok_or_else(|| invalid("selected TOC text is missing"))?,
            ),
        )]);
        for variant in package.variants.iter().filter(|variant| !variant.selected) {
            checkpoint(stop)?;
            let text: Arc<str> = match &sources {
                PackageSources::Disk(directory) => {
                    let bytes = directory.subdirectory(&package.root)?.read(
                        &variant.toc,
                        DISK_SOURCE_MAX_BYTES,
                        stop,
                    )?;
                    String::from_utf8(bytes)
                        .map_err(|_| invalid("unselected TOC variant must contain UTF-8"))?
                        .into()
                }
                PackageSources::Retained(scopes) => {
                    let text = scopes[package.name.as_str()]
                        .get(variant.toc.path())
                        .ok_or_else(|| invalid("unselected TOC variant source is missing"))?;
                    variant.toc.verify(text.as_bytes())?;
                    Arc::from(*text)
                }
            };
            if text.contains('\0') {
                return Err(invalid("unselected TOC variant contains a NUL character"));
            }
            receipts.push(ProjectPackageVariantReceipt {
                package: package.name.clone(),
                toc: variant.toc.path().to_owned(),
                selected: false,
                content_digest: crate::identity::source_digest(text.as_bytes()),
                byte_length: text.len() as u64,
            });
            total_sources = total_sources.checked_add(1).ok_or_else(budget)?;
            total_bytes = total_bytes.checked_add(text.len()).ok_or_else(budget)?;
            texts.insert(variant.toc.path().to_owned(), text);
        }
        receipts.sort_by(|left, right| left.toc.cmp(&right.toc));
        total_sources = total_sources
            .checked_add(plan.sources().len())
            .ok_or_else(budget)?;
        for source in plan.sources() {
            let length = usize::try_from(source.byte_length).map_err(|_| budget())?;
            total_bytes = total_bytes.checked_add(length).ok_or_else(budget)?;
        }
        if total_sources > MAX_PACKAGE_SOURCES || total_bytes > MAX_PACKAGE_SOURCE_BYTES {
            return Err(budget());
        }
        if let PackageSources::Retained(scopes) = &sources
            && scopes[package.name.as_str()].len() != plan.sources().len() + texts.len() - 1
        {
            return Err(invalid(
                "retained package source set includes undeclared variants",
            ));
        }
        variant_sources.insert(package.name.clone(), texts);
        variants.insert(package.name.clone(), receipts);
        loaded.push(ProjectLoadedPackage {
            package: package.name.clone(),
            files,
            plan,
        });
    }

    loaded.sort_by(|left, right| left.package.cmp(&right.package));
    let plan = build_plan(packages, &loaded, variants, variant_sources, profile, stop)?;
    Ok(ProjectPackageLoadInput {
        packages: loaded,
        plan,
    })
}

fn validate_declarations(
    packages: &[ProjectPackageInput],
) -> ProjectResult<Vec<&ProjectPackageInput>> {
    if packages.is_empty() || packages.len() > MAX_PACKAGES {
        return Err(invalid("invalid declared package count"));
    }
    let mut names = BTreeMap::<String, String>::new();
    let mut roots = BTreeMap::<String, String>::new();
    let mut selected_roots = 0usize;
    for package in packages {
        if !valid_package_name(&package.name) {
            return Err(invalid("invalid declared package name"));
        }
        if package.root != "." {
            validate_path(&package.root)?;
        }
        let folded_name = package.name.to_ascii_lowercase();
        if names.insert(folded_name, package.name.clone()).is_some() {
            return Err(invalid("declared package names collide ignoring case"));
        }
        let folded_root = package.root.to_ascii_lowercase();
        if roots.insert(folded_root, package.root.clone()).is_some() {
            return Err(invalid("declared package roots collide ignoring case"));
        }
        if package.selected_root {
            selected_roots += 1;
        }
        if package.variants.is_empty() || package.variants.len() > MAX_VARIANTS_PER_PACKAGE {
            return Err(invalid("invalid package TOC variant count"));
        }
        let mut selected = 0usize;
        let mut paths = BTreeSet::new();
        for variant in &package.variants {
            variant.toc.validate()?;
            if !variant.toc.path().ends_with(".toc") {
                return Err(invalid("package variants must name .toc files"));
            }
            if !paths.insert(variant.toc.path().to_ascii_lowercase()) {
                return Err(invalid("package TOC variants collide ignoring case"));
            }
            selected += usize::from(variant.selected);
        }
        if selected != 1 {
            return Err(invalid("package must select exactly one TOC variant"));
        }
    }
    if selected_roots == 0 {
        return Err(invalid("package universe has no selected root"));
    }
    let mut ordered = packages.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(ordered)
}

fn valid_package_name(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        && value != "."
        && value != ".."
}

/// Normalized selected-TOC metadata projection consumed by this owner.
#[derive(Default)]
pub(crate) struct MetadataProjection {
    pub(crate) dependencies: Vec<TocDependencyDeclaration>,
    pub(crate) load_on_demand: TocLoadOnDemandState,
    pub(crate) conflicting_load_on_demand: bool,
}

/// Project selected-TOC metadata from the normalized directives retained at
/// parse time. This reads the single source parse instead of re-splitting raw
/// spans, which would discard conditional filtering that already happened.
pub(crate) fn project_metadata(
    package: &str,
    plan: &ProjectLoadPlan,
) -> ProjectResult<MetadataProjection> {
    let document = plan.selected_toc();
    let mut projection = MetadataProjection::default();
    let mut load_values = Vec::new();
    for record in plan
        .records()
        .iter()
        .filter(|record| record.document == document && record.kind == LoadRecordKind::Metadata)
    {
        let Some(metadata) = record.metadata.as_ref() else {
            continue;
        };
        let key = metadata.key.as_str();
        let value = metadata.value.as_str();
        let dependency_kind = match key {
            "dependencies" | "requireddeps" | "dependson" => Some(TocDependencyKind::Required),
            "optionaldeps" => Some(TocDependencyKind::Optional),
            _ => None,
        };
        if let Some(kind) = dependency_kind {
            for dependency in split_dependencies(value)? {
                if projection.dependencies.len() >= MAX_DEPENDENCIES {
                    return Err(budget());
                }
                projection.dependencies.push(TocDependencyDeclaration {
                    ordinal: projection.dependencies.len() as u64,
                    package: package.to_owned(),
                    dependency,
                    kind,
                    source_key: key.to_owned(),
                    document: document.to_owned(),
                    byte_start: record.byte_start,
                    byte_end: record.byte_end,
                    selection: record.selection,
                    resolution: TocDependencyResolution::ConditionUnresolved,
                    resolved_package: None,
                });
            }
        } else if key == "loadondemand" && record.selection != LoadSelection::Excluded {
            load_values.push(match (record.selection, value) {
                (LoadSelection::Included, "0") => TocLoadOnDemandState::False,
                (LoadSelection::Included, "1") => TocLoadOnDemandState::True,
                _ => TocLoadOnDemandState::Unknown,
            });
        }
    }
    if let Some(first) = load_values.first().copied() {
        projection.load_on_demand = first;
        projection.conflicting_load_on_demand =
            load_values.len() > 1 || load_values.iter().any(|value| *value != first);
        if projection.conflicting_load_on_demand {
            projection.load_on_demand = TocLoadOnDemandState::Unknown;
        }
    }
    Ok(projection)
}

fn split_dependencies(value: &str) -> ProjectResult<Vec<String>> {
    let mut dependencies = Vec::new();
    for candidate in value.split(|ch: char| ch == ',' || ch.is_ascii_whitespace()) {
        if candidate.is_empty() {
            continue;
        }
        if dependencies.len() >= MAX_DEPENDENCIES || candidate.len() > 128 {
            return Err(budget());
        }
        dependencies.push(candidate.to_owned());
    }
    Ok(dependencies)
}

fn build_plan(
    declarations: &[ProjectPackageInput],
    loaded: &[ProjectLoadedPackage],
    variants: BTreeMap<String, Vec<ProjectPackageVariantReceipt>>,
    retained_variants: BTreeMap<String, BTreeMap<String, Arc<str>>>,
    profile: &ProfileIdentity,
    stop: &AtomicBool,
) -> ProjectResult<ProjectPackageLoadPlan> {
    let declaration_by_name = declarations
        .iter()
        .map(|package| (package.name.clone(), package))
        .collect::<BTreeMap<_, _>>();
    let canonical_by_folded = declarations
        .iter()
        .map(|package| (package.name.to_ascii_lowercase(), package.name.clone()))
        .collect::<BTreeMap<_, _>>();
    let plan_by_name = loaded
        .iter()
        .map(|package| (package.package.clone(), &package.plan))
        .collect::<BTreeMap<_, _>>();

    let mut dependencies = Vec::new();
    let mut load_on_demand = BTreeMap::new();
    let mut issues = Vec::new();
    let mut duplicate_edges = BTreeSet::new();
    for (package, plan) in &plan_by_name {
        checkpoint(stop)?;
        let projection = project_metadata(package, plan)?;
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
        for mut dependency in projection.dependencies {
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
        if !local_file_closure_complete(plan) {
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
    let components =
        strongly_connected_components(declaration_by_name.keys().cloned().collect(), &adjacency);
    let (order_groups, group_by_package, cycle_issues) =
        order_components(&components, &dependencies, &adjacency)?;
    issues.extend(cycle_issues);

    let reachability = classify_reachability(declarations, &dependencies);
    let mut nodes = Vec::with_capacity(declarations.len());
    for declaration in declaration_by_name.values() {
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
            selected_toc: plan.selected_toc().to_owned(),
            selected_plan_digest: plan.digest(),
            variants: variants
                .get(&declaration.name)
                .cloned()
                .ok_or_else(|| invalid("package variant receipt is missing"))?,
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
            let node = node_by_name
                .get(package.as_str())
                .ok_or_else(|| invalid("package order references an unknown node"))?;
            let plan = plan_by_name
                .get(package)
                .ok_or_else(|| invalid("package load plan is missing"))?;
            for record in plan.records().iter().filter(|record| {
                record.selection == LoadSelection::Included
                    && matches!(
                        record.kind,
                        LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
                    )
                    && record.target.is_some()
            }) {
                let target = record
                    .target
                    .as_ref()
                    .ok_or_else(|| invalid("load unit has no target"))?;
                #[derive(Serialize)]
                struct UnitIdentity<'a> {
                    profile: &'static str,
                    package: &'a str,
                    plan: ContentDigest<CanonicalResult>,
                    source_ordinal: u64,
                    document: &'a str,
                    target: &'a str,
                    kind: LoadRecordKind,
                    bootstrap: bool,
                }
                let unit_digest = crate::identity::canonical_digest(
                    "wow-project/package-load-unit/1",
                    &UnitIdentity {
                        profile: PACKAGE_LOAD_PROFILE,
                        package,
                        plan: plan.digest(),
                        source_ordinal: record.ordinal,
                        document: &record.document,
                        target,
                        kind: record.kind,
                        bootstrap: record.bootstrap,
                    },
                    ProjectPhase::Inventory,
                )?;
                units.push(ProjectPackageLoadUnit {
                    ordinal: units.len() as u64,
                    unit_digest,
                    package: package.clone(),
                    order_group: *group_by_package
                        .get(package)
                        .ok_or_else(|| invalid("package order group is missing"))?,
                    source_ordinal: record.ordinal,
                    document: record.document.clone(),
                    target: target.clone(),
                    kind: record.kind,
                    bootstrap: record.bootstrap,
                    reachability: node.reachability,
                    phase: if record.bootstrap {
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
    let digest = crate::identity::canonical_digest(
        "wow-project/package-load-plan/1",
        &Identity {
            profile: PACKAGE_LOAD_PROFILE,
            target_profile_digest,
            packages: &nodes,
            dependencies: &dependencies,
            order_groups: &order_groups,
            units: &units,
            issues: &issues,
            coverage,
        },
        ProjectPhase::Inventory,
    )?;
    let retained_plans = loaded
        .iter()
        .map(|package| (package.package.clone(), package.plan.clone()))
        .collect::<BTreeMap<_, _>>();
    checkpoint(stop)?;
    let plan = ProjectPackageLoadPlan {
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
        retained_plans,
        retained_variants,
    };
    plan.validate_retained_plans()?;
    Ok(plan)
}

fn local_file_closure_complete(plan: &ProjectLoadPlan) -> bool {
    !plan.issues().iter().any(|issue| {
        issue.blocks_complete
            && !matches!(
                issue.kind,
                LoadIssueKind::RequiredDependencyUnresolved
                    | LoadIssueKind::OptionalDependencyUnresolved
            )
    })
}

fn issue(
    kind: ProjectPackageLoadIssueKind,
    package: &str,
    dependency: Option<&str>,
    source: Option<(&str, u64, u64)>,
    blocks_complete: bool,
) -> ProjectPackageLoadIssue {
    ProjectPackageLoadIssue {
        kind,
        package: package.to_owned(),
        dependency: dependency.map(str::to_owned),
        document: source.map(|(document, _, _)| document.to_owned()),
        byte_start: source.map(|(_, start, _)| start),
        byte_end: source.map(|(_, _, end)| end),
        blocks_complete,
    }
}

fn issue_from_dependency(
    kind: ProjectPackageLoadIssueKind,
    dependency: &TocDependencyDeclaration,
    blocks_complete: bool,
) -> ProjectPackageLoadIssue {
    issue(
        kind,
        &dependency.package,
        Some(&dependency.dependency),
        Some((
            &dependency.document,
            dependency.byte_start,
            dependency.byte_end,
        )),
        blocks_complete,
    )
}

fn resolved_adjacency(
    declarations: &[ProjectPackageInput],
    dependencies: &[TocDependencyDeclaration],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency = declarations
        .iter()
        .map(|package| (package.name.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for dependency in dependencies.iter().filter(|dependency| {
        dependency.selection == LoadSelection::Included
            && dependency.resolution == TocDependencyResolution::Resolved
    }) {
        if let Some(resolved) = &dependency.resolved_package {
            adjacency
                .entry(dependency.package.clone())
                .or_default()
                .insert(resolved.clone());
        }
    }
    adjacency
}

fn strongly_connected_components(
    nodes: Vec<String>,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Vec<String>> {
    struct State {
        next_index: usize,
        indexes: BTreeMap<String, usize>,
        lowlinks: BTreeMap<String, usize>,
        stack: Vec<String>,
        on_stack: BTreeSet<String>,
        components: Vec<Vec<String>>,
    }
    fn visit(node: &str, adjacency: &BTreeMap<String, BTreeSet<String>>, state: &mut State) {
        let index = state.next_index;
        state.next_index += 1;
        state.indexes.insert(node.to_owned(), index);
        state.lowlinks.insert(node.to_owned(), index);
        state.stack.push(node.to_owned());
        state.on_stack.insert(node.to_owned());
        if let Some(neighbors) = adjacency.get(node) {
            for neighbor in neighbors {
                if !state.indexes.contains_key(neighbor) {
                    visit(neighbor, adjacency, state);
                    let low = state.lowlinks[node].min(state.lowlinks[neighbor]);
                    state.lowlinks.insert(node.to_owned(), low);
                } else if state.on_stack.contains(neighbor) {
                    let low = state.lowlinks[node].min(state.indexes[neighbor]);
                    state.lowlinks.insert(node.to_owned(), low);
                }
            }
        }
        if state.lowlinks[node] == state.indexes[node] {
            let mut component = Vec::new();
            while let Some(member) = state.stack.pop() {
                state.on_stack.remove(&member);
                let root = member == node;
                component.push(member);
                if root {
                    break;
                }
            }
            component.sort();
            state.components.push(component);
        }
    }

    let mut state = State {
        next_index: 0,
        indexes: BTreeMap::new(),
        lowlinks: BTreeMap::new(),
        stack: Vec::new(),
        on_stack: BTreeSet::new(),
        components: Vec::new(),
    };
    for node in nodes {
        if !state.indexes.contains_key(&node) {
            visit(&node, adjacency, &mut state);
        }
    }
    state.components
}

type OrderedComponents = (
    Vec<ProjectPackageOrderGroup>,
    BTreeMap<String, u64>,
    Vec<ProjectPackageLoadIssue>,
);

fn order_components(
    components: &[Vec<String>],
    dependencies: &[TocDependencyDeclaration],
    adjacency: &BTreeMap<String, BTreeSet<String>>,
) -> ProjectResult<OrderedComponents> {
    let mut component_by_package = BTreeMap::new();
    for (index, component) in components.iter().enumerate() {
        for package in component {
            component_by_package.insert(package.clone(), index);
        }
    }
    let mut successors = vec![BTreeSet::<usize>::new(); components.len()];
    let mut indegree = vec![0usize; components.len()];
    for (dependent, prerequisites) in adjacency {
        let dependent_component = component_by_package[dependent];
        for prerequisite in prerequisites {
            let prerequisite_component = component_by_package[prerequisite];
            if dependent_component != prerequisite_component
                && successors[prerequisite_component].insert(dependent_component)
            {
                indegree[dependent_component] += 1;
            }
        }
    }
    let mut ready = BTreeSet::<(String, usize)>::new();
    for (index, component) in components.iter().enumerate() {
        if indegree[index] == 0 {
            ready.insert((component[0].clone(), index));
        }
    }
    let mut ordered_components = Vec::with_capacity(components.len());
    while let Some((_key, index)) = ready.pop_first() {
        ordered_components.push(index);
        for successor in successors[index].clone() {
            indegree[successor] -= 1;
            if indegree[successor] == 0 {
                ready.insert((components[successor][0].clone(), successor));
            }
        }
    }
    if ordered_components.len() != components.len() {
        return Err(invalid("package component graph is internally cyclic"));
    }

    let mut groups = Vec::with_capacity(components.len());
    let mut group_by_package = BTreeMap::new();
    let mut cycle_issues = Vec::new();
    for component_index in ordered_components {
        let members = components[component_index].clone();
        let self_cycle = members.len() == 1
            && adjacency
                .get(&members[0])
                .is_some_and(|targets| targets.contains(&members[0]));
        let cyclic = members.len() > 1 || self_cycle;
        let cycle_kind = if cyclic {
            let member_set = members.iter().cloned().collect::<BTreeSet<_>>();
            let mut required = false;
            let mut optional = false;
            for dependency in dependencies.iter().filter(|dependency| {
                dependency.resolution == TocDependencyResolution::Resolved
                    && dependency.selection == LoadSelection::Included
                    && member_set.contains(&dependency.package)
                    && dependency
                        .resolved_package
                        .as_ref()
                        .is_some_and(|target| member_set.contains(target))
            }) {
                match dependency.kind {
                    TocDependencyKind::Required => required = true,
                    TocDependencyKind::Optional => optional = true,
                }
            }
            let kind = match (required, optional) {
                (true, false) => ProjectPackageCycleKind::Required,
                (false, true) => ProjectPackageCycleKind::Optional,
                _ => ProjectPackageCycleKind::Mixed,
            };
            cycle_issues.push(issue(
                match kind {
                    ProjectPackageCycleKind::Required => {
                        ProjectPackageLoadIssueKind::RequiredDependencyCycle
                    }
                    ProjectPackageCycleKind::Optional => {
                        ProjectPackageLoadIssueKind::OptionalDependencyCycle
                    }
                    ProjectPackageCycleKind::Mixed => {
                        ProjectPackageLoadIssueKind::MixedDependencyCycle
                    }
                },
                &members[0],
                None,
                None,
                true,
            ));
            Some(kind)
        } else {
            None
        };
        let ordinal = groups.len() as u64;
        for package in &members {
            group_by_package.insert(package.clone(), ordinal);
        }
        groups.push(ProjectPackageOrderGroup {
            ordinal,
            packages: members,
            cyclic,
            cycle_kind,
        });
    }
    Ok((groups, group_by_package, cycle_issues))
}

fn classify_reachability(
    declarations: &[ProjectPackageInput],
    dependencies: &[TocDependencyDeclaration],
) -> BTreeMap<String, ProjectPackageReachability> {
    let mut states = declarations
        .iter()
        .map(|package| {
            (
                package.name.clone(),
                if package.selected_root {
                    ProjectPackageReachability::Reachable
                } else {
                    ProjectPackageReachability::Unreachable
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    loop {
        let mut changed = false;
        for dependency in dependencies.iter().filter(|dependency| {
            dependency.selection == LoadSelection::Included
                && dependency.resolution == TocDependencyResolution::Resolved
        }) {
            let Some(target) = &dependency.resolved_package else {
                continue;
            };
            let dependent = states[&dependency.package];
            if dependent == ProjectPackageReachability::Unreachable {
                continue;
            }
            let candidate = if dependent == ProjectPackageReachability::Reachable
                && dependency.kind == TocDependencyKind::Required
            {
                ProjectPackageReachability::Reachable
            } else {
                ProjectPackageReachability::ConditionallyReachable
            };
            let current = states[target];
            if candidate > current {
                states.insert(target.clone(), candidate);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    states
}

fn package_phase(
    package: &ProjectPackageInput,
    reachability: ProjectPackageReachability,
    load_on_demand: TocLoadOnDemandState,
) -> ProjectPackageLoadPhase {
    if reachability == ProjectPackageReachability::Unreachable
        || load_on_demand == TocLoadOnDemandState::True
    {
        ProjectPackageLoadPhase::Deferred
    } else if reachability == ProjectPackageReachability::ConditionallyReachable
        || load_on_demand == TocLoadOnDemandState::Unknown
    {
        ProjectPackageLoadPhase::OptionalDependencyConditional
    } else if package.selected_root {
        ProjectPackageLoadPhase::Normal
    } else {
        ProjectPackageLoadPhase::DependencyPrerequisite
    }
}

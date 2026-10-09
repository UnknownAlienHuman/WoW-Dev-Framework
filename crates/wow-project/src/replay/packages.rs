//! Complete package-local captured sources, including unreachable package inputs.
//! Dependency closure and analyzer namespace are rebuilt by their existing owner.
use super::{MAX_FILE_BYTES, MAX_FILES, MAX_SOURCE_BYTES, ReplayFile, exhausted, invalid};
use crate::load::{
    ProjectPackageInput, ProjectPackageLoadPlan, ProjectPackageMainPlan, ProjectPackageNode,
    ProjectPackageVariantInput, TocLoadContext, read_retained_packages,
};
use crate::{ProjectInputFile, ProjectResult};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayPackage {
    name: String,
    selected_root: bool,
    selected_toc: String,
    selected_plan_digest: ContentDigest<CanonicalResult>,
    variants: Vec<String>,
    sources: Vec<ReplayFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReplayPackages {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    context: Option<TocLoadContext>,
    packages: Vec<ReplayPackage>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    main_fixture_refs: BTreeMap<String, String>,
    expected_load_digest: ContentDigest<CanonicalResult>,
    expected_main_digest: ContentDigest<CanonicalResult>,
}

fn source_entries<'a>(
    plan: &'a ProjectPackageLoadPlan,
    package: &'a ProjectPackageNode,
) -> ProjectResult<BTreeMap<&'a str, &'a str>> {
    let selected = plan.package_plan(&package.package).ok_or_else(invalid)?;
    let mut sources = BTreeMap::new();
    for source in selected.sources() {
        let text = selected.captured_text(&source.path).ok_or_else(invalid)?;
        if text.len() as u64 != source.byte_length
            || crate::identity::source_digest(text.as_bytes()) != source.content_digest
        {
            return Err(invalid());
        }
        sources.insert(source.path.as_str(), text);
    }
    for variant in &package.variants {
        let text = plan
            .variant_text(&package.package, &variant.toc)
            .ok_or_else(invalid)?;
        if text.len() as u64 != variant.byte_length
            || crate::identity::source_digest(text.as_bytes()) != variant.content_digest
        {
            return Err(invalid());
        }
        if let Some(prior) = sources.insert(variant.toc.as_str(), text)
            && prior != text
        {
            return Err(invalid());
        }
    }
    Ok(sources)
}

impl ReplayPackages {
    pub(super) fn capture(
        plan: &ProjectPackageLoadPlan,
        main: &ProjectPackageMainPlan,
        files: &[ProjectInputFile],
        stop: &AtomicBool,
        count: &mut usize,
        bytes: &mut usize,
    ) -> ProjectResult<Self> {
        main.validate_load_plan(plan)?;
        main.validate_main_files(files)?;
        // Precharge the complete borrowed corpus before any source text copy.
        for package in plan.packages() {
            for text in source_entries(plan, package)?.values() {
                crate::analyzer::checkpoint(stop)?;
                *count = count.checked_add(1).ok_or_else(exhausted)?;
                *bytes = bytes.checked_add(text.len()).ok_or_else(exhausted)?;
                if *count > MAX_FILES || *bytes > MAX_SOURCE_BYTES || text.len() > MAX_FILE_BYTES {
                    return Err(exhausted());
                }
            }
        }
        let context = plan
            .packages()
            .first()
            .and_then(|package| plan.package_plan(&package.package))
            .ok_or_else(invalid)?
            .load_context()
            .cloned();
        let mut packages = Vec::new();
        for package in plan.packages() {
            crate::analyzer::checkpoint(stop)?;
            if plan
                .package_plan(&package.package)
                .ok_or_else(invalid)?
                .load_context()
                != context.as_ref()
            {
                return Err(invalid());
            }
            packages.push(ReplayPackage {
                name: package.package.clone(),
                selected_root: package.selected_root,
                selected_toc: package.selected_toc.clone(),
                selected_plan_digest: package.selected_plan_digest,
                variants: package
                    .variants
                    .iter()
                    .map(|variant| variant.toc.clone())
                    .collect(),
                sources: source_entries(plan, package)?
                    .into_iter()
                    .map(|(path, text)| ReplayFile {
                        path: path.into(),
                        text: text.into(),
                        fixture_ref: None,
                    })
                    .collect(),
            });
        }
        Ok(Self {
            context,
            packages,
            main_fixture_refs: files
                .iter()
                .filter_map(|file| {
                    file.source_fixture_ref()
                        .map(|reference| (file.relative_path().as_str().into(), reference.into()))
                })
                .collect(),
            expected_load_digest: plan.digest(),
            expected_main_digest: main.digest(),
        })
    }

    pub(super) fn sources(&self) -> ProjectResult<impl Iterator<Item = &ReplayFile>> {
        if self.packages.is_empty()
            || self.packages.len() > 64
            || self
                .packages
                .windows(2)
                .any(|pair| pair[0].name >= pair[1].name)
            || self.packages.iter().any(|package| {
                package.sources.is_empty()
                    || package
                        .sources
                        .windows(2)
                        .any(|pair| pair[0].path >= pair[1].path)
                    || package
                        .sources
                        .iter()
                        .any(|file| file.fixture_ref.is_some())
                    || package.variants.is_empty()
                    || package.variants.len() > 32
                    || package.variants.windows(2).any(|pair| pair[0] >= pair[1])
            })
            || self.main_fixture_refs.len() > MAX_FILES
            || self
                .main_fixture_refs
                .iter()
                .any(|(path, reference)| path.len() > 4096 || reference.len() > 4096)
        {
            return Err(invalid());
        }
        Ok(self.packages.iter().flat_map(|package| &package.sources))
    }

    pub(super) fn rebuild(
        &self,
        profile: &ProfileIdentity,
        stop: &AtomicBool,
    ) -> ProjectResult<crate::load::ProjectPackageMainInput> {
        let _ = self.sources()?;
        let mut declarations = Vec::new();
        let mut scopes = BTreeMap::new();
        for (ordinal, package) in self.packages.iter().enumerate() {
            crate::analyzer::checkpoint(stop)?;
            let sources = package
                .sources
                .iter()
                .map(|file| (file.path.as_str(), file.text.as_str()))
                .collect::<BTreeMap<_, _>>();
            let variants = package
                .variants
                .iter()
                .map(|path| {
                    let text = sources.get(path.as_str()).ok_or_else(invalid)?;
                    Ok(ProjectPackageVariantInput::new(
                        crate::disk::ProjectDiskFile::new(path).with_identity(
                            crate::identity::source_digest(text.as_bytes()),
                            text.len() as u64,
                        ),
                        path == &package.selected_toc,
                    ))
                })
                .collect::<ProjectResult<Vec<_>>>()?;
            // Host/logical acquisition roots are not part of package semantics.
            // One synthetic, distinct confined root per exact declared package.
            declarations.push(ProjectPackageInput::new(
                &package.name,
                format!("replay-package-{ordinal}"),
                package.selected_root,
                variants,
            ));
            scopes.insert(package.name.as_str(), sources);
        }
        let loaded =
            read_retained_packages(scopes, &declarations, profile, self.context.as_ref(), stop)?;
        for package in &self.packages {
            if loaded
                .plan()
                .package_plan(&package.name)
                .ok_or_else(invalid)?
                .digest()
                != package.selected_plan_digest
            {
                return Err(invalid());
            }
        }
        let main = loaded.into_namespaced_main()?;
        if main.load_plan().digest() != self.expected_load_digest
            || main.main_plan().digest() != self.expected_main_digest
        {
            return Err(invalid());
        }
        Ok(main)
    }

    pub(super) fn restore_fixture_refs(
        &self,
        files: Vec<ProjectInputFile>,
    ) -> ProjectResult<Vec<ProjectInputFile>> {
        let mut refs = self.main_fixture_refs.clone();
        let files = files
            .into_iter()
            .map(|file| {
                let reference = refs.remove(file.relative_path().as_str());
                ProjectInputFile::declared(
                    file.relative_path().as_str(),
                    file.retained_text(),
                    file.language_kind(),
                    file.role(),
                    reference,
                )
            })
            .collect::<ProjectResult<Vec<_>>>()?;
        if !refs.is_empty() {
            return Err(invalid());
        }
        Ok(files)
    }
}

//! Shared native acquisition of one selected package; no source execution.
use super::*;
use crate::load::{budget_at, read_admitted_toc, read_admitted_toc_for_census, read_retained_toc};

pub(super) struct LoadedSelection {
    pub(super) loaded: ProjectLoadedPackage,
    pub(super) variants: Vec<ProjectPackageVariantReceipt>,
    pub(super) texts: BTreeMap<String, Arc<str>>,
}

pub(super) fn read_selected(
    sources: &PackageSources<'_>,
    package: &ProjectPackageInput,
    profile: &ProfileIdentity,
    context: Option<&TocLoadContext>,
    stop: &AtomicBool,
    capacity: PackageLoadCapacity,
) -> ProjectResult<LoadedSelection> {
    checkpoint(stop)?;
    let selected = package
        .variants
        .iter()
        .find(|variant| variant.selected)
        .ok_or_else(|| invalid("package has no selected TOC variant"))?;
    let input = match sources {
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
            read_retained_toc(selected_sources, &selected.toc, profile, context, stop)?
        }
        PackageSources::Admitted(source) if capacity.is_census() => {
            read_admitted_toc_for_census(source, &package.root, &selected.toc, context, stop)?
        }
        PackageSources::Admitted(source) => {
            read_admitted_toc(source, &package.root, &selected.toc, profile, context, stop)?
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
        let text: Arc<str> = match sources {
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
            PackageSources::Admitted(source) => {
                let bytes =
                    source.load_member(&package.root, &variant.toc, DISK_SOURCE_MAX_BYTES, stop)?;
                let text = std::str::from_utf8(bytes).map_err(|_| {
                    ProjectError::new(
                        ProjectErrorCode::InvalidEncoding,
                        ProjectPhase::Inventory,
                        "unselected TOC variant must contain UTF-8",
                    )
                })?;
                Arc::from(text)
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
        texts.insert(variant.toc.path().to_owned(), text);
    }
    receipts.sort_by(|left, right| left.toc.cmp(&right.toc));
    if let PackageSources::Retained(scopes) = sources
        && scopes[package.name.as_str()].len() != plan.sources().len() + texts.len() - 1
    {
        return Err(invalid(
            "retained package source set includes undeclared variants",
        ));
    }
    Ok(LoadedSelection {
        loaded: ProjectLoadedPackage {
            package: package.name.clone(),
            files,
            plan,
        },
        variants: receipts,
        texts,
    })
}

#[derive(Default)]
pub(super) struct PackageSourceBudget {
    sources: usize,
    bytes: usize,
}

impl PackageSourceBudget {
    pub(super) fn charge(&mut self, selection: &LoadedSelection) -> ProjectResult<()> {
        for source in selection.loaded.plan.sources() {
            self.add(source.byte_length)?;
        }
        for variant in selection
            .variants
            .iter()
            .filter(|variant| !variant.selected)
        {
            self.add(variant.byte_length)?;
        }
        Ok(())
    }

    fn add(&mut self, bytes: u64) -> ProjectResult<()> {
        self.sources = self.sources.checked_add(1).ok_or_else(budget)?;
        self.bytes = self
            .bytes
            .checked_add(usize::try_from(bytes).map_err(|_| budget())?)
            .ok_or_else(budget)?;
        if self.sources > MAX_PACKAGE_SOURCES {
            return Err(budget_at("package retained-source count limit exceeded"));
        }
        if self.bytes > MAX_PACKAGE_SOURCE_BYTES {
            return Err(budget_at("package retained-source byte limit exceeded"));
        }
        Ok(())
    }
}

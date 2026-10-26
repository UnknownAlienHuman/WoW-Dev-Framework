use std::collections::{BTreeMap, BTreeSet};

use wow_project::load::{
    LoadIssue, LoadIssueKind, ProjectLoadPlan, ProjectPackageInput, ProjectPackageLoadIssue,
    ProjectPackageLoadIssueKind, ProjectPackageLoadPlan,
};

use crate::{ExactSourceLocation, ServiceError, ServiceErrorCode, ServiceResult};

use super::{AddonCheckLimitation, AddonCheckLimitationCode};

/// Convert the detailed native Project load/package issues into the finite direct
/// result categories. Package declarations are required because their logical
/// roots are acquisition/source-location facts intentionally excluded from the
/// package semantic/replay identity.
///
/// Normal absence of an optional dependency and duplicate declarations that do
/// not block the native plan are not limitations. Aggregate file-closure issues
/// are omitted because the retained per-package load plans provide the exact
/// source-backed reasons.
pub fn project_package_limitations(
    plan: &ProjectPackageLoadPlan,
    packages: &[ProjectPackageInput],
) -> ServiceResult<Vec<AddonCheckLimitation>> {
    let declarations = validate_package_declarations(plan, packages)?;
    let mut limitations = Vec::new();

    for node in plan.packages() {
        let declaration = declarations.get(node.package.as_str()).ok_or_else(|| {
            invalid_context("package limitation mapping is missing its source declaration")
        })?;
        let package_plan = plan.package_plan(&node.package).ok_or_else(|| {
            invalid_context("package limitation mapping is missing its retained load plan")
        })?;
        append_load_limitations(
            &mut limitations,
            &node.package,
            declaration.root(),
            package_plan,
        )?;
    }

    for issue in plan.issues() {
        let Some(code) = package_issue_code(issue) else {
            continue;
        };
        let declaration = declarations.get(issue.package.as_str()).ok_or_else(|| {
            invalid_context("package issue references an undeclared source package")
        })?;
        let package_plan = plan.package_plan(&issue.package).ok_or_else(|| {
            invalid_context("package issue references a missing retained load plan")
        })?;
        let mut limitation = AddonCheckLimitation::new(code, package_issue_message(issue));
        if let Some(location) = package_issue_location(declaration.root(), package_plan, issue)? {
            limitation = limitation.with_location(location);
        }
        limitations.push(limitation);
    }

    limitations.sort_unstable();
    limitations.dedup();
    Ok(limitations)
}

fn validate_package_declarations<'a>(
    plan: &ProjectPackageLoadPlan,
    packages: &'a [ProjectPackageInput],
) -> ServiceResult<BTreeMap<&'a str, &'a ProjectPackageInput>> {
    if packages.len() != plan.packages().len() {
        return Err(invalid_context(
            "package declarations differ from the selected package plan",
        ));
    }

    let mut declarations = BTreeMap::new();
    for package in packages {
        if declarations.insert(package.name(), package).is_some() {
            return Err(invalid_context(
                "package limitation mapping contains duplicate declarations",
            ));
        }
    }

    let mut seen_plan_packages = BTreeSet::new();
    for node in plan.packages() {
        if !seen_plan_packages.insert(node.package.as_str()) {
            return Err(invalid_context(
                "package limitation mapping contains duplicate plan nodes",
            ));
        }
        let declaration = declarations
            .get(node.package.as_str())
            .ok_or_else(|| invalid_context("selected package plan has no matching declaration"))?;
        if declaration.selected_root() != node.selected_root {
            return Err(invalid_context(
                "package declaration root role differs from the selected plan",
            ));
        }
        let mut selected = declaration
            .variants()
            .iter()
            .filter(|variant| variant.selected());
        let selected_variant = selected
            .next()
            .ok_or_else(|| invalid_context("package declaration has no selected TOC variant"))?;
        if selected.next().is_some() || selected_variant.toc().path() != node.selected_toc {
            return Err(invalid_context(
                "package declaration selected TOC differs from the selected plan",
            ));
        }
    }

    Ok(declarations)
}

fn append_load_limitations(
    output: &mut Vec<AddonCheckLimitation>,
    package: &str,
    root: &str,
    plan: &ProjectLoadPlan,
) -> ServiceResult<()> {
    for issue in plan.issues() {
        let Some(code) = load_issue_code(issue) else {
            continue;
        };
        let location = load_issue_location(root, plan, issue)?;
        output.push(
            AddonCheckLimitation::new(
                code,
                format!("package `{package}`: {}", load_issue_message(issue.kind)),
            )
            .with_location(location),
        );
    }
    Ok(())
}

fn load_issue_code(issue: &LoadIssue) -> Option<AddonCheckLimitationCode> {
    if !issue.blocks_complete {
        return None;
    }
    Some(match issue.kind {
        LoadIssueKind::MissingFile
        | LoadIssueKind::IncludeCycle
        | LoadIssueKind::RepeatedLoad
        | LoadIssueKind::UnknownTocSyntax
        | LoadIssueKind::UnsupportedFileKind => AddonCheckLimitationCode::SourceSelectionIncomplete,
        LoadIssueKind::MissingInterface | LoadIssueKind::ConflictingInterface => {
            AddonCheckLimitationCode::CompatibilityUnresolved
        }
        LoadIssueKind::RequiredDependencyUnresolved => {
            AddonCheckLimitationCode::DependencyResolutionIncomplete
        }
        LoadIssueKind::OptionalDependencyUnresolved => return None,
        LoadIssueKind::LoadContextRequired | LoadIssueKind::LoadConditionUnresolved => {
            AddonCheckLimitationCode::LoadSelectionUnresolved
        }
        LoadIssueKind::InlineLuaNotAnalyzed => {
            AddonCheckLimitationCode::VirtualSourceProjectionUnavailable
        }
        LoadIssueKind::UnknownDirective
        | LoadIssueKind::XmlSemanticsUnresolved
        | LoadIssueKind::UnsupportedXmlNamespace
        | LoadIssueKind::UnsupportedXmlAttributes => {
            AddonCheckLimitationCode::LoadContextUnsupported
        }
    })
}

const fn load_issue_message(kind: LoadIssueKind) -> &'static str {
    match kind {
        LoadIssueKind::MissingFile => "selected source references a missing file",
        LoadIssueKind::IncludeCycle => "selected XML includes form a cycle",
        LoadIssueKind::RepeatedLoad => "selected source repeats a load occurrence",
        LoadIssueKind::MissingInterface => "selected TOC has no usable Interface value",
        LoadIssueKind::ConflictingInterface => "selected TOC contains conflicting Interface values",
        LoadIssueKind::RequiredDependencyUnresolved => {
            "required dependency declarations are unresolved"
        }
        LoadIssueKind::OptionalDependencyUnresolved => {
            "optional dependency declarations are unresolved"
        }
        LoadIssueKind::UnknownDirective => {
            "selected source contains an unsupported load-context directive"
        }
        LoadIssueKind::UnknownTocSyntax => "selected TOC contains unsupported syntax",
        LoadIssueKind::LoadContextRequired => "selected source requires an explicit load context",
        LoadIssueKind::LoadConditionUnresolved => {
            "selected source contains an unresolved load condition"
        }
        LoadIssueKind::UnsupportedFileKind => "selected source references an unsupported file kind",
        LoadIssueKind::XmlSemanticsUnresolved => {
            "selected XML requires unsupported semantic context"
        }
        LoadIssueKind::UnsupportedXmlNamespace => "selected XML uses an unsupported namespace",
        LoadIssueKind::UnsupportedXmlAttributes => "selected XML uses unsupported attributes",
        LoadIssueKind::InlineLuaNotAnalyzed => {
            "selected inline XML Lua is not yet source-projected for analysis"
        }
    }
}

fn package_issue_code(issue: &ProjectPackageLoadIssue) -> Option<AddonCheckLimitationCode> {
    if !issue.blocks_complete {
        return None;
    }
    Some(match issue.kind {
        ProjectPackageLoadIssueKind::MissingRequiredDependency
        | ProjectPackageLoadIssueKind::InvalidDependencyName
        | ProjectPackageLoadIssueKind::SelfDependency
        | ProjectPackageLoadIssueKind::RequiredDependencyCycle
        | ProjectPackageLoadIssueKind::OptionalDependencyCycle
        | ProjectPackageLoadIssueKind::MixedDependencyCycle => {
            AddonCheckLimitationCode::DependencyResolutionIncomplete
        }
        ProjectPackageLoadIssueKind::DependencyConditionUnresolved
        | ProjectPackageLoadIssueKind::ConflictingLoadOnDemand => {
            AddonCheckLimitationCode::LoadSelectionUnresolved
        }
        ProjectPackageLoadIssueKind::MissingOptionalDependency
        | ProjectPackageLoadIssueKind::DuplicateDependency
        | ProjectPackageLoadIssueKind::PackageFileClosurePartial => return None,
    })
}

fn package_issue_message(issue: &ProjectPackageLoadIssue) -> String {
    let dependency = issue.dependency.as_deref().unwrap_or("<unspecified>");
    match issue.kind {
        ProjectPackageLoadIssueKind::MissingRequiredDependency => format!(
            "package `{}` is missing required dependency `{dependency}`",
            issue.package
        ),
        ProjectPackageLoadIssueKind::MissingOptionalDependency => format!(
            "package `{}` is missing optional dependency `{dependency}`",
            issue.package
        ),
        ProjectPackageLoadIssueKind::DependencyConditionUnresolved => format!(
            "package `{}` has an unresolved dependency condition for `{dependency}`",
            issue.package
        ),
        ProjectPackageLoadIssueKind::InvalidDependencyName => format!(
            "package `{}` contains invalid dependency name `{dependency}`",
            issue.package
        ),
        ProjectPackageLoadIssueKind::DuplicateDependency => format!(
            "package `{}` repeats dependency `{dependency}`",
            issue.package
        ),
        ProjectPackageLoadIssueKind::SelfDependency => {
            format!("package `{}` depends on itself", issue.package)
        }
        ProjectPackageLoadIssueKind::RequiredDependencyCycle => format!(
            "package `{}` participates in a required-dependency cycle",
            issue.package
        ),
        ProjectPackageLoadIssueKind::OptionalDependencyCycle => format!(
            "package `{}` participates in an optional-dependency cycle",
            issue.package
        ),
        ProjectPackageLoadIssueKind::MixedDependencyCycle => format!(
            "package `{}` participates in a mixed dependency cycle",
            issue.package
        ),
        ProjectPackageLoadIssueKind::ConflictingLoadOnDemand => format!(
            "package `{}` contains conflicting LoadOnDemand declarations",
            issue.package
        ),
        ProjectPackageLoadIssueKind::PackageFileClosurePartial => format!(
            "package `{}` has an incomplete selected file closure",
            issue.package
        ),
    }
}

fn load_issue_location(
    root: &str,
    plan: &ProjectLoadPlan,
    issue: &LoadIssue,
) -> ServiceResult<ExactSourceLocation> {
    source_location(
        root,
        plan,
        &issue.document,
        issue.byte_start,
        issue.byte_end,
    )
}

fn package_issue_location(
    root: &str,
    plan: &ProjectLoadPlan,
    issue: &ProjectPackageLoadIssue,
) -> ServiceResult<Option<ExactSourceLocation>> {
    match (&issue.document, issue.byte_start, issue.byte_end) {
        (None, None, None) => Ok(None),
        (Some(document), Some(byte_start), Some(byte_end)) => {
            source_location(root, plan, document, byte_start, byte_end).map(Some)
        }
        _ => Err(invalid_context(
            "package issue contains an incomplete exact source location",
        )),
    }
}

fn source_location(
    root: &str,
    plan: &ProjectLoadPlan,
    document: &str,
    byte_start: u64,
    byte_end: u64,
) -> ServiceResult<ExactSourceLocation> {
    let source = plan
        .sources()
        .iter()
        .find(|source| source.path == document)
        .ok_or_else(|| invalid_context("Project issue source is absent from its retained plan"))?;
    if byte_end > source.byte_length {
        return Err(invalid_context(
            "Project issue span exceeds its retained source bytes",
        ));
    }
    let path = if root.is_empty() || root == "." {
        document.to_owned()
    } else {
        format!("{root}/{document}")
    };
    ExactSourceLocation::new(
        path,
        source.content_digest.to_string(),
        byte_start,
        byte_end,
    )
}

fn invalid_context(message: &'static str) -> ServiceError {
    ServiceError::new(ServiceErrorCode::InvalidContext, message)
}

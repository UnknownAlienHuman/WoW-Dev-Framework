//! Manifest-bound lexical measurements, streamed through the existing parsers.
//! Counts are not package closure, graph acceptance, or current-source proof.
mod measure;
mod model;
pub use model::*;

use super::invalid;
use crate::platform_source::{AdmittedPlatformSource, PlatformEntryDisposition, PlatformFileKind};
use crate::{
    ProjectPhase, ProjectResult,
    disk::{checkpoint, validate_path},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};
use wow_core::{CanonicalResult, ContentDigest, CoverageStatus};

pub const PLATFORM_CENSUS_PROFILE: &str = "wow-project/platform-source-census/1";

/// Measure exactly the retained manifest, with no disk reread or include expansion.
/// Each successful document contributes once; refused documents contribute no
/// invented syntax counts. Cancellation aborts the operation, never a partial sum.
pub fn census_platform_source(
    source: &AdmittedPlatformSource,
    selection: &PlatformCensusSelection,
    stop: &AtomicBool,
) -> ProjectResult<PlatformSourceCensus> {
    checkpoint(stop)?;
    source.profile().validate()?;
    validate_path(&selection.package_root)?;
    if !source.profile().roots().iter().any(|root| {
        related(&root.root, &selection.package_root) || related(&selection.package_root, &root.root)
    }) {
        return Err(invalid(
            "census package root does not intersect an admitted root",
        ));
    }
    if let Some(context) = &selection.load_context {
        context.validate()?;
    }
    let selected = source
        .profile()
        .roots()
        .iter()
        .flat_map(|root| root.selected_tocs.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let receipt = source.receipt();
    let entries = receipt
        .inventory()
        .entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut selection_issues = Vec::new();
    for path in &selected {
        checkpoint(stop)?;
        let kind = match entries.get(path) {
            None => Some(CensusSelectionIssueKind::MissingSelectedToc),
            Some(entry) if entry.kind != PlatformFileKind::Toc => {
                Some(CensusSelectionIssueKind::SelectedTocKindMismatch)
            }
            Some(entry)
                if !matches!(entry.disposition, PlatformEntryDisposition::Included { .. }) =>
            {
                Some(CensusSelectionIssueKind::SelectedTocNotIncluded)
            }
            Some(_) => None,
        };
        if let Some(kind) = kind {
            selection_issues.push(CensusSelectionIssue {
                path: (*path).to_owned(),
                kind,
            });
        }
    }
    let mut counts = CensusFileCounts::default();
    let mut packages = BTreeMap::<String, CensusPackageRoot>::new();
    let mut omissions = Vec::new();
    let mut outside_package_root = 0;
    for entry in &receipt.inventory().entries {
        checkpoint(stop)?;
        counts.add(entry)?;
        if let Some((package, _)) = entry
            .path
            .strip_prefix(&selection.package_root)
            .and_then(|suffix| suffix.strip_prefix('/'))
            .and_then(|suffix| suffix.split_once('/'))
        {
            let path = format!("{}/{package}", selection.package_root);
            let package = packages
                .entry(path.clone())
                .or_insert_with(|| CensusPackageRoot {
                    path,
                    ..CensusPackageRoot::default()
                });
            measure::add(&mut package.manifest_entries, 1)?;
            if entry.kind == PlatformFileKind::Toc {
                measure::add(&mut package.declared_tocs, 1)?;
                if matches!(entry.disposition, PlatformEntryDisposition::Included { .. }) {
                    measure::add(&mut package.included_tocs, 1)?;
                }
                if selected.contains(entry.path.as_str()) {
                    measure::add(&mut package.selected_tocs, 1)?;
                }
            }
        } else {
            measure::add(&mut outside_package_root, 1)?;
        }
        if !matches!(entry.disposition, PlatformEntryDisposition::Included { .. }) {
            omissions.push(entry.clone());
        }
    }
    let mut documents = Vec::new();
    let mut xml = CensusSyntaxCounts::default();
    let mut schemas = CensusSyntaxCounts::default();
    let mut selected_tocs = CensusSyntaxCounts::default();
    let mut refused_documents = 0;
    let mut peak_document_source_bytes = 0;
    let mut peak_document_index_json_bytes = 0;
    let mut members = source.raw_inventory(stop)?;
    while let Some(member) = members.next(stop)? {
        let kind = member.kind();
        if !matches!(
            kind,
            PlatformFileKind::Toc | PlatformFileKind::Xml | PlatformFileKind::Schema
        ) {
            continue;
        }
        let outcome = if kind == PlatformFileKind::Toc && !selected.contains(member.path()) {
            CensusDocumentOutcome::UnselectedToc
        } else {
            measure::document(
                &member,
                source.profile().target().reference_profile.interface(),
                selection.load_context.as_ref(),
                stop,
            )?
        };
        match &outcome {
            CensusDocumentOutcome::Measured(counts) => {
                let aggregate = match kind {
                    PlatformFileKind::Toc => &mut selected_tocs,
                    PlatformFileKind::Schema => &mut schemas,
                    _ => &mut xml,
                };
                aggregate.add(counts)?;
                peak_document_source_bytes = peak_document_source_bytes.max(member.byte_length());
                peak_document_index_json_bytes =
                    peak_document_index_json_bytes.max(counts.index_json_bytes);
            }
            CensusDocumentOutcome::Refused(_) => measure::add(&mut refused_documents, 1)?,
            CensusDocumentOutcome::UnselectedToc => {}
        }
        documents.push(CensusDocument {
            path: member.path().to_owned(),
            kind,
            content_digest: member.content_digest(),
            byte_length: member.byte_length(),
            outcome,
        });
    }
    let inventory_partial = receipt.coverage().inventory() != CoverageStatus::Complete;
    let partial = inventory_partial
        || !omissions.is_empty()
        || refused_documents != 0
        || !selection_issues.is_empty();
    let mut result = PlatformSourceCensus {
        profile: PLATFORM_CENSUS_PROFILE,
        source_snapshot_id: receipt.source_snapshot_id().to_owned(),
        source_profile_digest: receipt.profile_digest(),
        content_manifest_digest: receipt.content_manifest_digest(),
        admission_digest: receipt.admission_digest(),
        target: source.profile().target().clone(),
        origin: receipt.inventory().origin.clone(),
        materializer: receipt.inventory().materializer.clone(),
        root_assertions: receipt.inventory().roots.clone(),
        admission_coverage: receipt.coverage().clone(),
        selection: selection.clone(),
        selected_toc_paths: selected.into_iter().map(str::to_owned).collect(),
        selection_issues,
        coverage: if partial {
            CensusCoverage::Partial
        } else {
            CensusCoverage::DeclaredMembersMeasured
        },
        file_counts: counts,
        package_roots: packages.into_values().collect(),
        outside_package_root,
        omissions,
        documents,
        xml,
        schema_xml: schemas,
        selected_tocs,
        refused_documents,
        max_measured_document_source_bytes: peak_document_source_bytes,
        max_measured_document_index_json_bytes: peak_document_index_json_bytes,
        not_evaluated: vec![
            CensusUnevaluated::RootCompleteness,
            CensusUnevaluated::UnselectedTocSyntax,
            CensusUnevaluated::ExpandedLoads,
            CensusUnevaluated::PackageDependencyClosure,
            CensusUnevaluated::SourceHandlesAndEvidence,
            CensusUnevaluated::GraphNodesAndEdges,
            CensusUnevaluated::GraphMetadata,
            CensusUnevaluated::PeakMemory,
            CensusUnevaluated::FullCorpusCapacity,
            CensusUnevaluated::Runtime,
        ],
        digest: ContentDigest::<CanonicalResult>::from_bytes([0; 32]),
    };
    // Bound before the identity helper allocates canonical bytes. The JSON-byte
    // count is not an allocator/peak-RSS estimate.
    measure::encoded_size(&result, stop)?;
    result.digest = crate::identity::canonical_digest(
        PLATFORM_CENSUS_PROFILE,
        &result,
        ProjectPhase::Inventory,
    )?;
    checkpoint(stop)?;
    Ok(result)
}

fn related(path: &str, root: &str) -> bool {
    path == root
        || path
            .strip_prefix(root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

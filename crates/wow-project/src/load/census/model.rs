use super::{super::TocLoadContext, measure};
use crate::{
    ProjectResult,
    platform_source::{
        PlatformAdmissionCoverage, PlatformEntryDisposition, PlatformFileKind,
        PlatformInventoryEntry, PlatformMaterializer, PlatformRootInventory, PlatformSourceOrigin,
        PlatformTarget,
    },
};
use serde::{Deserialize, Serialize};
use wow_core::{CanonicalResult, ContentDigest, SourceContent};

/// Explicit package directory and TOC conditions. No flavor/name inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformCensusSelection {
    pub package_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_context: Option<TocLoadContext>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CensusCoverage {
    DeclaredMembersMeasured,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CensusRefusal {
    DocumentByteLimit,
    InvalidEncoding,
    ParserBudget,
    ParserInvalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CensusSelectionIssueKind {
    MissingSelectedToc,
    SelectedTocNotIncluded,
    SelectedTocKindMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CensusSelectionIssue {
    pub path: String,
    pub kind: CensusSelectionIssueKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "details", rename_all = "snake_case")]
pub enum CensusDocumentOutcome {
    Measured(CensusSyntaxCounts),
    UnselectedToc,
    Refused(CensusRefusal),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CensusDocument {
    pub path: String,
    pub kind: PlatformFileKind,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
    pub outcome: CensusDocumentOutcome,
}

/// Immediate child directories observed in manifest entries, not an enumeration
/// of disk directories or successful package specialization.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CensusPackageRoot {
    pub path: String,
    pub manifest_entries: u64,
    pub declared_tocs: u64,
    pub included_tocs: u64,
    pub selected_tocs: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CensusKindCounts {
    pub lua: u64,
    pub toc: u64,
    pub xml: u64,
    pub schema: u64,
    pub unknown: u64,
}
impl CensusKindCounts {
    fn add(&mut self, kind: PlatformFileKind, count: u64) -> ProjectResult<()> {
        measure::add(
            match kind {
                PlatformFileKind::Lua => &mut self.lua,
                PlatformFileKind::Toc => &mut self.toc,
                PlatformFileKind::Xml => &mut self.xml,
                PlatformFileKind::Schema => &mut self.schema,
                PlatformFileKind::Unknown => &mut self.unknown,
            },
            count,
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CensusFileCounts {
    pub declared: CensusKindCounts,
    pub included: CensusKindCounts,
    pub included_bytes: CensusKindCounts,
    pub excluded: u64,
    pub unsupported: u64,
    pub external: u64,
    pub conflict: u64,
    pub failed: u64,
}
impl CensusFileCounts {
    pub(super) fn add(&mut self, entry: &PlatformInventoryEntry) -> ProjectResult<()> {
        self.declared.add(entry.kind, 1)?;
        match &entry.disposition {
            PlatformEntryDisposition::Included { byte_length, .. } => {
                self.included.add(entry.kind, 1)?;
                self.included_bytes.add(entry.kind, *byte_length)
            }
            PlatformEntryDisposition::Excluded { .. } => measure::add(&mut self.excluded, 1),
            PlatformEntryDisposition::Unsupported { .. } => measure::add(&mut self.unsupported, 1),
            PlatformEntryDisposition::External { .. } => measure::add(&mut self.external, 1),
            PlatformEntryDisposition::Conflict { .. } => measure::add(&mut self.conflict, 1),
            PlatformEntryDisposition::Failed { .. } => measure::add(&mut self.failed, 1),
        }
    }
}

/// Lexical records are counted once per document, never expanded load occurrences.
/// XML index JSON excludes decoded attribute values and inline body text; those
/// retained byte counts are reported separately and do not imply peak memory.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CensusSyntaxCounts {
    pub documents: u64,
    pub lexical_records: u64,
    pub included_file_records: u64,
    pub excluded_records: u64,
    pub unresolved_records: u64,
    pub record_issues: u64,
    pub elements: u64,
    pub attributes: u64,
    pub script_sites: u64,
    pub external_scripts: u64,
    pub reference_scripts: u64,
    pub inline_scripts: u64,
    pub unresolved_scripts: u64,
    pub inline_units: u64,
    pub inline_bytes: u64,
    pub map_segments: u64,
    pub index_json_bytes: u64,
    pub decoded_attribute_value_bytes: u64,
}
impl CensusSyntaxCounts {
    pub(super) fn add(&mut self, other: &Self) -> ProjectResult<()> {
        macro_rules! fields { ($($field:ident),+ $(,)?) => { $(measure::add(&mut self.$field, other.$field)?;)+ }; }
        fields!(
            documents,
            lexical_records,
            included_file_records,
            excluded_records,
            unresolved_records,
            record_issues,
            elements,
            attributes,
            script_sites,
            external_scripts,
            reference_scripts,
            inline_scripts,
            unresolved_scripts,
            inline_units,
            inline_bytes,
            map_segments,
            index_json_bytes,
            decoded_attribute_value_bytes
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CensusUnevaluated {
    RootCompleteness,
    UnselectedTocSyntax,
    ExpandedLoads,
    PackageDependencyClosure,
    SourceHandlesAndEvidence,
    GraphNodesAndEdges,
    GraphMetadata,
    PeakMemory,
    FullCorpusCapacity,
    Runtime,
}

/// Serialize-only receipt; fields are private so edited JSON or counters cannot
/// be substituted for measurements made against an admitted source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlatformSourceCensus {
    pub(super) profile: &'static str,
    pub(super) source_snapshot_id: String,
    pub(super) source_profile_digest: ContentDigest<CanonicalResult>,
    pub(super) content_manifest_digest: ContentDigest<CanonicalResult>,
    pub(super) admission_digest: ContentDigest<CanonicalResult>,
    pub(super) target: PlatformTarget,
    pub(super) origin: PlatformSourceOrigin,
    pub(super) materializer: PlatformMaterializer,
    pub(super) root_assertions: Vec<PlatformRootInventory>,
    pub(super) admission_coverage: PlatformAdmissionCoverage,
    pub(super) selection: PlatformCensusSelection,
    pub(super) selected_toc_paths: Vec<String>,
    pub(super) selection_issues: Vec<CensusSelectionIssue>,
    pub(super) coverage: CensusCoverage,
    pub(super) file_counts: CensusFileCounts,
    pub(super) package_roots: Vec<CensusPackageRoot>,
    pub(super) outside_package_root: u64,
    pub(super) omissions: Vec<PlatformInventoryEntry>,
    pub(super) documents: Vec<CensusDocument>,
    pub(super) xml: CensusSyntaxCounts,
    pub(super) schema_xml: CensusSyntaxCounts,
    pub(super) selected_tocs: CensusSyntaxCounts,
    pub(super) refused_documents: u64,
    pub(super) max_measured_document_source_bytes: u64,
    pub(super) max_measured_document_index_json_bytes: u64,
    pub(super) not_evaluated: Vec<CensusUnevaluated>,
    pub(super) digest: ContentDigest<CanonicalResult>,
}
impl PlatformSourceCensus {
    pub const fn coverage(&self) -> CensusCoverage {
        self.coverage
    }
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
    pub const fn file_counts(&self) -> &CensusFileCounts {
        &self.file_counts
    }
    pub fn package_roots(&self) -> &[CensusPackageRoot] {
        &self.package_roots
    }
    pub fn documents(&self) -> &[CensusDocument] {
        &self.documents
    }
    pub const fn xml(&self) -> &CensusSyntaxCounts {
        &self.xml
    }
    pub const fn selected_tocs(&self) -> &CensusSyntaxCounts {
        &self.selected_tocs
    }
}

//! Conservative callable view of the existing native documentation model.
//! Raw metadata and every candidate remain retained. No restriction inference,
//! source execution, correction or alias substitution. Callable absence authority
//! requires exact manifested TOC closure and loss-free in-domain projection.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_core::ReferenceGenerationId;

use crate::native::{DocumentationDocument, NativeError, NativeErrorCode, Span, source_digest};
use crate::native_model::{SystemOwner, normalize_document};
use crate::{
    CoverageStatus, ReferenceConflict, ReferencePartition, ReferenceRecord, ReferenceRecordKind,
    ReferenceView,
};

pub const NATIVE_API_PARTITION: &str = "reference.native.apidoc.api";
pub const NATIVE_VIEW_PROFILE: &str = "wow-reference/native-callable-view/1";
pub const NATIVE_VIEW_AUTHORITY_PROFILE: &str = "wow-reference/native-callable-view/2";

/// Caller-supplied corpus closure evidence. The reference owner still downgrades
/// to Partial when admitted documents or in-domain projection results are lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NativeCallableCorpus {
    ExplicitPartial,
    ManifestToc {
        declared_documents: u64,
        selected_documents: usize,
        source_closure_complete: bool,
    },
}

impl NativeCallableCorpus {
    #[must_use]
    pub const fn explicit_partial() -> Self {
        Self::ExplicitPartial
    }

    #[must_use]
    pub const fn manifest_toc(
        declared_documents: u64,
        selected_documents: usize,
        source_closure_complete: bool,
    ) -> Self {
        Self::ManifestToc {
            declared_documents,
            selected_documents,
            source_closure_complete,
        }
    }

    #[must_use]
    pub const fn profile(self) -> &'static str {
        match self {
            Self::ExplicitPartial => NATIVE_VIEW_PROFILE,
            Self::ManifestToc { .. } => NATIVE_VIEW_AUTHORITY_PROFILE,
        }
    }

    fn complete_for(self, admitted_documents: usize) -> bool {
        match self {
            Self::ExplicitPartial => false,
            Self::ManifestToc {
                declared_documents,
                selected_documents,
                source_closure_complete,
            } => {
                source_closure_complete
                    && declared_documents > 0
                    && usize::try_from(declared_documents)
                        .is_ok_and(|declared| declared == selected_documents)
                    && selected_documents == admitted_documents
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeViewSource {
    pub id: String,
    pub path: String,
    pub sha256: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeViewIssue {
    pub code: &'static str,
    pub path: String,
    pub sha256: String,
    pub span: Span,
}

/// Retain all candidates, including conflict losers (there is no chosen winner).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeViewProjection {
    pub schema: &'static str,
    pub view: ReferenceView,
    pub candidates: Vec<ReferenceRecord>,
    pub sources: Vec<NativeViewSource>,
    pub issues: Vec<NativeViewIssue>,
    pub negative_authority: bool,
}

fn error(code: NativeErrorCode) -> NativeError {
    NativeError { code, span: None }
}
fn checkpoint(stop: &AtomicBool) -> Result<(), NativeError> {
    if stop.load(Ordering::Acquire) {
        Err(error(NativeErrorCode::Cancelled))
    } else {
        Ok(())
    }
}

/// Project only exact global/namespace callable declarations. ScriptObject
/// methods require their separate receiver contract and remain outside this
/// partition's domain. A manifested corpus becomes Complete only when the exact
/// TOC closes over every generated-API manifest member, every selected document
/// is admitted, and no in-domain normalization/payload/record loss occurs.
/// `selection` binds the caller's exact source/profile universe. The returned
/// generation also binds emitted partition/conflict content, without a hash cycle.
pub fn project_callables(
    documents: &[DocumentationDocument],
    environment: &str,
    selection: ReferenceGenerationId,
    corpus: NativeCallableCorpus,
    stop: &AtomicBool,
) -> Result<NativeViewProjection, NativeError> {
    checkpoint(stop)?;
    if documents.is_empty()
        || documents.len() > 1024
        || environment.is_empty()
        || environment.len() > 128
        || environment.chars().any(char::is_control)
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    let total_bytes = documents
        .iter()
        .try_fold(0usize, |total, doc| total.checked_add(doc.source_bytes()))
        .ok_or_else(|| error(NativeErrorCode::Limit))?;
    let registrations = documents
        .iter()
        .try_fold(0usize, |total, doc| {
            total.checked_add(doc.registrations().len())
        })
        .ok_or_else(|| error(NativeErrorCode::Limit))?;
    if total_bytes > 16 * 1024 * 1024 || registrations > 65_536 {
        return Err(error(NativeErrorCode::Limit));
    }
    let revision = documents[0].revision();
    let projection_profile = corpus.profile();
    let mut ordered = documents.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|doc| doc.path());
    if ordered
        .windows(2)
        .any(|pair| pair[0].path() == pair[1].path())
        || ordered.iter().any(|doc| doc.revision() != revision)
    {
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    let mut grouped: BTreeMap<String, Vec<ReferenceRecord>> = BTreeMap::new();
    let mut sources = Vec::new();
    let mut issues = Vec::new();
    let mut candidate_bytes = 0usize;
    let mut examined_functions = 0usize;
    for doc in ordered {
        checkpoint(stop)?;
        let normalized = normalize_document(doc);
        for (registration, normalized) in doc.registrations().iter().zip(normalized.systems) {
            checkpoint(stop)?;
            let mut omit = |code, span| {
                issues.push(NativeViewIssue {
                    code,
                    path: doc.path().to_owned(),
                    sha256: doc.sha256().to_owned(),
                    span,
                })
            };
            let system = match normalized {
                Ok(system) => system,
                Err(failure) => {
                    omit("normalization_rejected", failure.span);
                    continue;
                }
            };
            if system
                .environment
                .is_some_and(|e| e != "All" && e != environment)
            {
                omit("environment_not_selected", registration.value.span);
                continue;
            }
            let namespace = match system.owner {
                SystemOwner::Global => None,
                SystemOwner::Namespace(namespace) => Some(namespace),
                SystemOwner::ScriptObject(_) => {
                    omit("script_object_requires_receiver_contract", system.raw.span);
                    continue;
                }
            };
            for function in &system.functions {
                checkpoint(stop)?;
                examined_functions += 1;
                if examined_functions > 65_536 {
                    return Err(error(NativeErrorCode::Limit));
                }
                let key = match namespace {
                    Some(namespace) => format!("function:{namespace}.{}", function.name),
                    None => format!("function:{}", function.name),
                };
                let source_bytes = crate::wire_json::canonical_json_bytes(&(
                    projection_profile,
                    revision,
                    doc.path(),
                    doc.sha256(),
                    function.raw.span,
                ))
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
                let id = format!("native-apidoc:{}", source_digest(&source_bytes));
                let payload = crate::wire_json::canonical_json_bytes(function.raw)
                    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
                if payload.len() > 65_536 {
                    omit("callable_payload_limit", function.raw.span);
                    continue;
                }
                candidate_bytes = candidate_bytes
                    .checked_add(payload.len())
                    .ok_or_else(|| error(NativeErrorCode::Limit))?;
                if candidate_bytes > 32 * 1024 * 1024 {
                    return Err(error(NativeErrorCode::Limit));
                }
                let payload = String::from_utf8(payload)
                    .map_err(|_| error(NativeErrorCode::InvalidEncoding))?;
                let record = match ReferenceRecord::new(
                    &key,
                    ReferenceRecordKind::Api,
                    payload,
                    vec![id.clone()],
                    vec![],
                ) {
                    Ok(record) => record,
                    Err(_) => {
                        omit("callable_record_rejected", function.raw.span);
                        continue;
                    }
                };
                sources.push(NativeViewSource {
                    id,
                    path: doc.path().to_owned(),
                    sha256: doc.sha256().to_owned(),
                    span: function.raw.span,
                });
                grouped.entry(key).or_default().push(record);
            }
        }
    }
    let mut records = Vec::new();
    let mut candidates = Vec::new();
    let mut conflicts = Vec::new();
    for (key, group) in grouped {
        checkpoint(stop)?;
        if group.len() == 1 {
            records.push(group[0].clone());
        } else {
            let digests = group
                .iter()
                .map(ReferenceRecord::digest)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
            let ids: BTreeSet<String> = group
                .iter()
                .flat_map(|record| record.source_ids().iter().map(|id| id.to_string()))
                .collect();
            conflicts.push(
                ReferenceConflict::new(
                    NATIVE_API_PARTITION,
                    key,
                    digests,
                    ids.into_iter().collect(),
                )
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
            );
        }
        candidates.extend(group);
    }
    let has_in_domain_loss = issues.iter().any(|issue| {
        !matches!(
            issue.code,
            "environment_not_selected" | "script_object_requires_receiver_contract"
        )
    });
    let negative_authority = corpus.complete_for(documents.len()) && !has_in_domain_loss;
    let coverage = if negative_authority {
        CoverageStatus::Complete
    } else {
        CoverageStatus::Partial
    };
    let partition = ReferencePartition::new(NATIVE_API_PARTITION, coverage, records)
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    let generation = ReferenceGenerationId::derive(&(
        projection_profile,
        selection,
        environment,
        corpus,
        &partition,
        &conflicts,
        &issues,
    ))
    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    let view = ReferenceView::new(generation.to_string(), vec![partition], conflicts)
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    checkpoint(stop)?;
    Ok(NativeViewProjection {
        schema: projection_profile,
        view,
        candidates,
        sources,
        issues,
        negative_authority,
    })
}

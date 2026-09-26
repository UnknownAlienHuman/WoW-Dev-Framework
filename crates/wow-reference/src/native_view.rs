//! Conservative callable and restriction views of the existing native documentation model.
//! Raw metadata and every candidate remain retained. No source execution,
//! correction or alias substitution. Callable absence authority requires exact
//! manifested TOC closure and loss-free in-domain projection. Restriction facts
//! are positive source observations only and never establish runtime safety.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_core::ReferenceGenerationId;

use crate::native::{
    DocumentationDocument, NativeError, NativeErrorCode, RawKey, RawKind, Span, source_digest,
};
use crate::native_model::{CallableFact, FieldFact, SystemOwner, normalize_document, object};
use crate::{
    CoverageStatus, ReferenceConflict, ReferencePartition, ReferenceRecord, ReferenceRecordKind,
    ReferenceView, RestrictionFacet, RestrictionState,
};

pub const NATIVE_API_PARTITION: &str = "reference.native.apidoc.api";
pub const NATIVE_RESTRICTION_PARTITION: &str = "reference.native.apidoc.restriction";
pub const NATIVE_VIEW_PROFILE: &str = "wow-reference/native-callable-view/1";
pub const NATIVE_VIEW_AUTHORITY_PROFILE: &str = "wow-reference/native-callable-view/3";
pub const NATIVE_SECRET_RETURN_PAYLOAD: &str =
    "return_position:1;applicability:unconditional_source";
pub const NATIVE_ACCESS_PREDICATE_ENTITY: &str = "function:canaccessvalue";
pub const NATIVE_ACCESS_PREDICATE_PAYLOAD: &str =
    "predicate:access_single;argument_position:1;result:true;scope:immediate_caller";

/// Caller-supplied corpus closure evidence. The reference owner still downgrades
/// API absence to Partial when admitted documents or in-domain projection results
/// are lost. Restriction facts remain Partial even for a closed corpus because
/// only the first reviewed positive facet family is normalized.
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

    #[must_use]
    pub const fn carries_restriction_facts(self) -> bool {
        matches!(self, Self::ManifestToc { .. })
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
    /// Negative authority applies only to the native API callable partition.
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

/// Project exact global/namespace callable declarations. Manifest-selected input
/// also projects the reviewed positive `SecretReturns=true` first-return facet
/// and the exact global `canaccessvalue(value)` predicate contract. ScriptObject
/// methods require their separate receiver contract and remain outside these
/// partitions. The callable partition becomes Complete only when the exact TOC
/// closes over every generated-API manifest member, every selected document is
/// admitted, and no callable-domain projection loss occurs.
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

    let mut api_grouped: BTreeMap<String, Vec<ReferenceRecord>> = BTreeMap::new();
    let mut restriction_grouped: BTreeMap<String, Vec<ReferenceRecord>> = BTreeMap::new();
    let mut sources = Vec::new();
    let mut issues = Vec::new();
    let mut candidate_bytes = 0usize;
    let mut examined_functions = 0usize;
    for doc in ordered {
        checkpoint(stop)?;
        let normalized = normalize_document(doc);
        for (registration, normalized) in doc.registrations().iter().zip(normalized.systems) {
            checkpoint(stop)?;
            let system = match normalized {
                Ok(system) => system,
                Err(failure) => {
                    issues.push(issue("normalization_rejected", doc, failure.span));
                    continue;
                }
            };
            if system
                .environment
                .is_some_and(|e| e != "All" && e != environment)
            {
                issues.push(issue(
                    "environment_not_selected",
                    doc,
                    registration.value.span,
                ));
                continue;
            }
            let namespace = match system.owner {
                SystemOwner::Global => None,
                SystemOwner::Namespace(namespace) => Some(namespace),
                SystemOwner::ScriptObject(_) => {
                    issues.push(issue(
                        "script_object_requires_receiver_contract",
                        doc,
                        system.raw.span,
                    ));
                    continue;
                }
            };
            for function in &system.functions {
                checkpoint(stop)?;
                examined_functions += 1;
                if examined_functions > 65_536 {
                    return Err(error(NativeErrorCode::Limit));
                }
                let key = callable_key(namespace, function.name);
                let api_source_id = source_id(
                    projection_profile,
                    "callable",
                    revision,
                    doc,
                    function.raw.span,
                )?;
                let payload = crate::wire_json::canonical_json_bytes(function.raw)
                    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
                if payload.len() > 65_536 {
                    issues.push(issue("callable_payload_limit", doc, function.raw.span));
                    continue;
                }
                add_candidate_bytes(&mut candidate_bytes, payload.len())?;
                let payload = String::from_utf8(payload)
                    .map_err(|_| error(NativeErrorCode::InvalidEncoding))?;
                let record = match ReferenceRecord::new(
                    &key,
                    ReferenceRecordKind::Api,
                    payload,
                    vec![api_source_id.clone()],
                    vec![],
                ) {
                    Ok(record) => record,
                    Err(_) => {
                        issues.push(issue("callable_record_rejected", doc, function.raw.span));
                        continue;
                    }
                };
                sources.push(NativeViewSource {
                    id: api_source_id,
                    path: doc.path().to_owned(),
                    sha256: doc.sha256().to_owned(),
                    span: function.raw.span,
                });
                api_grouped.entry(key.clone()).or_default().push(record);

                if corpus.carries_restriction_facts() {
                    project_restrictions(
                        projection_profile,
                        revision,
                        doc,
                        namespace,
                        function,
                        &key,
                        &mut restriction_grouped,
                        &mut sources,
                        &mut issues,
                        &mut candidate_bytes,
                    )?;
                }
            }
        }
    }

    let (api_records, mut candidates, mut conflicts) =
        materialize(NATIVE_API_PARTITION, api_grouped, stop)?;
    let has_api_loss = issues.iter().any(|issue| blocks_api_authority(issue.code));
    let negative_authority = corpus.complete_for(documents.len()) && !has_api_loss;
    let api_coverage = if negative_authority {
        CoverageStatus::Complete
    } else {
        CoverageStatus::Partial
    };
    let api_partition = ReferencePartition::new(NATIVE_API_PARTITION, api_coverage, api_records)
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    let mut partitions = vec![api_partition];

    if corpus.carries_restriction_facts() {
        let (restriction_records, restriction_candidates, restriction_conflicts) =
            materialize(NATIVE_RESTRICTION_PARTITION, restriction_grouped, stop)?;
        candidates.extend(restriction_candidates);
        conflicts.extend(restriction_conflicts);
        partitions.push(
            ReferencePartition::new(
                NATIVE_RESTRICTION_PARTITION,
                CoverageStatus::Partial,
                restriction_records,
            )
            .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
        );
    }

    let generation = ReferenceGenerationId::derive(&(
        projection_profile,
        selection,
        environment,
        corpus,
        &partitions,
        &conflicts,
        &issues,
    ))
    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    let view = ReferenceView::new(generation.to_string(), partitions, conflicts)
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

#[allow(clippy::too_many_arguments)]
fn project_restrictions(
    projection_profile: &str,
    revision: &str,
    doc: &DocumentationDocument,
    namespace: Option<&str>,
    function: &CallableFact<'_>,
    key: &str,
    grouped: &mut BTreeMap<String, Vec<ReferenceRecord>>,
    sources: &mut Vec<NativeViewSource>,
    issues: &mut Vec<NativeViewIssue>,
    candidate_bytes: &mut usize,
) -> Result<(), NativeError> {
    let fields = object(function.raw).map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    for field in function.raw.fields().into_iter().flatten() {
        if matches!(&field.key, RawKey::Name(name) if name.starts_with("SecretReturns") && name != "SecretReturns")
        {
            issues.push(issue(
                "restriction_conditional_secret_return_unsupported",
                doc,
                field.value.span,
            ));
        }
    }

    if let Some(secret_returns) = fields.get("SecretReturns") {
        match &secret_returns.kind {
            RawKind::Boolean(false) => {}
            RawKind::Boolean(true) if function.returns.is_empty() => issues.push(issue(
                "restriction_secret_return_without_slot",
                doc,
                secret_returns.span,
            )),
            RawKind::Boolean(true) => {
                let id = source_id(
                    projection_profile,
                    "secret-return",
                    revision,
                    doc,
                    secret_returns.span,
                )?;
                let facet = RestrictionFacet::new(
                    "secret.return",
                    RestrictionState::Restricted,
                    vec![id.clone()],
                )
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
                add_candidate_bytes(candidate_bytes, NATIVE_SECRET_RETURN_PAYLOAD.len())?;
                let record = ReferenceRecord::new(
                    key,
                    ReferenceRecordKind::Restriction,
                    NATIVE_SECRET_RETURN_PAYLOAD,
                    vec![id.clone()],
                    vec![facet],
                )
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
                grouped.entry(key.to_owned()).or_default().push(record);
                sources.push(NativeViewSource {
                    id,
                    path: doc.path().to_owned(),
                    sha256: doc.sha256().to_owned(),
                    span: secret_returns.span,
                });
            }
            _ => issues.push(issue(
                "restriction_secret_return_shape_unsupported",
                doc,
                secret_returns.span,
            )),
        }
    }

    if namespace.is_none() && function.name == "canaccessvalue" {
        if access_predicate_shape(function) {
            let id = source_id(
                projection_profile,
                "access-predicate",
                revision,
                doc,
                function.raw.span,
            )?;
            let facet = RestrictionFacet::new(
                "secret.predicate",
                RestrictionState::Allowed,
                vec![id.clone()],
            )
            .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
            add_candidate_bytes(candidate_bytes, NATIVE_ACCESS_PREDICATE_PAYLOAD.len())?;
            let record = ReferenceRecord::new(
                NATIVE_ACCESS_PREDICATE_ENTITY,
                ReferenceRecordKind::Restriction,
                NATIVE_ACCESS_PREDICATE_PAYLOAD,
                vec![id.clone()],
                vec![facet],
            )
            .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
            grouped
                .entry(NATIVE_ACCESS_PREDICATE_ENTITY.to_owned())
                .or_default()
                .push(record);
            sources.push(NativeViewSource {
                id,
                path: doc.path().to_owned(),
                sha256: doc.sha256().to_owned(),
                span: function.raw.span,
            });
        } else {
            issues.push(issue(
                "restriction_access_predicate_shape_unsupported",
                doc,
                function.raw.span,
            ));
        }
    }
    Ok(())
}

fn access_predicate_shape(function: &CallableFact<'_>) -> bool {
    function.arguments.len() == 1
        && function.returns.len() == 1
        && exact_field(&function.arguments[0], "LuaValueReference")
        && exact_field(&function.returns[0], "bool")
}

fn exact_field(field: &FieldFact<'_>, type_name: &str) -> bool {
    field.type_name == type_name && field.nilable == Some(false)
}

fn callable_key(namespace: Option<&str>, name: &str) -> String {
    match namespace {
        Some(namespace) => format!("function:{namespace}.{name}"),
        None => format!("function:{name}"),
    }
}

fn issue(code: &'static str, doc: &DocumentationDocument, span: Span) -> NativeViewIssue {
    NativeViewIssue {
        code,
        path: doc.path().to_owned(),
        sha256: doc.sha256().to_owned(),
        span,
    }
}

fn source_id(
    profile: &str,
    kind: &str,
    revision: &str,
    doc: &DocumentationDocument,
    span: Span,
) -> Result<String, NativeError> {
    let bytes = crate::wire_json::canonical_json_bytes(&(
        profile,
        kind,
        revision,
        doc.path(),
        doc.sha256(),
        span,
    ))
    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
    Ok(format!("native-apidoc:{}", source_digest(&bytes)))
}

fn add_candidate_bytes(total: &mut usize, bytes: usize) -> Result<(), NativeError> {
    *total = total
        .checked_add(bytes)
        .ok_or_else(|| error(NativeErrorCode::Limit))?;
    if *total > 32 * 1024 * 1024 {
        return Err(error(NativeErrorCode::Limit));
    }
    Ok(())
}

type MaterializedRecords = (
    Vec<ReferenceRecord>,
    Vec<ReferenceRecord>,
    Vec<ReferenceConflict>,
);

fn materialize(
    partition_id: &'static str,
    grouped: BTreeMap<String, Vec<ReferenceRecord>>,
    stop: &AtomicBool,
) -> Result<MaterializedRecords, NativeError> {
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
                ReferenceConflict::new(partition_id, key, digests, ids.into_iter().collect())
                    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
            );
        }
        candidates.extend(group);
    }
    Ok((records, candidates, conflicts))
}

fn blocks_api_authority(code: &str) -> bool {
    !matches!(
        code,
        "environment_not_selected" | "script_object_requires_receiver_contract"
    ) && !code.starts_with("restriction_")
}

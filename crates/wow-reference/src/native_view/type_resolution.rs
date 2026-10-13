//! Bounded exact-name lookup for the selected native callable type overlay.
//! This is Reference lookup, not annotation syntax lowering.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use crate::native::{DocumentationDocument, NativeError, NativeErrorCode, Span};
use crate::native_corrections::{
    Application, CorrectedCorpus, CorrectionError, Lane, Projection, Property, Status, Target,
    ValidatedCorrections, Value,
};
use crate::native_model::{SystemFacts, SystemOwner, TableFact, normalize_document};

pub const PROFILE: &str = "wow-reference/native-callable-type-resolution/1";
const MAX_TYPES: usize = 65_536;
const MAX_FIELDS: usize = 131_072;
const MAX_CANDIDATES_PER_NAME: usize = 32;
const MAX_CANDIDATES_PER_FIELD: usize = 32;
const MAX_CORRECTIONS_PER_CALLABLE: usize = 256;
const MAX_CATALOG_BYTES: usize = 8 * 1024 * 1024;
const MAX_OVERLAY_BYTES: usize = 8 * 1024 * 1024;

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
fn charge(total: &mut usize, amount: usize, limit: usize) -> Result<(), NativeError> {
    *total = total
        .checked_add(amount)
        .ok_or_else(|| error(NativeErrorCode::Limit))?;
    if *total > limit {
        return Err(error(NativeErrorCode::Limit));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TypeTarget {
    pub kind: &'static str,
    pub original_name: String,
    pub effective_name: String,
    pub path: String,
    pub sha256: String,
    pub span: Span,
    pub registration: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub owner_correction_applications: Vec<CorrectionOutcome>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStatus {
    Resolved,
    Unresolved,
    Ambiguous,
    Unsupported,
    CorrectionBlocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FieldResolution {
    pub lane: Lane,
    pub ordinal: usize,
    pub name: String,
    pub original_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_type: Option<String>,
    pub status: ResolutionStatus,
    pub candidate_count: usize,
    pub candidates_truncated: bool,
    pub candidates: Vec<TypeTarget>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub correction_application_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NativeCallableTypeResolution {
    pub schema: &'static str,
    pub catalog_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_set_id: Option<String>,
    /// Full per-callable correction outcomes are kept once, not copied per field.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub correction_applications: Vec<CorrectionOutcome>,
    pub fields: Vec<FieldResolution>,
    #[serde(skip)]
    encoded_len: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CorrectionOutcome {
    pub correction_id: String,
    pub target: Target,
    pub status: Status,
    pub reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_source_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_raw_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
}

impl From<&Application> for CorrectionOutcome {
    fn from(application: &Application) -> Self {
        Self {
            correction_id: application.correction_id.clone(),
            target: application.target.clone(),
            status: application.status,
            reason: application.reason,
            observed_source_sha256: application.observed_source_sha256.clone(),
            observed_raw_sha256: application.observed_raw_sha256.clone(),
            span: application.span,
            before: application.before.clone(),
            after: application.after.clone(),
        }
    }
}
impl NativeCallableTypeResolution {
    /// Exact canonical JSON byte length, measured and capped by the owner before retention.
    pub const fn encoded_len(&self) -> usize {
        self.encoded_len
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct TypeCatalogIdentity<'a> {
    schema: &'static str,
    correction_set_id: Option<&'a str>,
    targets: &'a [TypeTarget],
}

#[derive(Clone, Debug)]
struct NameBucket {
    count: usize,
    candidates: Vec<TypeTarget>,
}

#[derive(Clone, Debug)]
pub(super) struct TypeCatalog {
    id: String,
    correction_set_id: Option<String>,
    correction_applications: Vec<CorrectionOutcome>,
    names: BTreeMap<String, NameBucket>,
}

impl TypeCatalog {
    pub(super) fn build<'a>(
        documents: &'a [DocumentationDocument],
        environment: &str,
        corrections: Option<&'a ValidatedCorrections>,
        stop: &AtomicBool,
    ) -> Result<(Self, Option<CorrectedCorpus<'a>>), NativeError> {
        checkpoint(stop)?;
        let corrected = corrections
            .map(|set| {
                crate::native_corrections::apply_to_documents(documents, environment, set, stop)
                    .map_err(|failure| {
                        error(match failure {
                            CorrectionError::Cancelled => NativeErrorCode::Cancelled,
                            CorrectionError::Limit => NativeErrorCode::Limit,
                            _ => NativeErrorCode::InvalidIdentity,
                        })
                    })
            })
            .transpose()?;
        let mut ordered_documents = documents.iter().collect::<Vec<_>>();
        ordered_documents.sort_by_key(|document| document.path());
        let report = corrected
            .as_ref()
            .map(|corpus| corpus.report.applications.as_slice())
            .unwrap_or(&[]);
        let mut report_bytes = 0usize;
        let mut application_bytes = Vec::with_capacity(report.len());
        for application in report {
            checkpoint(stop)?;
            let bytes = crate::wire_json::canonical_json_bytes(application)
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
            let size = bytes.len();
            charge(&mut report_bytes, size, MAX_CATALOG_BYTES)?;
            application_bytes.push(size);
        }
        charge(
            &mut report_bytes,
            128usize
                .checked_add(corrections.map_or(0, |set| set.id().len()))
                .ok_or_else(|| error(NativeErrorCode::Limit))?,
            MAX_CATALOG_BYTES,
        )?;
        let correction_set_id = corrections.map(|set| set.id().to_owned());
        let mut auxiliary_index_bytes = 0usize;
        for application in report {
            checkpoint(stop)?;
            if matches!(&application.target.projection, Projection::WidgetOwner) {
                let path_bytes = application
                    .target
                    .path
                    .len()
                    .checked_mul(6)
                    .ok_or_else(|| error(NativeErrorCode::Limit))?;
                charge(
                    &mut auxiliary_index_bytes,
                    512usize
                        .checked_add(path_bytes)
                        .ok_or_else(|| error(NativeErrorCode::Limit))?,
                    MAX_CATALOG_BYTES,
                )?;
            }
        }
        for document in &ordered_documents {
            checkpoint(stop)?;
            for system in normalize_document(document).systems.into_iter().flatten() {
                checkpoint(stop)?;
                if selected_environment(&system, environment)
                    && matches!(system.owner, SystemOwner::ScriptObject(_))
                {
                    let path_bytes = document
                        .path()
                        .len()
                        .checked_mul(6)
                        .ok_or_else(|| error(NativeErrorCode::Limit))?;
                    charge(
                        &mut auxiliary_index_bytes,
                        512usize
                            .checked_add(path_bytes)
                            .ok_or_else(|| error(NativeErrorCode::Limit))?,
                        MAX_CATALOG_BYTES,
                    )?;
                }
            }
        }
        let mut estimated_bytes = report_bytes;
        charge(
            &mut estimated_bytes,
            auxiliary_index_bytes,
            MAX_CATALOG_BYTES,
        )?;
        let mut owner_application_index: BTreeMap<(String, usize), Vec<usize>> = BTreeMap::new();
        for (index, application) in report.iter().enumerate() {
            checkpoint(stop)?;
            if matches!(&application.target.projection, Projection::WidgetOwner) {
                owner_application_index
                    .entry((
                        application.target.path.clone(),
                        application.target.registration,
                    ))
                    .or_default()
                    .push(index);
            }
        }
        let mut original_owners = BTreeMap::new();
        for document in &ordered_documents {
            checkpoint(stop)?;
            for system in normalize_document(document).systems.into_iter().flatten() {
                checkpoint(stop)?;
                if selected_environment(&system, environment)
                    && let SystemOwner::ScriptObject(name) = system.owner
                {
                    original_owners.insert(
                        (document.path().to_owned(), system.registration_ordinal),
                        name,
                    );
                }
            }
        }
        let mut targets = Vec::new();
        let mut type_count = 0usize;
        if let Some(corpus) = &corrected {
            for (document, system) in corpus.systems() {
                checkpoint(stop)?;
                if !selected_environment(system, environment) {
                    continue;
                }
                let key = (document.path().to_owned(), system.registration_ordinal);
                let original_owner = original_owners.get(&key).copied();
                let owner_applications = owner_application_index
                    .get(&key)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                preflight_system(
                    document,
                    system,
                    original_owner,
                    OwnerApplicationPreflight {
                        indices: owner_applications,
                        applications: &corpus.report.applications,
                        encoded_lengths: &application_bytes,
                    },
                    &mut type_count,
                    &mut estimated_bytes,
                    stop,
                )?;
            }
            targets.reserve(type_count);
            for (document, system) in corpus.systems() {
                checkpoint(stop)?;
                if !selected_environment(system, environment) {
                    continue;
                }
                let key = (document.path().to_owned(), system.registration_ordinal);
                let original_owner = original_owners.get(&key).copied();
                let owner_applications = owner_application_index
                    .get(&key)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                collect_system(
                    document,
                    system,
                    original_owner,
                    owner_applications,
                    &corpus.report.applications,
                    &mut targets,
                    stop,
                )?;
            }
        } else {
            for document in &ordered_documents {
                checkpoint(stop)?;
                for system in normalize_document(document).systems.into_iter().flatten() {
                    if system
                        .environment
                        .is_some_and(|e| e != "All" && e != environment)
                    {
                        continue;
                    }
                    preflight_system(
                        document,
                        &system,
                        match system.owner {
                            SystemOwner::ScriptObject(name) => Some(name),
                            _ => None,
                        },
                        OwnerApplicationPreflight {
                            indices: &[],
                            applications: &[],
                            encoded_lengths: &[],
                        },
                        &mut type_count,
                        &mut estimated_bytes,
                        stop,
                    )?;
                }
            }
            targets.reserve(type_count);
            for document in &ordered_documents {
                checkpoint(stop)?;
                for system in normalize_document(document).systems.into_iter().flatten() {
                    if system
                        .environment
                        .is_some_and(|e| e != "All" && e != environment)
                    {
                        continue;
                    }
                    collect_system(
                        document,
                        &system,
                        match system.owner {
                            SystemOwner::ScriptObject(name) => Some(name),
                            _ => None,
                        },
                        &[],
                        &[],
                        &mut targets,
                        stop,
                    )?;
                }
            }
        }
        targets.sort_by(|a, b| {
            (
                &a.effective_name,
                &a.path,
                a.registration,
                a.span.start,
                a.span.end,
                a.kind,
            )
                .cmp(&(
                    &b.effective_name,
                    &b.path,
                    b.registration,
                    b.span.start,
                    b.span.end,
                    b.kind,
                ))
        });
        let mut index_bytes = 0usize;
        let mut index = 0usize;
        while index < targets.len() {
            checkpoint(stop)?;
            let start = index;
            let name = targets[index].effective_name.as_str();
            while index < targets.len() && targets[index].effective_name == name {
                checkpoint(stop)?;
                index += 1;
            }
            let retained_end = (start + MAX_CANDIDATES_PER_NAME).min(index);
            let mut bucket_bytes = 160usize
                .checked_add(name.len())
                .ok_or_else(|| error(NativeErrorCode::Limit))?;
            for target in &targets[start..retained_end] {
                checkpoint(stop)?;
                let strings = target
                    .kind
                    .len()
                    .checked_add(target.path.len())
                    .and_then(|n| n.checked_add(target.sha256.len()))
                    .and_then(|n| n.checked_add(target.original_name.len()))
                    .and_then(|n| n.checked_add(target.effective_name.len()))
                    .ok_or_else(|| error(NativeErrorCode::Limit))?;
                let mut target_bytes = strings
                    .checked_mul(6)
                    .and_then(|n| n.checked_add(512))
                    .and_then(|n| n.checked_add(target.span.start.to_string().len()))
                    .and_then(|n| n.checked_add(target.span.end.to_string().len()))
                    .ok_or_else(|| error(NativeErrorCode::Limit))?;
                for application in &target.owner_correction_applications {
                    checkpoint(stop)?;
                    let encoded = crate::wire_json::canonical_json_bytes(application)
                        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
                    charge(&mut target_bytes, encoded.len(), MAX_CATALOG_BYTES)?;
                }
                charge(&mut bucket_bytes, target_bytes, MAX_CATALOG_BYTES)?;
            }
            charge(&mut index_bytes, bucket_bytes, MAX_CATALOG_BYTES)?;
        }
        charge(&mut estimated_bytes, index_bytes, MAX_CATALOG_BYTES)?;
        let mut names: BTreeMap<String, NameBucket> = BTreeMap::new();
        let mut index = 0usize;
        while index < targets.len() {
            checkpoint(stop)?;
            let start = index;
            let name = targets[index].effective_name.clone();
            while index < targets.len() && targets[index].effective_name == name {
                checkpoint(stop)?;
                index += 1;
            }
            let end = (start + MAX_CANDIDATES_PER_NAME).min(index);
            names.insert(
                name,
                NameBucket {
                    count: index - start,
                    candidates: targets[start..end].to_vec(),
                },
            );
        }
        let id_bytes = crate::wire_json::canonical_json_bytes(&TypeCatalogIdentity {
            schema: PROFILE,
            correction_set_id: correction_set_id.as_deref(),
            targets: &targets,
        })
        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
        if id_bytes.len() > MAX_CATALOG_BYTES {
            return Err(error(NativeErrorCode::Limit));
        }
        let id = crate::native::source_digest(&id_bytes);
        let mut correction_applications = Vec::with_capacity(report.len());
        for application in report {
            checkpoint(stop)?;
            correction_applications.push(CorrectionOutcome::from(application));
        }
        let catalog = Self {
            id,
            correction_set_id,
            correction_applications,
            names,
        };
        Ok((catalog, corrected))
    }

    pub(super) fn id(&self) -> &str {
        &self.id
    }

    pub(super) fn profile(&self) -> &'static str {
        PROFILE
    }

    pub(super) fn resolve_callable(
        &self,
        document: &DocumentationDocument,
        registration: usize,
        function: &str,
        original: &crate::native_model::CallableFact<'_>,
        corrected: Option<&SystemFacts<'_>>,
        stop: &AtomicBool,
    ) -> Result<NativeCallableTypeResolution, NativeError> {
        checkpoint(stop)?;
        let mut application_count = 0usize;
        let mut application_bytes = 0usize;
        for application in &self.correction_applications {
            checkpoint(stop)?;
            if application.target.path == document.path()
                && application.target.registration == registration
                && matches!(
                    &application.target.projection,
                    Projection::CallableField { function: target, .. } if target == function
                )
            {
                application_count = application_count
                    .checked_add(1)
                    .ok_or_else(|| error(NativeErrorCode::Limit))?;
                charge(
                    &mut application_bytes,
                    crate::wire_json::canonical_json_bytes(application)
                        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?
                        .len(),
                    MAX_OVERLAY_BYTES,
                )?;
            }
        }
        if application_count > MAX_CORRECTIONS_PER_CALLABLE {
            return Err(error(NativeErrorCode::Limit));
        }
        let base_strings = self
            .id
            .len()
            .checked_add(self.correction_set_id.as_ref().map_or(0, String::len))
            .ok_or_else(|| error(NativeErrorCode::Limit))?;
        let mut estimated = base_strings
            .checked_mul(6)
            .and_then(|bytes| bytes.checked_add(512))
            .ok_or_else(|| error(NativeErrorCode::Limit))?;
        charge(&mut estimated, application_bytes, MAX_OVERLAY_BYTES)?;
        let mut applications = Vec::with_capacity(application_count);
        for application in &self.correction_applications {
            checkpoint(stop)?;
            if application.target.path == document.path()
                && application.target.registration == registration
                && matches!(
                    &application.target.projection,
                    Projection::CallableField { function: target, .. } if target == function
                )
            {
                applications.push(application.clone());
            }
        }
        let mut fields = Vec::new();
        let mut count = 0usize;
        for lane in [Lane::Arguments, Lane::Returns] {
            let original_fields = match lane {
                Lane::Arguments => &original.arguments,
                Lane::Returns => &original.returns,
            };
            let corrected_fields = corrected.and_then(|system| {
                system
                    .functions
                    .iter()
                    .find(|candidate| candidate.name == function)
                    .map(|candidate| match lane {
                        Lane::Arguments => candidate.arguments.as_slice(),
                        Lane::Returns => candidate.returns.as_slice(),
                    })
            });
            if corrected.is_some() && corrected_fields.is_none() {
                return Err(error(NativeErrorCode::InvalidIdentity));
            }
            for (ordinal, original_field) in original_fields.iter().enumerate() {
                checkpoint(stop)?;
                count = count
                    .checked_add(1)
                    .ok_or_else(|| error(NativeErrorCode::Limit))?;
                if count > MAX_FIELDS {
                    return Err(error(NativeErrorCode::Limit));
                }
                let corrected_field = corrected_fields.and_then(|items| items.get(ordinal));
                if corrected.is_some()
                    && corrected_field.is_none_or(|field| field.name != original_field.name)
                {
                    return Err(error(NativeErrorCode::InvalidIdentity));
                }
                let mut correction_application_ids = Vec::new();
                let mut blocked = false;
                let mut applied_effective = None;
                for application in &applications {
                    checkpoint(stop)?;
                    if !is_type_application(application, lane, original_field.name) {
                        continue;
                    }
                    let id_cost = application
                        .correction_id
                        .len()
                        .checked_mul(6)
                        .and_then(|bytes| bytes.checked_add(128))
                        .ok_or_else(|| error(NativeErrorCode::Limit))?;
                    charge(&mut estimated, id_cost, MAX_OVERLAY_BYTES)?;
                    correction_application_ids.push(application.correction_id.clone());
                    match application.status {
                        Status::Applied => {
                            if let Some(Value::Text(value)) = application.after.as_ref() {
                                if corrected_field.is_none_or(|field| field.type_name != value) {
                                    return Err(error(NativeErrorCode::InvalidIdentity));
                                }
                                charge(
                                    &mut estimated,
                                    value
                                        .len()
                                        .checked_mul(6)
                                        .and_then(|bytes| bytes.checked_add(128))
                                        .ok_or_else(|| error(NativeErrorCode::Limit))?,
                                    MAX_OVERLAY_BYTES,
                                )?;
                                applied_effective = Some(value.clone());
                            } else {
                                return Err(error(NativeErrorCode::InvalidIdentity));
                            }
                        }
                        _ => blocked = true,
                    }
                }
                correction_application_ids.sort();
                correction_application_ids.dedup();
                let effective = (!blocked).then_some(applied_effective).flatten();
                let selected_token = if blocked || original_field.inner_type.is_some() {
                    None
                } else {
                    effective.as_deref().or(Some(original_field.type_name))
                };
                let (status, candidates, candidate_count, candidates_truncated) = if blocked {
                    (ResolutionStatus::CorrectionBlocked, Vec::new(), 0, false)
                } else if original_field.inner_type.is_some() {
                    (ResolutionStatus::Unsupported, Vec::new(), 0, false)
                } else if let Some(token) = selected_token {
                    self.resolve_token(token, stop, &mut estimated)?
                } else {
                    (ResolutionStatus::CorrectionBlocked, Vec::new(), 0, false)
                };
                let field_strings = original_field
                    .name
                    .len()
                    .checked_add(original_field.type_name.len())
                    .and_then(|n| n.checked_add(original_field.inner_type.map_or(0, str::len)))
                    .ok_or_else(|| error(NativeErrorCode::Limit))?;
                let field_cost = field_strings
                    .checked_mul(6)
                    .and_then(|n| n.checked_add(512))
                    .ok_or_else(|| error(NativeErrorCode::Limit))?;
                charge(&mut estimated, field_cost, MAX_OVERLAY_BYTES)?;
                fields.push(FieldResolution {
                    lane,
                    ordinal,
                    name: original_field.name.to_owned(),
                    original_type: original_field.type_name.to_owned(),
                    inner_type: original_field.inner_type.map(str::to_owned),
                    effective_type: effective,
                    status,
                    candidate_count,
                    candidates_truncated,
                    candidates,
                    correction_application_ids,
                });
            }
        }
        let mut overlay = NativeCallableTypeResolution {
            schema: PROFILE,
            catalog_id: self.id.clone(),
            correction_set_id: self.correction_set_id.clone(),
            correction_applications: applications,
            fields,
            encoded_len: 0,
        };
        let encoded = crate::wire_json::canonical_json_bytes(&overlay)
            .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
        if encoded.len() > MAX_OVERLAY_BYTES {
            return Err(error(NativeErrorCode::Limit));
        }
        overlay.encoded_len = encoded.len();
        Ok(overlay)
    }

    fn resolve_token(
        &self,
        token: &str,
        stop: &AtomicBool,
        estimated: &mut usize,
    ) -> Result<(ResolutionStatus, Vec<TypeTarget>, usize, bool), NativeError> {
        checkpoint(stop)?;
        if !atomic_identifier(token) || token == "any" {
            return Ok((ResolutionStatus::Unsupported, Vec::new(), 0, false));
        }
        let Some(bucket) = self.names.get(token) else {
            return Ok((ResolutionStatus::Unresolved, Vec::new(), 0, false));
        };
        let truncated = bucket.count > MAX_CANDIDATES_PER_FIELD;
        let candidate_count = bucket.candidates.len().min(MAX_CANDIDATES_PER_FIELD);
        let mut candidate_bytes = 0usize;
        for target in bucket.candidates.iter().take(candidate_count) {
            checkpoint(stop)?;
            let strings = target
                .path
                .len()
                .checked_add(target.sha256.len())
                .and_then(|n| n.checked_add(target.original_name.len()))
                .and_then(|n| n.checked_add(target.effective_name.len()))
                .and_then(|n| n.checked_add(target.kind.len()))
                .ok_or_else(|| error(NativeErrorCode::Limit))?;
            charge(
                &mut candidate_bytes,
                strings
                    .checked_mul(6)
                    .and_then(|n| n.checked_add(512))
                    .ok_or_else(|| error(NativeErrorCode::Limit))?,
                MAX_OVERLAY_BYTES,
            )?;
            for application in &target.owner_correction_applications {
                checkpoint(stop)?;
                let encoded = crate::wire_json::canonical_json_bytes(application)
                    .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
                charge(&mut candidate_bytes, encoded.len(), MAX_OVERLAY_BYTES)?;
            }
        }
        charge(estimated, candidate_bytes, MAX_OVERLAY_BYTES)?;
        let candidates = bucket
            .candidates
            .iter()
            .take(MAX_CANDIDATES_PER_FIELD)
            .cloned()
            .collect::<Vec<_>>();
        let blocked_owner_application = bucket.count == 1
            && bucket.candidates.first().is_some_and(|target| {
                target
                    .owner_correction_applications
                    .iter()
                    .any(|application| application.status != Status::Applied)
                    || target.owner_correction_applications.len() > 1
            });
        let status = if blocked_owner_application {
            ResolutionStatus::CorrectionBlocked
        } else if bucket.count == 1 {
            ResolutionStatus::Resolved
        } else {
            ResolutionStatus::Ambiguous
        };
        Ok((status, candidates, bucket.count, truncated))
    }
}

fn is_type_application(application: &CorrectionOutcome, lane: Lane, member: &str) -> bool {
    matches!(
        &application.target.projection,
        Projection::CallableField {
            lane: target_lane,
            member: target_member,
            property: Property::Type,
            ..
        } if *target_lane == lane && target_member == member
    )
}

struct OwnerApplicationPreflight<'a> {
    indices: &'a [usize],
    applications: &'a [Application],
    encoded_lengths: &'a [usize],
}

fn preflight_system(
    document: &DocumentationDocument,
    system: &SystemFacts<'_>,
    original_owner: Option<&str>,
    owner_applications: OwnerApplicationPreflight<'_>,
    count: &mut usize,
    bytes: &mut usize,
    stop: &AtomicBool,
) -> Result<(), NativeError> {
    let OwnerApplicationPreflight {
        indices: owner_application_indices,
        applications,
        encoded_lengths: application_bytes,
    } = owner_applications;
    let add = |kind: &str,
               original: &str,
               effective: &str,
               span: Span,
               count: &mut usize,
               bytes: &mut usize|
     -> Result<(), NativeError> {
        *count = count
            .checked_add(1)
            .ok_or_else(|| error(NativeErrorCode::Limit))?;
        if *count > MAX_TYPES {
            return Err(error(NativeErrorCode::Limit));
        }
        let string_bytes = kind
            .len()
            .checked_add(original.len())
            .and_then(|n| n.checked_add(effective.len()))
            .and_then(|n| n.checked_add(document.path().len()))
            .and_then(|n| n.checked_add(document.sha256().len()))
            .ok_or_else(|| error(NativeErrorCode::Limit))?;
        let estimate = string_bytes
            .checked_mul(6)
            .and_then(|n| n.checked_add(512))
            .and_then(|n| n.checked_add(span.start.to_string().len()))
            .and_then(|n| n.checked_add(span.end.to_string().len()))
            .ok_or_else(|| error(NativeErrorCode::Limit))?;
        charge(bytes, estimate, MAX_CATALOG_BYTES)
    };
    checkpoint(stop)?;
    if let SystemOwner::ScriptObject(effective_owner) = system.owner {
        let original = original_owner.unwrap_or(effective_owner);
        let mut sole_owner_application = None;
        for index in owner_application_indices {
            checkpoint(stop)?;
            let application = applications
                .get(*index)
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
            charge(
                bytes,
                *application_bytes
                    .get(*index)
                    .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?,
                MAX_CATALOG_BYTES,
            )?;
            sole_owner_application = Some(application);
        }
        let effective = match (owner_application_indices.len(), sole_owner_application) {
            (0, _) if effective_owner == original => effective_owner,
            (1, Some(application)) if application.status == Status::Applied => {
                let Some(Value::Text(name)) = application.after.as_ref() else {
                    return Err(error(NativeErrorCode::InvalidIdentity));
                };
                if name != effective_owner {
                    return Err(error(NativeErrorCode::InvalidIdentity));
                }
                name.as_str()
            }
            (_, _) if effective_owner == original => original,
            _ => return Err(error(NativeErrorCode::InvalidIdentity)),
        };
        add(
            "script_object",
            original,
            effective,
            system.raw.span,
            count,
            bytes,
        )?;
    }
    for table in &system.tables {
        checkpoint(stop)?;
        match table {
            TableFact::Structure { name, raw, .. } => {
                add("structure", name, name, raw.span, count, bytes)?;
            }
            TableFact::Enumeration { name, raw, .. } => {
                add("enum", name, name, raw.span, count, bytes)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn collect_system(
    document: &DocumentationDocument,
    system: &SystemFacts<'_>,
    original_owner: Option<&str>,
    owner_application_indices: &[usize],
    applications: &[Application],
    targets: &mut Vec<TypeTarget>,
    stop: &AtomicBool,
) -> Result<(), NativeError> {
    checkpoint(stop)?;
    let owner_corrections = owner_application_indices
        .iter()
        .map(|index| {
            checkpoint(stop)?;
            applications
                .get(*index)
                .map(CorrectionOutcome::from)
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let owner_blocked = owner_corrections
        .iter()
        .any(|application| application.status != Status::Applied)
        || owner_corrections.len() > 1;
    let add = |kind: &'static str,
               original: &str,
               effective: &str,
               span: Span,
               owner_evidence: &[CorrectionOutcome],
               targets: &mut Vec<TypeTarget>| {
        targets.push(TypeTarget {
            kind,
            original_name: original.to_owned(),
            effective_name: effective.to_owned(),
            path: document.path().to_owned(),
            sha256: document.sha256().to_owned(),
            span,
            registration: system.registration_ordinal,
            owner_correction_applications: owner_evidence.to_vec(),
        });
    };
    if let SystemOwner::ScriptObject(effective_owner) = system.owner {
        let original = original_owner.unwrap_or(effective_owner);
        let effective = if owner_corrections.is_empty() {
            if effective_owner != original {
                return Err(error(NativeErrorCode::InvalidIdentity));
            }
            effective_owner
        } else if !owner_blocked && owner_corrections.len() == 1 {
            let Some(Value::Text(name)) = owner_corrections[0].after.as_ref() else {
                return Err(error(NativeErrorCode::InvalidIdentity));
            };
            if name != effective_owner {
                return Err(error(NativeErrorCode::InvalidIdentity));
            }
            name.as_str()
        } else {
            if effective_owner != original {
                return Err(error(NativeErrorCode::InvalidIdentity));
            }
            original
        };
        add(
            "script_object",
            original,
            effective,
            system.raw.span,
            &owner_corrections,
            targets,
        );
    }
    for table in &system.tables {
        checkpoint(stop)?;
        match table {
            TableFact::Structure { name, raw, .. } => {
                add("structure", name, name, raw.span, &[], targets);
            }
            TableFact::Enumeration { name, raw, .. } => {
                add("enum", name, name, raw.span, &[], targets);
            }
            _ => {}
        }
    }
    Ok(())
}

fn atomic_identifier(token: &str) -> bool {
    let mut bytes = token.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn selected_environment(system: &SystemFacts<'_>, environment: &str) -> bool {
    system
        .environment
        .is_none_or(|selected| selected == "All" || selected == environment)
}

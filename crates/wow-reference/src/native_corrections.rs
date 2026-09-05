//! Ketho's name-addressed patch and widget-name lanes, with the E1 correction
//! contract's source guards. Changes affect borrowed normalized facts only.
//! No donor/source code is executed and no source digest is refreshed implicitly.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::native::{DocumentationDocument, RawValue, Span, source_digest};
use crate::native_model::{FieldFact, SystemFacts, SystemOwner};
use crate::wire_json::canonical_json_bytes;

pub const NORMALIZER: &str = "native-model/1";
pub const SCHEMA: &str = "wow-native-corrections/1";
const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_RECORDS: usize = 4096;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorrectionSet {
    pub schema: String,
    pub version: u32,
    pub revision: String,
    pub environment: String,
    pub normalizer: String,
    pub records: Vec<Correction>,
}

/// Each independent correction has its own review and exact target guards.
/// Dependent transforms and wildcard/basename selection are not supported.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Correction {
    pub id: String,
    pub target: Target,
    pub expected_source_sha256: String,
    pub expected_raw_sha256: String,
    pub before: Value,
    pub after: Value,
    pub reviewer: String,
    pub rationale: String,
    pub evidence: Vec<Evidence>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub path: String,
    pub registration: usize,
    pub projection: Projection,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Projection {
    WidgetOwner,
    CallableField {
        function: String,
        lane: Lane,
        member: String,
        property: Property,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Lane {
    Arguments,
    Returns,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Property {
    Type,
    Nilable,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Value {
    Absent,
    Text(String),
    Boolean(bool),
}

/// Public evidence identity only. The caller authorizes/reviews this data; these
/// strings are an audit record, not proof that a human or external service ran.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub revision: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ValidatedCorrections {
    id: String,
    set: CorrectionSet,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrectionError {
    InvalidSet,
    Limit,
    InvalidSource,
    Cancelled,
}
impl fmt::Display for CorrectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native correction rejected: {self:?}")
    }
}
impl std::error::Error for CorrectionError {}
type Result<T> = std::result::Result<T, CorrectionError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Applied,
    Expired,
    Rejected,
    Conflict,
    NotApplicable,
}
#[derive(Clone, Debug, Serialize)]
pub struct Application {
    pub correction_id: String,
    pub target: Target,
    pub status: Status,
    pub reason: &'static str,
    pub observed_source_sha256: Option<String>,
    pub observed_raw_sha256: Option<String>,
    pub span: Option<Span>,
    pub before: Option<Value>,
    pub after: Option<Value>,
}
#[derive(Clone, Debug, Serialize)]
pub struct CorrectionReport {
    pub schema: &'static str,
    pub corrections: ValidatedCorrections,
    pub applications: Vec<Application>,
}
impl CorrectionReport {
    pub fn has_blockers(&self) -> bool {
        self.applications.iter().any(|a| {
            matches!(
                a.status,
                Status::Expired | Status::Rejected | Status::Conflict
            )
        })
    }
}

fn oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn digest(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|v| v.len() == 64 && oid(v))
}
fn path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && !value.contains(['\\', ':'])
        && !value.chars().any(char::is_control)
        && value.split('/').all(|p| !matches!(p, "" | "." | ".."))
}
fn label(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}
fn identifier(value: &str) -> bool {
    let mut chars = value.bytes();
    value.len() <= 1024
        && chars
            .next()
            .is_some_and(|b| b == b'_' || b.is_ascii_alphabetic())
        && chars.all(|b| b == b'_' || b.is_ascii_alphanumeric())
        && !matches!(
            value,
            "and"
                | "break"
                | "do"
                | "else"
                | "elseif"
                | "end"
                | "false"
                | "for"
                | "function"
                | "goto"
                | "if"
                | "in"
                | "local"
                | "nil"
                | "not"
                | "or"
                | "repeat"
                | "return"
                | "then"
                | "true"
                | "until"
                | "while"
        )
}
fn type_name(value: &str) -> bool {
    let parts = value.split('|').collect::<Vec<_>>();
    value.len() <= 1024
        && parts.len() <= 16
        && !parts.is_empty()
        && parts
            .iter()
            .all(|p| matches!(*p, "nil" | "function") || p.split('.').all(identifier))
        && parts.iter().collect::<BTreeSet<_>>().len() == parts.len()
}

impl ValidatedCorrections {
    /// Strict bounded data input. No default resource or network discovery.
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(CorrectionError::Limit);
        }
        Self::new(serde_json::from_slice(bytes).map_err(|_| CorrectionError::InvalidSet)?)
    }
    pub fn new(mut set: CorrectionSet) -> Result<Self> {
        if set.records.len() > MAX_RECORDS {
            return Err(CorrectionError::Limit);
        }
        if set.schema != SCHEMA
            || set.version == 0
            || !oid(&set.revision)
            || !label(&set.environment, 128)
            || !label(&set.normalizer, 128)
        {
            return Err(CorrectionError::InvalidSet);
        }
        let mut ids = BTreeSet::new();
        for record in &set.records {
            if !label(&record.id, 128)
                || !ids.insert(&record.id)
                || !path(&record.target.path)
                || record.target.registration >= 128
                || !digest(&record.expected_source_sha256)
                || !digest(&record.expected_raw_sha256)
                || !label(&record.reviewer, 256)
                || !label(&record.rationale, 4096)
                || record.evidence.is_empty()
                || record.evidence.len() > 16
                || record
                    .evidence
                    .iter()
                    .any(|e| !oid(&e.revision) || !path(&e.path) || !digest(&e.sha256))
            {
                return Err(CorrectionError::InvalidSet);
            }
            let valid = match (&record.target.projection, &record.before, &record.after) {
                (Projection::WidgetOwner, Value::Text(before), Value::Text(after)) => {
                    identifier(before) && identifier(after)
                }
                (
                    Projection::CallableField {
                        function,
                        member,
                        property: Property::Type,
                        ..
                    },
                    Value::Text(before),
                    Value::Text(after),
                ) => {
                    identifier(function)
                        && label(member, 1024)
                        && type_name(before)
                        && type_name(after)
                }
                (
                    Projection::CallableField {
                        function,
                        member,
                        property: Property::Nilable,
                        ..
                    },
                    Value::Absent | Value::Boolean(_),
                    Value::Boolean(_),
                ) => identifier(function) && label(member, 1024),
                _ => false,
            };
            if !valid {
                return Err(CorrectionError::InvalidSet);
            }
        }
        set.records
            .sort_by(|a, b| (&a.target, &a.id).cmp(&(&b.target, &b.id)));
        let bytes = canonical_json_bytes(&set).map_err(|_| CorrectionError::InvalidSet)?;
        if bytes.len() > MAX_BYTES {
            return Err(CorrectionError::Limit);
        }
        Ok(Self {
            id: source_digest(&bytes),
            set,
        })
    }
    pub fn set(&self) -> &CorrectionSet {
        &self.set
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Raw observation digest includes original field order, values and source spans.
/// Computing it is inspection, not authority to update a reviewed expectation.
pub fn raw_digest(value: &RawValue) -> Result<String> {
    Ok(source_digest(
        &canonical_json_bytes(value).map_err(|_| CorrectionError::InvalidSource)?,
    ))
}

#[derive(Clone, Copy)]
struct Location {
    system: usize,
    field: Option<(usize, Lane, usize, Property)>,
}
fn locate(
    systems: &[(&DocumentationDocument, SystemFacts<'_>)],
    target: &Target,
) -> std::result::Result<Location, Status> {
    let matches = systems
        .iter()
        .enumerate()
        .filter(|(_, (d, s))| {
            d.path() == target.path && s.registration_ordinal == target.registration
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(if matches.is_empty() {
            Status::Expired
        } else {
            Status::Conflict
        });
    }
    let (system, (_, facts)) = matches[0];
    let field = match &target.projection {
        Projection::WidgetOwner => {
            if !matches!(facts.owner, SystemOwner::ScriptObject(_)) {
                return Err(Status::Rejected);
            }
            None
        }
        Projection::CallableField {
            function,
            lane,
            member,
            property,
        } => {
            let functions = facts
                .functions
                .iter()
                .enumerate()
                .filter(|(_, f)| f.name == function)
                .collect::<Vec<_>>();
            if functions.len() != 1 {
                return Err(if functions.is_empty() {
                    Status::Expired
                } else {
                    Status::Conflict
                });
            }
            let (fi, callable) = functions[0];
            let fields = match lane {
                Lane::Arguments => &callable.arguments,
                Lane::Returns => &callable.returns,
            };
            let fields = fields
                .iter()
                .enumerate()
                .filter(|(_, f)| f.name == member)
                .collect::<Vec<_>>();
            if fields.len() != 1 {
                return Err(if fields.is_empty() {
                    Status::Expired
                } else {
                    Status::Conflict
                });
            }
            Some((fi, *lane, fields[0].0, *property))
        }
    };
    Ok(Location { system, field })
}
fn field_at<'a, 's>(system: &'s SystemFacts<'a>, location: Location) -> Option<&'s FieldFact<'a>> {
    location.field.map(|(f, l, m, _)| match l {
        Lane::Arguments => &system.functions[f].arguments[m],
        Lane::Returns => &system.functions[f].returns[m],
    })
}
fn observed<'a>(system: &SystemFacts<'a>, location: Location) -> (&'a RawValue, Value) {
    if let Some(field) = field_at(system, location) {
        let value = match location.field.map(|v| v.3) {
            Some(Property::Type) => Value::Text(field.type_name.into()),
            _ => field.nilable.map_or(Value::Absent, Value::Boolean),
        };
        (field.raw, value)
    } else {
        let name = match system.owner {
            SystemOwner::ScriptObject(n) => n,
            _ => "",
        };
        (system.raw, Value::Text(name.into()))
    }
}

/// Plan and validate independent edits before mutating a normalized copy. Blocked records preserve their original projections. Every
/// record has an outcome; no mutation is performed before final cancellation check.
fn apply<'a>(
    corrections: &'a ValidatedCorrections,
    revision: &str,
    environment: &str,
    systems: &mut [(&'a DocumentationDocument, SystemFacts<'a>)],
    cancelled: &AtomicBool,
) -> Result<CorrectionReport> {
    if systems.len() > MAX_RECORDS * 32 || systems.iter().any(|(d, _)| d.revision() != revision) {
        return Err(CorrectionError::InvalidSource);
    }
    let set = &corrections.set;
    let mut seen = BTreeMap::new();
    for record in &set.records {
        *seen.entry(&record.target).or_insert(0usize) += 1;
    }
    let mut source_owners = BTreeMap::new();
    for (_, system) in systems.iter().filter(|(_, s)| selected(s, environment)) {
        if let SystemOwner::ScriptObject(name) = system.owner {
            *source_owners.entry(name).or_insert(0usize) += 1;
        }
    }
    let mut applications = Vec::new();
    let mut locations = Vec::new();
    for record in &set.records {
        if cancelled.load(Ordering::Relaxed) {
            return Err(CorrectionError::Cancelled);
        }
        let mut application = Application {
            correction_id: record.id.clone(),
            target: record.target.clone(),
            status: Status::Applied,
            reason: "exact_match",
            observed_source_sha256: None,
            observed_raw_sha256: None,
            span: None,
            before: None,
            after: None,
        };
        let mut location = None;
        let failure = if set.environment != environment {
            Some((Status::NotApplicable, "environment_not_selected"))
        } else if set.revision != revision || set.normalizer != NORMALIZER {
            Some((Status::Expired, "source_or_normalizer_changed"))
        } else if seen[&record.target] != 1 {
            Some((Status::Conflict, "multiple_corrections_for_target"))
        } else {
            match locate(systems, &record.target) {
                Err(status) => Some((status, "target_not_unique_or_supported")),
                Ok(found) => {
                    let (document, system) = &systems[found.system];
                    let (raw, before) = observed(system, found);
                    let hash = raw_digest(raw)?;
                    application.observed_source_sha256 = Some(document.sha256().into());
                    application.observed_raw_sha256 = Some(hash.clone());
                    application.span = Some(raw.span);
                    application.before = Some(before.clone());
                    if document.sha256() != record.expected_source_sha256
                        || hash != record.expected_raw_sha256
                        || before != record.before
                    {
                        Some((Status::Expired, "expected_source_or_value_changed"))
                    } else if system
                        .environment
                        .is_some_and(|e| e != "All" && e != environment)
                    {
                        Some((Status::NotApplicable, "source_environment_not_selected"))
                    } else if found.field.is_none()
                        && matches!(system.owner, SystemOwner::ScriptObject(name) if source_owners.get(name).is_some_and(|n| *n > 1))
                    {
                        Some((Status::Conflict, "source_widget_owner_conflict"))
                    } else if found.field.is_some_and(|(_, _, _, p)| p == Property::Type)
                        && field_at(system, found).is_some_and(|f| f.inner_type.is_some())
                    {
                        Some((Status::Rejected, "array_shape_requires_separate_correction"))
                    } else {
                        location = Some(found);
                        None
                    }
                }
            }
        };
        if let Some((status, reason)) = failure {
            application.status = status;
            application.reason = reason;
        }
        applications.push(application);
        locations.push(location);
    }
    // Validate simultaneous widget names before mutation. Include all unmodified
    // source objects and named types; aliases cannot silently merge receivers.
    // Rejected aliases revert, so repeat to catch a collision exposed by a revert.
    let mut collision_budget = 32 * 1024 * 1024usize;
    for _ in 0..=set.records.len() {
        collision_budget = collision_budget
            .checked_sub(systems.len().saturating_add(set.records.len()))
            .ok_or(CorrectionError::Limit)?;
        let replacements = locations
            .iter()
            .enumerate()
            .filter_map(|(i, l)| {
                l.filter(|l| l.field.is_none() && applications[i].status == Status::Applied)
                    .and_then(|l| match &set.records[i].after {
                        Value::Text(s) => Some((l.system, s.as_str())),
                        _ => None,
                    })
            })
            .collect::<BTreeMap<_, _>>();
        let mut owners = BTreeMap::<&str, Vec<usize>>::new();
        for (index, (_, system)) in systems
            .iter()
            .enumerate()
            .filter(|(_, (_, s))| selected(s, environment))
        {
            if let SystemOwner::ScriptObject(original) = system.owner {
                let replacement = replacements.get(&index).copied();
                owners
                    .entry(replacement.unwrap_or(original))
                    .or_default()
                    .push(index);
            }
        }
        let types = systems
            .iter()
            .filter(|(_, s)| selected(s, environment))
            .flat_map(|(_, s)| s.tables.iter())
            .filter_map(|t| match t {
                crate::native_model::TableFact::Structure { name, .. }
                | crate::native_model::TableFact::Callback { name, .. }
                | crate::native_model::TableFact::Unsupported { name, .. } => Some(*name),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let globals = systems
            .iter()
            .filter(|(_, s)| selected(s, environment))
            .flat_map(|(_, s)| match s.owner {
                SystemOwner::Global => s.functions.iter().map(|f| f.name).collect::<Vec<_>>(),
                SystemOwner::Namespace(n) => vec![n],
                _ => vec![],
            })
            .collect::<BTreeSet<_>>();
        let mut changed = false;
        for (i, location) in locations.iter().enumerate() {
            if location.is_some_and(|l| l.field.is_none())
                && applications[i].status == Status::Applied
            {
                let Value::Text(name) = &set.records[i].after else {
                    continue;
                };
                if owners
                    .get(name.as_str())
                    .is_some_and(|entries| entries.len() > 1)
                    || types.contains(name.as_str())
                    || globals.contains(name.as_str())
                {
                    applications[i].status = Status::Conflict;
                    applications[i].reason = "widget_name_collision";
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
        if cancelled.load(Ordering::Relaxed) {
            return Err(CorrectionError::Cancelled);
        }
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err(CorrectionError::Cancelled);
    }
    for (index, location) in locations.into_iter().enumerate() {
        if applications[index].status != Status::Applied {
            continue;
        }
        let Some(location) = location else {
            return Err(CorrectionError::InvalidSource);
        };
        let system = &mut systems[location.system].1;
        let after = &set.records[index].after;
        if let Some((function, lane, member, property)) = location.field {
            let field = match lane {
                Lane::Arguments => &mut system.functions[function].arguments[member],
                Lane::Returns => &mut system.functions[function].returns[member],
            };
            match (property, after) {
                (Property::Type, Value::Text(s)) => field.type_name = s,
                (Property::Nilable, Value::Boolean(b)) => field.nilable = Some(*b),
                _ => return Err(CorrectionError::InvalidSource),
            }
        } else if let Value::Text(name) = after {
            system.owner = SystemOwner::ScriptObject(name);
        }
        applications[index].after = Some(after.clone());
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err(CorrectionError::Cancelled);
    }
    Ok(CorrectionReport {
        schema: "wow-native-correction-applications/1",
        corrections: corrections.clone(),
        applications,
    })
}

/// Normalization failures are retained separately; applying a correction cannot
/// repair an unrecognized source schema or remove the corresponding failure.
pub struct CorrectedCorpus<'a> {
    systems: Vec<(&'a DocumentationDocument, SystemFacts<'a>)>,
    pub normalization_errors: Vec<(&'a DocumentationDocument, crate::native_model::ModelError)>,
    pub report: CorrectionReport,
}
impl<'a> CorrectedCorpus<'a> {
    pub fn systems(&self) -> &[(&'a DocumentationDocument, SystemFacts<'a>)] {
        &self.systems
    }
}

/// Normalize original validated documents here, not caller-supplied field copies.
/// This ensures the observed target is really part of the cited raw source.
pub fn apply_to_documents<'a>(
    documents: &'a [DocumentationDocument],
    environment: &str,
    corrections: &'a ValidatedCorrections,
    cancelled: &AtomicBool,
) -> Result<CorrectedCorpus<'a>> {
    if documents.is_empty() || documents.len() > 4096 {
        return Err(CorrectionError::InvalidSource);
    }
    let revision = documents[0].revision();
    let mut paths = BTreeSet::new();
    let mut bytes = 0usize;
    for document in documents {
        bytes = bytes.saturating_add(document.source_bytes());
        if document.revision() != revision || !paths.insert(document.path()) {
            return Err(CorrectionError::InvalidSource);
        }
        if bytes > 64 * 1024 * 1024 {
            return Err(CorrectionError::Limit);
        }
    }
    let mut sorted = documents.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|document| document.path());
    let mut systems = Vec::new();
    let mut normalization_errors = Vec::new();
    let mut units = 0usize;
    for document in sorted {
        if cancelled.load(Ordering::Relaxed) {
            return Err(CorrectionError::Cancelled);
        }
        for normalized in crate::native_model::normalize_document(document).systems {
            match normalized {
                Ok(system) => {
                    units = units.saturating_add(
                        1 + system
                            .functions
                            .iter()
                            .map(|f| 1 + f.arguments.len() + f.returns.len())
                            .sum::<usize>()
                            + system.tables.len(),
                    );
                    if units.saturating_mul(corrections.set.records.len().max(1)) > 32 * 1024 * 1024
                    {
                        return Err(CorrectionError::Limit);
                    }
                    systems.push((document, system));
                }
                Err(error) => normalization_errors.push((document, error)),
            }
        }
    }
    let report = apply(corrections, revision, environment, &mut systems, cancelled)?;
    Ok(CorrectedCorpus {
        systems,
        normalization_errors,
        report,
    })
}

fn selected(system: &SystemFacts<'_>, environment: &str) -> bool {
    system
        .environment
        .is_none_or(|e| e == "All" || e == environment)
}

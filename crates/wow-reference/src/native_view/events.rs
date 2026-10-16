//! Sealed event literals and payloads from the existing normalized system pass.
use super::*;
use crate::native_model::{EventFact, SystemFacts};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct EventData {
    definition_name: String,
    literal_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    payload: Vec<NativeCallableField>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct EventRow {
    digest: String,
    source_id: String,
    data: EventData,
    documentation: CapturedDocumentation,
}

/// Original event record and literal identity. No public constructor/deserializer.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct NativeEventSignature<'a> {
    profile: &'static str,
    reference_generation: &'a str,
    reference_view_digest: &'a str,
    record_key: &'a str,
    record_digest: &'a str,
    source_id: &'a str,
    data: &'a EventData,
    #[serde(skip)]
    documentation: &'a CapturedDocumentation,
}
impl NativeEventSignature<'_> {
    pub fn definition_name(&self) -> &str {
        &self.data.definition_name
    }
    pub fn literal_name(&self) -> &str {
        &self.data.literal_name
    }
    pub fn namespace(&self) -> Option<&str> {
        self.data.namespace.as_deref()
    }
    pub fn payload(&self) -> &[NativeCallableField] {
        &self.data.payload
    }
    pub fn documentation(&self) -> NativeDocumentation<'_> {
        self.documentation.view()
    }
    pub const fn reference_generation(&self) -> &str {
        self.reference_generation
    }
    pub const fn reference_view_digest(&self) -> &str {
        self.reference_view_digest
    }
    pub const fn record_key(&self) -> &str {
        self.record_key
    }
    pub const fn record_digest(&self) -> &str {
        self.record_digest
    }
    pub const fn source_id(&self) -> &str {
        self.source_id
    }
}

pub(super) struct EventBuild {
    enabled: bool,
    pub grouped: BTreeMap<String, Vec<ReferenceRecord>>,
    pending: BTreeMap<String, Vec<EventRow>>,
    pub sources: Vec<NativeViewSource>,
    pub issues: Vec<NativeViewIssue>,
    examined: usize,
}
impl EventBuild {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            grouped: BTreeMap::new(),
            pending: BTreeMap::new(),
            sources: Vec::new(),
            issues: Vec::new(),
            examined: 0,
        }
    }
    pub fn capture(
        &mut self,
        doc: &DocumentationDocument,
        system: &SystemFacts<'_>,
        profile: &str,
        candidate_bytes: &mut usize,
        stop: &AtomicBool,
    ) -> Result<(), NativeError> {
        let namespace = match system.owner {
            SystemOwner::Global => None,
            SystemOwner::Namespace(value) => Some(value),
            SystemOwner::ScriptObject(_) => return Err(error(NativeErrorCode::InvalidIdentity)),
        };
        for event in &system.events {
            checkpoint(stop)?;
            self.examined = self
                .examined
                .checked_add(1)
                .ok_or_else(|| error(NativeErrorCode::Limit))?;
            if self.examined > 65_536 {
                return Err(error(NativeErrorCode::Limit));
            }
            let payload = crate::wire_json::canonical_json_bytes(event.raw)
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
            if payload.len() > 65_536 {
                self.issues
                    .push(issue("event_payload_limit", doc, event.raw.span));
                continue;
            }
            add_candidate_bytes(candidate_bytes, payload.len())?;
            let id = source_id(profile, "event", doc.revision(), doc, event.raw.span)?;
            let key = format!("event:{}", event.literal_name);
            let record = match ReferenceRecord::new(
                &key,
                ReferenceRecordKind::Event,
                String::from_utf8(payload).map_err(|_| error(NativeErrorCode::InvalidEncoding))?,
                vec![id.clone()],
                Vec::new(),
            ) {
                Ok(record) => record,
                Err(_) => {
                    self.issues
                        .push(issue("event_record_rejected", doc, event.raw.span));
                    continue;
                }
            };
            let row = capture_row(&record, event, namespace, stop)?;
            self.sources.push(NativeViewSource {
                id,
                path: doc.path().into(),
                sha256: doc.sha256().into(),
                span: event.raw.span,
            });
            self.pending.entry(key.clone()).or_default().push(row);
            self.grouped.entry(key).or_default().push(record);
        }
        Ok(())
    }
    pub fn finish(
        mut self,
        view: &ReferenceView,
        stop: &AtomicBool,
    ) -> Result<EventCatalog, NativeError> {
        let mut catalog = EventCatalog {
            enabled: self.enabled,
            generation: view.generation_id().into(),
            view_digest: view.self_digest().into(),
            rows: BTreeMap::new(),
        };
        if !self.enabled {
            return Ok(catalog);
        }
        let partition = view
            .partitions()
            .iter()
            .find(|part| part.id() == NATIVE_EVENT_PARTITION)
            .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
        for record in partition.records() {
            checkpoint(stop)?;
            let mut group = self
                .pending
                .remove(record.key())
                .filter(|group| group.len() == 1)
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
            let row = group
                .pop()
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
            if record.source_ids().len() != 1 || record.source_ids()[0].as_ref() != row.source_id {
                return Err(error(NativeErrorCode::InvalidIdentity));
            }
            catalog.rows.insert(record.key().into(), row);
        }
        Ok(catalog)
    }
}
fn capture_row(
    record: &ReferenceRecord,
    event: &EventFact<'_>,
    namespace: Option<&str>,
    stop: &AtomicBool,
) -> Result<EventRow, NativeError> {
    let mut payload = Vec::with_capacity(event.payload.len());
    for field in &event.payload {
        checkpoint(stop)?;
        payload.push(NativeCallableField::capture(field));
    }
    Ok(EventRow {
        documentation: CapturedDocumentation::capture(event.raw, stop)?,
        digest: record
            .digest()
            .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
        source_id: record.source_ids()[0].to_string(),
        data: EventData {
            definition_name: event.name.into(),
            literal_name: event.literal_name.into(),
            namespace: namespace.map(str::to_owned),
            payload,
        },
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EventCatalog {
    enabled: bool,
    generation: String,
    view_digest: String,
    rows: BTreeMap<String, EventRow>,
}
impl EventCatalog {
    pub(super) fn rebind(
        &mut self,
        base: &ReferenceView,
        next: &ReferenceView,
        stop: &AtomicBool,
    ) -> Result<(), NativeError> {
        checkpoint(stop)?;
        validate_rebind(
            &self.generation,
            &self.view_digest,
            base,
            next,
            NATIVE_EVENT_PARTITION,
        )?;
        self.generation = next.generation_id().into();
        self.view_digest = next.self_digest().into();
        Ok(())
    }
    pub fn read<'a>(
        &'a self,
        record: &ReferenceRecord,
        stop: &AtomicBool,
    ) -> Result<NativeEventSignature<'a>, NativeError> {
        checkpoint(stop)?;
        let (key, row) = self
            .rows
            .get_key_value(record.key())
            .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
        if !self.enabled
            || record.kind() != ReferenceRecordKind::Event
            || record.payload().len() > 65_536
            || record.source_ids().len() != 1
            || record.source_ids()[0].as_ref() != row.source_id
            || !record.restrictions().is_empty()
            || record
                .digest()
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?
                != row.digest
        {
            return Err(error(NativeErrorCode::InvalidIdentity));
        }
        Ok(NativeEventSignature {
            profile: "wow-reference/native-source-event-signature/1",
            reference_generation: &self.generation,
            reference_view_digest: &self.view_digest,
            record_key: key,
            record_digest: &row.digest,
            source_id: &row.source_id,
            data: &row.data,
            documentation: &row.documentation,
        })
    }
}

//! Sealed named declaration observations from the existing NativeModel pass.
use super::*;
use crate::native_model::{SystemFacts, TableFact, ValueFact};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeDeclarationKind {
    Structure,
    Callback,
    Enumeration,
    Constants,
    ScriptObject,
}
impl NativeDeclarationKind {
    pub const fn record_kind(self) -> ReferenceRecordKind {
        match self {
            Self::Enumeration | Self::Constants => ReferenceRecordKind::Enumeration,
            Self::Callback => ReferenceRecordKind::Event,
            Self::ScriptObject => ReferenceRecordKind::Widget,
            Self::Structure => ReferenceRecordKind::Other,
        }
    }
    pub const fn key_prefix(self) -> &'static str {
        match self {
            Self::Structure => "structure",
            Self::Callback => "callback",
            Self::Enumeration => "enumeration",
            Self::Constants => "constants",
            Self::ScriptObject => "script_object",
        }
    }
}
/// Source spelling is retained; no scalar evaluation, coercion or type inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "source", rename_all = "snake_case")]
pub enum NativeDeclarationScalar {
    Boolean(bool),
    NumberLexeme(String),
    String(String),
    UnresolvedDigest(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeDeclarationValue {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_type: Option<String>,
    pub value: NativeDeclarationScalar,
    pub source_span: Span,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DeclarationData {
    kind: NativeDeclarationKind,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    definition_namespace: Option<String>,
    fields: Vec<NativeCallableField>,
    arguments: Vec<NativeCallableField>,
    returns: Vec<NativeCallableField>,
    values: Vec<NativeDeclarationValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_base: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct DeclarationRow {
    digest: String,
    source_id: String,
    data: DeclarationData,
    documentation: CapturedDocumentation,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct NativeDeclarationSignature<'a> {
    profile: &'static str,
    reference_generation: &'a str,
    reference_view_digest: &'a str,
    record_key: &'a str,
    record_digest: &'a str,
    source_id: &'a str,
    data: &'a DeclarationData,
    #[serde(skip)]
    documentation: &'a CapturedDocumentation,
}
impl NativeDeclarationSignature<'_> {
    pub const fn kind(&self) -> NativeDeclarationKind {
        self.data.kind
    }
    pub fn name(&self) -> &str {
        &self.data.name
    }
    pub fn definition_namespace(&self) -> Option<&str> {
        self.data.definition_namespace.as_deref()
    }
    pub fn fields(&self) -> &[NativeCallableField] {
        &self.data.fields
    }
    pub fn arguments(&self) -> &[NativeCallableField] {
        &self.data.arguments
    }
    pub fn returns(&self) -> &[NativeCallableField] {
        &self.data.returns
    }
    pub fn values(&self) -> &[NativeDeclarationValue] {
        &self.data.values
    }
    pub fn original_base(&self) -> Option<&str> {
        self.data.original_base.as_deref()
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
pub(super) struct DeclarationBuild {
    enabled: bool,
    examined: usize,
    pub grouped: BTreeMap<String, Vec<ReferenceRecord>>,
    pending: BTreeMap<String, Vec<DeclarationRow>>,
    pub sources: Vec<NativeViewSource>,
    pub issues: Vec<NativeViewIssue>,
}
impl DeclarationBuild {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            examined: 0,
            grouped: BTreeMap::new(),
            pending: BTreeMap::new(),
            sources: Vec::new(),
            issues: Vec::new(),
        }
    }
    pub fn capture(
        &mut self,
        doc: &DocumentationDocument,
        system: &SystemFacts<'_>,
        profile: &str,
        bytes: &mut usize,
        stop: &AtomicBool,
    ) -> Result<(), NativeError> {
        let namespace = match system.owner {
            SystemOwner::Namespace(name) => Some(name),
            _ => None,
        };
        if let SystemOwner::ScriptObject(name) = system.owner {
            self.row(
                doc,
                profile,
                system.raw,
                DeclarationInput {
                    kind: NativeDeclarationKind::ScriptObject,
                    name,
                    namespace,
                    fields: &[],
                    arguments: &[],
                    returns: &[],
                    values: &[],
                    base: system.widget_base,
                },
                bytes,
                stop,
            )?;
        }
        for table in &system.tables {
            checkpoint(stop)?;
            let (kind, name, fields, arguments, returns, values, raw) = match table {
                TableFact::Structure { name, fields, raw } => (
                    NativeDeclarationKind::Structure,
                    *name,
                    fields.as_slice(),
                    &[][..],
                    &[][..],
                    &[][..],
                    *raw,
                ),
                TableFact::Callback {
                    name,
                    arguments,
                    returns,
                    raw,
                } => (
                    NativeDeclarationKind::Callback,
                    *name,
                    &[][..],
                    arguments.as_slice(),
                    returns.as_slice(),
                    &[][..],
                    *raw,
                ),
                TableFact::Enumeration { name, values, raw } => (
                    NativeDeclarationKind::Enumeration,
                    *name,
                    &[][..],
                    &[][..],
                    &[][..],
                    values.as_slice(),
                    *raw,
                ),
                TableFact::Constants { name, values, raw } => (
                    NativeDeclarationKind::Constants,
                    *name,
                    &[][..],
                    &[][..],
                    &[][..],
                    values.as_slice(),
                    *raw,
                ),
                TableFact::Unsupported { raw, .. } => {
                    self.issues
                        .push(issue("declaration_kind_unsupported", doc, raw.span));
                    continue;
                }
            };
            self.row(
                doc,
                profile,
                raw,
                DeclarationInput {
                    kind,
                    name,
                    namespace,
                    fields,
                    arguments,
                    returns,
                    values,
                    base: None,
                },
                bytes,
                stop,
            )?;
        }
        Ok(())
    }
    fn row(
        &mut self,
        doc: &DocumentationDocument,
        profile: &str,
        raw: &crate::native::RawValue,
        input: DeclarationInput<'_>,
        bytes: &mut usize,
        stop: &AtomicBool,
    ) -> Result<(), NativeError> {
        checkpoint(stop)?;
        self.examined = self
            .examined
            .checked_add(1)
            .ok_or_else(|| error(NativeErrorCode::Limit))?;
        if self.examined > 65_536 {
            return Err(error(NativeErrorCode::Limit));
        }
        let payload = crate::wire_json::canonical_json_bytes(raw)
            .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
        if payload.len() > 65_536 {
            self.issues
                .push(issue("declaration_payload_limit", doc, raw.span));
            return Ok(());
        }
        add_candidate_bytes(bytes, payload.len())?;
        let id = source_id(profile, "declaration", doc.revision(), doc, raw.span)?;
        let key = format!("declaration:{}:{}", input.kind.key_prefix(), input.name);
        let record = match ReferenceRecord::new(
            &key,
            input.kind.record_kind(),
            String::from_utf8(payload).map_err(|_| error(NativeErrorCode::InvalidEncoding))?,
            vec![id.clone()],
            Vec::new(),
        ) {
            Ok(record) => record,
            Err(_) => {
                self.issues
                    .push(issue("declaration_record_rejected", doc, raw.span));
                return Ok(());
            }
        };
        let data = input.capture(stop)?;
        let row = DeclarationRow {
            documentation: CapturedDocumentation::capture(raw, stop)?,
            digest: record
                .digest()
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
            source_id: id.clone(),
            data,
        };
        self.sources.push(NativeViewSource {
            id,
            path: doc.path().into(),
            sha256: doc.sha256().into(),
            span: raw.span,
        });
        self.pending.entry(key.clone()).or_default().push(row);
        self.grouped.entry(key).or_default().push(record);
        Ok(())
    }
    pub fn finish(
        mut self,
        view: &ReferenceView,
        stop: &AtomicBool,
    ) -> Result<DeclarationCatalog, NativeError> {
        let mut catalog = DeclarationCatalog {
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
            .find(|part| part.id() == NATIVE_DECLARATION_PARTITION)
            .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
        for record in partition.records() {
            checkpoint(stop)?;
            let mut rows = self
                .pending
                .remove(record.key())
                .filter(|rows| rows.len() == 1)
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
            let row = rows
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
struct DeclarationInput<'a> {
    kind: NativeDeclarationKind,
    name: &'a str,
    namespace: Option<&'a str>,
    fields: &'a [FieldFact<'a>],
    arguments: &'a [FieldFact<'a>],
    returns: &'a [FieldFact<'a>],
    values: &'a [ValueFact<'a>],
    base: Option<&'a str>,
}
impl DeclarationInput<'_> {
    fn capture(self, stop: &AtomicBool) -> Result<DeclarationData, NativeError> {
        let fields = |values: &[FieldFact<'_>]| -> Result<Vec<NativeCallableField>, NativeError> {
            let mut output = Vec::new();
            for value in values {
                checkpoint(stop)?;
                output.push(NativeCallableField::capture(value));
            }
            Ok(output)
        };
        let mut values = Vec::new();
        for value in self.values {
            checkpoint(stop)?;
            let observed = match &value.value.kind {
                RawKind::Boolean(value) => NativeDeclarationScalar::Boolean(*value),
                RawKind::Number(value) => NativeDeclarationScalar::NumberLexeme(value.clone()),
                RawKind::String(value) => NativeDeclarationScalar::String(value.clone()),
                _ => NativeDeclarationScalar::UnresolvedDigest(source_digest(
                    &crate::wire_json::canonical_json_bytes(value.value)
                        .map_err(|_| error(NativeErrorCode::InvalidIdentity))?,
                )),
            };
            values.push(NativeDeclarationValue {
                name: value.name.into(),
                original_type: value.type_name.map(str::to_owned),
                value: observed,
                source_span: value.raw.span,
            });
        }
        Ok(DeclarationData {
            kind: self.kind,
            name: self.name.into(),
            definition_namespace: self.namespace.map(str::to_owned),
            fields: fields(self.fields)?,
            arguments: fields(self.arguments)?,
            returns: fields(self.returns)?,
            values,
            original_base: self.base.map(str::to_owned),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DeclarationCatalog {
    generation: String,
    view_digest: String,
    rows: BTreeMap<String, DeclarationRow>,
}
impl DeclarationCatalog {
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
            NATIVE_DECLARATION_PARTITION,
        )?;
        self.generation = next.generation_id().into();
        self.view_digest = next.self_digest().into();
        Ok(())
    }
    pub fn read<'a>(
        &'a self,
        record: &ReferenceRecord,
        stop: &AtomicBool,
    ) -> Result<NativeDeclarationSignature<'a>, NativeError> {
        checkpoint(stop)?;
        let (key, row) = self
            .rows
            .get_key_value(record.key())
            .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
        if record.kind() != row.data.kind.record_kind()
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
        Ok(NativeDeclarationSignature {
            profile: "wow-reference/native-source-declaration/1",
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

//! Source-normalized callable data captured during the original native projection.
//! No payload decoding, type resolution, annotation lowering or runtime authority.
use super::*;
use crate::native::RawValue;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NativeCallableField {
    name: String,
    type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    inner_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nilable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<RawValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stride_index: Option<RawValue>,
    source_span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NativeCallableSignatureData {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documentation: Option<Vec<String>>,
    arguments: Vec<NativeCallableField>,
    returns: Vec<NativeCallableField>,
    type_qualification: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SignatureRow {
    record_digest: String,
    source_id: String,
    data: NativeCallableSignatureData,
}

/// Genuine native owner data. No public constructor or deserialization path.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct NativeCallableSignature<'a> {
    profile: &'static str,
    reference_generation: &'a str,
    reference_view_digest: &'a str,
    record_key: &'a str,
    record_digest: &'a str,
    source_id: &'a str,
    signature: &'a NativeCallableSignatureData,
}
impl NativeCallableSignature<'_> {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SignatureCatalog {
    generation: String,
    view_digest: String,
    rows: BTreeMap<String, SignatureRow>,
}

impl SignatureRow {
    pub(super) fn capture(
        record: &ReferenceRecord,
        namespace: Option<&str>,
        function: &CallableFact<'_>,
        stop: &AtomicBool,
    ) -> Result<Self, NativeError> {
        checkpoint(stop)?;
        // Called only after the original 64 KiB payload and aggregate candidate
        // preflights. These disjoint field/default subtrees come from that same
        // admitted raw callable; no source or payload is parsed again.
        let fields = |input: &[FieldFact<'_>]| -> Result<Vec<NativeCallableField>, NativeError> {
            let mut result = Vec::with_capacity(input.len());
            for field in input {
                checkpoint(stop)?;
                result.push(NativeCallableField {
                    name: field.name.to_owned(),
                    type_name: field.type_name.to_owned(),
                    inner_type: field.inner_type.map(str::to_owned),
                    nilable: field.nilable,
                    default: field.default.cloned(),
                    stride_index: field.stride_index.cloned(),
                    source_span: field.raw.span,
                });
            }
            Ok(result)
        };
        let record_digest = record
            .digest()
            .map_err(|_| error(NativeErrorCode::InvalidIdentity))?;
        checkpoint(stop)?;
        Ok(Self {
            record_digest,
            source_id: record
                .source_ids()
                .first()
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?
                .to_string(),
            data: NativeCallableSignatureData {
                name: function.name.to_owned(),
                namespace: namespace.map(str::to_owned),
                documentation: function
                    .documentation
                    .as_ref()
                    .map(|rows| rows.iter().map(|row| (*row).to_owned()).collect()),
                arguments: fields(&function.arguments)?,
                returns: fields(&function.returns)?,
                type_qualification: "original_source_type_identities_unresolved",
            },
        })
    }
}

impl SignatureCatalog {
    pub(super) fn finish(
        view: &ReferenceView,
        mut pending: BTreeMap<String, Vec<SignatureRow>>,
        stop: &AtomicBool,
    ) -> Result<Self, NativeError> {
        let partition = view
            .partitions()
            .iter()
            .find(|partition| partition.id() == NATIVE_API_PARTITION)
            .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
        let mut rows = BTreeMap::new();
        for record in partition.records() {
            checkpoint(stop)?;
            // Materialization admits only a unique original callable. Conflict
            // candidates remain in the producer report and cannot select a row.
            let mut group = pending
                .remove(record.key())
                .filter(|group| group.len() == 1)
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
            let row = group
                .pop()
                .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
            if record.source_ids().len() != 1 || record.source_ids()[0].as_ref() != row.source_id {
                return Err(error(NativeErrorCode::InvalidIdentity));
            }
            rows.insert(record.key().to_owned(), row);
        }
        Ok(Self {
            generation: view.generation_id().to_owned(),
            view_digest: view.self_digest().to_owned(),
            rows,
        })
    }

    pub(super) fn read<'a>(
        &'a self,
        record: &ReferenceRecord,
        stop: &AtomicBool,
    ) -> Result<NativeCallableSignature<'a>, NativeError> {
        checkpoint(stop)?;
        let (key, row) = self
            .rows
            .get_key_value(record.key())
            .ok_or_else(|| error(NativeErrorCode::InvalidIdentity))?;
        if record.kind() != ReferenceRecordKind::Api
            || record.payload().len() > 65_536
            || record.source_ids().len() != 1
            || record.source_ids()[0].as_ref() != row.source_id
            || !record.restrictions().is_empty()
            || record
                .digest()
                .map_err(|_| error(NativeErrorCode::InvalidIdentity))?
                != row.record_digest
        {
            return Err(error(NativeErrorCode::InvalidIdentity));
        }
        checkpoint(stop)?;
        Ok(NativeCallableSignature {
            profile: "wow-reference/native-source-callable-signature/1",
            reference_generation: &self.generation,
            reference_view_digest: &self.view_digest,
            record_key: key,
            record_digest: &row.record_digest,
            source_id: &row.source_id,
            signature: &row.data,
        })
    }
}

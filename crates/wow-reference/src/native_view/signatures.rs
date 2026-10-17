//! Source-normalized callable data captured during the original native projection.
//! No payload decoding, type resolution, annotation lowering or runtime authority.
use super::type_resolution::NativeCallableTypeResolution;
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

impl NativeCallableField {
    pub(super) fn capture(field: &FieldFact<'_>) -> Self {
        Self {
            name: field.name.to_owned(),
            type_name: field.type_name.to_owned(),
            inner_type: field.inner_type.map(str::to_owned),
            nilable: field.nilable,
            default: field.default.cloned(),
            stride_index: field.stride_index.cloned(),
            source_span: field.raw.span,
        }
    }
    /// Original captured member name, exactly as admitted from the source payload.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    /// Original captured member type text. This is the source-normalized type
    /// identity exactly as captured; it stays unresolved because this reader
    /// performs no type resolution or annotation lowering, and any Resolution
    /// overlay is owned by `NativeCallableTypeResolution`, not by this field.
    pub fn type_name(&self) -> &str {
        self.type_name.as_str()
    }
    /// Original inner type text when the source stated one; `None` is not a claim
    /// that the member has no inner type.
    pub fn inner_type(&self) -> Option<&str> {
        self.inner_type.as_deref()
    }
    /// Tri-state nilability exactly as captured: `None` when the source stated
    /// nothing, `Some(true)` when it explicitly allowed nil, `Some(false)` when
    /// it explicitly disallowed nil.
    pub fn nilable(&self) -> Option<bool> {
        self.nilable
    }
    /// Original raw member range; it may refer to an explicitly reused local table.
    pub const fn source_span(&self) -> Span {
        self.source_span
    }

    pub(super) fn raw_default(&self) -> Option<&RawValue> {
        self.default.as_ref()
    }

    pub(super) fn raw_stride_index(&self) -> Option<&RawValue> {
        self.stride_index.as_ref()
    }
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
    type_resolution: Option<NativeCallableTypeResolution>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    type_resolution: Option<&'a NativeCallableTypeResolution>,
}
impl NativeCallableSignature<'_> {
    /// Original captured callable name from the sealed signature catalog;
    /// it is not a resolved or corrected identity.
    pub fn name(&self) -> &str {
        self.signature.name.as_str()
    }
    /// Original captured callable namespace when the source stated one;
    /// `None` is the admitted global-owner case; no namespace is inferred.
    pub fn namespace(&self) -> Option<&str> {
        self.signature.namespace.as_deref()
    }
    /// Original ordered documentation, borrowed from the sealed source cache.
    pub fn documentation(&self) -> NativeDocumentation<'_> {
        self.signature.documentation.as_deref().map_or(
            NativeDocumentation::NotStated,
            NativeDocumentation::Recorded,
        )
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
    /// Ordered original source fields, borrowed from the sealed signature catalog.
    pub fn arguments(&self) -> &[NativeCallableField] {
        &self.signature.arguments
    }
    pub fn returns(&self) -> &[NativeCallableField] {
        &self.signature.returns
    }
    pub const fn type_resolution(&self) -> Option<&NativeCallableTypeResolution> {
        self.type_resolution
    }
    /// Exact canonical JSON size of the selected overlay; absent on legacy reads.
    pub const fn type_resolution_encoded_len(&self) -> Option<usize> {
        match self.type_resolution {
            Some(resolution) => Some(resolution.encoded_len()),
            None => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SignatureCatalog {
    projection_profile: &'static str,
    generation: String,
    view_digest: String,
    rows: BTreeMap<String, SignatureRow>,
}

impl SignatureRow {
    pub(super) fn capture(
        record: &ReferenceRecord,
        namespace: Option<&str>,
        function: &CallableFact<'_>,
        type_resolution: Option<NativeCallableTypeResolution>,
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
                result.push(NativeCallableField::capture(field));
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
            type_resolution,
        })
    }
}

impl SignatureCatalog {
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
            NATIVE_API_PARTITION,
        )?;
        self.generation = next.generation_id().into();
        self.view_digest = next.self_digest().into();
        Ok(())
    }
    pub(super) fn finish(
        projection_profile: &'static str,
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
            projection_profile,
            generation: view.generation_id().to_owned(),
            view_digest: view.self_digest().to_owned(),
            rows,
        })
    }

    pub(super) fn require_view(
        &self,
        projection_profile: &str,
        view: &ReferenceView,
        stop: &AtomicBool,
    ) -> Result<(), NativeError> {
        checkpoint(stop)?;
        if self.projection_profile != projection_profile
            || self.generation != view.generation_id()
            || self.view_digest != view.self_digest()
        {
            return Err(error(NativeErrorCode::InvalidIdentity));
        }
        Ok(())
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
            type_resolution: row.type_resolution.as_ref(),
        })
    }
}

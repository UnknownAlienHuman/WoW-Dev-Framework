//! Lossless renderings of a sealed primary pack; labels remain quoted data.
use std::fmt;
use std::sync::atomic::AtomicBool;

use serde::ser::{Error as _, SerializeSeq};
use serde::{Serialize, Serializer};
use sha2::{Digest as _, Sha256};
use wow_core::{CanonicalResult, ContentDigest};

use crate::budget::{Budget, canonical, checkpoint, exhausted};
use crate::pack::{ContextSemanticPack, JSON_PROFILE, MARKDOWN_PROFILE, PackCore, RendererProfile};
use crate::{ContextError, ContextErrorCode, ContextResult};

const SCHEMA: &str = "wow-context/rendered-context-artifact/primary/1";
const DIGEST_BASIS: &str = "sha256_exact_rendered_utf8_bytes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderSelection {
    None,
    Json,
    Markdown,
    Both,
}

/// Ranges include the complete fixed record, including its LF in Markdown.
#[derive(Debug, Serialize)]
pub struct ContextRenderTrace {
    record_id: Box<str>,
    template_id: &'static str,
    semantic_fields: &'static [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    item_index: Option<usize>,
    byte_start: u64,
    byte_end: u64,
}
impl ContextRenderTrace {
    pub fn record_id(&self) -> &str {
        &self.record_id
    }
    pub const fn template_id(&self) -> &'static str {
        self.template_id
    }
    pub const fn semantic_fields(&self) -> &'static [&'static str] {
        self.semantic_fields
    }
    pub const fn item_index(&self) -> Option<usize> {
        self.item_index
    }
    pub const fn byte_start(&self) -> u64 {
        self.byte_start
    }
    pub const fn byte_end(&self) -> u64 {
        self.byte_end
    }
}

#[derive(Serialize)]
struct RenderCore {
    schema: &'static str,
    semantic_pack_id: Box<str>,
    profile: &'static RendererProfile,
    content: Box<str>,
    byte_length: u64,
    content_digest: ContentDigest<CanonicalResult>,
    content_digest_basis: &'static str,
    trace: Vec<ContextRenderTrace>,
}

/// Serialize-only presentation. It grants no new semantic/source authority.
#[derive(Serialize)]
pub struct RenderedContextArtifact {
    rendered_context_artifact_id: Box<str>,
    core: RenderCore,
}
impl RenderedContextArtifact {
    pub fn rendered_context_artifact_id(&self) -> &str {
        &self.rendered_context_artifact_id
    }
    pub fn semantic_pack_id(&self) -> &str {
        &self.core.semantic_pack_id
    }
    pub const fn renderer_profile_id(&self) -> &'static str {
        self.core.profile.profile_id
    }
    pub fn content(&self) -> &str {
        &self.core.content
    }
    pub const fn byte_length(&self) -> u64 {
        self.core.byte_length
    }
    pub const fn content_digest(&self) -> ContentDigest<CanonicalResult> {
        self.core.content_digest
    }
    pub fn trace(&self) -> &[ContextRenderTrace] {
        &self.core.trace
    }
}

#[derive(Clone, Copy)]
enum Format {
    Json,
    Markdown,
}
impl Format {
    fn profile(self) -> &'static RendererProfile {
        match self {
            Self::Json => &JSON_PROFILE,
            Self::Markdown => &MARKDOWN_PROFILE,
        }
    }
}

struct Input<'p, 'm, 'a> {
    id: &'p str,
    core: &'p PackCore<'m, 'a>,
    canonical_bytes: &'p [u8],
}

#[derive(Serialize)]
struct TraceRef<'p> {
    record_id: &'p str,
    template_id: &'static str,
    semantic_fields: &'static [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    item_index: Option<usize>,
    byte_start: u64,
    byte_end: u64,
}

enum Payload<'p> {
    CanonicalPack(&'p str),
    QuotedJson(Box<str>),
}
impl Payload<'_> {
    fn text(&self) -> &str {
        match self {
            Self::CanonicalPack(text) => text,
            Self::QuotedJson(text) => text,
        }
    }
}

struct Record<'p> {
    label: &'static str,
    payload: Payload<'p>,
    trace: TraceRef<'p>,
}
struct Plan<'p> {
    records: Vec<Record<'p>>,
    byte_length: u64,
    digest: ContentDigest<CanonicalResult>,
}

fn encoding() -> ContextError {
    ContextError::new(ContextErrorCode::EncodingInvalid)
}

#[derive(Serialize)]
struct SectionRef<'r, 'p, T: Serialize + ?Sized> {
    record_id: &'p str,
    template_id: &'static str,
    semantic_fields: &'static [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    item_index: Option<usize>,
    payload: &'r T,
}

/// A cancellable dry sink measures and hashes exact UTF-8 before content copies.
struct Measure<'s> {
    stop: &'s AtomicBool,
    limit: u64,
    bytes: u64,
    hash: Sha256,
    exceeded: bool,
}
impl fmt::Write for Measure<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        checkpoint(self.stop).map_err(|_| fmt::Error)?;
        let length = u64::try_from(text.len()).map_err(|_| fmt::Error)?;
        let Some(total) = self.bytes.checked_add(length).filter(|n| *n <= self.limit) else {
            self.exceeded = true;
            return Err(fmt::Error);
        };
        self.bytes = total;
        self.hash.update(text.as_bytes());
        Ok(())
    }
}

struct Planner<'b, 's, 'p> {
    budget: &'b mut Budget,
    stop: &'s AtomicBool,
    records: Vec<Record<'p>>,
    measure: Measure<'s>,
    limit: u64,
}
impl<'p> Planner<'_, '_, 'p> {
    fn add<T: Serialize + ?Sized>(
        &mut self,
        record_id: &'p str,
        template_id: &'static str,
        label: &'static str,
        fields: &'static [&'static str],
        item_index: Option<usize>,
        value: &T,
    ) -> ContextResult<()> {
        self.budget.work(1, self.stop)?;
        // Count the borrowed complete section before its canonical scratch copy.
        self.budget.retain(
            &SectionRef {
                record_id,
                template_id,
                semantic_fields: fields,
                item_index,
                payload: value,
            },
            self.stop,
        )?;
        let raw = canonical(value, self.limit, self.stop)?;
        let payload = Payload::QuotedJson(
            String::from_utf8(raw)
                .map_err(|_| encoding())?
                .into_boxed_str(),
        );
        self.add_payload(record_id, template_id, label, fields, item_index, payload)
    }

    fn add_payload(
        &mut self,
        record_id: &'p str,
        template_id: &'static str,
        label: &'static str,
        fields: &'static [&'static str],
        item_index: Option<usize>,
        payload: Payload<'p>,
    ) -> ContextResult<()> {
        // One conservative unit per input UTF-8 byte covers each character visit.
        self.budget.work(payload.text().len(), self.stop)?;
        let mut record = Record {
            label,
            payload,
            trace: TraceRef {
                record_id,
                template_id,
                semantic_fields: fields,
                item_index,
                byte_start: self.measure.bytes,
                byte_end: self.measure.bytes,
            },
        };
        let emitted = emit_record(&record, &mut self.measure);
        checkpoint(self.stop)?;
        if self.measure.exceeded {
            return Err(exhausted());
        }
        emitted.map_err(|_| encoding())?;
        record.trace.byte_end = self.measure.bytes;
        // The owned range/list entry is retained only after its borrowed charge.
        self.budget.retain(&record.trace, self.stop)?;
        self.records.push(record);
        Ok(())
    }

    fn finish(self) -> Plan<'p> {
        Plan {
            records: self.records,
            byte_length: self.measure.bytes,
            digest: ContentDigest::from_bytes(self.measure.hash.finalize().into()),
        }
    }
}

#[derive(Serialize)]
struct Header<'p> {
    schema: &'static str,
    semantic_pack_id: &'p str,
    /// Framing cardinality also represents an explicitly empty item array.
    item_count: usize,
}

fn prepare<'p>(
    input: &Input<'p, '_, '_>,
    format: Format,
    budget: &mut Budget,
    limit: u64,
    stop: &AtomicBool,
) -> ContextResult<Plan<'p>> {
    checkpoint(stop)?;
    let mut planner = Planner {
        budget,
        stop,
        records: Vec::new(),
        measure: Measure {
            stop,
            limit,
            bytes: 0,
            hash: Sha256::new(),
            exceeded: false,
        },
        limit,
    };
    if matches!(format, Format::Json) {
        let text = std::str::from_utf8(input.canonical_bytes).map_err(|_| encoding())?;
        planner.add_payload(
            input.id,
            "canonical_pack_json",
            "",
            &["semantic_pack_id", "core"],
            None,
            Payload::CanonicalPack(text),
        )?;
    } else {
        let core = input.core;
        planner.add(
            input.id,
            "pack_header",
            "PACK",
            &["semantic_pack_id", "core.schema"],
            None,
            &Header {
                schema: core.schema,
                semantic_pack_id: input.id,
                item_count: core.items.len(),
            },
        )?;
        planner.add(
            input.id,
            "pack_profile",
            "PROFILE",
            &["core.profile"],
            None,
            core.profile,
        )?;
        planner.add(
            input.id,
            "universe_binding",
            "UNIVERSE",
            &["core.universe_set_id", "core.binding"],
            None,
            &(core.universe_set_id, core.binding),
        )?;
        planner.add(
            input.id,
            "pack_request",
            "REQUEST",
            &["core.request"],
            None,
            &core.request,
        )?;
        planner.add(
            input.id,
            "boundary_notice",
            "BOUNDARY",
            &["core.boundary_notice"],
            None,
            core.boundary_notice,
        )?;
        planner.add(
            input.id,
            "native_skeletons",
            "SKELETONS",
            &["core.skeletons"],
            None,
            &core.skeletons,
        )?;
        for (index, item) in core.items.iter().enumerate() {
            checkpoint(stop)?;
            let (template, label) = match item.core.payload.template() {
                "native_entity_item" => ("native_entity_item", "ENTITY"),
                "native_relation_item" => ("native_relation_item", "RELATION"),
                _ => return Err(encoding()),
            };
            planner.add(
                item.context_item_id(),
                template,
                label,
                &["core.items"],
                Some(index),
                item,
            )?;
        }
        planner.add(
            input.id,
            "selection_trace",
            "SELECTION_TRACE",
            &["core.selection_trace"],
            None,
            &core.selection_trace,
        )?;
        planner.add(
            input.id,
            "coverage_and_omissions",
            "COVERAGE",
            &["core.omissions", "core.state", "core.absence_authoritative"],
            None,
            &(&core.omissions, core.state, core.absence_authoritative),
        )?;
        planner.add(
            input.id,
            "pack_budget",
            "BUDGET",
            &["core.budget"],
            None,
            &core.budget,
        )?;
    }
    Ok(planner.finish())
}

/// Unicode escapes alter only representation; decoding yields the same string.
/// Quotes/backslashes retain JSON escaping. All other ASCII punctuation is
/// Unicode-escaped, including every Markdown/HTML delimiter. Bidi formatting,
/// directional marks, Unicode line separators and controls cannot change lines.
fn emit_record(record: &Record<'_>, output: &mut impl fmt::Write) -> fmt::Result {
    let quoted = matches!(record.payload, Payload::QuotedJson(_));
    if quoted {
        output.write_str(record.label)?;
        output.write_str(" \"")?;
    }
    for ch in record.payload.text().chars() {
        if !quoted {
            output.write_char(ch)?;
        } else {
            match ch {
                '"' => output.write_str("\\\"")?,
                '\\' => output.write_str("\\\\")?,
                ch if ch.is_ascii_punctuation()
                    || ch.is_control()
                    || matches!(
                        ch,
                        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{2028}' | '\u{2029}'
                    )
                    || ('\u{202a}'..='\u{202e}').contains(&ch)
                    || ('\u{2066}'..='\u{2069}').contains(&ch) =>
                {
                    // Selected controls are BMP; supplementary values remain UTF-8.
                    write!(output, "\\u{:04x}", u32::from(ch))?;
                }
                ch => output.write_char(ch)?,
            }
        }
    }
    if quoted {
        output.write_str("\"\n")?;
    }
    Ok(())
}

struct Content<'r> {
    records: &'r [Record<'r>],
}
impl fmt::Display for Content<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for record in self.records {
            emit_record(record, formatter)?;
        }
        Ok(())
    }
}
impl Serialize for Content<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // serde_json's writer streams this dry representation through the
        // bounded counter. Formatting failures must originate in that writer:
        // serde_json's collect_str adapter cannot handle an independent fmt::Error.
        // The counter checks cancellation on every write; other emit sinks do too.
        serializer.collect_str(self)
    }
}

struct Trace<'r, 's> {
    records: &'r [Record<'r>],
    stop: &'s AtomicBool,
}
impl Serialize for Trace<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.records.len()))?;
        for record in self.records {
            checkpoint(self.stop).map_err(S::Error::custom)?;
            sequence.serialize_element(&record.trace)?;
        }
        sequence.end()
    }
}

#[derive(Serialize)]
struct CoreRef<'p, 'r, 's> {
    schema: &'static str,
    semantic_pack_id: &'p str,
    profile: &'static RendererProfile,
    content: Content<'r>,
    byte_length: u64,
    content_digest: ContentDigest<CanonicalResult>,
    content_digest_basis: &'static str,
    trace: Trace<'r, 's>,
}
fn core_ref<'p, 'r, 's>(
    id: &'p str,
    plan: &'r Plan<'r>,
    format: Format,
    stop: &'s AtomicBool,
) -> CoreRef<'p, 'r, 's> {
    CoreRef {
        schema: SCHEMA,
        semantic_pack_id: id,
        profile: format.profile(),
        content: Content {
            records: &plan.records,
        },
        byte_length: plan.byte_length,
        content_digest: plan.digest,
        content_digest_basis: DIGEST_BASIS,
        trace: Trace {
            records: &plan.records,
            stop,
        },
    }
}

// Same canonical field material as CoreRef/RenderCore, now borrowing the
// admitted actual content. No formatter is invoked by canonical ID hashing.
#[derive(Serialize)]
struct CompleteCoreRef<'p, 'r, 's> {
    schema: &'static str,
    semantic_pack_id: &'p str,
    profile: &'static RendererProfile,
    content: &'p str,
    byte_length: u64,
    content_digest: ContentDigest<CanonicalResult>,
    content_digest_basis: &'static str,
    trace: Trace<'r, 's>,
}

#[derive(Serialize)]
struct IdMaterial<'r, T: Serialize + ?Sized> {
    domain: &'static str,
    value: &'r T,
}
#[derive(Serialize)]
struct ArtifactRef<'r, T: Serialize + ?Sized> {
    rendered_context_artifact_id: &'r str,
    core: &'r T,
}

/// Every repeated character/trace walk uses the held operation ledger first.
fn pass(plan: &Plan<'_>, budget: &mut Budget, stop: &AtomicBool) -> ContextResult<()> {
    let mut amount = plan.records.len().checked_mul(2).ok_or_else(exhausted)?;
    // Charge this accounting walk before iterating the records.
    budget.work(plan.records.len(), stop)?;
    for record in &plan.records {
        amount = amount
            .checked_add(record.payload.text().len())
            .ok_or_else(exhausted)?;
    }
    budget.work(amount, stop)
}

fn artifact_id(value: &impl Serialize, stop: &AtomicBool) -> ContextResult<Box<str>> {
    checkpoint(stop)?;
    let hash = wow_core::domain_separated_digest(SCHEMA, value);
    checkpoint(stop)?;
    let digest = ContentDigest::<CanonicalResult>::from_bytes(hash.map_err(|_| encoding())?);
    Ok(format!("rendered-context:{digest}").into_boxed_str())
}

struct Output<'s> {
    text: String,
    limit: u64,
    stop: &'s AtomicBool,
    exceeded: bool,
}
impl fmt::Write for Output<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        checkpoint(self.stop).map_err(|_| fmt::Error)?;
        let total = self.text.len().checked_add(text.len()).ok_or(fmt::Error)?;
        if u64::try_from(total).map_err(|_| fmt::Error)? > self.limit {
            self.exceeded = true;
            return Err(fmt::Error);
        }
        self.text.push_str(text);
        Ok(())
    }
}

fn render(
    pack: &mut ContextSemanticPack<'_, '_>,
    format: Format,
    stop: &AtomicBool,
) -> ContextResult<RenderedContextArtifact> {
    let limit = pack.render_limit();
    // Root-owned crate-private split; it lends the SAME held ledger.
    let (id, core, canonical_bytes, budget) = pack.render_parts();
    let input = Input {
        id,
        core,
        canonical_bytes,
    };
    let plan = prepare(&input, format, budget, limit, stop)?;
    let core = core_ref(id, &plan, format, stop);
    pass(&plan, budget, stop)?;
    budget.retain(&core, stop)?;
    pass(&plan, budget, stop)?;
    budget.retain(
        &IdMaterial {
            domain: SCHEMA,
            value: &core,
        },
        stop,
    )?;
    // Complete borrowed core and identity envelope have passed cumulative
    // admission. Only now allocate owned content. Virtual Content is used ONLY
    // by serde_json's bounded streaming counter, never core canonical hashing:
    // default collect_str may allocate via Display::to_string, which cannot
    // safely propagate a mid-format cancellation as a formatting error.
    pass(&plan, budget, stop)?;
    let mut output = Output {
        text: String::with_capacity(usize::try_from(plan.byte_length).map_err(|_| exhausted())?),
        limit,
        stop,
        exceeded: false,
    };
    for record in &plan.records {
        let emitted = emit_record(record, &mut output);
        checkpoint(stop)?;
        if output.exceeded {
            return Err(exhausted());
        }
        emitted.map_err(|_| encoding())?;
    }
    if u64::try_from(output.text.len()).map_err(|_| exhausted())? != plan.byte_length {
        return Err(encoding());
    }
    let complete_core = CompleteCoreRef {
        schema: SCHEMA,
        semantic_pack_id: id,
        profile: format.profile(),
        content: &output.text,
        byte_length: plan.byte_length,
        content_digest: plan.digest,
        content_digest_basis: DIGEST_BASIS,
        trace: Trace {
            records: &plan.records,
            stop,
        },
    };
    budget.work(
        output
            .text
            .len()
            .checked_add(plan.records.len())
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let rendered_context_artifact_id = artifact_id(&complete_core, stop)?;
    budget.work(plan.records.len(), stop)?;
    budget.retain(
        &ArtifactRef {
            rendered_context_artifact_id: &rendered_context_artifact_id,
            core: &complete_core,
        },
        stop,
    )?;
    budget.work(plan.records.len(), stop)?;
    let mut trace = Vec::with_capacity(plan.records.len());
    for record in &plan.records {
        checkpoint(stop)?;
        trace.push(ContextRenderTrace {
            record_id: record.trace.record_id.into(),
            template_id: record.trace.template_id,
            semantic_fields: record.trace.semantic_fields,
            item_index: record.trace.item_index,
            byte_start: record.trace.byte_start,
            byte_end: record.trace.byte_end,
        });
    }
    checkpoint(stop)?;
    Ok(RenderedContextArtifact {
        rendered_context_artifact_id,
        core: RenderCore {
            schema: SCHEMA,
            semantic_pack_id: id.into(),
            profile: format.profile(),
            content: output.text.into_boxed_str(),
            byte_length: plan.byte_length,
            content_digest: plan.digest,
            content_digest_basis: DIGEST_BASIS,
            trace,
        },
    })
}

pub fn render_context_pack_json(
    pack: &mut ContextSemanticPack<'_, '_>,
    stop: &AtomicBool,
) -> ContextResult<RenderedContextArtifact> {
    render(pack, Format::Json, stop)
}

pub fn render_context_pack_markdown(
    pack: &mut ContextSemanticPack<'_, '_>,
    stop: &AtomicBool,
) -> ContextResult<RenderedContextArtifact> {
    render(pack, Format::Markdown, stop)
}

struct Compare<'p, 's> {
    actual: &'p str,
    offset: usize,
    stop: &'s AtomicBool,
}
impl fmt::Write for Compare<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        checkpoint(self.stop).map_err(|_| fmt::Error)?;
        let end = self.offset.checked_add(text.len()).ok_or(fmt::Error)?;
        if self.actual.get(self.offset..end) != Some(text) {
            return Err(fmt::Error);
        }
        self.offset = end;
        Ok(())
    }
}

fn raw_digest(
    content: &str,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<ContentDigest<CanonicalResult>> {
    budget.work(content.len(), stop)?;
    let mut hash = Sha256::new();
    for bytes in content.as_bytes().chunks(4096) {
        checkpoint(stop)?;
        hash.update(bytes);
    }
    checkpoint(stop)?;
    Ok(ContentDigest::from_bytes(hash.finalize().into()))
}

/// Rechecks content, exact ordered trace, complete profile and raw-byte identity.
/// Expected Markdown uses the fixed typed records, never another source parser.
pub fn validate_rendered_context_artifact(
    pack: &mut ContextSemanticPack<'_, '_>,
    artifact: &RenderedContextArtifact,
    stop: &AtomicBool,
) -> ContextResult<()> {
    let limit = pack.render_limit();
    let (id, core, canonical_bytes, budget) = pack.render_parts();
    let input = Input {
        id,
        core,
        canonical_bytes,
    };
    budget.work(artifact.core.trace.len(), stop)?;
    if artifact.core.schema != SCHEMA
        || artifact.core.semantic_pack_id.as_ref() != id
        || artifact.core.byte_length > limit
        || artifact.core.byte_length
            != u64::try_from(artifact.core.content.len()).map_err(|_| exhausted())?
        || artifact.core.content_digest_basis != DIGEST_BASIS
    {
        return Err(encoding());
    }
    let format = match artifact.core.profile.profile_id {
        crate::pack::PRIMARY_CONTEXT_JSON_RENDERER_PROFILE => Format::Json,
        crate::pack::PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE => Format::Markdown,
        _ => return Err(encoding()),
    };
    // Profiles are exact compiled descriptors, not caller-selectable templates.
    budget.work(format.profile().templates.len(), stop)?;
    budget.retain(&(artifact.core.profile, format.profile()), stop)?;
    if canonical(artifact.core.profile, limit, stop)? != canonical(format.profile(), limit, stop)? {
        return Err(encoding());
    }
    let expected = prepare(&input, format, budget, limit, stop)?;
    if artifact.core.trace.len() != expected.records.len()
        || artifact.core.byte_length != expected.byte_length
        || artifact.core.content_digest != expected.digest
    {
        return Err(encoding());
    }
    budget.work(expected.records.len(), stop)?;
    for (actual, record) in artifact.core.trace.iter().zip(&expected.records) {
        checkpoint(stop)?;
        let target = &record.trace;
        if actual.record_id.as_ref() != target.record_id
            || actual.template_id != target.template_id
            || actual.semantic_fields != target.semantic_fields
            || actual.item_index != target.item_index
            || actual.byte_start != target.byte_start
            || actual.byte_end != target.byte_end
            || artifact
                .core
                .content
                .get(
                    usize::try_from(actual.byte_start).map_err(|_| exhausted())?
                        ..usize::try_from(actual.byte_end).map_err(|_| exhausted())?,
                )
                .is_none()
        {
            return Err(encoding());
        }
    }
    if matches!(format, Format::Json) {
        budget.work(artifact.core.content.len(), stop)?;
        if artifact.core.content.as_bytes() != canonical_bytes {
            return Err(encoding());
        }
    } else {
        // Re-emission compares every exact boundary/payload byte without a new
        // aggregate content copy, and shares all preparation reservations.
        pass(&expected, budget, stop)?;
        budget.work(artifact.core.content.len(), stop)?;
        let mut comparison = Compare {
            actual: &artifact.core.content,
            offset: 0,
            stop,
        };
        for record in &expected.records {
            let compared = emit_record(record, &mut comparison);
            checkpoint(stop)?;
            compared.map_err(|_| encoding())?;
        }
        if comparison.offset != artifact.core.content.len() {
            return Err(encoding());
        }
    }
    if raw_digest(&artifact.core.content, budget, stop)? != artifact.core.content_digest {
        return Err(encoding());
    }
    let material = IdMaterial {
        domain: SCHEMA,
        value: &artifact.core,
    };
    budget.work(
        artifact
            .core
            .content
            .len()
            .checked_add(artifact.core.trace.len())
            .ok_or_else(exhausted)?,
        stop,
    )?;
    budget.retain(&material, stop)?;
    budget.work(
        artifact
            .core
            .content
            .len()
            .checked_add(artifact.core.trace.len())
            .ok_or_else(exhausted)?,
        stop,
    )?;
    if artifact_id(&artifact.core, stop)? != artifact.rendered_context_artifact_id {
        return Err(encoding());
    }
    checkpoint(stop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    use std::sync::atomic::Ordering;

    #[test]
    fn dry_content_cancellation_after_opening_quote_returns_error_without_panicking() {
        struct CancelAfterOpeningQuote<'s>(&'s AtomicBool);
        impl Write for CancelAfterOpeningQuote<'_> {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if self.0.load(Ordering::Acquire) {
                    return Err(io::Error::other("cancelled dry rendering"));
                }
                // collect_str has written its opening quote; cancellation now
                // occurs before the first Display payload byte reaches the writer.
                self.0.store(true, Ordering::Release);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let stop = AtomicBool::new(false);
        let records = [Record {
            label: "",
            payload: Payload::CanonicalPack(r#"{"schema":"fixture"}"#),
            trace: TraceRef {
                record_id: "fixture",
                template_id: "canonical_pack_json",
                semantic_fields: &["semantic_pack_id", "core"],
                item_index: None,
                byte_start: 0,
                byte_end: 0,
            },
        }];
        let content = Content { records: &records };
        let result = serde_json::to_writer(CancelAfterOpeningQuote(&stop), &content);
        assert!(result.is_err());
        assert!(stop.load(Ordering::Acquire));
        assert!(
            matches!(checkpoint(&stop), Err(error) if error.code() == ContextErrorCode::Cancelled)
        );
    }
}

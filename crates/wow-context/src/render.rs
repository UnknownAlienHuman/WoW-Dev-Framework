//! Lossless renderings of sealed native packs; labels remain quoted data.
use std::fmt;
use std::sync::atomic::AtomicBool;

use serde::ser::{Error as _, SerializeSeq};
use serde::{Serialize, Serializer};
use sha2::{Digest as _, Sha256};
use wow_core::{CanonicalResult, ContentDigest};

use crate::budget::{Budget, canonical, checkpoint, exhausted};
use crate::pack::source_excerpt::renderer_profiles as source_renderer_profiles;
use crate::pack::{
    COMBINED_RENDER_SCHEMA, CombinedContextSemanticPack, CombinedPackCore, ContextItemWire,
    ContextSemanticPack, ItemPayloadWire, JSON_PROFILE, MARKDOWN_PROFILE, OmissionsWire,
    PackBudget, PackBudgetWire, PackCore, PackOmissions, PackProfile, PackProfileWire, PackRequest,
    PrimaryPackRenderFormat, ProjectMapStateWire, RendererProfile, RequestWire, SelectionRecord,
    SelectionRecordWire, SkeletonsWire, UniverseBindingWire, ValidatedPrimaryL0Pack,
    combined_renderer_profiles, expansion_renderer_profiles,
};
use crate::skeleton::PrimarySkeletons;
use crate::universe::UniverseBinding;
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
#[derive(Clone, Copy)]
enum NativePackCore<'r, 'm, 'p, 'f> {
    Primary(&'r PackCore<'m, 'p>),
    Structural(&'r ValidatedPrimaryL0Pack),
    Combined(&'r CombinedPackCore<'m, 'p, 'f>),
}
impl Format {
    fn profile(
        self,
        core: NativePackCore<'_, '_, '_, '_>,
    ) -> ContextResult<&'static RendererProfile> {
        let (json, markdown) = match core {
            NativePackCore::Primary(core) => {
                if core.source_excerpt.is_some() {
                    source_renderer_profiles()
                } else if core.expansion.is_some() {
                    expansion_renderer_profiles(core)?
                } else {
                    (&JSON_PROFILE, &MARKDOWN_PROFILE)
                }
            }
            NativePackCore::Structural(_) => (&JSON_PROFILE, &MARKDOWN_PROFILE),
            NativePackCore::Combined(core) => combined_renderer_profiles(core)?,
        };
        Ok(match self {
            Self::Json => json,
            Self::Markdown => markdown,
        })
    }
}
struct Input<'r, 'm, 'p, 'f> {
    id: &'r str,
    core: NativePackCore<'r, 'm, 'p, 'f>,
    canonical_bytes: &'r [u8],
}
impl Input<'_, '_, '_, '_> {
    fn rendered_schema(&self) -> &'static str {
        match self.core {
            NativePackCore::Primary(core) if core.source_excerpt.is_some() => {
                crate::pack::SOURCE_RENDER_SCHEMA
            }
            NativePackCore::Primary(_) => SCHEMA,
            NativePackCore::Structural(_) => SCHEMA,
            NativePackCore::Combined(_) => COMBINED_RENDER_SCHEMA,
        }
    }
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
    SourceLine(&'p str),
}
impl Payload<'_> {
    fn text(&self) -> &str {
        match self {
            Self::CanonicalPack(text) => text,
            Self::QuotedJson(text) => text,
            Self::SourceLine(text) => text,
        }
    }
}

struct Record<'p> {
    label: &'static str,
    payload: Payload<'p>,
    trace: TraceRef<'p>,
    source_line: bool,
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
        let source_line = matches!(&payload, Payload::SourceLine(_));
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
            source_line,
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

    fn add_source_line(
        &mut self,
        record_id: &'p str,
        ordinal: usize,
        text: &'p str,
    ) -> ContextResult<()> {
        #[derive(Serialize)]
        struct SourceLineRef<'a> {
            ordinal: usize,
            text: &'a str,
        }
        self.budget
            .work(text.len().checked_add(32).ok_or_else(exhausted)?, self.stop)?;
        let line_number = ordinal.checked_add(1).ok_or_else(exhausted)?;
        self.budget.retain(
            &SourceLineRef {
                ordinal: line_number,
                text,
            },
            self.stop,
        )?;
        let mut record = Record {
            label: "",
            payload: Payload::SourceLine(text),
            trace: TraceRef {
                record_id,
                template_id: "source_data_line",
                semantic_fields: &["core.source_excerpt.text"],
                item_index: Some(line_number),
                byte_start: self.measure.bytes,
                byte_end: self.measure.bytes,
            },
            source_line: true,
        };
        let emitted = emit_record(&record, &mut self.measure);
        checkpoint(self.stop)?;
        if self.measure.exceeded {
            return Err(exhausted());
        }
        emitted.map_err(|_| encoding())?;
        record.trace.byte_end = self.measure.bytes;
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
    schema: &'p str,
    semantic_pack_id: &'p str,
    /// Framing cardinality also represents an explicitly empty item array.
    item_count: usize,
}

/// Borrowed metadata projections shared by sealed native packs and a
/// success-only StructuralOnly wire view. Implementations expose serialization
/// inputs only; they do not create or imply native custody.
trait PrimaryMetadataView<'p> {
    type Profile: Serialize;
    type Binding: Serialize;
    type Request: Serialize;
    type Skeletons: Serialize;
    type Item: Serialize;
    type Trace: Serialize;
    type Coverage: Serialize;
    type Budget: Serialize;

    fn schema(&'p self) -> &'p str;
    fn item_count(&self) -> usize;
    fn items_len(&self) -> usize;
    fn profile(&'p self) -> &'p Self::Profile;
    fn universe_binding(&'p self) -> (&'p str, &'p Self::Binding);
    fn request(&'p self) -> &'p Self::Request;
    fn boundary_notice(&'p self) -> &'p str;
    fn skeletons(&'p self) -> &'p Self::Skeletons;
    fn item(&'p self, index: usize) -> &'p Self::Item;
    fn item_id(&'p self, index: usize) -> &'p str;
    fn item_template_label(&self, index: usize) -> (&'static str, &'static str);
    fn trace(&'p self) -> &'p Self::Trace;
    fn coverage(&'p self) -> Self::Coverage;
    fn budget(&'p self) -> &'p Self::Budget;
}

impl<'p, 'm: 'p, 'a: 'p> PrimaryMetadataView<'p> for PackCore<'m, 'a> {
    type Profile = PackProfile;
    type Binding = UniverseBinding<'a>;
    type Request = PackRequest<'m>;
    type Skeletons = PrimarySkeletons<'m, 'a>;
    type Item = crate::pack::ContextItem<'m, 'a>;
    type Trace = Vec<SelectionRecord<'m>>;
    type Coverage = (&'p PackOmissions, &'p crate::ProjectMapState, bool);
    type Budget = PackBudget;

    fn schema(&'p self) -> &'p str {
        self.schema
    }
    fn item_count(&self) -> usize {
        self.items.len() + usize::from(self.source_excerpt.is_some())
    }
    fn items_len(&self) -> usize {
        self.items.len()
    }
    fn profile(&'p self) -> &'p Self::Profile {
        self.profile
    }
    fn universe_binding(&'p self) -> (&'p str, &'p Self::Binding) {
        (self.universe_set_id, self.binding)
    }
    fn request(&'p self) -> &'p Self::Request {
        &self.request
    }
    fn boundary_notice(&'p self) -> &'p str {
        self.boundary_notice
    }
    fn skeletons(&'p self) -> &'p Self::Skeletons {
        &self.skeletons
    }
    fn item(&'p self, index: usize) -> &'p Self::Item {
        &self.items[index]
    }
    fn item_id(&'p self, index: usize) -> &'p str {
        self.items[index].context_item_id()
    }
    fn item_template_label(&self, index: usize) -> (&'static str, &'static str) {
        match self.items[index].core.payload.template() {
            "native_entity_item" => ("native_entity_item", "ENTITY"),
            "native_relation_item" => ("native_relation_item", "RELATION"),
            _ => ("", ""),
        }
    }
    fn trace(&'p self) -> &'p Self::Trace {
        &self.selection_trace
    }
    fn coverage(&'p self) -> Self::Coverage {
        (&self.omissions, &self.state, self.absence_authoritative)
    }
    fn budget(&'p self) -> &'p Self::Budget {
        &self.budget
    }
}

impl<'p> PrimaryMetadataView<'p> for ValidatedPrimaryL0Pack {
    type Profile = PackProfileWire;
    type Binding = UniverseBindingWire;
    type Request = RequestWire;
    type Skeletons = SkeletonsWire;
    type Item = ContextItemWire;
    type Trace = Vec<SelectionRecordWire>;
    type Coverage = (&'p OmissionsWire, &'p ProjectMapStateWire, bool);
    type Budget = PackBudgetWire;

    fn schema(&'p self) -> &'p str {
        &self.core().schema
    }
    fn item_count(&self) -> usize {
        self.core().items.len()
    }
    fn items_len(&self) -> usize {
        self.core().items.len()
    }
    fn profile(&'p self) -> &'p Self::Profile {
        &self.core().profile
    }
    fn universe_binding(&'p self) -> (&'p str, &'p Self::Binding) {
        (&self.core().universe_set_id, &self.core().binding)
    }
    fn request(&'p self) -> &'p Self::Request {
        &self.core().request
    }
    fn boundary_notice(&'p self) -> &'p str {
        &self.core().boundary_notice
    }
    fn skeletons(&'p self) -> &'p Self::Skeletons {
        &self.core().skeletons
    }
    fn item(&'p self, index: usize) -> &'p Self::Item {
        &self.core().items[index]
    }
    fn item_id(&'p self, index: usize) -> &'p str {
        &self.core().items[index].context_item_id
    }
    fn item_template_label(&self, index: usize) -> (&'static str, &'static str) {
        match &self.core().items[index].core.payload {
            ItemPayloadWire::Entity { .. } => ("native_entity_item", "ENTITY"),
            ItemPayloadWire::Relation { .. } => ("native_relation_item", "RELATION"),
        }
    }
    fn trace(&'p self) -> &'p Self::Trace {
        &self.core().selection_trace
    }
    fn coverage(&'p self) -> Self::Coverage {
        (
            &self.core().omissions,
            &self.core().state,
            self.core().absence_authoritative,
        )
    }
    fn budget(&'p self) -> &'p Self::Budget {
        &self.core().budget
    }
}

fn prepare_primary_metadata_head<'p, V: PrimaryMetadataView<'p>>(
    planner: &mut Planner<'_, '_, 'p>,
    id: &'p str,
    view: &'p V,
) -> ContextResult<()> {
    planner.add(
        id,
        "pack_header",
        "PACK",
        &["semantic_pack_id", "core.schema"],
        None,
        &Header {
            schema: view.schema(),
            semantic_pack_id: id,
            item_count: view.item_count(),
        },
    )?;
    planner.add(
        id,
        "pack_profile",
        "PROFILE",
        &["core.profile"],
        None,
        view.profile(),
    )?;
    let binding = view.universe_binding();
    planner.add(
        id,
        "universe_binding",
        "UNIVERSE",
        &["core.universe_set_id", "core.binding"],
        None,
        &binding,
    )?;
    planner.add(
        id,
        "pack_request",
        "REQUEST",
        &["core.request"],
        None,
        view.request(),
    )?;
    planner.add(
        id,
        "boundary_notice",
        "BOUNDARY",
        &["core.boundary_notice"],
        None,
        view.boundary_notice(),
    )?;
    planner.add(
        id,
        "native_skeletons",
        "SKELETONS",
        &["core.skeletons"],
        None,
        view.skeletons(),
    )?;
    Ok(())
}

fn prepare_primary_metadata_items<'p, V: PrimaryMetadataView<'p>>(
    planner: &mut Planner<'_, '_, 'p>,
    view: &'p V,
    stop: &AtomicBool,
) -> ContextResult<()> {
    for index in 0..view.items_len() {
        checkpoint(stop)?;
        let (template, label) = view.item_template_label(index);
        if template.is_empty() {
            return Err(encoding());
        }
        planner.add(
            view.item_id(index),
            template,
            label,
            &["core.items"],
            Some(index),
            view.item(index),
        )?;
    }
    Ok(())
}

fn prepare_primary_metadata_tail<'p, V: PrimaryMetadataView<'p>>(
    planner: &mut Planner<'_, '_, 'p>,
    id: &'p str,
    view: &'p V,
) -> ContextResult<()> {
    planner.add(
        id,
        "selection_trace",
        "SELECTION_TRACE",
        &["core.selection_trace"],
        None,
        view.trace(),
    )?;
    let coverage = view.coverage();
    planner.add(
        id,
        "coverage_and_omissions",
        "COVERAGE",
        &["core.omissions", "core.state", "core.absence_authoritative"],
        None,
        &coverage,
    )?;
    planner.add(
        id,
        "pack_budget",
        "BUDGET",
        &["core.budget"],
        None,
        view.budget(),
    )?;
    Ok(())
}

fn prepare_primary_metadata<'p, V: PrimaryMetadataView<'p>>(
    planner: &mut Planner<'_, '_, 'p>,
    id: &'p str,
    view: &'p V,
    stop: &AtomicBool,
) -> ContextResult<()> {
    prepare_primary_metadata_head(planner, id, view)?;
    prepare_primary_metadata_items(planner, view, stop)?;
    prepare_primary_metadata_tail(planner, id, view)
}

fn prepare<'p>(
    input: &Input<'p, '_, '_, '_>,
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
        match input.core {
            NativePackCore::Primary(core) => {
                prepare_primary_metadata_head(&mut planner, input.id, core)?;
                if let Some(expansion) = &core.expansion {
                    planner.add(
                        expansion.expansion_id(),
                        "native_frontier_expansion",
                        "EXPANSION",
                        &["core.expansion"],
                        None,
                        expansion,
                    )?;
                }
                prepare_primary_metadata_items(&mut planner, core, stop)?;
                if let Some(source) = &core.source_excerpt {
                    #[derive(Serialize)]
                    struct SourceMetadata<'a> {
                        source_excerpt_id: &'a str,
                        #[serde(flatten)]
                        metadata: &'a crate::pack::source_excerpt::SourceExcerptMetadata,
                    }
                    planner.add(
                        source.source_excerpt_id.as_ref(),
                        "source_excerpt_metadata",
                        "SOURCE_METADATA",
                        &["core.source_excerpt"],
                        None,
                        &SourceMetadata {
                            source_excerpt_id: &source.source_excerpt_id,
                            metadata: &source.core.metadata,
                        },
                    )?;
                    planner.add_payload(
                        source.source_excerpt_id.as_ref(),
                        "source_data_begin",
                        "",
                        &["core.source_excerpt.untrusted_source_data"],
                        None,
                        Payload::CanonicalPack("SOURCE_DATA_BEGIN\n"),
                    )?;
                    for (index, line) in source.core.text.split('\n').enumerate() {
                        planner.add_source_line(source.source_excerpt_id.as_ref(), index, line)?;
                    }
                    planner.add_payload(
                        source.source_excerpt_id.as_ref(),
                        "source_data_end",
                        "",
                        &["generated_boundary"],
                        None,
                        Payload::CanonicalPack("SOURCE_DATA_END\n"),
                    )?;
                }
                prepare_primary_metadata_tail(&mut planner, input.id, core)?;
            }
            NativePackCore::Structural(view) => {
                prepare_primary_metadata(&mut planner, input.id, view, stop)?;
            }
            NativePackCore::Combined(core) => {
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
                    core.origins.inspect_id(),
                    "combined_native_origins",
                    "ORIGINS",
                    &["core.origins"],
                    None,
                    &core.origins,
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
                for (index, item) in core.items.iter().enumerate() {
                    checkpoint(stop)?;
                    let (template, label) = match item.core.payload.template() {
                        "native_entity_item" => ("qualified_native_entity_item", "ENTITY"),
                        "native_relation_item" => ("qualified_native_relation_item", "RELATION"),
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
        }
    }
    Ok(planner.finish())
}

/// Unicode escapes alter only representation; decoding yields the same string.
/// Quotes/backslashes retain JSON escaping. All other ASCII punctuation is
/// Unicode-escaped, including every Markdown/HTML delimiter. Bidi formatting,
/// directional marks, Unicode line separators and controls cannot change lines.
fn emit_record(record: &Record<'_>, output: &mut impl fmt::Write) -> fmt::Result {
    if record.source_line {
        write!(
            output,
            "SRC {:06} \"",
            record.trace.item_index.unwrap_or_default()
        )?;
        for ch in record.payload.text().chars() {
            match ch {
                '"' => output.write_str("\\\"")?,
                '\\' => output.write_str("\\\\")?,
                '\n' => output.write_str("\\n")?,
                '\r' => output.write_str("\\r")?,
                '\t' => output.write_str("\\t")?,
                ch if ch.is_ascii_punctuation()
                    || ch.is_control()
                    || matches!(
                        ch,
                        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{2028}' | '\u{2029}'
                    )
                    || ('\u{202a}'..='\u{202e}').contains(&ch)
                    || ('\u{2066}'..='\u{2069}').contains(&ch) =>
                {
                    write!(output, "\\u{:04x}", u32::from(ch))?
                }
                ch => output.write_char(ch)?,
            }
        }
        output.write_str("\"\n")?;
        return Ok(());
    }
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
    schema: &'static str,
    id: &'p str,
    plan: &'r Plan<'r>,
    profile: &'static RendererProfile,
    stop: &'s AtomicBool,
) -> CoreRef<'p, 'r, 's> {
    CoreRef {
        schema,
        semantic_pack_id: id,
        profile,
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

fn artifact_id(
    schema: &'static str,
    value: &impl Serialize,
    stop: &AtomicBool,
) -> ContextResult<Box<str>> {
    checkpoint(stop)?;
    let hash = wow_core::domain_separated_digest(schema, value);
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
    let (id, core, canonical_bytes, budget) = pack.render_parts();
    render_input(
        Input {
            id,
            core: NativePackCore::Primary(core),
            canonical_bytes,
        },
        format,
        budget,
        limit,
        stop,
    )
}

pub(crate) fn render_validated_primary_l0_pack(
    view: &ValidatedPrimaryL0Pack,
    semantic_pack_id: &str,
    canonical_bytes: &[u8],
    format: PrimaryPackRenderFormat,
    budget: &mut Budget,
    metadata_limit: u64,
    stop: &AtomicBool,
) -> ContextResult<(RenderedContextArtifact, u64)> {
    let format = match format {
        PrimaryPackRenderFormat::Json => Format::Json,
        PrimaryPackRenderFormat::Markdown => Format::Markdown,
    };
    let input = Input {
        id: semantic_pack_id,
        core: NativePackCore::Structural(view),
        canonical_bytes,
    };
    let artifact = render_input(input, format, budget, metadata_limit, stop)?;
    Ok((artifact, budget.work))
}

fn render_input(
    input: Input<'_, '_, '_, '_>,
    format: Format,
    budget: &mut Budget,
    limit: u64,
    stop: &AtomicBool,
) -> ContextResult<RenderedContextArtifact> {
    let id = input.id;
    let schema = input.rendered_schema();
    if let NativePackCore::Primary(core) = input.core {
        crate::pack::source_excerpt::validate_render_ready(core, input.canonical_bytes)?;
    }
    let profile = format.profile(input.core)?;
    let plan = prepare(&input, format, budget, limit, stop)?;
    let core = core_ref(schema, id, &plan, profile, stop);
    pass(&plan, budget, stop)?;
    budget.retain(&core, stop)?;
    pass(&plan, budget, stop)?;
    budget.retain(
        &IdMaterial {
            domain: schema,
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
        schema,
        semantic_pack_id: id,
        profile,
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
    let rendered_context_artifact_id = artifact_id(schema, &complete_core, stop)?;
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
            schema,
            semantic_pack_id: id.into(),
            profile,
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

pub fn render_combined_context_pack_json(
    pack: &mut CombinedContextSemanticPack<'_, '_, '_>,
    stop: &AtomicBool,
) -> ContextResult<RenderedContextArtifact> {
    let limit = pack.render_limit();
    let (id, core, canonical_bytes, budget) = pack.render_parts();
    render_input(
        Input {
            id,
            core: NativePackCore::Combined(core),
            canonical_bytes,
        },
        Format::Json,
        budget,
        limit,
        stop,
    )
}
pub fn render_combined_context_pack_markdown(
    pack: &mut CombinedContextSemanticPack<'_, '_, '_>,
    stop: &AtomicBool,
) -> ContextResult<RenderedContextArtifact> {
    let limit = pack.render_limit();
    let (id, core, canonical_bytes, budget) = pack.render_parts();
    render_input(
        Input {
            id,
            core: NativePackCore::Combined(core),
            canonical_bytes,
        },
        Format::Markdown,
        budget,
        limit,
        stop,
    )
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
    validate_rendered_input(
        Input {
            id,
            core: NativePackCore::Primary(core),
            canonical_bytes,
        },
        artifact,
        budget,
        limit,
        stop,
    )
}

pub fn validate_rendered_combined_context_artifact(
    pack: &mut CombinedContextSemanticPack<'_, '_, '_>,
    artifact: &RenderedContextArtifact,
    stop: &AtomicBool,
) -> ContextResult<()> {
    let limit = pack.render_limit();
    let (id, core, canonical_bytes, budget) = pack.render_parts();
    validate_rendered_input(
        Input {
            id,
            core: NativePackCore::Combined(core),
            canonical_bytes,
        },
        artifact,
        budget,
        limit,
        stop,
    )
}

fn validate_rendered_input(
    input: Input<'_, '_, '_, '_>,
    artifact: &RenderedContextArtifact,
    budget: &mut Budget,
    limit: u64,
    stop: &AtomicBool,
) -> ContextResult<()> {
    let id = input.id;
    let schema = input.rendered_schema();
    budget.work(artifact.core.trace.len(), stop)?;
    if artifact.core.schema != schema
        || artifact.core.semantic_pack_id.as_ref() != id
        || artifact.core.byte_length > limit
        || artifact.core.byte_length
            != u64::try_from(artifact.core.content.len()).map_err(|_| exhausted())?
        || artifact.core.content_digest_basis != DIGEST_BASIS
    {
        return Err(encoding());
    }
    let format = match artifact.core.profile.profile_id {
        crate::pack::COMBINED_CONTEXT_JSON_RENDERER_PROFILE
        | crate::pack::PRIMARY_CONTEXT_JSON_RENDERER_PROFILE
        | crate::pack::PRIMARY_SOURCE_CONTEXT_JSON_RENDERER_PROFILE
        | crate::pack::PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE
        | crate::pack::PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE => Format::Json,
        crate::pack::COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE
        | crate::pack::PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE
        | crate::pack::PRIMARY_SOURCE_CONTEXT_MARKDOWN_RENDERER_PROFILE
        | crate::pack::PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE
        | crate::pack::PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE => Format::Markdown,
        _ => return Err(encoding()),
    };
    // Profiles are exact compiled descriptors, not caller-selectable templates.
    let profile = format.profile(input.core)?;
    budget.work(profile.templates.len(), stop)?;
    budget.retain(&(artifact.core.profile, profile), stop)?;
    if canonical(artifact.core.profile, limit, stop)? != canonical(profile, limit, stop)? {
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
        if artifact.core.content.as_bytes() != input.canonical_bytes {
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
        domain: schema,
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
    if artifact_id(schema, &artifact.core, stop)? != artifact.rendered_context_artifact_id {
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
            source_line: false,
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

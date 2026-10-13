//! Bounded external validation of default Primary L0 metadata semantic packs.
//!
//! Raw admission precedes one closed typed decode and structural identity/reference
//! checks. Decoded records never recreate native owner custody or runtime authority.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{ContextError, ContextErrorCode, ContextResult};
mod assertion_links;
mod declaration_links;
mod declaration_wire;
mod l0_items_wire;
mod owner_records;
mod project_map_wire;
mod support_record_links;
mod wire;

pub(crate) use wire::{
    ContextItemWire, ItemPayloadWire, OmissionsWire, ProjectMapStateWire, SelectionRecordWire,
    UniverseBindingWire,
};
pub(crate) use wire::{
    PackBudgetWire, PackProfileWire, RequestWire, SkeletonsWire, ValidatedPrimaryL0Pack,
};

const PACK_SCHEMA: &str = "wow-context/semantic-pack/primary/1";
const PACK_PROFILE: &str = "wow-context/primary-structural-pack/1";
const MAP_SCHEMA: &str = "wow-context/project-map/primary-direct-navigation/1";
const MAP_PROFILE: &str = "wow-context/primary-direct-navigation/1";
const L0_SCHEMA: &str = "wow-context/l0-skeleton/primary/1";
const L0_PROFILE: &str = "wow-context/primary-l0/1";
const MAX_JSON_DEPTH: u32 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimaryPackValidationLimits {
    max_input_bytes: u64,
    max_metadata_bytes: u64,
    max_context_work: u64,
    max_depth: u32,
    max_tokens: u64,
}

impl PrimaryPackValidationLimits {
    pub fn new(
        max_input_bytes: u64,
        max_metadata_bytes: u64,
        max_context_work: u64,
        max_depth: u32,
        max_tokens: u64,
    ) -> ContextResult<Self> {
        if [
            max_input_bytes,
            max_metadata_bytes,
            max_context_work,
            max_tokens,
        ]
        .into_iter()
        .any(|value| value == 0 || usize::try_from(value).is_err())
            || max_depth == 0
            || max_depth > MAX_JSON_DEPTH
        {
            return Err(ContextError::new(ContextErrorCode::InvalidBudget));
        }
        Ok(Self {
            max_input_bytes,
            max_metadata_bytes,
            max_context_work,
            max_depth,
            max_tokens,
        })
    }

    pub const fn max_input_bytes(self) -> u64 {
        self.max_input_bytes
    }
    pub const fn max_metadata_bytes(self) -> u64 {
        self.max_metadata_bytes
    }
    pub const fn max_context_work(self) -> u64 {
        self.max_context_work
    }
    pub const fn max_depth(self) -> u32 {
        self.max_depth
    }
    pub const fn max_tokens(self) -> u64 {
        self.max_tokens
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryPackValidationState {
    Valid,
    Invalid,
    NotEvaluated,
}

/// Shared state spelling used by the private Service and CLI adapters.
pub type ValidationState = PrimaryPackValidationState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryPackValidationCheck {
    ByteAdmission,
    CanonicalBytes,
    Schema,
    CompiledProfile,
    DigestAndIdentity,
    InternalReferences,
    OwnerGenerationClosure,
    SourceOrRuntimeAuthority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryPackValidationCode {
    InputNotCanonicalJson,
    DuplicateObjectKey,
    NullOrFloatForbidden,
    NonCanonicalInteger,
    WireSchemaMismatch,
    CompiledProfileMismatch,
    DigestIdentityMismatch,
    InternalReferenceMismatch,
    UnsupportedDeclarationFamily,
    UnsupportedArtifactSchema,
    UnsupportedLevel,
    UnsupportedRoot,
    UnsupportedProfile,
    UnsupportedExpansion,
    UnsupportedSourceExcerpt,
    UnsupportedMapProfile,
    UnsupportedRootKind,
    MissingRequiredSelector,
    SelectorTypeInvalid,
    RecursiveWireClosurePending,
    SemanticPackIdentityMismatch,
    UnsupportedLoadCustody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryPackLevel {
    L0,
    L1,
    Both,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryPackRootVariant {
    Entity,
    Project,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryPackL0Profile {
    Omitted,
    ContainersV1,
    StateRootV1,
    ReferenceRootV1,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryPackOriginClosure {
    StructuralOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PrimaryPackSelection {
    level: PrimaryPackLevel,
    root: PrimaryPackRootVariant,
    l0_profile: PrimaryPackL0Profile,
    expansion_present: bool,
    source_excerpt_present: bool,
    unsupported_load_custody: bool,
}

impl PrimaryPackSelection {
    pub const fn level(&self) -> PrimaryPackLevel {
        self.level
    }
    pub const fn root(&self) -> PrimaryPackRootVariant {
        self.root
    }
    pub const fn l0_profile(&self) -> PrimaryPackL0Profile {
        self.l0_profile
    }
    pub const fn expansion_present(&self) -> bool {
        self.expansion_present
    }
    pub const fn source_excerpt_present(&self) -> bool {
        self.source_excerpt_present
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PrimaryPackValidationRecord {
    check: PrimaryPackValidationCheck,
    state: PrimaryPackValidationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<PrimaryPackValidationCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PrimaryPackValidationBudgetUsage {
    input_bytes: u64,
    scanner_work: u64,
    scanner_tokens: u64,
    max_input_bytes: u64,
    max_metadata_bytes: u64,
    max_context_work: u64,
    max_depth: u32,
    max_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrimaryPackValidation {
    state: PrimaryPackValidationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact_schema: Option<Box<str>>,
    artifact_digest: Box<str>,
    byte_length: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_pack_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection: Option<PrimaryPackSelection>,
    origin_closure: PrimaryPackOriginClosure,
    canonical_bytes: PrimaryPackValidationState,
    schema: PrimaryPackValidationState,
    compiled_profile: PrimaryPackValidationState,
    digest_and_identity: PrimaryPackValidationState,
    internal_references: PrimaryPackValidationState,
    owner_generation_closure: PrimaryPackValidationState,
    source_or_runtime_authority: PrimaryPackValidationState,
    records: Box<[PrimaryPackValidationRecord]>,
    errors: Box<[PrimaryPackValidationCode]>,
    warnings: Box<[PrimaryPackValidationCode]>,
    coverage: Box<[PrimaryPackValidationCode]>,
    omissions: Box<[PrimaryPackValidationCode]>,
    budget_usage: PrimaryPackValidationBudgetUsage,
    nonrepairing: bool,
}

impl PrimaryPackValidation {
    pub const fn state(&self) -> PrimaryPackValidationState {
        self.state
    }
    pub const fn overall(&self) -> PrimaryPackValidationState {
        self.state
    }
    pub const fn overall_state(&self) -> ValidationState {
        self.state
    }
    pub const fn canonical_bytes(&self) -> PrimaryPackValidationState {
        self.canonical_bytes
    }
    pub const fn schema(&self) -> PrimaryPackValidationState {
        self.schema
    }
    pub const fn compiled_profile(&self) -> PrimaryPackValidationState {
        self.compiled_profile
    }
    pub const fn digest_and_identity(&self) -> PrimaryPackValidationState {
        self.digest_and_identity
    }
    pub const fn internal_references(&self) -> PrimaryPackValidationState {
        self.internal_references
    }
    pub const fn owner_generation_closure(&self) -> PrimaryPackValidationState {
        self.owner_generation_closure
    }
    pub const fn source_or_runtime_authority(&self) -> PrimaryPackValidationState {
        self.source_or_runtime_authority
    }
    pub fn artifact_schema(&self) -> Option<&str> {
        self.artifact_schema.as_deref()
    }
    pub fn artifact_digest(&self) -> &str {
        &self.artifact_digest
    }
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }
    pub fn semantic_pack_id(&self) -> Option<&str> {
        self.semantic_pack_id.as_deref()
    }
    pub fn profile_id(&self) -> Option<&str> {
        self.profile_id.as_deref()
    }
    pub const fn selection(&self) -> Option<&PrimaryPackSelection> {
        self.selection.as_ref()
    }
    pub const fn origin_closure(&self) -> PrimaryPackOriginClosure {
        self.origin_closure
    }
    pub fn records(&self) -> &[PrimaryPackValidationRecord] {
        &self.records
    }
    pub fn errors(&self) -> &[PrimaryPackValidationCode] {
        &self.errors
    }
    pub fn warnings(&self) -> &[PrimaryPackValidationCode] {
        &self.warnings
    }
    pub fn coverage(&self) -> &[PrimaryPackValidationCode] {
        &self.coverage
    }
    pub fn omissions(&self) -> &[PrimaryPackValidationCode] {
        &self.omissions
    }
    pub const fn budget_usage(&self) -> &PrimaryPackValidationBudgetUsage {
        &self.budget_usage
    }
    pub const fn nonrepairing(&self) -> bool {
        self.nonrepairing
    }
}

impl PrimaryPackValidationRecord {
    pub const fn check(&self) -> PrimaryPackValidationCheck {
        self.check
    }
    pub const fn state(&self) -> PrimaryPackValidationState {
        self.state
    }
    pub const fn code(&self) -> Option<PrimaryPackValidationCode> {
        self.code
    }
}

impl PrimaryPackValidationBudgetUsage {
    pub const fn input_bytes(&self) -> u64 {
        self.input_bytes
    }
    pub const fn scanner_work(&self) -> u64 {
        self.scanner_work
    }
    pub const fn scanner_tokens(&self) -> u64 {
        self.scanner_tokens
    }
    pub const fn max_input_bytes(&self) -> u64 {
        self.max_input_bytes
    }
    pub const fn max_metadata_bytes(&self) -> u64 {
        self.max_metadata_bytes
    }
    pub const fn max_context_work(&self) -> u64 {
        self.max_context_work
    }
    pub const fn max_depth(&self) -> u32 {
        self.max_depth
    }
    pub const fn max_tokens(&self) -> u64 {
        self.max_tokens
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryPackRenderFormat {
    Json,
    Markdown,
}

/// Validation report and optional artifact from one admitted external Primary L0 pack.
#[derive(Serialize)]
pub struct PrimaryPackRenderResult {
    validation: PrimaryPackValidation,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact: Option<crate::RenderedContextArtifact>,
    total_work: u64,
}
impl PrimaryPackRenderResult {
    pub const fn validation(&self) -> &PrimaryPackValidation {
        &self.validation
    }
    pub fn artifact(&self) -> Option<&crate::RenderedContextArtifact> {
        self.artifact.as_ref()
    }
    pub const fn total_work(&self) -> u64 {
        self.total_work
    }
}

#[derive(Default)]
struct HeaderProbe {
    pack_schema: Option<String>,
    semantic_pack_id: Option<String>,
    profile_id: Option<String>,
    request_profile_id: Option<String>,
    level: Option<String>,
    root_kind: Option<String>,
    skeleton_level: Option<String>,
    l0_schema: Option<String>,
    l0_profile: Option<String>,
    map_schema: Option<String>,
    map_profile: Option<String>,
    scope_node_kind: Option<String>,
    map_root_kind: Option<String>,
    l0_profile_present: bool,
    l1_profile_present: bool,
    expansion_present: bool,
    source_excerpt_present: bool,
    selector_type_invalid: bool,
    unsupported_load_custody: bool,
}

impl HeaderProbe {
    fn is_string_target(path: &[String]) -> bool {
        matches_path(path, &["core", "schema"])
            || matches_path(path, &["semantic_pack_id"])
            || matches_path(path, &["core", "profile", "profile_id"])
            || matches_path(path, &["core", "request", "core", "profile_id"])
            || matches_path(path, &["core", "request", "core", "level"])
            || matches_path(path, &["core", "request", "core", "root", "kind"])
            || matches_path(path, &["core", "skeletons", "level"])
            || matches_path(path, &["core", "skeletons", "l0", "core", "schema"])
            || matches_path(path, &["core", "skeletons", "l0", "core", "profile"])
            || matches_path(
                path,
                &[
                    "core",
                    "skeletons",
                    "l0",
                    "core",
                    "profile_descriptor",
                    "profile_id",
                ],
            )
            || matches_path(
                path,
                &["core", "skeletons", "l0", "core", "scope", "node", "kind"],
            )
            || matches_path(
                path,
                &[
                    "core",
                    "skeletons",
                    "l0",
                    "core",
                    "context",
                    "origins",
                    "core",
                    "schema",
                ],
            )
            || matches_path(
                path,
                &[
                    "core",
                    "skeletons",
                    "l0",
                    "core",
                    "context",
                    "origins",
                    "core",
                    "profile",
                    "profile",
                ],
            )
            || matches_path(
                path,
                &[
                    "core",
                    "skeletons",
                    "l0",
                    "core",
                    "context",
                    "origins",
                    "core",
                    "navigation",
                    "schema",
                ],
            )
            || matches_path(
                path,
                &[
                    "core",
                    "skeletons",
                    "l0",
                    "core",
                    "context",
                    "origins",
                    "core",
                    "navigation",
                    "root",
                    "kind",
                ],
            )
    }

    fn capture(&mut self, path: &[String], value: String) {
        let slot = if matches_path(path, &["semantic_pack_id"]) {
            &mut self.semantic_pack_id
        } else if matches_path(path, &["core", "schema"]) {
            &mut self.pack_schema
        } else if matches_path(path, &["core", "profile", "profile_id"]) {
            &mut self.profile_id
        } else if matches_path(path, &["core", "request", "core", "profile_id"]) {
            &mut self.request_profile_id
        } else if matches_path(path, &["core", "request", "core", "level"]) {
            &mut self.level
        } else if matches_path(path, &["core", "request", "core", "root", "kind"]) {
            &mut self.root_kind
        } else if matches_path(path, &["core", "skeletons", "level"]) {
            &mut self.skeleton_level
        } else if matches_path(path, &["core", "skeletons", "l0", "core", "schema"]) {
            &mut self.l0_schema
        } else if matches_path(path, &["core", "skeletons", "l0", "core", "profile"])
            || matches_path(
                path,
                &[
                    "core",
                    "skeletons",
                    "l0",
                    "core",
                    "profile_descriptor",
                    "profile_id",
                ],
            )
        {
            &mut self.l0_profile
        } else if matches_path(
            path,
            &["core", "skeletons", "l0", "core", "scope", "node", "kind"],
        ) {
            &mut self.scope_node_kind
        } else if matches_path(
            path,
            &[
                "core",
                "skeletons",
                "l0",
                "core",
                "context",
                "origins",
                "core",
                "schema",
            ],
        ) {
            &mut self.map_schema
        } else if matches_path(
            path,
            &[
                "core",
                "skeletons",
                "l0",
                "core",
                "context",
                "origins",
                "core",
                "profile",
                "profile",
            ],
        ) {
            &mut self.map_profile
        } else if matches_path(
            path,
            &[
                "core",
                "skeletons",
                "l0",
                "core",
                "context",
                "origins",
                "core",
                "navigation",
                "schema",
            ],
        ) {
            &mut self.map_schema
        } else if matches_path(
            path,
            &[
                "core",
                "skeletons",
                "l0",
                "core",
                "context",
                "origins",
                "core",
                "navigation",
                "root",
                "kind",
            ],
        ) {
            &mut self.map_root_kind
        } else {
            return;
        };
        *slot = Some(value);
    }

    fn note_key(&mut self, path: &[String]) {
        self.l0_profile_present |=
            matches_path(path, &["core", "request", "core", "l0_profile_id"]);
        self.l1_profile_present |=
            matches_path(path, &["core", "request", "core", "l1_profile_id"]);
        self.expansion_present |= matches_path(path, &["core", "expansion"])
            || matches_path(path, &["core", "request", "core", "expansion_id"]);
        self.source_excerpt_present |= matches_path(path, &["core", "source_excerpt"])
            || matches_path(path, &["core", "request", "core", "source_excerpt"]);
        self.unsupported_load_custody |= matches_path(
            path,
            &[
                "core",
                "skeletons",
                "l0",
                "core",
                "context",
                "origins",
                "core",
                "load_custody",
                "selected_toc",
            ],
        ) || matches_path(
            path,
            &[
                "core",
                "skeletons",
                "l0",
                "core",
                "context",
                "origins",
                "core",
                "load_custody",
                "package_load",
            ],
        ) || matches_path(
            path,
            &[
                "core",
                "skeletons",
                "l0",
                "core",
                "context",
                "origins",
                "core",
                "load_custody",
                "package_main",
            ],
        );
    }
}

fn matches_path(actual: &[String], expected: &[&str]) -> bool {
    actual.len() == expected.len() && actual.iter().zip(expected).all(|(a, b)| a == b)
}

struct Scanner<'a> {
    input: &'a [u8],
    offset: usize,
    limits: PrimaryPackValidationLimits,
    stop: &'a AtomicBool,
    work: u64,
    tokens: u64,
    header: HeaderProbe,
    pack_core_span: Option<(usize, usize)>,
    duplicate_key: bool,
    noncanonical: bool,
    forbidden_number: bool,
    noncanonical_integer: bool,
}

/// Results supplied by the one composed typed wire decoder after raw admission.
/// The driver owns aggregation/reporting; it does not infer these checks from
/// header markers or substitute a success value when the decoder is absent.
pub(crate) struct PrimaryPackDecoderCheck {
    artifact_schema: Option<Box<str>>,
    profile_id: Option<Box<str>>,
    selection: Option<PrimaryPackSelection>,
    schema: PrimaryPackValidationState,
    compiled_profile: PrimaryPackValidationState,
    digest_and_identity: PrimaryPackValidationState,
    internal_references: PrimaryPackValidationState,
    semantic_pack_id: Option<Box<str>>,
    code: Option<PrimaryPackValidationCode>,
}

/// Raw admission is deliberately completed before the composed decoder is
/// allowed to allocate its closed wire records. `decoded_input` is available
/// only after byte, cancellation, syntax, canonical-order, duplicate-key,
/// integer-form, depth, token, and work admission succeeds.
pub(crate) struct PrimaryPackAdmission<'a> {
    scanner: Scanner<'a>,
    digest: String,
    input_len: usize,
    limits: PrimaryPackValidationLimits,
    syntax_error: Option<PrimaryPackValidationCode>,
    decoder_eligible: bool,
    decoded_view_bytes: u64,
}

impl<'a> PrimaryPackAdmission<'a> {
    pub(crate) fn decoded_input(&self) -> Option<&'a [u8]> {
        (self.syntax_error.is_none() && self.decoder_eligible).then_some(self.scanner.input)
    }

    /// Canonical-size metadata reserved for a decoder-eligible retained wire view.
    pub(crate) const fn decoded_view_bytes(&self) -> u64 {
        self.decoded_view_bytes
    }

    /// Return the raw selected route only after its finite preflight and raw
    /// PackCore identity have both passed.
    pub(crate) fn selection(&self) -> Option<PrimaryPackSelection> {
        self.decoder_eligible
            .then(|| selection(&self.scanner.header))
            .flatten()
    }

    /// Charge decoder traversal to the same cumulative work allowance already
    /// consumed by raw hashing and canonical scanning.
    pub(crate) fn charge_work(&mut self, amount: usize) -> ContextResult<()> {
        let amount = u64::try_from(amount).map_err(|_| invalid_budget())?;
        self.scanner.charge(amount)
    }

    pub(crate) fn checkpoint(&self) -> ContextResult<()> {
        if self.scanner.stop.load(Ordering::Acquire) {
            Err(ContextError::new(ContextErrorCode::Cancelled))
        } else {
            Ok(())
        }
    }

    pub(crate) const fn limits(&self) -> PrimaryPackValidationLimits {
        self.limits
    }

    /// Finalize from the exact typed checker result produced by the composed
    /// owner decoder. Unsupported preflight scopes remain NotEvaluated even if
    /// a caller supplies unrelated check states.
    pub(crate) fn finalize(self, checked: PrimaryPackDecoderCheck) -> PrimaryPackValidation {
        if let Some(code) = self.syntax_error {
            return report(
                ReportInput {
                    state: PrimaryPackValidationState::Invalid,
                    schema: None,
                    digest: self.digest,
                    byte_length: self.input_len,
                    semantic_pack_id: None,
                    profile_id: None,
                    selection: None,
                    canonical_bytes: PrimaryPackValidationState::Invalid,
                    schema_state: PrimaryPackValidationState::NotEvaluated,
                    compiled_profile: PrimaryPackValidationState::NotEvaluated,
                    digest_and_identity: PrimaryPackValidationState::NotEvaluated,
                    internal_references: PrimaryPackValidationState::NotEvaluated,
                    code: Some(code),
                    schema_code: None,
                    profile_code: None,
                    digest_code: None,
                    references_code: None,
                },
                &self.scanner,
                self.limits,
            );
        }

        let selection = selection(&self.scanner.header);
        let (preflight_state, preflight_schema, _profile, preflight_code) =
            classify(&self.scanner.header);
        if preflight_state != PrimaryPackValidationState::NotEvaluated
            || preflight_code != PrimaryPackValidationCode::RecursiveWireClosurePending
        {
            return report(
                ReportInput {
                    state: preflight_state,
                    schema: (self.scanner.header.pack_schema.as_deref() == Some(PACK_SCHEMA))
                        .then_some(PACK_SCHEMA),
                    digest: self.digest,
                    byte_length: self.input_len,
                    semantic_pack_id: None,
                    profile_id: None,
                    selection,
                    canonical_bytes: PrimaryPackValidationState::Valid,
                    schema_state: preflight_schema,
                    compiled_profile: _profile,
                    digest_and_identity: PrimaryPackValidationState::NotEvaluated,
                    internal_references: PrimaryPackValidationState::NotEvaluated,
                    code: Some(preflight_code),
                    schema_code: None,
                    profile_code: None,
                    digest_code: None,
                    references_code: None,
                },
                &self.scanner,
                self.limits,
            );
        }

        // The raw exact-span check is an independent early guard. The typed
        // result still has to report its complete pack/item/map/L0 digest and
        // reference checks before the overall state can become Valid.
        if !self.decoder_eligible {
            return report(
                ReportInput {
                    state: PrimaryPackValidationState::Invalid,
                    schema: Some(PACK_SCHEMA),
                    digest: self.digest,
                    byte_length: self.input_len,
                    semantic_pack_id: None,
                    profile_id: None,
                    selection,
                    canonical_bytes: PrimaryPackValidationState::Valid,
                    schema_state: PrimaryPackValidationState::Valid,
                    compiled_profile: PrimaryPackValidationState::NotEvaluated,
                    digest_and_identity: PrimaryPackValidationState::Invalid,
                    internal_references: PrimaryPackValidationState::NotEvaluated,
                    code: Some(PrimaryPackValidationCode::SemanticPackIdentityMismatch),
                    schema_code: None,
                    profile_code: None,
                    digest_code: Some(PrimaryPackValidationCode::DigestIdentityMismatch),
                    references_code: None,
                },
                &self.scanner,
                self.limits,
            );
        }

        let typed_schema_matches = checked.artifact_schema.as_deref() == Some(PACK_SCHEMA);
        let artifact_schema = checked
            .artifact_schema
            .filter(|schema| schema.as_ref() == PACK_SCHEMA)
            .or_else(|| {
                (checked.schema != PrimaryPackValidationState::Valid)
                    .then(|| {
                        (self.scanner.header.pack_schema.as_deref() == Some(PACK_SCHEMA))
                            .then(|| Box::<str>::from(PACK_SCHEMA))
                    })
                    .flatten()
            });
        let typed_profile_matches = checked.profile_id.as_deref() == Some(PACK_PROFILE);
        let profile_id = checked
            .profile_id
            .filter(|profile| profile.as_ref() == PACK_PROFILE);
        let typed_pack_id_matches = checked
            .semantic_pack_id
            .as_deref()
            .is_some_and(|id| self.scanner.header.semantic_pack_id.as_deref() == Some(id));
        let semantic_pack_id = self
            .scanner
            .header
            .semantic_pack_id
            .as_deref()
            .filter(|raw_id| {
                checked
                    .semantic_pack_id
                    .as_deref()
                    .is_none_or(|checked_id| checked_id == *raw_id)
            })
            .map(Box::<str>::from);
        let typed_selection_present = checked.selection.is_some();
        let typed_selection_matches = checked
            .selection
            .as_ref()
            .is_some_and(|typed| selection.as_ref() == Some(typed));
        let selected = checked
            .selection
            .filter(|typed| selection.as_ref() == Some(typed))
            .or(selection);
        let schema_state = if checked.schema == PrimaryPackValidationState::Valid
            && (!typed_schema_matches || artifact_schema.is_none())
        {
            PrimaryPackValidationState::Invalid
        } else {
            checked.schema
        };
        let compiled_profile_state = if checked.compiled_profile
            == PrimaryPackValidationState::Valid
            && (!typed_profile_matches
                || profile_id.is_none()
                || !typed_selection_present
                || !typed_selection_matches)
        {
            PrimaryPackValidationState::Invalid
        } else {
            checked.compiled_profile
        };
        let digest_state = if checked.digest_and_identity == PrimaryPackValidationState::Valid
            && (!typed_pack_id_matches || semantic_pack_id.is_none())
        {
            PrimaryPackValidationState::Invalid
        } else {
            checked.digest_and_identity
        };
        let required = [
            schema_state,
            compiled_profile_state,
            digest_state,
            checked.internal_references,
        ];
        let state = aggregate_required_checks(&required);
        let schema_code = if schema_state == PrimaryPackValidationState::Invalid {
            Some(if !typed_schema_matches {
                PrimaryPackValidationCode::WireSchemaMismatch
            } else {
                checked
                    .code
                    .unwrap_or(PrimaryPackValidationCode::WireSchemaMismatch)
            })
        } else {
            None
        };
        let profile_code = if compiled_profile_state == PrimaryPackValidationState::Invalid {
            Some(
                if !typed_profile_matches || !typed_selection_present || !typed_selection_matches {
                    PrimaryPackValidationCode::CompiledProfileMismatch
                } else {
                    checked
                        .code
                        .unwrap_or(PrimaryPackValidationCode::CompiledProfileMismatch)
                },
            )
        } else {
            None
        };
        let digest_code = if digest_state == PrimaryPackValidationState::Invalid {
            Some(if !typed_pack_id_matches || semantic_pack_id.is_none() {
                PrimaryPackValidationCode::DigestIdentityMismatch
            } else {
                checked
                    .code
                    .unwrap_or(PrimaryPackValidationCode::DigestIdentityMismatch)
            })
        } else {
            None
        };
        let references_code = if checked.internal_references == PrimaryPackValidationState::Invalid
        {
            Some(
                checked
                    .code
                    .unwrap_or(PrimaryPackValidationCode::InternalReferenceMismatch),
            )
        } else {
            None
        };
        let code = checked
            .code
            .filter(|_| state != PrimaryPackValidationState::Valid);
        let complete_identity =
            digest_state == PrimaryPackValidationState::Valid && semantic_pack_id.is_some();
        report(
            ReportInput {
                state,
                schema: artifact_schema.as_deref(),
                digest: self.digest,
                byte_length: self.input_len,
                semantic_pack_id: complete_identity
                    .then_some(semantic_pack_id.as_deref())
                    .flatten(),
                profile_id: profile_id.as_deref(),
                selection: selected,
                canonical_bytes: PrimaryPackValidationState::Valid,
                schema_state,
                compiled_profile: compiled_profile_state,
                digest_and_identity: digest_state,
                internal_references: checked.internal_references,
                code,
                schema_code,
                profile_code,
                digest_code,
                references_code,
            },
            &self.scanner,
            self.limits,
        )
    }
}

fn aggregate_required_checks(states: &[PrimaryPackValidationState]) -> PrimaryPackValidationState {
    if states.contains(&PrimaryPackValidationState::Invalid) {
        PrimaryPackValidationState::Invalid
    } else if states.contains(&PrimaryPackValidationState::NotEvaluated) {
        PrimaryPackValidationState::NotEvaluated
    } else {
        PrimaryPackValidationState::Valid
    }
}

impl<'a> Scanner<'a> {
    fn new(input: &'a [u8], limits: PrimaryPackValidationLimits, stop: &'a AtomicBool) -> Self {
        Self {
            input,
            offset: 0,
            limits,
            stop,
            work: 0,
            tokens: 0,
            header: HeaderProbe::default(),
            pack_core_span: None,
            duplicate_key: false,
            noncanonical: false,
            forbidden_number: false,
            noncanonical_integer: false,
        }
    }

    fn charge(&mut self, amount: u64) -> ContextResult<()> {
        if self.stop.load(Ordering::Acquire) {
            return Err(ContextError::new(ContextErrorCode::Cancelled));
        }
        self.work = self
            .work
            .checked_add(amount)
            .filter(|value| *value <= self.limits.max_context_work)
            .ok_or_else(|| {
                ContextError::new(ContextErrorCode::MinimumRequiredContentExceedsBudget)
            })?;
        Ok(())
    }

    fn token(&mut self) -> ContextResult<()> {
        self.charge(1)?;
        self.tokens = self
            .tokens
            .checked_add(1)
            .filter(|value| *value <= self.limits.max_tokens)
            .ok_or_else(|| {
                ContextError::new(ContextErrorCode::MinimumRequiredContentExceedsBudget)
            })?;
        Ok(())
    }

    fn scan(&mut self) -> ContextResult<bool> {
        let mut path = Vec::new();
        match self.value(0, &mut path) {
            Ok(()) => {}
            Err(error) if error.code() == ContextErrorCode::EncodingInvalid => return Ok(false),
            Err(error) => return Err(error),
        }
        Ok(self.offset == self.input.len())
    }

    fn value(&mut self, depth: u32, path: &mut Vec<String>) -> ContextResult<()> {
        self.token()?;
        let start = self.offset;
        let byte = *self.input.get(self.offset).ok_or_else(invalid_json)?;
        if HeaderProbe::is_string_target(path) && byte != b'"' {
            self.header.selector_type_invalid = true;
        }
        let result = match byte {
            b'{' => self.object(depth + 1, path),
            b'[' => self.array(depth + 1, path),
            b'"' => {
                let value = self.string()?;
                if HeaderProbe::is_string_target(path) {
                    self.header.capture(path, value);
                }
                Ok(())
            }
            b't' => self.literal(b"true"),
            b'f' => self.literal(b"false"),
            b'n' => {
                self.forbidden_number = true;
                Err(invalid_json())
            }
            b'-' | b'0'..=b'9' => self.integer(),
            _ => Err(invalid_json()),
        };
        result?;
        if matches_path(path, &["core"]) {
            self.pack_core_span = Some((start, self.offset));
        }
        Ok(())
    }

    fn object(&mut self, depth: u32, path: &mut Vec<String>) -> ContextResult<()> {
        self.check_depth(depth)?;
        self.offset += 1;
        let mut keys = HashSet::<String>::new();
        let mut previous: Option<String> = None;
        if self.input.get(self.offset) == Some(&b'}') {
            self.offset += 1;
            return Ok(());
        }
        loop {
            if self.input.get(self.offset) != Some(&b'"') {
                return Err(invalid_json());
            }
            self.token()?;
            let key = self.string()?;
            if key.is_empty() || key.bytes().any(|byte| byte.is_ascii_control()) {
                self.noncanonical = true;
                return Err(invalid_json());
            }
            if !keys.insert(key.clone()) {
                self.duplicate_key = true;
                return Err(invalid_json());
            }
            if previous
                .as_ref()
                .is_some_and(|prior| prior.as_bytes() >= key.as_bytes())
            {
                self.noncanonical = true;
                return Err(invalid_json());
            }
            previous = Some(key.clone());
            path.push(key);
            self.header.note_key(path);
            if self.input.get(self.offset) != Some(&b':') {
                return Err(invalid_json());
            }
            self.offset += 1;
            self.value(depth, path)?;
            path.pop();
            match self.input.get(self.offset) {
                Some(b',') => self.offset += 1,
                Some(b'}') => {
                    self.offset += 1;
                    break;
                }
                _ => return Err(invalid_json()),
            }
        }
        Ok(())
    }

    fn array(&mut self, depth: u32, path: &mut Vec<String>) -> ContextResult<()> {
        self.check_depth(depth)?;
        self.offset += 1;
        if self.input.get(self.offset) == Some(&b']') {
            self.offset += 1;
            return Ok(());
        }
        let mut index = 0u64;
        loop {
            path.push(format!("[{index}]"));
            self.value(depth, path)?;
            path.pop();
            index = index.checked_add(1).ok_or_else(|| {
                ContextError::new(ContextErrorCode::MinimumRequiredContentExceedsBudget)
            })?;
            match self.input.get(self.offset) {
                Some(b',') => self.offset += 1,
                Some(b']') => {
                    self.offset += 1;
                    break;
                }
                _ => return Err(invalid_json()),
            }
        }
        Ok(())
    }

    fn check_depth(&self, depth: u32) -> ContextResult<()> {
        if depth > self.limits.max_depth {
            Err(ContextError::new(
                ContextErrorCode::MinimumRequiredContentExceedsBudget,
            ))
        } else {
            Ok(())
        }
    }

    fn string(&mut self) -> ContextResult<String> {
        let start = self.offset;
        self.offset += 1;
        let mut closed = false;
        while let Some(byte) = self.input.get(self.offset).copied() {
            self.charge(1)?;
            match byte {
                b'"' => {
                    self.offset += 1;
                    closed = true;
                    break;
                }
                b'\\' => {
                    self.offset += 1;
                    self.charge(1)?;
                    if self.offset >= self.input.len() {
                        return Err(invalid_json());
                    }
                    self.offset += 1;
                }
                0x00..=0x1f => return Err(invalid_json()),
                _ => self.offset += 1,
            }
        }
        if !closed {
            return Err(invalid_json());
        }
        let raw = &self.input[start..self.offset];
        let decoded: String = serde_json::from_slice(raw).map_err(|_| invalid_json())?;
        let canonical = serde_json::to_vec(&decoded).map_err(|_| invalid_json())?;
        if canonical.as_slice() != raw {
            self.noncanonical = true;
            return Err(invalid_json());
        }
        Ok(decoded)
    }

    fn literal(&mut self, literal: &[u8]) -> ContextResult<()> {
        let end = self
            .offset
            .checked_add(literal.len())
            .ok_or_else(invalid_json)?;
        if self.input.get(self.offset..end) != Some(literal) {
            return Err(invalid_json());
        }
        self.offset = end;
        Ok(())
    }

    fn integer(&mut self) -> ContextResult<()> {
        let start = self.offset;
        if self.input[self.offset] == b'-' {
            self.offset += 1;
        }
        if self.input.get(self.offset) == Some(&b'0') {
            self.offset += 1;
            if self.input.get(self.offset).is_some_and(u8::is_ascii_digit) {
                return Err(invalid_json());
            }
        } else {
            if !self.input.get(self.offset).is_some_and(u8::is_ascii_digit) {
                return Err(invalid_json());
            }
            while self.input.get(self.offset).is_some_and(u8::is_ascii_digit) {
                self.charge(1)?;
                self.offset += 1;
            }
        }
        let raw =
            std::str::from_utf8(&self.input[start..self.offset]).map_err(|_| invalid_json())?;
        if raw.starts_with('-') || raw.parse::<u64>().is_err() {
            self.noncanonical_integer = true;
            return Err(invalid_json());
        }
        if self
            .input
            .get(self.offset)
            .is_some_and(|b| matches!(b, b'.' | b'e' | b'E'))
        {
            self.forbidden_number = true;
            return Err(invalid_json());
        }
        Ok(())
    }
}

fn invalid_json() -> ContextError {
    ContextError::new(ContextErrorCode::EncodingInvalid)
}

pub fn validate_primary_l0_pack(
    input: &[u8],
    limits: PrimaryPackValidationLimits,
    stop: &AtomicBool,
) -> ContextResult<PrimaryPackValidation> {
    let mut admission = admit_primary_l0_pack(input, limits, stop)?;
    let checked = wire::check(&mut admission, stop)?;
    admission.checkpoint()?;
    Ok(admission.finalize(checked.check))
}

/// Validate and render one admitted Primary Structural L0 pack using the exact
/// decoded view retained by the validator's single wire decode. No native pack
/// or owner custody is constructed. Unsupported and invalid inputs return the
/// validation report without a rendered artifact.
pub fn validate_and_render_primary_l0_pack(
    input: &[u8],
    limits: PrimaryPackValidationLimits,
    format: PrimaryPackRenderFormat,
    stop: &AtomicBool,
) -> ContextResult<PrimaryPackRenderResult> {
    let mut admission = admit_primary_l0_pack(input, limits, stop)?;
    let prior_metadata = admission.decoded_view_bytes();
    let checked = wire::check(&mut admission, stop)?;
    admission.checkpoint()?;
    let validated = checked.validated;
    let report = admission.finalize(checked.check);
    if report.state() != PrimaryPackValidationState::Valid {
        let total_work = report.budget_usage.scanner_work;
        return Ok(PrimaryPackRenderResult {
            validation: report,
            artifact: None,
            total_work,
        });
    }
    let Some(validated) = validated else {
        return Err(ContextError::new(ContextErrorCode::NativeOwnerRefused));
    };
    let Some(semantic_pack_id) = report.semantic_pack_id() else {
        return Err(ContextError::new(ContextErrorCode::NativeOwnerRefused));
    };
    let prior_work = report.budget_usage.scanner_work;
    let mut budget = crate::budget::Budget::continue_external(
        limits.max_metadata_bytes(),
        limits.max_context_work(),
        prior_metadata,
        prior_work,
    )?;
    let (artifact, total_work) = crate::render::render_validated_primary_l0_pack(
        &validated,
        semantic_pack_id,
        input,
        format,
        &mut budget,
        limits.max_metadata_bytes(),
        stop,
    )?;
    Ok(PrimaryPackRenderResult {
        validation: report,
        artifact: Some(artifact),
        total_work,
    })
}

/// Perform bounded raw admission before the single typed pack decoder runs.
/// The returned input accessor is present only for the exact finite supported
/// Primary L0 candidate whose PackCore identity subspan already matches.
pub(crate) fn admit_primary_l0_pack<'a>(
    input: &'a [u8],
    limits: PrimaryPackValidationLimits,
    stop: &'a AtomicBool,
) -> ContextResult<PrimaryPackAdmission<'a>> {
    let input_bytes = u64::try_from(input.len()).map_err(|_| invalid_budget())?;
    if input_bytes > limits.max_input_bytes || input_bytes > limits.max_metadata_bytes {
        return Err(invalid_budget());
    }
    let mut hasher = Sha256::new();
    let mut hash_work = 0u64;
    for chunk in input.chunks(4096) {
        if stop.load(Ordering::Acquire) {
            return Err(ContextError::new(ContextErrorCode::Cancelled));
        }
        hash_work = hash_work
            .checked_add(1)
            .filter(|work| *work <= limits.max_context_work)
            .ok_or_else(|| {
                ContextError::new(ContextErrorCode::MinimumRequiredContentExceedsBudget)
            })?;
        hasher.update(chunk);
    }
    let digest = format!("sha256:{}", lower_hex(&hasher.finalize()));
    let mut scanner = Scanner::new(input, limits, stop);
    scanner.work = hash_work;
    let syntax_valid = scanner.scan()?;
    let syntax_error = if !syntax_valid {
        Some(if scanner.duplicate_key {
            PrimaryPackValidationCode::DuplicateObjectKey
        } else if scanner.noncanonical_integer {
            PrimaryPackValidationCode::NonCanonicalInteger
        } else if scanner.forbidden_number {
            PrimaryPackValidationCode::NullOrFloatForbidden
        } else {
            PrimaryPackValidationCode::InputNotCanonicalJson
        })
    } else {
        None
    };
    let (_, _, _, preflight_code) = classify(&scanner.header);
    let decoder_eligible = if syntax_error.is_none()
        && preflight_code == PrimaryPackValidationCode::RecursiveWireClosurePending
    {
        check_pack_core_identity(&mut scanner)? == Some(true)
    } else {
        false
    };
    // Reserve the retained decoded view before serde allocates it. The typed
    // decoder later proves its canonical serialization equals this exact size.
    let decoded_view_bytes = if decoder_eligible { input_bytes } else { 0 };
    Ok(PrimaryPackAdmission {
        scanner,
        digest,
        input_len: input.len(),
        limits,
        syntax_error,
        decoder_eligible,
        decoded_view_bytes,
    })
}

/// Hash the exact canonical PackCore bytes in the received document. The
/// canonical preflight proves the subspan is already E0 JSON; no wire value is
/// reconstructed or reserialized for this identity equation.
fn check_pack_core_identity(scanner: &mut Scanner<'_>) -> ContextResult<Option<bool>> {
    let Some(advertised) = scanner.header.semantic_pack_id.as_deref() else {
        return Ok(None);
    };
    let Some((start, end)) = scanner.pack_core_span else {
        return Ok(None);
    };
    let Some(core) = scanner.input.get(start..end) else {
        return Ok(None);
    };
    let mut hasher = Sha256::new();
    hasher.update(b"{\"domain\":\"");
    hasher.update(PACK_SCHEMA.as_bytes());
    hasher.update(b"\",\"value\":");

    // Charge each borrowed span before hashing it; the exact bytes and the
    // domain-separated identity equation remain unchanged.
    let Scanner {
        stop, limits, work, ..
    } = scanner;
    for chunk in core.chunks(4096) {
        if stop.load(Ordering::Acquire) {
            return Err(ContextError::new(ContextErrorCode::Cancelled));
        }
        *work = (*work)
            .checked_add(1)
            .filter(|value| *value <= limits.max_context_work)
            .ok_or_else(|| {
                ContextError::new(ContextErrorCode::MinimumRequiredContentExceedsBudget)
            })?;
        hasher.update(chunk);
    }
    hasher.update(b"}");
    let expected = format!("context-pack:sha256:{}", lower_hex(&hasher.finalize()));
    Ok(Some(advertised == expected))
}

fn classify(
    header: &HeaderProbe,
) -> (
    PrimaryPackValidationState,
    PrimaryPackValidationState,
    PrimaryPackValidationState,
    PrimaryPackValidationCode,
) {
    if header.pack_schema.as_deref() != Some(PACK_SCHEMA) {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationCode::UnsupportedArtifactSchema,
        );
    }
    if header.selector_type_invalid {
        return (
            PrimaryPackValidationState::Invalid,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::Invalid,
            PrimaryPackValidationCode::SelectorTypeInvalid,
        );
    }
    if header.profile_id.is_none()
        || header.request_profile_id.is_none()
        || header.level.is_none()
        || header.root_kind.is_none()
    {
        return (
            PrimaryPackValidationState::Invalid,
            PrimaryPackValidationState::Invalid,
            PrimaryPackValidationState::Invalid,
            PrimaryPackValidationCode::MissingRequiredSelector,
        );
    }
    if header.profile_id.as_deref() != Some(PACK_PROFILE)
        || header.request_profile_id.as_deref() != Some(PACK_PROFILE)
    {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationCode::UnsupportedProfile,
        );
    }
    if header.level.as_deref() != Some("l0") || header.skeleton_level.as_deref() != Some("l0") {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationCode::UnsupportedLevel,
        );
    }
    if header.root_kind.as_deref() != Some("entity") {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationCode::UnsupportedRoot,
        );
    }
    if header.l0_profile_present
        || header.l1_profile_present
        || header
            .l0_profile
            .as_deref()
            .is_some_and(|profile| profile != L0_PROFILE)
    {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationCode::UnsupportedProfile,
        );
    }
    if header.expansion_present {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationCode::UnsupportedExpansion,
        );
    }
    if header.source_excerpt_present {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationCode::UnsupportedSourceExcerpt,
        );
    }
    if header.l0_schema.as_deref() != Some(L0_SCHEMA)
        || header.l0_profile.as_deref() != Some(L0_PROFILE)
    {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationCode::UnsupportedProfile,
        );
    }
    if header.map_schema.as_deref() != Some(MAP_SCHEMA)
        || header.map_profile.as_deref() != Some(MAP_PROFILE)
    {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationCode::UnsupportedMapProfile,
        );
    }
    if !matches!(
        header.scope_node_kind.as_deref(),
        Some("source_file" | "source_package")
    ) || header.scope_node_kind != header.map_root_kind
    {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::Valid,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationCode::UnsupportedRootKind,
        );
    }
    if header.unsupported_load_custody {
        return (
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationState::NotEvaluated,
            PrimaryPackValidationCode::UnsupportedLoadCustody,
        );
    }
    (
        PrimaryPackValidationState::NotEvaluated,
        PrimaryPackValidationState::NotEvaluated,
        PrimaryPackValidationState::NotEvaluated,
        PrimaryPackValidationCode::RecursiveWireClosurePending,
    )
}

fn selection(header: &HeaderProbe) -> Option<PrimaryPackSelection> {
    let level = match header.level.as_deref()? {
        "l0" => PrimaryPackLevel::L0,
        "l1" => PrimaryPackLevel::L1,
        "both" => PrimaryPackLevel::Both,
        _ => PrimaryPackLevel::Other,
    };
    let root = match header.root_kind.as_deref()? {
        "entity" => PrimaryPackRootVariant::Entity,
        "project" => PrimaryPackRootVariant::Project,
        _ => PrimaryPackRootVariant::Other,
    };
    let l0_profile = if !header.l0_profile_present {
        PrimaryPackL0Profile::Omitted
    } else {
        match header.l0_profile.as_deref() {
            Some(L0_PROFILE) => PrimaryPackL0Profile::ContainersV1,
            Some("wow-context/primary-state-root-l0/1") => PrimaryPackL0Profile::StateRootV1,
            Some("wow-context/primary-reference-root-l0/1") => {
                PrimaryPackL0Profile::ReferenceRootV1
            }
            Some(_) => PrimaryPackL0Profile::Other,
            None => PrimaryPackL0Profile::Other,
        }
    };
    Some(PrimaryPackSelection {
        level,
        root,
        l0_profile,
        expansion_present: header.expansion_present,
        source_excerpt_present: header.source_excerpt_present,
        unsupported_load_custody: header.unsupported_load_custody,
    })
}

struct ReportInput<'a> {
    state: PrimaryPackValidationState,
    schema: Option<&'a str>,
    digest: String,
    byte_length: usize,
    semantic_pack_id: Option<&'a str>,
    profile_id: Option<&'a str>,
    selection: Option<PrimaryPackSelection>,
    canonical_bytes: PrimaryPackValidationState,
    schema_state: PrimaryPackValidationState,
    compiled_profile: PrimaryPackValidationState,
    digest_and_identity: PrimaryPackValidationState,
    internal_references: PrimaryPackValidationState,
    code: Option<PrimaryPackValidationCode>,
    schema_code: Option<PrimaryPackValidationCode>,
    profile_code: Option<PrimaryPackValidationCode>,
    digest_code: Option<PrimaryPackValidationCode>,
    references_code: Option<PrimaryPackValidationCode>,
}

fn report(
    input: ReportInput<'_>,
    scanner: &Scanner<'_>,
    limits: PrimaryPackValidationLimits,
) -> PrimaryPackValidation {
    let ReportInput {
        state,
        schema,
        digest,
        byte_length,
        semantic_pack_id,
        profile_id,
        selection,
        canonical_bytes,
        schema_state,
        compiled_profile,
        digest_and_identity,
        internal_references,
        code,
        schema_code,
        profile_code,
        digest_code,
        references_code,
    } = input;
    let mut error_codes = Vec::new();
    if state == PrimaryPackValidationState::Invalid {
        for candidate in [
            code,
            schema_code,
            profile_code,
            digest_code,
            references_code,
        ]
        .into_iter()
        .flatten()
        {
            if !error_codes.contains(&candidate) {
                error_codes.push(candidate);
            }
        }
    }
    let codes = error_codes.into_boxed_slice();
    let records = [
        record(
            PrimaryPackValidationCheck::ByteAdmission,
            PrimaryPackValidationState::Valid,
            None,
        ),
        record(
            PrimaryPackValidationCheck::CanonicalBytes,
            canonical_bytes,
            (canonical_bytes == PrimaryPackValidationState::Invalid)
                .then_some(code)
                .flatten(),
        ),
        record(
            PrimaryPackValidationCheck::Schema,
            schema_state,
            schema_code.or((schema_state != PrimaryPackValidationState::Valid)
                .then_some(code)
                .flatten()),
        ),
        record(
            PrimaryPackValidationCheck::CompiledProfile,
            compiled_profile,
            profile_code.or((compiled_profile != PrimaryPackValidationState::Valid)
                .then_some(code)
                .flatten()),
        ),
        record(
            PrimaryPackValidationCheck::DigestAndIdentity,
            digest_and_identity,
            digest_code.or((digest_and_identity != PrimaryPackValidationState::Valid)
                .then_some(code)
                .flatten()),
        ),
        record(
            PrimaryPackValidationCheck::InternalReferences,
            internal_references,
            references_code.or((internal_references != PrimaryPackValidationState::Valid)
                .then_some(code)
                .flatten()),
        ),
        record(
            PrimaryPackValidationCheck::OwnerGenerationClosure,
            PrimaryPackValidationState::NotEvaluated,
            None,
        ),
        record(
            PrimaryPackValidationCheck::SourceOrRuntimeAuthority,
            PrimaryPackValidationState::NotEvaluated,
            None,
        ),
    ];
    PrimaryPackValidation {
        state,
        artifact_schema: schema.map(Into::into),
        artifact_digest: digest.into_boxed_str(),
        byte_length: byte_length as u64,
        semantic_pack_id: semantic_pack_id.map(Into::into),
        profile_id: profile_id.map(Into::into),
        selection,
        origin_closure: PrimaryPackOriginClosure::StructuralOnly,
        canonical_bytes,
        schema: schema_state,
        compiled_profile,
        digest_and_identity,
        internal_references,
        owner_generation_closure: PrimaryPackValidationState::NotEvaluated,
        source_or_runtime_authority: PrimaryPackValidationState::NotEvaluated,
        records: records.into(),
        errors: if state == PrimaryPackValidationState::Invalid {
            codes.clone()
        } else {
            Box::new([])
        },
        warnings: Box::new([]),
        coverage: Box::new([]),
        omissions: Box::new([]),
        budget_usage: PrimaryPackValidationBudgetUsage {
            input_bytes: byte_length as u64,
            scanner_work: scanner.work,
            scanner_tokens: scanner.tokens,
            max_input_bytes: limits.max_input_bytes,
            max_metadata_bytes: limits.max_metadata_bytes,
            max_context_work: limits.max_context_work,
            max_depth: limits.max_depth,
            max_tokens: limits.max_tokens,
        },
        nonrepairing: true,
    }
}

fn record(
    check: PrimaryPackValidationCheck,
    state: PrimaryPackValidationState,
    code: Option<PrimaryPackValidationCode>,
) -> PrimaryPackValidationRecord {
    PrimaryPackValidationRecord { check, state, code }
}

fn invalid_budget() -> ContextError {
    ContextError::new(ContextErrorCode::MinimumRequiredContentExceedsBudget)
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

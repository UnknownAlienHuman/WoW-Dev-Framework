# `wow-annotations` E1-C data model

**Status:** normative semantic model, rendering profile, artifact, source-map, loss, parity, and consumer-probe data contract.

Concrete Rust modules and serialized layouts may differ. Identity, profile/generation closure, lowering status, source mapping, projection loss, parity, and deterministic artifact semantics may not.

## 1. Object graph

```text
AnnotationBuildRequest
├── exact ReferenceView
├── AnnotationSemanticProfile
├── TypeLoweringProfile
├── LayoutRenderingProfile
├── WowDialectProfile
├── DocumentationSanitizationProfile
├── SourceMapLossProfile
├── ConsumerCapabilityProfile[]
└── budgets/cancellation

-> AnnotationSemanticModel
   ├── AnnotationModule[]
   ├── AnnotationDeclaration[]
   ├── AnnotationMember[]
   ├── AnnotationType[]
   ├── AnnotationDocumentation[]
   ├── AnnotationRestrictionProjection[]
   ├── ProjectionStatusRecord[]
   └── ReferenceProjectionLink[]

-> RenderedAnnotationArtifact
   ├── RenderedAnnotationFile[]
   ├── GeneratedSourceMap
   ├── ProjectionCoverageReport
   ├── ProjectionLossReport
   ├── ConsumerManifest
   ├── SemanticParityManifest/Report
   └── AnnotationArtifactManifest
```

## 2. Build request

```text
AnnotationBuildRequest
    request_id
    ReferenceView ID
    ProfileId / ReferenceGenerationId
    required reference capabilities/partitions
    semantic model profile ID
    type-lowering profile ID
    layout/rendering profile ID
    dialect/global profile ID
    docs sanitization profile ID
    source-map/loss profile ID
    consumer capability profile IDs[]
    artifact eligibility target
    budgets/cancellation
```

No implicit profile, oracle, consumer, layout, or output root.

## 3. Semantic profile

```text
AnnotationSemanticProfile
    profile_id/version
    supported reference entity/fact kinds
    declaration kinds
    member/ownership rules
    documentation/deprecation/restriction projection fields
    exact required reference capabilities
    semantic ordering rules
    unsupported/deferred rules
    canonical digest
```

Semantic profile does not contain file paths or whitespace.

## 4. Module

```text
AnnotationModule
    module_id
    module kind
    exact profile/reference generation
    logical namespace/system/owner scope
    declaration IDs[]
    reference entity/fact/source refs[]
    projection status/coverage
    canonical digest
```

Initial module kinds:

```text
core-globals
api-system
named-types
structures-tables
events
enums
cvars
widgets
script-objects
restriction-analysis-types
dialect
```

## 5. Declaration

```text
AnnotationDeclaration
    declaration_id
    declaration kind
    canonical logical name
    owner/module/namespace/receiver refs
    type/signature/member refs
    modifiers/visibility/availability/deprecation refs
    documentation ref
    restriction projection refs
    reference projection links[]
    projection status
    semantic order key
    canonical digest
```

Initial kinds:

```text
namespace_or_system
function
method
class
interface_like_class
alias
enum
enum_value
global
field
event_payload_alias_or_callback
callback_or_function_type
nominal_secret_type
dialect_marker
```

Exact supported set freezes through consumer profiles.

## 6. Member

```text
AnnotationMember
    member_id
    owner declaration ID
    kind: parameter | return | field | enum_value | payload | type_parameter
    semantic ordinal
    canonical source name
    rendered-name policy
    annotation type ID
    optional/default/variadic/nilability/status fields
    docs/restriction/reference links
    projection status
    canonical digest
```

Parameter optionality and type nilability are separate.

## 7. Consumer-neutral annotation type

```text
AnnotationType
    type_id
    kind
    child/reference/member IDs
    exact/loss status
    source reference type/fact IDs
    lowering rule/version
    consumer constraints
    canonical digest
```

Kinds may include:

```text
nil
boolean
integer
number
string
literal
named
array
map
tuple
union
intersection_if_supported_by-profile
optional_wrapper
function_or_callback
variadic
unknown_explicit
any_explicit
nominal_secret
consumer_extension
unsupported_placeholder_with-loss
```

Renderer later selects exact syntax per consumer/profile.

## 8. Documentation

```text
AnnotationDocumentation
    documentation_id
    source raw/fact IDs
    normalized text fragments
    sanitization profile/rule IDs
    rendered/omitted/truncated state
    loss record refs
    canonical semantic digest
```

Docs never determine declaration identity, type, restriction, or replacement semantics.

## 9. Restriction projection

```text
AnnotationRestrictionProjection
    projection_id
    exact ReferenceEntity/Fact/RestrictionFacet/Predicate IDs
    target declaration/member/type position
    projection form: nominal_type | union | tag | sidecar_only | unsupported
    consumer profile applicability
    analysis-only marker
    runtime-gap marker
    projection status/loss refs
    canonical digest
```

Raw facet payload remains in ReferenceView, not copied as canonical annotation truth.

## 10. Reference projection link

```text
ReferenceProjectionLink
    link_id
    annotation model element ID
    ReferenceEntity/Fact/RawObservation/CorrectionApplication/Evidence/SourceHandle IDs
    exact profile/reference generation
    projection/lowering rule IDs
    source coverage/conflict refs
```

Every material semantic element has at least one exact input link or identified deterministic derivation inputs.

## 11. Projection status

```text
ProjectionStatus
    Exact
    ExactWithSidecar
    LossyDeclared
    Unsupported
    NotEvaluated
```

```text
ProjectionStatusRecord
    record_id
    input reference fact/type/field IDs
    output semantic element IDs[]
    status
    lowering/rendering/consumer profile IDs
    source/projection coverage
    loss/sidecar refs
    reason codes
    canonical digest
```

## 12. Loss record

```text
ProjectionLossRecord
    loss_id
    category
    severity/policy
    exact input reference IDs
    affected output semantic/file/span IDs
    consumer profile(s)
    lowering/rendering/sanitization rule
    emitted approximation/omission/sidecar
    affected capabilities
    source/reference coverage/conflict state
    remediation/review status
    canonical digest
```

Categories:

```text
unrepresentable_type
consumer_syntax_gap
unknown_reference_field
unsupported_reference_fact
conditional_or_runtime_restriction_gap
documentation_sanitized_or_truncated
invalid_identifier_rendering
layout_partition_difference
oracle_difference
source_conflict_or_partial
budget_truncation
deferred_capability
```

## 13. Projection coverage

```text
ProjectionCoverageRecord
    record_id
    profile/reference generation
    semantic module/declaration/type/file/consumer capability partition
    reference coverage input IDs
    projection status/loss/conflict IDs
    status: Complete | Partial | Unknown | Failed | NotApplicable | NotEvaluated
    producer/version
```

Reference coverage and projection coverage are stored separately.

## 14. Layout/rendering profile

```text
LayoutRenderingProfile
    profile_id/version
    compatibility target/name
    file partition/path rules
    module/file header rules
    declaration stub templates
    ordering/spacing/line-ending/encoding rules
    identifier rendering rules
    documentation rendering rules
    consumer syntax profile mapping
    file/declaration budgets
    canonical digest
```

No hidden current Ketho convention; exact profile/version is part of artifact identity.

## 15. Type-lowering profile

```text
TypeLoweringProfile
    profile_id/version
    source type model version
    semantic type rules
    consumer-specific syntax capabilities
    optionality/nilability/variadic/callback rules
    named-reference resolution policy
    restriction/Secret projection rules
    unsupported/loss policy
    canonical digest
```

## 16. Dialect/global profile

```text
WowDialectProfile
    profile_id/version
    exact ReferenceProfile/ReferenceGeneration applicability
    allowed/removed standard globals
    Blizzard globals/namespaces
    require-like/nonstandard symbols
    restricted/secure environment globals
    type/global declarations to emit
    evidence/source refs
    canonical digest
```

No editor setting mutation.

## 17. Consumer capability profile

```text
ConsumerCapabilityProfile
    consumer_profile_id
    consumer kind: EmmyLua | LuaLS | other reviewed
    exact version/revision/features/config probe identity
    supported annotation tags/type forms/declaration syntax
    source span/diagnostic behavior assumptions
    known limitations/workarounds permitted
    forbidden mutation/suppression behavior
    canonical digest
```

## 18. Semantic model

```text
AnnotationSemanticModel
    model_id/version
    exact ProfileId / ReferenceGenerationId / ReferenceView ID
    semantic/type/dialect/consumer profile IDs
    modules/declarations/members/types/docs/restriction projections
    reference projection links
    projection status/coverage/loss candidate records
    semantic manifest/counts/digests
    eligibility state
    canonical digest
```

No file paths/spans yet.

## 19. Rendered file

```text
RenderedAnnotationFile
    file_id
    normalized artifact-relative path
    logical file kind/module refs
    UTF-8 LF bytes
    byte length/line count/SHA-256
    ordered rendered declaration/member fragments
    consumer/layout profile applicability
    file capability/coverage/loss state
```

File path derives only from validated renderer-owned logical names/profile rules.

## 20. Rendered fragment

```text
RenderedFragment
    fragment_id
    semantic element ID
    rendered file ID
    byte and line span
    renderer template/rule ID
    sanitized doc/string/identifier refs
    reference projection link IDs
    digest
```

## 21. Generated source map

```text
GeneratedSourceMap
    source_map_id/version
    artifact/file manifest IDs
    entries[]
    exact reference/profile/generation
    lowering/rendering profile IDs
    canonical digest
```

```text
GeneratedSourceMapEntry
    entry_id
    generated file/span/fragment/semantic element IDs
    reference entity/fact/raw/correction/evidence/source handle IDs
    projection/loss/status IDs
```

All generated spans validate against final file bytes/digests.

## 22. File manifest

```text
AnnotationFileManifest
    file_manifest_id
    artifact ID candidate context
    layout profile
    ordered file entries(path/kind/digest/length/line count/module refs)
    total files/bytes/declarations
    canonical digest
```

## 23. Semantic manifest

```text
AnnotationSemanticManifest
    semantic_manifest_id
    model/profile/reference/type/dialect/consumer IDs
    module/declaration/member/type/doc/restriction counts/digests
    projection status/coverage/loss counts/digests
    reference input manifest refs
    canonical digest
```

## 24. Artifact manifest

```text
AnnotationArtifactManifest
    artifact_id/version
    exact ProfileId / ReferenceGenerationId
    ReferenceView/ReferenceDataManifest IDs
    semantic/type/layout/dialect/docs/source-map/loss/consumer profile IDs
    semantic/file/source-map/coverage/loss/consumer/parity manifest IDs
    licenses/provenance/tool versions
    eligibility state
    checksums
    canonical digest
```

Artifact ID construction is noncyclic and excludes fields containing itself.

## 25. Parity model

```text
SemanticParityBaseline
    baseline_id
    oracle kind/revision/input source/profile/artifact IDs
    normalization/comparison profile
    semantic declaration/type/member manifest
    file/layout manifest: optional
    canonical digest
```

```text
SemanticParityRecord
    record_id
    subject semantic declaration/member/type/file
    our IDs/baseline IDs
    classification
    exact diff fields
    source/reference/consumer evidence
    required action/status
```

```text
SemanticParityReport
    report_id
    baseline/artifact/profile/reference/comparison IDs
    classification counts/records
    unresolved blockers
    semantic parity capability state
    canonical digest
```

## 26. Consumer probe

```text
ConsumerProbeRequest
    request_id
    artifact/file manifest
    exact consumer profile/version/config fixture
    expected positive/negative symbols/types/diagnostics/source spans
    no-config-mutation assertions
    budgets
```

```text
ConsumerProbeResult
    result_id
    consumer/profile/artifact IDs
    load/parse/index outcomes
    expected semantic/diagnostic assertions
    source-map/span observations
    config/filesystem mutation audit
    performance/resource observations
    status/coverage/errors
    raw log digest/handle: bounded external evidence
```

## 27. Artifact build report

```text
AnnotationBuildReport
    report_id
    request/reference/profile/profile IDs
    semantic model stage counts/status
    rendering/file/source-map counts/digests
    exact/lossy/unsupported/NotEvaluated counts
    consumer/parity statuses
    budget/cancellation
    eligibility/deferred capabilities
    canonical digest
```

## 28. Canonical ordering

```text
modules by kind/system/namespace/ID
declarations by module/kind/owner/name/signature/ID
members by semantic ordinal/kind/name/ID
types by structural canonical key/ID
docs/restrictions/links/status/loss by subject/rule/consumer/ID
files by layout partition order/path/file ID
fragments/source-map entries by file/span/semantic element/ID
parity records by subject/classification/field/ID
consumer profiles/results by consumer/profile ID
```

## 29. Budgets

```text
input facts/declarations/members/type nodes/depth/docs bytes
semantic model modules/declarations/members/types/links/loss records
rendered file count/declarations/lines/bytes
source-map entries/bytes
parity baseline/report records/bytes
consumer probe output/log bytes
```

Limit/truncation affects artifact eligibility and relevant projection coverage; never silent.

## 30. Required operations

```text
validate_annotation_build_request
build_annotation_semantic_profile
build_annotation_semantic_model
build_annotation_modules_declarations_members
lower_reference_types
project_reference_restrictions
sanitize_annotation_documentation_and_identifiers
build_projection_status_coverage_and_loss
validate_annotation_semantic_model
build_layout_rendering_profile
render_annotation_files
build_generated_source_map
validate_rendered_files_and_source_map
build_annotation_semantic_and_file_manifests
build_annotation_artifact_manifest
build_semantic_parity_baseline_or_report
validate_consumer_probe_result
classify_annotation_artifact_eligibility
canonicalize_annotation_artifact
```

## 31. Required tests

- context/profile/generation closure across every object;
- semantic IDs independent of layout/spans;
- all declaration/member/type/restriction/status kinds;
- no silent loss or orphan input/output link;
- file/fragment/source-map span/digest closure;
- parity/consumer profiles exact;
- deterministic ordering/manifests/artifact IDs;
- budget/truncation/cancellation;
- no private/source/instruction injection.

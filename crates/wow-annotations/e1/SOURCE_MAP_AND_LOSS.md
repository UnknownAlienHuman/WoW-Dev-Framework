# Generated source maps, projection coverage, and loss reports

**Status:** normative E1-C traceability, generated span, sidecar, projection coverage, and representational loss contract.

A readable generated declaration is insufficient evidence by itself. Every material projection must retain exact links to the ReferenceView facts, corrections, raw observations, coverage/conflicts, and lowering/rendering rules that produced it.

## 1. Source-map inputs

```text
validated AnnotationSemanticModel
final rendered file bytes and file manifest
rendered fragment records
ReferenceView/profile/reference generation
reference projection links
lowering/rendering/sanitization profile IDs
projection status/coverage/loss records
```

Source maps are generated only after final file bytes/digests are stable.

## 2. Generated span contract

```text
GeneratedSpan
    file ID/path/digest
    UTF-8 byte start/end
    line start/end
    optional column units under explicit profile
```

Rules:

- half-open byte ranges;
- line numbering/index convention frozen;
- columns omitted or explicitly encoded in bytes/Unicode scalar/UTF-16 according to consumer profile;
- span must align to final bytes and declared fragment boundaries;
- no span outside file/inside invalid encoding;
- overlapping/nested fragments follow explicit kind policy;
- source map invalid if file digest/length/line count changes.

## 3. Mapping granularity

Material map entries for:

```text
module/file header identity
namespace/system declaration
function/method declaration
parameter/return/member field
type alias/class/enum/event/CVar/widget declaration
restriction analysis type/tag/sidecar marker
deprecation/documentation fragments when emitted
safe generated identifier/alias mapping
```

Whitespace/blank lines need not map unless consumer/source-span contract requires.

## 4. Reference links

Each entry can reference:

```text
ReferenceEntity/Fact IDs
member/type/restriction/predicate/deprecation/transition IDs
RawObservation/UnknownField/UnsupportedConstruct IDs
CorrectionApplication IDs
Evidence/SourceHandle IDs
reference coverage/conflict records
semantic declaration/member/type/doc IDs
lowering/rendering/sanitization rule IDs
projection status/loss IDs
```

At least one exact reference/derivation link per material semantic fragment.

## 5. Source bodies

Source map can point to stable source handles without embedding source text. Optional bounded source snippets/objects are governed by ReferenceData license/object/security policy and are not required for semantic mapping.

No absolute local source/output root in public map.

## 6. Projection coverage

Projection coverage answers:

```text
was the reference fact available/complete?
was it selected by the semantic profile?
was it represented in semantic model?
was it rendered for each declared consumer?
was exact semantics preserved or sidecar/loss required?
was the generated fragment/source map complete?
```

Partitions can be:

```text
module/declaration/member/type/documentation/restriction
rendered file/fragment
consumer profile
layout/type-lowering/sanitization rule
reference capability/system/entity kind
```

Use core statuses Complete/Partial/Unknown/Failed/NotApplicable/NotEvaluated.

## 7. Loss categories

### `unrepresentable_type`

Consumer syntax cannot express exact type shape.

### `consumer_syntax_gap`

One consumer lacks tag/declaration/optional/tuple/enum/generic behavior.

### `unknown_reference_field`

ReferenceView preserves a field but no annotation semantic contract exists.

### `unsupported_reference_fact`

Reference fact/entity/member kind is outside active E1-C projection.

### `conditional_or_runtime_restriction_gap`

Static annotation cannot encode exact conditional/runtime behavior.

### `documentation_sanitized_or_truncated`

Docs altered/omitted for safety/budget/consumer reasons.

### `invalid_identifier_rendering`

Safe exact name form unavailable or generated alias required.

### `layout_partition_difference`

Semantic equivalent represented in different file/path/grouping from oracle/profile.

### `oracle_difference`

Parity discrepancy not itself a reference/source fact.

### `source_conflict_or_partial`

ReferenceView input unresolved/partial/conflicted.

### `budget_truncation`

Model/render/map/report limits omit detail.

### `deferred_capability`

Full UI graph/runtime/search/etc. intentionally not projected.

## 8. Loss severity/policy

```text
Informational
    no required consumer semantic impact; e.g. harmless layout difference

Advisory
    sidecar or documentation fidelity difference

BlockingForDeclaredCapability
    mandatory type/declaration/restriction/source-map semantics lost

BlockingForConsumer
    specific consumer cannot safely consume declared artifact profile

BlockingForReleaseReadyArtifact
    deterministic/security/parity/source-map/loss gate unresolved
```

Severity is profile/consumer/capability policy, not inferred from prose.

## 9. Loss record content

```text
ProjectionLossRecord
    loss ID/category/severity/policy
    exact reference/semantic/generated subjects
    source coverage/conflict/correction state
    consumer/type/layout/sanitization profile IDs
    exact emitted approximation/omission/sidecar
    affected capabilities
    expected reviewer/remediation state
    evidence/source map refs
    canonical digest
```

No loss record may claim source/API defect without source evidence; oracle/consumer findings are separate provenance.

## 10. Sidecars

Sidecar manifests can preserve:

```text
raw/restriction/predicate/provenance references
exact ReferenceEntity/Fact IDs
consumer limitations
projection statuses/losses
source maps
semantic parity classifications
```

Sidecar is versioned and linked in artifact manifest. Consumer ignoring sidecar cannot be considered to have preserved sidecar-only mandatory semantics unless the declared capability profile says so.

## 11. Unknown and unsupported inputs

Every selected ReferenceView unknown/unsupported/conflict/NotEvaluated result gets one of:

```text
semantic Unsupported/NotEvaluated element + loss
module/capability-level loss without declaration
sidecar-only record
explicitly NotApplicable under profile
```

Never absent from both artifact and loss/coverage report.

## 12. Documentation sanitization loss

Record:

```text
source documentation ID
sanitization rule(s)
bytes/lines removed/replaced/truncated
whether semantic declaration unaffected
whether oracle byte/layout parity affected
source-map entry for rendered fragment
```

Do not expose unsafe original text in default public report; stable raw/source handles suffice.

## 13. Identifier/name loss

When source/logical name cannot render exactly:

- bracket/alias form exact -> Exact or ExactWithSidecar;
- deterministic generated safe alias -> LossyDeclared/ExactWithSidecar per profile, with bidirectional map;
- unavailable -> Unsupported;
- collision -> conflict/blocking.

Never silently normalize names.

## 14. Artifact eligibility

Release-ready annotation artifact requires, for declared mandatory capabilities/consumers:

```text
no unreviewed blocking loss
all generated file/source-map closure valid
all selected input facts have status/loss coverage
reference source conflicts/partials declared and allowed by eligibility policy
consumer probes pass required semantics
parity blockers resolved/classified
no budget truncation/security sanitization failure affecting mandatory semantics
```

Nonblocking losses remain manifest-visible.

## 15. Differential use

Parity comparison consumes semantic model/manifests and can add oracle difference records. It cannot remove reference projection loss or change semantic output automatically.

A Ketho omission does not justify our omission if ReferenceView/consumer profile supports exact output; classify oracle difference.

## 16. Consumer use

Downstream rules/service can ask:

```text
is declaration/type/member exact for consumer X?
which ReferenceView fact/source produced this generated span?
which semantics are sidecar-only/lost/NotEvaluated?
is artifact release-ready for declared capability Y?
```

No need to parse human report prose.

## 17. Budgets and truncation

Bound source-map entries, reference links per entry, loss records, report bytes, and sidecar size. If report itself truncates:

- retain total/processed/omitted count state when exact;
- retain blocking summary and continuation/detail handle if supported;
- artifact cannot claim complete loss disclosure for affected scope;
- no deletion of decisive blocking records to fit budget.

## 18. Determinism

Equivalent reference/model/rendered bytes/profiles produce identical:

```text
fragment/source-map entry IDs/spans/order
projection coverage records/summaries
loss records/classifications
sidecar/source-map/loss manifest IDs/digests
artifact eligibility decision
```

Independent of worker/store row/probe order, timestamps, paths, prose formatting.

## 19. Required operations

```text
record_reference_projection_link
record_rendered_fragment
build_generated_source_map_entry
build_generated_source_map
validate_generated_source_map
build_projection_coverage_records
classify_projection_loss
build_projection_loss_record
build_projection_loss_report
build_projection_sidecar_manifest
validate_projection_status_closure
classify_annotation_artifact_eligibility
```

## 20. Required tests

- every declaration/member/type/doc/restriction map granularity;
- final UTF-8 byte/line span closure and file digest mutation;
- generated alias/bracket/unsupported identifier mapping;
- exact/sidecar/lossy/unsupported/NotEvaluated cases;
- unknown/unsupported/conflict input cannot disappear;
- documentation sanitization/truncation records;
- consumer-specific loss;
- blocking/nonblocking artifact eligibility;
- loss-report budget/truncation retains blockers;
- semantic parity adds records without altering output;
- deterministic map/loss/report under worker/order changes;
- no absolute path/full raw source/private payload leak.

## 21. Hard stops

- no generated declaration without reference/derivation link;
- no map span before final bytes;
- no file digest/span drift;
- no silent sidecar-only mandatory semantics;
- no unknown/unsupported/conflict disappearance;
- no loss report used to overwrite source truth;
- no unsafe raw text in public loss report;
- no complete eligibility under truncated/blocking loss.

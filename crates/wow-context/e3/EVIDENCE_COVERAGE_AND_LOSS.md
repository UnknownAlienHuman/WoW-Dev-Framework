# E3-A evidence, coverage, projection loss, and omissions

**Status:** normative traceability and honesty contract.

## Evidence principle

Every material Project Map/skeleton/context field is supported by exact project, graph, reference, evidence, or source records, or by an explicit deterministic derivation over such records. Context prose/layout is never its own evidence.

## Material claim examples

```text
entity kind/name/owner/role
selected TOC/load position
callable signature/member
native/custom/CVar signal registration
hook/state/API-use relation
coverage/conflict/ambiguity status
source excerpt
reason path
```

All require closure.

## Evidence link

```text
ContextEvidenceLink
    context artifact/record/field ID
    exact input entity/relation/assertion/fact/member/source IDs
    exact publication/project/graph/reference generations
    producer/provenance/confidence
    source/evidence handles
    derivation rule/path IDs
    coverage/conflict/ambiguity refs
```

Links are machine-readable. Human footnotes are optional projections.

## Provenance preservation

Do not relabel:

```text
project source as platform source
recognizer-derived role as explicit source declaration
external/candidate evidence as project fact
context summary as analyzer fact
source documentation prose as structured contract
```

Context can juxtapose evidence classes but must identify each.

## Confidence preservation

Context output confidence is no stronger than its exact supporting inputs/derivation policy. Repeated Possible assertions, graph centrality, source popularity, or multiple similar names do not promote confidence.

Candidate is excluded by default and remains Candidate when explicitly requested.

## Coverage axes

Keep separate:

```text
project source/materialization
TOC/XML/load model
analyzer facts/diagnostics
recognizer rule/partition
graph assertion/query
reference/API/restriction
ProjectStore read/query
context projection field family
source excerpt availability
renderer projection
tokenizer/evaluation
```

A complete context renderer does not make a partial graph/source complete.

## Context projection coverage

```text
ContextCoverageRecord
    context artifact/request/profile IDs
    subject/field/section/lane/detail partition
    input coverage/conflict refs
    considered/included/omitted/unsupported/truncated counts/digests
    status = Complete | Partial | Unknown | Failed | NotApplicable | NotEvaluated
    loss/omission/stopping refs
    producer/version
```

`Complete` means complete for the declared context profile partition, not full project/source knowledge.

## Projection status

### `Exact`

The declared semantic field is represented exactly with evidence.

### `ExactWithEvidenceSidecar`

The compact field is exact while necessary provenance/coverage/detail lives in sidecar records.

### `CompactButCompleteForDeclaredFields`

All profile-required fields are present, but the artifact intentionally does not represent full source/entity detail.

### `LossyDeclared`

A representation is useful but known semantics/detail were compacted/approximated under policy.

### `Unsupported`

The active profile cannot represent the input/detail.

### `NotEvaluated`

Required input capability/query was unavailable/conflicted/not run.

### `Truncated`

Budget stopped an otherwise supported scope.

## Loss categories

```text
unsupported_entity_or_field
unsupported_relation_lane_or_detail
input_partial_or_conflicted
confidence_or_ambiguity_compaction
source_excerpt_unavailable_or_forbidden
privacy_or_security_redaction
license_restriction
budget_truncation
renderer_limitation
tokenizer_unavailable_or_estimated
consumer_profile_difference
deferred_search_lineage_runtime_capability
deduplicated_presentation_with_preserved_sidecar
```

## Loss record

```text
ContextLossRecord
    exact input/subject/artifact/field IDs
    category/severity/policy
    emitted compact/approximate/omitted form
    exact semantics/detail not represented
    affected tasks/capabilities/consumers
    input/context coverage/conflict refs
    evidence/source/detail/continuation route refs
    review/remediation state
```

Loss cannot be described only in prose.

## Omission record

An omission can identify exact IDs or, for very large scopes, a canonical partition/count/digest plus deterministic continuation route.

```text
ContextOmissionRecord
    subject scope/partition
    exact total/considered/included/omitted state
    omitted ID set or manifest digest
    reason/priority/budget/blocker
    mandatory versus optional
    recoverable by continuation/detail route
```

If totals are unknown due to partial input, say unknown rather than estimate as exact.

## Deduplication

Presentation deduplication is not evidence deletion. One rendered item retains all exact input/evidence links and conflict differences. If merging would hide disagreement or source occurrence relevant to the task, do not merge.

## Negative authority

Context can present a domain-authoritative absence only when the underlying exact project/graph/reference query result already provides it with complete relevant coverage and no blocking conflict/truncation. Context itself never derives absence from omission, empty section, or budget.

## Conflict handling

- include conflict record and competing assertions/values under profile;
- do not select first/latest/majority/model-preferred value;
- mark dependent context fields NotEvaluated/Lossy/Conflict as applicable;
- independent fields/lanes can remain usable;
- source excerpt does not resolve semantic conflict automatically.

## Partial input

Available exact facts may be included with their exact ceilings. The bundle status and context coverage identify blocked partitions. Do not suppress useful independent context, but do not show missing fields as clean empty.

## Budget truncation

Truncation records:

```text
cutoff work item and stable key
used/reserved/remaining budgets
total/processed/omitted counts where exact
affected roots/lanes/fields
mandatory blocker state
continuation cursor/detail routes
```

## Source excerpt loss

Record when source cannot be included because of:

```text
missing/stale source handle
license/privacy/security policy
source byte/line budget
invalid encoding/span/digest
unsupported origin/object role
```

The structured skeleton/evidence can remain without source text.

## Renderer and tokenizer loss

Semantic bundle coverage is separate from renderer/tokenizer results. A Markdown renderer or tokenizer limitation cannot delete semantic records silently; it creates renderer/tokenizer-specific loss/metric state.

## Artifact eligibility

`ValidatedForDeclaredContextProfile` requires:

```text
all mandatory fields/roots have exact or allowed sidecar representation
all material claims evidence-linked
no hidden conflict/partial/NotEvaluated state
no unreported mandatory omission/truncation
source/privacy/security rules pass
continuation and metrics consistent
determinism and evaluation gates pass
```

Nonblocking losses remain visible.

## Required tests

- material claim without evidence link;
- provenance upgrade/misclassification;
- Possible/Candidate promotion;
- each independent coverage axis;
- exact/sidecar/compact/lossy/unsupported/NotEvaluated/truncated statuses;
- conflict competitors retained;
- partial input useful context plus blockers;
- authoritative absence only from domain query;
- dedup preserves all evidence;
- large omission manifest/continuation;
- source/renderer/tokenizer loss separation;
- blocker survives report budget;
- deterministic records/order/eligibility.

## Hard stops

- no context claim as self-evidence;
- no coverage collapse or authority upgrade;
- no empty/omitted section as absence proof;
- no hidden conflict/partial/truncation;
- no deduplication that deletes evidence;
- no complete eligibility with undisclosed blocking loss.

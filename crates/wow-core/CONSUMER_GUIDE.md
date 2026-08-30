# `wow-core` E0 consumer guide

**Status:** normative E0-A seam contract; no Rust code yet.

This document defines the smallest `wow-core` surface that each downstream E0 work package may consume. It prevents future agents from exporting every internal helper, reconstructing identity from strings, or moving orchestration into core.

## 1. General consumer rules

Every consumer must:

1. receive validated core values rather than raw identity-bearing strings;
2. preserve one exact `GenerationContext` through an operation;
3. create records only through constructors/operations that enforce the owning invariants;
4. keep source observations and platform/reference evidence as separate `SourceHandle` and `EvidenceRecord` objects;
5. retain exact `CoverageRecord` inputs; a `CapabilitySummary` is only a conservative derived view;
6. return `NotEvaluatedRecord` for unavailable required capabilities instead of manufacturing a clean result;
7. leave final cross-component ordering, status, and envelope finalization to `wow-service`;
8. validate source-registry eligibility before assigning platform/project/runtime/external provenance to an opaque `origin_id`;
9. avoid depending on internal modules, DTO implementation details, hash-map layouts, or canonicalization helpers not listed as public operations.

A consumer may use fewer items than listed. This is a maximum E0 seam, not a mandate to import every type.

## 2. `wow-reference` E0-B seam

### Consumes

```text
ProfileId
ProfileIdentity
ReferenceGenerationId
GenerationContext
ContentDigest
EntityKey
ProducerId
CapabilityId
CoveragePartitionId
SourceHandle
EvidenceRecord
ConflictRecord
CoverageRecord
NegativeAuthorityDecision
```

### Uses operations

```text
parse_profile_id
parse_entity_key
parse_coverage_partition_id
parse_content_digest
validate_profile_identity
build_source_handle
verify_source_handle_content
derive_generation_context_id
validate_generation_context
validate_evidence_record
derive_evidence_id
validate_conflict_record
derive_conflict_id
validate_coverage_record
derive_coverage_id
evaluate_negative_authority
```

### Produces

- one fixture `ProfileIdentity` and one `ReferenceGenerationId`;
- exact reference-side source handles;
- platform-contract or source-observation evidence;
- exact per-producer capability/partition coverage records;
- typed negative-authority decisions for exact lookups.

### Must not

- create project source handles;
- summarize coverage for the whole operation;
- create findings or operation envelopes;
- infer a replacement from similarity;
- represent API absence by attaching platform evidence to an addon source span.

For a missing-API finding, `wow-reference` proves the catalog/profile/coverage fact. `wow-rules` separately identifies the project use site and derives the rule finding.

## 3. `wow-emmy` E0-C seam

### Consumes

```text
ProfileIdentity
ReferenceGenerationId
ProjectGenerationId
GenerationContext
ContentDigest
ProducerId
RuleId
CapabilityId
CoveragePartitionId
SourceHandle
EvidenceRecord
CoverageRecord
Finding
```

### Uses operations

```text
parse_content_digest
build_source_handle
derive_generation_context_id
validate_generation_context
require_same_generation
validate_evidence_record
derive_evidence_id
validate_coverage_record
derive_coverage_id
validate_message_arguments
derive_root_cause_key
derive_finding_fingerprint
bind_finding_to_context
```

### Produces

- project-file source handles and exact byte spans;
- analyzer/project source observations;
- generic diagnostic findings normalized to the common contract;
- exact analyzer capability coverage records.

### Must not

- emit platform-source evidence without a reference-owned input;
- combine project and reference generations implicitly;
- decide WoW API absence or Secret legality;
- construct the final result envelope.

## 4. `wow-project` E0-D seam

### Consumes

```text
ProfileIdentity
ReferenceGenerationId
ProjectGenerationId
GenerationContext
ContentDigest
ProducerId
CapabilityId
CoveragePartitionId
SourceHandle
EvidenceRecord
CoverageRecord
```

### Uses operations

```text
normalize_source_path
parse_content_digest
build_source_handle
derive_generation_context_id
validate_generation_context
merge_generation_context
require_same_generation
validate_evidence_record
derive_evidence_id
validate_coverage_record
derive_coverage_id
```

### Produces

- one coherent project generation;
- normalized first-party file identities and content digests;
- project-owned source observations and coverage records needed by E0 rules.

### Must not

- add TOC/XML/graph abstractions outside the explicit E0 fixture;
- interpret platform restriction contracts;
- create operation-level capability summaries or final envelopes.

## 5. `wow-rules` E0-E seam

### Consumes

```text
GenerationContext
EntityKey
RuleId
ProducerId
CapabilityId
CoveragePartitionId
SourceHandle
EvidenceRecord
ConflictRecord
CoverageRecord
CapabilitySummary
NotEvaluatedRecord
Finding
NegativeAuthorityDecision
```

### Uses operations

```text
require_same_generation
validate_evidence_record
derive_evidence_id
derive_evidence
validate_conflict_record
validate_coverage_record
combine_coverage
validate_capability_summary
evaluate_capability_availability
derive_not_evaluated_id
validate_not_evaluated_record
evaluate_negative_authority
validate_message_arguments
derive_root_cause_key
derive_finding_fingerprint
bind_finding_to_context
```

### Produces

- rule-owned derived evidence that cites both project observation and reference contract inputs when required;
- one finding or one `NotEvaluatedRecord` per applicable E0 rule result;
- deterministic root-cause and remediation metadata.

### Must not

- mutate source/project/reference state;
- claim `Proven` for a derived rule conclusion;
- use `Candidate` evidence to authorize an exact edit;
- convert partial/conflicted/truncated coverage into a pass or absence finding;
- fold the final result stream.

## 6. `wow-service` E0-F seam

### Consumes

All E0 public records plus:

```text
CapabilitySummary
WarningRecord
Budget
TruncationState
E0CheckResultEnvelope
E0OperationErrorEnvelope
CoreError
```

### Uses operations

```text
require_profile_identity_match
merge_generation_context
require_same_generation
combine_coverage
validate_capability_summary
evaluate_capability_availability
canonical_finding_order
deduplicate_findings
derive_warning_id
validate_warning_record
validate_budget
accumulate_budget_usage
classify_truncation
validate_result_envelope
canonical_result_order
canonical_result_digest
finalize_result_envelope
validate_schema_version
```

### Produces

- operation-level capability summaries derived from exact coverage records;
- final deterministic ordering and root-cause presentation;
- one complete/partial/failed envelope or one structured operation-error envelope;
- explicit budget usage, truncation, warnings, conflicts, and `NotEvaluated` records.

### Must not

- upgrade evidence confidence;
- replace exact records with summaries;
- repair generation mismatches by silently reacquiring a different context;
- hide optional-lane failures or conflict blockers;
- accept unresolved internal IDs.

## 7. `apps/wow` E0 transport seam

The CLI consumes only validated `wow-service` request/response DTOs. It may render prose, line/column hints, and user-facing messages, but these are presentation projections and do not alter core semantic identity or canonical digests.

The application must not import lower crates to construct a richer answer than the service returned.

## 8. Public API minimization table

| Public family | Required E0 consumers | Keep public only when |
|---|---|---|
| identifiers/digests/profile/context | all E0 crates | cross-crate identity would otherwise be reconstructed from strings |
| path/span/source handle | reference, Emmy, project, rules | exact source evidence crosses a crate boundary |
| evidence/conflict | reference, Emmy, rules, service | source/derivation/conflict truth crosses a crate boundary |
| coverage/summary/NotEvaluated | reference, Emmy, project, rules, service | capability state or evaluation disposition crosses a boundary |
| finding/warning | Emmy, rules, service | normalized analysis output crosses a boundary |
| budget/envelope/error | service, app | operation finalization or transport response crosses a boundary |
| canonicalization internals | none by default | a named cross-crate operation cannot preserve deterministic identity otherwise |

An implementation review must make internal every helper without an identified consumer and executable boundary test.

## 9. Seam acceptance fixtures

Before E0-B/E0-C begin, core must expose fixtures proving:

```text
reference and project source handles remain separate
evidence derivation and conflict graphs are acyclic
coverage records remain present beside capability summaries
complete catalog coverage can support derived absence evidence
conflict/partial/failed coverage blocks rule evaluation
one context binds all records in a final envelope
canonical bytes remain stable under randomized input order
```

The committed examples in [`examples/`](examples/) are the initial seam fixtures. Downstream crates may add fixture inputs, but they may not weaken these outputs to simplify their implementation.

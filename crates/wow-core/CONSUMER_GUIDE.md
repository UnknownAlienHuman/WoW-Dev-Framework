# `wow-core` E0 consumer guide

**Status:** normative E0-A seam contract; partial executable implementation.

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


## Coverage consumer call migration

The checked coverage calls implement the existing CORE-010/CORE-022 proof boundary:

```text
evaluate_capability_availability(
    context_id, producer_id, producer_version, subject_kind, subject_id,
    reason_code, required_summaries, coverage_records, conflicts,
) -> CoreResult<CapabilityAvailability>

evaluate_negative_authority(
    context_id, scope_known, lookup_completed, required_summaries,
    coverage_records, conflicts, candidate_evidence_ids, evaluation, truncation,
) -> CoreResult<NegativeAuthorityDecision>
```

Availability now requires a conflict registry. Negative authority now requires
an expected context and raw records, is fallible, and retains `context_id` in its
output. The old unchecked seven-argument negative call is intentionally removed;
do not replace missing records with defaults or catch admission errors as clean.
No current sibling Rust crate called these operations at this checkpoint; the
existing core regression caller is migrated. Future consumers must use the
checked seam. See [selection rules and limits](COVERAGE_CONSUMERS.md).

## Checked generation composition

Use `require_same_generation` for exact multi-input admission, not a comparison
of caller-supplied `context_id` fields. The guard validates both complete contexts
before comparing their identities and propagates nested errors. A matching digest
is neither authenticity evidence nor proof of an external revision's existence.

`merge_generation_context` does not repair malformed inputs or choose versions.
Schema and producer inventories must match exactly in every mode. Only
`ExtendMissingOptional` can fill an absent project generation; only `ExternalUnion`
can combine compatible external scopes. Neither mode replaces existing owner
identities. Decode first, then use the checked owner operations; direct Serde
construction is not equivalent to validated admission.

## Evidence registry admission

Use `validate_evidence_derivation_graph` on the complete retained evidence registry
before consuming derivation authority. It validates one context, every input link,
confidence ceilings, transitive scenario restrictions, acyclicity and all record
IDs. The check-result envelope uses the same operation. Single-record validation
cannot resolve missing source, coverage or evidence records on its own.

Possible/Candidate parents cannot support a stronger child; runtime ancestry cannot
be relabeled into a platform contract. Independent components retain their own
ancestry. Constructors may sort set inputs, while decoded validators reject
noncanonical/duplicate references even with recomputed hashes. No input record or
conflict is rewritten, selected as a winner or promoted by validation.

The traversal is iterative and input-proportional, including deep/shared DAGs. Hosts
still enforce acquisition/decoding/result-size budgets. Structural acceptance of an
empty registry is not proof of source coverage, absence, or runtime behavior.

## Retained budget and truncation admission

The existing budget/authority seams share truncation validation. A raw
`TruncationState` status tag is not proof: decoded entries must retain canonical
collection/capability sets and unambiguous omission counts. `Budget::new`,
`Budget::validate_limits`, `with_output_bytes`, `classify_truncation`, envelope
validation/finalization, and `evaluate_negative_authority` reject malformed
nested truth; there is no new unchecked fallback or public API requirement.

Fresh construction can order sets. Decoded invalid sets are rejected, not
silently deduplicated. Exact known zero and explicit unknown remain distinct.
Usage counts include only returned records and final serialized bytes; omission
metadata does not increase allowed output. Producers still own completeness and
omission selection, and hosts still enforce acquisition/decoding-size budgets.
The pure core cannot detect a producer that changes both source claims and all
counts consistently without independently supplied source evidence.

## Retained finding and warning inputs

Keep each diagnostic's complete evidence ancestry and referenced handles available
when calling existing bind/validate operations. An ID inventory alone no longer
passes: the actual retained records must validate in the requested evidence
context. The public signatures are unchanged; `wow-rules` already supplies the
owned source/evidence bundles. E0 envelope validation shares a private
admitted registry across the whole result rather than repeating admission per item.

Fresh constructors retain set normalization. Do not pass duplicate/unsorted decoded
reference arrays, half-present warning subjects or constructor-bypassing remediation.
Exact-edit metadata needs a recipe and nonempty, non-Candidate evidence; any plan
handle must be retained. This does not certify or execute remediation. Supply
canonical integer/path argument strings; use Text for non-path prose. Source
eligibility, coverage completeness and actual target-client behavior are not proved
by diagnostic binding or by computing its digest.

## E0 envelope coverage scope and compact evaluation blockers

Assemble every raw record for each summarized capability. Do not put unrelated
partitions of the same capability into an envelope while claiming that a summary
represents only a selected subset: E0 does not encode such a scope selector.
Distinct capabilities can remain optional; availability still evaluates only
required summaries, even if other valid raw records are retained.

Both the complete envelope and negative-authority operation now use the shared
coverage/evaluation admission. Preserve the existing public API and exact golden
bytes. A legacy compact blocker without nested `conflict_ids` is valid when its
raw record and parent evaluation retain all actual conflicts. An explicitly
populated nested set must be exact. Do not strip conflicts from the parent,
fabricate blocking status, or treat a valid local NotEvaluatedId as owner proof.

Run `cargo test --locked -p wow-core --test e0_coverage_join_conformance` for the
focused join regressions, in addition to the existing owner/workspace gates.

## Importing complete E0 JSON

Use `E0CheckResultEnvelope::from_json_slice` or
`E0OperationErrorEnvelope::from_json_slice` with an explicit `E0DecodeLimits` for
untrusted serialized core results. They combine bounded raw admission with the
existing complete validator; direct Serde deserialization is structural only.
Limits come from the caller, never from the input Budget. Bound host acquisition
before producing the slice. Exact API, limits and error rules are in
[`JSON_ADMISSION.md`](JSON_ADMISSION.md). No general-purpose JSON API is exported.

## Profile admission before equality

Use `ProfileIdentityBuilder::build` for new profiles. For a decoded profile use
`validate_profile_identity`, `compare_profile_identity`, or
`require_profile_identity_match`; both comparison operations validate both
operands, including self-comparison. The nonfallible `ProfileIdentity::compare`
method assumes admitted inputs and is not a replacement for these boundaries.
Optional fixture builds must be positive and optional builder ID/version must
be paired. Text bounds and rejection rules are in `DATA_MODEL.md#4-profile-identity`.

## Checked source-handle consumers

Use `verify_source_handle_content(&handle, &source_digest)?` and
`compare_source_handles(&left, &right)?` (or `left.compare(&right)?`). Comparison
now returns `CoreResult<SourceHandleComparison>`; callers must handle rejection.
Both sides are validated before a category is returned, including self-comparison.
The verifier validates handle fields/ID before digest equality. No IO, provenance
attestation, content-span boundary check, context merge or lineage inference is
performed. Raw Serde decoding and `PartialEq` are not substitutes for these APIs.

### Error metadata

Validate assembled `CoreError` values or finalize `E0OperationErrorEnvelope`
before publishing them. Use canonical core IDs for Identifier arguments and
subject IDs, canonical repository paths for Path arguments, and schema field
coordinates rather than host paths. A subject kind can be retained without an ID
when no trustworthy ID is available. Reject invalid metadata; do not repair it
or retry finalization with the rejected value disguised as Text.
`ToolVersion` wire decoding enforces the same no-build-metadata policy as parsing.

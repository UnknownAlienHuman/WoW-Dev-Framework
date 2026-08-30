# `wow-core` data model

**Status:** normative E0-A value contract; no Rust code yet.

This document defines semantic fields and invariants. Concrete Rust structs, enum names, lifetimes, and module paths may differ only when the same semantics remain mechanically testable.

## 1. Primitive conventions

### Text

- Contract identifiers are valid UTF-8.
- Administrative IDs are lowercase ASCII and family-validated.
- Source paths/entity payloads may preserve case-sensitive UTF-8 text.
- NUL and C0/C1 control characters are rejected from identifiers and paths.
- Free-form human notes are never identity-critical.

### Numbers

- Offsets, counts, Interface values, and client build numbers are nonnegative integers.
- Identity-critical serialized numbers must be within the interoperable exact-integer range `0..=9007199254740991`.
- Values outside that range use a canonical decimal-string field defined by the owning schema; they are not emitted as lossy JSON numbers.
- Floating-point values are forbidden in E0 identity, coverage, finding, and result contracts.

### Canonical enum spellings

Serialized enum values use lowercase snake case, for example:

```text
complete partial unknown failed not_applicable not_evaluated
proven derived possible candidate
source_observation platform_contract project_fact runtime_scenario
error warning information hint
shadow advisory blocking
```

In-memory type names may follow Rust naming conventions, but serialization and hash material use the canonical spellings above.

### Collections

- Set-like collections contain no duplicates and are canonically sorted.
- Ordered domain collections document their ordering explicitly.
- Dynamic key/value data uses arrays of typed entries rather than externally controlled JSON object keys.
- Absence and an empty collection are distinct only when the owning field defines that distinction.

## 2. Identifier families

### Shared segment grammar

```text
lower_segment  := [a-z][a-z0-9_-]{0,62}
dotted_segment := [a-z][a-z0-9_]{0,62}
slug           := [a-z0-9][a-z0-9._-]{0,95}
qualified_id   := dotted_segment ("." dotted_segment)*
dotted_id      := dotted_segment ("." dotted_segment)+
sha256_hex     := exactly 64 lowercase hexadecimal characters
```

Reserved whole segments:

```text
current latest live head default auto implicit unknown
```

The reserved words may appear as part of a longer descriptive slug, but not as a complete identity segment whose meaning would float.

### Identifier table

| Type | Canonical form | Notes |
|---|---|---|
| `ProfileId` | `profile:<namespace>:<slug>` | Stable label only; structured profile identity remains authoritative. |
| `ReferenceGenerationId` | `generation:reference:sha256:<sha256_hex>` | Derived from canonical reference-generation material. |
| `ProjectGenerationId` | `generation:project:sha256:<sha256_hex>` | Derived from reference ID + workspace/config/file/tool material. |
| `ExternalGenerationId` | `generation:external:<provider>:sha256:<sha256_hex>` | Provider is a lower segment. |
| `ContentDigest` | `sha256:<sha256_hex>` | Full digest; no truncated identity. |
| `StableHandleId` | `handle:sha256:<sha256_hex>` | Derived from canonical source-handle identity projection. |
| `EvidenceId` | `evidence:sha256:<sha256_hex>` | Derived from evidence identity projection. |
| `ConflictId` | `conflict:sha256:<sha256_hex>` | Derived from an unresolved evidence-conflict relation. |
| `CoverageId` | `coverage:sha256:<sha256_hex>` | Derived from one producer/context/capability/partition coverage record. |
| `FindingFingerprint` | `finding-fingerprint:sha256:<sha256_hex>` | Semantic finding identity before generation binding. |
| `FindingId` | `finding:sha256:<sha256_hex>` | Context ID + finding fingerprint. |
| `GenerationContextId` | `context:sha256:<sha256_hex>` | Derived from one canonical generation context. |
| `RootCauseKey` | `root-cause:sha256:<sha256_hex>` | Deterministic causal grouping key, not message-text hash. |
| `NotEvaluatedId` | `not-evaluated:sha256:<sha256_hex>` | Context + producer + rule/operation + blocking capability material. |
| `WarningId` | `warning:sha256:<sha256_hex>` | Context-bound structured non-finding warning identity. |
| `RuleId` | `dotted_id` | Examples: `wow.api.exists`, `wow.secret.local_operation`. |
| `ProducerId` | `dotted_id` | Examples: `wow.reference`, `wow.emmy`, `wow.service`. |
| `CapabilityId` | `dotted_id` | Describes an available analysis/query capability. |
| `OperationId` | `dotted_id` | Examples: `wow.status`, `wow.check`. |
| `MessageCode` | `dotted_id` | Stable finding/error message code, independent of prose. |
| `SchemaId` | `schema:<namespace>:<slug>` | Version stored separately. |
| `ToolVersion` | canonical Semantic Version string | Prerelease allowed; build metadata excluded from identity unless contract says otherwise. |

### Structured keys with exact payloads

`EntityKey` and `CoveragePartitionId` contain a validated lower-case kind/scope plus an exact payload that may be case-sensitive.

```text
EntityKey:
  kind: qualified_id
  key: nonempty UTF-8 payload
  canonical string: entity:<kind>:<percent-encoded-key>

CoveragePartitionId:
  scope: qualified_id
  key: optional exact UTF-8 payload
  canonical string without key: partition:<scope>
  canonical string with key:    partition:<scope>:<percent-encoded-key>
```

Percent encoding:

- UTF-8 bytes in the unreserved set `A-Z a-z 0-9 - . _ ~` remain literal;
- every other byte is `%` followed by two uppercase hexadecimal digits;
- decoding must be lossless and produce valid UTF-8;
- a parser rejects noncanonical encodings such as lowercase hex or encoding an unreserved byte.

Examples:

```text
entity:api:C_UnitAuras.GetAuraDataByIndex
partition:project.file:Core%2FInit.lua
partition:restriction.facet:secret.return
```

## 3. Digest values and purposes

The common digest representation is:

```text
algorithm: sha256
hex: 64 lowercase hexadecimal characters
canonical: sha256:<hex>
```

Digest purpose remains typed or field-specific. Callers must not compare unlike purposes merely because the bytes match.

Required E0 purposes:

```text
source_content
source_logical_snapshot
generation_material
source_handle_identity
evidence_identity
finding_identity
context_identity
canonical_result
```

A Git commit SHA is stored as `source_revision`, not converted into a `ContentDigest` unless the actual content bytes were independently hashed.

## 4. Profile identity

### Fields

| Field | Required | Contract |
|---|---:|---|
| `profile_id` | yes | Valid `ProfileId`. |
| `profile_kind` | yes | `fixture` or `release`. |
| `flavor_id` | yes | Lower segment such as `retail`; core does not define the complete flavor registry. |
| `edition_id` | no | Lower segment for an additional product partition when needed. |
| `interface` | yes | Positive integer; semantic build mapping validated by `wow-reference`. |
| `client_version` | release: yes; fixture: optional | Canonical dotted/version string supplied by reference owner. |
| `client_build` | release: yes; fixture: optional | Positive integer. |
| `source_kind` | yes | `synthetic_fixture` or `blizzard_snapshot` in E0. |
| `source_revision` | yes | Exact immutable revision label; no floating branch name for release profiles. |
| `source_logical_digest` | yes | Full `ContentDigest`. |
| `builder_id` | release: yes; fixture: optional | `ProducerId`. |
| `builder_version` | release: yes; fixture: optional | `ToolVersion`. |
| `schema_versions` | yes | Sorted nonempty list of schema entries required to interpret the profile. |
| `correction_set_digest` | release: yes; fixture: optional | Full digest; explicit empty-set digest is allowed. |
| `fixture_scope` | fixture: yes; release: forbidden | Describes the intentionally incomplete capability boundary. |

### Invariants

1. `fixture` requires `source_kind = synthetic_fixture` unless a fixture explicitly pins a real snapshot as test input; that exception remains fixture-labeled.
2. `release` forbids `source_kind = synthetic_fixture`.
3. A release source revision cannot be `main`, `master`, `live`, `latest`, `head`, or another floating ref.
4. In E0, namespace `fixture` requires `profile_kind = fixture` and namespace `wow` requires `profile_kind = release`; other namespaces are unsupported until a contract revision.
5. Duplicate schema IDs with different versions are invalid.
6. The structured fields, not the profile ID text alone, establish identity.
7. Core does not claim that `interface`, `client_version`, and `client_build` are historically correct for WoW; `wow-reference` proves that relationship.

## 5. Generation context

### Fields

```text
context_id
profile_identity
reference_generation
project_generation?             # omitted when the operation has no project
external_generations[]          # sorted, explicitly separate
schema_versions[]               # sorted by schema ID
producer_versions[]             # sorted by producer ID
```

Nested version entries are explicit typed values:

```text
SchemaVersionEntry
  schema_id
  version

ProducerVersionEntry
  producer_id
  version
```

Each `ExternalGeneration` contains:

```text
provider_id
scope_id                         # repository/universe/index scope
external_generation_id
source_revision?
```

### Invariants

- Exactly one profile and one reference generation exist.
- At most one project generation exists.
- Two external generation entries cannot share the same `(provider_id, scope_id)` with different generation IDs.
- Every schema/producer ID appears at most once.
- `context_id` is derived from all identity-relevant fields except itself.
- A project generation is meaningful only with the reference generation used to produce it.
- External generations never replace or merge into reference/project generation fields.

### Compatibility modes

Operations use an explicit compatibility mode:

```text
strict
  every present generation field must match exactly; no field filling

extend_missing_optional
  profile/reference must match; an absent optional project field may be filled
  only when the caller explicitly requests context extension

external_union
  profile/reference/project must match; nonconflicting external scopes may be united
```

There is no implicit “best available” mode.

## 6. Source handle

### Fields

```text
handle_id
origin_kind                      # repository | reference_pack | generated_artifact | fixture
origin_id                        # bounded opaque identity owned by the source registry
revision                         # immutable source revision/profile artifact ID
reference_generation?            # generation binding when required by origin kind
project_generation?              # generation binding when required by origin kind
path                             # canonical repository/artifact-relative UTF-8 path
span                             # unknown | whole_file | byte_range
content_digest                   # exact source content digest
entity_key?                      # optional exact symbol/entity association
```

`byte_range` contains only:

```text
byte_start                       # zero-based inclusive
byte_end                         # zero-based exclusive
```

Line/column hints, excerpts, resolved checkout roots, and display aliases are presentation data outside the E0 canonical handle. A transport derives them from exact content when needed.

### Origin/generation matrix

```text
repository
  immutable revision required; generation fields forbidden

reference_pack
  reference_generation required; project_generation forbidden

generated_artifact
  at least one of reference_generation/project_generation required;
  both are allowed for an artifact generated from one project under one reference

fixture
  generation fields optional, but any supplied field must match the containing context
```

The matrix prevents an artifact from being detached from the exact generation that produced it without forcing ordinary repository source to duplicate envelope context.

### Identity projection

`handle_id` includes:

```text
origin_kind
origin_id
revision
reference_generation when present
project_generation when present
path
span state and canonical byte offsets
content_digest
entity_key when present
```

### Invariants

- `origin_id` and `revision` are nonempty, bounded, valid UTF-8, contain no controls or credentials, and cannot be floating aliases where immutability is required. `origin_id` identifies a registered source; it does not itself grant evidence authority.
- `path` is never absolute and never empty for a file handle.
- `byte_end >= byte_start`.
- `unknown`, `whole_file`, and `byte_range` are mutually exclusive states.
- `unknown` span cannot be treated as whole-file proof.
- `content_digest` is required even when span is unknown.
- Generation fields obey the origin matrix and, inside an envelope, match its context.
- A source handle grants no filesystem access and carries no host root.

## 7. Evidence and conflicts

### Provenance classes

```text
platform_source
project_source
runtime_probe
curated_correction
differential_oracle
external_implementation
semantic_candidate
historical_record
model_inference
```

### Confidence levels

```text
proven
derived
possible
candidate
```

### Claim scopes

E0 uses generic claim scopes so authority can be evaluated without embedding WoW algorithms in core:

```text
source_observation
platform_contract
project_fact
runtime_scenario
historical_relation
candidate_relation
```

### Coverage reference

Evidence points to coverage semantically rather than by `CoverageId` to keep the identity graph acyclic:

```text
capability_id
partition_id
producer_id
```

The containing envelope must resolve each triple to exactly one `CoverageRecord` in the same context.

### Evidence record

```text
evidence_id
context_id
provenance
confidence
claim_scope
producer_id
producer_version
source_handle_ids[]
coverage_refs[]                  # exact capability/partition/producer triples
derivation_input_ids[]
```

Validation rules:

- `semantic_candidate` and `model_inference` may only use `candidate` confidence in E0.
- `derived` requires at least one derivation input and a deterministic producer.
- The derivation graph is acyclic; a record cannot directly or transitively depend on itself.
- `proven` cannot use candidate-only inputs as sufficient proof.
- A runtime probe can be `proven` only for its stated scenario scope; higher layers cannot generalize it into a platform contract.
- Every handle, coverage ref, and derivation input resolves in the same context.
- A handle identifies location/content, while provenance and claim scope identify evidentiary role; a finding's primary project location cannot substitute for a separate platform/reference contract source.
- The owning producer/source registry validates that each opaque `origin_id` is eligible for the claimed provenance; core does not infer repository authority from the text of `origin_id`.
- Source excerpts, prose explanations, notes, display labels, and conflict back-references are not fields of the canonical E0 `EvidenceRecord`.
- Combining records preserves each record; it never mutates them into a stronger aggregate record.

### Conflict record

A conflict is a separate relation over immutable evidence records. Evidence does not point back to conflicts, avoiding identity cycles.

```text
conflict_id
context_id
conflict_code                    # stable MessageCode
evidence_ids[]                   # sorted, unique, at least two
affected_refs[]                  # sorted capability + optional partition refs
subject_entity_key?              # optional exact affected entity
```

Affected ref:

```text
capability_id
partition_id?                    # absent means the entire capability in this context
```

E0 conflict records are unresolved by definition. A resolved disagreement is represented by removing the conflict from the new generation and, when applicable, adding digest-bound correction/derivation evidence. Core does not pick a winner.

`conflict_id` includes every field above except itself. Conflict records are context-bound and every evidence ID must resolve exactly once.

## 8. Coverage, capability summaries, and evaluation

### Coverage record

```text
coverage_id
context_id
capability_id
partition_id
status                           # complete | partial | unknown | failed | not_applicable
producer_id
producer_version
missing_input_ids[]
failure_code?                    # required for failed; forbidden otherwise
conflict_ids[]
truncation_refs[]
```

Truncation ref:

```text
collection_id                    # stable lower identifier owned by producer/schema
reason_code                      # stable MessageCode
```

Coverage-record invariants:

- `coverage_id` is derived from every field except itself.
- One producer emits at most one record for one `(context, capability, partition)` key.
- `complete` has no missing inputs or failure code.
- `partial` explains missing inputs and/or truncation.
- `unknown` states why completeness cannot be established through `missing_input_ids` or an owning producer reason.
- `failed` requires `failure_code`.
- `not_applicable` has no missing inputs, failure code, conflicts, or truncation.
- Every conflict ID resolves to a conflict affecting this capability/partition.

### Capability summary

`combine_coverage` derives one summary from exact coverage records:

```text
context_id
capability_id
producer_id                      # producer of the summary, normally service/orchestrator
producer_version
status                           # complete | partial | unknown | failed | not_applicable
partition_refs[]                 # CoveragePartitionRef values

CoveragePartitionRef
  coverage_id
  partition_id
  status
conflict_ids[]
truncation_refs[]
```

The summary is not a replacement for its input records. It exposes the conservative effective state and exact blocking inputs.

### Evaluation record

```text
not_evaluated_id
context_id
producer_id
producer_version
subject_kind                     # rule | operation | lane
subject_id
reason_code
blocking_capability_ids[]
blocking_partitions[]            # BlockingPartitionRef values

BlockingPartitionRef
  capability_id
  coverage_id
  partition_id
  status
conflict_ids[]
```

The shared vocabulary includes `not_evaluated`, but source/index partitions do not use it as ingestion coverage. It is the disposition of a requested subject after capability evaluation.

### Conservative coverage order

For applicable required partitions:

```text
failed > unknown > partial > complete
```

`not_applicable` partitions are ignored when at least one required partition is applicable. If every required partition is `not_applicable`, the combined result is `not_applicable`.

An unresolved affecting conflict denies negative authority independently of source coverage status. Truncation is likewise preserved separately even when a summary status is already partial/failed.

## 9. Negative-authority decision

### Outcomes

```text
authoritative_absent
not_authoritative
not_applicable
```

`not_authoritative` includes one or more typed reasons:

```text
profile_unavailable
reference_generation_unavailable
generation_mismatch
partition_partial
partition_unknown
partition_failed
unresolved_conflict
capability_not_evaluated
candidate_only_evidence
scope_unknown
result_truncated
```

The decision records the exact capabilities, partitions, conflicts, and context used. A caller cannot replace this object with an unqualified boolean.

## 10. Structured finding

### Fields

```text
finding_id
fingerprint
context_id
rule_id
rule_version
finding_code
severity                         # error | warning | information | hint
policy                           # shadow | advisory | blocking
subject_entity_key?
primary_source_handle_id
related_source_handle_ids[]
evidence_ids[]
required_capability_ids[]
coverage_status
message_arguments[]
root_cause_key?
caused_by_root_cause_key?
remediation?
```

Message argument entry:

```text
name                             # lower segment
kind                             # text | integer | boolean | identifier | path | digest
value                            # canonical representation for kind
identity_relevant                # explicit boolean
```

Remediation entry:

```text
class                            # exact_edit | validated_recipe | plan_only | candidate_only
recipe_id?                       # stable ID owned by rule/remediation layer
plan_handle_id?                  # optional stable handle
```

### Finding identity

`FindingFingerprint` includes:

```text
rule_id and rule_version
finding_code
subject entity when present
primary source handle ID
identity-relevant message arguments
root-cause semantic key when present
```

It excludes:

```text
context_id
severity/policy overrides
rendered message text
nonidentity message arguments
notes and display labels
```

`FindingId` is derived from `context_id + fingerprint`.

### Invariants

- Every finding belongs to the envelope context.
- `primary_source_handle_id` identifies the reported location, not the authority source for every cited claim.
- Every referenced handle/evidence/root cause exists in the envelope or an explicitly linked registry.
- Severity does not imply rollout policy.
- Candidate evidence cannot authorize an `exact_edit` remediation.
- Missing required capabilities produce an evaluation record instead of a finding claiming clean or broken state.

## 11. Warnings and operation-error envelope

### Warning record

A warning is a structured operation-level condition that is neither a code diagnostic nor `NotEvaluated`, for example an optional lane being unavailable while required local lanes completed.

```text
warning_id
context_id
producer_id
producer_version
warning_code
subject_kind?
subject_id?
primary_source_handle_id?
related_source_handle_ids[]
evidence_ids[]
message_arguments[]
```

`warning_id` includes every semantic field above except itself. It excludes rendered prose because prose is not part of the E0 canonical envelope. All referenced handles/evidence resolve in the same context.

### Operation-error envelope

An operation error follows [`ERROR_MODEL.md`](ERROR_MODEL.md) and is returned when a boundary contract cannot be satisfied. Errors are outside findings, warnings, and `NotEvaluated` arrays.

The strict E0 operation-error envelope contains:

```text
schema
canonicalization_version
operation_id
error
canonical_digest
```

It carries no analysis records and cannot masquerade as a partial check result. Its canonical digest uses the result-domain profile over the envelope without `canonical_digest`.

## 12. Budget and truncation

### Budget limits

E0 recognizes:

```text
max_coverage_records
max_capability_summaries
max_source_handles
max_evidence_records
max_conflicts
max_findings
max_not_evaluated
max_warnings
max_output_bytes
```

A budget value must be positive and within the configured implementation maximum.

### Usage

```text
coverage_records
capability_summaries
source_handles
evidence_records
conflicts
findings
not_evaluated
warnings
output_bytes
```

All arithmetic is checked. Usage exactly matches the final canonical collections and bytes.

### Truncation state

```text
not_truncated
truncated
```

A `truncated` state contains sorted affected entries:

```text
collection_id
capability_ids[]
omitted_count?                   # nonnegative exact count when known
count_unknown                    # explicit boolean
reason_code
```

Silent clipping is invalid. Producers decide what can be omitted under their own correctness contracts; core only validates/records the result. Affecting truncation makes envelope status at least `partial` and denies negative authority for affected scopes.

## 13. E0 check result envelope

The E0 canonical check envelope contains:

```text
schema
canonicalization_version
operation_id = wow.check
context
status                           # complete | partial | failed
coverage_records[]
capability_summaries[]
source_handles[]
evidence_records[]
conflicts[]
findings[]
not_evaluated[]
warnings[]
budget
canonical_digest
```

### Status meaning

- `complete`: every requested required E0 lane evaluated and no affecting conflict/truncation/optional failure remains; findings may exist.
- `partial`: useful validated results exist, but at least one requested lane is `NotEvaluated`, an optional lane failed/staled, or an affecting conflict/truncation remains.
- `failed`: no valid requested result can be returned under the required context.

A nonempty findings array does not make an operation partial or failed.

### Reference layering

The canonical reference graph is acyclic by construction:

```text
source handles
  -> evidence (handle IDs + semantic coverage refs)
  -> conflicts (evidence IDs)
  -> coverage records (conflict IDs)
  -> capability summaries / NotEvaluated (coverage IDs + conflict IDs)
  -> findings / warnings (handle/evidence IDs)
  -> envelope
```

Derived evidence may reference earlier evidence arbitrarily, but the derivation graph must remain acyclic. Envelope validation resolves every internal reference and rejects duplicate IDs with differing content.

## 14. Unknown-field policy for E0

The internal E0 envelope is strict:

- unknown required or top-level fields are rejected;
- duplicate fields are rejected;
- unknown enum variants are rejected with `schema_version_unsupported` or `contract_violation`;
- fields are never silently ignored;
- free-form notes, line hints, rendered messages, and source excerpts are not canonical E0 fields and belong to transport presentation outside this envelope;
- public forward-compatible extension points are deferred until a versioned external schema exists.

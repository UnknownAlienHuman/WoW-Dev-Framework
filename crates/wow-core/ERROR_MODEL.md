# `wow-core` error model

**Status:** normative E0-A boundary-error contract; no Rust code yet.

Errors report that an operation could not accept or preserve its contract. They are not code diagnostics and are not `NotEvaluated` records.

## 1. Error record

Every error contains:

```text
code                             # stable lowercase snake-case code
category                         # validation | mismatch | unsupported | budget | invariant
operation_id                     # operation that failed
field_path?                      # safe schema field path
subject_kind?                    # identifier | profile | context | handle | evidence | conflict | coverage | evaluation | finding | warning | budget | schema | envelope
subject_id?                      # canonical safe ID when available
reason_arguments[]               # typed, sorted, nonsecret structured arguments
retry_class                      # never | after_input_change | after_dependency_recovery
cause_codes[]                    # optional stable nested error codes
```

Optional human prose is a transport projection and is not required to interpret the error.

## 2. Safety rules

Default error data must not contain:

- source file contents or excerpts;
- absolute host paths;
- credentials, tokens, environment variables, or repository secrets;
- arbitrary untrusted comments/documentation;
- full malformed input when a bounded field/path/reason is sufficient;
- localized prose as the only reason.

When reporting an unsafe candidate string, return a bounded length, character class, field path, and optional digest rather than echoing the entire value.

## 3. Retry classes

| Retry class | Meaning |
|---|---|
| `never` | Repeating the same operation with the same inputs cannot succeed. |
| `after_input_change` | Caller must correct identifiers, context, schema, budget, or records. |
| `after_dependency_recovery` | A higher layer may retry after an unavailable component/partition is repaired; core itself performs no retry. |

No automatic retry is performed by `wow-core`.

## 4. Error catalog

### Identifier and digest

| Code | Category | Trigger | Retry |
|---|---|---|---|
| `invalid_identifier` | validation | Family grammar, character, or segment violation. | after_input_change |
| `reserved_identifier_segment` | validation | Floating/reserved whole segment used as stable identity. | after_input_change |
| `identifier_too_long` | validation | Family maximum exceeded. | after_input_change |
| `invalid_entity_key` | validation | Empty/invalid exact key payload. | after_input_change |
| `noncanonical_percent_encoding` | validation | Invalid or unnecessarily encoded key bytes. | after_input_change |
| `invalid_digest` | validation | Malformed/truncated digest. | after_input_change |
| `unsupported_digest_algorithm` | unsupported | Algorithm other than E0 SHA-256. | after_input_change |
| `unsupported_identifier_family` | unsupported | Requested derived-ID family/domain is not part of E0. | after_input_change |
| `digest_purpose_mismatch` | mismatch | Digest used under the wrong typed purpose. | after_input_change |
| `digest_mismatch` | mismatch | Supplied content/identity digest differs. | after_input_change |

### Profile and generation

| Code | Category | Trigger | Retry |
|---|---|---|---|
| `invalid_profile_identity` | validation | Missing, duplicate, malformed, or internally inconsistent fields. | after_input_change |
| `profile_kind_violation` | validation | Fixture/release requirements mixed. | after_input_change |
| `profile_mismatch` | mismatch | Structured profiles differ where exact match is required. | after_input_change |
| `generation_mismatch` | mismatch | Reference/project/external context differs. | after_input_change |
| `duplicate_external_generation_scope` | validation | Same provider/scope appears with conflicting generation. | after_input_change |
| `merge_mode_violation` | validation | Requested context merge is not allowed by explicit mode. | after_input_change |
| `duplicate_schema_id` | validation | Schema ID appears more than once. | after_input_change |
| `duplicate_producer_id` | validation | Producer/tool ID appears more than once. | after_input_change |

Generation-context admission propagates nested profile, provider and text-bound
validation errors rather than replacing them with an ID mismatch. The strict
same-generation guard validates both operands, including an operand compared to
itself. Merge inputs must already be valid; schema/producer inventory differences
(including subsets and conflicting versions) return `merge_mode_violation` on the
affected collection. Duplicate IDs inside one input retain their duplicate codes.

### Source handles

| Code | Category | Trigger | Retry |
|---|---|---|---|
| `invalid_source_path` | validation | Empty, control-containing, malformed path. | after_input_change |
| `path_escape` | validation | `..` or equivalent escape component. | after_input_change |
| `absolute_path_forbidden` | validation | Rooted/drive/UNC/device/URI host path. | after_input_change |
| `unsupported_non_utf8_path` | unsupported | E0 public path cannot be represented losslessly as UTF-8. | after_input_change |
| `invalid_source_span` | validation | Invalid byte offsets or span state. | after_input_change |
| `span_state_conflict` | validation | Unknown/whole-file/range fields mixed. | after_input_change |
| `invalid_source_handle` | validation | Cross-field origin/revision/generation violation. | after_input_change |
| `missing_source_handle` | mismatch | Referenced handle absent from containing registry. | after_input_change |

### Evidence and conflict

| Code | Category | Trigger | Retry |
|---|---|---|---|
| `evidence_authority_violation` | invariant | Candidate-only provenance claims stronger authority or unauthorized remediation. | after_input_change |
| `evidence_context_mismatch` | mismatch | Evidence context differs from envelope. | after_input_change |
| `derived_evidence_missing_inputs` | validation | `derived` record has no derivation inputs. | after_input_change |
| `evidence_derivation_cycle` | invariant | Evidence derivation graph contains a direct or transitive cycle. | after_input_change |
| `missing_evidence_reference` | mismatch | Referenced evidence record absent. | after_input_change |
| `duplicate_evidence_reference` | validation | Duplicate evidence/input/conflict reference. | after_input_change |
| `conflict_context_mismatch` | mismatch | Conflict/evidence records do not share one context. | after_input_change |
| `conflict_scope_empty` | validation | Conflict has no affected capability/partition scope. | after_input_change |
| `missing_conflict_reference` | mismatch | Referenced conflict record is absent. | after_input_change |
| `duplicate_conflict_reference` | validation | Duplicate conflict reference appears where uniqueness is required. | after_input_change |
| `coverage_conflict` | mismatch | Duplicate, irreconcilable, or summary-inconsistent coverage records. | after_input_change |

Evidence-DAG admission returns `evidence_context_mismatch` for mixed registries,
including disconnected components. Missing inputs, confidence escalation and
runtime-to-platform ancestry have distinct reference/authority errors. Cyclic wire
IDs may yield `evidence_derivation_cycle` before a digest error; no successful
validation skips the ID checks. `duplicate_evidence_reference` covers noncanonical
source/input/evidence arrays; `duplicate_coverage_record` covers noncanonical
semantic coverage refs. Conflict affected-scope duplicates/order violations retain
the constructor's `conflict_scope_empty` error. Diagnostics name fixed fields;
they do not echo evidence contents or source text.

### Coverage and authority

| Code | Category | Trigger | Retry |
|---|---|---|---|
| `coverage_record_missing` | mismatch | Required capability/partition/producer record absent. | after_dependency_recovery |
| `coverage_context_mismatch` | mismatch | Coverage record/summary context differs from the operation context. | after_input_change |
| `duplicate_coverage_record` | validation | Same unique coverage key appears twice. | after_input_change |
| `negative_authority_unavailable` | mismatch | Caller requested authoritative absence but prerequisites fail. | after_dependency_recovery |

`negative_authority_unavailable` is appropriate only when the caller demanded authority as a precondition. Ordinary queries should return a typed `NegativeAuthorityDecision` rather than throw this error.

### Findings and envelope

| Code | Category | Trigger | Retry |
|---|---|---|---|
| `invalid_message_argument` | validation | Unknown kind, duplicate name, malformed canonical value. | after_input_change |
| `finding_context_mismatch` | mismatch | Finding context differs from envelope. | after_input_change |
| `warning_context_mismatch` | mismatch | Warning context differs from envelope. | after_input_change |
| `remediation_authority_violation` | invariant | Candidate/insufficient evidence authorizes exact edit. | after_input_change |
| `result_duplicate_id` | invariant | Same ID appears with different record content. | after_input_change |
| `result_context_violation` | mismatch | Envelope entries do not share one coherent context. | after_input_change |
| `result_status_violation` | invariant | `complete/partial/failed` disagrees with contained state. | after_input_change |
| `result_reference_violation` | mismatch | Internal handle/evidence/root-cause reference is unresolved. | after_input_change |
| `canonicalization_failure` | invariant | Valid semantic value cannot be encoded by canonical profile. | after_input_change |
| `canonical_digest_mismatch` | mismatch | Supplied result digest differs from recomputation. | after_input_change |
| `contract_violation` | invariant | Catch-all for an impossible state not covered by a narrower stable code. | never |

`contract_violation` must remain rare. Repeated use requires a new narrower code and test.

### Budget and schema

| Code | Category | Trigger | Retry |
|---|---|---|---|
| `budget_invalid` | validation | Zero, overflow, unknown dimension, or above implementation maximum. | after_input_change |
| `budget_exceeded` | budget | Producer cannot return requested result within explicit budget without typed truncation. | after_input_change |
| `usage_overflow` | invariant | Checked usage arithmetic overflow. | never |
| `schema_version_unsupported` | unsupported | Unknown major/version/required field/enum variant. | after_input_change |
| `unknown_field` | unsupported | Strict E0 object contains an undeclared field. | after_input_change |
| `duplicate_field` | validation | Serialized object contains duplicate key. | after_input_change |

## 5. Operation-to-error expectations

| Operation family | Normal errors |
|---|---|
| ID/digest parsing | identifier/digest validation codes only |
| Profile validation | profile, schema, producer, digest validation codes |
| Source-handle construction | path/span/digest/origin/context validation codes |
| Context merge | profile/generation/merge conflict codes |
| Evidence validation | authority/context/reference/derivation codes |
| Coverage aggregation | missing/duplicate/conflict codes |
| Finding binding | context/reference/remediation codes |
| Envelope validation/finalization | any contained narrow error plus result/schema/canonicalization codes |

A low-level parser must not return a service-level profile-unavailable or optional-lane error.

## 6. Error versus `NotEvaluated`

Use an **error** when:

- input is malformed;
- contexts contradict;
- a supplied record violates its own declared contract;
- serialization/schema is unsupported;
- requested strict finalization cannot be performed.

Use **`NotEvaluated`** when:

- a valid rule/operation intentionally does not run because required capabilities/partitions are absent, partial, unknown, failed, or conflicting.

Example:

```text
Missing coverage record while validating an allegedly complete envelope
  -> coverage_record_missing error

Known Partial coverage prevents wow.secret.local_operation from running
  -> valid NotEvaluated record
```

## 7. Error versus finding

An invalid source handle supplied by a caller is an error. A valid handle pointing to source where an API is absent may support a finding produced by `wow-rules`.

Core never turns malformed analysis data into an addon diagnostic.

## 8. Aggregation rules

When validating a composite envelope:

- retain all independent safe errors when bounded collection is possible;
- preserve causal nesting using `cause_codes` rather than concatenated prose;
- order errors by `field_path`, then code, then subject ID;
- cap reported errors using an explicit validation budget;
- if error collection is truncated, report that truncation; do not pretend the listed set is exhaustive.

E0 may stop at the first error in low-level constructors, but envelope validation should support bounded multi-error reporting when practical.

## 9. Transport projection

Applications may map errors to CLI exit codes, MCP errors, or LSP responses. `wow-core` does not define HTTP status codes, process exit codes, localization, or retry scheduling.

The stable error `code`, `category`, and structured arguments remain unchanged across transports.


## Checked coverage consumers

`evaluate_capability_availability` and `evaluate_negative_authority` propagate
record validation failures before any clean result. Empty/missing coverage uses
`coverage_record_missing`; duplicate logical record/summary keys use
`duplicate_coverage_record`; contradictory or omitted coverage/conflict scope uses
`coverage_conflict`; mixed records/summaries use `coverage_context_mismatch`;
mixed conflicts use `conflict_context_mismatch`. Missing or duplicate conflict IDs
use `missing_conflict_reference` / `duplicate_conflict_reference` respectively.
A foreign evaluation context uses `result_context_violation`. Malformed decoded
record metadata uses the same narrow identifier errors as construction; invalid
record identities use `canonical_digest_mismatch`. Valid partial/unknown/failed
coverage is not an operation error and still produces an explicit blocked result.

## Truncation admission detail

Malformed retained truncation propagates through budget validation, byte-count
replacement, envelope finalization and the negative-authority consumer.
`contract_violation` identifies empty/noncanonical/duplicate outer entries,
noncanonical/duplicate capability IDs or contradictory known/unknown counts at
`truncation.entries`, `entries.capability_ids`, or `entries.omitted_count`.
Collection-name failures retain the existing `invalid_identifier`,
`identifier_too_long`, or `reserved_identifier_segment` code at
`entries.collection_id`. These are existing validation codes, not new enums.
No raw offending collection string is echoed. Strict Serde field/type failures
remain decoding errors; this slice does not add a host error-normalization API.

## Finding/warning admission detail

Malformed diagnostic references return `result_duplicate_id` at
`related_source_handle_ids`, `evidence_ids` or `required_capability_ids` with a
fixed noncanonical/duplicate reason. A duplicated source registry uses
`result_duplicate_id` at `source_handles`; duplicate evidence IDs propagate
`duplicate_evidence_reference`. Foreign evidence uses `evidence_context_mismatch`
at `evidence_records.context_id`. Existing source/evidence validators propagate
their narrow shape, digest and ancestry errors before a diagnostic is accepted.
Missing evidence source links use `missing_source_handle` at
`evidence_records.source_handle_ids`; missing remediation plans use the same code
at `remediation.plan_handle_id`. Class/recipe failures retain
`remediation_authority_violation`. Incomplete warning subject pairs use
`invalid_message_argument` at `subject`; malformed subject values retain the
constructor's identifier/text error and safe field path. Noncanonical integer/path
arguments use `invalid_message_argument` at `arguments.value`. Raw offending data
is not echoed. No new error enum, silent repair or schema version is introduced.

## Coverage/evaluation joins in result envelopes

Envelope coverage admission propagates the existing narrow coverage errors:
`duplicate_coverage_record` for logical statement/summary-owner collisions,
`coverage_conflict` for incomplete summaries or inaccurate conflict projections,
and `coverage_record_missing` / `missing_conflict_reference` for unresolved joins.
`NotEvaluated` local shape uses `blocking_partitions`,
`blocking_partitions.capability_id` and `blocking_partitions.conflict_ids` field
paths. Retained evaluation joins use `evaluation.blocking_partitions` and
`evaluation.conflict_ids`, including when propagated through the envelope.
A well-formed blocker on healthy complete coverage is `coverage_conflict`, not a
valid denial receipt. A wrong self-ID still returns `canonical_digest_mismatch`.
No raw subject, source text or caller-provided identifier is echoed by these checks.

## Canonical serializer errors

Repeated object keys emitted by a Serialize implementation return `duplicate_field`
(category `validation`, operation `canonical_json`, retry `after_input_change`).
The key and its values are not echoed. This applies to nested maps, declared or
flattened struct fields and keys colliding after JSON spelling conversion.

`canonicalization_failure` remains the code for null/negative/floating/oversized
numeric values, malformed map emission, invalid keys and unsupported raw JSON.
A custom serializer error receives a fixed safe reason rather than copying the
upstream diagnostic. These checks do not turn errors into findings/NotEvaluated,
add automatic retry, or certify general secret redaction or host JSON admission.

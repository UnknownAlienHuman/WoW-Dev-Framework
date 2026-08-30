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

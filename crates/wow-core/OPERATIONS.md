# `wow-core` operations

**Status:** normative E0-A operation inventory; no Rust code yet.

The notation below is language-neutral:

```text
operation(input) -> success | typed error
```

Every operation is pure. Inputs are already in memory. No operation reads a file, resolves a repository, obtains the time, consults a registry over the network, or chooses a profile implicitly.

## 1. Identifier and digest operations

### `parse_profile_id`

```text
input:
  candidate string

success:
  canonical ProfileId
  was_canonical boolean

errors:
  invalid_identifier
  reserved_identifier_segment
  identifier_too_long
```

Rules:

1. Reject leading/trailing whitespace; do not trim it silently.
2. Require `profile:<namespace>:<slug>`.
3. ASCII uppercase in namespace/slug may be converted to lowercase, but `was_canonical = false`.
4. Reject empty segments, controls, Unicode lookalikes, unsupported separators, and reserved whole segments.
5. Return no profile metadata; parsing a label does not prove a `ProfileIdentity`.

Required tests: `ID-PROFILE-001..012`.

### `parse_rule_id`, `parse_producer_id`, `parse_capability_id`, `parse_operation_id`

```text
input:
  candidate string

success:
  family-specific dotted ID
  was_canonical boolean

errors:
  invalid_identifier
  reserved_identifier_segment
  identifier_too_long
```

Rules:

- Require at least two lower-case dotted segments.
- Allow underscore inside a segment.
- Reject empty segments, consecutive dots, leading digits, and hidden whitespace.
- These operations do not reinterpret one ID family as another even when text matches.

Required tests: `ID-DOTTED-001..014`.

### `parse_entity_key`

```text
input:
  kind candidate
  exact key payload

success:
  structured EntityKey
  canonical string projection

errors:
  invalid_identifier
  invalid_entity_key
  noncanonical_percent_encoding
```

Rules:

- Validate `kind` as a qualified lower-case ID with one or more dotted segments; single-segment kinds such as `api` are valid.
- Preserve exact key case and UTF-8 text.
- Reject empty key, control characters, invalid UTF-8, and malformed/noncanonical percent encoding.
- Canonical encoding follows `DATA_MODEL.md`.

Required tests: `ID-ENTITY-001..012`.

### `parse_coverage_partition_id`

Same rules as `parse_entity_key`, except the exact key may be absent. The distinction between no key and an empty key is not allowed; empty key input is invalid.

Required tests: `ID-PARTITION-001..010`.

### `parse_content_digest`

```text
input:
  candidate string
  expected purpose

success:
  typed ContentDigest
  was_canonical boolean

errors:
  invalid_digest
  unsupported_digest_algorithm
  digest_purpose_mismatch
```

Rules:

1. E0 accepts `sha256:<64 hex>` only.
2. Uppercase hex may be canonicalized to lowercase with `was_canonical = false`.
3. Truncated, bare, prefixed Git, base64, or whitespace-containing digests are rejected.
4. Digest purpose is supplied by the owning field; it is not inferred from input text.

Required tests: `DIGEST-001..012`.

### `derive_typed_digest_id`

```text
input:
  ID family tag
  canonical material bytes

success:
  full type-tagged SHA-256 identifier

errors:
  canonicalization_failure
  unsupported_identifier_family
```

Rules:

- Hash exactly the canonical material bytes.
- Prepend the family tag only after hashing; the material itself includes an explicit domain-separation record.
- Never accept an already hashed display prefix as material.
- Domain separation prevents identical bytes in different identity families from being interchangeable.

Required tests: `DIGEST-DOMAIN-001..008` and `examples/HASH_VECTORS.json`.

## 2. Profile operations

### `validate_profile_identity`

```text
input:
  ProfileIdentity

success:
  validated ProfileIdentity

errors:
  invalid_profile_identity
  profile_kind_violation
  invalid_identifier
  invalid_digest
  duplicate_schema_id
```

Checks:

- required fields by fixture/release kind;
- positive Interface/build values when present;
- source kind and profile kind compatibility;
- nonfloating release revision;
- valid/sorted/unique schema versions;
- builder/correction-set requirements for release;
- fixture scope requirement for fixture;
- E0 namespace/kind consistency: `profile:fixture:*` is fixture and `profile:wow:*` is release; other namespaces are unsupported.

Non-checks:

- whether Blizzard actually paired the Interface and build;
- whether a revision exists remotely;
- whether the supplied digest matches source bytes;
- whether the profile represents the newest live game state.

Those belong to `wow-reference` and explicit content verification.

Required tests: `PROFILE-001..020`.

### `compare_profile_identity`

```text
input:
  validated left identity
  validated right identity

success:
  identical
  same_label_different_identity with field differences
  different_label_same_material
  different
```

Rules:

- Compare all identity-relevant structured fields.
- Do not treat equal `ProfileId` as equal identity.
- `different_label_same_material` is reported for migration/alias review; it is not automatically accepted as compatible.

Required tests: `PROFILE-COMPARE-001..008`.

### `require_profile_identity_match`

Returns success only for fully identical validated structured identity. Any difference returns `profile_mismatch` with safe field paths; malformed inputs are rejected by `validate_profile_identity` before this operation.

Required tests: `PROFILE-REQUIRE-001..006`.

## 3. Source-handle operations

### `normalize_source_path`

```text
input:
  candidate UTF-8 path string

success:
  canonical repository-relative path
  was_canonical boolean

errors:
  invalid_source_path
  path_escape
  absolute_path_forbidden
  unsupported_non_utf8_path
```

Algorithm:

1. Reject NUL/control characters.
2. Treat `\` as a separator and replace it with `/`; E0 cannot represent a literal backslash filename distinctly and documents that path class as unsupported.
3. Reject drive, device, UNC, URI-like, and leading-root forms.
4. Split by `/`.
5. Remove empty and `.` components.
6. Reject any `..` component.
7. Rejoin with `/` and preserve component case/text exactly.
8. Reject an empty result.

Core does not resolve symlinks or test path existence.

Required tests: `PATH-001..024`.

### `validate_source_span`

```text
input:
  span state and values

success:
  validated canonical span

errors:
  invalid_source_span
  span_state_conflict
```

Rules:

- `unknown` contains no offsets.
- `whole_file` contains no offsets.
- `byte_range` requires nonnegative `start` and `end`, with `end >= start`.
- Line/column hints are not fields of the E0 canonical span; transports derive them from exact content.

Required tests: `SPAN-001..016`.

### `build_source_handle`

```text
input:
  origin
  immutable revision
  optional reference generation
  optional project generation
  candidate path
  candidate span
  source content digest
  optional EntityKey

success:
  validated SourceHandle with derived StableHandleId

errors:
  invalid_source_handle
  invalid_identifier
  invalid_source_path
  invalid_source_span
  invalid_digest
```

Rules:

- Normalize/validate each field first.
- Enforce the origin/generation matrix in `DATA_MODEL.md`.
- Forbid host roots, checkout paths, URLs containing credentials, and mutable branch-only revisions.
- Envelope validation separately checks any supplied generation fields against the containing context.
- Derive handle ID from the identity projection in `DATA_MODEL.md`.
- Rebuilding from equivalent logical input yields the same handle ID.

Required tests: `HANDLE-001..020`.

### `verify_source_handle_content`

```text
input:
  validated SourceHandle
  supplied source content digest

success:
  verified

errors:
  digest_mismatch
  digest_purpose_mismatch
```

This operation compares digests only. It never reads source content or a filesystem.

Required tests: `HANDLE-VERIFY-001..006`.

### `compare_source_handles`

Returns:

```text
identical
same_file_different_span
same_origin_path_different_revision
same_origin_revision_path_different_content
unrelated
```

No category implies automatic lineage or replacement.

Required tests: `HANDLE-COMPARE-001..010`.

## 4. Generation-context operations

### `derive_generation_context_id`

```text
input:
  validated context without context_id

success:
  GenerationContextId

errors:
  invalid_profile_identity
  generation_mismatch
  duplicate_external_generation_scope
  canonicalization_failure
```

Rules:

- Canonicalize all set-like lists.
- Include profile identity, reference generation, optional project generation, external generations, schema versions, and producer versions.
- Exclude display aliases and notes.

Required tests: `CONTEXT-ID-001..008`.

### `validate_generation_context`

Checks:

- one valid profile/reference generation;
- at most one project generation;
- unique external `(provider, scope)` entries;
- unique schema/producer IDs;
- canonical ordering;
- supplied context ID matches derived ID.

Returns structured errors rather than rewriting mismatched IDs.

Required tests: `CONTEXT-001..018`.

### `merge_generation_context`

```text
input:
  validated left context
  validated right context
  explicit merge mode

success:
  validated merged context with newly derived context ID

errors:
  generation_mismatch
  profile_mismatch
  duplicate_external_generation_scope
  merge_mode_violation
```

Rules by mode:

- `strict`: every present identity field and collection must be identical.
- `extend_missing_optional`: profile/reference must match; an absent optional project field may be filled, but conflicting project values fail.
- `external_union`: profile/reference/project must match; nonconflicting external scopes may be combined.
- Schema or producer version conflicts always fail.

Required tests: full matrix `CONTEXT-MERGE-001..028`.

### `require_same_generation`

Strict guard for multi-input operations. It returns success only when context IDs match exactly. It does not call a permissive merge mode.

Required tests: `CONTEXT-SAME-001..006`.

## 5. Evidence operations

### `validate_evidence_record`

Checks:

- record context equals the containing context;
- provenance/confidence/claim-scope combinations obey E0 authority rules;
- `derived` has nonempty unique derivation inputs;
- candidate-only provenance is not `proven`/`derived`;
- referenced source handles, semantic coverage refs, and input evidence records resolve exactly once;
- source/evidence/coverage-ref arrays are canonically sorted;
- supplied evidence ID equals the canonical identity projection;
- no conflict back-reference, note, rendered prose, or source excerpt is present.

Errors:

```text
evidence_authority_violation
evidence_context_mismatch
missing_source_handle
coverage_record_missing
missing_evidence_reference
derived_evidence_missing_inputs
duplicate_evidence_reference
```

Required tests: `EVIDENCE-001..026`.

### `derive_evidence_id`

Hash the canonical evidence identity projection. E0 evidence contains semantic data only; prose and transport presentation are not accepted fields.

Required tests: `EVIDENCE-ID-001..008`.

### `validate_evidence_derivation_graph`

```text
input:
  complete evidence registry for one context

success:
  acyclic derivation graph

errors:
  missing_evidence_reference
  evidence_derivation_cycle
```

Every `derivation_input_id` is an edge from the derived record to an existing input. The graph must be acyclic. Validation does not require evidence arrays to be topologically sorted because canonical order is by ID.

Required tests: `EVIDENCE-GRAPH-001..012`.

### `relate_evidence_conflict`

```text
input:
  one context
  two or more existing evidence IDs
  conflict code
  nonempty affected capability/partition refs
  optional subject entity

success:
  canonical unresolved ConflictRecord

errors:
  missing_evidence_reference
  conflict_context_mismatch
  conflict_scope_empty
  duplicate_evidence_reference
```

This operation records a conflict. It does not choose a winner, rewrite confidence, mutate evidence, or apply a correction.

Required tests: `EVIDENCE-CONFLICT-001..014`.

### `derive_conflict_id`

Hash the complete canonical conflict projection except `conflict_id` itself.

Required tests: `CONFLICT-ID-001..008`.

### `validate_conflict_record`

Checks context, code, at least two unique evidence IDs, nonempty unique affected refs, optional subject entity, reference resolution, canonical order, and supplied ID.

Errors:

```text
conflict_context_mismatch
conflict_scope_empty
missing_evidence_reference
duplicate_evidence_reference
canonicalization_failure
```

Required tests: `CONFLICT-VALIDATE-001..016`.

### `derive_evidence`

```text
input:
  deterministic producer ID/version
  claim scope
  output provenance/confidence = derived
  nonempty input evidence IDs
  source/semantic coverage refs

success:
  new derived EvidenceRecord

errors:
  derived_evidence_missing_inputs
  evidence_authority_violation
  evidence_context_mismatch
  missing_evidence_reference
  evidence_derivation_cycle
```

A plain merge of evidence is not derivation. The producer must state the deterministic rule/process that created the new conclusion. The new record is validated against the complete evidence registry before publication.

Required tests: `EVIDENCE-DERIVE-001..016`.


## 6. Coverage and negative-authority operations

### `validate_coverage_record`

Checks valid context/IDs, producer identity, status-specific fields, exact unique conflict references, sorted unique missing inputs/truncation refs, and supplied `CoverageId`.

Examples:

- `complete` cannot carry missing inputs, failure code, or truncation.
- `partial` identifies missing inputs and/or truncation.
- `failed` requires a stable failure code.
- `not_applicable` cannot hide unknown relevant input and carries no blockers.
- every conflict ID must resolve and affect the same capability/partition.

Errors:

```text
coverage_conflict
missing_conflict_reference
duplicate_conflict_reference
invalid_identifier
canonicalization_failure
```

Required tests: `COVERAGE-VALIDATE-001..020`.

### `derive_coverage_id`

Hash every canonical coverage-record field except `coverage_id`.

Required tests: `COVERAGE-ID-001..008`.

### `combine_coverage`

```text
input:
  one context
  requested capability
  nonempty required partition set
  exactly selected validated coverage records
  summary producer ID/version

success:
  CapabilitySummary retaining exact CoverageIds/blockers

errors:
  coverage_record_missing
  coverage_conflict
  duplicate_coverage_record
  result_context_violation
```

Algorithm:

1. Select exactly one record for each required `(capability, partition, producer)` statement requested by the caller.
2. If no required partition is applicable, return `not_applicable`.
3. Ignore `not_applicable` only when another required partition is applicable.
4. Among applicable partitions, use precedence `failed > unknown > partial > complete`.
5. Preserve every partition ref, affecting conflict ID, and truncation ref.
6. Do not return `not_evaluated`; that is produced when a subject consumes summaries.
7. Never drop the underlying coverage records from the envelope.

Required tests include `COVERAGE-COMBINE-001..034`.

### `validate_capability_summary`

Recompute the summary from its referenced coverage records and compare context, producer, status, partition refs, conflicts, and truncation. A summary cannot introduce authority not present in its records.

Required tests: `CAPABILITY-SUMMARY-001..014`.

### `evaluate_capability_availability`

```text
input:
  context and evaluation producer
  subject kind/id
  required capability policies
  capability summaries + referenced coverage records/conflicts

success:
  runnable
  or canonical NotEvaluatedRecord

errors:
  coverage_record_missing
  missing_conflict_reference
  result_context_violation
```

Rules:

- Required `failed`, `unknown`, or `partial` summaries block a subject that requires complete coverage.
- `not_applicable` blocks when the capability is required for that subject.
- Affecting unresolved conflicts block even when source coverage is `complete`.
- Return exact blocking capabilities, blocking coverage IDs/partitions, and conflict IDs.
- Never return a clean evaluated result when capability proof is absent.

Required tests: `CAPABILITY-001..024`.

### `derive_not_evaluated_id`

Hash context, producer, subject, reason, blocking capabilities, blocking partitions/coverage IDs, and conflicts.

Required tests: `NOT-EVALUATED-ID-001..008`.

### `validate_not_evaluated_record`

Checks context/producer, subject identity, reason-specific required fields, exact coverage/conflict references, canonical order, and supplied ID.

Required tests: `NOT-EVALUATED-001..016`.

### `evaluate_negative_authority`

```text
input:
  validated context
  requested scope/entity kind
  exact lookup outcome
  required capability summaries and records
  affecting conflicts
  candidate-only evidence IDs
  evaluation disposition
  truncation state

success:
  NegativeAuthorityDecision

errors:
  result_context_violation
  coverage_record_missing
  missing_conflict_reference
```

Authoritative absence requires:

```text
known exact scope
matching context
exact lookup completed
all required applicable coverage complete
no affecting conflict
no candidate-only substitution
no affecting truncation
subject was evaluated
```

Every denial reason and exact blocker is returned; do not stop at the first reason when the complete bounded safe reason set is available.

Required tests: `NEGATIVE-001..032`.


## 7. Finding operations

### `validate_message_arguments`

Checks unique lower-case names, allowed kinds, canonical values, deterministic order by name, and explicit `identity_relevant` flags. Arbitrary nested JSON values are forbidden.

Required tests: `MESSAGE-ARG-001..014`.

### `derive_root_cause_key`

```text
input:
  root-cause kind/code
  semantic subject
  primary source handle or component identity
  identity-relevant structured arguments

success:
  RootCauseKey
```

Do not hash rendered message text. Separate independent causes even when their messages match.

Required tests: `ROOT-CAUSE-001..010`.

### `derive_finding_fingerprint`

Hash the finding identity fields listed in `DATA_MODEL.md`. Exclude context, severity/policy overrides, rendered prose, notes, and nonidentity arguments.

Required tests: `FINDING-FINGERPRINT-001..012`.

### `bind_finding_to_context`

```text
input:
  validated context
  validated finding without finding_id

success:
  finding with FindingId

errors:
  finding_context_mismatch
  missing_source_handle
  missing_evidence_reference
  remediation_authority_violation
```

`FindingId` is derived from context ID and fingerprint. The operation validates all referenced IDs against the containing registry supplied by the caller.

Required tests: `FINDING-BIND-001..016`.

### `canonical_finding_order`

Sort by the tuple:

```text
primary source origin ID
primary source revision
primary source path
span rank: byte_range before whole_file before unknown
byte start, then byte end when present
rule ID
finding code
finding fingerprint
```

Severity, rendered text, thread order, and insertion order do not affect canonical order.

Required tests: `FINDING-ORDER-001..012` plus randomized property cases.

### `deduplicate_findings`

Deduplicate only byte-equivalent records with the same `FindingId`. A repeated ID with any different record field—including evidence, severity, or policy—returns `result_duplicate_id`; core does not guess how to merge them.

Findings with different fingerprints/IDs are retained even when displayed text matches. Producers that discover additional evidence for one semantic finding must construct one canonical finding before binding/finalization.

Required tests: `FINDING-DEDUP-001..010`.

### `derive_warning_id`

Hash the canonical structured warning projection except `warning_id`. Rendered prose is not an E0 warning field.

Required tests: `WARNING-ID-001..008`.

### `validate_warning_record`

Checks context/producer, stable warning code, optional subject, exact handle/evidence references, canonical arguments/order, and supplied ID. A warning cannot carry a diagnostic rule ID or replace `NotEvaluated`.

Errors:

```text
warning_context_mismatch
missing_source_handle
missing_evidence_reference
invalid_message_argument
canonicalization_failure
```

Required tests: `WARNING-001..014`.

## 8. Budget and truncation operations

### `validate_budget`

Validates positive limits, implementation maximums, and absence of unknown budget dimensions.

Required tests: `BUDGET-001..012`.

### `accumulate_budget_usage`

Purely adds usage values with checked arithmetic. Overflow returns `usage_overflow`; it never wraps.

Required tests: `BUDGET-USAGE-001..008`.

### `classify_truncation`

```text
input:
  validated budget
  observed usage
  omitted counts and affected collections/capabilities

success:
  not_truncated
  or explicit truncated state
```

If a producer cannot know an omitted count, it records `count_unknown`; it does not write zero.

Required tests: `TRUNCATION-001..012`.

## 9. Result-envelope operations

### `validate_result_envelope`

```text
input:
  E0 result envelope without trusted canonical digest

success:
  validated canonicalizable envelope

errors:
  invalid_profile_identity
  generation_mismatch
  result_context_violation
  result_reference_violation
  result_status_violation
  missing_source_handle
  missing_evidence_reference
  missing_conflict_reference
  result_duplicate_id
  evidence_derivation_cycle
  coverage_conflict
  budget_invalid
  schema_version_unsupported
  unknown_field
  duplicate_field
  canonical_digest_mismatch
  contract_violation
```

Checks:

1. supported schema/canonicalization versions;
2. validated context and matching context ID;
3. all entries use the same context;
4. IDs/fingerprints match canonical projections;
5. evidence derivation is acyclic and all layered references resolve exactly once;
6. coverage records and capability summaries agree;
7. arrays contain no duplicate IDs and are canonically ordered or can be ordered before finalization;
8. status agrees with `NotEvaluated`, affecting conflicts, optional failures, and truncation;
9. budget usage agrees with every collection count and serialized-size validation phase;
10. no presentation-only, volatile, duplicate, or unknown fields exist;
11. supplied canonical digest, when present, matches recomputation.

Required tests: `ENVELOPE-001..036`.

### `canonical_result_order`

Returns a new logically equivalent envelope with every set-like collection sorted according to `CANONICALIZATION.md`. It does not discard duplicates or repair invalid records.

Required tests: `ENVELOPE-ORDER-001..014`.

### `canonical_result_digest`

```text
input:
  validated, canonically ordered envelope without canonical_digest

success:
  canonical JSON bytes
  canonical result ContentDigest

errors:
  canonicalization_failure
  contract_violation
```

The caller inserts the digest, then revalidates the completed envelope. Hash vectors live in `examples/HASH_VECTORS.json`.

Required tests: `CANONICAL-001..020`.

### `finalize_result_envelope`

Pipeline:

```text
validate identities/context/layered references
→ canonical order
→ validate summaries/status/count budgets before byte count
→ set output_bytes to an initial checked value
→ serialize digest projection and derive canonical digest
→ insert fixed-length digest and serialize final envelope
→ update output_bytes to the final canonical byte length
→ recompute digest because output_bytes is inside the digest projection
→ repeat until both output_bytes and digest are stable
→ validate max_output_bytes
→ if over budget, require producer-level explicit truncation and restart from validation
```

E0 must converge within the bound specified by `CANONICALIZATION.md`; otherwise return `canonicalization_failure`. Core does not decide what to omit and refuses silent post-serialization clipping.

Required tests: `FINALIZE-001..016`.

## 10. Schema compatibility operations

### `validate_schema_version`

```text
input:
  schema ID
  encountered version
  supported version range

success:
  exact_supported
  compatible_supported

errors:
  schema_version_unsupported
```

E0 internal contracts use exact major compatibility. Unknown major versions fail. Unknown required fields fail even within a nominally compatible version.

Required tests: `SCHEMA-001..010`.

## 11. Forbidden convenience operations

Do not add any `wow-core` operation equivalent to:

```text
get_current_profile
read_source_handle
resolve_repository
fetch_url
open_database
log_finding
render_all_messages_for_locale
search_entity
infer_api_name
is_secret_value_safe
run_rule
load_configuration
now
random_id
best_effort_merge
```

Those operations require state or domain ownership outside this crate.

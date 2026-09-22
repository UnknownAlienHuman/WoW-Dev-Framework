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
9. Recheck the normalized result for host/drive prefixes: removing leading `.`
   components must not turn `./C:/file` or `./C:file` into an accepted path.

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
- every external entry passes the same provider grammar, provider-to-generation
  binding and bounded scope/revision checks as `ExternalGeneration::new`, including
  records obtained through deserialization;
- unique schema/producer IDs;
- canonical ordering;
- supplied context ID matches derived ID.

Returns structured errors rather than rewriting mismatched IDs. Profile and nested
external-entry admission errors propagate unchanged through validation, context-ID
retrieval, merge and the strict same-generation guard. A correctly recomputed
context digest does not waive nested-record validation.

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
- Schema and producer inventories must be identical in every mode, including
  absent/present entries. Only the optional project field or external scopes named
  by the chosen mode may change. Missing entries and version conflicts return
  `merge_mode_violation` on the affected version collection; input duplicates are
  rejected by context validation before merging.

Required tests: full matrix `CONTEXT-MERGE-001..028`.

### `require_same_generation`

Strict guard for multi-input operations. Revalidate both complete contexts before
comparing their IDs: matching caller-supplied IDs are not evidence that decoded
records retain the fields that produced those IDs. This also applies to two
references to the same decoded record. Propagate narrow context/profile validation
errors without rewriting IDs. Return success only when both contexts validate and
their exact IDs match; never call a permissive merge mode.

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

The executable single-record API validates local authority, canonical reference
arrays and the content-addressed ID; it does not own a source/coverage registry.
Derivation-input closure belongs to `validate_evidence_derivation_graph`; source
handles, coverage triples and the containing context are joined by the envelope
and source-owning producer. Neither constructor nor local validation certifies
source provenance. All three reference arrays reject duplicate or noncanonical
order after decoding, even when an attacker recomputes the record ID. Explicit
constructors may sort inputs, but validators do not repair decoded records.

Required tests: `EVIDENCE-001..030`, including the scoped executable cases in
`tests/evidence/records.rs`; remaining owner joins are not certified by those tests.

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
  evidence_context_mismatch
  duplicate_evidence_reference
  missing_evidence_reference
  evidence_authority_violation
  evidence_derivation_cycle
  local evidence shape/identity errors
```

Every input must resolve once in a single context, including disconnected records.
For each edge, output confidence cannot exceed input confidence in the order
`proven > derived > possible > candidate`. In particular, Possible cannot become
Derived and Candidate cannot become Possible by changing producer or provenance.
A proven record cannot carry derivation inputs at all. Weak input records remain
valid evidence; only an attempted strengthening is rejected.

Runtime-probe provenance or a runtime-scenario claim anywhere in a record's
ancestry prohibits a platform-contract conclusion. Changing an intermediate
record's provenance or using a diamond/shared input does not erase that restriction.
Unrelated runtime records do not block an independent platform derivation. This
is a proof ceiling, not a source-authority or runtime-behavior certification.

The validator sorts borrowed record references by ID, validates one-context shape,
checks edge closure/confidence, and walks the DAG iteratively with one active
frame per record. Shared inputs are visited once, not once per path. Temporary
storage is O(V); indexed graph work is O((V + E) log V), plus record-byte validation.
There is no recursion proportional to derivation depth and no copied JSON graph.
Callers still bound total input/serialized sizes; this operation does not replace
result/host budgets, count empty input as absence, or expose a traversal service.

Shape/graph errors precede ID verification so hostile cyclic wire data can be
rejected as a cycle without constructing a cryptographic hash fixed point. Every
record ID is nevertheless verified before success. The envelope calls this same
validator, rather than maintaining a second recursive implementation. Caller
records are never reordered or rewritten.

Required tests: `EVIDENCE-GRAPH-001..015` in `tests/evidence/derivation.rs`.

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

The constructor and decoded-record validator both require canonical unique evidence
and affected-scope arrays, at least two evidence IDs, and a nonempty affected scope.
A recomputed ConflictId does not admit duplicate or reversed affected refs. Scope
validation does not itself resolve opaque evidence IDs: the containing registry
must still join them and validate context. Capability-wide and partition-specific
refs are distinct allowed scope entries, not duplicates of each other.

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

1. Select exactly one record for each required `(capability, partition, producer)` statement requested by the caller. Validate uniqueness by this semantic key across the entire input, not by adjacency after digest sorting. Different producers remain independent statements, even on the same partition.
2. If no required partition is applicable, return `not_applicable`.
3. Ignore `not_applicable` only when another required partition is applicable.
4. Among applicable partitions, use precedence `failed > unknown > partial > complete`.
5. Preserve every partition ref, affecting conflict ID, and truncation ref.
6. Do not return `not_evaluated`; that is produced when a subject consumes summaries.
7. Never drop the underlying coverage records from the envelope.

Required tests include `COVERAGE-COMBINE-001..034`.

### `validate_capability_summary`

Recompute the summary and compare context, producer, status, partition refs,
conflicts, and truncation. The record-local API operates on an explicit selected
record slice. Before consuming it, availability, negative authority and the E0
envelope all validate the retained owner registry and select **all supplied
records for that capability**, not only the summary's advertised refs. A summary
cannot hide a worse partition or omit an affecting conflict by dropping its ref.
Logical duplicate statements and summary owners reject even if their version or
content-addressed ID differs; independent producers are preserved.

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
- The Rust entrypoint takes an explicit conflict registry as its last argument.
  Before returning either result it validates the subject, one context, nonempty
  required summaries, logical record uniqueness, every raw record, conflict
  closure/scope, and recomputes summaries from all supplied records for each
  required capability. Even a nominally complete summary is untrusted input.
- A missing or omitted required record, contradictory status, mixed generation,
  or omitted affecting conflict is an error; valid incomplete input becomes
  `NotEvaluated`. Optional capabilities not selected by a required summary do not
  block the subject. Shared blockers are explicitly deduplicated during aggregation.

Required tests: `CAPABILITY-001..024`.

### `derive_not_evaluated_id`

Hash context, producer, subject, reason, blocking capabilities, blocking partitions/coverage IDs, and conflicts.

Required tests: `NOT-EVALUATED-ID-001..008`.

### `validate_not_evaluated_record`

The local API checks context/producer, subject identity, canonical reference sets,
nonempty blocking capabilities and supplied ID. It also rejects duplicate blocker
CoverageIds, blockers outside the parent capability set, noncanonical nested
conflict arrays and nested conflict IDs absent from the parent. Constructors sort
outer sets; decoded validators never repair them.

Both the envelope and negative-authority consumer then join each blocker against
an already admitted raw coverage/conflict registry. Capability, partition and
status match exactly. Empty nested conflict arrays are the E0 compact projection:
the raw record supplies its full conflict set, all of which must occur in the
parent. Nonempty nested arrays match that set exactly. Complete coverage without
conflict or truncation is not a blocking partition. Evaluation conflicts resolve
and affect a declared blocking capability. Local ID validity alone is insufficient.
Errors propagate as `coverage_record_missing`, `coverage_conflict`,
`duplicate_coverage_record`, `duplicate_conflict_reference`,
`missing_conflict_reference`, `result_context_violation`, or the narrower local
identity/subject error. Record validation does not prove producer honesty or
exhaustiveness of a caller-selected required scope.

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

The Rust operation returns `CoreResult<NegativeAuthorityDecision>` and requires
`context_id` and the raw `coverage_records` alongside the summaries and conflict
registry. It shares admission with capability availability. An empty required
summary set is `coverage_record_missing`, never vacuous authoritative absence.
The result retains that exact `context_id`. A contradictory generation is an
error rather than a synthesized absence decision.

`lookup_completed` describes a caller-reported exact **miss**. Do not call this
operation for a positive lookup. This pure helper does not perform or attest
lookup execution, source acquisition, profile availability, registry completeness,
or the external eligibility of evidence. Those are caller/owner obligations.

A known scope with a lookup that did not run retains `capability_not_evaluated`,
not `scope_unknown`. Nonapplicable partitions are neutral alongside applicable
ones; all-nonapplicable input yields `not_applicable` only after evaluating every
candidate/evaluation/truncation/conflict blocker. Every partial/unknown/failed
partition contributes its reason, not only the summary's worst status.

See [coverage consumer admission and migration](COVERAGE_CONSUMERS.md).
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

`validate_budget(&BudgetLimits)` validates the nine positive limit dimensions.
Collection maxima are 10,000,000; output-byte maximum is 1,073,741,824. The wire
structs reject unknown/duplicate dimensions and invalid integer types during
Serde decoding. No new boundary decoder or CoreError conversion is claimed.

`Budget::validate_limits` additionally admits retained truncation entries and
checks actual usage. `Budget::new` may sort outer entries but never repairs
invalid nested metadata; `with_output_bytes` revalidates the resulting budget.
Truncation does not waive an exceeded count or byte limit. Count and byte usage
remain checked against actual envelope output, not inferred omission counts.

Executable cases: `BUDGET-001..006` in `tests/budget/limits.rs`.

### `accumulate_budget_usage`

Purely adds usage values with checked arithmetic. Overflow returns `usage_overflow`; it never wraps.

Executable cases: `BUDGET-USAGE-001/002` exercise all nine dimensions, zero,
commutativity, exact maximum addition and overflow without operand mutation.

### `classify_truncation`

The existing Rust operation accepts `Vec<TruncationEntry>`. Empty input means
`NotTruncated`; nonempty input is ordered and admitted as `Truncated`. Numeric
limits and usage are separate `Budget`/envelope checks, not extra implicit inputs.

The shared admission checks nonempty retained truncation, collection grammar,
unique ordered collection IDs, unique ordered capability IDs and exactly one
known-count/unknown-count representation. A fresh entry constructor orders and
deduplicates its capability set. Decoded entries must already be canonical.
`Truncated { entries: [] }` is invalid; it is not silently converted to clean.

If a producer cannot know an omitted count, it records `count_unknown`; it does
not write zero. Exact known zero remains supported by the nonnegative contract.
Collection/capability selection is still the producer's responsibility; the core
validator does not rediscover omitted source data or certify a claimed count.

Invalid names return the existing identifier errors at `entries.collection_id`.
Contradictory counts and invalid sets return `contract_violation` at
`entries.omitted_count`, `entries.capability_ids`, or `truncation.entries`.
Errors do not copy raw collection names into messages.

`evaluate_negative_authority` validates this same retained state before using it
as a denial condition. Valid truncation still denies absence conservatively;
validation does not broaden the authority scope or invent missing coverage.

Executable cases: `TRUNCATION-001..013`, `ENVELOPE-025/026`, and
`FINALIZE-001/002` in `tests/e0_budget_conformance.rs` and `tests/budget/`.

### Truncation wire admission

The retained `not_truncated` variant accepts no payload fields. Decoding uses a
private empty-struct wire variant, because a tagged unit variant can silently
ignore additional fields even when the container requests strict field handling.
The public enum, serialized bytes, constructor normalization and semantic
validation rules above are unchanged. `TRUNCATION-013` exercises the actual wire
decoder, including discarded-omission attempts and duplicate status fields.

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

The concrete E0 decoders bind this check to their own supported schema:
`schema:wow:check-result` or `schema:wow:operation-error`, version `0.1.0`.
They must never use the encountered schema as its own supported baseline.
Both finalization and validation reject another schema family or a future
version, even when the supplied digest and output-byte count are self-consistent.

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

### Finding/warning implementation admission

`FindingDraft::bind`, `Finding::validate` and `WarningRecord::validate` now admit
one retained source/evidence registry before resolving diagnostic references.
Handles are validated and their IDs must be unique. All supplied evidence must
belong to the requested context, pass the existing typed derivation validator
(including ancestor closure and confidence constraints), and resolve its source
handles. A matching caller-supplied ID is not validation. The envelope reuses one
private admitted registry for all findings/warnings instead of revalidating or
JSON-serializing the entire evidence registry per diagnostic.

Finding related-handle/evidence/required-capability arrays and warning reference
arrays must already be sorted and unique when decoded. Existing fresh constructors
still canonicalize their set inputs; validators never repair retained records.
Warning subject kind and ID are both present or both absent; present values use
the same grammar and bounds as construction. Fingerprint and record IDs are checked
after field/reference admission. Existing hash projections remain unchanged.

`Remediation::new` and retained finding validation share class-specific shape:
`exact_edit`/`validated_recipe` require a recipe ID; every supplied plan handle
must resolve. An `exact_edit` with empty or Candidate evidence is rejected. This
is a necessary metadata check, not permission to apply a recipe or proof that an
edit is correct. Rule-specific recipe guards remain with the rule/remediation owner.
Unreferenced optional candidates do not taint an otherwise valid local finding.

Message integers retain the existing unsigned 0..9,007,199,254,740,991 range and
now require minimal decimal spelling (no sign or leading zero except `0`). Path
arguments must already be canonical `NormalizedSourcePath` values. Absolute/drive/
UNC/traversal and normalization-requiring spellings reject; a filename containing
`..` without a parent component remains valid. Text/identifier/boolean/digest
semantics and the existing 128-argument/4096-byte value bounds remain unchanged.

Standalone registry admission does not possess a complete `GenerationContext`,
coverage/conflict registry or source bytes. Expected source-generation binding,
coverage/eligibility, root-cause closure, actual source content and producer
truth remain envelope/owner obligations. `derive_warning_id` is still a pure
identity computation, not a substitute for `validate_warning_record`.

### E0 envelope retained coverage admission

`validate_result_envelope`, finalization and canonical reordering require the
same raw-statement/conflict/summary and evaluation joins described above. The
shared checker runs even when no summaries reference a retained statement.
No separate weaker envelope implementation remains. Required-capability calls
still reject empty summaries/records; an envelope may represent an operation
with no retained coverage, such as a failed result. Structural validation of
that case is not an evaluation or negative-authority certificate. Root status
classification, requested-scope selection, source provenance, result ordering
and host input limits remain their separate contracts.

## Canonical serialization primitive admission

The value-based primitives `canonical_json_bytes`, `canonical_json_string`,
`domain_separated_digest` and typed `Id::derive` implementations reject duplicate
object fields before value projection. They return the existing `duplicate_field`
code, without key echo. Unsupported scalar values, malformed map emission and
custom Serialize errors return `canonicalization_failure`; custom error prose is
replaced with a fixed safe reason. Numeric wrappers under the workspace's unified
serde_json feature selection preserve exact allowed integer semantics.

This does not change the precondition of the raw-byte `derive_typed_digest_id`
operation or add JSON parsing to that helper. Container semantics, accepted golden
bytes, domain tags and public signatures remain unchanged. See the serializer
admission section in `CANONICALIZATION.md` and `tests/e0_canonical_conformance.rs`.

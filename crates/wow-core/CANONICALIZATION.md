# `wow-core` canonicalization and hashing

**Status:** normative E0-A canonical profile; no Rust code yet.

Canonicalization makes equivalent logical results byte-identical. It is not a cosmetic pretty-printer and must not be delegated to incidental map iteration or a transport library's defaults.

Canonicalization version:

```text
wow-core-json/e0-1
```

A result declares this version explicitly. A future incompatible rule set receives a new version and migration/hash vectors.

## 1. Pipeline

```text
validate semantic records
→ normalize family-specific identifiers, digests, paths, and spans
→ resolve and validate all internal references
→ reject duplicates and unknown fields
→ sort every set-like collection using its semantic key
→ construct the canonical digest projection
→ encode canonical JSON bytes
→ SHA-256 the projection bytes
→ insert canonical_digest
→ encode final canonical JSON bytes
```

The digest projection excludes the `canonical_digest` field itself.

## 2. Canonical JSON subset

E0 uses a deliberately small deterministic JSON subset.

### Encoding

- UTF-8 without BOM.
- No insignificant whitespace.
- Object keys use fixed ASCII schema names and are emitted in ascending bytewise order.
- Dynamic maps are represented as sorted arrays of entries; arbitrary external strings are not object keys.
- Strings use standard JSON escaping for quotation mark, reverse solidus, and control characters.
- Valid non-ASCII UTF-8 is emitted directly rather than converted to `\u` escapes.
- Unpaired surrogate values are invalid.
- Booleans are lowercase JSON `true`/`false`.
- Identity-critical numbers are integers only.
- Negative zero and floating-point numbers are forbidden.
- Duplicate object keys are invalid at parse time.

Because schema keys are ASCII and numbers are integral, equivalent implementations must not depend on locale or JavaScript floating-point formatting.

### Optional fields

- Optional fields are omitted when absent.
- `null` is forbidden unless a future field explicitly defines a distinct null state.
- Empty set/list fields required by schema are encoded as `[]`.
- Empty objects are used only for fixed schema objects that define an empty state.

### Unknown fields

The internal E0 schema is strict. Unknown fields are rejected, not ignored or reordered into an extension bag.

## 3. Text normalization rules

### Administrative IDs

- Canonical lowercase ASCII according to the owning grammar.
- No surrounding whitespace.
- No Unicode case folding.
- No normalization of lookalike characters into ASCII.

### Source paths and exact entity payloads

- Valid UTF-8.
- Case preserved.
- No Unicode normalization.
- Path separators normalized as defined in `DECISIONS.md` and `OPERATIONS.md`.
- Percent encoding uses uppercase hexadecimal and encodes UTF-8 bytes.

### Human notes and display prose

Preserve exact valid UTF-8 only in transport presentation outside the canonical E0 envelope. Notes, rendered messages, line/column hints, and source excerpts are not E0 canonical fields and therefore cannot affect IDs, result bytes, or result digests.

## 4. Canonical collection order

### Schema versions

```text
schema_id
then semantic version precedence
```

Duplicate schema IDs are invalid, so the second key is defensive only.

### Producer/tool versions

```text
producer_id
then semantic version precedence
```

Duplicate producer IDs are invalid.

### External generations

```text
provider_id
scope_id
external_generation_id
```

Two different generation IDs for one `(provider_id, scope_id)` are a context conflict.

### Capabilities

```text
capability_id
then combined status rank
```

Duplicate capability IDs in one summary are invalid.

Serialized status rank for tie-breaking only:

```text
complete
partial
unknown
failed
not_applicable
```

The rank does not imply coverage aggregation precedence.

### Coverage records

```text
capability_id
partition_id canonical string
producer_id
producer_version
coverage_id
```

Duplicate `(capability_id, partition_id, producer_id, context_id)` records are invalid.

### Capability summaries

```text
capability_id
producer_id
producer_version
```

Partition refs, conflict IDs, and truncation refs inside each summary are sorted by their own canonical keys.

### Source handles

```text
handle_id
```

The ID already binds the canonical identity projection. Human source ordering is handled in findings, not in the registry.

### Evidence records

```text
evidence_id
```

### Conflicts

```text
conflict_id
```

### Findings

Canonical finding order:

```text
primary source origin ID
primary source revision
primary source path
span rank
byte_start when present
byte_end when present
rule_id
finding_code
finding_fingerprint
```

Where `a` means “when present”; absent offsets sort after numeric offsets. Span rank:

```text
byte_range
whole_file
unknown
```

Severity, rollout policy, rendered prose, insertion order, and producer thread do not affect canonical order.

### `NotEvaluated` records

```text
subject_kind
subject_id
reason_code
blocking capability IDs joined in their canonical sorted order
blocking coverage IDs/partition IDs joined in canonical sorted order
conflict IDs joined in canonical sorted order
not_evaluated_id
```

### Warnings

```text
warning code
subject identifier when present
primary source handle ID when present
canonical structured-argument bytes
```

### Message arguments

```text
argument name
```

Argument names are unique.

### Reference arrays

Handle, evidence, conflict, coverage, capability, and derivation-input ID arrays are sorted by referenced canonical ID. Structured coverage/affected/truncation refs use the field tuple declared by their owning type. Duplicates are invalid.

## 5. Domain-separated hash material

Each hashed identity begins with a fixed ASCII domain record in its canonical material. Conceptually:

```text
{
  "domain": "wow-core/<identity-family>/e0-1",
  "value": <identity projection>
}
```

Required domains:

```text
wow-core/source-handle/e0-1
wow-core/evidence/e0-1
wow-core/conflict/e0-1
wow-core/coverage/e0-1
wow-core/finding-fingerprint/e0-1
wow-core/finding/e0-1
wow-core/root-cause/e0-1
wow-core/not-evaluated/e0-1
wow-core/warning/e0-1
wow-core/generation-context/e0-1
wow-core/reference-generation/e0-1
wow-core/project-generation/e0-1
wow-core/external-generation/e0-1
wow-core/result/e0-1
```

A caller may not reuse one family's digest bytes under another family tag without hashing the correct domain material.

## 6. Identity projections

### Source handle

Included:

```text
origin_kind
origin_id
revision
reference_generation when present
project_generation when present
canonical path
span state and byte offsets
content digest
entity key when present
```

Excluded/prohibited in the E0 canonical handle:

```text
line/column hints
display aliases
resolved host paths
source excerpts
```

### Generation context

Included:

```text
full structured profile identity
reference generation
project generation when present
sorted external generations
sorted schema versions
sorted producer versions
```

Excluded:

```text
display labels
activation time
host/process details
```

### Evidence

Included:

```text
context ID
provenance
confidence
claim scope
producer ID/version
sorted source handle IDs
sorted semantic coverage refs
sorted derivation input IDs
```

Conflict back-references, notes, rendered explanations, and source excerpts are not E0 evidence fields.


### Conflict

Included:

```text
context ID
conflict code
sorted evidence IDs
sorted affected capability/partition refs
subject entity key when present
```

### Coverage record

Included:

```text
context ID
capability and partition IDs
status
producer ID/version
sorted missing input IDs
failure code when present
sorted conflict IDs
sorted truncation refs
```

### NotEvaluated

Included:

```text
context ID
producer ID/version
subject kind/ID
reason code
sorted blocking capability IDs
sorted blocking coverage/partition records
sorted conflict IDs
```

### Warning

Included:

```text
context ID
producer ID/version
warning code
optional subject and primary source handle
sorted related handles/evidence IDs
canonical structured message arguments
```

### Finding fingerprint

Included:

```text
rule ID/version
finding code
subject entity key when present
primary source handle ID
identity-relevant message arguments
root-cause semantic key when present
```

Excluded:

```text
context ID
severity/policy overrides
nonidentity arguments
rendered prose
notes
```

### Finding ID

Included:

```text
context ID
finding fingerprint
```

### Result digest

Included:

```text
schema ID/version
canonicalization version
operation ID
context
status
coverage records
capability summaries
source handles
evidence records
conflicts
findings
NotEvaluated records
warnings
budget limits/usage/truncation
```

Excluded/prohibited in the canonical envelope:

```text
canonical_digest itself
timestamps
elapsed time
random/trace/request IDs
host/user/temp paths
thread/worker counts
memory/process metrics
localized/rendered prose or free-form notes
line/column hints
source excerpts
credentials or private configuration
```

## 7. Budget byte count

`output_bytes` is the length of the final canonical UTF-8 JSON including `canonical_digest`.

Finalization therefore uses two phases:

1. derive digest from the envelope without `canonical_digest`;
2. insert the fixed-length digest and encode final bytes;
3. set/check `output_bytes` using the final byte length;
4. if the usage field changes the byte length, repeat until stable; E0 must converge in at most two additional passes because only decimal digit width may change.

E0 uses this fixed-point model. Implementations must not move `output_bytes` outside the hashed projection or choose another model without a canonicalization-version change. The committed examples and `examples/HASH_VECTORS.json` record the required bytes.

## 8. Canonical status rules

Envelope status is derived, then validated:

```text
failed
  no valid requested result can be returned

partial
  valid result exists, but at least one requested lane is NotEvaluated,
  an optional lane failed/staled, or an affecting collection is truncated

complete
  all requested E0 lanes evaluated and no affecting truncation exists
```

Findings do not change `complete` into `partial`.

## 9. Determinism hazards to test

- randomized insertion order;
- hash-map/set iteration;
- parallel diagnostic completion order;
- Windows versus POSIX path separators;
- uppercase versus lowercase digest input;
- duplicate records with identical IDs but different contents;
- absent versus empty optional collections;
- large integer serialization;
- Unicode source path payloads;
- free-form note changes;
- severity/policy override changes;
- line-hint changes;
- reordered evidence and capability lists;
- different worker counts.

## 10. Hash vectors

[`examples/HASH_VECTORS.json`](examples/HASH_VECTORS.json) is normative for E0 implementation.

Each vector records:

```text
vector ID
domain
logical value
exact canonical UTF-8 JSON text
SHA-256 digest
expected type-tagged ID when applicable
```

The first coding agent must run these vectors before integrating any higher E0 crate. A serializer that passes round-trip tests but fails the hash vectors is not compatible.

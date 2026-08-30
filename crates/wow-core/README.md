# `wow-core` implementation contract

**Status:** E0-active contract scaffold; no Rust code yet.

## Mission

`wow-core` owns the smallest transport-, storage-, parser-, and product-independent contracts required to describe an exact WoW analysis result. Every other crate may depend on these contracts; therefore this crate must remain narrow, deterministic, and free of domain workflows.

## Owned responsibilities

- exact profile identity;
- immutable reference/project/external generation identity;
- stable entity, rule, producer, capability, partition, and handle identifiers;
- normalized source locations and content digests;
- evidence provenance and confidence;
- coverage and `NotEvaluated` state;
- normalized findings and result envelopes;
- deterministic ordering keys and canonical text-safe representations;
- common bounded-size/count types used at crate boundaries.

## Explicit non-responsibilities

`wow-core` does not:

- parse Lua, XML, TOC, JSON, or SQLite;
- access the filesystem, network, process environment, editor, or WoW client;
- know API names, addon frameworks, rule algorithms, search ranking, graph semantics, or database schemas;
- decide which profile is current;
- perform persistence or logging;
- own application configuration;
- contain convenience helpers that import higher-layer semantics.

## Conceptual data contracts

### Identity

Required value types:

```text
ProfileId
ReferenceGenerationId
ProjectGenerationId
ExternalGenerationId
EntityKey
RuleId
ProducerId
CapabilityId
CoveragePartitionId
StableHandleId
ContentDigest
SchemaVersion
ToolVersion
```

Every identifier must have a canonical string form, validation rules, and deterministic equality/order semantics. IDs must not contain unnormalized absolute local paths or secret data.

### Profile identity

A profile identity carries, as available:

```text
profile ID
flavor/edition
Interface number
client build
source revision and logical content digest
builder/schema/correction-set versions
```

The type must distinguish an incomplete fixture identity from a release-grade profile manifest. A fixture profile cannot masquerade as a released pack.

### Generation context

A result context carries exactly one reference generation and at most one project generation, plus explicitly separated external generations. Combining mismatched contexts must fail rather than silently selecting one.

### Source handle

A stable source handle contains:

```text
repository or pack identity
revision/profile/generation
normalized repository-relative path
byte and line span when known
content digest
optional symbol/entity key
```

Unknown spans are explicit. Paths are slash-normalized and root-relative. A handle is an identity, not permission to read arbitrary host paths.

### Evidence and confidence

Provenance classes and confidence levels follow [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md). Candidate systems cannot construct `Proven` evidence without an owning trusted producer path.

### Coverage

Coverage records:

```text
partition
capability
status = Complete | Partial | Unknown | Failed | NotApplicable | NotEvaluated
missing inputs/capabilities
producer/generation
optional conflict references
```

Negative authority is a derived decision over explicit complete coverage, not a boolean stored without evidence.

### Finding and result envelope

A normalized finding carries machine-readable rule identity, message arguments, severity/policy, source/evidence references, capability requirements, coverage status, generation context, root-cause key, and optional remediation class.

A result envelope carries:

- one coherent generation context;
- capability summary;
- ordered findings/data;
- warnings and `NotEvaluated` records;
- deterministic schema/version metadata;
- truncation/budget status.

## Required operations

Concrete Rust naming may differ only with an accompanying contract update. Required semantics:

| Operation | Required behavior |
|---|---|
| `parse_profile_id` | Validate and canonicalize a profile identifier without consulting external state. |
| `validate_profile_identity` | Reject contradictory Interface/build/revision/digest combinations. |
| `build_source_handle` | Normalize path/span/digest and reject host-path escape or invalid spans. |
| `verify_source_handle_content` | Compare a handle digest with supplied content identity; no filesystem read. |
| `merge_generation_context` | Combine compatible contexts or return an explicit mismatch. |
| `require_same_generation` | Guard a multi-input operation against cross-generation mixing. |
| `combine_coverage` | Compute conservative capability coverage from named partitions. |
| `evaluate_negative_authority` | Return authoritative/partial/failed/conflict state with reasons. |
| `canonical_finding_key` | Produce stable ordering/deduplication identity from structured fields. |
| `canonical_result_order` | Order outputs independently of hash iteration, thread order, or filesystem order. |
| `validate_result_envelope` | Ensure every item belongs to the envelope context and required fields are present. |

## Invariants

1. No floating `current`, `latest`, or implicit profile identity.
2. No mixed reference/project generations in one result.
3. No authoritative negative without complete relevant coverage.
4. No confidence upgrade based on source popularity or model inference.
5. No absolute host paths in public handles by default.
6. No timestamps or random IDs in canonical result digests.
7. Unknown fields/states remain distinguishable from absent fields/states.
8. Serialization round-trips must preserve semantic identity.
9. `NotEvaluated` is not success and is not a diagnostic false negative.
10. Public ordering is total and deterministic.

## Error taxonomy

The crate owns generic boundary errors only:

```text
invalid_identifier
invalid_profile_identity
generation_mismatch
invalid_source_handle
digest_mismatch
coverage_conflict
negative_authority_unavailable
result_context_violation
budget_invalid
schema_version_unsupported
```

Errors must be structured. Human messages are projections, not the contract.

## E0 deliverable

E0 implements only the types and operations required by:

- one fixture profile;
- one project generation;
- generic and WoW findings in one envelope;
- `Complete`, `Partial`, `Failed`, and `NotEvaluated` coverage cases;
- deterministic JSON/golden output handoff.

Do not implement a generalized plugin system, database IDs, distributed tracing, or compatibility framework in E0.

## Required tests

- valid/invalid/canonical profile IDs;
- contradictory profile identity rejection;
- source path normalization and traversal rejection;
- generation compatibility matrix;
- complete versus partial negative authority;
- candidate evidence cannot become proven through merge;
- deterministic finding/result ordering under randomized input order;
- serialization round-trip;
- no volatile field changes canonical digest;
- malformed spans and digest mismatch.

## Documentation sources

- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/GLOSSARY.md`](../../docs/GLOSSARY.md)
- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)

## Definition of done

`wow-core` E0 is complete when every other E0 crate can express its inputs/results without unstructured identity strings, all cross-generation mistakes fail deterministically, and the same logical findings serialize byte-identically after canonicalization.

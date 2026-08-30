# Exact lookup, coverage, and authority

**Status:** normative E0-B query contract.

This document defines what an exact reference lookup can state and, equally important, what it cannot state.

## 1. Query identity

```text
ExactSymbolQuery
    profile_id
    reference_generation
    entity_key
    expected_entity_kind
```

The query must name one explicit profile/reference generation. A request containing `current`, `latest`, an unqualified branch, or another floating selector is invalid before lookup begins.

## 2. Canonical fixture keys

E0-B accepts exact keys in the closed fixture grammar:

```text
system:C_E0Fixture
function:C_E0Fixture.KnownApi
function:C_E0Fixture.SecretText
function:C_E0Fixture.RemovedApi
```

The first three are known query keys; `RemovedApi` is intentionally absent.

The view does not auto-correct case, punctuation, namespace, member, or entity kind beyond parsing the canonical grammar.

## 3. Exact symbol outcomes

### `found`

Requirements:

- requested profile/reference generation equals the view identity;
- exact normalized key exists;
- the returned contract dimension is not blocked by conflict;
- required source/evidence/coverage references resolve.

A found result includes:

```text
entity
reference source handles
evidence IDs
coverage IDs
conflict IDs, normally empty for required dimensions
negative authority = not_applicable
```

A found result does not imply that every possible field/facet for the entity has complete coverage.

### `authoritative_absent`

Requirements:

- exact key does not exist;
- query domain/partition is known;
- all required exact coverage records are `Complete`;
- generation is current for the view;
- no conflict blocks the queried domain;
- no truncation/unsupported-input blocker applies;
- core negative-authority decision is authoritative.

This outcome means only:

```text
the exact entity key is absent from the selected complete fixture domain
```

It does not mean a replacement exists, the name is misspelled, or another profile is identical.

### `absent_without_authority`

Used when the key is not found but absence cannot be proved.

Reasons include:

```text
Partial coverage
Unknown coverage
Failed partition
missing declared input
unsupported construct affecting the query domain
truncation
stale/mismatched generation input
unresolved conflict affecting negative authority
```

The result includes exact blockers and coverage/conflict IDs.

### `conflict`

Used when competing complete observations prevent a single normalized answer for the requested contract dimension.

A conflict outcome:

- retains all evidence records;
- names affected capability/partition;
- does not choose the first/last record;
- may coexist with known symbol presence when only a facet/signature dimension conflicts.

### `profile_mismatch`

Returned/rejected before index lookup when the request profile/reference generation differs from the view.

This is not a miss and must not be converted into `absent_without_authority`.

### `capability_unavailable`

Used when the view/model lacks the required capability entirely or the operation is outside the milestone contract.

## 4. Coverage selection

Lookup must select the narrowest complete evidence-bearing partition set.

For E0 symbol lookup:

```text
capability: reference.symbol.exact_lookup
partition: reference.fixture.apidoc.system:C_E0Fixture
```

For restriction lookup:

```text
capability: reference.restriction.facets
partition: reference.fixture.restriction:C_E0Fixture.SecretText
```

Inventory/profile capabilities are prerequisites but do not replace the queried domain coverage.

## 5. Variant matrix

| Query | Complete | Partial | Conflict |
|---|---|---|---|
| `KnownApi` symbol | `found` | `found` when its record is intact, with partial domain coverage exposed | `found` unless symbol dimensions conflict |
| `RemovedApi` symbol | `authoritative_absent` | `absent_without_authority` | `absent_without_authority` when conflict affects domain authority |
| `SecretText` symbol | `found` | `found` if symbol partition intact | `found` if only restriction facet conflicts |
| `SecretText` facets | `found(secret.return)` | `unavailable` if facet partition partial | `conflict` |
| ordinary symbol facets | authoritative none only with complete facet coverage | unavailable | conflict if facet domain conflicted |

The exact expected fixture cases are in [`examples/lookup-cases.json`](examples/lookup-cases.json).

## 6. Complete coverage plus conflict

These states are independent:

```text
input ingestion coverage = Complete
normalized restriction contract = Conflict
facet lookup = Conflict
rule capability = NotEvaluated
```

Do not downgrade the source-ingestion record merely to encode conflict. Keep both axes explicit.

## 7. Existing facts under partial coverage

Partial domain coverage does not necessarily erase positive facts already established by direct evidence.

E0-B rule:

- an intact exact record may still return `found` with `Partial` surrounding coverage;
- an absent exact key cannot be authoritative under that same partial domain;
- consumers must inspect returned coverage even for positive hits;
- a rule requiring complete signature/facet semantics may still become `NotEvaluated` despite symbol presence.

## 8. Restriction facet outcomes

```text
RestrictionFacetLookupOutcome
    found
    none_authoritatively
    unavailable
    conflict
    profile_mismatch
```

### `found`

Returns normalized facet(s), raw field/source evidence, and exact coverage.

### `none_authoritatively`

Requires complete restriction-facet coverage for the subject domain and no blockers.

### `unavailable`

Used for partial/failed/unknown facet coverage or unsupported facet shape.

### `conflict`

Used when multiple observations disagree on kind, target slot, applicability, or payload.

## 9. No implicit candidate generation

Exact lookup never performs:

```text
case-insensitive guessing
prefix lookup
substring search
edit distance
trigram search
FTS
semantic search
external repository search
alias lookup
deprecation lookup
lineage lookup
replacement recommendation
```

Later search layers may consume exact lookup results but cannot retroactively change their evidence level.

## 10. Source/evidence separation for findings

A future `wow.api.exists` finding is assembled as:

```text
project evidence
    exact addon use-site SourceHandle from wow-emmy/project

reference result
    exact absent/coverage decision from wow-reference

rule derivation
    wow-rules finding with both references
```

`wow-reference` never fabricates the addon source span and never creates the finding itself.

For Secret-local analysis:

```text
project evidence
    producer call + local use + control-flow facts

reference evidence
    SecretText secret.return facet

rule derivation
    local-operation finding or NotEvaluated
```

## 11. Negative-authority blocker codes

E0-B uses structured reasons:

```text
query_partition_partial
query_partition_unknown
query_partition_failed
missing_declared_input
unsupported_construct
unknown_field_blocks_capability
unresolved_registration_conflict
truncated_input
profile_mismatch
reference_generation_mismatch
capability_not_implemented
```

Message prose may change; blocker codes are contract data.

## 12. Lookup determinism

Equivalent models and queries must yield identical:

- outcome;
- entity/facet IDs;
- evidence/conflict/coverage ID ordering;
- authority decision/reasons;
- canonical serialized lookup bytes.

Index implementation, hash map iteration, and worker scheduling cannot affect output order.

## 13. Consumer rules

Consumers must:

- preserve lookup context and coverage IDs;
- distinguish `authoritative_absent` from `absent_without_authority`;
- distinguish symbol presence from complete signature/facet availability;
- propagate conflict/NotEvaluated blockers;
- avoid presenting a non-authoritative miss as an error about user code;
- never infer a replacement from this exact result alone.

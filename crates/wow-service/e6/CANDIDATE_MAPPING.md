# E6-B exact owner mapping of external candidate locators

**Status:** normative orchestration seam. Mapping algorithms remain owner-owned.

## Input boundary

An E6-A candidate can carry provider metadata such as:

```text
provider repository/workspace label
provider path or URI
provider symbol/name/kind
provider line/span
provider content/revision/digest hints
provider snippet/summary/relationship
```

All of these enter E6-B as an immutable `UnverifiedProviderLocator`. They are not a `SourceHandle`, `EntityKey`, `ReferenceEntity`, alias, or verified revision.

## Mapping targets

E6-B v1 supports exactly one declared owner target per request:

```text
Project
    exact retained ProjectSnapshot/View and source universe

Reference
    exact retained ReferenceGeneration/View and profile
```

A provider candidate is not searched against every project/reference generation automatically.

## Owner ports

```text
ProjectExternalLocatorMappingPort
    validate_mapping_profile
    map_external_locator
    validate_mapping_receipt

ReferenceExternalLocatorMappingPort
    validate_mapping_profile
    map_external_locator
    validate_mapping_receipt
```

Ports accept typed locator fields and exact view identities; they do not accept raw provider clients or arbitrary callbacks.

## Mapping request

```text
ExternalCandidateMappingRequest
    exact E6-A result set and candidate
    exact locator ID/digest
    target owner kind and publication/view/generation
    exact mapping profile
    allowed locator fields
    privacy/license/consumer scope
    budgets/cancellation
    OperationId + request digest
```

## Mapping statuses

```text
ExactMapped
MultipleMappings
NoMappingWithOwnerAuthority
NoMappingPartial
Conflict
NotEvaluated
Failed
```

### ExactMapped

Owner proves the locator corresponds to one exact retained owner record under the exact profile. Receipt includes exact mapped entity/source/reference IDs, comparison facts, coverage, and owner evidence.

It does **not** validate provider prose, relationship, inferred role, rank, score, recommendation, or negative claim.

### MultipleMappings

More than one owner record satisfies the exact mapping profile. Service preserves every candidate and never chooses one by order, score, name, or path.

### NoMappingWithOwnerAuthority

Allowed only when the owner explicitly supplies scoped negative authority for the exact locator/profile/generation and complete relevant coverage with no conflict/truncation.

It does not prove that the real-world symbol/source/API does not exist elsewhere.

### NoMappingPartial

No mapping was found under incomplete or bounded owner coverage. This is nonauthoritative.

### Conflict / NotEvaluated / Failed

Preserved exactly. No fallback heuristic runs.

## Permitted exact signals

The owner profile may use only signals the owner can independently verify, for example:

```text
exact content digest and canonical source object identity
exact normalized owner path under the bound source manifest
exact semantic/source entity ID supplied and verified by owner
exact repository/source revision already bound to the owner publication
exact byte/span correspondence under one coordinate profile
exact Reference entity key/profile correspondence
```

The profile defines conjunctions, required coverage, normalization, and ambiguity behavior.

## Forbidden service heuristics

`wow-service` cannot map by:

```text
first/only/top result
same display name
same filename/path suffix
same snippet text without owner digest proof
fuzzy/FTS/embedding similarity
provider score or “exact” label
repository stars/owner/name
newest/current/latest publication
line number alone
provider revision label without owner verification
```

E4 `wow-search` is not a hidden mapping fallback in E6-B.

## Mapping privacy

- Provider path/URI may be private; public receipts use bounded normalized locator IDs.
- Owner source is not returned unless the mapping/consumer privacy and license profiles permit it.
- Errors expose exact stable IDs and structured reasons, not private paths, snippets, credentials, or full candidate payloads.
- Mapping cannot widen E6-A disclosure permissions.

## Cross-universe behavior

Same name/path/digest across user project, dependency, Blizzard UI source, Reference Pack, historical, external, and runtime universes remains distinct. Every mapping names exact universe and generation.

A combined multi-owner investigation requires separate mapping requests/receipts. Service never merges them into one identity.

## Generation changes

A mapping receipt is valid only for its exact owner generation/profile. It cannot be reused as proof for a later current generation. A later generation requires a new mapping request and receipt; similar content does not preserve semantic identity automatically.

## Mapping validation

`external_candidate_mapping_validate` verifies:

- exact result/candidate/locator closure;
- owner view/generation/profile retained and compatible;
- mapping implementation/profile identity;
- mapped roots resolve exactly;
- owner evidence/coverage/conflicts and negative-authority state;
- no unverified provider field promoted to owner truth;
- no cross-universe/generation substitution;
- privacy/license restrictions;
- canonical digest and retention/audit closure.

It is read-only and never repairs a mapping.

## Tests

- exact digest/path/entity/reference mappings;
- same-name and same-path ambiguity;
- provider-labelled exact but owner mismatch;
- multiple owner generations;
- partial inventory and no-mapping result;
- scoped authoritative no mapping;
- owner conflict/NotEvaluated/failure;
- provider URI/path traversal attempt;
- mapping receipt reused across generation;
- provider summary/rank upgraded after mapping;
- 1/2/N workers and shuffled owner result order.

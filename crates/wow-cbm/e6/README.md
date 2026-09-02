# `wow-cbm` E6-A external semantic-candidate bridge

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `wow-cbm/e6-a/external-semantic-candidate-bridge`

## Mission

Consume one exact reviewed provider descriptor, one already-acquired narrow transport, one explicit external-state binding, and one closed bounded candidate query; return immutable, loss-preserving, provider-scoped Candidate artifacts without affecting exact local/reference/project/search/context capability.

```text
reviewed ProviderDescriptor
+ negotiated ProviderCapabilitySet
+ ExternalStateBinding
+ ExternalCandidateQuery
+ narrow ExternalCandidateTransportPort
-> validate provider/state/query/profile closure
-> execute one allow-listed operation
-> bound and validate response bytes/items/depth/time
-> normalize every candidate without authority upgrade
-> retain raw-field loss/unknown/conflict records
-> classify coverage/partial/truncation/failure/zero result
-> emit immutable ExternalCandidateResultSet
-> optional ExternalCandidateArtifact / explanation / descriptive comparison
```

## Public operations

```text
validate_provider_descriptor
negotiate_provider_capabilities
validate_external_generation
normalize_external_candidate_query
query_external_candidates
continue_external_candidate_query
validate_external_candidate_result_set
explain_external_candidate
build_external_candidate_artifact
compare_external_candidate_results
validate_external_candidate_cache_entry
```

## External-state classes

```text
StableExternalGeneration
ObservedMutableGeneration
OpaqueExternalState
```

Only the first claims an immutable provider generation. The second binds one observation/session receipt. The third is explicitly nonreproducible and receives the narrowest cache/continuation guarantees.

## Authority ceiling

Every normalized item has:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

Provider labels such as exact, verified, authoritative, stable, top, unique, or high-confidence remain quoted provider metadata. Scores/ranks are provider-local and cannot be compared or fused across providers.

## Source boundary

Provider paths, repository names, URIs, symbols, spans, snippets, and digests become `UnverifiedProviderLocator` records. They are not `StableSourceHandle`, project/reference entities, or graph assertions. E6-B may submit them to exact owner mapping ports; E6-A does not follow or open them.

## Failure/degradation

Unconfigured, unavailable, incompatible, stale, malformed, timed-out, cancelled, partial, or opaque provider state disables only the external candidate lane. Exact local ReferenceView, project analysis, graph, search, context, diagnostics, and rules remain unaffected.

## Deferred to E6-B

- provider/session/credential acquisition and durable operations;
- provider process/MCP connection ownership;
- exact project/reference source-owner mapping;
- explicit candidate selection receipt;
- exact mapped-root context handoff;
- service envelopes, retention/store catalogs, and CLI.

## Completion gate

E6-A implementation is complete only when provider descriptors, state classes, query/transport allowlist, bounded normalization, Candidate ceiling, zero-result semantics, locator isolation, continuation/cache, degradation, privacy/license, cancellation, determinism, and adversarial tests pass with all implementation/profile/fixture/checksum pins frozen.
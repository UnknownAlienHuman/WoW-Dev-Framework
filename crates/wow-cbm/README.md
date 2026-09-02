# `wow-cbm` contract router

**Status:** E6-A external semantic-candidate owner documentation and its E6-B service handoff are complete; no Rust code exists.

`wow-cbm` is an optional, replaceable pure bridge over an already-acquired narrow external-candidate transport. It validates reviewed provider descriptors, state, query, and response contracts, normalizes untrusted results to `provenance=semantic_candidate`, `confidence=Candidate`, and `negative_authority=unavailable`, and never creates exact local authority.

The original scaffold is preserved as [`INITIAL_OVERVIEW.md`](INITIAL_OVERVIEW.md). Active ownership is:

```text
provider descriptor, state, query, and normalization -> wow-cbm E6-A
provider configuration and session authorization     -> wow-service E6-B + host adapters
provider index, build, and update lifecycle           -> separate provider-owner contract if ever needed
project/reference locator mapping                     -> exact owners coordinated by wow-service E6-B
caller selection and context handoff                  -> wow-service E6-B
CLI, daemon, LSP, and MCP transport                    -> apps/wow E7-A
artifact packaging, compatibility, and release        -> E7-B release lifecycle
```

`wow-cbm` does not spawn, configure, install, or discover providers; read or write provider databases; follow paths or URLs; create project or reference handles; select candidates; build context; own service sessions or credentials; or depend on service and application crates.

## Canonical E6-A route

Read the complete package:

1. [`e6/README.md`](e6/README.md)
2. [`e6/AGENTS.md`](e6/AGENTS.md)
3. [`e6/DECISIONS.md`](e6/DECISIONS.md)
4. [`e6/DATA_MODEL.md`](e6/DATA_MODEL.md)
5. [`e6/PROVIDER_DESCRIPTOR_AND_CAPABILITIES.md`](e6/PROVIDER_DESCRIPTOR_AND_CAPABILITIES.md)
6. [`e6/EXTERNAL_STATE_AND_GENERATIONS.md`](e6/EXTERNAL_STATE_AND_GENERATIONS.md)
7. [`e6/QUERY_AND_TRANSPORT.md`](e6/QUERY_AND_TRANSPORT.md)
8. [`e6/NORMALIZATION_AND_AUTHORITY.md`](e6/NORMALIZATION_AND_AUTHORITY.md)
9. [`e6/SOURCE_LOCATORS_AND_MAPPING_HANDOFF.md`](e6/SOURCE_LOCATORS_AND_MAPPING_HANDOFF.md)
10. [`e6/ZERO_RESULT_COVERAGE_AND_DEGRADATION.md`](e6/ZERO_RESULT_COVERAGE_AND_DEGRADATION.md)
11. [`e6/CONTINUATION_CACHE_AND_DETERMINISM.md`](e6/CONTINUATION_CACHE_AND_DETERMINISM.md)
12. [`e6/SECURITY_PRIVACY_AND_LICENSE.md`](e6/SECURITY_PRIVACY_AND_LICENSE.md)
13. [`e6/ERROR_MODEL.md`](e6/ERROR_MODEL.md)
14. [`e6/TEST_MATRIX.md`](e6/TEST_MATRIX.md)
15. [`e6/IMPLEMENTATION_PLAN.md`](e6/IMPLEMENTATION_PLAN.md)
16. [`e6/CONTRACT.json`](e6/CONTRACT.json) and [`e6/examples/`](e6/examples/README.md)

## Authority ceiling

Every normalized provider item, result set, and artifact remains:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
source_verification = unverified until exact owner mapping outside this crate
```

Provider labels, rank, score, top, sole, repetition, stable state, and zero results never raise authority. Scores remain provider-local and are not numerically fused across providers.

External state is explicit:

```text
StableExternalGeneration
ObservedMutableGeneration
OpaqueExternalState
```

A timestamp, repository name, `current`, `latest`, same query, same result, same count, or cache hit does not create generation identity. Opaque state remains explicitly nonreproducible.

## E6-B handoff

The coordinating contract is [`../wow-service/e6/README.md`](../wow-service/e6/README.md). It resolves exact nonsecret provider configuration, acquires an authorized host session, invokes E6-A, publishes immutable results, asks `wow-project` or `wow-reference` to map a bounded locator against one exact retained owner generation, records explicit caller selection, and invokes normal context with one exact mapped root.

`ExactMapped` proves locator-to-owner-record identity only. Selection is caller intent, not verification, edit authorization, lineage, replacement, impact, runtime proof, or core promotion. Provider metadata remains a separate Candidate sidecar.

## Direct dependency

```text
wow-cbm -> wow-core
```

No direct dependency on store, project, reference, graph, search, context, service, applications, MCP implementation, provider SDK, or release tooling.

## E7 exposure and release

The external lane can remain disabled in the product. An adapter enters an E7-A operation/exposure profile and an E7-B release only after exact implementation, capability, state, credential, privacy, license, outage, degradation, mapping, selection, and provider-on/off benefit evidence passes.

Provider adapter configuration contains no credentials. Provider artifacts and compatibility have independent release and update identities; an executable update never silently activates a provider adapter or data set.

## Current state

```text
planned owner documentation: E6-A complete
E6-B service handoff: complete
E7-A frontend and E7-B release routing: complete
implementation frontier: not-started
first wow-cbm implementation: I6-A after exact local prerequisites
repository next package: I0-A / wow-core E0-A
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

# `wow-cbm` contract router

**Status:** E6-A external semantic-candidate bridge is implementation-ready documentation; E6-B orchestration/owner mapping is defined in `wow-service`; no Rust code exists.

`wow-cbm` is an optional, replaceable adapter over an already-acquired narrow external-candidate transport. It validates provider descriptors and external state, issues closed bounded candidate queries, normalizes untrusted results to `provenance=semantic_candidate` and `confidence=Candidate`, and never participates in exact local authority without independent owner mapping.

The original scaffold is preserved as [`INITIAL_OVERVIEW.md`](INITIAL_OVERVIEW.md). The active contracts narrow that scaffold as follows:

```text
provider descriptor/state/query/normalization -> wow-cbm E6-A
provider configuration/session authorization -> wow-service E6-B + host adapters
provider index/build/update lifecycle         -> later explicit host/provider-owner contract
project/reference source-handle mapping       -> project/reference owner ports coordinated by E6-B
explicit selection and context handoff        -> wow-service E6-B
wire/session frontends                        -> E7-A
```

`wow-cbm` does not spawn/configure/install providers, read/write provider databases, discover tools dynamically, follow paths/URLs, create stable project/reference source handles, select candidates, build context, or depend on service/application crates.

## Canonical E6-A route

Read:

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

## E6-B handoff

The coordinating contract is [`../wow-service/e6/README.md`](../wow-service/e6/README.md). It acquires one exact authorized provider session, publishes immutable E6-A results, invokes project/reference owner mapping, records an explicit caller selection, and passes one exact mapped owner root to normal context operations. Mapping and selection never raise external Candidate authority.

## Direct dependency

```text
wow-core
```

No direct dependency on store, project, reference, graph, search, context, service, applications, MCP implementation, or provider SDK.

## Current state

```text
documentation frontier: E6-A owner contract; E6-B coordinating contract linked
implementation frontier: not-started
next documentation package: E7-A transport/session and developer-preview boundary
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
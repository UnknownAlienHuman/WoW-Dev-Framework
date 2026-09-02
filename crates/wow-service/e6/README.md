# `wow-service` E6-B external candidate mapping and context orchestration

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-service/e6-b/external-candidate-mapping-context-orchestration`

## Mission

Coordinate one configured E6-A external semantic-candidate provider through an exact, auditable, generation-bound service workflow without upgrading its evidence.

```text
explicit provider descriptor/profile
+ authorized opaque provider session
+ exact or explicitly classified external state
+ exact retained owner publications
+ closed external-candidate request
-> register durable operation identity
-> acquire and validate provider/owner resources in fixed order
-> call wow-cbm E6-A through its narrow port
-> retain immutable Candidate result/artifact
-> map selected unverified locators through exact project/reference owner ports
-> require an explicit candidate-selection receipt
-> pass only exact mapped owner roots to the existing context service path
-> preserve provider evidence separately in the outer envelope
-> close resources in reverse order
```

## Public operations

```text
external_candidate_status
external_provider_validate
external_generation_validate
external_candidate_query
external_candidate_continue
external_candidate_result_get
external_candidate_result_list
external_candidate_result_validate
external_candidate_explain
external_candidate_artifact_build
external_candidate_mapping_validate
external_candidate_map
external_candidate_selection_validate
external_candidate_select
external_candidate_context
external_candidate_operation_get
external_candidate_operation_reconcile
external_candidate_cache_validate
```

## Direct framework dependencies

```text
wow-core
wow-store
wow-reference
wow-project
wow-cbm
wow-context
```

The active E6-B slice does not directly depend on `wow-search`, `wow-graph`, `wow-recognizers`, `wow-rules`, `wow-emmy`, or `wow-annotations`.

## Authority ceiling

Every provider-origin claim remains:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

An exact owner mapping can prove only that one provider locator corresponds to one exact retained owner entity/source record under the mapping profile. It cannot validate the provider's summary, relation, rank, score, recommendation, inferred role, or absence claim.

## Canonical reading order

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`DATA_MODEL.md`](DATA_MODEL.md)
4. [`PROVIDER_SESSION_AND_CREDENTIALS.md`](PROVIDER_SESSION_AND_CREDENTIALS.md)
5. [`DURABLE_EXTERNAL_OPERATIONS.md`](DURABLE_EXTERNAL_OPERATIONS.md)
6. [`CANDIDATE_MAPPING.md`](CANDIDATE_MAPPING.md)
7. [`EXPLICIT_SELECTION.md`](EXPLICIT_SELECTION.md)
8. [`CONTEXT_HANDOFF.md`](CONTEXT_HANDOFF.md)
9. [`RETENTION_CACHE_AND_DEGRADATION.md`](RETENTION_CACHE_AND_DEGRADATION.md)
10. [`RESULT_ENVELOPE_AND_STATUS.md`](RESULT_ENVELOPE_AND_STATUS.md)
11. [`SECURITY_PRIVACY_AND_AUDIT.md`](SECURITY_PRIVACY_AND_AUDIT.md)
12. [`ERROR_MODEL.md`](ERROR_MODEL.md)
13. [`TEST_MATRIX.md`](TEST_MATRIX.md)
14. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
15. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)
16. [`../../../apps/wow/e6/`](../../../apps/wow/e6/README.md)
17. [`../../wow-cbm/e6/`](../../wow-cbm/e6/README.md)

## Required owner seams

```text
ExternalCandidateProviderCatalogPort
ProviderCredentialAuthorizationPort
ProviderSessionAcquirePort
ExternalCandidateArtifactStorePort
DurableExternalOperationPort
ProjectExternalLocatorMappingPort
ReferenceExternalLocatorMappingPort
ExternalCandidateRetentionPort
ExternalCandidateAuditPort
ContextUseCasePort
```

No raw provider/MCP client, database connection, credential, filesystem root, project actor, ReferenceStore connection, or context internals cross the public service surface.

## Mapping and selection sequence

```text
immutable ExternalCandidateResultSet
-> exact candidate ID
-> exact UnverifiedProviderLocator
-> owner-specific mapping request
-> exact MappingReceipt
-> explicit SelectionReceipt
-> exact mapped project/reference root
-> existing E3-C context operation
```

The service never silently chooses top-1, first, highest score, sole candidate, same name, same path, same snippet, or provider-labelled exact result.

## Current state

```text
documentation frontier: E6-B
implementation frontier: not-started
next documentation package: E7-A
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

# `wow-reference` E6-B external locator mapping handoff

**Status:** normative supporting seam; implementation not started.

## Purpose

Map one E6-A `UnverifiedProviderLocator` to an exact retained ReferenceView entity under one exact ReferenceProfile/generation. `wow-reference` owns mapping proof and negative authority; `wow-service` coordinates it.

## Port

```text
ReferenceExternalLocatorMappingPort
    validate_mapping_profile
    map_external_locator
    validate_mapping_receipt
```

## Request

```text
ReferenceExternalLocatorMappingRequest
    exact ReferenceProfile/ReferenceGeneration/ReferenceView
    exact external result/candidate/locator IDs and digests
    allowed typed locator fields
    exact mapping profile
    privacy/license/consumer scope
    budgets/cancellation
    OperationId + request digest
```

No provider client, raw MCP request, callback, SQL, source parser, filesystem path, or fuzzy query enters the port.

## Exact signals

A reviewed profile may use owner-verifiable combinations:

```text
exact canonical Reference entity key
exact API/type/event/namespace/member identity
exact profile/build/flavor/Interface compatibility
exact Reference source/content digest when present
exact provider revision/content identity independently bound to the Reference generation
exact source coordinate under the Reference source-map profile
```

Display name or provider “verified/exact” label alone is insufficient.

## Forbidden mapping

- same canonical-looking name without exact profile/entity proof;
- case-folded/fuzzy/FTS/shape/embedding candidate;
- first/only result;
- provider rank/score;
- current/latest Reference generation;
- source implementation used as API contract without Reference authority;
- provider path/URL followed by service/reference code;
- cross-flavor/build/Profile fallback.

## Result

```text
ReferenceExternalLocatorMappingReceipt
    status:
        ExactMapped
        MultipleMappings
        NoMappingWithOwnerAuthority
        NoMappingPartial
        Conflict
        NotEvaluated
        Failed
    exact Reference entity/profile/generation IDs
    comparison/source/evidence/correction refs
    coverage/conflicts/truncation
    privacy/license state
    authority nonclaims
    canonical digest
```

## Authority

`ExactMapped` validates locator correspondence to one Reference entity only. It does not validate provider summary/relationship/recommendation and does not raise the provider Candidate.

A Reference fact included in later context gets its authority from the exact ReferenceView record, not from the external provider or mapping receipt.

`NoMappingWithOwnerAuthority` requires exact profile/entity partition coverage, no conflict/truncation, and an explicit Reference negative-authority record. Empty SQL/lookup is insufficient.

## Generation discipline

A receipt binds one exact ReferenceProfile/generation. It is invalid for another build/flavor/Profile even when names/signatures match. Cross-generation continuity remains E4 lineage, not E6 mapping.

## Security/privacy

No credential/provider transport, filesystem/network/process/editor/client access. Provider text remains bounded untrusted data. Public output follows Reference source/license/privacy policy.

## Tests

- exact canonical entity/profile mapping;
- same API name across incompatible profiles;
- provider exact label without owner proof;
- partial coverage and authoritative no mapping;
- correction/conflict states;
- source implementation locator misused as public API authority;
- malicious URI/name/control text;
- deterministic mapping under record order changes.

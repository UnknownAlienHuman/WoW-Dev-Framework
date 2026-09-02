# `wow-project` E6-B external locator mapping handoff

**Status:** normative supporting seam; implementation not started.

## Purpose

Expose a narrow exact-generation mapping operation from an E6-A `UnverifiedProviderLocator` to one retained project/source entity. `wow-project` owns the mapping algorithm and evidence; `wow-service` owns orchestration.

## Port

```text
ProjectExternalLocatorMappingPort
    validate_mapping_profile
    map_external_locator
    validate_mapping_receipt
```

## Request

```text
ProjectExternalLocatorMappingRequest
    exact project publication/generation/view
    exact external result/candidate/locator IDs and digests
    allowed typed locator fields
    mapping profile ID
    privacy/license/consumer scope
    budgets/cancellation
    OperationId + request digest
```

The port does not receive a provider client, raw MCP request, callback, SQL, filesystem path to open, or arbitrary query language.

## Exact signals

A reviewed profile may require combinations of owner-verifiable signals:

```text
exact materialized repository/source revision already bound to ProjectSnapshot
exact normalized project/source path under the bound inventory
exact content/object digest
exact semantic/source entity ID supplied and independently resolved
exact byte/span correspondence under one coordinate/source-map profile
exact package/universe identity
```

Provider labels are inputs to verification, not authority.

## Forbidden mapping

- first or only same-name symbol;
- path suffix/basename alone;
- line number alone;
- fuzzy/FTS/embedding/search result;
- provider rank/score/exact label;
- repository owner/popularity;
- newest/current project generation;
- opening provider URL/path;
- reading outside the exact retained project view.

## Result

```text
ProjectExternalLocatorMappingReceipt
    status:
        ExactMapped
        MultipleMappings
        NoMappingWithOwnerAuthority
        NoMappingPartial
        Conflict
        NotEvaluated
        Failed
    exact project/entity/source/root IDs
    comparison and evidence records
    project/source inventory and coordinate profile
    coverage/conflicts/truncation
    privacy/license state
    authority nonclaims
    canonical digest
```

## Authority

`ExactMapped` means only that the locator corresponds to the exact project record under the profile. It does not validate provider prose/relations, make the provider Candidate a project fact, prove runtime behavior, or authorize an edit.

`NoMappingWithOwnerAuthority` requires explicit complete relevant project/source coverage and exact negative-authority policy. Empty lookup alone is insufficient.

## Generation and retention

The receipt binds one exact ProjectSnapshot/generation. It cannot be reused for later current. The project view and mapped source/entity must remain retained for any advertised receipt/context handoff.

## Security/privacy

No arbitrary filesystem/network/process/editor/client access. Provider text remains untrusted data. Public receipts use stable IDs and redact private absolute roots/source according to profile.

## Tests

- exact path+digest and entity-ID mapping;
- basename/name/line-only ambiguity;
- same file in another project generation;
- provider revision mismatch;
- partial inventory and authoritative no mapping;
- private source disclosure denial;
- malicious path/URI/control text;
- deterministic results under inventory/query order changes.

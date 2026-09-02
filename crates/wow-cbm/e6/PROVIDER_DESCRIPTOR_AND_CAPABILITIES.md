# E6-A provider descriptors and capability negotiation

**Status:** normative.

## Descriptor trust

A descriptor is repository-owned reviewed canonical data. It defines the maximum permitted provider surface and is pinned by ID/digest. Provider runtime responses cannot authorize new operations, larger limits, executable fields, or stronger authority.

Descriptor fields include:

```text
provider and adapter provenance
allowed transport profile families
allowed typed operation names
request/response schema versions
external-state capability classes
score/rank interpretation and units
pagination/cancellation guarantees
possible locator/snippet fields
privacy/license handling
system limits and canonicalization
```

Forbidden descriptor content:

```text
private endpoint or credential
executable command or shell fragment
arbitrary tool-call permission
raw database path/schema
source-controlled plugin/callback
model prompt
floating latest schema/profile
```

## Negotiation

`negotiate_provider_capabilities` compares one exact descriptor with one exact already-acquired transport/session observation.

Outcomes:

```text
Compatible
CompatibleWithDeclaredLoss
Unsupported
NotEvaluated
Conflict
Failed
```

Unknown operations/schema versions remain unsupported. The intersection of descriptor and runtime capability is active; runtime cannot widen the descriptor.

## Required checks

- provider/adapter/transport identity;
- operation schema/version;
- maximum request/response/item/page/depth limits;
- stable/mutable/opaque state support;
- continuation cursor ownership and replay behavior;
- cancellation and late-response behavior;
- score/rank field type and interpretation;
- locator/snippet/privacy/license field handling;
- unknown-field policy;
- provider error and partial-response semantics.

## Capability drift

A capability change creates a new observation/capability-set identity. An existing query/result/continuation/cache entry remains bound to the old exact capability set and cannot be silently upgraded.

## Provider labels

Marketing or provider labels such as `exact`, `verified`, `authoritative`, `fresh`, `complete`, `semantic`, or `production` are retained only as provider metadata. They do not alter bridge confidence/provenance/coverage.

## Missing negotiation

If runtime negotiation is unavailable, a descriptor profile may permit a strictly smaller static surface with explicit `NotEvaluated` runtime capabilities. It cannot assume support from documentation alone where execution depends on negotiation.

## Security

The capability exchange is bounded and parsed under a closed schema. No dynamic tool discovery, source-provided operation, generic method invocation, executable extension, or environment discovery is accepted.
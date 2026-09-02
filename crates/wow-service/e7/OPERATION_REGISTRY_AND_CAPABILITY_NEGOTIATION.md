# E7-A operation registry and capability negotiation

**Status:** normative.

## Registry purpose

`FrontendOperationRegistry` is the sole bridge from wire-visible methods/tools/commands to service operations. It is immutable, deterministic, versioned, and generated from reviewed descriptors—not runtime reflection or plugin discovery.

Each entry binds:

```text
wire profile and method/tool/command name
stable service operation ID
request/result/error schema digests
required owner capabilities and implementation pins
effect and authorization classes
privacy/license/consumer profiles
budgets, cancellation, continuation and progress behavior
output projection and mandatory nonclaims
```

## Registry construction

```text
load exact reviewed descriptor manifest
-> validate unique service and wire names per profile
-> validate request/result/error schemas and digests
-> validate service implementation capability manifest
-> disable entries with missing or incompatible owner capability
-> validate effect/authorization/exposure policy
-> canonicalize deterministic registry bytes
-> publish immutable registry generation
```

No entry can be activated because a function exists, a model requested it, a provider advertised it, or a client passed an arbitrary operation name.

## Capability states

```text
Available
AvailableWithRestrictions
DisabledByProfile
UnavailableImplementation
UnavailableDependency
IncompatibleProtocol
IncompatibleSchema
NotEvaluated
Failed
```

Only `Available` and explicitly described `AvailableWithRestrictions` entries are advertised. A transport may narrow these further based on client capabilities, consumer/privacy/license policy, exact workspace/profile state, and authorization.

## Negotiation

Negotiation inputs:

```text
protocol profile and requested revision
client implementation/version/capabilities
server operation registry and implementation manifest
consumer/privacy/license policy
workspace/project/profile registration state
transport security profile
```

Negotiation produces one immutable `FrontendCapabilitySet`. It cannot add an unregistered operation, raise authority, suppress mandatory status fields, widen source disclosure, or bypass authorization.

## Schema compatibility

Request/result/error schema IDs and digests are exact. Compatibility is declared by reviewed adapters:

```text
Exact
BackwardCompatibleProjection
ForwardCompatibleUnknownFieldsPreserved
Incompatible
NotEvaluated
```

Unknown fields are rejected by strict request schemas unless a named extension container explicitly permits them. Results never silently discard owner fields; transport projections either preserve them or record loss and are not canonical.

## Existing service operations

An E0–E6 service operation can be projected directly only when:

- one transport request maps one-to-one to it;
- request/result/error schemas are transport-safe;
- exact authorization/effect/privacy profiles permit the frontend;
- no extra owner call, selector resolution, or semantic rewrite occurs in the app;
- any long-running/result-recovery semantics are declared.

Otherwise a dedicated E7-A service use-case operation coordinates the required session/overlay/owner state.

## LSP projection

LSP capabilities are computed from the exact session capability set. Static or dynamic registration may be used only when deterministic and client-supported. The server does not advertise a method before the owner path is usable for the registered workspace/profile.

## MCP projection

The MCP tool/resource set is an allow-listed projection of registry entries. Tool names are stable and unique. The default profile includes read-only operations only. A client cannot request a hidden registry entry by calling a generic method.

## Local daemon projection

The daemon accepts the stable service operation name only after validating it against the session registry. It does not expose arbitrary JSON-RPC method forwarding or plugin namespaces.

## Registry changes

Adding/removing/changing an entry requires:

```text
owner/service operation contract
wire mapping and schema fixture
capability/effect/authorization/privacy classification
compatibility and migration analysis
LSP/MCP/daemon/CLI test updates as applicable
registry generation change
release compatibility manifest update
```

A changed registry never mutates an established session. New sessions negotiate the new generation; existing sessions retain their exact registry or are explicitly closed as incompatible.

## Security hard stops

- no generic `call`, `execute`, `invoke`, `tool`, or raw RPC entry;
- no source-controlled dynamic plugin/tool discovery;
- no command/shell/script/model callback;
- no effecting operation in a read-only exposure profile;
- no missing implementation advertised as available;
- no client capability treated as authorization;
- no wire annotation treated as service evidence.
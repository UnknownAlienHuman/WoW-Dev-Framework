# E6-B provider configuration, credentials, and session acquisition

**Status:** normative.

## Configuration catalog

`ExternalProviderConfigurationCatalogPort` resolves one exact reviewed configuration by ID/digest. It may expose a symbolic configured name only as an outer selector resolved once and recorded before authorization. There is no floating provider default in semantic identity.

A configuration binds:

```text
E6-A descriptor and adapter contract
transport profile and allowed operations
credential-reference class
session acquisition/close/cancellation profile
external-state acquisition policy
privacy/license/consumer policy
resource/quota limits
enabled/revoked/superseded state
```

Unknown, disabled, revoked, superseded, digest-mismatched, or policy-incompatible configurations block dispatch.

## Credential authorization port

```text
ExternalProviderCredentialAuthorizationPort
    authorize exact configuration, credential reference, operation/query purpose,
    consumer, privacy/license profile, and bounded quota
    return nonsecret authorization receipt
```

The port never returns raw credentials. Authorization is checked at use time and may expire or be revoked independently of the configuration.

GitHub, OS, terminal, repository, file, commit, environment-user, or provider-account identity is not credential-use authorization.

## Session factory port

```text
ExternalProviderSessionFactoryPort
    acquire exact session binding from configuration + authorization + policy
    expose one typed E6-A ExternalCandidateTransportPort
    report capability observation, external-state support,
    reconciliation, cancellation, late-response, and close semantics
    reconcile acquisition by exact operation/request identity where effecting
```

The service receives no command line, process handle, socket address, database path, token, environment block, or generic MCP client.

## Allowed session behavior

The session may implement only the E6-A reviewed transport operations permitted by descriptor and capability intersection. Runtime discovery can narrow but cannot add operations.

The adapter, outside semantic service code, may connect to a user-configured process or remote service according to host policy. That implementation detail does not enter canonical candidate identity except through stable adapter/session/profile receipts.

## Acquisition order

```text
resolve exact provider configuration
-> validate descriptor/profile compatibility
-> acquire credential-use authorization
-> register/advance durable operation state
-> acquire exact session
-> negotiate capabilities
-> acquire/bind external state
-> dispatch E6-A operation
```

If any step fails, close acquired resources in reverse order. No provider query is dispatched after cancellation or authorization revocation.

## Session reuse

Reuse is allowed only when the session binding, configuration, authorization, capability set, external-state class, privacy/license profile, and concurrency policy all match exactly. Reuse is operational and cannot change semantic identities or hide mutable/opaque state.

No global singleton session is assumed. Cross-tenant, cross-consumer, cross-credential, or cross-profile reuse is forbidden unless an explicit reviewed profile proves isolation.

## Mutable and opaque state

For `ObservedMutableGeneration`, the session observation receipt is part of the query binding. Reacquiring a new session does not recreate the old state.

For `OpaqueExternalState`, each live query is explicitly nonreproducible. A new session/query cannot be represented as retrying the same observation.

## Close semantics

Session close is mandatory before public success unless the exact host profile returns a retained lease whose lifecycle is owned outside the request and whose release/expiry is fully audited. Close failure is explicit and can change the result to partial, failed, or `OutcomeUnknown`.

No detached cleanup task is spawned.

## Prohibited inputs and outputs

```text
raw token/key/password/cookie
private endpoint or command
arbitrary environment variable name/value
provider database/index path
shell/MCP JSON/tool name
process/socket/client handle
credential-derived account data
```

Default errors/logs expose only stable IDs, state codes, counts, and redacted adapter error references.
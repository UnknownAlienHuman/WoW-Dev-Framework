# E6-B provider session and credential boundary

**Status:** normative.

## Principle

`wow-service` never receives or stores raw provider credentials. It asks a configured authorization/session boundary to create one scoped opaque session for one exact provider descriptor and permitted operation family.

## Ports

```text
ProviderCredentialAuthorizationPort
    validate_profile
    authorize_provider_session
    revalidate_authorization
    revoke_or_close_authorization

ProviderSessionAcquirePort
    validate_adapter
    acquire_session
    negotiate_capabilities
    observe_or_open_external_state
    reconcile_provider_effect
    close_session
```

These are narrow service deployment ports, not generic plugin/tool interfaces.

## Authorization request

```text
ProviderCredentialAuthorizationRequest
    provider descriptor/profile ID
    adapter implementation/profile ID
    requesting service identity
    operation family/scope
    external-state class requested
    privacy/license/consumer profile
    bounded lifetime/use count/replay profile
    OperationId + request digest
```

It contains no password, token, cookie, private key, endpoint secret, database path, or vault handle.

## Authorization decision

```text
Authorized
Unauthorized
Expired
Revoked
ScopeMismatch
ReplayDetected
Unsupported
NotEvaluated
Failed
```

The public receipt contains stable nonsecret references and verification state. Raw credential bytes remain inside the deployment adapter.

## Session receipt

A session receipt binds:

- exact provider descriptor and adapter versions;
- exact authorization receipt;
- negotiated capabilities;
- permitted operations;
- external-state acquisition semantics;
- response-loss reconciliation support;
- privacy/security profile;
- close requirements.

The opaque live session handle is process-local and excluded from canonical/public artifacts.

## Provider descriptors

Descriptors are immutable reviewed data and define:

```text
provider ID/version
adapter ID/version
allow-listed operations
supported external-state classes
capability schema
transport limits
response schema/profile
reconciliation support
privacy/license classifications
security constraints
```

A descriptor cannot contain executable code, arbitrary command names, dynamic tool schemas, raw credentials, or an unrestricted endpoint.

## Capability negotiation

E6-A validates provider capabilities. E6-B additionally verifies that the acquired session exposes the same exact descriptor/profile and the operation-required capability set.

Missing, changed, ambiguous, or partial capabilities produce `NotEvaluated`/failure; service does not probe arbitrary methods or downgrade secretly.

## External-state acquisition

### Stable generation

Session returns an exact provider generation/index/corpus identity and digest under a descriptor-defined verification profile.

### Observed mutable state

Session returns one observation receipt that binds the request/session/time-evidence/profile. All operations and continuation pages must use that exact receipt. A later observation is a different state.

### Opaque state

Session records that reproducibility cannot be established. Only operations explicitly allowed for opaque discovery may proceed; continuation/cache/negative claims are restricted.

## Current and latest

Provider “current” or “latest” is not a semantic selector. A configured adapter may observe provider state once, classify it, and return an exact receipt. Downstream E6-A/B operations bind that receipt rather than resolving current again.

## Reconciliation capability

Before dispatching a potentially effecting provider request, service validates that the adapter provides one of:

```text
provider-native idempotency key and receipt lookup
query-by-operation/request identity
deterministic no-effect proof
explicitly read-only transport with no provider-side mutation
```

If no reconciliation contract exists, the operation profile must either classify the call as strictly read-only or reject retryable exposure.

## Close

Session close is mandatory and auditable. A close failure after useful work prevents public success and returns exact recovery refs. Close never revokes or erases already committed provider effects.

## Credential nonleakage

Forbidden in repository, CLI, request/result JSON, logs, fixtures, source maps, telemetry, errors, and cache keys:

```text
API tokens
OAuth refresh/access tokens
cookies/session secrets
private endpoint credentials
SSH/private keys
KMS/HSM/vault secrets
provider database credentials
raw Authorization headers
opaque live session handles
```

## Identity nonshortcuts

GitHub account, repository owner, OS user, terminal identity, commit author, email, file owner, and current process do not authorize provider access.

## Tests

- valid scoped authorization and session;
- unauthorized/expired/revoked/replayed/scope-mismatch decisions;
- descriptor/session capability mismatch;
- state receipt substitution;
- raw credential in every public surface;
- no reconciliation after dispatch;
- response loss and exact lookup;
- close failure;
- no arbitrary method/tool negotiation;
- 1/2/N concurrency with one exact session policy.

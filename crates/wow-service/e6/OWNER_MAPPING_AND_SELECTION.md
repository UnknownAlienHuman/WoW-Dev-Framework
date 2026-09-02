# E6-B exact owner mapping and explicit candidate selection

**Status:** normative.

## Preconditions

Mapping starts from one exact retained validated E6-A result, one candidate, and one `UnverifiedProviderLocator`. It also requires one exact target owner universe/generation/view and mapping profile.

Service does not derive the target repository, generation, path, entity kind, or owner from provider prose.

## Owner ports

```text
ProjectExternalLocatorMappingPort
ReferenceExternalLocatorMappingPort
```

Each owner receives bounded typed locator fields, exact view/generation, requested mapping classes, privacy/license profile, budget, and cancellation. The owner may compare only fields it owns and must report checked, unchecked, missing, conflicting, and unsupported fields.

## Mapping statuses

```text
ExactMapped
MultipleMappings
NoMappingWithOwnerAuthority
NoMappingPartial
Conflict
NotEvaluated
Failed
```

### ExactMapped

Requires one exact stable owner handle under the mapping profile and validation of every required identity field. Optional provider fields can remain unchecked, but that loss is explicit.

### MultipleMappings

Two or more valid owner handles remain. Service never chooses based on rank, path length, name similarity, first order, newest generation, or popularity.

### NoMappingWithOwnerAuthority

Allowed only when the owner proves complete relevant enumeration/coverage for the exact view and mapping profile and no conflict/truncation exists. It proves no owner record matched the locator criteria; it does not prove the provider candidate or described concept is false.

### NoMappingPartial

Used when no mapping was found but owner coverage, locator fields, capability, or budget is incomplete.

### Conflict

Used for incompatible revision/path/span/digest/entity evidence or conflicting owner records. Conflict is not resolved by majority or preferred field.

### NotEvaluated / Failed

Missing required capability/profile/view yields `NotEvaluated`; owner execution/integrity failure yields `Failed`.

## Mapping publication

Mapping is a durable effect because it creates an immutable audited record consumed by later selection/context operations. Register operation identity before owner invocation; publish and read back the exact mapping record; retain the owner view and evidence required to reproduce the decision.

## Forbidden mapping shortcuts

```text
same repository or addon name
same path basename
same symbol text
same snippet/body
first/only/top/highest-score candidate
current/latest project or reference generation without exact resolution receipt
provider supplied digest without owner byte validation
local filesystem inspection by service
search result substituted for owner mapping
```

## Selection request

A caller explicitly supplies:

```text
result ID
candidate ID
mapping record ID
intended use
context profile when applicable
consumer/authorization profile
decision = Selected | Rejected | Deferred
reason codes and bounded note
OperationId + CanonicalRequestDigest
```

Service validates internal consistency and records the decision. It never invents or changes the decision.

## Selected state

`Selected` for context handoff requires `ExactMapped`, retained owner evidence, compatible privacy/license scope, and no revocation/conflict. A selection on `MultipleMappings`, partial, conflict, or `NotEvaluated` is invalid until the caller supplies an exact resolved mapping through a new request.

## Selection authority

Selection means only that the caller chose this exact candidate/mapping for the declared use. It does not establish:

```text
provider correctness
semantic acceptance
API authority
lineage or replacement
migration safety
static impact
runtime behavior
autofix or edit authorization
core recognizer admission
```

## Supersession and rejection

Selection receipts are immutable. A later decision creates a new receipt linked by explicit supersession. Rejection does not delete the candidate or mapping evidence. Deferred is not rejection or selection.

## Response loss

Mapping and selection effects reconcile separately. If the owner mapping or receipt publication may have completed, return `OutcomeUnknown`; do not rerun or record a second decision until exact reconciliation.
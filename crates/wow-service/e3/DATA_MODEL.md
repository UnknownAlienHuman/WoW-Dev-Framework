# E3-C service data model

**Status:** normative orchestration model. Concrete Rust names may differ; identity order and ownership may not.

## Configuration

```text
ContextServiceConfiguration
    service/schema/output contract versions
    primary project registry/acquisition provider ID
    optional Blizzard UI registry/acquisition provider ID
    ReferenceView provider ID
    wow-context provider/contract ID
    owner-port catalog IDs
    publication-selector policy
    compatibility profile ID
    exact profile-alias registry
    default exact map/L0/L1/intent/expansion/budget/tokenizer/privacy/renderer IDs
    operation budget/cancellation policy
    continuation retention policy
    security/error/output profile IDs
    canonical digest
```

Environment, current directory, editor, WoW installation, terminal, clock, credentials, and floating branches are not semantic configuration.

## Publication selectors

```text
ProjectPublicationSelector
    ExactStoreGeneration(ProjectStoreGenerationId)
    ExactPublicationSet(ProjectPublicationSetId)
    CurrentPublished(ProjectStoreId, ExpectedCurrentGuard?)
```

```text
OptionalPlatformSelector
    Omitted
    ExactStoreGeneration(...)
    ExactPublicationSet(...)
    CurrentPublished(...)
```

```text
ExpectedCurrentGuard
    expected CurrentPublicationRecordId and/or record digest
```

Reference selector:

```text
ReferenceSelectionGuard
    expected ProfileId/ReferenceGenerationId/ReferenceViewId: optional
    selection source = exact selected publication binding
```

## Public requests

```text
ContextStatusRequest
ContextMapServiceRequest
ContextInspectServiceRequest
ContextBuildServiceRequest
ContextContinueServiceRequest
ContextValidateServiceRequest
ContextRenderServiceRequest
```

Common fields:

```text
operation/request schema
primary selector
optional platform selector
reference guard
exact context/profile IDs or configured aliases to resolve
operation-specific exact roots
output/artifact policy
budgets/cancellation
canonical digest
```

Artifact validate/render requests contain bounded bytes/object values and declared media/schema/profile; never a host path.

## Resolution

```text
ResolvedPublicationSelection
    selector kind and safe supplemental metadata
    exact ProjectStoreId/EpochId/StoreGenerationId
    CurrentPublicationRecordId when resolved from current
    ProjectPublicationSetId
    ProjectGenerationId/ProjectSnapshotId/ProjectViewId
    GraphGenerationId/GraphSnapshotId/GraphViewId
    AnalyzerSnapshotId
    ProfileId/ReferenceGeneration binding
    owner validation/capability IDs
    canonical digest
```

```text
ResolvedContextSelectionSet
    primary selection
    optional platform selection
    exact reference selection
    resolution order/profile
    compatibility guard results
    canonical digest
```

No later current pointer enters this set.

## Owner ports

```text
ProjectPublicationAcquirePort
    resolve selector once
    acquire exact immutable published project/graph view set
    validate identity/capability
    close/release

ReferenceAcquirePort
    acquire exact immutable ReferenceView
    validate identity/capability
    close/release

ContextEnginePort
    bind/validate ContextUniverseSet
    execute/validate/render E3-B operations
```

The concrete project acquisition adapter may coordinate public `wow-store`, `wow-project`, and `wow-graph` APIs; raw handles do not cross the port.

## Lease set

```text
ServiceContextLeaseSet
    lease_set_id
    service configuration/request/resolution IDs
    primary PublishedProjectViewLease identity
    optional platform PublishedProjectViewLease identity
    exact ReferenceView lease identity
    exact ContextUniverseSetId
    owner-port and capability catalog IDs
    budget/cancellation state
    operational close stack (noncanonical/private)
    canonical digest excluding operational handles/timing
```

## Invocation plan

```text
ContextServiceInvocationPlan
    plan_id
    operation/request/resolved selection/lease-set IDs
    exact ContextUniverseSetId
    exact roots
    exact context profile set and renderer selection
    ordered owner/context operation calls
    mandatory validation steps
    budget/cancellation allocation
    expected output type
    canonical digest
```

## Context outcome

```text
ContextOperationOutcome
    operation-specific payload:
        ContextStatusPayload
        ProjectMap
        L0Skeleton and/or L1Skeleton
        ContextSemanticPack
        ContextContinuation result
        ContextValidationReport
        RenderedContextArtifact
    exact wow-context artifact IDs
    context status/coverage/conflict/omission/loss/budget/continuation records
    warnings
    validation report IDs
    canonical digest
```

Service does not mutate these payloads.

## Resource closure

```text
ResourceClosureReport
    request/resolution/lease-set IDs
    resources acquired in canonical owner order
    close attempts/results in reverse order
    unresolved operational resource failures
    state: Complete | Failed | CancelledDuringClose
    bounded error records
```

Timing/process/handle data is noncanonical. A failed report blocks a success envelope.

## Result envelope

```text
ContextServiceResultEnvelope
    envelope schema/version
    service result ID
    operation and request ID
    service configuration ID
    selector metadata and ResolvedContextSelectionSetId
    ContextUniverseSetId
    ContextServiceStatus
    exact profile/root/scope IDs
    ContextOperationOutcome
    context artifact/validation/renderer IDs
    lower capability/coverage/conflict/omission/truncation/continuation summaries
    operation budget usage
    safe warnings
    resource closure state/report ID
    producer/tool/schema versions
    canonical digest
```

```text
ContextServiceFailureResult
    operation/request/config IDs
    safely available resolved exact identities
    failed stage/owner/code/arguments
    acquired/released resource summary
    no malformed success payload
    canonical digest
```

```text
ContextServiceCancelledResult
    operation/request/config IDs
    cancellation stage
    safely available exact identities
    acquired/released resource summary
    no published success payload
    canonical digest
```

## Context status

```text
ContextServiceStatus
    complete
    partial
    truncated
    not_evaluated
    failed
    cancelled
```

Operation-specific payloads retain their own precise states.

## Continuation

```text
ServiceContextContinuation
    service continuation ID
    exact E3-B continuation object/token bytes and digest
    original request/universe/profile/artifact IDs
    exact retained store/project/graph/reference generations
    owner retention receipt IDs
    total budget state
    privacy/renderer policy IDs
    canonical digest
```

Expiry/deadline is operational policy metadata unless an exact retained-until field is part of the owner receipt. Continuation is not a current selector.

## Retention receipt

```text
ContextContinuationRetentionReceipt
    exact owner/store/generation
    retention reason/reference ID
    receipt/policy ID
    state
    release/expiry semantics
```

It proves only retention admission, not artifact completeness or authority.

## Identity DAG

```text
configuration + transport-neutral request
-> request ID

symbolic selectors + exact owner records
-> ResolvedContextSelectionSetId

resolved selections + exact acquired views
-> ContextUniverseSetId / ServiceContextLeaseSetId

request + universe + exact profiles/roots
-> ContextServiceInvocationPlanId

wow-context artifacts
-> ContextOperationOutcomeId

outcome + service status + safe summaries + complete closure report
-> ContextServiceResultId
```

Renderer/context artifact identities come from `wow-context`. CLI serialization/exit codes/logs/timings never enter service semantic identity.

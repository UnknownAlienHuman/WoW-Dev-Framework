# E7-A protocol/session data model

**Status:** normative transport-independent model. Concrete Rust names may differ; identity order and ownership may not.

## Protocol profile

```text
ProtocolProfile
    profile ID/version
    protocol family: LSP | MCP
    exact external specification revision
    transport/framing profile
    JSON-RPC/schema compatibility profile
    capability allow-list and required capabilities
    position/encoding/newline profile where applicable
    progress/cancellation/partial-result profile
    authorization/privacy/source-boundary profile
    message/queue/resource limits
    canonicalization and conformance vector digest
```

## Client handshake

```text
ClientHandshake
    protocol profile ID
    client implementation name/version: operational metadata
    declared protocol capabilities
    requested workspace/session capabilities
    transport authentication receipt: optional
    operation authorization context reference
    privacy/consumer trust profile
    canonical normalized capability digest
```

Client name/version/capabilities do not create semantic authority.

## Session

```text
ServiceProtocolSession
    SessionId
    protocol profile and adapter ID
    handshake/capability decision IDs
    authorization context ID
    privacy/budget/telemetry profiles
    state:
        Created
        Initializing
        Active
        Rebinding
        ShuttingDown
        Closed
        Failed
        Quarantined
    current SessionViewSetId: optional
    document overlay set ID: optional
    operation registry/queue references
    closure/audit receipts
```

`SessionId` is operational. It is not a project/reference/graph identity and does not enter owner fact identity.

## Workspace binding

```text
WorkspaceBindingRequest
    SessionId
    explicit workspace identity/root handle(s)
    permitted selector class
    expected project/profile/reference guards
    required capabilities
    authorization/privacy/budget profiles
    cancellation
```

```text
WorkspaceBindingReceipt
    binding ID
    exact normalized roots/owner handles
    resolved retained publication IDs
    compatibility/coverage/conflict state
    acquisition/retention receipts
    source privacy/license state
    canonical digest
```

Paths are normalized owner inputs, not semantic project identity.

## Exact session view

```text
SessionViewSet
    SessionViewSetId
    SessionId and workspace binding ID
    exact ProjectStoreGeneration/ProjectGeneration/ProjectSnapshot
    exact GraphGeneration/GraphSnapshot
    exact AnalyzerSnapshot and annotation artifact bindings
    exact ReferenceProfile/ReferenceGeneration/ReferenceView
    optional Blizzard UI source Project/Graph/SkeletonInput views
    optional external Candidate provider/session/generation binding
    capability/coverage/conflict summaries
    retained owner view/lease receipts
    canonical digest
```

A new rebind creates a new ID; no in-place generation replacement.

## Document identity

```text
ProtocolDocumentIdentity
    normalized protocol URI
    workspace binding ID
    exact owner source/file identity when mapped
    language ID/profile
    encoding/newline/position profile
    privacy/license/source class
    canonical digest
```

A URI is not trusted as a filesystem path. Owner mapping is explicit.

## Document overlay

```text
DocumentOverlayRecord
    DocumentOverlayId
    SessionId/SessionViewSetId
    ProtocolDocumentIdentity
    base source handle/content digest: optional
    protocol document version
    canonical full-content bytes/digest
    edit sequence receipt
    source-map/position profile
    state: Open | Changed | SavedObserved | Closed | Invalid
    coverage/conflicts/diagnostics invalidation refs
    canonical digest
```

```text
DocumentOverlayGeneration
    OverlayGenerationId
    SessionViewSetId
    complete ordered map DocumentIdentity -> DocumentOverlayId
    overlay analyzer/project-view plan and result IDs
    capability/coverage/conflict state
    canonical digest
```

The complete map is immutable; it is not a recursive edit chain for normal reads.

## Service operation

```text
ProtocolServiceOperation
    OperationId
    CanonicalRequestDigest
    SessionId/SessionViewSetId/OverlayGenerationId
    operation kind
    exact root/document/result/continuation IDs
    requested capability/profile/budgets
    authorization decision refs
    state:
        Planned
        Queued
        Acquiring
        Executing
        Serializing
        Closing
        Completed
        Partial
        Cancelled
        Failed
        OutcomeUnknown
    progress/cancellation/retention/closure refs
    canonical digest
```

## Diagnostics result

```text
DiagnosticResultSet
    result ID
    exact session view and overlay generation
    exact project/reference/rule/provider generations
    document/workspace scope
    finding IDs and canonical ordering
    unchanged/related previous result relationship
    capability/coverage/conflict/NotEvaluated state
    truncation/continuation
    canonical digest
```

## Symbol/navigation result

```text
NavigationResultSet
    operation kind
    exact root/source coordinate/entity IDs
    ordered target entities/source locations
    direct/path/relation evidence
    confidence/provenance/coverage/conflicts
    selection/omission/truncation state
    canonical digest
```

## Advisory action

```text
AdvisoryCodeAction
    action ID
    exact finding/root/session view/overlay
    remediation tier and typed plan from owner
    required validation/evidence
    applicability/conflict/NotEvaluated state
    edit/command fields: absent in E7-A
    canonical digest
```

Resolving an action returns more exact advisory data; it does not apply an edit.

## MCP tool/resource descriptor

```text
ProtocolCapabilityDescriptor
    stable method/tool/resource ID and version
    protocol profile compatibility
    static title/description schema
    exact request/result schemas
    mapped service operation or immutable artifact class
    authorization/privacy/budget requirements
    side-effect classification
    canonical digest
```

Descriptions are repository-owned static data, never source/provider-generated executable instructions.

## Progress and cancellation

```text
ProgressReceipt
    protocol request and OperationId
    sequence
    stage enum
    bounded counters/known totals
    partial-result artifact refs when supported
    noncanonical timing fields
```

```text
CancellationReceipt
    request/OperationId
    requested/observed/owner-safe-stop/terminal states
    committed-effect and OutcomeUnknown refs
    closure state
```

## Protocol projection

```text
ProtocolProjectionResult
    exact service result ID/digest
    protocol profile/method/tool/resource ID
    transport request ID: nonsemantic
    projection loss/transformation records
    exact response bytes/digest where frozen
    framing/output/closure state
```

## Identity exclusions

Clock, duration, process/thread IDs, transport connection ID, local socket/pipe name, terminal/editor/host name, progress text, queue depth, retry count, and physical cache/storage layout do not enter semantic owner result identities.

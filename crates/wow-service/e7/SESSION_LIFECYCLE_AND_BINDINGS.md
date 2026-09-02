# E7-A session lifecycle and exact binding generations

**Status:** normative.

## Session lifecycle

```text
Created
-> Initializing
-> InitializedAwaitingReadyNotification
-> ActiveUnbound | ActiveBound
-> ShuttingDown
-> Shutdown
-> Exiting
-> Exited
```

Failure/quarantine can occur from any nonterminal state under the exact profile.

## Initialization

Initialization validates:

- exact protocol/spec/framing/implementation profile;
- client identity metadata under privacy limits;
- client capability declaration;
- immutable server operation registry;
- workspace/root/initialization options under closed schemas;
- credential/source/tool permission exclusions;
- resource budgets;
- duplicate session/initialize state.

It produces one immutable negotiated capability set and session ID. No owner operation executes before initialization and capability validation complete.

## Ready notification

Where the pinned protocol requires a post-initialize notification, the session does not accept ordinary requests until the exact notification receipt closes. Duplicate/out-of-order ready messages are classified explicitly.

## Active unbound versus bound

`ActiveUnbound` supports only operations that do not require project/reference/source/document state, such as protocol status/ping and explicit workspace binding.

`ActiveBound` has one active immutable `SessionBindingGeneration` for admitting new bound requests.

## Initial bind

```text
validate exact workspace/profile selector
-> resolve permitted current selectors exactly once through existing service owner ports
-> acquire compatible project/reference/source publications
-> validate workspace/root/URI/privacy/permission profile
-> create binding generation 1
-> atomically activate for future requests
```

No protocol adapter resolves current.

## Rebind

```text
exact expected active binding ID/digest
+ explicit reason/input transition
-> validate no hidden refresh or incompatible profile substitution
-> create immutable candidate binding N+1
-> validate project/reference/source/document-overlay closure
-> atomically activate N+1 for future request admission
-> retain N for in-flight requests/reconciliation
```

A stale expected binding fails. There is no merge with concurrently changed binding state.

## Binding changes

Bindings can change only through explicit operations:

```text
workspace bind/rebind
server configuration generation change
document open
valid document change
document save policy transition
document close
explicit project/source refresh
```

Progress, request completion, transport retry, cache hit, client capability text, or current pointer changes do not alter a binding.

## Request admission

At admission:

1. validate session active state;
2. resolve protocol method/tool to exact registry entry;
3. capture current active binding ID or explicit unbound state;
4. validate method applicability and exact document/root refs;
5. normalize request params and domain operation identity;
6. create immutable `ProtocolRequestRecord`;
7. dispatch using that binding only.

A concurrent rebind affects later admissions only.

## Binding compatibility

A binding records exact:

```text
workspace roots and normalized URI policy
ProjectId/ProjectGeneration/ProjectSnapshot
ReferenceProfile/ReferenceGeneration/ReferenceView
Blizzard UI/source generation when required
open document overlay manifest
server configuration/profile generation
capability/coverage/conflict state
privacy/license/tool permission profile
```

No same-name/profile/display substitution.

## Multiple workspaces

E7-A can bind a bounded ordered set of workspace folders only if the exact protocol/application profile supports it. Each workspace has a distinct owner project binding and overlay namespace. There is no synthetic distributed atomic generation across independent stores.

A request targeting multiple workspaces must:

- declare exact workspace bindings;
- acquire each retained snapshot explicitly;
- preserve separate generation identities;
- use an operation whose service contract supports multi-workspace input;
- report partial/conflict/closure per workspace.

Otherwise it is rejected.

## Session configuration

Initialization options and explicit reconfiguration use closed server-owned schemas. Changes produce a new configuration generation and binding. E7-A does not read arbitrary editor settings or send mutations back to the client.

Unknown configuration is rejected or explicitly unsupported; no source/client text becomes a profile or command.

## Shutdown

A valid shutdown request:

- changes state to `ShuttingDown`;
- stops admission of new ordinary work;
- handles in-flight requests under the exact drain/cancel policy;
- retains/reconciles effecting domain operations;
- closes document/project/reference/source leases as permitted;
- validates required transcript/audit records;
- returns shutdown response;
- enters `Shutdown`.

Shutdown itself does not terminate the transport where the protocol separates shutdown and exit.

## Exit and EOF

Exit/transport EOF:

- records terminal reason;
- rejects/cancels remaining work under profile;
- preserves `OutcomeUnknown` where effects are uncertain;
- closes overlays, leases, sessions, output resources, audit/transcript writers synchronously;
- emits no further protocol messages after terminal close;
- enters `Exited` or `Failed/Quarantined` with recovery refs.

No detached/background work remains.

## Reconnect

E7-A stdio reconnect creates a new `ProtocolSessionId`. It can explicitly reconcile durable domain operations by exact operation ID/request digest, but cannot reconstruct open documents, request IDs, capabilities, or active binding from guesses.

A future resumable-session contract requires authenticated session tokens, expiry/revocation, replay, state retention, and exact transcript continuity; it is not present now.

## Retention

Older bindings remain retained while referenced by in-flight requests, continuations, progress, responses, audit/transcript records, or unresolved effects. GC cannot remove them until closure.

## Tests

- initialize/ready/active/shutdown/exit happy paths for both protocols;
- duplicate initialize/ready/shutdown/exit;
- request before initialize/ready and after shutdown;
- stale concurrent rebind;
- request remains on old binding after new binding activates;
- same-name project/profile substitution;
- current refresh mid-request/continuation;
- multi-workspace without supporting operation;
- EOF during effect and `OutcomeUnknown`;
- reconnect attempting implicit session resume;
- lease/transcript close failure.

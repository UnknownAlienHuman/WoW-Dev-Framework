# Crate dependency graph

**Status:** normative boundary through documentation frontier E7-A.

Dependencies point toward narrower foundations. Maximum edges do not require activation.

| Crate | Maximum permitted direct dependencies |
|---|---|
| `wow-core` | none |
| `wow-store` | `wow-core` |
| `wow-reference` | `wow-core`, `wow-store` |
| `wow-annotations` | `wow-core`, `wow-reference` |
| `wow-emmy` | `wow-core` |
| `wow-graph` | `wow-core`, `wow-store` |
| `wow-recognizers` | `wow-core`, `wow-emmy`, `wow-graph` |
| `wow-project` | `wow-core`, `wow-store`, `wow-emmy`, `wow-graph`, `wow-recognizers` |
| `wow-rules` | `wow-core`, `wow-reference`, `wow-emmy`, `wow-project`, `wow-graph` |
| `wow-search` | `wow-core`, `wow-store`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-context` | `wow-core`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-cbm` | `wow-core` |
| `wow-service` | reviewed production crates through narrow public contracts |
| applications/transports | `wow-service` only |

## E5 publication slice

```text
apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-project
        ├── wow-graph
        └── wow-recognizers
```

E5-B submissions are independently revalidated, built into distinct core artifacts, attested/signed, published inactive, read back, canaried, rolled out, activated, designated LKG, rolled back/revoked/deactivated, and closed into new project/graph generations. Owner crates never import service/applications and historical generations remain immutable.

## E6 external Candidate slice

```text
wow-cbm -> wow-core

apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-project
        ├── wow-reference
        ├── wow-graph
        ├── wow-context
        └── wow-cbm

host provider/session adapters
    -> narrow E6 service/transport ports
```

`wow-project` and `wow-reference` consume owner-neutral locator projections and do not depend on `wow-cbm`. `wow-context` receives only an exact mapped owner root and never provider metadata as semantic input. `wow-store` interprets no provider/mapping/selection semantics.

## E7-A frontend slice

```text
apps/wow
    -> wow-service
        ├── exact owner crates required by the registry entry
        └── wow-store for generic durable/session/journal state

LSP/MCP clients/editors/models
    -> protocol messages only
    -> apps/wow transport host
    -> wow-service
```

There is no `wow-lsp-core`, `wow-mcp-core`, or editor-specific semantic crate by default. Transport-neutral registry/session/request/result types live in `wow-service`; protocol framing and process/endpoint behavior live in `apps/wow`.

### Owner-neutral overlay seams

```text
apps/wow protocol message
-> wow-service session/workspace/document operation
-> wow-project explicit workspace and immutable overlay owner
-> wow-emmy overlay analyzer input/result
-> wow-rules/search/context/reference/graph owner results
-> wow-service transport projection
```

Owner crates do not import LSP/MCP/daemon/application types. They use existing core/source/result types or owner-neutral E7 input records.

### Store seam

`wow-store` may persist exact registry generations, bounded session metadata, operation tickets, response delivery journals, leases and retention edges. Unsaved overlay bodies remain memory-only by default. Store does not interpret protocol, workspace, diagnostic or editor semantics.

## Authority boundaries

```text
transport capability advertisement != owner implementation proof unless registry validated
client/editor/model identity       != semantic or effect authorization
wire request ID                    != durable OperationId
workspace root                     != trusted project
unsaved overlay                    != saved ProjectGeneration
progress                           != completion
response delivered                 != operation semantics
MCP tool annotation                != authorization
LSP WorkspaceEdit                  != permission to apply
```

The exact service/owner result remains authoritative. Transport projections cannot raise confidence, create negative authority, hide blockers or change effect state.

## Protocol boundaries

```text
LSP 3.18 stdio
    -> standard lifecycle/document/language projections
    -> one service operation per semantic request

MCP 2025-11-25 stdio
    -> fixed read-only tools and exact resources
    -> one service operation per tools/call

wow-local-jsonrpc/1
    -> current-user local IPC
    -> operation names validated against exact session registry

local MCP Streamable HTTP
    -> loopback only, explicit, authenticated, Origin-validated,
       disabled by default
```

No remote/public service, generic tool/RPC forwarding, provider proxy, shell, script, model sampling, editor setting mutation or automatic source edit enters E7-A.

## Distinct state/authority axes

```text
owner semantic result
frontend capability availability
session/workspace/document/overlay state
durable effect state
transport delivery state
progress/cancellation state
privacy/license/authorization
release/support status
runtime correctness
```

No edge collapses these states.

## Response loss and isolation

A disconnect does not cancel. `OutcomeUnknown` blocks blind retry. Durable results can be replayed by exact authorized lookup without reexecution. Session/workspace/overlay/source/authorization/operation/result/journal visibility is consumer-scoped.

## Next E7-B boundary

Release tooling may consume built `apps/wow` binaries and exact data packs after all implementation gates. It cannot import owner crates as a semantic runtime or bypass service contracts. Public packaging/signing/update artifacts are a separate release layer, not a new authority source.

## Forbidden patterns

- owner crate depending on service/application/protocol crates;
- application/transport importing any framework crate except `wow-service`;
- `wow-core` gaining transport/editor/provider semantics;
- `wow-store` interpreting project/provider/protocol/diagnostic semantics;
- editor-specific analyzer, graph, search or context forks;
- runtime reflection or generic `call_service`/MCP/RPC/tool proxy;
- capability advertisement without exact implementation/profile availability;
- implicit cwd/Git/editor/WoW workspace discovery;
- provider/source text becoming registry, command or authorization;
- stale document changes applied best-effort;
- raw filesystem/source/provider/session/process handles crossing service seams;
- default remote listeners, arbitrary shell/process execution or editor-setting mutation;
- effecting MCP tools in the default profile;
- progress/delivery treated as semantic completion;
- response replay causing a second effect;
- cross-client overlays/source/results;
- public release workflow before executable and E7-B gates.

Changing an edge requires exact crossing data/operation, insufficiency of current seam, cycle/identity/security/privacy/license/evidence analysis, tests/mutations, migration notes and manifest/workstream updates.
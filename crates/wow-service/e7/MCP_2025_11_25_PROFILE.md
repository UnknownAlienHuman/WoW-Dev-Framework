# E7-A Model Context Protocol 2025-11-25 profile

**Status:** normative supported profile; implementation has not started.

Official protocol reference: <https://modelcontextprotocol.io/specification/2025-11-25/>

## Supported transports

```text
mcp-stdio-2025-11-25-v1                  mandatory first profile
mcp-streamable-http-local-2025-11-25-v1 optional, explicit, disabled by default
```

The stdio profile exchanges individual UTF-8 JSON-RPC messages under the pinned MCP transport and writes only MCP messages to stdout. Logs use stderr.

The local Streamable HTTP profile binds loopback only, validates `Origin`, authenticates the exact session, enforces protocol/session headers, and has bounded POST/GET/SSE behavior. Remote/public HTTP hosting is outside E7-A.

## Lifecycle

The client initializes with protocol revision `2025-11-25`. The server returns only capabilities and implementation metadata that are actually available. Requests before initialization are rejected except protocol-permitted health behavior.

Disconnect is not cancellation. Session resumption/replacement never transfers workspace, authorization or private resources without exact session policy.

## Initial server capabilities

```text
tools      fixed allow-listed read-only or analysis-only set
resources  exact immutable service artifacts under consumer policy
logging    bounded redacted messages
progress   bounded nonauthoritative notifications
cancellation
```

Not supported initially:

```text
prompts
sampling
elicitation
task-augmented execution
server-requested roots
arbitrary completion extensions
```

The server makes no model call and does not ask the client to make one.

## Fixed tool projection

The default registry can expose only implemented non-mutating analysis operations such as:

```text
wow.status                              -> frontend_status
wow.check                               -> check
wow.search                              -> search_query
wow.search_explain                      -> search_explain
wow.context_build                       -> context_build
wow.context_continue                    -> context_continue
wow.context_inspect                     -> context_inspect
wow.lineage_explain                     -> lineage_explain
wow.migration_validate                  -> migration_validate
wow.static_impact                       -> impact_run
wow.external_candidate_result_get       -> external_candidate_query_get
wow.external_candidate_mapping_get      -> external_candidate_mapping_get
wow.external_candidate_selection_get    -> external_candidate_selection_get
```

Actual availability is implementation/profile-gated. `context_build`/`context_continue` may create retained analysis artifacts but cannot mutate user source, project configuration, provider state, core-pack state or release state; their exact durable local effect class remains declared in the registry.

There is no generic `wow.call`, arbitrary method, shell command, source edit, provider tool proxy or nested workflow runner.

Effecting operations—including workspace/project registration from a filesystem root, overlay mutation, provider dispatch, candidate selection recording, review/holdout access, pack publication, activation, rollback, source edits, release publication and update installation—are absent from the default MCP list. A future effecting profile requires separate method-specific authorization, user confirmation, audit and contract.

## Tool request/result mapping

Each `tools/call` maps to exactly one service operation. Arguments are strict JSON Schema objects; unknown fields fail. The service result is returned in `structuredContent` under the exact output schema. Faithful JSON text may accompany it for compatibility but cannot omit fields or rewrite status.

Protocol/argument/authorization failures use JSON-RPC or MCP tool errors as appropriate. A completed domain result such as `Invalid`, `Blocked`, `NotEvaluated`, zero candidates or Candidate-only evidence remains a structured service outcome rather than a transport crash. `Failed` and unresolved `OutcomeUnknown` set the tool error indication while preserving the exact service envelope.

Tool annotations are descriptive and cannot authorize execution or alter effect class.

## Resources

Exact resource URIs can expose retained immutable artifacts permitted to the session:

```text
wow://profiles/<profile-id>
wow://projects/<project-generation-id>/map
wow://results/<result-id>
wow://context/<context-artifact-id>
wow://diagnostics/<result-id>
wow://publications/<publication-id>
```

`resources/list` is snapshot-bound and consumer-scoped. `resources/read` resolves the exact ID; there is no floating `current`, filesystem passthrough or arbitrary source read. Resource subscriptions are disabled initially unless an exact invalidation contract is added.

Full source resources require explicit L2/privacy/license authorization and stable source handles, never raw provider paths. Private unsaved overlays are not resources by default.

## Project selection

The MCP server does not request client roots or infer a project from host cwd. A project/profile must already be explicitly registered through approved CLI/daemon/session setup. Tools accept exact registration/project/profile IDs.

A future roots-enabled profile treats returned roots as untrusted registration candidates and still requires one explicit service registration operation with user consent.

## Streamable HTTP security

The optional local HTTP profile:

- listens only on an explicitly supplied loopback address;
- rejects non-loopback binding;
- validates allowed Origin values and returns 403 for invalid origins;
- uses secure unpredictable MCP session IDs;
- requires exact protocol-version/session headers;
- authenticates each request under the local session profile;
- bounds POST bodies, SSE streams, concurrent connections, event queues, replay windows and session lifetime;
- prevents DNS rebinding and cross-session replay;
- never places credentials/session secrets in URLs or logs.

## Progress, cancellation and resumption

MCP progress tokens map to the service operation ticket. Progress cannot reset an absolute timeout indefinitely. Cancellation targets the exact operation. SSE reconnection/redelivery is transport delivery only and cannot duplicate a service effect.

## Security and trust

Tool/resource descriptions, client metadata, model text, roots and annotations are untrusted. They cannot alter the registry, authorize effects, widen source disclosure or become agent instructions. The host remains responsible for user-visible consent; the server never assumes a model invocation is user authorization.

## Nonclaims

MCP availability does not make the framework autonomous, permit edits, validate model conclusions, establish source truth or authorize effects.
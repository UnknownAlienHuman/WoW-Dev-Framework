# E7-A MCP tool and resource mapping

**Status:** normative initial non-mutating profile for MCP revision 2025-11-25.

## Fixed tools

The concrete registry is generated only from implemented service operations. Initial names and exact mappings are:

| MCP tool | Service operation | Effect class |
|---|---|---|
| `wow.status` | `frontend_status` | pure read |
| `wow.check` | `check` | analysis read |
| `wow.search` | `search_query` | analysis read |
| `wow.search_explain` | `search_explain` | pure read over retained result |
| `wow.context_build` | `context_build` | durable analysis artifact; no user-source mutation |
| `wow.context_continue` | `context_continue` | durable analysis artifact; no user-source mutation |
| `wow.context_inspect` | `context_inspect` | pure read |
| `wow.lineage_explain` | `lineage_explain` | pure read over retained lineage |
| `wow.migration_validate` | `migration_validate` | nonapplying validation |
| `wow.static_impact` | `impact_run` | analysis read/artifact |
| `wow.external_candidate_result_get` | `external_candidate_query_get` | pure read over retained result |
| `wow.external_candidate_mapping_get` | `external_candidate_mapping_get` | pure read |
| `wow.external_candidate_selection_get` | `external_candidate_selection_get` | pure read |

A tool is omitted when its exact operation is unimplemented, disabled or unauthorized. Names never alias arbitrary service methods.

## Not exposed by default

```text
workspace/project registration from arbitrary roots
document overlay mutation
provider query dispatch
candidate selection recording
calibration review/holdout
core-pack signing/publication/activation/rollback
source edits or migration application
release publication/update installation
arbitrary command/tool/RPC/shell/model execution
```

A future effecting profile must define one method-specific tool, authorization/confirmation, idempotency, audit and exact output contract. It cannot enable all service operations generically.

## Tool schema and output

Each descriptor binds exact input/output schema IDs/digests from `FrontendOperationRegistry`. Unknown arguments fail. `tools/call` invokes exactly one service operation.

`structuredContent` contains the exact service projection. Compatibility text is generated from the same result and cannot omit blockers, coverage, Candidate state, `NotEvaluated` or `OutcomeUnknown`. Tool annotations are descriptive only.

## Resources

| URI shape | Owner/result |
|---|---|
| `wow://profiles/<profile-id>` | exact profile manifest |
| `wow://projects/<project-generation-id>/map` | exact Project Map |
| `wow://results/<result-id>` | exact retained service result |
| `wow://diagnostics/<result-id>` | exact retained diagnostic result |
| `wow://context/<context-artifact-id>` | exact context artifact under source policy |
| `wow://publications/<publication-id>` | exact internal publication metadata under policy |

`resources/list` is snapshot-bound and consumer-scoped. `resources/read` resolves exact identity. Floating `current/latest/best`, raw filesystem paths, provider URIs, credentials and traversal are invalid.

Full source is not a default resource. A future exact source resource requires an owner stable source handle and L2/privacy/license authorization. Unsaved overlays are private and excluded by default.

## Initialization capabilities

```text
tools
resources
logging
progress
cancellation
```

Prompts, sampling, elicitation, tasks and server-requested roots are omitted. The app makes no client/model request for them.

## Local Streamable HTTP

The same fixed registry applies; HTTP adds no tools/resources. The app validates loopback bind, Origin, protocol/session headers, authenticated session scope, body/stream/replay limits and cross-session isolation before service dispatch.
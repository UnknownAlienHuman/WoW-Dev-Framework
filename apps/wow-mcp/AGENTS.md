# AGENTS.md — `apps/wow-mcp`

## Dependency

```text
apps/wow-mcp -> wow-service
```

Any direct framework dependency or provider/database/source access is an architecture failure.

## Responsibilities

- implement the exact frozen MCP transport/framing/profile;
- negotiate the supported protocol/capabilities;
- expose only repository-owned fixed tools/resources;
- validate all tool/resource arguments under closed schemas;
- map each tool call to exactly one `wow-service` operation;
- map each resource read to one exact retained service/artifact read;
- preserve cancellation, progress, status, authority, privacy, and source boundaries;
- emit exact MCP results/errors and close cleanly.

## Prohibited behavior

- no generic `call_tool`, arbitrary JSON-RPC owner dispatch, dynamic source/provider tools, or wildcard resources;
- no sampling, elicitation, model completion, prompt execution, or hidden agent loop;
- no arbitrary file/URL/repository/provider locator read;
- no source edit, shell, network, process, editor, WoW client, publication, or activation;
- no current/latest/best/top/sole candidate selection;
- no authorization inferred from MCP client/host approval, OS, GitHub, file, process, or repository identity;
- no background work after response, cancellation, shutdown, or transport loss.

## Candidate discipline

- external provider results remain `Candidate`;
- exact owner mapping remains separate;
- explicit selection receipt is required before context handoff;
- rank/score/snippet/provider labels never become framework facts;
- zero results never become negative authority.

## Resource discipline

- use opaque exact-generation framework resource URIs only;
- never dereference arbitrary paths or URLs;
- validate authorization/privacy/license/retention on every read;
- source excerpts use the structural untrusted-data boundary;
- do not expose raw stores, owner handles, credentials, hidden holdout/review data, or private roots.

## Completion report

```text
MCP/spec/profile/transport IDs
client and server capabilities
session/workspace/view IDs
static tool/resource descriptor IDs
tool/resource -> service/artifact mapping
candidate/mapping/selection/context state
cancellation/progress/backpressure/shutdown result
authorization/privacy/source-boundary result
protocol conformance/security/determinism tests
explicit unsupported capabilities
```

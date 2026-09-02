# E7-A MCP security and explicit-input contract

**Status:** normative.

## Input boundary

The adapter accepts only messages defined by the exact MCP profile over the configured stdio transport. It does not discover or read cwd, home, environment, Git, repositories, editor state, WoW installation, SavedVariables, logs, provider databases, credential stores, or network endpoints for semantic behavior.

## Closed schemas

- Reject unknown tool/resource names and unsupported methods.
- Validate exact descriptor version and input schema.
- Reject unknown fields unless the frozen MCP compatibility profile explicitly permits forward-compatible ignoring.
- Bound JSON bytes/depth/arrays/strings/numbers and reject duplicate keys/nonfinite values/invalid Unicode.
- Do not accept callbacks, expressions, scripts, SQL, regex programs, shell, templates, includes, plugin names, model prompts, or arbitrary owner operation IDs.

## No generic dispatch

Forbidden:

```text
call_tool(name, arbitrary_json)
call_service(operation, arbitrary_json)
raw JSON-RPC pass-through
raw MCP provider pass-through
source/provider-defined tool descriptor
dynamic tool installation or code loading
```

Every tool is a compiled/repository-owned descriptor mapping to one service operation.

## Resource safety

- Opaque `wowdev://` exact IDs only under the frozen URI profile.
- No arbitrary file/HTTP/repository/provider URI.
- No path traversal, globbing, symlink/reparse/device/UNC/ADS resolution.
- No following source-embedded links.
- No raw database/store/WAL/object-root resource.
- Bound list/read results, source excerpts, context bytes, and continuation pages.

## No model loop

Sampling, elicitation, prompt execution, model completion, embedding/reranking, autonomous planning, and automatic follow-up tool calls are absent. The server cannot ask a client model to decide authority, choose a candidate, authorize an effect, or fill missing evidence.

## Authorization and credentials

Client/host/model/OS/GitHub/process/file identity is not operation authorization. Private keys, bearer tokens, cookies, provider credentials, signing material, vault/KMS/HSM secrets, and raw authorization handles are never ordinary tool arguments, resources, fixtures, logs, or results.

## External providers

Provider access occurs only through the existing E6-B service credential/session ports. The MCP app never connects to providers, opens databases, interprets locators, or retries uncertain effects directly.

Provider strings remain Candidate data. Zero results are not negative authority.

## Source and prompt injection

All source/provider/query/review text is structurally data. It cannot:

- define tools/resources/capabilities;
- close JSON/protocol/source boundaries;
- request automatic tool calls;
- change authorization/profile/budgets;
- create paths/URLs/commands;
- raise confidence or override service results.

Structured source-data rendering and JSON escaping are mandatory.

## Session isolation

Every call/read/cancel validates exact session and view ownership. Cross-session result/resource/cursor use is rejected unless an explicit shareable retained-artifact profile permits it. No session resumption or multi-tenant network mode in v1.

## Resource limits

Bound frames/messages, concurrent calls, queues, tools/resources listed, arguments, roots/IDs, search/context/reference/source results, progress/partial chunks, output bytes, wall/CPU/memory, and shutdown/closure work.

Slow clients receive backpressure/close; the server never buffers without limit or silently drops responses.

## Effects and output failure

The initial profile exposes no source edits or core/release publication. For any explicitly mapped existing effect, service idempotency/reconciliation remains authoritative.

Broken pipe/output failure never repeats the tool/service operation. Committed effects retain actual state and may be `OutcomeUnknown` until reconciled.

## Logging and telemetry

Use stable IDs, method/tool/resource class, status, bounded counts, and stages. Exclude raw arguments, source/provider snippets, private roots, credentials, hidden review/holdout data, authorization material, owner handles, and unrestricted stacks by default.

## Adversarial corpus

- malformed/deep/duplicate-key/polyglot JSON-RPC;
- unknown/dynamic/generic tools and resources;
- source/prompt/tool-chain injection strings;
- arbitrary paths/URLs/repositories/provider locators;
- forged/cross-session resource IDs and continuations;
- top/best/sole candidate selection attempts;
- sampling/elicitation/model requests;
- credential/secret payloads;
- queue flood/slow reader/broken pipe/EOF;
- cancellation/response-loss/shutdown races;
- direct lower-crate/provider/database access mutation.

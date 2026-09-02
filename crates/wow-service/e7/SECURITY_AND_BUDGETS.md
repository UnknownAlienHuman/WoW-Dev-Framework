# E7-A protocol security and resource budgets

**Status:** normative.

## Trust boundary

Treat as untrusted:

- all protocol frames/messages/IDs/arguments/capabilities;
- workspace/document URIs and source text;
- MCP tool/resource arguments and provider text;
- diagnostic/query/context strings and source comments;
- client implementation metadata;
- cached/progress/continuation data until validated.

Repository-owned protocol descriptors/profiles are trusted only after exact ID/digest validation.

## Prohibited surfaces

E7-A does not expose:

- raw SQL/database/store handles;
- arbitrary filesystem, URL, Git, repository, network, process, shell, editor, clipboard, environment, or WoW client access;
- source/addon/XML/Lua/script/plugin/Wasm/native execution;
- model/embedding/reranker/sampling/elicitation calls;
- generic JSON-RPC owner dispatch or MCP `call_tool` passthrough;
- source/provider/client-controlled method/tool/resource registration;
- arbitrary LSP execute-command or workspace edits;
- implicit project/current/provider discovery;
- credentials, keys, tokens, cookies, vault/KMS/HSM material;
- unbounded source, graph, references, context, diagnostics, progress, or telemetry;
- remote listener/daemon/multi-tenant behavior in the initial profile.

## Message validation

Bound and validate before dispatch:

```text
frame/header/line lengths
JSON bytes/depth/keys/arrays/strings/numbers
duplicate keys and invalid Unicode
request/notification/batch counts
request ID type/length/uniqueness
method/tool/resource identifier and version
capability arrays/objects
URI bytes/scheme/normalization
workspace roots and document counts
document/change/range/text sizes
arguments/results/errors/progress/partial-result bytes
```

Batch requests are unsupported unless the exact protocol profile implements and tests them.

## URI and path safety

- URI syntax is parsed under the frozen profile.
- File-like URIs map through explicit workspace/project owner ports.
- Reject unauthorized scheme, host, traversal, NUL/control, device, UNC, alternate data stream, symlink/reparse escape, or normalization collision under the platform profile.
- MCP framework resource URIs are opaque exact-generation identifiers; they are not dereferenced as arbitrary URLs or paths.
- Provider locators remain E6 Candidate data and are never opened by E7 adapters.

## Source/prompt injection

Structural separation, not intent detection, provides the boundary:

- protocol control fields use closed schemas;
- source/query/provider/review text remains typed data;
- repository-owned static templates/descriptors define rendering;
- source text cannot create methods, tools, resources, options, authorization, operations, output paths, or commands;
- context/source excerpts use the existing source-data boundary;
- no downstream model permission is implied.

## Session isolation

- Every request validates session ownership and exact view/overlay identity.
- No cross-session document/result/continuation/resource access without an explicit shareable retained artifact profile.
- Document versions and overlays are session scoped.
- Authorization/privacy/cache keys bind the session consumer profile.
- Shutdown/revocation closes session resources.
- One session cannot cancel another’s operation.

## Resource budget profile

```text
max sessions/transports
max active requests per session/global
max inbound/outbound queue bytes/items
max frame/message/JSON bytes and depth
max workspaces/documents/overlay generations
max document/change/edit bytes/lines/ranges
max owner calls/query depth/fanout/results
max diagnostics/references/symbols/actions/context/source excerpts
max progress/partial chunks and continuation pages
max retained result/overlay/session duration or explicit lease policy
max wall/CPU/memory/serialization/closure work
```

Client values can narrow but not exceed system maxima. Zero/unlimited/overflow values are invalid where inappropriate.

## Denial of service

Defenses include:

- parse limits before allocation;
- bounded queues and admission control;
- per-session/global concurrency and work quotas;
- cycle-safe graph/context operations owned by lower layers;
- per-document mutation serialization;
- cancellation/shutdown priority;
- no unbounded recursive JSON or edit chains;
- deterministic truncation/continuation only for operations that permit partial output;
- no expensive fallback after capability failure;
- no provider/model fallback.

## Slow peer and broken transport

- Outbound queues are bounded.
- Slow readers trigger backpressure/typed close, not unbounded buffering.
- Broken pipe/socket/stdio output does not rerun service work.
- Lost response after an effect is reconciled by `OperationId`, never guessed absent.
- Framing desynchronization closes the transport under profile.

## Authentication and credentials

Deployment supplies an authenticated port/session. Protocol messages and initialization options do not carry normal private credentials. Public audit/results use nonsecret references.

Stdio initial profiles may rely on process launch containment for transport reachability, but operation authorization remains explicit. This does not authorize source edits or other effects.

## Output confidentiality

Default errors/logs/progress omit:

- document/source/query text;
- private absolute roots and workspace names;
- credentials/tokens/signatures/cookies;
- hidden review/holdout data;
- provider private endpoints/locators where restricted;
- raw owner/store/session handles;
- unrestricted stack traces.

Protocol output returns only fields allowed by owner + authorization + privacy/license + capability profile.

## Advisory actions

E7-A actions contain no executable command or applied edit. Any source edit in a later profile requires independent authorization, exact base/content guards, owner validation, preview/diff, atomic write, response-loss, rollback, and audit contracts.

## Conformance and fuzzing

Before implementation completion:

- official protocol conformance vectors;
- malformed framing/header/JSON/UTF/URI/request IDs;
- duplicate keys and unknown fields;
- capability downgrade/conflict;
- huge/deep/polyglot payloads;
- document edit bombs and Unicode ranges;
- cross-session/overlay/cursor substitution;
- dynamic tool/method/resource injection;
- raw path/URL/SQL/script/tool/model attempts;
- source prompt/control strings;
- authorization/revocation/replay;
- queue/slow-peer/broken-pipe/disconnect/shutdown races;
- cancellation around every owner/effect/output boundary;
- privacy/log redaction;
- 1/2/N-worker determinism.

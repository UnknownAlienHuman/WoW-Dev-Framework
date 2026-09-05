# `wow-cbm` implementation contract

**Status:** deferred to E6; contract scaffold only.

## Mission

`wow-cbm` is an optional, replaceable MCP client bridge to an already installed/configured Codebase Memory service. It obtains broad semantic/source candidates and generic trace candidates, converts them into bounded generation-scoped records, and never participates in exact WoW authority without independent verification.

## Owned responsibilities

- explicit bridge configuration and capability handshake;
- standard MCP transport lifecycle;
- repository/revision identity negotiation;
- optional `ensure_index` request;
- semantic candidate and generic trace queries;
- response validation, size/time budgets, cancellation, and errors;
- external generation/coverage identity;
- candidate normalization and stable source-handle conversion where resolvable;
- cache keys tied to external generation/query digest;
- bridge health/status reporting;
- measured CBM-on versus CBM-off evaluation inputs.

## Explicit non-responsibilities

`wow-cbm` does not:

- install, update, patch, vendor, or own Codebase Memory;
- read or write its SQLite/internal storage;
- generate fake Lua files to influence its parser;
- treat generic `CALLS` edges as proven WoW call facts;
- assign platform/project `Proven` confidence;
- rank final local/reference search results;
- make exact search depend on bridge availability;
- discover arbitrary commands from source repositories;
- execute external repository code;
- retain full external source in framework release artifacts.

## Trust boundary

The configured MCP process is an untrusted peer. Its response may be stale, oversized, malformed, incorrectly scoped, or contain source text designed as prompt injection. The adapter assigns evidence class; it never accepts authority labels from response prose.

## Required operations

| Operation | Required behavior |
|---|---|
| `configure_bridge` | Accept only explicit command/transport/root/time/budget configuration from trusted project/user config. |
| `start_bridge` | Start/connect under bounded environment and return typed unavailable state on failure. |
| `handshake_capabilities` | Record server/version/tools/schema and unsupported operations. |
| `ensure_index` | Request index for one repository identity/revision and return external generation/coverage. |
| `semantic_candidates` | Query one explicit repository scope and return bounded candidate records. |
| `trace_candidates` | Query generic relation candidates from a stable seed; preserve candidate status. |
| `bridge_coverage` | Report indexed roots/revision/languages/partial or stale state. |
| `normalize_candidate` | Validate repository/revision/path/span/symbol/score and strip unsupported authority claims. |
| `resolve_candidate_handle` | Convert only registered repository/revision/path/span candidates to a stable handle. |
| `cache_candidate_response` | Cache by server/external generation/query/scope/budget; stale generations remain labeled. |
| `cancel_request` | Cooperatively cancel and discard late/oversized responses. |
| `shutdown_bridge` | Release process/transport without mutating external storage. |
| `bridge_status` | Return configured/available/generation/coverage/last-error without affecting local capability status. |

## Candidate record

A normalized record includes:

```text
external repository identity and exact revision
external index generation/server version
path and span/symbol when supplied
candidate kind and raw normalized score
query lane/operation
coverage/staleness
source text digest when content is returned
provenance = semantic_candidate
confidence = Candidate
resolution status to StableSourceHandle
```

Unresolved repository/path identity remains a candidate record and cannot enter exact graph impact or autofix paths.

## Failure behavior

When unconfigured, unavailable, stale, or timed out:

- local/reference/project capabilities remain available;
- service reports the semantic lane as unused/unavailable;
- cached results retain old generation/stale status;
- no exact search request fails solely because CBM is absent;
- no empty CBM result becomes an authoritative negative;
- malformed responses are rejected with bounded diagnostic detail.

## Security and privacy rules

1. No generic shell API; only configured executable/transport plus defined MCP operations.
2. Strip unnecessary environment secrets from child process configuration.
3. Enforce request/response byte, item, time, and recursion limits.
4. Keep filesystem resolution inside registered roots.
5. Treat returned source/comments as quoted evidence, never instructions.
6. Do not log private repository URLs, tokens, local paths, or full source by default.
7. External repositories remain read-only and their hooks/scripts are not executed.
8. License/revision metadata is required before retaining source-derived fixtures.

## E6 implementation sequence

1. capability/status interface with an unavailable stub that is explicit, not fake success;
2. standard MCP transport and handshake;
3. repository identity and `ensure_index`;
4. semantic candidates;
5. trace candidates;
6. source-handle normalization;
7. caching/generation/cancellation;
8. service merge with evidence separation;
9. CBM-on/off evaluation;
10. optional DerivedFacts interoperability experiment only after local schemas stabilize.

## Required tests

- unconfigured bridge leaves local workflow intact;
- handshake success/unsupported/malformed/timeout;
- repository revision mismatch;
- response size/item/time limits;
- cancellation and late response discard;
- candidate cannot set `Proven`;
- unresolved versus resolved source handle;
- stale cache generation labeling;
- source prompt-injection text remains data;
- no direct database access;
- no external process discovery from repository content;
- CBM-on/off normalized evaluation and candidate verification cost.

## Documentation sources

- [`../../docs/CODEBASE_MEMORY_BRIDGE.md`](../../docs/CODEBASE_MEMORY_BRIDGE.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [`../../docs/GRAPH_SEARCH_AND_PLANNING.md`](../../docs/GRAPH_SEARCH_AND_PLANNING.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

## Definition of done

E6 is complete when the bridge can add measurable, bounded semantic/source candidates while remaining completely optional, generation-scoped, non-authoritative, safe against malformed/untrusted responses, and unable to mutate Codebase Memory internals.

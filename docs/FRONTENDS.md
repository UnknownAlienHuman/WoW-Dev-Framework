# CLI, editors and agents

CLI, LSP and MCP are thin adapters to the same Service owners. They do not create
parallel analyzers, Project publication state, result hierarchies or lifecycle
platforms. General Lua language intelligence remains upstream EmmyLua.

Read [current status](CURRENT_STATUS.md) for delivery state and
[the queue](REFACTOR_QUEUE.md) for implementation order. This guide defines the
stable interface contract only.

<a id="current-checkpoint"></a>
<a id="target-user-operations"></a>
<a id="configuration-and-check-input"></a>

## CLI

CLI is the complete offline path. The first usable command is a narrow `LuaTypes`
vertical slice, not the complete Blizzard loader:

```text
wow check
  --root <addon-dir>
  --toc <explicit.toc>
  --flavor retail
  --library <dir> [--library <dir> ...]
  [--config <wowdev.json>]
  [--format text|json]
  [--fail-on error|warning|information|hint]
```

Later slices may add auto-selection, target Interface, product channel,
dependency roots and selected WoW checks. They must not delay the explicit
physical-Lua command.

CLI precedence and path bases are defined in
[ordinary configuration](IMPLEMENTATION.md#ordinary-configuration).

<a id="check-results-and-command-status"></a>

Text and JSON render the same typed result:

- selected scope and completeness;
- diagnostics with exact code, message, severity and user-source range;
- scoped limitations with optional exact source location.

The direct public result contains no snapshot, analyzer, Project generation,
report, proof or component/capability identities. Paths are canonical relative to
the configured addon root; text rendering may show `root/path`, while JSON keeps
the relative identity the caller supplied with the root. Ambient absolute paths
never enter snapshot/result identity.

Malformed configuration, unsafe/missing selected input and malformed Library
content are invalid input (`64`), not partial success or internal component
failure. A retained source branch that was safely selected but is deferred or
unresolved is a scoped partial result (`2`) with independent diagnostics.

Recommended exit policy:

| Exit | Meaning |
| --- | --- |
| `0` | requested scope complete; no finding at/above `fail_on` |
| `1` | finding at/above `fail_on` |
| `2` | requested scope partial because a required selected input/fact is unavailable |
| `4` | internal/component failure |
| `64` | invalid configuration or usage |
| `130` | cancelled |

A limitation unrelated to the requested scope does not force exit `2`. Exit
precedence is deterministic: invalid input `64`; observed cancellation `130`;
internal/output failure `4`; otherwise a partial result `2`; otherwise a finding
at/above threshold `1`; otherwise `0`. Findings remain in partial output.

<a id="locations-and-document-updates"></a>
<a id="stdio-protocol-behavior"></a>
<a id="lsp-adapter"></a>

## LSP adapter

Use one LSP 3.18 stdio profile and the same configured Project/Service check used
by CLI.

Keep:

- exact unsaved text, document version and root isolation;
- UTF-8/UTF-16 negotiation and checked range conversion;
- increasing nonconsecutive client versions;
- stale/cancelled result rejection;
- standard `null`/empty results for supported no-match requests;
- protocol-only stdout, bounded frames and stderr logging.

Do not build isolated one-file Emmy owners in the adapter. Pass exact overlays to
the shared selected project. Only exact physical or mapped XML user locations are
published; helper/synthetic paths are never diagnostics.

Completion, hover, definition, references, rename, signature help and general
symbols remain upstream EmmyLua unless a retained WoW-specific source/index
operation has a named consumer.

<a id="mcp-adapter"></a>

## MCP adapter

Use one bounded MCP 2025-11-25 stdio profile. Advertise only implemented tools
backed by the same Service owners as CLI:

- addon check;
- direct source body/symbol search;
- readable exact context.

Accept bounded opaque extension `_meta` where the protocol permits it. A valid
unknown notification receives no reply and does not close the session. Malformed
envelopes and unsupported requests remain distinct. JSON-RPC IDs are transport
correlation only.

Do not expose internal profiles, registry/schema digests, effect classes,
Project/report identities, Context packs or durable result receipts.

## Configuration and output safety

- Source/configuration remains user-owned and read-only.
- Paths are resolved once by the frontend and validated by Project.
- Output is bounded and deterministic.
- Primary diagnostics survive scoped limitations.
- Cancellation is advertised only at actual cooperative checkpoints; a
  synchronous native Emmy call is not described as preemptible.

<a id="completion-and-verification"></a>

## Observation

An adapter is complete only after the trace register records the exact command or
framed sequence, input identities and visible result. Build/Clippy and unit owner
construction are not frontend acceptance.

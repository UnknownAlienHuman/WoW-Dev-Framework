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
  [--config <wowdev.json>]
  [--root <addon-dir>]
  [--toc <explicit.toc>]
  [--flavor retail]
  [--library <dir> ...]
  [--format text|json]
  [--fail-on error|warning|information|hint]
```

Later slices may add auto-selection, target Interface, product channel,
dependency roots and selected WoW checks. They must not delay the explicit
physical-Lua command.

CLI precedence and path bases are defined in
[ordinary configuration](IMPLEMENTATION.md#ordinary-configuration). Read at most
one explicit 256 KiB config through a narrow Project helper that opens its parent
and file component-by-component without following links; decode with existing
duplicate-key/depth-bounded `StrictJson`, reject unknown outer fields and duplicate
scalar options. Prevalidate exactly the analysis-relevant pinned
`emmy_diagnostics` keys and compile every `globalsRegex`; reject
`diagnosticInterval` because it is LS-only debounce state. Pinned upstream
otherwise accepts unknown keys and silently drops invalid expressions. If config
cannot be read/decoded, use an explicit CLI
`--format` only after the complete CLI grammar is admitted; otherwise emit the
text error form. The merged configuration must contain root, explicit TOC, exact
flavor `retail` and at least one Library root. CLI scalar options replace
configured scalars; one or more CLI `--library` options replace configured roots,
with at most 64 accepted. Defaults are text output, `fail_on: error` and upstream
default Emmy diagnostics. The explicit TOC is a canonical relative `.toc` path
under the addon root. The frontend resolves display/base paths, then calls one
public Service ordinary-check operation; it does not open addon/Library source,
compute path-authority keys, construct snapshots or attach limitations.

<a id="check-results-and-command-status"></a>

Text and JSON render the same typed result:

- selected scope and completeness;
- diagnostics with exact code, message, severity and user-source range;
- scoped limitations with optional exact source location.

Both formats use one deterministic Service order: diagnostics by role, canonical
path, byte span, code and message; limitations by code, optional location and
message. Filesystem enumeration and derived enum ordering do not affect output.

The direct public result contains no snapshot, analyzer, Project generation,
report, proof or component/capability identities. Paths are canonical relative to
the configured addon root; text rendering may show `root/path`, while JSON keeps
the relative identity the caller supplied with the root. Ambient absolute paths
never enter snapshot/result identity.

Malformed configuration, missing/unreadable addon root or explicit TOC, unsafe
source path/bytes and malformed Library content are invalid input (`64`), not
partial success or internal component failure. A missing direct Lua file named by
a safely parsed TOC, required dependency, unresolved included condition or
deferred selected XML is a scoped partial result (`2`) with independent
diagnostics.

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
precedence is deterministic: invalid input/resource refusal `64`; observed
cancellation `130`; internal/output I/O failure `4`; otherwise a partial result
`2`; otherwise a finding at/above threshold `1`; otherwise `0`. Findings remain
in partial output. Compare severity with an explicit Error > Warning > Information
> Hint ranking, never derived enum ordering. Preserve `Unknown` and treat it as
meeting every threshold because it cannot be proven lower.

A complete or partial `AddonCheckResult` is rendered into a bounded 32 MiB
buffer, then written once to stdout. Configuration, selected-source, Library,
cancellation and internal failures after output format is admitted are rendered
once to stderr. In JSON mode stderr receives one bounded object such as
`{"error":{"code":"invalid_input","message":"...","role":"library","root_index":0,"relative_path":"..."}}`;
stdout stays empty. Error messages are capped at 4 KiB. `root_index` is emitted
only for `role: library` and is zero-based in the merged Library list;
configuration/Main errors omit it. Public operation error codes are exactly
`invalid_input`,
`cancelled`, and `internal`. CLI syntax/option failures before format admission
use one concise text usage error. Human output uses JSON-style visible escaping for untrusted messages; the typed/
JSON result preserves exact strings. Text mode writes one concise error line. If
the fully escaped/encoded result exceeds 32 MiB, emit one `invalid_input` error
(exit `64`) with empty stdout. After encoding, check cancellation once before
the first output byte; a stop there emits only the cancellation error and exits
`130`. Once the write starts, do not replace or append another envelope; finish
the single write or return `4` on I/O failure. Serializer invariants,
cancellation-handler setup or stdout/stderr write failure return `4` without
retrying the operation.

<a id="locations-and-document-updates"></a>
<a id="stdio-protocol-behavior"></a>
<a id="lsp-adapter"></a>

## LSP adapter

Use one LSP 3.18 stdio profile and the same configured Project/Service check used
by CLI.

Keep:

- exact unsaved text, document version and root isolation;
- UTF-8/UTF-16 negotiation and checked range conversion;
- increasing nonconsecutive versions;
- stale/cancelled result rejection;
- standard `null`/empty results for supported no-match requests;
- protocol-only stdout, bounded frames and stderr logging.

Do not build isolated one-file Emmy owners in the adapter. Pass exact overlays to
the same public Service ordinary-check operation used by CLI; never construct the
lower exact-snapshot request or attach limitations in the adapter. Only exact
physical or mapped XML user locations are published; helper/synthetic paths are
never diagnostics.

Completion, hover, definition, references, rename, signature help and general
symbols remain upstream EmmyLua unless a retained WoW-specific source/index
operation has a named consumer.

<a id="mcp-adapter"></a>

## MCP adapter

Use one bounded MCP 2025-11-25 stdio profile. Advertise only implemented tools
backed by the same public Service operations as CLI:

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

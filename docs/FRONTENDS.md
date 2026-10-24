# CLI, editors and agents

Read [current status](CURRENT_STATUS.md). CLI, LSP and MCP are adapters to the
same Service check/search/context owners; they must not create parallel analysis
or lifecycle platforms. General Lua language features remain upstream EmmyLua.

## Current checkpoint

Delivered lower layers:

- `ConfiguredAnalyzerSession` over exact Main/Library/optional virtual snapshots;
- configured code/message/severity/range diagnostics;
- direct `wow-service::addon_check` result over already-acquired snapshots;
- compiled identity and exact Main reuse admission.

Still pending:

- ordinary root/TOC/flavor/Library/dependency acquisition;
- public `wow check` adapter and rendering/exit policy;
- exact XML projection, addon chunk and handler context;
- project-aware LSP reuse;
- useful MCP check/search tools;
- direct current source index/search/context.

The existing `wow` check/status/LSP/MCP command families remain transitional and
must not be presented as the target interface.

## Target user operations

The CLI is the complete offline entry point. It reads local source/configuration
as data, never executes Lua or rewrites user files, and needs no remote service,
editor extension or internal Project identity.

Target command shapes (names may still change):

```text
wow library build --source <checkout> --ref <ref> --flavor <flavor> [--toc <path>] --out <dir>
wow check [--config wowdev.json] [--root <addon>] [--toc <path>] [--flavor <flavor>] [--library <dir>...]
wow index --source <checkout> --ref <ref> --flavor <flavor> --db <path>
wow search --db <path> <query>
wow context --db <path> --hit <id>
wow lsp --transport stdio
wow mcp --transport stdio
```

Search/context is auxiliary. Addon checking must run without a populated index.
Source navigation returns an exact selected hit, readable source and direct known
relations—not a semantic pack or prose-only summary.

### Ordinary configuration

`wowdev.json` uses one small configuration. Config-relative directory paths use
the config directory; CLI-relative directory paths use cwd. Relative TOC uses the
resolved addon root.

```json
{
  "root": "./",
  "toc": "MyAddon.toc",
  "flavor": "retail",
  "library": ["../WoWLibrary"],
  "dependency_roots": ["../SharedAddons"],
  "checks": [],
  "fail_on": "error"
}
```

Optional `target_interface` enables explicit compatibility checking. Optional
`emmy_diagnostics` is the pinned upstream diagnostic object. Repeated CLI
`--library` replaces the configured array. Empty/omitted `checks` means no WoW
rules while Emmy language diagnostics remain enabled.

Unknown check names are configuration errors. A recognized unavailable check
becomes a scoped limitation and preserves independent language findings.

## Check results and command status

The result keeps **findings** separate from **limitations/completeness**. Missing
context or metadata cannot erase a diagnostic or manufacture clean. A disabled
optional rule cannot make a completed type check partial.

The failure threshold defaults to Error:

| Exit | Meaning |
| --- | --- |
| `0` | requested analysis completed; no enabled diagnostic reached threshold |
| `1` | requested analysis completed; at least one enabled diagnostic reached threshold |
| `2` | requested analysis was incomplete or a required capability/input was unavailable; retain findings |
| `64` | invalid command usage/configuration |
| `130` | caller cancellation observed |
| `4` | unexpected internal/serialization/output failure |

Warnings and lower-severity diagnostics remain visible even below threshold.
JSON carries findings and limitations explicitly. Static diagnostics do not prove
runtime load, callback execution, combat/taint safety or complete Secret flow.

`wow-service::addon_check` already provides a direct diagnostics/limitations DTO,
but it consumes exact snapshots and currently reports partial addon context. The
CLI adapter must call it rather than the old generic component result.

## Locations and document updates

Internal source ranges are half-open UTF-8 byte spans over exact captured content.
Displayed line/column positions derive from that content. LSP uses the negotiated
encoding, normally UTF-16, and every published diagnostic is tied to the document
version/result that produced it.

Pinned Emmy diagnostic columns are Unicode scalar counts despite their
LSP-shaped Rust type. Preserve the existing scalar -> UTF-8 byte conversion, then
Project's UTF-8/UTF-16 overlay conversion. Never forward the upstream `character`
field directly, normalize source under the same identity, or apply an old range
to changed text.

Unsaved text stays in memory. Ordered edits apply to successive intermediate
states. Client document versions must increase but may have gaps; internal
revision counters are separate. A stale/rejected/cancelled result cannot clear or
replace newer diagnostics.

Cancellation is truthful and cooperative. Do not claim that an in-progress
synchronous native analyzer call is preemptible.

## CLI adapter

The CLI owns only:

- arguments and config-file precedence;
- ordinary input validation before Service;
- cancellation wiring;
- text/JSON rendering;
- exit status.

It does not construct another Emmy owner, Reference/Rules for a type-only run,
Project publication, graph/store state, or compatibility reports. Human logs go
to stderr; stdout contains the requested result. Serialization/output failure
must not emit a second success-shaped result or replay a completed mutation.

## LSP adapter

Use one LSP 3.18 stdio profile. Framing uses `Content-Length` and CRLF headers;
body is UTF-8 JSON. Stdout is protocol-only and logs/startup errors use stderr.
Unsupported requests return protocol errors; valid no-match operations return
standard `null`/empty results.

Keep overlay responsibilities—exact unsaved text, client version, internal
revision, root isolation, edit validation and UTF conversion—but pass an exact
text override into the same configured addon-check owner. Do not build a second
one-file `LuaWorkspaceSnapshot` in the frontend.

General completion/hover/definition/references/rename/signature/symbols remain
upstream EmmyLua. Expose framework navigation only for WoW-specific selected
index/context behavior. An evaluated empty reference result is valid; the current
application error on empty `locations` must disappear when the caller moves.

Publish diagnostics only for the exact document version/result. Accept version
3 -> 5; reject stale versions/results. Preserve related exact Library/XML
locations and hide synthetic context-helper spans.

## MCP adapter

Use one MCP 2025-11-25 stdio profile with one UTF-8 JSON-RPC message per line.
Stdout is protocol-only. Advertise only implemented useful tools backed by the
same Service owners as CLI:

- configured addon check;
- direct body/symbol search;
- readable located context, when implemented;
- concise readiness only if a real operation needs it.

Accept bounded opaque `_meta` where MCP permits extensions. A valid unknown
notification receives no response and does not close the session. Malformed or
unsupported requests remain errors without corrupting later state. No profile
families, registry/schema digests, resource manifests without real resources,
generic invoke surface or Context pack continuation machinery.

## Source query adapters

A query selects exact addon/UI/API scope and returns located typed hits. Equal
names in different scopes remain distinct. Complete zero matches differs from
missing/partial index or truncation. Context quotes exact indexed source and
finite direct known relations; it never applies stale spans to newer disk text.

CLI/LSP/MCP serialize the same typed query/context result once. Protocol request
IDs remain transport correlation, not durable content identities.

## Verification boundary

No frontend is complete until its ordinary input reaches the real owner and a
useful result is observed. For each final adapter record an exact command or
framed sequence, input identities, result and cancellation/empty/error behavior
in [IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md). Build,
Clippy and source review are not protocol or product observations.
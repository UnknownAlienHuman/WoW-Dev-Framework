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

- ordinary root/TOC/flavor/product-channel/Library acquisition;
- dependency closure, reverse triggers and exact selected order/context;
- implicit bindings discovery and callback mapping;
- public `wow check` adapter, rendering and exit policy;
- addon chunk/UI handler context and exact XML/binding projection;
- project-aware LSP reuse;
- useful MCP check/search tools;
- direct current source index/search/context.

Existing `wow` check/status/LSP/MCP families are transitional and must not be
presented as the target interface.

## Target user operations

The CLI is the complete offline entry point. It reads local source/configuration
as data, never executes Lua or rewrites user files, and needs no remote service,
editor extension or internal Project identity.

Target shapes (names may still change):

```text
wow library build --source <checkout> --ref <ref> --flavor <flavor> [--toc <path>] --out <dir>
wow check [--config wowdev.json] [--root <addon>] [--toc <path>] [--flavor <flavor>] [--channel <stable|ptr|beta>] [--library <dir>...]
wow index --source <checkout> --ref <ref> --flavor <flavor> --channel <channel> --db <path>
wow search --db <path> <query>
wow context --db <path> --hit <id>
wow lsp --transport stdio
wow mcp --transport stdio
```

Search/context is auxiliary. Checking must run without an index. Navigation
returns an exact selected hit, readable source and direct known relations—not a
pack or prose-only summary.

### Ordinary configuration

Config-relative directory paths use the config directory; CLI-relative directory
paths use cwd. Relative TOC uses resolved addon root.

```json
{
  "root": "./",
  "toc": "MyAddon.toc",
  "flavor": "retail",
  "channel": "stable",
  "library": ["../WoWLibrary"],
  "dependency_roots": ["../SharedAddons"],
  "checks": [],
  "fail_on": "error"
}
```

`channel` is Stable/PTR/Beta product selection, not flavor, Interface,
game/glue or secure/global context. A normal explicitly selected installed-client
target may default to Stable; otherwise channel is required when source uses a
channel gate such as `OnlyBetaAndPTR`. Never infer it from repository ref/path.

Optional `target_interface` enables explicit compatibility checking. Optional
`emmy_diagnostics` is the pinned upstream diagnostic object. Repeated CLI
`--library` replaces the configured array. Empty/omitted `checks` means no WoW
rules while Emmy diagnostics remain enabled.

Dependency aliases, selected source metadata, secure/private-table phases and
selected order are Project-owned facts. `LoadWith` and implicit binding adapters
also remain Project-owned, but their client-authoritative versus donor-derived
evidence class must stay explicit. Frontends do not ask users for internal
receipts or reconstruct any of these values from filenames.

Unknown check names are configuration errors. A recognized unavailable check is
a scoped limitation and preserves independent findings.

## Check results and command status

Keep **findings** separate from **limitations/completeness**. Missing context or
metadata cannot erase a diagnostic or manufacture clean. A disabled optional rule
cannot make type checking partial.

| Exit | Meaning |
| --- | --- |
| `0` | requested analysis completed; no enabled diagnostic reached threshold |
| `1` | completed; at least one enabled diagnostic reached threshold |
| `2` | incomplete or required input/capability unavailable; retain findings |
| `64` | invalid usage/configuration |
| `130` | caller cancellation observed |
| `4` | unexpected internal/serialization/output failure |

Warnings and lower severity remain visible below threshold. JSON carries
findings and limitations explicitly. Static diagnostics do not prove runtime
load, trigger/callback execution, combat/taint safety or complete Secret flow.

`wow-service::addon_check` already supplies the direct DTO, but consumes exact
snapshots and currently reports partial addon context. CLI must call it instead of
the old generic component result.

## Locations and document updates

Internal ranges are half-open UTF-8 bytes over exact captured content. Displayed
positions derive from that content. LSP uses negotiated encoding and binds every
diagnostic to the exact document/result version.

Pinned Emmy columns are Unicode scalar counts despite LSP-shaped types. Preserve
scalar -> UTF-8 byte conversion, then Project UTF-8/UTF-16 conversion. Never
forward upstream `character` directly or apply old ranges to changed text.

Unsaved text stays in memory. Ordered edits apply to successive states. Client
versions increase but may have gaps; internal revisions are separate. Stale,
rejected or cancelled results cannot replace newer diagnostics.

Cancellation is truthful/cooperative. Do not call a synchronous native analyzer
call preemptible.

<a id="stdio-protocol-behavior"></a>

## CLI adapter

CLI owns arguments/config precedence, ordinary input validation, cancellation,
text/JSON rendering and exit status. It does not construct another Emmy owner,
Reference/Rules for type-only work, Project publication, graph/store or reports.
Human logs use stderr; stdout contains the requested result. Output failure must
not emit a second success or replay a completed mutation.

## LSP adapter

Use one LSP 3.18 stdio profile. `Content-Length`/CRLF framing, UTF-8 JSON body,
protocol-only stdout and stderr logging. Unsupported requests return protocol
errors; valid no-match returns standard null/empty.

Keep overlay text/version/revision/root/edit/coordinate ownership, but pass exact
overrides into the same configured addon-check project. Do not create a second
one-file snapshot in the frontend.

General completion/hover/definition/references/rename/signature/symbols remain
upstream EmmyLua. Framework navigation is only WoW-specific selected index/context.
Empty references is valid. Publish only exact document-version diagnostics;
accept 3 -> 5 and reject stale results. Preserve related exact Library/XML/
binding locations and hide synthetic wrappers.

## MCP adapter

Use one MCP 2025-11-25 stdio profile with one UTF-8 JSON-RPC message per line and
protocol-only stdout. Advertise only implemented tools backed by the same Service
owners: configured check, direct body/symbol search and readable context when
implemented.

Accept bounded opaque `_meta` where permitted. Valid unknown notification gets no
response and does not close the session. Malformed/unsupported requests are
errors without corrupting later state. No profile families, generic invoke,
resource manifests without resources or Context pack continuation platform.

## Source query adapters

A query selects exact addon/UI/API scope and exact revision/flavor/channel/context.
Equal names in different scopes remain distinct. Complete zero matches differs
from missing/partial index or truncation. Context quotes exact indexed source and
finite direct known dependency/trigger/load/callback/type/reference relations;
it never applies stale spans to newer text.

CLI/LSP/MCP serialize the same typed result once. Request IDs are transport
correlation, not durable content identities.

## Verification boundary

A frontend is incomplete until ordinary input reaches the real owner and a useful
result is observed. Record exact command/framed sequence, input identities,
result and cancellation/empty/error behavior in
[IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md). Build,
Clippy and source review are not protocol/product observations.

# CLI, editors and agents

This document separates the product target from the implementation at the
reviewed `c36a058` baseline. The target follows the [architecture](ARCHITECTURE.md)
and [implementation routes](IMPLEMENTATION.md); the [audit](AUDIT.md) and
[queue](REFACTOR_QUEUE.md) describe remaining work. No public workflow is
considered complete until its ordinary input reaches its real reader and useful
result.

## Target user operations

The CLI is the complete local entry point. It must work offline from local
project/source files and a local annotation Library, without a VS Code extension,
Lua interpreter, remote service, or internal project IDs. It does not execute
source files or rewrite user source/configuration.

The ordinary check accepts an addon root, optional TOC selection, client flavor,
and Library directory. Defaults should discover an unambiguous TOC and use
reasonable bounded work/output limits. Missing, ambiguous, unsupported, or
partially analyzed inputs must be visible. The ordinary path must not require a
compatibility report, probe report, contract ID, digest, generation ID, receipt,
or budget JSON.

CLI, LSP, and MCP are adapters to the same configured check owner and result
model. Search and readable source context may be exposed through those same
frontends as their backing index becomes available. General Lua completion,
hover, and other language features remain with upstream EmmyLua; see
[EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md).

Target command shapes (names and flags may still change):

```text
wow library build --source <checkout> --ref <ref> --flavor <flavor> [--toc <path>] --out <dir>
wow check [--config wowdev.json] [--root <addon>] [--toc <path>] [--flavor <flavor>] --library <dir>
wow index --source <checkout> --ref <ref> --flavor <flavor> --db <path>
wow search --db <path> <query>
wow context --db <path> --hit <id>
wow lsp --transport stdio
wow mcp --transport stdio
```

The optional target `wowdev.json` keeps ordinary setup in one small file. Paths
are resolved relative to the config file's directory:

```json
{
  "root": "./",
  "toc": "MyAddon.toc",
  "flavor": "retail",
  "library": "../WoWLibrary",
  "dependency_roots": ["../SharedAddons"],
  "checks": ["api-exists"]
}
```

The config is optional when equivalent CLI options are provided. Dependency
roots and selected checks are optional; no Project IDs or generated reports are
part of this target configuration.

## Check results and command status

The target result keeps findings separate from completeness. A finding is a
problem reported at the addon source; incomplete analysis describes what could
not be checked or which input/capability was unavailable. Partial coverage must
not erase a finding or manufacture a clean result. JSON output carries both
fields explicitly; text output states both in readable terms.

The target CLI exit policy is:

| Exit | Meaning |
| --- | --- |
| `0` | Requested analysis completed; no enabled diagnostic at or above the configured failure threshold was reported. |
| `1` | Requested analysis completed; at least one enabled diagnostic reached the configured failure threshold. |
| `2` | Analysis was incomplete or a requested input/capability was unavailable; retain and report any findings too. |
| `64` | Command usage or configuration is invalid. |
| `130` | The user cancelled the operation. |
| `4` | Unexpected internal, serialization, or output failure. |

Warnings and lower-severity findings are still rendered even when they do not
reach the configured failure threshold. Findings and incompleteness remain
separately inspectable in JSON regardless of exit code. The target policy does
not claim that all diagnostics or runtime semantics are currently supported.
Static results do not prove that an addon loads, that an event or callback
fires, or that code is safe in combat or under taint restrictions.

At `c36a058`, `wow status` and `wow check` require `--config`; checking also
requires a Project ID and accepts internal generation, file, and rule selectors.
The executable exposes many materialized-input and platform commands. Those are
development-era surfaces, not the target interface. Inspect the current command
list with `cargo run --locked -p wow-cli --bin wow -- --help`; use that output
for current syntax until ordinary root/TOC/flavor/Library configuration lands.
The current output mapping is implemented in
[`apps/wow/src/output.rs`](../apps/wow/src/output.rs); it is not the target
policy above.

## Locations and document updates

Locations must identify the exact source content analyzed. Internally, source
ranges use UTF-8 byte offsets over the captured bytes; displayed line and column
positions are derived from that content. LSP positions must use the negotiated
encoding (UTF-16 when that is the protocol default), and every diagnostic must
be associated with the document version that produced it. Do not apply an old
range to changed text.

Unsaved editor buffers stay in memory. Ordered edits and document versions are
validated before updating the shared owner. A stale, rejected, or cancelled
analysis must not publish diagnostics or edits over a newer document version.
Cancellation is cooperative: report it only when the operation observes it, and
do not claim that a synchronous analyzer call can be preempted. Keep work,
framing, and output bounded. Relevant current implementation lives under
[`crates/wow-service/src/frontend/lsp`](../crates/wow-service/src/frontend/lsp)
and Project update ownership under
[`crates/wow-project/src`](../crates/wow-project/src).

## Stdio protocol behavior

LSP 3.18 uses JSON-RPC messages with `Content-Length` headers and CRLF header
lines; the body is UTF-8 JSON. Stdout contains framed protocol messages only.
Logs and startup errors belong on stderr. Return the method's standard
empty/null shape when a valid request has no result, and return protocol errors
for unsupported methods rather than fabricating empty success. See the
[LSP 3.18 specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/).

MCP 2025-11-25 stdio uses one UTF-8 JSON-RPC message per line. Stdout is reserved
for protocol traffic. A valid unknown notification receives no response and
must not terminate the session. Accept bounded extensible metadata, and expose
only tools the selected implementation actually supports. Both adapters
preserve request/response framing, bounded input/output, clean shutdown, and
cooperative cancellation where the owner observes it. See the
[MCP stdio transport specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports#stdio).

These are target requirements, not claims that the current frontends meet them.
At `c36a058`, LSP requires an explicit host and selects one of private profiles;
MCP exposes older profile-specific tools. Current LSP implementation and framing
are in [`apps/wow/src/lsp.rs`](../apps/wow/src/lsp.rs) and
[`crates/wow-service/src/frontend/lsp`](../crates/wow-service/src/frontend/lsp);
MCP dispatch is in [`apps/wow/src/mcp.rs`](../apps/wow/src/mcp.rs) and its
Service adapter under [`crates/wow-service/src/frontend`](../crates/wow-service/src/frontend).
